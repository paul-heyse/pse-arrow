// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent analytic controls distinguish complete requested products from unused jets.
use super::*;

#[test]
fn requested_weighted_product_preserves_value_domain_without_materializing_unused_intermediate_second_jets()
 {
    crate::initialize().unwrap();
    let x = library::formal(0).unwrap();
    let u = library::formal(1).unwrap();
    let source_s = pse_ids::named_id(SemanticId::NIL, "original sqrt stage S");
    let source_t = pse_ids::named_id(SemanticId::NIL, "original composed stage T");
    let body = PreparedBody::new(
        1,
        3,
        vec![2],
        vec![
            Stage::Block {
                expressions: vec![x.sqrt()],
                outputs: vec![1],
                source: source_s,
            },
            Stage::Block {
                expressions: vec![u.pow(2) + x.pow(2)],
                outputs: vec![2],
                source: source_t,
            },
        ],
        DerivativeOrder::Second,
    )
    .unwrap();
    let cancel = pse_diagnostics::Cancellation::new(false);
    let support = body
        .prepare_support(&[0], &[0], DerivativeOrder::Second, &cancel)
        .unwrap();
    let selected = support
        .compile_product(
            ProductKind::SelectedPartials,
            Optimization::default(),
            EvaluationLimits::default(),
            &pse_kernels::ExecutionScope::new(cancel.clone(), None),
        )
        .unwrap();
    let mut selected = selected.worker();
    // On x>0, v=(sqrt(x))²+x²=x+x², hence v''=2 exactly. At this
    // finite positive x, u and u' are finite, while the unused u'' overflows.
    // This establishes requested-product validity, not equivalence to materializing
    // every intermediate Second jet of the original staged representation.
    let tiny: f64 = 1e-300;
    assert!(tiny.sqrt().is_finite());
    assert!((0.5 / tiny.sqrt()).is_finite());
    assert!(!(-0.25 / tiny.powf(1.5)).is_finite());
    assert!(
        matches!(selected.evaluate(&[tiny], DerivativeOrder::Second, &mut ProviderBindings::new(), &cancel),
        Err(MathError::Domain { source_id, .. }) if source_id == source_s)
    );
    for kind in [
        ProductKind::WeightedHessian,
        ProductKind::WeightedHessianAction,
    ] {
        let compiled = support
            .compile_product(
                kind,
                Optimization::default(),
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(cancel.clone(), None),
            )
            .unwrap();
        assert!(
            compiled
                .census()
                .find(|order| order.order == DerivativeOrder::Second)
                .unwrap()
                .direct_weighted
        );
        let mut worker = compiled.worker();
        let binding = if kind == ProductKind::WeightedHessian {
            ProductValues::WeightedHessian { weights: &[1.] }
        } else {
            ProductValues::WeightedHessianAction {
                weights: &[1.],
                direction: &[1.],
            }
        };
        let mut actual = [73.];
        worker
            .evaluate_product_into(
                &[tiny],
                binding,
                &mut ProviderBindings::new(),
                &cancel,
                &mut actual,
            )
            .unwrap();
        assert_eq!(
            actual,
            [2.],
            "the independent analytic requested product is finite"
        );
        let mut origins = Vec::new();
        compiled.visit_arithmetic_origins(DerivativeOrder::Second, |origin| origins.push(origin));
        assert!(origins.iter().any(|origin| origin.source == source_s));
        assert!(origins.iter().any(|origin| origin.source == source_t));
        assert!(
            origins
                .iter()
                .all(|origin| origin.check_order == DerivativeOrder::Value)
        );
        // Algebraic flattening gives a finite polynomial at x<0. It still cannot
        // erase the original sqrt value/domain refusal or overwrite prior output.
        assert!(
            matches!(worker.evaluate_product_into(&[-1.], binding, &mut ProviderBindings::new(), &cancel, &mut actual),
            Err(MathError::Domain { source_id, .. }) if source_id == source_s)
        );
        assert_eq!(actual, [2.]);
    }
    assert!(
        matches!(selected.evaluate(&[-1.], DerivativeOrder::Second, &mut ProviderBindings::new(), &cancel),
        Err(MathError::Domain { source_id, .. }) if source_id == source_s)
    );
}

