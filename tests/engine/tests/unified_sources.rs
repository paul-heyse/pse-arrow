// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Real package sources survive target publication without a source-object store.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration assertions"
)]

use datafusion::{
    common::ResolvedTableReference,
    execution::{context::SessionContext, session_state::SessionStateBuilder},
    physical_plan::collect,
};
use pse_authoring::{ParseBudget, document::load_package_texts_owned};
use pse_catalog::{
    delta::{
        provider::table_builder,
        publication::{Publication, PublicationRoot},
        publication_plan::{self, MemberWrite},
    },
    session::planner::UnifiedPlanner,
};
use pse_ids::{CancellationToken, FixedBudget, SemanticId};
use pse_relations::generated::{
    authored::documents, enums::PublicationKind, runtime::publications,
};
use std::{collections::BTreeMap, sync::Arc};

fn native_context() -> SessionContext {
    SessionContext::new_with_state(
        SessionStateBuilder::new_with_default_features()
            .with_query_planner(Arc::new(UnifiedPlanner::default()))
            .build(),
    )
}
fn texts() -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "package.toml".into(),
            include_str!("../../fixtures/packages/minimal_explicit/package.toml").to_owned(),
        ),
        (
            "materials/species.yaml".into(),
            format!(
                "# exact UTF-8 provenance: 水, ΔT\r\n{}",
                include_str!("../../fixtures/packages/minimal_explicit/materials/species.yaml")
            ),
        ),
    ])
}
fn header() -> publications::Row {
    let id = |value| SemanticId::from_bytes([value; 16]);
    publications::Row {
        workspace_id: id(1),
        publication_id: id(2),
        parent_publication_id: None,
        attempt_id: id(3),
        kind: PublicationKind::Source,
        inputs: vec![],
        members: vec![],
    }
}

#[tokio::test]
async fn exact_source_text_reopens_and_reparses_from_delta_alone() {
    let registry = pse_schema::shared_registry().unwrap();
    let sources = texts();
    let budget = FixedBudget::new(128 << 20);
    let cancel = CancellationToken::default();
    let loaded = load_package_texts_owned(
        &sources,
        &registry,
        ParseBudget::default(),
        budget.as_ref(),
        &cancel,
    )
    .unwrap();
    let expected = loaded.bundle().batches.clone();
    let context = native_context();
    let state = context.state();
    let root = tempfile::tempdir().unwrap();
    let source_input = context
        .read_batch(expected[&documents::RELATION_ID].batch().clone())
        .unwrap()
        .into_unoptimized_plan();
    let mut members = vec![];
    for id in expected.keys() {
        let spec = registry.relation_by_id(*id).unwrap();
        let location = format!("file://{}/{id}/", root.path().display())
            .parse()
            .unwrap();
        members.push(MemberWrite {
            reference: ResolvedTableReference {
                catalog: "source".into(),
                schema: spec.key.namespace.as_str().into(),
                table: spec.key.name.into(),
            },
            relation_id: *id,
            table: table_builder(location, &state).unwrap().build().unwrap(),
            input: pse_authoring::native::relation_plan(
                source_input.clone(),
                *id,
                &registry,
                ParseBudget::default(),
                budget.clone(),
                cancel.clone(),
            )
            .unwrap(),
        });
    }
    let selection = PublicationRoot {
        location: format!("file://{}/control/", root.path().display())
            .parse()
            .unwrap(),
        version: 0,
    };
    let plan = publication_plan::plan(
        selection.location.clone(),
        header(),
        members,
        Arc::clone(&registry),
    )
    .unwrap();
    collect(
        state.create_physical_plan(&plan).await.unwrap(),
        state.task_ctx(),
    )
    .await
    .unwrap();
    drop((plan, state, context, loaded));

    let cold = native_context();
    let publication = Publication::open(selection, &registry, Arc::new(cold.state()))
        .await
        .unwrap();
    let reopened = SessionContext::new_with_state(publication.session_state().await.unwrap());
    let stream = publication
        .relation_stream(&ResolvedTableReference {
            catalog: "source".into(),
            schema: "authored".into(),
            table: "documents".into(),
        })
        .await
        .unwrap();
    let facts = capture_source_facts(&publication, &registry, &cancel).await;
    let selected = facts.selection().unwrap().clone();
    drop(publication);
    verify_native_support(facts, selected, Arc::clone(&registry), &cancel).await;
    let batches = datafusion::physical_plan::common::collect(stream)
        .await
        .unwrap();
    let mut recovered = BTreeMap::new();
    for batch in batches {
        let view = documents::View::try_from_batch(&batch).unwrap();
        for row in view.rows().unwrap() {
            recovered.insert(row.path, row.source_text);
        }
    }
    assert_eq!(recovered, sources);
    let reparsed = load_package_texts_owned(
        &recovered,
        &registry,
        ParseBudget::default(),
        budget.as_ref(),
        &cancel,
    )
    .unwrap();
    assert_eq!(reparsed.bundle().batches.len(), expected.len());
    for (id, batch) in &expected {
        assert_eq!(reparsed.bundle().batches[id].batch(), batch.batch());
    }
    assert!(!root.path().join("documents").exists());
    assert_eq!(
        reopened
            .sql("SELECT name FROM source.authored.species")
            .await
            .unwrap()
            .count()
            .await
            .unwrap(),
        1
    );
}

