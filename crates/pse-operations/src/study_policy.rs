// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Shared, effect-free study admission and transition. Adapters own all I/O and fencing.
use petgraph::algo::is_cyclic_directed;
use petgraph::graphmap::DiGraphMap;
use pse_model::generated::enums::StudyPointState;
use pse_model::study::{
    ActionKind, Availability, Conclusion, ContinuationPermission, Dependency, EffectState,
    OccurrenceGraph, OccurrenceKey, PointAction, PointFacts, PointPolicy, Refusal, RetryFailure,
    SeedAvailability, SeedNeed, SeedRole, SeedUnavailable, StartPolicy, StartProvenance,
    StudyDecision, StudyLifecycle, UnavailableSeedPolicy, WaitReason,
};
use std::collections::{BTreeMap, BTreeSet};

/// Malformed policy/fact inputs; semantic dependent refusals remain returned actions.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PolicyError {
    /// A study must contain a requested occurrence.
    #[error("study has no occurrences")]
    Empty,
    /// Occurrence identity, never bindings, must be unique.
    #[error("duplicate study occurrence {key:?}")]
    DuplicateOccurrence {
        /// Repeated identity.
        key: OccurrenceKey,
    },
    /// Dependencies must refer to admitted occurrences.
    #[error("occurrence {key:?} depends on unknown occurrence {predecessor:?}")]
    UnknownDependency {
        /// Dependent occurrence.
        key: OccurrenceKey,
        /// Missing predecessor.
        predecessor: OccurrenceKey,
    },
    /// Duplicate edge declarations are refused instead of silently normalized.
    #[error("occurrence {key:?} repeats a dependency")]
    DuplicateDependency {
        /// Repeated edge's occurrence.
        key: OccurrenceKey,
    },
    /// The complete ordering/usability/seed dependency graph must be acyclic.
    #[error("study dependency graph is cyclic")]
    Cyclic,
    /// An attempt limit includes the initial execution.
    #[error("occurrence {key:?} has zero permitted attempts")]
    NoAttempts {
        /// Invalid occurrence.
        key: OccurrenceKey,
    },
    /// Constant operations cannot consume a supplied seed.
    #[error("seed-free occurrence {key:?} supplies an explicit seed")]
    SeedNotConsumed {
        /// Invalid occurrence.
        key: OccurrenceKey,
    },
    /// A snapshot must have one exact fact record per admitted occurrence.
    #[error("missing facts for occurrence {key:?}")]
    MissingFacts {
        /// Missing occurrence.
        key: OccurrenceKey,
    },
    /// Unadmitted fact identities cannot participate in a conclusion.
    #[error("facts name unknown occurrence {key:?}")]
    UnknownFacts {
        /// Unadmitted occurrence.
        key: OccurrenceKey,
    },
}
pse_diagnostics::impl_diagnostic! {
    PolicyError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::StudyPolicyAdmission) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None },
    facts(this) {
        use pse_diagnostics::{DiagnosticFacts, DiagnosticObservation, DiagnosticRule};
        let mut facts = DiagnosticFacts { rule: Some(DiagnosticRule::StudyPolicyAdmission), ..DiagnosticFacts::default() };
        let (kind, key) = match this {
            PolicyError::Empty => ("empty", None),
            PolicyError::DuplicateOccurrence { key } => ("duplicate_occurrence", Some(*key)),
            PolicyError::UnknownDependency { key, predecessor } => {
                facts.observe("predecessor", DiagnosticObservation::Integer(i64::from(predecessor.0)));
                ("unknown_dependency", Some(*key))
            }
            PolicyError::DuplicateDependency { key } => ("duplicate_dependency", Some(*key)),
            PolicyError::Cyclic => ("cyclic", None),
            PolicyError::NoAttempts { key } => ("no_attempts", Some(*key)),
            PolicyError::SeedNotConsumed { key } => ("seed_not_consumed", Some(*key)),
            PolicyError::MissingFacts { key } => ("missing_facts", Some(*key)),
            PolicyError::UnknownFacts { key } => ("unknown_facts", Some(*key)),
        };
        facts.observe("kind", DiagnosticObservation::Text(kind.to_owned()));
        if let Some(key) = key { facts.observe("occurrence", DiagnosticObservation::Integer(i64::from(key.0))); }
        facts
    }
}
impl pse_model::diagnostic::DiagnosticProjection for PolicyError {}

