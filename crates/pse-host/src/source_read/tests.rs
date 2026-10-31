// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;

#[test]
fn declaration_metadata_uses_native_visibility_and_module_nodes_without_wire_changes() {
    let outline = rust(
        "mod real { struct S; impl S { /// pub misleading\n pub(crate) fn same(){} fn member(&self){} } /// text pub\n pub fn same(){} }",
        RustEdition::E2024,
    );
    let functions = outline
        .entries
        .iter()
        .filter(|entry| entry.kind == "FN")
        .collect::<Vec<_>>();
    assert_eq!(functions.len(), 3);
    assert_eq!(
        functions[0].written_visibility.as_deref(),
        Some("pub(crate)")
    );
    assert_eq!(functions[0].modules, vec!["real"]);
    assert!(functions[0].associated);
    assert!(!functions[0].method);
    assert!(functions[1].method);
    assert_eq!(functions[2].written_visibility.as_deref(), Some("pub"));
    assert!(!functions[2].associated);
    let wire = serde_json::to_value(&outline).unwrap();
    assert!(wire["entries"][0].get("written_visibility").is_none());
    assert!(wire["entries"][0].get("modules").is_none());
}

#[test]
fn semantic_anchor_selects_native_names_instead_of_documentation_or_receiver_prefix() {
    let source = "/// authored text\n#[inline]\npub fn café(){ receiver.same(argument); }";
    assert_eq!(
        semantic_anchor(source, 0..source.len()),
        Some(source.find("café").unwrap())
    );
    let start = source.find("receiver.same").unwrap();
    let end = source.find(");").unwrap() + 1;
    assert_eq!(
        semantic_anchor(source, start..end),
        Some(source.find("same").unwrap())
    );
    assert_eq!(
        semantic_anchor(
            source,
            source.find("argument").unwrap()..source.find("argument").unwrap() + 3
        ),
        Some(source.find("argument").unwrap())
    );
}

#[test]
fn incomplete_source_keeps_neighbors_and_original_unicode_bytes() {
    let source =
        "mod m {\n/// café\n#[inline]\npub fn α() { /* preserved */ }\nfn unfinished(\n}\n";
    let outline = rust(source, RustEdition::E2024);
    assert!(!outline.parse_issues.is_empty());
    let selected = show(source, &outline, &["m".into(), "α".into()], None).unwrap();
    assert!(selected.text.contains("/// café"));
    assert!(selected.text.contains("#[inline]"));
    assert!(selected.text.ends_with("/* preserved */ }"));
    assert_eq!(selected.range.line_start, 2);
    assert_eq!(selected.range.line_end, 4);
    assert_eq!(
        selected.text,
        source[selected.range.byte_start..selected.range.byte_end]
    );
}

#[test]
fn duplicate_impl_paths_are_ambiguous_and_byte_selection_is_explicit() {
    let source = "impl A for S { fn act() {} }\nimpl B for S { fn act() {} }";
    let outline = rust(source, RustEdition::E2024);
    let path = ["S".into(), "act".into()];
    assert!(matches!(
        show(source, &outline, &path, None),
        Err(SourceReadError::Ambiguous(2))
    ));
    let at = source.rfind("fn act").unwrap();
    assert_eq!(
        show(source, &outline, &path, Some(at))
            .unwrap()
            .range
            .line_start,
        2
    );
    assert!(matches!(
        show(&format!("//changed\n{source}"), &outline, &path, Some(at)),
        Err(SourceReadError::StaleSource)
    ));
}

#[test]
fn macro_tokens_and_cfg_presence_are_not_expanded_or_resolved() {
    let source = "make! { fn generated() {} }\n#[cfg(feature = \"absent\")] fn written() {}";
    let outline = rust(source, RustEdition::E2024);
    assert!(outline.entries.iter().any(|x| x.path == ["written"]));
    assert!(!outline.entries.iter().any(|x| x.path == ["generated"]));
    assert!(outline.parse_issues.is_empty());
}

