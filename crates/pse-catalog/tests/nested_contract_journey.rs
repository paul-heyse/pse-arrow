// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! A new nested declaration crosses local, native predicate, Delta and publication paths.
#![allow(
    clippy::unwrap_used,
    reason = "independent integration fixture assertions"
)]
use datafusion::{
    arrow::{
        array::{Array, Int64Array, ListArray, RecordBatch},
        datatypes::{DataType, Int64Type},
    },
    common::ResolvedTableReference,
    datasource::{MemTable, provider_as_source},
    logical_expr::{LogicalPlanBuilder, col},
};
use pse_columnar::CancellationToken;
use pse_ids::SemanticId;
use pse_schema::{
    RegistryBuilder,
    model::{
        Authority, CollectionContract, FieldContract as F, Namespace, RelationDecl, SnapshotClass,
    },
};
use std::{collections::BTreeMap, sync::Arc};

fn name(schema: &str, table: &str) -> ResolvedTableReference {
    ResolvedTableReference {
        catalog: "contract".into(),
        schema: schema.into(),
        table: table.into(),
    }
}
#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "one declared nested constraint is followed across all authority boundaries"
)]
async fn new_collection_constraint_has_one_decision_across_local_native_delta_and_publication() {
    let mut declarations = RegistryBuilder::new();
    pse_schema::catalog::declare_publications(&mut declarations);
    declarations.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "bounded_values",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "A nested set with a declared bound",
        )
        .pk(&["id"])
        .columns(vec![
            F::key("id", F::native(DataType::Int64), "Key"),
            F::payload(
                "values",
                F::list(F::native(DataType::Int64)).with_collection(CollectionContract {
                    minimum: 1,
                    maximum: Some(2),
                    ..CollectionContract::SET
                }),
                "Values",
            ),
        ]),
    );
    let registry = Arc::new(declarations.build().unwrap());
    let spec = registry.relation("authored.bounded_values").unwrap();
    let schema = Arc::new(pse_schema::arrow::relation_schema(&registry, spec).unwrap());
    let factory = pse_testkit::NativeFixture::new((128 << 20).try_into().unwrap())
        .unwrap()
        .into_factory()
        .with_query_planner(Arc::new(pse_engine::session::planner::UnifiedPlanner::new(
            pse_catalog::assembly::planners(),
        )));
    let state = factory.native_state();
    let context = pse_relations::validate::ValidationContext::new(
        &registry,
        pse_engine::validation::NativeValidation(state.clone()),
    );
    let local = context.relation(&registry, spec).unwrap();
    let declared = pse_catalog::delta::contract::DeclaredCheck::new(&registry, spec.id).unwrap();
    let directory = tempfile::tempdir().unwrap();
    let base = url::Url::from_directory_path(directory.path()).unwrap();
    let cancel = CancellationToken::new();
    for (index, (values, valid)) in [
        (vec![1, 2], true),
        (vec![1, 1], false),
        (vec![], false),
        (vec![1, 2, 3], false),
    ]
    .into_iter()
    .enumerate()
    {
        let array = ListArray::from_iter_primitive::<Int64Type, _, _>([Some(
            values.into_iter().map(Some).collect::<Vec<_>>(),
        )]);
        // Preserve the declared child field (including non-nullability and metadata).
        let data = array.into_data();
        let array = datafusion::arrow::array::make_array(
            data.into_builder()
                .data_type(schema.field(1).data_type().clone())
                .build()
                .unwrap(),
        );
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(Int64Array::from(vec![1])), array],
        )
        .unwrap();
        let report = local.evaluate(&batch, 16, &cancel).unwrap();
        assert_eq!(report.is_valid(), valid);
        assert!(!report.truncated);
        let input = LogicalPlanBuilder::scan(
            "nested",
            provider_as_source(Arc::new(
                MemTable::try_new(schema.clone(), vec![vec![batch.clone()]]).unwrap(),
            )),
            None,
        )
        .unwrap()
        .build()
        .unwrap();
        let predicate = pse_relations::validate::predicates::field_value(
            &registry,
            schema.field(1),
            col("values"),
            0,
        )
        .unwrap();
        let queried = LogicalPlanBuilder::from(input.clone())
            .project([predicate.alias("valid")])
            .unwrap()
            .build()
            .unwrap();
        let native = pse_testkit::execution::run(state, registry.clone(), &queried)
            .await
            .unwrap();
        assert_eq!(
            native[0]
                .column(0)
                .as_any()
                .downcast_ref::<datafusion::arrow::array::BooleanArray>()
                .unwrap()
                .value(0),
            valid
        );
        let location = base.join(&format!("values-{index}/")).unwrap();
        let write = pse_catalog::delta::write::DeltaWrite::declared(
            deltalake::DeltaTableBuilder::from_url(location.clone())
                .unwrap()
                .build()
                .unwrap(),
            input,
            deltalake::protocol::SaveMode::ErrorIfExists,
            deltalake::kernel::transaction::CommitProperties::default(),
            declared.clone(),
        )
        .unwrap();
        let written = pse_testkit::execution::run(state, registry.clone(), &write).await;
        assert_eq!(written.is_ok(), valid);
        if !valid {
            continue;
        }
        let table = deltalake::DeltaTableBuilder::from_url(location)
            .unwrap()
            .load()
            .await
            .unwrap();
        declared.verify(&table).unwrap();
        let session = factory
            .candidate(
                BTreeMap::from([(spec.key, batch)]),
                registry.clone(),
                &cancel,
            )
            .unwrap();
        let reference = session
            .table_reference(&spec.key)
            .unwrap()
            .clone()
            .resolve("model", "authored");
        let relation = session.relation_plan(&reference).unwrap();
        let artifact = pse_catalog::artifact::ArtifactPlan::new(
            session,
            BTreeMap::from([(
                name("authored", "bounded_values"),
                pse_catalog::artifact::RelationOutput {
                    relation_id: spec.id,
                    plan: relation.plan().clone(),
                },
            )]),
            &cancel,
        )
        .unwrap();
        let control = base.join("publication/").unwrap();
        let prepared = artifact
            .prepare_publication(
                pse_catalog::artifact::PublicationTarget {
                    reference: name("runtime", "publications"),
                    location: control.clone(),
                },
                pse_relations::generated::runtime::publications::Row {
                    workspace_id: SemanticId::from_bytes([1; 16]),
                    publication_id: SemanticId::from_bytes([2; 16]),
                    parent_publication_id: None,
                    attempt_id: SemanticId::from_bytes([3; 16]),
                    kind: pse_relations::generated::enums::PublicationKind::Relations,
                    inputs: vec![],
                    members: vec![],
                },
                BTreeMap::from([(
                    name("authored", "bounded_values"),
                    base.join("published-values/").unwrap(),
                )]),
                vec![],
                &cancel,
            )
            .unwrap();
        let completed = prepared.execute(&cancel).await.unwrap();
        let version = completed.batches()[0]
            .column(0)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap()
            .value(0);
        drop(completed);
        let publication = pse_catalog::delta::publication::Publication::open(
            pse_catalog::delta::publication::PublicationRoot {
                location: control,
                version,
            },
            registry.clone(),
            &factory,
            &cancel,
        )
        .await
        .unwrap();
        let rows = publication
            .session()
            .capture_relation(&name("authored", "bounded_values"), &cancel)
            .await
            .unwrap();
        assert_eq!(rows.checked().batch().schema(), schema);
        assert!(
            local
                .evaluate(rows.checked().batch(), 16, &cancel)
                .unwrap()
                .is_valid()
        );
    }
}
