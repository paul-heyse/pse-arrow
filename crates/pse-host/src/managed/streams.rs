// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent exact stdout/stderr custody, required interpretation and optional presentation.
use crate::{HostError, extraction::OutputStream};
use std::{
    fmt,
    future::Future,
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    sync::{Mutex, mpsc},
    task::JoinHandle,
};

const CHUNK_BYTES: usize = 64 * 1024;

/// Optional live bytes, never the retained raw representation or a cross-pipe producer order.
#[derive(Clone, Debug)]
pub struct PresentationChunk {
    /// Original child pipe.
    pub stream: OutputStream,
    /// Starting byte offset in this pipe, independent of retention or view loss.
    pub byte_offset: u64,
    /// Exact bytes, including non-UTF-8 and partial lines.
    pub bytes: Vec<u8>,
}

/// Required raw custody, including explicit synchronization before declaring complete storage.
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
struct ObserverState<O: ChunkObserver> {
    observer: O,
    errors: [Option<O::Error>; 2],
    error_counts: [u64; 2],
}
fn index(stream: OutputStream) -> usize {
    match stream {
        OutputStream::Stdout => 0,
        OutputStream::Stderr => 1,
    }
}

/// Optional presentation delivery facts for one pipe; queue acceptance is not consumption.
#[derive(Debug, Default)]
pub struct PresentationReport {
    /// Chunks accepted by the optional queue.
    pub queued: u64,
    /// Chunks not enqueued because the view was full/closed.
    pub lost_chunks: u64,
    /// Original bytes represented by those optional losses.
    pub lost_bytes: u64,
    /// The optional receiver was observed closed.
    pub closed: bool,
}

struct StreamState<W: LogSink> {
    sink: W,
    bytes_retained: u64,
    write_error: Option<io::Error>,
    io_unresolved: bool,
    presentation: PresentationReport,
}
impl<W: LogSink> StreamState<W> {
    fn new(sink: W) -> Self {
        Self {
            sink,
            bytes_retained: 0,
            write_error: None,
            io_unresolved: false,
            presentation: PresentationReport::default(),
        }
    }

