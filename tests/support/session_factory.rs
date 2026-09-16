// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit native engine assembly for small catalog contract fixtures.

use pse_catalog::session::{
    ExecutionSettings, SessionFactory, ThreadBudget, native_engine_profile,
};
use pse_ids::MemoryReserver;
use std::{num::NonZeroUsize, sync::Arc};

#[expect(
    clippy::expect_used,
    reason = "fixed fixture configuration must construct its native engine"
)]
pub(crate) fn factory(reserver: Arc<dyn MemoryReserver>) -> Arc<SessionFactory> {
    Arc::new(
        SessionFactory::new(
            Arc::new(datafusion::execution::runtime_env::RuntimeEnv::default()),
            reserver,
            ExecutionSettings::default(),
            ThreadBudget {
                pool_threads: NonZeroUsize::MIN,
                target_partitions: NonZeroUsize::MIN,
            },
            native_engine_profile(),
        )
        .expect("native fixture engine"),
    )
}
