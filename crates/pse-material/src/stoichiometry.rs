// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Stoichiometry helpers (blueprint §6.4).
//!
//! Element residuals are diagnostics; an unbalanced authored reaction remains admissible.

use crate::element::{ElementCount, ElementTable, admit_composition};
use crate::{ElementId, MaterialError, PhaseId, ReactionId, SpeciesId};
use pse_quantity::BasisKind;
use std::collections::{BTreeMap, BTreeSet};

/// An explicit `authored.stoichiometry` row; positive coefficients denote products.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stoichiometry {
    /// Actual reaction identity.
    pub reaction: ReactionId,
    /// Actual phase identity; retained so distinct phase contributions are not collapsed.
    pub phase: PhaseId,
    /// Actual species identity.
    pub species: SpeciesId,
    /// Finite nonzero signed coefficient in the reaction's declared basis.
    pub coefficient: f64,
}

/// Compute signed element residuals for an explicitly molar reaction. Zero is balanced;
/// every nonzero residual is returned as data, never rejected as a material invariant.
/// No numerical tolerance or mass-to-molar basis conversion is invented. Terms are
/// accumulated in phase/species identity order; absent compositions are unresolved,
/// not assumed to contain no elements.
/// # Errors
/// Nonmolar basis (conversion is deferred), wrong/duplicate reaction rows, malformed
/// counts/coefficients, unknown elements/compositions, or nonfinite arithmetic.
pub fn element_balance(
    elements: &ElementTable,
    reaction: ReactionId,
    basis: BasisKind,
    terms: &[Stoichiometry],
    compositions: &BTreeMap<SpeciesId, Vec<ElementCount>>,
) -> Result<BTreeMap<ElementId, f64>, MaterialError> {
    if basis != BasisKind::Molar {
        return Err(invalid(
            reaction,
            "element balance requires an explicit molar basis; basis conversion is deferred",
        ));
    }
    let mut keys = BTreeSet::new();
    for term in terms {
        if term.reaction != reaction {
            return Err(invalid(
                reaction,
                "stoichiometry row belongs to a different reaction",
            ));
        }
        if !term.coefficient.is_finite() || term.coefficient == 0.0 {
            return Err(invalid(
                reaction,
                "stoichiometric coefficient must be finite and nonzero",
            ));
        }
        if !keys.insert((term.phase, term.species)) {
            return Err(invalid(reaction, "duplicate reaction/phase/species entry"));
        }
    }
    let mut ordered: Vec<_> = terms.iter().collect();
    ordered.sort_by_key(|term| (term.phase, term.species));
    let mut residuals = BTreeMap::new();
    for term in ordered {
        let composition = compositions
            .get(&term.species)
            .filter(|rows| !rows.is_empty())
            .ok_or(MaterialError::UnknownId {
                kind: "species composition",
                id: term.species.as_id(),
            })?;
        for item in admit_composition(elements, composition)? {
            let residual = residuals.entry(item.element).or_insert(0.0_f64);
            *residual += term.coefficient * item.count;
            if !residual.is_finite() {
                return Err(invalid(reaction, "element-balance arithmetic is nonfinite"));
            }
        }
    }
    Ok(residuals)
}

fn invalid(reaction: ReactionId, detail: &str) -> MaterialError {
    MaterialError::Invariant {
        rule: "material.stoichiometry_input",
        subject: reaction.as_id(),
        detail: detail.to_owned(),
    }
}
