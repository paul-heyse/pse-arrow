// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;

projection_enum! {
    #[derive(Debug)]
    enum Recursive {
        projection: { name: "Recursive", version: 1, direction: Direction::Both, reserved_tags: [9], reserved_names: ["Removed"] };
        Missing => 1;
        Number(f64) => 2;
        Rows(Vec<Recursive>) => 4;
        Record {
            id: pse_ids::SemanticId => { tag: 1, direction: Direction::Both, rules: [] };
            next: Option<Box<Recursive>> => { tag: 2, direction: Direction::Both, rules: [] };
        } => 7;
    }
}

projection_struct! {
    #[derive(Debug)]
    struct Request {
        projection: { name: "Request", version: 2, direction: Direction::Input, reserved_tags: [3], reserved_names: ["old"] };
        limit: u32 => { tag: 7, direction: Direction::Both, rules: [LocalRule::UnsignedRange { min: 1, max: 128 }] };
        nested: Option<Vec<Option<Recursive>>> => { tag: 2, direction: Direction::Input, rules: [] };
    }
}

#[test]
fn actual_declarations_resolve_recursive_fields_and_explicit_identity() {
    let graph = ProjectionGraph::of::<Request>().unwrap();
    assert_eq!(graph.definitions().len(), 2);
    let request = &graph.definitions()["Request"];
    assert_eq!(request.version, 2);
    assert_eq!(request.reserved_tags, [3]);
    let Shape::Record(fields) = &request.shape else {
        panic!("record")
    };
    assert_eq!(fields[0].tag, 7);
    assert_eq!(
        fields[1].ty,
        TypeRef::Optional(Box::new(TypeRef::Repeated(Box::new(TypeRef::Optional(
            Box::new(TypeRef::Named("Recursive".into()))
        )))))
    );
    let Shape::Alternatives(alternatives) = &graph.definitions()["Recursive"].shape else {
        panic!("enum")
    };
    assert_eq!(
        alternatives[1].fields[0].ty,
        TypeRef::Scalar(Scalar::F64Bits)
    );
    assert_eq!(
        alternatives[3].fields[0].ty,
        TypeRef::Scalar(Scalar::SemanticId)
    );
    // These constructors are the actual Rust owner, not a parallel metadata catalogue.
    let value = Request {
        limit: 128,
        nested: Some(vec![Some(Recursive::Rows(vec![
            Recursive::Number(-0.0),
            Recursive::Missing,
            Recursive::Record {
                id: pse_ids::SemanticId::from_bytes([0; 16]),
                next: None,
            },
        ]))]),
    };
    assert_eq!(value.limit, 128);
    assert_eq!(value.nested.as_ref().unwrap().len(), 1);
}

#[test]
fn duplicate_reserved_and_invalid_tags_refuse() {
    for (tags, reserved) in [
        (vec![1, 1], vec![]),
        (vec![1, 2], vec![2]),
        (vec![0, 2], vec![]),
        (vec![19_000, 2], vec![]),
    ] {
        let mut graph = ProjectionGraph::default();
        let result = graph.declare::<Request>(
            "Broken",
            file!(),
            1,
            Direction::Both,
            &reserved,
            &[],
            |_| {
                Ok(Shape::Record(
                    tags.iter()
                        .enumerate()
                        .map(|(i, tag)| Field {
                            name: format!("f{i}"),
                            tag: *tag,
                            ty: TypeRef::Scalar(Scalar::U32),
                            direction: Direction::Both,
                            rules: vec![],
                        })
                        .collect(),
                ))
            },
        );
        assert!(result.is_err(), "{tags:?}/{reserved:?}");
    }
}

#[test]
fn public_name_collisions_and_wrong_type_rules_refuse() {
    let mut graph = ProjectionGraph::of::<Request>().unwrap();
    let collision =
        graph.declare::<Recursive>("Request", file!(), 1, Direction::Both, &[], &[], |_| {
            Ok(Shape::Record(vec![]))
        });
    assert!(collision.is_err());
    let bad_rule =
        graph.declare::<Recursive>("BadRule", file!(), 1, Direction::Both, &[], &[], |_| {
            Ok(Shape::Record(vec![Field {
                name: "number".into(),
                tag: 1,
                ty: TypeRef::Scalar(Scalar::F64Bits),
                direction: Direction::Both,
                rules: vec![LocalRule::UnsignedRange { min: 0, max: 1 }],
            }]))
        });
    assert!(bad_rule.is_err());
}

#[test]
fn multiple_roots_share_one_complete_recursive_world() {
    let mut graph = ProjectionGraph::of::<Request>().unwrap();
    graph.register::<Recursive>().unwrap();
    graph.register::<Request>().unwrap();
    assert_eq!(graph.roots().len(), 2);
    assert_eq!(graph.definitions().len(), 2);
}

#[test]
fn one_owner_rule_validates_native_values_with_exact_field_paths() {
    let invalid = Request {
        limit: 0,
        nested: None,
    };
    assert_eq!(
        invalid.validate_projection(),
        Err(ValidationError::Rule {
            path: "limit".into(),
            rule: "unsigned.range",
            reason: "expected inclusive range 1..=128".into(),
        })
    );
    Request {
        limit: 128,
        nested: Some(vec![Some(Recursive::Missing)]),
    }
    .validate_projection()
    .unwrap();
    assert!(
        validate_field(
            &u64::MAX,
            "integer",
            &[LocalRule::UnsignedRange {
                min: u64::MAX,
                max: u64::MAX
            }]
        )
        .is_ok()
    );
    assert!(
        validate_field(
            &u64::MAX,
            "integer",
            &[LocalRule::UnsignedRange {
                min: 0,
                max: u64::MAX - 1
            }]
        )
        .is_err()
    );
    validate_field(
        &"é𐐷".to_owned(),
        "text",
        &[LocalRule::Length { min: 2, max: 2 }],
    )
    .unwrap();
    assert!(validate_field(&None::<u32>, "optional", &[LocalRule::Required]).is_err());
}
