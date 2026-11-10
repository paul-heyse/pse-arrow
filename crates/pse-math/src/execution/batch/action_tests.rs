// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent action oracles over actual native x4 product programs.
use super::tests::{body, cancellation_body, scope, source};
use super::*;
use pse_diagnostics::observation::{ObservationHub, Policy, Value};

fn native() -> Optimization {
    Optimization {
        evaluator: library::Evaluator::CompiledX4 {
            module_bytes: 16 << 20,
        },
        ..Default::default()
    }
}
fn expected(
    kind: ProductKind,
    pairs: &[(usize, usize)],
    inputs: [f64; 2],
    weights: [f64; 2],
    direction: [f64; 2],
) -> Vec<f64> {
    let [x, y] = inputs;
    let [a, b] = weights;
    let [dy, dx] = direction;
    // Outputs [xy,x²+3y], axes [y,x]. Weighted H=[[0,a],[a,2b]].
    match kind {
        ProductKind::JacobianAction => vec![x * dy + y * dx, 3. * dy + 2. * x * dx],
        ProductKind::TransposeJacobianAction => vec![a * x + 3. * b, a * y + 2. * b * x],
        ProductKind::WeightedHessianAction => vec![a * dx, a * dy + 2. * b * dx],
        ProductKind::WeightedHessian => pairs
            .iter()
            .map(|&(i, j)| match (i, j) {
                (0, 0) => 0.,
                (0, 1) => a,
                (1, 1) => 2. * b,
                _ => panic!("unexpected exact pair"),
            })
            .collect(),
        _ => unreachable!(),
    }
}
fn observed(
    hub: &ObservationHub,
    operation: pse_diagnostics::observation::OperationReader,
    name: &str,
) -> bool {
    operation
        .snapshot(std::time::Duration::from_secs(30))
        .metrics
        .iter()
        .any(|metric| {
            metric.name == name && matches!(metric.cumulative,Value::Count(count) if count>0)
        })
        || hub.list(0, 128).operations.into_iter().any(|entry| {
            hub.reader(entry.id).is_some_and(|reader| {
                reader
                    .snapshot(std::time::Duration::from_secs(30))
                    .metrics
                    .iter()
                    .any(|metric| {
                        metric.name == name
                            && matches!(metric.cumulative,Value::Count(count) if count>0)
                    })
            })
        })
}
fn evaluate_changed_runtime_bindings(kind: ProductKind) {
    let order = if matches!(
        kind,
        ProductKind::JacobianAction | ProductKind::TransposeJacobianAction
    ) {
        DerivativeOrder::First
    } else {
        DerivativeOrder::Second
    };
    let cancel = Arc::new(AtomicBool::new(false));
    // The original positive-x guard also selects the admitted direct route
    // for weighted actions; it remains authoritative for every row.
    let support = body(true)
        .prepare_support(&[1, 0], &[1, 0], order, &cancel)
        .unwrap();
    let compiled = Arc::new(
        support
            .compile_product(kind, native(), EvaluationLimits::default(), &scope())
            .unwrap(),
    );
    assert!(compiled.template.programs[DerivativeOrder::Second].is_none());
    if kind != ProductKind::JacobianAction {
        assert!(compiled.template.direct_product.is_some());
    }
    let prepared = PreparedBatch::new(compiled, order).unwrap();
    let mut worker = prepared.worker();
    for pass in [0_u32, 1] {
        let mut inputs = (0_u32..17)
            .flat_map(|row| [2. + f64::from(row), 5. + f64::from(row)])
            .collect::<Vec<_>>();
        let weights = (0_u32..17)
            .flat_map(|row| [f64::from(row + pass), 2. - f64::from(pass)])
            .collect::<Vec<_>>();
        let directions = (0_u32..17)
            .flat_map(|row| [3. - f64::from(row), f64::from(row + pass + 1)])
            .collect::<Vec<_>>();
        let bindings = match kind {
            ProductKind::JacobianAction => BatchBindings::JacobianAction {
                directions: &directions,
            },
            ProductKind::TransposeJacobianAction => {
                BatchBindings::TransposeJacobianAction { weights: &weights }
            }
            ProductKind::WeightedHessian => BatchBindings::WeightedHessian { weights: &weights },
            ProductKind::WeightedHessianAction => BatchBindings::WeightedHessianAction {
                weights: &weights,
                directions: &directions,
            },
            _ => unreachable!(),
        };
        for count in [0, 1, 3, 4, 5, 9, 17] {
            let sliced = match bindings {
                BatchBindings::JacobianAction { directions } => BatchBindings::JacobianAction {
                    directions: &directions[..2 * count],
                },
                BatchBindings::TransposeJacobianAction { weights } => {
                    BatchBindings::TransposeJacobianAction {
                        weights: &weights[..2 * count],
                    }
                }
                BatchBindings::WeightedHessian { weights } => BatchBindings::WeightedHessian {
                    weights: &weights[..2 * count],
                },
                BatchBindings::WeightedHessianAction {
                    weights,
                    directions,
                } => BatchBindings::WeightedHessianAction {
                    weights: &weights[..2 * count],
                    directions: &directions[..2 * count],
                },
                _ => unreachable!(),
            };
            let ids = vec![source(); count];
            let result = worker
                .evaluate(
                    BatchRequest {
                        layout: prepared.layout(),
                        points: &ids,
                        inputs: &inputs[..2 * count],
                        bindings: sliced,
                    },
                    &cancel,
                )
                .unwrap();
            for row in 0..count {
                let x = inputs[2 * row];
                let y = inputs[2 * row + 1];
                let a = weights[2 * row];
                let b = weights[2 * row + 1];
                let dy = directions[2 * row];
                let dx = directions[2 * row + 1];
                let expected = expected(kind, prepared.layout().pairs(), [x, y], [a, b], [dy, dx]);
                assert_eq!(
                    result.row(row),
                    Some(expected.as_slice()),
                    "{kind:?},pass{pass},row{row}"
                );
            }
        }
        inputs[2] = -1.;
        let result = worker
            .evaluate(
                BatchRequest {
                    layout: prepared.layout(),
                    points: &[source(); 17],
                    inputs: &inputs,
                    bindings,
                },
                &cancel,
            )
            .unwrap();
        assert!(
            matches!(result.points()[1].outcome,PointOutcome::Failed(MathError::Domain{source_id,..}) if source_id==source())
        );
        assert!(result.row(1).is_none());
        for row in (0..17).filter(|&row| row != 1) {
            let expected = expected(
                kind,
                prepared.layout().pairs(),
                [inputs[2 * row], inputs[2 * row + 1]],
                [weights[2 * row], weights[2 * row + 1]],
                [directions[2 * row], directions[2 * row + 1]],
            );
            assert_eq!(
                result.row(row),
                Some(expected.as_slice()),
                "original row map after refusal: {kind:?},row{row}"
            );
        }
    }
    assert!(worker.worker.numeric.programs[DerivativeOrder::Second].is_none());
}
fn check_changed_runtime_bindings(kind: ProductKind) {
    let hub = ObservationHub::new(Policy::default());
    let operation = hub.start("native action populations");
    let context = operation.context();
    let _entered = context.enter();
    evaluate_changed_runtime_bindings(kind);
    assert!(observed(
        &hub,
        operation.reader(),
        "math.native_x4_packed_rows"
    ));
    if kind != ProductKind::JacobianAction {
        assert!(observed(
            &hub,
            operation.reader(),
            "math.native_x4_direct_action_rows"
        ));
    }
    assert!(observed(
        &hub,
        operation.reader(),
        "math.native_x4_scalar_tail_rows"
    ));
}
#[test]
fn ordered_x4_action_profiles_compose_in_one_original_operation() {
    let hub = ObservationHub::new(Policy::default());
    let operation = hub.start("native action populations");
    let context = operation.context();
    let _entered = context.enter();
    for kind in [
        ProductKind::JacobianAction,
        ProductKind::TransposeJacobianAction,
        ProductKind::WeightedHessian,
        ProductKind::WeightedHessianAction,
    ] {
        evaluate_changed_runtime_bindings(kind);
    }
    for name in [
        "math.native_x4_packed_rows",
        "math.native_x4_direct_action_rows",
        "math.native_x4_scalar_tail_rows",
    ] {
        assert!(observed(&hub, operation.reader(), name));
    }
}

