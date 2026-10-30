// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent stdout/stderr drainage, required attributed bytes and optional presentation.
use crate::{HostError, extraction::OutputStream};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    future::Future,
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Instant,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    sync::{Mutex, mpsc},
    task::JoinHandle,
};

const CHUNK_BYTES: usize = 64 * 1024;
/// Maximum complete NDJSON record produced by this codec, independent of workload limits.
pub const MAX_LOG_RECORD_BYTES: usize = CHUNK_BYTES * 2;

/// Versioned exact-byte raw evidence, independent of any human/protocol interpretation.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LogChunk {
    /// Host-owned raw log representation version.
    pub version: u32,
    /// Observed log order, not an assertion about writes across independent pipes.
    pub sequence: u64,
    /// Original child pipe.
    pub stream: OutputStream,
    /// Starting byte offset in this pipe.
    pub byte_offset: u64,
    /// Original byte extent.
    pub byte_length: u64,
    /// Exact bytes, including non-UTF-8 and partial lines.
    pub bytes_base64: String,
}
impl LogChunk {
    /// Decode a complete raw record for a viewer or evidence reader.
    pub fn decode(&self) -> Result<Vec<u8>, LogDecodeError> {
        if self.version != 1 {
            return Err(LogDecodeError::Version(self.version));
        }
        if self.byte_length > CHUNK_BYTES as u64
            || self.bytes_base64.len() > CHUNK_BYTES.div_ceil(3) * 4
        {
            return Err(LogDecodeError::Limit);
        }
        let bytes = STANDARD.decode(&self.bytes_base64)?;
        if bytes.len() as u64 != self.byte_length {
            return Err(LogDecodeError::Length);
        }
        Ok(bytes)
    }
}
/// A malformed raw-log record is not empty child output.
#[derive(Debug, thiserror::Error)]
pub enum LogDecodeError {
    /// Unknown host-owned log codec.
    #[error("unsupported raw log version {0}")]
    Version(u32),
    /// Original encoding failure.
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
    /// Attribution does not match the decoded bytes.
    #[error("raw log byte length differs from decoded payload")]
    Length,
    /// The record exceeds the supported exact-byte chunk codec.
    #[error("raw log chunk exceeds its decoding policy")]
    Limit,
}
pse_diagnostics::impl_diagnostic! {
    LogDecodeError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}

/// Required log custody, including explicit synchronization before declaring a complete log.
pub trait LogSink: AsyncWrite + Unpin + Send + 'static {
    /// Complete the sink's durability operation; flush alone is not durable storage.
    fn synchronize(&mut self) -> impl Future<Output = io::Result<()>> + Send;
}
impl LogSink for tokio::fs::File {
    async fn synchronize(&mut self) -> io::Result<()> {
        self.sync_all().await
    }
}

/// Bounded, nonblocking interpretation of borrowed bytes after their required log attempt.
/// Implementations retain their own framing/record bounds and semantic completeness state.
pub trait ChunkObserver: Send + 'static {
    /// The original concrete error, retained without message parsing or type erasure.
    type Error: Send + 'static;
    /// Observe every drained chunk, including after a previous decode/log failure.
    fn observe(
        &mut self,
        stream: OutputStream,
        byte_offset: u64,
        bytes: &[u8],
    ) -> Result<(), Self::Error>;
}
impl<F, E> ChunkObserver for F
where
    F: FnMut(OutputStream, u64, &[u8]) -> Result<(), E> + Send + 'static,
    E: Send + 'static,
{
    type Error = E;
    fn observe(&mut self, stream: OutputStream, byte_offset: u64, bytes: &[u8]) -> Result<(), E> {
        self(stream, byte_offset, bytes)
    }
}

