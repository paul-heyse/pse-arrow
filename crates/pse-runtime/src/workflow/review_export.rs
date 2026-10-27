// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One selected publication composition over supplied facts and protected result owners.
use super::WorkflowError;
use pse_ids::SemanticId;
use serde::{Deserialize, Serialize};

/// Explicit existing deployment selection for protected CLI reads, never scientific reconstruction.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewExportDeployment {
    /// Deployment-selection codec.
    pub version: pse_model::document::Version<1>,
    /// Existing finite deployment settings.
    pub runtime: super::DiagnosticRuntimeSettings,
    /// Explicit canonical supervisor state; reading does not initialize a database.
    pub substrate: String,
    /// Explicit existing database, or the selected supervisor's configured database.
    pub database: Option<String>,
    /// Existing qualified mathematical producer when needed by the deployment.
    pub producer: Option<String>,
}

/// A request to the scientific owner, not a caller assertion of lineage or permission.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewScientificSelection {
    /// Selected pack member identifier.
    pub member: String,
    /// Exact existing canonical run key.
    pub run: String,
    /// Exact existing canonical attempt key.
    pub attempt: String,
    /// Registered scientific relation name.
    pub relation: String,
    /// Inclusive original relation row offset.
    pub row_start: u64,
    /// Exclusive original relation row offset.
    pub row_end: u64,
    /// Existing selected-output semantics; absence selects the original relation range.
    pub output: Option<ReviewOutputSelection>,
}
/// Original output owner/field/partition and finite-value or trajectory-time predicate.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewOutputSelection {
    /// Original scientific output owner.
    pub owner: SemanticId,
    /// Original scientific output field.
    pub field: String,
    /// Original scientific output partition.
    pub partition: String,
    /// Optional finite lower bound represented by IEEE bits.
    pub minimum_bits: Option<u64>,
    /// Optional finite upper bound represented by IEEE bits.
    pub maximum_bits: Option<u64>,
    /// Optional original missing-value selection.
    pub missing: Option<bool>,
}
impl ReviewOutputSelection {
    pub(crate) fn bounds(&self) -> Result<(Option<f64>, Option<f64>), WorkflowError> {
        let min = self.minimum_bits.map(f64::from_bits);
        let max = self.maximum_bits.map(f64::from_bits);
        super::result_selection::ValuePredicate::new(false, min, max, self.missing)?;
        Ok((min, max))
    }
}
