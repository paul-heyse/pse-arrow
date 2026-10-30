// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

#[derive(Debug, Default)]
struct MemorySink {
    bytes: Vec<u8>,
    write_calls: usize,
    fail_after: Option<usize>,
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
        let count = match self.fail_after {
            Some(limit) if self.bytes.len() >= limit => {
                return Poll::Ready(Err(io::Error::other("disk full fixture")));
            }
            Some(limit) => bytes.len().min(limit - self.bytes.len()),
            None => bytes.len(),
        };
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
        if self.fail {
            Err(HostError::Ownership("original observer fixture".into()))
        } else {
            Ok(())
        }
    }
}
fn chunks(bytes: &[u8]) -> Vec<LogChunk> {
    bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice(line).unwrap())
        .collect()
}
fn restore(records: &[LogChunk], stream: OutputStream) -> Vec<u8> {
    let mut result = vec![];
    for record in records.iter().filter(|record| record.stream == stream) {
        assert_eq!(record.byte_offset, result.len() as u64);
        result.extend(record.decode().unwrap());
    }
    result
}

#[tokio::test]
async fn both_streams_keep_exact_bytes_offsets_and_log_order_with_slow_presentation() {
    let (mut stdout_write, stdout) = tokio::io::duplex(8192);
    let (mut stderr_write, stderr) = tokio::io::duplex(8192);
    let out = [0_u8, 0xff, b'a', b'\n'].repeat(30_000);
    let err = b"stderr\nwithout line semantics".repeat(4000);
    let (view, _slow_receiver) = mpsc::channel(1);
    let tee = start(
        stdout,
        stderr,
        MemorySink::default(),
        Collect::default(),
        Some(view),
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
    assert!(report.log_complete());
    assert!(report.observer_complete());
    assert!(!report.io_unresolved);
    assert_eq!(report.presentation_queued, 1);
    assert!(report.presentation_lost_chunks > 0);
    assert!(report.presentation_lost_bytes > 0);
    let records = chunks(&report.sink.bytes);
    for (position, record) in records.iter().enumerate() {
        assert_eq!(record.sequence, position as u64);
    }
    assert_eq!(restore(&records, OutputStream::Stdout), expected_out);
    assert_eq!(restore(&records, OutputStream::Stderr), expected_err);
    assert_eq!(report.stdout.bytes_read, expected_out.len() as u64);
    assert_eq!(report.stderr.bytes_read, expected_err.len() as u64);
    assert_eq!(report.sink.sync_calls, 1);
}

#[tokio::test]
async fn storage_and_observer_failures_do_not_stop_either_pipe_or_repeat_sink_writes() {
    let out = b"one large output line without a delimiter".repeat(4000);
    let err = b"stderr".repeat(7000);
    let tee = start(
        io::Cursor::new(out.clone()),
        io::Cursor::new(err.clone()),
        MemorySink {
            fail_after: Some(9),
            flush_fails: true,
            sync_fails: true,
            ..Default::default()
        },
        Collect {
            fail: true,
            ..Default::default()
        },
        None,
    );
    let report = tee
        .finish(Instant::now() + Duration::from_secs(5))
        .await
        .unwrap();
    assert!(report.stdout.complete() && report.stderr.complete());
    assert!(!report.log_complete());
    assert!(!report.observer_complete());
    assert!(report.log_error.is_some());
    assert!(report.flush_error.is_some());
    assert!(report.synchronization_error.is_some());
    assert_eq!(
        report.sink.write_calls, 2,
        "partial write then first failure, with no later retry"
    );
    assert_eq!(report.sink.bytes.len(), 9);
    assert_eq!(report.sink.flush_calls, 1);
    assert_eq!(report.sink.sync_calls, 1);
    assert!(report.observer_errors.iter().all(Option::is_some));
    assert!(report.observer_error_counts.iter().all(|count| *count > 0));
    let observer = report.observer.unwrap();
    for (stream, expected) in [(OutputStream::Stdout, out), (OutputStream::Stderr, err)] {
        let mut observed = Vec::<u8>::new();
        for (source, offset, bytes) in observer
            .records
            .iter()
            .filter(|(source, _, _)| *source == stream)
        {
            assert_eq!(*source, stream);
            assert_eq!(*offset, observed.len() as u64);
            observed.extend_from_slice(bytes);
        }
        assert_eq!(observed, expected);
    }
}

#[tokio::test]
async fn closed_optional_receiver_does_not_degrade_required_evidence() {
    let (sender, receiver) = mpsc::channel(1);
    drop(receiver);
    let report = start(
        io::Cursor::new(b"stdout"),
        io::Cursor::new(b"stderr"),
        MemorySink::default(),
        Collect::default(),
        Some(sender),
    )
    .finish(Instant::now() + Duration::from_secs(2))
    .await
    .unwrap();
    assert!(report.log_complete() && report.observer_complete());
    assert!(report.presentation_closed);
    assert_eq!(report.presentation_lost_chunks, 2);
    assert_eq!(report.presentation_lost_bytes, 12);
}

#[tokio::test]
async fn output_observation_expiry_is_unresolved_without_a_workload_status_claim() {
    let (_stdout_writer, stdout) = tokio::io::duplex(8);
    let (_stderr_writer, stderr) = tokio::io::duplex(8);
    let report = start(
        stdout,
        stderr,
        MemorySink::default(),
        Collect::default(),
        None,
    )
    .finish(Instant::now() + Duration::from_millis(10))
    .await
    .unwrap();
    assert!(report.io_unresolved);
    assert!(!report.log_complete());
    assert!(!report.observer_complete());
    assert!(report.stdout.observation_expired && report.stderr.observation_expired);
    assert!(!report.stdout.eof && !report.stderr.eof);
}

#[derive(Default)]
struct StalledSink {
    flush_attempted: bool,
    synchronization_attempted: bool,
}
impl AsyncWrite for StalledSink {
    fn poll_write(self: Pin<&mut Self>, _: &mut Context<'_>, _: &[u8]) -> Poll<io::Result<usize>> {
        Poll::Pending
    }
    fn poll_flush(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.flush_attempted = true;
        Poll::Pending
    }
    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}
impl LogSink for StalledSink {
    async fn synchronize(&mut self) -> io::Result<()> {
        self.synchronization_attempted = true;
        Err(io::Error::other("independent synchronization fixture"))
    }
}

#[tokio::test]
async fn stalled_required_write_retains_io_custody_and_attempts_independent_finalization() {
    let tee = start(
        io::Cursor::new(b"stdout"),
        io::Cursor::new(b"stderr"),
        StalledSink::default(),
        Collect::default(),
        None,
    );
    let report = tokio::time::timeout(
        Duration::from_secs(2),
        tee.finish(Instant::now() + Duration::from_millis(10)),
    )
    .await
    .expect("an output deadline must also bound a stalled required sink")
    .unwrap();
    assert!(report.io_unresolved);
    assert!(!report.log_complete());
    assert!(!report.observer_complete());
    assert_eq!(report.committed_records, 0);
    assert!(report.sink.flush_attempted);
    assert!(report.sink.synchronization_attempted);
    assert_eq!(report.flush_error.unwrap().kind(), io::ErrorKind::TimedOut);
    assert_eq!(
        report.synchronization_error.unwrap().to_string(),
        "independent synchronization fixture"
    );
}

#[tokio::test]
async fn actual_file_sink_is_synchronized_and_decodable_after_both_readers_join() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("output.log");
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    let report = start(
        io::Cursor::new(b"real stdout\0"),
        io::Cursor::new(b"real stderr\xff"),
        tokio::fs::File::from_std(file),
        Collect::default(),
        None,
    )
    .finish(Instant::now() + Duration::from_secs(3))
    .await
    .unwrap();
    assert!(report.log_complete());
    let records = chunks(&std::fs::read(path).unwrap());
    assert_eq!(restore(&records, OutputStream::Stdout), b"real stdout\0");
    assert_eq!(restore(&records, OutputStream::Stderr), b"real stderr\xff");
}

#[test]
fn log_codec_rejects_unsupported_versions_and_misreported_lengths() {
    let mut record = LogChunk {
        version: 1,
        sequence: 0,
        stream: OutputStream::Stdout,
        byte_offset: 0,
        byte_length: 1,
        bytes_base64: STANDARD.encode([0xff]),
    };
    assert_eq!(record.decode().unwrap(), [0xff]);
    record.version = 2;
    assert!(matches!(record.decode(), Err(LogDecodeError::Version(2))));
    record.version = 1;
    record.byte_length = 3;
    assert!(matches!(record.decode(), Err(LogDecodeError::Length)));
}
