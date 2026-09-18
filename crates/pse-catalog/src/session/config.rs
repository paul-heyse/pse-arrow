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
                description: "Policies supplied by the frozen kernel context.",
            },
        ]
    }
}

pub(crate) fn build(
    settings: ExecutionSettings,
    budget: ThreadBudget,
) -> Result<SessionConfig, CatalogError> {
    validate(&settings)?;
    let mut config = SessionConfig::new();
    config.options_mut().extensions.insert(PseOptions);
    config.options_mut().execution.parquet.pushdown_filters = true;
    config
        .options_mut()
        .execution
        .parquet
        .max_predicate_cache_size = Some(0);
    for (key, value) in [
        (
            "datafusion.execution.batch_size",
            settings.batch_size.to_string(),
        ),
        (
            "datafusion.execution.target_partitions",
            budget.target_partitions.to_string(),
        ),
        ("datafusion.execution.time_zone", settings.time_zone),
        (
            "datafusion.execution.spill_compression",
            settings.spill_compression,
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

/// Known resource-only native options. Unknown extensions remain semantic inputs.
/// Resource admission always uses the full actual session, including these options.
#[must_use]
pub fn is_resource_setting(key: &str) -> bool {
    matches!(
        key,
        "datafusion.runtime.temp_directory"
            | "datafusion.runtime.memory_limit"
            | "datafusion.runtime.max_temp_directory_size"
            | "datafusion.runtime.max_spill_merge_fan_in"
            | "datafusion.runtime.metadata_cache_limit"
            | "datafusion.runtime.list_files_cache_limit"
            | "datafusion.runtime.list_files_cache_ttl"
            | "datafusion.runtime.file_statistics_cache_limit"
            | "datafusion.execution.spill_compression"
            | "datafusion.execution.max_spill_file_size_bytes"
            | "datafusion.execution.sort_spill_reservation_bytes"
            | "datafusion.execution.parquet.max_predicate_cache_size"
    )
}

pub(crate) fn inventory(
    state: &datafusion::execution::session_state::SessionState,
) -> BTreeMap<String, Option<String>> {
    state
        .config_options()
        .entries()
        .into_iter()
        .chain(state.runtime_env().config_entries())
        .map(|entry| (entry.key, entry.value))
        .collect()
}

/// Retain semantic settings and explicit absence from the actual sealed session.
/// Deployment resource limits remain available through the complete native inventory.
/// # Errors
/// An empty inventory indicates that no engine configuration was captured.
pub fn semantic_settings(
    all: &BTreeMap<String, Option<String>>,
) -> Result<BTreeMap<String, Option<String>>, CatalogError> {
    if all.is_empty() {
        return Err(invalid(
            "session.settings",
            "configured setting inventory is empty",
        ));
    }
    Ok(all
        .iter()
        .filter(|(key, _)| !is_resource_setting(key))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect())
}

pub(crate) fn semantic_policy(
    policy: &pse_schema::model::provider::ProviderPolicy,
) -> pse_schema::model::provider::ProviderPolicy {
    let mut result = policy.clone();
    result.max_bytes = None;
    result.defaults.retain(|key, _| !is_resource_setting(key));
    result
        .required_settings
        .retain(|key, _| !is_resource_setting(key));
    result
}

pub(super) fn policy_semantics_equal(
    a: &pse_schema::model::provider::ProviderPolicy,
    b: &pse_schema::model::provider::ProviderPolicy,
) -> bool {
    fn semantic(values: &BTreeMap<String, String>) -> impl Iterator<Item = (&String, &String)> {
        values.iter().filter(|(key, _)| !is_resource_setting(key))
    }
    a.id == b.id
        && a.scope == b.scope
        && a.requirements == b.requirements
        && a.effects == b.effects
        && semantic(&a.defaults).eq(semantic(&b.defaults))
        && semantic(&a.required_settings).eq(semantic(&b.required_settings))
}

pub(super) fn validate(settings: &ExecutionSettings) -> Result<(), CatalogError> {
    if settings.batch_size == 0 {
        return Err(invalid(
            "datafusion.execution.batch_size",
            "must be positive",
        ));
    }
    settings
        .time_zone
        .parse::<datafusion::arrow::array::timezone::Tz>()
        .map_err(|error| invalid("datafusion.execution.time_zone", &error.to_string()))?;
    let mut options = datafusion::common::config::ConfigOptions::new();
    for (key, value) in [
        (
            "datafusion.execution.spill_compression",
            settings.spill_compression.as_str(),
        ),
        (
            "datafusion.execution.time_zone",
            settings.time_zone.as_str(),
        ),
    ] {
        options
            .set(key, value)
            .map_err(|error| invalid(key, &error.to_string()))?;
    }
    Ok(())
}
/// Identity of semantic settings only, with absence distinct from text (ADR-0070).
pub fn settings_hash(settings: &BTreeMap<String, Option<String>>) -> ContentHash {
    let mut hash = FramedHasher::new(context::SETTINGS);
    hash.str("pse.settings.semantic.v3");
    for (key, value) in settings.iter().filter(|(key, _)| !is_resource_setting(key)) {
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

#[cfg(test)]
mod tests;
fn invalid(key: &str, reason: &str) -> CatalogError {
    CatalogError::ConfigInvalid {
        key: key.to_owned(),
        reason: reason.to_owned(),
    }
}
