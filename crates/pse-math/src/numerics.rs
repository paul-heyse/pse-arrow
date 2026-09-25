// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Resolve physical magnitudes and source precedence before numerical execution.
use crate::MathError;
use pse_ids::{FramedHasher, SemanticId};
use pse_model::{
    SemanticFrame,
    generated::enums::{NumericalCoordinates, NumericalSource, NumericalTarget},
    numerics::*,
};
use pse_quantity::{QuantityRegistry, QuantityTypeId, UnitId};

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
fn rank(source: NumericalSource) -> u8 {
    match source {
        NumericalSource::Analysis => 5,
        NumericalSource::Case => 4,
        NumericalSource::Model => 3,
        NumericalSource::PropertyDefault => 2,
        NumericalSource::QuantityNominal => 1,
        NumericalSource::CanonicalFallback => 0,
    }
}
fn failure(id: SemanticId, message: &str) -> MathError {
    MathError::Contract(format!("numerical target {id}: {message}"))
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
    let mut key = FramedHasher::new("pse.numerical.resolved.v1");
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
        let mut choose = |field: &'static str,
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
                            "conflicting {field}: {} and {}",
                            first.declaration.requirement_id, candidate.declaration.requirement_id
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
            "nominal",
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
            "absolute_tolerance",
            absolute,
            target.declared_tolerance.unwrap_or(1e-8 * nominal),
            if target.declared_tolerance.is_some() {
                NumericalSource::Model
            } else {
                NumericalSource::CanonicalFallback
            },
        )?;
        let relative = choose(
            "relative_tolerance",
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
                        p.field == "nominal"
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
            key.str(p.field)
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
                        p.field == "nominal"
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
    /// Complete source coordinates, including fixed variables whose requirements remain meaningful.
    pub fn numerical_targets(
        &self,
        registry: &QuantityRegistry,
    ) -> Result<Vec<TargetSpec>, MathError> {
        self.structure().numerical_targets(registry)
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
                model_id: id(90),
                case_id: None,
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
