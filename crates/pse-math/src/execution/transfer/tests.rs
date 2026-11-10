// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use pse_model::{
    applicability::{Claim, Node, Region},
    generated::enums::ModelingValidityLayer,
};
use std::{cell::Cell, sync::atomic::AtomicUsize};

#[derive(Debug)]
struct Construction;
impl ConstructionAdmission for Construction {
    fn try_grow(&self, _: usize) -> std::result::Result<(), MathError> {
        Ok(())
    }
}
fn scope() -> pse_kernels::ExecutionScope {
    pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::new(false), None)
}
fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
fn block(expression: Atom, output: usize, source: u8) -> Stage {
    Stage::Block {
        expressions: vec![expression],
        outputs: vec![output],
        source: id(source),
    }
}
fn fixture(nested: bool) -> CompiledBody {
    fixture_with_predicate(nested, 0)
}
fn fixture_with_predicate(nested: bool, predicate_offset: i32) -> CompiledBody {
    crate::initialize().unwrap();
    let x = library::formal(31).unwrap();
    let y = library::formal(127).unwrap();
    let guard = library::formal(255).unwrap();
    let lineage = Arc::new(pse_model::diagnostic::ValidityLineage {
        layer: ModelingValidityLayer::Form,
        source: id(9),
        form: Some(id(10)),
        sets: vec![id(11)],
        variables: vec![0],
        members: vec![],
    });
    let claim = Claim {
        id: Some(id(12)),
        coverage: Some(id(20)),
        owner: id(13),
        owner_lineage: vec![id(13), id(14)],
        evidence: Some(id(15)),
        form: id(16),
        call: id(17),
        records: vec![id(18)],
        dependencies: vec![id(23)],
        layer: ModelingValidityLayer::Data,
        basis: None,
        reason: Some("original evidence".into()),
    };
    let child = |n, region| Node {
        claim: Claim {
            id: Some(id(n)),
            dependencies: vec![],
            ..claim.clone()
        },
        region,
        dependencies: vec![],
        inputs: vec![(format!("child{n}"), 0, id(19))],
        permissions: vec![],
    };
    let plan = Arc::new(Node {
        region: Region::Union(vec![
            child(21, Region::Unknown),
            child(22, Region::Predicate(0)),
        ]),
        dependencies: vec![child(23, Region::Unrestricted)],
        inputs: vec![("guard".into(), 0, id(19))],
        permissions: vec![pse_model::applicability::Permission {
            id: id(24),
            scope: id(25),
            target_kind: pse_model::generated::enums::ModelingPermissionTarget::Records,
            targets: vec![id(18)],
            allow_unknown: false,
            allow_extrapolation: false,
        }],
        claim,
    });
    let stages = if nested {
        vec![
            Stage::Domain {
                stages: vec![block(guard.clone(), 3000, 1)],
                argument: 3000,
                token: 3001,
                lineage: lineage.clone(),
            },
            Stage::Applicability {
                stages: vec![block(guard - Atom::num(predicate_offset), 3002, 2)],
                predicates: vec![3002],
                inputs: vec![255],
                token: 3003,
                plan,
            },
            block(Atom::num(0), 3004, 3),
            Stage::Branch {
                continuity: DerivativeOrder::Value,
                comparison: Comparison::Lt,
                left: 3004,
                right: 31,
                then: vec![block(&x * &x, 2048, 4)],
                otherwise: vec![block(&y * &y, 2048, 5)],
            },
        ]
    } else {
        vec![
            Stage::Require {
                argument: 255,
                condition: Condition::Positive,
                order: DerivativeOrder::Value,
                source: id(6),
                lineage: Some(lineage),
            },
            block(&x * &x + &x * &y, 2048, 7),
            block(&y * &y, 2049, 8),
        ]
    };
    let body = PreparedBody::new(
        256,
        4096,
        if nested { vec![2048] } else { vec![2048, 2049] },
        stages,
        DerivativeOrder::First,
    )
    .unwrap();
    body.compile_branch_local(
        if nested { &[0] } else { &[1, 0] },
        &[31, 127],
        DerivativeOrder::First,
        Optimization::default(),
        EvaluationLimits::default(),
        &scope(),
    )
    .unwrap()
}
fn snapshot(body: &CompiledBody) -> ExecutionSnapshot {
    capture(body, &scope(), |_| Ok::<_, Infallible>(Arc::new(())))
        .unwrap()
        .into_parts()
        .0
}
fn restored(snapshot: ExecutionSnapshot) -> CheckedExecution {
    restore(snapshot, scope(), Arc::new(Construction), |_| {
        Ok::<_, Infallible>(Arc::new(()))
    })
    .unwrap()
}
fn restore<E: std::fmt::Debug + std::fmt::Display>(
    snapshot: ExecutionSnapshot,
    scope: pse_kernels::ExecutionScope,
    construction: Arc<dyn ConstructionAdmission>,
    admit: impl FnOnce(&TransferExtents) -> std::result::Result<Arc<dyn AllocationOwner>, E>,
) -> std::result::Result<CheckedExecution, TransferError<E>> {
    super::restore(
        CapturedExecution::from_admitted_snapshot(snapshot, Arc::new(())),
        scope,
        construction,
        admit,
    )
}
fn values(frame: &mut NumericFrame, input: &[f64], order: DerivativeOrder) -> Evaluation {
    frame
        .evaluate(
            input,
            order,
            &mut ProviderBindings::new(),
            scope().cancellation(),
        )
        .unwrap()
}
#[test]
fn real_sparse_asymmetric_root_and_nested_both_arms_value_first_roundtrip() {
    for nested in [false, true] {
        let body = fixture(nested);
        let mut original = body.numeric_frame_scoped(scope());
        let passive = snapshot(&body);
        assert_eq!(passive.input_formals, [31, 127, 255]);
        assert!(passive.first_layout.slot_offsets.is_some());
        let facts = passive.inspect().unwrap();
        assert!(facts.library_instructions > 0 && facts.library_operations > 0);
        let mut checked = restored(passive.clone());
        assert_eq!(checked.input_formals(), [31, 127, 255]);
        for input in [[2., 3., 1.], [-2., 3., 1.], [4., 5., 2.]] {
            for order in [DerivativeOrder::Value, DerivativeOrder::First] {
                let before = values(&mut original, &input, order);
                let after = values(&mut checked.frame, &input, order);
                assert_eq!(before, after);
            }
        }
        assert_eq!(
            capture_frame(&checked.frame, &scope(), |_| Ok::<_, Infallible>(Arc::new(
                ()
            )))
            .unwrap()
            .snapshot(),
            &passive,
            "capture omits warmed state"
        );
    }
}

