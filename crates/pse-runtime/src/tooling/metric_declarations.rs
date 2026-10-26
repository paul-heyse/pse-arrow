// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native instrument meanings shared by actual producers and generated passive consumers.

/// One native stream declaration. No exact operation/model/invocation IDs are dimensions.
#[derive(Clone, Copy, Debug, serde::Serialize, schemars::JsonSchema)]
pub struct MetricDeclaration {
    /// Stable emitted instrument name.
    pub name: &'static str,
    /// Actual SDK instrument role.
    pub kind: &'static str,
    /// Whether the instrument accumulates only nonnegative measurements.
    pub monotonic: bool,
    /// Whether the pinned producer SDK can emit exemplar pointers; never completeness.
    pub exemplars: bool,
    /// UCUM or explicit count unit.
    pub unit: &'static str,
    /// Performing-owner population, also the SDK instrument description.
    pub population: &'static str,
    /// Only these producer-bounded attribute keys are emitted.
    pub attributes: &'static [&'static str],
    /// Finite values from the same vocabularies used by actual producers.
    #[schemars(with = "std::collections::BTreeMap<String, Vec<String>>")]
    pub attribute_values: AttributeValues,
    /// Explicit finite histogram boundaries, in the declared unit.
    pub buckets: &'static [f64],
    /// Cumulative counter/histogram or instantaneous gauge.
    pub temporality: &'static str,
    /// Source generation/reset contract, independent of SDK activation.
    pub reset: &'static str,
    /// Coverage and source exclusions.
    pub scope: &'static str,
    /// Missing/saturated source handling and attribution limits.
    pub coverage: &'static str,
}

pub(super) const PSE_RUNTIME_EXECUTOR_WORKERS: MetricDeclaration = MetricDeclaration {
    name: "pse_runtime_executor_workers",
    kind: "observable_gauge",
    monotonic: false,
    exemplars: false,
    unit: "{thread}",
    population: "Configured workers of currently registered executors",
    attributes: &["executor.role"],
    attribute_values: AttributeValues::Executor,
    buckets: &[],
    temporality: "instantaneous",
    reset: "current registered executor population; registrations may disappear; busy totals originate at each runtime construction",
    scope: "registered Tokio executors only, aggregated by fixed composition role; not Rayon or process CPU",
    coverage: "registry-unavailable source omits executor values; approximate independent reads, not a scheduler census or settlement proof",
};

pub(super) const PSE_RUNTIME_EXECUTOR_ALIVE_TASKS: MetricDeclaration = MetricDeclaration {
    name: "pse_runtime_executor_alive_tasks",
    kind: "observable_gauge",
    monotonic: false,
    exemplars: false,
    unit: "{task}",
    population: "Approximate alive tasks of currently registered executors; not settlement",
    attributes: &["executor.role"],
    attribute_values: AttributeValues::Executor,
    buckets: &[],
    temporality: "instantaneous",
    reset: "current registered executor population; registrations may disappear; busy totals originate at each runtime construction",
    scope: "registered Tokio executors only, aggregated by fixed composition role; not Rayon or process CPU",
    coverage: "registry-unavailable source omits executor values; approximate independent reads, not a scheduler census or settlement proof",
};

pub(super) const PSE_RUNTIME_EXECUTOR_GLOBAL_QUEUE_DEPTH: MetricDeclaration = MetricDeclaration {
    name: "pse_runtime_executor_global_queue_depth",
    kind: "observable_gauge",
    monotonic: false,
    exemplars: false,
    unit: "{task}",
    population: "Global injection queues only; excludes worker-local queues",
    attributes: &["executor.role"],
    attribute_values: AttributeValues::Executor,
    buckets: &[],
    temporality: "instantaneous",
    reset: "current registered executor population; registrations may disappear; busy totals originate at each runtime construction",
    scope: "registered Tokio executors only, aggregated by fixed composition role; not Rayon or process CPU",
    coverage: "registry-unavailable source omits executor values; approximate independent reads, not a scheduler census or settlement proof",
};

pub(super) const PSE_RUNTIME_EXECUTOR_WORKER_BUSY: MetricDeclaration = MetricDeclaration {
    name: "pse_runtime_executor_worker_busy",
    kind: "observable_gauge",
    monotonic: false,
    exemplars: false,
    unit: "s",
    population: "Sum of cumulative worker busy time for currently registered executors; not CPU time",
    attributes: &["executor.role"],
    attribute_values: AttributeValues::Executor,
    buckets: &[],
    temporality: "instantaneous",
    reset: "current registered executor population; registrations may disappear; busy totals originate at each runtime construction",
    scope: "registered Tokio executors only, aggregated by fixed composition role; not Rayon or process CPU",
    coverage: "registry-unavailable source omits executor values; approximate independent reads, not a scheduler census or settlement proof",
};

