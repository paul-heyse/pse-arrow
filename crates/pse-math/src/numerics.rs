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
    let mut key = FramedHasher::new(pse_ids::Frame::NumericalResolvedV1);
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
        let absolute = candidates
            .iter()
            .filter_map(|r| r.declaration.absolute_tolerance.map(|v| (*r, v)))
            .map(|(r, v)| {
                Ok((
                    r,
                    v * if r.declaration.coordinates == NumericalCoordinates::Normalized {
                        coordinate_scale
                    } else {
                        magnitude(r.declaration.unit_id.map_or(target.unit, Into::into))?
                    },
                ))
            })
            .collect::<Result<Vec<_>, MathError>>()?;
        let absolute = choose(
            NumericalProvenanceField::AbsoluteTolerance,
            absolute,
            target.declared_tolerance.unwrap_or(1e-8 * nominal),
            if target.declared_tolerance.is_some() {
                NumericalSource::Model
            } else {
                NumericalSource::CanonicalFallback
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
                unit_id: Some(ids::unit("degC").as_id()),
                coordinates: NumericalCoordinates::Physical,
                priority: 0,
                required: true,
                provenance: "authored temperature magnitude".into(),
            },
        }
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
