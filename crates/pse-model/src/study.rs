// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Occurrence policy inputs and decisions, independent of execution and storage.
use crate::generated::enums::{CandidateUse, StudyPointState};
use crate::generated::identities::SolutionId;

/// Position-independent identity within one study; equal bindings do not merge keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct OccurrenceKey(pub u32);

/// Operation-owned output role selected explicitly for continuation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeedRole {
    /// Primal solution of a case solve.
    PrimalSolution,
    /// Shared parameters selected from an admitted fit.
    ParameterEstimates,
    /// A simulation's trajectory.
    Trajectory,
    /// An admitted horizon boundary state.
    HorizonState,
}

/// Dependencies whose meaning does not depend on seed availability.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "predecessor", rename_all = "snake_case")]
pub enum Dependency {
    /// Release after terminal completion, including failure or cancellation.
    Ordering(OccurrenceKey),
    /// Require the operation owner's aggregate scientific permission.
    UsableResult(OccurrenceKey),
}
impl Dependency {
    /// Referenced occurrence.
    pub const fn predecessor(self) -> OccurrenceKey {
        match self {
            Self::Ordering(key) | Self::UsableResult(key) => key,
        }
    }
}

/// Whether the selected operation can consume a start seed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeedNeed {
    /// Constant or seed-free evaluation; declared dependencies still apply.
    NotNeeded,
    /// The operation has a seed input, including a fresh operation-owned start.
    Required,
}
/// Scientific permission demanded from a continuation predecessor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContinuationPermission {
    /// Default: only a scientifically usable run may seed this occurrence.
    #[default]
    RequireUsable,
    /// Explicit choice: a seed-eligible run may supply a seed without usable results.
    AllowSeedOnly,
}
/// Choice when the selected continuation seed is absent or incompatible.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnavailableSeedPolicy {
    /// Default: retain an attributable refusal.
    #[default]
    Refuse,
    /// Explicitly permit a fresh start for absence or incompatibility only.
    FreshOnUnavailable,
}
/// One selected predecessor role; never an arbitrary first result.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SeedEdge {
    /// Occurrence supplying the selected role.
    pub predecessor: OccurrenceKey,
    /// Output role admitted by the operation owner.
    pub role: SeedRole,
    /// Scientific predecessor requirement.
    #[serde(default)]
    pub permission: ContinuationPermission,
    /// Explicit fallback choice.
    #[serde(default)]
    pub unavailable: UnavailableSeedPolicy,
}
/// Declared start intent, separate from the chosen actual start.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StartPolicy {
    /// Operation-owned fresh start.
    Fresh,
    /// Continue from one admitted predecessor role.
    Continuation(SeedEdge),
    /// Explicit supplied seed: incompatibility never falls back.
    Explicit {
        /// Role required from the supplied seed facts.
        role: SeedRole,
    },
}
/// Policy slice of an admitted operation occurrence.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PointPolicy {
    /// Stable occurrence identity.
    pub key: OccurrenceKey,
    /// Ordering and usable-result dependencies.
    pub dependencies: Vec<Dependency>,
    /// Seed consumption capability, admitted by the operation owner.
    pub seed_need: SeedNeed,
    /// Authored start intent.
    pub start: StartPolicy,
    /// Maximum attempts, including the initial attempt; timing belongs to the executor.
    pub attempt_limit: u32,
}
/// Shared policy slice; immutable operation/binding payloads remain with their owners.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OccurrenceGraph {
    /// Deterministic authored occurrence order.
    pub points: Vec<PointPolicy>,
}
/// Compatibility is supplied by the operation owner for this consumer and selected role.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SeedAvailability {
    /// Compatible seed artifact, resolved by the executor after the decision.
    Compatible {
        /// Artifact identity.
        seed: SolutionId,
    },
    /// No seed exists for the selected role.
    Absent,
    /// A seed exists but its admitted contract does not match this consumer.
    Incompatible,
    /// Acquisition/checking failed internally; fresh fallback is forbidden.
    InternalFailure,
}
/// Selected role and its operation-owned seed facts.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SeedFact {
    /// Explicit output role.
    pub role: SeedRole,
    /// Consumer-specific compatibility result.
    pub availability: SeedAvailability,
}
/// E supplies aggregate decisions; the study never counts result tables to infer these.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScientificFacts {
    /// Aggregate usable-result permission from the owning operation.
    pub usable: bool,
    /// Original scientific classification, when an assessment exists.
    pub candidate_use: Option<CandidateUse>,
    /// Independent seed permission from the owning operation.
    pub seed_permission: bool,
}
/// Effect knowledge used for publication reconciliation and retry safety.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectState {
    /// No effect took place.
    #[default]
    Absent,
    /// An effect took place, without a replay guarantee.
    Present,
    /// Publication acknowledgement is unresolved: reconcile before any retry.
    Unknown,
    /// An operation owner supplied a proven idempotent replay contract.
    Idempotent,
}
/// Purpose-specific retry projection; diagnostic class alone is not permission.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryFailure {
    /// Owner explicitly declared the failure transient.
    Transient,
    /// Deterministic encoding, admission or internal invariant defect.
    Deterministic,
}
/// Current operational and scientific facts for one occurrence.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PointFacts {
    /// Occurrence identity.
    pub key: OccurrenceKey,
    /// Revision an executor must revalidate under its locks before applying the action.
    pub revision: u64,
    /// Operational state, never a scientific decision.
    pub lifecycle: StudyPointState,
    /// Operation-owned aggregate assessment.
    pub scientific: ScientificFacts,
    /// Attempts already started.
    pub attempt_count: u32,
    /// Explicit retry projection of the latest failure.
    pub retry_failure: Option<RetryFailure>,
    /// Publication/effect knowledge.
    pub effect: EffectState,
    /// Selected explicit or continuation seed compatibility for this consumer.
    /// A continuation's artifact originates in the predecessor but is checked for this point.
    pub seed: Option<SeedFact>,
}
/// Reasons allowed to produce an explicitly requested fresh fallback.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeedUnavailable {
    /// Selected role has no seed.
    Absent,
    /// Seed and consumer contracts differ.
    Incompatible,
}
/// Actual start selected by pure policy and retained by either executor.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StartProvenance {
    /// Declared fresh start.
    Fresh,
    /// Constant/seed-free operation; no seed was requested.
    NotNeeded,
    /// Selected predecessor supplied the compatible artifact.
    Continuation {
        /// Source occurrence.
        predecessor: OccurrenceKey,
        /// Selected role.
        role: SeedRole,
        /// Seed artifact.
        seed: SolutionId,
    },
    /// An explicit supplied seed was compatible.
    Explicit {
        /// Selected role.
        role: SeedRole,
        /// Seed artifact.
        seed: SolutionId,
    },
    /// Explicit fresh fallback and its attributable absence/incompatibility.
    FreshFallback {
        /// Selected predecessor.
        predecessor: OccurrenceKey,
        /// Selected role.
        role: SeedRole,
        /// Reason a continuation seed could not be used.
        reason: SeedUnavailable,
    },
}
/// Attributable policy refusal, separate from cancellation and operation diagnostics.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Refusal {
    /// A terminal predecessor did not grant required scientific permission.
    DependencyUnusable {
        /// Failed dependency.
        predecessor: OccurrenceKey,
    },
    /// The continuation predecessor is neither usable nor explicitly seed-eligible.
    SeedPermission {
        /// Selected predecessor.
        predecessor: OccurrenceKey,
    },
    /// Selected seed absence or incompatibility, without fallback permission.
    SeedUnavailable {
        /// Explicit selected role.
        role: SeedRole,
        /// Typed unavailability.
        reason: SeedUnavailable,
    },
    /// Seed acquisition/checking failed internally; never fresh fallback.
    SeedInternal {
        /// Selected role.
        role: SeedRole,
    },
}
/// Reason for waiting; terminal observations produce no new effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WaitReason {
    /// Earlier occurrence is not yet terminal.
    Dependency {
        /// Earlier occurrence.
        predecessor: OccurrenceKey,
    },
    /// An attempt is executing.
    Assigned,
    /// Occurrence has ended and no safe retry is available.
    Terminal,
}
/// Effect-free requested action; adapters own fencing, locks and atomic application.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "detail", rename_all = "snake_case")]
pub enum ActionKind {
    /// Preserve current state.
    Wait(WaitReason),
    /// Dispatch through the existing operation executor.
    Start(StartProvenance),
    /// Record the attributable dependent refusal.
    Refuse(Refusal),
    /// Study cancellation prevents dispatch and requests any active attempt to stop.
    Cancel,
    /// Resolve unknown publication effects before any retry.
    Reconcile,
}
/// One action derived from one exact occurrence revision.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PointAction {
    /// Target occurrence.
    pub occurrence: OccurrenceKey,
    /// Snapshot revision to check inside the adapter's transaction/locks.
    pub expected_revision: u64,
    /// Shared scientific/operational choice.
    pub kind: ActionKind,
}
/// Scientific availability remains inspectable after operational cancellation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Availability {
    /// No occurrence grants aggregate usable-result permission.
    None,
    /// Some occurrences are usable, but the whole study is not scientifically complete.
    Partial,
    /// Every occurrence is terminal and scientifically usable.
    Complete,
}
/// Operational conclusion, independent of scientific availability.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StudyLifecycle {
    /// Work, retry or reconciliation remains.
    Active,
    /// All occurrences ended, without study cancellation.
    Terminal,
    /// Cancellation was requested, even if useful members remain available.
    Cancelled,
}
/// Shared conclusion of the current facts, not a fabricated result table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Conclusion {
    /// Whole-study scientific availability.
    pub availability: Availability,
    /// Whole-study operational state.
    pub lifecycle: StudyLifecycle,
}
/// Deterministic pure transition result in authored occurrence order.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StudyDecision {
    /// One action for every occurrence.
    pub actions: Vec<PointAction>,
    /// Current scientific and operational conclusion.
    pub conclusion: Conclusion,
}
