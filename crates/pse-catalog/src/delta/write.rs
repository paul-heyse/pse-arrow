// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native Delta write body under the caller session; engine owns command lifetime.
use datafusion::{
    arrow::{
        array::{Int64Array, RecordBatch},
        datatypes::{Schema, SchemaRef},
    },
    common::{DataFusionError, Result},
    datasource::provider_as_source,
    execution::{TaskContext, session_state::SessionState},
    logical_expr::{Expr, LogicalPlan, LogicalPlanBuilder},
    physical_plan::{ExecutionPlan, SendableRecordBatchStream},
};
use deltalake::{DeltaTable, kernel::transaction::CommitProperties, protocol::SaveMode};
use pse_engine::operation::{Body, Definition, Family, Operation};
use std::sync::Arc;

#[derive(Debug)]
struct WriteRequest {
    table: DeltaTable,
    mode: SaveMode,
    commit: CommitProperties,
    contract: Option<super::contract::DeclaredCheck>,
    attempt: Option<super::attempt::MemberAttempt>,
    settle_dependencies: bool,
}
/// Constructors for typed native Delta write commands.
#[derive(Debug)]
pub struct DeltaWrite;
impl DeltaWrite {
    /// Construct a native command over a real relational input.
    /// # Errors
    /// The native outcome schema cannot be constructed.
    pub fn plan(
        table: DeltaTable,
        input: LogicalPlan,
        mode: SaveMode,
        commit: CommitProperties,
    ) -> Result<LogicalPlan> {
        Self::build(table, input, mode, commit, None, None)
    }
    /// Write an exact declared Arrow relation with its generated lossless layout
    /// and persisted native Delta CHECK. Existing tables must already bind it.
    /// # Errors
    /// Incompatible input fields, durable layout or existing table contract.
    pub fn declared(
        table: DeltaTable,
        input: LogicalPlan,
        mode: SaveMode,
        commit: CommitProperties,
        contract: super::contract::DeclaredCheck,
    ) -> Result<LogicalPlan> {
        Self::declared_attempt(table, input, mode, commit, contract, None)
    }
    pub(crate) fn declared_attempt(
        table: DeltaTable,
        input: LogicalPlan,
        mode: SaveMode,
        commit: CommitProperties,
        contract: super::contract::DeclaredCheck,
        attempt: Option<super::attempt::MemberAttempt>,
    ) -> Result<LogicalPlan> {
        if table.version().is_some() {
            contract.verify(&table)?;
        }
        let input = contract.layout().encode(input)?;
        Self::build(table, input, mode, commit, Some(contract), attempt)
    }
    fn build(
        table: DeltaTable,
        input: LogicalPlan,
        mode: SaveMode,
        commit: CommitProperties,
        contract: Option<super::contract::DeclaredCheck>,
        attempt: Option<super::attempt::MemberAttempt>,
    ) -> Result<LogicalPlan> {
        let settle_dependencies = contains_effects(&input)?;
        Ok(pse_engine::session::contract::ExecutionContract::plan(
            Operation::plan(
                Arc::new(WriteRequest {
                    table,
                    mode,
                    commit,
                    contract,
                    attempt,
                    settle_dependencies,
                }),
                vec![input],
            )?,
            None,
            effects(),
        ))
    }
}
pub(super) fn version_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        pse_schema::model::IntegerRange::NONNEGATIVE.field("version"),
    ]))
}
#[async_trait::async_trait]
impl Definition for WriteRequest {
    fn name(&self) -> &'static str {
        "DeltaWrite"
    }
    fn schema(&self) -> SchemaRef {
        version_schema()
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
        let state = state.clone();
        Ok(Arc::new(WriteBody {
            request: self,
            state: Arc::new(state),
        }))
    }
}
#[derive(Debug)]
struct WriteBody {
    request: Arc<WriteRequest>,
    state: Arc<SessionState>,
}
impl Body for WriteBody {
    fn execute(
        &self,
        inputs: Vec<Arc<dyn ExecutionPlan>>,
        _: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        let [input]: [_; 1] = inputs
            .try_into()
            .map_err(|_| DataFusionError::Plan("write requires one child".into()))?;
        let request = self.request.clone();
        let state = self.state.clone();
        Ok(pse_engine::operation::batch(version_schema(), async move {
            run(request, input, state).await
        }))
    }
}
async fn run(
    request: Arc<WriteRequest>,
    input: Arc<dyn ExecutionPlan>,
    state: Arc<SessionState>,
) -> Result<RecordBatch> {
    let schema = version_schema();
    let context = super::operation::DeltaOperationContext::new(
        state.clone(),
        request.contract.clone(),
        request.commit.clone(),
        super::operation::CommitKind::Data,
    )?;
    let state = context.state.clone();
    let _writer = context.begin(&request.table).await?;
    if let Some(attempt) = &request.attempt {
        let version = write_attempt(&request, attempt, input, &context)
            .await
            .map_err(|error| {
                error.context(format!(
                    "write publication member {}.{}",
                    attempt.member.schema_name, attempt.member.table_name
                ))
            })?;
        return Ok(RecordBatch::try_new(
            schema,
            vec![Arc::new(Int64Array::from(vec![version]))],
        )?);
    }
    let (input, _output_permit) = admit_output(&request, input, &state).await?;
    let input = LogicalPlanBuilder::scan(
        "prepared_delta_input",
        provider_as_source(Arc::new(
            pse_engine::session::physical_input::PhysicalInput::storage(input),
        )),
        None,
    )?
    .build()?;
    let table = if request.table.version().is_none() {
        if let Some(contract) = &request.contract {
            contract
                .create(request.table.clone(), request.commit.clone())
                .await?
        } else {
            request.table.clone()
        }
    } else {
        request.table.clone()
    };
    let created = request
        .table
        .version()
        .is_none()
        .then(|| table.version())
        .flatten();
    let mode = if created.is_some() && request.mode == SaveMode::ErrorIfExists {
        // The create operation already established exclusive ownership.
        SaveMode::Append
    } else {
        request.mode
    };
    let builder = context.write(table, input, mode);
    let table = builder.await.map_err(|error| match created {
        Some(version) => super::settlement::committed(version, error),
        None => super::settlement::unresolved(error),
    })?;
    let version = table.version().ok_or_else(|| {
        DataFusionError::Execution("Delta write omitted its committed version".into())
    })?;
    super::provider::committed(&table, &state).await;
    let version =
        super::settlement::observed(Some(version), super::provider::signed_version(version))?;
    Ok(RecordBatch::try_new(
        schema,
        vec![Arc::new(Int64Array::from(vec![version]))],
    )?)
}

