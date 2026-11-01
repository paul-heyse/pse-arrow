// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Test-only exact-release composition. No production caller uses this protocol.
#[path = "generated/plan61/mod.rs"]
#[rustfmt::skip]
#[allow(unused_qualifications, elided_lifetimes_in_paths, trivial_numeric_casts, unreachable_pub, missing_debug_implementations, reason = "Pinned protobuf generator output uses portable spellings and public items inside this test-only module")]
pub(crate) mod pb;
#[path = "generated/plan61/validation/mod.rs"]
#[rustfmt::skip]
#[allow(unused_qualifications, elided_lifetimes_in_paths, trivial_numeric_casts, unreachable_pub, reason = "Pinned validator generator emits explicit scalar casts and public packaging modules")]
mod validation;
#[rustfmt::skip]
#[allow(unused_qualifications, elided_lifetimes_in_paths, trivial_numeric_casts, dead_code, unreachable_pub, missing_debug_implementations, reason = "Pinned RPC generator emits a complete public client/server surface; this test consumes selected operations")]
pub(crate) mod services {
    include!("generated/plan61/services/plan61.qualification.v1.mod.rs");
}

#[test]
fn plan61_invalid_validator_rules_are_defects_not_user_violations() {
    use pb::plan61::qualification::v1::{CompileDefect, EvaluationDefect};
    use protovalidate_buffa::Validate as _;
    let invalid_rule = CompileDefect::default().validate().unwrap_err();
    // This release conflates parse/unsupported-rule defects into runtime_error.
    // Production admission must preflight rules rather than infer phase from this slot.
    assert!(invalid_rule.runtime_error.is_some());
    assert!(invalid_rule.compile_error.is_none());
    assert!(invalid_rule.violations.is_empty());
    let failed_evaluation = EvaluationDefect::default().validate().unwrap_err();
    assert!(failed_evaluation.runtime_error.is_some() || failed_evaluation.compile_error.is_some());
    assert!(failed_evaluation.violations.is_empty());
}

#[test]
fn plan61_generated_validation_checks_owned_and_eager_views() {
    use buffa::{Message as _, MessageView as _};
    use pb::plan61::qualification::v1::{__buffa::view::RequestView, Request};
    use protovalidate_buffa::Validate as _;
    let valid = Request {
        query: "source declaration".into(),
        limit: Some(5),
        ..Default::default()
    };
    assert!(valid.validate().is_ok());
    let bytes = valid.encode_to_vec();
    let view = RequestView::decode_view(&bytes).unwrap();
    assert!(view.validate().is_ok());
    let invalid = Request {
        query: String::new(),
        limit: Some(0),
        ..Default::default()
    };
    let error = invalid.validate().unwrap_err();
    assert!(!error.violations.is_empty());
    assert!(error.compile_error.is_none());
    assert!(error.runtime_error.is_none());
    let bytes = invalid.encode_to_vec();
    assert!(
        RequestView::decode_view(&bytes)
            .unwrap()
            .validate()
            .is_err()
    );
    let cel_only = Request {
        query: "x".into(),
        limit: Some(2),
        ..Default::default()
    };
    let errors = cel_only.validate().unwrap_err();
    assert!(errors.violations.iter().any(|v| v.rule_id == "query.limit"));
    assert!(errors.compile_error.is_none() && errors.runtime_error.is_none());
}

#[test]
fn plan61_existing_truth_count_and_raw_path_keep_their_meaning() {
    use crate::investigation::{Count, Truth};
    use buffa::Message as _;
    use pb::plan61::qualification::v1::{Payload, Precision};
    use std::os::unix::ffi::{OsStrExt as _, OsStringExt as _};
    let truth = Truth::Unknown.negate();
    assert_eq!(truth, Truth::Unknown);
    let count = Count::AtLeast(u64::MAX);
    let native = std::path::PathBuf::from(std::ffi::OsString::from_vec(b"source-\xff.rs".to_vec()));
    let mut message = Payload {
        path: native.as_os_str().as_bytes().into(),
        ..Default::default()
    };
    if let Count::AtLeast(value) = count {
        message.measured_count = Some(value);
        message.precision = Precision::LowerBound.into();
    }
    let observed = Payload::decode_from_slice(&message.encode_to_vec()).unwrap();
    assert_eq!(observed.measured_count, Some(u64::MAX));
    assert_eq!(observed.precision, Precision::LowerBound);
    assert_eq!(&observed.path[..], native.as_os_str().as_bytes());
}

#[test]
fn plan61_known_validator_division_panic_is_not_safe_rule_support() {
    use pb::plan61::qualification::v1::UnsafeDivision;
    use protovalidate_buffa::Validate as _;
    // The selected request contract does not admit numeric division. Preserve this
    // release limitation as a negative qualification rather than assuming CEL parity.
    assert!(std::panic::catch_unwind(|| UnsafeDivision::default().validate()).is_err());
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
