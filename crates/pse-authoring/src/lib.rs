// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Package and case document loading, the expression DSL and change sets
//! (blueprint §3.2, §11, §22).
//!
//! Documents are parsed with spans, so every authoring failure carries a
//! [`span::SourceSpan`] and every authored row can point at the text that produced it
//! (§23.2 `authoring.parse`). YAML is one surface: the same change sets can be produced by
//! Python builders and by agents, which is why the change set — not the document — is the
//! write path (§22.2).
//!
//! # Layout
//!
//! - [`error`] — [`AuthoringError`] with its §23.2 codes.
//! - [`span`] — source spans and the parse budget.
//! - [`dsl`] — the expression DSL: parse, render, round trip.
//! - [`generated::documents`] — strict typed document contracts.
//! - [`ids`] — entity identity assignment under both policies.
//! - Native source edits preserve exact before-images; provider commands own writes.
//! - [`targets`] — `pse.target_path` parsing and resolution.
//! - [`p0`] — pure exact package resolution.
//!
//! `generated` is added together with the first generated `documents.rs` (packet A-6).

pub mod dsl;
pub mod error;
mod grammar;
pub mod ids;
pub mod p0;
pub mod span;
pub mod targets;

pub use crate::error::AuthoringError;
pub use crate::span::{ParseBudget, SourceSpan};

/// Authoring document declarations projected from the registry.
#[rustfmt::skip]
pub mod generated;
