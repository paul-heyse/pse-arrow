// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Request-owned bounded projection of immutable scientific completion.
use super::{WorkflowError, contract, relation};
use futures_util::{Stream, StreamExt};
use pse_columnar::{CancellationToken, MemoryPool};
use pse_ids::SemanticId;
use pse_relations::{
    RelationError,
    columnar::{Collection, FieldCheckedBatch, RelationRow},
};
use std::{
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

/// Physical traversal of the same scientific rows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ResultOrder {
    /// Existing public sample-major order.
    #[default]
    Public,
    /// Canonical output-major trajectory storage order.
    Canonical,
}
impl ResultOrder {
    pub(super) fn trajectory_coordinate(
        self,
        index: usize,
        outputs: usize,
        samples: usize,
    ) -> Result<(usize, usize), WorkflowError> {
        let extent = outputs
            .checked_mul(samples)
            .ok_or_else(|| contract("trajectory projection extent"))?;
        if index >= extent {
            return Err(contract("trajectory projection coordinate outside extent"));
        }
        Ok(match self {
            Self::Public => (index / outputs, index % outputs),
            Self::Canonical => (index % samples, index / samples),
        })
    }
}
/// Shared intrinsic failures only; request allocation and delivery never enter this map.
#[derive(Debug, Default)]
pub(super) struct EncodingState(Mutex<BTreeMap<SemanticId, Arc<WorkflowError>>>);
impl EncodingState {
    pub(super) fn failure(&self, id: SemanticId) -> Option<Arc<WorkflowError>> {
        self.0.lock().ok().and_then(|state| state.get(&id).cloned())
    }
    pub(super) fn record(&self, id: SemanticId, error: WorkflowError) -> Arc<WorkflowError> {
        let error = Arc::new(error);
        if intrinsic(&error)
            && let Ok(mut state) = self.0.lock()
        {
            return state.entry(id).or_insert_with(|| error.clone()).clone();
        }
        error
    }
}
fn intrinsic(error: &WorkflowError) -> bool {
    use pse_engine::EngineError;
    match error {
        WorkflowError::Input(_) | WorkflowError::Internal(_) => true,
        WorkflowError::Shared(error) => intrinsic(error),
        WorkflowError::Math(crate::math::MathRuntimeError::Solve(
            pse_backend_native::ProblemError::Contract(_),
        )) => true,
        WorkflowError::Engine(EngineError::Relation(error)) => intrinsic_relation(error),
        _ => false,
    }
}
fn intrinsic_relation(error: &RelationError) -> bool {
    use pse_columnar::CanonError;
    match error {
        RelationError::Preparation(error) => intrinsic_relation(error),
        RelationError::Canon(
            CanonError::Cancelled
            | CanonError::Reservation(_)
            | CanonError::NativeResource(_)
            | CanonError::Envelope { .. },
        ) => false,
        RelationError::Engine(_) => false,
        RelationError::Arrow(error) | RelationError::Canon(CanonError::Arrow(error)) => !matches!(
            error,
            datafusion::arrow::error::ArrowError::MemoryError(_)
                | datafusion::arrow::error::ArrowError::IoError(..)
                | datafusion::arrow::error::ArrowError::ExternalError(_)
                | datafusion::arrow::error::ArrowError::OffsetOverflowError(_)
                | datafusion::arrow::error::ArrowError::DictionaryKeyOverflowError
                | datafusion::arrow::error::ArrowError::RunEndIndexOverflowError
        ),
        RelationError::Validation { errors } => errors.iter().all(intrinsic_relation),
        _ => true,
    }
}
#[derive(Clone, Debug)]
pub(super) struct Projection {
    pub(super) relation: SemanticId,
    pub(super) rows: usize,
    pub(super) order: ResultOrder,
    pub(super) range: std::ops::Range<usize>,
    sender: tokio::sync::mpsc::Sender<FieldCheckedBatch>,
    pub(super) cancel: CancellationToken,
}
impl Projection {
    pub(super) async fn send_checked(
        &self,
        batch: FieldCheckedBatch,
    ) -> Result<(), Arc<WorkflowError>> {
        if self.cancel.is_cancelled() {
            return Err(Arc::new(contract("selected result cursor cancelled")));
        }
        self.sender
            .send(batch)
            .await
            .map_err(|_| Arc::new(contract("selected result consumer closed")))
    }
    pub(super) fn wants(&self, relation: SemanticId) -> bool {
        self.relation == relation
    }
}
type ResultProducer<'a> = Pin<Box<dyn Future<Output = Result<(), Arc<WorkflowError>>> + Send + 'a>>;

/// A delivered prefix is provisional. Only successful terminal exhaustion completes a relation.
pub struct ResultCursor<'a> {
    producer: Option<ResultProducer<'a>>,
    receiver: tokio::sync::mpsc::Receiver<FieldCheckedBatch>,
    terminal: Option<Result<(), Arc<WorkflowError>>>,
    schema: datafusion::arrow::datatypes::SchemaRef,
    complete: bool,
    cancel: CancellationToken,
    checked_buffers: bool,
    cancel_on_drop: bool,
    projection_memory_refused: bool,
}
impl std::fmt::Debug for ResultCursor<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResultCursor")
            .field("complete", &self.complete)
            .finish_non_exhaustive()
    }
}
impl<'a> ResultCursor<'a> {
    /// A selected original relation owns its buffers independently of a run report.
    pub(super) fn from_retained(
        schema: datafusion::arrow::datatypes::SchemaRef,
        id: SemanticId,
        batches: &'a [FieldCheckedBatch],
    ) -> Result<Self, WorkflowError> {
        let mut cursor = Self::new(
            schema,
            id,
            1024,
            ResultOrder::Public,
            move |request| async move {
                for batch in batches {
                    if request.cancel.is_cancelled() {
                        return Err(Arc::new(contract("selected result cursor cancelled")));
                    }
                    request
                        .sender
                        .send(batch.clone())
                        .await
                        .map_err(|_| Arc::new(contract("selected result consumer closed")))?;
                }
                Ok(())
            },
        )?;
        cursor.checked_buffers = true;
        Ok(cursor)
    }

    /// Traverse an already selected checked relation without another encoding pass.
    #[cfg(all(test, feature = "canonical-tests"))]
    pub(super) fn from_checked(batch: FieldCheckedBatch) -> Self {
        let schema = batch.batch().schema();
        let (sender, receiver) = tokio::sync::mpsc::channel(1);
        Self {
            producer: Some(Box::pin(async move {
                sender.send(batch).await.map_err(|_| {
                    Arc::new(WorkflowError::Engine(pse_engine::EngineError::Cancelled))
                })
            })),
            receiver,
            terminal: None,
            schema,
            complete: false,
            cancel: CancellationToken::new(),
            checked_buffers: true,
            cancel_on_drop: true,
            projection_memory_refused: false,
        }
    }

