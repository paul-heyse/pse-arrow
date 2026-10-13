// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Governing choices over owner-issued facts, without native execution or error payloads.
use super::{Backend, StartOrigin, StartPolicy, StartRules, Transition};
use crate::generated::enums::{
    HessianMode, NativeAssessmentState as CandidateState,
    NumericalAttemptObservation as Observation,
};

/// Scientific/class/sensitivity priority precedes every equivalent-profile preference.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CandidatePreference {
    /// Lower values denote stronger original mathematical preference.
    pub priority: u32,
    /// Adapter or builder's deterministic baseline preference.
    pub baseline_rank: u32,
    /// Reliable synchronous evidence among scientifically equivalent candidates only.
    pub observed: Option<u32>,
    /// Applicable frozen advisory preference, after synchronous evidence.
    pub advisory: Option<u32>,
}
/// Readiness remains an owner-issued fact; missing evidence is never a refusal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PolicyCandidate {
    /// Whole contextual assessment, including settings/artifacts when applicable.
    pub state: CandidateState,
    /// Preference over candidates with equivalent original meaning.
    pub preference: CandidatePreference,
}
/// Stable inventory indices. Explicit selection and declared order cannot be reordered.
/// Pending evidence keeps its scientific baseline position even under observed/advised
/// ranking, so an unassessed candidate cannot be silently bypassed.
pub fn candidate_order(
    candidates: &[PolicyCandidate],
    explicit: Option<usize>,
    declared: bool,
) -> Vec<usize> {
    if let Some(index) = explicit {
        return (index < candidates.len())
            .then_some(index)
            .into_iter()
            .collect();
    }
    if declared {
        return (0..candidates.len()).collect();
    }
    let mut order = (0..candidates.len())
        .filter(|&index| candidates[index].state != CandidateState::Refused)
        .collect::<Vec<_>>();
    order.sort_by_key(|&index| {
        let p = candidates[index].preference;
        (p.priority, p.baseline_rank, index)
    });
    // Only rank fully interpreted runs between scientific evidence barriers. A lower
    // class never crosses a higher class, even if its numerical history looks better.
    let mut first = 0;
    while first < order.len() {
        let priority = candidates[order[first]].preference.priority;
        if candidates[order[first]].state == CandidateState::PendingEvidence {
            first += 1;
            continue;
        }
        let end = (first + 1..order.len())
            .find(|&position| {
                let c = candidates[order[position]];
                c.preference.priority != priority || c.state == CandidateState::PendingEvidence
            })
            .unwrap_or(order.len());
        order[first..end].sort_by_key(|&index| {
            let p = candidates[index].preference;
            (
                p.observed.unwrap_or(u32::MAX),
                p.advisory.unwrap_or(u32::MAX),
                p.baseline_rank,
                index,
            )
        });
        first = end;
    }
    order
}
/// Offer only successors of a still-current first candidate, stopping at missing evidence.
/// The operation owner still admits every returned candidate's actual artifacts/settings.
pub fn candidate_successors(candidates: &[PolicyCandidate], current: usize) -> Vec<usize> {
    let order = candidate_order(candidates, None, false);
    if order.first() != Some(&current) {
        return Vec::new();
    }
    order
        .into_iter()
        .skip(1)
        .take_while(|&index| candidates[index].state != CandidateState::PendingEvidence)
        .collect()
}
/// Entry/recovery permission shared by every automatic builder and driver.
pub fn permits_candidate_start(
    rules: &StartRules,
    recovery: &[StartOrigin],
    origin: StartOrigin,
    replacement: bool,
    inherited: bool,
) -> bool {
    if replacement {
        recovery.contains(&origin) && rules.permits_recovery(origin, inherited)
    } else {
        rules.permits_entry(origin, inherited)
    }
}
/// Projection of actual outcomes; rich original conclusions/errors stay with their owner.
#[derive(Clone, Copy, Debug)]
pub struct TransitionFacts {
    /// Actual native/operation observation, never an inferred success flag.
    pub observation: Observation,
    /// Original scientific owner's conclusion, separate from auxiliary convergence.
    pub original_satisfied: bool,
    /// Typed projection of the original refusal/unavailability cause, when present.
    pub original_cause: Option<Observation>,
    /// Auxiliary success does not establish original scientific meaning.
    pub auxiliary: bool,
    /// Original owner has screened a retained point for seed permission.
    pub seed_permitted: bool,
}
/// Terminal veto precedes original success, including when producers disagree.
pub const fn permits_numerical_continuation(observation: Observation) -> bool {
    matches!(
        observation,
        Observation::Converged
            | Observation::Auxiliary
            | Observation::Limited
            | Observation::Stalled
            | Observation::NumericalFailure
    )
}
impl TransitionFacts {
    /// A hard stop cannot be reversed by native convergence or a satisfied flag.
    pub fn terminal(self) -> bool {
        !permits_numerical_continuation(self.observation)
            || self
                .original_cause
                .is_some_and(|cause| !permits_numerical_continuation(cause))
    }
}
/// Shared transition meaning; the existing walker owns effects, clocks and aggregate work.
pub fn governing_transition(allowed: &[Transition], facts: TransitionFacts) -> Transition {
    if facts.terminal() {
        return Transition::Stop;
    }
    if facts.seed_permitted && facts.original_satisfied && !facts.auxiliary {
        return Transition::Finish;
    }
    let preferred = match facts.observation {
        Observation::Auxiliary if facts.auxiliary => Transition::Continue,
        Observation::NumericalFailure | Observation::Stalled => Transition::Recover,
        Observation::Limited => Transition::Continue,
        _ => Transition::Stop,
    };
    if allowed.contains(&preferred) {
        preferred
    } else if preferred == Transition::Recover && allowed.contains(&Transition::Subdivide) {
        Transition::Subdivide
    } else {
        Transition::Stop
    }
}
/// Curvature facts supplied by the original derivative/representation producer.
#[derive(Clone, Copy, Debug, Default)]
pub struct CurvatureFacts {
    /// Exact second derivatives are available for this producer's whole contract.
    pub exact: bool,
    /// The objective is least squares and its response Jacobian is supplied.
    pub least_squares_response: bool,
}
/// Concrete preference only. Explicit choices are preserved for subsequent native admission.
pub fn resolve_curvature(requested: HessianMode, facts: CurvatureFacts) -> HessianMode {
    if requested != HessianMode::Auto {
        requested
    } else if facts.least_squares_response {
        HessianMode::GaussNewton
    } else if facts.exact {
        HessianMode::Exact
    } else {
        HessianMode::LimitedMemory
    }
}
/// Original request facts used to permit automatic structural/supplier alternatives.
#[derive(Clone, Copy, Debug)]
pub struct AutomaticOperationFacts {
    /// Existing entry policy, never replaced by a successful auxiliary start.
    pub start: StartPolicy,
    /// Exact selected native backend, if any.
    pub backend: Option<Backend>,
    /// Selected representation admits original roots or NLP.
    pub roots_or_nlp: bool,
    /// Original task is a root problem.
    pub root_intent: bool,
    /// A sensitivity request imposes its own producer/coordinate contract.
    pub sensitivity: bool,
    /// Numerical convexity inference cannot be assumed exact by a structural alternative.
    pub exact_convexity: bool,
    /// Explicit pass declarations cannot silently move to a different formulation.
    pub explicit_presolve: bool,
    /// Typed backend settings are unmodified defaults.
    pub default_settings: bool,
    /// The supplied map has one worker's semantics.
    pub single_thread: bool,
}
/// A block schedule may be offered; matching/schedule construction remains compiler-owned.
pub fn permits_automatic_blocks(facts: AutomaticOperationFacts) -> bool {
    facts.start == StartPolicy::NoPriorStart
        && !facts.sensitivity
        && facts.exact_convexity
        && !facts.explicit_presolve
        && facts.roots_or_nlp
}
/// A retained raw-map supplier may be offered; its actual contract remains supplier-owned.
pub fn permits_automatic_supplier(facts: AutomaticOperationFacts) -> bool {
    facts.start == StartPolicy::NoPriorStart
        && facts.backend == Some(Backend::Kinsol)
        && !facts.sensitivity
        && facts.root_intent
        && facts.single_thread
        && facts.default_settings
}
/// Cheap owner-issued facts for demand-driven affine preprocessing evidence.
#[derive(Clone, Copy, Debug)]
pub struct AffinePreparationFacts {
    /// Automatic preprocessing policy grants the ordinary affine evidence request.
    pub automatic: bool,
    /// An enabled explicit pass consumes affine or redundant-row evidence.
    pub explicitly_requested: bool,
    /// An allowed native NLP representation could consume this evidence.
    pub nlp_possible: bool,
    /// The immutable original evidence is not yet complete.
    pub missing: bool,
}
/// Demand evidence before effects; native pass/options admissibility remains adapter-owned.
pub fn requires_affine_preparation(facts: AffinePreparationFacts) -> bool {
    (facts.automatic || facts.explicitly_requested) && facts.nlp_possible && facts.missing
}

