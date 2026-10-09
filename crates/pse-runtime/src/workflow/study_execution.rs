// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! In-process adapter of the same occurrence policy used by durable workers.
use super::modeling::assessment::Obligations;
use super::staged::Staged;
use super::{
    ModelingPackage, PreparedStudyOperation, RunReport, RunRequest, RunResult, StudyDefinition,
    WorkflowError,
};
use futures_util::{FutureExt, StreamExt};
use pse_diagnostics::{DiagnosticRule, DiagnosticStage};
use pse_model::diagnostic::{BoundaryClass, BoundaryDiagnostic, Observation};
use pse_model::generated::{
    enums::StudyPointState,
    identities::{RunId, SolutionId},
};
use pse_model::study::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// One completed occurrence keeps an exact persisted handle in application mode.
/// Explicit kernel tests may retain their requested native owner in memory.
#[derive(Clone, Debug)]
pub enum StudyOccurrenceResult {
    /// Canonical storage remains the result authority after worker buffers release.
    Retained {
        /// Original scientific execution identity.
        run_id: RunId,
        /// Exact canonical execution key.
        run: String,
        /// Exact admitted native attempt key.
        attempt: String,
    },
    /// Explicitly selected ephemeral numerical workflow.
    Ephemeral(Arc<RunResult>),
}
impl StudyOccurrenceResult {
    /// Semantic execution identity, distinct from the opaque native attempt key.
    pub fn run_id(&self) -> RunId {
        match self {
            Self::Retained { run_id, .. } => *run_id,
            Self::Ephemeral(result) => result.run_id,
        }
    }
    /// Exact connected selection for persisted scientific rows.
    pub fn stored_keys(&self) -> Option<(&str, &str)> {
        match self {
            Self::Retained { run, attempt, .. } => Some((run, attempt)),
            Self::Ephemeral(_) => None,
        }
    }
    /// Original native report is available only in explicit kernel mode. Applications
    /// read retained rows through the canonical result selector.
    pub fn report(&self) -> Result<&RunReport, WorkflowError> {
        match self {
            Self::Ephemeral(result) => result
                .report
                .as_ref()
                .map_err(|error| WorkflowError::Shared(error.clone())),
            Self::Retained { .. } => Err(super::contract(
                "persisted occurrence requires a connected result selection",
            )),
        }
    }
}

/// Complete occurrences and retained owner results, including cancellation and refusals.
#[derive(Debug)]
pub struct StudyReport {
    /// Execution identity; equal binding content never merges studies or occurrences.
    pub run_id: RunId,
    /// One immutable admitted definition shared by both adapters.
    pub definition: StudyDefinition,
    /// One terminal observation for every requested occurrence.
    pub outcomes: Vec<PointOutcome>,
    /// Original joined operation reports and their available members.
    pub results: Vec<Option<StudyOccurrenceResult>>,
    /// Shared conclusion preserves lifecycle separately from scientific availability.
    pub decision: StudyDecision,
    /// Preparations performed in the calling process during this operation.
    /// Ephemeral execution includes its numerical work; a durable observer does not
    /// count work performed by the managed primary. These are observations, not
    /// cross-process execution or reuse evidence.
    pub preparations: crate::math::PreparationCounts,
    pub(in crate::workflow) runtime: super::Runtime,
    pub(in crate::workflow) _owner: Arc<pse_columnar::AllocationLease>,
}
impl StudyReport {
    /// Original application store and shared pool for exact connected results.
    pub fn runtime(&self) -> &super::Runtime {
        &self.runtime
    }
}

