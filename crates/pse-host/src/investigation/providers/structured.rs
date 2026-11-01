// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native structured values and explicit historical acquisition.
use super::super::*;
use super::{ProviderInput, record};
use std::{collections::BTreeMap, path::Path};
use tokio_util::sync::CancellationToken;

/// Select an actual typed native value; absent ranges do not assert source coordinates.
pub fn structured(
    input: &ProviderInput,
    query: &StructuredQuery,
) -> Result<Vec<SearchResult>, Error> {
    let text = std::str::from_utf8(&input.bytes)
        .map_err(|e| Error::Invalid(format!("structured input is not UTF-8: {e}")))?;
    let format = query
        .format
        .as_deref()
        .or_else(|| input.path.extension().and_then(|s| s.to_str()))
        .ok_or_else(|| Error::Invalid("select a structured format".into()))?;
    let value: serde_json::Value = match format {
        "json" => serde_json::from_str(text)?,
        "toml" => serde_json::to_value(
            toml::from_str::<toml::Value>(text).map_err(|e| Error::Invalid(e.to_string()))?,
        )?,
        "yaml" | "yml" => {
            serde_saphyr::from_str(text).map_err(|e| Error::Invalid(e.to_string()))?
        }
        _ => {
            return Err(Error::Invalid(format!(
                "unsupported structured format {format:?}"
            )));
        }
    };
    if !query.pointer.is_empty() && !query.pointer.starts_with('/') {
        return Err(Error::Invalid(
            "structured selector must be a JSON Pointer".into(),
        ));
    }
    let Some(selected) = value.pointer(&query.pointer) else {
        return Ok(Vec::new());
    };
    if query
        .equals
        .as_ref()
        .is_some_and(|expected| expected != selected)
    {
        return Ok(Vec::new());
    }
    let mut result = record(
        input,
        None,
        "structured_value",
        serde_json::to_string(selected)?,
    );
    result.name = Some(query.pointer.clone());
    Ok(vec![result])
}

/// Inspect a known source directly, without a preceding search or analyzer.
pub fn inspect(root: &Path, locator: &Locator) -> Result<SearchResult, Error> {
    let Locator::Source {
        path,
        source,
        range,
        guard,
    } = locator
    else {
        return Err(Error::Invalid(
            "source inspection requires a source locator".into(),
        ));
    };
    if let Some(original) = guard.as_ref().and_then(|guard| guard.root.as_ref())
        && root.canonicalize()? != original.to_path_buf()
    {
        return Err(Error::Stale(
            "result reference belongs to another selected root".into(),
        ));
    }
    let path = path.to_path_buf();
    let request = match source {
        SourceKind::Working => crate::source_read::SourceRequest::Working {
            path: root.join(&path),
        },
        SourceKind::Index => crate::source_read::SourceRequest::Index {
            repository: root.into(),
            path,
        },
        SourceKind::Revision { revision } => crate::source_read::SourceRequest::Blob {
            repository: root.into(),
            path,
            revision: revision.clone(),
        },
    };
    let input = super::acquire(root, &request)?;
    if let Some(expected) = guard {
        if let Some(observed) = &expected.observed {
            use sha2::{Digest as _, Sha256};
            let bytes = input
                .bytes
                .get(observed.range.byte_start..observed.range.byte_end)
                .ok_or_else(|| Error::Stale("observed source range no longer exists".into()))?;
            if super::hex(&Sha256::digest(bytes)) != observed.sha256 {
                return Err(Error::Stale("observed source witness changed".into()));
            }
        }
        if let Some(hash) = &expected.sha256 {
            use sha2::{Digest as _, Sha256};
            if super::hex(&Sha256::digest(&input.bytes)) != *hash {
                return Err(Error::Stale("source content changed".into()));
            }
        }
        if let Some(actual) = &input.guard {
            if expected
                .git_blob
                .as_ref()
                .is_some_and(|blob| actual.git_blob.as_ref() != Some(blob))
            {
                return Err(Error::Stale("selected Git blob changed".into()));
            }
            if expected.length.is_some_and(|n| actual.length != Some(n))
                || expected
                    .modified_nanos
                    .is_some_and(|n| actual.modified_nanos != Some(n))
            {
                return Err(Error::Stale("source observation changed".into()));
            }
        }
    }
    let bytes = if let Some(range) = range {
        input
            .bytes
            .get(range.byte_start..range.byte_end)
            .ok_or_else(|| Error::Invalid("inspection byte range is outside source".into()))?
    } else {
        &input.bytes
    };
    let mut result = record(
        &input,
        range.clone(),
        "source_content",
        String::from_utf8_lossy(bytes).into_owned(),
    );
    if std::str::from_utf8(bytes).is_err() {
        result.bytes = Some(bytes.to_vec());
    }
    Ok(result)
}

