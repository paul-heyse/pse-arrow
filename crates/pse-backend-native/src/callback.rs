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
            _ if error.recoverable() => Failure::Trial,
            _ => Failure::Fatal,
        }
    }
    fn math(error: &pse_math::MathError) -> Failure {
        use pse_math::MathError as E;
        match error {
            E::Instance { cause, .. } => math(cause),
            E::Validity(_) | E::Domain { .. } | E::OutsideRange { .. } => Failure::Trial,
            E::Cancelled => Failure::Stopped(Termination::Cancelled),
            E::Provider { cause, .. } => provider(cause),
            E::Native { cause, .. } => cause
                .downcast_ref::<ProblemError>()
                .map_or(Failure::Fatal, classify),
            E::Contract(_)
            | E::CoefficientRange
            | E::Library(_)
            | E::Evaluation { .. }
            | E::Limit(_)
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
        ProblemError::Unavailable { .. }
        | ProblemError::Contract(_)
        | ProblemError::Structural { .. }
        | ProblemError::Unsupported(_)
        | ProblemError::Reuse { .. }
        | ProblemError::Numerical { .. }
        | ProblemError::Limit { .. }
        | ProblemError::Internal(_) => Failure::Fatal,
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
    /// Cause of the latest failed callback, until a later callback succeeds.
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
            self.terminal = Some((stop, "execution checkpoint".into()));
            return None;
        }
        let start = std::time::Instant::now();
        let result = catch_unwind(AssertUnwindSafe(work));
        *self.counts.entry(demand.into()).or_default() += 1;
        *self.seconds.entry(demand.into()).or_default() += start.elapsed().as_secs_f64();
        let (failure, message) = match result {
            Ok(Ok(value)) => {
                self.last_failure = None;
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
        Some(match kind {
            Termination::Evaluation => self
                .last_failure
                .take()
                .unwrap_or_else(|| ProblemError::internal(message.clone())),
            Termination::Cancelled | Termination::TimeLimit => {
                ProblemError::stopped(*kind, message.clone())
            }
            _ => ProblemError::internal(message.clone()),
        })
    }
    /// Append callback measurements and preserve native status alongside terminal cause.
    pub fn finish(&mut self, report: &mut crate::solve::SolveReport) {
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
        if report.termination.category == Termination::Evaluation {
            if let Some(cause) = self.last_failure.take() {
                report.callback_failure = Some(std::sync::Arc::new(cause));
            }
        } else {
            report.callback_failure = None;
        }
        (report.events, report.dropped_events) = self.execution.progress.snapshot();
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
            E::Native { cause, .. } => cause
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
            let mut state = CallbackState::new(Execution::new(
                std::sync::Arc::default(),
                &crate::solve::Controls::default(),
            ));
            assert!(state.evaluate::<()>("f", || Err(error)).is_none());
            assert_eq!(state.terminal.as_ref().map(|t| t.0), Some(stop));
            assert!(matches!(
                state.terminal_error(),
                Some(ProblemError::Cancelled | ProblemError::Limit { .. })
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
        let mut state = CallbackState::new(execution);
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
        assert!(state.last_failure.is_none());
        assert!(state.evaluate::<()>("f", || panic!("contained")).is_none());
        assert_eq!(
            state.terminal.as_ref().map(|t| t.0),
            Some(Termination::Panic)
        );
        assert_eq!(state.evaluate("f", || Ok(43)), None);
    }
}
