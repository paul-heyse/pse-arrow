// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared-parameter experiment compilation. Trial values never enter Salsa queries.
mod covariance;
mod documents;
pub use documents::{FitPreparationDocument, FitProfileDocument};
mod modeling;
mod oracle;
mod preparation;
mod profile;
pub use covariance::{Covariance, FitWithheld, Interval, IntervalBound};
pub use modeling::FitDeclarations;
pub use profile::{ProfileChain, ProfileFailure, ProfilePoint, ProfileWorkerFailure};
#[cfg(test)]
pub(in crate::workflow) mod regression;
mod results;
mod sparse;
/// Exact fitting declaration from the schema registry.
pub type FitDeclaration = pse_relations::generated::authored::fit_cases::Row;
use super::integrated::{Binding, IntegratedExperiment};
use super::{WorkflowError, contract, math};
use crate::math::{ExecutableCase, solves::SolverProfile};
use preparation::PreparedExperiments;
use pse_backend_native::{
    self as native, OracleContract, Variable,
    solve::{HessianMode, SolveIntent},
};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_kernels::{DerivativeOrder, Port};
use pse_math::{
    binding::CaseValues,
    index::{GlobalCol, GlobalRow, OriginalCol, OriginalRow, TiVec},
    normalization::Normalization,
    numerics::{SourcedRequirement, TargetSpec},
};
/// Registry vocabulary of a fit's derivative source (ADR-0110 item 3): the response
/// Jacobian from forward sensitivities, or the objective gradient alone from adjoint
/// sensitivities of the transient experiments.
pub use pse_model::generated::enums::FitDerivatives;
use pse_model::generated::identities::{FitId, InstanceId};
use pse_model::{
    SemanticFrame, scalar,
    scalars::{Fraction, PositiveCount},
};
use pse_model::{
    generated::enums::{NumericalCoordinates, NumericalSource, NumericalTarget},
    numerics::{NumericalRequirement, ResolvedNumericalPolicy},
};
use pse_quantity::UnitId;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
/// Controls for the ordinary NLP and explicit dynamic experiment profiles.
#[derive(Clone, Debug)]
pub struct FitProfile {
    /// Ordinary native NLP controls and original-space quality tolerances.
    pub solver: SolverProfile,
    /// Smooth integration controls keyed by the experiment's instance. An experiment's
    /// modes, events and scheduled inputs are those its authored case declares (ADR-0119).
    pub simulations: BTreeMap<InstanceId, super::SimulationProfile>,
    /// Relative local response singular-value cutoff; not a confidence level.
    pub rank_tolerance: f64,
    /// Separate cap for sparse derivative contributions and optional dense rank cells.
    pub max_cells: usize,
    /// The derivative source of the NLP: the response Jacobian, or the objective gradient
    /// alone (with the limited-memory Hessian), whose final assessment reruns the forward
    /// sensitivities once for rank (PS-12).
    pub derivatives: FitDerivatives,
    /// The confidence intervals to derive from the estimate beyond its covariance (Plan 22
    /// S3); absent derives the covariance alone.
    pub uncertainty: Option<FitUncertainty>,
}
/// What a fit derives from its estimate beyond the covariance, which every fit with free
/// parameters derives (ADR-0118 item 8; Plan 22 S3).
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FitUncertainty {
    /// The confidence level of the intervals, below one.
    pub level: Fraction,
    /// Also derive profile-likelihood intervals under these controls; absent derives Wald
    /// intervals alone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<ProfileControls>,
    /// Also propagate the covariance to the included observations' predictions,
    /// `Σ_y = R·Σ_θ·Rᵀ` over the fit's response derivatives (Plan 22 S4); their count
    /// squared is bounded by the profile's `max_cells`.
    #[serde(default)]
    pub predictions: bool,
}
/// Controls of the profile-likelihood pin chains (ADR-0118 item 8).
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProfileControls {
    /// Pinned fits at most per chain, refinements and failed fits included.
    #[serde(default = "ProfileControls::default_points")]
    pub points: PositiveCount,
    /// Relative tolerance on the signed-root statistic at a threshold end.
    #[serde(default = "ProfileControls::default_tolerance")]
    pub tolerance: Fraction,
    /// Chains solved at once, each pinned fit on the fit's own threads; the fit's job admits
    /// the cores of all of them. Absent solves as many at once as one job's cores admit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workers: Option<PositiveCount>,
}
impl ProfileControls {
    const fn default_points() -> PositiveCount {
        scalar!(PositiveCount(40))
    }
    const fn default_tolerance() -> Fraction {
        scalar!(Fraction(1e-3))
    }
}
impl Default for ProfileControls {
    fn default() -> Self {
        Self {
            points: Self::default_points(),
            tolerance: Self::default_tolerance(),
            workers: None,
        }
    }
}
impl FitUncertainty {
    /// Intervals at `level`, Wald alone.
    pub fn wald(level: Fraction) -> Self {
        Self {
            level,
            profile: None,
            predictions: false,
        }
    }
    /// A confidence level below one: at one every interval is unbounded.
    fn admit(&self) -> Result<(), WorkflowError> {
        if self.level.into_inner() >= 1.0 {
            return Err(contract("a fit's confidence level lies below one"));
        }
        Ok(())
    }
}
impl FitProblem {
    /// The profile chains solved at once: the requested workers, at most one per chain and
    /// as many as `cores` admit at the fit's threads each; one without a profile.
    pub(crate) fn profile_workers(&self, cores: usize) -> usize {
        let Some(controls) = self
            .profile
            .uncertainty
            .as_ref()
            .and_then(|u| u.profile.as_ref())
        else {
            return 1;
        };
        let chains = 2 * self.parameter_columns.iter().flatten().count();
        let admitted = cores / self.profile.solver.controls.threads.max(1);
        controls
            .workers
            .map_or(usize::MAX, PositiveCount::into_inner)
            .min(chains)
            .min(admitted)
            .max(1)
    }
}
#[derive(Clone, Debug)]
struct Measurement {
    id: SemanticId,
    experiment: usize,
    row: usize,
    time: Option<f64>,
    #[cfg_attr(
        not(feature = "solver-diffsol"),
        expect(
            dead_code,
            reason = "prepared sample binding is consumed by the linked dynamic fit oracle"
        )
    )]
    sample_index: Option<usize>,
    included: bool,
    value: Option<f64>,
    sigma: Option<f64>,
    importance: f64,
    port: Port,
}
#[derive(Clone, Debug)]
struct Steady {
    variables: Vec<pse_math::binding::Variable>,
    case: Arc<ExecutableCase>,
    values: CaseValues,
    /// Each derivative column of the experiment case: the source coordinate it binds and
    /// the fit column that feeds it. Local states come first.
    coordinates: TiVec<GlobalCol, (SemanticId, OriginalCol)>,
    /// Each bounded row of the experiment case and the fit row it becomes.
    constraints: Vec<(GlobalRow, OriginalRow)>,
    local_states: usize,
    providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
}
#[derive(Clone, Debug)]
enum Experiment {
    Steady(Steady),
    Transient(Box<IntegratedExperiment>),
}
/// Immutable fitting product; mutable evaluators and native sessions are attempt-owned.
#[derive(Debug)]
pub(crate) struct FitProblem {
    snapshot: native::execution::Snapshot,
    structure: native::routing::Structure,
    structural_assessment: Option<native::structural::Assessment>,
    pub(crate) runtime: super::Runtime,
    pub(crate) quantities: Arc<pse_quantity::QuantityRegistry>,
    pub(crate) source_identity: ContentHash,
    pub(crate) declaration: FitDeclaration,
    /// The fit and the model and case its experiments share (`pse_model::lineage`).
    pub(crate) lineage: pse_model::lineage::Fitted,
    pub(crate) profile: FitProfile,
    pub(crate) key: ContentHash,
    pub(crate) profile_key: pse_ids::roles::ProfileHash,
    pub(crate) numerics: Arc<ResolvedNumericalPolicy>,
    pub(crate) normalization: Normalization,
    pub(crate) tolerances: native::quality::Tolerances,
    /// Stopping budgets resolved from the fit's numerical policy (F20).
    pub(crate) accuracy: native::solve::ResolvedAccuracy,
    pub(crate) bytes: usize,
    contract: OracleContract,
    layout: Arc<sparse::Layout>,
    bounds: Vec<(f64, f64)>,
    initial: Vec<f64>,
    parameter_ports: Vec<Port>,
    /// The fit column of each shared parameter; `None` when it is fixed.
    parameter_columns: Vec<Option<OriginalCol>>,
    experiments: Vec<Experiment>,
    measurements: Vec<Measurement>,
    _owner: Arc<pse_columnar::AllocationLease>,
}
/// Fitting mathematics with an admitted execution route.
#[derive(Clone, Debug)]
pub struct PreparedFit {
    pub(crate) source: super::ModelingPackage,
    assessments: Vec<modeling::Assessment>,
    pub(crate) problem: Arc<FitProblem>,
    route: native::routing::Route,
}
/// Joined native fit plus independently re-evaluated physical predictions.
#[derive(Debug)]
pub struct FitReport {
    /// Final original checks and requested goal permission, composed once after
    /// the authored assessment. Absent while the native report is being built.
    pub(crate) completion: Option<super::numerics::Completed>,
    /// Actual shared numerical-driver events, owned independently of result copies.
    pub strategy: Option<Arc<crate::math::strategy::Trace>>,
    /// Original authored checks evaluated at the final physical candidate.
    pub checks: Vec<super::ModelingCheck>,
    /// Original authored reports, including completed terminal integrals.
    pub reports: Vec<super::ModelingReport>,
    /// Every requested experiment received its original-model checks.
    pub checks_complete: bool,
    /// Typed cause when original-model assessment could not complete.
    pub validation_error: Option<pse_model::diagnostic::BoundaryDiagnostic>,
    /// Ordinary native NLP report, absent for all-fixed evaluation.
    pub solve: Option<native::solve::SolveReport>,
    /// The Hessian source of the native solve (PS-07): the exact Lagrangian, the
    /// Gauss–Newton Gram with constraint curvature, or the library's quasi-Newton
    /// approximation.
    pub hessian: HessianMode,
    /// The gradient source of the native solve (PS-07): the response Jacobian, or adjoint
    /// gradients of the transient experiments.
    pub derivatives: FitDerivatives,
    /// Independent original constraint and bound quality, including all-fixed evaluation.
    pub quality: Option<native::quality::Quality>,
    /// Independently evaluated steady physical constraints in admitted order.
    pub constraint_values: Vec<f64>,
    /// Fresh weighted least-squares objective, if evaluated.
    pub objective: Option<f64>,
    /// Complete original coordinate vector, absent without a candidate.
    pub candidate: Option<Vec<f64>>,
    /// Predictions in binding order, including excluded observations when evaluable.
    pub predictions: Vec<Option<f64>>,
    /// Final original experiment trajectories, retained under the fit result's allocation owner.
    /// Source interpretation can inspect these without repeating native integration.
    pub(crate) trajectories: BTreeMap<InstanceId, Arc<native::dynamics::Report>>,
    /// Local response derivatives in observation by free-parameter order.
    pub responses: Option<pse_columnar::Leased<faer::Mat<f64>>>,
    /// Singular values of the weighted, parameter-scaled response Jacobian.
    pub singular_values: Vec<f64>,
    /// Its right singular vectors in coordinates divided by each parameter's declared
    /// scale, one column per direction by decreasing singular value; the columns beyond
    /// `singular_values` span its null space. The first `rank` columns span the locally
    /// identifiable subspace.
    pub directions: Option<faer::Mat<f64>>,
    /// Local numerical column rank, when independently qualified.
    pub rank: Option<usize>,
    /// Why prediction or local sensitivity qualification is unavailable.
    pub diagnostic: Option<FitDiagnostic>,
    /// The covariance of the free parameters, certified or withheld; absent when no
    /// parameter is free (ADR-0118 item 8).
    pub covariance: Option<Covariance>,
    /// The Wald intervals, when the profile requested intervals.
    pub wald: Option<Result<Vec<Interval>, FitWithheld>>,
    /// The profile-likelihood chains, two per free parameter, when requested.
    pub profiles: Option<Result<Vec<ProfileChain>, FitWithheld>>,
}
/// Stable rule for an unavailable fresh prediction or local sensitivity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FitRule {
    /// The fresh final original evaluation failed.
    FinalEvaluation,
    /// The fresh weighted least-squares objective overflowed.
    ObjectiveOverflow,
    /// The local response rank diagnostic failed.
    ResponseRank,
}
impl FitRule {
    /// Stable published rule code; never prose.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FinalEvaluation => "fit.final_evaluation",
            Self::ObjectiveOverflow => "fit.objective_overflow",
            Self::ResponseRank => "fit.response_rank",
        }
    }
}
/// A fit diagnostic keeps a stable rule and its original typed cause.
#[derive(Clone, Debug)]
pub struct FitDiagnostic {
    /// Stable rule code.
    pub rule: FitRule,
    /// Original typed cause; its class is derived, never guessed.
    pub cause: Arc<native::ProblemError>,
}
impl FitDiagnostic {
    pub(crate) fn new(rule: FitRule, cause: native::ProblemError) -> Self {
        Self {
            rule,
            cause: Arc::new(cause),
        }
    }
}
impl std::fmt::Display for FitDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.rule.as_str(), self.cause)
    }
}
impl FitReport {
    /// Known result buffers; the solver's finite report allowance owns diagnostics.
    pub(crate) fn numeric_bytes(&self) -> usize {
        size_of::<Self>()
            + self.completion.as_ref().map_or(0, |completion| {
                completion.decision.qualifiers.capacity() * size_of::<pse_model::generated::enums::CandidateQualifier>()
                    + completion.decision.refusals.capacity() * size_of::<pse_model::generated::enums::CandidateRefusal>()
                    + completion.accuracy.capacity() * size_of::<pse_math::engineering_accuracy::GoalResult>()
                    + completion.accuracy.iter().map(|goal| {
                        pse_model::HeapUsage::heap_bytes(&goal.goal)
                            + goal.evidence.as_ref().map_or(0, |evidence| evidence.limitation.capacity())
                            + goal.work_demand.as_ref().map_or(0, |demand| {
                                demand.rows.capacity() * size_of::<pse_math::engineering_accuracy::ResidualRowDemand>()
                            })
                    }).sum::<usize>()
            })
            + self.checks.capacity() * size_of::<super::ModelingCheck>()
            + self.reports.iter().map(pse_model::HeapUsage::owned_bytes).sum::<usize>()
            + self.validation_error.as_ref().map_or(0, pse_model::HeapUsage::owned_bytes)
            + self.predictions.capacity() * size_of::<Option<f64>>()
            + self.trajectories.values().map(|report| report.numeric_bytes() + 128).sum::<usize>()
            + (self.constraint_values.capacity()
                + self.singular_values.capacity()
                + self.directions.as_ref().map_or(0, |v| v.nrows() * v.ncols())
                + self.candidate.as_ref().map_or(0, Vec::capacity)
                + self.covariance.as_ref().map_or(0, Covariance::cells))
                * size_of::<f64>()
            + self.wald.as_ref().map_or(0, |w| {
                w.as_ref().map_or(0, |w| w.len() * size_of::<Interval>())
            })
            + self.profiles.as_ref().map_or(0, |p| {
                p.as_ref()
                    .map_or(0, |p| p.iter().map(ProfileChain::bytes).sum())
            })
            // The optional dense response carries its own reservation.
            + self
                .diagnostic
                .as_ref()
                .map_or(0, |d| d.cause.retained_bytes())
    }
    /// Candidate use of the final estimate: the native decision, then the fresh
    /// independent original-model quality of the final evaluation.
    pub(crate) fn candidate_use(
        &self,
        policy: &pse_model::numerics::NumericalPolicy,
    ) -> super::numerics::CandidateDecision {
        use super::numerics::{constant_use, native_use, refused};
        use pse_model::generated::enums::CandidateRefusal;
        if self.candidate.is_none() {
            return refused(CandidateRefusal::NoCandidate);
        }
        let fresh = self
            .quality
            .as_ref()
            .map_or(refused(CandidateRefusal::Infeasible), constant_use);
        match self.solve.as_ref().map(|report| native_use(report, policy)) {
            Some(mut native) => {
                if !fresh.permits_use() {
                    native.refuse(CandidateRefusal::Infeasible);
                }
                native
            }
            None => fresh,
        }
    }
    /// A stationary feasible estimate with locally identifiable free parameters.
    /// This establishes neither global optimality nor a statistical confidence interval.
    pub fn estimate_qualified(&self) -> bool {
        self.checks_complete
            && self.checks.iter().all(|c| c.satisfied)
            && self.validation_error.is_none()
            && self.solve.as_ref().is_some_and(|s| {
                matches!(
                    s.qualification,
                    native::solve::Qualification::Stationary
                        | native::solve::Qualification::OptimalWithinTolerance
                )
            })
            && self
                .quality
                .as_ref()
                .is_some_and(native::quality::Quality::feasible)
            && self
                .responses
                .as_ref()
                .is_some_and(|j| j.ncols() > 0 && self.rank == Some(j.ncols()))
    }
    /// Spectral condition estimate of the weighted, parameter-scaled response at the candidate.
    pub fn response_condition(&self) -> Option<f64> {
        let largest = self.singular_values.first()?;
        let smallest = self.singular_values.last()?;
        let ratio = largest / smallest;
        (*smallest > 0.0 && ratio.is_finite()).then_some(ratio)
    }
}
impl PreparedFit {
    /// Requested composition constraints retained independently from native fit settings.
    pub fn composition_request(&self) -> &pse_model::strategy::CompositionRequest {
        &self.problem.profile.solver.composition
    }

