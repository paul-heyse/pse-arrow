// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public bounded study browsing and immutable cohort analysis over protected original results.
use super::{RetainedDataServices, Runtime, WorkflowError, contract};
use pse_columnar::{CancellationToken, MemoryConsumer, MemoryReservation};
pub use pse_operations::canonical_study_workbench::{
    CapturedStudyRow, StudyCohortMember, StudyWorkbenchFilter, StudyWorkbenchHeader,
};
use std::sync::Arc;

pub use pse_operations::canonical_control::{
    StudyWorkbenchCursor, StudyWorkbenchPage, StudyWorkbenchRequest,
};
/// One complete bounded population selected before any analytical payload work.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyCohortRequest {
    /// Exact native predicates.
    pub filter: StudyWorkbenchFilter,
    /// Capture refuses if more studies match; maximum 64.
    pub maximum_studies: usize,
    /// Capture refuses if complete selected population exceeds this or the declared cohort admission bound.
    pub maximum_members: usize,
}
/// Historical acknowledgement of a complete native cut. Identifiers grant no read permission.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyCohortReceipt {
    /// Exact issuing canonical database.
    pub issuing_store: String,
    /// Exact capture identity; a serialized receipt is not its live capability.
    pub capture: String,
    /// Original immutable selection.
    pub request: StudyCohortRequest,
    /// Exact original header population.
    pub header_count: u64,
    /// Complete occurrence population, including every unavailable member.
    pub member_count: u64,
    /// True only after exact native complete capture settlement.
    pub complete: bool,
}
/// Capture-bound keyset continuation, never a floating membership selection.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyCohortCursor {
    /// Issuing canonical database.
    pub issuing_store: String,
    /// Exact retained cut.
    pub capture: String,
    /// Last consumed global capture position.
    pub after: u64,
}
/// An explicitly bounded page request against a live retained capture.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyCohortPageRequest {
    /// Header pages admit 1..=64; member/classification pages admit 1..=256.
    pub maximum: usize,
    /// Original capture-bound position, absent for the first page.
    pub cursor: Option<StudyCohortCursor>,
}
/// One bounded immutable header prefix with its actual retained allocation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyCohortHeaderPage {
    /// Original compact scientific context, in stable capture order.
    pub headers: Vec<CapturedStudyRow<StudyWorkbenchHeader>>,
    /// Exact next position, absent only at the captured count.
    pub next: Option<StudyCohortCursor>,
    #[serde(skip)]
    #[schemars(skip)]
    pub(super) _owner: Option<Arc<MemoryReservation>>,
}
/// One bounded immutable occurrence prefix, including original unavailable facts.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyCohortMemberPage {
    /// Original occurrences; global positions differ from within-study ordinals.
    pub members: Vec<CapturedStudyRow<StudyCohortMember>>,
    /// Exact next position, absent only at the captured count.
    pub next: Option<StudyCohortCursor>,
    #[serde(skip)]
    #[schemars(skip)]
    pub(super) _owner: Option<Arc<MemoryReservation>>,
}
/// One bounded NDJSON export line; the receipt is written only after every page.
#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(
    tag = "kind",
    content = "body",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum StudyCohortExportRecord {
    /// One original header prefix.
    Headers(StudyCohortHeaderPage),
    /// One original member prefix, including captured unavailable facts.
    Members(StudyCohortMemberPage),
    /// Final acknowledgement of complete page traversal.
    Receipt(StudyCohortReceipt),
}
/// Knowledge about the exact selected predicate, separate from captured lifecycle facts.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum StudyCohortAvailability {
    /// This usable captured member has not been fully examined by the predicate.
    Pending,
    /// Original captured facts or missing manifest definitively forbid this analysis.
    CapturedUnavailable,
    /// The exact selected original rows were fully examined and yielded no observation.
    EmptySelection,
    /// At least one original selected observation was consumed.
    Included,
}
/// Population knowledge never treats an unvisited bit as unavailable.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum StudyCohortScanState {
    /// No execution has started.
    Pending,
    /// At least one scan is active; completeness has not been established.
    Scanning,
    /// An interrupted/limited execution left members unexamined.
    Partial,
    /// One exact complete traversal examined every member.
    Complete,
}
/// Exact selected-predicate knowledge, including concurrent physical scan custody.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyCohortScanProgress {
    /// Completeness of population knowledge, independent of original operation outcomes.
    pub state: StudyCohortScanState,
    /// Started physical scans that have not drained; unpolled plans retain custody separately.
    pub active_scans: u64,
    /// Members with original selected observations.
    pub included: u64,
    /// Definitively unavailable or completely examined empty members.
    pub unavailable: u64,
    /// Members not yet completely examined; absence is never numerical zero.
    pub pending: u64,
}
/// Original member facts and exact selected-predicate availability.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyCohortClassification {
    /// Unmodified captured original facts.
    pub member: StudyCohortMember,
    /// Knowledge under this analysis's exact predicate.
    pub availability: StudyCohortAvailability,
}
/// Bounded classification facts and current knowledge, not a full population vector.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyCohortClassificationPage {
    /// Original members with explicit availability.
    pub members: Vec<CapturedStudyRow<StudyCohortClassification>>,
    /// Current exact predicate traversal knowledge.
    pub progress: StudyCohortScanProgress,
    /// Capture-bound next position.
    pub next: Option<StudyCohortCursor>,
    #[serde(skip)]
    #[schemars(skip)]
    pub(super) _owner: Option<pse_operations::transport::client::ResponseAllocation>,
}
/// Nonserializable exact capture capability retained through pages and physical scans.
#[derive(Clone, Debug)]
pub struct FrozenStudyCohort {
    pub(super) services: RetainedDataServices,
    receipt: StudyCohortReceipt,
    pub(super) capture: pse_operations::canonical_study_workbench::StudyCapture,
    _owner: Arc<MemoryReservation>,
}
/// A native relation over one exact captured population and selected scientific output.
/// Cloned providers retain original capture custody and shared traversal knowledge.
#[derive(Clone, Debug)]
pub struct StudyCohortRelation {
    state: Arc<super::cohort_provider::CohortAnalysisState>,
}
impl StudyCohortRelation {
    /// Exact physically typed output schema.
    pub fn schema(&self) -> datafusion::arrow::datatypes::SchemaRef {
        self.state.schema()
    }
    /// Original configured functions, settings and shared deployment pool.
    pub fn session_state(&self) -> datafusion::execution::session_state::SessionState {
        self.state.session_state()
    }
    /// Compose native queries without hydrating members into client-side loops.
    pub fn table_provider(&self) -> Arc<dyn datafusion::catalog::TableProvider> {
        self.state.table_provider()
    }
    /// Exact traversal knowledge; an early-closed scan does not classify its unread tail.
    pub fn progress(&self) -> StudyCohortScanProgress {
        self.state.progress()
    }
    /// Original member attribution and explicit pending/unavailable/empty distinctions.
    pub async fn classification_page(
        &self,
        request: StudyCohortPageRequest,
    ) -> Result<StudyCohortClassificationPage, WorkflowError> {
        self.state.classification_page(request).await
    }
}
/// One identically selected original scientific output across the immutable population.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyKpiRequest {
    /// Existing registered result relation.
    pub relation: String,
    /// Exact authored scientific output owner.
    pub owner: pse_ids::SemanticId,
    /// Existing declared numeric field.
    pub field: String,
    /// Exact original output partition.
    pub partition: String,
    /// Inclusive original relation row coordinate.
    pub start: u64,
    /// Exclusive original relation row coordinate.
    pub end: u64,
}
/// Complete native aggregate and compact exact population knowledge; no optimum claim.
#[derive(Debug)]
pub struct StudyKpiAnalysis {
    /// Historical exact capture receipt.
    pub cohort: StudyCohortReceipt,
    /// Count of members with actual selected observations.
    pub included: u64,
    /// Count of explicitly unavailable/empty members after complete traversal.
    pub unavailable: u64,
    /// Selected count, minimum, maximum and mean with original quantity/unit columns.
    pub batches: Vec<datafusion::arrow::record_batch::RecordBatch>,
    pub(super) state: Arc<super::cohort_provider::CohortAnalysisState>,
}
/// Exact compact result of an explicitly requested existing cohort-analysis export.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyKpiExport {
    /// Original frozen cohort receipt; not an acquisition capability.
    pub cohort: StudyCohortReceipt,
    /// Original included members.
    pub included: u64,
    /// Original unavailable members.
    pub unavailable: u64,
    /// Fresh explicit output; absent when the aggregate has no batches.
    pub output: Option<std::path::PathBuf>,
}
impl RetainedDataServices {
    /// Freeze, aggregate and export the selected existing study operation under one owner.
    /// Original capture/canonical guards remain live through encoding and synced output.
    /// # Errors
    /// Original capture/analysis, accounted memory, IPC and create-new/sync failures.
    pub async fn export_study_analysis(
        &self,
        request: StudyCohortRequest,
        kpi: &StudyKpiRequest,
        output: &std::path::Path,
        cancel: &CancellationToken,
    ) -> Result<StudyKpiExport, WorkflowError> {
        let cohort = self.freeze_study_cohort(request).await?;
        let analysis = cohort.aggregate(kpi, cancel.clone()).await?;
        let written = write_analysis_ipc(&self.pool(), &analysis.batches, output)?;
        Ok(StudyKpiExport {
            cohort: analysis.cohort,
            included: analysis.included,
            unavailable: analysis.unavailable,
            output: written.then(|| output.to_owned()),
        })
    }
}
fn write_analysis_ipc(
    pool: &Arc<dyn pse_columnar::MemoryPool>,
    batches: &[datafusion::arrow::record_batch::RecordBatch],
    output: &std::path::Path,
) -> Result<bool, WorkflowError> {
    use std::io::Write;
    if batches.is_empty() {
        return Ok(false);
    }
    let owner = MemoryConsumer::new("study:analysis-export").register(pool);
    owner
        .try_grow(
            pse_columnar::ipc_file::writer_preparation_extent(
                batches[0].schema().as_ref(),
                batches.len(),
            )
            .map_err(pse_columnar::CanonError::from)
            .map_err(pse_engine::EngineError::from)?,
        )
        .map_err(pse_engine::EngineError::from)?;
    let extent = pse_columnar::ipc_file::writer_extent(batches, 4 * 1024 * 1024)
        .map_err(pse_columnar::CanonError::from)
        .map_err(pse_engine::EngineError::from)?;
    owner
        .try_grow(
            extent
                .checked_sub(owner.size())
                .ok_or_else(|| contract("analysis writer extent"))?,
        )
        .map_err(pse_engine::EngineError::from)?;
    let content = pse_columnar::ipc_file::encode(batches, 4 * 1024 * 1024)
        .map_err(datafusion::error::DataFusionError::from)
        .map_err(pse_engine::EngineError::from)?;
    let io = |source| pse_engine::EngineError::Infrastructure {
        op: "export selected study aggregate".into(),
        source: Box::new(source),
    };
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(io)?;
    file.write_all(&content).map_err(io)?;
    file.sync_all().map_err(io)?;
    Ok(true)
}
impl StudyKpiAnalysis {
    /// Read one bounded original membership/classification page under retained custody.
    pub async fn classification_page(
        &self,
        request: StudyCohortPageRequest,
    ) -> Result<StudyCohortClassificationPage, WorkflowError> {
        self.state.classification_page(request).await
    }
    /// Exact predicate traversal knowledge and still-active executions.
    pub fn progress(&self) -> StudyCohortScanProgress {
        self.state.progress()
    }
}
impl Runtime {
    /// Freeze retained original study data without supplying scientific execution services.
    pub async fn freeze_study_cohort(
        &self,
        request: StudyCohortRequest,
    ) -> Result<FrozenStudyCohort, WorkflowError> {
        self.retained_data_services().freeze_study_cohort(request).await
    }
    /// Export analysis of original captured data through the retained-data owner.
    pub async fn export_study_analysis(
        &self,
        request: StudyCohortRequest,
        kpi: &StudyKpiRequest,
        output: &std::path::Path,
        cancel: &CancellationToken,
    ) -> Result<StudyKpiExport, WorkflowError> {
        self.retained_data_services()
            .export_study_analysis(request, kpi, output, cancel)
            .await
    }
}
impl RetainedDataServices {
    /// Settle one exact native cut and return only counts plus retained custody.
    pub async fn freeze_study_cohort(
        &self,
        request: StudyCohortRequest,
    ) -> Result<FrozenStudyCohort, WorkflowError> {
        if !(1..=64).contains(&request.maximum_studies)
            || !(1..=pse_schema::catalog::study_workbench::STUDY_COHORT_MAX_MEMBERS)
                .contains(&request.maximum_members)
        {
            return Err(contract("cohort narrow selection outside public admission"));
        }
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let response = super::connected_results::pooled_response_owner(
                    &self.pool(),
                    "study-workbench:capture-receipt",
                    pse_operations::canonical::MetadataResponse::Selection,
                )?;
                let selected = pse_operations::canonical::response_ownership_scope(
                    response.clone(),
                    self.canonical_store()?.capture_study_workbench(
                        &pse_operations::mint_id::<pse_ids::SemanticId>().to_string(),
                        &request.filter,
                        request.maximum_studies,
                        request.maximum_members,
                    ),
                )
                .await?;
                let receipt = StudyCohortReceipt {
                    issuing_store: self.canonical_store()?.issuing_store()?.into(),
                    capture: selected.capture.key().into(),
                    request,
                    header_count: selected.header_count,
                    member_count: selected.member_count,
                    complete: true,
                };
                let retained = selected
                    .capture
                    .custody_bytes()?
                    .checked_add(64 * 1024)
                    .ok_or_else(|| contract("cohort receipt extent"))?;
                let owner = Arc::new(super::connected_results::retain_protected_metadata(
                    &self.pool(),
                    response,
                    retained,
                    |owner| {
                        selected.capture.retain_metadata_owner(owner);
                        Ok(())
                    },
                )?);
                selected.capture.retain_metadata_owner(owner.clone());
                Ok(FrozenStudyCohort {
                    services: self.clone(),
                    receipt,
                    capture: selected.capture,
                    _owner: owner,
                })
            })
            .await
    }
}
impl FrozenStudyCohort {
    /// Publish bounded original metadata pages as NDJSON, then the compact final receipt.
    /// Cancellation or failure retains only `.incomplete`; an existing output refuses.
    pub async fn export_pages(
        &self,
        destination: &std::path::Path,
        maximum_bytes: u64,
        cancel: &CancellationToken,
    ) -> Result<u64, WorkflowError> {
        use std::io::Write;
        let owner =
            MemoryConsumer::new("study-cohort:page-export").register(&self.services.pool());
        owner
            .try_grow(64 * 1024)
            .map_err(pse_engine::EngineError::from)?;
        let io = |source| pse_engine::EngineError::Infrastructure {
            op: "publish frozen cohort pages".into(),
            source: Box::new(source),
        };
        let mut staging = destination.as_os_str().to_os_string();
        staging.push(".incomplete");
        let staging = std::path::PathBuf::from(staging);
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staging)
            .map_err(io)?;
        let mut output = std::io::BufWriter::new(super::connected_results::BoundedExportFile {
            file,
            remaining: maximum_bytes,
            written: 0,
            cancel: cancel.clone(),
        });
        let mut cursor = None;
        loop {
            cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
            let page = cancel
                .until_cancelled(self.header_page(StudyCohortPageRequest {
                    maximum: 64,
                    cursor,
                }))
                .await
                .map_err(pse_engine::EngineError::from)??;
            cursor = page.next.clone();
            write_page_line(&mut output, &StudyCohortExportRecord::Headers(page))?;
            if cursor.is_none() {
                break;
            }
        }
        let mut cursor = None;
        loop {
            cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
            let page = cancel
                .until_cancelled(self.member_page(StudyCohortPageRequest {
                    maximum: pse_schema::catalog::study_workbench::STUDY_CAPTURE_PAGE_ROWS,
                    cursor,
                }))
                .await
                .map_err(pse_engine::EngineError::from)??;
            cursor = page.next.clone();
            write_page_line(&mut output, &StudyCohortExportRecord::Members(page))?;
            if cursor.is_none() {
                break;
            }
        }
        cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
        write_page_line(
            &mut output,
            &StudyCohortExportRecord::Receipt(self.receipt.clone()),
        )?;
        output.flush().map_err(io)?;
        output.get_ref().file.sync_all().map_err(io)?;
        let bytes = output.get_ref().written;
        drop(output);
        cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
        std::fs::hard_link(&staging, destination).map_err(io)?;
        std::fs::remove_file(staging).map_err(io)?;
        Ok(bytes)
    }
    /// Compact historical facts. This borrowed document grants no payload permission.
    pub fn receipt(&self) -> &StudyCohortReceipt {
        &self.receipt
    }
    fn after(
        &self,
        request: &StudyCohortPageRequest,
        count: u64,
        maximum: usize,
    ) -> Result<Option<u64>, WorkflowError> {
        if request.maximum == 0 || request.maximum > maximum {
            return Err(contract("cohort page allowance outside bounds"));
        }
        if let Some(cursor) = &request.cursor
            && (cursor.issuing_store != self.receipt.issuing_store
                || cursor.capture != self.receipt.capture
                || cursor.after >= count)
        {
            return Err(contract("cohort cursor issuer or exact capture differs"));
        }
        Ok(request.cursor.as_ref().map(|cursor| cursor.after))
    }
    fn next(&self, last: Option<u64>, count: u64) -> Option<StudyCohortCursor> {
        last.filter(|last| last.checked_add(1).is_some_and(|next| next < count))
            .map(|after| StudyCohortCursor {
                issuing_store: self.receipt.issuing_store.clone(),
                capture: self.receipt.capture.clone(),
                after,
            })
    }
    /// Read one immutable original header page under this retained capability.
    pub async fn header_page(
        &self,
        request: StudyCohortPageRequest,
    ) -> Result<StudyCohortHeaderPage, WorkflowError> {
        let after = self.after(&request, self.receipt.header_count, 64)?;
        let response = super::connected_results::pooled_response_owner(
            &self.services.pool(),
            "study-workbench:captured-headers",
            pse_operations::canonical::MetadataResponse::StudyPage,
        )?;
        let page = super::connected_results::protected_response_scope(
            self.capture.clone(),
            response.clone(),
            self.capture.header_page(after, request.maximum),
        )
        .await?;
        if page
            .rows
            .last()
            .is_some_and(|row| row.position >= self.receipt.header_count)
            || page.rows.is_empty()
                && after.map_or(0, |value| value + 1) < self.receipt.header_count
        {
            return Err(contract("captured header page incomplete"));
        }
        let next = self.next(
            page.rows.last().map(|row| row.position),
            self.receipt.header_count,
        );
        let bytes = header_bytes(&page.rows)?
            .checked_add(16 * 1024)
            .ok_or_else(|| contract("header retained extent"))?;
        let owner = Arc::new(super::connected_results::retain_metadata(
            &self.services.pool(),
            response,
            bytes,
        )?);
        Ok(StudyCohortHeaderPage {
            headers: page.rows,
            next,
            _owner: Some(owner),
        })
    }
    /// Read one immutable occurrence page, retaining missing/failed/unattempted facts.
    pub async fn member_page(
        &self,
        request: StudyCohortPageRequest,
    ) -> Result<StudyCohortMemberPage, WorkflowError> {
        let after = self.after(
            &request,
            self.receipt.member_count,
            pse_schema::catalog::study_workbench::STUDY_CAPTURE_PAGE_ROWS,
        )?;
        let response = super::connected_results::pooled_response_owner(
            &self.services.pool(),
            "study-workbench:captured-members",
            pse_operations::canonical::MetadataResponse::StudyPage,
        )?;
        let page = super::connected_results::protected_response_scope(
            self.capture.clone(),
            response.clone(),
            self.capture.member_page(after, request.maximum),
        )
        .await?;
        if page
            .rows
            .last()
            .is_some_and(|row| row.position >= self.receipt.member_count)
            || page.rows.is_empty()
                && after.map_or(0, |value| value + 1) < self.receipt.member_count
        {
            return Err(contract("captured member page incomplete"));
        }
        let next = self.next(
            page.rows.last().map(|row| row.position),
            self.receipt.member_count,
        );
        let bytes = member_bytes(&page.rows)?
            .checked_add(16 * 1024)
            .ok_or_else(|| contract("member retained extent"))?;
        let owner = Arc::new(super::connected_results::retain_metadata(
            &self.services.pool(),
            response,
            bytes,
        )?);
        Ok(StudyCohortMemberPage {
            members: page.rows,
            next,
            _owner: Some(owner),
        })
    }
    /// One explicitly selected detail reader retains its original independent permission.
    pub async fn result(
        &self,
        position: usize,
        relation: &str,
        start: u64,
        end: u64,
        cancel: CancellationToken,
    ) -> Result<super::CanonicalResultReader, WorkflowError> {
        let position =
            u64::try_from(position).map_err(|_| contract("cohort member coordinate extent"))?;
        if position >= self.receipt.member_count {
            return Err(contract("cohort member absent"));
        }
        let page = self
            .member_page(StudyCohortPageRequest {
                maximum: 1,
                cursor: position.checked_sub(1).map(|after| StudyCohortCursor {
                    issuing_store: self.receipt.issuing_store.clone(),
                    capture: self.receipt.capture.clone(),
                    after,
                }),
            })
            .await?;
        let point = &page
            .members
            .first()
            .ok_or_else(|| contract("cohort member absent"))?
            .value;
        let manifest = point
            .manifest
            .as_deref()
            .ok_or_else(|| contract("cohort member had no closed result at capture"))?;
        let attempt = point
            .attempt
            .as_deref()
            .ok_or_else(|| contract("cohort member has no attempted result"))?;
        let reader = self
            .services
            .results(&point.run, attempt, relation, start, end, cancel)
            .await?;
        if reader.selection().manifest().key != manifest
            || reader.selection().attempt().key != attempt
            || reader.selection().run().key != point.run
        {
            return Err(contract(
                "detail reader differs from captured exact manifest",
            ));
        }
        reader.retain_capture(self.capture.clone())?;
        Ok(reader)
    }
    /// Select a composable original relation without per-member snapshots.
    pub async fn relation(
        &self,
        request: StudyKpiRequest,
        cancel: CancellationToken,
    ) -> Result<StudyCohortRelation, WorkflowError> {
        cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
        let headers = self
            .header_page(StudyCohortPageRequest {
                maximum: 64,
                cursor: None,
            })
            .await?;
        compatible_rows(&headers.headers)?;
        let state = super::cohort_provider::CohortAnalysisState::new(
            self.clone(),
            request,
            cancel,
        )?;
        Ok(StudyCohortRelation { state })
    }
    /// Aggregate through the same public native relation and original capture custody.
    pub async fn aggregate(
        &self,
        request: &StudyKpiRequest,
        cancel: CancellationToken,
    ) -> Result<StudyKpiAnalysis, WorkflowError> {
        use datafusion::{
            functions_aggregate::expr_fn::{avg, count, max, min},
            prelude::{SessionContext, col},
        };
        let relation = self.relation(request.clone(), cancel.clone()).await?;
        let context = SessionContext::new_with_state(relation.session_state());
        let frame = context
            .read_table(relation.table_provider())
            .map_err(pse_engine::EngineError::from)?;
        let query = frame
            .aggregate(
                vec![col("quantity_id"), col("unit_id")],
                vec![
                    count(col(&request.field)).alias("count"),
                    min(col(&request.field)).alias("minimum"),
                    max(col(&request.field)).alias("maximum"),
                    avg(col(&request.field)).alias("mean"),
                ],
            )
            .map_err(pse_engine::EngineError::from)?;
        let batches = cancel
            .until_cancelled(query.collect())
            .await
            .map_err(pse_engine::EngineError::from)?
            .map_err(pse_engine::EngineError::from)?;
        let progress = relation.progress();
        if progress.state != StudyCohortScanState::Complete || progress.pending != 0 {
            return Err(contract("cohort aggregate traversal incomplete"));
        }
        if batches.iter().map(|batch| batch.num_rows()).sum::<usize>() > 1 {
            return Err(contract(
                "KPI outputs have incompatible actual quantity, basis, reference state or units",
            ));
        }
        Ok(StudyKpiAnalysis {
            cohort: self.receipt.clone(),
            included: progress.included,
            unavailable: progress.unavailable,
            batches,
            state: relation.state,
        })
    }
}
fn write_page_line(
    output: &mut impl std::io::Write,
    record: &StudyCohortExportRecord,
) -> Result<(), WorkflowError> {
    use std::io::Write;
    struct Line<'a, W> {
        output: &'a mut W,
        remaining: usize,
    }
    impl<W: Write> Write for Line<'_, W> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > self.remaining {
                return Err(std::io::Error::other(
                    "cohort page line allowance exhausted",
                ));
            }
            let written = self.output.write(bytes)?;
            self.remaining -= written;
            Ok(written)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            self.output.flush()
        }
    }
    let mut line = Line {
        output,
        remaining: 8 * pse_schema::catalog::study_workbench::STUDY_PAGE_BYTES,
    };
    serde_json::to_writer(&mut line, record).map_err(|error| contract(error.to_string()))?;
    line.write_all(b"\n")
        .map_err(|source| pse_engine::EngineError::Infrastructure {
            op: "write cohort page line".into(),
            source: Box::new(source),
        })?;
    Ok(())
}
fn member_bytes(rows: &Vec<CapturedStudyRow<StudyCohortMember>>) -> Result<usize, WorkflowError> {
    let mut bytes = rows
        .capacity()
        .checked_mul(size_of::<CapturedStudyRow<StudyCohortMember>>())
        .ok_or_else(|| contract("member page extent"))?;
    for row in rows {
        let member = &row.value;
        for value in [
            Some(&member.key),
            Some(&member.study),
            Some(&member.run),
            member.attempt.as_ref(),
            member.manifest.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            bytes = bytes
                .checked_add(value.capacity())
                .ok_or_else(|| contract("member page strings extent"))?;
        }
    }
    Ok(bytes)
}
fn header_bytes(
    rows: &Vec<CapturedStudyRow<StudyWorkbenchHeader>>,
) -> Result<usize, WorkflowError> {
    let mut bytes = rows
        .capacity()
        .checked_mul(size_of::<CapturedStudyRow<StudyWorkbenchHeader>>())
        .ok_or_else(|| contract("header page extent"))?;
    for row in rows {
        let header = &row.value;
        for value in [
            Some(&header.study),
            Some(&header.run),
            Some(&header.problem),
            Some(&header.source),
            header.physical_revision.as_ref(),
            header.physical_identity.as_ref(),
            header.context.as_ref(),
            header.result_attempt.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            bytes = bytes
                .checked_add(value.capacity())
                .ok_or_else(|| contract("header page strings extent"))?;
        }
    }
    Ok(bytes)
}
fn compatible_rows(rows: &[CapturedStudyRow<StudyWorkbenchHeader>]) -> Result<(), WorkflowError> {
    if let Some(first) = rows.first().map(|row| &row.value)
        && (first.physical_revision.is_none()
            || first.physical_identity.is_none()
            || rows.iter().any(|row| {
                row.value.physical_revision != first.physical_revision
                    || row.value.physical_identity != first.physical_identity
            }))
    {
        return Err(contract(
            "KPI source or physical meaning differs or is unavailable; select compatible exact sources or an admitted transform",
        ));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn project_analysis_export_preserves_exact_ipc_empty_and_existing_file_rules() {
        use datafusion::arrow::{
            array::{ArrayRef, Int64Array},
            datatypes::{DataType, Field, Schema},
            ipc::reader::FileReader,
            record_batch::RecordBatch,
        };
        let directory = tempfile::tempdir().unwrap();
        let destination = directory.path().join("aggregate.arrow");
        let pool: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(16 << 20));
        assert!(!write_analysis_ipc(&pool, &[], &destination).unwrap());
        assert!(!destination.exists());
        let counts: ArrayRef = Arc::new(Int64Array::from(vec![3, 5]));
        let batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                "original_count",
                DataType::Int64,
                false,
            )])),
            vec![counts],
        )
        .unwrap();
        let refused: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(0));
        assert!(write_analysis_ipc(&refused, std::slice::from_ref(&batch), &destination).is_err());
        assert!(
            !destination.exists(),
            "working refusal precedes output creation"
        );
        assert_eq!(refused.reserved(), 0);
        assert!(write_analysis_ipc(&pool, &[batch], &destination).unwrap());
        let original = std::fs::read(&destination).unwrap();
        let mut reader = FileReader::try_new(std::io::Cursor::new(&original), None).unwrap();
        let observed = reader.next().unwrap().unwrap();
        assert_eq!(observed.num_rows(), 2);
        assert_eq!(observed.schema().field(0).name(), "original_count");
        assert_eq!(
            observed
                .column(0)
                .as_any()
                .downcast_ref::<Int64Array>()
                .unwrap()
                .values(),
            &[3, 5]
        );
        assert!(reader.next().is_none());
        let batch = RecordBatch::new_empty(observed.schema());
        assert!(write_analysis_ipc(&pool, &[batch], &destination).is_err());
        assert_eq!(std::fs::read(&destination).unwrap(), original);
        assert_eq!(pool.reserved(), 0);
    }

    #[test]
    fn cohort_export_names_both_actual_generic_row_instantiations() {
        let schema = schemars::schema_for!(StudyCohortExportRecord).to_value();
        let definitions = schema["$defs"].as_object().unwrap();
        assert!(definitions.contains_key("CapturedStudyRowForStudyWorkbenchHeader"));
        assert!(definitions.contains_key("CapturedStudyRowForStudyCohortMember"));
        for alternative in schema["oneOf"].as_array().unwrap() {
            assert_eq!(
                alternative["additionalProperties"], false,
                "each export line is a closed owner-declared envelope"
            );
        }
    }
    #[test]
    fn workbench_refuses_incompatible_kpi_meaning_before_payload_access() {
        let mut h:StudyWorkbenchHeader=serde_json::from_value(serde_json::json!({"study":"s","run":"r","problem":"p","source":"revision","physical_revision":"physical","physical_identity":"identity","context":null,"point_count":3,"generation":0,"active":true,"cancelled":false,"terminal":false,"result_attempt":null,"outcome":null})).unwrap();
        let row = |value| CapturedStudyRow { position: 0, value };
        assert!(compatible_rows(&[row(h.clone()), row(h.clone())]).is_ok());
        h.physical_revision = Some("other-unit-system".into());
        let mut original = h.clone();
        original.physical_revision = Some("physical".into());
        assert!(compatible_rows(&[row(original), row(h)]).is_err());
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
#[path = "study_workbench_tests.rs"]
pub(in crate::workflow) mod canonical_tests;
