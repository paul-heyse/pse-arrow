// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual analyzer/optimizer observations and the plan that reached physical planning.
use crate::EngineError;
use datafusion::logical_expr::LogicalPlan;

use pse_columnar::{AllocationLease, MemoryPool};
use std::{
    fmt::Write,
    sync::{Arc, OnceLock},
};

mod graph;

const MAX_RULES: usize = u16::MAX as usize;
// Optional capture has its own bounded pool so it cannot consume execution admission.
pub(super) fn diagnostic_pool() -> Arc<dyn MemoryPool> {
    static POOL: std::sync::LazyLock<Arc<dyn MemoryPool>> =
        std::sync::LazyLock::new(|| Arc::new(pse_columnar::GreedyMemoryPool::new(16 << 20)));
    POOL.clone()
}
/// Retained diagnostic evidence; never a semantic validation or reuse certificate.
#[derive(Clone, Debug)]
pub struct PlanObservation {
    logical: Arc<ObservedPlan>,
    physical: Option<Arc<ObservedPhysical>>,
}
#[derive(Debug)]
struct ObservedPlan {
    captured: bool,
    explain: String,
    rules: Vec<String>,
    diagnostic: Option<String>,
    _lease: Arc<AllocationLease>,
}
#[derive(Debug)]
struct ObservedPhysical {
    explain: String,
    execution: u64,
    completion: OnceLock<ExecutionSnapshot>,
    diagnostic: Option<String>,
    _lease: Arc<AllocationLease>,
}
/// Native metric units remain explicit; an absent value is never fabricated as zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetricUnit {
    /// Rows.
    Rows,
    /// Batches.
    Batches,
    /// Bytes.
    Bytes,
    /// Native elapsed time, in nanoseconds.
    Nanoseconds,
    /// Dimensionless counter or gauge.
    Count,
    /// Library-defined diagnostic representation.
    Native,
}
/// One metric from one node and partition of the executed graph.
#[derive(Clone, Debug)]
pub struct ObservedMetric {
    /// Stable node ordinal within this execution snapshot.
    pub node: usize,
    /// Native operator name.
    pub operator: String,
    /// Native metric name.
    pub name: String,
    /// Native partition, absent for a global metric.
    pub partition: Option<usize>,
    /// Native labels, retaining metric scope.
    pub labels: Vec<(String, String)>,
    /// Explicit units for the captured value.
    pub unit: MetricUnit,
    /// Exact native numeric counter, absent for compound/custom/timestamp values.
    pub value: Option<usize>,
}
/// Bounded terminal evidence, with no retained execution plan or stream.
#[derive(Debug)]
pub struct ExecutionSnapshot {
    /// Unique invocation identity, independent of mathematical reuse.
    pub execution: u64,
    /// Whether the invocation exhausted, failed, cancelled or was dropped.
    pub status: super::assurance::TerminalStatus,
    /// Unique native graph nodes and partition metrics.
    pub metrics: Vec<ObservedMetric>,
    /// Capture was curtailed by a diagnostic limit or allocation refusal.
    pub truncated: bool,
    _lease: Arc<AllocationLease>,
}
/// Stream-local ownership makes partial and abandoned executions observable too.
pub(super) struct CompletionGuard {
    plan: Option<Arc<dyn datafusion::physical_plan::ExecutionPlan>>,
    observation: PlanObservation,
    pool: Arc<dyn MemoryPool>,
}
impl CompletionGuard {
    pub(super) fn new(
        plan: &Arc<dyn datafusion::physical_plan::ExecutionPlan>,
        observation: &PlanObservation,
        pool: &Arc<dyn MemoryPool>,
    ) -> Self {
        Self {
            plan: observation.is_captured().then(|| plan.clone()),
            observation: observation.clone(),
            pool: pool.clone(),
        }
    }
    pub(super) fn finish(&mut self, status: super::assurance::TerminalStatus) {
        if let Some(plan) = self.plan.take() {
            self.observation.complete(plan.as_ref(), status, &self.pool);
        }
    }
}
impl Drop for CompletionGuard {
    fn drop(&mut self) {
        self.finish(super::assurance::TerminalStatus::Abandoned);
    }
}
impl PlanObservation {
    /// Bounded terminal metrics from the actual execution, when capture was requested.
    pub fn execution(&self) -> Option<&ExecutionSnapshot> {
        self.physical.as_ref()?.completion.get()
    }
    /// Optional capture failure. It never changes the execution's result.
    pub fn diagnostic(&self) -> Option<&str> {
        self.physical
            .as_ref()
            .and_then(|p| p.diagnostic.as_deref())
            .or(self.logical.diagnostic.as_deref())
    }
    pub(super) fn complete(
        &self,
        plan: &dyn datafusion::physical_plan::ExecutionPlan,
        status: super::assurance::TerminalStatus,
        pool: &Arc<dyn MemoryPool>,
    ) {
        if let Some(physical) = &self.physical {
            physical
                .completion
                .get_or_init(|| snapshot(plan, physical.execution, status, pool));
        }
    }
    /// Whether diagnostic capture was explicitly requested.
    pub fn is_captured(&self) -> bool {
        self.logical.captured
    }
    /// Native PostgreSQL JSON for each actual optimized node, in a flat diagnostic
    /// graph. `Root` and ordered `Edges` reference `Nodes[].Id`; each node's `Native`
    /// value is DataFusion's own JSON. Shared nodes have one definition. No live
    /// provider, buffer or execution owner is retained by this evidence.
    pub fn explain_pgjson(&self) -> &str {
        &self.logical.explain
    }
    /// Analyzer and logical optimizer callbacks in their actual execution order.
    pub fn rules_fired(&self) -> &[String] {
        &self.logical.rules
    }
    /// The actual native physical graph, including independently planned cache producers.
    /// A preparation alone has no physical observation and executes no data operators.
    pub fn physical_plan(&self) -> Option<&str> {
        self.physical.as_ref().map(|value| value.explain.as_str())
    }
    pub(super) fn with_physical(
        &self,
        plan: &dyn datafusion::physical_plan::ExecutionPlan,
        pool: &Arc<dyn MemoryPool>,
    ) -> Result<Self, EngineError> {
        if !self.is_captured() {
            return Ok(self.clone());
        }
        let _ = pool;
        let capture_pool = diagnostic_pool();
        let pool = &capture_pool;
        let mut reservation =
            pse_columnar::MemoryConsumer::new("session:physical-plan-observation").register(pool);
        let mut writer = BoundedText {
            value: String::new(),
            reservation: &mut reservation,
            error: None,
        };
        let rendered = render_physical(plan, &mut writer, pool);
        let diagnostic = writer
            .error
            .or_else(|| rendered.err())
            .map(|e| e.to_string());
        let explain = if diagnostic.is_some() {
            String::new()
        } else {
            writer.value
        };
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        Ok(Self {
            logical: Arc::clone(&self.logical),
            physical: Some(Arc::new(ObservedPhysical {
                explain,
                execution: NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                completion: OnceLock::new(),
                diagnostic,
                _lease: AllocationLease::new(reservation),
            })),
        })
    }
}

