// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual native inspection with selected allocation custody through wire encoding.

use super::*;
use crate::host_operations::engineering_contract::{
    EngineeringInspectModeling, EngineeringInspectProviders,
};

/// Actual native model document and its original selected/encoding allocation owners.
#[derive(Debug)]
pub struct EngineeringModelingOutput {
    /// Original contextual inspection; its declared procedure has not been dispatched.
    pub document: crate::workflow::ModelingInspection,
    /// Actual selected native metadata and complete finite boundary encoding grants.
    pub owner: Arc<dyn pse_columnar::PayloadOwner>,
}

impl EngineeringService {
    /// Read original contextual model populations and provenance without executing science.
    /// # Errors
    /// Native contextual admission, field/rule, finite selection, deadline and pool errors propagate.
    pub async fn inspect_modeling(
        &self,
        input: EngineeringInspectModeling,
        cancel: &CancelSource,
    ) -> Result<EngineeringModelingOutput, EngineeringServiceError> {
        input.validate_projection().map_err(owner)?;
        let (package, services) = self.inspection_source(input.source_operation)?;
        let encoding = self
            .shared
            .math()
            .reserve("engineering:model-inspection-encoding", 4 * (64 << 10))
            .map_err(owner)?;
        let document = package
            .inspect_case_request(&services.preparation(), &input.request, cancel)
            .await
            .map_err(owner)?;
        cancel.checkpoint().map_err(owner)?;
        let selected = document.payload_owner();
        Ok(EngineeringModelingOutput {
            document,
            owner: Arc::new((selected, encoding)),
        })
    }

    /// Read original retained provider registrations; never creates or evaluates a native worker.
    /// # Errors
    /// Exact source, bounded query, original cancellation/clock and native pool refusals propagate.
    pub fn inspect_providers(
        &self,
        input: EngineeringInspectProviders,
        cancel: &CancelSource,
    ) -> Result<crate::workflow::ProviderInspectionOutput, EngineeringServiceError> {
        input.validate_projection().map_err(owner)?;
        let (package, services) = self.inspection_source(input.source_operation)?;
        let encoding = self
            .shared
            .math()
            .reserve("engineering:provider-inspection-encoding", 4 * (64 << 10))
            .map_err(owner)?;
        let original = package
            .inspect_providers(&services.preparation(), &input.request, cancel)
            .map_err(owner)?;
        Ok(crate::workflow::ProviderInspectionOutput {
            page: original.page,
            owner: Arc::new((original.owner, encoding)),
        })
    }
}