#[derive(Default)]
struct Progress {
    bytes: AtomicU64,
    eof: AtomicBool,
}
struct State<W: LogSink, O: ChunkObserver> {
    sink: W,
    observer: Option<O>,
    sequence: u64,
    committed: u64,
    log_error: Option<io::Error>,
    observer_errors: [Option<O::Error>; 2],
    observer_error_counts: [u64; 2],
    presentation: Option<mpsc::Sender<LogChunk>>,
    presented: u64,
    lost_chunks: u64,
    lost_bytes: u64,
    presentation_closed: bool,
}
fn index(stream: OutputStream) -> usize {
    match stream {
        OutputStream::Stdout => 0,
        OutputStream::Stderr => 1,
    }
}
impl<W: LogSink, O: ChunkObserver> State<W, O> {
    async fn accept(&mut self, stream: OutputStream, offset: u64, bytes: &[u8]) {
        let sequence = self.sequence;
        self.sequence = self.sequence.saturating_add(1);
        let permit = match self.presentation.as_ref().map(mpsc::Sender::try_reserve) {
            Some(Ok(permit)) => Some(permit),
            Some(Err(error)) => {
                self.lost_chunks = self.lost_chunks.saturating_add(1);
                self.lost_bytes = self.lost_bytes.saturating_add(bytes.len() as u64);
                if matches!(error, mpsc::error::TrySendError::Closed(_)) {
                    self.presentation_closed = true;
                }
                None
            }
            None => None,
        };
        // Avoid encoding/copying discarded optional data once required storage has failed.
        let record = (self.log_error.is_none() || permit.is_some()).then(|| LogChunk {
            version: 1,
            sequence,
            stream,
            byte_offset: offset,
            byte_length: bytes.len() as u64,
            bytes_base64: STANDARD.encode(bytes),
        });
        if self.log_error.is_none()
            && let Some(record) = &record
        {
            match serde_json::to_vec(record).map_err(io::Error::other) {
                Ok(mut encoded) => {
                    encoded.push(b'\n');
                    match self.sink.write_all(&encoded).await {
                        Ok(()) => self.committed = self.committed.saturating_add(1),
                        Err(error) => self.log_error = Some(error),
                    }
                }
                Err(error) => self.log_error = Some(error),
            }
        }
        if let Some(observer) = &mut self.observer
            && let Err(error) = observer.observe(stream, offset, bytes)
        {
            let slot = index(stream);
            self.observer_error_counts[slot] = self.observer_error_counts[slot].saturating_add(1);
            if self.observer_errors[slot].is_none() {
                self.observer_errors[slot] = Some(error);
            }
        }
        if let (Some(permit), Some(record)) = (permit, record) {
            permit.send(record);
            self.presented = self.presented.saturating_add(1);
        }
    }
}

/// Independent pipe observation facts; neither EOF nor task join proves workload settlement.
#[derive(Debug)]
pub struct PipeReport {
    /// Original stream.
    pub stream: OutputStream,
    /// Actual bytes read, including bytes lost to a later failed required sink.
    pub bytes_read: u64,
    /// Actual EOF observed by the pipe reader.
    pub eof: bool,
    /// Native read failure, if any.
    pub read_error: Option<io::Error>,
    /// Original task failure, including cancellation after an observation deadline.
    pub task_error: Option<tokio::task::JoinError>,
    /// Output observation expired; this is not a workload deadline or cancellation claim.
    pub observation_expired: bool,
}
impl PipeReport {
    /// All original bytes up to actual EOF were drained by this reader.
    pub fn complete(&self) -> bool {
        self.eof && self.read_error.is_none() && self.task_error.is_none()
    }
}

