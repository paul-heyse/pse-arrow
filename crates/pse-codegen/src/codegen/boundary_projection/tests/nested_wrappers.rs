// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use std::collections::BTreeMap;

projection_struct! {
    struct NestedCollections {
        projection: { name: "HTTP2Case", version: 1, direction: Direction::Both,
            reserved_tags: [], reserved_names: [] };
        trajectories: BTreeMap<u64, Vec<f64>> => { tag: 1, direction: Direction::Both, rules: [] };
        sparse_start: Option<BTreeMap<pse_ids::SemanticId, f64>> => { tag: 2, direction: Direction::Both, rules: [] };
        identities: Vec<Vec<pse_ids::SemanticId>> => { tag: 3, direction: Direction::Both, rules: [] };
    }
}

#[test]
fn collection_map_values_live_outside_synthetic_entries_and_use_actual_buffa_modules() {
    let graph = ProjectionGraph::of::<NestedCollections>().unwrap();
    let output = generate_with_config(&graph, "pse.nested.v1", "nested.proto", &imports(),
        &default_config("crate::wire::__buffa_fds")).unwrap();
    let file = output.descriptor_set.file.iter().find(|file| file.name.as_deref() == Some("nested.proto")).unwrap();
    let owner = file.message_type.iter().find(|owner| owner.name.as_deref() == Some("HTTP2Case")).unwrap();
    let entry = owner.nested_type.iter().find(|nested| nested.name.as_deref() == Some("Field1Entry")).unwrap();
    assert_eq!(entry.options.as_option().unwrap().map_entry, Some(true));
    assert!(entry.nested_type.is_empty(), "synthetic map entries cannot hide emitted value types");
    assert_eq!(entry.field[1].type_name.as_deref(), Some(".pse.nested.v1.HTTP2Case.Field1Value"));
    assert!(owner.nested_type.iter().any(|nested| nested.name.as_deref() == Some("Field1Value")));
    let source = &output.sources["pse.nested.v1.rs"];
    assert!(source.contains("pub mod http2case"));
    assert!(source.contains("pub mod field2value"));
    assert!(!source.contains("field1entry::Field2Value"));
    let adapters = native_adapters(&graph, &["HTTP2Case"], "crate::wire", "pse_codegen").unwrap();
    assert!(adapters.contains("http2case::Field1Value"));
    assert!(adapters.contains("http2case::field2value::Field1Entry"));
    syn::parse_file(&adapters).unwrap();
}
