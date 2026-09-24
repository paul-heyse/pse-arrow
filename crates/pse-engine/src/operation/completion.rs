// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Invocation-local completion. A dropped last waiter poisons unfinished pure
//! work, rather than allowing a later consumer to restart partial validation.
use datafusion::common::{DataFusionError, Result};
use futures_util::{
    FutureExt,
    future::{BoxFuture, Shared},
};
use std::sync::{Arc, Mutex};

/// Why an attempt cannot supply a completed valid value. Causes remain typed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureKind {
    /// A required semantic or output contract failed.
    Invalid,
    /// Cooperative cancellation was observed.
    Cancelled,
    /// The last interested reader left unfinished pure work.
    Abandoned,
    /// Fallible resource admission refused the attempt.
    ResourceRefused,
    /// A producer failed after emitting only part of its stream.
    PartialStream,
    /// External visibility has not settled, or delivery failed after an effect.
    UncertainEffects,
    /// Another typed native execution failure.
    Execution,
}

/// State of one producer attempt, independently of buffer ownership/publication.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompletionStatus {
    /// No reader has polled the producer.
    NotStarted,
    /// Interested readers share the admitted attempt.
    Running,
    /// The complete value and its required obligations have settled.
    Valid,
    /// A terminal unsuccessful outcome, never a cacheable successful value.
    Failed(FailureKind),
}

/// A typed completion boundary preserving its original native error tree.
#[derive(Debug, thiserror::Error)]
#[error("producer completion {kind:?}: {source}")]
pub struct CompletionFailure {
    /// Terminal outcome class.
    pub kind: FailureKind,
    /// Exact underlying failure, including settlement evidence.
    #[source]
    pub source: DataFusionError,
}
pse_diagnostics::impl_diagnostic! {
    CompletionFailure,
    code(this) { Some(match this.kind {
        FailureKind::Invalid => pse_diagnostics::DiagnosticCode::SchemaAdmission,
        FailureKind::Cancelled | FailureKind::Abandoned => pse_diagnostics::DiagnosticCode::RuntimeCancelled,
        FailureKind::ResourceRefused => pse_diagnostics::DiagnosticCode::RuntimeResourceLimit,
        _ => pse_diagnostics::DiagnosticCode::RuntimeInfrastructure,
    }) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}
impl FailureKind {
    /// Preserve a domain boundary's outcome through native error wrappers.
    pub fn error(self, source: DataFusionError) -> DataFusionError {
        pse_columnar::external(CompletionFailure { kind: self, source })
    }
}

fn failure_kind(error: &DataFusionError) -> FailureKind {
    let mut current: Option<&(dyn std::error::Error + 'static)> = Some(error);
    while let Some(cause) = current {
        if let Some(failure) = cause.downcast_ref::<CompletionFailure>() {
            return failure.kind;
        }
        current = cause.source();
    }
    for leaf in pse_columnar::observe(error, pse_columnar::PlanOrigin::Analytics) {
        match leaf.code.class() {
            pse_diagnostics::FailureClass::RuntimeCancelled => return FailureKind::Cancelled,
            pse_diagnostics::FailureClass::RuntimeResourceLimit => {
                return FailureKind::ResourceRefused;
            }
            pse_diagnostics::FailureClass::UserModel
            | pse_diagnostics::FailureClass::ValidationInvariant => return FailureKind::Invalid,
            _ => {}
        }
    }
    FailureKind::Execution
}

/// A transport failure after a value escaped is partial. Preserve more specific
/// cancellation, admission, resource and effect-settlement classifications.
pub(crate) fn stream_error(error: DataFusionError, yielded: bool) -> DataFusionError {
    if yielded && failure_kind(&error) == FailureKind::Execution {
        FailureKind::PartialStream.error(error)
    } else {
        error
    }
}

type Outcome<T> = std::result::Result<Arc<T>, Arc<DataFusionError>>;
type Work<T> = Shared<BoxFuture<'static, Outcome<T>>>;
struct State<T> {
    work: Option<Work<T>>,
    result: Option<Outcome<T>>,
    readers: usize,
    status: CompletionStatus,
}
/// A shared producer's result or terminal failure within one invocation/epoch.
pub struct Completion<T>(Mutex<State<T>>);
impl<T> Default for Completion<T> {
    fn default() -> Self {
        Self(Mutex::new(State {
            work: None,
            result: None,
            readers: 0,
            status: CompletionStatus::NotStarted,
        }))
    }
}
impl<T> std::fmt::Debug for Completion<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SharedCompletion")
    }
}
impl<T: Send + Sync + 'static> Completion<T> {
    #[cfg(test)]
    pub(crate) fn status(&self) -> Result<CompletionStatus> {
        Ok(self.0.lock().map_err(|_| poisoned())?.status)
    }
    /// Inspect the terminal outcome without starting work.
    /// # Errors
    /// The completion lock was poisoned.
    pub fn completed(&self) -> Result<Option<Outcome<T>>> {
        Ok(self.0.lock().map_err(|_| poisoned())?.result.clone())
    }
    /// Join this attempt's producer; the last departing reader terminates unfinished work.
    /// # Errors
    /// Producer refusal/failure, cancellation, abandonment or a poisoned owner lock.
    pub async fn get(
        self: &Arc<Self>,
        create: impl FnOnce() -> BoxFuture<'static, Result<T>>,
    ) -> Result<Arc<T>> {
        let work = {
            let mut state = self.0.lock().map_err(|_| poisoned())?;
            if let Some(result) = &state.result {
                return result.clone().map_err(DataFusionError::Shared);
            }
            let work = state
                .work
                .get_or_insert_with(|| {
                    let work = create();
                    async move { work.await.map(Arc::new).map_err(Arc::new) }
                        .boxed()
                        .shared()
                })
                .clone();
            state.readers += 1;
            state.status = CompletionStatus::Running;
            work
        };
        let _reader = Reader(self.clone());
        let result = work.await;
        let mut state = self.0.lock().map_err(|_| poisoned())?;
        state.result = Some(result.clone());
        state.status = match &result {
            Ok(_) => CompletionStatus::Valid,
            Err(error) => CompletionStatus::Failed(failure_kind(error)),
        };
        state.work = None;
        result.map_err(DataFusionError::Shared)
    }
}
struct Reader<T>(Arc<Completion<T>>);
impl<T> Drop for Reader<T> {
    fn drop(&mut self) {
        if let Ok(mut state) = self.0.0.lock() {
            state.readers -= 1;
            if state.readers == 0 && state.result.is_none() {
                state.status = CompletionStatus::Failed(FailureKind::Abandoned);
                state.result = Some(Err(Arc::new(FailureKind::Abandoned.error(
                    DataFusionError::Execution(
                        "shared producer abandoned before completion".into(),
                    ),
                ))));
                state.work = None;
            }
        }
    }
}
fn poisoned() -> DataFusionError {
    DataFusionError::Internal("shared completion lock poisoned".into())
}

