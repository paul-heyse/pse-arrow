// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pure contextual policy controls; no native session or evaluator is acquired.
use super::*;
use crate::execution::{BackendExecution, Input, Representation, Retained};
use crate::solve::{Controls, SolveReport};

#[derive(Debug)]
struct Candidate {
    backend: Backend,
    order: DerivativeCapability,
    rank: u8,
}
impl BackendExecution for Candidate {
    fn backend(&self) -> Backend {
        self.backend
    }
    fn capability(&self) -> &'static Capability {
        crate::execution::adapter(Backend::Ipopt).capability()
    }
    fn representation(&self) -> Representation {
        Representation::Nlp
    }
    fn linked(&self) -> bool {
        true
    }
    fn automatic(&self) -> Option<u8> {
        Some(self.rank)
    }
    fn required_order(&self, r: &Requirements<'_>) -> Option<DerivativeOrder> {
        if self.order == DerivativeCapability::JacobianOrProduct {
            Some(DerivativeOrder::First)
        } else {
            derivative_demand(self.capability(), r.controls)
        }
    }
    fn execute(&self, _: &mut Retained, _: Input<'_>) -> Result<SolveReport, ProblemError> {
        Err(ProblemError::internal("pure policy test must not execute"))
    }
}
static EXACT: Candidate = Candidate {
    backend: Backend::Ipopt,
    order: DerivativeCapability::ExactHessianOrLimitedMemory,
    rank: 0,
};
static FIRST: Candidate = Candidate {
    backend: Backend::Pounce,
    order: DerivativeCapability::JacobianOrProduct,
    rank: 1,
};
static TABLE: Table = Table::new(&[&EXACT, &FIRST]);
fn contract() -> crate::OracleContract {
    crate::OracleContract {
        identity: pse_ids::ContentHash::from_bytes([1; 32]),
        variables: vec![crate::Variable {
            id: pse_ids::SemanticId::from_bytes([1; 16]),
            lower: -1.,
            upper: 1.,
        }],
        rows: vec![],
        derivatives: DerivativeOrder::First,
        smoothness: DerivativeOrder::Second,
    }
}
fn context(c: &crate::OracleContract) -> Context<'_> {
    let pattern = faer::sparse::SymbolicSparseColMat::try_new_from_indices(0, 1, &[])
        .unwrap()
        .0;
    Context {
        structure: Some(
            crate::structural::oracle_structure(c, pattern.as_ref(), &[], true).unwrap(),
        ),
        oracle: Some(c),
        prepared: &[ArtifactDemand::Representation(Representation::Nlp)],
        ..test_context(&TABLE)
    }
}
#[test]
fn contextual_unit_constant_keeps_point_admission_independent_of_native_evidence() {
    let mut c = contract();
    c.variables.clear();
    c.derivatives = DerivativeOrder::Value;
    c.rows = vec![pse_ids::SemanticId::from_bytes([3; 16])];
    let pattern = faer::sparse::SymbolicSparseColMat::try_new_from_indices(1, 0, &[])
        .unwrap()
        .0;
    let original =
        crate::structural::oracle_structure(&c, pattern.as_ref(), &[(0., 1.)], true).unwrap();
    let facts = oracle_facts(&c, true, false);
    let controls = Controls::default();
    let requirements = Requirements {
        table: &TABLE,
        facts: &facts,
        intent: SolveIntent::Optimize,
        numerical_psd: false,
        least_squares: true,
        controls: &controls,
        settings: &crate::execution::BackendSettings::Default,
        sensitivity: false,
        context: Context {
            oracle: Some(&c),
            structure: Some(original.clone()),
            ..test_context(&TABLE)
        },
    };
    // A retained native proposal requires evidence unavailable to the direct route.
    let pending = vec![Eligibility {
        backend: Backend::Ipopt,
        reasons: vec![],
        causes: vec![],
        class_dependencies: vec![],
        evidence: vec![EvidenceDemand::Factorable],
        artifacts: vec![],
        factorable_refusals: vec![],
        structure: None,
        state: AssessmentState::PendingEvidence,
    }];
    let decision = requirements.decide_assessed(SolverSelection::Auto, pending, None);
    assert_eq!(decision.route().unwrap(), Route::Constant);
    assert_eq!(decision.state, AssessmentState::Ready);
    assert!(decision.evidence.is_empty() && decision.artifacts.is_empty());
    assert_eq!(decision.pending_backend, None);
    let point = decision.structure.as_ref().unwrap();
    assert_eq!(point.mode, crate::structural::Mode::PointEvaluation);
    assert_eq!(point.equations, original.equations);
    assert_eq!(point.witness, original.witness);
    assert_eq!(
        decision.row(c.identity, 0).selected,
        Some(pse_model::generated::enums::NativeRouteKind::Constant)
    );

    // Missing original structure is still a demanded point-evaluation dependency.
    let mut missing = requirements.context.clone();
    missing.structure = None;
    let missing = Requirements {
        context: missing,
        ..requirements
    };
    let decision = missing.decision(SolverSelection::Auto);
    assert_eq!(decision.selected, Some(Route::Constant));
    assert_eq!(decision.state, AssessmentState::PendingEvidence);
    assert_eq!(decision.evidence, [EvidenceDemand::Structure]);
    assert!(decision.route().is_err());

    // No objective, certification and native-only forms keep their own refusals.
    let mut no_objective = facts.clone();
    no_objective.objective = false;
    let no_objective = Requirements {
        facts: &no_objective,
        context: requirements.context.clone(),
        ..requirements
    };
    assert_eq!(
        no_objective.decision(SolverSelection::Auto).state,
        AssessmentState::Refused
    );
    let certify = Requirements {
        intent: SolveIntent::Certify,
        context: requirements.context.clone(),
        ..requirements
    };
    assert!(certify.decision(SolverSelection::Auto).route().is_err());
    let mut native = facts.clone();
    native.native = vec![NativeConstraintForm::Indicator];
    let native = Requirements {
        facts: &native,
        ..requirements
    };
    let decision = native.decision(SolverSelection::Auto);
    assert!(matches!(
        decision.refusal,
        Some(Refusal::ConstantNativeForms)
    ));
    assert_eq!(decision.state, AssessmentState::Refused);
}