pub(super) const PSE_NATIVE_SUPERVISOR_EVENTS: MetricDeclaration = MetricDeclaration {
    name: "pse_native_supervisor_events",
    kind: "observable_counter",
    monotonic: true,
    exemplars: false,
    unit: "{event}",
    population: "Process cumulative actual native supervisor route events; join witnesses native/TLS destruction, attempts are not scientific solves",
    attributes: &["event"],
    attribute_values: AttributeValues::NativeEvents,
    buckets: &[],
    temporality: "cumulative",
    reset: "process incarnation; source_start supplies first performing-owner activity, SDK activation is distinct",
    scope: "actual native supervisor/preparation owners; nested service/operation summaries overlap and are not re-added",
    coverage: "absent source omitted for totals; saturated operational totals omitted; independently copied state is not an atomic census",
};

pub(super) const PSE_NATIVE_SUPERVISORS: MetricDeclaration = MetricDeclaration {
    name: "pse_native_supervisors",
    kind: "observable_gauge",
    monotonic: false,
    exemplars: false,
    unit: "{owner}",
    population: "Outstanding actual supervisor owners; caller completion does not prove physical release",
    attributes: &[],
    attribute_values: AttributeValues::None,
    buckets: &[],
    temporality: "instantaneous",
    reset: "process incarnation; source_start supplies first performing-owner activity, SDK activation is distinct",
    scope: "actual native supervisor/preparation owners; nested service/operation summaries overlap and are not re-added",
    coverage: "absent source omitted for totals; saturated operational totals omitted; independently copied state is not an atomic census",
};

pub(super) const PSE_PREPARATION_FLIGHT_EVENTS: MetricDeclaration = MetricDeclaration {
    name: "pse_preparation_flight_events",
    kind: "observable_counter",
    monotonic: true,
    exemplars: false,
    unit: "{event}",
    population: "Process cumulative frontier/basis acquisition callers, retained reads and single-flight producers; overlapping populations, not additive work",
    attributes: &["stage", "event"],
    attribute_values: AttributeValues::PreparationEvents,
    buckets: &[],
    temporality: "cumulative",
    reset: "process incarnation; source_start supplies first performing-owner activity, SDK activation is distinct",
    scope: "actual native supervisor/preparation owners; nested service/operation summaries overlap and are not re-added",
    coverage: "absent source omitted for totals; saturated operational totals omitted; independently copied state is not an atomic census",
};

pub(super) const PSE_PREPARATION_PRODUCTS: MetricDeclaration = MetricDeclaration {
    name: "pse_preparation_products",
    kind: "observable_counter",
    monotonic: true,
    exemplars: false,
    unit: "{product}",
    population: "Process cumulative performing-owner preparations; nested operation/service summaries overlap and are not re-added",
    attributes: &["product"],
    attribute_values: AttributeValues::Products,
    buckets: &[],
    temporality: "cumulative",
    reset: "process incarnation; source_start supplies first performing-owner activity, SDK activation is distinct",
    scope: "actual native supervisor/preparation owners; nested service/operation summaries overlap and are not re-added",
    coverage: "absent source omitted for totals; saturated operational totals omitted; independently copied state is not an atomic census",
};

pub(super) const PSE_CANONICAL_QUERY_EVENTS: MetricDeclaration = MetricDeclaration {
    name: "pse_canonical_query_events",
    kind: "observable_counter",
    monotonic: true,
    exemplars: false,
    unit: "{event}",
    population: "Process cumulative selected canonical query adapter builders and SDK response events; builders are not dispatched requests, select/control APIs are partial coverage",
    attributes: &["event"],
    attribute_values: AttributeValues::QueryEvents,
    buckets: &[],
    temporality: "cumulative",
    reset: "first query-owner activity through process incarnation; source_start carries origin, SDK activation is distinct",
    scope: "canonical Query builder/statistics observer only; excludes other select/control SDK APIs and server-wide work",
    coverage: "untimed statements do not contribute durations; absent source and saturated counts omitted; no transport parent-chain proof",
};

