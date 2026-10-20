// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Operation-local performing owners and indivisible numerical checkpoints.
use super::*;

static PRODUCERS: AtomicU64 = AtomicU64::new(1);

/// A performing instance, not a scientific or cache identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProducerId {
    /// Actual enclosing diagnostic operation.
    pub operation: OperationId,
    /// Unique performing instance within this process.
    pub sequence: u64,
}
/// Responsibility declared at the performing owner, never inferred from a metric prefix.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProducerRole {
    /// Foreign solver callback boundary.
    NativeCallback,
    /// Original-to-native coordinate transport.
    CoordinateTransport,
    /// Original scientific oracle.
    OriginalOracle,
    /// Native-issued iteration or safe return checkpoint.
    NativeIteration,
    /// Prepared mathematical demand evaluator.
    Evaluator,
    /// Scientific property or implicit provider.
    Provider,
    /// Workflow or publication stage.
    Workflow,
    /// Native factor/setup/solve owner.
    NativeFactor,
}
/// Stable bounded producer declaration.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProducerDescriptor {
    /// Exact performing instance.
    pub id: ProducerId,
    /// Declared responsibility.
    pub role: ProducerRole,
    /// Owner-supplied label; not its identity.
    pub name: String,
}
/// Availability of an optional source observation, independent of a numerical zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RecordAvailability {
    /// Not selected or activated.
    NotRequested,
    /// The source interval is still running.
    Running,
    /// The source did not finish its interval.
    Interrupted,
    /// No fitting source capability exists.
    Unavailable,
    /// The source did not supply this field at this event.
    NotSupplied,
    /// Collection admission refused.
    Refused,
}
/// One typed member of an indivisible source-issued record.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecordField {
    /// Family-defined field name.
    pub name: String,
    /// Source-defined unit or convention.
    pub unit: String,
    /// Exact value, nonfinite category or explicit absence.
    pub value: Value,
    /// Structured source availability when not a completed measurement.
    pub availability: Option<RecordAvailability>,
}
/// All members belong to the same source event; a read never assembles a mixed row.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CoherentRecord {
    /// Performing producer and enclosing operation.
    pub producer: ProducerId,
    /// Source reset/attempt generation.
    pub generation: u64,
    /// Source sequence assigned before lossy fan-out.
    pub sequence: u64,
    /// Sequence within this exact source family; absent after counter/admission failure.
    pub family_sequence: Option<u64>,
    /// Actual producer event time on this operation's process clock.
    #[schemars(with = "crate::ClosedDuration")]
    pub observed_at: Duration,
    /// Versioned source family; native families retain different field meanings.
    pub family: String,
    /// Actual native/workflow phase.
    pub phase: String,
    /// Source coordinate convention, absent when not established.
    pub coordinates: Option<String>,
    /// Source norm convention, absent when not established.
    pub norm: Option<String>,
    /// Complete bounded tuple; large scientific payloads use their existing owner.
    pub values: Vec<RecordField>,
}
impl CoherentRecord {
    /// Conservative copied storage extent, including string/node overhead.
    #[must_use]
    pub fn retained_bytes(&self) -> usize {
        self.values.iter().fold(
            size_of::<Self>()
                .saturating_add(self.family.len())
                .saturating_add(self.phase.len())
                .saturating_add(self.coordinates.as_ref().map_or(0, String::len))
                .saturating_add(self.norm.as_ref().map_or(0, String::len)),
            |n, field| {
                n.saturating_add(size_of::<RecordField>() + 64)
                    .saturating_add(field.name.len())
                    .saturating_add(field.unit.len())
                    .saturating_add(match &field.value {
                        Value::Text(s) | Value::Unavailable(s) => s.len(),
                        _ => 0,
                    })
            },
        )
    }
}
/// Source-record retention coverage, separate from independent metric histories.
#[derive(Clone, Debug, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecordCoverage {
    /// Successfully retained whole source events.
    pub accepted: u64,
    /// Records rejected whole by admission or representation bounds.
    pub refused: u64,
    /// Recent records omitted or displaced independently of current retention.
    pub history_dropped: u64,
    /// Missing source sequences, refusal, overflow or publication contention.
    pub partial: bool,
}
/// One producer's independent current and recent record views.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProducerSnapshot {
    /// Actual declaration.
    pub descriptor: ProducerDescriptor,
    /// Latest admitted compact checkpoint, even when history is zero.
    pub current: Option<CoherentRecord>,
    /// Bounded whole records intersecting the requested interval.
    pub recent: Vec<CoherentRecord>,
    /// Independent source-record coverage.
    pub coverage: RecordCoverage,
}
/// Finite duration extrema; no inferred distribution or exclusive time.
#[derive(Clone, Debug, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DurationSummary {
    /// Completed observed intervals; overflow is explicit.
    pub samples: u64,
    /// Smallest complete interval in nanoseconds.
    pub minimum_ns: Option<u64>,
    /// Largest complete interval in nanoseconds.
    pub maximum_ns: Option<u64>,
    /// A count or duration could not be represented exactly.
    pub overflow: bool,
}
impl DurationSummary {
    fn observe(&mut self, elapsed: Duration) {
        self.overflow |= self.samples == u64::MAX;
        self.samples = self.samples.saturating_add(1);
        if let Ok(n) = u64::try_from(elapsed.as_nanos()) {
            self.minimum_ns = Some(self.minimum_ns.map_or(n, |old| old.min(n)));
            self.maximum_ns = Some(self.maximum_ns.map_or(n, |old| old.max(n)));
        } else {
            self.overflow = true;
        }
    }
}
pub(super) struct ProducerState {
    descriptor: ProducerDescriptor,
    current: Option<CoherentRecord>,
    recent: VecDeque<CoherentRecord>,
    coverage: RecordCoverage,
    current_reserved: bool,
    slots: usize,
}
impl ProducerState {
    pub(super) fn snapshot(
        &self,
        start: Duration,
        end: Duration,
        empty: bool,
        lost: u64,
    ) -> ProducerSnapshot {
        let mut coverage = self.coverage.clone();
        coverage.partial |= lost != 0;
        ProducerSnapshot {
            descriptor: self.descriptor.clone(),
            current: self.current.clone(),
            recent: self
                .recent
                .iter()
                .filter(|r| !empty && r.observed_at >= start && r.observed_at <= end)
                .cloned()
                .collect(),
            coverage,
        }
    }
}
/// Private live handle; it owns copied observation state, never native/scientific resources.
#[derive(Clone, Debug)]
pub struct Producer {
    context: ObservationContext,
    id: ProducerId,
    index: Option<usize>,
    sequence: Arc<AtomicU64>,
    descriptor: ProducerDescriptor,
    families: Arc<Mutex<FamilySequences>>,
}
/// Explicit selected record sink, independent from the ordinary bounded hub.
/// Implementations must not block numerical callbacks or change scientific permission.
pub trait RecordTap: std::any::Any + Send + Sync + std::fmt::Debug {
    /// Whether this family belongs to the active selected interval.
    fn selected(&self, family: &str) -> bool;
    /// An actual performing owner has installed this source before its work starts.
    /// This is independent from whether that owner eventually issues any events.
    fn activate(&self, _producer: &ProducerDescriptor, _family: &str) {}
    /// Explicit synchronous boundary thread-CPU selection; never enabled by default.
    fn thread_cpu(&self) -> bool {
        false
    }
    /// Whether an explicitly selected finite owner remains active.
    fn is_active(&self) -> bool {
        true
    }
    /// Optional owner-local storage from the selected collector's separate allowance.
    fn reserve_owner_bytes(&self, _bytes: usize) -> Option<OwnerReservation> {
        None
    }
    /// Observe a source-issued head before lossy publication, including a lost final event.
    fn issued(&self, _record: &CoherentRecord) {}
    /// A selected source issued work that could not obtain a producer/record mapping.
    fn source_unavailable(&self, _family: &str) {}
    /// Offer an already-issued source record; losses remain with the selected collector.
    fn observe(&self, record: &CoherentRecord);
    /// Offer bounded producer metadata before its first source record.
    fn describe(&self, _producer: &ProducerDescriptor) {}
    /// Offer metadata and record as one collector admission interval.
    fn offer(&self, record: &CoherentRecord, producer: &ProducerDescriptor) {
        self.describe(producer);
        self.observe(record);
    }
}
impl ObservationContext {
    /// Report source-registration loss before optional producer mapping can fail.
    pub fn record_source_unavailable(&self, family: &str) {
        if let Some(tap) = &self.record_tap {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if tap.selected(family) {
                    tap.source_unavailable(family);
                }
            }));
        }
    }
    /// Actual selected sink for typed owner composition; this grants no scientific authority.
    #[must_use]
    pub fn record_tap(&self) -> Option<&dyn RecordTap> {
        self.record_tap.as_deref()
    }
    /// Propagate an explicitly selected finite collector to future performing children.
    /// This never attaches to already-running work or grants scientific cancellation.
    #[must_use]
    pub fn with_record_tap(&self, tap: Arc<dyn RecordTap>) -> Self {
        let mut context = self.clone();
        context.record_tap = Some(tap);
        context
    }

    /// Register a performing instance once at its actual owner boundary.
    #[must_use]
    pub fn producer(&self, role: ProducerRole, name: &str) -> Producer {
        let id = ProducerId {
            operation: self.id,
            sequence: PRODUCERS.fetch_add(1, Ordering::Relaxed),
        };
        let mut index = None;
        if let Some(node) = &self.node {
            node.update(|state, _| {
                if name.len() > node.policy.max_text_bytes
                    || state.producers.len() >= node.policy.max_metrics
                {
                    state.refused = state.refused.saturating_add(1);
                    return;
                }
                if !state.reserve(2 * size_of::<ProducerState>() + 4 * name.len() + 512) {
                    return;
                }
                index = Some(state.producers.len());
                state.producers.push(ProducerState {
                    descriptor: ProducerDescriptor {
                        id,
                        role,
                        name: name.into(),
                    },
                    current: None,
                    recent: VecDeque::new(),
                    coverage: RecordCoverage::default(),
                    current_reserved: false,
                    slots: 0,
                });
            });
        }
        let mut context = self.clone();
        context.producer = Some(id);
        Producer {
            context,
            id,
            index,
            sequence: Arc::new(AtomicU64::new(1)),
            descriptor: ProducerDescriptor {
                id,
                role,
                name: bounded(name, 128),
            },
            families: Arc::new(Mutex::new(FamilySequences::default())),
        }
    }
}
impl Producer {
    /// Declare an applicable source at its real construction boundary. An empty
    /// completed interval then means zero issued events rather than missing activation.
    /// Calling this on an old producer does not attach a newly selected collector.
    pub fn activate_record_family(&self, family: &str) {
        if let Some(tap) = &self.context.record_tap {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if tap.selected(family) {
                    tap.activate(&self.descriptor, family);
                }
            }));
        }
    }
    /// Actual performing instance.
    #[must_use]
    pub fn id(&self) -> ProducerId {
        self.id
    }
    /// Captured performing context, suitable for explicit propagation.
    #[must_use]
    pub fn context(&self) -> &ObservationContext {
        &self.context
    }
    /// Whether ordinary or explicitly selected observation admission is available.
    #[must_use]
    pub fn enabled(&self) -> bool {
        self.index.is_some()
            || self.context.record_tap.as_ref().is_some_and(|tap| {
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| tap.is_active()))
                    .unwrap_or(false)
            })
    }
    /// Reserve optional owner-local observation storage outside scientific admission.
    /// The token must live as long as the actual local rows/copies it covers.
    #[must_use]
    pub fn reserve_owner_bytes(&self, bytes: usize) -> Option<OwnerReservation> {
        if bytes == 0 {
            return None;
        }
        if self.index.is_some()
            && let Some(node) = &self.context.node
        {
            if let Ok(mut state) = node.state.try_lock() {
                if let Some(reservation) = state.budget.reserve(bytes) {
                    return Some(OwnerReservation {
                        _reservation: reservation,
                    });
                }
                state.refused = state.refused.saturating_add(1);
            } else {
                bump(&node.lost);
            }
        }
        self.context.record_tap.as_ref().and_then(|tap| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                tap.reserve_owner_bytes(bytes)
            }))
            .ok()
            .flatten()
        })
    }
    /// Whether explicitly selected detail consumes this source family now.
    #[must_use]
    pub fn detail_selected(&self, family: &str) -> bool {
        self.context.record_tap.as_ref().is_some_and(|tap| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| tap.selected(family)))
                .unwrap_or(false)
        })
    }
    /// Explicit selected synchronous thread-CPU mode, independent from ordinary wall facts.
    #[must_use]
    pub fn thread_cpu_requested(&self) -> bool {
        self.context.record_tap.as_ref().is_some_and(|tap| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| tap.thread_cpu()))
                .unwrap_or(false)
        })
    }
    /// Prepare a population and unit once; updates use its admitted slot directly.
    #[must_use]
    pub fn fact(&self, name: &str, unit: &str, kind: FactKind) -> PreparedFact {
        let mut index = None;
        if self.index.is_some()
            && let Some(node) = &self.context.node
        {
            node.update(|state, at| {
                index = state.producer_metric_index(&node.policy, Some(self.id), name, unit, at);
                if let Some(i) = index {
                    let metric = &state.metrics[i];
                    let incompatible = (kind == FactKind::Duration && unit != "nanoseconds")
                        || (metric.counter
                            && metric.absolute.is_none()
                            && !metric.duration
                            && kind != FactKind::Counter)
                        || (metric.duration && kind != FactKind::Duration)
                        || (metric.absolute.is_some() && kind != FactKind::AbsoluteCounter)
                        || (!metric.counter
                            && !matches!(metric.latest, Value::Unavailable(_))
                            && kind != FactKind::Gauge);
                    if incompatible {
                        state.refused = state.refused.saturating_add(1);
                        index = None;
                        return;
                    }
                    if state.metrics[i]
                        .prepared_kind
                        .is_some_and(|prior| prior != kind)
                    {
                        state.refused = state.refused.saturating_add(1);
                        index = None;
                        return;
                    }
                    state.metrics[i].prepared_kind = Some(kind);
                }
                if let Some(i) = index
                    && kind != FactKind::Gauge
                    && state.metrics[i].buckets.is_empty()
                {
                    if state.reserve(node.policy.aggregate_bytes()) {
                        state.metrics[i].buckets = vec![Bucket::default(); BUCKETS];
                    } else {
                        index = None;
                    }
                }
            });
        }
        PreparedFact {
            context: self.context.clone(),
            index,
            kind,
        }
    }
    /// Issue a source event before any queue, hub or transport loss.
    #[must_use]
    pub fn record(
        &self,
        family: &str,
        phase: &str,
        coordinates: Option<&str>,
        norm: Option<&str>,
        values: Vec<RecordField>,
    ) -> CoherentRecord {
        // Issuing a real event also establishes applicability. Sources which can
        // legitimately issue no events declare activation at construction instead.
        self.activate_record_family(family);
        let family_sequence = self.next_family_sequence(family);
        let record = CoherentRecord {
            producer: self.id,
            generation: self.id.sequence,
            sequence: self
                .sequence
                .try_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
                    Some(v.saturating_add(1))
                })
                .unwrap_or(u64::MAX),
            family_sequence,
            observed_at: self
                .context
                .clock
                .as_ref()
                .map_or(Duration::ZERO, |clock| clock.now()),
            family: family.into(),
            phase: phase.into(),
            coordinates: coordinates.map(str::to_owned),
            norm: norm.map(str::to_owned),
            values,
        };
        if let Some(tap) = &self.context.record_tap {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if tap.selected(family) {
                    tap.issued(&record);
                }
            }));
        }
        record
    }
    fn next_family_sequence(&self, family: &str) -> Option<u64> {
        let mut families = self.families.try_lock().ok()?;
        if let Some((_, sequence)) = families.values.iter_mut().find(|(name, _)| name == family) {
            let issued = *sequence;
            *sequence = sequence.checked_add(1)?;
            return Some(issued);
        }
        if families.values.len() >= 32 || family.len() > 128 {
            return None;
        }
        // Family metadata is diagnostic storage, not scientific worker admission.
        let reservation = self.reserve_owner_bytes(2 * family.len() + 256)?;
        families.reservations.push(reservation);
        families.values.push((family.into(), 2));
        Some(1)
    }
    /// Atomically retain a whole compact checkpoint; rejection never changes science.
    pub fn publish(&self, record: &CoherentRecord) -> bool {
        if let Some(tap) = &self.context.record_tap {
            // A user-supplied optional collector must not unwind into native/scientific work.
            if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if tap.selected(&record.family) {
                    tap.offer(record, &self.descriptor);
                }
            }))
            .is_err()
                && let Some(node) = &self.context.node
            {
                bump(&node.lost);
            }
        }
        let (Some(index), Some(node)) = (self.index, &self.context.node) else {
            return false;
        };
        let mut accepted = false;
        node.update(|state, _| {
            let valid = record.producer == self.id
                && record.retained_bytes() <= node.policy.record_bytes
                && record.values.len() <= node.policy.max_metrics
                && record.sequence != u64::MAX
                && [&record.family, &record.phase]
                    .iter()
                    .all(|s| s.len() <= node.policy.max_text_bytes)
                && record.values.iter().all(|f| {
                    f.name.len() <= node.policy.max_text_bytes
                        && f.unit.len() <= node.policy.max_text_bytes
                });
            if !valid {
                state.producers[index].coverage.refused =
                    state.producers[index].coverage.refused.saturating_add(1);
                state.producers[index].coverage.partial = true;
                return;
            }
            if !state.producers[index].current_reserved {
                if !state.reserve(3 * node.policy.record_bytes) {
                    state.producers[index].coverage.refused =
                        state.producers[index].coverage.refused.saturating_add(1);
                    state.producers[index].coverage.partial = true;
                    return;
                }
                state.producers[index].current_reserved = true;
            }
            let prior = state.producers[index].current.as_ref();
            if prior.is_some_and(|old| {
                old.generation == record.generation && record.sequence <= old.sequence
            }) {
                state.producers[index].coverage.refused =
                    state.producers[index].coverage.refused.saturating_add(1);
                state.producers[index].coverage.partial = true;
                return;
            }
            if prior.map_or(record.sequence != 1, |old| {
                old.generation != record.generation
                    || old.sequence.checked_add(1) != Some(record.sequence)
            }) || record.family_sequence.is_none()
            {
                state.producers[index].coverage.partial = true;
            }
            state.producers[index].current = Some(record.clone());
            state.producers[index].coverage.accepted =
                state.producers[index].coverage.accepted.saturating_add(1);
            accepted = true;
            if node.policy.record_capacity == 0 {
                state.producers[index].coverage.history_dropped = state.producers[index]
                    .coverage
                    .history_dropped
                    .saturating_add(1);
                return;
            }
            if state.producers[index].recent.len() == state.producers[index].slots
                && state.producers[index].slots < node.policy.record_capacity
                && state.reserve(3 * node.policy.record_bytes)
            {
                state.producers[index].slots += 1;
            }
            let producer = &mut state.producers[index];
            if producer.slots == 0 {
                producer.coverage.history_dropped =
                    producer.coverage.history_dropped.saturating_add(1);
                producer.coverage.partial = true;
                return;
            }
            if producer.recent.len() == producer.slots {
                producer.recent.pop_front();
                producer.coverage.history_dropped =
                    producer.coverage.history_dropped.saturating_add(1);
                producer.coverage.partial = true;
            }
            producer.recent.push_back(record.clone());
        });
        accepted
    }
}
/// Stable update population, checked at preparation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FactKind {
    /// Additive count.
    Counter,
    /// Absolute counter with a reset generation.
    AbsoluteCounter,
    /// Inclusive elapsed wall intervals.
    Duration,
    /// Nonadditive typed checkpoints.
    Gauge,
}
/// Admitted operation/producer/metric slot; it is never serialized or rebound.
#[derive(Clone, Debug)]
pub struct PreparedFact {
    context: ObservationContext,
    index: Option<usize>,
    kind: FactKind,
}
impl PreparedFact {
    fn update(&self, kind: FactKind, f: impl FnOnce(&mut Metric, Duration)) {
        let (Some(index), Some(node)) = (self.index, &self.context.node) else {
            return;
        };
        node.update(|state, at| {
            if self.kind != kind {
                state.refused = state.refused.saturating_add(1);
                return;
            }
            let resets = state.metrics[index].resets;
            f(&mut state.metrics[index], at);
            let reset = state.metrics[index].resets != resets;
            state.overflow |= matches!(state.metrics[index].cumulative, Value::Overflow);
            if kind != FactKind::Counter && kind != FactKind::Duration {
                state.point(&node.policy, index, at, reset);
            }
        });
    }
    /// Add actual performed work.
    pub fn count(&self, delta: u64) {
        self.update(FactKind::Counter, |m, at| {
            m.delta(at, delta, at);
            m.latest = m.cumulative.clone();
            m.observed_at = at;
        });
    }
    /// Publish a source counter, retaining resets rather than subtracting generations.
    pub fn count_value(&self, value: u64, generation: u64) {
        self.update(FactKind::AbsoluteCounter, |m, at| {
            let reset = m.absolute.is_some()
                && (m.generation != generation || m.absolute.is_some_and(|(old, _)| value < old));
            if let Some((old, from)) = m.absolute.filter(|_| !reset) {
                m.delta(at, value - old, from);
            }
            if reset {
                m.resets = m.resets.saturating_add(1);
                let second = at.as_secs();
                let slot = &mut m.buckets[(second % BUCKETS as u64) as usize];
                if slot.second != Some(second) {
                    *slot = Bucket {
                        second: Some(second),
                        first: at,
                        last: at,
                        delta: Some(0),
                        reset: true,
                    };
                }
                slot.reset = true;
                slot.last = at;
            }
            m.counter = true;
            m.absolute = Some((value, at));
            m.generation = generation;
            m.latest = Value::Count(value);
            m.observed_at = at;
        });
    }
    /// Record one complete inclusive interval and bounded extrema.
    pub fn duration(&self, elapsed: Duration) {
        self.update(FactKind::Duration, |m, at| {
            m.elapsed(at, elapsed);
            m.duration_summary.get_or_insert_default().observe(elapsed);
        });
    }
    /// Publish a finite or explicitly nonfinite numeric checkpoint.
    pub fn gauge(&self, value: f64) {
        self.value(Value::number(value));
    }
    /// Publish an exact signed checkpoint.
    pub fn integer(&self, value: i64) {
        self.value(Value::Integer(value));
    }
    /// Publish a predicate.
    pub fn boolean(&self, value: bool) {
        self.value(Value::Boolean(value));
    }
    /// Publish a bounded textual checkpoint.
    pub fn text(&self, value: &str) {
        self.value(Value::Text(value.into()));
    }
    fn value(&self, value: Value) {
        if let Some(node) = &self.context.node
            && matches!(&value, Value::Text(s) | Value::Unavailable(s) if s.len() > node.policy.max_text_bytes)
        {
            node.update(|s, _| s.refused = s.refused.saturating_add(1));
            return;
        }
        self.update(FactKind::Gauge, |m, at| {
            copy_value(&mut m.latest, &value);
            m.observed_at = at;
        });
    }
    /// Explicit absence preserves the prepared population and unit.
    pub fn unavailable(&self, reason: &str) {
        let (Some(index), Some(node)) = (self.index, &self.context.node) else {
            return;
        };
        node.update(|state, at| {
            let m = &mut state.metrics[index];
            m.latest = Value::Unavailable(bounded(reason, node.policy.max_text_bytes));
            m.observed_at = at;
            state.point(&node.policy, index, at, false);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct TestClock(AtomicU64);
    impl Clock for TestClock {
        fn now(&self) -> Duration {
            Duration::from_millis(self.0.load(Ordering::Relaxed))
        }
    }
    fn tuple(producer: &Producer, n: i64) -> CoherentRecord {
        producer.record(
            "nlp_iteration.v1",
            "ordinary",
            Some("native_scaled"),
            Some("infinity"),
            vec![
                RecordField {
                    name: "iteration".into(),
                    unit: "iteration".into(),
                    value: Value::Integer(n),
                    availability: None,
                },
                RecordField {
                    name: "residual".into(),
                    unit: "scaled".into(),
                    value: Value::number(n as f64 * 10.0),
                    availability: None,
                },
            ],
        )
    }
    #[test]
    fn same_demand_has_distinct_performing_instances_and_fixed_population() {
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("solve");
        let outer = operation
            .context()
            .producer(ProducerRole::NativeCallback, "native");
        let inner = operation
            .context()
            .producer(ProducerRole::OriginalOracle, "original");
        let a = outer.fact("jacobian.completed", "calls", FactKind::AbsoluteCounter);
        let b = inner.fact("jacobian.completed", "calls", FactKind::AbsoluteCounter);
        a.count_value(0, 1);
        b.count_value(0, 1);
        a.count_value(1, 1);
        b.count_value(1, 1);
        a.count_value(2, 1);
        b.count_value(2, 1);
        let snapshot = operation.reader().snapshot(WINDOW);
        assert_eq!(snapshot.metrics.len(), 2);
        for metric in &snapshot.metrics {
            assert_eq!(metric.resets, 0);
            assert_eq!(metric.cumulative, Value::Count(2));
        }
        assert_ne!(snapshot.metrics[0].producer, snapshot.metrics[1].producer);
        let invalid = outer.fact("jacobian.completed", "calls", FactKind::Gauge);
        invalid.gauge(999.0);
        assert_eq!(
            operation.reader().snapshot(WINDOW).metrics[0].latest,
            Value::Count(2)
        );
    }
    #[test]
    fn coherent_current_survives_history_zero_and_same_time_events_remain_distinct() {
        let hub = ObservationHub::new(Policy {
            record_capacity: 0,
            ..Policy::default()
        });
        let operation = hub.start("solve");
        let p = operation
            .context()
            .producer(ProducerRole::NativeIteration, "iteration");
        assert!(p.publish(&tuple(&p, 1)));
        assert!(p.publish(&tuple(&p, 2)));
        let snapshot = operation.reader().snapshot(WINDOW);
        let producer = &snapshot.producers[0];
        assert!(producer.recent.is_empty());
        assert_eq!(
            producer.current.as_ref().unwrap().values[0].value,
            Value::Integer(2)
        );
        assert_eq!(
            producer.current.as_ref().unwrap().values[1].value,
            Value::Number(20.0)
        );
        assert_eq!(producer.coverage.accepted, 2);
        assert_eq!(producer.coverage.history_dropped, 2);
    }
    #[test]
    fn contention_and_oversize_never_commit_partial_rows_or_hide_sequence_gaps() {
        let clock = Arc::new(TestClock::default());
        let hub = ObservationHub::with_clock(Policy::default(), clock.clone());
        let operation = hub.start("solve");
        let p = operation
            .context()
            .producer(ProducerRole::NativeIteration, "iteration");
        assert!(p.publish(&tuple(&p, 1)));
        let held = operation
            .context
            .node
            .as_ref()
            .unwrap()
            .state
            .lock()
            .unwrap();
        assert!(!p.publish(&tuple(&p, 2)));
        drop(held);
        let mut large = tuple(&p, 3);
        large.values.push(RecordField {
            name: "text".into(),
            unit: "text".into(),
            value: Value::Text("x".repeat(65536)),
            availability: None,
        });
        assert!(!p.publish(&large));
        clock.0.store(5, Ordering::Relaxed);
        assert!(p.publish(&tuple(&p, 4)));
        let snapshot = operation.reader().snapshot(WINDOW);
        let producer = &snapshot.producers[0];
        assert_eq!(producer.recent.len(), 2);
        assert_eq!(producer.current.as_ref().unwrap().sequence, 4);
        assert!(producer.coverage.partial);
        assert_eq!(producer.coverage.refused, 1);
        assert_eq!(snapshot.coverage.dropped_updates, 1);
        assert_eq!(
            producer.current.as_ref().unwrap().observed_at,
            Duration::from_millis(5)
        );
    }
    #[test]
    fn compact_and_ring_pressure_preserve_complete_current_record() {
        let hub = ObservationHub::new(Policy {
            record_capacity: 1,
            ..Policy::default()
        });
        let operation = hub.start("solve");
        let p = operation
            .context()
            .producer(ProducerRole::NativeIteration, "iteration");
        for n in 0..3 {
            assert!(p.publish(&tuple(&p, n)));
        }
        let mut snapshot = operation.reader().snapshot(WINDOW);
        assert_eq!(snapshot.producers[0].recent.len(), 1);
        assert_eq!(snapshot.producers[0].coverage.history_dropped, 2);
        snapshot.compact();
        assert!(snapshot.producers[0].recent.is_empty());
        assert!(snapshot.producers[0].current.is_some());
        assert!(hub.retained_bytes() <= hub.policy().budget_bytes);
    }
    #[test]
    fn duration_extrema_and_disabled_admission_keep_their_actual_meanings() {
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("work");
        let p = operation
            .context()
            .producer(ProducerRole::OriginalOracle, "original");
        let duration = p.fact("jacobian.wall", "nanoseconds", FactKind::Duration);
        duration.duration(Duration::from_nanos(9));
        duration.duration(Duration::from_nanos(3));
        let snapshot = operation.reader().snapshot(WINDOW);
        let d = snapshot.metrics[0].duration_summary.as_ref().unwrap();
        assert_eq!(
            (d.samples, d.minimum_ns, d.maximum_ns),
            (2, Some(3), Some(9))
        );
        let off = ObservationHub::new(Policy {
            enabled: false,
            ..Policy::default()
        });
        let op = off.start("off");
        let p = op
            .context()
            .producer(ProducerRole::NativeIteration, "native");
        assert!(!p.enabled());
        assert!(!p.publish(&tuple(&p, 1)));
        assert!(op.reader().snapshot(WINDOW).producers.is_empty());
        assert_eq!(off.retained_bytes(), 0);
    }
}

/// Opaque optional diagnostic allocation; never charges a scientific work or storage ledger.
pub struct OwnerReservation {
    _reservation: Reservation,
}
impl std::fmt::Debug for OwnerReservation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnerReservation").finish_non_exhaustive()
    }
}

#[derive(Default)]
struct FamilySequences {
    values: Vec<(String, u64)>,
    reservations: Vec<OwnerReservation>,
}
impl std::fmt::Debug for FamilySequences {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FamilySequences")
            .field("families", &self.values.len())
            .finish_non_exhaustive()
    }
}

/// Existing diagnostic allocation mechanism composed with a separately selected finite allowance.
#[derive(Clone, Debug)]
pub struct OwnerBudget(Arc<Budget>);
impl OwnerBudget {
    /// Create an explicitly finite copied-observation allowance.
    #[must_use]
    pub fn new(bytes: usize) -> Self {
        Self(Arc::new(Budget {
            used: AtomicUsize::new(0),
            limit: bytes,
        }))
    }
    /// Reserve local observation storage before growth; the token owns its exact lifetime.
    #[must_use]
    pub fn reserve(&self, bytes: usize) -> Option<OwnerReservation> {
        self.0.reserve(bytes).map(|reservation| OwnerReservation {
            _reservation: reservation,
        })
    }
}
