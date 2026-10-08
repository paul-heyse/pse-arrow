// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The backend-execution seam (S18). One adapter per `Backend` owns its pse-owned settings
//! type, one capability record, admission of its settings, its native session lifecycle
//! on the owning worker, its warm-start payload and its typed evidence. A static table maps
//! every registry `Backend` to its adapter; there is no string lookup. Routing, the
//! published `runtime.solver_capabilities` inventory and the representation runners
//! consume only this seam, so adding a backend is one adapter, one registry value and its
//! capability record, with no workflow edits.
use crate::{
    CoefficientProblem, ConicProblem, NleOracle, NlpOracle, OracleContract, ProblemError,
    quality::Tolerances,
    routing::Requirements,
    solve::{
        Backend, Compatibility, Controls, DerivativeCapability, Execution, ProblemClass,
        ResolvedAccuracy, SolveIntent, SolveReport, WarmCapability, WarmPayload, WarmStart,
    },
};
use pse_ids::{ContentHash, SemanticId};
use pse_math::{
    binding::ObjectiveSense, convexity::QuadraticEvidence, factorable::FactorableProgram,
    normalization::Normalization, presolve::GuardSign,
};
use pse_model::generated::enums::{ModelingStructuralRequirement, NativeConstraintForm};
use std::{any::Any, collections::BTreeMap};

/// The runtime's existing compute allowance on the current native owner thread.
/// Waiting for a real native exclusion guard releases compute capacity; acquiring
/// that guard does not grant capacity, so it must be reacquired before native work.
pub trait ComputeAdmission: std::fmt::Debug + Send {
    /// Relinquish active compute capacity before native exclusion admission.
    fn pause(&mut self);
    /// Reacquire the same allowance under the original attempt/task clock and stop.
    /// # Errors
    /// Cancellation, deadline or a closed runtime admission.
    fn resume(&mut self, execution: &Execution) -> Result<(), ProblemError>;
}
thread_local! {
    static COMPUTE: std::cell::RefCell<Option<Box<dyn ComputeAdmission>>> = const { std::cell::RefCell::new(None) };
}
struct ComputeScope(Option<Box<dyn ComputeAdmission>>);
impl Drop for ComputeScope {
    fn drop(&mut self) {
        COMPUTE.with(|owner| *owner.borrow_mut() = self.0.take());
    }
}
/// Install the actual compute owner for a synchronous native operation. The owner
/// and its permit survive cleanup and unwind; neither native guards nor state cross
/// threads. Nested scopes restore their enclosing owner on exit.
pub fn compute_scoped<T>(owner: Box<dyn ComputeAdmission>, work: impl FnOnce() -> T) -> T {
    let _scope = ComputeScope(COMPUTE.with(|held| held.borrow_mut().replace(owner)));
    work()
}
#[cfg(any(feature = "highs", feature = "uno", feature = "petsc"))]
pub(crate) fn pause_compute() {
    COMPUTE.with(|held| {
        if let Some(owner) = held.borrow_mut().as_mut() {
            owner.pause();
        }
    });
}
#[cfg(any(feature = "highs", feature = "uno", feature = "petsc"))]
pub(crate) fn resume_compute(execution: &Execution) -> Result<(), ProblemError> {
    execution.check()?;
    COMPUTE.with(|held| {
        held.borrow_mut()
            .as_mut()
            .map_or(Ok(()), |owner| owner.resume(execution))
    })
}

/// Whether a waiting Uno owner requires idle retained HiGHS state to be relinquished.
pub fn scheduler_waiting() -> bool {
    #[cfg(feature = "uno")]
    {
        crate::highs_lifecycle::waiting()
    }
    #[cfg(not(feature = "uno"))]
    {
        false
    }
}

mod context;
pub use context::{BuildObservation, Snapshot};

mod clarabel;
mod dynamics;
pub(crate) mod factorable;
mod highs;
mod ipopt;
mod kinsol;
mod petsc;
mod pounce;
mod pounce_convex;
mod runner;
mod scip;
pub mod sos;
mod uno;
pub use factorable::{
    Factorable, Refusal, RelaxedOracle, Resolve, ResolveSensitivity, admit_program, factorable,
    factorable_resolve_allocation_bound, factorable_resolve_order,
};
pub use runner::{
    Analysis, Coefficients, Evaluation, Nlp, OriginalModel, Recognized, Roots, Step, coefficients,
    coefficients_batch, cone, nlp, recognized, roots,
};
pub use scip::Settings as ScipSettings;

/// Registry-owned native representation, separately admitted from intent.
pub use pse_model::generated::enums::NativeRepresentation as Representation;
/// Whether the algebraic router assesses this registry-owned representation.
pub const fn algebraic(representation: Representation) -> bool {
    !matches!(representation, Representation::Trajectory)
}

