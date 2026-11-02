// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native grep callbacks deliver witnesses without collecting the searched universe.
use super::super::*;
use super::{ProviderInput, State, record};
use crate::source_read::SourceRequest;
use grep_matcher::Matcher as _;
use grep_searcher::{
    BinaryDetection, Encoding, Searcher, SearcherBuilder, Sink, SinkContext, SinkFinish, SinkMatch,
};
use std::{
    collections::VecDeque,
    fs::File,
    io::{self, Read},
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use tokio_util::sync::CancellationToken;

pub(super) fn matcher(query: &TextQuery) -> Result<grep_regex::RegexMatcher, Error> {
    let mut builder = grep_regex::RegexMatcherBuilder::new();
    builder
        .fixed_strings(query.literal)
        .case_insensitive(query.case == Case::Insensitive)
        .case_smart(query.case == Case::Smart)
        .word(query.word)
        .whole_line(query.whole_line)
        .unicode(true)
        .multi_line(query.multiline);
    if query.multiline {
        builder.line_terminator(None);
    } else {
        builder.line_terminator(Some(b'\n'));
    }
    builder
        .build_many(&query.patterns)
        .map_err(|e| Error::Invalid(e.to_string()))
}

fn searcher(query: &TextQuery, context: usize) -> Result<Searcher, Error> {
    let encoding = query
        .encoding
        .as_deref()
        .map(Encoding::new)
        .transpose()
        .map_err(|e| Error::Invalid(e.to_string()))?;
    Ok(SearcherBuilder::new()
        .line_number(true)
        .multi_line(query.multiline)
        .invert_match(query.invert)
        .before_context(context)
        .after_context(context)
        .bom_sniffing(false)
        .encoding(encoding)
        .binary_detection(if query.binary == BinaryPolicy::Skip {
            BinaryDetection::quit(0)
        } else {
            BinaryDetection::none()
        })
        .build())
}

struct CheckedReader<R> {
    inner: R,
    token: CancellationToken,
    read: Arc<AtomicU64>,
}
impl<R: Read> Read for CheckedReader<R> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        // Interrupted is automatically retried by Read::read_to_end and native readers.
        // A typed non-retryable cause is necessary to actually leave no-match reads.
        if self.token.is_cancelled() {
            return Err(io::Error::other(Error::Cancelled));
        }
        let size = self.inner.read(bytes)?;
        self.read.fetch_add(size as u64, Ordering::Relaxed);
        Ok(size)
    }
}

