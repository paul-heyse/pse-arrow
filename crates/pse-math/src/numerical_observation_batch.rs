// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Ordered population transport around the shared checked packed arithmetic owner.
use super::*;
use crate::{
    assembly::{CaseBatchOutcome, CaseBatchPoint},
    batch::{CheckedBatchExecution, PointOutcome, PointResult},
};

fn empty_result(case: &NumericalObservationCase) -> Evaluation {
    let rows = case.rows.len();
    let columns = case.coordinates.len();
    Evaluation {
        applicability: Vec::new(),
        values: vec![0.; rows],
        jacobian: vec![
            0.;
            if case.order >= DerivativeOrder::First {
                rows * columns
            } else {
                0
            }
        ],
        hessians: vec![
            0.;
            if case.order >= DerivativeOrder::Second {
                rows * columns * columns
            } else {
                0
            }
        ],
    }
}
impl NumericalObservationAssembly {
    /// Complete private population backing before any checked worker is instantiated.
    /// Original finite profile limits remain distinct from the caller's actual grant.
    pub fn batch_worker_bytes(&self, max_rows: usize) -> Result<usize, MathError> {
        let overflow = || MathError::Limit("numerical observation population backing");
        let inputs = self
            .programs
            .iter()
            .map(|program| program.gather.len())
            .max()
            .unwrap_or(0);
        let mut bytes = size_of::<NumericalObservationBatchWorker>();
        let mut evidence = 0usize;
        for (program, instance) in self.programs.iter().zip(&self.binding.instances) {
            bytes = bytes
                .checked_add(
                    program
                        .template
                        .batch_backing_bytes(max_rows, self.binding.order)?,
                )
                .ok_or_else(overflow)?;
            evidence = evidence
                .checked_add(program.template.extents().evaluation_bytes)
                .and_then(|bytes| bytes.checked_add(instance.attribution.retained_bytes()))
                .ok_or_else(overflow)?;
        }
        let results = self
            .result_components
            .checked_mul(size_of::<f64>())
            .and_then(|bytes| bytes.checked_add(evidence))
            .and_then(|bytes| {
                bytes.checked_add(
                    size_of::<Result<Evaluation, MathError>>()
                        + size_of::<usize>()
                        + 2 * size_of::<bool>()
                        + size_of::<PointResult>(),
                )
            })
            .and_then(|bytes| {
                bytes.checked_add(self.binding.rows.len().checked_mul(size_of::<f64>())?)
            })
            .ok_or_else(overflow)?;
        bytes = bytes
            .checked_add(max_rows.checked_mul(results).ok_or_else(overflow)?)
            .and_then(|bytes| {
                bytes.checked_add(
                    inputs
                        .checked_mul(max_rows.checked_add(1)?)?
                        .checked_mul(size_of::<f64>())?,
                )
            })
            .ok_or_else(overflow)?;
        bytes = bytes
            .checked_add(size_of::<CaseBatchOutcome>())
            .and_then(|bytes| {
                bytes.checked_add(
                    self.binding
                        .rows
                        .len()
                        .checked_mul(size_of::<SemanticId>())?,
                )
            })
            .ok_or_else(overflow)?;
        if bytes > self.binding.limits.worker_bytes || max_rows > self.binding.limits.native_index {
            return Err(overflow());
        }
        Ok(bytes)
    }
    /// Admit a population once, then bind checked packed workers to that real owner.
    pub fn batch_worker_scoped<E: std::fmt::Debug + std::fmt::Display>(
        self: &Arc<Self>,
        scope: ExecutionScope,
        max_rows: usize,
        admit: impl FnOnce(usize) -> Result<Arc<dyn AllocationOwner>, E>,
    ) -> Result<NumericalObservationBatchWorker, TransferError<E>> {
        scope.check().map_err(crate::error::scope_error)?;
        let bytes = self.batch_worker_bytes(max_rows)?;
        let owner = admit(bytes).map_err(TransferError::Admission)?;
        scope.check().map_err(crate::error::scope_error)?;
        let mut workers = Vec::with_capacity(self.programs.len());
        for program in &self.programs {
            let backing = program
                .template
                .batch_backing_bytes(max_rows, self.binding.order)?;
            let worker = program
                .template
                .worker_scoped(scope.clone(), |_| Ok::<_, E>(Arc::clone(&owner)))?;
            workers.push(worker.into_batch(max_rows, self.binding.order, backing)?);
        }
        let inputs = self
            .programs
            .iter()
            .map(|program| program.gather.len())
            .max()
            .unwrap_or(0);
        let results = (0..max_rows)
            .map(|_| Ok(empty_result(&self.binding)))
            .collect();
        Ok(NumericalObservationBatchWorker {
            assembly: Arc::clone(self),
            workers,
            scope,
            max_rows,
            active: Vec::with_capacity(max_rows),
            packed: Vec::with_capacity(max_rows * inputs),
            gather: vec![0.; inputs],
            results,
            started: vec![false; max_rows],
            completed: vec![false; max_rows],
            _owner: owner,
        })
    }
}