#[test]
fn immutable_template_drops_old_scope_and_admits_independent_cold_workers() {
    let old_cancel = pse_diagnostics::Cancellation::new(false);
    let old_scope = pse_kernels::ExecutionScope::new(old_cancel.clone(), None);
    let mut original = restore(
        snapshot(&fixture(false)),
        old_scope,
        Arc::new(Construction),
        |_| Ok::<_, Infallible>(Arc::new(())),
    )
    .unwrap();
    assert_eq!(
        original
            .evaluate(&[2., 3., 1.], DerivativeOrder::First)
            .unwrap()
            .values,
        [9., 10.]
    );
    let template = original.into_template();
    old_cancel.cancel();
    let first_cancel = pse_diagnostics::Cancellation::new(false);
    let first_scope = pse_kernels::ExecutionScope::new(first_cancel.clone(), None);
    let admissions = Cell::new(0);
    let admit = |bytes| -> std::result::Result<Arc<dyn AllocationOwner>, Infallible> {
        assert_eq!(bytes, template.extents().worker_bytes);
        admissions.set(admissions.get() + 1);
        Ok(Arc::new(()))
    };
    let mut first = template.worker_scoped(first_scope, admit).unwrap();
    let mut second = template.worker_scoped(scope(), admit).unwrap();
    assert_eq!(admissions.get(), 2);
    assert!(Arc::ptr_eq(&first.frame.template, &second.frame.template));
    assert!(first.frame.frame.is_empty() && second.frame.frame.is_empty());
    assert!(first.frame.programs.values().all(Option::is_none));
    let one = first
        .evaluate(&[4., 5., 1.], DerivativeOrder::First)
        .unwrap();
    assert_eq!(one.values, [25., 36.]);
    assert_eq!(one.jacobian, [0., 10., 13., 4.]);
    assert!(second.frame.programs.values().all(Option::is_none));
    let two = second
        .evaluate(&[2., 3., 1.], DerivativeOrder::First)
        .unwrap();
    assert_eq!(two.values, [9., 10.]);
    assert_eq!(two.jacobian, [0., 6., 7., 2.]);
    first_cancel.cancel();
    assert!(matches!(
        first.evaluate(&[4., 5., 1.], DerivativeOrder::First),
        Err(MathError::Cancelled)
    ));
    assert_eq!(
        second
            .evaluate(&[2., 3., 1.], DerivativeOrder::First)
            .unwrap(),
        two
    );
    let refused = template.worker_scoped(scope(), |_| Err("worker admission"));
    assert!(matches!(
        refused,
        Err(TransferError::Admission("worker admission"))
    ));
    let stopped = pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::new(true), None);
    assert!(matches!(
        template.worker_scoped(
            stopped,
            |_| -> std::result::Result<Arc<dyn AllocationOwner>, Infallible> {
                panic!("stopped worker must not request backing")
            }
        ),
        Err(TransferError::Math(MathError::Cancelled))
    ));
}

#[test]
fn immutable_template_and_each_worker_retain_allocation_owners_until_drain() {
    #[derive(Debug)]
    struct Owner;
    let restored_owner = Arc::new(Owner);
    let restored_weak = Arc::downgrade(&restored_owner);
    let checked = restore(
        snapshot(&fixture(false)),
        scope(),
        Arc::new(Construction),
        |_| Ok::<Arc<dyn AllocationOwner>, Infallible>(restored_owner.clone()),
    )
    .unwrap();
    drop(restored_owner);
    let template = checked.into_template();
    let worker_owner = Arc::new(Owner);
    let worker_weak = Arc::downgrade(&worker_owner);
    let worker = template
        .worker_scoped(scope(), |_| {
            Ok::<Arc<dyn AllocationOwner>, Infallible>(worker_owner.clone())
        })
        .unwrap();
    drop(worker_owner);
    drop(template);
    assert!(restored_weak.upgrade().is_some());
    assert!(worker_weak.upgrade().is_some());
    drop(worker);
    assert!(restored_weak.upgrade().is_none());
    assert!(worker_weak.upgrade().is_none());
}

#[test]
fn immutable_template_releases_restoration_construction_and_cancellation_controls() {
    #[derive(Debug)]
    struct ControlledConstruction(pse_diagnostics::Cancellation);
    impl ConstructionAdmission for ControlledConstruction {
        fn try_grow(&self, _: usize) -> std::result::Result<(), MathError> {
            if self.0.is_cancelled() {
                Err(MathError::Cancelled)
            } else {
                Ok(())
            }
        }
    }
    let stopped = pse_diagnostics::Cancellation::new(false);
    let weak_control = Arc::downgrade(&stopped.flag());
    let checked = restore(
        snapshot(&fixture(false)),
        pse_kernels::ExecutionScope::new(stopped.clone(), None),
        Arc::new(ControlledConstruction(stopped.clone())),
        |_| Ok::<_, Infallible>(Arc::new(())),
    )
    .unwrap();
    drop(stopped);
    assert!(weak_control.upgrade().is_some());
    let template = checked.into_template();
    assert!(
        weak_control.upgrade().is_none(),
        "immutable publication must drop restoration control anchors"
    );
    let mut worker = template
        .worker_scoped(scope(), |_| Ok::<_, Infallible>(Arc::new(())))
        .unwrap();
    assert_eq!(
        worker
            .evaluate(&[2., 3., 1.], DerivativeOrder::First)
            .unwrap()
            .values,
        [9., 10.]
    );
}

