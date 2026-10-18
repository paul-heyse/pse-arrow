// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Compiler-issued conditional equations executed in one original task.
use super::*;
use pse_kernels::ExecutionScope;
use pse_model::strategy::{MechanismKind, WorkObservation};
use std::collections::{BTreeMap, BTreeSet};
use futures_util::{StreamExt, FutureExt};
use super::super::initialization::{Dependencies, Iterations, consumed_inputs};

/// A complete structural schedule with contextually admitted per-block native profiles.
#[derive(Clone, Debug)]
pub struct PreparedBlocks {
    original: Box<PreparedSolve>,
    blocks: Vec<PreparedSolve>,
    dependencies: Arc<Dependencies>,
    _dependency_owner: Arc<pse_columnar::AllocationLease>,
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
    #[cfg(test)]
    pub(super) fn canonical_test_columns(&self) -> Vec<Vec<pse_ids::SemanticId>> {
        self.boundaries.iter().map(|block| block.boundary.members.columns.clone()).collect()
    }
    #[cfg(test)]
    pub(super) fn independent_test_chain(&self) -> (usize, usize, usize) { self.dependencies.independent_chain() }
    #[cfg(test)]
    pub(super) fn fence_completion(&mut self, slow: usize) -> (Arc<tokio::sync::Notify>, tokio::sync::mpsc::Receiver<usize>) {
        Arc::make_mut(&mut self.dependencies).fence(slow)
    }
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
    pub fn workers(&self) -> usize {
        self.workers
    }
    pub(crate) fn threads(&self) -> usize {
        self.threads
    }
    /// Complete actual adapter inventory for the contextually resolved block profiles.
    pub fn backends(&self) -> &[Backend] {
        &self.backends
    }
    /// Effective profiles keyed by compiler identity, inspectable before execution.
    pub fn profiles(
        &self,
    ) -> impl Iterator<Item = (pse_structural::incidence::BlockId, &SolverProfile)> {
        self.boundaries
            .iter()
            .zip(&self.blocks)
            .map(|(block, step)| (block.boundary.id, &step.profile))
    }
    pub(crate) fn foreign_bytes(&self) -> usize {
        self.foreign_bytes
    }
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
    pub(crate) async fn validate_prepared_block_overrides(
        self: &Arc<Self>,
        prepared: &Preparation,
        profile: &SolverProfile,
        numerics: &ResolvedNumericalPolicy,
    ) -> Result<(), MathRuntimeError> {
        let source = prepared.prepared.clone();
        let construction = super::super::jobs::ConstructionEstimate::from_bound(
            source.automatic_alternatives_allocation_bound()?,
            self.policy.workspace_bytes,
        );
        let requested = profile.clone();
        let (boundaries, _inventory_lease) = self
            .job_retained_construction(
                1,
                construction,
                Cancellation::default(),
                None,
                move |flag| {
                    let pse_compiler::workspace::Alternative::Available(schedule) =
                        source.automatic_alternatives(&flag)?.initialization
                    else {
                        return Err(ProblemError::Unsupported(
                            "explicit block overrides require complete compiler block inventory"
                                .into(),
                        )
                        .into());
                    };
                    requested.validate_block_ids(schedule.blocks.iter().map(|block| block.id))?;
                    let mut boundaries = Vec::with_capacity(requested.block_overrides.len());
                    for block in schedule.blocks {
                        if requested.block_overrides.contains_key(&block.id) {
                            boundaries.push(block);
                        }
                    }
                    // Retain only authenticated requested boundaries. Discovery scratch and
                    // dependency sets die here, before the independently admitted fact job.
                    let overflow = || MathRuntimeError::Limit("block inventory retained extent");
                    let descriptors = boundaries
                        .capacity()
                        .checked_mul(size_of::<pse_structural::initialization::Block>())
                        .and_then(|n| {
                            n.checked_add(size_of::<Vec<pse_structural::initialization::Block>>())
                        })
                        .ok_or_else(overflow)?;
                    let bytes = boundaries
                        .iter()
                        .try_fold(descriptors, |n, block| {
                            block
                                .members
                                .rows
                                .capacity()
                                .checked_add(block.members.columns.capacity())?
                                .checked_add(block.inputs.capacity())?
                                .checked_mul(size_of::<pse_ids::SemanticId>())
                                .and_then(|members| n.checked_add(members))
                        })
                        .ok_or_else(overflow)?;
                    Ok((boundaries, bytes))
                },
            )
            .await?;
        let snapshot = execution::Snapshot::observe(&execution::LINKED);
        for boundary in boundaries {
            let source = prepared.prepared.clone();
            let id = boundary.id;
            let overflow = || MathRuntimeError::Limit("selected block oracle extent");
            let oracle_bytes = boundary
                .members
                .columns
                .len()
                .checked_mul(2 * (size_of::<native::Variable>() + size_of::<pse_ids::SemanticId>()))
                .and_then(|n| {
                    boundary
                        .members
                        .rows
                        .len()
                        .checked_mul(
                            2 * (size_of::<pse_ids::SemanticId>()
                                + size_of::<pse_structural::incidence::Constraint>()),
                        )
                        .and_then(|rows| n.checked_add(rows))
                })
                .and_then(|n| {
                    n.checked_add(
                        size_of::<native::OracleContract>() + size_of::<routing::Structure>() + 128,
                    )
                })
                .ok_or_else(overflow)?;
            let bound = source
                .block_admission_allocation_bound(&boundary)?
                .map(|n| n.checked_add(oracle_bytes).ok_or_else(overflow))
                .transpose()?;
            let construction = super::super::jobs::ConstructionEstimate::from_bound(
                bound,
                self.policy.workspace_bytes,
            );
            let ((contract, available, structure), _facts_lease) = self
                .job_retained_construction(
                    1,
                    construction,
                    Cancellation::default(),
                    None,
                    move |flag| {
                        let rows = boundary
                            .members
                            .rows
                            .iter()
                            .copied()
                            .collect::<BTreeSet<_>>();
                        let columns = boundary
                            .members
                            .columns
                            .iter()
                            .copied()
                            .collect::<BTreeSet<_>>();
                        let available = source
                            .plan
                            .conditional_available_order(&rows, &columns, &flag)?;
                        let contract = native::OracleContract {
                            identity: boundary.id.0,
                            variables: source
                                .plan
                                .columns()
                                .iter()
                                .filter(|column| columns.contains(column))
                                .map(|column| {
                                    native::assembled::variable_contract(&source.plan, *column)
                                })
                                .collect(),
                            rows: source
                                .plan
                                .structure()
                                .rows()
                                .iter()
                                .filter(|row| rows.contains(&row.id))
                                .map(|row| row.id)
                                .collect(),
                            derivatives: source.plan.order(),
                            smoothness: available,
                        };
                        let witness = source
                            .structure
                            .conditional_witness(&boundary)
                            .map_err(|error| ProblemError::Contract(error.to_string()))?;
                        let structure = routing::Structure {
                            variables: boundary.members.columns.clone(),
                            equations: source
                                .plan
                                .structure()
                                .rows()
                                .iter()
                                .filter(|row| rows.contains(&row.id))
                                .map(|row| pse_structural::incidence::Constraint {
                                    id: row.id,
                                    lower: row.lower.is_finite().then_some(row.lower),
                                    upper: row.upper.is_finite().then_some(row.upper),
                                })
                                .collect(),
                            witness: Arc::new(witness).into(),
                        };
                        let scope_members = rows
                            .len()
                            .checked_add(columns.len())
                            .and_then(|n| n.checked_add(boundary.inputs.len()))
                            .ok_or(MathRuntimeError::Limit("block scope fact extent"))?;
                        let scope_bytes = scope_members
                            .checked_mul(
                                11 * size_of::<pse_ids::SemanticId>() + 16 * size_of::<usize>(),
                            )
                            .ok_or(MathRuntimeError::Limit("block scope fact extent"))?;
                        let overflow =
                            || MathRuntimeError::Limit("block admission retained extent");
                        let owned = [
                            contract
                                .variables
                                .capacity()
                                .checked_mul(size_of::<native::Variable>()),
                            contract
                                .rows
                                .capacity()
                                .checked_mul(size_of::<pse_ids::SemanticId>()),
                            structure
                                .variables
                                .capacity()
                                .checked_mul(size_of::<pse_ids::SemanticId>()),
                            structure
                                .equations
                                .capacity()
                                .checked_mul(size_of::<pse_structural::incidence::Constraint>()),
                            Some(structure.witness.retained_bytes()),
                            Some(scope_bytes),
                        ]
                        .into_iter()
                        .try_fold(
                            size_of_val(&contract) + size_of_val(&structure) + 64,
                            |n, bytes| n.checked_add(bytes?),
                        )
                        .ok_or_else(overflow)?;
                        Ok(((contract, available, structure), owned))
                    },
                )
                .await?;
            let mut effective = profile.block_profile(id)?;
            effective.intent = SolveIntent::Initialize;
            effective.presolve = native::presolve::Policy::Off;
            super::super::initialization::conditional_contract_route(
                &snapshot,
                &contract,
                available,
                contract.derivatives,
                structure,
                &effective,
                numerics,
            )?;
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
        let control = Cancellation::default();
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
            let consumed = consumed_inputs(&view.plan, executable.assembly.input_ids());
            let value_owner = self.reserve("math:prepared-block-inputs", consumed.len().saturating_mul(96).saturating_add(256))?;
            let values = CaseValues { scalars: consumed.into_iter().map(|id| {
                source.values.scalars.get(&id).copied().map(|value| (id, value))
                    .ok_or_else(|| ProblemError::Contract(format!("missing initial block binding {id}")))
            }).collect::<Result<_, _>>()? };
            let values = pse_math::SharedAllocation::from(Arc::new(values)).with_owner(value_owner);
            let binding_values = values.clone();
            let control = Cancellation::default();
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
                    let bound = view.bind(quantities, preconditions, &binding_values, &flag)?;
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
                &original.snapshot,
                block,
                &executable,
                &profile,
                &original.numerics,
            )?;
            let Route::Native(actual_backend) = route else {
                return Err(ProblemError::Unsupported(
                    "block requires admitted native equations".into(),
                )
                .into());
            };
            // The block runner debits the original finite iteration allowance
            // from each completed native report. POUNCE's current adapter does
            // not expose a complete per-attempt iteration total for every exit
            // and method. Decline this optional preparation after contextual
            // routing, before any block runs; the existing strategy can still
            // execute the unchanged original native operation. Do not infer
            // coverage from the original backend, progress or a cached summary.
            if actual_backend == Backend::Pounce {
                return Err(ProblemError::Unsupported(
                    "POUNCE block aggregate iteration coverage unavailable".into(),
                )
                .into());
            }
            if !backends.contains(&actual_backend) {
                backends.push(actual_backend);
            }
            let step = self
                .prepare_conditional(
                    preparation,
                    executable,
                    values,
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
        let descriptor_bytes = products.iter().try_fold(4096usize, |total, block| {
            total.checked_add(block.plan.structure().variables().len().checked_add(block.plan.structure().parameters().len())?.checked_add(block.boundary.inputs.len())?.checked_mul(128)?)
        }).ok_or(MathRuntimeError::Limit("block dependency descriptor extent"))?;
        let dependency_owner = self.reserve("math:solve-block-dependencies", descriptor_bytes)?;
        let consumed = steps.iter().map(|step| {
            let Representation::Algebraic(local) = &step.representation else {
                return Err(ProblemError::Internal("conditional representation changed".into()));
            };
            let executable = local.case.as_ref().ok_or_else(|| ProblemError::Internal("conditional executable absent".into()))?;
            Ok(consumed_inputs(&local.prepared.prepared.plan, executable.assembly.input_ids()))
        }).collect::<Result<Vec<_>, ProblemError>>()?;
        let dependencies = Arc::new(Dependencies::prepare(products.iter().map(|block| &block.boundary), consumed)?);
        // A completion schedule can overlap blocks absent from any canonical wave.
        // Independent top-k sums bound every allowed live set, without enumerating it.
        let mut teams = Vec::with_capacity(steps.len());
        let mut residents = Vec::with_capacity(steps.len());
        for step in &steps {
            let backend = step.backend().ok_or_else(|| ProblemError::Internal("missing prepared block team".into()))?;
            let demand = super::super::initialization::BlockWaveDemand::new(std::iter::once((backend, step.profile.controls.clone())), &self.policy)?;
            demand.admit(self.cores)?;
            teams.push(demand.threads);
            residents.push(demand.foreign.checked_add(demand.stacks).ok_or(MathRuntimeError::Limit("block live resident extent"))?);
        }
        teams.sort_unstable_by(|a, b| b.cmp(a));
        residents.sort_unstable_by(|a, b| b.cmp(a));
        let threads = teams.into_iter().take(workers).try_fold(0usize, |n, team| n.checked_add(team)).ok_or(MathRuntimeError::Limit("block live team extent"))?.min(self.cores);
        let foreign_bytes = residents.into_iter().take(workers).try_fold(0usize, |n, bytes| n.checked_add(bytes))
            .and_then(|n| n.checked_add(workers.checked_mul(self.policy.stack_bytes)?))
            .ok_or(MathRuntimeError::Limit("block live resident extent"))?;
        Ok(PreparedBlocks {
            original: Box::new(original),
            blocks: steps,
            dependencies,
            _dependency_owner: dependency_owner,
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
    async fn run_block_owner(
        self: Arc<Self>, index: usize, step: PreparedSolve, execution: Execution,
        admission: Option<Arc<super::super::strategy::admission::TaskAdmission>>,
        #[cfg(test)] dependencies: Arc<Dependencies>,
    ) -> (usize, Result<Outcome, MathRuntimeError>) {
        let result = async {
            let affinity = step.preparation_identity()?;
            let backend = step.backend().ok_or_else(|| ProblemError::Internal("missing block backend".into()))?;
            let cores = step.profile.controls.threads;
            let scope = execution.scope()?;
            let cancel = crate::CancelSource::new().with_deadline(scope.deadline());
            let session = self.independent_session(affinity)?;
            let service = self.clone();
            let outcome = session.conditional_block(cores, backend, scope.clone(), &cancel, move |retained, _, local| {
                let budget = admission.map_or_else(|| local.clone(), |owner| local.with_admission(owner));
                service.run_step(step, execution, None, retained, &budget)
            }).await;
            let _ = session.park_independent(affinity, Some(scope.clone()), &cancel).await;
            #[cfg(test)]
            dependencies.delivered(index, &scope, &cancel).await;
            outcome
        }.await;
        (index, result)
    }
    /// One finite schedule on the current worker, followed by fresh full-original evaluation.
    pub(crate) fn blocks_step(
        self: &Arc<Self>,
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
            let mut values = source.values.as_ref().clone();
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
            let width = prepared.workers.min(self.policy.jobs.saturating_sub(1).max(1));
            let parallel = width > 1;
            let runtime = tokio::runtime::Handle::try_current().map_err(|e| MathRuntimeError::Infrastructure(e.to_string()))?;
            let mut iterations = Iterations::new(prepared.original.profile.controls.iterations);
            let mut readiness = prepared.dependencies.start();
            let mut outcomes = BTreeMap::new();
            let mut errors = BTreeMap::new();
            let mut refused = BTreeMap::new();
            if parallel { execution::pause_compute(); }
            let dispatch = async {
                let mut running = futures_util::stream::FuturesUnordered::new();
                let mut active_threads = 0usize;
                let mut stopped = false;
                loop {
                    if let Err(error) = execution.check() { errors.entry(usize::MAX).or_insert(error.into()); stopped = true; }
                    let mut deferred = Vec::new();
                    let mut immediate = None;
                    while !stopped && running.len() < width {
                        let Some(index) = readiness.take() else { break; };
                        let template = &prepared.blocks[index];
                        if active_threads + template.profile.controls.threads > prepared.threads { deferred.push(index); continue; }
                        let allowance = match iterations.reserve(index, template.controls().iterations, width - running.len()) {
                            Ok(allowance) => allowance,
                            Err(error) => {
                                readiness.defer(index);
                                if running.is_empty() { errors.insert(index, error.into()); stopped = true; }
                                break;
                            }
                        };
                        let rebound = (|| {
                            let mut step = template.clone();
                            step.profile.controls.iterations = allowance;
                            let Representation::Algebraic(local) = &mut step.representation else { return Err(ProblemError::Internal("conditional representation changed".into()).into()); };
                            let input_owner = self.reserve("math:block-dispatch-inputs", prepared.dependencies.binding_bytes(index))?;
                            let inputs: pse_math::SharedAllocation<CaseValues> = Arc::new(prepared.dependencies.inputs(index, &values)?).into();
                            let inputs = inputs.with_owner(input_owner);
                            let bound = local.prepared.prepared.rebind(&inputs, &execution.cancel)?;
                            let bytes = bound.rebind_allocation_bytes(&local.prepared.prepared);
                            local.prepared = self.own_rebind(&local.prepared, bound, self.reserve("math:block-rebind", bytes)?)?;
                            local.values = inputs;
                            step.compatibility = Some(compatibility(&local.prepared.prepared.plan, &local.values,
                                &step.profile, &step.numerics, template.backend().ok_or_else(|| ProblemError::Internal("missing block backend".into()))?,
                                &local.providers, &step.snapshot)?);
                            Ok::<_, MathRuntimeError>(step)
                        })();
                        let step = match rebound { Ok(step) => step, Err(error) => { let _ = iterations.complete(index, None); errors.insert(index, error); stopped = true; break; } };
                        let mut child_execution = execution.clone();
                        child_execution.memory = Some(self.policy.foreign_allowance(&step.profile.controls));
                        if parallel {
                            active_threads += step.profile.controls.threads;
                            running.push(self.clone().run_block_owner(index, step, child_execution, budget.admission(), #[cfg(test)] prepared.dependencies.clone()));
                        } else {
                            // The single-owner route retains its original native session.
                            let result = self.run_step(step, child_execution, None, retained, budget);
                            immediate = Some((index, result));
                            break;
                        }
                    }
                    for index in deferred { readiness.defer(index); }
                    let completion = if parallel { running.next().await } else { immediate };
                    let Some((index, outcome)) = completion else { break; };
                    if parallel { active_threads -= prepared.blocks[index].profile.controls.threads; }
                    let actual = match &outcome { Ok(Outcome::Native(report)) => report.evidence.work.iterations, _ => None };
                    if let Err(error) = iterations.complete(index, actual) { errors.entry(index).or_insert(error.into()); stopped = true; }
                    match &outcome {
                        Ok(Outcome::Native(report)) => {
                            match super::super::initialization::commit_block(&mut values, &prepared.boundaries[index].boundary, Some(report), &prepared.original.numerics.policy) {
                                Ok(()) if !errors.contains_key(&index) => { readiness.commit(index)?; }
                                Ok(()) => {},
                                Err(cause) => { refused.insert(index, cause); stopped = true; }
                            }
                        }
                        _ => stopped = true,
                    }
                    outcomes.insert(index, outcome);
                    if !parallel && (stopped || readiness.remaining == 0) { break; }
                }
                Ok::<_, MathRuntimeError>(())
            };
            let dispatched = if parallel { runtime.block_on(dispatch) } else {
                dispatch.now_or_never().ok_or_else(|| ProblemError::Internal("serial block unexpectedly suspended".into()))?
            };
            let resumed = if parallel { execution::resume_compute(&execution).map_err(MathRuntimeError::from) } else { Ok(()) };
            // Canonical reduction observes every issued child, even after an unrelated
            // failure stopped dispatch. Only complete original assessment accepts a stage.
            let mut first_error = None;
            for (index, outcome) in outcomes {
                let report = match outcome {
                    Ok(Outcome::Native(report)) => Some(report),
                    Ok(Outcome::Rejected(cause)) => { errors.entry(index).or_insert(MathRuntimeError::Shared(cause)); None },
                    Ok(_) => { errors.entry(index).or_insert(ProblemError::Internal("conditional native report unavailable".into()).into()); None },
                    Err(error) => { errors.entry(index).or_insert(error); None },
                };
                if let Some(report) = report {
                    work = sum_work(work, report.evidence.work);
                    if let Err(error) = capture_component(&report) { errors.entry(index).or_insert(error); }
                    if let Some(cause) = refused.remove(&index) {
                        if first_error.is_none() && errors.first_key_value().is_none_or(|(&other, _)| index < other) {
                            component_observation = Some(super::super::strategy::observe_native(&report)); failure_cause = Some(cause);
                        }
                        errors.entry(index).or_insert(ProblemError::Unsupported("conditional block candidate refused".into()).into());
                    }
                    components.push((index, report));
                } else { work = sum_work(work, WorkEvidence::default()); }
                if first_error.is_none() { first_error = errors.remove(&index); }
            }
            if let Some(error) = first_error.or_else(|| errors.into_values().next()) { return Err(error); }
            dispatched?;
            resumed?;
            if readiness.remaining != 0 { return Err(ProblemError::Contract("incomplete block dependency execution".into()).into()); }
            execution.check()?;
            let mut original = source.clone();
            original.values = Arc::new(values.clone()).into();
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
