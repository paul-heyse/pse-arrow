// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bounded copied observations. These handles never own scientific cancellation or resources.
//!
//! `tracing` remains the event/span facility. This module adds operation relationships,
//! coverage and bounded interval facts without depending on a subscriber or async runtime.

pub use crate::{NonFinite, OperationOutcome as Outcome, Relationship};
#[cfg(test)]
mod allocator_probe;
mod producer;
mod thread_cpu;
pub use producer::{
    CoherentRecord, DurationSummary, FactKind, FactUpdate, OwnerBudget, OwnerReservation,
    PreparedFact, Producer, ProducerDescriptor, ProducerId, ProducerRole, ProducerSnapshot,
    RecordAvailability, RecordCoverage, RecordField, RecordTap,
};
pub use thread_cpu::ThreadCpuSample;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const BUCKETS: usize = 62;
const WINDOW: Duration = Duration::from_secs(60);
static PROCESS_SEQUENCE: AtomicU64 = AtomicU64::new(1);
static OPERATION_SEQUENCE: AtomicU64 = AtomicU64::new(1);

/// Diagnostic policy, independent of scientific identities and execution limits.
#[derive(Clone, Debug)]
pub struct Policy {
    /// Whether collection was requested; disabled is absent coverage.
    pub enabled: bool,
    /// Maximum reserved diagnostic bytes for this hub; default 16 MiB.
    pub budget_bytes: usize,
    /// Catalogue bound, with an effective structural ceiling of 128.
    pub max_operations: usize,
    /// Per-operation metric and active-phase bound; default 128, ceiling 256.
    pub max_metrics: usize,
    /// Per-operation relationship bound; ceiling 128.
    pub max_children: usize,
    /// Per-operation copied source-reference bound; ceiling 32.
    pub max_source_refs: usize,
    /// Maximum copied string or source-reference byte length; effective range 1 to 256.
    pub max_text_bytes: usize,
    /// Recent detail ring capacity; ceiling 256.
    pub detail_capacity: usize,
    /// Recent coherent source records per producer; zero preserves current progress.
    pub record_capacity: usize,
    /// Maximum admitted compact record extent, including field metadata.
    pub record_bytes: usize,
    /// Time since the latest observation at which an active checkpoint is stale.
    pub stale_after: Duration,
    /// Operator-selected terminal summary lifetime, independent of recent detail retention.
    pub summary_retention: Duration,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            enabled: true,
            budget_bytes: 16 * 1024 * 1024,
            max_operations: 128,
            max_metrics: 128,
            max_children: 64,
            max_source_refs: 16,
            max_text_bytes: 128,
            detail_capacity: 256,
            record_capacity: 16,
            record_bytes: 8192,
            stale_after: Duration::from_secs(5),
            summary_retention: Duration::from_secs(60),
        }
    }
}
impl Policy {
    fn bounded(mut self) -> Self {
        // A caller-selected budget is effective; defaults are not execution caps.
        self.max_operations = self.max_operations.min(128);
        self.max_metrics = self.max_metrics.min(256);
        self.max_children = self.max_children.min(128);
        self.max_source_refs = self.max_source_refs.min(32);
        self.max_text_bytes = self.max_text_bytes.clamp(1, 256);
        self.detail_capacity = self.detail_capacity.min(256);
        self.record_capacity = self.record_capacity.min(256);
        self.record_bytes = self.record_bytes.clamp(512, 65536);
        self
    }
    // Charge structural state before starting, then bounded field storage before growth.
    // Bucket arrays are never cloned into snapshots; charging two complete arrays per metric
    // would unnecessarily deny ordinary nested operation graphs.
    fn node_bytes(&self) -> usize {
        2 * (size_of::<Node>() + size_of::<State>() + self.max_text_bytes + 4096)
    }
    fn metric_bytes(&self) -> usize {
        2 * size_of::<Metric>()
            + size_of::<MetricSnapshot>()
            + 8 * self.max_text_bytes
            + 256
            + Self::reservation_bytes()
    }
    fn aggregate_bytes(&self) -> usize {
        BUCKETS * size_of::<Bucket>() + Self::reservation_bytes()
    }
    fn history_capacity_bytes(&self, slots: usize) -> usize {
        slots * (size_of::<PointBucket>() + 2 * size_of::<PointSample>())
            + Self::reservation_bytes()
    }
    fn history_string_bytes(&self) -> usize {
        4 * self.max_text_bytes + Self::reservation_bytes()
    }
    fn detail_bytes(&self) -> usize {
        4 * size_of::<DetailEvent>() + 4 * self.max_text_bytes + 128 + Self::reservation_bytes()
    }
    fn link_bytes(&self) -> usize {
        // Weak links can retain the child's Arc allocation header after its values drop.
        2 * size_of::<LinkedNode>()
            + size_of::<ChildSnapshot>()
            + size_of::<Node>()
            + 2 * self.max_text_bytes
            + 128
            + Self::reservation_bytes()
    }
    fn source_bytes(&self) -> usize {
        3 * size_of::<SourceRef>() + 4 * self.max_text_bytes + 128 + Self::reservation_bytes()
    }
    fn phase_bytes(&self) -> usize {
        3 * size_of::<AdmittedPhase>() + 2 * self.max_text_bytes + 128 + Self::reservation_bytes()
    }
    const fn reservation_bytes() -> usize {
        2 * size_of::<Reservation>() + 64
    }
}

/// Monotonic observation clock. Test clocks must not run backwards.
pub trait Clock: Send + Sync {
    /// Return a monotonic observation timestamp.
    fn now(&self) -> Duration;
}
struct WallClock(Instant);
impl Clock for WallClock {
    fn now(&self) -> Duration {
        self.0.elapsed()
    }
}

/// Process instance plus local sequence; never a semantic or cache identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OperationId {
    /// Process-local instance locator; not a semantic identity or attestation.
    pub process_instance: [u64; 2],
    /// Unique local operation sequence.
    pub sequence: u64,
}
/// Canonical diagnostic operation locator: 32 process hex digits and a decimal sequence.
impl std::fmt::Display for OperationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:016x}{:016x}:{}",
            self.process_instance[0], self.process_instance[1], self.sequence
        )
    }
}
/// A malformed diagnostic locator, never a scientific identity-admission failure.
#[derive(Debug, thiserror::Error)]
#[error("expected a diagnostic operation as 32 hexadecimal process digits:decimal sequence")]
pub struct OperationIdParseError;
crate::impl_diagnostic! {
    OperationIdParseError,
    code(_this) { Some(crate::DiagnosticCode::RuntimeInfrastructure) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}
impl std::str::FromStr for OperationId {
    type Err = OperationIdParseError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (process, sequence) = value.split_once(':').ok_or(OperationIdParseError)?;
        if process.len() != 32
            || !process.bytes().all(|byte| byte.is_ascii_hexdigit())
            || sequence.is_empty()
            || !sequence.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err(OperationIdParseError);
        }
        Ok(Self {
            process_instance: [
                u64::from_str_radix(&process[..16], 16).map_err(|_| OperationIdParseError)?,
                u64::from_str_radix(&process[16..], 16).map_err(|_| OperationIdParseError)?,
            ],
            sequence: sequence.parse().map_err(|_| OperationIdParseError)?,
        })
    }
}
impl Outcome {
    fn code(self) -> u8 {
        match self {
            Self::Completed => 1,
            Self::Failed => 2,
            Self::Cancelled => 3,
            Self::Interrupted => 4,
        }
    }
    fn decode(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::Completed),
            2 => Some(Self::Failed),
            3 => Some(Self::Cancelled),
            4 => Some(Self::Interrupted),
            _ => None,
        }
    }
}
/// A value and its availability are inseparable.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub enum Value {
    /// Exact unsigned additive counter.
    Count(u64),
    /// Exact signed native checkpoint.
    Integer(i64),
    /// Finite numeric checkpoint.
    Number(f64),
    /// Producer-owned predicate.
    Boolean(bool),
    /// Bounded copied textual checkpoint.
    Text(String),
    /// Explicit absence with a bounded explanation.
    Unavailable(String),
    /// Original nonfinite category.
    NonFinite(NonFinite),
    /// Exact value exceeded fixed-width representation.
    Overflow,
}
impl Value {
    /// Preserve finite values and explicit IEEE nonfinite categories.
    pub fn number(value: f64) -> Self {
        if value.is_nan() {
            Self::NonFinite(NonFinite::Nan)
        } else if value == f64::INFINITY {
            Self::NonFinite(NonFinite::PositiveInfinity)
        } else if value == f64::NEG_INFINITY {
            Self::NonFinite(NonFinite::NegativeInfinity)
        } else {
            Self::Number(value)
        }
    }
}
/// Independent coverage dimensions; loss is not hidden by otherwise available values.
#[derive(Clone, Debug, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Coverage {
    /// Recent aggregates or detail have reduced temporal/detail resolution.
    pub sampled: bool,
    /// Whether collection was requested; disabled is absent coverage.
    pub enabled: bool,
    /// Whether copied evidence is available at this read boundary.
    pub available: bool,
    /// Collection or access refused observations; inspect the independent loss counts.
    pub curtailed: bool,
    /// The active operation has no sufficiently fresh published checkpoint.
    pub stale: bool,
    /// The owner guard ended without an explicit terminal explanation.
    pub interrupted: bool,
    /// Producer updates lost to contention or attempted after termination.
    pub dropped_updates: u64,
    /// Old detail events displaced by the bounded rolling ring.
    pub dropped_detail: u64,
    /// Items refused by catalogue, field or collection bounds.
    pub refused_items: u64,
    /// An exact counter exceeded its fixed-width representation.
    pub counter_overflow: bool,
}
/// Retained additive interval, copied for passive narrower-window reads.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CounterInterval {
    /// Actual first source timestamp in process-clock nanoseconds.
    pub first_ns: u64,
    /// Actual last source timestamp, not a fabricated bucket boundary.
    pub last_ns: u64,
    /// Exact retained delta; absence is arithmetic overflow.
    pub delta: Option<u64>,
    /// Source generation/reset boundary affected this interval.
    pub reset: bool,
}
/// Bounded additive history; it cannot be reconstructed from arbitrary point samples.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CounterWindow {
    /// Duration intervals have open intersection boundaries; counts have checkpoint boundaries.
    pub duration: bool,
    /// At least one actual counter interval has been observed, even if none intersects this window.
    pub observed: bool,
    /// Retained updates are incomplete independently of requested interval boundaries.
    pub partial: bool,
    /// Retained second intervals, bounded by the same low-owner retention.
    pub intervals: Vec<CounterInterval>,
}
/// Latest and cumulative facts remain available after recent detail has rolled over.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MetricSnapshot {
    /// Explicit performing producer; absent for unscoped legacy facts.
    pub producer: Option<ProducerId>,
    /// Bounded owner-local duration extrema, independent of recent retention.
    pub duration_summary: Option<DurationSummary>,
    /// Producer-owned observation name.
    pub name: String,
    /// Producer-declared unit; changes are refused.
    pub unit: String,
    /// Latest published value, retaining explicit absence and nonfinite categories.
    pub latest: Value,
    /// Observed additive work since operation start; nonadditive fields are unavailable.
    pub cumulative: Value,
    /// Sum of retained aggregates overlapping the request; partial values may include boundary work.
    pub recent: Value,
    /// Retained additive intervals for passive remote window selection; absent for gauges.
    pub counter_window: Option<CounterWindow>,
    /// Actual earliest included aggregate timestamp, including any overlapping boundary portion.
    #[schemars(with = "Option<crate::ClosedDuration>")]
    pub recent_observed_start: Option<Duration>,
    /// Actual latest included aggregate timestamp.
    #[schemars(with = "Option<crate::ClosedDuration>")]
    pub recent_observed_end: Option<Duration>,
    /// Actual monotonic producer timestamp of the latest published observation.
    #[schemars(with = "crate::ClosedDuration")]
    pub observed_at: Duration,
    /// Producer-declared generation of an absolute counter.
    pub generation: u64,
    /// Detected generation changes or backwards absolute-counter samples.
    pub resets: u64,
    /// The additive subtotal excludes an absolute baseline or has lost updates.
    pub cumulative_partial: bool,
    /// Included aggregates cross the requested boundary, contain resets or have observation loss.
    pub recent_partial: bool,
    /// Time-coalesced actual checkpoint values, with an optional immediate predecessor.
    #[serde(default)]
    pub history: Vec<PointSample>,
    /// History coalesces detail, crosses an interval boundary, has gaps or lost observations.
    #[serde(default)]
    pub history_partial: bool,
    /// Checkpoints or older retained slots lost specifically to history budget refusal.
    #[serde(default)]
    pub history_dropped: u64,
}
/// First and last checkpoint values in one observed second; intermediate detail is coalesced.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PointSample {
    /// Actual first observation timestamp, not a fabricated bucket boundary.
    #[schemars(with = "crate::ClosedDuration")]
    pub first_observed_at: Duration,
    /// Actual latest observation timestamp in this second.
    #[schemars(with = "crate::ClosedDuration")]
    pub last_observed_at: Duration,
    /// First value, retaining exact integer, absence and nonfinite meanings.
    pub first_value: Value,
    /// Latest value in the second, independent of additive work aggregates.
    pub value: Value,
    /// Number of actual observations coalesced into this sample.
    pub samples: u64,
    /// Generation of the first value; compare across generations only when the owner permits it.
    pub first_generation: u64,
    /// Generation of the latest value.
    pub generation: u64,
    /// At least one counter reset or generation transition occurred in this sample.
    pub reset: bool,
    /// At least one coalesced observation was unavailable, even if the latest became available.
    pub unavailable: bool,
    /// At least one coalesced observation was nonfinite, even if the latest became finite.
    pub nonfinite: bool,
    /// This actual checkpoint is the immediate second preceding the requested interval.
    pub predecessor: bool,
}
/// One independently active copied phase scope.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Phase {
    /// Unique active-phase token; phase destruction removes only this token.
    pub token: u64,
    /// Bounded producer-supplied operation or phase description.
    pub kind: String,
    /// Actual monotonic start time.
    #[schemars(with = "crate::ClosedDuration")]
    pub started_at: Duration,
}
/// Bounded opaque source-owned attribution.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceRef {
    /// Producer-owned reference name.
    pub name: String,
    /// Copied opaque source-owned identity or reference bytes.
    pub bytes: Vec<u8>,
}
/// Actual phase entry/exit facts, retaining token and exact timestamps for optional export.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PhaseTransition {
    /// Independently owned phase token within this operation.
    pub token: u64,
    /// Actual phase start, not export or read time.
    #[schemars(with = "crate::ClosedDuration")]
    pub started_at: Duration,
    /// Actual exit when observed; missing means an entry, not a completed interval.
    #[schemars(with = "Option<crate::ClosedDuration>")]
    pub finished_at: Option<Duration>,
}
/// One bounded actual-time detail observation.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DetailEvent {
    /// Actual monotonic event time.
    #[schemars(with = "crate::ClosedDuration")]
    pub at: Duration,
    /// Producer-owned observation name.
    pub name: String,
    /// Copied value; this event never supplies scientific permission.
    pub value: Value,
    /// Actual phase lifecycle facts when this detail came from a phase owner.
    pub phase: Option<PhaseTransition>,
}
/// Copied linked-operation summary; availability is explicit.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ChildSnapshot {
    /// Diagnostic identity of the associated operation.
    pub id: OperationId,
    /// Producer-declared relationship, independent of interval overlap.
    pub relationship: Relationship,
    /// Actual time this relationship was first observed in the parent clock.
    #[schemars(with = "crate::ClosedDuration")]
    pub observed_at: Duration,
    /// Bounded producer-supplied operation or phase description.
    pub kind: String,
    /// Explicit owner terminal explanation, or none while active or unavailable.
    pub outcome: Option<Outcome>,
    /// Actual monotonic start time.
    #[schemars(with = "crate::ClosedDuration")]
    pub started_at: Duration,
    /// Actual owner completion time, when known.
    #[schemars(with = "Option<crate::ClosedDuration>")]
    pub finished_at: Option<Duration>,
    /// Actual retained child summary, never the parent's copied metrics.
    pub available: bool,
}
/// A bounded independent read. Overlapping child times must not be added as exclusive time.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OperationSnapshot {
    /// Producer-declared responsibility and finite copied-output lifetime.
    pub summary: Option<SummaryPublication>,
    /// Diagnostic identity of the associated operation.
    pub id: OperationId,
    /// Bounded producer-supplied operation or phase description.
    pub kind: String,
    /// Actual monotonic start time.
    #[schemars(with = "crate::ClosedDuration")]
    pub started_at: Duration,
    /// Actual owner completion time, when known.
    #[schemars(with = "Option<crate::ClosedDuration>")]
    pub finished_at: Option<Duration>,
    /// Explicit owner terminal explanation, or none while active or unavailable.
    pub outcome: Option<Outcome>,
    /// Actual monotonic producer timestamp of the latest published observation.
    #[schemars(with = "Option<crate::ClosedDuration>")]
    pub observed_at: Option<Duration>,
    /// Monotonic time at which this read was requested.
    #[schemars(with = "crate::ClosedDuration")]
    pub read_at: Duration,
    /// Caller-requested lookback, anchored at read time.
    #[schemars(with = "crate::ClosedDuration")]
    pub requested_interval: Duration,
    /// Owner-selected freshness threshold; passive transport cannot replace this policy.
    #[schemars(with = "crate::ClosedDuration")]
    pub stale_after: Duration,
    /// Start of the requested interval intersected with operation lifetime.
    #[schemars(with = "crate::ClosedDuration")]
    pub interval_start: Duration,
    /// End of that lifetime intersection; may equal its start.
    #[schemars(with = "crate::ClosedDuration")]
    pub interval_end: Duration,
    /// Requested observations exceed retention or have update loss.
    pub interval_partial: bool,
    /// Detail events were displaced or updates were lost; aggregate coverage is separate.
    pub detail_partial: bool,
    /// Independent availability, freshness and loss facts.
    pub coverage: Coverage,
    /// Bounded producer-owned facts; child facts are never summed implicitly.
    pub metrics: Vec<MetricSnapshot>,
    /// Explicit producers with atomic current and independently retained recent records.
    pub producers: Vec<ProducerSnapshot>,
    /// Independently active phase scopes at the last retained checkpoint.
    pub phases: Vec<Phase>,
    /// Bounded linked-operation summaries, with explicit unavailable attribution.
    pub children: Vec<ChildSnapshot>,
    /// Bounded copied source references.
    pub sources: Vec<SourceRef>,
    /// Bounded actual-time detail prefix within the requested interval.
    pub detail: Vec<DetailEvent>,
}
impl OperationSnapshot {
    /// Keep producer-owned latest/cumulative facts and attribution, explicitly
    /// omitting optional recent histories. No aggregate is rebuilt from children.
    pub fn compact(&mut self) {
        // Optional phase/history omission must be visible to consumers deriving
        // comparisons from populations whose completion boundaries may differ.
        self.coverage.curtailed = true;
        self.detail_partial |= !self.detail.is_empty();
        self.coverage.dropped_detail = self
            .coverage
            .dropped_detail
            .saturating_add(self.detail.len() as u64);
        self.detail = Vec::new();
        for producer in &mut self.producers {
            producer.coverage.history_dropped = producer
                .coverage
                .history_dropped
                .saturating_add(producer.recent.len() as u64);
            producer.coverage.partial |= !producer.recent.is_empty();
            producer.recent.clear();
        }
        self.phases = Vec::new();
        self.interval_partial = true;
        self.coverage.sampled = true;
        for metric in &mut self.metrics {
            metric.history_partial = true;
            metric.history_dropped = metric
                .history_dropped
                .saturating_add(metric.history.len() as u64);
            metric.history = Vec::new();
            metric.counter_window = None;
            metric.recent = Value::Unavailable("compact output omits recent intervals".into());
            metric.recent_partial = true;
            metric.recent_observed_start = None;
            metric.recent_observed_end = None;
        }
    }
    /// Reapply the producer-selected semantic output deadline at each local or
    /// transported read. A retained copied reader does not extend eligibility.
    pub fn apply_summary_expiry(&mut self, now: Duration) {
        self.coverage.stale |= self
            .summary
            .as_ref()
            .and_then(|summary| summary.expires_at)
            .is_some_and(|deadline| now >= deadline);
    }
}
/// Small copied catalogue entry without metric or scientific ownership.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OperationSummary {
    /// Producer-declared responsibility and finite copied-output lifetime.
    pub summary: Option<SummaryPublication>,
    /// Diagnostic identity of the associated operation.
    pub id: OperationId,
    /// Bounded producer-supplied operation or phase description.
    pub kind: String,
    /// Actual monotonic start time.
    #[schemars(with = "crate::ClosedDuration")]
    pub started_at: Duration,
    /// Actual owner completion time, when known.
    #[schemars(with = "Option<crate::ClosedDuration>")]
    pub finished_at: Option<Duration>,
    /// Explicit owner terminal explanation, or none while active or unavailable.
    pub outcome: Option<Outcome>,
}
/// Bounded catalogue pagination with separate collection coverage.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OperationPage {
    /// Summary admission and loss facts, separate from scientific metrics.
    pub summaries: SummaryCoverage,
    /// Bounded current-catalogue page.
    pub operations: Vec<OperationSummary>,
    /// Next current-catalogue offset, when more entries are present.
    pub next_offset: Option<usize>,
    /// Independent availability, freshness and loss facts.
    pub coverage: Coverage,
}