    pub(super) fn new<F, Fut>(
        schema: datafusion::arrow::datatypes::SchemaRef,
        id: SemanticId,
        rows: usize,
        order: ResultOrder,
        build: F,
    ) -> Result<Self, WorkflowError>
    where
        F: FnOnce(Projection) -> Fut,
        Fut: Future<Output = Result<(), Arc<WorkflowError>>> + Send + 'a,
    {
        Self::new_range(schema, id, rows, order, 0..usize::MAX, build)
    }
    pub(super) fn new_range<F, Fut>(
        schema: datafusion::arrow::datatypes::SchemaRef,
        id: SemanticId,
        rows: usize,
        order: ResultOrder,
        range: std::ops::Range<usize>,
        build: F,
    ) -> Result<Self, WorkflowError>
    where
        F: FnOnce(Projection) -> Fut,
        Fut: Future<Output = Result<(), Arc<WorkflowError>>> + Send + 'a,
    {
        Self::new_range_controlled(
            schema,
            id,
            rows,
            order,
            range,
            CancellationToken::new(),
            build,
        )
    }
    pub(super) fn new_range_controlled<F, Fut>(
        schema: datafusion::arrow::datatypes::SchemaRef,
        id: SemanticId,
        rows: usize,
        order: ResultOrder,
        range: std::ops::Range<usize>,
        cancel: CancellationToken,
        build: F,
    ) -> Result<Self, WorkflowError>
    where
        F: FnOnce(Projection) -> Fut,
        Fut: Future<Output = Result<(), Arc<WorkflowError>>> + Send + 'a,
    {
        if rows == 0 {
            return Err(contract("result chunk row bound must be positive"));
        }
        if range.start > range.end {
            return Err(contract("result row range is reversed"));
        }
        let (sender, receiver) = tokio::sync::mpsc::channel(1);
        let producer = build(Projection {
            relation: id,
            rows,
            order,
            range,
            sender,
            cancel: cancel.clone(),
        });
        Ok(Self {
            producer: Some(Box::pin(producer)),
            receiver,
            terminal: None,
            schema,
            complete: false,
            cancel,
            checked_buffers: false,
            cancel_on_drop: true,
            projection_memory_refused: false,
        })
    }
    /// Only the typed immutable projection owner may reuse its token after a
    /// positively classified memory refusal. The failed producer has returned;
    /// no callback, background task or undelivered chunk may remain.
    pub(super) fn release_failed_projection(&mut self) -> Result<(), WorkflowError> {
        if !self.projection_memory_refused
            || self.producer.is_some()
            || !self.receiver.is_closed()
            || !self.receiver.is_empty()
            || self.complete
        {
            return Err(contract("failed projection has not drained"));
        }
        self.cancel_on_drop = false;
        Ok(())
    }
    /// Only private original-buffer constructors establish this fact. An
    /// allocating producer or filter must be polled without optional append
    /// retention until it provides a separately pre-admitted producer contract.
    pub(super) const fn owns_checked_buffers(&self) -> bool {
        self.checked_buffers
    }
    /// Request cancellation shared with foreign-language delivery and computation checkpoints.
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancel.clone()
    }
    /// Exact registry schema, including for an empty relation.
    pub fn schema(&self) -> datafusion::arrow::datatypes::SchemaRef {
        self.schema.clone()
    }
    /// Successful terminal exhaustion, never merely delivery of the last nonempty chunk.
    pub const fn complete(&self) -> bool {
        self.complete
    }
    /// Pull one bounded checked chunk without an executor or background task.
    pub fn next_chunk(&mut self) -> Result<Option<FieldCheckedBatch>, Arc<WorkflowError>> {
        let waker = futures_util::task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        let mut request = tokio::task::unconstrained(self.next());
        match Pin::new(&mut request).poll(&mut cx) {
            Poll::Ready(Some(result)) => result.map(Some),
            Poll::Ready(None) => Ok(None),
            Poll::Pending => Err(Arc::new(contract(
                "synchronous result projection unexpectedly suspended",
            ))),
        }
    }
    /// Pull one chunk with asynchronous backpressure.
    pub async fn next_batch(&mut self) -> Result<Option<FieldCheckedBatch>, Arc<WorkflowError>> {
        self.next().await.transpose()
    }
}
impl Drop for ResultCursor<'_> {
    fn drop(&mut self) {
        if self.cancel_on_drop {
            self.cancel.cancel();
        }
    }
}

impl Stream for ResultCursor<'_> {
    type Item = Result<FieldCheckedBatch, Arc<WorkflowError>>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if let Ok(batch) = this.receiver.try_recv() {
            return Poll::Ready(Some(Ok(batch)));
        }
        if let Some(producer) = &mut this.producer
            && let Poll::Ready(terminal) = producer.as_mut().poll(cx)
        {
            this.terminal = Some(terminal);
            this.producer = None;
        }
        match this.receiver.poll_recv(cx) {
            Poll::Ready(Some(batch)) => Poll::Ready(Some(Ok(batch))),
            Poll::Ready(None) => match this.terminal.take() {
                Some(Err(error)) => {
                    this.projection_memory_refused =
                        super::result_projection::projection_memory_refusal(&error);
                    Poll::Ready(Some(Err(error)))
                }
                Some(Ok(())) => {
                    this.complete = true;
                    Poll::Ready(None)
                }
                None => Poll::Ready(None),
            },
            Poll::Pending => Poll::Pending,
        }
    }
}

