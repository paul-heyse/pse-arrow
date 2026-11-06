// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::{
    SourceBindings, SourceBody,
    guarded::{Condition, PreparedBody, Stage},
    jets::EvaluationLimits,
    library::Optimization,
};
use pse_model::{
    diagnostic::{DiagnosticProjection, ValidityLineage},
    generated::enums::ModelingValidityLayer,
};
use std::{cell::Cell, sync::atomic::AtomicUsize};

fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
fn scope() -> pse_kernels::ExecutionScope {
    pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::new(false), None)
}
#[derive(Debug)]
struct Construction;
impl ConstructionAdmission for Construction {
    fn try_grow(&self, _: usize) -> std::result::Result<(), MathError> {
        panic!("restoration/evaluation backing was prepaid")
    }
}
fn fixture(closure: bool) -> Problem {
    crate::initialize().unwrap();
    let x = crate::library::formal(0).unwrap();
    let y = crate::library::formal(1).unwrap();
    let p = crate::library::formal(2).unwrap();
    let q = crate::library::formal(3).unwrap();
    let lineage = Arc::new(ValidityLineage {
        layer: if closure {
            ModelingValidityLayer::Closure
        } else {
            ModelingValidityLayer::Form
        },
        source: id(51),
        form: Some(id(52)),
        sets: vec![id(53)],
        variables: vec![0],
        members: if closure { vec![id(54)] } else { vec![] },
    });
    let mut source = PreparedBody::new(
        4,
        6,
        vec![4, 5],
        vec![
            Stage::Require {
                argument: 0,
                condition: Condition::Positive,
                order: DerivativeOrder::Value,
                source: id(51),
                lineage: Some(lineage),
            },
            Stage::Block {
                expressions: vec![&x * &x + 2 * &y - p, y * q],
                outputs: vec![4, 5],
                source: id(55),
            },
        ],
        DerivativeOrder::Second,
    )
    .unwrap();
    if closure {
        source = source
            .with_checked_members([id(54)].into())
            .unwrap()
            .with_checked_sources([id(51)].into())
            .unwrap()
            .with_occurrence_tokens([id(56)].into())
            .unwrap();
    }
    let compiled = Arc::new(
        source
            .compile(
                &[0, 1],
                &[0, 1, 2, 3],
                DerivativeOrder::Second,
                Optimization::default(),
                EvaluationLimits::default(),
                &scope(),
            )
            .unwrap(),
    );
    let location = SourceLocation {
        source: id(60),
        revision: Some(ContentHash::from_bytes([61; 32])),
        document: Some(id(62)),
        occurrence_role: Some("relation".into()),
        occurrence_position: Some(2),
        path: "original/equation".into(),
        name: Some("unknown closure".into()),
        start: Some(3),
        end: Some(9),
    };
    let bindings = SourceBindings::new(
        id(50),
        &source,
        if closure {
            [(id(54), id(58))].into()
        } else {
            BTreeMap::new()
        },
        if closure {
            [(id(51), id(59))].into()
        } else {
            BTreeMap::new()
        },
        if closure {
            [(id(56), location)].into()
        } else {
            BTreeMap::new()
        },
    )
    .unwrap();
    let body = SourceBody::new(compiled, bindings).unwrap();
    let mut problem = Problem::new(
        id(70),
        ContentHash::from_bytes([71; 32]),
        vec![
            Unknown {
                id: id(72),
                lower: -10.,
                upper: 10.,
            },
            Unknown {
                id: id(73),
                lower: -10.,
                upper: 10.,
            },
        ],
        vec![id(74), id(75)],
        2,
        residual_bodies_for_test(body),
        100,
    )
    .unwrap();
    problem.set_residual_offsets(&[0.5, -1.]).unwrap();
    problem
}
fn snapshot(root: &RootExecution) -> RootExecutionSnapshot {
    capture(root, &scope(), |_| Ok::<_, Infallible>(Arc::new(())))
        .unwrap()
        .into_parts()
        .0
}
fn delegated(snapshot: RootExecutionSnapshot) -> Arc<RootExecution> {
    let (numeric, metadata) = snapshot.into_restore_parts(Arc::new(()));
    let checked = crate::transfer::restore(numeric, scope(), Arc::new(Construction), |_| {
        Ok::<_, Infallible>(Arc::new(()))
    })
    .unwrap();
    restore(checked, metadata, scope(), Arc::new(Construction), |_| {
        Ok::<_, Infallible>(Arc::new(()))
    })
    .unwrap()
}
#[test]
fn delegated_root_uses_original_unknown_parameter_order_offsets_and_attribution_without_source() {
    for closure in [false, true] {
        let original = fixture(closure);
        let data = snapshot(&original);
        original
            .evaluate_unknown(
                &[7., 9.],
                &[4., 2.],
                DerivativeOrder::First,
                &Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
        assert_eq!(
            serde_json::to_value(snapshot(&original)).unwrap(),
            serde_json::to_value(&data).unwrap(),
            "capture excludes warmed arguments/results/frames"
        );
        assert_eq!(
            data.definition.requirements.residual_compilation,
            DerivativeOrder::Second
        );
        assert_eq!(data.definition.native_order, DerivativeOrder::First);
        let actual = delegated(data.clone());
        assert_eq!(actual.identity, original.identity);
        assert_eq!(actual.id, original.id);
        assert_eq!(actual.pattern.col_ptr(), original.pattern.col_ptr());
        assert_eq!(actual.pattern.row_idx(), original.pattern.row_idx());
        assert_eq!(actual.requirements, original.requirements);
        let cancel = Arc::new(AtomicBool::new(false));
        for (parameters, unknowns, expected, jacobian) in [
            ([5., 4.], [2., 3.], [4.5, 13.], [4., 2., 0., 4.]),
            ([1., 2.], [3., 1.], [9.5, 3.], [6., 2., 0., 2.]),
        ] {
            for order in [DerivativeOrder::Value, DerivativeOrder::First] {
                let baseline = original
                    .evaluate_unknown(&parameters, &unknowns, order, &cancel)
                    .unwrap();
                let restored = actual
                    .evaluate_unknown(&parameters, &unknowns, order, &cancel)
                    .unwrap();
                assert_eq!(restored, baseline);
                assert_eq!(restored.values, expected);
                if order == DerivativeOrder::First {
                    assert_eq!(restored.jacobian, jacobian);
                }
            }
        }
        let cause = original
            .evaluate_unknown(&[5., 4.], &[-1., 3.], DerivativeOrder::First, &cancel)
            .unwrap_err();
        let restored = actual
            .evaluate_unknown(&[5., 4.], &[-1., 3.], DerivativeOrder::First, &cancel)
            .unwrap_err();
        assert_eq!(
            serde_json::to_value(
                restored.boundary_diagnostic(pse_diagnostics::DiagnosticStage::Evaluation)
            )
            .unwrap(),
            serde_json::to_value(
                cause.boundary_diagnostic(pse_diagnostics::DiagnosticStage::Evaluation)
            )
            .unwrap()
        );
        assert_eq!(snapshot(&actual).attribution, data.attribution);
        drop(original);
        assert_eq!(
            actual
                .evaluate_unknown(&[5., 4.], &[2., 3.], DerivativeOrder::First, &cancel)
                .unwrap()
                .jacobian,
            [4., 2., 0., 4.]
        );
    }
}
#[test]
fn passive_planner_checks_csc_signature_and_actual_owned_capacity_before_materialization() {
    let base = snapshot(&fixture(true));
    let facts = base.restore_extents().unwrap();
    assert_eq!(
        facts.restore_peak_bytes,
        facts.numerical.restore_peak_bytes
            + facts.metadata_input_bytes
            + facts.construction_bytes
            + facts.validation_bytes
    );
    let mut extra = base.clone();
    extra.definition.rows.reserve_exact(17);
    assert!(extra.restore_extents().unwrap().metadata_input_bytes > facts.metadata_input_bytes);
    let decoded: RootExecutionSnapshot =
        serde_json::from_slice(&serde_json::to_vec(&base).unwrap()).unwrap();
    assert_eq!(
        serde_json::to_value(&base).unwrap(),
        serde_json::to_value(&decoded).unwrap()
    );
    delegated(decoded);
    let mutations: [fn(&mut RootExecutionSnapshot); 17] = [
        |s| s.definition.col_ptr.clear(),
        |s| s.definition.col_ptr[0] = 1,
        |s| s.definition.col_ptr[1] = usize::MAX,
        |s| s.definition.col_ptr[2] = 0,
        |s| s.definition.row_idx[0] = 2,
        |s| s.definition.row_idx[2] = s.definition.row_idx[1],
        |s| {
            s.definition.row_idx.remove(0);
            for p in &mut s.definition.col_ptr[1..] {
                *p -= 1;
            }
        },
        |s| s.definition.rows.pop().map(drop).unwrap(),
        |s| s.definition.residual_offsets.pop().map(drop).unwrap(),
        |s| s.definition.residual_offsets[0] = f64::NAN,
        |s| s.definition.unknowns[0].lower = 11.,
        |s| s.definition.unknowns[0].id = SemanticId::NIL,
        |s| s.definition.inputs = 0,
        |s| s.numerical.coordinate_offsets.swap(0, 1),
        |s| s.numerical.outputs.pop().map(drop).unwrap(),
        |s| s.definition.requirements.residual_compilation = DerivativeOrder::Value,
        |s| s.numerical.input_formals[0] = 4,
    ];
    for (index, mutate) in mutations.into_iter().enumerate() {
        let mut bad = base.clone();
        mutate(&mut bad);
        assert!(bad.restore_extents().is_err(), "malformed case {index}");
    }
}
#[test]
fn conservative_extra_csc_positions_are_legal_but_true_axes_cannot_be_omitted() {
    let mut data = snapshot(&fixture(false));
    // Producer support is (row0,col0),(row0,col1),(row1,col1); retain legal extra row1,col0.
    assert_eq!(data.definition.col_ptr, [0, 1, 3]);
    assert_eq!(data.definition.row_idx, [0, 0, 1]);
    data.definition.col_ptr = [0, 2, 4].into();
    data.definition.row_idx = [0, 1, 0, 1].into();
    let root = delegated(data);
    assert_eq!(root.pattern.row_idx(), [0, 1, 0, 1]);
    assert_eq!(
        root.evaluate_unknown(
            &[5., 4.],
            &[2., 3.],
            DerivativeOrder::First,
            &Arc::new(AtomicBool::new(false))
        )
        .unwrap()
        .jacobian,
        [4., 2., 0., 4.]
    );
}
#[test]
fn full_original_attribution_is_committed_and_only_closure_tokens_require_maps() {
    let base = snapshot(&fixture(true));
    for which in 0..3 {
        let mut missing = base.clone();
        match which {
            0 => missing.attribution.checked_members.clear(),
            1 => missing.attribution.checked_sources.clear(),
            _ => missing.attribution.occurrences.clear(),
        };
        assert!(matches!(
            missing.restore_extents(),
            Err(RootTransferError::Attribution(
                AttributionTransferError::Commitment
            ))
        ));
    }
    for which in 0..2 {
        let mut missing = base.clone();
        if which == 0 {
            missing.attribution.checked_members.clear();
        } else {
            missing.attribution.checked_sources.clear();
        }
        missing.definition.attribution_commitment = missing.attribution.commitment();
        let (numeric, metadata) = missing.into_restore_parts(Arc::new(()));
        let checked = crate::transfer::restore(numeric, scope(), Arc::new(Construction), |_| {
            Ok::<_, Infallible>(Arc::new(()))
        })
        .unwrap();
        let calls = Cell::new(0);
        let result = restore(checked, metadata, scope(), Arc::new(Construction), |_| {
            calls.set(calls.get() + 1);
            Ok::<_, Infallible>(Arc::new(()))
        });
        assert!(matches!(
            result,
            Err(RootTransferError::Attribution(
                AttributionTransferError::MissingMember(_)
                    | AttributionTransferError::MissingSource(_)
            ))
        ));
        assert_eq!(calls.get(), 0);
    }
    let form = snapshot(&fixture(false));
    assert!(form.attribution.checked_members.is_empty());
    assert!(form.attribution.checked_sources.is_empty());
    delegated(form);
    let mut extras = base;
    extras.attribution.checked_members.insert(id(100), id(101));
    extras.definition.attribution_commitment = extras.attribution.commitment();
    let root = delegated(extras.clone());
    assert_eq!(snapshot(&root).attribution, extras.attribution);
}
#[test]
fn root_metadata_admission_preserves_denial_source_and_calls_once() {
    #[derive(Debug, thiserror::Error)]
    #[error("original metadata denial")]
    struct Denial(Arc<AtomicUsize>);
    let payload = Arc::new(AtomicUsize::new(0));
    let calls = Cell::new(0);
    let (numeric, metadata) = snapshot(&fixture(false)).into_restore_parts(Arc::new(()));
    let checked = crate::transfer::restore(numeric, scope(), Arc::new(Construction), |_| {
        Ok::<_, Infallible>(Arc::new(()))
    })
    .unwrap();
    let error = restore(checked, metadata, scope(), Arc::new(Construction), |_| {
        calls.set(calls.get() + 1);
        Err(Denial(payload.clone()))
    })
    .unwrap_err();
    assert_eq!(calls.get(), 1);
    let source = std::error::Error::source(&error)
        .unwrap()
        .downcast_ref::<Denial>()
        .unwrap();
    assert!(Arc::ptr_eq(&source.0, &payload));
    assert!(matches!(error, RootTransferError::Admission(Denial(_))));
    let original = fixture(false);
    let error = capture(&original, &scope(), |_| Err(Denial(payload.clone()))).unwrap_err();
    let source = std::error::Error::source(&error)
        .unwrap()
        .downcast_ref::<Denial>()
        .unwrap();
    assert!(Arc::ptr_eq(&source.0, &payload));
    assert!(matches!(error, RootTransferError::Admission(Denial(_))));
}
#[test]
fn original_allocation_and_construction_owners_survive_root_and_escaped_frame_until_drain() {
    let source = snapshot(&fixture(false));
    let input = Arc::new(AtomicUsize::new(1));
    let numeric = Arc::new(AtomicUsize::new(2));
    let metadata = Arc::new(AtomicUsize::new(3));
    let construction = Arc::new(Construction);
    let weak = [
        Arc::downgrade(&input),
        Arc::downgrade(&numeric),
        Arc::downgrade(&metadata),
    ];
    let weak_construction = Arc::downgrade(&construction);
    let (captured, retained) = source.into_restore_parts(input.clone());
    let checked = crate::transfer::restore(captured, scope(), construction.clone(), |_| {
        Ok::<_, Infallible>(numeric.clone())
    })
    .unwrap();
    let root = restore(checked, retained, scope(), construction.clone(), |_| {
        Ok::<_, Infallible>(metadata.clone())
    })
    .unwrap();
    drop((input, numeric, metadata, construction));
    assert!(weak.iter().all(|owner| owner.upgrade().is_some()));
    assert!(weak_construction.upgrade().is_some());
    let frame = {
        let root = Arc::try_unwrap(root).unwrap();
        root.unknown_worker.into_inner().unwrap().worker.numeric
    };
    assert!(weak.iter().all(|owner| owner.upgrade().is_some()));
    assert!(weak_construction.upgrade().is_some());
    drop(frame);
    assert!(weak.iter().all(|owner| owner.upgrade().is_none()));
    assert!(weak_construction.upgrade().is_none());
}
#[test]
fn native_root_profile_refusals_remain_distinct_from_malformed_and_scope_errors() {
    let original = fixture(false);
    let base = snapshot(&original);
    for which in 0..3 {
        let mut unsupported = base.clone();
        match which {
            0 => unsupported.definition.version = 2,
            1 => unsupported.definition.native_order = DerivativeOrder::Value,
            _ => unsupported.definition.unknowns[0].lower = f64::NEG_INFINITY,
        };
        assert!(matches!(
            unsupported.restore_extents(),
            Err(RootTransferError::Unsupported(_))
        ));
    }
    let cancelled =
        pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::new(true), None);
    assert!(matches!(
        capture(
            &original,
            &cancelled,
            |_| -> std::result::Result<Arc<dyn AllocationOwner>, Infallible> {
                panic!("cancel before capture callback")
            }
        ),
        Err(RootTransferError::Math(MathError::Cancelled))
    ));
    let (numeric, metadata) = base.into_restore_parts(Arc::new(()));
    let checked = crate::transfer::restore(numeric, scope(), Arc::new(Construction), |_| {
        Ok::<_, Infallible>(Arc::new(()))
    })
    .unwrap();
    assert!(matches!(
        restore(
            checked,
            metadata,
            cancelled,
            Arc::new(Construction),
            |_| -> std::result::Result<Arc<dyn AllocationOwner>, Infallible> {
                panic!("cancel before metadata callback")
            }
        ),
        Err(RootTransferError::Math(MathError::Cancelled))
    ));
}
#[test]
fn metadata_identity_validation_and_scope_failures_follow_actual_admission() {
    let original = fixture(false);
    let base = snapshot(&original);
    for duplicate_unknown in [false, true] {
        let mut bad = base.clone();
        if duplicate_unknown {
            bad.definition.unknowns[1].id = bad.definition.unknowns[0].id;
        } else {
            bad.definition.rows[1] = bad.definition.rows[0];
        }
        // Structural planning allocates nothing; uniqueness gets exactly the admitted scratch.
        assert!(bad.restore_extents().is_ok());
        let (numeric, metadata) = bad.into_restore_parts(Arc::new(()));
        let checked = crate::transfer::restore(numeric, scope(), Arc::new(Construction), |_| {
            Ok::<_, Infallible>(Arc::new(()))
        })
        .unwrap();
        let calls = Cell::new(0);
        assert!(matches!(
            restore(checked, metadata, scope(), Arc::new(Construction), |_| {
                calls.set(calls.get() + 1);
                Ok::<_, Infallible>(Arc::new(()))
            }),
            Err(RootTransferError::Invalid(_))
        ));
        assert_eq!(calls.get(), 1);
    }
    let expired = pse_kernels::ExecutionScope::new(
        pse_diagnostics::Cancellation::new(false),
        Some(std::time::Instant::now() - Duration::from_secs(1)),
    );
    assert!(matches!(
        capture(
            &original,
            &expired,
            |_| -> std::result::Result<Arc<dyn AllocationOwner>, Infallible> {
                panic!("expired capture")
            }
        ),
        Err(RootTransferError::Math(MathError::Scope(
            pse_kernels::ProviderError::Deadline
        )))
    ));
    let live = scope();
    let control = live.cancellation_control().clone();
    assert!(matches!(
        capture(&original, &live, |_| {
            control.cancel();
            Ok::<_, Infallible>(Arc::new(()))
        }),
        Err(RootTransferError::Numerical(
            crate::transfer::TransferError::Math(MathError::Cancelled)
        ))
    ));
    let live = scope();
    let control = live.cancellation_control().clone();
    let (numeric, metadata) = base.into_restore_parts(Arc::new(()));
    let checked = crate::transfer::restore(numeric, live.clone(), Arc::new(Construction), |_| {
        Ok::<_, Infallible>(Arc::new(()))
    })
    .unwrap();
    assert!(matches!(
        restore(checked, metadata, live, Arc::new(Construction), |_| {
            control.cancel();
            Ok::<_, Infallible>(Arc::new(()))
        }),
        Err(RootTransferError::Math(MathError::Cancelled))
    ));
}
#[test]
fn capture_owner_and_metadata_budget_follow_complete_original_publication() {
    let original = fixture(true);
    let owner = Arc::new(AtomicUsize::new(1));
    let weak = Arc::downgrade(&owner);
    let calls = Cell::new(0);
    let captured = capture(&original, &scope(), |facts| {
        calls.set(calls.get() + 1);
        assert_eq!(facts.attribution_entries, 3);
        assert!(facts.attribution_text_bytes > 0);
        assert_eq!(
            facts.capture_bytes,
            facts.numerical.capture_bytes + facts.metadata_bytes + 2 * size_of::<SemanticId>()
        );
        Ok::<_, Infallible>(owner.clone())
    })
    .unwrap();
    assert_eq!(calls.get(), 1);
    drop(owner);
    drop(original);
    assert!(weak.upgrade().is_some());
    let (snapshot, owner) = captured.into_parts();
    assert!(weak.upgrade().is_some());
    assert_eq!(
        snapshot.definition.attribution_commitment,
        snapshot.attribution.commitment()
    );
    drop(snapshot);
    assert!(weak.upgrade().is_some());
    drop(owner);
    assert!(weak.upgrade().is_none());
}

