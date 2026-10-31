// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native Rust facts refine selected source candidates without reparsing display text.
use super::{providers::ProviderInput, *};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::Path,
};
use tokio_util::sync::CancellationToken;

/// Evaluate only the selected source; global semantic queries retain their separate owner.
pub(crate) fn evaluate_source(
    root: &Path,
    input: &ProviderInput,
    predicate: &Predicate,
    token: &CancellationToken,
    limit: Option<usize>,
    context: &mut rust::EvaluationContext,
) -> Result<rust::Evaluation, Error> {
    rust::evaluate(root, input, predicate, token, limit, context)
}

/// Select the orchestration owner only for genuinely cross-provider/context questions.
pub fn required(predicate: &Predicate) -> bool {
    match predicate {
        Predicate::Refine { .. } | Predicate::Related { .. } => true,
        Predicate::All {
            within: MatchScope::Entity,
            ..
        }
        | Predicate::Any {
            within: MatchScope::Entity,
            ..
        }
        | Predicate::Not {
            within: MatchScope::Entity,
            ..
        } => true,
        Predicate::All { clauses, .. } | Predicate::Any { clauses, .. } => clauses
            .iter()
            .any(|clause| required(clause) || global(clause)),
        Predicate::Not { clause, .. } => required(clause) || global(clause),
        _ => false,
    }
}
fn global(predicate: &Predicate) -> bool {
    matches!(
        predicate,
        Predicate::Api(_)
            | Predicate::History(_)
            | Predicate::Symbol(SymbolQuery {
                provider: DiscoveryProvider::Semantic | DiscoveryProvider::Api,
                ..
            })
    )
}
fn is_api(predicate: &Predicate) -> bool {
    matches!(
        predicate,
        Predicate::Api(_)
            | Predicate::Symbol(SymbolQuery {
                provider: DiscoveryProvider::Api,
                ..
            })
    )
}
fn contains_api(predicate: &Predicate) -> bool {
    match predicate {
        predicate if is_api(predicate) => true,
        Predicate::Refine { candidates, .. }
        | Predicate::Not {
            clause: candidates, ..
        } => contains_api(candidates),
        Predicate::All { clauses, .. } | Predicate::Any { clauses, .. } => {
            clauses.iter().any(contains_api)
        }
        _ => false,
    }
}
fn is_api_locator(locator: &Locator) -> bool {
    matches!(locator, Locator::Api { .. })
}
fn nomination_scope(
    root: &Path,
    selector: &SourceSelector,
    token: &CancellationToken,
) -> Result<Option<BTreeSet<std::path::PathBuf>>, Error> {
    if selector.kind != SourceKind::Working {
        return Err(Error::Unavailable("API/native source refinement cannot reinterpret index/revision selectors as working-source nominations".into()));
    }
    if selector == &SourceSelector::default() {
        return Ok(None);
    }
    providers::select(root, selector, token)?
        .into_iter()
        .map(|source| root.join(source.path()).canonicalize().map_err(Error::from))
        .collect::<Result<BTreeSet<_>, _>>()
        .map(Some)
}
fn nomination_allowed(
    scope: &Option<BTreeSet<std::path::PathBuf>>,
    locator: &Locator,
) -> Result<bool, Error> {
    let Some(scope) = scope else {
        return Ok(true);
    };
    let Locator::Semantic { path, .. } = locator else {
        return Err(Error::Unavailable(
            "verified API nomination has no guarded native source location".into(),
        ));
    };
    Ok(scope.contains(&path.to_path_buf().canonicalize()?))
}

