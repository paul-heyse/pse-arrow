// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit finite diagnostic samples share one model and bounded execution policy.
use super::*;
use pse_model::generated::identities::RunId;
use std::{
    collections::BTreeSet,
    time::{Duration, Instant},
};

/// Why a finite sample campaign stopped; individual evaluation failures are retained separately.
pub use pse_model::generated::enums::ModelingDiagnosticSampleStop as DiagnosticSampleStop;
/// Named sample evidence. Every result uses the same original model and numerical profile.
#[derive(Clone, Debug)]
pub struct ModelingDiagnosticSamples {
    pub run_id: RunId,
    pub(in crate::workflow::modeling) runtime: Runtime,
    pub outcomes: Vec<(
        SemanticId,
        Result<Arc<ModelingDiagnostics>, BoundaryDiagnostic>,
    )>,
    pub unattempted: usize,
    pub stop: DiagnosticSampleStop,
    pub(in crate::workflow::modeling) _owner: Arc<pse_columnar::AllocationLease>,
}
impl ModelingPackage {
    /// Inspect supplied points without changing starts or generating scientific guesses.
    /// The finding limit and wall allowance apply across the complete sample set.
    pub async fn diagnose_samples(
        &self,
        prepared: ModelingDiagnosticPreparation,
        samples: Vec<(SemanticId, CaseValues)>,
        policy: ModelingDiagnosticPolicy,
        compiler: pse_compiler::workspace::Profile,
        maximum_samples: usize,
        time_limit: Duration,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingDiagnosticSamples, WorkflowError> {
        policy.validate()?;
        if maximum_samples == 0
            || time_limit.is_zero()
            || samples
                .iter()
                .map(|(id, _)| id)
                .collect::<BTreeSet<_>>()
                .len()
                != samples.len()
        {
            return Err(contract("diagnostic sample identities or work limits"));
        }
        let deadline = Instant::now()
            .checked_add(time_limit)
            .ok_or_else(|| contract("diagnostic sample deadline"))?;
        let bytes = samples
            .len()
            .min(maximum_samples)
            .checked_mul(1024)
            .ok_or_else(|| contract("diagnostic sample extent"))?;
        let owner = self
            .runtime
            .shared
            .math()
            .reserve("modeling:diagnostic-samples", bytes)?;
        let mut report = ModelingDiagnosticSamples {
            run_id: pse_operations::mint_id(),
            runtime: self.runtime.clone(),
            outcomes: Vec::new(),
            unattempted: samples.len(),
            stop: DiagnosticSampleStop::Completed,
            _owner: owner,
        };
        let mut remaining = policy.maximum_findings;
        for (id, values) in samples {
            if report.outcomes.len() == maximum_samples {
                report.stop = DiagnosticSampleStop::SampleLimit;
                break;
            }
            if remaining == 0 {
                report.stop = DiagnosticSampleStop::FindingLimit;
                break;
            }
            if cancel.token().is_cancelled() {
                report.stop = DiagnosticSampleStop::Cancelled;
                break;
            }
            if Instant::now() >= deadline {
                report.stop = DiagnosticSampleStop::TimeLimit;
                break;
            }
            if prepared.validate_point(&values).is_err() {
                report.outcomes.push((
                    id,
                    Err(BoundaryDiagnostic::new(
                        BoundaryClass::InvalidModel,
                        "modeling.diagnostic_samples",
                        [id],
                        "sample changes frozen inputs; prepare another case",
                    )),
                ));
                report.unattempted -= 1;
                remaining = remaining.saturating_sub(1);
                continue;
            }
            let child = crate::CancelSource::new();
            let operation = self.diagnose_case(
                prepared.clone(),
                values,
                ModelingDiagnosticPolicy {
                    maximum_findings: remaining,
                    ..policy.clone()
                },
                compiler,
                &child,
            );
            tokio::pin!(operation);
            let result = tokio::select! {
                value = &mut operation => value,
                () = cancel.cancelled() => { child.cancel(); let value=operation.await; report.stop=DiagnosticSampleStop::Cancelled; value },
                () = tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)) => { child.cancel(); let value=operation.await; report.stop=DiagnosticSampleStop::TimeLimit; value },
            };
            let result = result
                .map(Arc::new)
                .map_err(|error| error.boundary_diagnostic());
            remaining = remaining.saturating_sub(result.as_ref().map_or(1, |r| r.findings.len()));
            report.outcomes.push((id, result));
            report.unattempted -= 1;
            if report.stop != DiagnosticSampleStop::Completed {
                break;
            }
        }
        Ok(report)
    }
}
