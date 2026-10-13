// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Compiler-issued conditional equations executed in one original task.
use super::*;
use std::collections::BTreeSet;
use pse_kernels::ExecutionScope;
use pse_model::strategy::{MechanismKind, WorkObservation};

/// A complete structural schedule with contextually admitted per-block native profiles.
#[derive(Clone, Debug)]
pub struct PreparedBlocks {
    original: Box<PreparedSolve>,
    blocks: Vec<PreparedSolve>,
    boundaries: Arc<Vec<pse_compiler::workspace::PreparedBlock>>,
    key: pse_ids::ContentHash,
    scope: ExecutionScope,
    backend: Backend,
    workers: usize,
    threads: usize,
    backends: Vec<Backend>,
    foreign_bytes: usize,
    _owner: Arc<super::super::products::ProductOwner>,
}
impl PreparedBlocks {
    /// Frozen complete original scientific source.
    pub fn original(&self) -> &PreparedSolve {
        &self.original
    }
    /// Identity of the actual original request and compiler schedule.
    pub fn key(&self) -> pse_ids::ContentHash {
        self.key
    }
    /// Original caller cancellation and absolute deadline.
    pub fn scope(&self) -> &ExecutionScope {
        &self.scope
    }
    /// Original solve adapter retained as the composition representative.
    pub fn backend(&self) -> Backend {
        self.backend
    }
    /// Maximum admitted independent ready-block width.
    pub fn workers(&self) -> usize { self.workers }
    pub(crate) fn threads(&self) -> usize { self.threads }
    /// Complete actual adapter inventory for the contextually resolved block profiles.
    pub fn backends(&self) -> &[Backend] { &self.backends }
    /// Effective profiles keyed by compiler identity, inspectable before execution.
    pub fn profiles(&self) -> impl Iterator<Item = (pse_structural::incidence::BlockId, &SolverProfile)> {
        self.boundaries.iter().zip(&self.blocks).map(|(block, step)| (block.boundary.id, &step.profile))
    }
    pub(crate) fn foreign_bytes(&self) -> usize { self.foreign_bytes }
    /// Number of actual conditional native operations in the finite schedule.
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }
    pub(crate) fn result_bytes(&self) -> Result<usize, MathRuntimeError> {
        self.blocks
            .iter()
            .try_fold(self.original.result_bytes()?, |bytes, block| {
                bytes
                    .checked_add(block.result_bytes()?)
                    .ok_or(MathRuntimeError::Limit("block result extent"))
            })
    }
    pub(super) fn completed_result_bytes(
        &self,
        report: &ConstantReport,
    ) -> Result<usize, MathRuntimeError> {
        let ConstantReport {
            components: _,
            owner: _,
            objective: _,
            observation: _,
            quality: _,
            coordinates: _,
            certified_reconstruction: _,
            work: _,
        } = report;
        if self.blocks.len() != report.components.len() {
            return Err(ProblemError::Internal(
                "completed block report count differs from its schedule".into(),
            )
            .into());
        }
        let limit = || ProblemError::memory("completed block reporting capacity");
        let original = self.original.result_bytes()?;
        let allowance = self.original.controls().report_allowance()?;
        let original_payload = size_of::<ConstantReport>()
            .checked_add(
                report
                    .observation
                    .completed_report_allowance()
                    .ok_or_else(limit)?,
            )
            .and_then(|n| n.checked_add(report.quality.completed_report_allowance()?))
            .and_then(|n| {
                n.checked_add(
                    report
                        .coordinates
                        .capacity()
                        .checked_mul(size_of::<(pse_ids::SemanticId, f64)>())?,
                )
            })
            .and_then(|n| {
                n.checked_add(
                    report
                        .components
                        .capacity()
                        .checked_mul(size_of::<Box<SolveReport>>())?,
                )
            })
            .ok_or_else(limit)?;
        if original_payload > original {
            return Err(limit().into());
        }
        // The completed bound includes visible numeric capacities. Keep source-derived
        // dimension headroom as well, capped only at an admission already proved to
        // cover that complete visible envelope; never cap an oversized envelope.
        let original_retained = original
            .checked_sub(allowance)
            .and_then(|n| n.checked_add(original_payload))
            .ok_or_else(limit)?
            .min(original);
        self.blocks.iter().zip(&report.components).try_fold(
            original_retained,
            |bytes, (prepared, component)| {
                let admitted = prepared.result_bytes()?;
                let retained = match component.completed_report_allowance()? {
                    Some(completed) => {
                        if completed > admitted {
                            return Err(limit().into());
                        }
                        admitted
                            .checked_sub(prepared.controls().report_allowance()?)
                            .and_then(|n| n.checked_add(completed))
                            .ok_or_else(limit)?
                            .min(admitted)
                    }
                    None => admitted,
                };
                bytes.checked_add(retained).ok_or_else(|| limit().into())
            },
        )
    }
}
impl PreparedSolve {
    /// Cheap execution-dependency schedule eligibility, without optional evaluator artifacts.
    pub(crate) fn automatic_block_count(
        &self,
        cancel: &Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<usize, MathRuntimeError> {
        if !pse_model::strategy::permits_automatic_blocks(self.automatic_operation_facts()) {
            return Ok(0);
        }
        let Representation::Algebraic(source) = &self.representation else {
            return Ok(0);
        };
        Ok(
            match source
                .prepared
                .compiled()
                .automatic_alternatives(cancel)?
                .initialization
            {
                pse_compiler::workspace::Alternative::Available(schedule)
                    if schedule.blocks.len() > 1 =>
                {
                    schedule.blocks.len()
                }
                _ => 0,
            },
        )
    }
}
impl MathService {
    pub(super) async fn validate_prepared_block_overrides(
        self: &Arc<Self>, prepared: &Preparation, profile: &SolverProfile,
        numerics: &ResolvedNumericalPolicy,
    ) -> Result<(), MathRuntimeError> {
        let source = prepared.prepared.clone();
        let construction = super::super::jobs::ConstructionEstimate::from_bound(
            source.initialization_allocation_bound()?, self.policy.workspace_bytes,
        );
        let requested = profile.clone();
        let (admissions, _lease) = self.job_retained_construction(1, construction, FlightCancellation::default(), None, move |flag| {
            let pse_compiler::workspace::Alternative::Available(schedule) = source.automatic_alternatives(&flag)?.initialization else {
                return Err(ProblemError::Unsupported("explicit block overrides require complete compiler block inventory".into()).into());
            };
            requested.validate_block_ids(schedule.blocks.iter().map(|block| block.id))?;
            let original = native::assembled::contract(&source.plan);
            let mut admissions = Vec::with_capacity(requested.block_overrides.len());
            let mut bytes = size_of::<Vec<(pse_structural::incidence::BlockId, native::OracleContract, pse_kernels::DerivativeOrder, native::routing::Structure)>>();
            for boundary in schedule.blocks.iter().filter(|block| requested.block_overrides.contains_key(&block.id)) {
                let rows = boundary.members.rows.iter().copied().collect::<BTreeSet<_>>();
                let columns = boundary.members.columns.iter().copied().collect::<BTreeSet<_>>();
                let available = source.plan.conditional_available_order(&rows, &columns, &flag)?;
                let contract = native::OracleContract {
                    identity: boundary.id.0,
                    variables: original.variables.iter().filter(|variable| columns.contains(&variable.id)).cloned().collect(),
                    rows: source.plan.structure().rows().iter().filter(|row| rows.contains(&row.id)).map(|row| row.id).collect(),
                    derivatives: source.plan.order(), smoothness: available,
                };
                let witness = source.structure.conditional_witness(boundary)
                    .map_err(|error| ProblemError::Contract(error.to_string()))?;
                let structure = native::routing::Structure {
                    variables: boundary.members.columns.clone(),
                    equations: source.plan.structure().rows().iter().filter(|row| rows.contains(&row.id)).map(|row| pse_structural::incidence::Constraint {
                        id: row.id, lower: row.lower.is_finite().then_some(row.lower), upper: row.upper.is_finite().then_some(row.upper),
                    }).collect(),
                    witness: Arc::new(witness).into(),
                };
                let scope_members = rows.len().checked_add(columns.len()).and_then(|n| n.checked_add(boundary.inputs.len()))
                    .ok_or(MathRuntimeError::Limit("block scope fact extent"))?;
                let scope_bytes = scope_members.checked_mul(11 * size_of::<pse_ids::SemanticId>() + 16 * size_of::<usize>())
                    .ok_or(MathRuntimeError::Limit("block scope fact extent"))?;
                let owned = size_of_val(&contract) + size_of_val(&structure) + size_of::<pse_structural::incidence::BlockId>()
                    + contract.variables.capacity() * size_of::<native::Variable>() + contract.rows.capacity() * size_of::<pse_ids::SemanticId>()
                    + structure.variables.capacity() * size_of::<pse_ids::SemanticId>() + structure.equations.capacity() * size_of::<pse_structural::incidence::Constraint>()
                    + structure.witness.retained_bytes() + scope_bytes + 64;
                bytes = bytes.checked_add(owned).ok_or(MathRuntimeError::Limit("block admission fact extent"))?;
                admissions.push((boundary.id, contract, available, structure));
            }
            Ok((admissions, bytes))
        }).await?;
        let snapshot = execution::Snapshot::observe(&execution::LINKED);
        for (id, contract, available, structure) in admissions.iter() {
            let mut effective = profile.block_profile(*id)?;
            effective.intent = SolveIntent::Initialize;
            effective.presolve = native::presolve::Policy::Off;
            super::super::initialization::conditional_contract_route(&snapshot, contract, *available, contract.derivatives, structure.clone(), &effective, numerics)?;
        }
        Ok(())
    }
    pub(crate) async fn prepare_blocks(
        self: &Arc<Self>,
        mut original: PreparedSolve,
        scope: ExecutionScope,
        driver: &crate::CancelSource,
    ) -> Result<PreparedBlocks, MathRuntimeError> {
        scope.check().map_err(ProblemError::Provider)?;
        if original.automatic_block_count(scope.cancellation())? < 2 {
            return Err(ProblemError::Unsupported(
                "complete original block schedule is unavailable".into(),
            )
            .into());
        }
        original.composition = None;
        original = original.within_task(scope.clone())?;
        let Representation::Algebraic(source) = &original.representation else {
            return Err(
                ProblemError::Unsupported("blocks require an algebraic original".into()).into(),
            );
        };
        let compiled = source.prepared.prepared.clone();
        let control = FlightCancellation::default();
        let construction = super::super::jobs::ConstructionEstimate::from_bound(
            compiled.initialization_allocation_bound()?,
            self.policy.workspace_bytes,
        );
        let job = self.job_retained_construction(
            1,
            construction,
            control.clone(),
            scope.deadline(),
            move |flag| {
                let pse_compiler::workspace::Alternative::Available(blocks) =
                    compiled.automatic_blocks(&flag)?
                else {
                    return Err(ProblemError::Unsupported(
                        "compiler block schedule unavailable".into(),
                    )
                    .into());
                };
                let bytes = pse_compiler::workspace::PreparedBlock::retained_group_bytes(&blocks)
                    .and_then(|bytes| bytes.checked_add(size_of::<PreparedBlocks>()))
                    .ok_or(MathRuntimeError::Limit("compiler block schedule extent"))?;
                Ok((blocks, bytes))
            },
        );
        tokio::pin!(job);
        let (products, lease) = tokio::select! { result=&mut job=>result?, ()=driver.cancelled()=>{ control.cancel(); let _=job.await; return Err(MathRuntimeError::Cancelled); } };
        let owner = self.shared_product(
            vec![41, Arc::as_ptr(&products) as usize],
            products.clone(),
            lease,
            vec![source.prepared._owner.clone()],
        )?;
        let backend = original
            .backend()
            .ok_or_else(|| ProblemError::Unsupported("blocks require native equations".into()))?;
        let mut requested_profile = original.profile.clone();
        requested_profile.controls.hessian = original.requested_hessian;
        requested_profile.validate_block_ids(products.iter().map(|block| block.boundary.id))?;
        let workers = original.profile.block_workers.min(products.len());
        let mut backends = Vec::new();
        let mut steps = Vec::with_capacity(products.len());
        let mut key = FramedHasher::new(pse_ids::Frame::DerivedBindingV2);
        key.hash(&original.request_identity()?.as_id())
            .str(MechanismKind::Block.as_str());
        for block in products.iter() {
            scope.check().map_err(ProblemError::Provider)?;
            let executable = Self::within_task(
                &scope,
                driver,
                self.assemble_functions(
                    pse_compiler::workspace::PreparedFunctions {
                        plan: block.plan.clone(),
                        artifacts: block.artifacts.clone(),
                    },
                    self.reserve(
                        "math:block-functions",
                        block.plan.owner_wrapper_bytes() + 1024,
                    )?,
                    driver,
                ),
            )
            .await?;
            let view = block.clone();
            let quantities = source.prepared.prepared.quantities.clone();
            let preconditions = source.prepared.prepared.preconditions.clone();
            let values = source.values.clone();
            let control = FlightCancellation::default();
            let construction = super::super::jobs::ConstructionEstimate::from_bound(
                view.binding_allocation_bound(&values)?,
                self.policy.worker_bytes,
            );
            if construction.bytes() > self.policy.worker_bytes {
                return Err(MathRuntimeError::Limit(
                    "block binding construction capacity",
                ));
            }
            let binding = self.job_retained_construction(
                1,
                construction,
                control.clone(),
                scope.deadline(),
                move |flag| {
                    let bound = view.bind(quantities, preconditions, &values, &flag)?;
                    let bytes = binding_bytes(&bound);
                    Ok((bound, bytes))
                },
            );
            tokio::pin!(binding);
            let (bound, lease) = tokio::select! { result=&mut binding=>result?, ()=driver.cancelled()=>{ control.cancel(); let _=binding.await; return Err(MathRuntimeError::Cancelled); } };
            scope.check().map_err(ProblemError::Provider)?;
            let preparation = self.own_binding(
                owner.clone(),
                bound,
                lease,
                Arc::new(std::sync::OnceLock::from(executable.clone())),
            )?;
            let mut profile = requested_profile.block_profile(block.boundary.id)?;
            profile.presolve = native::presolve::Policy::Off;
            profile.intent = SolveIntent::Initialize;
            profile.controls.reuse = ReusePolicy::Fresh;
            let route = super::super::initialization::conditional_route(
                &original.snapshot, block, &executable, &profile, &original.numerics,
            )?;
            let Route::Native(actual_backend) = route else {
                return Err(ProblemError::Unsupported("block requires admitted native equations".into()).into());
            };
            if !backends.contains(&actual_backend) { backends.push(actual_backend); }
            let step = self
                .prepare_conditional(
                    preparation,
                    executable,
                    source.values.clone(),
                    source.providers.clone(),
                    profile,
                    original.numerics.clone(),
                    route,
                    original.snapshot.clone(),
                    &scope,
                    driver,
                )
                .await?
                .within_task(scope.clone())?;
            key.hash(&step.preparation_identity()?);
            steps.push(step);
        }
        let mut done = vec![false; products.len()];
        let mut threads = 0;
        let mut foreign_bytes = 0;
        while done.iter().any(|done| !done) {
            let wave = super::super::initialization::ready_wave(products.iter().map(|block| &block.boundary), &done, workers)?;
            let profiles = wave.iter().map(|&index| {
                steps[index].backend().map(|backend| (backend, steps[index].profile.controls.clone()))
                    .ok_or_else(|| ProblemError::Internal("missing prepared block team".into()))
            }).collect::<Result<Vec<_>, _>>()?;
            let demand = super::super::initialization::BlockWaveDemand::new(profiles.into_iter(), &self.policy)?;
            demand.admit(self.cores)?;
            threads = threads.max(demand.threads);
            foreign_bytes = foreign_bytes.max(demand.foreign.checked_add(demand.stacks).ok_or(MathRuntimeError::Limit("ready native resident extent"))?);
            for index in wave { done[index] = true; }
        }
        Ok(PreparedBlocks {
            original: Box::new(original),
            blocks: steps,
            boundaries: products,
            key: key.finish_hash(),
            scope,
            backend,
            workers,
            threads,
            backends,
            foreign_bytes,
            _owner: owner,
        })
    }
    /// One finite schedule on the current worker, followed by fresh full-original evaluation.
    pub(crate) fn blocks_step(
        &self,
        prepared: &PreparedBlocks,
        mut execution: Execution,
        retained: &mut Retained,
        budget: &Arc<WorkerBudget>,
        original_start: &[f64],
        mut capture_component: impl FnMut(&SolveReport) -> Result<(), MathRuntimeError>,
    ) -> Result<Outcome, super::super::strategy::EffectFailure> {
        let mut work = match &prepared.original.representation {
            Representation::Algebraic(source) if !source.providers.is_empty() => {
                WorkEvidence::default()
            }
            _ => known_zero(),
        };
        let mut failure_cause = None;
        let mut component_observation = None;
        let result = (|| -> Result<Outcome, MathRuntimeError> {
            execution.check()?;
            let scope = execution.scope()?;
            if !Arc::ptr_eq(scope.cancellation(), prepared.scope.cancellation())
                || scope.deadline().is_some_and(|deadline| {
                    prepared
                        .scope
                        .deadline()
                        .is_none_or(|outer| deadline > outer)
                })
            {
                return Err(ProblemError::Contract(
                    "block task differs from prepared original scope".into(),
                )
                .into());
            }
            let Representation::Algebraic(source) = &prepared.original.representation else {
                return Err(
                    ProblemError::Internal("block original representation changed".into()).into(),
                );
            };
            let columns = source.prepared.prepared.plan.columns();
            if original_start.len() != columns.len()
                || original_start.iter().any(|value| !value.is_finite())
            {
                return Err(ProblemError::Contract(
                    "complete original block start required".into(),
                )
                .into());
            }
            let _charge = budget.charge(source.values.scalars.len().saturating_mul(96))?;
            let result_owner = self.reserve("math:block-results", prepared.result_bytes()?)?;
            let mut values = source.values.clone();
            values
                .scalars
                .extend(columns.iter().copied().zip(original_start.iter().copied()));
            execution.work_admission = budget
                .admission()
                .map(|owner| -> Arc<dyn WorkAdmission> { owner });
            let mut components = Vec::with_capacity(prepared.blocks.len());
            // Conditional data has a different compatibility stamp; the optional schedule
            // cannot reuse a state whose source/seed belongs to another block.
            retained.clear();
            let mut completed = vec![false; prepared.blocks.len()];
            let mut remaining_iterations = prepared.original.profile.controls.iterations;
            while completed.iter().any(|done| !done) {
                execution.check()?;
                let wave = super::super::initialization::ready_wave(
                    prepared.boundaries.iter().map(|block| &block.boundary),
                    &completed, prepared.workers,
                )?;
                let per_block_iterations = remaining_iterations / u32::try_from(wave.len()).map_err(|_| MathRuntimeError::Limit("block wave extent"))?;
                if per_block_iterations == 0 { return Err(ProblemError::Limit { kind: native::LimitKind::Work, detail: "aggregate block iteration allowance exhausted".into() }.into()); }
                let mut ready = Vec::with_capacity(wave.len());
                for &index in &wave {
                    let template = &prepared.blocks[index];
                execution.check()?;
                let mut step = template.clone();
                step.profile.controls.iterations = step.profile.controls.iterations.min(per_block_iterations);
                let Representation::Algebraic(local) = &mut step.representation else {
                    return Err(ProblemError::Internal(
                        "conditional representation changed".into(),
                    )
                    .into());
                };
                let bound = local.prepared.prepared.rebind(&values, &execution.cancel)?;
                let bytes = bound.rebind_allocation_bytes(&local.prepared.prepared);
                local.prepared = self.own_rebind(
                    &local.prepared,
                    bound,
                    self.reserve("math:block-rebind", bytes)?,
                )?;
                local.values = values.clone();
                step.compatibility = Some(compatibility(
                    &local.prepared.prepared.plan,
                    &values,
                    &step.profile,
                    &step.numerics,
                    template.backend().ok_or_else(|| ProblemError::Internal("missing block backend".into()))?,
                    &local.providers,
                    &step.snapshot,
                )?);
                    ready.push((index, step));
                }
                // The enclosing operation owns the admitted CPU width. Each child owns
                // its native state and destroys it before join; stacks remain charged.
                let profiles = ready.iter().map(|(_, step)| {
                    step.backend().map(|backend| (backend, step.profile.controls.clone()))
                        .ok_or_else(|| ProblemError::Internal("missing ready block team".into()))
                }).collect::<Result<Vec<_>, _>>()?;
                let demand = super::super::initialization::BlockWaveDemand::new(profiles.into_iter(), &self.policy)?;
                demand.admit(self.cores)?;
                if demand.threads > prepared.threads { return Err(ProblemError::Contract("ready wave exceeds prepared native team admission".into()).into()); }
                let _stacks = self.reserve("math:block-wave-stacks", demand.stacks)?;
                let (outcomes, spawn_failure) = if ready.len() == 1 {
                    let (index, step) = ready.remove(0);
                    let mut local_execution = execution.clone();
                    local_execution.memory = Some(self.policy.foreign_allowance(&step.profile.controls));
                    (vec![(index, self.run_step(step, local_execution, None, retained, budget))], None)
                } else { std::thread::scope(|scope| {
                    let mut children = Vec::with_capacity(ready.len());
                    let mut spawn_error = None;
                    for (index, step) in ready {
                        let mut child_execution = execution.clone();
                        child_execution.memory = Some(self.policy.foreign_allowance(&step.profile.controls));
                        let backend = step.backend().ok_or_else(|| ProblemError::Internal("missing block backend".into()))?;
                        let threads = step.profile.controls.threads;
                        match std::thread::Builder::new().name(format!("pse-block-{index}"))
                            .stack_size(self.policy.stack_bytes)
                            .spawn_scoped(scope, move || {
                                execution::scoped(&[execution::adapter(backend)], threads, self.policy.stack_bytes, || {
                                    let mut local_retained = Retained::default();
                                    let result = self.run_step(step, child_execution, None, &mut local_retained, budget);
                                    local_retained.clear();
                                    local_retained.release();
                                    result
                                })
                            }) {
                            Ok(child) => children.push((index, child)),
                            Err(error) => { spawn_error = Some(MathRuntimeError::Infrastructure(error.to_string())); break; }
                        }
                    }
                    let mut outcomes = Vec::with_capacity(children.len());
                    for (index, child) in children {
                        let result = child.join().unwrap_or_else(|_| Err(MathRuntimeError::Panic("ready block worker panicked".into())));
                        outcomes.push((index, result));
                    }
                    Ok::<_, MathRuntimeError>((outcomes, spawn_error))
                })? };
                retained.clear();
                retained.release();
                // Reconcile every joined child even when a sibling failed. No executed
                // native work disappears behind the first canonical error.
                let mut wave_error = spawn_failure;
                for (index, outcome) in outcomes {
                    let report = match outcome {
                        Ok(Outcome::Native(report)) => report,
                        Ok(Outcome::Rejected(error)) => { work = sum_work(work, WorkEvidence::default()); if wave_error.is_none() { wave_error = Some(MathRuntimeError::Shared(error)); } continue; }
                        Ok(_) => { work = sum_work(work, WorkEvidence::default()); if wave_error.is_none() { wave_error = Some(ProblemError::Internal("conditional native report unavailable".into()).into()); } continue; }
                        Err(error) => { work = sum_work(work, WorkEvidence::default()); if wave_error.is_none() { wave_error = Some(error); } continue; }
                    };
                    work = sum_work(work, report.evidence.work);
                    match report.evidence.work.iterations.and_then(|count| u32::try_from(count).ok()).and_then(|count| remaining_iterations.checked_sub(count)) {
                        Some(remaining) => remaining_iterations = remaining,
                        None if wave_error.is_none() => wave_error = Some(ProblemError::Unsupported("block aggregate iteration coverage unavailable or exceeded".into()).into()),
                        None => {}
                    }
                    if let Err(error) = capture_component(&report) && wave_error.is_none() { wave_error = Some(error); }
                    if let Err(cause) = super::super::initialization::commit_block(&mut values, &prepared.boundaries[index].boundary, Some(&report), &prepared.original.numerics.policy) {
                        if wave_error.is_none() {
                            component_observation = Some(super::super::strategy::observe_native(&report));
                            failure_cause = Some(cause);
                            wave_error = Some(ProblemError::Unsupported("conditional block candidate refused".into()).into());
                        }
                    }
                    components.push((index, report));
                    completed[index] = true;
                }
                if let Some(error) = wave_error { return Err(error); }
            }
            execution.check()?;
            let mut original = source.clone();
            original.values = values.clone();
            let evaluated = self.constant(
                Representation::Algebraic(original),
                &prepared.original.tolerances,
                &scope,
                budget,
            );
            let Outcome::Constant(mut report) = evaluated.inspect_err(|_| {
                work = sum_work(work, WorkEvidence::default());
            })?
            else {
                return Err(
                    ProblemError::Internal("original block assessment unavailable".into()).into(),
                );
            };
            let bounds = source
                .prepared
                .prepared
                .plan
                .structure()
                .variables()
                .iter()
                .filter(|variable| !variable.fixed)
                .zip(&prepared.original.tolerances.variables)
                .map(|(variable, tolerance)| {
                    let value = values.scalars[&variable.port.id];
                    Violation {
                        id: variable.port.id,
                        physical: quality::interval(
                            value,
                            variable.lower.unwrap_or(f64::NEG_INFINITY),
                            variable.upper.unwrap_or(f64::INFINITY),
                        ),
                        tolerance: *tolerance,
                    }
                })
                .collect();
            report.quality = Quality::new(report.quality.rows, bounds, vec![])?;
            report.coordinates = columns.iter().map(|id| (*id, values.scalars[id])).collect();
            work = sum_work(work, report.work);
            report.work = work;
            components.sort_unstable_by_key(|(index, _)| *index);
            report.components = components.into_iter().map(|(_, report)| report).collect();
            // Until every component and the original evaluation are complete, retain the
            // entire pre-admitted construction grant. Partition once while it is unique;
            // surplus drops without releasing/reacquiring any live report capacity.
            let retained_bytes = prepared.completed_result_bytes(&report)?;
            let mut leases = result_owner.partition(&[retained_bytes]).map_err(|_| {
                ProblemError::Internal("block result owner escaped before completion".into())
            })?;
            let owner = leases
                .pop()
                .ok_or_else(|| ProblemError::Internal("block result owner absent".into()))?;
            for component in &mut report.components {
                component.retain_owner(owner.clone());
            }
            report.owner = Some(owner);
            execution.check()?;
            Ok(Outcome::Constant(report))
        })();
        retained.clear();
        result.map_err(|error| {
            let cause = failure_cause.unwrap_or_else(|| Arc::new(error.into_problem()));
            let observed = WorkObservation {
                attempts: 1,
                evaluations: work.evaluations,
                iterations: work.iterations,
                factorizations: work.factorizations,
                proof_steps: work.proof_steps,
            };
            match component_observation {
                Some(observation) => {
                    super::super::strategy::EffectFailure::component(cause, observed, observation)
                }
                None => super::super::strategy::EffectFailure::observed(cause, observed),
            }
        })
    }
}
fn binding_bytes(bound: &pse_compiler::workspace::PreparedCase) -> usize {
    2 * size_of_val(bound)
        + 2048
        + bound.binding_bytes()
        + bound.presolve.bytes()
        + bound.provenance_bytes()
        + bound.derivation.retained_bytes()
        + bound.derived.retained_bytes()
        + bound
            .coefficients
            .as_ref()
            .map_or(0, |c| c.retained_bytes())
}
fn known_zero() -> WorkEvidence {
    WorkEvidence {
        evaluations: Some(0),
        iterations: Some(0),
        factorizations: Some(0),
        proof_steps: Some(0),
    }
}
fn sum_work(a: WorkEvidence, b: WorkEvidence) -> WorkEvidence {
    let add = |a: Option<u64>, b: Option<u64>| a.zip(b).and_then(|(a, b)| a.checked_add(b));
    WorkEvidence {
        evaluations: add(a.evaluations, b.evaluations),
        iterations: add(a.iterations, b.iterations),
        factorizations: add(a.factorizations, b.factorizations),
        proof_steps: add(a.proof_steps, b.proof_steps),
    }
}