#[test]
fn ordered_x4_action_jacobian_binds_changed_runtime_directions_and_reordered_axes() {
    check_changed_runtime_bindings(ProductKind::JacobianAction);
}
#[test]
fn ordered_x4_action_transpose_binds_changed_runtime_weights_and_reordered_axes() {
    check_changed_runtime_bindings(ProductKind::TransposeJacobianAction);
}
#[test]
fn ordered_x4_action_weighted_hessian_binds_changed_runtime_weights_and_reordered_axes() {
    check_changed_runtime_bindings(ProductKind::WeightedHessian);
}
#[test]
fn ordered_x4_action_weighted_hessian_action_binds_changed_runtime_directions_weights_and_reordered_axes()
 {
    check_changed_runtime_bindings(ProductKind::WeightedHessianAction);
}

#[test]
fn ordered_x4_direct_weighted_action_keeps_finite_cancellation_and_segmented_stop() {
    let cancel = Arc::new(AtomicBool::new(false));
    let body = cancellation_body();
    let support = body
        .prepare_support(&[0], &[0], DerivativeOrder::Second, &cancel)
        .unwrap();
    let compiled = Arc::new(
        support
            .compile_product(
                ProductKind::WeightedHessianAction,
                native(),
                EvaluationLimits::default(),
                &scope(),
            )
            .unwrap(),
    );
    assert!(compiled.template.direct_product.is_some());
    assert!(compiled.template.programs[DerivativeOrder::Second].is_none());
    let prepared = PreparedBatch::new(compiled, DerivativeOrder::Second).unwrap();
    let mut worker = prepared.worker();
    for count in [0, 1, 3, 4, 5, 9, 260] {
        let ids = vec![source(); count];
        let inputs = vec![1e-300; count];
        let weights = (0..count)
            .map(|row| f64::from(u32::try_from(row % 5).unwrap()) - 2.)
            .collect::<Vec<_>>();
        let directions = (0..count)
            .map(|row| 1. + f64::from(u32::try_from(row % 3).unwrap()))
            .collect::<Vec<_>>();
        let result = worker
            .evaluate(
                BatchRequest {
                    layout: prepared.layout(),
                    points: &ids,
                    inputs: &inputs,
                    bindings: BatchBindings::WeightedHessianAction {
                        weights: &weights,
                        directions: &directions,
                    },
                },
                &cancel,
            )
            .unwrap();
        for row in 0..count {
            assert_eq!(
                result.row(row),
                Some([2. * weights[row] * directions[row]].as_slice())
            );
        }
    }
    let mut refused = vec![1e-300; 5];
    refused[1] = -1.;
    let result = worker
        .evaluate(
            BatchRequest {
                layout: prepared.layout(),
                points: &[source(); 5],
                inputs: &refused,
                bindings: BatchBindings::WeightedHessianAction {
                    weights: &[1., 0., 2., -1., 3.],
                    directions: &[3.; 5],
                },
            },
            &cancel,
        )
        .unwrap();
    assert!(
        matches!(result.points()[1].outcome,PointOutcome::Failed(MathError::Domain{source_id,..}) if source_id==source())
    );
    assert!(
        result.row(1).is_none(),
        "zero weight cannot prune the original source guard"
    );
    for (row, expected) in [(0, 6.), (2, 12.), (3, -6.), (4, 18.)] {
        assert_eq!(result.row(row), Some([expected].as_slice()));
    }
    let ids = vec![source(); 260];
    let inputs = vec![1e-300; 260];
    let weights = vec![1.; 260];
    let directions = vec![3.; 260];
    let result = worker
        .evaluate_observing(
            BatchRequest {
                layout: prepared.layout(),
                points: &ids,
                inputs: &inputs,
                bindings: BatchBindings::WeightedHessianAction {
                    weights: &weights,
                    directions: &directions,
                },
            },
            &cancel,
            |row| {
                if row == 255 {
                    cancel.store(true, Ordering::Release)
                }
            },
        )
        .unwrap();
    assert!(result.canceled());
    assert!(
        result.points()[..256]
            .iter()
            .all(|point| matches!(point.outcome, PointOutcome::Succeeded))
    );
    assert!(
        result.points()[256..]
            .iter()
            .all(|point| matches!(point.outcome, PointOutcome::NotExecuted))
    );
    assert!(worker.worker.numeric.programs[DerivativeOrder::Second].is_none());
    let original = scope();
    let mut scoped = prepared.worker_scoped(original.clone());
    original.cancellation_control().cancel();
    let result = scoped
        .evaluate(
            BatchRequest {
                layout: prepared.layout(),
                points: &ids,
                inputs: &inputs,
                bindings: BatchBindings::WeightedHessianAction {
                    weights: &weights,
                    directions: &directions,
                },
            },
            original.cancellation(),
        )
        .unwrap();
    assert!(result.canceled());
    assert!(
        result
            .points()
            .iter()
            .all(|point| matches!(point.outcome, PointOutcome::NotExecuted))
    );
}

