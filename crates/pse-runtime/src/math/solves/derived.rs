// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Prepared mathematical families executed by the existing native strategy worker.
//! Auxiliary observations remain distinct from independent original completion.
use super::*;
use pse_backend_native::{
    NlpOracle, OracleContract, Variable,
    derived::{
        FamilyRoot, FeasibilityOracle, NlpBridge, ReconstructionAccuracy, ReducedOracle,
        ReducedRow, RootBridge,
    },
};
use pse_kernels::{DerivativeOrder, ExecutionScope, ProviderFactory};
use pse_math::{
    composite_reconstruction::CompositeReconstruction,
    derived::{
        self as math, DerivativeSupport, DerivedFamily, MassBinding, MassStructure,
        OriginalContract, OriginalObligations,
    },
    implicit::reconstruction::{
        ReconstructionFactory, SelectedImplicitReconstruction, SelectedResidualBinding,
        SelectedResidualRealization, SupplierFactory,
    },
    index::{Entry, GlobalCol, GlobalRow},
    sparse::AssemblyMatrix,
};
use pse_model::strategy::{AccuracyEvidence, MechanismKind, ProfileRef, WorkObservation};
use std::{cell::RefCell, collections::BTreeSet, rc::Rc, time::Instant};

#[path = "derived_accuracy.rs"]
mod certified;
pub(crate) use certified::CertifiedReconstructionPoint;

impl PreparedSolve {
    /// The actual current compiler coefficient projection, lowered by the native owner.
    /// Its lease admits the copied coefficient buffers and native coordinate metadata.
    pub(crate) fn convex_qp_problem(
        &self,
        service: &MathService,
        execution: &Execution,
    ) -> Result<
        (
            native::CoefficientProblem,
            Arc<pse_columnar::AllocationLease>,
        ),
        MathRuntimeError,
    > {
        let Representation::Algebraic(case) = &self.representation else {
            return Err(ProblemError::Contract(
                "QP prediction requires an original algebraic coefficient source".into(),
            )
            .into());
        };
        // Coefficient extraction is demanded proposal production, including its
        // original finite proof allowance. Unknown proof totals refuse a hard cap
        // through the same execution owner before construction starts.
        let source = &case.prepared.prepared;
        let _class_owner = source
            .coefficients
            .is_none()
            .then(|| service.reserve("math:qp-class-production", service.policy.workspace_bytes))
            .transpose()?;
        let classified = if source.coefficients.is_none() {
            let product = execution.counted(
                WorkEvidence {
                    evaluations: Some(0),
                    iterations: Some(0),
                    factorizations: Some(0),
                    proof_steps: None,
                },
                || Ok(source.prepare_class(&case.values, &execution.cancel)),
            )??;
            if product.retained_bytes() > service.policy.workspace_bytes {
                return Err(MathRuntimeError::Limit("QP class production workspace"));
            }
            Some(product)
        } else {
            None
        };
        let projection = classified.as_ref().unwrap_or(source);
        let coefficients = projection.coefficients.as_ref().ok_or_else(|| {
            ProblemError::Contract(
                "QP prediction requires the compiler's exact quadratic coefficient projection"
                    .into(),
            )
        })?;
        if !coefficients.matches_values(&case.values) {
            return Err(ProblemError::Contract(
                "QP coefficient projection has different fixed or parameter values".into(),
            )
            .into());
        }
        let plan = &case.prepared.prepared.plan;
        let entries = plan
            .columns()
            .len()
            .checked_add(plan.structure().rows().len())
            .ok_or(MathRuntimeError::Limit("QP coefficient coordinate extent"))?;
        let bytes = coefficients
            .retained_bytes()
            .checked_add(
                entries
                    .checked_mul(
                        size_of::<Variable>()
                            + size_of::<pse_ids::SemanticId>()
                            + 3 * size_of::<f64>(),
                    )
                    .ok_or(MathRuntimeError::Limit("QP coefficient coordinate bytes"))?,
            )
            .ok_or(MathRuntimeError::Limit("QP coefficient projection bytes"))?;
        let owner = service.reserve("math:qp-coefficient-source", bytes)?;
        let problem = native::CoefficientProblem::from_plan(plan, coefficients.as_ref().clone())?;
        let certificate: Option<&dyn QuadraticEvidence> = classified
            .as_ref()
            .and_then(|product| product.facts.convexity.convex_quadratic())
            .map(|certificate| -> &dyn QuadraticEvidence { certificate.as_ref() })
            .or(case.certificate.as_deref());
        execution.counted(
            WorkEvidence {
                evaluations: Some(0),
                iterations: Some(0),
                factorizations: Some(0),
                proof_steps: None,
            },
            || problem.validate_convex(certificate),
        )?;
        if !problem.objectives.is_empty()
            || problem.domains.iter().any(|domain| {
                *domain != pse_model::generated::enums::ModelingVariableDomain::Continuous
            })
        {
            return Err(ProblemError::Contract(
                "QP prediction requires one continuous convex objective".into(),
            )
            .into());
        }
        Ok((problem, owner))
    }
}

