// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Composition of independently frozen thermal and consuming energy requirements.
use crate::MathError;
use pse_ids::SemanticId;
use pse_model::numerics::{CaloricRequirement, NumericalPolicy};
use pse_quantity::{
    IndexSet, PhysicalPreconditions, QuantityRegistry, QuantityTypeId, ResolvedPhysicalContract,
};

/// Admit complete physical contracts before applying the SI composition. Values are
/// frozen source magnitudes; point offsets never participate in an error difference.
pub(super) fn admitted_caloric_allowance(
    registry: &QuantityRegistry,
    preconditions: &PhysicalPreconditions,
    target: &super::TargetSpec,
    source: &CaloricRequirement,
    policy: &NumericalPolicy,
    resolved_consumers: &std::collections::BTreeMap<
        SemanticId,
        pse_model::numerics::ResolvedTarget,
    >,
) -> Result<
    (
        CaloricAllowance,
        std::collections::BTreeMap<SemanticId, (f64, f64)>,
    ),
    MathError,
> {
    use pse_quantity::infer::OpRequest;
    let target_quantity = super::engineering_error_quantity(registry, preconditions, target)?;
    pse_quantity::admission::require_same_contract(
        source.quantity_id.into(),
        target_quantity,
        registry,
    )?;
    let contract =
        |id: SemanticId| ResolvedPhysicalContract::named(id.into(), IndexSet::default(), registry);
    let base = |quantity: QuantityTypeId, unit: SemanticId, value: f64| -> Result<f64, MathError> {
        let physical = registry.quantity_type(quantity)?;
        let unit = registry.unit(unit.into())?;
        let canonical = registry.unit(physical.canonical_unit)?;
        pse_quantity::convert_spec_for_type(unit, canonical, &physical.key)?;
        Ok(value * unit.scale_to_canonical)
    };
    let thermal = match (
        source.response_quantity_id,
        source.response_unit_id,
        source.response,
        source.thermal_quantity_id,
        source.thermal_unit_id,
        source.thermal_resolution,
    ) {
        (
            Some(response_id),
            Some(response_unit),
            Some(response),
            Some(thermal_id),
            Some(thermal_unit),
            Some(resolution),
        ) => {
            let product = pse_quantity::resolved::infer_operation(
                &OpRequest::Mul,
                &[contract(response_id)?, contract(thermal_id)?],
                Some(target_quantity),
                registry,
                preconditions,
            )?
            .result
            .require_named()?;
            pse_quantity::admission::require_same_contract(product, target_quantity, registry)?;
            Some(CaloricResolution {
                target: target.id,
                response_j_mol_k: base(response_id.into(), response_unit, response)?,
                temperature_resolution_k: base(thermal_id.into(), thermal_unit, resolution)?,
            })
        }
        (None, None, None, None, None, None) => None,
        _ => return Err(super::failure(target.id, "partial caloric thermal context")),
    };
    let mut consumers = Vec::with_capacity(source.consumers.len());
    let mut combined = std::collections::BTreeMap::new();
    for consumer in &source.consumers {
        // A stream power is a datum-bearing point. Its error follows from the
        // difference of the admitted flow-times-enthalpy point operation; an
        // unrelated energy-transfer-rate contract cannot substitute for it.
        let target_type = registry.quantity_type(target.quantity)?;
        let point = if target_type.key.scale_kind == pse_quantity::ScaleKind::Point {
            Some(target.quantity)
        } else {
            let mut key = target_type.key.clone();
            key.scale_kind = pse_quantity::ScaleKind::Point;
            let mut matches = registry
                .quantity_types()
                .filter(|quantity| quantity.key == key);
            let point = matches.next().map(|quantity| quantity.id);
            if matches.next().is_some() {
                return Err(super::failure(
                    target.id,
                    "ambiguous point contract for caloric difference",
                ));
            }
            point
        };
        let inferred = if let Some(point) = point {
            let power = pse_quantity::resolved::infer_operation(
                &OpRequest::Mul,
                &[
                    contract(consumer.exposure_quantity_id)?,
                    contract(point.as_id())?,
                ],
                None,
                registry,
                preconditions,
            )?
            .result;
            pse_quantity::resolved::infer_operation(
                &OpRequest::Sub,
                &[power.clone(), power],
                Some(consumer.power_quantity_id.into()),
                registry,
                preconditions,
            )?
            .result
            .require_named()?
        } else {
            pse_quantity::resolved::infer_operation(
                &OpRequest::Mul,
                &[
                    contract(consumer.exposure_quantity_id)?,
                    contract(target_quantity.as_id())?,
                ],
                Some(consumer.power_quantity_id.into()),
                registry,
                preconditions,
            )?
            .result
            .require_named()?
        };
        pse_quantity::admission::require_same_contract(
            inferred,
            consumer.power_quantity_id.into(),
            registry,
        )?;
        let mut contributions = std::collections::BTreeMap::new();
        for other in &policy.caloric_requirements {
            for item in other
                .consumers
                .iter()
                .filter(|item| item.target_id == consumer.target_id)
            {
                pse_quantity::admission::require_same_contract(
                    other.quantity_id.into(),
                    source.quantity_id.into(),
                    registry,
                )?;
                pse_quantity::admission::require_same_contract(
                    item.power_quantity_id.into(),
                    consumer.power_quantity_id.into(),
                    registry,
                )?;
                pse_quantity::admission::require_same_contract(
                    item.exposure_quantity_id.into(),
                    consumer.exposure_quantity_id.into(),
                    registry,
                )?;
                let power = base(
                    item.power_quantity_id.into(),
                    item.power_unit_id,
                    item.power_allowance,
                )?;
                let exposure = base(
                    item.exposure_quantity_id.into(),
                    item.exposure_unit_id,
                    item.aggregate_exposure,
                )?;
                let key = (item.declaration_id, item.actual_flow_id);
                if let Some((old_power, old_exposure, old_revision, old_rule, old_records)) =
                    contributions.insert(
                        key,
                        (
                            power,
                            exposure,
                            &other.science_revision,
                            item.power_rule_id,
                            &other.science_records,
                        ),
                    )
                {
                    if f64::to_bits(old_power) != power.to_bits()
                        || f64::to_bits(old_exposure) != exposure.to_bits()
                        || old_revision != &other.science_revision
                        || old_rule != item.power_rule_id
                        || old_records
                            .iter()
                            .collect::<std::collections::BTreeSet<_>>()
                            != other
                                .science_records
                                .iter()
                                .collect::<std::collections::BTreeSet<_>>()
                    {
                        return Err(super::failure(
                            target.id,
                            "conflicting copies of an energy-exposure contribution",
                        ));
                    }
                }
            }
        }
        let mut power = f64::INFINITY;
        let mut exposure = 0.0;
        for (power_value, exposure_value, _, _, _) in contributions.into_values() {
            power = power.min(power_value);
            exposure += exposure_value;
        }
        if let Some(resolved) = resolved_consumers.get(&consumer.target_id) {
            let resolved_quantity = super::engineering_error_quantity(
                registry,
                preconditions,
                &super::TargetSpec {
                    id: resolved.id,
                    kind: resolved.kind,
                    quantity: resolved.quantity.into(),
                    unit: resolved.unit.into(),
                    integer: false,
                    declared_tolerance: None,
                },
            )?;
            pse_quantity::admission::require_same_contract(
                resolved_quantity,
                consumer.power_quantity_id.into(),
                registry,
            )?;
            power = base(resolved_quantity, resolved.unit, resolved.budget)?;
        }
        if !power.is_finite() || !exposure.is_finite() {
            return Err(super::failure(
                target.id,
                "combined consumer exposure is not representable",
            ));
        }
        combined.insert(consumer.target_id, (power, exposure));
        consumers.push(CaloricConsumer {
            id: consumer.target_id,
            power_allowance_w: power,
            aggregate_exposure_mol_s: exposure,
        });
    }
    let mut result = resolve_consumption_allowance(target.id, thermal, &consumers)?;
    // The helper returns coherent base units; resolve in the target representation.
    result.joules_per_mole /= registry.unit(target.unit)?.scale_to_canonical;
    if !result.joules_per_mole.is_finite() || result.joules_per_mole <= 0.0 {
        return Err(super::failure(
            target.id,
            "caloric allowance conversion is not representable",
        ));
    }
    Ok((result, combined))
}