#[test]
fn commonmark_headings_exclude_code_and_retain_sections_and_duplicate_titles() {
    let source = "# Root\n```rust\n# not heading\n```\nTitle\n-----\nbody\n## Again\nfirst\n## Again\nsecond\n";
    let outline = markdown(source);
    assert_eq!(outline.entries.len(), 4);
    assert_eq!(outline.entries[1].path, ["Root", "Title"]);
    let selected = show(source, &outline, &["Root".into(), "Title".into()], None).unwrap();
    assert_eq!(selected.text, "Title\n-----\nbody\n");
    assert!(matches!(
        show(source, &outline, &["Root".into(), "Again".into()], None),
        Err(SourceReadError::Ambiguous(2))
    ));
}

#[test]
fn editions_and_original_source_are_part_of_the_reading() {
    let source = "fn async() {}";
    let old = rust(source, RustEdition::E2015);
    let new = rust(source, RustEdition::E2024);
    assert!(old.parse_issues.is_empty());
    assert!(!new.parse_issues.is_empty());
    assert_eq!(old.source_sha256, new.source_sha256);
    let value = serde_json::to_value(&old).unwrap();
    let schema = schemars::schema_for!(Outline);
    jsonschema::validator_for(&serde_json::to_value(schema).unwrap())
        .unwrap()
        .validate(&value)
        .unwrap();
}

#[test]
fn positional_selection_uses_unicode_scalars_and_innermost_recovered_declaration() {
    let text = "impl A for S { fn act() { let α = 1; } }\nimpl B for S { fn act() { let β = 2; } }\nfn broken(\n";
    let outline = rust(text, RustEdition::E2024);
    let offset = text.find('β').unwrap();
    let scalar_column = text[text.find('\n').unwrap() + 1..offset].chars().count() + 1;
    let selection = PositionSelector::LineColumn {
        line: 2,
        column: scalar_column,
    }
    .select(text, &outline, &[])
    .unwrap();
    assert!(selection.text.starts_with("fn act"));
    assert!(selection.text.contains('β'));
    assert_eq!(selection.range.line_start, 2);
    assert!(!outline.parse_issues.is_empty());
    let unicode = PositionSelector::Range {
        start: offset,
        end: offset + 'β'.len_utf8(),
    }
    .select(text, &outline, &[])
    .unwrap();
    assert_eq!(unicode.text, "β");
    assert!(matches!(
        PositionSelector::Byte { offset: offset + 1 }.select(text, &outline, &[]),
        Err(SourceReadError::InvalidRange)
    ));
    assert!(matches!(
        PositionSelector::LineColumn {
            line: 2,
            column: 999
        }
        .select(text, &outline, &[]),
        Err(SourceReadError::InvalidRange)
    ));
}

#[test]
fn declaration_pages_bound_nested_entries_and_bind_source_backend_and_edition() {
    let text = "mod m { struct S; impl S { fn first() {} fn second() {} } }";
    let original = SourceText::new("/source/example.rs".into(), text.into()).unwrap();
    let (first, state) = outline_page(
        &original,
        Some(RustEdition::E2024),
        PageRequest {
            max_records: Some(2),
            cursor: None,
        },
    )
    .unwrap();
    assert_eq!(first.entries.len(), 2);
    assert_eq!(state.total_records, 5);
    assert_eq!(state.omitted_records, 3);
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("example.rs");
    std::fs::write(&path, text).unwrap();
    let response = crate::source_command::outline_page(
        &SourceRequest::Working { path },
        None,
        RustEdition::E2024,
        PageRequest {
            max_records: Some(2),
            cursor: None,
        },
    )
    .unwrap();
    assert_eq!(
        response.exit_code(),
        0,
        "an explicitly requested page is successful despite omitted detail"
    );
    let cursor = state.next_cursor.unwrap();
    let (second, state) = outline_page(
        &original,
        Some(RustEdition::E2024),
        PageRequest {
            max_records: Some(2),
            cursor: Some(cursor.clone()),
        },
    )
    .unwrap();
    assert_eq!(second.entries.len(), 2);
    assert_eq!(second.entries[1].path, ["m", "S", "first"]);
    assert_eq!(state.offset, 2);
    let changed =
        SourceText::new("/source/example.rs".into(), format!("//changed\n{text}")).unwrap();
    assert!(matches!(
        outline_page(
            &changed,
            Some(RustEdition::E2024),
            PageRequest {
                max_records: Some(2),
                cursor: Some(cursor.clone())
            }
        ),
        Err(SourceReadError::StaleSource)
    ));
    assert!(matches!(
        outline_page(
            &original,
            Some(RustEdition::E2018),
            PageRequest {
                max_records: Some(2),
                cursor: Some(cursor)
            }
        ),
        Err(SourceReadError::InvalidCursor)
    ));
}