pub(super) fn scan(
    root: &Path,
    request: &SourceRequest,
    query: &TextQuery,
    state: &mut State<'_>,
) -> Result<(), Error> {
    let matcher = rust::EvaluationContext::new(state.program.clone())
        .lexical_matcher(query, || matcher(query))?;
    let context_requested = state.request.presentation.fields.is_empty()
        || state
            .request
            .presentation
            .fields
            .iter()
            .any(|field| field == "context");
    let context = if context_requested {
        state.request.presentation.context
    } else {
        0
    };
    let mut searcher = searcher(query, context)?;
    let mut input = match request {
        SourceRequest::Working { path } => ProviderInput {
            path: path.strip_prefix(root).unwrap_or(path).to_path_buf(),
            source: SourceKind::Working,
            bytes: Vec::new(),
            guard: Some(super::selection::guard(&root.join(path))?),
        },
        _ => super::acquire(root, request)?,
    };
    if let Some(guard) = &mut input.guard {
        guard.root = Some(NativePath::from(root.canonicalize()?));
    }
    let path = root.join(&input.path);
    let read = Arc::new(AtomicU64::new(0));
    let token = state.token.clone();
    let result = {
        let mut sink = StreamingSink {
            state,
            input: &input,
            matcher: &matcher,
            query,
            source: None,
            file_seen: false,
            context,
            before: VecDeque::new(),
            pending: VecDeque::new(),
        };
        if matches!(request, SourceRequest::Working { .. }) {
            let reader = CheckedReader {
                inner: File::open(&path).map_err(|source| {
                    Error::SourcePath(Box::new(SourceFailure::new(
                        SourceOperation::ReadSource,
                        path.clone(),
                        source,
                        true,
                    )))
                })?,
                token: token.clone(),
                read: read.clone(),
            };
            searcher.search_reader(&*matcher, reader, &mut sink)
        } else {
            let reader = CheckedReader {
                inner: input.bytes.as_slice(),
                token: token.clone(),
                read: read.clone(),
            };
            searcher.search_reader(&*matcher, reader, &mut sink)
        }
    };
    state.summary.examined_bytes += read.load(Ordering::Relaxed);
    if let Err(error) = result
        && !token.is_cancelled()
    {
        let error = Error::SourcePath(Box::new(SourceFailure::new(
            SourceOperation::ReadSource,
            path.clone(),
            error,
            true,
        )));
        state.failure_error(None, &error)?;
    }
    let mut after = if matches!(request, SourceRequest::Working { .. }) {
        Some(super::selection::guard(&path)?)
    } else {
        None
    };
    if let Some(after) = &mut after {
        after.root = input.guard.as_ref().and_then(|guard| guard.root.clone());
    }
    if matches!(request, SourceRequest::Working { .. }) && after.as_ref() != input.guard.as_ref() {
        state.summary.unknown += 1;
        state.summary.complete = false;
        let mut failure = SourceFailure::new(
            SourceOperation::ReadSource,
            path,
            io::Error::other("source changed while scanning"),
            true,
        );
        failure.recovery.cause = crate::envelope::RecoveryCause::SourceChanged;
        state.failure_error(None, &Error::SourcePath(Box::new(failure)))?;
    }
    Ok(())
}

/// Explicit stable/exhaustive scans reuse their acquired snapshot and stream native matches.
pub(super) fn scan_snapshot(
    input: &ProviderInput,
    query: &TextQuery,
    state: &mut State<'_>,
) -> Result<(), Error> {
    let matcher = rust::EvaluationContext::new(state.program.clone())
        .lexical_matcher(query, || matcher(query))?;
    let requested = state.request.presentation.fields.is_empty()
        || state
            .request
            .presentation
            .fields
            .iter()
            .any(|field| field == "context");
    let context = if requested {
        state.request.presentation.context
    } else {
        0
    };
    let mut searcher = searcher(query, context)?;
    let read = Arc::new(AtomicU64::new(0));
    let token = state.token.clone();
    let result = {
        let mut sink = StreamingSink {
            state,
            input,
            matcher: &matcher,
            query,
            source: None,
            file_seen: false,
            context,
            before: VecDeque::new(),
            pending: VecDeque::new(),
        };
        searcher.search_reader(
            &*matcher,
            CheckedReader {
                inner: input.bytes.as_slice(),
                token: token.clone(),
                read: read.clone(),
            },
            &mut sink,
        )
    };
    state.summary.examined_bytes += read.load(Ordering::Relaxed);
    if let Err(error) = result
        && !token.is_cancelled()
    {
        state.failure(None, error.to_string())?;
    }
    Ok(())
}