#[test]
fn selected_second_transfer_preserves_reordered_axes_outputs_and_pair_semantics() {
    crate::initialize().unwrap();
    let x = library::formal(7).unwrap();
    let y = library::formal(31).unwrap();
    let body = PreparedBody::new(
        32,
        1026,
        vec![1024, 1025, 1025],
        vec![
            block(&x * &x * &x * &y + &x, 1024, 61),
            block(&y * &y, 1025, 62),
        ],
        DerivativeOrder::Second,
    )
    .unwrap()
    .compile(
        &[1, 0, 2],
        &[31, 7],
        DerivativeOrder::Second,
        Optimization::default(),
        EvaluationLimits::default(),
        &scope(),
    )
    .unwrap();
    let passive = snapshot(&body);
    assert!(passive.second.is_some());
    assert_eq!(passive.coordinate_offsets, [1, 0]);
    let mut received = restored(passive.clone())
        .into_template()
        .worker_scoped(scope(), |_| Ok::<_, Infallible>(Arc::new(())))
        .unwrap();
    let result = received
        .evaluate(&[2., 3.], DerivativeOrder::Second)
        .unwrap();
    assert_eq!(result.values, [9., 26., 9.]);
    assert_eq!(result.jacobian, [6., 0., 8., 37., 6., 0.]);
    assert_eq!(
        result.hessians,
        [2., 0., 0., 0., 0., 12., 12., 36., 2., 0., 0., 0.]
    );
    for mutation in [0, 1, 2] {
        let mut corrupt = passive.clone();
        let second = corrupt.second.as_mut().unwrap();
        match mutation {
            0 => second.layout.pairs.swap(0, 1),
            1 => second.layout.shape.last_mut().unwrap()[0] = 9,
            _ => second.layout.order = DerivativeOrder::First,
        }
        assert!(
            restore(
                corrupt,
                scope(),
                Arc::new(Construction),
                |_| -> std::result::Result<Arc<dyn AllocationOwner>, Infallible> {
                    panic!("malformed Second layout must refuse before native admission")
                }
            )
            .is_err()
        );
    }
}

#[test]
fn stable_second_transfer_executes_closed_heads_after_producer_retirement() {
    use crate::stable::{self, Operation};
    crate::initialize().unwrap();
    let x = library::formal(0).unwrap();
    let y = library::formal(1).unwrap();
    let log = stable::call(Operation::Log1p, &[&x]).unwrap();
    let magnitude = stable::call(Operation::Magnitude, &[&x, &y]).unwrap();
    let source = PreparedBody::new(
        2,
        4,
        vec![2, 3],
        vec![block(log, 2, 81), block(magnitude, 3, 82)],
        DerivativeOrder::Second,
    )
    .unwrap()
    .compile(
        &[1, 0],
        &[1, 0],
        DerivativeOrder::Second,
        Optimization::default(),
        EvaluationLimits::default(),
        &scope(),
    )
    .unwrap();
    let passive = snapshot(&source);
    drop(source);
    let template = restored(passive).into_template();
    let mut worker = template
        .worker_scoped(scope(), |_| Ok::<_, Infallible>(Arc::new(())))
        .unwrap();
    let point = worker.evaluate(&[3., 4.], DerivativeOrder::Second).unwrap();
    assert_eq!(point.values[0], 5.);
    assert!((point.values[1] - 4_f64.ln()).abs() < 1e-15);
    let close = |actual: &[f64], expected: &[f64]| {
        assert_eq!(actual.len(), expected.len());
        for (&actual, &expected) in actual.iter().zip(expected) {
            assert!((actual - expected).abs() < 2e-15, "{actual} != {expected}");
        }
    };
    close(&point.jacobian, &[0.8, 0.6, 0., 0.25]);
    close(
        &point.hessians,
        &[0.072, -0.096, -0.096, 0.128, 0., 0., 0., -0.0625],
    );
    let tiny = worker
        .evaluate(&[1e-20, 1.], DerivativeOrder::Value)
        .unwrap();
    assert_eq!(tiny.values[1].to_bits(), 1e-20_f64.ln_1p().to_bits());
    assert_ne!(tiny.values[1], (1.0 + 1e-20_f64).ln());
}

#[test]
fn input_constant_outputs_allow_empty_arithmetic_programs() {
    crate::initialize().unwrap();
    let body = PreparedBody::new(1, 1, vec![0], vec![], DerivativeOrder::First).unwrap();
    let body = body
        .compile(
            &[0],
            &[0],
            DerivativeOrder::First,
            Optimization::default(),
            EvaluationLimits::default(),
            &scope(),
        )
        .unwrap();
    let passive = snapshot(&body);
    assert!(passive.value.is_empty() && passive.first.is_empty());
    assert_eq!(passive.coordinate_offsets, [0]);
    assert_eq!(passive.first_rows, [(0, 1)]);
    assert_eq!(passive.first_axes, [0]);
    assert_eq!(
        passive.first_layout.slot_components.as_deref(),
        Some([0, 1].as_slice())
    );
    let mut original = body.numeric_frame_scoped(scope());
    let before = values(&mut original, &[7.], DerivativeOrder::First);
    let mut checked = restored(passive);
    let actual = values(&mut checked.frame, &[7.], DerivativeOrder::First);
    assert_eq!(actual, before);
    assert_eq!(actual.values, [7.]);
    assert_eq!(
        before.jacobian,
        [1.],
        "original producer identity derivative"
    );
}
#[test]
fn producer_identity_mixed_constant_and_guarded_identity_keep_mathematical_derivatives() {
    crate::initialize().unwrap();
    for guarded in [false, true] {
        let mut stages = vec![];
        if guarded {
            stages.push(Stage::Require {
                argument: 1,
                condition: Condition::Positive,
                order: DerivativeOrder::Value,
                source: id(24),
                lineage: None,
            });
        }
        stages.push(block(Atom::num(7), 3, 25));
        let body = PreparedBody::new(2, 4, vec![0, 1, 3], stages, DerivativeOrder::Second).unwrap();
        for order in [
            DerivativeOrder::Value,
            DerivativeOrder::First,
            DerivativeOrder::Second,
        ] {
            let body = body
                .compile(
                    &[0, 1, 2],
                    &[0, 1],
                    order,
                    Optimization::default(),
                    EvaluationLimits::default(),
                    &scope(),
                )
                .unwrap();
            let mut frame = body.numeric_frame_scoped(scope());
            let result = values(&mut frame, &[2., 3.], order);
            assert_eq!(result.values, [2., 3., 7.]);
            if order >= DerivativeOrder::First {
                assert_eq!(result.jacobian, [1., 0., 0., 1., 0., 0.]);
            }
            if order >= DerivativeOrder::Second {
                assert_eq!(result.hessians, [0.; 12]);
            }
            if order == DerivativeOrder::First {
                let passive = snapshot(&body);
                let mut checked = restored(passive.clone());
                assert_eq!(values(&mut checked.frame, &[2., 3.], order), result);
                assert_eq!(
                    capture_frame(&checked.frame, &scope(), |_| Ok::<_, Infallible>(Arc::new(
                        ()
                    )))
                    .unwrap()
                    .snapshot(),
                    &passive
                );
            }
        }
    }
}

