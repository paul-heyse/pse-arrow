// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original protected scientific selection, independent of offline publication tooling.

pub use pse_model::recorded_result::{
    PackResultContext, PackResultPredicate, PackResultSelection, PackScientificState,
};

pub(super) fn context_from_read(
    read: &pse_operations::canonical_results::ResultRead,
) -> PackResultContext {
    PackResultContext {
        problem: read.run().problem.clone(),
        revision: read.run().revision.clone(),
        manifest: read.manifest().digest.clone(),
        interpretation: read.run().interpretation.clone(),
    }
}

/// One validated scalar-value or dense-time predicate. Endpoints retain their
/// original bits; comparison-only zero normalization never changes stored values.
#[derive(Clone, Copy, Debug)]
pub(super) struct ValuePredicate {
    dense: bool,
    minimum: Option<f64>,
    maximum: Option<f64>,
    missing: Option<bool>,
}
impl ValuePredicate {
    pub(super) fn new(
        dense: bool,
        minimum: Option<f64>,
        maximum: Option<f64>,
        missing: Option<bool>,
    ) -> Result<Self, super::WorkflowError> {
        if minimum.is_some_and(|value| !value.is_finite())
            || maximum.is_some_and(|value| !value.is_finite())
            || minimum
                .zip(maximum)
                .is_some_and(|(lower, upper)| lower > upper)
            || missing == Some(true) && (minimum.is_some() || maximum.is_some())
            || dense && missing.is_some()
        {
            return Err(super::contract(
                "invalid finite scientific output predicate",
            ));
        }
        Ok(Self {
            dense,
            minimum,
            maximum,
            missing,
        })
    }
    pub(super) fn dense(self) -> bool {
        self.dense
    }
    pub(super) fn minimum(self) -> Option<f64> {
        self.minimum
    }
    pub(super) fn maximum(self) -> Option<f64> {
        self.maximum
    }
    pub(super) fn missing(self) -> Option<bool> {
        self.missing
    }
    pub(super) fn matches_time(self, value: f64) -> bool {
        self.minimum.is_none_or(|lower| value >= lower)
            && self.maximum.is_none_or(|upper| value <= upper)
    }
    pub(super) fn scalar_expression(
        self,
        field: &str,
    ) -> Result<datafusion::logical_expr::Expr, super::WorkflowError> {
        use datafusion::logical_expr::{col, lit, when};
        let value = col(field);
        if self.missing == Some(true) {
            return Ok(value.is_null());
        }
        if self.missing.is_none() && self.minimum.is_none() && self.maximum.is_none() {
            return Ok(lit(true));
        }
        let mut predicate = value
            .clone()
            .gt(lit(f64::NEG_INFINITY))
            .and(value.clone().lt(lit(f64::INFINITY)));
        if self.minimum.is_some() || self.maximum.is_some() {
            let comparison = when(
                value.clone().eq(lit(0.0)).or(value.clone().eq(lit(-0.0))),
                lit(0.0),
            )
            .otherwise(value)
            .map_err(pse_engine::EngineError::from)?;
            if let Some(lower) = self.minimum {
                predicate = predicate.and(comparison.clone().gt_eq(lit(if lower == 0.0 {
                    0.0
                } else {
                    lower
                })));
            }
            if let Some(upper) = self.maximum {
                predicate =
                    predicate.and(comparison.lt_eq(lit(if upper == 0.0 { 0.0 } else { upper })));
            }
        }
        Ok(predicate)
    }
}

pub(super) const SELECTION_METADATA_KEY: &str = "pse.canonical.selection.proto.v1";
const SELECTION_METADATA_PREFIX: &str = "pse.canonical.selection.proto.";
const MAX_SELECTION_BYTES: u32 = 65_536;
const RETIRED_METADATA_KEYS: [&str; 17] = [
    "pse.canonical.run",
    "pse.canonical.attempt",
    "pse.canonical.relation",
    "pse.canonical.relation-version",
    "pse.canonical.row-start",
    "pse.canonical.row-end",
    "pse.canonical.terminal-class",
    "pse.canonical.field",
    "pse.canonical.output",
    "pse.canonical.partition",
    "pse.canonical.minimum-bits",
    "pse.canonical.maximum-bits",
    "pse.canonical.missing",
    "pse.canonical.problem",
    "pse.canonical.revision",
    "pse.canonical.manifest",
    "pse.canonical.interpretation",
];

