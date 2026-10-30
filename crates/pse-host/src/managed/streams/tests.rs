// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::sync::oneshot;

#[derive(Debug, Default)]
struct MemorySink {
    bytes: Vec<u8>,
    write_calls: usize,
    fail_after: Option<usize>,
    stall_after: Option<usize>,
    flush_fails: bool,
    sync_fails: bool,
    flush_calls: usize,
    sync_calls: usize,
}
impl AsyncWrite for MemorySink {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.write_calls += 1;
        let mut count = bytes.len();
        if let Some(limit) = self.fail_after {
            if self.bytes.len() >= limit {
                return Poll::Ready(Err(io::Error::other("disk full fixture")));
            }
            count = count.min(limit - self.bytes.len());
        }
        if let Some(limit) = self.stall_after {
            if self.bytes.len() >= limit {
                return Poll::Pending;
            }
            count = count.min(limit - self.bytes.len());
        }
        self.bytes.extend_from_slice(&bytes[..count]);
        Poll::Ready(Ok(count))
    }
    fn poll_flush(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.flush_calls += 1;
        Poll::Ready(if self.flush_fails {
            Err(io::Error::other("flush fixture"))
        } else {
            Ok(())
        })
    }
    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}
impl LogSink for MemorySink {
    async fn synchronize(&mut self) -> io::Result<()> {
        self.sync_calls += 1;
        if self.sync_fails {
            Err(io::Error::other("sync fixture"))
        } else {
            Ok(())
        }
    }
}
#[derive(Default)]
struct Collect {
    records: Vec<(OutputStream, u64, Vec<u8>)>,
    fail: bool,
    stderr_observed: Option<oneshot::Sender<()>>,
}
impl ChunkObserver for Collect {
    type Error = HostError;
    fn observe(
        &mut self,
        stream: OutputStream,
        offset: u64,
        bytes: &[u8],
    ) -> Result<(), HostError> {
        self.records.push((stream, offset, bytes.to_vec()));
        if stream == OutputStream::Stderr
            && let Some(sender) = self.stderr_observed.take()
        {
            let _ = sender.send(());
        }
        if self.fail {
            Err(HostError::Ownership("original observer fixture".into()))
        } else {
            Ok(())
        }
    }
}
fn observed(observer: &Collect, stream: OutputStream) -> Vec<u8> {
    let mut result = Vec::new();
    for (_, offset, bytes) in observer
        .records
        .iter()
        .filter(|(source, _, _)| *source == stream)
    {
        assert_eq!(*offset, result.len() as u64);
        result.extend_from_slice(bytes);
    }
    result
}
fn sinks() -> (MemorySink, MemorySink) {
    (MemorySink::default(), MemorySink::default())
}

#[tokio::test]
async fn both_streams_keep_exact_bytes_and_offsets_despite_slow_presentation() {
    let (mut stdout_write, stdout) = tokio::io::duplex(8192);
    let (mut stderr_write, stderr) = tokio::io::duplex(8192);
    let out = [0_u8, 0xff, b'a', b'\n'].repeat(30_000);
    let err = b"stderr\nwithout line semantics".repeat(4000);
    let (view, mut slow_receiver) = mpsc::channel(1);
    let tee = start(
        stdout,
        stderr,
        sinks(),
        Collect::default(),
        Some(view),
        Duration::from_secs(2),
    );
    let expected_out = out.clone();
    let expected_err = err.clone();
    let stdout_writer = tokio::spawn(async move {
        stdout_write.write_all(&out).await.unwrap();
        stdout_write.shutdown().await.unwrap();
    });
    let stderr_writer = tokio::spawn(async move {
        stderr_write.write_all(&err).await.unwrap();
        stderr_write.shutdown().await.unwrap();
    });
    let report = tee
        .finish(Instant::now() + Duration::from_secs(5))
        .await
        .unwrap();
    stdout_writer.await.unwrap();
    stderr_writer.await.unwrap();
    assert!(report.log_complete() && report.observer_complete());
    assert!(!report.io_unresolved());
    assert_eq!(report.stdout.sink.bytes, expected_out);
    assert_eq!(report.stderr.sink.bytes, expected_err);
    assert_eq!(report.stdout.bytes_retained, expected_out.len() as u64);
    assert_eq!(report.stderr.bytes_retained, expected_err.len() as u64);
    assert_eq!(report.stdout.sink.sync_calls, 1);
    assert_eq!(report.stderr.sink.sync_calls, 1);
    let view = slow_receiver.recv().await.unwrap();
    assert_eq!(view.byte_offset, 0);
    let expected = match view.stream {
        OutputStream::Stdout => &expected_out,
        OutputStream::Stderr => &expected_err,
    };
    assert_eq!(view.bytes, expected[..view.bytes.len()]);
    assert_eq!(
        report.stdout.presentation.queued + report.stderr.presentation.queued,
        1
    );
    assert!(report.stdout.presentation.lost_chunks + report.stderr.presentation.lost_chunks > 0);
    assert!(report.stdout.presentation.lost_bytes + report.stderr.presentation.lost_bytes > 0);
    let observer = report.observer.unwrap();
    assert_eq!(observed(&observer, OutputStream::Stdout), expected_out);
    assert_eq!(observed(&observer, OutputStream::Stderr), expected_err);
}

