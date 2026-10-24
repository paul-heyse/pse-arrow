// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded source-aware reduction. Its property is separate from exact reproduction.
use super::{
    BoundaryDiagnostic, DiagnosticExecutionRequest, DiagnosticOperation, DiagnosticRuntimeSettings,
    Runtime, WorkflowError, contract,
};
use crate::CancelSource;
use pse_admission::authoring_driver::document::{OwnedDocumentBundle, load_package_sources_owned};
use pse_authoring::{ParseBudget, language::Selected};
use std::collections::BTreeSet;
const MAXIMUM_SOURCE_BYTES: usize = 256 << 10;
const MAXIMUM_MOVES: usize = 1024;
use pse_model::document::Version;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// An exact source premise that must survive every accepted edit.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReductionAnchor {
    /// Index of the explicitly supplied authored package.
    pub package: usize,
    /// Original local document name.
    pub document: String,
    /// UTF-8 starting byte offset.
    pub start: usize,
    /// UTF-8 exclusive ending byte offset.
    pub end: usize,
    /// Scientific responsibility of the protected bytes.
    pub role: ReductionAnchorRole,
}
/// Protected expression and explicit point are separate premises.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReductionAnchorRole {
    /// Exact expression whose evaluation produced the selected domain issue.
    FailureExpression,
    /// Explicit authored starting point necessary to reproduce that issue.
    InitialPoint,
}
/// Supported source transformations, explicitly selected by the caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReductionOperator {
    /// Delete complete leaf declarations using the authored language's source spans.
    DeclarationDeletion,
    /// Replace a binary expression by one of its actual AST children.
    BinarySubexpression,
}
/// Separate selected reduction predicate, never an original-reproduction result.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReductionRequest {
    /// Codec.
    pub version: Version<1>,
    /// Complete explicit original authored diagnostic input.
    pub original: DiagnosticExecutionRequest,
    /// Selected original typed issue. Code/stage/rule/sources/validity/conditions must match.
    pub issue: BoundaryDiagnostic,
    /// Exactly one scalar call and its explicit start, verified against actual parser attribution.
    pub anchors: Vec<ReductionAnchor>,
    /// Nonempty permitted transformations; protected bytes and the predicate still apply.
    pub operators: Vec<ReductionOperator>,
    /// Maximum new candidate executions including the unchanged control, from 1 to 256.
    pub maximum_candidates: usize,
    /// One total clock, including preparation and candidate drain.
    #[schemars(with = "pse_model::document::ClosedDuration")]
    pub allowance: Duration,
}
/// CLI composition over the existing runtime deployment.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReductionCommandRequest {
    /// Codec.
    pub version: Version<1>,
    /// Existing deployment settings.
    pub runtime: DiagnosticRuntimeSettings,
    /// Separate scientific reduction request.
    pub reduction: ReductionRequest,
}
/// Actual candidate outcome, never inferred from a command exit or message.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReductionDisposition {
    /// The selected typed issue and protected source premises survived.
    Preserved,
    /// A complete candidate did not retain the selected issue.
    Different,
    /// Candidate admission or preparation refused before numerical work.
    Refused,
    /// The candidate population or execution did not complete.
    Incomplete,
    /// Execution could not establish a meaningful comparison.
    Inconclusive,
}
/// Literal source edit in one attempted candidate's current coordinates.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReductionEdit {
    /// Source package index.
    pub package: usize,
    /// Local source document.
    pub document: String,
    /// Starting UTF-8 offset.
    pub start: usize,
    /// Exclusive ending UTF-8 offset.
    pub end: usize,
    /// Exact replacement bytes as text.
    pub replacement: String,
}
/// A bounded attempt retains actual typed evidence and its source transformation.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReductionAttempt {
    /// Actual new diagnostic operation, absent if construction itself refused.
    pub operation: Option<pse_diagnostics::observation::OperationId>,
    /// Literal source transformations; empty for the unchanged control.
    pub edits: Vec<ReductionEdit>,
    /// Scientific property outcome.
    pub disposition: ReductionDisposition,
    /// Selected matching issue or actual refusal/different diagnostic.
    pub diagnostic: Option<BoundaryDiagnostic>,
}
/// Original-to-reduced mapping of an unchanged scientific source premise.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReductionCorrespondence {
    /// Original supplied location.
    pub original: ReductionAnchor,
    /// Actual reduced location, containing identical bytes.
    pub reduced: ReductionAnchor,
}
/// Source-aware local search outcome. Does not claim exact original reproduction.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReductionDocument {
    /// Actual reduction operation, distinct from original reproduction and candidates.
    pub operation: pse_diagnostics::observation::OperationId,
    /// Codec.
    pub version: Version<1>,
    /// Original complete input and preserved issue.
    pub request: ReductionRequest,
    /// Smallest accepted input found under the actual moves and bounds.
    pub reduced: DiagnosticExecutionRequest,
    /// Original/reduced protected source bytes correspondence.
    pub correspondence: Vec<ReductionCorrespondence>,
    /// Ordered candidate evidence, including the unchanged control.
    pub attempts: Vec<ReductionAttempt>,
    /// No further accepted deletion/subexpression move was found in the completed search.
    pub locally_irreducible: bool,
    /// Actual stop or unsupported-premise reason.
    pub reason: String,
}
fn classify(
    issue: &BoundaryDiagnostic,
    result: &super::DiagnosticExecutionDocument,
) -> (ReductionDisposition, Option<BoundaryDiagnostic>) {
    let predicate = super::ReproductionPredicate::from_diagnostic(issue);
    if let Some(actual) = result
        .failure
        .iter()
        .chain(&result.findings)
        .find(|d| predicate.matches(d))
    {
        return (ReductionDisposition::Preserved, Some(actual.clone()));
    }
    if let Some(failure) = &result.failure {
        let state = if matches!(
            failure.failure_class(),
            pse_diagnostics::FailureClass::RuntimeCancelled
                | pse_diagnostics::FailureClass::RuntimeTimeout
                | pse_diagnostics::FailureClass::RuntimeResourceLimit
        ) {
            ReductionDisposition::Incomplete
        } else if result.run_id.is_none() {
            ReductionDisposition::Refused
        } else {
            ReductionDisposition::Different
        };
        return (state, Some(failure.clone()));
    }
    (
        if result.complete {
            ReductionDisposition::Different
        } else {
            ReductionDisposition::Incomplete
        },
        result.findings.first().cloned(),
    )
}
fn source<'a>(
    request: &'a DiagnosticExecutionRequest,
    anchor: &ReductionAnchor,
) -> Result<&'a str, WorkflowError> {
    request
        .model_documents
        .get(anchor.package)
        .and_then(|p| p.get(&anchor.document))
        .map(String::as_str)
        .ok_or_else(|| contract("reduction anchor requires an explicit authored source"))
}
fn protected(request: &ReductionRequest) -> Result<(), WorkflowError> {
    if request.maximum_candidates == 0
        || request.maximum_candidates > 256
        || request.allowance.is_zero()
        || request.allowance > Duration::from_secs(86400)
        || request.anchors.len() != 2
        || request.operators.is_empty()
        || request.operators.len() > 2
        || pse_model::HeapUsage::heap_bytes(&request.issue) > MAXIMUM_SOURCE_BYTES
    {
        return Err(contract("finite scientific reduction policy"));
    }
    if request.original.model_documents.len() > 32
        || request.original.physical_documents.len() > 128
        || request
            .original
            .model_documents
            .iter()
            .any(|package| package.len() > 128)
    {
        return Err(contract("finite scientific reduction source inventory"));
    }
    if !matches!(
        request.issue.code,
        pse_model::diagnostic::DiagnosticCode::CandidateEvaluationFailed
            | pse_model::diagnostic::DiagnosticCode::MathDomain
            | pse_model::diagnostic::DiagnosticCode::MathEvaluation
    ) || request.issue.sources.is_empty()
    {
        return Err(contract(
            "reduction requires an attributed numerical evaluation-domain issue",
        ));
    }
    for role in [
        ReductionAnchorRole::FailureExpression,
        ReductionAnchorRole::InitialPoint,
    ] {
        if !request.anchors.iter().any(|a| a.role == role) {
            return Err(contract(
                "reduction requires explicit expression and initial-point premises",
            ));
        }
    }
    request
        .original
        .physical_documents
        .iter()
        .chain(
            request
                .original
                .model_documents
                .iter()
                .flat_map(|p| p.iter()),
        )
        .try_fold(0usize, |n, (path, text)| {
            n.checked_add(path.len())?
                .checked_add(text.len())
                .filter(|bytes| *bytes <= MAXIMUM_SOURCE_BYTES)
        })
        .filter(|bytes| *bytes <= MAXIMUM_SOURCE_BYTES)
        .ok_or_else(|| contract("source reduction supports at most 256 KiB of explicit source"))?;
    for anchor in &request.anchors {
        let text = source(&request.original, anchor)?;
        if anchor.start >= anchor.end || text.get(anchor.start..anchor.end).is_none() {
            return Err(contract(
                "reduction anchor requires nonempty UTF-8 source bounds",
            ));
        }
    }
    Ok(())
}
fn apply(
    request: &DiagnosticExecutionRequest,
    anchors: &[ReductionAnchor],
    edits: &[ReductionEdit],
) -> Option<(DiagnosticExecutionRequest, Vec<ReductionAnchor>)> {
    let mut next = request.clone();
    let mut mapped = anchors.to_vec();
    let mut ordered = edits.to_vec();
    ordered
        .sort_by(|a, b| (a.package, &a.document, b.start).cmp(&(b.package, &b.document, a.start)));
    for e in &ordered {
        let text = next
            .model_documents
            .get_mut(e.package)?
            .get_mut(&e.document)?;
        if e.start >= e.end || text.get(e.start..e.end).is_none() {
            return None;
        }
        let old = text.clone();
        for a in &mut mapped {
            if a.package != e.package || a.document != e.document {
                continue;
            }
            if a.end <= e.start {
                continue;
            }
            if a.start >= e.end {
                a.start = a
                    .start
                    .checked_sub(e.end - e.start)?
                    .checked_add(e.replacement.len())?;
                a.end = a
                    .end
                    .checked_sub(e.end - e.start)?
                    .checked_add(e.replacement.len())?;
            } else if e.start <= a.start && e.end >= a.end {
                let bytes = old.get(a.start..a.end)?;
                let mut matches = e.replacement.match_indices(bytes);
                let (offset, _) = matches.next()?;
                if matches.next().is_some() {
                    return None;
                }
                a.start = e.start + offset;
                a.end = a.start + bytes.len();
            } else {
                return None;
            }
        }
        text.replace_range(e.start..e.end, &e.replacement);
    }
    Some((next, mapped))
}
/// Exact parser-owned equation field; a quoted name may contain the same expression text.
fn expression_span(
    document: &pse_admission::authoring_driver::document::Document,
    row: &pse_authoring::language::Declaration,
) -> Option<pse_authoring::SourceSpan> {
    document.spans.span(&format!(
        "/modeling-fields/{}/equation.expression",
        row.declaration_id
    ))
}
fn validate_anchors(
    request: &ReductionRequest,
    actual: &BoundaryDiagnostic,
    sources: &[OwnedDocumentBundle],
    operation: &DiagnosticOperation,
) -> Result<(), WorkflowError> {
    let failure = request
        .anchors
        .iter()
        .find(|a| a.role == ReductionAnchorRole::FailureExpression)
        .ok_or_else(|| contract("missing expression premise"))?;
    let point = request
        .anchors
        .iter()
        .find(|a| a.role == ReductionAnchorRole::InitialPoint)
        .ok_or_else(|| contract("missing point premise"))?;
    if failure.package != point.package || failure.document != point.document {
        return Err(contract(
            "first reduction family requires expression and scalar start in the same document",
        ));
    }
    let document = sources
        .get(failure.package)
        .and_then(|p| {
            p.bundle()
                .documents
                .iter()
                .find(|d| d.path == failure.document)
        })
        .ok_or_else(|| contract("unknown reduction document"))?;
    let rows = document
        .modeling_rows()
        .ok_or_else(|| contract("reduction requires authored modeling source"))?;
    let mut selected = None;
    for row in rows.iter() {
        operation.check()?;
        let Ok(Selected::Equation(eq)) = row.value.selected() else {
            continue;
        };
        let Some(span) = expression_span(document, row) else {
            continue;
        };
        if !(span.start as usize <= failure.start && failure.end <= span.end as usize) {
            continue;
        }
        let ast = pse_authoring::dsl::parse_equation(&eq.expression)
            .map_err(|_| contract("expression premise cannot be parsed"))?;
        let mut calls = Vec::new();
        let mut paths = BTreeSet::new();
        let mut unsupported = false;
        if let pse_authoring::dsl::EquationKind::Relation { lhs, rhs, .. } = &ast.kind {
            for root in [lhs, rhs] {
                root.walk(|node| match &node.kind {
                    pse_authoring::dsl::ExprKind::Call { args, .. } => {
                        let target = match args.as_slice() {
                            [
                                pse_authoring::dsl::Expr {
                                    kind: pse_authoring::dsl::ExprKind::Path(path),
                                    ..
                                },
                            ] => path.ident().map(str::to_owned),
                            _ => None,
                        };
                        calls.push((node.span, target));
                    }
                    pse_authoring::dsl::ExprKind::Path(path) => {
                        if let Some(name) = path.ident() {
                            paths.insert(name.to_owned());
                        } else {
                            unsupported = true;
                        }
                    }
                    pse_authoring::dsl::ExprKind::Number(_)
                    | pse_authoring::dsl::ExprKind::Neg(_)
                    | pse_authoring::dsl::ExprKind::Binary { .. } => {}
                    _ => unsupported = true,
                });
            }
        }
        if unsupported
            || calls.len() != 1
            || paths.len() != 1
            || calls[0].1.as_ref() != paths.first()
            || failure.start != span.start as usize + calls[0].0.start as usize
            || failure.end != span.start as usize + calls[0].0.end as usize
        {
            continue;
        }
        if !actual.locations.iter().any(|location| {
            location.document == Some(document.id)
                && actual.sources.contains(&location.source)
                && location
                    .start
                    .is_some_and(|start| start as usize <= failure.start)
                && location.end.is_some_and(|end| failure.end <= end as usize)
        }) {
            continue;
        }
        selected = Some((
            row.parent_id,
            paths
                .into_iter()
                .next()
                .ok_or_else(|| contract("scalar premise"))?,
        ));
    }
    let (parent,target)=selected.ok_or_else(||contract("expression anchor must name the sole scalar call at the actual attributed issue occurrence"))?;
    let valid=rows.iter().any(|row|row.parent_id==parent && row.source_start as usize==point.start && row.source_end as usize==point.end
        && matches!(row.value.selected(),Ok(Selected::Annotation(a)) if a.kind==pse_model::generated::enums::ModelingAnnotationKind::Start
            && a.target==target && a.arguments.len()==1 && a.arguments[0].parse::<f64>().is_ok_and(f64::is_finite)));
    if !valid {
        return Err(contract(
            "initial-point anchor must be the explicit finite scalar start for the attributed expression coordinate",
        ));
    }
    operation.check()
}
fn admissible(edit: &ReductionEdit, anchors: &[ReductionAnchor]) -> bool {
    anchors.iter().all(|anchor| {
        anchor.package != edit.package
            || anchor.document != edit.document
            || anchor.end <= edit.start
            || anchor.start >= edit.end
            || (edit.start <= anchor.start
                && edit.end >= anchor.end
                && !edit.replacement.is_empty())
    })
}
fn retain_move(edits: &mut Vec<ReductionEdit>, bytes: &mut usize, edit: ReductionEdit) -> bool {
    let Some(next) = bytes
        .checked_add(edit.document.len())
        .and_then(|n| n.checked_add(edit.replacement.len()))
        .filter(|n| *n <= MAXIMUM_SOURCE_BYTES)
    else {
        return false;
    };
    if edits.len() >= MAXIMUM_MOVES {
        return false;
    }
    *bytes = next;
    edits.push(edit);
    true
}
/// Enumerate actual parser-owned declarations and expression spans within finite work bounds.
fn moves(
    sources: &[OwnedDocumentBundle],
    operation: &DiagnosticOperation,
) -> Result<Vec<ReductionEdit>, WorkflowError> {
    let mut edits = Vec::new();
    let mut declarations = 0usize;
    let mut bytes = 0usize;
    for (package, bundle) in sources.iter().enumerate() {
        for document in &bundle.bundle().documents {
            operation.check()?;
            let Some(rows) = document.modeling_rows() else {
                continue;
            };
            declarations = declarations
                .checked_add(rows.len())
                .filter(|n| *n <= MAXIMUM_MOVES)
                .ok_or_else(|| contract("reduction declaration bound exceeded"))?;
            let parents = rows
                .iter()
                .filter_map(|row| row.parent_id)
                .collect::<BTreeSet<_>>();
            for row in rows.iter() {
                operation.check()?;
                if !parents.contains(&row.declaration_id) {
                    if !retain_move(
                        &mut edits,
                        &mut bytes,
                        ReductionEdit {
                            package,
                            document: document.path.clone(),
                            start: row.source_start as usize,
                            end: row.source_end as usize,
                            replacement: String::new(),
                        },
                    ) {
                        return Err(contract("reduction move payload bound exceeded"));
                    }
                }
                if let Ok(Selected::Equation(eq)) = row.value.selected() {
                    let Some(span) = expression_span(document, row) else {
                        continue;
                    };
                    if let Ok(ast) = pse_authoring::dsl::parse_equation(&eq.expression)
                        && let pse_authoring::dsl::EquationKind::Relation { lhs, rhs, .. } =
                            &ast.kind
                    {
                        let mut refused = false;
                        for root in [lhs, rhs] {
                            root.walk(|node| {
                                if refused {
                                    return;
                                }
                                if edits.len() >= MAXIMUM_MOVES || operation.check().is_err() {
                                    refused = true;
                                    return;
                                }
                                if let pse_authoring::dsl::ExprKind::Binary { lhs, rhs, .. } =
                                    &node.kind
                                {
                                    for child in [lhs, rhs] {
                                        let replacement = pse_authoring::dsl::render_expr(child);
                                        if replacement.len()
                                            < (node.span.end - node.span.start) as usize
                                        {
                                            if !retain_move(
                                                &mut edits,
                                                &mut bytes,
                                                ReductionEdit {
                                                    package,
                                                    document: document.path.clone(),
                                                    start: span.start as usize
                                                        + node.span.start as usize,
                                                    end: span.start as usize
                                                        + node.span.end as usize,
                                                    replacement,
                                                },
                                            ) {
                                                refused = true;
                                                break;
                                            }
                                        }
                                    }
                                }
                            });
                        }
                        if refused {
                            return Err(contract(
                                "reduction move payload, count or execution bound reached",
                            ));
                        }
                    }
                }
                if edits.len() > MAXIMUM_MOVES {
                    return Err(contract("reduction move bound exceeded"));
                }
            }
        }
    }
    operation.check()?;
    Ok(edits)
}
impl Runtime {
    fn reduction_sources(
        &self,
        request: &DiagnosticExecutionRequest,
        operation: &DiagnosticOperation,
    ) -> Result<Vec<OwnedDocumentBundle>, WorkflowError> {
        let mut sources = Vec::new();
        let pool = self.shared.pool();
        let token = operation.driver().token();
        let validation = self.admission_context();
        let mut declarations = 0usize;
        for package in &request.model_documents {
            operation.check()?;
            sources.push(load_package_sources_owned(
                package
                    .iter()
                    .map(|(path, text)| (path.as_str(), text.as_bytes())),
                self.registry(),
                ParseBudget {
                    max_bytes: MAXIMUM_SOURCE_BYTES as u64,
                    ..ParseBudget::default()
                },
                &pool,
                &token,
                &validation,
            )?);
            declarations = sources
                .last()
                .ok_or_else(|| contract("missing parsed source"))?
                .bundle()
                .documents
                .iter()
                .try_fold(declarations, |n, document| {
                    n.checked_add(document.modeling_rows().map_or(0, |rows| rows.len()))
                        .filter(|n| *n <= MAXIMUM_MOVES)
                })
                .ok_or_else(|| contract("reduction declaration bound exceeded"))?;
            operation.check()?;
        }
        Ok(sources)
    }
    /// Execute bounded candidate work using the ordinary diagnostic and cancellation owners.
    pub async fn reduce_diagnostic(
        &self,
        request: ReductionRequest,
        parent: &CancelSource,
    ) -> Result<ReductionDocument, WorkflowError> {
        protected(&request)?;
        let operation =
            DiagnosticOperation::new(parent, request.allowance, request.original.origin.clone())?;
        operation.check()?;
        // Bound retained request/candidate copies and search metadata before parsing/search.
        let _search_owner = self.shared.math().reserve(
            "diagnostic:source-reduction",
            MAXIMUM_SOURCE_BYTES * 8 + MAXIMUM_MOVES * 512,
        )?;
        let mut attempt_owners = Vec::new();
        let initial_sources = self.reduction_sources(&request.original, &operation)?;
        let observer = self.observations().hub().start("diagnose.reduction");
        let context = observer.context();
        let mut result = ReductionDocument {
            operation: context.id(),
            version: Version,
            reduced: request.original.clone(),
            correspondence: request
                .anchors
                .iter()
                .map(|a| ReductionCorrespondence {
                    original: a.clone(),
                    reduced: a.clone(),
                })
                .collect(),
            request,
            attempts: Vec::new(),
            locally_irreducible: false,
            reason: String::new(),
        };
        let original = pse_diagnostics::observation::scope(
            Some(context.clone()),
            self.diagnose_documents(result.reduced.clone(), operation.driver())
                .boxed(),
        )
        .await?;
        let (disposition, diagnostic) = classify(&result.request.issue, &original);
        if let Some(actual) = &diagnostic
            && disposition == ReductionDisposition::Preserved
        {
            validate_anchors(&result.request, actual, &initial_sources, &operation)?;
        }
        if let Some(diagnostic) = &diagnostic {
            attempt_owners.push(self.shared.math().reserve(
                "diagnostic:reduction-evidence",
                pse_model::HeapUsage::heap_bytes(diagnostic) + size_of::<ReductionAttempt>(),
            )?);
        }
        result.attempts.push(ReductionAttempt {
            operation: Some(original.operation),
            edits: vec![],
            disposition,
            diagnostic,
        });
        if disposition != ReductionDisposition::Preserved {
            result.reason = "unchanged control did not establish the selected issue".into();
            observer.finish(pse_diagnostics::observation::Outcome::Failed);
            return Ok(result);
        }
        drop(initial_sources);
        let mut granularity = 2;
        loop {
            if result.attempts.len() >= result.request.maximum_candidates {
                result.reason = "candidate bound reached".into();
                break;
            }
            if operation.check().is_err() {
                result.reason =
                    "cancelled or total allowance exhausted; completed candidates retained".into();
                break;
            }
            let anchors = result
                .correspondence
                .iter()
                .map(|c| c.reduced.clone())
                .collect::<Vec<_>>();
            let sources = match self.reduction_sources(&result.reduced, &operation) {
                Ok(sources) => sources,
                Err(error) => {
                    result.reason = format!("source search unavailable: {error}");
                    break;
                }
            };
            let all = match moves(&sources, &operation) {
                Ok(moves) => moves,
                Err(error) => {
                    result.reason = format!("source search incomplete: {error}");
                    break;
                }
            };
            drop(sources);
            let all = all
                .into_iter()
                .filter(|edit| {
                    let kind = if edit.replacement.is_empty() {
                        ReductionOperator::DeclarationDeletion
                    } else {
                        ReductionOperator::BinarySubexpression
                    };
                    result.request.operators.contains(&kind) && admissible(edit, &anchors)
                })
                .collect::<Vec<_>>();
            let deletions = all
                .iter()
                .filter(|e| e.replacement.is_empty())
                .cloned()
                .collect::<Vec<_>>();
            let mut batches = Vec::new();
            if !deletions.is_empty() {
                for chunk in deletions.chunks(deletions.len().div_ceil(granularity)) {
                    batches.push(chunk.to_vec());
                }
            }
            batches.extend(
                all.into_iter()
                    .filter(|e| !e.replacement.is_empty())
                    .map(|e| vec![e]),
            );
            let mut accepted = false;
            let mut bounded = false;
            let mut uncertain = false;
            for edits in batches {
                if result.attempts.len() >= result.request.maximum_candidates
                    || operation.check().is_err()
                {
                    bounded = true;
                    break;
                }
                let Some((candidate, mapped)) = apply(&result.reduced, &anchors, &edits) else {
                    continue;
                };
                let execution = pse_diagnostics::observation::scope(
                    Some(context.clone()),
                    self.diagnose_documents(candidate.clone(), operation.driver())
                        .boxed(),
                )
                .await;
                let (candidate_operation, disposition, diagnostic) = match execution {
                    Ok(doc) => {
                        let (disposition, diagnostic) = classify(&result.request.issue, &doc);
                        (Some(doc.operation), disposition, diagnostic)
                    }
                    Err(error) => (
                        None,
                        ReductionDisposition::Inconclusive,
                        Some(error.boundary_diagnostic()),
                    ),
                };
                let retained = diagnostic
                    .as_ref()
                    .map_or(0, pse_model::HeapUsage::heap_bytes)
                    + edits
                        .iter()
                        .map(|edit| {
                            size_of::<ReductionEdit>()
                                + edit.document.len()
                                + edit.replacement.len()
                        })
                        .sum::<usize>()
                    + size_of::<ReductionAttempt>();
                match self
                    .shared
                    .math()
                    .reserve("diagnostic:reduction-evidence", retained)
                {
                    Ok(owner) => attempt_owners.push(owner),
                    Err(_) => {
                        result.reason =
                            "evidence memory admission refused; completed candidates retained"
                                .into();
                        bounded = true;
                        break;
                    }
                }
                result.attempts.push(ReductionAttempt {
                    operation: candidate_operation,
                    edits,
                    disposition,
                    diagnostic,
                });
                uncertain |= matches!(
                    disposition,
                    ReductionDisposition::Incomplete | ReductionDisposition::Inconclusive
                );
                if disposition == ReductionDisposition::Preserved {
                    result.reduced = candidate;
                    for (c, a) in result.correspondence.iter_mut().zip(mapped) {
                        c.reduced = a;
                    }
                    accepted = true;
                    granularity = 2;
                    break;
                }
            }
            if accepted {
                continue;
            }
            if bounded {
                result.reason = "candidate or execution bound reached".into();
                break;
            }
            if uncertain {
                result.reason="candidate evidence incomplete or inconclusive; local irreducibility not established".into();
                break;
            }
            if granularity < deletions.len() {
                granularity = (granularity * 2).min(deletions.len());
                continue;
            }
            result.locally_irreducible = true;
            result.reason =
                "no preserving declaration deletion or binary-subexpression move remains".into();
            break;
        }
        observer.finish(if operation.driver().token().is_cancelled() {
            pse_diagnostics::observation::Outcome::Cancelled
        } else if operation.check().is_err() {
            pse_diagnostics::observation::Outcome::Interrupted
        } else {
            pse_diagnostics::observation::Outcome::Completed
        });
        Ok(result)
    }
}
use futures_util::FutureExt;

