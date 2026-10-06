// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One completion-owned solve lifecycle; finite batches never create persistent native sessions.
#[cfg(feature = "solver-kinsol")]
pub mod blocks;
#[cfg(feature = "solver-kinsol")]
pub mod causal;
mod certified_accuracy;
mod derived;
pub(crate) mod engineering_accuracy;
mod kkt_accuracy;
pub mod multistart;
mod objective_accuracy;
pub(crate) mod output_program;
pub mod paths;
#[cfg(feature = "solver-petsc")]
pub mod petsc;
mod starts;
use super::{
    ExecutableCase, ExecutionWorker, MathRuntimeError, MathService, Preparation, WorkerBudget,
};
pub use derived::{
    DerivedAttempt, DerivedRequest, OriginalProposal, PreparedDerived, PreparedFamily, PreparedRung,
};
use pse_backend_native::{
    self as native, ProblemError,
    execution::{self, BackendSettings, Retained},
    quality::{self, Quality, Tolerances, Violation},
    routing::{self, Route},
    solve::*,
};
use pse_columnar::flight::FlightCancellation;
use pse_ids::FramedHasher;
use pse_kernels::ProviderFactory;
pub use pse_math::convexity::ConvexityPolicy;
/// Shared immutable driver history; cloning retains one admitted allocation.
pub type StrategyTrace = Arc<super::strategy::Trace>;
use pse_math::{
    binding::{CaseValues, ObjectiveSense},
    convexity::QuadraticEvidence,
    normalization::Normalization,
};
use pse_model::numerics::{NumericalPolicy, ResolvedNumericalPolicy};

/// Selected authored inputs to numerical resolution; analysis overrides stay in the profile.
#[derive(Clone, Debug, Default)]
pub struct NumericalInputs {
    /// Model/case/property declarations selected by the workflow.
    pub declarations: Vec<pse_math::numerics::SourcedRequirement>,
    /// Observable and physical closure targets beyond the algebraic coordinates.
    pub targets: Vec<pse_math::numerics::TargetSpec>,
    /// Original residual definitions of the case's implicit blocks, keyed by the provider
    /// their callers invoke, with unknown intervals under the case's bounds. A factorable
    /// route exports them in place of the blocks' realizations (ADR-0105 §1).
    pub implicit: BTreeMap<pse_kernels::ProviderKey, pse_math::factorable::ImplicitDefinition>,
}
use std::{collections::BTreeMap, future::Future, sync::Arc};

