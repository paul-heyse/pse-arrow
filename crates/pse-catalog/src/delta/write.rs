// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Validating Delta writes with native children and the actual invocation session.
//!
//! Delta's public builder accepts a logical input, while extension planning supplies
//! a physical child. The private provider below bridges those contracts without
//! collecting rows or re-planning the source. Delta still owns validation and commit.
use datafusion::{
    arrow::{
        array::{Int64Array, RecordBatch},
        datatypes::Schema,
    },
    catalog::Session,
    common::{DFSchema, DFSchemaRef, DataFusionError, Result},
    datasource::provider_as_source,
    execution::{TaskContext, session_state::SessionState},
    logical_expr::{
        Expr, Extension, LogicalPlan, LogicalPlanBuilder, UserDefinedLogicalNode,
        UserDefinedLogicalNodeCore, physical_planning_context::PhysicalPlanningContext,
    },
    physical_expr::EquivalenceProperties,
    physical_plan::{
        DisplayAs, DisplayFormatType, ExecutionPlan, Partitioning, PlanProperties,
        SendableRecordBatchStream,
        execution_plan::{Boundedness, EmissionType},
        stream::RecordBatchStreamAdapter,
    },
    physical_planner::{ExtensionPlanner, PhysicalPlanner},
};
use deltalake::{
    DeltaTable, delta_datafusion::SessionFallbackPolicy, kernel::transaction::CommitProperties,
    protocol::SaveMode,
};
use std::{
    hash::{Hash, Hasher},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

#[derive(Debug)]
struct WriteRequest {
    table: DeltaTable,
    mode: SaveMode,
    commit: CommitProperties,
    contract: Option<super::contract::DeclaredCheck>,
    attempt: Option<super::attempt::MemberAttempt>,
    started: AtomicBool,
}

/// A Delta write command. Planning and EXPLAIN are side-effect free; collecting its
/// outcome executes one validating commit and returns its actual version.
#[derive(Clone)]
pub struct DeltaWrite {
    request: Arc<WriteRequest>,
    input: LogicalPlan,
    schema: DFSchemaRef,
}
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
        Self::build(
            table,
            input,
            mode,
            commit.with_max_retries(0),
            Some(contract),
            attempt,
        )
    }
    fn build(
        table: DeltaTable,
        input: LogicalPlan,
        mode: SaveMode,
        commit: CommitProperties,
        contract: Option<super::contract::DeclaredCheck>,
        attempt: Option<super::attempt::MemberAttempt>,
    ) -> Result<LogicalPlan> {
        let schema = Arc::new(DFSchema::try_from(Schema::new(vec![
            pse_schema::model::IntegerRange::NONNEGATIVE.field("version"),
        ]))?);
        Ok(crate::session::contract::ExecutionContract::plan(
            LogicalPlan::Extension(Extension {
                node: Arc::new(Self {
                    request: Arc::new(WriteRequest {
                        table,
                        mode,
                        commit,
                        contract,
                        attempt,
                        started: AtomicBool::new(false),
                    }),
                    input,
                    schema,
                }),
            }),
            None,
            effects(),
        ))
    }
}
impl PartialEq for DeltaWrite {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.request, &other.request) && self.input == other.input
    }
}
impl Eq for DeltaWrite {}
impl Hash for DeltaWrite {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.request).hash(state);
        self.input.hash(state);
    }
}
impl PartialOrd for DeltaWrite {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match Arc::as_ptr(&self.request).cmp(&Arc::as_ptr(&other.request)) {
            std::cmp::Ordering::Equal => self.input.partial_cmp(&other.input),
            order => Some(order),
        }
    }
}
// Native plan renderers visit children separately. Debug describes this
// node without recursively duplicating complete subgraphs in JSON.
impl std::fmt::Debug for DeltaWrite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        UserDefinedLogicalNodeCore::fmt_for_explain(self, f)
    }
}
impl UserDefinedLogicalNodeCore for DeltaWrite {
    fn name(&self) -> &'static str {
        "DeltaWrite"
    }
    fn inputs(&self) -> Vec<&LogicalPlan> {
        vec![&self.input]
    }
    fn schema(&self) -> &DFSchemaRef {
        &self.schema
    }
    fn expressions(&self) -> Vec<Expr> {
        Vec::new()
    }
    fn fmt_for_explain(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DeltaWrite: mode={:?}", self.request.mode)
    }
    fn with_exprs_and_inputs(
        &self,
        exprs: Vec<Expr>,
        mut inputs: Vec<LogicalPlan>,
    ) -> Result<Self> {
        if !exprs.is_empty() || inputs.len() != 1 {
            return Err(DataFusionError::Plan(
                "DeltaWrite requires one input and no expressions".into(),
            ));
        }
        let input = inputs
            .pop()
            .ok_or_else(|| DataFusionError::Plan("DeltaWrite input absent".into()))?;
        Ok(Self {
            input,
            ..self.clone()
        })
    }
    fn prevent_predicate_push_down_columns(&self) -> std::collections::HashSet<String> {
        self.schema
            .fields()
            .iter()
            .map(|f| f.name().to_owned())
            .collect()
    }
}

