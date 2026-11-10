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
fn analytical_native_controls_stop_recursive_work_without_a_partial_product() {
    crate::initialize().unwrap();
    let x = crate::library::formal(0).unwrap();
    let expression = (&x + Atom::num(1)).log() + x.pow(3);
    let axis = x.get_symbol().unwrap().into();
    let mut polls = 0;
    let derivative = symbolica::analytical::derivative_with(
        expression.as_view(),
        &axis,
        &mut |_: symbolica::evaluate::OperationExtent| {
            polls += 1;
            if polls == 3 {
                Err("original recursive stop")
            } else {
                Ok(())
            }
        },
    );
    assert_eq!(derivative, Err("original recursive stop"));
    assert_eq!(polls, 3);
    let mut polls = 0;
    let series = symbolica::analytical::series_with(
        expression.as_view(),
        &axis,
        Atom::Zero.as_view(),
        SeriesDepth::absolute(5),
        &mut |_: symbolica::evaluate::OperationExtent| {
            polls += 1;
            if polls == 3 {
                Err("original series stop")
            } else {
                Ok(())
            }
        },
    );
    assert!(matches!(
        series,
        Err(symbolica::analytical::SeriesOperationError::Control(
            "original series stop"
        ))
    ));
    assert_eq!(polls, 3);
}

#[test]
fn analytical_indexed_common_alphabet_preserves_fixed_axes_and_binder_provenance() {
    crate::initialize().unwrap();
    let id = |byte| SemanticId::from_bytes([byte; 16]);
    let factors = |binder, external| {
        vec![
            TensorOccurrence {
                family: id(10),
                axes: vec![
                    TensorAxis::External {
                        identity: external,
                        group: id(1),
                    },
                    TensorAxis::Dummy {
                        binder,
                        group: id(1),
                    },
                ],
            },
            TensorOccurrence {
                family: id(11),
                axes: vec![TensorAxis::Dummy {
                    binder,
                    group: id(1),
                }],
            },
        ]
    };
    let observe = |binder, external| {
        canonicalize_contraction(
            &factors(binder, external),
            &[IndexAlphabet {
                group: id(1),
                labels: 2,
            }],
            EvaluationLimits::default(),
            &scope(),
        )
        .unwrap()
    };
    let first = observe(id(2), id(3));
    let renamed = observe(id(4), id(3));
    let fixed_changed = observe(id(2), id(5));
    assert_eq!(first.expression(), renamed.expression());
    assert_ne!(first.expression(), fixed_changed.expression());
    let AnalyticalObservation::Indexed(first) = first.observation else {
        panic!("indexed")
    };
    let AnalyticalObservation::Indexed(renamed) = renamed.observation else {
        panic!("indexed")
    };
    assert_ne!(first.original, renamed.original);
    assert_eq!(
        first.original_to_input[0].input,
        renamed.original_to_input[0].input
    );
    assert_ne!(
        first.original_to_input[0].binder,
        renamed.original_to_input[0].binder
    );
    assert_eq!(first.external_indices.len(), 1);
    let repeated = |external| {
        vec![
            TensorOccurrence {
                family: id(10),
                axes: vec![
                    TensorAxis::External {
                        identity: external,
                        group: id(1),
                    },
                    TensorAxis::Dummy {
                        binder: id(2),
                        group: id(1),
                    },
                ],
            },
            TensorOccurrence {
                family: id(11),
                axes: vec![
                    TensorAxis::External {
                        identity: external,
                        group: id(1),
                    },
                    TensorAxis::Dummy {
                        binder: id(2),
                        group: id(1),
                    },
                ],
            },
        ]
    };
    let first = canonicalize_contraction(
        &repeated(id(3)),
        &[IndexAlphabet {
            group: id(1),
            labels: 2,
        }],
        EvaluationLimits::default(),
        &scope(),
    )
    .unwrap();
    let changed = canonicalize_contraction(
        &repeated(id(5)),
        &[IndexAlphabet {
            group: id(1),
            labels: 2,
        }],
        EvaluationLimits::default(),
        &scope(),
    )
    .unwrap();
    assert_ne!(first.expression(), changed.expression());
    let AnalyticalObservation::Indexed(first) = first.observation else {
        panic!("indexed")
    };
    assert_eq!(first.external_indices.len(), 1);
    assert_eq!(first.dummy_indices.len(), 1);
}

#[test]
fn analytical_native_pade_checks_formal_congruence_and_preserves_symbolic_coefficients() {
    crate::initialize().unwrap();
    let x = crate::library::formal(0).unwrap();
    let a = crate::library::formal(1).unwrap();
    let source = selected(2, (&a * &x).exp());
    let result = source
        .rational_approximation(0, &Atom::Zero, 1, 1, EvaluationLimits::default(), &scope())
        .unwrap();
    let expected =
        (Atom::num(1) + &a * &x / Atom::num(2)) / (Atom::num(1) - &a * &x / Atom::num(2));
    let comparison = selected(2, expected);
    assert!(matches!(
        selected(2, result.expression().clone())
            .identity(&comparison, EvaluationLimits::default(), &scope())
            .unwrap()
            .observation,
        AnalyticalObservation::Identity {
            meaning: IdentityMeaning::Zero,
            ..
        }
    ));
    assert_eq!(
        result.expression().replace(a.clone()).with(Atom::Zero),
        Atom::num(1),
        "coefficient-field common scales cannot introduce a removable parameter pole"
    );
    let AnalyticalObservation::RationalApproximation { formal_order, .. } = result.observation
    else {
        panic!("Padé")
    };
    assert_eq!(formal_order, RationalView::from(&Rational::from(3)));
    let zero = selected(1, Atom::Zero)
        .rational_approximation(0, &Atom::Zero, 1, 1, EvaluationLimits::default(), &scope())
        .unwrap();
    assert!(zero.expression().is_zero());
    assert!(
        selected(1, x.pow(-1))
            .rational_approximation(0, &Atom::Zero, 1, 1, EvaluationLimits::default(), &scope())
            .is_err()
    );
}