/// One consuming energy requirement, expressed after ordinary physical admission.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaloricConsumer {
    /// Actual consuming requirement, retained as the limiting reason when selected.
    pub id: SemanticId,
    /// Consuming physical power allowance in watts.
    pub power_allowance_w: f64,
    /// Sum of absolute molar exposures of all affected contributions, in mol/s.
    /// This is an independently admitted frozen exposure, not the current iterate.
    pub aggregate_exposure_mol_s: f64,
}

/// Admitted caloric response and useful thermal resolution in explicit SI coordinates.
/// Callers own scientific source, phase, composition and validity qualification. A local
/// caloric response is not a coupled flash sensitivity or an output-error certificate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaloricResolution {
    /// Actual target whose physical allowance is being resolved.
    pub target: SemanticId,
    /// Independently admitted positive caloric response in J/(mol K).
    pub response_j_mol_k: f64,
    /// Selected useful temperature-difference resolution in kelvin.
    pub temperature_resolution_k: f64,
}

/// Tightest composed physical molar-enthalpy allowance, independent of its affine datum.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaloricAllowance {
    /// Physical molar-enthalpy difference allowance in J/mol.
    pub joules_per_mole: f64,
    /// Limiting consuming requirement; None denotes the thermal requirement.
    pub limiting_consumer: Option<SemanticId>,
}

