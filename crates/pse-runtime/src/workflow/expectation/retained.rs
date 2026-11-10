// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Protected original expectation readers: no journal replay or scientific services.
use super::*;
use crate::workflow::{RetainedDataServices, Runtime, WorkflowError, contract, relation};
use pse_model::generated::identities::StudyId;
use pse_operations::canonical_expectations::Publication;
use pse_relations::{columnar::{Collection, FieldCheckedBatch, RelationRow}, generated::runtime::{expectation_summaries as summaries, expectation_samples as samples, expectation_estimates as estimates}};
use std::sync::Arc;

/// Original report and nonexpiring protected-selection/encoding custody.
#[derive(Debug)]
pub struct RetainedExpectationReport {
    /// Frozen original report; no available subset is promoted to completion.
    pub report: ExpectationReport,
    /// Original recorded selection and its owned decode allowance.
    pub owner: Arc<dyn pse_columnar::PayloadOwner>,
}
impl std::ops::Deref for RetainedExpectationReport {
    type Target = ExpectationReport;
    fn deref(&self) -> &Self::Target { &self.report }
}
impl RetainedDataServices {
    async fn expectation_summary(&self, parent: StudyId, cancel: &crate::CancelSource) -> Result<(summaries::Row, Publication, crate::workflow::ResultSnapshot, Arc<pse_columnar::MemoryReservation>), WorkflowError> {
        let publication = self.canonical_store()?.expectation_publication(&parent.to_string()).await?.ok_or_else(|| contract("expectation has no acknowledged original result publication"))?;
        let attempt = self.canonical_store()?.canonical_attempt(&publication.attempt).await?.ok_or_else(|| contract("expectation original attempt unavailable"))?;
        if attempt.run != publication.run || attempt.closed_manifest.as_deref() != Some(publication.manifest.as_str()) { return Err(contract("expectation original result manifest differs")); }
        let mut reader = self.results(&publication.run, &publication.attempt, "runtime.expectation_summaries", 0, u64::MAX, cancel.token()).await?;
        let snapshot = reader.snapshot();
        let owner = Arc::new(pse_columnar::MemoryConsumer::new("expectation:retained-report-copy").register(&self.pool()));
        let mut selected = None;
        while let Some(batch) = reader.next_relation_batch().await? {
            let bytes = batch.batch().get_array_memory_size().checked_mul(10).ok_or_else(|| contract("expectation report copy extent"))?;
            owner.try_grow(bytes).map_err(pse_engine::EngineError::from)?;
            for row in summaries::Row::rows(&batch).map_err(relation)? {
                if row.parent_id != parent || selected.is_some() { return Err(contract("expectation original summary coverage differs")); }
                selected = Some(row);
            }
        }
        Ok((selected.ok_or_else(|| contract("expectation original summary unavailable"))?, publication, snapshot, owner))
    }
    /// Read the exact original parent report, retaining its selected result ticket through drain.
    pub async fn expectation_report_owned(&self, parent: StudyId, cancel: &crate::CancelSource) -> Result<RetainedExpectationReport, WorkflowError> {
        let (row, _, snapshot, owner) = self.expectation_summary(parent, cancel).await?;
        let report: ExpectationReport = execution::decode_report(row.original_report.as_bytes())?;
        if report.issued != row.issued as usize || report.completed_iterations != row.completed_iterations as usize || report.interrupted != row.interrupted || report.limit_reached != row.limit_reached || report.request.source.revision.as_id() != row.source_revision || report.request.source.physical_context != row.physical_context {
            return Err(contract("expectation recorded report disagrees with original summary scope"));
        }
        Ok(RetainedExpectationReport { report, owner: Arc::new((snapshot, owner)) })
    }
    /// Read an owned original report snapshot without retaining any scientific service.
    pub async fn expectation_report(&self, parent: StudyId) -> Result<RetainedExpectationReport, WorkflowError> {
        self.expectation_report_owned(parent, &crate::CancelSource::new()).await
    }
    /// Select original sample rows through the ordinary protected result reader.
    pub async fn expectation_samples(&self, parent: StudyId, iteration: usize, offset: usize, limit: usize, cancel: &crate::CancelSource) -> Result<FieldCheckedBatch, WorkflowError> {
        if limit == 0 || limit > 1024 { return Err(contract("expectation original sample page requires 1..=1024 rows")); }
        let (summary, publication, summary_snapshot, summary_owner) = self.expectation_summary(parent, cancel).await?;
        if iteration >= summary.recorded_iterations as usize { return Err(contract("expectation original iteration unavailable")); }
        let report: ExpectationReport = execution::decode_report(summary.original_report.as_bytes())?;
        let first = iteration.checked_mul(report.request.controls.samples_per_iteration).ok_or_else(|| contract("expectation original sample range"))?;
        let available = report.request.controls.samples_per_iteration.min(report.issued.saturating_sub(first));
        let start = first.checked_add(offset.min(available)).ok_or_else(|| contract("expectation original sample range"))?;
        let end = first.checked_add(offset.saturating_add(limit).min(available)).ok_or_else(|| contract("expectation original sample range"))?;
        let mut reader = self.results(&publication.run, &publication.attempt, "runtime.expectation_samples", start as u64, end as u64, cancel.token()).await?;
        let snapshot = reader.snapshot();
        let pool = self.pool(); let validation = self.validation_context(); let token = cancel.token();
        let mut columns = Collection::new(self.registry(), &pool, &token, &validation);
        columns.ensure::<samples::Row>().map_err(relation)?;
        while let Some(batch) = reader.next_relation_batch().await? {
            let copy = pse_columnar::MemoryConsumer::new("expectation:sample-row-copy").register(&pool);
            copy.try_grow(batch.batch().get_array_memory_size().checked_mul(4).ok_or_else(||contract("expectation sample copy extent"))?).map_err(pse_engine::EngineError::from)?;
            for row in samples::Row::rows(&batch).map_err(relation)? {
                if row.parent_id != parent || row.iteration != iteration as i64 || row.point_index < offset as i64 || row.point_index >= offset.saturating_add(limit) as i64 { return Err(contract("expectation original selected sample coordinates differ")); }
                columns.push(row).map_err(relation)?;
            }
        }
        columns.finish().map_err(relation)?.into_values().find(|batch|batch.relation_id()==samples::RELATION_ID).map(|batch|batch.with_export_owner(Arc::new((snapshot, summary_snapshot, summary_owner)))).ok_or_else(||contract("expectation original sample relation absent"))
    }
    /// Select original typed output estimates through recorded result protection.
    pub async fn expectation_estimates(&self, parent: StudyId, cancel: &crate::CancelSource) -> Result<FieldCheckedBatch, WorkflowError> {
        let (_, publication, summary_snapshot, summary_owner) = self.expectation_summary(parent, cancel).await?;
        let mut reader = self.results(&publication.run, &publication.attempt, "runtime.expectation_estimates", 0, u64::MAX, cancel.token()).await?;
        let snapshot = reader.snapshot();
        let pool = self.pool(); let validation = self.validation_context(); let token = cancel.token();
        let mut columns = Collection::new(self.registry(), &pool, &token, &validation);
        columns.ensure::<estimates::Row>().map_err(relation)?;
        while let Some(batch) = reader.next_relation_batch().await? {
            let copy = pse_columnar::MemoryConsumer::new("expectation:estimate-row-copy").register(&pool);
            copy.try_grow(batch.batch().get_array_memory_size().checked_mul(4).ok_or_else(||contract("expectation estimate copy extent"))?).map_err(pse_engine::EngineError::from)?;
            for row in estimates::Row::rows(&batch).map_err(relation)? {
                if row.parent_id != parent { return Err(contract("expectation original estimate parent differs")); }
                columns.push(row).map_err(relation)?;
            }
        }
        columns.finish().map_err(relation)?.into_values().find(|batch|batch.relation_id()==estimates::RELATION_ID).map(|batch|batch.with_export_owner(Arc::new((snapshot, summary_snapshot, summary_owner)))).ok_or_else(||contract("expectation original estimate relation absent"))
    }
}
impl Runtime {
    /// Read an original terminal report through source-free protected result capabilities.
    pub async fn expectation_report(&self, parent: StudyId) -> Result<RetainedExpectationReport, WorkflowError> { self.retained_data_services().expectation_report(parent).await }
    /// Read an exact original sample range without reconstructing a sampler or evaluator.
    pub async fn expectation_samples(&self, parent: StudyId, iteration: usize, offset: usize, limit: usize, cancel: &crate::CancelSource) -> Result<FieldCheckedBatch, WorkflowError> { self.retained_data_services().expectation_samples(parent,iteration,offset,limit,cancel).await }
    /// Read original typed output estimates without scientific services.
    pub async fn expectation_estimates(&self, parent: StudyId, cancel: &crate::CancelSource) -> Result<FieldCheckedBatch, WorkflowError> { self.retained_data_services().expectation_estimates(parent,cancel).await }
}