#[test]
fn regional_products_provider_barrier_preserves_actual_provider_calls_and_refusal() {
    use pse_ids::ContentHash;
    use pse_kernels::{DerivativeSource, Port, ProviderError, ProviderShapes};
    use std::sync::atomic::AtomicUsize;
    #[derive(Debug)]
    struct Square {
        spec: ProviderSpec,
        calls: Arc<AtomicUsize>,
    }
    impl Provider for Square {
        fn supports(&self, _: &pse_kernels::ProviderDemand) -> bool {
            true
        }
        fn supports_linearization(&self) -> bool {
            true
        }
        fn completed_linearization(&self) -> Option<ContentHash> {
            Some(self.spec.revision)
        }
        fn fill(
            &mut self,
            inputs: &[f64],
            request: &PreparedProviderRequest,
            runtime: ProviderRuntime<'_>,
            output: pse_kernels::ProviderOutput<'_>,
            context: &EvaluationContext<'_>,
        ) -> Result<(), ProviderError> {
            request.check_context(context)?;
            self.calls.fetch_add(1, Ordering::Relaxed);
            if inputs[0] < 0. {
                return Err(ProviderError::Trial(
                    "original provider's selected domain".into(),
                ));
            }
            assert_eq!(request.outputs, [0]);
            output.values[0] = inputs[0] * inputs[0];
            output.jacobian.fill(2. * inputs[0]);
            output.hessians.fill(2.);
            use pse_kernels::ProviderDemand;
            match request.demand {
                ProviderDemand::JacobianAction { .. } => {
                    output.jacobian[0] *= runtime.directions[0]
                }
                ProviderDemand::TransposeAction { .. } => {
                    output.jacobian[0] *= runtime.cotangents[0]
                }
                ProviderDemand::WeightedSecondAction { .. } => {
                    output.jacobian[0] *= runtime.weights[0];
                    output.hessians[0] *= runtime.weights[0] * runtime.directions[0];
                }
                _ => {}
            }
            Ok(())
        }
    }
    crate::initialize().unwrap();
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let quantity = pse_quantity::standard::ids::quantity("neutral");
    let unit = registry.quantity_type(quantity).unwrap().canonical_unit;
    let port = |n| Port {
        id: SemanticId::from_bytes([n; 16]),
        quantity,
        unit,
    };
    let spec = ProviderSpec {
        id: SemanticId::from_bytes([201; 16]),
        revision: ContentHash::from_bytes([202; 32]),
        data: ContentHash::from_bytes([203; 32]),
        shapes: ProviderShapes::default(),
        derivative_source: DerivativeSource::Analytic,
        inputs: vec![port(204)],
        outputs: vec![port(205)],
        derivatives: DerivativeOrder::Second,
        smoothness: DerivativeOrder::Second,
    };
    let source = pse_ids::named_id(SemanticId::NIL, "weighted original provider barrier");
    let x = library::formal(0).unwrap();
    let u = library::formal(1).unwrap();
    let body = PreparedBody::new(
        1,
        4,
        vec![3],
        vec![
            Stage::Provider {
                spec: spec.clone(),
                partial: Vec::new(),
                inputs: vec![0],
                outputs: vec![1],
                source,
            },
            Stage::Block {
                expressions: vec![u + x.pow(2)],
                outputs: vec![2],
                source,
            },
            Stage::Block {
                expressions: vec![library::formal(2).unwrap().pow(2)],
                outputs: vec![3],
                source,
            },
        ],
        DerivativeOrder::Second,
    )
    .unwrap();
    let cancel = pse_diagnostics::Cancellation::new(false);
    let support = body
        .prepare_support(&[0], &[0], DerivativeOrder::Second, &cancel)
        .unwrap();
    for kind in [
        ProductKind::WeightedHessian,
        ProductKind::WeightedHessianAction,
    ] {
        let compiled = support
            .compile_product(
                kind,
                Optimization::default(),
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(cancel.clone(), None),
            )
            .unwrap();
        let census = compiled
            .census()
            .find(|order| order.order == DerivativeOrder::Second)
            .unwrap();
        assert!(!census.direct_weighted);
        assert_eq!(
            census.provider_stages,
            if kind == ProductKind::WeightedHessianAction {
                3
            } else {
                1
            }
        );
        let calls = Arc::new(AtomicUsize::new(0));
        let provider = pse_kernels::ProviderBinding::new(
            spec.clone(),
            Box::new(Square {
                spec: spec.clone(),
                calls: calls.clone(),
            }),
        );
        let mut providers: ProviderBindings = ProviderBindings::from([(spec.key(), provider)]);
        let binding = if kind == ProductKind::WeightedHessian {
            ProductValues::WeightedHessian { weights: &[2.] }
        } else {
            ProductValues::WeightedHessianAction {
                weights: &[2.],
                direction: &[3.],
            }
        };
        let mut worker = compiled.worker();
        let mut actual = [97.];
        worker
            .evaluate_product_into(&[2.], binding, &mut providers, &cancel, &mut actual)
            .unwrap();
        // u=x², t=u+x², v=t²=4x⁴: at x=2, H=192; 2H=384 and 2H·3=1152.
        assert_eq!(
            actual,
            if kind == ProductKind::WeightedHessian {
                [384.]
            } else {
                [1152.]
            }
        );
        assert!(calls.load(Ordering::Relaxed) > 0);
        let before = calls.load(Ordering::Relaxed);
        let complete = actual;
        assert!(
            matches!(worker.evaluate_product_into(&[-2.], binding, &mut providers, &cancel, &mut actual),
            Err(MathError::Provider { source_id, .. }) if source_id == source)
        );
        assert!(
            calls.load(Ordering::Relaxed) > before,
            "the host provider barrier remains actual scientific work"
        );
        assert_eq!(actual, complete);
    }

    // The same actual provider feeds joint forward regions. These checks use an
    // independent analytic matrix; no callback supplies the final chain terms.
    let providers = || {
        let calls = Arc::new(AtomicUsize::new(0));
        (ProviderBindings::from([(spec.key(), pse_kernels::ProviderBinding::new(spec.clone(), Box::new(Square { spec: spec.clone(), calls: calls.clone() })))]), calls)
    };
    let scope = pse_kernels::ExecutionScope::new(cancel.clone(), None);
    let selected = support.compile(Optimization::default(), EvaluationLimits::default(), &scope).unwrap();
    assert_eq!(selected.census[DerivativeOrder::Second].unwrap().arithmetic_blocks, 1);
    let (mut bindings, calls) = providers();
    let result = selected.worker().evaluate(&[2.], DerivativeOrder::Second, &mut bindings, &cancel).unwrap();
    assert_eq!(result.values, [64.]);
    assert_eq!(result.jacobian, [128.]);
    assert_eq!(result.hessians, [192.]);
    assert!(calls.load(Ordering::Relaxed) > 0);
    let first = body.prepare_support(&[0], &[0], DerivativeOrder::First, &cancel).unwrap();
    for kind in [ProductKind::JacobianAction, ProductKind::TransposeJacobianAction] {
        let compiled = first.compile_product(kind, Optimization::default(), EvaluationLimits::default(), &scope).unwrap();
        if kind == ProductKind::JacobianAction { assert_eq!(compiled.census[DerivativeOrder::First].unwrap().arithmetic_blocks, 1); }
        let (mut bindings, calls) = providers();
        let product = if kind == ProductKind::JacobianAction { ProductValues::JacobianAction { direction: &[3.] } } else { ProductValues::TransposeJacobianAction { weights: &[2.] } };
        let mut actual = [97.];
        let mut worker = compiled.worker();
        worker.evaluate_product_into(&[2.], product, &mut bindings, &cancel, &mut actual).unwrap();
        assert_eq!(actual, if kind == ProductKind::JacobianAction { [384.] } else { [256.] });
        assert!(calls.load(Ordering::Relaxed) > 0);
        let complete = actual;
        assert!(matches!(worker.evaluate_product_into(&[-2.], product, &mut bindings, &cancel, &mut actual), Err(MathError::Provider { source_id, .. }) if source_id == source));
        assert_eq!(actual, complete);
    }
}