/// The one capability record of an adapter (F21). Routing eligibility and the published
/// `runtime.solver_capabilities` row both derive from these fields; nothing else grants
/// eligibility. Model-specific admission is a separate, contextual result.
#[derive(Clone, Copy, Debug)]
pub struct Capability {
    /// Original-coordinate structural admission declared by this adapter.
    pub structural: crate::structural::Policy,
    /// Native interpretation of authored lexicographic degradation.
    pub lexicographic_degradation: crate::routing::DegradationSupport,
    /// Representable mathematical classes.
    pub classes: &'static [ProblemClass],
    /// The classes automatic routing may choose this adapter for, a subset of `classes`;
    /// every other class needs explicit selection (ADR-0121). Automatic routing takes the
    /// problem's classes most specific first and selects among the eligible adapters
    /// automatic for the first class that has one, by [`BackendExecution::automatic`].
    pub automatic_classes: &'static [ProblemClass],
    /// Required derivative representation.
    pub derivatives: DerivativeCapability,
    /// Externally supplied starting-state support.
    pub warm: WarmCapability,
    /// Arbitrary variable bounds are representable.
    pub general_bounds: bool,
    /// One-sided bounds are representable, as sign constraints on shifted coordinates,
    /// when general bounds are not.
    pub sign_bounds: bool,
    /// The adapter can consume more than one admitted native thread.
    pub parallel: bool,
    /// The adapter serves the explicit certify intent: global bounds and infeasibility
    /// conclusions over declared finite boxes (ADR-0106 §8–§9).
    pub certifies: bool,
    /// Constraint handlers the adapter consumes for forms a native realization leaves to
    /// the backend (ADR-0104). A structure requiring any other form is ineligible.
    pub native_forms: &'static [NativeConstraintForm],
    /// Structural requirements of a formulation the adapter honours with a method its
    /// settings select (ADR-0104 §5, [`BackendSettings::honours`]). A structure stating any
    /// other requirement is ineligible.
    pub requirements: &'static [ModelingStructuralRequirement],
    /// The classes in which the adapter optimizes several objectives lexicographically in
    /// one native solve (ADR-0111 item 4); a structure with several objectives in any other
    /// class is ineligible.
    pub lexicographic: &'static [ProblemClass],
    /// The adapter solves the independent points of a study that share one prepared
    /// structure as one parallel batch on the admitted threads (Plan 22 N5).
    pub batch: bool,
    /// Supports contextual parameter analysis: original multipliers for optimizing KKT
    /// sensitivity, or a qualified regular-square Root response. Automatic routing of a
    /// sensitivity request prefers such an adapter (ADR-0118/ADR-0144).
    pub sensitivities: bool,
    /// Native allocation/data reuse boundary.
    pub reuse: &'static str,
    /// Actual interrupt checkpoints.
    pub cancellation: &'static str,
    /// Available native diagnostic families.
    pub diagnostics: &'static str,
}
impl Capability {
    /// The published inventory row of this record.
    pub fn row(&self, backend: Backend) -> pse_model::generated::runtime::solver_capabilities::Row {
        pse_model::generated::runtime::solver_capabilities::Row {
            backend,
            structural_policy: self.structural,
            lexicographic_degradation: self.lexicographic_degradation,
            classes: self.classes.to_vec(),
            automatic_classes: self.automatic_classes.to_vec(),
            derivatives: self.derivatives,
            warm: self.warm,
            reuse: self.reuse.into(),
            cancellation: self.cancellation.into(),
            diagnostics: self.diagnostics.into(),
            general_bounds: self.general_bounds,
            sign_bounds: self.sign_bounds,
            parallel: self.parallel,
            certifies: self.certifies,
            native_forms: self.native_forms.to_vec(),
            requirements: self.requirements.to_vec(),
            lexicographic_classes: self.lexicographic.to_vec(),
            batch: self.batch,
            sensitivities: self.sensitivities,
        }
    }
}

/// Original acceptance budgets, coordinate transport and the normalized feasibility budget
/// of one solved function: the policy inputs an adapter derives native scaling from.
#[derive(Clone, Copy, Debug)]
pub struct Budgets<'a> {
    /// Complete resolved numerical policy consumed by conditional adapter admission.
    pub accuracy: &'a ResolvedAccuracy,
    /// Original physical acceptance budgets.
    pub tolerances: &'a Tolerances,
    /// Coordinate transport of the function the adapter evaluates.
    pub normalization: &'a Normalization,
}

/// One execution of a selected adapter on the owning worker, in native coordinates.
pub struct Input<'a> {
    /// The adapter's representation.
    pub problem: Problem<'a>,
    /// Finite shared controls.
    pub controls: &'a Controls,
    /// Stopping budgets resolved for this attempt.
    pub accuracy: &'a ResolvedAccuracy,
    /// Typed settings, admitted for this adapter.
    pub settings: &'a BackendSettings,
    /// Immutable contextual observation admitted for this attempt.
    pub snapshot: &'a Snapshot,
    /// Original contextual witness; representation transforms retain their own checks.
    pub structure: Option<&'a crate::structural::Assessment>,
    /// Cancellation, deadline and bounded progress.
    pub execution: Execution,
    /// Native-coordinate acceptance budgets.
    pub tolerances: &'a Tolerances,
    /// Native-coordinate seed, when one is submitted.
    pub warm: Option<&'a WarmStart>,
    /// Native reuse identity.
    pub compatibility: Compatibility,
}
/// Representation-specific native input.
pub enum Problem<'a> {
    /// Preprocessed NLP transport from the shared pipeline.
    Nlp {
        /// Minimization callbacks in native coordinates.
        oracle: Box<dyn NlpOracle>,
        /// Projected start.
        initial: &'a [f64],
        /// Authored sense, used only to recover authored output.
        sense: ObjectiveSense,
    },
    /// Square residual equations in normalized coordinates.
    Roots {
        /// Normalized residual transport.
        oracle: Box<dyn NleOracle>,
        /// Normalized start.
        initial: &'a [f64],
        /// Policy inputs for native scaling.
        budgets: Budgets<'a>,
        /// Owner of the evaluators behind `oracle`; retained as long as a session retains it.
        owner: Option<Box<dyn Any>>,
    },
    /// Normalized coefficient model.
    Coefficients {
        /// Normalized coefficients.
        problem: &'a CoefficientProblem,
        /// Transported convexity evidence for a quadratic objective.
        certificate: Option<&'a dyn QuadraticEvidence>,
        /// Model coordinate transport, for native diagnostics and sparse starts.
        normalization: &'a Normalization,
        /// Original row constants, for native diagnostics.
        row_constants: &'a [f64],
    },
    /// Normalized explicit cone model.
    Cone {
        /// Normalized cone data.
        problem: &'a ConicProblem,
        /// Transported convexity evidence.
        certificate: &'a dyn QuadraticEvidence,
    },
    /// Factorable program in original coordinates; budgets and the seed are original too.
    Factorable {
        /// Admitted program; its objective is in the authored sense.
        program: &'a FactorableProgram,
        /// Original start in program column order.
        initial: &'a [f64],
        /// Mathematical purpose; only optimization intents export the objective.
        intent: SolveIntent,
        /// Model coordinate transport, which converts normalized objective budgets into
        /// original objective units.
        normalization: &'a Normalization,
    },
}

