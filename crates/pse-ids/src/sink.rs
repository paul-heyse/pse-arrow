// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bounded serialization without interpreting a library's error messages.
//! The caller prepays retained output and supplies its own cancellation/deadline check.

use std::{error::Error, io::{self, Write}};

/// A serialization refusal, retained by `io::Error::other` through library wrappers.
#[derive(Debug, thiserror::Error)]
pub enum SinkFailure {
    /// The next complete write would exceed the selected output allowance.
    #[error("encoded output exceeds byte limit {limit}")]
    LimitReached { /// Selected byte allowance.
        limit: usize },
    /// The logical output count cannot be represented.
    #[error("encoded output byte count overflow")]
    CountOverflow,
}

/// Find a typed refusal through serde, Arrow and ordinary source chains.
pub fn sink_failure<'a>(error: &'a (dyn Error + 'static)) -> Option<&'a SinkFailure> {
    error_payload(error)
}

/// Recover a concrete cause, including `io::Error`'s otherwise hidden payload.
pub fn error_payload<'a, T: Error + 'static>(mut error: &'a (dyn Error + 'static)) -> Option<&'a T> {
    loop {
        if let Some(failure) = error.downcast_ref::<T>() { return Some(failure); }
        if let Some(inner) = error.downcast_ref::<io::Error>().and_then(io::Error::get_ref) {
            error = inner;
        } else {
            error = error.source()?;
        }
    }
}

// serde_json exposes `io_error.source()`, which skips the io payload itself.
// This carrier keeps the concrete cause reachable through that extra hop.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct Cause<E: Error + 'static>(#[source] E);
fn failure(cause: impl Error + Send + Sync + 'static) -> io::Error {
    io::Error::other(Cause(cause))
}
fn forward(error: io::Error) -> io::Error {
    io::Error::new(error.kind(), Cause(error))
}

/// Count, retain (`Vec<u8>`) or forward bytes with one limit/checkpoint capability.
/// Partial writes count only bytes accepted by the inner writer. Its errors and
/// checkpoint errors retain their concrete payloads; neither becomes exhaustion.
pub struct BoundedSink<'a, W> {
    inner: W,
    count: usize,
    limit: usize,
    checkpoint: Option<Box<dyn FnMut() -> io::Result<()> + Send + 'a>>,
}
impl<W: std::fmt::Debug> std::fmt::Debug for BoundedSink<'_, W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundedSink").field("inner", &self.inner)
            .field("count", &self.count).field("limit", &self.limit)
            .field("checkpoint", &self.checkpoint.is_some()).finish()
    }
}
impl<W> BoundedSink<'_, W> {
    /// Wrap an already prepaid destination.
    pub fn new(inner: W, limit: usize) -> Self {
        Self { inner, count: 0, limit, checkpoint: None }
    }
    /// Bytes actually accepted by the destination.
    pub fn count(&self) -> usize { self.count }
    /// Inspect retained output, including a refused serialization's accepted prefix.
    pub fn inner(&self) -> &W { &self.inner }
    /// Return the destination without changing its ownership.
    pub fn into_inner(self) -> W { self.inner }
}
impl<'a, W> BoundedSink<'a, W> {
    /// Check cancellation/expiry before every write and flush.
    pub fn with_checkpoint(mut self, checkpoint: impl FnMut() -> io::Result<()> + Send + 'a) -> Self {
        self.checkpoint = Some(Box::new(checkpoint));
        self
    }
    fn check(&mut self) -> io::Result<()> {
        if let Some(checkpoint) = &mut self.checkpoint { checkpoint().map_err(forward)?; }
        Ok(())
    }
}
impl BoundedSink<'_, io::Sink> {
    /// Count serialization without retaining bytes.
    pub fn counting(limit: usize) -> Self { Self::new(io::sink(), limit) }
}
impl<W: Write> Write for BoundedSink<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.check()?;
        let next = self.count.checked_add(bytes.len())
            .ok_or_else(|| failure(SinkFailure::CountOverflow))?;
        if next > self.limit {
            return Err(failure(SinkFailure::LimitReached { limit: self.limit }));
        }
        let written = self.inner.write(bytes).map_err(forward)?;
        if written == 0 && !bytes.is_empty() {
            return Err(forward(io::Error::from(io::ErrorKind::WriteZero)));
        }
        self.count += written;
        Ok(written)
    }
    fn flush(&mut self) -> io::Result<()> { self.check()?; self.inner.flush().map_err(forward) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_bound_and_one_byte_overflow_are_typed_after_serde() {
        let mut sink = BoundedSink::new(Vec::new(), 3);
        serde_json::to_writer(&mut sink, &7).unwrap();
        sink.write_all(b"89").unwrap();
        assert_eq!(sink.count(), 3);
        let error = serde_json::to_writer(&mut sink, &0).unwrap_err();
        assert!(matches!(sink_failure(&error), Some(SinkFailure::LimitReached { limit: 3 })));
        assert_eq!(sink.into_inner(), b"789");
    }
    #[test]
    fn checked_count_overflow_is_distinct() {
        let mut sink = BoundedSink::counting(usize::MAX);
        sink.count = usize::MAX;
        let error = sink.write(b"x").unwrap_err();
        assert!(matches!(sink_failure(&error), Some(SinkFailure::CountOverflow)));
    }
    #[test]
    fn cancellation_and_forwarded_io_do_not_become_limits() {
        let mut cancelled = BoundedSink::counting(0).with_checkpoint(|| {
            Err(io::Error::other(pse_diagnostics::WorkError::Cancelled))
        });
        let error = serde_json::to_writer(&mut cancelled, &1).unwrap_err();
        assert!(sink_failure(&error).is_none());
        assert!(matches!(error_payload::<pse_diagnostics::WorkError>(&error), Some(pse_diagnostics::WorkError::Cancelled)));
        let mut closed = BoundedSink::new(io::Cursor::new(&mut [0u8; 0][..]), 4);
        let error = closed.write_all(b"x").unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::WriteZero);
        assert!(sink_failure(&error).is_none());
    }
    #[test]
    fn forwarded_io_payload_survives_serde() {
        #[derive(Debug, thiserror::Error)]
        #[error("forwarded test failure")]
        struct Forwarded;
        struct Broken;
        impl Write for Broken {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> { Err(io::Error::other(Forwarded)) }
            fn flush(&mut self) -> io::Result<()> { Ok(()) }
        }
        let error = serde_json::to_writer(BoundedSink::new(Broken, 4), &1).unwrap_err();
        assert!(error_payload::<Forwarded>(&error).is_some());
        assert!(sink_failure(&error).is_none());
    }
}
