// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit syntax-only structural queries and immutable replacement previews.
use crate::{
    envelope::{Completeness, Envelope},
    source_command::batch::{CpuTask, spawn_cpu},
    source_proposal::{ChangeProposal, EditCandidate, ProducerContext, ProposalError},
    source_read::{
        RustEdition, SourceIdentity, SourceLocator, SourceRange, SourceReadError, SourceRequest,
        SourceText,
    },
};
use ast_grep_core::{
    AstGrep, Language, MatchStrictness, Pattern, PatternError,
    matcher::{MatcherExt as _, PatternBuilder},
    meta_var::MetaVariable,
    replacer::TemplateFix,
    tree_sitter::{LanguageExt, StrDoc, TSLanguage},
};
use rayon::prelude::*;
use schemars::JsonSchema;
use serde::Serialize;
use std::borrow::Cow;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug)]
struct RustGrammar;
impl Language for RustGrammar {
    fn kind_to_id(&self, kind: &str) -> u16 {
        self.get_ts_language().id_for_node_kind(kind, true)
    }
    fn field_to_id(&self, field: &str) -> Option<u16> {
        self.get_ts_language()
            .field_id_for_name(field)
            .map(|id| id.get())
    }
    fn expando_char(&self) -> char {
        'µ'
    }
    fn pre_process_pattern<'a>(&self, query: &'a str) -> Cow<'a, str> {
        // Rust identifiers cannot start with $. This language binding preserves literal
        // dollars while adapting ast-grep's named/anonymous metavariable sigils.
        let mut characters = query.chars().peekable();
        let mut text = String::with_capacity(query.len());
        while let Some(character) = characters.next() {
            if character != '$' {
                text.push(character);
                continue;
            }
            let mut count = 1;
            while characters.peek() == Some(&'$') {
                characters.next();
                count += 1;
            }
            let sigil = if count == 3
                || characters
                    .peek()
                    .is_some_and(|next| next.is_ascii_uppercase() || *next == '_')
            {
                'µ'
            } else {
                '$'
            };
            text.extend(std::iter::repeat_n(sigil, count));
        }
        Cow::Owned(text)
    }
    fn build_pattern(&self, builder: &PatternBuilder<'_>) -> Result<Pattern, PatternError> {
        builder.build(|source| StrDoc::try_new(source, self.clone()))
    }
}
impl LanguageExt for RustGrammar {
    fn get_ts_language(&self) -> TSLanguage {
        tree_sitter_rust::LANGUAGE.into()
    }
}