/// Generic producer-owned compact explanation; it grants no execution ownership.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SummaryPublication {
    /// Bounded producer-supplied purpose, never interpreted by the hub.
    pub purpose: String,
    /// Finite terminal output deadline; absent for an active declaration or a refused copy.
    #[schemars(with = "Option<crate::ClosedDuration>")]
    pub expires_at: Option<Duration>,
}

/// Explicit lifetime/admission losses for producer-declared compact outputs.
#[derive(Clone, Debug, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SummaryCoverage {
    /// Declarations or terminal copies refused by bounds/contention/opt-out.
    pub refused: u64,
    /// Outputs whose selected terminal lifetime elapsed.
    pub expired: u64,
    /// Oldest terminal outputs displaced to preserve active admission.
    pub pressure_lost: u64,
}

struct RetainedSummary {
    snapshot: OperationSnapshot,
    clock: Arc<dyn Clock>,
    hub: Weak<ObservationHub>,
    _reservation: Reservation,
}
impl std::fmt::Debug for RetainedSummary {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RetainedSummary")
            .field("snapshot", &self.snapshot)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
struct Budget {
    used: AtomicUsize,
    limit: usize,
}
impl Budget {
    fn reserve(self: &Arc<Self>, bytes: usize) -> Option<Reservation> {
        self.used
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |used| {
                used.checked_add(bytes).filter(|next| *next <= self.limit)
            })
            .ok()
            .map(|_| Reservation {
                budget: Arc::clone(self),
                bytes,
            })
    }
}
#[derive(Debug)]
struct Reservation {
    budget: Arc<Budget>,
    bytes: usize,
}
impl Drop for Reservation {
    fn drop(&mut self) {
        self.budget.used.fetch_sub(self.bytes, Ordering::Relaxed);
    }
}

/// A bounded process-local catalogue of copied diagnostics only.
pub struct ObservationHub {
    policy: Policy,
    clock: Arc<dyn Clock>,
    process_instance: [u64; 2],
    nodes: Mutex<VecDeque<Arc<Node>>>,
    summaries: Mutex<VecDeque<Arc<RetainedSummary>>>,
    summary_refused: AtomicU64,
    summary_expired: AtomicU64,
    summary_pressure_lost: AtomicU64,
    budget: Arc<Budget>,
    refused: AtomicU64,
}
impl ObservationHub {
    #[must_use]
    /// Create an independent bounded hub using a monotonic clock.
    pub fn new(policy: Policy) -> Arc<Self> {
        Self::with_clock(policy, Arc::new(WallClock(Instant::now())))
    }
    #[must_use]
    /// Create a bounded hub with an injectable monotonic observation clock.
    pub fn with_clock(policy: Policy, clock: Arc<dyn Clock>) -> Arc<Self> {
        let policy = policy.bounded();
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let nonce = PROCESS_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        Arc::new(Self {
            budget: Arc::new(Budget {
                used: AtomicUsize::new(0),
                limit: policy.budget_bytes,
            }),
            policy,
            clock,
            process_instance: [
                stamp.as_secs(),
                u64::from(stamp.subsec_nanos()) ^ (u64::from(std::process::id()) << 32) ^ nonce,
            ],
            nodes: Mutex::new(VecDeque::new()),
            summaries: Mutex::new(VecDeque::new()),
            summary_refused: AtomicU64::new(0),
            summary_expired: AtomicU64::new(0),
            summary_pressure_lost: AtomicU64::new(0),
            refused: AtomicU64::new(0),
        })
    }
    #[must_use]
    /// Start diagnostics before scientific effects; refusal returns an absent diagnostic guard.
    pub fn start(self: &Arc<Self>, kind: impl AsRef<str>) -> OperationGuard {
        let id = OperationId {
            process_instance: self.process_instance,
            sequence: OPERATION_SEQUENCE.fetch_add(1, Ordering::Relaxed),
        };
        let mut context = ObservationContext {
            id,
            hub: Arc::downgrade(self),
            node: None,
            absent_enabled: self.policy.enabled,
            record_tap: None,
            clock: Some(Arc::clone(&self.clock)),
            producer: None,
        };
        if !self.policy.enabled {
            return OperationGuard { context };
        }
        let Ok(mut nodes) = self.nodes.try_lock() else {
            bump(&self.refused);
            return OperationGuard { context };
        };
        let Ok(mut summaries) = self.summaries.try_lock() else {
            bump(&self.refused);
            return OperationGuard { context };
        };
        self.expire_summaries(&mut summaries);
        // Evict completed catalogue entries, including those with independent readers.
        // Their allocation stays charged until those readers release their diagnostic lease.
        while nodes.len() + summaries.len() >= self.policy.max_operations
            || self
                .budget
                .used
                .load(Ordering::Relaxed)
                .saturating_add(self.policy.node_bytes())
                > self.budget.limit
        {
            if let Some(index) = nodes
                .iter()
                .position(|node| node.terminal.load(Ordering::Acquire) != 0)
            {
                nodes.remove(index);
            } else if summaries.pop_front().is_some() {
                bump(&self.summary_pressure_lost);
            } else {
                break;
            }
        }
        if nodes.len() + summaries.len() >= self.policy.max_operations {
            bump(&self.refused);
            return OperationGuard { context };
        }
        let Some(reservation) = self.budget.reserve(self.policy.node_bytes()) else {
            bump(&self.refused);
            return OperationGuard { context };
        };
        let node = Arc::new(Node {
            id,
            kind: bounded(kind.as_ref(), self.policy.max_text_bytes),
            hub: Arc::downgrade(self),
            started_at: self.clock.now(),
            terminal: AtomicU8::new(0),
            finished_ns: AtomicU64::new(0),
            phase_sequence: AtomicU64::new(1),
            lost: AtomicU64::new(0),
            summary: Mutex::new(None),
            policy: self.policy.clone(),
            clock: Arc::clone(&self.clock),
            state: Mutex::new({
                let mut state = State::new(Arc::clone(&self.budget), Arc::downgrade(self));
                if kind.as_ref().len() > self.policy.max_text_bytes {
                    state.refused = 1;
                }
                state
            }),
            _reservation: reservation,
        });
        nodes.push_back(Arc::clone(&node));
        context.node = Some(node);
        OperationGuard { context }
    }
    fn trim_completed(&self, bytes: usize) {
        let Ok(mut nodes) = self.nodes.try_lock() else {
            return;
        };
        let Ok(mut summaries) = self.summaries.try_lock() else {
            return;
        };
        self.expire_summaries(&mut summaries);
        while self
            .budget
            .used
            .load(Ordering::Relaxed)
            .saturating_add(bytes)
            > self.budget.limit
        {
            if let Some(index) = nodes
                .iter()
                .position(|node| node.terminal.load(Ordering::Acquire) != 0)
            {
                nodes.remove(index);
            } else if summaries.pop_front().is_some() {
                bump(&self.summary_pressure_lost);
            } else {
                break;
            }
        }
    }
    #[must_use]
    /// Borrow independent copied diagnostics without affecting scientific ownership.
    pub fn reader(&self, id: OperationId) -> Option<OperationReader> {
        self.prune_summaries();
        let node = self
            .nodes
            .lock()
            .ok()?
            .iter()
            .find(|node| node.id == id)
            .map(Arc::clone);
        if let Some(node) = node {
            return Some(OperationReader {
                context: ObservationContext {
                    id,
                    hub: node.hub.clone(),
                    node: Some(node),
                    absent_enabled: true,
                    record_tap: None,
                    clock: Some(Arc::clone(&self.clock)),
                    producer: None,
                },
                compact: None,
            });
        }
        let mut summaries = self.summaries.lock().ok()?;
        self.expire_summaries(&mut summaries);
        summaries
            .iter()
            .find(|summary| summary.snapshot.id == id)
            .map(|summary| OperationReader {
                context: ObservationContext {
                    id,
                    hub: summary.hub.clone(),
                    node: None,
                    absent_enabled: true,
                    record_tap: None,
                    clock: Some(Arc::clone(&self.clock)),
                    producer: None,
                },
                compact: Some(Arc::clone(summary)),
            })
    }
    /// Bounded pagination. Offsets describe the current catalogue, not a durable cursor.
    #[must_use]
    pub fn list(&self, offset: usize, limit: usize) -> OperationPage {
        let mut coverage = Coverage {
            enabled: self.policy.enabled,
            available: self.policy.enabled,
            refused_items: self.refused.load(Ordering::Relaxed),
            ..Coverage::default()
        };
        coverage.curtailed = coverage.refused_items != 0;
        let Ok(nodes) = self.nodes.lock() else {
            coverage.available = false;
            coverage.curtailed = true;
            return OperationPage {
                summaries: self.summary_coverage(),
                operations: Vec::new(),
                next_offset: None,
                coverage,
            };
        };
        let Ok(mut summaries) = self.summaries.lock() else {
            coverage.available = false;
            coverage.curtailed = true;
            return OperationPage {
                summaries: self.summary_coverage(),
                operations: Vec::new(),
                next_offset: None,
                coverage,
            };
        };
        self.expire_summaries(&mut summaries);
        let limit = limit.min(self.policy.max_operations);
        let operations = nodes
            .iter()
            .map(|node| node.summary())
            .chain(summaries.iter().map(|summary| {
                let snapshot = &summary.snapshot;
                OperationSummary {
                    summary: snapshot.summary.clone(),
                    id: snapshot.id,
                    kind: snapshot.kind.clone(),
                    started_at: snapshot.started_at,
                    finished_at: snapshot.finished_at,
                    outcome: snapshot.outcome,
                }
            }))
            .skip(offset)
            .take(limit)
            .collect();
        let end = offset.saturating_add(limit);
        OperationPage {
            summaries: self.summary_coverage(),
            operations,
            next_offset: (limit != 0 && end < nodes.len() + summaries.len()).then_some(end),
            coverage,
        }
    }
    fn summary_coverage(&self) -> SummaryCoverage {
        SummaryCoverage {
            refused: self.summary_refused.load(Ordering::Relaxed),
            expired: self.summary_expired.load(Ordering::Relaxed),
            pressure_lost: self.summary_pressure_lost.load(Ordering::Relaxed),
        }
    }
    fn expire_summaries(&self, summaries: &mut VecDeque<Arc<RetainedSummary>>) {
        let now = self.clock.now();
        summaries.retain(|summary| {
            let keep = summary
                .snapshot
                .summary
                .as_ref()
                .and_then(|summary| summary.expires_at)
                .is_some_and(|deadline| now < deadline);
            if !keep {
                bump(&self.summary_expired);
            }
            keep
        });
    }
    fn prune_summaries(&self) {
        if let Ok(mut summaries) = self.summaries.lock() {
            self.expire_summaries(&mut summaries);
        }
    }
    fn retain_summary(&self, node: &Arc<Node>) {
        let declaration = match node.summary.try_lock() {
            Ok(summary) => summary.clone(),
            Err(_) => {
                bump(&self.summary_refused);
                return;
            }
        };
        let Some(mut declaration) = declaration else {
            return;
        };
        let Some(deadline) = Duration::from_nanos(node.finished_ns.load(Ordering::Relaxed))
            .checked_add(self.policy.summary_retention)
        else {
            self.refuse_summary(node);
            return;
        };
        if self.policy.summary_retention.is_zero() {
            self.refuse_summary(node);
            return;
        }
        declaration.expires_at = Some(deadline);
        let reader = OperationReader {
            context: ObservationContext {
                id: node.id,
                hub: node.hub.clone(),
                node: Some(Arc::clone(node)),
                absent_enabled: true,
                record_tap: None,
                clock: Some(Arc::clone(&self.clock)),
                producer: None,
            },
            compact: None,
        };
        let mut snapshot = reader.snapshot(WINDOW);
        if !snapshot.coverage.available {
            self.refuse_summary(node);
            return;
        }
        snapshot.summary = Some(declaration);
        snapshot.compact();
        let bytes = self.policy.node_bytes()
            + snapshot.metrics.len() * self.policy.metric_bytes()
            + snapshot.children.len() * self.policy.link_bytes()
            + snapshot.sources.len() * self.policy.source_bytes()
            + snapshot.producers.len() * (2 * self.policy.record_bytes + 1024);
        // The original node and independent readers keep their own charges. A failed
        // copy cannot borrow their allocation or silently retain the rich owner instead.
        self.trim_completed(bytes);
        let Some(reservation) = self.budget.reserve(bytes) else {
            self.refuse_summary(node);
            return;
        };
        let Ok(mut nodes) = self.nodes.try_lock() else {
            self.refuse_summary(node);
            return;
        };
        let Ok(mut summaries) = self.summaries.try_lock() else {
            self.refuse_summary(node);
            return;
        };
        self.expire_summaries(&mut summaries);
        if nodes.len() + summaries.len() >= self.policy.max_operations
            && !nodes.iter().any(|entry| entry.id == node.id)
        {
            self.refuse_summary(node);
            return;
        }
        nodes.retain(|entry| entry.id != node.id);
        if let Ok(mut publication) = node.summary.try_lock() {
            *publication = snapshot.summary.clone();
        }
        summaries.push_back(Arc::new(RetainedSummary {
            snapshot,
            clock: Arc::clone(&self.clock),
            hub: node.hub.clone(),
            _reservation: reservation,
        }));
    }
    fn refuse_summary(&self, node: &Node) {
        bump(&self.summary_refused);
        if let Ok(mut publication) = node.summary.try_lock() {
            *publication = None;
        }
    }
    #[must_use]
    /// Reserved worst-case diagnostic bytes, including retained readers and a read clone.
    pub fn retained_bytes(&self) -> usize {
        self.prune_summaries();
        self.budget.used.load(Ordering::Relaxed)
    }
    #[must_use]
    /// Return the stable process-local diagnostic instance locator.
    pub fn process_instance(&self) -> [u64; 2] {
        self.process_instance
    }
    /// Effective collection bounds, including the documented structural ceilings.
    #[must_use]
    pub fn policy(&self) -> &Policy {
        &self.policy
    }
}