fn metadata_error(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, error)
}
fn maximum_selection_text() -> std::io::Result<usize> {
    base64::encoded_len(MAX_SELECTION_BYTES as usize, true)
        .ok_or_else(|| metadata_error("selected-result text extent overflow"))
}
/// ADR-0250's exact local representation, not general Protobuf canonicalization.
/// Buffa checks its size/cache and budget before writing any encoded output.
pub(super) fn selection_metadata(selection: &PackResultSelection) -> std::io::Result<String> {
    use base64::Engine as _;
    use buffa::Message as _;
    if selection.context.is_none() {
        return Err(metadata_error(
            "versioned selected-result metadata requires original context",
        ));
    }
    let wire = crate::recorded_result_projection::encode_recorded_result_selection(selection)
        .map_err(metadata_error)?;
    let mut bytes = Vec::new();
    wire.try_encode_bounded(MAX_SELECTION_BYTES, &mut bytes)
        .map_err(metadata_error)?;
    let length = base64::encoded_len(bytes.len(), true)
        .ok_or_else(|| metadata_error("selected-result text extent overflow"))?;
    if length > maximum_selection_text()? {
        return Err(metadata_error(
            "selected-result metadata outside text allowance",
        ));
    }
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}
fn incompatible_selection_key(key: &str) -> bool {
    RETIRED_METADATA_KEYS.contains(&key)
        || key != SELECTION_METADATA_KEY && key.starts_with(SELECTION_METADATA_PREFIX)
}

/// New exports retain unrelated metadata and require a single current interpretation.
pub(super) fn publish_selection_metadata(
    metadata: &mut std::collections::HashMap<String, String>,
    selection: &PackResultSelection,
) -> std::io::Result<()> {
    if metadata
        .keys()
        .any(|key| incompatible_selection_key(key))
    {
        return Err(metadata_error(
            "unsupported inherited selected-result metadata interpretation",
        ));
    }
    let encoded = selection_metadata(selection)?;
    metadata.insert(SELECTION_METADATA_KEY.into(), encoded);
    Ok(())
}
/// Exact passive artifact/manifest agreement under the current interpretation.
pub(super) fn validate_selection_metadata(
    metadata: &std::collections::HashMap<String, String>,
    selection: &PackResultSelection,
) -> std::io::Result<()> {
    let encoded = metadata.get(SELECTION_METADATA_KEY)
        .ok_or_else(|| metadata_error("current selected-result metadata is required"))?;
    if encoded.len() > maximum_selection_text()?
        || metadata.keys().any(|key| incompatible_selection_key(key))
    {
        return Err(metadata_error(
            "selected-result metadata extent or mixed interpretations",
        ));
    }
    if encoded != &selection_metadata(selection)? {
        return Err(metadata_error(
            "IPC selection observation differs from original manifest",
        ));
    }
    Ok(())
}

/// DataFusion 55.1's bounded scientific expression uses finite comparisons,
/// owner keys, a comparison-only CASE for zero, and at most two bounds. Its
/// row arrays fit 64 bytes/row; separately cover a validity-combined bitmap,
/// worst-case u32 indices and slice/array/schema control headers before building.
pub(super) fn selection_window_extent(rows: usize, fields: usize, scalar: bool) -> Option<usize> {
    fields
        .checked_mul(4096)?
        .checked_add(4096)?
        .checked_add(rows.checked_mul(if scalar {
            64 + size_of::<u32>()
        } else {
            size_of::<u32>()
        })?)?
        .checked_add(rows.checked_add(7)?.checked_div(8)?.checked_add(63)? / 64 * 64)
}
/// Arrow 59.3 partition uses one boundary bitmap for the selected symbol column;
/// `ranges` grows a Vec to at most twice the row count. Seen-owner tree geometry
/// is owned by pse-model, and descriptor/string storage remains the index owner.
pub(super) fn partition_working_extent(rows: usize) -> Option<usize> {
    rows.checked_mul(2 * size_of::<std::ops::Range<usize>>())?
        .checked_add(rows.checked_add(7)? / 8 + 64)?
        .checked_add(pse_model::allocation::btree_allocation_bound::<
            pse_ids::SemanticId,
            (),
        >(rows)?)?
        .checked_add(4096)
}
/// DataFusion 55.1 pruning builds bounded native statistics cells and Arrow
/// wrappers per field/member. Decoded scalar/string bodies conservatively use
/// 16 times the encoded fact bytes; input/backing owners are separate.
pub(super) fn pruning_working_extent(
    fields: usize,
    members: usize,
    mut encoded: impl Iterator<Item = usize>,
) -> Option<usize> {
    let headers = fields
        .checked_mul(512)?
        .checked_add(2048)?
        .checked_mul(members)?;
    encoded.try_fold(headers, |bytes, encoded| {
        bytes.checked_add(encoded.checked_mul(16)?)
    })
}

