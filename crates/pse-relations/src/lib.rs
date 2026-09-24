// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed relation views, builders, validators and migrations (blueprint §3.2, §4.2).
//!
//! Relation views are the only model authority: nothing hand-written may shadow a
//! generated struct (governance `no_shadow_structs`). Two definitions of one row shape is
//! how a contract silently forks — one of them gets a field, the other does not, and both
//! compile.
//!
//! # Layout
//!
//! - [`error`] — [`RelationError`] with its §23.2 codes.
//! - [`ext`] — the eleven `pse.*` `ExtensionType` implementations and their metadata codec.
//! - [`validate`] — the recursive field, schema and batch validators.
//! - [`columnar`] — native borrowed views and generated array builders.
//! - [`registry_relations`] — the registry materialized as batches.
//!
//! `generated` is added by packet A-6 together with the first generated tree; it is
//! deliberately absent until then, because a module declaring an empty generated
//! directory would make `codegen --check` pass over nothing.

pub mod canonical;
pub mod columnar;
pub mod error;
pub mod ext;
pub mod identity;

pub mod registry_relations;
#[cfg(any(test, feature = "test-support"))]
pub mod testing;
pub mod validate;
/// Typed contracts generated from the authoritative registry.
#[rustfmt::skip]
pub mod generated;


pub use crate::error::RelationError;

/// The Arrow types this crate's API speaks.
///
/// Re-exported rather than re-declared so that a downstream crate binds to the one
/// resolved Arrow version the workspace pins. Two majors in the graph make `downcast_ref`
/// return `None` with no compile error (blueprint §3.1), and a crate that reached for its
/// own `arrow` dependency to name a `RecordBatch` is exactly how the second major gets in.
pub use arrow_array::RecordBatch;
pub use arrow_schema::{DataType, Field, FieldRef, Schema, SchemaRef};

mod native;
