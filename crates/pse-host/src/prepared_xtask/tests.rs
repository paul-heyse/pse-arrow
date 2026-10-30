// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::{
    catalog::description::{self, Information},
    discovery::BuiltArtifact,
};
use serde_json::json;
use std::{
    collections::BTreeMap,
    os::unix::{ffi::OsStringExt as _, fs::PermissionsExt as _},
};

fn build(checkout: &Path, directory: &Path, executable: Executable) -> BuildCapture {
    let cargo:cargo_metadata::Artifact=serde_json::from_value(json!({"package_id":"xtask-id","manifest_path":checkout.join("xtask/Cargo.toml"),"target":{"name":"xtask","kind":["bin"],"crate_types":["bin"],"src_path":checkout.join("xtask/src/main.rs"),"edition":"2024"},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["force-validate"],"filenames":[directory.join("mutable-target")],"executable":directory.join("mutable-target"),"fresh":false})).unwrap();
    BuildCapture {
        version: 1,
        resolution: correctness::Resolution {
            arguments: vec!["build".into(), "--no-default-features".into()],
            packages: vec!["xtask".into()],
            units: vec![correctness::CompilerUnit {
                package_id: cargo.package_id.repr.clone(),
                package_name: "xtask".into(),
                target_name: "xtask".into(),
                target_kinds: vec!["bin".into()],
                source_path: cargo.target.src_path.to_string(),
                features: BTreeSet::from(["force-validate".into()]),
                platform: None,
                mode: "build".into(),
                root: true,
            }],
            native_capabilities: BTreeSet::new(),
        },
        artifacts: vec![BuiltArtifact { cargo, executable }],
        cargo_exit: 0,
        incomplete: BTreeMap::new(),
    }
}
fn metadata(root: &Path) -> cargo_metadata::Metadata {
    serde_json::from_value(json!({"version":1,"workspace_root":root,"target_directory":root.join("target"),"workspace_members":["xtask-id"],"workspace_default_members":["xtask-id"],"packages":[{"name":"xtask","version":"0.0.1","id":"xtask-id","manifest_path":root.join("xtask/Cargo.toml"),"dependencies":[],"targets":[],"features":{"force-validate":[]}}],"resolve":{"root":null,"nodes":[{"id":"xtask-id","dependencies":[],"deps":[],"features":["force-validate"]}]}})).unwrap()
}
fn fixture() -> (tempfile::TempDir, Environment, PreparedXtask) {
    let root = tempfile::tempdir().unwrap();
    assert!(
        Command::new("git")
            .args(["init", "--quiet"])
            .arg(root.path())
            .status()
            .unwrap()
            .success()
    );
    fs::create_dir(root.path().join("xtask")).unwrap();
    fs::create_dir(root.path().join("xtask/src")).unwrap();
    fs::create_dir(root.path().join(".cargo")).unwrap();
    for path in [
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        ".cargo/config.toml",
        "xtask/Cargo.toml",
        "xtask/src/main.rs",
    ] {
        fs::write(root.path().join(path), "independent source\n").unwrap();
    }
    let environment = Environment::from([
        ("HOME".into(), root.path().as_os_str().to_owned()),
        (
            "XDG_STATE_HOME".into(),
            root.path().join("state").into_os_string(),
        ),
        ("PATH".into(), "/path-that-does-not-exist".into()),
    ]);
    let state = CheckoutState::resolve(root.path(), &environment).unwrap();
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&state.root)
        .unwrap();
    private_directory(&state.root.join("tools"), true).unwrap();
    private_directory(&state.root.join("tools/xtask"), true).unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let directory = state.root.join("tools/xtask").join(&id);
    private_directory(&directory, true).unwrap();
    private_directory(&directory.join("build"), true).unwrap();
    let response = DocumentOwnership {
        path: root.path().join("docs/case.md").into_os_string(),
        doc_owner: Some("docs/README.md".into()),
        historical_owner: None,
        doc_role: Some("reference".into()),
        doc_retention: Some("while-current".into()),
        status: None,
        sections: vec![],
        workflow: crate::ownership::document::Workflow::DocumentOwner,
        coverage_limits: vec!["fixture response".into()],
    };
    fs::write(
        root.path().join("response.json"),
        serde_json::to_vec(&response).unwrap(),
    )
    .unwrap();
    fs::create_dir(root.path().join("docs")).unwrap();
    fs::write(root.path().join("docs/case.md"), "# Case\n").unwrap();
    let original = root.path().join("tool");
    fs::write(
        &original,
        b"#!/bin/sh\nif [ \"$1\" = owner ]; then /bin/cat response.json; else exit 9; fi\n",
    )
    .unwrap();
    fs::set_permissions(&original, fs::Permissions::from_mode(0o700)).unwrap();
    let executable = capture::executable(&original, &directory.join("build/artifact-0")).unwrap();
    let control = capture::executable(&original, &directory.join("cargo-pse-correctness")).unwrap();
    let build = build(root.path(), &directory, executable.clone());
    let bytes = serde_json::to_vec(&build).unwrap();
    publish(&directory.join("build/build.json"), &bytes).unwrap();
    let command = clap::Command::new("cargo xtask")
        .subcommand(
            clap::Command::new("owner").arg(clap::Arg::new("path")).arg(
                clap::Arg::new("schema")
                    .long("schema")
                    .action(clap::ArgAction::SetTrue),
            ),
        )
        .subcommand(clap::Command::new("commands"));
    let mut page = description::commands(command, &[], true, &description::Window::default());
    page.source.owner = "compiled xtask command parser".into();
    let catalog = serde_json::to_vec(&page).unwrap();
    publish(&directory.join("commands.json"), &catalog).unwrap();
    let schema = serde_json::to_vec(&schemars::schema_for!(DocumentOwnership)).unwrap();
    publish(&directory.join("owner-schema.json"), &schema).unwrap();
    let context = BuildContext::new(
        "observed rustc".into(),
        "observed cargo".into(),
        "dev".into(),
        BTreeSet::from(["xtask/force-validate".into()]),
        &environment,
    );
    let source = SourceFingerprint::from_resolution(
        root.path(),
        &metadata(root.path()),
        context.clone(),
        &build.resolution,
    )
    .unwrap();
    let selected = PreparedXtask {
        checkout: root.path().into(),
        directory,
        manifest: Manifest {
            version: 1,
            checkout_id: state.checkout_id.as_str().into(),
            selection_id: id,
            executable,
            control,
            source,
            context,
            features: BTreeSet::from(["force-validate".into()]),
            build: Evidence::new("build/build.json", &bytes),
            catalog: Evidence::new("commands.json", &catalog),
            owner_schema: Evidence::new("owner-schema.json", &schema),
        },
    };
    publish(
        &selected.directory.join("xtask.json"),
        &serde_json::to_vec(&selected.manifest).unwrap(),
    )
    .unwrap();
    (root, environment, selected)
}