impl std::fmt::Debug for Input<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Input")
            .field("problem", &self.problem)
            .field("settings", &self.settings)
            .field("compatibility", &self.compatibility)
            .finish_non_exhaustive()
    }
}
impl std::fmt::Debug for Problem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Nlp { .. } => "Problem::Nlp",
            Self::Roots { .. } => "Problem::Roots",
            Self::Coefficients { .. } => "Problem::Coefficients",
            Self::Cone { .. } => "Problem::Cone",
            Self::Factorable { .. } => "Problem::Factorable",
        })
    }
}

/// The backend-execution adapter. Implementations are unit values in the static table.
pub trait BackendExecution: Sync + std::fmt::Debug {
    /// Registry identity of this adapter.
    fn backend(&self) -> Backend;
    /// The one capability record.
    fn capability(&self) -> &'static Capability;
    /// Native input this adapter consumes.
    fn representation(&self) -> Representation;
    /// Project consumed physical residual allowances into this adapter's primal
    /// work tolerance. Physical acceptance and its native scaling remain frozen.
    /// Each pair names an original row position and its requested physical error.
    ///
    /// # Errors
    /// A row is absent, or the projected positive tolerance is not representable.
    fn residual_work_tolerance(
        &self,
        _settings: &BackendSettings,
        budgets: Budgets<'_>,
        demands: &[(usize, f64)],
    ) -> Result<Option<f64>, ProblemError> {
        let mut requested: Option<f64> = None;
        for &(row, allowance) in demands {
            let scale = budgets
                .normalization
                .rows
                .get(row)
                .ok_or_else(|| ProblemError::Contract("work demand row is absent".into()))?;
            if !allowance.is_finite() || allowance <= 0. || !scale.is_finite() || *scale <= 0. {
                return Err(ProblemError::Contract(
                    "invalid residual work allowance or coordinate scale".into(),
                ));
            }
            let value =
                pse_math::normalization::checked_ratio(allowance, *scale).map_err(|_| {
                    ProblemError::numerical("residual work tolerance is not representable")
                })?;
            requested = Some(requested.map_or(value, |held| held.min(value)));
        }
        Ok(requested)
    }
    /// Whether this binary links the native implementation.
    fn linked(&self) -> bool;
    /// Complete pre-operation counters for the numerical callback route. Coefficient and
    /// factorable preparation must separately qualify their actual route.
    fn work_coverage(&self, _execution: &Execution) -> crate::solve::WorkCoverage {
        crate::solve::WorkCoverage::default()
    }
    /// Automatic-selection preference among the eligible adapters automatic for one class
    /// (`Capability::automatic_classes`), lower first; `None` is explicit-only. It orders a
    /// choice and never grants eligibility.
    fn automatic(&self) -> Option<u8>;
    /// Identity of the linked native build beyond the pinned crate: library versions and
    /// the process's numerical contract (ADR-0108 item 14). `None` when the pinned crate
    /// fully determines the build. Every profile key includes it through
    /// [`Table::build_identity`].
    fn build(&self) -> Option<ContentHash> {
        None
    }
    /// Revalidate only observations this adapter's selected settings consume.
    fn validate_snapshot(
        &self,
        _settings: &BackendSettings,
        snapshot: &Snapshot,
    ) -> Result<(), ProblemError> {
        snapshot.validate_for(self)
    }
    /// Exact mathematical order required by this adapter's selected method.
    fn required_order(
        &self,
        requirements: &Requirements<'_>,
    ) -> Option<pse_kernels::DerivativeOrder> {
        crate::routing::derivative_demand(self.capability(), requirements.controls)
    }
    /// Exact executable operation product for the admitted method.
    fn required_artifact(
        &self,
        requirements: &Requirements<'_>,
    ) -> Option<crate::routing::ArtifactDemand> {
        self.required_order(requirements)
            .map(crate::routing::ArtifactDemand::Derivatives)
    }
    /// Compose static class/intent rules with existing contextual owners. This operation
    /// reads no ambient environment and starts no native service or worker.
    fn assess(&self, requirements: &Requirements<'_>) -> crate::routing::Eligibility {
        let mut assessment =
            crate::routing::assess_static(self.backend(), self.capability(), requirements);
        if let Some(cause) = requirements.context.refusals.get(&self.backend()) {
            assessment
                .reasons
                .push(crate::routing::Ineligible::Contextual);
            assessment.causes.push(cause.clone());
        }
        if let Err(cause) = self.admit_settings(
            requirements.settings,
            requirements.controls,
            &requirements.context.snapshot,
        ) {
            assessment.refuse(cause);
        }
        self.assess_representation(requirements, &mut assessment);
        assessment.finish();
        assessment
    }
    /// Representation-specific contextual checks remain with their adapter owner.
    fn assess_representation(
        &self,
        requirements: &Requirements<'_>,
        assessment: &mut crate::routing::Eligibility,
    ) {
        assess_representation(self, requirements, assessment);
    }
    /// Typed settings and native controls of a selected route: settings must belong to
    /// this adapter and the thread count to its capability.
    ///
    /// # Errors
    /// Foreign settings or an unadmitted thread count.
    fn admit_settings(
        &self,
        settings: &BackendSettings,
        controls: &Controls,
        _snapshot: &Snapshot,
    ) -> Result<(), ProblemError> {
        if settings.backend().is_some_and(|b| b != self.backend()) {
            return Err(ProblemError::Contract(
                "backend settings do not match selected route".into(),
            ));
        }
        if controls.threads != 1 && !self.capability().parallel {
            return Err(ProblemError::Unsupported(
                "selected linked native profile is serial".into(),
            ));
        }
        Ok(())
    }
    /// Model-contract admission of a selected route before any worker exists.
    ///
    /// # Errors
    /// The contract, guards or settings cannot be represented natively.
    fn admit_contract(
        &self,
        _contract: &OracleContract,
        _guards: &BTreeMap<SemanticId, GuardSign>,
        _settings: &BackendSettings,
        _budgets: Budgets<'_>,
    ) -> Result<(), ProblemError> {
        Ok(())
    }
    /// This adapter's warm-start payload for a primal-only explicit start.
    ///
    /// # Errors
    /// The adapter has no primal-start interface.
    fn primal_start(&self, _primal: Vec<f64>) -> Result<WarmPayload, ProblemError> {
        Err(ProblemError::Unsupported(
            "selected backend has no primal-start interface".into(),
        ))
    }
    /// Whether `payload` is this adapter's warm-start variant.
    fn accepts(&self, _payload: &WarmPayload) -> bool {
        false
    }
    /// Run `work` inside this adapter's execution scope, such as an admitted local thread
    /// pool. Sessions built inside the scope are also torn down inside it.
    ///
    /// # Errors
    /// The scope could not be entered.
    fn scope(
        &self,
        _threads: usize,
        _stack: usize,
        work: &mut (dyn FnMut() + Send),
    ) -> Result<(), ProblemError> {
        work();
        Ok(())
    }
    /// Additional threads created by this adapter's execution scope. Each distinct
    /// adapter is counted independently, including nested pools; serial scopes reuse
    /// the calling thread. The runtime reserves their qualified stack extent before
    /// entering the scope and retains it through pool teardown and join.
    fn scope_threads(&self, _threads: usize) -> usize {
        0
    }
    /// Prepare (reusing compatible retained state), solve and retain on the owning worker.
    ///
    /// # Errors
    /// Refused input, unavailable reuse or a failed native call.
    fn execute(
        &self,
        retained: &mut Retained,
        input: Input<'_>,
    ) -> Result<SolveReport, ProblemError>;
    /// Solve the independent inputs of one batch, one result per input in order (Plan 22
    /// N5). An adapter whose record declares `batch` solves them in parallel on the
    /// admitted threads; every other solves them in turn.
    fn execute_batch(
        &self,
        retained: &mut Retained,
        inputs: Vec<Input<'_>>,
    ) -> Vec<Result<SolveReport, ProblemError>> {
        inputs
            .into_iter()
            .map(|input| self.execute(retained, input))
            .collect()
    }
}

