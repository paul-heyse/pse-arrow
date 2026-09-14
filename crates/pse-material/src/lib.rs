// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Species, elements, phases, phase-species, reactions, stoichiometry and material systems
//! (blueprint §3.2, §6.4, §9).
//!
//! Depends on no engine crate: material facts are primitive relations (§3.2). What lives
//! here is the in-memory shape of §6.4's rows and the predicates over them — not a
//! property package, which is a template binding (§9.2), and not a thermodynamic model,
//! which is a kernel (§9).
//!
//! # Identity, and why a species is not its name
//!
//! `H2O` is a formula, not an identity: two packages may both declare water with different
//! reference states and different methods, and a rename of either must leave every
//! derivation intact (§5.1). So a [`SpeciesId`] is assigned when the species is written and
//! every relation joins on it. An element is different — `reference.elements` is a named
//! package, so an [`ElementId`] is the named-policy identity of its symbol and is stable
//! across packages by construction.
//!
//! # Layout
//!
//! Landed by the keel (packet K-4):
//!
//! - [`ids`] — [`ElementId`], [`SpeciesId`], [`PhaseId`], [`ReactionId`].
//! - [`enums`] — the §6.14 enumerations, spelled as IDAES spells them.
//! - [`error`] — [`MaterialError`] with its §23.2 class.
//!
//! Declared here and filled by packet Q-material: [`element`], [`phase_validity`],
//! [`stoichiometry`], and `testdata` behind `cfg(any(test, feature = "fixtures"))`.

pub mod element;
pub mod enums;
pub mod error;
pub mod ids;
pub mod phase_validity;
pub mod stoichiometry;
#[cfg(any(test, feature = "fixtures"))]
pub mod testdata;

pub use crate::enums::{ComponentType, ConcentrationForm, HenryType, PhaseType, ReactionKind};
pub use crate::error::MaterialError;
pub use crate::ids::{ElementId, PhaseId, ReactionId, SpeciesId};
