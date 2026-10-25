// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Tooling composition consumes protected runtime selections and supplied facts.
use super::{
    frozen_inspection::FrozenEvidenceSet, review_analysis::AnalysisDocument, review_pack::*,
};
use crate::workflow::{Runtime, WorkflowError, contract};
use pse_columnar::CancellationToken;
use std::path::Path;

fn pack_error(error: PackError) -> WorkflowError {
    contract(format!("review pack: {error}"))
}

/// Publish an explicit finite host investigation through the original member custody owner.
pub fn export_supplied_members(
    destination: &Path,
    request: PackExportRequest,
    cancel: CancellationToken,
) -> Result<PackIndex, WorkflowError> {
    if !request.scientific_results.is_empty() || !request.tables.is_empty() {
        return Err(contract(
            "supplied-member export has no scientific result or registry query authority",
        ));
    }
    let mut pack =
        PackSession::create_members(destination, request.selection, cancel).map_err(pack_error)?;
    pack.copy_files(&request.files).map_err(pack_error)?;
    pack.seal().map_err(pack_error)
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

/// Compose multiple exact protected selections, holding one result reader at a time.
/// Original model context is supplied factual material, never reconstructed from result rows.
pub async fn export_review_pack(
    runtime: &Runtime,
    destination: &Path,
    evidence: &FrozenEvidenceSet,
    analysis: &AnalysisDocument,
    mut request: PackExportRequest,
    cancel: CancellationToken,
) -> Result<PackIndex, WorkflowError> {
    let activity = runtime.admit_activity()?;
    activity
        .scope(async {
            let (declarations, _selection_owner) = runtime
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
                    match runtime
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
                    match runtime
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
