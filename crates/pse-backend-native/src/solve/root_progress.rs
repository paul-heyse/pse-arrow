// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Compact source-associated KINSOL evaluations, projected at real consumer boundaries.
use super::{Event, EventSource, Metric};
use pse_diagnostics::observation::{CoherentRecord, Producer, ProducerId};
use std::{collections::BTreeMap, sync::Arc, time::Duration};

const VARIABLES: [&str; 2] = ["scalar.0.variable", "scalar.1.variable"];
const ROWS: [&str; 2] = ["scalar.0.row", "scalar.1.row"];
const RESIDUALS: [&str; 2] = ["scalar.0.residual", "scalar.1.residual"];
const PHASE: &str = "kinsol.function_evaluation";

#[derive(Clone, Copy, Debug)]
enum Slot {
    Fixed,
    Ordinal,
    Residual(usize),
}
#[derive(Debug)]
struct Field {
    name: String,
    unit: &'static str,
    value: Metric,
    slot: Slot,
}

#[derive(Debug)]
pub(crate) struct RootEvaluationLayout {
    source: EventSource,
    fields: Vec<Field>,
    coordinates: usize,
}
impl RootEvaluationLayout {
    pub(crate) fn new(
        contract: &crate::OracleContract,
        picard: bool,
        role: pse_diagnostics::observation::ProducerRole,
    ) -> Arc<Self> {
        let coordinates = contract.variables.len().min(contract.rows.len()).min(2);
        let mut fields = Vec::with_capacity(4 + 3 * coordinates);
        let mut field = |name: &str, value, slot| {
            fields.push(Field {
                name: name.into(),
                unit: crate::observation::metric_unit("root_evaluation", name),
                value,
                slot,
            })
        };
        field("evaluation_ordinal", Metric::Integer(0), Slot::Ordinal);
        field(
            "output_role",
            Metric::Text(
                if picard {
                    "picard_residual"
                } else {
                    "equation_residual"
                }
                .into(),
            ),
            Slot::Fixed,
        );
        field(
            "coordinate_prefix_partial",
            Metric::Bool(contract.variables.len() > 2),
            Slot::Fixed,
        );
        field(
            "captured_coordinates",
            Metric::Integer(i64::try_from(coordinates).unwrap_or(i64::MAX)),
            Slot::Fixed,
        );
        for index in 0..coordinates {
            field(
                VARIABLES[index],
                Metric::Text(contract.variables[index].id.to_string()),
                Slot::Fixed,
            );
            field(
                ROWS[index],
                Metric::Text(contract.rows[index].to_string()),
                Slot::Fixed,
            );
            field(RESIDUALS[index], Metric::Real(0.0), Slot::Residual(index));
        }
        fields.sort_by(|left, right| left.name.cmp(&right.name));
        Arc::new(Self {
            source: EventSource::new("root_evaluation", Some("submitted_oracle"), None)
                .with_role(role),
            fields,
            coordinates,
        })
    }
    /// Conservatively charged in full by each retained compact entry and the session.
    pub(crate) fn retained_bytes(&self) -> usize {
        (size_of::<Self>() + 2 * size_of::<usize>() + self.source.retained_bytes())
            .saturating_add(self.fields.capacity().saturating_mul(size_of::<Field>()))
            .saturating_add(
                self.fields
                    .iter()
                    .map(|field| {
                        field.name.capacity()
                            + field.unit.len()
                            + match &field.value {
                                Metric::Text(value) => value.capacity(),
                                _ => 0,
                            }
                    })
                    .sum::<usize>(),
            )
    }
    pub(super) fn role(&self) -> pse_diagnostics::observation::ProducerRole {
        self.source.role
    }
    pub(super) fn family(&self) -> &str {
        &self.source.family
    }
}

