// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Ordinary Cargo freshness inputs, not an arbitrary executable-I/O proof.
//! Reuses the producer's actual unit/artifact/dep-info association. Opaque macro
//! and build-script package inputs deliberately retain conservative membership.

use super::{
    UnitGraph, absolute, artifact_dep_info, artifact_matches, parse_cargo_messages, parse_dep_info,
    selected_indices,
};
use anyhow::{Context, Result, ensure};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

fn build_script_directives(out_dir: &Path) -> Result<(Vec<PathBuf>, String)> {
    let paths = super::build_script_stdout_paths(out_dir)?
        .into_iter()
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    ensure!(
        !paths.is_empty(),
        "actual Cargo build-script directives unavailable for {}",
        out_dir.display()
    );
    let text = fs::read_to_string(&paths[0]).with_context(|| {
        format!(
            "actual Cargo build-script directives {}",
            paths[0].display()
        )
    })?;
    for path in paths.iter().skip(1) {
        ensure!(
            fs::read_to_string(path)? == text,
            "conflicting actual Cargo build-script directive locations for {}",
            out_dir.display()
        );
    }
    Ok((paths, text))
}

fn cargo_package_environment(name: &str) -> bool {
    matches!(
        name,
        "CARGO"
            | "CARGO_MANIFEST_DIR"
            | "CARGO_MANIFEST_PATH"
            | "CARGO_PKG_VERSION"
            | "CARGO_PKG_VERSION_MAJOR"
            | "CARGO_PKG_VERSION_MINOR"
            | "CARGO_PKG_VERSION_PATCH"
            | "CARGO_PKG_VERSION_PRE"
            | "CARGO_PKG_AUTHORS"
            | "CARGO_PKG_NAME"
            | "CARGO_PKG_DESCRIPTION"
            | "CARGO_PKG_HOMEPAGE"
            | "CARGO_PKG_REPOSITORY"
            | "CARGO_PKG_LICENSE"
            | "CARGO_PKG_LICENSE_FILE"
            | "CARGO_PKG_RUST_VERSION"
            | "CARGO_PKG_README"
    )
}

/// Cargo's crate-compilation environment, distinct from caller configuration.
/// Build-script execution-only names are deliberately not classified here.
fn cargo_crate_environment(name: &str, unit: &super::Unit, graph: &UnitGraph) -> bool {
    cargo_package_environment(name)
        || name == "CARGO_CRATE_NAME"
        || name == "CARGO_BIN_NAME" && unit.target.kind.iter().any(|kind| kind == "bin")
        || name == "CARGO_PRIMARY_PACKAGE"
            && graph
                .roots
                .iter()
                .any(|index| graph.units[*index].pkg_id == unit.pkg_id)
        || name == "OUT_DIR"
            && !unit.target.kind.iter().any(|kind| kind == "custom-build")
            && graph.units.iter().any(|candidate| {
                candidate.pkg_id == unit.pkg_id && candidate.mode == "run-custom-build"
            })
}