#[test]
fn regional_products_branch_barrier_keeps_actual_arm_and_original_domain_source() {
    crate::initialize().unwrap();
    let x = library::formal(0).unwrap();
    let square_source = pse_ids::named_id(SemanticId::NIL, "selected square branch");
    let root_source = pse_ids::named_id(SemanticId::NIL, "selected sqrt branch");
    let body = PreparedBody::new(
        2,
        6,
        vec![5],
        vec![
            Stage::Block {
                expressions: vec![Atom::num(0)],
                outputs: vec![2],
                source: SemanticId::NIL,
            },
            Stage::Branch {
                continuity: DerivativeOrder::Value,
                comparison: Comparison::Lt,
                left: 2,
                right: 1,
                then: vec![Stage::Block {
                    expressions: vec![x.pow(2)],
                    outputs: vec![3],
                    source: square_source,
                }],
                otherwise: vec![Stage::Block {
                    expressions: vec![x.sqrt()],
                    outputs: vec![3],
                    source: root_source,
                }],
            },
            Stage::Block { expressions: vec![library::formal(3).unwrap() + x.pow(2)], outputs: vec![4], source: SemanticId::NIL },
            Stage::Block { expressions: vec![library::formal(4).unwrap().pow(2)], outputs: vec![5], source: SemanticId::NIL },
        ],
        DerivativeOrder::Second,
    )
    .unwrap();
    let cancel = pse_diagnostics::Cancellation::new(false);
    let support = body
        .prepare_support(&[0], &[0], DerivativeOrder::Second, &cancel)
        .unwrap();
    for kind in [
        ProductKind::WeightedHessian,
        ProductKind::WeightedHessianAction,
    ] {
        let compiled = support
            .compile_product(
                kind,
                Optimization::default(),
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(cancel.clone(), None),
            )
            .unwrap();
        let census = compiled
            .census()
            .find(|order| order.order == DerivativeOrder::Second)
            .unwrap();
        assert!(!census.direct_weighted);
        assert_eq!(census.control_stages, 1);
        let mut origins = Vec::new();
        compiled.visit_arithmetic_origins(DerivativeOrder::Second, |origin| origins.push(origin));
        assert!(origins.iter().any(|origin| origin.source == square_source));
        assert!(origins.iter().any(|origin| origin.source == root_source));
        let binding = if kind == ProductKind::WeightedHessian {
            ProductValues::WeightedHessian { weights: &[2.] }
        } else {
            ProductValues::WeightedHessianAction {
                weights: &[2.],
                direction: &[3.],
            }
        };
        let mut worker = compiled.worker();
        let mut actual = [79.];
        worker
            .evaluate_product_into(
                &[-1., 1.],
                binding,
                &mut ProviderBindings::new(),
                &cancel,
                &mut actual,
            )
            .unwrap();
        assert_eq!(
            actual,
            if kind == ProductKind::WeightedHessian {
                [96.]
            } else {
                [288.]
            },
            "the unselected sqrt arm is not evaluated at its invalid input"
        );
        worker
            .evaluate_product_into(
                &[4., -1.],
                binding,
                &mut ProviderBindings::new(),
                &cancel,
                &mut actual,
            )
            .unwrap();
        assert_eq!(
            actual,
            if kind == ProductKind::WeightedHessian {
                [414.]
            } else {
                [1242.]
            },
            "v=(sqrt(x)+x²)² gives v''(4)=207, weighted by 2 and the requested direction"
        );
        let complete = actual;
        assert!(
            matches!(worker.evaluate_product_into(&[-1., -1.], binding, &mut ProviderBindings::new(), &cancel, &mut actual),
            Err(MathError::Domain { source_id, .. }) if source_id == root_source)
        );
        assert_eq!(actual, complete);
    }

    let scope = pse_kernels::ExecutionScope::new(cancel.clone(), None);
    let selected = support.compile(Optimization::default(), EvaluationLimits::default(), &scope).unwrap();
    for (point, value, first, second) in [([-1., 1.], 4., -16., 48.), ([4., -1.], 324., 297., 207.)] {
        let result = selected.worker().evaluate(&point, DerivativeOrder::Second, &mut ProviderBindings::new(), &cancel).unwrap();
        assert_eq!(result.values, [value]);
        assert_eq!(result.jacobian, [first]);
        assert_eq!(result.hessians, [second]);
    }
    let first = body.prepare_support(&[0], &[0], DerivativeOrder::First, &cancel).unwrap();
    {
        let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
        let operation = hub.start("reverse source preflight refusal");
        let context = operation.context();
        let _entered = context.enter();
        assert!(matches!(first.compile_product(ProductKind::TransposeJacobianAction, Optimization::default(), EvaluationLimits { operations: 1, ..Default::default() }, &scope), Err(MathError::Limit(_))));
        assert!(!context.reader().snapshot(std::time::Duration::from_secs(60)).metrics.iter().any(|metric| metric.name == "math.rational_representation_inspections"), "reverse local source limits refuse before any forward evaluator is built");
    }
    for kind in [ProductKind::JacobianAction, ProductKind::TransposeJacobianAction] {
        let compiled = first.compile_product(kind, Optimization::default(), EvaluationLimits::default(), &scope).unwrap();
        let product = if kind == ProductKind::JacobianAction { ProductValues::JacobianAction { direction: &[3.] } } else { ProductValues::TransposeJacobianAction { weights: &[2.] } };
        let mut actual = [79.];
        let mut worker = compiled.worker();
        for (point, expected) in [([-1., 1.], -16.), ([4., -1.], 297.)] {
            worker.evaluate_product_into(&point, product, &mut ProviderBindings::new(), &cancel, &mut actual).unwrap();
            assert_eq!(actual, [expected * if kind == ProductKind::JacobianAction { 3. } else { 2. }]);
        }
        let complete = actual;
        assert!(matches!(worker.evaluate_product_into(&[-1., -1.], product, &mut ProviderBindings::new(), &cancel, &mut actual), Err(MathError::Domain { source_id, .. }) if source_id == root_source));
        assert_eq!(actual, complete);
    }
}

