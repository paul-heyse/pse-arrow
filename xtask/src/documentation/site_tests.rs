// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;

struct Fixture {
    directory: tempfile::TempDir,
    config: Configuration,
}
impl Fixture {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let config: Configuration = toml::from_str(
            r#"
[tools]
mdbook = "0.5.4"
pagefind = "1.5.2"
[publication]
exclude = ["SUMMARY.md"]
current_work = ["plans/01*.md"]
entrypoints = ["README.md"]
[[collections]]
title = "Start"
root = "."
recursive = false
scope = "Current"
[[collections]]
title = "Architecture"
root = "authoritative_design"
scope = "Current"
[[collections]]
title = "Plans"
root = "plans"
scope = "History"
"#,
        )
        .unwrap();
        let fixture = Self { directory, config };
        fixture.write("docs/README.md", "# Start\n");
        fixture.write(
            "docs/plans/00-old.md",
            "---\nstatus: in-progress\n---\n# Old\n",
        );
        fixture.write("docs/plans/01-current.md", "# Current plan\n");
        fixture.write(
            "docs/authoritative_design/blueprint.md",
            "# Blueprint\n\n## 0.1 Contract\n",
        );
        fixture.write("docs/book.toml", "[book]\ntitle = 'Fixture'\n");
        fixture.write("docs/theme/search.css", "body{}\n");
        fixture
    }
    fn root(&self) -> &Path {
        self.directory.path()
    }
    fn write(&self, path: &str, text: &str) -> PathBuf {
        let path = self.root().join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, text).unwrap();
        path
    }
    fn pages(&self) -> Vec<Page> {
        discover(self.root(), &self.config).unwrap()
    }
}