#[test]
fn ordered_x4_action_profiles_refuse_reverse_and_lazy_branches_before_effects() {
    let cancel = Arc::new(AtomicBool::new(false));
    let support = body(false)
        .prepare_support(&[1, 0], &[1, 0], DerivativeOrder::First, &cancel)
        .unwrap();
    let compiled = Arc::new(
        support
            .compile_product(
                ProductKind::TransposeJacobianAction,
                native(),
                EvaluationLimits::default(),
                &scope(),
            )
            .unwrap(),
    );
    assert!(compiled.template.reverse_product.is_some());
    assert!(
        matches!(PreparedBatch::new(compiled,DerivativeOrder::First),Err(MathError::Contract(message)) if message=="x4 batch reverse products are unsupported")
    );
    let x = library::formal(0).unwrap();
    let block = |expression, output| Stage::Block {
        expressions: vec![expression],
        outputs: vec![output],
        source: source(),
    };
    let branch = PreparedBody::new(
        1,
        3,
        vec![1],
        vec![
            block(Atom::num(0), 2),
            Stage::Branch {
                continuity: DerivativeOrder::Value,
                comparison: Comparison::Lt,
                left: 2,
                right: 0,
                then: vec![block(x.clone(), 1)],
                otherwise: vec![block(-x, 1)],
            },
        ],
        DerivativeOrder::Value,
    )
    .unwrap();
    let compiled = Arc::new(
        branch
            .compile(
                &[0],
                &[],
                DerivativeOrder::Value,
                native(),
                EvaluationLimits::default(),
                &scope(),
            )
            .unwrap(),
    );
    assert!(
        matches!(PreparedBatch::new(compiled.clone(),DerivativeOrder::Value),Err(MathError::Contract(message)) if message=="packed batch requires pure branch-free stages")
    );
    // No top-level Block: the only packed arithmetic is inside the numerical
    // Branch. The reusable owner must use the same eligibility gate.
    let branch_only = PreparedBody::new(
        2,
        3,
        vec![2],
        vec![Stage::Branch {
            continuity: DerivativeOrder::Value,
            comparison: Comparison::Lt,
            left: 0,
            right: 1,
            then: vec![block(Atom::num(3), 2)],
            otherwise: vec![block(Atom::num(4), 2)],
        }],
        DerivativeOrder::Value,
    )
    .unwrap();
    let compiled = branch_only
        .compile(
            &[0],
            &[],
            DerivativeOrder::Value,
            native(),
            EvaluationLimits::default(),
            &scope(),
        )
        .unwrap();
    let backing = numerical_batch_backing(&compiled.template, 4, DerivativeOrder::Value).unwrap();
    let mut worker = CheckedBatchExecution::from_frame(
        compiled.worker().numeric,
        4,
        DerivativeOrder::Value,
        backing,
    )
    .unwrap();
    let mut received = 0;
    assert!(
        matches!(worker.evaluate_observing(4,&[1.,2.,2.,1.,3.,4.,4.,3.],|_,_|received+=1),Err(MathError::Contract(message)) if message=="packed batch requires pure branch-free stages")
    );
    assert_eq!(
        received, 0,
        "unsupported x4 control refuses before any row executes"
    );
}

