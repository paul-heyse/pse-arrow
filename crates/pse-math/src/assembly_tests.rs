// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use crate::{
    assembly::*,
    binding::*,
    convexity::{Definiteness, GramCertificate},
    guarded::PreparedBody,
    index::{Addend, GlobalCol, GlobalRow},
    jets::EvaluationLimits,
    library::Optimization,
    typed::{Binary, BodyBuilder, BodyLimits},
};
use pse_ids::{ContentHash, SemanticId};
use pse_kernels::{DerivativeOrder, Port};
use pse_model::generated::enums::ModelingVariableDomain;
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
        domain: ModelingVariableDomain::Continuous,
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
            target: Target::PRIMARY,
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
                checked_members: Default::default(),
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
    let f = a.presolve_facts(&values, 10_000, &cancel).unwrap();
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
    assert_eq!(
        f.key,
        a.presolve_facts(&values, 10_000, &cancel).unwrap().key
    );
    let (fixed, mut values) = fixture(true, true);
    let f = fixed.presolve_facts(&values, 10_000, &cancel).unwrap();
    assert_eq!(f.affine[0].as_ref().unwrap().constant, 20.0);
    values.scalars.insert(id(1), 4.0);
    assert!(!f.matches(&fixed, &values));
    assert_ne!(
        f.key,
        fixed.presolve_facts(&values, 10_000, &cancel).unwrap().key
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
        .coefficients(&x, 10_000, &Arc::new(AtomicBool::new(false)))
        .unwrap();
    assert_eq!(c.constraints.to_dense()[(0, 0)], 5.0);
    assert_eq!(c.constraints.to_dense()[(0, 1)], 5.0);
    assert_eq!(c.hessian.to_dense()[(0, 1)], 1.0);
    // The bilinear objective x·y is indefinite, exactly.
    assert!(matches!(
        GramCertificate::certify(&c.hessian, 1.0, 100, &AtomicBool::new(false)).unwrap(),
        Definiteness::Indefinite
    ));
}

