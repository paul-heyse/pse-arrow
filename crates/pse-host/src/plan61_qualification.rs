// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Test-only exact-release composition. No production caller uses this protocol.
#[path = "generated/plan61/mod.rs"]
#[rustfmt::skip]
#[allow(elided_lifetimes_in_paths, trivial_numeric_casts, unreachable_pub, missing_debug_implementations, clippy::upper_case_acronyms, reason = "Pinned protobuf generator output uses authored wire enum spellings and public items inside this test-only module")]
pub(crate) mod pb;
#[path = "generated/plan61/validation/mod.rs"]
#[rustfmt::skip]
#[allow(unused_qualifications, elided_lifetimes_in_paths, trivial_numeric_casts, unreachable_pub, reason = "Pinned validator generator emits explicit scalar casts and public packaging modules")]
mod validation;
#[test]
fn plan61_generated_validation_checks_owned_and_eager_views() {
    use crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::{
        __buffa::view::LocationView, Location,
    };
    use buffa::{Message as _, MessageView as _};
    use protovalidate_buffa::Validate as _;
    let valid = Location {
        path: b"source.rs".to_vec(),
        byte_start: Some(0),
        byte_end: Some(5),
        ..Default::default()
    };
    assert!(valid.validate().is_ok());
    let bytes = valid.encode_to_vec();
    assert!(
        LocationView::decode_view(&bytes)
            .unwrap()
            .validate()
            .is_ok()
    );
    let invalid = Location {
        byte_start: Some(10),
        byte_end: Some(5),
        ..Default::default()
    };
    let error = invalid.validate().unwrap_err();
    assert!(
        error
            .violations
            .iter()
            .any(|violation| violation.rule_id == "location.range")
    );
    assert!(error.compile_error.is_none() && error.runtime_error.is_none());
    let bytes = invalid.encode_to_vec();
    assert!(
        LocationView::decode_view(&bytes)
            .unwrap()
            .validate()
            .is_err()
    );
}

#[test]
fn plan61_composed_text_extents_execute_owned_and_eager_view_validators() {
    use buffa::{Message as _, MessageView as _};
    use pb::plan61::qualification::v1::{
        __buffa::view::TextExtentControlsView, TextExtentControls,
    };
    use protovalidate_buffa::Validate;

    fn check(value: &impl Validate, field: &str, expected: &[&str]) {
        let result = value.validate();
        if expected.is_empty() {
            assert!(result.is_ok(), "{field}: {result:?}");
            return;
        }
        let error = result.unwrap_err();
        assert!(error.compile_error.is_none() && error.runtime_error.is_none());
        assert_eq!(error.violations.len(), expected.len());
        let mut rules = error
            .violations
            .iter()
            .map(|violation| {
                assert_eq!(violation.field.to_string(), field);
                violation.rule_id.as_ref()
            })
            .collect::<Vec<_>>();
        rules.sort_unstable();
        let mut expected = expected.to_vec();
        expected.sort_unstable();
        assert_eq!(rules, expected);
    }

    let cases: &[(&str, &[&str])] = &[
        ("éa", &[]),
        ("éé", &[]),
        // One two-byte character violates both minimums; a three-byte character
        // isolates the Unicode scalar-value minimum from the byte minimum.
        ("é", &["string.min_len", "string.min_bytes"]),
        ("€", &["string.min_len"]),
        ("ab", &["string.min_bytes"]),
        ("你好", &["string.max_bytes"]),
        ("abcd", &["string.max_len"]),
        ("", &["string.min_len", "string.min_bytes"]),
    ];
    for &(text, expected) in cases {
        for (field, message) in [
            (
                "text",
                TextExtentControls {
                    text: text.into(),
                    optional_text: None,
                    ..Default::default()
                },
            ),
            (
                "optional_text",
                TextExtentControls {
                    text: "éa".into(),
                    optional_text: Some(text.into()),
                    ..Default::default()
                },
            ),
        ] {
            check(&message, field, expected);
            let bytes = message.encode_to_vec();
            let decoded = TextExtentControls::decode_from_slice(&bytes).unwrap();
            assert_eq!(decoded.text, message.text);
            assert_eq!(decoded.optional_text, message.optional_text);
            check(&decoded, field, expected);
            let view = TextExtentControlsView::decode_view(&bytes).unwrap();
            assert_eq!(view.text, message.text);
            assert_eq!(view.optional_text, message.optional_text.as_deref());
            check(&view, field, expected);
        }
    }
}

