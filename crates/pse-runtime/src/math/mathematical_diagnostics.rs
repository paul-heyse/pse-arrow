// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native job ownership for original-source mathematical diagnostics.
use super::{ConstructionEstimate, MathRuntimeError, MathService, modeling::ModelingPreparation};
use pse_math::{
    binding::CaseValues,
    mathematical_diagnostics::{DiagnosticRequest, DiagnosticResult},
};
use std::sync::Arc;

impl MathService {
    /// Prepare and execute one original diagnostic under the caller's unchanged
    /// deadline, joined cancellation and ordinary native allocation ownership.
    pub async fn diagnose_modeling(
        self: &Arc<Self>,
        model: ModelingPreparation,
        values: CaseValues,
        request: DiagnosticRequest,
        driver: &crate::CancelSource,
    ) -> Result<Arc<pse_columnar::Leased<DiagnosticResult>>, MathRuntimeError> {
        let driver = self.preparation_driver(driver)?;
        let control = pse_diagnostics::Cancellation::default();
        let deadline = driver.deadline();
        let cores = self.cores();
        let bytes = request.limits.bytes;
        let operation = self.job_retained_construction(
            cores,
            ConstructionEstimate::Opaque(bytes),
            control.clone(),
            deadline,
            move |cancellation| {
                let threads = std::num::NonZeroUsize::new(cores)
                    .ok_or(MathRuntimeError::Limit("diagnostic native threads"))?;
                let scope = pse_kernels::ExecutionScope::new(cancellation, deadline)
                    .with_admitted_threads(threads);
                let prepared = model
                    .compiled()
                    .prepare_mathematical_diagnostic(&values, request, &scope)?;
                let result = prepared.evaluate(&scope)?;
                let bytes = serde_json::to_vec(&result)
                    .map_err(|error| MathRuntimeError::Infrastructure(error.to_string()))?
                    .len()
                    .checked_add(size_of::<DiagnosticResult>())
                    .ok_or(MathRuntimeError::Limit("diagnostic result bytes"))?;
                Ok((result, bytes))
            },
        );
        tokio::pin!(operation);
        let (result, owner) = tokio::select! {
            result = &mut operation => result?,
            () = driver.cancelled() => { control.cancel(); let _ = operation.await; return Err(MathRuntimeError::Cancelled); }
        };
        Ok(Arc::new(pse_columnar::Leased::new(Arc::new(result), owner)))
    }
}
