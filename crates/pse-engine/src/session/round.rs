// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Stable finite-round providers. Each stream pins one immutable input epoch;
//! advancing any source while an earlier stream exists is an admission error.

use super::{EngineSession, engine};
use crate::{
    EngineError,
    provider::binding::{BindingKey, TableBinding},
};
use datafusion::{
    arrow::datatypes::SchemaRef,
    catalog::{Session, TableProvider, streaming::StreamingTable},
    common::{DataFusionError, Result, TableReference},
    execution::TaskContext,
    logical_expr::{Expr, TableType},
    physical_plan::{
        ExecutionPlan, SendableRecordBatchStream,
        stream::RecordBatchStreamAdapter,
        streaming::{PartitionStream, StreamingTableExec},
    },
};
#[cfg(test)]
use futures_util::TryStreamExt;
use pse_columnar::CancellationToken;
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::model::RelationKey;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

#[derive(Debug, Default)]
struct Epoch {
    inputs: BTreeMap<String, FieldCheckedBatch>,
    readers: usize,
}
/// An invocation's stable provider inventory. Retained values are field checked;
/// no changing row counts, constraints or ordering are advertised to the optimizer.
#[derive(Debug, Clone)]
pub struct RoundInputs(Arc<Mutex<Epoch>>);
impl RoundInputs {
    /// Replace a named source between fully settled executions.
    /// # Errors
    /// A live stream, unknown name, schema change or poisoned lock.
    pub fn replace(&self, name: &str, input: FieldCheckedBatch) -> Result<()> {
        self.replace_many(BTreeMap::from([(name.to_owned(), input)]))
    }
    /// Validate and install an entire epoch atomically. No input changes on failure.
    /// # Errors
    /// Live readers, an unknown name, schema change or poisoned lock.
    pub fn replace_many(&self, inputs: BTreeMap<String, FieldCheckedBatch>) -> Result<()> {
        let mut epoch = self.0.lock().map_err(|_| invalid("round lock poisoned"))?;
        if epoch.readers != 0 {
            return Err(invalid("cannot advance an epoch with live streams"));
        }
        for (name, input) in &inputs {
            let current = epoch
                .inputs
                .get(name)
                .ok_or_else(|| invalid("round source absent"))?;
            if current.batch().schema() != input.batch().schema()
                || current.relation_id() != input.relation_id()
            {
                return Err(invalid("round source schema changed"));
            }
        }
        epoch.inputs.extend(inputs);
        Ok(())
    }
}
impl EngineSession {
    /// Bind one stable source per relation and named role for an inner fixed point.
    /// These sources are private to the invocation and cannot become durable inputs.
    /// # Errors
    /// Colliding source, undeclared fields, cancellation or resource refusal.
    pub fn with_round_inputs(
        &self,
        relations: BTreeMap<RelationKey, FieldCheckedBatch>,
        roles: BTreeMap<String, FieldCheckedBatch>,
        cancel: &CancellationToken,
    ) -> std::result::Result<(Self, RoundInputs), EngineError> {
        let mut session = self.clone();
        let inputs = RoundInputs(Arc::new(Mutex::new(Epoch::default())));
        for (slot, name, reference, input) in relations
            .into_iter()
            .map(|(key, input)| {
                (
                    BindingKey::Relation(key),
                    key.qualified_name(),
                    TableReference::full("workspace", key.namespace.as_str(), key.name),
                    input,
                )
            })
            .chain(roles.into_iter().map(|(name, input)| {
                (
                    BindingKey::Input(name.clone()),
                    name.clone(),
                    TableReference::full("roles", "inputs", name),
                    input,
                )
            }))
        {
            cancel.checkpoint()?;
            let spec = session
                .registry
                .relation_by_id(input.relation_id())
                .ok_or_else(|| engine(invalid("round declaration absent")))?;
            input.check_declaration(&session.registry, spec)?;
            let input = input.retained(&session.pool, cancel)?;
            let schema = super::query_schema::schema(input.batch().schema().as_ref());
            let provider: Arc<dyn TableProvider> = Arc::new(RoundTable {
                partition: Arc::new(EpochPartition {
                    name: name.clone(),
                    inputs: inputs.clone(),
                    schema,
                }),
            });
            if inputs
                .0
                .lock()
                .map_err(|_| engine(invalid("round lock poisoned")))?
                .inputs
                .insert(name, input)
                .is_some()
            {
                return Err(engine(invalid("duplicate round source")));
            }
            session
                .bindings
                .insert(
                    slot,
                    TableBinding::new(reference, provider, Some(spec.key), None),
                )
                .map_err(engine)?;
        }
        Ok((session, inputs))
    }
}
#[derive(Debug)]
struct RoundTable {
    partition: Arc<dyn PartitionStream>,
}
#[async_trait::async_trait]
impl TableProvider for RoundTable {
    fn schema(&self) -> SchemaRef {
        self.partition.schema().clone()
    }
    fn table_type(&self) -> TableType {
        TableType::Temporary
    }
    async fn scan(
        &self,
        state: &dyn Session,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        if !filters.is_empty() {
            return Err(invalid("round sources do not push filters"));
        }
        StreamingTable::try_new(self.schema(), vec![self.partition.clone()])?
            .scan(state, projection, filters, limit)
            .await
    }
}
#[derive(Debug)]
struct EpochPartition {
    name: String,
    inputs: RoundInputs,
    schema: SchemaRef,
}
struct Reader(RoundInputs);
impl Drop for Reader {
    fn drop(&mut self) {
        if let Ok(mut epoch) = self.0.0.lock() {
            epoch.readers -= 1;
        }
    }
}
impl PartitionStream for EpochPartition {
    fn schema(&self) -> &SchemaRef {
        &self.schema
    }
    fn execute(&self, _: Arc<TaskContext>) -> SendableRecordBatchStream {
        let captured = (|| {
            let mut epoch = self
                .inputs
                .0
                .lock()
                .map_err(|_| invalid("round lock poisoned"))?;
            let input = epoch
                .inputs
                .get(&self.name)
                .cloned()
                .ok_or_else(|| invalid("round source absent"))?;
            epoch.readers += 1;
            Ok::<_, DataFusionError>((input, Reader(self.inputs.clone())))
        })();
        let (batch, owners) = match captured {
            Ok((input, reader)) => (
                super::query_schema::batch(input.batch()),
                Some((input, reader)),
            ),
            Err(error) => (Err(error), None),
        };
        let stream = futures_util::StreamExt::inspect(
            futures_util::stream::once(async move { batch }),
            move |_| {
                let _owners = &owners;
            },
        );
        Box::pin(RecordBatchStreamAdapter::new(self.schema.clone(), stream))
    }
}
/// Retain the actual admitted partitions; native streaming nodes are not all replayable.
pub(super) fn reset_partitions(session: &EngineSession) -> Vec<Arc<dyn PartitionStream>> {
    session
        .bindings
        .iter()
        .filter_map(|(_, binding)| {
            binding
                .provider
                .downcast_ref::<RoundTable>()
                .map(|table| table.partition.clone())
        })
        .collect()
}
fn invalid(message: &str) -> DataFusionError {
    DataFusionError::Plan(message.into())
}

