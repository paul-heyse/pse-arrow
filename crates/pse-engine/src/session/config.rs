// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed configuration and a versioned, explicit semantic setting inventory.

use super::{ExecutionSettings, ThreadBudget};
use crate::EngineError;
use datafusion::execution::config::SessionConfig;
use pse_ids::{ContentHash, FramedHasher, derive::context};
use std::collections::BTreeMap;

pub(crate) fn build(
    settings: ExecutionSettings,
    budget: ThreadBudget,
) -> Result<SessionConfig, EngineError> {
    validate(&settings)?;
    let mut config = SessionConfig::new();
    config.options_mut().execution.spill_compression = settings.spill_compression;
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

/// Version of the explicit setting classification contract.
pub const CLASSIFICATION_VERSION: &str = "pse.settings.semantic.v3";

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

/// Read the complete actual native configuration and runtime limit inventory.
pub fn inventory(
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

/// One resolved native setting inventory, projected by its effect on reuse.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectiveSettings {
    /// Every actual native option, including explicit absence and runtime limits.
    pub inspection: BTreeMap<String, Option<String>>,
    /// Settings affecting values or planned semantics; unknown options remain here.
    pub semantic: BTreeMap<String, Option<String>>,
    /// Resource-only options, re-admitted for each invocation.
    pub operational: BTreeMap<String, Option<String>>,
}
impl EffectiveSettings {
    /// Project a complete native inventory without inventing missing values.
    /// # Errors
    /// The actual inventory is empty.
    pub fn from_inventory(
        inspection: BTreeMap<String, Option<String>>,
    ) -> Result<Self, EngineError> {
        if inspection.is_empty() {
            return Err(invalid(
                "session.settings",
                "configured setting inventory is empty",
            ));
        }
        let (operational, semantic) = inspection
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .partition(|(key, _)| is_resource_setting(key));
        Ok(Self {
            inspection,
            semantic,
            operational,
        })
    }
    /// Apply the already composed ancestor/invocation policy to the actual native state.
    /// Functions, planners, extensions and the deployment runtime are retained.
    /// # Errors
    /// Native option parsing, zero execution capacity, or invalid configuration.
    pub fn resolve(
        state: datafusion::execution::session_state::SessionState,
        settings: &BTreeMap<String, String>,
    ) -> Result<(datafusion::execution::session_state::SessionState, Self), EngineError> {
        let mut config = state.config().clone();
        for (key, value) in settings {
            config
                .options_mut()
                .set(key, value)
                .map_err(|error| invalid(key, &error.to_string()))?;
        }
        if config.options().execution.batch_size.get() == 0
            || config.options().execution.target_partitions == 0
        {
            return Err(invalid(
                "datafusion.execution",
                "batch size and target partitions must be positive",
            ));
        }
        let mut state = state;
        *state.config_mut() = config;
        let resolved = Self::from_inventory(inventory(&state))?;
        Ok((state, resolved))
    }
}

/// Retain semantic settings and explicit absence from the actual sealed session.
/// Deployment resource limits remain available through the complete native inventory.
/// # Errors
/// An empty inventory indicates that no engine configuration was captured.
pub fn semantic_settings(
    all: &BTreeMap<String, Option<String>>,
) -> Result<BTreeMap<String, Option<String>>, EngineError> {
    Ok(EffectiveSettings::from_inventory(all.clone())?.semantic)
}

/// Project the declared policy onto settings/effects that affect semantic reuse.
pub fn semantic_policy(
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

pub(super) fn validate(settings: &ExecutionSettings) -> Result<(), EngineError> {
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
    Ok(())
}
/// Identity of semantic settings only, with absence distinct from text (ADR-0070).
pub fn settings_hash(settings: &BTreeMap<String, Option<String>>) -> ContentHash {
    let mut hash = FramedHasher::new(context::SETTINGS);
    hash.str(CLASSIFICATION_VERSION);
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
fn invalid(key: &str, reason: &str) -> EngineError {
    EngineError::ConfigInvalid {
        key: key.to_owned(),
        reason: reason.to_owned(),
    }
}
