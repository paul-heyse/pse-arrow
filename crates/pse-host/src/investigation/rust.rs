// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native source, analyzer and acquired-API adapters; no parallel semantic world.
use super::{providers::ProviderInput, *};
use crate::{
    source_proposal::{ChangeProposal, EditCandidate, ProducerContext},
    source_read::{SourceRequest, SourceText},
};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, Mutex},
};
use tokio_util::sync::CancellationToken;

fn native(error: impl std::error::Error + Send + Sync + 'static) -> Error {
    Error::Io(std::io::Error::other(error))
}
/// Candidate rows and their examination facts travel together across composition.
#[derive(Debug)]
pub struct Evaluation {
    /// Positive original witnesses, including recoverable syntax candidates.
    pub rows: Vec<SearchResult>,
    /// True only when the native provider completed relevant examination.
    pub complete: bool,
    /// Examination stopped solely at the selected result demand, without provider unknowns.
    pub limited: bool,
    /// Recovery/unsupported facts do not become negative matches.
    pub diagnostics: Vec<QueryDiagnostic>,
}

/// Compiled native selectors belong to one request, shared by its file workers.
#[derive(Debug, Default)]
pub struct CompiledProgram {
    syntax: Mutex<BTreeMap<String, Arc<crate::source_query::CompiledSyntax>>>,
    lexical: Mutex<BTreeMap<String, Arc<grep_regex::RegexMatcher>>>,
    modules: Mutex<BTreeMap<(std::path::PathBuf, String), Arc<providers::ModuleGraph>>>,
}
impl CompiledProgram {
    fn modules(
        &self,
        root: &Path,
        kind: &SourceKind,
        token: &CancellationToken,
    ) -> Result<Arc<providers::ModuleGraph>, Error> {
        let key = (root.to_owned(), serde_json::to_string(kind)?);
        let mut cache = self
            .modules
            .lock()
            .map_err(|_| Error::Unavailable("written module preparation owner panicked".into()))?;
        if let Some(graph) = cache.get(&key) {
            return Ok(graph.clone());
        }
        let graph = Arc::new(providers::ModuleGraph::discover(
            root,
            &SourceSelector {
                kind: kind.clone(),
                ..Default::default()
            },
            token,
        )?);
        cache.insert(key, graph.clone());
        Ok(graph)
    }
    fn syntax(
        &self,
        query: &SyntaxQuery,
    ) -> Result<Arc<crate::source_query::CompiledSyntax>, Error> {
        let key = serde_json::to_string(query)?;
        let mut compiled = self
            .syntax
            .lock()
            .map_err(|_| Error::Unavailable("native query preparation owner panicked".into()))?;
        if let Some(found) = compiled.get(&key) {
            return Ok(found.clone());
        }
        let selector = if let Some(text) = &query.tree_sitter {
            crate::source_query::CompiledSyntax::tree_sitter(text)
        } else {
            let rule = if let Some(rule) = &query.rule {
                rule.clone()
            } else {
                let pattern = query
                    .pattern
                    .clone()
                    .ok_or_else(|| Error::Invalid("missing syntax pattern".into()))?;
                if let Some(strictness) = &query.strictness {
                    serde_json::json!({"pattern":{"context":pattern,"strictness":strictness}})
                } else {
                    serde_json::json!({"pattern":pattern})
                }
            };
            crate::source_query::CompiledSyntax::rule(&rule)
        }
        .map_err(native)?;
        let selector = Arc::new(selector);
        compiled.insert(key, selector.clone());
        Ok(selector)
    }
}
/// Per-file exact acquisition and compatible parse; AST handles never cross workers.
#[derive(Debug)]
pub struct EvaluationContext {
    program: Arc<CompiledProgram>,
    selection: Option<(std::path::PathBuf, std::path::PathBuf, SourceKind)>,
    source: Option<SourceText>,
    parsed: Option<crate::source_query::ParsedSyntax>,
}
impl EvaluationContext {
    /// A request shares only immutable compiled selectors, not mutable parser state.
    pub fn new(program: Arc<CompiledProgram>) -> Self {
        Self {
            program,
            selection: None,
            source: None,
            parsed: None,
        }
    }
    /// Share one native lexical matcher for every occurrence of a request clause.
    pub fn lexical_matcher(
        &self,
        query: &TextQuery,
        prepare: impl FnOnce() -> Result<grep_regex::RegexMatcher, Error>,
    ) -> Result<Arc<grep_regex::RegexMatcher>, Error> {
        let key = serde_json::to_string(query)?;
        let mut matchers =
            self.program.lexical.lock().map_err(|_| {
                Error::Unavailable("lexical query preparation owner panicked".into())
            })?;
        if let Some(matcher) = matchers.get(&key) {
            return Ok(matcher.clone());
        }
        let matcher = Arc::new(prepare()?);
        matchers.insert(key, matcher.clone());
        Ok(matcher)
    }
    fn source(&mut self, root: &Path, input: &ProviderInput) -> Result<&SourceText, Error> {
        if self
            .selection
            .as_ref()
            .is_some_and(|(original_root, path, kind)| {
                original_root != root || path != &input.path || kind != &input.source
            })
        {
            return Err(Error::Stale(
                "file evaluation context was reused for another source selection".into(),
            ));
        }
        if self.source.is_none() {
            self.source = Some(exact(root, input)?);
            self.selection = Some((root.into(), input.path.clone(), input.source.clone()));
        }
        let source = self
            .source
            .as_ref()
            .ok_or_else(|| Error::Unavailable("exact source was not retained".into()))?;
        if source.text().as_bytes() != input.bytes {
            return Err(Error::Stale(
                "file evaluation context was reused for different source bytes".into(),
            ));
        }
        Ok(source)
    }
    fn parsed(
        &mut self,
        root: &Path,
        input: &ProviderInput,
    ) -> Result<(&SourceText, &crate::source_query::ParsedSyntax), Error> {
        self.source(root, input)?;
        let source = self
            .source
            .as_ref()
            .ok_or_else(|| Error::Unavailable("exact source was not retained".into()))?;
        if self.parsed.is_none() {
            self.parsed = Some(crate::source_query::ParsedSyntax::new(source).map_err(native)?);
        }
        Ok((
            source,
            self.parsed
                .as_ref()
                .ok_or_else(|| Error::Unavailable("native parse was not retained".into()))?,
        ))
    }
}