#[tracing::instrument(name = "pse.delta.write_attempt", skip_all, fields(operation_id = %attempt.operation_id, attempt_id = %attempt.attempt_id, base_version = ?attempt.base_version, committed_version = tracing::field::Empty), err)]
async fn write_attempt(
    request: &WriteRequest,
    attempt: &super::attempt::MemberAttempt,
    input: Arc<dyn ExecutionPlan>,
    context: &super::operation::DeltaOperationContext,
) -> Result<i64> {
    let result = write_attempt_inner(request, attempt, input, context).await;
    if let Ok(version) = &result {
        tracing::Span::current().record("committed_version", *version);
    }
    result
}

#[expect(
    clippy::too_many_lines,
    reason = "provision, recover, execute and settle remain one ordered member-attempt state machine"
)]
async fn write_attempt_inner(
    request: &WriteRequest,
    attempt: &super::attempt::MemberAttempt,
    input: Arc<dyn ExecutionPlan>,
    context: &super::operation::DeltaOperationContext,
) -> Result<i64> {
    use super::attempt::{MemberState, Phase, rejected, unresolved};
    let state = context.state.clone();
    let contract = request
        .contract
        .as_ref()
        .ok_or_else(|| rejected("member attempt has no declared contract"))?;
    let mut table = request.table.clone();
    let mut snapshot_owner = None;
    let mut provision_permit = None;
    if table
        .verify_deltatable_existence()
        .await
        .map_err(unresolved)?
    {
        let opened = super::provider::open_native(
            table.log_store().root_url().clone(),
            None,
            crate::cache_service::snapshot::LoadRequirement::Query,
            &state,
        )
        .await?;
        table = opened.table.clone();
        snapshot_owner = opened.owner;
        contract.verify(&table)?;
        match attempt.inspect(&table, &state).await? {
            MemberState::Committed(version) => return super::provider::signed_version(version),
            MemberState::Provisioned(version) if table.version() == Some(version) => {}
            MemberState::Unpublished if table.version() == attempt.base_version => {}
            _ => {
                return Err(rejected("destination changed outside this member attempt"));
            }
        }
    } else if attempt.base_version.is_some() {
        return Err(unresolved("member base version is unavailable"));
    } else {
        provision_permit = output_permit(&state).await?;
        let (properties, _receipt_owner) = attempt.commit(Phase::Provisioned, &state)?;
        let created = contract.create(table.clone(), properties).await;
        let known_success = created.is_ok();
        let mut creation_error = None;
        table = match created {
            Ok(created) => created,
            Err(original) => {
                let original: Arc<deltalake::DeltaTableError> = Arc::new(original.into());
                creation_error = Some(original.clone());
                let opened = super::provider::open_native(
                    table.log_store().root_url().clone(),
                    None,
                    crate::cache_service::snapshot::LoadRequirement::Query,
                    &state,
                )
                .await
                .map_err(|error| {
                    unresolved(super::settlement::recovery_failed(original.clone(), error))
                })?;
                snapshot_owner = opened.owner;
                opened.table
            }
        };
        let observe = |error| match &creation_error {
            Some(primary) => unresolved(super::settlement::recovery_failed(primary.clone(), error)),
            None => match table.version() {
                Some(version) => super::settlement::committed(version, error),
                None => unresolved(error),
            },
        };
        contract.verify(&table).map_err(observe)?;
        if known_success {
            if table.version() != Some(0) {
                return Err(unresolved(
                    "new member provision returned an unexpected version",
                ));
            }
            super::provider::committed(&table, &state).await;
        } else {
            match attempt.inspect(&table, &state).await.map_err(observe)? {
                MemberState::Committed(version) => return super::provider::signed_version(version),
                MemberState::Provisioned(version) if table.version() == Some(version) => {}
                _ => return Err(rejected("destination was created by a different operation")),
            }
        }
    }
    // A recovered data commit returns before executing the real input child.
    let (input, _output_permit) = if request.settle_dependencies {
        // Provisioning/recovery never executes the input. Release its slot before
        // settling child writers, then admit the parent write without deadlocking.
        drop(provision_permit);
        admit_output(request, input, &state).await?
    } else {
        let permit = match provision_permit {
            Some(permit) => Some(permit),
            None => output_permit(&state).await?,
        };
        (input, permit)
    };
    let input = LogicalPlanBuilder::scan(
        "prepared_delta_input",
        provider_as_source(Arc::new(
            pse_engine::session::physical_input::PhysicalInput::storage(input),
        )),
        None,
    )?
    .build()?;
    let mode = if attempt.base_version.is_none() {
        SaveMode::Append
    } else {
        request.mode
    };
    let (properties, _receipt_owner) = attempt.commit(Phase::Written, &state)?;
    let context = context.with_commit(properties, super::operation::CommitKind::Publication);
    let expected = table
        .version()
        .and_then(|v| v.checked_add(1))
        .ok_or_else(|| unresolved("member write version overflows"))?;
    let proof = super::write_evidence::Pending::reserve(
        &state,
        attempt,
        contract,
        attempt.base_version.is_none() || mode == SaveMode::Overwrite,
    )?;
    let result = context.write(table, input, mode).await;
    let known_success = result.is_ok();
    let _snapshot_owner = snapshot_owner;
    let version = settle_attempt(request, attempt, contract, expected, result, &state).await?;
    if known_success && let Some(proof) = proof {
        proof.finish(expected);
    }
    Ok(version)
}

