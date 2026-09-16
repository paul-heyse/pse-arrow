// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The snapshot catalog and the artifact store (blueprint §5.4, §20).
//!
//! # The two halves and the line between them
//!
//! [`store`] owns bytes: where an artifact lives (§20.1), what a manifest says (§20.2),
//! and the single-writer publication protocol that makes a ref move only after every
//! object it names exists. [`provider`] and [`session`] own queries: a
//! `CatalogProviderList` whose catalogs are snapshots, whose schemas are the seven
//! namespaces and whose tables are relations (§5.4).
//!
//! [`snapshot::Snapshot`] is the seam. It is produced by the store, consumed by the
//! providers, and immutable in between — which is what lets §5.4 promise that "a session
//! pins a snapshot so schemas cannot change mid-query" without any locking at all.
//!
//! # Three rules this crate exists to keep
//!
//! - **Logical content and physical bytes have different identities** (ADR-0045). A
//!   `logical_hash` identifies admitted logical content; `encoding_checksum` checks
//!   stored bytes. Neither certifies schema, values, contextual invariants or producer
//!   semantics: these are checked directly before admission or reuse.
//! - **A snapshot never contains itself** (§5.3 step 7). Refs, manifests and stage records
//!   are sidecars; [`store::layout::SIDECAR_DIRECTORIES`] names them and
//!   [`store::manifest::Manifest::validate`] refuses a manifest that points into one.
//! - **Engine failures are classified once** (§23.2). [`failure::classify`] is the only
//!   place a `DataFusionError` becomes a [`CatalogError`], because a mapping repeated at
//!   every call site converges on `internal::invariant` and stops being a taxonomy.
//!
//! # Arrow, once
//!
//! Every Arrow type here is reached through `datafusion::arrow::…`. Two resolved `arrow`
//! majors in one graph make `downcast_ref` return `None` with no compile error, and the
//! resulting bug reads like a logic error somewhere else entirely (§3.1).
//!
//! # Implemented foundation boundaries
//!
//! The store admits complete actual relations and explicit parent contexts, writes
//! finished artifacts, and publishes conditional refs. Sealed providers and sessions
//! retain those admitted inputs. Plan protobufs are diagnostic artifacts. Optional
//! evidence helpers and undeclared run producers remain outside this foundation.

pub mod computation;
pub mod contract;
pub mod delta;
pub mod error;
pub mod evidence;
pub mod failure;
pub mod source_production;
#[rustfmt::skip]
pub mod generated;
pub mod inspection;
pub mod provider;
pub mod session;
pub mod snapshot;
pub mod store;

pub use crate::contract::{EncodingPolicy, RelationContract};
pub use crate::error::CatalogError;
pub use crate::failure::{PlanOrigin, classify};
pub use crate::provider::BoxFut;
pub use crate::session::{ExecutionSettings, ThreadBudget};
pub use crate::snapshot::{LoadedRelation, Snapshot, TrustLevel};
pub use crate::store::clock::{Clock, FixedClock, SystemClock};
pub use crate::store::layout::{
    EncodingFormat, RefName, evidence_path, manifest_path, ref_path, relation_path, stage_path,
};
pub use crate::store::manifest::{
    CompilerRef, EncodingRecord, EngineProfileRef, EvidenceRecord, KernelRef, MANIFEST_VERSION,
    Manifest, NumericalPolicyRef, PackageRef, PassRef, RelationMember, ToolchainRef,
};

/// Immutable artifact catalog with active row admission.
pub use crate::store::open::Catalog;