#[test]
fn contextual_unit_available_second_order_does_not_fall_through_to_prepared_first() {
    let c = contract();
    let mut facts = oracle_facts(&c, true, true);
    facts.derivatives = DerivativeOrder::Second;
    let controls = Controls::default();
    let requirements = Requirements {
        table: &TABLE,
        facts: &facts,
        intent: SolveIntent::Optimize,
        numerical_psd: false,
        least_squares: false,
        controls: &controls,
        settings: &crate::execution::BackendSettings::Default,
        sensitivity: false,
        context: context(&c),
    };
    let decision = requirements.decision(SolverSelection::Auto);
    assert_eq!(decision.selected, Some(Route::Native(Backend::Ipopt)));
    assert_eq!(decision.state, AssessmentState::SupportedPendingArtifacts);
    assert_eq!(
        decision.artifacts,
        [ArtifactDemand::Derivatives(DerivativeOrder::Second)]
    );
    assert!(decision.route().is_err());
    let mut unavailable = facts.clone();
    unavailable.derivatives = DerivativeOrder::First;
    let requirements = Requirements {
        facts: &unavailable,
        ..requirements
    };
    assert_eq!(
        requirements
            .decision(SolverSelection::Auto)
            .route()
            .unwrap(),
        Route::Native(Backend::Pounce)
    );
}
#[test]
fn contextual_unit_callback_class_dependencies_preserve_actual_artifact_demands() {
    let c = contract();
    let mut facts = oracle_facts(&c, true, true);
    facts.derivatives = DerivativeOrder::Second;
    let controls = Controls::default();
    let obligation = pse_math::presolve::ClassDependency::UnestablishedObligation {
        instance: c.variables[0].id,
    };
    let missing = pse_math::presolve::ClassDependency::MissingSymbolicExpression {
        instance: c.variables[0].id,
        output: 7,
    };
    for dependencies in [vec![obligation.clone()], vec![missing, obligation]] {
        facts.class_status = pse_math::presolve::ClassStatus::Pending(dependencies.clone());
        let pending = pending_class_evidence(&facts, SolveIntent::Optimize, SolverSelection::Auto);
        let mut callback_context = context(&c);
        callback_context.pending_classes = &pending;
        let requirements = Requirements {
            table: &TABLE,
            facts: &facts,
            intent: SolveIntent::Optimize,
            numerical_psd: false,
            least_squares: false,
            controls: &controls,
            settings: &crate::execution::BackendSettings::Default,
            sensitivity: false,
            context: callback_context,
        };
        let decision = requirements.decision(SolverSelection::Auto);
        assert_eq!(decision.selected, Some(Route::Native(Backend::Ipopt)));
        assert_eq!(decision.state, AssessmentState::SupportedPendingArtifacts);
        assert_eq!(decision.pending_backend, None);
        assert!(decision.evidence.is_empty());
        assert_eq!(
            decision.artifacts,
            [ArtifactDemand::Derivatives(DerivativeOrder::Second)]
        );
        assert!(decision.route().is_err());
        assert_eq!(decision.eligibility[0].class_dependencies, dependencies);
        assert_eq!(
            facts.class_status,
            pse_math::presolve::ClassStatus::Pending(dependencies)
        );

        // Preparing the actual demanded callbacks resolves readiness without
        // turning the outstanding coefficient obligation into an established fact.
        let mut prepared = c.clone();
        prepared.derivatives = DerivativeOrder::Second;
        let mut prepared_facts = facts.clone();
        prepared_facts.prepared_derivatives = DerivativeOrder::Second;
        let mut prepared_context = context(&prepared);
        prepared_context.pending_classes = &pending;
        let prepared_requirements = Requirements {
            facts: &prepared_facts,
            context: prepared_context,
            ..requirements
        };
        let ready = prepared_requirements.decision(SolverSelection::Auto);
        assert_eq!(ready.state, AssessmentState::Ready);
        assert_eq!(ready.route().unwrap(), Route::Native(Backend::Ipopt));
        assert_eq!(
            ready.eligibility[0].class_dependencies,
            decision.eligibility[0].class_dependencies
        );
    }
    assert!(pending_class_evidence(&facts, SolveIntent::Root, SolverSelection::Auto).is_empty());
    assert!(
        pending_class_evidence(
            &facts,
            SolveIntent::Optimize,
            SolverSelection::Explicit(Backend::Ipopt)
        )
        .is_empty()
    );
}

