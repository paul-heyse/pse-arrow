// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual analyzer/optimizer observations and the plan that reached physical planning.
use crate::EngineError;
use datafusion::logical_expr::LogicalPlan;

use pse_columnar::{AllocationLease, MemoryPool};
use std::{
    fmt::Write,
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, Instant},
};

mod graph;
mod comparison;
pub use comparison::{PhysicalGraphComparison, PhysicalNodeMatch};

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
    operation: Option<pse_diagnostics::observation::OperationId>,
    logical: Arc<ObservedPlan>,
    physical: Option<Arc<ObservedPhysical>>,
}
#[derive(Debug)]
struct ObservedPlan {
    captured: bool,
    rich: bool,
    explain: String,
    rules: Vec<String>,
    diagnostic: Option<String>,
    _lease: Arc<AllocationLease>,
}
#[derive(Debug)]
struct ObservedPhysical {
    explain: String,
    graph: Option<ObservedPhysicalGraph>,
    execution: u64,
    completion: OnceLock<ExecutionSnapshot>,
    latest: Mutex<Option<Arc<LiveExecutionSnapshot>>>,
    started: Instant,
    diagnostic: Option<String>,
    _lease: Arc<AllocationLease>,
}
/// A bounded actual physical topology. Ordinals share the native metric traversal.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ObservedPhysicalGraph {
    /// First observed node, absent if capture admission failed.
    pub root: Option<usize>,
    /// Unique actual operators, including independently planned cache producers.
    pub nodes: Vec<ObservedPhysicalNode>,
    /// Ordered input/dependency edges; shared inputs retain all incoming edges.
    pub edges: Vec<ObservedPhysicalEdge>,
    /// Bounds or optional allocation admission curtailed capture.
    pub truncated: bool,
}
/// Original physical operator facts; local ordinal is not cross-run identity.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ObservedPhysicalNode {
    /// Snapshot-local metric ordinal.
    pub node: usize,
    /// Native operator family.
    pub operator: String,
    /// Original output schema, as ordered field name/type pairs.
    pub output_fields: Vec<(String, String)>,
    /// Actual output partition count.
    pub partitions: usize,
}
/// An ordered actual physical dependency, distinct from numerical/canonical edges.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ObservedPhysicalEdge {
    /// Parent metric ordinal.
    pub parent: usize,
    /// Original child position.
    pub ordinal: usize,
    /// Child metric ordinal.
    pub child: usize,
    /// The edge crosses to an independently planned cache producer.
    pub cache_producer: bool,
}

