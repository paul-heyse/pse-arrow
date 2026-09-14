// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a governance test reports by panicking with the offending path; the workspace panic policy governs library code"
)]

//! The regex layer of the blueprint §24.1 governance greps.
//!
//! `clippy.toml`'s `disallowed-methods` is the type-aware layer and catches the same calls
//! with full paths; this one catches them in macro arguments, in strings passed to a
//! builder, and in code that has not compiled yet. Two layers, because each misses what
//! the other sees.
//!
//! Generated sources are excluded: the fix for a violation there is the generator, and
//! `cargo xtask codegen --check` guards them.

mod common;

use std::path::{Path, PathBuf};

use regex::Regex;

/// A ban: the pattern, where it applies (empty = everywhere scanned), and why.
struct Ban {
    pattern: &'static str,
    scopes: &'static [&'static str],
    reason: &'static str,
}

const BANS: &[Ban] = &[
    Ban {
        pattern: r"Field::extension_type\(",
        scopes: &[],
        reason: "panics on a missing or invalid extension; use `try_extension_type` (blueprint §4.4)",
    },
    Ban {
        pattern: r"SessionConfig::set_str",
        scopes: &[],
        reason: "panics on an invalid key; use typed ConfigOptions (blueprint §23.2, `config.invalid`)",
    },
    Ban {
        pattern: r"SchemaLike::from_type",
        scopes: &[],
        reason: "schemas come from the registry, never inferred (blueprint §5.3)",
    },
    Ban {
        pattern: r"from_samples\(",
        scopes: &[],
        reason: "schemas come from the registry, never inferred from data (blueprint §5.3)",
    },
    Ban {
        pattern: r"SERDE_ARROW:",
        scopes: &[],
        reason: "a serde_arrow field-metadata key on a platform relation is a contract violation (blueprint §5.3)",
    },
    Ban {
        pattern: r"config_options\(",
        scopes: &["crates/pse-kernels", "crates/pse-kernels-ext"],
        reason: "a kernel that reads engine configuration is not a pure function of its inputs (blueprint §9, §24.1)",
    },
    Ban {
        pattern: r"pretty_format",
        scopes: &["crates/pse-ids", "crates/pse-catalog"],
        reason: "a rendered table is never an identity or manifest input (blueprint §5.3)",
    },
    Ban {
        pattern: r"try_with_compression",
        scopes: &["crates/pse-ids", "crates/pse-catalog"],
        reason: "canonical IPC is uncompressed (blueprint §5.3, pse.canon.v2)",
    },
    Ban {
        pattern: r#"format!\("\{:\?\}"#,
        scopes: &["crates/pse-ids"],
        reason: "a Debug rendering is not a stable hash input; hash canonical bytes (blueprint §5.3)",
    },
];

/// `crates/*/src` plus `xtask/src`: the code that is allowed to exist today.
fn scanned_files() -> Vec<PathBuf> {
    let mut files = common::crate_sources();
    files.extend(common::rust_sources(
        &common::workspace_root().join("xtask/src"),
    ));
    files
}

/// Does `path` fall inside one of a ban's scopes?
fn in_scope(path: &Path, scopes: &[&str]) -> bool {
    if scopes.is_empty() {
        return true;
    }
    let rel = common::rel(path);
    scopes
        .iter()
        .any(|scope| rel.starts_with(&format!("{scope}/")))
}

#[test]
fn no_banned_call_shapes() {
    let files = scanned_files();
    assert!(
        !files.is_empty(),
        "the scan found no sources; the paths are wrong"
    );

    let mut problems: Vec<String> = Vec::new();
    for ban in BANS {
        let re = Regex::new(ban.pattern).expect("static regex");
        for path in &files {
            if !in_scope(path, ban.scopes) {
                continue;
            }
            for (number, line) in common::read(path).lines().enumerate() {
                if re.is_match(&common::code_only(line)) {
                    problems.push(format!(
                        "{}:{}: `{}` — {}",
                        common::rel(path),
                        number + 1,
                        ban.pattern,
                        ban.reason
                    ));
                }
            }
        }
    }

    assert!(
        problems.is_empty(),
        "banned patterns:\n  {}",
        problems.join("\n  ")
    );
}
