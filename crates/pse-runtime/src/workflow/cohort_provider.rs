// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One lazy relation scan over exact captured original manifests, with bounded pages.
use super::{
    SourceServices, WorkflowError,
    connected_results::{self, OutputFilter, OutputRows},
    contract,
    result_snapshot::{NativeResultSelection, ResultBlockDecode, ResultPayloadAuthority},
    study_workbench::*,
};
use datafusion::{
    arrow::datatypes::SchemaRef,
    catalog::{Session, TableProvider, streaming::StreamingTable},
    common::{DataFusionError, Result as NativeResult},
    execution::{TaskContext, session_state::SessionState},
    logical_expr::{Expr, TableProviderFilterPushDown, TableType},
    physical_plan::{
        ExecutionPlan, SendableRecordBatchStream, stream::RecordBatchStreamAdapter,
        streaming::PartitionStream,
    },
};
use pse_columnar::{CancellationToken, MemoryConsumer, MemoryReservation};
use pse_model::{
    HeapUsage,
    generated::runtime::{
        canonical_result_batches::Row as Batch, canonical_result_blocks::Row as Block,
    },
};
use pse_operations::{
    canonical::MetadataResponse,
    canonical_capture_results::{CaptureResultCursor, CaptureResultLocators, CaptureResultRequest},
};
use std::{
    collections::{BTreeMap, VecDeque},
    sync::{Arc, Mutex},
};

