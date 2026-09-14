// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The admitted snapshot table exposes exactly the accepted read-only provider surface.
#![allow(clippy::expect_used, reason = "static provider-contract assertions")]
mod common;
use regex::Regex;
use std::collections::BTreeSet;

#[test]
fn snapshot_table_has_exact_read_only_trait_method_set() {
    let source = common::code_only_file(
        &common::workspace_root().join("crates/pse-catalog/src/provider/table.rs"),
    );
    let prefix = "impl TableProvider for RelationTable";
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
    let expected = [
        "schema",
        "constraints",
        "table_type",
        "scan",
        "scan_with_args",
        "supports_filters_pushdown",
        "statistics",
    ]
    .map(str::to_owned)
    .into_iter()
    .collect();
    assert_eq!(
        actual, expected,
        "review any new TableProvider capability before exposing it"
    );
}