#[derive(Clone, Debug, Default)]
struct Stamp {
    generation: u64,
    sequence: u64,
    producer: Option<ProducerId>,
    producer_generation: Option<u64>,
    producer_sequence: Option<u64>,
    producer_family_sequence: Option<u64>,
    observed_at: Option<Duration>,
}
#[derive(Clone, Debug)]
pub(crate) struct RootEvaluation {
    layout: Arc<RootEvaluationLayout>,
    ordinal: i64,
    residuals: [f64; 2],
    elapsed: Duration,
    stamp: Stamp,
}
impl RootEvaluation {
    pub(super) fn new(
        layout: Arc<RootEvaluationLayout>,
        ordinal: i64,
        residuals: &[f64],
        elapsed: Duration,
        generation: u64,
        sequence: u64,
    ) -> Self {
        let mut scalars = [0.0; 2];
        for (slot, value) in scalars
            .iter_mut()
            .zip(residuals.iter().take(layout.coordinates))
        {
            *slot = *value;
        }
        Self {
            layout,
            ordinal,
            residuals: scalars,
            elapsed,
            stamp: Stamp {
                generation,
                sequence,
                ..Stamp::default()
            },
        }
    }
    fn with_metric<T>(&self, field: &Field, consume: impl FnOnce(&Metric) -> T) -> T {
        match field.slot {
            Slot::Fixed => consume(&field.value),
            Slot::Ordinal => consume(&Metric::Integer(self.ordinal)),
            Slot::Residual(index) => consume(&Metric::Real(self.residuals[index])),
        }
    }
    pub(super) fn observe(&mut self, producer: Option<&Producer>) {
        let Some(producer) = producer.filter(|producer| producer.enabled()) else {
            return;
        };
        let mut fields: Vec<_> = self
            .layout
            .fields
            .iter()
            .map(|field| {
                self.with_metric(field, |metric| {
                    crate::observation::metric_field(&field.name, field.unit, metric)
                })
            })
            .collect();
        fields.push(crate::observation::elapsed_field(self.elapsed));
        let record = producer.record(
            self.layout.family(),
            PHASE,
            self.layout.source.coordinates.as_deref(),
            self.layout.source.norm.as_deref(),
            fields,
        );
        producer.publish(&record);
        self.record_stamp(&record);
    }
    fn record_stamp(&mut self, record: &CoherentRecord) {
        self.stamp.producer = Some(record.producer);
        self.stamp.producer_generation = Some(record.generation);
        self.stamp.producer_sequence = Some(record.sequence);
        self.stamp.producer_family_sequence = record.family_sequence;
        self.stamp.observed_at = Some(record.observed_at);
    }
    pub(super) fn event(&self) -> Event {
        let mut source = self.layout.source.clone();
        source.generation = self.stamp.generation;
        source.sequence = self.stamp.sequence;
        source.producer = self.stamp.producer;
        source.producer_generation = self.stamp.producer_generation;
        source.producer_sequence = self.stamp.producer_sequence;
        source.producer_family_sequence = self.stamp.producer_family_sequence;
        source.observed_at = self.stamp.observed_at;
        let values: BTreeMap<_, _> = self
            .layout
            .fields
            .iter()
            .map(|field| (field.name.clone(), self.with_metric(field, Clone::clone)))
            .collect();
        Event {
            source: Some(source),
            phase: PHASE.into(),
            elapsed: self.elapsed,
            values,
            incumbent: None,
        }
    }
    pub(super) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            .saturating_add(self.layout.retained_bytes())
            .saturating_add(128)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solve::{Progress, ProgressTap, StoredEvent};
    use crate::{DerivativeOrder, OracleContract, Variable};
    use pse_diagnostics::observation::{ObservationHub, Policy, ProducerRole, RecordTap, Value};
    use pse_ids::{ContentHash, SemanticId};
    use std::sync::{Mutex, atomic::Ordering};

