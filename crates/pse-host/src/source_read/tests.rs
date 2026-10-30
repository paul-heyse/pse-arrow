// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;

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
