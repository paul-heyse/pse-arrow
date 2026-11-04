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
