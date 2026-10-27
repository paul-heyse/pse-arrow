// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original protected scientific selection, independent of offline publication tooling.
use serde::{Deserialize, Serialize};

/// Exact original result lineage and range, including independent usability.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackResultSelection {
    /// Original canonical run identity; never a current-run alias.
    pub run: String,
    /// Exact original attempt identity; retries never substitute another attempt.
    pub attempt: String,
    /// Qualified original registry relation name.
    pub relation: String,
    /// Original registry relation schema version, independent of IPC transport encoding.
    pub schema_version: u32,
    /// Inclusive original row coordinate, before selected filtering.
    pub row_start: u64,
    /// Exclusive original row coordinate, before selected filtering.
    pub row_end: u64,
    /// Optional original field/output selection; schema remains the original typed relation.
    pub field: Option<String>,
    /// Original closed attempt classification, independent of publication.
    pub terminal_class: String,
    #[serde(default)]
    /// Exact original output predicate; absent for the whole selected relation.
    pub predicate: Option<PackResultPredicate>,
    /// Original scientific product usability, never upgraded by copying.
    pub usability: PackScientificState,
}
/// Exact protected output predicate. Floating endpoints retain IEEE bits, not JSON rounding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackResultPredicate {
    /// Original scientific output owner identity.
    pub output: String,
    /// Exact original selected output partition.
    pub partition: String,
    /// Optional lower predicate endpoint as exact IEEE-754 bits.
    pub minimum_bits: Option<u64>,
    /// Optional upper predicate endpoint as exact IEEE-754 bits.
    pub maximum_bits: Option<u64>,
    /// Original missing-value predicate, independent of empty output.
    pub missing: Option<bool>,
}
/// Scientific usability recorded by the producer; publication never upgrades it.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum PackScientificState {
    /// Original scientific usability was not established.
    #[default]
    Unknown,
    /// Original authority admitted this selected product as usable.
    Usable,
    /// Original selected scientific product has partial coverage.
    Partial,
    /// Original scientific work failed; export may still complete.
    Failed,
    /// Original product is retained but lacks scientific use permission.
    Unusable,
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
    pub(super) fn new(dense: bool, minimum: Option<f64>, maximum: Option<f64>, missing: Option<bool>) -> Result<Self, super::WorkflowError> {
        if minimum.is_some_and(|value| !value.is_finite())
            || maximum.is_some_and(|value| !value.is_finite())
            || minimum.zip(maximum).is_some_and(|(lower, upper)| lower > upper)
            || missing == Some(true) && (minimum.is_some() || maximum.is_some())
            || dense && missing.is_some() {
            return Err(super::contract("invalid finite scientific output predicate"));
        }
        Ok(Self { dense, minimum, maximum, missing })
    }
    pub(super) fn dense(self) -> bool { self.dense }
    pub(super) fn minimum(self) -> Option<f64> { self.minimum }
    pub(super) fn maximum(self) -> Option<f64> { self.maximum }
    pub(super) fn missing(self) -> Option<bool> { self.missing }
    pub(super) fn matches_time(self, value: f64) -> bool {
        self.minimum.is_none_or(|lower| value >= lower)
            && self.maximum.is_none_or(|upper| value <= upper)
    }
    pub(super) fn scalar_expression(self, field: &str) -> Result<datafusion::logical_expr::Expr, super::WorkflowError> {
        use datafusion::logical_expr::{col, lit, when};
        let value = col(field);
        if self.missing == Some(true) { return Ok(value.is_null()); }
        if self.missing.is_none() && self.minimum.is_none() && self.maximum.is_none() { return Ok(lit(true)); }
        let mut predicate = value.clone().gt(lit(f64::NEG_INFINITY)).and(value.clone().lt(lit(f64::INFINITY)));
        if self.minimum.is_some() || self.maximum.is_some() {
            let comparison = when(value.clone().eq(lit(0.0)).or(value.clone().eq(lit(-0.0))), lit(0.0))
                .otherwise(value).map_err(pse_engine::EngineError::from)?;
            if let Some(lower) = self.minimum { predicate = predicate.and(comparison.clone().gt_eq(lit(if lower == 0.0 { 0.0 } else { lower }))); }
            if let Some(upper) = self.maximum { predicate = predicate.and(comparison.lt_eq(lit(if upper == 0.0 { 0.0 } else { upper }))); }
        }
        Ok(predicate)
    }
}