/// Resolve the conjunction of thermal and all selected consuming energy requirements.
/// Zero exposure contributes no restriction. Aggregated exposure prevents assigning the
/// entire power allowance independently to several contributions of the same consumer.
/// SI conversion and full physical quantity checking belong to ordinary admission before
/// this operation; no conditioning nominal or absolute enthalpy datum is consulted.
///
/// # Errors
/// Refuses invalid/nonrepresentable thermal context, power allowance or flow exposure.
pub fn resolve_caloric_allowance(
    thermal: CaloricResolution,
    consumers: &[CaloricConsumer],
) -> Result<CaloricAllowance, MathError> {
    resolve_consumption_allowance(thermal.target, Some(thermal), consumers)
}

fn resolve_consumption_allowance(
    target: SemanticId,
    thermal: Option<CaloricResolution>,
    consumers: &[CaloricConsumer],
) -> Result<CaloricAllowance, MathError> {
    let mut result = if let Some(thermal) = thermal {
        if !thermal.response_j_mol_k.is_finite()
            || thermal.response_j_mol_k <= 0.0
            || !thermal.temperature_resolution_k.is_finite()
            || thermal.temperature_resolution_k <= 0.0
        {
            return Err(super::failure(
                thermal.target,
                "invalid frozen caloric thermal context",
            ));
        }
        let result = CaloricAllowance {
            joules_per_mole: thermal.response_j_mol_k * thermal.temperature_resolution_k,
            limiting_consumer: None,
        };
        if !result.joules_per_mole.is_finite() || result.joules_per_mole <= 0.0 {
            return Err(super::failure(
                thermal.target,
                "caloric thermal allowance is not representable",
            ));
        }
        Some(result)
    } else {
        None
    };
    let mut seen = std::collections::BTreeSet::new();
    for consumer in consumers {
        if !seen.insert(consumer.id)
            || !consumer.power_allowance_w.is_finite()
            || consumer.power_allowance_w <= 0.0
            || !consumer.aggregate_exposure_mol_s.is_finite()
            || consumer.aggregate_exposure_mol_s < 0.0
        {
            return Err(super::failure(
                target,
                "invalid or duplicate caloric energy consumer",
            ));
        }
        if consumer.aggregate_exposure_mol_s == 0.0 {
            continue;
        }
        let allowance = consumer.power_allowance_w / consumer.aggregate_exposure_mol_s;
        if !allowance.is_finite() || allowance <= 0.0 {
            return Err(super::failure(
                target,
                "caloric energy allowance is not representable",
            ));
        }
        if result.as_ref().is_none_or(|current| {
            allowance < current.joules_per_mole
                || (allowance == current.joules_per_mole
                    && current.limiting_consumer.is_some_and(|id| consumer.id < id))
        }) {
            result = Some(CaloricAllowance {
                joules_per_mole: allowance,
                limiting_consumer: Some(consumer.id),
            });
        }
    }
    result.ok_or_else(|| super::failure(target, "energy-only context has no positive exposure"))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn thermal() -> CaloricResolution {
        CaloricResolution {
            target: SemanticId::from_bytes([1; 16]),
            response_j_mol_k: 100.0,
            temperature_resolution_k: 0.1,
        }
    }
    #[test]
    fn thermal_and_energy_requirements_are_conjunctive_across_flow_scales() {
        assert_eq!(
            resolve_caloric_allowance(thermal(), &[])
                .unwrap()
                .joules_per_mole,
            10.0
        );
        for (flow, expected) in [(0.5, 2.0), (1.0, 1.0), (50.0, 0.02), (500.0, 0.002)] {
            let consumer = CaloricConsumer {
                id: SemanticId::from_bytes([2; 16]),
                power_allowance_w: 1.0,
                aggregate_exposure_mol_s: flow,
            };
            let actual = resolve_caloric_allowance(thermal(), &[consumer]).unwrap();
            assert_eq!(actual.joules_per_mole, expected);
            assert_eq!(actual.limiting_consumer, Some(consumer.id));
            assert!(flow * actual.joules_per_mole <= 1.0);
        }
    }
    #[test]
    fn all_consumers_and_aggregate_exposure_constrain_the_same_allowance() {
        let consumers = [
            CaloricConsumer {
                id: SemanticId::from_bytes([2; 16]),
                power_allowance_w: 5.0,
                aggregate_exposure_mol_s: 10.0,
            },
            CaloricConsumer {
                id: SemanticId::from_bytes([3; 16]),
                power_allowance_w: 1.0,
                aggregate_exposure_mol_s: 4.0,
            },
        ];
        let result = resolve_caloric_allowance(thermal(), &consumers).unwrap();
        assert_eq!(result.joules_per_mole, 0.25);
        assert_eq!(result.limiting_consumer, Some(consumers[1].id));
        assert_eq!(
            resolve_caloric_allowance(thermal(), &consumers.into_iter().rev().collect::<Vec<_>>())
                .unwrap(),
            result
        );
    }
    #[test]
    fn energy_only_requires_positive_exposure_and_never_invents_thermal_response() {
        let consumer = CaloricConsumer {
            id: SemanticId::from_bytes([2; 16]),
            power_allowance_w: 3.0,
            aggregate_exposure_mol_s: 6.0,
        };
        let result = resolve_consumption_allowance(thermal().target, None, &[consumer]).unwrap();
        assert_eq!(result.joules_per_mole, 0.5);
        assert_eq!(result.limiting_consumer, Some(consumer.id));
        assert!(resolve_consumption_allowance(thermal().target, None, &[]).is_err());
        assert!(
            resolve_consumption_allowance(
                thermal().target,
                None,
                &[CaloricConsumer {
                    aggregate_exposure_mol_s: 0.0,
                    ..consumer
                }]
            )
            .is_err()
        );
        assert!(
            resolve_consumption_allowance(
                thermal().target,
                None,
                &[CaloricConsumer {
                    power_allowance_w: f64::NAN,
                    ..consumer
                }]
            )
            .is_err()
        );
    }
    #[test]
    fn equal_consuming_limits_have_an_order_independent_reason() {
        let a = CaloricConsumer {
            id: SemanticId::from_bytes([2; 16]),
            power_allowance_w: 1.0,
            aggregate_exposure_mol_s: 4.0,
        };
        let b = CaloricConsumer {
            id: SemanticId::from_bytes([3; 16]),
            ..a
        };
        let forward = resolve_caloric_allowance(thermal(), &[a, b]).unwrap();
        let reverse = resolve_caloric_allowance(thermal(), &[b, a]).unwrap();
        assert_eq!(forward, reverse);
        assert_eq!(forward.limiting_consumer, Some(a.id));
    }
    #[test]
    fn frozen_response_uses_enthalpy_differences_not_absolute_datum() {
        let consumer = CaloricConsumer {
            id: SemanticId::from_bytes([2; 16]),
            power_allowance_w: 100.0,
            aggregate_exposure_mol_s: 1.0,
        };
        let mut outputs = Vec::new();
        for datum in [-1_000_000.0, 0.0, 1_000_000.0] {
            let response = ((datum + 1100.0) - (datum + 1000.0)) / 1.0;
            outputs.push(
                resolve_caloric_allowance(
                    CaloricResolution {
                        response_j_mol_k: response,
                        ..thermal()
                    },
                    &[consumer],
                )
                .unwrap(),
            );
        }
        assert_eq!(outputs[0], outputs[1]);
        assert_eq!(outputs[1], outputs[2]);
    }
    #[test]
    fn invalid_and_duplicate_context_refuses_instead_of_selecting_a_fallback() {
        for response in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(
                resolve_caloric_allowance(
                    CaloricResolution {
                        response_j_mol_k: response,
                        ..thermal()
                    },
                    &[]
                )
                .is_err()
            );
        }
        let consumer = CaloricConsumer {
            id: SemanticId::from_bytes([2; 16]),
            power_allowance_w: 1.0,
            aggregate_exposure_mol_s: 0.0,
        };
        assert_eq!(
            resolve_caloric_allowance(thermal(), &[consumer])
                .unwrap()
                .joules_per_mole,
            10.0
        );
        assert!(resolve_caloric_allowance(thermal(), &[consumer, consumer]).is_err());
        assert!(
            resolve_caloric_allowance(
                thermal(),
                &[CaloricConsumer {
                    aggregate_exposure_mol_s: -1.0,
                    ..consumer
                }]
            )
            .is_err()
        );
    }
}