/// Check occurrence identity and all dependency meanings before scheduling.
/// Operation owners admit descriptor/output/input compatibility before deriving this graph.
/// # Errors
/// Refuses duplicate identities/edges, missing references, cycles and malformed start policy.
pub fn admit(graph: &OccurrenceGraph) -> Result<(), PolicyError> {
    if graph.points.is_empty() {
        return Err(PolicyError::Empty);
    }
    let mut dag = DiGraphMap::<OccurrenceKey, ()>::new();
    for point in &graph.points {
        if dag.contains_node(point.key) {
            return Err(PolicyError::DuplicateOccurrence { key: point.key });
        }
        dag.add_node(point.key);
        if point.attempt_limit == 0 {
            return Err(PolicyError::NoAttempts { key: point.key });
        }
        if point.seed_need == SeedNeed::NotNeeded
            && matches!(point.start, StartPolicy::Explicit { .. })
        {
            return Err(PolicyError::SeedNotConsumed { key: point.key });
        }
    }
    for point in &graph.points {
        let mut seen = BTreeSet::new();
        for dependency in &point.dependencies {
            let predecessor = dependency.predecessor();
            let kind = matches!(dependency, Dependency::UsableResult(_));
            if !seen.insert((predecessor, kind)) {
                return Err(PolicyError::DuplicateDependency { key: point.key });
            }
            add_dependency(&mut dag, point.key, predecessor)?;
        }
        if let StartPolicy::Continuation(edge) = &point.start {
            add_dependency(&mut dag, point.key, edge.predecessor)?;
        }
    }
    if is_cyclic_directed(&dag) {
        return Err(PolicyError::Cyclic);
    }
    Ok(())
}

fn add_dependency(
    dag: &mut DiGraphMap<OccurrenceKey, ()>,
    key: OccurrenceKey,
    predecessor: OccurrenceKey,
) -> Result<(), PolicyError> {
    if !dag.contains_node(predecessor) {
        return Err(PolicyError::UnknownDependency { key, predecessor });
    }
    dag.add_edge(predecessor, key, ());
    Ok(())
}

/// Once-admitted occurrence topology. Dispatch evaluates only one occurrence and its
/// immediate scientific premises; whole-study admission is not repeated per claim.
#[derive(Clone, Debug)]
pub struct AdmittedStudy {
    policies: BTreeMap<OccurrenceKey, PointPolicy>,
    order: Vec<OccurrenceKey>,
    predecessors: BTreeMap<OccurrenceKey, BTreeSet<OccurrenceKey>>,
}

impl AdmittedStudy {
    /// Admit the complete immutable graph once before any execution effect.
    pub fn new(graph: &OccurrenceGraph) -> Result<Self, PolicyError> {
        admit(graph)?;
        let policies = graph.points.iter().map(|point| (point.key, point.clone())).collect();
        let predecessors = graph.points.iter().map(|point| {
            let mut keys: BTreeSet<_> = point.dependencies.iter().map(|dependency| dependency.predecessor()).collect();
            if let StartPolicy::Continuation(edge) = &point.start {
                keys.insert(edge.predecessor);
            }
            (point.key, keys)
        }).collect();
        Ok(Self { policies, predecessors,order:graph.points.iter().map(|point|point.key).collect() })
    }

    /// Exact immediate read set, excluding the candidate itself.
    pub fn predecessors(&self, key: OccurrenceKey) -> Result<&BTreeSet<OccurrenceKey>, PolicyError> {
        self.predecessors.get(&key).ok_or(PolicyError::UnknownFacts { key })
    }

    /// Compute a claim decision from the candidate and its immediate predecessor facts.
    /// The adapter fences every supplied revision together with cancellation before
    /// applying this result. No transitive predecessor snapshots are needed.
    pub fn action(&self, key: OccurrenceKey, facts: &[PointFacts], cancelled: bool) -> Result<PointAction, PolicyError> {
        let mut expected = self.predecessors(key)?.clone();
        expected.insert(key);
        let mut snapshot = BTreeMap::new();
        for fact in facts {
            if !expected.contains(&fact.key) {
                return Err(PolicyError::UnknownFacts { key: fact.key });
            }
            if snapshot.insert(fact.key, fact).is_some() {
                return Err(PolicyError::DuplicateOccurrence { key: fact.key });
            }
        }
        for required in expected {
            get_fact(&snapshot, required)?;
        }
        let point = self.policies.get(&key).ok_or(PolicyError::UnknownFacts { key })?;
        let fact = get_fact(&snapshot, key)?;
        Ok(PointAction {
            occurrence: key,
            expected_revision: fact.revision,
            kind: action(point, fact, &snapshot, &self.policies, cancelled)?,
        })
    }
    /// Derive the complete conclusion once from the already-admitted topology.
    /// Per-dispatch callers use `action`; this method does not readmit graph structure.
    pub fn decision(&self,facts:&[PointFacts],cancelled:bool)->Result<StudyDecision,PolicyError>{
    let keys: BTreeSet<_> = self.policies.keys().copied().collect();
    let mut snapshot = BTreeMap::new();
    for fact in facts {
        if !keys.contains(&fact.key) {
            return Err(PolicyError::UnknownFacts { key: fact.key });
        }
        if snapshot.insert(fact.key, fact).is_some() {
            return Err(PolicyError::DuplicateOccurrence { key: fact.key });
        }
    }
    let mut actions = Vec::with_capacity(self.order.len());
    for key in &self.order {
        let point=self.policies.get(key).ok_or(PolicyError::UnknownFacts{key:*key})?;
        let fact = get_fact(&snapshot, point.key)?;
        let kind = action(point, fact, &snapshot, &self.policies, cancelled)?;
        actions.push(PointAction {
            occurrence: point.key,
            expected_revision: fact.revision,
            kind,
        });
    }
    let usable = facts.iter().filter(|fact| fact.scientific.usable).count();
    let settled = actions
        .iter()
        .all(|action| matches!(action.kind, ActionKind::Wait(WaitReason::Terminal)));
    let availability = if settled && usable == self.order.len() {
        Availability::Complete
    } else if usable > 0 {
        Availability::Partial
    } else {
        Availability::None
    };
    let lifecycle = if cancelled {
        StudyLifecycle::Cancelled
    } else if settled {
        StudyLifecycle::Terminal
    } else {
        StudyLifecycle::Active
    };
    Ok(StudyDecision {
        actions,
        conclusion: Conclusion {
            availability,
            lifecycle,
        },
    })
    }
}

