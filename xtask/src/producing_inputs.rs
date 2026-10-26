// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Ordinary Cargo freshness inputs, not an arbitrary executable-I/O proof.
//! Reuses the producer's actual unit/artifact/dep-info association. Opaque macro
//! and build-script package inputs deliberately retain conservative membership.

use super::{UnitGraph, absolute, artifact_dep_info, artifact_matches, parse_cargo_messages,
    parse_dep_info, selected_indices};
use anyhow::{Context, Result, ensure};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

fn build_script_directives(out_dir: &Path) -> Result<(Vec<PathBuf>, String)> {
    let paths = super::build_script_stdout_paths(out_dir)?.into_iter()
        .filter(|path| path.is_file()).collect::<Vec<_>>();
    ensure!(!paths.is_empty(), "actual Cargo build-script directives unavailable for {}", out_dir.display());
    let text = fs::read_to_string(&paths[0])
        .with_context(|| format!("actual Cargo build-script directives {}", paths[0].display()))?;
    for path in paths.iter().skip(1) {
        ensure!(fs::read_to_string(path)? == text,
            "conflicting actual Cargo build-script directive locations for {}", out_dir.display());
    }
    Ok((paths, text))
}

pub(crate) fn graph(root: &Path, arguments: &Path, output: &Path) -> Result<()> {
    let arguments: Vec<String> = serde_json::from_slice(&fs::read(arguments)?)?;
    ensure!(arguments.first().is_some_and(|value| value == "rustc"), "ordinary installer must select actual cargo rustc");
    let bytes = super::cargo_output(root, &super::observed_graph_arguments(&arguments))?;
    let graph: UnitGraph = serde_json::from_slice(&bytes)?;
    ensure!(graph.version == 1 && graph.roots.len() == 1, "ordinary installer requires one actual Cargo root");
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
    ensure!(graph.version == 1 && graph.roots.len() == 1, "ordinary currency requires one Cargo v1 production root");
    let messages = parse_cargo_messages(&fs::read(messages_path)?)?;
    let metadata: cargo_metadata::Metadata = serde_json::from_slice(&fs::read(metadata_path)?)?;
    let packages = metadata.packages.into_iter()
        .map(|package| (package.id.to_string(), package)).collect::<BTreeMap<_, _>>();
    let selected = selected_indices(&graph)?;
    ensure!(selected.iter().all(|index| !graph.units[*index].is_std && graph.units[*index].platform.is_none()),
        "cross-compilation/build-std currency is outside the ordinary route");
    let has_macros = selected.iter().any(|index| graph.units[*index].target.kind.iter().any(|kind| kind == "proc-macro"));
    let mut files = BTreeSet::from([root.join("Cargo.toml"), root.join("Cargo.lock"), root.join("rust-toolchain.toml")]);
    let mut directories = BTreeSet::new();
    let mut environment = BTreeSet::new();
    let mut dep_environment = BTreeMap::new();
    let mut associations = BTreeMap::new();
    let mut scripts = Vec::new();
    for index in &selected {
        let unit = &graph.units[*index];
        let package = packages.get(&unit.pkg_id).context("producing unit missing from actual metadata")?;
        let manifest = package.manifest_path.as_std_path();
        let package_root = manifest.parent().context("producing package root")?;
        files.insert(manifest.to_path_buf());
        // Macro expansion can consume caller-package assets not reported by
        // rustc. This is a finite package closure, never a workspace tree scan.
        if has_macros || unit.mode == "run-custom-build" {
            directories.insert(package_root.to_path_buf());
        }
        if unit.mode == "build" {
            if let Some(paths) = artifact_dep_info(unit, &graph, &messages, root)? {
                for path in &paths {
                    let text = fs::read_to_string(path)?;
                    files.insert(path.clone());
                    files.extend(parse_dep_info(&text)?.iter().map(|path| absolute(root, path)));
                    for line in text.lines().filter_map(|line| line.strip_prefix("# env-dep:")) {
                        let (name, value) = line.split_once('=').map_or((line, None), |(name, value)| (name, Some(value)));
                        environment.insert(name.to_owned());
                        dep_environment.insert(name.to_owned(), value.map(str::to_owned));
                    }
                }
                associations.insert(*index, paths);
            } else {
                // Cargo cache relocations and opaque targets may lack a uniquely
                // associated rustc depfile. Preserve the whole package instead.
                directories.insert(package_root.to_path_buf());
            }
        }
        if unit.mode != "run-custom-build" { continue; }
        let outputs = messages.iter().filter(|message| message["reason"] == "build-script-executed" && message["package_id"] == unit.pkg_id).collect::<Vec<_>>();
        ensure!(!outputs.is_empty(), "selected build script lacks actual Cargo execution record");
        for message in outputs {
            let out_dir = absolute(root, Path::new(message["out_dir"].as_str().context("build script output directory")?));
            let (output_paths, text) = build_script_directives(&out_dir)?;
            files.extend(output_paths);
            let mut declared = false;
            for line in text.lines() {
                let Some(directive) = line.strip_prefix("cargo::").or_else(|| line.strip_prefix("cargo:")) else { continue; };
                if let Some(path) = directive.strip_prefix("rerun-if-changed=") {
                    ensure!(!path.is_empty(), "empty build-script rerun path");
                    let path = absolute(package_root, Path::new(path));
                    if path.is_dir() { directories.insert(path); } else { files.insert(path); }
                    declared = true;
                }
                if let Some(name) = directive.strip_prefix("rerun-if-env-changed=") {
                    ensure!(!name.is_empty(), "empty build-script environment selector");
                    environment.insert(name.to_owned());
                }
            }
            if !declared { directories.insert(package_root.to_path_buf()); }
            scripts.push(message.clone());
        }
    }
    let root_unit = &graph.units[graph.roots[0]];
    let artifacts = messages.iter().filter(|message| artifact_matches(root_unit, message)).collect::<Vec<_>>();
    ensure!(!artifacts.is_empty(), "ordinary producing root lacks actual artifact association");
    let value = serde_json::json!({
        "version": 1, "scope": "ordinary-cargo-inputs", "graph": graph,
        "files": files, "directories": directories, "environment": environment,
        "rustc_environment": dep_environment, "associations": associations,
        "build_scripts": scripts, "root_artifacts": artifacts,
    });
    fs::write(output, serde_json::to_vec(&value)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(build_script_directives(&out).unwrap().0, vec![legacy.clone()]);
        fs::remove_file(legacy).unwrap();
        assert!(build_script_directives(&out).is_err());
    }
}
