// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Repository selection and native lexical/structured/historical providers.

mod lexical;
mod module_graph;
mod parallel;
mod selection;
mod structured;
#[cfg(test)]
mod tests;

pub use module_graph::{ModuleGraph, WrittenModuleContext};
pub(crate) use selection::Selection;
pub(crate) use selection::select_prepared_context;
pub use selection::{ProviderInput, acquire, select};
pub use structured::{history, history_arguments, inspect, structured};

use super::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::Arc,
};
use tokio_util::sync::CancellationToken;

/// Execute selected providers. The caller owns begin/terminal framing and physical settlement.
pub fn run(
    root: &Path,
    request: &Request,
    token: &CancellationToken,
    emit: &mut (dyn FnMut(Event) -> Result<(), Error> + Send),
) -> Result<Summary, Error> {
    if request.sources.include_dependencies && request.sources.kind != SourceKind::Working {
        return Err(Error::Invalid("external dependency sources require the working universe; historical metadata is not inferred from current resolution".into()));
    }
    if matches!(request.predicate, Predicate::Files)
        && matches!(
            request.answer.count_unit,
            CountUnit::Lines | CountUnit::Entities
        )
    {
        return Err(Error::Invalid("filename enumeration counts names or files; line/entity counts require a provider supplying those witnesses".into()));
    }
    if let Predicate::History(query) = &request.predicate {
        let _ = query;
        return Err(Error::Invalid(
            "historical operations use the asynchronous owned Git route".into(),
        ));
    }
    if request.sources.kind == SourceKind::Working
        && request.answer.order == ResultOrder::Arrival
        && request.answer.cursor.is_none()
    {
        return parallel::run(root, request, token, emit);
    }
    let mut state = State {
        request,
        token,
        emit,
        summary: Summary::default(),
        sources: BTreeSet::new(),
        counted: BTreeSet::new(),
        program: Arc::new(rust::CompiledProgram::default()),
    };
    prepare_text(
        &request.predicate,
        &rust::EvaluationContext::new(state.program.clone()),
    )?;
    if request.answer.order == ResultOrder::Stable || request.answer.cursor.is_some() {
        return stable(root, request, &mut state);
    }
    selection::visit(root, &request.sources, token, &mut |source| {
        if state.stopped() {
            return Ok(false);
        }
        match source {
            Ok(source) => process(root, &source, &mut state)?,
            Err(error) => state.failure_error(None, &error)?,
        }
        (state.emit)(Event::Progress {
            examined_files: state.summary.examined_files,
            examined_bytes: state.summary.examined_bytes,
            matches: state.summary.matches,
        })?;
        Ok(!state.stopped())
    })?;
    Ok(state.finish())
}

