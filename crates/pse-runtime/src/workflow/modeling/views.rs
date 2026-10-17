// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Prepared structures shared by every analysis of one package (A6): solver views and
//! value-independent observation programs, each keyed on its complete structural
//! identity (DP-09). A value-only change finds its structure here and rebinds values; only
//! a structural change prepares again.
use super::*;
use crate::math::{ExecutableCase, modeling::ModelingCasePreparation};
use pse_compiler::workspace::{ModelingVariableState, Profile};
use pse_kernels::DerivativeOrder;
use pse_math::binding::CaseValues;
use std::{collections::BTreeSet, sync::Arc};
impl ModelingPackage {
    /// The solver view of `model` under `states`, bound to `values`. The first request for a
    /// structure prepares it; every later one rebinds values onto it (A6). The returned
    /// values are completed with the view's derived realization parameters (ADR-0104). The
    /// bound tightenings belong to `states`, not to the shared view (ADR-0103 item 4).
    pub(in crate::workflow) async fn bound_case(
        &self,
        model: &ModelingPreparation,
        values: CaseValues,
        states: &BTreeMap<SemanticId, ModelingVariableState>,
        order: DerivativeOrder,
        compiler: Profile,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingCasePreparation, WorkflowError> {
        self.bound_case_with_lookup_reservation(
            model,
            values,
            states,
            order,
            compiler,
            cancel,
            |bytes| {
                self.runtime
                    .shared
                    .math()
                    .reserve("math:bound-lookup", bytes)
            },
        )
        .await
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "the optional retention reservation is an independent effect from scientific case preparation"
    )]
    async fn bound_case_with_lookup_reservation(
        &self,
        model: &ModelingPreparation,
        values: CaseValues,
        states: &BTreeMap<SemanticId, ModelingVariableState>,
        order: DerivativeOrder,
        compiler: Profile,
        cancel: &crate::CancelSource,
        reserve: impl FnOnce(
            usize,
        ) -> Result<
            Arc<pse_columnar::AllocationLease>,
            crate::math::MathRuntimeError,
        > + Send,
    ) -> Result<ModelingCasePreparation, WorkflowError> {
        let preparation_driver = self.runtime.shared.math().preparation_driver(cancel)?;
        let cancel = &preparation_driver;
        let activity = self.runtime.admit_activity()?;
        activity
            .scope(async {
                let product = model.compiled();
                let lookup = product
                    .bound_lookup(states, order, compiler, &self.physical.key)
                    .map_err(crate::math::MathRuntimeError::from)?;
                let key = lookup.key.clone();
                let tightenings = lookup.tightenings.clone();
                let identity = product.occurrence_identity();
                let service = self.runtime.shared.math();
                let generation = service.modeling_cache.generation();
                let (case, retain) = match service.modeling_cache.bound_solver(key.clone()) {
                    Some(cached) => {
                        let case = service.rebind(&cached.view, values.clone(), cancel).await?;
                        if cached.identity == identity {
                            (case, false)
                        } else {
                            (service.attribute_modeling_view(&case, model)?, true)
                        }
                    }
                    None => {
                        let bound = product
                            .expand_bound_lookup(lookup)
                            .map_err(crate::math::MathRuntimeError::from)?;
                        (
                            service
                                .prepare_modeling_view(
                                    self.numerical_workspace()?,
                                    model.clone(),
                                    bound.structure,
                                    values.clone(),
                                    order,
                                    compiler,
                                    cancel,
                                )
                                .await?,
                            true,
                        )
                    }
                };
                if retain {
                    let bytes = key.retained_bytes()
                        + identity
                            .iter()
                            .map(|(_, role, _, _)| 128 + role.capacity())
                            .sum::<usize>()
                        + 1024;
                    // Retention is optional: denial cannot invalidate the scientifically
                    // admitted current case. The cache's publication is best-effort too.
                    if let Ok(owner) = reserve(bytes) {
                        service.modeling_cache.retain_bound_solver(
                            generation,
                            key,
                            Arc::new(crate::math::retention::RetainedBoundSolver {
                                view: case.clone(),
                                identity,
                                _owner: owner,
                            }),
                        );
                    }
                }
                Ok(ModelingCasePreparation {
                    model: model.clone(),
                    values: case.compiled().complete(&values),
                    case,
                    tightenings,
                })
            })
            .await
    }
    /// The parametric program of the solver view `prepared` over `parameters`, in request
    /// order (Plan 22 S1), compiled once per view and parameter selection and cached with
    /// the view (A6). Every declared parameter of the view's structure is admitted.
    pub(in crate::workflow) async fn parametric_program(
        &self,
        model: &ModelingPreparation,
        prepared: &ModelingCasePreparation,
        parameters: &[SemanticId],
        order: DerivativeOrder,
        compiler: Profile,
        cancel: &crate::CancelSource,
    ) -> Result<Arc<ExecutableCase>, WorkflowError> {
        let preparation_driver = self.runtime.shared.math().preparation_driver(cancel)?;
        let cancel = &preparation_driver;
        let activity = self.runtime.admit_activity()?;
        activity.scope(async {
        let structure = prepared.case.compiled().plan.structure();
        let declared: BTreeSet<_> = structure.parameters().iter().map(|p| p.id).collect();
        if let Some(unknown) = parameters.iter().find(|p| !declared.contains(p)) {
            return Err(contract(format!(
                "sensitivity parameter {unknown} is not a declared parameter of the solved case"
            )));
        }
        let generation = self.runtime.shared.math().modeling_cache.generation();
        let product = model.compiled();
        let view = product.view_key(structure, order, compiler, &self.physical.key);
        let key = pse_compiler::workspace::PreparedModeling::parametric_key(&view, parameters);
        if let Some(program) = self.runtime.shared.math().modeling_cache.program(key, true) {
            return Ok(program);
        }
        let program = self
            .runtime
            .shared
            .math()
            .prepare_modeling_parametric(
                self.numerical_workspace()?,
                prepared.case.clone(),
                parameters.to_vec(),
                order,
                compiler,
                cancel,
            )
            .await?;
        self.runtime.shared.math().modeling_cache.retain_program(
            generation,
            key,
            program.clone(),
            true,
        );
        Ok(program)

        }).await
    }
    /// The value-independent program observing `rows` of `model`, compiled once per
    /// structure.
    pub(in crate::workflow) async fn observation_program(
        &self,
        model: &ModelingPreparation,
        rows: &BTreeSet<SemanticId>,
        compiler: Profile,
        cancel: &crate::CancelSource,
    ) -> Result<Arc<ExecutableCase>, WorkflowError> {
        let preparation_driver = self.runtime.shared.math().preparation_driver(cancel)?;
        let cancel = &preparation_driver;
        let activity = self.runtime.admit_activity()?;
        activity
            .scope(async {
                let generation = self.runtime.shared.math().modeling_cache.generation();
                let product = model.compiled();
                let structure = product
                    .observation_structure(rows)
                    .map_err(crate::math::MathRuntimeError::from)?;
                let key = product.view_key(
                    &structure,
                    DerivativeOrder::Value,
                    compiler,
                    &self.physical.key,
                );
                if let Some(program) = self
                    .runtime
                    .shared
                    .math()
                    .modeling_cache
                    .program(key, false)
                {
                    return Ok(program);
                }
                let program = self
                    .runtime
                    .shared
                    .math()
                    .prepare_modeling_observations(
                        self.numerical_workspace()?,
                        model.clone(),
                        structure,
                        compiler,
                        cancel,
                    )
                    .await?;
                self.runtime.shared.math().modeling_cache.retain_program(
                    generation,
                    key,
                    program.clone(),
                    false,
                );
                Ok(program)
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_diagnostics::observation::{self, ObservationHub, Policy, Value};

    #[tokio::test]
    async fn denied_bound_lookup_retention_returns_current_case_without_cache_entry() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let runtime = crate::workflow::tests::runtime();
        let declarations = pse_authoring::language::parse(
            "package p {def Root {var x:Scalar; eq e:x==2;}}",
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let root = declarations
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(declarations, crate::workflow::tests::physical())
            .await
            .unwrap();
        let cancel = crate::CancelSource::new();
        let model = package
            .prepare(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                &cancel,
            )
            .await
            .unwrap();
        let x = model
            .compiled()
            .model
            .symbols
            .values()
            .find(|s| s.lineage.path.ends_with(".x"))
            .unwrap()
            .id;
        let compiler = crate::workflow::tests::compiler_profile();
        let states = BTreeMap::new();
        let key = model
            .compiled()
            .bound_lookup(
                &states,
                DerivativeOrder::First,
                compiler,
                &package.physical.key,
            )
            .unwrap()
            .key;
        let service = runtime.shared.math();
        assert!(service.modeling_cache.bound_solver(key.clone()).is_none());
        let denials = AtomicUsize::new(0);
        for value in [3., 5.] {
            let current = package
                .bound_case_with_lookup_reservation(
                    &model,
                    CaseValues {
                        scalars: BTreeMap::from([(x, value)]),
                    },
                    &states,
                    DerivativeOrder::First,
                    compiler,
                    &cancel,
                    |_| {
                        denials.fetch_add(1, Ordering::Relaxed);
                        Err(crate::math::MathRuntimeError::Limit(
                            "test:lookup-retention-denied",
                        ))
                    },
                )
                .await
                .unwrap();
            assert_eq!(current.values.scalars[&x], value);
            assert_eq!(current.case.compiled().plan.structure().rows().len(), 1);
            assert!(service.modeling_cache.bound_solver(key.clone()).is_none());
        }
        assert_eq!(
            denials.load(Ordering::Relaxed),
            2,
            "each completed case reached the denied optional reservation"
        );
    }

    #[tokio::test]
    async fn warm_bound_lookup_expands_once_rebinds_values_and_rejects_invalid_bounds() {
        let runtime = crate::workflow::tests::runtime();
        let source = "package p {def Root {var x:Scalar; eq e:x==2;}}";
        let declarations = pse_authoring::language::parse(
            source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let root = declarations
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(declarations, crate::workflow::tests::physical())
            .await
            .unwrap();
        let cancel = crate::CancelSource::new();
        let model = package
            .prepare(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                &cancel,
            )
            .await
            .unwrap();
        let x = model
            .compiled()
            .model
            .symbols
            .values()
            .find(|s| s.lineage.path.ends_with(".x"))
            .unwrap()
            .id;
        let compiler = crate::workflow::tests::compiler_profile();
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("bound.lookup.hit");
        let activity = runtime.admit_activity().unwrap();
        activity
            .scope(observation::scope(Some(operation.context()), async {
                let first = package
                    .bound_case(
                        &model,
                        CaseValues {
                            scalars: BTreeMap::from([(x, 3.)]),
                        },
                        &BTreeMap::new(),
                        DerivativeOrder::First,
                        compiler,
                        &cancel,
                    )
                    .await
                    .unwrap();
                let second = package
                    .bound_case(
                        &model,
                        CaseValues {
                            scalars: BTreeMap::from([(x, 5.)]),
                        },
                        &BTreeMap::new(),
                        DerivativeOrder::First,
                        compiler,
                        &cancel,
                    )
                    .await
                    .unwrap();
                assert!(Arc::ptr_eq(
                    &first.case.compiled().plan,
                    &second.case.compiled().plan
                ));
                assert_eq!(first.values.scalars[&x], 3.);
                assert_eq!(second.values.scalars[&x], 5.);
                assert_eq!(
                    first.case.compiled().occurrences.allocation_identity(),
                    second.case.compiled().occurrences.allocation_identity()
                );
                let invalid = BTreeMap::from([(
                    x,
                    ModelingVariableState {
                        lower: Some(Some(9.)),
                        upper: Some(Some(1.)),
                        ..Default::default()
                    },
                )]);
                assert!(
                    package
                        .bound_case(
                            &model,
                            second.values.clone(),
                            &invalid,
                            DerivativeOrder::First,
                            compiler,
                            &cancel
                        )
                        .await
                        .is_err()
                );
            }))
            .await;
        let snapshot = operation
            .reader()
            .snapshot(std::time::Duration::from_secs(60));
        assert_eq!(
            snapshot
                .metrics
                .iter()
                .find(|m| m.name == "compiler.bound_structures")
                .unwrap()
                .cumulative,
            Value::Count(2),
            "one valid cold expansion and one invalid-bound validation, with no warm-hit expansion"
        );
    }

    #[tokio::test]
    async fn selected_observation_cache_carries_one_structure_and_rebinds_actual_values() {
        let runtime = crate::workflow::tests::runtime();
        let declarations = pse_authoring::language::parse(
            "package p { def Root { var x:Scalar; eq e:x==2; } }",
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let root = declarations
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(declarations, crate::workflow::tests::physical())
            .await
            .unwrap();
        let cancel = crate::CancelSource::new();
        let model = package
            .prepare(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                &cancel,
            )
            .await
            .unwrap();
        let row = model
            .compiled()
            .model
            .equations
            .iter()
            .find(|equation| equation.lineage.path == "Root.e")
            .unwrap()
            .id;
        let rows = BTreeSet::from([row]);
        let compiler = crate::workflow::tests::compiler_profile();
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("selected.observation.cache");
        let activity = runtime.admit_activity().unwrap();
        activity
            .scope(observation::scope(Some(operation.context()), async {
                let first = package
                    .observation_program(&model, &rows, compiler, &cancel)
                    .await
                    .unwrap();
                let second = package
                    .observation_program(&model, &rows, compiler, &cancel)
                    .await
                    .unwrap();
                assert!(Arc::ptr_eq(&first, &second));
            }))
            .await;
        let snapshot = operation
            .reader()
            .snapshot(std::time::Duration::from_secs(60));
        let structures = snapshot
            .metrics
            .iter()
            .find(|metric| metric.name == "compiler.observation_structures")
            .unwrap();
        assert_eq!(
            structures.cumulative,
            Value::Count(2),
            "each key lookup constructs one structure; a miss must carry that same structure into preparation"
        );
        let x = model
            .compiled()
            .model
            .symbols
            .values()
            .find(|symbol| symbol.lineage.path == "Root.x")
            .unwrap()
            .id;
        for (value, expected) in [(3., 1.), (5., 3.)] {
            let observed = package
                .observe_registered(
                    model.clone(),
                    rows.clone(),
                    CaseValues {
                        scalars: BTreeMap::from([(x, value)]),
                    },
                    compiler,
                    BTreeMap::new(),
                    &cancel,
                )
                .await
                .unwrap();
            assert_eq!(observed[&row], expected);
        }
    }
}