/// Exact native Git arguments, executable through the existing managed child owner.
pub fn history_arguments(
    request: &Request,
    query: &HistoryQuery,
) -> Result<Vec<std::ffi::OsString>, Error> {
    use std::ffi::OsString;
    if !request.sources.packages.is_empty()
        || !request.sources.globs.is_empty()
        || !request.sources.exclude.is_empty()
        || !request.sources.types.is_empty()
        || request.sources.tracked.is_some()
        || request.sources.changed
        || request.sources.include_dependencies
        || request.sources.include_generated
        || request.sources.include_hidden
        || request.sources.include_ignored
    {
        return Err(Error::Invalid("native history currently accepts explicit source paths and revisions; package/glob/type/tracked/changed and widening selectors require source search first".into()));
    }
    let mut args: Vec<OsString> = match query.operation {
        HistoryOperation::Log => vec![
            "log".into(),
            "--format=%H%x00%P%x00%an%x00%aI%x00%s%x00".into(),
            "--no-patch".into(),
        ],
        HistoryOperation::Blame => vec!["blame".into(), "--line-porcelain".into()],
        HistoryOperation::Pickaxe => vec![
            "log".into(),
            "--format=%H%x00%P%x00%an%x00%aI%x00%s%x00".into(),
            "--no-patch".into(),
            "-S".into(),
            query
                .query
                .clone()
                .ok_or_else(|| Error::Invalid("pickaxe requires a string-count query".into()))?
                .into(),
        ],
        HistoryOperation::PickaxeLines => vec![
            "log".into(),
            "--format=%H%x00%P%x00%an%x00%aI%x00%s%x00".into(),
            "--no-patch".into(),
            "-G".into(),
            query
                .query
                .clone()
                .ok_or_else(|| {
                    Error::Invalid(
                        "pickaxe-lines requires a changed-line regular expression".into(),
                    )
                })?
                .into(),
        ],
    };
    args.push("--no-textconv".into());
    if query.operation != HistoryOperation::Blame {
        args.push("--no-ext-diff".into());
    }
    if request.answer.kind == AnswerKind::Files {
        return Err(Error::Invalid("history answers commits or blame evidence; select paths as sources rather than requesting a file answer".into()));
    }
    if request.answer.count_unit != CountUnit::Matches
        && !(request.answer.count_unit == CountUnit::Lines
            && query.operation == HistoryOperation::Blame)
    {
        return Err(Error::Invalid(
            "history count units are commits (matches), or original lines for blame".into(),
        ));
    }
    if request.answer.cursor.is_some() {
        return Err(Error::Unavailable("history continuation is not a source-page cursor; select an explicit starting revision".into()));
    }
    let limit = super::work_limit(request);
    if let Some(limit) = limit
        && query.operation != HistoryOperation::Blame
    {
        args.push(format!("--max-count={limit}").into());
    }
    if query.operation == HistoryOperation::Blame
        && let Some(lines) = &query.query
    {
        args.push("-L".into());
        args.push(lines.into());
    }
    if let Some(revision) = &query.revision {
        if revision.starts_with('-') {
            return Err(Error::Invalid("revision must not be an option".into()));
        }
        args.push(revision.into());
    }
    args.push("--".into());
    args.extend(
        request
            .sources
            .paths
            .iter()
            .map(|p| p.as_os_str().to_owned()),
    );
    if query.operation == HistoryOperation::Blame && request.sources.paths.len() != 1 {
        return Err(Error::Invalid(
            "blame requires exactly one explicit source path".into(),
        ));
    }
    Ok(args)
}