#[test]
fn physically_typed_identity_constant_and_guard_keep_value_first_second() {
    use crate::typed::{BodyBuilder, BodyLimits};
    use pse_quantity::{
        IndexSet,
        literal::LiteralContext,
        standard::{StandardInvariantChecker, ids, standard_registry},
    };
    let registry = standard_registry().unwrap();
    let quantity = ids::quantity("neutral");
    let unit = registry
        .unit(registry.quantity_type(quantity).unwrap().canonical_unit)
        .unwrap();
    for order in [
        DerivativeOrder::Value,
        DerivativeOrder::First,
        DerivativeOrder::Second,
    ] {
        let mut builder = BodyBuilder::new(
            crate::initialize().unwrap(),
            &registry,
            &StandardInvariantChecker,
            1,
            BodyLimits::default(),
        )
        .unwrap();
        let mut input = builder.input(0, quantity, IndexSet::new(), id(71)).unwrap();
        builder
            .require(
                &mut input,
                Condition::Positive,
                DerivativeOrder::Value,
                id(72),
            )
            .unwrap();
        let constant = builder
            .literal(
                7.,
                unit,
                LiteralContext::Explicit {
                    quantity_type: quantity,
                },
                id(73),
            )
            .unwrap();
        let body = builder
            .finish(&[input, constant], order, Optimization::default(), &scope())
            .unwrap();
        let mut original = body.numeric_frame_scoped(scope());
        let result = values(&mut original, &[2.], order);
        assert_eq!(result.values, [2., 7.]);
        if order >= DerivativeOrder::First {
            assert_eq!(result.jacobian, [1., 0.]);
        }
        if order >= DerivativeOrder::Second {
            assert_eq!(result.hessians, [0., 0.]);
        }
        if order == DerivativeOrder::First {
            let mut checked = restored(snapshot(&body));
            assert_eq!(values(&mut checked.frame, &[2.], order), result);
        }
    }
}