/// Effect-free scoped policy for an already-admitted immutable study. The adapter
/// supplies exactly the candidate and immediate predecessor policies/facts and
/// subsequently fences every consumed revision. This grants no execution authority.
pub fn candidate_action(point: &PointPolicy, predecessor_policies: &[PointPolicy], facts: &[PointFacts], cancelled: bool) -> Result<PointAction, PolicyError> {
    let mut required: BTreeSet<_> = point.dependencies.iter().map(|dependency| dependency.predecessor()).collect();
    if let StartPolicy::Continuation(edge) = &point.start { required.insert(edge.predecessor); }
    let mut policies = BTreeMap::from([(point.key, point.clone())]);
    for predecessor in predecessor_policies {
        if !required.contains(&predecessor.key) { return Err(PolicyError::UnknownFacts { key: predecessor.key }); }
        if policies.insert(predecessor.key, predecessor.clone()).is_some() { return Err(PolicyError::DuplicateOccurrence { key: predecessor.key }); }
    }
    for key in &required { if !policies.contains_key(key) { return Err(PolicyError::MissingFacts { key: *key }); } }
    required.insert(point.key);
    let mut snapshot = BTreeMap::new();
    for fact in facts {
        if !required.contains(&fact.key) { return Err(PolicyError::UnknownFacts { key: fact.key }); }
        if snapshot.insert(fact.key, fact).is_some() { return Err(PolicyError::DuplicateOccurrence { key: fact.key }); }
    }
    for key in required { get_fact(&snapshot, key)?; }
    let fact = get_fact(&snapshot, point.key)?;
    Ok(PointAction { occurrence: point.key, expected_revision: fact.revision,
        kind: action(point, fact, &snapshot, &policies, cancelled)? })
}

/// Pure transition over an exact snapshot, with one action per authored occurrence.
/// Applying adapters must reread/recompute under their existing locks and check revisions.
/// # Errors
/// Refuses malformed graphs or incomplete/duplicate/unadmitted occurrence facts.
pub fn transition(
    graph: &OccurrenceGraph,
    facts: &[PointFacts],
    cancelled: bool,
) -> Result<StudyDecision, PolicyError> {
    AdmittedStudy::new(graph)?.decision(facts,cancelled)
}

fn get_fact<'a>(
    snapshot: &BTreeMap<OccurrenceKey, &'a PointFacts>,
    key: OccurrenceKey,
) -> Result<&'a PointFacts, PolicyError> {
    snapshot
        .get(&key)
        .copied()
        .ok_or(PolicyError::MissingFacts { key })
}

const fn terminal(state: StudyPointState) -> bool {
    match state {
        StudyPointState::Pending | StudyPointState::Assigned => false,
        StudyPointState::Completed | StudyPointState::Failed | StudyPointState::Cancelled => true,
    }
}