struct State<'a> {
    request: &'a Request,
    token: &'a CancellationToken,
    emit: &'a mut (dyn FnMut(Event) -> Result<(), Error> + Send),
    summary: Summary,
    sources: BTreeSet<String>,
    counted: BTreeSet<String>,
    program: Arc<rust::CompiledProgram>,
}
impl State<'_> {
    fn stopped(&mut self) -> bool {
        if self.token.is_cancelled() {
            self.summary.complete = false;
            self.summary.cancellation_requested = true;
            self.summary.stop = StopReason::Cancelled;
            return true;
        }
        let limit = work_limit(self.request);
        if limit.is_some_and(|limit| self.summary.matches >= limit as u64) {
            self.summary.complete = false;
            self.summary.stop = StopReason::Limit;
            return true;
        }
        false
    }
    fn source(&mut self, path: &Path, kind: &SourceKind, provider: &str) -> Result<String, Error> {
        self.source_with_encoding(path, kind, provider, None)
    }
    fn source_with_encoding(
        &mut self,
        path: &Path,
        kind: &SourceKind,
        provider: &str,
        searched_encoding: Option<String>,
    ) -> Result<String, Error> {
        use sha2::{Digest as _, Sha256};
        use std::os::unix::ffi::OsStrExt as _;
        let mut digest = Sha256::new();
        digest.update(path.as_os_str().as_bytes());
        digest.update(serde_json::to_vec(kind)?);
        digest.update(provider.as_bytes());
        let id = hex(&digest.finalize());
        if self.request.answer.kind == AnswerKind::Count
            || self
                .request
                .presentation
                .budget
                .is_some_and(|budget| self.summary.emitted >= budget as u64)
        {
            return Ok(id);
        }
        if self.sources.insert(id.clone()) {
            (self.emit)(Event::Source(Box::new(SourceContext {
                id: id.clone(),
                path: NativePath::from(path.to_path_buf()),
                origin: Some(kind.clone()),
                provider: provider.into(),
                generation: None,
                searched_encoding,
                native: None,
            })))?;
        }
        Ok(id)
    }
    fn result(&mut self, result: SearchResult, matched: bool) -> Result<bool, Error> {
        if matched {
            self.observe(
                &result.source,
                result.range.as_ref(),
                result.name.as_deref(),
            );
        }
        if self.request.answer.kind == AnswerKind::Count
            || self
                .request
                .presentation
                .budget
                .is_some_and(|budget| self.summary.emitted >= budget as u64)
        {
            return Ok(!self.stopped());
        }
        if matched {
            self.summary.emitted += 1;
        }
        if let Err(error) = (self.emit)(Event::Result(Box::new(result))) {
            self.summary.complete = false;
            self.summary.stop = StopReason::ConsumerClosed;
            self.token.cancel();
            return Err(error);
        }
        Ok(!self.stopped())
    }
    fn observe(&mut self, source: &str, range: Option<&MatchRange>, name: Option<&str>) {
        self.summary.matches += 1;
        match self.request.answer.count_unit {
            CountUnit::Matches => {}
            CountUnit::Files => {
                if !self.counted.contains(source) {
                    self.counted.insert(source.into());
                }
            }
            CountUnit::Lines => {
                if let Some(range) = range {
                    for line in range.line_start..=range.line_end {
                        self.counted.insert(format!("{source}:{line}"));
                    }
                }
            }
            CountUnit::Entities => {
                self.counted
                    .insert(format!("{source}:{}:{range:?}", name.unwrap_or("")));
            }
        }
    }
    fn failure(&mut self, source: Option<String>, message: String) -> Result<(), Error> {
        self.failure_detail(source, message, None)
    }
    fn failure_error(&mut self, source: Option<String>, error: &Error) -> Result<(), Error> {
        self.summary.failures += 1;
        self.summary.complete = false;
        (self.emit)(Event::Diagnostic {
            phase: error.phase(),
            code: "search.provider".into(),
            message: error.to_string(),
            source,
            detail: error.query_error_record(),
        })
    }
    fn failure_detail(
        &mut self,
        source: Option<String>,
        message: String,
        detail: Option<Box<crate::envelope::ErrorRecord>>,
    ) -> Result<(), Error> {
        self.summary.failures += 1;
        self.summary.complete = false;
        (self.emit)(Event::Diagnostic {
            phase: DiagnosticPhase::Provider,
            code: "search.provider".into(),
            message,
            source,
            detail,
        })
    }
    fn finish(&mut self) -> Summary {
        let _ = self.stopped();
        let count = if self.request.answer.count_unit == CountUnit::Matches {
            self.summary.matches
        } else {
            self.counted.len() as u64
        };
        self.summary.count = if self.summary.complete {
            Count::Exact(count)
        } else if self.summary.failures == 0 && self.summary.unknown == 0 {
            Count::AtLeast(count)
        } else {
            Count::Unknown
        };
        std::mem::take(&mut self.summary)
    }
}