// The same bounded preorder supplies capture and metric ordinals. No display text is parsed.
fn physical_walk<'a>(root: &'a dyn datafusion::physical_plan::ExecutionPlan, reservation: &pse_columnar::MemoryReservation)
    -> (Vec<&'a dyn datafusion::physical_plan::ExecutionPlan>, Vec<ObservedPhysicalEdge>, bool)
{
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut visited = std::collections::HashMap::new();
    let mut pending = vec![(root, None)];
    let mut truncated = false;
    while let Some((plan, edge)) = pending.pop() {
        let identity = std::ptr::from_ref(plan).cast::<()>() as usize;
        let node = if let Some(&node) = visited.get(&identity) { node } else {
            if nodes.len() >= 4096 || reservation.try_grow(256).is_err() { truncated = true; break; }
            let node = nodes.len();
            visited.insert(identity, node);
            nodes.push(plan);
            let producer = super::cache::reset_dependency(plan);
            let cache_producer = producer.is_some();
            let children = producer.map_or_else(|| plan.children(), |input| vec![input]);
            if pending.len().saturating_add(children.len()) > 4096 || reservation.try_grow(children.len().saturating_mul(64)).is_err() {
                truncated = true;
            } else {
                pending.extend(children.into_iter().enumerate().rev().map(|(ordinal, child)| (child.as_ref(), Some((node, ordinal, cache_producer)))));
            }
            node
        };
        if let Some((parent, ordinal, cache_producer)) = edge {
            if edges.len() >= 16384 || reservation.try_grow(2 * size_of::<ObservedPhysicalEdge>()).is_err() { truncated = true; break; }
            edges.push(ObservedPhysicalEdge { parent, ordinal, child: node, cache_producer });
        }
    }
    (nodes, edges, truncated)
}
fn physical_graph(root: &dyn datafusion::physical_plan::ExecutionPlan, reservation: &pse_columnar::MemoryReservation) -> ObservedPhysicalGraph {
    let (plans, edges, mut truncated) = physical_walk(root, reservation);
    let mut nodes = Vec::new();
    for plan in plans {
        let schema = plan.schema();
        let extent = schema.fields().iter().map(|field| field.name().len().saturating_add(256)).sum::<usize>().saturating_add(plan.name().len()).saturating_add(256);
        if extent > 65536 || reservation.try_grow(extent).is_err() { truncated = true; break; }
        nodes.push(ObservedPhysicalNode {
            node: nodes.len(), operator: plan.name().into(),
            output_fields: schema.fields().iter().map(|field| (field.name().clone(), field.data_type().to_string())).collect(),
            partitions: plan.properties().output_partitioning().partition_count(),
        });
    }
    let edges = edges.into_iter().filter(|edge| edge.parent < nodes.len() && edge.child < nodes.len()).collect();
    ObservedPhysicalGraph { root: (!nodes.is_empty()).then_some(0), nodes, edges, truncated }
}

/// Native metric units remain explicit; an absent value is never fabricated as zero.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
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
/// Native value semantics, independent of its unit.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum MetricKind {
    /// Cumulative native work since this execution's metric allocation.
    Counter,
    /// Current or peak native state; never summed as work.
    Gauge,
    /// Timestamp, compound or custom value not projected numerically.
    Native,
}
/// One metric from one node and partition of the executed graph.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
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
    /// Counter versus gauge semantics from the native metric variant.
    pub kind: MetricKind,
    /// Exact native numeric counter, absent for compound/custom/timestamp values.
    pub value: Option<usize>,
}
/// Owner-copied live evidence; polling and reading never execute the graph.
#[derive(Debug)]
pub struct LiveExecutionSnapshot {
    /// Actual invocation identity, shared with its terminal snapshot.
    pub execution: u64,
    /// Unique node and partition metrics; values may update independently.
    pub metrics: Vec<ObservedMetric>,
    /// Collection hit an explicit diagnostic bound.
    pub truncated: bool,
    /// Time of this owner checkpoint relative to physical capture creation.
    pub observed_after_start: Duration,
    /// Wall time spent copying these metrics, separate from operator work.
    pub collection_elapsed: Duration,
    _lease: Arc<AllocationLease>,
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
    /// Final copy time relative to physical capture creation.
    pub observed_after_start: Duration,
    /// Time spent collecting the terminal copy, separate from operator work.
    pub collection_elapsed: Duration,
    _lease: Arc<AllocationLease>,
}
/// Stream-local ownership makes partial and abandoned executions observable too.
pub(super) struct CompletionGuard {
    plan: Option<Arc<dyn datafusion::physical_plan::ExecutionPlan>>,
    observation: PlanObservation,
    pool: Arc<dyn MemoryPool>,
    last_checkpoint: Option<Instant>,
    context: Option<pse_diagnostics::observation::ObservationContext>,
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
            last_checkpoint: None,
            context: pse_diagnostics::observation::current(),
        }
    }
    pub(super) fn checkpoint(&mut self) {
        if self
            .last_checkpoint
            .is_some_and(|last| last.elapsed() < Duration::from_millis(250))
        {
            return;
        }
        let Some(plan) = &self.plan else {
            return;
        };
        self.last_checkpoint = Some(Instant::now());
        self.observation
            .checkpoint(plan.as_ref(), &self.pool, self.context.as_ref());
    }
    pub(super) fn finish(&mut self, status: super::assurance::TerminalStatus) {
        if let Some(plan) = self.plan.take() {
            self.observation
                .checkpoint(plan.as_ref(), &self.pool, self.context.as_ref());
            self.observation.complete(plan.as_ref(), status, &self.pool);
            if let Some(context) = &self.context {
                context.text("query.metrics.coverage", "terminal owner copy after stream release; native counters may be partial on failure or abandonment");
                context.text("query.metrics.terminal", &format!("{status:?}"));
            }
        }
    }
}
impl Drop for CompletionGuard {
    fn drop(&mut self) {
        self.finish(super::assurance::TerminalStatus::Abandoned);
    }
}
impl PlanObservation {
    /// Latest owner-copied live metrics. Contended publication returns no copy;
    /// this getter never acquires a native plan or polls an execution stream.
    pub fn live_execution(&self) -> Option<Arc<LiveExecutionSnapshot>> {
        self.physical.as_ref()?.latest.try_lock().ok()?.clone()
    }
    fn checkpoint(
        &self,
        plan: &dyn datafusion::physical_plan::ExecutionPlan,
        pool: &Arc<dyn MemoryPool>,
        context: Option<&pse_diagnostics::observation::ObservationContext>,
    ) {
        let Some(physical) = &self.physical else {
            return;
        };
        if physical.completion.get().is_some() {
            return;
        }
        let live = Arc::new(snapshot(plan, physical.execution, physical.started, pool));
        if let Some(context) = context {
            context.text(
                "query.metrics.coverage",
                "owner checkpoint; independently updated native metrics; drop export pending",
            );
            context.boolean("query.metrics.truncated", live.truncated);
            context.source_ref("query.execution", physical.execution.to_le_bytes());
            context.count("query.metrics.checkpoints", "checkpoints", 1);
            context.duration("query.metrics.collection", live.collection_elapsed);
            for metric in &live.metrics {
                let key = format!(
                    "query.{}.node.{}.{}.{:?}.{:?}",
                    live.execution, metric.node, metric.name, metric.partition, metric.labels
                );
                context.source_ref(
                    &key,
                    format!(
                        "operator={};name={};partition={:?};labels={:?};kind={:?};unit={:?}",
                        metric.operator,
                        metric.name,
                        metric.partition,
                        metric.labels,
                        metric.kind,
                        metric.unit
                    ),
                );
                let unit = match metric.unit {
                    MetricUnit::Rows => "rows",
                    MetricUnit::Batches => "batches",
                    MetricUnit::Bytes => "bytes",
                    MetricUnit::Nanoseconds => "nanoseconds",
                    MetricUnit::Count => "count",
                    MetricUnit::Native => "native",
                };
                match (metric.kind, metric.value) {
                    (MetricKind::Counter, Some(value)) => {
                        context.count_value(&key, unit, value as u64, live.execution)
                    }
                    (MetricKind::Gauge, Some(value)) => match i64::try_from(value) {
                        Ok(value) => context.integer(&key, unit, value),
                        Err(_) => context
                            .unavailable(&key, "native gauge exceeds exact signed projection"),
                    },
                    _ => context.unavailable(
                        &key,
                        "compound, custom or timestamp metric not projected numerically",
                    ),
                }
            }
        }
        if let Ok(mut latest) = physical.latest.try_lock() {
            *latest = Some(live);
        } else if let Some(context) = context {
            context.count("query.metrics.dropped", "snapshots", 1);
        }
    }
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
            physical.completion.get_or_init(|| {
                let copy = snapshot(plan, physical.execution, physical.started, pool);
                ExecutionSnapshot {
                    execution: copy.execution,
                    status,
                    metrics: copy.metrics,
                    truncated: copy.truncated,
                    observed_after_start: copy.observed_after_start,
                    collection_elapsed: copy.collection_elapsed,
                    _lease: copy._lease,
                }
            });
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
        let rendered = if self.logical.rich {
            render_physical(plan, &mut writer, pool)
        } else {
            Ok(())
        };
        let diagnostic = writer
            .error
            .or_else(|| rendered.err())
            .map(|e| e.to_string());
        let explain = if diagnostic.is_some() {
            String::new()
        } else {
            writer.value
        };
        let graph = self.logical.rich.then(|| physical_graph(plan, &reservation));
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        Ok(Self {
            operation: pse_diagnostics::observation::current().map(|context| context.id()),
            logical: Arc::clone(&self.logical),
            physical: Some(Arc::new(ObservedPhysical {
                explain,
                graph,
                execution: NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                completion: OnceLock::new(),
                latest: Mutex::new(None),
                started: Instant::now(),
                diagnostic,
                _lease: AllocationLease::new(reservation),
            })),
        })
    }
}