struct Node {
    id: OperationId,
    hub: Weak<ObservationHub>,
    kind: String,
    started_at: Duration,
    terminal: AtomicU8,
    finished_ns: AtomicU64,
    phase_sequence: AtomicU64,
    lost: AtomicU64,
    summary: Mutex<Option<SummaryPublication>>,
    policy: Policy,
    clock: Arc<dyn Clock>,
    state: Mutex<State>,
    _reservation: Reservation,
}
impl Node {
    fn summary(&self) -> OperationSummary {
        let outcome = Outcome::decode(self.terminal.load(Ordering::Acquire));
        OperationSummary {
            summary: self.summary.lock().ok().and_then(|summary| summary.clone()),
            id: self.id,
            kind: self.kind.clone(),
            started_at: self.started_at,
            finished_at: outcome
                .map(|_| Duration::from_nanos(self.finished_ns.load(Ordering::Relaxed))),
            outcome,
        }
    }
    fn update(&self, f: impl FnOnce(&mut State, Duration)) {
        self.update_at(None, f);
    }
    fn update_at(&self, observed_at: Option<Duration>, f: impl FnOnce(&mut State, Duration)) {
        if self.terminal.load(Ordering::Acquire) != 0 {
            bump(&self.lost);
            return;
        }
        match self.state.try_lock() {
            Ok(mut state) => {
                let at = observed_at.unwrap_or_else(|| self.clock.now());
                state.last_observed =
                    Some(state.last_observed.map_or(at, |previous| previous.max(at)));
                f(&mut state, at);
            }
            Err(_) => bump(&self.lost),
        }
    }
    fn finish(&self, outcome: Outcome) {
        // A guard is the sole terminal writer; readers never terminate an operation.
        self.finished_ns
            .store(nanos(self.clock.now()), Ordering::Relaxed);
        let _ =
            self.terminal
                .compare_exchange(0, outcome.code(), Ordering::Release, Ordering::Relaxed);
    }
}
struct State {
    last_observed: Option<Duration>,
    metrics: Vec<Metric>,
    producers: Vec<producer::ProducerState>,
    phases: Vec<Arc<AdmittedPhase>>,
    links: Vec<LinkedNode>,
    sources: Vec<SourceRef>,
    detail: VecDeque<DetailEvent>,
    dropped_detail: u64,
    refused: u64,
    overflow: bool,
    budget: Arc<Budget>,
    hub: Weak<ObservationHub>,
    reservations: Vec<Reservation>,
}
impl State {
    fn new(budget: Arc<Budget>, hub: Weak<ObservationHub>) -> Self {
        Self {
            last_observed: None,
            metrics: Vec::new(),
            producers: Vec::new(),
            phases: Vec::new(),
            links: Vec::new(),
            sources: Vec::new(),
            detail: VecDeque::new(),
            dropped_detail: 0,
            refused: 0,
            overflow: false,
            budget,
            hub,
            reservations: Vec::new(),
        }
    }
    fn reserve(&mut self, bytes: usize) -> bool {
        if let Some(reservation) = self.reservation(bytes) {
            self.reservations.push(reservation);
            true
        } else {
            false
        }
    }
    fn reservation(&mut self, bytes: usize) -> Option<Reservation> {
        let reservation = self.budget.reserve(bytes).or_else(|| {
            if let Some(hub) = self.hub.upgrade() {
                hub.trim_completed(bytes);
            }
            self.budget.reserve(bytes)
        });
        if reservation.is_none() {
            self.refused = self.refused.saturating_add(1);
        }
        reservation
    }
    fn detail(&mut self, policy: &Policy, at: Duration, name: &str, value: Value) {
        if policy.detail_capacity == 0 {
            self.dropped_detail = self.dropped_detail.saturating_add(1);
            return;
        }
        if self.detail.len() == policy.detail_capacity {
            self.detail.pop_front();
            self.dropped_detail = self.dropped_detail.saturating_add(1);
        } else if !self.reserve(policy.detail_bytes()) {
            self.dropped_detail = self.dropped_detail.saturating_add(1);
            // Reuse an already charged slot so budget curtailment still keeps the latest detail.
            if self.detail.pop_front().is_none() {
                return;
            }
        }
        self.detail.push_back(DetailEvent {
            phase: None,
            at,
            name: bounded(name, policy.max_text_bytes),
            value,
        });
    }
    fn producer_elapsed(
        &mut self,
        policy: &Policy,
        producer: Option<ProducerId>,
        name: &str,
        at: Duration,
        elapsed: Duration,
    ) {
        if let Some(metric) =
            self.producer_aggregate_metric(policy, producer, name, "nanoseconds", at)
        {
            if metric
                .prepared_kind
                .is_some_and(|kind| kind != FactKind::Duration)
                || (metric.counter && !metric.duration)
                || (!metric.counter && !matches!(metric.latest, Value::Unavailable(_)))
            {
                self.refused = self.refused.saturating_add(1);
                return;
            }
            metric.elapsed(at, elapsed);
            self.overflow |= matches!(metric.cumulative, Value::Overflow);
        }
    }
    fn producer_metric_index(
        &mut self,
        policy: &Policy,
        producer: Option<ProducerId>,
        name: &str,
        unit: &str,
        at: Duration,
    ) -> Option<usize> {
        if name.len() > policy.max_text_bytes || unit.len() > policy.max_text_bytes {
            self.refused = self.refused.saturating_add(1);
            return None;
        }
        if let Some(index) = self
            .metrics
            .iter()
            .position(|metric| metric.name == name && metric.producer == producer)
        {
            if self.metrics[index].unit.is_empty() && !unit.is_empty() {
                self.metrics[index].unit = unit.to_owned();
            } else if self.metrics[index].unit != unit {
                self.refused = self.refused.saturating_add(1);
                return None;
            }
            return Some(index);
        }
        if self.metrics.len() >= policy.max_metrics {
            self.refused = self.refused.saturating_add(1);
            return None;
        }
        if !self.reserve(policy.metric_bytes()) {
            return None;
        }
        self.metrics.push(Metric {
            producer,
            duration_summary: None,
            prepared_kind: None,
            name: name.to_owned(),
            unit: unit.to_owned(),
            latest: unknown(),
            cumulative: unknown(),
            observed_at: at,
            generation: 0,
            resets: 0,
            absolute: None,
            buckets: Vec::new(),
            first_count_at: None,
            counter: false,
            duration: false,
            history: Vec::new(),
            history_dropped: 0,
            history_string_slots: 0,
        });
        Some(self.metrics.len() - 1)
    }
    fn producer_aggregate_index(
        &mut self,
        policy: &Policy,
        producer: Option<ProducerId>,
        name: &str,
        unit: &str,
        at: Duration,
    ) -> Option<usize> {
        let index = self.producer_metric_index(policy, producer, name, unit, at)?;
        if self.metrics[index].buckets.is_empty() {
            if !self.reserve(policy.aggregate_bytes()) {
                return None;
            }
            self.metrics[index].buckets = vec![Bucket::default(); BUCKETS];
        }
        Some(index)
    }
    fn producer_aggregate_metric(
        &mut self,
        policy: &Policy,
        producer: Option<ProducerId>,
        name: &str,
        unit: &str,
        at: Duration,
    ) -> Option<&mut Metric> {
        let index = self.producer_aggregate_index(policy, producer, name, unit, at)?;
        self.metrics.get_mut(index)
    }
    fn point(&mut self, policy: &Policy, index: usize, at: Duration, reset: bool) {
        let second = at.as_secs();
        // Discard only out-of-window point values; their charged capacity is reusable.
        while self.metrics[index].history.first().is_some_and(|bucket| {
            bucket
                .sample
                .last_observed_at
                .as_secs()
                .saturating_add(BUCKETS as u64 - 1)
                < second
        }) {
            self.metrics[index].history.remove(0);
        }
        let same_second = self.metrics[index]
            .history
            .last()
            .is_some_and(|bucket| bucket.sample.last_observed_at.as_secs() == second);
        if !same_second {
            let metric = &self.metrics[index];
            if metric.history.len() < BUCKETS && metric.history.len() == metric.history.capacity() {
                let capacity = metric.history.capacity();
                let next = capacity.saturating_mul(2).clamp(4, BUCKETS);
                if self.reserve(policy.history_capacity_bytes(next - capacity)) {
                    self.metrics[index].history.reserve_exact(next - capacity);
                } else {
                    self.metrics[index].history_dropped =
                        self.metrics[index].history_dropped.saturating_add(1);
                    if self.metrics[index].history.is_empty() {
                        return;
                    }
                }
            }
        }
        let replacing = !same_second
            && (self.metrics[index].history.len() == BUCKETS
                || self.metrics[index].history.len() == self.metrics[index].history.capacity());
        let needs_strings = matches!(
            self.metrics[index].latest,
            Value::Text(_) | Value::Unavailable(_)
        );
        let target_strings = if same_second {
            self.metrics[index]
                .history
                .last()
                .is_some_and(|bucket| bucket.strings_reserved)
        } else if replacing {
            self.metrics[index]
                .history
                .first()
                .is_some_and(|bucket| bucket.strings_reserved)
        } else {
            false
        };
        if needs_strings && !target_strings {
            let assigned = self.metrics[index]
                .history
                .iter()
                .filter(|bucket| bucket.strings_reserved)
                .count();
            if assigned == self.metrics[index].history_string_slots {
                if !self.reserve(policy.history_string_bytes()) {
                    self.metrics[index].history_dropped =
                        self.metrics[index].history_dropped.saturating_add(1);
                    return;
                }
                self.metrics[index].history_string_slots += 1;
            }
        }
        let metric = &mut self.metrics[index];
        if same_second {
            if let Some(bucket) = metric.history.last_mut() {
                bucket.strings_reserved |= needs_strings;
                copy_value(&mut bucket.sample.value, &metric.latest);
                bucket.sample.last_observed_at = at;
                bucket.sample.reset |= reset || bucket.sample.generation != metric.generation;
                bucket.sample.generation = metric.generation;
                bucket.sample.unavailable |= matches!(metric.latest, Value::Unavailable(_));
                bucket.sample.nonfinite |= matches!(metric.latest, Value::NonFinite(_));
                self.overflow |= bucket.sample.samples == u64::MAX;
                bucket.sample.samples = bucket.sample.samples.saturating_add(1);
            }
        } else {
            let mut bucket = if replacing {
                metric.history.remove(0)
            } else {
                PointBucket {
                    sample: PointSample {
                        first_observed_at: at,
                        last_observed_at: at,
                        first_value: Value::Count(0),
                        value: Value::Count(0),
                        samples: 1,
                        first_generation: metric.generation,
                        generation: metric.generation,
                        reset,
                        unavailable: false,
                        nonfinite: false,
                        predecessor: false,
                    },
                    strings_reserved: false,
                }
            };
            bucket.strings_reserved |= needs_strings;
            bucket.sample.first_observed_at = at;
            bucket.sample.last_observed_at = at;
            copy_value(&mut bucket.sample.first_value, &metric.latest);
            copy_value(&mut bucket.sample.value, &metric.latest);
            bucket.sample.samples = 1;
            bucket.sample.first_generation = metric.generation;
            bucket.sample.generation = metric.generation;
            bucket.sample.reset = reset;
            bucket.sample.unavailable = matches!(metric.latest, Value::Unavailable(_));
            bucket.sample.nonfinite = matches!(metric.latest, Value::NonFinite(_));
            bucket.sample.predecessor = false;
            metric.history.push(bucket);
        }
    }
}
struct LinkedNode {
    id: OperationId,
    relationship: Relationship,
    observed_at: Duration,
    kind: String,
    node: Weak<Node>,
}
struct Metric {
    producer: Option<ProducerId>,
    duration_summary: Option<DurationSummary>,
    prepared_kind: Option<FactKind>,
    name: String,
    unit: String,
    latest: Value,
    cumulative: Value,
    observed_at: Duration,
    generation: u64,
    resets: u64,
    // The value and its successful sample time form one baseline. Unavailability never moves it.
    absolute: Option<(u64, Duration)>,
    buckets: Vec<Bucket>,
    first_count_at: Option<Duration>,
    counter: bool,
    duration: bool,
    history: Vec<PointBucket>,
    history_dropped: u64,
    history_string_slots: usize,
}
struct PointBucket {
    sample: PointSample,
    strings_reserved: bool,
}
#[derive(Clone, Default)]
struct Bucket {
    second: Option<u64>,
    first: Duration,
    last: Duration,
    delta: Option<u64>,
    reset: bool,
}
impl Metric {
    fn delta(&mut self, at: Duration, delta: u64, from: Duration) {
        self.counter = true;
        self.first_count_at.get_or_insert(from);
        self.cumulative = match self.cumulative {
            Value::Count(value) => value
                .checked_add(delta)
                .map_or(Value::Overflow, Value::Count),
            Value::Overflow => Value::Overflow,
            _ => Value::Count(delta),
        };
        let second = at.as_secs();
        let slot = &mut self.buckets[(second % BUCKETS as u64) as usize];
        if slot.second != Some(second) {
            *slot = Bucket {
                second: Some(second),
                first: from,
                last: at,
                delta: Some(0),
                reset: false,
            };
        }
        slot.first = slot.first.min(from);
        slot.last = at;
        slot.delta = slot.delta.and_then(|value| value.checked_add(delta));
    }
    fn elapsed(&mut self, at: Duration, elapsed: Duration) {
        self.counter = true;
        self.duration = true;
        let from = at.saturating_sub(elapsed);
        self.first_count_at.get_or_insert(from);
        let total = u64::try_from(elapsed.as_nanos()).ok();
        self.cumulative = match (&self.cumulative, total) {
            (Value::Overflow, _) | (_, None) => Value::Overflow,
            (Value::Count(old), Some(total)) => {
                old.checked_add(total).map_or(Value::Overflow, Value::Count)
            }
            (_, Some(total)) => Value::Count(total),
        };
        // Only retained seconds are materialized. Full cumulative duration remains separate.
        // A preempted phase drop may publish its actual older endpoint after a
        // newer interval. It still contributes to the exact cumulative total,
        // but must not rotate a retained newer second out of the finite ring.
        let retained_end = self.observed_at.max(at);
        let mut cursor =
            from.max(retained_end.saturating_sub(Duration::from_secs(BUCKETS as u64 - 1)));
        while cursor < at {
            let second = cursor.as_secs();
            let boundary = Duration::from_secs(second.saturating_add(1));
            if boundary <= cursor {
                break;
            }
            let end = boundary.min(at);
            let delta = u64::try_from(end.saturating_sub(cursor).as_nanos()).ok();
            let slot = &mut self.buckets[(second % BUCKETS as u64) as usize];
            if slot.second != Some(second) {
                *slot = Bucket {
                    second: Some(second),
                    first: cursor,
                    last: end,
                    delta: Some(0),
                    reset: false,
                };
            }
            slot.first = slot.first.min(cursor);
            slot.last = slot.last.max(end);
            slot.delta = slot
                .delta
                .zip(delta)
                .and_then(|(old, delta)| old.checked_add(delta));
            cursor = end;
        }
        self.latest = self.cumulative.clone();
        self.observed_at = self.observed_at.max(at);
    }
    fn recent(
        &self,
        start: Duration,
        end: Duration,
    ) -> (Value, bool, Option<Duration>, Option<Duration>) {
        if !self.counter {
            return (
                Value::Unavailable("not an additive counter".into()),
                false,
                None,
                None,
            );
        }
        if self.first_count_at.is_none() {
            return (
                Value::Unavailable("no observed counter interval".into()),
                true,
                None,
                None,
            );
        }
        let mut sum = Some(0_u64);
        let mut partial = false;
        let mut first: Option<Duration> = None;
        let mut last: Option<Duration> = None;
        for bucket in &self.buckets {
            if bucket.second.is_none() || bucket.last < start || bucket.first > end {
                continue;
            }
            if self.duration && (bucket.last <= start || bucket.first >= end) {
                continue;
            }
            partial |= bucket.first < start || bucket.last > end || bucket.reset;
            first = Some(first.map_or(bucket.first, |first| first.min(bucket.first)));
            last = Some(last.map_or(bucket.last, |last| last.max(bucket.last)));
            // An overlapping bucket is included with explicit partial coverage, not prorated.
            sum = sum
                .zip(bucket.delta)
                .and_then(|(sum, delta)| sum.checked_add(delta));
        }
        (
            sum.map_or(Value::Overflow, Value::Count),
            partial,
            first,
            last,
        )
    }
}
fn copy_value(destination: &mut Value, source: &Value) {
    match (destination, source) {
        (Value::Text(destination), Value::Text(source))
        | (Value::Unavailable(destination), Value::Unavailable(source)) => {
            destination.clone_from(source)
        }
        (destination, source) => *destination = source.clone(),
    }
}
fn unknown() -> Value {
    Value::Unavailable("not observed".into())
}
fn bounded(text: &str, limit: usize) -> String {
    let mut end = text.len().min(limit);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].to_owned()
}
fn nanos(value: Duration) -> u64 {
    u64::try_from(value.as_nanos()).unwrap_or(u64::MAX)
}
fn bump(counter: &AtomicU64) {
    let _ = counter.try_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
        Some(value.saturating_add(1))
    });
}