#[derive(Debug)]
struct Knowledge {
    bits: Vec<u8>,
    count: u64,
    active: u64,
    started: bool,
    complete: bool,
    _owner: MemoryReservation,
}
impl Knowledge {
    fn get(&self, position: u64) -> StudyCohortAvailability {
        let byte = self.bits[(position / 4) as usize];
        match (byte >> ((position % 4) * 2)) & 3 {
            0 => StudyCohortAvailability::Pending,
            1 => StudyCohortAvailability::CapturedUnavailable,
            2 => StudyCohortAvailability::EmptySelection,
            _ => StudyCohortAvailability::Included,
        }
    }
    fn mark(&mut self, position: u64, value: StudyCohortAvailability) -> Result<(), WorkflowError> {
        if position >= self.count {
            return Err(contract("classification coordinate outside captured count"));
        }
        let old = self.get(position);
        if old == StudyCohortAvailability::Included {
            return if value == StudyCohortAvailability::CapturedUnavailable {
                Err(contract(
                    "included member conflicts with captured unavailable facts",
                ))
            } else {
                Ok(())
            };
        }
        if old == StudyCohortAvailability::CapturedUnavailable && value != old {
            return Err(contract("captured unavailable member was reclassified"));
        }
        let code = match value {
            StudyCohortAvailability::Pending => 0,
            StudyCohortAvailability::CapturedUnavailable => 1,
            StudyCohortAvailability::EmptySelection => 2,
            StudyCohortAvailability::Included => 3,
        };
        let shift = (position % 4) * 2;
        let byte = &mut self.bits[(position / 4) as usize];
        *byte = (*byte & !(3 << shift)) | (code << shift);
        Ok(())
    }
    fn progress(&self) -> StudyCohortScanProgress {
        let mut included = 0;
        let mut unavailable = 0;
        let mut pending = 0;
        for position in 0..self.count {
            match self.get(position) {
                StudyCohortAvailability::Pending => pending += 1,
                StudyCohortAvailability::Included => included += 1,
                _ => unavailable += 1,
            }
        }
        StudyCohortScanProgress {
            state: if self.complete {
                StudyCohortScanState::Complete
            } else if self.active > 0 {
                StudyCohortScanState::Scanning
            } else if self.started {
                StudyCohortScanState::Partial
            } else {
                StudyCohortScanState::Pending
            },
            active_scans: self.active,
            included,
            unavailable,
            pending,
        }
    }
}
/// Exact immutable predicate, one protected decoder, and compact traversal knowledge.
#[derive(Debug)]
pub(super) struct CohortAnalysisState {
    cohort: FrozenStudyCohort,
    request: StudyKpiRequest,
    services: SourceServices,
    filter: Arc<OutputFilter>,
    native: Option<Arc<NativeResultSelection>>,
    decode: Arc<ResultBlockDecode>,
    relation: pse_ids::SemanticId,
    schema: SchemaRef,
    cancel: CancellationToken,
    knowledge: Mutex<Knowledge>,
    _owner: MemoryReservation,
}
impl CohortAnalysisState {
    pub(super) fn new(
        cohort: FrozenStudyCohort,
        request: StudyKpiRequest,
        cancel: CancellationToken,
    ) -> Result<Arc<Self>, WorkflowError> {
        if request.start > request.end {
            return Err(contract("KPI row range reversed"));
        }
        let services = cohort.runtime.source_services();
        let spec = services
            .preparation()
            .registry()
            .relation(&request.relation)
            .ok_or_else(|| contract("unknown KPI relation"))?;
        if !spec
            .columns
            .iter()
            .any(|column| column.name() == "quantity_id")
            || !spec.columns.iter().any(|column| column.name() == "unit_id")
        {
            return Err(contract(
                "KPI relation does not retain complete quantity and unit meaning; use an original physically typed output",
            ));
        }
        let relation = spec.id;
        let schema =
            pse_schema::arrow::relation_schema_ref(services.preparation().registry(), spec)
                .map_err(|error| pse_engine::EngineError::Semantic(Arc::new(error)))?;
        let filter = connected_results::output_filter(
            &services,
            &request.relation,
            request.owner,
            &request.field,
            &request.partition,
            None,
            None,
            None,
        )?;
        let native = if filter.predicate.dense() {
            None
        } else {
            let expression = super::result_projection::scalar_selection_expression(
                relation,
                request.owner,
                &request.partition,
                &request.field,
                filter.predicate,
            )?;
            Some(
                NativeResultSelection::new(
                    &services,
                    relation,
                    schema.clone(),
                    &services.preparation().bound_model().native_state(),
                    expression,
                )
                .map_err(pse_engine::EngineError::from)?,
            )
        };
        let count = cohort.receipt().member_count;
        let bytes = usize::try_from(
            count
                .checked_add(3)
                .ok_or_else(|| contract("classification bit extent"))?
                / 4,
        )
        .map_err(|_| contract("classification bit extent"))?;
        let knowledge_owner = MemoryConsumer::new("study-cohort:classification-bits")
            .register(&services.preparation().pool());
        knowledge_owner
            .try_grow(bytes + size_of::<Knowledge>())
            .map_err(pse_engine::EngineError::from)?;
        let owner = MemoryConsumer::new("study-cohort:relation-provider")
            .register(&services.preparation().pool());
        owner
            .try_grow(
                size_of::<Self>()
                    + request.relation.capacity()
                    + request.field.capacity()
                    + request.partition.capacity()
                    + 4096,
            )
            .map_err(pse_engine::EngineError::from)?;
        let decode = ResultBlockDecode::new(&services, relation, schema.clone())?;
        Ok(Arc::new(Self {
            cohort,
            request,
            services,
            filter,
            native,
            decode,
            relation,
            schema,
            cancel,
            knowledge: Mutex::new(Knowledge {
                bits: vec![0; bytes],
                count,
                active: 0,
                started: false,
                complete: false,
                _owner: knowledge_owner,
            }),
            _owner: owner,
        }))
    }
    fn knowledge(&self) -> std::sync::MutexGuard<'_, Knowledge> {
        self.knowledge
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    pub(super) fn progress(&self) -> StudyCohortScanProgress {
        self.knowledge().progress()
    }
    pub(super) fn table_provider(self: &Arc<Self>) -> Arc<dyn TableProvider> {
        Arc::new(CohortProvider(self.clone()))
    }
    pub(super) async fn classification_page(
        &self,
        request: StudyCohortPageRequest,
    ) -> Result<StudyCohortClassificationPage, WorkflowError> {
        let page = self.cohort.member_page(request).await?;
        let owner = MemoryConsumer::new("study-cohort:classification-page")
            .register(&self.services.preparation().pool());
        owner
            .try_grow(
                page.members.len() * size_of::<CapturedStudyRow<StudyCohortClassification>>()
                    + 4096,
            )
            .map_err(pse_engine::EngineError::from)?;
        let mut members = Vec::with_capacity(page.members.len());
        let mut knowledge = self.knowledge();
        for row in page.members {
            if !usable(&row.value) {
                knowledge.mark(row.position, StudyCohortAvailability::CapturedUnavailable)?;
            }
            let availability = knowledge.get(row.position);
            members.push(CapturedStudyRow {
                position: row.position,
                value: StudyCohortClassification {
                    member: row.value,
                    availability,
                },
            });
        }
        let progress = knowledge.progress();
        drop(knowledge);
        Ok(StudyCohortClassificationPage {
            members,
            progress,
            next: page.next,
            _owner: Some(match page._owner {
                Some(page_owner) => pse_operations::transport::client::ResponseAllocation::combine(
                    page_owner,
                    Arc::new(owner),
                ),
                None => Arc::new(owner).into(),
            }),
        })
    }
}
fn usable(member: &StudyCohortMember) -> bool {
    member.facts.scientific.usable && member.attempt.is_some() && member.manifest.is_some()
}