fn first_block(snapshot: &mut ExecutionSnapshot) -> &mut BlockSnapshot {
    snapshot
        .first
        .iter_mut()
        .find_map(|s| {
            if let StageSnapshot::Block(b) = s {
                Some(b)
            } else {
                None
            }
        })
        .unwrap()
}
#[test]
fn malformed_signature_layout_and_every_block_index_refuse_before_native_admission() {
    let original = snapshot(&fixture(false));
    type Mutation = (&'static str, fn(&mut ExecutionSnapshot));
    let mutations: &[Mutation] = &[
        ("old version", |s| s.version = 1),
        ("input count", |s| s.inputs = usize::MAX),
        ("coordinate offset", |s| {
            s.coordinate_offsets[0] = usize::MAX
        }),
        ("output slot", |s| s.outputs[0] = usize::MAX),
        ("first row start", |s| s.first_rows[0].0 = 1),
        ("first row end", |s| s.first_rows[0].1 = usize::MAX),
        ("first axis", |s| s.first_axes[0] = usize::MAX),
        ("shape extent", |s| s.first_layout.shape[0].push(0)),
        ("shape value", |s| s.first_layout.shape[0][0] = 1),
        ("layout coordinates", |s| {
            s.first_layout.coordinates[0] = usize::MAX
        }),
        ("layout order", |s| {
            s.first_layout.order = DerivativeOrder::Second
        }),
        ("layout pair", |s| s.first_layout.pairs.push((0, 0))),
        ("sparse offset", |s| {
            s.first_layout.slot_offsets.as_mut().unwrap()[0] = 1
        }),
        ("sparse end", |s| {
            *s.first_layout
                .slot_offsets
                .as_mut()
                .unwrap()
                .last_mut()
                .unwrap() = usize::MAX
        }),
        ("sparse component", |s| {
            s.first_layout.slot_components.as_mut().unwrap()[0] = usize::MAX
        }),
        ("missing sparse half", |s| {
            s.first_layout.slot_components = None
        }),
        ("argument extent", |s| first_block(s).arguments = usize::MAX),
        ("value extent", |s| first_block(s).values = usize::MAX),
        ("nonresettable", |s| first_block(s).resettable = false),
        ("empty components", |s| first_block(s).components.clear()),
        ("component index", |s| {
            first_block(s).components[0] = usize::MAX
        }),
        ("block input", |s| first_block(s).inputs[0] = usize::MAX),
        ("block output", |s| first_block(s).outputs[0] = usize::MAX),
        ("gather argument", |s| {
            first_block(s).gather[0].0 = usize::MAX
        }),
        ("gather source", |s| first_block(s).gather[0].1 = usize::MAX),
        ("scatter destination", |s| {
            first_block(s).scatter[0].0 = usize::MAX
        }),
        ("scatter value", |s| {
            first_block(s).scatter[0].1 = Some(usize::MAX)
        }),
        ("source check count", |s| first_block(s).checks.clear()),
        ("source check end", |s| {
            first_block(s).checks[0].end_output = usize::MAX
        }),
        ("checked end", |s| {
            first_block(s).checked_ends[0] = usize::MAX
        }),
        ("supplier header", |s| first_block(s).program[0] = 0),
        ("supplier overflow", |s| {
            first_block(s).program[28..36].copy_from_slice(&u64::MAX.to_le_bytes())
        }),
        ("require argument", |s| {
            if let StageSnapshot::Require { argument, .. } = &mut s.first[0] {
                *argument = usize::MAX;
            } else {
                panic!("fixture");
            }
        }),
    ];
    for (name, mutate) in mutations {
        let mut bad = original.clone();
        mutate(&mut bad);
        ALLOCATIONS.with(|n| n.set(0));
        let called = Cell::new(false);
        let result = restore(bad, scope(), Arc::new(Construction), |_| {
            called.set(true);
            Ok::<_, Infallible>(Arc::new(()))
        });
        assert!(result.is_err(), "{name}");
        assert!(!called.get(), "{name}");
        assert_eq!(ALLOCATIONS.with(Cell::get), 0, "{name}");
    }
}

#[test]
fn nested_frame_plan_and_branch_indices_are_checked_and_plan_uniqueness_uses_admitted_scratch() {
    let original = snapshot(&fixture(true));
    let mutations: &[fn(&mut ExecutionSnapshot)] = &[
        |s| {
            if let StageSnapshot::Domain(d) = &mut s.first[0] {
                d.frame = usize::MAX
            }
        },
        |s| {
            if let StageSnapshot::Domain(d) = &mut s.first[0] {
                d.reads[0] = usize::MAX
            }
        },
        |s| {
            if let StageSnapshot::Domain(d) = &mut s.first[0] {
                d.argument = usize::MAX
            }
        },
        |s| {
            if let StageSnapshot::Domain(d) = &mut s.first[0] {
                d.token = usize::MAX
            }
        },
        |s| {
            if let StageSnapshot::Applicability(a) = &mut s.first[1] {
                a.predicates[0] = usize::MAX
            }
        },
        |s| {
            if let StageSnapshot::Applicability(a) = &mut s.first[1] {
                a.inputs[0] = usize::MAX
            }
        },
        |s| {
            if let StageSnapshot::Applicability(a) = &mut s.first[1] {
                a.plan.inputs[0].1 = usize::MAX
            }
        },
        |s| {
            if let StageSnapshot::Applicability(a) = &mut s.first[1] {
                a.plan.claim.owner_lineage[0] = id(99)
            }
        },
        |s| {
            if let StageSnapshot::Branch { left, .. } = &mut s.first[3] {
                *left = usize::MAX
            }
        },
        |s| {
            if let StageSnapshot::Branch { right, .. } = &mut s.first[3] {
                *right = usize::MAX
            }
        },
    ];
    for mutate in mutations {
        let mut bad = original.clone();
        mutate(&mut bad);
        let called = Cell::new(false);
        assert!(
            restore(bad, scope(), Arc::new(Construction), |_| {
                called.set(true);
                Ok::<_, Infallible>(Arc::new(()))
            })
            .is_err()
        );
        assert!(!called.get());
    }
    let mut bad = original;
    let StageSnapshot::Applicability(a) = &mut bad.first[1] else {
        panic!("fixture");
    };
    a.plan.claim.owner_lineage.push(a.plan.claim.owner);
    use pse_contract::boundary_projection::ValidateProjection;
    assert!(
        bad.validate_projection().is_ok(),
        "passive field validation grants no semantic lineage validity"
    );
    let called = Cell::new(0);
    assert!(matches!(
        restore(bad, scope(), Arc::new(Construction), |_| {
            called.set(called.get() + 1);
            Ok::<_, Infallible>(Arc::new(()))
        }),
        Err(TransferError::Invalid("applicability lineage validation"))
    ));
    assert_eq!(called.get(), 1);
}

#[test]
fn fresh_structural_zeros_poison_and_derived_growth_are_observed_before_allocating() {
    fn cold(stages: &[CompiledStage]) {
        for stage in stages {
            match stage {
                CompiledStage::Block {
                    arguments,
                    values,
                    gather,
                    ..
                } => {
                    for (index, value) in arguments.iter().enumerate() {
                        if gather.iter().any(|&(i, _)| i == index) {
                            assert!(value.is_nan());
                        } else {
                            assert_eq!(value.to_bits(), 0);
                        }
                    }
                    assert!(values.iter().all(|v| v.is_nan()));
                }
                CompiledStage::Domain { frame, stages, .. }
                | CompiledStage::Applicability { frame, stages, .. } => {
                    assert!(frame.iter().all(|v| v.is_nan()));
                    cold(stages);
                }
                CompiledStage::Branch {
                    then, otherwise, ..
                } => {
                    cold(then);
                    cold(otherwise);
                }
                _ => {}
            }
        }
    }
    let passive = snapshot(&fixture(true));
    let checked = restored(passive);
    assert!(checked.frame.frame.is_empty());
    assert!(checked.frame.programs.values().all(Option::is_none));
    for stages in checked.frame.template.programs.values().flatten() {
        cold(stages);
    }
    let mut passive = snapshot(&fixture(false));
    let program = &mut first_block(&mut passive).program;
    program[28..36].copy_from_slice(&1_000_000u64.to_le_bytes());
    ALLOCATIONS.with(|n| n.set(0));
    let called = Cell::new(0);
    let result = restore(passive, scope(), Arc::new(Construction), |facts| {
        called.set(called.get() + 1);
        assert_eq!(ALLOCATIONS.with(Cell::get), 0);
        assert!(facts.template_bytes >= 8_000_000 && facts.worker_bytes >= 8_000_000);
        Err(MathError::Limit("actual native growth"))
    });
    assert!(matches!(
        result,
        Err(TransferError::Admission(MathError::Limit(
            "actual native growth"
        )))
    ));
    assert_eq!(called.get(), 1);
    assert_eq!(ALLOCATIONS.with(Cell::get), 0);
}

#[test]
fn native_allocation_failure_and_supplier_dataflow_keep_typed_causes() {
    let passive = snapshot(&fixture(false));
    ALLOCATIONS.with(|n| n.set(0));
    FAIL_ALLOCATION.with(|n| n.set(Some(1)));
    let result = restore(passive.clone(), scope(), Arc::new(Construction), |_| {
        Ok::<_, Infallible>(Arc::new(()))
    });
    FAIL_ALLOCATION.with(|n| n.set(None));
    assert!(matches!(result, Err(TransferError::Allocation(_))));
    let mut bad = passive;
    let block = first_block(&mut bad);
    let extents = F64ProgramView::inspect(&block.program).unwrap().extents();
    block.program[28..36].copy_from_slice(&((extents.stack + 1) as u64).to_le_bytes());
    let result_offset = 52 + extents.constants * 8;
    block.program[result_offset..result_offset + 8]
        .copy_from_slice(&(extents.stack as u64).to_le_bytes());
    let called = Cell::new(0);
    let result = restore(bad, scope(), Arc::new(Construction), |_| {
        called.set(called.get() + 1);
        Ok::<_, Infallible>(Arc::new(()))
    });
    assert!(
        matches!(result,Err(TransferError::Supplier(F64TransferError::Undefined(i))) if i==extents.stack)
    );
    assert_eq!(called.get(), 1);
}

#[test]
fn actual_provider_directional_and_unavailable_order_profiles_refuse_before_capture_admission() {
    use pse_kernels::{DerivativeSource, Port, ProviderShapes};
    crate::initialize().unwrap();
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let quantity = pse_quantity::standard::ids::quantity("neutral");
    let unit = registry.quantity_type(quantity).unwrap().canonical_unit;
    let port = |n| Port {
        id: id(n),
        quantity,
        unit,
    };
    let spec = ProviderSpec {
        shapes: ProviderShapes::default(),
        derivative_source: DerivativeSource::Analytic,
        id: id(41),
        revision: pse_ids::ContentHash::from_bytes([42; 32]),
        data: pse_ids::ContentHash::from_bytes([43; 32]),
        inputs: vec![port(44)],
        outputs: vec![port(45)],
        derivatives: DerivativeOrder::First,
        smoothness: DerivativeOrder::First,
    };
    spec.validate(&registry).unwrap();
    let provider = PreparedBody::new(
        1,
        2,
        vec![1],
        vec![Stage::Provider {
            spec,
            partial: vec![],
            inputs: vec![0],
            outputs: vec![1],
            source: id(46),
        }],
        DerivativeOrder::First,
    )
    .unwrap()
    .compile(
        &[0],
        &[0],
        DerivativeOrder::First,
        Optimization::default(),
        EvaluationLimits::default(),
        &scope(),
    )
    .unwrap();
    let second = PreparedBody::new(1, 1, vec![0], vec![], DerivativeOrder::Second)
        .unwrap()
        .compile(
            &[0],
            &[0],
            DerivativeOrder::Second,
            Optimization::default(),
            EvaluationLimits::default(),
            &scope(),
        )
        .unwrap();
    let directional = fixture(false)
        .prepared_support()
        .compile_directional(
            Optimization::default(),
            EvaluationLimits::default(),
            &scope(),
        )
        .unwrap();
    let value = PreparedBody::new(1, 1, vec![0], vec![], DerivativeOrder::First)
        .unwrap()
        .compile(
            &[0],
            &[0],
            DerivativeOrder::Value,
            Optimization::default(),
            EvaluationLimits::default(),
            &scope(),
        )
        .unwrap();
    let mut received_second = restored(snapshot(&second));
    assert_eq!(received_second.order(), DerivativeOrder::Second);
    let result = received_second
        .evaluate(&[3.], DerivativeOrder::Second)
        .unwrap();
    assert_eq!(result.values, [3.]);
    assert_eq!(result.jacobian, [1.]);
    assert_eq!(result.hessians, [0.]);
    for body in [&provider, &directional, &value] {
        ALLOCATIONS.with(|n| n.set(0));
        let called = Cell::new(false);
        assert!(matches!(
            capture(body, &scope(), |_| {
                called.set(true);
                Ok::<_, Infallible>(Arc::new(()))
            }),
            Err(TransferError::Unsupported(_))
        ));
        assert!(!called.get());
        assert_eq!(ALLOCATIONS.with(Cell::get), 0);
    }
}

#[cfg(feature = "compiled-evaluator")]
#[test]
fn actual_native_jit_profile_refuses_without_interpreted_recompilation() {
    let body = fixture(false);
    let native = body
        .prepared_support()
        .compile(
            Optimization {
                evaluator: library::Evaluator::CompiledScalar {
                    module_bytes: 16 << 20,
                },
                ..Optimization::default()
            },
            EvaluationLimits::default(),
            &scope(),
        )
        .unwrap();
    assert!(native.census().any(|c| c.native_arithmetic_blocks > 0));
    let called = Cell::new(false);
    ALLOCATIONS.with(|n| n.set(0));
    assert!(matches!(
        capture(&native, &scope(), |_| {
            called.set(true);
            Ok::<_, Infallible>(Arc::new(()))
        }),
        Err(TransferError::Unsupported("native/JIT evaluator"))
    ));
    assert!(!called.get());
    assert_eq!(ALLOCATIONS.with(Cell::get), 0);
}

#[test]
fn original_scope_deadline_cancel_during_admission_and_all_anchors_survive() {
    use std::time::{Duration, Instant};
    let passive = snapshot(&fixture(true));
    let deadline = pse_kernels::ExecutionScope::new(
        pse_diagnostics::Cancellation::new(false),
        Some(Instant::now() - Duration::from_secs(1)),
    );
    assert!(matches!(
        restore(
            passive.clone(),
            deadline,
            Arc::new(Construction),
            |_| -> std::result::Result<Arc<dyn AllocationOwner>, Infallible> {
                panic!("expired scope")
            }
        ),
        Err(TransferError::Math(MathError::Scope(
            pse_kernels::ProviderError::Deadline
        )))
    ));
    let live = scope();
    let control = live.cancellation_control().clone();
    assert!(matches!(
        restore(passive.clone(), live, Arc::new(Construction), |_| {
            control.cancel();
            Ok::<_, Infallible>(Arc::new(()))
        }),
        Err(TransferError::Math(MathError::Cancelled))
    ));
    #[derive(Debug)]
    struct OriginalConstruction(Arc<AtomicUsize>);
    impl ConstructionAdmission for OriginalConstruction {
        fn try_grow(&self, _: usize) -> std::result::Result<(), MathError> {
            Ok(())
        }
    }
    impl Drop for OriginalConstruction {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let dropped = Arc::new(AtomicUsize::new(0));
    let construction = Arc::new(OriginalConstruction(dropped.clone()));
    let weak = Arc::downgrade(&construction);
    let checked = restore(passive, scope(), construction.clone(), |_| {
        Ok::<_, Infallible>(Arc::new(()))
    })
    .unwrap();
    drop(construction);
    assert!(weak.upgrade().is_some());
    assert_eq!(dropped.load(Ordering::SeqCst), 0);
    let frame = checked.frame;
    assert!(weak.upgrade().is_some());
    drop(frame);
    assert!(weak.upgrade().is_none());
    assert_eq!(dropped.load(Ordering::SeqCst), 1);
}

#[test]
fn original_guard_domain_branch_attribution_and_refusal_match_restored() {
    for nested in [false, true] {
        let body = fixture(nested);
        let mut original = body.numeric_frame_scoped(scope());
        let mut checked = restored(snapshot(&body));
        let before = original
            .evaluate(
                &[2., 3., -1.],
                DerivativeOrder::First,
                &mut ProviderBindings::new(),
                scope().cancellation(),
            )
            .unwrap_err();
        let after = checked
            .frame
            .evaluate(
                &[2., 3., -1.],
                DerivativeOrder::First,
                &mut ProviderBindings::new(),
                scope().cancellation(),
            )
            .unwrap_err();
        match (before, after) {
            (MathError::Validity(a), MathError::Validity(b)) => {
                assert_eq!(a, b);
                assert_eq!(a.source, id(9));
            }
            other => panic!("expected original validity: {other:?}"),
        }
        if nested {
            let before = original
                .evaluate(
                    &[0., 3., 1.],
                    DerivativeOrder::First,
                    &mut ProviderBindings::new(),
                    scope().cancellation(),
                )
                .unwrap_err();
            let after = checked
                .frame
                .evaluate(
                    &[0., 3., 1.],
                    DerivativeOrder::First,
                    &mut ProviderBindings::new(),
                    scope().cancellation(),
                )
                .unwrap_err();
            match (before, after) {
                (
                    MathError::Domain {
                        source_id: a,
                        requirement: x,
                    },
                    MathError::Domain {
                        source_id: b,
                        requirement: y,
                    },
                ) => {
                    assert_eq!(a, b);
                    assert_eq!(x, y);
                    assert_eq!(
                        x,
                        "branch-local derivative trial is on an unproved control boundary"
                    );
                }
                other => panic!("expected original branch refusal: {other:?}"),
            }
        }
    }
    let body = fixture_with_predicate(true, 1);
    let mut original = body.numeric_frame_scoped(scope());
    let mut checked = restored(snapshot(&body));
    let before = original
        .evaluate(
            &[2., 3., 1.],
            DerivativeOrder::First,
            &mut ProviderBindings::new(),
            scope().cancellation(),
        )
        .unwrap_err();
    let after = checked
        .frame
        .evaluate(
            &[2., 3., 1.],
            DerivativeOrder::First,
            &mut ProviderBindings::new(),
            scope().cancellation(),
        )
        .unwrap_err();
    match (before, after) {
        (MathError::Applicability(a), MathError::Applicability(b)) => {
            assert_eq!(a, b);
            assert_eq!(a.observations.len(), 4);
            assert!(!a.refused.is_empty());
            assert_eq!(a.observations[0].permissions[0].scope, id(25));
        }
        other => panic!("expected original applicability: {other:?}"),
    }
}

#[test]
fn passive_projection_and_json_do_not_construct_execution_or_duplicate_existing_plan_meaning() {
    use pse_contract::boundary_projection::{
        ConstructionRole, LocalRule, ProjectionGraph, Shape, ValidateProjection,
    };
    let passive = snapshot(&fixture(true));
    let bytes = serde_json::to_vec(&passive).unwrap();
    let decoded: ExecutionSnapshot = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(decoded, passive);
    let mut decoded_facts = decoded.inspect().unwrap();
    let mut original_facts = passive.inspect().unwrap();
    assert!(decoded_facts.input_bytes >= decoded_facts.capture_bytes);
    let input_difference = decoded_facts.input_bytes - original_facts.input_bytes;
    let retained_difference = decoded_facts.template_bytes - original_facts.template_bytes;
    assert_eq!(
        decoded_facts.restore_peak_bytes - original_facts.restore_peak_bytes,
        input_difference + retained_difference
    );
    decoded_facts.input_bytes = 0;
    decoded_facts.template_bytes = 0;
    decoded_facts.restore_peak_bytes = 0;
    original_facts.input_bytes = 0;
    original_facts.template_bytes = 0;
    original_facts.restore_peak_bytes = 0;
    assert_eq!(
        decoded_facts, original_facts,
        "identical populations with independently accounted owned input capacities"
    );
    let graph = ProjectionGraph::of::<ExecutionSnapshot>().unwrap();
    for name in [
        "MathExecutionSnapshot",
        "MathStageSnapshot",
        "MathJetLayoutSnapshot",
        "ApplicabilityNode",
        "ModelValidityLineage",
    ] {
        assert_eq!(
            graph.definitions()[name].construction,
            ConstructionRole::Data
        );
    }
    let Shape::Record(fields) = &graph.definitions()["MathExecutionSnapshot"].shape else {
        panic!("snapshot record");
    };
    assert_eq!(
        fields.iter().find(|f| f.name == "version").unwrap().rules,
        [LocalRule::UnsignedRange { min: 2, max: 2 }]
    );
    for name in ["limits", "value_layout", "first_layout"] {
        assert_eq!(
            fields.iter().find(|f| f.name == name).unwrap().rules,
            [LocalRule::Required]
        );
    }
    assert!(decoded.validate_projection().is_ok());
    let mut wrong_version = decoded.clone();
    wrong_version.version = 0;
    assert!(wrong_version.validate_projection().is_err());
    let mut unknown = serde_json::to_value(passive).unwrap();
    unknown["worker_bytes"] = serde_json::json!(0);
    assert!(serde_json::from_value::<ExecutionSnapshot>(unknown).is_err());
}

#[test]
fn native_depth_profile_is_explicit_and_capture_owner_survives_publication_handoff() {
    let body = fixture(false);
    let marker = Arc::new(AtomicUsize::new(0));
    let weak = Arc::downgrade(&marker);
    let captured = capture(&body, &scope(), |_| Ok::<_, Infallible>(marker.clone())).unwrap();
    drop(marker);
    assert!(weak.upgrade().is_some());
    let (passive, owner) = captured.into_parts();
    assert!(weak.upgrade().is_some());
    let _encoded = serde_json::to_vec(&passive).unwrap();
    drop(passive);
    assert!(weak.upgrade().is_some());
    drop(owner);
    assert!(weak.upgrade().is_none());
    crate::initialize().unwrap();
    let identity = PreparedBody::new(1, 1, vec![0], vec![], DerivativeOrder::First)
        .unwrap()
        .compile(
            &[0],
            &[0],
            DerivativeOrder::First,
            Optimization::default(),
            EvaluationLimits::default(),
            &scope(),
        )
        .unwrap();
    let mut passive = snapshot(&identity);
    for _ in 0..MAX_STAGE_DEPTH {
        passive.value = vec![StageSnapshot::Branch {
            require_separation: false,
            comparison: Comparison::Eq,
            left: 0,
            right: 0,
            then: std::mem::take(&mut passive.value),
            otherwise: vec![],
        }];
    }
    assert!(passive.inspect().is_ok());
    passive.value = vec![StageSnapshot::Branch {
        require_separation: false,
        comparison: Comparison::Eq,
        left: 0,
        right: 0,
        then: std::mem::take(&mut passive.value),
        otherwise: vec![],
    }];
    let called = Cell::new(false);
    assert!(
        restore(passive, scope(), Arc::new(Construction), |_| {
            called.set(true);
            Ok::<_, Infallible>(Arc::new(()))
        })
        .is_err()
    );
    assert!(!called.get());
}

#[test]
fn original_decoded_input_allocation_owner_is_retained_after_metadata_moves_into_frame() {
    let passive = snapshot(&fixture(true));
    let marker = Arc::new(AtomicUsize::new(0));
    let weak = Arc::downgrade(&marker);
    let input = CapturedExecution::from_admitted_snapshot(passive, marker.clone());
    drop(marker);
    let checked = super::restore(input, scope(), Arc::new(Construction), |_| {
        Ok::<_, Infallible>(Arc::new(()))
    })
    .unwrap();
    assert!(weak.upgrade().is_some());
    let frame = checked.frame;
    assert!(weak.upgrade().is_some());
    drop(frame);
    assert!(weak.upgrade().is_none());
}

#[test]
fn existing_taylor_producer_preserves_absent_zero_arguments_and_cold_reset() {
    crate::initialize().unwrap();
    let x = library::formal(0).unwrap();
    let y = library::formal(1).unwrap();
    let body = PreparedBody::new(
        3,
        4,
        vec![3],
        vec![block(x * y, 3, 81)],
        DerivativeOrder::First,
    )
    .unwrap();
    let support = body
        .prepare_support(
            &[0],
            &[0, 1, 2],
            DerivativeOrder::First,
            scope().cancellation(),
        )
        .unwrap();
    FORCE_TAYLOR_FIRST.set(true);
    let result = support.body().compile_scope(
        &[0],
        &support.dense_coordinates().unwrap(),
        DerivativeOrder::First,
        Optimization::default(),
        EvaluationLimits::default(),
        &scope(),
        false,
        ProductKind::SelectedPartials,
        &support,
        None,
        RegionStrategy::ReverseSweep,
    );
    FORCE_TAYLOR_FIRST.set(false);
    let compiled = result.unwrap();
    let mut checked = restored(snapshot(&compiled));
    let mut absent = 0;
    for stage in checked.frame.template.programs[DerivativeOrder::First]
        .as_ref()
        .unwrap()
    {
        if let CompiledStage::Block {
            arguments, gather, ..
        } = stage
        {
            for (i, value) in arguments.iter().enumerate() {
                if !gather.iter().any(|&(argument, _)| argument == i) {
                    absent += 1;
                    assert_eq!(value.to_bits(), 0);
                } else {
                    assert!(value.is_nan());
                }
            }
        }
    }
    assert!(
        absent > 0,
        "real supplier vectorization includes absent structural arguments"
    );
    let before = values(&mut checked.frame, &[2., 3., 4.], DerivativeOrder::First);
    assert_eq!(before.values, [6.]);
    assert_eq!(before.jacobian, [3., 2., 0.]);
    checked.frame.detach_controls();
    checked.frame.reset_scratch().unwrap();
    checked.frame.set_scope(scope());
    let after = values(&mut checked.frame, &[5., 7., 9.], DerivativeOrder::First);
    assert_eq!(after.values, [35.]);
    assert_eq!(after.jacobian, [7., 5., 0.]);
}
#[test]
fn admission_original_payload_and_owner_lifetime_and_cancel_are_preserved() {
    #[derive(Debug, thiserror::Error)]
    #[error("original denial")]
    struct Denial(Arc<AtomicUsize>);
    let marker = Arc::new(AtomicUsize::new(0));
    let passive = snapshot(&fixture(false));
    let calls = Cell::new(0);
    let error = restore(passive.clone(), scope(), Arc::new(Construction), |_| {
        calls.set(calls.get() + 1);
        Err(Denial(marker.clone()))
    })
    .unwrap_err();
    assert_eq!(calls.get(), 1);
    let original = std::error::Error::source(&error)
        .unwrap()
        .downcast_ref::<Denial>()
        .unwrap();
    assert!(Arc::ptr_eq(&marker, &original.0));
    let TransferError::Admission(Denial(payload)) = error else {
        panic!("original admission class");
    };
    assert!(Arc::ptr_eq(&marker, &payload));
    #[derive(Debug)]
    struct Owner(Arc<AtomicUsize>);
    impl Drop for Owner {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let dropped = Arc::new(AtomicUsize::new(0));
    let owner = Arc::new(Owner(dropped.clone()));
    let weak = Arc::downgrade(&owner);
    let checked = restore(passive.clone(), scope(), Arc::new(Construction), |_| {
        Ok::<_, Infallible>(owner.clone())
    })
    .unwrap();
    drop(owner);
    assert!(weak.upgrade().is_some());
    drop(checked);
    assert!(weak.upgrade().is_none());
    assert_eq!(dropped.load(Ordering::SeqCst), 1);
    let cancelled =
        pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::new(true), None);
    assert!(matches!(
        restore(
            passive,
            cancelled,
            Arc::new(Construction),
            |_| -> std::result::Result<Arc<dyn AllocationOwner>, Infallible> {
                panic!("cancelled admission")
            }
        ),
        Err(TransferError::Math(MathError::Cancelled))
    ));
}
