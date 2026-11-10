// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original native expectation custody survives RPC waiter and source-view release.
use super::*;
use crate::host_operations::engineering_contract::{EngineeringExecuteExpectation, EngineeringReadExpectation, EngineeringReadExpectationSamples, EngineeringExpectationSamples};

/// Native report with its complete encoding allowance retained through transmission.
#[derive(Clone, Debug)]
pub struct EngineeringExpectationOutput {
    /// Original report; reading never repairs a missing scientific contribution.
    pub report: crate::workflow::expectation::ExpectationReport,
    /// Original bounded response and retained-document ownership.
    pub owner: Arc<dyn pse_columnar::PayloadOwner>,
}
/// One bounded registered population page with its original response custody.
#[derive(Debug)]
pub struct EngineeringExpectationSamplesOutput {
    /// Original immutable sample attribution and observed physical values.
    pub page: EngineeringExpectationSamples,
    /// Complete original page encoding allowance.
    pub owner: Arc<dyn pse_columnar::PayloadOwner>,
}
impl EngineeringService {
    fn expectation_report_response(&self, report: &crate::workflow::expectation::ExpectationReport) -> Result<Arc<pse_columnar::AllocationLease>, EngineeringServiceError> {
        let extent = crate::workflow::document_extent(report, 16 << 20).map_err(owner)?;
        let bytes = extent.checked_mul(128).ok_or_else(|| request("expectation response extent"))?;
        self.shared.math().reserve("engineering:expectation-report-response", bytes).map_err(owner)
    }
    /// Read a finite original population page after source and producer retirement.
    pub async fn read_expectation_samples(&self, input: EngineeringReadExpectationSamples, cancel: &CancelSource) -> Result<EngineeringExpectationSamplesOutput, EngineeringServiceError> {
        use pse_relations::columnar::RelationRow;
        input.validate_projection().map_err(owner)?;
        let to_usize = |value| usize::try_from(value).map_err(|_| request("expectation page coordinate exceeds addressable memory"));
        let batch = self.runtime.expectation_samples(input.parent, to_usize(input.iteration)?, to_usize(input.offset)?, input.limit as usize, cancel).await.map_err(owner)?;
        let bytes = batch.batch().get_array_memory_size().checked_add(512).and_then(|bytes| bytes.checked_mul(128)).ok_or_else(|| request("expectation sample response extent"))?;
        let reservation = self.shared.math().reserve("engineering:expectation-sample-response", bytes).map_err(owner)?;
        let rows = pse_model::generated::runtime::expectation_samples::Row::rows(&batch).map_err(owner)?;
        let next_offset = (rows.len() == input.limit as usize).then(|| input.offset.checked_add(rows.len() as u64)).flatten();
        Ok(EngineeringExpectationSamplesOutput { page: EngineeringExpectationSamples { parent: input.parent, iteration: input.iteration, offset: input.offset, rows, next_offset }, owner: Arc::new((batch, reservation)) })
    }
    /// Execute/reconcile the selected law under original background operation custody.
    pub async fn execute_expectation(self: &Arc<Self>, input: EngineeringExecuteExpectation, cancel: &CancelSource) -> Result<EngineeringExpectationOutput, EngineeringServiceError> {
        input.validate_projection().map_err(owner)?;
        if input.operation == input.source_operation { return Err(request("expectation cannot depend on its own operation")); }
        let source = self.operation(&input.source_operation, Kind::Source)?;
        let (original, fresh) = self.issue(&input.operation, Kind::Expectation, &input)?;
        if fresh {
            let service = self.clone();
            let operation = original.clone();
            self.spawn(original.clone(), async move {
                let Outcome::Source { package, .. } = source.dependency(&operation.cancel).await? else { return Err(request("expectation original source is unavailable")); };
                let retained = service.runtime.execute_expectation(&package, input.parent, input.request, &operation.cancel).await.map_err(owner)?;
                Ok(Outcome::Expectation(Arc::new(EngineeringExpectationOutput { report: retained.report, owner: retained.owner })))
            })?;
        }
        match original.dependency(cancel).await? {
            Outcome::Expectation(output) => {
                // Dependency clones share the original product. Each waiter
                // admits its own DTO/protobuf geometry before making a copy.
                let response = self.expectation_report_response(&output.report)?;
                Ok(EngineeringExpectationOutput { report: output.report.clone(), owner: Arc::new((output, response)) })
            }
            Outcome::Failed(error) => Err(EngineeringServiceError::Original(error)),
            _ => Err(request("expectation original operation has no report")),
        }
    }
    /// Read the original protected boundary after native producer and source retirement.
    pub async fn read_expectation(&self, input: EngineeringReadExpectation, cancel: &CancelSource) -> Result<EngineeringExpectationOutput, EngineeringServiceError> {
        input.validate_projection().map_err(owner)?;
        cancel.checkpoint().map_err(owner)?;
        let retained = self.runtime.retained_data_services().expectation_report_owned(input.parent, cancel).await.map_err(owner)?;
        cancel.checkpoint().map_err(owner)?;
        let reservation = self.expectation_report_response(&retained.report)?;
        Ok(EngineeringExpectationOutput { report: retained.report, owner: Arc::new((retained.owner, reservation)) })
    }
}