pub(super) const PSE_CANONICAL_SERVER_DURATION: MetricDeclaration = MetricDeclaration {
    name: "pse_canonical_server_duration",
    kind: "observable_counter",
    monotonic: true,
    exemplars: false,
    unit: "s",
    population: "Process cumulative supplied statement server wall times for the selected query adapter; untimed statements omitted, not CPU time or a complete database population",
    attributes: &[],
    attribute_values: AttributeValues::None,
    buckets: &[],
    temporality: "cumulative",
    reset: "first query-owner activity through process incarnation; source_start carries origin, SDK activation is distinct",
    scope: "canonical Query builder/statistics observer only; excludes other select/control SDK APIs and server-wide work",
    coverage: "untimed statements do not contribute durations; absent source and saturated counts omitted; no transport parent-chain proof",
};

pub(super) const PSE_OPERATIONAL_SOURCE_START: MetricDeclaration = MetricDeclaration {
    name: "pse_operational_source_start",
    kind: "observable_gauge",
    monotonic: false,
    exemplars: false,
    unit: "s",
    population: "Actual cumulative source generation origin in Unix seconds; streams reset only on process incarnation change, distinct from SDK activation",
    attributes: &["owner"],
    attribute_values: AttributeValues::Owners,
    buckets: &[],
    temporality: "instantaneous",
    reset: "process incarnation; source_start supplies first performing-owner activity, SDK activation is distinct",
    scope: "actual native supervisor/preparation owners; nested service/operation summaries overlap and are not re-added",
    coverage: "absent source omitted for totals; saturated operational totals omitted; independently copied state is not an atomic census",
};

pub(super) const PSE_RUNTIME_TASK_EVENTS: MetricDeclaration = MetricDeclaration {
    name: "pse_runtime_task_events",
    kind: "observable_counter",
    monotonic: true,
    exemplars: false,
    unit: "{event}",
    population: "Process-wide selected future wrapper events; dropped wrappers do not establish native settlement",
    attributes: &["task.class", "event"],
    attribute_values: AttributeValues::TaskEvents,
    buckets: &[],
    temporality: "cumulative",
    reset: "process-wide selected TaskMonitor source generation; source_start supplies its origin, SDK activation is distinct; library u64 totals may wrap",
    scope: "explicitly instrumented future classes only; first/slow/long-delay subsets overlap poll/scheduled populations",
    coverage: "no uninstrumented futures or native settlement inference; thresholds reported by actual monitors",
};

pub(super) const PSE_RUNTIME_TASK_DURATION: MetricDeclaration = MetricDeclaration {
    name: "pse_runtime_task_duration",
    kind: "observable_counter",
    monotonic: true,
    exemplars: false,
    unit: "s",
    population: "Cumulative selected-future poll, wake-to-poll and pending-to-wake intervals; not CPU time",
    attributes: &["task.class", "phase"],
    attribute_values: AttributeValues::TaskPhases,
    buckets: &[],
    temporality: "cumulative",
    reset: "process-wide selected TaskMonitor source generation; source_start supplies its origin, SDK activation is distinct; library u64 totals may wrap",
    scope: "explicitly instrumented future classes only; first/slow/long-delay subsets overlap poll/scheduled populations",
    coverage: "no uninstrumented futures or native settlement inference; thresholds reported by actual monitors",
};

pub(super) const PSE_RUNTIME_TASK_THRESHOLD: MetricDeclaration = MetricDeclaration {
    name: "pse_runtime_task_threshold",
    kind: "observable_gauge",
    monotonic: false,
    exemplars: false,
    unit: "s",
    population: "Actual selected monitor thresholds; slow polls and long delays are subsets, not additional work",
    attributes: &["task.class", "threshold"],
    attribute_values: AttributeValues::TaskThresholds,
    buckets: &[],
    temporality: "instantaneous",
    reset: "process-wide selected TaskMonitor source generation; source_start supplies its origin, SDK activation is distinct; library u64 totals may wrap",
    scope: "explicitly instrumented future classes only; first/slow/long-delay subsets overlap poll/scheduled populations",
    coverage: "no uninstrumented futures or native settlement inference; thresholds reported by actual monitors",
};

pub(super) const PSE_RUNTIME_TASK_SOURCE_START: MetricDeclaration = MetricDeclaration {
    name: "pse_runtime_task_source_start",
    kind: "observable_gauge",
    monotonic: false,
    exemplars: false,
    unit: "s",
    population: "Monitor source generation start in Unix seconds; SDK stream start is collection activation, not this origin",
    attributes: &["task.class"],
    attribute_values: AttributeValues::TaskClasses,
    buckets: &[],
    temporality: "instantaneous",
    reset: "process-wide selected TaskMonitor source generation; source_start supplies its origin, SDK activation is distinct; library u64 totals may wrap",
    scope: "explicitly instrumented future classes only; first/slow/long-delay subsets overlap poll/scheduled populations",
    coverage: "no uninstrumented futures or native settlement inference; thresholds reported by actual monitors",
};