fn source_request(root: &Path, path: &Path, kind: &SourceKind) -> SourceRequest {
    match kind {
        SourceKind::Working => SourceRequest::Working {
            path: root.join(path),
        },
        SourceKind::Index => SourceRequest::Index {
            repository: root.into(),
            path: path.into(),
        },
        SourceKind::Revision { revision } => SourceRequest::Blob {
            repository: root.into(),
            path: path.into(),
            revision: revision.clone(),
        },
    }
}
fn range(value: &crate::source_read::SourceRange) -> MatchRange {
    MatchRange {
        byte_start: value.byte_start,
        byte_end: value.byte_end,
        line_start: value.line_start,
        line_end: value.line_end,
    }
}
fn exact(root: &Path, input: &ProviderInput) -> Result<SourceText, Error> {
    if input.source == SourceKind::Working {
        let text = std::str::from_utf8(&input.bytes)
            .map_err(native)?
            .to_owned();
        return SourceText::new(root.join(&input.path), text).map_err(native);
    }
    let source = source_request(root, &input.path, &input.source)
        .acquire()
        .map_err(native)?;
    if source.text().as_bytes() != input.bytes {
        return Err(Error::Stale(
            "source changed between selection and structural acquisition".into(),
        ));
    }
    Ok(source)
}
fn record(
    input: &ProviderInput,
    source: &SourceText,
    location: Option<MatchRange>,
    kind: String,
    name: Option<String>,
    context: String,
) -> SearchResult {
    let mut guard = input.guard.clone().unwrap_or_default();
    guard.sha256 = Some(source.identity().source_sha256().into());
    SearchResult {
        source: String::new(),
        kind,
        name,
        range: location.clone(),
        context,
        bytes: None,
        captures: BTreeMap::new(),
        entity: None,
        candidate_reference: None,
        refinement: None,
        reference: Some(ResultReference::new(Locator::Source {
            path: input.path.clone().into(),
            source: input.source.clone(),
            range: location,
            guard: Some(guard),
        })),
    }
}
/// Source-local native operations; semantic/API universes use `global_search` once instead.
pub fn evaluate(
    root: &Path,
    input: &ProviderInput,
    predicate: &Predicate,
    token: &CancellationToken,
    limit: Option<usize>,
    context: &mut EvaluationContext,
) -> Result<Evaluation, Error> {
    if token.is_cancelled() {
        return Err(Error::Cancelled);
    }
    if matches!(
        predicate,
        Predicate::Symbol(SymbolQuery {
            provider: DiscoveryProvider::Source,
            ..
        })
    ) && input.path.extension().is_none_or(|ext| ext != "rs")
    {
        return Ok(Evaluation {
            rows: Vec::new(),
            complete: true,
            limited: false,
            diagnostics: Vec::new(),
        });
    }
    match predicate {
        Predicate::Syntax(query) => {
            let selector = context.program.syntax(query)?;
            let (source, parsed) = context.parsed(root, input)?;
            let native = selector
                .evaluate(source, parsed, limit, token)
                .map_err(native)?;
            if token.is_cancelled() {
                return Err(Error::Cancelled);
            }
            let mut result = Vec::new();
            for matched in native.matches {
                let mut row = record(
                    input,
                    source,
                    Some(range(&matched.locator.range)),
                    matched.kind,
                    None,
                    matched.text,
                );
                for capture in matched.captures {
                    if !query.capture.is_empty() && !query.capture.contains(&capture.name) {
                        continue;
                    }
                    row.captures.insert(
                        capture.name,
                        capture
                            .nodes
                            .iter()
                            .map(|node| range(&node.locator.range))
                            .collect(),
                    );
                }
                result.push(row);
            }
            let diagnostics = if native.recovery {
                vec![QueryDiagnostic {code:"search.syntax_recovery".into(),message:"native parser recovered malformed Rust; positive syntax witnesses remain usable, exhaustive absence and exact semantic/parse completeness are not established".into(),source:None}]
            } else {
                Vec::new()
            };
            Ok(Evaluation {
                rows: result,
                complete: native.complete && !native.recovery,
                limited: !native.complete && !native.recovery,
                diagnostics,
            })
        }
        Predicate::Symbol(query) if query.provider == DiscoveryProvider::Source => {
            if query.matching == NameMatch::Fuzzy {
                return Err(Error::Unavailable("source declarations have no selected native fuzzy matcher; use exact/prefix/contains or prepared semantic fuzzy discovery".into()));
            }
            if input.path.extension().is_none_or(|ext| ext != "rs") {
                return Ok(Evaluation {
                    rows: Vec::new(),
                    complete: true,
                    limited: false,
                    diagnostics: Vec::new(),
                });
            }
            let graph = context.program.modules(root, &input.source, token)?;
            let contexts = graph.contexts(&input.path);
            let qualified_query = query.name.contains("::") || query.module.is_some();
            let missing = contexts.is_empty() && qualified_query;
            let source = context.source(root, input)?;
            let outline =
                crate::source_read::rust(source.text(), crate::source_read::RustEdition::E2024);
            let mut rows = Vec::new();
            for entry in outline.entries {
                if token.is_cancelled() {
                    return Err(Error::Cancelled);
                }
                if !query.kinds.is_empty()
                    && !query
                        .kinds
                        .iter()
                        .any(|kind| source_kind_matches(&entry, kind))
                {
                    continue;
                }
                if query.signature.as_ref().is_some_and(|signature| {
                    !entry
                        .written_header
                        .as_ref()
                        .and_then(|header| source.text().get(header.byte_start..header.byte_end))
                        .is_some_and(|header| header.contains(signature))
                }) {
                    continue;
                }
                if query
                    .visibility
                    .as_ref()
                    .is_some_and(|visibility| match visibility.as_str() {
                        "pub" | "public" => entry.written_visibility.as_deref() != Some("pub"),
                        "private" => entry.written_visibility.is_some(),
                        written => entry.written_visibility.as_deref() != Some(written),
                    })
                {
                    continue;
                }
                let prefixes = if contexts.is_empty() {
                    vec![Vec::new()]
                } else {
                    contexts
                        .iter()
                        .map(|context| context.modules.clone())
                        .collect()
                };
                for prefix in prefixes {
                    if missing {
                        continue;
                    }
                    let qualified = prefix
                        .iter()
                        .chain(&entry.path)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join("::");
                    if !name_matches(
                        &qualified,
                        entry.path.last().map_or("", String::as_str),
                        &query.name,
                        query.matching,
                    ) {
                        continue;
                    }
                    if query.module.as_ref().is_some_and(|module| {
                        let enclosing = prefix
                            .iter()
                            .chain(&entry.modules)
                            .cloned()
                            .collect::<Vec<_>>()
                            .join("::");
                        enclosing != *module && !enclosing.starts_with(&format!("{module}::"))
                    }) {
                        continue;
                    }
                    rows.push(record(
                        input,
                        source,
                        Some(range(&entry.range)),
                        entry.kind.clone(),
                        Some(qualified),
                        entry.signature.clone(),
                    ));
                    if limit.is_some_and(|limit| rows.len() >= limit) {
                        break;
                    }
                }
                if limit.is_some_and(|limit| rows.len() >= limit) {
                    break;
                }
            }
            let known =
                outline.parse_issues.is_empty() && graph.complete_for(&input.path) && !missing;
            let limited = known && limit.is_some_and(|limit| rows.len() >= limit);
            let complete = known && !limited;
            let mut diagnostics = outline
                .parse_issues
                .into_iter()
                .map(|issue| QueryDiagnostic {
                    code: "search.source_parse".into(),
                    message: format!("{issue:?}"),
                    source: None,
                })
                .collect::<Vec<_>>();
            diagnostics.extend(graph.diagnostics_for(&input.path).cloned());
            if missing {
                diagnostics.push(QueryDiagnostic {code:"search.module_unavailable".into(),message:"selected source has no established authored crate/module route; qualified absence is unknown".into(),source:None});
            }
            if contexts.len() > 1 || contexts.iter().any(|context| context.conditional) {
                diagnostics.push(QueryDiagnostic {code:"search.written_contexts".into(),message:format!("written module routes retain target/configuration ambiguity: {contexts:?}; no active compiler context is claimed"),source:None});
            }
            Ok(Evaluation {
                rows,
                complete,
                limited,
                diagnostics,
            })
        }
        _ => Err(Error::Unavailable(
            "selected provider requires an explicit global semantic/API operation".into(),
        )),
    }
}

fn name_matches(qualified: &str, simple: &str, needle: &str, matching: NameMatch) -> bool {
    let selected = if needle.contains("::") {
        qualified
    } else {
        simple
    };
    match matching {
        NameMatch::Contains => qualified.to_lowercase().contains(&needle.to_lowercase()),
        NameMatch::Exact => selected == needle,
        NameMatch::Prefix => selected.starts_with(needle),
        NameMatch::Fuzzy => true,
    }
}

fn source_kind_matches(entry: &crate::source_read::Entry, selected: &str) -> bool {
    let native = match selected.to_ascii_lowercase().as_str() {
        "function" => return entry.kind == "FN" && !entry.method,
        "associated_function" => return entry.kind == "FN" && entry.associated && !entry.method,
        "method" => return entry.kind == "FN" && entry.method,
        "type" | "type_alias" => "TYPE_ALIAS",
        "constant" => "CONST",
        "macro" => return matches!(entry.kind.as_str(), "MACRO_RULES" | "MACRO_DEF"),
        _ => return entry.kind.eq_ignore_ascii_case(selected),
    };
    entry.kind == native
}
fn hydrate(root: &Path, locator: &Locator) -> Result<(SourceText, MatchRange), Error> {
    let Locator::Source {
        path,
        source: kind,
        range: Some(location),
        guard,
    } = locator
    else {
        return Err(Error::Invalid(
            "a source proposal needs an addressed exact range".into(),
        ));
    };
    if let Some(original) = guard.as_ref().and_then(|guard| guard.root.as_ref()) {
        if original.to_path_buf().canonicalize()? != root.canonicalize()? {
            return Err(Error::Stale(
                "source reference belongs to a different checkout root".into(),
            ));
        }
    }
    let request = source_request(root, &path.to_path_buf(), kind);
    let current = providers::acquire(root, &request)?;
    if let Some(expected) = guard {
        if let Some(now) = &current.guard {
            if expected.git_blob.is_some() && expected.git_blob != now.git_blob {
                return Err(Error::Stale(
                    "selected Git blob no longer matches its source reference".into(),
                ));
            }
            if expected.length.is_some() && expected.length != now.length
                || expected.modified_nanos.is_some()
                    && expected.modified_nanos != now.modified_nanos
            {
                return Err(Error::Stale(
                    "selected source changed before exact hydration".into(),
                ));
            }
        }
        if let Some(observed) = &expected.observed {
            let bytes = current
                .bytes
                .get(observed.range.byte_start..observed.range.byte_end)
                .ok_or_else(|| {
                    Error::Stale("original lexical witness is outside current source".into())
                })?;
            if crate::probe::digest(bytes) != observed.sha256 {
                return Err(Error::Stale(
                    "original lexical witness bytes changed before exact hydration".into(),
                ));
            }
        }
    }
    let source = exact(root, &current)?;
    if guard
        .as_ref()
        .and_then(|guard| guard.sha256.as_ref())
        .is_some_and(|hash| hash != source.identity().source_sha256())
    {
        return Err(Error::Stale(
            "source content no longer matches selected result".into(),
        ));
    }
    source
        .text()
        .get(location.byte_start..location.byte_end)
        .ok_or_else(|| {
            Error::Invalid("selected range is outside original UTF-8 boundaries".into())
        })?;
    Ok((source, location.clone()))
}

