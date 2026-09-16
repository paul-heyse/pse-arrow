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

/// Retain every configured setting, including explicit absence. The inventory is
/// taken from the actual sealed session before any expression can be folded.
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
    Ok(all.clone())
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
/// Identity of the full exact key/value inventory, with absence distinct from text.
pub fn settings_hash(settings: &BTreeMap<String, Option<String>>) -> ContentHash {
    let mut hash = FramedHasher::new(context::SETTINGS);
    hash.str("pse.settings.complete.v2");
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