struct StreamingSink<'a, 'b> {
    state: &'a mut State<'b>,
    input: &'a ProviderInput,
    matcher: &'a grep_regex::RegexMatcher,
    query: &'a TextQuery,
    source: Option<String>,
    file_seen: bool,
    context: usize,
    before: VecDeque<Vec<u8>>,
    pending: VecDeque<Pending>,
}
struct Pending {
    row: SearchResult,
    raw: Vec<u8>,
    remaining: usize,
}
impl StreamingSink<'_, '_> {
    fn source(&mut self) -> Result<String, Error> {
        if let Some(source) = &self.source {
            return Ok(source.clone());
        }
        let provider = if let Some(encoding) = &self.query.encoding {
            format!("grep-searcher/0.1.17; searched_encoding={encoding}; ranges=searched_bytes")
        } else {
            "grep-searcher/0.1.17; ranges=original_bytes".into()
        };
        let source = self.state.source_with_encoding(
            &self.input.path,
            &self.input.source,
            &provider,
            self.query.encoding.clone(),
        )?;
        self.source = Some(source.clone());
        Ok(source)
    }
    fn make_row(
        &mut self,
        bytes: &[u8],
        offset: usize,
        line: usize,
        range: std::ops::Range<usize>,
        matched: bool,
    ) -> Result<SearchResult, Error> {
        let first_line = line
            + bytes[..range.start.min(bytes.len())]
                .iter()
                .filter(|&&b| b == b'\n')
                .count();
        let last = range
            .end
            .saturating_sub(usize::from(range.end > range.start));
        let last_line = line
            + bytes[..last.min(bytes.len())]
                .iter()
                .filter(|&&b| b == b'\n')
                .count();
        let coordinates = MatchRange {
            byte_start: offset + range.start,
            byte_end: offset + range.end,
            line_start: first_line,
            line_end: last_line,
        };
        let context_requested = self.state.request.presentation.fields.is_empty()
            || self
                .state
                .request
                .presentation
                .fields
                .iter()
                .any(|field| field == "context");
        let mut result = record(
            self.input,
            Some(coordinates),
            if self.query.encoding.is_some() {
                "transcoded_text"
            } else if matched {
                "text"
            } else {
                "context"
            },
            if context_requested {
                String::from_utf8_lossy(bytes).into_owned()
            } else {
                String::new()
            },
        );
        if self.query.encoding.is_none()
            && let Some(reference) = &mut result.reference
            && let Locator::Source {
                guard,
                range: Some(location),
                ..
            } = &mut reference.locator
        {
            use sha2::{Digest as _, Sha256};
            guard.get_or_insert_with(SourceGuard::default).observed = Some(ObservedGuard {
                range: location.clone(),
                sha256: super::hex(&Sha256::digest(&bytes[range.clone()])),
            });
        }
        result.source = self.source()?;
        if context_requested && std::str::from_utf8(bytes).is_err() {
            result.bytes = Some(bytes.to_vec());
        }
        if self.query.encoding.is_some() {
            // Transcoded offsets are never asserted to be original editable byte ranges.
            if let Some(reference) = &mut result.reference
                && let Locator::Source { range, .. } = &mut reference.locator
            {
                *range = None;
            }
        }
        Ok(result)
    }
    fn row(
        &mut self,
        bytes: &[u8],
        offset: usize,
        line: usize,
        range: std::ops::Range<usize>,
        matched: bool,
    ) -> Result<bool, Error> {
        let row = self.make_row(bytes, offset, line, range, matched)?;
        self.state.result(row, matched)
    }
    fn remember(&mut self, bytes: &[u8]) {
        for line in bytes.split_inclusive(|&b| b == b'\n') {
            self.before.push_back(line.to_vec());
            while self.before.len() > self.context {
                let _ = self.before.pop_front();
            }
        }
    }
    fn advance(&mut self, bytes: &[u8]) -> Result<bool, Error> {
        for pending in &mut self.pending {
            for line in bytes
                .split_inclusive(|&b| b == b'\n')
                .take(pending.remaining)
            {
                pending.raw.extend_from_slice(line);
                pending.remaining -= 1;
            }
        }
        while self
            .pending
            .front()
            .is_some_and(|pending| pending.remaining == 0)
        {
            if let Some(mut pending) = self.pending.pop_front() {
                pending.row.context = String::from_utf8_lossy(&pending.raw).into_owned();
                pending.row.bytes = if std::str::from_utf8(&pending.raw).is_err() {
                    Some(pending.raw)
                } else {
                    None
                };
                if !self.state.result(pending.row, true)? {
                    return Ok(false);
                }
            }
        }
        Ok(!self.state.token.is_cancelled())
    }
    fn enqueue(
        &mut self,
        bytes: &[u8],
        offset: usize,
        line: usize,
        range: std::ops::Range<usize>,
    ) -> Result<bool, Error> {
        let visible = self.state.summary.emitted as usize + self.pending.len();
        if self
            .state
            .request
            .presentation
            .budget
            .is_some_and(|budget| visible >= budget)
        {
            if self.state.request.answer.limit.is_none() {
                let source = self.source()?;
                let start = line + bytes[..range.start].iter().filter(|&&b| b == b'\n').count();
                let end = line
                    + bytes[..range
                        .end
                        .saturating_sub(usize::from(range.end > range.start))]
                        .iter()
                        .filter(|&&b| b == b'\n')
                        .count();
                self.state.observe(
                    &source,
                    Some(&MatchRange {
                        byte_start: offset + range.start,
                        byte_end: offset + range.end,
                        line_start: start,
                        line_end: end,
                    }),
                    None,
                );
                return Ok(true);
            }
            return Ok(false);
        }
        if self
            .state
            .request
            .answer
            .limit
            .is_some_and(|limit| visible >= limit)
        {
            return Ok(false);
        }
        let row = self.make_row(bytes, offset, line, range, true)?;
        if self.context == 0 {
            return self.state.result(row, true);
        }
        let mut raw = Vec::new();
        for before in &self.before {
            raw.extend_from_slice(before);
        }
        raw.extend_from_slice(bytes);
        self.pending.push_back(Pending {
            row,
            raw,
            remaining: self.context,
        });
        Ok(self.state.request.answer.limit.is_none()
            || !self.state.request.answer.limit.is_some_and(|limit| {
                self.state.summary.emitted as usize + self.pending.len() >= limit
            }))
    }
}
impl Sink for StreamingSink<'_, '_> {
    type Error = io::Error;
    fn matched(&mut self, _: &Searcher, mat: &SinkMatch<'_>) -> io::Result<bool> {
        if self.state.token.is_cancelled() {
            return Ok(false);
        }
        let bytes = mat.bytes();
        let offset = mat.absolute_byte_offset() as usize;
        let line = mat.line_number().unwrap_or(1) as usize;
        if self.state.request.answer.kind == AnswerKind::Count
            || self
                .state
                .request
                .presentation
                .budget
                .is_some_and(|budget| self.state.summary.emitted >= budget as u64)
        {
            let source = self.source().map_err(io::Error::other)?;
            match self.state.request.answer.count_unit {
                CountUnit::Files if self.state.request.answer.kind == AnswerKind::Count => {
                    self.state.summary.matches += 1;
                    self.state.counted.insert(source);
                    return Ok(false);
                }
                CountUnit::Lines => {
                    self.state.summary.matches += 1;
                    let end = line
                        + bytes[..bytes.len().saturating_sub(1)]
                            .iter()
                            .filter(|&&b| b == b'\n')
                            .count();
                    for n in line..=end {
                        self.state.counted.insert(format!("{source}:{n}"));
                    }
                }
                CountUnit::Matches | CountUnit::Files if self.query.invert => {
                    self.state.observe(&source, None, None)
                }
                CountUnit::Matches | CountUnit::Files => {
                    self.matcher
                        .find_iter(bytes, |_| {
                            self.state.observe(&source, None, None);
                            !self.state.stopped()
                        })
                        .map_err(io::Error::other)?;
                }
                CountUnit::Entities => {
                    return Err(io::Error::other(Error::Unavailable(
                        "lexical matching does not establish resolved entity counts".into(),
                    )));
                }
            }
            return Ok(!self.state.stopped());
        }
        let file_answer = matches!(
            self.state.request.answer.kind,
            AnswerKind::Files | AnswerKind::Exists
        );
        if file_answer {
            if !self.file_seen {
                self.file_seen = true;
                let _ = self
                    .row(bytes, offset, line, 0..bytes.len(), true)
                    .map_err(io::Error::other)?;
            }
            return Ok(false);
        }
        if !self.advance(bytes).map_err(io::Error::other)? {
            return Ok(false);
        }
        if self.query.invert {
            let keep = self
                .enqueue(bytes, offset, line, 0..bytes.len())
                .map_err(io::Error::other)?;
            self.remember(bytes);
            return Ok(keep || self.context > 0);
        }
        let matcher = self.matcher;
        let mut result = Ok(true);
        matcher
            .find_iter(bytes, |matched| {
                match self.enqueue(bytes, offset, line, matched.start()..matched.end()) {
                    Ok(true) => true,
                    Ok(false) => {
                        result = Ok(false);
                        false
                    }
                    Err(error) => {
                        result = Err(io::Error::other(error));
                        false
                    }
                }
            })
            .map_err(io::Error::other)?;
        self.remember(bytes);
        result.map(|keep| keep || self.context > 0)
    }
    fn context(&mut self, _: &Searcher, context: &SinkContext<'_>) -> io::Result<bool> {
        if self.context == 0
            || matches!(
                self.state.request.answer.kind,
                AnswerKind::Count | AnswerKind::Files | AnswerKind::Exists
            )
        {
            return Ok(!self.state.token.is_cancelled());
        }
        let bytes = context.bytes();
        let keep = self.advance(bytes).map_err(io::Error::other)?;
        self.remember(bytes);
        Ok(keep)
    }
    fn context_break(&mut self, _: &Searcher) -> io::Result<bool> {
        self.before.clear();
        Ok(!self.state.token.is_cancelled())
    }
    fn finish(&mut self, _: &Searcher, _: &SinkFinish) -> io::Result<()> {
        while !self.state.token.is_cancelled() {
            let Some(mut pending) = self.pending.pop_front() else {
                break;
            };
            pending.row.context = String::from_utf8_lossy(&pending.raw).into_owned();
            pending.row.bytes = if std::str::from_utf8(&pending.raw).is_err() {
                Some(pending.raw)
            } else {
                None
            };
            if !self
                .state
                .result(pending.row, true)
                .map_err(io::Error::other)?
            {
                break;
            }
        }
        Ok(())
    }
    fn binary_data(&mut self, _: &Searcher, offset: u64) -> io::Result<bool> {
        self.state.summary.complete = false;
        self.state.summary.unknown += 1;
        (self.state.emit)(Event::Diagnostic{phase:DiagnosticPhase::Provider,code:"search.binary_excluded".into(),message:format!("binary content detected at original byte {offset}; file examination stopped under binary=skip"),source:self.source.clone(),detail:None}).map_err(io::Error::other)?;
        Ok(false)
    }
}