/// Captured output and its original live interpretation/sink custody.
pub struct TeeReport<W: LogSink, O: ChunkObserver> {
    /// Actual stdout observation.
    pub stdout: PipeReport,
    /// Actual stderr observation.
    pub stderr: PipeReport,
    /// Number of fully accepted NDJSON raw records before any storage failure.
    pub committed_records: u64,
    /// Original required write/encoding failure; further writes were discarded.
    pub log_error: Option<io::Error>,
    /// Original flush failure, independent of write failures.
    pub flush_error: Option<io::Error>,
    /// Original synchronization failure, independent of write/flush failures.
    pub synchronization_error: Option<io::Error>,
    /// First concrete observer failure in each stream; later chunks still reached the observer.
    pub observer_errors: [Option<O::Error>; 2],
    /// Every observer failure count, bounded in representation.
    pub observer_error_counts: [u64; 2],
    /// Optional chunks accepted by a queue, not proof of terminal display/consumption.
    pub presentation_queued: u64,
    /// Optional chunks not enqueued because the view was full/closed.
    pub presentation_lost_chunks: u64,
    /// Original bytes represented by those optional losses.
    pub presentation_lost_bytes: u64,
    /// The optional receiver was observed closed.
    pub presentation_closed: bool,
    /// Output finalization expired. Tokio file work may outlive an aborted future.
    pub io_unresolved: bool,
    /// The original interpreter, for its owner to reconcile semantic completeness.
    pub observer: Option<O>,
    /// The original writer remains owned for pending-I/O recovery or explicit disposal.
    pub sink: W,
}
impl<W: LogSink, O: ChunkObserver> fmt::Debug for TeeReport<W, O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TeeReport")
            .field("stdout", &self.stdout)
            .field("stderr", &self.stderr)
            .field("committed_records", &self.committed_records)
            .field("io_unresolved", &self.io_unresolved)
            .finish_non_exhaustive()
    }
}
impl<W: LogSink, O: ChunkObserver> TeeReport<W, O> {
    /// Complete required raw log coverage, independent of decoding and optional presentation.
    pub fn log_complete(&self) -> bool {
        self.stdout.complete()
            && self.stderr.complete()
            && self.log_error.is_none()
            && self.flush_error.is_none()
            && self.synchronization_error.is_none()
            && !self.io_unresolved
    }
    /// Byte delivery to the interpreter completed; the interpreter owns semantic reconciliation.
    pub fn observer_complete(&self) -> bool {
        self.stdout.complete()
            && self.stderr.complete()
            && self.observer_errors.iter().all(Option::is_none)
    }
}

/// Original reader task custody. Dropping does not establish any physical/resource settlement.
pub struct TeeHandle<W: LogSink, O: ChunkObserver> {
    stdout: JoinHandle<io::Result<()>>,
    stderr: JoinHandle<io::Result<()>>,
    progress: [Arc<Progress>; 2],
    state: Option<Arc<Mutex<State<W, O>>>>,
}
impl<W: LogSink, O: ChunkObserver> fmt::Debug for TeeHandle<W, O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TeeHandle").finish_non_exhaustive()
    }
}
async fn read_pipe<R: AsyncRead + Unpin, W: LogSink, O: ChunkObserver>(
    mut reader: R,
    stream: OutputStream,
    progress: Arc<Progress>,
    state: Arc<Mutex<State<W, O>>>,
) -> io::Result<()> {
    let mut buffer = vec![0_u8; CHUNK_BYTES];
    loop {
        let count = reader.read(&mut buffer).await?;
        if count == 0 {
            progress.eof.store(true, Ordering::Release);
            return Ok(());
        }
        let offset = progress.bytes.fetch_add(count as u64, Ordering::AcqRel);
        state
            .lock()
            .await
            .accept(stream, offset, &buffer[..count])
            .await;
    }
}

/// Start independent readers. The caller publishes its selection and owns process placement.
pub fn start<R1, R2, W, O>(
    stdout: R1,
    stderr: R2,
    sink: W,
    observer: O,
    presentation: Option<mpsc::Sender<LogChunk>>,
) -> TeeHandle<W, O>
where
    R1: AsyncRead + Unpin + Send + 'static,
    R2: AsyncRead + Unpin + Send + 'static,
    W: LogSink,
    O: ChunkObserver,
{
    let progress = [Arc::new(Progress::default()), Arc::new(Progress::default())];
    let state = Arc::new(Mutex::new(State {
        sink,
        observer: Some(observer),
        sequence: 0,
        committed: 0,
        log_error: None,
        observer_errors: [None, None],
        observer_error_counts: [0, 0],
        presentation,
        presented: 0,
        lost_chunks: 0,
        lost_bytes: 0,
        presentation_closed: false,
    }));
    let stdout = tokio::spawn(read_pipe(
        stdout,
        OutputStream::Stdout,
        progress[0].clone(),
        state.clone(),
    ));
    let stderr = tokio::spawn(read_pipe(
        stderr,
        OutputStream::Stderr,
        progress[1].clone(),
        state.clone(),
    ));
    TeeHandle {
        stdout,
        stderr,
        progress,
        state: Some(state),
    }
}

