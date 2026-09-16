// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The semantic schema registry: the single authority for every relation shape
//! (blueprint §4).
//!
//! A relation is declared exactly once, as a [`model::RelationSpec`]. Everything else —
//! the Arrow schema, the typed view, the builder, the validator, the provider, the
//! documentation, the migration — is generated from that declaration (§4.2), and the
//! registry is itself stored as relations (`reference.schema_*`, §4.1) so the platform can
//! query its own schema, diff versions and generate migrations.
//!
//! # Why the registry describes itself
//!
//! Because the alternative is two descriptions. A registry that could not emit its own
//! rows would need a second, hand-written account of what it declares for the manifest,
//! the documentation and the Python contracts — and nothing would tell you which of the
//! two was authoritative when they disagreed. [`builder::Registry::schema_rows`] is the
//! one account; [`fingerprint::registry`] is its digest; `pse-relations` materializes the
//! same rows into `RecordBatch`es.
//!
//! # Layout
//!
//! - [`model`] — the declaration types: relations, columns, logical types, extension
//!   types, enumerations, invariants, migrations, passes, rules, cells, the manifest.
//! - [`catalog`] — every declaration the platform ships, in one fixed order.
//! - [`builder`] — assembly: declarations in, an immutable registry out.
//! - [`fingerprint`] — the registry and per-relation digests (ADR-0050).
//! - [`arrow`] — the Arrow schema of a declared relation, metadata attached at
//!   construction (§4.3).
//! - [`ext_metadata`] — the canonical `ARROW:extension:metadata` strings (§4.4).
//! - [`membership`] — snapshot membership, generated from `snapshot_class` (§5.3 step 7).
//! - [`codegen`] — the generated trees (ADR-0031, ADR-0051).
//! - [`error`] — [`SchemaError`] with its §23.2 codes.
//!
//! # Example
//!
//! ```
//! use pse_schema::registry;
//!
//! let reg = registry()?;
//! let spec = reg.relation("reference.schema_relations").expect("declared in §4.1");
//! // The fingerprint is filled at assembly; a hand-built spec would carry a meaningless one.
//! assert_ne!(spec.fingerprint, pse_ids::ContentHash::NIL);
//! # Ok::<(), pse_schema::SchemaError>(())
//! ```

pub mod arrow;
pub mod builder;
pub mod catalog;
mod checks;
pub mod codegen;
pub mod compiled_contract;
pub mod delta;
pub mod error;
pub mod ext_metadata;
pub mod field_contract;
pub mod fingerprint;
pub mod math;
pub mod membership;
pub mod model;
mod rule_deps;

use std::sync::{Arc, OnceLock};

pub use crate::builder::{REGISTRY_PACKAGE_ID, REGISTRY_PACKAGE_NAME, Registry, RegistryBuilder};
pub use crate::error::SchemaError;

/// The assembled registry, built once per process.
///
/// The `Result` is memoized rather than the `Registry`: a failed assembly is a
/// deterministic property of the declarations, so re-running it would produce the same
/// failure at a cost, and the second caller deserves the same diagnosis as the first.
static REGISTRY: OnceLock<Result<Arc<Registry>, SchemaError>> = OnceLock::new();

/// The registry.
///
/// # Errors
///
/// The errors of [`catalog::assemble`]: a duplicate declaration, a dangling reference, a
/// derived relation without a granularity, a broken stage graph or a rule that keys on a
/// float.
///
/// ```
/// use pse_schema::registry;
///
/// // Assembly is a pure function of the declarations, so two calls are the same registry.
/// assert_eq!(registry()?.fingerprint(), registry()?.fingerprint());
/// # Ok::<(), pse_schema::SchemaError>(())
/// ```
pub fn registry() -> Result<&'static Registry, SchemaError> {
    match REGISTRY.get_or_init(|| catalog::assemble().map(Arc::new)) {
        Ok(registry) => Ok(registry),
        Err(error) => Err(error.clone()),
    }
}

/// Shared ownership of the same declaration registry for native execution bindings.
/// # Errors
/// The errors of [`catalog::assemble`].
pub fn shared_registry() -> Result<Arc<Registry>, SchemaError> {
    REGISTRY
        .get_or_init(|| catalog::assemble().map(Arc::new))
        .clone()
}