/// Values supplied to a running build script are derived from its unit/profile,
/// target configuration and package. Unknown or inactive-feature names remain
/// ordinary caller inputs if a script explicitly declares consuming them.
fn cargo_build_script_environment(name: &str, unit: &super::Unit) -> bool {
    cargo_package_environment(name)
        || matches!(
            name,
            "CARGO_MANIFEST_LINKS"
                | "CARGO_MAKEFLAGS"
                | "OUT_DIR"
                | "TARGET"
                | "HOST"
                | "NUM_JOBS"
                | "DEBUG"
                | "OPT_LEVEL"
                | "PROFILE"
                | "RUSTC"
                | "RUSTDOC"
                | "RUSTC_WRAPPER"
                | "RUSTC_WORKSPACE_WRAPPER"
                | "RUSTC_LINKER"
                | "CARGO_ENCODED_RUSTFLAGS"
                | "CARGO_CFG_FEATURE"
                | "CARGO_CFG_UNIX"
                | "CARGO_CFG_WINDOWS"
                | "CARGO_CFG_TARGET_FAMILY"
                | "CARGO_CFG_TARGET_OS"
                | "CARGO_CFG_TARGET_ARCH"
                | "CARGO_CFG_TARGET_VENDOR"
                | "CARGO_CFG_TARGET_ENV"
                | "CARGO_CFG_TARGET_ABI"
                | "CARGO_CFG_TARGET_POINTER_WIDTH"
                | "CARGO_CFG_TARGET_ENDIAN"
                | "CARGO_CFG_TARGET_FEATURE"
                | "CARGO_CFG_TARGET_HAS_ATOMIC"
                | "CARGO_CFG_DEBUG_ASSERTIONS"
                | "CARGO_CFG_PANIC"
                | "CARGO_CFG_PROC_MACRO"
        )
        || unit.features.iter().any(|feature| {
            name == format!("CARGO_FEATURE_{}", feature.replace('-', "_").to_uppercase())
        })
}

fn build_script_sets_environment(
    unit: &super::Unit,
    messages: &[serde_json::Value],
    name: &str,
    value: Option<&str>,
) -> bool {
    if unit.target.kind.iter().any(|kind| kind == "custom-build") {
        return false;
    }
    let outputs = messages
        .iter()
        .filter(|message| {
            message["reason"] == "build-script-executed" && message["package_id"] == unit.pkg_id
        })
        .collect::<Vec<_>>();
    !outputs.is_empty()
        && outputs.iter().all(|message| {
            message["env"].as_array().is_some_and(|environment| {
                environment.iter().any(|pair| {
                    pair[0].as_str() == Some(name) && pair[1].as_str() == value && value.is_some()
                })
            })
        })
}

pub(crate) fn graph(root: &Path, arguments: &Path, output: &Path) -> Result<()> {
    let arguments: Vec<String> = serde_json::from_slice(&fs::read(arguments)?)?;
    ensure!(
        arguments.first().is_some_and(|value| value == "rustc"),
        "ordinary installer must select actual cargo rustc"
    );
    let bytes = super::cargo_output(root, &super::observed_graph_arguments(&arguments))?;
    let graph: UnitGraph = serde_json::from_slice(&bytes)?;
    ensure!(
        graph.version == 1 && graph.roots.len() == 1,
        "ordinary installer requires one actual Cargo root"
    );
    fs::write(output, bytes)?;
    Ok(())
}

