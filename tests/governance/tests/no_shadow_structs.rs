// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a governance test reports by panicking with the offending path; the workspace panic policy governs library code"
)]

//! No hand-written struct shadows a generated relation view.
//!
//! Blueprint §24.1: "no hand-written struct shadows a relation". Two definitions of the
//! same row shape is how a contract silently forks — one of them gets a field, the other
//! does not, and both compile.
//!
//! A missing or empty generated tree is a failure, never a vacuous pass.

mod common;

use std::collections::BTreeMap;

use regex::Regex;

#[test]
fn no_hand_written_struct_shadows_a_generated_one() {
    let root = common::workspace_root();
    let struct_re =
        Regex::new(r"(?m)^\s*pub(?:\([^)]*\))?\s+struct\s+(\w+)").expect("static regex");
    let mut generated: BTreeMap<String, String> = BTreeMap::new();
    for directory in ["pse-model", "pse-relations"] {
        let directory = root.join(format!("crates/{directory}/src/generated"));
        assert!(
            directory.is_dir(),
            "missing generated root: {}",
            directory.display()
        );
        let mut count = 0;
        for path in common::rust_sources(&directory) {
            for caps in struct_re.captures_iter(&common::read(&path)) {
                generated.insert(caps[1].to_owned(), common::rel(&path));
                count += 1;
            }
        }
        assert!(count > 0, "empty generated root: {}", directory.display());
    }

    let mut problems: Vec<String> = Vec::new();
    for path in common::crate_sources() {
        if common::rel(&path).contains("/src/generated/") {
            continue;
        }
        for caps in struct_re.captures_iter(&common::read(&path)) {
            let name = &caps[1];
            if let Some(origin) = generated.get(name) {
                problems.push(format!(
                    "{}: `pub struct {name}` shadows the generated one in {origin}",
                    common::rel(&path)
                ));
            }
        }
    }

    assert!(
        problems.is_empty(),
        "hand-written structs shadowing generated relation views:\n  {}\n\
         Use the generated view, or change the registry and regenerate.",
        problems.join("\n  ")
    );
}