/// Unified solver request policy; physical tolerances are never inferred from trial magnitudes.
#[derive(Clone, Debug)]
pub struct SolverProfile {
    /// Qualified library-owned NLP preprocessing policy.
    pub presolve: native::presolve::Policy,
    /// ID-keyed physical magnitudes and independent numerical requirements.
    pub numerics: NumericalPolicy,
    /// Exact default or explicitly requested numerical convexity qualification.
    pub convexity: ConvexityPolicy,
    /// Requested automatic or declared composition and its preserved branch/work constraints.
    pub composition: pse_model::strategy::CompositionRequest,
    /// Consumer-owned evidence class and finite work for optional selected reconstruction.
    pub reconstruction: Option<native::derived::ReconstructionAccuracy>,
    /// Mathematical purpose.
    pub intent: SolveIntent,
    /// Deterministic auto or an explicit eligible backend.
    pub selection: SolverSelection,
    /// Finite shared resource/method controls.
    pub controls: Controls,
    /// Complete backend-specific typed settings.
    pub backend: BackendSettings,
    /// Parametric sensitivities to compute at the candidate (Plan 22 S1); only a modeling
    /// solve prepares the parametric program they need.
    pub sensitivity: Option<super::settings::SensitivityRequest>,
}
/// The one owner of request defaults: every boundary takes an omitted field from here
/// rather than restating it (ADR-0113).
impl Default for SolverProfile {
    fn default() -> Self {
        Self {
            presolve: native::presolve::Policy::default(),
            numerics: NumericalPolicy::default(),
            convexity: ConvexityPolicy::default(),
            intent: SolveIntent::Optimize,
            selection: SolverSelection::Auto,
            controls: Controls {
                hessian: HessianMode::Auto,
                ..Controls::default()
            },
            backend: BackendSettings::Default,
            sensitivity: None,
            composition: Default::default(),
            reconstruction: None,
        }
    }
}
impl SolverProfile {
    /// Resolve request-only curvature from the original producer's actual capability.
    /// Native adapters receive only a concrete acting method.
    pub(crate) fn resolve_curvature(&mut self, available: pse_kernels::DerivativeOrder) {
        if self.controls.hessian == HessianMode::Auto {
            self.controls.hessian = if available >= pse_kernels::DerivativeOrder::Second {
                HessianMode::Exact
            } else {
                HessianMode::LimitedMemory
            };
        }
    }
}
/// A compiled case with its selected values, providers and convexity evidence.
#[derive(Clone, Debug)]
struct AlgebraicCase {
    prepared: Preparation,
    case: Option<Arc<ExecutableCase>>,
    /// Actual auxiliary NLP callbacks; the primary factorable program keeps its own order.
    pricing_case: Option<PricingCase>,
    values: CaseValues,
    providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
    certificate: Option<Arc<dyn QuadraticEvidence>>,
    /// Factorable projection under `values`, built only for a factorable route, with the
    /// reservation that admits its retained size.
    factorable: Option<(
        Arc<pse_math::factorable::FactorableProgram>,
        Arc<pse_columnar::AllocationLease>,
    )>,
    /// The parametric program of a sensitivity request (Plan 22 S1), attached by
    /// [`PreparedSolve::with_sensitivity`].
    sensitivity: Option<ParametricPreparation<SensitivityProgram>>,
    /// The admitted coefficient cone and its map retain their allocation through execution.
    coefficient_cone: Option<(
        Arc<native::conic::Lowered>,
        Arc<pse_columnar::AllocationLease>,
    )>,
    /// A recognized convex program's cone form (ADR-0121), built only for a cone route
    /// over a program that is not a coefficient program, with its reservation.
    recognized: Option<(
        Arc<native::conic::Recognized>,
        Arc<pse_math::convexity::GramCertificate>,
        Arc<pse_columnar::AllocationLease>,
    )>,
}
/// Auxiliary callbacks and the compiler-issued requests that identify those programs.
#[derive(Clone, Debug)]
struct PricingCase {
    prepared: Preparation,
    executable: Arc<ExecutableCase>,
}
/// Availability of a requested parametric program, independent of its base solve.
#[derive(Clone, Debug)]
pub enum ParametricPreparation<T> {
    /// A compiled program admitted for the requested derivative consumer.
    Available(T),
    /// A Root response cannot be prepared; its scientific base solve remains usable.
    Unavailable(native::square_response::Withheld),
}
impl<T> ParametricPreparation<T> {
    fn available(&self) -> Option<&T> {
        match self {
            Self::Available(value) => Some(value),
            Self::Unavailable(_) => None,
        }
    }
}
/// The parametric program a sensitivity request differentiates, with the normalization of
/// its columns and each parameter's value.
#[derive(Clone, Debug)]
struct SensitivityProgram {
    program: Arc<ExecutableCase>,
    normalization: Normalization,
    parameters: Vec<(pse_ids::SemanticId, f64)>,
    reduced_hessian: bool,
    /// Keep the pinned factor in the worker's retained state for an advanced step (Plan 22
    /// Y5c2), charged to the job's allowance; set by [`PreparedSolve::retaining_factor`].
    retain: bool,
    /// Immutable original dependencies for a requested fresh Root predictor. Its actual
    /// point is filled only by the postsolve fresh derivative producer.
    root_source: Option<pse_model::strategy::SemanticProductKey>,
}
impl SensitivityProgram {
    /// The request for the backend: callbacks over `worker`, or the reason none could be
    /// built.
    fn request(
        &self,
        worker: pse_math::assembly::CaseWorker,
        values: &CaseValues,
    ) -> Result<native::kkt::Sensitivity, ProblemError> {
        let oracle = native::assembled::AlgebraicOracle::new(worker, values.clone())?
            .with_normalization(self.normalization.clone())?;
        Ok(native::kkt::Sensitivity {
            oracle: Box::new(oracle),
            parameters: self.parameters.clone(),
            reduced_hessian: self.reduced_hessian,
            retain: self.retain,
            source: self.root_source,
        })
    }
    /// Every quantity withheld because the parametric callbacks could not be built.
    fn withheld(&self, cause: ProblemError) -> native::kkt::Parametric {
        native::kkt::Parametric::withheld(
            self.parameters.iter().map(|(id, _)| *id).collect(),
            self.reduced_hessian,
            native::kkt::Withheld::Analysis(cause.into()),
        )
    }
}
#[derive(Clone, Debug)]
#[expect(
    clippy::large_enum_variant,
    reason = "one representation per prepared solve; algebraic metadata stays inline and is charged by size_of::<PreparedSolve>() without a separate heap allocation"
)]
enum Representation {
    Algebraic(AlgebraicCase),
    Conic {
        /// Normalized at preparation.
        problem: Arc<native::ConicProblem>,
        /// The submitted data a certificate is verified against.
        original: Arc<native::ConicProblem>,
        certificate: Arc<dyn QuadraticEvidence>,
    },
}
/// Immutable routing decision, semantic maps and representation. Native state is constructed later.
#[derive(Clone, Debug)]
pub struct PreparedSolve {
    representation: Representation,
    profile: SolverProfile,
    requested_hessian: HessianMode,
    numerics: Arc<ResolvedNumericalPolicy>,
    normalization: Normalization,
    tolerances: Tolerances,
    /// Stopping budgets resolved from `numerics`; never taken from user controls (F20).
    accuracy: ResolvedAccuracy,
    /// Identity of a goal-driven operational stopping refinement, separate from
    /// frozen scientific numerical/acceptance identity.
    work_precision: Option<pse_ids::ContentHash>,
    route: Route,
    snapshot: execution::Snapshot,
    compatibility: Option<Compatibility>,
    explicit_start: Option<WarmStart>,
    /// Actual original-screened proposal, retaining its producer and entry permissions.
    proposal_start: Option<super::prediction::Screened>,
    /// Already available source-bound mathematical products eligible for automatic use.
    automatic_products: Vec<PreparedRung>,
    automatic_owner: Option<Arc<pse_columnar::AllocationLease>>,
    route_decision: Option<routing::Decision>,
    /// Finite original-problem profiles resolved before entering the effectful driver.
    composition: Option<Arc<PreparedComposition>>,
    task_scope: Option<pse_kernels::ExecutionScope>,
    task_admission: Option<Arc<super::strategy::admission::TaskAdmission>>,
    /// One workflow point's ephemeral pending factor; ordinary preparation owns none.
    point_accuracy: Option<Arc<kkt_accuracy::PointAccuracySink>>,
    selected_outputs: Option<Arc<output_program::SelectedOutputProgram>>,
    #[cfg(feature = "solver-kinsol")]
    causal_supplier: Option<Arc<dyn causal::CausalSupplier>>,
    pool: Arc<dyn datafusion::execution::memory_pool::MemoryPool>,
    _owner: Arc<pse_columnar::AllocationLease>,
}
#[derive(Clone, Debug)]
pub(crate) struct PreparedComposition {
    pub(crate) declaration: pse_model::strategy::NumericalStrategy,
    pub(crate) rungs: Vec<PreparedRung>,
    _owner: Arc<pse_columnar::AllocationLease>,
}
/// An immutable owner-issued operation beside the resolver's cheap applicability facts.
/// The index selected by the resolver is consumed from this inventory, never rebound.
#[derive(Clone, Debug)]
pub(crate) struct AutomaticOperation {
    pub(crate) candidate: super::strategy::AutoCandidate,
    binding: AutomaticBinding,
}
#[derive(Clone, Debug)]
enum AutomaticBinding {
    #[cfg(feature = "solver-kinsol")]
    Causal {
        original: Box<PreparedSolve>,
        supplier: Arc<dyn causal::CausalSupplier>,
    },
    #[cfg(feature = "solver-kinsol")]
    Blocks(Box<PreparedSolve>),
    Original(Box<PreparedSolve>),
    Prepared(Box<PreparedRung>),
    NativeRoute {
        original: Box<PreparedSolve>,
        backend: Backend,
    },
    #[cfg(feature = "solver-pounce")]
    NativeProfile {
        original: Box<PreparedSolve>,
        profile: Box<SolverProfile>,
        perturbation: Option<native::pounce::StartPerturbation>,
    },
    Reduced {
        original: Box<PreparedSolve>,
        accuracy: native::derived::ReconstructionAccuracy,
        profile: Box<SolverProfile>,
    },
}
/// Candidate completeness only; the existing reduced-request binder remains the
/// authority for actual named rows, bound inputs, output maps and dependency order.
fn selected_suppliers_cover(
    columns: &[pse_ids::SemanticId],
    providers: &BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
) -> bool {
    let supplied: std::collections::BTreeSet<_> = providers
        .values()
        .filter_map(|registration| {
            registration.source::<pse_math::implicit::reconstruction::ReconstructionFactory>()
        })
        .filter(|factory| factory.supports_reconstruction())
        .flat_map(|factory| factory.spec().outputs.iter().map(|port| port.id))
        .collect();
    !columns.is_empty() && columns.iter().all(|id| supplied.contains(id))
}
impl PreparedSolve {
    /// Admit already prepared source-bound products to the automatic inventory.
    /// The ordinary composition binder checks original meaning, actual profiles and scope.
    /// # Errors
    /// A product belongs to another original source, numerical policy, backend or task.
    pub fn with_automatic_products(
        mut self,
        mut products: Vec<PreparedRung>,
    ) -> Result<Self, ProblemError> {
        if self.composition.is_some() || !self.automatic_products.is_empty() {
            return Err(ProblemError::Contract(
                "automatic products must bind once before composition".into(),
            ));
        }
        for product in &products {
            if let SolverSelection::Explicit(backend) = self.profile.selection
                && product.backend().is_some_and(|actual| actual != backend)
            {
                return Err(ProblemError::Contract(
                    "automatic product changes explicitly selected backend".into(),
                ));
            }
        }
        let request = self.profile.composition.clone();
        products.push(self.clone().into());
        let mut declaration = self.numerical_strategy();
        declaration.branch = request.branch;
        declaration.start.recovery = request.recovery.clone();
        declaration.limits.attempts = u64::try_from(products.len())
            .map_err(|_| ProblemError::memory("automatic product extent"))?;
        declaration.mechanisms = products
            .iter()
            .map(|product| {
                let kind = match product {
                    PreparedRung::Derived(p) => p.mechanism(),
                    PreparedRung::Surrogate(_) => pse_model::strategy::MechanismKind::Surrogate,
                    PreparedRung::Path { .. } => pse_model::strategy::MechanismKind::Continuation,
                    PreparedRung::Multistart(_) => pse_model::strategy::MechanismKind::Multistart,
                    #[cfg(feature = "solver-petsc")]
                    PreparedRung::Petsc(p) => p.mechanism(),
                    #[cfg(feature = "solver-kinsol")]
                    PreparedRung::Blocks(_) => pse_model::strategy::MechanismKind::Block,
                    #[cfg(feature = "solver-kinsol")]
                    PreparedRung::Causal(_) => pse_model::strategy::MechanismKind::MapsAnderson,
                    PreparedRung::Original(_) => pse_model::strategy::MechanismKind::Direct,
                };
                Ok(pse_model::strategy::Mechanism {
                    kind,
                    position: pse_model::strategy::Position::Execution,
                    profile: product
                        .backend()
                        .map(|backend| {
                            product
                                .strategy_profile()
                                .map(|key| pse_model::strategy::ProfileRef { backend, key })
                        })
                        .transpose()?,
                    required: false,
                    support: Vec::new(),
                    operation: product.operation_contract()?,
                    limits: declaration.limits,
                    starts: request.recovery.clone(),
                    transitions: vec![
                        pse_model::strategy::Transition::Continue,
                        pse_model::strategy::Transition::Finish,
                        pse_model::strategy::Transition::Stop,
                    ],
                })
            })
            .collect::<Result<Vec<_>, ProblemError>>()?;
        let checked = self.clone().with_strategy(declaration, products)?;
        let composition = checked
            .composition
            .ok_or_else(|| ProblemError::Internal("automatic product admission absent".into()))?;
        self.automatic_products = composition.rungs[..composition.rungs.len() - 1].to_vec();
        self.automatic_owner = Some(composition._owner.clone());
        self.task_scope = checked.task_scope;
        Ok(self)
    }
    /// Normalized output resolution of actual reconstructed coordinates, independent
    /// of their bound-violation acceptance and of KKT stationarity.
    pub(crate) fn operational_point_allowance(&self) -> Result<f64, ProblemError> {
        let Representation::Algebraic(source) = &self.representation else {
            return Err(ProblemError::unsupported(
                "reconstructed-output context requires algebraic coordinates",
            ));
        };
        let quantities = &source.prepared.prepared.quantities;
        let mut allowance = f64::INFINITY;
        for (id, scale) in source
            .prepared
            .prepared
            .plan
            .columns()
            .iter()
            .zip(&self.normalization.variables)
        {
            let target = self
                .numerics
                .targets
                .iter()
                .find(|t| {
                    t.id == *id && t.kind == pse_model::generated::enums::NumericalTarget::Variable
                })
                .ok_or_else(|| {
                    ProblemError::Contract("operational output context lacks coordinate".into())
                })?;
            let context = match &target.engineering {
                Some(context) => context.clone(),
                None => pse_math::numerics::operational_output_context(
                    quantities,
                    &pse_math::numerics::TargetSpec {
                        id: target.id,
                        kind: target.kind,
                        quantity: target.quantity.into(),
                        unit: target.unit.into(),
                        integer: false,
                        declared_tolerance: None,
                    },
                    &self.numerics.policy,
                )
                .map_err(ProblemError::from)?,
            };
            // Named engineering outputs derive separate consuming demands from
            // their actual influence; this mandatory context also applies without goals.
            allowance = allowance.min(context.budget / scale);
        }
        if !allowance.is_finite() || allowance <= 0. {
            return Err(ProblemError::Contract(
                "operational output allowance is not finite positive".into(),
            ));
        }
        Ok(allowance)
    }
    fn automatic_reconstruction_accuracy(&self) -> Option<native::derived::ReconstructionAccuracy> {
        if let Some(accuracy) = self.profile.reconstruction {
            return Some(accuracy);
        }
        let Representation::Algebraic(source) = &self.representation else {
            return None;
        };
        if !source.providers.values().any(|registration| {
            registration
                .source::<pse_math::implicit::reconstruction::ReconstructionFactory>()
                .is_some_and(|factory| factory.supports_reconstruction())
        }) {
            return None;
        }
        let point = self.operational_point_allowance().ok()?;
        let rounds = usize::try_from(self.profile.controls.iterations).ok()?;
        let proof_cells = self
            .profile
            .composition
            .limits
            .and_then(|limits| limits.proof_steps)
            .unwrap_or(u64::from(self.profile.controls.iterations));
        if proof_cells == 0 {
            return None;
        }
        Some(native::derived::ReconstructionAccuracy {
            point,
            action: self.numerics.policy.supplier_action_accuracy,
            class: pse_model::strategy::AccuracyClass::Certified,
            refinement: pse_math::derived::RefinementLimits {
                rounds,
                proof_cells,
            },
        })
    }
    #[cfg(feature = "solver-kinsol")]
    pub(crate) fn with_causal_supplier(
        mut self,
        supplier: Arc<dyn causal::CausalSupplier>,
    ) -> Self {
        self.causal_supplier = Some(supplier);
        self
    }
    #[cfg(feature = "solver-kinsol")]
    fn applicable_causal_supplier(&self) -> Option<&Arc<dyn causal::CausalSupplier>> {
        (self.profile.controls.start == StartPolicy::NoPriorStart
            && self.backend() == Some(Backend::Kinsol)
            && self.profile.sensitivity.is_none()
            && self.profile.intent == SolveIntent::Root
            && self.profile.controls.threads == 1
            && matches!(self.profile.backend, BackendSettings::Default))
        .then_some(self.causal_supplier.as_ref())
        .flatten()
    }
    pub(crate) fn controls(&self) -> &Controls {
        &self.profile.controls
    }
    pub(crate) fn inclusive_work(&self, outcome: &Outcome) -> pse_model::strategy::WorkObservation {
        let mut work = super::strategy::work(outcome);
        if let Representation::Algebraic(source) = &self.representation
            && (!source.providers.is_empty() || source.sensitivity.is_some())
        {
            // The component report retains actual native counters. Nested provider
            // and response work does not yet have complete aggregate observations.
            work.evaluations = None;
            work.iterations = None;
            work.factorizations = None;
            work.proof_steps = None;
        }
        work
    }
    /// Complete accounting is a property of the actual frozen representation and
    /// adapter, never inferred from a report's partial native counters.
    pub(crate) fn work_admitted(
        &self,
        limits: pse_model::strategy::WorkLimits,
        scope: &pse_kernels::ExecutionScope,
        admission: Arc<super::strategy::admission::TaskAdmission>,
    ) -> bool {
        let Representation::Algebraic(source) = &self.representation else {
            return false;
        };
        // Opaque/nested providers own additional work without this callback hook.
        if !source.providers.is_empty() || self.profile.sensitivity.is_some() {
            return false;
        }
        if self.route == Route::Constant {
            return WorkCoverage {
                evaluations: true,
                iterations: true,
                factorizations: true,
                proof_steps: true,
            }
            .covers(limits);
        }
        let Some(backend) = self.backend() else {
            return false;
        };
        let adapter = execution::adapter(backend);
        if !matches!(
            adapter.representation(),
            execution::Representation::Nlp | execution::Representation::Roots
        ) {
            return false;
        }
        // Library preprocessing has no primitive proof/work admission contract.
        if !matches!(self.profile.presolve, native::presolve::Policy::Off) {
            return false;
        }
        let Ok(mut execution) =
            Execution::within(scope.cancellation().clone(), self.controls(), scope.clone())
        else {
            return false;
        };
        execution.work_admission = Some(admission);
        adapter.work_coverage(&execution).covers(limits)
    }
    /// Finite owned catalog, not an enlarged native iteration or fixture budget.
    pub(crate) fn automatic_attempt_capacity(
        &self,
        cancel: &Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<u64, ProblemError> {
        let count = 1usize
            .checked_add(2 * usize::from(self.automatic_reconstruction_accuracy().is_some()))
            .ok_or_else(|| ProblemError::memory("automatic catalog extent"))?;
        let count = self
            .automatic_products
            .iter()
            .try_fold(count, |count, product| {
                let allowance = match product {
                    PreparedRung::Path { prepared, .. } => prepared
                        .attempt_capacity()
                        .map_err(MathRuntimeError::into_problem)?
                        .checked_add(1)
                        .ok_or_else(|| ProblemError::memory("automatic path correction extent"))?,
                    // Each supplied phase has one invocation and its original correction.
                    _ => 2,
                };
                count
                    .checked_add(allowance)
                    .ok_or_else(|| ProblemError::memory("automatic supplied catalog extent"))
            })?;
        #[cfg(feature = "solver-kinsol")]
        let count = count
            .checked_add(
                2 * usize::from(
                    self.automatic_block_count(cancel)
                        .map_err(MathRuntimeError::into_problem)?
                        > 0,
                ),
            )
            .ok_or_else(|| ProblemError::memory("automatic block catalog extent"))?;
        #[cfg(feature = "solver-kinsol")]
        let count = count
            .checked_add(2 * usize::from(self.applicable_causal_supplier().is_some()))
            .ok_or_else(|| ProblemError::memory("automatic causal catalog extent"))?;
        #[cfg(not(feature = "solver-kinsol"))]
        let _ = cancel;
        let count = count
            .checked_add(self.automatic_backend_alternatives().len())
            .ok_or_else(|| ProblemError::memory("automatic backend catalog extent"))?;
        #[cfg(feature = "solver-pounce")]
        let count = if let Some((profile, settings)) = self.pounce_recovery_baseline()? {
            count
                .checked_add(
                    native::pounce::second_opinion_capacity(
                        &profile.controls,
                        &settings,
                        self.profile
                            .composition
                            .recovery
                            .contains(&pse_model::strategy::StartOrigin::Auxiliary),
                    )
                    .checked_mul(2)
                    .ok_or_else(|| ProblemError::memory("automatic recovery catalog extent"))?,
                )
                .ok_or_else(|| ProblemError::memory("automatic recovery catalog extent"))?
        } else {
            count
        };
        u64::try_from(count).map_err(|_| ProblemError::memory("automatic catalog extent"))
    }
    /// Produce descriptions only. Optional artifacts are prepared after one decision.
    pub(crate) fn automatic_operations(
        &self,
        last: Option<&SolveReport>,
        cancel: &Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<Vec<AutomaticOperation>, MathRuntimeError> {
        use pse_model::strategy::{MechanismKind, StartOrigin};
        if cancel.load(std::sync::atomic::Ordering::Acquire) {
            return Err(MathRuntimeError::Cancelled);
        }
        let start = self.entry_origin(false);
        let original_request = self.request_identity()?.as_id();
        let candidate = |kind: MechanismKind,
                         replacement,
                         profile: &SolverProfile|
         -> Result<super::strategy::AutoCandidate, ProblemError> {
            let mut binding = FramedHasher::new(pse_ids::Frame::DerivedBindingV2);
            binding
                .hash(&original_request)
                .hash(&profile_key(profile)?.as_id())
                .str(kind.as_str());
            Ok(super::strategy::AutoCandidate {
                identity: binding.finish_hash(),
                kind,
                start: if replacement {
                    StartOrigin::Auxiliary
                } else {
                    start
                },
                replacement,
                support: std::collections::BTreeSet::new(),
                reservation: None,
                prepared: kind == MechanismKind::Direct,
            })
        };
        if last.is_none() {
            let mut operations = Vec::new();
            for rung in &self.automatic_products {
                let kind = match rung {
                    PreparedRung::Derived(p) => p.mechanism(),
                    PreparedRung::Surrogate(_) => MechanismKind::Surrogate,
                    PreparedRung::Path { .. } => MechanismKind::Continuation,
                    PreparedRung::Multistart(_) => MechanismKind::Multistart,
                    #[cfg(feature = "solver-petsc")]
                    PreparedRung::Petsc(p) => p.mechanism(),
                    #[cfg(feature = "solver-kinsol")]
                    PreparedRung::Blocks(_) => MechanismKind::Block,
                    #[cfg(feature = "solver-kinsol")]
                    PreparedRung::Causal(_) => MechanismKind::MapsAnderson,
                    PreparedRung::Original(_) => MechanismKind::Direct,
                };
                let mut description = candidate(kind, false, &self.profile)?;
                description.identity = rung.request_identity()?.as_id();
                description.prepared = true;
                operations.push(AutomaticOperation {
                    candidate: description,
                    binding: AutomaticBinding::Prepared(Box::new(rung.clone())),
                });
            }
            #[cfg(feature = "solver-kinsol")]
            if let Some(supplier) = self.applicable_causal_supplier() {
                let mut description = candidate(MechanismKind::MapsAnderson, false, &self.profile)?;
                let mut identity = FramedHasher::new(pse_ids::Frame::DerivedBindingV2);
                identity.hash(&description.identity).hash(&supplier.key());
                description.identity = identity.finish_hash();
                operations.push(AutomaticOperation {
                    candidate: description,
                    binding: AutomaticBinding::Causal {
                        original: Box::new(self.clone()),
                        supplier: supplier.clone(),
                    },
                });
            }
            if let Some(accuracy) = self.automatic_reconstruction_accuracy()
                && let Representation::Algebraic(source) = &self.representation
            {
                let complete = selected_suppliers_cover(
                    source.prepared.prepared.plan.columns(),
                    &source.providers,
                );
                let adapter = execution::LINKED
                    .adapters()
                    .filter(|adapter| {
                        adapter.linked()
                            && adapter.representation() == execution::Representation::Nlp
                            && match self.profile.selection {
                                SolverSelection::Auto => true,
                                SolverSelection::Explicit(backend) => adapter.backend() == backend,
                            }
                            && self
                                .profile
                                .backend
                                .backend()
                                .is_none_or(|backend| adapter.backend() == backend)
                    })
                    .min_by_key(|adapter| adapter.automatic().unwrap_or(u8::MAX));
                // A complete reconstruction has no outer unknowns and needs no NLP backend.
                // The actual binder still checks every named supplier, dependency and row.
                if complete || adapter.is_some() {
                    let mut profile = self.profile.clone();
                    if complete
                        && matches!(profile.selection, SolverSelection::Auto)
                        && let Some(backend) = self.backend()
                    {
                        profile.selection = SolverSelection::Explicit(backend);
                    }
                    if !complete && let Some(adapter) = adapter {
                        profile.selection = SolverSelection::Explicit(adapter.backend());
                        if self.requested_hessian == HessianMode::Auto {
                            profile.controls.hessian = HessianMode::LimitedMemory;
                        }
                    }
                    profile.sensitivity = None;
                    operations.push(AutomaticOperation {
                        candidate: candidate(MechanismKind::ReducedSpace, false, &profile)?,
                        binding: AutomaticBinding::Reduced {
                            original: Box::new(self.clone()),
                            accuracy,
                            profile: Box::new(profile),
                        },
                    });
                }
            }
            #[cfg(feature = "solver-kinsol")]
            if self.automatic_block_count(cancel)? > 1 {
                operations.push(AutomaticOperation {
                    candidate: candidate(MechanismKind::Block, false, &self.profile)?,
                    binding: AutomaticBinding::Blocks(Box::new(self.clone())),
                });
            }
            operations.push(AutomaticOperation {
                candidate: candidate(MechanismKind::Direct, false, &self.profile)?,
                binding: AutomaticBinding::Original(Box::new(self.clone())),
            });
            return Ok(operations);
        }
        let mut operations = Vec::new();
        #[cfg(feature = "solver-pounce")]
        if last.is_some_and(|report| report.backend == Backend::Pounce)
            && let Some((baseline, settings)) = self.pounce_recovery_baseline()?
        {
            let allow_replacement = self
                .profile
                .composition
                .recovery
                .contains(&StartOrigin::Auxiliary);
            let descriptions = native::pounce::second_opinion_profiles(
                &baseline.controls,
                &settings,
                last.ok_or_else(|| {
                    ProblemError::Internal("missing actual native observation".into())
                })?,
                allow_replacement,
            )?;
            operations.extend(
                descriptions
                    .into_iter()
                    .map(|description| {
                        let mut profile = baseline.clone();
                        profile.controls = description.controls;
                        profile.backend = BackendSettings::Pounce(description.settings);
                        Ok(AutomaticOperation {
                            candidate: candidate(
                                MechanismKind::NativeGlobalization,
                                description.replaces_start,
                                &profile,
                            )?,
                            binding: AutomaticBinding::NativeProfile {
                                original: Box::new(self.clone()),
                                profile: Box::new(profile),
                                perturbation: description.perturbation,
                            },
                        })
                    })
                    .collect::<Result<Vec<_>, ProblemError>>()?,
            );
        }
        operations.extend(self.automatic_native_route_operations()?);
        Ok(operations)
    }

    fn automatic_native_route_operations(&self) -> Result<Vec<AutomaticOperation>, ProblemError> {
        use pse_model::strategy::MechanismKind;
        let start = self.entry_origin(false);
        let original_request = self.request_identity()?.as_id();
        self.automatic_backend_alternatives()
            .into_iter()
            .map(|backend| {
                let profile = self.profile_for_automatic_backend(backend)?;
                let mut binding = FramedHasher::new(pse_ids::Frame::DerivedBindingV2);
                binding
                    .hash(&original_request)
                    .hash(&profile_key(&profile)?.as_id())
                    .str(MechanismKind::NativeGlobalization.as_str());
                Ok(AutomaticOperation {
                    candidate: super::strategy::AutoCandidate {
                        identity: binding.finish_hash(),
                        kind: MechanismKind::NativeGlobalization,
                        start,
                        replacement: false,
                        support: std::collections::BTreeSet::new(),
                        reservation: None,
                        prepared: false,
                    },
                    binding: AutomaticBinding::NativeRoute {
                        original: Box::new(self.clone()),
                        backend,
                    },
                })
            })
            .collect()
    }

    /// Other automatic adapters in this request's selected mathematical class, in the
    /// router's rank order. A pending adapter is a barrier: later candidates are not
    /// offered until its required evidence has been resolved.
    fn automatic_backend_alternatives(&self) -> Vec<Backend> {
        if !matches!(self.profile.selection, SolverSelection::Auto)
            || !matches!(self.profile.backend, BackendSettings::Default)
        {
            return Vec::new();
        }
        let Some(decision) = &self.route_decision else {
            return Vec::new();
        };
        let Some(current) = self.backend() else {
            return Vec::new();
        };
        if decision.selection != SolverSelection::Auto
            || decision.state != routing::AssessmentState::Ready
            || decision.selected != Some(Route::Native(current))
        {
            return Vec::new();
        }
        let Some(current_adapter) = execution::LINKED.get(current) else {
            return Vec::new();
        };
        let Some(class) = decision.classes.iter().find(|class| {
            current_adapter
                .capability()
                .automatic_classes
                .contains(class)
        }) else {
            return Vec::new();
        };
        let mut ranked = execution::LINKED
            .adapters()
            .filter(|adapter| adapter.capability().automatic_classes.contains(class))
            .filter_map(|adapter| adapter.automatic().map(|rank| (rank, adapter.backend())))
            .collect::<Vec<_>>();
        ranked.sort_by_key(|(rank, backend)| (*rank, backend.as_str()));
        let Some(current_index) = ranked.iter().position(|(_, backend)| *backend == current) else {
            return Vec::new();
        };
        // A higher-ranked unresolved candidate makes the retained selection stale or
        // incomplete. Never skip it in search of an apparently ready lower-ranked route.
        if ranked[..current_index].iter().any(|(_, backend)| {
            decision
                .eligibility
                .iter()
                .find(|entry| entry.backend == *backend)
                .is_some_and(|entry| entry.state != routing::AssessmentState::Refused)
        }) {
            return Vec::new();
        }
        let mut alternatives = Vec::new();
        for (_, backend) in ranked.into_iter().skip(current_index + 1) {
            let state = decision
                .eligibility
                .iter()
                .find(|entry| entry.backend == backend)
                .map(|entry| entry.state);
            match state {
                Some(routing::AssessmentState::Refused) => {}
                Some(routing::AssessmentState::Ready)
                | Some(routing::AssessmentState::SupportedPendingArtifacts) => {
                    alternatives.push(backend);
                }
                Some(routing::AssessmentState::PendingEvidence) | None => break,
            }
        }
        alternatives
    }

    fn profile_for_automatic_backend(
        &self,
        backend: Backend,
    ) -> Result<SolverProfile, ProblemError> {
        let Representation::Algebraic(source) = &self.representation else {
            return Err(ProblemError::Unsupported(
                "automatic native backend alternatives require algebraic source".into(),
            ));
        };
        let mut profile = self.profile.clone();
        profile.selection = SolverSelection::Explicit(backend);
        profile.backend = BackendSettings::Default
            .for_requirements(backend, &source.prepared.prepared.facts.requirements);
        Ok(profile)
    }

    #[cfg(feature = "solver-pounce")]
    fn pounce_recovery_baseline(
        &self,
    ) -> Result<Option<(SolverProfile, native::settings::pounce::Settings)>, ProblemError> {
        let profile = if self.backend() == Some(Backend::Pounce) {
            self.profile.clone()
        } else if self
            .automatic_backend_alternatives()
            .contains(&Backend::Pounce)
        {
            self.profile_for_automatic_backend(Backend::Pounce)?
        } else {
            return Ok(None);
        };
        let settings = match &profile.backend {
            BackendSettings::Default => native::settings::pounce::Settings::default(),
            BackendSettings::Pounce(settings) => settings.clone(),
            _ => {
                return Err(ProblemError::Contract(
                    "POUNCE recovery route settings differ".into(),
                ));
            }
        };
        Ok(Some((profile, settings)))
    }
}
impl MathService {
    /// Bind only the selected description under the existing mathematical/native owners.
    pub(crate) async fn prepare_automatic_operation(
        self: &Arc<Self>,
        operation: AutomaticOperation,
        scope: pse_kernels::ExecutionScope,
        cancel: &crate::CancelSource,
    ) -> Result<PreparedRung, MathRuntimeError> {
        scope.check().map_err(ProblemError::Provider)?;
        if cancel.token().is_cancelled() {
            return Err(MathRuntimeError::Cancelled);
        }
        let prepared = match operation.binding {
            #[cfg(feature = "solver-kinsol")]
            AutomaticBinding::Causal { original, supplier } => {
                return Ok(PreparedRung::Causal(
                    supplier.prepare(*original, scope, cancel).await?,
                ));
            }
            #[cfg(feature = "solver-kinsol")]
            AutomaticBinding::Blocks(original) => {
                return Ok(PreparedRung::Blocks(
                    self.prepare_blocks(*original, scope, cancel).await?,
                ));
            }
            AutomaticBinding::Prepared(rung) => return Ok(*rung),
            AutomaticBinding::Original(original) => (*original).within_task(scope)?,
            AutomaticBinding::Reduced {
                original,
                accuracy,
                profile,
            } => {
                let request = self
                    .automatic_reduced_request(&original, accuracy)?
                    .ok_or_else(|| {
                        ProblemError::Unsupported(
                            "selected providers do not establish a complete original reduction"
                                .into(),
                        )
                    })?;
                return Ok(self
                    .prepare_derived(*original, request, *profile, scope, cancel)
                    .await?
                    .into());
            }
            AutomaticBinding::NativeRoute { original, backend } => {
                let profile = original.profile_for_automatic_backend(backend)?;
                self.prepare_automatic_native_profile(&original, profile, scope)
                    .await?
            }
            #[cfg(feature = "solver-pounce")]
            AutomaticBinding::NativeProfile {
                original,
                profile,
                perturbation,
            } => {
                let Representation::Algebraic(source) = &original.representation else {
                    return Err(ProblemError::Unsupported(
                        "native profile recovery requires algebraic source".into(),
                    )
                    .into());
                };
                let mut rebound = self
                    .prepare_automatic_native_profile(&original, *profile, scope.clone())
                    .await?;
                if let Some(perturbation) = perturbation {
                    let point = original.source_start(None)?;
                    let declaration = source.prepared.prepared.plan.structure();
                    let ids = source.prepared.prepared.plan.columns().to_vec();
                    let bounds = ids
                        .iter()
                        .map(|id| {
                            declaration
                                .variables()
                                .iter()
                                .find(|v| v.port.id == *id)
                                .map(|v| {
                                    (
                                        v.lower.unwrap_or(f64::NEG_INFINITY),
                                        v.upper.unwrap_or(f64::INFINITY),
                                    )
                                })
                                .ok_or_else(|| {
                                    ProblemError::Contract(
                                        "perturbation original variable bound absent".into(),
                                    )
                                })
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let produced = native::pounce::perturb_start(&point, &bounds, perturbation)?;
                    let mut identity = FramedHasher::new(pse_ids::Frame::DerivedBindingV2);
                    identity
                        .hash(&original.request_identity()?.as_id())
                        .hash(&rebound.strategy_profile()?)
                        .u64(perturbation.seed)
                        .u64(perturbation.scale.to_bits())
                        .u64(produced.displacement.to_bits());
                    let proposal = super::prediction::Proposal::auxiliary(
                        ids,
                        produced.coordinates,
                        original.semantic_point_key(&point)?,
                        original.original_identity()?,
                        original.profile.composition.branch,
                        identity.finish_hash(),
                    )?;
                    let screened = self
                        .screen_start(
                            rebound.clone(),
                            proposal,
                            original.profile.composition.branch,
                            scope,
                            cancel,
                        )
                        .await?;
                    rebound = rebound.with_recovery_start(&screened)?;
                }
                rebound
            }
        };
        Ok(prepared.into())
    }

    /// Rebind an admitted native alternative against the same original model and task.
    /// Native warm payloads are adapter-specific, so an explicit entry primal is rebuilt
    /// through the newly admitted adapter rather than copied across backend boundaries.
    async fn prepare_automatic_native_profile(
        self: &Arc<Self>,
        original: &PreparedSolve,
        profile: SolverProfile,
        scope: pse_kernels::ExecutionScope,
    ) -> Result<PreparedSolve, MathRuntimeError> {
        let Representation::Algebraic(source) = &original.representation else {
            return Err(ProblemError::Unsupported(
                "native profile recovery requires algebraic source".into(),
            )
            .into());
        };
        if original
            .task_admission
            .as_ref()
            .is_some_and(|admission| !admission.matches_scope(&scope))
        {
            return Err(ProblemError::Contract(
                "automatic native alternative differs from its task admission".into(),
            )
            .into());
        }
        if original.proposal_start.as_ref().is_some_and(|proposal| {
            !Arc::ptr_eq(proposal.scope().cancellation(), scope.cancellation())
                || proposal.scope().deadline() != scope.deadline()
        }) {
            return Err(ProblemError::Contract(
                "automatic native alternative differs from its screened entry start scope".into(),
            )
            .into());
        }
        let mut rebound = self
            .prepare_resolved(
                source.prepared.clone(),
                source.values.clone(),
                source.providers.clone(),
                profile,
                original.numerics.clone(),
                BTreeMap::new(),
            )
            .await?;
        if let Some(seed) = &original.explicit_start {
            let primal = match &seed.payload {
                WarmPayload::Nlp { primal, .. } | WarmPayload::Root(primal) => primal,
                WarmPayload::Highs {
                    primal: Some(primal),
                    ..
                } => primal,
                WarmPayload::Highs { primal: None, .. } => {
                    return Err(ProblemError::Contract(
                        "explicit entry start has no source-coordinate primal".into(),
                    )
                    .into());
                }
            };
            let ids = source.prepared.prepared.plan.columns();
            if ids.len() != primal.len() {
                return Err(ProblemError::Contract(
                    "explicit entry start differs from the original coordinate inventory".into(),
                )
                .into());
            }
            let values = ids.iter().copied().zip(primal.iter().copied()).collect();
            rebound = rebound.with_primal_start(values)?;
        }
        if let Representation::Algebraic(rebound_source) = &mut rebound.representation {
            // The original parametric program remains the mathematical owner.
            rebound_source.sensitivity = source.sensitivity.clone();
        }
        rebound.proposal_start = original.proposal_start.clone();
        rebound.task_admission = original.task_admission.clone();
        rebound.point_accuracy = original.point_accuracy.clone();
        rebound.selected_outputs = original.selected_outputs.clone();
        rebound.requested_hessian = original.requested_hessian;
        Ok(rebound.within_task(scope)?)
    }
}
impl PreparedSolve {
    /// Preserve a caller's finite task deadline and cancellation through queued execution.
    /// # Errors
    /// An unbounded, expired or conflicting task scope is supplied.
    pub fn within_task(mut self, scope: pse_kernels::ExecutionScope) -> Result<Self, ProblemError> {
        scope.check().map_err(ProblemError::Provider)?;
        if scope.deadline().is_none() {
            return Err(ProblemError::Contract(
                "solve task requires a finite deadline".into(),
            ));
        }
        if self.composition.as_ref().is_some_and(|c| {
            c.rungs
                .iter()
                .filter_map(PreparedRung::task_scope)
                .any(|r| {
                    !Arc::ptr_eq(r.cancellation(), scope.cancellation())
                        || r.deadline() != scope.deadline()
                })
        }) {
            return Err(ProblemError::Contract(
                "solve task differs from its prepared family scope".into(),
            ));
        }
        self.task_scope = Some(scope);
        Ok(self)
    }
    pub(crate) fn task_scope(&self) -> Option<pse_kernels::ExecutionScope> {
        self.task_scope.clone()
    }
    pub(crate) fn source_start(
        &self,
        previous: Option<&Predecessor>,
    ) -> Result<Vec<f64>, ProblemError> {
        if let Some(start) = &self.proposal_start {
            return Ok(start.proposal().values().map(|(_, value)| value).collect());
        }
        let seed = match self.profile.controls.start {
            StartPolicy::Explicit => self.explicit_start.as_ref(),
            StartPolicy::PreviousAccepted => previous.map(|p| &p.seed),
            StartPolicy::NoPriorStart => None,
        };
        if let Some(seed) = seed {
            match &seed.payload {
                WarmPayload::Root(x) | WarmPayload::Nlp { primal: x, .. } => return Ok(x.clone()),
                WarmPayload::Highs {
                    primal: Some(x), ..
                } => return Ok(x.clone()),
                WarmPayload::Highs { primal: None, .. } => {}
            }
        }
        match &self.representation {
            Representation::Algebraic(source) => source
                .prepared
                .prepared
                .plan
                .columns()
                .iter()
                .map(|id| {
                    source.values.scalars.get(id).copied().ok_or_else(|| {
                        ProblemError::Contract("original source coordinate missing".into())
                    })
                })
                .collect(),
            Representation::Conic { .. } => Err(ProblemError::Unsupported(
                "derived start requires original compiled algebraic coordinates".into(),
            )),
        }
    }
    /// Compose already admitted profiles for the same original problem and frozen policy.
    /// Every profile is explicit in the declaration; no trajectory failure changes routing.
    ///
    /// # Errors
    /// A declaration, profile, entry start, original binding or numerical policy differs.
    pub fn with_strategy(
        mut self,
        declaration: pse_model::strategy::NumericalStrategy,
        mut rungs: Vec<PreparedRung>,
    ) -> Result<Self, ProblemError> {
        use pse_model::strategy::ProfileRef;
        declaration
            .validate()
            .map_err(|e| ProblemError::Contract(e.to_string()))?;
        if declaration.start.policy != self.profile.controls.start
            || declaration.mechanisms.len() != rungs.len()
        {
            return Err(ProblemError::Contract(
                "prepared strategy must preserve entry policy and bind every mechanism".into(),
            ));
        }
        let original = self.original_identity()?;
        if !matches!(
            rungs.last(),
            Some(PreparedRung::Original(_) | PreparedRung::Multistart(_))
        ) {
            return Err(ProblemError::Contract(
                "a composed strategy must end with original correction and assessment".into(),
            ));
        }
        let mut task_scope = self.task_scope.clone();
        for (mechanism, rung) in declaration.mechanisms.iter().zip(&mut rungs) {
            if rung
                .operation_contract()?
                .outputs
                .iter()
                .any(|output| output.branch != declaration.branch)
            {
                return Err(ProblemError::Contract(
                    "prepared producer output branch differs from the original composition".into(),
                ));
            }
            if !rung.admits_mechanism(mechanism.kind) {
                return Err(ProblemError::Unsupported(
                    "this prepared original profile requires its mathematical mechanism producer"
                        .into(),
                ));
            }
            if let PreparedRung::Multistart(prepared) = rung
                && declaration.branch != prepared.branch()
            {
                return Err(ProblemError::Contract(
                    "multistart cannot replace the declared connected path".into(),
                ));
            }
            if matches!(rung, PreparedRung::Surrogate(_)) && declaration.branch.connected.is_some()
            {
                return Err(ProblemError::Contract(
                    "statistical starts do not establish connected path transport".into(),
                ));
            }
            if let PreparedRung::Path { prepared, start } = rung {
                prepared.origin_connected(start)?;
                if declaration.branch.connected.is_some() && declaration.branch != prepared.branch()
                {
                    return Err(ProblemError::Contract(
                        "declared connected path differs from its validated originating product"
                            .into(),
                    ));
                }
            }
            if let Some(scope) = rung.task_scope() {
                if task_scope.as_ref().is_some_and(|prior| {
                    !Arc::ptr_eq(prior.cancellation(), scope.cancellation())
                        || prior.deadline() != scope.deadline()
                }) {
                    return Err(ProblemError::Contract(
                        "composed families must retain one original task scope".into(),
                    ));
                }
                task_scope = Some(scope);
            }
            if rung.original_identity()? != original {
                return Err(ProblemError::Contract("strategy rung must preserve the original binding, bounds, objective and frozen accuracy".into()));
            }
            if rung.threads() != self.threads() {
                return Err(ProblemError::Contract(
                    "composed native profiles must use the task's admitted thread extent".into(),
                ));
            }
            let actual = match rung.backend() {
                Some(backend) => Some(ProfileRef {
                    backend,
                    key: rung.strategy_profile()?,
                }),
                None => None,
            };
            if actual != mechanism.profile {
                return Err(ProblemError::Contract(
                    "strategy rung profile must equal its admitted backend and settings identity"
                        .into(),
                ));
            }
            rung.clear_composition();
        }
        use pse_model::HeapUsage;
        let boxed_payloads = rungs.iter().try_fold(0usize, |bytes, rung| {
            bytes
                .checked_add(rung.boxed_payload_bytes())
                .ok_or_else(|| ProblemError::memory("prepared composition boxed payload extent"))
        })?;
        let bytes = size_of::<PreparedComposition>()
            .checked_add(declaration.heap_bytes())
            .and_then(|n| n.checked_add(rungs.capacity().checked_mul(size_of::<PreparedRung>())?))
            .and_then(|n| n.checked_add(boxed_payloads))
            .ok_or_else(|| ProblemError::memory("prepared composition extent"))?;
        let reservation =
            datafusion::execution::memory_pool::MemoryConsumer::new("math:prepared-composition")
                .register(&self.pool);
        reservation
            .try_grow(bytes)
            .map_err(|error| ProblemError::memory(error.to_string()))?;
        let owner = pse_columnar::AllocationLease::new(reservation);
        self.task_scope = task_scope;
        self.profile.composition.policy = pse_model::strategy::CompositionPolicy::Declared;
        self.composition = Some(Arc::new(PreparedComposition {
            declaration,
            rungs,
            _owner: owner,
        }));
        Ok(self)
    }
    /// Complete original physical state inventory, supplied by the compiled source owner.
    pub(crate) fn original_coordinates(&self) -> Result<Vec<pse_ids::SemanticId>, ProblemError> {
        match &self.representation {
            Representation::Algebraic(case) => Ok(case.prepared.prepared.plan.columns().to_vec()),
            Representation::Conic { .. } => Err(ProblemError::Unsupported(
                "compiled original coordinates required".into(),
            )),
        }
    }
    /// Original problem and final accuracy identity, independent of native profile/start.
    ///
    /// # Errors
    /// Canonical original data cannot be encoded.
    pub fn original_identity(&self) -> Result<pse_ids::ContentHash, ProblemError> {
        let mut h = FramedHasher::new(pse_ids::Frame::OriginalSolveContractV1);
        h.hash(&self.numerics.key).str(self.profile.intent.as_str());
        match &self.representation {
            Representation::Algebraic(case) => {
                h.str("algebraic")
                    .hash(&case.prepared.prepared.plan.structure().key());
                for (id, value) in &case.values.scalars {
                    h.id(id).f64(*value);
                }
                for (key, provider) in &case.providers {
                    h.hash(&key.0).hash(&provider.configuration_key());
                }
            }
            Representation::Conic { original, .. } => {
                h.str("conic").hash(&original.contract.identity);
                for variable in &original.contract.variables {
                    h.id(&variable.id).f64(variable.lower).f64(variable.upper);
                }
                for row in &original.contract.rows {
                    h.id(row);
                }
                h.hash(&native::conic::cone_key(&original.cones)?)
                    .f64(original.objective_constant);
                for matrix in [&original.quadratic, &original.constraints] {
                    h.u64(matrix.rows as u64).u64(matrix.columns as u64);
                    for index in &matrix.column_starts {
                        h.u64(*index as u64);
                    }
                    for index in &matrix.row_indices {
                        h.u64(*index as u64);
                    }
                    for value in &matrix.values {
                        h.f64(*value);
                    }
                }
                for value in &original.objective {
                    h.f64(*value);
                }
                for value in &original.rhs {
                    h.f64(*value);
                }
            }
        }
        Ok(h.finish_hash())
    }
    pub(crate) fn take_composition(&mut self) -> Option<Arc<PreparedComposition>> {
        self.composition.take()
    }
    pub(crate) fn composition_is_declared(&self) -> bool {
        self.composition.is_some()
    }
    pub(crate) fn entry_origin(&self, has_previous: bool) -> pse_model::strategy::StartOrigin {
        use pse_model::strategy::StartOrigin;
        if let Some(start) = &self.proposal_start {
            return start.proposal().origin();
        }
        match self.profile.controls.start {
            StartPolicy::Explicit => StartOrigin::Explicit,
            StartPolicy::PreviousAccepted if has_previous => StartOrigin::Accepted,
            _ => StartOrigin::Specification,
        }
    }
    /// Attach provider kernels prepared for the selected mandatory consumer demand.
    pub(crate) fn with_providers(
        mut self,
        providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
    ) -> Result<Self, ProblemError> {
        if let Representation::Algebraic(case) = &mut self.representation {
            case.providers = providers;
            if let Route::Native(backend) = self.route {
                self.compatibility = Some(compatibility(
                    &case.prepared.prepared.plan,
                    &case.values,
                    &self.profile,
                    &self.numerics,
                    backend,
                    &case.providers,
                    &self.snapshot,
                )?);
            }
        }
        Ok(self)
    }
    /// Kernel order consumed by selected callbacks, including an admitted auxiliary re-solve.
    pub fn required_order(&self) -> pse_kernels::DerivativeOrder {
        match &self.representation {
            Representation::Algebraic(case) => {
                let plan = &case.prepared.prepared.plan;
                let original = if plan.has_directional_actions() {
                    plan.order().max(pse_kernels::DerivativeOrder::First)
                } else {
                    plan.order()
                };
                case.pricing_case.as_ref().map_or(original, |pricing| {
                    original.max(pricing.executable.assembly.order())
                })
            }
            Representation::Conic { .. } => pse_kernels::DerivativeOrder::Value,
        }
    }

    /// Immutable compilation and normalization selected before attaching a seed.
    pub fn preparation_identity(&self) -> Result<pse_ids::ContentHash, ProblemError> {
        let mut h = FramedHasher::new(pse_ids::Frame::SolvePreparationV3);
        h.hash(&self.snapshot.identity())
            .hash(&self.strategy_profile()?)
            .hash(&self.numerics.key);
        match &self.representation {
            Representation::Algebraic(AlgebraicCase {
                prepared,
                providers,
                factorable,
                pricing_case,
                ..
            }) => {
                for provider in providers.values() {
                    h.hash(&provider.configuration_key());
                }
                // The projection's key covers its implicit definitions and envelopes.
                if let Some((program, _)) = factorable {
                    h.hash(&program.key);
                }
                if let Some(case) = pricing_case {
                    h.str("fixed-assignment-callbacks")
                        .hash(&case.executable.assembly.structure().key())
                        .u64(case.executable.assembly.order() as u64);
                    for artifact in case.prepared.compiled().artifacts.iter() {
                        h.hash(&artifact.key());
                    }
                }
                h.str("algebraic")
                    .hash(&prepared.compiled().plan.structure().key());
                for artifact in prepared.compiled().artifacts.iter() {
                    h.hash(&artifact.key());
                }
            }
            Representation::Conic { .. } => {
                h.str("conic");
            }
        }
        if let Some(c) = &self.compatibility {
            h.hash(&c.layout)
                .hash(&c.profile)
                .hash(&c.data)
                .str(c.backend.as_str());
        }
        let base = h.finish_hash();
        if let Some(composition) = &self.composition {
            let mut strategy = FramedHasher::new(pse_ids::Frame::SolveStrategyPreparationV3);
            strategy.hash(&base).hash(
                &composition
                    .declaration
                    .key()
                    .map_err(|e| ProblemError::Contract(e.to_string()))?,
            );
            for rung in &composition.rungs {
                strategy.hash(&rung.request_identity()?.as_id());
            }
            Ok(strategy.finish_hash())
        } else {
            Ok(base)
        }
    }
    /// The preparation a stored seed is keyed by (ADR-0112 Outcome 17): the compiled
    /// structure, its artifacts and providers, and the seed coordinates and backend. Numeric
    /// data, the start policy, native options and attempt limits are excluded, because a
    /// seed in source coordinates stays valid across them (F24); it is `None` for a constant
    /// evaluation, which consumes no seed.
    pub fn seed_preparation_identity(&self) -> Option<pse_ids::ContentHash> {
        let compatibility = self.compatibility.as_ref()?;
        let mut h = FramedHasher::new(pse_ids::Frame::SolveSeedPreparationV1);
        match &self.representation {
            Representation::Algebraic(AlgebraicCase {
                prepared,
                providers,
                ..
            }) => {
                for provider in providers.values() {
                    h.hash(&provider.configuration_key());
                }
                h.str("algebraic")
                    .hash(&prepared.compiled().plan.structure().key());
                for artifact in prepared.compiled().artifacts.iter() {
                    h.hash(&artifact.key());
                }
            }
            Representation::Conic { .. } => {
                h.str("conic");
            }
        }
        h.hash(&compatibility.layout)
            .str(compatibility.backend.as_str());
        Some(h.finish_hash())
    }
    /// Complete selected request, including explicit seed payload and compatibility data.
    pub fn request_identity(&self) -> Result<pse_ids::roles::LineageRequestHash, ProblemError> {
        let mut h = FramedHasher::new(if self.composition.is_some() {
            pse_ids::Frame::SolveStrategyRequestV3
        } else {
            pse_ids::Frame::SolveRequestV3
        });
        h.hash(&self.preparation_identity()?)
            .hash(&self.numerics.key)
            .str(self.requested_hessian.as_str());
        if let Some(compatibility) = &self.compatibility {
            h.bool(true)
                .hash(&compatibility.layout)
                .hash(&compatibility.profile)
                .hash(&compatibility.data)
                .str(compatibility.backend.as_str());
        } else {
            h.bool(false);
        }
        if let Some(start) = &self.explicit_start {
            h.bool(true).hash(&start.content_key());
        } else {
            h.bool(false);
        }
        if let Some(start) = &self.proposal_start {
            h.str("screened-proposal")
                .str(start.proposal().origin().as_str());
            let source =
                pse_ids::document::of(pse_ids::Frame::DerivedBindingV2, &start.proposal().source())
                    .map_err(|error| ProblemError::Contract(error.to_string()))?;
            h.hash(&source).hash(&start.proposal().target());
        }
        for product in &self.automatic_products {
            h.hash(&product.request_identity()?.as_id());
        }
        Ok(pse_ids::roles::LineageRequestHash::from(h.finish_hash()))
    }
    /// Construct a primal-only explicit seed in semantic source coordinates.
    pub fn with_primal_start(
        self,
        values: BTreeMap<pse_ids::SemanticId, f64>,
    ) -> Result<Self, ProblemError> {
        let compatibility = self.compatibility.clone().ok_or_else(|| {
            ProblemError::Contract("constant evaluation has no numerical start".into())
        })?;
        let ids: Vec<_> = match &self.representation {
            Representation::Algebraic(a) => a.prepared.prepared.plan.columns().to_vec(),
            Representation::Conic { problem, .. } => {
                problem.contract.variables.iter().map(|v| v.id).collect()
            }
        };
        if values.len() != ids.len()
            || ids
                .iter()
                .any(|id| values.get(id).is_none_or(|v| !v.is_finite()))
        {
            return Err(ProblemError::Contract(
                "explicit start must cover every original free coordinate exactly once".into(),
            ));
        }
        let primal = ids.iter().map(|id| values[id]).collect();
        let payload = execution::adapter(compatibility.backend).primal_start(primal)?;
        self.with_start(WarmStart {
            origin: None,
            compatibility,
            payload,
        })
    }
    /// All contextual alternatives, distinct from the linked adapter inventory.
    pub fn eligibility(&self) -> &[routing::Eligibility] {
        self.route_decision
            .as_ref()
            .map_or(&[], |d| d.eligibility.as_slice())
    }
    /// Retained request, capability and original structural facts.
    pub fn route_decision(&self) -> Option<&routing::Decision> {
        self.route_decision.as_ref()
    }
    /// Attach a compatible explicitly selected seed without changing allocation policy.
    pub fn with_start(mut self, seed: WarmStart) -> Result<Self, ProblemError> {
        let target = self.compatibility.as_ref().ok_or_else(|| {
            ProblemError::Contract("constant evaluation cannot consume a seed".into())
        })?;
        seed.validate(target)?;
        let (n, m) = match &self.representation {
            Representation::Algebraic(a) => (
                a.prepared.prepared.facts.variables,
                a.prepared.prepared.facts.rows,
            ),
            Representation::Conic { problem, .. } => (
                problem.contract.variables.len(),
                problem.contract.rows.len(),
            ),
        };
        seed.validate_shape(n, m)?;
        self.profile.controls.start = StartPolicy::Explicit;
        self.explicit_start = Some(seed);
        self.proposal_start = None;
        Ok(self)
    }
    /// Attach the availability of the profile's parametric sensitivity program:
    /// the case's plan with the requested parameters appended as coordinates
    /// ([`pse_math::assembly::CasePlan::parametric`]). Its parameter columns are normalized
    /// by the step's resolved numerical policy, which must resolve each parameter's
    /// coordinate scale. An unavailable Root response retains the base solve and later
    /// publishes its typed withholding cause; optimization requires an available program.
    ///
    /// # Errors
    /// No request in the profile, invalid parameter identities/values, a conic representation, a program whose columns are not
    /// the case's followed by the requested parameters, or an unresolved coordinate.
    pub fn with_sensitivity(
        mut self,
        preparation: ParametricPreparation<Arc<ExecutableCase>>,
    ) -> Result<Self, MathRuntimeError> {
        let request = self.profile.sensitivity.clone().ok_or_else(|| {
            ProblemError::Contract("no sensitivity request to attach a program to".into())
        })?;
        request.admit(self.profile.intent)?;
        let Representation::Algebraic(case) = &mut self.representation else {
            return Err(ProblemError::Contract(
                "parametric sensitivities need an algebraic case".into(),
            )
            .into());
        };
        let plan = &case.prepared.prepared.plan;
        let declared = plan
            .structure()
            .parameters()
            .iter()
            .map(|p| p.id)
            .collect::<std::collections::BTreeSet<_>>();
        if request.parameters.iter().any(|id| {
            !declared.contains(id)
                || plan.columns().contains(id)
                || case
                    .values
                    .scalars
                    .get(id)
                    .is_none_or(|value| !value.is_finite())
        }) {
            return Err(ProblemError::Contract(
                "sensitivity parameters must be declared fixed inputs with finite values".into(),
            )
            .into());
        }
        let program = match preparation {
            ParametricPreparation::Available(program) => program,
            ParametricPreparation::Unavailable(cause)
                if self.profile.intent == SolveIntent::Root =>
            {
                case.sensitivity = Some(ParametricPreparation::Unavailable(cause));
                return Ok(self);
            }
            ParametricPreparation::Unavailable(_) => {
                return Err(ProblemError::Contract(
                    "only a Root response may be withheld during parametric preparation".into(),
                )
                .into());
            }
        };
        let columns = program.assembly.columns();
        if columns.len() != plan.columns().len() + request.parameters.len()
            || columns[..plan.columns().len()] != *plan.columns()
            || columns[plan.columns().len()..] != *request.parameters
            || program.assembly.structure().key() != plan.structure().key()
        {
            return Err(ProblemError::Contract(
                "the parametric program is not the case's columns followed by the requested parameters".into(),
            )
            .into());
        }
        let rows: Vec<_> = plan.structure().rows().iter().map(|r| r.id).collect();
        let normalization = match Normalization::from_policy(&self.numerics, columns, &rows) {
            Ok(normalization) => normalization,
            Err(cause) if self.profile.intent == SolveIntent::Root => {
                case.sensitivity = Some(ParametricPreparation::Unavailable(
                    native::square_response::Withheld::Numerical(cause.to_string()),
                ));
                return Ok(self);
            }
            Err(cause) => return Err(cause.into()),
        };
        let parameters = request
            .parameters
            .iter()
            .map(|id| {
                case.values
                    .scalars
                    .get(id)
                    .map(|v| (*id, *v))
                    .ok_or_else(|| ProblemError::Contract(format!("no value for parameter {id}")))
            })
            .collect::<Result<_, _>>()?;
        case.sensitivity = Some(ParametricPreparation::Available(SensitivityProgram {
            program,
            normalization,
            parameters,
            reduced_hessian: request.reduced_hessian,
            retain: false,
            root_source: None,
        }));
        Ok(self)
    }
    /// Keep the attached sensitivity request's actual factor for related-target prediction.
    /// Optimization retains its KKT worker product. Root execution prepares a fresh sparse
    /// factor and parameter partials whose charged owner survives native-session teardown.
    ///
    /// # Errors
    /// No sensitivity program is attached.
    pub fn retaining_factor(mut self) -> Result<Self, MathRuntimeError> {
        let root_source = {
            let Representation::Algebraic(case) = &self.representation else {
                return Err(ProblemError::Contract(
                    "a Root predictor requires original algebraic coordinates".into(),
                )
                .into());
            };
            let point = case
                .prepared
                .compiled()
                .plan
                .columns()
                .iter()
                .map(|id| case.values.scalars[id])
                .collect::<Vec<_>>();
            Some(self.semantic_point_key(&point)?)
        };
        match &mut self.representation {
            Representation::Algebraic(AlgebraicCase {
                sensitivity: Some(ParametricPreparation::Available(program)),
                ..
            }) => {
                program.retain = true;
                program.root_source = root_source;
                Ok(self)
            }
            _ => Err(ProblemError::Contract(
                "an advanced step keeps the factor of an attached sensitivity request".into(),
            )
            .into()),
        }
    }
    /// Current quadratic evidence, including explicit inconclusive or numerical assessments.
    pub fn quadratic_evidence(&self) -> Option<&dyn QuadraticEvidence> {
        match &self.representation {
            Representation::Algebraic(a) => a.certificate.as_deref(),
            Representation::Conic { certificate, .. } => Some(certificate.as_ref()),
        }
    }

    /// Immutable physical requirements and their provenance.
    pub fn numerics(&self) -> &ResolvedNumericalPolicy {
        &self.numerics
    }
    /// Frozen original-coordinate acceptance budgets.
    pub fn tolerances(&self) -> &Tolerances {
        &self.tolerances
    }
    /// Native stopping budgets resolved from the numerical policy at preparation.
    pub fn accuracy(&self) -> &ResolvedAccuracy {
        &self.accuracy
    }
    /// Optional identity of a goal-driven operational stopping refinement.
    pub fn work_precision(&self) -> Option<pse_ids::ContentHash> {
        self.work_precision
    }
    /// The absolute accuracy of the objective value in original units: the continuous
    /// absolute gap budget at the objective's coordinate scale.
    pub fn objective_accuracy(&self) -> f64 {
        self.accuracy.gap_absolute * self.normalization.objective
    }
    /// Deterministic selected route, available for inspection before admission.
    pub fn route(&self) -> Route {
        self.route
    }
    /// Requested policy, retained separately from effective native settings and decisions.
    pub fn composition_request(&self) -> &pse_model::strategy::CompositionRequest {
        &self.profile.composition
    }
    /// One explicit minimal strategy when no composition was declared.
    pub fn numerical_strategy(&self) -> pse_model::strategy::NumericalStrategy {
        self.composition.as_ref().map_or_else(
            || super::strategy::direct(&self.profile.controls),
            |c| c.declaration.clone(),
        )
    }
    /// Existing finite wall allowance, carried across waiting, execution and assessment.
    pub(crate) fn time_limit(&self) -> std::time::Duration {
        self.profile.controls.time_limit
    }
    /// Identity of the complete effective native profile for a declared rung.
    ///
    /// # Errors
    /// The admitted profile cannot be encoded canonically.
    pub fn strategy_profile(&self) -> Result<pse_ids::ContentHash, ProblemError> {
        let profile = profile_key(&self.profile)?.as_id();
        match self.work_precision {
            Some(precision) => pse_ids::document::of(
                pse_ids::Frame::SolverProfileV5,
                &("goal-work-precision", profile, precision),
            )
            .map_err(|error| ProblemError::Internal(error.to_string())),
            None => Ok(profile),
        }
    }
    /// Semantic/numerical reuse identity, absent for all-fixed validation.
    pub fn compatibility(&self) -> Option<&Compatibility> {
        self.compatibility.as_ref()
    }
    /// Dependencies of an original physical semantic point. The completion owner grants
    /// its permission separately; a profile or native payload is not consumed by this key.
    /// # Errors
    /// Invalid point shape/values or original identity serialization.
    pub fn semantic_point_key(
        &self,
        point: &[f64],
    ) -> Result<pse_model::strategy::SemanticProductKey, ProblemError> {
        let Representation::Algebraic(case) = &self.representation else {
            return Err(ProblemError::Unsupported(
                "semantic point key requires original algebraic coordinates".into(),
            ));
        };
        let plan = &case.prepared.compiled().plan;
        if point.len() != plan.columns().len() || point.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::Contract(
                "semantic point shape or nonfinite values".into(),
            ));
        }
        let mut parameters = FramedHasher::new(pse_ids::Frame::ImplicitNumericalProductV1);
        parameters.str("original-bound-parameters");
        for (id, value) in &case.values.scalars {
            if !plan.columns().contains(id) {
                parameters.id(id).f64(*value);
            }
        }
        Ok(pse_model::strategy::SemanticProductKey {
            structure: plan.structure().key(),
            binding: self.original_identity()?,
            numerical_policy: Some(self.numerics.key),
            normalization: Some(self.normalization.key()),
            point: Some(native::square_response::point_key(point)),
            parameters: Some(parameters.finish_hash()),
            derivation: None,
            branch: None,
            accuracy: Some(self.numerics.key),
        })
    }
    /// Admit a related original target by its actual fixed dependencies, allowing changes
    /// only to the declared transported parameters and numerical starting point.
    pub(crate) fn related_target_parameters(
        &self,
        source: &Self,
        parameters: &[(pse_ids::SemanticId, f64)],
    ) -> Result<Vec<(pse_ids::SemanticId, f64)>, ProblemError> {
        let (Representation::Algebraic(target), Representation::Algebraic(base)) =
            (&self.representation, &source.representation)
        else {
            return Err(ProblemError::Unsupported(
                "prediction requires related original algebraic cases".into(),
            ));
        };
        let ids = parameters
            .iter()
            .map(|p| p.0)
            .collect::<std::collections::BTreeSet<_>>();
        let plan = &target.prepared.compiled().plan;
        let original = &base.prepared.compiled().plan;
        if self.profile.intent != source.profile.intent
            || self.numerics.key != source.numerics.key
            || self.normalization != source.normalization
            || plan.structure().key() != original.structure().key()
            || plan.columns() != original.columns()
            || target.providers.len() != base.providers.len()
            || target.providers.iter().any(|(id, provider)| {
                base.providers
                    .get(id)
                    .is_none_or(|p| p.configuration_key() != provider.configuration_key())
            })
            || target.values.scalars.len() != base.values.scalars.len()
            || target.values.scalars.iter().any(|(id, value)| {
                !plan.columns().contains(id)
                    && !ids.contains(id)
                    && base.values.scalars.get(id) != Some(value)
            })
            || parameters
                .iter()
                .any(|(id, value)| base.values.scalars.get(id) != Some(value))
        {
            return Err(ProblemError::Contract("predictor target changes an unconsumed parameter, authored structure, provider or frozen numerical policy".into()));
        }
        parameters
            .iter()
            .map(|(id, _)| {
                target
                    .values
                    .scalars
                    .get(id)
                    .copied()
                    .filter(|v| v.is_finite())
                    .map(|v| (*id, v))
                    .ok_or_else(|| {
                        ProblemError::Contract(
                            "prediction target parameter missing or nonfinite".into(),
                        )
                    })
            })
            .collect()
    }
    /// Retained result allowance of one attempt of this step.
    pub(crate) fn result_bytes(&self) -> Result<usize, MathRuntimeError> {
        if let Some(composition) = &self.composition {
            return composition.rungs.iter().try_fold(0usize, |maximum, rung| {
                Ok(maximum.max(rung.result_bytes()?))
            });
        }
        let (n, m) = match &self.representation {
            Representation::Algebraic(a) => (
                a.prepared.prepared.facts.variables,
                a.prepared.prepared.facts.rows,
            ),
            Representation::Conic { problem, .. } => (
                problem.contract.variables.len(),
                problem.contract.rows.len(),
            ),
        };
        let sources = match &self.representation {
            Representation::Algebraic(a) => a
                .prepared
                .prepared
                .plan
                .structure()
                .instances()
                .iter()
                .try_fold(0usize, |n, i| n.checked_add(i.contributions.len()))
                .ok_or(MathRuntimeError::Limit("source observation extent"))?,
            Representation::Conic { .. } => 0,
        };
        n.checked_add(m)
            .and_then(|v| v.checked_add(sources))
            .and_then(|v| v.checked_mul(512))
            .and_then(|v| v.checked_add(self.profile.controls.report_allowance().ok()?))
            .ok_or(MathRuntimeError::Limit("solve result allowance"))
    }
    /// The step's own starting point as a primal seed for its coordinates: what a later
    /// stage of a conditional block receives from the block's committed predecessor.
    #[cfg(feature = "solver-kinsol")]
    pub(crate) fn primal_seed(&self) -> Result<WarmStart, ProblemError> {
        let compatibility = self.compatibility.clone().ok_or_else(|| {
            ProblemError::Contract("constant evaluation has no numerical start".into())
        })?;
        let Representation::Algebraic(a) = &self.representation else {
            return Err(ProblemError::Contract(
                "a cone step has no primal seed".into(),
            ));
        };
        let primal = a
            .prepared
            .prepared
            .plan
            .columns()
            .iter()
            .map(|id| {
                a.values
                    .scalars
                    .get(id)
                    .copied()
                    .ok_or_else(|| ProblemError::Contract("missing start coordinate".into()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(WarmStart {
            origin: None,
            payload: execution::adapter(compatibility.backend).primal_start(primal)?,
            compatibility,
        })
    }
    /// Native threads this step is admitted with.
    pub(crate) fn threads(&self) -> usize {
        self.profile.controls.threads
    }
    /// The foreign allowance this step reserves while it runs, beyond the deployment
    /// allowance its native session holds: the one its controls declare, else none. A
    /// prepared step, and a result that keeps it, reserve no foreign allowance.
    pub(crate) fn declared_foreign_bytes(&self) -> usize {
        self.composition.as_ref().map_or_else(
            || self.profile.controls.foreign_bytes.unwrap_or(0),
            |c| {
                c.rungs
                    .iter()
                    .map(PreparedRung::declared_foreign_bytes)
                    .max()
                    .unwrap_or(0)
            },
        )
    }
    /// The adapter whose scope this step's native state lives in.
    pub(crate) fn backend(&self) -> Option<Backend> {
        match self.route {
            Route::Native(backend) => Some(backend),
            Route::Constant => None,
        }
    }
}
/// Fresh complete original-model evaluation, independently of native components.
#[derive(Clone, Debug)]
pub struct ConstantReport {
    // Keep the independently owned report allocations without moving their heap bodies.
    components: Vec<Box<SolveReport>>,
    owner: Option<Arc<pse_columnar::AllocationLease>>,
    /// Authored objective if declared.
    pub objective: Option<f64>,
    /// Fresh constraint values in the complete original row order.
    pub observation: quality::Observation,
    /// Source-space quality without a fake native attempt.
    pub quality: Quality,
    /// Actual original free coordinates from a complete reconstruction or block schedule.
    /// Empty for an all-fixed source.
    pub coordinates: Vec<(pse_ids::SemanticId, f64)>,
    /// Producer-issued selected-root coverage, present only for complete reconstruction.
    pub(crate) certified_reconstruction: Option<Arc<derived::CertifiedReconstructionPoint>>,
    /// Direct evaluation work; absent counters retain unavailable supplier totals.
    pub work: WorkEvidence,
}
impl ConstantReport {
    /// Actual native component reports of a complete original structural schedule.
    pub fn component_reports(&self) -> impl ExactSizeIterator<Item = &SolveReport> {
        self.components.iter().map(Box::as_ref)
    }
}
/// A step has either an actual native attempt or direct constant evaluation.
#[derive(Clone, Debug)]
pub enum Outcome {
    /// Native attempt, including limited/failed exits.
    Native(Box<SolveReport>),
    /// All-fixed original evaluation.
    Constant(Box<ConstantReport>),
    /// A typed admission/execution failure before a native report became available.
    Rejected(Arc<MathRuntimeError>),
}
impl Outcome {
    /// Registry run state of this outcome: the one name of each kind of step result.
    pub const fn state(&self) -> pse_model::generated::enums::NativeRunState {
        use pse_model::generated::enums::NativeRunState;
        match self {
            Self::Native(_) => NativeRunState::Native,
            Self::Constant(_) => NativeRunState::ConstantEvaluation,
            Self::Rejected(_) => NativeRunState::Rejected,
        }
    }
    /// The native candidate-use decision of the workflow completion owner (§16.6,
    /// ADR-0106). Seeding, commits, homotopy, studies and publication consume it.
    pub(crate) fn candidate_use(
        &self,
        policy: &NumericalPolicy,
    ) -> crate::workflow::numerics::CandidateDecision {
        use crate::workflow::numerics;
        match self {
            Self::Rejected(_) => {
                numerics::refused(pse_model::generated::enums::CandidateRefusal::NoCandidate)
            }
            Self::Constant(r) => numerics::constant_use(&r.quality),
            Self::Native(r) => numerics::native_use(r, policy),
        }
    }
}
/// Owned result of one prepared step, retaining its result allowance through the last reader.
#[derive(Debug)]
pub struct StepReport {
    /// Identity minted once for this standalone submitted operation.
    pub run_id: pse_model::generated::identities::RunId,
    /// The step's native attempt, constant evaluation or typed refusal.
    pub outcome: Outcome,
    /// Declared versus actual numerical operations, including report-less failures.
    pub strategy: StrategyTrace,
    _owner: Arc<pse_columnar::AllocationLease>,
}
/// The output seed of an earlier accepted step, offered to a step whose start policy is
/// `PreviousAccepted` (§17.6).
#[derive(Clone, Debug)]
pub(crate) struct Predecessor {
    /// The accepted step, recorded in the receipt.
    pub attempt: usize,
    /// Its output seed.
    pub seed: WarmStart,
}
/// Dropping the handle requests cancellation; awaiting it witnesses native destruction and join.
pub struct SolveHandle<T = StepReport> {
    pub(super) cancel: FlightCancellation,
    pub(super) receiver: Option<tokio::sync::oneshot::Receiver<Result<T, MathRuntimeError>>>,
    pub(super) progress: Arc<Progress>,
}
impl<T> std::fmt::Debug for SolveHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SolveHandle").finish_non_exhaustive()
    }
}
impl<T> Drop for SolveHandle<T> {
    fn drop(&mut self) {
        if self.receiver.is_some() {
            self.cancel.cancel();
        }
    }
}
impl<T: Send + 'static> SolveHandle<T> {
    /// Supervise asynchronous native work under this handle's lifecycle: cancelling or
    /// dropping the handle cancels `work`'s source, and finishing waits until `work` has
    /// completed, including every native join it awaits.
    pub(crate) fn supervise<F>(
        progress: Arc<Progress>,
        work: impl FnOnce(crate::CancelSource) -> F,
    ) -> Self
    where
        F: Future<Output = Result<T, MathRuntimeError>> + Send + 'static,
    {
        let cancel = FlightCancellation::default();
        let control = cancel.clone();
        let source = crate::CancelSource::new();
        let operation = work(source.clone());
        let (sender, receiver) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            tokio::pin!(operation);
            let result = tokio::select! {
                result = &mut operation => result,
                () = control.cancelled() => {
                    source.cancel();
                    operation.await
                }
            };
            let _ = sender.send(result);
        });
        Self {
            cancel,
            receiver: Some(receiver),
            progress,
        }
    }
}
impl<T> SolveHandle<T> {
    /// Share cancellation with a public supervisor without moving native ownership.
    pub fn cancellation(&self) -> FlightCancellation {
        self.cancel.clone()
    }
    /// Shared bounded progress retained independently of a waiter.
    pub fn progress_source(&self) -> Arc<Progress> {
        self.progress.clone()
    }
    /// Request cancellation and retain the ability to await the terminal report.
    pub fn cancel(&self) {
        self.cancel.cancel();
    }
    /// Bounded owned progress; no native buffer crosses the thread boundary.
    pub fn progress(&self) -> (Vec<Event>, u64) {
        self.progress.snapshot()
    }
    /// Completion occurs after native teardown, thread-local destruction and join.
    pub async fn finish(mut self) -> Result<T, MathRuntimeError> {
        let receiver = self
            .receiver
            .as_mut()
            .ok_or_else(|| MathRuntimeError::Infrastructure("consumed solve handle".into()))?;
        let result = receiver
            .await
            .map_err(|_| MathRuntimeError::Infrastructure("lost solve supervisor".into()))?;
        self.receiver.take();
        result
    }
}
/// Admit a selected route's typed settings and controls through its adapter. Required
/// library preprocessing needs an NLP representation.
pub(crate) fn admit_profile(
    profile: &SolverProfile,
    route: Route,
    snapshot: &execution::Snapshot,
) -> Result<(), ProblemError> {
    let adapter = match route {
        Route::Native(backend) => Some(execution::adapter(backend)),
        Route::Constant => None,
    };
    if adapter.is_none_or(|a| a.representation() != execution::Representation::Nlp)
        && matches!(&profile.presolve, native::presolve::Policy::Explicit { required, .. } if !required.is_empty())
    {
        return Err(ProblemError::Unsupported("common NLP scales/required preprocessing need an NLP route; use the selected class's native controls".into()));
    }
    let Some(adapter) = adapter else {
        return Ok(());
    };
    if !execution::algebraic(adapter.representation()) || !adapter.linked() {
        return Err(ProblemError::Unavailable {
            backend: adapter.backend(),
            alternatives: vec![],
        });
    }
    adapter.admit_settings(&profile.backend, &profile.controls, snapshot)
}
/// The native session profile: every control and setting a retained native session
/// depends on. Controls and backend settings are identified through serde (F09); only the
/// per-attempt budgets and the sequencing policies, which every attempt re-applies, are
/// left out, so a new control field enters the identity without an edit here.
fn hash_session(h: &mut FramedHasher, p: &SolverProfile) -> Result<(), ProblemError> {
    h.hash(&p.presolve.key()).hash(&p.numerics.key());
    match p.convexity {
        ConvexityPolicy::Exact => {
            h.u64(0);
        }
        ConvexityPolicy::Numerical { absolute, relative } => {
            h.u64(1).f64(absolute).f64(relative);
        }
    }
    let session = Controls {
        time_limit: std::time::Duration::ZERO,
        iterations: 0,
        history: 0,
        reuse: ReusePolicy::Fresh,
        start: StartPolicy::NoPriorStart,
        foreign_bytes: None,
        ..p.controls.clone()
    };
    h.str(p.intent.as_str())
        .hash(&session.identity()?)
        .hash(&p.backend.identity()?);
    Ok(())
}
fn compatibility(
    plan: &pse_math::assembly::CasePlan,
    values: &CaseValues,
    p: &SolverProfile,
    numerics: &ResolvedNumericalPolicy,
    backend: Backend,
    providers: &BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
    snapshot: &execution::Snapshot,
) -> Result<Compatibility, ProblemError> {
    // Seed coordinates only: backend, objective sense, free variables, rows and
    // structure. Controls, settings and the numerical policy form the profile (F24).
    let mut layout = FramedHasher::new(pse_ids::Frame::SolverCoordinatesV1);
    layout
        .str(backend.as_str())
        .u64(plan.structure().objective().map_or(0, |o| {
            if o.sense == ObjectiveSense::Minimize {
                1
            } else {
                2
            }
        }));
    for v in plan.structure().variables().iter().filter(|v| !v.fixed) {
        layout
            .id(&v.port.id)
            .id(&v.port.quantity.as_id())
            .id(&v.port.unit.as_id())
            .u64(v.domain as u64);
    }
    for r in plan.structure().rows() {
        layout
            .id(&r.id)
            .id(&r.quantity.as_id())
            .bool(r.lower == r.upper);
    }
    layout.u64(plan.bodies().len() as u64);
    for key in plan.bodies().keys() {
        layout.hash(key);
    }
    for pattern in [plan.jacobian_pattern(), plan.hessian_pattern()] {
        layout.hash(&pse_math::sparse::pattern_key(pattern));
    }
    let mut profile = FramedHasher::new(pse_ids::Frame::SolverSessionV3);
    hash_session(&mut profile, p)?;
    profile.hash(&numerics.key).hash(&snapshot.identity());
    let mut data = FramedHasher::new(pse_ids::Frame::SolverDataV1);
    data.hash(&plan.structure().key());
    for provider in providers.values() {
        data.hash(&provider.configuration_key());
    }
    for (id, v) in &values.scalars {
        data.id(id).u64(v.to_bits());
    }
    Ok(Compatibility {
        layout: layout.finish_hash(),
        profile: profile.finish_hash(),
        data: data.finish_hash(),
        backend,
    })
}
impl MathService {
    /// Obtain intent-relevant class evidence through the original math owner.
    pub(crate) async fn discover_class(
        self: &Arc<Self>,
        prepared: Preparation,
        values: CaseValues,
        profile: &SolverProfile,
    ) -> Result<Preparation, MathRuntimeError> {
        let result = if routing::class_evidence_required(
            &prepared.prepared.facts,
            profile.intent,
            profile.selection,
        ) {
            let source = prepared.prepared.clone();
            let bound_values = values;
            let product = self
                .job_retained(
                    1,
                    self.policy.workspace_bytes,
                    FlightCancellation::default(),
                    move |flag| {
                        let product = source.prepare_class(&bound_values, &flag)?;
                        let bytes = product.retained_bytes();
                        Ok((product, bytes))
                    },
                )
                .await?;
            self.own_preparation(product)?
        } else {
            prepared
        };
        Ok(result)
    }