    pub(crate) fn callable_source(&self) -> crate::math::strategy::target::Source<'_> {
        let problem = &self.problem;
        crate::math::strategy::target::Source {
            original: problem.source_identity,
            preparation: problem.key,
            profile: problem.profile_key.as_id(),
            backend: match self.route {
                native::routing::Route::Native(backend) => Some(backend),
                native::routing::Route::Constant => None,
            },
            solver: Some(&problem.profile.solver),
            controls: &problem.profile.solver.controls,
            request: &problem.profile.solver.composition,
            start: pse_model::strategy::StartOrigin::Specification,
            start_identity: Some(crate::math::strategy::target::point_identity(
                problem.source_identity,
                &problem.initial,
            )),
        }
    }
    /// Actual callable profile identity, including scientific fit settings.
    pub fn strategy_profile(&self) -> ContentHash {
        self.problem.profile_key.as_id()
    }
    /// Frozen callable execution declaration; inspection does not construct derivatives.
    pub fn numerical_strategy(&self) -> pse_model::strategy::NumericalStrategy {
        crate::math::strategy::target::declaration(&self.callable_source())
    }

    /// Complete immutable source/execution identity.
    pub fn identity(&self) -> ContentHash {
        self.problem.key
    }
    /// Admitted execution route, including direct constant evaluation.
    pub fn route(&self) -> native::routing::Route {
        self.route
    }
    /// Original generated fit declaration.
    pub fn declaration(&self) -> &FitDeclaration {
        &self.problem.declaration
    }
}
/// The fit coordinate of an experiment's source coordinate (a variable, row or
/// requirement of its case): distinct per experiment, so experiments sharing a case do
/// not share its coordinates.
fn alias(experiment: InstanceId, source: SemanticId) -> SemanticId {
    let mut h = FramedHasher::new(pse_ids::Frame::FitCoordinateV1);
    h.id(&experiment.as_id()).id(&source);
    h.finish_id()
}