fn snapshot(
    root: &dyn datafusion::physical_plan::ExecutionPlan,
    execution: u64,
    started: Instant,
    pool: &Arc<dyn MemoryPool>,
) -> LiveExecutionSnapshot {
    let collection_start = Instant::now();
    use datafusion::physical_plan::metrics::MetricValue as V;
    let _ = pool;
    let capture_pool = diagnostic_pool();
    let pool = &capture_pool;
    let reservation = pse_columnar::MemoryConsumer::new("session:completed-metrics").register(pool);
    let mut metrics = Vec::new();
    let (nodes, _, mut truncated) = physical_walk(root, &reservation);
    'nodes: for (node, plan) in nodes.into_iter().enumerate() {
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
                    kind: match value {
                        V::CurrentMemoryUsage(_) | V::PeakMemoryUsage { .. } | V::Gauge { .. } => {
                            MetricKind::Gauge
                        }
                        _ if numeric => MetricKind::Counter,
                        _ => MetricKind::Native,
                    },
                    value: numeric.then(|| value.as_usize()),
                });
            }
        }
    }
    LiveExecutionSnapshot {
        execution,
        metrics,
        truncated,
        observed_after_start: started.elapsed(),
        collection_elapsed: collection_start.elapsed(),
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
    captured: bool,
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
            captured: policy != crate::session::assurance::ObservationPolicy::Off,
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
            operation: pse_diagnostics::observation::current().map(|context| context.id()),
            logical: Arc::new(ObservedPlan {
                captured: self.captured,
                rich: self.enabled,
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
        let observation =
            Recorder::new(&pool, super::super::assurance::ObservationPolicy::Contract)
                .finish(&logical, &pse_columnar::CancellationToken::new())
                .unwrap()
                .with_physical(plan.as_ref(), &pool)
                .unwrap();
        let mut guard = CompletionGuard::new(&plan, &observation, &pool);
        assert!(observation.execution().is_none());
        let context = datafusion::prelude::SessionContext::new();
        use futures_util::StreamExt;
        assert!(observation.explain_pgjson().is_empty());
        assert_eq!(observation.physical_plan(), Some(""));
        let mut stream =
            datafusion::physical_plan::execute_stream(plan.clone(), context.task_ctx()).unwrap();
        assert_eq!(stream.next().await.unwrap().unwrap().num_rows(), 3);
        guard.checkpoint();
        let live = observation.live_execution().unwrap();
        assert!(
            observation.execution().is_none(),
            "live copy is not terminal evidence"
        );
        assert!(live.metrics.iter().any(|m| m.name == "output_rows"
            && m.value == Some(3)
            && m.kind == MetricKind::Counter));
        let held = observation
            .physical
            .as_ref()
            .unwrap()
            .latest
            .lock()
            .unwrap();
        assert!(
            observation.live_execution().is_none(),
            "reader never blocks on publication"
        );
        drop(held);
        assert!(stream.next().await.is_none());
        drop(stream);
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
    fn metrics_capture_is_selected_and_graph_curtailment_is_explicit() {
        use datafusion::physical_plan::{ExecutionPlan, empty::EmptyExec, union::UnionExec};
        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(0));
        let logical = datafusion::logical_expr::LogicalPlanBuilder::empty(false)
            .build()
            .unwrap();
        let schema = Arc::new(datafusion::arrow::datatypes::Schema::empty());
        let children = (0..4097)
            .map(|_| {
                let child: Arc<dyn ExecutionPlan> = Arc::new(EmptyExec::new(schema.clone()));
                child
            })
            .collect();
        let plan: Arc<dyn ExecutionPlan> = UnionExec::try_new(children).unwrap();
        let off = Recorder::new(&pool, super::super::assurance::ObservationPolicy::Off)
            .finish(&logical, &pse_columnar::CancellationToken::new())
            .unwrap()
            .with_physical(plan.as_ref(), &pool)
            .unwrap();
        let mut disabled = CompletionGuard::new(&plan, &off, &pool);
        disabled.checkpoint();
        disabled.finish(super::super::assurance::TerminalStatus::Completed);
        assert!(off.live_execution().is_none());
        assert!(off.execution().is_none());
        let observation =
            Recorder::new(&pool, super::super::assurance::ObservationPolicy::Contract)
                .finish(&logical, &pse_columnar::CancellationToken::new())
                .unwrap()
                .with_physical(plan.as_ref(), &pool)
                .unwrap();
        let mut capture = CompletionGuard::new(&plan, &observation, &pool);
        capture.checkpoint();
        assert!(observation.live_execution().unwrap().truncated);
        capture.finish(super::super::assurance::TerminalStatus::Abandoned);
        assert!(observation.execution().unwrap().truncated);
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

/// Actual selected instrumentation policy; copying never enables instrumentation.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum CopiedObservationPolicy {
    /// No retained native instrumentation.
    Off,
    /// Actual contract instrumentation without plan text.
    Contract,
    /// Contract instrumentation with bounded diagnostic plans/rules.
    Diagnostic,
}
/// Actual stream lifetime status, independent of scientific result acceptance.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum CopiedExecutionStatus {
    /// Last owner checkpoint only; terminal status unavailable.
    Running,
    /// Successful stream exhaustion.
    Completed,
    /// Owner failure.
    Failed,
    /// Cooperative cancellation.
    Cancelled,
    /// Dropped before successful exhaustion.
    Abandoned,
}
/// Data-only copy of one actual execution population; no plan or stream owner survives.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CopiedExecutionSnapshot {
    /// Process-local execution key; scope with the observation's original operation.
    pub execution: u64,
    /// Actual completed/drop state, or a running checkpoint.
    pub status: CopiedExecutionStatus,
    /// Native node/partition/label populations with exact units and value kinds.
    pub metrics: Vec<ObservedMetric>,
    /// Explicit collection curtailment.
    pub truncated: bool,
    /// Original physical-capture clock endpoint.
    #[schemars(with = "pse_diagnostics::ClosedDuration")]
    pub observed_after_start: Duration,
    /// Observer work, never operator execution time.
    #[schemars(with = "pse_diagnostics::ClosedDuration")]
    pub collection_elapsed: Duration,
}
/// Bounded original reports plus a single latest-or-terminal metric population.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CopiedPlanObservation {
    /// Actual enclosing operation captured at assembly, absent outside an observation scope.
    pub operation: Option<pse_diagnostics::observation::OperationId>,
    /// Actual original capture policy.
    pub policy: CopiedObservationPolicy,
    /// Bounded logical graph; empty under Contract/Off or failed capture.
    pub logical_plan: String,
    /// Bounded physical graph, absent before physical planning.
    pub physical_plan: Option<String>,
    /// Bounded actual topology with metric-local ordinals, selected only in Diagnostic.
    pub physical_graph: Option<ObservedPhysicalGraph>,
    /// Actual fired analyzer/optimizer rules, empty unless Diagnostic captured them.
    pub rules: Vec<String>,
    /// Optional capture failure; never an execution failure.
    pub diagnostic: Option<String>,
    /// Terminal copy preferred over a live checkpoint; populations are never summed.
    pub execution: Option<CopiedExecutionSnapshot>,
}
impl PlanObservation {
    /// Copy retained facts only; does not poll a stream or query native metrics.
    pub fn copied(&self) -> CopiedPlanObservation {
        self.select_copy().copied()
    }
    /// Pin one already copied metric population before sizing or materializing it.
    pub(super) fn select_copy(&self) -> SelectedPlanObservation<'_> {
        SelectedPlanObservation {
            observation: self,
            execution: if let Some(value) = self.execution() {
                Some(SelectedExecution::Terminal(value))
            } else {
                self.live_execution().map(SelectedExecution::Live)
            },
        }
    }
}
pub(super) struct SelectedPlanObservation<'a> {
    observation: &'a PlanObservation,
    execution: Option<SelectedExecution<'a>>,
}
enum SelectedExecution<'a> {
    Terminal(&'a ExecutionSnapshot),
    Live(Arc<LiveExecutionSnapshot>),
}
impl SelectedPlanObservation<'_> {
    /// Owned payload storage, including nested vector elements and string bytes.
    /// Selection pins live snapshots, so none of these populations can grow before copying.
    pub(super) fn copied_extent(&self) -> Option<usize> {
        let observation = self.observation;
        let mut bytes = size_of::<CopiedPlanObservation>()
            .checked_add(observation.explain_pgjson().len())?
            .checked_add(observation.physical_plan().map_or(0, str::len))?
            .checked_add(observation.diagnostic().map_or(0, str::len))?
            .checked_add(
                observation
                    .rules_fired()
                    .len()
                    .checked_mul(size_of::<String>())?,
            )?;
        for rule in observation.rules_fired() {
            bytes = bytes.checked_add(rule.len())?;
        }
        if let Some(graph) = observation.physical.as_ref().and_then(|physical| physical.graph.as_ref()) {
            bytes = bytes.checked_add(graph.edges.len().checked_mul(size_of::<ObservedPhysicalEdge>())?)?
                .checked_add(graph.nodes.len().checked_mul(size_of::<ObservedPhysicalNode>())?)?;
            for node in &graph.nodes {
                bytes = bytes.checked_add(node.operator.len())?.checked_add(node.output_fields.len().checked_mul(size_of::<(String, String)>())?)?;
                for (name, ty) in &node.output_fields { bytes = bytes.checked_add(name.len())?.checked_add(ty.len())?; }
            }
        }
        let metrics = match &self.execution {
            Some(SelectedExecution::Terminal(value)) => value.metrics.as_slice(),
            Some(SelectedExecution::Live(value)) => value.metrics.as_slice(),
            None => &[],
        };
        bytes = bytes.checked_add(metrics.len().checked_mul(size_of::<ObservedMetric>())?)?;
        for metric in metrics {
            bytes = bytes
                .checked_add(metric.operator.len())?
                .checked_add(metric.name.len())?
                .checked_add(
                    metric
                        .labels
                        .len()
                        .checked_mul(size_of::<(String, String)>())?,
                )?;
            for (name, value) in &metric.labels {
                bytes = bytes.checked_add(name.len())?.checked_add(value.len())?;
            }
        }
        Some(bytes)
    }
    pub(super) fn copied(self) -> CopiedPlanObservation {
        let observation = self.observation;
        let execution = match self.execution {
            Some(SelectedExecution::Terminal(value)) => {
                use super::assurance::TerminalStatus as S;
                Some(CopiedExecutionSnapshot {
                    execution: value.execution,
                    status: match value.status {
                        S::Completed => CopiedExecutionStatus::Completed,
                        S::Failed => CopiedExecutionStatus::Failed,
                        S::Cancelled => CopiedExecutionStatus::Cancelled,
                        S::Abandoned => CopiedExecutionStatus::Abandoned,
                    },
                    metrics: value.metrics.clone(),
                    truncated: value.truncated,
                    observed_after_start: value.observed_after_start,
                    collection_elapsed: value.collection_elapsed,
                })
            }
            Some(SelectedExecution::Live(value)) => Some(CopiedExecutionSnapshot {
                execution: value.execution,
                status: CopiedExecutionStatus::Running,
                metrics: value.metrics.clone(),
                truncated: value.truncated,
                observed_after_start: value.observed_after_start,
                collection_elapsed: value.collection_elapsed,
            }),
            None => None,
        };
        CopiedPlanObservation {
            operation: observation.operation,
            policy: if !observation.logical.captured {
                CopiedObservationPolicy::Off
            } else if observation.logical.rich {
                CopiedObservationPolicy::Diagnostic
            } else {
                CopiedObservationPolicy::Contract
            },
            logical_plan: observation.explain_pgjson().to_owned(),
            physical_plan: observation.physical_plan().map(str::to_owned),
            physical_graph: observation.physical.as_ref().and_then(|physical| physical.graph.clone()),
            rules: observation.rules_fired().to_vec(),
            diagnostic: observation.diagnostic().map(str::to_owned),
            execution,
        }
    }
}

