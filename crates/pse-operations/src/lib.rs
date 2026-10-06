// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Canonical problem revisions, selected compilation, durable execution and graph results.
//! The registry owns native schemas; guarded operations own admission and immutable replay.

pub mod canonical;
pub mod canonical_analyses;
pub mod canonical_codec;
pub mod canonical_execution;
pub mod canonical_result_retention;
pub mod canonical_results;
pub mod canonical_retention;
pub mod canonical_selection;
pub mod canonical_staging;
pub mod canonical_studies;
mod ids;
pub mod study_policy;
/// Registry-generated native schema and codec contracts.
#[rustfmt::skip]
pub mod generated;
#[cfg(any(test, feature = "test-support"))]
pub mod testing;
pub use ids::mint_id;
