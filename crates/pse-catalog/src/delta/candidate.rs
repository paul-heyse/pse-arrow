// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Candidate admission (Plan 22 X9): after its member writes, a publication's complete
//! record is checked exactly as a reader will open it — every input opens under its
//! contract, every member binds and every relation obligation holds — and returned with
//! each member's actual version. Nothing becomes visible here: the catalog commit of
//! the returned record is the visibility boundary.
use datafusion::{
    arrow::array::RecordBatch,
    common::{DataFusionError, Result},
    execution::{TaskContext, session_state::SessionState},
    logical_expr::{Expr, LogicalPlan},
    physical_plan::{ExecutionPlan, SendableRecordBatchStream, execute_stream},
};
use futures_util::TryStreamExt;
use pse_engine::operation::{Body, Definition, Family, Operation};
use pse_relations::generated::runtime::publication_manifests;
use std::sync::Arc;

#[derive(Debug)]
struct Request {
    registry: Arc<pse_schema::Registry>,
    schema: datafusion::arrow::datatypes::SchemaRef,
}

/// Construct the admission of one candidate record over its member writes.
#[derive(Debug)]
pub struct AdmitCandidate;
impl AdmitCandidate {
    /// Admit the one candidate row `input` produces after all its native dependencies
    /// (the member writes) complete. The plan's output is that admitted row.
    /// # Errors
    /// The input is not a `runtime.publication_manifests` row.
    pub fn plan(input: LogicalPlan, registry: Arc<pse_schema::Registry>) -> Result<LogicalPlan> {
        let schema = schema()?;
        if input.schema().as_arrow().fields() != schema.fields() {
            return Err(invalid(
                "AdmitCandidate input must be runtime.publication_manifests",
            ));
        }
        Ok(pse_engine::session::contract::ExecutionContract::plan(
            Operation::plan(Arc::new(Request { registry, schema }), vec![input])?,
            None,
            effects(),
        ))
    }
}

fn schema() -> Result<datafusion::arrow::datatypes::SchemaRef> {
    publication_manifests::schema().map_err(pse_columnar::external)
}

#[async_trait::async_trait]
impl Definition for Request {
    fn name(&self) -> &'static str {
        "AdmitCandidate"
    }
    fn schema(&self) -> datafusion::arrow::datatypes::SchemaRef {
        Arc::clone(&self.schema)
    }
    fn family(&self) -> Family {
        Family::Command
    }
    fn effects(&self) -> std::collections::BTreeSet<pse_schema::model::provider::OperationEffect> {
        effects()
    }
    async fn prepare(
        self: Arc<Self>,
        _: &[Expr],
        _: &[LogicalPlan],
        _: &[Arc<dyn ExecutionPlan>],
        state: &SessionState,
    ) -> Result<Arc<dyn Body>> {
        Ok(Arc::new(AdmitBody {
            request: self,
            state: Arc::new(state.clone()),
        }))
    }
}

#[derive(Debug)]
struct AdmitBody {
    request: Arc<Request>,
    state: Arc<SessionState>,
}
impl Body for AdmitBody {
    fn execute(
        &self,
        inputs: Vec<Arc<dyn ExecutionPlan>>,
        _: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        let [input]: [_; 1] = inputs
            .try_into()
            .map_err(|_| invalid("candidate admission requires one child"))?;
        let request = self.request.clone();
        let state = self.state.clone();
        Ok(pse_engine::operation::batch(Arc::clone(&request.schema), async move {
            admit(request, input, state).await
        }))
    }
}

#[tracing::instrument(name = "pse.delta.admit_candidate", skip_all, err)]
async fn admit(
    request: Arc<Request>,
    input: Arc<dyn ExecutionPlan>,
    state: Arc<SessionState>,
) -> Result<RecordBatch> {
    let input = super::layout::declared_output(input, &request.schema)?;
    let batch = one_row(input, &state)
        .await
        .map_err(|error| error.context("collect the candidate record"))?;
    let registry = request.registry.as_ref();
    let record = publication_manifests::View::try_from_batch_with_registry(registry, &batch)
        .map_err(pse_columnar::external)?
        .row(0)
        .map_err(pse_columnar::external)?;
    if record.parent_publication_id == Some(record.publication_id) {
        return Err(invalid("a publication cannot be its own parent"));
    }
    super::publication::verify_inputs(&record.inputs, registry, Arc::clone(&state))
        .await
        .map_err(|error| error.context("open the candidate's exact inputs"))?;
    let candidate = super::publication::bind_members(&record.members, registry, Arc::clone(&state))
        .await
        .map_err(|error| error.context("bind the candidate's members"))?;
    super::admission::admit(&record, Arc::clone(&request.registry), &candidate)
        .await
        .map_err(|error| error.context("admit the candidate's members"))?;
    Ok(batch)
}

async fn one_row(input: Arc<dyn ExecutionPlan>, state: &SessionState) -> Result<RecordBatch> {
    let mut stream = execute_stream(input, state.task_ctx())?;
    let mut row = None;
    while let Some(batch) = stream.try_next().await? {
        if batch.num_rows() == 0 {
            continue;
        }
        if batch.num_rows() != 1 || row.is_some() {
            return Err(invalid("a candidate is exactly one record"));
        }
        row = Some(batch);
    }
    row.ok_or_else(|| invalid("a candidate is exactly one record"))
}

fn effects() -> std::collections::BTreeSet<pse_schema::model::provider::OperationEffect> {
    use pse_schema::model::provider::OperationEffect::{Read, Write};
    [Read, Write].into_iter().collect()
}

fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.into())
}
