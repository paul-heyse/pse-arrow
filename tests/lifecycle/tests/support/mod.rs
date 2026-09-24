// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual Arrow inputs and shared native execution resources.
#![allow(dead_code, clippy::expect_used, reason = "shared integration fixtures")]
use pse_columnar::CancellationToken;
use pse_engine::{ExecutionSettings, ThreadBudget};
use pse_schema::{
    Registry, RegistryBuilder,
    model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass},
};
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};
pub(crate) fn registry() -> Arc<Registry> {
    let mut builder = RegistryBuilder::new();
    pse_schema::catalog::declare_diagnostics(&mut builder);
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "samples",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "Lifecycle rows",
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::key(
                "id",
                FieldContract::native(arrow::datatypes::DataType::UInt64),
                "Key",
            ),
            FieldContract::payload(
                "value",
                FieldContract::native(arrow::datatypes::DataType::UInt64),
                "Value",
            ),
        ]),
    );
    Arc::new(builder.build().expect("registry"))
}
pub(crate) fn batch(registry: &Registry, count: u64, offset: u64) -> pse_relations::RecordBatch {
    let spec = registry.relation("authored.samples").expect("relation");
    let rows = (0..count)
        .map(|id| {
            vec![
                serde_json::json!(["u64", id]),
                serde_json::json!(["u64", count - id + offset]),
            ]
        })
        .collect::<Vec<_>>();
    pse_relations::testing::batch_from_literals(registry, spec, &rows).expect("typed rows")
}
pub(crate) fn session(
    runtime: &pse_runtime::SharedRuntime,
    registry: Arc<Registry>,
    count: u64,
) -> pse_engine::session::EngineSession {
    let values = batch(&registry, count, 0);
    let key = registry.relation("authored.samples").expect("relation").key;
    runtime
        .session_factory(pse_engine::session::native_engine_profile())
        .expect("factory")
        .candidate(
            BTreeMap::from([(key, values)]),
            registry,
            &CancellationToken::new(),
        )
        .expect("owned native source")
}
pub(crate) fn runtime(
    directory: &std::path::Path,
    limit: usize,
) -> Arc<pse_runtime::SharedRuntime> {
    let one = NonZeroUsize::new(1).expect("one");
    pse_runtime::SharedRuntime::build(pse_runtime::ResourceBudget {
        memory_limit_bytes: NonZeroUsize::new(limit).expect("nonzero"),
        spill_dir: directory.to_owned(),
        max_temp_dir_bytes: 64 << 20,
        top_consumers: NonZeroUsize::new(10).expect("ten"),
        threads: ThreadBudget {
            pool_threads: one,
            target_partitions: one,
        },
        execution: ExecutionSettings::default(),
        cache: pse_runtime::DeltaCacheBudget::disabled(1),
        math: Default::default(),
        hashing_may_use_pool: false,

    })
    .expect("runtime")
}