/// The adapter of every registry `Backend`: a static map with an exhaustive match, so a
/// new registry value cannot compile without its adapter.
pub const fn adapter(backend: Backend) -> &'static dyn BackendExecution {
    match backend {
        Backend::Ipopt => &ipopt::ADAPTER,
        Backend::Pounce => &pounce::ADAPTER,
        Backend::Kinsol => &kinsol::ADAPTER,
        Backend::Highs => &highs::ADAPTER,
        Backend::Clarabel => &clarabel::ADAPTER,
        Backend::Diffsol => &dynamics::DIFFSOL,
        Backend::Idas => &dynamics::IDAS,
        Backend::Scip => &scip::ADAPTER,
        Backend::PounceConvex => &pounce_convex::ADAPTER,
        Backend::Uno => &uno::ADAPTER,
        Backend::Petsc => &petsc::ADAPTER,
    }
}
static ADAPTERS: [&dyn BackendExecution; Backend::ALL.len()] = {
    let mut out: [&dyn BackendExecution; Backend::ALL.len()] =
        [adapter(Backend::Ipopt); Backend::ALL.len()];
    let mut i = 0;
    while i < Backend::ALL.len() {
        out[i] = adapter(Backend::ALL[i]);
        i += 1;
    }
    out
};
/// Every adapter in registry order, linked or not.
pub static LINKED: Table = Table::new(&ADAPTERS);