/// One heterogeneous rung of the single declared strategy.
#[derive(Clone, Debug)]
pub enum PreparedRung {
    /// Original source whose completion may grant scientific permission.
    Original(Box<PreparedSolve>),
    /// Compiler-issued complete original structural schedule.
    #[cfg(feature = "solver-kinsol")]
    Blocks(blocks::PreparedBlocks),
    /// Authored-topology causal sweep with complete original reconstruction.
    #[cfg(feature = "solver-kinsol")]
    Causal(causal::PreparedCausal),
    /// One fresh specification coordinated by the common original solver/assessment.
    Multistart(multistart::PreparedMultistart),
    /// Auxiliary family whose result can supply an original-coordinate proposal.
    Derived(PreparedDerived),
    /// Actual retained library model-management phase producing an original-screened start.
    Surrogate(crate::math::surrogate::PreparedSurrogate),
    /// Actual source-bound PETSc artificial flow or declared block product.
    #[cfg(feature = "solver-petsc")]
    Petsc(petsc::PreparedPetsc),
    /// Source-bound continuation with an origin issued by original completion.
    Path {
        /// Frozen source, declared target and finite corrector operation.
        prepared: paths::PreparedPath,
        /// Actual original permission and explicitly oriented origin.
        start: Box<paths::PathStart>,
    },
}
impl From<PreparedSolve> for PreparedRung {
    fn from(value: PreparedSolve) -> Self {
        Self::Original(Box::new(value))
    }
}
impl From<PreparedDerived> for PreparedRung {
    fn from(value: PreparedDerived) -> Self {
        Self::Derived(value)
    }
}
impl PreparedRung {
    /// Extra heap bodies introduced by enum indirection. Their referenced products
    /// retain independent owners; this excludes those products' arrays and maps.
    pub(crate) fn boxed_payload_bytes(&self) -> usize {
        match self {
            Self::Original(prepared) => size_of_val(prepared.as_ref()),
            Self::Path { start, .. } => size_of_val(start.as_ref()),
            _ => 0,
        }
    }
    /// Frozen original scientific source identity.
    pub fn original_identity(&self) -> Result<pse_ids::ContentHash, ProblemError> {
        match self {
            #[cfg(feature = "solver-kinsol")]
            Self::Blocks(p) => p.original().original_identity(),
            #[cfg(feature = "solver-kinsol")]
            Self::Causal(p) => p.original().original_identity(),
            Self::Surrogate(p) => p.original_target().original_identity(),
            Self::Multistart(p) => p.original_identity(),
            Self::Original(p) => p.original_identity(),
            Self::Derived(p) => Ok(p.original_identity),
            Self::Path { prepared, .. } => prepared.original_target().original_identity(),
            #[cfg(feature = "solver-petsc")]
            Self::Petsc(p) => Ok(p.original_identity()),
        }
    }
    /// Exact declared solver profile identity.
    pub fn strategy_profile(&self) -> Result<pse_ids::ContentHash, ProblemError> {
        match self {
            #[cfg(feature = "solver-kinsol")]
            Self::Blocks(p) => Ok(p.key()),
            #[cfg(feature = "solver-kinsol")]
            Self::Causal(p) => Ok(p.key()),
            Self::Surrogate(p) => Ok(p.key()),
            Self::Multistart(p) => p.strategy_profile(),
            Self::Original(p) => p.strategy_profile(),
            Self::Derived(p) => p.strategy_profile(),
            Self::Path { prepared, .. } => Ok(profile_key(prepared.profile())?.as_id()),
            #[cfg(feature = "solver-petsc")]
            Self::Petsc(p) => p.strategy_profile(),
        }
    }
    pub(crate) fn backend(&self) -> Option<Backend> {
        match self {
            #[cfg(feature = "solver-kinsol")]
            Self::Blocks(p) => Some(p.backend()),
            #[cfg(feature = "solver-kinsol")]
            Self::Causal(p) => Some(p.backend()),
            Self::Surrogate(_) => None,
            Self::Multistart(p) => Some(p.backend()),
            Self::Original(p) => p.backend(),
            Self::Derived(p) => (!p.complete_reconstruction()).then_some(p.backend),
            Self::Path { .. } => Some(Backend::Ipopt),
            #[cfg(feature = "solver-petsc")]
            Self::Petsc(p) => Some(p.backend()),
        }
    }
    pub(crate) fn threads(&self) -> usize {
        match self {
            #[cfg(feature = "solver-kinsol")]
            Self::Blocks(p) => p.original().threads(),
            #[cfg(feature = "solver-kinsol")]
            Self::Causal(p) => p.original().threads(),
            Self::Surrogate(p) => p.threads(),
            Self::Multistart(p) => p.profile().controls.threads,
            Self::Original(p) => p.threads(),
            Self::Derived(p) => p.threads(),
            Self::Path { prepared, .. } => prepared.profile().controls.threads,
            #[cfg(feature = "solver-petsc")]
            Self::Petsc(p) => p.threads(),
        }
    }
    pub(crate) fn declared_foreign_bytes(&self) -> usize {
        match self {
            #[cfg(feature = "solver-kinsol")]
            Self::Blocks(p) => p.original().declared_foreign_bytes(),
            #[cfg(feature = "solver-kinsol")]
            Self::Causal(p) => p.original().declared_foreign_bytes(),
            Self::Surrogate(_) => 0,
            Self::Multistart(p) => p.profile().controls.foreign_bytes.unwrap_or(0),
            Self::Original(p) => p.declared_foreign_bytes(),
            Self::Derived(p) => p.declared_foreign_bytes(),
            Self::Path { prepared, .. } => prepared.profile().controls.foreign_bytes.unwrap_or(0),
            #[cfg(feature = "solver-petsc")]
            Self::Petsc(p) => p.declared_foreign_bytes(),
        }
    }
    /// Actual preparation request identity.
    pub fn request_identity(&self) -> Result<pse_ids::roles::LineageRequestHash, ProblemError> {
        match self {
            #[cfg(feature = "solver-kinsol")]
            Self::Blocks(p) => Ok(pse_ids::roles::LineageRequestHash::from(p.key())),
            #[cfg(feature = "solver-kinsol")]
            Self::Causal(p) => Ok(pse_ids::roles::LineageRequestHash::from(p.key())),
            Self::Surrogate(p) => Ok(pse_ids::roles::LineageRequestHash::from(p.key())),
            Self::Multistart(p) => Ok(pse_ids::roles::LineageRequestHash::from(p.key())),
            Self::Original(p) => p.request_identity(),
            Self::Derived(p) => Ok(pse_ids::roles::LineageRequestHash::from(p.key)),
            Self::Path { prepared, .. } => {
                Ok(pse_ids::roles::LineageRequestHash::from(prepared.key()))
            }
            #[cfg(feature = "solver-petsc")]
            Self::Petsc(p) => Ok(pse_ids::roles::LineageRequestHash::from(p.key())),
        }
    }
    /// Actual source and mathematical preparation identity.
    pub fn preparation_identity(&self) -> Result<pse_ids::ContentHash, ProblemError> {
        match self {
            #[cfg(feature = "solver-kinsol")]
            Self::Blocks(p) => Ok(p.key()),
            #[cfg(feature = "solver-kinsol")]
            Self::Causal(p) => Ok(p.key()),
            Self::Surrogate(p) => p.original_target().preparation_identity(),
            Self::Multistart(p) => p.original_target().preparation_identity(),
            Self::Original(p) => p.preparation_identity(),
            Self::Derived(p) => Ok(p.key),
            Self::Path { prepared, .. } => prepared.original_target().preparation_identity(),
            #[cfg(feature = "solver-petsc")]
            Self::Petsc(p) => Ok(p.source().preparation_identity()),
        }
    }
    pub(crate) fn result_bytes(&self) -> Result<usize, MathRuntimeError> {
        match self {
            #[cfg(feature = "solver-kinsol")]
            Self::Blocks(p) => p.result_bytes(),
            #[cfg(feature = "solver-kinsol")]
            Self::Causal(p) => p.result_bytes(),
            Self::Surrogate(p) => Ok(p.result_bytes()),
            Self::Multistart(p) => p.result_bytes(),
            Self::Original(p) => p.result_bytes(),
            Self::Derived(p) => p.result_bytes(),
            Self::Path { prepared, .. } => prepared.result_bytes(),
            #[cfg(feature = "solver-petsc")]
            Self::Petsc(p) => p.result_bytes(),
        }
    }
    pub(crate) fn entry_origin(&self, has_previous: bool) -> pse_model::strategy::StartOrigin {
        match self {
            #[cfg(feature = "solver-kinsol")]
            Self::Blocks(p) => p.original().entry_origin(has_previous),
            #[cfg(feature = "solver-kinsol")]
            Self::Causal(p) => p.original().entry_origin(has_previous),
            Self::Surrogate(p) => p.original_target().entry_origin(has_previous),
            Self::Multistart(p) => p.origin(),
            Self::Original(p) => p.entry_origin(has_previous),
            Self::Derived(p) => p.original.entry_origin(has_previous),
            Self::Path { .. } => pse_model::strategy::StartOrigin::Accepted,
            #[cfg(feature = "solver-petsc")]
            Self::Petsc(p) => p.source().original().entry_origin(has_previous),
        }
    }
    /// Callable producer identities consumed by strategy admission.
    pub fn support(&self) -> Result<BTreeSet<pse_ids::ContentHash>, ProblemError> {
        match self {
            #[cfg(feature = "solver-kinsol")]
            Self::Blocks(p) => PreparedRung::Original(Box::new(p.original().clone())).support(),
            #[cfg(feature = "solver-kinsol")]
            Self::Causal(p) => PreparedRung::Original(Box::new(p.original().clone())).support(),
            Self::Surrogate(p) => Ok(p.support()),
            Self::Multistart(p) => p.support(),
            Self::Derived(p) => Ok(p.support()),
            Self::Path { prepared, .. } => Ok(prepared.support()),
            #[cfg(feature = "solver-petsc")]
            Self::Petsc(p) => Ok(p.support()),
            Self::Original(p) => {
                let mut keys = BTreeSet::from([p.original_identity()?, p.preparation_identity()?]);
                if let Representation::Algebraic(source) = &p.representation {
                    for artifact in source.prepared.compiled().artifacts.iter() {
                        keys.insert(artifact.key());
                    }
                }
                Ok(keys)
            }
        }
    }
    /// Owner-issued prerequisites and finite production permissions for this rung.
    /// Native/source preparation is capability admission, never a future certificate.
    pub fn operation_contract(
        &self,
    ) -> Result<pse_model::strategy::OperationContract, ProblemError> {
        match self {
            Self::Derived(prepared) => prepared.operation_contract(),
            _ => Ok(pse_model::strategy::OperationContract::default()),
        }
    }
    /// Original caller task scope, when retained by an auxiliary preparation.
    pub fn task_scope(&self) -> Option<ExecutionScope> {
        match self {
            #[cfg(feature = "solver-kinsol")]
            Self::Blocks(p) => Some(p.scope().clone()),
            #[cfg(feature = "solver-kinsol")]
            Self::Causal(p) => Some(p.scope().clone()),
            Self::Surrogate(p) => Some(p.scope().clone()),
            Self::Multistart(p) => Some(p.scope().clone()),
            Self::Original(p) => p.task_scope.clone(),
            Self::Derived(p) => Some(p.scope.clone()),
            Self::Path { prepared, .. } => Some(prepared.task_scope().clone()),
            #[cfg(feature = "solver-petsc")]
            Self::Petsc(p) => Some(p.scope().clone()),
        }
    }
    pub(crate) fn clear_composition(&mut self) {
        if let Self::Original(p) = self {
            p.composition = None;
        }
    }
    pub(crate) fn admits_mechanism(&self, kind: MechanismKind) -> bool {
        match self {
            #[cfg(feature = "solver-kinsol")]
            Self::Blocks(_) => kind == MechanismKind::Block,
            #[cfg(feature = "solver-kinsol")]
            Self::Causal(_) => kind == MechanismKind::MapsAnderson,
            Self::Surrogate(_) => matches!(
                kind,
                MechanismKind::Surrogate | MechanismKind::Multifidelity
            ),
            Self::Multistart(_) => kind == MechanismKind::Multistart,
            Self::Original(_) => matches!(
                kind,
                MechanismKind::Direct
                    | MechanismKind::NativeGlobalization
                    | MechanismKind::SetupReuse
                    | MechanismKind::SelectedEvaluation
            ),
            Self::Derived(p) => kind == p.mechanism(),
            Self::Path { .. } => kind == MechanismKind::Continuation,
            #[cfg(feature = "solver-petsc")]
            Self::Petsc(p) => kind == p.mechanism(),
        }
    }
}
/// Mathematical realization declarations; strategy start/recovery policy lives once in
/// NumericalStrategy. Frozen mass coefficients are physical original callback actions.
#[derive(Clone, Debug)]
pub enum DerivedRequest {
    /// Preserve all original rows and bounds with an auxiliary constant objective.
    Feasibility,
    /// Explicit physical distance metric and named original coordinate roles.
    LeastDeviation {
        /// Finite reference point in original physical coordinates.
        center: Vec<f64>,
        /// Positive physical coordinate characteristic scales.
        scales: Vec<f64>,
        /// Positive dimensionless metric entries.
        weights: Vec<f64>,
        /// Original coordinate IDs permitted to move.
        free: Vec<pse_ids::SemanticId>,
        /// Original coordinate IDs fixed to their declared center.
        held: Vec<pse_ids::SemanticId>,
        /// Admitted physical metric producer identity.
        metric_source: pse_ids::ContentHash,
    },
    /// Fixed homotopy slice with original zero-residual terminal correspondence.
    AnchoredHomotopy {
        /// Original physical anchor coordinates.
        anchor: Vec<f64>,
        /// Declared continuation parameter in the closed unit interval.
        parameter: f64,
    },
    /// Frozen-mass shifted residual at one explicit artificial step.
    ShiftedPseudoTime {
        /// Original physical anchor coordinates.
        anchor: Vec<f64>,
        /// Positive artificial step size.
        step: f64,
        /// Declared residual/mass sign convention.
        sign: f64,
        /// Original state coordinate paired with each residual row.
        pairing: Vec<GlobalCol>,
        /// Actual immutable mass coefficients and incidence.
        mass: Arc<AssemblyMatrix>,
    },
    /// Selected implicit reconstruction composed with remaining original obligations.
    Reduced {
        /// Actual admitted local suppliers; their named rows/unknowns derive coverage.
        suppliers: Vec<ReducedSupplier>,
        /// Original coordinates exposed to the reduced native solver.
        retained: Vec<GlobalCol>,
        /// Explicit consumer point and derivative allowances.
        accuracy: ReconstructionAccuracy,
    },
}
/// One authored selected implicit supplier, retaining its source-owned admission.
#[derive(Clone, Debug)]
pub struct ReducedSupplier {
    /// Actual admitted factory; structural matching never creates this declaration.
    pub factory: Arc<ReconstructionFactory>,
    /// Existing reservation retained for the actual factory.
    pub factory_owner: Arc<pse_columnar::AllocationLease>,
    /// Source-owned validity interpretation.
    pub validity: pse_ids::ContentHash,
}
/// Actual mathematical product, retaining its distinct scientific correspondence.
#[derive(Clone, Debug)]
pub enum PreparedFamily {
    /// Existing structural family transformations and reconstruction contracts.
    General(Arc<DerivedFamily>),
    /// Analytic auxiliary objective with unchanged original constraints and held roles.
    LeastDeviation(Arc<pse_math::initialization::LeastDeviation>),
}
impl PreparedFamily {
    /// Actual mathematical producer identity, including the declared metric and roles.
    pub fn key(&self) -> pse_ids::ContentHash {
        match self {
            Self::General(f) => f.key(),
            Self::LeastDeviation(f) => f.key(),
        }
    }
    /// Complete physical original obligations, including the authored objective.
    pub fn original(&self) -> &OriginalContract {
        match self {
            Self::General(f) => f.original(),
            Self::LeastDeviation(f) => f.original(),
        }
    }
    /// Constraint derivative support from the actual original or reconstruction supplier.
    pub fn support(&self) -> DerivativeSupport {
        match self {
            Self::General(f) => {
                let mut support = f.support();
                support.order = support.order.min(DerivativeOrder::First);
                support
            }
            Self::LeastDeviation(f) => f.original().support(),
        }
    }
    fn general(&self) -> Result<&Arc<DerivedFamily>, ProblemError> {
        match self {
            Self::General(f) => Ok(f),
            Self::LeastDeviation(_) => Err(ProblemError::Contract(
                "distance family cannot be used as a general transformation".into(),
            )),
        }
    }
    fn coordinate_map(&self) -> Option<&[GlobalCol]> {
        match self {
            Self::General(f) => f.coordinate_map(),
            Self::LeastDeviation(_) => None,
        }
    }
    fn row_map(&self) -> Option<&[GlobalRow]> {
        match self {
            Self::General(f) => f.row_map(),
            Self::LeastDeviation(_) => None,
        }
    }
    fn incidence(&self) -> &[Entry<GlobalRow, GlobalCol>] {
        match self {
            Self::General(f) => f.incidence(),
            Self::LeastDeviation(f) => f.original().incidence(),
        }
    }
    fn reconstruction_contract(&self) -> Option<&math::ReconstructionContract> {
        match self {
            Self::General(f) => f.reconstruction_contract(),
            Self::LeastDeviation(_) => None,
        }
    }
    fn retained_bytes(&self) -> usize {
        match self {
            Self::General(f) => f.retained_bytes(),
            Self::LeastDeviation(f) => f.retained_bytes(),
        }
    }
    fn reconstruct(&self, x: &[f64]) -> Result<Vec<f64>, pse_math::MathError> {
        match self {
            Self::General(f) => f.reconstruct(x),
            Self::LeastDeviation(f) => {
                f.validate_point(x)?;
                Ok(x.to_vec())
            }
        }
    }
}
#[derive(Clone, Debug)]
struct ReducedPreparation {
    contract: Arc<math::ReconstructionContract>,
    suppliers: Vec<ReducedSupplierPreparation>,
}
#[derive(Clone, Debug)]
struct ReducedSupplierPreparation {
    contract: Arc<math::ReconstructionContract>,
    binding: SelectedResidualBinding,
    normalization: Normalization,
}
/// Actual immutable source/family/layout admission. No native session is built here.
#[derive(Clone, Debug)]
pub struct PreparedDerived {
    data: Arc<PreparedDerivedData>,
}
/// Immutable retained preparation storage; fields remain producer-owned.
#[derive(Debug)]
pub struct PreparedDerivedData {
    original: PreparedSolve,
    source: AlgebraicCase,
    physical: Arc<OriginalContract>,
    family: PreparedFamily,
    request: DerivedRequest,
    reduced: Option<ReducedPreparation>,
    profile: SolverProfile,
    backend: Backend,
    snapshot: execution::Snapshot,
    decision: routing::Decision,
    normalization: Normalization,
    tolerances: Tolerances,
    accuracy: ResolvedAccuracy,
    compatibility: Compatibility,
    original_identity: pse_ids::ContentHash,
    key: pse_ids::ContentHash,
    scope: ExecutionScope,
    operation_scope: ExecutionScope,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl std::ops::Deref for PreparedDerived {
    type Target = PreparedDerivedData;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl PreparedDerived {
    /// All original coordinates are supplied by admitted reconstruction; the outer
    /// operation is direct original evaluation and performs no native iteration.
    pub(crate) fn complete_reconstruction(&self) -> bool {
        matches!(&self.request, DerivedRequest::Reduced { retained, .. } if retained.is_empty())
    }
    /// Consumed source, family and declared profile identity.
    pub fn key(&self) -> pse_ids::ContentHash {
        self.key
    }
    /// Actual prepared mathematical producer.
    pub fn family(&self) -> &PreparedFamily {
        &self.family
    }
    /// Frozen original scientific source identity.
    pub fn original_identity(&self) -> pse_ids::ContentHash {
        self.original_identity
    }
    /// Unchanged original correction preparation.
    pub fn original(&self) -> &PreparedSolve {
        &self.original
    }
    /// Exact native backend and declared profile reference.
    pub fn profile_ref(&self) -> Result<ProfileRef, ProblemError> {
        Ok(ProfileRef {
            backend: self.backend,
            key: self.strategy_profile()?,
        })
    }
    /// Exact declared solver profile identity.
    pub fn strategy_profile(&self) -> Result<pse_ids::ContentHash, ProblemError> {
        Ok(profile_key(&self.profile)?.as_id())
    }
    /// Canonical strategy mechanism for this actual producer.
    pub fn mechanism(&self) -> MechanismKind {
        match self.request {
            DerivedRequest::Feasibility => MechanismKind::BoundedFeasibility,
            DerivedRequest::LeastDeviation { .. } => MechanismKind::LeastDeviation,
            DerivedRequest::AnchoredHomotopy { .. } => MechanismKind::Homotopy,
            DerivedRequest::ShiftedPseudoTime { .. } => MechanismKind::PseudoTransient,
            DerivedRequest::Reduced { .. } => MechanismKind::ReducedSpace,
        }
    }
    /// Callable producer identities consumed by strategy admission.
    pub fn support(&self) -> BTreeSet<pse_ids::ContentHash> {
        let mut result = BTreeSet::from([
            self.family.key(),
            self.physical.obligations().guards,
            self.physical.obligations().selection,
        ]);
        if self.family.support().jacobian_product {
            result.insert(self.family.support().source);
        }
        if let Some(r) = &self.reduced {
            result.insert(r.contract.source());
            result.insert(r.contract.support().source);
        }
        result
    }
    fn product_source(&self) -> Result<pse_model::strategy::SemanticProductKey, ProblemError> {
        Ok(pse_model::strategy::SemanticProductKey {
            structure: self.original.preparation_identity()?,
            binding: self.original_identity,
            numerical_policy: Some(self.original.numerics.policy.key()),
            normalization: Some(self.physical.normalization()),
            point: None,
            parameters: None,
            derivation: Some(self.family.key()),
            branch: None,
            accuracy: None,
        })
    }
    fn operation_contract(&self) -> Result<pse_model::strategy::OperationContract, ProblemError> {
        let DerivedRequest::Reduced { accuracy, .. } = &self.request else {
            return Ok(pse_model::strategy::OperationContract::default());
        };
        accuracy.validate()?;
        let source = self.product_source()?;
        let point = pse_model::strategy::ProductionDemand {
            source,
            derivative_order: 0,
            branch: self.original.profile.composition.branch,
            allowance: accuracy.point,
            class: accuracy.class,
        };
        point
            .validate()
            .map_err(|e| ProblemError::Contract(e.to_string()))?;
        let mut outputs = vec![point.clone()];
        if !self.complete_reconstruction() {
            outputs.push(pse_model::strategy::ProductionDemand {
                derivative_order: 1,
                allowance: accuracy.action,
                ..point
            });
        }
        Ok(pse_model::strategy::OperationContract {
            inputs: Vec::new(),
            outputs,
        })
    }
    /// Publish only evidence actually returned by the applied reconstruction worker.
    /// Point/order/class/dependencies remain independently checkable by each consumer.
    pub(crate) fn produced_evidence(
        &self,
        attempt: &DerivedAttempt,
        branch: pse_model::strategy::BranchPolicy,
    ) -> Result<Vec<pse_model::strategy::ProductEvidence>, ProblemError> {
        branch
            .validate()
            .map_err(|e| ProblemError::Contract(e.to_string()))?;
        if branch != self.original.profile.composition.branch {
            return Err(ProblemError::Contract(
                "derived product branch differs from original request".into(),
            ));
        }
        let declared = self.operation_contract()?;
        let source = self.product_source()?;
        let mut products = Vec::new();
        if let Some(proposal) = &attempt.proposal {
            if proposal.original != self.original_identity || proposal.family != self.family.key() {
                return Err(ProblemError::Contract(
                    "derived product source correspondence".into(),
                ));
            }
            if let Some(accuracy) = proposal.reconstruction_accuracy {
                let point = self
                    .original
                    .semantic_point_key(&proposal.coordinates)?
                    .point
                    .ok_or_else(|| {
                        ProblemError::Internal(
                            "original point owner omitted reconstruction point identity".into(),
                        )
                    })?;
                products.push(pse_model::strategy::ProductEvidence {
                    source: pse_model::strategy::SemanticProductKey {
                        point: Some(point),
                        accuracy: Some(accuracy.product),
                        ..source
                    },
                    derivative_order: 0,
                    branch,
                    accuracy,
                });
            }
        }
        if let Some((point, accuracy)) = attempt.applied_action {
            products.push(pse_model::strategy::ProductEvidence {
                source: pse_model::strategy::SemanticProductKey {
                    point: Some(point),
                    accuracy: Some(accuracy.product),
                    ..source
                },
                derivative_order: 1,
                branch,
                accuracy,
            });
        }
        if products
            .iter()
            .any(|product| !declared.outputs.iter().any(|demand| demand.admits(product)))
        {
            return Err(ProblemError::Contract(
                "applied reconstruction product violates producer output allowance".into(),
            ));
        }
        Ok(products)
    }
    /// Admission of the explicit auxiliary native operation.
    pub fn route_decision(&self) -> &routing::Decision {
        &self.decision
    }
    /// Original caller cancellation owner and submission deadline.
    pub fn scope(&self) -> &ExecutionScope {
        &self.scope
    }
    /// Already stamped auxiliary clock, which is never renewed at dispatch.
    pub fn operation_scope(&self) -> &ExecutionScope {
        &self.operation_scope
    }
    /// Actual local-clock admission refusal. An expired parent, cancellation or
    /// unrelated scope never becomes an optional local refusal.
    pub(crate) fn local_scope_refusal(
        &self,
        enclosing: &ExecutionScope,
    ) -> Option<Arc<ProblemError>> {
        if !Arc::ptr_eq(
            self.operation_scope.cancellation(),
            enclosing.cancellation(),
        ) || enclosing.check().is_err()
            || self.operation_scope.deadline()? >= enclosing.deadline()?
        {
            return None;
        }
        match self.operation_scope.check() {
            Err(error @ pse_kernels::ProviderError::Deadline) => {
                Some(Arc::new(ProblemError::Provider(error)))
            }
            _ => None,
        }
    }
    pub(crate) fn effect_failure(
        &self,
        cause: Arc<ProblemError>,
        observed: WorkObservation,
        enclosing: &ExecutionScope,
    ) -> super::super::strategy::EffectFailure {
        if genuine_time_cause(&cause) {
            super::super::strategy::EffectFailure::expired_scope(
                cause,
                observed,
                self.operation_scope(),
                enclosing,
            )
        } else {
            super::super::strategy::EffectFailure::observed(cause, observed)
        }
    }
    pub(crate) fn local_attempt_failure(
        &self,
        attempt: &DerivedAttempt,
        enclosing: &ExecutionScope,
    ) -> Option<super::super::strategy::EffectFailure> {
        let Outcome::Native(report) = &attempt.outcome else {
            return None;
        };
        let mut cause = None;
        let mut refinement = None;
        for recorded in [
            attempt.screening_failure.clone(),
            report.shared_validation_failure(),
            report
                .evidence
                .callback
                .terminal_failure
                .then(|| report.shared_callback_failure())
                .flatten(),
        ]
        .into_iter()
        .flatten()
        {
            if let Some((product, source_key, validity)) = refinement_cause(&recorded) {
                if attempt.refinement_product != Some(product)
                    || self.reduced.as_ref().is_none_or(|r| {
                        r.contract.source() != source_key || r.contract.validity() != validity
                    })
                {
                    return None;
                }
                refinement = Some(recorded);
                continue;
            }
            if !genuine_time_cause(&recorded) {
                return None;
            }
            cause = Some(recorded);
        }
        if let Some(cause) = refinement {
            return Some(super::super::strategy::EffectFailure::refinement(
                cause,
                attempt.work(),
                self.operation_scope(),
                enclosing,
            ));
        }
        let cause = match cause {
            Some(cause) => cause,
            None if report.termination.category == Termination::TimeLimit => {
                Arc::new(ProblemError::native(
                    native::NativeStatus {
                        backend: report.backend,
                        code: report.termination.code,
                        name: report.termination.name.clone(),
                    },
                    Termination::TimeLimit,
                    "derived native operation time limit",
                ))
            }
            None => return None,
        };
        Some(self.effect_failure(cause, attempt.work(), enclosing))
    }
    /// Declared admitted worker thread extent.
    pub fn threads(&self) -> usize {
        self.profile.controls.threads
    }
    /// Profile-declared foreign-library reservation.
    pub fn declared_foreign_bytes(&self) -> usize {
        self.profile.controls.foreign_bytes.unwrap_or(0)
    }
    /// Declared finite native operation controls.
    pub fn controls(&self) -> &Controls {
        &self.profile.controls
    }
    /// Mechanically transported original physical magnitudes.
    pub fn normalization(&self) -> &Normalization {
        &self.normalization
    }
    /// Frozen original specification coordinates in native order.
    pub fn specification_point(&self) -> Result<Vec<f64>, ProblemError> {
        self.source
            .prepared
            .prepared
            .plan
            .columns()
            .iter()
            .map(|id| {
                self.source.values.scalars.get(id).copied().ok_or_else(|| {
                    ProblemError::Contract("frozen original source coordinate missing".into())
                })
            })
            .collect()
    }
    /// Explicit admitted native backend.
    pub fn backend(&self) -> Backend {
        self.backend
    }
    pub(crate) fn result_bytes(&self) -> Result<usize, MathRuntimeError> {
        self.original
            .result_bytes()?
            .checked_add(self.profile.controls.report_allowance()?)
            .and_then(|n| {
                n.checked_add(
                    self.physical
                        .coordinates()
                        .len()
                        .checked_mul(size_of::<f64>())?,
                )
            })
            .ok_or(MathRuntimeError::Limit("derived result allowance"))
    }
}
/// A fresh, evaluable original-coordinate start. It supplies neither acceptance nor
/// objective/dual/status equivalence with an auxiliary native report.
#[derive(Debug)]
pub struct OriginalProposal {
    /// Frozen scientific source identity.
    pub original: pse_ids::ContentHash,
    /// Actual auxiliary producer identity.
    pub family: pse_ids::ContentHash,
    /// Freshly screened coordinates in original physical order.
    pub coordinates: Vec<f64>,
    /// Original guard interpretation used by the screen.
    pub guards: pse_ids::ContentHash,
    /// Consumed reconstruction evidence, when an implicit inverse supplied the point.
    pub reconstruction_accuracy: Option<AccuracyEvidence>,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl OriginalProposal {
    pub(super) fn from_original(
        original: pse_ids::ContentHash,
        family: pse_ids::ContentHash,
        coordinates: Vec<f64>,
        physical: &OriginalContract,
        reconstruction_accuracy: Option<AccuracyEvidence>,
        owner: Arc<pse_columnar::AllocationLease>,
    ) -> Result<Self, ProblemError> {
        screen_bounds(physical, &coordinates)?;
        let bytes = coordinates
            .capacity()
            .checked_mul(size_of::<f64>())
            .and_then(|n| n.checked_add(size_of::<Self>()))
            .ok_or_else(|| ProblemError::memory("original proposal extent"))?;
        if owner.size() < bytes {
            return Err(ProblemError::memory("original proposal allocation owner"));
        }
        Ok(Self {
            original,
            family,
            coordinates,
            guards: physical.obligations().guards,
            reconstruction_accuracy,
            _owner: owner,
        })
    }
}
#[derive(Clone, Debug)]
pub(crate) struct DerivedFailure {
    cause: Arc<ProblemError>,
    refinement_product: Option<pse_ids::ContentHash>,
    observed: WorkObservation,
}
impl From<Arc<ProblemError>> for DerivedFailure {
    fn from(cause: Arc<ProblemError>) -> Self {
        Self {
            cause,
            refinement_product: None,
            observed: WorkObservation {
                attempts: 1,
                evaluations: None,
                iterations: None,
                factorizations: None,
                proof_steps: None,
            },
        }
    }
}
impl From<ProblemError> for DerivedFailure {
    fn from(cause: ProblemError) -> Self {
        Arc::new(cause).into()
    }
}
impl From<MathRuntimeError> for DerivedFailure {
    fn from(cause: MathRuntimeError) -> Self {
        cause.into_problem().into()
    }
}
impl From<pse_math::MathError> for DerivedFailure {
    fn from(cause: pse_math::MathError) -> Self {
        ProblemError::from(cause).into()
    }
}
impl DerivedFailure {
    pub(crate) fn effect(
        self,
        prepared: &PreparedDerived,
        enclosing: &ExecutionScope,
    ) -> super::super::strategy::EffectFailure {
        if let Some((product, source_key, validity)) = refinement_cause(&self.cause)
            && self.refinement_product == Some(product)
            && prepared.reduced.as_ref().is_some_and(|r| {
                r.contract.source() == source_key && r.contract.validity() == validity
            })
        {
            return super::super::strategy::EffectFailure::refinement(
                self.cause,
                self.observed,
                prepared.operation_scope(),
                enclosing,
            );
        }
        prepared.effect_failure(self.cause, self.observed, enclosing)
    }
}
/// Actual auxiliary native report and independently screened original proposal.
#[derive(Debug)]
pub struct DerivedAttempt {
    /// Complete native result with its retained allocation owner.
    pub outcome: Outcome,
    /// Original start proposal; scientific acceptance belongs to the original corrector.
    pub proposal: Option<OriginalProposal>,
    /// Typed original-screen failure, preserving its full source cause.
    pub screening_failure: Option<Arc<ProblemError>>,
    pub(crate) refinement_product: Option<pse_ids::ContentHash>,
    /// Shared original-coordinate point key and the exact directional action receipt.
    pub(super) applied_action: Option<(pse_ids::ContentHash, AccuracyEvidence)>,
}
impl DerivedAttempt {
    /// Observed native work, retaining unavailable totals as unknown.
    pub fn work(&self) -> WorkObservation {
        let w = match &self.outcome {
            Outcome::Native(report) => report.evidence.work,
            Outcome::Constant(report) => report.work,
            Outcome::Rejected(_) => WorkEvidence::default(),
        };
        WorkObservation {
            attempts: 1,
            evaluations: w.evaluations,
            iterations: w.iterations,
            factorizations: w.factorizations,
            proof_steps: w.proof_steps,
        }
    }
}
pub(super) fn physical_contract(
    original: &PreparedSolve,
    source: &AlgebraicCase,
    cancel: &Arc<std::sync::atomic::AtomicBool>,
) -> Result<Arc<OriginalContract>, MathRuntimeError> {
    let plan = &source.prepared.prepared.plan;
    let identity = original.original_identity()?;
    let obligation = |purpose: &str| {
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV2);
        h.str(purpose).hash(&identity);
        h.finish_hash()
    };
    let mut derivative_source = FramedHasher::new(pse_ids::Frame::DerivedBindingV2);
    derivative_source
        .str("actual-frozen-compiled-derivative")
        .hash(&identity);
    let derivative_source =
        source
            .prepared
            .compiled()
            .artifacts
            .iter()
            .fold(derivative_source, |mut h, artifact| {
                h.hash(&artifact.key());
                h
            });
    let base = OriginalContract::from_case(
        plan,
        original.normalization.key(),
        derivative_source.finish_hash(),
        OriginalObligations {
            guards: obligation("frozen-original-guards"),
            selection: obligation("frozen-original-selected-function"),
            objective: plan
                .structure()
                .objective()
                .map(|_| obligation("frozen-authored-objective")),
        },
        cancel,
    )?;
    if plan.order() < DerivativeOrder::First && plan.has_directional_actions() {
        Ok(Arc::new(OriginalContract::new(
            base.identity(),
            base.normalization(),
            base.coordinates().to_vec(),
            base.constraints().to_vec(),
            base.incidence().to_vec(),
            DerivativeSupport {
                order: DerivativeOrder::First,
                jacobian_product: true,
                source: base.support().source,
            },
            base.obligations(),
        )?))
    } else {
        Ok(Arc::new(base))
    }
}
pub(super) fn zero_contract(
    original: &Arc<OriginalContract>,
) -> Result<Arc<OriginalContract>, MathRuntimeError> {
    if original.obligations().objective.is_some()
        || original.coordinates().len() != original.constraints().len()
        || original
            .constraints()
            .iter()
            .any(|r| !r.lower.is_finite() || r.lower.to_bits() != r.upper.to_bits())
    {
        return Err(ProblemError::Unsupported(
            "derived roots require an objective-free square system of finite authored equalities"
                .into(),
        )
        .into());
    }
    Ok(original.zero_residual_view()?)
}

fn mass_edges(matrix: &AssemblyMatrix) -> Vec<Entry<GlobalRow, GlobalCol>> {
    let s = matrix.matrix().symbolic();
    (0..s.ncols())
        .flat_map(|c| {
            s.row_idx_of_col(c)
                .map(move |r| Entry::new(GlobalRow::new(r), GlobalCol::new(c)))
        })
        .collect()
}
type FamilyLayout = (
    OracleContract,
    Vec<(f64, f64)>,
    Vec<Entry<GlobalRow, GlobalCol>>,
    Vec<ReducedRow>,
);
fn layout(
    family: &PreparedFamily,
    reduced: Option<&ReducedPreparation>,
) -> Result<FamilyLayout, MathRuntimeError> {
    let original = family.original();
    let mut coordinates = family.coordinate_map().map_or_else(
        || original.coordinates().to_vec(),
        |columns| {
            columns
                .iter()
                .map(|c| original.coordinates()[c.get()].clone())
                .collect()
        },
    );
    let mut rows = Vec::new();
    let mut bounds = Vec::new();
    let mut map = Vec::new();
    let mut edges = family
        .incidence()
        .iter()
        .copied()
        .filter(|e| e.col.get() < coordinates.len())
        .collect::<Vec<_>>();
    if let Some(reduced) = reduced {
        for &r in family
            .row_map()
            .ok_or_else(|| ProblemError::Contract("missing reduced row map".into()))?
        {
            let row = &original.constraints()[r.get()];
            rows.push(row.id);
            bounds.push((row.lower, row.upper));
            map.push(ReducedRow::Constraint(r));
        }
        for (i, c) in original.coordinates().iter().enumerate() {
            let col = GlobalCol::new(i);
            if reduced.contract.retained().contains(&col)
                || !c.lower.is_finite() && !c.upper.is_finite()
            {
                continue;
            }
            let row = rows.len();
            rows.push(pse_ids::named_id(c.id, "reconstructed-original-bound"));
            bounds.push((c.lower, c.upper));
            map.push(ReducedRow::CoordinateBound(col));
            edges.extend(
                reduced
                    .contract
                    .incidence()
                    .iter()
                    .filter(|e| e.row == col)
                    .map(|e| Entry::new(GlobalRow::new(row), e.col)),
            );
        }
    } else {
        for (i, row) in original.constraints().iter().enumerate() {
            rows.push(row.id);
            bounds.push((row.lower, row.upper));
            map.push(ReducedRow::Constraint(GlobalRow::new(i)));
        }
    }
    if let PreparedFamily::LeastDeviation(f) = family {
        for (coordinate, (lower, upper)) in coordinates.iter_mut().zip(f.coordinate_bounds()) {
            coordinate.lower = lower;
            coordinate.upper = upper;
        }
    }
    let support = family.support();
    let contract = OracleContract {
        identity: family.key(),
        variables: coordinates
            .into_iter()
            .map(|c| Variable {
                id: c.id,
                lower: c.lower,
                upper: c.upper,
            })
            .collect(),
        rows,
        derivatives: support.order,
        smoothness: if matches!(family, PreparedFamily::LeastDeviation(_)) {
            support.order
        } else {
            support.order.min(DerivativeOrder::First)
        },
    };
    if !contract.variables.is_empty() {
        contract.validate(support.order)?;
    }
    Ok((contract, bounds, edges, map))
}
impl MathService {
    /// Discover only actual retained selected factories with complete named original
    /// correspondence. Consumer accuracy/refinement is supplied explicitly, never
    /// inferred from a successful structural matching or a concrete source type.
    pub(crate) fn automatic_reduced_request(
        &self,
        original: &PreparedSolve,
        accuracy: ReconstructionAccuracy,
    ) -> Result<Option<DerivedRequest>, MathRuntimeError> {
        accuracy.refinement.validate()?;
        let Representation::Algebraic(source) = &original.representation else {
            return Ok(None);
        };
        let normalized_point = original.operational_point_allowance()?;
        if !accuracy.point.is_finite()
            || accuracy.point <= 0.0
            || accuracy.point > normalized_point
            || !accuracy.action.is_finite()
            || accuracy.action <= 0.0
            || accuracy.action > original.numerics.policy.supplier_action_accuracy
        {
            return Err(ProblemError::Contract(
                "automatic reconstruction allowance exceeds original consumer budget".into(),
            )
            .into());
        }
        let columns = source.prepared.prepared.plan.columns();
        let fixed: BTreeSet<_> = source
            .prepared
            .prepared
            .plan
            .structure()
            .parameters()
            .iter()
            .map(|p| p.id)
            .chain(
                source
                    .prepared
                    .prepared
                    .plan
                    .structure()
                    .variables()
                    .iter()
                    .filter(|v| v.fixed)
                    .map(|v| v.port.id),
            )
            .filter(|id| source.values.scalars.get(id).is_some_and(|v| v.is_finite()))
            .collect();
        let row_ids: BTreeSet<_> = source
            .prepared
            .prepared
            .plan
            .structure()
            .rows()
            .iter()
            .filter(|r| r.lower.is_finite() && r.lower == r.upper)
            .map(|r| r.id)
            .collect();
        let mut pending = Vec::new();
        for registration in source.providers.values() {
            let Some(factory) = registration.source::<ReconstructionFactory>() else {
                continue;
            };
            let residuals = factory.residuals();
            if factory
                .spec()
                .inputs
                .iter()
                .any(|p| !columns.contains(&p.id) && !fixed.contains(&p.id))
                || factory
                    .spec()
                    .outputs
                    .iter()
                    .any(|p| !columns.contains(&p.id))
                || residuals.is_empty()
                || residuals
                    .iter()
                    .any(|b| b.rows.iter().any(|r| !row_ids.contains(r)))
                || !factory.supports_reconstruction()
            {
                continue;
            }
            // Producer-issued subtraction is checked by named row against the
            // original bounds before admitting this optional operation.
            let realization = factory.realization().clone();
            let mut compatible = true;
            for id in &residuals[0].rows {
                let row = source
                    .prepared
                    .prepared
                    .plan
                    .structure()
                    .rows()
                    .iter()
                    .find(|r| r.id == *id)
                    .ok_or_else(|| {
                        ProblemError::Contract("automatic supplier named equality absent".into())
                    })?;
                if let SelectedResidualRealization::ZeroResiduals { authored_offsets } =
                    &realization
                {
                    compatible &= authored_offsets
                        .iter()
                        .find(|(name, _)| name == id)
                        .is_some_and(|(_, value)| value.to_bits() == row.lower.to_bits());
                }
            }
            if !compatible {
                continue;
            }
            let unknowns = factory
                .spec()
                .outputs
                .iter()
                .map(|port| {
                    let variable = source
                        .prepared
                        .prepared
                        .plan
                        .structure()
                        .variables()
                        .iter()
                        .find(|v| v.port.id == port.id)
                        .ok_or_else(|| {
                            ProblemError::Contract("original supplier output bounds absent".into())
                        })?;
                    Ok(pse_math::implicit::Unknown {
                        id: port.id,
                        lower: variable.lower.unwrap_or(f64::NEG_INFINITY),
                        upper: variable.upper.unwrap_or(f64::INFINITY),
                    })
                })
                .collect::<Result<Vec<_>, ProblemError>>()?;
            let factory = factory.bind_original_bounds(unknowns)?;
            let owner =
                self.reserve("math:automatic-admitted-supplier", factory_bytes(&factory)?)?;
            pending.push(ReducedSupplier {
                factory: Arc::new(factory),
                factory_owner: owner,
                validity: original.preparation_identity()?,
            });
        }
        if pending.is_empty() {
            return Ok(None);
        }
        let outputs: BTreeSet<_> = pending
            .iter()
            .flat_map(|s| s.factory.spec().outputs.iter().map(|p| p.id))
            .collect();
        let retained: Vec<_> = columns
            .iter()
            .enumerate()
            .filter(|(_, id)| !outputs.contains(id))
            .map(|(i, _)| GlobalCol::new(i))
            .collect();
        let mut available: BTreeSet<_> = retained
            .iter()
            .map(|c| columns[c.get()])
            .chain(fixed)
            .collect();
        let mut suppliers = Vec::new();
        while !pending.is_empty() {
            let Some(index) = pending.iter().position(|s| {
                s.factory
                    .spec()
                    .inputs
                    .iter()
                    .all(|p| available.contains(&p.id))
            }) else {
                return Ok(None);
            };
            let supplier = pending.remove(index);
            for port in &supplier.factory.spec().outputs {
                if !available.insert(port.id) {
                    return Ok(None);
                }
            }
            suppliers.push(supplier);
        }
        Ok(Some(DerivedRequest::Reduced {
            suppliers,
            retained,
            accuracy,
        }))
    }
    /// Freeze an actual family over the original compiled source, using an explicit
    /// admitted profile and the caller's original task scope while waiting for capacity.
    pub async fn prepare_derived(
        self: &Arc<Self>,
        mut original: PreparedSolve,
        request: DerivedRequest,
        profile: SolverProfile,
        scope: ExecutionScope,
        driver: &crate::CancelSource,
    ) -> Result<PreparedDerived, MathRuntimeError> {
        profile.controls.validate()?;
        scope.check().map_err(ProblemError::from)?;
        if original.task_scope.as_ref().is_some_and(|declared| {
            !Arc::ptr_eq(declared.cancellation(), scope.cancellation())
                || declared.deadline() != scope.deadline()
        }) {
            return Err(ProblemError::Contract(
                "derived preparation must preserve the original declared task scope".into(),
            )
            .into());
        }
        if profile.numerics.key() != original.numerics.policy.key()
            || profile.controls.threads != original.threads()
            || profile.sensitivity.is_some()
        {
            return Err(ProblemError::Contract("derived profile must preserve frozen numerical policy/thread extent and request no undeclared sensitivity".into()).into());
        }
        let backend = match profile.selection {
            SolverSelection::Explicit(b) => b,
            SolverSelection::Auto => return Err(ProblemError::Contract(
                "derived producer requires an explicit profile; original routing remains unchanged"
                    .into(),
            )
            .into()),
        };
        let source = match &original.representation {
            Representation::Algebraic(a) => a.clone(),
            _ => {
                return Err(ProblemError::Unsupported(
                    "derived family requires a compiled algebraic original source".into(),
                )
                .into());
            }
        };
        let plan = &source.prepared.prepared.plan;
        let complete_reconstruction =
            matches!(&request, DerivedRequest::Reduced { retained, .. } if retained.is_empty());
        if !plan.structure().native().is_empty() || !plan.structure().requirements().is_empty() {
            return Err(ProblemError::Unsupported("derived callbacks do not rebind authored native-handler or formulation requirements".into()).into());
        }
        if !complete_reconstruction
            && plan.order() < DerivativeOrder::First
            && (!matches!(
                request,
                DerivedRequest::AnchoredHomotopy { .. } | DerivedRequest::ShiftedPseudoTime { .. }
            ) || !plan.has_directional_actions())
        {
            return Err(ProblemError::Unsupported("derived source needs actual prepared First callbacks or actual demanded root actions".into()).into());
        }
        let representation = execution::adapter(backend).representation();
        let roots = matches!(
            request,
            DerivedRequest::AnchoredHomotopy { .. } | DerivedRequest::ShiftedPseudoTime { .. }
        );
        if !complete_reconstruction
            && (roots && representation != execution::Representation::Roots
                || !roots && representation != execution::Representation::Nlp)
        {
            return Err(ProblemError::Unsupported(
                "derived family/profile native representation mismatch".into(),
            )
            .into());
        }
        let intent = if roots {
            SolveIntent::Root
        } else if matches!(
            request,
            DerivedRequest::Feasibility | DerivedRequest::LeastDeviation { .. }
        ) {
            SolveIntent::Initialize
        } else {
            original.profile.intent
        };
        if profile.intent != intent {
            return Err(ProblemError::Contract("derived intent must match its actual root, feasibility or reduced-objective operation".into()).into());
        }
        if !complete_reconstruction
            && routing::derivative_demand(
                execution::adapter(backend).capability(),
                &profile.controls,
            )
            .is_some_and(|order| {
                order
                    > if matches!(request, DerivedRequest::LeastDeviation { .. }) {
                        plan.available_order()
                    } else {
                        DerivativeOrder::First
                    }
            })
        {
            return Err(ProblemError::Unsupported(
                "derived source does not supply the requested native derivative order".into(),
            )
            .into());
        }
        original.composition = None;
        let local = Instant::now()
            .checked_add(profile.controls.time_limit)
            .ok_or(MathRuntimeError::Limit("derived submission deadline"))?;
        let deadline = scope.deadline().map_or(local, |outer| outer.min(local));
        let operation_scope = ExecutionScope::new(scope.cancellation().clone(), Some(deadline));
        let worker_scope = operation_scope.clone();
        let task_scope = scope.clone();
        let control = FlightCancellation::default();
        let cores = profile.controls.threads;
        let service = self.clone();
        let workspace = preparation_bytes(&source, &request)?
            .checked_add(binding_metadata_bytes(&source, &profile)?)
            .ok_or(MathRuntimeError::Limit("derived preparation bindings"))?;
        if workspace > self.policy.workspace_bytes {
            return Err(MathRuntimeError::Limit("derived preparation workspace"));
        }
        let operation=self.job_retained_scoped(cores,workspace,control.clone(),operation_scope.deadline(),move |abort| {
            if abort.load(std::sync::atomic::Ordering::Acquire) {return Err(MathRuntimeError::Cancelled);}
            worker_scope.check().map_err(ProblemError::from)?;let physical=physical_contract(&original,&source,worker_scope.cancellation())?;let original_identity=original.original_identity()?;
            let mut reduced=None;
            let family=match &request {
                DerivedRequest::Feasibility=>PreparedFamily::General(Arc::new(DerivedFamily::bounded_feasibility(physical.clone()))),
                DerivedRequest::LeastDeviation {center,scales,weights,free,held,metric_source}=>{
                    let map=|ids:&[pse_ids::SemanticId]|ids.iter().map(|id|physical.coordinates().iter().position(|c|c.id==*id).map(GlobalCol::new).ok_or_else(||ProblemError::Contract("least-deviation role is not a frozen original coordinate".into()))).collect::<Result<Vec<_>,ProblemError>>();
                    PreparedFamily::LeastDeviation(Arc::new(pse_math::initialization::LeastDeviation::new(physical.clone(),center.clone(),scales.clone(),weights.clone(),map(free)?,map(held)?,*metric_source)?))
                },
                DerivedRequest::AnchoredHomotopy {anchor,parameter}=>{if anchor.len()!=physical.coordinates().len()||anchor.iter().zip(physical.coordinates()).any(|(v,c)|!v.is_finite()||*v<c.lower||*v>c.upper)||!parameter.is_finite()||!(0.0..=1.0).contains(parameter) {return Err(ProblemError::Contract("homotopy anchor/parameter outside declared domain".into()).into());}PreparedFamily::General(Arc::new(DerivedFamily::anchored_homotopy(zero_contract(&physical)?)?))},
                DerivedRequest::ShiftedPseudoTime {anchor,step,sign,pairing,mass}=>{if anchor.len()!=physical.coordinates().len()||anchor.iter().zip(physical.coordinates()).any(|(v,c)|!v.is_finite()||*v<c.lower||*v>c.upper)||!step.is_finite()||*step<=0.0||mass.matrix().nrows()!=physical.coordinates().len()||mass.matrix().ncols()!=physical.coordinates().len()||mass.matrix().val().iter().any(|v|!v.is_finite()) {return Err(ProblemError::Contract("shifted pseudo-time actual anchor/step/mass domain".into()).into());}PreparedFamily::General(Arc::new(DerivedFamily::shifted_pseudo_time(zero_contract(&physical)?,pairing.clone(),*sign,MassStructure::Frozen{incidence:mass_edges(mass)})?))},
                DerivedRequest::Reduced {suppliers,retained,accuracy}=>{
                    accuracy.refinement.validate()?;if !accuracy.point.is_finite()||accuracy.point<=0.0||!accuracy.action.is_finite()||accuracy.action<=0.0 {return Err(ProblemError::Contract("explicit reconstruction consumer accuracy allowances".into()).into());}
                    let prepared = suppliers.iter().map(|supplier| prepare_reduced_supplier(supplier, &physical, &original.normalization, &source.values, worker_scope.cancellation())).collect::<Result<Vec<_>, MathRuntimeError>>()?;
                    let contracts = prepared.iter().map(|p| p.contract.clone()).collect::<Vec<_>>();
                    let contract = CompositeReconstruction::<SelectedImplicitReconstruction<ProblemError>>::prepare_contract_normalized(physical.clone(), retained.clone(), &contracts, &original.normalization, &prepared.iter().map(|p| p.normalization.clone()).collect::<Vec<_>>())?;
                    let family=DerivedFamily::reduced_space(physical.clone(),contract.clone())?;reduced=Some(ReducedPreparation {contract,suppliers:prepared});PreparedFamily::General(Arc::new(family))
                },
            };
            let (mut contract,bounds,edges,row_map)=layout(&family,reduced.as_ref())?;
            if let PreparedFamily::LeastDeviation(f)=&family {let binding=native::initialization::OriginalNlpBinding {original:physical.identity(),source:physical.identity(),preparation:original.preparation_identity()?};contract.identity=native::initialization::LeastDeviationOracle::contract_identity(f,binding);}
            let pattern=AssemblyMatrix::new(contract.rows.len(),contract.variables.len(),&edges,isize::MAX as usize)?;
            if native::structural::construction_bytes(&contract,pattern.matrix().symbolic())?>workspace {return Err(MathRuntimeError::Limit("derived structural preparation workspace"));}
            let structure=native::structural::oracle_structure_with_cancel(&contract,pattern.matrix().symbolic(),&bounds,intent==SolveIntent::Optimize||matches!(family,PreparedFamily::LeastDeviation(_)),worker_scope.cancellation())?;
            let normalization=Normalization {variables:family.coordinate_map().map_or_else(||original.normalization.variables.clone(),|cols|cols.iter().map(|c|original.normalization.variables[c.get()]).collect()),rows:row_map.iter().map(|r|match r {ReducedRow::Constraint(r)=>original.normalization.rows[r.get()],ReducedRow::CoordinateBound(c)=>original.normalization.variables[c.get()]}).collect(),objective:if matches!(family,PreparedFamily::LeastDeviation(_)) {1.0}else{original.normalization.objective}};
            let tolerances=Tolerances {variables:family.coordinate_map().map_or_else(||original.tolerances.variables.clone(),|cols|cols.iter().map(|c|original.tolerances.variables[c.get()]).collect()),rows:row_map.iter().map(|r|match r {ReducedRow::Constraint(r)=>original.tolerances.rows[r.get()],ReducedRow::CoordinateBound(c)=>original.tolerances.variables[c.get()]}).collect(),integrality:original.tolerances.integrality};
            let accuracy=ResolvedAccuracy::resolve(&original.numerics.policy,&tolerances,&normalization)?;let snapshot=execution::Snapshot::observe(&execution::LINKED);admit_profile(&profile,if contract.variables.is_empty() {Route::Constant}else{Route::Native(backend)},&snapshot)?;
            let mut facts=source.prepared.prepared.facts.clone();facts.variables=contract.variables.len();facts.rows=contract.rows.len();facts.objective=matches!(family,PreparedFamily::LeastDeviation(_))||intent==SolveIntent::Optimize&&physical.obligations().objective.is_some();facts.objectives=usize::from(facts.objective);facts.equalities=bounds.iter().all(|(l,u)|l.is_finite()&&l==u);facts.derivatives=contract.derivatives;facts.prepared_derivatives=contract.derivatives;
            let source_domains=plan_domains(&source);facts.domains=family.coordinate_map().map_or(source_domains.clone(),|cols|cols.iter().map(|c|source_domains[c.get()]).collect());facts.bounds=contract.variables.iter().map(|v|bound_shape(v.lower,v.upper)).collect();facts.class_status=pse_math::presolve::ClassStatus::Unassessed;facts.coefficients=false;facts.affine_rows=vec![false;facts.rows];facts.objective_degree=None;facts.quadratic=false;facts.convexity=pse_math::convexity::Convexity::not_assessed(family.key());facts.bound_assumptions=family.key();
            let prepared=[routing::ArtifactDemand::Representation(representation)];let refusals=BTreeMap::new();let guards=BTreeMap::new();
            let decision=routing::Requirements {table:&execution::LINKED,facts:&facts,intent,numerical_psd:false,least_squares:false,controls:&profile.controls,settings:&profile.backend,sensitivity:false,context:routing::Context {snapshot:snapshot.clone(),pending_classes:&[],refusals:&refusals,structure:Some(structure.clone()),oracle:Some(&contract),guards:&guards,budgets:Some(execution::Budgets {accuracy:&accuracy,tolerances:&tolerances,normalization:&normalization}),coefficients:None,certificate:None,cone:None,factorable:None,prepared:&prepared}}.decision(SolverSelection::Explicit(backend));decision.route()?;
            let mut h=FramedHasher::new(pse_ids::Frame::DerivedBindingV2);h.hash(&original_identity).hash(&family.key()).hash(&profile_key(&profile)?.as_id()).hash(&snapshot.identity());frame_request(&request,&mut h);let key=h.finish_hash();let compatibility=Compatibility {layout:family.key(),profile:key,data:key,backend};
            let bytes=metadata_bytes(&physical,&family,&request,&decision)?.checked_add(binding_metadata_bytes(&source,&profile)?).ok_or(MathRuntimeError::Limit("derived prepared binding extent"))?;
            worker_scope.check().map_err(ProblemError::from)?;
            let placeholder=service.reserve("math:derived-prepared-placeholder",0)?;
            Ok((PreparedDerivedData {original,source,physical,family,request,reduced,profile,backend,snapshot,decision,normalization,tolerances,accuracy,compatibility,original_identity,key,scope:task_scope,operation_scope:worker_scope,_owner:placeholder},bytes))
        });
        tokio::pin!(operation);
        let (mut prepared, owner) = tokio::select! {result=&mut operation=>result?,()=driver.cancelled()=>{scope.cancellation().store(true,std::sync::atomic::Ordering::Release);control.cancel();let _=operation.await;return Err(MathRuntimeError::Cancelled);}};
        prepared._owner = owner;
        Ok(PreparedDerived {
            data: Arc::new(prepared),
        })
    }
}
fn plan_domains(
    source: &AlgebraicCase,
) -> Vec<pse_model::generated::enums::ModelingVariableDomain> {
    let plan = &source.prepared.prepared.plan;
    plan.columns()
        .iter()
        .filter_map(|id| {
            plan.structure()
                .variables()
                .iter()
                .find(|v| v.port.id == *id)
                .map(|v| v.domain)
        })
        .collect()
}
fn bound_shape(lower: f64, upper: f64) -> pse_math::facts::BoundShape {
    use pse_math::facts::BoundShape::*;
    match (lower.is_finite(), upper.is_finite()) {
        (false, false) => Free,
        (true, false) if lower == 0.0 => Nonnegative,
        (true, false) => Lower,
        (false, true) if upper == 0.0 => Nonpositive,
        (false, true) => Upper,
        (true, true) => Boxed,
    }
}
fn frame_request(request: &DerivedRequest, h: &mut FramedHasher) {
    match request {
        DerivedRequest::LeastDeviation { metric_source, .. } => {
            h.str("least-deviation").hash(metric_source);
        }
        DerivedRequest::Feasibility => {
            h.str("feasibility");
        }
        DerivedRequest::AnchoredHomotopy { anchor, parameter } => {
            h.str("anchored-homotopy").f64(*parameter);
            for x in anchor {
                h.f64(*x);
            }
        }
        DerivedRequest::ShiftedPseudoTime {
            anchor,
            step,
            sign,
            pairing,
            mass,
        } => {
            h.str("shifted-pseudo-time").f64(*step).f64(*sign);
            for x in anchor {
                h.f64(*x);
            }
            for c in pairing {
                h.u64(c.get() as u64);
            }
            for v in mass.matrix().val() {
                h.f64(*v);
            }
        }
        DerivedRequest::Reduced {
            suppliers,
            retained,
            accuracy,
        } => {
            h.str("composite-reduced")
                .u64(suppliers.len() as u64)
                .f64(accuracy.point)
                .f64(accuracy.action)
                .str(accuracy.class.as_str())
                .u64(accuracy.refinement.rounds as u64)
                .u64(accuracy.refinement.proof_cells);
            for column in retained {
                h.u64(column.get() as u64);
            }
            for supplier in suppliers {
                h.hash(&supplier.factory.configuration_key())
                    .hash(&supplier.validity);
                match supplier.factory.realization() {
                    SelectedResidualRealization::AuthoredValues => {
                        h.str("authored-values");
                    }
                    SelectedResidualRealization::ZeroResiduals { authored_offsets } => {
                        h.str("zero-residuals");
                        for (id, offset) in authored_offsets {
                            h.id(id).f64(*offset);
                        }
                    }
                }
            }
        }
    }
}
fn factory_bytes(factory: &ReconstructionFactory) -> Result<usize, MathRuntimeError> {
    Ok(factory.retained_bytes()?)
}
fn prepare_reduced_supplier(
    supplier: &ReducedSupplier,
    original: &Arc<OriginalContract>,
    normalization: &Normalization,
    values: &CaseValues,
    cancel: &Arc<std::sync::atomic::AtomicBool>,
) -> Result<ReducedSupplierPreparation, MathRuntimeError> {
    if supplier.factory_owner.size() < factory_bytes(&supplier.factory)? {
        return Err(MathRuntimeError::Limit(
            "selected factory actual retained owner",
        ));
    }
    let factory = supplier.factory.as_ref();
    let ports: Vec<_> = factory
        .spec()
        .inputs
        .iter()
        .chain(&factory.spec().outputs)
        .collect();
    let columns: Vec<_> = ports
        .iter()
        .map(|p| {
            original
                .coordinates()
                .iter()
                .position(|c| c.id == p.id)
                .map(GlobalCol::new)
        })
        .collect();
    let coordinates = ports
        .iter()
        .zip(&columns)
        .enumerate()
        .map(|(index, (port, col))| {
            if let Some(col) = col {
                return Ok(original.coordinates()[col.get()].clone());
            }
            if index >= factory.spec().inputs.len() {
                return Err(ProblemError::Contract(
                    "supplier output absent from original".into(),
                ));
            }
            let value = values
                .scalars
                .get(&port.id)
                .filter(|v| v.is_finite())
                .copied()
                .ok_or_else(|| {
                    ProblemError::Contract(
                        "supplier fixed input absent from original binding".into(),
                    )
                })?;
            Ok(math::Coordinate {
                id: port.id,
                lower: value,
                upper: value,
            })
        })
        .collect::<Result<Vec<_>, ProblemError>>()?;
    let residuals = factory.residuals();
    let branch = residuals
        .first()
        .ok_or_else(|| ProblemError::Contract("supplier residual source absent".into()))?;
    let rows: Vec<_> = branch
        .rows
        .iter()
        .map(|id| {
            original
                .constraints()
                .iter()
                .position(|r| r.id == *id)
                .map(GlobalRow::new)
                .ok_or_else(|| ProblemError::Contract("supplier row absent from original".into()))
        })
        .collect::<Result<_, _>>()?;
    let incidence = original
        .incidence()
        .iter()
        .filter_map(|edge| {
            Some(Entry::new(
                GlobalRow::new(rows.iter().position(|r| *r == edge.row)?),
                GlobalCol::new(columns.iter().position(|c| *c == Some(edge.col))?),
            ))
        })
        .collect();
    let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV2);
    h.str("original-local-supplier-projection")
        .hash(&original.identity())
        .hash(&factory.configuration_key());
    let normalization = Normalization {
        variables: columns
            .iter()
            .map(|c| c.map_or(1.0, |c| normalization.variables[c.get()]))
            .collect(),
        rows: rows.iter().map(|r| normalization.rows[r.get()]).collect(),
        objective: normalization.objective,
    };
    let local = Arc::new(OriginalContract::new(
        h.finish_hash(),
        normalization.key(),
        coordinates,
        rows.iter()
            .map(|r| original.constraints()[r.get()].clone())
            .collect(),
        incidence,
        original.support(),
        original.obligations(),
    )?);
    let eliminated: Vec<_> = (0..rows.len()).map(GlobalRow::new).collect();
    let retained: Vec<_> = (0..factory.spec().inputs.len())
        .map(GlobalCol::new)
        .collect();
    let binding = SelectedResidualBinding::for_source(
        factory,
        &local,
        &eliminated,
        supplier.factory.realization().clone(),
    )?;
    let contract = SelectedImplicitReconstruction::<ProblemError>::prepare_contract_with_binding(
        factory,
        local,
        retained,
        eliminated,
        supplier.validity,
        cancel,
        &binding,
    )?;
    Ok(ReducedSupplierPreparation {
        contract,
        binding,
        normalization,
    })
}
fn metadata_bytes(
    physical: &OriginalContract,
    family: &PreparedFamily,
    request: &DerivedRequest,
    decision: &routing::Decision,
) -> Result<usize, MathRuntimeError> {
    let maps = (physical.coordinates().len() + physical.constraints().len())
        .checked_mul(6 * size_of::<f64>())
        .ok_or(MathRuntimeError::Limit("derived transport metadata"))?;
    let extra = match request {
        DerivedRequest::Feasibility => 0,
        DerivedRequest::LeastDeviation {
            center,
            scales,
            weights,
            free,
            held,
            ..
        } => {
            (center.capacity() + scales.capacity() + weights.capacity()) * size_of::<f64>()
                + (free.capacity() + held.capacity()) * size_of::<pse_ids::SemanticId>()
        }
        DerivedRequest::AnchoredHomotopy { anchor, .. } => anchor.capacity() * size_of::<f64>(),
        DerivedRequest::ShiftedPseudoTime {
            anchor,
            pairing,
            mass,
            ..
        } => {
            anchor.capacity() * size_of::<f64>()
                + pairing.capacity() * size_of::<GlobalCol>()
                + mass.retained_bytes()
        }
        DerivedRequest::Reduced {
            retained,
            suppliers,
            ..
        } => {
            retained.capacity() * size_of::<GlobalCol>()
                + suppliers.capacity() * size_of::<ReducedSupplier>()
                + size_of::<ReducedPreparation>()
                + suppliers
                    .iter()
                    .map(|s| {
                        s.factory.spec().outputs.len()
                            * (size_of::<GlobalCol>() + size_of::<GlobalRow>() + 128)
                    })
                    .sum::<usize>()
        }
    };
    (size_of::<PreparedDerivedData>() + size_of::<PreparedDerived>() + 2 * size_of::<usize>())
        .checked_add(physical.retained_bytes())
        .and_then(|n| n.checked_add(family.retained_bytes()))
        .and_then(|n| {
            n.checked_add(if std::ptr::eq(physical, family.original()) {
                0
            } else {
                family.original().retained_bytes()
            })
        })
        .and_then(|n| {
            n.checked_add(
                family
                    .reconstruction_contract()
                    .map_or(0, |r| r.retained_bytes()),
            )
        })
        .and_then(|n| n.checked_add(maps))
        .and_then(|n| n.checked_add(extra))
        .and_then(|n| n.checked_add(decision.retained_bytes()))
        .ok_or(MathRuntimeError::Limit("derived metadata extent"))
}
pub(super) fn binding_metadata_bytes(
    source: &AlgebraicCase,
    profile: &SolverProfile,
) -> Result<usize, MathRuntimeError> {
    // A conservative upper reservation for the actual cloned ordered-map nodes,
    // including partially populated nodes. Shared programs/provider owners stay leased.
    case_values_bytes(source.values.scalars.len())?
        .checked_add(
            source
                .providers
                .len()
                .checked_mul(
                    22 * size_of::<(pse_kernels::ProviderKey, pse_kernels::Registration)>()
                        + 16 * size_of::<usize>(),
                )
                .ok_or(MathRuntimeError::Limit("derived provider binding metadata"))?,
        )
        .and_then(|n| n.checked_add(profile.controls.report_allowance().ok()?))
        .ok_or(MathRuntimeError::Limit("derived frozen binding metadata"))
}
/// Conservative allocated-node extent for one owned original/compact scalar map.
pub(super) fn case_values_bytes(entries: usize) -> Result<usize, MathRuntimeError> {
    entries
        .checked_mul(22 * size_of::<(pse_ids::SemanticId, f64)>() + 16 * size_of::<usize>())
        .ok_or(MathRuntimeError::Limit("derived scalar binding metadata"))
}

struct OwnedNlp {
    oracle: Box<dyn NlpOracle>,
    _owner: Box<dyn std::any::Any>,
}
impl std::fmt::Debug for OwnedNlp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnedDerivedNlp")
            .field("oracle", &self.oracle)
            .finish_non_exhaustive()
    }
}
impl NlpOracle for OwnedNlp {
    fn structural_analysis(&self) -> Option<&pse_structural::incidence::StructuralAnalysis> {
        self.oracle.structural_analysis()
    }
    fn normalization(&self) -> Option<&Normalization> {
        self.oracle.normalization()
    }
    fn constraint_sources(&self) -> Result<Vec<pse_math::assembly::OutputValue>, ProblemError> {
        self.oracle.constraint_sources()
    }
    fn derivative_facts(&self) -> native::DerivativeFacts {
        self.oracle.derivative_facts()
    }
    fn contract(&self) -> &OracleContract {
        self.oracle.contract()
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        self.oracle.constraint_bounds()
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.oracle.jacobian_pattern()
    }
    fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
        self.oracle.hessian_pattern()
    }
    fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
        self.oracle.objective(x)
    }
    fn constraints(&mut self, x: &[f64], o: &mut [f64]) -> Result<(), ProblemError> {
        self.oracle.constraints(x, o)
    }
    fn gradient(&mut self, x: &[f64], o: &mut [f64]) -> Result<(), ProblemError> {
        self.oracle.gradient(x, o)
    }
    fn jacobian(&mut self, x: &[f64], o: &mut [f64]) -> Result<(), ProblemError> {
        self.oracle.jacobian(x, o)
    }
    fn hessian(&mut self, x: &[f64], w: f64, l: &[f64], o: &mut [f64]) -> Result<(), ProblemError> {
        self.oracle.hessian(x, w, l, o)
    }
}
type ConcreteReduced =
    ReducedOracle<NlpBridge, CompositeReconstruction<SelectedImplicitReconstruction<ProblemError>>>;
