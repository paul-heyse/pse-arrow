// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Deterministic class routing with explicit representation and availability failures.
//! Eligibility is a function of each adapter's published capability record and linkage
//! (F21); automatic preference only orders eligible adapters.
use crate::{
    ProblemError,
    execution::{Capability, Table},
    solve::{
        Backend, DerivativeCapability, HessianMode, ProblemClass, SolveIntent, SolverSelection,
    },
};
use pse_kernels::DerivativeOrder;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Readiness is distinct from scientific eligibility and static inventory.
pub use pse_model::generated::enums::NativeAssessmentState as AssessmentState;
/// A finite piece of evidence whose absence prevents a contextual decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvidenceDemand {
    /// Establish or rule out this intent-relevant mathematical class.
    Class(ProblemClass),
    /// Complete original-equation conservative incidence and its matching witness.
    Structure,
    /// The callback contract, guards and policy budgets needed by its owner.
    CallbackContract,
    /// The original coefficient representation and its requested guarantees.
    Coefficients,
    /// The admitted cone subtypes and representation.
    Cone,
    /// The original factorable graph and native-handler admission.
    Factorable,
}
/// A scientifically supported candidate's still-unprepared execution products.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArtifactDemand {
    /// Exact selected mathematical derivative order.
    Derivatives(DerivativeOrder),
    /// First residual action without requiring a full assembled Jacobian program.
    JacobianProduct,
    /// The selected native representation and its executable realization.
    Representation(crate::execution::Representation),
}
impl EvidenceDemand {
    /// Generated publication vocabulary; exact class payload stays typed beside it.
    pub const fn code(self) -> pse_model::generated::enums::NativeEvidenceDemand {
        use pse_model::generated::enums::NativeEvidenceDemand as E;
        match self {
            Self::Class(_) => E::Class,
            Self::Structure => E::Structure,
            Self::CallbackContract => E::CallbackContract,
            Self::Coefficients => E::Coefficients,
            Self::Cone => E::Cone,
            Self::Factorable => E::Factorable,
        }
    }
}
impl ArtifactDemand {
    /// Generated publication vocabulary; exact order/representation stay typed beside it.
    pub const fn code(self) -> pse_model::generated::enums::NativeArtifactDemand {
        use pse_model::generated::enums::NativeArtifactDemand as A;
        match self {
            Self::Derivatives(_) => A::Derivatives,
            Self::JacobianProduct => A::JacobianProduct,
            Self::Representation(_) => A::Representation,
        }
    }
}
fn evidence_classes(evidence: &[EvidenceDemand]) -> Vec<ProblemClass> {
    evidence
        .iter()
        .filter_map(|demand| match demand {
            EvidenceDemand::Class(class) => Some(*class),
            _ => None,
        })
        .collect()
}
fn required_order(artifacts: &[ArtifactDemand]) -> Option<i64> {
    artifacts
        .iter()
        .filter_map(|demand| match demand {
            ArtifactDemand::Derivatives(order) => Some(*order as i64),
            _ => None,
        })
        .max()
}
fn artifact_representations(artifacts: &[ArtifactDemand]) -> Vec<crate::execution::Representation> {
    artifacts
        .iter()
        .filter_map(|demand| match demand {
            ArtifactDemand::Representation(representation) => Some(*representation),
            _ => None,
        })
        .collect()
}
/// Complete original structural inputs, shared by every candidate interpretation.
#[derive(Clone, Debug)]
pub struct Structure {
    /// All original free columns.
    pub variables: Vec<pse_ids::SemanticId>,
    /// All original rows, including inequalities and isolated rows.
    pub equations: Vec<pse_structural::incidence::Constraint>,
    /// Existing library matching, never a candidate-specific second search.
    pub witness: pse_math::SharedAllocation<pse_structural::incidence::StructuralAnalysis>,
}
/// One admitted cone representation paired with proof for that exact matrix/orientation.
#[derive(Clone, Copy, Debug)]
pub struct ConeEvidence<'a> {
    /// The actual cone-space representation.
    pub problem: &'a crate::ConicProblem,
    /// PSD evidence in these same cone coordinates and minimization orientation.
    pub certificate: &'a dyn pse_math::convexity::QuadraticEvidence,
}
/// Immutable contextual inputs supplied by the composition boundary.
#[derive(Clone, Debug)]
pub struct Context<'a> {
    /// Build/runtime observation captured before policy assessment.
    pub snapshot: crate::execution::Snapshot,
    /// Missing class proofs relevant to this intent/selection, in class priority order.
    pub pending_classes: &'a [ProblemClass],
    /// Established candidate-specific scientific/representation incompatibilities.
    /// Resource, cancellation, infrastructure and attempt failures are never entered here.
    pub refusals: &'a BTreeMap<Backend, Arc<ProblemError>>,
    /// Complete original structural evidence; absent means pending evidence.
    pub structure: Option<Structure>,
    /// Original callback metadata, without acquiring any evaluator/worker.
    pub oracle: Option<&'a crate::OracleContract>,
    /// Existing checked guard facts, used by root/sign admission.
    pub guards: &'a BTreeMap<pse_ids::SemanticId, pse_math::presolve::GuardSign>,
    /// Existing physical/native policy budgets used by callback settings admission.
    pub budgets: Option<crate::execution::Budgets<'a>>,
    /// Established original coefficient representation, when demanded.
    pub coefficients: Option<&'a crate::CoefficientProblem>,
    /// Original-space PSD proof consumed by coefficient/cone admission.
    pub certificate: Option<&'a dyn pse_math::convexity::QuadraticEvidence>,
    /// Established original cone representation, when demanded.
    pub cone: Option<ConeEvidence<'a>>,
    /// Established original factorable representation, when demanded.
    pub factorable: Option<&'a pse_math::factorable::FactorableProgram>,
    /// Already prepared products for this exact immutable case/profile.
    pub prepared: &'a [ArtifactDemand],
}

