// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The admitted snapshot table exposes exactly the accepted read-only provider surface.
#![allow(clippy::expect_used, reason = "static provider-contract assertions")]
mod common;
use regex::Regex;
use std::collections::BTreeSet;

#[test]
fn selected_snapshot_provider_keeps_native_mutations_unavailable() {
    let source = common::code_only_file(
        &common::workspace_root().join("crates/pse-catalog/src/delta/leased.rs"),
    );
    let prefix = "impl TableProvider for LeasedProvider";
    assert_eq!(
        source.matches(prefix).count(),
        1,
        "one generic table implementation"
    );
    let implementation = source
        .split(prefix)
        .nth(1)
        .expect("actual provider implementation");
    let start = implementation.find('{').expect("impl body");
    let mut depth = 0_u32;
    let end = implementation[start..]
        .char_indices()
        .find_map(|(index, character)| {
            if character == '{' {
                depth += 1;
            }
            if character == '}' {
                depth -= 1;
                if depth == 0 {
                    return Some(start + index);
                }
            }
            None
        })
        .expect("closed impl body");
    let method = Regex::new(r"\bfn\s+(\w+)\s*[<(]").expect("method names");
    let actual = method
        .captures_iter(&implementation[start..end])
        .map(|capture| capture[1].to_owned())
        .collect::<BTreeSet<_>>();
    for mutation in [
        "insert_into",
        "delete_from",
        "update",
        "truncate",
        "merge_into",
    ] {
        assert!(
            !actual.contains(mutation),
            "selected snapshot exposed {mutation}"
        );
    }
    assert!(actual.contains("scan"));
    assert!(actual.contains("schema"));
}