/// Lowers a Delta command wherever it occurs in a native logical plan.
#[derive(Debug)]
pub(crate) struct WritePlanner;
#[async_trait::async_trait]
impl ExtensionPlanner for WritePlanner {
    async fn plan_extension(
        &self,
        _: &dyn PhysicalPlanner,
        node: &dyn UserDefinedLogicalNode,
        _: &[&LogicalPlan],
        inputs: &[Arc<dyn ExecutionPlan>],
        session: &dyn Session,
        _: &PhysicalPlanningContext,
    ) -> Result<Option<Arc<dyn ExecutionPlan>>> {
        let Some(node) = node.as_any().downcast_ref::<DeltaWrite>() else {
            return Ok(None);
        };
        crate::session::execution::NativeExecutionContext::from_session(session)?
            .admit_effects(&effects())
            .map_err(|error| DataFusionError::External(Box::new(error)))?;
        let state = session
            .as_any()
            .downcast_ref::<SessionState>()
            .ok_or_else(|| {
                DataFusionError::Plan("DeltaWrite requires the actual SessionState".into())
            })?;
        let [input] = inputs else {
            return Err(DataFusionError::Plan(
                "DeltaWrite requires one physical child".into(),
            ));
        };
        let state = match &node.request.contract {
            Some(contract) => contract.bind(state)?,
            None => state.clone(),
        };
        Ok(Some(Arc::new(WriteExec {
            request: Arc::clone(&node.request),
            input: Arc::clone(input),
            state: Arc::new(state),
            properties: Arc::new(PlanProperties::new(
                EquivalenceProperties::new(Arc::new(node.schema.as_arrow().clone())),
                Partitioning::UnknownPartitioning(1),
                EmissionType::Final,
                Boundedness::Bounded,
            )),
        })))
    }
}

