// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Which species are valid in which phase (blueprint §6.4, §9.1).
//!
//! A single candidate predicate, not the set-oriented P4 closure executor (§14.2).

use crate::{PhaseType, SpeciesId};

/// The actual explicit phase list, or its known absence. Missing source facts remain
/// unresolved; `Members(&[])` explicitly excludes every species.
#[derive(Clone, Copy, Debug)]
pub enum PhaseRestriction<'a> {
    /// This phase declares no explicit species restriction.
    Unrestricted,
    /// Complete explicit species list for this phase.
    Members(&'a [SpeciesId]),
    /// The phase restriction facts have not been resolved.
    Unresolved,
}

/// Decide one species/phase candidate from the supplied facts (blueprint §6.4, §14.2).
/// `Some(true)` and `Some(false)` are decided; `None` is unknown. Conflict belongs to
/// the multi-rule executor and cannot be established by this single predicate.
///
/// A null `valid_phase_types` selects the declared nonaqueous default. A present empty
/// list excludes all phases. A missing phase type is unknown, distinct from the actual
/// declared `PhaseType::Undefined` enum member. Explicit phase lists further restrict
/// the result using Kleene conjunction, so a known exclusion remains false even when
/// another required fact is unavailable. No electrolyte/apparent-species inference is
/// performed here.
pub fn species_valid_in_phase(
    species: SpeciesId,
    phase: Option<PhaseType>,
    valid_phase_types: Option<&[PhaseType]>,
    restriction: PhaseRestriction<'_>,
) -> Option<bool> {
    let allowed = if valid_phase_types.is_some_and(<[PhaseType]>::is_empty) {
        Some(false)
    } else {
        phase.map(|phase| {
            valid_phase_types.map_or(phase != PhaseType::Aqueous, |types| types.contains(&phase))
        })
    };
    let listed = match restriction {
        PhaseRestriction::Unrestricted => Some(true),
        PhaseRestriction::Members(members) => Some(members.contains(&species)),
        PhaseRestriction::Unresolved => None,
    };
    match (allowed, listed) {
        (Some(false), _) | (_, Some(false)) => Some(false),
        (Some(true), Some(true)) => Some(true),
        _ => None,
    }
}
