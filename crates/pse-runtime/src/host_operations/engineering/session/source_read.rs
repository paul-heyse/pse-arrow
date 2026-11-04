// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-only reads delegate exact raw-document ownership to the original loader.
use super::*;

fn observation_checkpoint(cancel: &CancelSource) -> Result<(), EngineeringServiceError> {
    cancel.checkpoint().map_err(owner)?;
    if cancel
        .deadline()
        .is_some_and(|deadline| std::time::Instant::now() >= deadline)
    {
        return Err(owner(crate::workflow::deadline_limit(
            "engineering physical inspection original clock",
        )));
    }
    Ok(())
}

/// Original physical inspection and allocation custody through typed response publication.
#[derive(Debug)]
pub struct EngineeringPhysicalOutput {
    /// Actual native quantity/unit/basis/datum and coverage outcome.
    pub document: crate::workflow::PhysicalInspectionDocument,
    pub(crate) owner: Arc<dyn pse_columnar::PayloadOwner>,
}
impl EngineeringPhysicalOutput {
    /// Retain original native and boundary encoding allocations through consumption.
    pub fn owner(&self) -> Arc<dyn pse_columnar::PayloadOwner> {
        self.owner.clone()
    }
}

impl EngineeringService {
    /// Inspect original physical declarations without constructing or invoking a provider.
    /// # Errors
    /// Exact-source, local query, native quantity/unit admission, deadline and resource
    /// refusals retain their native categories; transport success grants no readiness.
    pub async fn inspect_physical(
        &self,
        input: crate::host_operations::engineering_contract::EngineeringInspectPhysical,
        cancel: &CancelSource,
    ) -> Result<EngineeringPhysicalOutput, EngineeringServiceError> {
        input.validate_projection().map_err(owner)?;
        let (package, services) = self.inspection_source(input.source_operation)?;
        let activity = services.preparation().admit_activity().map_err(owner)?;
        activity
            .scope(async {
                observation_checkpoint(cancel)?;
                let encoding = datafusion::execution::memory_pool::MemoryConsumer::new(
                    "engineering:physical-inspection-encoding",
                )
                .register(&self.shared.pool());
                encoding.try_grow(4 * (64 << 10)).map_err(owner)?;
                let document = package
                    .physical_context()
                    .describe(&input.request)
                    .map_err(owner)?;
                observation_checkpoint(cancel)?;
                let native = match &document {
                    crate::workflow::PhysicalInspectionDocument::Page { owner, .. } => {
                        owner.clone()
                    }
                    crate::workflow::PhysicalInspectionDocument::Unavailable { .. } => None,
                };
                Ok(EngineeringPhysicalOutput {
                    document,
                    owner: Arc::new((native, encoding)),
                })
            })
            .await
    }

    /// Read one exact original source/data file with its retained allocation and attribution.
    /// # Errors
    /// Unready or absent source, stale parent, unknown document, cancellation and
    /// finite response/allocation refusals stay with their actual native owners.
    pub async fn read_source_file(
        &self,
        input: crate::host_operations::engineering_contract::EngineeringReadSourceFile,
        cancel: &CancelSource,
    ) -> Result<crate::workflow::OriginalSourceFileOutput, EngineeringServiceError> {
        input.validate_projection().map_err(owner)?;
        let (package, services) = self.inspection_source(input.source_operation)?;
        package
            .original_source_file(services.preparation(), &input.request, cancel)
            .await
            .map_err(owner)
    }
}
