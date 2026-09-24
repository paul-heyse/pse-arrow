// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The run controller: one shared accounted runtime, the sessions built on it, and the
//! lifecycle that cancels them (blueprint §14.3, §18.8, §20.3, ADR-0046).
//!
//! # One budget, many sessions
//!
//! ADR-0046's decision is that a process has *one* configured `RuntimeEnv` and one
//! accounted budget, shared by every snapshot session. The alternative was rejected for a
//! concrete reason: with a budget per session, two concurrent sessions each consume a full
//! deployment budget, and a limit that multiplies by the number of sessions is not a
//! limit. [`budget::ResourceBudget`] is that one configuration, and it is validated
//! before anything is built from it rather than discovered wrong at the first spill.
//!
//! # What "accounted" covers
//!
//! DataFusion's pool governs query operators. Platform allocations — canonicalization
//! copies, binding buffers, result ingestion — are invisible to it, so they reserve
//! through [`pse_columnar::MemoryPool`] *before* allocating. What neither covers is the
//! global allocator, a solver process and any Arrow array built outside a reservation;
//! §14.3 says so, and the peak reporting keeps the accounted peak and the process peak as
//! two numbers for exactly that reason.
//!
//! # Foundation scope
//!
//! The finite shared runtime, pool-backed reservations, peak reporting and cancellation
//! are implemented. Session factories bind the same runtime and reservation adapter to
//! admitted snapshots or explicitly unpublished candidate rows.

pub mod budget;
pub mod cancel;
pub mod env;
pub mod error;
pub mod peak;
pub mod session_factory;
pub mod settings;

pub use crate::budget::{DeltaCacheBudget, ResourceBudget};
pub use crate::error::RuntimeError;

pub use crate::cancel::CancelSource;
pub use crate::env::SharedRuntime;
pub use crate::peak::ResourceReport;

/// Effectful authoring adapters around pure parser and generated document values.
pub mod authoring_driver;
/// Physical declarations admitted once from actual typed source rows.
pub mod physical;
#[rustfmt::skip]
mod generated;

/// Finite authored definitions into library-owned mathematical bodies.
pub mod math;
/// Typed native modeling, completion-owned jobs, physical results and explicit publication.
pub mod workflow;
