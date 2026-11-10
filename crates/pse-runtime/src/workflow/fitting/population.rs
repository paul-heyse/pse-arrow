// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pure steady experiment frontiers through their existing physical assemblies.
use super::*;
use pse_math::assembly::{
    CasePopulationEvaluation, CrossCasePopulation, CrossCasePopulationWorker,
};

/// Original experiment addresses and their admitted mathematical projection.
/// A foreign lane selects from this owner without changing placement or width.
#[derive(Debug)]
pub(super) struct PreparedSteadyPopulation {
    indices: Vec<usize>,
    projection: Arc<CrossCasePopulation>,
}
impl PreparedSteadyPopulation {
    pub(super) fn candidates(
        experiments: &[Experiment],
        order: DerivativeOrder,
    ) -> Vec<(usize, Arc<pse_math::assembly::CaseAssembly>)> {
        let ceiling = order.min(DerivativeOrder::First);
        experiments
            .iter()
            .enumerate()
            .filter_map(|(index, experiment)| match experiment {
                Experiment::Steady(source)
                    if source.providers.is_empty()
                        && CrossCasePopulation::eligible(&source.case.assembly, ceiling) =>
                {
                    Some((index, Arc::clone(&source.case.assembly)))
                }
                _ => None,
            })
            .collect()
    }
    pub(super) fn new(
        candidates: Vec<(usize, Arc<pse_math::assembly::CaseAssembly>)>,
        order: DerivativeOrder,
    ) -> Result<Self, pse_math::MathError> {
        let (indices, cases) = candidates.into_iter().unzip();
        Ok(Self {
            indices,
            projection: CrossCasePopulation::new(cases, order.min(DerivativeOrder::First))?,
        })
    }
    pub(super) fn contains(&self, index: usize) -> bool {
        self.indices.binary_search(&index).is_ok()
    }
    pub(super) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.indices.capacity() * size_of::<usize>()
            + self.projection.retained_bytes()
    }
    pub(super) fn worker_bytes(
        &self,
        experiments: &[Experiment],
    ) -> Result<usize, pse_math::MathError> {
        let values = self
            .indices
            .iter()
            .try_fold(0usize, |bytes, &index| {
                let Experiment::Steady(source) = &experiments[index] else {
                    return None;
                };
                // Original steady values previously cloned on every trial. Keep one
                // original-coordinate map per assigned lane instead.
                bytes.checked_add(source.values.clone_allocation_bytes().ok()?)
            })
            .ok_or(pse_math::MathError::Limit("fit population physical values"))?;
        self.projection
            .worker_bytes()?
            .checked_add(values)
            .and_then(|bytes| {
                bytes.checked_add(
                    size_of::<SteadyPopulationWorker>() + self.indices.len() * size_of::<usize>(),
                )
            })
            .ok_or(pse_math::MathError::Limit("fit population worker"))
    }
    pub(super) fn worker(
        &self,
        problem: &FitProblem,
        assigned: Option<&[usize]>,
        scope: pse_kernels::ExecutionScope,
    ) -> Result<SteadyPopulationWorker, native::ProblemError> {
        let indices = self
            .indices
            .iter()
            .copied()
            .filter(|index| assigned.is_none_or(|assigned| assigned.contains(index)))
            .collect::<Vec<_>>();
        let projection = if indices == self.indices {
            Arc::clone(&self.projection)
        } else {
            let cases = indices
                .iter()
                .map(|index| {
                    self.indices
                        .binary_search(index)
                        .map(|local| Arc::clone(&self.projection.cases()[local]))
                        .map_err(|_| native::ProblemError::internal("fit population assignment"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            CrossCasePopulation::new(
                cases,
                problem.contract.derivatives.min(DerivativeOrder::First),
            )?
        };
        let values = indices
            .iter()
            .map(|&index| match &problem.experiments[index] {
                Experiment::Steady(source) => Ok(source.values.clone()),
                _ => Err(native::ProblemError::internal(
                    "fit population original experiment",
                )),
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(SteadyPopulationWorker {
            indices,
            values,
            worker: projection.worker_scoped(scope)?,
        })
    }
}
#[derive(Debug)]
pub(super) struct SteadyPopulationWorker {
    indices: Vec<usize>,
    values: Vec<CaseValues>,
    worker: CrossCasePopulationWorker,
}
impl SteadyPopulationWorker {
    pub(super) fn evaluate(
        &mut self,
        problem: &FitProblem,
        x: &[f64],
        demand: oracle::Demand,
    ) -> Result<(), native::ProblemError> {
        for (&index, values) in self.indices.iter().zip(&mut self.values) {
            let Experiment::Steady(source) = &problem.experiments[index] else {
                return Err(native::ProblemError::internal("fit population source"));
            };
            for &(id, column) in &source.coordinates {
                values.scalars.insert(id, x[column.get()]);
            }
        }
        let order = if demand.jacobian || demand.responses || demand.adjoint {
            DerivativeOrder::First
        } else {
            DerivativeOrder::Value
        };
        self.worker.evaluate(&self.values, order)?;
        Ok(())
    }
    pub(super) fn result(
        &mut self,
        index: usize,
    ) -> Option<Result<&CasePopulationEvaluation, pse_math::MathError>> {
        self.indices
            .binary_search(&index)
            .ok()
            .map(|local| self.worker.result(local))
    }
}
