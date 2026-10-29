// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use std::collections::BTreeSet;
use std::path::Path;

use sha2::{Digest, Sha256};

use super::headings::markdown_headings;
use super::metadata::{Document, Value, read, validate_fields};

fn parse(source: &str) -> Document<'_> {
    read(source.as_bytes(), "fixture.md", true, None, Path::new(".")).unwrap()
}

fn rejected(metadata: &str) {
    let source = format!("---\n{metadata}\n---\n");
    let error = read(source.as_bytes(), "fixture.md", true, None, Path::new("."))
        .expect_err("strict metadata must reject this fixture");
    assert!(error.to_string().starts_with("fixture.md:"));
}

#[test]
fn preserves_original_crlf_prefix_and_uninterpreted_body_bytes() {
    let source = b"---\r\ntitle: sample \xc3\xa9\r\ndoc_role: reference\r\n---\r\n# Body\r\n\xff";
    let document = read(source, "fixture.md", true, None, Path::new(".")).unwrap();
    assert_eq!(
        document.prefix,
        b"---\r\ntitle: sample \xc3\xa9\r\ndoc_role: reference\r\n---\r\n"
    );
    assert_eq!(document.body, b"# Body\r\n\xff");
    assert_eq!(document.metadata["title"].as_str(), Some("sample \u{e9}"));
}

#[test]
fn optional_frontmatter_requires_exact_delimiters() {
    for source in [b"body".as_slice(), b"--- \ntitle: text\n---\n", b""] {
        let document = read(source, "fixture", false, None, Path::new(".")).unwrap();
        assert!(document.metadata.is_empty());
        assert!(document.prefix.is_empty());
        assert_eq!(document.body, source);
        assert!(read(source, "fixture", true, None, Path::new(".")).is_err());
    }
    assert!(read(b"---\ntitle: text", "fixture", false, None, Path::new(".")).is_err());
    rejected("");
    rejected("[one, two]");
}

#[test]
fn rejects_duplicate_merge_and_nonstring_keys_recursively() {
    for metadata in [
        "title: one\ntitle: two",
        "nested: {key: one, key: two}",
        "nested: {1: one}",
        "nested: {true: one}",
        "nested: {2026-10-29: one}",
        "nested: {[one, two]: one}",
        "nested: {? {key: value}: one}",
        "base: &base {key: one}\nmerged: {<<: *base}",
        "date: &date 2026-10-29\nnested: {*date: one}",
    ] {
        rejected(metadata);
    }
}

#[test]
fn preserves_shared_aliases_but_rejects_recursive_aliases() {
    let document = parse("---\nbase: &base {name: value, count: 2}\ncopy: *base\n---\n");
    assert_eq!(document.metadata["base"], document.metadata["copy"]);
    rejected("cycle: &cycle [*cycle]");
    rejected("cycle: &cycle {again: *cycle}");
}

#[test]
fn refuses_custom_and_unconsumed_safe_yaml_tag_shapes() {
    // The consumed contract is scalar/list/string-keyed map metadata. These safe
    // YAML constructors previously produced Python-only bytes, sets or pairs;
    // no consuming documentation contract admits those extra value types.
    for metadata in [
        "value: !custom thing",
        "value: !timestamp 2026-10-29",
        "value: !!binary SGVsbG8=",
        "value: !!set {one: null}",
        "value: !!omap [{one: 1}]",
        "value: !!pairs [{one: 1}]",
        "nested: [!custom thing]",
    ] {
        rejected(metadata);
    }
    let document = parse(
        "---\ntext: !!str true\nnumber: !!int 2\nflag: !!bool true\nlist: !!seq [one]\n---\n",
    );
    assert_eq!(document.metadata["text"].as_str(), Some("true"));
    assert!(matches!(
        document.metadata["number"],
        Value::Integer(2) | Value::Unsigned(2)
    ));
    assert_eq!(document.metadata["flag"], Value::Bool(true));
}

#[test]
fn normalizes_dates_recursively_without_coercing_quoted_strings() {
    let document = parse(
        "---\ndate: &date 2026-10-29\ncopy: *date\nquoted: '2026-1-2'\nforced: !!str 2026-1-2\ntext: 2026-99-29suffix\njoined: 2026-10-2912:30:00\nnested: [2026-1-2, 2026-10-29T12:30:01.25Z, 2026-10-29 1:02:03-5:30]\n---\n",
    );
    assert_eq!(document.metadata["date"].as_str(), Some("2026-10-29"));
    assert_eq!(document.metadata["copy"].as_str(), Some("2026-10-29"));
    assert_eq!(document.metadata["quoted"].as_str(), Some("2026-1-2"));
    assert_eq!(document.metadata["forced"].as_str(), Some("2026-1-2"));
    assert_eq!(document.metadata["text"].as_str(), Some("2026-99-29suffix"));
    assert_eq!(
        document.metadata["joined"].as_str(),
        Some("2026-10-2912:30:00")
    );
    let values = document.metadata["nested"].as_sequence().unwrap();
    assert_eq!(values[0].as_str(), Some("2026-1-2"));
    assert_eq!(values[1].as_str(), Some("2026-10-29T12:30:01.250000+00:00"));
    assert_eq!(values[2].as_str(), Some("2026-10-29T01:02:03-05:30"));
}