fn snapshot(
    root: &dyn datafusion::physical_plan::ExecutionPlan,
    execution: u64,
    status: super::assurance::TerminalStatus,
    pool: &Arc<dyn MemoryPool>,
) -> ExecutionSnapshot {
    use datafusion::physical_plan::metrics::MetricValue as V;
    let _ = pool;
    let capture_pool = diagnostic_pool();
    let pool = &capture_pool;
    let reservation = pse_columnar::MemoryConsumer::new("session:completed-metrics").register(pool);
    let mut metrics = Vec::new();
    let mut visited = std::collections::HashSet::new();
    let mut pending = vec![root];
    let mut truncated = false;
    'nodes: while let Some(plan) = pending.pop() {
        let identity = std::ptr::from_ref(plan).cast::<()>() as usize;
        if visited.contains(&identity) {
            continue;
        }
        if visited.len() >= 4096 || reservation.try_grow(128).is_err() {
            truncated = true;
            break;
        }
        let node = visited.len();
        visited.insert(identity);
        if let Some(native) = plan.metrics() {
            for metric in native.iter() {
                let value = metric.value();
                let text_size = plan
                    .name()
                    .len()
                    .saturating_add(value.name().len())
                    .saturating_add(
                        metric
                            .labels()
                            .iter()
                            .map(|l| {
                                l.name()
                                    .len()
                                    .saturating_add(l.value().len())
                                    .saturating_add(2 * size_of::<String>())
                            })
                            .sum::<usize>(),
                    );
                if metrics.len() >= 4096
                    || text_size > 4096
                    || reservation
                        .try_grow(text_size.saturating_add(2 * size_of::<ObservedMetric>()))
                        .is_err()
                {
                    truncated = true;
                    break 'nodes;
                }
                let (unit, numeric) = match value {
                    V::OutputRows(_) | V::SpilledRows(_) => (MetricUnit::Rows, true),
                    V::OutputBatches(_) => (MetricUnit::Batches, true),
                    V::SpilledBytes(_)
                    | V::OutputBytes(_)
                    | V::CurrentMemoryUsage(_)
                    | V::PeakMemoryUsage { .. } => (MetricUnit::Bytes, true),
                    V::ElapsedCompute(_) | V::Time { .. } => (MetricUnit::Nanoseconds, true),
                    V::SpillCount(_) | V::Count { .. } | V::Gauge { .. } => {
                        (MetricUnit::Count, true)
                    }
                    _ => (MetricUnit::Native, false),
                };
                metrics.push(ObservedMetric {
                    node,
                    operator: plan.name().into(),
                    name: value.name().into(),
                    partition: metric.partition(),
                    labels: metric
                        .labels()
                        .iter()
                        .map(|l| (l.name().into(), l.value().into()))
                        .collect(),
                    unit,
                    value: numeric.then(|| value.as_usize()),
                });
            }
        }
        let children = super::cache::reset_dependency(plan)
            .map_or_else(|| plan.children(), |input| vec![input]);
        if pending.len().saturating_add(children.len()) > 4096
            || reservation
                .try_grow(children.len().saturating_mul(size_of::<usize>() * 2))
                .is_err()
        {
            truncated = true;
            break;
        }
        pending.extend(children.into_iter().rev().map(AsRef::as_ref));
    }
    ExecutionSnapshot {
        execution,
        status,
        metrics,
        truncated,
        _lease: AllocationLease::new(reservation),
    }
}

