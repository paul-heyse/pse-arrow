// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded elastic explanations. Local obstruction never certifies infeasibility.
use super::*;
use crate::math::solves::Outcome;
use pse_backend_native::solve::{Qualification, SolveIntent};
use pse_modeling::specialize::{Formulation, Value};
use std::{
    collections::BTreeSet,
    time::{Duration, Instant},
};

/// Explicit physical row nominals weight the dimensionless L1 elastic objective.
#[derive(Clone, Debug)]
pub struct ModelingNonlinearPolicy {
    pub nominals: BTreeMap<SemanticId, f64>,
    pub penalty_tolerance: f64,
    pub maximum_attempts: usize,
    pub time_limit: Duration,
}
/// Evidence from one bounded local optimization, not a global verdict.
pub use pse_model::generated::enums::ModelingElasticObservation as ElasticObservation;
#[derive(Clone, Debug)]
pub struct ModelingElasticAttempt {
    pub omitted: BTreeSet<SemanticId>,
    pub observation: ElasticObservation,
    pub penalty: Option<f64>,
    pub result: Result<ModelingResult, Arc<WorkflowError>>,
    pub interruption: Option<BoundaryDiagnostic>,
}
impl ModelingElasticAttempt {
    /// Structured interruption, preparation failure or rejected numerical result.
    pub fn diagnostic(&self) -> Option<BoundaryDiagnostic> {
        self.interruption.clone().or_else(|| match &self.result {
            Err(error) => Some(error.boundary_diagnostic()),
            Ok(result) => result.diagnostic(),
        })
    }
}
/// Candidate explanation under unchanged fixedness, bounds, validity and inner systems.
/// Neither `complete` nor the candidate set asserts mathematical infeasibility/minimality.
#[derive(Clone, Debug)]
pub struct ModelingNonlinearExplanation {
    pub run_id: SemanticId,
    pub(in crate::workflow::modeling) runtime: Runtime,
    pub(in crate::workflow::modeling) source_identity: pse_ids::ContentHash,
    pub(in crate::workflow::modeling) nominals: BTreeMap<SemanticId, f64>,
    pub(in crate::workflow::modeling) penalty_tolerance: f64,
    pub attempts: Vec<ModelingElasticAttempt>,
    pub candidate_rows: BTreeSet<SemanticId>,
    pub background_variables: Vec<SemanticId>,
    pub complete: bool,
    pub stop: Option<BoundaryDiagnostic>,
    pub(in crate::workflow::modeling) _owner: Arc<pse_columnar::AllocationLease>,
}
impl ModelingPackage {
    /// Reuse the source elastic transformation, common native solver and joined deadline.
    /// A deletion is a local diagnostic heuristic; every trial and its scope are retained.
    pub async fn explain_nonlinear(
        &self,
        analysis: &ModelingAnalysis,
        policy: ModelingNonlinearPolicy,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingNonlinearExplanation, WorkflowError> {
        if policy.maximum_attempts == 0
            || policy.maximum_attempts > 10000
            || policy.time_limit.is_zero()
            || !policy.penalty_tolerance.is_finite()
            || policy.penalty_tolerance < 0.
            || policy.nominals.is_empty()
            || policy.nominals.len() > analysis.limits.items
            || policy.nominals.values().any(|v| !v.is_finite() || *v <= 0.)
            || analysis.bindings.formulation != Formulation::default()
        {
            return Err(contract(
                "nonlinear explanation requires explicit positive row nominals, finite limits and an original formulation",
            ));
        }
        let started = Instant::now();
        let deadline = started
            .checked_add(policy.time_limit)
            .ok_or_else(|| contract("explanation deadline overflow"))?;
        let (prepared, interruption) = engines::bounded_work(
            "nonlinear explanation",
            deadline,
            cancel,
            |child| async move { self.prepare_diagnostics(analysis, &child).await },
        )
        .await;
        if let Some(error) = interruption {
            return Err(error.into());
        }
        let prepared = prepared?;
        let original = prepared.model.model.compiled();
        if !original.model.elastic.is_empty() {
            return Err(contract("explanation requires an unrelaxed source model"));
        }
        let rows = prepared
            .model
            .case
            .compiled()
            .plan
            .structure()
            .rows()
            .iter()
            .map(|r| (r.id, r.quantity))
            .collect::<BTreeMap<_, _>>();
        let all = rows.keys().copied().collect::<BTreeSet<_>>();
        if all != policy.nominals.keys().copied().collect() {
            return Err(contract(
                "explanation must declare a physical nominal for every outer equation",
            ));
        }
        let bytes = all
            .len()
            .checked_mul(policy.maximum_attempts)
            .and_then(|n| n.checked_mul(128))
            .and_then(|n| n.checked_add(policy.maximum_attempts * 512))
            .ok_or_else(|| contract("explanation result extent"))?;
        let owner = self
            .runtime
            .shared
            .math()
            .reserve("modeling:nonlinear-explanation", bytes)?;
        let background_variables = prepared
            .model
            .case
            .compiled()
            .plan
            .structure()
            .variables()
            .iter()
            .filter(|v| v.fixed || v.lower.is_some() || v.upper.is_some())
            .map(|v| v.port.id)
            .collect();
        let mut report = ModelingNonlinearExplanation {
            run_id: pse_authoring::ids::uuid_v7(),
            runtime: self.runtime.clone(),
            source_identity: prepared.model.case.compiled().plan.structure().key(),
            nominals: policy.nominals.clone(),
            penalty_tolerance: policy.penalty_tolerance,
            attempts: vec![],
            candidate_rows: all.clone(),
            background_variables,
            complete: true,
            stop: None,
            _owner: owner,
        };
        let mut omitted = BTreeSet::new();
        let candidates = std::iter::once(None).chain(all.iter().copied().map(Some));
        for candidate in candidates {
            if report.attempts.len() >= policy.maximum_attempts
                || Instant::now() >= deadline
                || cancel.token().is_cancelled()
            {
                report.complete = false;
                let (class, rule) = if cancel.token().is_cancelled() {
                    (BoundaryClass::Cancelled, "modeling.nonlinear.cancelled")
                } else if Instant::now() >= deadline {
                    (
                        BoundaryClass::ResourceLimit,
                        "modeling.nonlinear.time_limit",
                    )
                } else {
                    (
                        BoundaryClass::ResourceLimit,
                        "modeling.nonlinear.attempt_limit",
                    )
                };
                report.stop = Some(BoundaryDiagnostic::new(
                    class,
                    "nonlinear-explanation",
                    [analysis.root, analysis.instance],
                    rule,
                ));
                break;
            }
            let mut trial_omitted = omitted.clone();
            if let Some(id) = candidate {
                trial_omitted.insert(id);
            }
            let mut trial = analysis.clone();
            trial.order = pse_kernels::DerivativeOrder::Second;
            trial.bindings.formulation = Formulation {
                omitted: trial_omitted.clone(),
                elastic: rows
                    .iter()
                    .filter(|(id, _)| !trial_omitted.contains(id))
                    .map(|(id, q)| {
                        (
                            *id,
                            Value::Number {
                                quantity: *q,
                                bits: policy.nominals[id].to_bits(),
                            },
                        )
                    })
                    .collect(),
            };
            trial.solver.intent = if trial.bindings.formulation.elastic.is_empty() {
                SolveIntent::FeasiblePoint
            } else {
                SolveIntent::Optimize
            };
            trial.solver.controls.time_limit = trial
                .solver
                .controls
                .time_limit
                .min(deadline.saturating_duration_since(Instant::now()));
            let seed = prepared.model.values.scalars.clone();
            let (result, interruption) = engines::bounded_work(
                "nonlinear explanation",
                deadline,
                cancel,
                |child| async move {
                    let prepared = self
                        .prepare_analysis_attempt(&trial, seed, BTreeMap::new(), &child)
                        .await?;
                    self.solve_case(prepared, trial.compiler, &child).await
                },
            )
            .await;
            let (observation, penalty) = result
                .as_ref()
                .map_or((ElasticObservation::Inconclusive, None), |result| {
                    classify(result, &policy)
                });
            let observation = if interruption.is_some() {
                ElasticObservation::Inconclusive
            } else {
                observation
            };
            report.attempts.push(ModelingElasticAttempt {
                omitted: trial_omitted.clone(),
                observation,
                penalty,
                result: result.map_err(Arc::new),
                interruption,
            });
            match observation {
                ElasticObservation::LocalObstruction => {
                    omitted = trial_omitted;
                    report.candidate_rows = all.difference(&omitted).copied().collect();
                }
                ElasticObservation::FeasibleWitness if candidate.is_none() => {
                    report.candidate_rows.clear();
                    break;
                }
                ElasticObservation::Inconclusive => {
                    report.complete = false;
                    if candidate.is_none() {
                        report.candidate_rows.clear();
                        report.stop = Some(
                            report
                                .attempts
                                .last()
                                .and_then(ModelingElasticAttempt::diagnostic)
                                .unwrap_or_else(|| {
                                    BoundaryDiagnostic::new(
                                        BoundaryClass::Inconclusive,
                                        "nonlinear-explanation",
                                        [analysis.root, analysis.instance],
                                        "modeling.nonlinear.initial_inconclusive",
                                    )
                                }),
                        );
                        break;
                    }
                }
                _ => {}
            }
        }
        Ok(report)
    }
}
fn classify(
    result: &ModelingResult,
    policy: &ModelingNonlinearPolicy,
) -> (ElasticObservation, Option<f64>) {
    let model = result.prepared.model.model.compiled();
    let penalty = model.model.elastic.iter().try_fold(0., |total, (id, row)| {
        let sum = row.slacks.iter().try_fold(0., |s, id| {
            result
                .values
                .scalars
                .get(id)
                .filter(|v| v.is_finite())
                .map(|v| s + v.max(0.))
        })?;
        Some(total + sum / policy.nominals[id])
    });
    // The shared candidate-use decision: a limited or failed stop is never a witness.
    let feasible = result.outcome.candidate_use().permits_use();
    let stationary = feasible
        && matches!(
            &result.outcome,
            Outcome::Native(r) if matches!(
                r.qualification,
                Qualification::Stationary
                    | Qualification::OptimalWithinTolerance
                    | Qualification::GapQualified
            )
        );
    let original_ok = model.model.elastic.keys().all(|id| {
        result.checks.iter().any(|c| {
            c.target_id == *id
                && c.kind == pse_model::generated::enums::ModelingCheckKind::OriginalEquation
                && c.satisfied
        })
    });
    let observation =
        if feasible && original_ok && penalty.is_some_and(|v| v <= policy.penalty_tolerance) {
            ElasticObservation::FeasibleWitness
        } else if feasible
            && stationary
            && penalty.is_some_and(|v| v.is_finite() && v > policy.penalty_tolerance)
        {
            ElasticObservation::LocalObstruction
        } else {
            ElasticObservation::Inconclusive
        };
    (observation, penalty)
}

#[cfg(all(test, feature = "solver-ipopt"))]
mod tests {
    use super::*;
    use pse_backend_native::solve::{Backend, SolverSelection};
    #[tokio::test]
    async fn kernel_nonlinear_explanation_keeps_local_evidence_and_original_specification() {
        let rt = super::super::super::super::tests::runtime();
        let physical = super::super::super::super::tests::physical();
        let names = BTreeMap::from([(
            "Scalar".into(),
            physical.quantities.neutral_dimensionless().unwrap(),
        )]);
        let source = "package p { def Root { var x:Scalar; eq lo:x*x>=4; eq hi:x*x<=1; eq spare:x*x<=100; annotation start x(1.5); } }";
        let rows = pse_authoring::language::parse(
            source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = rt.modeling_package(rows, physical, names).unwrap();
        let compiler = super::super::super::super::tests::compiler_profile();
        let cancel = crate::CancelSource::new();
        let mut solver = super::super::super::super::tests::profile();
        solver.selection = SolverSelection::Explicit(Backend::Ipopt);
        let analysis = ModelingAnalysis {
            root,
            instance: root,
            bindings: Bindings::default(),
            limits: Limits::default(),
            case: Default::default(),
            order: pse_kernels::DerivativeOrder::Second,
            compiler,
            solver,
            numerical: Default::default(),
        };
        let before = package
            .prepare_diagnostics(&analysis, &cancel)
            .await
            .unwrap();
        let original_key = before.model.case.compiled().plan.structure().key();
        let rows = before
            .model
            .case
            .compiled()
            .plan
            .structure()
            .rows()
            .iter()
            .map(|r| (r.id, 1.))
            .collect();
        let policy = ModelingNonlinearPolicy {
            nominals: rows,
            penalty_tolerance: 1e-6,
            maximum_attempts: 4,
            time_limit: Duration::from_secs(30),
        };
        let report = package
            .explain_nonlinear(&analysis, policy.clone(), &cancel)
            .await
            .unwrap();
        let summarize = || {
            report
                .attempts
                .iter()
                .map(|a| {
                    format!(
                        "{:?} penalty={:?} {}",
                        a.observation,
                        a.penalty,
                        match &a.result {
                            Err(e) => e.to_string(),
                            Ok(r) => format!("{:?}", r.outcome),
                        }
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        assert!(report.complete, "{}", summarize());
        assert_eq!(report.attempts.len(), 4);
        assert_eq!(
            report.attempts[0].observation,
            ElasticObservation::LocalObstruction,
            "{}",
            summarize()
        );
        assert_eq!(report.candidate_rows.len(), 2, "{}", summarize());
        let spare = before
            .model
            .model
            .compiled()
            .model
            .equations
            .iter()
            .find(|r| r.lineage.path.ends_with(".spare"))
            .unwrap()
            .id;
        assert!(!report.candidate_rows.contains(&spare));
        assert_eq!(
            report
                .attempts
                .iter()
                .filter(|a| a.observation == ElasticObservation::FeasibleWitness)
                .count(),
            2
        );
        let after = package
            .prepare_diagnostics(&analysis, &cancel)
            .await
            .unwrap();
        assert_eq!(
            original_key,
            after.model.case.compiled().plan.structure().key()
        );
        assert_eq!(before.model.values, after.model.values);
        let mut exhausted = analysis.clone();
        exhausted.solver.controls.iterations = 1;
        let failed = package
            .explain_nonlinear(&exhausted, policy.clone(), &cancel)
            .await
            .unwrap();
        assert!(!failed.complete);
        assert!(failed.candidate_rows.is_empty());
        assert_eq!(
            failed.attempts[0].observation,
            ElasticObservation::Inconclusive
        );
        assert!(
            matches!(&failed.attempts[0].result,Ok(r) if matches!(r.outcome,Outcome::Native(_)))
        );
        let mut limited = policy;
        limited.maximum_attempts = 1;
        let partial = package
            .explain_nonlinear(&analysis, limited, &cancel)
            .await
            .unwrap();
        assert!(!partial.complete);
        assert_eq!(
            partial.stop.as_ref().unwrap().rule,
            "modeling.nonlinear.attempt_limit"
        );
        assert_eq!(
            partial.stop.as_ref().unwrap().class,
            BoundaryClass::ResourceLimit
        );
        assert_eq!(partial.attempts.len(), 1);
    }
}
