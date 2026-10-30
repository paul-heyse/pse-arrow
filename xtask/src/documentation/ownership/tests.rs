// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use clap::Parser;
#[derive(Debug, Parser)]
struct Cli {
    #[command(flatten)]
    query: Query,
}

#[test]
fn native_owner_grammar_requires_exact_path_or_schema() {
    assert!(Cli::try_parse_from(["owner"]).is_err());
    assert!(Cli::try_parse_from(["owner", "docs/README.md", "--schema"]).is_err());
    assert!(
        Cli::try_parse_from(["owner", "--schema"])
            .unwrap()
            .query
            .schema
    );
    assert_eq!(
        Cli::try_parse_from(["owner", "docs/README.md"])
            .unwrap()
            .query
            .path,
        Some("docs/README.md".into())
    );
    assert!(serde_json::to_value(schemars::schema_for!(DocumentOwnership)).unwrap()["properties"]["path"].is_object());
}
fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("docs/authoritative_design/sections")).unwrap();
    fs::create_dir_all(root.path().join("docs/adr")).unwrap();
    fs::write(root.path().join("docs/README.md"), "# Owner\n").unwrap();
    fs::write(
        root.path().join("docs/site.toml"),
        "[publication]\n[[collections]]\ntitle='Docs'\nroot='.'\nscope='Reference'\n",
    )
    .unwrap();
    fs::write(root.path().join("docs/lifecycle.toml"), "version=1\n[defaults]\ndoc_role='reference'\ndoc_owner='docs/README.md'\ndoc_provenance='authored'\ndoc_retention='while-current'\ndoc_topics=[]\ndoc_retirement_trigger='owner-release'\n").unwrap();
    fs::write(
        root.path().join("docs/authoritative_design/blueprint.md"),
        "# 1. Overview\n",
    )
    .unwrap();
    root
}
#[test]
fn native_interpretation_preserves_owner_status_and_immutable_workflow() {
    let root = fixture();
    let file = root.path().join("docs/adr/0001-case.md");
    fs::write(
        &file,
        "---\ndoc_role: decision\nstatus: accepted\n---\n# Case\n",
    )
    .unwrap();
    let owner = read(root.path(), Path::new("docs/adr/0001-case.md")).unwrap();
    assert_eq!(owner.doc_owner.as_deref(), Some("docs/README.md"));
    assert_eq!(owner.status.as_deref(), Some("accepted"));
    assert_eq!(owner.workflow, Workflow::SupersedeDecision);
    fs::write(
        root.path().join("docs/adr/README.md"),
        "---\ndoc_role: decision\n---\n# Index\n",
    )
    .unwrap();
    assert_eq!(
        read(root.path(), Path::new("docs/adr/README.md"))
            .unwrap()
            .workflow,
        Workflow::DocumentOwner
    );
    fs::write(
        &file,
        "---\ndoc_role: decision\nstatus: proposed\n---\n# Case\n",
    )
    .unwrap();
    assert_eq!(
        read(root.path(), &file).unwrap().workflow,
        Workflow::ProposedDecision
    );
    fs::write(&file, "---\nstatus: accepted\nstatus: proposed\n---\n").unwrap();
    assert!(read(root.path(), &file).is_err());
}
#[test]
fn section_ids_come_from_native_headings_and_selected_source() {
    let root = fixture();
    let relative = Path::new("docs/authoritative_design/sections/model.md");
    fs::write(
        root.path().join(relative),
        "## 2. Actual model\n\n```md\n## 3. Not an owner\n```\n",
    )
    .unwrap();
    let owner = read(root.path(), relative).unwrap();
    assert_eq!(owner.workflow, Workflow::ArchitectureDecision);
    assert_eq!(owner.sections.len(), 1);
    assert_eq!(owner.sections[0].id, "2");
    assert!(owner.sections[0].heading.contains("Actual model"));
    assert!(read(root.path(), Path::new("../outside.md")).is_err());
    assert!(read(root.path(), Path::new("Cargo.toml")).is_err());
}

#[test]
fn native_owner_export_exercises_current_lifecycle_and_section_sources() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let owner = read(
        root,
        Path::new("docs/authoritative_design/sections/operations-and-validation.md"),
    )
    .unwrap();
    assert_eq!(owner.workflow, Workflow::ArchitectureDecision);
    assert!(owner.sections.iter().any(|section| section.id == "24.2"));
    assert!(owner.doc_owner.is_some());
    let owner = read(
        root,
        Path::new("docs/adr/0002-project-name-license-and-distribution.md"),
    )
    .unwrap();
    assert_eq!(owner.status.as_deref(), Some("accepted"));
    assert_eq!(owner.workflow, Workflow::SupersedeDecision);
    assert!(
        read(root, Path::new("docs/adr/README.md"))
            .unwrap()
            .workflow
            == Workflow::DocumentOwner
    );
}