/// Run a typed native pipeline. Each bounded producer is joined before returning, including
/// sink failure and cancellation; the caller still owns the enclosing operation's settlement.
pub fn run(
    root: &Path,
    request: &Request,
    token: &CancellationToken,
    emit: &mut (dyn FnMut(Event) -> Result<(), Error> + Send),
    runtime: &tokio::runtime::Handle,
) -> Result<Summary, Error> {
    request.validate()?;
    // Keep the consumer's original refusal outside nested provider error reporting. A
    // closed writer cannot receive the diagnostic for its own failure, and each admitted
    // producer must still be joined before that refusal reaches the caller.
    let mut sink_error = None;
    let mut native_failures = BTreeSet::new();
    let mut forward = |event| {
        if let Event::NativeFailure { failure } = &event {
            if !native_failures.insert(serde_json::to_string(failure)?) {
                return Ok(());
            }
        }
        match emit(event) {
            Ok(()) => Ok(()),
            Err(error) => {
                if sink_error.is_none() {
                    sink_error = Some(error);
                }
                Err(Error::Cancelled)
            }
        }
    };
    let result = (|| -> Result<Summary, Error> {
        let resolution = rust::ResolutionContext::default();
        let membership = std::sync::Mutex::new(BTreeMap::new());
        let work = std::sync::Mutex::new(Summary {
            complete: true,
            ..Default::default()
        });
        let source_ids = std::sync::Mutex::new(BTreeSet::new());
        let native = Native {
            runtime,
            resolution: &resolution,
            membership: &membership,
            work: &work,
            api_nomination_scope: false,
            source_ids: &source_ids,
        };
        let mut summary = if request.answer.order == ResultOrder::Stable {
            stable_related(root, request, token, &mut forward, native)?
        } else {
            run_inner(
                root,
                request,
                &request.predicate,
                token,
                &mut forward,
                native,
            )?
        };
        if let Err(error) = native.resolution.verify_unchanged() {
            failed(&mut summary, &mut forward, error)?;
        }
        let examined = work
            .lock()
            .map_err(|_| Error::Unavailable("native clause-work owner panicked".into()))?;
        merge_work(&mut summary, &examined);
        if token.is_cancelled() {
            summary.cancellation_requested = true;
            summary.complete = false;
            summary.stop = StopReason::Cancelled;
        }
        summary.count = if summary.complete && summary.unknown == 0 && summary.failures == 0 {
            Count::Exact(summary.matches)
        } else {
            Count::AtLeast(summary.matches)
        };
        Ok(summary)
    })();
    drop(forward);
    if let Some(error) = sink_error {
        Err(error)
    } else {
        result
    }
}
#[derive(Clone, Copy)]
struct Native<'a> {
    runtime: &'a tokio::runtime::Handle,
    resolution: &'a rust::ResolutionContext,
    membership: &'a std::sync::Mutex<BTreeMap<String, std::sync::Arc<MembershipIndex>>>,
    work: &'a std::sync::Mutex<Summary>,
    api_nomination_scope: bool,
    source_ids: &'a std::sync::Mutex<BTreeSet<String>>,
}
impl Native<'_> {
    fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.runtime.block_on(future)
    }
}
fn source_key(source: &SourceContext) -> Result<String, Error> {
    use sha2::{Digest as _, Sha256};
    let encoded = if let Some(native) = &source.native {
        serde_json::to_vec(&(
            &native.session_invocation,
            native.generation,
            native
                .analysis
                .configuration
                .identity()
                .map_err(Error::from)?,
        ))?
    } else {
        serde_json::to_vec(source)?
    };
    Ok(format!("context-{}", hex(&Sha256::digest(encoded))))
}
fn emit_source(
    mut source: SourceContext,
    emit: &mut (dyn FnMut(Event) -> Result<(), Error> + Send),
    native: Native<'_>,
) -> Result<String, Error> {
    let id = source_key(&source)?;
    source.id = id.clone();
    if native
        .source_ids
        .lock()
        .map_err(|_| Error::Unavailable("shared source-context owner panicked".into()))?
        .insert(id.clone())
    {
        emit(Event::Source(source))?;
    }
    Ok(id)
}
fn resolved_context(
    entity: &rust::ResolvedEntity,
    emit: &mut (dyn FnMut(Event) -> Result<(), Error> + Send),
    native: Native<'_>,
) -> Result<(), Error> {
    let analysis: crate::semantic::AnalysisContext =
        serde_json::from_value(entity.native["session"].clone())?;
    let source = SourceContext {
        id: String::new(),
        path: analysis.configuration.project.clone().into(),
        origin: Some(SourceKind::Working),
        provider: analysis.backend.clone(),
        generation: Some(entity.identity.generation.to_string()),
        searched_encoding: None,
        native: Some(NativeContext {
            session_invocation: entity.identity.session_invocation.clone(),
            generation: entity.identity.generation,
            analysis,
        }),
    };
    emit_source(source, emit, native).map(|_| ())
}
#[derive(Clone)]
struct MembershipIndex {
    entities: BTreeSet<EntityIdentity>,
    complete: bool,
}
impl MembershipIndex {
    fn contains(&self, identity: &EntityIdentity) -> Truth {
        if self
            .entities
            .iter()
            .any(|entity| same_entity(entity, identity) == Truth::True)
        {
            Truth::True
        } else if self.complete
            && self
                .entities
                .iter()
                .all(|entity| same_entity(entity, identity) != Truth::Unknown)
        {
            Truth::False
        } else {
            Truth::Unknown
        }
    }
}
fn limits(request: &Request) -> Option<usize> {
    if request.answer.kind == AnswerKind::Count {
        None
    } else if request.answer.kind == AnswerKind::Exists {
        Some(1)
    } else {
        request.answer.limit
    }
}
fn visible(request: &Request, summary: &Summary) -> bool {
    !matches!(request.answer.kind, AnswerKind::Count | AnswerKind::Exists)
        && request
            .presentation
            .budget
            .is_none_or(|n| summary.emitted < n as u64)
}
fn diagnostic(
    emit: &mut (dyn FnMut(Event) -> Result<(), Error> + Send),
    error: &Error,
) -> Result<(), Error> {
    if let Some(failure) = error.native_refusal() {
        emit(Event::NativeFailure {
            failure: Box::new(failure.clone()),
        })?;
    }
    emit(Event::Diagnostic {
        code: if unavailable(error) {
            "search.composition_unavailable"
        } else {
            "search.composition_failure"
        }
        .into(),
        message: error.to_string(),
        source: None,
    })
}
fn unavailable(error: &Error) -> bool {
    error.disposition() == FailureDisposition::Unavailable
}
fn failed(
    summary: &mut Summary,
    emit: &mut (dyn FnMut(Event) -> Result<(), Error> + Send),
    error: Error,
) -> Result<(), Error> {
    if error.disposition() == FailureDisposition::Cancelled {
        if let Some(failure) = error.native_refusal() {
            emit(Event::NativeFailure {
                failure: Box::new(failure.clone()),
            })?;
        }
        summary.complete = false;
        summary.cancellation_requested = true;
        summary.stop = StopReason::Cancelled;
        return Ok(());
    }
    diagnostic(emit, &error)?;
    summary.complete = false;
    if unavailable(&error) {
        summary.unknown += 1;
    } else {
        summary.failures += 1;
    }
    Ok(())
}
fn merge_work(summary: &mut Summary, other: &Summary) {
    summary.examined_files = summary.examined_files.saturating_add(other.examined_files);
    summary.examined_bytes = summary.examined_bytes.saturating_add(other.examined_bytes);
    summary.failures = summary.failures.saturating_add(other.failures);
    summary.unknown = summary.unknown.saturating_add(other.unknown);
    summary.complete &= other.complete;
}
fn run_inner(
    root: &Path,
    request: &Request,
    predicate: &Predicate,
    token: &CancellationToken,
    emit: &mut (dyn FnMut(Event) -> Result<(), Error> + Send),
    native: Native<'_>,
) -> Result<Summary, Error> {
    if token.is_cancelled() {
        return Err(Error::Cancelled);
    }
    match predicate {
        Predicate::Refine {candidates,filter}=>refine(root,request,candidates,filter,token,emit,native),
        Predicate::Related {seed,relation,direction,depth,max_nodes}=>related(root,request,Traversal {seed,relation:*relation,direction:*direction,depth:*depth,max_nodes:*max_nodes},token,emit,native),
        Predicate::Any {clauses,..} if required(predicate)=>union(root,request,clauses,token,emit,native),
        Predicate::All {within:MatchScope::Entity,clauses}=>entity_conjunction(root,request,clauses,token,emit,native),
        Predicate::All {..}|Predicate::Not {..} if required(predicate)=>Err(Error::Unavailable("cross-provider conjunction/negation needs an explicit entity-bound candidate stage; file/node metadata cannot identify a resolved Rust entity".into())),
        _=>pump(root,request,predicate,token,emit,native),
    }
}