pub(super) struct SelectedCollection<'a> {
    request: &'a Projection,
    registry: &'a pse_schema::Registry,
    pool: &'a Arc<dyn MemoryPool>,
    cancel: &'a CancellationToken,
    validation: &'a pse_relations::validate::ValidationContext,
    columns: Option<Collection<'a>>,
    row_copy: Option<pse_columnar::MemoryReservation>,
    count: usize,
    position: usize,
}
impl<'a> SelectedCollection<'a> {
    pub(super) fn new(
        request: &'a Projection,
        registry: &'a pse_schema::Registry,
        pool: &'a Arc<dyn MemoryPool>,
        cancel: &'a CancellationToken,
        validation: &'a pse_relations::validate::ValidationContext,
    ) -> Self {
        Self {
            request,
            registry,
            pool,
            cancel,
            validation,
            columns: None,
            row_copy: None,
            count: 0,
            position: 0,
        }
    }
    pub(super) fn ensure<T: RelationRow>(&mut self) -> Result<(), RelationError> {
        if !self.request.wants(T::relation(self.registry)?.id) {
            return Ok(());
        }
        if self.columns.is_none() {
            let mut columns =
                Collection::new(self.registry, self.pool, self.cancel, self.validation);
            columns.ensure::<T>()?;
            self.columns = Some(columns);
        }
        Ok(())
    }
    pub(super) fn set_position(&mut self, position: usize) {
        self.position = position;
    }
    pub(super) fn skip_next<T: RelationRow>(&mut self) -> bool {
        if !self
            .request
            .wants(T::relation(self.registry).map_or(SemanticId::NIL, |spec| spec.id))
        {
            return true;
        }
        if self.request.range.contains(&self.position) {
            false
        } else {
            self.position = self.position.saturating_add(1);
            true
        }
    }
    pub(super) async fn push_ref<T: RelationRow + Clone>(
        &mut self,
        row: &T,
    ) -> Result<(), RelationError> {
        if self.skip_next::<T>() {
            return Ok(());
        }
        let bytes = row.allocation_size()?;
        let scratch = self
            .row_copy
            .get_or_insert_with(|| {
                pse_columnar::MemoryConsumer::new("result:chunk-row-copy").register(self.pool)
            })
            .new_empty();
        // The child reservation shares this chunk's registration. It frees the row's
        // charge even when a caller abandons this future during sender backpressure.
        scratch
            .try_grow(bytes)
            .map_err(pse_columnar::CanonError::from)?;
        self.push(row.clone()).await
    }
    pub(super) async fn push<T: RelationRow>(&mut self, row: T) -> Result<(), RelationError> {
        if !self.request.wants(T::relation(self.registry)?.id) {
            return Ok(());
        }
        if self.skip_next::<T>() {
            return Ok(());
        }
        self.position = self.position.saturating_add(1);
        self.ensure::<T>()?;
        if let Some(columns) = &mut self.columns {
            columns.push(row)?;
        }
        self.count += 1;
        if self.count == self.request.rows {
            self.flush().await?;
        }
        Ok(())
    }
    fn strategy_window(
        &mut self,
        extent: usize,
    ) -> Result<(usize, std::ops::Range<usize>), WorkflowError> {
        let origin = self.position;
        let end = origin
            .checked_add(extent)
            .ok_or_else(|| contract("strategy result global row extent"))?;
        let range = self.request.range.start.saturating_sub(origin).min(extent)
            ..self.request.range.end.saturating_sub(origin).min(extent);
        self.position = origin
            .checked_add(range.start)
            .ok_or_else(|| contract("strategy result selected row extent"))?;
        Ok((end, range))
    }
    pub(super) async fn strategy_events(
        &mut self,
        trace: &crate::math::solves::StrategyTrace,
        run: pse_model::generated::identities::RunId,
        step: usize,
        service: &crate::math::MathService,
    ) -> Result<(), WorkflowError> {
        if !self
            .request
            .wants(pse_relations::generated::runtime::solve_strategy_events::RELATION_ID)
        {
            return Ok(());
        }
        let (end, range) = self.strategy_window(trace.event_count())?;
        if !range.is_empty() {
            for row in trace
                .rows(run, step, service, range)
                .map_err(crate::math::MathRuntimeError::from)?
            {
                self.push(row.map_err(crate::math::MathRuntimeError::from)?)
                    .await
                    .map_err(relation)?;
            }
        }
        self.position = end;
        Ok(())
    }
    pub(super) async fn strategy_products(
        &mut self,
        trace: &crate::math::solves::StrategyTrace,
        run: pse_model::generated::identities::RunId,
        step: usize,
    ) -> Result<(), WorkflowError> {
        if !self
            .request
            .wants(pse_relations::generated::runtime::solve_strategy_products::RELATION_ID)
        {
            return Ok(());
        }
        let (end, range) = self.strategy_window(
            trace
                .product_count()
                .map_err(crate::math::MathRuntimeError::from)?,
        )?;
        if !range.is_empty() {
            for row in trace
                .product_rows(run, step, range)
                .map_err(crate::math::MathRuntimeError::from)?
            {
                self.push(row.map_err(crate::math::MathRuntimeError::from)?)
                    .await
                    .map_err(relation)?;
            }
        }
        self.position = end;
        Ok(())
    }
    async fn flush(&mut self) -> Result<(), RelationError> {
        if let Some(columns) = self.columns.take() {
            for batch in columns.finish()?.into_values() {
                self.request
                    .sender
                    .send(batch)
                    .await
                    .map_err(|_| pse_columnar::CanonError::Cancelled)?;
                let mut yielded = false;
                futures_util::future::poll_fn(|cx| {
                    if yielded {
                        Poll::Ready(())
                    } else {
                        yielded = true;
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                })
                .await;
            }
            self.count = 0;
        }
        // Registration follows the bounded chunk through sender backpressure. Each
        // temporary row clone is charged only while live, with no per-row consumer.
        self.row_copy.take();
        Ok(())
    }
    pub(super) async fn finish(mut self) -> Result<(), RelationError> {
        self.flush().await
    }
}
/// Typed convenience over the selected collection; unrelated builders allocate nothing.
pub(super) struct Rows<'a, T: RelationRow> {
    columns: SelectedCollection<'a>,
    marker: std::marker::PhantomData<fn() -> T>,
}
impl<'a, T: RelationRow> Rows<'a, T> {
    pub(super) fn new(
        request: &'a Projection,
        registry: &'a pse_schema::Registry,
        pool: &'a Arc<dyn MemoryPool>,
        cancel: &'a CancellationToken,
        validation: &'a pse_relations::validate::ValidationContext,
    ) -> Result<Self, RelationError> {
        let mut columns = SelectedCollection::new(request, registry, pool, cancel, validation);
        columns.ensure::<T>()?;
        Ok(Self {
            columns,
            marker: std::marker::PhantomData,
        })
    }
    pub(super) async fn strategy_events(
        &mut self,
        trace: &crate::math::solves::StrategyTrace,
        run: pse_model::generated::identities::RunId,
        step: usize,
        service: &crate::math::MathService,
    ) -> Result<(), WorkflowError> {
        self.columns
            .strategy_events(trace, run, step, service)
            .await
    }
    pub(super) async fn strategy_products(
        &mut self,
        trace: &crate::math::solves::StrategyTrace,
        run: pse_model::generated::identities::RunId,
        step: usize,
    ) -> Result<(), WorkflowError> {
        self.columns.strategy_products(trace, run, step).await
    }
    pub(super) fn pool(&self) -> Arc<dyn MemoryPool> {
        self.columns.pool.clone()
    }
    pub(super) fn working(
        &self,
        parts: &[(usize, usize)],
    ) -> Result<pse_columnar::MemoryReservation, WorkflowError> {
        working(self.columns.pool, "result:selected-row-working", parts)
    }
    pub(super) fn skip_next(&mut self) -> bool {
        self.columns.skip_next::<T>()
    }
    pub(super) fn wanted(&self) -> bool {
        self.columns
            .request
            .wants(T::relation(self.columns.registry).map_or(SemanticId::NIL, |spec| spec.id))
    }
    pub(super) async fn push(&mut self, row: T) -> Result<(), RelationError> {
        self.columns.push(row).await
    }
    pub(super) async fn push_ref(&mut self, row: &T) -> Result<(), RelationError>
    where
        T: Clone,
    {
        self.columns.push_ref(row).await
    }
    pub(super) async fn finish(self) -> Result<(), RelationError> {
        self.columns.finish().await
    }
}
pub(super) fn working(
    pool: &Arc<dyn MemoryPool>,
    name: &str,
    parts: &[(usize, usize)],
) -> Result<pse_columnar::MemoryReservation, WorkflowError> {
    let bytes = parts
        .iter()
        .try_fold(0usize, |total, (count, width)| {
            count
                .checked_mul(*width)
                .and_then(|bytes| total.checked_add(bytes))
        })
        .ok_or_else(|| contract("result working extent"))?;
    let reservation = pse_columnar::MemoryConsumer::new(name).register(pool);
    reservation
        .try_grow(bytes)
        .map_err(pse_columnar::CanonError::from)
        .map_err(|error| relation(error.into()))?;
    Ok(reservation)
}

