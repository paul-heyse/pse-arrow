// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Copied attribution only; execution authority always remains explicit at its owner.
use pse_diagnostics::observation::{ObservationContext, OperationGuard, Outcome, Relationship};
use std::{
    future::Future,
    sync::{Arc, Mutex},
};
use tracing::{Instrument, instrument::WithSubscriber};

/// Attribution captured before crossing a runtime task or native thread boundary.
#[derive(Clone)]
pub struct ExecutionContext {
    runtime: Option<tokio::runtime::Handle>,
    observation: Option<ObservationContext>,
    invocation: crate::invocation::TransferredContext,
    dispatch: tracing::Dispatch,
    span: tracing::Span,
    carrier: crate::tooling::telemetry::Carrier,
    operation: Option<Arc<Mutex<Option<OperationGuard>>>>,
}
/// Lexically entered attribution; no lifetime/capacity permission is acquired.
pub struct EnteredExecution<'a> {
    _span: tracing::span::Entered<'a>,
    _dispatch: tracing::dispatcher::DefaultGuard,
    _parent: opentelemetry::ContextGuard,
    _native: Option<crate::observation::ThreadScope>,
    _observation: pse_diagnostics::observation::EnterGuard,
    _invocation: crate::invocation::EnteredContext,
    _runtime: Option<tokio::runtime::EnterGuard<'a>>,
}
impl ExecutionContext {
    /// Capture current carriers without starting a recorder or acquiring authority.
    pub fn capture() -> Self {
        Self {
            runtime: tokio::runtime::Handle::try_current().ok(),
            observation: pse_diagnostics::observation::current(),
            invocation: crate::invocation::TransferredContext::capture(),
            dispatch: tracing::dispatcher::get_default(Clone::clone),
            span: tracing::Span::current(),
            carrier: crate::tooling::telemetry::Carrier::current(),
            operation: None,
        }
    }
    pub(crate) fn with_observation(mut self, observation: Option<ObservationContext>) -> Self {
        self.observation = observation;
        self
    }
    pub(crate) fn without_request(mut self) -> Self {
        self.observation = None;
        self.invocation = crate::invocation::TransferredContext::unavailable();
        self.span = tracing::Span::none();
        self.dispatch = tracing::Dispatch::none();
        self.carrier = crate::tooling::telemetry::Carrier::default();
        self.operation = None;
        self
    }
    /// Enter the same attribution for synchronous work on another thread.
    pub fn enter(&self) -> EnteredExecution<'_> {
        self.enter_mode(false)
    }
    /// Additionally observe an actual dedicated native execution thread.
    pub(crate) fn enter_native(&self) -> EnteredExecution<'_> {
        self.enter_mode(true)
    }
    fn enter_mode(&self, native: bool) -> EnteredExecution<'_> {
        let runtime = self.runtime.as_ref().map(tokio::runtime::Handle::enter);
        let invocation = self.invocation.enter();
        let observation = pse_diagnostics::observation::enter_optional(self.observation.clone());
        let native = native
            .then(crate::observation::native_execution_scope)
            .flatten();
        let parent = self.carrier.context().attach();
        let dispatch = tracing::dispatcher::set_default(&self.dispatch);
        let span = self.span.enter();
        EnteredExecution {
            _span: span,
            _dispatch: dispatch,
            _parent: parent,
            _native: native,
            _observation: observation,
            _invocation: invocation,
            _runtime: runtime,
        }
    }
    /// Poll with copied attribution. No entered thread-local guard crosses an await.
    pub fn instrument<F: Future>(self, future: F) -> impl Future<Output = F::Output> {
        self.instrument_with(future, |_| None)
    }
    /// Derived operations require their performing owner to classify terminal outcomes.
    pub fn instrument_with<F: Future>(
        self,
        future: F,
        outcome: impl FnOnce(&F::Output) -> Option<Outcome>,
    ) -> impl Future<Output = F::Output> {
        let invocation = self.invocation.clone();
        let observation = self.observation.clone();
        let operation = self.operation.clone();
        let carrier = self.carrier.clone();
        let future = async move {
            let mut future = std::pin::pin!(future);
            let value = std::future::poll_fn(|cx| {
                let _parent = carrier.context().attach();
                future.as_mut().poll(cx)
            })
            .await;
            if let Some(terminal) = outcome(&value)
                && let Some(operation) = operation
                && let Ok(mut guard) = operation.lock()
                && let Some(guard) = guard.take()
            {
                guard.finish(terminal);
            }
            value
        };
        invocation
            .scope_inherited(pse_diagnostics::observation::scope(observation, future))
            .instrument(self.span)
            .with_subscriber(self.dispatch)
    }
    /// Adopt an observation already owned by a neutral shared producer.
    pub(crate) fn shared_producer_at(
        &self,
        kind: &'static str,
        observation: Option<ObservationContext>,
    ) -> Self {
        let mut context = self.clone();
        context.observation = observation;
        context.operation = None;
        context.span =
            tracing::dispatcher::with_default(&self.dispatch, || self.carrier.linked_span(kind));
        context.carrier = crate::tooling::telemetry::Carrier::default();
        context.invocation = crate::invocation::TransferredContext::unavailable();
        context
    }
    /// A nested observed operation retains this invocation's recorder.
    pub fn nested(&self, kind: &'static str) -> Self {
        self.derived(kind, Relationship::Nested, false)
    }
    /// Independent shared production links the initiator and never borrows its recorder.
    pub fn shared_producer(&self, kind: &'static str) -> Self {
        self.derived(kind, Relationship::SharedProduction, true)
    }
    fn derived(&self, kind: &'static str, relationship: Relationship, shared: bool) -> Self {
        let mut context = self.clone();
        let operation = self
            .observation
            .as_ref()
            .map(|parent| parent.child(kind, relationship));
        context.observation = operation.as_ref().map(OperationGuard::context);
        context.operation = operation.map(|owner| Arc::new(Mutex::new(Some(owner))));
        context.span = tracing::dispatcher::with_default(&self.dispatch, || {
            if shared {
                self.carrier.linked_span(kind)
            } else {
                tracing::info_span!(parent: &self.span, "pse.native.producer", pse.owner.stage = true, owner = kind)
            }
        });
        if shared {
            context.invocation = crate::invocation::TransferredContext::unavailable();
            context.carrier = crate::tooling::telemetry::Carrier::default();
        }
        context
    }
}

