// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Resolve physical magnitudes and source precedence before numerical execution.
use crate::MathError;
use pse_ids::{FramedHasher, SemanticId};
use pse_model::{
    SemanticFrame,
    generated::enums::{
        NumericalCoordinates, NumericalProvenanceField, NumericalSource, NumericalTarget,
    },
    numerics::*,
};
use pse_quantity::{PhysicalPreconditions, QuantityRegistry, QuantityTypeId, UnitId};
use std::collections::BTreeMap;
mod caloric;
mod retained;
pub use caloric::{
    CaloricAllowance, CaloricConsumer, CaloricResolution, resolve_caloric_allowance,
};
pub use retained::{ResolutionPremises, resolution_allocation_bound, resolved_extent};

/// Resolved kind/identity access shared by coordinate transport and original quality.
/// The source policy owns frozen scales, allowances and provenance throughout access.
#[derive(Debug)]
pub struct TargetAccess<'a> {
    targets: crate::index::CheckedInventory<'a, (NumericalTarget, SemanticId), ResolvedTarget>,
}
impl<'a> TargetAccess<'a> {
    /// Refuse ambiguous resolved targets rather than selecting the first duplicate.
    pub fn new(policy: &'a ResolvedNumericalPolicy) -> Result<Self, MathError> {
        Ok(Self {
            targets: crate::index::CheckedInventory::new(&policy.targets, |t| (t.kind, t.id))?,
        })
    }
    /// Resolve one target without repeating the source-vector search.
    pub fn get(
        &self,
        kind: NumericalTarget,
        id: SemanticId,
    ) -> Result<&'a ResolvedTarget, MathError> {
        self.find(kind, id)
            .ok_or_else(|| failure(id, "target was not resolved"))
    }
    /// Optional source access for owners whose explicit missing-target rule selects defaults.
    pub fn find(&self, kind: NumericalTarget, id: SemanticId) -> Option<&'a ResolvedTarget> {
        self.targets.get(&(kind, id))
    }
    /// Resolve ordered coordinates; duplicate consumer requests retain their multiplicity.
    pub fn ordered(
        &self,
        kind: NumericalTarget,
        ids: &[SemanticId],
    ) -> Result<Vec<&'a ResolvedTarget>, MathError> {
        ids.iter().map(|id| self.get(kind, *id)).collect()
    }
}

/// Exact admitted coordinate; declared representation is retained through normalization.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TargetSpec {
    /// Source identity, NIL only for the single objective.
    pub id: SemanticId,
    /// Variable, row, objective, observable or closure.
    pub kind: NumericalTarget,
    /// Complete physical type.
    pub quantity: QuantityTypeId,
    /// Representation of values crossing the original oracle.
    pub unit: UnitId,
    /// Coordinate substitution must preserve the integer lattice.
    pub integer: bool,
    /// Explicit physical acceptance budget already owned by a balance/output declaration.
    pub declared_tolerance: Option<f64>,
}
/// Selection supplies precedence; declarations cannot promote themselves to analysis overrides.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourcedRequirement {
    /// Selected source category.
    pub source: NumericalSource,
    /// Registry-owned declaration.
    pub declaration: NumericalRequirement,
}
/// An explicit coordinate projection preserves the already resolved source policy.
#[derive(Clone, Debug)]
pub struct TargetProjection {
    /// Original selected coordinate.
    pub source: SemanticId,
    /// Original coordinate role.
    pub source_kind: NumericalTarget,
    /// Representation required by the consumer.
    pub target: TargetSpec,
}
/// Project resolved magnitudes without selecting defaults or applying precedence again.
/// # Errors
/// Missing or repeated targets, changed physical contracts, integer substitutions and
/// nonrepresentable magnitudes are refused. Affine offsets never enter magnitudes.
pub fn project(
    registry: &QuantityRegistry,
    source: &ResolvedNumericalPolicy,
    projections: &[TargetProjection],
) -> Result<ResolvedNumericalPolicy, MathError> {
    project_with(registry, source, projections, ProjectionMeaning::Coordinate)
}
/// Preserve resolved magnitude policy on the difference of two source observations.
/// The physical owner must admit subtraction to the requested target contract;
/// affine origins never enter nominal, tolerance or scaling magnitudes.
/// # Errors
/// Refuses a target other than the admitted subtraction result, malformed unit
/// representations and all invalid projections also refused by [`project`].
pub fn project_difference(
    registry: &QuantityRegistry,
    preconditions: &PhysicalPreconditions,
    source: &ResolvedNumericalPolicy,
    projections: &[TargetProjection],
) -> Result<ResolvedNumericalPolicy, MathError> {
    project_with(
        registry,
        source,
        projections,
        ProjectionMeaning::Difference(preconditions),
    )
}
#[derive(Clone, Copy)]
enum ProjectionMeaning<'a> {
    Coordinate,
    Difference(&'a PhysicalPreconditions),
}
fn project_with(
    registry: &QuantityRegistry,
    source: &ResolvedNumericalPolicy,
    projections: &[TargetProjection],
    meaning: ProjectionMeaning<'_>,
) -> Result<ResolvedNumericalPolicy, MathError> {
    let frame = match meaning {
        ProjectionMeaning::Coordinate => pse_ids::Frame::NumericalProjectionV2,
        ProjectionMeaning::Difference(_) => pse_ids::Frame::NumericalDifferenceProjectionV1,
    };
    let mut key = FramedHasher::new(frame);
    key.hash(&source.key);
    let access = TargetAccess::new(source)?;
    let mut ordered = projections.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|p| (p.target.id, p.target.kind.as_str()));
    let mut seen = std::collections::BTreeSet::new();
    let mut targets = Vec::with_capacity(ordered.len());
    for projection in ordered {
        let target = &projection.target;
        if !seen.insert((target.id, target.kind.as_str()))
            || target.integer
            || target.declared_tolerance.is_some()
        {
            return Err(failure(
                target.id,
                "projection must name unique continuous coordinates without new tolerances",
            ));
        }
        let original = access.get(projection.source_kind, projection.source)?;
        let projected_quantity = match meaning {
            ProjectionMeaning::Coordinate => original.quantity.into(),
            ProjectionMeaning::Difference(preconditions) => {
                let value = pse_quantity::ResolvedPhysicalContract::named(
                    original.quantity.into(),
                    pse_quantity::IndexSet::default(),
                    registry,
                )?;
                pse_quantity::resolved::infer_operation(
                    &pse_quantity::infer::OpRequest::Sub,
                    &[value.clone(), value],
                    None,
                    registry,
                    preconditions,
                )?
                .result
                .require_named()?
            }
        };
        pse_quantity::admission::require_same_contract(
            projected_quantity,
            target.quantity,
            registry,
        )?;
        let scale = pse_quantity::convert_spec_for_type(
            registry.unit(original.unit.into())?,
            registry.unit(target.unit)?,
            &registry.quantity_type(target.quantity)?.key,
        )?
        .scale
        .abs();
        let mut result = original.clone();
        result.id = target.id;
        result.kind = target.kind;
        result.quantity = target.quantity.as_id();
        result.unit = target.unit.as_id();
        result.nominal *= scale;
        result.coordinate_scale *= scale;
        result.absolute *= scale;
        result.budget *= scale;
        if let Some(engineering) = &mut result.engineering {
            engineering.physical_allowance = engineering.physical_allowance.map(|v| v * scale);
            engineering.characteristic = engineering.characteristic.map(|v| v * scale);
            engineering.budget *= scale;
            if let Some(caloric) = engineering.caloric.as_mut() {
                // Source inputs retain their own recorded units; only the derived
                // allowance is expressed in the projected target representation.
                caloric.allowance *= scale;
            }
            if !engineering.budget.is_finite()
                || engineering.budget <= 0.0
                || engineering
                    .physical_allowance
                    .is_some_and(|v| !v.is_finite() || v <= 0.0)
                || engineering
                    .characteristic
                    .is_some_and(|v| !v.is_finite() || v < 0.0)
                || engineering
                    .caloric
                    .as_ref()
                    .is_some_and(|v| !v.allowance.is_finite() || v.allowance <= 0.0)
            {
                return Err(failure(
                    target.id,
                    "projected engineering context is not representable",
                ));
            }
        }
        if [result.nominal, result.coordinate_scale, result.budget]
            .iter()
            .any(|v| !v.is_finite() || *v <= 0.0)
            || !result.absolute.is_finite()
        {
            return Err(failure(
                target.id,
                "projected magnitude is not representable",
            ));
        }
        for p in &mut result.provenance {
            if p.field != NumericalProvenanceField::RelativeTolerance {
                p.value *= scale;
            }
        }
        key.id(&projection.source)
            .id(&target.id)
            .id(&target.quantity.as_id())
            .id(&target.unit.as_id())
            .u64(scale.to_bits());
        projection.source_kind.frame(&mut key);
        target.kind.frame(&mut key);
        targets.push(result);
    }
    Ok(ResolvedNumericalPolicy {
        policy: source.policy.clone(),
        targets,
        key: key.finish_hash(),
    })
}
fn rank(source: NumericalSource) -> u8 {
    match source {
        NumericalSource::Analysis => 7,
        NumericalSource::Case => 6,
        NumericalSource::Model => 5,
        NumericalSource::ModelHint => 4,
        NumericalSource::DerivedNominal => 2,
        NumericalSource::PropertyDefault => 3,
        NumericalSource::QuantityNominal => 1,
        NumericalSource::CanonicalFallback => 0,
    }
}
fn failure(id: SemanticId, message: &str) -> MathError {
    MathError::Contract(format!("numerical target {id}: {message}"))
}

fn engineering_rank(source: NumericalSource) -> Option<u8> {
    match source {
        NumericalSource::Analysis => Some(6),
        NumericalSource::Case => Some(5),
        NumericalSource::Model => Some(4),
        NumericalSource::PropertyDefault => Some(3),
        NumericalSource::ModelHint => Some(2),
        NumericalSource::QuantityNominal => Some(1),
        NumericalSource::DerivedNominal | NumericalSource::CanonicalFallback => None,
    }
}

/// Convert error magnitudes in the target's legal difference context; affine datum
/// offsets never become engineering characteristic magnitudes.
fn engineering_magnitude(
    registry: &QuantityRegistry,
    preconditions: &PhysicalPreconditions,
    target: &TargetSpec,
    unit: UnitId,
) -> Result<f64, MathError> {
    let difference = engineering_error_quantity(registry, preconditions, target)?;
    Ok(pse_quantity::convert_spec_for_type(
        registry.unit(unit)?,
        registry.unit(target.unit)?,
        &registry.quantity_type(difference)?.key,
    )?
    .scale
    .abs())
}

/// Error allowances use the complete legal difference contract of affine points.
fn engineering_error_quantity(
    registry: &QuantityRegistry,
    preconditions: &PhysicalPreconditions,
    target: &TargetSpec,
) -> Result<QuantityTypeId, MathError> {
    Ok(
        if registry.quantity_type(target.quantity)?.key.scale_kind == pse_quantity::ScaleKind::Point
        {
            let physical = pse_quantity::ResolvedPhysicalContract::named(
                target.quantity,
                pse_quantity::IndexSet::default(),
                registry,
            )?;
            pse_quantity::resolved::infer_operation(
                &pse_quantity::infer::OpRequest::Sub,
                &[physical.clone(), physical],
                None,
                registry,
                preconditions,
            )?
            .result
            .require_named()?
        } else {
            target.quantity
        },
    )
}

