// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit, bounded diagnostic capture. A session owns exporters, never scientific work.

pub use pse_diagnostics::{CaptureKind, CaptureOutputKind, CaptureStatus};

use crate::observation_detail::{CaptureBudget, CaptureError};
use crate::observation::{ProcessLocator, read_checkpoint};
use pse_diagnostics::observation::OperationId;
use pse_model::document::Version;
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
    time::Duration,
};

mod cpu;
mod launch;
pub(crate) use launch::spawn_owned_command;
pub use launch::{LaunchCaptureRequest, LaunchedCommand, launch_capture};
mod handoff;
mod recording;
pub use handoff::{
    ReviewHandoffAcknowledgement, ReviewHandoffMember, ReviewHandoffReceipt, ReviewHandoffRequest,
    ReviewHandoffState, read_review_handoff,
};
pub use recording::{
    SOURCE_RECORDING_PROTOCOL_BYTES, SOURCE_RECORDING_REQUEST_ENV, SourceRecordingAcknowledgement,
    SourceRecordingFamilyAcknowledgement, SourceRecordingRequest, SourceRecordingSession,
    consume_source_recording_request, source_recording_from_environment, start_source_recording,
};
pub mod evidence;
mod layer;
mod timeline;
pub use layer::CaptureLayer;
#[cfg(test)]
mod tests;

