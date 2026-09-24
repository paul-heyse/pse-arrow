// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit engine rule catalogs and reproducible profile identities.

use crate::EngineError;
use datafusion::arrow::ARROW_VERSION;
use datafusion::optimizer::{AnalyzerRule, OptimizerRule};
use datafusion::physical_optimizer::PhysicalOptimizerRule;
use pse_ids::{ContentHash, FramedHasher, derive::context};
use std::sync::Arc;

/// Ordered rule descriptions for the exact pinned execution pipeline.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineProfile {
    /// Profile contract version.
    pub version: String,
    /// Analyzer names, excluding the mandatory admission brackets.
    pub analyzer_rules: Vec<String>,
    /// Logical optimizer names.
    pub optimizer_rules: Vec<String>,
    /// Physical optimizer names.
    pub physical_optimizer_rules: Vec<String>,
}
/// The complete native pipeline of the pinned engine, in library-defined order.
/// This is a selected configuration, not a capability allow-list.
pub fn native_engine_profile() -> EngineProfile {
    EngineRules::native().profile("pse.engine.native.v1")
}
impl EngineProfile {
    /// Identify the exact versioned rule lists and read-back semantic settings.
    pub fn hash(&self, settings: ContentHash) -> ContentHash {
        let mut hash = FramedHasher::new(context::SETTINGS);
        hash.str(&self.version);
        hash.str(datafusion::DATAFUSION_VERSION);
        hash.str(ARROW_VERSION);
        for (kind, names) in [
            ("analyzer", &self.analyzer_rules),
            ("optimizer", &self.optimizer_rules),
            ("physical", &self.physical_optimizer_rules),
        ] {
            hash.str(kind);
            hash.u64(names.len() as u64);
            for name in names {
                hash.str(name);
            }
        }
        hash.part(settings.as_bytes());
        hash.finish_hash()
    }
}
/// Actual analyzer and optimizer objects frozen before preparation.
/// Custom rules use the same native traits as the built-in pipeline.
#[derive(Clone, Debug)]
pub struct EngineRules {
    /// Ordered semantic analyzers.
    pub analyzers: Vec<Arc<dyn AnalyzerRule + Send + Sync>>,
    /// Ordered logical optimizers.
    pub optimizers: Vec<Arc<dyn OptimizerRule + Send + Sync>>,
    /// Ordered physical optimizers, including distinct repeated rule instances.
    pub physical: Vec<Arc<dyn PhysicalOptimizerRule + Send + Sync>>,
}
impl EngineRules {
    /// Capture the pinned library's complete recommended pipeline.
    pub fn native() -> Self {
        Self {
            analyzers: datafusion::optimizer::analyzer::Analyzer::new().rules,
            optimizers: datafusion::optimizer::Optimizer::new().rules,
            physical: datafusion::physical_optimizer::optimizer::PhysicalOptimizer::new().rules,
        }
    }
    /// Describe these actual implementations for diagnostics and persisted context.
    pub fn profile(&self, version: &str) -> EngineProfile {
        EngineProfile {
            version: version.to_owned(),
            analyzer_rules: self
                .analyzers
                .iter()
                .map(|rule| rule.name().to_owned())
                .collect(),
            optimizer_rules: self
                .optimizers
                .iter()
                .map(|rule| rule.name().to_owned())
                .collect(),
            physical_optimizer_rules: self
                .physical
                .iter()
                .map(|rule| rule.name().to_owned())
                .collect(),
        }
    }
    /// Bind an explicit selection from the pinned native pipeline.
    /// Repeated names consume distinct instances in native order (for example,
    /// adding and removing output requirements are different implementations).
    /// # Errors
    /// A requested implementation is absent from this pinned native inventory.
    pub fn from_profile(profile: &EngineProfile) -> Result<Self, EngineError> {
        let native = Self::native();
        Ok(Self {
            analyzers: select(&profile.analyzer_rules, native.analyzers, |rule| {
                rule.name()
            })?,
            optimizers: select(&profile.optimizer_rules, native.optimizers, |rule| {
                rule.name()
            })?,
            physical: select(&profile.physical_optimizer_rules, native.physical, |rule| {
                rule.name()
            })?,
        })
    }
}
fn select<T>(
    names: &[String],
    mut available: Vec<T>,
    name: impl Fn(&T) -> &str,
) -> Result<Vec<T>, EngineError> {
    names.iter().map(|requested| {
        let index = available.iter().position(|rule| name(rule) == requested)
            .ok_or_else(|| EngineError::ConfigInvalid {
                key: "engine_profile.rules".to_owned(),
                reason: format!("native rule instance {requested} is absent; supply custom implementations through EngineRules"),
            })?;
        Ok(available.remove(index))
    }).collect()
}