/// Resolve an operational reconstructed-output allowance independently of bound
/// feasibility. A caller freezes this context in the consumed producer identity.
/// # Errors
/// Missing strict context, ambiguous full-quantity rules or invalid tagged scale.
pub fn operational_output_context(
    registry: &QuantityRegistry,
    preconditions: &PhysicalPreconditions,
    target: &TargetSpec,
    policy: &NumericalPolicy,
    resolved_targets: &[ResolvedTarget],
) -> Result<EngineeringContext, MathError> {
    policy
        .validate()
        .map_err(|error| MathError::Contract(error.to_string()))?;
    let selected = policy
        .requirements
        .iter()
        .find(|r| {
            r.target_id == target.id
                && r.target_kind == target.kind
                && r.shared_engineering_allowance == Some(true)
        })
        .map(|declaration| SourcedRequirement {
            declaration: declaration.clone(),
            source: NumericalSource::Analysis,
        });
    let consumers = policy
        .caloric_requirements
        .iter()
        .filter(|requirement| {
            requirement.target_id == target.id && requirement.target_kind == target.kind
        })
        .flat_map(|requirement| {
            requirement
                .consumers
                .iter()
                .map(|consumer| consumer.target_id)
        })
        .collect::<std::collections::BTreeSet<_>>();
    let mut resolved_consumers = BTreeMap::new();
    for resolved in resolved_targets
        .iter()
        .filter(|resolved| consumers.contains(&resolved.id))
    {
        if resolved_consumers
            .insert(resolved.id, resolved.clone())
            .is_some()
        {
            return Err(failure(
                resolved.id,
                "caloric consumer has ambiguous resolved targets",
            ));
        }
    }
    engineering_default(
        registry,
        preconditions,
        target,
        policy,
        selected.as_ref(),
        true,
        &resolved_consumers,
    )
}

fn engineering_default(
    registry: &QuantityRegistry,
    preconditions: &PhysicalPreconditions,
    target: &TargetSpec,
    policy: &NumericalPolicy,
    selected: Option<&SourcedRequirement>,
    enforce_strict: bool,
    resolved_consumers: &BTreeMap<SemanticId, ResolvedTarget>,
) -> Result<EngineeringContext, MathError> {
    let explicit_rule = selected.and_then(|r| r.declaration.engineering_rule_id);
    let rule = if let Some(id) = explicit_rule {
        let rule = policy
            .engineering_rules
            .iter()
            .find(|r| r.rule_id == id)
            .ok_or_else(|| failure(target.id, "engineering rule reference is unavailable"))?;
        pse_quantity::admission::require_same_contract(
            rule.quantity_id.into(),
            engineering_error_quantity(registry, preconditions, target)?,
            registry,
        )?;
        Some(rule)
    } else {
        let applicable = policy
            .engineering_rules
            .iter()
            .filter(|r| {
                engineering_error_quantity(registry, preconditions, target).is_ok_and(|quantity| {
                    pse_quantity::admission::require_same_contract(
                        r.quantity_id.into(),
                        quantity,
                        registry,
                    )
                    .is_ok()
                })
            })
            .collect::<Vec<_>>();
        if applicable.len() > 1 {
            return Err(failure(
                target.id,
                "conflicting applicable engineering default rules require explicit selection",
            ));
        }
        applicable.first().copied()
    };
    let floor = rule
        .and_then(|r| r.physical_allowance)
        .map(|v| {
            Ok::<f64, MathError>(
                v * engineering_magnitude(
                    registry,
                    preconditions,
                    target,
                    rule.map_or(target.unit, |r| r.unit_id.into()),
                )?,
            )
        })
        .transpose()?;
    let fraction = rule
        .and_then(|r| r.relative_fraction)
        .unwrap_or(policy.engineering_relative_fraction);
    let mut scales = policy
        .engineering_scales
        .iter()
        .filter(|s| s.target_id == target.id && s.target_kind == target.kind)
        .map(|s| {
            pse_quantity::admission::require_same_contract(
                s.quantity_id.into(),
                target.quantity,
                registry,
            )?;
            let precedence = engineering_rank(s.source).ok_or_else(|| {
                failure(
                    target.id,
                    "conditioning or canonical nominal is not an engineering scale",
                )
            })?;
            let magnitude =
                s.value * engineering_magnitude(registry, preconditions, target, s.unit_id.into())?;
            if !magnitude.is_finite() || magnitude < 0.0 {
                return Err(failure(target.id, "engineering scale is not representable"));
            }
            Ok((s, magnitude, (precedence, s.priority)))
        })
        .collect::<Result<Vec<_>, MathError>>()?;
    scales.sort_by_key(|(s, _, rank)| (std::cmp::Reverse(*rank), s.scale_id));
    let chosen = scales.first();
    if let Some((scale, value, priority)) = chosen {
        for (other, other_value, other_priority) in &scales {
            if priority == other_priority
                && (pse_ids::canonical_f64_bits(*value)
                    != pse_ids::canonical_f64_bits(*other_value)
                    || scale.kind != other.kind)
            {
                return Err(failure(
                    target.id,
                    "conflicting engineering scales at equal precedence",
                ));
            }
        }
    }
    let characteristic = chosen.map(|(_, value, _)| *value);
    let budget = floor
        .unwrap_or(0.0)
        .max(fraction * characteristic.unwrap_or(0.0));
    let canonical_fallback =
        floor.is_none() && (characteristic.is_none_or(|s| s == 0.0) || fraction == 0.0);
    let caloric = policy
        .caloric_requirements
        .iter()
        .find(|r| r.target_id == target.id && r.target_kind == target.kind);
    let resolved_caloric = caloric.map(|source| {
        engineering_magnitude(registry, preconditions, target, source.unit_id.into())?;
        let (result, combined) = caloric::admitted_caloric_allowance(registry, preconditions, target, source, policy, resolved_consumers)?;
        use pse_model::generated::runtime::resolved_numerics::{RuntimeResolvedNumericsFieldEngineeringCaloric as Snapshot, RuntimeResolvedNumericsFieldEngineeringCaloricConsumersItem as Consumer, RuntimeResolvedNumericsFieldEngineeringCaloricConsumersItemContributorsItem as Contributor};
        let ordered_records = |records: &[SemanticId]| { let mut records = records.to_vec(); records.sort(); records };
        let mut ordered_consumers = source.consumers.iter().collect::<Vec<_>>();
        ordered_consumers.sort_by_key(|consumer| consumer.target_id);
        let mut ordered_requirements = policy.caloric_requirements.iter().collect::<Vec<_>>();
        ordered_requirements.sort_by_key(|requirement| requirement.requirement_id);
        Ok::<_, MathError>(Snapshot {
            requirement_id: source.requirement_id,
            instance_id: source.instance_id.map(Into::into),
            source: source.source,
            science_revision: source.science_revision.clone(),
            science_records: ordered_records(&source.science_records),
            response_quantity_id: source.response_quantity_id,
            response_unit_id: source.response_unit_id,
            response: source.response,
            thermal_quantity_id: source.thermal_quantity_id,
            thermal_unit_id: source.thermal_unit_id,
            thermal_resolution: source.thermal_resolution,
            consumers: ordered_consumers.into_iter().map(|c| {
                let (power, exposure) = *combined.get(&c.target_id).ok_or_else(|| failure(target.id, "combined consumer context is absent"))?;
                let mut contributors = BTreeMap::new();
                for requirement in &ordered_requirements {
                    for item in requirement.consumers.iter().filter(|item| item.target_id == c.target_id) {
                        contributors.entry((item.declaration_id, item.actual_flow_id)).or_insert_with(|| Contributor {
                            declaration_id: item.declaration_id, actual_flow_id: item.actual_flow_id,
                            science_revision: requirement.science_revision.clone(), science_records: ordered_records(&requirement.science_records), power_rule_id: item.power_rule_id,
                            exposure_quantity_id: item.exposure_quantity_id, exposure_unit_id: item.exposure_unit_id,
                            aggregate_exposure: item.aggregate_exposure, power_quantity_id: item.power_quantity_id,
                            power_unit_id: item.power_unit_id, power_allowance: item.power_allowance,
                        });
                    }
                }
                Ok::<_, MathError>(Consumer {
                target_id: c.target_id, declaration_id: c.declaration_id,
                power_rule_id: c.power_rule_id, actual_flow_id: c.actual_flow_id,
                exposure_expression: c.exposure_expression.clone(), power_quantity_id: c.power_quantity_id,
                power_unit_id: c.power_unit_id, power_allowance: c.power_allowance,
                exposure_quantity_id: c.exposure_quantity_id, exposure_unit_id: c.exposure_unit_id,
                aggregate_exposure: c.aggregate_exposure,
                combined_exposure: exposure / registry.unit(c.exposure_unit_id.into())?.scale_to_canonical,
                combined_power_allowance: power / registry.unit(c.power_unit_id.into())?.scale_to_canonical,
                contributors: contributors.into_values().collect(),
            }) }).collect::<Result<Vec<_>, MathError>>()?,
            limiting_consumer: result.limiting_consumer,
            allowance: result.joules_per_mole,
        })
    }).transpose()?;
    let budget = if canonical_fallback && resolved_caloric.is_some() {
        resolved_caloric
            .as_ref()
            .map_or(0.0, |context| context.allowance)
    } else if canonical_fallback {
        if enforce_strict && policy.strict_engineering_context {
            return Err(failure(
                target.id,
                "strict engineering context forbids canonical fallback",
            ));
        }
        let canonical = registry.quantity_type(target.quantity)?.canonical_unit;
        DEFAULT_ENGINEERING_ACCURACY
            * engineering_magnitude(registry, preconditions, target, canonical)?
    } else {
        resolved_caloric
            .as_ref()
            .map_or(budget, |context| budget.min(context.allowance))
    };
    if !budget.is_finite() || budget <= 0.0 || floor.is_some_and(|v| !v.is_finite() || v <= 0.0) {
        return Err(failure(
            target.id,
            "engineering allowance is not finite and positive",
        ));
    }
    Ok(EngineeringContext {
        rule_id: rule.map(|r| r.rule_id),
        scale_id: chosen.map(|(s, _, _)| s.scale_id),
        physical_allowance: floor,
        relative_fraction: fraction,
        characteristic,
        scale_kind: chosen.map(|(s, _, _)| s.kind),
        source: chosen.map_or_else(
            || {
                caloric.map_or_else(
                    || {
                        selected.map_or(
                            if rule.is_some() {
                                NumericalSource::PropertyDefault
                            } else {
                                NumericalSource::CanonicalFallback
                            },
                            |r| r.source,
                        )
                    },
                    |c| c.source,
                )
            },
            |(s, _, _)| s.source,
        ),
        budget,
        canonical_fallback: canonical_fallback && resolved_caloric.is_none(),
        limitation: if resolved_caloric
            .as_ref()
            .is_some_and(|context| context.response.is_some())
        {
            "Local caloric response is an estimate, not coupled output-error evidence".into()
        } else if resolved_caloric.is_some() {
            "Energy-only context supplies no thermal-response or coupled output-error evidence"
                .into()
        } else if canonical_fallback {
            "No positive admitted engineering allowance; canonical magnitude fallback is not output-error evidence".into()
        } else {
            String::new()
        },
        caloric: resolved_caloric,
    })
}

