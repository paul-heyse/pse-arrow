// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared-parameter experiment compilation. Trial values never enter Salsa queries.
mod modeling;
mod oracle;
mod preparation;
pub use modeling::FitData;
mod results;
mod sparse;
/// Exact fitting declaration from the schema registry.
pub type FitDeclaration = pse_relations::generated::authored::fit_cases::Row;
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
use pse_model::SemanticFrame;
use pse_model::generated::identities::{FitId, InstanceId};
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
    /// Smooth integration controls keyed by the experiment's instance.
    pub simulations: BTreeMap<InstanceId, super::SimulationProfile>,
    /// Authored dynamic modes keyed by the experiment's instance; absence selects one
    /// smooth mode.
    pub modes: BTreeMap<InstanceId, Vec<super::ModelingDynamicMode>>,
    /// Relative local response singular-value cutoff; not a confidence level.
    pub rank_tolerance: f64,
    /// Separate cap for sparse derivative contributions and optional dense rank cells.
    pub max_cells: usize,
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
/// A fit coordinate and an experiment coordinate need not have the same semantic ID.
#[derive(Clone, Debug)]
struct ParameterBinding {
    local: usize,
    parameter: usize,
    conversion: pse_quantity::UnitConvertSpec,
}
#[derive(Clone, Debug)]
struct Transient {
    program: super::dynamics::DynamicProgram,
    profile: super::SimulationProfile,
    parameters: Vec<f64>,
    output_ports: Vec<Port>,
    bindings: Vec<ParameterBinding>,
}
#[derive(Clone, Debug)]
enum Experiment {
    Steady(Steady),
    Transient(Transient),
}
/// Immutable fitting product; mutable evaluators and native sessions are attempt-owned.
#[derive(Debug)]
pub(crate) struct FitProblem {
    pub(crate) runtime: super::Runtime,
    pub(crate) quantities: Arc<pse_quantity::QuantityRegistry>,
    pub(crate) source_identity: ContentHash,
    pub(crate) declaration: FitDeclaration,
    pub(crate) profile: FitProfile,
    pub(crate) key: ContentHash,
    pub(crate) profile_key: ContentHash,
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
    /// Local numerical column rank, when independently qualified.
    pub rank: Option<usize>,
    /// Why prediction or local sensitivity qualification is unavailable.
    pub diagnostic: Option<FitDiagnostic>,
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
            + self.checks.capacity() * size_of::<super::ModelingCheck>()
            + self.reports.iter().map(pse_model::HeapUsage::owned_bytes).sum::<usize>()
            + self.validation_error.as_ref().map_or(0, pse_model::HeapUsage::owned_bytes)
            + self.predictions.capacity() * size_of::<Option<f64>>()
            + self.trajectories.values().map(|report| report.numeric_bytes() + 128).sum::<usize>()
            + (self.constraint_values.capacity()
                + self.singular_values.capacity()
                + self.candidate.as_ref().map_or(0, Vec::capacity))
                * size_of::<f64>()
            // The optional dense response carries its own reservation.
            + self
                .diagnostic
                .as_ref()
                .map_or(0, |d| d.cause.retained_bytes())
    }
    /// Candidate use of the final estimate: the native decision, then the fresh
    /// independent original-model quality of the final evaluation.
    pub(crate) fn candidate_use(&self) -> super::numerics::CandidateDecision {
        use super::numerics::{CandidateReason, constant_use, native_use, refused};
        if self.candidate.is_none() {
            return refused(CandidateReason::NoCandidate);
        }
        let fresh = self
            .quality
            .as_ref()
            .map_or(refused(CandidateReason::Infeasible), constant_use);
        match self.solve.as_ref().map(native_use) {
            // A native seed-only or refused estimate keeps its decision.
            Some(native) if !native.permits_use() => native,
            Some(_) | None => fresh,
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