    async fn retain(&mut self, bytes: &[u8], write_timeout: Duration) {
        if self.write_error.is_some() {
            return;
        }
        let Some(deadline) = Instant::now().checked_add(write_timeout) else {
            self.write_error = Some(io::Error::new(
                io::ErrorKind::InvalidInput,
                "raw stream write timeout exceeds the supported observation horizon",
            ));
            return;
        };
        let mut written = 0;
        while written < bytes.len() {
            match tokio::time::timeout_at(deadline.into(), self.sink.write(&bytes[written..])).await
            {
                Ok(Ok(0)) => {
                    self.write_error = Some(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "raw stream sink accepted no bytes",
                    ));
                    return;
                }
                Ok(Ok(count)) => {
                    written += count;
                    self.bytes_retained = self.bytes_retained.saturating_add(count as u64);
                }
                Ok(Err(error)) => {
                    self.write_error = Some(error);
                    return;
                }
                Err(_) => {
                    // A cancelled write may still have backing I/O in flight. Only completed
                    // write() advances count toward the confirmed prefix; its suffix is unknown.
                    self.io_unresolved = true;
                    self.write_error = Some(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "raw stream write observation expired",
                    ));
                    return;
                }
            }
        }
    }

    fn present(
        &mut self,
        sender: Option<&mpsc::Sender<PresentationChunk>>,
        stream: OutputStream,
        byte_offset: u64,
        bytes: &[u8],
    ) {
        let Some(sender) = sender else { return };
        match sender.try_reserve() {
            Ok(permit) => {
                permit.send(PresentationChunk {
                    stream,
                    byte_offset,
                    bytes: bytes.to_vec(),
                });
                self.presentation.queued = self.presentation.queued.saturating_add(1);
            }
            Err(error) => {
                self.presentation.lost_chunks = self.presentation.lost_chunks.saturating_add(1);
                self.presentation.lost_bytes = self
                    .presentation
                    .lost_bytes
                    .saturating_add(bytes.len() as u64);
                if matches!(error, mpsc::error::TrySendError::Closed(_)) {
                    self.presentation.closed = true;
                }
            }
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

/// One independently retained raw stream, with its original sink and failures.
#[derive(Debug)]
pub struct StreamReport<W: LogSink> {
    /// Actual reader observation after its task was joined.
    pub pipe: PipeReport,
    /// Confirmed contiguous prefix accepted by completed writes. Unresolved I/O may add bytes.
    /// Synchronization outcomes separately establish whether this prefix is durable.
    pub bytes_retained: u64,
    /// Original required write failure; later chunks were drained without further writes.
    pub write_error: Option<io::Error>,
    /// Original flush failure, independent of write failures.
    pub flush_error: Option<io::Error>,
    /// Original synchronization failure, independent of write/flush failures.
    pub synchronization_error: Option<io::Error>,
    /// Cancellation/expiry left underlying I/O custody unresolved.
    pub io_unresolved: bool,
    /// Optional presentation losses, independent of raw custody and interpretation.
    pub presentation: PresentationReport,
    /// Original writer retained for pending-I/O recovery or explicit disposal.
    pub sink: W,
}
impl<W: LogSink> StreamReport<W> {
    /// This stream reached actual EOF, retained every byte and synchronized without failure.
    pub fn log_complete(&self) -> bool {
        self.pipe.complete()
            && self.bytes_retained == self.pipe.bytes_read
            && self.write_error.is_none()
            && self.flush_error.is_none()
            && self.synchronization_error.is_none()
            && !self.io_unresolved
    }
}

/// Captured output and its original interpretation and independently owned raw sinks.
pub struct TeeReport<W1: LogSink, W2: LogSink, O: ChunkObserver> {
    /// Exact stdout retention and reader facts.
    pub stdout: StreamReport<W1>,
    /// Exact stderr retention and reader facts.
    pub stderr: StreamReport<W2>,
    /// First concrete observer failure in each stream; later chunks still reached the observer.
    pub observer_errors: [Option<O::Error>; 2],
    /// Every observer failure count, bounded in representation.
    pub observer_error_counts: [u64; 2],
    /// Original interpreter for its owner to reconcile semantic completeness.
    pub observer: Option<O>,
}
impl<W1: LogSink, W2: LogSink, O: ChunkObserver> fmt::Debug for TeeReport<W1, W2, O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TeeReport")
            .field("stdout", &self.stdout.pipe)
            .field("stderr", &self.stderr.pipe)
            .field("io_unresolved", &self.io_unresolved())
            .finish_non_exhaustive()
    }
}
impl<W1: LogSink, W2: LogSink, O: ChunkObserver> TeeReport<W1, W2, O> {
    /// Complete required raw coverage, independent of interpretation and optional presentation.
    pub fn log_complete(&self) -> bool {
        self.stdout.log_complete() && self.stderr.log_complete()
    }
    /// Byte delivery completed; the interpreter still owns semantic reconciliation.
    pub fn observer_complete(&self) -> bool {
        self.stdout.pipe.complete()
            && self.stderr.pipe.complete()
            && self.observer_errors.iter().all(Option::is_none)
    }
    /// Either stream has unresolved output I/O; this is not a workload settlement claim.
    pub fn io_unresolved(&self) -> bool {
        self.stdout.io_unresolved || self.stderr.io_unresolved
    }
}

/// Original reader task custody. Dropping does not establish physical/resource settlement.
pub struct TeeHandle<W1: LogSink, W2: LogSink, O: ChunkObserver> {
    stdout: JoinHandle<io::Result<()>>,
    stderr: JoinHandle<io::Result<()>>,
    progress: [Arc<Progress>; 2],
    stdout_state: Option<Arc<Mutex<StreamState<W1>>>>,
    stderr_state: Option<Arc<Mutex<StreamState<W2>>>>,
    observer_state: Option<Arc<Mutex<ObserverState<O>>>>,
}
impl<W1: LogSink, W2: LogSink, O: ChunkObserver> fmt::Debug for TeeHandle<W1, W2, O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TeeHandle").finish_non_exhaustive()
    }
}
async fn read_pipe<R: AsyncRead + Unpin, W: LogSink, O: ChunkObserver>(
    mut reader: R,
    stream: OutputStream,
    progress: Arc<Progress>,
    state: Arc<Mutex<StreamState<W>>>,
    observer_state: Arc<Mutex<ObserverState<O>>>,
    presentation: Option<mpsc::Sender<PresentationChunk>>,
    write_timeout: Duration,
) -> io::Result<()> {
    let mut buffer = vec![0_u8; CHUNK_BYTES];
    loop {
        let count = reader.read(&mut buffer).await?;
        if count == 0 {
            progress.eof.store(true, Ordering::Release);
            return Ok(());
        }
        let offset = progress.bytes.fetch_add(count as u64, Ordering::AcqRel);
        {
            let mut state = state.lock().await;
            state.retain(&buffer[..count], write_timeout).await;
        }
        {
            // Only synchronous bounded interpretation is shared, never sink I/O or backpressure.
            let mut state = observer_state.lock().await;
            if let Err(error) = state.observer.observe(stream, offset, &buffer[..count]) {
                let slot = index(stream);
                state.error_counts[slot] = state.error_counts[slot].saturating_add(1);
                if state.errors[slot].is_none() {
                    state.errors[slot] = Some(error);
                }
            }
        }
        state
            .lock()
            .await
            .present(presentation.as_ref(), stream, offset, &buffer[..count]);
    }
}

