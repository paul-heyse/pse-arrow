// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit finite restart starts; KINSOL retains sole ownership of iteration/history.
use super::*;

/// Explicit restart candidates in the map's physical coordinate order.
#[derive(Clone, Debug)]
pub struct RestartSchedule {
    /// Finite screened starts; these are caller-authorized, never previous roots.
    pub starts: Vec<Vec<f64>>,
    /// Native iteration slice for the original attempt and each authorized restart.
    pub iterations: u32,
}
impl RestartSchedule {
    /// Refuse extent, coordinates and expanded work before native iteration.
    pub fn validate(
        &self,
        contract: &OracleContract,
        controls: &Controls,
    ) -> Result<(), ProblemError> {
        if self.starts.is_empty()
            || self.starts.len() > 32
            || self.iterations == 0
            || self.iterations > controls.iterations
            || self
                .starts
                .len()
                .checked_mul(contract.variables.len())
                .is_none_or(|n| n > 4096)
            || self.starts.iter().any(|point| {
                point.len() != contract.variables.len()
                    || point
                        .iter()
                        .zip(&contract.variables)
                        .any(|(value, variable)| {
                            !value.is_finite() || *value < variable.lower || *value > variable.upper
                        })
            })
        {
            return Err(ProblemError::Contract(
                "finite authorized recycle restart extent/coordinates/work".into(),
            ));
        }
        Ok(())
    }
}
fn add_work(a: WorkEvidence, b: WorkEvidence) -> WorkEvidence {
    let sum = |a: Option<u64>, b: Option<u64>| a.zip(b).and_then(|(a, b)| a.checked_add(b));
    WorkEvidence {
        evaluations: sum(a.evaluations, b.evaluations),
        iterations: sum(a.iterations, b.iterations),
        factorizations: sum(a.factorizations, b.factorizations),
        proof_steps: sum(a.proof_steps, b.proof_steps),
    }
}
fn append_history(
    total: &mut Option<RawMapHistory>,
    next: Option<std::sync::Arc<RawMapHistory>>,
    limit: usize,
) {
    let Some(next) = next else {
        return;
    };
    let total = total.get_or_insert_with(|| RawMapHistory {
        problem: next.problem,
        profile: next.profile,
        samples: Vec::with_capacity(limit),
        omitted: 0,
        last_successful: None,
        last_evaluation: None,
    });
    let offset = total
        .last_evaluation
        .as_ref()
        .map_or(0, |sample| sample.ordinal);
    let shifted = |sample: &std::sync::Arc<RawMapSample>| {
        let mut sample = (**sample).clone();
        sample.ordinal = sample.ordinal.saturating_add(offset);
        std::sync::Arc::new(sample)
    };
    for sample in &next.samples {
        if total.samples.len() < limit {
            total.samples.push(shifted(sample));
        } else {
            total.omitted = total.omitted.saturating_add(1);
        }
    }
    total.omitted = total.omitted.saturating_add(next.omitted);
    if let Some(sample) = &next.last_successful {
        total.last_successful = Some(shifted(sample));
    }
    if let Some(sample) = &next.last_evaluation {
        total.last_evaluation = Some(shifted(sample));
    }
}
impl Session {
    /// Execute only declared bounded restarts. Every slice uses the original clock,
    /// work admission and iteration remainder; a terminal scientific cause vetoes it.
    /// Pinned SUNDIALS 7.1.1 KINFP resets kin_nni to zero on each KINSol call;
    /// Anderson iteration zero overwrites gold/fold and iteration one rebuilds QR.
    /// Screening occurs before that reset, so refused starts cannot erase history.
    #[allow(
        clippy::too_many_arguments,
        reason = "restart execution preserves the original solve inputs and an explicit finite schedule"
    )]
    pub fn solve_with_restarts(
        &mut self,
        initial: &[f64],
        controls: &Controls,
        accuracy: &ResolvedAccuracy,
        execution: Execution,
        tolerances: &Tolerances,
        warm: Option<&WarmStart>,
        schedule: Option<&RestartSchedule>,
    ) -> Result<SolveReport, ProblemError> {
        let Some(schedule) = schedule else {
            return self.solve(initial, controls, accuracy, execution, tolerances, warm);
        };
        schedule.validate(self.callback.function.contract(), controls)?;
        if !matches!(self.callback.function, Function::FixedPoint(_)) || warm.is_some() {
            return Err(ProblemError::Contract(
                "declared restarts require a frozen fixed-point map and authored start".into(),
            ));
        }
        let mut remaining = controls.iterations;
        let mut total = WorkEvidence {
            evaluations: Some(0),
            iterations: Some(0),
            factorizations: Some(0),
            proof_steps: Some(0),
        };
        let mut history = None;
        let mut starts = std::iter::once(initial).chain(schedule.starts.iter().map(Vec::as_slice));
        let mut attempt = 0u64;
        let mut report: Option<SolveReport> = None;
        while let Some(start) = starts.next() {
            if let Err(cause) = execution.check() {
                if let Some(report) = &mut report {
                    report.termination.category =
                        execution.stopped().unwrap_or(Termination::Cancelled);
                    report.record_validation_failure(cause);
                    break;
                }
                return Err(cause);
            }
            if remaining == 0 {
                break;
            }
            if attempt > 0 {
                let Function::FixedPoint(oracle) = &mut self.callback.function else {
                    return Err(ProblemError::Internal(
                        "recycle representation changed".into(),
                    ));
                };
                let mut mapped = vec![0.0; start.len()];
                let entered = std::cell::Cell::new(false);
                let screened = execution.counted(
                    WorkEvidence {
                        evaluations: Some(1),
                        iterations: Some(0),
                        factorizations: Some(0),
                        proof_steps: Some(0),
                    },
                    || {
                        entered.set(true);
                        crate::quality::contained(|| oracle.map(start, &mut mapped))?;
                        if mapped.iter().all(|value| value.is_finite()) {
                            Ok(())
                        } else {
                            Err(ProblemError::numerical(
                                "nonfinite screened recycle restart",
                            ))
                        }
                    },
                );
                if !entered.get() {
                    if let Some(report) = &mut report {
                        if let Some(stop) = execution.stopped() {
                            report.termination.category = stop;
                        }
                        if let Err(cause) = screened {
                            report.record_validation_failure(cause);
                        }
                    }
                    break;
                }
                let retained = history
                    .as_ref()
                    .map_or(0, |history: &RawMapHistory| history.samples.len());
                let mut capture = RawMapCapture::new(
                    oracle.contract(),
                    self.compatibility.profile,
                    controls.history.saturating_sub(retained),
                    oracle.physical_tear_coordinates(),
                    &tolerances.rows,
                )?
                .with_inner_context(oracle.inner_context());
                capture.record(
                    oracle.contract(),
                    start,
                    screened.is_ok().then_some(mapped.as_slice()),
                    RawMapRole::RestartScreening,
                    screened.as_ref().err().map(raw_map_failure),
                );
                append_history(&mut history, Some(capture.finish()), controls.history);
                total = add_work(
                    total,
                    WorkEvidence {
                        evaluations: Some(1),
                        iterations: Some(0),
                        factorizations: Some(0),
                        proof_steps: Some(0),
                    },
                );
                if let Err(cause) = screened {
                    if let Some(report) = &mut report {
                        report.record_validation_failure(cause);
                    }
                    break;
                }
            }
            let mut slice = controls.clone();
            slice.iterations = remaining.min(schedule.iterations);
            slice.history = controls.history.saturating_sub(
                history
                    .as_ref()
                    .map_or(0, |history: &RawMapHistory| history.samples.len()),
            );
            let mut current =
                self.solve(start, &slice, accuracy, execution.clone(), tolerances, None)?;
            attempt += 1;
            let consumed = current
                .evidence
                .work
                .iterations
                .and_then(|n| u32::try_from(n).ok())
                .ok_or_else(|| {
                    ProblemError::Unsupported(
                        "recycle restart iteration coverage unavailable".into(),
                    )
                })?;
            remaining = remaining
                .checked_sub(consumed)
                .ok_or_else(|| ProblemError::Limit {
                    kind: crate::LimitKind::Work,
                    detail: "aggregate recycle iteration cap exceeded".into(),
                })?;
            total = add_work(total, current.evidence.work);
            append_history(
                &mut history,
                current.evidence.raw_map.take(),
                controls.history,
            );
            let terminal = current.evidence.callback.terminal_failure
                || current.evidence.abandoned.is_some()
                || current.shared_effective_failure().is_some_and(|cause| {
                    !matches!(
                        crate::callback::classify(&cause),
                        crate::callback::Failure::Trial
                    )
                });
            let accepted = current.validation_failure().is_none()
                && current.quality.as_ref().is_some_and(Quality::feasible);
            let restartable = matches!(
                current.termination.category,
                Termination::IterationLimit
                    | Termination::Limit
                    | Termination::Success
                    | Termination::Acceptable
            );
            report = Some(current);
            if terminal || accepted || !restartable {
                break;
            }
        }
        let mut report = report.ok_or_else(|| {
            ProblemError::Internal("recycle restart schedule executed no native attempt".into())
        })?;
        report.evidence.work = total;
        report.evidence.raw_map = history.map(std::sync::Arc::new);
        report.metrics.insert(
            "recycle.native_attempts".into(),
            Metric::Integer(i64::try_from(attempt).unwrap_or(i64::MAX)),
        );
        report.metrics.insert(
            "recycle.iteration_allowance_remaining".into(),
            Metric::Integer(i64::from(remaining)),
        );
        if let Some(stop) = execution.stopped() {
            report.termination.category = stop;
            if let Err(cause) = execution.check() {
                report.record_validation_failure(cause);
            }
        }
        Ok(report)
    }
}