/// An adapter table in a fixed order. Production uses [`LINKED`]; a caller may expose a
/// subset, and a test may expose its own adapter.
#[derive(Clone, Copy, Debug)]
pub struct Table {
    adapters: &'static [&'static dyn BackendExecution],
}
impl Table {
    /// A table over exactly these adapters, in this order.
    pub const fn new(adapters: &'static [&'static dyn BackendExecution]) -> Self {
        Self { adapters }
    }
    /// The adapter registered for `backend`.
    pub fn get(&self, backend: Backend) -> Option<&'static dyn BackendExecution> {
        self.adapters
            .iter()
            .copied()
            .find(|a| a.backend() == backend)
    }
    /// Every adapter in table order.
    pub fn adapters(&self) -> impl Iterator<Item = &'static dyn BackendExecution> + '_ {
        self.adapters.iter().copied()
    }
    /// Identity of every linked adapter's native build, in table order.
    pub fn build_identity(&self) -> ContentHash {
        let mut h = pse_ids::FramedHasher::new(pse_ids::Frame::NativeBuildV1);
        h.str(include_str!(concat!(env!("OUT_DIR"), "/pounce-source.txt")));
        for adapter in self.adapters().filter(|a| a.linked()) {
            h.str(adapter.backend().as_str());
            match adapter.build() {
                Some(build) => h.hash(&build),
                None => h.u64(0),
            };
        }
        h.finish_hash()
    }
    /// The published inventory: one row per linked adapter.
    pub fn published(&self) -> Vec<pse_model::generated::runtime::solver_capabilities::Row> {
        self.adapters()
            .filter(|a| a.linked())
            .map(|a| a.capability().row(a.backend()))
            .collect()
    }
}

/// Typed backend settings of a request. Each variant is its adapter's pse-owned settings
/// type (`crate::settings`), present in every build, and identity derives from serde, never
/// from a hand-written field list (F09). The document form is tagged by the registry
/// `backend` spelling; the routed backend's native defaults have no document form, so a
/// document states them by omitting its settings. A backend this build does not link is
/// refused at admission, never at decoding.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum BackendSettings {
    /// Native defaults and the common semantic controls on the routed backend.
    #[default]
    #[serde(skip)]
    Default,
    /// Ipopt linear solver with its parameters, barrier strategy and initial-point push.
    #[schemars(title = "IpoptSettings")]
    Ipopt(crate::settings::ipopt::Settings),
    /// POUNCE method and complete native FERAL configuration.
    #[schemars(title = "PounceSettings")]
    Pounce(crate::settings::pounce::Settings),
    /// KINSOL method controls; scales and the step tolerance derive from policy.
    #[schemars(title = "KinsolSettings")]
    Kinsol(crate::settings::kinsol::Method),
    /// HiGHS method, opt-in diagnostics and partial MIP start.
    #[schemars(title = "HighsSettings")]
    Highs(crate::settings::highs::Settings),
    /// The pse-owned Clarabel settings: mode and every admitted native control.
    #[schemars(title = "ClarabelSettings")]
    Clarabel(crate::conic::Settings),
    /// SCIP's nested Ipopt linear solver, seed and node budget.
    #[schemars(title = "ScipSettings")]
    Scip(ScipSettings),
    /// The POUNCE-convex interior-point method's choices and FERAL configuration.
    #[schemars(title = "PounceConvexSettings")]
    PounceConvex(crate::settings::pounce_convex::Settings),
    /// First-order Uno filter trust-region SQP or SLP.
    #[schemars(title = "UnoSettings")]
    Uno(crate::settings::uno::Settings),
    /// Serial PETSc trust-region, pseudo-time or nonlinear Schwarz profile.
    #[schemars(title = "PetscSettings")]
    Petsc(crate::settings::petsc::Settings),
}
impl BackendSettings {
    /// The backend these settings belong to; `None` for native defaults.
    pub fn backend(&self) -> Option<Backend> {
        match self {
            Self::Default => None,
            Self::Ipopt(_) => Some(Backend::Ipopt),
            Self::Pounce(_) => Some(Backend::Pounce),
            Self::Kinsol(_) => Some(Backend::Kinsol),
            Self::Highs(_) => Some(Backend::Highs),
            Self::Clarabel(_) => Some(Backend::Clarabel),
            Self::Scip(_) => Some(Backend::Scip),
            Self::PounceConvex(_) => Some(Backend::PounceConvex),
            Self::Uno(_) => Some(Backend::Uno),
            Self::Petsc(_) => Some(Backend::Petsc),
        }
    }
    /// The document form of these settings: `None` for native defaults.
    pub fn document(&self) -> Option<&Self> {
        match self {
            Self::Default => None,
            settings => Some(settings),
        }
    }
    /// The settings a document states; its absence is the routed backend's native defaults.
    pub fn from_document(document: Option<Self>) -> Self {
        document.unwrap_or_default()
    }
    /// Whether these settings, on `backend`, run a method that honours a structural
    /// requirement of the formulation (ADR-0104 §5). The l1 exact penalty an authored
    /// `penalty(l1)` realization states is honoured only by POUNCE's
    /// [`crate::settings::pounce::Method::L1ExactPenalty`]: explicit POUNCE settings must
    /// select it, and native defaults take it from the author's realization
    /// ([`Self::for_requirements`]), which is the author's selection, never an automatic one.
    pub fn honours(&self, backend: Backend, requirement: ModelingStructuralRequirement) -> bool {
        match requirement {
            ModelingStructuralRequirement::L1ExactPenalty => {
                backend == Backend::Pounce
                    && match self {
                        Self::Default => true,
                        Self::Pounce(settings) => {
                            settings.method == crate::settings::pounce::Method::L1ExactPenalty
                        }
                        _ => false,
                    }
            }
        }
    }
    /// The effective settings of a route over a structure stating `requirements`: native
    /// defaults on POUNCE under an authored `penalty(l1)` realization run its l1 exact
    /// penalty (ADR-0104 §5); any other settings are unchanged, and routing has already
    /// refused settings that do not honour a requirement ([`Self::honours`]).
    #[must_use]
    pub fn for_requirements(
        self,
        backend: Backend,
        requirements: &[ModelingStructuralRequirement],
    ) -> Self {
        match self {
            Self::Default
                if backend == Backend::Pounce
                    && requirements.contains(&ModelingStructuralRequirement::L1ExactPenalty) =>
            {
                Self::Pounce(crate::settings::pounce::Settings {
                    method: crate::settings::pounce::Method::L1ExactPenalty,
                    ..Default::default()
                })
            }
            settings => settings,
        }
    }
    /// The selected method relaxes every constraint row (the ℓ1 exact penalty, ADR-0109), so
    /// no presolve pass may assume the rows hold.
    pub fn relaxes_rows(&self) -> bool {
        match self {
            Self::Pounce(settings) => {
                settings.method == crate::settings::pounce::Method::L1ExactPenalty
            }
            _ => false,
        }
    }
    /// The partial explicit start these settings submit, in original coordinates.
    pub fn partial_start(&self) -> Option<&BTreeMap<SemanticId, f64>> {
        match self {
            Self::Highs(settings) => settings.sparse_start.as_ref(),
            _ => None,
        }
    }
    /// Complete settings identity, derived from the serde encoding of the document form
    /// (registry and serde spellings, never Rust type names; ADR-0116 Outcome 9).
    ///
    /// # Errors
    /// A native settings serializer refused its value.
    pub fn identity(&self) -> Result<ContentHash, ProblemError> {
        pse_ids::document::of(pse_ids::Frame::BackendSettingsV5, &self.document())
            .map_err(|e| ProblemError::Internal(e.to_string()))
    }
}