/// Derive an inverse characteristic magnitude from independently evaluated original
/// additive terms. The sanctioned scheme names retain IDAES's exact definitions,
/// including `harmonicMean` being a sum of reciprocals rather than an average.
/// # Errors
/// Nonfinite observations or an unrepresentable positive scale are explicit failures.
pub fn term_scale(
    scheme: pse_model::generated::enums::ConstraintScalingScheme,
    terms: &[f64],
) -> Result<f64, MathError> {
    use pse_model::generated::enums::ConstraintScalingScheme as S;
    if terms.is_empty() || terms.iter().any(|v| !v.is_finite()) {
        return Err(MathError::Contract(
            "derived nominal requires finite original term observations".into(),
        ));
    }
    let nonzero = terms
        .iter()
        .map(|v| v.abs())
        .filter(|v| *v > 0.0)
        .collect::<Vec<_>>();
    if nonzero.is_empty() {
        return Ok(1.0);
    }
    // Rescaling avoids spurious overflow of sums/squares. No finite observation is
    // silently replaced with 1 when the mathematical scale cannot be represented.
    let maximum = nonzero.iter().copied().fold(0.0, f64::max);
    let minimum = nonzero.iter().copied().fold(f64::INFINITY, f64::min);
    let value = match scheme {
        S::HarmonicMean => nonzero.iter().map(|v| minimum / v).sum::<f64>() / minimum,
        S::InverseSum => (1.0 / maximum) / nonzero.iter().map(|v| v / maximum).sum::<f64>(),
        S::InverseRSS => {
            (1.0 / maximum)
                / nonzero
                    .iter()
                    .map(|v| (v / maximum).powi(2))
                    .sum::<f64>()
                    .sqrt()
        }
        S::InverseMaximum => 1.0 / maximum,
        S::InverseMinimum => 1.0 / minimum,
    };
    if !value.is_finite() || value <= 0.0 {
        return Err(MathError::Contract(
            "derived nominal scale is outside representable positive values".into(),
        ));
    }
    Ok(value)
}
/// Resolve all selected targets, refusing unknown requirements and equal-priority conflicts.
pub fn resolve(
    registry: &QuantityRegistry,
    preconditions: &PhysicalPreconditions,
    targets: &[TargetSpec],
    declarations: &[SourcedRequirement],
    policy: &NumericalPolicy,
) -> Result<ResolvedNumericalPolicy, MathError> {
    policy
        .validate()
        .map_err(|e| MathError::Contract(e.to_string()))?;
    let target_access = crate::index::CheckedInventory::new(targets, |t| (t.kind, t.id))?;
    for scale in &policy.engineering_scales {
        let target = target_access
            .get(&(scale.target_kind, scale.target_id))
            .ok_or_else(|| {
                failure(
                    scale.target_id,
                    "engineering scale names an unselected coordinate",
                )
            })?;
        pse_quantity::admission::require_same_contract(
            scale.quantity_id.into(),
            target.quantity,
            registry,
        )?;
        engineering_magnitude(registry, preconditions, target, scale.unit_id.into())?;
    }
    for requirement in &policy.caloric_requirements {
        let target = target_access
            .get(&(requirement.target_kind, requirement.target_id))
            .ok_or_else(|| {
                failure(
                    requirement.target_id,
                    "caloric requirement names an unselected coordinate",
                )
            })?;
        pse_quantity::admission::require_same_contract(
            requirement.quantity_id.into(),
            engineering_error_quantity(registry, preconditions, target)?,
            registry,
        )?;
        engineering_magnitude(registry, preconditions, target, requirement.unit_id.into())?;
        caloric::admitted_caloric_allowance(
            registry,
            preconditions,
            target,
            requirement,
            policy,
            &BTreeMap::new(),
        )?;
    }
    for rule in &policy.engineering_rules {
        let quantity = registry.quantity_type(rule.quantity_id.into())?;
        pse_quantity::convert_spec_for_type(
            registry.unit(rule.unit_id.into())?,
            registry.unit(quantity.canonical_unit)?,
            &quantity.key,
        )?;
    }
    // Resolve distinct selected consumers once through the same numerical owner.
    // Standalone supplied caloric requirements can retain their explicit power
    // premise when no consumer target is part of this inventory.
    let consumer_ids = policy
        .caloric_requirements
        .iter()
        .flat_map(|requirement| {
            requirement
                .consumers
                .iter()
                .map(|consumer| consumer.target_id)
        })
        .collect::<std::collections::BTreeSet<_>>();
    let consumer_targets = targets
        .iter()
        .filter(|target| consumer_ids.contains(&target.id))
        .cloned()
        .collect::<Vec<_>>();
    let mut resolved_consumers = BTreeMap::new();
    if !consumer_targets.is_empty() {
        if consumer_targets.iter().any(|target| {
            policy.caloric_requirements.iter().any(|requirement| {
                requirement.target_id == target.id && requirement.target_kind == target.kind
            })
        }) {
            return Err(MathError::Contract(
                "caloric consumer cannot depend on its own caloric target".into(),
            ));
        }
        let selected = |id, kind| {
            consumer_targets
                .iter()
                .any(|target| target.id == id && target.kind == kind)
        };
        let mut consumer_policy = policy.clone();
        consumer_policy.caloric_requirements.clear();
        consumer_policy
            .requirements
            .retain(|row| selected(row.target_id, row.target_kind));
        consumer_policy
            .engineering_scales
            .retain(|row| selected(row.target_id, row.target_kind));
        consumer_policy.goals.clear();
        let consumer_declarations = declarations
            .iter()
            .filter(|row| selected(row.declaration.target_id, row.declaration.target_kind))
            .cloned()
            .collect::<Vec<_>>();
        let result = resolve(
            registry,
            preconditions,
            &consumer_targets,
            &consumer_declarations,
            &consumer_policy,
        )?;
        for target in result.targets {
            if resolved_consumers.insert(target.id, target).is_some() {
                return Err(MathError::Contract(
                    "caloric consumer has ambiguous target kinds".into(),
                ));
            }
        }
    }
    let mut all = declarations.to_vec();
    all.extend(
        policy
            .requirements
            .iter()
            .cloned()
            .map(|declaration| SourcedRequirement {
                source: NumericalSource::Analysis,
                declaration,
            }),
    );
    let mut identities = std::collections::BTreeSet::new();
    let mut requirements: BTreeMap<_, Vec<&SourcedRequirement>> = BTreeMap::new();
    for candidate in &all {
        let row = &candidate.declaration;
        if !identities.insert(row.requirement_id) {
            return Err(failure(
                row.target_id,
                "duplicate numerical requirement identity",
            ));
        }
        if target_access
            .get(&(row.target_kind, row.target_id))
            .is_none()
        {
            return Err(failure(
                row.target_id,
                "requirement names an unselected coordinate",
            ));
        }
        if row.provenance.trim().is_empty()
            || row
                .nominal
                .into_iter()
                .chain(row.scaling_factor)
                .any(|v| !v.is_finite() || v <= 0.0)
            || row
                .absolute_tolerance
                .into_iter()
                .chain(row.relative_tolerance)
                .any(|v| !v.is_finite() || v < 0.0)
            || row.coordinates == NumericalCoordinates::Normalized && row.unit_id.is_some()
        {
            return Err(failure(
                row.target_id,
                "invalid magnitude, coordinate unit or provenance",
            ));
        }
        if row.engineering_rule_id.is_some() && row.shared_engineering_allowance != Some(true) {
            return Err(failure(
                row.target_id,
                "engineering rule reference requires inherited engineering allowance",
            ));
        }
        if row.shared_engineering_allowance == Some(true)
            && row.coordinates != NumericalCoordinates::Physical
        {
            return Err(failure(
                row.target_id,
                "inherited engineering allowance is a physical magnitude",
            ));
        }
        if let (Some(n), Some(s)) = (row.nominal, row.scaling_factor)
            && n.to_bits() != (1.0 / s).to_bits()
        {
            return Err(failure(
                row.target_id,
                "nominal and scaling factor disagree",
            ));
        }
        requirements
            .entry((row.target_kind, row.target_id))
            .or_default()
            .push(candidate);
    }
    for candidates in requirements.values_mut() {
        candidates.sort_by_key(|r| {
            (
                std::cmp::Reverse((rank(r.source), r.declaration.priority)),
                r.declaration.requirement_id,
            )
        });
    }
    let mut resolved = Vec::with_capacity(targets.len());
    let mut key = FramedHasher::new(if policy.caloric_requirements.is_empty() {
        pse_ids::Frame::NumericalResolvedV2
    } else {
        pse_ids::Frame::NumericalResolvedV3
    });
    key.hash(&policy.key());
    let mut identity_sources = all.iter().collect::<Vec<_>>();
    identity_sources.sort_by_key(|r| r.declaration.requirement_id);
    for source in identity_sources {
        source.source.frame(&mut key);
        source.declaration.frame(&mut key);
    }
    let mut ordered = targets.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|t| (t.id, t.kind.as_str()));
    for target in ordered {
        let quantity = registry.quantity_type(target.quantity)?;
        let magnitude = |unit: UnitId| -> Result<f64, MathError> {
            Ok(pse_quantity::convert_spec_for_type(
                registry.unit(unit)?,
                registry.unit(target.unit)?,
                &quantity.key,
            )?
            .scale
            .abs())
        };
        let canonical = magnitude(quantity.canonical_unit)?;
        let fallback = quantity.nominal_magnitude.unwrap_or(1.0) * canonical;
        let candidates = requirements
            .get(&(target.kind, target.id))
            .map_or(&[][..], Vec::as_slice);
        let mut provenance = Vec::new();
        let mut choose = |field: NumericalProvenanceField,
                          values: Vec<(&SourcedRequirement, f64)>,
                          default: f64,
                          default_source|
         -> Result<f64, MathError> {
            let Some((first, value)) = values.first() else {
                provenance.push(NumericalProvenance {
                    declaration: None,
                    source: default_source,
                    field,
                    selected: true,
                    value: default,
                    description: "recorded numerical default".into(),
                });
                return Ok(default);
            };
            let top = (rank(first.source), first.declaration.priority);
            for (candidate, other) in &values {
                let selected = (rank(candidate.source), candidate.declaration.priority) == top;
                if selected && other.to_bits() != value.to_bits() {
                    return Err(failure(
                        target.id,
                        &format!(
                            "conflicting {}: {} and {}",
                            field.as_str(),
                            first.declaration.requirement_id,
                            candidate.declaration.requirement_id
                        ),
                    ));
                }
                provenance.push(NumericalProvenance {
                    declaration: Some(candidate.declaration.requirement_id),
                    source: candidate.source,
                    field,
                    selected,
                    value: *other,
                    description: candidate.declaration.provenance.clone(),
                });
            }
            Ok(*value)
        };
        let nominals = candidates
            .iter()
            .filter_map(|r| {
                r.declaration
                    .nominal
                    .or_else(|| r.declaration.scaling_factor.map(|v| 1.0 / v))
                    .map(|v| (*r, v))
            })
            .map(|(r, v)| {
                Ok((
                    r,
                    v * magnitude(r.declaration.unit_id.map_or(target.unit, Into::into))?,
                ))
            })
            .collect::<Result<Vec<_>, MathError>>()?;
        if policy.strict_nominals && nominals.is_empty() && quantity.nominal_magnitude.is_none() {
            return Err(failure(
                target.id,
                "strict nominal completeness forbids canonical fallback",
            ));
        }
        let nominal = choose(
            NumericalProvenanceField::Nominal,
            nominals,
            fallback,
            if quantity.nominal_magnitude.is_some() {
                NumericalSource::QuantityNominal
            } else {
                NumericalSource::CanonicalFallback
            },
        )?;
        let coordinate_scale = if target.integer { 1.0 } else { nominal };
        let absolute_candidates = candidates
            .iter()
            .filter(|r| {
                r.declaration.shared_engineering_allowance == Some(true)
                    || r.declaration.absolute_tolerance.is_some()
            })
            .copied()
            .collect::<Vec<_>>();
        if let Some(first) = absolute_candidates.first() {
            for other in &absolute_candidates {
                if (rank(first.source), first.declaration.priority)
                    == (rank(other.source), other.declaration.priority)
                    && ((first.declaration.shared_engineering_allowance == Some(true))
                        != (other.declaration.shared_engineering_allowance == Some(true))
                        || first.declaration.engineering_rule_id
                            != other.declaration.engineering_rule_id)
                {
                    return Err(failure(
                        target.id,
                        "conflicting engineering allowance selections at equal precedence",
                    ));
                }
            }
        }
        let mut selected_engineering = None;
        let absolute = absolute_candidates
            .iter()
            .enumerate()
            .map(|(index, r)| {
                if r.declaration.shared_engineering_allowance == Some(true) {
                    let context = engineering_default(
                        registry,
                        preconditions,
                        target,
                        policy,
                        Some(r),
                        index == 0,
                        &resolved_consumers,
                    )?;
                    let budget = context.budget;
                    if index == 0 {
                        selected_engineering = Some(context);
                    }
                    Ok((*r, budget))
                } else {
                    let value = r
                        .declaration
                        .absolute_tolerance
                        .ok_or_else(|| failure(target.id, "missing explicit absolute tolerance"))?;
                    Ok((
                        *r,
                        value
                            * if r.declaration.coordinates == NumericalCoordinates::Normalized {
                                coordinate_scale
                            } else {
                                magnitude(r.declaration.unit_id.map_or(target.unit, Into::into))?
                            },
                    ))
                }
            })
            .collect::<Result<Vec<_>, MathError>>()?;
        let default_absolute = if absolute.is_empty() {
            if let Some(explicit) = target.declared_tolerance {
                explicit
            } else {
                let context = engineering_default(
                    registry,
                    preconditions,
                    target,
                    policy,
                    None,
                    true,
                    &resolved_consumers,
                )?;
                let budget = context.budget;
                selected_engineering = Some(context);
                budget
            }
        } else {
            0.0
        };
        let absolute = choose(
            NumericalProvenanceField::AbsoluteTolerance,
            absolute,
            default_absolute,
            if target.declared_tolerance.is_some() {
                NumericalSource::Model
            } else {
                selected_engineering
                    .as_ref()
                    .map_or(NumericalSource::CanonicalFallback, |context| context.source)
            },
        )?;
        let relative = choose(
            NumericalProvenanceField::RelativeTolerance,
            candidates
                .iter()
                .filter_map(|r| r.declaration.relative_tolerance.map(|v| (*r, v)))
                .collect(),
            0.0,
            NumericalSource::CanonicalFallback,
        )?;
        let budget = absolute + relative * nominal;
        if !nominal.is_finite() || nominal <= 0.0 || !budget.is_finite() || budget <= 0.0 {
            return Err(failure(
                target.id,
                "resolved nominal or budget is not finite and positive",
            ));
        }
        if target.integer
            && candidates.iter().any(|r| {
                r.declaration.required
                    && r.declaration.scaling_factor.is_some_and(|v| v != 1.0)
                    && provenance.iter().any(|p| {
                        p.field == NumericalProvenanceField::Nominal
                            && p.selected
                            && p.declaration == Some(r.declaration.requirement_id)
                    })
            })
        {
            return Err(failure(
                target.id,
                "required variable substitution changes the integer lattice",
            ));
        }
        key.id(&target.id)
            .id(&target.quantity.as_id())
            .id(&target.unit.as_id());
        target.kind.frame(&mut key);
        for value in [nominal, coordinate_scale, absolute, relative, budget] {
            key.u64(value.to_bits());
        }
        if policy.caloric_requirements.is_empty() {
            key.bool(selected_engineering.is_some());
            if let Some(context) = &selected_engineering {
                macro_rules! legacy_field {
                    ($field:ident) => {{
                        key.str(concat!("r#", stringify!($field)));
                        context.$field.frame(&mut key);
                    }};
                }
                legacy_field!(rule_id);
                legacy_field!(scale_id);
                legacy_field!(physical_allowance);
                legacy_field!(relative_fraction);
                legacy_field!(characteristic);
                legacy_field!(scale_kind);
                legacy_field!(source);
                legacy_field!(budget);
                legacy_field!(canonical_fallback);
                legacy_field!(limitation);
            }
        } else {
            selected_engineering.frame(&mut key);
        }
        for p in &provenance {
            p.declaration.frame(&mut key);
            p.source.frame(&mut key);
            // The framed spelling is the member's registry spelling, as before.
            key.str(p.field.as_str())
                .bool(p.selected)
                .u64(p.value.to_bits())
                .str(&p.description);
        }
        resolved.push(ResolvedTarget {
            id: target.id,
            kind: target.kind,
            quantity: target.quantity.as_id(),
            unit: target.unit.as_id(),
            nominal,
            coordinate_scale,
            required_scale: candidates.iter().any(|r| {
                r.declaration.required
                    && (r.declaration.nominal.is_some() || r.declaration.scaling_factor.is_some())
                    && provenance.iter().any(|p| {
                        p.field == NumericalProvenanceField::Nominal
                            && p.selected
                            && p.declaration == Some(r.declaration.requirement_id)
                    })
            }),
            absolute,
            relative,
            budget,
            engineering: selected_engineering,
            provenance,
        });
    }
    Ok(ResolvedNumericalPolicy {
        policy: policy.clone(),
        targets: resolved,
        key: key.finish_hash(),
    })
}

