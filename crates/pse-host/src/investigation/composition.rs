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
    let mut summary = if request.answer.order == ResultOrder::Stable {
        stable_related(root, request, token, emit, runtime)?
    } else {
        run_inner(root, request, &request.predicate, token, emit, runtime)?
    };
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
}
fn limits(request: &Request) -> Option<usize> {
    if request.answer.kind == AnswerKind::Exists {
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
    matches!(error, Error::Unavailable(_))
        || matches!(error,Error::Semantic(error) if matches!(error.as_ref(),crate::semantic::Error::Unavailable(_)))
}
fn failed(
    summary: &mut Summary,
    emit: &mut (dyn FnMut(Event) -> Result<(), Error> + Send),
    error: Error,
) -> Result<(), Error> {
    if matches!(&error, Error::Cancelled)
        || matches!(&error,Error::Semantic(error) if matches!(error.as_ref(),crate::semantic::Error::Cancelled))
    {
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
    runtime: &tokio::runtime::Handle,
) -> Result<Summary, Error> {
    if token.is_cancelled() {
        return Err(Error::Cancelled);
    }
    match predicate {
        Predicate::Refine {candidates,filter}=>refine(root,request,candidates,filter,token,emit,runtime),
        Predicate::Related {seed,relation,direction,depth,max_nodes}=>related(root,request,Traversal {seed,relation:*relation,direction:*direction,depth:*depth,max_nodes:*max_nodes},token,emit,runtime),
        Predicate::Any {clauses,..} if required(predicate)=>union(root,request,clauses,token,emit,runtime),
        Predicate::All {within:MatchScope::Entity,clauses}=>entity_conjunction(root,request,clauses,token,emit,runtime),
        Predicate::All {..}|Predicate::Not {..} if required(predicate)=>Err(Error::Unavailable("cross-provider conjunction/negation needs an explicit entity-bound candidate stage; file/node metadata cannot identify a resolved Rust entity".into())),
        _=>pump(root,request,predicate,token,emit,runtime),
    }
}

/// The producer sends only a bounded number of whole typed records. The receiving thread
/// never enters a Tokio runtime while running a synchronous provider callback, so native
/// per-candidate asynchronous operations can use the original runtime without nested entry.
fn pump(
    root: &Path,
    request: &Request,
    predicate: &Predicate,
    token: &CancellationToken,
    emit: &mut (dyn FnMut(Event) -> Result<(), Error> + Send),
    runtime: &tokio::runtime::Handle,
) -> Result<Summary, Error> {
    let mut selected = request.clone();
    selected.predicate = predicate.clone();
    let child = token.child_token();
    std::thread::scope(|scope| {
        let (send, receive) = std::sync::mpsc::sync_channel(8);
        let producer_token = child.clone();
        let producer = scope.spawn(move || {
            let mut forward = |event| send.send(event).map_err(|_| Error::Cancelled);
            match &selected.predicate {
                Predicate::History(history) => runtime.block_on(providers::history(
                    root,
                    &selected,
                    history,
                    &producer_token,
                    &mut forward,
                )),
                predicate if global(predicate) => runtime.block_on(rust::global_search(
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
                    use sha2::Digest as _;
                    let id = format!(
                        "composition:{}",
                        hex(&sha2::Sha256::digest(serde_json::to_vec(source)?))
                    );
                    ids.insert(source.id.clone(), id.clone());
                    source.id = id;
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
    runtime: &tokio::runtime::Handle,
) -> Result<Summary, Error> {
    let mut selected = request.clone();
    selected.answer.kind = AnswerKind::Matches;
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
        runtime,
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
fn refine(
    root: &Path,
    request: &Request,
    candidates: &Predicate,
    filter: &Refinement,
    token: &CancellationToken,
    emit: &mut (dyn FnMut(Event) -> Result<(), Error> + Send),
    runtime: &tokio::runtime::Handle,
) -> Result<Summary, Error> {
    let mut summary = Summary {
        complete: true,
        ..Default::default()
    };
    let mut counted = BTreeSet::new();
    let scan = token.child_token();
    let mut stopped = false;
    let mut candidate_request = request.clone();
    candidate_request.answer = AnswerDemand {
        limit: None,
        kind: AnswerKind::Matches,
        ..request.answer.clone()
    };
    candidate_request.presentation = QueryPresentation::default();
    let expected = match filter {
        Refinement::ResolvesTo { target }
        | Refinement::UsedAt {
            destination: target,
        }
        | Refinement::ImportableAt {
            destination: target,
        } => Some(runtime.block_on(rust::resolve_entity(root, &target.locator, token))?),
        Refinement::NativePredicate { .. } => None,
    };
    let mut consume = |event| -> Result<(), Error> {
        match event {
            Event::Result(mut row) => {
                if stopped || token.is_cancelled() {
                    return Ok(());
                }
                let Some(reference) = &row.reference else {
                    failed(
                        &mut summary,
                        emit,
                        Error::Unavailable(
                            "candidate has no native address for contextual refinement".into(),
                        ),
                    )?;
                    return Ok(());
                };
                let resolved =
                    match runtime.block_on(rust::resolve_entity(root, &reference.locator, token)) {
                        Ok(found) => found,
                        Err(error) => {
                            failed(&mut summary, emit, error)?;
                            return Ok(());
                        }
                    };
                let truth = match filter {
                    Refinement::ResolvesTo { .. } | Refinement::UsedAt { .. } => {
                        expected.as_ref().map_or(Truth::Unknown, |target| {
                            same_entity(&target.identity, &resolved.identity)
                        })
                    }
                    Refinement::ImportableAt { .. } => {
                        if expected.as_ref().is_some_and(|target| {
                            same_entity(&target.identity, &resolved.identity) == Truth::True
                        }) {
                            Truth::True
                        } else {
                            Truth::Unknown
                        }
                    }
                    Refinement::NativePredicate { text } => {
                        match runtime.block_on(rust::semantic_operation_cancellable(
                            root,
                            &resolved.locator,
                            crate::semantic::Kind::Predicate { text: text.clone() },
                            token,
                        )) {
                            Ok(answer) => predicate_truth(&answer),
                            Err(error) => {
                                failed(&mut summary, emit, error)?;
                                return Ok(());
                            }
                        }
                    }
                };
                match truth {
                    Truth::True => {
                        row.entity = Some(resolved.identity);
                        row.candidate_reference = row.reference.take();
                        row.reference = Some(ResultReference::new(resolved.locator));
                        accepted(row, request, &mut summary, emit, &mut counted)?;
                    }
                    Truth::False => {}
                    Truth::Unknown => {
                        failed(&mut summary,emit,Error::Unavailable("native destination/predicate did not prove this candidate importable/true; a different selected entity is not a negative importability proof".into()))?;
                    }
                }
                if limits(request).is_some_and(|limit| summary.matches >= limit as u64) {
                    stopped = true;
                    scan.cancel();
                    summary.complete = false;
                    summary.stop = StopReason::Limit;
                }
                Ok(())
            }
            other => emit(other),
        }
    };
    let examined = run_inner(
        root,
        &candidate_request,
        candidates,
        &scan,
        &mut consume,
        runtime,
    );
    drop(consume);
    match examined {
        Ok(work) => {
            merge_work(&mut summary, &work);
            if stopped {
                summary.cancellation_requested = false;
                summary.stop = StopReason::Limit;
            }
        }
        Err(Error::Cancelled) if stopped => {}
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
    runtime: &tokio::runtime::Handle,
) -> Result<Summary, Error> {
    let mut summary = Summary {
        complete: true,
        ..Default::default()
    };
    let mut counted = BTreeSet::new();
    let mut seen = BTreeSet::new();
    for clause in clauses {
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }
        let child = token.child_token();
        let mut stopped = false;
        let mut added = Summary::default();
        let mut rendered = request.clone();
        rendered.presentation.budget = request.presentation.budget.map(|budget| {
            budget.saturating_sub(usize::try_from(summary.emitted).unwrap_or(usize::MAX))
        });
        let mut selected = request.clone();
        selected.answer.kind = AnswerKind::Matches;
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
                        accepted(row, &rendered, &mut added, emit, &mut counted)?;
                    }
                    if limits(request)
                        .is_some_and(|limit| summary.matches + added.matches >= limit as u64)
                    {
                        stopped = true;
                        child.cancel();
                    }
                    Ok(())
                }
                other => emit(other),
            }
        };
        let result = run_inner(root, &selected, clause, &child, &mut consume, runtime);
        drop(consume);
        summary.matches += added.matches;
        summary.emitted += added.emitted;
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
    runtime: &tokio::runtime::Handle,
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
    let child = token.child_token();
    let mut stopped = false;
    let mut seen = BTreeSet::new();
    let mut selected = request.clone();
    selected.answer.kind = AnswerKind::Matches;
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
                let entity =
                    match runtime.block_on(rust::resolve_entity(root, &reference.locator, token)) {
                        Ok(entity) => entity,
                        Err(error) => {
                            failed(&mut summary, emit, error)?;
                            return Ok(());
                        }
                    };
                let mut truth = Truth::True;
                for clause in remaining {
                    truth = truth.and(entity_membership(
                        root,
                        request,
                        clause,
                        &entity.identity,
                        token,
                        emit,
                        runtime,
                    )?);
                    if truth == Truth::False {
                        break;
                    }
                }
                if truth == Truth::True && seen.insert(entity.identity.clone()) {
                    row.entity = Some(entity.identity);
                    row.candidate_reference = row.reference.take();
                    row.reference = Some(ResultReference::new(entity.locator));
                    accepted(row, request, &mut summary, emit, &mut counted)?;
                } else if truth == Truth::Unknown {
                    failed(&mut summary,emit,Error::Unavailable("an entity-scoped clause lacked a complete negative or native identity witness".into()))?;
                }
                if limits(request).is_some_and(|limit| summary.matches >= limit as u64) {
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
    let work = run_inner(root, &selected, first, &child, &mut consume, runtime);
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
    runtime: &tokio::runtime::Handle,
) -> Result<Truth, Error> {
    if let Predicate::Not {
        within: MatchScope::Entity,
        clause,
    } = predicate
    {
        return entity_membership(root, request, clause, identity, token, emit, runtime)
            .map(Truth::negate);
    }
    let mut selected = request.clone();
    selected.answer.kind = AnswerKind::Matches;
    selected.answer.limit = None;
    selected.presentation = QueryPresentation::default();
    let mut found = false;
    let mut unknown = false;
    let mut consume = |event| -> Result<(), Error> {
        match event {
            Event::Result(row) => {
                if let Some(reference) = row.reference {
                    match runtime.block_on(rust::resolve_entity(root, &reference.locator, token)) {
                        Ok(entity) => match same_entity(&entity.identity, identity) {
                            Truth::True => found = true,
                            Truth::False => {}
                            Truth::Unknown => unknown = true,
                        },
                        Err(error) if unavailable(&error) => {
                            unknown = true;
                            diagnostic(emit, &error)?;
                        }
                        Err(error) => return Err(error),
                    }
                } else {
                    unknown = true;
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
    match run_inner(root, &selected, predicate, token, &mut consume, runtime) {
        Ok(_summary) if found => Ok(Truth::True),
        Ok(summary)
            if !unknown && summary.complete && summary.failures == 0 && summary.unknown == 0 =>
        {
            Ok(Truth::False)
        }
        Ok(_) => Ok(Truth::Unknown),
        Err(error) if unavailable(&error) => {
            diagnostic(emit, &error)?;
            Ok(Truth::Unknown)
        }
        Err(error) => Err(error),
    }
}

struct Traversal<'a> {
    seed: &'a ResultReference,
    relation: Relationship,
    direction: Direction,
    depth: usize,
    max_nodes: usize,
}
fn related(
    root: &Path,
    request: &Request,
    traversal: Traversal<'_>,
    token: &CancellationToken,
    emit: &mut (dyn FnMut(Event) -> Result<(), Error> + Send),
    runtime: &tokio::runtime::Handle,
) -> Result<Summary, Error> {
    let Traversal {
        seed,
        relation,
        direction,
        depth,
        max_nodes,
    } = traversal;
    let mut summary = Summary {
        complete: true,
        ..Default::default()
    };
    let mut counted = BTreeSet::new();
    let mut queue = VecDeque::from([(seed.locator.clone(), 0usize)]);
    let mut visited = BTreeSet::new();
    let mut emitted = BTreeSet::new();
    visited.insert(serde_json::to_string(&seed.locator)?);
    while let Some((locator, distance)) = queue.pop_front() {
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }
        if distance >= depth {
            continue;
        }
        let result = match runtime.block_on(rust::relationship(
            root,
            &locator,
            relation,
            direction,
            max_nodes.saturating_add(1),
            token,
        )) {
            Ok(result) => result,
            Err(error) => {
                failed(&mut summary, emit, error)?;
                continue;
            }
        };
        summary.complete &= result.complete;
        for source in result.sources {
            emit(Event::Source(source))?;
        }
        for diagnostic in result.diagnostics {
            summary.unknown += 1;
            summary.complete = false;
            emit(Event::Diagnostic {
                code: diagnostic.code,
                message: diagnostic.message,
                source: diagnostic.source,
            })?;
        }
        for row in result.rows {
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
            let key = serde_json::to_string(&reference.locator)?;
            if emitted.insert(key.clone()) {
                accepted(row.clone(), request, &mut summary, emit, &mut counted)?;
            }
            if limits(request).is_some_and(|limit| summary.matches >= limit as u64) {
                summary.complete = false;
                summary.stop = StopReason::Limit;
                return Ok(summary);
            }
            if !visited.contains(&key) {
                if visited.len() >= max_nodes {
                    summary.complete = false;
                    summary.stop = StopReason::Limit;
                    return Ok(summary);
                }
                visited.insert(key);
                queue.push_back((reference.locator.clone(), distance + 1));
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
        let result = run(
            root.path(),
            &request,
            &CancellationToken::new(),
            &mut |_event| Err(Error::Io(std::io::ErrorKind::BrokenPipe.into())),
            runtime.handle(),
        );
        assert!(
            matches!(result,Err(Error::Io(error)) if error.kind()==std::io::ErrorKind::BrokenPipe)
        );
        std::fs::remove_file(root.path().join("source.rs")).unwrap();
    }
}