/// The producer sends only a bounded number of whole typed records. The receiving thread
/// never enters a Tokio native while running a synchronous provider callback, so native
/// per-candidate asynchronous operations can use the original native without nested entry.
fn pump(
    root: &Path,
    request: &Request,
    predicate: &Predicate,
    token: &CancellationToken,
    emit: &mut (dyn FnMut(Event) -> Result<(), Error> + Send),
    native: Native<'_>,
) -> Result<Summary, Error> {
    let mut selected = request.clone();
    selected.predicate = predicate.clone();
    if native.api_nomination_scope && is_api(predicate) {
        selected.sources = SourceSelector::default();
    }
    let child = token.child_token();
    std::thread::scope(|scope| {
        let (send, receive) = std::sync::mpsc::sync_channel(8);
        let producer_token = child.clone();
        let producer = scope.spawn(move || {
            let mut forward = |event| send.send(event).map_err(|_| Error::Cancelled);
            match &selected.predicate {
                Predicate::History(history) => native.block_on(providers::history(
                    root,
                    &selected,
                    history,
                    &producer_token,
                    &mut forward,
                )),
                predicate if global(predicate) => native.block_on(rust::global_search(
                    root,
                    &selected,
                    &producer_token,
                    &mut forward,
                )),
                _ => providers::run(root, &selected, &producer_token, &mut forward),
            }
        });
        let mut failure = None;
        let mut ids = BTreeMap::new();
        for mut event in receive {
            if failure.is_some() {
                continue;
            }
            // API/global producers may use local labels such as "api". Namespace those
            // labels by their actual original context before combining independent branches.
            match &mut event {
                Event::Source(source) => {
                    let id = source_key(source)?;
                    ids.insert(source.id.clone(), id.clone());
                    source.id = id.clone();
                    if !native
                        .source_ids
                        .lock()
                        .map_err(|_| {
                            Error::Unavailable("shared source-context owner panicked".into())
                        })?
                        .insert(id)
                    {
                        continue;
                    }
                }
                Event::Result(row) => {
                    if let Some(id) = ids.get(&row.source) {
                        row.source = id.clone();
                    }
                }
                Event::Diagnostic {
                    source: Some(source),
                    ..
                } => {
                    if let Some(id) = ids.get(source) {
                        *source = id.clone();
                    }
                }
                _ => {}
            }
            if let Err(error) = emit(event) {
                failure = Some(error);
                child.cancel();
            }
        }
        let result = producer.join().map_err(|_| {
            Error::Unavailable(
                "native candidate producer panicked; its scoped thread has settled".into(),
            )
        });
        if let Some(error) = failure {
            return Err(error);
        }
        result?
    })
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn file_identity(row: &SearchResult) -> Result<Option<String>, Error> {
    let locator = row.reference.as_ref().map(|reference| &reference.locator);
    match locator {
        Some(Locator::Source {
            path,
            source,
            guard,
            ..
        }) => {
            let path = if let Some(root) = guard.as_ref().and_then(|guard| guard.root.as_ref()) {
                NativePath::from(root.to_path_buf().join(path.to_path_buf()))
            } else {
                path.clone()
            };
            Ok(Some(serde_json::to_string(&(path, source))?))
        }
        Some(Locator::Semantic { path, .. }) => {
            Ok(Some(serde_json::to_string(&(path, SourceKind::Working))?))
        }
        _ => Ok(None),
    }
}

fn accepted(
    row: SearchResult,
    request: &Request,
    summary: &mut Summary,
    emit: &mut (dyn FnMut(Event) -> Result<(), Error> + Send),
    counted: &mut BTreeSet<String>,
) -> Result<(), Error> {
    if let Some(refinement) = &row.refinement {
        if refinement.truth == Truth::False {
            return Ok(());
        }
        if refinement.truth == Truth::Unknown {
            summary.complete = false;
            if visible(request, summary) {
                emit(Event::Result(row))?;
                summary.emitted += 1;
            }
            return Ok(());
        }
    }
    if request.answer.kind == AnswerKind::Files {
        let Some(file) = file_identity(&row)? else {
            summary.complete = false;
            summary.unknown += 1;
            return Ok(());
        };
        if !counted.insert(file) {
            return Ok(());
        }
    }
    let count = if request.answer.kind != AnswerKind::Count {
        1
    } else {
        match request.answer.count_unit {
            CountUnit::Matches => 1,
            CountUnit::Files => {
                if let Some(file) = file_identity(&row)? {
                    u64::from(counted.insert(file))
                } else {
                    summary.complete = false;
                    summary.unknown += 1;
                    0
                }
            }
            CountUnit::Lines => {
                if let (Some(range), Some(file)) = (&row.range, file_identity(&row)?) {
                    (range.line_start..=range.line_end)
                        .filter(|line| counted.insert(format!("{file}:{line}")))
                        .count() as u64
                } else {
                    summary.complete = false;
                    summary.unknown += 1;
                    0
                }
            }
            CountUnit::Entities => {
                if let Some(entity) = &row.entity {
                    u64::from(counted.insert(serde_json::to_string(entity)?))
                } else {
                    summary.complete = false;
                    summary.unknown += 1;
                    0
                }
            }
        }
    };
    summary.matches += count;
    if visible(request, summary) {
        emit(Event::Result(row))?;
        summary.emitted += 1;
    }
    Ok(())
}
fn stable_related(
    root: &Path,
    request: &Request,
    token: &CancellationToken,
    emit: &mut (dyn FnMut(Event) -> Result<(), Error> + Send),
    native: Native<'_>,
) -> Result<Summary, Error> {
    let mut selected = request.clone();
    selected.answer.kind = AnswerKind::Matches;
    selected.answer.count_unit = CountUnit::Matches;
    selected.answer.limit = None;
    selected.answer.order = ResultOrder::Arrival;
    selected.presentation = QueryPresentation::default();
    let mut rows = Vec::new();
    let mut collect = |event| match event {
        Event::Result(row) => {
            rows.push(row);
            Ok(())
        }
        other => emit(other),
    };
    let mut summary = run_inner(
        root,
        &selected,
        &selected.predicate,
        token,
        &mut collect,
        native,
    )?;
    drop(collect);
    let mut keyed = rows
        .into_iter()
        .map(|row| Ok((serde_json::to_string(&row.reference)?, row)))
        .collect::<Result<Vec<_>, Error>>()?;
    keyed.sort_by(|left, right| left.0.cmp(&right.0));
    summary.matches = 0;
    summary.emitted = 0;
    let mut counted = BTreeSet::new();
    let limit = limits(request).unwrap_or(usize::MAX);
    if keyed.len() > limit {
        summary.complete = false;
        summary.stop = StopReason::Limit;
    }
    for (_, row) in keyed.into_iter().take(limit) {
        accepted(row, request, &mut summary, emit, &mut counted)?;
    }
    Ok(summary)
}
fn same_entity(left: &EntityIdentity, right: &EntityIdentity) -> Truth {
    if left.session_invocation != right.session_invocation
        || left.configuration != right.configuration
        || left.generation != right.generation
    {
        Truth::Unknown
    } else if left == right {
        Truth::True
    } else {
        Truth::False
    }
}
fn retain_unknown(
    mut row: SearchResult,
    error: &Error,
    request: &Request,
    summary: &mut Summary,
    emit: &mut (dyn FnMut(Event) -> Result<(), Error> + Send),
) -> Result<(), Error> {
    summary.unknown += 1;
    summary.complete = false;
    if let Some(failure) = error.native_refusal() {
        emit(Event::NativeFailure {
            failure: Box::new(failure.clone()),
        })?;
    }
    let original = QueryDiagnostic {
        code: "search.refinement_unknown".into(),
        message: error.to_string(),
        source: Some(row.source.clone()),
    };
    let previous = row.refinement.take();
    let evidence = previous.as_ref().and_then(|fact| fact.evidence.clone());
    let diagnostic = previous
        .as_ref()
        .and_then(|fact| fact.diagnostic.clone())
        .unwrap_or_else(|| original.clone());
    let original_native = error
        .native_refusal()
        .cloned()
        .or_else(|| previous.and_then(|fact| fact.native));
    row.refinement = Some(RefinementFact {
        truth: Truth::Unknown,
        diagnostic: Some(diagnostic),
        native: original_native,
        evidence,
    });
    emit(Event::Diagnostic {
        code: original.code,
        message: original.message,
        source: original.source,
    })?;
    if visible(request, summary) {
        emit(Event::Result(row))?;
        summary.emitted += 1;
    }
    Ok(())
}
fn refine(
    root: &Path,
    request: &Request,
    candidates: &Predicate,
    filter: &Refinement,
    token: &CancellationToken,
    emit: &mut (dyn FnMut(Event) -> Result<(), Error> + Send),
    native: Native<'_>,
) -> Result<Summary, Error> {
    let mut summary = Summary {
        complete: true,
        ..Default::default()
    };
    let mut counted = BTreeSet::new();
    let mut useful = 0u64;
    let scope = if contains_api(candidates) {
        nomination_scope(root, &request.sources, token)?
    } else {
        None
    };
    let scan = token.child_token();
    let mut stopped = false;
    let mut candidate_request = request.clone();
    candidate_request.answer = AnswerDemand {
        limit: None,
        kind: AnswerKind::Matches,
        count_unit: CountUnit::Matches,
        ..request.answer.clone()
    };
    candidate_request.presentation = QueryPresentation::default();
    let expected = match filter {
        Refinement::ResolvesTo { target }
        | Refinement::UsedAt {
            destination: target,
        } => native
            .block_on(rust::resolve_entity_with_context(
                root,
                &target.locator,
                token,
                native.resolution,
            ))
            .map(Some),
        Refinement::NativePredicate { .. } | Refinement::ImportableAt { .. } => Ok(None),
    };
    if let Err(error) = &expected {
        if !unavailable(error) {
            return Err(match expected {
                Err(error) => error,
                Ok(_) => unreachable!(),
            });
        }
    }
    if let Ok(Some(target)) = &expected {
        resolved_context(target, emit, native)?;
    }
    let mut consume = |event| -> Result<(), Error> {
        let Event::Result(mut row) = event else {
            return emit(event);
        };
        if stopped || token.is_cancelled() {
            return Ok(());
        }
        let evaluated: Result<(), Error> = if let Err(error) = &expected {
            retain_unknown(row, error, request, &mut summary, emit)?;
            useful += 1;
            Ok(())
        } else {
            let resolved = match &row.reference {
                Some(reference) => native.block_on(rust::resolve_entity_with_context(
                    root,
                    &reference.locator,
                    token,
                    native.resolution,
                )),
                None => Err(Error::Unavailable(
                    "candidate has no native address for contextual refinement".into(),
                )),
            };
            match resolved {
                Err(error) if unavailable(&error) => {
                    retain_unknown(row, &error, request, &mut summary, emit)?;
                    useful += 1;
                    Ok(())
                }
                Err(error) => {
                    let cancelled = error.disposition() == FailureDisposition::Cancelled;
                    failed(&mut summary, emit, error)?;
                    if cancelled {
                        stopped = true;
                        scan.cancel();
                    }
                    Ok(())
                }
                Ok(resolved) => {
                    resolved_context(&resolved, emit, native)?;
                    if row
                        .reference
                        .as_ref()
                        .is_some_and(|reference| is_api_locator(&reference.locator))
                        && !nomination_allowed(&scope, &resolved.locator)?
                    {
                        return Ok(());
                    }
                    let producer = row.refinement.take();
                    let producer_truth = producer.as_ref().map_or(Truth::True, |fact| fact.truth);
                    let mut evidence = None;
                    let outcome = match filter {
                        Refinement::ResolvesTo { .. } | Refinement::UsedAt { .. } => Ok(expected
                            .as_ref()
                            .ok()
                            .and_then(Option::as_ref)
                            .map_or(Truth::Unknown, |target| {
                                same_entity(&target.identity, &resolved.identity)
                            })),
                        Refinement::ImportableAt { destination } => native
                            .block_on(rust::importability(
                                root,
                                &resolved,
                                &destination.locator,
                                token,
                                native.resolution,
                            ))
                            .map(|(truth, facts)| {
                                evidence = facts["sections"]
                                    .as_array()
                                    .and_then(|sections| {
                                        sections
                                            .iter()
                                            .find(|section| section["category"] == "importability")
                                    })
                                    .map(|section| section["evidence"].clone());
                                truth
                            }),
                        Refinement::NativePredicate { text } => native
                            .block_on(rust::semantic_operation_cancellable(
                                root,
                                &resolved.locator,
                                crate::semantic::Kind::Predicate { text: text.clone() },
                                token,
                            ))
                            .map(|answer| predicate_truth(&answer)),
                    }
                    .map(|truth| producer_truth.and(truth));
                    row.entity = Some(resolved.identity);
                    if row.candidate_reference.is_none() {
                        row.candidate_reference = row.reference.take();
                    }
                    row.reference = Some(ResultReference::new(resolved.locator));
                    row.refinement = Some(RefinementFact {
                        truth: Truth::Unknown,
                        diagnostic: producer.as_ref().and_then(|fact| fact.diagnostic.clone()),
                        native: producer.as_ref().and_then(|fact| fact.native.clone()),
                        evidence: evidence.or_else(|| producer.and_then(|fact| fact.evidence)),
                    });
                    match outcome {
                        Ok(Truth::True) => {
                            if let Some(fact) = &mut row.refinement {
                                fact.truth = Truth::True;
                            }
                            let before = summary.matches;
                            accepted(row, request, &mut summary, emit, &mut counted)?;
                            if summary.matches > before {
                                useful += 1;
                            }
                        }
                        Ok(Truth::False) => {}
                        Ok(Truth::Unknown) => {
                            retain_unknown(row,&Error::Unavailable("native context did not prove this predicate/importability; changed world, different selection and not-proven are not contradiction".into()),request,&mut summary,emit)?;
                            useful += 1;
                        }
                        Err(error) if unavailable(&error) => {
                            retain_unknown(row, &error, request, &mut summary, emit)?;
                            useful += 1;
                        }
                        Err(error) => {
                            let cancelled = error.disposition() == FailureDisposition::Cancelled;
                            failed(&mut summary, emit, error)?;
                            if cancelled {
                                stopped = true;
                                scan.cancel();
                            }
                        }
                    }
                    Ok(())
                }
            }
        };
        evaluated?;
        let reached = match request.answer.kind {
            AnswerKind::Count => false,
            AnswerKind::Exists => summary.matches > 0,
            _ => limits(request).is_some_and(|limit| useful >= limit as u64),
        };
        if reached {
            stopped = true;
            scan.cancel();
            summary.complete = false;
            summary.stop = StopReason::Limit;
        }
        Ok(())
    };
    let examined = run_inner(
        root,
        &candidate_request,
        candidates,
        &scan,
        &mut consume,
        Native {
            api_nomination_scope: true,
            ..native
        },
    );
    drop(consume);
    match examined {
        Ok(work) => {
            merge_work(&mut summary, &work);
            if stopped && !summary.cancellation_requested {
                summary.stop = StopReason::Limit;
            }
        }
        Err(error) if error.disposition() == FailureDisposition::Cancelled && stopped => {}
        Err(error) => failed(&mut summary, emit, error)?,
    }
    Ok(summary)
}
fn predicate_truth(answer: &serde_json::Value) -> Truth {
    for section in answer["sections"].as_array().into_iter().flatten() {
        if section["category"] != "predicate_evaluation" {
            continue;
        }
        if let Some(value) = section["evidence"]
            .as_bool()
            .or_else(|| section["evidence"]["holds"].as_bool())
        {
            return if value { Truth::True } else { Truth::False };
        }
    }
    Truth::Unknown
}

fn union(
    root: &Path,
    request: &Request,
    clauses: &[Predicate],
    token: &CancellationToken,
    emit: &mut (dyn FnMut(Event) -> Result<(), Error> + Send),
    native: Native<'_>,
) -> Result<Summary, Error> {
    let mut summary = Summary {
        complete: true,
        ..Default::default()
    };
    let mut counted = BTreeSet::new();
    let mut seen = BTreeSet::new();
    let mut useful = 0u64;
    for clause in clauses {
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }
        let child = token.child_token();
        let mut stopped = false;
        let mut added = Summary {
            complete: true,
            ..Default::default()
        };
        let mut rendered = request.clone();
        rendered.presentation.budget = request.presentation.budget.map(|budget| {
            budget.saturating_sub(usize::try_from(summary.emitted).unwrap_or(usize::MAX))
        });
        let mut selected = request.clone();
        selected.answer.kind = AnswerKind::Matches;
        selected.answer.count_unit = CountUnit::Matches;
        selected.answer.limit = None;
        selected.presentation = QueryPresentation::default();
        let mut consume = |event| -> Result<(), Error> {
            match event {
                Event::Result(row) => {
                    if stopped {
                        return Ok(());
                    }
                    let key = row
                        .reference
                        .as_ref()
                        .map(ResultReference::encode)
                        .transpose()?
                        .unwrap_or(serde_json::to_string(&row)?);
                    if seen.insert(key) {
                        let unknown = row
                            .refinement
                            .as_ref()
                            .is_some_and(|fact| fact.truth == Truth::Unknown);
                        let before = added.matches;
                        accepted(row, &rendered, &mut added, emit, &mut counted)?;
                        if unknown || added.matches > before {
                            useful += 1;
                        }
                    }
                    let reached = match request.answer.kind {
                        AnswerKind::Count => false,
                        AnswerKind::Exists => summary.matches + added.matches > 0,
                        _ => limits(request).is_some_and(|limit| useful >= limit as u64),
                    };
                    if reached {
                        stopped = true;
                        child.cancel();
                    }
                    Ok(())
                }
                other => emit(other),
            }
        };
        let result = run_inner(root, &selected, clause, &child, &mut consume, native);
        drop(consume);
        summary.matches += added.matches;
        summary.emitted += added.emitted;
        merge_work(&mut summary, &added);
        match result {
            Ok(work) => merge_work(&mut summary, &work),
            Err(Error::Cancelled) if stopped => {}
            Err(error) => failed(&mut summary, emit, error)?,
        }
        if stopped {
            summary.complete = false;
            summary.stop = StopReason::Limit;
            break;
        }
    }
    Ok(summary)
}

