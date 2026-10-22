// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit cooperative inspection of a caller's actual executor.
//!
//! The collector waits outside the target executor. Ending that wait never claims
//! cancellation of Tokio's separately issued tracing request.

use crate::observation_detail::CaptureError;
use pse_model::document::Version;
use serde::{Deserialize, Serialize};
#[cfg(all(feature = "diagnostic-taskdump", tokio_unstable))]
use std::sync::atomic::{AtomicU64, Ordering};
use std::{
    sync::{Arc, Condvar, Mutex},
    time::{Duration, Instant},
};

/// Explicit diagnostic action, never an option on passive capture.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskDumpRequest {
    /// Request interpretation.
    pub version: Version<1>,
    /// Independent collector wait, not a scientific or target cancellation deadline.
    #[schemars(with = "pse_model::document::ClosedDuration")]
    pub wait: Duration,
    /// Returned task traces; requesting a dump still traces the executor's population.
    pub maximum_tasks: usize,
    /// Total retained UTF-8 trace bytes.
    pub maximum_bytes: usize,
}

/// Copied task trace from the selected actual executor.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskTrace {
    /// Tokio-local task identifier, never a scientific invocation identity.
    pub task: String,
    /// Actual cooperative trace, possibly curtailed.
    pub trace: String,
}

/// Independent collector result and actual target-request settlement.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskDumpDocument {
    /// Result interpretation.
    pub version: Version<1>,
    /// Process-local request identity, retained by its executor owner.
    pub request: u64,
    /// Actual process containing the executor.
    pub process: u32,
    /// Actual selected runtime identifier.
    pub executor: String,
    /// Pending, completed or collector_failed; pending never means drained.
    pub target_state: String,
    /// This observer stopped waiting while target work may continue.
    pub observer_wait_expired: bool,
    /// Retained original task traces.
    pub tasks: Vec<TaskTrace>,
    /// Retained traces curtailed; local executors can be absent independently.
    pub truncated: bool,
    /// Collector failure, independent of scientific outcome.
    pub failure: Option<String>,
    /// Coverage and perturbation, never a recovery guarantee.
    pub coverage: String,
}

#[derive(Debug)]
struct Shared {
    document: Mutex<TaskDumpDocument>,
    ready: Condvar,
}

/// The actual executor owner retains this ticket after an observer stops waiting.
#[derive(Debug)]
pub struct TaskDumpTicket {
    shared: Arc<Shared>,
}

impl TaskDumpTicket {
    /// Copy current request state without issuing another dump.
    pub fn snapshot(&self) -> Result<TaskDumpDocument, CaptureError> {
        self.shared
            .document
            .lock()
            .map(|value| value.clone())
            .map_err(|_| CaptureError::Invalid("task dump observation lock poisoned".into()))
    }

    /// Wait using an OS condition variable, independent of the target's timers/workers.
    pub fn wait(&self, allowance: Duration) -> Result<TaskDumpDocument, CaptureError> {
        if allowance.is_zero() || allowance > Duration::from_secs(60) {
            return Err(CaptureError::Invalid(
                "task dump observer wait must be positive and at most 60 seconds".into(),
            ));
        }
        let deadline = Instant::now() + allowance;
        let mut document = self
            .shared
            .document
            .lock()
            .map_err(|_| CaptureError::Invalid("task dump observation lock poisoned".into()))?;
        while document.target_state == "pending" {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }
            document = self
                .shared
                .ready
                .wait_timeout(document, remaining)
                .map_err(|_| CaptureError::Invalid("task dump observation lock poisoned".into()))?
                .0;
        }
        let mut copied = document.clone();
        copied.observer_wait_expired = copied.target_state == "pending";
        Ok(copied)
    }
}

/// Issue one cooperative request on the supplied actual multi-thread executor.
/// Caller ownership must retain the ticket through settlement, including observer timeout.
pub fn request(
    handle: tokio::runtime::Handle,
    request: TaskDumpRequest,
    cancel: &crate::CancelSource,
) -> Result<TaskDumpTicket, CaptureError> {
    cancel
        .checkpoint()
        .map_err(|_| CaptureError::Invalid("task dump cancelled before issuance".into()))?;
    if request.wait.is_zero()
        || request.wait > Duration::from_secs(60)
        || !(1..=4096).contains(&request.maximum_tasks)
        || !(1..=16_777_216).contains(&request.maximum_bytes)
    {
        return Err(CaptureError::Invalid(
            "invalid finite task dump request".into(),
        ));
    }
    issue(handle, request)
}

