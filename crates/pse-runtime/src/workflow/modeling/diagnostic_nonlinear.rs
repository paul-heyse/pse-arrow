// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded whole-model infeasibility explanations on POUNCE's ℓ1 exact penalty (ADR-0109
//! item 2a). Local obstruction never certifies infeasibility.
use pse_model::generated::identities::RunId;
use super::*;
use crate::math::solves::Outcome;
use pse_backend_native::solve::{Backend, SolveIntent, SolverSelection};
use pse_modeling::specialize::Formulation;
use std::{
    collections::BTreeSet,
    time::{Duration, Instant},
};

/// Explicit physical row nominals weight the dimensionless ℓ1 violation that classifies each
/// attempt.
#[derive(Clone, Debug)]
pub struct ModelingNonlinearPolicy {
    /// Positive physical nominal of every outer equation, by equation id.
    pub nominals: BTreeMap<SemanticId, f64>,
    /// Nominal-weighted ℓ1 violation at or below which an attempt is a feasible witness.
    pub penalty_tolerance: f64,
    /// Attempt budget of the deletion filter.
    pub maximum_attempts: usize,
    /// Joined deadline of every attempt.
    pub time_limit: Duration,
}
/// Evidence from one bounded local optimization, not a global verdict.
pub use pse_model::generated::enums::ModelingElasticObservation as ElasticObservation;
/// One attempt of the deletion filter: evidence from one bounded local solve.
#[derive(Clone, Debug)]
pub struct ModelingElasticAttempt {
    /// Outer equations removed for this attempt.
    pub omitted: BTreeSet<SemanticId>,
    /// Local classification of the attempt.
    pub observation: ElasticObservation,
    /// Nominal-weighted ℓ1 violation of the retained equations at the candidate.
    pub penalty: Option<f64>,
    /// The attempt's solve, or why it could not run.
    pub result: Result<ModelingResult, Arc<WorkflowError>>,
    /// Deadline or cancellation that stopped the attempt.
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
    /// Identity of this explanation run.
    pub run_id: RunId,
    pub(in crate::workflow::modeling) runtime: Runtime,
    pub(in crate::workflow::modeling) source_identity: pse_ids::ContentHash,
    pub(in crate::workflow::modeling) nominals: BTreeMap<SemanticId, f64>,
    pub(in crate::workflow::modeling) penalty_tolerance: f64,
    /// Every attempt in order.
    pub attempts: Vec<ModelingElasticAttempt>,
    /// Outer equations the deletion filter retained as the candidate explanation.
    pub candidate_rows: BTreeSet<SemanticId>,
    /// Fixed or bounded variables held unchanged by every attempt.
    pub background_variables: Vec<SemanticId>,
    /// The filter visited every equation within its budgets.
    pub complete: bool,
    /// The budget or interruption that ended an incomplete filter.
    pub stop: Option<BoundaryDiagnostic>,
    pub(in crate::workflow::modeling) _owner: Arc<pse_columnar::AllocationLease>,
}
impl ModelingPackage {
    /// A deletion filter over the outer equations. Every attempt solves the original rows
    /// that remain with POUNCE's explicit ℓ1 exact-penalty method, which returns either a
    /// feasible point or a labelled least-infeasible point; no elastic reformulation of the
    /// model is built. A deletion is a local diagnostic heuristic; every trial and its scope
    /// are retained.
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
        let (prepared, interruption) = crate::workflow::staged::bounded(
            "nonlinear explanation",
            Some(deadline),
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
        let all = prepared
            .model
            .case
            .compiled()
            .plan
            .structure()
            .rows()
            .iter()
            .map(|r| r.id)
            .collect::<BTreeSet<_>>();
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
            run_id: pse_operations::mint_id(),
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
                    [analysis.root.as_id(), analysis.instance.as_id()],
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
                ..Formulation::default()
            };
            l1_route(&mut trial.solver)?;
            trial.solver.controls.time_limit = trial
                .solver
                .controls
                .time_limit
                .min(deadline.saturating_duration_since(Instant::now()));
            let seed = prepared.model.values.scalars.clone();
            let (result, interruption) = crate::workflow::staged::bounded(
                "nonlinear explanation",
                Some(deadline),
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
                                        [analysis.root.as_id(), analysis.instance.as_id()],
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
/// The explanation's own explicit route: POUNCE's ℓ1 exact penalty minimizes the violation of
/// every remaining row (ADR-0109 items 1 and 2a), with presolve off because its passes assume
/// the rows hold. Without POUNCE linked there is no route.
fn l1_route(solver: &mut crate::math::solves::SolverProfile) -> Result<(), WorkflowError> {
    solver.intent = SolveIntent::FeasiblePoint;
    solver.selection = SolverSelection::Explicit(Backend::Pounce);
    solver.presolve = pse_backend_native::presolve::Policy::Off;
    #[cfg(feature = "solver-pounce")]
    {
        use pse_backend_native::{execution::BackendSettings, pounce};
        solver.backend = BackendSettings::Pounce(pounce::Settings {
            method: pounce::Method::L1ExactPenalty,
            ..pounce::Settings::default()
        });
        Ok(())
    }
    #[cfg(not(feature = "solver-pounce"))]
    {
        Err(
            crate::math::MathRuntimeError::from(pse_backend_native::ProblemError::Unavailable {
                backend: Backend::Pounce,
                alternatives: vec![],
            })
            .into(),
        )
    }
}
/// A global infeasibility conclusion beside the local elastic explanation (ADR-0106 §9):
/// the certifying backend's verdict over the declared box for the exported program, and
/// its infeasible subsystem when one was found. `assurance` is `proven_infeasible` (or an
/// `exact_certificate`) only when infeasibility was proved; it is never a local
/// observation and the local explanation never falls back to it.
#[derive(Clone, Debug)]
pub struct ModelingInfeasibilityCertificate {
    /// The certifying solve's assurance.
    pub assurance: pse_backend_native::solve::Assurance,
    /// Worst export fidelity: a relaxed export keeps an infeasibility proof sound.
    pub fidelity: Option<pse_math::factorable::Fidelity>,
    /// Rows of the infeasible subsystem.
    pub rows: BTreeSet<SemanticId>,
    /// Every member of the infeasible subsystem, including obligations, implicit
    /// residuals, native forms and kept bounds.
    pub members: Vec<pse_backend_native::solve::IisMember>,
    /// The backend reports the subsystem irreducible.
    pub irreducible: bool,
    /// The certifying solve.
    pub result: ModelingResult,
}
impl ModelingInfeasibilityCertificate {
    /// Infeasibility of the exported program was proved over the declared box.
    pub fn proven(&self) -> bool {
        use pse_backend_native::solve::Assurance;
        matches!(
            self.assurance,
            Assurance::ProvenInfeasible | Assurance::ExactCertificate
        )
    }
}
impl ModelingPackage {
    /// Certify infeasibility globally through the certify intent and a certifying backend,
    /// computing an irreducible infeasible subsystem of the true program when infeasible.
    /// The analysis keeps its original formulation; a feasible model yields no proof.
    pub async fn certify_infeasibility(
        &self,
        analysis: &ModelingAnalysis,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingInfeasibilityCertificate, WorkflowError> {
        use pse_backend_native::{
            execution::{BackendSettings, ScipSettings},
            solve::{Backend, SolverSelection},
        };
        if analysis.bindings.formulation != Formulation::default() {
            return Err(contract(
                "global infeasibility certification requires an original formulation",
            ));
        }
        let mut trial = analysis.clone();
        trial.solver.intent = SolveIntent::Certify;
        trial.solver.selection = SolverSelection::Explicit(Backend::Scip);
        trial.solver.backend = BackendSettings::Scip(ScipSettings {
            iis: true,
            ..ScipSettings::default()
        });
        let prepared = self.prepare_analysis(&trial, cancel).await?;
        let result = self.solve_case(prepared, trial.compiler, cancel).await?;
        let Outcome::Native(native) = &result.outcome else {
            return Err(contract(
                "global infeasibility certification needs a native certifying solve",
            ));
        };
        let iis = native.global.as_ref().and_then(|g| g.iis.clone());
        let members = iis.as_ref().map(|i| i.members.clone()).unwrap_or_default();
        Ok(ModelingInfeasibilityCertificate {
            assurance: native.termination.assurance,
            fidelity: native.evidence.global.map(|g| g.fidelity),
            rows: members
                .iter()
                .filter_map(|m| match m {
                    pse_backend_native::solve::IisMember::Row(id) => Some(*id),
                    _ => None,
                })
                .collect(),
            members,
            irreducible: iis.is_some_and(|i| i.irreducible),
            result: result.clone(),
        })
    }
}
fn classify(
    result: &ModelingResult,
    policy: &ModelingNonlinearPolicy,
) -> (ElasticObservation, Option<f64>) {
    let Outcome::Native(report) = &result.outcome else {
        return (ElasticObservation::Inconclusive, None);
    };
    // The ℓ1 violation of the retained original rows at the candidate, each divided by its
    // declared physical nominal.
    let penalty = report
        .candidate
        .as_ref()
        .and(report.quality.as_ref())
        .and_then(|q| {
            q.rows.iter().try_fold(0., |total, v| {
                policy
                    .nominals
                    .get(&v.id)
                    .map(|nominal| total + v.physical / nominal)
            })
        })
        .filter(|v| v.is_finite());
    // The shared candidate-use decision: a limited or failed stop is never a witness.
    let feasible = result.outcome.candidate_use().permits_use();
    let observation = if feasible && penalty.is_some_and(|v| v <= policy.penalty_tolerance) {
        ElasticObservation::FeasibleWitness
    } else if report.least_infeasible.is_some()
        && penalty.is_some_and(|v| v > policy.penalty_tolerance)
    {
        // The exact penalty stopped at a least-infeasible point: local evidence only.
        ElasticObservation::LocalObstruction
    } else {
        ElasticObservation::Inconclusive
    };
    (observation, penalty)
}

#[cfg(all(test, feature = "solver-ipopt", feature = "solver-pounce"))]
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
            instance: pse_modeling::specialize::root_instance(root),
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
        // The first attempt ran the explicit ℓ1 route on the unrelaxed rows and stopped at a
        // labelled least-infeasible point: x² ≥ 4 and x² ≤ 1 leave a least violation of 3.
        let first = &report.attempts[0];
        assert!(
            (first.penalty.unwrap() - 3.).abs() < 1e-5,
            "{}",
            summarize()
        );
        let Ok(result) = &first.result else {
            panic!("{}", summarize())
        };
        let Outcome::Native(native) = &result.outcome else {
            panic!("{}", summarize())
        };
        assert_eq!(native.backend, Backend::Pounce);
        assert_eq!(
            native.options["l1_exact_penalty_barrier"],
            pse_backend_native::solve::OptionValue::Bool(true)
        );
        assert!(native.least_infeasible.is_some());
        assert!(
            result
                .prepared
                .model
                .model
                .compiled()
                .model
                .elastic
                .is_empty()
        );
        assert_eq!(
            first.diagnostic().unwrap().rule,
            "modeling.qualification.rejected"
        );
        assert_eq!(report.candidate_rows.len(), 2, "{}", summarize());
        // The least-infeasible point names the rows it leaves violated (ADR-0109 item 3).
        let violated = first.diagnostic().unwrap().sources;
        assert!(!violated.is_empty(), "{}", summarize());
        assert!(
            violated.iter().all(|id| report.candidate_rows.contains(id)),
            "{}",
            summarize()
        );
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