/// Native reset is not itself an eligibility validator. Check actual operator
/// types; an arbitrary extension cannot obtain permission by copying a name.
pub(super) fn validate_reset_plan(
    plan: &Arc<dyn ExecutionPlan>,
    support: Option<&RoundResetSupport>,
    partitions: &[Arc<dyn PartitionStream>],
) -> Result<()> {
    use datafusion::common::tree_node::{TreeNode, TreeNodeRecursion};
    use datafusion::physical_plan::{
        aggregates::AggregateExec,
        coalesce_partitions::CoalescePartitionsExec,
        empty::EmptyExec,
        filter::FilterExec,
        joins::{CrossJoinExec, HashJoinExec, NestedLoopJoinExec, SortMergeJoinExec},
        limit::{GlobalLimitExec, LocalLimitExec},
        placeholder_row::PlaceholderRowExec,
        projection::ProjectionExec,
        repartition::RepartitionExec,
        sorts::{sort::SortExec, sort_preserving_merge::SortPreservingMergeExec},
        union::UnionExec,
    };
    let producers = plan.dynamic_expressions_produced();
    plan.apply_expressions(&mut |root| {
        // Native aggregate/sort constructors create producer state even when
        // pushdown is disabled. An unconsumed producer does not filter any input.
        // Reject every consumer occurrence (including nested references); only
        // the actual producer root may be ignored here.
        if producers.iter().any(|producer| Arc::ptr_eq(producer, root)) {
            return Ok(TreeNodeRecursion::Continue);
        }
        root.apply(|expr| {
            if expr.is::<datafusion::physical_expr::expressions::DynamicFilterPhysicalExpr>() {
                return Err(invalid(
                    "dynamic filters cannot retain state across round epochs",
                ));
            }
            Ok(TreeNodeRecursion::Continue)
        })
    })?;
    let supported = plan
        .downcast_ref::<StreamingTableExec>()
        .is_some_and(|stream| {
            stream
                .partitions()
                .iter()
                .all(|actual| partitions.iter().any(|known| Arc::ptr_eq(actual, known)))
        })
        || plan.is::<AggregateExec>()
        || plan.is::<CoalescePartitionsExec>()
        || plan.is::<EmptyExec>()
        || plan.is::<FilterExec>()
        || plan.is::<ProjectionExec>()
        || plan.is::<RepartitionExec>()
        || plan.is::<UnionExec>()
        || plan.is::<GlobalLimitExec>()
        || plan.is::<LocalLimitExec>()
        || plan.is::<SortExec>()
        || plan.is::<SortPreservingMergeExec>()
        || plan.is::<CrossJoinExec>()
        || plan.is::<HashJoinExec>()
        || plan.is::<NestedLoopJoinExec>()
        || plan.is::<SortMergeJoinExec>()
        || plan.is::<PlaceholderRowExec>()
        || plan.is::<datafusion::physical_plan::coop::CooperativeExec>()
        || plan.is::<datafusion::physical_plan::union::InterleaveExec>()
        || plan.is::<datafusion::physical_plan::unnest::UnnestExec>()
        || plan.is::<datafusion::physical_plan::windows::WindowAggExec>()
        || plan.is::<datafusion::physical_plan::windows::BoundedWindowAggExec>()
        || support.is_some_and(|support| (support.0)(plan.as_ref()))
        || plan.is::<super::contract::ContractExec>()
        || super::cache::supports_physical_reset(plan.as_ref())
        || plan
            .downcast_ref::<datafusion::datasource::source::DataSourceExec>()
            .is_some_and(|source| {
                source
                    .data_source()
                    .is::<datafusion::datasource::memory::MemorySourceConfig>()
            });
    if !supported {
        return Err(invalid(&format!(
            "{} has no admitted round reset contract",
            plan.name()
        )));
    }
    for child in plan.children() {
        validate_reset_plan(child, support, partitions)?;
    }
    if let Some(producer) = super::cache::reset_dependency(plan.as_ref()) {
        validate_reset_plan(producer, support, partitions)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::arrow::{
        array::{Int64Array, RecordBatch},
        datatypes::{DataType, Field},
    };
    use pse_schema::model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass};
    fn fixture() -> (pse_schema::Registry, FieldCheckedBatch, FieldCheckedBatch) {
        let mut registry = pse_schema::RegistryBuilder::new();
        registry.declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                "inputs",
                1,
                Authority::Authored,
                SnapshotClass::Model,
                "isolated round source",
            )
            .pk(&["value"])
            .columns(vec![FieldContract::from_field(Field::new(
                "value",
                DataType::Int64,
                false,
            ))]),
        );
        let registry = registry.build().unwrap();
        let spec = registry.relation("authored.inputs").unwrap();
        let schema = Arc::new(pse_schema::arrow::relation_schema(&registry, spec).unwrap());
        let empty =
            FieldCheckedBatch::admit(&registry, spec, RecordBatch::new_empty(schema.clone()))
                .unwrap();
        let values = FieldCheckedBatch::admit(
            &registry,
            spec,
            RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(vec![3, 7]))]).unwrap(),
        )
        .unwrap();
        (registry, empty, values)
    }
    #[test]
    fn epoch_replacement_is_atomic_when_a_later_source_is_invalid() {
        let (_, empty, values) = fixture();
        let inputs = RoundInputs(Arc::new(Mutex::new(Epoch {
            inputs: BTreeMap::from([("a".into(), empty.clone())]),
            readers: 0,
        })));
        assert!(
            inputs
                .replace_many(BTreeMap::from([
                    ("a".into(), values),
                    ("z-missing".into(), empty)
                ]))
                .is_err()
        );
        assert_eq!(inputs.0.lock().unwrap().inputs["a"].batch().num_rows(), 0);
    }
    #[tokio::test]
    async fn round_stream_blocks_turnover_and_retains_unknown_statistics() {
        let (_, empty, values) = fixture();
        let inputs = RoundInputs(Arc::new(Mutex::new(Epoch {
            inputs: BTreeMap::from([("x".into(), empty.clone())]),
            readers: 0,
        })));
        let table = RoundTable {
            partition: Arc::new(EpochPartition {
                name: "x".into(),
                inputs: inputs.clone(),
                schema: super::super::query_schema::schema(empty.batch().schema().as_ref()),
            }),
        };
        let state = datafusion::execution::session_state::SessionStateBuilder::new()
            .with_default_features()
            .build();
        let physical = table.scan(&state, None, &[], None).await.unwrap();
        assert_eq!(
            datafusion::physical_plan::statistics::StatisticsContext::new()
                .compute(
                    physical.as_ref(),
                    &datafusion::physical_plan::StatisticsArgs::new()
                )
                .unwrap()
                .num_rows,
            datafusion::common::stats::Precision::Absent
        );
        let stream = physical.execute(0, state.task_ctx()).unwrap();
        assert!(inputs.replace("x", values.clone()).is_err());
        drop(stream);
        inputs.replace("x", values).unwrap();
        let batches = physical
            .execute(0, state.task_ctx())
            .unwrap()
            .try_collect::<Vec<_>>()
            .await
            .unwrap();
        assert_eq!(batches.iter().map(RecordBatch::num_rows).sum::<usize>(), 2);
        inputs.replace("x", empty).unwrap();
    }
    #[tokio::test]
    async fn prepared_joins_observe_empty_then_nonempty_epochs_without_replanning() {
        for (hash_join, cached) in [false, true]
            .into_iter()
            .flat_map(|hash| [false, true].map(|cached| (hash, cached)))
        {
            let (registry, empty, values) = fixture();
            let key = registry.relation("authored.inputs").unwrap().key;
            let cancel = CancellationToken::new();
            let session = super::super::EngineFactory::new(
                Arc::new(datafusion::execution::runtime_env::RuntimeEnv::default()),
                Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20)),
                super::super::ExecutionSettings::default(),
                super::super::ThreadBudget {
                    pool_threads: 1.try_into().unwrap(),
                    target_partitions: 1.try_into().unwrap(),
                },
                super::super::native_engine_profile(),
            )
            .unwrap()
            .candidate(BTreeMap::new(), Arc::new(registry), &cancel)
            .unwrap();
            let (session, inputs) = session
                .with_round_inputs(BTreeMap::from([(key, empty)]), BTreeMap::new(), &cancel)
                .unwrap();
            let scan = datafusion::logical_expr::LogicalPlanBuilder::scan(
                session.table_reference(&key).unwrap(),
                session.table_source(&key).unwrap(),
                None,
            )
            .unwrap()
            .build()
            .unwrap();
            let right = datafusion::logical_expr::LogicalPlanBuilder::from(scan.clone())
                .alias("right")
                .unwrap()
                .build()
                .unwrap();
            let left = datafusion::logical_expr::LogicalPlanBuilder::from(scan)
                .alias("left")
                .unwrap();
            let plan = if hash_join {
                left.join_on(
                    right,
                    datafusion::logical_expr::JoinType::Inner,
                    [datafusion::logical_expr::col("left.value")
                        .eq(datafusion::logical_expr::col("right.value"))],
                )
                .unwrap()
            } else {
                left.cross_join(right).unwrap()
            }
            .build()
            .unwrap();
            let plan = if cached {
                session.cache_plan(plan, &cancel).unwrap()
            } else {
                plan
            };
            let mut program = session
                .prepare_rule_plan(plan, &cancel)
                .unwrap()
                .prepare_reusable(&cancel)
                .await
                .unwrap();
            let first = program.execute(&cancel).await.unwrap();
            assert_eq!(
                first
                    .batches()
                    .iter()
                    .map(|batch| batch.num_rows())
                    .sum::<usize>(),
                0
            );
            inputs.replace(&key.qualified_name(), values).unwrap();
            let second = program.execute(&cancel).await.unwrap();
            assert_eq!(
                second
                    .batches()
                    .iter()
                    .map(|batch| batch.num_rows())
                    .sum::<usize>(),
                if hash_join { 2 } else { 4 }
            );
            let cancelled = CancellationToken::new();
            cancelled.cancel();
            assert!(program.execute(&cancelled).await.is_err());
            assert!(
                program.execute(&cancel).await.is_err(),
                "failed epochs cannot be resumed with a fresh token"
            );
        }
    }
}

/// Explicit support for source-specific reset contracts outside the generic engine.
#[derive(Clone, Debug)]
pub struct RoundResetSupport(pub fn(&dyn ExecutionPlan) -> bool);
