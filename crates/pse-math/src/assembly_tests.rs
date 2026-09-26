// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use crate::{
    assembly::*,
    binding::*,
    coefficients::GramCertificate,
    guarded::PreparedBody,
    jets::EvaluationLimits,
    library::Optimization,
    typed::{Binary, BodyBuilder, BodyLimits},
};
use pse_ids::{ContentHash, SemanticId};
use pse_kernels::{DerivativeOrder, Port};
use pse_quantity::{
    IndexSet,
    standard::{StandardInvariantChecker, ids, standard_registry},
};
use std::{
    collections::BTreeMap,
    sync::{Arc, atomic::AtomicBool},
};
fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
fn fixture(alias: bool, fixed: bool) -> (Arc<CaseAssembly>, CaseValues) {
    let registry = standard_registry().unwrap();
    let quantity = ids::quantity("neutral");
    let port = |n| Port {
        id: id(n),
        quantity,
        unit: registry.quantity_type(quantity).unwrap().canonical_unit,
    };
    let mut builder = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &StandardInvariantChecker,
        2,
        BodyLimits::default(),
    )
    .unwrap();
    let a = builder.input(0, quantity, IndexSet::new(), id(20)).unwrap();
    let b = builder.input(1, quantity, IndexSet::new(), id(21)).unwrap();
    let product = builder
        .binary(Binary::Mul, a.clone(), b.clone(), None, id(22))
        .unwrap();
    let sum = builder.binary(Binary::Add, a, b, None, id(23)).unwrap();
    let body = Arc::new(builder.prepare(&[product, sum]).unwrap());
    let key = ContentHash::from_bytes([1; 32]);
    let variable = |n| Variable {
        port: port(n),
        fixed,
        domain: VariableDomain::Continuous,
        lower: None,
        upper: None,
    };
    let slots = vec![
        SlotBinding::new(&port(1), &port(1), &registry).unwrap(),
        SlotBinding::new(&port(if alias { 1 } else { 2 }), &port(2), &registry).unwrap(),
    ];
    let contributions = vec![
        Contribution {
            output: 0,
            target: Target::Objective,
            scale: 1.0,
        },
        Contribution {
            output: 1,
            target: Target::Row(id(10)),
            scale: 2.0,
        },
        Contribution {
            output: 1,
            target: Target::Row(id(10)),
            scale: 3.0,
        },
    ];
    let structure = Arc::new(
        CaseStructure::new(
            vec![variable(2), variable(1)],
            vec![],
            vec![InstanceBinding {
                instance: id(9),
                body: key,
                slots,
                contributions,
            }],
            vec![
                Row {
                    id: id(11),
                    quantity,
                    lower: 0.0,
                    upper: 0.0,
                },
                Row {
                    id: id(10),
                    quantity,
                    lower: 0.0,
                    upper: 100.0,
                },
            ],
            Some(Objective {
                quantity,
                sense: ObjectiveSense::Minimize,
            }),
            CaseLimits::default(),
        )
        .unwrap(),
    );
    let assembly = Arc::new(
        Arc::new(
            CasePlan::prepare(
                structure,
                BTreeMap::from([(key, body)]),
                &registry,
                DerivativeOrder::Second,
                AssemblyLimits::default(),
                &Arc::new(AtomicBool::new(false)),
            )
            .unwrap(),
        )
        .compile(
            Optimization::default(),
            EvaluationLimits::default(),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap(),
    );
    (
        assembly,
        CaseValues {
            scalars: BTreeMap::from([(id(1), 2.0), (id(2), 3.0)]),
        },
    )
}
#[test]
fn aliases_repeated_rows_isolates_and_stable_refill() {
    crate::initialize().unwrap();
    let (a, mut x) = fixture(true, false);
    let mut w = a.worker(BTreeMap::new(), Arc::new(AtomicBool::new(false)));
    assert_eq!(a.columns(), &[id(1), id(2)]);
    assert_eq!(w.objective(&x).unwrap(), 4.0);
    assert_eq!(w.gradient(&x).unwrap(), vec![4.0, 0.0]);
    assert_eq!(w.constraints(&x).unwrap(), vec![20.0, 0.0]);
    let ptr = w.jacobian(&x).unwrap().val().as_ptr();
    assert_eq!(w.jacobian(&x).unwrap().to_dense()[(0, 0)], 10.0);
    assert_eq!(
        w.hessian(&x, 1.0, &[0.0, 0.0]).unwrap().to_dense()[(0, 0)],
        2.0
    );
    assert_eq!(
        w.hessian(&x, 3.0, &[5.0, 7.0]).unwrap().to_dense()[(0, 0)],
        6.0
    );
    x.scalars.insert(id(1), 0.0);
    assert_eq!(w.gradient(&x).unwrap(), vec![0.0, 0.0]);
    assert_eq!(w.jacobian(&x).unwrap().val().as_ptr(), ptr);
    assert_eq!(w.hessian(&x, 1.0, &[0.0, 0.0]).unwrap().val(), &[2.0]);
    let mut out = vec![0.0; 2];
    w.jacobian_product(&x, &[4.0, 9.0], &mut out).unwrap();
    assert_eq!(out, vec![40.0, 0.0]);
}
#[test]
fn presolve_projection_keeps_alias_coefficients_and_parameter_identity() {
    crate::initialize().unwrap();
    let (a, mut values) = fixture(true, false);
    let cancel = Arc::new(AtomicBool::new(false));
    let f = a.presolve_facts(&values, 1000, &cancel).unwrap();
    assert_eq!(
        f.affine[0].as_ref().unwrap().entries,
        BTreeMap::from([(0, 10.0)])
    );
    assert_eq!(f.affine[1].as_ref().unwrap().constant, 0.0);
    assert_eq!(f.objective_linear, vec![false, true]);
    assert!(
        f.obligations
            .values()
            .all(|s| *s == crate::presolve::ObligationStatus::Discharged)
    );
    assert!(f.complete.iter().all(|v| *v));
    assert!(f.tapes.iter().all(|t| t.first_invalid_slot().is_none()));
    values.scalars.insert(id(1), 9.0);
    assert_eq!(f.key, a.presolve_facts(&values, 1000, &cancel).unwrap().key);
    let (fixed, mut values) = fixture(true, true);
    let f = fixed.presolve_facts(&values, 1000, &cancel).unwrap();
    assert_eq!(f.affine[0].as_ref().unwrap().constant, 20.0);
    values.scalars.insert(id(1), 4.0);
    assert!(!f.matches(&fixed, &values));
    assert_ne!(
        f.key,
        fixed.presolve_facts(&values, 1000, &cancel).unwrap().key
    );
}
#[test]
fn distinct_columns_keep_one_off_diagonal_and_coefficient_views() {
    crate::initialize().unwrap();
    let (a, x) = fixture(false, false);
    let mut w = a.worker(BTreeMap::new(), Arc::new(AtomicBool::new(false)));
    assert_eq!(w.gradient(&x).unwrap(), vec![3.0, 2.0]);
    let h = w.hessian(&x, 1.0, &[0.0, 0.0]).unwrap().to_dense();
    assert_eq!(h[(1, 0)], 1.0);
    assert_eq!(h[(0, 1)], 0.0);
    let c = a
        .coefficients(&x, 1000, &Arc::new(AtomicBool::new(false)))
        .unwrap();
    assert_eq!(c.constraints.to_dense()[(0, 0)], 5.0);
    assert_eq!(c.constraints.to_dense()[(0, 1)], 5.0);
    assert_eq!(c.hessian.to_dense()[(0, 1)], 1.0);
    let factors = faer::Mat::from_fn(1, 2, |_, _| 1.0);
    assert!(GramCertificate::new(&c.hessian, 1.0, &factors, &[1.0], 100).is_err());
}