async fn finish_pipe(
    task: &mut JoinHandle<io::Result<()>>,
    stream: OutputStream,
    progress: &Progress,
    deadline: Instant,
) -> PipeReport {
    let (result, expired) = match tokio::time::timeout_at(deadline.into(), &mut *task).await {
        Ok(result) => (result, false),
        Err(_) => {
            task.abort();
            (task.await, true)
        }
    };
    let (read_error, task_error) = match result {
        Ok(Ok(())) => (None, None),
        Ok(Err(error)) => (Some(error), None),
        Err(error) => (None, Some(error)),
    };
    PipeReport {
        stream,
        bytes_read: progress.bytes.load(Ordering::Acquire),
        eof: progress.eof.load(Ordering::Acquire),
        read_error,
        task_error,
        observation_expired: expired,
    }
}
impl<W: LogSink, O: ChunkObserver> TeeHandle<W, O> {
    /// Wait for output under its own absolute observation deadline, without stopping a workload.
    /// Expiry cancels readers and keeps I/O custody unresolved; no child/cgroup claim is inferred.
    pub async fn finish(mut self, deadline: Instant) -> Result<TeeReport<W, O>, HostError> {
        let (stdout, stderr) = tokio::join!(
            finish_pipe(
                &mut self.stdout,
                OutputStream::Stdout,
                &self.progress[0],
                deadline
            ),
            finish_pipe(
                &mut self.stderr,
                OutputStream::Stderr,
                &self.progress[1],
                deadline
            )
        );
        let state = self
            .state
            .take()
            .ok_or_else(|| HostError::Ownership("raw output state custody is missing".into()))?;
        let state = Arc::try_unwrap(state)
            .map_err(|_| {
                HostError::Ownership(
                    "raw output readers retain shared state custody after join".into(),
                )
            })?
            .into_inner();
        let State {
            mut sink,
            observer,
            committed,
            log_error,
            observer_errors,
            observer_error_counts,
            presented,
            lost_chunks,
            lost_bytes,
            presentation_closed,
            ..
        } = state;
        let mut unresolved = stdout.observation_expired
            || stderr.observation_expired
            || stdout.task_error.is_some()
            || stderr.task_error.is_some();
        let flush_error = match tokio::time::timeout_at(deadline.into(), sink.flush()).await {
            Ok(result) => result.err(),
            Err(_) => {
                unresolved = true;
                Some(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "raw log flush observation expired",
                ))
            }
        };
        let synchronization_error =
            match tokio::time::timeout_at(deadline.into(), sink.synchronize()).await {
                Ok(result) => result.err(),
                Err(_) => {
                    unresolved = true;
                    Some(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "raw log synchronization observation expired",
                    ))
                }
            };
        Ok(TeeReport {
            stdout,
            stderr,
            committed_records: committed,
            log_error,
            flush_error,
            synchronization_error,
            observer_errors,
            observer_error_counts,
            presentation_queued: presented,
            presentation_lost_chunks: lost_chunks,
            presentation_lost_bytes: lost_bytes,
            presentation_closed,
            io_unresolved: unresolved,
            observer,
            sink,
        })
    }
}

impl<W: LogSink, O: ChunkObserver> Drop for TeeHandle<W, O> {
    fn drop(&mut self) {
        self.stdout.abort();
        self.stderr.abort();
    }
}

#[cfg(test)]
mod tests;
