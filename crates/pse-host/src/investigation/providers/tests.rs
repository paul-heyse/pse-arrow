// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
#[test]
fn stable_provider_failure_retains_the_actual_answer_selector() {
    let root = tempfile::tempdir().unwrap();
    let request = Request {
        sources: SourceSelector {
            paths: vec![root.path().join("disappeared")],
            ..Default::default()
        },
        answer: AnswerDemand {
            order: ResultOrder::Stable,
            ..Default::default()
        },
        ..Default::default()
    };
    let error = run(
        root.path(),
        &request,
        &CancellationToken::new(),
        &mut |_| Ok(()),
    )
    .unwrap_err();
    let Error::SourcePath(failure) = error else {
        panic!("lost source cause")
    };
    let selection = failure.recovery.selection.unwrap();
    assert_eq!(selection.role, SelectionRole::Answer);
    assert_eq!(selection.selector, request.sources);
}
use std::{fs, path::PathBuf, process::Command};
use tokio_util::sync::CancellationToken;

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .env("GIT_AUTHOR_NAME", "A")
        .env("GIT_AUTHOR_EMAIL", "a@example.invalid")
        .env("GIT_COMMITTER_NAME", "A")
        .env("GIT_COMMITTER_EMAIL", "a@example.invalid")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
fn execute(root: &Path, request: &Request) -> (Summary, Vec<Event>) {
    let mut events = Vec::new();
    let summary = run(root, request, &CancellationToken::new(), &mut |event| {
        events.push(event);
        Ok(())
    })
    .unwrap();
    (summary, events)
}
fn text(pattern: &str) -> Request {
    Request {
        predicate: Predicate::Text(TextQuery {
            patterns: vec![pattern.into()],
            literal: true,
            ..Default::default()
        }),
        presentation: QueryPresentation {
            context: 0,
            ..Default::default()
        },
        ..Default::default()
    }
}
#[test]
fn ignored_explicit_and_revision_only_paths_keep_their_universe() {
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "-q"]);
    fs::write(root.path().join(".gitignore"), "ignored.rs\n").unwrap();
    fs::write(root.path().join("ignored.rs"), "ignored").unwrap();
    fs::write(root.path().join("gone.rs"), "committed").unwrap();
    git(root.path(), &["add", ".gitignore", "gone.rs"]);
    git(root.path(), &["commit", "-qm", "base"]);
    fs::remove_file(root.path().join("gone.rs")).unwrap();
    let mut request = Request::default();
    request.sources.paths = vec!["ignored.rs".into()];
    let (_, events) = execute(root.path(), &request);
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::Result(_)))
            .count(),
        1
    );
    request.sources.paths = vec!["gone.rs".into()];
    request.sources.kind = SourceKind::Revision {
        revision: "HEAD".into(),
    };
    let sources = select(root.path(), &request.sources, &CancellationToken::new()).unwrap();
    assert_eq!(sources.len(), 1);
    let input = acquire(root.path(), &sources[0]).unwrap();
    assert_eq!(input.bytes, b"committed");
    assert!(matches!(input.source,SourceKind::Revision{ref revision} if revision!="HEAD"));
}
#[test]
fn package_selection_uses_nearest_authored_manifest_in_each_source_universe() {
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "-q"]);
    fs::write(
        root.path().join("Cargo.toml"),
        "[package]\nname='outer'\nversion='0.1.0'\n",
    )
    .unwrap();
    fs::write(root.path().join("outer.rs"), "hit\n").unwrap();
    fs::create_dir(root.path().join("nested")).unwrap();
    fs::write(
        root.path().join("nested/Cargo.toml"),
        "[package]\nname='inner'\nversion='0.1.0'\n",
    )
    .unwrap();
    fs::write(root.path().join("nested/inner.rs"), "hit\n").unwrap();
    git(root.path(), &["add", "."]);
    git(root.path(), &["commit", "-qm", "packages"]);
    for kind in [
        SourceKind::Working,
        SourceKind::Index,
        SourceKind::Revision {
            revision: "HEAD".into(),
        },
    ] {
        let mut request = text("hit");
        request.sources.kind = kind;
        request.sources.packages = vec!["outer".into()];
        let (summary, _) = execute(root.path(), &request);
        assert_eq!(summary.matches, 1);
        assert_eq!(summary.failures, 0);
        request.sources.packages = vec!["inner".into()];
        let (summary, _) = execute(root.path(), &request);
        assert_eq!(summary.matches, 1);
        assert_eq!(summary.failures, 0);
    }
}
#[test]
fn filename_answers_use_names_and_metadata_without_reading_or_hashing_contents() {
    use std::os::unix::fs::PermissionsExt as _;
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "-q"]);
    let path = root.path().join("a.rs");
    fs::write(&path, "content not requested").unwrap();
    git(root.path(), &["add", "a.rs"]);
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
    for order in [ResultOrder::Arrival, ResultOrder::Stable] {
        let mut request = Request::default();
        request.answer.order = order;
        let (summary, events) = execute(root.path(), &request);
        assert_eq!(summary.examined_bytes, 0);
        assert_eq!(summary.failures, 0);
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
        let Some(ResultReference {
            locator: Locator::Source {
                guard: Some(guard), ..
            },
            ..
        }) = row.reference
        else {
            panic!("name witness must retain its observation");
        };
        assert!(guard.sha256.is_none());
        assert!(guard.observed.is_none());
    }
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    let mut request = Request::default();
    request.sources.kind = SourceKind::Index;
    let (_, events) = execute(root.path(), &request);
    let reference = events
        .into_iter()
        .find_map(|event| {
            if let Event::Result(row) = event {
                row.reference
            } else {
                None
            }
        })
        .unwrap();
    assert!(
        matches!(&reference.locator,Locator::Source{guard:Some(guard),..}if guard.git_blob.is_some()&&guard.sha256.is_none())
    );
    fs::write(&path, "new staged content").unwrap();
    git(root.path(), &["add", "a.rs"]);
    assert!(matches!(
        inspect(root.path(), &reference.locator),
        Err(Error::Stale(_))
    ));
}
#[test]
fn explicit_dependencies_read_locked_offline_roots_without_building_or_changing_the_lock() {
    let sandbox = tempfile::tempdir().unwrap();
    let root = sandbox.path().join("consumer");
    let dependency = sandbox.path().join("outside");
    for path in [&root, &dependency] {
        fs::create_dir_all(path.join("src")).unwrap();
    }
    fs::write(root.join("Cargo.toml"),"[package]\nname='consumer'\nversion='0.1.0'\nedition='2024'\n[dependencies]\noutside={path='../outside'}\n").unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn local() {}\n").unwrap();
    fs::write(
        dependency.join("Cargo.toml"),
        "[package]\nname='outside'\nversion='0.1.0'\nedition='2024'\n",
    )
    .unwrap();
    fs::write(
        dependency.join("src/lib.rs"),
        "pub fn dependency_needle() {}\n",
    )
    .unwrap();
    let lock = "version = 4\n[[package]]\nname = \"consumer\"\nversion = \"0.1.0\"\ndependencies = [\"outside\"]\n[[package]]\nname = \"outside\"\nversion = \"0.1.0\"\n";
    fs::write(root.join("Cargo.lock"), lock).unwrap();
    let mut request = text("dependency_needle");
    assert_eq!(execute(&root, &request).0.matches, 0);
    request.sources.include_dependencies = true;
    request.sources.packages = vec!["outside".into()];
    let (summary, events) = execute(&root, &request);
    assert_eq!(summary.matches, 1);
    assert_eq!(summary.failures, 0);
    let source = events
        .into_iter()
        .find_map(|event| {
            if let Event::Source(source) = event {
                Some(source)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(source.path.to_path_buf(), dependency.join("src/lib.rs"));
    assert_eq!(fs::read_to_string(root.join("Cargo.lock")).unwrap(), lock);
    assert!(!root.join("target").exists());
    assert!(!dependency.join("target").exists());
    request.sources.kind = SourceKind::Revision {
        revision: "HEAD".into(),
    };
    assert!(matches!(
        run(&root, &request, &CancellationToken::new(), &mut |_| Ok(())),
        Err(Error::Invalid(_))
    ));
}
#[test]
fn prepared_semantic_selection_never_acquires_current_cargo_metadata() {
    let root = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join("Cargo.toml"),
        "invalid manifest proving no metadata call",
    )
    .unwrap();
    fs::write(root.path().join("a.rs"), "fn run() {}").unwrap();
    let selector = SourceSelector {
        include_dependencies: true,
        ..Default::default()
    };
    let selection = Selection::for_prepared_context(root.path(), &selector).unwrap();
    assert_eq!(selection.dependency_roots().count(), 0);
    let sources =
        selection::select_prepared_context(root.path(), &selector, &CancellationToken::new())
            .unwrap();
    assert_eq!(sources.len(), 2);
}
#[test]
fn index_change_and_moving_revision_ref_cannot_change_a_recorded_result() {
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "-q"]);
    fs::write(root.path().join("a.rs"), "old").unwrap();
    git(root.path(), &["add", "a.rs"]);
    git(root.path(), &["commit", "-qm", "first"]);
    let source = crate::source_read::SourceRequest::Index {
        repository: root.path().into(),
        path: "a.rs".into(),
    };
    let input = acquire(root.path(), &source).unwrap();
    let reference = record(&input, None, "file", "".into()).reference.unwrap();
    let committed = acquire(
        root.path(),
        &crate::source_read::SourceRequest::Blob {
            repository: root.path().into(),
            path: "a.rs".into(),
            revision: "HEAD".into(),
        },
    )
    .unwrap();
    fs::write(root.path().join("a.rs"), "new").unwrap();
    git(root.path(), &["add", "a.rs"]);
    git(root.path(), &["commit", "-qm", "second"]);
    assert!(matches!(
        inspect(root.path(), &reference.locator),
        Err(Error::Stale(_))
    ));
    let old = record(&committed, None, "file", "".into())
        .reference
        .unwrap();
    assert_eq!(inspect(root.path(), &old.locator).unwrap().context, "old");
}
#[test]
fn demand_count_units_and_no_match_do_not_emit_an_input_inventory() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("a.rs"), "hit hit\nother\nhit\n").unwrap();
    let mut request = text("hit");
    request.answer.kind = AnswerKind::Count;
    request.answer.count_unit = CountUnit::Matches;
    let (summary, events) = execute(root.path(), &request);
    assert!(matches!(summary.count, Count::Exact(3)));
    assert!(
        events
            .iter()
            .all(|event| matches!(event, Event::Progress { .. }))
    );
    request.answer.count_unit = CountUnit::Lines;
    let (summary, _) = execute(root.path(), &request);
    assert!(matches!(summary.count, Count::Exact(2)));
    request.answer.count_unit = CountUnit::Files;
    let (summary, _) = execute(root.path(), &request);
    assert!(matches!(summary.count, Count::Exact(1)));
    let (summary, events) = execute(root.path(), &text("absent"));
    assert!(summary.complete);
    assert_eq!(summary.matches, 0);
    assert!(
        events
            .iter()
            .all(|event| matches!(event, Event::Progress { .. }))
    );
    let mut request = text("hit");
    request.answer.limit = Some(1);
    let (summary, events) = execute(root.path(), &request);
    assert_eq!(summary.emitted, 1);
    assert_eq!(summary.stop, StopReason::Limit);
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::Source(_)))
            .count(),
        1
    );
}
#[test]
fn parallel_limit_and_consumer_failure_settle_without_cancelling_the_caller() {
    let root = tempfile::tempdir().unwrap();
    for index in 0..32 {
        fs::write(
            root.path().join(format!("{index}.rs")),
            "hit\n".repeat(1024),
        )
        .unwrap();
    }
    let token = CancellationToken::new();
    let mut request = text("hit");
    request.answer.limit = Some(1);
    let mut rows = 0;
    let summary = run(root.path(), &request, &token, &mut |event| {
        if matches!(event, Event::Result(_)) {
            rows += 1;
        }
        Ok(())
    })
    .unwrap();
    assert_eq!(rows, 1);
    assert_eq!(summary.stop, StopReason::Limit);
    assert!(!summary.cancellation_requested);
    assert!(!token.is_cancelled());
    let error = run(root.path(), &request, &token, &mut |event| {
        if matches!(event, Event::Result(_)) {
            Err(Error::Invalid("closed consumer witness".into()))
        } else {
            Ok(())
        }
    })
    .unwrap_err();
    assert!(matches!(error,Error::Invalid(message) if message=="closed consumer witness"));
    assert!(!token.is_cancelled());
}
#[test]
fn request_compiles_compatible_lexical_clauses_once_even_across_file_contexts() {
    let program = Arc::new(rust::CompiledProgram::default());
    let mut prepared = 0;
    let Predicate::Text(query) = text("hit").predicate else {
        panic!("the authored fixture must construct a text predicate")
    };
    let first = rust::EvaluationContext::new(program.clone())
        .lexical_matcher(&query, || {
            prepared += 1;
            lexical::matcher(&query)
        })
        .unwrap();
    let second = rust::EvaluationContext::new(program)
        .lexical_matcher(&query, || {
            prepared += 1;
            lexical::matcher(&query)
        })
        .unwrap();
    assert_eq!(prepared, 1);
    assert!(Arc::ptr_eq(&first, &second));
    let root = tempfile::tempdir().unwrap();
    let mut request = text("(");
    if let Predicate::Text(query) = &mut request.predicate {
        query.literal = false;
    }
    assert!(
        matches!(
            run(
                root.path(),
                &request,
                &CancellationToken::new(),
                &mut |_| Ok(())
            ),
            Err(Error::Invalid(_))
        ),
        "native pattern errors remain errors even in an empty universe"
    );
}
#[test]
fn existential_file_conjunction_counts_are_exact_after_limited_native_witnesses() {
    let root = tempfile::tempdir().unwrap();
    for path in ["a.rs", "b.rs"] {
        fs::write(
            root.path().join(path),
            "fn needle_one() {}\nfn needle_two() {}\n",
        )
        .unwrap();
    }
    let symbol = Predicate::Symbol(SymbolQuery {
        name: "needle".into(),
        ..Default::default()
    });
    let mut request = Request {
        predicate: Predicate::All {
            within: MatchScope::File,
            clauses: vec![symbol.clone(), text("needle").predicate],
        },
        ..Default::default()
    };
    request.answer.kind = AnswerKind::Count;
    let (summary, _) = execute(root.path(), &request);
    assert!(summary.complete);
    assert_eq!(summary.unknown, 0);
    assert_eq!(summary.failures, 0);
    assert!(matches!(summary.count, Count::Exact(2)));
    request.predicate = Predicate::Not {
        within: MatchScope::File,
        clause: Box::new(symbol),
    };
    let (summary, _) = execute(root.path(), &request);
    assert!(summary.complete);
    assert_eq!(summary.unknown, 0);
    assert!(matches!(summary.count, Count::Exact(0)));
    request.answer.kind = AnswerKind::Matches;
    request.answer.limit = Some(1);
    request.predicate = Predicate::Symbol(SymbolQuery {
        name: "needle".into(),
        ..Default::default()
    });
    let (summary, _) = execute(root.path(), &request);
    assert_eq!(summary.unknown, 0);
    assert_eq!(summary.stop, StopReason::Limit);
    assert!(matches!(summary.count, Count::AtLeast(_)));
}
#[test]
fn native_context_attaches_to_hits_without_extra_rows_and_keeps_first_n_complete() {
    let root = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join("a.rs"),
        "before\nhit one\nhit two\nafter\nlast\n",
    )
    .unwrap();
    let mut request = text("hit");
    request.presentation.context = 1;
    request.answer.limit = Some(1);
    let (summary, events) = execute(root.path(), &request);
    assert_eq!(summary.emitted, 1);
    let rows = events
        .into_iter()
        .filter_map(|e| {
            if let Event::Result(row) = e {
                Some(row)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].context, "before\nhit one\nhit two\n");
    assert_eq!(rows[0].range.as_ref().unwrap().line_start, 2);
    request.answer.limit = None;
    let (summary, events) = execute(root.path(), &request);
    assert!(summary.complete);
    assert_eq!(summary.emitted, 2);
    let rows = events
        .into_iter()
        .filter_map(|e| {
            if let Event::Result(row) = e {
                Some(row)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1].context, "hit one\nhit two\nafter\n");
}
#[test]
fn presentation_budget_does_not_change_explicit_exhaustive_demand() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("a.rs"), "hit hit hit hit\nhit hit\n").unwrap();
    for order in [ResultOrder::Arrival, ResultOrder::Stable] {
        let mut request = text("hit");
        request.answer.limit = None;
        request.answer.order = order;
        request.presentation.budget = Some(2);
        request.presentation.context = 1;
        let (summary, events) = execute(root.path(), &request);
        assert_eq!(summary.emitted, 2);
        assert_eq!(summary.matches, 6);
        assert!(summary.complete);
        assert!(matches!(summary.count, Count::Exact(6)));
        assert_eq!(
            events
                .into_iter()
                .filter(|event| matches!(event, Event::Result(_)))
                .count(),
            2
        );
        request.presentation.budget = Some(0);
        let (summary, events) = execute(root.path(), &request);
        assert_eq!(summary.emitted, 0);
        assert!(summary.complete);
        assert!(matches!(summary.count, Count::Exact(6)));
        assert!(
            events
                .iter()
                .all(|event| matches!(event, Event::Progress { .. }))
        );
    }
}
#[test]
fn stable_continuation_refuses_changed_source_and_does_not_need_a_store() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("a.rs"), "hit\nhit\nhit\n").unwrap();
    let mut request = text("hit");
    request.answer.limit = Some(1);
    request.answer.order = ResultOrder::Stable;
    let (first, _) = execute(root.path(), &request);
    request.answer.cursor = first.next_cursor;
    assert!(request.answer.cursor.is_some());
    let (second, events) = execute(root.path(), &request);
    assert_eq!(second.emitted, 1);
    let result = events
        .into_iter()
        .find_map(|e| {
            if let Event::Result(row) = e {
                Some(row)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(result.range.unwrap().line_start, 2);
    fs::write(root.path().join("a.rs"), "changed\n").unwrap();
    assert!(matches!(
        run(
            root.path(),
            &request,
            &CancellationToken::new(),
            &mut |_| Ok(())
        ),
        Err(Error::Stale(_))
    ));
}
#[test]
fn lexical_witness_guard_refuses_same_length_change_even_if_mtime_is_restored() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("a.rs");
    fs::write(&path, "needle\n").unwrap();
    let modified = fs::metadata(&path).unwrap().modified().unwrap();
    let (_, events) = execute(root.path(), &text("needle"));
    let reference = events
        .into_iter()
        .find_map(|event| {
            if let Event::Result(row) = event {
                row.reference
            } else {
                None
            }
        })
        .unwrap();
    assert!(
        matches!(&reference.locator,Locator::Source{guard:Some(guard),..}if guard.sha256.is_none()&&guard.observed.is_some())
    );
    fs::write(&path, "NEEDLE\n").unwrap();
    fs::File::open(&path)
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(modified))
        .unwrap();
    assert!(matches!(
        inspect(root.path(), &reference.locator),
        Err(Error::Stale(_))
    ));
}
#[test]
fn file_conjunction_witnesses_need_not_be_one_node_or_line() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("a.rs"), "alpha\nbeta\n").unwrap();
    let mut request = Request::default();
    request.sources.paths = vec![PathBuf::from("a.rs")];
    request.predicate = Predicate::All {
        within: MatchScope::File,
        clauses: vec![text("alpha").predicate, text("beta").predicate],
    };
    let (summary, events) = execute(root.path(), &request);
    assert_eq!(summary.matches, 1);
    let row = events
        .into_iter()
        .find_map(|e| {
            if let Event::Result(row) = e {
                Some(row)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(row.captures.len(), 2);
    assert_ne!(
        row.captures["clause_0"][0].line_start,
        row.captures["clause_1"][0].line_start
    );
    request.predicate = Predicate::All {
        within: MatchScope::Line,
        clauses: vec![text("alpha").predicate, text("beta").predicate],
    };
    assert_eq!(execute(root.path(), &request).0.matches, 0);
    fs::write(root.path().join("a.rs"), "alpha beta\nalpha\n").unwrap();
    assert_eq!(execute(root.path(), &request).0.matches, 1);
}
#[tokio::test]
async fn native_pickaxe_and_log_preserve_commits_and_blame_settles() {
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "-q"]);
    fs::write(root.path().join("a.rs"), "first\n").unwrap();
    git(root.path(), &["add", "a.rs"]);
    git(root.path(), &["commit", "-qm", "first"]);
    fs::write(root.path().join("a.rs"), "first\nneedle\n").unwrap();
    git(root.path(), &["add", "a.rs"]);
    git(root.path(), &["commit", "-qm", "second"]);
    let mut request = Request::default();
    request.sources.paths = vec!["a.rs".into()];
    let query = HistoryQuery {
        operation: HistoryOperation::Pickaxe,
        query: Some("needle".into()),
        revision: Some("HEAD".into()),
    };
    let mut events = Vec::new();
    let summary = history(
        root.path(),
        &request,
        &query,
        &CancellationToken::new(),
        &mut |event| {
            events.push(event);
            Ok(())
        },
    )
    .await
    .unwrap();
    assert!(summary.complete);
    assert_eq!(summary.matches, 1);
    let row = events
        .into_iter()
        .find_map(|e| {
            if let Event::Result(row) = e {
                Some(row)
            } else {
                None
            }
        })
        .unwrap();
    assert!(row.context.contains("second"));
    assert!(row.reference.is_some());
    let query = HistoryQuery {
        operation: HistoryOperation::Blame,
        query: Some("2,2".into()),
        revision: Some("HEAD".into()),
    };
    let mut events = Vec::new();
    let summary = history(
        root.path(),
        &request,
        &query,
        &CancellationToken::new(),
        &mut |event| {
            events.push(event);
            Ok(())
        },
    )
    .await
    .unwrap();
    assert!(summary.complete);
    assert!(
        events
            .into_iter()
            .any(|e| matches!(e,Event::Result(row) if row.context.contains("\tneedle")))
    );
    let query = HistoryQuery {
        operation: HistoryOperation::Log,
        query: None,
        revision: Some("HEAD".into()),
    };
    request.answer.limit = None;
    request.presentation.budget = Some(1);
    let mut events = Vec::new();
    let summary = history(
        root.path(),
        &request,
        &query,
        &CancellationToken::new(),
        &mut |event| {
            events.push(event);
            Ok(())
        },
    )
    .await
    .unwrap();
    assert!(summary.complete);
    assert_eq!(summary.matches, 2);
    assert_eq!(summary.emitted, 1);
    assert!(matches!(summary.count, Count::Exact(2)));
    request.answer.kind = AnswerKind::Count;
    let mut events = Vec::new();
    let summary = history(
        root.path(),
        &request,
        &query,
        &CancellationToken::new(),
        &mut |event| {
            events.push(event);
            Ok(())
        },
    )
    .await
    .unwrap();
    assert!(matches!(summary.count, Count::Exact(2)));
    assert!(
        events
            .iter()
            .all(|event| matches!(event, Event::Progress { .. }))
    );
    request.answer.kind = AnswerKind::Matches;
    request.presentation.budget = Some(0);
    let mut events = Vec::new();
    let summary = history(
        root.path(),
        &request,
        &query,
        &CancellationToken::new(),
        &mut |event| {
            events.push(event);
            Ok(())
        },
    )
    .await
    .unwrap();
    assert!(summary.complete);
    assert_eq!(summary.matches, 2);
    assert!(
        events
            .iter()
            .all(|event| matches!(event, Event::Progress { .. }))
    );
    fs::write(root.path().join("a.rs"), "first\nneedle modified\n").unwrap();
    git(root.path(), &["add", "a.rs"]);
    git(root.path(), &["commit", "-qm", "third"]);
    request.answer.kind = AnswerKind::Count;
    let query = HistoryQuery {
        operation: HistoryOperation::Pickaxe,
        query: Some("needle".into()),
        revision: Some("HEAD".into()),
    };
    let string_count = history(
        root.path(),
        &request,
        &query,
        &CancellationToken::new(),
        &mut |_| Ok(()),
    )
    .await
    .unwrap();
    assert!(matches!(string_count.count, Count::Exact(1)));
    let query = HistoryQuery {
        operation: HistoryOperation::PickaxeLines,
        ..query
    };
    let matching_lines = history(
        root.path(),
        &request,
        &query,
        &CancellationToken::new(),
        &mut |_| Ok(()),
    )
    .await
    .unwrap();
    assert!(matches!(matching_lines.count, Count::Exact(2)));
    request.answer.kind = AnswerKind::Exists;
    request.presentation.budget = None;
    let query = HistoryQuery {
        operation: HistoryOperation::Blame,
        query: None,
        revision: Some("HEAD".into()),
    };
    let token = CancellationToken::new();
    let mut events = Vec::new();
    let summary = history(root.path(), &request, &query, &token, &mut |event| {
        events.push(event);
        Ok(())
    })
    .await
    .unwrap();
    assert_eq!(summary.matches, 1);
    assert_eq!(summary.emitted, 1);
    assert_eq!(summary.failures, 0);
    assert_eq!(summary.stop, StopReason::Limit);
    assert!(!token.is_cancelled());
    let query = HistoryQuery {
        operation: HistoryOperation::Log,
        ..query
    };
    let error = history(root.path(), &request, &query, &token, &mut |event| {
        if matches!(event, Event::Result(_)) {
            Err(Error::Invalid("history consumer closed".into()))
        } else {
            Ok(())
        }
    })
    .await
    .unwrap_err();
    assert!(matches!(error,Error::Invalid(message) if message=="history consumer closed"));
    assert!(!token.is_cancelled());
}
