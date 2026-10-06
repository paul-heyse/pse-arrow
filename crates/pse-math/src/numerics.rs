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
use pse_quantity::{QuantityRegistry, QuantityTypeId, UnitId};
use std::collections::BTreeMap;

/// Exact admitted coordinate; declared representation is retained through normalization.
#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
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
    source: &ResolvedNumericalPolicy,
    projections: &[TargetProjection],
) -> Result<ResolvedNumericalPolicy, MathError> {
    project_with(registry, source, projections, ProjectionMeaning::Difference)
}
#[derive(Clone, Copy)]
enum ProjectionMeaning {
    Coordinate,
    Difference,
}
fn project_with(
    registry: &QuantityRegistry,
    source: &ResolvedNumericalPolicy,
    projections: &[TargetProjection],
    meaning: ProjectionMeaning,
) -> Result<ResolvedNumericalPolicy, MathError> {
    let frame = match meaning {
        ProjectionMeaning::Coordinate => pse_ids::Frame::NumericalProjectionV2,
        ProjectionMeaning::Difference => pse_ids::Frame::NumericalDifferenceProjectionV1,
    };
    let mut key = FramedHasher::new(frame);
    key.hash(&source.key);
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
        let original = source
            .targets
            .iter()
            .find(|t| t.id == projection.source && t.kind == projection.source_kind)
            .ok_or_else(|| failure(projection.source, "projection source was not resolved"))?;
        let projected_quantity = match meaning {
            ProjectionMeaning::Coordinate => original.quantity.into(),
            ProjectionMeaning::Difference => {
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
                    &pse_quantity::infer::NoInvariantFacts,
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
            if !engineering.budget.is_finite()
                || engineering.budget <= 0.0
                || engineering
                    .physical_allowance
                    .is_some_and(|v| !v.is_finite() || v <= 0.0)
                || engineering
                    .characteristic
                    .is_some_and(|v| !v.is_finite() || v < 0.0)
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
    target: &TargetSpec,
    unit: UnitId,
) -> Result<f64, MathError> {
    let difference = engineering_error_quantity(registry, target)?;
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
                &pse_quantity::infer::NoInvariantFacts,
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
    target: &TargetSpec,
    policy: &NumericalPolicy,
) -> Result<EngineeringContext, MathError> {
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
    engineering_default(registry, target, policy, selected.as_ref(), true)
}

fn engineering_default(
    registry: &QuantityRegistry,
    target: &TargetSpec,
    policy: &NumericalPolicy,
    selected: Option<&SourcedRequirement>,
    enforce_strict: bool,
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
            engineering_error_quantity(registry, target)?,
            registry,
        )?;
        Some(rule)
    } else {
        let applicable = policy
            .engineering_rules
            .iter()
            .filter(|r| {
                engineering_error_quantity(registry, target).is_ok_and(|quantity| {
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
            let magnitude = s.value * engineering_magnitude(registry, target, s.unit_id.into())?;
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
    let budget = if canonical_fallback {
        if enforce_strict && policy.strict_engineering_context {
            return Err(failure(
                target.id,
                "strict engineering context forbids canonical fallback",
            ));
        }
        let canonical = registry.quantity_type(target.quantity)?.canonical_unit;
        DEFAULT_ENGINEERING_ACCURACY * engineering_magnitude(registry, target, canonical)?
    } else {
        budget
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
                selected.map_or(
                    if rule.is_some() {
                        NumericalSource::PropertyDefault
                    } else {
                        NumericalSource::CanonicalFallback
                    },
                    |r| r.source,
                )
            },
            |(s, _, _)| s.source,
        ),
        budget,
        canonical_fallback,
        limitation: if canonical_fallback {
            "No positive admitted engineering allowance; canonical magnitude fallback is not output-error evidence".into()
        } else {
            String::new()
        },
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
    targets: &[TargetSpec],
    declarations: &[SourcedRequirement],
    policy: &NumericalPolicy,
) -> Result<ResolvedNumericalPolicy, MathError> {
    policy
        .validate()
        .map_err(|e| MathError::Contract(e.to_string()))?;
    let mut target_ids = std::collections::BTreeSet::new();
    for t in targets {
        if !target_ids.insert((t.id, t.kind.as_str())) {
            return Err(failure(t.id, "duplicate numerical target"));
        }
    }
    for scale in &policy.engineering_scales {
        let target = targets
            .iter()
            .find(|t| t.id == scale.target_id && t.kind == scale.target_kind)
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
        engineering_magnitude(registry, target, scale.unit_id.into())?;
    }
    for rule in &policy.engineering_rules {
        let quantity = registry.quantity_type(rule.quantity_id.into())?;
        pse_quantity::convert_spec_for_type(
            registry.unit(rule.unit_id.into())?,
            registry.unit(quantity.canonical_unit)?,
            &quantity.key,
        )?;
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
    for candidate in &all {
        let row = &candidate.declaration;
        if !identities.insert(row.requirement_id) {
            return Err(failure(
                row.target_id,
                "duplicate numerical requirement identity",
            ));
        }
        if !targets
            .iter()
            .any(|t| t.id == row.target_id && t.kind == row.target_kind)
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
    }
    let mut resolved = Vec::with_capacity(targets.len());
    let mut key = FramedHasher::new(pse_ids::Frame::NumericalResolvedV2);
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
        let mut candidates: Vec<_> = all
            .iter()
            .filter(|r| {
                r.declaration.target_id == target.id && r.declaration.target_kind == target.kind
            })
            .collect();
        candidates.sort_by_key(|r| {
            (
                std::cmp::Reverse((rank(r.source), r.declaration.priority)),
                r.declaration.requirement_id,
            )
        });
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
                    let context =
                        engineering_default(registry, target, policy, Some(r), index == 0)?;
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
                let context = engineering_default(registry, target, policy, None, true)?;
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
        selected_engineering.frame(&mut key);
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
        let output = operational_output_context(&registry, &target(), &policy).unwrap();
        assert_eq!(output.budget, 0.1);
        let mut changed = policy.clone();
        changed.requirements[0].absolute_tolerance = Some(10.);
        changed.kkt.stationarity = 1e-12;
        changed.kkt.complementarity = 0.5;
        assert_eq!(
            operational_output_context(&registry, &target(), &changed).unwrap(),
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
            let resolved = resolve(&registry, &[coordinate], &[scale.clone()], &policy).unwrap();
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
        let resolved = resolve(&registry, &[target()], &[authored], &policy).unwrap();
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
        let resolved = resolve(&registry, &[declared], &[], &Default::default()).unwrap();
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
            let resolved = resolve(&registry, &[target()], &[conditioning], &policy).unwrap();
            assert_eq!(resolved.targets[0].nominal, nominal);
            assert_eq!(resolved.targets[0].budget, 0.3);
            assert_eq!(resolved.targets[0].relative, 0.0);
            assert_eq!(
                resolved.targets[0].engineering.as_ref().unwrap().scale_id,
                Some(id(8).into())
            );
        }
        policy.engineering_scales[0].value = 0.0;
        let resolved = resolve(&registry, &[target()], &[], &policy).unwrap();
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
        assert!(resolve(&registry, &[target()], &[conditioning.clone()], &strict).is_err());
        conditioning.declaration.absolute_tolerance = Some(0.02);
        let explicit = resolve(&registry, &[target()], &[conditioning], &strict).unwrap();
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
        let resolved = resolve(&registry, &[target()], &[], &policy).unwrap();
        assert_eq!(resolved.targets[0].budget, 0.2);
        assert_eq!(
            resolved.targets[0].engineering.as_ref().unwrap().source,
            NumericalSource::PropertyDefault
        );
        policy
            .engineering_scales
            .push(scale(11, NumericalSource::PropertyDefault, 300.0));
        assert!(resolve(&registry, &[target()], &[], &policy).is_err());
        policy.engineering_scales.pop();
        policy.engineering_rules.push(rule(12, Some(0.7)));
        assert!(resolve(&registry, &[target()], &[], &policy).is_err());
        let mut inherited = requirement(2, NumericalSource::Model, 20.0);
        inherited.declaration.shared_engineering_allowance = Some(true);
        inherited.declaration.engineering_rule_id = Some(id(12).into());
        inherited.declaration.relative_tolerance = None;
        let resolved = resolve(&registry, &[target()], &[inherited.clone()], &policy).unwrap();
        assert_eq!(resolved.targets[0].budget, 0.7);
        assert_eq!(
            resolved.targets[0].engineering.as_ref().unwrap().rule_id,
            Some(id(12).into())
        );
        inherited.declaration.shared_engineering_allowance = Some(false);
        assert!(resolve(&registry, &[target()], &[inherited.clone()], &policy).is_err());
        inherited.declaration.engineering_rule_id = None;
        let explicit = resolve(&registry, &[target()], &[inherited], &policy).unwrap();
        assert_eq!(explicit.targets[0].budget, 0.5);
        assert!(explicit.targets[0].engineering.is_none());
        policy.engineering_rules.pop();
        policy.engineering_scales[0].source = NumericalSource::DerivedNominal;
        assert!(resolve(&registry, &[target()], &[], &policy).is_err());
    }
    #[test]
    fn contextual_engineering_fraction_identity_and_full_quantity_meaning() {
        let registry = standard_registry().unwrap();
        let mut policy = NumericalPolicy {
            engineering_scales: vec![scale(8, NumericalSource::Model, 300.0)],
            engineering_rules: vec![rule(9, Some(0.1))],
            ..Default::default()
        };
        let initial = resolve(&registry, &[target()], &[], &policy).unwrap();
        policy.engineering_relative_fraction = 0.002;
        let changed = resolve(&registry, &[target()], &[], &policy).unwrap();
        assert_eq!(changed.targets[0].budget, 0.6);
        assert_ne!(initial.key, changed.key);
        policy.engineering_rules[0].relative_fraction = Some(0.001);
        let overridden = resolve(&registry, &[target()], &[], &policy).unwrap();
        assert_eq!(overridden.targets[0].budget, 0.3);
        // Equal dimensions do not admit a point quantity as a difference quantity.
        policy.engineering_scales[0].quantity_id = ids::quantity("temperature.difference").as_id();
        assert!(resolve(&registry, &[target()], &[], &policy).is_err());
        policy.engineering_scales.clear();
        policy.engineering_rules[0].quantity_id = ids::quantity("temperature.point").as_id();
        let unmatched = resolve(&registry, &[target()], &[], &policy).unwrap();
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
        let resolved = resolve(&registry, &[target()], &[], &policy).unwrap();
        assert_eq!(resolved.targets[0].budget, 0.2);
        policy.engineering_scales.reverse();
        assert_eq!(policy.key(), key);
        assert_eq!(
            resolve(&registry, &[target()], &[], &policy).unwrap().key,
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
        let fallback = resolve(&registry, &[target()], &[], &policy).unwrap();
        assert_eq!(fallback.targets[0].budget, 0.001);
        let context = fallback.targets[0].engineering.as_ref().unwrap();
        assert_eq!(context.characteristic, Some(300.0));
        assert_eq!(context.relative_fraction, 0.0);
        assert!(context.canonical_fallback);
        policy.strict_engineering_context = true;
        assert!(resolve(&registry, &[target()], &[], &policy).is_err());
        policy.engineering_rules.push(rule(9, Some(0.1)));
        let floor = resolve(&registry, &[target()], &[], &policy).unwrap();
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
        let relative = resolve(&registry, &[target()], &[], &policy).unwrap();
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
        let source = resolve(&registry, &[target()], &[], &policy).unwrap();
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
        let projected = project_difference(&registry, &source, &[projection]).unwrap();
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
        let source = resolve(&registry, &[original], &[authored], &Default::default()).unwrap();
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
        let projected = project_difference(&registry, &source, &[projection.clone()]).unwrap();
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
                &source,
                &[projection.clone(), projection.clone()]
            )
            .is_err()
        );
        projection.target.quantity = point;
        assert!(project_difference(&registry, &source, &[projection.clone()]).is_err());
        projection.target.quantity = difference;
        projection.target.unit = ids::unit("s");
        assert!(project_difference(&registry, &source, &[projection]).is_err());
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
        let defaults = resolve(&registry, &[target()], &[], &NumericalPolicy::default()).unwrap();
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
        let override_ = resolve(&registry, &[target()], &authored, &policy).unwrap();
        assert_eq!(override_.targets[0].nominal, 30.0);
        assert_ne!(override_.key, p.key);
        let conflicting = [
            requirement(2, NumericalSource::Model, 10.0),
            requirement(3, NumericalSource::Model, 20.0),
        ];
        assert!(resolve(&registry, &[target()], &conflicting, &policy).is_ok());
        assert!(
            resolve(
                &registry,
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
        let result = resolve(&registry, &[t], &[r], &Default::default()).unwrap();
        assert_eq!(result.targets[0].coordinate_scale, 1.0);
        assert_eq!(result.targets[0].budget, 1.25);
    }
    #[test]
    fn numerical_policy_recorded_fallback_and_integer_lattice() {
        let registry = standard_registry().unwrap();
        let mut t = target();
        t.integer = true;
        let p = resolve(&registry, &[t.clone()], &[], &NumericalPolicy::default()).unwrap();
        assert_eq!(p.targets[0].nominal, 1.0);
        assert_eq!(
            p.targets[0].provenance[0].source,
            NumericalSource::CanonicalFallback
        );
        let strict = NumericalPolicy {
            strict_nominals: true,
            ..NumericalPolicy::default()
        };
        assert!(resolve(&registry, &[t.clone()], &[], &strict).is_err());
        let mut r = requirement(2, NumericalSource::Model, 20.0);
        let p = resolve(&registry, &[t.clone()], &[r.clone()], &strict).unwrap();
        assert_eq!(p.targets[0].nominal, 20.0);
        assert_eq!(p.targets[0].coordinate_scale, 1.0);
        r.declaration.scaling_factor = Some(0.05);
        assert!(resolve(&registry, &[t], &[r], &strict).is_err());
    }
}
