// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use serde_json::json;
fn write(root: &Path, path: &str, text: &str) -> PathBuf {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, text).unwrap();
    path
}
fn report(files: Vec<PathBuf>) -> Report {
    Report {
        version: 1,
        files,
        tool_version: None,
        tool_exit: None,
        diagnostics: vec![],
        passed: false,
        limitations: vec![],
    }
}
fn link(url: String, line: usize, message: &str) -> serde_json::Value {
    json!({"url":url,"status":{"text":message},"span":{"line":line,"column":1}})
}
#[test]
fn three_native_lychee_errors_preserve_selected_file_line_and_distinct_check_families() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let source = write(
        root,
        "probe.md",
        "# Probe\n\n[missing](missing.md)\n[heading](target.md#no-heading)\n[section](target.md#section-999-9)\n",
    );
    let target = write(root, "target.md", "# Target\n");
    write(
        root,
        "unrelated-historical.md",
        "[unrelated](also-missing.md)\n",
    );
    let files = vec![PathBuf::from("probe.md")];
    let mut report = report(files.clone());
    let missing = Url::from_file_path(root.join("missing.md"))
        .unwrap()
        .to_string();
    let target = Url::from_file_path(target).unwrap().to_string();
    let response = json!({"errors":3,"error_map":{source.to_str().unwrap():[
        link(missing,3,"File not found"),link(format!("{target}#no-heading"),4,"Cannot find fragment"),link(format!("{target}#section-999-9"),5,"Cannot find fragment")
    ]}});
    merge_links(
        root,
        &files,
        &serde_json::to_vec(&response).unwrap(),
        &mut report,
    )
    .unwrap();
    let mut selected: Vec<_> = report
        .diagnostics
        .iter()
        .map(|item| (item.line, item.code.as_str()))
        .collect();
    selected.sort();
    assert_eq!(
        selected,
        vec![
            (3, "docs.link"),
            (4, "docs.fragment"),
            (5, "docs.section_anchor")
        ]
    );
    assert!(report.diagnostics.iter().all(
        |finding| finding.file == Path::new("probe.md") && finding.category == Category::Scoped
    ));
}
#[test]
fn native_section_owner_accepts_virtual_published_anchor_only_on_its_actual_owner() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let source = write(root, "probe.md", "# Probe\n");
    let owner = write(
        root,
        "docs/authoritative_design/blueprint.md",
        "# Architecture\n\n## 9.1 Native owner\n",
    );
    let other = write(root, "other.md", "# Other\n");
    fs::create_dir_all(root.join("docs/authoritative_design/sections")).unwrap();
    let files = vec![PathBuf::from("probe.md")];
    let mut report = report(files.clone());
    let response = json!({"errors":2,"error_map":{source.to_str().unwrap():[
        link(format!("{}#section-9-1",Url::from_file_path(owner).unwrap()),2,"Cannot find fragment"),
        link(format!("{}#section-9-1",Url::from_file_path(other).unwrap()),3,"Cannot find fragment")
    ]}});
    merge_links(
        root,
        &files,
        &serde_json::to_vec(&response).unwrap(),
        &mut report,
    )
    .unwrap();
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(report.diagnostics[0].line, 3);
}
#[test]
fn clean_indexed_review_passes_native_metadata_and_opportunity_anchor_checks() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    write(
        root,
        "reviews/README.md",
        "# Reviews\n\n[Current](clean.md)\n",
    );
    write(
        root,
        "reviews/clean.md",
        "---\ntitle: Clean review\ndoc_role: review\n---\n# Clean\n\n<a id=\"O01\"></a>\n## O01 Useful capability\n",
    );
    let files = vec![PathBuf::from("reviews/clean.md")];
    let mut report = report(files.clone());
    native_checks(root, &files, &mut report).unwrap();
    assert!(report.diagnostics.is_empty());
    write(
        root,
        "reviews/clean.md",
        "---\ndoc_role: review\n---\n# Clean\n\n## O01 Missing explicit anchor\n",
    );
    native_checks(root, &files, &mut report).unwrap();
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(report.diagnostics[0].code, "docs.opportunity_anchor");
    write(
        root,
        "reviews/clean.md",
        "---\ndoc_role: review\n---\n# Clean\n\n```html\n<a id=\"O01\"></a>\n```\n## O01 Code is not an owner anchor\n",
    );
    let mut fenced = Report {
        diagnostics: vec![],
        ..report
    };
    native_checks(root, &files, &mut fenced).unwrap();
    assert_eq!(fenced.diagnostics.len(), 1);
    assert_eq!(fenced.diagnostics[0].code, "docs.opportunity_anchor");
}
#[test]
fn offline_external_and_immutable_historical_links_remain_explicitly_unchecked() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let source = write(root, "probe.md", "# Probe\n");
    let files = vec![PathBuf::from("probe.md")];
    let mut report = report(files.clone());
    let response = json!({"errors":0,"excluded_map":{source.to_str().unwrap():[
        link("https://example.invalid/not-fetched".into(),2,"Excluded"),
        link("https://github.com/example/project/blob/123456789abc/source.md".into(),3,"Excluded")
    ]}});
    merge_links(
        root,
        &files,
        &serde_json::to_vec(&response).unwrap(),
        &mut report,
    )
    .unwrap();
    assert_eq!(report.diagnostics[0].category, Category::External);
    assert_eq!(report.diagnostics[1].category, Category::Historical);
    assert!(
        report
            .diagnostics
            .iter()
            .all(|finding| finding.severity == Severity::Information)
    );
}
#[test]
fn malformed_tool_counts_and_unselected_sources_refuse_instead_of_expanding_scope() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let source = write(root, "other.md", "# Other\n");
    let files = vec![PathBuf::from("probe.md")];
    let mut report = report(files.clone());
    let response = json!({"errors":1,"error_map":{source.to_str().unwrap():[link("file:///missing".into(),1,"File not found")]}});
    assert!(
        merge_links(
            root,
            &files,
            &serde_json::to_vec(&response).unwrap(),
            &mut report
        )
        .is_err()
    );
    assert!(merge_links(root, &files, br#"{"errors":1,"error_map":{}}"#, &mut report).is_err());
}
#[test]
fn changed_selection_unions_index_worktree_and_untracked_markdown_without_historical_scan() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let git = |arguments: &[&str]| {
        let output = Command::new("git")
            .arg("-C")
            .arg(root)
            .args([
                "-c",
                "user.name=docs fixture",
                "-c",
                "user.email=docs@fixture.invalid",
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
    git(&["init", "--quiet"]);
    write(root, "staged.md", "# Original\n");
    write(root, "working.md", "# Original\n");
    write(root, "historical.md", "[unrelated](missing.md)\n");
    git(&["add", "."]);
    git(&["commit", "--quiet", "-m", "original"]);
    write(root, "staged.md", "# Staged\n");
    git(&["add", "staged.md"]);
    write(root, "staged.md", "# Original\n");
    write(root, "working.md", "# Working\n");
    write(root, "new.md", "# New\n");
    write(root, "unrelated.rs", "fn unchanged() {}\n");
    let files = selected(
        root,
        &Query {
            files: vec![],
            changed: true,
            schema: false,
        },
    )
    .unwrap();
    assert_eq!(
        files,
        vec![
            PathBuf::from("new.md"),
            PathBuf::from("staged.md"),
            PathBuf::from("working.md")
        ]
    );
}
#[test]
fn immutable_git_reference_is_classified_without_fetching_original_object() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    write(
        root,
        "probe.md",
        "# Probe\n\n[retained](git:123456789abc:docs/retired.md#O01)\n",
    );
    let files = vec![PathBuf::from("probe.md")];
    let mut report = report(files.clone());
    native_checks(root, &files, &mut report).unwrap();
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(report.diagnostics[0].line, 3);
    assert_eq!(report.diagnostics[0].category, Category::Historical);
    assert_eq!(report.diagnostics[0].severity, Severity::Information);
}