fn action(
    point: &PointPolicy,
    fact: &PointFacts,
    snapshot: &BTreeMap<OccurrenceKey, &PointFacts>,
    policies: &BTreeMap<OccurrenceKey, PointPolicy>,
    cancelled: bool,
) -> Result<ActionKind, PolicyError> {
    if fact.effect == EffectState::Unknown {
        return Ok(ActionKind::Reconcile);
    }
    if cancelled && !terminal(fact.lifecycle) {
        return Ok(ActionKind::Cancel);
    }
    let retry = may_retry(point, fact, cancelled);
    if terminal(fact.lifecycle) && !retry {
        return Ok(ActionKind::Wait(WaitReason::Terminal));
    }
    if fact.lifecycle == StudyPointState::Assigned && fact.native_started {
        return Ok(ActionKind::Wait(WaitReason::Assigned));
    }
    let mut waiting = None;
    for dependency in &point.dependencies {
        let predecessor = dependency.predecessor();
        let previous = get_fact(snapshot, predecessor)?;
        if !predecessor_settled(policies, previous, cancelled)? {
            waiting.get_or_insert(predecessor);
        } else if matches!(dependency, Dependency::UsableResult(_)) && !previous.scientific.usable {
            return Ok(ActionKind::Refuse(Refusal::DependencyUnusable {
                predecessor,
            }));
        }
    }
    if let Some(predecessor) = waiting {
        return Ok(ActionKind::Wait(WaitReason::Dependency { predecessor }));
    }
    match &point.start {
        StartPolicy::Fresh => Ok(ActionKind::Start(
            if point.seed_need == SeedNeed::NotNeeded {
                StartProvenance::NotNeeded
            } else {
                StartProvenance::Fresh
            },
        )),
        StartPolicy::Explicit {
            role,
            seed: requested,
        } => Ok(match seed_availability(fact, *role) {
            SeedAvailability::Unresolved => {
                ActionKind::Wait(WaitReason::SeedResolution { role: *role })
            }
            SeedAvailability::Compatible { seed } if seed == *requested => {
                ActionKind::Start(StartProvenance::Explicit { role: *role, seed })
            }
            SeedAvailability::Compatible { .. } => {
                unavailable(None, *role, SeedUnavailable::Incompatible)
            }
            SeedAvailability::Absent => unavailable(None, *role, SeedUnavailable::Absent),
            SeedAvailability::Incompatible => {
                unavailable(None, *role, SeedUnavailable::Incompatible)
            }
            SeedAvailability::InternalFailure => ActionKind::Refuse(Refusal::SeedInternal {
                predecessor: None,
                role: *role,
            }),
        }),
        StartPolicy::Continuation(edge) => {
            let previous = get_fact(snapshot, edge.predecessor)?;
            if !predecessor_settled(policies, previous, cancelled)? {
                return Ok(ActionKind::Wait(WaitReason::Dependency {
                    predecessor: edge.predecessor,
                }));
            }
            let permitted = previous.scientific.usable
                || (edge.permission == ContinuationPermission::AllowSeedOnly
                    && previous.scientific.seed_permission);
            if point.seed_need == SeedNeed::NotNeeded {
                return Ok(if permitted {
                    ActionKind::Start(StartProvenance::NotNeeded)
                } else {
                    ActionKind::Refuse(Refusal::DependencyUnusable {
                        predecessor: edge.predecessor,
                    })
                });
            }
            let absent = match seed_availability(fact, edge.role) {
                SeedAvailability::Unresolved => {
                    return Ok(
                        if permitted
                            || edge.unavailable == UnavailableSeedPolicy::FreshOnUnavailable
                        {
                            ActionKind::Wait(WaitReason::SeedResolution { role: edge.role })
                        } else {
                            ActionKind::Refuse(Refusal::SeedPermission {
                                predecessor: edge.predecessor,
                                role: edge.role,
                            })
                        },
                    );
                }
                SeedAvailability::Compatible { seed } => {
                    return Ok(if permitted {
                        ActionKind::Start(StartProvenance::Continuation {
                            predecessor: edge.predecessor,
                            role: edge.role,
                            seed,
                        })
                    } else {
                        ActionKind::Refuse(Refusal::SeedPermission {
                            predecessor: edge.predecessor,
                            role: edge.role,
                        })
                    });
                }
                SeedAvailability::InternalFailure => {
                    return Ok(ActionKind::Refuse(Refusal::SeedInternal {
                        predecessor: Some(edge.predecessor),
                        role: edge.role,
                    }));
                }
                SeedAvailability::Absent => SeedUnavailable::Absent,
                SeedAvailability::Incompatible => SeedUnavailable::Incompatible,
            };
            Ok(
                if edge.unavailable == UnavailableSeedPolicy::FreshOnUnavailable {
                    ActionKind::Start(StartProvenance::FreshFallback {
                        predecessor: edge.predecessor,
                        role: edge.role,
                        reason: absent,
                    })
                } else {
                    unavailable(Some(edge.predecessor), edge.role, absent)
                },
            )
        }
    }
}

/// A terminal failure may be retried only under the admitted attempt/effect policy.
pub fn may_retry(point: &PointPolicy, fact: &PointFacts, cancelled: bool) -> bool {
    fact.lifecycle == StudyPointState::Failed
        && !cancelled
        && fact.retry_failure == Some(RetryFailure::Transient)
        && fact.attempt_count < point.attempt_limit
        && matches!(fact.effect, EffectState::Absent | EffectState::Idempotent)
}

fn predecessor_settled(
    policies: &BTreeMap<OccurrenceKey, PointPolicy>,
    fact: &PointFacts,
    cancelled: bool,
) -> Result<bool, PolicyError> {
    let point = policies
        .get(&fact.key)
        .ok_or(PolicyError::UnknownFacts { key: fact.key })?;
    Ok(terminal(fact.lifecycle)
        && fact.effect != EffectState::Unknown
        && !may_retry(point, fact, cancelled))
}

fn seed_availability(fact: &PointFacts, role: SeedRole) -> SeedAvailability {
    match &fact.seed {
        None => SeedAvailability::Absent,
        Some(seed) if seed.role == role => seed.availability,
        Some(_) => SeedAvailability::Incompatible,
    }
}

fn unavailable(
    predecessor: Option<OccurrenceKey>,
    role: SeedRole,
    reason: SeedUnavailable,
) -> ActionKind {
    ActionKind::Refuse(Refusal::SeedUnavailable {
        predecessor,
        role,
        reason,
    })
}

#[cfg(test)]
mod study_policy_unit {
    use super::*;
    use pse_model::generated::enums::CandidateUse;
    use pse_model::generated::identities::SolutionId;
    use pse_model::study::{ScientificFacts, SeedEdge, SeedFact};