/// Worker-owned native state retained from the previous step of a finite sequence.
/// Opaque to workflows: each adapter recognizes only its own session. It never crosses a
/// worker boundary and is dropped on the owning thread.
///
/// It also holds the parametric factor of an advanced step (Plan 22 Y5c2), which is not
/// an adapter's state: a certified sensitivity step that keeps it replaces any earlier one,
/// and the worker charges its bytes to the job's allowance before the next step runs
/// (ADR-0118 item 12, I14). Clearing the adapter's session keeps it.
#[derive(Default)]
pub struct Retained {
    session: Option<(Backend, Box<dyn Any>)>,
    session_charge: Option<Box<dyn Any + Send>>,
    advance: Option<(crate::kkt::Advance, Option<Box<dyn Any + Send>>)>,
    scheduler_discard: Option<Backend>,
}
impl std::fmt::Debug for Retained {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Retained")
            .field("backend", &self.session.as_ref().map(|s| s.0))
            .field(
                "advance",
                &self
                    .advance
                    .as_ref()
                    .map(|(a, charge)| (a.bytes(), charge.is_some())),
            )
            .finish_non_exhaustive()
    }
}
impl Retained {
    /// Drop the retained adapter session now. A kept advanced-step factor stays.
    pub fn clear(&mut self) {
        self.session = None;
        self.session_charge = None;
        self.scheduler_discard = None;
    }
    /// Relinquish idle direct HiGHS state on its owning thread for a waiting Uno
    /// scheduler reset. Immutable starts and independent response factors survive.
    pub fn relinquish_scheduler(&mut self) {
        if self.backend() == Some(Backend::Highs) {
            self.clear();
            self.scheduler_discard = Some(Backend::Highs);
        }
    }
    /// Keep the admitted opaque-library allowance until native session destruction.
    /// The runtime supplies this lease; native allocation observations are not exact extents.
    pub fn charge_session(&mut self, charge: Box<dyn Any + Send>) {
        if self.session.is_some() {
            self.session_charge = Some(charge);
        }
    }
    /// Keep an advanced-step factor, uncharged, replacing any earlier one with its charge.
    pub fn keep(&mut self, advance: crate::kkt::Advance) {
        self.advance = Some((advance, None));
    }
    /// Bytes of a kept advanced-step factor not yet charged to the job's allowance.
    pub fn uncharged(&self) -> Option<usize> {
        match &self.advance {
            Some((advance, None)) => Some(advance.bytes()),
            _ => None,
        }
    }
    /// Hold `charge`, the job allowance reserved for the kept factor, until the factor is
    /// dropped.
    pub fn charge(&mut self, charge: Box<dyn Any + Send>) {
        if let Some((_, held)) = &mut self.advance {
            *held = Some(charge);
        }
    }
    /// The kept advanced-step factor.
    pub fn advance(&self) -> Option<&crate::kkt::Advance> {
        self.advance.as_ref().map(|(a, _)| a)
    }
    /// Drop the kept advanced-step factor and release its charge.
    pub fn release(&mut self) {
        self.advance = None;
    }
    /// Whether any adapter retains native state.
    pub fn is_empty(&self) -> bool {
        self.session.is_none()
    }
    /// The backend whose native state is retained.
    pub fn backend(&self) -> Option<Backend> {
        self.session.as_ref().map(|s| s.0)
    }
    /// Reuse the retained session of `backend` when `reuse` accepts it (refreshing it with
    /// this step's data); otherwise tear the old state down and build a new session when
    /// `policy` allows. Returns the session and whether it was reused.
    ///
    /// # Errors
    /// `RequireReuse` with incompatible retained state, or a failed update or build.
    pub fn session<S: Any>(
        &mut self,
        backend: Backend,
        policy: crate::solve::ReusePolicy,
        reuse: impl FnOnce(&mut S) -> Result<bool, ProblemError>,
        build: impl FnOnce() -> Result<S, ProblemError>,
    ) -> Result<(&mut S, bool), ProblemError> {
        if self.session.is_none()
            && self.scheduler_discard == Some(backend)
            && policy == crate::solve::ReusePolicy::RequireReuse
        {
            return Err(ProblemError::Reuse {
                backend,
                refusal: crate::ReuseRefusal::Structure,
            });
        }
        let refusal = match &mut self.session {
            Some((held, state)) if *held == backend => match state.downcast_mut::<S>() {
                Some(session) => (!reuse(session)?).then_some(crate::ReuseRefusal::Structure),
                None => Some(crate::ReuseRefusal::Structure),
            },
            Some((held, _)) => Some(crate::ReuseRefusal::Foreign(*held)),
            None => Some(crate::ReuseRefusal::Structure),
        };
        let reused = refusal.is_none();
        if let Some(refusal) = refusal {
            if policy == crate::solve::ReusePolicy::RequireReuse && self.session.is_some() {
                return Err(ProblemError::Reuse { backend, refusal });
            }
            // Native teardown precedes construction of the replacement.
            self.clear();
            self.session = Some((backend, Box::new(build()?)));
        }
        let session = self
            .session
            .as_mut()
            .and_then(|(_, state)| state.downcast_mut::<S>())
            .ok_or_else(|| ProblemError::Internal("lost retained native session".into()))?;
        Ok((session, reused))
    }
}

