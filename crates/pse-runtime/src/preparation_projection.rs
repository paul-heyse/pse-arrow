// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Generated typed preparation crossings; native preparation retains contextual authority.

use crate::projection_support::*;
use pse_model::boundary_projection::{AdapterError, AdapterFailure};

#[path = "generated/engineering/native-adapters.rs"]
mod generated;
pub use generated::*;

/// A complete typed physical page plus its original native allocation custody.
#[derive(Debug)]
pub struct PhysicalInspectionProjection {
    /// Actual source-generated physical interpretation, ready for bounded encoding.
    pub document: crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalInspectionDocument,
    /// Keep this owner alive through serialization and attach it to the encoded bytes.
    pub owner: Option<std::sync::Arc<pse_columnar::MemoryReservation>>,
}

/// Cross the actual physical document while preserving its native allocation owner.
/// # Errors
/// Original native local envelopes and outgoing generated validation must hold.
pub fn encode_owned_physical_inspection_document(
    native: crate::workflow::PhysicalInspectionDocument,
) -> Result<PhysicalInspectionProjection, BoundaryProjectionError> {
    let owner = match &native {
        crate::workflow::PhysicalInspectionDocument::Page { owner, .. } => owner.clone(),
        crate::workflow::PhysicalInspectionDocument::Unavailable { .. } => None,
    };
    Ok(PhysicalInspectionProjection {
        document: encode_physical_inspection_document(native)?,
        owner,
    })
}

/// Original native inspection failure or typed projection refusal, with both owners retained.
#[derive(Debug, thiserror::Error)]
pub enum PreparationDiscoveryProjectionError {
    /// Local wire/native conversion failed before publication.
    #[error(transparent)]
    Projection(#[from] BoundaryProjectionError),
    /// Original exact-source, contextual, cancellation or infrastructure outcome.
    #[error(transparent)]
    Workflow(#[from] crate::workflow::WorkflowError),
}
pse_diagnostics::impl_diagnostic! {
    PreparationDiscoveryProjectionError,
    code(_this) { None },
    forward(this) { match this { Self::Projection(error) => Some(error), Self::Workflow(error) => Some(error) } },
    help(_this) { None }, related(_this) { None }, source(_this) { None }
}
impl pse_model::diagnostic::DiagnosticProjection for PreparationDiscoveryProjectionError {
    fn boundary_diagnostic(
        &self,
        stage: pse_diagnostics::DiagnosticStage,
    ) -> pse_model::diagnostic::BoundaryDiagnostic {
        use pse_model::diagnostic::DiagnosticProjection;
        match self {
            Self::Projection(error) => DiagnosticProjection::boundary_diagnostic(error, stage),
            Self::Workflow(error) => DiagnosticProjection::boundary_diagnostic(error, stage),
        }
    }
}

/// Inspect original preparation metadata through typed Buffa request/report adapters.
/// This creates no runtime, evaluator, provider, scientific payload or alternate cache owner.
///
/// # Errors
/// Local rules, unknown-data adoption and original native contextual failures stay typed.
pub async fn discover(
    package: &crate::workflow::ModelingPackage,
    services: &crate::workflow::SourceServices,
    request: crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationDiscoveryRequest,
    cancel: &crate::CancelSource,
) -> Result<crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationDiscoveryReport, PreparationDiscoveryProjectionError>{
    let request = decode_preparation_discovery_request(request)?;
    let report = package
        .preparation_discovery(services, &request, cancel)
        .await?;
    Ok(encode_preparation_discovery_report(report)?)
}

#[cfg(test)]
mod tests;
