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
use pse_model::generated::enums::{ModelingVariableDomain, NativeConstraintForm, NativeIneligibility};
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
    /// Convexity qualified against this coefficient snapshot.
    pub convex: bool,
    /// Complete effective native controls.
    pub controls: &'a crate::solve::Controls,
}
/// Project an already admitted native oracle into the same contextual selector.
/// Callers supply the represented objective/equality meaning, not a backend preference.
pub fn oracle_facts(c: &crate::OracleContract, objective: bool, equalities: bool) -> ProblemFacts {
    ProblemFacts {
        variables: c.variables.len(),
        rows: c.rows.len(),
        objective,
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
/// The mathematical classes the facts and intent establish (ADR-0106 §7). Root intents
/// make a square problem a root system; coefficient and discrete classes are optimization
/// classes; a degree-two coefficient problem without a convexity certificate is
/// nonconvex; an explicit cone or a trajectory is never inferred from algebraic facts.
pub fn problem_classes(f: &ProblemFacts, intent: SolveIntent, convex: bool) -> Vec<ProblemClass> {
    let mut classes = Vec::new();
    if root_intent(intent) && square_root(f) {
        classes.push(ProblemClass::SquareRoot);
    }
    if continuous(f) {
        classes.push(ProblemClass::SmoothNlp);
    }
    if !root_intent(intent) {
        match (f.coefficients, f.quadratic, continuous(f)) {
            (true, false, true) => classes.push(ProblemClass::Linear),
            (true, false, false) => classes.push(ProblemClass::MixedLinear),
            (true, true, true) if convex => classes.push(ProblemClass::ConvexQuadratic),
            (true, true, true) => classes.push(ProblemClass::NonconvexQuadratic),
            (true, true, false) => classes.push(ProblemClass::MixedIntegerQuadratic),
            (false, _, false) => classes.push(ProblemClass::MixedIntegerNonlinear),
            (false, _, true) => {}
        }
    }
    classes
}
/// The one eligibility rule: a function of an adapter's capability record, its linkage and
/// the request. Every adapter is assessed through it.
pub fn admit(capability: &Capability, linked: bool, r: &Requirements<'_>) -> Vec<Ineligible> {
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
    let classes = problem_classes(f, r.intent, r.convex);
    if !classes.iter().any(|c| capability.classes.contains(c)) {
        reasons.push(Ineligible::Class { problem: classes });
    }
    let required = match capability.derivatives {
        DerivativeCapability::ExactHessianOrLimitedMemory
            if r.controls.hessian == HessianMode::Exact =>
        {
            Some(DerivativeOrder::Second)
        }
        DerivativeCapability::ExactHessianOrLimitedMemory
        | DerivativeCapability::JacobianOrProduct
        | DerivativeCapability::FirstWithSmoothSensitivities => Some(DerivativeOrder::First),
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
                let mut automatic: Vec<_> = self
                    .table
                    .adapters()
                    .filter_map(|a| a.automatic().map(|rank| (rank, a.backend())))
                    .filter(|(_, b)| admitted(*b))
                    .collect();
                automatic.sort_by_key(|(rank, _)| *rank);
                automatic.first().map(|(_, b)| *b).ok_or_else(|| {
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
    use crate::execution::{LINKED, adapter};
    #[test]
    fn caller_inventory_limits_automatic_selection() {
        let facts = root_facts();
        let requirements = Requirements {
            table: &Table::new(&[]),
            facts: &facts,
            intent: SolveIntent::Root,
            convex: false,
            controls: &crate::solve::Controls::default(),
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
        convex: bool,
    ) -> Result<Route, ProblemError> {
        Requirements {
            table: &LINKED,
            facts: f,
            intent,
            convex,
            controls: &crate::solve::Controls::default(),
        }
        .select(selection)
    }
    fn root_facts() -> ProblemFacts {
        ProblemFacts {
            variables: 1,
            rows: 1,
            objective: false,
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
                convex: false,
                controls: c,
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
                convex: false,
                controls: &controls
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
            convex: true,
            controls: &crate::solve::Controls::default(),
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
        assert!(admit(highs, true, &requirements).contains(&missing));
        assert!(admit(&consuming, true, &requirements).is_empty());
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
        // Coefficient MILPs and opaque nonlinear expressions are not conic data.
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
        f.coefficients = true;
        f.objective = true;
        assert_eq!(
            problem_classes(&f, SolveIntent::Optimize, false),
            [ProblemClass::SmoothNlp, ProblemClass::Linear]
        );
        // Degree two over affine rows: nonconvex without a certificate.
        f.quadratic = true;
        assert_eq!(
            problem_classes(&f, SolveIntent::Optimize, false),
            [ProblemClass::SmoothNlp, ProblemClass::NonconvexQuadratic]
        );
        assert_eq!(
            problem_classes(&f, SolveIntent::Optimize, true),
            [ProblemClass::SmoothNlp, ProblemClass::ConvexQuadratic]
        );
        f.domains = vec![ModelingVariableDomain::Binary];
        for convex in [false, true] {
            assert_eq!(
                problem_classes(&f, SolveIntent::Optimize, convex),
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
    }
    #[test]
    fn miqp_routes_to_scip() {
        let f = miqp_facts();
        let assess = |convex| {
            Requirements {
                table: &LINKED,
                facts: &f,
                intent: SolveIntent::Optimize,
                convex,
                controls: &crate::solve::Controls::default(),
            }
            .eligibility()
        };
        for convex in [false, true] {
            // Every admitted adapter represents the mixed-integer quadratic class.
            for e in assess(convex) {
                if e.reasons.is_empty() {
                    assert_eq!(e.backend, Backend::Scip);
                }
            }
            let route = select(&f, SolveIntent::Optimize, SolverSelection::Auto, convex);
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
            convex: false,
            controls: &crate::solve::Controls::default(),
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
