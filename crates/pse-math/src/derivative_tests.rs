// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use crate::{
    MathError,
    guarded::*,
    jets::EvaluationLimits,
    library::{self, Optimization},
};
use pse_ids::{ContentHash, SemanticId};
use pse_kernels::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};
use symbolica::atom::{Atom, AtomCore};
fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
fn compile(
    inputs: usize,
    slots: usize,
    stages: Vec<Stage>,
    outputs: Vec<usize>,
    order: DerivativeOrder,
) -> CompiledBody {
    let prepared =
        PreparedBody::new(inputs, slots, outputs, stages, DerivativeOrder::Second).unwrap();
    prepared
        .compile(
            &(0..prepared.output_count()).collect::<Vec<_>>(),
            &(0..inputs).collect::<Vec<_>>(),
            order,
            Optimization::default(),
            EvaluationLimits::default(),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
}
fn block(expressions: Vec<Atom>, outputs: Vec<usize>) -> Stage {
    Stage::Block {
        expressions,
        outputs,
        source: id(1),
    }
}

// A branch keeps an affine sum opaque without manufacturing a dense Hessian.
fn wide_opaque_stages(n: usize, nonlinear: bool) -> Vec<Stage> {
    let sum = (0..n).fold(Atom::num(0), |sum, i| sum + library::formal(i).unwrap());
    let opaque = library::formal(n + 2).unwrap();
    vec![
        block(vec![Atom::num(0)], vec![n + 1]),
        Stage::Branch {
            continuity: DerivativeOrder::Value,
            comparison: Comparison::Lt,
            left: n + 1,
            right: n,
            then: vec![block(vec![sum.clone()], vec![n + 2])],
            otherwise: vec![block(vec![sum + Atom::num(1)], vec![n + 2])],
        },
        block(
            vec![if nonlinear { &opaque * &opaque } else { opaque }],
            vec![n + 3],
        ),
    ]
}

#[test]
fn wide_opaque_affine_support_retains_sparse_derivatives() {
    crate::initialize().unwrap();
    let n = 300;
    let body = PreparedBody::new(
        n + 1,
        n + 4,
        vec![n + 3],
        wide_opaque_stages(n, false),
        DerivativeOrder::Second,
    )
    .unwrap();
    assert_eq!(
        crate::requested_test_support(&body).first[0],
        (0..n).collect()
    );
    assert!(crate::requested_test_support(&body).second[0].is_empty());
    let mut worker = body
        .compile(
            &[0],
            &[0, n - 1],
            DerivativeOrder::Second,
            Optimization::default(),
            EvaluationLimits::default(),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
        .worker();
    for guard in [-1.0, 1.0] {
        let mut inputs = vec![1.0; n + 1];
        inputs[n] = guard;
        let result = worker
            .evaluate(
                &inputs,
                DerivativeOrder::Second,
                &mut BTreeMap::new(),
                &Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
        assert_eq!(
            result.values,
            vec![n as f64 + if guard < 0.0 { 1.0 } else { 0.0 }]
        );
        assert_eq!(result.jacobian, vec![1.0, 1.0]);
        assert_eq!(result.hessians, vec![0.0; 4]);
    }
}

#[test]
fn shared_branch_snapshots_keep_distinct_alternatives_isolated() {
    crate::initialize().unwrap();
    let x = library::formal(0).unwrap();
    let y = library::formal(1).unwrap();
    let body = PreparedBody::new(
        3,
        5,
        vec![4],
        vec![
            block(vec![Atom::num(0)], vec![3]),
            Stage::Branch {
                continuity: DerivativeOrder::Value,
                comparison: Comparison::Lt,
                left: 3,
                right: 2,
                then: vec![block(vec![&x * &x], vec![4])],
                otherwise: vec![block(vec![&y * &y * &y], vec![4])],
            },
        ],
        DerivativeOrder::Second,
    )
    .unwrap();
    assert_eq!(
        crate::requested_test_support(&body).first[0],
        [0, 1].into_iter().collect()
    );
    assert_eq!(
        crate::requested_test_support(&body).second[0],
        [(0, 0), (1, 1)].into_iter().collect()
    );
    assert!(body.expression(0).is_none());
    let cancel = Arc::new(AtomicBool::new(false));
    let mut worker = body
        .compile(
            &[0],
            &[0, 1],
            DerivativeOrder::Second,
            Optimization::default(),
            EvaluationLimits::default(),
            &cancel,
        )
        .unwrap()
        .worker();
    for (guard, value, jacobian, hessian) in [
        (1.0, 4.0, vec![4.0, 0.0], vec![2.0, 0.0, 0.0, 0.0]),
        (-1.0, 27.0, vec![0.0, 27.0], vec![0.0, 0.0, 0.0, 18.0]),
    ] {
        let result = worker
            .evaluate(
                &[2.0, 3.0, guard],
                DerivativeOrder::Second,
                &mut BTreeMap::new(),
                &cancel,
            )
            .unwrap();
        assert_eq!(result.values, vec![value]);
        assert_eq!(result.jacobian, jacobian);
        assert_eq!(result.hessians, hessian);
    }
}

#[test]
fn separately_reconstructed_equal_branch_facts_remain_symbolic() {
    crate::initialize().unwrap();
    let x = library::formal(0).unwrap();
    let y = library::formal(1).unwrap();
    let expression = (&x + &y) * (&x + &y);
    let body = PreparedBody::new(
        3,
        5,
        vec![4],
        vec![
            block(vec![Atom::num(0)], vec![3]),
            Stage::Branch {
                continuity: DerivativeOrder::Value,
                comparison: Comparison::Lt,
                left: 3,
                right: 2,
                then: vec![Stage::Block {
                    expressions: vec![expression.clone()],
                    outputs: vec![4],
                    source: id(11),
                }],
                otherwise: vec![Stage::Block {
                    expressions: vec![(&y + &x) * (&y + &x)],
                    outputs: vec![4],
                    source: id(12),
                }],
            },
        ],
        DerivativeOrder::Second,
    )
    .unwrap();
    assert_eq!(body.expression(0), Some(&expression));
    assert_eq!(
        crate::requested_test_support(&body).first[0],
        [0, 1].into_iter().collect()
    );
    assert_eq!(
        crate::requested_test_support(&body).second[0],
        [(0, 0), (0, 1), (1, 1)].into_iter().collect()
    );
    let cancel = Arc::new(AtomicBool::new(false));
    let mut worker = body
        .compile(
            &[0],
            &[0, 1],
            DerivativeOrder::Second,
            Optimization::default(),
            EvaluationLimits::default(),
            &cancel,
        )
        .unwrap()
        .worker();
    for guard in [-1.0, 1.0] {
        let result = worker
            .evaluate(
                &[2.0, 3.0, guard],
                DerivativeOrder::Second,
                &mut BTreeMap::new(),
                &cancel,
            )
            .unwrap();
        assert_eq!(result.values, vec![25.0]);
        assert_eq!(result.jacobian, vec![10.0, 10.0]);
        assert_eq!(result.hessians, vec![2.0; 4]);
    }
}

#[test]
fn local_guard_facts_do_not_copy_the_enclosing_slot_inventory() {
    crate::initialize().unwrap();
    let x = library::formal(0).unwrap();
    let y = library::formal(1).unwrap();
    let lineage = form_lineage(id(13));
    let mut stages = vec![block(vec![&x * &x], vec![2])];
    for guard in 0..128 {
        stages.push(Stage::Domain {
            stages: vec![block(vec![x.clone()], vec![3])],
            argument: 3,
            token: 4 + guard,
            lineage: lineage.clone(),
        });
    }
    // Every local producer assigns slot 3, but the outer producer still owns it.
    stages.push(block(vec![&y * &y], vec![3]));
    // This covers base slot admission and actual local writes. Full snapshots for
    // these 128 guards would require over half a million handle copies.
    let mut remaining = 25_000;
    let body =
        PreparedBody::new_with_allowance(2, 4096, vec![2, 3], stages, &mut remaining).unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let support = body
        .prepare_support(&[0, 1], &[0, 1], DerivativeOrder::Second, &cancel)
        .unwrap();
    assert_eq!(
        support.support().first,
        vec![BTreeSet::from([0]), BTreeSet::from([1])]
    );
    assert_eq!(
        support.support().second,
        vec![BTreeSet::from([(0, 0)]), BTreeSet::from([(1, 1)])]
    );
    assert!(support.remaining_occurrences() > 0);
    let mut worker = body
        .compile(
            &[0, 1],
            &[0, 1],
            DerivativeOrder::Second,
            Optimization::default(),
            EvaluationLimits::default(),
            &cancel,
        )
        .unwrap()
        .worker();
    let result = worker
        .evaluate(
            &[2.0, 3.0],
            DerivativeOrder::Second,
            &mut BTreeMap::new(),
            &cancel,
        )
        .unwrap();
    assert_eq!(result.values, vec![4.0, 9.0]);
    assert_eq!(result.jacobian, vec![4.0, 0.0, 0.0, 6.0]);
    assert_eq!(
        result.hessians,
        vec![2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 2.0]
    );
    assert!(
        matches!(worker.evaluate(&[-2.0, 3.0], DerivativeOrder::Second,
        &mut BTreeMap::new(), &cancel), Err(MathError::Validity(actual)) if actual.as_ref() == lineage.as_ref())
    );
}

#[test]
fn shared_domain_snapshots_preserve_dense_support_and_local_isolation() {
    crate::initialize().unwrap();
    let n = 12;
    let guards = 16;
    let sum = (0..n).fold(Atom::num(0), |sum, i| sum + library::formal(i).unwrap());
    let mut stages = vec![block(vec![&sum * &sum], vec![n])];
    let lineage = form_lineage(id(13));
    for guard in 0..guards {
        stages.push(Stage::Domain {
            stages: vec![block(vec![library::formal(0).unwrap()], vec![n + 1])],
            argument: n + 1,
            token: n + 2 + guard,
            lineage: lineage.clone(),
        });
    }
    // The domains' private producers leave this outer destination unassigned.
    stages.push(block(vec![library::formal(1).unwrap()], vec![n + 1]));
    // This finite allowance covers new supports and handle snapshots, but cannot
    // cover sixteen deep copies of the already accumulated dense Hessian support.
    let mut remaining = 2400;
    let body =
        PreparedBody::new_with_allowance(n, n + 2 + guards, vec![n, n + 1], stages, &mut remaining)
            .unwrap();
    assert!(remaining > 0 && remaining < 2400);
    assert_eq!(
        crate::requested_test_support(&body).first[0],
        (0..n).collect()
    );
    assert_eq!(
        crate::requested_test_support(&body).second[0].len(),
        n * (n + 1) / 2
    );
    assert_eq!(
        crate::requested_test_support(&body).first[1],
        [1].into_iter().collect()
    );
    assert!(crate::requested_test_support(&body).second[1].is_empty());
    let cancel = Arc::new(AtomicBool::new(false));
    let mut worker = body
        .compile(
            &[0, 1],
            &[0, 1],
            DerivativeOrder::Second,
            Optimization::default(),
            EvaluationLimits::default(),
            &cancel,
        )
        .unwrap()
        .worker();
    let mut inputs = vec![1.0; n];
    inputs[1] = 2.0;
    let result = worker
        .evaluate(
            &inputs,
            DerivativeOrder::Second,
            &mut BTreeMap::new(),
            &cancel,
        )
        .unwrap();
    assert_eq!(result.values, vec![169.0, 2.0]);
    assert_eq!(result.jacobian, vec![26.0, 26.0, 0.0, 1.0]);
    assert_eq!(
        result.hessians,
        vec![2.0, 2.0, 2.0, 2.0, 0.0, 0.0, 0.0, 0.0]
    );
    inputs[0] = -1.0;
    assert!(matches!(worker.evaluate(
        &inputs, DerivativeOrder::Second, &mut BTreeMap::new(), &cancel,
    ), Err(MathError::Validity(actual)) if actual.as_ref() == lineage.as_ref()));
}

#[test]
fn opaque_nonlinear_support_refuses_actual_work_and_preserves_complete_pairs() {
    crate::initialize().unwrap();
    let n = 300;
    let mut remaining = crate::typed::BodyLimits::default().occurrences;
    let weak_body = PreparedBody::new_with_allowance(
        n + 1,
        n + 4,
        vec![n + 3],
        wide_opaque_stages(n, true),
        &mut remaining,
    )
    .unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let first = weak_body
        .incidence(&[0], &(0..n).collect::<Vec<_>>(), &cancel)
        .unwrap();
    assert!(first.support().second.is_empty());
    let error = first.upgrade(DerivativeOrder::Second, &cancel).unwrap_err();
    assert_eq!(first.support().first[0], (0..n).collect());
    assert!(first.support().second.is_empty());
    assert!(matches!(error, MathError::WorkLimit {
        source_id, resource: "derivative support construction", ..
    } if source_id == id(1)));
    let mut remaining = 300_000;
    let body = PreparedBody::new_with_allowance(
        n + 1,
        n + 4,
        vec![n + 3],
        wide_opaque_stages(n, true),
        &mut remaining,
    )
    .unwrap();
    assert!(remaining < 300_000);
    assert_eq!(
        crate::requested_test_support(&body).second[0].len(),
        n * (n + 1) / 2
    );
    assert!(crate::requested_test_support(&body).second[0].contains(&(0, n - 1)));
    let result = body
        .compile(
            &[0],
            &[0, n - 1],
            DerivativeOrder::Second,
            Optimization::default(),
            EvaluationLimits::default(),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
        .worker()
        .evaluate(
            &vec![1.0; n + 1],
            DerivativeOrder::Second,
            &mut BTreeMap::new(),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert_eq!(result.values, vec![(n * n) as f64]);
    assert_eq!(result.jacobian, vec![(2 * n) as f64; 2]);
    assert_eq!(result.hessians, vec![2.0; 4]);
}

#[test]
fn dense_provider_support_checks_cardinality_before_allocation() {
    crate::initialize().unwrap();
    let n = 64;
    let (mut spec, _, _) = provider();
    let port = spec.inputs[0].clone();
    spec.inputs = (0..n)
        .map(|i| Port {
            id: pse_ids::named_id(spec.id, &i.to_string()),
            ..port.clone()
        })
        .collect();
    let stages = vec![Stage::Provider {
        spec,
        partial: vec![],
        inputs: (0..n).collect(),
        outputs: vec![n],
        source: id(1),
    }];
    let mut remaining = 1000;
    let weak_body =
        PreparedBody::new_with_allowance(n, n + 1, vec![n], stages.clone(), &mut remaining)
            .unwrap();
    let error = weak_body
        .prepare_support(
            &[0],
            &(0..n).collect::<Vec<_>>(),
            DerivativeOrder::Second,
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap_err();
    assert!(matches!(error, MathError::WorkLimit {
        source_id, resource: "derivative support construction", required, available, ..
    } if source_id == id(1) && required == n * (n + 1) / 2 && available < required));
    let mut remaining = 10_000;
    let body = PreparedBody::new_with_allowance(n, n + 1, vec![n], stages, &mut remaining).unwrap();
    assert_eq!(
        crate::requested_test_support(&body).second[0].len(),
        n * (n + 1) / 2
    );
}

#[test]
fn conditional_first_reuses_exhausted_support_and_restricts_formal_coordinates() {
    use crate::typed::{Binary, BodyBuilder, BodyLimits};
    use pse_quantity::{IndexSet, standard::StandardInvariantChecker};
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let (body, ready) = (1..256)
        .find_map(|occurrences| {
            let mut builder = BodyBuilder::new(
                crate::initialize().unwrap(),
                &registry,
                &StandardInvariantChecker,
                2,
                BodyLimits {
                    occurrences,
                    ..Default::default()
                },
            )
            .ok()?;
            let x = builder
                .input(
                    0,
                    pse_quantity::standard::ids::quantity("neutral"),
                    IndexSet::new(),
                    id(1),
                )
                .ok()?;
            let y = builder
                .input(
                    1,
                    pse_quantity::standard::ids::quantity("neutral"),
                    IndexSet::new(),
                    id(2),
                )
                .ok()?;
            let product = builder
                .binary(Binary::Mul, x.clone(), y.clone(), None, id(3))
                .ok()?;
            let sum = builder.binary(Binary::Add, x, y, None, id(4)).ok()?;
            let body = builder.prepare(&[product, sum]).ok()?;
            let ready = body
                .prepare_support(&[0, 1], &[0, 1], DerivativeOrder::First, &cancel)
                .ok()?;
            (ready.remaining_occurrences() == 0).then_some((body, ready))
        })
        .expect("a finite source First construction can consume its exact allowance");
    let owner = Arc::new(());
    let weak = Arc::downgrade(&owner);
    let ready = ready.with_owner(owner.clone());
    assert!(matches!(
        ready.incidence(&[1], &cancel),
        Err(MathError::WorkLimit { .. })
    ));
    let projected = ready
        .conditional_support(&[1], &[1], DerivativeOrder::First, &cancel)
        .unwrap();
    assert_eq!(projected.outputs(), &[1]);
    assert_eq!(projected.coordinates(), &[1]);
    assert_eq!(projected.first_for_output(1).unwrap(), &BTreeSet::from([1]));
    assert!(projected.first_for_output(0).is_none());
    assert!(projected.support().second.is_empty());
    assert_eq!(projected.remaining_occurrences(), 0);
    assert_eq!(projected.derivative_operations(), 0);
    let result = projected
        .compile(
            Optimization::default(),
            EvaluationLimits::default(),
            &cancel,
        )
        .unwrap()
        .worker()
        .evaluate(
            &[4.0, 3.0],
            DerivativeOrder::First,
            &mut BTreeMap::new(),
            &cancel,
        )
        .unwrap();
    assert_eq!(result.values, vec![7.0]);
    assert_eq!(result.jacobian, vec![1.0]);
    // The case producer must also reuse this exhausted support before scheduling:
    // dependencies inspect the complete source; the block restricts output [1].
    use crate::{
        assembly::{AssemblyLimits, CasePlan},
        binding::*,
    };
    let quantity = pse_quantity::standard::ids::quantity("neutral");
    let port = |n| Port {
        id: id(n),
        quantity,
        unit: registry.quantity_type(quantity).unwrap().canonical_unit,
    };
    let key = ContentHash::from_bytes([7; 32]);
    let structure = Arc::new(
        CaseStructure::new(
            [1, 2]
                .into_iter()
                .map(|n| Variable {
                    port: port(n),
                    fixed: false,
                    domain: pse_model::generated::enums::ModelingVariableDomain::Continuous,
                    lower: None,
                    upper: None,
                })
                .collect(),
            vec![],
            vec![InstanceBinding {
                instance: id(9),
                body: key,
                checked_members: Default::default(),
                slots: ([1, 2]
                    .into_iter()
                    .map(|n| SlotBinding::new(&port(n), &port(n), &registry).unwrap())
                    .collect::<Vec<_>>())
                .into(),
                contributions: [10, 11]
                    .into_iter()
                    .enumerate()
                    .map(|(output, n)| Contribution {
                        output,
                        target: Target::Row(id(n)),
                        scale: 1.0,
                    })
                    .collect(),
            }],
            [10, 11]
                .into_iter()
                .map(|n| Row {
                    id: id(n),
                    quantity,
                    lower: 0.0,
                    upper: 0.0,
                })
                .collect(),
            None,
            CaseLimits {
                scalars: 2,
                instances: 1,
                rows: 2,
                bodies: 1,
                slots: 2,
            },
        )
        .unwrap(),
    );
    let plan = CasePlan::prepare(
        structure,
        BTreeMap::from([(key, Arc::new(body.clone()))]),
        &registry,
        DerivativeOrder::First,
        AssemblyLimits::default(),
        &cancel,
    )
    .unwrap();
    assert_eq!(plan.supports()[0].remaining_occurrences(), 0);
    let dependencies = plan.dependencies(&cancel).unwrap();
    assert_eq!(dependencies.len(), 2);
    let mut visited = Vec::new();
    assert_eq!(
        plan.visit_dependencies(&cancel, |dependency| {
            visited.push(dependency);
            std::ops::ControlFlow::<()>::Continue(())
        })
        .unwrap(),
        std::ops::ControlFlow::Continue(())
    );
    assert_eq!(visited, dependencies);
    let mut calls = 0;
    assert_eq!(
        plan.visit_dependencies(&cancel, |dependency| {
            calls += 1;
            std::ops::ControlFlow::Break(dependency.target)
        })
        .unwrap(),
        std::ops::ControlFlow::Break(dependencies[0].target)
    );
    assert_eq!(
        calls, 1,
        "a bounded consumer must stop before another contribution is projected"
    );
    cancel.store(true, Ordering::Relaxed);
    assert!(matches!(
        plan.visit_dependencies(&cancel, |_| {
            calls += 1;
            std::ops::ControlFlow::<()>::Continue(())
        }),
        Err(MathError::Cancelled)
    ));
    assert_eq!(
        calls, 1,
        "pre-cancelled visits must not invoke the consumer"
    );
    cancel.store(false, Ordering::Relaxed);
    calls = 0;
    assert!(matches!(
        plan.visit_dependencies(&cancel, |_| {
            calls += 1;
            cancel.store(true, Ordering::Relaxed);
            std::ops::ControlFlow::<()>::Continue(())
        }),
        Err(MathError::Cancelled)
    ));
    assert_eq!(
        calls, 1,
        "cancellation after one contribution stops the remaining inventory"
    );
    cancel.store(false, Ordering::Relaxed);
    let conditional = plan
        .conditional(
            &BTreeSet::from([id(11)]),
            &BTreeSet::from([id(2)]),
            &registry,
            &cancel,
        )
        .unwrap();
    assert_eq!(conditional.supports()[0].outputs(), &[1]);
    assert_eq!(conditional.supports()[0].coordinates(), &[1]);
    assert_eq!(conditional.supports()[0].remaining_occurrences(), 0);
    assert_eq!(conditional.supports()[0].derivative_operations(), 0);
    assert_eq!(
        conditional.limits().worker_bytes,
        plan.limits().worker_bytes
    );
    assert!(
        ready
            .conditional_support(&[2], &[1], DerivativeOrder::First, &cancel)
            .is_err()
    );
    let narrowed = body
        .prepare_support(&[0], &[0], DerivativeOrder::First, &cancel)
        .unwrap();
    assert!(
        narrowed
            .conditional_support(&[0], &[1], DerivativeOrder::First, &cancel)
            .is_err()
    );
    assert!(
        narrowed
            .conditional_support(&[1], &[0], DerivativeOrder::First, &cancel)
            .is_err()
    );
    cancel.store(true, Ordering::Release);
    assert!(matches!(
        ready.conditional_support(&[1], &[1], DerivativeOrder::First, &cancel),
        Err(MathError::Cancelled)
    ));
    drop(owner);
    drop(ready);
    assert!(weak.upgrade().is_some());
    drop(projected);
    assert!(weak.upgrade().is_none());
}

#[test]
fn conditional_second_reuses_exhausted_support_and_projects_selected_controls() {
    crate::initialize().unwrap();
    let x = library::formal(0).unwrap();
    let y = library::formal(1).unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let support = (1..512)
        .find_map(|allowance| {
            let mut remaining = allowance;
            let body = PreparedBody::new_with_allowance(
                2,
                5,
                vec![2, 3],
                vec![
                    block(vec![&x * &x], vec![2]),
                    Stage::Domain {
                        stages: vec![block(vec![y.clone()], vec![3])],
                        argument: 3,
                        token: 4,
                        lineage: form_lineage(id(13)),
                    },
                    block(vec![&y * &y], vec![3]),
                ],
                &mut remaining,
            )
            .ok()?;
            let support = body
                .prepare_support(&[0, 1], &[0, 1], DerivativeOrder::Second, &cancel)
                .ok()?;
            (support.remaining_occurrences() == 0).then_some(support)
        })
        .expect("an admitted Second support can consume its exact finite allowance");
    assert_eq!(support.support().controls, BTreeSet::from([1]));
    let projected = support
        .conditional_support(&[1], &[1], DerivativeOrder::Second, &cancel)
        .unwrap();
    assert_eq!(projected.order(), DerivativeOrder::Second);
    assert_eq!(projected.outputs(), &[1]);
    assert_eq!(projected.first_for_output(1).unwrap(), &BTreeSet::from([1]));
    assert_eq!(
        projected.second_for_output(1).unwrap(),
        &BTreeSet::from([(1, 1)])
    );
    assert_eq!(projected.support().controls, BTreeSet::from([1]));
    assert_eq!(projected.remaining_occurrences(), 0);
    assert_eq!(projected.derivative_operations(), 0);
    assert_eq!(projected.input_formals(), &[1]);
    let mut worker = projected
        .compile(
            Optimization::default(),
            EvaluationLimits::default(),
            &cancel,
        )
        .unwrap()
        .worker();
    let result = worker
        .evaluate(
            &[3.0],
            DerivativeOrder::Second,
            &mut BTreeMap::new(),
            &cancel,
        )
        .unwrap();
    assert_eq!(result.values, vec![9.0]);
    assert_eq!(result.jacobian, vec![6.0]);
    assert_eq!(result.hessians, vec![2.0]);
    assert!(matches!(
        worker.evaluate(
            &[-3.0],
            DerivativeOrder::Second,
            &mut BTreeMap::new(),
            &cancel
        ),
        Err(MathError::Validity(_))
    ));
    let other = support
        .conditional_support(&[0], &[0], DerivativeOrder::Second, &cancel)
        .unwrap();
    assert_eq!(
        other.second_for_output(0).unwrap(),
        &BTreeSet::from([(0, 0)])
    );
    assert!(other.support().controls.is_empty());
    let first = support
        .conditional_support(&[1], &[1], DerivativeOrder::First, &cancel)
        .unwrap();
    assert!(
        first
            .conditional_support(&[1], &[1], DerivativeOrder::Second, &cancel)
            .is_err()
    );
}

#[test]
fn support_construction_consumes_the_remaining_authored_body_allowance() {
    use crate::typed::{Binary, BodyBuilder, BodyLimits};
    use pse_quantity::{IndexSet, standard::StandardInvariantChecker};
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let prepare = |occurrences| {
        let mut builder = BodyBuilder::new(
            crate::initialize().unwrap(),
            &registry,
            &StandardInvariantChecker,
            1,
            BodyLimits {
                occurrences,
                ..BodyLimits::default()
            },
        )
        .unwrap();
        let x = builder
            .input(
                0,
                pse_quantity::standard::ids::quantity("neutral"),
                IndexSet::new(),
                id(1),
            )
            .unwrap();
        let value = builder
            .binary(Binary::Mul, x.clone(), x, None, id(1))
            .unwrap();
        builder.prepare(&[value])
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let body = prepare(128).unwrap();
    assert_eq!(body.occurrence_count(), 2);
    let value = body
        .prepare_support(&[0], &[0], DerivativeOrder::Value, &cancel)
        .unwrap();
    assert!(value.support().first.is_empty());
    assert!(value.support().second.is_empty());
    let first = value.upgrade(DerivativeOrder::First, &cancel).unwrap();
    assert!(first.remaining_occurrences() < value.remaining_occurrences());
    assert!(first.support().second.is_empty());
    let second = first.upgrade(DerivativeOrder::Second, &cancel).unwrap();
    assert!(second.remaining_occurrences() < first.remaining_occurrences());
    assert_eq!(second.support().second[0], [(0, 0)].into_iter().collect());
    assert!(first.support().second.is_empty());
    // An authored allowance cannot be reset at the support boundary.
    let exhausted = (4..128)
        .filter_map(|limit| prepare(limit).ok())
        .find(|body| {
            body.prepare_support(&[0], &[0], DerivativeOrder::Value, &cancel)
                .is_ok()
                && body
                    .prepare_support(&[0], &[0], DerivativeOrder::Second, &cancel)
                    .is_err()
        })
        .unwrap();
    assert!(
        exhausted
            .compile(
                &[0],
                &[0],
                DerivativeOrder::Value,
                Optimization::default(),
                EvaluationLimits::default(),
                &cancel
            )
            .is_ok()
    );
}

#[test]
fn failed_builder_cannot_prepare_a_retained_valid_input_after_exhaustion() {
    use crate::typed::{BodyBuilder, BodyLimits};
    use pse_quantity::{IndexSet, standard::StandardInvariantChecker};
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let quantity = pse_quantity::standard::ids::quantity("neutral");
    let mut builder = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &StandardInvariantChecker,
        1,
        BodyLimits {
            occurrences: 1,
            ..BodyLimits::default()
        },
    )
    .unwrap();
    let retained = builder.input(0, quantity, IndexSet::new(), id(1)).unwrap();
    assert!(matches!(
        builder.input(0, quantity, IndexSet::new(), id(2)),
        Err(MathError::Limit("body occurrences")),
    ));
    assert!(matches!(
        builder.prepare(&[retained]),
        Err(MathError::Limit("body occurrences")),
    ));
}

/// An evaluator's retained storage counts its instruction stream beside its numeric stack:
/// the library storage a retained program's reservation covers now that no foreign
/// allowance is retained with it (H9).
#[test]
fn evaluator_storage_counts_its_instruction_stream() {
    crate::initialize().unwrap();
    let x = library::formal(0).unwrap();
    let y = library::formal(1).unwrap();
    let limits = EvaluationLimits::default();
    let layout = crate::jets::JetLayout::new(vec![], DerivativeOrder::Value, limits).unwrap();
    let evaluator = library::bounded_evaluator(
        id(1),
        &[(&x * &y).sin() + &x * &x * &y + (&x + &y).exp()],
        &[x, y],
        &layout,
        &[],
        Optimization::default(),
        &Arc::new(AtomicBool::new(false)),
        limits,
        limits.operations,
        0,
    )
    .unwrap();
    let export = evaluator.export_instructions();
    let storage = library::storage(&evaluator).unwrap();
    assert!(!export.instructions.is_empty());
    assert!(storage.instruction_bytes > size_of_val(export.instructions.as_slice()));
    assert_eq!(
        storage.numeric_entries,
        export.input_count + export.constants.len() + export.temporary_count
    );
}
#[test]
fn numerica_raw_diagonal_mixed_and_multi_output_derivatives() {
    crate::initialize().unwrap();
    let x = library::formal(0).unwrap();
    let y = library::formal(1).unwrap();
    let body = compile(
        2,
        4,
        vec![block(vec![&x * &x * &y, x.sin()], vec![2, 3])],
        vec![2, 3],
        DerivativeOrder::Second,
    );
    let cancel = Arc::new(AtomicBool::new(false));
    let mut worker = body.worker();
    let mut providers = BTreeMap::new();
    let v = worker
        .evaluate(
            &[2.0, 3.0],
            DerivativeOrder::Second,
            &mut providers,
            &cancel,
        )
        .unwrap();
    assert_eq!(v.values[0], 12.0);
    assert_eq!(&v.jacobian[..2], &[12.0, 4.0]);
    assert_eq!(&v.hessians[..4], &[6.0, 4.0, 4.0, 0.0]);
    assert!((v.hessians[4] + 2.0_f64.sin()).abs() < 1e-14);
    let v = worker
        .evaluate(&[2.0, 3.0], DerivativeOrder::Value, &mut providers, &cancel)
        .unwrap();
    assert!(v.jacobian.is_empty() && v.hessians.is_empty());
    assert_eq!(
        body.support().second[0],
        [(0, 0), (0, 1)].into_iter().collect()
    );
}

#[derive(Debug)]
struct Cubic {
    spec: ProviderSpec,
    calls: Arc<AtomicUsize>,
}
impl Provider for Cubic {
    fn spec(&self) -> &ProviderSpec {
        &self.spec
    }
    fn evaluate(
        &mut self,
        inputs: &[f64],
        request: &ProviderRequest,
        context: &EvaluationContext<'_>,
    ) -> Result<ProviderValues, ProviderError> {
        request.validate(&self.spec, context)?;
        self.calls.fetch_add(1, Ordering::Relaxed);
        if inputs[0] <= 0.0 {
            return Err(ProviderError::Trial("positive cubic input".into()));
        }
        let x = inputs[0];
        Ok(ProviderValues {
            values: request.outputs.iter().map(|_| x * x * x).collect(),
            jacobian: if request.order >= DerivativeOrder::First {
                vec![3.0 * x * x; request.outputs.len()]
            } else {
                vec![]
            },
            hessians: if request.order >= DerivativeOrder::Second {
                vec![6.0 * x; request.outputs.len()]
            } else {
                vec![]
            },
        })
    }
}
fn provider() -> (ProviderSpec, Box<dyn Provider>, Arc<AtomicUsize>) {
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let q = pse_quantity::standard::ids::quantity("neutral");
    let u = registry.quantity_type(q).unwrap().canonical_unit;
    let spec = ProviderSpec {
        shapes: ProviderShapes::default(),
        derivative_source: DerivativeSource::Analytic,

        id: id(10),
        revision: ContentHash::from_bytes([1; 32]),
        data: ContentHash::from_bytes([2; 32]),

        inputs: vec![Port {
            id: id(12),
            quantity: q,
            unit: u,
        }],
        outputs: vec![Port {
            id: id(13),
            quantity: q,
            unit: u,
        }],
        derivatives: DerivativeOrder::Second,
        smoothness: DerivativeOrder::Second,
    };
    let calls = Arc::new(AtomicUsize::new(0));
    (
        spec.clone(),
        Box::new(Cubic {
            spec,
            calls: calls.clone(),
        }),
        calls,
    )
}
#[test]
fn domain_predicates_use_value_only_providers_and_remain_demand_scoped() {
    use crate::typed::{Binary, BodyBuilder, BodyLimits};
    use pse_quantity::{
        IndexSet,
        standard::{StandardInvariantChecker, standard_registry},
    };
    let registry = standard_registry().unwrap();
    let mut b = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &StandardInvariantChecker,
        1,
        BodyLimits::default(),
    )
    .unwrap();
    let (mut spec, _, calls) = provider();
    spec.derivatives = DerivativeOrder::Value;
    spec.smoothness = DerivativeOrder::Value;
    let admitted = AdmittedProvider::new(spec.clone(), &registry).unwrap();
    let x = b
        .input(0, spec.inputs[0].quantity, IndexSet::new(), id(1))
        .unwrap();
    let assumption = b
        .domain(form_lineage(id(22)), |b| {
            Ok(b.provider(&admitted, std::slice::from_ref(&x), id(21))?
                .remove(0))
        })
        .unwrap();
    let square = b
        .binary(Binary::Mul, x.clone(), x.clone(), None, id(1))
        .unwrap();
    let guarded = b.with_assumption(square.clone(), &assumption);
    let prepared = b.prepare(&[guarded, square]).unwrap();
    assert_eq!(prepared.available_order(), DerivativeOrder::Second);
    assert!(
        crate::requested_test_support(&prepared)
            .controls
            .contains(&0)
    );
    let cancel = Arc::new(AtomicBool::new(false));
    let key = spec.key();
    let provider: Box<dyn Provider> = Box::new(Cubic {
        spec,
        calls: calls.clone(),
    });
    let mut workers = BTreeMap::from([(key, provider)]);
    let mut guarded = prepared
        .compile(
            &[0],
            &[0],
            DerivativeOrder::Second,
            Optimization::default(),
            EvaluationLimits::default(),
            &cancel,
        )
        .unwrap()
        .worker();
    let value = guarded
        .evaluate(&[2.], DerivativeOrder::Second, &mut workers, &cancel)
        .unwrap();
    assert_eq!(value.values, [4.]);
    assert_eq!(value.jacobian, [4.]);
    assert_eq!(value.hessians, [2.]);
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert!(
        guarded
            .evaluate(&[-2.], DerivativeOrder::Second, &mut workers, &cancel)
            .is_err()
    );
    let before = calls.load(Ordering::Relaxed);
    let mut unguarded = prepared
        .compile(
            &[1],
            &[0],
            DerivativeOrder::Second,
            Optimization::default(),
            EvaluationLimits::default(),
            &cancel,
        )
        .unwrap()
        .worker();
    assert_eq!(
        unguarded
            .evaluate(&[-2.], DerivativeOrder::Second, &mut workers, &cancel)
            .unwrap()
            .values,
        [4.]
    );
    assert_eq!(calls.load(Ordering::Relaxed), before);
    cancel.store(true, Ordering::Relaxed);
    assert!(matches!(
        guarded.evaluate(&[2.], DerivativeOrder::Second, &mut workers, &cancel),
        Err(MathError::Cancelled)
    ));
}

#[test]
fn composed_external_partials_use_symbolica_chain_rule_through_nested_calls() {
    use crate::typed::{BodyBuilder, BodyLimits};
    use pse_quantity::{
        IndexSet,
        standard::{StandardInvariantChecker, standard_registry},
    };
    let registry = standard_registry().unwrap();
    for (order, partials, expected, derivative) in [
        (DerivativeOrder::First, 1, 2304., 9216.),
        (DerivativeOrder::Value, 2, 9216., 0.),
    ] {
        let mut b = BodyBuilder::new(
            crate::initialize().unwrap(),
            &registry,
            &StandardInvariantChecker,
            1,
            BodyLimits::default(),
        )
        .unwrap();
        let (spec, provider, _) = provider();
        let admitted = AdmittedProvider::new(spec.clone(), &registry).unwrap();
        let x = b
            .input(0, spec.inputs[0].quantity, IndexSet::new(), id(1))
            .unwrap();
        let x = b.bind(x).unwrap();
        let scope = b.function_scope();
        let y = b
            .provider(&admitted, std::slice::from_ref(&x), id(21))
            .unwrap()
            .remove(0);
        let z = b.provider(&admitted, &[y], id(22)).unwrap().remove(0);
        let partial = b.partial(scope, z, &vec![x; partials], id(23)).unwrap();
        let body = b.prepare(&[partial]).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let mut compiled = body
            .compile(
                &[0],
                &[0],
                order,
                Optimization::default(),
                EvaluationLimits::default(),
                &cancel,
            )
            .unwrap()
            .worker();
        let mut providers = BTreeMap::from([(spec.key(), provider)]);
        let result = compiled
            .evaluate(&[2.], order, &mut providers, &cancel)
            .unwrap();
        assert_eq!(result.values, [expected]);
        if order == DerivativeOrder::First {
            assert_eq!(result.jacobian, [derivative]);
        }
        assert!(
            compiled
                .evaluate(&[0.], order, &mut providers, &cancel)
                .is_err()
        );
        assert!(
            body.compile(
                &[0],
                &[0],
                DerivativeOrder::Second,
                Optimization::default(),
                EvaluationLimits::default(),
                &cancel
            )
            .is_err()
        );
    }
}

#[test]
fn symbolica_composes_provider_partials_after_arithmetic_barriers() {
    crate::initialize().unwrap();
    let (spec, provider, calls) = provider();
    let x = library::formal(0).unwrap();
    let stages = vec![
        block(vec![&x * &x], vec![1]),
        Stage::Provider {
            partial: vec![],
            spec: spec.clone(),
            inputs: vec![1],
            outputs: vec![2],
            source: id(21),
        },
        block(vec![library::formal(2).unwrap() + &x], vec![3]),
    ];
    let body = compile(1, 4, stages, vec![3], DerivativeOrder::Second);
    let mut worker = body.worker();
    let cancel = Arc::new(AtomicBool::new(false));
    let mut providers = BTreeMap::from([(spec.key(), provider)]);
    let v = worker
        .evaluate(&[2.0], DerivativeOrder::Second, &mut providers, &cancel)
        .unwrap();
    assert_eq!(v.values, vec![66.0]);
    assert_eq!(v.jacobian, vec![193.0]);
    assert_eq!(v.hessians, vec![480.0]);
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert!(
        matches!(worker.evaluate(&[0.0],DerivativeOrder::Second,&mut providers,&cancel),Err(MathError::Provider{source_id,..}) if source_id==id(21))
    );
    let mut other = body.worker();
    assert_eq!(
        other
            .evaluate(&[1.0], DerivativeOrder::Second, &mut providers, &cancel)
            .unwrap()
            .hessians,
        vec![30.0]
    );
    assert_eq!(
        worker
            .evaluate(&[2.0], DerivativeOrder::Second, &mut providers, &cancel)
            .unwrap()
            .hessians,
        vec![480.0]
    );
}

#[test]
fn derivative_budget_cancellation_and_numeric_zero_support() {
    crate::initialize().unwrap();
    let x = library::formal(0).unwrap();
    let p = PreparedBody::new(
        1,
        2,
        vec![1],
        vec![block(vec![&x * &x * &x], vec![1])],
        DerivativeOrder::Second,
    )
    .unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let budget = EvaluationLimits {
        derivative_components: 2,
        ..EvaluationLimits::default()
    };
    assert!(matches!(
        p.compile(
            &[0],
            &[0],
            DerivativeOrder::Second,
            Optimization::default(),
            budget,
            &cancel
        ),
        Err(MathError::Limit(_))
    ));
    let body = p
        .compile(
            &[0],
            &[0],
            DerivativeOrder::Second,
            Optimization::default(),
            EvaluationLimits::default(),
            &cancel,
        )
        .unwrap();
    let mut worker = body.worker();
    assert_eq!(
        worker
            .evaluate(
                &[0.0],
                DerivativeOrder::Second,
                &mut BTreeMap::new(),
                &cancel
            )
            .unwrap()
            .hessians,
        vec![0.0]
    );
    assert!(body.support().second[0].contains(&(0, 0)));
    cancel.store(true, Ordering::Relaxed);
    assert!(matches!(
        worker.evaluate(
            &[2.0],
            DerivativeOrder::Value,
            &mut BTreeMap::new(),
            &cancel
        ),
        Err(MathError::Cancelled)
    ));
}

#[test]
fn parameter_only_switches_preserve_all_branch_support() {
    crate::initialize().unwrap();
    let x = library::formal(0).unwrap();
    let p = library::formal(1).unwrap();
    let body = PreparedBody::new(
        2,
        4,
        vec![3],
        vec![
            block(vec![Atom::num(0)], vec![2]),
            Stage::Branch {
                continuity: DerivativeOrder::Value,
                comparison: Comparison::Lt,
                left: 2,
                right: 1,
                then: vec![block(vec![&x * &x], vec![3])],
                otherwise: vec![block(vec![&x * &x * &x], vec![3])],
            },
        ],
        DerivativeOrder::Second,
    )
    .unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    assert!(
        body.compile(
            &[0],
            &[0, 1],
            DerivativeOrder::First,
            Optimization::default(),
            EvaluationLimits::default(),
            &cancel
        )
        .is_err()
    );
    let mut worker = body
        .compile(
            &[0],
            &[0],
            DerivativeOrder::Second,
            Optimization::default(),
            EvaluationLimits::default(),
            &cancel,
        )
        .unwrap()
        .worker();
    let a = worker
        .evaluate(
            &[2.0, 1.0],
            DerivativeOrder::Second,
            &mut BTreeMap::new(),
            &cancel,
        )
        .unwrap();
    assert_eq!(a.hessians, vec![2.0]);
    let b = worker
        .evaluate(
            &[2.0, -1.0],
            DerivativeOrder::Second,
            &mut BTreeMap::new(),
            &cancel,
        )
        .unwrap();
    assert_eq!(b.hessians, vec![12.0]);
    assert!(crate::requested_test_support(&body).controls.contains(&1));
    assert!(!crate::requested_test_support(&body).first[0].contains(&1));
    assert_eq!(
        crate::requested_test_support(&body).second[0],
        [(0, 0)].into_iter().collect()
    );
    let _ = p;
}
#[test]
fn multi_input_provider_lift_preserves_mixed_raw_partials() {
    crate::initialize().unwrap();
    #[derive(Debug)]
    struct Product {
        spec: ProviderSpec,
    }
    impl Provider for Product {
        fn spec(&self) -> &ProviderSpec {
            &self.spec
        }
        fn evaluate(
            &mut self,
            x: &[f64],
            r: &ProviderRequest,
            c: &EvaluationContext<'_>,
        ) -> Result<ProviderValues, ProviderError> {
            r.validate(&self.spec, c)?;
            Ok(ProviderValues {
                values: vec![x[0] * x[1]],
                jacobian: if r.order >= DerivativeOrder::First {
                    vec![x[1], x[0]]
                } else {
                    vec![]
                },
                hessians: if r.order >= DerivativeOrder::Second {
                    vec![0.0, 1.0, 1.0, 0.0]
                } else {
                    vec![]
                },
            })
        }
    }
    let (mut spec, _, _) = provider();
    let mut second = spec.inputs[0].clone();
    second.id = id(77);
    spec.inputs.push(second);
    let x = library::formal(0).unwrap();
    let y = library::formal(1).unwrap();
    let key = spec.key();
    let body = compile(
        2,
        5,
        vec![
            block(vec![&x * &x, &x * &y], vec![2, 3]),
            Stage::Provider {
                partial: vec![],
                spec: spec.clone(),
                inputs: vec![2, 3],
                outputs: vec![4],
                source: id(9),
            },
        ],
        vec![4],
        DerivativeOrder::Second,
    );
    let provider: Box<dyn Provider> = Box::new(Product { spec });
    let mut providers: BTreeMap<_, Box<dyn Provider>> = BTreeMap::from([(key, provider)]);
    let v = body
        .worker()
        .evaluate(
            &[2.0, 3.0],
            DerivativeOrder::Second,
            &mut providers,
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert_eq!(v.values, vec![24.0]);
    assert_eq!(v.jacobian, vec![36.0, 8.0]);
    assert_eq!(v.hessians, vec![36.0, 12.0, 12.0, 0.0]);
}
#[test]
fn typed_output_demand_coalesces_calls_and_keeps_canceled_obligations() {
    crate::initialize().unwrap();
    use crate::typed::{Binary, BodyBuilder, BodyLimits};
    use pse_quantity::{
        IndexSet,
        standard::{StandardInvariantChecker, ids, standard_registry},
    };
    #[derive(Debug)]
    struct Factory {
        spec: ProviderSpec,
        requests: Arc<std::sync::Mutex<Vec<ProviderRequest>>>,
    }
    #[derive(Debug)]
    struct Properties {
        spec: ProviderSpec,
        requests: Arc<std::sync::Mutex<Vec<ProviderRequest>>>,
    }
    impl Provider for Properties {
        fn spec(&self) -> &ProviderSpec {
            &self.spec
        }
        fn evaluate(
            &mut self,
            x: &[f64],
            r: &ProviderRequest,
            c: &EvaluationContext<'_>,
        ) -> Result<ProviderValues, ProviderError> {
            r.validate(&self.spec, c)?;
            self.requests.lock().unwrap().push(r.clone());
            if x[0] <= 0.0 {
                return Err(ProviderError::Trial("positive property input".into()));
            }
            let mut result = ProviderValues {
                values: vec![],
                jacobian: vec![],
                hessians: vec![],
            };
            for &o in &r.outputs {
                result
                    .values
                    .push(if o == 0 { x[0] * x[0] } else { x[0].ln() });
                if r.order >= DerivativeOrder::First {
                    result
                        .jacobian
                        .push(if o == 0 { 2.0 * x[0] } else { 1.0 / x[0] });
                }
                if r.order >= DerivativeOrder::Second {
                    result
                        .hessians
                        .push(if o == 0 { 2.0 } else { -1.0 / (x[0] * x[0]) });
                }
            }
            Ok(result)
        }
    }
    impl ProviderFactory for Factory {
        fn spec(&self) -> &ProviderSpec {
            &self.spec
        }
        fn create(&self) -> Result<Box<dyn Provider>, ProviderError> {
            Ok(Box::new(Properties {
                spec: self.spec.clone(),
                requests: self.requests.clone(),
            }))
        }
    }
    let registry = standard_registry().unwrap();
    let (mut spec, _, _) = provider();
    let mut port = spec.outputs[0].clone();
    port.id = id(88);
    spec.outputs.push(port);
    let key = spec.key();
    let requests = Arc::new(std::sync::Mutex::new(vec![]));
    let registration = Registration::new(
        Arc::new(Factory {
            spec,
            requests: requests.clone(),
        }),
        &registry,
    )
    .unwrap();
    let mut b = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &StandardInvariantChecker,
        1,
        BodyLimits::default(),
    )
    .unwrap();
    let x = b
        .input(0, ids::quantity("neutral"), IndexSet::new(), id(1))
        .unwrap();
    let first = b
        .provider(&registration.descriptor(), std::slice::from_ref(&x), id(2))
        .unwrap();
    let second = b
        .provider(&registration.descriptor(), std::slice::from_ref(&x), id(3))
        .unwrap();
    let canceled = b
        .binary(Binary::Sub, first[0].clone(), first[0].clone(), None, id(4))
        .unwrap();
    let prepared = b.prepare(&[x, canceled, second[1].clone()]).unwrap();
    assert_eq!(prepared.providers().len(), 1);
    let cancel = Arc::new(AtomicBool::new(false));
    let mut workers = BTreeMap::from([(key, registration.worker().unwrap())]);
    let compile = |outputs: &[usize]| {
        prepared
            .compile(
                outputs,
                &[0],
                DerivativeOrder::Second,
                Optimization::default(),
                EvaluationLimits::default(),
                &cancel,
            )
            .unwrap()
            .worker()
    };
    assert_eq!(
        compile(&[0])
            .evaluate(&[-2.0], DerivativeOrder::Value, &mut workers, &cancel)
            .unwrap()
            .values,
        vec![-2.0]
    );
    assert!(requests.lock().unwrap().is_empty());
    assert!(
        compile(&[1])
            .evaluate(&[-2.0], DerivativeOrder::Value, &mut workers, &cancel)
            .is_err()
    );
    requests.lock().unwrap().clear();
    let result = compile(&[1, 2])
        .evaluate(&[2.0], DerivativeOrder::Second, &mut workers, &cancel)
        .unwrap();
    assert_eq!(result.values, vec![0.0, 2.0_f64.ln()]);
    assert_eq!(result.hessians, vec![0.0, -0.25]);
    assert_eq!(requests.lock().unwrap().len(), 1);
    assert_eq!(requests.lock().unwrap()[0].outputs, vec![0, 1]);
    requests.lock().unwrap().clear();
    compile(&[2])
        .evaluate(&[2.0], DerivativeOrder::Value, &mut workers, &cancel)
        .unwrap();
    assert_eq!(
        requests.lock().unwrap()[0],
        ProviderRequest {
            outputs: vec![1],
            order: DerivativeOrder::Value
        }
    );
    use crate::assembly::{AssemblyLimits, CasePlan};
    use crate::binding::*;
    use pse_model::generated::enums::ModelingVariableDomain;
    let quantity = ids::quantity("neutral");
    let port = Port {
        id: id(1),
        quantity,
        unit: registry.quantity_type(quantity).unwrap().canonical_unit,
    };
    let body_key = ContentHash::from_bytes([9; 32]);
    let structure = Arc::new(
        CaseStructure::new(
            vec![Variable {
                port: port.clone(),
                fixed: false,
                domain: ModelingVariableDomain::Continuous,
                lower: Some(0.1),
                upper: Some(10.0),
            }],
            vec![],
            vec![InstanceBinding {
                checked_members: Default::default(),
                instance: id(5),
                body: body_key,
                slots: (vec![SlotBinding::new(&port, &port, &registry).unwrap()]).into(),
                contributions: vec![
                    Contribution {
                        output: 2,
                        target: Target::PRIMARY,
                        scale: 1.0,
                    },
                    Contribution {
                        output: 1,
                        target: Target::Row(id(6)),
                        scale: 1.0,
                    },
                ],
            }],
            vec![Row {
                id: id(6),
                quantity,
                lower: 0.0,
                upper: 0.0,
            }],
            Some(Objective {
                quantity,
                sense: ObjectiveSense::Minimize,
            }),
            CaseLimits::default(),
        )
        .unwrap(),
    );
    let case = Arc::new(
        Arc::new(
            CasePlan::prepare(
                structure,
                BTreeMap::from([(body_key, Arc::new(prepared))]),
                &registry,
                DerivativeOrder::Second,
                AssemblyLimits::default(),
                &cancel,
            )
            .unwrap(),
        )
        .compile(
            Optimization::default(),
            EvaluationLimits::default(),
            &cancel,
        )
        .unwrap(),
    );
    let mut worker = case.worker(workers, cancel);
    let values = CaseValues {
        scalars: BTreeMap::from([(id(1), 2.0)]),
    };
    requests.lock().unwrap().clear();
    assert_eq!(worker.hessian(&values, 2.0, &[3.0]).unwrap().val(), &[-0.5]);
    assert_eq!(worker.hessian(&values, 4.0, &[9.0]).unwrap().val(), &[-1.0]);
    assert_eq!(requests.lock().unwrap().len(), 1);
    let bad = CaseValues {
        scalars: BTreeMap::from([(id(1), -2.0)]),
    };
    let error = worker.hessian(&bad, 1.0, &[0.0]).unwrap_err();
    assert!(
        matches!(error,MathError::Instance{instance,cause,..} if instance==id(5) && matches!(*cause,MathError::Provider{source_id,..} if source_id==id(2)))
    );
}

#[test]
fn full_twenty_and_forty_four_coordinate_jets_fit_real_convolution_budget() {
    crate::initialize().unwrap();
    for n in [20, 44] {
        let atoms = (0..n)
            .map(|i| library::formal(i).unwrap())
            .collect::<Vec<_>>();
        let sum = atoms.iter().fold(Atom::num(0), |s, x| s + x);
        let squares = atoms.iter().fold(Atom::num(0), |s, x| s + x * x);
        let body = compile(
            n,
            n + 2,
            vec![block(vec![squares, sum.sin()], vec![n, n + 1])],
            vec![n, n + 1],
            DerivativeOrder::Second,
        );
        let x = vec![0.5 / n as f64; n];
        let result = body
            .worker()
            .evaluate(
                &x,
                DerivativeOrder::Second,
                &mut BTreeMap::new(),
                &Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
        assert!((result.values[0] - 0.25 / n as f64).abs() < 1e-12);
        assert!((result.values[1] - 0.5_f64.sin()).abs() < 1e-12);
        for i in 0..n {
            assert!((result.jacobian[i] - 1.0 / n as f64).abs() < 1e-12);
            assert!((result.jacobian[n + i] - 0.5_f64.cos()).abs() < 1e-12);
            for j in 0..n {
                assert!(
                    (result.hessians[i * n + j] - if i == j { 2.0 } else { 0.0 }).abs() < 1e-12
                );
                assert!((result.hessians[n * n + i * n + j] + 0.5_f64.sin()).abs() < 1e-12);
            }
        }
    }
}

#[test]
fn supported_taylor_primitives_reconcile_with_admitted_expansion() {
    crate::initialize().unwrap();
    let x = library::formal(0).unwrap();
    let y = library::formal(1).unwrap();
    let expressions = vec![
        x.exp(),
        x.log(),
        x.sin(),
        x.cos(),
        x.sqrt(),
        Atom::num(1) / &x,
        x.clone().pow(y.clone()),
    ];
    let outputs = (2..2 + expressions.len()).collect::<Vec<_>>();
    let body = compile(
        2,
        outputs.len() + 2,
        vec![block(expressions, outputs.clone())],
        outputs,
        DerivativeOrder::Second,
    );
    let result = body
        .worker()
        .evaluate(
            &[2.0, 1.5],
            DerivativeOrder::Second,
            &mut BTreeMap::new(),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert!(result.hessians.iter().all(|x| x.is_finite()));
    assert!((result.values[6] - 2.0_f64.powf(1.5)).abs() < 1e-12);
}

#[test]
fn derivative_work_refusal_retains_required_and_available_operations() {
    crate::initialize().unwrap();
    let body = PreparedBody::new(
        2,
        3,
        vec![2],
        vec![block(
            vec![library::formal(0).unwrap() * library::formal(1).unwrap()],
            vec![2],
        )],
        DerivativeOrder::Second,
    )
    .unwrap();
    // Value consumes one operation; First receives the remaining one, not a reset budget.
    let error = body
        .compile(
            &[0],
            &[0, 1],
            DerivativeOrder::First,
            Optimization::default(),
            EvaluationLimits {
                operations: 2,
                ..Default::default()
            },
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap_err();
    assert!(
        matches!(error,MathError::WorkLimit{source_id,required,available:1,components:3,..} if source_id==id(1) && required>1),
        "{error:?}"
    );
}

#[test]
fn local_taylor_coordinates_preserve_transitive_and_permuted_derivatives() {
    crate::initialize().unwrap();
    let n = 16;
    let mut stages = vec![];
    for i in 0..n {
        let x = library::formal(i).unwrap();
        stages.push(block(vec![&x * &x], vec![n + i]));
        stages.push(block(
            vec![library::formal(n + i).unwrap().sin()],
            vec![2 * n + i],
        ));
    }
    let outputs = (2 * n..3 * n).collect::<Vec<_>>();
    let body = PreparedBody::new(n, 3 * n, outputs, stages, DerivativeOrder::Second).unwrap();
    let coordinates = (0..n).rev().collect::<Vec<_>>();
    let cancel = Arc::new(AtomicBool::new(false));
    let compiled = body
        .compile(
            &(0..n).collect::<Vec<_>>(),
            &coordinates,
            DerivativeOrder::Second,
            Optimization::default(),
            EvaluationLimits {
                operations: 5000,
                ..Default::default()
            },
            &cancel,
        )
        .unwrap();
    let inputs = (0..n).map(|i| (i + 1) as f64 / 10.0).collect::<Vec<_>>();
    let result = compiled
        .worker()
        .evaluate(
            &inputs,
            DerivativeOrder::Second,
            &mut BTreeMap::new(),
            &cancel,
        )
        .unwrap();
    for (row, &x) in inputs.iter().enumerate() {
        assert!((result.values[row] - (x * x).sin()).abs() < 1e-12);
        for (i, &coordinate) in coordinates.iter().enumerate() {
            let expected = if coordinate == row {
                2.0 * x * (x * x).cos()
            } else {
                0.0
            };
            assert!((result.jacobian[row * n + i] - expected).abs() < 1e-12);
            for j in 0..n {
                let expected = if coordinate == row && i == j {
                    2.0 * (x * x).cos() - 4.0 * x * x * (x * x).sin()
                } else {
                    0.0
                };
                assert!((result.hessians[row * n * n + i * n + j] - expected).abs() < 1e-12);
            }
        }
    }
}

/// The lineage of a form-layer predicate stated by `source`, reading no parameter set.
fn form_lineage(source: SemanticId) -> Arc<pse_model::diagnostic::ValidityLineage> {
    Arc::new(pse_model::diagnostic::ValidityLineage {
        layer: pse_model::generated::enums::ModelingValidityLayer::Form,
        source,
        form: Some(source),
        sets: Vec::new(),
        variables: Vec::new(),
        members: Vec::new(),
    })
}

#[test]
fn selected_support_separates_value_first_second_and_retries_after_cancellation() {
    crate::initialize().unwrap();
    let n = 80;
    let body = PreparedBody::new(
        n + 1,
        n + 4,
        vec![n + 3],
        wide_opaque_stages(n, true),
        DerivativeOrder::Second,
    )
    .unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let value = body
        .prepare_support(&[0], &[0, n - 1], DerivativeOrder::Value, &cancel)
        .unwrap();
    assert!(value.support().first.is_empty());
    assert!(value.support().second.is_empty());
    assert_eq!(value.derivative_operations(), 0);
    let first = value.upgrade(DerivativeOrder::First, &cancel).unwrap();
    assert_eq!(first.support().first[0], [0, n - 1].into_iter().collect());
    assert!(first.support().second.is_empty());
    assert_eq!(first.derivative_operations(), 0);
    cancel.store(true, Ordering::Relaxed);
    assert!(matches!(
        first.upgrade(DerivativeOrder::Second, &cancel),
        Err(MathError::Cancelled)
    ));
    cancel.store(false, Ordering::Relaxed);
    let second = first.upgrade(DerivativeOrder::Second, &cancel).unwrap();
    assert_eq!(second.outputs(), &[0]);
    assert_eq!(second.coordinates(), &[0, n - 1]);
    assert_eq!(
        second.support().second[0],
        [(0, 0), (0, n - 1), (n - 1, n - 1)].into_iter().collect()
    );
    assert!(second.derivative_operations() > 0);
    assert!(first.support().second.is_empty());
}

#[test]
fn selected_provider_demand_matches_fixed_coordinate_program_and_authored_partials() {
    crate::initialize().unwrap();
    let (mut spec, _, calls) = provider();
    spec.derivatives = DerivativeOrder::Value;
    spec.smoothness = DerivativeOrder::Value;
    let body = PreparedBody::new(
        2,
        4,
        vec![3],
        vec![
            Stage::Provider {
                spec: spec.clone(),
                partial: vec![],
                inputs: vec![1],
                outputs: vec![2],
                source: id(20),
            },
            block(
                vec![library::formal(0).unwrap() + library::formal(2).unwrap()],
                vec![3],
            ),
        ],
        DerivativeOrder::Second,
    )
    .unwrap();
    assert_eq!(
        body.provider_demands_for_selection(&[0], &[0], DerivativeOrder::Second)
            .unwrap()[&spec.key()],
        DerivativeOrder::Value
    );
    assert_eq!(
        body.provider_demands_for_selection(&[0], &[1], DerivativeOrder::First)
            .unwrap()[&spec.key()],
        DerivativeOrder::First
    );
    let cancel = Arc::new(AtomicBool::new(false));
    let mut worker = body
        .compile(
            &[0],
            &[0],
            DerivativeOrder::Second,
            Optimization::default(),
            EvaluationLimits::default(),
            &cancel,
        )
        .unwrap()
        .worker();
    let selected_provider: Box<dyn Provider> = Box::new(Cubic {
        spec: spec.clone(),
        calls: calls.clone(),
    });
    let mut providers = BTreeMap::from([(spec.key(), selected_provider)]);
    let result = worker
        .evaluate(&[3., 2.], DerivativeOrder::Second, &mut providers, &cancel)
        .unwrap();
    assert_eq!(result.values, [11.]);
    assert_eq!(result.jacobian, [1.]);
    assert_eq!(result.hessians, [0.]);
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    let error = body
        .compile(
            &[0],
            &[1],
            DerivativeOrder::First,
            Optimization::default(),
            EvaluationLimits::default(),
            &cancel,
        )
        .unwrap_err();
    assert!(
        matches!(error, MathError::DerivativeDemand { ref outputs, ref coordinates, requested: DerivativeOrder::First, available: DerivativeOrder::Value, .. } if outputs == &[0] && coordinates == &[1])
    );

    let (spec, _, _) = provider();
    let partial = PreparedBody::new(
        1,
        2,
        vec![1],
        vec![Stage::Provider {
            spec: spec.clone(),
            partial: vec![0],
            inputs: vec![0],
            outputs: vec![1],
            source: id(21),
        }],
        DerivativeOrder::First,
    )
    .unwrap();
    assert_eq!(
        partial
            .provider_demands_for_selection(&[0], &[], DerivativeOrder::Value)
            .unwrap()[&spec.key()],
        DerivativeOrder::First
    );
    assert_eq!(
        partial
            .provider_demands_for_selection(&[0], &[0], DerivativeOrder::First)
            .unwrap()[&spec.key()],
        DerivativeOrder::Second
    );
}

#[test]
fn selected_output_availability_and_incidence_ignore_unrelated_value_provider() {
    use crate::typed::{Binary, BodyBuilder, BodyLimits};
    use pse_quantity::{
        IndexSet,
        standard::{StandardInvariantChecker, standard_registry},
    };
    let registry = standard_registry().unwrap();
    let (mut spec, _, _) = provider();
    spec.derivatives = DerivativeOrder::Value;
    spec.smoothness = DerivativeOrder::Value;
    let admitted = AdmittedProvider::new(spec.clone(), &registry).unwrap();
    let mut builder = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &StandardInvariantChecker,
        1,
        BodyLimits::default(),
    )
    .unwrap();
    let x = builder
        .input(0, spec.inputs[0].quantity, IndexSet::new(), id(1))
        .unwrap();
    let opaque = builder
        .provider(&admitted, std::slice::from_ref(&x), id(2))
        .unwrap()
        .remove(0);
    let square = builder
        .binary(Binary::Mul, x.clone(), x, None, id(3))
        .unwrap();
    let canceled = builder
        .binary(Binary::Sub, opaque.clone(), opaque.clone(), None, id(4))
        .unwrap();
    let effectful_square = builder
        .binary(Binary::Add, square.clone(), canceled, None, id(5))
        .unwrap();
    let body = builder
        .prepare(&[opaque, square, effectful_square])
        .unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    assert_eq!(
        body.available_order_for_outputs(&[1], &[0]).unwrap(),
        DerivativeOrder::Second
    );
    assert_eq!(
        body.available_order_for_outputs(&[0], &[0]).unwrap(),
        DerivativeOrder::Value
    );
    assert!(
        body.provider_demands_for_outputs(&[1], DerivativeOrder::Second)
            .unwrap()
            .is_empty()
    );
    let incidence = body.incidence(&[0], &[0], &cancel).unwrap();
    assert_eq!(incidence.support().first[0], [0].into_iter().collect());
    assert!(incidence.support().second.is_empty());
    assert!(
        incidence
            .compile(
                Optimization::default(),
                EvaluationLimits::default(),
                &cancel
            )
            .is_err()
    );
    let selected = body
        .prepare_support(&[1], &[0], DerivativeOrder::Second, &cancel)
        .unwrap();
    assert_eq!(selected.outputs(), &[1]);
    assert_eq!(selected.support().second[0], [(0, 0)].into_iter().collect());
    let mut worker = selected
        .compile(
            Optimization::default(),
            EvaluationLimits::default(),
            &cancel,
        )
        .unwrap()
        .worker();
    let result = worker
        .evaluate(
            &[3.0],
            DerivativeOrder::Second,
            &mut BTreeMap::new(),
            &cancel,
        )
        .unwrap();
    assert_eq!(result.values, [9.0]);
    assert_eq!(result.jacobian, [6.0]);
    assert_eq!(result.hessians, [2.0]);
    // Algebraic cancellation keeps the provider's fallible Value effect without
    // demanding numerical provider derivatives for the surviving square.
    assert_eq!(
        body.available_order_for_outputs(&[2], &[0]).unwrap(),
        DerivativeOrder::Second
    );
    assert_eq!(
        body.provider_demands_for_outputs(&[2], DerivativeOrder::Second)
            .unwrap()[&spec.key()],
        DerivativeOrder::Value
    );
    let calls = Arc::new(AtomicUsize::new(0));
    let provider: Box<dyn Provider> = Box::new(Cubic {
        spec,
        calls: calls.clone(),
    });
    let mut providers = BTreeMap::from([(provider.spec().key(), provider)]);
    let mut effect_worker = body
        .compile(
            &[2],
            &[0],
            DerivativeOrder::Second,
            Optimization::default(),
            EvaluationLimits::default(),
            &cancel,
        )
        .unwrap()
        .worker();
    let result = effect_worker
        .evaluate(&[3.0], DerivativeOrder::Second, &mut providers, &cancel)
        .unwrap();
    assert_eq!(result.values, [9.0]);
    assert_eq!(result.jacobian, [6.0]);
    assert_eq!(result.hessians, [2.0]);
    assert_eq!(calls.load(Ordering::Relaxed), 1);
}

#[test]
fn opaque_class_evidence_remains_pending_after_actual_bounded_request() {
    use crate::{
        assembly::{AssemblyLimits, CasePlan},
        binding::*,
        presolve::{ClassDependency, ClassEvidence, ClassRequest, ClassStatus},
        typed::{BodyBuilder, BodyLimits},
    };
    use pse_model::generated::enums::ModelingVariableDomain;
    use pse_quantity::{
        IndexSet,
        standard::{StandardInvariantChecker, standard_registry},
    };
    let registry = standard_registry().unwrap();
    let (spec, _, _) = provider();
    let admitted = AdmittedProvider::new(spec.clone(), &registry).unwrap();
    let mut builder = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &StandardInvariantChecker,
        1,
        BodyLimits::default(),
    )
    .unwrap();
    let x = builder
        .input(0, spec.inputs[0].quantity, IndexSet::new(), id(1))
        .unwrap();
    let opaque = builder.provider(&admitted, &[x], id(2)).unwrap().remove(0);
    let body = Arc::new(builder.prepare(&[opaque]).unwrap());
    let key = ContentHash::from_bytes([4; 32]);
    let structure = CaseStructure::new(
        vec![Variable {
            port: spec.inputs[0].clone(),
            fixed: false,
            domain: ModelingVariableDomain::Continuous,
            lower: None,
            upper: None,
        }],
        vec![],
        vec![InstanceBinding {
            instance: id(3),
            body: key,
            checked_members: Default::default(),
            slots: (vec![SlotBinding::new(&spec.inputs[0], &spec.inputs[0], &registry).unwrap()])
                .into(),
            contributions: vec![Contribution {
                output: 0,
                target: Target::PRIMARY,
                scale: 1.0,
            }],
        }],
        vec![],
        Some(Objective {
            quantity: spec.outputs[0].quantity,
            sense: ObjectiveSense::Minimize,
        }),
        CaseLimits::default(),
    )
    .unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let values = CaseValues::default();
    let plan = CasePlan::prepare(
        Arc::new(structure),
        BTreeMap::from([(key, body)]),
        &registry,
        DerivativeOrder::Value,
        AssemblyLimits::default(),
        &cancel,
    )
    .unwrap();
    let base = plan
        .presolve_domain_facts(&values, 10_000, &cancel)
        .unwrap();
    let ClassEvidence::Pending {
        facts,
        dependencies,
    } = plan
        .class_evidence(&values, &base, ClassRequest::Coefficients, 10_000, &cancel)
        .unwrap()
    else {
        panic!("opaque scientific class must remain unresolved");
    };
    assert_eq!(
        dependencies,
        [ClassDependency::MissingSymbolicExpression {
            instance: id(3),
            output: 0
        }]
    );
    assert_eq!(facts.class_status, ClassStatus::Pending(dependencies));
    assert_eq!(facts.objective_degree, None);
    assert!(!facts.coefficient_eligible());
    assert!(facts.proof_remaining < base.proof_remaining);
    assert_eq!(base.class_status, ClassStatus::Unassessed);
}

#[test]
fn demanded_directional_taylor_axis_preserves_all_support_under_two_component_limit() {
    crate::initialize().unwrap();
    let n = 12;
    let sum = (0..n).fold(Atom::num(0), |sum, i| sum + library::formal(i).unwrap());
    let body = PreparedBody::new(
        n,
        n + 1,
        vec![n],
        vec![block(vec![&sum * &sum], vec![n])],
        DerivativeOrder::Second,
    )
    .unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let support = body
        .prepare_support(
            &[0],
            &(0..n).collect::<Vec<_>>(),
            DerivativeOrder::First,
            &cancel,
        )
        .unwrap();
    let limits = EvaluationLimits {
        derivative_components: 2,
        ..EvaluationLimits::default()
    };
    assert!(matches!(
        support.compile(Optimization::default(), limits, &cancel),
        Err(MathError::Limit("derivative components"))
    ));
    let action = support
        .compile_directional(Optimization::default(), limits, &cancel)
        .unwrap();
    assert!(action.is_directional());
    assert_eq!(action.coordinates(), (0..n).collect::<Vec<_>>());
    assert_eq!(action.support().first[0], (0..n).collect::<BTreeSet<_>>());
    let inputs = (0..n).map(|i| i as f64 - 2.0).collect::<Vec<_>>();
    let direction = (0..n)
        .map(|i| if i % 2 == 0 { i as f64 } else { -2.0 })
        .collect::<Vec<_>>();
    let reference = support
        .compile(
            Optimization::default(),
            EvaluationLimits::default(),
            &cancel,
        )
        .unwrap()
        .worker()
        .evaluate(
            &inputs,
            DerivativeOrder::First,
            &mut BTreeMap::new(),
            &cancel,
        )
        .unwrap();
    let mut worker = action.worker();
    let result = worker
        .evaluate_directional(&inputs, &direction, &mut BTreeMap::new(), &cancel)
        .unwrap();
    assert_eq!(result.values, reference.values);
    assert_eq!(
        result.jacobian,
        vec![
            reference
                .jacobian
                .iter()
                .zip(&direction)
                .map(|(a, b)| a * b)
                .sum::<f64>()
        ]
    );
    assert!(result.hessians.is_empty());
    assert!(
        worker
            .evaluate(
                &inputs,
                DerivativeOrder::First,
                &mut BTreeMap::new(),
                &cancel
            )
            .is_err()
    );
    assert!(
        worker
            .evaluate_directional(&inputs, &direction[..n - 1], &mut BTreeMap::new(), &cancel)
            .is_err()
    );
    cancel.store(true, Ordering::Relaxed);
    assert!(
        worker
            .evaluate_directional(&inputs, &direction, &mut BTreeMap::new(), &cancel)
            .is_err()
    );
}

#[test]
fn demanded_directional_provider_composition_keeps_original_guard_and_first_partials() {
    crate::initialize().unwrap();
    let (spec, provider, calls) = provider();
    let x = library::formal(0).unwrap();
    let y = library::formal(1).unwrap();
    let body = PreparedBody::new(
        2,
        5,
        vec![4],
        vec![
            Stage::Require {
                argument: 1,
                condition: Condition::Positive,
                order: DerivativeOrder::First,
                source: id(88),
                lineage: None,
            },
            block(vec![&x * &x + &y], vec![2]),
            Stage::Provider {
                partial: vec![],
                spec: spec.clone(),
                inputs: vec![2],
                outputs: vec![3],
                source: id(21),
            },
            block(vec![library::formal(3).unwrap() + &x], vec![4]),
        ],
        DerivativeOrder::Second,
    )
    .unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let support = body
        .prepare_support(&[0], &[0, 1], DerivativeOrder::First, &cancel)
        .unwrap();
    let limits = EvaluationLimits {
        derivative_components: 2,
        ..EvaluationLimits::default()
    };
    let mut worker = support
        .compile_directional(Optimization::default(), limits, &cancel)
        .unwrap()
        .worker();
    let mut providers = BTreeMap::from([(spec.key(), provider)]);
    let result = worker
        .evaluate_directional(&[2.0, 3.0], &[4.0, -1.0], &mut providers, &cancel)
        .unwrap();
    // (x²+y)^3+x, DF*v=3*(7²)*(2*x*4-1)+4.
    assert_eq!(result.values, [345.0]);
    assert_eq!(result.jacobian, [2209.0]);
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert!(
        matches!(worker.evaluate_directional(&[2.0,-3.0],&[4.0,-1.0],&mut providers,&cancel),Err(MathError::Domain { source_id,.. }) if source_id==id(88))
    );
    assert_eq!(
        calls.load(Ordering::Relaxed),
        1,
        "guard refuses before provider execution"
    );
    assert!(
        matches!(worker.evaluate_directional(&[0.0,0.0],&[4.0,-1.0],&mut providers,&cancel),Err(MathError::Domain { source_id,.. }) if source_id==id(88))
    );
    assert_eq!(
        worker
            .evaluate_directional(&[1.0, 2.0], &[0.0, 2.0], &mut providers, &cancel)
            .unwrap()
            .jacobian,
        [54.0]
    );
}

#[test]
fn structural_taylor_zeros_preserve_complete_second_shape_and_reduce_library_work() {
    crate::initialize().unwrap();
    let x = library::formal(0).unwrap();
    let y = library::formal(1).unwrap();
    let limits = EvaluationLimits::default();
    let layout = crate::jets::JetLayout::new(vec![0, 1], DerivativeOrder::Second, limits).unwrap();
    let expressions = [(&x * &y) + x.sin() + (&y * &y)];
    let parameters = [x, y];
    let cancel = Arc::new(AtomicBool::new(false));
    let build = |zeros: &[(usize, usize)]| {
        library::bounded_evaluator(
            id(1),
            &expressions,
            &parameters,
            &layout,
            zeros,
            Optimization::default(),
            &cancel,
            limits,
            limits.operations,
            0,
        )
        .unwrap()
    };
    let mut dense = build(&[]);
    let mut sparse = build(&[(0, 2), (0, 4), (0, 5), (1, 1), (1, 3), (1, 4)]);
    let work = |e: &symbolica::evaluate::ExpressionEvaluator<f64>| {
        let c = e.count_operations();
        c.additions + c.multiplications + c.inversions + c.function_calls
    };
    assert!(work(&sparse) < work(&dense));
    assert_eq!(sparse.get_input_len(), 12);
    assert_eq!(sparse.get_output_len(), 6);
    let input = [2.0, 1.0, 0.0, 0.0, 0.0, 0.0, 3.0, 0.0, 1.0, 0.0, 0.0, 0.0];
    let mut full = [0.0; 6];
    let mut suppressed = [0.0; 6];
    dense.evaluate(&input, &mut full);
    sparse.evaluate(&input, &mut suppressed);
    let expected = [
        15.0 + 2.0_f64.sin(),
        3.0 + 2.0_f64.cos(),
        8.0,
        -2.0_f64.sin(),
        1.0,
        2.0,
    ];
    for component in 0..6 {
        let factor = layout.raw_factor(component);
        assert!((suppressed[component] * factor - expected[component]).abs() < 1e-13);
        assert!((suppressed[component] - full[component]).abs() < 1e-13);
    }
}

#[test]
fn conditional_block_keeps_actual_guard_provider_execution_inputs() {
    use crate::{
        assembly::{AssemblyLimits, CasePlan},
        binding::*,
        typed::{Binary, BodyBuilder, BodyLimits},
    };
    use pse_model::generated::enums::ModelingVariableDomain;
    use pse_quantity::{
        IndexSet,
        standard::{StandardInvariantChecker, ids, standard_registry},
    };
    let registry = standard_registry().unwrap();
    let quantity = ids::quantity("neutral");
    let port = |n| Port {
        id: id(n),
        quantity,
        unit: registry.quantity_type(quantity).unwrap().canonical_unit,
    };
    let (mut spec, _, calls) = provider();
    spec.derivatives = DerivativeOrder::Value;
    spec.smoothness = DerivativeOrder::Value;
    let admitted = AdmittedProvider::new(spec.clone(), &registry).unwrap();
    let mut builder = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &StandardInvariantChecker,
        2,
        BodyLimits::default(),
    )
    .unwrap();
    let x = builder.input(0, quantity, IndexSet::new(), id(30)).unwrap();
    let guard_input = builder.input(1, quantity, IndexSet::new(), id(31)).unwrap();
    let assumption = builder
        .domain(form_lineage(id(32)), |builder| {
            Ok(builder
                .provider(&admitted, std::slice::from_ref(&guard_input), id(33))?
                .remove(0))
        })
        .unwrap();
    let square = builder
        .binary(Binary::Mul, x.clone(), x, None, id(34))
        .unwrap();
    let guarded = builder.with_assumption(square.clone(), &assumption);
    let body = Arc::new(builder.prepare(&[guarded, square]).unwrap());
    let key = ContentHash::from_bytes([211; 32]);
    let structure = Arc::new(
        CaseStructure::new(
            (1..=2)
                .map(|n| Variable {
                    port: port(n),
                    fixed: false,
                    domain: ModelingVariableDomain::Continuous,
                    lower: None,
                    upper: None,
                })
                .collect(),
            vec![],
            vec![InstanceBinding {
                instance: id(40),
                body: key,
                checked_members: Default::default(),
                slots: vec![
                    SlotBinding::new(&port(1), &port(1), &registry).unwrap(),
                    SlotBinding::new(&port(2), &port(2), &registry).unwrap(),
                ]
                .into(),
                contributions: (0..2)
                    .map(|output| Contribution {
                        output,
                        target: Target::Row(id(60 + output as u8)),
                        scale: 1.0,
                    })
                    .collect(),
            }],
            (60..=61)
                .map(|n| Row {
                    id: id(n),
                    quantity,
                    lower: 0.0,
                    upper: 0.0,
                })
                .collect(),
            None,
            CaseLimits::default(),
        )
        .unwrap(),
    );
    let cancel = Arc::new(AtomicBool::new(false));
    let source = CasePlan::prepare(
        structure,
        BTreeMap::from([(key, body)]),
        &registry,
        DerivativeOrder::First,
        AssemblyLimits::default(),
        &cancel,
    )
    .unwrap();
    let selected = |row| {
        source
            .conditional(
                &[id(row)].into_iter().collect(),
                &[id(1)].into_iter().collect(),
                &registry,
                &cancel,
            )
            .unwrap()
    };
    let guarded = selected(60);
    let dependency = guarded.dependencies(&cancel).unwrap().remove(0);
    assert!(dependency.execution.contains(&id(2)));
    assert!(!dependency.numerical.contains(&id(2)));
    assert_eq!(guarded.structure().instances()[0].slots.len(), 2);
    let guarded = Arc::new(
        Arc::new(guarded)
            .compile(
                Optimization::default(),
                EvaluationLimits::default(),
                &cancel,
            )
            .unwrap(),
    );
    let selected_provider: Box<dyn Provider> = Box::new(Cubic {
        spec: spec.clone(),
        calls: calls.clone(),
    });
    let providers = BTreeMap::from([(spec.key(), selected_provider)]);
    let mut worker = guarded.worker(providers, cancel.clone());
    let mut values = CaseValues {
        scalars: BTreeMap::from([(id(1), 3.0), (id(2), 2.0)]),
    };
    assert_eq!(worker.constraints(&values).unwrap(), [9.0]);
    assert_eq!(worker.jacobian(&values).unwrap().val(), &[6.0]);
    values.scalars.remove(&id(2));
    assert!(worker.constraints(&values).is_err());
    values.scalars.insert(id(2), -2.0);
    assert!(worker.constraints(&values).is_err());
    let before = calls.load(Ordering::Relaxed);
    let plain = Arc::new(
        Arc::new(selected(61))
            .compile(
                Optimization::default(),
                EvaluationLimits::default(),
                &cancel,
            )
            .unwrap(),
    );
    assert_eq!(
        plain
            .worker(BTreeMap::new(), cancel.clone())
            .constraints(&values)
            .unwrap(),
        [9.0]
    );
    assert_eq!(calls.load(Ordering::Relaxed), before);
    values.scalars.insert(id(2), 2.0);
    assert_eq!(worker.constraints(&values).unwrap(), [9.0]);
}