async fn capture_source_facts(
    publication: &Publication,
    registry: &Arc<pse_schema::Registry>,
    cancel: &CancellationToken,
) -> pse_catalog::session::RelationFacts {
    use pse_catalog::session::{
        ExecutionSettings, SessionFactory, ThreadBudget, native_engine_profile,
    };
    use pse_schema::model::provider::{OperationEffect, ProviderPolicy, ProviderScope};
    let factory = SessionFactory::new(
        Arc::new(datafusion::execution::runtime_env::RuntimeEnv::default()),
        FixedBudget::new(128 << 20),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: 1.try_into().unwrap(),
            target_partitions: 1.try_into().unwrap(),
        },
        native_engine_profile(),
    )
    .unwrap();
    let session = factory
        .open_publication(publication.root().clone(), Arc::clone(registry), cancel)
        .await
        .unwrap()
        .with_purpose(pse_schema::model::provider::OperationPurpose::Inspect);
    let queried = session
        .sql("SELECT source_text FROM source.authored.documents", cancel)
        .await
        .unwrap();
    assert_eq!(
        queried.iter().map(|batch| batch.num_rows()).sum::<usize>(),
        2
    );
    let reference = ResolvedTableReference {
        catalog: "source".into(),
        schema: "authored".into(),
        table: "documents".into(),
    };
    let mut policy = ProviderPolicy::new(
        SemanticId::from_bytes([99; 16]),
        ProviderScope::Schema("source".into(), "authored".into()),
    );
    policy.effects.remove(&OperationEffect::Read);
    let refused = session
        .with_policies([policy])
        .unwrap()
        .capture_relation(&reference, cancel)
        .await
        .unwrap_err();
    assert!(
        refused
            .to_string()
            .contains("outside the effective purpose/policy"),
        "{refused}"
    );
    let cancelled = CancellationToken::default();
    cancelled.cancel();
    assert!(
        session
            .capture_relation(&reference, &cancelled)
            .await
            .is_err()
    );
    let facts = session.capture_relation(&reference, cancel).await.unwrap();
    assert_eq!(
        facts.selection().unwrap(),
        publication.member(&reference).unwrap()
    );
    facts
}

async fn verify_native_support(
    facts: pse_catalog::session::RelationFacts,
    selection: publications::RuntimePublicationsFieldMembersItem,
    registry: Arc<pse_schema::Registry>,
    cancel: &CancellationToken,
) {
    use datafusion::logical_expr::LogicalPlanBuilder;
    use pse_catalog::session::{
        ExecutionSettings, SessionFactory, ThreadBudget, native_engine_profile,
    };
    use pse_relations::typed::CellCodec;
    use pse_rules::strata::{
        LocatedRuleInput, RuleInputLocation,
        native_input::{NativeInput, NativeWitness},
    };
    let declaration = registry.relation_by_id(documents::RELATION_ID).unwrap();
    let factory = SessionFactory::new(
        Arc::new(datafusion::execution::runtime_env::RuntimeEnv::default()),
        FixedBudget::new(128 << 20),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: 1.try_into().unwrap(),
            target_partitions: 1.try_into().unwrap(),
        },
        native_engine_profile(),
    )
    .unwrap();
    let session = factory
        .candidate_checked(
            BTreeMap::from([(declaration.key, facts.checked().clone())]),
            Arc::clone(&registry),
            cancel,
        )
        .unwrap();
    let plan = LogicalPlanBuilder::scan(
        "exact_documents",
        session.table_source(&declaration.key).unwrap(),
        None,
    )
    .unwrap()
    .build()
    .unwrap();
    let result = NativeInput::build(
        plan,
        declaration.key,
        SemanticId::NIL,
        declaration
            .columns
            .iter()
            .map(|field| (field.name().to_owned(), field.name().to_owned()))
            .collect(),
        vec![NativeWitness {
            port: "documents".into(),
            input: LocatedRuleInput {
                relation: declaration.key,
                location: RuleInputLocation::Facts(Arc::new(facts)),
            },
            key_columns: declaration
                .primary_key
                .iter()
                .map(|key| (*key).to_owned())
                .collect(),
            when: None,
        }],
        &session,
        cancel,
    )
    .await
    .unwrap();
    drop((session, factory));
    let supports = registry
        .relation("provenance.constructed_supports")
        .unwrap();
    let rows = pse_relations::cells::cells_from_batch(
        &registry,
        supports,
        result.support_mapping().batch(),
    )
    .unwrap();
    assert_eq!(rows.len(), 2);
    let expected = selection.into_cell();
    for row in rows {
        assert_eq!(row[5], expected);
        assert_eq!(row[6], pse_schema::model::Cell::Enum("delta"));
    }
}
