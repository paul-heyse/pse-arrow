// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use pse_contract::boundary_projection::ValidateProjection;

#[test]
fn signed_i128_bits_keep_optional_presence_and_exact_width_validation() {
    projection_struct! {
        struct ExactInteger {
            projection:{name:"ExactInteger",version:1,direction:Direction::Both,reserved_tags:[],reserved_names:[]};
            integer:i128 => {tag:1,direction:Direction::Both,rules:[]};
            exact_integer:Option<i128> => {tag:2,direction:Direction::Both,rules:[]};
        }
    }
    for value in [i128::MIN, -1, 0, i128::from(u64::MAX) + 1, i128::MAX] {
        ExactInteger {
            integer: value,
            exact_integer: Some(value),
        }
        .validate_projection()
        .unwrap();
    }
    let graph = ProjectionGraph::of::<ExactInteger>().unwrap();
    let descriptors =
        descriptors_with_imports(&graph, "pse.exact.v1", "exact.proto", &imports()).unwrap();
    let owner = &descriptors
        .file
        .iter()
        .find(|file| file.name.as_deref() == Some("exact.proto"))
        .unwrap()
        .message_type[0];
    for field in &owner.field {
        assert_eq!(field.r#type, Some(Type::TYPE_BYTES));
        let envelope = field
            .options
            .as_option()
            .unwrap()
            .extension(&validate::FIELD)
            .unwrap();
        let Some(validate::__buffa::oneof::field_rules::Type::Bytes(rule)) = envelope.r#type else {
            panic!("fixed-width byte validation");
        };
        assert_eq!(rule.len, Some(16));
    }
    assert_eq!(owner.field[1].proto3_optional, Some(true));
    let adapter = native_adapters(&graph, &["ExactInteger"], "crate::wire", "pse_codegen").unwrap();
    syn::parse_file(&adapter).unwrap();
    assert!(adapter.contains("native_from::<_, Vec<u8>>"));
    assert!(adapter.contains("native_into::<_, Vec<u8>>"));
}

#[test]
fn signed_octet_keeps_actual_native_width_in_standard_sint32_rules() {
    projection_struct! {
        struct Transfer {
            projection:{name:"Transfer",version:1,direction:Direction::Both,reserved_tags:[],reserved_names:[]};
            factor:i8 => {tag:1,direction:Direction::Both,rules:[]};
        }
    }
    let graph = ProjectionGraph::of::<Transfer>().unwrap();
    let descriptors =
        descriptors_with_imports(&graph, "pse.exact.v1", "exact.proto", &imports()).unwrap();
    let field = &descriptors
        .file
        .iter()
        .find(|file| file.name.as_deref() == Some("exact.proto"))
        .unwrap()
        .message_type[0]
        .field[0];
    assert_eq!(field.r#type, Some(Type::TYPE_SINT32));
    let rules = field
        .options
        .as_option()
        .unwrap()
        .extension(&validate::FIELD)
        .unwrap();
    let Some(validate::__buffa::oneof::field_rules::Type::Sint32(rule)) = rules.r#type else {
        panic!("native signed octet width");
    };
    assert_eq!(
        rule.greater_than,
        Some(s_int32rules::GreaterThan::Gte(-128))
    );
    assert_eq!(rule.less_than, Some(s_int32rules::LessThan::Lte(127)));
}
