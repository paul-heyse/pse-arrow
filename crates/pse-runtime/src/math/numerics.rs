// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Charged immutable numerical resolution, with optional service-local retention.
use super::*;
use pse_math::{
    SharedAllocation,
    numerics::{ResolutionPremises, SourcedRequirement, TargetSpec},
};
use pse_model::numerics::{NumericalPolicy, ResolvedNumericalPolicy};
use pse_quantity::{PhysicalPreconditions, QuantityRegistry};
use std::sync::Weak;

/// The actual pool survives escaped preparations; caches and pressure remain optional.
#[derive(Clone)]
pub(crate) struct PolicyResolutionOwner {
    pool: Arc<dyn MemoryPool>,
    pressure: Weak<retention::Pressure>,
    service: Weak<MathService>,
    ceiling: usize,
}
impl std::fmt::Debug for PolicyResolutionOwner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PolicyResolutionOwner")
            .field("ceiling", &self.ceiling)
            .finish_non_exhaustive()
    }
}
#[derive(Clone)]
pub(super) struct RetainedPolicy {
    pub value: SharedAllocation<ResolvedNumericalPolicy>,
    pub bytes: usize,
}
impl PolicyResolutionOwner {
    fn reserve(
        &self,
        bytes: usize,
    ) -> Result<Arc<pse_columnar::AllocationLease>, MathRuntimeError> {
        if bytes > self.ceiling {
            return Err(MathRuntimeError::Limit(
                "numerical resolution construction capacity",
            ));
        }
        let reservation = MemoryConsumer::new("math:resolved-numerics").register(&self.pool);
        if let Some(pressure) = self.pressure.upgrade() {
            pressure.try_grow(&reservation, bytes)?;
        } else {
            reservation.try_grow(bytes)?;
        }
        Ok(pse_columnar::AllocationLease::new(reservation))
    }
    /// Cache availability never changes resolution, physical admission or allocation ownership.
    pub(crate) fn resolve(
        &self,
        registry: SharedAllocation<QuantityRegistry>,
        preconditions: SharedAllocation<PhysicalPreconditions>,
        targets: &[TargetSpec],
        declarations: &[SourcedRequirement],
        policy: &NumericalPolicy,
    ) -> Result<SharedAllocation<ResolvedNumericalPolicy>, MathRuntimeError> {
        let service = self.service.upgrade();
        let generation = service.as_ref().map(|s| s.modeling_cache.generation());
        let key_bound = ResolutionPremises::allocation_bound(targets, declarations, policy);
        let key_lease = self.reserve(key_bound)?;
        let premises = ResolutionPremises::new(
            registry.clone(),
            preconditions.clone(),
            targets,
            declarations,
            policy,
        )?;
        let key_lease = retained_lease(key_lease, premises.retained_bytes())?;
        let premises = Arc::new(premises.with_owner(key_lease));
        if let Some(value) = service
            .as_ref()
            .and_then(|service| service.modeling_cache.numerics(&premises))
        {
            return Ok(value.value);
        }
        let bound = pse_math::numerics::resolution_allocation_bound(targets, declarations, policy);
        if bound > self.ceiling {
            return Err(MathRuntimeError::Limit(
                "numerical resolution construction capacity",
            ));
        }
        let lease = self.reserve(bound.checked_sub(key_bound).ok_or_else(|| {
            MathRuntimeError::Infrastructure(
                "numerical key extent exceeds resolver construction allowance".into(),
            )
        })?)?;
        let resolved =
            pse_math::numerics::resolve(&registry, &preconditions, targets, declarations, policy)?;
        let value_bytes = pse_math::numerics::resolved_extent(&resolved);
        let lease = retained_lease(lease, value_bytes)?;
        let value = SharedAllocation::from(Arc::new(resolved)).with_owner(Arc::new((
            lease,
            premises.clone(),
            registry,
            preconditions,
        )));
        if let (Some(service), Some(generation)) = (service, generation) {
            service.modeling_cache.retain_numerics(
                generation,
                premises,
                RetainedPolicy {
                    value: value.clone(),
                    bytes: value_bytes,
                },
            );
        }
        Ok(value)
    }
    pub(crate) fn derivation(
        &self,
        source: &SharedAllocation<ResolvedNumericalPolicy>,
        additional: usize,
    ) -> Result<PolicyDerivation, MathRuntimeError> {
        // Every projected target can duplicate at most the complete source provenance;
        // the policy's declarations also cover any newly resolved default targets.
        let bound = pse_math::numerics::resolved_extent(source)
            .checked_add(4096)
            .and_then(|n| n.checked_mul(additional.checked_add(4)?))
            .ok_or(MathRuntimeError::Limit("numerical projection extent"))?;
        Ok(PolicyDerivation {
            lease: self.reserve(bound)?,
            source: source.clone(),
        })
    }
}
pub(crate) struct PolicyDerivation {
    lease: Arc<pse_columnar::AllocationLease>,
    source: SharedAllocation<ResolvedNumericalPolicy>,
}
impl PolicyDerivation {
    pub(crate) fn finish(
        self,
        resolved: ResolvedNumericalPolicy,
    ) -> Result<SharedAllocation<ResolvedNumericalPolicy>, MathRuntimeError> {
        let lease = retained_lease(self.lease, pse_math::numerics::resolved_extent(&resolved))?;
        Ok(SharedAllocation::from(Arc::new(resolved)).with_owner(Arc::new((lease, self.source))))
    }
}
fn retained_lease(
    lease: Arc<pse_columnar::AllocationLease>,
    bytes: usize,
) -> Result<Arc<pse_columnar::AllocationLease>, MathRuntimeError> {
    let mut leases = lease.partition(&[bytes]).map_err(|_| {
        MathRuntimeError::Infrastructure(
            "numerical resolution extent exceeded its admitted construction allowance".into(),
        )
    })?;
    leases.pop().ok_or_else(|| {
        MathRuntimeError::Infrastructure("numerical resolution lease missing".into())
    })
}
impl MathService {
    pub(crate) fn policy_resolution_owner(self: &Arc<Self>) -> PolicyResolutionOwner {
        PolicyResolutionOwner {
            pool: self.pool.clone(),
            pressure: Arc::downgrade(&self.pressure),
            service: Arc::downgrade(self),
            ceiling: self.policy.worker_bytes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_model::generated::enums::NumericalTarget;
    use pse_quantity::standard::{ids, standard_registry};
    fn physical() -> (
        SharedAllocation<QuantityRegistry>,
        SharedAllocation<PhysicalPreconditions>,
    ) {
        (
            Arc::new(standard_registry().unwrap()).into(),
            Arc::new(
                PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
                    .unwrap(),
            )
            .into(),
        )
    }
    fn target(n: u8) -> TargetSpec {
        TargetSpec {
            id: pse_ids::SemanticId::from_bytes([n; 16]),
            kind: NumericalTarget::Variable,
            quantity: ids::quantity("temperature.point"),
            unit: ids::unit("K"),
            integer: false,
            declared_tolerance: None,
        }
    }
    #[test]
    fn complete_policy_memo_shares_original_products_and_invalidates_physical_and_request_changes()
    {
        let service = super::super::tests::service();
        let owner = service.policy_resolution_owner();
        let (registry, facts) = physical();
        let policy = NumericalPolicy::default();
        let first = owner
            .resolve(
                registry.clone(),
                facts.clone(),
                &[target(2), target(1)],
                &[],
                &policy,
            )
            .unwrap();
        let same = owner
            .resolve(
                registry.clone(),
                facts.clone(),
                &[target(2), target(1)],
                &[],
                &policy,
            )
            .unwrap();
        assert!(SharedAllocation::ptr_eq(&first, &same));
        let original =
            pse_math::numerics::resolve(&registry, &facts, &[target(2), target(1)], &[], &policy)
                .unwrap();
        assert_eq!(first.key, original.key);
        assert_eq!(first.targets, original.targets);
        let ordered = owner
            .resolve(
                registry.clone(),
                facts.clone(),
                &[target(1), target(2)],
                &[],
                &policy,
            )
            .unwrap();
        assert!(!SharedAllocation::ptr_eq(&first, &ordered));
        assert_eq!(first.key, ordered.key);
        let (foreign, _) = physical();
        let foreign = owner
            .resolve(
                foreign,
                facts.clone(),
                &[target(2), target(1)],
                &[],
                &policy,
            )
            .unwrap();
        assert!(!SharedAllocation::ptr_eq(&first, &foreign));
        assert_eq!(first.key, foreign.key);
        let mut changed = policy.clone();
        changed.integrality *= 0.5;
        let changed = owner
            .resolve(
                registry.clone(),
                facts.clone(),
                &[target(2), target(1)],
                &[],
                &changed,
            )
            .unwrap();
        assert!(!SharedAllocation::ptr_eq(&first, &changed));
        assert_ne!(first.key, changed.key);
        let mut unit = target(1);
        unit.unit = ids::unit("degC");
        let changed = owner
            .resolve(registry.clone(), facts.clone(), &[unit], &[], &policy)
            .unwrap();
        assert_ne!(first.key, changed.key);
        let quantity = registry
            .quantity_types()
            .find(|q| q.name.as_deref() == Some("LogFugacityCoefficient"))
            .unwrap();
        let log = TargetSpec {
            quantity: quantity.id,
            unit: quantity.canonical_unit,
            ..target(3)
        };
        owner
            .resolve(
                registry.clone(),
                facts,
                std::slice::from_ref(&log),
                &[],
                &policy,
            )
            .unwrap();
        assert!(
            owner
                .resolve(
                    registry,
                    Arc::new(PhysicalPreconditions::new(vec![]).unwrap()).into(),
                    &[log],
                    &[],
                    &policy
                )
                .is_err()
        );
    }
    #[test]
    fn escaped_policy_and_derivation_keep_actual_charge_and_physical_owner_after_cache_and_service_drop()
     {
        let service = super::super::tests::service();
        let owner = service.policy_resolution_owner();
        let pool = service.pool.clone();
        let baseline = pool.reserved();
        let anchor = Arc::new(());
        let weak = Arc::downgrade(&anchor);
        let (registry, facts) = physical();
        let registry = registry.with_owner(anchor.clone());
        drop(anchor);
        let source = owner
            .resolve(
                registry.clone(),
                facts.clone(),
                &[target(1)],
                &[],
                &NumericalPolicy::default(),
            )
            .unwrap();
        let derivation = owner.derivation(&source, 1).unwrap();
        let projected = pse_math::numerics::project(
            &registry,
            &source,
            &[pse_math::numerics::TargetProjection {
                source: target(1).id,
                source_kind: NumericalTarget::Variable,
                target: target(2),
            }],
        )
        .unwrap();
        let derived = derivation.finish(projected).unwrap();
        drop(source);
        drop(registry);
        drop(facts);
        service.pressure.clear();
        assert!(pool.reserved() > baseline);
        assert!(weak.upgrade().is_some());
        let service_weak = Arc::downgrade(&service);
        drop(service);
        assert!(service_weak.upgrade().is_none());
        assert!(pool.reserved() > baseline);
        assert!(weak.upgrade().is_some());
        drop(derived);
        assert_eq!(pool.reserved(), baseline);
        assert!(weak.upgrade().is_none());
        let (registry, facts) = physical();
        let detached = owner
            .resolve(
                registry,
                facts,
                &[target(1)],
                &[],
                &NumericalPolicy::default(),
            )
            .unwrap();
        let clone = detached.clone();
        drop(detached);
        assert!(pool.reserved() > baseline);
        drop(clone);
        assert_eq!(pool.reserved(), baseline);
    }
    #[test]
    fn changed_evaluated_requirements_refuse_value_rebind_reuse() {
        let service = super::super::tests::service();
        let owner = service.policy_resolution_owner();
        let (registry, facts) = physical();
        let policy = NumericalPolicy::default();
        let mut selected = target(1);
        selected.declared_tolerance = Some(0.5);
        let first = owner
            .resolve(
                registry.clone(),
                facts.clone(),
                std::slice::from_ref(&selected),
                &[],
                &policy,
            )
            .unwrap();
        // A free start/value rebind does not change these evaluated policy premises.
        let same = owner
            .resolve(
                registry.clone(),
                facts.clone(),
                std::slice::from_ref(&selected),
                &[],
                &policy,
            )
            .unwrap();
        assert!(SharedAllocation::ptr_eq(&first, &same));
        // A rebound declaration which changes its physical allowance is a new premise.
        selected.declared_tolerance = Some(0.25);
        let changed = owner
            .resolve(registry, facts, &[selected], &[], &policy)
            .unwrap();
        assert!(!SharedAllocation::ptr_eq(&first, &changed));
        assert_ne!(first.key, changed.key);
    }
}
