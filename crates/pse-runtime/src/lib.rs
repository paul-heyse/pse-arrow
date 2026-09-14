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
//! through [`pse_ids::MemoryReserver`] *before* allocating. What neither covers is the
//! global allocator, a solver process and any Arrow array built outside a reservation;
//! §14.3 says so, and the peak reporting keeps the accounted peak and the process peak as
//! two numbers for exactly that reason.
//!
//! # Phase 0
//!
//! The error taxonomy and the resource budget are implemented. The shared runtime, the
//! pool-backed reserver, peak reporting, cancellation and the session factory are declared
//! and empty; each names the packet that fills it.

pub mod budget;
pub mod cancel;
pub mod env;
pub mod error;
pub mod peak;
pub mod reserve;
pub mod session_factory;

pub use crate::budget::ResourceBudget;
pub use crate::error::RuntimeError;