#[tokio::test]
async fn failed_stdout_keeps_its_exact_prefix_and_stderr_retains_every_byte() {
    let out = b"one large output line without a delimiter".repeat(4000);
    let err = b"stderr".repeat(7000);
    let report = start(
        io::Cursor::new(out.clone()),
        io::Cursor::new(err.clone()),
        (
            MemorySink {
                fail_after: Some(9),
                ..Default::default()
            },
            MemorySink::default(),
        ),
        Collect::default(),
        None,
        Duration::from_secs(2),
    )
    .finish(Instant::now() + Duration::from_secs(5))
    .await
    .unwrap();
    assert!(report.stdout.pipe.complete() && report.stderr.pipe.complete());
    assert!(!report.log_complete());
    assert!(report.observer_complete());
    assert!(report.stdout.write_error.is_some());
    assert_eq!(report.stdout.sink.bytes, out[..9]);
    assert_eq!(report.stdout.bytes_retained, 9);
    assert_eq!(
        report.stdout.sink.write_calls, 2,
        "partial write then failure, no later retry"
    );
    assert!(report.stderr.log_complete());
    assert_eq!(report.stderr.sink.bytes, err);
    let observer = report.observer.unwrap();
    assert_eq!(observed(&observer, OutputStream::Stdout), out);
    assert_eq!(observed(&observer, OutputStream::Stderr), err);
}

#[tokio::test]
async fn both_streams_preserve_independent_write_flush_sync_and_observer_failures() {
    let out = b"stdout".repeat(30_000);
    let err = b"stderr".repeat(30_000);
    let failing = || MemorySink {
        fail_after: Some(9),
        flush_fails: true,
        sync_fails: true,
        ..Default::default()
    };
    let report = start(
        io::Cursor::new(out.clone()),
        io::Cursor::new(err.clone()),
        (failing(), failing()),
        Collect {
            fail: true,
            ..Default::default()
        },
        None,
        Duration::from_secs(2),
    )
    .finish(Instant::now() + Duration::from_secs(5))
    .await
    .unwrap();
    assert!(!report.log_complete() && !report.observer_complete());
    for stream in [&report.stdout, &report.stderr] {
        assert!(stream.pipe.complete());
        assert!(stream.write_error.is_some());
        assert!(stream.flush_error.is_some());
        assert!(stream.synchronization_error.is_some());
        assert_eq!(stream.sink.flush_calls, 1);
        assert_eq!(stream.sink.sync_calls, 1);
        assert_eq!(stream.bytes_retained, 9);
    }
    assert!(report.observer_errors.iter().all(Option::is_some));
    assert!(report.observer_error_counts.iter().all(|count| *count > 1));
    let observer = report.observer.unwrap();
    assert_eq!(observed(&observer, OutputStream::Stdout), out);
    assert_eq!(observed(&observer, OutputStream::Stderr), err);
}

#[tokio::test]
async fn timed_out_write_preserves_known_prefix_and_continues_drain_and_extraction() {
    let out = b"stdout".repeat(30_000);
    let err = b"required stderr record".repeat(4000);
    let report = start(
        io::Cursor::new(out.clone()),
        io::Cursor::new(err.clone()),
        (
            MemorySink {
                stall_after: Some(9),
                ..Default::default()
            },
            MemorySink::default(),
        ),
        Collect::default(),
        None,
        Duration::from_millis(10),
    )
    .finish(Instant::now() + Duration::from_secs(3))
    .await
    .unwrap();
    assert!(report.stdout.pipe.complete() && report.stderr.pipe.complete());
    assert!(report.io_unresolved());
    assert!(!report.log_complete());
    assert!(report.observer_complete());
    assert_eq!(
        report.stdout.write_error.as_ref().unwrap().kind(),
        io::ErrorKind::TimedOut
    );
    assert_eq!(report.stdout.bytes_retained, 9);
    assert_eq!(report.stdout.sink.bytes, out[..9]);
    assert_eq!(report.stdout.sink.flush_calls, 1);
    assert_eq!(report.stdout.sink.sync_calls, 1);
    assert!(report.stderr.log_complete());
    assert_eq!(report.stderr.sink.bytes, err);
    let observer = report.observer.unwrap();
    assert_eq!(observed(&observer, OutputStream::Stdout), out);
    assert_eq!(observed(&observer, OutputStream::Stderr), err);
}

