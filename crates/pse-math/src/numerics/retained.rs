// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Process-local resolution premises; these never replace scientific policy framing.
use super::*;
use crate::SharedAllocation;
use pse_ids::ContentHash;
use pse_model::HeapUsage;
use std::hash::{Hash, Hasher};

/// Complete selected resolution inputs and their original immutable physical owners.
#[derive(Clone, Debug)]
pub struct ResolutionPremises {
    registry: SharedAllocation<QuantityRegistry>,
    preconditions: SharedAllocation<PhysicalPreconditions>,
    targets: Vec<TargetSpec>,
    declarations: Vec<SourcedRequirement>,
    policy: Vec<u8>,
    policy_index: ContentHash,
    owner: Option<std::sync::Arc<dyn crate::AllocationOwner>>,
}
impl ResolutionPremises {
    /// Construct only after the caller admits the input/key construction allowance.
    pub fn new(
        registry: SharedAllocation<QuantityRegistry>,
        preconditions: SharedAllocation<PhysicalPreconditions>,
        targets: &[TargetSpec],
        declarations: &[SourcedRequirement],
        policy: &NumericalPolicy,
    ) -> Result<Self, MathError> {
        policy
            .validate()
            .map_err(|e| MathError::Contract(e.to_string()))?;
        // JSON preserves finite IEEE magnitudes and signed zero. Refuse nonfinite
        // requirement fields first, so its null representation cannot erase a value.
        if policy.requirements.iter().any(|r| {
            [
                r.nominal,
                r.scaling_factor,
                r.absolute_tolerance,
                r.relative_tolerance,
            ]
            .into_iter()
            .flatten()
            .any(|v| !v.is_finite())
        }) {
            return Err(MathError::Contract(
                "nonfinite numerical policy requirement".into(),
            ));
        }
        let mut encoded = Vec::with_capacity(policy_encoding_extent(policy));
        serde_json::to_writer(&mut encoded, policy)
            .map_err(|e| MathError::Contract(e.to_string()))?;
        Ok(Self {
            registry,
            preconditions,
            targets: targets.to_vec(),
            declarations: declarations.to_vec(),
            policy: encoded,
            policy_index: policy.key(),
            owner: None,
        })
    }
    /// Anchor the key's own retained allocation independently of a cached value.
    pub fn with_owner(mut self, owner: std::sync::Arc<dyn crate::AllocationOwner>) -> Self {
        self.owner = Some(crate::retain_allocation_owner(self.owner.take(), owner));
        self
    }
    /// Key/validation construction capacity, independently admitted before a cache lookup.
    pub fn allocation_bound(
        targets: &[TargetSpec],
        declarations: &[SourcedRequirement],
        policy: &NumericalPolicy,
    ) -> usize {
        let declarations = declarations.iter().fold(0usize, |n, r| {
            n.saturating_add(size_of::<SourcedRequirement>())
                .saturating_add(r.declaration.heap_bytes())
        });
        4096usize
            .saturating_add(
                targets
                    .len()
                    .saturating_mul(size_of::<TargetSpec>() + 2 * size_of::<usize>()),
            )
            .saturating_add(declarations.saturating_mul(2))
            .saturating_add(policy_extent(policy).saturating_mul(2))
            .saturating_add(policy_encoding_extent(policy))
    }
    /// Owned key storage; original shared physical allocations are counted by their owner.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            .saturating_add(128)
            .saturating_add(self.policy.capacity())
            .saturating_add(
                self.targets
                    .capacity()
                    .saturating_mul(size_of::<TargetSpec>()),
            )
            .saturating_add(
                self.declarations
                    .capacity()
                    .saturating_mul(size_of::<SourcedRequirement>()),
            )
            .saturating_add(
                self.declarations
                    .iter()
                    .map(|r| r.declaration.heap_bytes())
                    .fold(0usize, usize::saturating_add),
            )
    }
}
impl PartialEq for ResolutionPremises {
    fn eq(&self, other: &Self) -> bool {
        SharedAllocation::ptr_eq(&self.registry, &other.registry)
            && SharedAllocation::ptr_eq(&self.preconditions, &other.preconditions)
            && self.preconditions == other.preconditions
            && self.policy == other.policy
            && self.targets.len() == other.targets.len()
            && self.targets.iter().zip(&other.targets).all(|(a, b)| {
                a.id == b.id
                    && a.kind == b.kind
                    && a.quantity == b.quantity
                    && a.unit == b.unit
                    && a.integer == b.integer
                    && a.declared_tolerance.map(f64::to_bits)
                        == b.declared_tolerance.map(f64::to_bits)
            })
            && self.declarations.len() == other.declarations.len()
            && self
                .declarations
                .iter()
                .zip(&other.declarations)
                .all(|(a, b)| {
                    a.source == b.source && requirement_eq(&a.declaration, &b.declaration)
                })
    }
}
impl Eq for ResolutionPremises {}
impl Hash for ResolutionPremises {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.registry.allocation_identity().hash(state);
        self.preconditions.allocation_identity().hash(state);
        self.policy_index.hash(state);
        // Full equality above remains authoritative, including deliberately colliding
        // input identities with changed magnitudes, provenance or physical contracts.
        self.targets.len().hash(state);
        for target in &self.targets {
            target.id.hash(state);
            target.kind.hash(state);
        }
        self.declarations.len().hash(state);
        for declaration in &self.declarations {
            declaration.source.hash(state);
            declaration.declaration.requirement_id.hash(state);
        }
    }
}
// Ordered struct/row encoding retains every field, including provenance and scope.
// This is process-local equality storage, never a new scientific semantic frame.
fn policy_encoding_extent(policy: &NumericalPolicy) -> usize {
    struct Extent(usize);
    impl std::io::Write for Extent {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_add(bytes.len())
                .ok_or_else(|| std::io::Error::other("numerical policy encoding extent"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut extent = Extent(0);
    if serde_json::to_writer(&mut extent, policy).is_err() {
        usize::MAX
    } else {
        extent.0
    }
}
fn requirement_eq(a: &NumericalRequirement, b: &NumericalRequirement) -> bool {
    a.requirement_id == b.requirement_id
        && a.model_id == b.model_id
        && a.case_id == b.case_id
        && a.instance_id == b.instance_id
        && a.fit_id == b.fit_id
        && a.target_id == b.target_id
        && a.target_kind == b.target_kind
        && a.nominal.map(f64::to_bits) == b.nominal.map(f64::to_bits)
        && a.scaling_factor.map(f64::to_bits) == b.scaling_factor.map(f64::to_bits)
        && a.absolute_tolerance.map(f64::to_bits) == b.absolute_tolerance.map(f64::to_bits)
        && a.relative_tolerance.map(f64::to_bits) == b.relative_tolerance.map(f64::to_bits)
        && a.shared_engineering_allowance == b.shared_engineering_allowance
        && a.engineering_rule_id == b.engineering_rule_id
        && a.unit_id == b.unit_id
        && a.coordinates == b.coordinates
        && a.priority == b.priority
        && a.required == b.required
        && a.provenance == b.provenance
}
fn policy_extent(policy: &NumericalPolicy) -> usize {
    size_of::<NumericalPolicy>()
        .saturating_add(policy.requirements.heap_bytes())
        .saturating_add(policy.goals.heap_bytes())
        .saturating_add(policy.engineering_rules.heap_bytes())
        .saturating_add(policy.engineering_scales.heap_bytes())
}
/// Actual logical owned policy storage, excluding already owned physical inventories.
pub fn resolved_extent(policy: &ResolvedNumericalPolicy) -> usize {
    let mut bytes = size_of::<ResolvedNumericalPolicy>()
        .saturating_add(policy_extent(&policy.policy))
        .saturating_add(
            policy
                .targets
                .capacity()
                .saturating_mul(size_of::<ResolvedTarget>()),
        );
    for target in &policy.targets {
        bytes = bytes
            .saturating_add(
                target
                    .provenance
                    .capacity()
                    .saturating_mul(size_of::<NumericalProvenance>()),
            )
            .saturating_add(
                target
                    .provenance
                    .iter()
                    .map(|p| p.description.capacity())
                    .fold(0usize, usize::saturating_add),
            )
            .saturating_add(target.engineering.as_ref().map_or(0, HeapUsage::heap_bytes));
    }
    bytes.saturating_add(512)
}
/// Conservative peak for key, validation/buckets, policy cloning and all field provenance.
/// Overflow saturates, so finite runtime admission refuses before construction.
pub fn resolution_allocation_bound(
    targets: &[TargetSpec],
    declarations: &[SourcedRequirement],
    policy: &NumericalPolicy,
) -> usize {
    let declaration_bytes = declarations.iter().fold(0usize, |n, r| {
        n.saturating_add(size_of::<SourcedRequirement>())
            .saturating_add(r.declaration.heap_bytes())
    });
    let requirements = declarations.len().saturating_add(policy.requirements.len());
    4096usize
        .saturating_add(policy_extent(policy).saturating_mul(8))
        .saturating_add(policy_encoding_extent(policy))
        .saturating_add(declaration_bytes.saturating_mul(8))
        .saturating_add(targets.len().saturating_mul(
            size_of::<TargetSpec>()
                + size_of::<ResolvedTarget>()
                + 6 * size_of::<NumericalProvenance>()
                + 1024,
        ))
        .saturating_add(requirements.saturating_mul(8 * (size_of::<NumericalProvenance>() + 128)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_quantity::standard::{ids, standard_registry};
    use std::sync::Arc;
    fn target(n: u8) -> TargetSpec {
        TargetSpec {
            id: SemanticId::from_bytes([n; 16]),
            kind: NumericalTarget::Variable,
            quantity: ids::quantity("temperature.point"),
            unit: ids::unit("K"),
            integer: false,
            declared_tolerance: None,
        }
    }
    fn requirement() -> SourcedRequirement {
        SourcedRequirement {
            source: NumericalSource::Model,
            declaration: NumericalRequirement {
                requirement_id: SemanticId::from_bytes([3; 16]),
                model_id: None,
                case_id: None,
                instance_id: None,
                fit_id: None,
                target_id: target(1).id,
                target_kind: NumericalTarget::Variable,
                nominal: Some(300.),
                scaling_factor: None,
                absolute_tolerance: Some(0.5),
                relative_tolerance: Some(0.01),
                shared_engineering_allowance: None,
                engineering_rule_id: None,
                unit_id: Some(ids::unit("K").as_id()),
                coordinates: NumericalCoordinates::Physical,
                priority: 0,
                required: true,
                provenance: "independent original declaration".into(),
            },
        }
    }
    #[test]
    fn complete_resolution_premises_refuse_changed_inputs_even_when_hashes_collide() {
        let registry = SharedAllocation::from(Arc::new(standard_registry().unwrap()));
        let facts = SharedAllocation::from(Arc::new(
            PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions()).unwrap(),
        ));
        let policy = NumericalPolicy::default();
        let make = |target: TargetSpec, declaration: SourcedRequirement| {
            ResolutionPremises::new(
                registry.clone(),
                facts.clone(),
                &[target],
                &[declaration],
                &policy,
            )
            .unwrap()
        };
        let base = make(target(1), requirement());
        let mut declarations = Vec::new();
        let mut changed = requirement();
        changed.declaration.nominal = Some(301.);
        declarations.push(changed);
        let mut changed = requirement();
        changed.declaration.provenance.push('!');
        declarations.push(changed);
        let mut changed = requirement();
        changed.declaration.priority = 2;
        declarations.push(changed);
        let mut changed = requirement();
        changed.source = NumericalSource::Case;
        declarations.push(changed);
        for declaration in declarations {
            let candidate = make(target(1), declaration);
            if candidate.declarations[0].source == base.declarations[0].source {
                let hash = |value: &ResolutionPremises| {
                    let mut h = std::hash::DefaultHasher::new();
                    value.hash(&mut h);
                    h.finish()
                };
                assert_eq!(hash(&base), hash(&candidate));
            }
            assert_ne!(base, candidate);
        }
        let mut unit = target(1);
        unit.unit = ids::unit("degC");
        assert_ne!(base, make(unit, requirement()));
        let mut integer = target(1);
        integer.integer = true;
        assert_ne!(base, make(integer, requirement()));
        let mut zero = target(1);
        zero.declared_tolerance = Some(0.);
        let mut minus_zero = zero.clone();
        minus_zero.declared_tolerance = Some(-0.);
        assert_ne!(make(zero, requirement()), make(minus_zero, requirement()));
        let mut positive = requirement();
        positive.declaration.relative_tolerance = Some(0.0);
        let mut negative = positive.clone();
        negative.declaration.relative_tolerance = Some(-0.0);
        assert_ne!(make(target(1), positive), make(target(1), negative));
        let mut nan = requirement();
        nan.declaration.relative_tolerance = Some(f64::from_bits(0x7ff8_0000_0000_0001));
        let nan = make(target(1), nan);
        assert_eq!(nan, nan.clone());
        let mut other_nan = requirement();
        other_nan.declaration.relative_tolerance = Some(f64::from_bits(0x7ff8_0000_0000_0002));
        assert_ne!(nan, make(target(1), other_nan));
        // A forced digest collision cannot substitute for complete policy equality.
        let mut changed_policy = policy.clone();
        changed_policy.strict_nominals = true;
        let mut changed = ResolutionPremises::new(
            registry.clone(),
            facts.clone(),
            &[target(1)],
            &[requirement()],
            &changed_policy,
        )
        .unwrap();
        changed.policy_index = base.policy_index;
        assert_ne!(base, changed);
        let mut positive_policy = policy.clone();
        positive_policy.mip_absolute_gap = 0.0;
        let mut negative_policy = positive_policy.clone();
        negative_policy.mip_absolute_gap = -0.0;
        let positive =
            ResolutionPremises::new(registry.clone(), facts.clone(), &[], &[], &positive_policy)
                .unwrap();
        let negative =
            ResolutionPremises::new(registry.clone(), facts.clone(), &[], &[], &negative_policy)
                .unwrap();
        assert_ne!(positive, negative);
        let mut invalid = policy.clone();
        let mut invalid_requirement = requirement().declaration;
        invalid_requirement.nominal = Some(f64::NAN);
        invalid.requirements.push(invalid_requirement);
        assert!(
            ResolutionPremises::new(registry.clone(), facts.clone(), &[], &[], &invalid).is_err()
        );
        assert!(
            base.retained_bytes()
                <= ResolutionPremises::allocation_bound(&[target(1)], &[requirement()], &policy)
        );
        let foreign = ResolutionPremises::new(
            SharedAllocation::from(Arc::new(standard_registry().unwrap())),
            facts.clone(),
            &[target(1)],
            &[requirement()],
            &policy,
        )
        .unwrap();
        assert_ne!(base, foreign);
        let foreign = ResolutionPremises::new(
            registry,
            SharedAllocation::from(Arc::new(facts.as_ref().clone())),
            &[target(1)],
            &[requirement()],
            &policy,
        )
        .unwrap();
        assert_ne!(base, foreign);
    }
    #[test]
    fn indexed_resolution_keeps_original_order_access_conflicts_and_duplicate_refusal() {
        let registry = standard_registry().unwrap();
        let facts =
            PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions()).unwrap();
        let policy = NumericalPolicy::default();
        let first = resolve(
            &registry,
            &facts,
            &[target(2), target(1)],
            &[requirement()],
            &policy,
        )
        .unwrap();
        let second = resolve(
            &registry,
            &facts,
            &[target(1), target(2)],
            &[requirement()],
            &policy,
        )
        .unwrap();
        assert_eq!(first.key, second.key);
        assert_eq!(first.targets, second.targets);
        let access = TargetAccess::new(&first).unwrap();
        let selected = access
            .ordered(
                NumericalTarget::Variable,
                &[target(2).id, target(1).id, target(2).id],
            )
            .unwrap();
        assert_eq!(
            selected.iter().map(|t| t.id).collect::<Vec<_>>(),
            [target(2).id, target(1).id, target(2).id]
        );
        let mut conflict = requirement();
        conflict.declaration.requirement_id = target(4).id;
        conflict.declaration.nominal = Some(301.);
        assert!(
            resolve(
                &registry,
                &facts,
                &[target(1)],
                &[requirement(), conflict],
                &policy
            )
            .is_err()
        );
        assert!(resolve(&registry, &facts, &[target(1), target(1)], &[], &policy).is_err());
        assert!(resolve(&registry, &facts, &[target(2)], &[requirement()], &policy).is_err());
        assert!(
            resolved_extent(&first)
                < resolution_allocation_bound(&[target(1), target(2)], &[requirement()], &policy)
        );
    }
}
