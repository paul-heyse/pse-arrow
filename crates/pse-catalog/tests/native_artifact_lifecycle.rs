// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Delta integration oracles. Author/compile now; execute only after Plan 13 W18.
#![allow(clippy::unwrap_used, reason = "integration fixture assertions")]
use datafusion::{
    arrow::{
        array::{Int64Array, RecordBatch},
        datatypes::DataType,
    },
    common::ResolvedTableReference,
};
use pse_catalog::{
    artifact::{ArtifactPlan, PublicationTarget, RelationOutput},
    delta::{
        maintenance::{MaintenanceAction, MaintenanceTarget},
        publication::{Publication, PublicationRoot},
    },
};
use pse_columnar::CancellationToken;
use pse_engine::session::EngineFactory;
use pse_ids::SemanticId;
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
    builder.declare_artifact_profile(
        "inspection",
        ["runtime.artifact_descriptors".to_owned()].into(),
    );
    Arc::new(builder.build().unwrap())
}

fn descriptor(registry: &Registry) -> pse_model::artifact::ArtifactDescriptor {
    use pse_ids::ContentHash;
    use pse_model::{
        artifact::ArtifactDescriptor,
        generated::{enums::ArtifactReconstruction, runtime::artifact_descriptors as wire},
    };
    let hash = ContentHash::from_bytes([1; 32]);
    ArtifactDescriptor::create(wire::Row {
        artifact_id: hash,
        descriptor_version: 2,
        profile: PublicationKind::Inspection,
        profile_contract: pse_schema::fingerprint::semantic_profile(
            registry,
            "inspection",
            &[registry.relation("authored.values").unwrap().id].into(),
        )
        .unwrap(),
        requested_relations: vec![registry.relation("authored.values").unwrap().id],
        release_id: hash,
        release_members: vec![],
        semantic_identity: hash,
        implementation: wire::RuntimeArtifactDescriptorsFieldImplementation {
            source: hash,
            build: hash,
            registry: registry.fingerprint(),
            algorithms: hash,
        },
        target_contract: hash,
        value_assumptions: vec![],
        reconstruction: ArtifactReconstruction::None,
    })
    .unwrap()
}