#[test]
fn ordered_x4_scalar_scientific_envelope_keeps_lazy_log_and_original_form_failure() {
    crate::initialize().unwrap();
    let lineage = Arc::new(pse_model::diagnostic::ValidityLineage {
        layer: pse_model::generated::enums::ModelingValidityLayer::Form,
        source: source(),
        form: Some(source()),
        sets: vec![],
        variables: vec![],
        members: vec![],
    });
    let x = library::formal(0).unwrap();
    let block = |expression, output| Stage::Block {
        expressions: vec![expression],
        outputs: vec![output],
        source: source(),
    };
    // x>0 && ln(x)>0. For a negative x, evaluating the inactive ln arm
    // would produce an arithmetic domain error instead of this Form refusal.
    let body = PreparedBody::new(
        1,
        5,
        vec![4],
        vec![
            Stage::Domain {
                stages: vec![
                    block(Atom::num(0), 1),
                    Stage::Branch {
                        continuity: DerivativeOrder::Value,
                        comparison: Comparison::Lt,
                        left: 1,
                        right: 0,
                        then: vec![block(x.clone().log(), 2)],
                        otherwise: vec![block(Atom::num(0), 2)],
                    },
                ],
                argument: 2,
                token: 3,
                lineage,
            },
            block(&x * &x, 4),
        ],
        DerivativeOrder::Second,
    )
    .unwrap();
    let compiled = Arc::new(
        body.compile(
            &[0],
            &[0],
            DerivativeOrder::First,
            native(),
            EvaluationLimits::default(),
            &scope(),
        )
        .unwrap(),
    );
    assert!(compiled.pure_branch_free(DerivativeOrder::First));
    let prepared = PreparedBatch::new(compiled, DerivativeOrder::First).unwrap();
    let mut worker = prepared.worker();
    let values = [2., 3., -1., 4., 5., 6., 7., 8., 9.];
    let result = worker
        .evaluate(
            BatchRequest {
                layout: prepared.layout(),
                points: &[source(); 9],
                inputs: &values,
                bindings: BatchBindings::SelectedPartials,
            },
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert!(
        matches!(&result.points()[2].outcome,PointOutcome::Failed(MathError::Validity(lineage)) if lineage.layer==pse_model::generated::enums::ModelingValidityLayer::Form && lineage.source==source())
    );
    for (row, &x) in values.iter().enumerate().filter(|(row, _)| *row != 2) {
        assert_eq!(result.row(row), Some([x * x, 2. * x].as_slice()));
    }
}
