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
//! `crates/pse-relations/src/generated/` does not exist in phase 0, so this passes by
//! having nothing to compare against. It is written now so it is already in the gate on
//! the day the generator first emits into that directory.

mod common;

use std::collections::BTreeMap;

use regex::Regex;

#[test]
fn no_hand_written_struct_shadows_a_generated_one() {
    let root = common::workspace_root();
    let generated_dir = root.join("crates/pse-relations/src/generated");
    if !generated_dir.is_dir() {
        // Documented no-op; `blueprint_crates_all_exist` and codegen --check cover the
        // day the directory appears.
        return;
    }

    let struct_re =
        Regex::new(r"(?m)^\s*pub(?:\([^)]*\))?\s+struct\s+(\w+)").expect("static regex");

    let mut generated: BTreeMap<String, String> = BTreeMap::new();
    for path in common::rust_sources(&generated_dir) {
        for caps in struct_re.captures_iter(&common::read(&path)) {
            generated.insert(caps[1].to_owned(), common::rel(&path));
        }
    }
    assert!(
        !generated.is_empty(),
        "crates/pse-relations/src/generated exists but declares no `pub struct`; the \
         generator emitted nothing"
    );

    let mut problems: Vec<String> = Vec::new();
    for path in common::crate_sources() {
        if common::rel(&path).starts_with("crates/pse-relations/src/generated/") {
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