fn entity_conjunction(
    root: &Path,
    request: &Request,
    clauses: &[Predicate],
    token: &CancellationToken,
    emit: &mut (dyn FnMut(Event) -> Result<(), Error> + Send),
    native: Native<'_>,
) -> Result<Summary, Error> {
    let Some((first, remaining)) = clauses.split_first() else {
        return Err(Error::Invalid(
            "entity conjunction requires a candidate clause".into(),
        ));
    };
    let mut summary = Summary {
        complete: true,
        ..Default::default()
    };
    let mut counted = BTreeSet::new();
    let mut useful = 0u64;
    let child = token.child_token();
    let mut stopped = false;
    let mut seen = BTreeSet::new();
    let scope = if contains_api(first) {
        nomination_scope(root, &request.sources, token)?
    } else {
        None
    };
    let mut selected = request.clone();
    selected.answer.kind = AnswerKind::Matches;
    selected.answer.count_unit = CountUnit::Matches;
    selected.answer.limit = None;
    selected.presentation = QueryPresentation::default();
    let mut consume = |event| -> Result<(), Error> {
        match event {
            Event::Result(mut row) => {
                if stopped {
                    return Ok(());
                }
                let Some(reference) = &row.reference else {
                    failed(
                        &mut summary,
                        emit,
                        Error::Unavailable(
                            "entity conjunction candidate has no native identity address".into(),
                        ),
                    )?;
                    return Ok(());
                };
                let entity = match native.block_on(rust::resolve_entity_with_context(
                    root,
                    &reference.locator,
                    token,
                    native.resolution,
                )) {
                    Ok(entity) => entity,
                    Err(error) if unavailable(&error) => {
                        retain_unknown(row, &error, request, &mut summary, emit)?;
                        useful += 1;
                        if matches!(request.answer.kind, AnswerKind::Matches | AnswerKind::Files)
                            && limits(request).is_some_and(|limit| useful >= limit as u64)
                        {
                            stopped = true;
                            child.cancel();
                            summary.stop = StopReason::Limit;
                        }
                        return Ok(());
                    }
                    Err(error) => {
                        failed(&mut summary, emit, error)?;
                        return Ok(());
                    }
                };
                resolved_context(&entity, emit, native)?;
                if row
                    .reference
                    .as_ref()
                    .is_some_and(|reference| is_api_locator(&reference.locator))
                    && !nomination_allowed(&scope, &entity.locator)?
                {
                    return Ok(());
                }
                let mut truth = Truth::True;
                for clause in remaining {
                    truth = truth.and(entity_membership(
                        root,
                        request,
                        clause,
                        &entity.identity,
                        token,
                        emit,
                        native,
                    )?);
                    if truth == Truth::False {
                        break;
                    }
                }
                if truth == Truth::True && seen.insert(entity.identity.clone()) {
                    row.entity = Some(entity.identity);
                    if row.candidate_reference.is_none() {
                        row.candidate_reference = row.reference.take();
                    }
                    row.reference = Some(ResultReference::new(entity.locator));
                    row.refinement = Some(RefinementFact {
                        truth: Truth::True,
                        diagnostic: None,
                        native: None,
                        evidence: None,
                    });
                    accepted(row, request, &mut summary, emit, &mut counted)?;
                    useful += 1;
                } else if truth == Truth::Unknown {
                    row.entity = Some(entity.identity);
                    if row.candidate_reference.is_none() {
                        row.candidate_reference = row.reference.take();
                    }
                    row.reference = Some(ResultReference::new(entity.locator));
                    retain_unknown(row,&Error::Unavailable("an entity-scoped clause lacked a complete negative or native identity witness".into()),request,&mut summary,emit)?;
                    useful += 1;
                }
                let reached = match request.answer.kind {
                    AnswerKind::Count => false,
                    AnswerKind::Exists => summary.matches > 0,
                    _ => limits(request).is_some_and(|limit| useful >= limit as u64),
                };
                if reached {
                    stopped = true;
                    child.cancel();
                    summary.complete = false;
                    summary.stop = StopReason::Limit;
                }
                Ok(())
            }
            other => emit(other),
        }
    };
    let work = run_inner(
        root,
        &selected,
        first,
        &child,
        &mut consume,
        Native {
            api_nomination_scope: true,
            ..native
        },
    );
    drop(consume);
    match work {
        Ok(work) => merge_work(&mut summary, &work),
        Err(Error::Cancelled) if stopped => {}
        Err(error) => failed(&mut summary, emit, error)?,
    }
    Ok(summary)
}
fn entity_membership(
    root: &Path,
    request: &Request,
    predicate: &Predicate,
    identity: &EntityIdentity,
    token: &CancellationToken,
    emit: &mut (dyn FnMut(Event) -> Result<(), Error> + Send),
    native: Native<'_>,
) -> Result<Truth, Error> {
    if let Predicate::Not {
        within: MatchScope::Entity,
        clause,
    } = predicate
    {
        return entity_membership(root, request, clause, identity, token, emit, native)
            .map(Truth::negate);
    }
    let key = serde_json::to_string(&(&request.sources, predicate))?;
    let existing = native
        .membership
        .lock()
        .map_err(|_| Error::Unavailable("native membership owner panicked".into()))?
        .get(&key)
        .cloned();
    if let Some(index) = existing {
        return Ok(index.contains(identity));
    }
    let mut selected = request.clone();
    selected.answer.kind = AnswerKind::Matches;
    selected.answer.count_unit = CountUnit::Matches;
    selected.answer.limit = None;
    selected.presentation = QueryPresentation::default();
    let mut index = MembershipIndex {
        entities: BTreeSet::new(),
        complete: true,
    };
    let scope = if contains_api(predicate) {
        nomination_scope(root, &request.sources, token)?
    } else {
        None
    };
    let mut consume = |event| -> Result<(), Error> {
        match event {
            Event::Result(row) => {
                if let Some(reference) = row.reference {
                    match native.block_on(rust::resolve_entity_with_context(
                        root,
                        &reference.locator,
                        token,
                        native.resolution,
                    )) {
                        Ok(entity) => {
                            resolved_context(&entity, emit, native)?;
                            if !is_api_locator(&reference.locator)
                                || nomination_allowed(&scope, &entity.locator)?
                            {
                                index.entities.insert(entity.identity);
                            }
                        }
                        Err(error) if unavailable(&error) => {
                            index.complete = false;
                            diagnostic(emit, &error)?;
                        }
                        Err(error) => return Err(error),
                    }
                } else {
                    index.complete = false;
                }
            }
            Event::Diagnostic {
                code,
                message,
                source,
            } => emit(Event::Diagnostic {
                code,
                message,
                source,
            })?,
            Event::Source(source) => emit(Event::Source(source))?,
            _ => {}
        }
        Ok(())
    };
    let result = run_inner(
        root,
        &selected,
        predicate,
        token,
        &mut consume,
        Native {
            api_nomination_scope: true,
            ..native
        },
    );
    drop(consume);
    match result {
        Ok(summary) => {
            index.complete &= summary.complete && summary.unknown == 0 && summary.failures == 0;
            let mut work = native
                .work
                .lock()
                .map_err(|_| Error::Unavailable("native clause-work owner panicked".into()))?;
            merge_work(&mut work, &summary);
        }
        Err(error) if unavailable(&error) => {
            index.complete = false;
            diagnostic(emit, &error)?;
        }
        Err(error) => return Err(error),
    }
    let truth = index.contains(identity);
    native
        .membership
        .lock()
        .map_err(|_| Error::Unavailable("native membership owner panicked".into()))?
        .insert(key, std::sync::Arc::new(index));
    Ok(truth)
}