#[cfg(test)]
mod tests {
    use super::*;
    fn anchor(text: &str, needle: &str, role: ReductionAnchorRole) -> ReductionAnchor {
        let start = text.find(needle).unwrap();
        ReductionAnchor {
            package: 0,
            document: "models/original.pse".into(),
            start,
            end: start + needle.len(),
            role,
        }
    }
    #[tokio::test]
    async fn numerical_reduction_retains_domain_issue_and_rejects_other_failures() {
        let runtime = super::super::tests::runtime();
        let mut original=super::super::reproducer::tests::numerical_command(
            "var x:Scalar;eq e:log(x)==0;annotation start x(-1);var spare:Scalar;eq spare_eq:log(spare)==0;annotation start spare(2);").operation;
        original.settings.intent = pse_backend_native::solve::SolveIntent::Root;
        let first = runtime
            .diagnose_documents(original.clone(), &CancelSource::new())
            .await
            .unwrap();
        let issue = first
            .failure
            .iter()
            .chain(&first.findings)
            .find(|d| d.failure_class() == pse_diagnostics::FailureClass::SolveEvaluationError)
            .unwrap_or_else(|| panic!("original must exercise domain failure: {first:?}"))
            .clone();
        let text = &original.model_documents[0]["models/original.pse"];
        let anchors = vec![
            anchor(text, "log(x)", ReductionAnchorRole::FailureExpression),
            anchor(
                text,
                "annotation start x(-1);",
                ReductionAnchorRole::InitialPoint,
            ),
        ];
        let request = ReductionRequest {
            version: Version,
            original: original.clone(),
            issue: issue.clone(),
            anchors: anchors.clone(),
            operators: vec![
                ReductionOperator::DeclarationDeletion,
                ReductionOperator::BinarySubexpression,
            ],
            maximum_candidates: 48,
            allowance: Duration::from_secs(60),
        };
        let mut unrelated = request.clone();
        let spare = anchor(
            text,
            "var spare:Scalar;",
            ReductionAnchorRole::FailureExpression,
        );
        unrelated.anchors[0] = spare.clone();
        unrelated.anchors[1] = ReductionAnchor {
            role: ReductionAnchorRole::InitialPoint,
            ..spare
        };
        assert!(
            runtime
                .reduce_diagnostic(unrelated, &CancelSource::new())
                .await
                .is_err(),
            "labels cannot give unrelated bytes scientific roles"
        );
        let mut fabricated = request.clone();
        fabricated.anchors = vec![
            anchor(text, "log(spare)", ReductionAnchorRole::FailureExpression),
            anchor(
                text,
                "annotation start spare(2);",
                ReductionAnchorRole::InitialPoint,
            ),
        ];
        for location in &mut fabricated.issue.locations {
            location.start = Some(fabricated.anchors[0].start as u32);
            location.end = Some(fabricated.anchors[0].end as u32);
        }
        assert!(
            runtime
                .reduce_diagnostic(fabricated, &CancelSource::new())
                .await
                .is_err(),
            "caller-edited locations cannot attribute a different successful call to the failure"
        );
        let cancelled = CancelSource::new();
        cancelled.cancel();
        assert!(
            runtime
                .reduce_diagnostic(request.clone(), &cancelled)
                .await
                .is_err()
        );
        let mut expired = request.clone();
        expired.allowance = Duration::from_nanos(1);
        assert!(
            runtime
                .reduce_diagnostic(expired, &CancelSource::new())
                .await
                .is_err()
        );
        let reduced = runtime
            .reduce_diagnostic(request, &CancelSource::new())
            .await
            .unwrap();
        assert_eq!(
            reduced.attempts[0].disposition,
            ReductionDisposition::Preserved
        );
        let result = &reduced.reduced.model_documents[0]["models/original.pse"];
        assert!(
            result.len() < text.len(),
            "must actually remove irrelevant authored operations: {reduced:?}"
        );
        assert!(!result.contains("spare_eq"));
        for mapping in &reduced.correspondence {
            assert_eq!(
                &text[mapping.original.start..mapping.original.end],
                &result[mapping.reduced.start..mapping.reduced.end]
            );
        }
        let edit = ReductionEdit {
            package: 0,
            document: anchors[0].document.clone(),
            start: anchors[0].start,
            end: anchors[0].end,
            replacement: "sqrt(x)".into(),
        };
        assert!(
            apply(&original, &anchors, &[edit]).is_none(),
            "different domain operation cannot replace the issue premise"
        );
        let mut malformed = original.clone();
        malformed.model_documents[0].insert("models/original.pse".into(), "package {".into());
        let refused = runtime
            .diagnose_documents(malformed, &CancelSource::new())
            .await
            .unwrap();
        assert_ne!(
            classify(&issue, &refused).0,
            ReductionDisposition::Preserved
        );
        let mut singular = super::super::reproducer::tests::numerical_command(
            "var y:Scalar;eq singular:1/y==0;annotation start y(0);",
        )
        .operation;
        singular.settings.intent = pse_backend_native::solve::SolveIntent::Root;
        let different = runtime
            .diagnose_documents(singular, &CancelSource::new())
            .await
            .unwrap();
        assert_ne!(
            classify(&issue, &different).0,
            ReductionDisposition::Preserved
        );
        let concrete = runtime
            .diagnose_documents(reduced.reduced, &CancelSource::new())
            .await
            .unwrap();
        assert_eq!(
            classify(&issue, &concrete).0,
            ReductionDisposition::Preserved
        );
    }
    #[test]
    fn protected_initial_point_and_utf8_correspondence_are_not_repaired() {
        let original = super::super::reproducer::tests::numerical_command(
            "var x:Scalar;eq e:log(x)==0;annotation start x(-1);",
        )
        .operation;
        let text = &original.model_documents[0]["models/original.pse"];
        let a = anchor(
            text,
            "annotation start x(-1);",
            ReductionAnchorRole::InitialPoint,
        );
        let edit = ReductionEdit {
            package: 0,
            document: a.document.clone(),
            start: a.start,
            end: a.end,
            replacement: String::new(),
        };
        assert!(apply(&original, &[a], &[edit]).is_none());
    }
    #[tokio::test]
    async fn quoted_equation_name_cannot_redirect_ast_edits() {
        let runtime = super::super::tests::runtime();
        let original = super::super::reproducer::tests::numerical_command(
            "var x:Scalar;eq \"log(x)+0==0\":log(x)+0==0;annotation start x(-1);",
        )
        .operation;
        let operation =
            DiagnosticOperation::new(&CancelSource::new(), Duration::from_secs(10), None).unwrap();
        let sources = runtime.reduction_sources(&original, &operation).unwrap();
        let text = &original.model_documents[0]["models/original.pse"];
        let expression = text.rfind("log(x)+0==0").unwrap();
        let edits = moves(&sources, &operation).unwrap();
        let binary = edits
            .iter()
            .filter(|edit| !edit.replacement.is_empty())
            .collect::<Vec<_>>();
        assert!(
            !binary.is_empty(),
            "the actual +0 expression has an AST reduction"
        );
        assert!(
            binary.iter().all(|edit| edit.start >= expression),
            "quoted name is not the expression source span"
        );
        let name = "v".repeat(256);
        let repeated = std::iter::repeat_n(name.as_str(), 48)
            .collect::<Vec<_>>()
            .join(" + ");
        let large = super::super::reproducer::tests::numerical_command(&format!(
            "var {name}:Scalar;eq e:{repeated}==0;"
        ))
        .operation;
        let sources = runtime.reduction_sources(&large, &operation).unwrap();
        assert!(
            moves(&sources, &operation).is_err(),
            "overlapping large subexpressions must not escape the aggregate edit payload budget"
        );
    }
}
