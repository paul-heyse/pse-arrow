// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed configuration and a versioned, explicit semantic setting inventory.

use super::{ExecutionSettings, ThreadBudget};
use crate::CatalogError;
use datafusion::common::{
    DataFusionError,
    config::{ConfigEntry, ConfigExtension, ExtensionOptions},
};
use datafusion::execution::config::SessionConfig;
use pse_ids::{ContentHash, FramedHasher, derive::context};
use std::any::Any;
use std::collections::BTreeMap;

/// Contract-owned settings are readable but cannot independently override a kernel policy.
#[derive(Clone, Debug, Default)]
pub struct PseOptions;
impl ConfigExtension for PseOptions {
    const PREFIX: &'static str = "datafusion.pse";
}
impl ExtensionOptions for PseOptions {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn cloned(&self) -> Box<dyn ExtensionOptions> {
        Box::new(self.clone())
    }
    fn set(&mut self, key: &str, _value: &str) -> datafusion::common::Result<()> {
        Err(DataFusionError::Configuration(format!(
            "{key} is a view of the bound contract and cannot be overridden"
        )))
    }
    fn entries(&self) -> Vec<ConfigEntry> {
        vec![
            ConfigEntry {
                key: "datafusion.pse.null_policy".to_owned(),
                value: Some("four_valued".to_owned()),
                description: "Rule heads contain only decided-true rows.",
            },
            ConfigEntry {
                key: "datafusion.pse.kernel_outcome_policies".to_owned(),
                value: Some("[]".to_owned()),
                description: "Wave 1 has no executable numerical kernel outcomes.",
            },
        ]
    }
}

pub(crate) fn build(
    settings: &ExecutionSettings,
    budget: ThreadBudget,
) -> Result<SessionConfig, CatalogError> {
    budget.validate()?;
    if settings.spill_compression != "uncompressed" || settings.time_zone != "UTC" {
        return Err(invalid(
            "execution_settings",
            "Wave 1 requires uncompressed spill and UTC",
        ));
    }
    let mut config = SessionConfig::new();
    config.options_mut().extensions.insert(PseOptions);
    for (key, value) in [
        (
            "datafusion.execution.batch_size",
            settings.batch_size.to_string(),
        ),
        (
            "datafusion.execution.target_partitions",
            budget.target_partitions.to_string(),
        ),
        ("datafusion.execution.time_zone", settings.time_zone.clone()),
        (
            "datafusion.execution.spill_compression",
            settings.spill_compression.clone(),
        ),
        (
            "datafusion.execution.max_spill_file_size_bytes",
            settings.max_spill_file_size_bytes.to_string(),
        ),
        (
            "datafusion.execution.sort_spill_reservation_bytes",
            settings.sort_spill_reservation_bytes.to_string(),
        ),
        ("datafusion.execution.enable_ansi_mode", "false".to_owned()),
        (
            "datafusion.execution.skip_physical_aggregate_schema_check",
            "false".to_owned(),
        ),
        ("datafusion.catalog.information_schema", "true".to_owned()),
        (
            "datafusion.catalog.create_default_catalog_and_schema",
            "false".to_owned(),
        ),
        ("datafusion.catalog.default_catalog", "model".to_owned()),
        ("datafusion.catalog.default_schema", "authored".to_owned()),
        ("datafusion.explain.format", "pgjson".to_owned()),
    ] {
        config
            .options_mut()
            .set(key, &value)
            .map_err(|error| invalid(key, &error.to_string()))?;
    }
    Ok(config)
}

