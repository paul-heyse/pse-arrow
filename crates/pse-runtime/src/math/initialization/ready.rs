// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent admitted block owners, with provisional dependency commits and original assessment.
use super::*;
use futures_util::StreamExt;

impl MathService {
    #[allow(
        clippy::too_many_arguments,
        reason = "ready initialization preserves original preparation, values, providers, profile, numerical policy and selected routes"
    )]
    pub(super) fn initialize_ready(
        self: &Arc<Self>,
        prepared: PreparedInitialization,
        original: CaseValues,
        providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        profile: InitializationProfile,
        strategies: Vec<native::routing::Route>,
        numerics: pse_math::SharedAllocation<pse_model::numerics::ResolvedNumericalPolicy>,
    ) -> Result<SolveHandle<InitializationReport>, MathRuntimeError> {
        if strategies
            .iter()
            .any(|route| matches!(route, native::routing::Route::Native(Backend::Pounce)))
        {
            return Err(native::ProblemError::Unsupported(
                "POUNCE block aggregate iteration coverage unavailable".into(),
            )
            .into());
        }
        let effective = prepared
            .blocks
            .iter()
            .zip(&strategies)
            .map(|(block, route)| {
                let native::routing::Route::Native(backend) = route else {
                    return Err(native::ProblemError::Contract(
                        "ready block requires native team".into(),
                    ));
                };
                let controls = profile.solver.block_profile(block.boundary.id)?.controls;
                BlockDemand::new(std::iter::once((*backend, controls.clone())), &self.policy)?
                    .admit(self.cores)
                    .map_err(MathRuntimeError::into_problem)?;
                Ok(controls)
            })
            .collect::<Result<Vec<_>, native::ProblemError>>()?;
        let deadline = std::time::Instant::now()
            .checked_add(profile.solver.controls.time_limit)
            .ok_or(MathRuntimeError::Limit(
                "initialization task deadline extent",
            ))?;
        let scope = pse_kernels::ExecutionScope::new(Cancellation::default(), Some(deadline));
        let progress = Arc::new(Progress::new(profile.solver.controls.history));
        let report_bytes = prepared.blocks.iter().try_fold(
            original.scalars.len().saturating_mul(64),
            |total, block| {
                block
                    .boundary
                    .members
                    .columns
                    .len()
                    .checked_add(block.boundary.members.rows.len())
                    .and_then(|n| n.checked_mul(512))
                    .and_then(|n| n.checked_add(profile.solver.controls.report_allowance().ok()?))
                    .and_then(|n| n.checked_mul(profile.stages.len()))
                    .and_then(|n| total.checked_add(n))
                    .ok_or(MathRuntimeError::Limit("initialization result extent"))
            },
        )?;
        let owner = self.reserve("math:ready-initialization-results", report_bytes)?;
        let service = self.clone();
        let events = progress.clone();
        let run_id = pse_operations::mint_id();
        Ok(SolveHandle::supervise(progress, move |cancel| async move {
            let prepared = Arc::new(prepared);
            let admission = profile.solver.composition.limits.map(|limits| {
                super::super::strategy::admission::TaskAdmission::new_for(
                    limits,
                    scope.clone(),
                    &service,
                    false,
                )
            });
            let solved: BTreeSet<_> = prepared
                .boundaries()
                .flat_map(|block| block.members.columns.iter().copied())
                .collect();
            let mut attempts: Vec<BlockAttempt> = Vec::new();
            let mut stages = Vec::new();
            let mut committed = CaseValues {
                scalars: BTreeMap::new(),
            };
            let mut completed_stages = 0;
            let mut iterations = Iterations::new(profile.solver.controls.iterations);
            let mut bindings =
                vec![None::<pse_math::SharedAllocation<CaseValues>>; prepared.blocks.len()];
            let previous_accepted = profile.solver.controls.start == StartPolicy::PreviousAccepted;
            for (stage, updates) in profile.stages.iter().enumerate() {
                let mut values = original.clone();
                if previous_accepted {
                    values
                        .scalars
                        .extend(committed.scalars.iter().map(|(id, value)| (*id, *value)));
                }
                values
                    .scalars
                    .extend(updates.iter().map(|(id, value)| (*id, *value)));
                let first = attempts.len();
                let mut completed = true;
                let mut readiness = prepared.dependencies.start();
                let mut running = futures_util::stream::FuturesUnordered::new();
                let mut active_threads = 0usize;
                let mut issued = (0..prepared.blocks.len()).map(|_| None).collect::<Vec<_>>();
                while readiness.remaining != 0 || !running.is_empty() {
                    if cancel.token().is_cancelled() || scope.check().is_err() {
                        completed = false;
                    }
                    let mut deferred = Vec::new();
                    while completed && running.len() < profile.solver.block_workers {
                        let Some(index) = readiness.take() else {
                            break;
                        };
                        if active_threads + effective[index].threads > service.cores {
                            deferred.push(index);
                            continue;
                        }
                        let allowance = match iterations.reserve(
                            index,
                            effective[index].iterations,
                            profile.solver.block_workers - running.len(),
                        ) {
                            Ok(allowance) => allowance,
                            Err(_) => {
                                readiness.defer(index);
                                break;
                            }
                        };
                        let dispatch = (|| {
                            let reusable = bindings[index]
                                .as_mut()
                                .map(|inputs| {
                                    prepared.dependencies.refresh_inputs(index, inputs, &values)
                                })
                                .transpose()?
                                .unwrap_or(false);
                            if !reusable {
                                let input_owner = service.reserve(
                                    "math:block-dispatch-inputs",
                                    prepared.dependencies.binding_bytes(index),
                                )?;
                                let inputs: pse_math::SharedAllocation<CaseValues> =
                                    Arc::new(prepared.dependencies.inputs(index, &values)?).into();
                                bindings[index] = Some(inputs.with_owner(input_owner));
                            }
                            let inputs = bindings[index]
                                .as_ref()
                                .ok_or_else(|| {
                                    native::ProblemError::Internal(
                                        "conditional binding base absent".into(),
                                    )
                                })?
                                .clone();
                            let admission = admission
                                .as_ref()
                                .map(|parent| parent.child())
                                .transpose()?;
                            Ok::<_, MathRuntimeError>((inputs, admission))
                        })();
                        let (inputs, child_admission) = match dispatch {
                            Ok(dispatch) => dispatch,
                            Err(error) => {
                                let _ = iterations.complete(index, Some(0));
                                issued[index] = Some(BlockAttempt {
                                    stage,
                                    strategy: strategies[index],
                                    boundary: prepared.blocks[index].boundary.clone(),
                                    result: Err(Arc::new(error)),
                                    trace: None,
                                    committed: false,
                                });
                                completed = false;
                                break;
                            }
                        };
                        let block = &prepared.blocks[index];
                        let previous = (previous_accepted && stage > 0)
                            .then(|| {
                                attempts.iter().rposition(|attempt| {
                                    attempt.boundary.id == block.boundary.id && attempt.committed
                                })
                            })
                            .flatten();
                        active_threads += effective[index].threads;
                        running.push(service.clone().run_ready_block(
                            prepared.clone(),
                            index,
                            inputs,
                            providers.clone(),
                            profile.solver.clone(),
                            numerics.clone(),
                            strategies[index],
                            scope.clone(),
                            child_admission,
                            cancel.clone(),
                            allowance,
                            previous,
                            first + index,
                            events.clone(),
                            owner.clone(),
                        ));
                    }
                    for index in deferred {
                        readiness.defer(index);
                    }
                    let Some((index, attempted)) = running.next().await else {
                        if readiness.remaining != 0 {
                            completed = false;
                        }
                        break;
                    };
                    active_threads -= effective[index].threads;
                    let trace = match &attempted {
                        Ok((_, trace)) => Some(trace.clone()),
                        Err(MathRuntimeError::Strategy { trace, .. }) => Some(trace.clone()),
                        _ => None,
                    };
                    let result = attempted.map(|(report, _)| report).map_err(Arc::new);
                    let accounted = iterations
                        .complete(
                            index,
                            result
                                .as_ref()
                                .ok()
                                .and_then(|report| report.evidence.work.iterations),
                        )
                        .is_ok();
                    let mut committed_block = accounted
                        && commit_block(
                            &mut values,
                            &prepared.blocks[index].boundary,
                            result.as_deref().ok(),
                            &numerics.policy,
                        )
                        .is_ok();
                    if committed_block && let Err(error) = readiness.commit(index) {
                        committed_block = false;
                        if let Some(context) = pse_diagnostics::observation::current() {
                            context.unavailable("math.block_dependency_commit", &error.to_string());
                        }
                    }
                    completed &= committed_block;
                    issued[index] = Some(BlockAttempt {
                        stage,
                        strategy: strategies[index],
                        boundary: prepared.blocks[index].boundary.clone(),
                        result,
                        trace,
                        committed: committed_block,
                    });
                    // Known failure stops dispatch, while every already-issued future is
                    // polled to completion. Dependency values remain provisional here.
                }
                attempts.extend(issued.into_iter().flatten());
                if completed && let Some(original_case) = &prepared.original {
                    completed = service
                        .assess_ready_original(
                            original_case.clone(),
                            values.clone(),
                            providers.clone(),
                            numerics.clone(),
                            &profile.solver.controls,
                            &scope,
                            admission.clone(),
                            &cancel,
                        )
                        .await?;
                }
                if completed {
                    committed.scalars = values
                        .scalars
                        .iter()
                        .filter(|(id, _)| solved.contains(id))
                        .map(|(id, value)| (*id, *value))
                        .collect();
                    completed_stages += 1;
                } else {
                    for attempt in &mut attempts[first..] {
                        attempt.committed = false;
                    }
                }
                stages.push(StageAttempt {
                    stage,
                    overlay: updates.clone(),
                    candidate: values,
                    completed,
                });
                if !completed {
                    break;
                }
            }
            let original_bindings_restored = stages.last().is_some_and(|stage| {
                stage.completed
                    && stage
                        .overlay
                        .iter()
                        .all(|(id, value)| original.scalars.get(id) == Some(value))
            });
            Ok(InitializationReport {
                run_id,
                original,
                values: committed,
                stages,
                original_bindings_restored,
                attempts,
                completed_stages,
                cancelled: cancel.token().is_cancelled(),
                _owner: owner,
            })
        }))
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "issued block retains its exact task and admission owners through completion and neutral reset"
    )]
    async fn run_ready_block(
        self: Arc<Self>,
        prepared: Arc<PreparedInitialization>,
        index: usize,
        values: pse_math::SharedAllocation<CaseValues>,
        providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        profile: SolverProfile,
        numerics: pse_math::SharedAllocation<pse_model::numerics::ResolvedNumericalPolicy>,
        route: native::routing::Route,
        scope: pse_kernels::ExecutionScope,
        admission: Option<Arc<super::super::strategy::admission::TaskAdmission>>,
        cancel: crate::CancelSource,
        iterations: u32,
        previous: Option<usize>,
        attempt: usize,
        events: Arc<Progress>,
        owner: Arc<pse_columnar::AllocationLease>,
    ) -> (
        usize,
        Result<(Box<SolveReport>, super::super::solves::StrategyTrace), MathRuntimeError>,
    ) {
        let child_admission = admission.clone();
        let mut result = async {
            let (step, assessor, previous) = self
                .prepare_ready_block(
                    &prepared, index, values, &providers, &profile, &numerics, route, &scope,
                    admission, &cancel, iterations, previous,
                )
                .await?;
            let affinity = step.preparation_identity()?;
            let session = self.independent_session(affinity)?;
            let result = session
                .step(
                    step,
                    previous,
                    attempt,
                    events,
                    owner,
                    &cancel,
                    move |outcome, scope, budget| assessor.assess(outcome, scope, budget),
                )
                .await;
            let _ = session
                .park_independent(affinity, Some(scope.clone()), &cancel)
                .await;
            let (outcome, independent, trace) = result?;
            match outcome {
                super::super::solves::Outcome::Native(mut report) => {
                    independent.apply(&mut report);
                    Ok((report, trace))
                }
                super::super::solves::Outcome::Rejected(cause) => {
                    Err(MathRuntimeError::Strategy { cause, trace })
                }
                _ => Err(native::ProblemError::Internal(
                    "conditional native report unavailable".into(),
                )
                .into()),
            }
        }
        .await;
        if let Some(admission) = child_admission {
            let drained = admission.finish_child().map_err(MathRuntimeError::from);
            if result.is_ok() {
                result = drained.and(result);
            } else if let Err(error) = drained
                && let Some(context) = pse_diagnostics::observation::current()
            {
                context.unavailable("math.block_child_drain", &error.to_string());
            }
        }
        #[cfg(test)]
        prepared
            .dependencies
            .delivered(index, &scope, &cancel)
            .await;
        (index, result)
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "one ready block preserves exact original task, effective profile, numerical policy and predecessor seed authorities"
    )]
    async fn prepare_ready_block(
        self: &Arc<Self>,
        prepared: &PreparedInitialization,
        index: usize,
        values: pse_math::SharedAllocation<CaseValues>,
        providers: &BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        original_profile: &SolverProfile,
        numerics: &pse_math::SharedAllocation<pse_model::numerics::ResolvedNumericalPolicy>,
        route: native::routing::Route,
        scope: &pse_kernels::ExecutionScope,
        admission: Option<Arc<super::super::strategy::admission::TaskAdmission>>,
        cancel: &crate::CancelSource,
        iterations: u32,
        previous: Option<usize>,
    ) -> Result<
        (
            super::super::solves::PreparedSolve,
            BlockAssessor,
            Option<Predecessor>,
        ),
        MathRuntimeError,
    > {
        scope.check().map_err(native::ProblemError::Provider)?;
        let block = prepared.blocks[index].clone();
        let view = block.view.clone();
        let quantities = prepared.quantities.clone();
        let preconditions = prepared.preconditions.clone();
        let inputs = values.clone();
        let control = Cancellation::default();
        let construction = super::super::jobs::ConstructionEstimate::from_bound(
            view.binding_allocation_bound(&inputs)?,
            self.policy.worker_bytes,
        );
        let operation = self.job_retained_construction(
            1,
            construction,
            control.clone(),
            scope.deadline(),
            move |flag| {
                let bound = view.bind(quantities, preconditions, &inputs, &flag)?;
                let bytes = block_binding_bytes(&bound);
                Ok((bound, bytes))
            },
        );
        tokio::pin!(operation);
        let (bound, lease) = tokio::select! { result = &mut operation => result?, () = cancel.cancelled() => { control.cancel(); let _ = operation.await; return Err(MathRuntimeError::Cancelled); } };
        let preparation = self.own_binding(
            block.owner.clone(),
            bound,
            lease,
            Arc::new(std::sync::OnceLock::from(block.executable.clone())),
        )?;
        let mut profile = original_profile.block_profile(block.boundary.id)?;
        profile.controls.iterations = profile.controls.iterations.min(iterations);
        profile.presolve = native::presolve::Policy::Off;
        profile.intent = SolveIntent::Initialize;
        let native::routing::Route::Native(backend) = route else {
            return Err(native::ProblemError::Unsupported(
                "conditional native route required".into(),
            )
            .into());
        };
        let (preparation, executable) = self
            .conditional_program(
                preparation,
                block.executable,
                &profile,
                backend,
                scope,
                cancel,
            )
            .await?;
        let source_terms = executable
            .assembly
            .structure()
            .instances()
            .iter()
            .try_fold(0usize, |total, instance| {
                total.checked_add(instance.contributions.len())
            })
            .ok_or(MathRuntimeError::Limit("block assessment source terms"))?;
        let assessment_bytes = executable
            .assembly
            .numeric_worker_bytes()
            .checked_add(
                values
                    .scalars
                    .len()
                    .checked_mul(64)
                    .ok_or(MathRuntimeError::Limit("block assessment extent"))?,
            )
            .and_then(|bytes| {
                bytes.checked_add(executable.assembly.input_ids().len().checked_mul(8)?)
            })
            .and_then(|bytes| {
                bytes.checked_add(block.boundary.members.rows.len().checked_mul(512)?)
            })
            .and_then(|bytes| {
                bytes.checked_add(block.boundary.members.columns.len().checked_mul(256)?)
            })
            .and_then(|bytes| bytes.checked_add(source_terms.checked_mul(128)?))
            .and_then(|bytes| bytes.checked_add(4096))
            .ok_or(MathRuntimeError::Limit("block assessment extent"))?;
        let mut original_execution = Execution::within(
            scope.cancellation_control().clone(),
            &original_profile.controls,
            scope.clone(),
        )?;
        original_execution.work_admission = admission
            .clone()
            .map(|owner| -> Arc<dyn WorkAdmission> { owner });
        let assessor = BlockAssessor {
            service: self.clone(),
            case: executable.clone(),
            values: values.clone(),
            providers: providers.clone(),
            numerics: numerics.clone(),
            original: original_execution,
            storage: self.reserve("math:ready-block-assessment", assessment_bytes)?,
        };
        let step = self.prepare_conditional_bound(
            preparation,
            executable,
            values.clone(),
            providers.clone(),
            profile,
            numerics.clone(),
            route,
            prepared.snapshot.clone(),
        )?;
        let step = match admission {
            Some(admission) => step.within_admitted_task(scope.clone(), admission)?,
            None => step.within_task(scope.clone())?,
        };
        let previous = previous
            .map(|attempt| step.primal_seed().map(|seed| Predecessor { attempt, seed }))
            .transpose()?;
        Ok((step, assessor, previous))
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "original assessment retains physical source, exact bindings, policy and inherited aggregate task"
    )]
    async fn assess_ready_original(
        self: &Arc<Self>,
        case: Arc<ExecutableCase>,
        values: CaseValues,
        providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        numerics: pse_math::SharedAllocation<pse_model::numerics::ResolvedNumericalPolicy>,
        controls: &Controls,
        scope: &pse_kernels::ExecutionScope,
        admission: Option<Arc<super::super::strategy::admission::TaskAdmission>>,
        cancel: &crate::CancelSource,
    ) -> Result<bool, MathRuntimeError> {
        let structure = case.assembly.structure().clone();
        let mut execution = Execution::within(
            scope.cancellation_control().clone(),
            controls,
            scope.clone(),
        )?;
        execution.work_admission = admission.map(|owner| -> Arc<dyn WorkAdmission> { owner });
        let control = Cancellation::default();
        let task = pse_kernels::ExecutionScope::new(control.clone(), scope.deadline());
        self.with_owned_worker(
            case,
            providers,
            cancel,
            Some((task, control)),
            move |mut worker| {
                let rows = execution.counted(
                    WorkEvidence {
                        evaluations: Some(1),
                        iterations: Some(0),
                        factorizations: Some(0),
                        proof_steps: Some(0),
                    },
                    || worker.constraints(&values).map_err(Into::into),
                )?;
                let row_ids: Vec<_> = structure.rows().iter().map(|row| row.id).collect();
                let columns: Vec<_> = structure
                    .variables()
                    .iter()
                    .filter(|variable| !variable.fixed)
                    .map(|variable| variable.port.id)
                    .collect();
                let tolerances = Tolerances::from_policy(&numerics, &columns, &row_ids)?;
                let rows_ok = rows.iter().zip(structure.rows()).zip(&tolerances.rows).all(
                    |((value, row), tolerance)| {
                        value.is_finite()
                            && *value >= row.lower - tolerance
                            && *value <= row.upper + tolerance
                    },
                );
                let bounds_ok = structure.variables().iter().all(|variable| {
                    values.scalars.get(&variable.port.id).is_some_and(|value| {
                        value.is_finite()
                            && variable.lower.is_none_or(|lower| *value >= lower)
                            && variable.upper.is_none_or(|upper| *value <= upper)
                    })
                });
                execution.check()?;
                Ok(rows_ok && bounds_ok)
            },
        )
        .await
    }
}
