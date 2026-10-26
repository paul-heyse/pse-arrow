// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Closed payload placement; descriptor discovery never carries encoded bodies.

use super::{CanonicalError, ResultBatch};
use pse_model::generated::enums::CanonicalPayloadPlacement;

/// One actual payload owner and its member coordinate, with no synthetic file identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PayloadLocator {
    /// Canonical immutable encoded body, shared by its native members.
    Inline {
        /// Original immutable encoded body.
        key: String,
        /// Native member coordinate.
        index: u64,
    },
    /// Original sealed physical instance and native member coordinate.
    Sealed {
        /// Actual physical instance.
        instance: String,
        /// Native member coordinate.
        index: u64,
    },
}

impl PayloadLocator {
    /// Exact body or file identity used by the statistics binding.
    pub fn payload_identity(&self) -> &str {
        match self { Self::Inline { key, .. } => key, Self::Sealed { instance, .. } => instance }
    }
    /// Member coordinate within the complete admitted encoding.
    pub fn index(&self) -> u64 {
        match self { Self::Inline { index, .. } | Self::Sealed { index, .. } => *index }
    }
}

/// Validate exactly one declared placement arm before any physical or body access.
pub fn result_payload_locator(batch: &ResultBatch) -> Result<PayloadLocator, CanonicalError> {
    match (batch.placement, batch.inline_key.as_deref(), batch.file_instance.as_deref()) {
        (CanonicalPayloadPlacement::Inline, Some(key), None)
            if key.len() == 64 && key.bytes().all(|byte| byte.is_ascii_hexdigit()) =>
            Ok(PayloadLocator::Inline { key: key.into(), index: batch.payload_index }),
        (CanonicalPayloadPlacement::Sealed, None, Some(instance))
            if uuid::Uuid::parse_str(instance).is_ok() =>
            Ok(PayloadLocator::Sealed { instance: instance.into(), index: batch.payload_index }),
        _ => Err(CanonicalError::Configuration("result payload placement arms or identity differ".into())),
    }
}

#[cfg(test)]
mod canonical_payload_unit {
    #![allow(clippy::unwrap_used, clippy::unreachable, reason = "pure payload placement controls assert exact owner facts and encoded envelope shapes")]
    use super::*;
    use super::super::{AttemptFence, ResultPayload, ResultBlockAppend, result_batch_key, result_set_key, validate_result_batch, payload_binding};

    fn descriptor() -> ResultBatch {
        let digest = super::super::result_payload_digest(b"body");
        let mut batch = ResultBatch {
            key: result_batch_key("attempt", "set", 0), attempt: "attempt".into(), result_set: "set".into(), ordinal: 0,
            digest, row_count: 1, placement: CanonicalPayloadPlacement::Inline,
            inline_key: Some("a".repeat(64)), file_instance: None, payload_index: 0,
            payload_binding: String::new(), payload_bytes: 4, schema: String::new(), encoding: crate::canonical_files::OPAQUE_BYTES.into(),
        };
        batch.payload_binding = payload_binding(&batch);
        batch
    }

    #[test]
    fn locator_rejects_missing_dual_and_mismatched_arms() {
        let batch = descriptor();
        assert!(matches!(result_payload_locator(&batch).unwrap(), PayloadLocator::Inline { index: 0, .. }));
        let mut missing = batch.clone(); missing.inline_key = None;
        assert!(result_payload_locator(&missing).is_err());
        let mut dual = batch.clone(); dual.file_instance = Some(uuid::Uuid::new_v4().to_string());
        assert!(result_payload_locator(&dual).is_err());
        let mut wrong = batch; wrong.placement = CanonicalPayloadPlacement::Sealed;
        assert!(result_payload_locator(&wrong).is_err());
    }

    #[test]
    fn inline_extent_and_payload_binding_are_checked_before_hydration() {
        let mut batch = descriptor(); validate_result_batch(&batch).unwrap();
        batch.payload_bytes = pse_schema::catalog::substrate::RESULT_INLINE_BYTES as u64 + 1;
        batch.payload_binding = payload_binding(&batch);
        assert!(validate_result_batch(&batch).is_err());
        let mut changed = descriptor(); changed.inline_key = Some("b".repeat(64));
        assert!(validate_result_batch(&changed).is_err());
        let mut invalid = descriptor(); invalid.encoding = "unknown-codec".into();
        invalid.payload_binding = payload_binding(&invalid);
        assert!(validate_result_batch(&invalid).is_err());
        let mut wrong_schema = descriptor(); wrong_schema.schema = "0".repeat(32);
        wrong_schema.payload_binding = payload_binding(&wrong_schema);
        assert!(validate_result_batch(&wrong_schema).is_err());
    }

    #[test]
    fn complete_encoding_threshold_selects_before_any_file_work() {
        assert!(ResultPayload::new(Vec::new()).is_err());
        assert!(ResultPayload::new_owned(Vec::new(), std::sync::Arc::new(())).is_err());
        let fence = AttemptFence { run: "run".into(), attempt: "attempt".into(), generation: 1 };
        for extent in [pse_schema::catalog::substrate::RESULT_INLINE_BYTES, pse_schema::catalog::substrate::RESULT_INLINE_BYTES + 1] {
            let payload = ResultPayload::new(vec![7; extent]).unwrap();
            let prepared = ResultBlockAppend::prepare(&fence, "body-operation", "opaque", 0, payload, 1, None, &[], None).unwrap();
            let Some(surrealdb::types::Value::Object(batch)) = prepared.item.get("batch") else { unreachable!() };
            let batch = crate::generated::surreal::decode_canonical_result_batches(batch.clone()).unwrap();
            assert_eq!(batch.placement == CanonicalPayloadPlacement::Inline, extent <= pse_schema::catalog::substrate::RESULT_INLINE_BYTES);
            assert!(batch.file_instance.is_none());
            assert_eq!(batch.result_set, result_set_key(fence.attempt(), "opaque"));
        }
    }
}
