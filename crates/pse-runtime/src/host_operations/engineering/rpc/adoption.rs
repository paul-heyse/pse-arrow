// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Refuse unsupported native meaning without discarding retained future binary fields.

use buffa_descriptor::reflect::{ReflectMessage, ValueRef};
use pse_contract::boundary_projection::{AdapterError, AdapterFailure};

pub(super) fn validate<M: ReflectMessage + protovalidate_buffa::Validate>(
    message: &M,
) -> Result<(), connectrpc::ConnectError> {
    message
        .validate()
        .map_err(protovalidate_buffa::ValidationError::into_connect_error)?;
    if let Some(path) = unknown_message(message) {
        return Err(super::transport(super::EngineeringServiceError::Owner(
            Box::new(AdapterError::refused(
                path,
                AdapterFailure::UnknownField,
                "the binary value retains future fields which this native interpretation cannot adopt",
            )),
        )));
    }
    Ok(())
}

// Walk only actual set fields, borrowing vtable reflection; build a path only on
// refusal. Unknown bytes never become a guessed message or a JSON intermediate.
fn unknown_message(message: &dyn ReflectMessage) -> Option<String> {
    if !message.unknown_fields().is_empty() {
        return Some(String::new());
    }
    let mut unknown = None;
    message.for_each_set(&mut |field, value| {
        if unknown.is_none() {
            unknown = unknown_value(value).map(|path| {
                if path.is_empty() {
                    field.name().to_owned()
                } else if path.starts_with('[') {
                    format!("{}{path}", field.name())
                } else {
                    format!("{}.{path}", field.name())
                }
            });
        }
    });
    unknown
}

fn unknown_value(value: ValueRef<'_>) -> Option<String> {
    match value {
        ValueRef::Message(message) => unknown_message(&*message),
        ValueRef::List(values) => {
            for index in 0..values.len() {
                if let Some(value) = values.get(index)
                    && let Some(path) = unknown_value(value)
                {
                    return Some(if path.is_empty() {
                        format!("[{index}]")
                    } else {
                        format!("[{index}].{path}")
                    });
                }
            }
            None
        }
        ValueRef::Map(values) => {
            let mut unknown = None;
            values.for_each(&mut |key, value| {
                if unknown.is_none() {
                    unknown = unknown_value(value).map(|path| {
                        if path.is_empty() {
                            format!("[{key:?}]")
                        } else {
                            format!("[{key:?}].{path}")
                        }
                    });
                }
            });
            unknown
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "literal future-wire fields exercise actual Buffa retention"
    )]
    use super::super::pb;
    use super::*;
    use buffa::Message;

    fn future<M: Message>(message: M) -> M {
        let mut encoded = message.encode_to_vec();
        encoded.extend_from_slice(&[0xa0, 0x06, 0x01]); // Future tag100, varint1.
        M::decode_from_slice(&encoded).unwrap()
    }

    #[test]
    fn operation_extent_is_the_same_utf8_byte_rule_for_native_and_wire_owners() {
        use pse_contract::boundary_projection::ValidateProjection as _;
        for (count, accepted) in [(64, true), (65, false)] {
            let operation = "é".repeat(count);
            let native = pse_model::engineering::EngineeringOperationKey {
                operation: operation.clone(),
            };
            let wire = pb::EngineeringOperationKey {
                operation,
                ..Default::default()
            };
            assert_eq!(native.validate_projection().is_ok(), accepted);
            assert_eq!(validate(&wire).is_ok(), accepted);
        }
    }

    #[test]
    fn native_adoption_refuses_retained_future_fields_in_root_and_nested_repeated_messages() {
        let key = future(pb::EngineeringOperationKey {
            operation: "original".into(),
            ..Default::default()
        });
        assert!(!ReflectMessage::unknown_fields(&key).is_empty());
        assert!(validate(&key).is_err());
        assert!(
            crate::engineering_projection::decode_engineering_operation_key(key.clone()).is_err()
        );
        // A copied/reencoded binary value still carries the unsupported field.
        let copied = pb::EngineeringOperationKey::decode_from_slice(&key.encode_to_vec()).unwrap();
        assert!(!ReflectMessage::unknown_fields(&copied).is_empty());
        let package = pb::EngineeringPackage {
            files: [("package.toml".into(), b"source".to_vec())]
                .into_iter()
                .collect(),
            ..Default::default()
        };
        let request = pb::EngineeringAdmitSource {
            operation: "original".into(),
            physical: vec![future(package.clone())],
            modeling: vec![package],
            ..Default::default()
        };
        assert!(ReflectMessage::unknown_fields(&request).is_empty());
        assert_eq!(unknown_message(&request).as_deref(), Some("physical[0]"));
        assert!(validate(&request).is_err());
        assert!(crate::engineering_projection::decode_engineering_admit_source(request).is_err());
    }

    #[test]
    fn generated_service_adoption_requires_original_intent_and_exact_identity() {
        let original = pb::EngineeringPrepareCase {
            operation: "prepare-original".into(),
            source_operation: "source-original".into(),
            expected_local_revision: "exact-original-parent".into(),
            case: vec![7; 16],
            intent: pb::NativeSolveIntent {
                value: "root".into(),
                ..Default::default()
            }
            .into(),
            ..Default::default()
        };
        let native =
            crate::engineering_projection::decode_engineering_prepare_case(original.clone())
                .unwrap();
        assert_eq!(
            native.intent,
            pse_model::generated::enums::NativeSolveIntent::Root
        );
        assert_eq!(native.backend, None);
        assert_eq!(native.case.as_bytes(), &[7; 16]);
        let mut missing_intent = original.clone();
        missing_intent.intent = Default::default();
        let failure = super::super::study::projection_error(
            crate::engineering_projection::decode_engineering_prepare_case(missing_intent)
                .unwrap_err(),
        );
        assert_eq!(failure.code, connectrpc::ErrorCode::InvalidArgument);
        assert_eq!(
            failure
                .details
                .iter()
                .filter(|detail| detail.type_url == "buf.validate.Violations")
                .count(),
            1
        );
        assert!(
            failure
                .details
                .iter()
                .any(|detail| { detail.type_url == "pse.engineering.v1.EngineeringFailure" })
        );
        for changed in 0..3 {
            let mut request = original.clone();
            match changed {
                0 => request.intent = Default::default(),
                1 => {
                    request.intent = pb::NativeSolveIntent {
                        value: "future-intent".into(),
                        ..Default::default()
                    }
                    .into()
                }
                _ => request.case = vec![7; 15],
            }
            assert!(
                crate::engineering_projection::decode_engineering_prepare_case(request).is_err()
            );
        }
    }
}