#[test]
fn complete_capture_finalization_preserves_late_control_cause_and_releases_owner() {
    #[derive(Debug)]
    struct Owner(Arc<AtomicUsize>);
    impl Drop for Owner {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let original = fixture(true);
    for deadline in [false, true] {
        let live = scope();
        let dropped = Arc::new(AtomicUsize::new(0));
        let owner = Arc::new(Owner(dropped.clone()));
        let weak = Arc::downgrade(&owner);
        let captured = capture(&original, &live, |_| Ok::<_, Infallible>(owner.clone())).unwrap();
        assert!(!captured.snapshot().numerical.first.is_empty());
        assert!(!captured.snapshot().attribution.occurrences.is_empty());
        drop(owner);
        assert_eq!(dropped.load(Ordering::SeqCst), 0);
        let final_scope = if deadline {
            pse_kernels::ExecutionScope::new(
                pse_diagnostics::Cancellation::new(false),
                Some(std::time::Instant::now() - Duration::from_secs(1)),
            )
        } else {
            live.cancellation_control().cancel();
            live
        };
        // Exercise the production owning finalizer after every capture buffer
        // exists. This verifies the late refusal class and owner release; Vec
        // deallocation ordering is guaranteed by the owning type's field order,
        // not observed through allocator instrumentation.
        let error = captured.check_scope(&final_scope).unwrap_err();
        if deadline {
            assert!(matches!(
                error,
                RootTransferError::Math(MathError::Scope(pse_kernels::ProviderError::Deadline))
            ));
        } else {
            assert!(matches!(
                error,
                RootTransferError::Math(MathError::Cancelled)
            ));
        }
        assert!(weak.upgrade().is_none());
        assert_eq!(dropped.load(Ordering::SeqCst), 1);
    }
}