/// Cloneable diagnostic producer context; retains only copied diagnostic state.
#[derive(Clone)]
pub struct ObservationContext {
    id: OperationId,
    hub: Weak<ObservationHub>,
    node: Option<Arc<Node>>,
    absent_enabled: bool,
    producer: Option<ProducerId>,
    record_tap: Option<Arc<dyn RecordTap>>,
    clock: Option<Arc<dyn Clock>>,
}
impl ObservationContext {
    /// Effective byte ceiling for copied text in this selected operation.
    /// Absent when collection was disabled or this operation was not admitted.
    #[must_use]
    pub fn text_limit_bytes(&self) -> Option<usize> {
        self.node.as_ref().map(|node| node.policy.max_text_bytes)
    }
    /// Return the operation diagnostic identity.
    #[must_use]
    pub fn id(&self) -> OperationId {
        self.id
    }
    /// Whether this retained diagnostic owner has not recorded a terminal outcome.
    /// Absent collection is false; this grants no scientific execution permission.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.node
            .as_ref()
            .is_some_and(|node| node.terminal.load(Ordering::Acquire) == 0)
    }
    /// Borrow independent copied diagnostics without affecting scientific ownership.
    #[must_use]
    pub fn reader(&self) -> OperationReader {
        if let Some(hub) = self.hub.upgrade() {
            hub.prune_summaries();
        }
        OperationReader {
            context: self.clone(),
            compact: None,
        }
    }
    /// Declare this producer's bounded current/cumulative facts as a responsibility
    /// summary. The declaration never changes execution completion or numerical admission.
    pub fn declare_summary(&self, purpose: impl AsRef<str>) {
        let Some(hub) = self.hub.upgrade() else {
            return;
        };
        let Some(node) = &self.node else {
            bump(&hub.summary_refused);
            return;
        };
        if node.terminal.load(Ordering::Acquire) != 0
            || purpose.as_ref().is_empty()
            || purpose.as_ref().len() > node.policy.max_text_bytes
        {
            bump(&hub.summary_refused);
            return;
        }
        let Ok(mut summary) = node.summary.try_lock() else {
            bump(&hub.summary_refused);
            return;
        };
        // The bounded purpose fits the string allowance charged with the node.
        *summary = Some(SummaryPublication {
            purpose: purpose.as_ref().to_owned(),
            expires_at: None,
        });
    }
    #[must_use]
    /// Start and link diagnostic child work without implicitly entering its thread context.
    pub fn child(&self, kind: impl AsRef<str>, relationship: Relationship) -> OperationGuard {
        if let Some(hub) = self.hub.upgrade() {
            let mut child = hub.start(kind);
            child.context.record_tap = self.record_tap.clone();
            self.link(&child.context, relationship);
            child
        } else {
            OperationGuard {
                context: Self {
                    id: OperationId {
                        process_instance: self.id.process_instance,
                        sequence: OPERATION_SEQUENCE.fetch_add(1, Ordering::Relaxed),
                    },
                    hub: Weak::new(),
                    node: None,
                    absent_enabled: self.absent_enabled,
                    record_tap: self.record_tap.clone(),
                    clock: self.clock.clone(),
                    producer: None,
                },
            }
        }
    }
    /// Link existing shared production or a dependency without copying its counters.
    pub fn link(&self, child: &Self, relationship: Relationship) {
        let Some(node) = &self.node else {
            return;
        };
        node.update(|state, at| {
            if state
                .links
                .iter()
                .any(|link| link.id == child.id && link.relationship == relationship)
            {
                return;
            }
            if state.links.len() >= node.policy.max_children {
                state.refused = state.refused.saturating_add(1);
                return;
            }
            if !state.reserve(node.policy.link_bytes()) {
                return;
            }
            state.links.push(LinkedNode {
                id: child.id,
                relationship,
                observed_at: at,
                kind: child
                    .node
                    .as_ref()
                    .map_or_else(String::new, |node| node.kind.clone()),
                node: child.node.as_ref().map_or_else(Weak::new, Arc::downgrade),
            });
        });
    }
    /// Associate a remote or evicted operation without inventing a local summary.
    pub fn link_id(&self, id: OperationId, relationship: Relationship) {
        let Some(node) = &self.node else {
            return;
        };
        node.update(|state, at| {
            if state
                .links
                .iter()
                .any(|link| link.id == id && link.relationship == relationship)
            {
                return;
            }
            if state.links.len() >= node.policy.max_children {
                state.refused = state.refused.saturating_add(1);
                return;
            }
            if !state.reserve(node.policy.link_bytes()) {
                return;
            }
            state.links.push(LinkedNode {
                id,
                relationship,
                observed_at: at,
                kind: String::new(),
                node: Weak::new(),
            });
        });
    }
    /// Exact signed native checkpoint, independent of additive counters.
    pub fn integer(&self, name: &str, unit: &str, value: i64) {
        self.latest(name, unit, Value::Integer(value));
    }
    /// Publish a bounded producer-owned predicate checkpoint.
    pub fn boolean(&self, name: &str, value: bool) {
        self.latest(name, "boolean", Value::Boolean(value));
    }
    /// Publish bounded textual state; overlong values are explicitly refused.
    pub fn text(&self, name: &str, value: &str) {
        let Some(node) = &self.node else {
            return;
        };
        if value.len() > node.policy.max_text_bytes {
            node.update(|state, _| {
                state.refused = state.refused.saturating_add(1);
            });
            return;
        }
        self.latest(name, "text", Value::Text(value.to_owned()));
    }
    fn latest(&self, name: &str, unit: &str, value: Value) {
        let Some(node) = &self.node else {
            return;
        };
        node.update(|state, at| {
            if let Some(index) =
                state.producer_metric_index(&node.policy, self.producer, name, unit, at)
            {
                let metric = &mut state.metrics[index];
                if metric
                    .prepared_kind
                    .is_some_and(|kind| kind != FactKind::Gauge)
                    || metric.counter
                {
                    state.refused = state.refused.saturating_add(1);
                    return;
                }
                if !matches!(value, Value::Unavailable(_)) {
                    metric.prepared_kind = Some(FactKind::Gauge);
                }
                copy_value(&mut metric.latest, &value);
                metric.observed_at = at;
                state.point(&node.policy, index, at, false);
            }
        });
    }
    /// Exact producer-owned increment in the supplied unit.
    pub fn count(&self, name: &str, unit: &str, delta: u64) {
        let Some(node) = &self.node else {
            return;
        };
        node.update(|state, at| {
            if let Some(metric) =
                state.producer_aggregate_metric(&node.policy, self.producer, name, unit, at)
            {
                if metric
                    .prepared_kind
                    .is_some_and(|kind| kind != FactKind::Counter)
                    || metric.duration
                    || metric.absolute.is_some()
                    || (!metric.counter && !matches!(metric.latest, Value::Unavailable(_)))
                {
                    state.refused = state.refused.saturating_add(1);
                    return;
                }
                metric.delta(at, delta, at);
                metric.latest = metric.cumulative.clone();
                metric.observed_at = at;
                state.overflow |= matches!(metric.cumulative, Value::Overflow);
            }
        });
    }
    /// Record producer-owned elapsed wall time ending now, in exact nanoseconds.
    /// Retained time is split across actual second intersections; overlapping scopes remain additive work,
    /// not exclusive or critical-path time. Boundary-bucket partial coverage remains explicit.
    pub fn duration(&self, name: &str, elapsed: Duration) {
        let Some(node) = &self.node else {
            return;
        };
        node.update(|state, at| {
            state.producer_elapsed(&node.policy, self.producer, name, at, elapsed);
        });
    }
    /// Absolute counter sample; generations and backwards samples are reset boundaries.
    /// The first sample is a baseline and never fabricates pre-observation increments.
    pub fn count_value(&self, name: &str, unit: &str, value: u64, generation: u64) {
        let Some(node) = &self.node else {
            return;
        };
        node.update(|state, at| {
            if let Some(index) =
                state.producer_aggregate_index(&node.policy, self.producer, name, unit, at)
            {
                let metric = &mut state.metrics[index];
                if metric
                    .prepared_kind
                    .is_some_and(|kind| kind != FactKind::AbsoluteCounter)
                    || metric.duration
                    || (metric.counter && metric.absolute.is_none())
                {
                    state.refused = state.refused.saturating_add(1);
                    return;
                }
                if !metric.counter && !matches!(metric.latest, Value::Unavailable(_)) {
                    state.refused = state.refused.saturating_add(1);
                    return;
                }
                metric.counter = true;
                let reset = metric.absolute.is_some()
                    && (metric.generation != generation
                        || metric.absolute.is_some_and(|(old, _)| value < old));
                if let Some((old, from)) = metric.absolute.filter(|_| !reset) {
                    metric.delta(at, value - old, from);
                }
                if reset {
                    metric.resets = metric.resets.saturating_add(1);
                    let second = at.as_secs();
                    let slot = &mut metric.buckets[(second % BUCKETS as u64) as usize];
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
                metric.absolute = Some((value, at));
                metric.generation = generation;
                metric.latest = Value::Count(value);
                metric.observed_at = at;
                state.overflow |= matches!(metric.cumulative, Value::Overflow);
                state.point(&node.policy, index, at, reset);
            }
        });
    }
    /// Publish a numeric checkpoint, retaining nonfinite values explicitly.
    pub fn gauge(&self, name: &str, unit: &str, value: f64) {
        self.latest(name, unit, Value::number(value));
    }
    /// Publish explicit absence without fabricating numerical work.
    pub fn unavailable(&self, name: &str, reason: &str) {
        let Some(node) = &self.node else {
            return;
        };
        node.update(|state, at| {
            // Unavailability does not invent a unit or change a previously declared one.
            let unit = state
                .metrics
                .iter()
                .find(|metric| metric.name == name && metric.producer == self.producer)
                .map(|metric| metric.unit.clone())
                .unwrap_or_default();
            if let Some(index) =
                state.producer_metric_index(&node.policy, self.producer, name, &unit, at)
            {
                let metric = &mut state.metrics[index];
                let value = Value::Unavailable(bounded(reason, node.policy.max_text_bytes));
                copy_value(&mut metric.latest, &value);
                metric.observed_at = at;
                state.point(&node.policy, index, at, false);
            }
        });
    }
    /// Source references are copied only when they fit. No scientific objects are retained.
    pub fn source_ref(&self, name: &str, bytes: impl AsRef<[u8]>) {
        let Some(node) = &self.node else {
            return;
        };
        let bytes = bytes.as_ref();
        node.update(|state, _| {
            if name.len() > node.policy.max_text_bytes || bytes.len() > node.policy.max_text_bytes {
                state.refused = state.refused.saturating_add(1);
                return;
            }
            if let Some(source) = state.sources.iter_mut().find(|source| source.name == name) {
                source.bytes = bytes.to_vec();
            } else if state.sources.len() < node.policy.max_source_refs {
                if !state.reserve(node.policy.source_bytes()) {
                    return;
                }
                state.sources.push(SourceRef {
                    name: name.into(),
                    bytes: bytes.to_vec(),
                });
            } else {
                state.refused = state.refused.saturating_add(1);
            }
        });
    }
    #[must_use]
    /// Enter an independently owned elapsed phase scope; scope exit does not imply success.
    pub fn phase(&self, kind: impl AsRef<str>) -> PhaseGuard {
        let mut admitted = None;
        if let Some(node) = &self.node {
            let next = node.phase_sequence.fetch_add(1, Ordering::Relaxed);
            node.update(|state, at| {
                // Retirement is independent of this mutex. Reclaim its population before
                // admitting another scope, even when the optional exit update was lost.
                state
                    .phases
                    .retain(|phase| phase.active.load(Ordering::Acquire));
                if state.phases.len() >= node.policy.max_metrics {
                    state.refused = state.refused.saturating_add(1);
                    return;
                }
                let Some(reservation) = state.reservation(node.policy.phase_bytes()) else {
                    return;
                };
                if kind.as_ref().len() > node.policy.max_text_bytes {
                    state.refused = state.refused.saturating_add(1);
                }
                let kind = bounded(kind.as_ref(), node.policy.max_text_bytes);
                state.detail(
                    &node.policy,
                    at,
                    &kind,
                    Value::Unavailable("phase entered".into()),
                );
                if let Some(event) = state
                    .detail
                    .back_mut()
                    .filter(|event| event.at == at && event.name == kind)
                {
                    event.phase = Some(PhaseTransition {
                        token: next,
                        started_at: at,
                        finished_at: None,
                    });
                }
                let phase = Arc::new(AdmittedPhase {
                    phase: Phase {
                        token: next,
                        kind,
                        started_at: at,
                    },
                    active: AtomicBool::new(true),
                    _reservation: reservation,
                });
                state.phases.push(Arc::clone(&phase));
                admitted = Some(phase);
            });
        }
        PhaseGuard {
            context: self.clone(),
            admitted,
        }
    }
    /// Synchronous thread-local scope. Async code must install context on each poll.
    #[must_use]
    pub fn enter(&self) -> EnterGuard {
        enter_optional(Some(self.clone()))
    }
}
/// Owner guard. Its destruction records interruption, never requests scientific cancellation.
#[derive(Debug)]
pub struct OperationGuard {
    context: ObservationContext,
}
impl OperationGuard {
    /// Clone the diagnostic producer context.
    #[must_use]
    pub fn context(&self) -> ObservationContext {
        self.context.clone()
    }
    /// Borrow independent copied diagnostics without affecting scientific ownership.
    #[must_use]
    pub fn reader(&self) -> OperationReader {
        self.context.reader()
    }
    /// Record an explicit terminal explanation; does not cancel or accept scientific work.
    pub fn finish(self, outcome: Outcome) {
        if let Some(node) = &self.context.node {
            node.finish(outcome);
            if let Some(hub) = self.context.hub.upgrade() {
                hub.retain_summary(node);
            }
        }
    }
}
impl Drop for OperationGuard {
    fn drop(&mut self) {
        if let Some(node) = &self.context.node
            && node.terminal.load(Ordering::Acquire) == 0
        {
            node.finish(Outcome::Interrupted);
            if let Some(hub) = self.context.hub.upgrade() {
                hub.retain_summary(node);
            }
        }
    }
}
/// An independently owned active phase; dropping it cannot overwrite another phase.
#[derive(Debug)]
pub struct PhaseGuard {
    context: ObservationContext,
    admitted: Option<Arc<AdmittedPhase>>,
}
#[derive(Debug)]
struct AdmittedPhase {
    phase: Phase,
    active: AtomicBool,
    // Shared current/guard storage remains charged until both owners release it.
    _reservation: Reservation,
}
impl Drop for PhaseGuard {
    fn drop(&mut self) {
        let (Some(node), Some(admitted)) = (&self.context.node, self.admitted.take()) else {
            return;
        };
        // An admitted scope has one owner. Neither a busy/poisoned optional lock nor
        // terminal publication may retain it as current work or consume its slot.
        let at = node.clock.now();
        admitted.active.store(false, Ordering::Release);
        node.update_at(Some(at), |state, at| {
            state
                .phases
                .retain(|phase| phase.active.load(Ordering::Acquire));
            let phase = &admitted.phase;
            let elapsed = at.saturating_sub(phase.started_at);
            state.detail(
                &node.policy,
                at,
                &phase.kind,
                Value::number(elapsed.as_secs_f64()),
            );
            if let Some(event) = state
                .detail
                .back_mut()
                .filter(|event| event.at == at && event.name == phase.kind)
            {
                event.phase = Some(PhaseTransition {
                    token: phase.token,
                    started_at: phase.started_at,
                    finished_at: Some(at),
                });
            }
            let name = format!("phase.{}.nanoseconds", phase.kind);
            state.producer_elapsed(&node.policy, self.context.producer, &name, at, elapsed);
        });
    }
}
thread_local! { static CURRENT: RefCell<Option<ObservationContext>> = const { RefCell::new(None) }; }
#[must_use]
/// Enter an independently owned elapsed phase scope; scope exit does not imply success.
pub fn phase(kind: impl AsRef<str>) -> Option<PhaseGuard> {
    current().map(|context| context.phase(kind))
}
/// Clone the synchronous thread-local context, if one has been entered.
#[must_use]
pub fn current() -> Option<ObservationContext> {
    CURRENT.with(|current| current.borrow().clone())
}
/// Poll a future under the supplied diagnostic context, restoring the caller on every poll.
/// An absent context explicitly clears unrelated ambient context for the duration of each poll.
pub fn scope<F: Future>(
    context: Option<ObservationContext>,
    future: F,
) -> impl Future<Output = F::Output> {
    Scoped { context, future }
}
pin_project_lite::pin_project! {
    // One pinned input, with no async argument/local duplication or heap allocation.
    struct Scoped<F> {
        context: Option<ObservationContext>,
        #[pin]
        future: F,
    }
}
impl<F: Future> Future for Scoped<F> {
    type Output = F::Output;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        let _entered = enter_optional(this.context.clone());
        this.future.poll(cx)
    }
}
/// Capture the current context now and restore it independently on each future poll.
pub fn inherit<F: Future>(future: F) -> impl Future<Output = F::Output> {
    let context = current();
    scope(context, future)
}
fn enter_optional(context: Option<ObservationContext>) -> EnterGuard {
    let previous = CURRENT.with(|current| current.replace(context));
    EnterGuard {
        previous,
        _not_send: PhantomData,
    }
}
/// Restores the enclosing synchronous context and cannot move to another thread.
#[derive(Debug)]
pub struct EnterGuard {
    previous: Option<ObservationContext>,
    _not_send: PhantomData<Rc<()>>,
}
impl Drop for EnterGuard {
    fn drop(&mut self) {
        CURRENT.with(|current| {
            current.replace(self.previous.take());
        });
    }
}