#[test]
fn transpose_first_keeps_second_only_guards_inactive() {
    crate::initialize().unwrap();
    let source = SemanticId::from_bytes([83; 16]);
    let x = library::formal(0).unwrap();
    let body = PreparedBody::new(
        1,
        2,
        vec![1],
        vec![
            Stage::Require {
                argument: 0,
                condition: Condition::Positive,
                order: DerivativeOrder::Second,
                source,
                lineage: None,
            },
            Stage::Block {
                expressions: vec![&x * &x],
                outputs: vec![1],
                source,
            },
        ],
        DerivativeOrder::Second,
    )
    .unwrap();
    let stop = pse_diagnostics::Cancellation::new(false);
    let scope = pse_kernels::ExecutionScope::new(stop.clone(), None);
    let first = body
        .prepare_support(&[0], &[0], DerivativeOrder::First, &stop)
        .unwrap();
    let ordinary = first
        .compile_product(
            ProductKind::SelectedPartials,
            Optimization::default(),
            EvaluationLimits::default(),
            &scope,
        )
        .unwrap();
    assert_eq!(
        ordinary
            .worker()
            .evaluate(
                &[-1.],
                DerivativeOrder::First,
                &mut ProviderBindings::new(),
                &stop
            )
            .unwrap()
            .jacobian,
        [-2.]
    );
    let transpose = first
        .compile_product(
            ProductKind::TransposeJacobianAction,
            Optimization::default(),
            EvaluationLimits::default(),
            &scope,
        )
        .unwrap();
    let mut output = [91.];
    transpose
        .worker()
        .evaluate_product_into(
            &[-1.],
            ProductValues::TransposeJacobianAction { weights: &[3.] },
            &mut ProviderBindings::new(),
            &stop,
            &mut output,
        )
        .unwrap();
    assert_eq!(output, [-6.]);
    let second = body
        .prepare_support(&[0], &[0], DerivativeOrder::Second, &stop)
        .unwrap();
    let hessian = second
        .compile_product(
            ProductKind::WeightedHessian,
            Optimization::default(),
            EvaluationLimits::default(),
            &scope,
        )
        .unwrap();
    assert!(
        matches!(hessian.worker().evaluate_product_into(&[-1.], ProductValues::WeightedHessian { weights: &[0.] },
        &mut ProviderBindings::new(), &stop, &mut output), Err(MathError::Domain { source_id, .. }) if source_id == source)
    );
    assert_eq!(output, [-6.]);
}

