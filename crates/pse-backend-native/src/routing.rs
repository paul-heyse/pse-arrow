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
    /// The prepared smooth derivative order is below the adapter's requirement.
    Derivatives {
        /// Required prepared order.
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
    /// Several objectives, in none of the classes the adapter optimizes lexicographically
    /// in one native solve (ADR-0111 item 4); a staged sequence optimizes them one level
    /// at a time instead.
    Lexicographic {
        /// Classes the facts and intent establish for this problem.
        problem: Vec<ProblemClass>,
    },
}
impl std::fmt::Display for Ineligible {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
                "requires prepared smooth {} derivatives",
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
}
/// Project an already admitted native oracle into the same contextual selector.
/// Callers supply the represented objective/equality meaning, not a backend preference.
pub fn oracle_facts(c: &crate::OracleContract, objective: bool, equalities: bool) -> ProblemFacts {
    ProblemFacts {
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
fn continuous(f: &ProblemFacts) -> bool {
    f.domains
        .iter()
        .all(|d| *d == ModelingVariableDomain::Continuous)
}
fn square_root(f: &ProblemFacts) -> bool {
    f.equalities && f.rows == f.variables && !f.objective && continuous(f)
}
const fn root_intent(intent: SolveIntent) -> bool {
    matches!(intent, SolveIntent::Root | SolveIntent::Initialize)
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
    if root_intent(intent) && square_root(f) {
        classes.push(ProblemClass::SquareRoot);
    }
    if !root_intent(intent) {
        let convex = f.convexity.convex_quadratic().is_some() || numerical_psd;
        match (f.coefficients, f.quadratic, continuous(f)) {
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
    if root_intent(r.intent) && !square_root(f) {
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
    if !classes.iter().any(|c| capability.classes.contains(c)) {
        reasons.push(Ineligible::Class { problem: classes });
    }
    if r.controls.hessian == HessianMode::GaussNewton && !r.least_squares {
        reasons.push(Ineligible::LeastSquares);
    }
    // Every mode other than the library's quasi-Newton approximation is a supplied
    // Hessian: exact, or the Gauss–Newton Gram with constraint curvature.
    let required = match capability.derivatives {
        DerivativeCapability::ExactHessianOrLimitedMemory
            if r.controls.hessian != HessianMode::LimitedMemory =>
        {
            Some(DerivativeOrder::Second)
        }
        DerivativeCapability::ExactHessianOrLimitedMemory
        | DerivativeCapability::JacobianOrProduct
        | DerivativeCapability::ForwardAndAdjointSensitivities
        | DerivativeCapability::SecondOrderAdjointSensitivities => Some(DerivativeOrder::First),
        DerivativeCapability::Coefficients | DerivativeCapability::Factorable => None,
    };
    if let Some(required) = required
        && f.derivatives.min(f.prepared_derivatives) < required
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
impl Requirements<'_> {
    /// Assess every exposed algebraic adapter through the one eligibility rule.
    pub fn eligibility(&self) -> Vec<Eligibility> {
        self.table
            .adapters()
            .filter(|a| a.representation().algebraic())
            .map(|a| Eligibility {
                backend: a.backend(),
                reasons: a.admit(self),
            })
            .collect()
    }
    /// Deterministic route; explicit selection never silently falls back.
    pub fn select(&self, selection: SolverSelection) -> Result<Route, ProblemError> {
        self.controls.validate()?;
        if self.intent == SolveIntent::Optimize && !self.facts.objective {
            return Err(ProblemError::Contract(
                "optimization needs an authored objective".into(),
            ));
        }
        let choices = self.eligibility();
        let admitted = |backend| {
            choices
                .iter()
                .any(|c| c.backend == backend && c.reasons.is_empty())
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
                            assessed(&choices)
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
                assessed(&choices)
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
        fact(ConvexityClass::ConvexQuadratic(std::sync::Arc::new(c)))
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
            requirements.select(SolverSelection::Auto),
            Err(ProblemError::Unsupported(_))
        ));
        if adapter(Backend::Ipopt).linked() {
            static IPOPT_ONLY: Table = Table::new(&[adapter(Backend::Ipopt)]);
            let requirements = Requirements {
                table: &IPOPT_ONLY,
                ..requirements
            };
            assert_eq!(
                requirements.select(SolverSelection::Auto).unwrap(),
                Route::Native(Backend::Ipopt)
            );
        }
    }
    fn select(
        f: &ProblemFacts,
        intent: SolveIntent,
        selection: SolverSelection,
        numerical_psd: bool,
    ) -> Result<Route, ProblemError> {
        Requirements {
            table: &LINKED,
            facts: f,
            intent,
            numerical_psd,
            least_squares: false,
            controls: &crate::solve::Controls::default(),
            settings: &BackendSettings::Default,
            sensitivity: false,
        }
        .select(selection)
    }
    fn root_facts() -> ProblemFacts {
        ProblemFacts {
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
            table: &LINKED,
            facts: &f,
            intent: SolveIntent::Optimize,
            numerical_psd: false,
            least_squares,
            controls: &controls,
            settings: &BackendSettings::Default,
            sensitivity: false,
        };
        let steady = requirements(false).eligibility();
        assert!(!steady.is_empty());
        for e in &steady {
            assert!(e.reasons.contains(&Ineligible::LeastSquares), "{e}");
            assert!(e.to_string().contains("least-squares"), "{e}");
        }
        assert_eq!(
            Ineligible::LeastSquares.code(),
            NativeIneligibility::LeastSquares
        );
        assert!(requirements(false).select(SolverSelection::Auto).is_err());
        for e in requirements(true).eligibility() {
            assert!(!e.reasons.contains(&Ineligible::LeastSquares), "{e}");
            // A supplied Gauss–Newton Hessian needs second-order derivatives from the
            // Hessian-consuming NLP adapters, like the exact one.
            if matches!(e.backend, Backend::Ipopt | Backend::Pounce) {
                assert!(
                    e.reasons.contains(&Ineligible::Derivatives {
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
                table: &LINKED,
                facts: f,
                intent: SolveIntent::Root,
                numerical_psd: false,
                least_squares: false,
                controls: c,
                settings: &BackendSettings::Default,
                sensitivity: false,
            }
            .eligibility()
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
                table: &LINKED,
                facts: &f,
                intent: SolveIntent::Optimize,
                numerical_psd: false,
                least_squares: false,
                controls: &controls,
                settings: &BackendSettings::Default,
                sensitivity: false,
            }
            .select(SolverSelection::Auto)
            .is_err()
        );
    }
    fn miqp_facts() -> ProblemFacts {
        ProblemFacts {
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
        for choice in requirements.eligibility() {
            let consumes = adapter(choice.backend)
                .capability()
                .native_forms
                .contains(&NativeConstraintForm::Indicator);
            assert_eq!(consumes, choice.backend == Backend::Scip, "{choice:?}");
            assert_eq!(!consumes, choice.reasons.contains(&missing), "{choice:?}");
        }
        assert_eq!(
            requirements.select(SolverSelection::Auto).ok(),
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
            Err(ProblemError::Unsupported(_))
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
                requirements.eligibility(),
                requirements.select(SolverSelection::Auto).ok(),
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
        // A coefficient MILP is not conic data, and neither is an opaque nonlinear model.
        f.quadratic = false;
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
                    .select(SolverSelection::Explicit(Backend::Clarabel))
                    .unwrap(),
                Route::Native(Backend::Clarabel)
            );
            assert!(matches!(
                requirements.select(SolverSelection::Auto),
                Err(ProblemError::Unsupported(_))
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
            sensitivity: false,
            ..requirements
        };
        if adapter(Backend::Highs).linked() {
            assert_eq!(
                plain.select(SolverSelection::Auto).unwrap(),
                Route::Native(Backend::Highs)
            );
            assert_eq!(
                requirements
                    .select(SolverSelection::Explicit(Backend::Highs))
                    .unwrap(),
                Route::Native(Backend::Highs)
            );
        }
        let Route::Native(backend) = requirements.select(SolverSelection::Auto).unwrap() else {
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
                matches!(&explicit, Err(ProblemError::Unsupported(m)) if m.contains("problem class")),
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
            for e in requirements(settings).eligibility() {
                let honours = e.backend == Backend::Pounce && !std::ptr::eq(settings, &interior);
                assert_eq!(!e.reasons.contains(&requirement), honours, "{e}");
            }
        }
        if adapter(Backend::Pounce).linked() {
            for settings in [&BackendSettings::Default, &l1] {
                assert_eq!(
                    requirements(settings)
                        .select(SolverSelection::Auto)
                        .unwrap(),
                    Route::Native(Backend::Pounce)
                );
            }
            assert!(
                requirements(&interior)
                    .select(SolverSelection::Explicit(Backend::Pounce))
                    .is_err()
            );
        }
        assert!(
            requirements(&BackendSettings::Default)
                .select(SolverSelection::Explicit(Backend::Ipopt))
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
        for e in unrequired.eligibility() {
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
                table: &LINKED,
                facts: &f,
                intent: SolveIntent::Optimize,
                numerical_psd,
                least_squares: false,
                controls: &crate::solve::Controls::default(),
                settings: &BackendSettings::Default,
                sensitivity: false,
            }
            .eligibility()
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
            assert!(matches!(error, ProblemError::Unsupported(_)), "{error:?}");
        }
        let route = select(&f, SolveIntent::Certify, SolverSelection::Auto, false);
        if linked {
            assert_eq!(route.unwrap(), Route::Native(Backend::Scip));
        } else {
            assert!(
                matches!(&route, Err(ProblemError::Unsupported(m)) if m == "no linked backend certifies global bounds"),
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
            Err(ProblemError::Unsupported(_))
        ));
        let requirements = Requirements {
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
        for e in requirements.eligibility() {
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