    /// Complete pure routing and required immutable artifact compilation before native admission.
    pub async fn prepare_solve(
        self: &Arc<Self>,
        prepared: Preparation,
        values: CaseValues,
        providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        profile: SolverProfile,
        numerical: NumericalInputs,
    ) -> Result<PreparedSolve, MathRuntimeError> {
        profile.controls.validate()?;
        let mut targets = prepared
            .prepared
            .plan
            .numerical_targets(&prepared.prepared.quantities)?;
        targets.extend(numerical.targets);
        let numerics = Arc::new(pse_math::numerics::resolve(
            &prepared.prepared.quantities,
            &targets,
            &numerical.declarations,
            &profile.numerics,
        )?);
        self.prepare_resolved(
            prepared,
            values,
            providers,
            profile,
            numerics,
            numerical.implicit,
        )
        .await
    }
    /// Admission of a bound case under an already resolved numerical policy. A conditional
    /// initialization block resolves its policy once for the whole case and passes it here.
    /// `implicit` holds the residual definitions a factorable route exports.
    ///
    /// Convexity is the preparation's fact (ADR-0121): routing reads it, and a convex
    /// quadratic coefficient objective carries its exact certificate. An explicit
    /// [`ConvexityPolicy::Numerical`] may qualify this request's coefficient objective as
    /// positive semidefinite when the fact does not; that evidence serves this request only
    /// and never becomes a fact.
    #[expect(
        clippy::too_many_lines,
        reason = "one admission binds routing, the representation each route needs and its identity"
    )]
    pub(crate) async fn prepare_resolved(
        self: &Arc<Self>,
        prepared: Preparation,
        values: CaseValues,
        providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        mut profile: SolverProfile,
        numerics: Arc<ResolvedNumericalPolicy>,
        implicit: BTreeMap<pse_kernels::ProviderKey, pse_math::factorable::ImplicitDefinition>,
    ) -> Result<PreparedSolve, MathRuntimeError> {
        profile.controls.validate()?;
        let requested_hessian = profile.controls.hessian;
        profile.resolve_curvature(prepared.prepared.plan.available_order());
        profile
            .composition
            .validate()
            .map_err(|error| ProblemError::Contract(error.to_string()))?;
        if let Some(accuracy) = profile.reconstruction {
            accuracy.validate()?;
        }
        // Per-solve overlays are separate from the shared compiler product.
        let prepared = self
            .discover_class(prepared, values.clone(), &profile)
            .await?;
        let structure = prepared.prepared.plan.structure();
        let entries = structure
            .variables()
            .len()
            .checked_add(structure.rows().len())
            .and_then(|n| n.checked_add(numerics.targets.len()))
            .ok_or(MathRuntimeError::Limit("solve metadata extent"))?;
        let bytes = entries
            .checked_mul(
                size_of::<pse_math::numerics::TargetSpec>()
                    + 8 * size_of::<f64>()
                    + (execution::LINKED.adapters().count() + 1)
                        * (size_of::<pse_structural::incidence::Constraint>()
                            + 3 * size_of::<pse_ids::SemanticId>()),
            )
            .and_then(|n| {
                n.checked_add(values.scalars.len() * size_of::<(pse_ids::SemanticId, f64)>())
            })
            .and_then(|n| n.checked_add(size_of::<PreparedSolve>()))
            .ok_or(MathRuntimeError::Limit("solve metadata extent"))?;
        let owner = self.reserve("math:prepared-solve", bytes)?;
        prepared
            .prepared
            .plan
            .structure()
            .validate_values(&values)?;
        if !prepared
            .prepared
            .presolve
            .matches(&prepared.prepared.plan, &values)
        {
            return Err(ProblemError::Contract("fixed/parameter values differ from compiler assumptions; rebind the prepared structure to these values".into()).into());
        }
        let f = &prepared.prepared.facts;
        let plan = &prepared.prepared.plan;
        let rows: Vec<_> = plan.structure().rows().iter().map(|r| r.id).collect();
        let normalization = Normalization::from_policy(&numerics, plan.columns(), &rows)?;
        let tolerances = Tolerances::from_policy(&numerics, plan.columns(), &rows)?;
        let accuracy = ResolvedAccuracy::resolve(&numerics.policy, &tolerances, &normalization)?;

        let mut certificate: Option<Arc<dyn QuadraticEvidence>> = f
            .convexity
            .convex_quadratic()
            .cloned()
            .map(|c| -> Arc<dyn QuadraticEvidence> { c });
        let mut numerical_psd = false;
        if let Some(c) = &prepared.prepared.coefficients {
            if prepared
                .prepared
                .coefficient_values
                .iter()
                .any(|(id, bits)| values.scalars.get(id).map(|v| v.to_bits()) != Some(*bits))
            {
                return Err(ProblemError::Contract("case values differ from the compiler's coefficient assumptions; prepare a new snapshot".into()).into());
            }
            if let (None, true, ConvexityPolicy::Numerical { absolute, relative }) =
                (&certificate, f.quadratic, profile.convexity)
            {
                let coefficients = c.clone();
                let coordinates = normalization.clone();
                let sign = plan.structure().objective().map_or(1.0, |o| o.sense.sign());
                let bytes = self.policy.worker_bytes;
                let evidence = self
                    .job(1, bytes, FlightCancellation::default(), move |flag| {
                        Ok(coefficients.numerical_convexity(
                            sign,
                            &coordinates.variables,
                            coordinates.objective,
                            absolute,
                            relative,
                            bytes,
                            &flag,
                        )?)
                    })
                    .await?;
                numerical_psd = evidence.accepted();
                certificate = Some(Arc::new(evidence));
            }
        }
        let snapshot = execution::Snapshot::observe(&execution::LINKED);
        // A complete selected producer performs original assessment directly, so a
        // refused outer native representation need not forbid preparing its source.
        // The original route decision remains refused and direct dispatch remains
        // guarded; this is only an actual linked profile context for the producer.
        let complete_context = (profile.composition.policy
            == pse_model::strategy::CompositionPolicy::Auto
            && profile.sensitivity.is_none()
            && plan.structure().native().is_empty()
            && plan.structure().requirements().is_empty()
            && selected_suppliers_cover(plan.columns(), &providers))
        .then(|| match profile.selection {
            SolverSelection::Explicit(backend) => {
                execution::adapter(backend).linked().then_some(backend)
            }
            SolverSelection::Auto => execution::LINKED
                .adapters()
                .filter(|adapter| {
                    adapter.linked()
                        && adapter.representation() == execution::Representation::Roots
                        && profile
                            .backend
                            .backend()
                            .is_none_or(|backend| backend == adapter.backend())
                })
                .min_by_key(|adapter| adapter.automatic().unwrap_or(u8::MAX))
                .map(|adapter| adapter.backend()),
        })
        .flatten();
        let mut prepared = prepared;
        let mut factorable = None;
        let mut recognized = None;
        let mut coefficient_cone = None;
        let mut case = None;
        let mut artifacts = Vec::new();
        let mut refusals = BTreeMap::new();
        let mut previous_demands = None;
        let mut decision = loop {
            let f = &prepared.prepared.facts;
            let plan = &prepared.prepared.plan;
            let coefficient_problem = prepared
                .prepared
                .coefficients
                .as_ref()
                .map(|coefficients| {
                    native::CoefficientProblem::from_plan(plan, coefficients.as_ref().clone())
                })
                .transpose()?;
            let oracle = native::assembled::contract(plan);
            let pending_classes =
                routing::pending_class_evidence(f, profile.intent, profile.selection);
            let requirements = routing::Requirements {
                table: &execution::LINKED,
                facts: f,
                intent: profile.intent,
                numerical_psd,
                least_squares: false,
                controls: &profile.controls,
                settings: &profile.backend,
                sensitivity: profile.sensitivity.is_some(),
                context: routing::Context {
                    snapshot: snapshot.clone(),
                    pending_classes: &pending_classes,
                    structure: None,
                    oracle: Some(&oracle),
                    guards: &prepared.prepared.presolve.signs,
                    budgets: Some(execution::Budgets {
                        tolerances: &tolerances,
                        normalization: &normalization,
                        accuracy: &accuracy,
                    }),
                    coefficients: coefficient_problem.as_ref(),
                    cone: recognized
                        .as_ref()
                        .map(
                            |(cone, proof, _): &(
                                Arc<native::conic::Recognized>,
                                Arc<pse_math::convexity::GramCertificate>,
                                Arc<pse_columnar::AllocationLease>,
                            )| routing::ConeEvidence {
                                problem: &cone.problem,
                                certificate: proof.as_ref(),
                            },
                        )
                        .or_else(|| {
                            coefficient_cone.as_ref().map(
                                |(cone, _): &(
                                    Arc<native::conic::Lowered>,
                                    Arc<pse_columnar::AllocationLease>,
                                )| routing::ConeEvidence {
                                    problem: &cone.problem,
                                    certificate: &cone.evidence,
                                },
                            )
                        }),
                    factorable: factorable.as_ref().map(
                        |(program, _): &(
                            Arc<pse_math::factorable::FactorableProgram>,
                            Arc<pse_columnar::AllocationLease>,
                        )| program.as_ref(),
                    ),
                    certificate: certificate.as_deref(),
                    prepared: &artifacts,
                    refusals: &refusals,
                },
            };
            let decision = requirements.bound_decision(
                profile.selection,
                plan.columns().to_vec(),
                plan.structure()
                    .rows()
                    .iter()
                    .map(|row| pse_structural::incidence::Constraint {
                        id: row.id,
                        lower: row.lower.is_finite().then_some(row.lower),
                        upper: row.upper.is_finite().then_some(row.upper),
                    })
                    .collect(),
                prepared.prepared.structure.clone(),
            )?;
            if decision.state == routing::AssessmentState::Ready {
                break decision;
            }
            if decision.state == routing::AssessmentState::Refused {
                if complete_context.is_some() {
                    break decision;
                }
                return Err(ProblemError::RouteRefused(Box::new(decision)).into());
            }
            let demands = (
                decision.pending_backend,
                decision.evidence.clone(),
                decision.artifacts.clone(),
            );
            if previous_demands.as_ref() == Some(&demands) {
                return Err(ProblemError::RouteRefused(Box::new(decision)).into());
            }
            previous_demands = Some(demands);
            let backend = decision
                .pending_backend
                .or(match decision.selected {
                    Some(Route::Native(backend)) => Some(backend),
                    _ => None,
                })
                .ok_or_else(|| ProblemError::RouteRefused(Box::new(decision.clone())))?;
            let representation = execution::adapter(backend).representation();
            if decision
                .evidence
                .contains(&routing::EvidenceDemand::Factorable)
            {
                let result: Result<_, MathRuntimeError> = async {
                    let plan = prepared.prepared.plan.clone();
                    let values = values.clone();
                    let limit = self.policy.worker_bytes / 256;
                    // Implicit blocks export their residuals; providers that declare an
                    // enforced envelope export as auxiliaries inside it (ADR-0105 §1).
                    let mut envelopes = BTreeMap::new();
                    for (key, registration) in &providers {
                        if let Some(envelope) = registration.envelope() {
                            envelopes.insert(*key, envelope.to_vec());
                        }
                    }
                    let request = pse_math::factorable::FactorableRequest {
                        implicit: implicit.clone(),
                        envelopes,
                        ..Default::default()
                    };
                    let program = self
                        .job(
                            1,
                            self.policy.worker_bytes,
                            FlightCancellation::default(),
                            move |flag| {
                                let program = plan
                                    .factorable_program(&values, &request, limit, &flag)
                                    .map_err(|e| match e {
                                        pse_math::factorable::FactorableError::Math(e) => {
                                            ProblemError::Math(e)
                                        }
                                        other => ProblemError::Unsupported(other.to_string()),
                                    })?;
                                Ok(program)
                            },
                        )
                        .await?;
                    let owner = self.reserve("math:factorable-program", program.bytes())?;

                    Ok((Arc::new(program), owner))
                }
                .await;
                match result {
                    Ok(product) => {
                        factorable = Some(product);
                    }
                    Err(cause) => {
                        let cause = cause.into_problem();
                        if matches!(
                            cause,
                            ProblemError::Unsupported(_) | ProblemError::Contract(_)
                        ) {
                            refusals.insert(backend, Arc::new(cause));
                            continue;
                        }
                        return Err(cause.into());
                    }
                }
            }
            if decision.evidence.contains(&routing::EvidenceDemand::Cone)
                || decision
                    .artifacts
                    .contains(&routing::ArtifactDemand::Representation(
                        execution::Representation::Cone,
                    ))
            {
                if let Some(problem) = &coefficient_problem {
                    let source = problem.clone();
                    let evidence = certificate.clone();
                    let (lowered, lease) = self
                        .job_retained(
                            1,
                            self.policy.workspace_bytes,
                            FlightCancellation::default(),
                            move |flag| {
                                if flag.load(std::sync::atomic::Ordering::Relaxed) {
                                    return Err(ProblemError::Cancelled.into());
                                }
                                let lowered = native::ConicProblem::from_coefficients(
                                    &source,
                                    evidence.as_deref(),
                                )?;
                                let cone = &lowered.problem;
                                let extent =
                                    [&cone.quadratic, &cone.constraints]
                                        .iter()
                                        .try_fold(
                                            size_of::<native::conic::Lowered>(),
                                            |bytes, matrix| {
                                                bytes
                                                    .checked_add(
                                                        matrix
                                                            .values
                                                            .capacity()
                                                            .checked_mul(size_of::<f64>())?,
                                                    )?
                                                    .checked_add(
                                                        (matrix.column_starts.capacity()
                                                            + matrix.row_indices.capacity())
                                                        .checked_mul(size_of::<usize>())?,
                                                    )
                                            },
                                        )
                                        .and_then(|bytes| {
                                            bytes.checked_add(
                                                (cone.objective.capacity() + cone.rhs.capacity())
                                                    .checked_mul(size_of::<f64>())?,
                                            )
                                        })
                                        .and_then(|bytes| {
                                            bytes.checked_add(
                                                cone.cones
                                                    .capacity()
                                                    .checked_mul(size_of::<native::conic::Cone>())?,
                                            )
                                        })
                                        .and_then(|bytes| {
                                            bytes.checked_add(
                                                cone.contract
                                                    .variables
                                                    .capacity()
                                                    .checked_mul(size_of::<native::Variable>())?,
                                            )
                                        })
                                        .and_then(|bytes| {
                                            bytes.checked_add(
                                                cone.contract
                                                    .rows
                                                    .capacity()
                                                    .checked_mul(size_of::<pse_ids::SemanticId>())?,
                                            )
                                        })
                                        .and_then(|bytes| {
                                            bytes.checked_add(lowered.rows.capacity().checked_mul(
                                                size_of::<native::conic::LoweredRow>(),
                                            )?)
                                        })
                                        .ok_or(MathRuntimeError::Limit(
                                            "coefficient cone extent",
                                        ))?;
                                Ok((lowered, extent))
                            },
                        )
                        .await?;
                    coefficient_cone = Some((Arc::new(lowered), lease));
                } else if f.convexity.cone() {
                    let plan = prepared.prepared.plan.clone();
                    let values = values.clone();
                    let limit = self.policy.worker_bytes / 256;
                    let intent = profile.intent;
                    let fact = f.convexity.clone();
                    let ((lowered, proof), owner) = self
                        .job_retained(
                            1,
                            self.policy.worker_bytes,
                            FlightCancellation::default(),
                            move |flag| {
                                let program = plan
                                    .factorable_program(
                                        &values,
                                        &pse_math::factorable::FactorableRequest::default(),
                                        limit,
                                        &flag,
                                    )
                                    .map_err(|e| match e {
                                        pse_math::factorable::FactorableError::Math(e) => {
                                            ProblemError::Math(e)
                                        }
                                        other => ProblemError::Unsupported(other.to_string()),
                                    })?;
                                let lowered = native::conic::lower(&program, &fact, intent, &flag)?;
                                let proof = native::conic::zero_certificate(
                                    lowered.problem.contract.variables.len(),
                                    &flag,
                                )?;
                                let bytes = lowered
                                    .bytes()
                                    .checked_add(
                                        size_of::<pse_math::convexity::GramCertificate>() + 256,
                                    )
                                    .ok_or(MathRuntimeError::Limit(
                                        "recognized cone evidence extent",
                                    ))?;
                                Ok(((lowered, proof), bytes))
                            },
                        )
                        .await?;

                    recognized = Some((Arc::new(lowered), Arc::new(proof), owner));
                }
            }
            let order = decision
                .artifacts
                .iter()
                .filter_map(|artifact| match artifact {
                    routing::ArtifactDemand::Derivatives(order) => Some(*order),
                    _ => None,
                })
                .max();
            let directional = decision
                .artifacts
                .contains(&routing::ArtifactDemand::JacobianProduct);
            if order.is_some() || directional {
                // Executable readiness belongs to the current prepared support.
                // A later directional/order upgrade must not keep an earlier
                // assembly ready merely because its representation is unchanged.
                case = None;
                artifacts.retain(|artifact| {
                    !matches!(
                        artifact,
                        routing::ArtifactDemand::Representation(
                            execution::Representation::Nlp | execution::Representation::Roots
                        )
                    )
                });
            }
            if let Some(order) = order {
                prepared = self
                    .prepare_order(prepared, order, FlightCancellation::default())
                    .await?;
            }
            if directional {
                prepared = self.prepare_directional_actions(prepared).await?;
                artifacts.push(routing::ArtifactDemand::JacobianProduct);
            }
            if decision
                .artifacts
                .contains(&routing::ArtifactDemand::Representation(representation))
            {
                match representation {
                    execution::Representation::Nlp | execution::Representation::Roots => {
                        case = Some(self.assemble(prepared.clone()).await?);
                    }
                    execution::Representation::Coefficients if coefficient_problem.is_some() => {}
                    execution::Representation::Cone
                        if recognized.is_some() || coefficient_cone.is_some() => {}
                    execution::Representation::Factorable if factorable.is_some() => {}
                    _ => return Err(ProblemError::RouteRefused(Box::new(decision)).into()),
                }
                artifacts.push(routing::ArtifactDemand::Representation(representation));
            }
        };
        // Every escaping original witness keeps the admission's metadata allocation alive.
        for assessment in decision
            .eligibility
            .iter_mut()
            .filter_map(|entry| entry.structure.as_mut())
            .chain(decision.structure.iter_mut())
        {
            assessment.witness = assessment.witness.clone().with_owner(owner.clone());
        }
        let deferred = decision.state == routing::AssessmentState::Refused;
        let route = if deferred {
            Route::Native(
                complete_context
                    .ok_or_else(|| ProblemError::RouteRefused(Box::new(decision.clone())))?,
            )
        } else {
            decision.route()?
        };
        let f = &prepared.prepared.facts;
        let profile = match route {
            Route::Native(backend) => SolverProfile {
                backend: profile.backend.for_requirements(backend, &f.requirements),
                ..profile
            },
            Route::Constant => profile,
        };
        admit_profile(&profile, route, &snapshot)?;
        let stamp = match route {
            Route::Constant => None,
            Route::Native(backend) => Some(compatibility(
                &prepared.prepared.plan,
                &values,
                &profile,
                &numerics,
                backend,
                &providers,
                &snapshot,
            )?),
        };
        let case = match case {
            Some(case) => Some(case),
            None => Some(self.assemble(prepared.clone()).await?),
        };
        let pricing_case = if let Some((program, _)) = &factorable {
            let program = program.clone();
            let intent = profile.intent;
            let controls = profile.controls.clone();
            let order = self
                .job(
                    1,
                    self.policy.worker_bytes,
                    FlightCancellation::default(),
                    move |_| {
                        execution::factorable_resolve_order(&program, intent, &controls)
                            .map_err(Into::into)
                    },
                )
                .await?;
            if let Some(order) = order {
                let callbacks = self
                    .prepare_order(prepared.clone(), order, FlightCancellation::default())
                    .await?;
                Some(PricingCase {
                    executable: self.assemble(callbacks.clone()).await?,
                    prepared: callbacks,
                })
            } else {
                None
            }
        } else {
            None
        };
        let result = PreparedSolve {
            snapshot: snapshot.clone(),
            representation: Representation::Algebraic(AlgebraicCase {
                prepared,
                case,
                pricing_case,
                values,
                providers,
                certificate,
                factorable,
                sensitivity: None,
                recognized,
                coefficient_cone,
            }),
            profile,
            requested_hessian,
            numerics,
            normalization,
            tolerances,
            accuracy,
            work_precision: None,
            route,
            compatibility: stamp,
            explicit_start: None,
            proposal_start: None,
            automatic_products: Vec::new(),
            automatic_owner: None,
            route_decision: Some(decision),
            composition: None,
            task_scope: None,
            task_admission: None,
            point_accuracy: None,
            selected_outputs: None,
            #[cfg(feature = "solver-kinsol")]
            causal_supplier: None,
            pool: self.pool.clone(),
            _owner: owner,
        };
        if deferred {
            let complete = match result.automatic_reconstruction_accuracy() {
                Some(accuracy) => self.automatic_reduced_request(&result, accuracy)?
                    .is_some_and(|request| matches!(request, DerivedRequest::Reduced { retained, .. } if retained.is_empty())),
                None => false,
            };
            if !complete {
                return Err(ProblemError::RouteRefused(Box::new(
                    result.route_decision.clone().ok_or_else(|| {
                        ProblemError::Internal("deferred original route decision absent".into())
                    })?,
                ))
                .into());
            }
        }
        Ok(result)
    }
    /// A conditional initialization block as a solve step (A6): the block's bound view, its
    /// assembled programs, the case-level numerical policy and the route resolved for the
    /// block before execution ([`super::initialization::PreparedInitialization::strategies`]).
    /// The block's acceptance budgets and transport come from that policy over its own
    /// columns and rows, as for any solve.
    #[cfg(feature = "solver-kinsol")]
    #[expect(
        clippy::too_many_arguments,
        reason = "a block step binds its view, programs, values, providers, profile, policy and route"
    )]
    pub(crate) async fn prepare_conditional(
        self: &Arc<Self>,
        prepared: Preparation,
        executable: Arc<ExecutableCase>,
        values: CaseValues,
        providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        mut profile: SolverProfile,
        numerics: Arc<ResolvedNumericalPolicy>,
        route: Route,
        snapshot: execution::Snapshot,
        scope: &pse_kernels::ExecutionScope,
        driver: &crate::CancelSource,
    ) -> Result<PreparedSolve, MathRuntimeError> {
        let requested_hessian = profile.controls.hessian;
        profile.resolve_curvature(prepared.prepared.plan.available_order());
        profile.controls.validate()?;
        let Route::Native(selected) = route else {
            return Err(ProblemError::Contract(
                "conditional block needs a selected native route".into(),
            )
            .into());
        };
        let order = routing::derivative_demand(
            execution::adapter(selected).capability(),
            &profile.controls,
        )
        .unwrap_or(pse_kernels::DerivativeOrder::Value);
        let method = match &profile.backend {
            BackendSettings::Kinsol(method) => *method,
            _ => native::kinsol::Method::default(),
        };
        let directional_only = selected == Backend::Kinsol && method.consumes_directional_only();
        let consumes_jvp = selected == Backend::Kinsol && method.consumes_jvp();
        let mut changed = false;
        let prepared = if !directional_only && prepared.prepared.plan.order() < order {
            changed = true;
            Self::within_task(
                scope,
                driver,
                self.prepare_order(prepared, order, FlightCancellation::default()),
            )
            .await?
        } else {
            prepared
        };
        let prepared = if consumes_jvp && !prepared.prepared.plan.has_directional_actions() {
            changed = true;
            Self::within_task(scope, driver, self.prepare_directional_actions(prepared)).await?
        } else {
            prepared
        };
        let executable = if changed {
            Self::within_task(scope, driver, self.assemble(prepared.clone())).await?
        } else {
            executable
        };
        let plan = &prepared.prepared.plan;
        plan.structure().validate_values(&values)?;
        if !prepared.prepared.presolve.matches(plan, &values) {
            return Err(ProblemError::Internal(
                "conditional block values differ from its bound view".into(),
            )
            .into());
        }
        let Route::Native(backend) = route else {
            return Err(ProblemError::Unsupported(
                "unsupported conditional initialization route".into(),
            )
            .into());
        };
        if !matches!(
            execution::adapter(backend).representation(),
            execution::Representation::Roots | execution::Representation::Nlp
        ) {
            return Err(ProblemError::Unsupported(
                "unsupported conditional initialization route".into(),
            )
            .into());
        }
        admit_profile(&profile, route, &snapshot)?;
        let rows: Vec<_> = plan.structure().rows().iter().map(|r| r.id).collect();
        let normalization = Normalization::from_policy(&numerics, plan.columns(), &rows)?;
        let tolerances = Tolerances::from_policy(&numerics, plan.columns(), &rows)?;
        let accuracy = ResolvedAccuracy::resolve(&numerics.policy, &tolerances, &normalization)?;
        let oracle = native::assembled::contract(plan);
        let mut prepared_operations = vec![routing::ArtifactDemand::Representation(
            execution::adapter(backend).representation(),
        )];
        if plan.has_directional_actions() {
            prepared_operations.push(routing::ArtifactDemand::JacobianProduct);
        }
        let decision = routing::Requirements {
            table: &execution::LINKED,
            facts: &prepared.prepared.facts,
            intent: profile.intent,
            numerical_psd: false,
            least_squares: false,
            controls: &profile.controls,
            settings: &profile.backend,
            sensitivity: false,
            context: routing::Context {
                snapshot: snapshot.clone(),
                pending_classes: &[],
                structure: None,
                oracle: Some(&oracle),
                guards: &prepared.prepared.presolve.signs,
                budgets: Some(execution::Budgets {
                    tolerances: &tolerances,
                    normalization: &normalization,
                    accuracy: &accuracy,
                }),
                coefficients: None,
                cone: None,
                factorable: None,
                certificate: None,
                prepared: &prepared_operations,
                refusals: &BTreeMap::new(),
            },
        }
        .bound_decision(
            SolverSelection::Explicit(backend),
            plan.columns().to_vec(),
            plan.structure()
                .rows()
                .iter()
                .map(|row| pse_structural::incidence::Constraint {
                    id: row.id,
                    lower: row.lower.is_finite().then_some(row.lower),
                    upper: row.upper.is_finite().then_some(row.upper),
                })
                .collect(),
            prepared.structural_witness(),
        )?;
        decision.route()?;
        let stamp = compatibility(
            plan, &values, &profile, &numerics, backend, &providers, &snapshot,
        )?;
        let bytes = (plan.structure().variables().len() + rows.len())
            .checked_mul(size_of::<pse_math::numerics::TargetSpec>() + 8 * size_of::<f64>())
            .and_then(|n| n.checked_add(size_of::<PreparedSolve>()))
            .ok_or(MathRuntimeError::Limit("solve metadata extent"))?;
        Ok(PreparedSolve {
            snapshot,
            representation: Representation::Algebraic(AlgebraicCase {
                prepared,
                case: Some(executable),
                pricing_case: None,
                values,
                providers,
                certificate: None,
                // A block runs only on a root or NLP route, refused above otherwise.
                factorable: None,
                sensitivity: None,
                recognized: None,
                coefficient_cone: None,
            }),
            profile,
            requested_hessian,
            numerics,
            normalization,
            tolerances,
            accuracy,
            work_precision: None,
            route,
            compatibility: Some(stamp),
            explicit_start: None,
            proposal_start: None,
            automatic_products: Vec::new(),
            automatic_owner: None,
            route_decision: Some(decision),
            composition: None,
            task_scope: None,
            task_admission: None,
            point_accuracy: None,
            selected_outputs: None,
            #[cfg(feature = "solver-kinsol")]
            causal_supplier: None,
            pool: self.pool.clone(),
            _owner: self.reserve("math:prepared-block", bytes)?,
        })
    }
    /// Explicit conic representation enters the same bounded worker/report lifecycle.
    pub async fn prepare_conic(
        self: &Arc<Self>,
        problem: Arc<native::ConicProblem>,
        certificate: Arc<dyn QuadraticEvidence>,
        mut profile: SolverProfile,
        numerics: Arc<ResolvedNumericalPolicy>,
    ) -> Result<PreparedSolve, MathRuntimeError> {
        let requested_hessian = profile.controls.hessian;
        profile.resolve_curvature(problem.contract.derivatives);
        profile.controls.validate()?;
        if profile.numerics.key() != numerics.policy.key() {
            return Err(ProblemError::Contract(
                "conic numerical policy differs from its resolved authority".into(),
            )
            .into());
        }
        let sparse_bytes = [&problem.quadratic, &problem.constraints]
            .iter()
            .try_fold(0usize, |n, a| {
                n.checked_add(
                    a.column_starts
                        .capacity()
                        .checked_add(a.row_indices.capacity())?
                        .checked_mul(size_of::<usize>())?,
                )?
                .checked_add(a.values.capacity().checked_mul(size_of::<f64>())?)
            })
            .ok_or(MathRuntimeError::Limit("conic product extent"))?;
        // The normalized copy beside the retained original.
        let bytes = (problem.contract.variables.len() + problem.contract.rows.len())
            .checked_mul(size_of::<pse_math::numerics::TargetSpec>() + 8 * size_of::<f64>())
            .and_then(|n| n.checked_add(sparse_bytes.checked_mul(2)?))
            .ok_or(MathRuntimeError::Limit("conic product extent"))?;
        let owner = self.reserve("math:prepared-conic", bytes)?;
        let admitted = problem.clone();
        let proof = certificate.clone();
        self.job(
            1,
            self.policy.worker_bytes,
            FlightCancellation::default(),
            move |_| admitted.validate(proof.as_ref()).map_err(Into::into),
        )
        .await?;
        let ids: Vec<_> = problem.contract.variables.iter().map(|v| v.id).collect();
        let normalization = native::transport::cone_normalization(&problem, &numerics)?;
        certificate.validate_policy(
            profile.convexity,
            &normalization.variables,
            normalization.objective,
        )?;
        let mut resolved = numerics.as_ref().clone();
        for (id, scale) in problem.contract.rows.iter().zip(&normalization.rows) {
            let target = resolved
                .targets
                .iter_mut()
                .find(|t| {
                    t.id == *id && t.kind == pse_model::generated::enums::NumericalTarget::Row
                })
                .ok_or_else(|| ProblemError::Contract("missing cone numerical row".into()))?;
            if target.coordinate_scale != *scale {
                target.coordinate_scale = *scale;
                target
                    .provenance
                    .push(pse_model::numerics::NumericalProvenance {
                        declaration: None,
                        source: pse_model::generated::enums::NumericalSource::CanonicalFallback,
                        field:
                            pse_model::generated::enums::NumericalProvenanceField::CoordinateScale,
                        selected: true,
                        value: *scale,
                        description: "common positive scale preserving the declared cone geometry"
                            .into(),
                    });
            }
        }
        let mut identity = FramedHasher::new(pse_ids::Frame::NumericalConeV1);
        identity.hash(&resolved.key).hash(&normalization.key());
        resolved.key = identity.finish_hash();
        let numerics = Arc::new(resolved);
        let tolerances = Tolerances::from_policy(&numerics, &ids, &problem.contract.rows)?;
        let accuracy = ResolvedAccuracy::resolve(&numerics.policy, &tolerances, &normalization)?;
        let snapshot = execution::Snapshot::observe(&execution::LINKED);
        let facts = routing::conic_facts(&problem, certificate.as_ref())?;
        let source = problem.clone();
        let extent = native::structural::conic_construction_bytes(&source)?;
        let (mut structure, lease) = self
            .job_retained(1, extent, FlightCancellation::default(), move |flag| {
                let structure = native::structural::conic_structure_with_cancel(&source, &flag)?;
                let bytes = native::structural::retained_bytes(&structure);
                Ok((structure, bytes))
            })
            .await?;
        structure.witness = structure.witness.with_owner(lease);
        let mut decision = routing::Requirements {
            table: &execution::LINKED,
            facts: &facts,
            intent: profile.intent,
            numerical_psd: false,
            least_squares: false,
            controls: &profile.controls,
            settings: &profile.backend,
            sensitivity: profile.sensitivity.is_some(),
            context: routing::Context {
                snapshot: snapshot.clone(),
                pending_classes: &[],
                structure: Some(structure),
                oracle: Some(&problem.contract),
                guards: &BTreeMap::new(),
                budgets: Some(execution::Budgets {
                    tolerances: &tolerances,
                    normalization: &normalization,
                    accuracy: &accuracy,
                }),
                coefficients: None,
                cone: Some(routing::ConeEvidence {
                    problem: &problem,
                    certificate: certificate.as_ref(),
                }),
                factorable: None,
                certificate: Some(certificate.as_ref()),
                prepared: &[routing::ArtifactDemand::Representation(
                    execution::Representation::Cone,
                )],
                refusals: &BTreeMap::new(),
            },
        }
        .decision(profile.selection);
        let route = decision.route()?;
        let Route::Native(backend) = route else {
            return Err(ProblemError::Contract(
                "explicit cone requires a native optimization route".into(),
            )
            .into());
        };
        let adapter = execution::adapter(backend);
        if adapter.representation() != execution::Representation::Cone {
            return Err(ProblemError::Contract(
                "explicit cone requires a cone representation".into(),
            )
            .into());
        }
        admit_profile(&profile, route, &snapshot)?;
        let admission_owner = self.reserve("math:cone-admission", decision.retained_bytes())?;
        for assessment in decision
            .eligibility
            .iter_mut()
            .filter_map(|entry| entry.structure.as_mut())
            .chain(decision.structure.iter_mut())
        {
            assessment.witness = assessment
                .witness
                .clone()
                .with_owner(admission_owner.clone());
        }
        let (normalized, transported) =
            native::transport::conic(&problem, &normalization, certificate.as_ref())?;
        let original = problem;
        let problem = Arc::new(normalized);
        let certificate: Arc<dyn QuadraticEvidence> = Arc::new(transported);
        // Cone coordinates are normalized at preparation, so the numerical policy belongs to
        // them; controls and settings form the profile (F24).
        let mut h = FramedHasher::new(pse_ids::Frame::SolverConicLayoutV3);
        h.hash(&numerics.key).hash(&normalization.key());
        h.hash(&problem.contract.identity);
        let mut session = FramedHasher::new(pse_ids::Frame::SolverConicSessionV3);
        hash_session(&mut session, &profile)?;
        session.hash(&snapshot.identity());
        h.hash(&native::conic::cone_key(&problem.cones)?);
        for v in &problem.contract.variables {
            h.id(&v.id);
        }
        for id in &problem.contract.rows {
            h.id(id);
        }
        for a in [&problem.quadratic, &problem.constraints] {
            h.u64(a.rows as u64)
                .u64(a.columns as u64)
                .u64(a.column_starts.len() as u64)
                .u64(a.row_indices.len() as u64);
            for v in a.column_starts.iter().chain(&a.row_indices) {
                h.u64(*v as u64);
            }
        }
        let mut d = FramedHasher::new(pse_ids::Frame::SolverConicDataV1);
        for x in problem
            .quadratic
            .values
            .iter()
            .chain(&problem.constraints.values)
            .chain(&problem.objective)
            .chain(&problem.rhs)
        {
            d.u64(x.to_bits());
        }
        d.u64(problem.objective_constant.to_bits());
        for v in &problem.contract.variables {
            d.u64(v.lower.to_bits()).u64(v.upper.to_bits());
            h.bool(v.lower.is_finite()).bool(v.upper.is_finite());
        }
        let stamp = Compatibility {
            layout: h.finish_hash(),
            profile: session.finish_hash(),
            data: d.finish_hash(),
            backend: adapter.backend(),
        };
        Ok(PreparedSolve {
            snapshot,
            representation: Representation::Conic {
                problem,
                original,
                certificate,
            },
            profile,
            requested_hessian,
            numerics,
            normalization,
            tolerances,
            accuracy,
            work_precision: None,
            route,
            compatibility: Some(stamp),
            explicit_start: None,
            proposal_start: None,
            automatic_products: Vec::new(),
            automatic_owner: None,
            route_decision: Some(decision),
            composition: None,
            task_scope: None,
            task_admission: None,
            point_accuracy: None,
            selected_outputs: None,
            #[cfg(feature = "solver-kinsol")]
            causal_supplier: None,
            pool: self.pool.clone(),
            _owner: owner,
        })
    }
    /// Start one prepared step on its own native session: the job lifecycle every single
    /// solve shares with staged sequences (A6). Dropping the handle requests cancellation;
    /// finishing witnesses native teardown and join.
    ///
    /// # Errors
    /// An explicit start policy without a seed, or no admission capacity.
    pub fn solve(
        self: &Arc<Self>,
        step: PreparedSolve,
    ) -> Result<SolveHandle<StepReport>, MathRuntimeError> {
        let step = if step.task_scope.is_none() {
            let deadline = std::time::Instant::now()
                .checked_add(step.time_limit())
                .ok_or_else(|| ProblemError::Contract("submitted solve deadline extent".into()))?;
            step.within_task(pse_kernels::ExecutionScope::new(
                Arc::default(),
                Some(deadline),
            ))?
        } else {
            step
        };
        if step.profile.controls.start == StartPolicy::Explicit && step.explicit_start.is_none() {
            return Err(ProblemError::Contract(
                "explicit start policy requires a seed before submission".into(),
            )
            .into());
        }
        let owner = self.reserve("math:solve-results", step.result_bytes()?)?;
        let run_id = pse_operations::mint_id();
        let progress = Arc::new(Progress::new(step.profile.controls.history));
        let session = self.open_session()?;
        let events = progress.clone();
        let numerical_policy = step.numerics.policy.clone();
        Ok(SolveHandle::supervise(progress, move |cancel| async move {
            let result = session
                .step(
                    step,
                    None,
                    0,
                    events,
                    owner.clone(),
                    &cancel,
                    move |outcome, _, _| {
                        super::strategy::Assessed::native(
                            (),
                            super::StepRetention {
                                candidate: outcome.candidate_use(&numerical_policy),
                                session: super::SessionDisposition::Discard,
                            },
                            outcome,
                        )
                    },
                )
                .await;
            session.close().await;
            result.map(|(outcome, (), strategy)| StepReport {
                run_id,
                outcome,
                strategy,
                _owner: owner,
            })
        }))
    }
    /// One bound step on a session's retained native state (A6): the seed its start policy
    /// selects, checked against its coordinates, the selected adapter's representation
    /// runner and the submitted-start receipt. A `Fresh` reuse policy drops retained state
    /// first; a refused seed drops it as well.
    #[expect(
        clippy::too_many_arguments,
        reason = "the session supplies its retained state, stop flag, progress, worker share and result owner"
    )]
    pub(crate) fn execute(
        &self,
        step: PreparedSolve,
        previous: Option<Predecessor>,
        attempt: usize,
        retained: &mut Retained,
        flag: &Arc<std::sync::atomic::AtomicBool>,
        progress: &Arc<Progress>,
        budget: &Arc<WorkerBudget>,
        owner: &Arc<pse_columnar::AllocationLease>,
        scope: &pse_kernels::ExecutionScope,
    ) -> Result<ScopedOutcome, MathRuntimeError> {
        let mut admitted =
            match self.admit_step(step, previous, retained, flag, progress, Some(scope)) {
                Ok(admitted) => admitted,
                Err(refused) => return Ok((refused, None)),
            };
        admitted.execution.work_admission = budget
            .admission()
            .map(|owner| -> Arc<dyn WorkAdmission> { owner });
        let Admitted {
            step,
            chosen,
            execution,
            receipt,
            normalization,
        } = admitted;
        let scope = execution.scope()?;
        let outcome = self
            .run_step(step, execution, chosen.as_ref(), retained, budget)
            .unwrap_or_else(|e| Outcome::Rejected(Arc::new(e)));
        self.conclude(outcome, receipt, normalization, attempt, owner)
            .map(|outcome| (outcome, Some(scope)))
    }
    /// The independent steps of one batch on the session's retained native state (Plan 22
    /// N5): each is admitted as [`Self::execute`] admits it, those that share one batching
    /// adapter over coefficient programs are solved together
    /// ([`execution::coefficients_batch`]), and each outcome is concluded on its own. Steps
    /// that cannot join the batch run in turn. One outcome per step, in order.
    pub(crate) fn execute_batch(
        &self,
        members: Vec<BatchMember>,
        retained: &mut Retained,
        flag: &Arc<std::sync::atomic::AtomicBool>,
        progress: &Arc<Progress>,
        budget: &Arc<WorkerBudget>,
        scope: &pse_kernels::ExecutionScope,
    ) -> Vec<Result<ScopedOutcome, MathRuntimeError>> {
        let mut outcomes: Vec<Option<Result<ScopedOutcome, MathRuntimeError>>> =
            members.iter().map(|_| None).collect();
        let mut admitted = Vec::new();
        for (i, member) in members.into_iter().enumerate() {
            match self.admit_step(member.step, None, retained, flag, progress, Some(scope)) {
                Ok(step) => admitted.push((i, member.attempt, member.owner, step)),
                Err(refused) => outcomes[i] = Some(Ok((refused, None))),
            }
        }
        let batching = |a: &Admitted| {
            let Route::Native(backend) = a.step.route else {
                return None;
            };
            let adapter = execution::adapter(backend);
            let coefficients = matches!(
                &a.step.representation,
                Representation::Algebraic(case)
                    if case.prepared.prepared.coefficients.is_some()
                        && case.recognized.is_none()
                        && case.sensitivity.is_none()
            );
            (adapter.capability().batch
                && coefficients
                && matches!(
                    adapter.representation(),
                    execution::Representation::Coefficients | execution::Representation::Cone
                ))
            .then_some(backend)
        };
        let shared = admitted
            .first()
            .and_then(|(_, _, _, a)| batching(a))
            .filter(|backend| {
                admitted
                    .iter()
                    .all(|(_, _, _, a)| batching(a) == Some(*backend))
            });
        if shared.is_some() && admitted.len() > 1 {
            let concluded = self.coefficient_batch(admitted, retained, budget);
            for (i, outcome) in concluded {
                outcomes[i] = Some(outcome);
            }
        } else {
            for (i, attempt, owner, a) in admitted {
                let Admitted {
                    step,
                    chosen,
                    execution,
                    receipt,
                    normalization,
                } = a;
                let scope = match execution.scope() {
                    Ok(scope) => scope,
                    Err(error) => {
                        outcomes[i] = Some(Err(error.into()));
                        continue;
                    }
                };
                let outcome = self
                    .run_step(step, execution, chosen.as_ref(), retained, budget)
                    .unwrap_or_else(|e| Outcome::Rejected(Arc::new(e)));
                outcomes[i] = Some(
                    self.conclude(outcome, receipt, normalization, attempt, &owner)
                        .map(|outcome| (outcome, Some(scope))),
                );
            }
        }
        outcomes
            .into_iter()
            .map(|o| {
                o.unwrap_or_else(|| {
                    Err(MathRuntimeError::Infrastructure(
                        "batch member without an outcome".into(),
                    ))
                })
            })
            .collect()
    }
    /// Admit one step: its reuse policy, its seed by start policy, checked against its
    /// coordinates, its execution controls and its start receipt; or the outcome that
    /// refuses it.
    fn admit_step(
        &self,
        step: PreparedSolve,
        previous: Option<Predecessor>,
        retained: &mut Retained,
        flag: &Arc<std::sync::atomic::AtomicBool>,
        progress: &Arc<Progress>,
        scope: Option<&pse_kernels::ExecutionScope>,
    ) -> Result<Admitted, Outcome> {
        if let Some(decision) = &step.route_decision
            && decision.state == routing::AssessmentState::Refused
        {
            return Err(Outcome::Rejected(Arc::new(
                ProblemError::RouteRefused(Box::new(decision.clone())).into(),
            )));
        }
        let controls = step.profile.controls.clone();
        if controls.reuse == ReusePolicy::Fresh {
            retained.clear();
        }
        let (chosen, previous_attempt) = if step.proposal_start.is_some() {
            (step.explicit_start.clone(), None)
        } else {
            match controls.start {
                StartPolicy::NoPriorStart => (None, None),
                StartPolicy::Explicit => match step.explicit_start.clone() {
                    Some(seed) => (Some(seed), None),
                    None => {
                        return Err(Outcome::Rejected(Arc::new(
                            ProblemError::Contract("explicit start policy requires a seed".into())
                                .into(),
                        )));
                    }
                },
                StartPolicy::PreviousAccepted => {
                    previous.map_or((None, None), |p| (Some(p.seed), Some(p.attempt)))
                }
            }
        };
        if let Some(seed) = &chosen {
            let validation = step
                .compatibility
                .as_ref()
                .ok_or_else(|| {
                    ProblemError::Contract("constant evaluation cannot consume a seed".into())
                })
                .and_then(|target| seed.validate(target));
            if let Err(error) = validation {
                retained.clear();
                return Err(Outcome::Rejected(Arc::new(error.into())));
            }
        }
        let mut execution = match scope {
            Some(scope) => Execution::within(flag.clone(), &controls, scope.clone())
                .map_err(|error| Outcome::Rejected(Arc::new(error.into())))?,
            None => Execution::new(flag.clone(), &controls),
        };
        execution.progress = progress.clone();
        // Libraries that enforce their own memory limit read the solve's foreign allowance.
        execution.memory = Some(self.policy.foreign_allowance(&controls));
        let receipt = StartReceipt {
            previous_attempt,
            seed: chosen.clone(),
            sparse_seed: step.profile.backend.partial_start().cloned(),
            transformations: vec![],
            submitted: chosen.is_some(),
        };
        let normalization = step.normalization.key();
        Ok(Admitted {
            step,
            chosen,
            execution,
            receipt,
            normalization,
        })
    }
    /// A step's outcome with its failure owner, its seed's origin, its start receipt and its
    /// result owner.
    fn conclude(
        &self,
        outcome: Outcome,
        receipt: StartReceipt,
        normalization: pse_ids::ContentHash,
        attempt: usize,
        owner: &Arc<pse_columnar::AllocationLease>,
    ) -> Result<Outcome, MathRuntimeError> {
        Ok(match outcome {
            Outcome::Native(mut r) => {
                if r.failure_bytes() > 0 {
                    let failure_owner = self.reserve("math:solve-failure", r.failure_bytes())?;
                    *r = (*r).with_failure_owner(failure_owner);
                }
                if let Some(seed) = &mut r.warm_start {
                    seed.origin = Some(SeedOrigin { run: None, attempt });
                }
                // The recorded path is what this step's transport, library presolve and
                // interior-point restart actually applied, never a constant label (F25).
                let mut receipt = receipt;
                receipt.record(&r, normalization);
                r.start_receipt = Some(receipt);
                Outcome::Native(Box::new((*r).with_owner(owner.clone())))
            }
            Outcome::Constant(mut r) => {
                r.owner = Some(owner.clone());
                Outcome::Constant(r)
            }
            other => other,
        })
    }
    /// The sum-of-squares bound on a prepared polynomial program (Plan 22 N5; I5): its
    /// factorable projection, expanded into monomials and bounded by the moment relaxation
    /// on one admitted worker within the step's time limit. The bound is labelled
    /// `sos_bound_nonrigorous` (never a certified global bound).
    ///
    /// # Errors
    /// A step that is not an algebraic case, a program that is not polynomial or exceeds
    /// the relaxation's bounds, admission, or a build without POUNCE-convex.
    pub(crate) async fn sos_bound(
        self: &Arc<Self>,
        step: &PreparedSolve,
        order: Option<usize>,
    ) -> Result<execution::sos::SosBound, MathRuntimeError> {
        let Representation::Algebraic(case) = &step.representation else {
            return Err(
                ProblemError::Contract("an SOS bound needs an algebraic case".into()).into(),
            );
        };
        let plan = case.prepared.prepared.plan.clone();
        let values = case.values.clone();
        let limit = self.policy.worker_bytes / 256;
        let tolerance = step.accuracy.stationarity.max(1e-9);
        let time_limit = step.profile.controls.time_limit;
        self.job(
            1,
            self.policy.worker_bytes,
            FlightCancellation::default(),
            move |flag| {
                let program = plan
                    .factorable_program(
                        &values,
                        &pse_math::factorable::FactorableRequest::default(),
                        limit,
                        &flag,
                    )
                    .map_err(|e| match e {
                        pse_math::factorable::FactorableError::Math(e) => ProblemError::Math(e),
                        other => ProblemError::Unsupported(other.to_string()),
                    })?;
                let problem = execution::sos::polynomial(&program)?;
                Ok(execution::sos::bound(
                    &problem, order, tolerance, time_limit,
                )?)
            },
        )
        .await
    }
    /// The coefficient programs of one batch on their shared batching adapter: each member's
    /// case, projection and original model, solved together by the runner and concluded one
    /// by one.
    fn coefficient_batch(
        &self,
        admitted: Vec<(usize, usize, Arc<pse_columnar::AllocationLease>, Admitted)>,
        retained: &mut Retained,
        budget: &Arc<WorkerBudget>,
    ) -> Vec<(usize, Result<ScopedOutcome, MathRuntimeError>)> {
        /// One member's owned parts, which the runner's views borrow.
        struct Part {
            index: usize,
            attempt: usize,
            owner: Arc<pse_columnar::AllocationLease>,
            receipt: StartReceipt,
            key: pse_ids::ContentHash,
            chosen: Option<WarmStart>,
            execution: Execution,
            scope: pse_kernels::ExecutionScope,
            profile: SolverProfile,
            snapshot: execution::Snapshot,
            structure: Option<native::structural::Assessment>,
            normalization: Normalization,
            tolerances: Tolerances,
            accuracy: ResolvedAccuracy,
            compatibility: Compatibility,
            backend: Backend,
            case: AlgebraicCase,
            problem: native::CoefficientProblem,
        }
        let mut concluded = Vec::new();
        let mut parts = Vec::new();
        for (index, attempt, owner, a) in admitted {
            let Admitted {
                step,
                chosen,
                execution,
                receipt,
                normalization: key,
            } = a;
            let PreparedSolve {
                representation,
                profile,
                normalization,
                tolerances,
                accuracy,
                route,
                snapshot,
                route_decision,
                compatibility,
                ..
            } = step;
            let part = (|| -> Result<Part, MathRuntimeError> {
                let scope = execution.scope()?;
                let (Representation::Algebraic(case), Route::Native(backend)) =
                    (representation, route)
                else {
                    return Err(ProblemError::Internal(
                        "a batch member is a native coefficient step".into(),
                    )
                    .into());
                };
                let coefficients = case
                    .prepared
                    .prepared
                    .coefficients
                    .as_ref()
                    .ok_or_else(|| ProblemError::Internal("missing coefficient product".into()))?;
                let problem = native::CoefficientProblem::from_plan(
                    &case.prepared.prepared.plan,
                    coefficients.as_ref().clone(),
                )?;
                let compatibility = compatibility.ok_or_else(|| {
                    ProblemError::Internal("missing native compatibility stamp".into())
                })?;
                Ok(Part {
                    index,
                    attempt,
                    owner: owner.clone(),
                    receipt: receipt.clone(),
                    key,
                    chosen: chosen.clone(),
                    execution: execution.clone(),
                    scope,
                    profile,
                    snapshot,
                    structure: route_decision.and_then(|decision| decision.structure),
                    normalization,
                    tolerances,
                    accuracy,
                    compatibility,
                    backend,
                    case,
                    problem,
                })
            })();
            match part {
                Ok(part) => parts.push(part),
                Err(error) => {
                    let outcome = Outcome::Rejected(Arc::new(error));
                    concluded.push((
                        index,
                        self.conclude(outcome, receipt, key, attempt, &owner)
                            .map(|outcome| (outcome, None)),
                    ));
                }
            }
        }
        let mut originals: Vec<OriginalCase<'_>> = parts
            .iter()
            .map(|p| OriginalCase {
                service: self,
                case: p.case.case.clone(),
                providers: &p.case.providers,
                values: &p.case.values,
                plan: &p.case.prepared.prepared.plan,
                scope: p.scope.clone(),
                budget,
            })
            .collect();
        let mut steps = Vec::with_capacity(parts.len());
        for (p, original) in parts.iter().zip(originals.iter_mut()) {
            let Some(coefficients) = p.case.prepared.prepared.coefficients.as_ref() else {
                continue;
            };
            let plan = &p.case.prepared.prepared.plan;
            steps.push((
                execution::Step {
                    adapter: execution::adapter(p.backend),
                    snapshot: &p.snapshot,
                    structure: p.structure.as_ref(),
                    settings: &p.profile.backend,
                    controls: &p.profile.controls,
                    accuracy: &p.accuracy,
                    execution: p.execution.clone(),
                    tolerances: &p.tolerances,
                    normalization: &p.normalization,
                    compatibility: p.compatibility.clone(),
                    warm: p.chosen.as_ref(),
                },
                execution::Coefficients {
                    problem: &p.problem,
                    certificate: p.case.certificate.as_deref(),
                    lowered: p
                        .case
                        .coefficient_cone
                        .as_ref()
                        .map(|(cone, _)| cone.as_ref()),
                    row_constants: &coefficients.row_constants,
                    row_bounds: plan
                        .structure()
                        .rows()
                        .iter()
                        .map(|r| (r.lower, r.upper))
                        .collect(),
                    original,
                },
            ));
        }
        let reports = execution::coefficients_batch(retained, steps);
        drop(originals);
        for (p, report) in parts.into_iter().zip(reports) {
            let outcome = match report {
                Ok(report) => Outcome::Native(Box::new(report)),
                Err(error) => Outcome::Rejected(Arc::new(error.into())),
            };
            concluded.push((
                p.index,
                self.conclude(outcome, p.receipt, p.key, p.attempt, &p.owner)
                    .map(|outcome| (outcome, Some(p.scope))),
            ));
        }
        concluded
    }
    /// One prepared step: constant evaluation, or the selected adapter's representation
    /// runner. No backend is named; the adapter declares its representation.
    fn run_step(
        &self,
        step: PreparedSolve,
        execution: Execution,
        warm: Option<&WarmStart>,
        retained: &mut Retained,
        budget: &Arc<WorkerBudget>,
    ) -> Result<Outcome, MathRuntimeError> {
        // Every dispatch replaces this point's pending artifact. No previous attempt
        // can supply a factor to a later candidate, even if that attempt was refused.
        step.clear_point_accuracy()?;
        let output_accuracy = step.kkt_observer(budget)?;
        let PreparedSolve {
            representation,
            profile,
            normalization,
            tolerances,
            accuracy,
            route,
            snapshot,
            route_decision,
            compatibility,
            ..
        } = step;
        // Only a modeling solve prepares the parametric program a sensitivity request
        // differentiates; any other caller is refused rather than silently ignored.
        if profile.sensitivity.is_some()
            && !matches!(&representation, Representation::Algebraic(a) if a.sensitivity.is_some())
        {
            return Err(ProblemError::Contract(
                "a sensitivity request needs the parametric program a modeling solve prepares"
                    .into(),
            )
            .into());
        }
        let Route::Native(backend) = route else {
            return self.constant(representation, &tolerances, &execution.scope()?, budget);
        };
        let adapter = execution::adapter(backend);
        let stamp = compatibility
            .ok_or_else(|| ProblemError::Internal("missing native compatibility stamp".into()))?;
        let run = execution::Step {
            adapter,
            snapshot: &snapshot,
            structure: route_decision
                .as_ref()
                .and_then(|decision| decision.structure.as_ref()),
            settings: &profile.backend,
            controls: &profile.controls,
            accuracy: &accuracy,
            execution,
            tolerances: &tolerances,
            normalization: &normalization,
            compatibility: stamp,
            warm,
        };
        let authored = match &representation {
            Representation::Algebraic(a) => a.prepared.prepared.facts.requirements.clone(),
            Representation::Conic { .. } => Vec::new(),
        };
        let mut report = match (representation, adapter.representation()) {
            (
                Representation::Conic {
                    problem,
                    original,
                    certificate,
                },
                execution::Representation::Cone,
            ) => execution::cone(run, retained, &problem, &original, certificate.as_ref())?,
            // A cone adapter serves a recognized convex program in its cone form, and a
            // coefficient model through the coefficient runner's lowering.
            (Representation::Algebraic(case), execution::Representation::Cone)
                if case.recognized.is_some() =>
            {
                self.recognized_step(run, retained, case, budget)?
            }
            (
                Representation::Algebraic(case),
                execution::Representation::Coefficients | execution::Representation::Cone,
            ) => self.coefficient_step(run, retained, case, budget)?,
            (Representation::Algebraic(case), execution::Representation::Factorable) => {
                self.factorable_step(run, retained, case, &profile, budget)?
            }
            (
                Representation::Algebraic(case),
                kind @ (execution::Representation::Nlp | execution::Representation::Roots),
            ) => {
                self.callback_step(run, retained, case, kind, &profile, budget, output_accuracy)?
            }
            _ => {
                return Err(ProblemError::Internal(
                    "prepared representation differs from the selected adapter".into(),
                )
                .into());
            }
        };
        if authored
            .contains(&pse_model::generated::enums::ModelingStructuralRequirement::L1ExactPenalty)
        {
            report.provenance.insert(
                "method.selection".into(),
                "the authored penalty(l1) realization selects POUNCE's l1 exact penalty-barrier (ADR-0104 §5): the author's selection, not an automatic one".into(),
            );
            report.provenance.insert(
                "method.penalty_scope".into(),
                "the l1 exact penalty relaxes every constraint row of the solve, not only the rows of the penalty(l1) realization".into(),
            );
        }
        Ok(Outcome::Native(Box::new(report)))
    }
    /// Coefficient projection of the compiled case, re-checked against the original case.
    fn coefficient_step(
        &self,
        run: execution::Step<'_>,
        retained: &mut Retained,
        case: AlgebraicCase,
        budget: &Arc<WorkerBudget>,
    ) -> Result<SolveReport, MathRuntimeError> {
        let AlgebraicCase {
            prepared,
            case,
            values,
            providers,
            certificate,
            coefficient_cone,
            ..
        } = case;
        let coefficients = prepared
            .prepared
            .coefficients
            .as_ref()
            .ok_or_else(|| ProblemError::Internal("missing coefficient product".into()))?;
        let plan = &prepared.prepared.plan;
        let problem = native::CoefficientProblem::from_plan(plan, coefficients.as_ref().clone())?;
        let mut original = OriginalCase {
            service: self,
            case,
            providers: &providers,
            values: &values,
            plan,
            scope: run.execution.scope()?,
            budget,
        };
        Ok(execution::coefficients(
            run,
            retained,
            execution::Coefficients {
                problem: &problem,
                certificate: certificate.as_deref(),
                lowered: coefficient_cone.as_ref().map(|(cone, _)| cone.as_ref()),
                row_constants: &coefficients.row_constants,
                row_bounds: plan
                    .structure()
                    .rows()
                    .iter()
                    .map(|r| (r.lower, r.upper))
                    .collect(),
                original: &mut original,
            },
        )?)
    }
    /// A recognized convex program's cone form on a cone adapter, raised to the program
    /// and re-checked against the original case (ADR-0121 Outcome 6).
    fn recognized_step(
        &self,
        run: execution::Step<'_>,
        retained: &mut Retained,
        case: AlgebraicCase,
        budget: &Arc<WorkerBudget>,
    ) -> Result<SolveReport, MathRuntimeError> {
        let AlgebraicCase {
            prepared,
            case,
            values,
            providers,
            recognized,
            ..
        } = case;
        let (lowered, proof, _owner) = recognized
            .ok_or_else(|| ProblemError::Internal("missing recognized cone form".into()))?;
        let plan = &prepared.prepared.plan;
        let mut original = OriginalCase {
            service: self,
            case,
            providers: &providers,
            values: &values,
            plan,
            scope: run.execution.scope()?,
            budget,
        };
        Ok(execution::recognized(
            run,
            retained,
            execution::Recognized {
                lowered: &lowered,
                certificate: Some(proof.as_ref()),
                original: &mut original,
            },
        )?)
    }
    /// Factorable export of the compiled case for a global adapter. Candidates are
    /// re-checked against the original case, and a discrete assignment is re-solved as a
    /// continuous problem through the one NLP runner (ADR-0105 §2).
    fn factorable_step(
        &self,
        run: execution::Step<'_>,
        retained: &mut Retained,
        case: AlgebraicCase,
        profile: &SolverProfile,
        budget: &Arc<WorkerBudget>,
    ) -> Result<SolveReport, MathRuntimeError> {
        let AlgebraicCase {
            prepared,
            case,
            values,
            providers,
            factorable,
            pricing_case,
            sensitivity,
            ..
        } = case;
        let (program, _owner) = factorable
            .ok_or_else(|| ProblemError::Internal("missing factorable program".into()))?;
        let plan = &prepared.prepared.plan;
        if !program.matches(plan, &values) {
            return Err(ProblemError::Contract(
                "factorable program assumptions differ from the case values".into(),
            )
            .into());
        }
        let initial: Vec<_> = plan.columns().iter().map(|id| values.scalars[id]).collect();
        let mut original = OriginalCase {
            service: self,
            case: case.clone(),
            providers: &providers,
            values: &values,
            plan,
            scope: run.execution.scope()?,
            budget,
        };
        let normalization = run.normalization.clone();
        let tolerance = run.tolerances.clone();
        let scope = run.execution.scope()?;
        // Executable owners and their budget charges outlive every re-solve oracle.
        let mut owners = Vec::new();
        let mut relaxed = || {
            (|| -> Result<native::transform::Relaxed, MathRuntimeError> {
                let ExecutionWorker {
                    worker,
                    _case,
                    _charge,
                } = self.case_worker(
                    pricing_case
                        .as_ref()
                        .map(|case| case.executable.clone())
                        .or_else(|| case.clone()),
                    providers.clone(),
                    &scope,
                    budget,
                )?;
                owners.push((_case, _charge));
                Ok(native::assembled::AlgebraicOracle::relaxation(
                    worker,
                    values.clone(),
                    normalization.clone(),
                )?)
            })()
            .map_err(MathRuntimeError::into_problem)
        };
        // Root response uses the original postsolve system, never optimizing KKT
        // callbacks from the optional continuous re-solve.
        let optimizing_sensitivity = if profile.intent == SolveIntent::Root {
            None
        } else {
            sensitivity
                .as_ref()
                .and_then(ParametricPreparation::available)
        };
        // The re-solve's parametric callbacks under the same assignment (Plan 22 S1).
        let mut parametric_owners = Vec::new();
        let mut parametric = || {
            (|| -> Result<native::transform::Relaxed, MathRuntimeError> {
                let request = optimizing_sensitivity.ok_or_else(|| {
                    ProblemError::Internal("no sensitivity program to differentiate".into())
                })?;
                let ExecutionWorker {
                    worker,
                    _case,
                    _charge,
                } = self.worker(request.program.clone(), &providers, scope.clone(), budget)?;
                parametric_owners.push((_case, _charge));
                Ok(native::assembled::AlgebraicOracle::relaxation(
                    worker,
                    values.clone(),
                    request.normalization.clone(),
                )?)
            })()
            .map_err(MathRuntimeError::into_problem)
        };
        let mut report = execution::factorable(
            run,
            retained,
            execution::Factorable {
                program: &program,
                initial: &initial,
                intent: profile.intent,
                original: &mut original,
                resolve: Some(execution::Resolve {
                    oracle: &mut relaxed,
                    presolve: &profile.presolve,
                    limit: self.policy.worker_bytes / 256,
                    sensitivity: optimizing_sensitivity.map(|request| {
                        execution::ResolveSensitivity {
                            oracle: &mut parametric,
                            parameters: request.parameters.clone(),
                            reduced_hessian: request.reduced_hessian,
                        }
                    }),
                }),
            },
        )?;
        drop(owners);
        drop(parametric_owners);
        if profile.intent == SolveIntent::Root {
            self.root_response(
                &mut report,
                &prepared,
                sensitivity.as_ref(),
                &values,
                &providers,
                &scope,
                &normalization,
                &tolerance,
                budget,
            );
        }
        Ok(report)
    }
    /// Callback oracle over the compiled case for an NLP or root-system adapter.
    #[expect(
        clippy::too_many_arguments,
        reason = "the shared callback runner also carries the point-owned deferred accuracy consumer"
    )]
    fn callback_step(
        &self,
        run: execution::Step<'_>,
        retained: &mut Retained,
        case: AlgebraicCase,
        kind: execution::Representation,
        profile: &SolverProfile,
        budget: &Arc<WorkerBudget>,
        output_accuracy: Option<Box<dyn native::engineering_accuracy::KktOutputObserver>>,
    ) -> Result<SolveReport, MathRuntimeError> {
        let AlgebraicCase {
            prepared,
            case,
            values,
            providers,
            sensitivity,
            ..
        } = case;
        // The parametric callbacks of a sensitivity request (Plan 22 S1), with their
        // evaluator's owner and charge; callbacks that cannot be built withhold the
        // sensitivities and never refuse the solve.
        let available = sensitivity
            .as_ref()
            .and_then(ParametricPreparation::available);
        let scope = run.execution.scope()?;
        let build_parametric = |program: &SensitivityProgram| {
            self.worker(program.program.clone(), &providers, scope.clone(), budget)
                .map_err(MathRuntimeError::into_problem)
                .and_then(
                    |ExecutionWorker {
                         worker,
                         _case,
                         _charge,
                     }| {
                        Ok((program.request(worker, &values)?, (_case, _charge)))
                    },
                )
        };
        // Optional Root analysis must not consume the base solver's resource allowance.
        // Construct its worker only after the original root is solved and qualified.
        let parametric = if profile.intent == SolveIntent::Root {
            None
        } else {
            available.map(&build_parametric)
        };
        let ExecutionWorker {
            worker,
            _case,
            _charge,
        } = self.case_worker(case, providers.clone(), &scope, budget)?;
        let plan = &prepared.prepared.plan;
        let initial: Vec<_> = plan.columns().iter().map(|id| values.scalars[id]).collect();
        let _root_support =
            budget.charge(native::assembled::AlgebraicOracle::root_support_allowance(
                plan,
                &prepared.prepared.structure,
            )?)?;
        let mut oracle = native::assembled::AlgebraicOracle::new(worker, values.clone())?
            .with_structural_analysis(prepared.prepared.structure.clone())?
            .with_presolve_facts(prepared.prepared.presolve.clone())?
            .with_normalization(run.normalization.clone())?;
        if let Some(c) = &prepared.prepared.coefficients {
            oracle = oracle.with_coefficient_facts(c)?;
        }
        if run.adapter.backend() == Backend::Pounce
            && profile.composition.policy == pse_model::strategy::CompositionPolicy::Auto
            && let Some(separator) = prepared
                .compiled()
                .automatic_separator(&scope.cancellation().clone())?
        {
            let ordinal = |id: &pse_ids::SemanticId, ids: &[pse_ids::SemanticId]| {
                ids.iter()
                    .position(|candidate| candidate == id)
                    .ok_or_else(|| {
                        ProblemError::Contract(
                            "compiler separator outside original inventory".into(),
                        )
                    })
            };
            let row_ids = plan
                .structure()
                .rows()
                .iter()
                .map(|row| row.id)
                .collect::<Vec<_>>();
            oracle = oracle.with_solve_separator(native::SolveSeparator {
                variables: separator
                    .columns
                    .iter()
                    .map(|id| ordinal(id, plan.columns()))
                    .collect::<Result<_, _>>()?,
                rows: separator
                    .rows
                    .iter()
                    .map(|id| ordinal(id, &row_ids))
                    .collect::<Result<_, _>>()?,
            })?;
        }

        let sense = plan
            .structure()
            .objective()
            .map_or(ObjectiveSense::Minimize, |o| o.sense);
        let (request, unbuilt, parametric_owner) = match (parametric, available) {
            (Some(Ok((request, owner))), _) => (Some(request), None, Some(owner)),
            (Some(Err(cause)), Some(program)) => (None, Some(program.withheld(cause)), None),
            _ => (None, None, None),
        };
        let normalization = run.normalization.clone();
        let tolerance = run.tolerances.clone();
        // A retained root session keeps the original evaluator's allocation owner.
        let mut base_owner = Some((_case, _charge));
        let mut report = if kind == execution::Representation::Roots {
            oracle.admit_nle()?;
            execution::roots(
                run,
                retained,
                execution::Roots {
                    oracle: Box::new(oracle),
                    initial: &initial,
                    owner: base_owner
                        .take()
                        .map(|owner| -> Box<dyn std::any::Any> { Box::new(owner) }),
                },
            )?
        } else {
            execution::nlp(
                run,
                retained,
                execution::Nlp {
                    oracle: Box::new(oracle),
                    initial: &initial,
                    presolve: &profile.presolve,
                    intent: profile.intent,
                    sense,
                    limit: self.policy.worker_bytes / 256,
                    analysis: execution::Analysis {
                        sensitivity: request,
                        output_accuracy,
                        ..execution::Analysis::for_intent(profile.intent)
                    },
                },
            )?
        };
        // Response meaning follows Root intent, independently of the chosen base solver.
        // NLP Root routes submit no optimizing KKT sensitivity request.
        if profile.intent == SolveIntent::Root {
            self.root_response(
                &mut report,
                &prepared,
                sensitivity.as_ref(),
                &values,
                &providers,
                &scope,
                &normalization,
                &tolerance,
                budget,
            );
        }
        if unbuilt.is_some() {
            report.evidence.sensitivity = unbuilt;
        }
        // A kept advanced-step factor is charged to the job's allowance before the next
        // step runs; one the allowance cannot hold is released and recorded as not kept.
        if let Some(bytes) = retained.uncharged() {
            match budget.charge(bytes) {
                Ok(charge) => retained.charge(Box::new(charge)),
                Err(_) => {
                    retained.release();
                    if let Some(parametric) = &mut report.evidence.sensitivity {
                        parametric.retained = None;
                    }
                }
            }
        }
        // The case owner and enclosing job reservation outlive every native callback.
        drop(base_owner);
        drop(parametric_owner);
        Ok(report)
    }
    /// The common optional Root analysis after the selected adapter's original result
    /// has been qualified; no parameter worker or response storage precedes the base solve.
    #[expect(
        clippy::too_many_arguments,
        reason = "the shared postsolve analysis consumes the original prepared case, values, providers and resolved execution budgets"
    )]
    fn root_response(
        &self,
        report: &mut SolveReport,
        prepared: &Preparation,
        sensitivity: Option<&ParametricPreparation<SensitivityProgram>>,
        values: &CaseValues,
        providers: &BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        scope: &pse_kernels::ExecutionScope,
        normalization: &Normalization,
        tolerances: &Tolerances,
        budget: &Arc<WorkerBudget>,
    ) {
        use native::square_response::{self, SquareScope, Withheld};
        let plan = &prepared.prepared.plan;
        if let Some(ParametricPreparation::Available(program)) = sensitivity
            && program.retain
            && program.root_source.is_some()
        {
            report.evidence.root_predictor = Some(self.fresh_root_predictor(
                report,
                prepared,
                program,
                values,
                providers,
                scope,
                normalization,
                tolerances,
                budget,
            ));
        }
        if let Some(preparation) = sensitivity {
            let result = (|| {
                let candidate = report.candidate.as_ref().ok_or(Withheld::NoCandidate)?;
                if report.quality.as_ref().is_none_or(|q| !q.feasible()) {
                    return Err(Withheld::Infeasible(
                        "base root is not independently qualified".into(),
                    ));
                }
                let program = match preparation {
                    ParametricPreparation::Available(program) => program,
                    ParametricPreparation::Unavailable(cause) => return Err(cause.clone()),
                };
                let (mut request, _owner) = self
                    .worker(program.program.clone(), providers, scope.clone(), budget)
                    .map_err(MathRuntimeError::into_problem)
                    .and_then(
                        |ExecutionWorker {
                             worker,
                             _case,
                             _charge,
                         }| {
                            Ok((program.request(worker, values)?, (_case, _charge)))
                        },
                    )
                    .map_err(|cause| match cause {
                        ProblemError::Limit { .. } => Withheld::Memory,
                        ProblemError::Math(
                            pse_math::MathError::Limit(_)
                            | pse_math::MathError::ByteLimit { .. }
                            | pse_math::MathError::SlotLimit { .. }
                            | pse_math::MathError::WorkLimit { .. },
                        ) => Withheld::Memory,
                        _ => Withheld::Neighborhood(cause.to_string()),
                    })?;
                let n = candidate.primal.len();
                let np = program.parameters.len();
                let bytes = square_response::workspace_bytes(n, np)
                    .and_then(|b| {
                        b.checked_add(
                            request
                                .oracle
                                .jacobian_pattern()
                                .row_idx()
                                .len()
                                .checked_mul(size_of::<f64>())?,
                        )
                    })
                    .and_then(|b| b.checked_add(n.checked_add(np)?.checked_mul(size_of::<f64>())?))
                    .and_then(|b| b.checked_add(np.checked_mul(size_of::<pse_ids::SemanticId>())?))
                    .and_then(|b| b.checked_add(n.checked_mul(128)?.checked_add(256)?))
                    .ok_or(Withheld::Memory)?;
                let charge = budget.charge(bytes).map_err(|_| Withheld::Memory)?;
                let contract = native::assembled::contract(plan);
                let bounds = plan
                    .structure()
                    .rows()
                    .iter()
                    .map(|r| (r.lower, r.upper))
                    .collect::<Vec<_>>();
                request
                    .admit(&contract)
                    .map_err(|cause| Withheld::Numerical(cause.to_string()))?;
                let scope = SquareScope::admit(
                    &contract,
                    plan.jacobian_pattern(),
                    &bounds,
                    Some(&prepared.prepared.structure),
                )?;
                let parameters = program
                    .parameters
                    .iter()
                    .map(|(id, _)| *id)
                    .collect::<Vec<_>>();
                let scales = &program.normalization.variables[n..];
                let values = report
                    .observation
                    .as_ref()
                    .ok_or_else(|| Withheld::Infeasible("no fresh original observation".into()))?;
                let mut point = candidate.primal.clone();
                point.extend(program.parameters.iter().map(|(_, v)| *v));
                let response = square_response::response(
                    square_response::Request {
                        scope: &scope,
                        point: &candidate.primal,
                        residual_values: &values.values,
                        tolerances,
                        normalization,
                        parameters: &parameters,
                        parameter_scales: scales,
                        rank_tolerance: square_response::DEFAULT_RELATIVE_RANK_CUTOFF,
                        bytes,
                    },
                    || {
                        let entries = request.oracle.jacobian_pattern().row_idx().len();
                        let mut jac = vec![0.; entries];
                        request.oracle.jacobian(&point, &mut jac)?;
                        let pattern = request.oracle.jacobian_pattern();
                        let mut fx = faer::Mat::zeros(n, n);
                        let mut fp = faer::Mat::zeros(n, np);
                        for c in 0..n + np {
                            for k in pattern.col_range(c) {
                                let r = pattern.row_idx()[k];
                                if c < n {
                                    fx[(r, c)] = jac[k];
                                } else {
                                    fp[(r, c - n)] = jac[k];
                                }
                            }
                        }
                        Ok((fx, fp))
                    },
                )?;
                Ok(response.with_owner(Arc::new(charge)))
            })();
            report.evidence.root_response = Some(result);
        }
    }
    /// Factor the fresh original state Jacobian once, retaining sparse parameter
    /// partials for subsequent actions. This operation is independent of a native
    /// iteration setup and of the optional dense public response analysis.
    #[expect(
        clippy::too_many_arguments,
        reason = "the postsolve producer consumes the complete original case, scope and resolved physical budgets"
    )]
    fn fresh_root_predictor(
        &self,
        report: &SolveReport,
        prepared: &Preparation,
        program: &SensitivityProgram,
        values: &CaseValues,
        providers: &BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        scope: &pse_kernels::ExecutionScope,
        normalization: &Normalization,
        tolerances: &Tolerances,
        budget: &Arc<WorkerBudget>,
    ) -> Result<native::square_response::SparsePredictor, native::square_response::Withheld> {
        use native::square_response::{
            Withheld,
        };
        let cause = |error: ProblemError| Withheld::Cause(Arc::new(error));
        scope.check().map_err(ProblemError::from).map_err(cause)?;
        let candidate = report.candidate.as_ref().ok_or(Withheld::NoCandidate)?;
        if report.quality.as_ref().is_none_or(|q| !q.feasible())
            || report.callback_failure().is_some()
            || report.validation_failure().is_some()
        {
            return Err(Withheld::Infeasible("fresh predictor requires independent original feasibility without a terminal cause".into()));
        }
        let observed = report
            .observation
            .as_ref()
            .ok_or_else(|| Withheld::Infeasible("fresh predictor lacks original values".into()))?;
        self.fresh_root_predictor_at(&candidate.primal,&observed.values,prepared,program,values,providers,scope,normalization,tolerances,budget)
    }
    #[expect(clippy::too_many_arguments,reason="original factor producer consumes the complete admitted source and task")]
    fn fresh_root_predictor_at(&self,primal:&[f64],observed:&[f64],prepared:&Preparation,program:&SensitivityProgram,values:&CaseValues,providers:&BTreeMap<pse_kernels::ProviderKey,pse_kernels::Registration>,scope:&pse_kernels::ExecutionScope,normalization:&Normalization,tolerances:&Tolerances,budget:&Arc<WorkerBudget>)->Result<native::square_response::SparsePredictor,native::square_response::Withheld>{
        use native::square_response::{SparseFactor,SparsePredictor,SparseRequest,SquareScope,Withheld};
        let cause=|error:ProblemError|Withheld::Cause(Arc::new(error));
        scope.check().map_err(ProblemError::from).map_err(cause)?;
        let plan = &prepared.compiled().plan;
        let n = plan.columns().len();
        let np = program.parameters.len();
        let bytes = SparseFactor::allowance(n)
            .and_then(|b| {
                b.checked_add(
                    program
                        .program
                        .assembly
                        .jacobian_pattern()
                        .compute_nnz()
                        .checked_mul(64)?,
                )
            })
            .and_then(|b| b.checked_add(n.checked_add(np)?.checked_mul(256)?))
            .ok_or(Withheld::Memory)?;
        let charge = budget
            .charge(bytes)
            .map_err(|error| cause(error.into_problem()))?;
        let ExecutionWorker {
            worker,
            _case,
            _charge,
        } = self
            .worker(program.program.clone(), providers, scope.clone(), budget)
            .map_err(|e| cause(e.into_problem()))?;
        let mut request = program.request(worker, values).map_err(cause)?;
        let contract = native::assembled::contract(plan);
        request.admit(&contract).map_err(cause)?;
        let row_bounds = plan
            .structure()
            .rows()
            .iter()
            .map(|r| (r.lower, r.upper))
            .collect::<Vec<_>>();
        let square = SquareScope::admit(
            &contract,
            plan.jacobian_pattern(),
            &row_bounds,
            Some(&prepared.compiled().structure),
        )?;
        let mut point = primal.to_vec();
        point.extend(program.parameters.iter().map(|(_, v)| *v));
        let mut jacobian = vec![0.; request.oracle.jacobian_pattern().compute_nnz()];
        budget
            .evaluate(|| {
                request
                    .oracle
                    .jacobian(&point, &mut jacobian)
                    .map_err(Into::into)
            })
            .map_err(|error| cause(error.into_problem()))?;
        scope.check().map_err(ProblemError::from).map_err(cause)?;
        let pattern = request.oracle.jacobian_pattern();
        let mut state_entries = Vec::new();
        let mut parameter_entries = Vec::new();
        for column in 0..n + np {
            for entry in pattern.col_range(column) {
                let row = pattern.row_idx()[entry];
                let value = jacobian[entry];
                if column < n {
                    state_entries.push(faer::sparse::Triplet::new(row, column, value));
                } else {
                    parameter_entries.push(faer::sparse::Triplet::new(row, column - n, value));
                }
            }
        }
        let state = faer::sparse::SparseColMat::try_new_from_triplets(n, n, &state_entries)
            .map_err(|e| {
                cause(ProblemError::numerical(format!(
                    "fresh original state partials: {e:?}"
                )))
            })?;
        let parameters =
            faer::sparse::SparseColMat::try_new_from_triplets(n, np, &parameter_entries).map_err(
                |e| {
                    cause(ProblemError::numerical(format!(
                        "fresh original parameter partials: {e:?}"
                    )))
                },
            )?;
        let mut key = program.root_source.ok_or_else(|| {
            cause(ProblemError::internal(
                "retained Root predictor lacks original dependencies",
            ))
        })?;
        key.point = Some(native::square_response::point_key(primal));
        let mut execution = Execution::within(
            scope.cancellation().clone(),
            &Controls::default(),
            scope.clone(),
        )
        .map_err(cause)?;
        execution.work_admission = budget
            .admission()
            .map(|owner| -> Arc<dyn WorkAdmission> { owner });
        let owner: Arc<dyn pse_math::AllocationOwner> = Arc::new(charge);
        let factor = SparseFactor::prepare(
            SparseRequest {
                scope: &square,
                point: primal,
                values: observed,
                tolerances,
                normalization,
                key,
                bytes,
            },
            || Ok(state),
            &execution,
        )?
        .with_owner(owner.clone());
        Ok(
            SparsePredictor::new(factor, program.parameters.clone(), parameters)
                .map_err(cause)?
                .with_owner(owner),
        )
    }
    /// Rebuild a previously qualified factor from its exact portable original point.
    /// This is numerical preparation, called only under the claimed target's task.
    pub(crate) async fn restore_root_predictor(self:&Arc<Self>,source:PreparedSolve,primal:Vec<f64>,observed:Vec<f64>,key:pse_model::strategy::SemanticProductKey,scope:pse_kernels::ExecutionScope,admission:Option<Arc<super::strategy::admission::TaskAdmission>>,driver:&crate::CancelSource)->Result<native::square_response::SparsePredictor,MathRuntimeError>{
        let source=source.retaining_factor()?;
        if source.semantic_point_key(&primal)?!=key{return Err(ProblemError::Contract("retained root factor source identity differs".into()).into());}
        let Representation::Algebraic(case)=&source.representation else{return Err(ProblemError::Unsupported("retained root factor requires original algebraic source".into()).into());};
        let Some(ParametricPreparation::Available(program))=&case.sensitivity else{return Err(ProblemError::Unsupported("retained root factor lacks original parametric program".into()).into());};
        let bytes=native::square_response::SparseFactor::allowance(primal.len()).and_then(|n|n.checked_add(program.program.assembly.numeric_worker_bytes())).and_then(|n|n.checked_add(program.program.assembly.jacobian_pattern().compute_nnz().checked_mul(64)?)).and_then(|n|n.checked_add(primal.len().checked_add(program.parameters.len())?.checked_mul(256)?)).ok_or(MathRuntimeError::Limit("retained root factor extent"))?;
        let budget=WorkerBudget::new(bytes);let budget=admission.map_or_else(||budget.clone(),|owner|budget.with_admission(owner));
        let service=self.clone();let worker_scope=scope.clone();let stop=FlightCancellation::default();let cancel=stop.clone();
        let operation=self.job_retained_scoped(1,bytes,stop,scope.deadline(),move |_|{
            let Representation::Algebraic(case)=&source.representation else{return Err(ProblemError::Internal("retained factor source changed".into()).into());};
            let Some(ParametricPreparation::Available(program))=&case.sensitivity else{return Err(ProblemError::Internal("retained parametric source changed".into()).into());};
            let predictor=service.fresh_root_predictor_at(&primal,&observed,&case.prepared,program,&case.values,&case.providers,&worker_scope,&source.normalization,&source.tolerances,&budget).map_err(|error|match error{native::square_response::Withheld::Cause(cause)=>ProblemError::Math(pse_math::MathError::Typed{retained:cause.retained_bytes(),cause:pse_model::diagnostic::DiagnosticCause::from_shared(cause)}),native::square_response::Withheld::Memory=>ProblemError::memory("retained root factor allowance"),native::square_response::Withheld::Numerical(detail)=>ProblemError::numerical(detail),other=>ProblemError::Unsupported(format!("retained root factor withheld: {other:?}"))})?;
            if predictor.factor().key()!=key{return Err(ProblemError::Contract("rebuilt root factor differs from recorded qualification".into()).into());}
            Ok((predictor,bytes))
        });tokio::pin!(operation);
        let result=tokio::select!{result=&mut operation=>result,()=driver.cancelled()=>{scope.cancellation().store(true,std::sync::atomic::Ordering::Release);cancel.cancel();let _=operation.await;Err(MathRuntimeError::Cancelled)}};
        result.map(|(predictor,owner)|predictor.with_owner(owner))
    }
    /// Worker-scoped providers and one attempt-local evaluator on this thread.
    fn case_worker(
        &self,
        case: Option<Arc<ExecutableCase>>,
        providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        scope: &pse_kernels::ExecutionScope,
        budget: &Arc<WorkerBudget>,
    ) -> Result<ExecutionWorker, MathRuntimeError> {
        let case =
            case.ok_or_else(|| ProblemError::Internal("missing executable representation".into()))?;
        self.worker(case, &providers, scope.clone(), budget)
    }
    /// All-fixed original evaluation without a native attempt.
    fn constant(
        &self,
        representation: Representation,
        tolerances: &Tolerances,
        scope: &pse_kernels::ExecutionScope,
        budget: &Arc<WorkerBudget>,
    ) -> Result<Outcome, MathRuntimeError> {
        let Representation::Algebraic(AlgebraicCase {
            prepared,
            case,
            values,
            providers,
            ..
        }) = representation
        else {
            return Err(
                ProblemError::Internal("constant route needs an algebraic case".into()).into(),
            );
        };
        let complete_work = providers.is_empty();
        let ExecutionWorker {
            mut worker,
            _case,
            _charge,
        } = self.case_worker(case, providers, scope, budget)?;
        let structure = prepared.prepared.plan.structure();
        let objective = structure
            .objective()
            .map(|o| {
                budget.evaluate(|| {
                    worker
                        .objective(&values)
                        .map(|v| v * o.sense.sign())
                        .map_err(Into::into)
                })
            })
            .transpose()?;
        let constraints = budget.evaluate(|| worker.constraints(&values).map_err(Into::into))?;
        let rows = structure
            .rows()
            .iter()
            .zip(&constraints)
            .zip(&tolerances.rows)
            .map(|((r, v), t)| Violation {
                id: r.id,
                physical: quality::interval(*v, r.lower, r.upper),
                tolerance: *t,
            })
            .collect();
        let sources = worker.constraint_sources()?;
        let mut observation = quality::Observation::from_values(
            objective,
            constraints,
            structure
                .rows()
                .iter()
                .map(|r| (r.lower, r.upper))
                .collect(),
        )?;
        observation.sources = sources;
        Ok(Outcome::Constant(Box::new(ConstantReport {
            owner: None,
            components: Vec::new(),
            objective,
            observation,
            quality: Quality::new(rows, vec![], vec![])?,
            coordinates: vec![],
            certified_reconstruction: None,
            work: if complete_work {
                WorkEvidence {
                    evaluations: Some(1 + u64::from(objective.is_some())),
                    iterations: Some(0),
                    factorizations: Some(0),
                    proof_steps: Some(0),
                }
            } else {
                WorkEvidence::default()
            },
        })))
    }
}
/// Outcome with its original admitted execution, absent for a pre-attempt refusal.
pub(crate) type ScopedOutcome = (Outcome, Option<pse_kernels::ExecutionScope>);