fn process(
    root: &Path,
    source: &crate::source_read::SourceRequest,
    state: &mut State<'_>,
) -> Result<(), Error> {
    state.summary.examined_files += 1;
    if matches!(state.request.predicate, Predicate::Files) {
        let input = match selection::observe(root, source) {
            Ok(input) => input,
            Err(error) => return state.failure_error(None, &error),
        };
        let id = state.source(&input.path, &input.source, "source_name")?;
        let mut row = record(&input, None, "file", input.path.display().to_string());
        row.source = id;
        let _ = state.result(row, true)?;
        return Ok(());
    }
    if let Predicate::Text(query) = &state.request.predicate {
        return match lexical::scan(root, source, query, state) {
            Err(error @ (Error::Io(_) | Error::SourcePath(_))) if !state.token.is_cancelled() => {
                state.failure_error(None, &error)
            }
            other => other,
        };
    }
    let input = match selection::acquire_cancellable(root, source, state.token) {
        Ok(input) => input,
        Err(Error::Cancelled) if state.token.is_cancelled() => return Ok(()),
        Err(error) => return state.failure_error(None, &error),
    };
    state.summary.examined_bytes += input.bytes.len() as u64;
    let predicate = &state.request.predicate;
    let limit = match state.request.answer.kind {
        AnswerKind::Files | AnswerKind::Exists => Some(1),
        AnswerKind::Count => None,
        _ => state
            .request
            .answer
            .limit
            .map(|n| n.saturating_sub(state.summary.emitted as usize)),
    };
    let mut context = rust::EvaluationContext::new(state.program.clone());
    let results = match evaluate(root, &input, predicate, state.token, limit, &mut context) {
        Ok(results) => results,
        Err(Error::Cancelled) if state.token.is_cancelled() => return Ok(()),
        Err(Error::Unavailable(message)) => {
            state.summary.unknown += 1;
            state.summary.complete = false;
            return (state.emit)(Event::Diagnostic {
                phase: DiagnosticPhase::Context,
                code: "search.unavailable".into(),
                message,
                source: None,
                detail: None,
            });
        }
        Err(error) => {
            return state.failure_detail(None, error.to_string(), error.query_error_record());
        }
    };
    if !results.complete && !results.limited {
        state.summary.complete = false;
        state.summary.unknown += 1;
    }
    for diagnostic in results.diagnostics {
        (state.emit)(Event::Diagnostic {
            phase: diagnostic.phase,
            code: diagnostic.code,
            message: diagnostic.message,
            source: diagnostic.source,
            detail: diagnostic.detail,
        })?;
    }
    if results.rows.is_empty() {
        return Ok(());
    }
    let id = state.source(&input.path, &input.source, "source")?;
    for mut result in results.rows {
        result.source = id.clone();
        if !state.result(result, true)? {
            break;
        }
    }
    Ok(())
}

