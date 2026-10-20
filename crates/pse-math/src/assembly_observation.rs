// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Optional diagnostic populations, admitted separately from numerical scratch.
use super::{CaseAssembly, DemandCounts, EvaluationDemand};
use enum_map::EnumMap;
use pse_diagnostics::observation::{
    FactKind, ObservationContext, OwnerReservation, PreparedFact, Producer, ProducerRole,
    RecordAvailability, RecordField, Value,
};
use pse_kernels::DerivativeOrder;

const COUNTERS: [&str; 14] = [
    "math.demand.calls",
    "math.demand.completed",
    "math.demand.failed",
    "math.demand.evaluations",
    "math.demand.cache_hits",
    "math.demand.cache_projections",
    "math.demand.empty_misses",
    "math.demand.invalidated_misses",
    "math.demand.changed_input_misses",
    "math.demand.order_upgrade_misses",
    "math.demand.unclassified_misses",
    "math.demand.achieved_value",
    "math.demand.achieved_first",
    "math.demand.achieved_second",
];
#[derive(Debug, Default)]
pub(super) struct GroupObservation {
    pub(super) counts: EnumMap<DerivativeOrder, DemandCounts>,
    invalidated: bool,
    emitted_calls: [u64; 3],
}
#[derive(Debug)]
pub(super) struct CaseObservation {
    pub(super) producer: Producer,
    pub(super) rows: Vec<EnumMap<EvaluationDemand, Option<GroupObservation>>>,
    _reservation: OwnerReservation,
    _rows_reservation: Option<OwnerReservation>,
    pub(super) indexed_rows: bool,
    facts: [PreparedFact; 14],
    overflow: PreparedFact,
    batch: DemandCounts,
    total: DemandCounts,
    timing: Option<EvaluationTiming>,
}
impl CaseObservation {
    pub(super) fn new(context: &ObservationContext, assembly: &CaseAssembly) -> Option<Box<Self>> {
        let producer = context.producer(ProducerRole::Evaluator, "case evaluator");
        let Some(reservation) = producer.reserve_owner_bytes(size_of::<Self>() + 8192) else {
            for family in [
                "evaluator_demand_batch.v1",
                "evaluator_demand.v1",
                "evaluator_census.v1",
            ] {
                context.record_source_unavailable(family);
            }
            return None;
        };
        for family in [
            "evaluator_demand_batch.v1",
            "evaluator_demand.v1",
            "evaluator_census.v1",
        ] {
            producer.activate_record_family(family);
        }
        let facts =
            std::array::from_fn(|index| producer.fact(COUNTERS[index], "calls", FactKind::Counter));
        let overflow = producer.fact("math.demand.overflowed", "predicate", FactKind::Gauge);
        let census = assembly.census();
        for (name, unit, value) in [
            ("math.census.occurrences", "occurrences", census.occurrences),
            ("math.census.templates", "templates", census.templates),
            (
                "math.census.columns",
                "physical-coordinates",
                census.columns,
            ),
            ("math.census.rows", "rows", census.rows),
            (
                "math.census.jacobian_addends",
                "addends",
                census.jacobian_addends,
            ),
            (
                "math.census.hessian_addends",
                "addends",
                census.hessian_addends,
            ),
            ("math.census.jacobian_nnz", "entries", census.jacobian_nnz),
            ("math.census.hessian_nnz", "entries", census.hessian_nnz),
            (
                "math.census.worker_capacity",
                "bytes",
                census.numeric_worker_bytes,
            ),
        ] {
            let fact = producer.fact(name, unit, FactKind::Gauge);
            if let Ok(value) = i64::try_from(value) {
                fact.integer(value);
            } else {
                fact.unavailable("population exceeds signed diagnostic representation");
            }
        }
        let mut rows = Vec::new();
        let rows_reservation = assembly
            .instances
            .len()
            .checked_mul(size_of::<EnumMap<EvaluationDemand, Option<GroupObservation>>>())
            .and_then(|bytes| bytes.checked_mul(2))
            .and_then(|bytes| producer.reserve_owner_bytes(bytes.max(1)));
        let indexed_rows =
            rows_reservation.is_some() && rows.try_reserve_exact(assembly.instances.len()).is_ok();
        if indexed_rows {
            rows.extend(assembly.instances.iter().map(|instance| {
                EnumMap::from_fn(|demand| {
                    instance.groups[demand]
                        .as_ref()
                        .map(|_| GroupObservation::default())
                })
            }));
        } else {
            producer.fact("math.demand.indexed_rows", "availability", FactKind::Gauge)
                .unavailable("owner-local indexed demand observation storage refused; aggregate facts retained");
            if producer.detail_selected("evaluator_demand.v1") {
                let mut unavailable = field(
                    "indexed_rows",
                    "availability",
                    Value::Unavailable("owner-local indexed demand storage refused".into()),
                );
                unavailable.availability = Some(RecordAvailability::Refused);
                let record = producer.record(
                    "evaluator_demand.v1",
                    "collection_refused",
                    None,
                    None,
                    vec![unavailable],
                );
                producer.publish(&record);
            }
        }
        let result = Self {
            producer,
            rows,
            _reservation: reservation,
            _rows_reservation: if indexed_rows { rows_reservation } else { None },
            indexed_rows,
            facts,
            overflow,
            batch: DemandCounts::default(),
            total: DemandCounts::default(),
            timing: None,
        };
        result.publish_census(assembly);
        Some(Box::new(result))
    }
    pub(super) fn begin(&mut self) {
        self.batch = DemandCounts::default();
        self.timing = self
            .producer
            .detail_selected("evaluator_demand_batch.v1")
            .then(EvaluationTiming::default);
    }
    pub(super) fn evaluation_started(&self) -> Option<std::time::Instant> {
        self.timing.as_ref().map(|_| std::time::Instant::now())
    }
    pub(super) fn evaluation_finished(
        &mut self,
        started: Option<std::time::Instant>,
        completed: bool,
    ) {
        if let (Some(timing), Some(started)) = (&mut self.timing, started) {
            timing.record(started.elapsed(), completed);
        }
    }
    pub(super) fn phase(
        &self,
        demand: EvaluationDemand,
    ) -> pse_diagnostics::observation::PhaseGuard {
        let name = match demand {
            EvaluationDemand::Objective => "math.demand.objective",
            EvaluationDemand::Constraints => "math.demand.constraints",
            EvaluationDemand::All => "math.demand.all",
            EvaluationDemand::Directional => "math.demand.directional",
            EvaluationDemand::TransposeDirectional => "math.demand.transpose_directional",
            EvaluationDemand::HessianAction => "math.demand.hessian_action",
        };
        self.producer.context().phase(name)
    }
    pub(super) fn invalidate(&mut self) {
        for groups in &mut self.rows {
            for row in groups.values_mut().flatten() {
                row.invalidated = true;
            }
        }
    }
    fn apply(
        &mut self,
        instance: usize,
        demand: EvaluationDemand,
        order: DerivativeOrder,
        action: impl Fn(&mut DemandCounts),
    ) {
        if let Some(row) = self
            .rows
            .get_mut(instance)
            .and_then(|groups| groups[demand].as_mut())
        {
            action(&mut row.counts[order]);
        }
        action(&mut self.batch);
        action(&mut self.total);
    }
    pub(super) fn called(
        &mut self,
        instance: usize,
        demand: EvaluationDemand,
        order: DerivativeOrder,
    ) {
        self.apply(instance, demand, order, DemandCounts::called);
    }
    pub(super) fn failed(
        &mut self,
        instance: usize,
        demand: EvaluationDemand,
        order: DerivativeOrder,
    ) {
        self.apply(instance, demand, order, DemandCounts::failed);
        if let Some(row) = self
            .rows
            .get_mut(instance)
            .and_then(|groups| groups[demand].as_mut())
        {
            row.invalidated = true;
        }
    }
    pub(super) fn completed(
        &mut self,
        instance: usize,
        demand: EvaluationDemand,
        order: DerivativeOrder,
    ) {
        self.apply(instance, demand, order, |counts| counts.completed(order));
        if let Some(row) = self
            .rows
            .get_mut(instance)
            .and_then(|groups| groups[demand].as_mut())
        {
            row.invalidated = false;
        }
    }
    pub(super) fn hit(
        &mut self,
        instance: usize,
        demand: EvaluationDemand,
        requested: DerivativeOrder,
        achieved: DerivativeOrder,
    ) {
        self.apply(instance, demand, requested, |counts| {
            counts.hit(requested, achieved)
        });
    }
    pub(super) fn projected(
        &mut self,
        instance: usize,
        demand: EvaluationDemand,
        requested: DerivativeOrder,
        achieved: DerivativeOrder,
    ) {
        self.apply(instance, demand, requested, |counts| {
            counts.projected(achieved)
        });
    }
    pub(super) fn miss(
        &mut self,
        instance: usize,
        demand: EvaluationDemand,
        order: DerivativeOrder,
        previous: Option<DerivativeOrder>,
        same: bool,
    ) {
        let invalidated = self
            .rows
            .get(instance)
            .and_then(|groups| groups[demand].as_ref())
            .map(|row| row.invalidated);
        self.apply(instance, demand, order, |counts| {
            counts.miss(previous, same, invalidated)
        });
    }
    pub(super) fn publish(
        &mut self,
        assembly: &CaseAssembly,
        demand: EvaluationDemand,
        order: DerivativeOrder,
    ) {
        for (fact, value) in self.facts.iter().zip(values(&self.batch)) {
            if value != 0 {
                fact.count(value);
            }
        }
        self.overflow.boolean(self.total.overflowed);
        let mut fields = vec![
            field(
                "demand",
                "local-demand",
                Value::Text(demand_name(demand).into()),
            ),
            field(
                "requested_order",
                "derivative-order",
                Value::Integer(order as i64),
            ),
        ];
        fields.extend(counter_fields(&self.batch));
        if let Some(timing) = &self.timing {
            fields.extend(timing.fields());
        }
        let record = self.producer.record(
            "evaluator_demand_batch.v1",
            "evaluation_return",
            Some("physical"),
            None,
            fields,
        );
        self.producer.publish(&record);
        self.publish_selected_rows(assembly, demand, order);
    }
    fn publish_census(&self, assembly: &CaseAssembly) {
        if !self.producer.detail_selected("evaluator_census.v1") {
            return;
        }
        for template in assembly.demand_census() {
            for census in template.program.census() {
                let fields = vec![
                    field(
                        "product",
                        "mathematical-product",
                        Value::Text(format!("{:?}", census.product)),
                    ),
                    field(
                        "direct_weighted",
                        "predicate",
                        Value::Boolean(census.direct_weighted),
                    ),
                    field(
                        "derivative_components",
                        "components",
                        Value::Count(census.derivative_components as u64),
                    ),
                    field(
                        "second_pairs",
                        "symmetric-pairs",
                        Value::Count(census.second_pairs as u64),
                    ),
                    field(
                        "template",
                        "template-index",
                        Value::Count(template.template as u64),
                    ),
                    field(
                        "body",
                        "semantic-body",
                        Value::Text(template.demand.body.to_string()),
                    ),
                    field(
                        "order",
                        "derivative-order",
                        Value::Integer(census.order as i64),
                    ),
                    field(
                        "directional",
                        "predicate",
                        Value::Boolean(census.directional),
                    ),
                    field(
                        "outputs",
                        "outputs",
                        Value::Count(template.demand.outputs.len() as u64),
                    ),
                    field(
                        "coordinates",
                        "formal-coordinates",
                        Value::Count(template.demand.coordinates.len() as u64),
                    ),
                    field(
                        "stages",
                        "static-stages",
                        Value::Count(census.stages as u64),
                    ),
                    field(
                        "arithmetic_blocks",
                        "static-blocks",
                        Value::Count(census.arithmetic_blocks as u64),
                    ),
                    field(
                        "native_arithmetic_blocks",
                        "static-blocks",
                        Value::Count(census.native_arithmetic_blocks as u64),
                    ),
                    field(
                        "provider_stages",
                        "static-stages",
                        Value::Count(census.provider_stages as u64),
                    ),
                    field(
                        "control_stages",
                        "static-stages",
                        Value::Count(census.control_stages as u64),
                    ),
                    field(
                        "library_operations",
                        "static-operations",
                        Value::Count(census.library_operations as u64),
                    ),
                    field(
                        "library_instructions",
                        "static-instructions",
                        Value::Count(census.library_instructions as u64),
                    ),
                    field(
                        "instruction_storage",
                        "capacity-bytes",
                        Value::Count(census.instruction_storage_bytes as u64),
                    ),
                    field(
                        "frame_entries",
                        "capacity-entries",
                        Value::Count(census.frame_entries as u64),
                    ),
                    field(
                        "result_entries",
                        "capacity-entries",
                        Value::Count(census.result_entries as u64),
                    ),
                    field(
                        "worker_storage",
                        "capacity-bytes",
                        Value::Count(census.worker_bytes as u64),
                    ),
                ];
                let record = self.producer.record(
                    "evaluator_census.v1",
                    "prepared_template",
                    Some("formal"),
                    None,
                    fields,
                );
                self.producer.publish(&record);
            }
        }
    }
    fn publish_selected_rows(
        &mut self,
        assembly: &CaseAssembly,
        demand: EvaluationDemand,
        order: DerivativeOrder,
    ) {
        if !self.producer.detail_selected("evaluator_demand.v1") {
            return;
        }
        for (index, groups) in self.rows.iter_mut().enumerate() {
            let Some(row) = &mut groups[demand] else {
                continue;
            };
            let counts = &row.counts[order];
            if counts.calls == row.emitted_calls[order as usize] {
                continue;
            }
            row.emitted_calls[order as usize] = counts.calls;
            let Some(group) = &assembly.instances[index].groups[demand] else {
                continue;
            };
            let mut fields = vec![
                field(
                    "instance",
                    "authored-instance",
                    Value::Text(assembly.structure.instances()[index].instance.to_string()),
                ),
                field(
                    "template",
                    "template-index",
                    Value::Count(group.request as u64),
                ),
                field(
                    "demand",
                    "local-demand",
                    Value::Text(demand_name(demand).into()),
                ),
                field(
                    "requested_order",
                    "derivative-order",
                    Value::Integer(order as i64),
                ),
            ];
            fields.extend(counter_fields(counts));
            let record = self.producer.record(
                "evaluator_demand.v1",
                "occurrence_return",
                Some("physical"),
                None,
                fields,
            );
            self.producer.publish(&record);
        }
    }
}
fn values(counts: &DemandCounts) -> [u64; 14] {
    [
        counts.calls,
        counts.completed,
        counts.failed,
        counts.evaluations,
        counts.cache_hits,
        counts.cache_projections,
        counts.empty_misses,
        counts.invalidated_misses,
        counts.changed_input_misses,
        counts.order_upgrade_misses,
        counts.unclassified_misses,
        counts.achieved(DerivativeOrder::Value),
        counts.achieved(DerivativeOrder::First),
        counts.achieved(DerivativeOrder::Second),
    ]
}
fn counter_fields(counts: &DemandCounts) -> impl Iterator<Item = RecordField> + '_ {
    COUNTERS
        .into_iter()
        .zip(values(counts))
        .map(move |(name, value)| {
            field(
                name,
                "calls",
                if counts.overflowed {
                    Value::Overflow
                } else {
                    Value::Count(value)
                },
            )
        })
        .chain(std::iter::once(field(
            "overflowed",
            "predicate",
            Value::Boolean(counts.overflowed),
        )))
}
fn field(name: &str, unit: &str, value: Value) -> RecordField {
    RecordField {
        name: name.into(),
        unit: unit.into(),
        value,
        availability: None,
    }
}
fn demand_name(demand: EvaluationDemand) -> &'static str {
    match demand {
        EvaluationDemand::Objective => "objective",
        EvaluationDemand::Constraints => "constraints",
        EvaluationDemand::All => "all",
        EvaluationDemand::Directional => "directional",
        EvaluationDemand::TransposeDirectional => "transpose_directional",
        EvaluationDemand::HessianAction => "hessian_action",
    }
}