#[cfg(test)]
fn measured_rows(
    cells: &[(SemanticId, &str, Option<f64>, Option<f64>)],
) -> Vec<pse_authoring::language::Declaration> {
    let mut text = String::from(
        "package measurements { entity kind origin provenance {attribute title:Text;} entity origin experiment {title=\"analytic measurement fixture\"} enum role {measured facets(measured)} ",
    );
    for (i, (id, ty, value, sigma)) in cells.iter().enumerate() {
        let literal = |value: Option<f64>| {
            value.map_or_else(
                || "missing".to_owned(),
                |value| {
                    if *ty == "Time" {
                        format!("{value}{{s}}")
                    } else {
                        format!("{value}")
                    }
                },
            )
        };
        text.push_str(&format!("entity kind sample{i} {{attribute value:{ty}?; attribute sigma:{ty}?;}} @id(\"{id}\") entity sample{i} observation{i} provenance(experiment,role.measured) {{value={},sigma={}}} ",literal(*value),literal(*sigma)));
    }
    text.push('}');
    pse_authoring::language::parse(
        &text,
        SemanticId::from_bytes([222; 16]),
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap()
}

impl PreparedFit {
    /// The original scientific permission used by the numerical driver and run completion.
    pub(crate) fn assess_completion(&self, report: &FitReport) -> super::numerics::Completed {
        let policy = &self.problem.numerics.policy;
        let accuracy = policy
            .goals
            .iter()
            .cloned()
            .map(|goal| {
                pse_math::engineering_accuracy::GoalResult::unavailable(
                    goal,
                    pse_model::generated::enums::AccuracyUnavailableReason::Unsupported,
                )
            })
            .collect::<Vec<_>>();
        let mut evidence = super::numerics::CompletionEvidence::point(
            &report.checks,
            report.checks_complete && report.validation_error.is_none(),
            self.required_closure_checks(),
        );
        evidence.accuracy = &accuracy;
        super::numerics::complete(report.candidate_use(policy), evidence, policy)
            .with_context(&self.problem.numerics)
    }
    pub(crate) fn required_closure_checks(&self) -> usize {
        self.assessments
            .iter()
            .map(|assessment| match assessment {
                modeling::Assessment::Steady { model, .. } => {
                    model.compiled().model.required_closure_checks()
                }
                modeling::Assessment::Transient(simulation) => simulation.required_closure_checks(),
            })
            .sum()
    }
}
