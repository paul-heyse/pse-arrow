// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a governance test reports by panicking with the offending path; the workspace panic policy governs library code"
)]

//! Every `[workspace.dependencies]` entry that blueprint §3.1 declares a version for
//! declares that same version.
//!
//! §3.1's tables are the PIN authority; the manifest is its mechanical form. Without this
//! test the two drift, and the blueprint quietly stops describing what gets built.
//!
//! §3.1 is **not** an admission list. A dependency the blueprint does not mention is
//! reported and allowed: during phases 0-1 nothing about which library you use blocks a
//! merge (ADR-0066, blueprint §3.3.2, `docs/dev/dependency-policy.md`). The drift arm
//! below is what still fails, because a pin that disagrees with §3.1 makes the blueprint
//! false about the one type universe.
//!
//! Family subcrates inherit their family's row: §3.1 writes `arrow`, `arrow-*`, `parquet`
//! on one line and `datafusion` and every `datafusion-*` subcrate on another, because a
//! family moves as a unit.

mod common;

use std::collections::{BTreeMap, BTreeSet};

use regex::Regex;
use toml::Value;

/// name -> version, parsed out of the tables between `### 3.1` and `### 3.2`.
fn blueprint_pins() -> BTreeMap<String, String> {
    let path = common::workspace_root().join("docs/authoritative_design/blueprint.md");
    let text = common::read(&path);
    let start = text.find("### 3.1").expect("blueprint §3.1 heading");
    let end = text[start..]
        .find("### 3.2")
        .map_or(text.len(), |i| start + i);
    let section = &text[start..end];

    let name_re = Regex::new(r"`([A-Za-z0-9_*.+-]+)`").expect("static regex");
    let version_re = Regex::new(r"\b(\d+\.\d+(?:\.\d+)?)\b").expect("static regex");

    let mut pins: BTreeMap<String, String> = BTreeMap::new();
    for line in section.lines() {
        let line = line.trim();
        if !line.starts_with('|') {
            continue;
        }
        let cells: Vec<&str> = line.trim_matches('|').split('|').collect();
        let [names_cell, version_cell, ..] = cells.as_slice() else {
            continue;
        };
        let names: Vec<String> = name_re
            .captures_iter(names_cell)
            .map(|caps| caps[1].to_owned())
            .filter(|name| !name.contains('*'))
            .collect();
        let versions: Vec<String> = version_re
            .captures_iter(version_cell)
            .map(|caps| caps[1].to_owned())
            .collect();
        if names.is_empty() || versions.is_empty() {
            continue;
        }
        let pairs: Vec<(String, String)> = if versions.len() == 1 {
            names
                .into_iter()
                .map(|n| (n, versions[0].clone()))
                .collect()
        } else if names.len() == versions.len() {
            names.into_iter().zip(versions).collect()
        } else {
            continue;
        };
        for (name, version) in pairs {
            if let Some(existing) = pins.get(&name) {
                assert_eq!(
                    existing, &version,
                    "blueprint §3.1 pins `{name}` twice with different versions \
                     ({existing} and {version}); the tables disagree with each other"
                );
            }
            pins.insert(name, version);
        }
    }
    assert!(
        pins.len() > 20,
        "parsed only {} pins out of blueprint §3.1; the table shape changed and this parser \
         needs updating",
        pins.len()
    );
    pins
}

/// Names exempted by `tooling_deps.toml`, each with a non-empty reason.
fn tooling_exemptions() -> BTreeSet<String> {
    let path = common::workspace_root().join("tests/governance/tooling_deps.toml");
    let doc = common::parse_toml(&path);
    let entries = doc
        .get("dep")
        .and_then(Value::as_array)
        .expect("tooling_deps.toml [[dep]] entries");
    entries
        .iter()
        .map(|entry| {
            let name = entry
                .get("name")
                .and_then(Value::as_str)
                .expect("[[dep]] name")
                .to_owned();
            let reason = entry
                .get("reason")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .trim();
            assert!(
                !reason.is_empty(),
                "tooling_deps.toml entry `{name}` has no reason; an exemption without one is \
                 a permanent silent hole"
            );
            name
        })
        .collect()
}