#[tokio::test]
async fn explicit_product_reopens_after_eviction_and_refuses_another_descriptor() {
    use pse_ids::ContentHash;
    use pse_model::artifact::ArtifactDescriptor;
    let cancel = CancellationToken::new();
    let registry = registry();
    let factory = factory();
    let descriptor = descriptor(&registry);
    let artifact = artifact(&factory, registry.clone(), &cancel)
        .inspection()
        .with_product(descriptor.clone(), &cancel)
        .unwrap();
    assert_eq!(artifact.outputs().len(), 2);
    let directory = tempfile::tempdir().unwrap();
    let base = url::Url::from_directory_path(directory.path()).unwrap();
    let control = base.join("control/").unwrap();
    let destinations = artifact
        .outputs()
        .keys()
        .enumerate()
        .map(|(index, name)| {
            (
                name.clone(),
                base.join(&format!("members/{index}/")).unwrap(),
            )
        })
        .collect();
    let header = publications::Row {
        workspace_id: id(10),
        publication_id: id(11),
        attempt_id: id(12),
        parent_publication_id: None,
        kind: PublicationKind::Inspection,
        inputs: vec![],
        members: vec![],
    };
    let command = artifact
        .prepare_publication(
            PublicationTarget {
                reference: name("runtime", "publications"),
                location: control.clone(),
            },
            header,
            destinations,
            vec![],
            &cancel,
        )
        .map(|(command, _ticket)| command)
        .unwrap();
    let result = command.execute(&cancel).await.unwrap();
    let version = result.batches()[0]
        .column(0)
        .as_any()
        .downcast_ref::<Int64Array>()
        .unwrap()
        .value(0);
    drop(result);
    drop(artifact);
    // A fresh native factory cannot inherit producer cells or mutable old providers.
    let cold = self::factory();
    let publication = Publication::open(
        PublicationRoot {
            location: control,
            version,
        },
        registry,
        &cold,
        &cancel,
    )
    .await
    .unwrap();
    assert_eq!(
        **publication
            .require_artifact(&descriptor, &cancel)
            .await
            .unwrap(),
        descriptor
    );
    let mut changed = descriptor.row().clone();
    changed.target_contract = ContentHash::from_bytes([2; 32]);
    let changed = ArtifactDescriptor::create(changed).unwrap();
    assert!(
        publication
            .require_artifact(&changed, &cancel)
            .await
            .is_err()
    );
}
fn factory() -> EngineFactory {
    pse_testkit::NativeFixture::new((128 << 20).try_into().unwrap())
        .unwrap()
        .into_factory()
        .with_query_planner(Arc::new(pse_engine::session::planner::UnifiedPlanner::new(
            pse_catalog::assembly::planners(),
        )))
}
fn artifact(
    factory: &EngineFactory,
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
    let runtime = factory.native_state().runtime_env();
    let faults = pse_testkit::fault_store::FaultStore::new(
        runtime
            .object_store(datafusion::execution::object_store::ObjectStoreUrl::local_filesystem())
            .unwrap(),
    );
    runtime.register_object_store(&base, faults.clone());
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
            .map(|(command, _ticket)| command)
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
    let changes = pse_catalog::delta::changes::prepare_changes(
        publication.session(),
        &name("authored", "values"),
        0,
        &cancel,
    )
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
    let retained_publications = pse_catalog::delta::retention::publication_retention(
        &administrative,
        &selected_controls,
        &cancel,
    )
    .unwrap();
    let retention = pse_catalog::delta::retention::combine_retention(
        &administrative,
        &[retention, retained_publications],
        &cancel,
    )
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
        pse_catalog::delta::maintenance::prepare_maintenance(
            &administrative,
            maintenance.clone(),
            &retention,
            &cancel
        )
        .unwrap()
        .execute(&cancel)
        .await
        .is_err()
    );
    drop(publication);
    // The fence commits before checkpointing. A later IO error must preserve that
    // durable boundary and the original cause; a generic pre-commit error is false.
    faults.arm(pse_testkit::fault_store::FaultPlan {
        operation: "put",
        prefix: base
            .join("values/_delta_log/_last_checkpoint")
            .unwrap()
            .path()
            .trim_start_matches('/')
            .into(),
        call: 1,
        fault: pse_testkit::fault_store::Fault::FailBefore,
    });
    let interrupted = pse_catalog::delta::maintenance::prepare_maintenance(
        &administrative,
        maintenance.clone(),
        &retention,
        &cancel,
    )
    .unwrap()
    .execute(&cancel)
    .await
    .unwrap_err();
    assert_eq!(faults.fired(), 1);
    let mut cause: &(dyn std::error::Error + 'static) = &interrupted;
    let fence_version = loop {
        if let Some(pse_catalog::delta::settlement::SettlementError::MaintenanceInterrupted {
            fence_version,
            source,
        }) = cause.downcast_ref()
        {
            assert!(source.source().is_some());
            break *fence_version;
        }
        cause = cause.source().unwrap();
    };
    assert!(fence_version > 0);
    let protected = Publication::open(root.clone(), registry.clone(), &factory, &cancel)
        .await
        .unwrap();
    let protected_rows = protected
        .session()
        .capture_relation(&name("authored", "values"), &cancel)
        .await
        .unwrap();
    assert_eq!(
        protected_rows
            .checked()
            .batch()
            .column(1)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap()
            .value(0),
        41
    );
    drop(protected_rows);
    drop(protected);
    // A fresh prepared command reconciles current native fences; it does not replay
    // a stale pre-maintenance snapshot or discard the selected historical member.
    pse_catalog::delta::maintenance::prepare_maintenance(
        &administrative,
        maintenance,
        &retention,
        &cancel,
    )
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
            .map(|(command, _ticket)| command)
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
    pse_catalog::delta::maintenance::prepare_maintenance(
        &administrative,
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
