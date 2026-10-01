// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Real guarded-library execution, including inactive branches, cancellation and partials.
use crate::{
    MathError,
    guarded::*,
    library::{self, Optimization},
};
use pse_ids::SemanticId;
use pse_kernels::DerivativeOrder;
use pse_model::{
    applicability::*,
    generated::enums::{ModelingPermissionTarget as Target, ModelingValidityLayer as Layer},
};
use std::{
    collections::BTreeMap,
    sync::{Arc, atomic::AtomicBool},
};
use symbolica::atom::Atom;
fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
fn plan(region: Region, unknown: bool, extrapolate: bool) -> Arc<Node> {
    Arc::new(Node {
        claim: Claim {
            id: Some(id(1)),
            coverage: None,
            owner: id(2),
            owner_lineage: vec![id(2)],
            evidence: Some(id(3)),
            form: id(4),
            call: id(5),
            records: vec![id(6)],
            dependencies: vec![],
            layer: Layer::Data,
            basis: None,
            reason: None,
        },
        region,
        dependencies: vec![],
        inputs: vec![("T".into(), 0, id(7))],
        permissions: vec![Permission {
            id: id(8),
            scope: id(9),
            target_kind: Target::Records,
            targets: vec![id(6)],
            allow_unknown: unknown,
            allow_extrapolation: extrapolate,
        }],
    })
}
fn block(expr: Atom, slot: usize) -> Stage {
    Stage::Block {
        expressions: vec![expr],
        outputs: vec![slot],
        source: id(4),
    }
}
fn compile(
    stages: Vec<Stage>,
    slots: usize,
    outputs: Vec<usize>,
    order: DerivativeOrder,
) -> PreparedBody {
    crate::initialize().unwrap();
    PreparedBody::new(1, slots, outputs, stages, order).unwrap()
}
fn evaluate(
    body: &PreparedBody,
    output: usize,
    x: f64,
    order: DerivativeOrder,
) -> Result<Evaluation, MathError> {
    let flag = Arc::new(AtomicBool::new(false));
    let mut worker = body
        .compile(
            &[output],
            &[0],
            order,
            Optimization::default(),
            crate::jets::EvaluationLimits::default(),
            &flag,
        )?
        .worker();
    worker.evaluate(&[x], order, &mut BTreeMap::new(), &flag)
}
#[test]
fn applicability_observations_survive_erased_value_and_second_partials() {
    crate::initialize().unwrap();
    let x = library::formal(0).unwrap();
    let token = library::formal(2).unwrap();
    let body = compile(
        vec![
            Stage::Applicability {
                stages: vec![block(Atom::num(1), 1)],
                predicates: vec![1],
                inputs: vec![0],
                token: 2,
                plan: plan(Region::Predicate(0), false, false),
            },
            block((&x - &x) + token, 3),
        ],
        4,
        vec![3],
        DerivativeOrder::Second,
    );
    let evaluated = evaluate(&body, 0, 420., DerivativeOrder::Second).unwrap();
    assert_eq!(evaluated.values, vec![0.]);
    assert_eq!(evaluated.jacobian, vec![0.]);
    assert_eq!(evaluated.hessians, vec![0.]);
    assert_eq!(evaluated.applicability.len(), 1);
    assert_eq!(evaluated.applicability[0].inputs[0].value, 420.);
}
#[test]
fn missing_evidence_refuses_even_a_numerically_cancelled_output() {
    crate::initialize().unwrap();
    let token = library::formal(1).unwrap();
    let body = compile(
        vec![
            Stage::Applicability {
                stages: vec![],
                predicates: vec![],
                inputs: vec![0],
                token: 1,
                plan: plan(Region::Unknown, false, true),
            },
            block(token, 2),
        ],
        3,
        vec![2],
        DerivativeOrder::Second,
    );
    assert!(matches!(
        evaluate(&body, 0, 1., DerivativeOrder::Second),
        Err(MathError::Applicability(_))
    ));
}
#[test]
fn inactive_branch_and_undemanded_output_have_no_evidence_observations() {
    crate::initialize().unwrap();
    let x = library::formal(0).unwrap();
    let token = library::formal(2).unwrap();
    let unknown = Stage::Applicability {
        stages: vec![],
        predicates: vec![],
        inputs: vec![0],
        token: 2,
        plan: plan(Region::Unknown, false, false),
    };
    let stages = vec![
        block(Atom::num(0), 1),
        Stage::Branch {
            continuity: DerivativeOrder::Value,
            comparison: Comparison::Lt,
            left: 0,
            right: 1,
            then: vec![unknown, block(token, 3)],
            otherwise: vec![block(x, 3)],
        },
    ];
    let body = compile(stages, 4, vec![3], DerivativeOrder::Value);
    assert!(
        evaluate(&body, 0, 2., DerivativeOrder::Value)
            .unwrap()
            .applicability
            .is_empty()
    );
    assert!(matches!(
        evaluate(&body, 0, -2., DerivativeOrder::Value),
        Err(MathError::Applicability(_))
    ));
}
#[test]
fn neither_permission_can_waive_an_invalid_mathematical_domain() {
    crate::initialize().unwrap();
    let token = library::formal(1).unwrap();
    let body = compile(
        vec![
            Stage::Applicability {
                stages: vec![Stage::Require {
                    argument: 0,
                    condition: Condition::Positive,
                    order: DerivativeOrder::Value,
                    source: id(20),
                    lineage: None,
                }],
                predicates: vec![],
                inputs: vec![0],
                token: 1,
                plan: plan(Region::Unknown, true, true),
            },
            block(token, 2),
        ],
        3,
        vec![2],
        DerivativeOrder::Second,
    );
    assert!(
        matches!(evaluate(&body,0,-1.,DerivativeOrder::Value),Err(MathError::Domain {source_id,..}) if source_id==id(20))
    );
}