    fn point(key: u32) -> PointPolicy {
        PointPolicy {
            key: OccurrenceKey(key),
            dependencies: vec![],
            seed_need: SeedNeed::Required,
            start: StartPolicy::Fresh,
            attempt_limit: 1,
        }
    }
    fn fact(key: u32, lifecycle: StudyPointState, usable: bool) -> PointFacts {
        PointFacts {
            key: OccurrenceKey(key),
            revision: 41 + u64::from(key),
            lifecycle,
            native_started: lifecycle == StudyPointState::Assigned,
            scientific: ScientificFacts {
                usable,
                candidate_use: None,
                seed_permission: usable,
            },
            attempt_count: u32::from(terminal(lifecycle)),
            retry_failure: None,
            effect: EffectState::Absent,
            seed: None,
        }
    }
    fn continuation() -> (OccurrenceGraph, Vec<PointFacts>) {
        let mut child = point(7);
        child.start = StartPolicy::Continuation(SeedEdge {
            predecessor: OccurrenceKey(3),
            role: SeedRole::PrimalSolution,
            permission: ContinuationPermission::RequireUsable,
            unavailable: UnavailableSeedPolicy::Refuse,
        });
        let mut child_fact = fact(7, StudyPointState::Pending, false);
        child_fact.seed = Some(SeedFact {
            role: SeedRole::PrimalSolution,
            availability: SeedAvailability::Compatible {
                seed: SolutionId::from_bytes([9; 16]),
            },
        });
        (
            OccurrenceGraph {
                points: vec![point(3), child],
            },
            vec![fact(3, StudyPointState::Completed, true), child_fact],
        )
    }

    #[test]
    fn scoped_dispatch_consumes_only_immediate_predecessors() {
        let (mut graph, mut facts) = continuation();
        let mut grandparent = point(1);
        grandparent.seed_need = SeedNeed::NotNeeded;
        graph.points[0].dependencies.push(Dependency::Ordering(OccurrenceKey(1)));
        graph.points.push(grandparent);
        // The grandparent's pending state does not need to be read again once its
        // direct dependent has truthfully completed under that dependency.
        facts.push(fact(1, StudyPointState::Pending, false));
        let admitted = AdmittedStudy::new(&graph).unwrap();
        assert_eq!(admitted.predecessors(OccurrenceKey(7)).unwrap(), &BTreeSet::from([OccurrenceKey(3)]));
        let scoped = admitted.action(OccurrenceKey(7), &facts[..2], false).unwrap();
        assert!(matches!(scoped.kind, ActionKind::Start(StartProvenance::Continuation { predecessor: OccurrenceKey(3), .. })));
        assert!(admitted.action(OccurrenceKey(7), &facts, false).is_err());
        assert!(admitted.action(OccurrenceKey(7), &facts[1..2], false).is_err());
    }

    #[test]
    fn independent_dispatch_does_not_require_unrelated_facts() {
        let graph = OccurrenceGraph { points: (0..1000).map(point).collect() };
        let admitted = AdmittedStudy::new(&graph).unwrap();
        let candidate = fact(900, StudyPointState::Pending, false);
        let start = admitted.action(OccurrenceKey(900), &[candidate.clone()], false).unwrap();
        assert!(matches!(start.kind, ActionKind::Start(StartProvenance::Fresh)));
        let cancelled = admitted.action(OccurrenceKey(900), &[candidate], true).unwrap();
        assert!(matches!(cancelled.kind, ActionKind::Cancel));
    }
    fn child_action(graph: &OccurrenceGraph, facts: &[PointFacts]) -> ActionKind {
        transition(graph, facts, false).unwrap().actions[1]
            .kind
            .clone()
    }
    fn edge(graph: &mut OccurrenceGraph) -> &mut SeedEdge {
        let StartPolicy::Continuation(edge) = &mut graph.points[1].start else {
            panic!("fixture edge")
        };
        edge
    }

    #[test]
    fn unresolved_seed_grants_acquisition_without_native_start() {
        let (graph, mut facts) = continuation();
        facts[1].seed.as_mut().unwrap().availability = SeedAvailability::Unresolved;
        assert!(matches!(
            child_action(&graph, &facts),
            ActionKind::Wait(WaitReason::SeedResolution { .. })
        ));
        facts[1].lifecycle = StudyPointState::Assigned;
        facts[1].native_started = false;
        assert!(matches!(
            child_action(&graph, &facts),
            ActionKind::Wait(WaitReason::SeedResolution { .. })
        ));
        facts[1].seed.as_mut().unwrap().availability = SeedAvailability::Compatible {
            seed: SolutionId::from_bytes([9; 16]),
        };
        assert!(matches!(child_action(&graph, &facts), ActionKind::Start(_)));
        facts[1].native_started = true;
        assert!(matches!(
            child_action(&graph, &facts),
            ActionKind::Wait(WaitReason::Assigned)
        ));
    }

