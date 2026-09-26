// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Escaping immutable product ownership, separate from the bounded Salsa generation.
use super::*;

#[derive(Debug)]
pub(super) struct ProductOwner {
    _lease: Arc<pse_columnar::AllocationLease>,
    // Keep the exact original allocations alive: addresses below are only local
    // allocation identities, never semantic IDs or serialized compatibility keys.
    _payload: Arc<dyn pse_math::AllocationOwner>,
}
impl MathService {
    pub(super) fn shared_product(
        &self,
        key: Vec<usize>,
        payload: Arc<dyn pse_math::AllocationOwner>,
        lease: Arc<pse_columnar::AllocationLease>,
    ) -> Result<Arc<ProductOwner>, MathRuntimeError> {
        let mut products = self.products.lock().map_err(|_| {
            MathRuntimeError::Infrastructure("product ownership lock poisoned".into())
        })?;
        products.retain(|_, owner| owner.strong_count() != 0);
        if let Some(owner) = products.get(&key).and_then(std::sync::Weak::upgrade) {
            return Ok(owner);
        }
        let owner = Arc::new(ProductOwner {
            _lease: lease,
            _payload: payload,
        });
        products.insert(key, Arc::downgrade(&owner));
        Ok(owner)
    }
    pub(super) fn own_preparation(
        &self,
        (mut prepared, lease): (PreparedCase, Arc<pse_columnar::AllocationLease>),
    ) -> Result<Preparation, MathRuntimeError> {
        let key = vec![
            0,
            Arc::as_ptr(&prepared.plan) as usize,
            Arc::as_ptr(&prepared.structure) as usize,
            Arc::as_ptr(&prepared.presolve) as usize,
            prepared
                .coefficients
                .as_ref()
                .map_or(0, |p| Arc::as_ptr(p) as usize),
        ];
        let owner = self.shared_product(key, Arc::new(prepared.clone()), lease)?;
        prepared.plan = Arc::new(prepared.plan.as_ref().clone().with_owner(owner.clone()));
        prepared.artifacts = Arc::new(
            prepared
                .artifacts
                .iter()
                .cloned()
                .map(|request| request.with_owner(owner.clone()))
                .collect(),
        );
        Ok(Preparation {
            prepared: Arc::new(prepared),
            owner,
        })
    }
}
