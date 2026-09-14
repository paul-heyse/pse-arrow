// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Element and phase-validity test data (blueprint §6.4).
//!
//! Small explicit fixtures, never a default periodic-table or material-package authority.

use crate::element::{Element, ElementCount, ElementTable};
use crate::stoichiometry::Stoichiometry;
use crate::{ElementId, MaterialError, PhaseId, PhaseType, ReactionId, SpeciesId};
use pse_ids::SemanticId;
use std::collections::BTreeMap;

/// Explicit hydrogen/oxygen/water fixture with rounded test masses in kg/mol.
#[derive(Debug)]
pub struct WaterReaction {
    /// Actual element rows.
    pub elements: ElementTable,
    /// Hydrogen and oxygen element identities, in that order.
    pub element_ids: [ElementId; 2],
    /// Hydrogen, oxygen and water species identities, in that order.
    pub species: [SpeciesId; 3],
    /// Actual reaction identity.
    pub reaction: ReactionId,
    /// Complete explicit compositions.
    pub compositions: BTreeMap<SpeciesId, Vec<ElementCount>>,
    /// `2 H2 + O2 -> 2 H2O` in one declared phase, on a molar basis.
    pub terms: Vec<Stoichiometry>,
}

/// Build the independently inspectable fixture rows.
/// # Errors
/// Invalid fixture element declarations.
pub fn water_reaction() -> Result<WaterReaction, MaterialError> {
    let hydrogen = ElementId::from_id(sid(1));
    let oxygen = ElementId::from_id(sid(2));
    let elements = ElementTable::new([
        Element {
            id: hydrogen,
            symbol: "H".into(),
            name: "Hydrogen".into(),
            atomic_mass: 0.001,
        },
        Element {
            id: oxygen,
            symbol: "O".into(),
            name: "Oxygen".into(),
            atomic_mass: 0.016,
        },
    ])?;
    let species = [10, 11, 12].map(|value| SpeciesId::from_id(sid(value)));
    let compositions = BTreeMap::from([
        (
            species[0],
            vec![ElementCount {
                element: hydrogen,
                count: 2.0,
            }],
        ),
        (
            species[1],
            vec![ElementCount {
                element: oxygen,
                count: 2.0,
            }],
        ),
        (
            species[2],
            vec![
                ElementCount {
                    element: hydrogen,
                    count: 2.0,
                },
                ElementCount {
                    element: oxygen,
                    count: 1.0,
                },
            ],
        ),
    ]);
    let reaction = ReactionId::from_id(sid(20));
    let phase = PhaseId::from_id(sid(30));
    let terms = species
        .into_iter()
        .zip([-2.0, -1.0, 2.0])
        .map(|(species, coefficient)| Stoichiometry {
            reaction,
            phase,
            species,
            coefficient,
        })
        .collect();
    Ok(WaterReaction {
        elements,
        element_ids: [hydrogen, oxygen],
        species,
        reaction,
        compositions,
        terms,
    })
}

/// One single-predicate fixture with decided or unresolved phase facts.
#[derive(Debug)]
pub struct PhaseCase {
    /// Human-readable condition.
    pub name: &'static str,
    /// Actual declared phase, or unresolved input.
    pub phase: Option<PhaseType>,
    /// Null default versus an explicit membership list.
    pub valid_phase_types: Option<Vec<PhaseType>>,
    /// Expected three-valued result without an explicit species restriction.
    pub expected: Option<bool>,
}

/// Default, explicit-empty, explicit-aqueous and unresolved phase cases.
pub fn phase_cases() -> Vec<PhaseCase> {
    let mut cases: Vec<_> = PhaseType::ALL
        .iter()
        .map(|phase| PhaseCase {
            name: "null default",
            phase: Some(*phase),
            valid_phase_types: None,
            expected: Some(*phase != PhaseType::Aqueous),
        })
        .collect();
    cases.extend([
        PhaseCase {
            name: "explicit empty",
            phase: Some(PhaseType::Liquid),
            valid_phase_types: Some(vec![]),
            expected: Some(false),
        },
        PhaseCase {
            name: "explicit aqueous",
            phase: Some(PhaseType::Aqueous),
            valid_phase_types: Some(vec![PhaseType::Aqueous]),
            expected: Some(true),
        },
        PhaseCase {
            name: "unresolved phase",
            phase: None,
            valid_phase_types: None,
            expected: None,
        },
    ]);
    cases
}

fn sid(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}

#[cfg(test)]
mod tests {
    use super::{phase_cases, sid, water_reaction};
    use crate::element::{ElementCount, ElementTable, molecular_weight};
    use crate::phase_validity::{PhaseRestriction, species_valid_in_phase};
    use crate::stoichiometry::element_balance;
    use crate::{ElementId, PhaseId};
    use pse_quantity::BasisKind;

    #[test]
    fn phase_fixture_expectations_are_exercised() {
        let fixture = water_reaction().unwrap();
        for case in phase_cases() {
            assert_eq!(
                species_valid_in_phase(
                    fixture.species[2],
                    case.phase,
                    case.valid_phase_types.as_deref(),
                    PhaseRestriction::Unrestricted
                ),
                case.expected,
                "{}",
                case.name
            );
        }
    }

