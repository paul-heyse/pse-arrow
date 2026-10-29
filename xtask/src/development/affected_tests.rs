// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-only selection, conservative reach and execution refusal controls.
use super::*;

fn workspace(root: &Path) -> Result<Workspace> {
    let dependency = |name: &str, kind: &str, optional: bool, rename: Option<&str>| json!({"name":name,"source":null,"req":"*","kind":kind,"optional":optional,"uses_default_features":true,"features":[],"target":"cfg(unix)","rename":rename,"registry":null});
    let package = |name: &str, directory: &str, dependencies: Vec<Value>| json!({"name":name,"version":"1.0.0","id":name,"source":null,"manifest_path":root.join(directory).join("Cargo.toml"),"dependencies":dependencies,"targets":[],"features":{}});
    let metadata: Metadata = serde_json::from_value(json!({
        "version":1,"workspace_root":root,"target_directory":root.join("target"),"resolve":null,
        "workspace_members":["base","consumer","downstream","unrelated","nested"],
        "packages":[
            package("base","crates/base",vec![]),
            package("consumer","crates/consumer",vec![dependency("base","build",true,Some("renamed"))]),
            package("downstream","tests/downstream",vec![dependency("consumer","dev",false,None)]),
            package("unrelated","crates/unrelated",vec![]),
            package("nested","crates/base/nested",vec![]),
            package("external","vendor/external",vec![dependency("base","normal",false,None)])
        ]
    }))?;
    Workspace::from_metadata(root, &metadata)
}

fn options(changes: &[(&str, &str)]) -> Options {
    Options { changes: Some(changes.iter().map(|(status,path)| ((*status).into(),(*path).into())).collect()), ..Options::default() }
}

#[test]
fn dependency_preview_includes_renamed_optional_build_dev_and_transitive_consumers() -> Result<()> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path();
    let options = Options { features: "consumer/native-solvers".into(), extra: vec!["--no-fail-fast".into(), "--".into(), "literal test".into()], ..options(&[("D","crates/base/src/removed.rs")]) };
    let selection = assemble(root, &options, options.changes.clone().context("changes")?, Ok(workspace(root)?))?;
    assert_eq!(selection.changed_packages, ["base"]);
    assert_eq!(selection.selected_packages, ["base", "consumer", "downstream"]);
    assert_eq!(selection.graph_status, "available");
    assert_eq!(selection.command, Some(vec!["scripts/pse-env", "--", "scripts/cargo-bin/cargo-pse-correctness", "nextest", "run", "--locked", "--features", "consumer/native-solvers", "-p", "base", "-p", "consumer", "-p", "downstream", "-E", "rdeps(=base)", "--no-fail-fast", "--", "literal test"].into_iter().map(str::to_owned).collect()));
    assert!(!root.join("target").exists());
    assert!(!root.join("build").exists());
    Ok(())
}

#[test]
fn ownership_uses_component_boundaries_and_the_nearest_member() -> Result<()> {
    let fixture = tempfile::tempdir()?;
    let workspace = workspace(fixture.path())?;
    assert_eq!(workspace.owner("crates/base/src/lib.rs"), Some("base"));
    assert_eq!(workspace.owner("crates/base/nested/src/lib.rs"), Some("nested"));
    assert_eq!(workspace.owner("crates/base-other/src/lib.rs"), None);
    assert_eq!(workspace.owner("vendor/external/src/lib.rs"), None);
    Ok(())
}

#[test]
fn root_configuration_widens_to_real_members_and_keeps_other_exclusions() -> Result<()> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path();
    let options = options(&[("M",".cargo/config.toml"),("M","python/reference.py"),("M","crates/pse-codegen/src/lib.rs"),("?","scripts/custom-tool")]);
    let selection = assemble(root, &options, options.changes.clone().context("changes")?, Ok(workspace(root)?))?;
    assert_eq!(selection.workspace_widening, [".cargo/config.toml"]);
    assert_eq!(selection.selected_packages, ["base","consumer","downstream","nested","unrelated"]);
    let command = selection.command.context("workspace command")?;
    assert!(command.iter().any(|argument| argument == "--workspace"));
    assert!(!command.iter().any(|argument| argument == "-E"));
    assert!(selection.exclusions.iter().any(|item| item["kind"] == "python" && item["files"] == json!(["python/reference.py"])));
    assert!(selection.exclusions.iter().any(|item| item["kind"] == "generator" && item["files"] == json!(["crates/pse-codegen/src/lib.rs"])));
    assert!(selection.exclusions.iter().any(|item| item["kind"] == "unowned" && item["files"].as_array().is_some_and(|files| files.contains(&json!("? scripts/custom-tool")))));
    Ok(())
}