#[test]
fn validates_explicit_timestamp_values_and_calendar_ranges() {
    let document = parse(
        "---\nvalue: !!timestamp '2026-10-29T12:30:01.123456789+0'\nanchored: &date !!timestamp 2026-10-29\ncopy: *date\nreordered: !!timestamp &other 2026-10-29\nother_copy: *other\n---\n",
    );
    assert_eq!(
        document.metadata["value"].as_str(),
        Some("2026-10-29T12:30:01.123456+00:00")
    );
    assert_eq!(document.metadata["anchored"], document.metadata["copy"]);
    assert_eq!(
        document.metadata["reordered"],
        document.metadata["other_copy"]
    );
    assert_eq!(document.metadata["anchored"].as_str(), Some("2026-10-29"));
    for metadata in [
        "value: 2026-02-30",
        "value: 0000-01-01",
        "value: 2026-10-29T25:00:00Z",
        "value: 2026-10-29T12:00:00+24:00",
        "value: !!timestamp arbitrary",
    ] {
        rejected(metadata);
    }
}

#[test]
fn retains_scalar_types_instead_of_string_coercion() {
    let document =
        parse("---\nzero: 0\nflag: false\nfloat: 1.5\nempty: null\nquoted: 'false'\n---\n");
    assert!(matches!(
        document.metadata["zero"],
        Value::Integer(0) | Value::Unsigned(0)
    ));
    assert_eq!(document.metadata["flag"], Value::Bool(false));
    assert_eq!(document.metadata["float"], Value::Float(1.5));
    assert_eq!(document.metadata["empty"], Value::Null);
    assert_eq!(document.metadata["quoted"].as_str(), Some("false"));
}

#[test]
fn validates_controlled_fields_and_declared_topic_membership() {
    for metadata in [
        "doc_extra: value",
        "doc_role: arbitrary",
        "doc_owner: ' '",
        "doc_topics: topic",
        "doc_topics: [one, one]",
        "doc_topics: [one, 2]",
    ] {
        rejected(metadata);
    }
    let document = parse(
        "---\ndoc_role: plan\ndoc_owner: docs/plans/example.md\ndoc_topics: [one, two]\n---\n",
    );
    assert!(
        validate_fields(
            &document.metadata,
            Some(&BTreeSet::from(["one".to_owned()]))
        )
        .is_err()
    );
    assert!(
        validate_fields(
            &document.metadata,
            Some(&BTreeSet::from(["one".to_owned(), "two".to_owned()]))
        )
        .is_ok()
    );
}

#[test]
fn historical_scalar_exception_requires_exact_path_key_and_hash() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("docs")).unwrap();
    let literal = " legacy: unquoted";
    let hash = Sha256::digest(literal.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let policy = format!(
        "[[legacy_yaml]]\npath = 'docs/adr/0001-example.md'\n[legacy_yaml.scalars]\ntitle = '{hash}'\n"
    );
    std::fs::write(root.path().join("docs/lifecycle.toml"), &policy).unwrap();
    let source = b"---\r\ntitle: legacy: unquoted\r\n---\r\nBody\r\n";
    let document = read(
        source,
        "fixture",
        true,
        Some("docs/adr/0001-example.md"),
        root.path(),
    )
    .unwrap();
    assert_eq!(
        document.metadata["title"].as_str(),
        Some("legacy: unquoted")
    );
    assert_eq!(document.prefix, &source[..source.len() - 6]);
    assert_eq!(document.body, b"Body\r\n");
    for identity in [
        None,
        Some("docs/adr/0002-other.md"),
        Some("docs/plans/0001-example.md"),
    ] {
        assert!(read(source, "fixture", true, identity, root.path()).is_err());
    }
    assert!(
        read(
            b"---\ntitle: changed: unquoted\n---\n",
            "fixture",
            true,
            Some("docs/adr/0001-example.md"),
            root.path()
        )
        .is_err()
    );
    std::fs::write(
        root.path().join("docs/lifecycle.toml"),
        format!("{policy}\n{policy}"),
    )
    .unwrap();
    assert!(
        read(
            source,
            "fixture",
            true,
            Some("docs/adr/0001-example.md"),
            root.path()
        )
        .is_err()
    );
}

#[test]
fn headings_use_real_fences_and_preserve_column_zero_atx_contract() {
    let source = "# Accepted\n\n```rust\n# Not a heading\n````\n\n~~~\n## Also fenced\n~~~\n\n    # Indented code\n\n  # Indented heading\n\n> # Quote\n\n- # List\n\nSetext\n------\n\n## Last ###\n";
    let headings = markdown_headings(source);
    assert_eq!(
        headings
            .iter()
            .map(|heading| heading.source)
            .collect::<Vec<_>>(),
        ["# Accepted", "## Last ###"]
    );
    assert_eq!(headings[0].line, 0);
    assert_eq!(source.lines().nth(headings[1].line), Some("## Last ###"));
}

#[test]
fn heading_offsets_follow_utf8_and_crlf_source_lines() {
    let source = "sample \u{e9}\r\n\r\n# First\r\n\r\n## Second\r\n";
    let headings = markdown_headings(source);
    assert_eq!(
        headings
            .iter()
            .map(|heading| (heading.line, heading.source))
            .collect::<Vec<_>>(),
        [(2, "# First"), (4, "## Second")]
    );
}
