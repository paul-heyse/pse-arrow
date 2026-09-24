// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use pse_columnar::CancellationToken;
use pse_engine::session::{EngineSession, ExecutionSettings, ThreadBudget};
use pse_schema::Registry;
use std::{
    collections::BTreeMap,
    num::NonZeroUsize,
    sync::{Arc, OnceLock},
};

pub(super) fn session(
    registry: Arc<Registry>,
    budget: Arc<dyn pse_columnar::MemoryPool>,
) -> EngineSession {
    let threads = NonZeroUsize::new(1).unwrap();
    pse_testkit::factory(
        budget,
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: threads,
            target_partitions: threads,
        },
    )
    .unwrap()
    .candidate(BTreeMap::new(), registry, &CancellationToken::new())
    .unwrap()
}

pub(super) fn registry() -> Arc<Registry> {
    static REGISTRY: OnceLock<Arc<Registry>> = OnceLock::new();
    Arc::clone(REGISTRY.get_or_init(|| Arc::new(pse_schema::catalog::assemble().unwrap())))
}
