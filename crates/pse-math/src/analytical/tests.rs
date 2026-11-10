// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
fn scope() -> ExecutionScope {
    ExecutionScope::new(pse_diagnostics::Cancellation::new(false), None)
}
fn body(inputs: usize, expression: Atom, requirements: Vec<Stage>) -> PreparedBody {
    let mut stages = requirements;
    stages.push(Stage::Block {
        expressions: vec![expression],
        outputs: vec![inputs],
        source: SemanticId::NIL,
    });
    PreparedBody::new(
        inputs,
        inputs + 1,
        vec![inputs],
        stages,
        DerivativeOrder::Second,
    )
    .unwrap()
}
fn selected(inputs: usize, expression: Atom) -> SelectedExpression {
    body(inputs, expression, vec![])
        .analytical_expression(0, EvaluationLimits::default(), &scope())
        .unwrap()
}
#[test]
fn analytical_caloric_polynomial_preserves_reference_and_exact_coefficients() {
    crate::initialize().unwrap();
    let [t, tref, href, a, b] = std::array::from_fn(crate::library::formal).map(Result::unwrap);
    let theta = &t - &tref;
    let h = &href + &a * &theta + &b * theta.pow(2) / Atom::num(2);
    let source = selected(5, h);
    let derivative = source
        .derivative(&[0], EvaluationLimits::default(), &scope())
        .unwrap();
    assert!(
        (derivative.expression() - (&a + &b * &theta))
            .expand()
            .is_zero()
    );
    let series = source
        .series(
            0,
            &tref,
            SeriesDepth::absolute(4),
            EvaluationLimits::default(),
            &scope(),
        )
        .unwrap();
    let AnalyticalObservation::Series(observation) = series.observation else {
        panic!("series")
    };
    assert!(observation.exactly_terminating);
    assert_eq!(observation.ramification, 1);
    assert!(observation.coefficients.iter().any(|term| {
        term.exponent == RationalView::from(&Rational::from(2))
            && term.coefficient.symbolica
                == (&b / Atom::num(2))
                    .printer(PrintOptions::file())
                    .to_string()
    }));
    assert!(
        (source.expression().replace(t.clone()).with(tref.clone()) - &href)
            .expand()
            .is_zero()
    );
    assert!(matches!(
        source.derivative(&[5], EvaluationLimits::default(), &scope()),
        Err(MathError::Contract(_))
    ));
}
#[test]
fn analytical_stable_log_ratio_has_higher_series_without_changing_numerical_heads() {
    crate::initialize().unwrap();
    let t = crate::library::formal(0).unwrap();
    let tref = crate::library::formal(1).unwrap();
    let expression = crate::stable::call(crate::stable::Operation::LogRatio, &[&t, &tref]).unwrap();
    let source_id = SemanticId::from_bytes([73; 16]);
    let original = body(
        2,
        expression,
        vec![
            Stage::Require {
                argument: 0,
                condition: Condition::Positive,
                order: DerivativeOrder::Value,
                source: source_id,
                lineage: None,
            },
            Stage::Require {
                argument: 1,
                condition: Condition::Positive,
                order: DerivativeOrder::Value,
                source: source_id,
                lineage: None,
            },
        ],
    );
    let source = original
        .analytical_expression(0, EvaluationLimits::default(), &scope())
        .unwrap();
    assert_eq!(source.requirements().len(), 2);
    assert!(
        source
            .requirements()
            .iter()
            .all(|requirement| requirement.source == source_id)
    );
    assert_eq!(source.original_body(), &original);
    let result = source
        .series(
            0,
            &tref,
            SeriesDepth::absolute(4),
            EvaluationLimits::default(),
            &scope(),
        )
        .unwrap();
    let expected = (&t - &tref) / &tref - (&t - &tref).pow(2) / (Atom::num(2) * tref.pow(2))
        + (&t - &tref).pow(3) / (Atom::num(3) * tref.pow(3))
        - (&t - &tref).pow(4) / (Atom::num(4) * tref.pow(4));
    assert!((result.expression() - expected).expand().cancel().is_zero());
    let AnalyticalObservation::Series(observation) = result.observation else {
        panic!("series")
    };
    assert!(!observation.exactly_terminating);
    assert_eq!(
        observation.absolute_order,
        Some(RationalView::from(&Rational::from(5)))
    );
}
#[test]
fn analytical_observations_honor_original_stop_and_refuse_unsupported_region() {
    crate::initialize().unwrap();
    let x = crate::library::formal(0).unwrap();
    let original = body(1, x.clone(), vec![]);
    let cancelled = ExecutionScope::new(pse_diagnostics::Cancellation::new(true), None);
    assert!(matches!(
        original.analytical_expression(0, EvaluationLimits::default(), &cancelled),
        Err(MathError::Cancelled)
    ));
    let expired = ExecutionScope::new(
        pse_diagnostics::Cancellation::new(false),
        Some(std::time::Instant::now()),
    );
    assert!(
        original
            .analytical_expression(0, EvaluationLimits::default(), &expired)
            .is_err()
    );
    let source = selected(1, x);
    assert!(
        source
            .series(
                0,
                &Atom::Zero,
                SeriesDepth::absolute(32),
                EvaluationLimits {
                    derivative_components: 4,
                    ..Default::default()
                },
                &scope()
            )
            .is_err()
    );
    let opaque = crate::stable::call(
        crate::stable::Operation::Magnitude,
        &[&Atom::num(1), &Atom::num(2)],
    )
    .unwrap();
    assert!(
        body(0, opaque, vec![])
            .analytical_expression(0, EvaluationLimits::default(), &scope())
            .is_err()
    );
}
#[test]
fn analytical_identity_normalizes_rational_difference_and_retains_inconclusive_functions() {
    crate::initialize().unwrap();
    let x = crate::library::formal(0).unwrap();
    let first = selected(1, (&x + Atom::num(1)).pow(2));
    let second = selected(1, x.pow(2) + Atom::num(2) * &x + Atom::num(1));
    let result = first
        .identity(&second, EvaluationLimits::default(), &scope())
        .unwrap();
    assert!(matches!(
        result.observation,
        AnalyticalObservation::Identity {
            meaning: IdentityMeaning::Zero,
            ..
        }
    ));
    let other = selected(1, x.sin());
    let result = other
        .identity(&second, EvaluationLimits::default(), &scope())
        .unwrap();
    assert!(matches!(
        result.observation,
        AnalyticalObservation::Identity {
            meaning: IdentityMeaning::Inconclusive,
            ..
        }
    ));
}
#[test]
fn analytical_printer_preserves_namespace_collisions() {
    crate::initialize().unwrap();
    let expression = symbolica::parse!("analytical_a::x+analytical_b::x");
    let view = mathematical_view(&expression, EvaluationLimits::default(), &scope()).unwrap();
    assert!(
        view.symbolica.contains("analytical_a::x") && view.symbolica.contains("analytical_b::x")
    );
    assert!(view.latex.contains("analytical_a") && view.latex.contains("analytical_b"));
    assert!(view.typst.contains("analytical_a") && view.typst.contains("analytical_b"));
}