#[test]
fn contextual_unit_admissible_coefficient_pending_keeps_priority_over_ready_callback() {
    static COEFFICIENT_AND_CALLBACKS: Table =
        Table::new(&[crate::execution::adapter(Backend::Highs), &EXACT, &FIRST]);
    let mut c = contract();
    c.derivatives = DerivativeOrder::Second;
    let mut facts = oracle_facts(&c, true, true);
    let dependency = pse_math::presolve::ClassDependency::UnestablishedObligation {
        instance: c.variables[0].id,
    };
    facts.class_status = pse_math::presolve::ClassStatus::Pending(vec![dependency.clone()]);
    let pending = pending_class_evidence(&facts, SolveIntent::Optimize, SolverSelection::Auto);
    let mut ctx = context(&c);
    ctx.snapshot = test_context(&COEFFICIENT_AND_CALLBACKS).snapshot;
    ctx.snapshot
        .adapters
        .get_mut(&Backend::Highs)
        .unwrap()
        .linked = true;
    ctx.pending_classes = &pending;
    let controls = Controls::default();
    let requirements = Requirements {
        table: &COEFFICIENT_AND_CALLBACKS,
        facts: &facts,
        intent: SolveIntent::Optimize,
        numerical_psd: false,
        least_squares: false,
        controls: &controls,
        settings: &crate::execution::BackendSettings::Default,
        sensitivity: false,
        context: ctx,
    };
    let decision = requirements.decision(SolverSelection::Auto);
    assert_eq!(decision.state, AssessmentState::PendingEvidence);
    assert_eq!(decision.pending_backend, Some(Backend::Highs));
    assert_eq!(decision.selected, None);
    assert!(
        decision
            .evidence
            .contains(&EvidenceDemand::Class(ProblemClass::Linear))
    );
    assert!(decision.evidence.contains(&EvidenceDemand::Coefficients));
    assert!(decision.route().is_err());
    let callback = decision
        .eligibility
        .iter()
        .find(|entry| entry.backend == Backend::Ipopt)
        .unwrap();
    assert_eq!(callback.state, AssessmentState::Ready);
    assert_eq!(callback.class_dependencies, [dependency]);
    assert!(callback.evidence.is_empty());
    assert_eq!(
        requirements
            .decision(SolverSelection::Explicit(Backend::Ipopt))
            .route()
            .unwrap(),
        Route::Native(Backend::Ipopt)
    );
}

