// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Opt-in native execution evidence. Observation never establishes value correctness.
use datafusion::execution::session_state::{SessionState, SessionStateBuilder};
use futures_util::FutureExt;
use std::{
    any::Any,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};
use tracing::{Instrument, instrument::WithSubscriber};

/// Operational inspection policy, excluded from semantic identities.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ObservationPolicy {
    /// No diagnostic capture or plan rendering.
    #[default]
    Off,
    /// Native phase/operator spans and explicit operation completion.
    Contract,
    /// Contract evidence plus bounded plan text and rule names; no value previews.
    Diagnostic,
}
impl ObservationPolicy {
    /// Actual policy retained by the native assembly.
    pub fn from_state(state: &SessionState) -> Self {
        state
            .config()
            .get_extension::<Self>()
            .as_deref()
            .copied()
            .unwrap_or_default()
    }
}

/// Whether this process owns the hook needed to capture both task kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskPropagation {
    /// Both asynchronous and blocking tasks propagate captured span and dispatch.
    Complete,
    /// Another process-wide hook was installed; complete propagation is unproved.
    ForeignHook,
}
struct TaskTracer;
impl datafusion::common::runtime::JoinSetTracer for TaskTracer {
    fn trace_future(
        &self,
        future: futures_util::future::BoxFuture<'static, Box<dyn Any + Send>>,
    ) -> futures_util::future::BoxFuture<'static, Box<dyn Any + Send>> {
        let span = tracing::Span::current();
        let dispatch = tracing::dispatcher::get_default(Clone::clone);
        future.instrument(span).with_subscriber(dispatch).boxed()
    }
    fn trace_block(
        &self,
        work: Box<dyn FnOnce() -> Box<dyn Any + Send> + Send>,
    ) -> Box<dyn FnOnce() -> Box<dyn Any + Send> + Send> {
        let span = tracing::Span::current();
        let dispatch = tracing::dispatcher::get_default(Clone::clone);
        Box::new(move || tracing::dispatcher::with_default(&dispatch, || span.in_scope(work)))
    }
}
/// Install once, before the library instrumentation macro tries its own hook.
/// A foreign hook leaves completeness unproved; repeated calls retain that fact.
pub fn task_propagation() -> TaskPropagation {
    static STATUS: OnceLock<TaskPropagation> = OnceLock::new();
    *STATUS.get_or_init(
        || match datafusion::common::runtime::set_join_set_tracer(&TaskTracer) {
            Ok(()) => TaskPropagation::Complete,
            Err(_) => TaskPropagation::ForeignHook,
        },
    )
}

#[derive(Debug)]
struct InstrumentedState;
/// Instrument the cold model assembly once, retaining caller planner and rule order.
pub(super) fn instrument(state: SessionState) -> SessionState {
    let policy = ObservationPolicy::from_state(&state);
    if policy == ObservationPolicy::Off
        || state
            .config()
            .get_extension::<InstrumentedState>()
            .is_some()
    {
        return state;
    }
    let _ = task_propagation();
    let options = datafusion_tracing::InstrumentationOptions::builder()
        .record_metrics(true)
        .preview_limit(0)
        .build();
    let rule = datafusion_tracing::instrument_with_info_spans!(target: "pse_engine::execution", options: options);
    let mut rules = state.physical_optimizers().to_vec();
    rules.push(rule);
    let config = state
        .config()
        .clone()
        .with_extension(Arc::new(InstrumentedState));
    let state = SessionStateBuilder::new_from_existing(state)
        .with_config(config)
        .with_physical_optimizer_rules(rules)
        .build();
    datafusion_tracing::instrument_rules_with_info_spans!(target: "pse_engine::execution", options: datafusion_tracing::RuleInstrumentationOptions::phase_only(), state: state)
}

/// Successful exhaustion is different from dropping a partially consumed stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalStatus {
    /// The owned stream reached successful exhaustion.
    Completed,
    /// Planning, allocation or native execution failed.
    Failed,
    /// The invocation observed cooperative cancellation.
    Cancelled,
    /// The owner was dropped before another terminal outcome was established.
    Abandoned,
}
/// One invocation's explicit lifecycle, including errors before a stream is returned.
#[derive(Debug)]
pub struct OperationObservation {
    span: tracing::Span,
    terminal: Option<TerminalStatus>,
}
impl OperationObservation {
    /// Start a correlated operation. Span fields are declared, not inferred from logs.
    pub fn start(policy: ObservationPolicy, kind: &'static str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let span = if policy == ObservationPolicy::Off {
            tracing::Span::none()
        } else {
            tracing::info_span!(target: "pse_engine::execution", "pse.operation", operation_id = NEXT.fetch_add(1, Ordering::Relaxed), operation_kind = kind, terminal = tracing::field::Empty, task_propagation = ?task_propagation())
        };
        Self {
            span,
            terminal: None,
        }
    }
    /// Span propagated while planning, starting and polling this invocation.
    pub fn span(&self) -> tracing::Span {
        self.span.clone()
    }
    /// Record the first terminal outcome only. Native metrics remain on the plan.
    pub fn finish(&mut self, status: TerminalStatus) {
        if self.terminal.is_none() {
            self.span.record("terminal", tracing::field::debug(status));
            self.terminal = Some(status);
        }
    }
}
impl Drop for OperationObservation {
    fn drop(&mut self) {
        self.finish(TerminalStatus::Abandoned);
    }
}