#[test]
fn fnmatch_adapter_preserves_case_separator_globstar_and_literal_braces() {
    assert!(matches("a/b/file.md", &["*.md".into()]).unwrap());
    assert!(!matches("a/file.md", &["a/**/*.md".into()]).unwrap());
    assert!(matches("a/sub/file.md", &["a/**/*.md".into()]).unwrap());
    assert!(matches("a/{x,y}.md", &["a/{x,y}.md".into()]).unwrap());
    assert!(!matches("a/x.md", &["a/{x,y}.md".into()]).unwrap());
    assert!(!matches("File.md", &["file.md".into()]).unwrap());
    assert!(matches("[unfinished", &["[unfinished".into()]).unwrap());
}
#[test]
fn discovery_is_read_only_and_status_alone_does_not_promote_history() {
    let fixture = Fixture::new();
    fixture.write("docs/plans/02-added.md", "# Newly discovered\n");
    let pages = fixture.pages();
    let old = pages
        .iter()
        .find(|page| page.path == Path::new("plans/00-old.md"))
        .unwrap();
    assert_eq!(old.scope, "History");
    assert_eq!(old.status, "in-progress");
    assert_eq!(
        pages
            .iter()
            .find(|page| page.path == Path::new("plans/01-current.md"))
            .unwrap()
            .scope,
        "Current"
    );
    assert!(pages.iter().any(|page| page.title == "Newly discovered"));
    assert!(!fixture.root().join("docs/SUMMARY.md").exists());
}
#[test]
fn titles_ignore_fences_and_null_title_falls_back_to_real_h1() {
    let fixture = Fixture::new();
    fixture.write(
        "docs/example.md",
        "---\ntitle:\nstatus: proposed\n---\n```md\n# Example only\n```\n\n# Actual title\n",
    );
    assert_eq!(
        fixture
            .pages()
            .iter()
            .find(|page| page.path == Path::new("example.md"))
            .unwrap()
            .title,
        "Actual title"
    );
    fixture.write("docs/example.md", "```md\n# Example only\n```\n");
    assert!(
        discover(fixture.root(), &fixture.config)
            .unwrap_err()
            .to_string()
            .contains("scalar title or first H1")
    );
}
#[test]
fn duplicate_collections_missing_entrypoints_and_escape_paths_are_rejected() {
    let mut fixture = Fixture::new();
    fixture
        .config
        .collections
        .push(fixture.config.collections[0].clone());
    assert!(
        discover(fixture.root(), &fixture.config)
            .unwrap_err()
            .to_string()
            .contains("duplicate publication")
    );
    fixture.config.collections.pop();
    fixture
        .config
        .publication
        .entrypoints
        .push("missing.md".into());
    assert!(
        discover(fixture.root(), &fixture.config)
            .unwrap_err()
            .to_string()
            .contains("entrypoint")
    );
    fixture.config.publication.entrypoints.pop();
    fixture.config.collections[0].root = "../outside".into();
    assert!(
        discover(fixture.root(), &fixture.config)
            .unwrap_err()
            .to_string()
            .contains("escapes docs")
    );
}
#[test]
fn staging_delivers_only_selected_assets_and_preserves_authored_sources() {
    let mut fixture = Fixture::new();
    fixture.write("docs/plans/evidence.json", "{\"observation\":7}\n");
    fixture.write("docs/plans/private.json", "{\"retained\":true}\n");
    fixture.write(
        "docs/authoritative_design/blueprint.md",
        "# Blueprint\n\n```md\n## 0.1 Contract\n```\n\n## 0.1 Contract\n",
    );
    fixture
        .config
        .publication
        .assets
        .push("plans/evidence.json".into());
    let before: BTreeMap<_, _> = walk_files(&fixture.root().join("docs"), true, None)
        .unwrap()
        .into_iter()
        .map(|path| {
            let raw = fs::read(&path).unwrap();
            (path, raw)
        })
        .collect();
    let output = fixture.root().join("stage");
    let pages = stage(fixture.root(), &output, &fixture.pages(), &fixture.config).unwrap();
    assert!(output.join("plans/evidence.json").is_file());
    assert!(!output.join("plans/private.json").exists());
    assert!(
        fs::read_to_string(output.join("SUMMARY.md"))
            .unwrap()
            .contains("architecture-sections.md")
    );
    let blueprint = fs::read_to_string(output.join("authoritative_design/blueprint.md")).unwrap();
    assert!(blueprint.contains("```md\n## 0.1 Contract\n```"));
    assert_eq!(blueprint.matches("id=\"section-0-1\"").count(), 1);
    assert!(
        !fs::read_to_string(output.join("plans/00-old.md"))
            .unwrap()
            .starts_with("---")
    );
    assert_eq!(pages.len(), fixture.pages().len() + 1);
    for (path, raw) in before {
        assert_eq!(fs::read(path).unwrap(), raw);
    }
}
#[test]
fn attachments_require_explicit_finite_bundles_and_parse_fences() {
    let mut fixture = Fixture::new();
    fixture.write("docs/plans/chart.svg", "<svg/>\n");
    fixture.write(
        "docs/plans/01-current.md",
        "# Work\n\n![Chart](chart.svg)\n\n```md\n![Ignored](not-selected.json)\n```\n",
    );
    assert!(
        stage(
            fixture.root(),
            &fixture.root().join("missing"),
            &fixture.pages(),
            &fixture.config
        )
        .unwrap_err()
        .to_string()
        .contains("not explicitly selected")
    );
    fixture.config.asset_bundles.push(AssetBundle {
        name: "chart".into(),
        root: "plans".into(),
        files: vec!["chart.svg".into()],
    });
    fixture
        .config
        .publication
        .asset_bundles
        .push("chart".into());
    stage(
        fixture.root(),
        &fixture.root().join("selected"),
        &fixture.pages(),
        &fixture.config,
    )
    .unwrap();
    fixture.config.asset_bundles[0].files = vec!["../../outside".into()];
    assert!(
        selected_assets(fixture.root(), &fixture.config)
            .unwrap_err()
            .to_string()
            .contains("escapes bundle")
    );
}
#[test]
fn asset_sources_must_be_canonical_finite_and_unambiguous() {
    let mut fixture = Fixture::new();
    fixture.write("docs/plans/evidence.json", "{}");
    for relative in [
        "../docs/plans/evidence.json".to_owned(),
        fixture
            .root()
            .join("docs/plans/evidence.json")
            .to_string_lossy()
            .into_owned(),
    ] {
        fixture.config.publication.assets = vec![relative];
        assert!(
            selected_assets(fixture.root(), &fixture.config)
                .unwrap_err()
                .to_string()
                .contains("noncanonical")
        );
    }
    fixture.config.publication.assets =
        vec!["plans/evidence.json".into(), "plans/evidence.json".into()];
    assert!(
        selected_assets(fixture.root(), &fixture.config)
            .unwrap_err()
            .to_string()
            .contains("duplicate selected")
    );
}
#[test]
#[cfg(unix)]
fn symlinked_assets_and_collection_escapes_are_refused() {
    let mut fixture = Fixture::new();
    fixture.write("private.json", "{}");
    std::os::unix::fs::symlink(
        fixture.root().join("private.json"),
        fixture.root().join("docs/plans/link.json"),
    )
    .unwrap();
    fixture.config.publication.assets = vec!["plans/link.json".into()];
    assert!(selected_assets(fixture.root(), &fixture.config).is_err());
    std::os::unix::fs::symlink(fixture.root(), fixture.root().join("docs/escaped")).unwrap();
    fixture.config.collections[0].root = "escaped".into();
    assert!(discover(fixture.root(), &fixture.config).is_err());
}
#[test]
fn html_annotation_adds_scope_search_and_excludes_landing_alias() {
    let fixture = Fixture::new();
    let pages = vec![Page {
        path: "plans/example.md".into(),
        title: "A < B & \"C\"".into(),
        body: String::new(),
        collection: "Plans".into(),
        scope: "Reference".into(),
        status: "proposed".into(),
    }];
    let output = fixture.root().join("rendered");
    fs::create_dir_all(output.join("plans")).unwrap();
    fs::write(
        output.join("plans/example.html"),
        "<head></head><main>body</main>",
    )
    .unwrap();
    annotate(&output, &pages, fixture.root()).unwrap();
    let text = fs::read_to_string(output.join("plans/example.html")).unwrap();
    assert!(text.contains("scope:Reference"));
    assert!(text.contains("A &lt; B &amp; &quot;C&quot;"));
    assert!(text.contains("../theme/search.js"));
    assert!(
        !fs::read_to_string(output.join("index.html"))
            .unwrap()
            .contains("data-pagefind-body")
    );
}
#[test]
fn failed_publication_restores_previous_complete_site() {
    let fixture = Fixture::new();
    let destination = fixture.root().join("published");
    fs::create_dir(&destination).unwrap();
    fs::write(destination.join("old.html"), "old").unwrap();
    let scratch = fixture.root().join("work");
    fs::create_dir(&scratch).unwrap();
    assert!(publish(&scratch.join("missing"), &destination).is_err());
    assert_eq!(
        fs::read_to_string(destination.join("old.html")).unwrap(),
        "old"
    );
    let candidate = scratch.join("site");
    fs::create_dir(&candidate).unwrap();
    fs::write(candidate.join("new.html"), "new").unwrap();
    publish(&candidate, &destination).unwrap();
    assert!(destination.join("new.html").exists());
    assert!(!destination.join("old.html").exists());
}