/// Exact-byte preview; semantic rename remains with its original native producer.
pub fn propose(
    root: &Path,
    locator: &Locator,
    replacement: Option<String>,
    rename: Option<String>,
) -> Result<ChangeProposal, Error> {
    if rename.is_some() {
        return Err(Error::Invalid(
            "semantic rename requires a semantic locator and the asynchronous native producer"
                .into(),
        ));
    }
    let replacement = replacement.ok_or_else(|| {
        Error::Invalid("select an explicit replacement or native semantic rename".into())
    })?;
    let (source, location) = hydrate(root, locator)?;
    authored(root, &source)?;
    let edit = EditCandidate::new(
        source.identity().clone(),
        location.byte_start..location.byte_end,
        replacement,
    );
    ChangeProposal::new(ProducerContext::Syntax {backend:"exact-source-hydration/v1".into(),configuration:"selected source/range; byte replacement; no inferred semantic applicability".into()},vec![source],vec![edit],vec![0],vec!["Immutable preview only; no source/index write, semantic resolution or compilation established".into()],vec![]).map_err(native)
}

fn environment() -> crate::environment::Environment {
    std::env::vars_os().collect()
}
fn authored(root: &Path, source: &SourceText) -> Result<(), Error> {
    if let Some(reason) =
        crate::agent_hooks::protected(root, source.identity().path()).map_err(native)?
    {
        return Err(Error::Invalid(format!(
            "selected preview targets protected source: {reason}"
        )));
    }
    Ok(())
}
/// Revalidate a selected structural candidate before expanding native template captures.
pub fn propose_pattern(
    root: &Path,
    locator: &Locator,
    pattern: &str,
    replacement: &str,
) -> Result<ChangeProposal, Error> {
    let (source, location) = hydrate(root, locator)?;
    authored(root, &source)?;
    crate::source_query::preview_selected(
        source,
        location.byte_start..location.byte_end,
        pattern,
        crate::source_query::Strictness::Smart,
        replacement,
    )
    .map_err(native)
}
/// A global provider runs once; it neither enumerates source files nor prepares anything.
pub async fn global_search(
    root: &Path,
    request: &Request,
    token: &CancellationToken,
    emit: &mut (dyn FnMut(Event) -> Result<(), Error> + Send),
) -> Result<Summary, Error> {
    if token.is_cancelled() {
        return Err(Error::Cancelled);
    }
    let limit = if request.answer.kind == AnswerKind::Exists {
        1
    } else if request.answer.kind == AnswerKind::Count {
        usize::MAX
    } else {
        request.answer.limit.unwrap_or(usize::MAX)
    };
    match &request.predicate {
        Predicate::Api(query) => api_search(root, query, None, request, limit, emit),
        Predicate::Symbol(query) if query.provider == DiscoveryProvider::Api => {
            if query.matching == NameMatch::Fuzzy || query.signature.is_some() {
                return Err(Error::Unavailable("API symbol discovery supports exact/prefix/contains names and native declaration kind/public-path filters; fuzzy/written-source signature filters require their selected provider".into()));
            }
            if query
                .visibility
                .as_deref()
                .is_some_and(|visibility| !matches!(visibility, "pub" | "public"))
            {
                return Err(Error::Unavailable("the acquired API public-path universe does not establish private destination visibility".into()));
            }
            let api = ApiQuery {
                query: query.name.clone(),
                artifact: query.artifact.clone(),
                dependency: query.dependency.clone(),
                ..ApiQuery::default()
            };
            api_search(root, &api, Some(query), request, limit, emit)
        }
        Predicate::Symbol(query) if query.provider == DiscoveryProvider::Semantic => {
            if query.signature.is_some() {
                return Err(Error::Unavailable("workspace symbols do not supply full written signatures; use source signature or acquired API structural filters".into()));
            }
            if request.sources.kind != SourceKind::Working {
                return Err(Error::Unavailable("live semantic discovery cannot interpret index/revision source as working bytes; select a separately prepared context explicitly".into()));
            }
            if request.answer.cursor.is_some() || request.answer.order == ResultOrder::Stable {
                return Err(Error::Unavailable("provider-ranked native workspace discovery has no reproducible whole-universe continuation; request arrival-order candidates in the selected session".into()));
            }
            if request.answer.count_unit != CountUnit::Matches {
                return Err(Error::Unavailable("native workspace-symbol output does not establish complete file/line/entity populations; count candidate matches or inspect original native relations".into()));
            }
            if query.visibility.is_some() {
                return Err(Error::Unavailable("native workspace symbols do not expose authoritative visibility; inspect source or request a supported typed native provider".into()));
            }
            if query.module.is_some() {
                return Err(Error::Unavailable("native workspace-symbol container labels do not establish full module identity; use source-module discovery or a supported selected semantic view".into()));
            }
            let env = environment();
            let record = crate::semantic::load(root, &env)
                .map_err(Error::from)?
                .ok_or_else(|| {
                    Error::Unavailable("explicitly prepare the native analyzer session".into())
                })?;
            let project = &record.configuration.project;
            let selector = providers::Selection::for_prepared_context(project, &request.sources)?;
            let selected = providers::select_prepared_context(project, &request.sources, token)?
                .into_iter()
                .map(|source| source.path().canonicalize().map_err(Error::from))
                .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
            if token.is_cancelled() {
                return Err(Error::Cancelled);
            }
            // Native response caps are not the user's filtered result allowance.
            let native_request =
                crate::semantic::Request::WorkspaceSymbols(crate::semantic::WorkspaceQuery {
                    query: query.name.clone(),
                    matching: match query.matching {
                        NameMatch::Exact => crate::semantic::NativeNameMode::Exact,
                        NameMatch::Prefix => crate::semantic::NativeNameMode::Prefix,
                        NameMatch::Contains | NameMatch::Fuzzy => {
                            crate::semantic::NativeNameMode::Fuzzy
                        }
                    },
                    limit: usize::MAX,
                    include_dependencies: request.sources.include_dependencies,
                    if_generation: None,
                });
            let value =
                crate::semantic::client_cancellable(root, &env, &record, &native_request, token)
                    .await
                    .map_err(Error::from)?;
            let answer: crate::semantic::WorkspaceAnswer = serde_json::from_value(value)?;
            emit(Event::Source(SourceContext {
                id: "semantic".into(),
                path: record.configuration.project.clone().into(),
                origin: Some(SourceKind::Working),
                provider: answer.session.backend.clone(),
                generation: Some(answer.generation.to_string()),
                searched_encoding: None,
                native: Some(NativeContext {
                    session_invocation: record.invocation.to_string(),
                    generation: answer.generation,
                    analysis: answer.session.clone(),
                }),
            }))?;
            let mut summary = Summary::default();
            for symbol in answer.symbols.evidence.as_array().into_iter().flatten() {
                if token.is_cancelled() {
                    return Err(Error::Cancelled);
                }
                let Some(uri) = symbol
                    .pointer("/location/uri")
                    .and_then(serde_json::Value::as_str)
                else {
                    continue;
                };
                let Some(path) = url::Url::parse(uri)
                    .ok()
                    .and_then(|uri| uri.to_file_path().ok())
                else {
                    continue;
                };
                let path = path.canonicalize()?;
                let accepted = if let Ok(relative) = path.strip_prefix(project) {
                    selected.contains(&path) && selector.accepts(relative, &request.sources)
                } else {
                    request.sources.include_dependencies
                        && selector.accepts(&path, &request.sources)
                };
                if !accepted {
                    continue;
                }
                let name = symbol["name"].as_str().unwrap_or("").to_owned();
                // Native module-path resolution established the qualified prefix;
                // apply the remaining written-name policy only to the final item.
                let item_needle = query.name.rsplit("::").next().unwrap_or(&query.name);
                if !name_matches(&name, &name, item_needle, query.matching) {
                    continue;
                }
                let kind = symbol["kind"]
                    .as_u64()
                    .map(symbol_kind)
                    .or_else(|| symbol["kind"].as_str().map(native_symbol_kind))
                    .unwrap_or("unknown");
                if !query.kinds.is_empty()
                    && !query
                        .kinds
                        .iter()
                        .any(|selected| selected.eq_ignore_ascii_case(kind))
                {
                    continue;
                }
                let reference = semantic_locator(
                    &record.configuration.project,
                    record.invocation.as_str(),
                    &answer,
                    symbol,
                )?;
                if request.answer.kind != AnswerKind::Count {
                    emit(Event::Result(SearchResult {
                        source: "semantic".into(),
                        kind: kind.into(),
                        name: Some(name),
                        range: None,
                        context: serde_json::to_string(symbol)?,
                        bytes: None,
                        reference,
                        captures: BTreeMap::new(),
                        entity: None,
                        candidate_reference: None,
                        refinement: None,
                    }))?;
                    summary.emitted += 1;
                }
                summary.matches += 1;
                if summary.matches >= limit as u64 {
                    summary.stop = StopReason::Limit;
                    break;
                }
            }
            // The protocol's returned array does not prove that the native fuzzy
            // workspace search examined/exposed every candidate in the universe.
            summary.complete = false;
            summary.count = Count::AtLeast(summary.matches);
            if answer.symbols.omitted > 0 {
                summary.stop = StopReason::Limit;
            }
            emit(Event::Diagnostic {code:"search.semantic_scope".into(),message:"native workspace-symbol response is non-exhaustive/provider-ranked; missing candidates are not established absent, and a returned array does not prove uncapped search".into(),source:Some("semantic".into())})?;
            Ok(summary)
        }
        _ => Err(Error::Invalid(
            "expected an explicitly selected global semantic/API query".into(),
        )),
    }
}
fn symbol_kind(kind: u64) -> &'static str {
    match kind {
        2 => "module",
        5 => "class",
        6 => "method",
        8 => "field",
        9 => "constructor",
        10 => "enum",
        11 => "interface",
        12 => "function",
        13 => "variable",
        14 => "constant",
        22 => "enum_member",
        23 => "struct",
        26 => "type_parameter",
        _ => "other",
    }
}
fn native_symbol_kind(kind: &str) -> &str {
    match kind {
        "Function" => "function",
        "Method" => "method",
        "Module" => "module",
        "Struct" => "struct",
        "Enum" => "enum",
        "Trait" => "trait",
        "TypeAlias" => "type_alias",
        "Const" => "constant",
        "Static" => "static",
        "Macro" => "macro",
        _ => kind,
    }
}
fn semantic_locator(
    project: &Path,
    invocation: &str,
    answer: &crate::semantic::WorkspaceAnswer,
    symbol: &serde_json::Value,
) -> Result<Option<ResultReference>, Error> {
    let Some(uri) = symbol
        .pointer("/location/uri")
        .and_then(serde_json::Value::as_str)
    else {
        return Ok(None);
    };
    let path = url::Url::parse(uri)
        .map_err(|error| Error::Invalid(error.to_string()))?
        .to_file_path()
        .map_err(|()| Error::Invalid("native symbol URI is not a local file".into()))?;
    let Some(position) = symbol.pointer("/location/range/start") else {
        return Ok(None);
    };
    let Some(original_digest) = symbol["source_sha256"].as_str() else {
        return Ok(None);
    };
    let source = SourceRequest::Working { path: path.clone() }
        .acquire()
        .map_err(native)?;
    if original_digest != source.identity().source_sha256() {
        return Err(Error::Stale(
            "native declaration source changed before publishing its locator".into(),
        ));
    }
    let offset = crate::semantic::native_byte_position(
        source.text(),
        position,
        &answer.session.position_encoding,
    )
    .map_err(Error::from)?;
    let line = source.text()[..offset]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1;
    let start = source.text()[..offset]
        .rfind('\n')
        .map_or(0, |index| index + 1);
    let column = source.text()[start..offset].chars().count() + 1;
    Ok(Some(ResultReference::new(Locator::Semantic {
        project: project.into(),
        path: path.into(),
        line,
        column,
        generation: Some(answer.generation.to_string()),
        source_sha256: Some(source.identity().source_sha256().into()),
        candidate_fingerprint: None,
        session_invocation: Some(invocation.into()),
    })))
}
/// Invoke the actual semantic owner with source and generation guards intact.
/// Its checked proposal remains original owner evidence, not an unchecked re-decoded proposal.
pub async fn semantic_operation(
    root: &Path,
    locator: &Locator,
    kind: crate::semantic::Kind,
) -> Result<serde_json::Value, Error> {
    semantic_operation_cancellable(root, locator, kind, &CancellationToken::new()).await
}
/// Use one selected native request, retaining actual snapshot cancellation settlement.
pub async fn semantic_operation_cancellable(
    root: &Path,
    locator: &Locator,
    kind: crate::semantic::Kind,
    token: &CancellationToken,
) -> Result<serde_json::Value, Error> {
    semantic_query(root, locator, kind, 20, token).await
}
async fn semantic_query(
    root: &Path,
    locator: &Locator,
    kind: crate::semantic::Kind,
    limit: usize,
    token: &CancellationToken,
) -> Result<serde_json::Value, Error> {
    let Locator::Semantic {
        project,
        path,
        line,
        column,
        generation,
        source_sha256,
        candidate_fingerprint,
        session_invocation,
    } = locator
    else {
        return Err(Error::Invalid(
            "native semantic operation requires a semantic locator".into(),
        ));
    };
    let env = environment();
    let record = crate::semantic::load(root, &env)
        .map_err(Error::from)?
        .ok_or_else(|| {
            Error::Unavailable("explicitly prepare the native analyzer session".into())
        })?;
    if session_invocation
        .as_ref()
        .is_some_and(|expected| expected != record.invocation.as_str())
    {
        return Err(Error::Stale(
            "semantic locator belongs to a different original session".into(),
        ));
    }
    if record.configuration.project.canonicalize()? != project.canonicalize()? {
        return Err(Error::Stale(
            "semantic locator belongs to a different prepared project".into(),
        ));
    }
    let generation = generation
        .as_deref()
        .map(str::parse::<u64>)
        .transpose()
        .map_err(|_| Error::Invalid("invalid semantic generation".into()))?;
    let query = crate::semantic::Query {
        kind,
        file: path.to_path_buf(),
        line: *line,
        column: *column,
        if_source: source_sha256.clone(),
        if_generation: generation,
        if_candidate: candidate_fingerprint.clone(),
        limit,
        end_line: None,
    };
    crate::semantic::client_cancellable(
        root,
        &env,
        &record,
        &crate::semantic::Request::Query(query),
        token,
    )
    .await
    .map_err(Error::from)
}
/// Original artifact guards and selected native configuration define a batch proof.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ApiContextKey {
    artifact: std::path::PathBuf,
    sha256: Option<String>,
    context: Option<String>,
    native_configuration: String,
    native_session: String,
}
/// One request owns acquired-API/source proofs; no artifact/package scan repeats per candidate.
#[derive(Debug, Default)]
pub struct ResolutionContext {
    api: Mutex<BTreeMap<ApiContextKey, Arc<crate::api::bridge::SourceBridge>>>,
}
impl ResolutionContext {
    async fn api_source(
        &self,
        root: &Path,
        locator: &Locator,
        record: &crate::semantic::SessionRecord,
        environment: &crate::environment::Environment,
        token: &CancellationToken,
    ) -> Result<(SourceText, std::ops::Range<usize>), Error> {
        let Locator::Api {
            artifact,
            item,
            artifact_sha256,
            context_key,
            public_path,
        } = locator
        else {
            return Err(Error::Invalid(
                "API source bridge requires an acquired artifact locator".into(),
            ));
        };
        let key = ApiContextKey {
            artifact: root.join(artifact),
            sha256: artifact_sha256.clone(),
            context: context_key.clone(),
            native_configuration: record.configuration.identity()?,
            native_session: record.invocation.as_str().into(),
        };
        let existing = {
            self.api
                .lock()
                .map_err(|_| Error::Unavailable("API resolution context owner panicked".into()))?
                .get(&key)
                .cloned()
        };
        let bridge = if let Some(bridge) = existing {
            bridge
        } else {
            let artifact = Arc::new(checked_api(root, locator)?);
            let value = crate::semantic::client_cancellable(
                root,
                environment,
                record,
                &crate::semantic::Request::PackageContext {
                    package_id: artifact.context.package_id.clone(),
                },
                token,
            )
            .await?;
            let captured: crate::semantic::PackageContextAnswer = serde_json::from_value(value)?;
            if captured.session.configuration != record.configuration {
                return Err(Error::Stale(
                    "captured native package context belongs to another selected configuration"
                        .into(),
                ));
            }
            let bridge = Arc::new(crate::api::bridge::SourceBridge::prepare(
                artifact,
                &record.configuration,
                &captured.packages,
                environment,
            )?);
            self.api
                .lock()
                .map_err(|_| Error::Unavailable("API resolution context owner panicked".into()))?
                .insert(key, bridge.clone());
            bridge
        };
        bridge
            .source(item, public_path.as_deref())
            .map_err(Error::from)
    }
    /// Recheck original acquired source/context premises before accepting a completed batch.
    pub fn verify_unchanged(&self) -> Result<(), Error> {
        let cache = self
            .api
            .lock()
            .map_err(|_| Error::Unavailable("API resolution context owner panicked".into()))?;
        let environment = environment();
        for bridge in cache.values() {
            bridge.verify_unchanged(&environment)?;
        }
        Ok(())
    }
}
/// Lift a guarded current-source/API candidate to its real native declaration/callee token.
pub async fn source_semantic(
    root: &Path,
    locator: &Locator,
    token: &CancellationToken,
) -> Result<Locator, Error> {
    let context = ResolutionContext::default();
    let result = source_semantic_with_context(root, locator, token, &context).await?;
    context.verify_unchanged()?;
    Ok(result)
}
/// Lift a candidate while retaining the request's shared API/source proof owner.
pub async fn source_semantic_with_context(
    root: &Path,
    locator: &Locator,
    token: &CancellationToken,
    context: &ResolutionContext,
) -> Result<Locator, Error> {
    if token.is_cancelled() {
        return Err(Error::Cancelled);
    }
    if let Locator::Semantic { .. } = locator {
        return Ok(locator.clone());
    }
    let env = environment();
    let record = crate::semantic::load(root, &env)
        .map_err(Error::from)?
        .ok_or_else(|| {
            Error::Unavailable("explicitly prepare a semantic context before refinement".into())
        })?;
    let (source,range)=match locator {
        Locator::Source {source:SourceKind::Working,..}=>{let(source,range)=hydrate(root,locator)?;(source,range.byte_start..range.byte_end)},
        Locator::Api {..}=>context.api_source(root,locator,&record,&env,token).await?,
        _=>return Err(Error::Unavailable("semantic refinement requires current guarded source or an API declaration with matching original package/target/feature premises; index/revision evidence is not current source".into())),
    };
    let offset=crate::source_read::semantic_anchor(source.text(),range).ok_or_else(||Error::Unavailable("selection has no unique native declaration/callee token; inspect a narrower source witness".into()))?;
    Ok(semantic_position(&record, &source, offset))
}
fn semantic_position(
    record: &crate::semantic::SessionRecord,
    source: &SourceText,
    offset: usize,
) -> Locator {
    let prefix = &source.text()[..offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = prefix
        .rsplit('\n')
        .next()
        .map_or(1, |line| line.chars().count() + 1);
    Locator::Semantic {
        project: record.configuration.project.clone().into(),
        path: source.identity().path().to_owned().into(),
        line,
        column,
        generation: None,
        source_sha256: Some(source.identity().source_sha256().into()),
        candidate_fingerprint: None,
        session_invocation: Some(record.invocation.as_str().into()),
    }
}

async fn semantic_scope(
    root: &Path,
    locator: &Locator,
    token: &CancellationToken,
) -> Result<Locator, Error> {
    if token.is_cancelled() {
        return Err(Error::Cancelled);
    }
    if let Locator::Semantic { .. } = locator {
        return Ok(locator.clone());
    }
    let mut selected = locator.clone();
    let Locator::Source {
        source: SourceKind::Working,
        range,
        ..
    } = &mut selected
    else {
        return Err(Error::Unavailable(
            "native destination scope requires a guarded current source position".into(),
        ));
    };
    if range.is_none() {
        *range = Some(MatchRange {
            byte_start: 0,
            byte_end: 0,
            line_start: 1,
            line_end: 1,
        });
    }
    let (source, range) = hydrate(root, &selected)?;
    let offset =
        crate::source_read::semantic_anchor(source.text(), range.byte_start..range.byte_end)
            .unwrap_or(range.byte_start);
    let record = crate::semantic::load(root, &environment())?.ok_or_else(|| {
        Error::Unavailable("explicitly prepare a native destination context".into())
    })?;
    Ok(semantic_position(&record, &source, offset))
}

/// Ask actual native visibility and an available naming path at an unused destination scope.
pub async fn importability(
    root: &Path,
    target: &ResolvedEntity,
    destination: &Locator,
    token: &CancellationToken,
    _context: &ResolutionContext,
) -> Result<(Truth, serde_json::Value), Error> {
    let location = serde_json::json!({"path":target.identity.declaration.to_path_buf(),"start":target.identity.byte_start,"end":target.identity.byte_end});
    let (declaration, _) = guard_location(root, &target.locator, &target.native, &location)?;
    let Locator::Semantic {
        source_sha256: Some(source_sha256),
        ..
    } = declaration
    else {
        return Err(Error::Unavailable(
            "native import target has no original byte guard".into(),
        ));
    };
    let mut destination = semantic_scope(root, destination, token).await?;
    let Locator::Semantic {
        generation,
        session_invocation,
        ..
    } = &mut destination
    else {
        return Err(Error::Invalid("destination lost its native context".into()));
    };
    if session_invocation.as_deref() != Some(target.identity.session_invocation.as_str()) {
        return Err(Error::Stale(
            "native import target and destination belong to different sessions".into(),
        ));
    }
    *generation = Some(target.identity.generation.to_string());
    let answer = semantic_operation_cancellable(
        root,
        &destination,
        crate::semantic::Kind::Importable {
            target: crate::semantic::ImportTarget {
                file: target.identity.declaration.to_path_buf(),
                byte_start: target.identity.byte_start,
                byte_end: target.identity.byte_end,
                source_sha256,
            },
        },
        token,
    )
    .await?;
    let fact = answer["sections"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|section| section["category"] == "importability")
        .map(|section| &section["evidence"]);
    let truth = match fact.and_then(|fact| fact["importable"].as_bool()) {
        Some(true) => Truth::True,
        Some(false) => Truth::False,
        None => Truth::Unknown,
    };
    Ok((truth, answer))
}

/// One native resolved declaration; original candidate context remains separately inspectable.
#[derive(Debug)]
pub struct ResolvedEntity {
    /// Exact original session/configuration/generation and native declaration interval.
    pub identity: EntityIdentity,
    /// Guarded native position used to obtain the resolution.
    pub locator: Locator,
    /// Faithfully retained native answer, never a name-based reconstructed proof.
    pub native: serde_json::Value,
}
/// One directed native relationship evaluation, including unavailable/omitted coverage.
#[derive(Debug)]
pub struct RelationEvaluation {
    /// Original guarded native target/use/caller witnesses.
    pub rows: Vec<SearchResult>,
    /// Selected session contexts shared by those witnesses.
    pub sources: Vec<SourceContext>,
    /// Exact relevant native examination completed without omitted or unknown facts.
    pub complete: bool,
    /// Original coverage/unsupported mapping facts.
    pub diagnostics: Vec<QueryDiagnostic>,
}

/// Decode a native location only against the source bytes bound by that native answer.
pub fn guard_location(
    root: &Path,
    seed: &Locator,
    answer: &serde_json::Value,
    location: &serde_json::Value,
) -> Result<(Locator, MatchRange), Error> {
    let (path, bytes) = if let Some(path) = location.get("path") {
        let path: std::path::PathBuf = serde_json::from_value(path.clone())?;
        let start = location["start"]
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| Error::Invalid("native byte location has no supported start".into()))?;
        let end = location["end"]
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| Error::Invalid("native byte location has no supported end".into()))?;
        (path, Some((start, end)))
    } else {
        let uri = location
            .get("targetUri")
            .or_else(|| location.get("uri"))
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                Error::Unavailable("native result has no original local source location".into())
            })?;
        (
            url::Url::parse(uri)
                .map_err(|error| Error::Invalid(error.to_string()))?
                .to_file_path()
                .map_err(|()| Error::Unavailable("native location is not a local file".into()))?,
            None,
        )
    };
    let path = path.canonicalize()?;
    let original_uri = answer["source_uri"]
        .as_str()
        .and_then(|uri| url::Url::parse(uri).ok())
        .and_then(|uri| uri.to_file_path().ok());
    let guards: Vec<crate::semantic::LocationSource> = serde_json::from_value(
        answer
            .get("location_sources")
            .cloned()
            .unwrap_or_else(|| serde_json::json!([])),
    )?;
    let digest = if original_uri
        .as_ref()
        .is_some_and(|original| original == &path)
    {
        answer["source"]["source_sha256"].as_str()
    } else {
        guards
            .iter()
            .find(|guard| guard.path == path)
            .map(|guard| guard.source_sha256.as_str())
    }
    .ok_or_else(|| {
        Error::Unavailable("native result did not bind this target source to original bytes".into())
    })?;
    let source = SourceRequest::Working { path: path.clone() }
        .acquire()
        .map_err(native)?;
    if source.identity().source_sha256() != digest {
        return Err(Error::Stale(
            "native linked source changed before guarded location conversion".into(),
        ));
    }
    let (start, end) = if let Some(bytes) = bytes {
        bytes
    } else {
        let range = location
            .get("targetSelectionRange")
            .or_else(|| location.get("range"))
            .ok_or_else(|| Error::Unavailable("native target has no addressed range".into()))?;
        let encoding = answer["session"]["position_encoding"]
            .as_str()
            .ok_or_else(|| Error::Unavailable("native answer has no coordinate encoding".into()))?;
        (
            crate::semantic::native_byte_position(source.text(), &range["start"], encoding)?,
            crate::semantic::native_byte_position(source.text(), &range["end"], encoding)?,
        )
    };
    if start > end || source.text().get(start..end).is_none() {
        return Err(Error::Stale(
            "native linked range is outside its original UTF-8 source".into(),
        ));
    }
    let Locator::Semantic {
        project,
        session_invocation,
        ..
    } = seed
    else {
        return Err(Error::Invalid(
            "native linked location needs an original semantic session".into(),
        ));
    };
    let session = session_invocation.as_ref().ok_or_else(|| {
        Error::Unavailable("semantic location has no original session owner".into())
    })?;
    let record = crate::semantic::load(root, &environment())?
        .ok_or_else(|| Error::Unavailable("original semantic session is not selected".into()))?;
    if record.invocation.as_str() != session {
        return Err(Error::Stale(
            "native linked location belongs to a replaced session".into(),
        ));
    }
    let configuration: crate::semantic::Configuration =
        serde_json::from_value(answer["session"]["configuration"].clone())?;
    if configuration != record.configuration {
        return Err(Error::Stale(
            "native linked location has a different selected configuration".into(),
        ));
    }
    let generation = answer["generation"]
        .as_u64()
        .ok_or_else(|| Error::Unavailable("native answer has no generation".into()))?;
    let anchor = crate::source_read::semantic_anchor(source.text(), start..end).unwrap_or(start);
    let line = source.text()[..anchor]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1;
    let column = source.text()[..anchor]
        .rsplit('\n')
        .next()
        .map_or(1, |line| line.chars().count() + 1);
    let line_start = source.text()[..start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1;
    let line_end = source.text()[..end.saturating_sub(usize::from(end > start))]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1;
    Ok((
        Locator::Semantic {
            project: project.clone(),
            path: path.into(),
            line,
            column,
            generation: Some(generation.to_string()),
            source_sha256: Some(digest.into()),
            candidate_fingerprint: None,
            session_invocation: Some(session.clone()),
        },
        MatchRange {
            byte_start: start,
            byte_end: end,
            line_start,
            line_end,
        },
    ))
}
fn definition_locations(answer: &serde_json::Value) -> Vec<serde_json::Value> {
    let locations = answer["sections"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|section| section["category"] == "definition")
        .flat_map(|section| {
            if let Some(items) = section["evidence"].as_array() {
                items
                    .iter()
                    .map(|item| item.get("location").unwrap_or(item).clone())
                    .collect()
            } else if section["evidence"].is_object() {
                vec![section["evidence"].clone()]
            } else {
                Vec::new()
            }
        })
        .collect::<Vec<_>>();
    if !locations.is_empty() {
        return locations;
    }
    answer
        .pointer("/facts/target/location")
        .filter(|location| !location.is_null())
        .cloned()
        .into_iter()
        .collect()
}
/// Resolve only genuine native declaration locations in one selected world, never names.
pub async fn resolve_entity(
    root: &Path,
    locator: &Locator,
    token: &CancellationToken,
) -> Result<ResolvedEntity, Error> {
    let context = ResolutionContext::default();
    let result = resolve_entity_with_context(root, locator, token, &context).await?;
    context.verify_unchanged()?;
    Ok(result)
}
/// Resolve several candidates through one request-local acquired artifact/native context.
pub async fn resolve_entity_with_context(
    root: &Path,
    locator: &Locator,
    token: &CancellationToken,
    context: &ResolutionContext,
) -> Result<ResolvedEntity, Error> {
    let semantic = source_semantic_with_context(root, locator, token, context).await?;
    let answer =
        semantic_operation_cancellable(root, &semantic, crate::semantic::Kind::Explain, token)
            .await?;
    if answer["sections"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|section| {
            section["category"] == "definition"
                && section["omitted"]
                    .as_u64()
                    .is_some_and(|omitted| omitted > 0)
        })
    {
        return Err(Error::Unavailable(
            "native declaration identity is ambiguous beyond the returned target allowance".into(),
        ));
    }
    let mut targets = BTreeMap::new();
    for target in definition_locations(&answer) {
        let (locator, range) = guard_location(root, &semantic, &answer, &target)?;
        let Locator::Semantic { path, .. } = &locator else {
            continue;
        };
        targets.insert((path.clone(), range.byte_start, range.byte_end), locator);
    }
    if targets.len() != 1 {
        return Err(Error::Unavailable(
            "native declaration identity is unavailable or ambiguous in the selected context"
                .into(),
        ));
    }
    let ((declaration, byte_start, byte_end), _target) = targets
        .into_iter()
        .next()
        .ok_or_else(|| Error::Unavailable("native declaration target is unavailable".into()))?;
    let configuration: crate::semantic::Configuration =
        serde_json::from_value(answer["session"]["configuration"].clone())?;
    let generation = answer["generation"]
        .as_u64()
        .ok_or_else(|| Error::Unavailable("native declaration has no generation".into()))?;
    let mut locator = semantic;
    let Locator::Semantic {
        generation: guard,
        session_invocation,
        ..
    } = &mut locator
    else {
        return Err(Error::Invalid(
            "resolved native position lost its session".into(),
        ));
    };
    *guard = Some(generation.to_string());
    let session_invocation = session_invocation.clone().ok_or_else(|| {
        Error::Unavailable("native declaration has no original session identity".into())
    })?;
    Ok(ResolvedEntity {
        identity: EntityIdentity {
            session_invocation,
            configuration: configuration.identity()?,
            generation,
            declaration,
            byte_start,
            byte_end,
        },
        locator,
        native: answer,
    })
}
/// Follow the original native directed relation with explicit byte/context guards.
pub async fn relationship(
    root: &Path,
    seed: &Locator,
    relation: Relationship,
    direction: Direction,
    limit: usize,
    token: &CancellationToken,
) -> Result<RelationEvaluation, Error> {
    let context = ResolutionContext::default();
    let result =
        relationship_with_context(root, seed, relation, direction, limit, token, &context).await?;
    context.verify_unchanged()?;
    Ok(result)
}
/// Traverse using the request's one acquired-API/source verification owner.
pub async fn relationship_with_context(
    root: &Path,
    seed: &Locator,
    relation: Relationship,
    direction: Direction,
    limit: usize,
    token: &CancellationToken,
    context: &ResolutionContext,
) -> Result<RelationEvaluation, Error> {
    use crate::semantic::Kind;
    let semantic = source_semantic_with_context(root, seed, token, context).await?;
    if relation == Relationship::References && direction == Direction::Outgoing {
        return Err(Error::Unavailable("native references enumerate incoming uses of the selected entity; outgoing references are unavailable here, while the separate outgoing definition relation resolves a selected use".into()));
    }
    let kind = match relation {
        Relationship::Definition => Kind::Explain,
        Relationship::References => Kind::Refs,
        Relationship::Calls => Kind::Calls,
        Relationship::Implementations => Kind::Implementations,
    };
    if matches!(
        relation,
        Relationship::Definition | Relationship::Implementations
    ) && direction == Direction::Incoming
    {
        return Err(Error::Unavailable(
            "this native relation exposes outgoing targets only; it is not an inverse source graph"
                .into(),
        ));
    }
    let answer = semantic_query(root, &semantic, kind, limit, token).await?;
    let configuration: crate::semantic::Configuration =
        serde_json::from_value(answer["session"]["configuration"].clone())?;
    let mut locations = Vec::new();
    if relation == Relationship::Definition {
        locations.extend(
            definition_locations(&answer)
                .into_iter()
                .map(|location| (location.clone(), location)),
        );
    } else if relation == Relationship::References && answer["references"].is_array() {
        for reference in answer["references"].as_array().into_iter().flatten() {
            if let Some(location) = reference.get("location") {
                locations.push((location.clone(), reference.clone()));
            }
        }
    } else {
        for section in answer["sections"].as_array().into_iter().flatten() {
            let category = section["category"].as_str().unwrap_or("");
            match relation {
                Relationship::References if category == "references_by_file" => {
                    for file in section["evidence"]["files"]
                        .as_array()
                        .into_iter()
                        .flatten()
                    {
                        for reference in file["references"].as_array().into_iter().flatten() {
                            locations.push((reference.clone(), reference.clone()));
                        }
                    }
                }
                Relationship::Implementations
                    if matches!(category, "implementations" | "native_implementations") =>
                {
                    for target in section["evidence"].as_array().into_iter().flatten() {
                        locations.push((
                            target.get("location").unwrap_or(target).clone(),
                            target.clone(),
                        ));
                    }
                }
                Relationship::Calls
                    if category
                        == if direction == Direction::Incoming {
                            "incoming_calls_one_hop"
                        } else {
                            "outgoing_calls_one_hop"
                        } =>
                {
                    let key = if direction == Direction::Incoming {
                        "from"
                    } else {
                        "to"
                    };
                    for edge in section["evidence"].as_array().into_iter().flatten() {
                        let target = &edge[key];
                        locations.push((serde_json::json!({"uri":target["uri"],"range":target["selectionRange"]}),edge.clone()));
                    }
                }
                _ => {}
            }
        }
    }
    let mut rows = Vec::new();
    let mut diagnostics = Vec::new();
    let mut complete = configuration.backend == crate::semantic::Backend::NativeSnapshot
        && answer["availability"] == "available"
        && !answer["sections"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|section| {
                section["omitted"]
                    .as_u64()
                    .is_some_and(|omitted| omitted > 0)
            });
    let generation = answer["generation"]
        .as_u64()
        .ok_or_else(|| Error::Unavailable("native relation has no generation".into()))?;
    for (location, evidence) in locations {
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }
        match guard_location(root, &semantic, &answer, &location) {
            Ok((locator, range)) => {
                let entity = if relation != Relationship::References {
                    let Locator::Semantic {
                        path,
                        session_invocation: Some(session),
                        ..
                    } = &locator
                    else {
                        return Err(Error::Unavailable(
                            "native related target has no session identity".into(),
                        ));
                    };
                    Some(EntityIdentity {
                        session_invocation: session.clone(),
                        configuration: configuration.identity()?,
                        generation,
                        declaration: path.clone(),
                        byte_start: range.byte_start,
                        byte_end: range.byte_end,
                    })
                } else {
                    None
                };
                rows.push(SearchResult {
                    source: "native_relationship".into(),
                    kind: format!("{relation:?}").to_lowercase(),
                    name: evidence["name"].as_str().map(str::to_owned),
                    range: Some(range),
                    context: serde_json::to_string(&evidence)?,
                    bytes: None,
                    reference: Some(ResultReference::new(locator)),
                    captures: BTreeMap::new(),
                    entity,
                    candidate_reference: None,
                    refinement: None,
                });
            }
            Err(error) => {
                complete = false;
                diagnostics.push(QueryDiagnostic {
                    code: "search.native_location_unavailable".into(),
                    message: error.to_string(),
                    source: Some("native_relationship".into()),
                });
            }
        }
    }
    if !complete {
        diagnostics.push(QueryDiagnostic {code:"search.native_relationship_scope".into(),message:"native relationship output is bounded/selected-context evidence; omissions, unknown preparation or provider coverage do not establish whole-program absence or runtime reach".into(),source:Some("native_relationship".into())});
    }
    let sources = vec![SourceContext {
        id: "native_relationship".into(),
        path: configuration.project.into(),
        origin: Some(SourceKind::Working),
        provider: answer["session"]["backend"]
            .as_str()
            .unwrap_or("native")
            .into(),
        generation: Some(generation.to_string()),
        searched_encoding: None,
        native: Some(NativeContext {
            session_invocation: match &semantic {
                Locator::Semantic {
                    session_invocation: Some(invocation),
                    ..
                } => invocation.clone(),
                _ => {
                    return Err(Error::Unavailable(
                        "native relationship lost its original invocation".into(),
                    ));
                }
            },
            generation,
            analysis: serde_json::from_value(answer["session"].clone())?,
        }),
    }];
    Ok(RelationEvaluation {
        rows,
        sources,
        complete,
        diagnostics,
    })
}
/// A faithful native-owner result, retaining the exact selected context locator.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct SemanticInspection {
    /// Original guarded semantic address, not a display-name identity.
    pub locator: Locator,
    /// Actual native answer including source/generation, sections and checked proposal.
    pub native: serde_json::Value,
    /// Native discovery handles rebound to the consolidated interface.
    pub followups: Vec<SemanticAction>,
}
/// One lazy action keeps its original record identity, generation and exact source.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct SemanticAction {
    /// Inspect candidate/hint or propose assist; no obsolete code-query spelling.
    pub view: String,
    /// Index is useful only with the retained native fingerprint.
    pub select: usize,
    /// The full guarded address for a later invocation.
    pub reference: ResultReference,
}
/// Inspect or preview through the actual selected native operation owner.
pub async fn semantic_view(
    root: &Path,
    locator: &Locator,
    kind: crate::semantic::Kind,
) -> Result<SemanticInspection, Error> {
    let native = semantic_operation(root, locator, kind).await?;
    let mut followups = Vec::new();
    for section in native["sections"].as_array().into_iter().flatten() {
        let view = match section["category"].as_str() {
            Some("completion_candidates") => "candidate",
            Some("selected_inlay_annotations") => "hint",
            Some("assist_candidates") => "assist",
            _ => continue,
        };
        for handle in section["handles"].as_array().into_iter().flatten() {
            let Some(index) = handle["index"]
                .as_u64()
                .and_then(|value| usize::try_from(value).ok())
            else {
                continue;
            };
            let Some(fingerprint) = handle["fingerprint"].as_str() else {
                continue;
            };
            let mut selected = locator.clone();
            if let Locator::Semantic {
                generation,
                source_sha256,
                candidate_fingerprint,
                ..
            } = &mut selected
            {
                *generation = native["generation"].as_u64().map(|value| value.to_string());
                *source_sha256 = native["source"]["source_sha256"]
                    .as_str()
                    .map(str::to_owned);
                *candidate_fingerprint = Some(fingerprint.into());
            }
            followups.push(SemanticAction {
                view: view.into(),
                select: index,
                reference: ResultReference::new(selected),
            });
        }
    }
    Ok(SemanticInspection {
        locator: locator.clone(),
        native,
        followups,
    })
}
/// Inspect an acquired artifact-local declaration without another search/acquisition.
pub fn inspect_api(root: &Path, locator: &Locator) -> Result<SearchResult, Error> {
    let Locator::Api {
        item, public_path, ..
    } = locator
    else {
        return Err(Error::Invalid(
            "expected an acquired API item locator".into(),
        ));
    };
    let artifact = checked_api(root, locator)?;
    let id = item
        .parse::<u32>()
        .map_err(|_| Error::Invalid("invalid artifact-local item identity".into()))?;
    let declaration = artifact
        .document
        .index
        .get(&rustdoc_types::Id(id))
        .ok_or_else(|| Error::Stale("original API artifact has no selected declaration".into()))?;
    Ok(SearchResult {
        source: artifact.sha256,
        kind: "api_declaration".into(),
        name: public_path.clone().or_else(|| declaration.name.clone()),
        range: None,
        context: serde_json::to_string(declaration)?,
        bytes: None,
        reference: Some(ResultReference::new(locator.clone())),
        captures: BTreeMap::new(),
        entity: None,
        candidate_reference: None,
        refinement: None,
    })
}
fn checked_api(root: &Path, locator: &Locator) -> Result<crate::api::Artifact, Error> {
    let Locator::Api {
        artifact,
        artifact_sha256,
        context_key,
        ..
    } = locator
    else {
        return Err(Error::Invalid(
            "expected an acquired API item locator".into(),
        ));
    };
    let artifact = crate::api::retained(&root.join(artifact)).map_err(Error::from)?;
    if artifact_sha256
        .as_ref()
        .is_some_and(|expected| expected != &artifact.sha256)
    {
        return Err(Error::Stale(
            "acquired API artifact no longer matches the selected item reference".into(),
        ));
    }
    let actual_context = artifact.context.key().map_err(Error::from)?;
    if context_key
        .as_ref()
        .is_some_and(|expected| expected != &actual_context)
    {
        return Err(Error::Stale(
            "acquired API context no longer matches the selected item reference".into(),
        ));
    }
    Ok(artifact)
}
fn api_search(
    root: &Path,
    query: &ApiQuery,
    symbol: Option<&SymbolQuery>,
    request: &Request,
    limit: usize,
    emit: &mut (dyn FnMut(Event) -> Result<(), Error> + Send),
) -> Result<Summary, Error> {
    let source = &request.sources;
    if source.kind != SourceKind::Working
        || !source.paths.is_empty()
        || !source.packages.is_empty()
        || !source.types.is_empty()
        || source.tracked.is_some()
        || !source.globs.is_empty()
        || !source.exclude.is_empty()
        || source.include_ignored
        || source.include_hidden
        || source.include_generated
        || source.include_dependencies
        || source.changed
    {
        return Err(Error::Unavailable("API artifact discovery has an acquired documentation universe; repository source selectors require an explicit source/identity refinement rather than being silently applied to historical declarations".into()));
    }
    if request.answer.cursor.is_some() || request.answer.order == ResultOrder::Stable {
        return Err(Error::Unavailable("use the acquired API inspection owner's guarded native page continuation; common source-tree continuations do not address artifact-local declarations".into()));
    }
    if request.answer.count_unit != CountUnit::Matches {
        return Err(Error::Unavailable("API candidate search counts declared public-path candidates; file/line/entity counts require an explicit acquired-artifact population query".into()));
    }
    let artifact = if let Some(path) = &query.artifact {
        crate::api::retained(&root.join(path)).map_err(Error::from)?
    } else if let Some(dependency) = &query.dependency {
        let env = environment();
        let snapshot = crate::dependency::capture(root, &env).map_err(native)?;
        let store = crate::paths::CheckoutState::resolve(root, &env)
            .map_err(native)?
            .root;
        crate::api::locate(root,&store,&snapshot,dependency,None,&env).map_err(Error::from)?.artifact
            .ok_or_else(||Error::Unavailable("the selected exact dependency context has no acquired API artifact; acquire explicitly".into()))?
    } else {
        return Err(Error::Unavailable(
            "select an acquired API artifact or exact dependency; acquisition remains explicit"
                .into(),
        ));
    };
    let selected = crate::api::query::Query {
        search: Some(query.query.clone()),
        input_shape: query
            .input
            .as_deref()
            .map(crate::api::query::TypePattern::parse)
            .transpose()
            .map_err(Error::from)?,
        output_shape: query
            .output
            .as_deref()
            .map(crate::api::query::TypePattern::parse)
            .transpose()
            .map_err(Error::from)?,
        declared_bound: query.bound.clone(),
        ..crate::api::query::Query::default()
    };
    let page = crate::api::query::inspect(
        &artifact,
        &selected,
        if symbol.is_some() {
            usize::MAX
        } else {
            limit.max(1)
        },
        None,
    )
    .map_err(Error::from)?;
    emit(Event::Source(SourceContext {
        id: "api".into(),
        path: artifact.path.clone().into(),
        origin: None,
        provider: format!(
            "rustdoc-json/{};package={};context={}",
            page.rustdoc_format, page.package_id, page.context_key
        ),
        generation: Some(artifact.sha256.clone()),
        searched_encoding: None,
        native: None,
    }))?;
    let mut summary = Summary::default();
    let root_name = artifact
        .document
        .index
        .get(&artifact.document.root)
        .and_then(|item| item.name.as_deref());
    let candidates = page
        .candidates
        .into_iter()
        .filter(|candidate| {
            let Some(symbol) = symbol else {
                return true;
            };
            let simple = artifact
                .document
                .index
                .get(&rustdoc_types::Id(candidate.item_id))
                .and_then(|item| item.name.as_deref())
                .unwrap_or("");
            let relative = root_name
                .and_then(|name| candidate.public_path.strip_prefix(&format!("{name}::")))
                .unwrap_or(&candidate.public_path);
            if !name_matches(relative, simple, &symbol.name, symbol.matching)
                && !name_matches(
                    &candidate.public_path,
                    simple,
                    &symbol.name,
                    symbol.matching,
                )
            {
                return false;
            }
            if !symbol.kinds.is_empty()
                && !symbol.kinds.iter().any(|kind| {
                    native_symbol_kind(&format!("{:?}", candidate.kind)).eq_ignore_ascii_case(kind)
                })
            {
                return false;
            }
            if symbol.module.as_ref().is_some_and(|module| {
                let parent = relative.rsplit_once("::").map_or("", |(parent, _)| parent);
                parent != module && !parent.starts_with(&format!("{module}::"))
            }) {
                return false;
            }
            true
        })
        .collect::<Vec<_>>();
    summary.matches = if symbol.is_some() {
        candidates.len() as u64
    } else {
        page.matched_candidates as u64
    };
    let limited = candidates.len() > limit;
    for candidate in candidates.into_iter().take(limit) {
        if request.answer.kind != AnswerKind::Count {
            emit(Event::Result(SearchResult {
                source: "api".into(),
                kind: format!("{:?}", candidate.kind),
                name: Some(candidate.public_path.clone()),
                range: None,
                context: candidate.signature,
                bytes: None,
                reference: Some(ResultReference::new(Locator::Api {
                    artifact: artifact.path.clone(),
                    item: candidate.item_id.to_string(),
                    artifact_sha256: Some(artifact.sha256.clone()),
                    context_key: Some(page.context_key.clone()),
                    public_path: Some(candidate.public_path.clone()),
                })),
                captures: BTreeMap::new(),
                entity: None,
                candidate_reference: None,
                refinement: None,
            }))?;
            summary.emitted += 1;
        }
    }
    summary.count = Count::Exact(summary.matches);
    summary.complete = page.omitted_candidates == 0 && !limited;
    if !summary.complete {
        summary.stop = StopReason::Limit;
    }
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_name_policies_and_written_signature_do_not_match_documentation() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("control.rs");
        std::fs::write(
            &path,
            "/// fn misleading() -> u32\n#[inline]\npub fn actual(x: bool) -> bool {x}\n",
        )
        .unwrap();
        let input = providers::acquire(root.path(), &SourceRequest::Working { path }).unwrap();
        let mut context = EvaluationContext::new(Arc::new(CompiledProgram::default()));
        let mut query = SymbolQuery {
            name: "actual".into(),
            matching: NameMatch::Exact,
            signature: Some("u32".into()),
            ..Default::default()
        };
        let absent = evaluate(
            root.path(),
            &input,
            &Predicate::Symbol(query.clone()),
            &CancellationToken::new(),
            None,
            &mut context,
        )
        .unwrap();
        assert!(absent.rows.is_empty() && absent.complete);
        query.signature = Some("bool".into());
        assert_eq!(
            evaluate(
                root.path(),
                &input,
                &Predicate::Symbol(query.clone()),
                &CancellationToken::new(),
                None,
                &mut context
            )
            .unwrap()
            .rows
            .len(),
            1
        );
        query.name = "act".into();
        query.matching = NameMatch::Prefix;
        assert_eq!(
            evaluate(
                root.path(),
                &input,
                &Predicate::Symbol(query.clone()),
                &CancellationToken::new(),
                None,
                &mut context
            )
            .unwrap()
            .rows
            .len(),
            1
        );
        query.matching = NameMatch::Exact;
        assert!(
            evaluate(
                root.path(),
                &input,
                &Predicate::Symbol(query.clone()),
                &CancellationToken::new(),
                None,
                &mut context
            )
            .unwrap()
            .rows
            .is_empty()
        );
        query.matching = NameMatch::Fuzzy;
        assert!(matches!(
            evaluate(
                root.path(),
                &input,
                &Predicate::Symbol(query),
                &CancellationToken::new(),
                None,
                &mut context
            ),
            Err(Error::Unavailable(_))
        ));
    }
    #[test]
    fn native_file_context_shares_parse_and_request_selector_without_capture_leakage() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("control.rs");
        std::fs::write(&path, "fn control(){ foo(x); bar(y); }\n").unwrap();
        let input = providers::acquire(root.path(), &SourceRequest::Working { path }).unwrap();
        let program = Arc::new(CompiledProgram::default());
        let first = SyntaxQuery {
            pattern: Some("foo($A)".into()),
            ..Default::default()
        };
        assert!(Arc::ptr_eq(
            &program.syntax(&first).unwrap(),
            &program.syntax(&first).unwrap()
        ));
        let mut context = EvaluationContext::new(program);
        let a = evaluate(
            root.path(),
            &input,
            &Predicate::Syntax(first),
            &CancellationToken::new(),
            None,
            &mut context,
        )
        .unwrap();
        let parsed: *const crate::source_query::ParsedSyntax = context.parsed.as_ref().unwrap();
        assert_eq!(a.rows.len(), 1);
        assert!(a.rows[0].captures.contains_key("A"));
        let b = evaluate(
            root.path(),
            &input,
            &Predicate::Syntax(SyntaxQuery {
                pattern: Some("bar($B)".into()),
                ..Default::default()
            }),
            &CancellationToken::new(),
            None,
            &mut context,
        )
        .unwrap();
        assert!(std::ptr::eq(parsed, context.parsed.as_ref().unwrap()));
        assert!(!b.rows[0].captures.contains_key("A"));
        assert!(b.rows[0].captures.contains_key("B"));
        let tree = evaluate(
            root.path(),
            &input,
            &Predicate::Syntax(SyntaxQuery {
                tree_sitter: Some("(call_expression function: (identifier) @name)".into()),
                ..Default::default()
            }),
            &CancellationToken::new(),
            None,
            &mut context,
        )
        .unwrap();
        assert_eq!(tree.rows.len(), 2);
        assert!(
            std::ptr::eq(parsed, context.parsed.as_ref().unwrap()),
            "AST and extraction clauses must share their compatible native tree"
        );
    }
}