/// Only one callback borrows the selected worker at a time. The original-proposal
/// consumer retains the actual worker/lineage used by native iteration, not a replay.
#[derive(Debug)]
struct SharedReduced {
    actual: Rc<RefCell<ConcreteReduced>>,
    contract: OracleContract,
    bounds: Vec<(f64, f64)>,
    pattern: AssemblyMatrix,
}
impl NlpOracle for SharedReduced {
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.pattern.matrix().symbolic()
    }
    fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
        None
    }
    fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
        self.actual
            .try_borrow_mut()
            .map_err(|_| ProblemError::Contract("simultaneous reduced oracle application".into()))?
            .objective(x)
    }
    fn constraints(&mut self, x: &[f64], o: &mut [f64]) -> Result<(), ProblemError> {
        self.actual
            .try_borrow_mut()
            .map_err(|_| ProblemError::Contract("simultaneous reduced oracle application".into()))?
            .constraints(x, o)
    }
    fn gradient(&mut self, x: &[f64], o: &mut [f64]) -> Result<(), ProblemError> {
        self.actual
            .try_borrow_mut()
            .map_err(|_| ProblemError::Contract("simultaneous reduced oracle application".into()))?
            .gradient(x, o)
    }
    fn jacobian(&mut self, x: &[f64], o: &mut [f64]) -> Result<(), ProblemError> {
        self.actual
            .try_borrow_mut()
            .map_err(|_| ProblemError::Contract("simultaneous reduced oracle application".into()))?
            .jacobian(x, o)
    }
    fn hessian(&mut self, _: &[f64], _: f64, _: &[f64], _: &mut [f64]) -> Result<(), ProblemError> {
        Err(ProblemError::Unsupported(
            "reduced exact Hessian unavailable".into(),
        ))
    }
}
fn step<'a>(p: &'a PreparedDerived, execution: Execution) -> execution::Step<'a> {
    execution::Step {
        adapter: execution::adapter(p.backend),
        settings: &p.profile.backend,
        snapshot: &p.snapshot,
        structure: p.decision.structure.as_ref(),
        controls: &p.profile.controls,
        accuracy: &p.accuracy,
        execution,
        tolerances: &p.tolerances,
        normalization: &p.normalization,
        compatibility: p.compatibility.clone(),
        warm: None,
    }
}
impl MathService {
    /// Independently assess a complete original proposal without starting a correction
    /// solve. Native termination/work remain observations of the auxiliary attempt;
    /// only fresh original feasibility is transported to the scientific assessor.
    pub(crate) fn derived_original_outcome(
        &self,
        p: &PreparedDerived,
        attempt: &DerivedAttempt,
        execution: &Execution,
        budget: &Arc<WorkerBudget>,
    ) -> Result<Outcome, MathRuntimeError> {
        let proposal = attempt.proposal.as_ref().ok_or_else(|| {
            ProblemError::Contract("derived completion has no complete original proposal".into())
        })?;
        if proposal.original != p.original_identity
            || proposal.family != p.family.key()
            || proposal.guards != p.physical.obligations().guards
        {
            return Err(ProblemError::Contract(
                "derived completion original correspondence".into(),
            )
            .into());
        }
        execution.check()?;
        let scope = execution.scope()?;
        let owner = self.reserve(
            "math:derived-original-assessment",
            p.original.result_bytes()?,
        )?;
        let Outcome::Native(auxiliary) = &attempt.outcome else {
            return match &attempt.outcome {
                Outcome::Constant(report) => Ok(Outcome::Constant(report.clone())),
                Outcome::Rejected(error) => Ok(Outcome::Rejected(error.clone())),
                Outcome::Native(_) => {
                    Err(ProblemError::internal("derived completion native shape").into())
                }
            };
        };
        let mut report = auxiliary.as_ref().clone();
        report.variables = p.physical.coordinates().iter().map(|c| c.id).collect();
        report.rows = p.physical.constraints().iter().map(|r| r.id).collect();
        let kind = report
            .candidate
            .as_ref()
            .ok_or_else(|| {
                ProblemError::Contract("derived completion native candidate absent".into())
            })?
            .kind;
        report.candidate = Some(Candidate {
            kind,
            primal: proposal.coordinates.clone(),
            objective: None,
            row_dual: None,
            bound_dual: None,
            reduced_costs: None,
            slacks: None,
            commitment: None,
        });
        report.certificate = None;
        report.global = None;
        report.warm_start = None;
        report.start_receipt = None;
        report.preprocessing = None;
        report.least_infeasible = None;
        report.evidence.kkt = None;
        report.evidence.coefficient = None;
        report.evidence.conic = None;
        report.evidence.global = None;
        report.evidence.original_bound = None;
        report.evidence.contradiction = None;
        report.evidence.local = None;
        report.evidence.output_accuracy = None;
        report.evidence.sensitivity = None;
        report.evidence.root_response = None;
        report.evidence.root_predictor = None;
        report.evidence.inverse_reduced_hessian = None;
        report.quality = None;
        report.observation = None;
        report.qualification = Qualification::Unqualified;
        report.termination.assurance = Assurance::None;
        let mut calls = 0u64;
        let checked = (|| -> Result<_, ProblemError> {
            screen_bounds(&p.physical, &proposal.coordinates)?;
            let (mut oracle, _oracle_owner) = self
                .derived_original_oracle(p, &scope, budget)
                .map_err(MathRuntimeError::into_problem)?;
            let eval = WorkEvidence {
                evaluations: Some(1),
                iterations: Some(0),
                factorizations: Some(0),
                proof_steps: Some(0),
            };
            let mut rows = vec![0.0; p.physical.constraints().len()];
            execution.counted(eval, || {
                calls += 1;
                oracle.constraints(&proposal.coordinates, &mut rows)
            })?;
            let objective = execution.counted(eval, || {
                calls += 1;
                oracle.objective(&proposal.coordinates)
            })?;
            let quality = quality::observed(
                oracle.contract(),
                oracle.constraint_bounds(),
                &proposal.coordinates,
                &rows,
                &p.original.tolerances,
            )?;
            let sense = p
                .source
                .prepared
                .prepared
                .plan
                .structure()
                .objective()
                .map(|o| o.sense);
            let mut observation = quality::Observation::from_values(
                sense.map(|sense| objective * sense.sign()),
                rows,
                oracle.constraint_bounds().to_vec(),
            )?;
            observation.sources = oracle.constraint_sources()?;
            execution.check()?;
            Ok((quality, observation))
        })();
        report.evidence.work.evaluations = report
            .evidence
            .work
            .evaluations
            .and_then(|n| n.checked_add(calls));
        match checked {
            Ok((quality, observation)) => {
                if quality.feasible()
                    && report.callback_failure().is_none()
                    && report.validation_failure().is_none()
                {
                    report.qualification = Qualification::Feasible;
                    report.termination.assurance = Assurance::Feasible;
                }
                if let Some(candidate) = report.candidate.as_mut() {
                    candidate.objective = observation.objective;
                }
                report.quality = Some(quality);
                report.observation = Some(observation);
            }
            Err(error) => report.record_validation_failure(error),
        }
        Ok(Outcome::Native(Box::new(report.with_owner(owner))))
    }
    fn derived_original_oracle(
        &self,
        p: &PreparedDerived,
        scope: &ExecutionScope,
        budget: &Arc<WorkerBudget>,
    ) -> Result<(native::assembled::AlgebraicOracle, Box<dyn std::any::Any>), MathRuntimeError>
    {
        let source = &p.source;
        let ExecutionWorker {
            worker,
            _case,
            _charge,
        } = self.case_worker(source.case.clone(), source.providers.clone(), scope, budget)?;
        let root_support =
            budget.charge(native::assembled::AlgebraicOracle::root_support_allowance(
                &source.prepared.prepared.plan,
                &source.prepared.prepared.structure,
            )?)?;
        let oracle = native::assembled::AlgebraicOracle::new(worker, source.values.clone())?
            .with_structural_analysis(source.prepared.prepared.structure.clone())?
            .with_presolve_facts(source.prepared.prepared.presolve.clone())?
            .with_normalization(p.original.normalization.clone())?;
        Ok((
            oracle,
            Box::new((_case, _charge, root_support, p._owner.clone())),
        ))
    }
    /// Run on the existing admitted NativeSession worker. The supplied execution's
    /// start/progress/abandonment are retained; its scope is only narrowed to the
    /// already stamped derived deadline. No second strategy or permission owner.
    pub(crate) fn derived_step(
        &self,
        p: &PreparedDerived,
        mut execution: Execution,
        retained: &mut Retained,
        budget: &Arc<WorkerBudget>,
        original_start: &[f64],
    ) -> Result<DerivedAttempt, DerivedFailure> {
        if !Arc::ptr_eq(&execution.cancel, p.scope.cancellation()) {
            return Err(ProblemError::Contract(
                "derived execution must share original task cancellation owner".into(),
            )
            .into());
        }
        let outer = execution.scope()?;
        let deadline = match (outer.deadline(), p.operation_scope.deadline()) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        let scope = ExecutionScope::new(execution.cancel.clone(), deadline);
        scope.check().map_err(ProblemError::from)?;
        execution.enclosing_scope = Some(scope.clone());
        if let Some(deadline) = deadline {
            execution.time_limit = execution
                .time_limit
                .min(deadline.saturating_duration_since(execution.started));
        }
        execution.check()?;
        screen_bounds(&p.physical, original_start)?;
        let n = p.physical.coordinates().len();
        let m = p.physical.constraints().len();
        // Basis assembly, composed products and offset verifier DAG copies are bounded
        // by actual declared dimensions/factory budgets, then held with native owners.
        let bytes = n
            .checked_mul(m.max(n))
            .and_then(|n| n.checked_mul(8 * size_of::<f64>()))
            .and_then(|n| {
                n.checked_add(metadata_bytes(&p.physical, &p.family, &p.request, &p.decision).ok()?)
            })
            .and_then(|n| n.checked_add(binding_metadata_bytes(&p.source, &p.profile).ok()?))
            .and_then(|n| {
                n.checked_add(match &p.request {
                    DerivedRequest::Reduced {
                        suppliers,
                        accuracy,
                        ..
                    } => suppliers.iter().try_fold(0usize, |n, s| {
                        n.checked_add(factory_bytes(&s.factory).ok()?)?.checked_add(
                            s.factory
                                .reconstruction_workspace_bytes(accuracy.refinement.proof_cells)
                                .ok()?,
                        )
                    })?,
                    _ => 0,
                })
            })
            .ok_or(MathRuntimeError::Limit("derived worker extent"))?;
        let family_charge = Arc::new(budget.charge(bytes)?);
        let result_owner = self.reserve("math:derived-native-report", p.result_bytes()?)?;
        let (mut oracle, source_owner) = self.derived_original_oracle(p, &scope, budget)?;
        let owner: Box<dyn std::any::Any> = Box::new((source_owner, family_charge));
        if let DerivedRequest::Reduced {
            suppliers,
            retained: columns,
            accuracy,
        } = &p.request
            && columns.is_empty()
        {
            let prepared = p.reduced.as_ref().ok_or_else(|| {
                ProblemError::Contract("full reconstruction preparation absent".into())
            })?;
            let workers = suppliers
                .iter()
                .zip(&prepared.suppliers)
                .map(|(supplier, prepared)| {
                    SelectedImplicitReconstruction::<ProblemError>::new_with_binding(
                        supplier.factory.as_ref(),
                        prepared.contract.clone(),
                        prepared.normalization.clone(),
                        scope.clone(),
                        &prepared.binding,
                    )?
                    .with_proof_cell_limit(accuracy.refinement.proof_cells)
                })
                .collect::<Result<Vec<_>, pse_math::MathError>>()?;
            let mut reconstruction = CompositeReconstruction::new_normalized(
                prepared.contract.clone(),
                workers,
                &p.original.normalization,
                &prepared
                    .suppliers
                    .iter()
                    .map(|p| p.normalization.clone())
                    .collect::<Vec<_>>(),
            )?;
            math::ReconstructionOracle::admit(&mut reconstruction, &[])?;
            let mut product = FramedHasher::new(pse_ids::Frame::DerivedBindingV2);
            product
                .str("complete-original-reconstruction")
                .hash(&p.original_identity)
                .hash(&p.family.key());
            let demand = pse_model::strategy::AccuracyDemand {
                product: product.finish_hash(),
                normalization: p.physical.normalization(),
                allowance: accuracy.point,
                class: accuracy.class,
            };
            let optional_refinement = !p.original.numerics.policy.goals.is_empty()
                && accuracy.refinement.rounds >= prepared.suppliers.len().saturating_mul(2)
                && accuracy.refinement.proof_cells
                    >= (prepared.suppliers.len() as u64).saturating_mul(2);
            // Two consumed products share the declared finite round/proof-cell ceiling.
            // No-goal execution retains its whole mandatory operational allowance.
            let first_limits = if optional_refinement {
                math::RefinementLimits {
                    rounds: accuracy.refinement.rounds / 2,
                    proof_cells: accuracy.refinement.proof_cells / 2,
                }
            } else {
                accuracy.refinement
            };
            let mut point =
                math::ReconstructionOracle::point(&mut reconstruction, &[], &demand, first_limits)?;
            let eval = WorkEvidence {
                evaluations: Some(1),
                iterations: Some(0),
                factorizations: Some(0),
                proof_steps: Some(0),
            };
            let mut assess = |values: &[f64]| -> Result<_, ProblemError> {
                screen_bounds(&p.physical, values)?;
                let mut rows = vec![0.0; m];
                execution.counted(eval, || oracle.constraints(values, &mut rows))?;
                let objective = execution.counted(eval, || oracle.objective(values))?;
                let quality = quality::observed(
                    oracle.contract(),
                    oracle.constraint_bounds(),
                    values,
                    &rows,
                    &p.original.tolerances,
                )?;
                let objective = p
                    .source
                    .prepared
                    .prepared
                    .plan
                    .structure()
                    .objective()
                    .map(|o| objective * o.sense.sign());
                let mut observation = quality::Observation::from_values(
                    objective,
                    rows,
                    oracle.constraint_bounds().to_vec(),
                )?;
                observation.sources = oracle.constraint_sources()?;
                Ok((quality, objective, observation))
            };
            let (mut quality, mut objective, mut observation) = assess(&point.values)?;
            let mut certified_reconstruction = CertifiedReconstructionPoint::issue(
                self,
                p,
                &point,
                math::ReconstructionOracle::realization(&reconstruction),
            )?;
            if optional_refinement
                && quality.feasible()
                && let Some(receipt) = &certified_reconstruction
            {
                let _values_charge = budget.charge(case_values_bytes(
                    p.source
                        .values
                        .scalars
                        .len()
                        .checked_add(n)
                        .ok_or(MathRuntimeError::Limit("goal reconstruction values extent"))?,
                )?)?;
                let mut values = p.source.values.clone();
                values.scalars.extend(
                    p.physical
                        .coordinates()
                        .iter()
                        .map(|coordinate| coordinate.id)
                        .zip(point.values.iter().copied()),
                );
                let consuming = match receipt.refinement_demand(
                    self,
                    &p.original,
                    &values,
                    &scope,
                    budget,
                    &execution,
                ) {
                    Ok(demand) => demand,
                    Err(error) if engineering_accuracy::optional_failure(&error) => None,
                    Err(error) => return Err(error.into()),
                };
                if let Some(consuming) = consuming {
                    let remaining = math::RefinementLimits {
                        rounds: accuracy.refinement.rounds - first_limits.rounds,
                        proof_cells: accuracy.refinement.proof_cells - first_limits.proof_cells,
                    };
                    match math::ReconstructionOracle::point(
                        &mut reconstruction,
                        &[],
                        &consuming,
                        remaining,
                    ) {
                        Ok(refined) => {
                            point = refined;
                            (quality, objective, observation) = assess(&point.values)?;
                            certified_reconstruction = CertifiedReconstructionPoint::issue(
                                self,
                                p,
                                &point,
                                math::ReconstructionOracle::realization(&reconstruction),
                            )?;
                        }
                        // A bounded optional proof refusal retains the actual first
                        // certificate and unmet goal; original checks remain mandatory.
                        Err(ProblemError::Math(pse_math::MathError::Refinement { .. })) => {}
                        Err(error) => return Err(error.into()),
                    }
                }
            }
            execution.check()?;
            let proposal_owner = self.reserve(
                "math:complete-reconstruction-proposal",
                size_of::<OriginalProposal>() + point.values.capacity() * size_of::<f64>(),
            )?;
            let proposal = OriginalProposal::from_original(
                p.original_identity,
                p.family.key(),
                point.values,
                &p.physical,
                Some(point.accuracy),
                proposal_owner,
            )?;
            let coordinates = p
                .physical
                .coordinates()
                .iter()
                .map(|c| c.id)
                .zip(proposal.coordinates.iter().copied())
                .collect();
            drop(owner);
            return Ok(DerivedAttempt {
                outcome: Outcome::Constant(Box::new(ConstantReport {
                    owner: Some(result_owner),
                    components: Vec::new(),
                    objective,
                    observation,
                    quality,
                    coordinates,
                    certified_reconstruction,
                    // Actual nested root/proof totals remain unavailable, not invented zero.
                    work: WorkEvidence::default(),
                })),
                proposal: Some(proposal),
                screening_failure: None,
                refinement_product: None,
                applied_action: None,
            });
        }
        let mut selected: Option<Rc<RefCell<ConcreteReduced>>> = None;
        let mut initial = p.family.coordinate_map().map_or_else(
            || original_start.to_vec(),
            |cols| {
                cols.iter()
                    .map(|c| original_start[c.get()])
                    .collect::<Vec<_>>()
            },
        );
        if let PreparedFamily::LeastDeviation(f) = &p.family {
            for c in f.held() {
                initial[c.get()] = f.center()[c.get()];
            }
            f.validate_point(&initial)?;
        }
        let result = (|| -> Result<SolveReport, MathRuntimeError> {
            let report = match &p.request {
                DerivedRequest::LeastDeviation { .. } => {
                    let PreparedFamily::LeastDeviation(family) = &p.family else {
                        return Err(ProblemError::Internal(
                            "least-deviation mathematical product mismatch".into(),
                        )
                        .into());
                    };
                    let binding = native::initialization::OriginalNlpBinding {
                        original: p.physical.identity(),
                        source: NlpOracle::contract(&oracle).identity,
                        preparation: p.original.preparation_identity()?,
                    };
                    let oracle = native::initialization::LeastDeviationOracle::new(
                        Box::new(oracle),
                        family.clone(),
                        binding,
                        scope.clone(),
                        isize::MAX as usize,
                    )?;
                    execution::nlp(
                        step(p, execution.clone()),
                        retained,
                        execution::Nlp {
                            oracle: Box::new(OwnedNlp {
                                oracle: Box::new(oracle),
                                _owner: owner,
                            }),
                            initial: &initial,
                            presolve: &p.profile.presolve,
                            intent: SolveIntent::Initialize,
                            sense: ObjectiveSense::Minimize,
                            limit: self.policy.worker_bytes / 256,
                            analysis: execution::Analysis::NONE,
                        },
                    )?
                }
                DerivedRequest::Feasibility => {
                    let bridge = NlpBridge::new(Box::new(oracle), p.physical.clone())?;
                    let bound = p.family.general()?.bind_feasibility(bridge)?;
                    let oracle = FeasibilityOracle::new(
                        bound,
                        p.source.prepared.prepared.plan.available_order(),
                        isize::MAX as usize,
                    )?;
                    execution::nlp(
                        step(p, execution.clone()),
                        retained,
                        execution::Nlp {
                            oracle: Box::new(OwnedNlp {
                                oracle: Box::new(oracle),
                                _owner: owner,
                            }),
                            initial: &initial,
                            presolve: &p.profile.presolve,
                            intent: p.profile.intent,
                            sense: ObjectiveSense::Minimize,
                            limit: self.policy.worker_bytes / 256,
                            analysis: execution::Analysis::NONE,
                        },
                    )?
                }
                DerivedRequest::AnchoredHomotopy { anchor, parameter } => {
                    oracle.admit_nle()?;
                    let offsets = p.physical.constraints().iter().map(|r| r.lower).collect();
                    let bridge = RootBridge::new(Box::new(oracle), p.physical.clone(), offsets)?
                        .zero_residuals()?;
                    let bound = p.family.general()?.bind_anchored_homotopy(
                        bridge,
                        anchor.clone(),
                        *parameter,
                        p.original_identity,
                    )?;
                    let oracle = FamilyRoot::new(
                        bound,
                        p.source.prepared.prepared.plan.available_order(),
                        isize::MAX as usize,
                    )?;
                    execution::roots(
                        step(p, execution.clone()),
                        retained,
                        execution::Roots {
                            oracle: Box::new(oracle),
                            initial: &initial,
                            owner: Some(owner),
                        },
                    )?
                }
                DerivedRequest::ShiftedPseudoTime {
                    anchor,
                    step: dt,
                    mass,
                    ..
                } => {
                    oracle.admit_nle()?;
                    let offsets = p.physical.constraints().iter().map(|r| r.lower).collect();
                    let bridge = RootBridge::new(Box::new(oracle), p.physical.clone(), offsets)?
                        .zero_residuals()?;
                    let bound = p.family.general()?.bind_pseudo_time(
                        bridge,
                        anchor.clone(),
                        *dt,
                        MassBinding::Frozen(mass.as_ref().clone()),
                    )?;
                    let oracle = FamilyRoot::new(
                        bound,
                        p.source.prepared.prepared.plan.available_order(),
                        isize::MAX as usize,
                    )?;
                    execution::roots(
                        step(p, execution.clone()),
                        retained,
                        execution::Roots {
                            oracle: Box::new(oracle),
                            initial: &initial,
                            owner: Some(owner),
                        },
                    )?
                }
                DerivedRequest::Reduced {
                    suppliers,
                    accuracy,
                    ..
                } => {
                    let prepared = p.reduced.as_ref().ok_or_else(|| {
                        ProblemError::Contract("missing selected reconstruction preparation".into())
                    })?;
                    let bridge = NlpBridge::new(Box::new(oracle), p.physical.clone())?;
                    let workers = suppliers
                        .iter()
                        .zip(&prepared.suppliers)
                        .map(|(supplier, prepared)| {
                            let normalization = prepared.normalization.clone();
                            SelectedImplicitReconstruction::<ProblemError>::new_with_binding(
                                supplier.factory.as_ref(),
                                prepared.contract.clone(),
                                normalization,
                                scope.clone(),
                                &prepared.binding,
                            )?
                            .with_proof_cell_limit(accuracy.refinement.proof_cells)
                        })
                        .collect::<Result<Vec<_>, pse_math::MathError>>()?;
                    let reconstruction = CompositeReconstruction::new_normalized(
                        prepared.contract.clone(),
                        workers,
                        &p.original.normalization,
                        &prepared
                            .suppliers
                            .iter()
                            .map(|p| p.normalization.clone())
                            .collect::<Vec<_>>(),
                    )?;
                    let bound = p.family.general()?.bind_reduced(
                        bridge,
                        reconstruction,
                        p.original_identity,
                    )?;
                    let actual = ReducedOracle::new(
                        bound,
                        &initial,
                        *accuracy,
                        p.source.prepared.prepared.plan.available_order(),
                        isize::MAX as usize,
                    )?;
                    let contract = actual.contract().clone();
                    let bounds = actual.constraint_bounds().to_vec();
                    let (_, _, entries, _) = layout(&p.family, p.reduced.as_ref())?;
                    let pattern = AssemblyMatrix::new(
                        contract.rows.len(),
                        contract.variables.len(),
                        &entries,
                        isize::MAX as usize,
                    )?;
                    let actual = Rc::new(RefCell::new(actual));
                    selected = Some(actual.clone());
                    let oracle = SharedReduced {
                        actual,
                        contract,
                        bounds,
                        pattern,
                    };
                    execution::nlp(
                        step(p, execution.clone()),
                        retained,
                        execution::Nlp {
                            oracle: Box::new(OwnedNlp {
                                oracle: Box::new(oracle),
                                _owner: owner,
                            }),
                            initial: &initial,
                            presolve: &p.profile.presolve,
                            intent: p.profile.intent,
                            sense: p
                                .source
                                .prepared
                                .prepared
                                .plan
                                .structure()
                                .objective()
                                .map_or(ObjectiveSense::Minimize, |o| o.sense),
                            limit: self.policy.worker_bytes / 256,
                            analysis: execution::Analysis::NONE,
                        },
                    )?
                }
            };
            Ok(report)
        })();
        let report = match result {
            Ok(report) => report,
            Err(cause) => {
                let cause = Arc::new(cause.into_problem());
                let refinement_product = issued_refinement(p, &cause, selected.as_ref());
                let mut failure = DerivedFailure::from(cause);
                failure.refinement_product = refinement_product;
                return Err(failure);
            }
        };
        let outcome = Outcome::Native(Box::new(report.with_owner(result_owner)));
        let observation = super::super::strategy::observe(&outcome);
        let cause = super::super::strategy::cause(&outcome);
        let permitted = proposal_observation_allowed(observation)
            && cause.as_deref().is_none_or(|cause| {
                proposal_observation_allowed(super::super::strategy::failure(cause))
            });
        let mut report = match outcome {
            Outcome::Native(report) => *report,
            _ => {
                return Err(
                    ProblemError::Internal("derived runner lost its native report".into()).into(),
                );
            }
        };
        let mut screening_calls = 0u64;
        let proposal = if let Some(candidate) = report.candidate.as_ref().filter(|_| permitted) {
            let reconstructed = match selected.as_ref() {
                Some(actual) => actual
                    .try_borrow_mut()
                    .map_err(|_| {
                        ProblemError::Contract(
                            "reconstruction still borrowed by native callback".into(),
                        )
                    })
                    .and_then(|mut oracle| oracle.original_proposal(&candidate.primal))
                    .map(|observed| (observed.values, Some(observed.accuracy))),
                None => p
                    .family
                    .reconstruct(&candidate.primal)
                    .map(|point| (point, None))
                    .map_err(ProblemError::from),
            };
            match reconstructed.and_then(|(point, accuracy)| {
                self.screen_derived_original(p, &point, &scope, budget, &mut screening_calls)
                    .map(|_| (point, accuracy))
                    .map_err(MathRuntimeError::into_problem)
            }) {
                Ok((coordinates, reconstruction_accuracy)) => {
                    // A late stop or pool refusal remains part of this completed
                    // native attempt, retaining its actual counters and cause.
                    Some(
                        execution
                            .check()
                            .and_then(|()| {
                                self.reserve(
                                    "math:derived-original-proposal",
                                    size_of::<OriginalProposal>()
                                        + coordinates.capacity() * size_of::<f64>(),
                                )
                                .map_err(MathRuntimeError::into_problem)
                            })
                            .and_then(|owner| {
                                OriginalProposal::from_original(
                                    p.original_identity,
                                    p.family.key(),
                                    coordinates,
                                    &p.physical,
                                    reconstruction_accuracy,
                                    owner,
                                )
                            }),
                    )
                }
                Err(error) => Some(Err(error)),
            }
        } else {
            None
        };
        // Internal selected reconstruction work is not present in every native counter.
        // Keep unknown accounting unknown rather than treat callbacks as total work.
        if let Some(observed) = report.evidence.work.evaluations {
            report.evidence.work.evaluations = observed.checked_add(screening_calls);
        }
        if p.reduced.is_some() {
            report.evidence.work.evaluations = None;
            report.evidence.work.factorizations = None;
            report.evidence.work.proof_steps = None;
        }
        let (proposal, screening_failure) = match proposal {
            Some(Ok(p)) => (Some(p), None),
            Some(Err(e)) => (None, Some(Arc::new(e))),
            None => (None, None),
        };
        let recorded = screening_failure
            .as_deref()
            .or_else(|| report.callback_failure());
        let refinement_product =
            recorded.and_then(|cause| issued_refinement(p, cause, selected.as_ref()));
        let applied_action = selected.as_ref().and_then(|actual| {
            actual
                .try_borrow()
                .ok()
                .and_then(|oracle| oracle.applied_action().cloned())
        });
        Ok(DerivedAttempt {
            applied_action,
            outcome: Outcome::Native(Box::new(report)),
            proposal,
            screening_failure,
            refinement_product,
        })
    }
    fn screen_derived_original(
        &self,
        p: &PreparedDerived,
        point: &[f64],
        scope: &ExecutionScope,
        budget: &Arc<WorkerBudget>,
        screening_calls: &mut u64,
    ) -> Result<(), MathRuntimeError> {
        let _screen_storage = budget.charge(
            p.physical
                .constraints()
                .len()
                .checked_add(1)
                .and_then(|n| n.checked_mul(size_of::<f64>()))
                .ok_or(MathRuntimeError::Limit("original screening storage"))?,
        )?;
        screen_bounds(&p.physical, point)?;
        scope.check().map_err(ProblemError::from)?;
        let (mut oracle, _owner) = self.derived_original_oracle(p, scope, budget)?;
        let mut rows = vec![0.0; p.physical.constraints().len()];
        *screening_calls += 1;
        NlpOracle::constraints(&mut oracle, point, &mut rows)?;
        *screening_calls += 1;
        let objective = NlpOracle::objective(&mut oracle, point)?;
        if rows.iter().any(|v| !v.is_finite()) || !objective.is_finite() {
            return Err(ProblemError::numerical(
                "original proposal evaluates to nonfinite authored values",
            )
            .into());
        }
        scope.check().map_err(ProblemError::from)?;
        Ok(())
    }
}
fn issued_refinement(
    prepared: &PreparedDerived,
    cause: &ProblemError,
    actual: Option<&Rc<RefCell<ConcreteReduced>>>,
) -> Option<pse_ids::ContentHash> {
    let (product, source_key, validity) = refinement_cause(cause)?;
    if prepared
        .reduced
        .as_ref()
        .is_none_or(|r| r.contract.source() != source_key || r.contract.validity() != validity)
    {
        return None;
    }
    actual
        .and_then(|actual| actual.try_borrow().ok())
        .filter(|actual| actual.accepts_refinement_product(product))
        .map(|_| product)
}
fn refinement_cause(
    cause: &ProblemError,
) -> Option<(
    pse_ids::ContentHash,
    pse_ids::ContentHash,
    pse_ids::ContentHash,
)> {
    fn mathematical(
        cause: &pse_math::MathError,
    ) -> Option<(
        pse_ids::ContentHash,
        pse_ids::ContentHash,
        pse_ids::ContentHash,
    )> {
        match cause {
            pse_math::MathError::Refinement {
                product,
                source_key,
                validity,
                ..
            } => Some((*product, *source_key, *validity)),
            pse_math::MathError::Instance { cause, .. } => mathematical(cause),
            _ => None,
        }
    }
    match cause {
        ProblemError::Math(cause) => mathematical(cause),
        _ => None,
    }
}
fn genuine_time_cause(cause: &ProblemError) -> bool {
    use pse_diagnostics::TypedDiagnostic;
    fn scoped_math(cause: &pse_math::MathError) -> bool {
        match cause {
            pse_math::MathError::Instance { cause, .. } => scoped_math(cause),
            pse_math::MathError::Provider {
                cause: pse_kernels::ProviderError::Deadline,
                ..
            }
            | pse_math::MathError::Scope(pse_kernels::ProviderError::Deadline) => true,
            _ => false,
        }
    }
    cause.diagnostic_code() == Some(pse_diagnostics::DiagnosticCode::RuntimeTimeout)
        || matches!(
            cause,
            ProblemError::Provider(pse_kernels::ProviderError::Deadline)
        )
        || matches!(cause,ProblemError::Math(error) if scoped_math(error))
}
pub(super) fn proposal_observation_allowed(
    observation: pse_model::generated::enums::NumericalAttemptObservation,
) -> bool {
    use pse_model::generated::enums::NumericalAttemptObservation as O;
    matches!(
        observation,
        O::Converged | O::Limited | O::NumericalFailure | O::Stalled | O::Auxiliary
    )
}
fn screen_bounds(original: &OriginalContract, point: &[f64]) -> Result<(), ProblemError> {
    if point.len() != original.coordinates().len()
        || point
            .iter()
            .zip(original.coordinates())
            .any(|(v, c)| !v.is_finite() || *v < c.lower || *v > c.upper)
    {
        return Err(ProblemError::Contract(
            "auxiliary proposal violates an original coordinate bound or extent".into(),
        ));
    }
    Ok(())
}