#[test]
fn empty_preview_needs_no_manifest_cargo_git_or_executable() -> Result<()> {
    let fixture = tempfile::tempdir()?;
    let selection = preview(fixture.path(), &options(&[]))?;
    assert_eq!(selection.graph_status, "not-needed");
    assert!(selection.command.is_none());
    assert!(selection.selected_packages.is_empty());
    assert!(run(fixture.path(), &selection)?.is_none());
    assert_eq!(std::fs::read_dir(fixture.path())?.count(), 0);
    Ok(())
}

#[test]
fn unavailable_graph_is_reported_and_execution_refuses_before_launch() -> Result<()> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path();
    let options = options(&[("M","crates/base/src/lib.rs")]);
    let selection = assemble(root, &options, options.changes.clone().context("changes")?, Err(anyhow::anyhow!("metadata intentionally unavailable")))?;
    assert_eq!(selection.graph_status, "unavailable");
    assert!(selection.graph_error.as_ref().is_some_and(|error| error.contains("intentionally unavailable")));
    assert!(selection.command.is_none());
    assert!(run(root, &selection).unwrap_err().to_string().contains("graph is unavailable"));
    assert_eq!(std::fs::read_dir(root)?.count(), 0);
    Ok(())
}

#[test]
fn scientific_sources_conservatively_select_declared_consumers_without_capture() -> Result<()> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path();
    std::fs::create_dir_all(root.join("tests/fixtures"))?;
    std::fs::write(root.join(CATALOG), "version = 1\npackage_inventory = 'packages/reference'\n[[families]]\nname='example'\nconsumers=['consumer']\nroots=['not-read']\n")?;
    let options = options(&[("D","packages/reference/removed/package.toml")]);
    let selection = assemble(root, &options, options.changes.clone().context("changes")?, Ok(workspace(root)?))?;
    assert_eq!(selection.changed_packages, ["consumer"]);
    assert_eq!(selection.selected_packages, ["consumer","downstream"]);
    assert_eq!(selection.scientific_inputs["unknown"], json!(["packages/reference/removed/package.toml"]));
    assert_eq!(selection.scientific_inputs["families"], json!([]));
    assert!(!root.join("not-read").exists());
    assert!(!root.join("build").exists());
    assert!(!root.join("target").exists());
    std::fs::write(root.join(CATALOG), "not valid TOML!")?;
    let selection = assemble(root, &options, options.changes.clone().context("changes")?, Ok(workspace(root)?))?;
    assert_eq!(selection.selected_packages, ["base","consumer","downstream","nested","unrelated"]);
    assert!(selection.scientific_inputs.get("reason").is_some());
    Ok(())
}

#[test]
fn git_name_status_preserves_tabs_newlines_deletion_and_untracked_paths() -> Result<()> {
    assert_eq!(parse_diff(b"M\0crates/base/src/a\tb.rs\0D\0crates/base/src/c\nd.rs\0")?, vec![("M".into(),"crates/base/src/a\tb.rs".into()),("D".into(),"crates/base/src/c\nd.rs".into())]);
    assert!(parse_diff(b"M\0incomplete\0D\0").is_err());
    assert!(parse_diff(b"M\0missing terminator").is_err());
    assert_eq!(nul_fields(b"new\tfile\0new\nfile\0")?, ["new\tfile","new\nfile"]);
    Ok(())
}

#[test]
fn preview_rejects_escaping_changes_before_any_external_observation() -> Result<()> {
    let fixture = tempfile::tempdir()?;
    for path in ["../outside.rs", "crates/../../outside.rs", "/absolute.rs", "", "embedded\0nul"] {
        let error = preview(fixture.path(), &options(&[("M",path)])).unwrap_err();
        assert!(error.to_string().contains("inside the selected checkout"), "{path:?}: {error}");
    }
    assert_eq!(std::fs::read_dir(fixture.path())?.count(), 0);
    Ok(())
}
