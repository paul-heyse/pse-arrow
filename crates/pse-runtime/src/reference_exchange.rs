// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded generated scientific interchange. This codec never constructs reference answers.
use crate::projection_support::*;
use buffa::Message as _;
use pse_model::{
    boundary_projection::{AdapterError, AdapterFailure},
    reference_exchange as native,
};

/// Source-generated messages and validators, with the complete imported descriptor world.
pub mod generated;
use generated::pb::pse::reference::v1 as pb;
#[path = "generated/reference-exchange/native-adapters.rs"]
mod native_projection;

/// Structural exchange refusal; physical comparison remains a separate operation.
#[derive(Debug, thiserror::Error)]
pub enum ExchangeError {
    /// A binary extent, decoding or encoding operation refused.
    #[error("scientific exchange: {0}")]
    Invalid(String),
    /// Original typed local-rule or mechanical-adoption refusal.
    #[error(transparent)]
    Projection(#[from] BoundaryProjectionError),
}
pse_diagnostics::impl_diagnostic! {
    ExchangeError,
    code(this) { match this { Self::Invalid(_) => Some(pse_diagnostics::DiagnosticCode::ValidationInvariant), Self::Projection(_) => None } },
    forward(this) { match this { Self::Projection(error) => Some(error), Self::Invalid(_) => None } }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}
fn invalid(error: impl std::fmt::Display) -> ExchangeError {
    ExchangeError::Invalid(error.to_string())
}

/// A checked known interpretation plus its complete original message, including unknown fields.
#[derive(Debug)]
pub struct DecodedReference {
    native: native::ReferenceEnvelope,
    original: pb::ReferenceEnvelope,
}
impl DecodedReference {
    /// Known versioned scientific metadata. This is not contextual scientific admission.
    pub fn known(&self) -> &native::ReferenceEnvelope {
        &self.native
    }
    /// Forward the original complete binary message without dropping unknown binary fields.
    /// # Errors
    /// An extent outside the caller's finite allowance refuses before any output is returned.
    pub fn forward(&self, maximum_bytes: usize) -> Result<Vec<u8>, ExchangeError> {
        encode_message(&self.original, maximum_bytes)
    }
}

/// Encode a source-owned scientific envelope, preserving scalar bits and presence.
/// # Errors
/// Shared local rules, generated validation and output extent must all hold.
pub fn encode(
    value: native::ReferenceEnvelope,
    maximum_bytes: usize,
) -> Result<Vec<u8>, ExchangeError> {
    let wire = native_projection::encode_reference_envelope(value)?;
    encode_message(&wire, maximum_bytes)
}
fn encode_message(
    value: &pb::ReferenceEnvelope,
    maximum_bytes: usize,
) -> Result<Vec<u8>, ExchangeError> {
    if maximum_bytes == 0 || value.try_encoded_len().map_err(invalid)? as usize > maximum_bytes {
        return Err(invalid(
            "scientific envelope exceeds the selected output extent",
        ));
    }
    value.try_encode_to_vec().map_err(invalid)
}

/// Decode with finite byte, element, recursion and unknown-field budgets.
/// # Errors
/// Malformed/oversized wire, unsupported version, local rules and absent alternatives refuse.
pub fn decode(bytes: &[u8], maximum_bytes: usize) -> Result<DecodedReference, ExchangeError> {
    if maximum_bytes == 0 || maximum_bytes > 16 << 20 {
        return Err(invalid(
            "select a scientific envelope allowance in 1..=16MiB",
        ));
    }
    let original: pb::ReferenceEnvelope = buffa::DecodeOptions::new()
        .with_max_message_size(maximum_bytes)
        .with_element_memory_limit(
            maximum_bytes
                .checked_mul(8)
                .ok_or_else(|| invalid("decode extent overflow"))?,
        )
        .with_recursion_limit(32)
        .with_unknown_field_limit(256)
        .decode_from_slice(bytes)
        .map_err(invalid)?;
    let native = native_projection::decode_reference_envelope(original.clone())?;
    Ok(DecodedReference { native, original })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_ids::{ContentHash, SemanticId};
    fn original() -> native::ReferenceEnvelope {
        native::ReferenceEnvelope {
            version: 1,
            meaning: native::ReferenceMeaning {
                case: SemanticId::from_bytes([1; 16]),
                source: ContentHash::from_bytes([2; 32]),
                formulation: ContentHash::from_bytes([3; 32]),
                physical: ContentHash::from_bytes([4; 32]),
                configuration: ContentHash::from_bytes([5; 32]),
            },
            required: vec![SemanticId::from_bytes([6; 16])],
            outcome: native::ReferenceOutcome::Completed {
                outputs: vec![native::ReferenceObservable {
                    id: SemanticId::from_bytes([6; 16]),
                    quantity: SemanticId::from_bytes([7; 16]),
                    unit: SemanticId::from_bytes([8; 16]),
                    basis: None,
                    datum: Some(SemanticId::NIL),
                    value: -0.0,
                }],
                blocks: vec![],
            },
        }
    }
    #[test]
    fn reference_wire_has_independently_specified_bits_presence_and_unknown_binary_forwarding() {
        let bytes = encode(original(), 1 << 20).unwrap();
        let wire = pb::ReferenceEnvelope::decode_from_slice(&bytes).unwrap();
        let pb::reference_outcome::Kind::Completed(value) = wire.outcome.kind.as_ref().unwrap()
        else {
            panic!("completed source alternative")
        };
        assert_eq!(
            value.outputs[0].value, 0x8000_0000_0000_0000,
            "literal IEEE-754 signed-zero payload"
        );
        assert_eq!(value.outputs[0].basis, None);
        assert_eq!(
            value.outputs[0].datum,
            Some(vec![0; 16]),
            "absent and explicit identity stay distinct"
        );
        assert_eq!(
            pb::ReferenceObservable {
                value: 0x8000_0000_0000_0000,
                ..Default::default()
            }
            .encode_to_vec(),
            [0x31, 0, 0, 0, 0, 0, 0, 0, 0x80],
            "independent tag6 fixed64 little-endian bytes"
        );
        let mut future = bytes;
        future.extend([0xc0, 0x3e, 0x2a]); // unknown field1000 varint42
        let decoded = decode(&future, 1 << 20).unwrap();
        let forwarded = decoded.forward(1 << 20).unwrap();
        let forwarded = pb::ReferenceEnvelope::decode_from_slice(&forwarded).unwrap();
        assert!(!forwarded.__buffa_unknown_fields.is_empty());
        let native::ReferenceOutcome::Completed { outputs, .. } = &decoded.known().outcome else {
            panic!("known interpretation")
        };
        assert_eq!(outputs[0].value.to_bits(), 0x8000_0000_0000_0000);
    }
    #[test]
    fn reference_native_adoption_keeps_nested_unknowns_in_the_retained_original() {
        let mut wire = native_projection::encode_reference_envelope(original()).unwrap();
        let pb::reference_outcome::Kind::Completed(completed) =
            wire.outcome.as_option_mut().unwrap().kind.as_mut().unwrap()
        else {
            panic!("completed source alternative")
        };
        let mut future_output = completed.outputs[0].encode_to_vec();
        future_output.extend([0xc0, 0x3e, 0x2a]);
        completed.outputs[0] = pb::ReferenceObservable::decode_from_slice(&future_output).unwrap();
        let decoded = decode(&wire.encode_to_vec(), 1 << 20).unwrap();
        let forwarded =
            pb::ReferenceEnvelope::decode_from_slice(&decoded.forward(1 << 20).unwrap()).unwrap();
        let pb::reference_outcome::Kind::Completed(completed) =
            forwarded.outcome.kind.as_ref().unwrap()
        else {
            panic!("retained alternative")
        };
        assert!(!completed.outputs[0].__buffa_unknown_fields.is_empty());
        let native::ReferenceOutcome::Completed { outputs, .. } = &decoded.known().outcome else {
            panic!("known interpretation")
        };
        assert_eq!(outputs[0].value.to_bits(), (-0.0_f64).to_bits());
    }
    #[test]
    fn reference_wire_refuses_missing_unknown_alternative_bad_version_and_extent_before_output() {
        let bytes = encode(original(), 1 << 20).unwrap();
        assert!(decode(&bytes, bytes.len() - 1).is_err());
        assert!(decode(&bytes[..bytes.len() - 1], 1 << 20).is_err());
        assert!(encode(original(), 1).is_err());
        let mut wire = native_projection::encode_reference_envelope(original()).unwrap();
        wire.version = 2;
        assert!(decode(&wire.encode_to_vec(), 1 << 20).is_err());
        wire.version = 1;
        wire.outcome = pb::ReferenceOutcome::default().into();
        assert!(decode(&wire.encode_to_vec(), 1 << 20).is_err());
        let unknown_alternative = [0xa2, 0x06, 0x00]; // field100 length-delimited empty
        let outcome = pb::ReferenceOutcome::decode_from_slice(&unknown_alternative).unwrap();
        wire.outcome = outcome.into();
        assert!(decode(&wire.encode_to_vec(), 1 << 20).is_err());
    }

    #[tokio::test]
    async fn native_buf_structural_checks_and_scientific_datum_controls_remain_distinct() {
        use pse_host::schema::{Bundle, compare};
        let original_schema = Bundle::from_bytes(
            "reference-baseline.bin".into(),
            generated::DESCRIPTORS.to_vec(),
        )
        .unwrap();
        let current_schema = Bundle::from_bytes(
            "reference-current.bin".into(),
            generated::DESCRIPTORS.to_vec(),
        )
        .unwrap();
        let classifications = compare::classify(&current_schema, &original_schema)
            .await
            .unwrap();
        assert_eq!(classifications.len(), 3);
        assert!(
            classifications
                .iter()
                .all(|classification| classification.outcome == "compatible"),
            "missing Buf or tool error fails selected compatibility work: {classifications:?}"
        );

        let reference = original();
        let mut incompatible = original();
        let native::ReferenceOutcome::Completed { outputs, .. } = &mut incompatible.outcome else {
            panic!("completed source")
        };
        outputs[0].datum = None;
        assert!(
            native::compare(
                &incompatible,
                &reference,
                &[native::ComparisonAllowance {
                    output: SemanticId::from_bytes([6; 16]),
                    absolute: 0.,
                    relative: 0.,
                }]
            )
            .is_err(),
            "wire compatibility does not establish scientific datum equivalence"
        );

        let mut changed =
            buffa_descriptor::generated::descriptor::FileDescriptorSet::decode_from_slice(
                generated::DESCRIPTORS,
            )
            .unwrap();
        let file = changed
            .file
            .iter_mut()
            .find(|file| file.package.as_deref() == Some("pse.reference.v1"))
            .unwrap();
        let message = file
            .message_type
            .iter_mut()
            .find(|message| message.name.as_deref() == Some("ReferenceEnvelope"))
            .unwrap();
        message
            .field
            .iter_mut()
            .find(|field| field.name.as_deref() == Some("outcome"))
            .unwrap()
            .number = Some(20);
        let changed_schema =
            Bundle::from_bytes("reference-breaking.bin".into(), changed.encode_to_vec()).unwrap();
        let classifications = compare::classify(&changed_schema, &original_schema)
            .await
            .unwrap();
        assert!(
            classifications
                .iter()
                .any(|classification| classification.category == "WIRE"
                    && classification.outcome == "breaking"),
            "independent Buf must detect the changed stable field tag: {classifications:?}"
        );
    }
}