#[test]
fn convexity_distinguishes_exact_numerical_indefinite_and_inconclusive() {
    use crate::convexity::*;
    let (a, values) = fixture(false, false);
    let cancel = Arc::new(AtomicBool::new(false));
    let mut c = a.coefficients(&values, 1000, &cancel).unwrap();
    let limits = ConvexityLimits {
        bytes: 1 << 20,
        exact_operations: 1000,
    };
    let numerical = ConvexityPolicy::Numerical {
        absolute: 1e-12,
        relative: 1e-12,
    };
    let negative = c
        .convexity(1.0, &[1.0, 1.0], 1.0, numerical, limits, &cancel)
        .unwrap();
    assert!(matches!(
        negative.assessment(),
        ConvexityAssessment::Indefinite { .. }
    ));
    let mut q =
        crate::sparse::AssemblyMatrix::new(2, 2, &[(0, 0), (0, 1), (1, 0), (1, 1)], 10).unwrap();
    for (i, v) in [3.0, 1.0, 1.0, 3.0].into_iter().enumerate() {
        q.add(i, v).unwrap();
    }
    c.hessian = q.matrix().clone();
    assert!(negative.validate_matrix(&c.hessian, 1.0).is_err());
    let exact = c
        .convexity(
            1.0,
            &[1.0, 1.0],
            1.0,
            ConvexityPolicy::Exact,
            limits,
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        exact.assessment(),
        ConvexityAssessment::Inconclusive(InconclusiveReason::NoExactWitness)
    ));
    let approx = c
        .convexity(1.0, &[1.0, 1.0], 1.0, numerical, limits, &cancel)
        .unwrap();
    assert!(matches!(
        approx.assessment(),
        ConvexityAssessment::NumericalPsd { .. }
    ));
    assert!(approx.validate_policy(numerical, &[1.0, 1.0], 1.0).is_ok());
    assert!(
        approx
            .validate_policy(ConvexityPolicy::Exact, &[1.0, 1.0], 1.0)
            .is_err()
    );
    assert!(approx.validate_policy(numerical, &[2.0, 1.0], 1.0).is_err());
    let limited = c
        .convexity(
            1.0,
            &[1.0, 1.0],
            1.0,
            numerical,
            ConvexityLimits { bytes: 1, ..limits },
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        limited.assessment(),
        ConvexityAssessment::Inconclusive(InconclusiveReason::ResourceLimit)
    ));
    assert_ne!(limited.key(), approx.key());
    q.clear();
    q.add(0, 2.0).unwrap();
    c.hessian = q.matrix().clone();
    let rank_deficient = c
        .convexity(
            1.0,
            &[1.0, 1.0],
            1.0,
            ConvexityPolicy::Exact,
            limits,
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        rank_deficient.assessment(),
        ConvexityAssessment::Exact(_)
    ));
}
#[test]
fn all_fixed_uses_constant_math_and_parameter_changes_reclassify() {
    crate::initialize().unwrap();
    let (a, mut x) = fixture(true, true);
    let mut w = a.worker(BTreeMap::new(), Arc::new(AtomicBool::new(false)));
    assert!(a.columns().is_empty());
    assert_eq!(w.objective(&x).unwrap(), 4.0);
    assert!(w.gradient(&x).unwrap().is_empty());
    let cancel = Arc::new(AtomicBool::new(false));
    let c = a.coefficients(&x, 1000, &cancel).unwrap();
    assert_eq!(c.objective_constant, 4.0);
    x.scalars.insert(id(1), 3.0);
    let d = a.coefficients(&x, 1000, &cancel).unwrap();
    assert_eq!(d.objective_constant, 9.0);
    assert_ne!(c.assumptions, d.assumptions);
}
#[test]
fn gram_evidence_is_exact_nonnegative_and_current() {
    crate::initialize().unwrap();
    let (a, x) = fixture(true, false);
    let mut c = a
        .coefficients(&x, 1000, &Arc::new(AtomicBool::new(false)))
        .unwrap();
    let factors = faer::Mat::from_fn(1, 2, |_, j| if j == 0 { 1.0 } else { 0.0 });
    let certificate = GramCertificate::new(&c.hessian, 1.0, &factors, &[2.0], 100).unwrap();
    certificate.validate(&c.hessian, 1.0).unwrap();
    assert!(certificate.validate(&c.hessian, -1.0).is_err());
    assert!(GramCertificate::new(&c.hessian, 1.0, &factors, &[-2.0], 100).is_err());
    c.hessian.val_mut()[0] = 3.0;
    assert!(certificate.validate(&c.hessian, 1.0).is_err());
}
#[test]
fn sparse_limits_fail_before_library_allocation() {
    crate::initialize().unwrap();
    assert!(crate::sparse::AssemblyMatrix::new(2, 2, &[(2, 0)], 100).is_err());
    assert!(crate::sparse::AssemblyMatrix::new(usize::MAX, 1, &[], i32::MAX as usize).is_err());
}
#[test]
fn coefficient_projection_preserves_erased_domain_obligations() {
    crate::initialize().unwrap();
    let registry = standard_registry().unwrap();
    let quantity = ids::quantity("neutral");
    let unit = registry.quantity_type(quantity).unwrap().canonical_unit;
    let port = Port {
        id: id(1),
        quantity,
        unit,
    };
    let mut b = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &StandardInvariantChecker,
        1,
        BodyLimits::default(),
    )
    .unwrap();
    let x = b.input(0, quantity, IndexSet::new(), id(20)).unwrap();
    let out = b.binary(Binary::Div, x.clone(), x, None, id(22)).unwrap();
    let body: Arc<PreparedBody> = Arc::new(b.prepare(&[out]).unwrap());
    let key = ContentHash::from_bytes([2; 32]);
    let prepare = |lower| {
        let structure = Arc::new(
            CaseStructure::new(
                vec![Variable {
                    port: port.clone(),
                    fixed: false,
                    domain: VariableDomain::Continuous,
                    lower: Some(lower),
                    upper: Some(3.0),
                }],
                vec![],
                vec![InstanceBinding {
                    instance: id(9),
                    body: key,
                    slots: vec![SlotBinding::new(&port, &port, &registry).unwrap()],
                    contributions: vec![Contribution {
                        output: 0,
                        target: Target::Objective,
                        scale: 1.0,
                    }],
                }],
                vec![],
                Some(Objective {
                    quantity,
                    sense: ObjectiveSense::Minimize,
                }),
                CaseLimits::default(),
            )
            .unwrap(),
        );
        Arc::new(
            CasePlan::prepare(
                structure,
                BTreeMap::from([(key, body.clone())]),
                &registry,
                DerivativeOrder::Second,
                AssemblyLimits::default(),
                &Arc::new(AtomicBool::new(false)),
            )
            .unwrap(),
        )
        .compile(
            Optimization::default(),
            EvaluationLimits::default(),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
    };
    let values = CaseValues {
        scalars: BTreeMap::from([(id(1), 2.0)]),
    };
    let cancel = Arc::new(AtomicBool::new(false));
    assert_eq!(
        prepare(0.0)
            .presolve_facts(&values, 100, &cancel)
            .unwrap()
            .obligations[&id(9)],
        crate::presolve::ObligationStatus::Unestablished
    );
    assert_eq!(
        prepare(1.0)
            .presolve_facts(&values, 100, &cancel)
            .unwrap()
            .obligations[&id(9)],
        crate::presolve::ObligationStatus::Discharged
    );
    assert!(prepare(0.0).coefficients(&values, 100, &cancel).is_err());
    assert_eq!(
        prepare(1.0)
            .coefficients(&values, 100, &cancel)
            .unwrap()
            .objective_constant,
        1.0
    );
}

