// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit original-source diagnostic work under one caller clock, including admission.
use super::{
    DiagnosticOperation, DiagnosticOrigin, ModelingDiagnosticPolicy, Runtime, WorkflowError,
    contract,
};
use crate::{CancelSource, math::settings::SolveSettings};
use pse_diagnostics::observation::{OperationId, Outcome};
use pse_ids::SemanticId;
use pse_model::{diagnostic::BoundaryDiagnostic, document::Version};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, time::Duration};

/// CLI deployment selection forwarded to the existing runtime settings constructor.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticRuntimeSettings {
    /// Accounted participating execution-memory allowance, not a process RSS promise.
    pub memory_limit_bytes: usize,
    /// Existing shared runtime thread budget.
    pub threads: usize,
    /// Explicit spill destination, never discovered from descriptive source context.
    pub spill_dir: String,
    /// Existing disk-manager spill allowance.
    pub max_spill_bytes: u64,
    /// Existing execution batch size.
    pub batch_size: usize,
}
/// Explicit new-work CLI composition. Passive commands accept none of these capabilities.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticCommandRequest {
    /// Command codec.
    pub version: Version<1>,
    /// Finite existing deployment settings.
    pub runtime: DiagnosticRuntimeSettings,
    /// Original-source scientific diagnostic operation.
    pub operation: DiagnosticExecutionRequest,
}