#[cfg(not(all(
    feature = "diagnostic-taskdump",
    tokio_unstable,
    target_os = "linux",
    any(
        target_arch = "x86_64",
        target_arch = "x86",
        target_arch = "aarch64",
        target_arch = "s390x"
    )
)))]
fn issue(
    _handle: tokio::runtime::Handle,
    _request: TaskDumpRequest,
) -> Result<TaskDumpTicket, CaptureError> {
    Err(CaptureError::Invalid(
        "task dump requires the explicit diagnostic-taskdump build on supported Linux".into(),
    ))
}

#[cfg(all(
    feature = "diagnostic-taskdump",
    tokio_unstable,
    target_os = "linux",
    any(
        target_arch = "x86_64",
        target_arch = "x86",
        target_arch = "aarch64",
        target_arch = "s390x"
    )
))]
fn issue(
    handle: tokio::runtime::Handle,
    request: TaskDumpRequest,
) -> Result<TaskDumpTicket, CaptureError> {
    use std::fmt::Write;
    if handle.runtime_flavor() != tokio::runtime::RuntimeFlavor::MultiThread {
        return Err(CaptureError::Invalid("external collector supports the actual multi-thread executor only; current-thread dumping must be in its own runtime context".into()));
    }
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let identity = NEXT
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            value.checked_add(1)
        })
        .map_err(|_| CaptureError::Invalid("task dump request identity exhausted".into()))?;
    let shared = Arc::new(Shared { document: Mutex::new(TaskDumpDocument {
        version: Version, request: identity, process: std::process::id(),
        executor: format!("{:?}", handle.id()), target_state: "pending".into(), observer_wait_expired: false,
        tasks: Vec::new(), truncated: false, failure: None,
        coverage: "actual supplied multi-thread executor; synchronization pauses workers and re-polls tasks; LocalSet/FuturesUnordered and native threads may be absent; collector timeout does not cancel an issued request".into(),
    }), ready: Condvar::new() });
    let retained = Arc::clone(&shared);
    std::thread::Builder::new()
        .name("pse-taskdump-observer".into())
        .spawn(move || {
            let observed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                handle.block_on(handle.dump())
            }));
            if let Ok(mut document) = retained.document.lock() {
                match observed {
                    Ok(dump) => {
                        let mut left = request.maximum_bytes;
                        for task in dump.tasks().iter() {
                            if document.tasks.len() == request.maximum_tasks || left == 0 {
                                document.truncated = true;
                                break;
                            }
                            let mut text = TraceWriter {
                                text: String::new(),
                                remaining: left,
                                truncated: false,
                            };
                            let _ = write!(text, "{}", task.trace());
                            left = text.remaining;
                            document.truncated |= text.truncated;
                            document.tasks.push(TaskTrace {
                                task: format!("{:?}", task.id()),
                                trace: text.text,
                            });
                        }
                        document.target_state = "completed".into();
                    }
                    Err(_) => {
                        document.target_state = "collector_failed".into();
                        document.failure = Some(
                            "cooperative dump collector unwound; target outcome unchanged".into(),
                        );
                    }
                }
                retained.ready.notify_all();
            }
        })?;
    Ok(TaskDumpTicket { shared })
}

#[cfg(all(feature = "diagnostic-taskdump", tokio_unstable))]
struct TraceWriter {
    text: String,
    remaining: usize,
    truncated: bool,
}
#[cfg(all(feature = "diagnostic-taskdump", tokio_unstable))]
impl std::fmt::Write for TraceWriter {
    fn write_str(&mut self, value: &str) -> std::fmt::Result {
        let mut end = value.len().min(self.remaining);
        while !value.is_char_boundary(end) {
            end -= 1;
        }
        self.text.push_str(&value[..end]);
        self.remaining -= end;
        self.truncated |= end != value.len();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independent_wait_preserves_pending_target_and_later_completion() {
        let shared = Arc::new(Shared {
            document: Mutex::new(TaskDumpDocument {
                version: Version,
                request: 1,
                process: std::process::id(),
                executor: "fixture".into(),
                target_state: "pending".into(),
                observer_wait_expired: false,
                tasks: vec![],
                truncated: false,
                failure: None,
                coverage: "independent owner fixture".into(),
            }),
            ready: Condvar::new(),
        });
        let ticket = TaskDumpTicket {
            shared: Arc::clone(&shared),
        };
        let first = ticket.wait(Duration::from_millis(1)).unwrap();
        assert!(first.observer_wait_expired);
        assert_eq!(first.target_state, "pending");
        assert_eq!(ticket.snapshot().unwrap().target_state, "pending");
        shared.document.lock().unwrap().target_state = "completed".into();
        let final_copy = ticket.wait(Duration::from_millis(1)).unwrap();
        assert!(!final_copy.observer_wait_expired);
        assert_eq!(final_copy.target_state, "completed");
    }
}
