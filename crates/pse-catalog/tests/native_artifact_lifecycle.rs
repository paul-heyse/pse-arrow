// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Final-cut Delta integration oracles. Do not run before Plan 08 implementation closure.
#![allow(clippy::unwrap_used, reason = "integration fixture assertions")]
use datafusion::{
    arrow::{
        array::{Int64Array, RecordBatch},
        datatypes::DataType,
    },
    common::ResolvedTableReference,
    execution::runtime_env::RuntimeEnv,
};
use pse_catalog::{
    artifact::{ArtifactPlan, PublicationTarget, RelationOutput},
    delta::{
        maintenance::{MaintenanceAction, MaintenanceTarget},
        publication::{Publication, PublicationRoot},
    },
    session::{ExecutionSettings, SessionFactory, ThreadBudget, native_engine_profile},
};
use pse_ids::{CancellationToken, FixedBudget, SemanticId};
use pse_relations::generated::{
    enums::PublicationKind,
    runtime::{publications, retained_versions},
};
use pse_schema::{
    Registry, RegistryBuilder,
    model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass},
};
use std::{collections::BTreeMap, sync::Arc};

fn name(schema: &str, table: &str) -> ResolvedTableReference {
    ResolvedTableReference {
        catalog: "artifact".into(),
        schema: schema.to_owned().into(),
        table: table.to_owned().into(),
    }
}
fn id(byte: u8) -> SemanticId {
    SemanticId::from_bytes([byte; 16])
}
fn registry() -> Arc<Registry> {
    let mut builder = RegistryBuilder::new();
    pse_schema::catalog::declare_publications(&mut builder);
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "values",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "Current native lifecycle fixture",
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::key("id", FieldContract::native(DataType::Int64), "Key"),
            FieldContract::payload("value", FieldContract::native(DataType::Int64), "Value"),
        ]),
    );
    Arc::new(builder.build().unwrap())
}
fn factory() -> SessionFactory {
    SessionFactory::new(
        Arc::new(RuntimeEnv::default()),
        FixedBudget::new(128 << 20),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: 1.try_into().unwrap(),
            target_partitions: 1.try_into().unwrap(),
        },
        native_engine_profile(),
    )
    .unwrap()
}
fn artifact(
    factory: &SessionFactory,
    registry: Arc<Registry>,
    cancel: &CancellationToken,
) -> ArtifactPlan {
    let spec = registry.relation("authored.values").unwrap();
    let batch = RecordBatch::try_new(
        Arc::new(pse_schema::arrow::relation_schema(&registry, spec).unwrap()),
        vec![
            Arc::new(Int64Array::from(vec![1])),
            Arc::new(Int64Array::from(vec![41])),
        ],
    )
    .unwrap();
    let key = spec.key;
    let relation_id = spec.id;
    let session = factory
        .candidate(BTreeMap::from([(key, batch)]), registry, cancel)
        .unwrap();
    let reference = session.table_reference(&key).unwrap();
    let input = session
        .relation_plan(&ResolvedTableReference {
            catalog: reference.catalog().unwrap().into(),
            schema: reference.schema().unwrap().into(),
            table: reference.table().into(),
        })
        .unwrap();
    ArtifactPlan::new(
        session,
        [(
            name("authored", "values"),
            RelationOutput {
                relation_id,
                plan: input.plan().clone(),
            },
        )]
        .into_iter()
        .collect(),
        cancel,
    )
    .unwrap()
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "ordered native lifecycle qualification preserves the independent assertions beside each phase"
)]
async fn exact_reuse_cdf_and_maintenance_share_native_ownership() {
    let cancel = CancellationToken::new();
    let registry = registry();
    let factory = factory();
    let artifact = artifact(&factory, registry.clone(), &cancel);
    let directory = tempfile::tempdir().unwrap();
    let base = url::Url::from_directory_path(directory.path()).unwrap();
    let target = PublicationTarget {
        reference: name("runtime", "publications"),
        location: base.join("control/").unwrap(),
    };
    let header = publications::Row {
        workspace_id: id(1),
        publication_id: id(2),
        parent_publication_id: None,
        attempt_id: id(3),
        kind: PublicationKind::Relations,
        inputs: vec![],
        members: vec![],
    };
    let destinations =
        BTreeMap::from([(name("authored", "values"), base.join("values/").unwrap())]);
    let publish = || {
        artifact
            .prepare_publication(
                target.clone(),
                header.clone(),
                destinations.clone(),
                vec![],
                &cancel,
            )
            .unwrap()
    };
    let first = publish().execute(&cancel).await.unwrap();
    let version = first.batches()[0]
        .column(0)
        .as_any()
        .downcast_ref::<Int64Array>()
        .unwrap()
        .value(0);
    drop(first);
    let retry = publish().execute(&cancel).await.unwrap();
    assert_eq!(
        retry.batches()[0]
            .column(0)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap()
            .value(0),
        version
    );
    drop(retry);
    let root = PublicationRoot {
        location: target.location,
        version,
    };
    let publication = Publication::open(root.clone(), registry.clone(), &factory, &cancel)
        .await
        .unwrap();
    let reused = artifact
        .prepare_reuse(&publication, &name("authored", "values"), &cancel)
        .await
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    assert_eq!(
        reused.batches()[0]
            .column(1)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap()
            .value(0),
        41
    );
    drop(reused);
    let cold = ArtifactPlan::new(
        artifact.session().clone(),
        artifact.outputs().clone(),
        &cancel,
    )
    .unwrap();
    assert!(
        cold.prepare_reuse(&publication, &name("authored", "values"), &cancel)
            .await
            .unwrap()
            .execute(&cancel)
            .await
            .is_err()
    );
    let changes = publication
        .session()
        .prepare_changes(&name("authored", "values"), 0, &cancel)
        .await
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    assert_eq!(
        changes
            .batches()
            .iter()
            .map(|batch| batch.num_rows())
            .sum::<usize>(),
        1
    );
    assert_eq!(
        changes.batches()[0]
            .schema()
            .fields()
            .iter()
            .map(|field| field.name().as_str())
            .collect::<Vec<_>>(),
        vec!["change", "value"]
    );
    drop(changes);
    let member = publication.member(&name("authored", "values")).unwrap();
    let relation_id = member.relation_id;
    let mut controls = publications::Builder::with_registry(&registry, 1).unwrap();
    controls.push(publication.record().clone()).unwrap();
    let empty = retained_versions::Builder::with_registry(&registry, 0)
        .unwrap()
        .finish()
        .unwrap()
        .into_batch();
    let administrative = factory
        .candidate(
            [
                (retained_versions::spec(&registry).unwrap().key, empty),
                (
                    publications::spec(&registry).unwrap().key,
                    controls.finish().unwrap().into_batch(),
                ),
            ]
            .into_iter()
            .collect(),
            registry.clone(),
            &cancel,
        )
        .unwrap();
    let source = administrative
        .table_reference(&retained_versions::spec(&registry).unwrap().key)
        .unwrap();
    let retention = administrative
        .relation_plan(&ResolvedTableReference {
            catalog: source.catalog().unwrap().into(),
            schema: source.schema().unwrap().into(),
            table: source.table().into(),
        })
        .unwrap();
    let selected_controls = administrative
        .table_reference(&publications::spec(&registry).unwrap().key)
        .unwrap();
    let selected_controls = administrative
        .relation_plan(&ResolvedTableReference {
            catalog: selected_controls.catalog().unwrap().into(),
            schema: selected_controls.schema().unwrap().into(),
            table: selected_controls.table().into(),
        })
        .unwrap();
    let retained_publications = administrative
        .publication_retention(&selected_controls, &cancel)
        .unwrap();
    let retention = administrative
        .combine_retention(&[retention, retained_publications], &cancel)
        .unwrap();
    let maintenance = MaintenanceTarget {
        head: root.clone(),
        reference: name("authored", "values"),
        location: base.join("values/").unwrap(),
        relation_id,
        action: MaintenanceAction::Optimize,
        log_cutoff_ms: 0,
    };
    assert!(
        administrative
            .prepare_maintenance(maintenance.clone(), &retention, &cancel)
            .unwrap()
            .execute(&cancel)
            .await
            .is_err()
    );
    drop(publication);
    administrative
        .prepare_maintenance(maintenance, &retention, &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    let reopened = Publication::open(root.clone(), registry.clone(), &factory, &cancel)
        .await
        .unwrap();
    let rows = reopened
        .session()
        .capture_relation(&name("authored", "values"), &cancel)
        .await
        .unwrap();
    assert_eq!(
        rows.checked()
            .batch()
            .column(1)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap()
            .value(0),
        41
    );
    drop(rows);
    drop(reopened);
    let mut rejected = header;
    rejected.publication_id = id(5);
    rejected.attempt_id = id(6);
    rejected.parent_publication_id = Some(id(99));
    let orphan = base.join("unpublished/").unwrap();
    let failed_publication = || {
        artifact
            .prepare_publication(
                PublicationTarget {
                    reference: name("runtime", "publications"),
                    location: root.location.clone(),
                },
                rejected.clone(),
                [(name("authored", "values"), orphan.clone())]
                    .into_iter()
                    .collect(),
                vec![],
                &cancel,
            )
            .unwrap()
    };
    assert!(failed_publication().execute(&cancel).await.is_err());
    // The valid member committed before the stale-parent control rejection.
    let before = deltalake::DeltaTableBuilder::from_url(orphan.clone())
        .unwrap()
        .load()
        .await
        .unwrap();
    let version = before.version().unwrap();
    drop(before);
    administrative
        .prepare_maintenance(
            MaintenanceTarget {
                head: root.clone(),
                reference: name("authored", "values"),
                location: orphan.clone(),
                relation_id,
                action: MaintenanceAction::ReclaimUnpublished,
                log_cutoff_ms: 0,
            },
            &retention,
            &cancel,
        )
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    let retired = deltalake::DeltaTableBuilder::from_url(orphan.clone())
        .unwrap()
        .load()
        .await
        .unwrap();
    assert!(retired.version().unwrap() > version);
    assert!(
        failed_publication().execute(&cancel).await.is_err(),
        "retirement cannot resurrect the original writer"
    );
    let actual = Publication::open(root, registry, &factory, &cancel)
        .await
        .unwrap();
    assert_eq!(actual.record().publication_id, id(2));
}
