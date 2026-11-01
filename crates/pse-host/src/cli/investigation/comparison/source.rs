// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Lossless written changes; lexical correspondence is never native identity.
use super::*;
use crate::source_read::{self, Entry, RustEdition};

#[derive(Debug, Serialize, JsonSchema)]
pub(super) struct Declaration {
    pub path: Vec<String>,
    pub kind: String,
    pub reference: ResultReference,
    pub header: String,
    pub written_sha256: String,
    pub body_sha256: String,
}
#[derive(Debug, Serialize, JsonSchema)]
pub(super) struct Side {
    pub reference: ResultReference,
    pub source_sha256: String,
    pub declarations: Vec<Declaration>,
    pub omitted_declarations: usize,
    pub parse_issues: Vec<source_read::ParseIssue>,
}
#[derive(Clone, Copy, Debug, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum Correspondence {
    QualifiedCandidate,
    Ambiguous,
    Unmatched,
}
#[derive(Debug, Serialize, JsonSchema)]
pub(super) struct Pair {
    pub before: Vec<usize>,
    pub after: Vec<usize>,
    pub correspondence: Correspondence,
    pub basis: String,
    pub written_changed: Option<bool>,
    pub header_changed: Option<bool>,
    pub body_changed: Option<bool>,
}
#[derive(Debug, Serialize, JsonSchema)]
pub(in crate::cli) struct Comparison {
    before: Observation<Side>,
    after: Observation<Side>,
    pairs: Vec<Pair>,
    omitted_pairs: usize,
    limitations: Vec<String>,
}
impl Comparison {
    pub(super) fn partial(&self) -> bool {
        self.before.value().is_none()
            || self.after.value().is_none()
            || self.omitted_pairs > 0
            || [&self.before, &self.after]
                .into_iter()
                .filter_map(Observation::value)
                .any(|s| s.omitted_declarations > 0 || !s.parse_issues.is_empty())
            || self
                .pairs
                .iter()
                .any(|p| p.correspondence != Correspondence::QualifiedCandidate)
    }
}
fn declaration(entry: &Entry, text: &str, reference: &ResultReference) -> Declaration {
    let mut selected = reference.clone();
    if let Locator::Source { range, .. } = &mut selected.locator {
        *range = Some(query::MatchRange {
            byte_start: entry.range.byte_start,
            byte_end: entry.range.byte_end,
            line_start: entry.range.line_start,
            line_end: entry.range.line_end,
        });
    }
    let body_start = entry
        .written_header
        .as_ref()
        .map_or(entry.range.byte_end, |range| range.byte_end);
    Declaration {
        path: entry.path.clone(),
        kind: entry.kind.clone(),
        reference: selected,
        header: entry.signature.clone(),
        written_sha256: source_read::source_digest(
            &text[entry.range.byte_start..entry.range.byte_end],
        ),
        body_sha256: source_read::source_digest(&text[body_start..entry.range.byte_end]),
    }
}
pub(super) fn read(
    root: &Path,
    reference: &ResultReference,
    limit: usize,
) -> Result<Side, HostError> {
    let Locator::Source { range, .. } = &reference.locator else {
        return Err(invalid("written comparison requires source references"));
    };
    let selected = range.clone();
    let mut whole = reference.clone();
    if let Locator::Source { range, .. } = &mut whole.locator {
        *range = None;
    }
    // Canonical hydration checks whole-source, byte witness, Git blob and root guards once.
    let record = query::providers::inspect(root, &whole.locator)?;
    if record.bytes.is_some() {
        return Err(invalid(
            "written declaration comparison requires UTF-8 Rust source",
        ));
    }
    let text = &record.context;
    if selected
        .as_ref()
        .is_some_and(|range| text.get(range.byte_start..range.byte_end).is_none())
    {
        return Err(invalid(
            "original selected source range is outside UTF-8 input",
        ));
    }
    let outline = source_read::rust(text, RustEdition::E2024);
    let original = record
        .reference
        .ok_or_else(|| invalid("source owner did not return original guards"))?;
    let entries = outline
        .entries
        .iter()
        .filter(|entry| {
            selected.as_ref().is_none_or(|range| {
                entry.range.byte_start < range.byte_end && entry.range.byte_end > range.byte_start
                    || range.byte_start == range.byte_end
                        && entry.range.byte_start <= range.byte_start
                        && range.byte_start < entry.range.byte_end
            })
        })
        .collect::<Vec<_>>();
    let omitted_declarations = entries.len().saturating_sub(limit);
    let declarations = entries
        .into_iter()
        .take(limit)
        .map(|entry| declaration(entry, text, &original))
        .collect();
    Ok(Side {
        reference: original,
        source_sha256: outline.source_sha256,
        declarations,
        omitted_declarations,
        parse_issues: outline.parse_issues,
    })
}
fn pair(before: Vec<usize>, after: Vec<usize>, basis: &str, a: &Side, b: &Side) -> Pair {
    let correspondence = if before.len() == 1 && after.len() == 1 {
        Correspondence::QualifiedCandidate
    } else if before.is_empty() || after.is_empty() {
        Correspondence::Unmatched
    } else {
        Correspondence::Ambiguous
    };
    let changes = if let ([old], [new]) = (before.as_slice(), after.as_slice()) {
        let a = &a.declarations[*old];
        let b = &b.declarations[*new];
        Some((
            a.written_sha256 != b.written_sha256,
            a.header != b.header,
            a.body_sha256 != b.body_sha256,
        ))
    } else {
        None
    };
    Pair {
        before,
        after,
        correspondence,
        basis: basis.into(),
        written_changed: changes.map(|c| c.0),
        header_changed: changes.map(|c| c.1),
        body_changed: changes.map(|c| c.2),
    }
}
pub(super) fn align(a: &Side, b: &Side, limit: usize) -> (Vec<Pair>, usize) {
    use std::collections::{BTreeMap, BTreeSet};
    let mut keys = BTreeMap::<(String, Vec<String>), (Vec<usize>, Vec<usize>)>::new();
    for (index, d) in a.declarations.iter().enumerate() {
        keys.entry((d.kind.clone(), d.path.clone()))
            .or_default()
            .0
            .push(index);
    }
    for (index, d) in b.declarations.iter().enumerate() {
        keys.entry((d.kind.clone(), d.path.clone()))
            .or_default()
            .1
            .push(index);
    }
    let mut pairs = Vec::new();
    let mut used_a = BTreeSet::new();
    let mut used_b = BTreeSet::new();
    for (before, after) in keys
        .values()
        .filter(|(a, b)| !a.is_empty() && !b.is_empty())
    {
        used_a.extend(before.iter().copied());
        used_b.extend(after.iter().copied());
        pairs.push(pair(
            before.clone(),
            after.clone(),
            "same written declaration kind and qualified lexical path; not identity",
            a,
            b,
        ));
    }
    // Moves/renames can nominate exact original content, but do not silently bind namesakes.
    for (index, old) in a
        .declarations
        .iter()
        .enumerate()
        .filter(|(i, _)| !used_a.contains(i))
    {
        let after = b
            .declarations
            .iter()
            .enumerate()
            .filter(|(i, new)| {
                !used_b.contains(i)
                    && old.kind == new.kind
                    && (old.written_sha256 == new.written_sha256
                        || old.path.last() == new.path.last()
                        || old.body_sha256 == new.body_sha256)
            })
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        pairs.push(pair(
            vec![index],
            after,
            "unmatched declaration content/name/body nomination; inspect both originals",
            a,
            b,
        ));
    }
    let unmatched = b
        .declarations
        .iter()
        .enumerate()
        .filter(|(i, _)| !used_b.contains(i) && !pairs.iter().any(|p| p.after.contains(i)))
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    for index in unmatched {
        pairs.push(pair(
            Vec::new(),
            vec![index],
            "no selected before declaration correspondence",
            a,
            b,
        ));
    }
    let mut reverse = BTreeMap::<usize, usize>::new();
    for pair in &pairs {
        for index in &pair.after {
            *reverse.entry(*index).or_default() += pair.before.len();
        }
    }
    for pair in &mut pairs {
        if !pair.before.is_empty()
            && pair
                .after
                .iter()
                .any(|index| reverse.get(index).is_some_and(|count| *count > 1))
        {
            pair.correspondence = Correspondence::Ambiguous;
            pair.written_changed = None;
            pair.header_changed = None;
            pair.body_changed = None;
        }
    }
    let omitted = pairs.len().saturating_sub(limit);
    pairs.truncate(limit);
    (pairs, omitted)
}
pub(super) fn inspect(request: &Request, before_root: &Path, after_root: &Path) -> Comparison {
    let before = Observation::from_result(read(before_root, &request.before, request.max_records));
    let after = Observation::from_result(read(after_root, &request.after, request.max_records));
    let (pairs, omitted_pairs) = match (before.value(), after.value()) {
        (Some(a), Some(b)) => align(a, b, request.max_records),
        _ => (Vec::new(), 0),
    };
    Comparison{before,after,pairs,omitted_pairs,limitations:vec!["Written Rust declarations and body/structural bytes only; cfg, macros, native dependency resolution and API acquisition were not examined.".into(),"Unguarded source selectors observe bytes during this inquiry and do not establish an earlier snapshot; returned source references bind those observed bytes. Supplied original guards are enforced.".into(),"Lexical paths, names and matching content nominate correspondence, including similarly named unrelated declarations; no cross-revision declaration identity is established.".into()]}
}

