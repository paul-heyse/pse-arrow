// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Prepared structures shared by every analysis of one package (A6): solver views and
//! value-independent observation programs, each keyed on its complete structural
//! identity (DP-09). A value-only change finds its structure here and rebinds values; only
//! a structural change prepares again.
use super::*;
use crate::math::{ExecutableCase, Preparation, modeling::ModelingCasePreparation};
use pse_compiler::workspace::{ModelingVariableState, Profile};
use pse_ids::ContentHash;
use pse_kernels::DerivativeOrder;
use pse_math::binding::CaseValues;
use std::{
    collections::{BTreeSet, VecDeque},
    sync::{Arc, Mutex},
};

/// Retained views per kind. Bounded, least recently used first out (DP-20).
const CAPACITY: usize = 16;
/// A bounded map from structural identity to a prepared product.
#[derive(Debug)]
struct Bounded<T>(VecDeque<(ContentHash, T)>);
impl<T: Clone> Bounded<T> {
    fn get(&mut self, key: &ContentHash) -> Option<T> {
        let index = self.0.iter().position(|(k, _)| k == key)?;
        let entry = self.0.remove(index)?;
        let value = entry.1.clone();
        self.0.push_back(entry);
        Some(value)
    }
    fn insert(&mut self, key: ContentHash, value: T) {
        self.0.retain(|(k, _)| *k != key);
        if self.0.len() == CAPACITY {
            self.0.pop_front();
        }
        self.0.push_back((key, value));
    }
}
impl<T> Default for Bounded<T> {
    fn default() -> Self {
        Self(VecDeque::new())
    }
}
/// The package's prepared solver views and observation programs.
#[derive(Debug, Default)]
pub(in crate::workflow) struct Views {
    solver: Mutex<Bounded<Preparation>>,
    observations: Mutex<Bounded<Arc<ExecutableCase>>>,
    parametric: Mutex<Bounded<Arc<ExecutableCase>>>,
}
impl Views {
    fn lock<T>(
        slot: &Mutex<Bounded<T>>,
    ) -> Result<std::sync::MutexGuard<'_, Bounded<T>>, WorkflowError> {
        slot.lock().map_err(|_| {
            crate::math::MathRuntimeError::Infrastructure("prepared view lock poisoned".into())
                .into()
        })
    }
}
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
        let cached = Views::lock(&self.views.solver)?.get(&key);
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
                Views::lock(&self.views.solver)?.insert(key, view.clone());
                view
            }
        };
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
        let product = model.compiled();
        let view = product.view_key(structure, order, compiler, &self.physical.key);
        let key = pse_compiler::workspace::PreparedModeling::parametric_key(&view, parameters);
        if let Some(program) = Views::lock(&self.views.parametric)?.get(&key) {
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
                compiler,
                cancel,
            )
            .await?;
        Views::lock(&self.views.parametric)?.insert(key, program.clone());
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
        if let Some(program) = Views::lock(&self.views.observations)?.get(&key) {
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
        Views::lock(&self.views.observations)?.insert(key, program.clone());
        Ok(program)
    }
}