pub(super) fn collect(
    mut cursor: ResultCursor<'_>,
    runtime: &super::Runtime,
    id: SemanticId,
) -> Result<FieldCheckedBatch, Arc<WorkflowError>> {
    let mut chunks = Vec::new();
    let metadata = pse_columnar::MemoryConsumer::new("result:convenience-chunks")
        .register(&runtime.shared.pool());
    while let Some(chunk) = cursor.next_chunk()? {
        metadata
            .try_grow(2 * size_of::<FieldCheckedBatch>())
            .map_err(|e| Arc::new(relation(pse_columnar::CanonError::from(e).into())))?;
        chunks.push(chunk);
    }
    let spec = runtime
        .registry()
        .relation_by_id(id)
        .ok_or_else(|| Arc::new(contract("result declaration absent")))?;
    FieldCheckedBatch::concat_reserved(
        runtime.registry(),
        spec,
        &chunks,
        &runtime.shared.pool(),
        &CancellationToken::new(),
    )
    .map_err(|e| Arc::new(relation(e)))
}

/// Shared immutable strategy traces with independent selected event transport.
#[derive(Debug)]
pub struct StrategyEventExport {
    model: pse_engine::session::BoundModel,
    shared: Arc<crate::SharedRuntime>,
    traces: Vec<(
        pse_model::generated::identities::RunId,
        usize,
        crate::math::solves::StrategyTrace,
    )>,
    failures: EncodingState,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl StrategyEventExport {
    /// Retain shared traces in original attempt order; payload encoding remains lazy.
    pub fn new(
        model: pse_engine::session::BoundModel,
        shared: Arc<crate::SharedRuntime>,
        upper: usize,
        traces: impl Iterator<
            Item = (
                pse_model::generated::identities::RunId,
                usize,
                crate::math::solves::StrategyTrace,
            ),
        >,
    ) -> Result<Self, WorkflowError> {
        let bytes = upper
            .checked_mul(size_of::<(
                pse_model::generated::identities::RunId,
                usize,
                crate::math::solves::StrategyTrace,
            )>())
            .ok_or_else(|| contract("strategy trace export extent"))?;
        let owner = shared
            .math()
            .reserve("result:strategy-trace-index", bytes)?;
        let mut retained = Vec::with_capacity(upper);
        for trace in traces {
            if retained.len() == upper {
                return Err(contract("strategy trace export exceeds declared extent"));
            }
            retained.push(trace);
        }
        Ok(Self {
            model,
            shared,
            traces: retained,
            failures: EncodingState::default(),
            _owner: owner,
        })
    }
    /// A bounded cursor over actual original events, with sticky intrinsic defects only.
    pub fn into_cursor(
        self: Arc<Self>,
        rows: usize,
    ) -> Result<ResultCursor<'static>, WorkflowError> {
        use pse_relations::generated::runtime::solve_strategy_events as events;
        let id = events::RELATION_ID;
        if let Some(error) = self.failures.failure(id) {
            return Err(WorkflowError::Shared(error));
        }
        let spec = self
            .model
            .registry()
            .relation_by_id(id)
            .ok_or_else(|| contract("strategy event declaration absent"))?;
        let schema = pse_schema::arrow::relation_schema_ref(self.model.registry(), spec)
            .map_err(RelationError::from)
            .map_err(relation)?;
        ResultCursor::new(
            schema,
            id,
            rows,
            ResultOrder::Public,
            |request| async move {
                let result = async {
                    let pool = self.shared.pool();
                    let cancel = request.cancel.clone();
                    let validation = self.model.validation_context();
                    let mut columns = Rows::<events::Row>::new(
                        &request,
                        self.model.registry(),
                        &pool,
                        &cancel,
                        &validation,
                    )
                    .map_err(relation)?;
                    for (run, step, trace) in &self.traces {
                        columns
                            .strategy_events(trace, *run, *step, self.shared.math())
                            .await?;
                    }
                    columns.finish().await.map_err(relation)
                }
                .await;
                result.map_err(|error| self.failures.record(id, error))
            },
        )
    }
}