/// Borrowed view of the canonical export metadata. It formats the same keys and
/// exact endpoint bits for publication and selected-lineage validation.
pub(super) struct ScientificMetadata<'a> {
    pub run: &'a str,
    pub attempt: &'a str,
    pub relation: &'a str,
    pub schema_version: u32,
    pub row_start: u64,
    pub row_end: u64,
    pub field: Option<&'a str>,
    pub terminal_class: &'a str,
    pub predicate: Option<PredicateMetadata<'a>>,
    pub extra: Option<ScientificMetadataExtra<'a>>,
}
pub(super) struct PredicateMetadata<'a> {
    pub output: &'a str,
    pub partition: &'a str,
    pub minimum_bits: Option<u64>,
    pub maximum_bits: Option<u64>,
    pub missing: Option<bool>,
}
pub(super) struct ScientificMetadataExtra<'a> {
    pub problem: &'a str,
    pub revision: &'a str,
    pub manifest: &'a str,
    pub interpretation: &'a str,
}
impl<'a> From<&'a PackResultSelection> for ScientificMetadata<'a> {
    fn from(value: &'a PackResultSelection) -> Self {
        Self { run: &value.run, attempt: &value.attempt, relation: &value.relation,
            schema_version: value.schema_version, row_start: value.row_start, row_end: value.row_end,
            field: value.field.as_deref(), terminal_class: &value.terminal_class,
            predicate: value.predicate.as_ref().map(|predicate| PredicateMetadata {
                output: &predicate.output, partition: &predicate.partition, minimum_bits: predicate.minimum_bits,
                maximum_bits: predicate.maximum_bits, missing: predicate.missing,
            }), extra: None }
    }
}
impl ScientificMetadata<'_> {
    pub(super) fn entries(&self) -> Vec<(&'static str, Option<String>)> {
        let mut out = Vec::with_capacity(17);
        out.extend([
            ("pse.canonical.run", Some(self.run.to_owned())),
            ("pse.canonical.attempt", Some(self.attempt.to_owned())),
            ("pse.canonical.relation", Some(self.relation.to_owned())),
            ("pse.canonical.relation-version", Some(self.schema_version.to_string())),
            ("pse.canonical.row-start", Some(self.row_start.to_string())),
            ("pse.canonical.row-end", Some(self.row_end.to_string())),
            ("pse.canonical.terminal-class", Some(self.terminal_class.to_owned())),
            ("pse.canonical.field", self.field.map(str::to_owned)),
        ]);
        if let Some(predicate) = &self.predicate {
            out.extend([
                ("pse.canonical.output", Some(predicate.output.to_owned())),
                ("pse.canonical.partition", Some(predicate.partition.to_owned())),
                ("pse.canonical.minimum-bits", predicate.minimum_bits.map(|bits| bits.to_string())),
                ("pse.canonical.maximum-bits", predicate.maximum_bits.map(|bits| bits.to_string())),
                ("pse.canonical.missing", predicate.missing.map(|missing| missing.to_string())),
            ]);
        } else { out.push(("pse.canonical.output", None)); }
        if let Some(extra) = &self.extra {
            out.extend([
                ("pse.canonical.problem", Some(extra.problem.to_owned())),
                ("pse.canonical.revision", Some(extra.revision.to_owned())),
                ("pse.canonical.manifest", Some(extra.manifest.to_owned())),
                ("pse.canonical.interpretation", Some(extra.interpretation.to_owned())),
            ]);
        }
        out
    }
}