struct FailedReader {
    prefix: Option<&'static [u8]>,
}
impl AsyncRead for FailedReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buffer: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if let Some(prefix) = self.prefix.take() {
            buffer.put_slice(prefix);
            Poll::Ready(Ok(()))
        } else {
            Poll::Ready(Err(io::Error::other("original stdout read failure")))
        }
    }
}

#[tokio::test]
async fn reader_failure_retains_available_prefix_and_other_stream_failures_independently() {
    let report = start(
        FailedReader {
            prefix: Some(b"readable stdout prefix"),
        },
        io::Cursor::new(b"stderr"),
        (
            MemorySink {
                flush_fails: true,
                ..Default::default()
            },
            MemorySink {
                sync_fails: true,
                ..Default::default()
            },
        ),
        Collect::default(),
        None,
        Duration::from_secs(2),
    )
    .finish(Instant::now() + Duration::from_secs(2))
    .await
    .unwrap();
    assert!(!report.log_complete() && !report.observer_complete());
    assert!(!report.stdout.pipe.eof);
    assert_eq!(
        report.stdout.pipe.read_error.as_ref().unwrap().to_string(),
        "original stdout read failure"
    );
    assert_eq!(report.stdout.sink.bytes, b"readable stdout prefix");
    assert_eq!(report.stdout.bytes_retained, 22);
    assert!(report.stdout.flush_error.is_some());
    assert!(report.stderr.pipe.complete());
    assert!(report.stderr.synchronization_error.is_some());
    assert_eq!(report.stderr.sink.bytes, b"stderr");
    assert_eq!(report.stdout.sink.sync_calls, 1);
    assert_eq!(report.stderr.sink.sync_calls, 1);
}

struct BlockedSink {
    started: Option<oneshot::Sender<()>>,
    unblock: Option<oneshot::Receiver<()>>,
    inner: MemorySink,
}
impl AsyncWrite for BlockedSink {
    fn poll_write(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<io::Result<usize>> {
        if let Some(sender) = self.started.take() {
            let _ = sender.send(());
        }
        if let Some(receiver) = &mut self.unblock {
            match Pin::new(receiver).poll(context) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(_) => self.unblock = None,
            }
        }
        Pin::new(&mut self.inner).poll_write(context, bytes)
    }
    fn poll_flush(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(context)
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(context)
    }
}
impl LogSink for BlockedSink {
    async fn synchronize(&mut self) -> io::Result<()> {
        self.inner.synchronize().await
    }
}

#[tokio::test]
async fn stalled_stdout_sink_does_not_block_required_stderr_capture_or_interpretation() {
    let (started, waiting) = oneshot::channel();
    let (release, unblock) = oneshot::channel();
    let (observed_stderr, stderr_observation) = oneshot::channel();
    let (mut stderr_writer, stderr) = tokio::io::duplex(8192);
    let tee = start(
        io::Cursor::new(b"stdout"),
        stderr,
        (
            BlockedSink {
                started: Some(started),
                unblock: Some(unblock),
                inner: MemorySink::default(),
            },
            MemorySink::default(),
        ),
        Collect {
            stderr_observed: Some(observed_stderr),
            ..Default::default()
        },
        None,
        Duration::from_secs(10),
    );
    tokio::time::timeout(Duration::from_secs(2), waiting)
        .await
        .unwrap()
        .unwrap();
    stderr_writer
        .write_all(b"required stderr record")
        .await
        .unwrap();
    stderr_writer.shutdown().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), stderr_observation)
        .await
        .expect("stderr must reach required interpretation while stdout storage is stalled")
        .unwrap();
    release.send(()).unwrap();
    let report = tee
        .finish(Instant::now() + Duration::from_secs(3))
        .await
        .unwrap();
    assert!(report.log_complete() && report.observer_complete());
    assert_eq!(report.stdout.sink.inner.bytes, b"stdout");
    assert_eq!(report.stderr.sink.bytes, b"required stderr record");
}