async fn settle_attempt(
    request: &WriteRequest,
    attempt: &super::attempt::MemberAttempt,
    contract: &super::contract::DeclaredCheck,
    expected: u64,
    result: std::result::Result<DeltaTable, deltalake::DeltaTableError>,
    state: &Arc<SessionState>,
) -> Result<i64> {
    use super::attempt::{MemberState, rejected, unresolved};
    let error = match result {
        Ok(table) => {
            super::settlement::observed(
                table.version(),
                (|| {
                    super::settlement::expected_version(table.version(), expected)?;
                    contract.verify(&table)
                })(),
            )?;
            super::provider::committed(&table, state).await;
            return super::provider::signed_version(expected);
        }
        Err(error) => Arc::new(error),
    };
    let observed = super::provider::open_native(
        request.table.log_store().root_url().clone(),
        None,
        crate::cache_service::snapshot::LoadRequirement::Query,
        state,
    )
    .await
    .map_err(|observation| {
        unresolved(super::settlement::recovery_failed(
            error.clone(),
            observation,
        ))
    })?;
    let receipt = async {
        contract.verify(&observed.table)?;
        attempt.inspect(&observed.table, state).await
    }
    .await
    .map_err(|observation| {
        unresolved(super::settlement::recovery_failed(
            error.clone(),
            observation,
        ))
    })?;
    match receipt {
        MemberState::Committed(version) => {
            super::provider::committed(&observed.table, state).await;
            super::provider::signed_version(version)
        }
        _ => Err(rejected(error)),
    }
}