#[cfg(test)]
mod copied_observation_tests {
    use super::super::assurance::{ObservationPolicy, TerminalStatus};
    use super::*;
    fn observation(policy: ObservationPolicy) -> PlanObservation {
        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(0));
        let logical = datafusion::logical_expr::LogicalPlanBuilder::empty(false)
            .build()
            .unwrap();
        Recorder::new(&pool, policy)
            .finish(&logical, &pse_columnar::CancellationToken::new())
            .unwrap()
    }
    #[test]
    fn copied_observation_preserves_off_contract_diagnostic_without_enabling() {
        let off = observation(ObservationPolicy::Off).copied();
        assert_eq!(off.policy, CopiedObservationPolicy::Off);
        assert!(off.logical_plan.is_empty());
        assert!(off.execution.is_none());
        let contract = observation(ObservationPolicy::Contract).copied();
        assert_eq!(contract.policy, CopiedObservationPolicy::Contract);
        assert!(contract.logical_plan.is_empty());
        let diagnostic = observation(ObservationPolicy::Diagnostic).copied();
        assert_eq!(diagnostic.policy, CopiedObservationPolicy::Diagnostic);
        assert!(!diagnostic.logical_plan.is_empty());
        let encoded = serde_json::to_vec(&diagnostic).unwrap();
        let decoded: CopiedPlanObservation = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded.logical_plan, diagnostic.logical_plan);
    }
    #[test]
    fn selected_copy_keeps_sized_live_population_after_terminal_publication() {
        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(0));
        let plan: Arc<dyn datafusion::physical_plan::ExecutionPlan> =
            Arc::new(datafusion::physical_plan::empty::EmptyExec::new(Arc::new(
                datafusion::arrow::datatypes::Schema::empty(),
            )));
        let observation = observation(ObservationPolicy::Contract)
            .with_physical(plan.as_ref(), &pool)
            .unwrap();
        let mut guard = CompletionGuard::new(&plan, &observation, &pool);
        guard.checkpoint();
        let selected = observation.select_copy();
        let sized = selected.copied_extent().unwrap();
        guard.finish(TerminalStatus::Completed);
        assert_eq!(selected.copied_extent(), Some(sized));
        let copied = selected.copied();
        assert_eq!(
            copied.execution.unwrap().status,
            CopiedExecutionStatus::Running
        );
        assert_eq!(
            observation.copied().execution.unwrap().status,
            CopiedExecutionStatus::Completed
        );
    }
    #[test]
    fn copied_observation_abandonment_and_operation_are_original_facts() {
        let hub = pse_diagnostics::observation::ObservationHub::new(
            pse_diagnostics::observation::Policy::default(),
        );
        let owner = hub.start("original engine owner");
        let context = owner.context();
        let entered = context.enter();
        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(0));
        let plan: Arc<dyn datafusion::physical_plan::ExecutionPlan> =
            Arc::new(datafusion::physical_plan::empty::EmptyExec::new(Arc::new(
                datafusion::arrow::datatypes::Schema::empty(),
            )));
        let captured = observation(ObservationPolicy::Contract)
            .with_physical(plan.as_ref(), &pool)
            .unwrap();
        drop(CompletionGuard::new(&plan, &captured, &pool));
        drop(entered);
        captured.complete(plan.as_ref(), TerminalStatus::Completed, &pool);
        let copied = captured.copied();
        assert_eq!(copied.operation, Some(owner.context().id()));
        assert_eq!(
            copied.execution.unwrap().status,
            CopiedExecutionStatus::Abandoned
        );
    }
}