/// File-local lexical witnesses for explicitly composed questions.
pub(super) fn evaluate_prepared(
    input: &ProviderInput,
    query: &TextQuery,
    token: &CancellationToken,
    limit: Option<usize>,
    matcher: &grep_regex::RegexMatcher,
) -> Result<Vec<SearchResult>, Error> {
    if query.binary == BinaryPolicy::Skip && input.bytes.contains(&0) {
        return Err(Error::Unavailable(
            "binary input excluded by the requested text policy".into(),
        ));
    }
    let request = Request {
        predicate: Predicate::Text(query.clone()),
        answer: AnswerDemand {
            limit,
            ..Default::default()
        },
        presentation: QueryPresentation {
            context: 0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut results = Vec::new();
    let mut emit = |event| {
        if let Event::Result(result) = event {
            results.push(*result);
        }
        Ok(())
    };
    let mut state = State {
        request: &request,
        token,
        emit: &mut emit,
        summary: Summary::default(),
        sources: Default::default(),
        counted: Default::default(),
        program: Arc::new(rust::CompiledProgram::default()),
    };
    let mut searcher = searcher(query, 0)?;
    let mut sink = StreamingSink {
        state: &mut state,
        input,
        matcher,
        query,
        source: None,
        file_seen: false,
        context: 0,
        before: VecDeque::new(),
        pending: VecDeque::new(),
    };
    let reader = CheckedReader {
        inner: input.bytes.as_slice(),
        token: token.clone(),
        read: Arc::new(AtomicU64::new(0)),
    };
    searcher.search_reader(matcher, reader, &mut sink)?;
    if token.is_cancelled() {
        return Err(Error::Cancelled);
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn evaluate(
        input: &ProviderInput,
        query: &TextQuery,
        token: &CancellationToken,
        limit: Option<usize>,
    ) -> Result<Vec<SearchResult>, Error> {
        evaluate_prepared(input, query, token, limit, &matcher(query)?)
    }
    fn input(text: &str) -> ProviderInput {
        ProviderInput {
            path: "a.rs".into(),
            source: SourceKind::Working,
            bytes: text.as_bytes().to_vec(),
            guard: None,
        }
    }
    #[test]
    fn native_occurrences_and_line_count_remain_distinct() {
        let q = TextQuery {
            patterns: vec!["needle".into()],
            literal: true,
            ..Default::default()
        };
        let found = evaluate(
            &input("needle needle\nother\nneedle\n"),
            &q,
            &CancellationToken::new(),
            None,
        )
        .unwrap();
        assert_eq!(found.len(), 3);
        assert_eq!(found[1].range.as_ref().unwrap().byte_start, 7);
    }
    #[test]
    fn multiline_inverse_zero_width_and_early_demand() {
        let token = CancellationToken::new();
        let q = TextQuery {
            patterns: vec!["a\\nb".into()],
            multiline: true,
            ..Default::default()
        };
        assert_eq!(
            evaluate(&input("a\nb\n"), &q, &token, None).unwrap().len(),
            1
        );
        let q = TextQuery {
            patterns: vec!["^".into()],
            ..Default::default()
        };
        assert_eq!(
            evaluate(&input("first\nsecond\n"), &q, &token, Some(1))
                .unwrap()
                .len(),
            1
        );
        let q = TextQuery {
            patterns: vec!["yes".into()],
            invert: true,
            ..Default::default()
        };
        let found = evaluate(&input("yes\nno\n"), &q, &token, None).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].context, "no\n");
    }
    #[test]
    fn cancelled_no_match_reader_does_not_claim_eof() {
        let token = CancellationToken::new();
        token.cancel();
        let mut reader = CheckedReader {
            inner: &b"no match"[..],
            token,
            read: Arc::new(AtomicU64::new(0)),
        };
        assert_eq!(
            reader.read(&mut [0; 16]).unwrap_err().kind(),
            io::ErrorKind::Other
        );
    }
    #[test]
    fn native_case_word_crlf_and_binary_bytes_preserve_their_meaning() {
        let token = CancellationToken::new();
        let query = TextQuery {
            patterns: vec!["cat".into()],
            word: true,
            case: Case::Smart,
            ..Default::default()
        };
        let results = evaluate(&input("CAT\r\ncatlike\ncat\n"), &query, &token, None).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].range.as_ref().unwrap().byte_start, 0);
        assert_eq!(results[0].range.as_ref().unwrap().byte_end, 3);
        let query = TextQuery {
            patterns: vec!["Cat".into()],
            case: Case::Smart,
            ..Default::default()
        };
        assert!(
            evaluate(&input("CAT cat\n"), &query, &token, None)
                .unwrap()
                .is_empty()
        );
        let input = ProviderInput {
            path: "a.rs".into(),
            source: SourceKind::Working,
            bytes: b"a\xfftarget\0\n".to_vec(),
            guard: None,
        };
        let query = TextQuery {
            patterns: vec!["target".into()],
            literal: true,
            binary: BinaryPolicy::Text,
            ..Default::default()
        };
        let results = evaluate(&input, &query, &token, None).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].bytes.is_some());
        assert_eq!(results[0].range.as_ref().unwrap().byte_start, 2);
        let query = TextQuery {
            binary: BinaryPolicy::Skip,
            ..query
        };
        assert!(matches!(
            evaluate(&input, &query, &token, None),
            Err(Error::Unavailable(_))
        ));
    }
}