impl ModelingPackage {
    /// Execute the admitted definition through existing operation owners and pure policy.
    pub async fn study(
        &self,
        definition: &StudyDefinition,
        maximum_points: usize,
        cancel: &crate::CancelSource,
    ) -> Result<StudyReport, WorkflowError> {
        let (report, preparations) = crate::math::counted(async {
            match self.runtime.durability() {
                super::Durability::Durable(_) => {
                    self.study_durable(definition, maximum_points, cancel).await
                }
                super::Durability::Ephemeral => {
                    self.study_inner(definition, maximum_points, cancel).await
                }
            }
        })
        .await;
        let mut report = report?;
        report.preparations = preparations;
        Ok(report)
    }
    async fn study_durable(
        &self,
        definition: &StudyDefinition,
        maximum_points: usize,
        cancel: &crate::CancelSource,
    ) -> Result<StudyReport, WorkflowError> {
        if definition.modeling_revision != self.canonical_revision().key
            || maximum_points == 0
            || maximum_points > super::MAXIMUM_STUDY_POINTS
            || definition.points.len() > maximum_points
        {
            return Err(super::contract(
                "bounded study extent and exact modeling revision",
            ));
        }
        let operations = self.runtime.operations()?;
        self.runtime.ensure_managed_primary(cancel).await?;
        let sources =
            super::study::creation_effect(cancel, operations.sources(&definition.physical)).await?;
        let handle = self
            .runtime
            .start_defined_study((**sources).clone(), definition.clone(), cancel)
            .await?;
        drop(sources);
        let mut cancellation_recorded = false;
        loop {
            if cancel.token().is_cancelled() && !cancellation_recorded {
                handle.cancel().await?;
                cancellation_recorded = true;
            }
            if handle.result().await?.is_some() {
                break;
            }
            // This observer has no native work to schedule. Bound header polling
            // independently of case count so waiting does not flood the worker's store.
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        let bytes = definition
            .points
            .len()
            .checked_mul(size_of::<PointOutcome>() + 4096)
            .ok_or_else(|| super::contract("study outcome extent"))?;
        let owner = self
            .runtime
            .shared
            .math()
            .reserve("study:retained-occurrence-summaries", bytes)?;
        let status = handle.status().await?;
        let mut after = None;
        let mut facts = Vec::with_capacity(status.points.len());
        loop {
            let page = operations
                .store()
                .study_point_page(&handle.study_id().to_string(), after)
                .await?;
            if page.is_empty() {
                break;
            }
            for point in page {
                after = Some(point.ordinal);
                facts.push(point.facts()?);
            }
        }
        let decision =
            pse_operations::study_policy::transition(&definition.graph(), &facts, status.cancelled)
                .map_err(policy_error)?;
        let run_id = RunId::from_id(
            pse_ids::SemanticId::parse_hex(
                status
                    .run
                    .strip_prefix("run:")
                    .ok_or_else(|| super::contract("study run lineage absent"))?,
            )
            .map_err(|error| super::contract(error.to_string()))?,
        );
        let mut outcomes = Vec::with_capacity(status.points.len());
        let mut results = Vec::with_capacity(status.points.len());
        for point in status.points {
            outcomes.push(
                point
                    .outcome
                    .ok_or_else(|| super::contract("terminal occurrence outcome absent"))?,
            );
            results.push(
                point
                    .attempt
                    .map(|attempt| {
                        let run_id = RunId::from_id(
                            pse_ids::SemanticId::parse_hex(
                                point
                                    .run
                                    .strip_prefix("run:")
                                    .ok_or_else(|| super::contract("occurrence lineage absent"))?,
                            )
                            .map_err(|error| super::contract(error.to_string()))?,
                        );
                        Ok::<_, WorkflowError>(StudyOccurrenceResult::Retained {
                            run_id,
                            run: point.run,
                            attempt,
                        })
                    })
                    .transpose()?,
            );
        }
        Ok(StudyReport {
            run_id,
            definition: definition.clone(),
            outcomes,
            results,
            decision,
            preparations: Default::default(),
            runtime: self.runtime.clone(),
            _owner: owner,
        })
    }
    async fn study_inner(
        &self,
        definition: &StudyDefinition,
        maximum_points: usize,
        cancel: &crate::CancelSource,
    ) -> Result<StudyReport, WorkflowError> {
        if definition.modeling_revision != self.canonical_revision().key {
            return Err(super::contract(
                "study definition canonical revision differs",
            ));
        }
        if maximum_points == 0
            || maximum_points > super::MAXIMUM_STUDY_POINTS
            || definition.points.len() > maximum_points
        {
            return Err(super::contract("bounded study extent"));
        }
        // Existing operation preparation can own a worker while entering the native
        // sequence. Refuse before issuance unless both population owners can coexist.
        if !definition.points.is_empty() && self.runtime.shared.budget().math.jobs < 2 {
            return Err(crate::math::MathRuntimeError::Limit(
                "study preparation and native sequence population",
            )
            .into());
        }
        let graph = definition.graph();
        let admitted =
            pse_operations::study_policy::AdmittedStudy::new(&graph).map_err(policy_error)?;
        let positions: BTreeMap<_, _> = graph
            .points
            .iter()
            .enumerate()
            .map(|(i, p)| (p.key, i))
            .collect();
        let bytes = definition
            .points
            .len()
            .checked_mul(size_of::<PointOutcome>() + 4096)
            .ok_or_else(|| super::contract("study outcome extent"))?;
        let owner = self
            .runtime
            .shared
            .math()
            .reserve("study:occurrence-outcomes", bytes)?;
        for point in &definition.points {
            if point.binding_hash != point.binding.identity() {
                return Err(study_error(
                    DiagnosticRule::StudyBindingRevision,
                    "binding content identity differs from the recorded binding",
                ));
            }
            if let StartPolicy::Continuation(edge) = &point.policy.start {
                point.operation.admit_seed_role(edge.role)?;
                definition.points[positions[&edge.predecessor]]
                    .operation
                    .admit_seed_role(edge.role)?;
            } else if let StartPolicy::Explicit { role, .. } = point.policy.start {
                point.operation.admit_seed_role(role)?;
            }
        }
        let mut dependents = BTreeMap::<OccurrenceKey, BTreeSet<usize>>::new();
        for (index, point) in graph.points.iter().enumerate() {
            for predecessor in admitted.predecessors(point.key).map_err(policy_error)? {
                dependents.entry(*predecessor).or_default().insert(index);
            }
        }
        let mut frontier = (0..graph.points.len()).collect::<BTreeSet<_>>();
        let run_id = pse_operations::mint_id();
        let mut facts: Vec<_> = graph
            .points
            .iter()
            .map(|point| PointFacts {
                key: point.key,
                revision: 0,
                native_started: false,
                lifecycle: StudyPointState::Pending,
                scientific: ScientificFacts::default(),
                attempt_count: 0,
                retry_failure: None,
                effect: EffectState::Absent,
                seed: match &point.start {
                    StartPolicy::Fresh => None,
                    StartPolicy::Explicit { role, .. } => Some(SeedFact {
                        role: *role,
                        availability: SeedAvailability::Unresolved,
                    }),
                    StartPolicy::Continuation(edge) => Some(SeedFact {
                        role: edge.role,
                        availability: SeedAvailability::Unresolved,
                    }),
                },
            })
            .collect();
        let mut outcomes: Vec<_> = graph
            .points
            .iter()
            .map(|point| PointOutcome {
                key: point.key,
                lifecycle: StudyPointState::Pending,
                scientific: ScientificFacts::default(),
                diagnostic: None,
                start: None,
                effect: EffectState::Absent,
                attempts: vec![],
            })
            .collect();
        let mut results: Vec<Option<Arc<RunResult>>> = vec![None; graph.points.len()];
        let width = self
            .runtime
            .shared
            .math()
            .cores()
            .min(self.runtime.shared.budget().math.jobs / 2)
            .max(1);
        let work_cancel = cancel.child_with_deadline(None);
        let mut pending: futures_util::stream::FuturesUnordered<
            futures_util::future::BoxFuture<'_, LaneCompletion>,
        > = futures_util::stream::FuturesUnordered::new();
        let mut inflight = BTreeSet::new();
        let mut ready = BTreeMap::<usize, ReadyOperation>::new();
        let mut retained = BTreeMap::<usize, Staged>::new();
        // A single continuation successor may inherit private native state. Fan-out
        // consumers retain their exact result seeds but receive independent sessions.
        let mut successors = BTreeMap::<usize, Vec<usize>>::new();
        for (index, point) in graph.points.iter().enumerate() {
            if let StartPolicy::Continuation(edge) = &point.start {
                successors
                    .entry(positions[&edge.predecessor])
                    .or_default()
                    .push(index);
            }
        }
        let mut completed = Vec::new();
        let (decision, preparations) = crate::math::counted(async {
            loop {
                if cancel.token().is_cancelled() && !work_cancel.token().is_cancelled() {
                    work_cancel.cancel();
                    frontier.extend(0..facts.len());
                }
                // Poll every issued future; immediately available completions need no
                // frontier-wide barrier and can expose a real native batch together.
                while let Some(Some(value)) = pending.next().now_or_never() {
                    completed.push(value);
                }
                while !completed.is_empty() {
                    let completion = completed.remove(0);
                    match completion {
                        LaneCompletion::Prepared { index, operation, mut staged } => {
                            inflight.remove(&index);
                            match operation {
                                Ok(operation) => {
                                    ready.insert(index, ReadyOperation { operation, staged });
                                    frontier.insert(index);
                                }
                                Err(error) => {
                                    if let Some(staged) = staged.take() { staged.close().await; }
                                    record_preparation_failure(index, &error, definition,
                                        &mut facts, &mut outcomes);
                                    if matches!(error.boundary_diagnostic().class,
                                        BoundaryClass::Infrastructure | BoundaryClass::Internal) {
                                        return Err(error);
                                    }
                                    if let Some(children) = dependents.get(&graph.points[index].key) {
                                        frontier.extend(children);
                                    }
                                }
                            }
                        }
                        LaneCompletion::Executed { points, mut staged } => {
                            let only = (points.len() == 1).then(|| points[0].index);
                            for point in points {
                                let index = point.index;
                                inflight.remove(&index);
                                if let Some(error) = record_execution(point, definition,
                                    &work_cancel, &mut facts, &mut outcomes, &mut results) {
                                    if let Some(staged) = staged.take() { staged.close().await; }
                                    return Err(error);
                                }
                                if let Some(children) = dependents.get(&graph.points[index].key) {
                                    frontier.extend(children);
                                }
                                if pse_operations::study_policy::may_retry(&graph.points[index],
                                    &facts[index], work_cancel.token().is_cancelled()) {
                                    frontier.insert(index);
                                }
                            }
                            if let Some(staged) = staged {
                                let reusable = only.and_then(|index| {
                                    successors.get(&index).filter(|children| children.len() == 1)
                                        .map(|children| (index, children[0]))
                                });
                                if let Some((index, child)) = reusable {
                                    let action = match scoped_action(&admitted, graph.points[child].key,
                                        &facts, &positions, work_cancel.token().is_cancelled()) {
                                        Ok(action) => action,
                                        Err(error) => { staged.close().await; return Err(error); }
                                    };
                                    if matches!(action.kind, ActionKind::Start(_)
                                        | ActionKind::Wait(WaitReason::SeedResolution { .. })) {
                                        retained.insert(index, staged);
                                    } else { staged.close().await; }
                                } else { staged.close().await; }
                            }
                        }
                    }
                }
                // Continuations with an available private owner get first opportunity
                // to replace that same lane rather than add idle native population.
                let mut candidates = retained.keys().filter_map(|index| successors.get(index))
                    .flatten().filter(|index| frontier.contains(index)).copied().collect::<Vec<_>>();
                let remaining = frontier.iter().filter(|index| !candidates.contains(index))
                    .copied().collect::<Vec<_>>();
                candidates.extend(remaining);
                let mut starts = Vec::new();
                for index in candidates {
                    if inflight.contains(&index) { frontier.remove(&index); continue; }
                    let point = &graph.points[index];
                    if let Some(preparation) = ready.get(&index) {
                        facts[index].seed = match &point.start {
                            StartPolicy::Continuation(edge) => Some(SeedFact {
                                role: edge.role,
                                availability: memory_seed(&preparation.operation,
                                    results[positions[&edge.predecessor]].as_deref()),
                            }),
                            StartPolicy::Explicit { role, .. } => Some(SeedFact {
                                role: *role, availability: SeedAvailability::Absent,
                            }),
                            StartPolicy::Fresh => None,
                        };
                    }
                    let action = scoped_action(&admitted, point.key, &facts, &positions,
                        work_cancel.token().is_cancelled())?;
                    match action.kind {
                        ActionKind::Start(start) if ready.contains_key(&index) => {
                            frontier.remove(&index);
                            starts.push((index, start));
                        }
                        ActionKind::Start(_) | ActionKind::Wait(WaitReason::SeedResolution { .. }) => {
                            let parent = match &point.start {
                                StartPolicy::Continuation(edge) => Some(positions[&edge.predecessor]),
                                _ => None,
                            };
                            let replaces = parent.is_some_and(|parent| retained.contains_key(&parent));
                            if !replaces && inflight.len() + ready.len() + retained.len() >= width {
                                continue;
                            }
                            let staged = parent.and_then(|parent| retained.remove(&parent));
                            let requested = &definition.points[index];
                            let child_cancel = &work_cancel;
                            inflight.insert(index);
                            frontier.remove(&index);
                            pending.push(async move {
                                let operation = self.prepare_bound_operation(&requested.operation,
                                    &requested.binding, child_cancel).await.and_then(|operation| {
                                    if operation.seed_need() == point.seed_need { Ok(operation) }
                                    else { Err(study_error(DiagnosticRule::StudySeedIncompatible,
                                        "declared seed need differs from operation-owned admission")) }
                                });
                                LaneCompletion::Prepared { index, operation, staged }
                            }.boxed());
                        }
                        ActionKind::Wait(_) => { frontier.remove(&index); }
                        ActionKind::Reconcile => return Err(WorkflowError::Internal(
                            "an ephemeral study has no publication effect to reconcile".into())),
                        kind @ (ActionKind::Cancel | ActionKind::Refuse(_)) => {
                            frontier.remove(&index);
                            if let Some(mut preparation) = ready.remove(&index)
                                && let Some(staged) = preparation.staged.take() {
                                staged.close().await;
                            }
                            facts[index].revision += 1;
                            if let ActionKind::Refuse(refusal) = kind {
                                facts[index].lifecycle = StudyPointState::Failed;
                                outcomes[index].lifecycle = StudyPointState::Failed;
                                let mut diagnostic = policy_refusal(&refusal);
                                if let Some(cause) = outcomes[index].diagnostic.take() {
                                    diagnostic.causes.push(cause);
                                }
                                outcomes[index].diagnostic = Some(diagnostic);
                            } else {
                                facts[index].lifecycle = StudyPointState::Cancelled;
                                outcomes[index].lifecycle = StudyPointState::Cancelled;
                                outcomes[index].diagnostic = Some(WorkflowError::from(
                                    crate::math::MathRuntimeError::Cancelled).boundary_diagnostic());
                            }
                            if let Some(children) = dependents.get(&point.key) { frontier.extend(children); }
                        }
                    }
                }
                // Group only already-ready fresh steps whose owning adapter supports
                // batching. All other cases execute in independent sequence futures.
                let mut groups = Vec::<(Option<pse_backend_native::solve::Backend>, Vec<(usize, StartProvenance)>)>::new();
                for (index, start) in starts {
                    let batching = if matches!(start, StartProvenance::Fresh
                        | StartProvenance::FreshFallback { .. }) && !successors.contains_key(&index) {
                        match ready.get(&index) {
                            Some(ReadyOperation { operation: PreparedStudyOperation::DeclaredCase(case),
                                staged: None }) => case.solve.batch_backend(),
                            _ => None,
                        }
                    } else { None };
                    if let Some(backend) = batching
                        && let Some((_, group)) = groups.iter_mut().find(|(owner, _)| *owner == Some(backend)) {
                        group.push((index, start)); continue;
                    }
                    groups.push((batching, vec![(index, start)]));
                }
                for (_, group) in groups {
                    let mut members = Vec::new();
                    for (index, start) in group {
                        let preparation = ready.remove(&index).ok_or_else(||
                            WorkflowError::Internal("study ready operation absent".into()))?;
                        facts[index].revision += 1;
                        facts[index].attempt_count += 1;
                        facts[index].lifecycle = StudyPointState::Assigned;
                        inflight.insert(index);
                        let (previous, older) = match &start {
                            StartProvenance::Continuation { predecessor, .. } => {
                                let parent = positions[predecessor];
                                let older = match &graph.points[parent].start {
                                    StartPolicy::Continuation(edge) => results[positions[&edge.predecessor]].clone(),
                                    _ => None,
                                };
                                (results[parent].clone(), older)
                            }
                            _ => (None, None),
                        };
                        members.push(LaneMember { index, start,
                            attempt: facts[index].attempt_count - 1, preparation, previous, older });
                    }
                    let child_cancel = &work_cancel;
                    pending.push(execute_lane(self, members, child_cancel).boxed());
                }
                if pending.is_empty() {
                    if !frontier.is_empty() { continue; }
                    // The pure policy owns blocked, terminal and cancelled conclusions.
                    break admitted.decision(&facts, work_cancel.token().is_cancelled())
                        .map_err(policy_error);
                }
                let next = tokio::select! {
                    value = pending.next() => value,
                    () = cancel.cancelled(), if !work_cancel.token().is_cancelled() => {
                        work_cancel.cancel();
                        frontier.extend(0..facts.len());
                        continue;
                    }
                };
                if let Some(value) = next { completed.push(value); }
            }
        }).await;
        // A fatal adapter error also stops issuance and drains every issued future.
        // Native teardown stays owned through join; no return drops running lanes.
        if decision.is_err() {
            work_cancel.cancel();
        }
        for completion in completed {
            completion.close().await;
        }
        while let Some(completion) = pending.next().await {
            completion.close().await;
        }
        for (_, mut preparation) in ready {
            if let Some(staged) = preparation.staged.take() {
                staged.close().await;
            }
        }
        for (_, staged) in retained {
            staged.close().await;
        }
        Ok(StudyReport {
            run_id,
            definition: definition.clone(),
            outcomes,
            results: results
                .into_iter()
                .map(|result| result.map(StudyOccurrenceResult::Ephemeral))
                .collect(),
            decision: decision?,
            preparations,
            runtime: self.runtime.clone(),
            _owner: owner,
        })
    }
}

struct ReadyOperation {
    operation: PreparedStudyOperation,
    staged: Option<Staged>,
}
struct LaneMember {
    index: usize,
    start: StartProvenance,
    attempt: u32,
    preparation: ReadyOperation,
    previous: Option<Arc<RunResult>>,
    older: Option<Arc<RunResult>>,
}
struct ExecutedPoint {
    index: usize,
    start: StartProvenance,
    result: Result<Arc<RunResult>, WorkflowError>,
}
enum LaneCompletion {
    Prepared {
        index: usize,
        operation: Result<PreparedStudyOperation, WorkflowError>,
        staged: Option<Staged>,
    },
    Executed {
        points: Vec<ExecutedPoint>,
        staged: Option<Staged>,
    },
}
impl LaneCompletion {
    async fn close(self) {
        let staged = match self {
            Self::Prepared { staged, .. } | Self::Executed { staged, .. } => staged,
        };
        if let Some(staged) = staged {
            staged.close().await;
        }
    }
}

async fn execute_lane(
    package: &ModelingPackage,
    mut members: Vec<LaneMember>,
    cancel: &crate::CancelSource,
) -> LaneCompletion {
    if members.len() == 1 {
        let mut member = members.remove(0);
        let mut staged = member.preparation.staged.take();
        let result = async {
            if let (
                StartProvenance::Continuation { .. },
                PreparedStudyOperation::DeclaredCase(target),
            ) = (&member.start, &mut member.preparation.operation)
            {
                let older = member
                    .older
                    .as_deref()
                    .and_then(|result| result.report().ok())
                    .and_then(|report| match report {
                        RunReport::Modeling(points) if points.len() == 1 => points.first(),
                        _ => None,
                    });
                continuation_start(package, target, member.previous.as_deref(), older, cancel)
                    .await?;
            }
            match member.preparation.operation {
                PreparedStudyOperation::DeclaredCase(case) => {
                    if staged.is_none() {
                        let deadline = case
                            .solve
                            .task_scope()
                            .and_then(|scope| scope.deadline())
                            .or_else(|| {
                                std::time::Instant::now().checked_add(case.solve.time_limit())
                            });
                        staged = Some(
                            Staged::open_queued(&package.runtime, None, cancel, deadline).await?,
                        );
                    }
                    execute(
                        package,
                        staged.as_mut().ok_or_else(|| {
                            WorkflowError::Internal("study native sequence absent".into())
                        })?,
                        PreparedStudyOperation::DeclaredCase(case),
                        member.attempt,
                        cancel,
                    )
                    .await
                }
                operation => {
                    operation
                        .start(&package.runtime, cancel)
                        .await?
                        .wait()
                        .await
                }
            }
        }
        .await;
        return LaneCompletion::Executed {
            points: vec![ExecutedPoint {
                index: member.index,
                start: member.start,
                result,
            }],
            staged,
        };
    }
    let mut points = Vec::with_capacity(members.len());
    let deadline = members
        .iter()
        .filter_map(|member| match &member.preparation.operation {
            PreparedStudyOperation::DeclaredCase(case) => case
                .solve
                .task_scope()
                .and_then(|scope| scope.deadline())
                .or_else(|| std::time::Instant::now().checked_add(case.solve.time_limit())),
            _ => None,
        })
        .min();
    match Staged::open_queued(&package.runtime, None, cancel, deadline).await {
        Ok(mut staged) => {
            let preparations = members
                .iter()
                .filter_map(|member| match &member.preparation.operation {
                    PreparedStudyOperation::DeclaredCase(case) => Some(case.as_ref().clone()),
                    _ => None,
                })
                .collect::<Vec<_>>();
            let solved = staged
                .batch(&preparations, Obligations::Final, cancel)
                .await;
            for ((member, preparation), result) in members.into_iter().zip(preparations).zip(solved)
            {
                let request = RunRequest::Modeling(vec![preparation]);
                let report = result.map(|result| RunReport::Modeling(vec![result]));
                let result = RunResult::joined(
                    pse_operations::mint_id(),
                    package.runtime.clone(),
                    request,
                    None,
                    report,
                )
                .finished(None, cancel.token().is_cancelled())
                .await;
                points.push(ExecutedPoint {
                    index: member.index,
                    start: member.start,
                    result: Ok(Arc::new(result)),
                });
            }
            staged.close().await;
        }
        Err(error) => {
            let error = Arc::new(error);
            for member in members {
                points.push(ExecutedPoint {
                    index: member.index,
                    start: member.start,
                    result: Err(WorkflowError::Shared(error.clone())),
                });
            }
        }
    }
    LaneCompletion::Executed {
        points,
        staged: None,
    }
}

fn record_preparation_failure(
    index: usize,
    error: &WorkflowError,
    definition: &StudyDefinition,
    facts: &mut [PointFacts],
    outcomes: &mut [PointOutcome],
) {
    let diagnostic = error
        .boundary_diagnostic()
        .with_revision(definition.points[index].operation.source.revision.as_id());
    let cancelled = diagnostic.class == BoundaryClass::Cancelled;
    facts[index].revision += 1;
    facts[index].lifecycle = if cancelled {
        StudyPointState::Cancelled
    } else {
        StudyPointState::Failed
    };
    facts[index].retry_failure = Some(RetryFailure::Deterministic);
    outcomes[index].lifecycle = facts[index].lifecycle;
    outcomes[index].diagnostic = Some(diagnostic.clone());
    if !cancelled {
        outcomes[index].attempts.push(PointAttemptOutcome {
            attempt_id: None,
            lifecycle: Some(pse_model::generated::enums::AttemptState::Failed),
            diagnostic: Some(diagnostic),
            scientific: ScientificFacts::default(),
            start: None,
            effect: EffectState::Absent,
        });
    }
}

fn record_execution(
    point: ExecutedPoint,
    definition: &StudyDefinition,
    cancel: &crate::CancelSource,
    facts: &mut [PointFacts],
    outcomes: &mut [PointOutcome],
    results: &mut [Option<Arc<RunResult>>],
) -> Option<WorkflowError> {
    let ExecutedPoint {
        index,
        start,
        result,
    } = point;
    let (scientific, diagnostic) = match &result {
        Ok(result) => (
            super::study_operations::scientific_facts(result),
            result_diagnostic(result).map(|diagnostic| {
                diagnostic.with_revision(definition.points[index].operation.source.revision.as_id())
            }),
        ),
        Err(error) => (
            ScientificFacts::default(),
            Some(
                error
                    .boundary_diagnostic()
                    .with_revision(definition.points[index].operation.source.revision.as_id()),
            ),
        ),
    };
    let cancelled = cancel.token().is_cancelled()
        || diagnostic
            .as_ref()
            .is_some_and(|diagnostic| diagnostic.class == BoundaryClass::Cancelled);
    let completed = result.as_ref().is_ok_and(|result| result.report().is_ok());
    let lifecycle = if cancelled {
        StudyPointState::Cancelled
    } else if completed {
        StudyPointState::Completed
    } else {
        StudyPointState::Failed
    };
    facts[index].lifecycle = lifecycle;
    facts[index].scientific = scientific.clone();
    facts[index].retry_failure = Some(RetryFailure::Deterministic);
    outcomes[index].lifecycle = lifecycle;
    outcomes[index].scientific = scientific.clone();
    outcomes[index].diagnostic = diagnostic.clone();
    outcomes[index].start = Some(start.clone());
    outcomes[index].attempts.push(PointAttemptOutcome {
        attempt_id: None,
        lifecycle: Some(if cancelled {
            pse_model::generated::enums::AttemptState::Cancelled
        } else if completed {
            if scientific.usable {
                pse_model::generated::enums::AttemptState::Completed
            } else {
                pse_model::generated::enums::AttemptState::Partial
            }
        } else {
            pse_model::generated::enums::AttemptState::Failed
        }),
        scientific,
        diagnostic,
        start: Some(start),
        effect: EffectState::Absent,
    });
    match result {
        Ok(result) => {
            let fatal = result
                .report
                .as_ref()
                .err()
                .or_else(|| result.completion.as_ref().err())
                .filter(|error| {
                    matches!(
                        error.boundary_diagnostic().class,
                        BoundaryClass::Infrastructure | BoundaryClass::Internal
                    )
                })
                .map(|error| WorkflowError::Shared(error.clone()));
            results[index] = Some(result);
            fatal
        }
        Err(error)
            if matches!(
                error.boundary_diagnostic().class,
                BoundaryClass::Infrastructure | BoundaryClass::Internal
            ) =>
        {
            Some(error)
        }
        Err(_) => None,
    }
}

pub(in crate::workflow) async fn continuation_start(
    package: &ModelingPackage,
    target: &mut Box<super::ModelingSolvePreparation>,
    predecessor: Option<&RunResult>,
    older: Option<&super::ModelingResult>,
    cancel: &crate::CancelSource,
) -> Result<(), WorkflowError> {
    let predecessor_result = predecessor;
    let predicted = if target
        .solve
        .composition_request()
        .recovery
        .contains(&pse_model::strategy::StartOrigin::Predicted)
        && target.solve.numerical_strategy().start.policy
            != pse_backend_native::solve::StartPolicy::Explicit
    {
        if let Some(RunReport::Modeling(points)) =
            predecessor_result.and_then(|result| result.report().ok())
            && let [point] = points.as_slice()
        {
            let deadline = std::time::Instant::now()
                .checked_add(target.solve.time_limit())
                .ok_or_else(|| super::contract("study target deadline extent"))?;
            let scope = target.solve.task_scope().unwrap_or_else(|| {
                pse_kernels::ExecutionScope::new(Arc::default(), Some(deadline))
            });
            let mut execution = pse_backend_native::solve::Execution::within(
                scope.cancellation().clone(),
                &pse_backend_native::solve::Controls::default(),
                scope.clone(),
            )
            .map_err(crate::math::MathRuntimeError::from)?;
            let branch = target.solve.composition_request().branch;
            target.solve = package
                .runtime
                .native()
                .admit_proposal_task(target.solve.clone(), scope.clone())
                .map_err(crate::math::MathRuntimeError::from)?;
            execution.work_admission = target
                .solve
                .task_admission()
                .map(|owner| -> Arc<dyn pse_backend_native::solve::WorkAdmission> { owner });
            let proposal = point.available_prediction(older, target, branch, &execution);
            match proposal {
                Ok(proposal) => {
                    let screened = package
                        .runtime
                        .native()
                        .screen_start(target.solve.clone(), proposal, branch, scope, cancel)
                        .await?;
                    Some(
                        target
                            .solve
                            .clone()
                            .with_screened_start(&screened)
                            .map_err(crate::math::MathRuntimeError::from)?,
                    )
                }
                Err(error)
                    if matches!(
                        error.boundary_diagnostic().class,
                        BoundaryClass::Unsupported
                            | BoundaryClass::Incompatible
                            | BoundaryClass::Numerical
                    ) =>
                {
                    None
                }
                Err(error) => return Err(error),
            }
        } else {
            None
        }
    } else {
        None
    };
    if let Some(solve) = predicted {
        target.solve = solve;
    } else if target.solve.numerical_strategy().start.policy
        != pse_backend_native::solve::StartPolicy::Explicit
    {
        let source = predecessor.and_then(modeling_seed).ok_or_else(|| {
            WorkflowError::Internal("policy selected a missing admitted seed".into())
        })?;
        let owner = package
            .runtime
            .native()
            .reserve("study:continuation-seed", source.owned_extent()?)?;
        let (seed, owner) = source.materialize(owner)?.into_owned()?;
        target.solve = target
            .solve
            .clone()
            .with_start_owned(seed, owner.clone())
            .map_err(crate::math::MathRuntimeError::from)?;
        target.profile.controls.start = pse_backend_native::solve::StartPolicy::Explicit;
        target.stored_seed_owner = Some(owner);
    }
    Ok(())
}

async fn execute(
    package: &ModelingPackage,
    staged: &mut Staged,
    operation: PreparedStudyOperation,
    attempt: u32,
    cancel: &crate::CancelSource,
) -> Result<Arc<RunResult>, WorkflowError> {
    match operation {
        PreparedStudyOperation::DeclaredCase(case) => {
            let run_id = pse_operations::mint_id();
            let request = RunRequest::Modeling(vec![case.as_ref().clone()]);
            let report = staged
                .run(
                    *case,
                    Obligations::Final,
                    run_id,
                    attempt as usize,
                    None,
                    cancel,
                )
                .await
                .map(|result| RunReport::Modeling(vec![result]));
            Ok(Arc::new(
                RunResult::joined(run_id, package.runtime.clone(), request, None, report)
                    .finished(None, cancel.token().is_cancelled())
                    .await,
            ))
        }
        operation => {
            operation
                .start(&package.runtime, cancel)
                .await?
                .wait()
                .await
        }
    }
}
fn scoped_action(
    admitted: &pse_operations::study_policy::AdmittedStudy,
    key: OccurrenceKey,
    facts: &[PointFacts],
    positions: &BTreeMap<OccurrenceKey, usize>,
    cancelled: bool,
) -> Result<PointAction, WorkflowError> {
    let mut scope = Vec::with_capacity(admitted.predecessors(key).map_err(policy_error)?.len() + 1);
    scope.push(facts[positions[&key]].clone());
    for predecessor in admitted.predecessors(key).map_err(policy_error)? {
        scope.push(facts[positions[predecessor]].clone());
    }
    admitted
        .action(key, &scope, cancelled)
        .map_err(policy_error)
}

/// Allocation-free view of a completion-permitted original seed.
pub(in crate::workflow) struct CompletedModelingSeed<'a> {
    point: &'a super::ModelingResult,
    compatibility: &'a pse_backend_native::solve::Compatibility,
}
/// A materialized seed never loses its pre-allocation pool grant. In-memory
/// consumers transfer that grant to the target preparation's existing seed owner.
pub(in crate::workflow) struct LeasedModelingSeed<'a> {
    seed: std::borrow::Cow<'a, pse_backend_native::solve::WarmStart>,
    owner: Arc<pse_columnar::AllocationLease>,
    owned_extent: usize,
}
impl LeasedModelingSeed<'_> {
    pub(in crate::workflow) fn warm(&self) -> &pse_backend_native::solve::WarmStart {
        &self.seed
    }
    fn into_owned(
        self,
    ) -> Result<
        (
            pse_backend_native::solve::WarmStart,
            Arc<pse_columnar::AllocationLease>,
        ),
        WorkflowError,
    > {
        if self.owner.size() < self.owned_extent {
            return Err(super::contract(
                "owned seed copy was not admitted before allocation",
            ));
        }
        Ok((self.seed.into_owned(), self.owner))
    }
}
impl CompletedModelingSeed<'_> {
    pub(in crate::workflow) fn compatibility(&self) -> &pse_backend_native::solve::Compatibility {
        self.compatibility
    }
    /// Original producer bound for a copied native payload, or the exact primal
    /// population for a complete original evaluation, before any numeric allocation.
    pub(in crate::workflow) fn owned_extent(&self) -> Result<usize, WorkflowError> {
        match &self.point.outcome {
            crate::math::solves::Outcome::Native(native) => native
                .completed_report_allowance()
                .map_err(crate::math::MathRuntimeError::from)?
                .map_or_else(
                    || {
                        self.point
                            .prepared
                            .solve
                            .result_bytes()
                            .map_err(WorkflowError::from)
                    },
                    Ok,
                ),
            crate::math::solves::Outcome::Constant(completed) => completed.coordinates.len()
                .checked_mul(size_of::<f64>())
                .and_then(|n| n.checked_add(size_of::<pse_backend_native::solve::WarmStart>()))
                // PreparedSolve shares an Arc body, including strong/weak counts.
                .and_then(|n| n.checked_add(2 * size_of::<usize>()))
                .ok_or_else(|| super::contract("complete original primal seed extent")),
            crate::math::solves::Outcome::Rejected(_) => {
                Err(super::contract("rejected seed source"))
            }
        }
    }
    /// Scientific serialization extent, without constructing a primal or prediction.
    pub(in crate::workflow) fn serialization_extent(&self) -> Result<usize, WorkflowError> {
        match &self.point.outcome {
            crate::math::solves::Outcome::Native(native) => super::durable::seed_extent(
                &native
                    .warm_start
                    .as_ref()
                    .ok_or_else(|| super::contract("native seed lost its payload"))?
                    .payload,
            ),
            crate::math::solves::Outcome::Constant(completed) => {
                super::durable::seed_values_extent(completed.coordinates.len())
            }
            crate::math::solves::Outcome::Rejected(_) => {
                Err(super::contract("rejected seed source"))
            }
        }
    }
    pub(in crate::workflow) fn materialize(
        &self,
        owner: Arc<pse_columnar::AllocationLease>,
    ) -> Result<LeasedModelingSeed<'_>, WorkflowError> {
        let seed = match &self.point.outcome {
            crate::math::solves::Outcome::Native(native) => std::borrow::Cow::Borrowed(
                native
                    .warm_start
                    .as_ref()
                    .ok_or_else(|| super::contract("native seed lost its payload"))?,
            ),
            crate::math::solves::Outcome::Constant(completed) => {
                if owner.size() < self.owned_extent()? {
                    return Err(super::contract(
                        "complete original seed was not admitted before allocation",
                    ));
                }
                let payload = pse_backend_native::execution::adapter(self.compatibility.backend)
                    .primal_start(
                        completed
                            .coordinates
                            .iter()
                            .map(|(_, value)| *value)
                            .collect(),
                    )
                    .map_err(crate::math::MathRuntimeError::from)?;
                let seed = pse_backend_native::solve::WarmStart {
                    origin: Some(pse_backend_native::solve::SeedOrigin {
                        run: Some(self.point.run_id),
                        attempt: self.point.original_attempt,
                    }),
                    compatibility: self.compatibility.clone(),
                    payload,
                };
                let plan = &self.point.prepared.model.case.compiled().plan;
                seed.validate_shape(plan.columns().len(), plan.structure().rows().len())
                    .map_err(crate::math::MathRuntimeError::from)?;
                std::borrow::Cow::Owned(seed)
            }
            crate::math::solves::Outcome::Rejected(_) => {
                return Err(super::contract("rejected seed source"));
            }
        };
        Ok(LeasedModelingSeed {
            seed,
            owner,
            owned_extent: self.owned_extent()?,
        })
    }
}
/// Inspect permission, ordered original coverage and operation-owned compatibility
/// without cloning a native payload or allocating a numeric vector.
pub(in crate::workflow) fn completed_modeling_seed(
    point: &super::ModelingResult,
) -> Option<CompletedModelingSeed<'_>> {
    if !point.completion.decision.permits_seed() {
        return None;
    }
    let compatibility = match &point.outcome {
        crate::math::solves::Outcome::Native(native) => &native.warm_start.as_ref()?.compatibility,
        crate::math::solves::Outcome::Constant(completed) => {
            let compatibility = point.prepared.solve.compatibility()?;
            let columns = point.prepared.model.case.compiled().plan.columns();
            if columns.is_empty()
                || completed.coordinates.len() != columns.len()
                || completed
                    .coordinates
                    .iter()
                    .zip(columns)
                    .any(|((id, value), column)| id != column || !value.is_finite())
            {
                return None;
            }
            compatibility
        }
        crate::math::solves::Outcome::Rejected(_) => return None,
    };
    Some(CompletedModelingSeed {
        point,
        compatibility,
    })
}
fn modeling_seed(result: &RunResult) -> Option<CompletedModelingSeed<'_>> {
    match result.report().ok()? {
        RunReport::Modeling(points) => match points.as_slice() {
            [point] => completed_modeling_seed(point),
            _ => None,
        },
        _ => None,
    }
}
fn memory_seed(target: &PreparedStudyOperation, previous: Option<&RunResult>) -> SeedAvailability {
    let Some(previous) = previous else {
        return SeedAvailability::Absent;
    };
    let Some(seed) = modeling_seed(previous) else {
        return SeedAvailability::Absent;
    };
    let PreparedStudyOperation::DeclaredCase(target) = target else {
        return SeedAvailability::Incompatible;
    };
    let Some(compatibility) = target.solve.compatibility() else {
        return SeedAvailability::Absent;
    };
    if compatibility.layout != seed.compatibility().layout
        || compatibility.backend != seed.compatibility().backend
    {
        return SeedAvailability::Incompatible;
    }
    SeedAvailability::Compatible {
        seed: SolutionId::from_id(previous.run_id.as_id()),
    }
}
pub(in crate::workflow) fn result_diagnostic(result: &RunResult) -> Option<BoundaryDiagnostic> {
    match result.completion() {
        Err(error) => Some(error.boundary_diagnostic()),
        Ok(completion) => match completion.diagnostics.as_slice() {
            [] => None,
            [diagnostic] => Some(diagnostic.clone()),
            diagnostics => {
                let mut envelope = BoundaryDiagnostic::new(
                    BoundaryClass::Conflict,
                    DiagnosticStage::StudyPolicy,
                    [],
                    DiagnosticRule::DiagnosticAggregate,
                );
                envelope.causes = diagnostics.to_vec();
                Some(envelope)
            }
        },
    }
}
pub(in crate::workflow) fn policy_refusal(refusal: &Refusal) -> BoundaryDiagnostic {
    refusal.boundary_diagnostic()
}
fn policy_error(error: pse_operations::study_policy::PolicyError) -> WorkflowError {
    pse_model::diagnostic::project_typed(&error, DiagnosticStage::StudyPolicy).into()
}
fn study_error(rule: DiagnosticRule, detail: &str) -> WorkflowError {
    let mut diagnostic = BoundaryDiagnostic::new(
        BoundaryClass::Incompatible,
        DiagnosticStage::StudyAdmission,
        [],
        rule,
    );
    diagnostic
        .observations
        .insert("detail".into(), Observation::Text(detail.into()));
    diagnostic.into()
}