#[derive(Debug)]
struct ScanGuard {
    state: Arc<CohortAnalysisState>,
}
impl ScanGuard {
    fn new(state: Arc<CohortAnalysisState>) -> Self {
        {
            let mut knowledge = state.knowledge();
            knowledge.started = true;
            knowledge.active += 1;
        }
        Self { state }
    }
    fn complete(&self) -> Result<(), WorkflowError> {
        let mut knowledge = self.state.knowledge();
        if knowledge.progress().pending != 0 {
            return Err(contract("completed cohort scan retained unvisited members"));
        }
        knowledge.complete = true;
        Ok(())
    }
}
impl Drop for ScanGuard {
    fn drop(&mut self) {
        let mut knowledge = self.state.knowledge();
        knowledge.active -= 1;
    }
}

#[derive(Debug)]
struct Pending {
    position: u64,
    read: Arc<pse_operations::canonical_capture_results::CapturedResultRead>,
    block: Block,
    batch: Batch,
    output: OutputRows,
}
#[derive(Debug)]
struct Scan {
    state: Arc<CohortAnalysisState>,
    _guard: ScanGuard,
    members: Option<StudyCohortMemberPage>,
    member_cursor: Option<StudyCohortCursor>,
    members_finished: bool,
    selected: Vec<u64>,
    cursor: CaptureResultCursor,
    locators_finished: bool,
    pending: VecDeque<Pending>,
    finished: Vec<u64>,
    page_owner: Option<MemoryReservation>,
    _owner: MemoryReservation,
}
impl Scan {
    fn new(state: Arc<CohortAnalysisState>) -> Result<Self, WorkflowError> {
        let owner = MemoryConsumer::new("study-cohort:scan-state")
            .register(&state.services.preparation().pool());
        owner
            .try_grow(
                size_of::<Self>()
                    + 4096
                    + pse_schema::catalog::study_workbench::STUDY_CAPTURE_PAGE_ROWS
                        * size_of::<u64>(),
            )
            .map_err(pse_engine::EngineError::from)?;
        Ok(Self {
            _guard: ScanGuard::new(state.clone()),
            state,
            members: None,
            member_cursor: None,
            members_finished: false,
            selected: Vec::new(),
            cursor: CaptureResultCursor::default(),
            locators_finished: true,
            pending: VecDeque::new(),
            finished: Vec::new(),
            page_owner: None,
            _owner: owner,
        })
    }
    fn finish_sources(&mut self) -> Result<(), WorkflowError> {
        let mut knowledge = self.state.knowledge();
        for position in self.finished.drain(..) {
            knowledge.mark(position, StudyCohortAvailability::EmptySelection)?;
        }
        Ok(())
    }
    async fn load_members(&mut self) -> Result<bool, WorkflowError> {
        self.members = None;
        self.selected.clear();
        if self.members_finished {
            return Ok(false);
        }
        let page = self
            .state
            .cohort
            .member_page(StudyCohortPageRequest {
                maximum: pse_schema::catalog::study_workbench::STUDY_CAPTURE_PAGE_ROWS,
                cursor: self.member_cursor.take(),
            })
            .await?;
        self.member_cursor = page.next.clone();
        self.members_finished = page.next.is_none();
        {
            let mut knowledge = self.state.knowledge();
            for row in &page.members {
                if usable(&row.value) {
                    self.selected.push(row.position);
                } else {
                    knowledge.mark(row.position, StudyCohortAvailability::CapturedUnavailable)?;
                }
            }
        }
        self.members = Some(page);
        self.cursor = CaptureResultCursor::default();
        self.locators_finished = self.selected.is_empty();
        Ok(true)
    }
    async fn load_locators(&mut self) -> Result<(), WorkflowError> {
        self.pending = VecDeque::new();
        self.page_owner = None;
        let response = connected_results::pooled_response_owner(
            &self.state.services.preparation().pool(),
            "study-cohort:locator-response",
            MetadataResponse::CaptureLocators,
        )?;
        let request = CaptureResultRequest {
            relation: self.state.relation.to_string(),
            output: self.state.filter.output.clone(),
            partition: self.state.filter.partition.clone(),
            dense: self.state.filter.predicate.dense(),
            start: self.state.request.start,
            end: self.state.request.end,
        };
        let page = connected_results::protected_response_scope(
            self.state.cohort.capture.clone(),
            response.clone(),
            self.state
                .cohort
                .capture
                .result_page(&self.selected, &request, self.cursor),
        )
        .await?;
        self.cursor = page.next;
        self.locators_finished = page.exhausted;
        let members = self
            .members
            .as_ref()
            .ok_or_else(|| contract("cohort locator page lost member custody"))?;
        for source in page.sources {
            let member = &members
                .members
                .iter()
                .find(|row| row.position == source.position)
                .ok_or_else(|| contract("locator outside captured member page"))?
                .value;
            if member.manifest.as_deref() != Some(source.manifest.key.as_str())
                || member.attempt.as_deref() != Some(source.manifest.attempt.as_str())
            {
                return Err(contract(
                    "locator manifest differs from exact captured reference",
                ));
            }
            let Some(locators) = source.locators else {
                return Err(contract("relation is not part of selected attempt"));
            };
            let read = Arc::new(
                source
                    .read
                    .ok_or_else(|| contract("captured closed-set permission absent"))?,
            );
            let permission_owner = MemoryConsumer::new("study-cohort:closed-set-permission")
                .register(&self.state.services.preparation().pool());
            permission_owner
                .try_grow(
                    read.retained_bytes()?
                        .checked_add(size_of::<MemoryReservation>() + 4 * size_of::<usize>())
                        .ok_or_else(|| contract("captured permission metadata extent"))?,
                )
                .map_err(pse_engine::EngineError::from)?;
            read.retain_metadata_owner(Arc::new(permission_owner))?;
            let (blocks, batches, mut outputs, finished) = match locators {
                CaptureResultLocators::Scalar(page) => {
                    (page.blocks, page.batches, None, page.exhausted)
                }
                CaptureResultLocators::Dense(page) => {
                    let mut outputs = BTreeMap::new();
                    for output in page.outputs {
                        if outputs.insert(output.batch.clone(), output).is_some() {
                            return Err(contract("duplicate dense output index"));
                        }
                    }
                    (page.blocks, page.batches, Some(outputs), page.exhausted)
                }
            };
            if finished {
                self.finished.push(source.position);
            }
            for (block, batch) in blocks.into_iter().zip(batches) {
                let output = if let Some(outputs) = &mut outputs {
                    OutputRows::Dense(
                        outputs
                            .remove(&block.batch)
                            .ok_or_else(|| contract("cohort dense block lacks exact output"))?,
                    )
                } else {
                    OutputRows::Scalar
                };
                self.pending.push_back(Pending {
                    position: source.position,
                    read: read.clone(),
                    block,
                    batch,
                    output,
                });
            }
            if outputs.is_some_and(|outputs| !outputs.is_empty()) {
                return Err(contract("cohort output index has no original block"));
            }
        }
        let mut bytes = self.pending.capacity() * size_of::<Pending>()
            + self.finished.capacity() * size_of::<u64>()
            + 4096;
        for item in &self.pending {
            bytes = bytes
                .checked_add(
                    item.block.heap_bytes()
                        + item.batch.heap_bytes()
                        + match &item.output {
                            OutputRows::Scalar => 0,
                            OutputRows::Dense(output) => output.heap_bytes(),
                        },
                )
                .ok_or_else(|| contract("cohort locator extent"))?;
        }
        self.page_owner = Some(connected_results::retain_metadata(
            &self.state.services.preparation().pool(),
            response,
            bytes,
        )?);
        Ok(())
    }
    async fn next_body(
        &mut self,
    ) -> Result<Option<datafusion::arrow::record_batch::RecordBatch>, WorkflowError> {
        loop {
            self.state
                .cancel
                .checkpoint()
                .map_err(pse_engine::EngineError::from)?;
            if let Some(item) = self.pending.pop_front() {
                if item.block.columns != self.state.schema.fields().len() as u64 {
                    return Err(contract("recorded cohort block shape differs"));
                }
                let checked = self
                    .state
                    .decode
                    .read_block(
                        ResultPayloadAuthority::Capture { read: item.read },
                        &item.block,
                        &item.batch,
                        &self.state.cancel,
                    )
                    .await?;
                let Some((selected, _rows)) = connected_results::select_original_rows(
                    &self.state.services,
                    connected_results::OriginalRowSelection {
                        relation: self.state.relation,
                        filter: &self.state.filter,
                        native: self.state.native.as_deref(),
                    },
                    &item.block,
                    item.output,
                    checked,
                    self.state.request.start..self.state.request.end,
                    &self.state.cancel,
                )?
                else {
                    continue;
                };
                if selected.batch().num_rows() == 0 {
                    continue;
                }
                self.state
                    .knowledge()
                    .mark(item.position, StudyCohortAvailability::Included)?;
                return selected
                    .checked_export(&self.state.cancel)
                    .map(Some)
                    .map_err(super::relation);
            }
            self.finish_sources()?;
            self.page_owner = None;
            if !self.locators_finished {
                self.load_locators().await?;
                continue;
            }
            if !self.load_members().await? {
                self._guard.complete()?;
                return Ok(None);
            }
        }
    }
    fn next(
        &mut self,
    ) -> Result<
        connected_results::PrepaidReaderFuture<
            '_,
            Result<Option<datafusion::arrow::record_batch::RecordBatch>, WorkflowError>,
        >,
        WorkflowError,
    > {
        let pool = self.state.services.preparation().pool();
        connected_results::prepaid_reader_future(&pool, self.next_body())
    }
}