fn effects() -> std::collections::BTreeSet<pse_schema::model::provider::OperationEffect> {
    use pse_schema::model::provider::OperationEffect::{Read, Write};
    [Read, Write].into_iter().collect()
}

fn contains_effects(input: &LogicalPlan) -> Result<bool> {
    use datafusion::common::tree_node::TreeNodeRecursion;
    let mut effects = false;
    let mut visited = std::collections::HashSet::new();
    let mut providers = std::collections::HashSet::new();
    // Retain every discovered view plan so address keys cannot be recycled.
    // Extension identity is its immutable owner, not a cloned enum wrapper.
    let mut roots = vec![Arc::new(input.clone())];
    let mut index = 0;
    while index < roots.len() && !effects {
        let root = Arc::clone(&roots[index]);
        index += 1;
        root.apply_with_subqueries(|plan| {
            if !matches!(plan, LogicalPlan::Subquery(_)) {
                let key = match plan {
                    LogicalPlan::Extension(extension) => {
                        (true, Arc::as_ptr(&extension.node).cast::<()>() as usize)
                    }
                    _ => (false, std::ptr::from_ref(plan) as usize),
                };
                if !visited.insert(key) {
                    return Ok(TreeNodeRecursion::Jump);
                }
            }
            match plan {
                LogicalPlan::Extension(extension) => {
                    if let Some(operation) = extension.node.as_any().downcast_ref::<Operation>() {
                        effects |= operation.family() != Family::Finite;
                    }
                    if let Some(contract) = extension
                        .node
                        .as_any()
                        .downcast_ref::<pse_engine::session::contract::ExecutionContract>(
                    ) {
                        effects |= contract.effects().iter().any(|effect| {
                            *effect != pse_schema::model::provider::OperationEffect::Read
                        });
                    }
                }
                LogicalPlan::TableScan(scan) => {
                    let provider = datafusion::datasource::source_as_provider(&scan.source)?;
                    if providers.insert(Arc::as_ptr(&provider).cast::<()>() as usize)
                        && let Some(plan) = provider.get_logical_plan()
                    {
                        roots.push(Arc::new(plan.into_owned()));
                    }
                }
                _ => {}
            }
            Ok(if effects {
                TreeNodeRecursion::Stop
            } else {
                TreeNodeRecursion::Continue
            })
        })?;
    }
    Ok(effects)
}

async fn admit_output(
    request: &WriteRequest,
    input: Arc<dyn ExecutionPlan>,
    state: &SessionState,
) -> Result<(
    Arc<dyn ExecutionPlan>,
    Option<tokio::sync::OwnedSemaphorePermit>,
)> {
    // An orchestrating writer cannot occupy a slot while awaiting a child writer.
    // Pure single-consumer inputs continue streaming into the native WriteBuilder.
    let input = if request.settle_dependencies {
        Arc::new(
            pse_engine::session::cache::Cached::collect_bounded(
                input,
                state.task_ctx(),
                &datafusion::physical_plan::metrics::ExecutionPlanMetricsSet::new(),
                usize::MAX,
            )
            .await?,
        )
        .reader_plan()?
    } else {
        input
    };
    let permit = output_permit(state).await?;
    Ok((input, permit))
}

async fn output_permit(state: &SessionState) -> Result<Option<tokio::sync::OwnedSemaphorePermit>> {
    if let Some(service) = state
        .config()
        .get_extension::<pse_engine::cache_service::NativeCacheService>()
    {
        let services = pse_engine::session::execution::NativeExecutionContext::from_session(state)?;
        Ok(Some(service.admit_output(services.cancellation()).await?))
    } else {
        Ok(None)
    }
}