#[test]
fn contextual_unit_callback_class_independence_preserves_contract_and_derivative_refusals() {
    let mut c = contract();
    c.derivatives = DerivativeOrder::Second;
    let mut facts = oracle_facts(&c, true, true);
    facts.class_status = pse_math::presolve::ClassStatus::Pending(vec![
        pse_math::presolve::ClassDependency::UnestablishedObligation {
            instance: c.variables[0].id,
        },
    ]);
    let pending = pending_class_evidence(&facts, SolveIntent::Optimize, SolverSelection::Auto);
    let mut ctx = context(&c);
    ctx.pending_classes = &pending;
    let controls = Controls::default();
    let requirements = Requirements {
        table: &TABLE,
        facts: &facts,
        intent: SolveIntent::Optimize,
        numerical_psd: false,
        least_squares: false,
        controls: &controls,
        settings: &crate::execution::BackendSettings::Default,
        sensitivity: false,
        context: ctx,
    };
    let mut missing = requirements.context.clone();
    missing.oracle = None;
    let missing = Requirements {
        context: missing,
        ..requirements
    };
    let decision = missing.decision(SolverSelection::Auto);
    assert_eq!(decision.state, AssessmentState::PendingEvidence);
    assert_eq!(decision.evidence, [EvidenceDemand::CallbackContract]);
    assert!(decision.route().is_err());

    let mut invalid = c.clone();
    invalid.variables[0].lower = 2.;
    let mut invalid_context = requirements.context.clone();
    invalid_context.oracle = Some(&invalid);
    let invalid = Requirements {
        context: invalid_context,
        ..requirements
    };
    let decision = invalid.decision(SolverSelection::Auto);
    assert_eq!(decision.state, AssessmentState::Refused);
    assert!(decision.eligibility.iter().all(|entry| {
        entry
            .causes
            .iter()
            .any(|cause| matches!(cause.as_ref(), ProblemError::Contract(_)))
    }));
    assert!(decision.route().is_err());

    let mut unavailable = facts.clone();
    unavailable.derivatives = DerivativeOrder::Value;
    let unavailable = Requirements {
        facts: &unavailable,
        ..requirements
    };
    let decision = unavailable.decision(SolverSelection::Auto);
    assert_eq!(decision.state, AssessmentState::Refused);
    assert!(decision.eligibility.iter().all(|entry| {
        entry
            .reasons
            .iter()
            .any(|reason| matches!(reason, Ineligible::Derivatives { .. }))
    }));
    assert!(
        unavailable
            .decision(SolverSelection::Explicit(Backend::Ipopt))
            .route()
            .is_err()
    );
}
#[test]
fn contextual_unit_representation_limit_preserves_coefficient_science() {
    let c = contract();
    let mut facts = oracle_facts(&c, true, true);
    facts.class_status = pse_math::presolve::ClassStatus::RepresentationLimited(
        pse_math::presolve::ClassWitness::CoefficientRange,
    );
    facts.affine_rows = vec![];
    facts.objective_degree = Some(1);
    assert_eq!(
        problem_classes(&facts, SolveIntent::Optimize, false),
        [ProblemClass::Linear, ProblemClass::SmoothNlp]
    );
    let controls = Controls::default();
    let requirements = Requirements {
        table: &crate::execution::LINKED,
        facts: &facts,
        intent: SolveIntent::Optimize,
        numerical_psd: false,
        least_squares: false,
        controls: &controls,
        settings: &crate::execution::BackendSettings::Default,
        sensitivity: false,
        context: test_context(&crate::execution::LINKED),
    };
    let assessed = crate::execution::adapter(Backend::Highs).assess(&requirements);
    assert!(assessed.causes.iter().any(|cause| matches!(
        cause.as_ref(),
        ProblemError::Math(pse_math::MathError::CoefficientRange)
    )));
}
#[test]
fn contextual_unit_original_structure_is_interpreted_for_each_candidate() {
    let mut c = contract();
    c.rows = vec![
        pse_ids::SemanticId::from_bytes([2; 16]),
        pse_ids::SemanticId::from_bytes([3; 16]),
    ];
    let pattern = faer::sparse::SymbolicSparseColMat::try_new_from_indices(
        2,
        1,
        &[faer::sparse::Pair::new(0, 0), faer::sparse::Pair::new(1, 0)],
    )
    .unwrap()
    .0;
    let original =
        crate::structural::oracle_structure(&c, pattern.as_ref(), &[(0., 0.), (0., 0.)], true)
            .unwrap();
    let mut facts = oracle_facts(&c, true, true);
    facts.coefficients = true;
    facts.affine_rows = vec![true, true];
    facts.objective_degree = Some(1);
    let mut context = test_context(&crate::execution::LINKED);
    context.structure = Some(original);
    for backend in [Backend::Ipopt, Backend::Highs] {
        context.snapshot.adapters.insert(
            backend,
            crate::execution::BuildObservation {
                linked: true,
                identity: None,
            },
        );
    }
    let controls = Controls::default();
    let requirements = Requirements {
        table: &crate::execution::LINKED,
        facts: &facts,
        intent: SolveIntent::Optimize,
        numerical_psd: false,
        least_squares: false,
        controls: &controls,
        settings: &crate::execution::BackendSettings::Default,
        sensitivity: false,
        context,
    };
    let nlp = assess_static(
        Backend::Ipopt,
        crate::execution::adapter(Backend::Ipopt).capability(),
        &requirements,
    );
    let native = assess_static(
        Backend::Highs,
        crate::execution::adapter(Backend::Highs).capability(),
        &requirements,
    );
    assert!(nlp.reasons.contains(&Ineligible::Structural));
    assert_eq!(nlp.structure.unwrap().mode, crate::structural::Mode::Nlp);
    assert!(!native.reasons.contains(&Ineligible::Structural));
    assert_eq!(
        native.structure.unwrap().mode,
        crate::structural::Mode::NativeFeasibility
    );
}