#[cfg(test)]
mod physical_topology_tests {
    use super::*;
    use datafusion::physical_plan::{ExecutionPlan, empty::EmptyExec, union::UnionExec};
    #[test]
    fn topology_captures_ordered_shared_edges_with_metric_preorder_and_no_live_owner() {
        let schema = Arc::new(datafusion::arrow::datatypes::Schema::empty());
        let shared: Arc<dyn ExecutionPlan> = Arc::new(EmptyExec::new(schema));
        let weak = Arc::downgrade(&shared);
        let root: Arc<dyn ExecutionPlan> = UnionExec::try_new(vec![shared.clone(), shared.clone()]).unwrap();
        let reservation = pse_columnar::MemoryConsumer::new("test topology").register(&diagnostic_pool());
        let graph = physical_graph(root.as_ref(), &reservation);
        assert_eq!(graph.nodes.iter().map(|node| (node.node,node.operator.as_str())).collect::<Vec<_>>(), [(0,"UnionExec"),(1,"EmptyExec")]);
        assert_eq!(graph.edges.iter().map(|edge| (edge.parent,edge.ordinal,edge.child)).collect::<Vec<_>>(), [(0,0,1),(0,1,1)]);
        assert!(!graph.truncated);
        let (walk,_,_) = physical_walk(root.as_ref(), &reservation);
        assert_eq!(walk.iter().map(|node| node.name()).collect::<Vec<_>>(), ["UnionExec","EmptyExec"]);
        drop(walk); drop(root); drop(shared);
        assert!(weak.upgrade().is_none());
        assert_eq!(graph.nodes.len(), 2);
    }
}
