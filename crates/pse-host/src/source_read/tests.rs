// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;

#[test]
fn inquiry_positions_keep_exact_operands_separate_from_declaration_identity() {
    let source = "fn f() { first; second; let s = \"é{value}\"; }";
    let first = source.find("first;").unwrap();
    let second_end = source.find("second;").unwrap() + "second;".len();
    assert_eq!(
        inquiry_anchor(source, first..first + "first;".len()),
        Some(first)
    );
    assert_eq!(inquiry_anchor(source, first..second_end), None);
    let string = source.find('"').unwrap();
    let value = source.find("{value}").unwrap();
    assert_eq!(
        inquiry_anchor(source, value..value + "{value}".len()),
        Some(string)
    );
    assert_eq!(inquiry_anchor(source, string + 2..string + 3), None);
    assert_eq!(inquiry_anchor(source, first..first), Some(first));
}

#[test]
fn raw_identifiers_have_logical_names_while_ranges_preserve_original_syntax() {
    let source = "mod r#type { pub fn r#match() {} }";
    let outline = rust(source, RustEdition::E2024);
    let function = outline
        .entries
        .iter()
        .find(|entry| entry.kind == "FN")
        .unwrap();
    assert_eq!(function.path, ["type", "match"]);
    assert_eq!(function.modules, ["type"]);
    assert_eq!(
        &source[function.range.byte_start..function.range.byte_end],
        "pub fn r#match() {}"
    );
}

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
    assert_eq!(
        semantic_anchor(source, source.find("pub fn").unwrap()..source.len()),
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
fn semantic_anchor_uses_unique_reexport_path_instead_of_alias_or_group_guess() {
    let source = "pub use dependency::original as facade;";
    assert_eq!(
        semantic_anchor(source, 0..source.len()),
        Some(source.find("original").unwrap())
    );
    let grouped = "pub use dependency::{first,second};";
    assert_eq!(semantic_anchor(grouped, 0..grouped.len()), None);
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
            detail: SourceDetail::Navigation,
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
            detail: SourceDetail::Navigation,
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
            detail: SourceDetail::Navigation,
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
                detail: SourceDetail::Navigation,
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
                detail: SourceDetail::Navigation,
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
#[test]
fn interpreted_facts_preserve_escapes_raw_delimiters_docs_and_cfg_predicates() {
    let source = r####"//! crate docs
/// outer docs
#[cfg_attr(feature = "x", cfg_attr(unix, doc = "nested"), allow(dead_code))]
fn actual() {
    let escaped = "a\n\u{3b2}";
    let raw = r##"raw\n"##;
    let bytes = c"\xFF";
    fn nested() { let inner = "owned"; }
}
fn broken(
"####;
    let outline = rust_detail(source, RustEdition::E2024, SourceDetail::Mapped);
    assert!(!outline.parse_issues.is_empty());
    assert_eq!(
        outline
            .facts
            .iter()
            .filter(|fact| fact.kind == "documentation")
            .count(),
        2
    );
    let escaped = outline
        .facts
        .iter()
        .find(|fact| fact.written == r#""a\n\u{3b2}""#)
        .unwrap();
    assert!(matches!(&escaped.value, Some(InterpretedValue::Text(value)) if value == "a\nβ"));
    let newline = &escaped.mapping.as_ref().unwrap()[1];
    assert_eq!(
        escaped.map_interpreted_range(1..2),
        Some(newline.source.clone())
    );
    assert_eq!(newline.interpreted, 1..2);
    assert_eq!(
        &source[newline.source.byte_start..newline.source.byte_end],
        r"\n"
    );
    let beta = &escaped.mapping.as_ref().unwrap()[2];
    assert_eq!(beta.interpreted, 2..4);
    assert_eq!(
        &source[beta.source.byte_start..beta.source.byte_end],
        r"\u{3b2}"
    );
    assert!(
        outline
            .facts
            .iter()
            .any(|fact| fact.written == "r##\"raw\\n\"##")
    );
    assert!(outline.facts.iter().any(
        |fact| matches!(&fact.value, Some(InterpretedValue::Bytes(value)) if value == &[255])
    ));
    let attribute = outline
        .facts
        .iter()
        .find_map(|fact| fact.meta.as_ref())
        .unwrap();
    assert_eq!(attribute.predicate.as_deref(), Some("feature = \"x\""));
    assert_eq!(attribute.children[0].predicate.as_deref(), Some("unix"));
    let owned = outline
        .facts
        .iter()
        .find(|fact| fact.written == "\"owned\"")
        .unwrap();
    let owner = owned.owner.as_ref().unwrap();
    assert!(source[owner.byte_start..owner.byte_end].starts_with("fn nested"));
}

#[test]
fn invalid_literal_interpretations_retain_spelling_without_fabricated_decoded_ranges() {
    let outline = rust_detail(
        r#"fn f(){ let invalid = "a\qz"; let valid = "β"; }"#,
        RustEdition::E2024,
        SourceDetail::Mapped,
    );
    let invalid = outline
        .facts
        .iter()
        .find(|fact| fact.written == r#""a\qz""#)
        .unwrap();
    assert!(invalid.interpretation_error.is_some());
    assert!(invalid.value.is_none() && invalid.mapping.as_ref().unwrap().is_empty());
    let valid = outline
        .facts
        .iter()
        .find(|fact| fact.written == "\"β\"")
        .unwrap();
    assert!(valid.map_interpreted_range(0..1).is_none());
    assert!(valid.map_interpreted_range(0..2).is_some());
}

#[test]
fn navigation_does_not_construct_facts_or_copy_unrelated_initializers_and_docs() {
    let huge = "x".repeat(200_000);
    let source = format!(
        "/// {huge}\nconst BIG: &str = \"{huge}\";\nfn selected() {{ let value = \"a\\nβ\"; }}"
    );
    facts::reset_construction();
    let (outline, count) =
        rust_bounded(&source, RustEdition::E2024, 0, 1, SourceDetail::Navigation);
    assert_eq!(count, 2);
    assert_eq!(outline.entries[0].signature, "const BIG: &str");
    assert!(outline.facts.is_empty());
    assert!(outline.detail_coverage.ranges.is_empty());
    assert!(!outline.detail_coverage.includes_file_facts);
    assert_eq!(facts::construction(), (0, 0));
    assert!(serde_json::to_vec(&outline).unwrap().len() < 2_000);
}

#[test]
fn selected_facts_precede_construction_and_mapping_demand() {
    let source = format!(
        "//! file docs\n/// {}\nconst BIG: &str = \"{}\";\n/// selected docs\nfn selected() {{ let value = \"a\\nβ\"; fn nested() {{ let n = \"excluded\"; }} }}",
        "d".repeat(80_000),
        "x".repeat(80_000)
    );
    let (outline, syntax) = rust_with_syntax(&source, RustEdition::E2024);
    let owner = outline
        .entries
        .iter()
        .find(|entry| entry.path == ["selected"])
        .unwrap()
        .range
        .clone();
    let owners = [owner.clone()];
    let selection = FactSelection::Owners {
        ranges: &owners,
        file: false,
    };
    facts::reset_construction();
    let selected = selected_facts(
        &syntax,
        &line_starts(&source),
        SourceDetail::Facts,
        &selection,
    );
    assert_eq!(facts::construction(), (2, 0));
    assert_eq!(selected.len(), 2);
    assert!(
        selected
            .iter()
            .all(|fact| fact.owner.as_ref() == Some(&owner) && fact.mapping.is_none())
    );
    assert_eq!(selected[0].written, "/// selected docs");
    assert!(matches!(&selected[1].value, Some(InterpretedValue::Text(text)) if text == "a\nβ"));
    facts::reset_construction();
    let mapped = selected_facts(
        &syntax,
        &line_starts(&source),
        SourceDetail::Mapped,
        &selection,
    );
    assert_eq!(
        facts::construction(),
        (2, " selected docs".chars().count() + 3)
    );
    let literal = &mapped[1];
    let ranges: Vec<_> = literal
        .mapping
        .as_ref()
        .unwrap()
        .iter()
        .map(|mapping| source[mapping.source.byte_start..mapping.source.byte_end].to_string())
        .collect();
    assert_eq!(ranges, ["a", "\\n", "β"]);
}

#[test]
fn selected_range_intersection_retains_complete_original_fact_and_file_docs_have_own_scope() {
    let source = "//! file\n/// item\n#[doc = \"owned\"]\nfn selected() { let s = \"abc\"; }";
    let (_, syntax) = rust_with_syntax(source, RustEdition::E2024);
    let lines = line_starts(source);
    let literal_start = source.find("\"abc\"").unwrap();
    let selected = location(&lines, literal_start + 2..literal_start + 3);
    let facts = selected_facts(
        &syntax,
        &lines,
        SourceDetail::Facts,
        &FactSelection::Range(&selected),
    );
    assert_eq!(facts.len(), 1);
    assert_eq!(
        facts[0].range,
        location(&lines, literal_start..literal_start + 5)
    );
    assert_eq!(facts[0].written, "\"abc\"");
    let root = selected_facts(
        &syntax,
        &lines,
        SourceDetail::Facts,
        &FactSelection::Owners {
            ranges: &[],
            file: true,
        },
    );
    assert_eq!(root.len(), 1);
    assert_eq!(root[0].written, "//! file");
    assert!(root[0].owner.is_none());
}

#[test]
fn detail_is_bound_into_guarded_continuation_and_public_schema() {
    let source = SourceText::new("/source/detail.rs".into(), "fn a() {} fn b() {}".into()).unwrap();
    let (_, page) = outline_page(
        &source,
        Some(RustEdition::E2024),
        PageRequest {
            detail: SourceDetail::Facts,
            max_records: Some(1),
            cursor: None,
        },
    )
    .unwrap();
    assert!(matches!(
        outline_page(
            &source,
            Some(RustEdition::E2024),
            PageRequest {
                detail: SourceDetail::Mapped,
                max_records: Some(1),
                cursor: page.next_cursor.clone()
            }
        ),
        Err(SourceReadError::InvalidCursor)
    ));
    let (next, _) = outline_page(
        &source,
        Some(RustEdition::E2024),
        PageRequest {
            detail: SourceDetail::Facts,
            max_records: Some(1),
            cursor: page.next_cursor,
        },
    )
    .unwrap();
    assert_eq!(next.detail_coverage.detail, SourceDetail::Facts);
    let schema = serde_json::to_value(schemars::schema_for!(Outline)).unwrap();
    let detail_schema = jsonschema::validator_for(&schema["$defs"]["SourceDetail"]).unwrap();
    for detail in ["navigation", "facts", "mapped"] {
        detail_schema.validate(&serde_json::json!(detail)).unwrap();
    }
    assert!(
        detail_schema
            .validate(&serde_json::json!("unknown"))
            .is_err()
    );
    let validator = jsonschema::validator_for(&schema).unwrap();
    for detail in [
        SourceDetail::Navigation,
        SourceDetail::Facts,
        SourceDetail::Mapped,
    ] {
        let outline = rust_detail(
            "fn selected() { let s = \"a\\nβ\"; }",
            RustEdition::E2024,
            detail,
        );
        validator
            .validate(&serde_json::to_value(outline).unwrap())
            .unwrap();
    }
    assert!(schema["properties"].get("detail_coverage").is_some());
    assert!(serde_json::from_str::<SourceDetail>("\"unknown\"").is_err());
}

#[test]
fn explicit_large_selected_value_retains_every_decoded_character() {
    let value = "β".repeat(20_000);
    let source = format!("const SELECTED: &str = \"{value}\";");
    facts::reset_construction();
    let outline = rust_detail(&source, RustEdition::E2024, SourceDetail::Mapped);
    assert_eq!(outline.facts.len(), 1);
    assert!(
        matches!(&outline.facts[0].value, Some(InterpretedValue::Text(decoded)) if decoded == &value)
    );
    assert_eq!(outline.facts[0].mapping.as_ref().unwrap().len(), 20_000);
    assert_eq!(facts::construction(), (1, 20_000));
    let last = outline.facts[0].mapping.as_ref().unwrap().last().unwrap();
    assert_eq!(&source[last.source.byte_start..last.source.byte_end], "β");
    assert_eq!(last.interpreted, 39_998..40_000);
}
