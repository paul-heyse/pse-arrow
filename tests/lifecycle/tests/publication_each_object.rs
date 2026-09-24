// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Interrupt actual Delta writes and require a coherent old or committed publication.
#![allow(
    clippy::unwrap_used,
    reason = "test fixture construction and exact independent value assertions"
)]

use datafusion::{arrow::array::Int64Array, common::ResolvedTableReference};
use pse_catalog::{
    artifact::{ArtifactPlan, PublicationTarget, RelationOutput},
    delta::publication::{Publication, PublicationRoot},
};
use pse_columnar::CancellationToken;
use pse_engine::{EngineError, session::EngineFactory};
use pse_ids::SemanticId;
use pse_relations::generated::{enums::PublicationKind, runtime::publications};
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
    ) -> Result<PublicationRoot, EngineError> {
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
        let control = self.base.join("control/").unwrap();
        let header = publications::Row {
            workspace_id: id(1),
            publication_id: id(attempt),
            parent_publication_id: parent.map(id),
            attempt_id: id(attempt + 100),
            kind: PublicationKind::Relations,
            inputs: vec![],
            members: vec![],
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
            .prepare_publication(
                PublicationTarget {
                    reference: ResolvedTableReference {
                        catalog: "artifact".into(),
                        schema: "runtime".into(),
                        table: "publications".into(),
                    },
                    location: control.clone(),
                },
                header,
                destinations,
                vec![],
                &cancel,
            )?
            .execute(&cancel)
            .await?;
        let version = result.batches()[0]
            .column(0)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap()
            .value(0);
        Ok(PublicationRoot {
            location: control,
            version,
        })
    }
    async fn latest(&self) -> Publication {
        let control = self.base.join("control/").unwrap();
        let table = deltalake::DeltaTableBuilder::from_url(control.clone())
            .unwrap()
            .with_storage_backend(self.store.clone(), control.clone())
            .load()
            .await
            .unwrap();
        let root = PublicationRoot {
            location: control,
            version: i64::try_from(table.version().unwrap()).unwrap(),
        };
        Publication::open(
            root,
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
    assert!(
        trace
            .iter()
            .any(|path| path.starts_with("control/_delta_log/"))
    );
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
        let latest = fixture.latest().await;
        // A native optional post-commit write may fail after the transaction settled.
        // In either outcome the visible control row selects one complete vector.
        if let Ok(root) = result {
            assert_eq!(latest.root(), &root);
            fixture.assert_values(&latest, 99).await;
        } else {
            assert_eq!(latest.root(), &old);
            fixture.assert_values(&latest, 1).await;
        }
        let original = Publication::open(
            old,
            fixture.registry.clone(),
            &fixture.factory,
            &CancellationToken::new(),
        )
        .await
        .unwrap();
        fixture.assert_values(&original, 1).await;
    }
}
