// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Interrupt every actual Delta write of a publication candidate: the candidate either
//! returns a complete admitted record whose members open, or returns nothing, and an
//! earlier publication is unaffected. (The catalog commit alone makes a record visible.)
#![allow(
    clippy::unwrap_used,
    reason = "test fixture construction and exact independent value assertions"
)]

use datafusion::{arrow::array::Int64Array, common::ResolvedTableReference};
use pse_catalog::{
    artifact::{ArtifactPlan, RelationOutput},
    delta::publication::{Publication, PublicationSelection},
};
use pse_columnar::CancellationToken;
use pse_engine::{EngineError, session::EngineFactory};
use pse_ids::SemanticId;
use pse_relations::generated::{enums::PublicationKind, runtime::publication_manifests};
use pse_schema::{
    Registry, RegistryBuilder,
    model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass},
};
use pse_testkit::fault_store::{Fault, FaultPlan, FaultStore};
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};
fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}
fn name(table: &str) -> ResolvedTableReference {
    ResolvedTableReference {
        catalog: "artifact".into(),
        schema: "authored".into(),
        table: table.into(),
    }
}
struct Fixture {
    registry: Arc<Registry>,
    factory: EngineFactory,
    store: Arc<FaultStore>,
    base: url::Url,
}
impl Fixture {
    fn new() -> Self {
        let mut builder = RegistryBuilder::new();
        pse_schema::catalog::declare_publications(&mut builder);
        for table in ["first", "second", "third"] {
            builder.declare_relation(
                RelationDecl::new(
                    Namespace::Authored,
                    table,
                    1,
                    Authority::Authored,
                    SnapshotClass::Model,
                    "Delta publication fault fixture",
                )
                .pk(&["id"])
                .delta_properties([("delta.checkpointInterval".into(), "1".into())])
                .columns(vec![
                    FieldContract::key(
                        "id",
                        FieldContract::native(arrow::datatypes::DataType::Int64),
                        "Key",
                    ),
                    FieldContract::payload(
                        "value",
                        FieldContract::native(arrow::datatypes::DataType::Int64),
                        "Value",
                    ),
                ]),
            );
        }
        let registry = Arc::new(builder.build().unwrap());
        let fixture =
            pse_testkit::NativeFixture::new(NonZeroUsize::new(64 << 20).unwrap()).unwrap();
        let runtime = fixture.resources.runtime.clone();
        let mut cache_policy = pse_catalog::cache_service::DeltaCacheBudget::for_memory(64 << 20);
        cache_policy.crc_replay_max_commits = 16;
        cache_policy.checksum_interval = 1;
        let caches =
            pse_catalog::cache_service::DeltaCacheService::new(cache_policy, &runtime.memory_pool)
                .unwrap();
        let base = url::Url::parse("memory://lifecycle/").unwrap();
        let store = FaultStore::new(Arc::new(object_store::memory::InMemory::new()));
        runtime.register_object_store(&base, store.clone());
        let factory = fixture
            .into_factory()
            .with_cache_service(caches.native().clone())
            .with_extension(caches)
            .with_query_planner(Arc::new(pse_engine::session::planner::UnifiedPlanner::new(
                pse_catalog::assembly::planners(),
            )));
        Self {
            registry,
            factory,
            store,
            base,
        }
    }
    async fn publish(
        &self,
        attempt: u8,
        parent: Option<u8>,
        value: i64,
    ) -> Result<publication_manifests::Row, EngineError> {
        let cancel = CancellationToken::new();
        let mut batches = BTreeMap::new();
        for table in ["first", "second", "third"] {
            let spec = self
                .registry
                .relation(&format!("authored.{table}"))
                .unwrap();
            batches.insert(
                spec.key,
                pse_relations::testing::batch_from_literals(
                    &self.registry,
                    spec,
                    &[vec![
                        serde_json::json!(["i64", 1]),
                        serde_json::json!(["i64", value]),
                    ]],
                )
                .unwrap(),
            );
        }
        let session = self
            .factory
            .candidate(batches, self.registry.clone(), &cancel)?;
        let outputs = session
            .input_keys()
            .map(|key| {
                let reference = session.table_reference(&key).unwrap();
                let source = session
                    .relation_plan(&ResolvedTableReference {
                        catalog: reference.catalog().unwrap().into(),
                        schema: reference.schema().unwrap().into(),
                        table: reference.table().into(),
                    })
                    .unwrap();
                (
                    name(key.name),
                    RelationOutput {
                        relation_id: source.relation_id(),
                        plan: source.plan().clone(),
                    },
                )
            })
            .collect();
        let plan = ArtifactPlan::new(session, outputs, &cancel)?;
        let header = publication_manifests::Row {
            publication_id: id(attempt),
            workspace_id: id(1).into(),
            parent_publication_id: parent.map(|parent| id(parent).into()),
            attempt_id: id(attempt + 100).into(),
            kind: PublicationKind::Relations,
            inputs: vec![],
            members: vec![],
            windows: vec![],
            exported_at: None,
            export_lease_id: None,
            export_expires_at: None,
            maintenance_epoch: None,
            store_fingerprint: None,
        };
        let destinations = plan
            .outputs()
            .keys()
            .map(|reference| {
                (
                    reference.clone(),
                    self.base
                        .join(&format!("members/{attempt}/{}/", reference.table))
                        .unwrap(),
                )
            })
            .collect();
        let result = plan
            .prepare_publication(header, destinations, vec![], &cancel)
            .map(|(command, _ticket)| command)?
            .execute(&cancel)
            .await?;
        let [batch] = result.batches() else {
            panic!("a candidate returns one record")
        };
        Ok(
            publication_manifests::View::try_from_batch_with_registry(&self.registry, batch)
                .unwrap()
                .row(0)
                .unwrap(),
        )
    }
    async fn open(&self, record: &publication_manifests::Row) -> Publication {
        Publication::open(
            PublicationSelection {
                record: record.clone(),
                scope: None,
                owner: None,
            },
            self.registry.clone(),
            &self.factory,
            &CancellationToken::new(),
        )
        .await
        .unwrap()
    }
    async fn assert_values(&self, publication: &Publication, expected: i64) {
        assert_eq!(publication.record().members.len(), 3);
        for table in ["first", "second", "third"] {
            let value = publication
                .session()
                .capture_relation(&name(table), &CancellationToken::new())
                .await
                .unwrap();
            assert_eq!(value.checked().batch().num_rows(), 1);
            let value = value
                .checked()
                .batch()
                .column(1)
                .as_any()
                .downcast_ref::<Int64Array>()
                .unwrap();
            assert_eq!(value.value(0), expected);
        }
    }
}
#[tokio::test]
async fn every_actual_delta_write_boundary_preserves_a_complete_publication() {
    let complete = Fixture::new();
    complete.publish(2, None, 1).await.unwrap();
    complete.store.clear_trace();
    complete.publish(3, Some(2), 99).await.unwrap();
    let trace = complete.store.put_trace();
    assert!(trace.iter().any(|path| path.starts_with("members/3/")));
    // A candidate writes members only: visibility is the catalog's commit.
    assert!(trace.iter().all(|path| path.starts_with("members/3/")), "{trace:?}");
    for (index, path) in trace.iter().enumerate() {
        let fixture = Fixture::new();
        let old = fixture.publish(2, None, 1).await.unwrap();
        fixture.store.arm(FaultPlan {
            operation: "put",
            prefix: String::new(),
            call: index + 1,
            fault: Fault::FailBefore,
        });
        let result = fixture.publish(3, Some(2), 99).await;
        assert_eq!(fixture.store.fired(), 1, "actual write {index}: {path}");
        // A native optional post-commit write may fail after every member committed; an
        // admitted record is then complete. Otherwise nothing is admitted.
        if let Ok(record) = result {
            let admitted = fixture.open(&record).await;
            fixture.assert_values(&admitted, 99).await;
        }
        let original = fixture.open(&old).await;
        fixture.assert_values(&original, 1).await;
    }
}
