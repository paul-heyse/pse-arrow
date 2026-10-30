// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded decoding frames over an independently retained raw byte stream.
use std::num::NonZeroUsize;

/// Borrowed framing fact; only complete records are eligible for protocol decoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Frame<'a> {
    /// Entire record including its newline delimiter.
    Complete(&'a [u8]),
    /// An oversized record fragment, forwarded without retaining the entire record.
    Oversized {
        /// Original bytes in exact stream order.
        bytes: &'a [u8],
        /// First fragment of this oversized record.
        first: bool,
        /// Fragment includes the record's newline delimiter.
        last: bool,
    },
    /// EOF interrupted a record. For an already oversized record, bytes are empty.
    Truncated(&'a [u8]),
}

/// Newline framing with a protocol-specific record bound; this is no workload resource cap.
#[derive(Debug)]
pub struct RecordFramer {
    limit: NonZeroUsize,
    pending: Vec<u8>,
    oversized: bool,
}
impl RecordFramer {
    /// Select the largest record the protocol consumer can decode in memory.
    pub fn new(limit: NonZeroUsize) -> Self {
        Self {
            limit,
            pending: Vec::new(),
            oversized: false,
        }
    }

    /// Emit complete records and streamed oversized fragments in original byte order.
    pub fn push(&mut self, bytes: &[u8], mut emit: impl FnMut(Frame<'_>)) {
        for part in bytes.split_inclusive(|byte| *byte == b'\n') {
            let last = part.last() == Some(&b'\n');
            if !self.oversized && self.pending.len() + part.len() <= self.limit.get() {
                self.pending.extend_from_slice(part);
            } else {
                if !self.oversized {
                    emit(Frame::Oversized {
                        bytes: &self.pending,
                        first: true,
                        last: false,
                    });
                    self.pending.clear();
                    self.oversized = true;
                }
                emit(Frame::Oversized {
                    bytes: part,
                    first: false,
                    last,
                });
            }
            if last {
                if !self.oversized {
                    emit(Frame::Complete(&self.pending));
                    self.pending.clear();
                }
                self.oversized = false;
            }
        }
    }

    /// Mark an unterminated final record explicitly; never promote it to a complete frame.
    pub fn finish(&mut self, mut emit: impl FnMut(Frame<'_>)) {
        if self.oversized || !self.pending.is_empty() {
            emit(Frame::Truncated(&self.pending));
        }
        self.pending.clear();
        self.oversized = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chunk_splits_preserve_exact_record_bytes_and_do_not_promote_truncated_tail() {
        let mut framer = RecordFramer::new(NonZeroUsize::new(8).unwrap());
        let mut facts = vec![];
        let mut collect = |frame: Frame<'_>| match frame {
            Frame::Complete(bytes) => facts.push(("complete", bytes.to_vec())),
            Frame::Truncated(bytes) => facts.push(("truncated", bytes.to_vec())),
            Frame::Oversized { bytes, .. } => facts.push(("oversized", bytes.to_vec())),
        };
        for bytes in [b"a".as_slice(), b"\nb\nc", b"d"] {
            framer.push(bytes, &mut collect);
        }
        framer.finish(&mut collect);
        assert_eq!(
            facts,
            [
                ("complete", b"a\n".to_vec()),
                ("complete", b"b\n".to_vec()),
                ("truncated", b"cd".to_vec())
            ]
        );
    }
    #[test]
    fn oversized_records_stream_every_byte_once_and_resume_at_the_next_record() {
        let mut framer = RecordFramer::new(NonZeroUsize::new(4).unwrap());
        let mut all: Vec<u8> = vec![];
        let mut complete = vec![];
        let mut starts = 0;
        let mut ends = 0;
        let mut collect = |frame: Frame<'_>| match frame {
            Frame::Complete(bytes) => {
                all.extend(bytes);
                complete.push(bytes.to_vec());
            }
            Frame::Oversized { bytes, first, last } => {
                all.extend(bytes);
                starts += usize::from(first);
                ends += usize::from(last);
            }
            Frame::Truncated(_) => panic!("fixture ends on a delimiter"),
        };
        for bytes in [b"abcd".as_slice(), b"ef", b"gh\nz\n"] {
            framer.push(bytes, &mut collect);
        }
        framer.finish(&mut collect);
        assert_eq!(all, b"abcdefgh\nz\n");
        assert_eq!(complete, [b"z\n".to_vec()]);
        assert_eq!((starts, ends), (1, 1));
    }
    #[test]
    fn oversized_eof_still_has_an_explicit_truncated_fact() {
        let mut framer = RecordFramer::new(NonZeroUsize::new(1).unwrap());
        framer.push(b"oversized", |_| {});
        let mut truncated = false;
        framer.finish(|frame| {
            assert_eq!(frame, Frame::Truncated(b""));
            truncated = true;
        });
        assert!(truncated);
    }
}
