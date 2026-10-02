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
                        self.workspace.clone(),
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
                self.workspace.clone(),
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
                self.workspace.clone(),
                model.clone(),
                rows.clone(),
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
    }
}