#[test]
fn contextual_unit_performed_missing_symbolic_export_refuses_only_its_representation() {
    let c = contract();
    let mut facts = oracle_facts(&c, true, true);
    facts.derivatives = DerivativeOrder::Second;
    let dependency = pse_math::presolve::ClassDependency::MissingSymbolicExpression {
        instance: c.variables[0].id,
        output: 7,
    };
    facts.class_status = pse_math::presolve::ClassStatus::Pending(vec![dependency.clone()]);
    let pending = pending_class_evidence(&facts, SolveIntent::Optimize, SolverSelection::Auto);
    let mut context = context(&c);
    context.pending_classes = &pending;
    let controls = Controls::default();
    let requirements = Requirements {
        table: &TABLE,
        facts: &facts,
        intent: SolveIntent::Optimize,
        numerical_psd: false,
        least_squares: false,
        controls: &controls,
        settings: &crate::execution::BackendSettings::Default,
        sensitivity: false,
        context,
    };
    let decision = requirements.decision(SolverSelection::Auto);
    assert_eq!(decision.selected, Some(Route::Native(Backend::Ipopt)));
    assert_eq!(decision.state, AssessmentState::SupportedPendingArtifacts);
    assert!(decision.evidence.is_empty());
    assert_eq!(decision.eligibility[0].class_dependencies, [dependency]);
    assert!(matches!(
        facts.class_status,
        pse_math::presolve::ClassStatus::Pending(_)
    ));
    let adapter = crate::execution::adapter(Backend::Highs);
    let mut assessment = assess_static(Backend::Highs, adapter.capability(), &requirements);
    crate::execution::assess_representation(adapter, &requirements, &mut assessment);
    assessment.finish();
    assert_eq!(assessment.state, AssessmentState::Refused);
    assert!(assessment.reasons.contains(&Ineligible::Contextual));
    assert_eq!(assessment.class_dependencies.len(), 1);
}

