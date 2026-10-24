// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Real body admission and worker evaluation controls for current receiving attribution.
use super::*;
use crate::{
    guarded::{Condition, Stage},
    jets::EvaluationLimits,
    library::{self, Optimization},
};
use pse_model::{
    diagnostic::{DiagnosticProjection, ValidityLineage},
    generated::enums::ModelingValidityLayer,
};
use std::collections::BTreeSet;

fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
fn fixture_body(closure: bool) -> PreparedBody {
    crate::initialize().unwrap();
    let lineage = closure.then(|| {
        Arc::new(ValidityLineage {
            layer: ModelingValidityLayer::Closure,
            source: id(1),
            form: None,
            sets: vec![],
            variables: vec![],
            members: vec![id(1)],
        })
    });
    PreparedBody::new(
        1,
        2,
        vec![1],
        vec![
            Stage::Require {
                argument: 0,
                condition: Condition::Positive,
                order: DerivativeOrder::Value,
                source: id(3),
                lineage,
            },
            Stage::Block {
                expressions: vec![library::formal(0).unwrap()],
                outputs: vec![1],
                source: id(3),
            },
        ],
        DerivativeOrder::Value,
    )
    .unwrap()
    .with_checked_members(if closure {
        BTreeSet::from([id(1)])
    } else {
        BTreeSet::new()
    })
    .unwrap()
    .with_checked_sources(if closure {
        BTreeSet::from([id(1)])
    } else {
        BTreeSet::new()
    })
    .unwrap()
    .with_occurrence_tokens(BTreeSet::from([id(3)]))
    .unwrap()
}
fn location(n: u8) -> SourceLocation {
    SourceLocation {
        source: id(n),
        revision: None,
        document: Some(id(90)),
        occurrence_role: Some("expression".into()),
        occurrence_position: Some(0),
        path: "actual.fixture.x".into(),
        name: None,
        start: Some(4),
        end: Some(12),
    }
}
fn bindings(body: &PreparedBody, n: u8) -> SourceBindings {
    SourceBindings::new(
        id(n),
        body,
        body.checked_members()
            .iter()
            .map(|&token| (token, id(n + 1)))
            .collect(),
        body.checked_sources()
            .iter()
            .map(|&token| (token, id(n + 2)))
            .collect(),
        body.occurrence_tokens()
            .iter()
            .map(|&token| (token, location(n + 3)))
            .collect(),
    )
    .unwrap()
}
fn compile(body: &PreparedBody) -> Arc<CompiledBody> {
    let control = pse_diagnostics::Cancellation::new(false);
    Arc::new(
        body.prepare_support(&[0], &[], DerivativeOrder::Value, &control.flag())
            .unwrap()
            .compile(
                Optimization::default(),
                EvaluationLimits::default(),
                &ExecutionScope::new(control, None),
            )
            .unwrap(),
    )
}
#[test]
fn source_bindings_require_exact_member_source_and_occurrence_coverage() {
    let body = fixture_body(true);
    let member = BTreeMap::from([(id(1), id(11))]);
    let source = BTreeMap::from([(id(1), id(12))]);
    let occurrences = BTreeMap::from([(id(3), location(13))]);
    assert!(
        SourceBindings::new(
            id(10),
            &body,
            member.clone(),
            source.clone(),
            occurrences.clone()
        )
        .is_ok()
    );
    for slot in 0..3 {
        let mut m = member.clone();
        let mut s = source.clone();
        let mut o = occurrences.clone();
        match slot {
            0 => m.clear(),
            1 => s.clear(),
            _ => o.clear(),
        };
        assert!(SourceBindings::new(id(10), &body, m, s, o).is_err());
        let mut m = member.clone();
        let mut s = source.clone();
        let mut o = occurrences.clone();
        match slot {
            0 => {
                m.insert(id(9), id(19));
            }
            1 => {
                s.insert(id(9), id(19));
            }
            _ => {
                o.insert(id(9), location(19));
            }
        };
        assert!(SourceBindings::new(id(10), &body, m, s, o).is_err());
    }
    assert!(
        SourceBindings::new(
            SemanticId::NIL,
            &body,
            member.clone(),
            source.clone(),
            occurrences.clone()
        )
        .is_err()
    );
    for slot in 0..3 {
        let mut m = member.clone();
        let mut s = source.clone();
        let mut o = occurrences.clone();
        match slot {
            0 => {
                m.insert(id(1), SemanticId::NIL);
            }
            1 => {
                s.insert(id(1), SemanticId::NIL);
            }
            _ => {
                o.get_mut(&id(3)).unwrap().source = SemanticId::NIL;
            }
        };
        assert!(SourceBindings::new(id(10), &body, m, s, o).is_err());
    }
    for mode in 0..3 {
        let mut o = occurrences.clone();
        let l = o.get_mut(&id(3)).unwrap();
        match mode {
            0 => l.end = None,
            1 => l.start = Some(13),
            _ => l.document = None,
        };
        assert!(SourceBindings::new(id(10), &body, member.clone(), source.clone(), o).is_err());
    }
}
#[test]
fn source_body_checks_original_source_and_never_retains_its_owner() {
    let owner = Arc::new(AtomicBool::new(false));
    let body = fixture_body(true).with_owner(owner.clone());
    let weak = Arc::downgrade(&body.data);
    let bindings = bindings(&body, 10);
    let compiled = compile(&body);
    let equal = fixture_body(true);
    assert!(bindings.check(&equal).is_ok());
    assert!(SourceBody::new(compile(&equal), bindings.clone()).is_ok());
    let other = fixture_body(false);
    assert!(bindings.check(&other).is_err());
    assert!(SourceBody::new(compile(&other), bindings.clone()).is_err());
    drop(body);
    assert!(weak.upgrade().is_none());
    assert_eq!(
        Arc::strong_count(&owner),
        1,
        "bindings do not keep original admission alive"
    );
    assert!(
        bindings.check(&equal).is_err(),
        "equal-live fallback cannot guess a released source"
    );
    let paired = SourceBody::new(compiled, bindings).unwrap();
    let attached = paired.clone().with_owner(owner.clone());
    assert!(
        attached
            .bindings()
            .check_compiled(attached.compiled())
            .is_ok()
    );
    drop(attached);
    drop(paired);
    assert_eq!(Arc::strong_count(&owner), 1);
}
#[test]
fn source_workers_keep_independent_body_contexts_and_member_source_collisions() {
    let body = fixture_body(true);
    let compiled = compile(&body);
    for n in [10, 30] {
        let paired = SourceBody::new(compiled.clone(), bindings(&body, n)).unwrap();
        let error = paired
            .worker()
            .evaluate(
                &[-1.],
                DerivativeOrder::Value,
                &mut ProviderBindings::new(),
                &Arc::new(AtomicBool::new(false)),
            )
            .unwrap_err();
        let MathError::Instance {
            instance,
            checked_members,
            checked_sources,
            occurrences,
            cause,
        } = &error
        else {
            panic!("missing actual instance attribution: {error}")
        };
        assert_eq!(*instance, id(n));
        assert_eq!(checked_members[&id(1)], id(n + 1));
        assert_eq!(checked_sources[&id(1)], id(n + 2));
        assert_eq!(occurrences[&id(3)], location(n + 3));
        assert!(
            matches!(cause.as_ref(),MathError::Validity(lineage) if lineage.source==id(1)&&lineage.members==[id(1)])
        );
        let diagnostic = error.boundary_diagnostic(pse_diagnostics::DiagnosticStage::Evaluation);
        assert_eq!(diagnostic.validity.as_ref().unwrap().source, id(n + 2));
        assert_eq!(diagnostic.validity.as_ref().unwrap().members, [id(n + 1)]);
    }
}
#[test]
fn source_worker_keeps_domain_disposition_and_direct_cancellation_deadline_limits() {
    let body = fixture_body(false);
    let paired = SourceBody::new(compile(&body), bindings(&body, 10)).unwrap();
    let error = paired
        .worker()
        .evaluate(
            &[-1.],
            DerivativeOrder::Value,
            &mut ProviderBindings::new(),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap_err();
    assert!(matches!(error, MathError::Instance { .. }));
    assert!(
        matches!(error.instance_cause(), MathError::Domain { .. }),
        "same scientific selector class through receiving attribution"
    );
    let error = paired
        .worker()
        .evaluate(
            &[1.],
            DerivativeOrder::Value,
            &mut ProviderBindings::new(),
            &Arc::new(AtomicBool::new(true)),
        )
        .unwrap_err();
    assert!(matches!(error, MathError::Cancelled));
    let expired = ExecutionScope::new(
        pse_diagnostics::Cancellation::new(false),
        Some(std::time::Instant::now() - std::time::Duration::from_secs(1)),
    );
    let error = paired
        .worker_scoped(expired)
        .evaluate(
            &[1.],
            DerivativeOrder::Value,
            &mut ProviderBindings::new(),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap_err();
    assert!(matches!(error, MathError::Scope(ProviderError::Deadline)));
    let error = paired
        .bindings()
        .evaluation_error(MathError::Limit("bounded fixture"));
    assert!(matches!(error, MathError::Limit("bounded fixture")));
}
#[test]
fn source_product_worker_retains_the_original_body_attribution() {
    let body = fixture_body(false);
    let control = pse_diagnostics::Cancellation::new(false);
    let product = body
        .prepare_support(&[0], &[0], DerivativeOrder::First, &control.flag())
        .unwrap()
        .compile_product(
            crate::jets::ProductKind::JacobianAction,
            Optimization::default(),
            EvaluationLimits::default(),
            &ExecutionScope::new(control.clone(), None),
        )
        .unwrap();
    let paired = SourceBody::new(Arc::new(product), bindings(&body, 10)).unwrap();
    let mut worker = paired.worker();
    let mut trial = TrialEvaluation::default();
    let mut output = [f64::NAN];
    let error = worker
        .evaluate_product_trial_into(
            &[-1.],
            ProductValues::JacobianAction { direction: &[1.] },
            &mut ProviderBindings::new(),
            &control.flag(),
            &mut output,
            &mut trial,
        )
        .unwrap_err();
    assert!(matches!(&error,MathError::Instance {instance,..} if *instance==id(10)));
    assert!(matches!(error.instance_cause(),MathError::Domain {source_id,..} if *source_id==id(3)));
    assert!(
        output[0].is_nan(),
        "no derivative action published after an original guard failure"
    );
}

#[test]
fn bound_source_preparation_preserves_attribution_outputs_and_derivatives() {
    use crate::typed::{Binary, BodyBuilder, BodyLimits};
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let quantity = registry.neutral_dimensionless().unwrap();
    let mut builder = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &pse_quantity::standard::StandardInvariantChecker,
        2,
        BodyLimits::default(),
    )
    .unwrap();
    let x = builder.input(0, quantity, Default::default(), id(1)).unwrap();
    let y = builder.input(1, quantity, Default::default(), id(2)).unwrap();
    let sum = builder.binary(Binary::Add, x.clone(), y.clone(), None, id(3)).unwrap();
    let product = builder.binary(Binary::Mul, x, y, None, id(4)).unwrap();
    let control = pse_diagnostics::Cancellation::new(false);
    let body = SourceBody::prepare(
        builder,
        &[sum, product],
        |prepared| SourceBindings::new(id(10), prepared, Default::default(), Default::default(), Default::default()),
        DerivativeOrder::Second,
        Optimization::default(),
        EvaluationLimits::default(),
        &ExecutionScope::new(control.clone(), None),
    )
    .unwrap();
    assert_eq!(body.bindings().instance(), id(10));
    let result = body.worker().evaluate(
        &[2., 3.],
        DerivativeOrder::Second,
        &mut ProviderBindings::new(),
        &control.flag(),
    ).unwrap();
    assert_eq!(result.values, [5., 6.]);
    assert_eq!(result.jacobian, [1., 1., 3., 2.]);
    assert_eq!(result.hessians, [0., 0., 0., 0., 0., 1., 1., 0.]);
}

#[test]
fn bound_source_preparation_refuses_attribution_before_compilation_and_keeps_controls() {
    use crate::typed::{BodyBuilder, BodyLimits};
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let quantity = registry.neutral_dimensionless().unwrap();
    let build = || {
        let mut builder = BodyBuilder::new(
            crate::initialize().unwrap(),
            &registry,
            &pse_quantity::standard::StandardInvariantChecker,
            1,
            BodyLimits::default(),
        ).unwrap();
        let value = builder.input(0, quantity, Default::default(), id(1)).unwrap();
        (builder, value)
    };
    let compile = |instance, limits, cancelled| {
        let (builder, value) = build();
        SourceBody::prepare(
            builder,
            &[value],
            |prepared| SourceBindings::new(instance, prepared, Default::default(), Default::default(), Default::default()),
            DerivativeOrder::Value,
            Optimization::default(),
            limits,
            &ExecutionScope::new(pse_diagnostics::Cancellation::new(cancelled), None),
        )
    };
    assert!(matches!(compile(SemanticId::NIL, EvaluationLimits::default(), true), Err(MathError::Contract(_))));
    assert!(matches!(compile(id(10), EvaluationLimits { operations: 0, ..EvaluationLimits::default() }, false), Err(MathError::Limit("zero evaluation allowance"))));
    assert!(matches!(compile(id(10), EvaluationLimits::default(), true), Err(MathError::Cancelled)));
    let unrelated = fixture_body(false);
    let unrelated_bindings = bindings(&unrelated, 10);
    let (builder, value) = build();
    let result = SourceBody::prepare(
        builder,
        &[value],
        |_| Ok(unrelated_bindings),
        DerivativeOrder::Value,
        Optimization::default(),
        EvaluationLimits::default(),
        &ExecutionScope::new(pse_diagnostics::Cancellation::new(false), None),
    );
    assert!(matches!(result, Err(MathError::Contract(_))));
}
