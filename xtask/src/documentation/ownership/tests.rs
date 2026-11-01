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

#[test]
fn associations_use_native_defaults_scope_bindings_and_keep_missing_citations_unknown() {
    let root = fixture();
    fs::create_dir_all(root.path().join("docs/plans")).unwrap();
    let path = Path::new("docs/plans/selected.md");
    fs::write(root.path().join(path), "---\nstatus: active\n---\n# Work\n\nSee blueprint §2 and [missing](absent.md).\n\n## Packages\n\n| Package | Status |\n| --- | --- |\n| E02 | planned |\n").unwrap();
    fs::write(
        root.path()
            .join("docs/authoritative_design/sections/model.md"),
        "## 2. Selected model\n",
    )
    .unwrap();
    let policy_path = root.path().join("docs/lifecycle.toml");
    let mut policy = fs::read_to_string(&policy_path).unwrap();
    policy.push_str("\n[status_profiles.work]\norder=['open','complete']\nopen=['planned']\ncomplete=['complete']\n[[scope_tables]]\npath='docs/plans/selected.md'\nname='packages'\nheading='Packages'\nheaders=['Package','Status']\nkind='package'\nid_column=0\nstatus_column=1\nstatus_profile='work'\n");
    fs::write(policy_path, policy).unwrap();
    let result = read_associations(root.path(), path, None, false, None, &[]).unwrap();
    assert_eq!(
        result.document.as_ref().unwrap().doc_owner.as_deref(),
        Some("docs/README.md")
    );
    assert_eq!(result.metadata_origins["doc_owner"], "global default");
    assert!(
        result
            .scope_items
            .iter()
            .any(|item| item["native_id"] == "E02"
                && item["raw_status"] == "planned"
                && item["state"] == "open")
    );
    assert!(
        result
            .associations
            .iter()
            .any(|item| item.written == "absent.md"
                && item.resolved.is_none()
                && item.limitation.is_some())
    );
    assert!(
        result
            .associations
            .iter()
            .any(|item| item.written == "blueprint §2"
                && item.resolved.as_deref()
                    == Some("docs/authoritative_design/sections/model.md#section-2"))
    );
    assert!(
        result
            .inputs
            .iter()
            .any(|input| input.path == "docs/lifecycle.toml")
    );
}

#[test]
fn historical_associations_read_original_policy_membership_and_section_owner() {
    use std::process::Command;
    let root = fixture();
    let run = |arguments: &[&str]| {
        let output = Command::new("git")
            .arg("-C")
            .arg(root.path())
            .args(arguments)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    };
    fs::write(
        root.path().join("docs/selected.md"),
        "# Selected\n\nSee blueprint §2.\n",
    )
    .unwrap();
    fs::write(
        root.path()
            .join("docs/authoritative_design/sections/original.md"),
        "## 2. Original section\n",
    )
    .unwrap();
    run(&["init", "--quiet"]);
    run(&["add", "."]);
    run(&[
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "commit",
        "--quiet",
        "-m",
        "original source",
    ]);
    let revision = run(&["rev-parse", "HEAD"]);
    fs::remove_file(
        root.path()
            .join("docs/authoritative_design/sections/original.md"),
    )
    .unwrap();
    fs::write(
        root.path()
            .join("docs/authoritative_design/sections/moved.md"),
        "## 2. Current moved section\n",
    )
    .unwrap();
    fs::write(root.path().join("docs/current.md"), "# Current owner\n").unwrap();
    let policy_path = root.path().join("docs/lifecycle.toml");
    let original_policy = fs::read_to_string(&policy_path).unwrap();
    fs::write(
        &policy_path,
        original_policy.replace("docs/README.md", "docs/current.md"),
    )
    .unwrap();
    let historical = read_associations(
        root.path(),
        Path::new("docs/selected.md"),
        Some(&revision),
        false,
        None,
        &[],
    )
    .unwrap();
    assert_eq!(historical.context, revision);
    assert_eq!(
        historical.document.unwrap().doc_owner.as_deref(),
        Some("docs/README.md")
    );
    assert!(
        historical
            .associations
            .iter()
            .any(|item| item.resolved.as_deref()
                == Some("docs/authoritative_design/sections/original.md#section-2"))
    );
    let current = read_associations(
        root.path(),
        Path::new("docs/selected.md"),
        None,
        false,
        None,
        &[],
    )
    .unwrap();
    assert_eq!(
        current.document.unwrap().doc_owner.as_deref(),
        Some("docs/current.md")
    );
    assert!(
        current
            .associations
            .iter()
            .any(|item| item.resolved.as_deref()
                == Some("docs/authoritative_design/sections/moved.md#section-2"))
    );
    // A selected universe missing policy stays unknown even when a current policy exists.
    run(&["rm", "--cached", "docs/lifecycle.toml"]);
    run(&[
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "commit",
        "--quiet",
        "-m",
        "missing original policy",
    ]);
    let missing_revision = run(&["rev-parse", "HEAD"]);
    let missing = read_associations(
        root.path(),
        Path::new("docs/selected.md"),
        Some(&missing_revision),
        false,
        None,
        &[],
    )
    .unwrap();
    assert!(missing.document.is_none());
    assert!(
        missing
            .limitations
            .iter()
            .any(|reason| reason.contains("current defaults are not substituted"))
    );
}