#[test]
fn contextual_unit_coefficient_pending_does_not_replace_factorable_evidence() {
    let c = contract();
    let mut facts = oracle_facts(&c, true, true);
    let dependency = pse_math::presolve::ClassDependency::MissingSymbolicExpression {
        instance: c.variables[0].id,
        output: 7,
    };
    facts.class_status = pse_math::presolve::ClassStatus::Pending(vec![dependency.clone()]);
    let pending = pending_class_evidence(&facts, SolveIntent::Optimize, SolverSelection::Auto);
    let mut context = context(&c);
    context.pending_classes = &pending;
    // Pure policy observations; this control constructs no foreign solver or proof.
    for backend in [Backend::Scip, Backend::Highs] {
        context.snapshot.adapters.insert(
            backend,
            crate::execution::BuildObservation {
                linked: true,
                identity: None,
            },
        );
    }
    let controls = Controls::default();
    let requirements = Requirements {
        table: &TABLE,
        facts: &facts,
        intent: SolveIntent::Optimize,
        numerical_psd: false,
        least_squares: false,
        controls: &controls,
        settings: &crate::execution::BackendSettings::Default,
        sensitivity: false,
        context,
    };
    let scip = crate::execution::adapter(Backend::Scip);
    let mut factorable = assess_static(Backend::Scip, scip.capability(), &requirements);
    crate::execution::assess_representation(scip, &requirements, &mut factorable);
    factorable.finish();
    assert_eq!(factorable.state, AssessmentState::PendingEvidence);
    assert_eq!(
        factorable.class_dependencies.as_slice(),
        std::slice::from_ref(&dependency)
    );
    assert_eq!(factorable.evidence, [EvidenceDemand::Factorable]);
    assert!(
        factorable
            .artifacts
            .contains(&ArtifactDemand::Representation(Representation::Factorable))
    );
    let mut prepared = requirements.context.clone();
    prepared.prepared = &[ArtifactDemand::Representation(Representation::Factorable)];
    let requirements = Requirements {
        context: prepared,
        ..requirements
    };
    let mut absent = assess_static(Backend::Scip, scip.capability(), &requirements);
    crate::execution::assess_representation(scip, &requirements, &mut absent);
    absent.finish();
    assert_eq!(absent.state, AssessmentState::PendingEvidence);
    assert_eq!(absent.evidence, [EvidenceDemand::Factorable]);
    let highs = crate::execution::adapter(Backend::Highs);
    let mut coefficients = assess_static(Backend::Highs, highs.capability(), &requirements);
    crate::execution::assess_representation(highs, &requirements, &mut coefficients);
    coefficients.finish();
    assert_eq!(coefficients.state, AssessmentState::Refused);
    assert!(coefficients.reasons.contains(&Ineligible::Contextual));
    assert_eq!(coefficients.class_dependencies, [dependency]);
    assert!(
        coefficients
            .evidence
            .iter()
            .any(|demand| matches!(demand, EvidenceDemand::Class(_)))
    );
    assert!(matches!(
        facts.class_status,
        pse_math::presolve::ClassStatus::Pending(_)
    ));
}

#[test]
fn contextual_unit_factorable_route_refusal_preserves_typed_member_and_bound_evidence() {
    use pse_model::diagnostic::{DiagnosticProjection, Observation};
    let c = contract();
    let facts = oracle_facts(&c, true, true);
    let controls = Controls::default();
    let mut decision = Requirements {
        table: &TABLE,
        facts: &facts,
        intent: SolveIntent::Optimize,
        numerical_psd: false,
        least_squares: false,
        controls: &controls,
        settings: &crate::execution::BackendSettings::Default,
        sensitivity: false,
        context: context(&c),
    }
    .decision(SolverSelection::Explicit(Backend::Ipopt));
    let member = c.variables[0].id;
    decision.eligibility[0].factorable_refusals = vec![
        crate::execution::Refusal::UnboundedNonlinear(pse_math::factorable::MissingBound {
            owner: pse_math::factorable::BoundOwner::Variable(member),
            lower: true,
            upper: false,
        }),
        crate::execution::Refusal::NativeOperand(usize::MAX),
    ];
    let diagnostic = ProblemError::RouteRefused(Box::new(decision))
        .boundary_diagnostic(pse_diagnostics::DiagnosticStage::ModelingAdmission);
    let bound = diagnostic
        .causes
        .iter()
        .find(|cause| cause.sources.contains(&member))
        .unwrap();
    assert!(matches!(
        bound.observations.get("missing_lower"),
        Some(Observation::Boolean(true))
    ));
    assert!(matches!(
        bound.observations.get("missing_upper"),
        Some(Observation::Boolean(false))
    ));
    assert!(diagnostic.causes.iter().any(|cause| matches!(cause.observations.get("native_constraint"), Some(Observation::Text(value)) if *value == usize::MAX.to_string())));
}