/// Select exact declared keys, retaining explicit absence as a semantic value.
/// # Errors
/// A library version lacks a required setting or violates an asserted engine mode.
pub fn semantic_settings(
    all: &BTreeMap<String, Option<String>>,
) -> Result<BTreeMap<String, Option<String>>, CatalogError> {
    let mut selected = BTreeMap::new();
    for key in SEMANTIC_KEYS {
        let value = all
            .get(*key)
            .ok_or_else(|| invalid(key, "required semantic setting is absent"))?;
        selected.insert((*key).to_owned(), value.clone());
    }
    for key in [
        "datafusion.execution.enable_ansi_mode",
        "datafusion.execution.skip_physical_aggregate_schema_check",
    ] {
        if selected.get(key).and_then(Option::as_deref) != Some("false") {
            return Err(invalid(key, "required mode is false"));
        }
    }
    Ok(selected)
}
/// Identity of the full exact key/value inventory, with absence distinct from text.
pub fn settings_hash(settings: &BTreeMap<String, Option<String>>) -> ContentHash {
    let mut hash = FramedHasher::new(context::SETTINGS);
    hash.str("pse.settings.allowlist.v1");
    for (key, value) in settings {
        hash.str(key);
        match value {
            Some(value) => {
                hash.part(&[1]);
                hash.str(value);
            }
            None => {
                hash.part(&[0]);
            }
        }
    }
    hash.finish_hash()
}
fn invalid(key: &str, reason: &str) -> CatalogError {
    CatalogError::ConfigInvalid {
        key: key.to_owned(),
        reason: reason.to_owned(),
    }
}

/// Explicit version-one allow-list; a pinned-library inventory test detects drift.
pub const SEMANTIC_KEYS: &[&str] = &[
    "datafusion.execution.enable_ansi_mode",
    "datafusion.execution.skip_physical_aggregate_schema_check",
    "datafusion.execution.time_zone",
    "datafusion.optimizer.allow_symmetric_joins_without_pruning",
    "datafusion.optimizer.default_filter_selectivity",
    "datafusion.optimizer.enable_aggregate_dynamic_filter_pushdown",
    "datafusion.optimizer.enable_distinct_aggregation_soft_limit",
    "datafusion.optimizer.enable_dynamic_filter_pushdown",
    "datafusion.optimizer.enable_join_dynamic_filter_pushdown",
    "datafusion.optimizer.enable_leaf_expression_pushdown",
    "datafusion.optimizer.enable_physical_uncorrelated_scalar_subquery",
    "datafusion.optimizer.enable_piecewise_merge_join",
    "datafusion.optimizer.enable_round_robin_repartition",
    "datafusion.optimizer.enable_sort_pushdown",
    "datafusion.optimizer.enable_topk_aggregation",
    "datafusion.optimizer.enable_topk_dynamic_filter_pushdown",
    "datafusion.optimizer.enable_topk_repartition",
    "datafusion.optimizer.enable_unions_to_filter",
    "datafusion.optimizer.enable_window_limits",
    "datafusion.optimizer.enable_window_topn",
    "datafusion.optimizer.expand_views_at_output",
    "datafusion.optimizer.filter_null_join_keys",
    "datafusion.optimizer.hash_join_inlist_pushdown_max_distinct_values",
    "datafusion.optimizer.hash_join_inlist_pushdown_max_size",
    "datafusion.optimizer.hash_join_single_partition_threshold",
    "datafusion.optimizer.hash_join_single_partition_threshold_rows",
    "datafusion.optimizer.join_reordering",
    "datafusion.optimizer.max_passes",
    "datafusion.optimizer.prefer_existing_sort",
    "datafusion.optimizer.prefer_existing_union",
    "datafusion.optimizer.prefer_hash_join",
    "datafusion.optimizer.preserve_file_partitions",
    "datafusion.optimizer.repartition_aggregations",
    "datafusion.optimizer.repartition_file_min_size",
    "datafusion.optimizer.repartition_file_scans",
    "datafusion.optimizer.repartition_joins",
    "datafusion.optimizer.repartition_sorts",
    "datafusion.optimizer.repartition_windows",
    "datafusion.optimizer.skip_failed_rules",
    "datafusion.optimizer.subset_repartition_threshold",
    "datafusion.optimizer.top_down_join_key_reordering",
    "datafusion.optimizer.use_statistics_registry",
    "datafusion.pse.kernel_outcome_policies",
    "datafusion.pse.null_policy",
    "datafusion.sql_parser.collect_spans",
    "datafusion.sql_parser.default_null_ordering",
    "datafusion.sql_parser.dialect",
    "datafusion.sql_parser.enable_ident_normalization",
    "datafusion.sql_parser.enable_options_value_normalization",
    "datafusion.sql_parser.enable_subquery_sort_elimination",
    "datafusion.sql_parser.map_string_types_to_utf8view",
    "datafusion.sql_parser.parse_float_as_decimal",
    "datafusion.sql_parser.recursion_limit",
    "datafusion.sql_parser.support_varchar_with_length",
];
