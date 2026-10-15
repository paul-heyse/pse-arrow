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
        let preparation_driver = self.runtime.shared.math().preparation_driver(cancel)?;
        let cancel = &preparation_driver;
        let activity = self.runtime.admit_activity()?;
        activity
            .scope(async {
                let product = model.compiled();
                let bound = product
                    .bound_structure(states)
                    .map_err(crate::math::MathRuntimeError::from)?;
                let key = product.view_key(&bound.structure, order, compiler, &self.physical.key);
                let service = self.runtime.shared.math();
                let generation = service.modeling_cache.generation();
                let cached = service.modeling_cache.solver(key);
                let case = match cached {
                    Some(view) => service.rebind(&view, values.clone(), cancel).await?,
                    None => {
                        let view = service
                            .prepare_modeling_view(
                                self.numerical_workspace()?,
                                model.clone(),
                                bound.structure,
                                values.clone(),
                                order,
                                compiler,
                                cancel,
                            )
                            .await?;
                        service
                            .modeling_cache
                            .retain_solver(generation, key, view.clone());
                        view
                    }
                };
                let case = service.attribute_modeling_view(&case, model)?;
                Ok(ModelingCasePreparation {
                    model: model.clone(),
                    values: case.compiled().complete(&values),
                    case,
                    tightenings: bound.tightenings,
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
