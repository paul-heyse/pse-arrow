// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One selected publication composition over supplied facts and protected result owners.
use super::{Runtime, WorkflowError, contract};
use crate::{
    frozen_inspection::FrozenEvidenceSet, review_analysis::AnalysisDocument, review_pack::*,
};
use pse_columnar::CancellationToken;
use pse_ids::SemanticId;
use serde::{Deserialize, Serialize};
use std::path::Path;

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
    pub(super) fn bounds(&self) -> Result<(Option<f64>, Option<f64>), WorkflowError> {
        let min = self.minimum_bits.map(f64::from_bits);
        let max = self.maximum_bits.map(f64::from_bits);
        if min.is_some_and(|v| !v.is_finite())
            || max.is_some_and(|v| !v.is_finite())
            || min.zip(max).is_some_and(|(a, b)| a > b)
            || self.missing == Some(true) && (min.is_some() || max.is_some())
        {
            return Err(contract("invalid selected review output predicate"));
        }
        Ok((min, max))
    }
}
fn pack_error(error: PackError) -> WorkflowError {
    contract(format!("review pack: {error}"))
}

/// Shared publication preparation. Pure file consumers use this same mechanism without a store.
pub fn prepare_review_pack(
    destination: &Path,
    evidence: &FrozenEvidenceSet,
    analysis: &AnalysisDocument,
    request: &PackExportRequest,
    cancel: CancellationToken,
) -> Result<PackSession, WorkflowError> {
    let mut pack =
        PackSession::create(destination, request.selection.clone(), cancel).map_err(pack_error)?;
    pack.add_frozen(evidence).map_err(pack_error)?;
    pack.add_analysis(analysis).map_err(pack_error)?;
    pack.copy_files(&request.files).map_err(pack_error)?;
    for table in &request.tables {
        pack.export_table(&table.member, evidence, &table.request)
            .map_err(pack_error)?;
    }
    Ok(pack)
}

/// Supplied-only export has no runtime, deployment or native capability.
pub fn export_supplied_review_pack(
    destination: &Path,
    evidence: &FrozenEvidenceSet,
    analysis: &AnalysisDocument,
    mut request: PackExportRequest,
    cancel: CancellationToken,
) -> Result<PackIndex, WorkflowError> {
    if !request.scientific_results.is_empty() {
        return Err(contract(
            "protected scientific export requires an explicit durable runtime",
        ));
    }
    request.declare_tables()?;
    prepare_review_pack(destination, evidence, analysis, &request, cancel)?
        .seal()
        .map_err(pack_error)
}

impl Runtime {
    /// Compose multiple exact protected selections, holding one result reader at a time.
    /// Original model context is supplied factual material, never reconstructed from result rows.
    pub async fn export_review_pack(
        &self,
        destination: &Path,
        evidence: &FrozenEvidenceSet,
        analysis: &AnalysisDocument,
        mut request: PackExportRequest,
        cancel: CancellationToken,
    ) -> Result<PackIndex, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let (declarations, _selection_owner) = self
                    .review_result_selections(&request.scientific_results, &cancel)
                    .await?;
                for (selection, declared) in request.scientific_results.iter().zip(declarations) {
                    if request
                        .selection
                        .members
                        .iter()
                        .any(|member| member.id == selection.member)
                    {
                        return Err(contract("scientific review member is declared twice"));
                    }
                    request.selection.members.push(PackMemberSelection {
                        id: selection.member.clone(),
                        role: PackMemberRole::Result,
                        required: true,
                        format: PackFormat::ArrowIpcStream,
                        document_version: None,
                        source: None,
                        references: Vec::new(),
                        result: Some(declared),
                        diagnostic_table: None,
                        capture_scope: None,
                    });
                }
                request.declare_tables()?;
                let mut pack =
                    prepare_review_pack(destination, evidence, analysis, &request, cancel.clone())?;
                for selected in &request.scientific_results {
                    cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
                    let outcome = if let Some(output) = &selected.output {
                        let (minimum, maximum) = output.bounds()?;
                        match self
                            .output_results(
                                &selected.run,
                                &selected.attempt,
                                &selected.relation,
                                output.owner,
                                &output.field,
                                &output.partition,
                                selected.row_start,
                                selected.row_end,
                                minimum,
                                maximum,
                                output.missing,
                                cancel.clone(),
                            )
                            .await
                        {
                            Ok(reader) => pack
                                .export_selected_result(&selected.member, reader)
                                .await
                                .map_err(pack_error),
                            Err(error) => Err(error),
                        }
                    } else {
                        match self
                            .results(
                                &selected.run,
                                &selected.attempt,
                                &selected.relation,
                                selected.row_start,
                                selected.row_end,
                                cancel.clone(),
                            )
                            .await
                        {
                            Ok(reader) => pack
                                .export_result(&selected.member, reader)
                                .await
                                .map_err(pack_error),
                            Err(error) => Err(error),
                        }
                    };
                    if let Err(error) = outcome {
                        let _ = pack.fail_member(&selected.member, error.to_string());
                        return Err(error);
                    }
                }
                pack.seal().map_err(pack_error)
            })
            .await
    }
}
