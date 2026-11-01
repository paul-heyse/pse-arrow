// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Incremental complete-record delivery, independent of non-search envelope rendering.
use super::{Error, Event};
use std::io::Write;
use tokio_util::sync::CancellationToken;

/// One invocation writer; failed consumers cancel admission rather than retain all results.
#[derive(Debug)]
pub struct StreamWriter<W> {
    writer: W,
    token: CancellationToken,
    began: bool,
    terminal: bool,
    failed: bool,
}
impl<W: Write> StreamWriter<W> {
    /// Use the performing operation's cancellation token, not a separate lifetime owner.
    pub fn new(writer: W, token: CancellationToken) -> Self {
        Self {
            writer,
            token,
            began: false,
            terminal: false,
            failed: false,
        }
    }
    /// Serialize only one complete record; flush makes useful live rows observable.
    pub fn emit(&mut self, event: &Event) -> Result<(), Error> {
        if self.failed || self.terminal {
            return Err(Error::Invalid("stream already settled or failed".into()));
        }
        let is_begin = matches!(event, Event::Begin { .. });
        if is_begin == self.began {
            return Err(Error::Invalid(
                "stream requires exactly one initial begin record".into(),
            ));
        }
        let mut record = serde_json::to_vec(event).map_err(|error| {
            self.failed = true;
            self.token.cancel();
            Error::Json(error)
        })?;
        record.push(b'\n');
        if let Err(error) = self
            .writer
            .write_all(&record)
            .and_then(|()| self.writer.flush())
        {
            self.failed = true;
            self.token.cancel();
            return Err(Error::Delivery(error));
        }
        self.began = true;
        self.terminal = matches!(event, Event::Terminal(_));
        Ok(())
    }
    /// Recover the writer after delivery; callers still own actual CPU/native settlement.
    pub fn finish(self) -> W {
        self.writer
    }
    /// A terminal summary has actually been written and flushed.
    pub fn completed(&self) -> bool {
        self.terminal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::investigation::Summary;
    #[test]
    fn complete_records_are_visible_before_terminal_and_cannot_repeat_finality() {
        let token = CancellationToken::new();
        let mut stream = StreamWriter::new(Vec::new(), token);
        stream
            .emit(&Event::Begin {
                root: std::path::PathBuf::from("/stream-fixture").into(),
                invocation_id: "invocation".into(),
                request: Box::default(),
            })
            .unwrap();
        assert!(!stream.completed());
        assert_eq!(stream.writer.iter().filter(|b| **b == b'\n').count(), 1);
        stream.emit(&Event::Terminal(Summary::default())).unwrap();
        assert!(stream.completed());
        assert!(stream.emit(&Event::Terminal(Summary::default())).is_err());
        let bytes = stream.finish();
        let records: Vec<serde_json::Value> = bytes
            .split(|b| *b == b'\n')
            .filter(|s| !s.is_empty())
            .map(|s| serde_json::from_slice(s).unwrap())
            .collect();
        assert_eq!(records[0]["event"], "begin");
        assert_eq!(records[1]["event"], "terminal");
    }
    #[test]
    fn closed_consumer_requests_cancellation_and_cannot_claim_terminal_delivery() {
        struct Closed;
        impl Write for Closed {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::ErrorKind::BrokenPipe.into())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let token = CancellationToken::new();
        let mut stream = StreamWriter::new(Closed, token.clone());
        assert!(matches!(
            stream.emit(&Event::Begin {
                root: std::path::PathBuf::from("/stream-fixture").into(),
                invocation_id: "id".into(),
                request: Box::default()
            }),
            Err(Error::Delivery(_))
        ));
        assert!(token.is_cancelled());
        assert!(!stream.completed());
        assert!(stream.emit(&Event::Terminal(Summary::default())).is_err());
    }
    #[test]
    fn missing_begin_refuses_without_writing_a_false_complete_stream() {
        let mut stream = StreamWriter::new(Vec::new(), CancellationToken::new());
        assert!(stream.emit(&Event::Terminal(Summary::default())).is_err());
        assert!(stream.finish().is_empty());
    }
}
