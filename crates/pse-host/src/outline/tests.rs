// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use std::path::{Path, PathBuf};

use super::*;

fn repo_file(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

/// Line (1-based) of the first line equal to `needle`, located without the outline.
fn line_of(text: &str, needle: &str) -> usize {
    text.lines()
        .position(|line| line == needle)
        .map(|index| index + 1)
        .unwrap_or_else(|| panic!("`{needle}` not found"))
}

#[test]
fn show_returns_exactly_host_manager_observe_from_the_real_file() {
    let path = repo_file("crates/pse-host/src/manager.rs");
    let text = std::fs::read_to_string(&path).unwrap();
    let lines: Vec<&str> = text.lines().collect();

    // Independent expectation: the signature, its contiguous doc/attribute lines above, and
    // the first closing brace at the method's indentation below.
    let signature = line_of(
        &text,
        "    pub async fn observe(&self, name: &str) -> Result<UnitObservation, HostError> {",
    );
    let mut start = signature;
    while lines[start - 2].trim_start().starts_with("///")
        || lines[start - 2].trim_start().starts_with("#[")
    {
        start -= 1;
    }
    let end = signature
        + lines[signature..]
            .iter()
            .position(|line| *line == "    }")
            .unwrap()
        + 1;
    assert!(start < signature, "observe carries a doc comment");

    let item = show(&path, "HostManager::observe").unwrap();
    assert_eq!(item.kind, OutlineKind::Method);
    assert_eq!(item.path, "HostManager::observe");
    assert_eq!((item.start_line, item.end_line), (start, end));
    assert_eq!(item.text, lines[start - 1..end].join("\n"));

    let rendered = item.to_string();
    let first = rendered.lines().next().unwrap();
    assert_eq!(first, format!("{start} | {}", lines[start - 1]));
    assert_eq!(rendered.lines().count(), end - start + 1);

    // Other units' `observe` methods make the bare name ambiguous, with locations offered.
    let Err(OutlineError::Ambiguous { candidates, .. }) = show(&path, "observe") else {
        panic!("bare `observe` must be ambiguous");
    };
    assert!(candidates
        .iter()
        .any(|candidate| candidate.starts_with("HostManager::observe (method, lines ")));
    // A line hint picks the method containing that line.
    let hinted = show(&path, &format!("observe@{signature}")).unwrap();
    assert_eq!(hinted.path, "HostManager::observe");
}

#[test]
fn markdown_plan_outline_lists_headings_with_line_ranges() {
    let path = repo_file("docs/plans/59d-knowledge-and-workflow-verbs.md");
    let text = std::fs::read_to_string(&path).unwrap();
    let outline = outline_file(&path).unwrap();
    assert_eq!(outline.language, SourceLanguage::Markdown);
    assert!(outline.iter().all(|entry| entry.kind == OutlineKind::Heading));

    let d03 = outline.find("D03 — Structural reading").unwrap();
    let d03_line = line_of(&text, "### D03 — Structural reading");
    let d04_line = line_of(&text, "### D04 — Fast documentation checks");
    assert_eq!(d03.level, Some(3));
    assert_eq!(d03.start_line, d03_line);
    assert!(d03.end_line < d04_line && d03.end_line > d03.start_line);
    assert!(text.lines().nth(d03.end_line - 1).unwrap().trim().len() > 0);
    // The section is nested under the H2 that precedes it.
    assert!(d03.path.ends_with(" > D03 — Structural reading"), "{}", d03.path);

    // Every ATX heading line outside fences appears exactly once, in order; front matter
    // contributes none.
    let mut fenced = false;
    let expected: Vec<usize> = text
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            if line.starts_with("```") {
                fenced = !fenced;
            }
            (!fenced && line.starts_with('#') && line.contains("# ")).then_some(index + 1)
        })
        .collect();
    let found: Vec<usize> = outline.iter().map(|entry| entry.start_line).collect();
    assert_eq!(found, expected);

    let rendered = outline.to_string();
    assert!(rendered.contains(&format!("{d03_line}-{}", d03.end_line)));
}

#[test]
fn markdown_ignores_fenced_hashes_and_closes_sections() {
    let source = "---\ntitle: x\n---\n# Top\n\nintro\n\n```sh\n# not a heading\n```\n\n## A\n\ntext a\n\n## B\ntext b\n\n\n# Second\n";
    let outline = outline_source(SourceLanguage::Markdown, source).unwrap();
    let flat: Vec<(&str, usize, usize)> = outline
        .iter()
        .map(|entry| (entry.path.as_str(), entry.start_line, entry.end_line))
        .collect();
    assert_eq!(
        flat,
        vec![
            ("Top", 4, 17),
            ("Top > A", 12, 14),
            ("Top > B", 16, 17),
            ("Second", 20, 20),
        ]
    );
    let section = show_source(SourceLanguage::Markdown, source, "A").unwrap();
    assert_eq!(section.text, "## A\n\ntext a");
}

