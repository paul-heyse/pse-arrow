// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::projection_struct;

projection_struct! {
    struct OptionalName {
        projection: { name: "OptionalName", version: 1, direction: Direction::Both,
            reserved_tags: [], reserved_names: [] };
        name: Option<String> => { tag: 1, direction: Direction::Both,
            rules: [LocalRule::Utf8Bytes { min: 1, max: 2 }] };
    }
}

#[test]
fn optional_utf8_byte_limits_preserve_absence_empty_and_multibyte_distinctions() {
    ProjectionGraph::of::<OptionalName>().unwrap();
    for value in [None, Some("a"), Some("ab"), Some("é")] {
        OptionalName {
            name: value.map(str::to_owned),
        }
        .validate_projection()
        .unwrap();
    }
    for value in ["", "abc", "éa"] {
        assert!(
            OptionalName {
                name: Some(value.into())
            }
            .validate_projection()
            .is_err()
        );
    }
}

projection_struct! {
    struct ComposedNames {
        projection: { name: "ComposedNames", version: 1, direction: Direction::Both,
            reserved_tags: [], reserved_names: [] };
        name: String => { tag: 1, direction: Direction::Both,
            rules: [LocalRule::Length { min: 2, max: 3 }, LocalRule::Utf8Bytes { min: 3, max: 5 }] };
        optional: Option<String> => { tag: 2, direction: Direction::Both,
            rules: [LocalRule::Utf8Bytes { min: 3, max: 5 }, LocalRule::Length { min: 2, max: 3 }] };
    }
}

#[test]
fn composed_text_dimensions_validate_independently_with_optional_presence() {
    ProjectionGraph::of::<ComposedNames>().unwrap();
    for value in ["aé", "éa", "abc", "ééa", "e\u{301}"] {
        for optional in [None, Some(value.to_owned())] {
            ComposedNames {
                name: value.into(),
                optional,
            }
            .validate_projection()
            .unwrap();
        }
    }
    for (value, expected_rule) in [
        ("€", "length"),
        ("abcd", "length"),
        ("ab", "utf8_bytes"),
        ("€€", "utf8_bytes"),
    ] {
        let error = ComposedNames {
            name: value.into(),
            optional: None,
        }
        .validate_projection()
        .unwrap_err();
        assert!(matches!(error, ValidationError::Rule { path, rule, .. }
            if path == "name" && rule == expected_rule));
        let error = ComposedNames {
            name: "aé".into(),
            optional: Some(value.into()),
        }
        .validate_projection()
        .unwrap_err();
        assert!(matches!(error, ValidationError::Rule { path, rule, .. }
            if path == "optional" && rule == expected_rule));
    }
    for value in [None, Some(String::new())] {
        let value = ComposedNames {
            name: "aé".into(),
            optional: value,
        };
        assert_eq!(
            value.validate_projection().is_ok(),
            value.optional.is_none()
        );
    }
    let error = validate_field(
        &None::<String>,
        "optional",
        &[
            LocalRule::Required,
            LocalRule::Length { min: 2, max: 3 },
            LocalRule::Utf8Bytes { min: 3, max: 5 },
        ],
    )
    .unwrap_err();
    assert!(
        matches!(error, ValidationError::Rule { path, rule: "required", .. }
        if path == "optional")
    );
}

#[test]
fn composed_text_dimensions_still_reject_duplicate_and_unsupported_rules() {
    let length = LocalRule::Length { min: 2, max: 3 };
    let bytes = LocalRule::Utf8Bytes { min: 3, max: 5 };
    for ty in [
        TypeRef::Scalar(Scalar::String),
        TypeRef::Optional(Box::new(TypeRef::Scalar(Scalar::String))),
    ] {
        for rules in [
            vec![length, length],
            vec![bytes, bytes],
            vec![length, LocalRule::Length { min: 1, max: 4 }],
            vec![bytes, LocalRule::Utf8Bytes { min: 1, max: 8 }],
            vec![length, bytes, length],
            vec![bytes, length, bytes],
        ] {
            let field = Field {
                name: "name".into(),
                tag: 1,
                ty: ty.clone(),
                direction: Direction::Both,
                rules,
                named_facets: BTreeMap::new(),
            };
            let error = validate_fields("Owner", &[field], &BTreeMap::new()).unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("local rule family is declared more than once")
            );
        }
    }
    for (ty, rules) in [
        (
            TypeRef::Scalar(Scalar::String),
            vec![LocalRule::Length { min: 3, max: 2 }, bytes],
        ),
        (
            TypeRef::Scalar(Scalar::String),
            vec![length, LocalRule::Utf8Bytes { min: 5, max: 3 }],
        ),
        (TypeRef::Scalar(Scalar::Bytes), vec![length, bytes]),
        (
            TypeRef::Repeated(Box::new(TypeRef::Scalar(Scalar::String))),
            vec![length, bytes],
        ),
    ] {
        let field = Field {
            name: "name".into(),
            tag: 1,
            ty,
            direction: Direction::Both,
            rules,
            named_facets: BTreeMap::new(),
        };
        assert!(validate_fields("Owner", &[field], &BTreeMap::new()).is_err());
    }
}