/// Evaluate file-local clauses without converting syntax matches into spelling evidence.
pub fn evaluate(
    root: &Path,
    input: &ProviderInput,
    predicate: &Predicate,
    token: &CancellationToken,
    limit: Option<usize>,
    context: &mut rust::EvaluationContext,
) -> Result<rust::Evaluation, Error> {
    if token.is_cancelled() {
        return Err(Error::Cancelled);
    }
    match predicate {
        Predicate::Files => Ok(evaluated(vec![record(
            input,
            None,
            "file",
            input.path.display().to_string(),
        )])),
        Predicate::Text(query) => {
            let matcher = context.lexical_matcher(query, || lexical::matcher(query))?;
            lexical::evaluate_prepared(input, query, token, limit, &matcher).map(evaluated)
        }
        Predicate::Structured(query) => structured(input, query).map(evaluated),
        Predicate::All {
            within: MatchScope::Node,
            ..
        }
        | Predicate::Any {
            within: MatchScope::Node,
            ..
        }
        | Predicate::Not {
            within: MatchScope::Node,
            ..
        } => {
            let rule=native_rule(predicate).ok_or_else(||Error::Unavailable("node-scoped combinations require native AST rule clauses; source spellings cannot supply resolved node identity".into()))?;
            let predicate = Predicate::Syntax(SyntaxQuery {
                rule: Some(rule),
                ..Default::default()
            });
            composition::evaluate_source(root, input, &predicate, token, limit, context)
        }
        Predicate::All {
            within: MatchScope::Line,
            clauses,
        } => line_conjunction(root, input, clauses, token, limit, context),
        Predicate::Not {
            within: MatchScope::Line,
            clause,
        } => {
            let inner = evaluate(root, input, clause, token, None, context)?;
            if !inner.complete {
                return Ok(rust::Evaluation {
                    rows: Vec::new(),
                    complete: false,
                    limited: false,
                    diagnostics: inner.diagnostics,
                });
            }
            let mut excluded = BTreeSet::new();
            for row in inner.rows {
                let range = row.range.ok_or_else(|| {
                    Error::Unavailable("line negation requires original line coordinates".into())
                })?;
                excluded.extend(range.line_start..=range.line_end);
            }
            let mut rows = Vec::new();
            let mut offset = 0;
            for (line, bytes) in input.bytes.split_inclusive(|&b| b == b'\n').enumerate() {
                if token.is_cancelled() {
                    return Err(Error::Cancelled);
                }
                if !excluded.contains(&(line + 1)) {
                    rows.push(record(
                        input,
                        Some(MatchRange {
                            byte_start: offset,
                            byte_end: offset + bytes.len(),
                            line_start: line + 1,
                            line_end: line + 1,
                        }),
                        "line_negation",
                        String::from_utf8_lossy(bytes).into_owned(),
                    ));
                    if limit.is_some_and(|n| rows.len() >= n) {
                        break;
                    }
                }
                offset += bytes.len();
            }
            Ok(evaluated(rows))
        }
        Predicate::All {
            within: MatchScope::File,
            clauses,
        } => {
            let mut witnesses = Vec::new();
            let mut unknown = None;
            let mut complete = true;
            let mut diagnostics = Vec::new();
            for clause in clauses {
                let results = match evaluate(root, input, clause, token, Some(1), context) {
                    Ok(rows) => rows,
                    Err(error @ Error::Unavailable(_)) => {
                        unknown = Some(error);
                        continue;
                    }
                    Err(error) => return Err(error),
                };
                if results.rows.is_empty() && results.complete {
                    return Ok(evaluated(Vec::new()));
                }
                if !results.complete && !results.limited {
                    complete = false;
                }
                if results.rows.is_empty() {
                    unknown = Some(Error::Unavailable(
                        "conjunction clause has no complete negative or positive witness".into(),
                    ));
                }
                diagnostics.extend(results.diagnostics);
                witnesses.extend(results.rows);
            }
            if let Some(error) = unknown {
                return Err(error);
            }
            let mut result = record(
                input,
                None,
                "file_conjunction",
                input.path.display().to_string(),
            );
            for (index, witness) in witnesses.into_iter().enumerate() {
                if let Some(range) = witness.range {
                    result
                        .captures
                        .insert(format!("clause_{index}"), vec![range]);
                }
            }
            Ok(rust::Evaluation {
                rows: vec![result],
                complete,
                limited: false,
                diagnostics,
            })
        }
        Predicate::Any { clauses, .. } => {
            let mut results = Vec::new();
            let mut unknown = None;
            let mut complete = true;
            let mut limited = false;
            let mut diagnostics = Vec::new();
            for clause in clauses {
                match evaluate(root, input, clause, token, limit, context) {
                    Ok(found) => {
                        complete &= found.complete || found.limited;
                        limited |= found.limited;
                        diagnostics.extend(found.diagnostics);
                        results.extend(found.rows);
                    }
                    Err(error @ Error::Unavailable(_)) => {
                        diagnostics.push(QueryDiagnostic {
                            phase: error.phase(),
                            code: "search.unavailable".into(),
                            message: error.to_string(),
                            source: None,
                            detail: error.query_error_record(),
                        });
                        unknown = Some(error);
                        complete = false;
                    }
                    Err(error) => return Err(error),
                }
                if limit.is_some_and(|n| results.len() >= n) {
                    limited = true;
                    break;
                }
            }
            if results.is_empty()
                && let Some(error) = unknown
            {
                return Err(error);
            }
            if let Some(n) = limit {
                results.truncate(n);
            }
            Ok(rust::Evaluation {
                rows: results,
                complete: complete && !limited,
                limited: limited && complete,
                diagnostics,
            })
        }
        Predicate::Not {
            within: MatchScope::File,
            clause,
        } => {
            let evaluated = evaluate(root, input, clause, token, Some(1), context)?;
            if evaluated.rows.is_empty() {
                if !evaluated.complete {
                    return Err(Error::Unavailable(
                        "incomplete examination cannot prove file-level negation".into(),
                    ));
                }
                Ok(self::evaluated(vec![record(
                    input,
                    None,
                    "file_negation",
                    input.path.display().to_string(),
                )]))
            } else {
                Ok(self::evaluated(Vec::new()))
            }
        }
        _ => composition::evaluate_source(root, input, predicate, token, limit, context),
    }
}

pub(super) fn record(
    input: &ProviderInput,
    range: Option<MatchRange>,
    kind: &str,
    context: String,
) -> SearchResult {
    SearchResult {
        source_relationship: None,
        source: String::new(),
        kind: kind.into(),
        name: None,
        entity: None,
        candidate_reference: None,
        refinement: None,
        range: range.clone(),
        context,
        reference: Some(ResultReference {
            version: 1,
            locator: Locator::Source {
                path: NativePath::from(input.path.clone()),
                source: input.source.clone(),
                range,
                guard: input.guard.clone(),
            },
        }),
        captures: BTreeMap::new(),
        bytes: None,
    }
}

