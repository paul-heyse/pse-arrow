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
    let input = super::super::tests::inputs();
    let names = BTreeMap::from([
        (
            "Scalar".into(),
            input.quantities.neutral_dimensionless().unwrap(),
        ),
        ("Count".into(), quantity("3a8f6d2c9b1e4f7a8c5d0e3b6a9f2c18")),
        (
            "Indicator".into(),
            quantity("b5d9e1c4a7f2483e9d6c1b0a5e8f3d27"),
        ),
        ("Power".into(), power()),
    ]);
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
    let (_, prepared, _) = workspace.prepare_modeling_case_cancellable(
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
    Ok((variables[0].domain, variables[0].lower, variables[0].upper))
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
    let input = super::super::tests::inputs();
    let mut w = CompilerWorkspace::new(input, WorkspaceLimits::default()).unwrap();
    assert!(w.publish_modeling(rows, BTreeMap::new()).is_err());
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
        let variable = &admitted.case.variables()[0];
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
        values: BTreeMap::from([("n".into(), 2.0)]),
        variables: BTreeMap::from([(
            "n".into(),
            ModelingVariableState {
                fixed: Some(true),
                ..Default::default()
            },
        )]),
    };
    let (_, prepared, _) = w
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
    let declared = &admitted.case.variables()[0];
    assert_eq!((declared.lower, declared.upper), (Some(0.0), Some(1.0)));
    for (case, bounds) in [
        (ModelingCaseBindings::default(), (Some(0.0), Some(1.0))),
        (bounded(None, Some(5.0)), (Some(0.0), Some(1.0))),
        (bounded(Some(-3.0), Some(0.5)), (Some(0.0), Some(0.5))),
        (bounded(Some(1.0), None), (Some(1.0), Some(1.0))),
    ] {
        assert_eq!(
            prepare(&mut w, root, &case).unwrap(),
            (Domain::Binary, bounds.0, bounds.1)
        );
    }
    for case in [bounded(Some(2.0), None), bounded(Some(0.2), Some(0.8))] {
        assert_eq!(
            refusal(prepare(&mut w, root, &case).unwrap_err()),
            (
                "n".into(),
                DomainAnalysis::Preparation,
                DomainRefusal::EmptyDomain
            )
        );
    }
}