/// Selected successful actual evaluator invocations, excluding request cache/projection hits.
/// Scope includes provider/implicit work and the final execution-scope check, not value binding.
#[derive(Debug, Default)]
struct EvaluationTiming {
    completed: u64,
    failed: u64,
    completed_nanoseconds: u64,
    failed_nanoseconds: u64,
    overflow: bool,
}
impl EvaluationTiming {
    fn record(&mut self, elapsed: std::time::Duration, completed: bool) {
        let (count, nanos) = if completed {
            (&mut self.completed, &mut self.completed_nanoseconds)
        } else {
            (&mut self.failed, &mut self.failed_nanoseconds)
        };
        match (
            count.checked_add(1),
            u64::try_from(elapsed.as_nanos())
                .ok()
                .and_then(|elapsed| nanos.checked_add(elapsed)),
        ) {
            (Some(next_count), Some(next_nanos)) => {
                *count = next_count;
                *nanos = next_nanos;
            }
            _ => self.overflow = true,
        }
    }
    fn fields(&self) -> Vec<RecordField> {
        let mut fields = [
            ("actual_evaluations_completed", "calls", self.completed),
            ("actual_evaluations_failed", "calls", self.failed),
            (
                "actual_evaluations_completed_nanoseconds",
                "nanoseconds",
                self.completed_nanoseconds,
            ),
            (
                "actual_evaluations_failed_nanoseconds",
                "nanoseconds",
                self.failed_nanoseconds,
            ),
        ]
        .into_iter()
        .map(|(name, unit, value)| {
            field(
                name,
                unit,
                if self.overflow {
                    Value::Overflow
                } else {
                    Value::Count(value)
                },
            )
        })
        .collect::<Vec<_>>();
        fields.push(field("actual_evaluation_timing_scope","scope",Value::Text("inclusive actual local evaluator plus provider/implicit work and final execution-scope check; excludes input binding, cache/projection hits and assembly; successful and failed populations separate".into())));
        fields
    }
}
#[cfg(test)]
mod selected_evaluator_timing_tests {
    use super::*;
    #[test]
    fn selected_evaluator_timing_separates_completed_failed_and_overflow() {
        let mut timing = EvaluationTiming::default();
        timing.record(std::time::Duration::from_nanos(17), true);
        timing.record(std::time::Duration::from_nanos(23), false);
        let fields = timing.fields();
        assert_eq!(fields[0].value, Value::Count(1));
        assert_eq!(fields[1].value, Value::Count(1));
        assert_eq!(fields[2].value, Value::Count(17));
        assert_eq!(fields[3].value, Value::Count(23));
        timing.record(std::time::Duration::MAX, true);
        assert!(matches!(timing.fields()[0].value, Value::Overflow));
    }
}
