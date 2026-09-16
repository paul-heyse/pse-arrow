// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use datafusion::execution::runtime_env::RuntimeEnv;
use pse_catalog::session::{
    ExecutionSettings, SessionFactory, SnapshotSession, ThreadBudget, native_engine_profile,
};
use pse_ids::{CancellationToken, FixedBudget};
use pse_schema::Registry;
use std::{
    collections::BTreeMap,
    num::NonZeroUsize,
    sync::{Arc, OnceLock},
};

pub(super) fn session(registry: Arc<Registry>, budget: Arc<FixedBudget>) -> SnapshotSession {
    let threads = NonZeroUsize::new(1).unwrap();
    SessionFactory::new(
        Arc::new(RuntimeEnv::default()),
        budget,
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: threads,
            target_partitions: threads,
        },
        native_engine_profile(),
    )
    .unwrap()
    .candidate(BTreeMap::new(), registry, &CancellationToken::new())
    .unwrap()
}

pub(super) fn registry() -> Arc<Registry> {
    static REGISTRY: OnceLock<Arc<Registry>> = OnceLock::new();
    Arc::clone(REGISTRY.get_or_init(|| Arc::new(pse_schema::catalog::assemble().unwrap())))
}