#[test]
fn passive_open_authenticates_source_and_evidence_without_a_compiler_or_build() {
    let (root, environment, original) = fixture();
    let selected = PreparedXtask::open(root.path(), &environment, original.id()).unwrap();
    assert_eq!(selected.id(), original.id());
    assert_eq!(
        selected.features(),
        &BTreeSet::from(["force-validate".into()])
    );
    assert_eq!(
        selected.source_identity().unwrap(),
        original.source_identity().unwrap()
    );
    assert!(!root.path().join("target").exists());
    let catalog = selected.catalog().unwrap();
    assert!(
        catalog
            .records
            .iter()
            .all(|record| matches!(record.output_schemas, Information::Unavailable { .. }))
    );
    assert!(selected.owner_schema().unwrap()["properties"]["workflow"].is_object());
    let arguments = vec![
        OsString::from_vec(b"opaque-\xff".to_vec()),
        "--literal".into(),
    ];
    let supplied =
        Environment::from([("EXACT".into(), OsString::from_vec(b"bytes-\xff".to_vec()))]);
    let launch = selected
        .launch(arguments.clone(), supplied.clone())
        .unwrap();
    assert_eq!(launch.program, selected.executable().captured);
    assert_eq!(launch.arguments, arguments);
    assert_eq!(launch.environment, supplied);
    assert_eq!(launch.cwd, root.path());
    let document = selected
        .document_owner(Path::new("docs/case.md"), &environment)
        .unwrap();
    assert_eq!(document.doc_owner.as_deref(), Some("docs/README.md"));
    fs::write(
        root.path().join("xtask/src/main.rs"),
        "changed after capture",
    )
    .unwrap();
    let retained = PreparedXtask::open(root.path(), &environment, original.id()).unwrap();
    let metadata = retained.catalog().unwrap();
    assert_eq!(
        metadata.source.freshness,
        description::Freshness::RetainedSnapshot
    );
    assert_eq!(metadata.source.generation.as_deref(), Some(original.id()));
    assert!(retained.owner_schema().unwrap()["properties"].is_object());
    assert!(selected.launch(vec![], Environment::new()).is_err());
    assert!(
        retained
            .document_owner(Path::new("docs/case.md"), &environment)
            .is_err()
    );
}
#[test]
fn unprepared_open_creates_no_state_and_invalid_keys_never_become_paths() {
    let scratch = tempfile::tempdir().unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let environment = Environment::from([
        ("HOME".into(), scratch.path().as_os_str().to_owned()),
        (
            "XDG_STATE_HOME".into(),
            scratch.path().join("state").into_os_string(),
        ),
        ("PATH".into(), "/missing".into()),
    ]);
    assert!(PreparedXtask::open(root, &environment, &uuid::Uuid::new_v4().to_string()).is_err());
    for id in ["../arbitrary", "/absolute", "not-a-uuid"] {
        assert!(PreparedXtask::open(root, &environment, id).is_err());
    }
    assert_eq!(fs::read_dir(scratch.path()).unwrap().count(), 0);
}
#[test]
fn original_manifest_evidence_and_executable_cannot_redirect_or_silently_change() {
    for name in [
        "commands.json",
        "owner-schema.json",
        "build/build.json",
        "build/artifact-0",
        "cargo-pse-correctness",
    ] {
        let (root, environment, selected) = fixture();
        fs::set_permissions(
            selected.directory.join(name),
            fs::Permissions::from_mode(0o600),
        )
        .unwrap();
        fs::write(selected.directory.join(name), b"changed original evidence").unwrap();
        assert!(
            PreparedXtask::open(root.path(), &environment, selected.id()).is_err(),
            "{name}"
        );
    }
    let (root, environment, mut selected) = fixture();
    selected.manifest.catalog.name = "../response.json".into();
    assert!(selected.verify().is_err());
    selected.manifest.catalog.name = "commands.json".into();
    selected.manifest.control.captured = root.path().join("tool");
    assert!(selected.verify().is_err());
    let other = tempfile::tempdir().unwrap();
    assert!(
        Command::new("git")
            .args(["init", "--quiet"])
            .arg(other.path())
            .status()
            .unwrap()
            .success()
    );
    assert!(PreparedXtask::open(other.path(), &environment, selected.id()).is_err());
}
#[test]
fn zero_exit_is_not_artifact_capture_and_unit_source_feature_identity_must_match() {
    let (root, _, selected) = fixture();
    let bytes = read(&selected.directory.join("build/build.json"), MAX_DOCUMENT).unwrap();
    let mut build: BuildCapture = serde_json::from_slice(&bytes).unwrap();
    select_executable(&build, &selected.directory, root.path()).unwrap();
    let admitted = build.resolution.clone();
    require_same_compiler_context(&admitted, &build.resolution).unwrap();
    let mut changed_context = admitted.clone();
    changed_context.units[0].platform = Some("different-target".into());
    assert!(require_same_compiler_context(&admitted, &changed_context).is_err());
    changed_context = admitted.clone();
    changed_context.units[0]
        .features
        .insert("newly-activated-feature".into());
    assert!(require_same_compiler_context(&admitted, &changed_context).is_err());
    build.resolution.units[0].package_id = "different-source".into();
    assert!(select_executable(&build, &selected.directory, root.path()).is_err());
    build.resolution.units[0].package_id = "xtask-id".into();
    build.artifacts[0]
        .cargo
        .features
        .push("native-solvers".into());
    assert!(select_executable(&build, &selected.directory, root.path()).is_err());
    build.artifacts.clear();
    assert!(select_executable(&build, &selected.directory, root.path()).is_err());
    let missing = build_evidence(&selected.directory.join("missing")).unwrap_err();
    assert!(
        matches!(missing,PreparedXtaskError::BuildEvidence{source,..} if matches!(*source,PreparedXtaskError::Io(_)))
    );
    let command =
        source_build_command(&selected.manifest.control, root.path(), &Environment::new());
    let environment: BTreeMap<_, _> = command
        .get_envs()
        .map(|(name, value)| (name.to_owned(), value.map(OsStr::to_owned)))
        .collect();
    assert_eq!(
        environment.get(OsStr::new("PSE_CORRECTNESS_ADAPTER")),
        Some(&Some("1".into()))
    );
    assert_eq!(
        environment.get(OsStr::new("PSE_CORRECTNESS_EXECUTABLE")),
        Some(&Some(
            selected.manifest.control.captured.as_os_str().to_owned()
        ))
    );
}
#[test]
fn native_child_refusal_and_timeout_keep_original_streams_after_settlement() {
    let mut command = Command::new("/bin/sh");
    command
        .args([
            "-c",
            "printf original-out; printf original-error >&2; exit 7",
        ])
        .env_clear();
    let output = process::run(command, 1024, Some(Duration::from_secs(2))).unwrap();
    match successful(output, "fixture-read", true).unwrap_err() {
        PreparedXtaskError::Command {
            status,
            stdout,
            stderr,
            ..
        } => {
            assert_eq!(status, 7);
            assert_eq!(stdout, b"original-out");
            assert_eq!(stderr, b"original-error");
        }
        error => panic!("{error}"),
    }
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "printf before-timeout >&2; sleep 60"])
        .env_clear();
    let error = process::run(command, 1024, Some(Duration::from_millis(100)))
        .err()
        .unwrap();
    match &error {
        PreparedXtaskError::Execution {
            failures, stderr, ..
        } => {
            assert!(failures.iter().any(|failure| matches!(
                failure,
                PreparedXtaskError::Supervision(pse_diagnostics::SupervisionError::Deadline)
            )));
            assert_eq!(stderr, b"before-timeout");
            assert_eq!(
                pse_diagnostics::TypedDiagnostic::diagnostic_children(&error)
                    .unwrap()
                    .count(),
                failures.len()
            );
        }
        error => panic!("{error}"),
    }
    let mut command = Command::new("/bin/sh");
    command
        .args([
            "-c",
            "printf original-complete-frame; printf original-error >&2",
        ])
        .env_clear();
    let output = process::run(command, 3, Some(Duration::from_secs(2))).unwrap();
    match successful(output, "fixture-read", true).unwrap_err() {
        PreparedXtaskError::Extent {
            bytes,
            stdout,
            stderr,
            ..
        } => {
            assert_eq!(bytes, 23);
            assert_eq!(stdout, b"ori");
            assert_eq!(stderr, b"original-error");
        }
        error => panic!("{error}"),
    }
}

