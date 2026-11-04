// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Unlinked controls for the recursively declared binary64 container capability.

use super::*;
use std::collections::BTreeMap;

projection_struct! {
    struct FloatingCollections {
        projection: { name: "FloatingCollections", version: 1, direction: Direction::Both,
            reserved_tags: [], reserved_names: [] };
        matrix: Option<Vec<Option<Vec<f64>>>> => { tag: 1, direction: Direction::Both,
            rules: [LocalRule::Float64Domain(Float64Domain::Finite)] };
        coordinates: BTreeMap<pse_ids::SemanticId, Option<Vec<[f64; 2]>>> => {
            tag: 2, direction: Direction::Both,
            rules: [LocalRule::Float64Domain(Float64Domain::Finite)] };
        windows: [Option<Vec<f64>>; 2] => { tag: 3, direction: Direction::Both,
            rules: [LocalRule::Float64Domain(Float64Domain::NonnegativeFinite)] };
    }
}

#[test]
fn float_collection_domains_preserve_absence_empty_values_and_both_signed_zeros() {
    let mut source = FloatingCollections {
        matrix: None,
        coordinates: BTreeMap::new(),
        windows: [None, Some(vec![])],
    };
    source.validate_projection().unwrap();
    source.matrix = Some(vec![
        None,
        Some(vec![]),
        Some(vec![0.0, -0.0, -f64::MAX, f64::MAX]),
    ]);
    let key = pse_ids::SemanticId::from_bytes([7; 16]);
    source.coordinates.insert(key, Some(vec![[0.0, -0.0]]));
    source.windows = [Some(vec![-0.0, 0.0]), None];
    let graph = ProjectionGraph::of::<FloatingCollections>().unwrap();
    assert!(graph.definitions().contains_key("FloatingCollections"));
    source.validate_projection().unwrap();
    assert_eq!(
        source.coordinates[&key].as_ref().unwrap()[0][1].to_bits(),
        (-0.0_f64).to_bits()
    );
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        source.matrix = Some(vec![Some(vec![invalid])]);
        assert!(source.validate_projection().is_err());
        source.matrix = None;
        source.coordinates.insert(key, Some(vec![[0.0, invalid]]));
        assert!(source.validate_projection().is_err());
        source.coordinates.insert(key, None);
        source.windows[0] = Some(vec![invalid]);
        assert!(source.validate_projection().is_err());
        source.windows[0] = None;
    }
    source.windows[0] = Some(vec![-f64::MIN_POSITIVE]);
    assert!(source.validate_projection().is_err());
}

#[test]
fn floating_domain_refuses_record_resemblance_and_non_binary64_leaves() {
    projection_struct! {
        struct NotAnArray {
            projection: { name: "NotAnArray", version: 1, direction: Direction::Both,
                reserved_tags: [], reserved_names: [] };
            values: Vec<f64> => { tag: 1, direction: Direction::Both, rules: [] };
        }
    }
    let mut graph = ProjectionGraph::of::<NotAnArray>().unwrap();
    let root = NotAnArray::projection(&mut graph).unwrap();
    assert!(!super::float_collections::supports_float64_domain(
        &root,
        graph.definitions()
    ));
    assert!(
        graph
            .array_container::<f64, 2>(&root, TypeRef::Scalar(Scalar::F64Bits))
            .is_err()
    );
    assert!(!super::float_collections::supports_float64_domain(
        &TypeRef::Repeated(Box::new(TypeRef::Scalar(Scalar::U64))),
        graph.definitions()
    ));
}