pub(crate) fn extract(
    root: &Path,
    graph_path: &Path,
    messages_path: &Path,
    metadata_path: &Path,
    output: &Path,
) -> Result<()> {
    let graph: UnitGraph = serde_json::from_slice(&fs::read(graph_path)?)?;
    ensure!(
        graph.version == 1 && graph.roots.len() == 1,
        "ordinary currency requires one Cargo v1 production root"
    );
    let messages = parse_cargo_messages(&fs::read(messages_path)?)?;
    let metadata: cargo_metadata::Metadata = serde_json::from_slice(&fs::read(metadata_path)?)?;
    let packages = metadata
        .packages
        .into_iter()
        .map(|package| (package.id.to_string(), package))
        .collect::<BTreeMap<_, _>>();
    let selected = selected_indices(&graph)?;
    ensure!(
        selected
            .iter()
            .all(|index| !graph.units[*index].is_std && graph.units[*index].platform.is_none()),
        "cross-compilation/build-std currency is outside the ordinary route"
    );
    let has_macros = selected.iter().any(|index| {
        graph.units[*index]
            .target
            .kind
            .iter()
            .any(|kind| kind == "proc-macro")
    });
    let mut files = BTreeSet::from([
        root.join("Cargo.toml"),
        root.join("Cargo.lock"),
        root.join("rust-toolchain.toml"),
    ]);
    let mut directories = BTreeSet::new();
    let mut environment = BTreeSet::new();
    let mut dep_environment = BTreeMap::<usize, BTreeMap<String, Option<String>>>::new();
    let mut script_environment = BTreeMap::<usize, BTreeSet<String>>::new();
    let mut associations = BTreeMap::new();
    let mut scripts = Vec::new();
    for index in &selected {
        let unit = &graph.units[*index];
        let package = packages
            .get(&unit.pkg_id)
            .context("producing unit missing from actual metadata")?;
        let manifest = package.manifest_path.as_std_path();
        let package_root = manifest.parent().context("producing package root")?;
        files.insert(manifest.to_path_buf());
        // Macro expansion can consume caller-package assets not reported by
        // rustc. This is a finite package closure, never a workspace tree scan.
        if has_macros || unit.mode == "run-custom-build" {
            directories.insert(package_root.to_path_buf());
        }
        if unit.mode == "build" {
            let paths = artifact_dep_info(unit, &graph, &messages, root)?.with_context(|| format!(
                "unsupported ordinary producing inputs: unit {index}, package {}, target {} ({:?}) lacks artifact-associated rustc dep-info; package source closure cannot establish consumed compiler environment",
                unit.pkg_id, unit.target.name, unit.target.kind))?;
            ensure!(
                !paths.is_empty(),
                "unit {index}, package {}, target {} has empty associated rustc dep-info",
                unit.pkg_id,
                unit.target.name
            );
            for path in &paths {
                let text = fs::read_to_string(path)?;
                files.insert(path.clone());
                files.extend(
                    parse_dep_info(&text)?
                        .iter()
                        .map(|path| absolute(root, path)),
                );
                for line in text
                    .lines()
                    .filter_map(|line| line.strip_prefix("# env-dep:"))
                {
                    let (name, value) = line
                        .split_once('=')
                        .map_or((line, None), |(name, value)| (name, Some(value)));
                    // Preserve every actually compiled value under its unit;
                    // two packages can consume different CARGO_PKG_VERSION values.
                    dep_environment
                        .entry(*index)
                        .or_default()
                        .insert(name.to_owned(), value.map(str::to_owned));
                    if !cargo_crate_environment(name, unit, &graph)
                        && !build_script_sets_environment(unit, &messages, name, value)
                    {
                        environment.insert(name.to_owned());
                    }
                }
            }
            associations.insert(*index, paths);
        }
        if unit.mode != "run-custom-build" {
            continue;
        }
        let outputs = messages
            .iter()
            .filter(|message| {
                message["reason"] == "build-script-executed" && message["package_id"] == unit.pkg_id
            })
            .collect::<Vec<_>>();
        ensure!(
            !outputs.is_empty(),
            "selected build script lacks actual Cargo execution record"
        );
        for message in outputs {
            let out_dir = absolute(
                root,
                Path::new(
                    message["out_dir"]
                        .as_str()
                        .context("build script output directory")?,
                ),
            );
            let (output_paths, text) = build_script_directives(&out_dir)?;
            files.extend(output_paths);
            let mut declared = false;
            for line in text.lines() {
                let Some(directive) = line
                    .strip_prefix("cargo::")
                    .or_else(|| line.strip_prefix("cargo:"))
                else {
                    continue;
                };
                if let Some(path) = directive.strip_prefix("rerun-if-changed=") {
                    ensure!(!path.is_empty(), "empty build-script rerun path");
                    let path = absolute(package_root, Path::new(path));
                    if path.is_dir() {
                        directories.insert(path);
                    } else {
                        files.insert(path);
                    }
                    declared = true;
                }
                if let Some(name) = directive.strip_prefix("rerun-if-env-changed=") {
                    ensure!(!name.is_empty(), "empty build-script environment selector");
                    if cargo_build_script_environment(name, unit) {
                        script_environment
                            .entry(*index)
                            .or_default()
                            .insert(name.to_owned());
                    } else {
                        environment.insert(name.to_owned());
                    }
                }
            }
            if !declared {
                directories.insert(package_root.to_path_buf());
            }
            scripts.push(message.clone());
        }
    }
    let root_unit = &graph.units[graph.roots[0]];
    let artifacts = messages
        .iter()
        .filter(|message| artifact_matches(root_unit, message))
        .collect::<Vec<_>>();
    ensure!(
        !artifacts.is_empty(),
        "ordinary producing root lacks actual artifact association"
    );
    let value = serde_json::json!({
        "version": 1, "scope": "ordinary-cargo-inputs", "graph": graph,
        "files": files, "directories": directories, "environment": environment,
        "rustc_environment": dep_environment, "associations": associations,
        "build_script_cargo_environment": script_environment,
        "build_scripts": scripts, "root_artifacts": artifacts,
    });
    fs::write(output, serde_json::to_vec(&value)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn producing_refuses_unassociated_rustc_environment_evidence() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        fs::create_dir(root.join("src")).unwrap();
        let source = root.join("src/main.rs");
        fs::write(
            &source,
            "fn main() { println!(\"{}\", env!(\"CUSTOM_COMPILER_INPUT\")); }",
        )
        .unwrap();
        let manifest = root.join("Cargo.toml");
        fs::write(
            &manifest,
            "[package]\nname='fixture'\nversion='0.1.0'\nedition='2024'\n",
        )
        .unwrap();
        let binary = root.join("fixture-binary");
        fs::write(
            &binary,
            "actual artifact location fixture without its emitted dep-info",
        )
        .unwrap();
        let id = format!("path+file://{}#fixture@0.1.0", root.display());
        let target = serde_json::json!({"name":"fixture", "kind":["bin"], "crate_types":["bin"], "src_path":source, "edition":"2024"});
        let profile = serde_json::json!({"opt_level":"0", "debuginfo":2, "debug_assertions":true, "overflow_checks":true});
        let graph_path = root.join("graph.json");
        fs::write(&graph_path, serde_json::to_vec(&serde_json::json!({"version":1,"roots":[0],"units":[{
            "pkg_id":id,"target":target,"profile":profile,"platform":null,"mode":"build","features":[],"dependencies":[]
        }]})).unwrap()).unwrap();
        let messages_path = root.join("messages.jsonl");
        let mut artifact_profile = profile.clone();
        artifact_profile["test"] = serde_json::json!(false);
        fs::write(&messages_path, format!("{}\n{}\n", serde_json::json!({"reason":"compiler-artifact","package_id":id,"target":target,
            "profile":artifact_profile,"features":[],"filenames":[binary],"executable":binary,"fresh":true}),
            serde_json::json!({"reason":"build-finished","success":true}))).unwrap();
        let metadata_path = root.join("metadata.json");
        fs::write(
            &metadata_path,
            serde_json::to_vec(&serde_json::json!({"version":1,"workspace_root":root,
            "target_directory":root.join("target"),"workspace_members":[id],"resolve":null,
            "packages":[{"name":"fixture","version":"0.1.0","id":id,"manifest_path":manifest,
                "dependencies":[],"targets":[],"features":{},"edition":"2024"}]}))
            .unwrap(),
        )
        .unwrap();
        let output = root.join("selection.json");
        let error = extract(root, &graph_path, &messages_path, &metadata_path, &output)
            .unwrap_err()
            .to_string();
        assert!(error.contains("unit 0"), "{error}");
        assert!(error.contains("package path+file:"), "{error}");
        assert!(error.contains("target fixture"), "{error}");
        assert!(error.contains("consumed compiler environment"), "{error}");
        assert!(!output.exists());
    }

    #[test]
    fn producing_cargo_owned_environment_keeps_unit_scope_and_custom_inputs() {
        let unit = super::super::Unit {
            pkg_id: "fixture#0.1.0".into(),
            target: super::super::CargoTarget {
                name: "fixture".into(),
                kind: vec!["bin".into()],
                crate_types: vec!["bin".into()],
                src_path: "/fixture/src/main.rs".into(),
                edition: "2024".into(),
            },
            profile: serde_json::json!({}),
            platform: None,
            mode: "build".into(),
            features: vec!["chosen-feature".into()],
            dependencies: Vec::new(),
            is_std: false,
        };
        let mut script = unit.clone();
        script.mode = "run-custom-build".into();
        script.target.kind = vec!["custom-build".into()];
        let graph = UnitGraph {
            version: 1,
            units: vec![unit.clone(), script.clone()],
            roots: vec![0],
        };
        for name in [
            "CARGO_PKG_VERSION",
            "CARGO_MANIFEST_DIR",
            "CARGO_CRATE_NAME",
            "OUT_DIR",
        ] {
            assert!(cargo_crate_environment(name, &unit, &graph), "{name}");
        }
        for name in [
            "RUST_BACKTRACE",
            "CARGO_LOG",
            "CARGO_PKG_CUSTOM",
            "CARGO_FEATURE_CHOSEN_FEATURE",
            "CARGO_CFG_TARGET_OS",
        ] {
            assert!(!cargo_crate_environment(name, &unit, &graph), "{name}");
        }
        assert!(!cargo_crate_environment("OUT_DIR", &script, &graph));
        assert!(!cargo_build_script_environment("CARGO_CRATE_NAME", &script));
        assert!(!cargo_build_script_environment(
            "CARGO_PRIMARY_PACKAGE",
            &script
        ));
        assert!(cargo_build_script_environment(
            "CARGO_FEATURE_CHOSEN_FEATURE",
            &script
        ));
        assert!(!cargo_build_script_environment(
            "CARGO_FEATURE_UNDECLARED",
            &script
        ));
        assert!(cargo_build_script_environment(
            "CARGO_CFG_TARGET_OS",
            &script
        ));
        assert!(!cargo_build_script_environment(
            "CARGO_CFG_CUSTOM_AMBIENT",
            &script
        ));
        let output = serde_json::json!({"reason":"build-script-executed", "package_id":unit.pkg_id,
            "env":[["GENERATED_CONTEXT","derived-value"]]});
        assert!(build_script_sets_environment(
            &unit,
            std::slice::from_ref(&output),
            "GENERATED_CONTEXT",
            Some("derived-value")
        ));
        assert!(!build_script_sets_environment(
            &script,
            std::slice::from_ref(&output),
            "GENERATED_CONTEXT",
            Some("derived-value")
        ));
        let other = serde_json::json!({"reason":"build-script-executed", "package_id":unit.pkg_id, "env":[]});
        assert!(!build_script_sets_environment(
            &unit,
            &[output, other],
            "GENERATED_CONTEXT",
            Some("derived-value")
        ));
    }

    #[test]
    fn producing_directives_use_actual_run_stdout_and_refuse_ambiguous_locations() {
        let directory = tempfile::tempdir().unwrap();
        let parent = directory.path().join("selected-unit");
        let out = parent.join("out");
        fs::create_dir_all(parent.join("run")).unwrap();
        let current = parent.join("run/stdout");
        fs::write(&current, "cargo::rerun-if-env-changed=ACTUAL_SELECTOR\n").unwrap();
        let (paths, text) = build_script_directives(&out).unwrap();
        assert_eq!(paths, vec![current.clone()]);
        assert!(text.contains("ACTUAL_SELECTOR"));
        let legacy = parent.join("output");
        fs::write(&legacy, &text).unwrap();
        assert_eq!(build_script_directives(&out).unwrap().0.len(), 2);
        fs::write(&legacy, "cargo:rerun-if-changed=another-input\n").unwrap();
        assert!(build_script_directives(&out).is_err());
        fs::remove_file(&current).unwrap();
        assert_eq!(
            build_script_directives(&out).unwrap().0,
            vec![legacy.clone()]
        );
        fs::remove_file(legacy).unwrap();
        assert!(build_script_directives(&out).is_err());
    }
}