const RUST: &str = r#"//! Module docs.
use std::fmt;

/// A widget.
#[derive(Debug)]
pub struct Widget<T> {
    value: T,
}

impl<T> Widget<T> {
    /// Build one.
    pub fn new(value: T) -> Self {
        Self { value }
    }
    const LIMIT: usize = 3;
}

impl<T: fmt::Debug> fmt::Display for Widget<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

pub trait Shape {
    type Unit;
    fn area(&self) -> f64;
}

macro_rules! twice {
    ($e:expr) => { $e; $e };
}

fn helper() {}

#[cfg(test)]
mod tests {
    #[test]
    fn builds() {}

    #[tokio::test]
    async fn runs() {}

    fn helper() {}
}
"#;

#[test]
fn rust_outline_lists_items_impls_members_and_tests() {
    let outline = outline_source(SourceLanguage::Rust, RUST).unwrap();
    let flat: Vec<(OutlineKind, &str, usize, usize)> = outline
        .iter()
        .map(|entry| {
            (
                entry.kind,
                entry.path.as_str(),
                entry.start_line,
                entry.end_line,
            )
        })
        .collect();
    assert_eq!(
        flat,
        vec![
            (OutlineKind::Struct, "Widget", 4, 8),
            (OutlineKind::Impl, "impl Widget<T>", 10, 16),
            (OutlineKind::Method, "Widget::new", 11, 14),
            (OutlineKind::AssociatedConst, "Widget::LIMIT", 15, 15),
            (OutlineKind::Impl, "impl fmt::Display for Widget<T>", 18, 22),
            (OutlineKind::Method, "Widget::fmt", 19, 21),
            (OutlineKind::Trait, "Shape", 24, 27),
            (OutlineKind::AssociatedType, "Shape::Unit", 25, 25),
            (OutlineKind::Method, "Shape::area", 26, 26),
            (OutlineKind::Macro, "twice", 29, 31),
            (OutlineKind::Function, "helper", 33, 33),
            (OutlineKind::Module, "tests", 35, 44),
            (OutlineKind::Test, "tests::builds", 37, 38),
            (OutlineKind::Test, "tests::runs", 40, 41),
            (OutlineKind::Function, "tests::helper", 43, 43),
        ]
    );

    // Exact paths win over suffix matches: `helper` is the top-level function.
    assert_eq!(outline.find("helper").unwrap().start_line, 33);
    assert_eq!(outline.find("tests::helper").unwrap().start_line, 43);
    // A unique trailing segment resolves.
    assert_eq!(outline.find("builds").unwrap().path, "tests::builds");

    let Err(OutlineError::NotFound { suggestions, .. }) = outline.find("Gadget::new") else {
        panic!("unknown selector must not resolve");
    };
    assert_eq!(suggestions, vec!["Widget::new (method, lines 11-14)"]);

    let rendered = outline.to_string();
    assert!(rendered.contains("10-16  impl impl Widget<T>"), "{rendered}");
    assert!(rendered.contains("11-14    method new"), "{rendered}");
}

#[test]
fn rust_parse_failures_name_their_location() {
    let Err(OutlineError::Parse { line, origin, .. }) =
        outline_source(SourceLanguage::Rust, "fn ok() {}\nfn broken( {\n")
    else {
        panic!("invalid Rust must fail to parse");
    };
    assert_eq!(origin, "<source>");
    assert!(line >= 2);
}

#[test]
fn show_targets_split_after_the_file_extension() {
    let (file, selector) =
        parse_show_target("crates/pse-host/src/manager.rs::HostManager::observe").unwrap();
    assert_eq!(file, PathBuf::from("crates/pse-host/src/manager.rs"));
    assert_eq!(selector, "HostManager::observe");
    let (file, selector) = parse_show_target("docs/plans/x.md::D03 — Structural reading").unwrap();
    assert_eq!(file, PathBuf::from("docs/plans/x.md"));
    assert_eq!(selector, "D03 — Structural reading");
    assert!(parse_show_target("crates/pse-host/src/manager.rs").is_err());
    assert!(parse_show_target("notes.txt::x").is_err());
    assert!(matches!(
        outline_file(Path::new("Cargo.toml")),
        Err(OutlineError::Unsupported(_))
    ));
}
