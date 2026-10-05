// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! In-process adapter of the same occurrence policy used by durable workers.
use super::modeling::assessment::Obligations;
use super::staged::Staged;
use super::{
    ModelingPackage, PreparedStudyOperation, RunReport, RunRequest, RunResult, StudyDefinition,
    WorkflowError,
};
use pse_diagnostics::{DiagnosticRule, DiagnosticStage};
use pse_model::diagnostic::{BoundaryClass, BoundaryDiagnostic, Observation};
use pse_model::generated::{
    enums::StudyPointState,
    identities::{RunId, SolutionId},
};
use pse_model::study::*;
use std::{collections::BTreeMap, sync::Arc};

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
    pub results: Vec<Option<Arc<RunResult>>>,
    /// Shared conclusion preserves lifecycle separately from scientific availability.
    pub decision: StudyDecision,
    /// Existing preparation/session reuse remains owned by the mathematical service.
    pub preparations: crate::math::PreparationCounts,
    pub(in crate::workflow) runtime: super::Runtime,
    pub(in crate::workflow) _owner: Arc<pse_columnar::AllocationLease>,
}

impl ModelingPackage {
    /// Execute the admitted definition through existing operation owners and pure policy.
    pub async fn study(
        &self,
        definition: &StudyDefinition,
        maximum_points: usize,
        cancel: &crate::CancelSource,
    ) -> Result<StudyReport, WorkflowError> {
        let (report, preparations) =
            crate::math::counted(self.study_inner(definition, maximum_points, cancel)).await;
        let mut report = report?;
        report.preparations = preparations;
        Ok(report)
    }
    async fn study_inner(
        &self,
        definition: &StudyDefinition,
        maximum_points: usize,
        cancel: &crate::CancelSource,
    ) -> Result<StudyReport, WorkflowError> {
        if maximum_points == 0
            || maximum_points > super::MAXIMUM_STUDY_POINTS
            || definition.points.len() > maximum_points
        {
            return Err(super::contract("bounded study extent"));
        }
        let graph = definition.graph();
        pse_operations::study_policy::admit(&graph).map_err(policy_error)?;
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
        let mut prepared = Vec::with_capacity(definition.points.len());
        let mut preparation_failures = Vec::with_capacity(definition.points.len());
        for point in &definition.points {
            if point.binding_hash != point.binding.identity() {
                return Err(study_error(
                    DiagnosticRule::StudyBindingRevision,
                    "binding content identity differs from the recorded binding",
                ));
            }
            if let StartPolicy::Continuation(edge) = &point.policy.start {
                point.operation.admit_seed_role(edge.role)?;
                let predecessor = &definition.points[positions[&edge.predecessor]];
                predecessor.operation.admit_seed_role(edge.role)?;
            } else if let StartPolicy::Explicit { role, .. } = point.policy.start {
                point.operation.admit_seed_role(role)?;
            }
            let operation = self
                .prepare_bound_operation(&point.operation, &point.binding, cancel)
                .await;
            let operation = operation.and_then(|operation| {
                if operation.seed_need() != point.policy.seed_need {
                    Err(study_error(
                        DiagnosticRule::StudySeedIncompatible,
                        "declared seed need differs from operation-owned admission",
                    ))
                } else {
                    Ok(operation)
                }
            });
            match operation {
                Ok(operation) => {
                    prepared.push(Some(operation));
                    preparation_failures.push(None);
                }
                Err(error) => {
                    prepared.push(None);
                    preparation_failures.push(Some(
                        error
                            .boundary_diagnostic()
                            .with_revision(point.operation.source.revision.as_id()),
                    ));
                }
            }
        }
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
                seed: None,
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
        for (index, diagnostic) in preparation_failures.into_iter().enumerate() {
            if let Some(diagnostic) = diagnostic {
                let cancelled = diagnostic.class == BoundaryClass::Cancelled;
                let lifecycle = if cancelled {
                    StudyPointState::Cancelled
                } else {
                    StudyPointState::Failed
                };
                facts[index].lifecycle = lifecycle;
                facts[index].retry_failure = Some(RetryFailure::Deterministic);
                outcomes[index].lifecycle = lifecycle;
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
        }
        let mut results: Vec<Option<Arc<RunResult>>> = vec![None; graph.points.len()];
        let mut staged = Staged::open(&self.runtime, None)?;
        let (decision, preparations) = crate::math::counted(async {
            loop {
                // Seed acquisition is an adapter fact. Policy alone decides permission/fallback.
                for (index, point) in graph.points.iter().enumerate() {
                    let Some(preparation) = prepared[index].as_ref() else {
                        continue;
                    };
                    facts[index].seed = match &point.start {
                        StartPolicy::Continuation(edge) => {
                            let predecessor = positions[&edge.predecessor];
                            Some(SeedFact {
                                role: edge.role,
                                availability: memory_seed(
                                    preparation,
                                    results[predecessor].as_deref(),
                                ),
                            })
                        }
                        StartPolicy::Explicit { role, seed } => {
                            let availability = if let (
                                PreparedStudyOperation::DeclaredCase(case),
                                super::Durability::Durable(operations),
                            ) = (preparation, self.runtime.durability())
                            {
                                match case
                                    .as_ref()
                                    .clone()
                                    .with_stored_start(
                                        operations,
                                        super::StoredStart::Solution(*seed),
                                    )
                                    .await
                                {
                                    Ok(case) => {
                                        prepared[index] = Some(
                                            PreparedStudyOperation::DeclaredCase(Box::new(case)),
                                        );
                                        SeedAvailability::Compatible { seed: *seed }
                                    }
                                    Err(WorkflowError::Operations(
                                        pse_operations::OperationsError::NotFound { .. },
                                    )) => SeedAvailability::Absent,
                                    Err(error)
                                        if error.boundary_diagnostic().class
                                            == BoundaryClass::Incompatible =>
                                    {
                                        SeedAvailability::Incompatible
                                    }
                                    Err(error) => {
                                        outcomes[index].diagnostic =
                                            Some(error.boundary_diagnostic());
                                        SeedAvailability::InternalFailure
                                    }
                                }
                            } else {
                                SeedAvailability::Absent
                            };
                            Some(SeedFact {
                                role: *role,
                                availability,
                            })
                        }
                        StartPolicy::Fresh => None,
                    };
                }
                let decision = pse_operations::study_policy::transition(
                    &graph,
                    &facts,
                    cancel.token().is_cancelled(),
                )
                .map_err(policy_error)?;
                // The mathematical owner's batch consumes the already-admitted preparations.
                // Only fresh starts can share a batch; selected continuation seeds stay explicit.
                let batch_indices: Vec<_> = decision
                    .actions
                    .iter()
                    .filter_map(|action| {
                        let index = positions[&action.occurrence];
                        if matches!(
                            action.kind,
                            ActionKind::Start(
                                StartProvenance::Fresh | StartProvenance::FreshFallback { .. }
                            )
                        ) && matches!(
                            prepared[index],
                            Some(PreparedStudyOperation::DeclaredCase(_))
                        ) {
                            Some(index)
                        } else {
                            None
                        }
                    })
                    .collect();
                let mut batched = BTreeMap::new();
                if batch_indices.len() > 1 {
                    let preparations: Vec<_> = batch_indices
                        .iter()
                        .filter_map(|index| match &prepared[*index] {
                            Some(PreparedStudyOperation::DeclaredCase(case)) => {
                                Some(case.as_ref().clone())
                            }
                            _ => None,
                        })
                        .collect();
                    let solved = staged
                        .batch(&preparations, Obligations::Final, cancel)
                        .await;
                    for ((index, preparation), result) in
                        batch_indices.into_iter().zip(preparations).zip(solved)
                    {
                        let run_id = pse_operations::mint_id();
                        let request = RunRequest::Modeling(vec![preparation]);
                        let report = result.map(|result| RunReport::Modeling(vec![result]));
                        let result =
                            RunResult::joined(run_id, self.runtime.clone(), request, None, report)
                                .finished(None, cancel.token().is_cancelled())
                                .await;
                        batched.insert(index, Ok(Arc::new(result)));
                    }
                }
                let mut changed = false;
                for action in &decision.actions {
                    let index = positions[&action.occurrence];
                    if facts[index].revision != action.expected_revision {
                        continue;
                    }
                    match &action.kind {
                        ActionKind::Wait(_) => {}
                        ActionKind::Reconcile => {
                            return Err(WorkflowError::Internal(
                                "an ephemeral study has no publication effect to reconcile".into(),
                            ));
                        }
                        ActionKind::Cancel => {
                            changed = true;
                            facts[index].revision += 1;
                            facts[index].lifecycle = StudyPointState::Cancelled;
                            outcomes[index].lifecycle = StudyPointState::Cancelled;
                            outcomes[index].diagnostic = Some(
                                WorkflowError::from(crate::math::MathRuntimeError::Cancelled)
                                    .boundary_diagnostic(),
                            );
                        }
                        ActionKind::Refuse(refusal) => {
                            changed = true;
                            facts[index].revision += 1;
                            facts[index].lifecycle = StudyPointState::Failed;
                            outcomes[index].lifecycle = StudyPointState::Failed;
                            let mut diagnostic = policy_refusal(refusal);
                            if let Some(cause) = outcomes[index].diagnostic.take() {
                                diagnostic.causes.push(cause);
                            }
                            outcomes[index].diagnostic = Some(diagnostic);
                        }
                        ActionKind::Start(start) => {
                            changed = true;
                            facts[index].revision += 1;
                            facts[index].attempt_count += 1;
                            let mut operation = prepared[index].clone().ok_or_else(|| {
                                WorkflowError::Internal(
                                    "policy started an unavailable operation".into(),
                                )
                            })?;
                            if let (
                                StartProvenance::Continuation { predecessor, .. },
                                PreparedStudyOperation::DeclaredCase(target),
                            ) = (start, &mut operation)
                            {
                                let previous = results[positions[predecessor]]
                                    .as_deref()
                                    .and_then(primal_seed)
                                    .ok_or_else(|| {
                                        WorkflowError::Internal(
                                            "policy selected a missing admitted seed".into(),
                                        )
                                    })?;
                                let predecessor_result = results[positions[predecessor]].as_deref();
                                let predicted = if target
                                    .solve
                                    .composition_request()
                                    .recovery
                                    .contains(&pse_model::strategy::StartOrigin::Predicted)
                                    && target.solve.numerical_strategy().start.policy
                                        != pse_backend_native::solve::StartPolicy::NoPriorStart
                                {
                                    if let Some(RunReport::Modeling(points)) =
                                        predecessor_result.and_then(|result| result.report().ok())
                                        && let [point] = points.as_slice()
                                    {
                                        let deadline = std::time::Instant::now()
                                            .checked_add(target.solve.time_limit())
                                            .ok_or_else(|| {
                                                super::contract("study target deadline extent")
                                            })?;
                                        let scope =
                                            target.solve.task_scope().unwrap_or_else(|| {
                                                pse_kernels::ExecutionScope::new(
                                                    Arc::default(),
                                                    Some(deadline),
                                                )
                                            });
                                        let execution =
                                            pse_backend_native::solve::Execution::within(
                                                scope.cancellation().clone(),
                                                &pse_backend_native::solve::Controls::default(),
                                                scope.clone(),
                                            )
                                            .map_err(crate::math::MathRuntimeError::from)?;
                                        let branch = target.solve.composition_request().branch;
                                        target.solve = target
                                            .solve
                                            .clone()
                                            .within_task(scope.clone())
                                            .map_err(crate::math::MathRuntimeError::from)?;
                                        let proposal = point
                                            .root_prediction(&target.solve, branch, &execution)
                                            .map(|(proposal, _)| proposal)
                                            .or_else(|error| {
                                                if !matches!(
                                                    error.boundary_diagnostic().class,
                                                    BoundaryClass::Unsupported
                                                        | BoundaryClass::Incompatible
                                                        | BoundaryClass::Numerical
                                                ) {
                                                    return Err(error);
                                                }
                                                let StartPolicy::Continuation(edge) =
                                                    &graph.points[positions[predecessor]].start
                                                else {
                                                    return Err(error);
                                                };
                                                let Some(RunReport::Modeling(older)) = results
                                                    [positions[&edge.predecessor]]
                                                    .as_deref()
                                                    .and_then(|result| result.report().ok())
                                                else {
                                                    return Err(error);
                                                };
                                                let [older] = older.as_slice() else {
                                                    return Err(error);
                                                };
                                                point.secant_prediction(
                                                    older, target, branch, &execution,
                                                )
                                            });
                                        match proposal {
                                            Ok(proposal) => {
                                                let screened = self
                                                    .runtime
                                                    .native()
                                                    .screen_start(
                                                        target.solve.clone(),
                                                        proposal,
                                                        branch,
                                                        scope,
                                                        cancel,
                                                    )
                                                    .await?;
                                                Some(
                                                    target
                                                        .solve
                                                        .clone()
                                                        .with_screened_start(&screened)
                                                        .map_err(
                                                            crate::math::MathRuntimeError::from,
                                                        )?,
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
                                } else {
                                    **target = target.as_ref().clone().with_start(previous)?;
                                }
                            }
                            let result = if let Some(result) = batched.remove(&index) {
                                result
                            } else {
                                execute(
                                    self,
                                    &mut staged,
                                    operation,
                                    facts[index].attempt_count - 1,
                                    cancel,
                                )
                                .await
                            };
                            let (scientific, diagnostic) = match &result {
                                Ok(result) => (
                                    super::study_operations::scientific_facts(result),
                                    result_diagnostic(result).map(|diagnostic| {
                                        diagnostic.with_revision(
                                            definition.points[index]
                                                .operation
                                                .source
                                                .revision
                                                .as_id(),
                                        )
                                    }),
                                ),
                                Err(error) => (
                                    ScientificFacts::default(),
                                    Some(error.boundary_diagnostic().with_revision(
                                        definition.points[index].operation.source.revision.as_id(),
                                    )),
                                ),
                            };
                            let cancelled = cancel.token().is_cancelled()
                                || diagnostic
                                    .as_ref()
                                    .is_some_and(|d| d.class == BoundaryClass::Cancelled);
                            let lifecycle = if cancelled {
                                StudyPointState::Cancelled
                            } else if result.as_ref().is_ok_and(|result| result.report().is_ok()) {
                                StudyPointState::Completed
                            } else {
                                StudyPointState::Failed
                            };
                            facts[index].lifecycle = lifecycle;
                            facts[index].scientific = scientific.clone();
                            // Mathematical attempts have no external publication effect; deterministic scientific failures do not retry automatically.
                            facts[index].retry_failure = Some(RetryFailure::Deterministic);
                            outcomes[index].lifecycle = lifecycle;
                            outcomes[index].scientific = scientific.clone();
                            outcomes[index].diagnostic = diagnostic.clone();
                            outcomes[index].start = Some(start.clone());
                            outcomes[index].attempts.push(PointAttemptOutcome {
                                attempt_id: None,
                                lifecycle: Some(if cancelled {
                                    pse_model::generated::enums::AttemptState::Cancelled
                                } else if result
                                    .as_ref()
                                    .is_ok_and(|result| result.report().is_ok())
                                {
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
                                start: Some(start.clone()),
                                effect: EffectState::Absent,
                            });
                            if let Ok(result) = result {
                                results[index] = Some(result);
                            }
                        }
                    }
                }
                if !changed {
                    break Ok::<_, WorkflowError>(decision);
                }
            }
        })
        .await;
        staged.close().await;
        Ok(StudyReport {
            run_id,
            definition: definition.clone(),
            outcomes,
            results,
            decision: decision?,
            preparations,
            runtime: self.runtime.clone(),
            _owner: owner,
        })
    }
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
fn primal_seed(result: &RunResult) -> Option<pse_backend_native::solve::WarmStart> {
    match result.report().ok()? {
        RunReport::Modeling(points) => match points.as_slice() {
            [point] => match &point.outcome {
                crate::math::solves::Outcome::Native(native) => native.warm_start.clone(),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}
fn memory_seed(target: &PreparedStudyOperation, previous: Option<&RunResult>) -> SeedAvailability {
    let Some(previous) = previous else {
        return SeedAvailability::Absent;
    };
    let Some(seed) = primal_seed(previous) else {
        return SeedAvailability::Absent;
    };
    let PreparedStudyOperation::DeclaredCase(target) = target else {
        return SeedAvailability::Incompatible;
    };
    let Some(compatibility) = target.solve.compatibility() else {
        return SeedAvailability::Absent;
    };
    if compatibility.layout != seed.compatibility.layout
        || compatibility.backend != seed.compatibility.backend
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
    use crate::workflow::tests::{compiler_profile, physical, runtime};
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
        let runtime = runtime();
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
        let package = runtime.modeling_package(rows, physical()).unwrap();
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
                ContentHash::from_bytes([0; 32]),
                vec![],
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
            report.results[0].as_ref().unwrap().run_id,
            report.results[1].as_ref().unwrap().run_id
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
        assert!((result(&report, 3).values.scalars[&variable] - 4.).abs() < 1e-8);
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
                report.preparations.shared + report.preparations.rebuilt >= report.outcomes.len(),
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
                .all(|point| point.scientific.usable)
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
        assert!(a.preparations.shared + a.preparations.rebuilt >= 4);
        assert!(b.preparations.shared + b.preparations.rebuilt >= 2);
    }
    #[cfg(feature = "solver-pounce")]
    #[tokio::test]
    async fn occurrence_adapter_reaches_existing_pounce_batch_owner() {
        use pse_backend_native::solve::Metric;
        let source = "package p { def Root { param t: Scalar = 1; param b: Scalar = 0.4; var x: Scalar; var y: Scalar; eq budget: x + y <= 1; let cost: Scalar = (x-t)*(x-t)+(y-b)*(y-b); annotation objective cost(minimize); annotation bounds x(0,5); annotation bounds y(0,5); annotation start x(0); annotation start y(0); } }";
        let (package, definition) =
            fixture_source(&[0.25, 0.5, 0.75, 1.], source, Backend::PounceConvex, 2).await;
        let report = package
            .study(&definition, 8, &crate::CancelSource::new())
            .await
            .unwrap();
        assert_eq!(report.outcomes.len(), 4);
        for (index, outcome) in report.outcomes.iter().enumerate() {
            assert!(outcome.scientific.usable, "{index}: {outcome:?}");
            let point = result(&report, index);
            let crate::math::solves::Outcome::Native(native) = &point.outcome else {
                panic!("{:?}", point.outcome)
            };
            assert_eq!(native.backend, Backend::PounceConvex);
            assert_eq!(native.metrics["batch"], Metric::Integer(4));
            let (a, b) = (
                definition.points[index].binding.values()[&member(point, "t")],
                0.4,
            );
            let excess = (a + b - 1.).max(0.) / 2.;
            assert!((point.values.scalars[&member(point, "x")] - (a - excess)).abs() < 1e-6);
            assert!((point.values.scalars[&member(point, "y")] - (b - excess)).abs() < 1e-6);
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
            assert!((point.values.scalars[&member(point, "x")] - expected).abs() < 1e-8);
        }
    }
}
