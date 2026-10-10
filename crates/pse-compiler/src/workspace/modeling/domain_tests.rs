// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Declared variable domains: physical typing and finite-bound admission (ADR-0103).
use super::*;
use pse_authoring::{
    ParseBudget,
    language::{IdentityPolicy, parse},
};
use pse_model::generated::enums::ModelingVariableDomain as Domain;
use pse_modeling::{DomainAnalysis, DomainRefusal, ModelingError};

pub(super) fn quantity(hex: &str) -> QuantityTypeId {
    QuantityTypeId::from_id(SemanticId::parse_hex(hex).unwrap())
}
pub(super) fn power() -> QuantityTypeId {
    quantity("e1f2106da9eb4fe0aa2749fa5469fa1a")
}
pub(super) fn setup(text: &str) -> (CompilerWorkspace, DeclarationId) {
    let input = crate::authored_transfer_tests::context();
    let names = PhysicalScope::default();
    let rows = parse(
        text,
        SemanticId::from_bytes([82; 16]),
        IdentityPolicy::Named,
        ParseBudget::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let mut workspace = CompilerWorkspace::new(input, WorkspaceLimits::default()).unwrap();
    workspace.publish_modeling(rows, names).unwrap();
    (workspace, root)
}
fn admit(workspace: &mut CompilerWorkspace, root: DeclarationId) -> Result<Arc<AdmittedModeling>> {
    workspace.admit_modeling(
        root,
        InstanceId::from_id(SemanticId::NIL),
        Bindings::default(),
        Limits::default(),
    )
}
fn bounded(lower: Option<f64>, upper: Option<f64>) -> ModelingCaseBindings {
    ModelingCaseBindings {
        members: BTreeMap::new(),
        values: BTreeMap::new(),
        variables: BTreeMap::from([(
            "n".into(),
            ModelingVariableState {
                fixed: None,
                lower: lower.map(Some),
                upper: upper.map(Some),
            },
        )]),
    }
}
/// The prepared bounds of `n`, or the typed refusal.
fn prepare(
    workspace: &mut CompilerWorkspace,
    root: DeclarationId,
    case: &ModelingCaseBindings,
) -> Result<(Domain, Option<f64>, Option<f64>)> {
    prepare_recorded(workspace, root, case).map(|(bounds, _)| bounds)
}
/// A variable's prepared domain and bounds.
type Prepared = (Domain, Option<f64>, Option<f64>);
/// The prepared bounds of `n` with the tightenings admission recorded.
fn prepare_recorded(
    workspace: &mut CompilerWorkspace,
    root: DeclarationId,
    case: &ModelingCaseBindings,
) -> Result<(Prepared, Vec<pse_modeling::DomainTightening>)> {
    let (_, prepared, _, tightenings) = workspace.prepare_modeling_case_cancellable(
        root,
        InstanceId::from_id(SemanticId::NIL),
        Bindings::default(),
        Limits::default(),
        case,
        DerivativeOrder::First,
        Profile::default(),
        Arc::new(AtomicBool::new(false)),
    )?;
    let variables = prepared.plan.structure().variables();
    assert_eq!(variables.len(), 1);
    assert_eq!(prepared.facts.domains, vec![variables[0].domain]);
    Ok((
        (variables[0].domain, variables[0].lower, variables[0].upper),
        tightenings,
    ))
}
fn refusal(error: CompileError) -> (String, DomainAnalysis, DomainRefusal) {
    match error {
        CompileError::Modeling(ModelingError::Domain {
            path,
            analysis,
            reason,
            ..
        }) => (
            // The refusal names the variable by its instance path.
            path.rsplit('.').next().unwrap_or_default().to_owned(),
            analysis,
            reason,
        ),
        other => panic!("expected a typed domain refusal, got {other:?}"),
    }
}

#[test]
fn integer_requires_count_or_indicator_type() {
    for (ty, domain) in [
        ("Scalar", "integer"),
        ("Scalar", "binary"),
        ("Power", "integer"),
    ] {
        let (mut w, root) = setup(&format!(
            "package p {{ def Root {{ var n: {ty} in {domain}; eq e: n == n; }} }}"
        ));
        assert_eq!(
            refusal(admit(&mut w, root).unwrap_err()),
            (
                "n".into(),
                DomainAnalysis::Preparation,
                DomainRefusal::QuantityKind
            ),
            "{ty} in {domain}"
        );
    }
    for ty in ["Count", "Indicator"] {
        for (spelling, domain) in [("integer", Domain::Integer), ("binary", Domain::Binary)] {
            let (mut w, root) = setup(&format!(
                "package p {{ def Root {{ var n: {ty} in {spelling}; eq e: n == 1{{1}}; }} }}"
            ));
            let admitted = admit(&mut w, root).unwrap();
            assert_eq!(admitted.case.variables()[0].domain, domain);
        }
    }
    // A discrete domain needs a physical type at all.
    let rows = parse(
        "package p { entity kind k {} def Root { var n: k in integer; } }",
        SemanticId::NIL,
        IdentityPolicy::Named,
        ParseBudget::default(),
    )
    .unwrap();
    let input = crate::authored_transfer_tests::context();
    let mut w = CompilerWorkspace::new(input, WorkspaceLimits::default()).unwrap();
    assert!(w.publish_modeling(rows, PhysicalScope::default()).is_err());
}

#[test]
fn semicontinuous_keeps_physical_quantity() {
    for (spelling, domain) in [
        ("semicontinuous", Domain::Semicontinuous),
        ("semiinteger", Domain::Semiinteger),
    ] {
        let (mut w, root) = setup(&format!(
            "package p {{ def Root {{ var n: Power in {spelling}; eq cap: n <= 10{{W}}; }} }}"
        ));
        let admitted = admit(&mut w, root).unwrap();
        let variable = admitted.case.variables().get(0).unwrap();
        assert_eq!((variable.domain, variable.port.quantity), (domain, power()));
        assert_eq!(
            prepare(&mut w, root, &bounded(Some(2.0), Some(8.0))).unwrap(),
            (domain, Some(2.0), Some(8.0))
        );
        // The active branch is a positive interval; the zero branch is separate.
        assert_eq!(
            refusal(prepare(&mut w, root, &bounded(Some(0.0), Some(8.0))).unwrap_err()).2,
            DomainRefusal::EmptyDomain
        );
    }
}

#[test]
fn integer_requires_finite_bounds() {
    let (mut w, root) =
        setup("package p { def Root { var n: Count in integer; eq e: n >= 1{1}; } }");
    for case in [
        ModelingCaseBindings::default(),
        bounded(Some(0.0), None),
        bounded(None, Some(5.0)),
    ] {
        assert_eq!(
            refusal(prepare(&mut w, root, &case).unwrap_err()),
            (
                "n".into(),
                DomainAnalysis::Preparation,
                DomainRefusal::InfiniteBound
            )
        );
    }
    assert_eq!(
        prepare(&mut w, root, &bounded(Some(0.0), Some(5.0))).unwrap(),
        (Domain::Integer, Some(0.0), Some(5.0))
    );
    assert_eq!(
        refusal(prepare(&mut w, root, &bounded(Some(2.5), Some(2.7))).unwrap_err()).2,
        DomainRefusal::EmptyDomain
    );
    // A case that fixes the variable leaves no search range to bound.
    let fixed = ModelingCaseBindings {
        members: BTreeMap::new(),
        values: BTreeMap::from([("n".into(), 2.0)]),
        variables: BTreeMap::from([(
            "n".into(),
            ModelingVariableState {
                fixed: Some(true),
                ..Default::default()
            },
        )]),
    };
    let (_, prepared, _, _) = w
        .prepare_modeling_case_cancellable(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings::default(),
            Limits::default(),
            &fixed,
            DerivativeOrder::First,
            Profile::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert_eq!(prepared.facts.variables, 0);
}

#[test]
fn binary_implies_unit_box() {
    let (mut w, root) =
        setup("package p { def Root { var n: Indicator in binary; eq e: n <= 1{1}; } }");
    let admitted = admit(&mut w, root).unwrap();
    let declared = admitted.case.variables().get(0).unwrap();
    assert_eq!((declared.lower, declared.upper), (Some(0.0), Some(1.0)));
    for (case, bounds) in [
        (ModelingCaseBindings::default(), (Some(0.0), Some(1.0))),
        (bounded(Some(0.0), Some(1.0)), (Some(0.0), Some(1.0))),
        (bounded(Some(1.0), None), (Some(1.0), Some(1.0))),
        (bounded(None, Some(0.0)), (Some(0.0), Some(0.0))),
    ] {
        assert_eq!(
            prepare(&mut w, root, &case).unwrap(),
            (Domain::Binary, bounds.0, bounds.1)
        );
    }
    // A box inside the unit box that contains neither 0 nor 1.
    assert_eq!(
        refusal(prepare(&mut w, root, &bounded(Some(0.2), Some(0.8))).unwrap_err()),
        (
            "n".into(),
            DomainAnalysis::Preparation,
            DomainRefusal::EmptyDomain
        )
    );
}

#[test]
fn binary_bounds_outside_unit_box_refused() {
    let (mut w, root) =
        setup("package p { def Root { var n: Indicator in binary; eq e: n <= 1{1}; } }");
    // No bound is clamped into the unit box the domain implies: each conflicts.
    for case in [
        bounded(None, Some(5.0)),
        bounded(Some(-3.0), Some(0.5)),
        bounded(Some(2.0), None),
        bounded(Some(-0.1), Some(1.0)),
        bounded(Some(0.0), Some(1.0 + f64::EPSILON)),
        bounded(None, Some(f64::INFINITY)),
        bounded(Some(f64::NAN), None),
    ] {
        let error = prepare(&mut w, root, &case).unwrap_err();
        let CompileError::Modeling(cause) = &error else {
            panic!("expected a modeling refusal, got {error:?}");
        };
        let diagnostic = cause.boundary_diagnostic();
        assert_eq!(
            refusal(error),
            (
                "n".into(),
                DomainAnalysis::Preparation,
                DomainRefusal::ConflictingBound
            ),
            "{case:?}"
        );
        // A conflicting bound is an invalid model, not a missing capability.
        assert_eq!(
            diagnostic.class,
            pse_model::diagnostic::BoundaryClass::InvalidModel
        );
        assert_eq!(
            diagnostic.rule,
            pse_diagnostics::DiagnosticRule::ModelingDomain
        );
    }
}

#[test]
fn integer_bounds_tightened_inward_and_recorded() {
    use pse_model::diagnostic::{Observation, Severity};
    let (mut w, root) =
        setup("package p { def Root { var n: Count in integer; eq e: n >= 1{1}; } }");
    let ((domain, lower, upper), tightenings) =
        prepare_recorded(&mut w, root, &bounded(Some(0.5), Some(5.5))).unwrap();
    // Exactly the ceiling of the lower bound and the floor of the upper.
    assert_eq!(
        (domain, lower, upper),
        (Domain::Integer, Some(1.0), Some(5.0))
    );
    let [tightening] = tightenings.as_slice() else {
        panic!("one tightening expected, got {tightenings:?}");
    };
    assert_eq!(tightening.path.rsplit('.').next(), Some("n"));
    assert_eq!(tightening.domain, Domain::Integer);
    assert_eq!(
        (tightening.specified, tightening.tightened),
        ([0.5, 5.5], [1.0, 5.0])
    );
    // The informational finding names the variable and both bound pairs.
    let finding = tightening.boundary_diagnostic();
    assert_eq!(
        finding.rule,
        pse_diagnostics::DiagnosticRule::ModelingDomainTightened
    );
    assert_eq!(finding.severity, Severity::Info);
    assert!(finding.sources.contains(&tightening.variable));
    assert!(matches!(
        finding.observations.get("specified_lower"),
        Some(Observation::Real(v)) if *v == 0.5
    ));
    assert!(matches!(
        finding.observations.get("upper"),
        Some(Observation::Real(v)) if *v == 5.0
    ));
    // Integral bounds are admitted unchanged and record nothing.
    let (bounds, tightenings) =
        prepare_recorded(&mut w, root, &bounded(Some(1.0), Some(5.0))).unwrap();
    assert_eq!(bounds, (Domain::Integer, Some(1.0), Some(5.0)));
    assert!(tightenings.is_empty());
    // One endpoint tightens alone; a ceiling of -0.5 is a positive zero.
    let ((_, lower, _), tightenings) =
        prepare_recorded(&mut w, root, &bounded(Some(-0.5), Some(5.0))).unwrap();
    assert_eq!(lower.map(f64::to_bits), Some(0.0_f64.to_bits()));
    assert_eq!(tightenings[0].tightened, [0.0, 5.0]);
    // The tightened structure is the integral one: they share a view, not a record.
    let model = w
        .prepare_modeling_cancellable(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings {
                demand: vec!["n".into()],
                ..Bindings::default()
            },
            Limits::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    let n = model.model.paths["n"];
    let states = |lower: f64, upper: f64| {
        BTreeMap::from([(
            n,
            ModelingVariableState {
                fixed: None,
                lower: Some(Some(lower)),
                upper: Some(Some(upper)),
            },
        )])
    };
    let fractional = model.bound_structure(&states(0.5, 5.5)).unwrap();
    let integral = model.bound_structure(&states(1.0, 5.0)).unwrap();
    assert_eq!(fractional.structure.key(), integral.structure.key());
    assert_eq!(
        (fractional.tightenings.len(), integral.tightenings.len()),
        (1, 0)
    );

    // Semi-integer and binary domains tighten too; a semicontinuous one does not.
    for (declaration, bounds, expected, recorded) in [
        (
            "var n: Power in semiinteger; eq cap: n <= 10{W};",
            (2.5, 8.7),
            (Domain::Semiinteger, 3.0, 8.0),
            true,
        ),
        (
            "var n: Power in semicontinuous; eq cap: n <= 10{W};",
            (2.5, 8.7),
            (Domain::Semicontinuous, 2.5, 8.7),
            false,
        ),
        (
            "var n: Indicator in binary; eq e: n <= 1{1};",
            (0.0, 0.5),
            (Domain::Binary, 0.0, 0.0),
            true,
        ),
    ] {
        let (mut w, root) = setup(&format!("package p {{ def Root {{ {declaration} }} }}"));
        let (prepared, tightenings) =
            prepare_recorded(&mut w, root, &bounded(Some(bounds.0), Some(bounds.1))).unwrap();
        assert_eq!(
            prepared,
            (expected.0, Some(expected.1), Some(expected.2)),
            "{declaration}"
        );
        assert_eq!(!tightenings.is_empty(), recorded, "{declaration}");
    }
    // Tightening that leaves no integer is still refused.
    assert_eq!(
        refusal(prepare(&mut w, root, &bounded(Some(2.5), Some(2.7))).unwrap_err()).2,
        DomainRefusal::EmptyDomain
    );
}
