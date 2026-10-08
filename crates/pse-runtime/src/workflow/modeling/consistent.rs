// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! IC-only execution of the original authored dynamic supplier. This module is a
//! child of `dynamics`; shared scientific completion remains a separate owner.
use super::*;
use crate::math::{MathRuntimeError, Submission, solves::SolveHandle};
use pse_backend_native::solve::{Controls, Execution, Progress};
use pse_columnar::flight::FlightCancellation;
use pse_kernels::ExecutionScope;

/// Joined IC observations whose buffers and diagnostics retain their original reservation.
#[derive(Clone, Debug)]
pub struct ConsistentInitializationResult {
    inner: Arc<OwnedConsistent>,
}
#[derive(Debug)]
struct OwnedConsistent {
    observed: ConsistentObservation,
    _owner: Arc<pse_columnar::AllocationLease>,
}
/// Original observations passed to the shared completion owner, without a use decision.
#[derive(Debug)]
pub(crate) struct ConsistentObservation {
    pub(crate) report: native::ConsistentStateReport,
    pub(crate) checks: Vec<ModelingCheck>,
    pub(crate) checks_complete: bool,
    scope: ExecutionScope,
    strategy: Option<Arc<crate::math::strategy::Trace>>,
}
impl ConsistentObservation {
    /// Reject late publication after thread-local destruction and join, retaining the
    /// actual native code and observed work as diagnostics.
    pub(crate) fn check_after_join(&mut self) {
        if let Err(error) = self.scope.check() {
            self.report.termination.category =
                if matches!(error, pse_kernels::ProviderError::Cancelled) {
                    pse_backend_native::solve::Termination::Cancelled
                } else {
                    pse_backend_native::solve::Termination::TimeLimit
                };
            self.report.evidence.callback.terminal_failure = true;
            self.report.assessment = None;
            self.checks_complete = false;
            let error = ProblemError::Provider(error);
            if self.report.error.is_none() {
                self.report.error = Some(error);
            } else if self.report.validation_error.is_none() {
                self.report.validation_error = Some(error);
            }
            // This trace is still private until joining publishes OwnedConsistent.
            // Update existing fields only: no new unreserved diagnostic/vector payload.
            if let Some(trace) = self.strategy.as_mut().and_then(Arc::get_mut)
                && let Some(event) = trace.events.last_mut()
            {
                event.kind = pse_model::generated::enums::NumericalEventKind::Abandoned;
                event.original = None;
                event.decision = None;
                event.observation = Some(
                    if self.report.termination.category
                        == pse_backend_native::solve::Termination::Cancelled
                    {
                        pse_model::generated::enums::NumericalAttemptObservation::Cancelled
                    } else {
                        pse_model::generated::enums::NumericalAttemptObservation::ResourceExhausted
                    },
                );
                event.transition = Some(pse_model::strategy::Transition::Stop);
                event.permission = Some(pse_model::generated::enums::CandidateUse::Unusable);
            }
        }
    }
}
impl ConsistentInitializationResult {
    /// Actual numerical decisions; IC observations confer no scientific use permission.
    pub fn strategy(&self) -> Option<&Arc<crate::math::strategy::Trace>> {
        self.inner.observed.strategy.as_ref()
    }
    /// Actual native termination, work and original residual/role assessment.
    pub fn report(&self) -> &native::ConsistentStateReport {
        &self.inner.observed.report
    }
    /// Original physical initial equations, projected through their admitted scale/offset.
    pub fn checks(&self) -> &[ModelingCheck] {
        &self.inner.observed.checks
    }
    /// Every authored initial equation was checked after a valid nonterminal assessment.
    /// This flag grants no scientific permission.
    pub fn checks_complete(&self) -> bool {
        self.inner.observed.checks_complete
    }
}
type ConsistentHandle = SolveHandle<(ConsistentObservation, Arc<pse_columnar::AllocationLease>)>;
fn admit_controls(controls: &Controls) -> Result<(), ProblemError> {
    controls.validate()?;
    let defaults = Controls::default();
    if controls.threads != 1
        || controls.iterations != defaults.iterations
        || controls.hessian != defaults.hessian
        || controls.reuse != defaults.reuse
        || controls.start != defaults.start
        || !controls.options.is_empty()
    {
        return Err(ProblemError::Unsupported("IDAS IC consumes its explicit IC controls; generic method/iteration/Hessian/start/reuse options cannot act on this serial fresh attempt".into()));
    }
    Ok(())
}
fn original_initial_checks(
    run_id: RunId,
    time: f64,
    state: &[f64],
    coordinates: &[CoordinateBinding],
    obligations: &[OriginalInitialCondition],
    scope: &ExecutionScope,
) -> Result<Vec<ModelingCheck>, ProblemError> {
    scope.check()?;
    if state.len() != coordinates.len() || state.iter().any(|v| !v.is_finite()) {
        return Err(ProblemError::Contract(
            "IC original state extent or nonfinite coordinate".into(),
        ));
    }
    let mut checks = Vec::with_capacity(obligations.len());
    for obligation in obligations {
        scope.check()?;
        let coordinate = coordinates.get(obligation.coordinate).ok_or_else(|| {
            ProblemError::Contract("IC original initial coordinate absent".into())
        })?;
        if !coordinate.scale.is_finite()
            || coordinate.scale <= 0.0
            || !coordinate.offset.is_finite()
        {
            return Err(ProblemError::Contract(
                "IC original physical coordinate admission".into(),
            ));
        }
        let residual = state[obligation.coordinate] * coordinate.scale + coordinate.offset
            - obligation.expected;
        if !residual.is_finite() {
            return Err(ProblemError::numerical(
                "IC nonfinite original physical initial residual",
            ));
        }
        checks.push(results::temporal_initial_check(
            run_id,
            obligation.source,
            obligation.row,
            time,
            residual,
            obligation.tolerance,
        ));
    }
    scope.check()?;
    Ok(checks)
}
fn classify(
    observed: &ConsistentObservation,
) -> pse_model::generated::enums::NumericalAttemptObservation {
    use pse_backend_native::solve::Termination as T;
    use pse_model::generated::enums::NumericalAttemptObservation as O;
    let report = &observed.report;
    if let Some(abandoned) = report.evidence.abandoned {
        return abandoned;
    }
    if let Some(error) = report.validation_error.as_ref().or(report.error.as_ref()) {
        return crate::math::strategy::failure(error);
    }
    match report.termination.category {
        T::Success | T::Acceptable => O::Converged,
        T::Cancelled => O::Cancelled,
        T::TimeLimit | T::ResourceExhausted => O::ResourceExhausted,
        T::Numerical | T::Evaluation => O::NumericalFailure,
        T::Panic => O::Panic,
        T::Invalid => O::ContractFailure,
        _ => O::Limited,
    }
}
fn assess_consistent(
    observed: &ConsistentObservation,
    observation: pse_model::generated::enums::NumericalAttemptObservation,
) -> crate::math::strategy::Assessment {
    use pse_model::diagnostic::DiagnosticProjection;
    use pse_model::generated::enums::{CandidateRefusal, CandidateUse};
    let original_failure = observed
        .report
        .validation_error
        .as_ref()
        .or(observed.report.error.as_ref())
        .map(|error| {
            crate::math::strategy::target::assessment_failure(
                &error.boundary_diagnostic(pse_diagnostics::DiagnosticStage::Native),
            )
        });
    let satisfied = original_failure.is_none()
        && !observed.report.evidence.callback.terminal_failure
        && observed
            .report
            .assessment
            .as_ref()
            .is_some_and(|assessment| {
                assessment.residual_satisfied
                    && assessment.roles_preserved
                    && assessment.signs_satisfied
            })
        && observed.checks_complete
        && observed.checks.iter().all(|check| check.satisfied);
    let decision = if satisfied {
        crate::workflow::numerics::CandidateDecision {
            usability: CandidateUse::SeedOnly,
            qualifiers: Vec::new(),
            refusals: Vec::new(),
            bound: None,
        }
    } else {
        crate::workflow::numerics::refused(CandidateRefusal::ValidationFailed)
    };
    let mut assessment = crate::math::strategy::target::original_assessment(
        decision,
        original_failure,
        None,
        observation,
    );
    if satisfied {
        assessment.original = crate::math::strategy::OriginalConclusion::Satisfied;
    }
    assessment
}
fn profile_identity(
    request: &native::ConsistentInitialization,
    controls: &Controls,
) -> Result<ContentHash, ProblemError> {
    let mut identity = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
    identity
        .str("idas-consistent-callable-profile")
        .hash(&controls.identity()?);
    pse_ids::document::frame(
        &mut identity,
        &(
            request.time,
            request.toward,
            request.rtol,
            &request.atol,
            &request.residual_tolerances,
            request.mode,
            &request.linear,
            &request.controls,
            request.trial_failures,
        ),
    )
    .map_err(|error| ProblemError::Contract(error.to_string()))?;
    Ok(identity.finish_hash())
}
impl ModelingSimulation {
    /// Execute only authored consistent initialization at the declared initial time.
    /// The supplied scope and cancellation share one original task clock and owner;
    /// dropping this future requests cancellation while the supervisor retains teardown.
    /// Returned observations require the shared original assessment before scientific use.
    /// # Errors
    /// Wrong source time/owner, inactive generic controls, bounded admission failure,
    /// cancellation, or failure to join the actual native operation.
    pub async fn initialize_consistent(
        &self,
        run_id: RunId,
        request: native::ConsistentInitialization,
        controls: Controls,
        scope: ExecutionScope,
        cancellation: FlightCancellation,
    ) -> Result<ConsistentInitializationResult, WorkflowError> {
        let submission = Submission {
            cancel: cancellation,
            progress: Arc::new(Progress::new(controls.history)),
            admitted: None,
            deadline: scope.deadline(),
        };
        let handle = self.submit_consistent(run_id, request, controls, scope, submission)?;
        let (mut observed, owner) = handle.finish().await?;
        observed.check_after_join();
        Ok(ConsistentInitializationResult {
            inner: Arc::new(OwnedConsistent {
                observed,
                _owner: owner,
            }),
        })
    }
    pub(crate) fn submit_consistent(
        &self,
        run_id: RunId,
        request: native::ConsistentInitialization,
        controls: Controls,
        scope: ExecutionScope,
        submission: Submission,
    ) -> Result<ConsistentHandle, WorkflowError> {
        admit_controls(&controls).map_err(MathRuntimeError::from)?;
        scope
            .check()
            .map_err(ProblemError::Provider)
            .map_err(MathRuntimeError::from)?;
        if !Arc::ptr_eq(scope.cancellation(), &submission.cancel.flag())
            || scope.deadline().is_none()
            || submission.deadline != scope.deadline()
        {
            return Err(contract(
                "IC admission must preserve the original finite task clock and cancellation owner",
            ));
        }
        if request.time.to_bits() != self.profile.start.to_bits() {
            return Err(contract(
                "authored initial obligations apply only at the original profile start",
            ));
        }
        self.contract.validate().map_err(MathRuntimeError::from)?;
        let foreign = self
            .runtime
            .shared
            .budget()
            .math
            .foreign_allowance(&controls);
        if foreign == 0 {
            return Err(contract("IC requires a finite positive foreign allowance"));
        }
        // The shared job owner already reserves deployment foreign storage. Explicit
        // foreign storage is additionally reserved exactly as for an algebraic step.
        let bytes = self
            .bytes
            .checked_add(controls.foreign_bytes.unwrap_or(0))
            .and_then(|n| {
                n.checked_add(
                    self.modes[0]
                        .original_initial_conditions
                        .len()
                        .checked_mul(size_of::<ModelingCheck>())?,
                )
            })
            .ok_or(MathRuntimeError::Limit("IC worker/residual extent"))?;
        let prepared = self.clone();
        Ok(self.runtime.shared.math().submit_with(
            1,
            bytes,
            submission,
            move |flag, progress| {
                let composition = pse_model::strategy::CompositionRequest::default();
                let (mut observed, trace) = crate::math::strategy::target::callable(
                    prepared.runtime.shared.math(),
                    crate::math::strategy::target::Source {
                        original: prepared.contract.identity,
                        preparation: prepared.contract.identity,
                        profile: profile_identity(&request, &controls)?,
                        backend: None,
                        solver: None,
                        controls: &controls,
                        request: &composition,
                        start: pse_model::strategy::StartOrigin::Specification,
                        start_identity: None,
                    },
                    &scope,
                    |_, admission| {
                        let mut execution =
                            Execution::within(flag.clone(), &controls, scope.clone())?;
                        execution.work_admission = Some(admission);
                        execution.memory = Some(foreign);
                        execution.progress = progress.clone();
                        let attempt_scope = execution.scope()?;
                        let parameters = prepared
                            .profile
                            .parameters_at(&prepared.parameters, request.time);
                        let mut worker = prepared.worker(attempt_scope.clone())?;
                        let mut report = native::initialize_consistent(
                            &mut worker,
                            &parameters,
                            &request,
                            execution.clone(),
                        )?;
                        // No callback or new mathematical assessment follows a terminal latch.
                        // The native IC owner has already assessed the actual original RHS/guard,
                        // residual, signs and state roles. These additional rows are physical
                        // authored initial obligations, not replacement residuals.
                        let can_check = !report.evidence.callback.terminal_failure
                            && report.validation_error.is_none()
                            && report.assessment.is_some();
                        let checks = if can_check {
                            match report
                                .candidate
                                .as_ref()
                                .map(|point| {
                                    original_initial_checks(
                                        run_id,
                                        request.time,
                                        &point.state,
                                        &prepared.coordinates.state,
                                        &prepared.modes[0].original_initial_conditions,
                                        &attempt_scope,
                                    )
                                })
                                .transpose()
                            {
                                Ok(checks) => checks.unwrap_or_default(),
                                Err(error) => {
                                    report.validation_error = Some(error);
                                    vec![]
                                }
                            }
                        } else {
                            vec![]
                        };
                        let checks_complete = can_check
                            && report.validation_error.is_none()
                            && report.candidate.is_some()
                            && checks.len() == prepared.modes[0].original_initial_conditions.len();
                        drop(worker);
                        let mut observed = ConsistentObservation {
                            report,
                            checks,
                            checks_complete,
                            scope: attempt_scope,
                            strategy: None,
                        };
                        observed.check_after_join();
                        Ok(observed)
                    },
                    |_| None,
                    classify,
                    assess_consistent,
                )?;
                observed.strategy = Some(trace);
                let report = &observed.report;
                let checks = &observed.checks;
                // The report and errors are inline in OwnedConsistent; their owned
                // payload estimators include those inline sizes, so charge them once.
                let error_payload = |error: &ProblemError| {
                    error
                        .retained_bytes()
                        .saturating_sub(size_of::<ProblemError>())
                };
                let retained = report
                    .numeric_bytes()
                    .saturating_sub(size_of::<native::ConsistentStateReport>())
                    .checked_add(checks.capacity() * size_of::<ModelingCheck>())
                    .and_then(|n| n.checked_add(report.error.as_ref().map_or(0, error_payload)))
                    .and_then(|n| {
                        n.checked_add(report.validation_error.as_ref().map_or(0, error_payload))
                    })
                    .and_then(|n| n.checked_add(report.termination.name.capacity()))
                    .and_then(|n| {
                        n.checked_add(size_of::<OwnedConsistent>() + 2 * size_of::<usize>())
                    })
                    .ok_or(MathRuntimeError::Limit("IC retained observation extent"))?;
                Ok((observed, retained))
            },
        )?)
    }
}

#[cfg(test)]
#[path = "consistent_tests.rs"]
mod tests;