#[test]
fn contextual_unit_highs_conditional_method_and_scaling_are_assessed_before_effects() {
    use crate::settings::highs::{Method, Settings};
    let mut c = contract();
    c.derivatives = DerivativeOrder::Value;
    let mut facts = oracle_facts(&c, true, true);
    facts.coefficients = true;
    facts.affine_rows = vec![];
    facts.objective_degree = Some(1);
    let p = crate::CoefficientProblem {
        contract: c.clone(),
        objective: vec![1.0],
        objective_constant: 0.0,
        sense: pse_math::binding::ObjectiveSense::Minimize,
        domains: vec![ModelingVariableDomain::Continuous],
        assumptions: c.identity,
        constraints: faer::sparse::SparseColMat::try_new_from_triplets(0, 1, &[]).unwrap(),
        hessian: None,
        bounds: vec![],
        objectives: vec![],
    };
    let mut accuracy = crate::solve::ResolvedAccuracy::nominal();
    accuracy.native_scaling = false;
    let normalization = pse_math::normalization::Normalization {
        variables: vec![1.0],
        rows: vec![],
        objective: 1.0,
    };
    let tolerances = crate::quality::Tolerances {
        variables: vec![1e-8],
        rows: vec![],
        integrality: 1e-8,
    };
    let mut context = context(&c);
    context.coefficients = Some(&p);
    context.prepared = &[ArtifactDemand::Representation(Representation::Coefficients)];
    context.budgets = Some(crate::execution::Budgets {
        accuracy: &accuracy,
        normalization: &normalization,
        tolerances: &tolerances,
    });
    context.snapshot.adapters.insert(
        Backend::Highs,
        crate::execution::BuildObservation {
            linked: true,
            identity: None,
        },
    );
    let controls = Controls::default();
    let settings = crate::execution::BackendSettings::Highs(Settings {
        method: Method::Simplex,
        ..Settings::default()
    });
    let requirements = Requirements {
        table: &crate::execution::LINKED,
        facts: &facts,
        intent: SolveIntent::Optimize,
        numerical_psd: false,
        least_squares: false,
        controls: &controls,
        settings: &settings,
        sensitivity: false,
        context,
    };
    let adapter = crate::execution::adapter(Backend::Highs);
    assert_eq!(adapter.assess(&requirements).state, AssessmentState::Ready);
    let choose = crate::execution::BackendSettings::Highs(Settings::default());
    let choosing = Requirements {
        settings: &choose,
        context: requirements.context.clone(),
        ..requirements
    };
    assert_eq!(adapter.assess(&choosing).state, AssessmentState::Refused);
    let mut discrete = p.clone();
    discrete.domains[0] = ModelingVariableDomain::Integer;
    assert!(
        Settings {
            method: Method::Simplex,
            ..Settings::default()
        }
        .admit_model(&discrete, &crate::solve::ResolvedAccuracy::nominal())
        .is_err()
    );
    let mut quadratic = p.clone();
    quadratic.hessian = Some(
        faer::sparse::SparseColMat::try_new_from_triplets(
            1,
            1,
            &[faer::sparse::Triplet::new(0, 0, 2.0)],
        )
        .unwrap(),
    );
    assert!(
        Settings {
            method: Method::Simplex,
            ..Settings::default()
        }
        .admit_model(&quadratic, &crate::solve::ResolvedAccuracy::nominal())
        .is_err()
    );
    assert!(
        Settings::default()
            .admit_model(&quadratic, &crate::solve::ResolvedAccuracy::nominal())
            .is_ok()
    );
    assert!(
        Settings {
            nodes: Some(0),
            ..Settings::default()
        }
        .admit_controls(&controls)
        .is_err()
    );
    let mut reserved = controls.clone();
    reserved
        .options
        .insert("threads".into(), crate::solve::OptionValue::Integer(2));
    assert!(Settings::default().admit_controls(&reserved).is_err());
}
#[test]
fn contextual_unit_pounce_threshold_uses_original_keyed_bounds_and_selected_coordinates() {
    let mut c = contract();
    c.variables[0].upper = 1e20;
    c.rows = vec![pse_ids::SemanticId::from_bytes([2; 16])];
    let pattern = faer::sparse::SymbolicSparseColMat::try_new_from_indices(
        1,
        1,
        &[faer::sparse::Pair::new(0, 0)],
    )
    .unwrap()
    .0;
    let mut facts = oracle_facts(&c, true, false);
    facts.derivatives = DerivativeOrder::Second;
    let mut context = test_context(&crate::execution::LINKED);
    context.oracle = Some(&c);
    context.structure = Some(
        crate::structural::oracle_structure(
            &c,
            pattern.as_ref(),
            &[(f64::NEG_INFINITY, 1e20)],
            true,
        )
        .unwrap(),
    );
    context.prepared = &[ArtifactDemand::Representation(Representation::Nlp)];
    context.snapshot.adapters.insert(
        Backend::Pounce,
        crate::execution::BuildObservation {
            linked: true,
            identity: None,
        },
    );
    let accuracy = crate::solve::ResolvedAccuracy::nominal();
    let tolerances = crate::quality::Tolerances {
        variables: vec![1e-8],
        rows: vec![1e-8],
        integrality: 1e-8,
    };
    let scales = pse_math::normalization::Normalization {
        variables: vec![100.0],
        rows: vec![100.0],
        objective: 1.0,
    };
    context.budgets = Some(crate::execution::Budgets {
        accuracy: &accuracy,
        normalization: &scales,
        tolerances: &tolerances,
    });
    let controls = Controls::default();
    let requirements = Requirements {
        table: &crate::execution::LINKED,
        facts: &facts,
        intent: SolveIntent::Optimize,
        numerical_psd: false,
        least_squares: false,
        controls: &controls,
        settings: &crate::execution::BackendSettings::Default,
        sensitivity: false,
        context,
    };
    let adapter = crate::execution::adapter(Backend::Pounce);
    assert_ne!(
        adapter.assess(&requirements).state,
        AssessmentState::Refused
    );
    let row_unscaled = pse_math::normalization::Normalization {
        rows: vec![1.0],
        ..scales.clone()
    };
    let mut unscaled = Requirements {
        context: requirements.context.clone(),
        ..requirements
    };
    unscaled.context.budgets.as_mut().unwrap().normalization = &row_unscaled;
    assert_eq!(adapter.assess(&unscaled).state, AssessmentState::Refused);
    let variable_unscaled = pse_math::normalization::Normalization {
        variables: vec![1.0],
        ..scales.clone()
    };
    unscaled.context.budgets.as_mut().unwrap().normalization = &variable_unscaled;
    assert_eq!(adapter.assess(&unscaled).state, AssessmentState::Refused);
}

