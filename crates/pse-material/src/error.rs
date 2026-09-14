// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Every error this crate returns, with its blueprint §23.2 class.
//!
//! Material facts are data: a species that names an element nobody declared, or a
//! stoichiometry whose species is not valid in its phase, is a `validation::invariant`
//! failure (§23.2) — the same class a foreign key or a cardinality violation carries. It is
//! not `compile::property`, which is reserved for an unresolved *method* or an undeclared
//! property demand (§9), and it is not `compile::law`, which is a balance binding.
//!
//! Phase validity in particular is four-valued (§7.6): a species that cannot be decided
//! for a phase produces an `inferred.undecided` row and a diagnostic, not an error here.

use pse_ids::SemanticId;

/// Every failure of the material model (blueprint §6.4, §23.2).
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[non_exhaustive]
pub enum MaterialError {
    /// An identity does not resolve in the material system.
    #[error("unknown {kind} `{id}`")]
    #[diagnostic(code(validation::invariant))]
    UnknownId {
        /// What kind of identity it was meant to be (`species`, `phase`, `element`, …).
        kind: &'static str,
        /// The identity that does not resolve.
        id: SemanticId,
    },

    /// A material-system invariant was violated.
    #[error("material rule `{rule}` rejected `{subject}`: {detail}")]
    #[diagnostic(code(validation::invariant))]
    Invariant {
        /// The invariant that refused, named as `closure:species_has_phase` is in §9.1.
        rule: &'static str,
        /// The identity the invariant was applied to.
        subject: SemanticId,
        /// What specifically was wrong.
        detail: String,
    },
}

#[cfg(test)]
mod tests {
    use miette::Diagnostic as _;
    use pse_ids::SemanticId;

    use super::MaterialError;

    fn code_of(error: &MaterialError) -> String {
        error
            .code()
            .map(|code| code.to_string())
            .unwrap_or_default()
    }

    #[test]
    fn each_variant_carries_its_section_23_2_class() {
        assert_eq!(
            code_of(&MaterialError::UnknownId {
                kind: "species",
                id: SemanticId::NIL,
            }),
            "validation::invariant"
        );
        assert_eq!(
            code_of(&MaterialError::Invariant {
                rule: "closure:species_has_phase",
                subject: SemanticId::NIL,
                detail: "no phase admits it".to_owned(),
            }),
            "validation::invariant"
        );
    }

    /// A diagnostic names the identity in hexadecimal, never through `Debug`.
    #[test]
    fn messages_render_identities_not_debug() {
        let error = MaterialError::UnknownId {
            kind: "phase",
            id: SemanticId::from_bytes([0x0b; 16]),
        };
        let rendered = error.to_string();
        assert_eq!(rendered, "unknown phase `0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b`");
    }
}