/// A step admitted for native work: its seed, execution controls and start receipt.
struct Admitted {
    step: PreparedSolve,
    chosen: Option<WarmStart>,
    execution: Execution,
    receipt: StartReceipt,
    normalization: pse_ids::ContentHash,
}
/// One independent step of a batch (Plan 22 N5), with its attempt and result owner.
#[derive(Debug)]
pub(crate) struct BatchMember {
    /// The bound step.
    pub step: PreparedSolve,
    /// Its attempt in the run.
    pub attempt: usize,
    /// The owner of its result.
    pub owner: Arc<pse_columnar::AllocationLease>,
}
/// The original compiled case, evaluated fresh at a coefficient candidate.
struct OriginalCase<'a> {
    service: &'a MathService,
    case: Option<Arc<ExecutableCase>>,
    providers: &'a BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
    values: &'a CaseValues,
    plan: &'a pse_math::assembly::CasePlan,
    scope: pse_kernels::ExecutionScope,
    budget: &'a Arc<WorkerBudget>,
}
impl execution::OriginalModel for OriginalCase<'_> {
    fn evaluate(&mut self, primal: &[f64]) -> Result<execution::Evaluation, ProblemError> {
        (|| -> Result<_, MathRuntimeError> {
            let case = self.case.clone().ok_or_else(|| {
                ProblemError::Internal("missing original coefficient evaluator".into())
            })?;
            let mut original =
                self.service
                    .worker(case, self.providers, self.scope.clone(), self.budget)?;
            let mut trial = self.values.clone();
            for (id, value) in self.plan.columns().iter().zip(primal) {
                trial.scalars.insert(*id, *value);
            }
            let constraints = original.worker.constraints(&trial)?;
            let objective = self
                .plan
                .structure()
                .objective()
                .map(|o| {
                    original
                        .worker
                        .objective(&trial)
                        .map(|v| v * o.sense.sign())
                })
                .transpose()?;
            let sources = original.worker.constraint_sources()?;
            Ok(execution::Evaluation {
                constraints,
                objective,
                sources,
            })
        })()
        .map_err(MathRuntimeError::into_problem)
    }
}

