// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;

#[test]
fn composed_string_rules_keep_both_dimensions_in_either_order_and_optional_text() {
    projection_struct! {
        struct ComposedText {
            projection: { name: "ComposedText", version: 1, direction: Direction::Both,
                reserved_tags: [], reserved_names: [] };
            forward: String => { tag: 1, direction: Direction::Both,
                rules: [LocalRule::Length { min: 2, max: 3 }, LocalRule::Utf8Bytes { min: 3, max: 5 }] };
            reverse: String => { tag: 2, direction: Direction::Both,
                rules: [LocalRule::Utf8Bytes { min: 3, max: 5 }, LocalRule::Length { min: 2, max: 3 }] };
            optional_forward: Option<String> => { tag: 3, direction: Direction::Both,
                rules: [LocalRule::Length { min: 2, max: 3 }, LocalRule::Utf8Bytes { min: 3, max: 5 }] };
            optional_reverse: Option<String> => { tag: 4, direction: Direction::Both,
                rules: [LocalRule::Utf8Bytes { min: 3, max: 5 }, LocalRule::Length { min: 2, max: 3 }] };
            empty: Option<String> => { tag: 5, direction: Direction::Both,
                rules: [LocalRule::Utf8Bytes { min: 0, max: 0 }, LocalRule::Length { min: 0, max: 0 }] };
        }
    }
    let graph = ProjectionGraph::of::<ComposedText>().unwrap();
    let descriptors =
        descriptors_with_imports(&graph, "pse.text.v1", "text.proto", &imports()).unwrap();
    let owner = &descriptors
        .file
        .iter()
        .find(|file| file.name.as_deref() == Some("text.proto"))
        .unwrap()
        .message_type[0];
    for field in &owner.field {
        assert_eq!(
            field.proto3_optional.unwrap_or(false),
            field.number.unwrap() >= 3
        );
        let envelope = field
            .options
            .as_option()
            .unwrap()
            .extension(&validate::FIELD)
            .unwrap();
        assert!(!envelope.required.unwrap_or(false));
        let Some(validate::__buffa::oneof::field_rules::Type::String(rules)) = envelope.r#type
        else {
            panic!("composed rules retain actual string type");
        };
        if field.number == Some(5) {
            assert_eq!((rules.min_len, rules.max_len), (None, Some(0)));
            assert_eq!((rules.min_bytes, rules.max_bytes), (None, Some(0)));
        } else {
            assert_eq!((rules.min_len, rules.max_len), (Some(2), Some(3)));
            assert_eq!((rules.min_bytes, rules.max_bytes), (Some(3), Some(5)));
        }
    }
}
