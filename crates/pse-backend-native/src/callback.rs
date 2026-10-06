// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One typed failure policy for all native trial evaluators.
use crate::{
    ProblemError,
    solve::{Event, Execution, Metric, Termination},
};
use std::{
    collections::BTreeMap,
    panic::{AssertUnwindSafe, catch_unwind},
};
/// A failed trial can be recoverable without making the entire attempt fail.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    /// The native method may select another trial.
    Trial,
    /// Abort evaluation; never substitute a previous value.
    Fatal,
    /// A cooperative stop: cancellation or an exhausted deadline, never an evaluation failure.
    Stopped(Termination),
}
/// Classify typed causes, never native diagnostic strings.
pub fn classify(error: &ProblemError) -> Failure {
    fn provider(error: &pse_kernels::ProviderError) -> Failure {
        use pse_kernels::ProviderError as E;
        match error {
            E::Cancelled => Failure::Stopped(Termination::Cancelled),
            E::Deadline => Failure::Stopped(Termination::TimeLimit),
            E::Nested { cause, .. } => {
                use pse_diagnostics::{DiagnosticCode, TypedDiagnostic};
                match cause.diagnostic_code() {
                    Some(DiagnosticCode::RuntimeTimeout) => {
                        Failure::Stopped(Termination::TimeLimit)
                    }
                    Some(DiagnosticCode::RuntimeCancelled) => {
                        Failure::Stopped(Termination::Cancelled)
                    }
                    _ if error.recoverable() => Failure::Trial,
                    _ => Failure::Fatal,
                }
            }
            _ if error.recoverable() => Failure::Trial,
            _ => Failure::Fatal,
        }
    }
    fn math(error: &pse_math::MathError) -> Failure {
        use pse_math::MathError as E;
        match error {
            E::Instance { cause, .. } => math(cause),
            E::Applicability(_) | E::Validity(_) | E::Domain { .. } | E::OutsideRange { .. } => {
                Failure::Trial
            }
            E::Cancelled => Failure::Stopped(Termination::Cancelled),
            E::Provider { cause, .. } | E::Scope(cause) => provider(cause),
            E::Native { cause, .. } | E::Typed { cause, .. } => cause
                .as_error()
                .downcast_ref::<ProblemError>()
                .map_or(Failure::Fatal, classify),
            E::Contract(_)
            | E::DerivativeDemand { .. }
            | E::CoefficientRange
            | E::Library(_)
            | E::Evaluation { .. }
            | E::Limit(_)
            | E::Refinement { .. }
            | E::ByteLimit { .. }
            | E::SlotLimit { .. }
            | E::WorkLimit { .. }
            | E::Quantity(_) => Failure::Fatal,
        }
    }
    match error {
        ProblemError::Math(e) => math(e),
        ProblemError::Provider(e) => provider(e),
        ProblemError::Cancelled => Failure::Stopped(Termination::Cancelled),
        ProblemError::Limit {
            kind: crate::LimitKind::Time,
            ..
        } => Failure::Stopped(Termination::TimeLimit),
        ProblemError::Limit {
            kind: crate::LimitKind::Work,
            ..
        } => Failure::Stopped(Termination::Limit),
        ProblemError::Native {
            kind: crate::NativeFailureKind::Resource,
            ..
        } => Failure::Stopped(Termination::ResourceExhausted),
        ProblemError::Native { .. } => Failure::Fatal,
        ProblemError::Linear {
            kind: crate::LinearFailureKind::Memory,
            ..
        } => Failure::Stopped(Termination::ResourceExhausted),
        ProblemError::Linear { .. } => Failure::Fatal,
        ProblemError::DynamicRouteRefused(_)
        | ProblemError::RouteRefused(_)
        | ProblemError::Unavailable { .. }
        | ProblemError::Contract(_)
        | ProblemError::Structural { .. }
        | ProblemError::Unsupported(_)
        | ProblemError::Reuse { .. }
        | ProblemError::Numerical { .. }
        | ProblemError::Limit { .. }
        | ProblemError::Internal(_) => Failure::Fatal,
    }
}
/// Borrowed original oracle used only for fresh validation/analysis outside TNLP callbacks.
/// Native callbacks already have CallbackState; wrapping those would charge twice.
#[derive(Debug)]
pub(crate) struct CountedNlp<'a> {
    pub(crate) oracle: &'a mut dyn crate::NlpOracle,
    pub(crate) execution: &'a Execution,
}
impl crate::NlpOracle for CountedNlp<'_> {
    fn structural_analysis(&self) -> Option<&pse_structural::incidence::StructuralAnalysis> {
        self.oracle.structural_analysis()
    }
    fn normalization(&self) -> Option<&pse_math::normalization::Normalization> {
        self.oracle.normalization()
    }
    fn solve_separator(&self) -> Option<&crate::SolveSeparator> {
        self.oracle.solve_separator()
    }
    fn constraint_sources(&self) -> Result<Vec<pse_math::assembly::OutputValue>, ProblemError> {
        self.oracle.constraint_sources()
    }
    fn presolve_facts(&self) -> Option<&pse_math::presolve::Facts> {
        self.oracle.presolve_facts()
    }
    fn derivative_facts(&self) -> crate::DerivativeFacts {
        self.oracle.derivative_facts()
    }
    fn contract(&self) -> &crate::OracleContract {
        self.oracle.contract()
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.oracle.jacobian_pattern()
    }
    fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
        self.oracle.hessian_pattern()
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        self.oracle.constraint_bounds()
    }
    fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
        self.execution
            .counted(evaluation_unit(), || self.oracle.objective(x))
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.execution
            .counted(evaluation_unit(), || self.oracle.constraints(x, out))
    }
    fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.execution
            .counted(evaluation_unit(), || self.oracle.gradient(x, out))
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.execution
            .counted(evaluation_unit(), || self.oracle.jacobian(x, out))
    }
    fn hessian(
        &mut self,
        x: &[f64],
        weight: f64,
        multipliers: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        self.execution.counted(evaluation_unit(), || {
            self.oracle.hessian(x, weight, multipliers, out)
        })
    }
}
pub(crate) const fn evaluation_unit() -> crate::solve::WorkEvidence {
    crate::solve::WorkEvidence {
        evaluations: Some(1),
        iterations: Some(0),
        factorizations: Some(0),
        proof_steps: Some(0),
    }
}
/// Worker-local failure state; recoverable history never becomes a terminal latch.
#[derive(Debug)]
pub struct CallbackState {
    /// Rejected evaluations, including recoverable trials. Diagnostic consumers must
    /// not interpret a partially evaluated library check as successful evidence.
    pub rejected_evaluations: usize,
    /// Recoverable trial refusals, retained even when the event history is disabled.
    pub trial_rejections: usize,
    /// Cause of the latest failed callback, retained through the native attempt.
    /// A later callback cannot establish recovery of a partially built FD matrix;
    /// successful native completion clears this diagnostic without latching a stop.
    /// A wrapper which aborts immediately can return the original typed witness.
    pub last_failure: Option<ProblemError>,
    /// Shared stop/progress controls.
    pub execution: Execution,
    /// Only terminal failures latch. Successful later trials preserve native success.
    pub terminal: Option<(Termination, String)>,
    /// Per-demand call count, including rejected trials.
    pub counts: BTreeMap<String, i64>,
    /// Per-demand wall seconds, including rejected trials.
    pub seconds: BTreeMap<String, f64>,
    /// Trials refused for crossing a nested implicit stage's bound regime.
    pub regime_crossings: usize,
    /// Of those, the crossings since the adapter last reported a completed outer iteration.
    pub iteration_crossings: usize,
}
impl CallbackState {
    /// Start a worker-local callback boundary.
    pub fn new(execution: Execution) -> Self {
        Self {
            rejected_evaluations: 0,
            trial_rejections: 0,
            last_failure: None,
            execution,
            terminal: None,
            counts: BTreeMap::new(),
            seconds: BTreeMap::new(),
            regime_crossings: 0,
            iteration_crossings: 0,
        }
    }
    /// The regime crossings of the outer iteration an adapter reports complete, recorded in
    /// that iteration's progress event; the next iteration counts afresh.
    pub fn complete_iteration(&mut self) -> usize {
        std::mem::take(&mut self.iteration_crossings)
    }
    /// Catch Rust unwinds and record typed errors. Outputs are published by the caller
    /// only after this returns `Some`, so a failed trial cannot publish partial buffers.
    pub fn evaluate<T>(
        &mut self,
        demand: &str,
        work: impl FnOnce() -> Result<T, ProblemError>,
    ) -> Option<T> {
        if self.terminal.is_some() {
            return None;
        }
        if let Some(stop) = self.execution.stopped() {
            self.last_failure = None;
            self.terminal = Some((stop, "execution checkpoint".into()));
            return None;
        }
        let unit = crate::solve::WorkEvidence {
            evaluations: Some(1),
            iterations: Some(0),
            factorizations: Some(0),
            proof_steps: Some(0),
        };
        if let Some(admission) = &self.execution.work_admission
            && self.execution.callback_work_owner
        {
            match catch_unwind(AssertUnwindSafe(|| admission.admit(unit))) {
                Ok(Ok(())) => {}
                Ok(Err(error)) => {
                    self.terminal = Some((
                        match classify(&error) {
                            Failure::Stopped(stop) => stop,
                            _ => Termination::Evaluation,
                        },
                        error.to_string(),
                    ));
                    self.last_failure = Some(error);
                    return None;
                }
                Err(_) => {
                    self.terminal = Some((Termination::Panic, "panic in work admission".into()));
                    return None;
                }
            }
        }
        let start = std::time::Instant::now();
        let result = catch_unwind(AssertUnwindSafe(work));
        if let Some(admission) = &self.execution.work_admission
            && self.execution.callback_work_owner
        {
            match catch_unwind(AssertUnwindSafe(|| admission.observe(unit))) {
                Ok(Ok(())) => {}
                Ok(Err(error)) => {
                    self.terminal = Some((
                        match classify(&error) {
                            Failure::Stopped(stop) => stop,
                            _ => Termination::Evaluation,
                        },
                        error.to_string(),
                    ));
                    self.last_failure = Some(error);
                }
                Err(_) => {
                    self.terminal = Some((Termination::Panic, "panic in work observation".into()));
                }
            }
        }
        *self.counts.entry(demand.into()).or_default() += 1;
        *self.seconds.entry(demand.into()).or_default() += start.elapsed().as_secs_f64();
        if self.terminal.is_some() {
            self.rejected_evaluations = self.rejected_evaluations.saturating_add(1);
            return None;
        }
        let (failure, message) = match result {
            Ok(Ok(value)) => {
                if let Some(stop) = self.execution.stopped() {
                    self.rejected_evaluations = self.rejected_evaluations.saturating_add(1);
                    self.last_failure = self.execution.check().err();
                    self.terminal = Some((stop, "execution checkpoint after callback".into()));
                    return None;
                }
                return Some(value);
            }
            Ok(Err(e)) => {
                if regime_crossing(&e) {
                    self.regime_crossings = self.regime_crossings.saturating_add(1);
                    self.iteration_crossings = self.iteration_crossings.saturating_add(1);
                }
                let failure = classify(&e);
                let message = e.to_string();
                self.last_failure = Some(e);
                (failure, message)
            }
            Err(_) => {
                self.last_failure = None;
                self.rejected_evaluations = self.rejected_evaluations.saturating_add(1);
                self.terminal = Some((Termination::Panic, "panic in native callback".into()));
                return None;
            }
        };
        self.rejected_evaluations = self.rejected_evaluations.saturating_add(1);
        if failure == Failure::Trial {
            self.trial_rejections = self.trial_rejections.saturating_add(1);
        }
        self.execution.progress.push(Event {
            phase: format!("{demand}.failure"),
            elapsed: self.execution.started.elapsed(),
            values: BTreeMap::from([
                ("message".into(), Metric::Text(message.clone())),
                (
                    "recoverable".into(),
                    Metric::Bool(failure == Failure::Trial),
                ),
            ]),
            incumbent: None,
        });
        match failure {
            Failure::Trial => {}
            Failure::Stopped(stop) => {
                self.terminal = Some((self.execution.stopped().unwrap_or(stop), message));
            }
            Failure::Fatal => self.terminal = Some((Termination::Evaluation, message)),
        }
        None
    }
    /// Typed cause of the latched terminal stop, consumed once. Evaluation stops return
    /// the callback's own witness; checkpoint stops and panics return their typed class.
    pub fn terminal_error(&mut self) -> Option<ProblemError> {
        let (kind, message) = self.terminal.as_ref()?;
        if let Some(cause) = self.last_failure.take() {
            return Some(cause);
        }
        Some(match kind {
            Termination::Evaluation => ProblemError::internal(message.clone()),
            Termination::Cancelled | Termination::TimeLimit => {
                ProblemError::stopped(*kind, message.clone())
            }
            Termination::Limit => ProblemError::Limit {
                kind: crate::LimitKind::Work,
                detail: message.clone(),
            },
            _ => ProblemError::internal(message.clone()),
        })
    }
    /// Append callback measurements and preserve native status alongside terminal cause.
    pub fn finish(&mut self, report: &mut crate::solve::SolveReport) {
        if self.terminal.is_none()
            && let Some(stop) = self.execution.stopped()
        {
            self.last_failure = self.execution.check().err();
            self.terminal = Some((stop, "execution checkpoint after native work".into()));
        }
        report.evidence.abandoned = self.execution.abandonment.observation();
        report.evidence.work.evaluations = self.counts.values().try_fold(0_u64, |total, count| {
            total.checked_add(u64::try_from(*count).ok()?)
        });
        report.evidence.callback = crate::solve::CallbackEvidence {
            trial_rejections: self.trial_rejections,
            regime_crossings: self.regime_crossings,
            terminal_failure: self.terminal.is_some(),
        };
        report.metrics.insert(
            "callback.trial_rejections".into(),
            Metric::Integer(self.trial_rejections.try_into().unwrap_or(i64::MAX)),
        );
        report.metrics.insert(
            "callback.regime_crossings".into(),
            Metric::Integer(self.regime_crossings.try_into().unwrap_or(i64::MAX)),
        );
        report.metrics.insert(
            "callback.terminal_failure".into(),
            Metric::Bool(self.terminal.is_some()),
        );
        for (name, count) in &self.counts {
            report
                .metrics
                .insert(format!("callback.{name}.calls"), Metric::Integer(*count));
        }
        for (name, seconds) in &self.seconds {
            report
                .metrics
                .insert(format!("callback.{name}.seconds"), Metric::Real(*seconds));
        }
        if let Some((kind, message)) = &self.terminal {
            report.termination.category = *kind;
            report.termination.assurance = crate::solve::Assurance::None;
            report.termination.message = Some(message.clone());
        }
        self.retain_failure(report);
        (report.events, report.dropped_events) = self.execution.progress.snapshot();
    }
    /// Retain the actual native-exit witness before independent validation can
    /// replace the callback state. Successful recovered trials have no failure
    /// receipt. Measurements remain finalized by `finish` after validation.
    pub(crate) fn retain_failure(&mut self, report: &mut crate::solve::SolveReport) {
        if self.terminal.is_some()
            || matches!(
                report.termination.category,
                Termination::Evaluation | Termination::Panic
            )
        {
            if let Some(cause) = self.last_failure.take() {
                report.callback_failure = Some(std::sync::Arc::new(cause));
            }
        } else {
            report.callback_failure = None;
            self.last_failure = None;
        }
    }
}
/// A trial refused for crossing a nested implicit stage's bound regime, typed through its
/// provider and instance wrappers.
fn regime_crossing(error: &ProblemError) -> bool {
    fn math(error: &pse_math::MathError) -> bool {
        use pse_math::MathError as E;
        match error {
            E::Instance { cause, .. } => math(cause),
            E::Provider { cause, .. } => {
                matches!(cause, pse_kernels::ProviderError::RegimeCrossing { .. })
            }
            E::Native { cause, .. } | E::Typed { cause, .. } => cause
                .as_error()
                .downcast_ref::<ProblemError>()
                .is_some_and(regime_crossing),
            _ => false,
        }
    }
    match error {
        ProblemError::Math(e) => math(e),
        ProblemError::Provider(e) => matches!(e, pse_kernels::ProviderError::RegimeCrossing { .. }),
        _ => false,
    }
}
/// Native evaluation stops may be retried only with positive recoverability evidence.
/// This does not depend on bounded progress events or parse human diagnostic messages.
pub fn retryable_evaluation(report: &crate::solve::SolveReport) -> bool {
    let evidence = report.evidence.callback;
    report.termination.category == Termination::Evaluation
        && evidence.trial_rejections > 0
        && !evidence.terminal_failure
}
#[cfg(test)]
mod tests {
    use super::*;
    fn report(execution: &Execution) -> crate::solve::SolveReport {
        use crate::solve::{Assurance, Backend, NativeTermination, SolveReport};
        SolveReport::new(
            Backend::Kinsol,
            &crate::OracleContract {
                identity: pse_ids::ContentHash::from_bytes([0; 32]),
                variables: vec![],
                rows: vec![],
                derivatives: pse_kernels::DerivativeOrder::First,
                smoothness: pse_kernels::DerivativeOrder::First,
            },
            NativeTermination {
                code: 0,
                name: "success".into(),
                message: None,
                category: Termination::Success,
                assurance: Assurance::None,
            },
            execution,
        )
    }
    #[derive(Debug)]
    struct Capped {
        cap: u64,
        admitted: std::sync::atomic::AtomicU64,
        observed: std::sync::atomic::AtomicU64,
    }
    impl crate::solve::WorkAdmission for Capped {
        fn admit(&self, work: crate::solve::WorkEvidence) -> Result<(), ProblemError> {
            let count = work.evaluations.unwrap();
            self.admitted
                .try_update(
                    std::sync::atomic::Ordering::SeqCst,
                    std::sync::atomic::Ordering::SeqCst,
                    |n| n.checked_add(count).filter(|n| *n <= self.cap),
                )
                .map(|_| ())
                .map_err(|_| ProblemError::Limit {
                    kind: crate::LimitKind::Work,
                    detail: "evaluation cap".into(),
                })
        }
        fn observe(&self, work: crate::solve::WorkEvidence) -> Result<(), ProblemError> {
            self.observed.fetch_add(
                work.evaluations.unwrap(),
                std::sync::atomic::Ordering::SeqCst,
            );
            Ok(())
        }
        fn admit_storage(
            &self,
            _: crate::solve::NativeStorageScope,
            _: &str,
            _: usize,
            _: bool,
        ) -> Result<(), ProblemError> {
            Ok(())
        }
    }
    #[test]
    fn transported_callback_has_one_charging_owner_and_failed_operations_reconcile() {
        use std::sync::{Arc, atomic::Ordering};
        for transported in [false, true] {
            let admission = Arc::new(Capped {
                cap: 3,
                admitted: 0.into(),
                observed: 0.into(),
            });
            let mut execution = Execution::new(Arc::default(), &crate::solve::Controls::default());
            execution.work_admission = Some(admission.clone());
            let mut original = CallbackState::new(execution.clone());
            if transported {
                let mut forwarding = execution.clone();
                forwarding.callback_work_owner = false;
                let mut outer = CallbackState::new(forwarding);
                assert_eq!(
                    outer.evaluate("transport.objective", || original
                        .evaluate("original.objective", || Ok(7))
                        .ok_or_else(|| ProblemError::internal("inner refused"))),
                    Some(7)
                );
            } else {
                assert_eq!(original.evaluate("objective", || Ok(7)), Some(7));
            }
            assert_eq!(admission.observed.load(Ordering::SeqCst), 1);
            assert!(
                execution
                    .counted(evaluation_unit(), || Err::<(), _>(ProblemError::numerical(
                        "original evaluation failed"
                    )))
                    .is_err()
            );
            assert!(
                execution
                    .counted::<()>(evaluation_unit(), || panic!("failed indivisible operation"))
                    .is_err()
            );
            assert_eq!(admission.admitted.load(Ordering::SeqCst), 3);
            assert_eq!(admission.observed.load(Ordering::SeqCst), 3);
            let mut invoked = false;
            assert!(
                execution
                    .counted(evaluation_unit(), || {
                        invoked = true;
                        Ok(())
                    })
                    .is_err()
            );
            assert!(!invoked);
            assert_eq!(admission.observed.load(Ordering::SeqCst), 3);
        }
    }
    #[test]
    fn original_callback_refusal_survives_transported_native_unwind() {
        let execution = Execution::new(
            std::sync::Arc::default(),
            &crate::solve::Controls::default(),
        );
        let mut original = CallbackState::new(execution.clone());
        assert!(
            original
                .evaluate::<()>("gradient", || Err(pse_math::MathError::Domain {
                    source_id: pse_ids::SemanticId::from_bytes([1; 16]),
                    requirement: "finite difference probe envelope",
                }
                .into()))
                .is_none()
        );
        assert_eq!(original.evaluate("constraints", || Ok(7)), Some(7));
        let mut report = report(&execution);
        report.termination.category = Termination::Panic;
        original.finish(&mut report);
        assert_eq!(report.termination.category, Termination::Panic);
        assert!(matches!(
            report.callback_failure(),
            Some(ProblemError::Math(pse_math::MathError::Domain {
                requirement: "finite difference probe envelope",
                ..
            }))
        ));
        assert_eq!(report.evidence.callback.trial_rejections, 1);
    }
    #[test]
    fn scoped_deadline_classification_preserves_time_stop_and_other_limits() {
        let deadline = ProblemError::Math(pse_math::MathError::Scope(
            pse_kernels::ProviderError::Deadline,
        ));
        assert_eq!(
            classify(&deadline),
            Failure::Stopped(Termination::TimeLimit)
        );
        let limit = ProblemError::Math(pse_math::MathError::Scope(
            pse_kernels::ProviderError::Limit("allocation"),
        ));
        assert_eq!(classify(&limit), Failure::Fatal);
    }
    #[test]
    fn stopped_callback_keeps_its_original_typed_cause() {
        let execution = Execution::new(
            std::sync::Arc::default(),
            &crate::solve::Controls::default(),
        );
        let mut state = CallbackState::new(execution.clone());
        assert!(
            state
                .evaluate::<()>("residual", || Err(ProblemError::Limit {
                    kind: crate::LimitKind::Memory,
                    detail: "nested retained allocation".into()
                }))
                .is_none()
        );
        let mut report = report(&execution);
        state.finish(&mut report);
        assert!(
            matches!(report.callback_failure.as_deref(), Some(ProblemError::Limit {kind:crate::LimitKind::Memory,detail}) if detail == "nested retained allocation")
        );
        assert!(report.evidence.callback.terminal_failure);
    }
    #[test]
    fn callback_abandonment_discards_output_and_counts_owned_work_once() {
        use pse_model::generated::enums::NumericalAttemptObservation as O;
        let execution = Execution::new(
            std::sync::Arc::default(),
            &crate::solve::Controls::default(),
        );
        let mut state = CallbackState::new(execution.clone());
        assert_eq!(state.evaluate("residual", || Ok(1)), Some(1));
        assert_eq!(
            state.evaluate::<()>("jacobian", || Err(pse_math::MathError::Domain {
                source_id: pse_ids::SemanticId::NIL,
                requirement: "positive",
            }
            .into())),
            None
        );
        assert_eq!(
            state.evaluate("residual", || {
                execution.abandon(O::Limited)?;
                Ok(2)
            }),
            None
        );
        let mut invoked = false;
        assert_eq!(
            state.evaluate("residual", || {
                invoked = true;
                Ok(3)
            }),
            None
        );
        assert!(!invoked);
        assert!(!execution.cancel.load(std::sync::atomic::Ordering::Acquire));
        assert!(matches!(
            state.terminal_error(),
            Some(ProblemError::Limit {
                kind: crate::LimitKind::Work,
                ..
            })
        ));
        let mut report = report(&execution);
        assert_eq!(report.evidence.abandoned, Some(O::Limited));
        report.evidence.work.iterations = Some(7);
        state.finish(&mut report);
        assert_eq!(report.termination.category, Termination::Limit);
        assert_eq!(report.termination.code, 0);
        assert_eq!(report.termination.name, "success");
        assert_eq!(report.evidence.abandoned, Some(O::Limited));
        assert_eq!(report.evidence.work.evaluations, Some(3));
        assert_eq!(report.evidence.work.iterations, Some(7));
        assert_eq!(report.evidence.work.factorizations, None);
        assert_eq!(report.evidence.work.proof_steps, None);
        assert!(report.evidence.callback.terminal_failure);
        assert!(!retryable_evaluation(&report));
    }
    #[test]
    fn finish_rejects_late_native_success_without_another_callback() {
        let mut execution = Execution::new(
            std::sync::Arc::default(),
            &crate::solve::Controls::default(),
        );
        let mut report = report(&execution);
        execution.enclosing_scope = Some(pse_kernels::ExecutionScope::new(
            execution.cancel.clone(),
            Some(std::time::Instant::now()),
        ));
        let mut state = CallbackState::new(execution);
        state.finish(&mut report);
        assert_eq!(report.termination.category, Termination::TimeLimit);
        assert_eq!(report.evidence.work.evaluations, Some(0));
        assert_eq!(report.evidence.abandoned, None);
        assert_eq!(report.evidence.work.iterations, None);
        assert!(report.evidence.callback.terminal_failure);
        assert!(matches!(
            report.callback_failure.as_deref(),
            Some(ProblemError::Limit {
                kind: crate::LimitKind::Time,
                ..
            })
        ));
    }
    #[test]
    fn callback_recovery_evidence_survives_empty_history_and_refuses_fatal_failures() {
        use crate::solve::{Assurance, Backend, Controls, NativeTermination, SolveReport};
        let execution = Execution::new(
            std::sync::Arc::default(),
            &Controls {
                history: 0,
                ..Controls::default()
            },
        );
        let mut state = CallbackState::new(execution.clone());
        let contract = crate::OracleContract {
            identity: pse_ids::ContentHash::from_bytes([0; 32]),
            variables: vec![],
            rows: vec![],
            derivatives: pse_kernels::DerivativeOrder::First,
            smoothness: pse_kernels::DerivativeOrder::First,
        };
        let mut report = SolveReport::new(
            Backend::Kinsol,
            &contract,
            NativeTermination {
                code: -14,
                name: "evaluation".into(),
                message: None,
                category: Termination::Evaluation,
                assurance: Assurance::None,
            },
            &execution,
        );
        state.finish(&mut report);
        assert!(!retryable_evaluation(&report));
        state.evaluate::<()>("f", || {
            Err(pse_math::MathError::Domain {
                source_id: pse_ids::SemanticId::NIL,
                requirement: "positive",
            }
            .into())
        });
        state.finish(&mut report);
        assert!(report.events.is_empty());
        assert!(retryable_evaluation(&report));
        assert!(matches!(
            report.callback_failure(),
            Some(ProblemError::Math(pse_math::MathError::Domain { .. }))
        ));
        state.evaluate::<()>("f", || Err(ProblemError::Contract("terminal".into())));
        state.finish(&mut report);
        assert!(!retryable_evaluation(&report));
        assert_eq!(state.rejected_evaluations, 2);
        assert_eq!(state.trial_rejections, 1);
        assert!(
            matches!(report.callback_failure(),Some(ProblemError::Contract(s)) if s=="terminal")
        );
        let owner = std::sync::Arc::new(());
        let weak = std::sync::Arc::downgrade(&owner);
        let report = report.with_failure_owner(owner);
        let retained = report.clone();
        drop(report);
        assert!(weak.upgrade().is_some());
        assert!(retained.failure_bytes() >= size_of::<ProblemError>() + "terminal".len());
        drop(retained);
        assert!(weak.upgrade().is_none());
    }
    #[test]
    fn stopped_failures_keep_time_limit_and_cancellation() {
        for (error, stop) in [
            (
                ProblemError::Limit {
                    kind: crate::LimitKind::Time,
                    detail: "inner deadline".into(),
                },
                Termination::TimeLimit,
            ),
            (ProblemError::Cancelled, Termination::Cancelled),
            (
                pse_math::MathError::Cancelled.into(),
                Termination::Cancelled,
            ),
        ] {
            assert_eq!(classify(&error), Failure::Stopped(stop));
            let original_kind = std::mem::discriminant(&error);
            let original_message = error.to_string();
            let mut state = CallbackState::new(Execution::new(
                std::sync::Arc::default(),
                &crate::solve::Controls::default(),
            ));
            assert!(state.evaluate::<()>("f", || Err(error)).is_none());
            assert_eq!(state.terminal.as_ref().map(|t| t.0), Some(stop));
            let retained = state.terminal_error().unwrap();
            assert_eq!(classify(&retained), Failure::Stopped(stop));
            assert_eq!(std::mem::discriminant(&retained), original_kind);
            assert_eq!(retained.to_string(), original_message);
            assert!(matches!(
                retained,
                ProblemError::Cancelled
                    | ProblemError::Limit {
                        kind: crate::LimitKind::Time,
                        ..
                    }
                    | ProblemError::Math(pse_math::MathError::Cancelled)
            ));
        }
        for fatal in [
            ProblemError::numerical("native failure"),
            ProblemError::Limit {
                kind: crate::LimitKind::Memory,
                detail: "allocation".into(),
            },
            ProblemError::internal("postcondition"),
        ] {
            assert_eq!(classify(&fatal), Failure::Fatal);
        }
    }
    #[test]
    fn recovered_trials_do_not_poison_success_but_panics_do() {
        let execution = Execution::new(
            std::sync::Arc::default(),
            &crate::solve::Controls::default(),
        );
        let mut state = CallbackState::new(execution.clone());
        assert!(
            state
                .evaluate::<()>("f", || Err(pse_math::MathError::Domain {
                    source_id: pse_ids::SemanticId::from_bytes([1; 16]),
                    requirement: "positive"
                }
                .into()))
                .is_none()
        );
        assert!(state.terminal.is_none());
        assert!(matches!(
            state.last_failure,
            Some(ProblemError::Math(pse_math::MathError::Domain { .. }))
        ));
        assert_eq!(state.evaluate("f", || Ok(42)), Some(42));
        assert!(state.terminal.is_none());
        let mut successful = report(&execution);
        state.finish(&mut successful);
        assert!(successful.callback_failure().is_none());
        assert!(state.last_failure.is_none());
        assert!(state.evaluate::<()>("f", || panic!("contained")).is_none());
        assert_eq!(
            state.terminal.as_ref().map(|t| t.0),
            Some(Termination::Panic)
        );
        assert_eq!(state.evaluate("f", || Ok(43)), None);
    }
}