#[test]
fn plan61_existing_truth_count_and_raw_path_keep_their_meaning() {
    use crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::{
        Count, CountUnit, Location, PredicateFact, Truth, count::Precision,
    };
    use crate::investigation::{Count as NativeCount, Truth as NativeTruth};
    use buffa::Message as _;
    use std::os::unix::ffi::{OsStrExt as _, OsStringExt as _};
    let native_truth = NativeTruth::Unknown.negate();
    assert_eq!(native_truth, NativeTruth::Unknown);
    let fact = PredicateFact {
        truth: Truth::Unknown.into(),
        ..Default::default()
    };
    assert_eq!(
        PredicateFact::decode_from_slice(&fact.encode_to_vec())
            .unwrap()
            .truth,
        Truth::Unknown
    );
    let native_count = NativeCount::AtLeast(u64::MAX);
    let value = match native_count {
        NativeCount::AtLeast(value) => value,
        other => panic!("expected lower-bound count, got {other:?}"),
    };
    let count = Count {
        precision: Some(Precision::LowerBound(value)),
        unit: CountUnit::MatchUnit.into(),
        ..Default::default()
    };
    assert_eq!(
        Count::decode_from_slice(&count.encode_to_vec())
            .unwrap()
            .precision,
        Some(Precision::LowerBound(u64::MAX))
    );
    let native = std::path::PathBuf::from(std::ffi::OsString::from_vec(b"source-\xff.rs".to_vec()));
    let location = Location {
        path: native.as_os_str().as_bytes().to_vec(),
        ..Default::default()
    };
    assert_eq!(
        Location::decode_from_slice(&location.encode_to_vec())
            .unwrap()
            .path
            .as_slice(),
        native.as_os_str().as_bytes()
    );
}

#[test]
fn plan61_corrected_validator_division_reports_evaluation_error() {
    use pb::plan61::qualification::v1::UnsafeDivision;
    use protovalidate_buffa::Validate as _;
    let error = UnsafeDivision::default().validate().unwrap_err();
    assert!(error.runtime_error.is_some());
    assert!(error.compile_error.is_none());
    assert!(error.violations.is_empty());
}

#[test]
fn plan61_quantifier_errors_follow_cel_masking_rules() {
    use pb::plan61::qualification::v1::{
        ErrorExistsOne, MaskedAll, MaskedExists, RangeConstructionError,
    };
    use protovalidate_buffa::Validate as _;
    let false_result = MaskedAll::default().validate().unwrap_err();
    assert!(false_result.runtime_error.is_none());
    assert_eq!(false_result.violations.len(), 1);
    assert!(MaskedExists::default().validate().is_ok());
    assert!(
        ErrorExistsOne::default()
            .validate()
            .unwrap_err()
            .runtime_error
            .is_some()
    );
    assert!(
        RangeConstructionError::default()
            .validate()
            .unwrap_err()
            .runtime_error
            .is_some()
    );
}

#[test]
fn plan61_vtable_projection_borrows_owned_and_view_fields_directly() {
    use buffa::{Message as _, MessageView as _};
    use buffa_descriptor::reflect::{ReflectCow, ReflectMessage, Reflectable as _, ValueRef};
    use pb::plan61::qualification::v1::{__buffa::view::PayloadView, Payload};
    fn selected(message: &dyn ReflectMessage) -> (&str, bool) {
        let descriptor = message.message_descriptor();
        let field = descriptor.field_by_name("label").unwrap();
        let ValueRef::String(label) = message.get(field) else {
            panic!("label type changed")
        };
        (
            label,
            message.has(descriptor.field_by_name("count").unwrap()),
        )
    }
    let value = Payload {
        label: "selected declaration".into(),
        count: Some(0),
        ..Default::default()
    };
    assert!(matches!(value.reflect(), ReflectCow::Borrowed(_)));
    let (label, present) = selected(&value);
    assert_eq!(label.as_ptr(), value.label.as_ptr());
    assert!(present);
    let bytes = value.encode_to_vec();
    let view = PayloadView::decode_view(&bytes).unwrap();
    let (label, present) = selected(&view);
    assert_eq!(label, "selected declaration");
    assert!(present);
    let pointer = label.as_ptr() as usize;
    assert!((bytes.as_ptr() as usize..bytes.as_ptr() as usize + bytes.len()).contains(&pointer));
    assert!(!selected(&Payload::default()).1);
}