pub(super) const PSE_RUNTIME_EXECUTOR_REGISTRY_AVAILABLE: MetricDeclaration = MetricDeclaration {
    name: "pse_runtime_executor_registry_available",
    kind: "observable_gauge",
    monotonic: false,
    exemplars: false,
    unit: "1",
    population: "Whether the executor source registry was available at collection",
    attributes: &[],
    attribute_values: AttributeValues::None,
    buckets: &[],
    temporality: "instantaneous",
    reset: "current registered executor population; registrations may disappear; busy totals originate at each runtime construction",
    scope: "registered Tokio executors only, aggregated by fixed composition role; not Rayon or process CPU",
    coverage: "registry-unavailable source omits executor values; approximate independent reads, not a scheduler census or settlement proof",
};

pub(super) const PSE_RUNTIME_EXECUTOR_REGISTRATION_REFUSED: MetricDeclaration = MetricDeclaration {
    name: "pse_runtime_executor_registration_refused",
    kind: "observable_counter",
    monotonic: true,
    exemplars: false,
    unit: "{registration}",
    population: "Process-lifetime executor observation registration refusals; never workload admission",
    attributes: &[],
    attribute_values: AttributeValues::None,
    buckets: &[],
    temporality: "cumulative",
    reset: "process lifetime; observation-registration refusal total never resets within a process",
    scope: "bounded executor observation registry; no workload-admission interpretation",
    coverage: "counts capacity and synchronization refusals; no execution or settlement inference",
};

pub(super) const PSE_OWNER_INTERVAL: MetricDeclaration = MetricDeclaration {
    name: "pse_owner_interval",
    kind: "histogram",
    monotonic: true,
    exemplars: false,
    unit: "s",
    population: "Completed inclusive owner wall intervals starting after SDK activation; includes failed or cancelled owners, not CPU time",
    attributes: &["owner.family", "stage"],
    attribute_values: AttributeValues::OwnerIntervals,
    buckets: &[0.000_1, 0.001, 0.01, 0.1, 1.0, 10.0, 60.0, 600.0],
    temporality: "cumulative",
    reset: "SDK activation; completed owner intervals started before activation are absent, no historical histogram reconstruction",
    scope: "native supervisor inclusive wall intervals and frontier/basis single-flight producer intervals",
    coverage: "explicit buckets; includes cancelled/failed owners; does not establish active CPU or scientific solve outcomes",
};

/// Same declarations used by native instrument builders, suitable for generated JSON publication.
pub fn metric_declarations() -> &'static [MetricDeclaration] {
    &[
        PSE_RUNTIME_EXECUTOR_WORKERS,
        PSE_RUNTIME_EXECUTOR_ALIVE_TASKS,
        PSE_RUNTIME_EXECUTOR_GLOBAL_QUEUE_DEPTH,
        PSE_RUNTIME_EXECUTOR_WORKER_BUSY,
        PSE_NATIVE_SUPERVISOR_EVENTS,
        PSE_NATIVE_SUPERVISORS,
        PSE_PREPARATION_FLIGHT_EVENTS,
        PSE_PREPARATION_PRODUCTS,
        PSE_CANONICAL_QUERY_EVENTS,
        PSE_CANONICAL_SERVER_DURATION,
        PSE_OPERATIONAL_SOURCE_START,
        PSE_RUNTIME_TASK_EVENTS,
        PSE_RUNTIME_TASK_DURATION,
        PSE_RUNTIME_TASK_THRESHOLD,
        PSE_RUNTIME_TASK_SOURCE_START,
        PSE_RUNTIME_EXECUTOR_REGISTRY_AVAILABLE,
        PSE_RUNTIME_EXECUTOR_REGISTRATION_REFUSED,
        PSE_OWNER_INTERVAL,
    ]
}