#[derive(Debug)]
struct WriteExec {
    request: Arc<WriteRequest>,
    input: Arc<dyn ExecutionPlan>,
    state: Arc<SessionState>,
    properties: Arc<PlanProperties>,
}
impl DisplayAs for WriteExec {
    fn fmt_as(&self, _: DisplayFormatType, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DeltaWriteExec: validating builder, caller session")
    }
}
impl ExecutionPlan for WriteExec {
    fn apply_expressions(
        &self,
        _: &mut dyn FnMut(
            &Arc<dyn datafusion::physical_expr::PhysicalExpr>,
        ) -> Result<datafusion::common::tree_node::TreeNodeRecursion>,
    ) -> Result<datafusion::common::tree_node::TreeNodeRecursion> {
        Ok(datafusion::common::tree_node::TreeNodeRecursion::Continue)
    }
    fn name(&self) -> &'static str {
        "DeltaWriteExec"
    }
    fn properties(&self) -> &Arc<PlanProperties> {
        &self.properties
    }
    fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
        vec![&self.input]
    }
    fn with_new_children(
        self: Arc<Self>,
        mut children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        if children.len() != 1 {
            return Err(DataFusionError::Plan(
                "DeltaWrite requires one physical child".into(),
            ));
        }
        let input = children
            .pop()
            .ok_or_else(|| DataFusionError::Plan("DeltaWrite input absent".into()))?;
        Ok(Arc::new(Self {
            input,
            request: Arc::clone(&self.request),
            state: Arc::clone(&self.state),
            properties: Arc::clone(&self.properties),
        }))
    }
    fn execute(&self, partition: usize, _: Arc<TaskContext>) -> Result<SendableRecordBatchStream> {
        if partition != 0 || self.request.started.swap(true, Ordering::AcqRel) {
            return Err(DataFusionError::Execution(
                "a prepared Delta write executes once in partition zero".into(),
            ));
        }
        let input = Arc::clone(&self.input);
        let request = Arc::clone(&self.request);
        let state = Arc::clone(&self.state);
        let schema = self.schema();
        let output_schema = Arc::clone(&schema);
        let stream = futures_util::stream::once(async move {
            let services =
                crate::session::execution::NativeExecutionContext::from_session(state.as_ref())?;
            let _writer =
                super::lease::write(request.table.table_url(), services.cancellation()).await?;
            services.require_settlement();
            if let Some(attempt) = &request.attempt {
                let version = write_attempt(&request, attempt, input, state)
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
            let input = LogicalPlanBuilder::scan(
                "prepared_delta_input",
                provider_as_source(Arc::new(
                    crate::session::physical_input::PhysicalInput::storage(input),
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
            let builder = table
                .write(Vec::<RecordBatch>::new())
                .with_input_plan(input)
                .with_session_state(state.clone())
                .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
                .with_save_mode(mode)
                .with_commit_properties(request.commit.clone());
            let table = builder.await.map_err(|error| match created {
                Some(version) => {
                    DataFusionError::External(Box::new(super::dml::MutationError::Committed {
                        version,
                        source: Box::new(error),
                    }))
                }
                None => DataFusionError::External(Box::new(error)),
            })?;
            let version = table.version().ok_or_else(|| {
                DataFusionError::Execution("Delta write omitted its committed version".into())
            })?;
            let version = super::provider::signed_version(version).map_err(|error| {
                DataFusionError::External(Box::new(super::dml::MutationError::Committed {
                    version,
                    source: Box::new(error),
                }))
            })?;
            Ok(RecordBatch::try_new(
                schema,
                vec![Arc::new(Int64Array::from(vec![version]))],
            )?)
        });
        Ok(Box::pin(RecordBatchStreamAdapter::new(
            output_schema,
            stream,
        )))
    }
}

async fn write_attempt(
    request: &WriteRequest,
    attempt: &super::attempt::MemberAttempt,
    input: Arc<dyn ExecutionPlan>,
    state: Arc<SessionState>,
) -> Result<i64> {
    use super::attempt::{MemberState, Phase, rejected, unresolved};
    let contract = request
        .contract
        .as_ref()
        .ok_or_else(|| rejected("member attempt has no declared contract".into()))?;
    let mut table = request.table.clone();
    let mut snapshot_owner = None;
    if table
        .verify_deltatable_existence()
        .await
        .map_err(|error| unresolved(error.to_string()))?
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
                return Err(rejected(
                    "destination changed outside this member attempt".into(),
                ));
            }
        }
    } else if attempt.base_version.is_some() {
        return Err(unresolved("member base version is unavailable".into()));
    } else {
        let (properties, _receipt_owner) = attempt.commit(Phase::Provisioned, &state)?;
        let created = contract.create(table.clone(), properties).await;
        table = match created {
            Ok(created) => created,
            Err(original) => {
                let opened = super::provider::open_native(
                    table.log_store().root_url().clone(),
                    None,
                    crate::cache_service::snapshot::LoadRequirement::Query,
                    &state,
                )
                .await
                .map_err(|error| unresolved(format!("create: {original}; observation: {error}")))?;
                snapshot_owner = opened.owner;
                opened.table
            }
        };
        contract.verify(&table)?;
        match attempt.inspect(&table, &state).await? {
            MemberState::Committed(version) => return super::provider::signed_version(version),
            MemberState::Provisioned(version) if table.version() == Some(version) => {}
            _ => {
                return Err(rejected(
                    "destination was created by a different operation".into(),
                ));
            }
        }
    }
    // A recovered data commit returns before executing the real input child.
    let input = LogicalPlanBuilder::scan(
        "prepared_delta_input",
        provider_as_source(Arc::new(
            crate::session::physical_input::PhysicalInput::storage(input),
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
    let result = table
        .write(Vec::<RecordBatch>::new())
        .with_input_plan(input)
        .with_session_state(state.clone())
        .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
        .with_save_mode(mode)
        .with_commit_properties(properties)
        .await;
    let _snapshot_owner = snapshot_owner;
    settle_attempt(request, attempt, contract, result, &state).await
}

async fn settle_attempt(
    request: &WriteRequest,
    attempt: &super::attempt::MemberAttempt,
    contract: &super::contract::DeclaredCheck,
    result: std::result::Result<DeltaTable, deltalake::DeltaTableError>,
    state: &Arc<SessionState>,
) -> Result<i64> {
    use super::attempt::{MemberState, rejected, unresolved};
    let (observed, original_error) = match result {
        Ok(table) => (super::provider::Opened { table, owner: None }, None),
        Err(error) => {
            let opened = super::provider::open_native(
                request.table.log_store().root_url().clone(),
                None,
                crate::cache_service::snapshot::LoadRequirement::Query,
                state,
            )
            .await
            .map_err(|error| unresolved(error.to_string()))?;
            (opened, Some(error))
        }
    };
    contract.verify(&observed.table)?;
    match attempt.inspect(&observed.table, state).await? {
        MemberState::Committed(version) => {
            super::provider::committed(&observed.table, state).await;
            super::provider::signed_version(version)
        }
        _ => match original_error {
            None => Err(unresolved(
                "native write returned success without its committed transaction".into(),
            )),
            Some(error) => Err(rejected(error.to_string())),
        },
    }
}

fn effects() -> std::collections::BTreeSet<pse_schema::model::provider::OperationEffect> {
    use pse_schema::model::provider::OperationEffect::{Read, Write};
    [Read, Write].into_iter().collect()
}