#[test]
fn sparse_flat_weighted_actions_match_analytic_and_entry_products_in_selected_axis_order() {
    crate::initialize().unwrap();
    let n = 5;
    let x = (0..n)
        .map(|i| library::formal(i).unwrap())
        .collect::<Vec<_>>();
    let point = [0.4, -0.3, 0.6, 0.8, -0.5];
    let axes = [4, 0, 2];
    let direction = [0.7, -0.2, 0.3];
    let control = pse_diagnostics::Cancellation::default();
    let scope = pse_kernels::ExecutionScope::new(control.clone(), None);
    for shape in ["diagonal", "banded", "disconnected"] {
        let expression = match shape {
            "diagonal" => (0..n).fold(Atom::num(0), |sum, i| {
                sum + Atom::num((i + 1) as i64) * x[i].pow(3)
            }),
            "banded" => (0..n - 1).fold(Atom::num(0), |sum, i| sum + x[i].pow(2) * &x[i + 1]),
            _ => x[0].pow(2) * &x[1] + &x[3] * x[4].pow(2),
        };
        let mut hessian = vec![vec![0.; n]; n];
        match shape {
            "diagonal" => {
                for i in 0..n {
                    hessian[i][i] = 6. * (i + 1) as f64 * point[i];
                }
            }
            "banded" => {
                for i in 0..n - 1 {
                    hessian[i][i] += 2. * point[i + 1];
                    hessian[i][i + 1] = 2. * point[i];
                    hessian[i + 1][i] = 2. * point[i];
                }
            }
            _ => {
                hessian[0][0] = 2. * point[1];
                hessian[0][1] = 2. * point[0];
                hessian[1][0] = 2. * point[0];
                hessian[4][4] = 2. * point[3];
                hessian[3][4] = 2. * point[4];
                hessian[4][3] = 2. * point[4];
            }
        }
        let body = PreparedBody::new(
            n,
            n + 2,
            vec![n, n + 1],
            vec![Stage::Block {
                expressions: vec![expression.clone(), -expression + x[2].pow(2)],
                outputs: vec![n, n + 1],
                source: SemanticId::from_bytes([102; 16]),
            }],
            DerivativeOrder::Second,
        )
        .unwrap();
        let support = body
            .prepare_support(&[1, 0], &axes, DerivativeOrder::Second, &control.flag())
            .unwrap();
        let compile = |kind| {
            support
                .compile_product(
                    kind,
                    Optimization::default(),
                    EvaluationLimits::default(),
                    &scope,
                )
                .unwrap()
        };
        let action = compile(ProductKind::WeightedHessianAction);
        let entries = compile(ProductKind::WeightedHessian);
        assert!(
            action.product_pairs().is_empty(),
            "{shape}: actions retain no second-entry population"
        );
        let census = action
            .census()
            .find(|c| c.order == DerivativeOrder::Second)
            .unwrap();
        assert!(census.direct_weighted);
        assert_eq!(census.derivative_components, 2);
        assert_eq!(census.second_pairs, 0);
        let mut action = action.worker();
        let mut matrix = entries.worker();
        let mut out = vec![73.; axes.len()];
        for weights in [[2., -0.5], [1., 1.], [0., 0.]] {
            action
                .evaluate_product_into(
                    &point,
                    ProductValues::WeightedHessianAction {
                        weights: &weights,
                        direction: &direction,
                    },
                    &mut ProviderBindings::new(),
                    &control.flag(),
                    &mut out,
                )
                .unwrap();
            let expected = axes
                .iter()
                .map(|&row| {
                    axes.iter()
                        .zip(direction)
                        .map(|(&col, seed)| {
                            ((weights[1] - weights[0]) * hessian[row][col]
                                + if row == 2 && col == 2 {
                                    2. * weights[0]
                                } else {
                                    0.
                                })
                                * seed
                        })
                        .sum::<f64>()
                })
                .collect::<Vec<_>>();
            for (actual, expected) in out.iter().zip(&expected) {
                assert!(
                    (actual - expected).abs() < 1e-12,
                    "{shape}: {out:?} vs {expected:?}"
                );
            }
            let mut coefficients = vec![f64::NAN; entries.product_len()];
            matrix
                .evaluate_product_into(
                    &point,
                    ProductValues::WeightedHessian { weights: &weights },
                    &mut ProviderBindings::new(),
                    &control.flag(),
                    &mut coefficients,
                )
                .unwrap();
            let mut contracted = vec![0.; axes.len()];
            for (&(i, j), value) in entries.product_pairs().iter().zip(coefficients) {
                contracted[i] += value * direction[j];
                if i != j {
                    contracted[j] += value * direction[i];
                }
            }
            for (actual, expected) in out.iter().zip(contracted) {
                assert!((actual - expected).abs() < 1e-12);
            }
        }
        let intact = out.clone();
        let mut nonfinite = point;
        nonfinite[0] = f64::MAX;
        assert!(
            action
                .evaluate_product_into(
                    &nonfinite,
                    ProductValues::WeightedHessianAction {
                        weights: &[0., 0.],
                        direction: &direction
                    },
                    &mut ProviderBindings::new(),
                    &control.flag(),
                    &mut out
                )
                .is_err()
        );
        assert_eq!(out, intact);
        control.cancel();
        assert!(
            action
                .evaluate_product_into(
                    &point,
                    ProductValues::WeightedHessianAction {
                        weights: &[1., 1.],
                        direction: &direction
                    },
                    &mut ProviderBindings::new(),
                    &control.flag(),
                    &mut out
                )
                .is_err()
        );
        assert_eq!(out, intact);
        control.flag().store(false, Ordering::Release);
    }
}
