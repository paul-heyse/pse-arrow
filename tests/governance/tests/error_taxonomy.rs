// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a governance test reports by panicking with the offending path; the workspace panic policy governs library code"
)]

//! Every `pub enum *Error` under `crates/*/src` derives `thiserror::Error` and
//! `miette::Diagnostic`, and no crate reaches for `anyhow` or `miette::Result`.
//!
//! Blueprint §23.2: every `pse-*` crate returns concrete error enums carrying the §23.2
//! class as a `#[diagnostic(code(...))]`; `miette::Result` and the graphical reporter
//! exist only in the CLI and the driver (`xtask`, which is outside this scan). An
//! `anyhow::Error` in a library erases exactly the structure a finding is made of.
//!
//! Trivially green in phase 0 — which is the point: the first error enum written is
//! checked, not the hundredth.

mod common;

use regex::Regex;

#[test]
fn error_enums_derive_thiserror_and_miette() {
    let enum_re = Regex::new(r"^\s*pub(?:\([^)]*\))?\s+enum\s+(\w*Error)\b").expect("static regex");
    let mut problems: Vec<String> = Vec::new();

    for path in common::crate_sources() {
        let text = common::read(&path);
        let lines: Vec<&str> = text.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            let Some(caps) = enum_re.captures(line) else {
                continue;
            };
            let name = &caps[1];

            // Walk back over attributes, doc comments and blank lines to the derive block.
            let mut attributes = String::new();
            let mut cursor = index;
            while cursor > 0 {
                cursor -= 1;
                let previous = lines[cursor].trim();
                if previous.is_empty() || previous.starts_with("///") || previous.starts_with("//")
                {
                    continue;
                }
                if previous.starts_with('#')
                    || previous.starts_with(')')
                    || previous.ends_with(',')
                    || previous.starts_with('"')
                {
                    attributes.insert_str(0, previous);
                    continue;
                }
                break;
            }

            for required in ["thiserror::Error", "miette::Diagnostic"] {
                if !attributes.contains(required) {
                    problems.push(format!(
                        "{}:{}: `pub enum {name}` does not derive {required} (blueprint §23.2)",
                        common::rel(&path),
                        index + 1
                    ));
                }
            }
        }
    }

    assert!(
        problems.is_empty(),
        "error taxonomy:\n  {}",
        problems.join("\n  ")
    );
}

#[test]
fn no_anyhow_or_miette_result_in_library_crates() {
    let patterns = [
        (
            Regex::new(r"\banyhow\b").expect("static regex"),
            "anyhow erases the structure a finding is made of; return a concrete thiserror enum",
        ),
        (
            Regex::new(r"miette\s*::\s*Result").expect("static regex"),
            "miette::Result and the graphical reporter live in the driver only (blueprint §23.2)",
        ),
    ];
    let mut problems: Vec<String> = Vec::new();

    for path in common::crate_sources() {
        for (number, line) in common::read(&path).lines().enumerate() {
            let code = common::code_only(line);
            for (re, why) in &patterns {
                if re.is_match(&code) {
                    problems.push(format!("{}:{}: {why}", common::rel(&path), number + 1));
                }
            }
        }
    }

    assert!(
        problems.is_empty(),
        "error policy:\n  {}",
        problems.join("\n  ")
    );
}
