// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;

struct Fixture {
    directory: tempfile::TempDir,
}
impl Fixture {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        fs::create_dir_all(root.join("docs/adr")).unwrap();
        fs::create_dir_all(root.join("docs/authoritative_design/sections")).unwrap();
        fs::write(
            root.join("docs/authoritative_design/blueprint.md"),
            "# Blueprint\n\n## 1. Summary\n",
        )
        .unwrap();
        Self { directory }
    }
    fn root(&self) -> &Path {
        self.directory.path()
    }
    fn write(&self, relative: &str, text: &str) -> PathBuf {
        let path = self.root().join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, text).unwrap();
        path
    }
    fn record(&self, number: u32, replacements: &[(&str, &str)]) -> PathBuf {
        let mut front = BTreeMap::from([
            ("id", format!("ADR-{number:04}")),
            ("title", format!("Fixture {number}")),
            ("status", "accepted".to_owned()),
            ("date", "2026-09-25".to_owned()),
            ("deciders", "[fixture]".to_owned()),
            ("level", "decision".to_owned()),
            ("principles", "[AP-01]".to_owned()),
            ("blueprint", "[§1]".to_owned()),
            ("review", "not-required: fixture".to_owned()),
            ("evidence", "Proposed".to_owned()),
            ("supersedes", "[]".to_owned()),
            ("superseded-by", "null".to_owned()),
            ("revisit", "a fixture trigger".to_owned()),
            ("verification", "a fixture check".to_owned()),
        ]);
        for (key, value) in replacements {
            front.insert(key, (*value).to_owned());
        }
        let mut source = "---\n".to_owned();
        for (key, value) in front {
            let value = if value.contains(": ") {
                serde_json::to_string(&value).unwrap()
            } else {
                value
            };
            source.push_str(&format!("{key}: {value}\n"));
        }
        source.push_str("---\n\n## Context\n\nFixture.\n");
        self.write(
            &format!("docs/adr/{number:04}-fixture-{number}.md"),
            &source,
        )
    }
    fn git(&self, args: &[&str]) -> String {
        let (ok, output) = git(self.root(), args).unwrap();
        assert!(ok, "Git fixture failed: {args:?}");
        output.trim().to_owned()
    }
    fn commit(&self) -> String {
        if !self.root().join(".git").is_dir() {
            self.git(&["init", "-q", "-b", "main"]);
        }
        self.git(&["add", "-A"]);
        self.git(&[
            "-c",
            "commit.gpgsign=false",
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-q",
            "--allow-empty",
            "-m",
            "fixture",
        ]);
        self.git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        self.git(&["rev-parse", "HEAD"])
    }
    fn lint(&self) -> LintReport {
        index(self.root(), false).unwrap();
        inspect(self.root()).unwrap()
    }
}

#[test]
fn index_is_deterministic_and_check_mode_never_publishes() {
    let fixture = Fixture::new();
    fixture.record(4, &[]);
    fixture.record(1, &[]);
    let rendered = render_index(fixture.root()).unwrap();
    assert!(rendered.find("[ADR-0001]").unwrap() < rendered.find("[ADR-0004]").unwrap());
    assert!(rendered.contains("| accepted | 2026-09-25 | §1 | a fixture trigger |"));
    let path = fixture.root().join("docs/adr/README.md");
    assert!(index(fixture.root(), true).is_err());
    assert!(!path.exists());
    index(fixture.root(), false).unwrap();
    let original = fs::read(&path).unwrap();
    index(fixture.root(), true).unwrap();
    assert_eq!(fs::read(&path).unwrap(), original);
    fs::write(&path, "stale").unwrap();
    assert!(index(fixture.root(), true).is_err());
    assert_eq!(fs::read_to_string(path).unwrap(), "stale");
}

#[test]
fn sparse_retired_references_use_known_history_or_explicit_fallback() {
    let fixture = Fixture::new();
    fixture.record(1, &[]);
    fixture.record(4, &[("supersedes", "[ADR-0002, ADR-0003]")]);
    let report = fixture.lint();
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    assert!(!report.notes.is_empty());
    assert_eq!(next_number(fixture.root()).unwrap(), 5);
    fixture.commit();
    let errors = fixture.lint().errors;
    assert!(
        errors
            .iter()
            .any(|e| e.contains("supersedes unknown record ADR-0002"))
    );
}

#[test]
fn duplicate_unknown_and_asymmetric_retained_records_are_reported() {
    let fixture = Fixture::new();
    fixture.record(
        1,
        &[("status", "superseded"), ("superseded-by", "ADR-0002")],
    );
    let second = fixture.record(2, &[("supersedes", "[ADR-0009]")]);
    fixture.write(
        "docs/adr/0002-duplicate.md",
        &fs::read_to_string(second).unwrap(),
    );
    let errors = fixture.lint().errors;
    assert!(
        errors
            .iter()
            .any(|e| e.contains("issued more than once: [2]"))
    );
    assert!(errors.iter().any(|e| e.contains("unknown record ADR-0009")));
    assert!(
        errors
            .iter()
            .any(|e| e.contains("does not name it in supersedes"))
    );
}

