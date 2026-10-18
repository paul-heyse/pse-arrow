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
fn weighted_product_provider_barrier_preserves_actual_provider_calls_and_refusal() {
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
        3,
        vec![2],
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
        // v=2x², weighted by 2: H=8, and H·3=24, independently of callback output.
        assert_eq!(
            actual,
            if kind == ProductKind::WeightedHessian {
                [8.]
            } else {
                [24.]
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
}

#[test]
fn weighted_product_branch_barrier_keeps_actual_arm_and_original_domain_source() {
    crate::initialize().unwrap();
    let x = library::formal(0).unwrap();
    let square_source = pse_ids::named_id(SemanticId::NIL, "selected square branch");
    let root_source = pse_ids::named_id(SemanticId::NIL, "selected sqrt branch");
    let body = PreparedBody::new(
        2,
        4,
        vec![3],
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
                [4.]
            } else {
                [12.]
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
                [-1. / 16.]
            } else {
                [-3. / 16.]
            },
            "sqrt''(4)=-1/32, weighted by 2, with the explicitly requested direction"
        );
        let complete = actual;
        assert!(
            matches!(worker.evaluate_product_into(&[-1., -1.], binding, &mut ProviderBindings::new(), &cancel, &mut actual),
            Err(MathError::Domain { source_id, .. }) if source_id == root_source)
        );
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