fn render_physical(
    root: &dyn datafusion::physical_plan::ExecutionPlan,
    writer: &mut impl Write,
    pool: &Arc<dyn MemoryPool>,
) -> Result<(), EngineError> {
    let reservation =
        pse_columnar::MemoryConsumer::new("session:physical-observation-traversal").register(pool);
    let mut visited = std::collections::HashSet::new();
    let mut pending = vec![(root, 0)];
    while let Some((plan, depth)) = pending.pop() {
        let identity = std::ptr::from_ref(plan).cast::<()>() as usize;
        reservation.try_grow(4 * size_of::<usize>())?;
        for _ in 0..depth {
            writer
                .write_str("  ")
                .map_err(|_| invalid("physical plan rendering failed"))?;
        }
        writeln!(
            writer,
            "{}",
            datafusion::physical_plan::displayable(plan).one_line()
        )
        .map_err(|_| invalid("physical plan rendering failed"))?;
        if !visited.insert(identity) {
            continue;
        }
        // Optimizer children deliberately stop at an independently planned
        // producer. Diagnostic traversal still includes that actual owner.
        let children = super::cache::reset_dependency(plan)
            .map_or_else(|| plan.children(), |input| vec![input]);
        reservation.try_grow(children.len().saturating_mul(3 * size_of::<usize>()))?;
        pending.extend(
            children
                .into_iter()
                .rev()
                .map(|child| (child.as_ref(), depth + 1)),
        );
    }
    Ok(())
}
pub(super) struct Recorder {
    enabled: bool,
    reservation: pse_columnar::MemoryReservation,
    rules: Vec<String>,
    error: Option<EngineError>,
}
impl Recorder {
    pub(super) fn new(
        _pool: &Arc<dyn MemoryPool>,
        policy: super::assurance::ObservationPolicy,
    ) -> Self {
        Self {
            enabled: policy == crate::session::assurance::ObservationPolicy::Diagnostic,
            reservation: pse_columnar::MemoryConsumer::new("session:plan-observation")
                .register(&diagnostic_pool()),
            rules: Vec::new(),
            error: None,
        }
    }
    pub(super) fn rule(&mut self, name: &str) {
        if !self.enabled || self.error.is_some() {
            return;
        }
        if self.rules.len() >= MAX_RULES {
            self.error = Some(invalid("observed rule count exceeds u16 evidence contract"));
            return;
        }
        let grow = name.len().saturating_add(2 * size_of::<String>());
        if let Err(error) = self.reservation.try_grow(grow) {
            self.error = Some(error.into());
            return;
        }
        self.rules.reserve_exact(1);
        self.rules.push(name.to_owned());
    }
    pub(super) fn finish(
        mut self,
        plan: &LogicalPlan,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<PlanObservation, EngineError> {
        let prior_error = self.error.take();
        let mut writer = BoundedText {
            value: String::new(),
            reservation: &mut self.reservation,
            error: None,
        };
        let rendered = if self.enabled {
            graph::write(plan, &mut writer, cancel)
        } else {
            Ok(())
        };
        let diagnostic = prior_error
            .or(writer.error)
            .or_else(|| rendered.err())
            .map(|e| e.to_string());
        let explain = if diagnostic.is_some() {
            String::new()
        } else {
            writer.value
        };
        Ok(PlanObservation {
            logical: Arc::new(ObservedPlan {
                captured: self.enabled,
                explain,
                rules: self.rules,
                diagnostic,
                _lease: AllocationLease::new(self.reservation),
            }),
            physical: None,
        })
    }
}
struct BoundedText<'a> {
    value: String,
    reservation: &'a mut pse_columnar::MemoryReservation,
    error: Option<EngineError>,
}
impl Write for BoundedText<'_> {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        let Some(required) = self.value.len().checked_add(text.len()) else {
            self.error = Some(invalid("plan rendering size overflow"));
            return Err(std::fmt::Error);
        };
        if required > (1 << 20) {
            self.error = Some(invalid("diagnostic plan exceeds 1 MiB capture bound"));
            return Err(std::fmt::Error);
        }
        let current = self.value.capacity();
        if required > current {
            // Native display emits many short writes. Reserving exactly each
            // fragment repeatedly copies the entire accumulated plan. Charge
            // amortized capacity before allocation, with an exact-size fallback
            // when the remaining budget cannot admit speculative spare space.
            let preferred = required.checked_next_power_of_two().unwrap_or(required);
            let capacity = if self.reservation.try_grow(preferred - current).is_ok() {
                preferred
            } else {
                if let Err(error) = self.reservation.try_grow(required - current) {
                    self.error = Some(error.into());
                    return Err(std::fmt::Error);
                }
                required
            };
            if self
                .value
                .try_reserve_exact(capacity - self.value.len())
                .is_err()
            {
                self.error = Some(invalid("plan rendering allocation failed"));
                return Err(std::fmt::Error);
            }
            if self.value.capacity() > capacity
                && let Err(error) = self.reservation.try_grow(self.value.capacity() - capacity)
            {
                self.error = Some(error.into());
                return Err(std::fmt::Error);
            }
        }
        self.value.push_str(text);
        Ok(())
    }
}

