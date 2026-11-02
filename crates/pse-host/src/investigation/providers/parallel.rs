// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One native walker performs file work; only complete event publication is serialized.
use super::{State, *};
use std::{
    collections::BTreeSet,
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

pub(super) fn run(
    root: &Path,
    request: &Request,
    token: &CancellationToken,
    emit: &mut (dyn FnMut(Event) -> Result<(), Error> + Send),
) -> Result<Summary, Error> {
    if token.is_cancelled() {
        return Ok(Summary {
            complete: false,
            cancellation_requested: true,
            stop: StopReason::Cancelled,
            ..Default::default()
        });
    }
    let selection = Selection::new(root, &request.sources)?;
    let mut initial = Summary::default();
    for path in &request.sources.paths {
        if let Err(error) = std::fs::metadata(root.join(path)) {
            let error = Error::SourcePath(Box::new(SourceFailure::new(
                SourceOperation::SelectRoot,
                root.join(path),
                error,
                false,
            )))
            .with_selection(&request.sources, SelectionRole::Answer);
            emit(Event::Diagnostic {
                phase: error.phase(),
                code: "search.input".into(),
                message: error.to_string(),
                source: None,
                detail: error.query_error_record(),
            })?;
            initial.failures += 1;
            initial.complete = false;
        }
    }
    let mut walk = ignore::WalkBuilder::new(root);
    for dependency in selection.dependency_roots() {
        walk.add(dependency);
    }
    walk.hidden(!request.sources.include_hidden)
        .ignore(!request.sources.include_ignored)
        .git_ignore(!request.sources.include_ignored)
        .git_global(!request.sources.include_ignored)
        .git_exclude(!request.sources.include_ignored)
        .follow_links(false)
        .threads(rayon::current_num_threads());
    let mut explicit = BTreeSet::new();
    for path in &request.sources.paths {
        let path = root.join(path);
        if path.is_file() {
            explicit.insert(path.clone());
            walk.add(path);
        }
    }
    let emit = Mutex::new(emit);
    let summary = Mutex::new(initial);
    let failure = Mutex::new(None);
    let seen = Mutex::new(BTreeSet::new());
    let emitted = AtomicU64::new(0);
    let counted = AtomicU64::new(0);
    let stop = AtomicBool::new(false);
    let operation_token = token.child_token();
    let program = Arc::new(rust::CompiledProgram::default());
    prepare_text(
        &request.predicate,
        &rust::EvaluationContext::new(program.clone()),
    )?;
    let limit = work_limit(request);
    let row_limit = match (limit, request.presentation.budget) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (None, b) => b,
        (a, None) => a,
    };
    let observed = AtomicU64::new(0);
    walk.build_parallel().run(|| {
        let selection = &selection;
        let explicit = &explicit;
        let emit = &emit;
        let summary = &summary;
        let seen = &seen;
        let emitted = &emitted;
        let counted = &counted;
        let stop = &stop;
        let failure = &failure;
        let observed = &observed;
        let operation_token = &operation_token;
        let program = &program;
        Box::new(move |entry| {
            if token.is_cancelled() || stop.load(Ordering::Acquire) {
                return ignore::WalkState::Quit;
            }
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    let error = selection::traversal_error(error)
                        .with_selection(&request.sources, SelectionRole::Answer);
                    let published = match emit.lock() {
                        Ok(mut output) => output(Event::Diagnostic {
                            phase: error.phase(),
                            code: "search.traversal".into(),
                            message: error.to_string(),
                            source: None,
                            detail: error.query_error_record(),
                        }),
                        Err(_) => Err(Error::Unavailable("search writer owner panicked".into())),
                    };
                    if let Err(error) = published {
                        consumer_failed(error, failure, stop, operation_token);
                        return ignore::WalkState::Quit;
                    }
                    if let Ok(mut stats) = summary.lock() {
                        stats.failures += 1;
                        stats.complete = false;
                    }
                    return ignore::WalkState::Continue;
                }
            };
            let path = entry.path();
            let relative = path.strip_prefix(root).unwrap_or(path);
            if entry.file_type().is_some_and(|kind| kind.is_dir()) {
                if !selection.descends(relative) {
                    return ignore::WalkState::Skip;
                }
                if relative.components().any(|c| c.as_os_str() == ".git")
                    || (!request.sources.include_generated
                        && relative
                            .components()
                            .any(|c| matches!(c.as_os_str().to_str(), Some("target" | "build"))))
                {
                    return ignore::WalkState::Skip;
                }
                return ignore::WalkState::Continue;
            }
            if !entry.file_type().is_some_and(|kind| kind.is_file())
                || !selection.accepts(relative, &request.sources)
            {
                return ignore::WalkState::Continue;
            }
            if !explicit.contains(path) && entry.depth() == 0 && path != root {
                return ignore::WalkState::Continue;
            }
            if explicit.contains(path) {
                if let Ok(mut seen) = seen.lock() {
                    if !seen.insert(path.to_path_buf()) {
                        return ignore::WalkState::Continue;
                    }
                } else {
                    return ignore::WalkState::Quit;
                }
            }
            let local_token = operation_token.child_token();
            let mut local_request = request.clone();
            local_request.answer.limit = limit.map(|limit| {
                limit
                    .saturating_sub(emitted.load(Ordering::Acquire) as usize)
                    .max(1)
            });
            local_request.presentation.budget = row_limit
                .map(|limit| limit.saturating_sub(emitted.load(Ordering::Acquire) as usize));
            let mut pending_source = None;
            let mut local_emit = |event: Event| {
                if matches!(&event, Event::Source(_)) {
                    pending_source = Some(event);
                    return Ok(());
                }
                let is_result = matches!(&event,Event::Result(row) if row.kind!="context");
                if is_result {
                    let admitted = emitted
                        .try_update(Ordering::AcqRel, Ordering::Acquire, |n| {
                            if row_limit.is_none_or(|limit| n < (limit as u64)) {
                                Some(n + 1)
                            } else {
                                None
                            }
                        })
                        .is_ok();
                    if !admitted {
                        if limit.is_some() && request.presentation.budget != Some(0) {
                            stop.store(true, Ordering::Release);
                            operation_token.cancel();
                        }
                        return Ok(());
                    }
                }
                let result = match emit.lock() {
                    Ok(mut output) => {
                        let prelude = if let Some(source) = pending_source.take() {
                            output(source)
                        } else {
                            Ok(())
                        };
                        prelude.and_then(|()| output(event))
                    }
                    Err(_) => Err(Error::Unavailable("search writer owner panicked".into())),
                };
                if let Err(error) = result {
                    consumer_failed(error, failure, stop, operation_token);
                    return Err(Error::Unavailable("search consumer closed".into()));
                }
                if is_result
                    && limit.is_some_and(|limit| emitted.load(Ordering::Acquire) >= limit as u64)
                {
                    stop.store(true, Ordering::Release);
                    operation_token.cancel();
                }
                Ok(())
            };
            let mut state = State {
                request: &local_request,
                token: &local_token,
                emit: &mut local_emit,
                summary: Summary::default(),
                sources: BTreeSet::new(),
                counted: BTreeSet::new(),
                program: program.clone(),
            };
            let result = process(
                root,
                &crate::source_read::SourceRequest::Working {
                    path: path.to_path_buf(),
                },
                &mut state,
            );
            let count = if request.answer.count_unit == CountUnit::Matches {
                state.summary.matches
            } else {
                state.counted.len() as u64
            };
            let local = state.finish();
            let total_observed =
                observed.fetch_add(local.matches, Ordering::AcqRel) + local.matches;
            if limit.is_some_and(|limit| total_observed >= limit as u64) {
                stop.store(true, Ordering::Release);
                operation_token.cancel();
            }
            counted.fetch_add(count, Ordering::Relaxed);
            if let Err(error) = result
                && !local_token.is_cancelled()
            {
                if let Ok(mut original) = failure.lock()
                    && original.is_none()
                {
                    *original = Some(error);
                }
                stop.store(true, Ordering::Release);
            }
            if let Ok(mut total) = summary.lock() {
                total.examined_files += local.examined_files;
                total.examined_bytes += local.examined_bytes;
                total.matches += local.matches;
                total.failures += local.failures;
                total.unknown += local.unknown;
                if !local.complete && !local.cancellation_requested {
                    total.complete = false;
                }
                let published = match emit.lock() {
                    Ok(mut output) => output(Event::Progress {
                        examined_files: total.examined_files,
                        examined_bytes: total.examined_bytes,
                        matches: total.matches,
                    }),
                    Err(_) => Err(Error::Unavailable("search writer owner panicked".into())),
                };
                if let Err(error) = published {
                    consumer_failed(error, failure, stop, operation_token);
                }
            }
            if token.is_cancelled() || stop.load(Ordering::Acquire) {
                ignore::WalkState::Quit
            } else {
                ignore::WalkState::Continue
            }
        })
    });
    if let Some(error) = failure
        .into_inner()
        .map_err(|_| Error::Unavailable("native search error owner panicked".into()))?
    {
        return Err(error);
    }
    let mut summary = summary
        .into_inner()
        .map_err(|_| Error::Unavailable("native search summary owner panicked".into()))?;
    summary.emitted = emitted.load(Ordering::Acquire);
    if token.is_cancelled() {
        summary.complete = false;
        summary.cancellation_requested = true;
        summary.stop = StopReason::Cancelled;
    } else if stop.load(Ordering::Acquire) {
        summary.complete = false;
        summary.stop = StopReason::Limit;
    }
    summary.count = if summary.complete {
        Count::Exact(counted.load(Ordering::Acquire))
    } else if summary.unknown == 0 && summary.failures == 0 {
        Count::AtLeast(counted.load(Ordering::Acquire))
    } else {
        Count::Unknown
    };
    Ok(summary)
}

fn consumer_failed(
    error: Error,
    failure: &Mutex<Option<Error>>,
    stop: &AtomicBool,
    token: &CancellationToken,
) {
    if let Ok(mut original) = failure.lock()
        && original.is_none()
    {
        *original = Some(error);
    }
    stop.store(true, Ordering::Release);
    token.cancel();
}