fn default_duration() -> Duration {
    Duration::from_secs(10)
}
/// Explicit capture request; passive inspection never creates one implicitly.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptureRequest {
    /// Standalone request encoding version.
    pub version: Version<1>,
    /// Exact published root process checkpoint.
    pub checkpoint: PathBuf,
    /// Selected root diagnostic operation, not a numerical result identity.
    pub operation: OperationId,
    /// Optional evidence requested by the caller.
    pub kind: CaptureKind,
    /// New directory reserved exclusively for this capture; existing paths are refused.
    pub output: PathBuf,
    /// Actual collection duration from attachment; no preceding history is sampled.
    #[serde(default = "default_duration")]
    #[schemars(with = "pse_model::document::ClosedDuration")]
    pub duration: Duration,
    /// Explicit collection allowances.
    #[serde(default)]
    pub budget: CaptureBudget,
}
/// A retained file and the readable complete prefix observed by its owner.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptureOutput {
    /// File owned by the capture, retained after session stop/drop.
    pub path: PathBuf,
    /// File interpretation.
    pub kind: CaptureOutputKind,
    /// Bytes last observed, possibly including an incomplete tail.
    pub bytes: u64,
    /// Flushed complete packet offset, or whole file length after successful tool completion.
    pub readable_prefix_bytes: u64,
    /// The published prefix can be decoded in this format.
    pub readable: bool,
}
/// One actual process followed by capture; process CPU is never operation-exclusive CPU.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptureTarget {
    /// Diagnostic process instance, scoped independently from OS PID reuse.
    pub process: String,
    /// OS attachment identity copied from its owner.
    pub locator: ProcessLocator,
    /// Explicit known same-host alignment availability.
    pub clock_aligned: bool,
    /// Reported clock mapping uncertainty; missing is unavailable, never exact zero.
    pub clock_uncertainty_nanoseconds: Option<u64>,
    /// Last published actual native thread IDs for the selected operation set.
    pub thread_ids: Vec<u32>,
    /// Target also had operations outside the selected graph; samples are inclusive.
    pub concurrent_operations: bool,
    /// Thread publication was bounded or unavailable.
    pub threads_partial: bool,
}
/// Symbol evidence from a completed bounded perf-script decode, never just file creation.
#[derive(Clone, Debug, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptureSymbols {
    /// Actual decoded frames with names other than unknown/address placeholders.
    pub resolved_frames: u64,
    /// Actual decoded frames without useful names.
    pub unresolved_frames: u64,
    /// Small recognizable name prefix, not proof of every frame or correct unwinding.
    pub names: Vec<String>,
    /// Decoder successfully completed; false also covers not selected/not yet attempted.
    pub checked: bool,
}
/// Independent capture losses and unavailable owner facts.
#[derive(Clone, Debug, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptureLoss {
    /// Lifecycle packets refused by the bounded optional caller-composed layer.
    pub queued_packets_dropped: u64,
    /// Exporter identity/event state refused by its working allowance.
    pub refused_items: u64,
    /// The retained output allowance ended collection.
    pub output_exhausted: bool,
    /// At least one owner checkpoint reported observation or thread loss.
    pub source_partial: bool,
    /// Some actual-time facts could not align to the same-host clock and were omitted.
    pub unknown_clock: bool,
    /// CPU kernel loss was not quantified by this adapter; never silently reported as zero.
    pub sampling_loss_unknown: bool,
    /// CPU sampling inherits newly created threads and forked tasks, beyond published thread IDs.
    pub sampling_inherited_tasks: bool,
    /// Independently attached related processes may duplicate inherited samples; never sum them.
    pub sampling_overlap_possible: bool,
}
/// Typed capture answer; it contains no scientific results or result permissions.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptureDocument {
    /// Standalone answer encoding version.
    pub version: Version<2>,
    /// Explicitly selected mechanism.
    pub kind: CaptureKind,
    /// Selected root diagnostic operation, absent for a process launch without one.
    pub operation: Option<OperationId>,
    /// Actual published operation relationships; an empty list means not observed.
    pub related_operations: Vec<OperationId>,
    /// Exact launched command process; absent for checkpoint-based attachment.
    pub launch_process: Option<ProcessLocator>,
    /// Exporter lifecycle only.
    pub status: CaptureStatus,
    /// Requested attachment duration.
    #[schemars(with = "pse_model::document::ClosedDuration")]
    pub requested_duration: Duration,
    /// Actual elapsed collection interval from exporter start (no retrospective CPU samples).
    #[schemars(with = "pse_model::document::ClosedDuration")]
    pub observed_duration: Duration,
    /// Known process and actual native-thread attribution; CPU includes inherited tasks.
    pub targets: Vec<CaptureTarget>,
    /// Retained raw and projected files, including readable prefix boundaries.
    pub outputs: Vec<CaptureOutput>,
    /// Explicit bounded-loss and unavailable-clock facts.
    pub loss: CaptureLoss,
    /// CPU symbol evidence; timeline collection leaves this unchecked.
    pub symbols: CaptureSymbols,
    /// Intervals actually begun without an observed actual end, including interrupted ones.
    pub open_intervals: usize,
    /// Bounded independent diagnostic failures; none changes a target outcome.
    pub errors: Vec<String>,
}
type Shared = Arc<Mutex<CaptureDocument>>;
/// Owns only the capture worker and its child tools. Dropping requests stop and joins them.
/// It never holds an operation/native cancellation handle or a scientific store lease.
#[derive(Debug)]
pub struct CaptureSession {
    stop: Arc<AtomicBool>,
    state: Shared,
    layer: Option<CaptureLayer>,
    worker: Option<JoinHandle<()>>,
}
impl CaptureSession {
    /// Nonblocking request to stop only this exporter/sampler.
    pub fn request_stop(&self) {
        self.stop.store(true, Ordering::Release);
    }
    /// Copy the latest flushed prefix and independent capture state without waiting for target work.
    pub fn snapshot(&self) -> CaptureDocument {
        document(&self.state).clone()
    }
    /// Whether the owned worker returned, including a panic; this does not inspect target science.
    pub fn is_finished(&self) -> bool {
        self.worker.as_ref().is_none_or(JoinHandle::is_finished)
    }
    /// Optional layer for caller composition. No global subscriber is installed or replaced.
    /// CPU sessions have no timeline layer; ordinary checkpoints never require one.
    pub fn layer(&self) -> Option<CaptureLayer> {
        self.layer.clone()
    }
    /// Wait for the selected interval or exporter failure, then join only owned workers.
    pub fn wait(mut self) -> Result<CaptureDocument, CaptureError> {
        if let Some(worker) = self.worker.take() {
            worker.join().map_err(|_| CaptureError::Worker)?;
        }
        Ok(self.snapshot())
    }
    /// Request capture stop and join owned workers; target work continues independently.
    pub fn stop(self) -> Result<CaptureDocument, CaptureError> {
        self.request_stop();
        self.wait()
    }
}
impl Drop for CaptureSession {
    fn drop(&mut self) {
        self.request_stop();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
fn document(state: &Shared) -> std::sync::MutexGuard<'_, CaptureDocument> {
    state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
fn failure(state: &Shared, message: impl ToString) {
    let mut state = document(state);
    if state.errors.len() < 16 {
        let text = message.to_string();
        state.errors.push(text.chars().take(512).collect());
    } else {
        state.loss.refused_items = state.loss.refused_items.saturating_add(1);
    }
}
fn target(
    image: &crate::observation::ProcessCheckpoint,
    selected: &[OperationId],
) -> CaptureTarget {
    CaptureTarget {
        process: image.process.clone(),
        locator: image.locator.clone(),
        clock_aligned: image.clock.is_some(),
        clock_uncertainty_nanoseconds: image
            .clock
            .as_ref()
            .map(|clock| clock.uncertainty_nanoseconds),
        thread_ids: image
            .threads
            .iter()
            .filter(|thread| selected.contains(&thread.operation))
            .take(32)
            .map(|thread| thread.tid)
            .collect(),
        concurrent_operations: image
            .operations
            .iter()
            .any(|operation| operation.finished_at.is_none() && !selected.contains(&operation.id)),
        threads_partial: image.threads_partial,
    }
}
fn source_partial(image: &crate::observation::ProcessCheckpoint, selected: &[OperationId]) -> bool {
    let missing = |coverage: &pse_diagnostics::observation::Coverage| {
        !coverage.enabled
            || !coverage.available
            || coverage.curtailed
            || coverage.dropped_updates > 0
            || coverage.refused_items > 0
    };
    image.omitted > 0
        || image.threads_partial
        || missing(&image.registry_coverage)
        || image
            .operations
            .iter()
            .any(|operation| selected.contains(&operation.id) && missing(&operation.coverage))
}
/// Start explicit capture from an already-published exact operation checkpoint.
/// Existing output paths, absent operations and unauthenticated CPU PIDs are refused.
pub fn start_capture(request: CaptureRequest) -> Result<CaptureSession, CaptureError> {
    if request.duration.is_zero() || request.duration > Duration::from_secs(3600) {
        return Err(CaptureError::Invalid(
            "duration must be positive and at most one hour".into(),
        ));
    }
    if request.budget.memory_bytes < 4 << 20 || request.budget.output_bytes == 0 {
        return Err(CaptureError::Invalid(
            "exporter needs at least 4 MiB working memory and positive output allowance".into(),
        ));
    }
    let image = read_checkpoint(&request.checkpoint)?;
    let operation = image
        .operations
        .iter()
        .find(|operation| operation.id == request.operation)
        .ok_or_else(|| {
            CaptureError::Invalid(
                "selected operation is absent from the published checkpoint".into(),
            )
        })?;
    if request.kind == CaptureKind::Cpu && operation.finished_at.is_some() {
        return Err(CaptureError::Invalid("CPU capture requires an active operation; select an active related operation or use retained timeline history".into()));
    }
    if request.kind == CaptureKind::Cpu && !image.locator.is_current() {
        return Err(CaptureError::Invalid(
            "CPU target process identity is unavailable or no longer current".into(),
        ));
    }
    std::fs::create_dir(&request.output)?;
    let stop = Arc::new(AtomicBool::new(false));
    let state = Arc::new(Mutex::new(CaptureDocument {
        version: Version,
        kind: request.kind,
        operation: Some(request.operation),
        related_operations: vec![request.operation],
        launch_process: None,
        status: CaptureStatus::Running,
        requested_duration: request.duration,
        observed_duration: Duration::ZERO,
        targets: vec![target(&image, &[request.operation])],
        outputs: Vec::new(),
        loss: CaptureLoss::default(),
        symbols: CaptureSymbols::default(),
        open_intervals: 0,
        errors: Vec::new(),
    }));
    let (layer, receiver) = layer::channel(&request, &stop);
    let session_layer = (request.kind == CaptureKind::Timeline).then_some(layer);
    let worker_stop = stop.clone();
    let worker_state = state.clone();
    let worker = std::thread::Builder::new()
        .name("pse-capture".into())
        .spawn(move || {
            let result = match request.kind {
                CaptureKind::Timeline => {
                    timeline::run(&request, image, &worker_stop, &worker_state, receiver)
                }
                CaptureKind::Cpu => cpu::run(&request, image, &worker_stop, &worker_state),
            };
            if let Err(error) = result {
                failure(&worker_state, error);
            }
            let mut state = document(&worker_state);
            if state.status == CaptureStatus::Running {
                state.status = if !state.errors.is_empty() {
                    if state.outputs.iter().any(|output| output.readable) {
                        CaptureStatus::Partial
                    } else {
                        CaptureStatus::Failed
                    }
                } else if state.loss.output_exhausted
                    || state.loss.refused_items > 0
                    || state.loss.source_partial
                    || state.loss.unknown_clock
                    || state.loss.queued_packets_dropped > 0
                {
                    CaptureStatus::Partial
                } else if worker_stop.load(Ordering::Acquire) {
                    CaptureStatus::Stopped
                } else {
                    CaptureStatus::Completed
                };
            }
            worker_stop.store(true, Ordering::Release);
        })?;
    Ok(CaptureSession {
        stop,
        state,
        layer: session_layer,
        worker: Some(worker),
    })
}
