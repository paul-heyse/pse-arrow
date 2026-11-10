// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Occurrence policy inputs and decisions, independent of execution and storage.
use crate::generated::enums::{CandidateUse, StudyPointState};
use crate::generated::identities::SolutionId;

/// Actual owner closure consumed by native database code generation.
/// # Errors
/// Conflicting source declarations refuse rather than select another meaning.
pub fn native_projection_graph() -> Result<
    pse_contract::boundary_projection::ProjectionGraph,
    pse_contract::boundary_projection::ProjectionError,
> {
    let mut graph = pse_contract::boundary_projection::ProjectionGraph::default();
    graph.register::<PointPolicy>()?;
    graph.register::<PointFacts>()?;
    graph.register::<PointAction>()?;
    graph.register::<PointOutcome>()?;
    graph.register::<SeedFact>()?;
    graph.register::<ReadinessPremises>()?;
    Ok(graph)
}

impl crate::HeapUsage for PointFacts {
    fn heap_bytes(&self) -> usize {
        let Self {
            admitted_seed_need: _,
            key: _,
            revision: _,
            lifecycle: _,
            native_started: _,
            scientific: _,
            attempt_count: _,
            retry_failure: _,
            effect: _,
            seed: _,
        } = self;
        0
    }
}
impl crate::HeapUsage for PointPolicy {
    fn heap_bytes(&self) -> usize {
        let Self {
            key: _,
            dependencies,
            seed_need: _,
            start: _,
            attempt_limit: _,
        } = self;
        dependencies
            .capacity()
            .saturating_mul(size_of::<Dependency>())
    }
}
impl crate::HeapUsage for PointAttemptOutcome {
    fn heap_bytes(&self) -> usize {
        let Self {
            attempt_id: _,
            lifecycle: _,
            diagnostic,
            scientific: _,
            start: _,
            effect: _,
        } = self;
        crate::HeapUsage::heap_bytes(diagnostic)
    }
}
impl crate::HeapUsage for PointOutcome {
    fn heap_bytes(&self) -> usize {
        let Self {
            key: _,
            lifecycle: _,
            scientific: _,
            diagnostic,
            start: _,
            effect: _,
            attempts,
        } = self;
        crate::HeapUsage::heap_bytes(diagnostic)
            .saturating_add(crate::HeapUsage::heap_bytes(attempts))
    }
}

pse_contract::projection_newtype! {
/// Position-independent identity within one study; equal bindings do not merge keys.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(transparent)]
pub struct OccurrenceKey(pub u32) {
    projection: { name: "StudyOccurrenceKey", version: 1, direction: pse_contract::boundary_projection::Direction::Both,
        reserved_tags: [], reserved_names: [] };
    value: { tag: 1, rules: [] };
}
}

/// Operation-owned output role selected explicitly for continuation.
pub use crate::generated::enums::StudySeedRole as SeedRole;

pse_contract::projection_enum! {
/// Dependencies whose meaning does not depend on seed availability.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(
    tag = "kind",
    content = "predecessor",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Dependency {
    projection: { name: "StudyDependency", version: 1, direction: pse_contract::boundary_projection::Direction::Both,
        reserved_tags: [], reserved_names: [] };
    /// Release after terminal completion, including failure or cancellation.
    Ordering(OccurrenceKey) => 1;
    /// Require the operation owner's aggregate scientific permission.
    UsableResult(OccurrenceKey) => 2;
}
}
impl Dependency {
    /// Referenced occurrence.
    pub const fn predecessor(self) -> OccurrenceKey {
        match self {
            Self::Ordering(key) | Self::UsableResult(key) => key,
        }
    }
}

pse_contract::projection_struct! {
/// Exact bounded readiness read-set, distinct from scientific permission.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadinessPremises {
    projection: { name: "StudyReadinessPremises", version: 1,
        direction: pse_contract::boundary_projection::Direction::Both,
        reserved_tags: [], reserved_names: [] };
    /// Study interpretation generation observed by the decision.
    pub generation: u64 => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Exact selected point identities and their original revisions.
    pub expected: Vec<(String,u64)> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::Length { min: 0, max: 65 }] };
}
}

