// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Concrete parent operation: ordinary run custody around original finite child studies.
use super::*;
use crate::workflow::{ModelingPackage, Runtime, WorkflowError, contract};
use pse_model::generated::identities::StudyId;

/// Source-bound reconstructable expectation intent and its actual runtime owner.
#[derive(Clone, Debug)]
pub struct PreparedExpectationOperation {
    /// Original source and physical context, retained until actual publication joins.
    pub source: ModelingPackage,
    /// Complete source-owned law/member/output/control declaration.
    pub request: ExpectationRequest,
    /// Original statistical parent, independent of coordinator execution identity.
    pub parent: StudyId,
    /// Existing native study orchestration capability; never serialized into a request.
    pub(crate) runtime: Runtime,
}
/// Original frozen parent products, published under the producing ordinary run manifest.
#[derive(Clone, Debug)]
pub struct ExpectationProduct {
    /// Original statistical parent and its exact child identities.
    pub parent: StudyId,
    /// Original terminal report, including unmet precision or incomplete scope.
    pub report: ExpectationReport,
    /// All settled original child populations, including a final incomplete child.
    pub recorded_iterations: usize,
    pub(crate) runtime: Runtime,
    pub(crate) _owner: std::sync::Arc<pse_columnar::AllocationLease>,
}
impl PreparedExpectationOperation {
    /// Source, measure and sampler controls define mathematical intent, not the controller run.
    pub fn identity(&self) -> Result<pse_ids::ContentHash, WorkflowError> {
        pse_ids::document::of(pse_ids::Frame::ExpectationRequestV1, &self.request)
            .map_err(|error| contract(error.to_string()))
    }
    /// Execute or reconcile original children; the enclosing run owns final result publication.
    pub(crate) async fn evaluate(
        &self,
        cancel: &crate::CancelSource,
    ) -> Result<ExpectationProduct, WorkflowError> {
        let owner = self.runtime.preparation_services().math().reserve(
            "expectation:terminal-product",
            8 * execution::DOCUMENT_BYTES,
        )?;
        let report = self
            .runtime
            .execute_expectation_science(&self.source, self.parent, self.request.clone(), cancel)
            .await?;
        let finalization = self
            .runtime
            .canonical_store()?
            .expectation_finalization(&self.parent.to_string())
            .await?
            .ok_or_else(|| contract("expectation original terminal decision absent"))?;
        let recorded_iterations = finalization.latest_boundary.map_or(Ok(0), |index| {
            usize::try_from(index)
                .ok()
                .and_then(|index| index.checked_add(1))
                .ok_or_else(|| contract("expectation original iteration extent"))
        })?;
        Ok(ExpectationProduct {
            parent: self.parent,
            report,
            recorded_iterations,
            runtime: self.runtime.clone(),
            _owner: owner,
        })
    }
}
impl ExpectationProduct {
    /// Whole original requested statistical scope and precision are available.
    pub fn permits_result(&self) -> bool {
        !self.report.interrupted
            && self.report.outputs.len() == self.report.request.outputs.len()
            && self
                .report
                .outputs
                .iter()
                .zip(&self.report.request.outputs)
                .all(|(output, request)| {
                    output.complete
                        && ((request.absolute_error.is_none() && request.relative_error.is_none())
                            || output.precision_met)
                })
            && (!self.report.request.failure_probability
                || self.report.failure_probability_complete)
    }
}