#[cfg(test)]
mod tests {
    use super::*;
    fn side(root: &Path, path: &str, text: &str) -> Side {
        std::fs::write(root.join(path), text).unwrap();
        read(
            root,
            &ResultReference::new(Locator::Source {
                path: PathBuf::from(path).into(),
                source: query::SourceKind::Working,
                range: None,
                guard: None,
            }),
            100,
        )
        .unwrap()
    }
    #[test]
    fn comparison_body_header_rename_and_namesake_candidates_remain_distinct() {
        let t = tempfile::tempdir().unwrap();
        let a = side(
            t.path(),
            "a.rs",
            "pub fn value() -> u8 { 1 }\nmod old { fn same() {} }\n",
        );
        let b = side(
            t.path(),
            "b.rs",
            "pub fn value() -> u8 { 2 }\nmod new { fn same() {} }\n",
        );
        let (pairs, _) = align(&a, &b, 100);
        let value = pairs.iter().find(|p| p.before == [0]).unwrap();
        assert_eq!(value.header_changed, Some(false));
        assert_eq!(value.body_changed, Some(true));
        assert!(pairs.iter().any(|p| p.basis.contains("nomination")));
        let c = side(
            t.path(),
            "c.rs",
            "mod first { fn same() {} } mod second { fn same() {} }",
        );
        let (_, omitted) = align(&a, &c, 1);
        assert!(omitted > 0);
        assert!(
            align(&a, &c, 100)
                .0
                .iter()
                .any(|p| p.correspondence == Correspondence::Ambiguous)
        );
    }
    #[test]
    fn comparison_hydration_refuses_changed_working_bytes() {
        let t = tempfile::tempdir().unwrap();
        let a = side(t.path(), "a.rs", "fn old() {}\n");
        std::fs::write(t.path().join("a.rs"), "fn new() {}\n").unwrap();
        assert!(read(t.path(), &a.reference, 100).is_err());
    }
    #[test]
    fn comparison_many_before_candidates_for_one_after_remain_ambiguous() {
        let t = tempfile::tempdir().unwrap();
        let before = side(t.path(), "before.rs", "fn a() {} fn b() {}\n");
        let after = side(t.path(), "after.rs", "fn c() {}\n");
        let pairs = align(&before, &after, 100).0;
        assert_eq!(pairs.len(), 2);
        assert!(
            pairs
                .iter()
                .all(|pair| pair.correspondence == Correspondence::Ambiguous
                    && pair.written_changed.is_none()
                    && pair.header_changed.is_none()
                    && pair.body_changed.is_none())
        );
    }
    #[test]
    fn comparison_historical_blob_keeps_original_bytes_after_worktree_change() {
        let t = tempfile::tempdir().unwrap();
        std::fs::write(t.path().join("a.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();
        for args in [
            vec!["init", "--quiet"],
            vec!["add", "a.rs"],
            vec![
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "--quiet",
                "-m",
                "original",
            ],
        ] {
            let output = std::process::Command::new("git")
                .args(args)
                .current_dir(t.path())
                .output()
                .unwrap();
            assert!(output.status.success(), "{output:?}");
        }
        let before = read(
            t.path(),
            &ResultReference::new(Locator::Source {
                path: PathBuf::from("a.rs").into(),
                source: query::SourceKind::Revision {
                    revision: "HEAD".into(),
                },
                range: None,
                guard: None,
            }),
            100,
        )
        .unwrap();
        let after = side(t.path(), "a.rs", "pub fn value() -> u8 { 2 }\n");
        assert_ne!(before.source_sha256, after.source_sha256);
        let pairs = align(&before, &after, 100).0;
        assert_eq!(pairs[0].header_changed, Some(false));
        assert_eq!(pairs[0].body_changed, Some(true));
        assert_eq!(
            read(t.path(), &before.reference, 100)
                .unwrap()
                .source_sha256,
            before.source_sha256
        );
    }
}