/// The version requirement a `[workspace.dependencies]` entry declares, if it is not a
/// path dependency.
fn declared_version(value: &Value) -> Option<String> {
    match value {
        Value::String(req) => Some(req.clone()),
        Value::Table(table) => {
            if table.contains_key("path") {
                return None;
            }
            table
                .get("version")
                .and_then(Value::as_str)
                .map(str::to_owned)
        }
        _ => None,
    }
}

/// The blueprint row a dependency answers to: itself, or its family's.
fn blueprint_version(name: &str, pins: &BTreeMap<String, String>) -> Option<String> {
    if let Some(version) = pins.get(name) {
        return Some(version.clone());
    }
    for family in ["arrow", "datafusion", "parquet", "pyo3"] {
        if name.starts_with(&format!("{family}-"))
            && let Some(version) = pins.get(family)
        {
            return Some(version.clone());
        }
    }
    None
}

/// `=0.29.2` satisfies a `0.29` row; `=59.3.0` satisfies a `59.3.0` row. A shorter
/// blueprint version is a minor-level pin and matches on a component boundary.
fn satisfies(declared: &str, blueprint: &str) -> bool {
    let declared = declared.trim().trim_start_matches('=').trim();
    declared == blueprint || declared.starts_with(&format!("{blueprint}."))
}

#[test]
fn declared_pins_do_not_drift_from_blueprint() {
    let pins = blueprint_pins();
    let exempt = tooling_exemptions();
    let manifest = common::root_manifest();
    let deps = common::dig(&manifest, &["workspace", "dependencies"])
        .and_then(Value::as_table)
        .expect("[workspace.dependencies]");

    let mut problems: Vec<String> = Vec::new();
    let mut undescribed: Vec<String> = Vec::new();
    for (name, value) in deps {
        let Some(declared) = declared_version(value) else {
            continue; // path dependency: an internal crate, versioned by the workspace
        };
        if exempt.contains(name) {
            continue;
        }
        match blueprint_version(name, &pins) {
            // Not a failure. §3.1 pins what the platform is built around; it does not
            // enumerate what the platform may use (ADR-0066).
            None => undescribed.push(format!("{name} = {declared}")),
            Some(expected) if !satisfies(&declared, &expected) => problems.push(format!(
                "`{name}`: manifest declares {declared}, blueprint §3.1 says {expected}"
            )),
            Some(_) => {}
        }
    }

    if !undescribed.is_empty() {
        eprintln!(
            "note: {} [workspace.dependencies] entr{} not described in blueprint §3.1, which \
             is expected while dependency admission is advisory (ADR-0066):\n  {}",
            undescribed.len(),
            if undescribed.len() == 1 {
                "y is"
            } else {
                "ies are"
            },
            undescribed.join("\n  ")
        );
    }

    assert!(
        problems.is_empty(),
        "pin drift:\n  {}",
        problems.join("\n  ")
    );
}

#[test]
fn tooling_exemptions_are_well_formed() {
    let manifest = common::root_manifest();
    let deps = common::dig(&manifest, &["workspace", "dependencies"])
        .and_then(Value::as_table)
        .expect("[workspace.dependencies]");
    let stale: Vec<String> = tooling_exemptions()
        .into_iter()
        .filter(|name| !deps.contains_key(name))
        .collect();
    // Not a failure: an exemption only ever suppressed the "not in §3.1" report, which no
    // longer fails either. The list stays because its reasons explain why these are not
    // platform commitments.
    if !stale.is_empty() {
        eprintln!(
            "note: tooling_deps.toml lists dependencies that are no longer declared: \
             {stale:?}. Prune them when convenient."
        );
    }
}