/// DataFusion 55.1's bounded scientific expression uses finite comparisons,
/// owner keys, a comparison-only CASE for zero, and at most two bounds. Its
/// row arrays fit 64 bytes/row; separately cover a validity-combined bitmap,
/// worst-case u32 indices and slice/array/schema control headers before building.
pub(super) fn selection_window_extent(rows: usize, fields: usize, scalar: bool) -> Option<usize> {
    fields.checked_mul(4096)?.checked_add(4096)?
        .checked_add(rows.checked_mul(if scalar { 64 + size_of::<u32>() } else { size_of::<u32>() })?)?
        .checked_add(rows.checked_add(7)?.checked_div(8)?.checked_add(63)? / 64 * 64)
}
/// Arrow 59.3 partition uses one boundary bitmap for the selected symbol column;
/// `ranges` grows a Vec to at most twice the row count. Seen-owner tree geometry
/// is owned by pse-model, and descriptor/string storage remains the index owner.
pub(super) fn partition_working_extent(rows: usize) -> Option<usize> {
    rows.checked_mul(2 * size_of::<std::ops::Range<usize>>())?
        .checked_add(rows.checked_add(7)? / 8 + 64)?
        .checked_add(pse_model::allocation::btree_allocation_bound::<pse_ids::SemanticId, ()>(rows)?)?
        .checked_add(4096)
}
/// DataFusion 55.1 pruning builds bounded native statistics cells and Arrow
/// wrappers per field/member. Decoded scalar/string bodies conservatively use
/// 16 times the encoded fact bytes; input/backing owners are separate.
pub(super) fn pruning_working_extent(fields: usize, members: usize, mut encoded: impl Iterator<Item = usize>) -> Option<usize> {
    let headers = fields.checked_mul(512)?.checked_add(2048)?.checked_mul(members)?;
    encoded.try_fold(headers, |bytes, encoded| bytes.checked_add(encoded.checked_mul(16)?))
}

/// Dense descriptors contain fixed-size hexadecimal identities, canonical keys,
/// field/partition names and wire metadata; 1024 bytes each cover those strings
/// alongside the native row. Partition scratch and container geometry coexist.
pub(super) fn trajectory_index_extent(rows: usize) -> Option<usize> {
    partition_working_extent(rows)?.checked_add(rows.checked_mul(size_of::<pse_model::generated::runtime::canonical_result_block_outputs::Row>() + 1024)?)
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
        assert!(ValuePredicate::new(true, None, None, None).unwrap().matches_time(f64::NAN));
        assert!(ValuePredicate::new(true, None, None, Some(false)).is_err());
        for (minimum, maximum, missing) in [(Some(f64::NAN), None, None), (None, Some(f64::INFINITY), None), (Some(1.), Some(0.), None), (Some(0.), None, Some(true))] {
            assert!(ValuePredicate::new(false, minimum, maximum, missing).is_err());
        }
    }
    #[test]
    fn metadata_codec_preserves_declared_keys_and_exact_predicate_bits() {
        let selected = PackResultSelection { run: "run".into(), attempt: "attempt".into(), relation: "runtime.fit_parameters".into(), schema_version: 1,
            row_start: 4, row_end: 9, field: Some("value".into()), terminal_class: "converged".into(),
            predicate: Some(PackResultPredicate { output: "owner".into(), partition: "0".into(), minimum_bits: Some((-0f64).to_bits()), maximum_bits: Some(0), missing: Some(false) }),
            usability: PackScientificState::Unknown };
        let values = ScientificMetadata::from(&selected).entries().into_iter().filter_map(|(key, value)| value.map(|value| (key, value))).collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(values, [
            ("pse.canonical.run", "run"), ("pse.canonical.attempt", "attempt"), ("pse.canonical.relation", "runtime.fit_parameters"),
            ("pse.canonical.relation-version", "1"), ("pse.canonical.row-start", "4"), ("pse.canonical.row-end", "9"),
            ("pse.canonical.terminal-class", "converged"), ("pse.canonical.field", "value"), ("pse.canonical.output", "owner"),
            ("pse.canonical.partition", "0"), ("pse.canonical.minimum-bits", "9223372036854775808"), ("pse.canonical.maximum-bits", "0"), ("pse.canonical.missing", "false"),
        ].into_iter().map(|(key, value)| (key, value.to_owned())).collect());
    }
    #[test]
    fn residual_bounds_refuse_overflow_and_scale_to_the_selected_window() {
        assert!(selection_window_extent(usize::MAX, 1, true).is_none());
        assert!(partition_working_extent(usize::MAX).is_none());
        assert!(trajectory_index_extent(usize::MAX).is_none());
        assert!(pruning_working_extent(usize::MAX, 2, [0].into_iter()).is_none());
        assert!(selection_window_extent(1, 7, true).unwrap() < selection_window_extent(8192, 7, true).unwrap());
    }
}