#[test]
fn malformed_capture_is_fatal_even_when_both_permissions_are_granted() {
    crate::initialize().unwrap();
    let token = library::formal(1).unwrap();
    let stages = vec![
        Stage::Applicability {
            stages: vec![],
            predicates: vec![],
            inputs: vec![],
            token: 1,
            plan: plan(Region::Unknown, true, true),
        },
        block(token, 2),
    ];
    assert!(matches!(
        PreparedBody::new(1, 3, vec![2], stages, DerivativeOrder::Value),
        Err(MathError::Contract(_))
    ));
}

#[test]
fn guarded_unknown_admission_matches_only_checked_nominal_ancestor_family() {
    crate::initialize().unwrap();
    for target in [id(2), id(22)] {
        let mut claim_plan = (*plan(Region::Unknown, true, false)).clone();
        claim_plan.claim.owner = id(21);
        claim_plan.claim.owner_lineage = vec![id(21), id(2)];
        claim_plan.permissions[0].target_kind = Target::Families;
        claim_plan.permissions[0].targets = vec![target];
        let body = compile(
            vec![
                Stage::Applicability {
                    stages: vec![],
                    predicates: vec![],
                    inputs: vec![0],
                    token: 1,
                    plan: Arc::new(claim_plan),
                },
                block(library::formal(1).unwrap(), 2),
            ],
            3,
            vec![2],
            DerivativeOrder::Value,
        );
        let result = evaluate(&body, 0, 300., DerivativeOrder::Value);
        if target == id(2) {
            let observation = result.unwrap().applicability.remove(0);
            assert_eq!(observation.claim.owner, id(21));
            assert_eq!(observation.claim.owner_lineage, vec![id(21), id(2)]);
            assert_eq!(observation.permissions[0].targets, vec![id(2)]);
        } else {
            assert!(matches!(result, Err(MathError::Applicability(_))));
        }
    }
}

#[test]
fn numerical_prerequisite_keeps_input_value_and_partials_but_requires_source_effect() {
    use crate::typed::{Binary, BodyBuilder, BodyLimits};
    use pse_quantity::{
        IndexSet,
        standard::{StandardInvariantChecker, ids, standard_registry},
    };
    let registry = standard_registry().unwrap();
    let quantity = ids::quantity("neutral");
    let cancelled = Arc::new(AtomicBool::new(false));
    for allowed in [false, true] {
        let mut builder = BodyBuilder::new(
            crate::initialize().unwrap(),
            &registry,
            &StandardInvariantChecker,
            2,
            BodyLimits::default(),
        )
        .unwrap();
        let input = builder.input(0, quantity, IndexSet::new(), id(30)).unwrap();
        let source = builder.input(1, quantity, IndexSet::new(), id(31)).unwrap();
        let mut claim = (*plan(Region::Unknown, allowed, false)).clone();
        claim.inputs[0].2 = quantity.as_id();
        let obligation = builder
            .applicability(Arc::new(claim), |_| Ok((vec![], vec![source.clone()])))
            .unwrap();
        let prerequisite = builder.with_assumption(source, &obligation);
        let actual = builder.with_prerequisite(input.clone(), &prerequisite);
        assert_eq!(actual.physical_contract(), input.physical_contract());
        let erased = builder
            .binary(Binary::Sub, actual.clone(), actual.clone(), None, id(32))
            .unwrap();
        let body = builder
            .finish(
                &[actual, erased],
                DerivativeOrder::Second,
                Optimization::default(),
                &cancelled,
            )
            .unwrap();
        let result = body.worker().evaluate(
            &[7., 999.],
            DerivativeOrder::Second,
            &mut BTreeMap::new(),
            &cancelled,
        );
        if allowed {
            let result = result.unwrap();
            assert_eq!(result.values, [7., 0.]);
            assert_eq!(result.jacobian, [1., 0., 0., 0.]);
            assert!(result.hessians.iter().all(|value| *value == 0.));
            assert_eq!(result.applicability.len(), 1);
            assert_eq!(result.applicability[0].inputs[0].value, 999.);
            assert!(result.applicability[0].unknown_allowed);
        } else {
            assert!(matches!(result, Err(MathError::Applicability(_))));
        }
    }
}