/// Start independent readers into distinct raw sinks (normally stdout.raw and stderr.raw).
/// The write timeout is explicit caller policy per chunk, not a workload/resource limit. On expiry
/// that sink becomes incomplete and subsequent pipe bytes are still drained and interpreted.
pub fn start<R1, R2, W1, W2, O>(
    stdout: R1,
    stderr: R2,
    sinks: (W1, W2),
    observer: O,
    presentation: Option<mpsc::Sender<PresentationChunk>>,
    write_timeout: Duration,
) -> TeeHandle<W1, W2, O>
where
    R1: AsyncRead + Unpin + Send + 'static,
    R2: AsyncRead + Unpin + Send + 'static,
    W1: LogSink,
    W2: LogSink,
    O: ChunkObserver,
{
    let progress = [Arc::new(Progress::default()), Arc::new(Progress::default())];
    let stdout_state = Arc::new(Mutex::new(StreamState::new(sinks.0)));
    let stderr_state = Arc::new(Mutex::new(StreamState::new(sinks.1)));
    let observer_state = Arc::new(Mutex::new(ObserverState {
        observer,
        errors: [None, None],
        error_counts: [0, 0],
    }));
    let stdout = tokio::spawn(read_pipe(
        stdout,
        OutputStream::Stdout,
        progress[0].clone(),
        stdout_state.clone(),
        observer_state.clone(),
        presentation.clone(),
        write_timeout,
    ));
    let stderr = tokio::spawn(read_pipe(
        stderr,
        OutputStream::Stderr,
        progress[1].clone(),
        stderr_state.clone(),
        observer_state.clone(),
        presentation,
        write_timeout,
    ));
    TeeHandle {
        stdout,
        stderr,
        progress,
        stdout_state: Some(stdout_state),
        stderr_state: Some(stderr_state),
        observer_state: Some(observer_state),
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

fn recover<T>(state: Option<Arc<Mutex<T>>>) -> Result<T, HostError> {
    let state =
        state.ok_or_else(|| HostError::Ownership("raw output state custody is missing".into()))?;
    Ok(Arc::try_unwrap(state)
        .map_err(|_| {
            HostError::Ownership("raw output readers retain state custody after join".into())
        })?
        .into_inner())
}

async fn finalize<W: LogSink>(
    mut state: StreamState<W>,
    pipe: PipeReport,
    deadline: Instant,
) -> StreamReport<W> {
    state.io_unresolved |= pipe.observation_expired || pipe.task_error.is_some();
    let flush_error = match tokio::time::timeout_at(deadline.into(), state.sink.flush()).await {
        Ok(result) => result.err(),
        Err(_) => {
            state.io_unresolved = true;
            Some(io::Error::new(
                io::ErrorKind::TimedOut,
                "raw stream flush observation expired",
            ))
        }
    };
    let synchronization_error =
        match tokio::time::timeout_at(deadline.into(), state.sink.synchronize()).await {
            Ok(result) => result.err(),
            Err(_) => {
                state.io_unresolved = true;
                Some(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "raw stream synchronization observation expired",
                ))
            }
        };
    StreamReport {
        pipe,
        bytes_retained: state.bytes_retained,
        write_error: state.write_error,
        flush_error,
        synchronization_error,
        io_unresolved: state.io_unresolved,
        presentation: state.presentation,
        sink: state.sink,
    }
}

impl<W1: LogSink, W2: LogSink, O: ChunkObserver> TeeHandle<W1, W2, O> {
    /// Wait under an absolute output-observation deadline, without stopping a workload.
    /// Active capture must remain alive while the payload drains. Expiry joins cancelled readers
    /// and retains unresolved I/O custody; it establishes no child/cgroup settlement claim.
    pub async fn finish(mut self, deadline: Instant) -> Result<TeeReport<W1, W2, O>, HostError> {
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
        let stdout_state = recover(self.stdout_state.take())?;
        let stderr_state = recover(self.stderr_state.take())?;
        let observer_state = recover(self.observer_state.take())?;
        let (stdout, stderr) = tokio::join!(
            finalize(stdout_state, stdout, deadline),
            finalize(stderr_state, stderr, deadline)
        );
        Ok(TeeReport {
            stdout,
            stderr,
            observer_errors: observer_state.errors,
            observer_error_counts: observer_state.error_counts,
            observer: Some(observer_state.observer),
        })
    }
}

impl<W1: LogSink, W2: LogSink, O: ChunkObserver> Drop for TeeHandle<W1, W2, O> {
    fn drop(&mut self) {
        self.stdout.abort();
        self.stderr.abort();
    }
}

#[cfg(test)]
mod tests;
