// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Selected Delta snapshots use the native immutable view contract.
mod common;

#[test]
fn selected_snapshot_provider_keeps_native_mutations_unavailable() {
    let root = common::workspace_root();
    let views = common::code_only_file(&root.join("crates/pse-catalog/src/delta/provider.rs"));
    assert!(views.contains("ViewTable::new(layout.decode(input)?"));
    let owners = common::code_only_file(&root.join("crates/pse-catalog/src/delta/leased.rs"));
    assert!(owners.contains("ownership::provider"));
    assert!(
        !owners.contains("impl TableProvider"),
        "lease ownership must use the complete engine adapter, not a second partial provider"
    );
}
