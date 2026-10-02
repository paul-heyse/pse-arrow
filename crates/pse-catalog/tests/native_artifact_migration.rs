// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![recursion_limit = "256"]
//! Real immutable-member migration journey, authored for Plan 25k execution.
#![allow(clippy::unwrap_used, reason = "independent storage journey assertions")]
use datafusion::{
    arrow::{
        array::{Int64Array, RecordBatch},
        datatypes::DataType,
    },
    common::{ResolvedTableReference, ScalarValue},
};
use pse_catalog::{
    artifact::{
        ArtifactPlan, RelationOutput,
        migration::{ArtifactMigration, MemberMigration},
    },
    delta::publication::{Publication, PublicationSelection},
};
use pse_columnar::CancellationToken;
use pse_engine::session::{EngineFactory, planner::UnifiedPlanner};
use pse_ids::SemanticId;
use pse_relations::generated::{
    enums::PublicationKind,
    runtime::{artifact_migration_lineage, publication_manifests},
};
use pse_schema::{
    NativeLiteral, Registry, RegistryBuilder,
    model::{
        Authority, FieldContract, MappingPolicy, MigrationNulls, MigrationSpec, MigrationStep,
        MigrationValuePair, MigrationValues, Namespace, RelationDecl, SnapshotClass,
    },
};
use std::{collections::BTreeMap, sync::Arc};
fn name(table: &str) -> ResolvedTableReference {
    ResolvedTableReference {
        catalog: "artifact".into(),
        schema: "authored".into(),
        table: table.into(),
    }
}
fn registry(version: u32) -> Arc<Registry> {
    let mut builder = RegistryBuilder::new();
    pse_schema::catalog::declare_publications(&mut builder);
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "migration_values",
            version,
            Authority::Authored,
            SnapshotClass::Model,
            "Independent storage migration fixture",
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::key("id", FieldContract::native(DataType::Int64), "Key"),
            FieldContract::payload("value", FieldContract::native(DataType::Int64), "Value"),
        ]),
    );
    Arc::new(builder.build().unwrap())
}
fn header(byte: u8, kind: PublicationKind) -> publication_manifests::Row {
    let id = |byte| SemanticId::from_bytes([byte; 16]);
    publication_manifests::Row {
        publication_id: id(byte).into(),
        workspace_id: id(1).into(),
        parent_publication_id: None,
        attempt_id: id(byte + 1).into(),
        kind,
        inputs: vec![],
        members: vec![],
        windows: vec![],
        exported_at: None,
        export_lease_id: None,
        export_expires_at: None,
        maintenance_epoch: None,
        store_fingerprint: None,
    }
}
fn record(
    registry: &Registry,
    validation: &pse_relations::validate::ValidationContext,
    completed: &pse_engine::session::CompletedComputation,
) -> publication_manifests::Row {
    publication_manifests::View::try_from_batch_with_registry(
        registry,
        &completed.batches()[0],
        validation,
    )
    .unwrap()
    .row(0)
    .unwrap()
}
fn migration(source: &Registry, target: &Registry, include_two: bool) -> MemberMigration {
    let source_field = pse_schema::arrow::relation_schema(
        source,
        source.relation("authored.migration_values").unwrap(),
    )
    .unwrap()
    .field(1)
    .clone();
    let target_field = pse_schema::arrow::relation_schema(
        target,
        target.relation("authored.migration_values").unwrap(),
    )
    .unwrap()
    .field(1)
    .clone();
    let row = |from, to| MigrationValuePair {
        source: vec![
            NativeLiteral::from_scalar(
                Arc::new(source_field.clone()),
                &ScalarValue::Int64(Some(from)),
            )
            .unwrap(),
        ],
        target: vec![
            NativeLiteral::from_scalar(
                Arc::new(target_field.clone()),
                &ScalarValue::Int64(Some(to)),
            )
            .unwrap(),
        ],
    };
    let mut rows = vec![row(1, 11)];
    if include_two {
        rows.push(row(2, 22));
    }
    MemberMigration {
        source: name("migration_values"),
        output: name("migration_values"),
        declaration: MigrationSpec {
            relation: "authored.migration_values",
            from_version: 1,
            to_version: 2,
            steps: vec![MigrationStep::RecodeDomain {
                name: "value",
                mapping: MigrationValues {
                    policy: MappingPolicy::Finite(rows),
                    nulls: MigrationNulls::Reject,
                },
            }],
            doc: "Explicit finite recoding",
        },
    }
}
async fn source(
    factory: &EngineFactory,
    registry: Arc<Registry>,
    root: &url::Url,
    cancel: &CancellationToken,
) -> publication_manifests::Row {
    let spec = registry.relation("authored.migration_values").unwrap();
    let batch = RecordBatch::try_new(
        Arc::new(pse_schema::arrow::relation_schema(&registry, spec).unwrap()),
        vec![
            Arc::new(Int64Array::from(vec![1, 2])),
            Arc::new(Int64Array::from(vec![1, 2])),
        ],
    )
    .unwrap();
    let session = factory
        .candidate(
            BTreeMap::from([(spec.key, batch)]),
            registry.clone(),
            cancel,
        )
        .unwrap();
    let source = session.table_reference(&spec.key).unwrap().resolve("", "");
    let artifact = ArtifactPlan::new(
        session.clone(),
        BTreeMap::from([(
            name("migration_values"),
            RelationOutput {
                relation_id: spec.id,
                plan: session.relation_plan(&source).unwrap().plan().clone(),
            },
        )]),
        cancel,
    )
    .unwrap();
    let (candidate, _) = artifact
        .prepare_publication(
            header(2, PublicationKind::Relations),
            BTreeMap::from([(
                name("migration_values"),
                root.join("source/values/").unwrap(),
            )]),
            vec![],
            cancel,
        )
        .unwrap();
    record(
        &registry,
        &artifact.session().validation_context().unwrap(),
        &candidate.execute(cancel).await.unwrap(),
    )
}
#[tokio::test]
async fn explicit_migration_checks_transient_output_publishes_lineage_and_preserves_source_on_failure()
 {
    let cancel = CancellationToken::new();
    let root = url::Url::parse("memory://migration-journey/").unwrap();
    let fixture = pse_testkit::NativeFixture::new((256 << 20).try_into().unwrap()).unwrap();
    fixture
        .resources
        .runtime
        .register_object_store(&root, Arc::new(object_store::memory::InMemory::new()));
    let factory = fixture
        .into_factory()
        .with_query_planner(Arc::new(UnifiedPlanner::new(
            pse_catalog::assembly::planners(),
        )));
    let old = registry(1);
    let current = registry(2);
    let original = source(&factory, old.clone(), &root, &cancel).await;
    let source_member = original.members[0].clone();
    let source_version = source_member.delta_version;
    let opened = Publication::open_recorded(
        PublicationSelection {
            record: original.clone(),
            scope: None,
            owner: None,
        },
        current.clone(),
        &factory,
        &cancel,
    )
    .await
    .unwrap();
    let failed = ArtifactMigration::new(
        &opened,
        vec![migration(&old, &current, false)],
        vec![],
        &cancel,
    )
    .unwrap();
    assert!(
        failed
            .prepare_read(&name("migration_values"), &cancel)
            .await
            .is_err()
    );
    let failed_lineage = failed.lineage()[0].transformation_digest;
    let checked = ArtifactMigration::new(
        &opened,
        vec![migration(&old, &current, true)],
        vec![],
        &cancel,
    )
    .unwrap();
    assert_ne!(failed_lineage, checked.lineage()[0].transformation_digest);
    assert_eq!(checked.lineage()[0].source_member, original.members[0]);
    let completed = checked
        .prepare_read(&name("migration_values"), &cancel)
        .await
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    let values = completed
        .batches()
        .iter()
        .flat_map(|batch| {
            batch
                .batch()
                .column(1)
                .as_any()
                .downcast_ref::<Int64Array>()
                .unwrap()
                .iter()
        })
        .collect::<Vec<_>>();
    assert_eq!(values, vec![Some(11), Some(22)]);
    let (artifact, retained) = checked.into_publication();
    let operation_id = SemanticId::from_bytes([9; 16]);
    let artifact = artifact.with_operation(operation_id).unwrap();
    let destinations: BTreeMap<_, _> = artifact
        .outputs()
        .keys()
        .map(|name| {
            (
                name.clone(),
                root.join(&format!("target/{}/{}/", name.schema, name.table))
                    .unwrap(),
            )
        })
        .collect();
    let mut target_header = header(4, PublicationKind::Migration);
    target_header.inputs = original.members.clone();
    let (candidate, ticket) = artifact
        .prepare_publication(
            target_header.clone(),
            destinations.clone(),
            retained,
            &cancel,
        )
        .unwrap();
    assert_eq!(ticket.candidate().members.len(), 2);
    let migrated = record(
        &current,
        &artifact.session().validation_context().unwrap(),
        &candidate.execute(&cancel).await.unwrap(),
    );
    let mut changed = migration(&old, &current, true);
    if let MigrationStep::RecodeDomain {
        mapping:
            MigrationValues {
                policy: MappingPolicy::Finite(rows),
                ..
            },
        ..
    } = &mut changed.declaration.steps[0]
    {
        let field = Arc::new(
            pse_schema::arrow::relation_schema(
                &current,
                current.relation("authored.migration_values").unwrap(),
            )
            .unwrap()
            .field(1)
            .clone(),
        );
        rows[0].target[0] =
            NativeLiteral::from_scalar(field, &ScalarValue::Int64(Some(33))).unwrap();
    }
    let (changed, retained) = ArtifactMigration::new(&opened, vec![changed], vec![], &cancel)
        .unwrap()
        .into_publication();
    let (changed_candidate, _) = changed
        .with_operation(operation_id)
        .unwrap()
        .prepare_publication(target_header, destinations, retained, &cancel)
        .unwrap();
    assert!(
        changed_candidate
            .execute(&cancel)
            .await
            .unwrap_err()
            .to_string()
            .contains("attempt identity was reused")
    );
    let target = Publication::open(
        PublicationSelection {
            record: migrated.clone(),
            scope: None,
            owner: None,
        },
        current,
        &factory,
        &cancel,
    )
    .await
    .unwrap();
    let mut stream = target
        .relation_stream(
            &ResolvedTableReference {
                catalog: "artifact".into(),
                schema: "runtime".into(),
                table: artifact_migration_lineage::NAME.into(),
            },
            &cancel,
        )
        .await
        .unwrap();
    let batch = stream.next_batch(&cancel).await.unwrap().unwrap();
    let view = artifact_migration_lineage::View::try_from_batch_with_registry(
        target.session().registry(),
        batch.batch(),
        &target.session().validation_context().unwrap(),
    )
    .unwrap();
    assert_eq!(
        view.row(0).unwrap().source_publication_id,
        original.publication_id
    );
    assert_eq!(view.row(0).unwrap().source_member, source_member);
    assert!(stream.next_batch(&cancel).await.unwrap().is_none());
    drop(stream);
    let original_again = Publication::open(
        PublicationSelection {
            record: original.clone(),
            scope: None,
            owner: None,
        },
        old,
        &factory,
        &cancel,
    )
    .await
    .unwrap();
    assert_eq!(original_again.record(), &original);
    assert_eq!(
        original_again
            .member(&name("migration_values"))
            .unwrap()
            .delta_version,
        source_version
    );
    let mut stream = original_again
        .relation_stream(&name("migration_values"), &cancel)
        .await
        .unwrap();
    let batch = stream.next_batch(&cancel).await.unwrap().unwrap();
    assert_eq!(
        batch
            .batch()
            .column(1)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap()
            .iter()
            .collect::<Vec<_>>(),
        vec![Some(1), Some(2)]
    );
    assert!(stream.next_batch(&cancel).await.unwrap().is_none());
}