/// Native matching policy; the selected policy is always reported with the result.
#[derive(Clone, Copy, Debug, clap::ValueEnum, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Strictness {
    /// Match all concrete syntax nodes.
    Cst,
    /// Native smart policy, retaining significant written syntax.
    Smart,
    /// Match named AST nodes.
    Ast,
    /// Ignore comments and unnamed punctuation.
    Relaxed,
    /// Compare structural signatures without terminal text.
    Signature,
    /// Native textual template matching policy.
    Template,
}
impl Strictness {
    fn native(self) -> MatchStrictness {
        match self {
            Self::Cst => MatchStrictness::Cst,
            Self::Smart => MatchStrictness::Smart,
            Self::Ast => MatchStrictness::Ast,
            Self::Relaxed => MatchStrictness::Relaxed,
            Self::Signature => MatchStrictness::Signature,
            Self::Template => MatchStrictness::Template,
        }
    }
}
/// Explicit syntax query; contexts are native structural patterns, never resolved symbols.
#[derive(Clone, Debug)]
pub struct Query {
    /// Positive code-shaped pattern.
    pub pattern: String,
    /// Native matching strictness.
    pub strictness: Strictness,
    /// Every pattern must match an ancestor of a candidate.
    pub inside: Vec<String>,
    /// Every pattern must match a proper descendant of a candidate.
    pub has: Vec<String>,
    /// Patterns that must not match the candidate itself.
    pub excluded: Vec<String>,
    /// Retain only these named/sequence captures; empty selects every capture.
    pub captures: Vec<String>,
    /// Explicit ra_ap_syntax diagnostic edition; tree-sitter has its own grammar policy.
    pub edition: RustEdition,
    /// Global match-record allowance, including nested matches.
    pub max_records: Option<usize>,
}
/// One exact capture; sequence captures retain each native captured node separately.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Capture {
    /// Metavariable name without sigils.
    pub name: String,
    /// True for a $$$ sequence capture.
    pub sequence: bool,
    /// Original nodes, including exact source text and coordinates.
    pub nodes: Vec<CapturedNode>,
}
/// Captured original source, without token regeneration.
#[derive(Debug, Serialize, JsonSchema)]
pub struct CapturedNode {
    /// Exact original source coordinates.
    pub locator: SourceLocator,
    /// Exact matched bytes, retaining trivia inside the node.
    pub text: String,
}
/// Syntax candidate; equal spellings can refer to unrelated bindings.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SyntaxMatch {
    /// Stable index in the returned match/candidate collection.
    pub index: usize,
    /// Exact original syntax location.
    pub locator: SourceLocator,
    /// Native tree-sitter node kind.
    pub kind: String,
    /// Original matched text.
    pub text: String,
    /// Explicit selected captures, sorted by name.
    pub captures: Vec<Capture>,
}
/// Examined source and recovery regions remain visible even when it had no matches.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SourceCoverage {
    /// Exact original source identity.
    pub source: SourceIdentity,
    /// Entire examined source extent, independent of returned match allowance.
    pub examined_range: SourceRange,
    /// Native tree-sitter error or missing-node regions.
    pub recovery_regions: Vec<SourceLocator>,
    /// Explicit-edition Rust parser diagnostics, independent of tree-sitter matching.
    pub rust_parse_issues: Vec<crate::source_read::ParseIssue>,
    /// Total qualifying matches, including omitted/nested matches.
    pub total_matches: usize,
}
/// One completed syntax query, not name resolution or compiler acceptance.
#[derive(Debug, Serialize, JsonSchema)]
pub struct QueryReport {
    /// Exact backend pins.
    pub backend: String,
    /// Actual native grammar ABI accepted by the runtime during parsing.
    pub grammar_abi: usize,
    /// Selected native strictness.
    pub strictness: Strictness,
    /// Explicit diagnostic edition; grammar matching does not claim edition validation.
    pub rust_edition: RustEdition,
    /// Examined source set, with each independently acquired generation.
    pub sources: Vec<SourceCoverage>,
    /// Returned matches, including nested/overlapping candidates.
    pub matches: Vec<SyntaxMatch>,
    /// Total qualifying matches across examined sources.
    pub total_matches: usize,
    /// Matches omitted by the producer's explicit global record allowance.
    pub omitted_matches: usize,
    /// Cancellation observed by the actual CPU owner.
    pub cancellation_requested: bool,
    /// Every submitted source job actually returned.
    pub cpu_work_completed: bool,
}
/// An immutable preview shares the same proposal contract as semantic producers.
#[derive(Debug, Serialize, JsonSchema)]
pub struct PreviewReport {
    /// Matches, captures, examined context and omitted scope.
    pub query: QueryReport,
    /// Checked candidates and explicit nonoverlapping selected edit set.
    pub proposal: ChangeProposal,
}
/// Native pattern/acquisition/proposal refusal, before emitting a successful result.
#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    /// Mandatory provenance retains its original serialization failure.
    #[error("syntax provenance projection: {0}")]
    Projection(#[from] serde_json::Error),
    /// Exact source owner failure.
    #[error(transparent)]
    Source(#[from] SourceReadError),
    /// Native fallible pattern-construction failure.
    #[error(transparent)]
    Pattern(#[from] PatternError),
    /// Grammar/runtime refusal is explicit rather than a missing match.
    #[error("syntax backend refused source: {0}")]
    Parse(String),
    /// An explicitly requested capture or template variable is not defined.
    #[error("undefined syntax capture: {0}")]
    UndefinedCapture(String),
    /// Pattern recovery is unsuitable for a claimed structural query.
    #[error("syntax pattern contains a native parse error")]
    InvalidPattern,
    /// Source-bound proposal consistency refusal.
    #[error(transparent)]
    Proposal(#[from] ProposalError),
    /// An unavailable mode must not silently become syntax matching.
    #[error("{0}")]
    Unavailable(String),
}
pse_diagnostics::impl_diagnostic! {QueryError,
    code(_this){Some(pse_diagnostics::DiagnosticCode::HostOwnership)},
    forward(_this){None},help(_this){None},related(_this){None},source(_this){None}
}

fn coordinates(source: &str, range: std::ops::Range<usize>) -> SourceRange {
    crate::source_read::location(&crate::source_read::line_starts(source), range)
}
fn locator(source: &SourceText, range: std::ops::Range<usize>) -> SourceLocator {
    SourceLocator {
        source: source.identity().clone(),
        range: coordinates(source.text(), range),
    }
}
fn pattern(text: &str, strictness: Strictness) -> Result<Pattern, QueryError> {
    let pattern = Pattern::try_new(text, RustGrammar)?.with_strictness(strictness.native());
    if pattern.has_error() {
        return Err(QueryError::InvalidPattern);
    }
    Ok(pattern)
}

struct FileResult {
    coverage: SourceCoverage,
    matches: Vec<SyntaxMatch>,
    edits: Vec<EditCandidate>,
}
fn analyze(
    source: &SourceText,
    query: &Query,
    replacement: Option<&str>,
    limit: usize,
) -> Result<FileResult, QueryError> {
    let matcher = pattern(&query.pattern, query.strictness)?;
    for name in &query.captures {
        if !matcher.defined_vars().contains(name.as_str()) {
            return Err(QueryError::UndefinedCapture(name.clone()));
        }
    }
    let inside = query
        .inside
        .iter()
        .map(|text| pattern(text, query.strictness))
        .collect::<Result<Vec<_>, _>>()?;
    let has = query
        .has
        .iter()
        .map(|text| pattern(text, query.strictness))
        .collect::<Result<Vec<_>, _>>()?;
    let excluded = query
        .excluded
        .iter()
        .map(|text| pattern(text, query.strictness))
        .collect::<Result<Vec<_>, _>>()?;
    let template = replacement
        .map(|text| {
            TemplateFix::try_new(text, &RustGrammar)
                .map_err(|error| QueryError::Parse(error.to_string()))
        })
        .transpose()?;
    if let Some(template) = &template {
        for variable in template.used_vars() {
            if !matcher.defined_vars().contains(variable) {
                return Err(QueryError::UndefinedCapture(variable.into()));
            }
        }
    }
    let document = StrDoc::try_new(source.text(), RustGrammar).map_err(QueryError::Parse)?;
    let tree = AstGrep::doc(document);
    let root = tree.root();
    let recovery_regions = root
        .dfs()
        .filter(|node| node.is_error() || node.is_missing())
        .map(|node| locator(source, node.range()))
        .collect();
    let mut matches = Vec::new();
    let mut edits = Vec::new();
    let mut total = 0;
    // find_all traverses every native node; replace_all intentionally skips nested matches.
    for matched in root.find_all(&matcher) {
        if !inside.iter().all(|pattern| {
            matched
                .ancestors()
                .any(|node| pattern.match_node(node).is_some())
        }) || !has.iter().all(|pattern| {
            matched
                .dfs()
                .skip(1)
                .any(|node| pattern.match_node(node).is_some())
        }) || excluded
            .iter()
            .any(|pattern| pattern.match_node(matched.get_node().clone()).is_some())
        {
            continue;
        }
        total += 1;
        if matches.len() >= limit {
            continue;
        }
        let mut captures = Vec::new();
        for variable in matched.get_env().get_matched_variables() {
            let (name, sequence) = match variable {
                MetaVariable::Capture(name, _) => (name, false),
                MetaVariable::MultiCapture(name) => (name, true),
                _ => continue,
            };
            if !query.captures.is_empty() && !query.captures.contains(&name) {
                continue;
            }
            let nodes = if sequence {
                matched.get_env().get_multiple_matches(&name)
            } else {
                matched
                    .get_env()
                    .get_match(&name)
                    .cloned()
                    .into_iter()
                    .collect()
            };
            captures.push(Capture {
                name,
                sequence,
                nodes: nodes
                    .into_iter()
                    .map(|node| CapturedNode {
                        locator: locator(source, node.range()),
                        text: node.text().into_owned(),
                    })
                    .collect(),
            });
        }
        captures.sort_by(|left, right| left.name.cmp(&right.name));
        if let Some(template) = &template {
            let edit = matched.replace_by(template);
            let text = String::from_utf8(edit.inserted_text)
                .map_err(|error| QueryError::Parse(error.to_string()))?;
            edits.push(EditCandidate::new(
                source.identity().clone(),
                edit.position..edit.position + edit.deleted_length,
                text,
            ));
        }
        matches.push(SyntaxMatch {
            index: matches.len(),
            locator: locator(source, matched.range()),
            kind: matched.kind().into_owned(),
            text: matched.text().into_owned(),
            captures,
        });
    }
    let diagnostics = crate::source_read::rust_bounded(source.text(), query.edition, 0, 0)
        .0
        .parse_issues;
    Ok(FileResult {
        coverage: SourceCoverage {
            source: source.identity().clone(),
            examined_range: coordinates(source.text(), 0..source.text().len()),
            recovery_regions,
            rust_parse_issues: diagnostics,
            total_matches: total,
        },
        matches,
        edits,
    })
}

fn execute(
    requests: Vec<SourceRequest>,
    query: Query,
    replacement: Option<String>,
    selected: Vec<usize>,
    token: CancellationToken,
) -> Result<(QueryReport, Option<ChangeProposal>), QueryError> {
    let inputs = requests
        .into_par_iter()
        .map(|request| request.acquire())
        .collect::<Result<Vec<_>, _>>()?;
    // Count without retaining candidate records, then give each worker its deterministic quota.
    let counts = inputs
        .par_iter()
        .map(|source| {
            analyze(source, &query, replacement.as_deref(), 0)
                .map(|result| result.coverage.total_matches)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut remaining = query.max_records.unwrap_or(usize::MAX);
    let quotas: Vec<_> = counts
        .iter()
        .map(|count| {
            let quota = (*count).min(remaining);
            remaining -= quota;
            quota
        })
        .collect();
    let results = inputs
        .par_iter()
        .zip(quotas)
        .map(|(source, quota)| analyze(source, &query, replacement.as_deref(), quota))
        .collect::<Result<Vec<_>, _>>()?;
    let mut sources = Vec::new();
    let mut matches = Vec::new();
    let mut edits = Vec::new();
    let mut originals = Vec::new();
    for (original, mut result) in inputs.into_iter().zip(results) {
        for matched in &mut result.matches {
            matched.index += matches.len();
        }
        if !result.edits.is_empty() {
            originals.push(original);
        }
        sources.push(result.coverage);
        matches.extend(result.matches);
        edits.extend(result.edits);
    }
    let total_matches = counts.iter().sum();
    let omitted_matches = total_matches - matches.len();
    let proposal = if replacement.is_some() {
        Some(ChangeProposal::new(
            ProducerContext::Syntax {
                backend: "ast-grep-core/0.51.0;tree-sitter-rust/0.24.2".into(),
                configuration: format!(
                    "strictness={:?};diagnostic_edition={:?};syntax-only;inside={:?};has={:?};excluded={:?}",
                    query.strictness, query.edition, query.inside, query.has, query.excluded
                ),
            },
            originals,
            edits,
            selected,
            vec![format!(
                "syntax candidates only; {omitted_matches} matches omitted; no compilation, equivalence, file application or Git publication established"
            )],
            vec![],
        )?)
    } else {
        None
    };
    Ok((
        QueryReport {
            backend: "ast-grep-core/0.51.0;tree-sitter-rust/0.24.2;tree-sitter/0.27.0".into(),
            grammar_abi: RustGrammar.get_ts_language().abi_version(),
            strictness: query.strictness,
            rust_edition: query.edition,
            sources,
            matches,
            total_matches,
            omitted_matches,
            cancellation_requested: token.is_cancelled(),
            cpu_work_completed: true,
        },
        proposal,
    ))
}
fn envelope<T>(command: &str, payload: T, partial: bool) -> Envelope<T> {
    let mut response = Envelope::response(
        command,
        "written Rust syntax candidates; no resolved-symbol or application authority",
        payload,
    );
    response.observation.scope = "explicit_source_set_syntax_only".into();
    response.observation.limitations.push("syntax candidates do not establish binding identity, active cfg, macro expansion, type correctness, equivalence or compiler acceptance; sources are independent snapshots".into());
    if partial {
        response.observation.completeness = Completeness::Partial;
        response.observation.required = false;
    }
    response
}
/// Query on the existing shared pool; retained completion survives a cancelled settle awaiter.
pub fn start_query(
    requests: Vec<SourceRequest>,
    query: Query,
    cancellation: CancellationToken,
) -> CpuTask<Result<Envelope<QueryReport>, QueryError>> {
    spawn_cpu(cancellation, move |token| {
        let (report, _) = execute(requests, query, None, vec![], token)?;
        let partial = report.omitted_matches > 0
            || report.sources.iter().any(|source| {
                !source.recovery_regions.is_empty() || !source.rust_parse_issues.is_empty()
            });
        let providers = providers(&report)?;
        let mut response = envelope("code query", report, partial);
        response.providers = providers;
        Ok(response)
    })
}
/// Construct capture-preserving replacements without changing any source or Git index.
pub fn start_preview(
    requests: Vec<SourceRequest>,
    query: Query,
    replacement: String,
    selected: Vec<usize>,
    cancellation: CancellationToken,
) -> CpuTask<Result<Envelope<PreviewReport>, QueryError>> {
    spawn_cpu(cancellation, move |token| {
        let (report, proposal) = execute(requests, query, Some(replacement), selected, token)?;
        let partial = report.omitted_matches > 0
            || report.sources.iter().any(|source| {
                !source.recovery_regions.is_empty() || !source.rust_parse_issues.is_empty()
            });
        let providers = providers(&report)?;
        let mut response = envelope(
            "code preview",
            PreviewReport {
                query: report,
                proposal: proposal.ok_or_else(|| {
                    QueryError::Unavailable(
                        "preview did not construct its source-bound proposal".into(),
                    )
                })?,
            },
            partial,
        );
        response.providers = providers;
        Ok(response)
    })
}
fn providers(
    report: &QueryReport,
) -> Result<Vec<crate::envelope::ProviderContext>, serde_json::Error> {
    report
        .sources
        .iter()
        .map(|source| {
            crate::source_command::provider_context(
                &source.source,
                &format!(
                    "{};strictness={:?};diagnostic_edition={:?}",
                    report.backend, report.strictness, report.rust_edition
                ),
                std::slice::from_ref(&source.examined_range),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests;