#[test]
fn contextual_unit_positive_lower_priority_cone_does_not_replace_missing_coefficient_proof() {
    let c = contract();
    let mut facts = oracle_facts(&c, true, true);
    facts.convexity = pse_math::convexity::Convexity {
        key: c.identity,
        class: pse_math::convexity::ConvexityClass::Cone(
            pse_math::convexity::ConeSummary::default(),
        ),
    };
    assert!(class_evidence_required(
        &facts,
        SolveIntent::Optimize,
        SolverSelection::Auto
    ));
    assert!(class_evidence_required(
        &facts,
        SolveIntent::Optimize,
        SolverSelection::Explicit(Backend::Clarabel)
    ));
    assert_eq!(
        pending_class_evidence(&facts, SolveIntent::Optimize, SolverSelection::Auto),
        [
            ProblemClass::Linear,
            ProblemClass::ConvexQuadratic,
            ProblemClass::NonconvexQuadratic
        ]
    );
    // Initialization with an authored objective retains the NLP class demands.
    assert!(class_evidence_required(
        &facts,
        SolveIntent::Initialize,
        SolverSelection::Auto
    ));
    let mut root_contract = c.clone();
    root_contract.rows = vec![pse_ids::SemanticId::from_bytes([2; 16])];
    let objective_facts = oracle_facts(&root_contract, true, true);
    assert!(!root_intent(&objective_facts, SolveIntent::Initialize));
    assert!(class_evidence_required(
        &objective_facts,
        SolveIntent::Initialize,
        SolverSelection::Auto
    ));
    let root_facts = oracle_facts(&root_contract, false, true);
    assert!(root_intent(&root_facts, SolveIntent::Initialize));
    assert!(!class_evidence_required(
        &root_facts,
        SolveIntent::Initialize,
        SolverSelection::Auto
    ));
    assert!(
        pending_class_evidence(&root_facts, SolveIntent::Initialize, SolverSelection::Auto)
            .is_empty()
    );
    facts.class_status =
        pse_math::presolve::ClassStatus::RuledOut(pse_math::presolve::ClassWitness::NonAffineRow {
            row: c.variables[0].id,
        });
    assert!(!class_evidence_required(
        &facts,
        SolveIntent::Optimize,
        SolverSelection::Auto
    ));
    assert!(
        pending_class_evidence(&facts, SolveIntent::Optimize, SolverSelection::Auto).is_empty()
    );
    assert_eq!(
        problem_classes(&facts, SolveIntent::Optimize, false)[0],
        ProblemClass::ContinuousCone
    );
}