use pse_math::facts::{BoundShape, ProblemFacts};
use pse_model::generated::enums::{
    ModelingStructuralRequirement, ModelingVariableDomain, NativeConstraintForm,
    NativeIneligibility,
};
/// Selected execution class, including the zero-variable path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    /// Direct original-model evaluation, without a native solver attempt.
    Constant,
    /// Native implementation and its separately admitted representation.
    Native(Backend),
}
/// Native degradation semantics are registry-owned.
pub use pse_model::generated::enums::NativeLexicographicDegradation as DegradationSupport;
/// Registry-owned realization of authored priorities.
pub use pse_model::generated::enums::NativeLexicographicRealization as Lexicographic;
/// Attributable refusal independent of whether a native attempt exists.
#[derive(Clone, Debug)]
pub enum Refusal {
    /// Invalid request retained independently of capability eligibility.
    InvalidRequest(String),
    /// No admitted automatic representation.
    NoEligible,
    /// Requested backend is absent from the linked table.
    Unavailable(Backend),
    /// Requested backend cannot represent this request.
    Ineligible(Backend),
    /// Constant evaluation cannot consume native forms.
    ConstantNativeForms,
    /// Original-coordinate structural admission refused.
    Structure,
}
/// Complete route facts retained on success and refusal.
#[derive(Clone, Debug)]
pub struct Decision {
    /// Authored mathematical purpose.
    pub intent: SolveIntent,
    /// Requested automatic or explicit selection.
    pub selection: SolverSelection,
    /// Compiler-derived classes in preference order.
    pub classes: Vec<ProblemClass>,
    /// Every contextual capability assessment.
    pub eligibility: Vec<Eligibility>,
    /// Selected route, absent on refusal.
    pub selected: Option<Route>,
    /// Chosen native representation, retained from the selected capability table.
    pub representation: Option<crate::execution::Representation>,
    /// Native/staged priority realization, absent for one objective.
    pub lexicographic: Option<Lexicographic>,
    /// Original structural facts, including refused witness.
    pub structure: Option<crate::structural::Assessment>,
    /// Admission refusal, never inferred from an absent candidate.
    pub refusal: Option<Refusal>,
    /// Contextual readiness of this decision.
    pub state: AssessmentState,
    /// Finite missing evidence; no eligibility conclusion is inferred from its absence.
    pub evidence: Vec<EvidenceDemand>,
    /// Highest-priority unresolved candidate whose contextual evidence is requested.
    pub pending_backend: Option<Backend>,
    /// Execution products demanded by the selected scientifically supported candidate.
    pub artifacts: Vec<ArtifactDemand>,
    /// Exact build/runtime observation consumed by this assessment.
    pub snapshot: pse_ids::ContentHash,
}
impl Decision {
    /// Return the admitted route or preserve all refusal facts in its typed cause.
    pub fn route(&self) -> Result<Route, ProblemError> {
        self.selected
            .filter(|_| self.refusal.is_none() && self.state == AssessmentState::Ready)
            .ok_or_else(|| ProblemError::RouteRefused(Box::new(self.clone())))
    }
    /// Select the established candidate whose pending artifacts the caller must prepare.
    /// Missing decision evidence or a contextual refusal never selects a candidate.
    pub fn candidate(&self) -> Result<Route, ProblemError> {
        self.selected
            .filter(|_| {
                self.refusal.is_none()
                    && matches!(
                        self.state,
                        AssessmentState::Ready | AssessmentState::SupportedPendingArtifacts
                    )
            })
            .ok_or_else(|| ProblemError::RouteRefused(Box::new(self.clone())))
    }
    /// Publish retained admission facts without manufacturing a numerical run identity.
    pub fn row(
        &self,
        request_identity: pse_ids::ContentHash,
        step: i64,
    ) -> pse_model::generated::runtime::route_decisions::Row {
        use pse_model::generated::{
            enums::{
                NativeRouteKind as Kind, NativeRouteRefusal as Refused,
                NativeRouteSelection as Selection,
            },
            runtime::route_decisions::*,
        };
        let (selection, requested_backend) = match self.selection {
            SolverSelection::Auto => (Selection::Auto, None),
            SolverSelection::Explicit(backend) => (Selection::Explicit, Some(backend)),
        };
        let (selected, backend) = match self.selected {
            None => (None, None),
            Some(Route::Constant) => (Some(Kind::Constant), None),
            Some(Route::Native(backend)) => (Some(Kind::Native), Some(backend)),
        };
        let refusal = self.refusal.as_ref().map(|reason| match reason {
            Refusal::InvalidRequest(_) => Refused::InvalidRequest,
            Refusal::NoEligible => Refused::NoEligible,
            Refusal::Unavailable(_) => Refused::Unavailable,
            Refusal::Ineligible(_) => Refused::Ineligible,
            Refusal::ConstantNativeForms => Refused::ConstantNativeForms,
            Refusal::Structure => Refused::Structure,
        });
        RuntimeRouteDecisionsRow {
            request_identity,
            step,
            intent: self.intent,
            selection,
            requested_backend,
            classes: self.classes.clone(),
            state: self.state,
            snapshot: self.snapshot,
            evidence: self.evidence.iter().map(|demand| demand.code()).collect(),
            artifacts: self.artifacts.iter().map(|demand| demand.code()).collect(),
            evidence_classes: evidence_classes(&self.evidence),
            required_order: required_order(&self.artifacts),
            artifact_representations: artifact_representations(&self.artifacts),
            pending_backend: self.pending_backend,
            eligibility: self
                .eligibility
                .iter()
                .map(|entry| RuntimeRouteDecisionsFieldEligibilityItem {
                    backend: entry.backend,
                    reasons: entry.reasons.iter().map(Ineligible::code).collect(),
                    state: entry.state,
                    evidence: entry.evidence.iter().map(|demand| demand.code()).collect(),
                    artifacts: entry.artifacts.iter().map(|demand| demand.code()).collect(),
                    evidence_classes: evidence_classes(&entry.evidence),
                    required_order: required_order(&entry.artifacts),
                    artifact_representations: artifact_representations(&entry.artifacts),
                    structural_mode: entry.structure.as_ref().map(|structure| structure.mode),
                    structurally_admitted: entry
                        .structure
                        .as_ref()
                        .map(|structure| structure.refusal.is_none()),
                })
                .collect(),
            selected,
            backend,
            representation: self.representation,
            lexicographic: self.lexicographic,
            refusal,
            detail: self.refusal.as_ref().map(|_| self.to_string()),
        }
    }
    /// Bytes retained outside a fixed route receipt.
    pub fn retained_bytes(&self) -> usize {
        let structure_bytes = |structure: &crate::structural::Assessment| {
            structure.variables.capacity() * size_of::<pse_ids::SemanticId>()
                + structure.equations.capacity()
                    * size_of::<pse_structural::incidence::Constraint>()
                + structure.refusal.as_ref().map_or(0, |(rows, columns)| {
                    (rows.capacity() + columns.capacity()) * size_of::<pse_ids::SemanticId>()
                })
        };
        self.classes.capacity() * size_of::<ProblemClass>()
            + self.evidence.capacity() * size_of::<EvidenceDemand>()
            + self.artifacts.capacity() * size_of::<ArtifactDemand>()
            + self.eligibility.capacity() * size_of::<Eligibility>()
            + self
                .eligibility
                .iter()
                .map(|entry| {
                    entry.reasons.capacity() * size_of::<Ineligible>()
                        + entry.causes.capacity() * size_of::<Arc<ProblemError>>()
                        + entry.class_dependencies.capacity()
                            * size_of::<pse_math::presolve::ClassDependency>()
                        + entry
                            .causes
                            .iter()
                            .map(|cause| cause.retained_bytes())
                            .sum::<usize>()
                        + entry.evidence.capacity() * size_of::<EvidenceDemand>()
                        + entry.artifacts.capacity() * size_of::<ArtifactDemand>()
                        + entry.factorable_refusals.capacity()
                            * size_of::<crate::execution::Refusal>()
                        + entry.structure.as_ref().map_or(0, structure_bytes)
                })
                .sum::<usize>()
            + self.structure.as_ref().map_or(0, structure_bytes)
    }
}
impl std::fmt::Display for Decision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {:?}: {:?}; {}",
            self.intent,
            self.selection,
            self.refusal,
            assessed(&self.eligibility)
        )
    }
}
/// Why one adapter cannot represent this request; every applicable reason is reported.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ineligible {
    /// The binary does not link the adapter.
    NotLinked,
    /// More than one thread was requested from a serial adapter.
    Serial,
    /// Root intents need square continuous equalities without an objective.
    NotSquareRoot,
    /// Optimization needs an authored objective.
    NoObjective,
    /// The certify intent needs an adapter whose record certifies global bounds.
    Certification,
    /// None of the problem's classes is among the adapter's classes.
    Class {
        /// Classes the facts and intent establish for this problem.
        problem: Vec<ProblemClass>,
    },
    /// Scientifically available selected smooth derivatives are below the requirement.
    Derivatives {
        /// Required scientific order.
        required: DerivativeOrder,
    },
    /// A bound shape the adapter cannot represent.
    Bounds {
        /// The adapter still represents one-sided bounds as shifted sign constraints.
        signs: bool,
    },
    /// The structure leaves constraint forms to native handlers the adapter's record does
    /// not consume (ADR-0104); there is no silent linear conversion.
    NativeForms {
        /// Required forms outside the adapter's record, in order.
        missing: Vec<NativeConstraintForm>,
    },
    /// A Gauss–Newton Hessian needs a least-squares objective; only fits state one.
    LeastSquares,
    /// The formulation states structural requirements (ADR-0104 §5) that the adapter's record
    /// does not honour, or that the request's settings select a method that does not.
    Method {
        /// Unmet requirements, in order.
        requirements: Vec<ModelingStructuralRequirement>,
    },
    /// The adapter-owned settings or representation contract refuses this case.
    Contextual,
    /// The candidate's original structural interpretation refuses this case.
    Structural,
    /// Several objectives require an admitted native or staged lexicographic sequence.
    Lexicographic {
        /// Classes the facts and intent establish for this problem.
        problem: Vec<ProblemClass>,
    },
}
impl std::fmt::Display for Ineligible {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Contextual => f.write_str("adapter-owned contextual contract refused"),
            Self::Structural => f.write_str("original structural interpretation refused"),
            Self::NotLinked => f.write_str("adapter not linked"),
            Self::Serial => f.write_str("linked profile is serial"),
            Self::NotSquareRoot => f.write_str(
                "root analysis requires square continuous equalities without an objective",
            ),
            Self::NoObjective => f.write_str("optimization requires an authored objective"),
            Self::Certification => f.write_str("adapter does not certify global bounds"),
            Self::Class { problem } => {
                f.write_str("problem class ")?;
                if problem.is_empty() {
                    f.write_str("(none)")?;
                }
                for (i, class) in problem.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    f.write_str(class.as_str())?;
                }
                f.write_str(" is not represented by this adapter")
            }
            Self::Derivatives { required } => write!(
                f,
                "requires available smooth {} derivatives",
                match required {
                    DerivativeOrder::Second => "second",
                    _ => "first",
                }
            ),
            Self::Bounds { signs: true } => f.write_str(
                "cannot represent two-sided bounds; only one-sided (shifted sign) bounds",
            ),
            Self::Bounds { signs: false } => f.write_str("cannot represent variable bounds"),
            Self::NativeForms { missing } => write!(
                f,
                "native {} realization needs constraint handlers this adapter lacks",
                native_forms(missing)
            ),
            Self::LeastSquares => {
                f.write_str("a Gauss–Newton Hessian requires a least-squares fit objective")
            }
            Self::Method { requirements } => write!(
                f,
                "the formulation's {} requirement needs a method this adapter or its settings do not select",
                requirements
                    .iter()
                    .map(|r| r.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Lexicographic { problem } => {
                f.write_str("several objectives in problem class ")?;
                for (i, class) in problem.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    f.write_str(class.as_str())?;
                }
                f.write_str(" are not optimized lexicographically by this adapter")
            }
        }
    }
}
impl Ineligible {
    /// Registry reason code: the one name of this reason across the Python boundary.
    pub const fn code(&self) -> NativeIneligibility {
        match self {
            Self::Contextual => NativeIneligibility::Contextual,
            Self::Structural => NativeIneligibility::Structural,
            Self::NotLinked => NativeIneligibility::NotLinked,
            Self::Serial => NativeIneligibility::Serial,
            Self::NotSquareRoot => NativeIneligibility::NotSquareRoot,
            Self::NoObjective => NativeIneligibility::NoObjective,
            Self::Certification => NativeIneligibility::Certification,
            Self::Class { .. } => NativeIneligibility::Class,
            Self::Derivatives { .. } => NativeIneligibility::Derivatives,
            Self::Bounds { .. } => NativeIneligibility::Bounds,
            Self::NativeForms { .. } => NativeIneligibility::NativeForms,
            Self::LeastSquares => NativeIneligibility::LeastSquares,
            Self::Method { .. } => NativeIneligibility::Method,
            Self::Lexicographic { .. } => NativeIneligibility::Lexicographic,
        }
    }
}
/// One adapter's contextual assessment; inventory alone never grants eligibility.
#[derive(Clone, Debug)]
pub struct Eligibility {
    /// Backend whose concrete representation is assessed.
    pub backend: Backend,
    /// Every applicable refusal; empty means eligible.
    pub reasons: Vec<Ineligible>,
    /// Original typed causes supplied by settings/representation/structural owners.
    pub causes: Vec<Arc<ProblemError>>,
    /// Original unresolved class dependencies, retaining actual instance/output attribution.
    pub class_dependencies: Vec<pse_math::presolve::ClassDependency>,
    /// Finite missing decision evidence.
    pub evidence: Vec<EvidenceDemand>,
    /// Supported but unprepared execution products.
    pub artifacts: Vec<ArtifactDemand>,
    /// Existing factorable owner refusals, retained without rendering away their types.
    pub factorable_refusals: Vec<crate::execution::Refusal>,
    /// This candidate's interpretation of the shared original witness.
    pub structure: Option<crate::structural::Assessment>,
    /// Contextual readiness, never inferred merely from empty static reasons.
    pub state: AssessmentState,
}
impl Eligibility {
    /// Retain an original typed owner refusal without replacing its cause.
    pub fn refuse(&mut self, cause: ProblemError) {
        if !self.reasons.contains(&Ineligible::Contextual) {
            self.reasons.push(Ineligible::Contextual);
        }
        self.causes.push(Arc::new(cause));
    }
    /// Derive readiness from the complete assessment, in semantic precedence order.
    pub fn finish(&mut self) {
        self.evidence.dedup();
        self.artifacts.dedup();
        self.state = if !self.reasons.is_empty() || !self.causes.is_empty() {
            AssessmentState::Refused
        } else if !self.evidence.is_empty() {
            AssessmentState::PendingEvidence
        } else if !self.artifacts.is_empty() {
            AssessmentState::SupportedPendingArtifacts
        } else {
            AssessmentState::Ready
        };
    }
}
impl std::fmt::Display for Eligibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: ", self.backend.as_str())?;
        if self.reasons.is_empty() {
            return f.write_str("eligible");
        }
        for (i, reason) in self.reasons.iter().enumerate() {
            if i > 0 {
                f.write_str("; ")?;
            }
            write!(f, "{reason}")?;
        }
        Ok(())
    }
}
/// Every assessment, one adapter per line segment.
fn assessed(choices: &[Eligibility]) -> String {
    choices
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" | ")
}
/// Requirements established by the selected mathematical representation and policy.
#[derive(Debug)]
pub struct Requirements<'a> {
    /// Adapters exposed to this request: the linked table in production.
    pub table: &'a Table,
    /// Compiler-established class and derivative facts.
    pub facts: &'a ProblemFacts,
    /// Selected analysis purpose.
    pub intent: SolveIntent,
    /// This request's explicit numerical convexity policy qualified its coefficient
    /// objective as positive semidefinite (ADR-0121 Outcome 4): `convex_quadratic` for this
    /// request only. It is never a fact; without it only `facts.convexity` counts.
    pub numerical_psd: bool,
    /// The objective is a weighted least-squares sum with a response Jacobian, the only
    /// objective whose Gauss–Newton Hessian is defined (a fit).
    pub least_squares: bool,
    /// Complete effective native controls.
    pub controls: &'a crate::solve::Controls,
    /// The request's typed backend settings, which select a method.
    pub settings: &'a crate::execution::BackendSettings,
    /// The request asks for parametric sensitivities at its candidate (ADR-0118): automatic
    /// routing prefers an adapter whose candidate carries the multipliers the KKT-point
    /// analysis differentiates. An explicit selection of any other adapter still solves,
    /// and the quantities are withheld with their reason.
    pub sensitivity: bool,
    /// Explicit immutable case/profile/build context consumed by this assessment.
    pub context: Context<'a>,
}
/// Whether authored-model routing needs original coefficient-class proof before choice.
/// Callback-only workflows state their available representation separately; they do not
/// invent symbolic export evidence from numerical callbacks.
pub fn class_evidence_required(
    facts: &ProblemFacts,
    intent: SolveIntent,
    selection: SolverSelection,
) -> bool {
    if facts.variables == 0 || root_intent(facts, intent) {
        return false;
    }
    if !matches!(
        facts.class_status,
        pse_math::presolve::ClassStatus::Unassessed
    ) {
        return false;
    }
    match selection {
        SolverSelection::Auto => true,
        SolverSelection::Explicit(backend) => matches!(
            crate::execution::adapter(backend).representation(),
            crate::execution::Representation::Coefficients | crate::execution::Representation::Cone
        ),
    }
}
/// Unknown authored coefficient proofs remain finite evidence demands, never a negative class.
pub fn pending_class_evidence(
    facts: &ProblemFacts,
    intent: SolveIntent,
    selection: SolverSelection,
) -> Vec<ProblemClass> {
    if facts.variables == 0
        || root_intent(facts, intent)
        || !matches!(
            facts.class_status,
            pse_math::presolve::ClassStatus::Pending(_)
                | pse_math::presolve::ClassStatus::Unassessed
        )
    {
        return vec![];
    }
    if let SolverSelection::Explicit(backend) = selection
        && !matches!(
            crate::execution::adapter(backend).representation(),
            crate::execution::Representation::Coefficients | crate::execution::Representation::Cone
        )
    {
        return vec![];
    }
    if continuous(facts) {
        vec![
            ProblemClass::Linear,
            ProblemClass::ConvexQuadratic,
            ProblemClass::NonconvexQuadratic,
        ]
    } else {
        vec![
            ProblemClass::MixedLinear,
            ProblemClass::MixedIntegerQuadratic,
        ]
    }
}
/// Project an already admitted native oracle into the same contextual selector.
/// Callers supply the represented objective/equality meaning, not a backend preference.
pub fn oracle_facts(c: &crate::OracleContract, objective: bool, equalities: bool) -> ProblemFacts {
    ProblemFacts {
        class_status: pse_math::presolve::ClassStatus::Unassessed,
        variables: c.variables.len(),
        rows: c.rows.len(),
        objective,
        objectives: usize::from(objective),
        equalities,
        domains: vec![ModelingVariableDomain::Continuous; c.variables.len()],
        derivatives: c.derivatives.min(c.smoothness),
        prepared_derivatives: c.derivatives,
        bounds: c
            .variables
            .iter()
            .map(|v| match (v.lower.is_finite(), v.upper.is_finite()) {
                (false, false) => BoundShape::Free,
                (true, true) => BoundShape::Boxed,
                (true, false) if v.lower == 0.0 => BoundShape::Nonnegative,
                (false, true) if v.upper == 0.0 => BoundShape::Nonpositive,
                (true, false) => BoundShape::Lower,
                (false, true) => BoundShape::Upper,
            })
            .collect(),
        guarded: false,
        coefficients: false,
        affine_rows: vec![false; c.rows.len()],
        objective_degree: None,
        bound_assumptions: c.identity,
        quadratic: false,
        native: vec![],
        requirements: vec![],
        convexity: pse_math::convexity::Convexity::not_assessed(c.identity),
    }
}
/// Facts established by a validated explicit convex-cone representation.
/// Cone membership itself establishes this class; no callback coefficient proof is invented.
pub fn conic_facts(
    problem: &crate::ConicProblem,
    certificate: &dyn pse_math::convexity::QuadraticEvidence,
) -> Result<ProblemFacts, ProblemError> {
    use crate::conic::Cone;
    problem.validate(certificate)?;
    let mut summary = pse_math::convexity::ConeSummary::default();
    for cone in &problem.cones {
        match cone {
            Cone::Zero { .. } | Cone::PsdTriangle { .. } => {}
            Cone::Nonnegative { dimension } => summary.nonnegative += dimension,
            Cone::SecondOrder { .. } => summary.second_order += 1,
            Cone::Exponential => summary.exponential += 1,
            Cone::Power { .. } | Cone::GeneralizedPower { .. } => summary.power += 1,
        }
    }
    let mut facts = oracle_facts(
        &problem.contract,
        true,
        problem
            .cones
            .iter()
            .all(|cone| matches!(cone, Cone::Zero { .. })),
    );
    facts.convexity = pse_math::convexity::Convexity {
        key: problem.contract.identity,
        class: pse_math::convexity::ConvexityClass::Cone(summary),
    };
    Ok(facts)
}
fn continuous(f: &ProblemFacts) -> bool {
    f.domains
        .iter()
        .all(|d| *d == ModelingVariableDomain::Continuous)
}
fn square_root(f: &ProblemFacts) -> bool {
    f.equalities && f.rows == f.variables && !f.objective && continuous(f)
}
/// Initialization retains the mathematical class of its actual representation.
/// A square equality system is root initialization; an authored auxiliary objective
/// or inequality/freedom is NLP initialization, without changing the caller's intent.
pub(crate) fn root_intent(facts: &ProblemFacts, intent: SolveIntent) -> bool {
    intent == SolveIntent::Root || intent == SolveIntent::Initialize && square_root(facts)
}
/// Conservative compilation demand of one adapter's declared derivative capability.
/// Numerical callers and eligibility consume this same operation.
pub fn derivative_demand(
    capability: &Capability,
    controls: &crate::solve::Controls,
) -> Option<DerivativeOrder> {
    match capability.derivatives {
        DerivativeCapability::ExactHessianOrLimitedMemory
            if matches!(
                controls.hessian,
                HessianMode::Exact | HessianMode::GaussNewton
            ) =>
        {
            Some(DerivativeOrder::Second)
        }
        DerivativeCapability::ExactHessianOrLimitedMemory
        | DerivativeCapability::JacobianOrProduct
        | DerivativeCapability::ForwardAndAdjointSensitivities
        | DerivativeCapability::SecondOrderAdjointSensitivities => Some(DerivativeOrder::First),
        DerivativeCapability::Coefficients | DerivativeCapability::Factorable => None,
    }
}

