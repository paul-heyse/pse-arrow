// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Compiler-issued conditional equations executed in one original task.
use super::*;
use pse_kernels::ExecutionScope;
use pse_model::strategy::{MechanismKind, WorkObservation};

/// A complete structural schedule; every block uses the same admitted native backend.
#[derive(Clone, Debug)]
pub struct PreparedBlocks {
    original: Box<PreparedSolve>,
    blocks: Vec<PreparedSolve>,
    boundaries: Arc<Vec<pse_compiler::workspace::PreparedBlock>>,
    key: pse_ids::ContentHash,
    scope: ExecutionScope,
    backend: Backend,
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
    /// Single admitted adapter shared by every conditional block.
    pub fn backend(&self) -> Backend {
        self.backend
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
        if self.profile.controls.start != StartPolicy::NoPriorStart
            || self.profile.sensitivity.is_some()
            || !matches!(self.profile.convexity, ConvexityPolicy::Exact)
            || matches!(
                self.profile.presolve,
                native::presolve::Policy::Explicit { .. }
            )
            || self.backend().is_none_or(|backend| {
                !matches!(
                    execution::adapter(backend).representation(),
                    execution::Representation::Roots | execution::Representation::Nlp
                )
            })
        {
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
        let demand = compiled
            .initialization_allocation_bound()?
            .unwrap_or(self.policy.workspace_bytes);
        let job =
            self.job_retained_scoped(1, demand, control.clone(), scope.deadline(), move |flag| {
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
            });
        tokio::pin!(job);
        let (products, lease) = tokio::select! { result=&mut job=>result?, ()=driver.cancelled()=>{ control.cancel(); let _=job.await; return Err(MathRuntimeError::Cancelled); } };
        let owner = self.shared_product(
            vec![41, Arc::as_ptr(&products) as usize],
            products.clone(),
            lease,
            vec![source.prepared.owner.clone()],
        )?;
        let backend = original
            .backend()
            .ok_or_else(|| ProblemError::Unsupported("blocks require native equations".into()))?;
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
            let demand = view
                .binding_allocation_bound(&values)?
                .unwrap_or(self.policy.worker_bytes);
            if demand > self.policy.worker_bytes {
                return Err(MathRuntimeError::Limit(
                    "block binding construction capacity",
                ));
            }
            let binding = self.job_retained_scoped(
                1,
                demand,
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
            let preparation = Self::own_binding(
                owner.clone(),
                bound,
                lease,
                Arc::new(std::sync::OnceLock::from(executable.clone())),
            );
            let mut profile = original.profile.clone();
            profile.presolve = native::presolve::Policy::Off;
            profile.intent = SolveIntent::Initialize;
            profile.controls.reuse = ReusePolicy::Fresh;
            let step = self
                .prepare_conditional(
                    preparation,
                    executable,
                    source.values.clone(),
                    source.providers.clone(),
                    profile,
                    original.numerics.clone(),
                    Route::Native(backend),
                    original.snapshot.clone(),
                    &scope,
                    driver,
                )
                .await?
                .within_task(scope.clone())?;
            key.hash(&step.preparation_identity()?);
            steps.push(step);
        }
        Ok(PreparedBlocks {
            original: Box::new(original),
            blocks: steps,
            boundaries: products,
            key: key.finish_hash(),
            scope,
            backend,
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
            for (template, block) in prepared.blocks.iter().zip(prepared.boundaries.iter()) {
                execution.check()?;
                let mut step = template.clone();
                let Representation::Algebraic(local) = &mut step.representation else {
                    return Err(ProblemError::Internal(
                        "conditional representation changed".into(),
                    )
                    .into());
                };
                let bound = local.prepared.prepared.rebind(&values, &execution.cancel)?;
                let bytes = bound.rebind_allocation_bytes(&local.prepared.prepared);
                local.prepared = Self::own_rebind(
                    &local.prepared,
                    bound,
                    self.reserve("math:block-rebind", bytes)?,
                );
                local.values = values.clone();
                step.compatibility = Some(compatibility(
                    &local.prepared.prepared.plan,
                    &values,
                    &step.profile,
                    &step.numerics,
                    prepared.backend,
                    &local.providers,
                    &step.snapshot,
                )?);
                let outcome = self.run_step(step, execution.clone(), None, retained, budget);
                retained.clear();
                let outcome = match outcome {
                    Ok(outcome) => outcome,
                    Err(error) => {
                        work = sum_work(work, WorkEvidence::default());
                        return Err(error);
                    }
                };
                let Outcome::Native(report) = outcome else {
                    work = sum_work(work, WorkEvidence::default());
                    return Err(match outcome {
                        Outcome::Rejected(error) => MathRuntimeError::Shared(error),
                        _ => ProblemError::Internal("conditional native report unavailable".into())
                            .into(),
                    });
                };
                work = sum_work(work, report.evidence.work);
                if let Err(cause) = super::super::initialization::commit_block(
                    &mut values,
                    &block.boundary,
                    Some(&report),
                    &prepared.original.numerics.policy,
                ) {
                    component_observation = Some(super::super::strategy::observe_native(&report));
                    failure_cause = Some(cause);
                    return Err(ProblemError::Unsupported(
                        "conditional block candidate refused".into(),
                    )
                    .into());
                }
                components.push(report);
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
            report.components = components;
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