impl std::fmt::Debug for EnteredExecution<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnteredExecution").finish_non_exhaustive()
    }
}

impl std::fmt::Debug for ExecutionContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecutionContext")
            .field("runtime", &self.runtime.is_some())
            .field(
                "operation",
                &self.observation.as_ref().map(ObservationContext::id),
            )
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_diagnostics::observation::{ObservationHub, Policy};
    use std::{
        sync::atomic::{AtomicUsize, Ordering},
        time::Duration,
    };
    use tracing_subscriber::prelude::*;
    #[test]
    fn absent_context_clears_and_restores_enclosing_observation() {
        let absent = ExecutionContext::capture().without_request();
        let hub = ObservationHub::new(Policy::default());
        let parent = hub.start("parent");
        let _parent = parent.context().enter();
        {
            let _entered = absent.enter();
            assert!(pse_diagnostics::observation::current().is_none());
        }
        assert_eq!(
            pse_diagnostics::observation::current().unwrap().id(),
            parent.context().id()
        );
    }
    #[test]
    fn thread_transfer_preserves_exact_recorder_and_restores_absence() {
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("original invocation");
        let _entered = operation.context().enter();
        crate::invocation::start_scope(|| {
            let recorder = crate::invocation::TransferredContext::capture().recorder_ptr_for_test();
            assert!(recorder.is_some());
            let context = ExecutionContext::capture();
            std::thread::spawn(move || {
                {
                    let _entered = context.enter();
                    assert_eq!(
                        crate::invocation::TransferredContext::capture().recorder_ptr_for_test(),
                        recorder
                    );
                }
                assert!(!crate::invocation::TransferredContext::capture().has_recorder_for_test());
            })
            .join()
            .unwrap();
        })();
    }
    #[tokio::test]
    async fn neutral_flight_producer_observation_is_adopted_without_an_extra_child() {
        let hub = ObservationHub::new(Policy::default());
        let parent = hub.start("first waiter");
        let flights = pse_columnar::flight::Flights::<u8, usize, &'static str>::new(2);
        let result = pse_diagnostics::observation::scope(Some(parent.context()), async {
            let context = ExecutionContext::capture();
            flights
                .load_owned(1, move |_| {
                    let actual = pse_diagnostics::observation::current().unwrap();
                    let expected = actual.id();
                    context
                        .shared_producer_at("actual producer", Some(actual))
                        .instrument(async move {
                            assert_eq!(
                                pse_diagnostics::observation::current().unwrap().id(),
                                expected
                            );
                            assert!(
                                !crate::invocation::TransferredContext::capture()
                                    .has_recorder_for_test()
                            );
                            Ok(Arc::new(9))
                        })
                })
                .await
                .unwrap()
        })
        .await;
        assert_eq!(*result, 9);
        assert_eq!(
            parent
                .reader()
                .snapshot(Duration::from_secs(60))
                .children
                .len(),
            1
        );
    }
    #[test]
    fn captured_dispatch_reaches_thread_and_does_not_leak_after_guard_drop() {
        struct Events(Arc<AtomicUsize>);
        impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for Events {
            fn on_event(
                &self,
                _: &tracing::Event<'_>,
                _: tracing_subscriber::layer::Context<'_, S>,
            ) {
                self.0.fetch_add(1, Ordering::Relaxed);
            }
        }
        let events = Arc::new(AtomicUsize::new(0));
        let dispatch =
            tracing::Dispatch::new(tracing_subscriber::registry().with(Events(events.clone())));
        let context = tracing::dispatcher::with_default(&dispatch, ExecutionContext::capture);
        std::thread::spawn(move || {
            {
                let _context = context.enter();
                tracing::info!("captured");
            }
            tracing::info!("outside captured scope");
        })
        .join()
        .unwrap();
        assert_eq!(events.load(Ordering::Relaxed), 1);
    }
    #[tokio::test]
    async fn shared_producer_has_explicit_failed_and_cancelled_outcomes_without_recorder() {
        let hub = ObservationHub::new(Policy::default());
        let parent = hub.start("parent");
        let context = pse_diagnostics::observation::scope(Some(parent.context()), async {
            ExecutionContext::capture()
        })
        .await;
        for outcome in [Outcome::Failed, Outcome::Cancelled] {
            let producer = context.shared_producer("shared");
            let reader = producer.observation.as_ref().unwrap().reader();
            assert!(!producer.invocation.has_recorder_for_test());
            let result: Result<(), &str> = producer
                .instrument_with(async { Err("actual failure") }, |_| Some(outcome))
                .await;
            assert!(result.is_err());
            assert_eq!(
                reader.snapshot(Duration::from_secs(60)).outcome,
                Some(outcome)
            );
        }
    }
}
