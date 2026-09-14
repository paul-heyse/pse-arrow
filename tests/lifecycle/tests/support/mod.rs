// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Common actual catalog and shared runtime fixture construction.
#![allow(
    dead_code,
    clippy::expect_used,
    reason = "shared integration factories with fixed valid declarations"
)]

use pse_catalog::store::{
    membership::AdmissionContext,
    open::Catalog,
    publish::{BundleDraft, RelationDraft},
};
use pse_catalog::{
    EncodingPolicy, ExecutionSettings, FixedClock, RelationContract, ThreadBudget, TrustLevel,
};
use pse_ids::{CancellationToken, MemoryReserver, SnapshotKind};
use pse_schema::{
    Registry, RegistryBuilder,
    model::{Authority, Cell, ColumnSpec, LogicalType, Namespace, RelationDecl, SnapshotClass},
};
use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::sync::Arc;

pub(crate) fn registry() -> Arc<Registry> {
    let mut builder = RegistryBuilder::new();
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
            ColumnSpec::key("id", LogicalType::U64, "Key"),
            ColumnSpec::payload("value", LogicalType::U64, "Value"),
        ]),
    );
    Arc::new(builder.build().expect("registry"))
}
pub(crate) fn catalog(
    store: Arc<dyn object_store::ObjectStore>,
    reg: Arc<Registry>,
    reserver: Arc<dyn MemoryReserver>,
) -> Catalog {
    Catalog::open(
        store,
        reg,
        TrustLevel::Untrusted,
        Arc::new(FixedClock("2026-09-14T00:00:00Z".to_owned())),
        reserver,
    )
}
pub(crate) fn draft(
    catalog: &Catalog,
    count: u64,
    offset: u64,
    cancel: &CancellationToken,
) -> BundleDraft {
    let spec = catalog
        .registry()
        .relation("authored.samples")
        .expect("relation");
    let rows: Vec<_> = (0..count)
        .map(|id| vec![Cell::U64(id), Cell::U64(count - id + offset)])
        .collect();
    let batch = pse_relations::cells::batch_from_cells_owned(
        catalog.registry(),
        spec,
        &rows,
        catalog.reserver().as_ref(),
        cancel,
    )
    .expect("bounded typed rows");
    BundleDraft {
        manifest: catalog
            .manifest_template(SnapshotKind::Model, &AdmissionContext::default())
            .expect("template"),
        relations: BTreeMap::from([(
            pse_ids::model_port_name("authored", spec.id),
            RelationDraft {
                contract: Arc::new(
                    RelationContract::from_spec(catalog.registry(), spec, EncodingPolicy::IpcFile)
                        .expect("contract"),
                ),
                batches: vec![batch],
            },
        )]),
        context: AdmissionContext::default(),
    }
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
        hashing_may_use_pool: false,
    })
    .expect("runtime")
}