impl crate::assembly::CasePlan {
    /// Complete source coordinates, including fixed variables whose requirements remain
    /// meaningful, and the parameter coordinates of a parametric plan.
    pub fn numerical_targets(
        &self,
        registry: &QuantityRegistry,
    ) -> Result<Vec<TargetSpec>, MathError> {
        let mut out = self.structure().numerical_targets(registry)?;
        out.extend(self.parameter_targets());
        Ok(out)
    }
    /// The parameter coordinates of a parametric plan ([`Self::parametric`]) as variable
    /// targets: the analysis differentiates along them as it does along a free variable,
    /// so each resolves a coordinate scale and a budget through the same sources, and the
    /// scale's source is recorded with it. Empty for every other plan.
    pub fn parameter_targets(&self) -> Vec<TargetSpec> {
        let parameters: BTreeMap<_, _> = self
            .structure()
            .parameters()
            .iter()
            .map(|p| (p.id, p))
            .collect();
        self.columns()
            .iter()
            .filter_map(|id| parameters.get(id))
            .map(|p| TargetSpec {
                id: p.id,
                kind: NumericalTarget::Variable,
                quantity: p.quantity,
                unit: p.unit,
                integer: false,
                declared_tolerance: None,
            })
            .collect()
    }
}
impl crate::binding::CaseStructure {
    /// Source-coordinate inventory; model normalization never changes physical authority.
    pub fn numerical_targets(
        &self,
        registry: &QuantityRegistry,
    ) -> Result<Vec<TargetSpec>, MathError> {
        let mut out: Vec<_> = self
            .variables()
            .iter()
            .map(|v| TargetSpec {
                id: v.port.id,
                kind: NumericalTarget::Variable,
                quantity: v.port.quantity,
                unit: v.port.unit,
                integer: v.domain.is_integer(),
                declared_tolerance: None,
            })
            .collect();
        for row in self.rows() {
            out.push(TargetSpec {
                id: row.id,
                kind: NumericalTarget::Row,
                quantity: row.quantity,
                unit: registry.quantity_type(row.quantity)?.canonical_unit,
                integer: false,
                declared_tolerance: None,
            });
        }
        if let Some(objective) = self.objective() {
            out.push(TargetSpec {
                id: SemanticId::NIL,
                kind: NumericalTarget::Objective,
                quantity: objective.quantity,
                unit: registry.quantity_type(objective.quantity)?.canonical_unit,
                integer: false,
                declared_tolerance: None,
            });
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_model::generated::enums::EngineeringScaleKind;
    use pse_quantity::standard::{ids, standard_registry};
    fn facts() -> PhysicalPreconditions {
        PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions()).unwrap()
    }
    fn id(n: u8) -> SemanticId {
        SemanticId::from_bytes([n; 16])
    }
    fn target() -> TargetSpec {
        TargetSpec {
            id: id(1),
            kind: NumericalTarget::Variable,
            quantity: ids::quantity("temperature.point"),
            unit: ids::unit("K"),
            integer: false,
            declared_tolerance: None,
        }
    }
    #[test]
    fn caloric_context_resolves_frozen_consumer_without_widening_a_rule() {
        use pse_model::generated::authored::caloric_requirements::AuthoredCaloricRequirementsFieldConsumersItem as Consumer;
        let registry = standard_registry().unwrap();
        let quantity = |name: &str| match registry.physical_name(name).unwrap() {
            pse_quantity::PhysicalName::QuantityType(id) => id,
            _ => panic!("expected quantity {name}"),
        };
        let unit = |quantity| registry.quantity_type(quantity).unwrap().canonical_unit;
        let delta = |quantity| {
            pse_quantity::scheme::Scheme::Delta(Box::new(pse_quantity::scheme::Scheme::Concrete(
                quantity,
            )))
            .resolve(&registry, &BTreeMap::new())
            .unwrap()
        };
        let h = quantity("MolarEnthalpy");
        let dh = delta(h);
        let cp = quantity("MolarCp");
        let dt = delta(quantity("Temperature"));
        let power = delta(quantity("Power"));
        let flow = quantity("Flow");
        let target = TargetSpec {
            id: id(1),
            kind: NumericalTarget::Variable,
            quantity: h,
            unit: unit(h),
            integer: false,
            declared_tolerance: None,
        };
        let mut policy = NumericalPolicy::default();
        policy.strict_engineering_context = true;
        policy.caloric_requirements.push(CaloricRequirement {
            requirement_id: id(2),
            instance_id: None,
            target_id: target.id,
            target_kind: target.kind,
            quantity_id: dh.as_id(),
            unit_id: unit(dh).as_id(),
            source: NumericalSource::Model,
            science_revision: "selected-revision".into(),
            science_records: vec![id(3)],
            response_quantity_id: Some(cp.as_id()),
            response_unit_id: Some(unit(cp).as_id()),
            response: Some(100.0),
            thermal_quantity_id: Some(dt.as_id()),
            thermal_unit_id: Some(unit(dt).as_id()),
            thermal_resolution: Some(0.1),
            consumers: vec![Consumer {
                target_id: id(4),
                declaration_id: id(6),
                power_rule_id: id(7),
                actual_flow_id: id(8),
                exposure_expression: "feed/purge".into(),
                power_quantity_id: power.as_id(),
                power_unit_id: unit(power).as_id(),
                power_allowance: 1.0,
                exposure_quantity_id: flow.as_id(),
                exposure_unit_id: unit(flow).as_id(),
                aggregate_exposure: 4.0,
            }],
        });
        let result = resolve(
            &registry,
            &facts(),
            std::slice::from_ref(&target),
            &[],
            &policy,
        )
        .unwrap();
        let context = result.targets[0].engineering.as_ref().unwrap();
        assert!((context.budget - 0.25).abs() < 1e-12);
        assert_eq!(
            context.caloric.as_ref().unwrap().limiting_consumer,
            Some(id(4))
        );
        assert!(!context.canonical_fallback);
        // Resolve the actual consuming point with the ordinary rule and scale,
        // rather than treating its 1 W floor as the complete power budget.
        let power_point = quantity("Power");
        let consumer_target = TargetSpec {
            id: id(4),
            kind: NumericalTarget::Variable,
            quantity: power_point,
            unit: unit(power_point),
            integer: false,
            declared_tolerance: None,
        };
        let mut consuming = policy.clone();
        consuming.caloric_requirements[0].response = None;
        consuming.caloric_requirements[0].response_quantity_id = None;
        consuming.caloric_requirements[0].response_unit_id = None;
        consuming.caloric_requirements[0].thermal_quantity_id = None;
        consuming.caloric_requirements[0].thermal_unit_id = None;
        consuming.caloric_requirements[0].thermal_resolution = None;
        consuming.engineering_rules.push(EngineeringRule {
            rule_id: id(7).into(),
            quantity_id: power.as_id(),
            unit_id: unit(power).as_id(),
            physical_allowance: Some(1.0),
            relative_fraction: Some(0.001),
            provenance: "consumer power floor".into(),
        });
        let mut power_scale = scale(50, NumericalSource::Model, 100_000.0);
        power_scale.target_id = consumer_target.id;
        power_scale.quantity_id = power_point.as_id();
        power_scale.unit_id = unit(power_point).as_id();
        consuming.engineering_scales.push(power_scale);
        let targets = [target.clone(), consumer_target.clone()];
        let budget = |policy: &NumericalPolicy| {
            resolve(&registry, &facts(), &targets, &[], policy)
                .unwrap()
                .targets
                .into_iter()
                .find(|row| row.id == target.id)
                .unwrap()
                .budget
        };
        assert!((budget(&consuming) - 25.0).abs() < 1e-12);
        let mut explicit_enthalpy = requirement(52, NumericalSource::Analysis, 1.0).declaration;
        explicit_enthalpy.nominal = None;
        explicit_enthalpy.absolute_tolerance = Some(0.01);
        explicit_enthalpy.relative_tolerance = None;
        explicit_enthalpy.unit_id = Some(unit(h).as_id());
        let mut operational = consuming.clone();
        operational.requirements.push(explicit_enthalpy);
        let frozen = resolve(&registry, &facts(), &targets, &[], &operational).unwrap();
        assert!(
            frozen
                .targets
                .iter()
                .find(|row| row.id == target.id)
                .unwrap()
                .engineering
                .is_none()
        );
        assert!(
            (operational_output_context(
                &registry,
                &facts(),
                &target,
                &operational,
                &frozen.targets
            )
            .unwrap()
            .budget
                - 25.0)
                .abs()
                < 1e-12
        );
        let mut thermal_consumer = consuming.clone();
        thermal_consumer.caloric_requirements[0].response_quantity_id = Some(cp.as_id());
        thermal_consumer.caloric_requirements[0].response_unit_id = Some(unit(cp).as_id());
        thermal_consumer.caloric_requirements[0].response = Some(1000.0);
        thermal_consumer.caloric_requirements[0].thermal_quantity_id = Some(dt.as_id());
        thermal_consumer.caloric_requirements[0].thermal_unit_id = Some(unit(dt).as_id());
        thermal_consumer.caloric_requirements[0].thermal_resolution = Some(0.1);
        assert!((budget(&thermal_consumer) - 25.0).abs() < 1e-12);
        let mut explicit = requirement(51, NumericalSource::Analysis, 1.0).declaration;
        explicit.target_id = consumer_target.id;
        explicit.nominal = None;
        explicit.absolute_tolerance = Some(0.5);
        explicit.relative_tolerance = None;
        explicit.unit_id = Some(unit(power_point).as_id());
        consuming.requirements.push(explicit.clone());
        thermal_consumer.requirements.push(explicit);
        assert!((budget(&consuming) - 0.125).abs() < 1e-12);
        assert!((budget(&thermal_consumer) - 0.125).abs() < 1e-12);
        operational
            .requirements
            .push(consuming.requirements[0].clone());
        let frozen = resolve(&registry, &facts(), &targets, &[], &operational).unwrap();
        assert!(
            (operational_output_context(
                &registry,
                &facts(),
                &target,
                &operational,
                &frozen.targets
            )
            .unwrap()
            .budget
                - 0.125)
                .abs()
                < 1e-12
        );
        consuming.requirements.clear();
        consuming.engineering_scales.clear();
        assert!((budget(&consuming) - 0.25).abs() < 1e-12);
        let mut invalid = policy.clone();
        invalid.caloric_requirements[0].consumers[0].aggregate_exposure = -1.0;
        assert!(operational_output_context(&registry, &facts(), &target, &invalid, &[]).is_err());
        let mut tied = policy.clone();
        let mut second_consumer = tied.caloric_requirements[0].consumers[0].clone();
        second_consumer.target_id = id(10);
        second_consumer.declaration_id = id(11);
        tied.caloric_requirements[0].consumers.push(second_consumer);
        let tied_forward = resolve(
            &registry,
            &facts(),
            std::slice::from_ref(&target),
            &[],
            &tied,
        )
        .unwrap();
        tied.caloric_requirements[0].consumers.reverse();
        let tied_reverse = resolve(
            &registry,
            &facts(),
            std::slice::from_ref(&target),
            &[],
            &tied,
        )
        .unwrap();
        assert_eq!(tied_forward.key, tied_reverse.key);
        assert_eq!(
            tied_forward.targets[0]
                .engineering
                .as_ref()
                .unwrap()
                .caloric
                .as_ref()
                .unwrap()
                .limiting_consumer,
            Some(id(4))
        );
        let mut thermal_policy = policy.clone();
        thermal_policy.caloric_requirements[0].consumers[0].power_allowance = 1000.0;
        let thermal = resolve(
            &registry,
            &facts(),
            std::slice::from_ref(&target),
            &[],
            &thermal_policy,
        )
        .unwrap();
        let mut fahrenheit = thermal_policy;
        fahrenheit.caloric_requirements[0].thermal_unit_id = Some(ids::unit("degF").as_id());
        fahrenheit.caloric_requirements[0].thermal_resolution = Some(0.18);
        let converted = resolve(
            &registry,
            &facts(),
            std::slice::from_ref(&target),
            &[],
            &fahrenheit,
        )
        .unwrap();
        assert!((converted.targets[0].budget - thermal.targets[0].budget).abs() < 1e-12);
        assert!((thermal.targets[0].budget - 10.0).abs() < 1e-12);
        let projected = project_difference(
            &registry,
            &facts(),
            &result,
            &[TargetProjection {
                source: target.id,
                source_kind: target.kind,
                target: TargetSpec {
                    id: id(20),
                    kind: NumericalTarget::Row,
                    quantity: dh,
                    unit: unit(dh),
                    integer: false,
                    declared_tolerance: None,
                },
            }],
        )
        .unwrap();
        let projected_context = projected.targets[0]
            .engineering
            .as_ref()
            .unwrap()
            .caloric
            .as_ref()
            .unwrap();
        assert!((projected_context.allowance - 0.25).abs() < 1e-12);
        assert_eq!(projected_context.science_revision, "selected-revision");
        assert_eq!(projected_context.response_unit_id, Some(unit(cp).as_id()));
        assert_eq!(projected_context.consumers[0].actual_flow_id, id(8));
        let mut nominal = requirement(21, NumericalSource::Model, 100_000.0);
        nominal.declaration.unit_id = Some(target.unit.as_id());
        nominal.declaration.absolute_tolerance = None;
        nominal.declaration.relative_tolerance = None;
        let shifted_nominal = resolve(
            &registry,
            &facts(),
            std::slice::from_ref(&target),
            &[nominal],
            &policy,
        )
        .unwrap();
        assert!((shifted_nominal.targets[0].budget - result.targets[0].budget).abs() < 1e-12);
        let mut energy_only = policy.clone();
        let requirement = &mut energy_only.caloric_requirements[0];
        requirement.science_records.clear();
        requirement.response_quantity_id = None;
        requirement.response_unit_id = None;
        requirement.response = None;
        requirement.thermal_quantity_id = None;
        requirement.thermal_unit_id = None;
        requirement.thermal_resolution = None;
        let energy = resolve(
            &registry,
            &facts(),
            std::slice::from_ref(&target),
            &[],
            &energy_only,
        )
        .unwrap();
        let context = energy.targets[0].engineering.as_ref().unwrap();
        assert!((context.budget - 0.25).abs() < 1e-12);
        assert!(context.caloric.as_ref().unwrap().response.is_none());
        assert!(context.limitation.contains("no thermal-response"));
        let before = energy_only.key();
        energy_only.caloric_requirements[0].consumers[0].aggregate_exposure = 5.0;
        assert_ne!(before, energy_only.key());
        let changed = resolve(
            &registry,
            &facts(),
            std::slice::from_ref(&target),
            &[],
            &energy_only,
        )
        .unwrap();
        assert!((changed.targets[0].budget - 0.2).abs() < 1e-12);
        // Equal physical copies can use different source units. Their retained
        // representative is selected by stable requirement identity, not input order.
        let mut alternate_watt = registry.unit(unit(power)).unwrap().clone();
        alternate_watt.id = ids::unit("test_mW");
        alternate_watt.symbol = "test_mW".into();
        alternate_watt.scale_to_canonical = 0.001;
        let mut builder = registry.to_builder();
        builder.unit(alternate_watt);
        let alternate_registry = builder.build().unwrap();
        let mut equivalent = energy_only.clone();
        equivalent.caloric_requirements[0].consumers[0].aggregate_exposure = 1.0;
        let mut alternate = equivalent.caloric_requirements[0].clone();
        alternate.requirement_id = id(30);
        alternate.target_id = id(31);
        alternate.consumers[0].power_unit_id = ids::unit("test_mW").as_id();
        alternate.consumers[0].power_allowance = 1000.0;
        equivalent.caloric_requirements.push(alternate);
        let mut alternate_target = target.clone();
        alternate_target.id = id(31);
        let pair = [target.clone(), alternate_target];
        let forward = resolve(&alternate_registry, &facts(), &pair, &[], &equivalent).unwrap();
        equivalent.caloric_requirements.reverse();
        let reverse = resolve(&alternate_registry, &facts(), &pair, &[], &equivalent).unwrap();
        assert_eq!(forward.key, reverse.key);
        let mut shared = energy_only.clone();
        shared.caloric_requirements[0].consumers[0].aggregate_exposure = 1.0;
        let mut other = shared.caloric_requirements[0].clone();
        other.requirement_id = id(30);
        other.target_id = id(31);
        other.consumers[0].declaration_id = id(32);
        other.consumers[0].actual_flow_id = id(33);
        let mut projected_copy = shared.caloric_requirements[0].clone();
        projected_copy.requirement_id = id(34);
        projected_copy.target_id = id(35);
        shared.caloric_requirements.extend([other, projected_copy]);
        let mut target2 = target.clone();
        target2.id = id(31);
        let mut target3 = target.clone();
        target3.id = id(35);
        let all = [target.clone(), target2, target3];
        let first = resolve(&registry, &facts(), &all, &[], &shared).unwrap();
        assert!(
            first
                .targets
                .iter()
                .all(|target| (target.budget - 0.5).abs() < 1e-12)
        );
        for target in &first.targets {
            let consumer = &target
                .engineering
                .as_ref()
                .unwrap()
                .caloric
                .as_ref()
                .unwrap()
                .consumers[0];
            assert!((consumer.combined_exposure - 2.0).abs() < 1e-12);
            assert_eq!(consumer.contributors.len(), 2);
        }
        shared.caloric_requirements.reverse();
        let reversed = resolve(&registry, &facts(), &all, &[], &shared).unwrap();
        assert_eq!(first.key, reversed.key);
        let mut many = energy_only.clone();
        many.caloric_requirements.clear();
        let mut many_targets = Vec::new();
        for index in 0_u8..12 {
            let mut requirement = energy_only.caloric_requirements[0].clone();
            requirement.requirement_id = id(40 + index);
            requirement.target_id = id(60 + index);
            requirement.consumers[0].declaration_id = id(80 + index);
            requirement.consumers[0].actual_flow_id = id(100 + index);
            requirement.consumers[0].exposure_expression = "x".repeat(4096);
            many.caloric_requirements.push(requirement);
            let mut member = target.clone();
            member.id = id(60 + index);
            many_targets.push(member);
        }
        let bound = resolution_allocation_bound(&many_targets, &[], &many);
        let many_resolved = resolve(&registry, &facts(), &many_targets, &[], &many).unwrap();
        assert!(
            bound >= resolved_extent(&many_resolved),
            "shared-consumer result exceeds its admitted bound"
        );
        shared.caloric_requirements[0].target_id = id(99);
        assert!(resolve(&registry, &facts(), &all, &[], &shared).is_err());
        shared.caloric_requirements[0].target_id = id(35);
        shared.caloric_requirements[0].quantity_id = id(100);
        assert!(resolve(&registry, &facts(), &all, &[], &shared).is_err());
        policy.engineering_rules.push(EngineeringRule {
            rule_id: id(5).into(),
            quantity_id: dh.as_id(),
            unit_id: unit(dh).as_id(),
            physical_allowance: Some(0.1),
            relative_fraction: Some(0.0),
            provenance: "independent physical requirement".into(),
        });
        let tighter = resolve(&registry, &facts(), &[target], &[], &policy).unwrap();
        assert!((tighter.targets[0].budget - 0.1).abs() < 1e-12);
    }
    #[test]
    fn target_access_preserves_kinds_order_frozen_budgets_and_duplicate_requests() {
        let registry = standard_registry().unwrap();
        let mut row = target();
        row.kind = NumericalTarget::Row;
        let policy = resolve(
            &registry,
            &facts(),
            &[row, target()],
            &[],
            &NumericalPolicy::default(),
        )
        .unwrap();
        let access = TargetAccess::new(&policy).unwrap();
        let variable = access.get(NumericalTarget::Variable, id(1)).unwrap();
        let row = access.get(NumericalTarget::Row, id(1)).unwrap();
        assert_eq!(variable.kind, NumericalTarget::Variable);
        assert_eq!(row.kind, NumericalTarget::Row);
        let requested = access
            .ordered(NumericalTarget::Variable, &[id(1), id(1)])
            .unwrap();
        assert_eq!(requested.len(), 2);
        assert!(std::ptr::eq(requested[0], variable));
        assert!(std::ptr::eq(requested[1], variable));
        assert!(access.get(NumericalTarget::Variable, id(2)).is_err());
        let scales =
            crate::normalization::Normalization::from_policy(&policy, &[id(1)], &[id(1)]).unwrap();
        assert_eq!(scales.variables, [variable.coordinate_scale]);
        assert_eq!(scales.rows, [row.coordinate_scale]);
        let mut ambiguous = policy.clone();
        ambiguous.targets.push(variable.clone());
        assert!(TargetAccess::new(&ambiguous).is_err());
    }

    fn requirement(n: u8, source: NumericalSource, nominal: f64) -> SourcedRequirement {
        SourcedRequirement {
            source,
            declaration: NumericalRequirement {
                requirement_id: id(n),
                model_id: Some(id(90).into()),
                case_id: None,
                instance_id: None,
                fit_id: None,
                target_id: id(1),
                target_kind: NumericalTarget::Variable,
                nominal: Some(nominal),
                scaling_factor: None,
                absolute_tolerance: Some(0.5),
                relative_tolerance: Some(0.01),
                shared_engineering_allowance: None,
                engineering_rule_id: None,
                unit_id: Some(ids::unit("degC").as_id()),
                coordinates: NumericalCoordinates::Physical,
                priority: 0,
                required: true,
                provenance: "authored temperature magnitude".into(),
            },
        }
    }
    fn scale(n: u8, source: NumericalSource, value: f64) -> EngineeringScale {
        EngineeringScale {
            scale_id: id(n).into(),
            model_id: Some(id(90).into()),
            case_id: None,
            instance_id: None,
            fit_id: None,
            target_id: id(1),
            target_kind: NumericalTarget::Variable,
            quantity_id: ids::quantity("temperature.point").as_id(),
            unit_id: ids::unit("K").as_id(),
            kind: EngineeringScaleKind::RangeWidth,
            value,
            source,
            priority: 0,
            provenance: "declared temperature operating range width".into(),
        }
    }
    fn rule(n: u8, allowance: Option<f64>) -> EngineeringRule {
        EngineeringRule {
            rule_id: id(n).into(),
            quantity_id: ids::quantity("temperature.difference").as_id(),
            unit_id: ids::unit("K").as_id(),
            physical_allowance: allowance,
            relative_fraction: None,
            provenance: "shared temperature allowance".into(),
        }
    }
    #[test]
    fn engineering_accuracy_operational_output_is_independent_of_bound_and_kkt_budgets() {
        let registry = standard_registry().unwrap();
        let mut bound = requirement(2, NumericalSource::Analysis, 300.);
        bound.declaration.relative_tolerance = Some(0.);
        bound.declaration.absolute_tolerance = Some(1e-9);
        let policy = NumericalPolicy {
            requirements: vec![bound.declaration],
            engineering_rules: vec![rule(4, Some(0.1))],
            ..Default::default()
        };
        let output =
            operational_output_context(&registry, &facts(), &target(), &policy, &[]).unwrap();
        assert_eq!(output.budget, 0.1);
        let mut changed = policy.clone();
        changed.requirements[0].absolute_tolerance = Some(10.);
        changed.kkt.stationarity = 1e-12;
        changed.kkt.complementarity = 0.5;
        assert_eq!(
            operational_output_context(&registry, &facts(), &target(), &changed, &[]).unwrap(),
            output
        );
    }
    #[test]
    fn engineering_accuracy_uses_frozen_scale_and_equivalent_unit_magnitudes() {
        let registry = standard_registry().unwrap();
        let mut scale = requirement(2, NumericalSource::Model, 300.0);
        scale.declaration.unit_id = Some(ids::unit("K").as_id());
        scale.declaration.absolute_tolerance = None;
        scale.declaration.relative_tolerance = None;
        let policy = NumericalPolicy {
            engineering_scales: vec![self::scale(8, NumericalSource::Model, 300.0)],
            engineering_rules: vec![rule(9, Some(0.1))],
            ..Default::default()
        };
        for (unit, magnitude) in [("K", 1.0), ("degC", 1.0), ("degF", 1.8)] {
            let coordinate = TargetSpec {
                unit: ids::unit(unit),
                ..target()
            };
            let resolved = resolve(
                &registry,
                &facts(),
                &[coordinate],
                &[scale.clone()],
                &policy,
            )
            .unwrap();
            let value = &resolved.targets[0];
            assert!((value.nominal - 300.0 * magnitude).abs() < 1e-12);
            assert!((value.budget - 0.3 * magnitude).abs() < 1e-12);
            assert!(
                (value.budget / value.coordinate_scale - DEFAULT_ENGINEERING_ACCURACY).abs()
                    < 1e-15
            );
            assert!(
                value
                    .provenance
                    .iter()
                    .any(|p| p.field == NumericalProvenanceField::AbsoluteTolerance
                        && p.selected
                        && p.declaration.is_none())
            );
        }
    }
    #[test]
    fn engineering_accuracy_preserves_authored_and_analysis_precision() {
        let registry = standard_registry().unwrap();
        let mut authored = requirement(2, NumericalSource::Model, 300.0);
        authored.declaration.unit_id = Some(ids::unit("K").as_id());
        authored.declaration.absolute_tolerance = Some(0.01);
        authored.declaration.relative_tolerance = Some(0.0);
        authored.declaration.provenance = "design temperature decision resolution: 0.01 K".into();
        let resolved = resolve(
            &registry,
            &facts(),
            &[target()],
            &[authored.clone()],
            &Default::default(),
        )
        .unwrap();
        assert_eq!(resolved.targets[0].budget, 0.01);
        let mut policy = NumericalPolicy::default();
        let mut verification = authored.declaration.clone();
        verification.requirement_id = id(3);
        verification.absolute_tolerance = Some(1e-8);
        verification.provenance =
            "analytic derivative verification requires 1e-8 K evaluation accuracy".into();
        policy.requirements.push(verification);
        let resolved = resolve(&registry, &facts(), &[target()], &[authored], &policy).unwrap();
        assert_eq!(resolved.targets[0].budget, 1e-8);
        assert!(resolved.targets[0].provenance.iter().any(|p| p.field
            == NumericalProvenanceField::AbsoluteTolerance
            && p.selected
            && p.source == NumericalSource::Analysis
            && p.declaration == Some(id(3))));
        let declared = TargetSpec {
            declared_tolerance: Some(0.005),
            ..target()
        };
        let resolved = resolve(&registry, &facts(), &[declared], &[], &Default::default()).unwrap();
        assert_eq!(resolved.targets[0].budget, 0.005);
    }
    #[test]
    fn contextual_engineering_is_independent_of_conditioning_and_zero_net_values() {
        let registry = standard_registry().unwrap();
        let mut policy = NumericalPolicy {
            engineering_scales: vec![scale(8, NumericalSource::Model, 300.0)],
            engineering_rules: vec![rule(9, Some(0.1))],
            ..Default::default()
        };
        for nominal in [1.0, 300.0, 1e6] {
            let mut conditioning = requirement(2, NumericalSource::Model, nominal);
            conditioning.declaration.absolute_tolerance = None;
            conditioning.declaration.relative_tolerance = None;
            let resolved =
                resolve(&registry, &facts(), &[target()], &[conditioning], &policy).unwrap();
            assert_eq!(resolved.targets[0].nominal, nominal);
            assert_eq!(resolved.targets[0].budget, 0.3);
            assert_eq!(resolved.targets[0].relative, 0.0);
            assert_eq!(
                resolved.targets[0].engineering.as_ref().unwrap().scale_id,
                Some(id(8).into())
            );
        }
        policy.engineering_scales[0].value = 0.0;
        let resolved = resolve(&registry, &facts(), &[target()], &[], &policy).unwrap();
        let context = resolved.targets[0].engineering.as_ref().unwrap();
        assert_eq!(resolved.targets[0].budget, 0.1);
        assert_eq!(context.characteristic, Some(0.0));
        assert!(!context.canonical_fallback);
    }
    #[test]
    fn contextual_engineering_canonical_fallback_is_recorded_and_strict_is_effective_only() {
        let registry = standard_registry().unwrap();
        let mut conditioning = requirement(2, NumericalSource::Model, 1e6);
        conditioning.declaration.absolute_tolerance = None;
        conditioning.declaration.relative_tolerance = None;
        let resolved = resolve(
            &registry,
            &facts(),
            &[target()],
            &[conditioning.clone()],
            &Default::default(),
        )
        .unwrap();
        assert_eq!(resolved.targets[0].budget, 0.001);
        let context = resolved.targets[0].engineering.as_ref().unwrap();
        assert!(context.canonical_fallback);
        assert!(!context.limitation.is_empty());
        let strict = NumericalPolicy {
            strict_engineering_context: true,
            ..Default::default()
        };
        assert!(
            resolve(
                &registry,
                &facts(),
                &[target()],
                &[conditioning.clone()],
                &strict
            )
            .is_err()
        );
        conditioning.declaration.absolute_tolerance = Some(0.02);
        let explicit = resolve(&registry, &facts(), &[target()], &[conditioning], &strict).unwrap();
        assert_eq!(explicit.targets[0].budget, 0.02);
        assert!(explicit.targets[0].engineering.is_none());
    }
    #[test]
    fn contextual_engineering_source_precedence_conflicts_and_rule_selection() {
        let registry = standard_registry().unwrap();
        let mut policy = NumericalPolicy {
            engineering_scales: vec![
                scale(8, NumericalSource::ModelHint, 1e6),
                scale(9, NumericalSource::PropertyDefault, 200.0),
            ],
            engineering_rules: vec![rule(10, Some(0.1))],
            ..Default::default()
        };
        let resolved = resolve(&registry, &facts(), &[target()], &[], &policy).unwrap();
        assert_eq!(resolved.targets[0].budget, 0.2);
        assert_eq!(
            resolved.targets[0].engineering.as_ref().unwrap().source,
            NumericalSource::PropertyDefault
        );
        policy
            .engineering_scales
            .push(scale(11, NumericalSource::PropertyDefault, 300.0));
        assert!(resolve(&registry, &facts(), &[target()], &[], &policy).is_err());
        policy.engineering_scales.pop();
        policy.engineering_rules.push(rule(12, Some(0.7)));
        assert!(resolve(&registry, &facts(), &[target()], &[], &policy).is_err());
        let mut inherited = requirement(2, NumericalSource::Model, 20.0);
        inherited.declaration.shared_engineering_allowance = Some(true);
        inherited.declaration.engineering_rule_id = Some(id(12).into());
        inherited.declaration.relative_tolerance = None;
        let resolved = resolve(
            &registry,
            &facts(),
            &[target()],
            &[inherited.clone()],
            &policy,
        )
        .unwrap();
        assert_eq!(resolved.targets[0].budget, 0.7);
        assert_eq!(
            resolved.targets[0].engineering.as_ref().unwrap().rule_id,
            Some(id(12).into())
        );
        inherited.declaration.shared_engineering_allowance = Some(false);
        assert!(
            resolve(
                &registry,
                &facts(),
                &[target()],
                &[inherited.clone()],
                &policy
            )
            .is_err()
        );
        inherited.declaration.engineering_rule_id = None;
        let explicit = resolve(&registry, &facts(), &[target()], &[inherited], &policy).unwrap();
        assert_eq!(explicit.targets[0].budget, 0.5);
        assert!(explicit.targets[0].engineering.is_none());
        policy.engineering_rules.pop();
        policy.engineering_scales[0].source = NumericalSource::DerivedNominal;
        assert!(resolve(&registry, &facts(), &[target()], &[], &policy).is_err());
    }
    #[test]
    fn contextual_engineering_fraction_identity_and_full_quantity_meaning() {
        let registry = standard_registry().unwrap();
        let mut policy = NumericalPolicy {
            engineering_scales: vec![scale(8, NumericalSource::Model, 300.0)],
            engineering_rules: vec![rule(9, Some(0.1))],
            ..Default::default()
        };
        let initial = resolve(&registry, &facts(), &[target()], &[], &policy).unwrap();
        policy.engineering_relative_fraction = 0.002;
        let changed = resolve(&registry, &facts(), &[target()], &[], &policy).unwrap();
        assert_eq!(changed.targets[0].budget, 0.6);
        assert_ne!(initial.key, changed.key);
        policy.engineering_rules[0].relative_fraction = Some(0.001);
        let overridden = resolve(&registry, &facts(), &[target()], &[], &policy).unwrap();
        assert_eq!(overridden.targets[0].budget, 0.3);
        // Equal dimensions do not admit a point quantity as a difference quantity.
        policy.engineering_scales[0].quantity_id = ids::quantity("temperature.difference").as_id();
        assert!(resolve(&registry, &facts(), &[target()], &[], &policy).is_err());
        policy.engineering_scales.clear();
        policy.engineering_rules[0].quantity_id = ids::quantity("temperature.point").as_id();
        let unmatched = resolve(&registry, &facts(), &[target()], &[], &policy).unwrap();
        assert_eq!(unmatched.targets[0].budget, 0.001);
        assert!(
            unmatched.targets[0]
                .engineering
                .as_ref()
                .unwrap()
                .canonical_fallback
        );
    }
    #[test]
    fn contextual_engineering_policy_frames_are_declaration_order_independent() {
        let registry = standard_registry().unwrap();
        let mut policy = NumericalPolicy {
            engineering_scales: vec![
                scale(8, NumericalSource::Model, 300.0),
                scale(9, NumericalSource::Case, 200.0),
            ],
            engineering_rules: vec![rule(10, Some(0.1))],
            ..Default::default()
        };
        let key = policy.key();
        let resolved = resolve(&registry, &facts(), &[target()], &[], &policy).unwrap();
        assert_eq!(resolved.targets[0].budget, 0.2);
        policy.engineering_scales.reverse();
        assert_eq!(policy.key(), key);
        assert_eq!(
            resolve(&registry, &facts(), &[target()], &[], &policy)
                .unwrap()
                .key,
            resolved.key
        );
        policy.engineering_scales[0].kind = EngineeringScaleKind::Magnitude;
        assert_ne!(policy.key(), key);
        let mut duplicate = policy.clone();
        duplicate
            .engineering_scales
            .push(policy.engineering_scales[0].clone());
        assert!(duplicate.validate().is_err());
    }
    #[test]
    fn contextual_engineering_zero_fraction_uses_floor_or_recorded_fallback() {
        let registry = standard_registry().unwrap();
        let mut policy = NumericalPolicy {
            engineering_scales: vec![scale(8, NumericalSource::Model, 300.0)],
            engineering_relative_fraction: 0.0,
            ..Default::default()
        };
        let fallback = resolve(&registry, &facts(), &[target()], &[], &policy).unwrap();
        assert_eq!(fallback.targets[0].budget, 0.001);
        let context = fallback.targets[0].engineering.as_ref().unwrap();
        assert_eq!(context.characteristic, Some(300.0));
        assert_eq!(context.relative_fraction, 0.0);
        assert!(context.canonical_fallback);
        policy.strict_engineering_context = true;
        assert!(resolve(&registry, &facts(), &[target()], &[], &policy).is_err());
        policy.engineering_rules.push(rule(9, Some(0.1)));
        let floor = resolve(&registry, &facts(), &[target()], &[], &policy).unwrap();
        assert_eq!(floor.targets[0].budget, 0.1);
        assert!(
            !floor.targets[0]
                .engineering
                .as_ref()
                .unwrap()
                .canonical_fallback
        );
        // The selected rule owns its explicit fraction override.
        policy.engineering_rules[0].physical_allowance = None;
        policy.engineering_rules[0].relative_fraction = Some(0.002);
        let relative = resolve(&registry, &facts(), &[target()], &[], &policy).unwrap();
        assert_eq!(relative.targets[0].budget, 0.6);
        assert!(
            !relative.targets[0]
                .engineering
                .as_ref()
                .unwrap()
                .canonical_fallback
        );
    }
    #[test]
    fn contextual_engineering_affine_magnitudes_and_projection_transport() {
        let registry = standard_registry().unwrap();
        let mut engineering = scale(8, NumericalSource::Model, 18.0);
        engineering.kind = EngineeringScaleKind::ReferenceDifference;
        engineering.unit_id = ids::unit("degF").as_id();
        let policy = NumericalPolicy {
            engineering_scales: vec![engineering],
            engineering_rules: vec![rule(9, Some(0.001))],
            ..Default::default()
        };
        let source = resolve(&registry, &facts(), &[target()], &[], &policy).unwrap();
        assert!((source.targets[0].budget - 0.01).abs() < 1e-15);
        assert!(
            (source.targets[0]
                .engineering
                .as_ref()
                .unwrap()
                .characteristic
                .unwrap()
                - 10.0)
                .abs()
                < 1e-12
        );
        let projection = TargetProjection {
            source: id(1),
            source_kind: NumericalTarget::Variable,
            target: TargetSpec {
                id: id(3),
                quantity: ids::quantity("temperature.difference"),
                unit: ids::unit("degF"),
                ..target()
            },
        };
        let projected = project_difference(&registry, &facts(), &source, &[projection]).unwrap();
        let result = &projected.targets[0];
        assert!((result.budget - 0.018).abs() < 1e-15);
        let context = result.engineering.as_ref().unwrap();
        assert!((context.characteristic.unwrap() - 18.0).abs() < 1e-12);
        assert!((context.physical_allowance.unwrap() - 0.0018).abs() < 1e-15);
        assert_eq!(context.scale_id, Some(id(8).into()));
        assert_eq!(context.relative_fraction, 0.001);
        assert_eq!(context.rule_id, Some(id(9).into()));
    }
    #[test]
    fn numerical_projection_preserves_precedence_and_affine_magnitudes() {
        let registry = standard_registry().unwrap();
        let source = resolve(
            &registry,
            &facts(),
            &[target()],
            &[requirement(2, NumericalSource::Model, 20.0)],
            &Default::default(),
        )
        .unwrap();
        let mut projection = TargetProjection {
            source: id(1),
            source_kind: NumericalTarget::Variable,
            target: TargetSpec {
                id: id(3),
                kind: NumericalTarget::Row,
                unit: ids::unit("degC"),
                ..target()
            },
        };
        let projected = project(&registry, &source, &[projection.clone()]).unwrap();
        let result = &projected.targets[0];
        assert_eq!(result.nominal, 20.0);
        assert_eq!(result.budget, 0.7);
        assert_eq!(result.coordinate_scale, source.targets[0].coordinate_scale);
        assert_eq!(result.provenance, source.targets[0].provenance);
        assert_ne!(projected.key, source.key);
        let mut fahrenheit = projection.clone();
        fahrenheit.target.unit = ids::unit("degF");
        let converted = project(&registry, &source, &[fahrenheit]).unwrap();
        let converted = &converted.targets[0];
        assert!((converted.nominal - 36.0).abs() < 1e-12);
        assert!((converted.coordinate_scale - 36.0).abs() < 1e-12);
        assert!((converted.budget - 1.26).abs() < 1e-12);
        assert!(
            (0.9 / converted.coordinate_scale - 0.5 / source.targets[0].coordinate_scale).abs()
                < 1e-12
        );
        assert!(
            project(
                &registry,
                &source,
                &[projection.clone(), projection.clone()]
            )
            .is_err()
        );
        projection.source = id(4);
        assert!(project(&registry, &source, &[projection.clone()]).is_err());
        projection.source = id(1);
        projection.target.quantity = ids::quantity("neutral");
        assert!(project(&registry, &source, &[projection]).is_err());
    }
    #[test]
    fn numerical_difference_projection_preserves_affine_policy_and_refuses_wrong_contracts() {
        let registry = standard_registry().unwrap();
        let point = ids::quantity("temperature.point");
        let difference = ids::quantity("temperature.difference");
        let kelvin = ids::unit("K");
        let fahrenheit = ids::unit("degF");
        let original = TargetSpec {
            kind: NumericalTarget::Observable,
            quantity: point,
            unit: kelvin,
            ..target()
        };
        let mut authored = requirement(2, NumericalSource::Model, 8.0);
        authored.declaration.target_kind = NumericalTarget::Observable;
        authored.declaration.unit_id = Some(fahrenheit.as_id());
        authored.declaration.absolute_tolerance = Some(0.08);
        authored.declaration.scaling_factor = Some(0.125);
        let source = resolve(
            &registry,
            &facts(),
            &[original],
            &[authored],
            &Default::default(),
        )
        .unwrap();
        let mut projection = TargetProjection {
            source: id(1),
            source_kind: NumericalTarget::Observable,
            target: TargetSpec {
                id: id(3),
                kind: NumericalTarget::Row,
                quantity: difference,
                unit: fahrenheit,
                integer: false,
                declared_tolerance: None,
            },
        };
        let projected =
            project_difference(&registry, &facts(), &source, &[projection.clone()]).unwrap();
        let result = &projected.targets[0];
        assert_eq!(result.quantity, difference.as_id());
        assert!((result.nominal - 8.0).abs() < 1e-12);
        assert!((result.coordinate_scale - 8.0).abs() < 1e-12);
        assert!((result.absolute - 0.08).abs() < 1e-12);
        assert!((result.budget - 0.16).abs() < 1e-12);
        assert_eq!(result.relative, 0.01);
        assert!(result.required_scale);
        for (actual, original) in result.provenance.iter().zip(&source.targets[0].provenance) {
            assert_eq!(actual.declaration, original.declaration);
            assert_eq!(actual.source, original.source);
            assert_eq!(actual.selected, original.selected);
            let scale = if actual.field == NumericalProvenanceField::RelativeTolerance {
                1.0
            } else {
                9.0 / 5.0
            };
            assert!((actual.value - original.value * scale).abs() < 1e-12);
        }
        assert!(project(&registry, &source, &[projection.clone()]).is_err());
        assert!(
            project_difference(
                &registry,
                &facts(),
                &source,
                &[projection.clone(), projection.clone()]
            )
            .is_err()
        );
        projection.target.quantity = point;
        assert!(project_difference(&registry, &facts(), &source, &[projection.clone()]).is_err());
        projection.target.quantity = difference;
        projection.target.unit = ids::unit("s");
        assert!(project_difference(&registry, &facts(), &source, &[projection]).is_err());
    }
    /// Each provenance entry names its field by the registry enumeration, never by text,
    /// and the resolution key frames the member's registry spelling, as it framed the text
    /// before (ADR-0115 Outcome 3).
    #[test]
    fn provenance_field_typed() {
        use NumericalProvenanceField as F;
        let registry = standard_registry().unwrap();
        let resolved = resolve(
            &registry,
            &facts(),
            &[target()],
            &[requirement(2, NumericalSource::Model, 20.0)],
            &NumericalPolicy::default(),
        )
        .unwrap();
        let fields = resolved.targets[0]
            .provenance
            .iter()
            .map(|p| (p.field, p.selected))
            .collect::<Vec<_>>();
        assert_eq!(
            fields,
            [
                (F::Nominal, true),
                (F::AbsoluteTolerance, true),
                (F::RelativeTolerance, true)
            ]
        );
        assert_eq!(
            F::ALL.map(F::as_str),
            [
                "nominal",
                "absolute_tolerance",
                "relative_tolerance",
                "coordinate_scale"
            ]
        );
        let defaults = resolve(
            &registry,
            &facts(),
            &[target()],
            &[],
            &NumericalPolicy::default(),
        )
        .unwrap();
        assert!(
            defaults.targets[0]
                .provenance
                .iter()
                .all(|p| p.declaration.is_none() && p.field != F::CoordinateScale)
        );
    }
    #[test]
    fn numerical_policy_precedence_frozen_budget_and_affine_unit_magnitudes() {
        let registry = standard_registry().unwrap();
        let authored = [
            requirement(2, NumericalSource::Model, 10.0),
            requirement(3, NumericalSource::Case, 20.0),
        ];
        let p = resolve(
            &registry,
            &facts(),
            &[target()],
            &authored,
            &NumericalPolicy::default(),
        )
        .unwrap();
        let t = &p.targets[0];
        assert_eq!(t.nominal, 20.0); // Celsius offset is not a magnitude.
        assert_eq!(t.absolute, 0.5);
        assert_eq!(t.budget, 0.7);
        assert!(
            t.provenance
                .iter()
                .any(|p| p.declaration == Some(id(2)) && !p.selected)
        );
        let mut policy = NumericalPolicy::default();
        policy
            .requirements
            .push(requirement(4, NumericalSource::Analysis, 30.0).declaration);
        let override_ = resolve(&registry, &facts(), &[target()], &authored, &policy).unwrap();
        assert_eq!(override_.targets[0].nominal, 30.0);
        assert_ne!(override_.key, p.key);
        let conflicting = [
            requirement(2, NumericalSource::Model, 10.0),
            requirement(3, NumericalSource::Model, 20.0),
        ];
        assert!(resolve(&registry, &facts(), &[target()], &conflicting, &policy).is_ok());
        assert!(
            resolve(
                &registry,
                &facts(),
                &[target()],
                &conflicting,
                &NumericalPolicy::default()
            )
            .is_err()
        );
    }
    #[test]
    fn original_term_schemes_keep_names_zero_rules_and_finite_semantics() {
        use pse_model::generated::enums::ConstraintScalingScheme as S;
        for (scheme, expected) in [
            (S::HarmonicMean, 0.75),
            (S::InverseSum, 1.0 / 6.0),
            (S::InverseRSS, 1.0 / 20.0_f64.sqrt()),
            (S::InverseMaximum, 0.25),
            (S::InverseMinimum, 0.5),
        ] {
            assert!((term_scale(scheme, &[0.0, -2.0, 4.0]).unwrap() - expected).abs() < 1e-15);
            assert_eq!(term_scale(scheme, &[0.0, 0.0]).unwrap(), 1.0);
            assert!(term_scale(scheme, &[f64::NAN]).is_err());
            assert!(term_scale(scheme, &[]).is_err());
        }
        assert!(term_scale(S::InverseRSS, &[1e308, 1e308]).unwrap() > 0.0);
        let sources = [
            NumericalSource::CanonicalFallback,
            NumericalSource::QuantityNominal,
            NumericalSource::DerivedNominal,
            NumericalSource::PropertyDefault,
            NumericalSource::ModelHint,
            NumericalSource::Model,
            NumericalSource::Case,
            NumericalSource::Analysis,
        ];
        assert!(sources.windows(2).all(|s| rank(s[0]) < rank(s[1])));
    }
    #[test]
    fn numerical_policy_integer_normalized_budget_uses_lattice_coordinates() {
        let registry = standard_registry().unwrap();
        let mut t = target();
        t.integer = true;
        let mut r = requirement(9, NumericalSource::Model, 100.0);
        r.declaration.coordinates = NumericalCoordinates::Normalized;
        r.declaration.unit_id = None;
        r.declaration.absolute_tolerance = Some(0.25);
        r.declaration.relative_tolerance = Some(0.01);
        let result = resolve(&registry, &facts(), &[t], &[r], &Default::default()).unwrap();
        assert_eq!(result.targets[0].coordinate_scale, 1.0);
        assert_eq!(result.targets[0].budget, 1.25);
    }
    #[test]
    fn numerical_policy_recorded_fallback_and_integer_lattice() {
        let registry = standard_registry().unwrap();
        let mut t = target();
        t.integer = true;
        let p = resolve(
            &registry,
            &facts(),
            &[t.clone()],
            &[],
            &NumericalPolicy::default(),
        )
        .unwrap();
        assert_eq!(p.targets[0].nominal, 1.0);
        assert_eq!(
            p.targets[0].provenance[0].source,
            NumericalSource::CanonicalFallback
        );
        let strict = NumericalPolicy {
            strict_nominals: true,
            ..NumericalPolicy::default()
        };
        assert!(resolve(&registry, &facts(), &[t.clone()], &[], &strict).is_err());
        let mut r = requirement(2, NumericalSource::Model, 20.0);
        let p = resolve(&registry, &facts(), &[t.clone()], &[r.clone()], &strict).unwrap();
        assert_eq!(p.targets[0].nominal, 20.0);
        assert_eq!(p.targets[0].coordinate_scale, 1.0);
        r.declaration.scaling_factor = Some(0.05);
        assert!(resolve(&registry, &facts(), &[t], &[r], &strict).is_err());
    }
    #[test]
    fn actual_numerical_facts_check_affine_operands_and_preserve_exact_allowances() {
        let registry = standard_registry().unwrap();
        let quantity = registry
            .quantity_types()
            .find(|q| q.name.as_deref() == Some("LogFugacityCoefficient"))
            .unwrap();
        let target = TargetSpec {
            id: id(81),
            kind: NumericalTarget::Observable,
            quantity: quantity.id,
            unit: quantity.canonical_unit,
            integer: false,
            declared_tolerance: None,
        };
        let present = facts();
        let point = pse_quantity::ResolvedPhysicalContract::named(
            quantity.id,
            pse_quantity::IndexSet::default(),
            &registry,
        )
        .unwrap();
        let no_facts = pse_quantity::resolved::infer_operation(
            &pse_quantity::infer::OpRequest::Sub,
            &[point.clone(), point],
            None,
            &registry,
            &pse_quantity::infer::NoInvariantFacts,
        )
        .unwrap_err();
        assert!(matches!(
            no_facts,
            pse_quantity::QuantityError::InferencePrecondition {
                rule: "operation.unestablished_invariant",
                ..
            }
        ));
        let absent = PhysicalPreconditions::new(vec![]).unwrap();
        let mut declarations = present.declarations().to_vec();
        let operation = registry
            .operations()
            .find(|op| {
                op.opcode == pse_quantity::Opcode::Sub
                    && op.input_kinds == vec![quantity.key.kind, quantity.key.kind]
            })
            .unwrap();
        assert!(!operation.precondition_invariants.is_empty());
        for fact in &mut declarations {
            if operation.precondition_invariants.contains(&fact.id) {
                fact.requirement = pse_quantity::PhysicalRequirement::OperandQuantityContract {
                    required: registry.neutral_dimensionless().unwrap(),
                    match_shape: true,
                };
            }
        }
        let changed = PhysicalPreconditions::new(declarations).unwrap();
        let policy = NumericalPolicy::default();
        let source = resolve(
            &registry,
            &present,
            std::slice::from_ref(&target),
            &[],
            &policy,
        )
        .unwrap();
        assert_eq!(source.targets[0].budget, DEFAULT_ENGINEERING_ACCURACY);
        let difference = engineering_error_quantity(&registry, &present, &target).unwrap();
        let projections = [TargetProjection {
            source: target.id,
            source_kind: target.kind,
            target: TargetSpec {
                id: id(82),
                kind: NumericalTarget::Row,
                quantity: difference,
                ..target.clone()
            },
        }];
        let projected = project_difference(&registry, &present, &source, &projections).unwrap();
        assert_eq!(projected.targets[0].budget, source.targets[0].budget);
        assert_eq!(projected.targets[0].absolute, source.targets[0].absolute);
        let output =
            operational_output_context(&registry, &present, &target, &policy, &[]).unwrap();
        assert_eq!(output.budget, DEFAULT_ENGINEERING_ACCURACY);
        fn refuses<T: std::fmt::Debug>(result: Result<T, MathError>) {
            assert!(matches!(
                result.unwrap_err(),
                MathError::Quantity(pse_quantity::QuantityError::InferencePrecondition {
                    rule: "operation.physical_prerequisite",
                    ..
                })
            ));
        }
        for missing in [&absent, &changed] {
            refuses(resolve(
                &registry,
                missing,
                std::slice::from_ref(&target),
                &[],
                &policy,
            ));
            refuses(operational_output_context(
                &registry,
                missing,
                &target,
                &policy,
                &[],
            ));
            refuses(project_difference(
                &registry,
                missing,
                &source,
                &projections,
            ));
        }
        let policy = NumericalPolicy {
            engineering_rules: vec![EngineeringRule {
                rule_id: id(83).into(),
                quantity_id: difference.as_id(),
                unit_id: target.unit.as_id(),
                physical_allowance: Some(2e-7),
                relative_fraction: None,
                provenance: "authored physical allowance control".into(),
            }],
            engineering_scales: vec![EngineeringScale {
                scale_id: id(84).into(),
                model_id: Some(id(90).into()),
                case_id: None,
                instance_id: None,
                fit_id: None,
                target_id: target.id,
                target_kind: target.kind,
                quantity_id: target.quantity.as_id(),
                unit_id: target.unit.as_id(),
                source: NumericalSource::Model,
                priority: 0,
                kind: EngineeringScaleKind::RangeWidth,
                value: 0.0,
                provenance: "authored zero range control".into(),
            }],
            ..Default::default()
        };
        let exact = resolve(
            &registry,
            &present,
            std::slice::from_ref(&target),
            &[],
            &policy,
        )
        .unwrap();
        assert_eq!(exact.targets[0].budget, 2e-7);
        assert_eq!(
            exact.targets[0]
                .engineering
                .as_ref()
                .unwrap()
                .physical_allowance,
            Some(2e-7)
        );
        assert_eq!(
            operational_output_context(&registry, &present, &target, &policy, &[])
                .unwrap()
                .budget,
            2e-7
        );
        for missing in [&absent, &changed] {
            refuses(resolve(
                &registry,
                missing,
                std::slice::from_ref(&target),
                &[],
                &policy,
            ));
        }
    }
}
