// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Controls for recursively declared binary64 containers and checked nominal scalars.

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
    assert!(!float_collections::supports_float64_domain(
        &root,
        graph.definitions()
    ));
    assert!(
        graph
            .array_container::<f64, 2>(&root, TypeRef::Scalar(Scalar::F64Bits))
            .is_err()
    );
    assert!(!float_collections::supports_float64_domain(
        &TypeRef::Repeated(Box::new(TypeRef::Scalar(Scalar::U64))),
        graph.definitions()
    ));
}

#[test]
fn source_nominal_float_domains_preserve_subtypes_and_strict_native_intervals() {
    const OPEN_UNIT: Float64Domain =
        Float64Domain::Interval(Float64Interval::new(0.0, false, 1.0, false));
    const SIGNED: Float64Domain =
        Float64Domain::Interval(Float64Interval::new(-1.0, true, 1.0, false));
    projection_struct! {
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        struct Confidence {
            projection:{name:"NominalConfidence",version:1,direction:Direction::Both,reserved_tags:[],reserved_names:[]};
            level: crate::scalars::Fraction => {tag:1,direction:Direction::Both,rules:[LocalRule::Float64Domain(OPEN_UNIT)]};
            offsets: Vec<Option<crate::scalars::FiniteBound>> => {tag:2,direction:Direction::Both,rules:[LocalRule::Float64Domain(SIGNED)]};
        }
    }
    let source = Confidence {
        level: crate::scalars::Fraction::try_new(0.5).unwrap(),
        offsets: vec![
            None,
            Some(crate::scalars::FiniteBound::try_new(-0.0).unwrap()),
            Some(crate::scalars::FiniteBound::try_new(-1.0).unwrap()),
        ],
    };
    let mut graph = ProjectionGraph::of::<Confidence>().unwrap();
    graph.register::<crate::scalars::PositiveCount>().unwrap();
    assert_eq!(
        graph.definitions()["Fraction"].native_container,
        Some(NativeContainer::NominalFloat64)
    );
    assert_eq!(graph.definitions()["PositiveCount"].native_container, None);
    source.validate_projection().unwrap();
    let bytes = serde_json::to_vec(&source).unwrap();
    let mut restored = Confidence::from_projection_fields(source.into_projection_fields()).unwrap();
    assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
    assert_eq!(
        restored.offsets[1].as_ref().unwrap().into_inner().to_bits(),
        (-0.0_f64).to_bits()
    );
    restored.level = crate::scalars::Fraction::try_new(1.0).unwrap();
    assert!(
        restored.validate_projection().is_err(),
        "the actual Fraction subtype remains (0,1] while this owner requires <1"
    );
    restored.level =
        crate::scalars::Fraction::try_new(f64::from_bits(1.0_f64.to_bits() - 1)).unwrap();
    restored.validate_projection().unwrap();
    restored
        .offsets
        .push(Some(crate::scalars::FiniteBound::try_new(1.0).unwrap()));
    assert!(restored.validate_projection().is_err());
}

#[test]
fn malformed_finite_intervals_refuse_before_graph_publication() {
    for interval in [
        Float64Interval::new(f64::NEG_INFINITY, true, 1.0, true),
        Float64Interval::new(0.0, true, f64::NAN, true),
        Float64Interval::new(1.0, false, 1.0, true),
        Float64Interval::new(1.0, false, f64::from_bits(1.0_f64.to_bits() + 1), false),
    ] {
        let mut graph = ProjectionGraph::default();
        assert!(
            graph
                .declare::<f64>(
                    "MalformedInterval",
                    file!(),
                    1,
                    Direction::Both,
                    &[],
                    &[],
                    |graph| Ok(Shape::Record(vec![Field::of::<f64>(
                        graph,
                        "value",
                        1,
                        Direction::Both,
                        vec![LocalRule::Float64Domain(Float64Domain::Interval(interval))]
                    )?]))
                )
                .is_err()
        );
        assert!(!interval.contains(0.5));
    }
}
