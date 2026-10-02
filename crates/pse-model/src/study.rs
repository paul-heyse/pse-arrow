// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Occurrence policy inputs and decisions, independent of execution and storage.
use crate::generated::enums::{CandidateUse, StudyPointState};
use crate::generated::identities::SolutionId;

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
pub struct OccurrenceKey(pub u32);

/// Operation-owned output role selected explicitly for continuation.
pub use crate::generated::enums::StudySeedRole as SeedRole;

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

/// Scientific permission demanded from a continuation predecessor.
pub use crate::generated::enums::StudyContinuationPermission as ContinuationPermission;
/// Operation-owned seed consumption vocabulary.
pub use crate::generated::enums::StudySeedNeed as SeedNeed;
/// Explicit fallback choice for an unavailable continuation seed.
pub use crate::generated::enums::StudyUnavailableSeedPolicy as UnavailableSeedPolicy;
impl Default for ContinuationPermission {
    fn default() -> Self {
        Self::RequireUsable
    }
}
impl Default for UnavailableSeedPolicy {
    fn default() -> Self {
        Self::Refuse
    }
}
/// One selected predecessor role; never an arbitrary first result.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
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
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum StartPolicy {
    /// Operation-owned fresh start.
    Fresh,
    /// Continue from one admitted predecessor role.
    Continuation(SeedEdge),
    /// Explicit supplied seed: incompatibility never falls back.
    Explicit {
        /// Role required from the supplied seed facts.
        role: SeedRole,
        /// Immutable supplied artifact identity; facts cannot substitute another seed.
        seed: SolutionId,
    },
}
/// Policy slice of an admitted operation occurrence.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
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
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct OccurrenceGraph {
    /// Deterministic authored occurrence order.
    pub points: Vec<PointPolicy>,
}
/// Compatibility is supplied by the operation owner for this consumer and selected role.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SeedAvailability {
    /// Adapter has not acquired and validated the consumer seed yet.
    Unresolved,
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
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct SeedFact {
    /// Explicit output role.
    pub role: SeedRole,
    /// Consumer-specific compatibility result.
    pub availability: SeedAvailability,
}
/// E supplies aggregate decisions; the study never counts result tables to infer these.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ScientificFacts {
    /// Aggregate usable-result permission from the owning operation.
    pub usable: bool,
    /// Original scientific classification, when an assessment exists.
    pub candidate_use: Option<CandidateUse>,
    /// Independent seed permission from the owning operation.
    pub seed_permission: bool,
}
/// Effect knowledge used for publication reconciliation and retry safety.
pub use crate::generated::enums::StudyEffectState as EffectState;
/// Purpose-specific retry knowledge, independently supplied by its operation owner.
pub use crate::generated::enums::StudyRetryFailure as RetryFailure;
/// Current operational and scientific facts for one occurrence.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct PointFacts {
    /// Occurrence identity.
    pub key: OccurrenceKey,
    /// Revision an executor must revalidate under its locks before applying the action.
    pub revision: u64,
    /// Operational state, never a scientific decision.
    pub lifecycle: StudyPointState,
    /// A claimed worker may acquire inputs before native dispatch is admitted.
    pub native_started: bool,
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
pub use crate::generated::enums::StudySeedUnavailable as SeedUnavailable;
/// Actual start selected by pure policy and retained by either executor.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
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
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
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
        /// Selected seed output role.
        role: SeedRole,
    },
    /// Selected seed absence or incompatibility, without fallback permission.
    SeedUnavailable {
        /// Continuation producer, absent for an explicit external seed.
        predecessor: Option<OccurrenceKey>,
        /// Explicit selected role.
        role: SeedRole,
        /// Typed unavailability.
        reason: SeedUnavailable,
    },
    /// Seed acquisition/checking failed internally; never fresh fallback.
    SeedInternal {
        /// Continuation producer, absent for an explicit external seed.
        predecessor: Option<OccurrenceKey>,
        /// Selected role.
        role: SeedRole,
    },
}
/// Reason for waiting; terminal observations produce no new effect.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum WaitReason {
    /// Queue admission grants acquisition only; no native start has been chosen.
    SeedResolution {
        /// Consumer input role whose compatibility remains unresolved.
        role: SeedRole,
    },
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
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct PointAction {
    /// Target occurrence.
    pub occurrence: OccurrenceKey,
    /// Snapshot revision to check inside the adapter's transaction/locks.
    pub expected_revision: u64,
    /// Shared scientific/operational choice.
    pub kind: ActionKind,
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

/// One retained try; attempt identity is distinct from the requested occurrence.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PointAttemptOutcome {
    /// Present for a durable attempt; ephemeral tries still retain their ordered history.
    pub attempt_id: Option<crate::generated::identities::AttemptId>,
    /// Actual try lifecycle; absent when no operational attempt was made.
    pub lifecycle: Option<crate::generated::enums::AttemptState>,
    /// Original detailed diagnostic, including admission failures without result tables.
    pub diagnostic: Option<crate::diagnostic::BoundaryDiagnostic>,
    /// Original operation-owned scientific permission.
    pub scientific: ScientificFacts,
    /// Actual chosen start, if dispatch was attempted.
    pub start: Option<StartProvenance>,
    /// Knowledge of effects for this try.
    pub effect: EffectState,
}
/// Shared outcome projection consumed by both executors and the durable store.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PointOutcome {
    /// Requested occurrence, independent of preparation/binding content.
    pub key: OccurrenceKey,
    /// Operational point state, independent of scientific availability.
    pub lifecycle: StudyPointState,
    /// Latest operation-owned scientific permission.
    pub scientific: ScientificFacts,
    /// Detailed latest refusal/failure, without fabricating a scientific result.
    pub diagnostic: Option<crate::diagnostic::BoundaryDiagnostic>,
    /// Latest actual start provenance.
    pub start: Option<StartProvenance>,
    /// Latest publication/effect knowledge.
    pub effect: EffectState,
    /// Every actual try, including failed admission and retry attempts.
    pub attempts: Vec<PointAttemptOutcome>,
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