/// The mathematical classes the facts and intent establish (ADR-0106 §7), most specific
/// first: a square root system, then the coefficient, cone or discrete class, then smooth
/// NLP. Root intents make a square problem a root system; coefficient, cone and discrete
/// classes are optimization classes. Convexity is the preparation's fact (ADR-0121): a
/// degree-two coefficient problem is convex only with its exact certificate, or, for one
/// request, its explicit numerical qualification (`numerical_psd`); a continuous nonlinear
/// problem is a continuous cone problem when the curvature pass recognized it. A trajectory
/// is never inferred from algebraic facts.
pub fn problem_classes(
    f: &ProblemFacts,
    intent: SolveIntent,
    numerical_psd: bool,
) -> Vec<ProblemClass> {
    let mut classes = Vec::new();
    if root_intent(f, intent) && square_root(f) {
        classes.push(ProblemClass::SquareRoot);
    }
    if !root_intent(f, intent) {
        let convex = f.convexity.convex_quadratic().is_some() || numerical_psd;
        let coefficient_class = f.coefficients
            || matches!(
                f.class_status,
                pse_math::presolve::ClassStatus::Established
                    | pse_math::presolve::ClassStatus::RepresentationLimited(_)
            ) && f.affine_rows.iter().all(|affine| *affine)
                && f.objective_degree.is_some_and(|degree| degree <= 2);
        let quadratic = f.quadratic || !f.coefficients && f.objective_degree == Some(2);
        match (coefficient_class, quadratic, continuous(f)) {
            (true, false, true) => classes.push(ProblemClass::Linear),
            (true, false, false) => classes.push(ProblemClass::MixedLinear),
            (true, true, true) if convex => classes.push(ProblemClass::ConvexQuadratic),
            (true, true, true) => classes.push(ProblemClass::NonconvexQuadratic),
            (true, true, false) => classes.push(ProblemClass::MixedIntegerQuadratic),
            (false, _, false) => classes.push(ProblemClass::MixedIntegerNonlinear),
            (false, _, true) if f.convexity.cone() => classes.push(ProblemClass::ContinuousCone),
            (false, _, true) => {}
        }
    }
    if continuous(f) {
        classes.push(ProblemClass::SmoothNlp);
    }
    classes
}
/// The one eligibility rule: a function of an adapter's capability record, its linkage and
/// the request. Every adapter is assessed through it.
pub fn admit(
    backend: Backend,
    capability: &Capability,
    linked: bool,
    r: &Requirements<'_>,
) -> Vec<Ineligible> {
    let f = r.facts;
    let mut reasons = Vec::new();
    if !linked {
        reasons.push(Ineligible::NotLinked);
    }
    if r.controls.threads != 1 && !capability.parallel {
        reasons.push(Ineligible::Serial);
    }
    if root_intent(f, r.intent) && !square_root(f) {
        reasons.push(Ineligible::NotSquareRoot);
    }
    if r.intent == SolveIntent::Optimize && !f.objective {
        reasons.push(Ineligible::NoObjective);
    }
    // Certification is only ever explicit and served only by a certifying record.
    if r.intent == SolveIntent::Certify && !capability.certifies {
        reasons.push(Ineligible::Certification);
    }
    let classes = problem_classes(f, r.intent, r.numerical_psd);
    if f.objectives > 1 && !classes.iter().any(|c| capability.lexicographic.contains(c)) {
        reasons.push(Ineligible::Lexicographic {
            problem: classes.clone(),
        });
    }
    if !classes.iter().any(|c| capability.classes.contains(c))
        && !r
            .context
            .pending_classes
            .iter()
            .any(|c| capability.classes.contains(c))
    {
        reasons.push(Ineligible::Class { problem: classes });
    }
    if r.controls.hessian == HessianMode::GaussNewton && !r.least_squares {
        reasons.push(Ineligible::LeastSquares);
    }
    // Every mode other than the library's quasi-Newton approximation is a supplied
    // Hessian: exact, or the Gauss–Newton Gram with constraint curvature.
    let required = r
        .table
        .get(backend)
        .and_then(|adapter| adapter.required_order(r));
    if let Some(required) = required
        && f.derivatives < required
    {
        reasons.push(Ineligible::Derivatives { required });
    }
    if !capability.general_bounds
        && !f.bounds.iter().all(|b| match b {
            BoundShape::Free => true,
            // A one-sided bound is a sign bound on shifted coordinates (Plan 22 Y6).
            BoundShape::Nonnegative
            | BoundShape::Nonpositive
            | BoundShape::Lower
            | BoundShape::Upper => capability.sign_bounds,
            BoundShape::Boxed => false,
        })
    {
        reasons.push(Ineligible::Bounds {
            signs: capability.sign_bounds,
        });
    }
    let missing: Vec<_> = f
        .native
        .iter()
        .filter(|form| !capability.native_forms.contains(form))
        .copied()
        .collect();
    if !missing.is_empty() {
        reasons.push(Ineligible::NativeForms { missing });
    }
    // A structural requirement is a fact of the formulation (ADR-0104 §5): only a record that
    // honours it, run with settings whose method does, is eligible. An authored realization
    // is the author's selection, so nothing here is chosen automatically.
    let unmet: Vec<_> = f
        .requirements
        .iter()
        .filter(|q| !capability.requirements.contains(q) || !r.settings.honours(backend, **q))
        .copied()
        .collect();
    if !unmet.is_empty() {
        reasons.push(Ineligible::Method {
            requirements: unmet,
        });
    }
    reasons
}
/// Interpret shared original evidence separately for each candidate before suitability.
pub fn assess_static(
    backend: Backend,
    capability: &Capability,
    requirements: &Requirements<'_>,
) -> Eligibility {
    let mut assessment = Eligibility {
        backend,
        reasons: admit(
            backend,
            capability,
            requirements.context.snapshot.linked(backend),
            requirements,
        ),
        class_dependencies: if requirements.context.pending_classes.is_empty() {
            vec![]
        } else {
            match &requirements.facts.class_status {
                pse_math::presolve::ClassStatus::Pending(dependencies) => dependencies.clone(),
                _ => vec![],
            }
        },
        causes: vec![],
        evidence: requirements
            .context
            .pending_classes
            .iter()
            .copied()
            .map(EvidenceDemand::Class)
            .collect(),
        artifacts: vec![],
        factorable_refusals: vec![],
        structure: None,
        state: AssessmentState::PendingEvidence,
    };
    if let Some(original) = &requirements.context.structure {
        let mode = crate::structural::mode(
            capability.structural,
            requirements.facts,
            requirements.intent,
        );
        match crate::structural::Assessment::new(
            mode,
            original.variables.clone(),
            original.equations.clone(),
            original.witness.clone(),
        ) {
            Ok(structure) => {
                if let Err(cause) = structure.admit() {
                    assessment.reasons.push(Ineligible::Structural);
                    assessment.causes.push(Arc::new(cause));
                }
                assessment.structure = Some(structure);
            }
            Err(cause) => assessment.refuse(cause),
        }
    } else {
        assessment.evidence.push(EvidenceDemand::Structure);
    }
    if let Some(required) = requirements
        .table
        .get(backend)
        .and_then(|adapter| adapter.required_artifact(requirements))
    {
        let missing = match required {
            ArtifactDemand::Derivatives(order) => {
                requirements.facts.derivatives >= order
                    && requirements.facts.prepared_derivatives < order
            }
            ArtifactDemand::JacobianProduct => {
                requirements.facts.derivatives >= DerivativeOrder::First
                    && !requirements.context.prepared.contains(&required)
            }
            ArtifactDemand::Representation(_) => !requirements.context.prepared.contains(&required),
        };
        if missing {
            assessment.artifacts.push(required);
        }
    }
    assessment
}
impl Requirements<'_> {
    /// Assess every exposed algebraic adapter through the one eligibility rule.
    pub fn eligibility(&self) -> Vec<Eligibility> {
        self.table
            .adapters()
            .filter(|adapter| crate::execution::algebraic(adapter.representation()))
            .map(|adapter| adapter.assess(self))
            .collect()
    }
    /// Deterministic route; explicit selection never silently falls back.
    /// Assess once and retain every fact even when routing refuses.
    pub fn decision(&self, selection: SolverSelection) -> Decision {
        self.decide_assessed(selection, self.eligibility(), None)
    }
    /// Reuse the same original witness for solving, diagnosis and conformance.
    pub fn bound_decision(
        &self,
        selection: SolverSelection,
        variables: Vec<pse_ids::SemanticId>,
        equations: Vec<pse_structural::incidence::Constraint>,
        witness: pse_math::SharedAllocation<pse_structural::incidence::StructuralAnalysis>,
    ) -> Result<Decision, ProblemError> {
        let mut context = self.context.clone();
        context.structure = Some(Structure {
            variables,
            equations,
            witness,
        });
        let requirements = Requirements {
            context,
            table: self.table,
            facts: self.facts,
            intent: self.intent,
            numerical_psd: self.numerical_psd,
            least_squares: self.least_squares,
            controls: self.controls,
            settings: self.settings,
            sensitivity: self.sensitivity,
        };
        Ok(requirements.decision(selection))
    }
    fn decide_assessed(
        &self,
        selection: SolverSelection,
        eligibility: Vec<Eligibility>,
        lexicographic: Option<Lexicographic>,
    ) -> Decision {
        // Candidates needing evidence keep their class/rank position. They cannot be
        // silently skipped in favour of a lower-class or already-prepared candidate.
        let ordered = |sensitivities: bool| {
            let mut classes = self.context.pending_classes.to_vec();
            for class in problem_classes(self.facts, self.intent, self.numerical_psd) {
                if !classes.contains(&class) {
                    classes.push(class);
                }
            }
            classes.into_iter().find_map(|class| {
                self.table
                    .adapters()
                    .filter(|a| a.capability().automatic_classes.contains(&class))
                    .filter(|a| !sensitivities || a.capability().sensitivities)
                    .filter_map(|a| a.automatic().map(|rank| (rank, a.backend())))
                    .filter(|(_, backend)| {
                        eligibility.iter().any(|entry| {
                            entry.backend == *backend && entry.state != AssessmentState::Refused
                        })
                    })
                    .min_by_key(|(rank, _)| *rank)
                    .map(|(_, backend)| backend)
            })
        };
        // A point evaluation consumes original source/structure, not native solver
        // evidence. Native pending candidates must not prevent its own admission.
        let preferred = if self.facts.variables == 0 {
            None
        } else {
            match selection {
                SolverSelection::Explicit(backend) => Some(backend),
                SolverSelection::Auto => self
                    .sensitivity
                    .then(|| ordered(true))
                    .flatten()
                    .or_else(|| ordered(false)),
            }
        };
        let preferred_entry =
            preferred.and_then(|backend| eligibility.iter().find(|entry| entry.backend == backend));
        let mut evidence = preferred_entry.map_or_else(Vec::new, |entry| entry.evidence.clone());
        let pending =
            preferred_entry.is_some_and(|entry| entry.state == AssessmentState::PendingEvidence);
        let result = if let Err(cause) = self.controls.validate() {
            Some(Err(cause))
        } else if pending {
            None
        } else {
            Some(self.select_assessed(selection, &eligibility))
        };
        let (selected, mut refusal) = match result {
            None => (None, None),
            Some(Ok(route)) => (Some(route), None),
            Some(Err(ProblemError::Contract(detail))) => {
                (None, Some(Refusal::InvalidRequest(detail)))
            }
            Some(Err(ProblemError::Unavailable { backend, .. })) => {
                (None, Some(Refusal::Unavailable(backend)))
            }
            Some(Err(_)) if self.facts.variables == 0 && !self.facts.native.is_empty() => {
                (None, Some(Refusal::ConstantNativeForms))
            }
            Some(Err(_)) => (
                None,
                Some(match selection {
                    SolverSelection::Auto => Refusal::NoEligible,
                    SolverSelection::Explicit(backend) => Refusal::Ineligible(backend),
                }),
            ),
        };
        let chosen = selected.and_then(|route| match route {
            Route::Native(backend) => eligibility.iter().find(|entry| entry.backend == backend),
            Route::Constant => None,
        });
        let mut structure = chosen.and_then(|entry| entry.structure.clone());
        if selected == Some(Route::Constant) {
            if let Some(original) = &self.context.structure {
                match crate::structural::Assessment::new(
                    crate::structural::Mode::PointEvaluation,
                    original.variables.clone(),
                    original.equations.clone(),
                    original.witness.clone(),
                ) {
                    Ok(assessment) => structure = Some(assessment),
                    Err(_) => refusal = Some(Refusal::Structure),
                }
            } else {
                evidence.push(EvidenceDemand::Structure);
            }
        }
        if structure.is_none() {
            // A refused or pending decision can still have one unambiguous original
            // assessment. Retain that witness without selecting a candidate or changing
            // readiness; distinct candidate interpretations remain on eligibility entries.
            let mut assessments = eligibility
                .iter()
                .filter_map(|entry| entry.structure.as_ref());
            if let Some(first) = assessments.next() {
                let original = first.row(self.context.snapshot.identity(), 0);
                if assessments.all(|assessment| {
                    assessment.row(self.context.snapshot.identity(), 0) == original
                }) {
                    structure = Some(first.clone());
                }
            }
        }
        let artifacts = chosen.map_or_else(Vec::new, |entry| entry.artifacts.clone());
        let state = if refusal.is_some() {
            AssessmentState::Refused
        } else if pending || !evidence.is_empty() {
            AssessmentState::PendingEvidence
        } else if !artifacts.is_empty() {
            AssessmentState::SupportedPendingArtifacts
        } else {
            AssessmentState::Ready
        };
        Decision {
            intent: self.intent,
            selection,
            classes: problem_classes(self.facts, self.intent, self.numerical_psd),
            selected,
            representation: selected.and_then(|route| match route {
                Route::Native(backend) => self
                    .table
                    .get(backend)
                    .map(|adapter| adapter.representation()),
                Route::Constant => None,
            }),
            lexicographic,
            structure,
            refusal,
            state,
            evidence,
            artifacts,
            snapshot: self.context.snapshot.identity(),
            pending_backend: pending.then_some(preferred).flatten(),
            eligibility,
        }
    }
    /// A capability query chooses native priorities first, otherwise admitted stages.
    /// An explicit backend is retained for every stage; no fallback is requested.
    pub fn lexicographic(&self, selection: SolverSelection, single_nonzero: bool) -> Decision {
        let mut eligibility = self.eligibility();
        for choice in &mut eligibility {
            if !single_nonzero
                && self.table.get(choice.backend).is_some_and(|a| {
                    a.capability().lexicographic_degradation == DegradationSupport::SingleNonzero
                })
            {
                choice.reasons.push(Ineligible::Lexicographic {
                    problem: problem_classes(self.facts, self.intent, self.numerical_psd),
                });
            }
            choice.finish();
        }
        let native = self.decide_assessed(selection, eligibility, Some(Lexicographic::Native));
        if native.selected.is_some() || native.state == AssessmentState::PendingEvidence {
            return native;
        }
        let mut facts = self.facts.clone();
        facts.objectives = 1;
        let staged = Requirements {
            table: self.table,
            facts: &facts,
            intent: self.intent,
            numerical_psd: self.numerical_psd,
            least_squares: self.least_squares,
            controls: self.controls,
            settings: self.settings,
            sensitivity: self.sensitivity,
            context: self.context.clone(),
        };
        let mut decision =
            staged.decide_assessed(selection, staged.eligibility(), Some(Lexicographic::Staged));
        decision.classes = problem_classes(self.facts, self.intent, self.numerical_psd);
        decision
    }
    /// Select from the retained decision consumed by diagnostics.
    pub fn select(&self, selection: SolverSelection) -> Result<Route, ProblemError> {
        self.decision(selection).candidate()
    }
    fn select_assessed(
        &self,
        selection: SolverSelection,
        choices: &[Eligibility],
    ) -> Result<Route, ProblemError> {
        self.controls.validate()?;
        if self.intent == SolveIntent::Optimize && !self.facts.objective {
            return Err(ProblemError::Contract(
                "optimization needs an authored objective".into(),
            ));
        }
        let admitted = |backend| {
            choices.iter().any(|c| {
                c.backend == backend
                    && matches!(
                        c.state,
                        AssessmentState::Ready | AssessmentState::SupportedPendingArtifacts
                    )
            })
        };
        if self.intent == SolveIntent::Certify && !choices.iter().any(|c| c.reasons.is_empty()) {
            return Err(ProblemError::Unsupported(
                "no linked backend certifies global bounds".into(),
            ));
        }
        if self.facts.variables == 0 {
            // Constant evaluation proves nothing global, so certification is not rerouted.
            if self.intent == SolveIntent::Certify {
                return Err(ProblemError::Unsupported(
                    "certification needs free variables; constant evaluation proves no bound"
                        .into(),
                ));
            }
            // Nor does it enforce a constraint left to a native handler (ADR-0104).
            if !self.facts.native.is_empty() {
                return Err(ProblemError::Unsupported(format!(
                    "native {} realization has no constant evaluation",
                    native_forms(&self.facts.native)
                )));
            }
            return Ok(Route::Constant);
        }
        let selected = match selection {
            SolverSelection::Explicit(b) => b,
            SolverSelection::Auto => {
                // Classes most specific first; the first class with an eligible automatic
                // owner decides, and its owners' preference orders them (ADR-0121). A
                // sensitivity request first looks only among the adapters whose candidate
                // the KKT-point analysis differentiates (ADR-0118).
                let pick = |sensitivities: bool| {
                    problem_classes(self.facts, self.intent, self.numerical_psd)
                        .into_iter()
                        .find_map(|class| {
                            self.table
                                .adapters()
                                .filter(|a| a.capability().automatic_classes.contains(&class))
                                .filter(|a| !sensitivities || a.capability().sensitivities)
                                .filter_map(|a| a.automatic().map(|rank| (rank, a.backend())))
                                .filter(|(_, b)| admitted(*b))
                                .min_by_key(|(rank, _)| *rank)
                                .map(|(_, b)| b)
                        })
                };
                self.sensitivity
                    .then(|| pick(true))
                    .flatten()
                    .or_else(|| pick(false))
                    .ok_or_else(|| {
                        ProblemError::Unsupported(format!(
                            "no eligible native route: {}",
                            assessed(choices)
                        ))
                    })?
            }
        };
        if !self.available(selected) {
            return Err(ProblemError::Unavailable {
                backend: selected,
                alternatives: choices
                    .iter()
                    .filter(|c| c.reasons.is_empty())
                    .map(|c| c.backend)
                    .collect(),
            });
        }
        if !admitted(selected) {
            return Err(ProblemError::Unsupported(format!(
                "selected {} is ineligible: {}",
                selected.as_str(),
                assessed(choices)
            )));
        }
        Ok(Route::Native(selected))
    }
    fn available(&self, backend: Backend) -> bool {
        self.table.get(backend).is_some_and(|a| a.linked())
    }
}
/// Diagnostic spelling of native constraint forms.
fn native_forms(forms: &[NativeConstraintForm]) -> String {
    forms
        .iter()
        .map(|form| form.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}
#[cfg(test)]
pub(crate) fn test_context(table: &Table) -> Context<'static> {
    static GUARDS: std::sync::LazyLock<
        BTreeMap<pse_ids::SemanticId, pse_math::presolve::GuardSign>,
    > = std::sync::LazyLock::new(BTreeMap::new);
    static REFUSALS: std::sync::LazyLock<BTreeMap<Backend, Arc<ProblemError>>> =
        std::sync::LazyLock::new(BTreeMap::new);
    Context {
        snapshot: crate::execution::Snapshot {
            adapters: table
                .adapters()
                .map(|adapter| {
                    (
                        adapter.backend(),
                        crate::execution::BuildObservation {
                            linked: adapter.linked(),
                            identity: None,
                        },
                    )
                })
                .collect(),
            ipopt: None,
            scip_omp_cancellation: false,
        },
        pending_classes: &[],
        refusals: &REFUSALS,
        structure: None,
        oracle: None,
        guards: &GUARDS,
        budgets: None,
        coefficients: None,
        certificate: None,
        cone: None,
        factorable: None,
        prepared: &[],
    }
}
#[cfg(test)]
impl Requirements<'_> {
    // These existing controls exercise the static class/rank rule only. Contextual
    // readiness is exercised separately with concrete structural/representation inputs.
    pub(crate) fn policy_eligibility_for_test(&self) -> Vec<Eligibility> {
        self.table
            .adapters()
            .filter(|adapter| crate::execution::algebraic(adapter.representation()))
            .map(|adapter| {
                let reasons = admit(
                    adapter.backend(),
                    adapter.capability(),
                    adapter.linked(),
                    self,
                );
                let state = if reasons.is_empty() {
                    AssessmentState::Ready
                } else {
                    AssessmentState::Refused
                };
                Eligibility {
                    backend: adapter.backend(),
                    reasons,
                    causes: vec![],
                    class_dependencies: vec![],
                    evidence: vec![],
                    artifacts: vec![],
                    factorable_refusals: vec![],
                    structure: None,
                    state,
                }
            })
            .collect()
    }
    pub(crate) fn policy_decision_for_test(&self, selection: SolverSelection) -> Decision {
        let choices = self.policy_eligibility_for_test();
        let mut decision = self.decide_assessed(selection, choices, None);
        if decision.selected == Some(Route::Constant) && decision.refusal.is_none() {
            decision.evidence.clear();
            decision.state = AssessmentState::Ready;
        }
        decision
    }
    pub(crate) fn policy_select_for_test(
        &self,
        selection: SolverSelection,
    ) -> Result<Route, ProblemError> {
        self.policy_decision_for_test(selection).candidate()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::{BackendSettings, LINKED, adapter};
    use pse_math::convexity::{
        ConeSummary, Convexity, ConvexityClass, Definiteness, GramCertificate, Unrecognized,
    };
    fn fact(class: ConvexityClass) -> Convexity {
        Convexity {
            key: pse_ids::ContentHash::from_bytes([3; 32]),
            class,
        }
    }
    /// The fact of a certified convex quadratic objective.
    fn convex_quadratic() -> Convexity {
        let q = faer::sparse::SparseColMat::try_new_from_triplets(
            2,
            2,
            &[
                faer::sparse::Triplet::new(0, 0, 2.0),
                faer::sparse::Triplet::new(1, 1, 2.0),
            ],
        )
        .unwrap();
        let Definiteness::Psd(c) =
            GramCertificate::certify(&q, 1.0, 100, &std::sync::atomic::AtomicBool::new(false))
                .unwrap()
        else {
            panic!("a positive diagonal is PSD")
        };
        fact(ConvexityClass::ConvexQuadratic(Arc::new(c)))
    }
    fn cone() -> Convexity {
        fact(ConvexityClass::Cone(ConeSummary {
            auxiliaries: 1,
            exponential: 1,
            ..ConeSummary::default()
        }))
    }
    #[test]
    fn caller_inventory_limits_automatic_selection() {
        let facts = root_facts();
        let requirements = Requirements {
            context: test_context(&Table::new(&[])),
            table: &Table::new(&[]),
            facts: &facts,
            intent: SolveIntent::Root,
            numerical_psd: false,
            least_squares: false,
            controls: &crate::solve::Controls::default(),
            settings: &BackendSettings::Default,
            sensitivity: false,
        };
        assert!(matches!(
            requirements.policy_select_for_test(SolverSelection::Auto),
            Err(ProblemError::RouteRefused(_))
        ));
        if adapter(Backend::Ipopt).linked() {
            static IPOPT_ONLY: Table = Table::new(&[adapter(Backend::Ipopt)]);
            let requirements = Requirements {
                table: &IPOPT_ONLY,
                ..requirements
            };
            assert_eq!(
                requirements
                    .policy_select_for_test(SolverSelection::Auto)
                    .unwrap(),
                Route::Native(Backend::Ipopt)
            );
        }
    }
    #[test]
    fn initialization_routes_its_actual_root_or_nlp_representation() {
        let mut facts = root_facts();
        assert!(root_intent(&facts, SolveIntent::Initialize));
        assert!(
            problem_classes(&facts, SolveIntent::Initialize, false)
                .contains(&ProblemClass::SquareRoot)
        );
        facts.objective = true;
        facts.objectives = 1;
        assert!(!root_intent(&facts, SolveIntent::Initialize));
        assert!(
            !problem_classes(&facts, SolveIntent::Initialize, false)
                .contains(&ProblemClass::SquareRoot)
        );
        assert!(
            problem_classes(&facts, SolveIntent::Initialize, false)
                .contains(&ProblemClass::SmoothNlp)
        );
        assert_eq!(
            crate::structural::mode(
                crate::structural::Policy::Equalities,
                &facts,
                SolveIntent::Initialize
            ),
            crate::structural::Mode::Nlp
        );
        assert_eq!(
            crate::structural::mode(
                crate::structural::Policy::Equalities,
                &facts,
                SolveIntent::Root
            ),
            crate::structural::Mode::Roots
        );
        facts.objective = false;
        facts.objectives = 0;
        facts.equalities = false;
        assert!(!root_intent(&facts, SolveIntent::Initialize));
        assert_eq!(
            crate::structural::mode(
                crate::structural::Policy::Equalities,
                &facts,
                SolveIntent::Initialize
            ),
            crate::structural::Mode::Nlp
        );
    }
    fn select(
        f: &ProblemFacts,
        intent: SolveIntent,
        selection: SolverSelection,
        numerical_psd: bool,
    ) -> Result<Route, ProblemError> {
        Requirements {
            context: test_context(&LINKED),
            table: &LINKED,
            facts: f,
            intent,
            numerical_psd,
            least_squares: false,
            controls: &crate::solve::Controls::default(),
            settings: &BackendSettings::Default,
            sensitivity: false,
        }
        .policy_select_for_test(selection)
    }
    #[test]
    fn conditional_unit_derivative_demand_comes_from_adapter_capability() {
        let mut controls = crate::solve::Controls::default();
        let root = adapter(Backend::Kinsol).capability();
        assert_eq!(
            derivative_demand(root, &controls),
            Some(DerivativeOrder::First)
        );
        let nlp = adapter(Backend::Ipopt).capability();
        controls.hessian = HessianMode::Exact;
        assert_eq!(
            derivative_demand(nlp, &controls),
            Some(DerivativeOrder::Second)
        );
        controls.hessian = HessianMode::LimitedMemory;
        assert_eq!(
            derivative_demand(nlp, &controls),
            Some(DerivativeOrder::First)
        );
    }
    fn root_facts() -> ProblemFacts {
        ProblemFacts {
            class_status: pse_math::presolve::ClassStatus::Established,
            variables: 1,
            rows: 1,
            objective: false,
            objectives: 0,
            equalities: true,
            domains: vec![ModelingVariableDomain::Continuous],
            derivatives: DerivativeOrder::Second,
            prepared_derivatives: DerivativeOrder::Second,
            bounds: vec![BoundShape::Free],
            guarded: false,
            coefficients: false,
            affine_rows: vec![false],
            objective_degree: Some(0),
            bound_assumptions: pse_ids::ContentHash::from_bytes([0; 32]),
            quadratic: false,
            native: vec![],
            requirements: vec![],
            convexity: Convexity::not_assessed(pse_ids::ContentHash::from_bytes([0; 32])),
        }
    }
    #[test]
    fn krylov_product_preparation_is_distinct_from_assembled_first_derivatives() {
        let mut facts = root_facts();
        facts.prepared_derivatives = DerivativeOrder::Value;
        let controls = crate::solve::Controls::default();
        let method = crate::settings::kinsol::Method {
            linear: crate::settings::kinsol::Linear::Spgmr {
                dimension: pse_model::scalars::PositiveCount::try_new(3).unwrap(),
            },
            ..Default::default()
        };
        let settings = BackendSettings::Kinsol(method);
        let mut context = test_context(&LINKED);
        let mut requirements = Requirements {
            table: &LINKED,
            facts: &facts,
            intent: SolveIntent::Root,
            numerical_psd: false,
            least_squares: false,
            controls: &controls,
            settings: &settings,
            sensitivity: false,
            context: context.clone(),
        };
        let adapter = adapter(Backend::Kinsol);
        let missing = assess_static(Backend::Kinsol, adapter.capability(), &requirements);
        assert!(missing.artifacts.contains(&ArtifactDemand::JacobianProduct));
        assert!(
            !missing
                .artifacts
                .contains(&ArtifactDemand::Derivatives(DerivativeOrder::First))
        );
        context.prepared = &[ArtifactDemand::JacobianProduct];
        requirements.context = context;
        assert!(
            assess_static(Backend::Kinsol, adapter.capability(), &requirements)
                .artifacts
                .is_empty()
        );
        let jacobi = BackendSettings::Kinsol(crate::settings::kinsol::Method {
            preconditioner: crate::solve::Preconditioner::Jacobi,
            ..method
        });
        requirements.settings = &jacobi;
        let assembled = assess_static(Backend::Kinsol, adapter.capability(), &requirements);
        assert!(
            assembled
                .artifacts
                .contains(&ArtifactDemand::Derivatives(DerivativeOrder::First))
        );
        // An assembled program is not a compiled directional action, including
        // when the Krylov setup also consumes an assembled preconditioner.
        let mut assembled_facts = facts.clone();
        assembled_facts.prepared_derivatives = DerivativeOrder::First;
        requirements.facts = &assembled_facts;
        requirements.context.prepared = &[ArtifactDemand::Derivatives(DerivativeOrder::First)];
        let krylov_settings = [
            crate::solve::Preconditioner::None,
            crate::solve::Preconditioner::Jacobi,
            crate::solve::Preconditioner::BlockFactor,
        ]
        .map(|preconditioner| {
            BackendSettings::Kinsol(crate::settings::kinsol::Method {
                preconditioner,
                ..method
            })
        });
        for settings in &krylov_settings {
            requirements.settings = settings;
            let missing = assess_static(Backend::Kinsol, adapter.capability(), &requirements);
            assert_eq!(missing.artifacts, vec![ArtifactDemand::JacobianProduct]);
            requirements.context.prepared = &[
                ArtifactDemand::Derivatives(DerivativeOrder::First),
                ArtifactDemand::JacobianProduct,
            ];
            assert!(
                assess_static(Backend::Kinsol, adapter.capability(), &requirements)
                    .artifacts
                    .is_empty()
            );
            requirements.context.prepared = &[ArtifactDemand::Derivatives(DerivativeOrder::First)];
        }
    }
    /// I8: a Gauss–Newton Hessian is admitted only for a least-squares objective; a
    /// steady solve is refused with the typed reason by every adapter, before native work,
    /// and a supplied Hessian of either kind requires second-order preparation.
    #[test]
    fn gauss_newton_requires_least_squares_objective() {
        let mut f = root_facts();
        f.objective = true;
        f.equalities = false;
        f.rows = 0;
        f.affine_rows.clear();
        f.prepared_derivatives = DerivativeOrder::First;
        let controls = crate::solve::Controls {
            hessian: HessianMode::GaussNewton,
            ..Default::default()
        };
        let requirements = |least_squares| Requirements {
            context: test_context(&LINKED),
            table: &LINKED,
            facts: &f,
            intent: SolveIntent::Optimize,
            numerical_psd: false,
            least_squares,
            controls: &controls,
            settings: &BackendSettings::Default,
            sensitivity: false,
        };
        let steady = requirements(false).policy_eligibility_for_test();
        assert!(!steady.is_empty());
        for e in &steady {
            assert!(e.reasons.contains(&Ineligible::LeastSquares), "{e}");
            assert!(e.to_string().contains("least-squares"), "{e}");
        }
        assert_eq!(
            Ineligible::LeastSquares.code(),
            NativeIneligibility::LeastSquares
        );
        assert!(
            requirements(false)
                .policy_select_for_test(SolverSelection::Auto)
                .is_err()
        );
        for e in requirements(true).policy_eligibility_for_test() {
            assert!(!e.reasons.contains(&Ineligible::LeastSquares), "{e}");
            // Available second-order mathematics is eligible before its executable
            // artifact is prepared; contextual controls cover that demand separately.
            if matches!(e.backend, Backend::Ipopt | Backend::Pounce) {
                assert!(
                    !e.reasons.contains(&Ineligible::Derivatives {
                        required: DerivativeOrder::Second
                    }),
                    "{e}"
                );
            }
        }
    }
    #[test]
    fn contextual_bounds_derivatives_and_threads_are_preparation_facts() {
        let mut f = root_facts();
        f.bounds[0] = BoundShape::Boxed;
        let mut controls = crate::solve::Controls::default();
        let assess = |f: &ProblemFacts, c: &crate::solve::Controls| {
            Requirements {
                context: test_context(&LINKED),
                table: &LINKED,
                facts: f,
                intent: SolveIntent::Root,
                numerical_psd: false,
                least_squares: false,
                controls: c,
                settings: &BackendSettings::Default,
                sensitivity: false,
            }
            .policy_eligibility_for_test()
        };
        let reasons = |q: &[Eligibility], backend| {
            q.iter()
                .find(|e| e.backend == backend)
                .unwrap()
                .reasons
                .clone()
        };
        let q = assess(&f, &controls);
        assert!(
            reasons(&q, Backend::Kinsol)
                .iter()
                .any(|r| matches!(r, Ineligible::Bounds { signs: true }))
        );
        assert!(
            reasons(&q, Backend::Ipopt)
                .iter()
                .all(|r| *r == Ineligible::NotLinked)
        );
        f.prepared_derivatives = DerivativeOrder::Value;
        // Available mathematics remains eligible; the unprepared executable order is
        // an artifact demand. Genuine unavailable derivatives still refuse below.
        let requirements = Requirements {
            context: test_context(&LINKED),
            table: &LINKED,
            facts: &f,
            intent: SolveIntent::Root,
            numerical_psd: false,
            least_squares: false,
            controls: &controls,
            settings: &BackendSettings::Default,
            sensitivity: false,
        };
        for backend in [Backend::Ipopt, Backend::Pounce, Backend::Kinsol] {
            let adapter = adapter(backend);
            let required = adapter.required_order(&requirements).unwrap();
            let candidate = assess_static(backend, adapter.capability(), &requirements);
            assert!(
                !candidate
                    .reasons
                    .iter()
                    .any(|reason| matches!(reason, Ineligible::Derivatives { .. })),
                "{candidate:?}"
            );
            assert!(
                candidate
                    .artifacts
                    .contains(&ArtifactDemand::Derivatives(required))
            );
        }
        f.derivatives = DerivativeOrder::Value;
        assert!(
            assess(&f, &controls)
                .iter()
                .filter(|e| matches!(
                    e.backend,
                    Backend::Ipopt | Backend::Pounce | Backend::Kinsol
                ))
                .all(|e| e
                    .reasons
                    .iter()
                    .any(|r| matches!(r, Ineligible::Derivatives { .. })))
        );
        f = root_facts();
        controls.threads = 2;
        assert!(reasons(&assess(&f, &controls), Backend::Kinsol).contains(&Ineligible::Serial));
        f.variables = 0;
        assert!(
            Requirements {
                context: test_context(&LINKED),
                table: &LINKED,
                facts: &f,
                intent: SolveIntent::Optimize,
                numerical_psd: false,
                least_squares: false,
                controls: &controls,
                settings: &BackendSettings::Default,
                sensitivity: false,
            }
            .policy_select_for_test(SolverSelection::Auto)
            .is_err()
        );
    }
    fn miqp_facts() -> ProblemFacts {
        ProblemFacts {
            class_status: pse_math::presolve::ClassStatus::Established,
            variables: 2,
            rows: 1,
            objective: true,
            objectives: 1,
            equalities: true,
            domains: vec![ModelingVariableDomain::Integer; 2],
            derivatives: DerivativeOrder::Second,
            prepared_derivatives: DerivativeOrder::Second,
            bounds: vec![BoundShape::Free; 2],
            guarded: false,
            coefficients: true,
            quadratic: true,
            affine_rows: vec![true],
            objective_degree: Some(2),
            bound_assumptions: pse_ids::ContentHash::from_bytes([0; 32]),
            native: vec![],
            requirements: vec![],
            convexity: Convexity::not_assessed(pse_ids::ContentHash::from_bytes([0; 32])),
        }
    }
    #[test]
    fn native_forms_are_admitted_only_by_a_record_that_consumes_them() {
        // An LP every coefficient adapter represents, except that it leaves an indicator
        // row to a native handler (ADR-0104).
        let mut f = miqp_facts();
        f.domains.fill(ModelingVariableDomain::Continuous);
        f.quadratic = false;
        f.objective_degree = Some(1);
        f.native = vec![NativeConstraintForm::Indicator];
        let requirements = Requirements {
            context: test_context(&LINKED),
            table: &LINKED,
            facts: &f,
            intent: SolveIntent::Optimize,
            numerical_psd: true,
            least_squares: false,
            controls: &crate::solve::Controls::default(),
            settings: &BackendSettings::Default,
            sensitivity: false,
        };
        let missing = Ineligible::NativeForms {
            missing: vec![NativeConstraintForm::Indicator],
        };
        // SCIP's record consumes the indicator handler (Plan 22 G7); every other record
        // is refused with the form it lacks, and automatic routing selects SCIP.
        for choice in requirements.policy_eligibility_for_test() {
            let consumes = adapter(choice.backend)
                .capability()
                .native_forms
                .contains(&NativeConstraintForm::Indicator);
            assert_eq!(consumes, choice.backend == Backend::Scip, "{choice:?}");
            assert_eq!(!consumes, choice.reasons.contains(&missing), "{choice:?}");
        }
        assert_eq!(
            requirements
                .policy_select_for_test(SolverSelection::Auto)
                .ok(),
            adapter(Backend::Scip)
                .linked()
                .then_some(Route::Native(Backend::Scip))
        );
        // The rule reads the record: one that consumes the handler admits the form.
        let highs = adapter(Backend::Highs).capability();
        let consuming = Capability {
            native_forms: &[NativeConstraintForm::Indicator],
            ..*highs
        };
        assert!(admit(Backend::Highs, highs, true, &requirements).contains(&missing));
        assert!(admit(Backend::Highs, &consuming, true, &requirements).is_empty());
        // Without free variables the constant route cannot enforce the form either.
        f.variables = 0;
        assert!(matches!(
            select(&f, SolveIntent::Optimize, SolverSelection::Auto, true),
            Err(ProblemError::RouteRefused(_))
        ));
        f.native.clear();
        assert_eq!(
            select(&f, SolveIntent::Optimize, SolverSelection::Auto, true).unwrap(),
            Route::Constant
        );
    }
    /// Several objectives are optimized in one native solve only by a record that states
    /// the class lexicographic (ADR-0111 item 4): HiGHS for LP and MILP; every other class
    /// and adapter is refused, and a staged sequence optimizes the levels instead.
    #[test]
    fn several_objectives_route_only_to_a_lexicographic_record() {
        let mut f = miqp_facts();
        f.quadratic = false;
        f.objective_degree = Some(1);
        f.objectives = 2;
        let controls = crate::solve::Controls::default();
        let route = |f: &ProblemFacts| {
            let requirements = Requirements {
                context: test_context(&LINKED),
                table: &LINKED,
                facts: f,
                intent: SolveIntent::Optimize,
                numerical_psd: false,
                least_squares: false,
                controls: &controls,
                settings: &BackendSettings::Default,
                sensitivity: false,
            };
            (
                requirements.policy_eligibility_for_test(),
                requirements
                    .policy_select_for_test(SolverSelection::Auto)
                    .ok(),
            )
        };
        let lexicographic = |choice: &Eligibility| {
            choice
                .reasons
                .iter()
                .any(|r| matches!(r, Ineligible::Lexicographic { .. }))
        };
        let (choices, selected) = route(&f);
        for choice in &choices {
            assert_eq!(
                lexicographic(choice),
                choice.backend != Backend::Highs,
                "{choice:?}"
            );
        }
        assert_eq!(
            selected,
            adapter(Backend::Highs)
                .linked()
                .then_some(Route::Native(Backend::Highs))
        );
        // A quadratic level is no lexicographic class of any record.
        f.quadratic = true;
        f.domains.fill(ModelingVariableDomain::Continuous);
        let (choices, selected) = route(&f);
        assert!(choices.iter().all(lexicographic));
        assert!(selected.is_none());
        assert_eq!(
            Ineligible::Lexicographic { problem: vec![] }.code(),
            NativeIneligibility::Lexicographic
        );
    }
    #[test]
    fn class_refusals_do_not_relax_discrete_or_square_requirements() {
        let f = miqp_facts();
        // Only a mixed-integer quadratic adapter represents the class.
        assert_eq!(
            select(&f, SolveIntent::Optimize, SolverSelection::Auto, true).is_ok(),
            adapter(Backend::Scip).linked()
        );
        assert!(select(&f, SolveIntent::Root, SolverSelection::Auto, true).is_err());
        for local in [Backend::Ipopt, Backend::Pounce, Backend::Highs] {
            assert!(
                select(
                    &f,
                    SolveIntent::Optimize,
                    SolverSelection::Explicit(local),
                    true
                )
                .is_err()
            );
        }
        let mut f = f;
        // A coefficient MILP retains its authored discrete domains and cannot use a
        // continuous cone adapter.
        f.quadratic = false;
        f.objective_degree = Some(1);
        assert!(
            select(
                &f,
                SolveIntent::Optimize,
                SolverSelection::Explicit(Backend::Clarabel),
                true,
            )
            .is_err()
        );
        f.domains.fill(ModelingVariableDomain::Continuous);
        f.coefficients = false;
        // The proved affine rows and degree-one objective still establish a linear
        // class without a materialized coefficient snapshot.
        assert_eq!(
            select(
                &f,
                SolveIntent::Optimize,
                SolverSelection::Explicit(Backend::Clarabel),
                true,
            )
            .unwrap(),
            Route::Native(Backend::Clarabel)
        );
        // Actual nonlinear evidence excludes coefficient admission; without recognized
        // cone evidence this model must still refuse the explicit cone adapter.
        f.class_status = pse_math::presolve::ClassStatus::RuledOut(
            pse_math::presolve::ClassWitness::NonAffineRow {
                row: pse_ids::SemanticId::from_bytes([9; 16]),
            },
        );
        f.affine_rows.fill(false);
        f.objective_degree = None;
        assert_eq!(
            problem_classes(&f, SolveIntent::Optimize, true),
            [ProblemClass::SmoothNlp]
        );
        assert!(
            select(
                &f,
                SolveIntent::Optimize,
                SolverSelection::Explicit(Backend::Clarabel),
                true,
            )
            .is_err()
        );
        f.variables = 0;
        assert_eq!(
            select(&f, SolveIntent::Optimize, SolverSelection::Auto, false).unwrap(),
            Route::Constant
        );
    }
    #[test]
    fn problem_classes_follow_facts_and_intent() {
        let mut f = root_facts();
        assert_eq!(
            problem_classes(&f, SolveIntent::Root, false),
            [ProblemClass::SquareRoot, ProblemClass::SmoothNlp]
        );
        assert_eq!(
            problem_classes(&f, SolveIntent::FeasiblePoint, false),
            [ProblemClass::SmoothNlp]
        );
        // Most specific first: the coefficient class, then smooth NLP.
        f.coefficients = true;
        f.objective = true;
        assert_eq!(
            problem_classes(&f, SolveIntent::Optimize, false),
            [ProblemClass::Linear, ProblemClass::SmoothNlp]
        );
        // Degree two over affine rows: nonconvex without the exact fact, convex with it,
        // and convex for one request under its explicit numerical qualification.
        f.quadratic = true;
        assert_eq!(
            problem_classes(&f, SolveIntent::Optimize, false),
            [ProblemClass::NonconvexQuadratic, ProblemClass::SmoothNlp]
        );
        assert_eq!(
            problem_classes(&f, SolveIntent::Optimize, true),
            [ProblemClass::ConvexQuadratic, ProblemClass::SmoothNlp]
        );
        f.convexity = convex_quadratic();
        assert_eq!(
            problem_classes(&f, SolveIntent::Optimize, false),
            [ProblemClass::ConvexQuadratic, ProblemClass::SmoothNlp]
        );
        f.domains = vec![ModelingVariableDomain::Binary];
        for numerical_psd in [false, true] {
            assert_eq!(
                problem_classes(&f, SolveIntent::Optimize, numerical_psd),
                [ProblemClass::MixedIntegerQuadratic]
            );
        }
        f.quadratic = false;
        assert_eq!(
            problem_classes(&f, SolveIntent::Optimize, false),
            [ProblemClass::MixedLinear]
        );
        // Nonlinear rows or objective with a discrete column.
        f.coefficients = false;
        assert_eq!(
            problem_classes(&f, SolveIntent::Optimize, false),
            [ProblemClass::MixedIntegerNonlinear]
        );
        // Discrete classes are optimization classes only.
        assert!(problem_classes(&f, SolveIntent::Root, false).is_empty());
        // A recognized continuous nonlinear program is a cone program first (ADR-0121); a
        // recognized program with a discrete column is not.
        f.convexity = cone();
        assert_eq!(
            problem_classes(&f, SolveIntent::Optimize, false),
            [ProblemClass::MixedIntegerNonlinear]
        );
        f.domains = vec![ModelingVariableDomain::Continuous];
        assert_eq!(
            problem_classes(&f, SolveIntent::Optimize, false),
            [ProblemClass::ContinuousCone, ProblemClass::SmoothNlp]
        );
        assert_eq!(
            problem_classes(&f, SolveIntent::Root, false),
            [ProblemClass::SmoothNlp]
        );
    }
    /// Clarabel represents linear and convex quadratic programs but owns only continuous
    /// cones automatically (ADR-0121): explicit selection admits it, automatic routing never
    /// picks it for those classes, even when no other adapter is exposed. Every record's
    /// automatic classes are among its classes.
    #[test]
    fn explicit_only_classes_never_automatic() {
        let mut f = miqp_facts();
        f.domains.fill(ModelingVariableDomain::Continuous);
        static CLARABEL_ONLY: Table = Table::new(&[adapter(Backend::Clarabel)]);
        for quadratic in [false, true] {
            f.quadratic = quadratic;
            f.objective_degree = Some(if quadratic { 2 } else { 1 });
            f.convexity = if quadratic {
                convex_quadratic()
            } else {
                fact(ConvexityClass::Affine)
            };
            let requirements = Requirements {
                context: test_context(&CLARABEL_ONLY),
                table: &CLARABEL_ONLY,
                facts: &f,
                intent: SolveIntent::Optimize,
                numerical_psd: false,
                least_squares: false,
                controls: &crate::solve::Controls::default(),
                settings: &BackendSettings::Default,
                sensitivity: false,
            };
            assert_eq!(
                requirements
                    .policy_select_for_test(SolverSelection::Explicit(Backend::Clarabel))
                    .unwrap(),
                Route::Native(Backend::Clarabel)
            );
            assert!(matches!(
                requirements.policy_select_for_test(SolverSelection::Auto),
                Err(ProblemError::RouteRefused(_))
            ));
            // In the linked table, HiGHS keeps both classes automatically.
            if adapter(Backend::Highs).linked() {
                assert_eq!(
                    select(&f, SolveIntent::Optimize, SolverSelection::Auto, false).unwrap(),
                    Route::Native(Backend::Highs)
                );
            }
        }
        for a in LINKED.adapters() {
            let record = a.capability();
            assert!(
                record
                    .automatic_classes
                    .iter()
                    .all(|c| record.classes.contains(c)),
                "{:?}",
                a.backend()
            );
            assert_eq!(
                record.automatic_classes.is_empty(),
                a.automatic().is_none(),
                "{:?}",
                a.backend()
            );
        }
        let clarabel = adapter(Backend::Clarabel).capability();
        for class in clarabel.classes {
            assert_eq!(
                clarabel.automatic_classes.contains(class),
                *class == ProblemClass::ContinuousCone,
                "{class:?}"
            );
        }
    }
    /// POUNCE-convex (Plan 22 N5) is explicit only: automatic routing never selects it for
    /// a linear, convex quadratic or cone program, and an explicit selection is eligible
    /// wherever the adapter is linked.
    #[test]
    fn pounce_convex_never_automatic() {
        let record = adapter(Backend::PounceConvex).capability();
        assert!(record.automatic_classes.is_empty());
        assert!(adapter(Backend::PounceConvex).automatic().is_none());
        assert!(record.batch && !record.sensitivities);
        let mut f = miqp_facts();
        f.domains.fill(ModelingVariableDomain::Continuous);
        for (quadratic, convexity) in [
            (false, fact(ConvexityClass::Affine)),
            (true, convex_quadratic()),
        ] {
            f.quadratic = quadratic;
            f.objective_degree = Some(if quadratic { 2 } else { 1 });
            f.convexity = convexity;
            let automatic = select(&f, SolveIntent::Optimize, SolverSelection::Auto, false);
            assert_ne!(automatic.ok(), Some(Route::Native(Backend::PounceConvex)));
            let explicit = select(
                &f,
                SolveIntent::Optimize,
                SolverSelection::Explicit(Backend::PounceConvex),
                false,
            );
            if adapter(Backend::PounceConvex).linked() {
                assert_eq!(explicit.unwrap(), Route::Native(Backend::PounceConvex));
            } else {
                assert!(explicit.is_err());
            }
        }
    }
    /// Automatic routing of a parametric sensitivity request (ADR-0118) prefers an adapter
    /// whose candidate carries the multipliers the KKT-point analysis differentiates: a
    /// convex quadratic program with one routes past HiGHS to an NLP adapter. Eligibility
    /// is unchanged, so an explicit coefficient adapter still solves it and the quantities
    /// are withheld with their reason.
    #[test]
    fn sensitivity_requests_route_to_multiplier_adapters() {
        if !adapter(Backend::Ipopt).linked() && !adapter(Backend::Pounce).linked() {
            return;
        }
        let mut f = miqp_facts();
        f.domains.fill(ModelingVariableDomain::Continuous);
        f.quadratic = true;
        f.objective_degree = Some(2);
        f.convexity = convex_quadratic();
        let controls = crate::solve::Controls::default();
        let requirements = Requirements {
            context: test_context(&LINKED),
            table: &LINKED,
            facts: &f,
            intent: SolveIntent::Optimize,
            numerical_psd: false,
            least_squares: false,
            controls: &controls,
            settings: &BackendSettings::Default,
            sensitivity: true,
        };
        let plain = Requirements {
            context: requirements.context.clone(),
            sensitivity: false,
            ..requirements
        };
        if adapter(Backend::Highs).linked() {
            assert_eq!(
                plain.policy_select_for_test(SolverSelection::Auto).unwrap(),
                Route::Native(Backend::Highs)
            );
            assert_eq!(
                requirements
                    .policy_select_for_test(SolverSelection::Explicit(Backend::Highs))
                    .unwrap(),
                Route::Native(Backend::Highs)
            );
        }
        let Route::Native(backend) = requirements
            .policy_select_for_test(SolverSelection::Auto)
            .unwrap()
        else {
            panic!("a native route");
        };
        assert!(adapter(backend).capability().sensitivities, "{backend:?}");
        assert_eq!(
            adapter(backend).representation(),
            crate::execution::Representation::Nlp
        );
    }
    /// ADR-0121 negative control: a continuous nonlinear program the curvature pass did not
    /// recognize (or could not decide) has no cone class, so automatic routing never
    /// reaches Clarabel and an explicit Clarabel selection is refused with the class
    /// reason; a request's numerical qualification never adds a cone class. The same facts
    /// with a recognized cone route to Clarabel automatically.
    #[test]
    fn unrecognized_problem_not_routed() {
        let mut f = root_facts();
        f.objective = true;
        f.equalities = false;
        for class in [
            ConvexityClass::Unrecognized(Unrecognized::Curvature { row: Some(0) }),
            ConvexityClass::Unrecognized(Unrecognized::Inexact),
            ConvexityClass::Unrecognized(Unrecognized::Domain),
            ConvexityClass::Inconclusive,
        ] {
            f.convexity = fact(class);
            for numerical_psd in [false, true] {
                assert_eq!(
                    problem_classes(&f, SolveIntent::Optimize, numerical_psd),
                    [ProblemClass::SmoothNlp]
                );
            }
            let explicit = select(
                &f,
                SolveIntent::Optimize,
                SolverSelection::Explicit(Backend::Clarabel),
                false,
            );
            assert!(
                matches!(&explicit, Err(ProblemError::RouteRefused(d)) if d.eligibility.iter().any(|e| e.backend == Backend::Clarabel && e.reasons.iter().any(|r| matches!(r, Ineligible::Class {..})))),
                "{explicit:?}"
            );
            if let Ok(route) = select(&f, SolveIntent::Optimize, SolverSelection::Auto, false) {
                assert_ne!(route, Route::Native(Backend::Clarabel));
            }
        }
        f.convexity = cone();
        assert_eq!(
            select(&f, SolveIntent::Optimize, SolverSelection::Auto, false).unwrap(),
            Route::Native(Backend::Clarabel)
        );
    }
    /// ADR-0104 §5 and ADR-0109: a structure whose authored `penalty(l1)` realization states
    /// the l1 exact penalty admits only an adapter whose record honours it, run with a method
    /// that does. Every other adapter, and POUNCE with another method, is ineligible with the
    /// typed `method` reason; automatic routing reaches POUNCE because the author selected
    /// the realization, and its native defaults take the l1 method. Without the requirement
    /// nothing selects the l1 method.
    #[test]
    fn authored_l1_requirement_is_routing_fact() {
        use crate::settings::pounce::{Method, Settings};
        let mut f = root_facts();
        f.equalities = false;
        f.requirements = vec![ModelingStructuralRequirement::L1ExactPenalty];
        let requirement = Ineligible::Method {
            requirements: vec![ModelingStructuralRequirement::L1ExactPenalty],
        };
        assert_eq!(requirement.code(), NativeIneligibility::Method);
        assert!(
            requirement.to_string().contains("l1_exact_penalty"),
            "{requirement}"
        );
        let interior = BackendSettings::Pounce(Settings::default());
        let l1 = BackendSettings::Pounce(Settings {
            method: Method::L1ExactPenalty,
            ..Settings::default()
        });
        let controls = crate::solve::Controls::default();
        let requirements = |settings| Requirements {
            context: test_context(&LINKED),
            table: &LINKED,
            facts: &f,
            intent: SolveIntent::FeasiblePoint,
            numerical_psd: false,
            least_squares: false,
            controls: &controls,
            settings,
            sensitivity: false,
        };
        for settings in [&BackendSettings::Default, &interior, &l1] {
            for e in requirements(settings).policy_eligibility_for_test() {
                let honours = e.backend == Backend::Pounce && !std::ptr::eq(settings, &interior);
                assert_eq!(!e.reasons.contains(&requirement), honours, "{e}");
            }
        }
        if adapter(Backend::Pounce).linked() {
            for settings in [&BackendSettings::Default, &l1] {
                assert_eq!(
                    requirements(settings)
                        .policy_select_for_test(SolverSelection::Auto)
                        .unwrap(),
                    Route::Native(Backend::Pounce)
                );
            }
            assert!(
                requirements(&interior)
                    .policy_select_for_test(SolverSelection::Explicit(Backend::Pounce))
                    .is_err()
            );
        }
        assert!(
            requirements(&BackendSettings::Default)
                .policy_select_for_test(SolverSelection::Explicit(Backend::Ipopt))
                .is_err()
        );
        // Native defaults take the author's method on POUNCE only, and only under the
        // requirement; explicit settings are unchanged.
        let effective = BackendSettings::Default.for_requirements(Backend::Pounce, &f.requirements);
        assert!(matches!(
            effective,
            BackendSettings::Pounce(Settings {
                method: Method::L1ExactPenalty,
                ..
            })
        ));
        assert!(matches!(
            BackendSettings::Default.for_requirements(Backend::Pounce, &[]),
            BackendSettings::Default
        ));
        assert!(matches!(
            BackendSettings::Default.for_requirements(Backend::Ipopt, &f.requirements),
            BackendSettings::Default
        ));
        // Without the requirement automatic routing is unchanged and never takes l1.
        let mut plain = f.clone();
        plain.requirements.clear();
        let unrequired = Requirements {
            facts: &plain,
            ..requirements(&BackendSettings::Default)
        };
        for e in unrequired.policy_eligibility_for_test() {
            assert!(
                !e.reasons
                    .iter()
                    .any(|r| matches!(r, Ineligible::Method { .. })),
                "{e}"
            );
        }
    }
    #[test]
    fn miqp_routes_to_scip() {
        let f = miqp_facts();
        let assess = |numerical_psd| {
            Requirements {
                context: test_context(&LINKED),
                table: &LINKED,
                facts: &f,
                intent: SolveIntent::Optimize,
                numerical_psd,
                least_squares: false,
                controls: &crate::solve::Controls::default(),
                settings: &BackendSettings::Default,
                sensitivity: false,
            }
            .policy_eligibility_for_test()
        };
        for numerical_psd in [false, true] {
            // Every admitted adapter represents the mixed-integer quadratic class.
            for e in assess(numerical_psd) {
                if e.reasons.is_empty() {
                    assert_eq!(e.backend, Backend::Scip);
                }
            }
            let route = select(
                &f,
                SolveIntent::Optimize,
                SolverSelection::Auto,
                numerical_psd,
            );
            if adapter(Backend::Scip).linked() {
                assert_eq!(route.unwrap(), Route::Native(Backend::Scip));
            } else {
                assert!(route.is_err());
            }
        }
        // A mixed-integer nonlinear program routes the same way.
        let mut f = f;
        f.coefficients = false;
        f.quadratic = false;
        let route = select(&f, SolveIntent::Optimize, SolverSelection::Auto, false);
        assert_eq!(route.is_ok(), adapter(Backend::Scip).linked());
        // A continuous nonconvex quadratic program stays local automatically.
        f.domains.fill(ModelingVariableDomain::Continuous);
        f.coefficients = true;
        f.quadratic = true;
        if adapter(Backend::Ipopt).linked() || adapter(Backend::Pounce).linked() {
            let route = select(&f, SolveIntent::Optimize, SolverSelection::Auto, false).unwrap();
            assert_ne!(route, Route::Native(Backend::Scip));
        }
        // A linear program keeps HiGHS automatically and admits SCIP explicitly.
        f.quadratic = false;
        if adapter(Backend::Highs).linked() {
            assert_eq!(
                select(&f, SolveIntent::Optimize, SolverSelection::Auto, false).unwrap(),
                Route::Native(Backend::Highs)
            );
        }
        if adapter(Backend::Scip).linked() {
            assert_eq!(
                select(
                    &f,
                    SolveIntent::Optimize,
                    SolverSelection::Explicit(Backend::Scip),
                    false
                )
                .unwrap(),
                Route::Native(Backend::Scip)
            );
        }
    }
    #[test]
    fn certify_routes_to_scip_when_linked() {
        let mut f = root_facts();
        f.objective = true;
        f.equalities = false;
        let linked = adapter(Backend::Scip).linked();
        // Local adapters never certify, even when selected explicitly.
        for local in [Backend::Ipopt, Backend::Pounce] {
            let error = select(
                &f,
                SolveIntent::Certify,
                SolverSelection::Explicit(local),
                false,
            )
            .unwrap_err();
            assert!(
                matches!(error, ProblemError::RouteRefused(ref decision) if decision.selected.is_none() && decision.refusal.is_some()),
                "{error:?}"
            );
        }
        let route = select(&f, SolveIntent::Certify, SolverSelection::Auto, false);
        if linked {
            assert_eq!(route.unwrap(), Route::Native(Backend::Scip));
        } else {
            assert!(
                matches!(&route, Err(ProblemError::RouteRefused(d)) if matches!(d.refusal, Some(Refusal::NoEligible))),
                "{route:?}"
            );
        }
        // A nonconvex quadratic program certifies through the same record.
        f.coefficients = true;
        f.quadratic = true;
        assert_eq!(
            select(&f, SolveIntent::Certify, SolverSelection::Auto, false).is_ok(),
            linked
        );
        // Even an all-fixed model is not silently certified by constant evaluation.
        f.variables = 0;
        assert!(matches!(
            select(&f, SolveIntent::Certify, SolverSelection::Auto, false),
            Err(ProblemError::RouteRefused(_))
        ));
        let requirements = Requirements {
            context: test_context(&LINKED),
            table: &LINKED,
            facts: &root_facts(),
            intent: SolveIntent::Certify,
            numerical_psd: false,
            least_squares: false,
            controls: &crate::solve::Controls::default(),
            settings: &BackendSettings::Default,
            sensitivity: false,
        };
        // Only a certifying record is eligible; the rule reads the record, not the backend.
        for e in requirements.policy_eligibility_for_test() {
            let certifies = LINKED
                .get(e.backend)
                .is_some_and(|a| a.capability().certifies);
            assert_eq!(
                e.reasons.contains(&Ineligible::Certification),
                !certifies,
                "{e:?}"
            );
        }
        // Certification is a typed intent with a registry name, parsed like every other.
        assert_eq!(
            "certify".parse::<SolveIntent>().unwrap(),
            SolveIntent::Certify
        );
        assert_eq!(SolveIntent::Certify.as_str(), "certify");
    }
}

#[cfg(test)]
mod contextual_tests;