fn preparation_bytes(
    source: &AlgebraicCase,
    request: &DerivedRequest,
) -> Result<usize, MathRuntimeError> {
    let plan = &source.prepared.prepared.plan;
    let n = plan.columns().len();
    let m = plan
        .structure()
        .rows()
        .len()
        .checked_add(n)
        .ok_or(MathRuntimeError::Limit("derived preparation rows"))?;
    let edges = n
        .checked_mul(m)
        .ok_or(MathRuntimeError::Limit("derived preparation edges"))?;
    let matching = pse_structural::incidence::CaseIncidence::memory_extent(m, n, edges)
        .map_err(|e| ProblemError::Contract(e.to_string()))?;
    let contribution_edges = plan
        .structure()
        .instances()
        .iter()
        .try_fold(0usize, |count, b| {
            count.checked_add(b.contributions.len().checked_mul(b.slots.len())?)
        })
        .ok_or(MathRuntimeError::Limit("derived source support extent"))?;
    matching
        .checked_add(
            contribution_edges
                .checked_mul(4 * size_of::<Entry<GlobalRow, GlobalCol>>())
                .ok_or(MathRuntimeError::Limit("derived contribution edges"))?,
        )
        .and_then(|bytes| {
            bytes.checked_add((n + m).checked_mul(
                size_of::<PreparedDerivedData>()
                    + size_of::<PreparedDerived>()
                    + 16 * size_of::<f64>(),
            )?)
        })
        .and_then(|bytes| {
            bytes.checked_add(match request {
                DerivedRequest::Reduced { suppliers, .. } => {
                    suppliers.iter().try_fold(0usize, |n, s| {
                        n.checked_add(factory_bytes(&s.factory).ok()?)
                    })?
                }
                _ => 0,
            })
        })
        .ok_or(MathRuntimeError::Limit(
            "derived preparation finite workspace",
        ))
}

#[cfg(test)]
#[path = "derived_tests.rs"]
mod tests;
