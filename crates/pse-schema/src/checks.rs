// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Cross-cutting registry checks that need the whole assembled registry.
//!
//! The per-declaration checks live in [`crate::builder`], where the declaration that
//! fails is still in hand. This module is for the ones that are only decidable once
//! everything is declared — rule stratification against the writers of each relation
//! (blueprint §14.2 rule 2, [`crate::SchemaError::RuleStratification`]) and the stage-graph
//! reachability rules of §14.1 beyond one pass's own ports.
//!
//! Packet A-4 fills this when it declares the pass and rule catalogs; with no rules and no
//! passes declared there is nothing for it to decide yet.

use crate::builder::Registry;
use crate::error::SchemaError;

/// Runs every whole-registry check.
///
/// # Errors
///
/// [`SchemaError::RuleStratification`] and [`SchemaError::StageGraph`] once packet A-4
/// declares the rule and pass catalogs this check reads.
#[expect(
    clippy::unnecessary_wraps,
    reason = "the signature is the seam packet A-4 fills; making it infallible now would make adding the first check a breaking change for every caller"
)]
pub(crate) fn run(_registry: &Registry) -> Result<(), SchemaError> {
    Ok(())
}