    #[test]
    fn admission_checks_all_edge_kinds_and_occurrence_identity() {
        let (mut graph, _) = continuation();
        assert_eq!(admit(&graph), Ok(()));
        graph.points[0]
            .dependencies
            .push(Dependency::Ordering(OccurrenceKey(7)));
        assert_eq!(admit(&graph), Err(PolicyError::Cyclic));
        graph.points[0].dependencies = vec![Dependency::UsableResult(OccurrenceKey(99))];
        assert_eq!(
            admit(&graph),
            Err(PolicyError::UnknownDependency {
                key: OccurrenceKey(3),
                predecessor: OccurrenceKey(99)
            })
        );
        graph.points[0].dependencies.clear();
        graph.points[1].key = OccurrenceKey(3);
        assert_eq!(
            admit(&graph),
            Err(PolicyError::DuplicateOccurrence {
                key: OccurrenceKey(3)
            })
        );
        let mut duplicate = point(1);
        duplicate.dependencies = vec![Dependency::Ordering(OccurrenceKey(0)); 2];
        assert_eq!(
            admit(&OccurrenceGraph {
                points: vec![point(0), duplicate]
            }),
            Err(PolicyError::DuplicateDependency {
                key: OccurrenceKey(1)
            })
        );
    }

    #[test]
    fn equal_operation_policies_remain_independent_repetitions() {
        let graph = OccurrenceGraph {
            points: vec![point(1), point(2)],
        };
        let facts = vec![
            fact(2, StudyPointState::Pending, false),
            fact(1, StudyPointState::Pending, false),
        ];
        let decision = transition(&graph, &facts, false).unwrap();
        assert_eq!(
            decision
                .actions
                .iter()
                .map(|a| a.occurrence)
                .collect::<Vec<_>>(),
            vec![OccurrenceKey(1), OccurrenceKey(2)]
        );
        assert_eq!(
            decision
                .actions
                .iter()
                .map(|a| a.expected_revision)
                .collect::<Vec<_>>(),
            vec![42, 43]
        );
        assert!(
            decision
                .actions
                .iter()
                .all(|a| a.kind == ActionKind::Start(StartProvenance::Fresh))
        );
        assert_eq!(
            transition(&graph, &facts.into_iter().rev().collect::<Vec<_>>(), false).unwrap(),
            decision
        );
    }

    #[test]
    fn strict_continuation_waits_then_requires_usable_compatible_seed() {
        let (graph, mut facts) = continuation();
        assert!(matches!(
            child_action(&graph, &facts),
            ActionKind::Start(StartProvenance::Continuation {
                predecessor: OccurrenceKey(3),
                role: SeedRole::PrimalSolution,
                ..
            })
        ));
        facts[0].lifecycle = StudyPointState::Assigned;
        assert_eq!(
            child_action(&graph, &facts),
            ActionKind::Wait(WaitReason::Dependency {
                predecessor: OccurrenceKey(3)
            })
        );
        facts[0].lifecycle = StudyPointState::Failed;
        facts[0].scientific.usable = false;
        facts[0].scientific.seed_permission = true;
        facts[0].scientific.candidate_use = Some(CandidateUse::SeedOnly);
        assert_eq!(
            child_action(&graph, &facts),
            ActionKind::Refuse(Refusal::SeedPermission {
                role: SeedRole::PrimalSolution,
                predecessor: OccurrenceKey(3)
            })
        );
    }

    #[test]
    fn seed_only_permission_never_satisfies_usable_result_dependency() {
        let (mut graph, mut facts) = continuation();
        facts[0].scientific.usable = false;
        facts[0].scientific.seed_permission = true;
        facts[0].scientific.candidate_use = Some(CandidateUse::SeedOnly);
        edge(&mut graph).permission = ContinuationPermission::AllowSeedOnly;
        assert!(matches!(
            child_action(&graph, &facts),
            ActionKind::Start(StartProvenance::Continuation { .. })
        ));
        graph.points[1]
            .dependencies
            .push(Dependency::UsableResult(OccurrenceKey(3)));
        assert_eq!(
            child_action(&graph, &facts),
            ActionKind::Refuse(Refusal::DependencyUnusable {
                predecessor: OccurrenceKey(3)
            })
        );
    }

    #[test]
    fn ordering_failure_releases_but_usable_failure_refuses() {
        let graph = OccurrenceGraph {
            points: vec![
                point(0),
                PointPolicy {
                    dependencies: vec![Dependency::Ordering(OccurrenceKey(0))],
                    ..point(1)
                },
            ],
        };
        let facts = vec![
            fact(0, StudyPointState::Failed, false),
            fact(1, StudyPointState::Pending, false),
        ];
        assert_eq!(
            child_action(&graph, &facts),
            ActionKind::Start(StartProvenance::Fresh)
        );
        let mut usable_graph = graph;
        usable_graph.points[1].dependencies = vec![Dependency::UsableResult(OccurrenceKey(0))];
        assert_eq!(
            child_action(&usable_graph, &facts),
            ActionKind::Refuse(Refusal::DependencyUnusable {
                predecessor: OccurrenceKey(0)
            })
        );
        assert_eq!(
            transition(&usable_graph, &facts, true).unwrap().actions[1].kind,
            ActionKind::Cancel
        );
    }