/// Dense descriptors contain fixed-size hexadecimal identities, canonical keys,
/// field/partition names and wire metadata; 1024 bytes each cover those strings
/// alongside the native row. Partition scratch and container geometry coexist.
pub(super) fn trajectory_index_extent(rows: usize) -> Option<usize> {
    partition_working_extent(rows)?.checked_add(rows.checked_mul(
        size_of::<pse_model::generated::runtime::canonical_result_block_outputs::Row>() + 1024,
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn predicate_validation_keeps_zero_bits_dense_nan_behavior_and_missing_rules() {
        let zeros = ValuePredicate::new(true, Some(-0.), Some(0.), None).unwrap();
        assert_eq!(zeros.minimum().unwrap().to_bits(), (-0f64).to_bits());
        assert!(zeros.matches_time(-0.) && zeros.matches_time(0.));
        assert!(!zeros.matches_time(f64::NAN));
        assert!(
            ValuePredicate::new(true, None, None, None)
                .unwrap()
                .matches_time(f64::NAN)
        );
        assert!(ValuePredicate::new(true, None, None, Some(false)).is_err());
        for (minimum, maximum, missing) in [
            (Some(f64::NAN), None, None),
            (None, Some(f64::INFINITY), None),
            (Some(1.), Some(0.), None),
            (Some(0.), None, Some(true)),
        ] {
            assert!(ValuePredicate::new(false, minimum, maximum, missing).is_err());
        }
    }
    fn selected_observation() -> PackResultSelection {
        PackResultSelection {
            run: "run".into(),
            attempt: "attempt".into(),
            relation: "runtime.fit_parameters".into(),
            schema_version: 1,
            row_start: 4,
            row_end: 9,
            field: Some("value".into()),
            terminal_class: "converged".into(),
            predicate: Some(PackResultPredicate {
                output: "owner".into(),
                partition: "0".into(),
                minimum_bits: Some((-0f64).to_bits()),
                maximum_bits: Some(0),
                missing: Some(false),
            }),
            usability: PackScientificState::Unknown,
            context: Some(PackResultContext {
                problem: "problem".into(),
                revision: "revision".into(),
                manifest: "manifest".into(),
                interpretation: "original-policy".into(),
            }),
        }
    }
    #[test]
    fn versioned_selection_metadata_requires_exact_current_representation() {
        use base64::Engine as _;
        use buffa::Message as _;
        let selected = selected_observation();
        // Independent Buf encoding against the original 9eec0b873 engineering
        // descriptor, before moving source ownership or splitting its file.
        // This is a retained payload contract, not a descriptor identity assertion.
        const ORIGINAL_V1: &str = "CgNydW4SB2F0dGVtcHQaFnJ1bnRpbWUuZml0X3BhcmFtZXRlcnMgASgEMAk6BXZhbHVlQgljb252ZXJnZWRKGQoFb3duZXISATAYgICAgICAgICAASAAKABSAgoAWi4KB3Byb2JsZW0SCHJldmlzaW9uGghtYW5pZmVzdCIPb3JpZ2luYWwtcG9saWN5";
        assert_eq!(selection_metadata(&selected).unwrap(), ORIGINAL_V1);
        let mut metadata = std::collections::HashMap::from([
            ("pse.canonical.annotation".into(), "retained".into()),
        ]);
        assert!(validate_selection_metadata(&metadata, &selected).is_err());
        publish_selection_metadata(&mut metadata, &selected).unwrap();
        assert_eq!(metadata.len(), 2);
        assert_eq!(metadata["pse.canonical.annotation"], "retained");
        validate_selection_metadata(&metadata, &selected).unwrap();
        let original = metadata.clone();
        let encoded = metadata[SELECTION_METADATA_KEY].clone();
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&encoded)
            .unwrap();
        assert!(bytes.len() <= MAX_SELECTION_BYTES as usize);
        // Each alternate has the same selected known fields or an original valid
        // prefix. Version one promises the producer's exact local encoding.
        for suffix in [
            &[0x0a, 3, b'r', b'u', b'n'][..],
            &[0x98, 0x06, 1][..],
            &[0][..],
        ] {
            let mut alternative = bytes.clone();
            alternative.extend_from_slice(suffix);
            metadata.insert(
                SELECTION_METADATA_KEY.into(),
                base64::engine::general_purpose::STANDARD.encode(alternative),
            );
            assert!(validate_selection_metadata(&metadata, &selected).is_err());
        }
        for alternate in [
            format!(" {encoded}"),
            String::new(),
            "?".repeat(8),
            "A".repeat(maximum_selection_text().unwrap() + 1),
        ] {
            metadata.insert(SELECTION_METADATA_KEY.into(), alternate);
            assert!(validate_selection_metadata(&metadata, &selected).is_err());
        }
        metadata = original.clone();
        for key in RETIRED_METADATA_KEYS {
            metadata.insert(key.into(), "present".into());
            assert!(validate_selection_metadata(&metadata, &selected).is_err());
            assert!(publish_selection_metadata(&mut metadata, &selected).is_err());
            metadata.remove(key);
        }
        for coordinate in 0..4 {
            let mut changed = selected.clone();
            let context = changed.context.as_mut().unwrap();
            match coordinate {
                0 => context.problem.push('x'),
                1 => context.revision.push('x'),
                2 => context.manifest.push('x'),
                _ => context.interpretation.push('x'),
            }
            assert!(validate_selection_metadata(&metadata, &changed).is_err());
        }
        let mut missing_presence = selected.clone();
        missing_presence.predicate.as_mut().unwrap().minimum_bits = None;
        assert!(validate_selection_metadata(&metadata, &missing_presence).is_err());
        let mut absent_context = selected.clone();
        absent_context.context = None;
        assert!(selection_metadata(&absent_context).is_err());
        assert!(validate_selection_metadata(&metadata, &absent_context).is_err());
        let mut missing_context_wire = Vec::new();
        crate::recorded_result_projection::encode_recorded_result_selection(&absent_context)
            .unwrap()
            .try_encode_bounded(MAX_SELECTION_BYTES, &mut missing_context_wire)
            .unwrap();
        metadata.insert(
            SELECTION_METADATA_KEY.into(),
            base64::engine::general_purpose::STANDARD.encode(missing_context_wire),
        );
        assert!(
            validate_selection_metadata(&metadata, &absent_context).is_err(),
            "exact bytes cannot replace original context"
        );
        metadata = original.clone();
        let mut empty_field = selected.clone();
        empty_field.field = Some(String::new());
        let mut absent_field = empty_field.clone();
        absent_field.field = None;
        assert_ne!(
            selection_metadata(&empty_field).unwrap(),
            selection_metadata(&absent_field).unwrap()
        );
        let mut other_zero = selected.clone();
        other_zero.predicate.as_mut().unwrap().minimum_bits = Some(0);
        assert!(validate_selection_metadata(&metadata, &other_zero).is_err());
        metadata.insert("pse.canonical.selection.proto.v2".into(), "future".into());
        assert!(validate_selection_metadata(&metadata, &selected).is_err());
        assert!(publish_selection_metadata(&mut metadata, &selected).is_err());
        metadata.remove(SELECTION_METADATA_KEY);
        assert!(validate_selection_metadata(&metadata, &selected).is_err());
        let mut oversized = selected.clone();
        oversized.run = "x".repeat(MAX_SELECTION_BYTES as usize);
        assert!(selection_metadata(&oversized).is_err());
        metadata = original;
        let unchanged = metadata.clone();
        assert!(publish_selection_metadata(&mut metadata, &oversized).is_err());
        assert_eq!(metadata, unchanged);
    }
    #[test]
    fn recorded_selection_binary_reflection_preserves_extent_presence_and_original_context() {
        use base64::Engine as _;
        use buffa_descriptor::{DescriptorPool, DynamicMessage, Value};
        let mut selected = selected_observation();
        selected.row_start = u64::MAX - 1;
        selected.row_end = u64::MAX;
        selected.field = Some(String::new());
        selected.usability = PackScientificState::Unusable;
        let encoded = selection_metadata(&selected).unwrap();
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .unwrap();
        let world = std::sync::Arc::new(
            DescriptorPool::decode(crate::recorded_result_projection::DESCRIPTORS).unwrap(),
        );
        let message = world
            .message_index("pse.engineering.v1.RecordedResultSelection")
            .unwrap();
        let field = world.message(message).field(7).unwrap();
        assert_eq!(field.name(), "field");
        assert_eq!(field.json_name(), "field");
        let independent = DynamicMessage::decode(world, message, &bytes).unwrap();
        assert_eq!(
            independent.field_by_number(5),
            Some(&Value::U64(u64::MAX - 1))
        );
        assert_eq!(independent.field_by_number(6), Some(&Value::U64(u64::MAX)));
        assert_eq!(
            independent.field_by_number(7),
            Some(&Value::String(String::new()))
        );
        let Some(Value::Message(usability)) = independent.field_by_number(10) else {
            panic!("recorded usability must retain its source-owned alternative");
        };
        assert!(matches!(
            usability.field_by_number(5),
            Some(Value::Message(_))
        ));
        for alternative in 1..5 {
            assert_eq!(usability.field_by_number(alternative), None);
        }
        let Some(Value::Message(predicate)) = independent.field_by_number(9) else {
            panic!("original predicate missing");
        };
        assert_eq!(
            predicate.field_by_number(3),
            Some(&Value::U64((-0.0f64).to_bits()))
        );
        assert_eq!(predicate.field_by_number(4), Some(&Value::U64(0)));
        assert_eq!(predicate.field_by_number(5), Some(&Value::Bool(false)));
        let Some(Value::Message(context)) = independent.field_by_number(11) else {
            panic!("original context missing");
        };
        assert_eq!(
            context.field_by_number(3),
            Some(&Value::String("manifest".into()))
        );
        assert_eq!(
            context.field_by_number(4),
            Some(&Value::String("original-policy".into()))
        );
    }
    #[cfg(feature = "engineering-service")]
    #[test]
    fn recorded_selection_imported_world_keeps_native_callbacks_in_the_owning_pool() {
        use buffa::Message as _;
        use buffa_descriptor::{
            DynamicMessage, Value, ValueRef,
            reflect::{ReflectCow, Reflectable},
        };
        use std::sync::Arc;
        let wire = crate::recorded_result_projection::encode_recorded_result_selection(
            &selected_observation(),
        )
        .unwrap();
        let reflection = wire.reflect();
        assert!(matches!(reflection, ReflectCow::Borrowed(_)));
        let own = reflection.pool();
        let full =
            crate::host_operations::engineering::generated::pb::__buffa_fds::descriptor_pool();
        assert!(
            !Arc::ptr_eq(own, full),
            "complete early and full corpora have distinct pool identities"
        );
        let name = "pse.engineering.v1.RecordedResultSelection";
        assert!(own.message_index(name).is_some());
        let imported = full.message_index(name).unwrap();
        // Native getters receive only fields from their own message descriptor.
        let field = reflection.message_descriptor().field(5).unwrap();
        assert!(matches!(reflection.get(field), ValueRef::U64(4)));
        // The full world inspects an imported observation through its own dynamic
        // representation; it never forwards a same-FQN foreign field to native getters.
        let independent =
            DynamicMessage::decode(Arc::clone(full), imported, &wire.encode_to_vec()).unwrap();
        assert_eq!(independent.field_by_number(5), Some(&Value::U64(4)));
        assert!(Arc::ptr_eq(independent.pool(), full));
        assert!(!std::ptr::eq(
            field,
            full.message(imported).field(5).unwrap()
        ));
    }
    #[test]
    fn residual_bounds_refuse_overflow_and_scale_to_the_selected_window() {
        assert!(selection_window_extent(usize::MAX, 1, true).is_none());
        assert!(partition_working_extent(usize::MAX).is_none());
        assert!(trajectory_index_extent(usize::MAX).is_none());
        assert!(pruning_working_extent(usize::MAX, 2, [0].into_iter()).is_none());
        assert!(
            selection_window_extent(1, 7, true).unwrap()
                < selection_window_extent(8192, 7, true).unwrap()
        );
    }
}