/// A read lease owns no solver, runtime, cancellation token or canonical-result permission.
#[derive(Clone, Debug)]
pub struct OperationReader {
    context: ObservationContext,
    compact: Option<Arc<RetainedSummary>>,
}
impl OperationReader {
    /// Return the operation diagnostic identity.
    #[must_use]
    pub fn id(&self) -> OperationId {
        self.context.id
    }
    #[must_use]
    /// Read a bounded copied snapshot; contended publication returns explicit absent coverage.
    pub fn snapshot(&self, interval: Duration) -> OperationSnapshot {
        if let Some(hub) = self.context.hub.upgrade() {
            hub.prune_summaries();
        }
        if let Some(compact) = &self.compact {
            let mut snapshot = compact.snapshot.clone();
            snapshot.read_at = compact.clock.now();
            snapshot.requested_interval = interval;
            snapshot.apply_summary_expiry(snapshot.read_at);
            return snapshot;
        }
        let Some(node) = &self.context.node else {
            return OperationSnapshot {
                summary: None,
                id: self.context.id,
                kind: String::new(),
                started_at: Duration::ZERO,
                finished_at: None,
                outcome: None,
                observed_at: None,
                read_at: Duration::ZERO,
                requested_interval: interval,
                stale_after: Duration::from_secs(5),
                interval_start: Duration::ZERO,
                interval_end: Duration::ZERO,
                interval_partial: true,
                detail_partial: true,
                coverage: Coverage {
                    enabled: self.context.absent_enabled,
                    curtailed: self.context.absent_enabled,
                    ..Coverage::default()
                },
                metrics: Vec::new(),
                producers: Vec::new(),
                phases: Vec::new(),
                children: Vec::new(),
                sources: Vec::new(),
                detail: Vec::new(),
            };
        };
        let now = node.clock.now();
        let summary = node.summary();
        let end = summary.finished_at.unwrap_or(now);
        let start = now.saturating_sub(interval).max(node.started_at).min(end);
        let retained_start = now.saturating_sub(WINDOW).max(node.started_at).min(end);
        let empty_intersection = now.saturating_sub(interval) > end;
        let mut result = OperationSnapshot {
            summary: summary.summary,
            id: node.id,
            kind: node.kind.clone(),
            started_at: node.started_at,
            finished_at: summary.finished_at,
            outcome: summary.outcome,
            observed_at: None,
            read_at: now,
            requested_interval: interval,
            stale_after: node.policy.stale_after,
            interval_start: start,
            interval_end: end,
            interval_partial: start < retained_start,
            detail_partial: false,
            coverage: Coverage {
                enabled: true,
                available: true,
                interrupted: summary.outcome == Some(Outcome::Interrupted),
                dropped_updates: node.lost.load(Ordering::Relaxed),
                ..Coverage::default()
            },
            metrics: Vec::new(),
            producers: Vec::new(),
            phases: Vec::new(),
            children: Vec::new(),
            sources: Vec::new(),
            detail: Vec::new(),
        };
        result.apply_summary_expiry(now);
        let Ok(state) = node.state.try_lock() else {
            result.coverage.available = false;
            result.coverage.curtailed = true;
            result.interval_partial = true;
            result.detail_partial = true;
            return result;
        };
        result.observed_at = state.last_observed;
        result.coverage.stale |= summary.outcome.is_none()
            && now.saturating_sub(state.last_observed.unwrap_or(node.started_at))
                > node.policy.stale_after;
        result.coverage.dropped_detail = state.dropped_detail;
        result.detail_partial = state.dropped_detail != 0 || result.coverage.dropped_updates != 0;
        result.coverage.refused_items = state.refused;
        result.coverage.counter_overflow = state.overflow
            || state.refused == u64::MAX
            || state.dropped_detail == u64::MAX
            || result.coverage.dropped_updates == u64::MAX;
        result.coverage.curtailed = state.refused != 0 || result.coverage.dropped_updates != 0;
        result.interval_partial |= result.coverage.dropped_updates != 0;
        // Reserve only the counter intervals actually copied by this read. Once returned,
        // snapshots are caller-owned memory; no empty 62-slot copy is charged per metric.
        let interval_count: usize = state
            .metrics
            .iter()
            .map(|metric| {
                metric
                    .buckets
                    .iter()
                    .filter(|bucket| bucket.second.is_some())
                    .count()
            })
            .sum();
        let counter_copy = state
            .budget
            .reserve(interval_count.saturating_mul(3 * size_of::<CounterInterval>()));
        if counter_copy.is_none() {
            result.coverage.curtailed = true;
            result.interval_partial = true;
        }
        result.metrics = state
            .metrics
            .iter()
            .map(|metric| {
                let (recent, partial, recent_observed_start, recent_observed_end) =
                    if empty_intersection && metric.counter {
                        (Value::Count(0), false, None, None)
                    } else {
                        metric.recent(start.max(retained_start), end)
                    };
                let history_start = start.max(retained_start);
                let predecessor = metric
                    .history
                    .iter()
                    .filter(|bucket| {
                        bucket.sample.last_observed_at < history_start
                            && bucket.sample.last_observed_at
                                >= history_start.saturating_sub(Duration::from_secs(1))
                    })
                    .map(|bucket| bucket.sample.last_observed_at)
                    .max();
                let history: Vec<_> = metric
                    .history
                    .iter()
                    .filter(|bucket| {
                        !empty_intersection
                            && (bucket.sample.last_observed_at >= history_start
                                || Some(bucket.sample.last_observed_at) == predecessor)
                            && bucket.sample.first_observed_at <= end
                    })
                    .map(|bucket| {
                        let mut sample = bucket.sample.clone();
                        sample.predecessor = sample.last_observed_at < start;
                        sample
                    })
                    .collect();
                let history_partial = metric.history_dropped != 0
                    || result.interval_partial
                    || history.iter().any(|sample| {
                        sample.samples > 1
                            || sample.reset
                            || sample.first_observed_at < start
                            || sample.last_observed_at > end
                    })
                    || history.windows(2).any(|pair| {
                        pair[1].first_observed_at.as_secs()
                            > pair[0].last_observed_at.as_secs().saturating_add(1)
                    })
                    || history
                        .first()
                        .is_some_and(|sample| sample.first_observed_at > start)
                    || history
                        .last()
                        .is_some_and(|sample| sample.last_observed_at < end)
                    || (!metric.history.is_empty() && history.is_empty() && !empty_intersection);
                MetricSnapshot {
                    producer: metric.producer,
                    duration_summary: metric.duration_summary.clone(),
                    name: metric.name.clone(),
                    unit: metric.unit.clone(),
                    latest: metric.latest.clone(),
                    cumulative: metric.cumulative.clone(),
                    recent,
                    counter_window: (metric.counter && counter_copy.is_some()).then(|| {
                        CounterWindow {
                            duration: metric.duration,
                            observed: metric.first_count_at.is_some(),
                            partial: result.coverage.dropped_updates != 0
                                || metric.buckets.iter().any(|bucket| {
                                    bucket.second.is_some()
                                        && (u64::try_from(bucket.first.as_nanos()).is_err()
                                            || u64::try_from(bucket.last.as_nanos()).is_err())
                                }),
                            intervals: metric
                                .buckets
                                .iter()
                                .filter(|bucket| bucket.second.is_some())
                                .filter_map(|bucket| {
                                    Some(CounterInterval {
                                        first_ns: u64::try_from(bucket.first.as_nanos()).ok()?,
                                        last_ns: u64::try_from(bucket.last.as_nanos()).ok()?,
                                        delta: bucket.delta,
                                        reset: bucket.reset,
                                    })
                                })
                                .collect(),
                        }
                    }),
                    recent_observed_start,
                    recent_observed_end,
                    observed_at: metric.observed_at,
                    generation: metric.generation,
                    resets: metric.resets,
                    cumulative_partial: metric.absolute.is_some()
                        || result.coverage.dropped_updates != 0,
                    recent_partial: partial || result.interval_partial,
                    history,
                    history_partial,
                    history_dropped: metric.history_dropped,
                }
            })
            .collect();
        result.coverage.sampled = result.detail_partial
            || result
                .metrics
                .iter()
                .any(|metric| metric.recent_partial || metric.history_partial);
        result.producers = state
            .producers
            .iter()
            .map(|producer| {
                producer.snapshot(
                    start,
                    end,
                    empty_intersection,
                    node.lost.load(Ordering::Relaxed),
                )
            })
            .collect();
        // Terminal operation state retires all current scopes, including guards that
        // outlive the operation. Optional end records are never synthesized here.
        if node.terminal.load(Ordering::Acquire) == 0 {
            result.phases = state
                .phases
                .iter()
                .filter(|phase| phase.active.load(Ordering::Acquire))
                .map(|phase| phase.phase.clone())
                .collect();
        }
        result.sources = state.sources.clone();
        result.children = state
            .links
            .iter()
            .map(|link| {
                let child = link
                    .node
                    .upgrade()
                    .map(|child| child.summary())
                    .or_else(|| {
                        let hub = state.hub.upgrade()?;
                        let reader = hub.reader(link.id)?;
                        reader
                            .context
                            .node
                            .as_ref()
                            .map(|node| node.summary())
                            .or_else(|| {
                                reader.compact.as_ref().map(|compact| {
                                    let snapshot = &compact.snapshot;
                                    OperationSummary {
                                        summary: snapshot.summary.clone(),
                                        id: snapshot.id,
                                        kind: snapshot.kind.clone(),
                                        started_at: snapshot.started_at,
                                        finished_at: snapshot.finished_at,
                                        outcome: snapshot.outcome,
                                    }
                                })
                            })
                    });
                if let Some(child) = child {
                    ChildSnapshot {
                        id: child.id,
                        kind: child.kind,
                        relationship: link.relationship,
                        observed_at: link.observed_at,
                        outcome: child.outcome,
                        started_at: child.started_at,
                        finished_at: child.finished_at,
                        available: true,
                    }
                } else {
                    ChildSnapshot {
                        id: link.id,
                        relationship: link.relationship,
                        observed_at: link.observed_at,
                        kind: link.kind.clone(),
                        outcome: None,
                        started_at: Duration::ZERO,
                        finished_at: None,
                        available: false,
                    }
                }
            })
            .collect();
        result.detail = state
            .detail
            .iter()
            .filter(|event| !empty_intersection && event.at >= start && event.at <= end)
            .cloned()
            .collect();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_text_limit_reports_effective_policy_and_absent_collection() {
        for (requested, effective) in [(0, 1), (37, 37), (1024, 256)] {
            let hub = ObservationHub::new(Policy {
                max_text_bytes: requested,
                ..Default::default()
            });
            let operation = hub.start("text-limit");
            assert_eq!(operation.context().text_limit_bytes(), Some(effective));
        }
        let disabled = ObservationHub::new(Policy {
            enabled: false,
            ..Default::default()
        });
        assert_eq!(
            disabled.start("disabled").context().text_limit_bytes(),
            None
        );
    }

    #[derive(Default)]
    struct TestClock(AtomicU64);
    impl Clock for TestClock {
        fn now(&self) -> Duration {
            Duration::from_millis(self.0.load(Ordering::Relaxed))
        }
    }
    impl TestClock {
        fn set(&self, milliseconds: u64) {
            self.0.store(milliseconds, Ordering::Relaxed);
        }
    }
    fn setup(policy: Policy) -> (Arc<ObservationHub>, Arc<TestClock>) {
        let clock = Arc::new(TestClock::default());
        (ObservationHub::with_clock(policy, clock.clone()), clock)
    }
    #[test]
    fn responsibility_survives_many_inner_attempts_as_terminal_compact_output() {
        let (hub, clock) = setup(Policy {
            max_operations: 8,
            ..Policy::default()
        });
        let parent = hub.start("composition");
        let operation = parent
            .context()
            .child("responsibility", Relationship::Nested);
        let id = operation.context().id();
        operation.context().declare_summary("producer purpose");
        for _ in 0..1000 {
            operation.context().count("trials", "count", 1);
            operation
                .context()
                .child("inner", Relationship::Nested)
                .finish(Outcome::Completed);
        }
        clock.set(1000);
        operation.finish(Outcome::Completed);
        for _ in 0..1000 {
            hub.start("inner later").finish(Outcome::Completed);
        }
        let snapshot = hub.reader(id).unwrap().snapshot(WINDOW);
        assert_eq!(snapshot.outcome, Some(Outcome::Completed));
        assert_eq!(snapshot.finished_at, Some(Duration::from_secs(1)));
        assert_eq!(metric(&snapshot, "trials").cumulative, Value::Count(1000));
        assert!(snapshot.detail.is_empty());
        assert!(
            snapshot
                .metrics
                .iter()
                .all(|metric| metric.history.is_empty())
        );
        assert_eq!(
            snapshot.summary.unwrap().expires_at,
            Some(Duration::from_secs(61))
        );
        assert!(
            parent
                .reader()
                .snapshot(WINDOW)
                .children
                .iter()
                .any(|child| child.id == id && child.available)
        );
        assert!(hub.retained_bytes() <= hub.policy().budget_bytes);
    }
    #[test]
    fn summary_expiry_removes_discovery_but_independent_reader_remains_charged() {
        let (hub, clock) = setup(Policy::default());
        let operation = hub.start("responsibility");
        let id = operation.context().id();
        operation.context().declare_summary("purpose");
        operation.finish(Outcome::Completed);
        let reader = hub.reader(id).unwrap();
        let charged = hub.retained_bytes();
        assert!(charged > 0);
        clock.set(60000);
        assert!(hub.reader(id).is_none());
        assert_eq!(hub.list(0, 128).summaries.expired, 1);
        assert_eq!(hub.retained_bytes(), charged);
        assert_eq!(reader.snapshot(WINDOW).outcome, Some(Outcome::Completed));
        drop(reader);
        assert_eq!(hub.retained_bytes(), 0);
    }
    #[test]
    fn history_heavy_summary_releases_optional_backing_buffers_before_charging_copy() {
        let (hub, clock) = setup(Policy::default());
        let operation = hub.start("history-heavy responsibility");
        let id = operation.context().id();
        operation
            .context()
            .declare_summary("current and cumulative facts");
        for second in 0..60 {
            clock.set(second * 1000);
            operation
                .context()
                .gauge("residual", "scaled", second as f64);
            drop(operation.context().phase("actual phase"));
        }
        let rich = operation.reader().snapshot(WINDOW);
        assert!(metric(&rich, "residual").history.len() >= 50);
        assert!(!rich.detail.is_empty());
        operation.finish(Outcome::Completed);
        let summaries = hub.summaries.lock().unwrap();
        let copied = &summaries
            .iter()
            .find(|summary| summary.snapshot.id == id)
            .unwrap()
            .snapshot;
        assert_eq!(copied.detail.capacity(), 0);
        assert_eq!(copied.phases.capacity(), 0);
        assert!(
            copied
                .metrics
                .iter()
                .all(|metric| metric.history.capacity() == 0 && metric.counter_window.is_none())
        );
        assert_eq!(metric(copied, "residual").latest, Value::Number(59.0));
    }
    #[test]
    fn semantic_summary_expiry_applies_to_rich_compact_and_transported_readers() {
        let (hub, clock) = setup(Policy::default());
        let operation = hub.start("responsibility");
        let id = operation.context().id();
        operation.context().declare_summary("owned facts");
        let rich = operation.reader();
        operation.context().count("actual.calls", "calls", 3);
        operation.finish(Outcome::Completed);
        let compact = hub.reader(id).unwrap();
        let frozen_rich = rich.snapshot(WINDOW);
        let frozen_compact = compact.snapshot(WINDOW);
        for (milliseconds, expired) in [(59_999, false), (60_000, true)] {
            clock.set(milliseconds);
            assert_eq!(rich.snapshot(WINDOW).coverage.stale, expired);
            assert_eq!(compact.snapshot(WINDOW).coverage.stale, expired);
            for frozen in [&frozen_rich, &frozen_compact] {
                let selected = crate::inspection::select_window(
                    frozen.clone(),
                    Duration::from_millis(milliseconds),
                    WINDOW,
                );
                assert_eq!(selected.coverage.stale, expired);
                assert_eq!(
                    metric(&selected, "actual.calls").cumulative,
                    Value::Count(3)
                );
            }
        }
        assert!(hub.reader(id).is_none());
    }
    #[test]
    fn active_admission_preempts_oldest_summary_with_explicit_loss() {
        let (hub, _) = setup(Policy {
            max_operations: 2,
            ..Policy::default()
        });
        let operation = hub.start("responsibility");
        let id = operation.context().id();
        operation.context().declare_summary("purpose");
        operation.finish(Outcome::Completed);
        let first = hub.start("active one");
        let second = hub.start("active two");
        assert!(first.context().is_active());
        assert!(second.context().is_active());
        assert!(hub.reader(id).is_none());
        assert_eq!(hub.list(0, 128).summaries.pressure_lost, 1);
    }
    #[test]
    fn summary_opt_out_and_copy_refusal_never_change_completion() {
        for policy in [
            Policy {
                summary_retention: Duration::ZERO,
                ..Policy::default()
            },
            Policy {
                budget_bytes: Policy::default().node_bytes(),
                ..Policy::default()
            },
        ] {
            let (hub, _) = setup(policy);
            let operation = hub.start("responsibility");
            let reader = operation.reader();
            operation.context().declare_summary("purpose");
            operation.finish(Outcome::Completed);
            assert_eq!(reader.snapshot(WINDOW).outcome, Some(Outcome::Completed));
            assert_eq!(hub.list(0, 128).summaries.refused, 1);
            assert!(hub.retained_bytes() <= hub.policy().budget_bytes);
        }
        let (hub, _) = setup(Policy {
            enabled: false,
            ..Policy::default()
        });
        let operation = hub.start("disabled");
        operation.context().declare_summary("purpose");
        operation.finish(Outcome::Completed);
        assert_eq!(hub.list(0, 128).summaries.refused, 1);
        assert_eq!(hub.retained_bytes(), 0);
    }
    fn metric<'a>(snapshot: &'a OperationSnapshot, name: &str) -> &'a MetricSnapshot {
        snapshot
            .metrics
            .iter()
            .find(|metric| metric.name == name)
            .unwrap()
    }

    #[test]
    fn late_sixty_second_window_keeps_cumulative_and_actual_timestamps() {
        let (hub, clock) = setup(Policy::default());
        let operation = hub.start("long preparation");
        let context = operation.context();
        for second in 0..=120 {
            clock.set(second * 1000);
            context.count("work", "calls", 1);
        }
        let snapshot = operation.reader().snapshot(WINDOW);
        assert_eq!(snapshot.interval_start, Duration::from_secs(60));
        assert_eq!(snapshot.interval_end, Duration::from_secs(120));
        assert_eq!(metric(&snapshot, "work").cumulative, Value::Count(121));
        assert_eq!(metric(&snapshot, "work").recent, Value::Count(61));
        assert!(!snapshot.interval_partial);
        assert!(!metric(&snapshot, "work").recent_partial);
        let longer = operation.reader().snapshot(Duration::from_secs(100));
        assert!(longer.interval_partial);
        assert!(metric(&longer, "work").recent_partial);
        operation.finish(Outcome::Completed);
    }
    #[test]
    fn bucket_boundary_reports_overlap_instead_of_fabricating_precision() {
        let (hub, clock) = setup(Policy::default());
        let operation = hub.start("boundary");
        clock.set(10_100);
        operation.context().count("work", "calls", 3);
        clock.set(10_900);
        operation.context().count("work", "calls", 4);
        clock.set(70_500);
        let snapshot = operation.reader().snapshot(WINDOW);
        assert_eq!(metric(&snapshot, "work").recent, Value::Count(7));
        assert!(metric(&snapshot, "work").recent_partial);
        assert_eq!(
            metric(&snapshot, "work").observed_at,
            Duration::from_millis(10_900)
        );
        assert_eq!(
            metric(&snapshot, "work").recent_observed_start,
            Some(Duration::from_millis(10_100))
        );
        assert_eq!(
            metric(&snapshot, "work").recent_observed_end,
            Some(Duration::from_millis(10_900))
        );
    }
    #[test]
    fn burst_detail_rolls_without_losing_current_or_cumulative_facts() {
        let (hub, clock) = setup(Policy::default());
        let operation = hub.start("bursty");
        for index in 0..1000 {
            clock.set(index);
            let context = operation.context();
            context.count("callbacks", "calls", 1);
            let _phase = context.phase("callback");
        }
        let snapshot = operation.reader().snapshot(WINDOW);
        assert_eq!(snapshot.detail.len(), 256);
        assert_eq!(snapshot.coverage.dropped_detail, 1744);
        assert_eq!(
            metric(&snapshot, "callbacks").cumulative,
            Value::Count(1000)
        );
        assert_eq!(metric(&snapshot, "callbacks").recent, Value::Count(1000));
        assert!(!snapshot.interval_partial);
        assert!(hub.retained_bytes() <= 16 * 1024 * 1024);
    }
    #[test]
    fn disabled_and_budget_refusal_are_absence_not_zero_work() {
        for policy in [
            Policy {
                enabled: false,
                ..Policy::default()
            },
            Policy {
                budget_bytes: 0,
                ..Policy::default()
            },
        ] {
            let enabled = policy.enabled;
            let (hub, _) = setup(policy);
            let operation = hub.start("scientific work still runs");
            operation.context().count("calls", "count", 100);
            let snapshot = operation.reader().snapshot(WINDOW);
            assert_eq!(snapshot.coverage.enabled, enabled);
            assert!(!snapshot.coverage.available);
            assert_eq!(snapshot.coverage.curtailed, enabled);
            assert!(snapshot.metrics.is_empty());
            assert_eq!(hub.retained_bytes(), 0);
        }
    }
    #[test]
    fn unavailable_nonfinite_exact_integer_and_overflow_remain_explicit() {
        let (hub, _) = setup(Policy::default());
        let operation = hub.start("values");
        let context = operation.context();
        context.gauge("residual", "scaled", f64::NAN);
        context.gauge("positive", "units", f64::INFINITY);
        context.gauge("negative", "units", f64::NEG_INFINITY);
        context.integer("native", "count", i64::MAX);
        context.boolean("reuse", true);
        context.text("status", "native stop");
        context.unavailable("cpu", "owner cannot attribute shared CPU");
        context.count("calls", "count", u64::MAX);
        context.count("calls", "count", 1);
        let snapshot = operation.reader().snapshot(WINDOW);
        assert_eq!(
            metric(&snapshot, "residual").latest,
            Value::NonFinite(NonFinite::Nan)
        );
        assert_eq!(
            metric(&snapshot, "positive").latest,
            Value::NonFinite(NonFinite::PositiveInfinity)
        );
        assert_eq!(
            metric(&snapshot, "negative").latest,
            Value::NonFinite(NonFinite::NegativeInfinity)
        );
        assert_eq!(metric(&snapshot, "native").latest, Value::Integer(i64::MAX));
        assert_eq!(metric(&snapshot, "reuse").latest, Value::Boolean(true));
        assert_eq!(
            metric(&snapshot, "status").latest,
            Value::Text("native stop".into())
        );
        assert!(matches!(
            metric(&snapshot, "cpu").latest,
            Value::Unavailable(_)
        ));
        assert_eq!(metric(&snapshot, "calls").cumulative, Value::Overflow);
        assert_eq!(metric(&snapshot, "calls").recent, Value::Overflow);
        assert!(snapshot.coverage.counter_overflow);
    }
    #[test]
    fn counter_resets_never_subtract_incompatible_generations() {
        let (hub, clock) = setup(Policy::default());
        let operation = hub.start("retry");
        let context = operation.context();
        context.count_value("native_calls", "count", 1000, 7);
        assert!(matches!(
            metric(&operation.reader().snapshot(WINDOW), "native_calls").recent,
            Value::Unavailable(_)
        ));
        clock.set(1000);
        context.count_value("native_calls", "count", 1005, 7);
        clock.set(2000);
        context.count_value("native_calls", "count", 2, 8);
        clock.set(3000);
        context.count_value("native_calls", "count", 5, 8);
        clock.set(4000);
        context.count_value("native_calls", "count", 1, 8);
        clock.set(5000);
        context.count_value("native_calls", "count", 4, 8);
        let snapshot = operation.reader().snapshot(WINDOW);
        let metric = metric(&snapshot, "native_calls");
        assert_eq!(metric.cumulative, Value::Count(11));
        assert_eq!(metric.latest, Value::Count(4));
        assert_eq!(metric.resets, 2);
        assert!(metric.recent_partial);
    }
    #[test]
    fn phases_end_independently_and_tls_restores_enclosing_context() {
        let (hub, clock) = setup(Policy::default());
        let parent = hub.start("parent");
        let child = parent.context().child("child", Relationship::Nested);
        assert!(current().is_none());
        let entered = parent.context().enter();
        let parent_phase = phase("parent phase").unwrap();
        {
            let _entered = child.context().enter();
            assert_eq!(current().unwrap().id(), child.context().id());
        }
        assert_eq!(current().unwrap().id(), parent.context().id());
        let another = parent.context().phase("other concurrent phase");
        clock.set(1234);
        drop(parent_phase);
        let snapshot = parent.reader().snapshot(WINDOW);
        assert_eq!(snapshot.phases.len(), 1);
        assert_eq!(snapshot.phases[0].kind, "other concurrent phase");
        assert!(
            snapshot
                .detail
                .iter()
                .any(|event| event.value == Value::Number(1.234))
        );
        drop(another);
        drop(entered);
        assert!(current().is_none());
    }

    #[test]
    fn phase_retirement_survives_held_history_lock_and_reuses_finite_capacity() {
        let (hub, clock) = setup(Policy {
            max_metrics: 1,
            ..Policy::default()
        });
        let operation = hub.start("contended phase");
        let context = operation.context();
        context.source_ref("source", b"actual source");
        let node = context.node.as_ref().unwrap();
        for index in 0..100 {
            let phase = context.phase("work");
            let admitted = Arc::clone(phase.admitted.as_ref().unwrap());
            let token = admitted.phase.token;
            let held = node.state.lock().unwrap();
            clock.set(index + 1);
            drop(phase);
            assert!(!admitted.active.load(Ordering::Acquire));
            drop(held);
            drop(admitted);
            let snapshot = operation.reader().snapshot(WINDOW);
            assert!(snapshot.phases.is_empty());
            assert_eq!(snapshot.coverage.dropped_updates, index + 1);
            assert!(snapshot.coverage.curtailed && snapshot.detail_partial);
            assert!(
                !snapshot
                    .detail
                    .iter()
                    .any(|event| event
                        .phase
                        .as_ref()
                        .is_some_and(|transition| transition.token == token
                            && transition.finished_at.is_some()))
            );
            assert!(snapshot.metrics.is_empty());
            assert_eq!(snapshot.sources[0].bytes, b"actual source");
        }
        assert_eq!(
            operation.reader().snapshot(WINDOW).coverage.refused_items,
            0
        );
        assert!(hub.retained_bytes() <= hub.policy().budget_bytes);
    }

    #[test]
    fn phase_retirement_reuses_capacity_despite_detail_overflow_and_metric_refusal() {
        for detail_capacity in [0, 1] {
            let (hub, clock) = setup(Policy {
                max_metrics: 1,
                detail_capacity,
                ..Policy::default()
            });
            let operation = hub.start("optional bounds");
            let context = operation.context();
            context.count("occupied", "calls", 1);
            let first = context.phase("work");
            let refused = context.phase("cannot admit");
            assert!(refused.admitted.is_none());
            drop(refused);
            clock.set(5);
            drop(first);
            let second = context.phase("work");
            assert!(second.admitted.is_some());
            clock.set(9);
            drop(second);
            let snapshot = operation.reader().snapshot(WINDOW);
            assert!(snapshot.phases.is_empty());
            assert_eq!(snapshot.coverage.refused_items, 3);
            assert_eq!(snapshot.coverage.dropped_updates, 0);
            assert_eq!(
                snapshot.coverage.dropped_detail,
                if detail_capacity == 0 { 4 } else { 3 }
            );
            assert_eq!(snapshot.metrics.len(), 1);
            if detail_capacity != 0 {
                let transition = snapshot.detail[0].phase.as_ref().unwrap();
                assert_eq!(transition.started_at, Duration::from_millis(5));
                assert_eq!(transition.finished_at, Some(Duration::from_millis(9)));
                assert_eq!(snapshot.detail[0].value, Value::Number(0.004));
            }
        }
    }

    #[test]
    fn phase_retirement_on_unwind_and_thread_exit_does_not_wait_for_history() {
        let (hub, _) = setup(Policy {
            max_metrics: 1,
            ..Policy::default()
        });
        let operation = hub.start("scope exits");
        let context = operation.context();
        let node = context.node.as_ref().unwrap();
        let phase = context.phase("unwind");
        let held = node.state.lock().unwrap();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let _phase = phase;
            panic!("scope interrupted");
        }));
        assert!(result.is_err());
        drop(held);
        let phase = context.phase("thread exit");
        assert!(phase.admitted.is_some());
        let held = node.state.lock().unwrap();
        std::thread::spawn(move || {
            let _phase = phase;
        })
        .join()
        .unwrap();
        drop(held);
        assert!(operation.reader().snapshot(WINDOW).phases.is_empty());
        let next = context.phase("reused");
        assert!(next.admitted.is_some());
        drop(next);
        assert_eq!(
            operation.reader().snapshot(WINDOW).coverage.dropped_updates,
            2
        );
    }

    #[test]
    fn phase_retirement_survives_poisoned_optional_state() {
        let (hub, _) = setup(Policy::default());
        let operation = hub.start("poisoned history");
        let context = operation.context();
        let phase = context.phase("work");
        let admitted = Arc::clone(phase.admitted.as_ref().unwrap());
        let node = context.node.as_ref().unwrap();
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _held = node.state.lock().unwrap();
            panic!("optional state poisoned");
        }));
        drop(phase);
        assert!(!admitted.active.load(Ordering::Acquire));
        assert_eq!(node.lost.load(Ordering::Relaxed), 1);
        let snapshot = operation.reader().snapshot(WINDOW);
        assert!(!snapshot.coverage.available);
        assert!(snapshot.coverage.curtailed);
        assert!(snapshot.phases.is_empty());
    }

    #[test]
    fn phase_retirement_releases_budget_for_next_admission_after_lost_exit() {
        let policy = Policy {
            max_metrics: 2,
            detail_capacity: 0,
            ..Policy::default()
        };
        let limit = policy.node_bytes() + policy.phase_bytes();
        let (hub, _) = setup(Policy {
            budget_bytes: limit,
            ..policy
        });
        let operation = hub.start("one reservation");
        let context = operation.context();
        let node = context.node.as_ref().unwrap();
        for _ in 0..20 {
            let phase = context.phase("admitted");
            assert!(phase.admitted.is_some());
            assert!(context.phase("budget refused").admitted.is_none());
            let held = node.state.lock().unwrap();
            drop(phase);
            drop(held);
            assert!(operation.reader().snapshot(WINDOW).phases.is_empty());
            assert!(hub.retained_bytes() <= limit);
        }
        let snapshot = operation.reader().snapshot(WINDOW);
        assert_eq!(snapshot.coverage.refused_items, 20);
        assert_eq!(snapshot.coverage.dropped_updates, 20);
    }

    #[test]
    fn delayed_phase_exit_keeps_actual_time_without_regressing_current_freshness() {
        use std::sync::Barrier;
        #[derive(Default)]
        struct DelayedClock {
            at: AtomicU64,
            pause: Mutex<Option<(Arc<Barrier>, Arc<Barrier>)>>,
        }
        impl Clock for DelayedClock {
            fn now(&self) -> Duration {
                let at = Duration::from_millis(self.at.load(Ordering::Relaxed));
                let pause = self.pause.lock().unwrap().take();
                if let Some((sampled, resume)) = pause {
                    sampled.wait();
                    resume.wait();
                }
                at
            }
        }
        for (older, newer, recent) in [(1, 2, 3_000_000), (1_000, 63_000, 60_000_000_000)] {
            let clock = Arc::new(DelayedClock::default());
            let hub = ObservationHub::with_clock(Policy::default(), clock.clone());
            let operation = hub.start("delayed exit");
            let slow = operation.context().phase("work");
            let fast = operation.context().phase("work");
            let sampled = Arc::new(Barrier::new(2));
            let resume = Arc::new(Barrier::new(2));
            clock.at.store(older, Ordering::Relaxed);
            *clock.pause.lock().unwrap() = Some((sampled.clone(), resume.clone()));
            let dropping = std::thread::spawn(move || drop(slow));
            sampled.wait();
            clock.at.store(newer, Ordering::Relaxed);
            drop(fast);
            resume.wait();
            dropping.join().unwrap();
            let snapshot = operation.reader().snapshot(WINDOW);
            assert!(snapshot.phases.is_empty());
            assert_eq!(snapshot.observed_at, Some(Duration::from_millis(newer)));
            let elapsed = metric(&snapshot, "phase.work.nanoseconds");
            assert_eq!(elapsed.observed_at, Duration::from_millis(newer));
            assert_eq!(
                elapsed.cumulative,
                Value::Count((older + newer) * 1_000_000)
            );
            assert_eq!(elapsed.recent, Value::Count(recent));
            let exits: Vec<_> = snapshot
                .detail
                .iter()
                .filter_map(|event| event.phase.as_ref()?.finished_at)
                .collect();
            let expected: Vec<_> = [newer, older]
                .into_iter()
                .filter(|at| *at >= newer.saturating_sub(60_000))
                .map(Duration::from_millis)
                .collect();
            assert_eq!(exits, expected);
            // The snapshot excludes actual endpoints outside its requested window;
            // storage still carries their original times, rather than a read-time repair.
            let context = operation.context();
            let state = context.node.as_ref().unwrap().state.lock().unwrap();
            let stored: Vec<_> = state
                .detail
                .iter()
                .filter_map(|event| event.phase.as_ref()?.finished_at)
                .collect();
            assert_eq!(
                stored,
                vec![Duration::from_millis(newer), Duration::from_millis(older)]
            );
        }
    }

    #[test]
    fn terminal_operation_retires_live_phase_without_fabricating_exit() {
        for outcome in [
            Outcome::Completed,
            Outcome::Failed,
            Outcome::Cancelled,
            Outcome::Interrupted,
        ] {
            let (hub, clock) = setup(Policy::default());
            let operation = hub.start("terminal phase");
            let context = operation.context();
            let phase = context.phase("still owned");
            let reader = operation.reader();
            let node = context.node.as_ref().unwrap();
            let held = node.state.lock().unwrap();
            clock.set(7);
            operation.finish(outcome);
            drop(held);
            let snapshot = reader.snapshot(WINDOW);
            assert_eq!(snapshot.outcome, Some(outcome));
            assert_eq!(snapshot.finished_at, Some(Duration::from_millis(7)));
            assert!(snapshot.phases.is_empty());
            assert_eq!(snapshot.detail.len(), 1);
            assert!(
                snapshot.detail[0]
                    .phase
                    .as_ref()
                    .unwrap()
                    .finished_at
                    .is_none()
            );
            clock.set(10);
            drop(phase);
            let snapshot = reader.snapshot(WINDOW);
            assert!(snapshot.phases.is_empty());
            assert_eq!(snapshot.coverage.dropped_updates, 1);
            assert_eq!(snapshot.detail.len(), 1);
        }
    }
    #[test]
    fn shared_producer_is_linked_twice_without_parent_counter_duplication() {
        let (hub, _) = setup(Policy::default());
        let waiter_one = hub.start("waiter one");
        let waiter_two = hub.start("waiter two");
        let producer = hub.start("shared producer");
        producer.context().count("construction", "count", 1);
        waiter_one
            .context()
            .link(&producer.context(), Relationship::SharedProduction);
        waiter_two
            .context()
            .link(&producer.context(), Relationship::SharedProduction);
        let producer_id = producer.context().id();
        producer.finish(Outcome::Completed);
        for waiter in [&waiter_one, &waiter_two] {
            let snapshot = waiter.reader().snapshot(WINDOW);
            assert_eq!(snapshot.children.len(), 1);
            assert_eq!(snapshot.children[0].id, producer_id);
            assert_eq!(snapshot.children[0].outcome, Some(Outcome::Completed));
            assert!(snapshot.metrics.is_empty());
        }
        waiter_one.finish(Outcome::Cancelled);
        assert_eq!(
            hub.reader(producer_id).unwrap().snapshot(WINDOW).outcome,
            Some(Outcome::Completed)
        );
    }
    #[test]
    fn readers_never_cancel_and_completed_retention_stays_bounded() {
        let (hub, _) = setup(Policy {
            max_operations: 2,
            ..Policy::default()
        });
        let operation = hub.start("live");
        let reader = operation.reader();
        drop(operation.reader());
        assert!(reader.snapshot(WINDOW).outcome.is_none());
        operation.finish(Outcome::Completed);
        let bytes = hub.retained_bytes();
        for _ in 0..20 {
            hub.start("completed").finish(Outcome::Completed);
        }
        assert_eq!(reader.snapshot(WINDOW).outcome, Some(Outcome::Completed));
        assert!(hub.list(0, usize::MAX).operations.len() <= 2);
        assert!(hub.retained_bytes() <= bytes * 3);
        assert!(hub.reader(reader.id()).is_none());
    }
    #[test]
    fn active_root_survives_history_pressure_and_eviction_never_rebinds_its_child() {
        let (hub, _) = setup(Policy {
            max_operations: 3,
            ..Policy::default()
        });
        let root = hub.start("study.root");
        let root_reader = root.reader();
        let child = root
            .context()
            .child("study.active.lane", Relationship::Nested);
        child.context().count("actual.requests", "calls", 3);
        for _ in 0..20 {
            hub.start("completed.history").finish(Outcome::Completed);
            let snapshot = hub.reader(root_reader.id()).unwrap().snapshot(WINDOW);
            assert!(snapshot.outcome.is_none());
            assert_eq!(snapshot.children.len(), 1);
            assert_eq!(snapshot.children[0].id, child.context().id());
            assert_eq!(hub.list(0, usize::MAX).operations.len(), 3);
        }
        root.finish(Outcome::Completed);
        let replacement = hub.start("study.replacement");
        assert_ne!(replacement.context().id(), root_reader.id());
        assert!(hub.reader(root_reader.id()).is_none());
        assert!(replacement.reader().snapshot(WINDOW).children.is_empty());
        let retained = root_reader.snapshot(WINDOW);
        assert_eq!(retained.id, root_reader.id());
        assert_eq!(retained.outcome, Some(Outcome::Completed));
        assert_eq!(retained.children[0].id, child.context().id());
        let active = hub.reader(child.context().id()).unwrap().snapshot(WINDOW);
        assert!(active.outcome.is_none());
        assert_eq!(
            metric(&active, "actual.requests").cumulative,
            Value::Count(3)
        );
    }
    #[test]
    fn producer_contention_is_loss_not_wait_and_terminal_survives() {
        let (hub, clock) = setup(Policy::default());
        let operation = hub.start("occupied");
        let reader = operation.reader();
        let node = operation.context.node.as_ref().unwrap();
        let held = node.state.lock().unwrap();
        operation.context().count("not retained", "calls", 1);
        assert_eq!(node.lost.load(Ordering::Relaxed), 1);
        let busy_read = reader.snapshot(WINDOW);
        assert!(!busy_read.coverage.available);
        assert!(busy_read.coverage.curtailed);
        drop(held);
        clock.set(10_000);
        let snapshot = reader.snapshot(WINDOW);
        assert_eq!(snapshot.coverage.dropped_updates, 1);
        assert!(snapshot.coverage.curtailed);
        assert!(snapshot.coverage.stale);
        assert!(snapshot.interval_partial);
        drop(operation);
        assert_eq!(reader.snapshot(WINDOW).outcome, Some(Outcome::Interrupted));
    }
    #[test]
    fn names_sources_links_and_active_operations_are_bounded() {
        let (hub, _) = setup(Policy {
            max_operations: 2,
            max_metrics: 1,
            max_children: 1,
            max_source_refs: 1,
            max_text_bytes: 8,
            ..Policy::default()
        });
        let operation = hub.start("a very long operation kind");
        let context = operation.context();
        context.count("first", "calls", 1);
        context.count("second", "calls", 1);
        context.count("first", "different", 1);
        context.text("first", "too long for cap");
        context.source_ref("first", [1, 2, 3]);
        context.source_ref("second", [4]);
        let child = context.child("child", Relationship::Nested);
        let refused = hub.start("refused");
        assert!(!refused.reader().snapshot(WINDOW).coverage.available);
        context.link_id(
            OperationId {
                process_instance: [9, 9],
                sequence: 1,
            },
            Relationship::Continuation,
        );
        let snapshot = operation.reader().snapshot(WINDOW);
        assert_eq!(snapshot.kind, "a very l");
        assert_eq!(snapshot.metrics.len(), 1);
        assert_eq!(snapshot.sources.len(), 1);
        assert_eq!(snapshot.children.len(), 1);
        assert!(snapshot.coverage.refused_items >= 5);
        assert!(snapshot.coverage.curtailed);
        assert_eq!(hub.list(0, 1).next_offset, Some(1));
        drop(child);
    }
    #[test]
    fn completed_late_read_does_not_rebase_requested_window_to_old_completion() {
        let (hub, clock) = setup(Policy::default());
        let operation = hub.start("short");
        clock.set(1000);
        operation.context().count("calls", "count", 4);
        let reader = operation.reader();
        operation.finish(Outcome::Completed);
        clock.set(120_000);
        let snapshot = reader.snapshot(WINDOW);
        assert_eq!(snapshot.interval_start, Duration::from_secs(1));
        assert_eq!(snapshot.interval_end, Duration::from_secs(1));
        // The empty intersection contains no work. The old completion's last minute is not substituted.
        assert_eq!(metric(&snapshot, "calls").recent, Value::Count(0));
        assert_eq!(metric(&snapshot, "calls").cumulative, Value::Count(4));
    }
    #[test]
    fn async_context_is_scoped_to_each_poll_and_absence_clears_ambient_state() {
        use std::future::Future;
        use std::task::{Context, Poll, Waker};
        struct Probe {
            expected: Option<OperationId>,
            polls: usize,
        }
        impl Future for Probe {
            type Output = ();
            fn poll(mut self: std::pin::Pin<&mut Self>, _: &mut Context<'_>) -> Poll<()> {
                assert_eq!(current().map(|context| context.id()), self.expected);
                self.polls += 1;
                if self.polls == 1 {
                    Poll::Pending
                } else {
                    Poll::Ready(())
                }
            }
        }
        let (hub, _) = setup(Policy::default());
        let first = hub.start("first");
        let second = hub.start("second");
        let mut first_future = Box::pin(scope(
            Some(first.context()),
            Probe {
                expected: Some(first.context().id()),
                polls: 0,
            },
        ));
        let mut second_future = {
            let _entered = second.context().enter();
            Box::pin(inherit(Probe {
                expected: Some(second.context().id()),
                polls: 0,
            }))
        };
        let mut cx = Context::from_waker(Waker::noop());
        assert!(first_future.as_mut().poll(&mut cx).is_pending());
        assert!(current().is_none());
        assert!(second_future.as_mut().poll(&mut cx).is_pending());
        assert!(current().is_none());
        let _ambient = first.context().enter();
        assert!(second_future.as_mut().poll(&mut cx).is_ready());
        assert_eq!(current().unwrap().id(), first.context().id());
        let mut absent = Box::pin(scope(
            None,
            Probe {
                expected: None,
                polls: 0,
            },
        ));
        assert!(absent.as_mut().poll(&mut cx).is_pending());
        assert_eq!(current().unwrap().id(), first.context().id());
        assert!(first_future.as_mut().poll(&mut cx).is_ready());
    }
    #[test]
    fn metric_interpretation_changes_are_refused_and_unavailable_can_become_observed() {
        let (hub, _) = setup(Policy::default());
        let operation = hub.start("meaning");
        let context = operation.context();
        context.count("delta", "calls", 2);
        context.gauge("delta", "calls", 9.0);
        context.count_value("delta", "calls", 100, 0);
        context.count_value("absolute", "calls", 100, 0);
        context.count("absolute", "calls", 2);
        context.gauge("gauge", "units", 8.0);
        context.count("gauge", "units", 1);
        context.unavailable("cpu", "no sample yet");
        context.gauge("cpu", "seconds", 1.5);
        let snapshot = operation.reader().snapshot(WINDOW);
        assert_eq!(metric(&snapshot, "delta").latest, Value::Count(2));
        assert_eq!(metric(&snapshot, "absolute").latest, Value::Count(100));
        assert_eq!(metric(&snapshot, "gauge").latest, Value::Number(8.0));
        assert_eq!(metric(&snapshot, "cpu").latest, Value::Number(1.5));
        assert_eq!(metric(&snapshot, "cpu").unit, "seconds");
        assert_eq!(snapshot.coverage.refused_items, 4);
    }

    #[test]
    fn long_phase_duration_is_split_and_overlapping_scopes_are_not_exclusive_time() {
        let (hub, clock) = setup(Policy::default());
        let operation = hub.start("duration");
        let first = operation.context().phase("work");
        clock.set(10_000);
        let overlap = operation.context().phase("work");
        clock.set(120_000);
        drop(first);
        drop(overlap);
        let snapshot = operation.reader().snapshot(WINDOW);
        let metric = metric(&snapshot, "phase.work.nanoseconds");
        assert_eq!(metric.cumulative, Value::Count(230_000_000_000));
        assert_eq!(metric.recent, Value::Count(120_000_000_000));
        assert!(!metric.recent_partial);
        assert_eq!(metric.unit, "nanoseconds");
        assert!(snapshot.phases.is_empty());
    }
    #[test]
    fn absolute_sample_interval_crossing_window_is_explicitly_partial() {
        let (hub, clock) = setup(Policy::default());
        let operation = hub.start("absolute interval");
        operation.context().count_value("samples", "count", 0, 0);
        clock.set(120_000);
        operation.context().count_value("samples", "count", 100, 0);
        let snapshot = operation.reader().snapshot(WINDOW);
        let metric = metric(&snapshot, "samples");
        assert_eq!(metric.recent, Value::Count(100));
        assert!(metric.recent_partial);
        assert!(metric.cumulative_partial);
    }

    #[test]
    fn competing_producers_account_for_loss_without_refusing_scientific_work() {
        let (hub, _) = setup(Policy::default());
        let operation = hub.start("concurrent");
        let barrier = Arc::new(std::sync::Barrier::new(4));
        let scientific_work = Arc::new(AtomicU64::new(0));
        std::thread::scope(|threads| {
            for _ in 0..4 {
                let context = operation.context();
                let barrier = barrier.clone();
                let scientific_work = scientific_work.clone();
                threads.spawn(move || {
                    barrier.wait();
                    for _ in 0..1000 {
                        scientific_work.fetch_add(1, Ordering::Relaxed);
                        context.count("work", "calls", 1);
                    }
                });
            }
        });
        assert_eq!(scientific_work.load(Ordering::Relaxed), 4000);
        let snapshot = operation.reader().snapshot(WINDOW);
        let Value::Count(retained) = metric(&snapshot, "work").cumulative else {
            panic!("exact retained count expected");
        };
        assert_eq!(retained + snapshot.coverage.dropped_updates, 4000);
        assert_eq!(
            snapshot.interval_partial,
            snapshot.coverage.dropped_updates != 0
        );
        fn require_send(_: impl Send) {}
        require_send(scope(Some(operation.context()), std::future::ready(())));
        require_send(inherit(std::future::ready(())));
    }

    #[test]
    fn default_budget_supports_sixteen_busy_lanes_with_root_fit_and_shared_work() {
        let (hub, clock) = setup(Policy::default());
        assert_eq!(hub.policy().max_metrics, 128);
        let root = hub.start("root");
        let fit = root.context().child("fit", Relationship::Nested);
        let shared = fit
            .context()
            .child("shared producer", Relationship::SharedProduction);
        let lanes: Vec<_> = (0..16)
            .map(|_| fit.context().child("fit lane", Relationship::Nested))
            .collect();
        for (operation_index, operation) in [&root, &fit, &shared]
            .into_iter()
            .chain(lanes.iter())
            .enumerate()
        {
            let context = operation.context();
            context.link(&shared.context(), Relationship::SharedProduction);
            for index in 0..63 {
                context.count(&format!("metric_{index}"), "calls", 1);
            }
            // Exercise a full rolling detail ring and its one duration metric on every node.
            for tick in 0..200 {
                clock.set(operation_index as u64 * 1000 + tick);
                drop(context.phase("work"));
            }
            context.source_ref("model", [7; 16]);
            let snapshot = operation.reader().snapshot(WINDOW);
            assert!(snapshot.coverage.available);
            assert!(!snapshot.coverage.curtailed);
            assert_eq!(snapshot.metrics.len(), 64);
            assert_eq!(snapshot.detail.len(), 256);
        }
        assert_eq!(hub.list(0, usize::MAX).operations.len(), 19);
        assert!(!hub.list(0, usize::MAX).coverage.curtailed);
        assert!(hub.retained_bytes() < 16 * 1024 * 1024);
        println!("19 busy nodes reserved {} bytes", hub.retained_bytes());
        // Idle operation structure does not reserve all possible metrics and histories.
        let idle_before = hub.retained_bytes();
        let idle = hub.start("idle child");
        assert!(idle.reader().snapshot(WINDOW).coverage.available);
        assert_eq!(
            hub.retained_bytes() - idle_before,
            hub.policy().node_bytes()
        );
    }
    #[test]
    fn lazy_field_refusal_and_completed_eviction_preserve_existing_facts() {
        let policy = Policy::default();
        let limit = 2 * policy.node_bytes() + policy.metric_bytes() + policy.aggregate_bytes();
        let (hub, _) = setup(Policy {
            budget_bytes: limit,
            ..policy
        });
        let completed = hub.start("completed");
        completed.context().count("old", "calls", 5);
        let completed_id = completed.context().id();
        completed.finish(Outcome::Completed);
        let live = hub.start("live");
        live.context().count("new", "calls", 1);
        assert!(hub.reader(completed_id).is_none());
        for index in 0..20 {
            live.context().count(&format!("more_{index}"), "calls", 1);
        }
        let snapshot = live.reader().snapshot(WINDOW);
        assert_eq!(metric(&snapshot, "new").cumulative, Value::Count(1));
        assert!(snapshot.coverage.curtailed);
        assert!(snapshot.coverage.refused_items > 0);
        assert!(hub.retained_bytes() <= limit);
    }

    #[test]
    fn detail_growth_refusal_reuses_charged_slot_for_latest_context() {
        let policy = Policy::default();
        let limit = policy.node_bytes() + policy.phase_bytes() + policy.detail_bytes();
        let (hub, _) = setup(Policy {
            budget_bytes: limit,
            ..policy
        });
        let operation = hub.start("curtailed details");
        drop(operation.context().phase("first"));
        drop(operation.context().phase("last"));
        let snapshot = operation.reader().snapshot(WINDOW);
        assert_eq!(snapshot.detail.len(), 1);
        assert_eq!(snapshot.detail[0].name, "last");
        assert!(snapshot.coverage.curtailed);
        assert_eq!(snapshot.coverage.dropped_detail, 3);
        assert!(hub.retained_bytes() <= limit);
    }

    #[test]
    fn unavailable_checkpoint_does_not_move_successful_absolute_sample_baseline() {
        let (hub, clock) = setup(Policy::default());
        let operation = hub.start("absolute sample availability");
        operation.context().count_value("samples", "count", 0, 0);
        clock.set(119_000);
        operation
            .context()
            .unavailable("samples", "owner could not sample");
        let unavailable = operation.reader().snapshot(WINDOW);
        assert_eq!(
            metric(&unavailable, "samples").observed_at,
            Duration::from_secs(119)
        );
        assert!(matches!(
            metric(&unavailable, "samples").latest,
            Value::Unavailable(_)
        ));
        clock.set(120_000);
        operation.context().count_value("samples", "count", 120, 0);
        let snapshot = operation.reader().snapshot(WINDOW);
        let metric = metric(&snapshot, "samples");
        assert_eq!(metric.cumulative, Value::Count(120));
        assert_eq!(metric.recent, Value::Count(120));
        assert_eq!(metric.recent_observed_start, Some(Duration::ZERO));
        assert_eq!(metric.recent_observed_end, Some(Duration::from_secs(120)));
        assert!(metric.recent_partial);
        assert!(metric.cumulative_partial);
    }

    #[test]
    fn checkpoint_history_distinguishes_changing_and_stagnant_residuals() {
        let (hub, clock) = setup(Policy::default());
        let operation = hub.start("convergence");
        let context = operation.context();
        for second in 0..=60 {
            clock.set(second * 1000 + 100);
            context.gauge("changing", "scaled", 100.0 - second as f64);
            context.gauge("stagnant", "scaled", 5.0);
        }
        let snapshot = operation.reader().snapshot(WINDOW);
        let changing = metric(&snapshot, "changing");
        let stagnant = metric(&snapshot, "stagnant");
        assert_eq!(changing.history.len(), 61);
        assert_eq!(
            changing.history.first().unwrap().value,
            Value::Number(100.0)
        );
        assert_eq!(changing.history.last().unwrap().value, Value::Number(40.0));
        assert!(
            stagnant
                .history
                .iter()
                .all(|sample| sample.first_value == Value::Number(5.0)
                    && sample.value == Value::Number(5.0))
        );
        assert_eq!(
            changing.history.first().unwrap().first_observed_at,
            Duration::from_millis(100)
        );
        assert_eq!(
            changing.history.last().unwrap().last_observed_at,
            Duration::from_millis(60_100)
        );
    }
    #[test]
    fn checkpoint_burst_coalesces_without_per_update_growth_and_preserves_endpoints() {
        let (hub, clock) = setup(Policy::default());
        let operation = hub.start("hot callback");
        let context = operation.context();
        clock.set(100);
        context.gauge("residual", "scaled", 1000.0);
        let bytes = hub.retained_bytes();
        let node = context.node.as_ref().unwrap();
        let pointer = node.state.lock().unwrap().metrics[0].history.as_ptr();
        for index in 1..1000 {
            clock.set(100 + index / 2);
            context.gauge("residual", "scaled", 1000.0 - index as f64);
        }
        assert_eq!(hub.retained_bytes(), bytes);
        assert_eq!(
            node.state.lock().unwrap().metrics[0].history.as_ptr(),
            pointer
        );
        let snapshot = operation.reader().snapshot(WINDOW);
        let residual = metric(&snapshot, "residual");
        assert_eq!(residual.history.len(), 1);
        assert_eq!(residual.history[0].samples, 1000);
        assert_eq!(residual.history[0].first_value, Value::Number(1000.0));
        assert_eq!(residual.history[0].value, Value::Number(1.0));
        assert_eq!(
            residual.history[0].first_observed_at,
            Duration::from_millis(100)
        );
        assert_eq!(
            residual.history[0].last_observed_at,
            Duration::from_millis(599)
        );
        assert!(residual.history_partial);
    }
    #[test]
    fn history_keeps_unavailable_nonfinite_exact_integers_and_resets_explicit() {
        let (hub, clock) = setup(Policy::default());
        let operation = hub.start("checkpoints");
        let context = operation.context();
        context.gauge("residual", "scaled", 5.0);
        clock.set(100);
        context.unavailable("residual", "owner busy");
        clock.set(200);
        context.gauge("residual", "scaled", f64::NAN);
        clock.set(300);
        context.gauge("residual", "scaled", 1.0);
        context.integer("native", "count", i64::MAX);
        context.boolean("reuse", true);
        context.text("status", "working");
        context.count_value("absolute", "calls", 100, 7);
        clock.set(400);
        context.count_value("absolute", "calls", 2, 8);
        let snapshot = operation.reader().snapshot(WINDOW);
        let residual = &metric(&snapshot, "residual").history[0];
        assert_eq!(residual.first_value, Value::Number(5.0));
        assert_eq!(residual.value, Value::Number(1.0));
        assert!(residual.unavailable);
        assert!(residual.nonfinite);
        assert_eq!(residual.samples, 4);
        assert_eq!(
            metric(&snapshot, "native").history[0].value,
            Value::Integer(i64::MAX)
        );
        assert_eq!(
            metric(&snapshot, "reuse").history[0].value,
            Value::Boolean(true)
        );
        assert_eq!(
            metric(&snapshot, "status").history[0].value,
            Value::Text("working".into())
        );
        let reset = &metric(&snapshot, "absolute").history[0];
        assert!(reset.reset);
        assert_eq!(reset.first_generation, 7);
        assert_eq!(reset.generation, 8);
        assert_eq!(reset.first_value, Value::Count(100));
        assert_eq!(reset.value, Value::Count(2));
        assert!(metric(&snapshot, "absolute").history_partial);
    }
    #[test]
    fn history_window_prunes_old_points_and_keeps_actual_predecessor() {
        let (hub, clock) = setup(Policy::default());
        let operation = hub.start("late inspection");
        let context = operation.context();
        for second in 0..=120 {
            clock.set(second * 1000 + 100);
            context.gauge("residual", "scaled", second as f64);
        }
        clock.set(120_500);
        let snapshot = operation.reader().snapshot(WINDOW);
        let history = &metric(&snapshot, "residual").history;
        assert_eq!(history.len(), 61);
        assert_eq!(history[0].last_observed_at, Duration::from_millis(60_100));
        assert!(history[0].predecessor);
        assert!(history.iter().skip(1).all(|sample| !sample.predecessor));
        assert!(
            history
                .iter()
                .all(|sample| sample.last_observed_at >= Duration::from_millis(59_500))
        );
        let state = context.node.as_ref().unwrap().state.lock().unwrap();
        assert_eq!(state.metrics[0].history.len(), BUCKETS);
        assert!(
            state.metrics[0]
                .history
                .iter()
                .all(|bucket| bucket.sample.last_observed_at.as_secs() >= 59)
        );
        drop(state);
        clock.set(300_000);
        let stale = operation.reader().snapshot(WINDOW);
        assert!(metric(&stale, "residual").history.is_empty());
        assert!(metric(&stale, "residual").history_partial);
        assert_eq!(metric(&stale, "residual").latest, Value::Number(120.0));
    }
    #[test]
    fn history_budget_refusal_keeps_latest_and_reuses_existing_numeric_slots() {
        let policy = Policy::default();
        let limit = policy.node_bytes() + policy.metric_bytes() + policy.history_capacity_bytes(4);
        let (hub, clock) = setup(Policy {
            budget_bytes: limit,
            ..policy
        });
        let operation = hub.start("history budget");
        for second in 0..10 {
            clock.set(second * 1000);
            operation
                .context()
                .gauge("residual", "scaled", second as f64);
        }
        let snapshot = operation.reader().snapshot(WINDOW);
        let residual = metric(&snapshot, "residual");
        assert_eq!(residual.latest, Value::Number(9.0));
        assert_eq!(residual.history.last().unwrap().value, Value::Number(9.0));
        assert_eq!(residual.history.len(), 4);
        assert!(residual.history_partial);
        assert_eq!(residual.history_dropped, 6);
        assert!(snapshot.coverage.curtailed);
        assert!(hub.retained_bytes() <= limit);
        operation
            .context()
            .unavailable("residual", "no string budget");
        let snapshot = operation.reader().snapshot(WINDOW);
        assert!(matches!(
            metric(&snapshot, "residual").latest,
            Value::Unavailable(_)
        ));
        assert_eq!(metric(&snapshot, "residual").history_dropped, 7);
    }
    #[test]
    fn sparse_text_history_reuses_string_reservations_after_pruning() {
        let (hub, clock) = setup(Policy::default());
        let operation = hub.start("sparse text");
        let context = operation.context();
        context.text("status", "first");
        let bytes = hub.retained_bytes();
        for index in 1..100 {
            clock.set(index * 120_000);
            context.text("status", "later");
        }
        assert_eq!(hub.retained_bytes(), bytes);
        let snapshot = operation.reader().snapshot(WINDOW);
        assert_eq!(metric(&snapshot, "status").history.len(), 1);
        assert_eq!(
            metric(&snapshot, "status").history[0].value,
            Value::Text("later".into())
        );
    }
    #[test]
    fn sixteen_lanes_keep_selected_gauge_histories_under_publication_adjusted_budget() {
        let (hub, clock) = setup(Policy {
            budget_bytes: 14 * 1024 * 1024 - 4096,
            ..Policy::default()
        });
        let root = hub.start("root");
        let fit = root.context().child("fit", Relationship::Nested);
        let shared = fit
            .context()
            .child("shared", Relationship::SharedProduction);
        let lanes: Vec<_> = (0..16)
            .map(|_| fit.context().child("lane", Relationship::Nested))
            .collect();
        for lane in &lanes {
            let context = lane.context();
            context.link(&shared.context(), Relationship::SharedProduction);
            for index in 0..24 {
                context.count(&format!("count_{index}"), "calls", 1);
            }
        }
        for second in 0..=120 {
            clock.set(second * 1000);
            for lane in &lanes {
                let context = lane.context();
                for index in 0..3 {
                    context.gauge(
                        &format!("residual_{index}"),
                        "scaled",
                        (120 - second) as f64,
                    );
                }
                drop(context.phase("callback"));
            }
        }
        for lane in &lanes {
            let snapshot = lane.reader().snapshot(WINDOW);
            assert!(!snapshot.coverage.curtailed);
            assert_eq!(metric(&snapshot, "residual_0").history.len(), 62);
        }
        assert_eq!(hub.list(0, usize::MAX).operations.len(), 19);
        assert!(hub.retained_bytes() <= 14 * 1024 * 1024 - 4096);
    }
}