fn stable(root: &Path, request: &Request, state: &mut State<'_>) -> Result<Summary, Error> {
    use sha2::{Digest as _, Sha256};
    let mut selected = select(root, &request.sources, state.token)?;
    selected.sort_by(|a, b| a.path().cmp(b.path()));
    let mut generation = Sha256::new();
    let mut hashes = BTreeMap::new();
    let names = matches!(request.predicate, Predicate::Files);
    for source in &selected {
        if state.token.is_cancelled() {
            return Ok(state.finish());
        }
        let input = match if names {
            selection::observe(root, source)
        } else {
            selection::acquire_cancellable(root, source, state.token)
        } {
            Ok(input) => input,
            Err(Error::Cancelled) if state.token.is_cancelled() => return Ok(state.finish()),
            Err(error) => return Err(error),
        };
        let identity = serde_json::to_vec(&(
            NativePath::from(input.path.clone()),
            &input.source,
            &input.guard,
        ))?;
        generation.update(&identity);
        hashes.insert(input.path, identity);
    }
    let generation = hex(&generation.finalize());
    let mut unpaged = request.clone();
    unpaged.answer.cursor = None;
    let query = hex(&Sha256::digest(serde_json::to_vec(&(
        NativePath::from(root.to_path_buf()),
        &unpaged,
    ))?));
    let mut offset = 0usize;
    if let Some(cursor) = &request.answer.cursor {
        let values: serde_json::Value = serde_json::from_str(cursor)?;
        if values["version"] != 1 {
            return Err(Error::Invalid("unsupported continuation version".into()));
        }
        if values["generation"] != generation || values["query"] != query {
            return Err(Error::Stale(
                "continuation source universe or query changed".into(),
            ));
        }
        offset = values["offset"]
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .ok_or_else(|| Error::Invalid("invalid continuation offset".into()))?;
    }
    let mut seen = 0usize;
    let mut has_more = false;
    for source in selected {
        if state.token.is_cancelled() {
            break;
        }
        let input = match if names {
            selection::observe(root, &source)
        } else {
            selection::acquire_cancellable(root, &source, state.token)
        } {
            Ok(input) => input,
            Err(Error::Cancelled) if state.token.is_cancelled() => break,
            Err(error) => return Err(error),
        };
        let identity = serde_json::to_vec(&(
            NativePath::from(input.path.clone()),
            &input.source,
            &input.guard,
        ))?;
        if hashes.get(&input.path) != Some(&identity) {
            return Err(Error::Stale(
                "selected source changed during stable page acquisition".into(),
            ));
        }
        state.summary.examined_files += 1;
        if let Predicate::Text(query) = &request.predicate
            && (request.answer.kind == AnswerKind::Count || request.answer.limit.is_none())
        {
            lexical::scan_snapshot(&input, query, state)?;
            (state.emit)(Event::Progress {
                examined_files: state.summary.examined_files,
                examined_bytes: state.summary.examined_bytes,
                matches: state.summary.matches,
            })?;
            continue;
        }
        state.summary.examined_bytes += input.bytes.len() as u64;
        let limit = if request.answer.kind == AnswerKind::Count {
            None
        } else {
            request.answer.limit.map(|limit| {
                offset
                    .saturating_add(limit)
                    .saturating_add(1)
                    .saturating_sub(seen)
            })
        };
        let mut context = rust::EvaluationContext::new(state.program.clone());
        let results = evaluate(
            root,
            &input,
            &request.predicate,
            state.token,
            limit,
            &mut context,
        )?;
        if !results.complete && !results.limited {
            state.summary.complete = false;
            state.summary.unknown += 1;
        }
        for diagnostic in results.diagnostics {
            (state.emit)(Event::Diagnostic {
                phase: diagnostic.phase,
                code: diagnostic.code,
                message: diagnostic.message,
                source: diagnostic.source,
                detail: diagnostic.detail,
            })?;
        }
        if results.rows.is_empty() {
            continue;
        }
        let id = state.source(&input.path, &input.source, "stable_source")?;
        for mut result in results.rows {
            if seen < offset {
                seen += 1;
                continue;
            }
            if state.stopped() {
                has_more = true;
                break;
            }
            result.source = id.clone();
            let _ = state.result(result, true)?;
            seen += 1;
        }
        if has_more {
            break;
        }
    }
    if has_more {
        state.summary.next_cursor = Some(
            serde_json::json!({"version":1,"generation":generation,"query":query,"offset":seen})
                .to_string(),
        );
    }
    Ok(state.finish())
}

pub(super) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn evaluated(rows: Vec<SearchResult>) -> rust::Evaluation {
    rust::Evaluation {
        rows,
        complete: true,
        limited: false,
        diagnostics: Vec::new(),
    }
}

