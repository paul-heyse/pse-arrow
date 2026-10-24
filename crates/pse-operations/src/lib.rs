// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Canonical problem revisions, selected compilation, durable execution and graph results.
//! The registry owns native schemas; guarded operations own admission and immutable replay.

pub mod canonical;
pub mod canonical_analyses;
pub mod canonical_artifacts;
#[cfg(all(test, feature = "canonical-tests"))]
mod canonical_capability_tests;
pub mod canonical_codec;
pub mod canonical_contexts;
pub mod canonical_execution;
pub mod canonical_explanations;
pub mod canonical_files;
pub mod canonical_history;
pub mod canonical_lifecycle;
pub mod canonical_project_imports;
pub mod canonical_projects;
pub mod canonical_research;
pub mod canonical_result_retention;
pub mod canonical_results;
pub mod canonical_retention;
pub mod canonical_selection;
pub mod canonical_staging;
pub mod canonical_studies;
pub mod canonical_study_workbench;
pub mod evidence;
/// Supplied-only selected evidence questions using the native columnar engine.
#[cfg(feature = "evidence-query")]
pub mod evidence_query;
mod ids;
/// Disposable typed development-knowledge corpora on the existing tooling connection.
pub mod knowledge;
/// Registry-generated native schema and codec contracts.
#[rustfmt::skip]
pub mod generated;
#[cfg(any(test, feature = "test-support"))]
pub mod testing;
pub use ids::mint_id;