/// Private ordered population state. Packed arithmetic belongs to CheckedBatchExecution;
/// this owner performs only original scalar admission and sparse physical transport.
#[derive(Debug)]
pub struct NumericalObservationBatchWorker {
    assembly: Arc<NumericalObservationAssembly>,
    workers: Vec<CheckedBatchExecution>,
    scope: ExecutionScope,
    max_rows: usize,
    active: Vec<usize>,
    packed: Vec<f64>,
    gather: Vec<f64>,
    results: Vec<Result<Evaluation, MathError>>,
    started: Vec<bool>,
    completed: Vec<bool>,
    _owner: Arc<dyn AllocationOwner>,
}
impl NumericalObservationBatchWorker {
    /// Gather and scatter an ordered population, preserving each point's original
    /// validation/error context. Failed points do not execute later instance guards.
    /// Results borrow the preadmitted population owner and cannot escape its lease.
    pub fn evaluate(
        &mut self,
        points: &[CaseValues],
    ) -> Result<&[Result<Evaluation, MathError>], MathError> {
        self.evaluate_population(points.len(), |point| &points[point])
    }
    fn evaluate_population<'a>(
        &mut self,
        count: usize,
        point: impl Fn(usize) -> &'a CaseValues,
    ) -> Result<&[Result<Evaluation, MathError>], MathError> {
        if count > self.max_rows {
            return Err(MathError::Limit("numerical observation population"));
        }
        self.started[..count].fill(false);
        self.completed[..count].fill(false);
        self.scope.check().map_err(crate::error::scope_error)?;
        let case = &self.assembly.binding;
        for (index, result) in self.results.iter_mut().take(count).enumerate() {
            self.scope.check().map_err(crate::error::scope_error)?;
            if let Err(error) = case.inputs.validate_values(point(index)) {
                *result = Err(error);
                self.started[index] = true;
                self.completed[index] = true;
                continue;
            }
            if let Ok(value) = result {
                value.applicability.clear();
                value.values.fill(0.);
                value.jacobian.fill(0.);
                value.hessians.fill(0.);
            } else {
                *result = Ok(empty_result(case));
            }
            if self.assembly.programs.is_empty() {
                self.started[index] = true;
                self.completed[index] = true;
            }
        }
        for (i, program) in self.assembly.programs.iter().enumerate() {
            self.active.clear();
            self.packed.clear();
            let attribution = &case.instances[i].attribution;
            for index in 0..count {
                if self.results[index].is_err() {
                    continue;
                }
                self.scope.check().map_err(crate::error::scope_error)?;
                self.started[index] = true;
                match program.gather.fill(
                    Some(point(index)),
                    &[],
                    &[],
                    &mut self.gather[..program.gather.len()],
                ) {
                    Ok(()) => {
                        self.active.push(index);
                        self.packed
                            .extend_from_slice(&self.gather[..program.gather.len()]);
                    }
                    Err(error) => {
                        self.results[index] = Err(attribution.evaluation_error(error));
                        self.completed[index] = true;
                    }
                }
            }
            let active = &self.active;
            let results = &mut self.results;
            let completed = &mut self.completed;
            self.workers[i].evaluate_observing(active.len(), &self.packed, |ordinal, local| {
                let point = active[ordinal];
                match local {
                    Ok(local) => {
                        if let Ok(result) = &mut results[point] {
                            if let Err(error) = scatter(
                                program,
                                local,
                                result,
                                case.coordinates.len(),
                                case.order,
                                attribution.instance(),
                            ) {
                                results[point] = Err(attribution.evaluation_error(error));
                                completed[point] = true;
                            } else if i + 1 == self.assembly.programs.len() {
                                completed[point] = true;
                            }
                        }
                    }
                    Err(error) => {
                        completed[point] = !matches!(&error, MathError::Cancelled);
                        results[point] = Err(attribution.evaluation_error(error));
                    }
                }
            })?;
        }
        self.scope.check().map_err(crate::error::scope_error)?;
        Ok(&self.results[..count])
    }
    /// Evaluate pure Value observations in the established case-batch outcome
    /// format, moving original failures and withholding every incomplete row.
    /// The runtime must retain its independently admitted escaped-result lease.
    pub fn constraints(
        &mut self,
        points: &[CaseBatchPoint<'_>],
    ) -> Result<CaseBatchOutcome, MathError> {
        if self.assembly.binding.order != DerivativeOrder::Value {
            return Err(MathError::Contract(
                "value batch outcome requires a Value request".into(),
            ));
        }
        let canceled = match self
            .evaluate_population(points.len(), |point| points[point].values)
            .map(|_| ())
        {
            Ok(()) => false,
            Err(MathError::Cancelled) => true,
            Err(error) => return Err(error),
        };
        self.value_outcome(points, canceled)
    }
    fn value_outcome(
        &mut self,
        points: &[CaseBatchPoint<'_>],
        canceled: bool,
    ) -> Result<CaseBatchOutcome, MathError> {
        let rows = self
            .assembly
            .binding
            .rows
            .iter()
            .map(|row| row.id)
            .collect::<Vec<_>>();
        let mut cells = vec![f64::NAN; points.len() * rows.len()];
        let mut outcomes = Vec::with_capacity(points.len());
        for (index, point) in points.iter().enumerate() {
            let mut applicability = Vec::new();
            let outcome = if self.completed[index] {
                match std::mem::replace(
                    &mut self.results[index],
                    Err(MathError::Limit("observation result transferred")),
                ) {
                    Ok(value) => {
                        cells[index * rows.len()..(index + 1) * rows.len()]
                            .copy_from_slice(&value.values);
                        applicability = value.applicability;
                        PointOutcome::Succeeded
                    }
                    Err(MathError::Cancelled) => PointOutcome::Canceled,
                    Err(error) => PointOutcome::Failed(error),
                }
            } else if self.started[index] {
                PointOutcome::Canceled
            } else {
                PointOutcome::NotExecuted
            };
            outcomes.push(PointResult {
                id: point.id,
                outcome,
                applicability,
            });
        }
        CaseBatchOutcome::from_parts(rows, outcomes, cells, canceled)
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::transport_fixture;
    use super::*;
    fn id(n: u8) -> SemanticId {
        SemanticId::from_bytes([n; 16])
    }
    fn values() -> CaseValues {
        CaseValues {
            scalars: BTreeMap::from([(id(1), 1.), (id(2), 4.)]),
        }
    }
    #[test]
    fn execution_only_value_outcome_moves_failures_and_hides_partial_canceled_rows() {
        let (binding, template) = transport_fixture(false, DerivativeOrder::Value);
        let assembly = binding.assemble(vec![template]).unwrap();
        let control = pse_diagnostics::Cancellation::default();
        let mut worker = assembly
            .batch_worker_scoped(ExecutionScope::new(control.clone(), None), 4, |_| {
                Ok::<Arc<dyn AllocationOwner>, &str>(Arc::new(()))
            })
            .unwrap();
        let values = values();
        let points = (0..4)
            .map(|_| CaseBatchPoint {
                id: id(90),
                values: &values,
            })
            .collect::<Vec<_>>();
        let complete = worker.constraints(&points).unwrap();
        assert!(!complete.canceled());
        assert_eq!(complete.rows(), [id(7)]);
        assert!(complete.points().iter().all(|point| point.id == id(90)));
        for index in 0..4 {
            assert_eq!(complete.row(index), Some([256.].as_slice()));
        }
        // A finite private cell is insufficient completion evidence. Represent
        // one complete row, one interrupted row, one untouched row and a typed
        // terminal refusal without timing-dependent cancellation hooks.
        worker.results = vec![
            Ok(empty_result(&binding)),
            Ok(empty_result(&binding)),
            Ok(empty_result(&binding)),
            Err(MathError::Contract("original failure".into())),
        ];
        worker.results[0].as_mut().unwrap().values[0] = 256.;
        worker.results[1].as_mut().unwrap().values[0] = 123.;
        worker.started = vec![true, true, false, true];
        worker.completed = vec![true, false, false, true];
        let partial = worker.value_outcome(&points, true).unwrap();
        assert!(partial.canceled());
        assert_eq!(partial.row(0), Some([256.].as_slice()));
        assert!(matches!(
            partial.points()[1].outcome,
            PointOutcome::Canceled
        ));
        assert!(matches!(
            partial.points()[2].outcome,
            PointOutcome::NotExecuted
        ));
        assert!(
            matches!(&partial.points()[3].outcome,PointOutcome::Failed(MathError::Contract(message)) if message=="original failure")
        );
        for index in 1..4 {
            assert!(partial.row(index).is_none());
        }
        control.cancel();
        let canceled = worker.constraints(&points).unwrap();
        assert!(canceled.canceled());
        assert!(
            canceled
                .points()
                .iter()
                .all(|point| matches!(point.outcome, PointOutcome::NotExecuted))
        );
        assert!((0..4).all(|index| canceled.row(index).is_none()));
    }
}