/// New numerical work. Descriptive ReviewContext is deliberately not an input constructor.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticExecutionRequest {
    /// Request codec.
    pub version: Version<1>,
    /// Explicit original physical source documents, keyed by declared filename.
    pub physical_documents: BTreeMap<String, String>,
    /// Explicit original authored model packages.
    pub model_documents: Vec<BTreeMap<String, String>>,
    /// Exact declared case.
    pub case_id: SemanticId,
    /// Existing solve policy; no diagnostic accuracy override is introduced.
    pub settings: SolveSettings,
    /// Existing operation-specific point/matrix/term diagnostic controls.
    pub diagnostics: ModelingDiagnosticPolicy,
    /// Original retained failure/attempt link, independent of new success.
    pub origin: Option<DiagnosticOrigin>,
    /// Finite clock starts before document admission and preparation.
    #[schemars(with = "pse_model::document::ClosedDuration")]
    pub allowance: Duration,
    /// Finite encoded original-source extent.
    pub maximum_source_bytes: usize,
}
/// Copied findings from the existing explicit diagnostic operation.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticExecutionDocument {
    /// Result codec.
    pub version: Version<1>,
    /// Actual new diagnostic operation established before source admission.
    pub operation: OperationId,
    /// Original attribution, never replaced by this operation's outcome.
    pub origin: Option<DiagnosticOrigin>,
    /// Existing diagnostic run identity, absent on pre-handle failure.
    pub run_id: Option<pse_model::generated::identities::RunId>,
    /// Original case requested.
    pub case_id: SemanticId,
    /// Existing diagnostic findings in original coordinates where available.
    pub findings: Vec<BoundaryDiagnostic>,
    /// Actual model counts from the diagnostic owner.
    pub statistics: BTreeMap<String, usize>,
    /// Every requested point/matrix/term analysis completed within its own budgets.
    pub complete: bool,
    /// Failure before or during diagnostic work; not erased by an empty finding list.
    pub failure: Option<BoundaryDiagnostic>,
    /// Actual original row coordinates and physical nominals.
    pub row_nominals: Vec<(SemanticId, f64)>,
    /// Actual original variable coordinates and physical nominals.
    pub variable_nominals: Vec<(SemanticId, f64)>,
}
impl Runtime {
    /// Diagnose admitted original sources. No hidden store discovery or replay reconstruction.
    pub async fn diagnose_documents(
        &self,
        request: DiagnosticExecutionRequest,
        parent: &CancelSource,
    ) -> Result<DiagnosticExecutionDocument, WorkflowError> {
        let operation =
            DiagnosticOperation::new(parent, request.allowance, request.origin.clone())?;
        let observer = pse_diagnostics::observation::current().map_or_else(
            || self.observations().hub().start("diagnose.original_sources"),
            |parent| {
                parent.child(
                    "diagnose.original_sources",
                    pse_diagnostics::Relationship::Nested,
                )
            },
        );
        let context = observer.context();
        let mut document = DiagnosticExecutionDocument {
            version: Version,
            operation: context.id(),
            origin: request.origin.clone(),
            run_id: None,
            case_id: request.case_id,
            findings: Vec::new(),
            statistics: BTreeMap::new(),
            complete: false,
            failure: None,
            row_nominals: Vec::new(),
            variable_nominals: Vec::new(),
        };
        let outcome = pse_diagnostics::observation::scope(
            Some(context),
            operation.within(self.scope_activity(async {
                operation.check()?;
                if request.maximum_source_bytes == 0
                    || request.maximum_source_bytes > 64 << 20
                    || request.model_documents.len() > 32
                {
                    return Err(contract("explicit diagnostic source budget"));
                }
                let bytes = request
                    .physical_documents
                    .iter()
                    .chain(request.model_documents.iter().flat_map(|m| m.iter()))
                    .try_fold(0usize, |n, (key, value)| {
                        n.checked_add(key.len())
                            .and_then(|n| n.checked_add(value.len()))
                            .filter(|n| *n <= request.maximum_source_bytes)
                            .ok_or_else(|| contract("explicit diagnostic source extent"))
                    })?;
                let _source_owner = self
                    .shared
                    .math()
                    .reserve("diagnostic:source-documents", bytes)?;
                request.diagnostics.validate()?;
                let profile = request.settings.profile()?;
                let pool = self.shared.pool();
                let token = operation.driver().token();
                let validation = self.admission_context();
                let load = |documents: &BTreeMap<String, String>| {
                    let bytes = documents
                        .iter()
                        .map(|(name, value)| (name.clone(), value.as_bytes().to_vec()))
                        .collect();
                    pse_admission::authoring_driver::document::load_package_documents_owned(
                        &bytes,
                        self.registry(),
                        pse_authoring::ParseBudget::default(),
                        &pool,
                        &token,
                        &validation,
                    )
                };
                let physical =
                    pse_admission::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
                        vec![load(&request.physical_documents)?],
                        &pool,
                        &token,
                    )?;
                let physical = self.physical_from_documents(&physical, &token).await?;
                operation.check()?;
                let bundles = request
                    .model_documents
                    .iter()
                    .map(load)
                    .collect::<Result<Vec<_>, _>>()?;
                let source =
                    pse_admission::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
                        bundles, &pool, &token,
                    )?;
                let model = self
                    .modeling_from_documents(&source, physical, operation.driver())
                    .await?;
                operation.check()?;
                let execution = model
                    .declared_execution(
                        &self.preparation_services(),
                        request.case_id.into(),
                        Default::default(),
                        profile,
                        Default::default(),
                        Default::default(),
                        operation.driver(),
                    )
                    .await?;
                let prepared = model
                    .prepare_declared_diagnostics(
                        &self.preparation_services(),
                        &execution,
                        operation.driver(),
                    )
                    .await?;
                let values = prepared.model.values.clone();
                model
                    .diagnose_case(
                        &self.execution_services(),
                        prepared,
                        values,
                        request.diagnostics,
                        execution.analysis.compiler,
                        &operation,
                    )
                    .await
            })),
        )
        .await;
        match outcome {
            Ok(result) => {
                document.run_id = Some(result.run_id);
                document.complete = result.complete;
                document.findings = result.findings;
                document.statistics = result.statistics;
                document.row_nominals = result.row_nominals;
                document.variable_nominals = result.variable_nominals;
                observer.finish(Outcome::Completed);
            }
            Err(error) => {
                document.failure = Some(error.boundary_diagnostic());
                observer.finish(if operation.driver().token().is_cancelled() {
                    Outcome::Cancelled
                } else {
                    Outcome::Failed
                });
            }
        }
        Ok(document)
    }
}
