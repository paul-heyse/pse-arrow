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
    replacer::TemplateFix,
    tree_sitter::{LanguageExt, StrDoc, TSLanguage},
};
use rayon::prelude::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use tokio_util::sync::CancellationToken;
mod assertions;
mod grammar;
mod native;
pub use assertions::{
    AssertionObservation, BindingObservation, CallObservation, ValidationStructure,
    validation_structure,
};
pub use grammar::{GrammarLanguage, GrammarReport, grammar};
pub use native::{
    CompiledSyntax, NativeMatches, ParsedSyntax, SelectionDemand, native_rule, tree_sitter_query,
};

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
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
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
    /// Same-source native containment/call evidence; no binding or effective-cfg claim.
    pub relationship: Option<SourceRelationship>,
}
/// Original authored context for one structural witness, excluding nested declaration ownership.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct SourceRelationship {
    /// Closest containing declaration from the same immutable source.
    pub enclosing: Option<SourceLocator>,
    /// Native method receiver, if the selected node is a method call.
    pub receiver: Option<CapturedNode>,
    /// Native positional arguments in authored order.
    pub arguments: Vec<CapturedNode>,
    /// Authored docs/attributes owned by precisely this declaration, never nested ones.
    pub attributes: Vec<crate::source_read::SourceFact>,
}
fn explain(source: &SourceText, matches: &mut [SyntaxMatch]) {
    use ra_ap_syntax::{AstNode as _, ast, ast::HasArgList as _};
    if matches.is_empty() {
        return;
    }
    let (outline, syntax) = crate::source_read::rust_with_syntax(source.text(), RustEdition::E2024);
    let owners: Vec<_> = matches
        .iter()
        .filter_map(|matched| {
            let range = &matched.locator.range;
            outline
                .entries
                .iter()
                .filter(|entry| {
                    entry.range.byte_start <= range.byte_start
                        && entry.range.byte_end >= range.byte_end
                })
                .min_by_key(|entry| entry.range.byte_end - entry.range.byte_start)
                .map(|entry| entry.range.clone())
        })
        .collect();
    let facts = crate::source_read::selected_facts(
        &syntax,
        &crate::source_read::line_starts(source.text()),
        crate::source_read::SourceDetail::Facts,
        &crate::source_read::FactSelection::Attributes { ranges: &owners },
    );
    let capture = |node: ra_ap_syntax::SyntaxNode| {
        let range = node.text_range();
        CapturedNode {
            locator: locator(source, usize::from(range.start())..usize::from(range.end())),
            text: node.text().to_string(),
        }
    };
    for matched in matches {
        let range = &matched.locator.range;
        let enclosing = outline
            .entries
            .iter()
            .filter(|entry| {
                entry.range.byte_start <= range.byte_start && entry.range.byte_end >= range.byte_end
            })
            .min_by_key(|entry| entry.range.byte_end - entry.range.byte_start);
        let mut receiver = None;
        let mut arguments = Vec::new();
        if let Some(node) = syntax.descendants().find(|node| {
            usize::from(node.text_range().start()) == range.byte_start
                && usize::from(node.text_range().end()) == range.byte_end
        }) {
            let args = if let Some(method) = ast::MethodCallExpr::cast(node.clone()) {
                receiver = method.receiver().map(|node| capture(node.syntax().clone()));
                method.arg_list()
            } else {
                ast::CallExpr::cast(node).and_then(|call| call.arg_list())
            };
            if let Some(args) = args {
                arguments = args
                    .args()
                    .map(|node| capture(node.syntax().clone()))
                    .collect();
            }
        }
        matched.relationship = Some(SourceRelationship {
            enclosing: enclosing.map(|entry| SourceLocator {
                source: source.identity().clone(),
                range: entry.range.clone(),
            }),
            receiver,
            arguments,
            attributes: facts
                .iter()
                .filter(|fact| {
                    matches!(fact.kind.as_str(), "attribute" | "documentation")
                        && enclosing.is_some_and(|entry| fact.owner.as_ref() == Some(&entry.range))
                })
                .cloned()
                .collect(),
        });
    }
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
    /// Qualifying witnesses observed, including omitted/nested matches; exact only if complete.
    pub total_matches: usize,
    /// False when parse/selection cancellation prevented exact totals.
    pub complete: bool,
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
    /// Qualifying witnesses across examined sources; exact when every source completed.
    pub total_matches: usize,
    /// Observed witnesses omitted by the producer's allowance, excluding unexamined scope.
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
    /// Native rule compilation retains its complete original cause chain.
    #[error("native structural rule: {0}")]
    Rule(#[source] Box<ast_grep_config::RuleCoreError>),
    /// Grammar/runtime refusal is explicit rather than a missing match.
    #[error("syntax backend refused source: {0}")]
    Parse(String),
    /// Native query construction retains the original kind and query-byte coordinates.
    #[error("Tree-sitter query: {0}")]
    TreeSitter(#[from] TreeSitterQueryError),
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

/// Owned native query-construction failure; coordinates are zero-based query bytes.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, thiserror::Error)]
#[error("{kind} at row {row}, byte column {column} (offset {offset}): {message}")]
pub struct TreeSitterQueryError {
    /// Native `QueryErrorKind` spelling.
    pub kind: String,
    /// Zero-based query row, independent from source locations.
    pub row: usize,
    /// Zero-based UTF-8 byte column in the query.
    pub column: usize,
    /// UTF-8 byte offset in the query.
    pub offset: usize,
    /// Original native explanation.
    pub message: String,
}
impl From<tree_sitter::QueryError> for TreeSitterQueryError {
    fn from(error: tree_sitter::QueryError) -> Self {
        Self {
            kind: format!("{:?}", error.kind),
            row: error.row,
            column: error.column,
            offset: error.offset,
            message: error.message,
        }
    }
}
impl TreeSitterQueryError {
    /// Use the existing diagnostic projection's parser context, without a second taxonomy.
    pub fn record(&self) -> crate::envelope::ErrorRecord {
        let policy = crate::envelope::RedactionPolicy::from_environment();
        let mut record = crate::envelope::ErrorRecord::from_diagnostic(self, None, &policy);
        record.parser_context = [
            ("error_kind".into(), self.kind.clone()),
            ("row".into(), self.row.to_string()),
            ("column".into(), self.column.to_string()),
            ("offset".into(), self.offset.to_string()),
            ("coordinate_encoding".into(), "zero_based_utf8_bytes".into()),
        ]
        .into();
        record
    }
}
impl miette::Diagnostic for TreeSitterQueryError {}
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
struct PreparedQuery {
    matcher: ast_grep_config::RuleCore,
    template: Option<TemplateFix>,
}
impl PreparedQuery {
    fn new(query: &Query, replacement: Option<&str>) -> Result<Self, QueryError> {
        let code = |text: &str| -> Result<serde_json::Value, QueryError> {
            // Keep the native malformed-pattern refusal, then let the rule family
            // own relation/capture behavior for the whole request.
            pattern(text, query.strictness)?;
            Ok(serde_json::json!({"pattern":{"context":text,"strictness":query.strictness}}))
        };
        let mut clauses = vec![code(&query.pattern)?];
        for text in &query.inside {
            let mut relation = code(text)?;
            relation["stopBy"] = serde_json::json!("end");
            clauses.push(serde_json::json!({"inside":relation}));
        }
        for text in &query.has {
            let mut relation = code(text)?;
            relation["stopBy"] = serde_json::json!("end");
            clauses.push(serde_json::json!({"has":relation}));
        }
        for text in &query.excluded {
            clauses.push(serde_json::json!({"not":code(text)?}));
        }
        let rule: ast_grep_config::SerializableRuleCore =
            serde_json::from_value(serde_json::json!({"rule":{"all":clauses}}))?;
        let matcher = rule
            .get_matcher(ast_grep_config::DeserializeEnv::new(RustGrammar))
            .map_err(|error| QueryError::Rule(Box::new(error)))?;
        for name in &query.captures {
            if !matcher.defined_vars().contains(name.as_str()) {
                return Err(QueryError::UndefinedCapture(name.clone()));
            }
        }
        let template = replacement
            .map(|text| {
                TemplateFix::try_new(text, &RustGrammar)
                    .map_err(|error| QueryError::Parse(error.to_string()))
            })
            .transpose()?;
        if let Some(template) = &template {
            for name in template.used_vars() {
                if !matcher.defined_vars().contains(name) {
                    return Err(QueryError::UndefinedCapture(name.into()));
                }
            }
        }
        Ok(Self { matcher, template })
    }
}
fn analyze(
    source: &SourceText,
    query: &Query,
    prepared: &PreparedQuery,
    limit: usize,
    token: &CancellationToken,
) -> Result<FileResult, QueryError> {
    let parsed = ParsedSyntax::cancellable(source, token)?;
    let selected = parsed
        .as_ref()
        .map(|parsed| {
            native::select_rule(
                source,
                parsed,
                &prepared.matcher,
                native::RuleSelection {
                    demand: SelectionDemand::ExactTotal { retain: limit },
                    captures: &query.captures,
                    template: prepared.template.as_ref(),
                    containing: None,
                },
                token,
            )
        })
        .transpose()?;
    let recovery_regions = parsed
        .as_ref()
        .map_or_else(Vec::new, |parsed| parsed.recovery_regions(source));
    let diagnostics = if token.is_cancelled() {
        Vec::new()
    } else {
        crate::source_read::rust_parse_issues(source.text(), query.edition)
    };
    let complete = selected.as_ref().is_some_and(|selected| selected.complete);
    let (matches, edits, total_matches) = selected.map_or_else(
        || (Vec::new(), Vec::new(), 0),
        |selected| (selected.matches, selected.edits, selected.total_matches),
    );
    Ok(FileResult {
        coverage: SourceCoverage {
            source: source.identity().clone(),
            examined_range: coordinates(source.text(), 0..source.text().len()),
            recovery_regions,
            rust_parse_issues: diagnostics,
            total_matches,
            complete,
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
    let prepared = PreparedQuery::new(&query, replacement.as_deref())?;
    let inputs = requests
        .into_par_iter()
        .map(|request| request.acquire())
        .collect::<Result<Vec<_>, _>>()?;
    // Each source is analyzed once. Exact totals come from the same native traversal;
    // per-file retention never exceeds the global requested allowance.
    let results = inputs
        .par_iter()
        .map(|source| {
            analyze(
                source,
                &query,
                &prepared,
                query.max_records.unwrap_or(usize::MAX),
                &token,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let total_matches = results
        .iter()
        .map(|result| result.coverage.total_matches)
        .sum::<usize>();
    let mut remaining = query.max_records.unwrap_or(usize::MAX);
    let mut sources = Vec::new();
    let mut matches = Vec::new();
    let mut edits = Vec::new();
    let mut originals = Vec::new();
    for (original, mut result) in inputs.into_iter().zip(results) {
        result.matches.truncate(remaining);
        result.edits.truncate(remaining);
        remaining = remaining.saturating_sub(result.matches.len());
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
/// Re-match one exactly selected candidate and construct a capture-preserving preview.
/// The selected range cannot drift to another occurrence with the same spelling.
pub fn preview_selected(
    source: SourceText,
    selected: std::ops::Range<usize>,
    pattern_text: &str,
    strictness: Strictness,
    replacement: &str,
) -> Result<ChangeProposal, QueryError> {
    let query = Query {
        pattern: pattern_text.into(),
        strictness,
        inside: Vec::new(),
        has: Vec::new(),
        excluded: Vec::new(),
        captures: Vec::new(),
        edition: RustEdition::E2024,
        max_records: None,
    };
    let prepared = PreparedQuery::new(&query, Some(replacement))?;
    let parsed = ParsedSyntax::new(&source)?;
    let template = prepared
        .template
        .as_ref()
        .ok_or_else(|| QueryError::Unavailable("selected preview has no native template".into()))?;
    let edit = native::selected_edit(&source, &parsed, selected, &prepared.matcher, template)?;
    Ok(ChangeProposal::new(ProducerContext::Syntax {backend:"ast-grep-core/0.51.0;tree-sitter-rust/0.24.2".into(),configuration:format!("selected exact range; strictness={strictness:?}; native capture-preserving template")},vec![source],vec![edit],vec![0],vec!["Syntax preview only; no semantic identity, compilation or source/index application established".into()],vec![])?)
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
        let partial = report.cancellation_requested
            || report.sources.iter().any(|source| !source.complete)
            || report.omitted_matches > 0
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
        let partial = report.cancellation_requested
            || report.sources.iter().any(|source| !source.complete)
            || report.omitted_matches > 0
            || report.sources.iter().any(|source| {
                !source.recovery_regions.is_empty() || !source.rust_parse_issues.is_empty()
            });
        let providers = providers(&report)?;
        let mut response = envelope(
            "propose syntax",
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
