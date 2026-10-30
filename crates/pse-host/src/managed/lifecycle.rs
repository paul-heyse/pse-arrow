// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Library-owned cooperative invocation subsystems. OS settlement is a separate owner result.
use crate::HostError;
use std::time::Duration;
use tokio_graceful_shutdown::{
    AsyncSubsysFn, SubsystemBuilder, SubsystemHandle, Toplevel,
    errors::{GracefulShutdownError, SubsystemError},
};
use tokio_util::sync::CancellationToken;

/// One original named task failure; the native concrete cause remains available.
#[derive(Debug, thiserror::Error)]
pub enum SubsystemFailure {
    /// A cooperating component returned its concrete host error.
    #[error("invocation subsystem {name} failed: {cause}")]
    Failed {
        /// Original native subsystem path.
        name: String,
        /// Original error, not parsed display text.
        #[source]
        cause: Box<HostError>,
    },
    /// The library reports a panic separately; it does not expose the panic payload.
    #[error("invocation subsystem {name} panicked")]
    Panicked {
        /// Original native subsystem path.
        name: String,
    },
}
pse_diagnostics::impl_diagnostic! {
    SubsystemFailure,
    code(this) { match this { Self::Failed {cause,..} => pse_diagnostics::TypedDiagnostic::diagnostic_code(cause.as_ref()), Self::Panicked {..} => Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure) } },
    forward(this) { match this {Self::Failed {cause,..} => Some(cause.as_ref()), _ => None} },
    help(_this) {None}, related(_this) {None}, source(_this) {None}
}
/// Completion of the native async hierarchy, independently of processes, files and fixtures.
#[derive(Debug)]
pub struct ShutdownReport {
    /// Native shutdown deadline expired. This never means cleanup completed.
    pub timed_out: bool,
    /// Every uncaught native failure reported before completion/deadline.
    pub failures: Vec<SubsystemFailure>,
}
impl ShutdownReport {
    /// The cooperative hierarchy returned without deadline failure. Subsystem errors can
    /// coexist with complete async shutdown; external payload settlement is not inferred.
    pub fn tasks_completed(&self) -> bool {
        !self.timed_out
    }
}

/// Run the concrete invocation owner under the native subsystem hierarchy. The owner can
/// create named children and explicitly order independent capture/cleanup through native
/// SubsystemHandle/NestedSubsystem APIs, without a second host task registry.
///
/// The external token belongs to the signal/origin owner. This function does not install
/// process-wide signal handlers, replay effects, or treat async abortion as OS cleanup.
pub async fn supervise<Owner>(
    cancellation: CancellationToken,
    shutdown_timeout: Duration,
    owner: Owner,
) -> ShutdownReport
where
    Owner:
        'static + for<'a> AsyncSubsysFn<&'a mut SubsystemHandle<HostError>, Result<(), HostError>>,
{
    let result = Toplevel::<HostError>::new_with_shutdown_token(
        async move |root: &mut SubsystemHandle<HostError>| {
            root.start(SubsystemBuilder::new("invocation-owner", owner));
        },
        cancellation,
    )
    .handle_shutdown_requests(shutdown_timeout)
    .await;
    match result {
        Ok(()) => ShutdownReport {
            timed_out: false,
            failures: Vec::new(),
        },
        Err(error) => {
            let timed_out = matches!(&error, GracefulShutdownError::ShutdownTimeout(_));
            let failures = error
                .into_subsystem_errors()
                .into_vec()
                .into_iter()
                .map(|error| match error {
                    SubsystemError::Failed(name, error) => SubsystemFailure::Failed {
                        name: name.to_string(),
                        cause: Box::new(error.into_error()),
                    },
                    SubsystemError::Panicked(name) => SubsystemFailure::Panicked {
                        name: name.to_string(),
                    },
                })
                .collect();
            ShutdownReport {
                timed_out,
                failures,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[tokio::test]
    async fn actual_library_hierarchy_collects_independent_native_cleanup_failures() {
        let count = Arc::new(AtomicUsize::new(0));
        let cancellation = CancellationToken::new();
        let trigger = cancellation.clone();
        let observed = count.clone();
        let report = supervise(
            cancellation,
            Duration::from_secs(2),
            async move |owner: &mut SubsystemHandle<HostError>| {
                let mut children = Vec::new();
                for name in ["first-cleanup", "second-cleanup"] {
                    let count = observed.clone();
                    children.push(owner.start(SubsystemBuilder::new(
                        name,
                        async move |child: &mut SubsystemHandle<HostError>| {
                            child.on_shutdown_requested().await;
                            count.fetch_add(1, Ordering::SeqCst);
                            Err::<(), HostError>(std::io::Error::other(name).into())
                        },
                    )));
                }
                trigger.cancel();
                // Forwarding belongs to the native hierarchy. Joining every independent child
                // does not reconstruct a substitute error or short-circuit the second cleanup.
                for child in children {
                    let _ = child.join().await;
                }
                Ok(())
            },
        )
        .await;
        assert!(report.tasks_completed());
        assert_eq!(count.load(Ordering::SeqCst), 2);
        let mut errors: Vec<_> = report
            .failures
            .iter()
            .map(|error| match error {
                SubsystemFailure::Failed { cause, .. } => match cause.as_ref() {
                    HostError::Io(error) => error.to_string(),
                    other => panic!("original IO cause lost: {other}"),
                },
                other => panic!("unexpected failure: {other}"),
            })
            .collect();
        errors.sort();
        assert_eq!(errors, ["first-cleanup", "second-cleanup"]);
    }

    #[tokio::test]
    async fn deadline_is_not_reported_as_task_or_payload_completion() {
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let report = supervise(
            cancellation,
            Duration::from_millis(20),
            async |_owner: &mut SubsystemHandle<HostError>| {
                std::future::pending::<Result<(), HostError>>().await
            },
        )
        .await;
        assert!(report.timed_out);
        assert!(!report.tasks_completed());
    }

    #[tokio::test]
    async fn normal_owner_completion_requires_no_signal_or_extra_registry() {
        let report = supervise(
            CancellationToken::new(),
            Duration::from_secs(1),
            async |_owner: &mut SubsystemHandle<HostError>| Ok(()),
        )
        .await;
        assert!(report.tasks_completed());
        assert!(report.failures.is_empty());
    }
}
