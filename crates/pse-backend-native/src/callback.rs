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
    /// Explicit cancellation.
    Cancelled,
}
/// Classify typed causes, never native diagnostic strings.
pub fn classify(error: &ProblemError) -> Failure {
    fn math(error: &pse_math::MathError) -> Failure {
        use pse_math::MathError as E;
        match error {
            E::Instance { cause, .. } => math(cause),
            E::Domain { .. } => Failure::Trial,
            E::Cancelled => Failure::Cancelled,
            E::Provider { cause, .. } => match cause {
                pse_kernels::ProviderError::Trial { .. }
                | pse_kernels::ProviderError::OutsideEnvelope { .. }
                | pse_kernels::ProviderError::Singular { .. } => Failure::Trial,
                pse_kernels::ProviderError::Cancelled => Failure::Cancelled,
                _ => Failure::Fatal,
            },
            _ => Failure::Fatal,
        }
    }
    match error {
        ProblemError::Math(e) => math(e),
        _ => Failure::Fatal,
    }
}
/// Worker-local failure state; recoverable history never becomes a terminal latch.
#[derive(Debug)]
pub struct CallbackState {
    /// Shared stop/progress controls.
    pub execution: Execution,
    /// Only terminal failures latch. Successful later trials preserve native success.
    pub terminal: Option<(Termination, String)>,
    /// Per-demand call count, including rejected trials.
    pub counts: BTreeMap<String, i64>,
    /// Per-demand wall seconds, including rejected trials.
    pub seconds: BTreeMap<String, f64>,
}
impl CallbackState {
    /// Start a worker-local callback boundary.
    pub fn new(execution: Execution) -> Self {
        Self {
            execution,
            terminal: None,
            counts: BTreeMap::new(),
            seconds: BTreeMap::new(),
        }
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
            Ok(Ok(value)) => return Some(value),
            Ok(Err(e)) => {
                let failure = classify(&e);
                (failure, e.to_string())
            }
            Err(_) => {
                self.terminal = Some((Termination::Panic, "panic in native callback".into()));
                return None;
            }
        };
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
        });
        if failure != Failure::Trial {
            self.terminal = Some((
                if failure == Failure::Cancelled {
                    self.execution.stopped().unwrap_or(Termination::Cancelled)
                } else {
                    Termination::Evaluation
                },
                message,
            ));
        }
        None
    }
    /// Append callback measurements and preserve native status alongside terminal cause.
    pub fn finish(&self, report: &mut crate::solve::SolveReport) {
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
        (report.events, report.dropped_events) = self.execution.progress.snapshot();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(state.evaluate("f", || Ok(42)), Some(42));
        assert!(state.evaluate::<()>("f", || panic!("contained")).is_none());
        assert_eq!(
            state.terminal.as_ref().map(|t| t.0),
            Some(Termination::Panic)
        );
        assert_eq!(state.evaluate("f", || Ok(43)), None);
    }
}