#[test]
fn actual_captured_adapter_build_records_the_same_xtask_artifact() {
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("xtask/src")).unwrap();
    fs::write(
        root.path().join("Cargo.toml"),
        "[workspace]\nmembers=['xtask']\nresolver='3'\n",
    )
    .unwrap();
    fs::write(
        root.path().join("xtask/Cargo.toml"),
        "[package]\nname='xtask'\nversion='0.0.1'\nedition='2024'\n[features]\nforce-validate=[]\n",
    )
    .unwrap();
    fs::write(root.path().join("xtask/src/main.rs"), "fn main() {}\n").unwrap();
    fs::copy(
        owner.join("rust-toolchain.toml"),
        root.path().join("rust-toolchain.toml"),
    )
    .unwrap();
    let mut environment =
        correctness::standalone_adapter_environment(&std::env::vars_os().collect()).unwrap();
    let cargo = correctness::real_cargo_in(&environment).unwrap();
    let mut lock = Command::new(cargo);
    lock.args(["generate-lockfile", "--offline"])
        .current_dir(root.path())
        .env_clear()
        .envs(&environment);
    successful(
        process::run(lock, 65536, Some(READ_TIMEOUT)).unwrap(),
        "fixture-lockfile",
        true,
    )
    .unwrap();
    let directory = root.path().join("captured");
    private_directory(&directory, true).unwrap();
    let build_dir = directory.join("build");
    private_directory(&build_dir, true).unwrap();
    let generation = crate::generation::prepared(owner).unwrap().unwrap();
    let source_control = generation
        .executable(owner, crate::generation::HostTool::Correctness)
        .unwrap();
    let control =
        capture::executable(&source_control, &directory.join("cargo-pse-correctness")).unwrap();
    environment.insert(
        "PSE_CORRECTNESS_BUILD_CAPTURE".into(),
        build_dir.clone().into_os_string(),
    );
    successful(
        process::run(
            source_build_command(&control, root.path(), &environment),
            65536,
            None,
        )
        .unwrap(),
        "fixture-build",
        false,
    )
    .unwrap();
    let (_, build) = build_evidence(&build_dir).unwrap();
    let artifact = select_executable(&build, &directory, root.path()).unwrap();
    artifact.verify().unwrap();
    let original = artifact.original.clone();
    let captured = read(&artifact.captured, MAX_DOCUMENT).unwrap();
    fs::write(
        original,
        b"changed mutable Cargo output after native capture",
    )
    .unwrap();
    artifact.verify().unwrap();
    assert_eq!(read(&artifact.captured, MAX_DOCUMENT).unwrap(), captured);
}

