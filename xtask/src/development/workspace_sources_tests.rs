// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use serde_json::json;

struct Fixture(tempfile::TempDir);
impl Fixture {
    fn new() -> Self {
        let fixture = Self(tempfile::tempdir().unwrap());
        fixture.write(SITE, "[publication]\ncurrent_work=['plans/58-work.md']\n[[collections]]\ntitle='Architecture'\nroot='authoritative_design'\nscope='Current'\n[[collections]]\ntitle='Plans'\nroot='plans'\nscope='History'\n[[collections]]\ntitle='Decisions'\nroot='adr'\nscope='History'\n");
        fixture.write(POLICY, "version=1\ntopics=['boundary']\n");
        fixture
    }
    fn root(&self) -> &Path { self.0.path() }
    fn write(&self, path: &str, text: &str) {
        let path = self.root().join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    fn selection(&self, paths: &[&str]) -> SourceSelection {
        SourceSelection { paths: paths.iter().map(|path| (*path).into()).collect(), ..SourceSelection::default() }
    }
    fn capture(&self, paths: &[&str]) -> SourceCut {
        bounded_source_cut(self.root(), &self.selection(paths)).unwrap()
    }
    fn architecture(&self) {
        self.write(BLUEPRINT, "---\ndoc_role: contract\ndoc_owner: docs/authoritative_design/blueprint.md\n---\n# Blueprint\n");
        self.write("docs/authoritative_design/sections/boundary.md", "---\ndoc_role: contract\ndoc_owner: docs/authoritative_design/sections/boundary.md\ndoc_topics: [boundary]\n---\n# Boundary\n\n## 23.2 Admission\n\nReject invalid values.\n\n```md\n## 99.9 Fake\n```\n\n## 23.3 Publication\n\nOnly admitted values publish.\n");
    }
}
fn subject<'a>(build: &'a SourceCorpusBuild, id: &str) -> &'a KnowledgeSubject {
    build.corpus.subjects.iter().find(|subject| subject.id == id).unwrap()
}
#[test]
fn completed_cut_replays_metadata_owner_anchors_and_sections_without_live_files() {
    let fixture = Fixture::new(); fixture.architecture();
    fixture.write("docs/review.md", "---\ndoc_role: review\ndoc_owner: docs/authoritative_design/sections/boundary.md#section-23-2\n---\n# Review\nCaptured proposal.\n");
    let cut = fixture.capture(&["docs/authoritative_design/sections/boundary.md", "docs/review.md"]);
    let original = build_corpus(&cut).unwrap();
    fs::remove_dir_all(fixture.root().join("docs")).unwrap();
    let absent = build_corpus(&cut).unwrap();
    assert_eq!(original.corpus.generation().unwrap(), absent.corpus.generation().unwrap());
    // A new, conflicting live owner and policy cannot alter a completed cut.
    fixture.write(POLICY, "version=99\n");
    fixture.write("docs/authoritative_design/sections/boundary.md", "# Conflicting live owner\n");
    let conflicting = build_corpus(&cut).unwrap();
    assert_eq!(original.corpus.generation().unwrap(), conflicting.corpus.generation().unwrap());
    let section = subject(&conflicting, "contract:23.2");
    assert_eq!(section.native_id, "23.2");
    assert_eq!(section.document_role.unwrap().as_str(), "contract");
    assert_eq!(section.publication_scope.as_deref(), Some("Current"));
    assert_eq!(section.topics[0].topic, "boundary");
    assert!(section.sources.iter().any(|source| source.source_id == POLICY && source.role.as_str() == "interpretation_policy"));
    assert!(!conflicting.corpus.subjects.iter().any(|subject| subject.native_id == "99.9"));
    assert!(conflicting.unknowns.iter().any(|unknown| unknown.contains("typed fragment relationship undeclared")));
    assert!(conflicting.omissions.is_empty());
    conflicting.corpus.validate().unwrap();
}
#[test]
fn exact_section_extents_do_not_attribute_sibling_or_fenced_headings() {
    let fixture = Fixture::new(); fixture.architecture();
    let cut = fixture.capture(&["docs/authoritative_design/sections/boundary.md"]);
    let build = build_corpus(&cut).unwrap();
    let extent = build.ranges.iter().find(|range| range.subject_id == "contract:23.2").unwrap();
    let raw = std::str::from_utf8(cut.sources().get(&extent.source_id).unwrap()).unwrap();
    assert_eq!(raw.lines().nth(extent.range.start_line - 1).unwrap(), "## 23.2 Admission");
    assert_eq!(raw.lines().nth(extent.range.end_line).unwrap(), "## 23.3 Publication");
    let attributed = build.corpus.excerpts.iter().filter(|excerpt| excerpt.subject_id.as_deref() == Some("contract:23.2"))
        .map(|excerpt| excerpt.text.as_str()).collect::<Vec<_>>().join("\n");
    assert!(attributed.contains("Reject invalid values."));
    assert!(!attributed.contains("Only admitted values publish."));
    assert!(!build.corpus.subjects.iter().any(|subject| subject.id == "contract:99.9"));
}
#[test]
fn native_packages_keep_status_owner_chain_and_physical_table_rows_offline() {
    let fixture = Fixture::new();
    fixture.write("docs/plans/58-work.md", "---\ndoc_role: plan\nstatus: in-progress\n---\n# Work\n\n## Packages\n\n| Package | Status |\n|---|---|\n| F01 | done |\n");
    fixture.write("docs/plans/59-owner.md", "---\ndoc_role: plan\nstatus: proposed\n---\n# Owner\n\n## Packages\n\n| Package | Status |\n|---|---|\n| A01 | blocked |\n");
    fixture.write(POLICY, r#"version=1
topics=['boundary']
[status_profiles.native]
order=['blocked','complete']
blocked=['blocked']
complete=['^done$']
[[scope_tables]]
path='docs/plans/58-work.md'
name='packages'
heading='Packages'
headers=['Package','Status']
kind='package'
id_column=0
status_column=1
status_profile='native'
[[scope_tables]]
path='docs/plans/59-owner.md'
name='packages'
heading='Packages'
headers=['Package','Status']
kind='package'
id_column=0
status_column=1
status_profile='native'
[[relationships]]
from='docs/plans/58-work.md::packages::F01'
kind='status-owned-by'
to='docs/plans/59-owner.md::packages::A01'
"#);
    let cut = fixture.capture(&["docs/plans/58-work.md"]);
    fs::remove_dir_all(fixture.root().join("docs")).unwrap();
    let build = build_corpus(&cut).unwrap();
    let package = subject(&build, "plan_package:docs/plans/58-work.md::packages::F01");
    assert_eq!(package.native_state.as_deref(), Some("blocked"));
    assert_eq!(package.state_basis.as_deref(), Some("canonical owner through explicit status-owned-by relationship"));
    assert_eq!(package.publication_scope.as_deref(), Some("Current"));
    assert!(package.sources.iter().any(|source| source.source_id == "docs/plans/59-owner.md" && source.role.as_str() == "native_scope"));
    assert!(build.corpus.edges.iter().any(|edge| edge.from == package.id && edge.relation.as_str() == "status_owned_by"));
    let extent = build.ranges.iter().find(|range| range.subject_id == package.id).unwrap();
    let raw = std::str::from_utf8(cut.sources().get(&extent.source_id).unwrap()).unwrap();
    assert_eq!(extent.range.start_line, extent.range.end_line);
    assert!(raw.lines().nth(extent.range.start_line - 1).unwrap().contains("| F01 | done |"));
    assert!(build.omissions.is_empty()); build.corpus.validate().unwrap();
}
#[test]
fn adr_native_identity_acceptance_and_supersession_remain_independent_from_scope() {
    let fixture = Fixture::new();
    fixture.write("docs/adr/0001-first.md", "---\nid: ADR-0001\ntitle: First\nstatus: superseded\ndoc_role: decision\nsuperseded-by: ADR-0002\n---\n# First\n");
    fixture.write("docs/adr/0002-second.md", "---\nid: ADR-0002\ntitle: Second\nstatus: proposed\ndoc_role: decision\nsupersedes: [ADR-0001]\n---\n# Second\n");
    let cut = fixture.capture(&["docs/adr/0001-first.md", "docs/adr/0002-second.md"]);
    fs::remove_file(fixture.root().join(POLICY)).unwrap();
    let build = build_corpus(&cut).unwrap();
    assert_eq!(subject(&build, "decision:ADR-0002").native_state.as_deref(), Some("proposed"));
    assert_eq!(subject(&build, "decision:ADR-0002").publication_scope.as_deref(), Some("Reference"));
    assert_eq!(subject(&build, "decision:ADR-0001").publication_scope.as_deref(), Some("History"));
    assert_eq!(build.corpus.edges.iter().filter(|edge| edge.relation.as_str() == "superseded_by").count(), 1);
}
#[test]
fn producer_triples_do_not_form_a_cross_product_and_govern_only_declared_outputs() {
    let fixture = Fixture::new(); fixture.architecture();
    fixture.write("src/declarations.rs", "struct First;\nstruct Second;\n");
    fixture.write("src/emitter.rs", "fn generate() {}\n");
    fixture.write("docs/generated/first.rs", "// first\n");
    fixture.write("docs/generated/second.rs", "// second\n");
    let row = |key: &str, output: &str, sections: Vec<&str>| json!({
        "declaration_key":key,"declaration_kind":"fixture","source_path":"src/declarations.rs",
        "source_symbol":key,"source_role":"type_definition","source_identity":"fixture-v1",
        "generator_path":"src/emitter.rs","generator_symbol":"generate","output_path":output,
        "direction":"output","contract_sections":sections,"basis":"declared"
    });
    fixture.write(&format!("{PRODUCERS}/fixture/relationships.json"), &json!({"schema_version":1,"producer":"fixture",
        "relationships":[row("First","docs/generated/first.rs",vec!["23.2"]),row("Second","docs/generated/second.rs",vec![])]}).to_string());
    let cut = fixture.capture(&["src/declarations.rs"]);
    fs::remove_dir_all(fixture.root().join("src")).unwrap();
    fs::remove_dir_all(fixture.root().join("docs")).unwrap();
    let build = build_corpus(&cut).unwrap();
    assert_eq!(build.corpus.productions.len(), 2);
    assert!(build.corpus.productions.iter().all(|production| {
        (production.declaration_id.ends_with("First") && production.artifact_id.ends_with("first.rs"))
        || (production.declaration_id.ends_with("Second") && production.artifact_id.ends_with("second.rs"))
    }));
    let governed: Vec<_> = build.corpus.edges.iter().filter(|edge| edge.relation.as_str() == "governs").collect();
    assert_eq!(governed.len(), 1); assert!(governed[0].to.ends_with("first.rs"));
    assert_eq!(governed[0].from, "contract:23.2");
    assert!(subject(&build, "declaration:fixture:First").sources.iter().any(|source| source.source_id == "src/declarations.rs" && source.role.as_str() == "definition"));
    assert!(build.unknowns.iter().any(|unknown| unknown.contains("fingerprint freshness")));
    assert!(build.omissions.is_empty()); build.corpus.validate().unwrap();
}
#[test]
fn cargo_dependencies_are_declarations_with_exact_bytes_and_explicit_activation_unknowns() {
    let fixture = Fixture::new();
    let manifest = "[workspace.dependencies]\nrename={package='actual',version='=1.2.3',features=['parse']}\n[dependencies]\nrename.workspace=true\n[target.'cfg(unix)'.build-dependencies]\nhelper='=2.0.0'\n";
    fixture.write("Cargo.toml", manifest);
    let cut = fixture.capture(&["Cargo.toml"]);
    fixture.write("Cargo.toml", "[dependencies]\nwrong='=99.0.0'\n");
    let build = build_corpus(&cut).unwrap();
    assert_eq!(build.corpus.subjects.len(), 3);
    assert!(build.corpus.subjects.iter().all(|subject| subject.kind.as_str() == "declaration"));
    assert!(build.corpus.subjects.iter().any(|subject| subject.native_id == "cargo:Cargo.toml#workspace.dependencies.rename" && subject.label.contains("actual")));
    assert!(build.corpus.subjects.iter().any(|subject| subject.native_id.contains("cfg(unix).build-dependencies.helper")));
    assert_eq!(build.corpus.sources.iter().find(|source| source.id == "Cargo.toml").unwrap().version_digest, digest(manifest.as_bytes()));
    assert!(build.unknowns.iter().any(|unknown| unknown.contains("actual API use") && unknown.contains("target activation")));
    assert!(build.corpus.productions.is_empty()); build.corpus.validate().unwrap();
}
#[test]
fn a_code_file_alone_does_not_fabricate_a_semantic_file_subject() {
    let fixture = Fixture::new(); fixture.write("src/unknown.rs", "fn looks_important() {}\n");
    let build = build_corpus(&fixture.capture(&["src/unknown.rs"])).unwrap();
    assert!(build.corpus.subjects.is_empty());
    assert!(build.unknowns.iter().any(|unknown| unknown.contains("no owner-native development concept")));
    assert!(build.corpus.validate().is_err());
}
#[test]
fn source_bytes_are_read_once_and_final_stamp_check_refuses_mutation() {
    let fixture = Fixture::new(); fixture.write("src/captured.rs", "original\n");
    let selection = fixture.selection(&["src/captured.rs"]);
    let capture = Capture { root: fixture.root(), selection: &selection, deadline: Instant::now() + Duration::from_secs(10),
        files: RefCell::new(BTreeMap::new()), stamps: RefCell::new(BTreeMap::new()), unavailable: RefCell::new(BTreeMap::new()) };
    assert_eq!(capture.read("src/captured.rs").unwrap(), b"original\n");
    fixture.write("src/captured.rs", "conflicting bytes\n");
    assert_eq!(capture.read("src/captured.rs").unwrap(), b"original\n");
    assert!(capture.verify(&Layout::default()).unwrap_err().to_string().contains("source cut changed"));
}
#[test]
fn admission_rejects_excluded_paths_symlinks_changed_hashes_and_bounds() {
    let fixture = Fixture::new(); fixture.write("Cargo.toml", "[dependencies]\nfirst='=1.0.0'\n");
    for path in ["../outside.rs", "/tmp/outside.rs", "external/code.rs", ".venv/code.py", "secrets/config.toml", ".env.toml", "build/result.json"] {
        assert!(bounded_source_cut(fixture.root(), &fixture.selection(&[path])).is_err(), "admitted {path}");
    }
    let mut selection = fixture.selection(&["Cargo.toml"]);
    selection.expected_digests.insert("Cargo.toml".into(), "0".repeat(64));
    assert!(bounded_source_cut(fixture.root(), &selection).unwrap_err().to_string().contains("digest mismatch"));
    selection.expected_digests.clear(); selection.bounds.max_source_bytes = 4;
    assert!(bounded_source_cut(fixture.root(), &selection).unwrap_err().to_string().contains("byte bound"));
    #[cfg(unix)] {
        std::os::unix::fs::symlink(fixture.root().join("Cargo.toml"), fixture.root().join("linked.toml")).unwrap();
        assert!(bounded_source_cut(fixture.root(), &fixture.selection(&["linked.toml"])).unwrap_err().to_string().contains("symbolic"));
    }
}
#[test]
fn exact_expected_contributors_and_excerpt_ranges_are_checked() {
    let fixture = Fixture::new(); fixture.architecture();
    let mut selection = fixture.selection(&["docs/authoritative_design/sections/boundary.md"]);
    selection.expected_digests.insert(POLICY.into(), digest(&fs::read(fixture.root().join(POLICY)).unwrap()));
    selection.ranges.insert("docs/authoritative_design/sections/boundary.md".into(), SourceLineRange { start_line: 8, end_line: 10 });
    let cut = bounded_source_cut(fixture.root(), &selection).unwrap();
    let build = build_corpus(&cut).unwrap();
    assert!(build.corpus.excerpts.iter().filter(|excerpt| excerpt.source_id.ends_with("boundary.md"))
        .all(|excerpt| !excerpt.text.contains("Only admitted values publish.")));
    selection.ranges.get_mut("docs/authoritative_design/sections/boundary.md").unwrap().end_line = 999;
    let cut = bounded_source_cut(fixture.root(), &selection).unwrap();
    assert!(build_corpus(&cut).unwrap_err().to_string().contains("invalid selected excerpt range"));
    selection.expected_digests.insert("src/not-selected.rs".into(), "a".repeat(64));
    assert!(bounded_source_cut(fixture.root(), &selection).unwrap_err().to_string().contains("uncaptured contributor"));
}
#[test]
fn duplicate_stable_sections_and_unknown_producer_direction_are_not_accepted() {
    let fixture = Fixture::new(); fixture.architecture();
    fixture.write("docs/authoritative_design/sections/duplicate.md", "## 23.2 Duplicate\n");
    assert!(bounded_source_cut(fixture.root(), &fixture.selection(&["docs/authoritative_design/sections/boundary.md"]))
        .unwrap_err().to_string().contains("duplicate section"));
    fs::remove_file(fixture.root().join("docs/authoritative_design/sections/duplicate.md")).unwrap();
    fixture.write("src/declaration.rs", "struct D;\n"); fixture.write("src/generator.rs", "fn emit() {}\n");
    fixture.write("docs/generated/out.rs", "// output\n");
    fixture.write(&format!("{PRODUCERS}/fixture/relationships.json"), &json!({"schema_version":1,"producer":"fixture","relationships":[{
        "declaration_key":"D","declaration_kind":"fixture","source_path":"src/declaration.rs","source_symbol":"D",
        "source_role":"type_definition","source_identity":null,"generator_path":"src/generator.rs","generator_symbol":"emit",
        "output_path":"docs/generated/out.rs","direction":"invented","basis":"declared"
    }]}).to_string());
    assert!(bounded_source_cut(fixture.root(), &fixture.selection(&["src/declaration.rs"])).unwrap_err().to_string().contains("unknown native vocabulary"));
}
#[test]
fn closure_rejects_changed_owner_index_membership_and_newly_present_policy() {
    let fixture = Fixture::new(); fixture.architecture();
    let selection = fixture.selection(&["docs/authoritative_design/sections/boundary.md"]);
    let deadline = Instant::now() + Duration::from_secs(10);
    let layout = discover(fixture.root(), &selection, deadline).unwrap();
    let capture = Capture { root: fixture.root(), selection: &selection, deadline,
        files: RefCell::new(BTreeMap::new()), stamps: RefCell::new(BTreeMap::new()), unavailable: RefCell::new(BTreeMap::new()) };
    fixture.write("docs/authoritative_design/sections/new.md", "## 99.1 New section\n");
    assert!(capture.verify(&layout).unwrap_err().to_string().contains("membership changed"));
    fs::remove_file(fixture.root().join(POLICY)).unwrap();
    let layout = discover(fixture.root(), &selection, deadline).unwrap();
    fixture.write(POLICY, "version=1\n");
    assert!(capture.verify(&layout).unwrap_err().to_string().contains("appeared before closure"));
}
