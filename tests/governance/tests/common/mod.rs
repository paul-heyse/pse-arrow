// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    dead_code,
    reason = "shared helpers; each governance test binary uses a subset"
)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a governance test reports by panicking with the offending path; the workspace panic policy governs library code"
)]

//! Shared helpers for the governance tests: locating the workspace, parsing manifests,
//! walking Rust sources, and the crude comment/string stripper the grep-layer checks use.

use std::path::{Path, PathBuf};

#[path = "../../../../scripts/workspace.rs"]
pub(crate) mod workspace;

/// Discover the runtime checkout so relocated/cached test binaries inspect their caller.
pub(crate) fn workspace_root() -> PathBuf {
    let start = std::env::current_dir().expect("reading invocation directory");
    workspace::find_workspace_root(&start).unwrap_or_else(|error| panic!("{error}"))
}

/// Reads a file, or panics naming it.
pub(crate) fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|err| panic!("reading {}: {err}", path.display()))
}

/// Parses a TOML file, or panics naming it.
pub(crate) fn parse_toml(path: &Path) -> toml::Value {
    toml::from_str(&read(path)).unwrap_or_else(|err| panic!("parsing {}: {err}", path.display()))
}

/// Follows a dotted path through a TOML document.
pub(crate) fn dig<'v>(value: &'v toml::Value, path: &[&str]) -> Option<&'v toml::Value> {
    path.iter().try_fold(value, |acc, key| acc.get(key))
}

/// `[workspace.package]` of the root manifest as `(key, value)` lookups.
pub(crate) fn root_manifest() -> toml::Value {
    parse_toml(&workspace_root().join("Cargo.toml"))
}

/// `[workspace.members]` with `crates/*` expanded, as `(package directory name, path)`.
pub(crate) fn member_dirs() -> Vec<PathBuf> {
    let root = workspace_root();
    let manifest = root_manifest();
    let members = dig(&manifest, &["workspace", "members"])
        .and_then(toml::Value::as_array)
        .expect("[workspace] members");
    let mut out = Vec::new();
    for member in members {
        let pattern = member.as_str().expect("member entries are strings");
        if let Some(prefix) = pattern.strip_suffix("/*") {
            let mut dirs: Vec<PathBuf> = std::fs::read_dir(root.join(prefix))
                .unwrap_or_else(|err| panic!("reading {prefix}: {err}"))
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.join("Cargo.toml").is_file())
                .collect();
            dirs.sort();
            out.extend(dirs);
        } else {
            out.push(root.join(pattern));
        }
    }
    out
}

/// Directory name plus path for every crate under `crates/`, sorted.
pub(crate) fn crate_dirs() -> Vec<(String, PathBuf)> {
    let crates = workspace_root().join("crates");
    let mut out: Vec<(String, PathBuf)> = std::fs::read_dir(&crates)
        .unwrap_or_else(|err| panic!("reading {}: {err}", crates.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .map(|path| {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_owned();
            (name, path)
        })
        .collect();
    out.sort();
    out
}

/// Every `.rs` file under `dir`, gitignore-aware, sorted. Missing directories yield an
/// empty list so a check can be written before the tree it guards exists.
pub(crate) fn rust_sources(dir: &Path) -> Vec<PathBuf> {
    if !dir.exists() {
        return Vec::new();
    }
    let mut out: Vec<PathBuf> = ignore::WalkBuilder::new(dir)
        .hidden(false)
        .build()
        .filter_map(Result::ok)
        .map(ignore::DirEntry::into_path)
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .collect();
    out.sort();
    out
}

/// Every `.rs` file under `crates/*/src`, minus the generated trees. Generated sources are
/// excluded from every grep-layer check: the fix for a violation there is the generator,
/// and `cargo xtask codegen --check` is what guards them.
pub(crate) fn crate_sources() -> Vec<PathBuf> {
    crate_dirs()
        .into_iter()
        .flat_map(|(_, dir)| rust_sources(&dir.join("src")))
        .filter(|path| !is_generated(path))
        .collect()
}

/// True for files a generator owns.
pub(crate) fn is_generated(path: &Path) -> bool {
    let text = path.to_string_lossy().replace('\\', "/");
    text.contains("/src/generated/") || text.ends_with("/bindings.rs")
}

/// Path relative to the workspace root, for messages.
pub(crate) fn rel(path: &Path) -> String {
    path.strip_prefix(workspace_root())
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Drops `//`-comments (doc comments included) and the contents of string literals from a
/// single line, so a grep-layer check sees code and not prose.
///
/// Deliberately crude: block comments and raw strings with hashes are not modelled. That
/// is acceptable for a check whose failure mode is "a reviewer looks at the line".
pub(crate) fn code_only(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len());
    let mut i = 0;
    let mut in_string = false;
    while i < chars.len() {
        let ch = chars[i];
        if in_string {
            if ch == '\\' {
                i += 2;
                continue;
            }
            if ch == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if ch == '"' {
            in_string = true;
            i += 1;
            continue;
        }
        if ch == '/' && chars.get(i + 1) == Some(&'/') {
            break;
        }
        out.push(ch);
        i += 1;
    }
    out
}

/// The whole file with `//`-comments and string contents removed, line by line.
pub(crate) fn code_only_file(path: &Path) -> String {
    read(path)
        .lines()
        .map(code_only)
        .collect::<Vec<_>>()
        .join("\n")
}

/// `cargo metadata --locked --format-version 1` for the workspace.
pub(crate) fn metadata() -> cargo_metadata::Metadata {
    cargo_metadata::MetadataCommand::new()
        .manifest_path(workspace_root().join("Cargo.toml"))
        .other_options(vec!["--locked".to_owned()])
        .exec()
        .expect("cargo metadata --locked")
}