#[derive(Debug)]
struct CohortProvider(Arc<CohortAnalysisState>);
#[async_trait::async_trait]
impl TableProvider for CohortProvider {
    fn schema(&self) -> SchemaRef {
        self.0.schema.clone()
    }
    fn table_type(&self) -> TableType {
        TableType::Temporary
    }
    fn supports_filters_pushdown(
        &self,
        filters: &[&Expr],
    ) -> NativeResult<Vec<TableProviderFilterPushDown>> {
        Ok(vec![
            TableProviderFilterPushDown::Unsupported;
            filters.len()
        ])
    }
    async fn scan(
        &self,
        state: &dyn Session,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        limit: Option<usize>,
    ) -> NativeResult<Arc<dyn ExecutionPlan>> {
        let state = state
            .as_any()
            .downcast_ref::<SessionState>()
            .ok_or_else(|| {
                DataFusionError::Plan("cohort requires configured native session state".into())
            })?;
        if !Arc::ptr_eq(
            &state.runtime_env().memory_pool,
            &self.0.services.preparation().pool(),
        ) {
            return Err(DataFusionError::Plan(
                "cohort query uses another deployment pool".into(),
            ));
        }
        StreamingTable::try_new(
            self.schema(),
            vec![Arc::new(CohortPartition(self.0.clone()))],
        )?
        .scan(
            state,
            projection,
            &[],
            if filters.is_empty() { limit } else { None },
        )
        .await
    }
}
#[derive(Debug)]
struct CohortPartition(Arc<CohortAnalysisState>);
impl PartitionStream for CohortPartition {
    fn schema(&self) -> &SchemaRef {
        &self.0.schema
    }
    fn execute(&self, context: Arc<TaskContext>) -> SendableRecordBatchStream {
        let state = self.0.clone();
        let matches = Arc::ptr_eq(
            &context.runtime_env().memory_pool,
            &state.services.preparation().pool(),
        );
        let stream = futures_util::stream::try_unfold(
            (state, None::<Scan>, matches),
            |(state, scan, matches)| async move {
                if !matches {
                    return Err(DataFusionError::Execution(
                        "cohort execution uses another deployment pool".into(),
                    ));
                }
                let mut scan = match scan {
                    Some(scan) => scan,
                    None => Scan::new(state.clone()).map_err(pse_columnar::external)?,
                };
                let Some(batch) = scan
                    .next()
                    .map_err(pse_columnar::external)?
                    .await
                    .map_err(pse_columnar::external)?
                else {
                    return Ok(None);
                };
                Ok(Some((batch, (state, Some(scan), matches))))
            },
        );
        Box::pin(RecordBatchStreamAdapter::new(self.0.schema.clone(), stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn knowledge(count: u64) -> Knowledge {
        let pool: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(4096));
        let owner = MemoryConsumer::new("classification-control").register(&pool);
        owner.try_grow(1024).unwrap();
        Knowledge {
            bits: vec![0; count.div_ceil(4) as usize],
            count,
            active: 0,
            started: false,
            complete: false,
            _owner: owner,
        }
    }
    #[test]
    fn cohort_unvisited_and_partial_scans_never_become_unavailable() {
        let mut knowledge = knowledge(9);
        assert_eq!(knowledge.progress().state, StudyCohortScanState::Pending);
        assert_eq!(knowledge.progress().pending, 9);
        knowledge.started = true;
        knowledge.active = 2;
        knowledge
            .mark(0, StudyCohortAvailability::CapturedUnavailable)
            .unwrap();
        knowledge
            .mark(4, StudyCohortAvailability::EmptySelection)
            .unwrap();
        knowledge
            .mark(8, StudyCohortAvailability::Included)
            .unwrap();
        assert_eq!(knowledge.progress().state, StudyCohortScanState::Scanning);
        assert_eq!(knowledge.progress().pending, 6);
        knowledge.active = 0;
        assert_eq!(knowledge.progress().state, StudyCohortScanState::Partial);
        assert_eq!(knowledge.progress().unavailable, 2);
        assert_eq!(knowledge.progress().included, 1);
        assert!(!knowledge.complete);
    }
    #[test]
    fn cohort_exact_repeated_scans_merge_only_established_membership() {
        let mut knowledge = knowledge(4);
        knowledge
            .mark(0, StudyCohortAvailability::CapturedUnavailable)
            .unwrap();
        assert!(
            knowledge
                .mark(0, StudyCohortAvailability::Included)
                .is_err()
        );
        knowledge
            .mark(1, StudyCohortAvailability::Included)
            .unwrap();
        assert!(
            knowledge
                .mark(1, StudyCohortAvailability::CapturedUnavailable)
                .is_err()
        );
        knowledge
            .mark(1, StudyCohortAvailability::EmptySelection)
            .unwrap();
        assert_eq!(knowledge.get(1), StudyCohortAvailability::Included);
        knowledge
            .mark(2, StudyCohortAvailability::EmptySelection)
            .unwrap();
        knowledge
            .mark(3, StudyCohortAvailability::EmptySelection)
            .unwrap();
        knowledge.complete = true;
        knowledge.active = 1;
        let complete = knowledge.progress();
        assert_eq!(complete.state, StudyCohortScanState::Complete);
        assert_eq!(complete.pending, 0);
        assert_eq!(
            complete.active_scans, 1,
            "another physical execution still owns its capture"
        );
        assert!(
            knowledge
                .mark(4, StudyCohortAvailability::Included)
                .is_err()
        );
    }
}