#[test]
fn symmetric_retained_supersession_and_retired_predecessors_are_valid() {
    let fixture = Fixture::new();
    fixture.record(
        1,
        &[("status", "superseded"), ("superseded-by", "ADR-0002")],
    );
    let second = fixture.record(2, &[("supersedes", "[ADR-0001]")]);
    assert!(fixture.lint().errors.is_empty());
    fs::remove_file(second).unwrap();
    fixture.record(3, &[]);
    assert!(
        fixture
            .lint()
            .errors
            .iter()
            .any(|e| e.contains("retire the superseded record too"))
    );
}

#[test]
fn accepted_fields_and_body_are_immutable_but_history_is_append_only() {
    let fixture = Fixture::new();
    let path = fixture.record(1, &[]);
    let mut source = fs::read_to_string(&path).unwrap();
    source.push_str("\n## Status history\n\n- 2026-09-25 — accepted.\n");
    fs::write(&path, &source).unwrap();
    fixture.commit();
    fs::write(&path, format!("{source}- 2026-09-26 — note.\n")).unwrap();
    assert!(
        lint_immutability(fixture.root(), std::slice::from_ref(&path), &mut Vec::new())
            .unwrap()
            .is_empty()
    );
    fs::write(&path, source.replace("accepted.", "proposed.")).unwrap();
    assert!(
        lint_immutability(fixture.root(), std::slice::from_ref(&path), &mut Vec::new())
            .unwrap()
            .iter()
            .any(|e| e.contains("append-only"))
    );
    fs::write(
        &path,
        source
            .replace("Fixture 1", "Rewritten")
            .replace("Fixture.", "Altered."),
    )
    .unwrap();
    let errors = lint_immutability(fixture.root(), &[path], &mut Vec::new()).unwrap();
    assert!(errors.iter().any(|e| e.contains("title changed")));
    assert!(
        errors
            .iter()
            .any(|e| e.contains("section 'Context' changed"))
    );
}

#[test]
fn proposed_records_remain_editable() {
    let fixture = Fixture::new();
    let path = fixture.record(1, &[("status", "proposed")]);
    fixture.commit();
    fixture.record(1, &[("title", "Rewritten")]);
    assert!(
        lint_immutability(fixture.root(), &[path], &mut Vec::new())
            .unwrap()
            .is_empty()
    );
}

#[test]
fn review_relocation_requires_removed_identical_reachable_source_and_fragment() {
    let fixture = Fixture::new();
    let review = fixture.write("docs/review.md", "# Review\n");
    let path = fixture.record(1, &[("review", "docs/review.md#f1")]);
    let commit = fixture.commit();
    let relocated = format!("git:{}:docs/review.md#f1", &commit[..12]);
    fixture.record(1, &[("review", &relocated)]);
    assert!(
        !lint_immutability(fixture.root(), std::slice::from_ref(&path), &mut Vec::new())
            .unwrap()
            .is_empty()
    );
    fs::remove_file(review).unwrap();
    assert!(
        lint_immutability(fixture.root(), std::slice::from_ref(&path), &mut Vec::new())
            .unwrap()
            .is_empty()
    );
    fixture.record(1, &[("review", &relocated.replace("#f1", "#f2"))]);
    assert!(
        !lint_immutability(fixture.root(), &[path], &mut Vec::new())
            .unwrap()
            .is_empty()
    );
}

#[test]
fn review_relocation_rejects_old_content_and_unreachable_commits() {
    let fixture = Fixture::new();
    fixture.write("docs/review.md", "# Draft\n");
    fixture.record(2, &[]);
    let early = fixture.commit();
    fixture.write("docs/review.md", "# Final\n");
    let path = fixture.record(1, &[("review", "docs/review.md")]);
    fixture.commit();
    fs::remove_file(fixture.root().join("docs/review.md")).unwrap();
    fixture.record(
        1,
        &[("review", &format!("git:{}:docs/review.md", &early[..12]))],
    );
    assert!(
        !lint_immutability(fixture.root(), std::slice::from_ref(&path), &mut Vec::new())
            .unwrap()
            .is_empty()
    );
    fixture.git(&["checkout", "-q", "-b", "side"]);
    fixture.write("docs/review.md", "# Final\n");
    fixture.git(&["add", "-A"]);
    fixture.git(&[
        "-c",
        "commit.gpgsign=false",
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "commit",
        "-q",
        "-m",
        "side",
    ]);
    let unreachable = fixture.git(&["rev-parse", "HEAD"]);
    fixture.git(&["checkout", "-q", "main"]);
    fixture.git(&["branch", "-D", "side"]);
    fixture.record(
        1,
        &[(
            "review",
            &format!("git:{}:docs/review.md", &unreachable[..12]),
        )],
    );
    assert!(
        !lint_immutability(fixture.root(), &[path], &mut Vec::new())
            .unwrap()
            .is_empty()
    );
}