#[cfg(test)]
mod integrated_performance_unit {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn concurrent_waiters_share_one_body_and_final_abandonment_is_attempt_local() {
        let completion = Arc::new(Completion::<usize>::default());
        let starts = Arc::new(AtomicUsize::new(0));
        let create = || {
            starts.fetch_add(1, Ordering::SeqCst);
            std::future::pending::<Result<usize>>().boxed()
        };
        let mut first = Box::pin(completion.get(create));
        let mut second = Box::pin(completion.get(create));
        assert!(futures_util::poll!(first.as_mut()).is_pending());
        assert!(futures_util::poll!(second.as_mut()).is_pending());
        assert_eq!(starts.load(Ordering::SeqCst), 1);
        drop(first);
        assert_eq!(completion.status().unwrap(), CompletionStatus::Running);
        drop(second);
        assert_eq!(
            completion.status().unwrap(),
            CompletionStatus::Failed(FailureKind::Abandoned)
        );
        assert!(completion.get(|| async { Ok(7) }.boxed()).await.is_err());
        let next = Arc::new(Completion::default());
        assert_eq!(*next.get(|| async { Ok(7) }.boxed()).await.unwrap(), 7);
        assert_eq!(next.status().unwrap(), CompletionStatus::Valid);
    }

    #[tokio::test]
    async fn unsuccessful_outcomes_are_distinct_and_never_publish_values() {
        for kind in [
            FailureKind::Invalid,
            FailureKind::Cancelled,
            FailureKind::ResourceRefused,
            FailureKind::PartialStream,
            FailureKind::UncertainEffects,
            FailureKind::Execution,
        ] {
            let completion = Arc::new(Completion::<usize>::default());
            let error = completion
                .get(|| {
                    async move { Err(kind.error(DataFusionError::Execution("typed cause".into()))) }
                        .boxed()
                })
                .await
                .unwrap_err();
            let expected = match kind {
                FailureKind::Invalid => pse_diagnostics::DiagnosticCode::SchemaAdmission,
                FailureKind::Cancelled => pse_diagnostics::DiagnosticCode::RuntimeCancelled,
                FailureKind::ResourceRefused => {
                    pse_diagnostics::DiagnosticCode::RuntimeResourceLimit
                }
                _ => pse_diagnostics::DiagnosticCode::RuntimeInfrastructure,
            };
            let leaves = pse_columnar::observe(&error, pse_columnar::PlanOrigin::Analytics);
            assert_eq!(leaves.len(), 1);
            assert_eq!(leaves[0].code, expected);
            assert_eq!(completion.status().unwrap(), CompletionStatus::Failed(kind));
            assert!(completion.completed().unwrap().unwrap().is_err());
            assert!(completion.get(|| async { Ok(9) }.boxed()).await.is_err());
        }
    }

    #[test]
    fn completion_uses_typed_native_leaves_for_collections_and_domain_wrappers() {
        let resource = DataFusionError::Context(
            "outer".into(),
            Box::new(DataFusionError::Collection(vec![
                DataFusionError::ResourcesExhausted("bound".into()),
            ])),
        );
        assert_eq!(
            failure_kind(&stream_error(resource, true)),
            FailureKind::ResourceRefused
        );
        let cancel: DataFusionError = crate::EngineError::Cancelled.into();
        assert_eq!(
            failure_kind(&stream_error(cancel, true)),
            FailureKind::Cancelled
        );
    }

    #[test]
    fn partial_stream_keeps_specific_cancellation_and_settlement_causes() {
        assert_eq!(
            failure_kind(&stream_error(
                DataFusionError::Execution("read failed".into()),
                true
            )),
            FailureKind::PartialStream
        );
        for kind in [
            FailureKind::Cancelled,
            FailureKind::UncertainEffects,
            FailureKind::ResourceRefused,
        ] {
            assert_eq!(
                failure_kind(&stream_error(
                    kind.error(DataFusionError::Execution("cause".into())),
                    true
                )),
                kind
            );
        }
    }
}

pse_columnar::impl_native_error!(CompletionFailure);