#[cfg(test)]
mod tests {
    use super::*;
    fn candidate(state: CandidateState, priority: u32, baseline_rank: u32) -> PolicyCandidate {
        PolicyCandidate {
            state,
            preference: CandidatePreference {
                priority,
                baseline_rank,
                ..Default::default()
            },
        }
    }
    #[test]
    fn preference_keeps_science_explicit_order_and_pending_barriers() {
        let mut candidates = vec![
            candidate(CandidateState::Ready, 0, 0),
            candidate(CandidateState::Ready, 0, 1),
            candidate(CandidateState::Ready, 1, 0),
        ];
        candidates[1].preference.advisory = Some(0);
        candidates[2].preference.observed = Some(0);
        assert_eq!(candidate_order(&candidates, None, false), vec![1, 0, 2]);
        candidates[0].preference.observed = Some(0);
        assert_eq!(candidate_order(&candidates, None, false), vec![0, 1, 2]);
        assert_eq!(candidate_order(&candidates, Some(1), false), vec![1]);
        assert_eq!(candidate_order(&candidates, None, true), vec![0, 1, 2]);
        candidates[0].state = CandidateState::PendingEvidence;
        assert_eq!(candidate_order(&candidates, None, false), vec![0, 1, 2]);
        assert!(candidate_successors(&candidates, 1).is_empty());
        candidates[0].state = CandidateState::Refused;
        assert_eq!(candidate_order(&candidates, None, false), vec![1, 2]);
    }
    #[test]
    fn recovery_does_not_cross_pending_evidence_or_stale_selection() {
        let candidates = vec![
            candidate(CandidateState::Refused, 0, 0),
            candidate(CandidateState::Ready, 0, 1),
            candidate(CandidateState::SupportedPendingArtifacts, 0, 2),
            candidate(CandidateState::PendingEvidence, 0, 3),
            candidate(CandidateState::Ready, 0, 4),
        ];
        assert_eq!(candidate_successors(&candidates, 1), vec![2]);
        assert!(candidate_successors(&candidates, 2).is_empty());
    }
    #[test]
    fn every_hard_terminal_dominates_satisfied_and_recovery() {
        let allowed = [
            Transition::Finish,
            Transition::Recover,
            Transition::Continue,
            Transition::Subdivide,
            Transition::Stop,
        ];
        for observation in Observation::ALL {
            let facts = TransitionFacts {
                observation,
                original_satisfied: true,
                original_cause: None,
                auxiliary: false,
                seed_permitted: true,
            };
            assert_eq!(
                governing_transition(&allowed, facts),
                if permits_numerical_continuation(observation) {
                    Transition::Finish
                } else {
                    Transition::Stop
                }
            );
            assert_eq!(
                governing_transition(
                    &allowed,
                    TransitionFacts {
                        observation: Observation::Converged,
                        original_cause: Some(observation),
                        ..facts
                    }
                ),
                if permits_numerical_continuation(observation) {
                    Transition::Finish
                } else {
                    Transition::Stop
                }
            );
        }
        assert_eq!(
            governing_transition(
                &allowed,
                TransitionFacts {
                    observation: Observation::Converged,
                    original_satisfied: false,
                    original_cause: None,
                    auxiliary: false,
                    seed_permitted: true
                }
            ),
            Transition::Stop
        );
    }
    #[test]
    fn automatic_curvature_uses_whole_producer_facts_and_preserves_explicit() {
        assert_eq!(
            resolve_curvature(HessianMode::Auto, CurvatureFacts::default()),
            HessianMode::LimitedMemory
        );
        assert_eq!(
            resolve_curvature(
                HessianMode::Auto,
                CurvatureFacts {
                    exact: true,
                    least_squares_response: false
                }
            ),
            HessianMode::Exact
        );
        assert_eq!(
            resolve_curvature(
                HessianMode::Auto,
                CurvatureFacts {
                    exact: true,
                    least_squares_response: true
                }
            ),
            HessianMode::GaussNewton
        );
        assert_eq!(
            resolve_curvature(HessianMode::Partitioned, CurvatureFacts::default()),
            HessianMode::Partitioned
        );
    }
    #[test]
    fn structural_alternatives_preserve_original_entry_and_request_permissions() {
        let facts = AutomaticOperationFacts {
            start: StartPolicy::NoPriorStart,
            backend: Some(Backend::Kinsol),
            roots_or_nlp: true,
            root_intent: true,
            sensitivity: false,
            exact_convexity: true,
            explicit_presolve: false,
            default_settings: true,
            single_thread: true,
        };
        assert!(permits_automatic_blocks(facts));
        assert!(permits_automatic_supplier(facts));
        assert!(!permits_automatic_blocks(AutomaticOperationFacts {
            start: StartPolicy::Explicit,
            ..facts
        }));
        assert!(!permits_automatic_blocks(AutomaticOperationFacts {
            explicit_presolve: true,
            ..facts
        }));
        assert!(!permits_automatic_blocks(AutomaticOperationFacts {
            exact_convexity: false,
            ..facts
        }));
        assert!(!permits_automatic_supplier(AutomaticOperationFacts {
            root_intent: false,
            ..facts
        }));
        assert!(!permits_automatic_supplier(AutomaticOperationFacts {
            default_settings: false,
            ..facts
        }));
        assert!(!permits_automatic_supplier(AutomaticOperationFacts {
            single_thread: false,
            ..facts
        }));
        let rules = StartRules {
            policy: StartPolicy::NoPriorStart,
            recovery: vec![StartOrigin::Auxiliary],
        };
        assert!(!permits_candidate_start(
            &rules,
            &[],
            StartOrigin::Auxiliary,
            true,
            false
        ));
        assert!(permits_candidate_start(
            &rules,
            &rules.recovery,
            StartOrigin::Auxiliary,
            true,
            false
        ));
        assert!(!permits_candidate_start(
            &rules,
            &rules.recovery,
            StartOrigin::Auxiliary,
            true,
            true
        ));
        let explicit = StartRules {
            policy: StartPolicy::Explicit,
            recovery: vec![StartOrigin::Auxiliary],
        };
        assert!(!permits_candidate_start(
            &explicit,
            &[],
            StartOrigin::Specification,
            false,
            false
        ));
        assert!(permits_candidate_start(
            &explicit,
            &[],
            StartOrigin::Explicit,
            false,
            true
        ));
    }
    #[test]
    fn affine_evidence_is_demanded_only_for_a_consuming_representation() {
        let facts = AffinePreparationFacts {
            automatic: true,
            explicitly_requested: false,
            nlp_possible: true,
            missing: true,
        };
        assert!(requires_affine_preparation(facts));
        assert!(!requires_affine_preparation(AffinePreparationFacts {
            nlp_possible: false,
            ..facts
        }));
        assert!(!requires_affine_preparation(AffinePreparationFacts {
            missing: false,
            ..facts
        }));
        assert!(!requires_affine_preparation(AffinePreparationFacts {
            automatic: false,
            ..facts
        }));
        assert!(requires_affine_preparation(AffinePreparationFacts {
            automatic: false,
            explicitly_requested: true,
            ..facts
        }));
    }
}