    #[test]
    fn fallback_is_explicit_and_only_for_absence_or_incompatibility() {
        let (mut graph, mut facts) = continuation();
        for (availability, reason) in [
            (SeedAvailability::Absent, SeedUnavailable::Absent),
            (
                SeedAvailability::Incompatible,
                SeedUnavailable::Incompatible,
            ),
        ] {
            facts[1].seed.as_mut().unwrap().availability = availability;
            assert_eq!(
                child_action(&graph, &facts),
                unavailable(Some(OccurrenceKey(3)), SeedRole::PrimalSolution, reason)
            );
            edge(&mut graph).unavailable = UnavailableSeedPolicy::FreshOnUnavailable;
            assert_eq!(
                child_action(&graph, &facts),
                ActionKind::Start(StartProvenance::FreshFallback {
                    predecessor: OccurrenceKey(3),
                    role: SeedRole::PrimalSolution,
                    reason
                })
            );
            edge(&mut graph).unavailable = UnavailableSeedPolicy::Refuse;
        }
        edge(&mut graph).unavailable = UnavailableSeedPolicy::FreshOnUnavailable;
        facts[1].seed.as_mut().unwrap().availability = SeedAvailability::InternalFailure;
        assert_eq!(
            child_action(&graph, &facts),
            ActionKind::Refuse(Refusal::SeedInternal {
                predecessor: Some(OccurrenceKey(3)),
                role: SeedRole::PrimalSolution
            })
        );
    }

    #[test]
    fn failed_predecessor_fallback_acquires_seed_facts_without_granting_seed_permission() {
        let (mut graph, mut facts) = continuation();
        facts[0].lifecycle = StudyPointState::Failed;
        facts[0].scientific = ScientificFacts::default();
        edge(&mut graph).unavailable = UnavailableSeedPolicy::FreshOnUnavailable;
        facts[1].seed.as_mut().unwrap().availability = SeedAvailability::Unresolved;
        assert_eq!(
            child_action(&graph, &facts),
            ActionKind::Wait(WaitReason::SeedResolution {
                role: SeedRole::PrimalSolution,
            })
        );
        for (availability, reason) in [
            (SeedAvailability::Absent, SeedUnavailable::Absent),
            (
                SeedAvailability::Incompatible,
                SeedUnavailable::Incompatible,
            ),
        ] {
            facts[1].seed.as_mut().unwrap().availability = availability;
            assert_eq!(
                child_action(&graph, &facts),
                ActionKind::Start(StartProvenance::FreshFallback {
                    predecessor: OccurrenceKey(3),
                    role: SeedRole::PrimalSolution,
                    reason,
                })
            );
        }
        facts[1].seed.as_mut().unwrap().availability = SeedAvailability::Compatible {
            seed: SolutionId::from_bytes([8; 16]),
        };
        assert_eq!(
            child_action(&graph, &facts),
            ActionKind::Refuse(Refusal::SeedPermission {
                predecessor: OccurrenceKey(3),
                role: SeedRole::PrimalSolution,
            })
        );
        facts[1].seed.as_mut().unwrap().availability = SeedAvailability::InternalFailure;
        assert_eq!(
            child_action(&graph, &facts),
            ActionKind::Refuse(Refusal::SeedInternal {
                predecessor: Some(OccurrenceKey(3)),
                role: SeedRole::PrimalSolution,
            })
        );
        facts[1].seed.as_mut().unwrap().availability = SeedAvailability::Absent;
        graph.points[1]
            .dependencies
            .push(Dependency::UsableResult(OccurrenceKey(3)));
        assert_eq!(
            child_action(&graph, &facts),
            ActionKind::Refuse(Refusal::DependencyUnusable {
                predecessor: OccurrenceKey(3),
            })
        );
    }

    #[test]
    fn supplied_seed_and_wrong_output_role_never_silently_fall_back() {
        let (mut graph, mut facts) = continuation();
        graph.points[1].start = StartPolicy::Explicit {
            role: SeedRole::PrimalSolution,
            seed: SolutionId::from_bytes([9; 16]),
        };
        facts[1].seed.as_mut().unwrap().availability = SeedAvailability::Compatible {
            seed: SolutionId::from_bytes([8; 16]),
        };
        assert_eq!(
            child_action(&graph, &facts),
            unavailable(
                None,
                SeedRole::PrimalSolution,
                SeedUnavailable::Incompatible
            )
        );
        facts[1].seed.as_mut().unwrap().role = SeedRole::Trajectory;
        assert_eq!(
            child_action(&graph, &facts),
            unavailable(
                None,
                SeedRole::PrimalSolution,
                SeedUnavailable::Incompatible
            )
        );
        facts[1].seed = None;
        assert_eq!(
            child_action(&graph, &facts),
            unavailable(None, SeedRole::PrimalSolution, SeedUnavailable::Absent)
        );
    }