/// Complete effective request identity, distinct from native session compatibility: the
/// session profile, every control through serde (F09), the selection, whose backend is
/// named by its registry spelling, and the linked native build (library versions, image
/// manifest and numerical contract, ADR-0108 item 14).
pub(crate) fn profile_key(p: &SolverProfile) -> Result<pse_ids::roles::ProfileHash, ProblemError> {
    let mut h = FramedHasher::new(pse_ids::Frame::SolverProfileV5);
    hash_session(&mut h, p)?;
    pse_ids::document::frame(&mut h, &p.composition)
        .map_err(|error| ProblemError::Internal(error.to_string()))?;
    pse_ids::document::frame(&mut h, &p.reconstruction)
        .map_err(|error| ProblemError::Internal(error.to_string()))?;
    h.hash(&p.controls.identity()?)
        .hash(&execution::LINKED.build_identity());
    match p.selection {
        SolverSelection::Auto => {
            h.str("auto");
        }
        SolverSelection::Explicit(b) => {
            h.str(b.as_str());
        }
    }
    // A sensitivity request changes what the step computes and publishes, not how it
    // solves; a profile without one keeps its identity.
    if let Some(request) = &p.sensitivity {
        h.str("sensitivity")
            .bool(request.reduced_hessian)
            .u64(request.parameters.len() as u64);
        for parameter in &request.parameters {
            h.id(parameter);
        }
        // A propagation is identified by its complete serde encoding (F09); a request
        // without one keeps its identity.
        if let Some(propagation) = &request.propagation {
            h.str("propagation");
            pse_ids::document::frame(&mut h, propagation)
                .map_err(|e| ProblemError::Internal(e.to_string()))?;
        }
    }
    Ok(pse_ids::roles::ProfileHash::from_id(h.finish_hash()))
}

#[cfg(test)]
mod preparation_tests;

#[cfg(all(test, feature = "solver-pounce"))]
mod native_recovery_tests;