#[tokio::test]
async fn closed_optional_receiver_does_not_degrade_required_evidence() {
    let (sender, receiver) = mpsc::channel(1);
    drop(receiver);
    let report = start(
        io::Cursor::new(b"stdout"),
        io::Cursor::new(b"stderr"),
        sinks(),
        Collect::default(),
        Some(sender),
        Duration::from_secs(2),
    )
    .finish(Instant::now() + Duration::from_secs(2))
    .await
    .unwrap();
    assert!(report.log_complete() && report.observer_complete());
    for stream in [&report.stdout, &report.stderr] {
        assert!(stream.presentation.closed);
        assert_eq!(stream.presentation.lost_chunks, 1);
        assert_eq!(stream.presentation.lost_bytes, 6);
    }
}

#[tokio::test]
async fn output_observation_expiry_joins_cancelled_readers_and_retains_custody() {
    let (_stdout_writer, stdout) = tokio::io::duplex(8);
    let (_stderr_writer, stderr) = tokio::io::duplex(8);
    let report = start(
        stdout,
        stderr,
        sinks(),
        Collect::default(),
        None,
        Duration::from_secs(2),
    )
    .finish(Instant::now() + Duration::from_millis(10))
    .await
    .unwrap();
    assert!(report.io_unresolved());
    assert!(!report.log_complete() && !report.observer_complete());
    for stream in [&report.stdout, &report.stderr] {
        assert!(stream.pipe.observation_expired);
        assert!(stream.pipe.task_error.as_ref().unwrap().is_cancelled());
        assert!(!stream.pipe.eof);
        assert_eq!(stream.sink.flush_calls, 1);
        assert_eq!(stream.sink.sync_calls, 1);
    }
}

struct StalledFinalization {
    flush_attempted: bool,
    synchronization_attempted: bool,
}
impl AsyncWrite for StalledFinalization {
    fn poll_write(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<io::Result<usize>> {
        Poll::Ready(Ok(bytes.len()))
    }
    fn poll_flush(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.flush_attempted = true;
        Poll::Pending
    }
    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}
impl LogSink for StalledFinalization {
    async fn synchronize(&mut self) -> io::Result<()> {
        self.synchronization_attempted = true;
        Err(io::Error::other("independent synchronization fixture"))
    }
}

#[tokio::test]
async fn stalled_flush_still_attempts_sync_and_does_not_degrade_other_sink() {
    let report = tokio::time::timeout(
        Duration::from_secs(2),
        start(
            io::Cursor::new(b"stdout"),
            io::Cursor::new(b"stderr"),
            (
                StalledFinalization {
                    flush_attempted: false,
                    synchronization_attempted: false,
                },
                MemorySink::default(),
            ),
            Collect::default(),
            None,
            Duration::from_secs(2),
        )
        .finish(Instant::now() + Duration::from_millis(10)),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(report.io_unresolved());
    assert!(!report.log_complete());
    assert!(report.observer_complete());
    assert!(report.stdout.sink.flush_attempted && report.stdout.sink.synchronization_attempted);
    assert_eq!(
        report.stdout.flush_error.unwrap().kind(),
        io::ErrorKind::TimedOut
    );
    assert_eq!(
        report.stdout.synchronization_error.unwrap().to_string(),
        "independent synchronization fixture"
    );
    assert!(report.stderr.log_complete());
}

#[tokio::test]
async fn actual_raw_files_are_synchronized_and_readable_without_a_codec_after_join() {
    let root = tempfile::tempdir().unwrap();
    let out_path = root.path().join("stdout.raw");
    let err_path = root.path().join("stderr.raw");
    let file = |path| {
        tokio::fs::File::from_std(
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .unwrap(),
        )
    };
    let report = start(
        io::Cursor::new(b"real stdout\0"),
        io::Cursor::new(b"real stderr\xff"),
        (file(&out_path), file(&err_path)),
        Collect::default(),
        None,
        Duration::from_secs(2),
    )
    .finish(Instant::now() + Duration::from_secs(3))
    .await
    .unwrap();
    assert!(report.log_complete());
    assert_eq!(std::fs::read(out_path).unwrap(), b"real stdout\0");
    assert_eq!(std::fs::read(err_path).unwrap(), b"real stderr\xff");
}