    #[test]
    fn seed_free_operation_retains_order_and_scientific_condition() {
        let (mut graph, mut facts) = continuation();
        graph.points[1].seed_need = SeedNeed::NotNeeded;
        facts[1].seed = None;
        assert_eq!(
            child_action(&graph, &facts),
            ActionKind::Start(StartProvenance::NotNeeded)
        );
        facts[0].lifecycle = StudyPointState::Assigned;
        assert!(matches!(
            child_action(&graph, &facts),
            ActionKind::Wait(WaitReason::Dependency { .. })
        ));
        facts[0].lifecycle = StudyPointState::Failed;
        facts[0].scientific.usable = false;
        assert_eq!(
            child_action(&graph, &facts),
            ActionKind::Refuse(Refusal::DependencyUnusable {
                predecessor: OccurrenceKey(3)
            })
        );
        graph.points[1].start = StartPolicy::Fresh;
        graph.points[1].dependencies = vec![Dependency::Ordering(OccurrenceKey(3))];
        assert_eq!(
            child_action(&graph, &facts),
            ActionKind::Start(StartProvenance::NotNeeded)
        );
    }

    #[test]
    fn retry_requires_transient_failure_bounded_attempts_and_safe_effect() {
        let graph = OccurrenceGraph {
            points: vec![PointPolicy {
                attempt_limit: 2,
                ..point(0)
            }],
        };
        let mut facts = vec![fact(0, StudyPointState::Failed, false)];
        for failure in [None, Some(RetryFailure::Deterministic)] {
            facts[0].retry_failure = failure;
            assert_eq!(
                transition(&graph, &facts, false).unwrap().actions[0].kind,
                ActionKind::Wait(WaitReason::Terminal)
            );
        }
        facts[0].retry_failure = Some(RetryFailure::Transient);
        for effect in [EffectState::Absent, EffectState::Idempotent] {
            facts[0].effect = effect;
            assert_eq!(
                transition(&graph, &facts, false).unwrap().actions[0].kind,
                ActionKind::Start(StartProvenance::Fresh)
            );
        }
        facts[0].effect = EffectState::Present;
        assert_eq!(
            transition(&graph, &facts, false).unwrap().actions[0].kind,
            ActionKind::Wait(WaitReason::Terminal)
        );
        facts[0].effect = EffectState::Unknown;
        let decision = transition(&graph, &facts, false).unwrap();
        assert_eq!(decision.actions[0].kind, ActionKind::Reconcile);
        assert_eq!(decision.conclusion.lifecycle, StudyLifecycle::Active);
        facts[0].effect = EffectState::Absent;
        facts[0].attempt_count = 2;
        assert_eq!(
            transition(&graph, &facts, false).unwrap().actions[0].kind,
            ActionKind::Wait(WaitReason::Terminal)
        );
    }

    #[test]
    fn partial_multi_result_and_cancellation_keep_availability_separate() {
        let graph = OccurrenceGraph {
            points: vec![point(0), point(1)],
        };
        let mut facts = vec![
            fact(0, StudyPointState::Completed, true),
            fact(1, StudyPointState::Failed, false),
        ];
        // One candidate/table can look usable without E granting aggregate run permission.
        facts[1].scientific.candidate_use = Some(CandidateUse::Usable);
        let decision = transition(&graph, &facts, false).unwrap();
        assert_eq!(
            decision.conclusion,
            Conclusion {
                availability: Availability::Partial,
                lifecycle: StudyLifecycle::Terminal
            }
        );
        assert_eq!(
            transition(&graph, &facts, true).unwrap().conclusion,
            Conclusion {
                availability: Availability::Partial,
                lifecycle: StudyLifecycle::Cancelled
            }
        );
        facts[1].scientific.usable = true;
        assert_eq!(
            transition(&graph, &facts, false)
                .unwrap()
                .conclusion
                .availability,
            Availability::Complete
        );
    }

    #[test]
    fn incomplete_or_unadmitted_snapshots_are_not_scientific_facts() {
        let graph = OccurrenceGraph {
            points: vec![point(0), point(1)],
        };
        assert_eq!(
            transition(&graph, &[fact(0, StudyPointState::Pending, false)], false),
            Err(PolicyError::MissingFacts {
                key: OccurrenceKey(1)
            })
        );
        assert_eq!(
            transition(&graph, &[fact(9, StudyPointState::Completed, true)], false),
            Err(PolicyError::UnknownFacts {
                key: OccurrenceKey(9)
            })
        );
    }

    #[test]
    fn retry_and_unknown_publication_keep_dependents_waiting() {
        let (mut graph, mut facts) = continuation();
        graph.points[0].attempt_limit = 2;
        facts[0].lifecycle = StudyPointState::Failed;
        facts[0].retry_failure = Some(RetryFailure::Transient);
        assert_eq!(
            child_action(&graph, &facts),
            ActionKind::Wait(WaitReason::Dependency {
                predecessor: OccurrenceKey(3)
            })
        );
        facts[0].retry_failure = Some(RetryFailure::Deterministic);
        facts[0].effect = EffectState::Unknown;
        assert_eq!(
            child_action(&graph, &facts),
            ActionKind::Wait(WaitReason::Dependency {
                predecessor: OccurrenceKey(3)
            })
        );
        facts[0].effect = EffectState::Absent;
        assert!(matches!(
            child_action(&graph, &facts),
            ActionKind::Start(StartProvenance::Continuation { .. })
        ));
    }
}