struct Traversal<'a> {
    seed: &'a ResultReference,
    relation: Relationship,
    direction: Direction,
    depth: usize,
    max_nodes: usize,
}
fn native_position_key(locator: &Locator) -> Result<String, Error> {
    if let Locator::Semantic {
        project,
        path,
        line,
        column,
        session_invocation,
        ..
    } = locator
    {
        // Generation is fenced once for the traversal below. Source hashes and lazy-action
        // fingerprints are guards, not additional native locations to admit a second time.
        serde_json::to_string(&(project, path, line, column, session_invocation))
            .map_err(Error::from)
    } else {
        Err(Error::Invalid(
            "relationship traversal requires a native guarded source position".into(),
        ))
    }
}
fn related(
    root: &Path,
    request: &Request,
    traversal: Traversal<'_>,
    token: &CancellationToken,
    emit: &mut (dyn FnMut(Event) -> Result<(), Error> + Send),
    native: Native<'_>,
) -> Result<Summary, Error> {
    let Traversal {
        seed,
        relation,
        direction,
        depth,
        max_nodes,
    } = traversal;
    if request.sources.kind != SourceKind::Working {
        return Err(Error::Unavailable("native relationship traversal requires a working source universe; index/revision selection cannot be reinterpreted as current semantics".into()));
    }
    let allowed = providers::select(root, &request.sources, token)?
        .into_iter()
        .map(|source| root.join(source.path()).canonicalize().map_err(Error::from))
        .collect::<Result<BTreeSet<_>, Error>>()?;
    let mut summary = Summary {
        complete: true,
        ..Default::default()
    };
    let mut counted = BTreeSet::new();
    let selected = native.block_on(rust::source_semantic_with_context(
        root,
        &seed.locator,
        token,
        native.resolution,
    ))?;
    let Locator::Semantic { path, .. } = &selected else {
        return Err(Error::Invalid(
            "native traversal seed has no selected source position".into(),
        ));
    };
    if !allowed.contains(&path.to_path_buf().canonicalize()?) {
        return Err(Error::Invalid("relationship seed is outside the requested source selection; select its source path/package/type explicitly".into()));
    }
    let mut world = if let Locator::Semantic { generation, .. } = &selected {
        generation.clone()
    } else {
        None
    };
    let mut queue = VecDeque::from([(selected.clone(), 0usize)]);
    let mut visited = BTreeSet::new();
    let mut emitted = BTreeSet::new();
    visited.insert(native_position_key(&selected)?);
    while let Some((locator, distance)) = queue.pop_front() {
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }
        if distance >= depth {
            continue;
        }
        let result = match native.block_on(rust::relationship_with_context(
            root,
            &locator,
            relation,
            direction,
            max_nodes.saturating_add(1),
            token,
            native.resolution,
        )) {
            Ok(result) => result,
            Err(error) => {
                failed(&mut summary, emit, error)?;
                continue;
            }
        };
        summary.complete &= result.complete;
        let mut source_ids = BTreeMap::new();
        for source in result.sources {
            let original = source.id.clone();
            source_ids.insert(original, emit_source(source, emit, native)?);
        }
        for diagnostic in result.diagnostics {
            summary.unknown += 1;
            summary.complete = false;
            emit(Event::Diagnostic {
                code: diagnostic.code,
                message: diagnostic.message,
                source: diagnostic
                    .source
                    .map(|source| source_ids.get(&source).cloned().unwrap_or(source)),
            })?;
        }
        for mut row in result.rows {
            if let Some(id) = source_ids.get(&row.source) {
                row.source = id.clone();
            }
            let Some(reference) = &row.reference else {
                failed(
                    &mut summary,
                    emit,
                    Error::Unavailable(
                        "native relationship has no guarded destination address".into(),
                    ),
                )?;
                continue;
            };
            let Locator::Semantic { path, .. } = &reference.locator else {
                failed(
                    &mut summary,
                    emit,
                    Error::Unavailable(
                        "native relationship destination has no guarded working source".into(),
                    ),
                )?;
                continue;
            };
            if !allowed.contains(&path.to_path_buf().canonicalize()?) {
                continue;
            }
            if let Locator::Semantic {
                generation: Some(generation),
                ..
            } = &reference.locator
            {
                if world
                    .as_ref()
                    .is_some_and(|original| original != generation)
                {
                    failed(
                        &mut summary,
                        emit,
                        Error::Stale(
                            "native relationship traversal crossed semantic world generations"
                                .into(),
                        ),
                    )?;
                    return Ok(summary);
                }
                world.get_or_insert_with(|| generation.clone());
            }
            let key = native_position_key(&reference.locator)?;
            if !visited.contains(&key) {
                if visited.len() >= max_nodes {
                    summary.complete = false;
                    summary.stop = StopReason::Limit;
                    return Ok(summary);
                }
                visited.insert(key.clone());
                queue.push_back((reference.locator.clone(), distance + 1));
            }
            if emitted.insert(key.clone()) {
                accepted(row.clone(), request, &mut summary, emit, &mut counted)?;
            }
            if limits(request).is_some_and(|limit| summary.matches >= limit as u64) {
                summary.complete = false;
                summary.stop = StopReason::Limit;
                return Ok(summary);
            }
        }
    }
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unrelated_same_spelled_declarations_never_share_entity_identity() {
        let first = EntityIdentity {
            session_invocation: "session".into(),
            configuration: "features-a".into(),
            generation: 1,
            declaration: std::path::PathBuf::from("source.rs").into(),
            byte_start: 10,
            byte_end: 20,
        };
        let mut second = first.clone();
        second.byte_start = 30;
        assert_ne!(first, second);
        second = first.clone();
        second.configuration = "features-b".into();
        assert_ne!(first, second);
        second = first.clone();
        second.session_invocation = "replacement-session".into();
        assert_ne!(first, second);
    }
    #[test]
    fn unknown_predicate_evidence_is_not_a_negative_or_hover_inference() {
        assert_eq!(
            predicate_truth(
                &serde_json::json!({"sections":[{"category":"hover","evidence":"true"}]})
            ),
            Truth::Unknown
        );
        assert_eq!(
            predicate_truth(
                &serde_json::json!({"sections":[{"category":"predicate_evaluation","evidence":{"holds":true}}]})
            ),
            Truth::True
        );
        assert_eq!(
            predicate_truth(
                &serde_json::json!({"sections":[{"category":"predicate_evaluation","evidence":{"state":"not_proven"}}]})
            ),
            Truth::Unknown
        );
    }
    #[test]
    fn unavailable_refinement_retains_candidates_and_count_and_exists_do_not_stop_at_twenty() {
        let root = tempfile::tempdir().unwrap();
        let git = std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(root.path())
            .output()
            .unwrap();
        assert!(git.status.success());
        let source = (0..30)
            .map(|index| format!("fn needle{index}() {{}}\n"))
            .collect::<String>();
        let path = root.path().join("source.rs");
        std::fs::write(&path, &source).unwrap();
        let input = providers::acquire(
            root.path(),
            &crate::source_read::SourceRequest::Working { path },
        )
        .unwrap();
        let target = ResultReference::new(Locator::Source {
            path: std::path::PathBuf::from("source.rs").into(),
            source: SourceKind::Working,
            range: Some(MatchRange {
                byte_start: 0,
                byte_end: source.lines().next().unwrap().len(),
                line_start: 1,
                line_end: 1,
            }),
            guard: input.guard,
        });
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        for (kind, unknown, emitted) in [
            (AnswerKind::Matches, 20, 20),
            (AnswerKind::Count, 30, 0),
            (AnswerKind::Exists, 30, 0),
        ] {
            let mut request = Request {
                predicate: Predicate::Refine {
                    candidates: Box::new(Predicate::Symbol(SymbolQuery {
                        name: "needle".into(),
                        ..Default::default()
                    })),
                    filter: Refinement::ResolvesTo {
                        target: target.clone(),
                    },
                },
                ..Default::default()
            };
            request.sources.paths.push("source.rs".into());
            request.answer.kind = kind;
            let mut records = Vec::new();
            let result = run(
                root.path(),
                &request,
                &CancellationToken::new(),
                &mut |event| {
                    records.push(event);
                    Ok(())
                },
                runtime.handle(),
            )
            .unwrap();
            assert_eq!(result.matches, 0);
            assert_eq!(result.unknown, unknown);
            assert_eq!(result.emitted, emitted);
            assert!(!result.complete);
            let witnesses = records
                .iter()
                .filter_map(|event| {
                    if let Event::Result(row) = event {
                        Some(row)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            assert_eq!(witnesses.len(), emitted as usize);
            assert!(witnesses.iter().all(|row| {
                row.refinement
                    .as_ref()
                    .is_some_and(|fact| fact.truth == Truth::Unknown)
                    && row.reference.is_some()
                    && row
                        .name
                        .as_ref()
                        .is_some_and(|name| name.contains("needle"))
            }));
            assert!(matches!(result.count, Count::AtLeast(0)));
        }
    }
    #[test]
    fn typed_remote_refusals_keep_original_data_and_cancellation_classification() {
        for (state, disposition) in [
            (
                crate::semantic::FailureState::Unsupported,
                FailureDisposition::Unavailable,
            ),
            (
                crate::semantic::FailureState::Cancelled,
                FailureDisposition::Cancelled,
            ),
        ] {
            let original = crate::semantic::Failure {
                state,
                message: "native owner refusal".into(),
                backend_code: Some(-32800),
                backend_data: Some(serde_json::json!({"cpu_settled":true,"original":123})),
            };
            let error =
                Error::Semantic(Box::new(crate::semantic::Error::Refusal(original.clone())));
            assert_eq!(error.disposition(), disposition);
            let mut summary = Summary::default();
            let mut records = Vec::new();
            failed(
                &mut summary,
                &mut |event| {
                    records.push(event);
                    Ok(())
                },
                error,
            )
            .unwrap();
            assert_eq!(summary.failures, 0);
            assert!(!summary.complete);
            assert!(records.iter().any(|record|matches!(record,Event::NativeFailure{failure} if failure.backend_data==original.backend_data&&failure.backend_code==original.backend_code)));
            if disposition == FailureDisposition::Cancelled {
                assert!(summary.cancellation_requested);
                assert_eq!(summary.stop, StopReason::Cancelled);
            } else {
                assert_eq!(summary.unknown, 1);
            }
        }
        let error = Error::Semantic(Box::new(crate::semantic::Error::Refusal(
            crate::semantic::Failure {
                state: crate::semantic::FailureState::Unavailable,
                message: "selected backend could not prove enrichment".into(),
                backend_code: Some(91),
                backend_data: Some(serde_json::json!({"original":true})),
            },
        )));
        let row: SearchResult = serde_json::from_value(
            serde_json::json!({"source":"candidate","kind":"api","range":null,"reference":null}),
        )
        .unwrap();
        let request = Request {
            answer: AnswerDemand {
                kind: AnswerKind::Count,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut summary = Summary::default();
        let mut records = Vec::new();
        retain_unknown(row, &error, &request, &mut summary, &mut |event| {
            records.push(event);
            Ok(())
        })
        .unwrap();
        assert_eq!(summary.emitted, 0);
        assert!(records.iter().any(|record|matches!(record,Event::NativeFailure{failure} if failure.backend_code==Some(91)&&failure.backend_data==Some(serde_json::json!({"original":true})))));
    }
    #[test]
    fn mixed_source_and_absent_semantic_alternatives_retain_witnesses_and_unknown_in_either_order()
    {
        let root = tempfile::tempdir().unwrap();
        let initialized = std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(root.path())
            .output()
            .unwrap();
        assert!(initialized.status.success(), "{initialized:?}");
        std::fs::write(root.path().join("source.rs"), "fn needle() {}\n").unwrap();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let source = Predicate::Text(TextQuery {
            patterns: vec!["needle".into()],
            literal: true,
            ..Default::default()
        });
        let semantic = Predicate::Symbol(SymbolQuery {
            name: "needle".into(),
            provider: DiscoveryProvider::Semantic,
            ..Default::default()
        });
        for clauses in [
            vec![source.clone(), semantic.clone()],
            vec![semantic.clone(), source.clone()],
        ] {
            let mut request = Request {
                predicate: Predicate::Any {
                    within: MatchScope::File,
                    clauses,
                },
                ..Default::default()
            };
            request.sources.paths.push("source.rs".into());
            let mut events = Vec::new();
            let result = run(
                root.path(),
                &request,
                &CancellationToken::new(),
                &mut |event| {
                    events.push(event);
                    Ok(())
                },
                runtime.handle(),
            )
            .unwrap();
            assert_eq!(result.matches, 1);
            assert!(!result.complete);
            assert_eq!(result.unknown, 1, "{events:?}");
            assert!(
                events.iter().any(
                    |event| matches!(event,Event::Result(row) if row.context.contains("needle"))
                )
            );
            assert!(events.iter().any(|event|matches!(event,Event::Diagnostic {code,..} if code=="search.composition_unavailable")));
            let sources = events
                .iter()
                .filter_map(|event| {
                    if let Event::Source(source) = event {
                        Some(source)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            assert_eq!(sources.len(), 1);
            assert_eq!(sources[0].origin, Some(SourceKind::Working));
        }
    }
    #[test]
    fn consumer_failure_settles_the_real_scoped_producer_and_keeps_the_original_error() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("source.rs"), "needle\n".repeat(4096)).unwrap();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let mut request = Request {
            predicate: Predicate::Any {
                within: MatchScope::File,
                clauses: vec![Predicate::Text(TextQuery {
                    patterns: vec!["needle".into()],
                    ..Default::default()
                })],
            },
            ..Default::default()
        };
        request.sources.paths.push("source.rs".into());
        let mut calls = 0;
        let result = run(
            root.path(),
            &request,
            &CancellationToken::new(),
            &mut |_event| {
                calls += 1;
                if calls == 1 {
                    Err(Error::Io(std::io::ErrorKind::BrokenPipe.into()))
                } else {
                    Err(Error::Invalid("consumer is already closed".into()))
                }
            },
            runtime.handle(),
        );
        assert!(
            matches!(result,Err(Error::Io(error)) if error.kind()==std::io::ErrorKind::BrokenPipe)
        );
        std::fs::remove_file(root.path().join("source.rs")).unwrap();
    }
}