/// ADR-0121 Outcome 4: numerical PSD evidence is a separate, explicitly requested
/// assessment: it distinguishes numerical PSD, indefinite and inconclusive, is tied to the
/// request's policy and coordinates, and never establishes exact convexity, which the
/// exact certificate decides (here for the same nondiagonal matrix).
#[test]
fn numerical_convexity_distinguishes_psd_indefinite_and_inconclusive() {
    use crate::convexity::*;
    let (a, values) = fixture(false, false);
    let cancel = Arc::new(AtomicBool::new(false));
    let mut c = a.coefficients(&values, 10_000, &cancel).unwrap();
    let assess = |c: &crate::coefficients::Coefficients, bytes| {
        c.numerical_convexity(1.0, &[1.0, 1.0], 1.0, 1e-12, 1e-12, bytes, &cancel)
            .unwrap()
    };
    let negative = assess(&c, 1 << 20);
    assert!(matches!(
        negative.assessment(),
        ConvexityAssessment::Indefinite { .. }
    ));
    assert!(!negative.accepted());
    let hessian = [(0, 0), (0, 1), (1, 0), (1, 1)]
        .map(|(i, j)| crate::index::Entry::new(GlobalCol::new(i), GlobalCol::new(j)));
    let mut q = crate::sparse::AssemblyMatrix::hessian(2, &hessian, 10).unwrap();
    for (i, v) in [3.0, 1.0, 1.0, 3.0].into_iter().enumerate() {
        q.add(Addend::new(i), v).unwrap();
    }
    c.hessian = q.matrix().clone();
    assert!(negative.validate_matrix(&c.hessian, 1.0).is_err());
    // The exact decision certifies the nondiagonal matrix; the numerical one is PSD.
    assert!(matches!(
        GramCertificate::certify(&c.hessian, 1.0, 1000, &cancel).unwrap(),
        Definiteness::Psd(_)
    ));
    let numerical = ConvexityPolicy::Numerical {
        absolute: 1e-12,
        relative: 1e-12,
    };
    let approx = assess(&c, 1 << 20);
    assert!(matches!(
        approx.assessment(),
        ConvexityAssessment::NumericalPsd { .. }
    ));
    assert!(approx.accepted());
    assert!(approx.validate_policy(numerical, &[1.0, 1.0], 1.0).is_ok());
    assert!(
        approx
            .validate_policy(ConvexityPolicy::Exact, &[1.0, 1.0], 1.0)
            .is_err()
    );
    assert!(approx.validate_policy(numerical, &[2.0, 1.0], 1.0).is_err());
    let limited = assess(&c, 1);
    assert!(matches!(
        limited.assessment(),
        ConvexityAssessment::Inconclusive(InconclusiveReason::ResourceLimit)
    ));
    assert_ne!(limited.key(), approx.key());
    // Tolerances are the request's: none is not a numerical request.
    assert!(
        c.numerical_convexity(1.0, &[1.0, 1.0], 1.0, 0.0, 0.0, 1 << 20, &cancel)
            .is_err()
    );
    q.clear();
    q.add(Addend::new(0), 2.0).unwrap();
    c.hessian = q.matrix().clone();
    // A rank-deficient diagonal matrix is certified exactly.
    assert!(matches!(
        GramCertificate::certify(&c.hessian, 1.0, 1000, &cancel).unwrap(),
        Definiteness::Psd(p) if p.factors().rank() == 1
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
    let c = a.coefficients(&x, 100_000, &cancel).unwrap();
    assert_eq!(c.objective_constant, 4.0);
    x.scalars.insert(id(1), 3.0);
    let d = a.coefficients(&x, 100_000, &cancel).unwrap();
    assert_eq!(d.objective_constant, 9.0);
    assert_ne!(c.assumptions, d.assumptions);
}
#[test]
fn gram_evidence_is_exact_nonnegative_and_current() {
    crate::initialize().unwrap();
    let (a, x) = fixture(true, false);
    let mut c = a
        .coefficients(&x, 10_000, &Arc::new(AtomicBool::new(false)))
        .unwrap();
    let never = AtomicBool::new(false);
    let Definiteness::Psd(certificate) =
        GramCertificate::certify(&c.hessian, 1.0, 100, &never).unwrap()
    else {
        panic!("x² is PSD")
    };
    certificate.validate(&c.hessian, 1.0).unwrap();
    assert!(certificate.validate(&c.hessian, -1.0).is_err());
    // Its maximization is not concave-certified.
    assert!(matches!(
        GramCertificate::certify(&c.hessian, -1.0, 100, &never).unwrap(),
        Definiteness::Indefinite
    ));
    c.hessian.val_mut()[0] = 3.0;
    assert!(certificate.validate(&c.hessian, 1.0).is_err());
}
#[test]
fn sparse_limits_fail_before_library_allocation() {
    crate::initialize().unwrap();
    let outside = [crate::index::Entry::new(
        GlobalRow::new(2),
        GlobalCol::new(0),
    )];
    assert!(crate::sparse::AssemblyMatrix::new(2, 2, &outside, 100).is_err());
    assert!(
        crate::sparse::AssemblyMatrix::new::<GlobalRow, GlobalCol>(
            usize::MAX,
            1,
            &[],
            i32::MAX as usize
        )
        .is_err()
    );
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
    let squared = b.binary(Binary::Mul, x.clone(), x, None, id(21)).unwrap();
    let out = b
        .binary(Binary::Div, squared.clone(), squared, None, id(22))
        .unwrap();
    let body: Arc<PreparedBody> = Arc::new(b.prepare(&[out]).unwrap());
    let key = ContentHash::from_bytes([2; 32]);
    let prepare = |lower| {
        let structure = Arc::new(
            CaseStructure::new(
                vec![Variable {
                    port: port.clone(),
                    fixed: false,
                    domain: ModelingVariableDomain::Continuous,
                    lower: Some(lower),
                    upper: Some(3.0),
                }],
                vec![],
                vec![InstanceBinding {
                    checked_members: Default::default(),
                    instance: id(9),
                    body: key,
                    slots: vec![SlotBinding::new(&port, &port, &registry).unwrap()],
                    contributions: vec![Contribution {
                        output: 0,
                        target: Target::PRIMARY,
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
            .presolve_domain_facts(&values, 100, &cancel)
            .unwrap()
            .obligations[&id(9)],
        crate::presolve::ObligationStatus::Unestablished
    );
    assert_eq!(
        prepare(1.0)
            .presolve_domain_facts(&values, 100, &cancel)
            .unwrap()
            .obligations[&id(9)],
        crate::presolve::ObligationStatus::Discharged
    );
    assert!(matches!(
        prepare(1.0).presolve_domain_facts(&values, 1, &cancel),
        Err(crate::MathError::Limit("factorable projection extent"))
    ));
    assert!(prepare(1.0).coefficients(&values, 1, &cancel).is_err());
    assert!(prepare(0.0).coefficients(&values, 10_000, &cancel).is_err());
    assert_eq!(
        prepare(1.0)
            .coefficients(&values, 10_000, &cancel)
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
    // An atomic representation unit: its own authored scale, not a composition.
    scaled.definition = None;
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
                domain: ModelingVariableDomain::Continuous,
                lower: Some(-10.0),
                upper: Some(10.0),
            }],
            vec![parameter.clone()],
            vec![InstanceBinding {
                checked_members: Default::default(),
                instance: id(9),
                body: key,
                slots: vec![
                    SlotBinding::new(&port, &formal, &registry).unwrap(),
                    SlotBinding::new(&parameter, &formal, &registry).unwrap(),
                ],
                contributions: vec![
                    Contribution {
                        output: 0,
                        target: Target::PRIMARY,
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
    let c = a.coefficients(&values, 100_000, &cancel).unwrap();
    assert_eq!(c.objective_constant, 1.0);
    assert_eq!(c.objective, vec![6.0]);
    assert_eq!(c.hessian.val(), &[18.0]);
    let Definiteness::Psd(proof) =
        GramCertificate::certify(&c.hessian, 1.0, 100, &AtomicBool::new(false)).unwrap()
    else {
        panic!("18 is positive")
    };
    values.scalars.insert(id(2), -1.0);
    let d = a.coefficients(&values, 100_000, &cancel).unwrap();
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
    let facts = crate::facts::ProblemFacts::from_plan(&plan, None, &bound, &cancel).unwrap();
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
                domain: ModelingVariableDomain::Continuous,
                lower: Some(0.0),
                upper: Some(3.0),
            }],
            vec![],
            vec![InstanceBinding {
                checked_members: Default::default(),
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

#[test]
fn exhausted_factorable_projection_refuses_facts_and_preserves_original_evaluation() {
    let (assembly, values) = fixture(true, false);
    let cancel = Arc::new(AtomicBool::new(false));
    assert!(matches!(
        assembly.presolve_domain_facts(&values, 2, &cancel),
        Err(crate::MathError::Limit("factorable projection extent"))
    ));
    let mut worker = assembly.worker(BTreeMap::new(), cancel.clone());
    assert_eq!(worker.constraints(&values).unwrap(), vec![20., 0.]);
    let complete = assembly.presolve_facts(&values, 1000, &cancel).unwrap();
    assert!(complete.complete.iter().all(|v| *v));
    assert_eq!(
        complete.affine[0].as_ref().unwrap().entries,
        BTreeMap::from([(0, 10.)])
    );
    let (fixed, _) = fixture(true, true);
    assert!(
        fixed
            .presolve_facts(&CaseValues::default(), 2, &cancel)
            .is_err()
    );
    cancel.store(true, std::sync::atomic::Ordering::Release);
    assert!(matches!(
        assembly.presolve_facts(&values, 2, &cancel),
        Err(crate::MathError::Cancelled)
    ));
}

/// The parametric projection (Plan 22 S1) keeps the objective and every row, appends the
/// requested parameters to the free columns as derivative coordinates at second order, and
/// resolves the parameters as variable targets; only declared parameters are admitted.
#[test]
fn parametric_plan_keeps_objective_and_differentiates_parameters() {
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
    let x = builder.input(0, quantity, IndexSet::new(), id(20)).unwrap();
    let p = builder.input(1, quantity, IndexSet::new(), id(21)).unwrap();
    let product = builder
        .binary(Binary::Mul, x.clone(), p.clone(), None, id(22))
        .unwrap();
    let sum = builder.binary(Binary::Add, x, p, None, id(23)).unwrap();
    let body = Arc::new(builder.prepare(&[product, sum]).unwrap());
    let key = ContentHash::from_bytes([4; 32]);
    let structure = Arc::new(
        CaseStructure::new(
            vec![Variable {
                port: port(1),
                fixed: false,
                domain: ModelingVariableDomain::Continuous,
                lower: None,
                upper: None,
            }],
            vec![port(2)],
            vec![InstanceBinding {
                checked_members: Default::default(),
                instance: id(9),
                body: key,
                slots: vec![
                    SlotBinding::new(&port(1), &port(1), &registry).unwrap(),
                    SlotBinding::new(&port(2), &port(2), &registry).unwrap(),
                ],
                contributions: vec![
                    Contribution {
                        output: 0,
                        target: Target::PRIMARY,
                        scale: 1.0,
                    },
                    Contribution {
                        output: 1,
                        target: Target::Row(id(10)),
                        scale: 1.0,
                    },
                ],
            }],
            vec![Row {
                id: id(10),
                quantity,
                lower: 0.0,
                upper: 10.0,
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
    let plan = CasePlan::prepare(
        structure,
        BTreeMap::from([(key, body)]),
        &registry,
        DerivativeOrder::First,
        AssemblyLimits::default(),
        &cancel,
    )
    .unwrap();
    assert_eq!(plan.columns(), &[id(1)]);
    assert!(plan.parameter_targets().is_empty());
    for refused in [&[][..], &[id(1)][..], &[id(3)][..], &[id(2), id(2)][..]] {
        assert!(
            plan.parametric(refused, DerivativeOrder::Second, &registry, &cancel)
                .is_err()
        );
    }
    // Root response requests physical First partials without constructing Hessians.
    let first = Arc::new(
        plan.parametric(&[id(2)], DerivativeOrder::First, &registry, &cancel)
            .unwrap(),
    );
    assert_eq!(first.order(), DerivativeOrder::First);
    let first_assembly = Arc::new(
        first
            .compile(
                Optimization::default(),
                EvaluationLimits::default(),
                &cancel,
            )
            .unwrap(),
    );
    let mut first_worker = first_assembly.worker(BTreeMap::new(), cancel.clone());
    let first_values = CaseValues {
        scalars: BTreeMap::from([(id(1), 2.), (id(2), 3.)]),
    };
    assert_eq!(
        first_worker.jacobian(&first_values).unwrap().to_dense()[(0, 1)],
        1.
    );
    assert_eq!(first_worker.gradient(&first_values).unwrap(), vec![3., 2.]);
    let parametric = Arc::new(
        plan.parametric(&[id(2)], DerivativeOrder::Second, &registry, &cancel)
            .unwrap(),
    );
    assert_eq!(parametric.columns(), &[id(1), id(2)]);
    assert_eq!(parametric.order(), DerivativeOrder::Second);
    let targets = parametric.parameter_targets();
    assert_eq!(targets.len(), 1);
    assert_eq!(
        (targets[0].id, targets[0].kind),
        (
            id(2),
            pse_model::generated::enums::NumericalTarget::Variable
        )
    );
    assert!(
        parametric
            .numerical_targets(&registry)
            .unwrap()
            .iter()
            .any(|t| t.id == id(2))
    );
    // f = x·p and g = x + p at (x, p) = (2, 3): ∇f = (p, x), ∂²f/∂x∂p = 1, ∇g = (1, 1).
    let assembly = Arc::new(
        parametric
            .compile(
                Optimization::default(),
                EvaluationLimits::default(),
                &cancel,
            )
            .unwrap(),
    );
    let mut worker = assembly.worker(BTreeMap::new(), cancel.clone());
    let values = CaseValues {
        scalars: BTreeMap::from([(id(1), 2.0), (id(2), 3.0)]),
    };
    assert_eq!(worker.objective(&values).unwrap(), 6.0);
    assert_eq!(worker.gradient(&values).unwrap(), vec![3.0, 2.0]);
    assert_eq!(worker.jacobian(&values).unwrap().to_dense()[(0, 1)], 1.0);
    let hessian = worker.hessian(&values, 1.0, &[0.0]).unwrap().to_dense();
    assert_eq!(hessian[(1, 0)], 1.0);
}

/// A structure with several objectives keeps them in lexicographic order (ADR-0111): the
/// first is the primary objective every evaluation reads, and each later one is projected
/// to linear coefficients for a native lexicographic route only.
#[test]
fn lexicographic_structure_projects_every_level() {
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
    let sum = builder
        .binary(Binary::Add, a.clone(), b.clone(), None, id(22))
        .unwrap();
    let difference = builder
        .binary(Binary::Sub, a.clone(), b.clone(), None, id(23))
        .unwrap();
    let product = builder.binary(Binary::Mul, a, b, None, id(24)).unwrap();
    let body = Arc::new(builder.prepare(&[sum, difference, product]).unwrap());
    let key = ContentHash::from_bytes([2; 32]);
    let variable = |n| Variable {
        port: port(n),
        fixed: false,
        domain: ModelingVariableDomain::Continuous,
        lower: Some(0.0),
        upper: Some(10.0),
    };
    let objective = |sense| Objective { quantity, sense };
    let degradation = Degradation {
        absolute: 0.5,
        relative: 0.0,
    };
    // Sum first, then twice the difference (or the product), on a row sum ≤ 4.
    let structure = |later: usize, levels: Vec<(Objective, Option<Degradation>)>| {
        CaseStructure::lexicographic(
            vec![variable(1), variable(2)],
            vec![],
            vec![InstanceBinding {
                checked_members: Default::default(),
                instance: id(9),
                body: key,
                slots: vec![
                    SlotBinding::new(&port(1), &port(1), &registry).unwrap(),
                    SlotBinding::new(&port(2), &port(2), &registry).unwrap(),
                ],
                contributions: vec![
                    Contribution {
                        output: 0,
                        target: Target::PRIMARY,
                        scale: 1.0,
                    },
                    Contribution {
                        output: later,
                        target: Target::Objective(1),
                        scale: 2.0,
                    },
                    Contribution {
                        output: 0,
                        target: Target::Row(id(10)),
                        scale: 1.0,
                    },
                ],
            }],
            vec![Row {
                id: id(10),
                quantity,
                lower: f64::NEG_INFINITY,
                upper: 4.0,
            }],
            levels,
            CaseLimits::default(),
        )
    };
    let levels = vec![
        (objective(ObjectiveSense::Maximize), Some(degradation)),
        (objective(ObjectiveSense::Minimize), None),
    ];
    let lexicographic = structure(1, levels.clone()).unwrap();
    assert!(lexicographic.lexicographic_levels());
    assert_eq!(lexicographic.degradations(), [degradation]);
    assert_eq!(
        lexicographic.objective().unwrap().sense,
        ObjectiveSense::Maximize
    );
    // One level, a degradation on the last level, a missing or negative one are refused.
    for levels in [
        vec![levels[0].clone()],
        vec![levels[0].clone(), (levels[1].0.clone(), Some(degradation))],
        vec![(levels[0].0.clone(), None), levels[1].clone()],
        vec![
            (
                levels[0].0.clone(),
                Some(Degradation {
                    absolute: -1.0,
                    relative: 0.0,
                }),
            ),
            levels[1].clone(),
        ],
    ] {
        assert!(structure(1, levels).is_err());
    }
    // The objectives and their degradations are structure.
    let tighter = structure(
        1,
        vec![
            (
                objective(ObjectiveSense::Maximize),
                Some(Degradation {
                    absolute: 0.25,
                    relative: 0.0,
                }),
            ),
            levels[1].clone(),
        ],
    )
    .unwrap();
    assert_ne!(tighter.key(), lexicographic.key());
    let cancel = Arc::new(AtomicBool::new(false));
    let plan = |structure: CaseStructure| {
        CasePlan::prepare(
            Arc::new(structure),
            BTreeMap::from([(key, body.clone())]),
            &registry,
            DerivativeOrder::Second,
            AssemblyLimits::default(),
            &cancel,
        )
        .unwrap()
    };
    let values = CaseValues {
        scalars: BTreeMap::from([(id(1), 3.0), (id(2), 1.0)]),
    };
    let linear = plan(lexicographic);
    // Every evaluation reads the primary objective: −(a + b) in minimization orientation.
    let assembly = Arc::new(
        Arc::new(linear.clone())
            .compile(
                Optimization::default(),
                EvaluationLimits::default(),
                &cancel,
            )
            .unwrap(),
    );
    let mut worker = assembly.worker(BTreeMap::new(), cancel.clone());
    assert_eq!(worker.objective(&values).unwrap(), -4.0);
    let coefficients = linear.coefficients(&values, 1000, &cancel).unwrap();
    assert_eq!(coefficients.objective, vec![1.0, 1.0]);
    assert_eq!(coefficients.lexicographic, vec![(vec![2.0, -2.0], 0.0)]);
    // A later level must be linear to be a coefficient model.
    let quadratic = plan(structure(2, levels).unwrap());
    let facts = quadratic.presolve_facts(&values, 1000, &cancel).unwrap();
    assert_eq!(facts.lexicographic_degree, None);
    assert!(matches!(
        facts.class_status,
        crate::presolve::ClassStatus::RuledOut(
            crate::presolve::ClassWitness::NonlinearLaterObjective { level: 1 }
        )
    ));
    assert!(!facts.coefficient_eligible());
    assert!(quadratic.coefficients(&values, 1000, &cancel).is_err());
}

#[test]
fn case_order_upgrade_preserves_original_limits_maps_and_weaker_products() {
    let (assembly, _) = fixture(true, false);
    let registry = standard_registry().unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let value = CasePlan::prepare(
        Arc::new(assembly.structure().clone()),
        assembly.bodies().clone(),
        &registry,
        DerivativeOrder::Value,
        AssemblyLimits {
            contributions: 4,
            ..AssemblyLimits::default()
        },
        &cancel,
    )
    .unwrap();
    assert!(
        value
            .supports()
            .iter()
            .all(|s| s.support().first.is_empty() && s.support().second.is_empty())
    );
    let original_incidence = value.incidence(&cancel).unwrap();
    assert_eq!(
        original_incidence[0].first_for_output(1).unwrap(),
        &[0, 1].into_iter().collect()
    );
    let first = value
        .prepare_order(DerivativeOrder::First, &cancel)
        .unwrap();
    assert_eq!(first.structure().key(), value.structure().key());
    assert_eq!(first.columns(), value.columns());
    assert_eq!(first.jacobian_pattern().nrows(), 2);
    assert_eq!(first.jacobian_pattern().ncols(), 2);
    assert!(
        first
            .supports()
            .iter()
            .zip(value.supports())
            .all(|(stronger, weaker)| {
                stronger.remaining_occurrences() < weaker.remaining_occurrences()
                    && stronger.support().second.is_empty()
            })
    );
    assert!(matches!(
        first.prepare_order(DerivativeOrder::Second, &cancel),
        Err(crate::MathError::Limit("case derivative contributions"))
    ));
    assert_eq!(first.order(), DerivativeOrder::First);
    assert!(
        first
            .supports()
            .iter()
            .all(|s| s.support().second.is_empty())
    );
    assert_eq!(value.order(), DerivativeOrder::Value);
    assert!(
        value
            .supports()
            .iter()
            .all(|s| s.support().first.is_empty())
    );
}

#[test]
fn domain_projection_is_unassessed_until_bounded_class_evidence_is_requested() {
    use crate::presolve::{ClassEvidence, ClassRequest, ClassStatus};
    let (assembly, values) = fixture(true, false);
    let cancel = Arc::new(AtomicBool::new(false));
    let base = assembly
        .presolve_domain_facts(&values, 10_000, &cancel)
        .unwrap();
    assert_eq!(base.class_status, ClassStatus::Unassessed);
    assert!(base.affine.iter().all(Option::is_none));
    assert_eq!(base.objective_degree, None);
    assert!(matches!(
        assembly.class_evidence(&values, &base, ClassRequest::Coefficients, 1, &cancel),
        Err(crate::MathError::WorkLimit {
            resource: "class proof construction",
            ..
        })
    ));
    assert_eq!(base.class_status, ClassStatus::Unassessed);
    let ClassEvidence::Established {
        facts,
        coefficients,
    } = assembly
        .class_evidence(&values, &base, ClassRequest::Coefficients, 10_000, &cancel)
        .unwrap()
    else {
        panic!("coefficient class should be established");
    };
    assert_eq!(facts.class_status, ClassStatus::Established);
    assert_eq!(facts.objective_degree, Some(2));
    assert!(facts.proof_remaining < base.proof_remaining);
    assert_eq!(coefficients.hessian.val(), &[2.0]);
    let consumed = base.proof_remaining - facts.proof_remaining;
    assert!(matches!(
        assembly.class_evidence(
            &values,
            &base,
            ClassRequest::Coefficients,
            consumed - 1,
            &cancel
        ),
        Err(crate::MathError::WorkLimit {
            resource: "class proof construction",
            ..
        })
    ));
    assert!(matches!(
        assembly.class_evidence(
            &values,
            &base,
            ClassRequest::Coefficients,
            consumed,
            &cancel
        ),
        Ok(ClassEvidence::Established { .. })
    ));
    cancel.store(true, std::sync::atomic::Ordering::Release);
    assert!(matches!(
        assembly.class_evidence(&values, &base, ClassRequest::Coefficients, 10_000, &cancel),
        Err(crate::MathError::Cancelled)
    ));
    cancel.store(false, std::sync::atomic::Ordering::Release);
    assert_eq!(base.class_status, ClassStatus::Unassessed);
}

#[test]
fn class_proof_aggregates_nonlinear_row_contributions_before_ruling_out_coefficients() {
    use crate::presolve::{ClassEvidence, ClassRequest};
    let (assembly, values) = fixture(true, false);
    let original = assembly.structure();
    let mut instances = original.instances().to_vec();
    instances[0].contributions = vec![
        Contribution {
            output: 0,
            target: Target::Row(id(10)),
            scale: 1.0,
        },
        Contribution {
            output: 0,
            target: Target::Row(id(10)),
            scale: -1.0,
        },
        Contribution {
            output: 1,
            target: Target::PRIMARY,
            scale: 1.0,
        },
    ];
    let structure = CaseStructure::new(
        original.variables().to_vec(),
        original.parameters().to_vec(),
        instances,
        original.rows().to_vec(),
        original.objective().cloned(),
        CaseLimits::default(),
    )
    .unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let registry = standard_registry().unwrap();
    let plan = CasePlan::prepare(
        Arc::new(structure),
        assembly.bodies().clone(),
        &registry,
        DerivativeOrder::Value,
        AssemblyLimits::default(),
        &cancel,
    )
    .unwrap();
    let base = plan
        .presolve_domain_facts(&values, 10_000, &cancel)
        .unwrap();
    let ClassEvidence::Established {
        facts,
        coefficients,
    } = plan
        .class_evidence(&values, &base, ClassRequest::Coefficients, 10_000, &cancel)
        .unwrap()
    else {
        panic!("aggregate affine case should be established");
    };
    assert!(facts.affine[0].as_ref().unwrap().entries.is_empty());
    assert_eq!(facts.objective_degree, Some(1));
    assert_eq!(coefficients.objective, [2.0, 0.0]);
    assert!(coefficients.constraints.val().is_empty());
    assert!(coefficients.hessian.val().is_empty());
}

#[test]
fn ruled_out_coefficient_class_retains_independent_affine_rows() {
    use crate::presolve::{ClassEvidence, ClassRequest, ClassWitness};
    let (assembly, values) = fixture(true, false);
    let original = assembly.structure();
    let mut instances = original.instances().to_vec();
    instances[0].contributions = vec![
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
    ];
    let structure = CaseStructure::new(
        original.variables().to_vec(),
        original.parameters().to_vec(),
        instances,
        original.rows().to_vec(),
        original.objective().cloned(),
        CaseLimits::default(),
    )
    .unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let registry = standard_registry().unwrap();
    let plan = CasePlan::prepare(
        Arc::new(structure),
        assembly.bodies().clone(),
        &registry,
        DerivativeOrder::Value,
        AssemblyLimits::default(),
        &cancel,
    )
    .unwrap();
    let base = plan
        .presolve_domain_facts(&values, 10_000, &cancel)
        .unwrap();
    let ClassEvidence::RuledOut { facts, witness } = plan
        .class_evidence(&values, &base, ClassRequest::Coefficients, 10_000, &cancel)
        .unwrap()
    else {
        panic!("nonaffine row must rule out coefficients");
    };
    assert_eq!(witness, ClassWitness::NonAffineRow { row: id(10) });
    assert!(facts.affine[0].is_none());
    assert_eq!(
        facts.affine[1].as_ref().unwrap().entries,
        BTreeMap::from([(0, 2.0)])
    );
    assert_eq!(facts.objective_degree, None);
}

#[test]
fn coefficient_objective_projection_reuses_proved_rows_under_one_work_ledger() {
    let (original, values) = fixture(true, false);
    let registry = standard_registry().unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let quantity = ids::quantity("neutral");
    let project = |count: u8| {
        let mut instances = original.structure().instances().to_vec();
        instances[0].contributions = vec![Contribution {
            output: 0,
            target: Target::PRIMARY,
            scale: 1.,
        }];
        let rows = (0..count)
            .map(|index| {
                let row = id(index + 30);
                instances[0].contributions.push(Contribution {
                    output: 1,
                    target: Target::Row(row),
                    scale: 3.,
                });
                Row {
                    id: row,
                    quantity,
                    lower: 0.,
                    upper: 100.,
                }
            })
            .collect();
        let structure = CaseStructure::new(
            original.structure().variables().to_vec(),
            original.structure().parameters().to_vec(),
            instances,
            rows,
            original.structure().objective().cloned(),
            CaseLimits::default(),
        )
        .unwrap();
        let plan = CasePlan::prepare(
            Arc::new(structure),
            original.bodies().clone(),
            &registry,
            DerivativeOrder::Value,
            AssemblyLimits::default(),
            &cancel,
        )
        .unwrap();
        let facts = plan.presolve_facts(&values, 10_000_000, &cancel).unwrap();
        assert_eq!(
            facts.class_status,
            crate::presolve::ClassStatus::Established
        );
        assert_eq!(facts.objective_degree, Some(2));
        assert!(facts.affine.iter().all(Option::is_some));
        let (coefficients, remaining) = plan
            .coefficient_projection(&values, &facts, facts.proof_remaining, &cancel)
            .unwrap();
        let consumed = facts.proof_remaining - remaining;
        // The exact remaining work suffices; one less still refuses the shared ledger.
        assert!(
            plan.coefficient_projection(&values, &facts, consumed, &cancel)
                .is_ok()
        );
        assert!(matches!(
            plan.coefficient_projection(&values, &facts, consumed - 1, &cancel),
            Err(crate::MathError::WorkLimit {
                resource: "class proof construction",
                ..
            }),
        ));
        assert_eq!(coefficients.constraints.val(), vec![6.; usize::from(count)]);
        assert_eq!(coefficients.row_constants, vec![0.; usize::from(count)]);
        (coefficients, consumed)
    };
    let (small, small_work) = project(1);
    let (large, large_work) = project(200);
    assert_eq!(large.objective, small.objective);
    assert_eq!(large.objective_constant, small.objective_constant);
    assert_eq!(large.hessian.to_dense(), small.hessian.to_dense());
    // Each extra proved row needs one output slot, affine entry and sparse entry.
    // No symbolic row reconstruction or substitution is consumed by this product.
    assert_eq!(large_work - small_work, 3 * 199);
}

#[test]
fn class_proof_and_coefficients_share_aggregate_objective_cancellation() {
    use crate::presolve::{ClassEvidence, ClassRequest};
    let (assembly, values) = fixture(true, false);
    let registry = standard_registry().unwrap();
    let quantity = ids::quantity("neutral");
    let mut builder = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &StandardInvariantChecker,
        2,
        BodyLimits::default(),
    )
    .unwrap();
    let x = builder.input(0, quantity, IndexSet::new(), id(20)).unwrap();
    let y = builder.input(1, quantity, IndexSet::new(), id(21)).unwrap();
    let square = builder
        .binary(Binary::Mul, x.clone(), x.clone(), None, id(22))
        .unwrap();
    let cubic = builder
        .binary(Binary::Mul, square, x.clone(), None, id(23))
        .unwrap();
    let linear = builder.binary(Binary::Add, x, y, None, id(24)).unwrap();
    let body = Arc::new(builder.prepare(&[cubic, linear]).unwrap());
    let original = assembly.structure();
    let mut instances = original.instances().to_vec();
    let key = instances[0].body;
    instances[0].contributions = vec![
        Contribution {
            output: 0,
            target: Target::PRIMARY,
            scale: 1.0,
        },
        Contribution {
            output: 0,
            target: Target::PRIMARY,
            scale: -1.0,
        },
        Contribution {
            output: 1,
            target: Target::PRIMARY,
            scale: 1.0,
        },
    ];
    let structure = CaseStructure::new(
        original.variables().to_vec(),
        original.parameters().to_vec(),
        instances,
        original.rows().to_vec(),
        original.objective().cloned(),
        CaseLimits::default(),
    )
    .unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
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
    let ClassEvidence::Established {
        facts,
        coefficients,
    } = plan
        .class_evidence(&values, &base, ClassRequest::Coefficients, 10_000, &cancel)
        .unwrap()
    else {
        panic!("canceled cubic objective must admit linear coefficients");
    };
    assert_eq!(facts.objective_degree, Some(1));
    assert_eq!(coefficients.objective, [2.0, 0.0]);
    assert!(coefficients.hessian.val().is_empty());
}

#[test]
fn repeated_occurrences_share_serial_scratch_and_keep_bound_caches_under_finite_budget() {
    let (original, mut values) = fixture(true, false);
    let registry = standard_registry().unwrap();
    let quantity = ids::quantity("neutral");
    let cancel = Arc::new(AtomicBool::new(false));
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
    let sum = builder
        .binary(Binary::Add, a.clone(), b, None, id(23))
        .unwrap();
    let square = builder
        .binary(Binary::Mul, sum.clone(), sum, None, id(25))
        .unwrap();
    let aa = builder
        .binary(Binary::Mul, a.clone(), a.clone(), None, id(26))
        .unwrap();
    let cube = builder
        .binary(Binary::Mul, aa, a.clone(), None, id(27))
        .unwrap();
    // Admitted inventory can be much larger than this consumer's selected outputs.
    let mut unrelated = a.clone();
    for _ in 0..512 {
        unrelated = builder
            .binary(Binary::Add, unrelated, a.clone(), None, id(24))
            .unwrap();
    }
    let body = Arc::new(builder.prepare(&[product, square, cube]).unwrap());
    let key = original.structure().instances()[0].body;
    let port = |n| Port {
        id: id(n),
        quantity,
        unit: registry.quantity_type(quantity).unwrap().canonical_unit,
    };
    let instances = (0..64)
        .map(|n| {
            let mut binding = original.structure().instances()[0].clone();
            binding.instance = id(n + 100);
            binding.contributions.push(Contribution {
                output: 2,
                target: Target::Row(id(11)),
                scale: -2.0,
            });
            let source = if n % 2 == 0 { 1 } else { 2 };
            binding.slots = vec![
                SlotBinding::new(&port(source), &port(1), &registry).unwrap(),
                SlotBinding::new(&port(source), &port(2), &registry).unwrap(),
            ];
            binding
        })
        .collect();
    let structure = Arc::new(
        CaseStructure::new(
            original.structure().variables().to_vec(),
            vec![],
            instances,
            original.structure().rows().to_vec(),
            original.structure().objectives().first().cloned(),
            CaseLimits::default(),
        )
        .unwrap(),
    );
    let prepare = |worker_bytes| {
        Arc::new(
            CasePlan::prepare(
                structure.clone(),
                BTreeMap::from([(key, body.clone())]),
                &registry,
                DerivativeOrder::Second,
                AssemblyLimits {
                    worker_bytes,
                    ..AssemblyLimits::default()
                },
                &cancel,
            )
            .unwrap(),
        )
    };
    let assembly = prepare(usize::MAX)
        .compile(
            Optimization::default(),
            EvaluationLimits::default(),
            &cancel,
        )
        .unwrap();
    let bytes = assembly.numeric_worker_bytes();
    // Repeated workers would exceed this admitted extent even before their caches.
    assert!(
        bytes
            < 64 * assembly
                .programs()
                .iter()
                .map(|p| p.worker_bytes())
                .sum::<usize>()
    );
    assert!(matches!(
        prepare(bytes - 1).assemble(assembly.programs().to_vec()),
        Err(crate::MathError::Limit("case worker bytes"))
    ));
    let assembly = Arc::new(
        prepare(bytes)
            .assemble(assembly.programs().to_vec())
            .unwrap(),
    );
    let mut worker = assembly.worker(BTreeMap::new(), cancel.clone());
    for (x, objective, row, gradient) in [
        (2.0, 416.0, 8320.0, 128.0),
        (5.0, 1088.0, 21760.0, 320.0),
        (2.0, 416.0, 8320.0, 128.0),
    ] {
        values.scalars.insert(id(1), x);
        assert_eq!(worker.objective(&values).unwrap(), objective);
        assert_eq!(
            worker.constraints(&values).unwrap(),
            vec![row, -64.0 * (x * x * x + 27.0)]
        );
        assert_eq!(worker.gradient(&values).unwrap(), vec![gradient, 192.0]);
        assert_eq!(
            worker
                .hessian(&values, 1.0, &[0.0, 0.0])
                .unwrap()
                .to_dense()[(0, 0)],
            64.0
        );
        // Distinct signed row weights must contract the occurrence's selected output,
        // even when the derivative cache at this same point is already populated.
        let h = worker
            .hessian(&values, 2.5, &[0.75, -0.25])
            .unwrap()
            .to_dense();
        assert_eq!(h[(0, 0)], 1120.0 + 96.0 * x);
        assert_eq!(h[(1, 1)], 1408.0);
        let h = worker
            .hessian(&values, 0.0, &[-1.0, 2.0])
            .unwrap()
            .to_dense();
        assert_eq!(h[(0, 0)], -1280.0 - 768.0 * x);
        assert_eq!(h[(1, 1)], -3584.0);
        let sources = worker.constraint_sources().unwrap();
        assert_eq!(sources.len(), 128);
        assert_eq!(sources.iter().filter(|s| s.instance == id(100)).count(), 2);
        assert_eq!(
            sources
                .iter()
                .find(|s| s.instance == id(100) && s.output == 1)
                .unwrap()
                .value,
            4.0 * x * x
        );
        assert_eq!(
            sources
                .iter()
                .find(|s| s.instance == id(101) && s.output == 1)
                .unwrap()
                .value,
            36.0
        );
    }
    // Separate attempts have separate scratch and caches even over the same assembly.
    let mut other = assembly.worker(BTreeMap::new(), cancel);
    values.scalars.insert(id(1), 5.0);
    assert_eq!(other.objective(&values).unwrap(), 1088.0);
    values.scalars.insert(id(1), 2.0);
    assert_eq!(worker.objective(&values).unwrap(), 416.0);
}

#[test]
fn value_function_incidence_retains_all_original_free_inputs_and_selected_outputs() {
    let (assembly, _) = fixture(false, false);
    let registry = standard_registry().unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let original = CasePlan::prepare(
        Arc::new(assembly.structure().clone()),
        assembly.bodies().clone(),
        &registry,
        DerivativeOrder::Value,
        AssemblyLimits::default(),
        &cancel,
    )
    .unwrap();
    // id(2) is a free source variable even when it is omitted from numeric columns.
    for coordinates in [vec![], vec![id(1)]] {
        let function = original
            .functions(
                &[id(10)],
                coordinates.clone(),
                &registry,
                DerivativeOrder::Value,
                &cancel,
            )
            .unwrap();
        assert_eq!(function.columns(), coordinates);
        assert_eq!(function.jacobian_pattern().ncols(), coordinates.len());
        assert!(
            function
                .supports()
                .iter()
                .all(|support| support.support().first.is_empty())
        );
        let incidence = function.incidence(&cancel).unwrap();
        assert_eq!(incidence.len(), 1);
        assert_eq!(incidence[0].outputs(), &[1]);
        assert_eq!(incidence[0].coordinates(), &[0, 1]);
        assert_eq!(
            incidence[0].first_for_output(1).unwrap(),
            &[0, 1].into_iter().collect()
        );
        assert!(incidence[0].first_for_output(0).is_none());
        assert_eq!(function.order(), DerivativeOrder::Value);
        assert_eq!(function.columns(), coordinates);
        assert!(
            function
                .supports()
                .iter()
                .all(|support| support.support().first.is_empty())
        );
    }
}

#[test]
fn changed_coordinate_incidence_retains_support_owner_and_unspent_budget() {
    let registry = standard_registry().unwrap();
    let quantity = ids::quantity("neutral");
    let cancel = Arc::new(AtomicBool::new(false));
    let mut builder = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &StandardInvariantChecker,
        2,
        BodyLimits {
            slots: 8,
            occurrences: 128,
        },
    )
    .unwrap();
    let first = builder.input(0, quantity, IndexSet::new(), id(20)).unwrap();
    let second = builder.input(1, quantity, IndexSet::new(), id(21)).unwrap();
    let body = builder.prepare(&[first, second]).unwrap();
    let owner = Arc::new(());
    let weak_owner = Arc::downgrade(&owner);
    let ready = body
        .prepare_support(&[0], &[], DerivativeOrder::Value, &cancel)
        .unwrap()
        .with_owner(owner.clone());
    let original_remaining = ready.remaining_occurrences();
    let incidence = ready.incidence(&[0, 1], &cancel).unwrap();
    let same_coordinates = ready.incidence(&[], &cancel).unwrap();
    assert_eq!(incidence.outputs(), &[0]);
    assert_eq!(
        incidence.first_for_output(0).unwrap(),
        &[0].into_iter().collect()
    );
    assert!(incidence.first_for_output(1).is_none());
    assert!(incidence.remaining_occurrences() < original_remaining);
    assert_eq!(ready.remaining_occurrences(), original_remaining);
    assert!(ready.support().first.is_empty());
    drop(owner);
    drop(ready);
    assert!(weak_owner.upgrade().is_some());
    drop(incidence);
    assert!(weak_owner.upgrade().is_some());
    drop(same_coordinates);
    assert!(weak_owner.upgrade().is_none());

    // Selecting the same output consumes the existing finite map allowance only.
    // Exhaust that product while the original body still has fresh admission capacity.
    let mut exhausted = body
        .prepare_support(&[0], &[], DerivativeOrder::Value, &cancel)
        .unwrap();
    while exhausted.remaining_occurrences() > 0 {
        exhausted = exhausted.select_outputs(&[0], &cancel).unwrap();
    }
    assert!(matches!(
        exhausted.incidence(&[0, 1], &cancel),
        Err(crate::MathError::WorkLimit { .. })
    ));
    assert_eq!(exhausted.remaining_occurrences(), 0);
    assert!(exhausted.support().first.is_empty());
    assert!(body.incidence(&[0], &[0, 1], &cancel).is_ok());
}

#[test]
fn demanded_directional_case_actions_preserve_aliases_rows_cache_and_value_only_programs() {
    let registry = standard_registry().unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    for alias in [false, true] {
        let (reference, values) = fixture(alias, false);
        let value_plan = CasePlan::prepare(
            Arc::new(reference.structure().clone()),
            reference.bodies().clone(),
            &registry,
            DerivativeOrder::Value,
            AssemblyLimits::default(),
            &cancel,
        )
        .unwrap();
        let old = value_plan.demands().to_vec();
        let requested = value_plan.with_directional_actions(&cancel).unwrap();
        assert_eq!(&requested.demands()[..old.len()], old);
        assert_eq!(requested.order(), DerivativeOrder::Value);
        assert!(
            requested.demands()[old.len()..]
                .iter()
                .all(|d| d.directional && d.order == DerivativeOrder::First)
        );
        assert_eq!(
            requested
                .with_directional_actions(&cancel)
                .unwrap()
                .demands(),
            requested.demands()
        );
        let limits = EvaluationLimits {
            derivative_components: 2,
            ..EvaluationLimits::default()
        };
        let action = Arc::new(
            Arc::new(requested)
                .compile(Optimization::default(), limits, &cancel)
                .unwrap(),
        );
        let mut actual = action.worker(BTreeMap::new(), cancel.clone());
        let mut assembled = reference.worker(BTreeMap::new(), cancel.clone());
        for direction in [[2.0, -3.0], [-5.0, 7.0], [0.0, 0.0]] {
            let mut expected = [0.0; 2];
            assembled
                .jacobian_product(&values, &direction, &mut expected)
                .unwrap();
            let mut output = [123.0; 2];
            actual
                .jacobian_product(&values, &direction, &mut output)
                .unwrap();
            assert_eq!(output, expected);
            let mut same = [123.0; 2];
            actual
                .jacobian_product(&values, &direction, &mut same)
                .unwrap();
            assert_eq!(same, expected);
        }
        // The ordinary Value programs cannot evaluate First: the action has not used them.
        assert!(actual.jacobian(&values).is_err());
        let mut output = [123.0; 2];
        let mut missing = values.clone();
        missing.scalars.remove(&id(1));
        assert!(
            actual
                .jacobian_product(&missing, &[2.0, 3.0], &mut output)
                .is_err()
        );
        assert_eq!(output, [123.0; 2]);
        assert!(
            actual
                .jacobian_product(&values, &[f64::NAN, 3.0], &mut output)
                .is_err()
        );
        assert_eq!(output, [123.0; 2]);
        // A stronger assembled demand preserves the separately keyed First-only action.
        let upgraded = action
            .prepare_order(DerivativeOrder::Second, &cancel)
            .unwrap();
        assert_eq!(
            upgraded.demands().iter().filter(|d| d.directional).count(),
            1
        );
        assert!(
            upgraded
                .demands()
                .iter()
                .filter(|d| d.directional)
                .all(|d| d.order == DerivativeOrder::First)
        );
    }
}

#[test]
fn demanded_directional_case_actions_apply_affine_physical_gather_scales_once_per_alias() {
    let registry = standard_registry().unwrap();
    let quantity = ids::quantity("temperature.point");
    let difference = ids::quantity("temperature.difference");
    let canonical = registry.quantity_type(quantity).unwrap().canonical_unit;
    let source = Port {
        id: id(1),
        quantity,
        unit: ids::unit("degF"),
    };
    let formal = Port {
        id: id(2),
        quantity,
        unit: canonical,
    };
    let mut builder = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &StandardInvariantChecker,
        3,
        BodyLimits::default(),
    )
    .unwrap();
    let a = builder.input(0, quantity, IndexSet::new(), id(20)).unwrap();
    let b = builder.input(1, quantity, IndexSet::new(), id(21)).unwrap();
    let c = builder.input(2, quantity, IndexSet::new(), id(22)).unwrap();
    let ac = builder
        .binary(Binary::Sub, a, c.clone(), None, id(23))
        .unwrap();
    let bc = builder.binary(Binary::Sub, b, c, None, id(24)).unwrap();
    let body = Arc::new(builder.prepare(&[ac, bc]).unwrap());
    let key = ContentHash::from_bytes([92; 32]);
    let variable = |port| Variable {
        port,
        fixed: false,
        domain: ModelingVariableDomain::Continuous,
        lower: None,
        upper: None,
    };
    let structure = Arc::new(
        CaseStructure::new(
            vec![variable(source.clone()), variable(formal.clone())],
            vec![],
            vec![InstanceBinding {
                checked_members: Default::default(),
                instance: id(9),
                body: key,
                slots: vec![
                    SlotBinding::new(&source, &formal, &registry).unwrap(),
                    SlotBinding::new(&source, &formal, &registry).unwrap(),
                    SlotBinding::new(&formal, &formal, &registry).unwrap(),
                ],
                contributions: vec![
                    Contribution {
                        output: 0,
                        target: Target::Row(id(10)),
                        scale: 1.0,
                    },
                    Contribution {
                        output: 1,
                        target: Target::Row(id(10)),
                        scale: -3.0,
                    },
                ],
            }],
            vec![Row {
                id: id(10),
                quantity: difference,
                lower: 0.0,
                upper: 0.0,
            }],
            None,
            CaseLimits::default(),
        )
        .unwrap(),
    );
    let cancel = Arc::new(AtomicBool::new(false));
    let values = CaseValues {
        scalars: BTreeMap::from([(id(1), 68.0), (id(2), 300.0)]),
    };
    let plan = CasePlan::prepare(
        structure,
        BTreeMap::from([(key, body)]),
        &registry,
        DerivativeOrder::First,
        AssemblyLimits::default(),
        &cancel,
    )
    .unwrap();
    let reference = Arc::new(
        Arc::new(plan.clone())
            .compile(
                Optimization::default(),
                EvaluationLimits::default(),
                &cancel,
            )
            .unwrap(),
    );
    let direct = Arc::new(
        Arc::new(plan.with_directional_actions(&cancel).unwrap())
            .compile(
                Optimization::default(),
                EvaluationLimits::default(),
                &cancel,
            )
            .unwrap(),
    );
    let mut expected = [0.0];
    reference
        .worker(BTreeMap::new(), cancel.clone())
        .jacobian_product(&values, &[9.0, 4.0], &mut expected)
        .unwrap();
    let mut output = [0.0];
    direct
        .worker(BTreeMap::new(), cancel)
        .jacobian_product(&values, &[9.0, 4.0], &mut output)
        .unwrap();
    assert_eq!(output, expected);
    assert!((output[0] + 2.0).abs() < 1e-12);
}