#[test]
fn scaled_gathers_factored_quadratics_and_parameter_class_changes() {
    crate::initialize().unwrap();
    use pse_quantity::UnitId;
    let standard = standard_registry().unwrap();
    let quantity = ids::quantity("neutral");
    let canonical = standard.quantity_type(quantity).unwrap().canonical_unit;
    // Extend the fixture registry with one representation unit for an otherwise unchanged type.
    let mut registry_builder = standard.to_builder();
    let unit = UnitId::from_id(id(70));
    let mut scaled = standard.unit(canonical).unwrap().clone();
    scaled.id = unit;
    scaled.symbol = "triple-neutral".into();
    scaled.scale_to_canonical = 3.0;
    registry_builder.unit(scaled);
    let registry = registry_builder.build().unwrap();
    let port = Port {
        id: id(1),
        quantity,
        unit,
    };
    let formal = Port {
        unit: canonical,
        ..port.clone()
    };
    let parameter = Port {
        id: id(2),
        ..formal.clone()
    };
    let mut builder = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &StandardInvariantChecker,
        2,
        BodyLimits::default(),
    )
    .unwrap();
    let x = builder.input(0, quantity, IndexSet::new(), id(20)).unwrap();
    let p = builder.input(1, quantity, IndexSet::new(), id(21)).unwrap();
    let sum = builder
        .binary(Binary::Add, x.clone(), p.clone(), None, id(22))
        .unwrap();
    let square = builder
        .binary(Binary::Mul, sum.clone(), sum, None, id(23))
        .unwrap();
    let objective = builder
        .binary(Binary::Mul, p, square, None, id(24))
        .unwrap();
    let body = Arc::new(builder.prepare(&[objective, x]).unwrap());
    let key = ContentHash::from_bytes([3; 32]);
    let structure = Arc::new(
        CaseStructure::new(
            vec![Variable {
                port: port.clone(),
                fixed: false,
                domain: VariableDomain::Continuous,
                lower: Some(-10.0),
                upper: Some(10.0),
            }],
            vec![parameter.clone()],
            vec![InstanceBinding {
                instance: id(9),
                body: key,
                slots: vec![
                    SlotBinding::new(&port, &formal, &registry).unwrap(),
                    SlotBinding::new(&parameter, &formal, &registry).unwrap(),
                ],
                contributions: vec![
                    Contribution {
                        output: 0,
                        target: Target::Objective,
                        scale: 1.0,
                    },
                    Contribution {
                        output: 1,
                        target: Target::Row(id(10)),
                        scale: 2.0,
                    },
                ],
            }],
            vec![Row {
                id: id(10),
                quantity,
                lower: -100.0,
                upper: 100.0,
            }],
            Some(Objective {
                quantity,
                sense: ObjectiveSense::Minimize,
            }),
            CaseLimits::default(),
        )
        .unwrap(),
    );
    let cancel = Arc::new(AtomicBool::new(false));
    let a = Arc::new(
        Arc::new(
            CasePlan::prepare(
                structure,
                BTreeMap::from([(key, body)]),
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
    let mut values = CaseValues {
        scalars: BTreeMap::from([(id(1), 2.0), (id(2), 1.0)]),
    };
    let mut worker = a.worker(BTreeMap::new(), cancel.clone());
    assert_eq!(worker.objective(&values).unwrap(), 49.0);
    assert_eq!(worker.gradient(&values).unwrap(), vec![42.0]);
    assert_eq!(worker.hessian(&values, 1.0, &[0.0]).unwrap().val(), &[18.0]);
    assert_eq!(worker.jacobian(&values).unwrap().val(), &[6.0]);
    let c = a.coefficients(&values, 100, &cancel).unwrap();
    assert_eq!(c.objective_constant, 1.0);
    assert_eq!(c.objective, vec![6.0]);
    assert_eq!(c.hessian.val(), &[18.0]);
    let factors = faer::Mat::from_fn(1, 1, |_, _| 3.0);
    let proof = GramCertificate::new(&c.hessian, 1.0, &factors, &[2.0], 100).unwrap();
    values.scalars.insert(id(2), -1.0);
    let d = a.coefficients(&values, 100, &cancel).unwrap();
    assert_ne!(c.assumptions, d.assumptions);
    assert!(proof.validate(&d.hessian, 1.0).is_err());
    assert_eq!(
        worker.hessian(&values, 1.0, &[0.0]).unwrap().val(),
        &[-18.0]
    );
}

#[test]
fn mathematical_facts_are_independent_of_requested_artifacts() {
    crate::initialize().unwrap();
    let (assembly, values) = fixture(false, false);
    let cancel = Arc::new(AtomicBool::new(false));
    let registry = standard_registry().unwrap();
    let plan = CasePlan::prepare(
        assembly.structure().clone().into(),
        assembly.bodies().clone(),
        &registry,
        DerivativeOrder::First,
        AssemblyLimits::default(),
        &cancel,
    )
    .unwrap();
    let bound = plan.presolve_facts(&values, 1000, &cancel).unwrap();
    let facts = crate::facts::ProblemFacts::from_plan(&plan, None, &bound).unwrap();
    assert_eq!(facts.derivatives, DerivativeOrder::Second);
    assert_eq!(facts.prepared_derivatives, DerivativeOrder::First);
    assert_eq!(facts.objective_degree, Some(2));
    assert_eq!(facts.affine_rows, vec![true, true]);
    assert!(!facts.coefficients);
}

#[test]
fn admitted_transcendentals_and_strict_guards_feed_library_fbbt() {
    use crate::Function;
    crate::initialize().unwrap();
    let registry = standard_registry().unwrap();
    let quantity = ids::quantity("neutral");
    let unit = registry.quantity_type(quantity).unwrap().canonical_unit;
    let port = Port {
        id: id(1),
        quantity,
        unit,
    };
    let mut b = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &StandardInvariantChecker,
        1,
        BodyLimits::default(),
    )
    .unwrap();
    let x = b.input(0, quantity, IndexSet::new(), id(20)).unwrap();
    let logarithm = b.unary(Function::Log, x.clone(), id(21)).unwrap();
    let exponential = b.unary(Function::Exp, x, id(22)).unwrap();
    let body = Arc::new(b.prepare(&[logarithm, exponential]).unwrap());
    let key = ContentHash::from_bytes([5; 32]);
    let structure = Arc::new(
        CaseStructure::new(
            vec![Variable {
                port: port.clone(),
                fixed: false,
                domain: VariableDomain::Continuous,
                lower: Some(0.0),
                upper: Some(3.0),
            }],
            vec![],
            vec![InstanceBinding {
                instance: id(9),
                body: key,
                slots: vec![SlotBinding::new(&port, &port, &registry).unwrap()],
                contributions: vec![
                    Contribution {
                        output: 0,
                        target: Target::Row(id(10)),
                        scale: 1.0,
                    },
                    Contribution {
                        output: 1,
                        target: Target::Row(id(11)),
                        scale: 1.0,
                    },
                ],
            }],
            vec![
                Row {
                    id: id(10),
                    quantity,
                    lower: 0.0,
                    upper: 1.0,
                },
                Row {
                    id: id(11),
                    quantity,
                    lower: 0.0,
                    upper: 10.0,
                },
            ],
            None,
            CaseLimits::default(),
        )
        .unwrap(),
    );
    let cancel = Arc::new(AtomicBool::new(false));
    let plan = CasePlan::prepare(
        structure,
        BTreeMap::from([(key, body)]),
        &registry,
        DerivativeOrder::First,
        AssemblyLimits::default(),
        &cancel,
    )
    .unwrap();
    let facts = plan
        .presolve_facts(
            &CaseValues {
                scalars: BTreeMap::new(),
            },
            1000,
            &cancel,
        )
        .unwrap();
    assert_eq!(facts.complete, vec![true, true]);
    assert_eq!(
        facts.signs[&id(1)],
        crate::presolve::GuardSign {
            positive: true,
            strict: true
        }
    );
    assert!(
        facts.tapes[0]
            .ops
            .iter()
            .any(|op| matches!(op, pounce_nlp::expression_provider::FbbtOp::Ln(_)))
    );
    assert!(
        facts.tapes[1]
            .ops
            .iter()
            .any(|op| matches!(op, pounce_nlp::expression_provider::FbbtOp::Exp(_)))
    );
    let interval = pounce_presolve::fbbt::forward_pass(&facts.tapes[1], &[1.0], &[2.0]).unwrap();
    let bound = pounce_presolve::fbbt::forward_result(&interval);
    assert!(bound.lo <= std::f64::consts::E && bound.hi >= 2.0_f64.exp());
}