/// Run explicit native Git history with joined child wait and independent stdout/stderr drains.
pub async fn history(
    root: &Path,
    request: &Request,
    query: &HistoryQuery,
    token: &CancellationToken,
    emit: &mut (dyn FnMut(Event) -> Result<(), Error> + Send),
) -> Result<Summary, Error> {
    use std::process::Stdio;
    use tokio::io::AsyncReadExt as _;
    if token.is_cancelled() {
        return Ok(Summary {
            complete: false,
            cancellation_requested: true,
            stop: StopReason::Cancelled,
            ..Default::default()
        });
    }
    let _ = history_arguments(request, query)?;
    if request.sources.kind == SourceKind::Index {
        return Err(Error::Invalid(
            "the staging index is not a commit-history universe".into(),
        ));
    }
    let mut resolved = query.clone();
    let selected = query.revision.as_deref().or(match &request.sources.kind {
        SourceKind::Revision { revision } => Some(revision.as_str()),
        _ => None,
    });
    let working_blame = query.operation == HistoryOperation::Blame && selected.is_none();
    let working = if working_blame {
        if request.sources.paths.len() != 1 {
            return Err(Error::Invalid(
                "blame requires exactly one explicit source path".into(),
            ));
        }
        Some(super::acquire(
            root,
            &crate::source_read::SourceRequest::Working {
                path: root.join(&request.sources.paths[0]),
            },
        )?)
    } else {
        None
    };
    if !working_blame {
        let repository = gix::open(root).map_err(|e| Error::Io(std::io::Error::other(e)))?;
        resolved.revision = Some(
            repository
                .rev_parse_single(selected.unwrap_or("HEAD"))
                .map_err(|e| Error::Io(std::io::Error::other(e)))?
                .to_string(),
        );
    }
    let query = &resolved;
    let arguments = history_arguments(request, query)?;
    let mut child = tokio::process::Command::new("git")
        .arg("--no-pager")
        .arg("--literal-pathspecs")
        .arg("-c")
        .arg("color.ui=false")
        .arg("-c")
        .arg("core.fsmonitor=false")
        .arg("-c")
        .arg("log.showSignature=false")
        .arg("-c")
        .arg("log.showNotes=false")
        .arg("-c")
        .arg("log.decorate=false")
        .args(arguments)
        .current_dir(root)
        .env("GIT_PAGER", "cat")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| Error::Invalid("native Git stdout pipe absent".into()))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| Error::Invalid("native Git stderr pipe absent".into()))?;
    let operation = token.child_token();
    let drain_out = async {
        let result = history_records(
            root,
            request,
            query,
            working.as_ref(),
            stdout,
            &operation,
            emit,
        )
        .await;
        if result.is_err()
            || result.as_ref().is_ok_and(|summary| {
                query.operation == HistoryOperation::Blame && summary.stop == StopReason::Limit
            })
        {
            operation.cancel();
        }
        result
    };
    let drain_err = async move {
        let mut bytes = Vec::new();
        stderr.read_to_end(&mut bytes).await.map(|_| bytes)
    };
    let wait = async {
        tokio::select! {
            status=child.wait()=>status.map(|s|(s,false)),
            ()=operation.cancelled()=>{
                let kill=child.start_kill();
                let waited=child.wait().await;
                match(kill,waited){(_,Ok(status))=>Ok((status,true)),(Err(error),Err(_))|(_,Err(error))=>Err(error)}
            }
        }
    };
    let (out, err, status) = tokio::join!(drain_out, drain_err, wait);
    // Every independent drain/wait is attempted before any failure is propagated.
    let mut summary = out?;
    let stderr = err?;
    let (status, _cancelled) = status?;
    if token.is_cancelled() {
        summary.complete = false;
        summary.cancellation_requested = true;
        summary.stop = StopReason::Cancelled;
    } else if !status.success() && !(operation.is_cancelled() && summary.stop == StopReason::Limit)
    {
        emit(Event::Diagnostic {
            phase: DiagnosticPhase::Provider,
            code: "search.git".into(),
            message: format!(
                "native Git exited {status}: {}",
                String::from_utf8_lossy(&stderr)
            ),
            source: None,
        })?;
        summary.failures += 1;
        summary.complete = false;
    }
    if let Some(input) = &working {
        let locator = Locator::Source {
            path: NativePath::from(input.path.clone()),
            source: SourceKind::Working,
            range: None,
            guard: input.guard.clone(),
        };
        let _ = inspect(root, &locator)?;
    }
    summary.count = if summary.failures > 0 {
        Count::Unknown
    } else if summary.complete {
        Count::Exact(summary.matches)
    } else {
        Count::AtLeast(summary.matches)
    };
    Ok(summary)
}

