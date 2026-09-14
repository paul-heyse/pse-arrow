// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit engine rule catalogs and reproducible profile identities.

use crate::CatalogError;
use datafusion::arrow::ARROW_VERSION;
use datafusion::optimizer::{AnalyzerRule, OptimizerRule};
use datafusion::physical_optimizer::PhysicalOptimizerRule;
use pse_ids::{ContentHash, FramedHasher, derive::context};
use std::sync::Arc;

/// Ordered engine rules are semantic input, never read from evolving engine defaults.
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
/// A small explicit profile supporting the Wave 1 relational algebra.
pub fn phase0_reference_profile() -> EngineProfile {
    EngineProfile {
        version: "pse.engine.wave1.v1".to_owned(),
        analyzer_rules: vec![
            "type_coercion".to_owned(),
            "resolve_grouping_function".to_owned(),
        ],
        optimizer_rules: vec![
            "simplify_expressions".to_owned(),
            "push_down_filter".to_owned(),
            "eliminate_filter".to_owned(),
            "replace_distinct_aggregate".to_owned(),
        ],
        physical_optimizer_rules: vec![
            "EnsureRequirements".to_owned(),
            "SanityCheckPlan".to_owned(),
            "EnsureCooperative".to_owned(),
        ],
    }
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
/// Resolve only declared implementations. Unknown rules are configuration errors.
#[derive(Debug, Default)]
pub struct RuleCatalog;
impl RuleCatalog {
    /// Resolve an analyzer implementation.
    /// # Errors
    /// An unregistered implementation name.
    pub fn analyzer(name: &str) -> Result<Arc<dyn AnalyzerRule + Send + Sync>, CatalogError> {
        use datafusion::optimizer::analyzer::{
            resolve_grouping_function::ResolveGroupingFunction, type_coercion::TypeCoercion,
        };
        match name {
            "type_coercion" => Ok(Arc::new(TypeCoercion::new())),
            "resolve_grouping_function" => Ok(Arc::new(ResolveGroupingFunction::new())),
            _ => Err(unknown(name)),
        }
    }
    /// Resolve a logical optimizer implementation.
    /// # Errors
    /// An unregistered implementation name.
    pub fn optimizer(name: &str) -> Result<Arc<dyn OptimizerRule + Send + Sync>, CatalogError> {
        use datafusion::optimizer::{
            eliminate_filter::EliminateFilter, push_down_filter::PushDownFilter,
            simplify_expressions::SimplifyExpressions,
        };
        match name { "replace_distinct_aggregate"=>Ok(Arc::new(datafusion::optimizer::replace_distinct_aggregate::ReplaceDistinctWithAggregate::new())), "simplify_expressions"=>Ok(Arc::new(SimplifyExpressions::new())),"push_down_filter"=>Ok(Arc::new(PushDownFilter::new())),
            "eliminate_filter"=>Ok(Arc::new(EliminateFilter::new())),_=>Err(unknown(name)) }
    }
    /// Resolve a physical optimizer implementation.
    /// # Errors
    /// An unregistered implementation name.
    pub fn physical(
        name: &str,
    ) -> Result<Arc<dyn PhysicalOptimizerRule + Send + Sync>, CatalogError> {
        use datafusion::physical_optimizer::{
            ensure_coop::EnsureCooperative, ensure_requirements::EnsureRequirements,
            sanity_checker::SanityCheckPlan,
        };
        match name {
            "EnsureRequirements" => Ok(Arc::new(EnsureRequirements::new())),
            "SanityCheckPlan" => Ok(Arc::new(SanityCheckPlan::new())),
            "EnsureCooperative" => Ok(Arc::new(EnsureCooperative::new())),
            _ => Err(unknown(name)),
        }
    }
}
fn unknown(name: &str) -> CatalogError {
    CatalogError::ConfigInvalid {
        key: "engine_profile.rules".to_owned(),
        reason: format!("unknown rule implementation {name}"),
    }
}