fn invalid(reason: &str) -> EngineError {
    EngineError::ConfigInvalid {
        key: "session.plan_evidence".to_owned(),
        reason: reason.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn completed_metrics_observe_actual_execution_and_release_plan() {
        use datafusion::arrow::{
            array::{Int64Array, RecordBatch},
            datatypes::{DataType, Field, Schema},
        };
        use datafusion::physical_plan::{ExecutionPlan, projection::ProjectionExec};
        let schema = Arc::new(Schema::new(vec![Field::new("n", DataType::Int64, false)]));
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(Int64Array::from(vec![1, 2, 3]))],
        )
        .unwrap();
        let source = datafusion::datasource::memory::MemorySourceConfig::try_new_exec(
            &[vec![batch]],
            schema,
            None,
        )
        .unwrap();
        let expression: Arc<dyn datafusion::physical_expr::PhysicalExpr> =
            Arc::new(datafusion::physical_expr::expressions::Column::new("n", 0));
        let plan: Arc<dyn ExecutionPlan> =
            Arc::new(ProjectionExec::try_new(vec![(expression, "n".to_owned())], source).unwrap());
        let weak = Arc::downgrade(&plan);
        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(0));
        let logical = datafusion::logical_expr::LogicalPlanBuilder::empty(false)
            .build()
            .unwrap();
        let observation = Recorder::new(
            &pool,
            super::super::assurance::ObservationPolicy::Diagnostic,
        )
        .finish(&logical, &pse_columnar::CancellationToken::new())
        .unwrap()
        .with_physical(plan.as_ref(), &pool)
        .unwrap();
        let mut guard = CompletionGuard::new(&plan, &observation, &pool);
        assert!(observation.execution().is_none());
        let context = datafusion::prelude::SessionContext::new();
        let batches = datafusion::physical_plan::collect(plan.clone(), context.task_ctx())
            .await
            .unwrap();
        assert_eq!(batches.iter().map(|b| b.num_rows()).sum::<usize>(), 3);
        guard.finish(super::super::assurance::TerminalStatus::Completed);
        drop(plan);
        assert!(
            weak.upgrade().is_none(),
            "snapshot retains no native operator"
        );
        let completed = observation.execution().unwrap();
        assert!(
            completed.metrics.iter().any(|m| m.name == "output_rows"
                && m.value == Some(3)
                && m.unit == MetricUnit::Rows)
        );
        assert_eq!(
            pool.reserved(),
            0,
            "optional evidence cannot consume functional admission"
        );
    }

    #[test]
    fn abandoned_capture_is_terminal_and_idempotent() {
        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(0));
        let logical = datafusion::logical_expr::LogicalPlanBuilder::empty(false)
            .build()
            .unwrap();
        let plan: Arc<dyn datafusion::physical_plan::ExecutionPlan> =
            Arc::new(datafusion::physical_plan::empty::EmptyExec::new(Arc::new(
                datafusion::arrow::datatypes::Schema::empty(),
            )));
        let observation = Recorder::new(
            &pool,
            super::super::assurance::ObservationPolicy::Diagnostic,
        )
        .finish(&logical, &pse_columnar::CancellationToken::new())
        .unwrap()
        .with_physical(plan.as_ref(), &pool)
        .unwrap();
        drop(CompletionGuard::new(&plan, &observation, &pool));
        observation.complete(
            plan.as_ref(),
            super::super::assurance::TerminalStatus::Completed,
            &pool,
        );
        assert_eq!(
            observation.execution().unwrap().status,
            super::super::assurance::TerminalStatus::Abandoned
        );
    }

    #[test]
    fn native_json_reports_each_extension_once_without_recursive_debug_payloads() {
        use datafusion::execution::session_state::{CacheFactory, SessionStateBuilder};
        let state = SessionStateBuilder::new().with_default_features().build();
        let mut plan = datafusion::logical_expr::LogicalPlanBuilder::empty(true)
            .build()
            .unwrap();
        for _ in 0..8 {
            plan = crate::session::cache::NativeCacheFactory
                .create(plan, &state)
                .unwrap();
        }
        let budget: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 10));
        let observation = Recorder::new(
            &budget,
            crate::session::assurance::ObservationPolicy::Diagnostic,
        )
        .finish(&plan, &pse_columnar::CancellationToken::new())
        .unwrap();
        let json: serde_json::Value = serde_json::from_str(observation.explain_pgjson()).unwrap();
        assert_eq!(json["Nodes"].as_array().unwrap().len(), 17);
        assert_eq!(json["Edges"].as_array().unwrap().len(), 16);
        for node in &json["Nodes"].as_array().unwrap()[..16] {
            let native = &node["Native"][0]["Plan"];
            assert!(native["Detail"].as_str().unwrap().len() < 160);
            assert!(native["Plans"].as_array().unwrap().is_empty());
        }
        assert_eq!(
            json["Nodes"][16]["Native"][0]["Plan"]["Node Type"],
            "EmptyRelation"
        );
        drop(observation);
        assert_eq!(budget.reserved(), 0);
    }

    #[test]
    fn shared_producer_json_has_complete_definitions_and_explicit_references() {
        use datafusion::{
            execution::session_state::{CacheFactory, SessionStateBuilder},
            logical_expr::{LogicalPlanBuilder, Union},
        };
        let state = SessionStateBuilder::new().with_default_features().build();
        let mut plan = LogicalPlanBuilder::empty(true).build().unwrap();
        for _ in 0..12 {
            let cached = Arc::new(
                super::super::cache::NativeCacheFactory
                    .create(plan, &state)
                    .unwrap(),
            );
            plan = LogicalPlan::Union(Union {
                inputs: vec![Arc::clone(&cached), Arc::clone(&cached)],
                schema: Arc::clone(cached.schema()),
            });
        }
        let budget: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(128 << 10));
        let observation = Recorder::new(
            &budget,
            crate::session::assurance::ObservationPolicy::Diagnostic,
        )
        .finish(&plan, &pse_columnar::CancellationToken::new())
        .unwrap();
        let json: serde_json::Value = serde_json::from_str(observation.explain_pgjson()).unwrap();
        let nodes = json["Nodes"].as_array().unwrap();
        let edges = json["Edges"].as_array().unwrap();
        assert_eq!(nodes.len(), 37);
        assert_eq!(edges.len(), 48);
        assert_eq!(
            nodes
                .iter()
                .filter(|node| node["Native"][0]["Plan"]["Node Type"] == "NativeCache")
                .count(),
            12
        );
        for edge in edges {
            assert!(edge[0].as_u64().unwrap() < nodes.len() as u64);
            assert!(edge[2].as_u64().unwrap() < nodes.len() as u64);
        }
        assert!(
            restore_graph(&json, 0)
                == serde_json::from_str::<serde_json::Value>(&plan.display_pg_json().to_string())
                    .unwrap()[0]["Plan"],
            "flat evidence must reconstruct the native tree including its output fields"
        );
        assert!(observation.explain_pgjson().len() < 32 << 10);
        drop(observation);
        assert_eq!(budget.reserved(), 0);
    }

    fn restore_graph(graph: &serde_json::Value, id: usize) -> serde_json::Value {
        let mut node = graph["Nodes"][id]["Native"][0]["Plan"].clone();
        let mut edges: Vec<_> = graph["Edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|edge| usize::try_from(edge[0].as_u64().unwrap()).unwrap() == id)
            .collect();
        edges.sort_by_key(|edge| edge[1].as_u64().unwrap());
        node["Plans"] = edges
            .iter()
            .map(|edge| restore_graph(graph, usize::try_from(edge[2].as_u64().unwrap()).unwrap()))
            .collect::<Vec<_>>()
            .into();
        node
    }

    #[test]
    fn deep_observation_is_linear_and_releases_every_live_plan_owner() {
        use datafusion::{
            execution::session_state::{CacheFactory, SessionStateBuilder},
            logical_expr::{LogicalPlanBuilder, Projection},
        };
        let state = SessionStateBuilder::new().with_default_features().build();
        let mut plan = super::super::cache::NativeCacheFactory
            .create(LogicalPlanBuilder::empty(true).build().unwrap(), &state)
            .unwrap();
        let LogicalPlan::Extension(extension) = &plan else {
            panic!("expected native extension")
        };
        let owner = Arc::downgrade(&extension.node);
        for _ in 0..512 {
            plan = LogicalPlan::Projection(Projection::try_new(vec![], Arc::new(plan)).unwrap());
        }
        let budget: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(512 << 10));
        let observation = Recorder::new(
            &budget,
            crate::session::assurance::ObservationPolicy::Diagnostic,
        )
        .finish(&plan, &pse_columnar::CancellationToken::new())
        .unwrap();
        let graph: serde_json::Value = serde_json::from_str(observation.explain_pgjson()).unwrap();
        assert_eq!(graph["Nodes"].as_array().unwrap().len(), 515);
        assert_eq!(graph["Edges"].as_array().unwrap().len(), 514);
        assert!(observation.explain_pgjson().len() < 128 << 10);
        drop(plan);
        assert!(
            owner.upgrade().is_none(),
            "diagnostics must not pin live native owners"
        );
        drop(observation);
        assert_eq!(budget.reserved(), 0);
    }

    #[test]
    fn native_subquery_edges_and_resource_refusals_remain_observable() {
        use datafusion::logical_expr::{LogicalPlanBuilder, expr_fn::scalar_subquery, lit};
        let subquery = Arc::new(
            LogicalPlanBuilder::empty(true)
                .project(vec![lit(7_i64)])
                .unwrap()
                .build()
                .unwrap(),
        );
        let plan = LogicalPlanBuilder::empty(true)
            .project(vec![
                scalar_subquery(Arc::clone(&subquery)).alias("a"),
                scalar_subquery(subquery).alias("b"),
            ])
            .unwrap()
            .build()
            .unwrap();
        let cancel = pse_columnar::CancellationToken::new();
        let budget: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 10));
        let observation = Recorder::new(
            &budget,
            crate::session::assurance::ObservationPolicy::Diagnostic,
        )
        .finish(&plan, &cancel)
        .unwrap();
        let graph: serde_json::Value = serde_json::from_str(observation.explain_pgjson()).unwrap();
        let expected: serde_json::Value =
            serde_json::from_str(&plan.display_pg_json().to_string()).unwrap();
        assert_eq!(restore_graph(&graph, 0), expected[0]["Plan"]);
        drop(observation);
        assert_eq!(budget.reserved(), 0);
        cancel.cancel();
        let cancelled = Recorder::new(
            &budget,
            crate::session::assurance::ObservationPolicy::Diagnostic,
        )
        .finish(&plan, &cancel)
        .unwrap();
        assert!(cancelled.diagnostic().is_some());
        assert!(cancelled.explain_pgjson().is_empty());
        assert_eq!(budget.reserved(), 0);
        let tiny: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(0));
        let captured = Recorder::new(
            &tiny,
            crate::session::assurance::ObservationPolicy::Diagnostic,
        )
        .finish(&plan, &pse_columnar::CancellationToken::new())
        .unwrap();
        assert!(captured.diagnostic().is_none());
        assert!(!captured.explain_pgjson().is_empty());
        assert_eq!(tiny.reserved(), 0);
    }

    #[test]
    fn diagnostic_fragments_charge_capacity_and_use_the_exact_budget_tail() {
        let budget: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(100_003));
        let mut reservation =
            pse_columnar::MemoryConsumer::new("test:observation").register(&budget);
        {
            let mut writer = BoundedText {
                value: String::new(),
                reservation: &mut reservation,
                error: None,
            };
            for _ in 0..10_000 {
                writer.write_str("1234567890").unwrap();
            }
            assert_eq!(writer.value.len(), 100_000);
            assert_eq!(budget.reserved(), writer.value.capacity());
            writer.write_str("end").unwrap();
            assert_eq!(writer.value.len(), 100_003);
            assert!(writer.value.ends_with("end"));
            assert!(writer.write_str("!").is_err());
            assert_eq!(writer.value.len(), 100_003);
        }
        drop(reservation);
        assert_eq!(budget.reserved(), 0);
    }
}