pub(super) fn collect_tables<'a>(
    ids: Vec<SemanticId>,
    runtime: &super::Runtime,
    mut cursor: impl FnMut(SemanticId) -> Result<ResultCursor<'a>, Arc<WorkflowError>>,
) -> Result<BTreeMap<SemanticId, FieldCheckedBatch>, Arc<WorkflowError>> {
    let bytes = ids
        .len()
        .checked_mul(size_of::<FieldCheckedBatch>() + size_of::<SemanticId>() + 128)
        .ok_or_else(|| Arc::new(contract("result map extent")))?;
    let owner = runtime
        .shared
        .math()
        .reserve("result:complete-map", bytes)
        .map_err(|error| Arc::new(WorkflowError::Math(error)))?;
    ids.into_iter()
        .map(|id| {
            collect(cursor(id)?, runtime, id)
                .map(|batch| (id, batch.with_export_owner(owner.clone())))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::execution::memory_pool::GreedyMemoryPool;
    use pse_columnar::MemoryConsumer;
    use pse_relations::generated::runtime::{fit_parameters, solve_metrics};
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Debug)]
    struct CopyCountingPool {
        inner: GreedyMemoryPool,
        registrations: AtomicUsize,
        departures: AtomicUsize,
        live: AtomicUsize,
    }
    impl std::fmt::Display for CopyCountingPool {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            std::fmt::Display::fmt(&self.inner, f)
        }
    }
    impl MemoryPool for CopyCountingPool {
        fn name(&self) -> &str {
            "row-copy-control"
        }
        fn register(&self, consumer: &pse_columnar::MemoryConsumer) {
            self.inner.register(consumer);
            if consumer.name() == "result:chunk-row-copy" {
                self.registrations.fetch_add(1, Ordering::Relaxed);
                assert_eq!(self.live.fetch_add(1, Ordering::Relaxed), 0);
            }
        }
        fn unregister(&self, consumer: &pse_columnar::MemoryConsumer) {
            self.inner.unregister(consumer);
            if consumer.name() == "result:chunk-row-copy" {
                self.departures.fetch_add(1, Ordering::Relaxed);
                assert_eq!(self.live.fetch_sub(1, Ordering::Relaxed), 1);
            }
        }
        fn grow(&self, reservation: &pse_columnar::MemoryReservation, bytes: usize) {
            self.inner.grow(reservation, bytes);
        }
        fn shrink(&self, reservation: &pse_columnar::MemoryReservation, bytes: usize) {
            self.inner.shrink(reservation, bytes);
        }
        fn try_grow(
            &self,
            reservation: &pse_columnar::MemoryReservation,
            bytes: usize,
        ) -> datafusion::common::Result<()> {
            self.inner.try_grow(reservation, bytes)
        }
        fn reserved(&self) -> usize {
            self.inner.reserved()
        }
        fn memory_limit(&self) -> datafusion::execution::memory_pool::MemoryLimit {
            self.inner.memory_limit()
        }
    }
    #[tokio::test]
    async fn row_copy_admission_is_lazy_and_owned_by_bounded_chunk_through_backpressure() {
        let source = Source::new(1 << 20);
        let observed = Arc::new(CopyCountingPool {
            inner: GreedyMemoryPool::new(1 << 20),
            registrations: AtomicUsize::new(0),
            departures: AtomicUsize::new(0),
            live: AtomicUsize::new(0),
        });
        let pool: Arc<dyn MemoryPool> = observed.clone();
        let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
        let request = Projection {
            relation: fit_parameters::RELATION_ID,
            rows: 3,
            order: ResultOrder::Public,
            range: 1..6,
            sender,
            cancel: CancellationToken::new(),
        };
        let mut columns = SelectedCollection::new(
            &request,
            source.registry,
            &pool,
            &request.cancel,
            &source.validation,
        );
        let row = fit_parameters::Row {
            run_id: SemanticId::NIL.into(),
            parameter_id: SemanticId::NIL,
            fixed: false,
            value: Some(1.0),
            unit_id: SemanticId::NIL,
            scale: 1.0,
            at_bound: None,
        };
        columns.push_ref(&row).await.unwrap();
        assert_eq!(observed.registrations.load(Ordering::Relaxed), 0); // Excluded row.
        for _ in 0..3 {
            columns.push_ref(&row).await.unwrap();
        }
        assert_eq!(observed.registrations.load(Ordering::Relaxed), 1);
        assert_eq!(observed.departures.load(Ordering::Relaxed), 1);
        for _ in 0..2 {
            columns.push_ref(&row).await.unwrap();
        }
        assert_eq!(observed.registrations.load(Ordering::Relaxed), 2);
        let mut finish = Box::pin(columns.finish());
        assert!(futures_util::poll!(finish.as_mut()).is_pending());
        assert_eq!(observed.live.load(Ordering::Relaxed), 1);
        let first = receiver.recv().await.unwrap();
        assert_eq!(first.batch().num_rows(), 3);
        finish.await.unwrap();
        let second = receiver.recv().await.unwrap();
        assert_eq!(second.batch().num_rows(), 2);
        assert_eq!(observed.departures.load(Ordering::Relaxed), 2);
        assert_eq!(observed.live.load(Ordering::Relaxed), 0);
        drop((first, second));
        assert_eq!(pool.reserved(), 0);
    }
    #[tokio::test]
    async fn row_copy_refusal_and_abandoned_send_release_exact_owned_charge() {
        let source = Source::new(1 << 20);
        let observed = Arc::new(CopyCountingPool {
            inner: GreedyMemoryPool::new(1 << 20),
            registrations: AtomicUsize::new(0),
            departures: AtomicUsize::new(0),
            live: AtomicUsize::new(0),
        });
        let pool: Arc<dyn MemoryPool> = observed.clone();
        let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
        let request = Projection {
            relation: fit_parameters::RELATION_ID,
            rows: 1,
            order: ResultOrder::Public,
            range: 0..usize::MAX,
            sender,
            cancel: CancellationToken::new(),
        };
        let row = fit_parameters::Row {
            run_id: SemanticId::NIL.into(),
            parameter_id: SemanticId::NIL,
            fixed: false,
            value: Some(1.0),
            unit_id: SemanticId::NIL,
            scale: 1.0,
            at_bound: None,
        };
        let mut columns = SelectedCollection::new(
            &request,
            source.registry,
            &pool,
            &request.cancel,
            &source.validation,
        );
        let fill = pse_columnar::MemoryConsumer::new("row-copy-refusal-control").register(&pool);
        fill.try_grow(1 << 20).unwrap();
        assert!(columns.push_ref(&row).await.is_err());
        assert_eq!(columns.count, 0);
        assert_eq!(pool.reserved(), 1 << 20);
        drop(fill);
        columns.push_ref(&row).await.unwrap();
        let mut blocked = Box::pin(columns.push_ref(&row));
        assert!(futures_util::poll!(blocked.as_mut()).is_pending());
        let first = receiver.recv().await.unwrap();
        assert!(pool.reserved() > row.allocation_size().unwrap());
        drop(blocked);
        drop(first);
        columns.finish().await.unwrap();
        assert_eq!(observed.live.load(Ordering::Relaxed), 0);
        assert_eq!(pool.reserved(), 0);
    }
    #[tokio::test]
    async fn failed_pure_projection_transfers_same_token_only_after_memory_drain() {
        let source = Source::new(1 << 20);
        let id = fit_parameters::RELATION_ID;
        let schema = pse_schema::arrow::relation_schema_ref(
            source.registry,
            source.registry.relation_by_id(id).unwrap(),
        )
        .unwrap();
        let token = CancellationToken::new();
        let pressure = MemoryConsumer::new("test:projection-failure").register(&source.pool);
        pressure.try_grow(1 << 20).unwrap();
        let producer_source = source.clone();
        let mut failed = ResultCursor::new_range_controlled(
            schema.clone(),
            id,
            1,
            ResultOrder::Canonical,
            0..usize::MAX,
            token.clone(),
            move |request| async move {
                let mut builder = fit_parameters::Builder::with_registry(
                    producer_source.registry,
                    1,
                    &producer_source.validation,
                )
                .map_err(|e| Arc::new(relation(e)))?;
                builder
                    .push(fit_parameters::Row {
                        run_id: SemanticId::NIL.into(),
                        parameter_id: SemanticId::NIL,
                        fixed: false,
                        value: Some(0.0),
                        unit_id: SemanticId::NIL,
                        scale: 1.0,
                        at_bound: None,
                    })
                    .map_err(|e| Arc::new(relation(e)))?;
                request
                    .sender
                    .send(builder.finish().map_err(|e| Arc::new(relation(e)))?)
                    .await
                    .unwrap();
                let allocation = MemoryConsumer::new("test:second-projection-chunk")
                    .register(&producer_source.pool);
                allocation
                    .try_grow(1)
                    .map_err(|e| Arc::new(WorkflowError::from(pse_engine::EngineError::from(e))))?;
                Ok(())
            },
        )
        .unwrap();
        assert!(
            failed.release_failed_projection().is_err(),
            "a live producer cannot transfer its cancellation owner"
        );
        let delivered = failed.next_batch().await.unwrap().unwrap();
        assert_eq!(delivered.batch().num_rows(), 1);
        drop(delivered);
        let error = failed.next_batch().await.unwrap_err();
        assert!(super::super::result_projection::projection_memory_refusal(
            &error
        ));
        failed.release_failed_projection().unwrap();
        drop(failed);
        assert!(
            !token.is_cancelled(),
            "failed drained cursor cannot cancel its enclosing retry scope"
        );
        drop(pressure);
        let next_source = source.clone();
        let mut next = ResultCursor::new_range_controlled(
            schema,
            id,
            2,
            ResultOrder::Canonical,
            1..3,
            token.clone(),
            move |request| async move {
                let mut columns = SelectedCollection::new(
                    &request,
                    next_source.registry,
                    &next_source.pool,
                    &request.cancel,
                    &next_source.validation,
                );
                for value in 0..3 {
                    columns
                        .push(fit_parameters::Row {
                            run_id: SemanticId::NIL.into(),
                            parameter_id: SemanticId::NIL,
                            fixed: false,
                            value: Some(value as f64),
                            unit_id: SemanticId::NIL,
                            scale: 1.0,
                            at_bound: None,
                        })
                        .await
                        .map_err(|e| Arc::new(relation(e)))?;
                }
                columns.finish().await.map_err(|e| Arc::new(relation(e)))
            },
        )
        .unwrap();
        let selected = next.next_batch().await.unwrap().unwrap();
        let view = fit_parameters::View::from_checked(&selected).unwrap();
        assert_eq!(view.len(), 2);
        assert_eq!(view.row(0).unwrap().value, Some(1.0));
        assert_eq!(view.row(1).unwrap().value, Some(2.0));
        assert!(next.next_batch().await.unwrap().is_none());
        assert!(next.complete());
        assert!(next.release_failed_projection().is_err());
        drop(next);
        assert!(
            token.is_cancelled(),
            "the final cursor still owns cancellation"
        );
    }

    struct Source {
        registry: &'static pse_schema::Registry,
        validation: pse_relations::validate::ValidationContext,
        pool: Arc<dyn MemoryPool>,
        failures: EncodingState,
        projected: AtomicUsize,
    }
    impl Source {
        fn new(bytes: usize) -> Arc<Self> {
            let registry = pse_schema::registry().unwrap();
            Arc::new(Self {
                registry,
                validation: pse_relations::validate::ValidationContext::new(
                    registry,
                    pse_engine::validation::NativeValidation(
                        datafusion::prelude::SessionContext::new().state(),
                    ),
                ),
                pool: Arc::new(GreedyMemoryPool::new(bytes)),
                failures: EncodingState::default(),
                projected: AtomicUsize::new(0),
            })
        }
        fn cursor(
            self: &Arc<Self>,
            id: SemanticId,
            rows: usize,
            count: usize,
            bad: Option<usize>,
        ) -> Result<ResultCursor<'static>, Arc<WorkflowError>> {
            self.range(id, rows, count, bad, 0..usize::MAX)
        }
        fn range(
            self: &Arc<Self>,
            id: SemanticId,
            rows: usize,
            count: usize,
            bad: Option<usize>,
            range: std::ops::Range<usize>,
        ) -> Result<ResultCursor<'static>, Arc<WorkflowError>> {
            if let Some(error) = self.failures.failure(id) {
                return Err(error);
            }
            let schema = pse_schema::arrow::relation_schema_ref(
                self.registry,
                self.registry.relation_by_id(id).unwrap(),
            )
            .unwrap();
            let source = self.clone();
            ResultCursor::new_range(
                schema,
                id,
                rows,
                ResultOrder::Public,
                range,
                move |request| async move {
                    let result = async {
                        let cancel = request.cancel.clone();
                        let mut columns = SelectedCollection::new(
                            &request,
                            source.registry,
                            &source.pool,
                            &cancel,
                            &source.validation,
                        );
                        columns.ensure::<fit_parameters::Row>().map_err(relation)?;
                        columns.ensure::<solve_metrics::Row>().map_err(relation)?;
                        if request.wants(fit_parameters::RELATION_ID) {
                            for index in 0..count {
                                if columns.skip_next::<fit_parameters::Row>() {
                                    continue;
                                }
                                source.projected.fetch_add(1, Ordering::Relaxed);
                                if bad == Some(index) {
                                    return Err(contract("selected encoding fixture defect"));
                                }
                                columns
                                    .push(fit_parameters::Row {
                                        run_id: SemanticId::NIL.into(),
                                        parameter_id: SemanticId::NIL,
                                        fixed: false,
                                        value: Some(index as f64),
                                        unit_id: SemanticId::NIL,
                                        scale: 1.0,
                                        at_bound: None,
                                    })
                                    .await
                                    .map_err(relation)?;
                            }
                        }
                        columns.finish().await.map_err(relation)
                    }
                    .await;
                    result.map_err(|error| source.failures.record(id, error))
                },
            )
            .map_err(Arc::new)
        }
    }

    #[test]
    fn strategy_windows_keep_checked_global_origins_without_skipped_projection() {
        let source = Source::new(1 << 20);
        let (sender, _) = tokio::sync::mpsc::channel(1);
        let request = Projection {
            relation: solve_metrics::RELATION_ID,
            rows: 1,
            order: ResultOrder::Public,
            range: 4..5,
            sender,
            cancel: CancellationToken::new(),
        };
        let mut columns = SelectedCollection::new(
            &request,
            source.registry,
            &source.pool,
            &request.cancel,
            &source.validation,
        );
        let (end, window) = columns.strategy_window(3).unwrap();
        assert_eq!((end, window), (3, 3..3));
        columns.position = end;
        let (end, window) = columns.strategy_window(4).unwrap();
        assert_eq!((end, window, columns.position), (7, 1..2, 4));
        columns.position = end;
        assert_eq!(columns.strategy_window(2).unwrap(), (9, 0..0));
        columns.position = usize::MAX;
        assert!(columns.strategy_window(1).is_err());
    }

    #[test]
    fn native_certificate_and_text_copy_pressure_are_selected_and_request_local() {
        use pse_backend_native::solve::{
            Backend, CertificateAccuracy, CertificateKind, InfeasibilityCertificate, RayCoordinate,
            RayEntry,
        };
        use pse_relations::generated::runtime::infeasibility_certificates as certificates;
        let source = Source::new(8 << 20);
        let certificate = Arc::new(InfeasibilityCertificate {
            kind: CertificateKind::PrimalInfeasible,
            accuracy: CertificateAccuracy::Full,
            ray: vec![
                RayEntry {
                    coordinate: RayCoordinate::Row,
                    id: SemanticId::NIL,
                    value: 1.0
                };
                8192
            ],
            verification: None,
        });
        let cursor = |id, range, text: Arc<String>| {
            let source = source.clone();
            let certificate = certificate.clone();
            let spec = source.registry.relation_by_id(id).unwrap();
            let schema = pse_schema::arrow::relation_schema_ref(source.registry, spec).unwrap();
            ResultCursor::new_range(
                schema,
                id,
                1,
                ResultOrder::Public,
                range,
                move |request| async move {
                    let result = async {
                        let mut metrics = Rows::<solve_metrics::Row>::new(
                            &request,
                            source.registry,
                            &source.pool,
                            &request.cancel,
                            &source.validation,
                        )
                        .map_err(relation)?;
                        let mut certificates = Rows::<certificates::Row>::new(
                            &request,
                            source.registry,
                            &source.pool,
                            &request.cancel,
                            &source.validation,
                        )
                        .map_err(relation)?;
                        super::super::results::push_metric_lazy(
                            &mut metrics,
                            pse_model::generated::identities::RunId::from_bytes([1; 16]),
                            0,
                            "native",
                            "text",
                            text.len(),
                            || {
                                Ok(pse_backend_native::solve::Metric::Text(
                                    text.as_ref().clone(),
                                ))
                            },
                        )
                        .await?;
                        super::super::results::push_certificate(
                            &mut certificates,
                            pse_model::generated::identities::RunId::from_bytes([1; 16]),
                            0,
                            Backend::Clarabel,
                            Some(&certificate),
                        )
                        .await?;
                        metrics.finish().await.map_err(relation)?;
                        certificates.finish().await.map_err(relation)
                    }
                    .await;
                    result.map_err(|error| source.failures.record(id, error))
                },
            )
            .unwrap()
        };
        let pressure =
            pse_columnar::MemoryConsumer::new("test:native-export-pressure").register(&source.pool);
        pressure.try_grow((8 << 20) - (64 << 10)).unwrap();
        let small = Arc::new("short".to_owned());
        assert!(
            cursor(solve_metrics::RELATION_ID, 0..1, small.clone())
                .next_chunk()
                .unwrap()
                .is_some()
        );
        assert!(
            cursor(certificates::RELATION_ID, 1..1, small.clone())
                .next_chunk()
                .unwrap()
                .is_some()
        );
        assert!(
            cursor(certificates::RELATION_ID, 0..1, small.clone())
                .next_chunk()
                .is_err()
        );
        assert!(
            cursor(
                solve_metrics::RELATION_ID,
                0..1,
                Arc::new("x".repeat(128 << 10))
            )
            .next_chunk()
            .is_err()
        );
        assert!(source.failures.failure(certificates::RELATION_ID).is_none());
        assert!(
            source
                .failures
                .failure(solve_metrics::RELATION_ID)
                .is_none()
        );
        drop(pressure);
        assert!(
            cursor(certificates::RELATION_ID, 0..1, small.clone())
                .next_chunk()
                .unwrap()
                .is_some()
        );
        assert!(
            cursor(
                solve_metrics::RELATION_ID,
                0..1,
                Arc::new("x".repeat(128 << 10))
            )
            .next_chunk()
            .unwrap()
            .is_some()
        );
    }

    #[test]
    fn trajectory_orders_bind_exact_sample_and_original_output_coordinates() {
        let public = (0..6)
            .map(|i| ResultOrder::Public.trajectory_coordinate(i, 2, 3).unwrap())
            .collect::<Vec<_>>();
        let canonical = (0..6)
            .map(|i| {
                ResultOrder::Canonical
                    .trajectory_coordinate(i, 2, 3)
                    .unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(public, vec![(0, 0), (0, 1), (1, 0), (1, 1), (2, 0), (2, 1)]);
        assert_eq!(
            canonical,
            vec![(0, 0), (1, 0), (2, 0), (0, 1), (1, 1), (2, 1)]
        );
        assert!(
            ResultOrder::Canonical
                .trajectory_coordinate(0, 0, 3)
                .is_err()
        );
        assert!(ResultOrder::Public.trajectory_coordinate(6, 2, 3).is_err());
    }

    #[test]
    fn demand_bounds_work_and_independent_cursors_keep_exact_rows() {
        let source = Source::new(64 << 20);
        let mut first = source
            .cursor(fit_parameters::RELATION_ID, 7, 31, None)
            .unwrap();
        let mut second = source
            .cursor(fit_parameters::RELATION_ID, 11, 31, None)
            .unwrap();
        assert_eq!(
            source.projected.load(Ordering::Relaxed),
            0,
            "construction is metadata only"
        );
        let chunk = first.next_chunk().unwrap().unwrap();
        assert_eq!(chunk.batch().num_rows(), 7);
        assert_eq!(
            source.projected.load(Ordering::Relaxed),
            7,
            "the next chunk must not be projected eagerly"
        );
        assert!(!first.complete());
        let other = second.next_chunk().unwrap().unwrap();
        assert_eq!(
            fit_parameters::Row::rows(&other).unwrap()[0].value,
            Some(0.0)
        );
        let mut values = fit_parameters::Row::rows(&chunk)
            .unwrap()
            .into_iter()
            .map(|row| row.value.unwrap())
            .collect::<Vec<_>>();
        while let Some(chunk) = first.next_chunk().unwrap() {
            assert!(chunk.batch().num_rows() <= 7);
            values.extend(
                fit_parameters::Row::rows(&chunk)
                    .unwrap()
                    .into_iter()
                    .map(|row| row.value.unwrap()),
            );
        }
        assert!(first.complete());
        assert_eq!(values, (0..31).map(|v| v as f64).collect::<Vec<_>>());
        drop(second);
        assert!(
            source
                .failures
                .failure(fit_parameters::RELATION_ID)
                .is_none(),
            "dropping a request cannot poison completion"
        );
    }

    #[test]
    fn selected_range_preserves_coordinates_without_validating_other_rows() {
        let source = Source::new(8 << 20);
        let mut cursor = source
            .range(fit_parameters::RELATION_ID, 2, 20, Some(3), 9..14)
            .unwrap();
        let mut values = Vec::new();
        while let Some(chunk) = cursor.next_chunk().unwrap() {
            values.extend(
                fit_parameters::Row::rows(&chunk)
                    .unwrap()
                    .into_iter()
                    .map(|row| row.value.unwrap()),
            );
        }
        assert_eq!(values, vec![9.0, 10.0, 11.0, 12.0, 13.0]);
        assert_eq!(source.projected.load(Ordering::Relaxed), 5);
        assert!(
            source
                .failures
                .failure(fit_parameters::RELATION_ID)
                .is_none()
        );
        assert!(cursor.complete());
    }

    #[test]
    fn intrinsic_failure_is_sticky_only_for_its_relation_and_prefix_is_incomplete() {
        let source = Source::new(64 << 20);
        let mut bad = source
            .cursor(fit_parameters::RELATION_ID, 2, 8, Some(3))
            .unwrap();
        assert_eq!(bad.next_chunk().unwrap().unwrap().batch().num_rows(), 2);
        let error = bad.next_chunk().unwrap_err();
        assert!(!bad.complete());
        let retry = source
            .clone()
            .cursor(fit_parameters::RELATION_ID, 1, 8, None)
            .unwrap_err();
        assert!(Arc::ptr_eq(&error, &retry));
        let mut unrelated = source
            .cursor(solve_metrics::RELATION_ID, 2, 8, Some(3))
            .unwrap();
        let empty = unrelated.next_chunk().unwrap().unwrap();
        assert_eq!(empty.relation_id(), solve_metrics::RELATION_ID);
        assert_eq!(empty.batch().num_rows(), 0);
        assert!(unrelated.next_chunk().unwrap().is_none());
        assert!(unrelated.complete());
    }

    #[test]
    fn cancelling_projection_interrupts_work_without_poisoning_completion() {
        let source = Source::new(8 << 20);
        let mut cursor = source
            .cursor(fit_parameters::RELATION_ID, 2, 20, None)
            .unwrap();
        assert_eq!(cursor.next_chunk().unwrap().unwrap().batch().num_rows(), 2);
        cursor.cancellation_token().cancel();
        assert!(cursor.next_chunk().is_err());
        assert!(!cursor.complete());
        assert!(
            source
                .failures
                .failure(fit_parameters::RELATION_ID)
                .is_none()
        );
        let mut retry = source
            .cursor(fit_parameters::RELATION_ID, 2, 20, None)
            .unwrap();
        while retry.next_chunk().unwrap().is_some() {}
        assert!(retry.complete());
    }

    #[test]
    fn cancelled_delivery_keeps_prefix_incomplete_and_allows_a_fresh_request() {
        let source = Source::new(8 << 20);
        let batch = source
            .cursor(fit_parameters::RELATION_ID, 2, 8, None)
            .unwrap()
            .next_chunk()
            .unwrap()
            .unwrap();
        let schema = batch.batch().schema();
        let completion = source.clone();
        let mut cursor = ResultCursor::new(
            schema,
            fit_parameters::RELATION_ID,
            2,
            ResultOrder::Public,
            move |projection| async move {
                projection
                    .sender
                    .send(batch)
                    .await
                    .map_err(|_| Arc::new(contract("fixture delivery dropped")))?;
                Err(completion.failures.record(
                    fit_parameters::RELATION_ID,
                    WorkflowError::Engine(pse_engine::EngineError::Cancelled),
                ))
            },
        )
        .unwrap();
        assert_eq!(cursor.next_chunk().unwrap().unwrap().batch().num_rows(), 2);
        assert!(cursor.next_chunk().is_err());
        assert!(!cursor.complete());
        assert!(
            source
                .failures
                .failure(fit_parameters::RELATION_ID)
                .is_none()
        );
        let mut retry = source
            .cursor(fit_parameters::RELATION_ID, 2, 8, None)
            .unwrap();
        let mut rows = 0;
        while let Some(batch) = retry.next_chunk().unwrap() {
            rows += batch.batch().num_rows();
        }
        assert_eq!(rows, 8);
        assert!(retry.complete());
    }

    #[test]
    fn whole_materialization_refusal_does_not_poison_bounded_export() {
        let source = Source::new(2 << 20);
        let mut whole = source
            .cursor(fit_parameters::RELATION_ID, 4096, 4096, None)
            .unwrap();
        assert!(whole.next_chunk().is_err());
        assert!(
            source
                .failures
                .failure(fit_parameters::RELATION_ID)
                .is_none()
        );
        drop(whole);
        let mut bounded = source
            .cursor(fit_parameters::RELATION_ID, 8, 4096, None)
            .unwrap();
        let mut rows = 0;
        while let Some(chunk) = bounded.next_chunk().unwrap() {
            rows += chunk.batch().num_rows();
        }
        assert_eq!(rows, 4096);
        assert!(bounded.complete());
    }

    #[test]
    fn escaped_arrays_keep_owners_after_source_and_cursor_drop() {
        let source = Source::new(4 << 20);
        let pool = source.pool.clone();
        let mut cursor = source
            .cursor(fit_parameters::RELATION_ID, 3, 10, None)
            .unwrap();
        let batch = cursor.next_chunk().unwrap().unwrap();
        let escaped = batch
            .checked_export(&CancellationToken::new())
            .unwrap()
            .column(3)
            .clone();
        drop((batch, cursor, source));
        assert!(pool.reserved() > 0);
        assert_eq!(escaped.len(), 3);
        drop(escaped);
        assert_eq!(pool.reserved(), 0);
    }

    #[tokio::test]
    async fn synchronous_collection_inside_executor_is_not_limited_by_channel_coop_budget() {
        let source = Source::new(8 << 20);
        let mut cursor = source
            .cursor(fit_parameters::RELATION_ID, 1, 300, None)
            .unwrap();
        let mut rows = 0;
        while let Some(chunk) = cursor.next_chunk().unwrap() {
            rows += chunk.batch().num_rows();
        }
        assert_eq!(rows, 300);
        assert!(cursor.complete());
    }
}