/// Run `work` inside the scope of every distinct adapter it executes, such as an admitted
/// local thread pool, so native sessions are built and torn down inside those scopes.
///
/// # Errors
/// A scope could not be entered, or `work` failed.
pub fn scoped<T: Send, E: From<ProblemError> + Send>(
    adapters: &[&dyn BackendExecution],
    threads: usize,
    stack: usize,
    work: impl FnOnce() -> Result<T, E> + Send,
) -> Result<T, E> {
    let mut distinct: Vec<&dyn BackendExecution> = Vec::with_capacity(adapters.len());
    for adapter in adapters {
        if !distinct.iter().any(|a| a.backend() == adapter.backend()) {
            distinct.push(*adapter);
        }
    }
    let mut work = Some(work);
    let mut out = None;
    let mut run = || {
        if let Some(work) = work.take() {
            out = Some(work());
        }
    };
    enter(&distinct, threads, stack, &mut run)?;
    out.unwrap_or_else(|| {
        Err(ProblemError::Internal("execution scope did not run its work".into()).into())
    })
}
/// Qualified stack extent of the actual teams created by [`scoped`]. This is an
/// allocation allowance, not an observation or bound of resident native memory.
///
/// # Errors
/// The sum or multiplication is not representable.
pub fn scope_stack_bytes(
    adapters: &[&dyn BackendExecution],
    threads: usize,
    stack: usize,
) -> Result<usize, ProblemError> {
    let mut distinct = Vec::with_capacity(adapters.len());
    let mut count = 0usize;
    for adapter in adapters {
        if distinct.contains(&adapter.backend()) {
            continue;
        }
        distinct.push(adapter.backend());
        count = count
            .checked_add(adapter.scope_threads(threads))
            .ok_or_else(|| ProblemError::memory("native scope thread count overflow"))?;
    }
    count
        .checked_mul(stack)
        .ok_or_else(|| ProblemError::memory("native scope stack extent overflow"))
}
fn enter(
    adapters: &[&dyn BackendExecution],
    threads: usize,
    stack: usize,
    work: &mut (dyn FnMut() + Send),
) -> Result<(), ProblemError> {
    let Some((first, rest)) = adapters.split_first() else {
        work();
        return Ok(());
    };
    let mut inner = Ok(());
    first.scope(threads, stack, &mut || {
        inner = enter(rest, threads, stack, work);
    })?;
    inner
}

/// Another adapter's settings variant reached `backend`.
fn foreign(backend: Backend) -> ProblemError {
    ProblemError::Contract(format!("wrong {} settings", backend.as_str()))
}
#[cfg(not(all(
    feature = "ipopt",
    feature = "pounce",
    feature = "kinsol",
    feature = "highs",
    feature = "scip",
    feature = "uno",
    feature = "petsc"
)))]
fn unlinked(backend: Backend) -> ProblemError {
    ProblemError::Unavailable {
        backend,
        alternatives: vec![],
    }
}
fn representation(backend: Backend) -> ProblemError {
    ProblemError::Internal(format!(
        "{} received another adapter's representation",
        backend.as_str()
    ))
}

