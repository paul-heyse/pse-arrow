// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-bearing symbolic preparation acquires qualified recipes before body materialization.
use super::*;
impl ModelingPackage {
    /// Prepare authored mathematics for later derivative, structural or class work.
    /// Protected typed recipes retain their original source/proof authority; a
    /// genuine absence uses ordinary source admission under this same operation.
    #[expect(
        clippy::too_many_arguments,
        reason = "original source specialization and the current source/deployment operation remain explicit"
    )]
    pub async fn prepare_authored(
        &self,
        services: &crate::workflow::SourceServices,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingPreparation, WorkflowError> {
        limits.validate().map_err(modeling_error)?;
        let preparation = services.preparation();
        self.check_service_bindings(preparation)?;
        let activity = preparation.admit_activity()?;
        activity
            .scope(async {
                let math = preparation.math();
                let driver = math.preparation_driver(cancel)?;
                let deadline = math.preparation_deadline(&driver)?;
                let original = engagement::OriginalPreparationInputs::capture(
                    preparation,
                    root,
                    instance,
                    &bindings,
                    limits,
                )?;
                let mut selected = self
                    .checked_selection_scoped(preparation, &[root], &driver, deadline)
                    .await?;
                let key = math.basis_key(
                    &selected.admission,
                    root,
                    instance,
                    bindings.clone(),
                    limits,
                )?;
                let workspace = self.pure_workspace(preparation)?;
                let frontier = math
                    .plan_frontier_keyed(
                        key,
                        workspace,
                        selected.admission.revision.clone(),
                        &driver,
                        deadline,
                    )
                    .await?;
                let context = compiler_context(&self.physical, &self.providers);
                let generation = math.modeling_cache.generation();
                let control = pse_diagnostics::Cancellation::default();
                let mut protected = portable_preparation::ProtectedPreparation::open(
                    self,
                    Some(services),
                    math,
                    &selected.admission.versions,
                    &driver,
                    deadline,
                )
                .await?;
                let result = portable_preparation::run(&driver, &control, async {
                    // Only this acquisition owns the sorted set. Prepay its complete
                    // flat backing before allocation; at most one seal per request is added.
                    let receiving = protected
                        .as_ref()
                        .is_some_and(|current| current.publication().is_some());
                    let received_count = if receiving {
                        frontier.body_requests().count()
                    } else {
                        0
                    };
                    let received_bytes = received_count
                        .checked_mul(size_of::<pse_ids::roles::SemanticBodyHash>())
                        .ok_or_else(|| contract("received mathematical recipe inventory extent"))?;
                    let _received_owner =
                        math.reserve("math:received-typed-recipe-seals", received_bytes)?;
                    let mut received = Vec::new();
                    received.try_reserve_exact(received_count).map_err(|_| {
                        crate::math::MathRuntimeError::Limit(
                            "received mathematical recipe inventory allocation",
                        )
                    })?;
                    if let Some((store, read, namespace)) =
                        protected.as_mut().and_then(|current| current.receiving())
                    {
                        for identity in frontier.body_requests() {
                            canonical::checkpoint(&driver)?;
                            if let Some(body) = crate::math::portable::reuse_body_current(
                                math,
                                store,
                                read,
                                namespace,
                                identity,
                                &context,
                                &control.flag(),
                            )
                            .await
                            .map_err(crate::math::portable::runtime_error)?
                            {
                                math.retain_replayed_body(generation, body)?;
                                received.push(identity);
                            }
                        }
                    }
                    received.sort_unstable();
                    received.dedup();
                    let (mut model, _basis) = self
                        .prepare_selected_basis(
                            preparation,
                            &mut selected,
                            root,
                            instance,
                            bindings,
                            limits,
                            &driver,
                            deadline,
                        )
                        .await?;
                    model.original = Some(original);
                    if let Some((store, read, namespace)) =
                        protected.as_ref().and_then(|current| current.publication())
                    {
                        let selection = crate::math::portable::selection_material(math, read)
                            .map_err(crate::math::portable::runtime_error)?;
                        let context = model.compiled().compiler_context();
                        for body in model
                            .compiled()
                            .portable_bodies()
                            .filter(|body| body.has_portable_payload())
                        {
                            canonical::checkpoint(&driver)?;
                            if body
                                .semantic_identity()
                                .is_some_and(|identity| received.binary_search(&identity).is_ok())
                            {
                                continue;
                            }
                            let description = crate::math::portable::describe_body(
                                math,
                                read,
                                selection.clone(),
                                namespace.producer_key().to_owned(),
                                namespace.producer_class(),
                                body,
                                &context,
                            )
                            .map_err(crate::math::portable::runtime_error)?;
                            crate::math::portable::publish_description(
                                math,
                                store,
                                read,
                                &description,
                                deadline.into(),
                            )
                            .await
                            .map_err(crate::math::portable::runtime_error)?;
                        }
                    }
                    Ok(model)
                })
                .await;
                portable_preparation::finish(protected, result).await
            })
            .await
    }
}

#[cfg(test)]
#[path = "symbolic-preparation-tests.rs"]
mod tests;