/// Closed producer vocabularies, serialized as queryable attribute-to-value maps.
#[derive(Clone, Copy, Debug)]
pub enum AttributeValues {
    /// Instrument has no dimensions.
    None,
    /// Existing executor composition-role vocabulary.
    Executor,
    /// Native supervisor lifecycle events.
    NativeEvents,
    /// Preparation stages and acquisition/reuse events.
    PreparationEvents,
    /// Native immutable preparation-product populations.
    Products,
    /// Canonical request-builder and statement events.
    QueryEvents,
    /// Original cumulative operational source owners.
    Owners,
    /// Selected future classes and observed lifecycle events.
    TaskEvents,
    /// Selected future classes and duration interval kinds.
    TaskPhases,
    /// Selected future classes and actual monitor thresholds.
    TaskThresholds,
    /// Existing selected future-class vocabulary.
    TaskClasses,
    /// Inclusive native/preparation owner interval populations.
    OwnerIntervals,
}

impl AttributeValues {
    fn domains(self) -> std::collections::BTreeMap<&'static str, Vec<&'static str>> {
        use super::operational_metrics::{
            NativeEvent, OwnerIntervalFamily, PreparationEvent, PreparationStage, SUPERVISOR_STAGE,
        };
        use super::runtime_metrics::{OperationalSource, TaskEvent, TaskPhase, TaskThreshold};
        use crate::math::PreparationProduct;
        use pse_diagnostics::{ExecutorRole, NativeTaskClass};
        use pse_operations::canonical::CanonicalQueryEvent;
        let tasks = || {
            NativeTaskClass::ALL
                .iter()
                .map(|class| class.as_str())
                .collect()
        };
        let stages = || {
            PreparationStage::ALL
                .iter()
                .map(|stage| stage.name())
                .collect()
        };
        match self {
            Self::None => std::collections::BTreeMap::new(),
            Self::Executor => [(
                "executor.role",
                ExecutorRole::ALL.iter().map(|role| role.as_str()).collect(),
            )]
            .into(),
            Self::NativeEvents => [(
                "event",
                NativeEvent::ALL
                    .into_iter()
                    .map(NativeEvent::as_str)
                    .collect(),
            )]
            .into(),
            Self::PreparationEvents => [
                ("stage", stages()),
                (
                    "event",
                    PreparationEvent::ALL
                        .into_iter()
                        .map(PreparationEvent::as_str)
                        .collect(),
                ),
            ]
            .into(),
            Self::Products => [(
                "product",
                PreparationProduct::ALL
                    .into_iter()
                    .map(PreparationProduct::as_str)
                    .collect(),
            )]
            .into(),
            Self::QueryEvents => [(
                "event",
                CanonicalQueryEvent::ALL
                    .into_iter()
                    .map(CanonicalQueryEvent::as_str)
                    .collect(),
            )]
            .into(),
            Self::Owners => [(
                "owner",
                OperationalSource::ALL
                    .into_iter()
                    .map(OperationalSource::as_str)
                    .collect(),
            )]
            .into(),
            Self::TaskEvents => [
                ("task.class", tasks()),
                (
                    "event",
                    TaskEvent::ALL.into_iter().map(TaskEvent::as_str).collect(),
                ),
            ]
            .into(),
            Self::TaskPhases => [
                ("task.class", tasks()),
                (
                    "phase",
                    TaskPhase::ALL.into_iter().map(TaskPhase::as_str).collect(),
                ),
            ]
            .into(),
            Self::TaskThresholds => [
                ("task.class", tasks()),
                (
                    "threshold",
                    TaskThreshold::ALL
                        .into_iter()
                        .map(TaskThreshold::as_str)
                        .collect(),
                ),
            ]
            .into(),
            Self::TaskClasses => [("task.class", tasks())].into(),
            Self::OwnerIntervals => [
                (
                    "owner.family",
                    OwnerIntervalFamily::ALL
                        .into_iter()
                        .map(OwnerIntervalFamily::as_str)
                        .collect(),
                ),
                (
                    "stage",
                    std::iter::once(SUPERVISOR_STAGE).chain(stages()).collect(),
                ),
            ]
            .into(),
        }
    }
}
impl serde::Serialize for AttributeValues {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serde::Serialize::serialize(&self.domains(), serializer)
    }
}

#[cfg(test)]
mod vocabulary_tests {
    #[test]
    fn metric_dimensions_are_exact_finite_producer_domains() {
        for declaration in super::metric_declarations() {
            let domains = declaration.attribute_values.domains();
            assert_eq!(
                domains
                    .keys()
                    .copied()
                    .collect::<std::collections::BTreeSet<_>>(),
                declaration.attributes.iter().copied().collect()
            );
            for values in domains.values() {
                assert!(!values.is_empty());
                assert_eq!(
                    values.len(),
                    values
                        .iter()
                        .collect::<std::collections::BTreeSet<_>>()
                        .len()
                );
            }
        }
    }
}