    fn layout(picard: bool, role: ProducerRole) -> Arc<RootEvaluationLayout> {
        RootEvaluationLayout::new(
            &OracleContract {
                identity: ContentHash::from_bytes([17; 32]),
                variables: (1..=3)
                    .map(|id| Variable {
                        id: SemanticId::from_bytes([id; 16]),
                        lower: f64::NEG_INFINITY,
                        upper: f64::INFINITY,
                    })
                    .collect(),
                rows: (4..=6).map(|id| SemanticId::from_bytes([id; 16])).collect(),
                derivatives: DerivativeOrder::First,
                smoothness: DerivativeOrder::First,
            },
            picard,
            role,
        )
    }
    #[derive(Debug, Default)]
    struct Durable(Mutex<Vec<Event>>);
    impl ProgressTap for Durable {
        fn observe(&self, event: &Event) {
            self.0.lock().unwrap().push(event.clone());
        }
    }
    #[derive(Debug, Default)]
    struct Selected {
        records: Mutex<Vec<CoherentRecord>>,
        refused: std::sync::atomic::AtomicU64,
    }
    impl RecordTap for Selected {
        fn selected(&self, family: &str) -> bool {
            family == "root_evaluation.v1"
        }
        fn observe(&self, record: &CoherentRecord) {
            self.records.lock().unwrap().push(record.clone());
        }
        fn source_unavailable(&self, _: &str) {
            self.refused.fetch_add(1, Ordering::Relaxed);
        }
    }
    #[test]
    fn compact_projection_preserves_exact_prefix_roles_ordinals_and_nonfinite() {
        let layout = layout(true, ProducerRole::OriginalOracle);
        let progress = Progress::new(2);
        for ordinal in [i64::MAX - 2, i64::MAX - 1, i64::MAX] {
            progress.push_root_evaluation(
                layout.clone(),
                ordinal,
                &[f64::NAN, f64::NEG_INFINITY, 99.0],
                Duration::from_millis(17),
                None,
            );
        }
        let event = progress.current().unwrap();
        assert_eq!(event.values.len(), 10);
        assert_eq!(
            event.values["evaluation_ordinal"],
            Metric::Integer(i64::MAX)
        );
        assert_eq!(
            event.values["output_role"],
            Metric::Text("picard_residual".into())
        );
        assert_eq!(
            event.values["coordinate_prefix_partial"],
            Metric::Bool(true)
        );
        assert_eq!(event.values["captured_coordinates"], Metric::Integer(2));
        for (key, id) in [
            ("scalar.0.variable", 1),
            ("scalar.1.variable", 2),
            ("scalar.0.row", 4),
            ("scalar.1.row", 5),
        ] {
            assert_eq!(
                event.values[key],
                Metric::Text(SemanticId::from_bytes([id; 16]).to_string())
            );
        }
        assert!(matches!(event.values["scalar.0.residual"], Metric::Real(value) if value.is_nan()));
        assert_eq!(
            event.values["scalar.1.residual"],
            Metric::Real(f64::NEG_INFINITY)
        );
        assert_eq!(event.elapsed, Duration::from_millis(17));
        let source = event.source.unwrap();
        assert_eq!(source.role, ProducerRole::OriginalOracle);
        assert_eq!(source.family, "root_evaluation.v1");
        assert_eq!(source.coordinates.as_deref(), Some("submitted_oracle"));
        assert!(source.norm.is_none());
        assert_eq!(source.sequence, 3);
        let state = progress.events.lock().unwrap();
        assert_eq!(state.recent.len(), 2);
        for entry in state.recent.iter().chain(state.current.iter()) {
            let StoredEvent::Root(entry) = entry else {
                panic!("eager rich event retained")
            };
            assert!(Arc::ptr_eq(&layout, &entry.layout));
            assert!(
                entry.retained_bytes() >= layout.retained_bytes() + size_of::<RootEvaluation>()
            );
            assert!(entry.retained_bytes() <= 4096);
        }
        drop(state);
        assert_eq!(progress.snapshot().1, 1);
        assert_eq!(
            progress
                .snapshot()
                .0
                .iter()
                .map(|event| event.source.as_ref().unwrap().sequence)
                .collect::<Vec<_>>(),
            [2, 3]
        );
    }
    #[test]
    fn compact_history_zero_and_disabled_hub_still_publish_durable_and_current() {
        let hub = ObservationHub::new(Policy {
            enabled: false,
            ..Policy::default()
        });
        let operation = hub.start("off");
        let _entered = operation.context().enter();
        let durable = Arc::new(Durable::default());
        let progress = Progress::tapped(0, durable.clone());
        let layout = layout(false, ProducerRole::NativeCallback);
        for ordinal in 1..=2 {
            progress.push_root_evaluation(
                layout.clone(),
                ordinal,
                &[1.5, 2.5],
                Duration::ZERO,
                None,
            );
        }
        assert!(progress.snapshot().0.is_empty());
        assert_eq!(progress.dropped_count(), 2);
        assert_eq!(
            progress.current().unwrap().values["evaluation_ordinal"],
            Metric::Integer(2)
        );
        assert!(progress.observers.lock().unwrap().producers.is_empty());
        let events = durable.0.lock().unwrap();
        assert_eq!(events.len(), 2);
        for (index, event) in events.iter().enumerate() {
            let source = event.source.as_ref().unwrap();
            assert_eq!(
                (source.generation, source.sequence),
                (progress.source_identity().0, index as u64 + 1)
            );
            assert!(source.producer.is_none());
            assert_eq!(
                event.values["output_role"],
                Metric::Text("equation_residual".into())
            );
        }
    }
    #[test]
    fn compact_ordinary_record_keeps_current_without_selected_detail_or_history() {
        let hub = ObservationHub::new(Policy {
            record_capacity: 0,
            ..Policy::default()
        });
        let operation = hub.start("ordinary root");
        let _entered = operation.context().enter();
        let progress = Progress::new(0);
        progress.push_root_evaluation(
            layout(false, ProducerRole::NativeCallback),
            19,
            &[7.25, -3.5],
            Duration::from_millis(125),
            None,
        );
        let event = progress.current().unwrap();
        let snapshot = operation.reader().snapshot(Duration::from_secs(1));
        assert_eq!(snapshot.producers.len(), 1);
        let producer = &snapshot.producers[0];
        assert!(producer.recent.is_empty());
        let record = producer.current.as_ref().unwrap();
        assert_eq!(record.family, "root_evaluation.v1");
        assert_eq!(record.phase, event.phase);
        let source = event.source.as_ref().unwrap();
        assert_eq!(source.producer, Some(record.producer));
        assert_eq!(source.producer_sequence, Some(record.sequence));
        assert_eq!(source.observed_at, Some(record.observed_at));
        assert_eq!(
            record
                .values
                .iter()
                .find(|field| field.name == "evaluation_ordinal")
                .unwrap()
                .value,
            Value::Integer(19)
        );
        assert_eq!(
            record
                .values
                .iter()
                .find(|field| field.name == "scalar.1.residual")
                .unwrap()
                .value,
            Value::Number(-3.5)
        );
        assert!(progress.snapshot().0.is_empty());
    }
    #[test]
    fn compact_refusal_and_retention_contention_keep_independent_fanout_stamps() {
        let hub = ObservationHub::new(Policy {
            record_capacity: 0,
            ..Policy::default()
        });
        let operation = hub.start("science");
        let selected = Arc::new(Selected::default());
        let _entered = operation
            .context()
            .with_record_tap(selected.clone())
            .enter();
        let durable = Arc::new(Durable::default());
        let progress = Progress::tapped(0, durable.clone());
        let layout = layout(false, ProducerRole::NativeCallback);
        let held = progress.observers.lock().unwrap();
        progress.push_root_evaluation(layout.clone(), 7, &[1.0, 2.0], Duration::ZERO, None);
        assert_eq!(selected.refused.load(Ordering::Relaxed), 1);
        assert!(
            progress
                .current()
                .unwrap()
                .source
                .unwrap()
                .producer
                .is_none()
        );
        drop(held);
        let held = progress.events.lock().unwrap();
        progress.push_root_evaluation(
            layout,
            8,
            &[f64::INFINITY, 2.0],
            Duration::from_secs(3),
            None,
        );
        assert!(progress.current().is_none());
        drop(held);
        assert_eq!(
            progress.current().unwrap().values["evaluation_ordinal"],
            Metric::Integer(7)
        );
        assert_eq!(progress.dropped_count(), 2);
        let records = selected.records.lock().unwrap();
        let record = &records[0];
        let events = durable.0.lock().unwrap();
        assert_eq!(events.len(), 2);
        let source = events[1].source.as_ref().unwrap();
        assert_eq!(source.sequence, 2);
        assert_eq!(source.producer, Some(record.producer));
        assert_eq!(source.producer_sequence, Some(record.sequence));
        assert_eq!(source.producer_family_sequence, record.family_sequence);
        assert_eq!(source.producer_generation, Some(record.generation));
        assert_eq!(source.observed_at, Some(record.observed_at));
        let field = record
            .values
            .iter()
            .find(|field| field.name == "scalar.0.residual")
            .unwrap();
        assert_eq!(field.unit, "source_residual");
        assert_eq!(
            field.value,
            Value::NonFinite(pse_diagnostics::NonFinite::PositiveInfinity)
        );
        assert_eq!(
            record
                .values
                .iter()
                .find(|field| field.name == "execution_elapsed_seconds")
                .unwrap()
                .value,
            Value::Number(3.0)
        );
        let snapshot = operation.reader().snapshot(Duration::from_secs(10));
        assert_eq!(snapshot.producers.len(), 1);
        assert!(snapshot.producers[0].recent.is_empty());
        assert_eq!(
            snapshot.producers[0].current.as_ref().unwrap().sequence,
            record.sequence
        );
    }
}
