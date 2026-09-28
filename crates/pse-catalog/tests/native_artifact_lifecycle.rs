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
    artifact::{ArtifactPlan, RelationOutput},
    delta::{
        collect::{CollectAction, CollectTarget},
        publication::{Publication, PublicationSelection},
    },
};
use pse_columnar::CancellationToken;
use pse_engine::session::EngineFactory;
use pse_ids::SemanticId;
use pse_relations::generated::{
    enums::{PublicationKind, RetentionReason},
    runtime::{publication_manifests, retained_versions},
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
fn header(publication: u8, kind: PublicationKind) -> publication_manifests::Row {
    publication_manifests::Row {
        publication_id: id(publication),
        workspace_id: id(1),
        parent_publication_id: None,
        attempt_id: id(publication + 1),
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
/// The admitted record an executed candidate returns.
fn admitted(
    registry: &Registry,
    completed: &pse_engine::session::CompletedComputation,
) -> publication_manifests::Row {
    publication_manifests::View::try_from_batch_with_registry(registry, &completed.batches()[0])
        .unwrap()
        .row(0)
        .unwrap()
}
/// Open exactly the members a (catalog-granted) record selects.
async fn open(
    record: &publication_manifests::Row,
    registry: Arc<Registry>,
    factory: &EngineFactory,
    cancel: &CancellationToken,
) -> Publication {
    Publication::open(
        PublicationSelection {
            record: record.clone(),
            scope: None,
            owner: None,
        },
        registry,
        factory,
        cancel,
    )
    .await
    .unwrap()
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
    let command = artifact
        .prepare_publication(
            header(11, PublicationKind::Inspection),
            destinations,
            vec![],
            &cancel,
        )
        .map(|(command, _ticket)| command)
        .unwrap();
    let result = command.execute(&cancel).await.unwrap();
    let record = admitted(&registry, &result);
    drop(result);
    drop(artifact);
    // A fresh native factory cannot inherit producer cells or mutable old providers.
    let cold = self::factory();
    let publication = open(&record, registry, &cold, &cancel).await;
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
    let destinations =
        BTreeMap::from([(name("authored", "values"), base.join("values/").unwrap())]);
    let publish = || {
        artifact
            .prepare_publication(
                header(2, PublicationKind::Relations),
                destinations.clone(),
                vec![],
                &cancel,
            )
            .map(|(command, _ticket)| command)
            .unwrap()
    };
    let first = publish().execute(&cancel).await.unwrap();
    let record = admitted(&registry, &first);
    drop(first);
    // Re-preparing the same composition recovers its written members.
    let retry = publish().execute(&cancel).await.unwrap();
    assert_eq!(admitted(&registry, &retry), record);
    drop(retry);
    let publication = open(&record, registry.clone(), &factory, &cancel).await;
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
    // Collection keeps the version the publication selects (the catalog's range).
    let member = publication.member(&name("authored", "values")).unwrap();
    let retained = vec![retained_versions::Row {
        table_uri: member.table_uri.clone(),
        from_version: member.delta_version,
        through_version: member.delta_version,
        reason: RetentionReason::Publication,
    }];
    let administrative = factory
        .candidate(BTreeMap::new(), registry.clone(), &cancel)
        .unwrap();
    let collect = || {
        pse_catalog::delta::collect::prepare_collect(
            &administrative,
            CollectTarget {
                reference: name("authored", "values"),
                location: base.join("values/").unwrap(),
                action: CollectAction::Optimize,
                log_cutoff_ms: 0,
            },
            retained.clone(),
            &cancel,
        )
        .unwrap()
    };
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
    let interrupted = collect().execute(&cancel).await.unwrap_err();
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
    let protected = open(&record, registry.clone(), &factory, &cancel).await;
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
    collect().execute(&cancel).await.unwrap();
    let reopened = open(&record, registry.clone(), &factory, &cancel).await;
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
}