async fn history_records<R: tokio::io::AsyncRead + Unpin>(
    root: &Path,
    request: &Request,
    query: &HistoryQuery,
    working: Option<&ProviderInput>,
    reader: R,
    token: &CancellationToken,
    emit: &mut (dyn FnMut(Event) -> Result<(), Error> + Send),
) -> Result<Summary, Error> {
    use tokio::io::AsyncBufReadExt as _;
    let source = "git_history".to_owned();
    let mut guard = working
        .and_then(|input| input.guard.clone())
        .unwrap_or_default();
    guard.root = Some(NativePath::from(root.canonicalize()?));
    let source_context = SourceContext {
        id: source.clone(),
        path: NativePath::from(root.to_path_buf()),
        origin: None,
        provider: "native Git; original history evidence; no current semantic interpretation"
            .into(),
        generation: query.revision.clone(),
        searched_encoding: None,
        native: None,
    };
    let mut source_context = Some(source_context);
    let mut summary = Summary::default();
    let mut reader = tokio::io::BufReader::new(reader);
    let mut part = Vec::new();
    let mut fields = Vec::new();
    let mut raw = Vec::new();
    let mut field_count = 0;
    let mut blame_open = false;
    loop {
        part.clear();
        let size = reader
            .read_until(
                if query.operation == HistoryOperation::Blame {
                    b'\n'
                } else {
                    0
                },
                &mut part,
            )
            .await?;
        if token.is_cancelled() {
            summary.complete = false;
            summary.stop = StopReason::Cancelled;
            break;
        }
        if size == 0 {
            if field_count != 0 || !raw.is_empty() || blame_open {
                return Err(Error::Invalid(
                    "native Git ended within a history record".into(),
                ));
            }
            break;
        }
        summary.examined_bytes += size as u64;
        let publish = request.answer.kind != AnswerKind::Count
            && request
                .presentation
                .budget
                .is_none_or(|budget| summary.emitted < budget as u64);
        let row = if query.operation == HistoryOperation::Blame {
            blame_open = true;
            if publish {
                raw.extend_from_slice(&part);
            }
            if !part.starts_with(b"\t") {
                continue;
            }
            blame_open = false;
            summary.matches += 1;
            if publish {
                let reference = request.sources.paths.first().map(|path| ResultReference {
                    version: 1,
                    locator: Locator::Source {
                        path: NativePath::from(path.clone()),
                        source: query
                            .revision
                            .as_ref()
                            .map_or(SourceKind::Working, |revision| SourceKind::Revision {
                                revision: revision.clone(),
                            }),
                        range: None,
                        guard: Some(guard.clone()),
                    },
                });
                Some(SearchResult {
                    source: source.clone(),
                    kind: "git_blame_line_porcelain".into(),
                    name: None,
                    entity: None,
                    candidate_reference: None,
                    refinement: None,
                    range: None,
                    context: String::from_utf8_lossy(&raw).into_owned(),
                    reference,
                    captures: BTreeMap::new(),
                    bytes: Some(std::mem::take(&mut raw)),
                })
            } else {
                None
            }
        } else {
            if part.last() != Some(&0) {
                if field_count == 0 && part.iter().all(u8::is_ascii_whitespace) {
                    break;
                }
                return Err(Error::Invalid(
                    "native Git ended within a NUL-delimited history field".into(),
                ));
            }
            if publish {
                raw.extend_from_slice(&part);
                part.pop();
                fields.push(part.clone());
            }
            field_count += 1;
            if field_count < 5 {
                continue;
            }
            field_count = 0;
            summary.matches += 1;
            if publish {
                let commit = String::from_utf8_lossy(&fields[0]).trim().to_owned();
                let payload = serde_json::json!({"commit":commit,"parents":String::from_utf8_lossy(&fields[1]).split_whitespace().collect::<Vec<_>>(),"author":String::from_utf8_lossy(&fields[2]),"authored_at":String::from_utf8_lossy(&fields[3]),"subject":String::from_utf8_lossy(&fields[4]),"rename_inference":"not_requested"});
                let reference = request.sources.paths.first().map(|path| ResultReference {
                    version: 1,
                    locator: Locator::Source {
                        path: NativePath::from(path.clone()),
                        source: SourceKind::Revision {
                            revision: commit.clone(),
                        },
                        range: None,
                        guard: Some(guard.clone()),
                    },
                });
                fields.clear();
                Some(SearchResult {
                    source: source.clone(),
                    kind: "git_commit".into(),
                    name: Some(commit),
                    entity: None,
                    candidate_reference: None,
                    refinement: None,
                    range: None,
                    context: serde_json::to_string(&payload)?,
                    reference,
                    captures: BTreeMap::new(),
                    bytes: Some(std::mem::take(&mut raw)),
                })
            } else {
                None
            }
        };
        if let Some(row) = row {
            if let Some(source) = source_context.take() {
                emit(Event::Source(Box::new(source)))?;
            }
            emit(Event::Result(Box::new(row)))?;
            summary.emitted += 1;
        }
        emit(Event::Progress {
            examined_files: 0,
            examined_bytes: summary.examined_bytes,
            matches: summary.matches,
        })?;
        // Git limits log itself; blame requires an early owned child stop after a witness.
        if query.operation == HistoryOperation::Blame
            && super::work_limit(request).is_some_and(|limit| summary.matches >= limit as u64)
        {
            summary.complete = false;
            summary.stop = StopReason::Limit;
            break;
        }
    }
    if super::work_limit(request).is_some_and(|limit| summary.matches >= limit as u64) {
        summary.complete = false;
        summary.stop = StopReason::Limit;
    }
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn unsupported_history_selectors_refuse_before_repository_or_child_access() {
        let changes: [fn(&mut SourceSelector); 10] = [
            |source| source.packages.push("package".into()),
            |source| source.globs.push("*.rs".into()),
            |source| source.exclude.push("ignored".into()),
            |source| source.types.push("rust".into()),
            |source| source.tracked = Some(true),
            |source| source.changed = true,
            |source| source.include_dependencies = true,
            |source| source.include_generated = true,
            |source| source.include_hidden = true,
            |source| source.include_ignored = true,
        ];
        for change in changes {
            let mut request = Request::default();
            change(&mut request.sources);
            let error = history(
                Path::new("/does-not-exist/pse-history-selector"),
                &request,
                &HistoryQuery::default(),
                &CancellationToken::new(),
                &mut |_| Ok(()),
            )
            .await
            .unwrap_err();
            assert!(
                matches!(error,Error::Invalid(message) if message.contains("explicit source paths"))
            );
        }
    }
    #[tokio::test]
    async fn history_frames_stream_before_eof_and_preserve_fragments_and_budget() {
        use tokio::io::AsyncWriteExt as _;
        let first = b"abc\0parent\0author\x002026-10-31\0first\0";
        let second = b"\ndef\0abc\0author\x002026-10-31\0second\0";
        let (reader, mut writer) = tokio::io::duplex(3);
        let (acknowledge, received) = tokio::sync::oneshot::channel();
        let mut acknowledge = Some(acknowledge);
        let mut request = Request::default();
        request.answer.limit = None;
        request.presentation.budget = Some(1);
        let query = HistoryQuery::default();
        let token = CancellationToken::new();
        let mut events = Vec::new();
        let mut emit = |event| {
            if matches!(event, Event::Result(_))
                && let Some(acknowledge) = acknowledge.take()
            {
                let _ = acknowledge.send(());
            }
            events.push(event);
            Ok(())
        };
        let scan = history_records(
            Path::new("/"),
            &request,
            &query,
            None,
            reader,
            &token,
            &mut emit,
        );
        let write = async move {
            for byte in first {
                writer.write_all(&[*byte]).await.unwrap();
            }
            received.await.unwrap();
            for byte in second {
                writer.write_all(&[*byte]).await.unwrap();
            }
        };
        let (summary, ()) = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            tokio::join!(scan, write)
        })
        .await
        .expect("first history record must be published before producer closes");
        let summary = summary.unwrap();
        assert!(summary.complete);
        assert_eq!(summary.matches, 2);
        assert_eq!(summary.emitted, 1);
        let row = events
            .into_iter()
            .find_map(|event| {
                if let Event::Result(row) = event {
                    Some(row)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(row.bytes.as_deref(), Some(first.as_slice()));
        let mut emit = |_| Ok(());
        let invalid = history_records(
            Path::new("/"),
            &request,
            &query,
            None,
            b"abc\0partial".as_slice(),
            &token,
            &mut emit,
        )
        .await;
        assert!(matches!(invalid, Err(Error::Invalid(_))));
    }
    #[test]
    fn structured_values_keep_typed_equality_and_unknown_ranges() {
        for (path, text) in [
            ("a.json", r#"{"x":{"enabled":true}}"#),
            ("a.toml", "[x]\nenabled = true\n"),
            ("a.yaml", "x:\n  enabled: true\n"),
        ] {
            let input = ProviderInput {
                path: path.into(),
                source: SourceKind::Working,
                bytes: text.as_bytes().to_vec(),
                guard: None,
            };
            let query = StructuredQuery {
                pointer: "/x/enabled".into(),
                equals: Some(serde_json::json!(true)),
                format: None,
            };
            let results = structured(&input, &query).unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].context, "true");
            assert!(results[0].range.is_none());
            let query = StructuredQuery {
                equals: Some(serde_json::json!("true")),
                ..query
            };
            assert!(structured(&input, &query).unwrap().is_empty());
        }
    }
    #[test]
    fn malformed_content_is_not_an_empty_result() {
        let input = ProviderInput {
            path: "a.json".into(),
            source: SourceKind::Working,
            bytes: b"{".to_vec(),
            guard: None,
        };
        assert!(structured(&input, &StructuredQuery::default()).is_err());
    }
    #[test]
    fn direct_inspect_refuses_changed_observation_and_preserves_raw_bytes() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("bytes");
        std::fs::write(&path, [0, 255]).unwrap();
        let guard = super::super::selection::guard(&path).unwrap();
        let locator = Locator::Source {
            path: NativePath::from(std::path::PathBuf::from("bytes")),
            source: SourceKind::Working,
            range: None,
            guard: Some(guard),
        };
        assert_eq!(
            inspect(root.path(), &locator).unwrap().bytes,
            Some(vec![0, 255])
        );
        std::fs::write(path, [1, 2, 3]).unwrap();
        assert!(matches!(
            inspect(root.path(), &locator),
            Err(Error::Stale(_))
        ));
    }
}
