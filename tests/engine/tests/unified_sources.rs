// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Real package sources survive target publication without a source-object store.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration assertions"
)]

use datafusion::{common::ResolvedTableReference, logical_expr::LogicalPlanBuilder};
use pse_authoring::ParseBudget;
use pse_catalog::{
    artifact::{ArtifactPlan, PublicationTarget, RelationOutput},
    delta::publication::{Publication, PublicationRoot},
};
use pse_columnar::CancellationToken;
use pse_engine::session::planner::UnifiedPlanner;
use pse_ids::SemanticId;
use pse_relations::generated::{
    authored::documents, enums::PublicationKind, runtime::publications,
};
use pse_runtime::authoring_driver::document::load_package_texts_owned;
use std::{collections::BTreeMap, sync::Arc};

fn native_fixture() -> pse_testkit::NativeFixture {
    let mut fixture = pse_testkit::NativeFixture::with_settings(
        (128 << 20).try_into().unwrap(),
        pse_engine::ThreadBudget {
            pool_threads: 1.try_into().unwrap(),
            target_partitions: 1.try_into().unwrap(),
        },
        pse_engine::ExecutionSettings::default(),
        pse_engine::cache_service::CacheBudget::disabled(128 << 20),
    )
    .unwrap();
    fixture.factory = fixture
        .factory
        .with_query_planner(Arc::new(UnifiedPlanner::new(
            pse_catalog::assembly::planners(),
        )));
    fixture
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
        kind: PublicationKind::Relations,
        inputs: vec![],
        members: vec![],
    }
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "complete target fixture construction and its independent assertions are kept in execution order"
)]
async fn exact_source_text_reopens_and_reparses_from_delta_alone() {
    let registry = pse_schema::shared_registry().unwrap();
    let sources = texts();
    let fixture = native_fixture();
    let budget = fixture.resources.pool.clone();
    let cancel = CancellationToken::default();
    let loaded = load_package_texts_owned(
        &sources,
        &registry,
        ParseBudget::default(),
        &budget,
        &cancel,
    )
    .unwrap();
    let expected = loaded.bundle().batches.clone();
    let factory = fixture.factory.clone();
    let session = factory
        .candidate_checked(
            BTreeMap::from([(
                documents::RELATION_KEY,
                expected[&documents::RELATION_ID].clone(),
            )]),
            Arc::clone(&registry),
            &cancel,
        )
        .unwrap();
    let root = tempfile::tempdir().unwrap();
    let source_input = LogicalPlanBuilder::scan(
        "source_documents",
        session.table_source(&documents::RELATION_KEY).unwrap(),
        None,
    )
    .unwrap()
    .build()
    .unwrap();
    let mut outputs = BTreeMap::new();
    let mut destinations = BTreeMap::new();
    for id in expected.keys() {
        let spec = registry.relation_by_id(*id).unwrap();
        let location = format!("file://{}/{id}/", root.path().display())
            .parse()
            .unwrap();
        let reference = ResolvedTableReference {
            catalog: "source".into(),
            schema: spec.key.namespace.as_str().into(),
            table: spec.key.name.into(),
        };
        destinations.insert(reference.clone(), location);
        outputs.insert(
            reference,
            RelationOutput {
                relation_id: *id,
                plan: pse_runtime::authoring_driver::native::relation_plan(
                    source_input.clone(),
                    *id,
                    &registry,
                    ParseBudget::default(),
                    budget.clone(),
                    cancel.clone(),
                )
                .unwrap(),
            },
        );
    }
    let selection = PublicationRoot {
        location: format!("file://{}/control/", root.path().display())
            .parse()
            .unwrap(),
        version: 1,
    };
    let plan = ArtifactPlan::new(session, outputs, &cancel).unwrap();
    plan.prepare_publication(
        PublicationTarget {
            reference: ResolvedTableReference {
                catalog: "source".into(),
                schema: "runtime".into(),
                table: "publications".into(),
            },
            location: selection.location.clone(),
        },
        header(),
        destinations,
        vec![],
        &cancel,
    )
    .unwrap()
    .execute(&cancel)
    .await
    .unwrap();
    drop((plan, factory, fixture, loaded));

    let cold = native_fixture();
    let budget = cold.resources.pool.clone();
    let factory = cold.factory.clone();
    let publication = Publication::open(selection, Arc::clone(&registry), &factory, &cancel)
        .await
        .unwrap();
    let reopened = publication.session().clone();
    let mut stream = publication
        .relation_stream(
            &ResolvedTableReference {
                catalog: "source".into(),
                schema: "authored".into(),
                table: "documents".into(),
            },
            &cancel,
        )
        .await
        .unwrap();
    let facts = capture_source_facts(&publication, &registry, &cancel).await;
    let selected = facts
        .witness()
        .and_then(|w| w.value::<publications::RuntimePublicationsFieldMembersItem>())
        .unwrap()
        .clone();
    drop(publication);
    assert_eq!(selected.relation_id, documents::RELATION_ID);
    drop(facts);
    let mut batches = Vec::new();
    while let Some(batch) = stream.next_batch(&cancel).await.unwrap() {
        batches.push(batch.into_batch());
    }
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
        &budget,
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
            .sql("SELECT name FROM source.authored.species", &cancel)
            .await
            .unwrap()
            .iter()
            .map(arrow::array::RecordBatch::num_rows)
            .sum::<usize>(),
        1
    );
}

async fn capture_source_facts(
    publication: &Publication,
    registry: &Arc<pse_schema::Registry>,
    cancel: &CancellationToken,
) -> pse_engine::session::RelationFacts {
    use pse_engine::session::{ExecutionSettings, ThreadBudget};
    use pse_schema::model::provider::{OperationEffect, ProviderPolicy, ProviderScope};
    let factory = pse_testkit::factory(
        Arc::new(pse_columnar::GreedyMemoryPool::new(128 << 20)),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: 1.try_into().unwrap(),
            target_partitions: 1.try_into().unwrap(),
        },
    )
    .unwrap();
    let session = Publication::open(
        publication.root().clone(),
        Arc::clone(registry),
        &factory,
        cancel,
    )
    .await
    .unwrap()
    .into_session()
    .with_purpose(pse_schema::model::provider::OperationPurpose::Inspect);
    let queried = session
        .sql("SELECT source_text FROM source.authored.documents", cancel)
        .await
        .unwrap();
    assert_eq!(
        queried
            .iter()
            .map(arrow::array::RecordBatch::num_rows)
            .sum::<usize>(),
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
        facts
            .witness()
            .and_then(|w| w.value::<publications::RuntimePublicationsFieldMembersItem>())
            .unwrap(),
        &publication.member(&reference).unwrap()
    );
    facts
}
