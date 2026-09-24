// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Conditional initialization and finite supplied continuation; KINSOL owns iteration.
use super::{
    ExecutableCase, ExecutionWorker, MathRuntimeError, MathService, Workspace, solves::SolveHandle,
};
use pse_backend_native::{self as native, kinsol, quality::Tolerances, solve::*};
use pse_compiler::workspace::Profile;
use pse_engine::cache_service::flight::FlightCancellation;
use pse_ids::{FramedHasher, SemanticId};
use pse_math::binding::CaseValues;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// Compiled predecessor-ordered blocks, with explicit conditional input coordinates.
#[derive(Debug)]
pub struct PreparedInitialization {
    blocks: Vec<(pse_structural::initialization::Block, Arc<ExecutableCase>)>,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl PreparedInitialization {
    /// Inspect the immutable conditional boundaries before executing native work.
    pub fn boundaries(&self) -> impl Iterator<Item = &pse_structural::initialization::Block> {
        self.blocks.iter().map(|(b, _)| b)
    }
}
/// One source-attributed block attempt. A failed block never commits trial coordinates.
#[derive(Clone, Debug)]
pub struct BlockAttempt {
    /// Supplied continuation stage, zero-based.
    pub stage: usize,
    /// Exact original rows, solved columns and predecessor inputs.
    pub boundary: pse_structural::initialization::Block,
    /// Faithful native report or typed pre-execution failure.
    pub result: Result<Box<SolveReport>, Arc<MathRuntimeError>>,
    /// Whether its original-quality-validated coordinates were committed.
    pub committed: bool,
}
/// Initialized values and complete bounded attempt history, without an optimum claim.
#[derive(Debug)]
pub struct InitializationReport {
    /// Last committed values; failed block trials are absent.
    pub values: CaseValues,
    /// Actual block attempts in predecessor and continuation order.
    pub attempts: Vec<BlockAttempt>,
    /// Number of fully committed stages.
    pub completed_stages: usize,
    /// Cancellation request observed before the next block.
    pub cancelled: bool,
    _owner: Arc<pse_columnar::AllocationLease>,
}
/// Explicit finite continuation data and physical acceptance scales.
#[derive(Clone, Debug)]
pub struct InitializationProfile {
    /// Same finite execution policy used by other native attempts.
    pub controls: Controls,
    /// KLU, bounded dense or matrix-free SPGMR; no custom factorization.
    pub linear: kinsol::Linear,
    /// Per-coordinate physical variable tolerances.
    pub variable_tolerances: BTreeMap<SemanticId, f64>,
    /// Per-original-row physical residual tolerances.
    pub row_tolerances: BTreeMap<SemanticId, f64>,
    /// Finite prescribed parameter/fixed-coordinate replacements. Use one empty map
    /// for ordinary initialization. Successful prior variable values seed each stage.
    pub stages: Vec<BTreeMap<SemanticId, f64>>,
}
/// Accounted terminal report for an explicitly declared map or Picard splitting.
#[derive(Debug)]
pub struct DeclaredRootReport {
    /// Native root outcome with original residual validation.
    pub report: SolveReport,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl MathService {
    /// Execute a declared root/map factory on its owning admitted worker. Causal
    /// sweeps may use nested native unit calculations on this same admission; they
    /// must not recursively request CPU permits. Mutable oracles never enter Salsa.
    pub fn solve_declared_root(
        self: &Arc<Self>,
        contract: native::OracleContract,
        initial: Vec<f64>,
        settings: kinsol::Settings,
        controls: Controls,
        tolerances: Tolerances,
        factory: impl FnOnce(Execution) -> Result<kinsol::Function, native::ProblemError>
        + Send
        + 'static,
    ) -> Result<SolveHandle<DeclaredRootReport>, MathRuntimeError> {
        controls.validate()?;
        contract.validate(pse_kernels::DerivativeOrder::Value)?;
        let n = contract.variables.len();
        tolerances.validate(n, contract.rows.len())?;
        if controls.threads != 1 || initial.len() != n || initial.iter().any(|v| !v.is_finite()) {
            return Err(native::ProblemError::Contract(
                "declared serial root dimensions/controls".into(),
            )
            .into());
        }
        let bytes = n
            .checked_add(contract.rows.len())
            .and_then(|v| v.checked_mul(512))
            .and_then(|v| v.checked_add(controls.report_allowance().ok()?))
            .ok_or(MathRuntimeError::Limit("declared root result allowance"))?;
        let owner = self.reserve("math:declared-root-result", bytes)?;
        let cancel = FlightCancellation::default();
        let control = cancel.clone();
        let progress = Arc::new(Progress::new(controls.history));
        let events = progress.clone();
        let service = self.clone();
        let (tx, receiver) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let result = service
                .job(1, service.policy.worker_bytes, control, move |flag| {
                    let mut execution = Execution::new(flag, &controls);
                    execution.progress = events;
                    let function = factory(execution.clone())?;
                    let actual = function.contract();
                    if actual.identity != contract.identity
                        || actual.rows != contract.rows
                        || actual.variables.len() != n
                        || actual
                            .variables
                            .iter()
                            .zip(&contract.variables)
                            .any(|(a, b)| {
                                a.id != b.id
                                    || a.lower.to_bits() != b.lower.to_bits()
                                    || a.upper.to_bits() != b.upper.to_bits()
                            })
                    {
                        return Err(native::ProblemError::Contract(
                            "root factory differs from admitted source contract".into(),
                        )
                        .into());
                    }
                    let stamp = Compatibility {
                        layout: contract.identity,
                        data: contract.identity,
                        backend: Backend::Kinsol,
                    };
                    let mut session =
                        kinsol::Session::new(function, settings, execution.clone(), stamp)?;
                    let report =
                        session.solve(&initial, &controls, execution, &tolerances, None)?;
                    drop(session);
                    Ok(DeclaredRootReport {
                        report: report.with_owner(owner.clone()),
                        _owner: owner,
                    })
                })
                .await;
            let _ = tx.send(result);
        });
        Ok(SolveHandle {
            cancel,
            receiver: Some(receiver),
            progress,
        })
    }
    /// Prepare conditional demands through the same Salsa database and artifact cache.
    pub async fn prepare_initialization(
        self: &Arc<Self>,
        workspace: Workspace,
        id: SemanticId,
        profile: Profile,
    ) -> Result<PreparedInitialization, MathRuntimeError> {
        let owner = self.reserve("math:initialization-products", self.policy.workspace_bytes)?;
        let products = self
            .job(
                1,
                self.policy.stack_bytes,
                FlightCancellation::default(),
                move |_| {
                    let _lease = workspace.lease;
                    workspace
                        .compiler
                        .lock()
                        .map_err(|_| {
                            MathRuntimeError::Infrastructure("compiler lock poisoned".into())
                        })?
                        .prepare_initialization_blocks(id, profile)
                        .map_err(Into::into)
                },
            )
            .await?;
        let mut blocks = Vec::with_capacity(products.len());
        for block in products.iter() {
            let mut artifacts = vec![];
            for request in block.artifacts.iter() {
                artifacts.push(self.artifact(request.clone()).await?);
            }
            let plan = Arc::new(block.plan.as_ref().clone().with_owner(owner.clone()));
            let assembly =
                Arc::new(plan.assemble(artifacts.iter().map(|a| a.program.clone()).collect())?);
            blocks.push((
                block.boundary.clone(),
                Arc::new(ExecutableCase {
                    assembly,
                    _artifacts: artifacts,
                    _owner: owner.clone(),
                }),
            ));
        }
        Ok(PreparedInitialization {
            blocks,
            _owner: owner,
        })
    }
    /// Admit the complete finite schedule once; each block is solved and validated by
    /// KINSOL before its values become predecessor inputs. No iteration is differentiated.
    pub fn initialize(
        self: &Arc<Self>,
        prepared: PreparedInitialization,
        values: CaseValues,
        providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        profile: InitializationProfile,
    ) -> Result<SolveHandle<InitializationReport>, MathRuntimeError> {
        profile.controls.validate()?;
        if profile.controls.threads != 1
            || profile.stages.is_empty()
            || profile
                .stages
                .len()
                .checked_mul(prepared.blocks.len())
                .is_none_or(|v| v > 4096)
        {
            return Err(MathRuntimeError::Limit(
                "serial finite initialization schedule",
            ));
        }
        let solved: BTreeSet<_> = prepared
            .boundaries()
            .flat_map(|b| b.members.columns.iter().copied())
            .collect();
        for stage in &profile.stages {
            if stage.iter().any(|(id, v)| {
                solved.contains(id) || !values.scalars.contains_key(id) || !v.is_finite()
            }) {
                return Err(native::ProblemError::Contract(
                    "continuation may only replace declared finite fixed/parameter inputs".into(),
                )
                .into());
            }
        }
        for (boundary, _) in &prepared.blocks {
            block_tolerances(boundary, &profile)?;
            if boundary
                .inputs
                .iter()
                .chain(&boundary.members.columns)
                .any(|id| !values.scalars.contains_key(id))
            {
                return Err(native::ProblemError::Contract(
                    "missing initialization boundary value".into(),
                )
                .into());
            }
        }
        let size = prepared.blocks.iter().try_fold(
            values.scalars.len().saturating_mul(64),
            |total, (b, _)| {
                b.members
                    .columns
                    .len()
                    .checked_add(b.members.rows.len())
                    .and_then(|v| v.checked_mul(512))
                    .and_then(|v| v.checked_add(profile.controls.report_allowance().ok()?))
                    .and_then(|v| v.checked_mul(profile.stages.len()))
                    .and_then(|v| total.checked_add(v))
                    .ok_or(MathRuntimeError::Limit("initialization result allowance"))
            },
        )?;
        let owner = self.reserve("math:initialization-results", size)?;
        let cancel = FlightCancellation::default();
        let control = cancel.clone();
        let progress = Arc::new(Progress::new(profile.controls.history));
        let events = progress.clone();
        let service = self.clone();
        let (tx, receiver) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let runner = service.clone();
            let result = service
                .job(1, service.policy.worker_bytes, control, move |flag| {
                    runner.run_initialization(
                        prepared, values, providers, profile, flag, events, owner,
                    )
                })
                .await;
            let _ = tx.send(result);
        });
        Ok(SolveHandle {
            cancel,
            receiver: Some(receiver),
            progress,
        })
    }
    fn run_initialization(
        &self,
        prepared: PreparedInitialization,
        mut values: CaseValues,
        providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        profile: InitializationProfile,
        flag: Arc<AtomicBool>,
        progress: Arc<Progress>,
        owner: Arc<pse_columnar::AllocationLease>,
    ) -> Result<InitializationReport, MathRuntimeError> {
        let mut attempts = vec![];
        let mut completed_stages = 0;
        'stages: for (stage, updates) in profile.stages.iter().enumerate() {
            values.scalars.extend(updates.iter().map(|(k, v)| (*k, *v)));
            for (boundary, case) in &prepared.blocks {
                if flag.load(Ordering::Acquire) {
                    break 'stages;
                }
                let result = (|| -> Result<Box<SolveReport>, MathRuntimeError> {
                    let providers = providers
                        .iter()
                        .map(|(k, f)| {
                            f.worker()
                                .map(|v| (*k, v))
                                .map_err(|e| native::ProblemError::Contract(e.to_string()))
                        })
                        .collect::<Result<_, _>>()?;
                    let ExecutionWorker {
                        worker,
                        _case,
                        _lease,
                    } = self.worker(case.clone(), providers, flag.clone())?;
                    let oracle = native::assembled::AlgebraicOracle::new(worker, values.clone())?;
                    oracle.admit_nle()?;
                    let tolerances = block_tolerances(boundary, &profile)?;
                    let initial: Vec<_> = boundary
                        .members
                        .columns
                        .iter()
                        .map(|id| values.scalars[id])
                        .collect();
                    let mut execution = Execution::new(flag.clone(), &profile.controls);
                    execution.progress = progress.clone();
                    let mut h = FramedHasher::new("pse.initialization.block.v1");
                    for id in &boundary.members.columns {
                        h.id(id);
                    }
                    for id in &boundary.members.rows {
                        h.id(id);
                    }
                    let layout = h.finish_hash();
                    let settings = kinsol::Settings {
                        strategy: kinsol::Strategy::LineSearch,
                        linear: profile.linear,
                        variable_scales: vec![1.0; initial.len()],
                        residual_scales: tolerances
                            .rows
                            .iter()
                            .map(|v| profile.controls.tolerance / v)
                            .collect(),
                        anderson: 0,
                        damping: 1.0,
                        setup_interval: 10,
                        step_tolerance: profile.controls.tolerance,
                    };
                    let mut session = kinsol::Session::new(
                        kinsol::Function::Equations(Box::new(oracle)),
                        settings,
                        execution.clone(),
                        Compatibility {
                            layout,
                            data: layout,
                            backend: Backend::Kinsol,
                        },
                    )?;
                    let report =
                        session.solve(&initial, &profile.controls, execution, &tolerances, None)?;
                    drop(session);
                    drop(_lease);
                    drop(_case);
                    Ok(Box::new(report.with_owner(owner.clone())))
                })()
                .map_err(Arc::new);
                let committed = commit_block(&mut values, boundary, result.as_deref().ok());
                attempts.push(BlockAttempt {
                    stage,
                    boundary: boundary.clone(),
                    result,
                    committed,
                });
                if !committed {
                    break 'stages;
                }
            }
            completed_stages += 1;
        }
        Ok(InitializationReport {
            values,
            attempts,
            completed_stages,
            cancelled: flag.load(Ordering::Acquire),
            _owner: owner,
        })
    }
}
fn block_tolerances(
    b: &pse_structural::initialization::Block,
    p: &InitializationProfile,
) -> Result<Tolerances, MathRuntimeError> {
    let get = |ids: &[SemanticId], map: &BTreeMap<SemanticId, f64>| {
        ids.iter()
            .map(|id| {
                map.get(id).copied().ok_or_else(|| {
                    native::ProblemError::Contract(format!(
                        "missing physical initialization tolerance {id}"
                    ))
                })
            })
            .collect::<Result<Vec<_>, _>>()
    };
    let t = Tolerances {
        variables: get(&b.members.columns, &p.variable_tolerances)?,
        rows: get(&b.members.rows, &p.row_tolerances)?,
        integrality: 1e-8,
    };
    t.validate(b.members.columns.len(), b.members.rows.len())?;
    Ok(t)
}
fn commit_block(
    values: &mut CaseValues,
    b: &pse_structural::initialization::Block,
    report: Option<&SolveReport>,
) -> bool {
    let Some(r) = report else { return false };
    if !matches!(
        r.termination.category,
        Termination::Success | Termination::Acceptable
    ) || !r.quality.as_ref().is_some_and(|q| q.feasible())
        || r.variables != b.members.columns
    {
        return false;
    }
    let Some(c) = &r.candidate else { return false };
    if c.primal.len() != b.members.columns.len() || c.primal.iter().any(|v| !v.is_finite()) {
        return false;
    }
    values.scalars.extend(
        b.members
            .columns
            .iter()
            .copied()
            .zip(c.primal.iter().copied()),
    );
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    fn id(v: u8) -> SemanticId {
        SemanticId::from_bytes([v; 16])
    }
    #[test]
    fn block_commit_is_atomic_and_requires_native_success_plus_original_quality() {
        let boundary = pse_structural::initialization::Block {
            members: pse_structural::incidence::Part {
                rows: vec![id(2)],
                columns: vec![id(1)],
            },
            inputs: vec![],
        };
        let contract = native::OracleContract {
            identity: pse_ids::ContentHash::from_bytes([1; 32]),
            variables: vec![native::Variable {
                id: id(1),
                lower: f64::NEG_INFINITY,
                upper: f64::INFINITY,
            }],
            rows: vec![id(2)],
            derivatives: pse_kernels::DerivativeOrder::First,
            smoothness: pse_kernels::DerivativeOrder::First,
        };
        let mut r = SolveReport::new(
            Backend::Kinsol,
            &contract,
            kinsol::termination(0),
            &Execution::new(Arc::new(AtomicBool::new(false)), &Controls::default()),
        );
        r.quality = Some(native::quality::Quality::new(vec![], vec![], vec![]).unwrap());
        r.candidate = Some(Candidate {
            primal: vec![2.0],
            objective: None,
            row_dual: None,
            bound_dual: None,
            reduced_costs: None,
            slacks: None,
        });
        let mut values = CaseValues {
            scalars: BTreeMap::from([(id(1), 1.0)]),
        };
        r.termination.category = Termination::Limit;
        assert!(!commit_block(&mut values, &boundary, Some(&r)));
        assert_eq!(values.scalars[&id(1)], 1.0);
        r.termination.category = Termination::Success;
        r.candidate.as_mut().unwrap().primal[0] = f64::NAN;
        assert!(!commit_block(&mut values, &boundary, Some(&r)));
        assert_eq!(values.scalars[&id(1)], 1.0);
        r.candidate.as_mut().unwrap().primal[0] = 2.0;
        assert!(commit_block(&mut values, &boundary, Some(&r)));
        assert_eq!(values.scalars[&id(1)], 2.0);
    }
}