impl std::fmt::Debug for ObservationHub {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ObservationHub")
            .field("policy", &self.policy)
            .field("process_instance", &self.process_instance)
            .field("retained_bytes", &self.retained_bytes())
            .finish_non_exhaustive()
    }
}
impl std::fmt::Debug for ObservationContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ObservationContext")
            .field("id", &self.id)
            .field("available", &self.node.is_some())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod scope_layout_tests {
    use super::*;
    #[test]
    fn observation_scope_stores_one_inline_pinned_future() {
        struct Inline {
            bytes: [u8; 64 * 1024],
            _pinned: std::marker::PhantomPinned,
        }
        impl Future for Inline {
            type Output = u8;
            fn poll(
                self: std::pin::Pin<&mut Self>,
                _: &mut std::task::Context<'_>,
            ) -> std::task::Poll<u8> {
                std::task::Poll::Ready(self.as_ref().get_ref().bytes[0])
            }
        }
        let future = scope(
            None,
            Inline {
                bytes: [7; 64 * 1024],
                _pinned: std::marker::PhantomPinned,
            },
        );
        assert!(
            size_of_val(&future) <= size_of::<Inline>() + 512,
            "context composition must not duplicate the inline input future"
        );
        let mut future = std::pin::pin!(future);
        let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
        assert_eq!(future.as_mut().poll(&mut cx), std::task::Poll::Ready(7));
    }
}
#[cfg(test)]
#[path = "observation/acceptance_tests.rs"]
mod acceptance_tests;