#[cfg(all(test, feature = "solver-kinsol"))]
#[allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "native occurrence controls fail on invalid setup or unexpected fixture variants"
)]
mod occurrence_execution_tests {
    use super::*;
    use crate::workflow::tests::{compiler_profile, physical, runtime_with};
    use crate::workflow::{
        BindingAssignment, BindingQuantity, BindingTarget, CaseOperation, OperationRequest,
        PointOverlay, PreparationSettings, StudyPoint, StudyPointPolicy,
    };
    use pse_backend_native::solve::Backend;
    use pse_ids::{ContentHash, SemanticId};
    use pse_model::generated::identities::DeclarationId;
    use pse_model::scalars::FiniteBound;

    async fn fixture(values: &[f64]) -> (ModelingPackage, StudyDefinition) {
        fixture_source(values, "package p { def Root { param t: Scalar = 1; var x: Scalar; eq e: x == 2+t; annotation start x(2+t); annotation check x(x > 0); } }", Backend::Kinsol, 1).await
    }
    async fn fixture_source(
        values: &[f64],
        source: &str,
        backend: Backend,
        threads: usize,
    ) -> (ModelingPackage, StudyDefinition) {
        fixture_source_typed(values, source, backend, threads, "Scalar", "1").await
    }
    async fn fixture_source_typed(
        values: &[f64],
        source: &str,
        backend: Backend,
        threads: usize,
        quantity_name: &str,
        unit_symbol: &str,
    ) -> (ModelingPackage, StudyDefinition) {
        let runtime = runtime_with(1 << 30, 1 << 20, 8 << 30);
        let rows = pse_authoring::language::parse(
            source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime.modeling_package(rows, physical()).await.unwrap();
        let quantity = package
            .quantities
            .quantity_types()
            .find(|q| q.name.as_deref() == Some(quantity_name))
            .unwrap()
            .id
            .as_id();
        let unit = package
            .quantities
            .units()
            .find(|unit| unit.symbol == unit_symbol)
            .unwrap()
            .id
            .as_id();
        let points: Vec<_> = values
            .iter()
            .enumerate()
            .map(|(index, value)| StudyPoint {
                operation: OperationRequest::DeclaredCase(CaseOperation {
                    case: root,
                    route: pse_model::generated::enums::ModelingAnalysisRoute::Steady,
                    settings: crate::math::settings::SolveSettings {
                        backend: Some(backend),
                        intent: if backend == Backend::PounceConvex {
                            pse_backend_native::solve::SolveIntent::Optimize
                        } else {
                            pse_backend_native::solve::SolveIntent::Root
                        },
                        controls: pse_backend_native::solve::Controls {
                            threads,
                            ..crate::workflow::tests::profile().controls
                        },
                        ..Default::default()
                    },
                }),
                preparation: PreparationSettings {
                    compiler: compiler_profile(),
                    ..Default::default()
                },
                overlay: PointOverlay {
                    assignments: vec![BindingAssignment {
                        target: BindingTarget::Path("t".into()),
                        value: BindingQuantity {
                            magnitude: FiniteBound::try_new(*value).unwrap(),
                            quantity,
                            unit,
                        },
                    }],
                },
                policy: StudyPointPolicy {
                    key: OccurrenceKey(index as u32),
                    dependencies: vec![],
                    start: StartPolicy::Fresh,
                    attempt_limit: 1,
                },
            })
            .collect();
        let definition = package
            .admit_study_points(
                super::super::PhysicalSource {
                    revision: "explicit-ephemeral-fixture".into(),
                    identity: ContentHash::from_bytes([0; 32]),
                },
                &points,
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        (package, definition)
    }
    fn result(report: &StudyReport, index: usize) -> &super::super::ModelingResult {
        match report.results[index].as_ref().unwrap().report().unwrap() {
            RunReport::Modeling(results) => &results[0],
            other => panic!("{other:?}"),
        }
    }
    fn member(point: &super::super::ModelingResult, name: &str) -> SemanticId {
        point
            .prepared
            .model
            .model
            .compiled()
            .model
            .symbols
            .values()
            .find(|symbol| symbol.lineage.path == format!("Root.{name}"))
            .unwrap()
            .id
    }
    #[tokio::test]
    async fn equal_bindings_are_distinct_and_failed_dependencies_keep_typed_outcomes() {
        let (package, mut definition) = fixture(&[1., 1., -5., 2., 3., 4.]).await;
        assert_eq!(
            definition.points[0].binding_hash,
            definition.points[1].binding_hash
        );
        assert!(
            definition.points.iter().all(|point| point
                .binding
                .entries
                .values()
                .all(|entry| entry.parameter))
        );
        definition.points[4].policy.dependencies = vec![Dependency::UsableResult(OccurrenceKey(2))];
        definition.points[5].policy.dependencies = vec![Dependency::Ordering(OccurrenceKey(2))];
        let report = package
            .study(&definition, 8, &crate::CancelSource::new())
            .await
            .unwrap();
        assert_eq!(report.outcomes.len(), 6);
        assert_ne!(
            report.results[0].as_ref().unwrap().run_id(),
            report.results[1].as_ref().unwrap().run_id()
        );
        assert!(report.outcomes[0].scientific.usable && report.outcomes[1].scientific.usable);
        assert!(!report.outcomes[2].scientific.usable);
        assert!(report.outcomes[3].scientific.usable && report.outcomes[5].scientific.usable);
        assert_eq!(report.outcomes[4].lifecycle, StudyPointState::Failed);
        assert_eq!(
            report.outcomes[4].diagnostic.as_ref().unwrap().rule,
            DiagnosticRule::StudyDependencyUnusable
        );
        assert!(report.outcomes[4].attempts.is_empty());
        assert!(report.results[4].is_none());
        assert!(report.table().unwrap().batch().num_rows() == 6);
        assert!(report.findings_table().unwrap().batch().num_rows() >= 2);
        let parameter = member(result(&report, 3), "t");
        assert_eq!(
            result(&report, 3).prepared.model.values.scalars[&parameter],
            2.
        );
        let variable = member(result(&report, 3), "x");
        let point = result(&report, 3);
        let target = super::super::tests::engineering_target(
            point.prepared.solve.numerics(),
            pse_relations::generated::enums::NumericalTarget::Variable,
            variable,
        );
        assert!(
            (point.values.scalars[&variable] - 4.).abs()
                <= target.engineering.as_ref().unwrap().budget
        );
    }
    #[tokio::test]
    async fn precancellation_retains_every_occurrence_without_native_dispatch() {
        let (package, definition) = fixture(&[1., 2., 3.]).await;
        let cancel = crate::CancelSource::new();
        cancel.cancel();
        let report = package.study(&definition, 8, &cancel).await.unwrap();
        assert_eq!(report.outcomes.len(), 3);
        assert!(report.outcomes.iter().all(|outcome| {
            outcome.lifecycle == StudyPointState::Cancelled
                && outcome.attempts.is_empty()
                && outcome
                    .diagnostic
                    .as_ref()
                    .is_some_and(|d| d.class == BoundaryClass::Cancelled)
        }));
        assert!(report.results.iter().all(Option::is_none));
        assert_eq!(report.preparations.views, 0);
    }
    #[tokio::test]
    async fn preparation_failure_has_an_envelope_and_does_not_stop_independent_points() {
        let (package, mut definition) = fixture(&[1., 2.]).await;
        let OperationRequest::DeclaredCase(case) = &mut definition.points[0].operation.operation
        else {
            panic!("fixture must retain its declared case operation")
        };
        case.case = DeclarationId::from_bytes([255; 16]);
        let report = package
            .study(&definition, 8, &crate::CancelSource::new())
            .await
            .unwrap();
        assert_eq!(report.outcomes[0].lifecycle, StudyPointState::Failed);
        assert!(report.outcomes[0].diagnostic.is_some());
        assert_eq!(report.outcomes[0].attempts.len(), 1);
        assert_eq!(
            report.outcomes[0].attempts[0].lifecycle,
            Some(pse_model::generated::enums::AttemptState::Failed)
        );
        assert!(!report.outcomes[0].attempts[0].scientific.usable);
        assert!(report.results[0].is_none());
        assert!(report.outcomes[1].scientific.usable);
    }
    #[tokio::test]
    async fn preparation_reuse_is_bounded_as_occurrence_count_grows() {
        let (package, small) = fixture(&[1., 2.]).await;
        let (large_package, large) = fixture(&[1., 2., 3., 4., 5., 6., 7., 8.]).await;
        let before = package.runtime.native().preparations();
        for maximum in [0, 1] {
            assert!(
                package
                    .study(&small, maximum, &crate::CancelSource::new())
                    .await
                    .is_err()
            );
        }
        assert_eq!(package.runtime.native().preparations(), before);
        let small_report = package
            .study(&small, 8, &crate::CancelSource::new())
            .await
            .unwrap();
        let large_report = large_package
            .study(&large, 8, &crate::CancelSource::new())
            .await
            .unwrap();
        for report in [&small_report, &large_report] {
            assert!(
                report.outcomes.iter().all(|point| point.scientific.usable),
                "{:?}",
                report.outcomes
            );
            assert!(report.preparations.views <= 1, "{:?}", report.preparations);
            assert!(
                report.preparations.observations <= 2,
                "{:?}",
                report.preparations
            );
            assert!(
                report.preparations.views
                    + report.preparations.shared
                    + report.preparations.rebuilt
                    >= report.outcomes.len(),
                "{:?}",
                report.preparations
            );
        }
        assert!(large_report.preparations.views <= small_report.preparations.views + 1);
    }
    #[tokio::test]
    async fn simultaneous_studies_count_only_their_own_preparations() {
        let (package, first) = fixture(&[1., 2., 3., 4.]).await;
        let mut second = first.clone();
        second.points.truncate(2);
        let before = package.runtime.native().preparations();
        let cancel = crate::CancelSource::new();
        let (a, b) = tokio::join!(
            package.study(&first, 8, &cancel),
            package.study(&second, 8, &cancel)
        );
        let (a, b) = (a.unwrap(), b.unwrap());
        assert!(
            a.outcomes
                .iter()
                .chain(&b.outcomes)
                .all(|point| point.scientific.usable),
            "first={:?}; second={:?}",
            a.outcomes,
            b.outcomes
        );
        assert_ne!(a.run_id, b.run_id);
        let after = package.runtime.native().preparations();
        assert_eq!(
            a.preparations.views + b.preparations.views,
            after.views - before.views
        );
        assert_eq!(
            a.preparations.observations + b.preparations.observations,
            after.observations - before.observations
        );
        assert_eq!(
            a.preparations.shared + b.preparations.shared,
            after.shared - before.shared
        );
        assert_eq!(
            a.preparations.rebuilt + b.preparations.rebuilt,
            after.rebuilt - before.rebuilt
        );
        assert!(a.preparations.views + a.preparations.shared + a.preparations.rebuilt >= 4);
        assert!(b.preparations.views + b.preparations.shared + b.preparations.rebuilt >= 2);
    }
    #[cfg(feature = "solver-pounce")]
    const POUNCE_OCCURRENCE_SOURCE: &str = "package p { def Root { param t: Scalar = 1; param b: Scalar = 0.4; var x: Scalar; var y: Scalar; eq budget: x + y <= 1; let cost: Scalar = (x-t)*(x-t)+(y-b)*(y-b); annotation objective cost(minimize); annotation bounds x(0,5); annotation bounds y(0,5); annotation start x(0); annotation start y(0); } }";

    #[cfg(feature = "solver-pounce")]
    fn assert_pounce_occurrence(
        definition: &StudyDefinition,
        index: usize,
        run: &RunResult,
    ) -> usize {
        use pse_backend_native::solve::Metric;
        assert!(
            run.usable(),
            "occurrence {index} is not scientifically usable"
        );
        let RunReport::Modeling(points) = run.report().unwrap() else {
            panic!("expected a modeling occurrence");
        };
        assert_eq!(points.len(), 1);
        let point = &points[0];
        let crate::math::solves::Outcome::Native(native) = &point.outcome else {
            panic!("{:?}", point.outcome)
        };
        assert_eq!(native.backend, Backend::PounceConvex);
        let Some(Metric::Integer(batch)) = native.metrics.get("batch") else {
            panic!("expected the native batch population");
        };
        let batch = usize::try_from(*batch).unwrap();
        assert!(batch > 0);
        let (a, b) = (
            definition.points[index].binding.values()[&member(point, "t")],
            0.4,
        );
        let excess = (a + b - 1.).max(0.) / 2.;
        let (x, y) = (
            point.values.scalars[&member(point, "x")],
            point.values.scalars[&member(point, "y")],
        );
        let actual_cost = (x - a).powi(2) + (y - b).powi(2);
        let optimal_cost = 2. * excess.powi(2);
        let target = super::super::tests::engineering_target(
            point.prepared.solve.numerics(),
            pse_relations::generated::enums::NumericalTarget::Objective,
            SemanticId::NIL,
        );
        assert!(
            (actual_cost - optimal_cost).abs() <= target.engineering.as_ref().unwrap().budget,
            "point {index}: cost={actual_cost}, optimum={optimal_cost}, x={x}, y={y}"
        );
        batch
    }

    #[cfg(feature = "solver-pounce")]
    #[tokio::test]
    async fn occurrence_adapter_reaches_existing_pounce_batch_owner() {
        let (package, definition) = fixture_source(
            &[0.25, 0.5, 0.75, 1.],
            POUNCE_OCCURRENCE_SOURCE,
            Backend::PounceConvex,
            2,
        )
        .await;
        let width = package
            .runtime
            .shared
            .math()
            .cores()
            .min(package.runtime.shared.budget().math.jobs / 2)
            .max(1);
        let report = package
            .study(&definition, 8, &crate::CancelSource::new())
            .await
            .unwrap();
        assert_eq!(report.outcomes.len(), 4);
        for (index, outcome) in report.outcomes.iter().enumerate() {
            assert!(outcome.scientific.usable, "{index}: {outcome:?}");
            let StudyOccurrenceResult::Ephemeral(run) = report.results[index].as_ref().unwrap()
            else {
                panic!("expected an ephemeral occurrence");
            };
            let batch = assert_pounce_occurrence(&definition, index, run);
            // The scheduler groups already-ready preparations without waiting for
            // the whole frontier, so native batches respect the admitted lane width.
            assert!(
                batch <= width.min(report.outcomes.len()),
                "batch {batch}, width {width}"
            );
        }
    }

    #[cfg(feature = "solver-pounce")]
    #[tokio::test]
    async fn four_ready_occurrences_reach_existing_pounce_batch_owner() {
        let (package, definition) = fixture_source(
            &[0.25, 0.5, 0.75, 1.],
            POUNCE_OCCURRENCE_SOURCE,
            Backend::PounceConvex,
            2,
        )
        .await;
        let cancel = crate::CancelSource::new();
        let mut members = Vec::new();
        for (index, point) in definition.points.iter().enumerate() {
            let operation = package
                .prepare_bound_operation(&point.operation, &point.binding, &cancel)
                .await
                .unwrap();
            members.push(LaneMember {
                index,
                start: StartProvenance::Fresh,
                attempt: 0,
                preparation: ReadyOperation {
                    operation,
                    staged: None,
                },
                previous: None,
                older: None,
            });
        }
        // Supply one fully prepared cohort to the same runtime adapter used by the
        // study scheduler, without making preparation readiness depend on timing.
        let LaneCompletion::Executed { points, staged } =
            execute_lane(&package, members, &cancel).await
        else {
            panic!("expected completed occurrences");
        };
        assert!(staged.is_none());
        assert_eq!(points.len(), 4);
        for (index, point) in points.into_iter().enumerate() {
            assert_eq!(point.index, index);
            assert!(matches!(point.start, StartProvenance::Fresh));
            let run = point.result.unwrap();
            assert_eq!(assert_pounce_occurrence(&definition, index, &run), 4);
        }
    }

    #[tokio::test]
    async fn canonical_parameter_overlay_replaces_authored_default_once() {
        let source = "package p { def Root { param t: Temperature = 300{K}; var x: Temperature; eq e: x == t; annotation start x(t); } }";
        let (package, definition) = fixture_source_typed(
            &[80., 26.85],
            source,
            Backend::Kinsol,
            1,
            "Temperature",
            "degC",
        )
        .await;
        assert!(
            definition.points.iter().all(|point| point
                .binding
                .entries
                .values()
                .all(|entry| entry.parameter))
        );
        let report = package
            .study(&definition, 8, &crate::CancelSource::new())
            .await
            .unwrap();
        for (index, expected) in [353.15, 300.].into_iter().enumerate() {
            assert!(
                report.outcomes[index].scientific.usable,
                "{:?}",
                report.outcomes[index]
            );
            let point = result(&report, index);
            assert!(
                (point.prepared.model.values.scalars[&member(point, "t")] - expected).abs() < 1e-10
            );
            let variable = member(point, "x");
            let target = super::super::tests::engineering_target(
                point.prepared.solve.numerics(),
                pse_relations::generated::enums::NumericalTarget::Variable,
                variable,
            );
            assert!(
                (point.values.scalars[&variable] - expected).abs()
                    <= target.engineering.as_ref().unwrap().budget
            );
        }
    }
}