    #[test]
    fn molecular_weight_uses_explicit_rows_and_recomputes_changed_atomic_masses() {
        let fixture = water_reaction().unwrap();
        let composition = &fixture.compositions[&fixture.species[2]];
        let original = molecular_weight(&fixture.elements, composition)
            .unwrap()
            .unwrap();
        assert!((original - 0.018).abs() < 1e-15);
        let mut reordered = composition.clone();
        reordered.reverse();
        assert_eq!(
            molecular_weight(&fixture.elements, &reordered).unwrap(),
            Some(original)
        );
        assert_eq!(molecular_weight(&fixture.elements, &[]).unwrap(), None);
        let mut elements: Vec<_> = fixture.elements.elements().cloned().collect();
        elements[0].atomic_mass = 0.002;
        let changed = ElementTable::new(elements).unwrap();
        assert!((molecular_weight(&changed, composition).unwrap().unwrap() - 0.020).abs() < 1e-15);
        let fractional = [ElementCount {
            element: fixture.element_ids[1],
            count: 0.5,
        }];
        assert_eq!(
            molecular_weight(&fixture.elements, &fractional).unwrap(),
            Some(0.008)
        );
    }

    #[test]
    fn malformed_element_and_composition_rows_are_refused_before_arithmetic() {
        let fixture = water_reaction().unwrap();
        let element = fixture.elements.elements().next().unwrap().clone();
        assert!(ElementTable::new([element.clone(), element.clone()]).is_err());
        for mass in [0.0, -1.0, f64::INFINITY, f64::NAN] {
            let mut invalid = element.clone();
            invalid.atomic_mass = mass;
            assert!(ElementTable::new([invalid]).is_err());
        }
        let item = ElementCount {
            element: element.id,
            count: 1.0,
        };
        assert!(molecular_weight(&fixture.elements, &[item, item]).is_err());
        for count in [-1.0, f64::INFINITY, f64::NAN] {
            assert!(
                molecular_weight(&fixture.elements, &[ElementCount { count, ..item }]).is_err()
            );
        }
        assert!(
            molecular_weight(
                &fixture.elements,
                &[ElementCount {
                    element: ElementId::from_id(sid(99)),
                    ..item
                }]
            )
            .is_err()
        );
        let mut huge = element;
        huge.atomic_mass = f64::MAX;
        let table = ElementTable::new([huge]).unwrap();
        assert!(molecular_weight(&table, &[ElementCount { count: 2.0, ..item }]).is_err());
    }

    #[test]
    fn balanced_and_unbalanced_reactions_both_return_inspectable_element_residuals() {
        let fixture = water_reaction().unwrap();
        let balanced = element_balance(
            &fixture.elements,
            fixture.reaction,
            BasisKind::Molar,
            &fixture.terms,
            &fixture.compositions,
        )
        .unwrap();
        assert_eq!(
            balanced[&fixture.element_ids[0]].to_bits(),
            (0.0_f64).to_bits()
        );
        assert_eq!(
            balanced[&fixture.element_ids[1]].to_bits(),
            (0.0_f64).to_bits()
        );
        let mut terms = fixture.terms.clone();
        terms[2].coefficient = 1.0;
        let unbalanced = element_balance(
            &fixture.elements,
            fixture.reaction,
            BasisKind::Molar,
            &terms,
            &fixture.compositions,
        )
        .unwrap();
        assert_eq!(
            unbalanced[&fixture.element_ids[0]].to_bits(),
            (-2.0_f64).to_bits()
        );
        assert_eq!(
            unbalanced[&fixture.element_ids[1]].to_bits(),
            (-1.0_f64).to_bits()
        );
        terms.reverse();
        assert_eq!(
            element_balance(
                &fixture.elements,
                fixture.reaction,
                BasisKind::Molar,
                &terms,
                &fixture.compositions
            )
            .unwrap(),
            unbalanced
        );
        // The same species in distinct phases contributes twice; it is not a duplicate key.
        let mut transfer = vec![fixture.terms[2]; 2];
        transfer[0].coefficient = -1.0;
        transfer[1].coefficient = 1.0;
        transfer[1].phase = PhaseId::from_id(sid(31));
        let result = element_balance(
            &fixture.elements,
            fixture.reaction,
            BasisKind::Molar,
            &transfer,
            &fixture.compositions,
        )
        .unwrap();
        assert!(result.values().all(|value| *value == 0.0));
    }

    #[test]
    fn missing_composition_invalid_coefficients_and_deferred_basis_conversion_refuse() {
        let fixture = water_reaction().unwrap();
        let check = |terms: &[_], compositions: &_| {
            element_balance(
                &fixture.elements,
                fixture.reaction,
                BasisKind::Molar,
                terms,
                compositions,
            )
        };
        let mut missing = fixture.compositions.clone();
        missing.remove(&fixture.species[1]);
        assert!(check(&fixture.terms, &missing).is_err());
        missing.insert(fixture.species[1], vec![]);
        assert!(check(&fixture.terms, &missing).is_err());
        assert!(check(&[fixture.terms[0], fixture.terms[0]], &fixture.compositions).is_err());
        for coefficient in [0.0, f64::INFINITY, f64::NAN] {
            let mut term = fixture.terms[0];
            term.coefficient = coefficient;
            assert!(check(&[term], &fixture.compositions).is_err());
        }
        assert!(
            element_balance(
                &fixture.elements,
                fixture.reaction,
                BasisKind::Mass,
                &fixture.terms,
                &fixture.compositions
            )
            .is_err()
        );
        let mut term = fixture.terms[0];
        term.coefficient = f64::MAX;
        assert!(check(&[term], &fixture.compositions).is_err());
    }
}