#[test]
fn body_and_scenario_permalinks_allow_only_same_path_source_relocation() {
    let fixture = Fixture::new();
    let plan = fixture.write("docs/plans/01-old.md", "# Old plan\n");
    let path = fixture.record(1, &[("scenarios", "[docs/plans/01-old.md#outcome]")]);
    let original = format!(
        "{}See [plan](../plans/01-old.md#outcome).\n",
        fs::read_to_string(&path).unwrap()
    );
    fs::write(&path, &original).unwrap();
    let commit = fixture.commit();
    let link = format!("https://github.com/owner/repo/blob/{commit}/docs/plans/01-old.md");
    let relocated = original.replace("../plans/01-old.md", &link).replace(
        "[docs/plans/01-old.md#outcome]",
        &format!("[{link}#outcome]"),
    );
    fs::write(&path, &relocated).unwrap();
    assert!(
        !lint_immutability(fixture.root(), std::slice::from_ref(&path), &mut Vec::new())
            .unwrap()
            .is_empty()
    );
    fs::remove_file(plan).unwrap();
    assert!(
        lint_immutability(fixture.root(), std::slice::from_ref(&path), &mut Vec::new())
            .unwrap()
            .is_empty()
    );
    fs::write(&path, relocated.replace("01-old.md", "02-other.md")).unwrap();
    let errors = lint_immutability(fixture.root(), &[path], &mut Vec::new()).unwrap();
    assert!(errors.iter().any(|e| e.contains("names no file")));
    assert!(
        errors
            .iter()
            .any(|e| e.contains("changed on an accepted record"))
    );
}

#[test]
fn new_allocation_remembers_retired_highest_id_and_uses_authored_template() {
    let fixture = Fixture::new();
    fixture.record(1, &[]);
    let third = fixture.record(3, &[]);
    fixture.commit();
    fs::remove_file(third).unwrap();
    fixture.commit();
    assert_eq!(next_number(fixture.root()).unwrap(), 4);
    assert!(
        fixture
            .lint()
            .errors
            .iter()
            .any(|e| e.contains("highest issued record"))
    );
    fixture.write(
        "docs/adr/template.md",
        "---\nid: ADR-NNNN\ntitle: TITLE\ndate: YYYY-MM-DD\n---\nBody\n",
    );
    let path = new_record(fixture.root(), "new-record", "Decision title", "2026-10-29").unwrap();
    let source = fs::read_to_string(path).unwrap();
    assert!(source.contains("id: ADR-0004"));
    assert!(source.contains("title: Decision title"));
    assert!(source.contains("date: 2026-10-29"));
    assert!(new_record(fixture.root(), "Invalid", "title", "2026-10-29").is_err());
}

#[test]
fn supersede_preflights_both_records_and_preserves_untouched_source_bytes() {
    let fixture = Fixture::new();
    let old = fixture.record(1, &[]);
    let new = fixture.record(2, &[]);
    let original = fs::read_to_string(&old).unwrap().replace('\n', "\r\n");
    fs::write(&old, &original).unwrap();
    let (before, after) = supersede(fixture.root(), "ADR-0001", "2", "2026-10-29").unwrap();
    assert_eq!((before.as_str(), after.as_str()), ("ADR-0001", "ADR-0002"));
    let source = fs::read_to_string(&old).unwrap();
    assert!(source.contains("status: superseded\r\n"));
    assert!(source.contains("superseded-by: ADR-0002\r\n"));
    assert!(source.ends_with("- 2026-10-29 — superseded by ADR-0002.\n"));
    assert!(
        fs::read_to_string(new)
            .unwrap()
            .contains("supersedes: [ADR-0001]")
    );
    assert!(supersede(fixture.root(), "1", "1", "2026-10-29").is_err());
}

#[test]
fn section_ownership_ignores_examples_and_rejects_duplicate_identities() {
    let fixture = Fixture::new();
    fixture.write(
        "docs/authoritative_design/sections/one.md",
        "```md\n## 99. Example\n```\n## 1.1. Child\n",
    );
    let blueprint = fixture
        .root()
        .join("docs/authoritative_design/blueprint.md");
    let owners = section_owners(&blueprint).unwrap();
    assert!(owners.contains_key("1.1"));
    assert!(!owners.contains_key("99"));
    fixture.write(
        "docs/authoritative_design/sections/two.md",
        "## 1.1. Duplicate\n",
    );
    assert!(
        section_owners(&blueprint)
            .unwrap_err()
            .to_string()
            .contains("duplicate section §1.1")
    );
}

#[test]
fn shallow_history_keeps_shape_only_limitations_explicit() {
    let fixture = Fixture::new();
    fixture.record(1, &[]);
    let commit = fixture.commit();
    fs::write(fixture.root().join(".git/shallow"), format!("{commit}\n")).unwrap();
    assert!(issued_history(fixture.root(), "HEAD").unwrap().is_none());
    let mut notes = Vec::new();
    assert!(
        review_error(
            fixture.root(),
            "git:abcdefabcdef:docs/retired.md",
            &mut notes
        )
        .unwrap()
        .is_none()
    );
    assert!(notes.iter().any(|note| note.contains("shape")));
    assert!(
        review_error(fixture.root(), "git:HEAD:docs/retired.md", &mut notes)
            .unwrap()
            .is_some()
    );
}
