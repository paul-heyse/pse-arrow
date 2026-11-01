// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit, test-only protobuf composition corpus; never generated during host builds.
use anyhow::{Context, Result, bail};
use pse_codegen::codegen::GeneratedTree;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub(super) fn generate(root: &Path) -> Result<GeneratedTree> {
    let source = root.join("crates/pse-host/tests/fixtures/plan61");
    let output = tempfile::tempdir()?;
    let tools = root.join(".local/plan61-tools/bin");
    prepared_plugins(root, &tools)?;
    let mut paths = vec![tools];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let generated = Command::new("buf")
        .args(["generate", "--output"])
        .arg(output.path())
        .current_dir(&source)
        .env("PATH", std::env::join_paths(paths)?)
        .output()
        .context("execute explicitly prepared Plan61 generators")?;
    if !generated.status.success() {
        bail!(
            "Plan61 generation failed: {}",
            String::from_utf8_lossy(&generated.stderr)
        );
    }
    let descriptor = Command::new("buf")
        .args(["build", "--as-file-descriptor-set", "-o"])
        .arg(output.path().join("descriptor.bin"))
        .current_dir(&source)
        .output()
        .context("compile complete Plan61 descriptor corpus")?;
    if !descriptor.status.success() {
        bail!(
            "Plan61 descriptor failed: {}",
            String::from_utf8_lossy(&descriptor.stderr)
        );
    }
    let destination = PathBuf::from("crates/pse-host/src/generated/plan61");
    let mut tree = GeneratedTree::empty(vec![destination.clone()]);
    collect(output.path(), output.path(), &destination, &mut tree)?;
    Ok(tree)
}

fn prepared_plugins(root: &Path, binaries: &Path) -> Result<()> {
    let metadata: toml::Value = toml::from_str(
        &std::fs::read_to_string(root.join(".local/plan61-tools/.crates.toml"))
            .context("prepare exact Plan61 generators explicitly in .local/plan61-tools")?,
    )?;
    let installed = metadata
        .get("v1")
        .and_then(toml::Value::as_table)
        .context("Plan61 generator installation metadata has no v1 table")?;
    for (package, version, binary) in [
        ("protoc-gen-buffa", "0.9.2", "protoc-gen-buffa"),
        (
            "protoc-gen-buffa-packaging",
            "0.9.2",
            "protoc-gen-buffa-packaging",
        ),
        ("connectrpc-codegen", "0.9.0", "protoc-gen-connect-rust"),
        (
            "protoc-gen-protovalidate-buffa",
            "0.7.2",
            "protoc-gen-protovalidate-buffa",
        ),
    ] {
        let prefix = format!("{package} {version} (");
        let matched = installed.iter().any(|(key, files)| {
            key.starts_with(&prefix)
                && files
                    .as_array()
                    .is_some_and(|files| files.iter().any(|file| file.as_str() == Some(binary)))
        });
        if !matched || !binaries.join(binary).is_file() {
            bail!(
                "prepare {package} {version} ({binary}) explicitly in .local/plan61-tools; global tools are not selected"
            );
        }
    }
    Ok(())
}

fn collect(
    base: &Path,
    directory: &Path,
    destination: &Path,
    tree: &mut GeneratedTree,
) -> Result<()> {
    for entry in std::fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(base, &path, destination, tree)?;
        } else {
            tree.files.insert(
                destination.join(path.strip_prefix(base)?),
                std::fs::read(path)?,
            );
        }
    }
    Ok(())
}
