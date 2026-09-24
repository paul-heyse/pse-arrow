// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The selected durable planner composed with the production native fixture factory.
#![allow(
    clippy::unwrap_used,
    reason = "fixed isolated catalog fixture must construct"
)]
use datafusion::prelude::SessionContext;
use std::sync::Arc;

pub(crate) fn context() -> SessionContext {
    let factory = pse_testkit::NativeFixture::new((256 << 20).try_into().unwrap())
        .unwrap()
        .into_factory()
        .with_query_planner(Arc::new(pse_engine::session::planner::UnifiedPlanner::new(
            pse_catalog::assembly::planners(),
        )));
    // These raw DataFusion fixtures register tables directly. Production sessions
    // install their captured catalog separately and deliberately disable defaults.
    let config = factory
        .native_state()
        .config()
        .clone()
        .with_default_catalog_and_schema("datafusion", "public")
        .with_create_default_catalog_and_schema(true);
    SessionContext::new_with_state(
        datafusion::execution::session_state::SessionStateBuilder::from(
            factory.native_state().clone(),
        )
        .with_config(config)
        .build(),
    )
}