fn prepare_text(predicate: &Predicate, context: &rust::EvaluationContext) -> Result<(), Error> {
    match predicate {
        Predicate::Text(query) => {
            context.lexical_matcher(query, || lexical::matcher(query))?;
        }
        Predicate::All { clauses, .. } | Predicate::Any { clauses, .. } => {
            for clause in clauses {
                prepare_text(clause, context)?;
            }
        }
        Predicate::Not { clause, .. } => prepare_text(clause, context)?,
        _ => {}
    }
    Ok(())
}

fn work_limit(request: &Request) -> Option<usize> {
    let limit = match request.answer.kind {
        AnswerKind::Exists => Some(1),
        AnswerKind::Count => None,
        _ => request.answer.limit,
    };
    match (limit, request.presentation.budget) {
        (Some(limit), Some(budget)) if budget > 0 => Some(limit.min(budget)),
        _ => limit,
    }
}

fn native_rule(predicate: &Predicate) -> Option<serde_json::Value> {
    match predicate {
        Predicate::Syntax(query) => query.rule.clone().or_else(|| {
            query.pattern.as_ref().map(|pattern| {
                if let Some(strictness) = &query.strictness {
                    serde_json::json!({"pattern":{"context":pattern,"strictness":strictness}})
                } else {
                    serde_json::json!({"pattern":pattern})
                }
            })
        }),
        Predicate::All {
            within: MatchScope::Node,
            clauses,
        } => Some(
            serde_json::json!({"all":clauses.iter().map(native_rule).collect::<Option<Vec<_>>>()?}),
        ),
        Predicate::Any {
            within: MatchScope::Node,
            clauses,
        } => Some(
            serde_json::json!({"any":clauses.iter().map(native_rule).collect::<Option<Vec<_>>>()?}),
        ),
        Predicate::Not {
            within: MatchScope::Node,
            clause,
        } => Some(serde_json::json!({"not":native_rule(clause)?})),
        _ => None,
    }
}

fn line_conjunction(
    root: &Path,
    input: &ProviderInput,
    clauses: &[Predicate],
    token: &CancellationToken,
    limit: Option<usize>,
    context: &mut rust::EvaluationContext,
) -> Result<rust::Evaluation, Error> {
    let mut groups: Option<BTreeMap<usize, Vec<MatchRange>>> = None;
    let mut diagnostics = Vec::new();
    let mut complete = true;
    for clause in clauses {
        let evaluation = evaluate(root, input, clause, token, None, context)?;
        complete &= evaluation.complete;
        diagnostics.extend(evaluation.diagnostics);
        let mut candidates: BTreeMap<usize, MatchRange> = BTreeMap::new();
        for row in evaluation.rows {
            let range = row.range.ok_or_else(|| {
                Error::Unavailable(
                    "line-scoped conjunction requires original line coordinates".into(),
                )
            })?;
            for line in range.line_start..=range.line_end {
                candidates.entry(line).or_insert_with(|| range.clone());
            }
        }
        groups = Some(if let Some(mut existing) = groups {
            existing.retain(|line, witnesses| {
                if let Some(candidate) = candidates.get(line) {
                    witnesses.push(candidate.clone());
                    true
                } else {
                    false
                }
            });
            existing
        } else {
            candidates
                .into_iter()
                .map(|(line, range)| (line, vec![range]))
                .collect()
        });
    }
    let groups = groups.unwrap_or_default();
    let mut rows = Vec::new();
    let mut offset = 0;
    let mut limited = false;
    for (line, bytes) in input.bytes.split_inclusive(|&b| b == b'\n').enumerate() {
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }
        if let Some(witnesses) = groups.get(&(line + 1)) {
            let mut row = record(
                input,
                Some(MatchRange {
                    byte_start: offset,
                    byte_end: offset + bytes.len(),
                    line_start: line + 1,
                    line_end: line + 1,
                }),
                "line_conjunction",
                String::from_utf8_lossy(bytes).into_owned(),
            );
            for (index, range) in witnesses.iter().enumerate() {
                row.captures
                    .insert(format!("clause_{index}"), vec![range.clone()]);
            }
            rows.push(row);
            if limit.is_some_and(|limit| rows.len() >= limit) {
                limited = true;
                break;
            }
        }
        offset += bytes.len();
    }
    Ok(rust::Evaluation {
        rows,
        complete: complete && !limited,
        limited: limited && complete,
        diagnostics,
    })
}