#[test]
fn reusable_locator_checks_identity_and_full_byte_line_coordinates() {
    let source = SourceText::new("/source/a.rs".into(), "// café\nfn α() {}\n".into()).unwrap();
    let outline = rust(source.text(), RustEdition::E2024);
    let selection = PositionSelector::Byte {
        offset: source.text().find('α').unwrap(),
    }
    .select(source.text(), &outline, &[])
    .unwrap();
    let mut locator = SourceLocator {
        source: source.identity().clone(),
        range: selection.range,
    };
    assert_eq!(locator.select(&source).unwrap(), "// café\nfn α() {}");
    locator.range.line_start = 9;
    assert!(matches!(
        locator.select(&source),
        Err(SourceReadError::InvalidRange)
    ));
}

#[test]
fn acquisition_separates_working_index_and_resolved_committed_bytes_with_native_paths() {
    use std::{
        ffi::{OsStr, OsString},
        os::unix::ffi::OsStringExt as _,
        process::Command,
    };
    let root = tempfile::tempdir().unwrap();
    let git = |arguments: &[&OsStr]| {
        let output = Command::new("git")
            .current_dir(root.path())
            .args([
                "-c",
                "user.name=source fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "-c",
                "commit.gpgsign=false",
                "-c",
                "core.hooksPath=/dev/null",
            ])
            .args(arguments)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
    };
    git(&[
        OsStr::new("init"),
        OsStr::new("--quiet"),
        OsStr::new("--initial-branch=main"),
    ]);
    let path = std::path::PathBuf::from(OsString::from_vec(b"native-\xff.rs".to_vec()));
    std::fs::write(root.path().join(&path), "fn committed() {}\n").unwrap();
    git(&[OsStr::new("add"), OsStr::new("--"), path.as_os_str()]);
    git(&[
        OsStr::new("commit"),
        OsStr::new("--quiet"),
        OsStr::new("-m"),
        OsStr::new("source fixture"),
    ]);
    std::fs::write(root.path().join(&path), "fn staged() {}\n").unwrap();
    git(&[OsStr::new("add"), OsStr::new("--"), path.as_os_str()]);
    std::fs::write(root.path().join(&path), "fn working() {}\n").unwrap();
    let index_before = std::fs::read(root.path().join(".git/index")).unwrap();
    let committed = SourceRequest::Blob {
        repository: root.path().into(),
        revision: "HEAD".into(),
        path: path.clone(),
    }
    .acquire()
    .unwrap();
    let staged = SourceRequest::Index {
        repository: root.path().into(),
        path: path.clone(),
    }
    .acquire()
    .unwrap();
    let working = SourceRequest::Working {
        path: root.path().join(&path),
    }
    .acquire()
    .unwrap();
    assert_eq!(committed.text(), "fn committed() {}\n");
    assert_eq!(staged.text(), "fn staged() {}\n");
    assert_eq!(working.text(), "fn working() {}\n");
    assert_eq!(committed.identity().path(), path);
    assert_eq!(staged.identity().path(), path);
    assert_ne!(committed.identity(), staged.identity());
    let SourceOrigin::Blob { revision, blob, .. } = committed.identity().origin() else {
        panic!("committed kind")
    };
    assert_ne!(revision, "HEAD");
    assert_eq!(revision.len(), 40);
    assert_eq!(blob.len(), 40);
    assert_eq!(
        std::fs::read(root.path().join(".git/index")).unwrap(),
        index_before
    );
    assert!(
        SourceRequest::Index {
            repository: root.path().into(),
            path: "missing.rs".into()
        }
        .acquire()
        .is_err()
    );
    assert!(
        SourceRequest::Blob {
            repository: root.path().into(),
            revision: "HEAD".into(),
            path: "../native.rs".into()
        }
        .acquire()
        .is_err()
    );
    assert!(
        SourceRequest::Working {
            path: root.path().join("missing.rs")
        }
        .acquire()
        .is_err()
    );
    std::fs::write(root.path().join("binary.rs"), [0xff]).unwrap();
    assert!(
        SourceRequest::Working {
            path: root.path().join("binary.rs")
        }
        .acquire()
        .is_err()
    );
}