#[cfg(test)]
mod tests;

/// Shared representation contracts, composed by adapter-owned subtype checks.
pub(crate) fn assess_representation<A: BackendExecution + ?Sized>(
    adapter: &A,
    requirements: &Requirements<'_>,
    assessment: &mut crate::routing::Eligibility,
) {
    use crate::routing::{ArtifactDemand, EvidenceDemand};
    let context = &requirements.context;
    if matches!(
        adapter.representation(),
        Representation::Factorable | Representation::Nlp | Representation::Roots
    ) {
        // Coefficient classification belongs to a different representation's proof.
        // Preserve its original dependencies while this owner independently admits
        // the actual guarded program or callback contract. Ordered selection still
        // retains any admissible higher-priority coefficient candidate's demand.
        assessment
            .evidence
            .retain(|demand| !matches!(demand, EvidenceDemand::Class(_)));
    }
    if matches!(
        adapter.representation(),
        Representation::Coefficients | Representation::Cone
    ) && matches!(
        requirements.facts.class_status,
        pse_math::presolve::ClassStatus::RepresentationLimited(
            pse_math::presolve::ClassWitness::CoefficientRange
        )
    ) {
        assessment.refuse(ProblemError::Math(pse_math::MathError::CoefficientRange));
        return;
    }

    if let pse_math::presolve::ClassStatus::Pending(dependencies) = &requirements.facts.class_status
    {
        let unavailable = dependencies.iter().any(|dependency| {
            matches!(
                dependency,
                pse_math::presolve::ClassDependency::MissingSymbolicExpression { .. }
            )
        });
        if unavailable
            && matches!(
                adapter.representation(),
                Representation::Coefficients | Representation::Cone
            )
        {
            // The performed extractor named a missing mandatory input for this
            // representation; this is not a proof of mathematical nonlinearity.
            assessment
                .reasons
                .push(crate::routing::Ineligible::Contextual);
            return;
        }
    }

    match adapter.representation() {
        Representation::Nlp | Representation::Roots => {
            if let Some(original) = context.oracle {
                // Prepared callbacks may be weaker than available selected mathematics.
                // Scientific admission uses that availability; final execution validates
                // the actually prepared contract before native construction.
                let mut scientific = original.clone();
                scientific.derivatives = requirements.facts.derivatives;
                let contract = &scientific;
                let order = adapter
                    .required_order(requirements)
                    .unwrap_or(pse_kernels::DerivativeOrder::Value);
                if let Err(cause) = contract.validate(order) {
                    assessment.refuse(cause);
                }
                if let Some(budgets) = context.budgets {
                    if let Err(cause) = adapter.admit_contract(
                        contract,
                        context.guards,
                        requirements.settings,
                        budgets,
                    ) {
                        assessment.refuse(cause);
                    }
                } else if adapter.backend() == Backend::Kinsol {
                    assessment.evidence.push(EvidenceDemand::CallbackContract);
                }
            } else {
                assessment.evidence.push(EvidenceDemand::CallbackContract);
            }
        }
        Representation::Coefficients => {
            if let Some(problem) = context.coefficients {
                if problem
                    .hessian
                    .as_ref()
                    .is_some_and(|q| q.val().iter().any(|value| *value != 0.0))
                    && context.certificate.is_none()
                {
                    assessment
                        .evidence
                        .push(EvidenceDemand::Class(ProblemClass::ConvexQuadratic));
                } else if let Err(cause) = problem.validate_convex(context.certificate) {
                    assessment.refuse(cause);
                }
            } else {
                assessment.evidence.push(EvidenceDemand::Coefficients);
            }
        }
        Representation::Cone => {
            if let Some(cone) = context.cone {
                if let Err(cause) = cone.problem.validate(cone.certificate) {
                    assessment.refuse(cause);
                }
            } else if let Some(problem) = context.coefficients {
                // The existing coefficient-to-cone owner lowers only equality and
                // nonnegative row/bound cones. Assess its existing convex contract;
                // lowering is a selected artifact, not an unrelated cone proof.
                if problem
                    .hessian
                    .as_ref()
                    .is_some_and(|q| q.val().iter().any(|value| *value != 0.0))
                    && context.certificate.is_none()
                {
                    assessment
                        .evidence
                        .push(EvidenceDemand::Class(ProblemClass::ConvexQuadratic));
                } else if let Err(cause) = problem.validate_convex(context.certificate) {
                    assessment.refuse(cause);
                }
            } else {
                assessment.evidence.push(EvidenceDemand::Cone);
            }
        }
        Representation::Factorable => {
            if let Some(program) = context.factorable {
                assessment.factorable_refusals = admit_program(program, requirements.intent);
                if !assessment.factorable_refusals.is_empty() {
                    assessment
                        .reasons
                        .push(crate::routing::Ineligible::Contextual);
                }
            } else {
                assessment.evidence.push(EvidenceDemand::Factorable);
            }
        }
        Representation::Trajectory => {}
    }
    let representation = ArtifactDemand::Representation(adapter.representation());
    if !context.prepared.contains(&representation) {
        assessment.artifacts.push(representation);
    }
}
