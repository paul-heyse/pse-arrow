// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Null defaults, explicit phase membership and unresolved material facts.
use pse_ids::SemanticId;
use pse_material::phase_validity::{PhaseRestriction, species_valid_in_phase};
use pse_material::{PhaseType, SpeciesId};

fn species(value: u8) -> SpeciesId {
    SpeciesId::from_id(SemanticId::from_bytes([value; 16]))
}

#[test]
fn null_default_explicit_empty_and_explicit_membership_remain_distinct() {
    let candidate = species(1);
    for phase in PhaseType::ALL {
        assert_eq!(
            species_valid_in_phase(
                candidate,
                Some(*phase),
                None,
                PhaseRestriction::Unrestricted
            ),
            Some(*phase != PhaseType::Aqueous)
        );
        assert_eq!(
            species_valid_in_phase(
                candidate,
                Some(*phase),
                Some(&[]),
                PhaseRestriction::Unrestricted
            ),
            Some(false)
        );
        assert_eq!(
            species_valid_in_phase(
                candidate,
                Some(*phase),
                Some(&[*phase]),
                PhaseRestriction::Unrestricted
            ),
            Some(true)
        );
    }
    assert_eq!(
        species_valid_in_phase(
            candidate,
            Some(PhaseType::Vapor),
            Some(&[PhaseType::Liquid]),
            PhaseRestriction::Unrestricted
        ),
        Some(false)
    );
}

#[test]
fn actual_species_restrictions_and_unresolved_facts_use_kleene_conjunction() {
    let candidate = species(1);
    let other = species(2);
    assert_eq!(
        species_valid_in_phase(
            candidate,
            Some(PhaseType::Liquid),
            None,
            PhaseRestriction::Members(&[candidate])
        ),
        Some(true)
    );
    for members in [&[][..], &[other][..]] {
        assert_eq!(
            species_valid_in_phase(
                candidate,
                Some(PhaseType::Liquid),
                None,
                PhaseRestriction::Members(members)
            ),
            Some(false)
        );
        assert_eq!(
            species_valid_in_phase(candidate, None, None, PhaseRestriction::Members(members)),
            Some(false)
        );
    }
    assert_eq!(
        species_valid_in_phase(candidate, None, None, PhaseRestriction::Unrestricted),
        None
    );
    assert_eq!(
        species_valid_in_phase(
            candidate,
            Some(PhaseType::Liquid),
            None,
            PhaseRestriction::Unresolved
        ),
        None
    );
    assert_eq!(
        species_valid_in_phase(
            candidate,
            Some(PhaseType::Aqueous),
            None,
            PhaseRestriction::Unresolved
        ),
        Some(false)
    );
    assert_eq!(
        species_valid_in_phase(candidate, None, Some(&[]), PhaseRestriction::Unresolved),
        Some(false)
    );
}