/// Scientific permission demanded from a continuation predecessor.
pub use crate::generated::enums::StudyContinuationPermission as ContinuationPermission;
/// Operation-owned seed consumption vocabulary.
pub use crate::generated::enums::StudySeedNeed as SeedNeed;
/// Explicit fallback choice for an unavailable continuation seed.
pub use crate::generated::enums::StudyUnavailableSeedPolicy as UnavailableSeedPolicy;
pse_contract::projection_struct! {
/// One selected predecessor role; never an arbitrary first result.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct SeedEdge {
    projection: { name: "StudySeedEdge", version: 1, direction: pse_contract::boundary_projection::Direction::Both,
        reserved_tags: [], reserved_names: [] };
    /// Occurrence supplying the selected role.
    pub predecessor: OccurrenceKey => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Output role admitted by the operation owner.
    pub role: SeedRole => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Scientific predecessor requirement.
    #[serde(default)]
    pub permission: ContinuationPermission => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Explicit fallback choice.
    #[serde(default)]
    pub unavailable: UnavailableSeedPolicy => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
pse_contract::projection_enum! {
/// Declared start intent, separate from the chosen actual start.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum StartPolicy {
    projection: { name: "StudyStartPolicy", version: 1, direction: pse_contract::boundary_projection::Direction::Both,
        reserved_tags: [], reserved_names: [] };
    /// Operation-owned fresh start.
    Fresh => 1;
    /// Continue from one admitted predecessor role.
    Continuation(SeedEdge) => 2;
    /// Explicit supplied seed: incompatibility never falls back.
    Explicit {
        /// Role required from the supplied seed facts.
        role: SeedRole => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Immutable supplied artifact identity; facts cannot substitute another seed.
        seed: SolutionId => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 3;
}
}
pse_contract::projection_struct! {
/// Policy slice of an admitted operation occurrence.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct PointPolicy {
    projection: { name: "AdmittedStudyPointPolicy", version: 1, direction: pse_contract::boundary_projection::Direction::Both,
        reserved_tags: [], reserved_names: [] };
    /// Stable occurrence identity.
    pub key: OccurrenceKey => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Ordering and usable-result dependencies.
    pub dependencies: Vec<Dependency> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Seed consumption capability, admitted by the operation owner.
    pub seed_need: SeedNeed => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Authored start intent.
    pub start: StartPolicy => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Maximum attempts, including the initial attempt; timing belongs to the executor.
    pub attempt_limit: u32 => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::UnsignedRange { min: 1, max: u32::MAX as u64 }] };
}
}
pse_contract::projection_struct! {
/// Shared policy slice; immutable operation/binding payloads remain with their owners.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct OccurrenceGraph {
    projection: { name: "StudyOccurrenceGraph", version: 1, direction: pse_contract::boundary_projection::Direction::Both,
        reserved_tags: [], reserved_names: [] };
    /// Deterministic authored occurrence order.
    pub points: Vec<PointPolicy> => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
pse_contract::projection_enum! {
/// Compatibility is supplied by the operation owner for this consumer and selected role.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SeedAvailability {
    projection: { name: "StudySeedAvailability", version: 1, direction: pse_contract::boundary_projection::Direction::Both,
        reserved_tags: [], reserved_names: [] };
    /// Adapter has not acquired and validated the consumer seed yet.
    Unresolved => 1;
    /// Compatible seed artifact, resolved by the executor after the decision.
    Compatible {
        /// Artifact identity.
        seed: SolutionId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 2;
    /// No seed exists for the selected role.
    Absent => 3;
    /// A seed exists but its admitted contract does not match this consumer.
    Incompatible => 4;
    /// Acquisition/checking failed internally; fresh fallback is forbidden.
    InternalFailure => 5;
}
}
pse_contract::projection_struct! {
/// Selected role and its operation-owned seed facts.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct SeedFact {
    projection: { name: "StudySeedFact", version: 1, direction: pse_contract::boundary_projection::Direction::Both,
        reserved_tags: [], reserved_names: [] };
    /// Explicit output role.
    pub role: SeedRole => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Consumer-specific compatibility result.
    pub availability: SeedAvailability => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
pse_contract::projection_struct! {
/// E supplies aggregate decisions; the study never counts result tables to infer these.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ScientificFacts {
    projection: { name: "StudyScientificFacts", version: 1, direction: pse_contract::boundary_projection::Direction::Both,
        reserved_tags: [], reserved_names: [] };
    /// Aggregate usable-result permission from the owning operation.
    pub usable: bool => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Original scientific classification, when an assessment exists.
    pub candidate_use: Option<CandidateUse> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Independent seed permission from the owning operation.
    pub seed_permission: bool => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
/// Effect knowledge used for publication reconciliation and retry safety.
pub use crate::generated::enums::StudyEffectState as EffectState;
/// Purpose-specific retry knowledge, independently supplied by its operation owner.
pub use crate::generated::enums::StudyRetryFailure as RetryFailure;
pse_contract::projection_struct! {
/// Current operational and scientific facts for one occurrence.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct PointFacts {
    projection: { name: "StudyPointFacts", version: 1, direction: pse_contract::boundary_projection::Direction::Both,
        reserved_tags: [], reserved_names: [] };
    /// Selected lazy science has supplied its exact seed consumption capability.
    /// Absence preserves pending admission; this never grants start permission.
    #[serde(default)]
    pub admitted_seed_need: Option<SeedNeed> => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Occurrence identity.
    pub key: OccurrenceKey => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Revision an executor must revalidate under its locks before applying the action.
    pub revision: u64 => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Operational state, never a scientific decision.
    pub lifecycle: StudyPointState => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// A claimed worker may acquire inputs before native dispatch is admitted.
    pub native_started: bool => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Operation-owned aggregate assessment.
    pub scientific: ScientificFacts => { tag: 6, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Attempts already started.
    pub attempt_count: u32 => { tag: 7, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Explicit retry projection of the latest failure.
    pub retry_failure: Option<RetryFailure> => { tag: 8, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Publication/effect knowledge.
    pub effect: EffectState => { tag: 9, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Selected explicit or continuation seed compatibility for this consumer.
    /// A continuation's artifact originates in the predecessor but is checked for this point.
    pub seed: Option<SeedFact> => { tag: 10, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
/// Reasons allowed to produce an explicitly requested fresh fallback.
pub use crate::generated::enums::StudySeedUnavailable as SeedUnavailable;
pse_contract::projection_enum! {
/// Actual start selected by pure policy and retained by either executor.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum StartProvenance {
    projection: { name: "StudyStartProvenance", version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Declared fresh start.
    Fresh => 1;
    /// Constant/seed-free operation; no seed was requested.
    NotNeeded => 2;
    /// Selected predecessor supplied the compatible artifact.
    Continuation {
        /// Source occurrence.
        predecessor: OccurrenceKey => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Selected role.
        role: SeedRole => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Seed artifact.
        seed: SolutionId => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 3;
    /// An explicit supplied seed was compatible.
    Explicit {
        /// Selected role.
        role: SeedRole => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Seed artifact.
        seed: SolutionId => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 4;
    /// Explicit fresh fallback and its attributable absence/incompatibility.
    FreshFallback {
        /// Selected predecessor.
        predecessor: OccurrenceKey => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Selected role.
        role: SeedRole => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Reason a continuation seed could not be used.
        reason: SeedUnavailable => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 5;
}
}
pse_contract::projection_enum! {
/// Attributable policy refusal, separate from cancellation and operation diagnostics.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Refusal {
    projection: { name: "StudyRefusal", version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// A terminal predecessor did not grant required scientific permission.
    DependencyUnusable {
        /// Failed dependency.
        predecessor: OccurrenceKey => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 1;
    /// The continuation predecessor is neither usable nor explicitly seed-eligible.
    SeedPermission {
        /// Selected predecessor.
        predecessor: OccurrenceKey => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Selected seed output role.
        role: SeedRole => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 2;
    /// Selected seed absence or incompatibility, without fallback permission.
    SeedUnavailable {
        /// Continuation producer, absent for an explicit external seed.
        predecessor: Option<OccurrenceKey> => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Explicit selected role.
        role: SeedRole => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Typed unavailability.
        reason: SeedUnavailable => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 3;
    /// Seed acquisition/checking failed internally; never fresh fallback.
    SeedInternal {
        /// Continuation producer, absent for an explicit external seed.
        predecessor: Option<OccurrenceKey> => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Selected role.
        role: SeedRole => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 4;
}
}
pse_contract::projection_enum! {
/// Reason for waiting; terminal observations produce no new effect.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum WaitReason {
    projection: { name: "StudyWaitReason", version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Dependencies permit selected source/overlay admission, before any native claim.
    ScientificAdmission => 1;
    /// Queue admission grants acquisition only; no native start has been chosen.
    SeedResolution {
        /// Consumer input role whose compatibility remains unresolved.
        role: SeedRole => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 2;
    /// Earlier occurrence is not yet terminal.
    Dependency {
        /// Earlier occurrence.
        predecessor: OccurrenceKey => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 3;
    /// An attempt is executing.
    Assigned => 4;
    /// Occurrence has ended and no safe retry is available.
    Terminal => 5;
}
}
pse_contract::projection_enum! {
/// Effect-free requested action; adapters own fencing, locks and atomic application.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(
    tag = "kind",
    content = "detail",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ActionKind {
    projection: { name: "StudyActionKind", version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Preserve current state.
    Wait(WaitReason) => 1;
    /// Dispatch through the existing operation executor.
    Start(StartProvenance) => 2;
    /// Record the attributable dependent refusal.
    Refuse(Refusal) => 3;
    /// Study cancellation prevents dispatch and requests any active attempt to stop.
    Cancel => 4;
    /// Resolve unknown publication effects before any retry.
    Reconcile => 5;
}
}
pse_contract::projection_struct! {
/// One action derived from one exact occurrence revision.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct PointAction {
    projection: { name: "StudyPointAction", version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Target occurrence.
    pub occurrence: OccurrenceKey => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Snapshot revision to check inside the adapter's transaction/locks.
    pub expected_revision: u64 => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Shared scientific/operational choice.
    pub kind: ActionKind => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
/// Scientific availability, independent of operational lifecycle.
pub use crate::generated::enums::StudyAvailability as Availability;
/// Operational conclusion, independent of scientific availability.
pub use crate::generated::enums::StudyLifecycle;
/// Shared conclusion of the current facts, not a fabricated result table.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct Conclusion {
    /// Whole-study scientific availability.
    pub availability: Availability,
    /// Whole-study operational state.
    pub lifecycle: StudyLifecycle,
}
/// Deterministic pure transition result in authored occurrence order.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct StudyDecision {
    /// One action for every occurrence.
    pub actions: Vec<PointAction>,
    /// Current scientific and operational conclusion.
    pub conclusion: Conclusion,
}

pse_contract::projection_struct! {
/// One retained try; attempt identity is distinct from the requested occurrence.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PointAttemptOutcome {
    projection: { name: "StudyPointAttemptOutcome", version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Present for a durable attempt; ephemeral tries still retain their ordered history.
    pub attempt_id: Option<crate::generated::identities::AttemptId> => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Actual try lifecycle; absent when no operational attempt was made.
    pub lifecycle: Option<crate::generated::enums::AttemptState> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Original detailed diagnostic, including admission failures without result tables.
    pub diagnostic: Option<crate::diagnostic::BoundaryDiagnostic> => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Original operation-owned scientific permission.
    pub scientific: ScientificFacts => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Actual chosen start, if dispatch was attempted.
    pub start: Option<StartProvenance> => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Knowledge of effects for this try.
    pub effect: EffectState => { tag: 6, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
pse_contract::projection_struct! {
/// Shared outcome projection consumed by both executors and the durable store.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PointOutcome {
    projection: { name: "StudyPointOutcome", version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Requested occurrence, independent of preparation/binding content.
    pub key: OccurrenceKey => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Operational point state, independent of scientific availability.
    pub lifecycle: StudyPointState => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Latest operation-owned scientific permission.
    pub scientific: ScientificFacts => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Detailed latest refusal/failure, without fabricating a scientific result.
    pub diagnostic: Option<crate::diagnostic::BoundaryDiagnostic> => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Latest actual start provenance.
    pub start: Option<StartProvenance> => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Latest publication/effect knowledge.
    pub effect: EffectState => { tag: 6, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Every actual try, including failed admission and retry attempts.
    pub attempts: Vec<PointAttemptOutcome> => { tag: 7, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}

impl StartProvenance {
    /// Exhaustive relational projection of the actual selected start.
    pub const fn kind(&self) -> crate::generated::enums::StudyStartKind {
        use crate::generated::enums::StudyStartKind as K;
        match self {
            Self::Fresh => K::Fresh,
            Self::NotNeeded => K::NotNeeded,
            Self::Continuation { .. } => K::Continuation,
            Self::Explicit { .. } => K::Explicit,
            Self::FreshFallback { .. } => K::FreshFallback,
        }
    }
}

impl Refusal {
    /// One owned diagnostic projection used by both executor adapters.
    pub fn boundary_diagnostic(&self) -> crate::diagnostic::BoundaryDiagnostic {
        use crate::diagnostic::{BoundaryClass, BoundaryDiagnostic, Observation};
        use pse_diagnostics::{DiagnosticRule as R, DiagnosticStage};
        let (rule, kind) = match self {
            Self::DependencyUnusable { .. } => (R::StudyDependencyUnusable, "dependency_unusable"),
            Self::SeedPermission { .. } => (R::StudyDependencyUnusable, "seed_permission"),
            Self::SeedUnavailable {
                reason: SeedUnavailable::Absent,
                ..
            } => (R::StudySeedUnavailable, "seed_unavailable"),
            Self::SeedUnavailable {
                reason: SeedUnavailable::Incompatible,
                ..
            } => (R::StudySeedIncompatible, "seed_incompatible"),
            Self::SeedInternal { .. } => (R::StudySeedInternal, "seed_internal"),
        };
        let mut diagnostic = BoundaryDiagnostic::new(
            if matches!(self, Self::SeedInternal { .. }) {
                BoundaryClass::Internal
            } else {
                BoundaryClass::Conflict
            },
            DiagnosticStage::StudyPolicy,
            [],
            rule,
        );
        diagnostic
            .observations
            .insert("kind".into(), Observation::Text(kind.into()));
        match self {
            Self::DependencyUnusable { predecessor } => {
                diagnostic.observations.insert(
                    "predecessor".into(),
                    Observation::Integer(i64::from(predecessor.0)),
                );
            }
            Self::SeedPermission { predecessor, role } => {
                diagnostic.observations.insert(
                    "predecessor".into(),
                    Observation::Integer(i64::from(predecessor.0)),
                );
                diagnostic
                    .observations
                    .insert("role".into(), Observation::Text(role.as_str().into()));
            }
            Self::SeedUnavailable {
                predecessor,
                role,
                reason,
            } => {
                if let Some(predecessor) = predecessor {
                    diagnostic.observations.insert(
                        "predecessor".into(),
                        Observation::Integer(i64::from(predecessor.0)),
                    );
                }
                diagnostic
                    .observations
                    .insert("role".into(), Observation::Text(role.as_str().into()));
                diagnostic
                    .observations
                    .insert("reason".into(), Observation::Text(reason.as_str().into()));
            }
            Self::SeedInternal { predecessor, role } => {
                if let Some(predecessor) = predecessor {
                    diagnostic.observations.insert(
                        "predecessor".into(),
                        Observation::Integer(i64::from(predecessor.0)),
                    );
                }
                diagnostic
                    .observations
                    .insert("role".into(), Observation::Text(role.as_str().into()));
            }
        }
        diagnostic
    }
}
