// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Escaping immutable product ownership, separate from the bounded Salsa generation.
use super::*;
use pse_backend_native::ProblemError;

#[derive(Debug)]
pub(super) struct ProductOwner {
    _lease: Arc<pse_columnar::AllocationLease>,
    // Keep the exact original allocations alive: addresses below are only local
    // allocation identities, never semantic IDs or serialized compatibility keys.
    _payload: Arc<dyn pse_math::AllocationOwner>,
    _parents: Vec<Arc<ProductOwner>>,
}
impl MathService {
    /// Bound a preparation consumer by its task clock without renewing a shared loader's
    /// lifetime. Dropping the waiter leaves native teardown with the completion owner.
    pub(crate) async fn within_task<T, E>(
        scope: &pse_kernels::ExecutionScope,
        driver: &crate::CancelSource,
        operation: impl Future<Output = Result<T, E>>,
    ) -> Result<T, E>
    where
        E: From<MathRuntimeError>,
    {
        scope
            .check()
            .map_err(ProblemError::Provider)
            .map_err(MathRuntimeError::from)?;
        if driver.token().is_cancelled() {
            return Err(MathRuntimeError::Cancelled.into());
        }
        let expires = async {
            match scope.deadline() {
                Some(deadline) => tokio::time::sleep_until(deadline.into()).await,
                None => std::future::pending::<()>().await,
            }
        };
        tokio::pin!(operation);
        let value = tokio::select! {
            biased;
            () = driver.cancelled() => return Err(MathRuntimeError::Cancelled.into()),
            () = expires => return Err(MathRuntimeError::from(ProblemError::Provider(pse_kernels::ProviderError::Deadline)).into()),
            value = &mut operation => value?,
        };
        scope
            .check()
            .map_err(ProblemError::Provider)
            .map_err(MathRuntimeError::from)?;
        if driver.token().is_cancelled() {
            return Err(MathRuntimeError::Cancelled.into());
        }
        Ok(value)
    }
    /// Await immutable derivative compilation within the consumer's original clock.
    /// Loader abandonment cancels its own flight, never the diagnostic's task flag.
    pub(crate) async fn prepare_order_within_task(
        self: &Arc<Self>,
        prepared: Preparation,
        order: pse_kernels::DerivativeOrder,
        scope: &pse_kernels::ExecutionScope,
        driver: &crate::CancelSource,
    ) -> Result<Preparation, MathRuntimeError> {
        Self::within_task(
            scope,
            driver,
            self.prepare_order(prepared, order, FlightCancellation::default()),
        )
        .await
    }
    /// Prepare the consumer's derivative demand on an admitted job, retaining the
    /// original weaker product and sharing its unchanged owned components.
    pub(crate) async fn prepare_order(
        self: &Arc<Self>,
        prepared: Preparation,
        order: pse_kernels::DerivativeOrder,
        control: FlightCancellation,
    ) -> Result<Preparation, MathRuntimeError> {
        if prepared.compiled().plan.order() >= order {
            return Ok(prepared);
        }
        let source = prepared.prepared.clone();
        let upgraded = self
            .job_retained(1, self.policy.workspace_bytes, control, move |flag| {
                let product = source.prepare_order(order, &flag)?;
                let bytes = product.retained_bytes();
                Ok((product, bytes))
            })
            .await?;
        self.own_preparation(upgraded)
    }
    /// Compile only separately demanded first actions, retaining normal artifact identities.
    pub(crate) async fn prepare_directional_actions(
        self: &Arc<Self>,
        prepared: Preparation,
    ) -> Result<Preparation, MathRuntimeError> {
        if prepared.compiled().plan.has_directional_actions() {
            return Ok(prepared);
        }
        let source = prepared.prepared.clone();
        let upgraded = self
            .job_retained(
                1,
                self.policy.workspace_bytes,
                FlightCancellation::default(),
                move |flag| {
                    let product = source.prepare_directional_actions(&flag)?;
                    let bytes = product.retained_bytes();
                    Ok((product, bytes))
                },
            )
            .await?;
        self.own_preparation(upgraded)
    }
    pub(super) fn shared_product(
        &self,
        key: Vec<usize>,
        payload: Arc<dyn pse_math::AllocationOwner>,
        lease: Arc<pse_columnar::AllocationLease>,
        parents: Vec<Arc<ProductOwner>>,
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
            _parents: parents,
        });
        products.insert(key, Arc::downgrade(&owner));
        Ok(owner)
    }
    pub(super) fn own_preparation(
        &self,
        (mut prepared, lease): (PreparedCase, Arc<pse_columnar::AllocationLease>),
    ) -> Result<Preparation, MathRuntimeError> {
        let mut components: Vec<(Vec<usize>, usize, Arc<dyn pse_math::AllocationOwner>)> =
            Vec::new();
        let mut add =
            |kind, pointer, bytes, payload| components.push((vec![kind, pointer], bytes, payload));
        for body in prepared.plan.bodies().values() {
            add(
                10,
                body.allocation_identity(),
                body.retained_bytes(),
                body.clone(),
            );
        }
        let mut plan_key = vec![11];
        plan_key.extend(prepared.plan.allocation_identity());
        add(
            12,
            prepared.quantities.allocation_identity(),
            prepared.quantities.allocation_extent(),
            prepared.quantities.allocation_payload(),
        );
        add(
            13,
            prepared.structure.allocation_identity(),
            prepared.structural_bytes(),
            prepared.structure.allocation_payload(),
        );
        add(
            14,
            prepared.presolve.allocation_identity(),
            prepared.presolve.bytes(),
            prepared.presolve.allocation_payload(),
        );
        add(
            15,
            prepared.coefficient_values.allocation_identity(),
            prepared.binding_bytes(),
            prepared.coefficient_values.allocation_payload(),
        );
        add(
            16,
            prepared.occurrences.allocation_identity(),
            prepared.provenance_bytes(),
            prepared.occurrences.allocation_payload(),
        );
        add(
            17,
            prepared.derivation.allocation_identity(),
            prepared.derivation.retained_bytes() + 64,
            prepared.derivation.allocation_payload(),
        );
        add(
            18,
            prepared.derived.allocation_identity(),
            prepared.derived.retained_bytes() + 64,
            prepared.derived.allocation_payload(),
        );
        add(
            20,
            prepared.artifacts.allocation_identity(),
            prepared.artifact_descriptor_bytes(),
            prepared.artifacts.allocation_payload(),
        );
        if let Some(coefficients) = &prepared.coefficients {
            add(
                19,
                coefficients.allocation_identity(),
                coefficients.retained_bytes() + 64,
                coefficients.allocation_payload(),
            );
        }
        components.push((
            plan_key,
            prepared.plan.allocation_bytes(),
            prepared.plan.clone(),
        ));
        let total: usize = components.iter().map(|(_, b, _)| *b).sum();
        let wrappers = lease.size().checked_sub(total).ok_or_else(|| {
            MathRuntimeError::Infrastructure(
                "prepared allocation extent does not cover components".into(),
            )
        })?;
        let mut sizes: Vec<_> = components.iter().map(|(_, b, _)| *b).collect();
        sizes.push(wrappers);
        let mut leases = lease.partition(&sizes).map_err(|_| {
            MathRuntimeError::Infrastructure(
                "prepared job lease was shared before allocation transfer".into(),
            )
        })?;
        let wrapper_lease = leases.pop().ok_or_else(|| {
            MathRuntimeError::Infrastructure("prepared wrapper lease missing".into())
        })?;
        let parents = components
            .into_iter()
            .zip(leases)
            .map(|((key, _, payload), lease)| self.shared_product(key, payload, lease, Vec::new()))
            .collect::<Result<Vec<_>, _>>()?;
        let owner = Arc::new(ProductOwner {
            _lease: wrapper_lease,
            _payload: Arc::new(prepared.clone()),
            _parents: parents,
        });
        attach_field_owners(&mut prepared, owner.clone());
        prepared.plan = Arc::new(prepared.plan.as_ref().clone().with_owner(owner.clone()));
        prepared.artifacts = Arc::new(
            prepared
                .artifacts
                .iter()
                .cloned()
                .map(|request| request.with_owner(owner.clone()))
                .collect::<Vec<_>>(),
        )
        .into();
        prepared.artifacts = prepared.artifacts.with_owner(owner.clone());
        Ok(Preparation {
            prepared: Arc::new(prepared),
            owner,
            executable: Arc::default(),
        })
    }
    pub(super) fn own_semantic_body(
        &self,
        body: Arc<pse_compiler::typed_math::AdmittedBody>,
    ) -> Result<Arc<pse_compiler::typed_math::AdmittedBody>, MathRuntimeError> {
        let math_bytes = body.math().retained_bytes();
        let wrapper_bytes = 2 * body.descriptor_bytes() + 256;
        let mut leases = self
            .reserve("modeling:semantic-body", math_bytes + wrapper_bytes)?
            .partition(&[math_bytes, wrapper_bytes])
            .map_err(|_| MathRuntimeError::Infrastructure("body allocation transfer".into()))?;
        let wrapper = leases
            .pop()
            .ok_or_else(|| MathRuntimeError::Infrastructure("body wrapper lease".into()))?;
        let math = leases
            .pop()
            .ok_or_else(|| MathRuntimeError::Infrastructure("body math lease".into()))?;
        let math_owner = self.shared_product(
            vec![10, body.math().allocation_identity()],
            body.math().clone(),
            math,
            Vec::new(),
        )?;
        let owner = Arc::new(ProductOwner {
            _lease: wrapper,
            _payload: body.clone(),
            _parents: vec![math_owner],
        });
        let owned = Arc::new(body.as_ref().clone().with_owner(owner.clone()));
        self.products
            .lock()
            .map_err(|_| {
                MathRuntimeError::Infrastructure("body descriptor ownership lock poisoned".into())
            })?
            .insert(
                vec![31, Arc::as_ptr(&owned) as usize],
                Arc::downgrade(&owner),
            );
        Ok(owned)
    }
    pub(super) fn own_modeling_product(
        &self,
        product: &pse_compiler::workspace::PreparedModeling,
        lease: Arc<pse_columnar::AllocationLease>,
    ) -> Result<Arc<ProductOwner>, MathRuntimeError> {
        let mut components: Vec<(Vec<usize>, usize, Arc<dyn pse_math::AllocationOwner>)> =
            Vec::new();
        for body in product.admitted.bodies.values() {
            components.push((
                vec![10, body.math().allocation_identity()],
                body.math().retained_bytes(),
                body.math().clone(),
            ));
            components.push((
                vec![31, Arc::as_ptr(body) as usize],
                body.descriptor_bytes(),
                body.clone(),
            ));
        }
        let mut sizes: Vec<_> = components.iter().map(|(_, bytes, _)| *bytes).collect();
        let body_bytes: usize = sizes.iter().sum();
        sizes.push(lease.size().checked_sub(body_bytes).ok_or_else(|| {
            MathRuntimeError::Infrastructure("modeling allocation extent".into())
        })?);
        let mut leases = lease.partition(&sizes).map_err(|_| {
            MathRuntimeError::Infrastructure("modeling job lease already shared".into())
        })?;
        let revision = leases.pop().ok_or_else(|| {
            MathRuntimeError::Infrastructure("modeling revision lease missing".into())
        })?;
        let parents = components
            .into_iter()
            .zip(leases)
            .map(|((key, _, payload), lease)| self.shared_product(key, payload, lease, Vec::new()))
            .collect::<Result<Vec<_>, _>>()?;
        let owner = self.shared_product(
            vec![
                30,
                product.model.allocation_identity(),
                product.admitted.allocation_identity(),
            ],
            Arc::new(product.clone()),
            revision,
            parents,
        )?;
        Ok(owner)
    }
    /// Bind fresh revision attribution onto shared mathematics without copying old maps.
    pub(crate) fn attribute_modeling_view(
        &self,
        view: &Preparation,
        model: &modeling::ModelingPreparation,
    ) -> Result<Preparation, MathRuntimeError> {
        let bytes = model
            .compiled()
            .model
            .source_occurrences()
            .len()
            .checked_mul(4 * size_of::<pse_compiler::typed_math::Occurrence>() + 1024)
            .and_then(|bytes| bytes.checked_add(2 * size_of::<PreparedCase>() + 1024))
            .ok_or(MathRuntimeError::Limit("revision attribution extent"))?;
        let lease = self.reserve("math:revision-attribution", bytes)?;
        let occurrences = model.compiled().occurrences();
        if view.compiled().occurrences.as_ref() == &occurrences {
            return Ok(view.clone());
        }
        let mut prepared = view.compiled().clone();
        prepared.occurrences = Arc::new(occurrences).into();
        Ok(Self::own_rebind(view, prepared, lease))
    }
    /// Own the value-dependent products of a rebind (A6). The structure, its artifact
    /// owners and its assembled programs stay those of `structure`; `lease` covers only the
    /// rebuilt value products.
    pub(super) fn own_rebind(
        structure: &Preparation,
        prepared: PreparedCase,
        lease: Arc<pse_columnar::AllocationLease>,
    ) -> Preparation {
        Self::own_binding(
            structure.owner.clone(),
            prepared,
            lease,
            structure.executable.clone(),
        )
    }
    /// Attach newly allocated bindings to an existing structural/evaluator owner.
    pub(super) fn own_binding(
        parent: Arc<ProductOwner>,
        prepared: PreparedCase,
        lease: Arc<pse_columnar::AllocationLease>,
        executable: Arc<std::sync::OnceLock<Arc<ExecutableCase>>>,
    ) -> Preparation {
        let mut prepared = prepared;
        let owner = Arc::new(ProductOwner {
            _lease: lease,
            _payload: Arc::new(prepared.clone()),
            _parents: vec![parent],
        });
        attach_field_owners(&mut prepared, owner.clone());
        Preparation {
            owner,
            prepared: Arc::new(prepared),
            executable,
        }
    }
}
fn attach_field_owners(prepared: &mut PreparedCase, owner: Arc<ProductOwner>) {
    prepared.artifacts = prepared.artifacts.clone().with_owner(owner.clone());
    prepared.quantities = prepared.quantities.clone().with_owner(owner.clone());
    prepared.structure = prepared.structure.clone().with_owner(owner.clone());
    prepared.presolve = prepared.presolve.clone().with_owner(owner.clone());
    prepared.coefficient_values = prepared
        .coefficient_values
        .clone()
        .with_owner(owner.clone());
    prepared.occurrences = prepared.occurrences.clone().with_owner(owner.clone());
    prepared.derivation = prepared.derivation.clone().with_owner(owner.clone());
    prepared.derived = prepared.derived.clone().with_owner(owner.clone());
    prepared.coefficients = prepared
        .coefficients
        .take()
        .map(|value| value.with_owner(owner));
}
