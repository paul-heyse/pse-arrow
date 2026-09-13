// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a governance test reports by panicking with the offending path; the workspace panic policy governs library code"
)]

//! `unsafe` appears only in the crates `[workspace.metadata.pse].unsafe-allowlist` names,
//! and each of those carries `#![allow(unsafe_code, reason = ...)]`.
//!
//! `unsafe_code = "deny"` is a workspace lint, so the compiler already stops an accidental
//! `unsafe` block. What the compiler cannot stop is a crate quietly adding the crate-level
//! allow to get past it. This test makes that a governance failure instead, and is the
//! trigger condition for re-adopting Miri (docs/adr/register.md): today the only `unsafe`
//! in the workspace is FFI.

mod common;

use std::collections::BTreeSet;

use regex::Regex;
use toml::Value;

fn allowlist() -> BTreeSet<String> {
    let manifest = common::root_manifest();
    common::dig(
        &manifest,
        &["workspace", "metadata", "pse", "unsafe-allowlist"],
    )
    .and_then(Value::as_array)
    .expect("[workspace.metadata.pse] unsafe-allowlist")
    .iter()
    .map(|v| {
        v.as_str()
            .expect("allowlist entries are strings")
            .to_owned()
    })
    .collect()
}

#[test]
fn unsafe_appears_only_in_allowlisted_crates() {
    let allowed = allowlist();
    let re = Regex::new(r"\bunsafe\b").expect("static regex");
    let mut problems: Vec<String> = Vec::new();

    for (name, dir) in common::crate_dirs() {
        if allowed.contains(&name) {
            continue;
        }
        for path in common::rust_sources(&dir.join("src")) {
            for (number, line) in common::read(&path).lines().enumerate() {
                if re.is_match(&common::code_only(line)) {
                    problems.push(format!("{}:{}", common::rel(&path), number + 1));
                }
            }
        }
    }

    assert!(
        problems.is_empty(),
        "`unsafe` outside the unsafe-allowlist:\n  {}\n\
         Adding a crate to the allowlist is an ADR, and re-enables Miri in the scheduled \
         workflow (register row).",
        problems.join("\n  ")
    );
}

#[test]
fn allowlisted_crates_declare_the_allow_with_a_reason() {
    for name in allowlist() {
        let lib = common::workspace_root()
            .join("crates")
            .join(&name)
            .join("src/lib.rs");
        assert!(
            lib.is_file(),
            "unsafe-allowlist names `{name}`, which has no src/lib.rs"
        );
        // Whitespace-insensitive: rustfmt may wrap the attribute across lines.
        let squashed: String = common::read(&lib)
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        assert!(
            squashed.contains("#![allow(unsafe_code,reason"),
            "{name}/src/lib.rs is on the unsafe-allowlist but does not carry \
             `#![allow(unsafe_code, reason = \"...\")]`. Either declare it with a reason or \
             take the crate off the allowlist."
        );
    }
}