#[test]
fn analytical_native_relation_keeps_hypothesis_stopping_and_exact_algebraic_proof_separate() {
    crate::initialize().unwrap();
    let first = selected(0, Atom::num(2).sqrt());
    let second = selected(0, Atom::num(3).sqrt());
    let third = selected(0, Atom::num(2).sqrt() + Atom::num(3).sqrt());
    let request = RelationRequest {
        precision: 192,
        tolerance: RationalView {
            numerator: "1".into(),
            denominator: "100000000000000000000000000000000000000000000000000".into(),
        },
        max_iterations: 100,
        max_coefficient: 100,
    };
    let result = first
        .integer_relation(
            &[&second, &third],
            &request,
            EvaluationLimits::default(),
            &scope(),
        )
        .unwrap();
    let AnalyticalObservation::IntegerRelation(observation) = result.observation else {
        panic!("relation")
    };
    assert_eq!(observation.stop, RelationStop::Tolerance);
    assert_eq!(observation.meaning, RelationMeaning::ExactAlgebraicZero);
    assert_eq!(observation.algebraic_normal_form.unwrap().symbolica, "0");
    assert_eq!(observation.candidate.unwrap().len(), 3);
    let rounded = |value| selected(0, Atom::num(Float::with_val(128, value)));
    let [a, b, c] = [-32.0177, 3.1416, 2.7183].map(rounded);
    let request = RelationRequest {
        precision: 128,
        tolerance: RationalView {
            numerator: "1".into(),
            denominator: "10000".into(),
        },
        max_iterations: 100,
        max_coefficient: 100,
    };
    let result = a
        .integer_relation(&[&b, &c], &request, EvaluationLimits::default(), &scope())
        .unwrap();
    let AnalyticalObservation::IntegerRelation(observation) = result.observation else {
        panic!("relation")
    };
    assert_eq!(observation.meaning, RelationMeaning::NumericalHypothesis);
    assert!(observation.algebraic_normal_form.is_none());
    let exhausted = first
        .integer_relation(
            &[&second, &third],
            &RelationRequest {
                max_iterations: 1,
                ..request
            },
            EvaluationLimits::default(),
            &scope(),
        )
        .unwrap();
    let AnalyticalObservation::IntegerRelation(exhausted) = exhausted.observation else {
        panic!("relation")
    };
    assert!(matches!(
        exhausted.stop,
        RelationStop::IterationLimit | RelationStop::Tolerance
    ));
    use symbolica::domains::{
        float::{Float, Real},
        integer::{Integer, IntegerRelationError},
    };
    let mut polls = 0;
    let search = Integer::solve_integer_relation_with(
        &[
            Float::with_val(128, 2).sqrt(),
            Float::with_val(128, 3).sqrt(),
            Float::with_val(128, 5).sqrt(),
        ],
        Float::with_val(128, 1e-30),
        100,
        Some(Integer::from(100)),
        None,
        || {
            polls += 1;
            polls < 3
        },
    );
    assert_eq!(search, Err(IntegerRelationError::Interrupted));
    assert_eq!(polls, 3);
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
    assert!(
        matches!(source.series(0,&Atom::Zero,SeriesDepth::absolute(4),EvaluationLimits::default(),&scope()),Err(MathError::Domain{source_id:actual,..}) if actual==source_id)
    );
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

#[test]
fn analytical_printer_refuses_the_complete_view_before_returning_partial_text() {
    crate::initialize().unwrap();
    let atom = named_atom("long_native_namespace::long_original_physical_axis".into()).unwrap();
    let limits = EvaluationLimits {
        scratch_bytes: atom.as_view().get_byte_size() + 1,
        ..Default::default()
    };
    assert!(matches!(
        mathematical_view(&atom, limits, &scope()),
        Err(MathError::Limit("analytical presentation bytes"))
    ));
}

#[test]
fn analytical_float_residual_is_inconclusive_without_exact_normalization_panic() {
    crate::initialize().unwrap();
    let left = selected(0, Atom::num(0.1));
    let right = selected(0, Atom::num(0.1 + f64::EPSILON));
    let result = left
        .identity(&right, EvaluationLimits::default(), &scope())
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
fn analytical_identity_refuses_explosive_polynomial_expansion_before_native_normalization() {
    crate::initialize().unwrap();
    let x = crate::library::formal(0).unwrap();
    let original = selected(1, (&x + Atom::num(1)).pow(1024));
    let zero = selected(1, Atom::Zero);
    assert!(matches!(
        original.identity(
            &zero,
            EvaluationLimits {
                operations: 10_000,
                ..Default::default()
            },
            &scope()
        ),
        Err(MathError::Limit("analytical normalization terms"))
    ));
}