#[test]
fn actual_prepared_xtask_exports_same_image_parser_schema_and_document_owner() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let state = tempfile::tempdir().unwrap();
    let mut environment: Environment = std::env::vars_os().collect();
    environment.insert(
        "XDG_STATE_HOME".into(),
        state.path().join("state").into_os_string(),
    );
    let selected = prepare(root, &environment).unwrap();
    let footprint = serde_json::to_value(&selected.manifest.source).unwrap();
    let roots = footprint["package_roots"].as_array().unwrap();
    assert!(!roots.contains(&json!("crates/pse-runtime")));
    assert!(!roots.contains(&json!("crates/pse-host")));
    let opened = PreparedXtask::open(root, &environment, selected.id()).unwrap();
    let catalog = opened.catalog().unwrap();
    assert_eq!(
        catalog.source.freshness,
        description::Freshness::RetainedSnapshot
    );
    assert!(catalog.records.iter().any(|record| record.path == ["docs"]));
    assert!(
        catalog
            .records
            .iter()
            .any(|record| record.path == ["owner"])
    );
    assert_eq!(
        opened.owner_schema().unwrap(),
        serde_json::to_value(schemars::schema_for!(DocumentOwnership)).unwrap()
    );
    let document = opened
        .document_owner(
            Path::new("docs/authoritative_design/sections/operations-and-validation.md"),
            &environment,
        )
        .unwrap();
    assert_eq!(
        document.workflow,
        crate::ownership::document::Workflow::ArchitectureDecision
    );
    assert!(document.sections.iter().any(|section| section.id == "24.2"));
    match opened
        .document_owner(Path::new("../../outside.md"), &environment)
        .unwrap_err()
    {
        PreparedXtaskError::Command { status, stderr, .. } => {
            assert_eq!(status, 1);
            assert!(!stderr.is_empty());
        }
        error => panic!("{error}"),
    }
}
