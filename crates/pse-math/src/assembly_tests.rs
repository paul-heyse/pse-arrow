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
                slots: slots.into(),
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
    let mut w = a.worker(BTreeMap::new(), pse_diagnostics::Cancellation::default());
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
    let mut w = a.worker(BTreeMap::new(), pse_diagnostics::Cancellation::default());
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
    let demand = c
        .numerical_convexity_allocation_bound(16usize << 30)
        .unwrap();
    assert!(demand < 1 << 20);
    assert_eq!(
        c.numerical_convexity_allocation_bound(1).unwrap(),
        2 * size_of::<f64>() + size_of::<ConvexityEvidence>()
    );
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
    let mut w = a.worker(BTreeMap::new(), pse_diagnostics::Cancellation::default());
    assert!(a.columns().is_empty());
    assert_eq!(w.objective(&x).unwrap(), 4.0);
    assert!(w.gradient(&x).unwrap().is_empty());
    let control = pse_diagnostics::Cancellation::new(false);
    let cancel = control.flag();
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
                    slots: (vec![SlotBinding::new(&port, &port, &registry).unwrap()]).into(),
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
                slots: (vec![
                    SlotBinding::new(&port, &formal, &registry).unwrap(),
                    SlotBinding::new(&parameter, &formal, &registry).unwrap(),
                ])
                .into(),
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
    let control = pse_diagnostics::Cancellation::new(false);
    let cancel = control.flag();
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
    let mut worker = a.worker(BTreeMap::new(), control.clone());
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
    // This checks mathematical demand, not a small symbolic-construction cap.
    let bound = plan.presolve_facts(&values, 10_000, &cancel).unwrap();
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
                slots: (vec![SlotBinding::new(&port, &port, &registry).unwrap()]).into(),
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
    let control = pse_diagnostics::Cancellation::new(false);
    let cancel = control.flag();
    assert!(matches!(
        assembly.presolve_domain_facts(&values, 2, &cancel),
        Err(crate::MathError::Limit("factorable projection extent"))
    ));
    let mut worker = assembly.worker(BTreeMap::new(), control.clone());
    assert_eq!(worker.constraints(&values).unwrap(), vec![20., 0.]);
    let proof_work = 10_000;
    let complete = assembly
        .presolve_facts(&values, proof_work, &cancel)
        .unwrap();
    let consumed = proof_work - complete.proof_remaining;
    assert!(consumed > 0);
    assert!(assembly.presolve_facts(&values, consumed, &cancel).is_ok());
    assert!(matches!(
        assembly.presolve_facts(&values, consumed - 1, &cancel),
        Err(crate::MathError::WorkLimit {
            resource: "class proof construction",
            ..
        })
    ));
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
    control.cancel();
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
                slots: (vec![
                    SlotBinding::new(&port(1), &port(1), &registry).unwrap(),
                    SlotBinding::new(&port(2), &port(2), &registry).unwrap(),
                ])
                .into(),
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
    let control = pse_diagnostics::Cancellation::new(false);
    let cancel = control.flag();
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
    let mut first_worker = first_assembly.worker(BTreeMap::new(), control.clone());
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
    let mut worker = assembly.worker(BTreeMap::new(), control.clone());
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
                slots: (vec![
                    SlotBinding::new(&port(1), &port(1), &registry).unwrap(),
                    SlotBinding::new(&port(2), &port(2), &registry).unwrap(),
                ])
                .into(),
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
    let control = pse_diagnostics::Cancellation::new(false);
    let cancel = control.flag();
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
    let mut worker = assembly.worker(BTreeMap::new(), control.clone());
    assert_eq!(worker.objective(&values).unwrap(), -4.0);
    let proof_work = 10_000;
    let coefficients = linear.coefficients(&values, proof_work, &cancel).unwrap();
    assert_eq!(coefficients.objective, vec![1.0, 1.0]);
    assert_eq!(coefficients.lexicographic, vec![(vec![2.0, -2.0], 0.0)]);
    // A later level must be linear to be a coefficient model.
    let quadratic = plan(structure(2, levels).unwrap());
    let facts = quadratic
        .presolve_facts(&values, proof_work, &cancel)
        .unwrap();
    assert_eq!(facts.lexicographic_degree, None);
    assert!(matches!(
        facts.class_status,
        crate::presolve::ClassStatus::RuledOut(
            crate::presolve::ClassWitness::NonlinearLaterObjective { level: 1 }
        )
    ));
    assert!(!facts.coefficient_eligible());
    assert!(matches!(
        quadratic.coefficients(&values, proof_work, &cancel),
        Err(crate::MathError::Contract(_))
    ));
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
    let reused_rows = assembly
        .affine_row_facts(&values, &facts, 10_000, &cancel)
        .unwrap();
    assert_eq!(reused_rows.key, facts.key);
    assert_eq!(reused_rows.affine, facts.affine);
    assert_eq!(reused_rows.class_status, facts.class_status);
    assert!(reused_rows.proof_remaining <= facts.proof_remaining);
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
fn affine_row_demand_is_independent_of_nonlinear_rows_and_whole_class_admission() {
    use crate::presolve::{ClassStatus, ObligationStatus};
    let (original, values) = fixture(true, false);
    let mut instances = original.structure().instances().to_vec();
    instances[0].contributions.push(Contribution {
        output: 0,
        target: Target::Row(id(11)),
        scale: 1.0,
    });
    let structure = CaseStructure::new(
        original.structure().variables().to_vec(),
        original.structure().parameters().to_vec(),
        instances,
        original.structure().rows().to_vec(),
        original.structure().objective().cloned(),
        CaseLimits::default(),
    )
    .unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let plan = CasePlan::prepare(
        Arc::new(structure),
        original.bodies().clone(),
        &standard_registry().unwrap(),
        DerivativeOrder::Value,
        AssemblyLimits::default(),
        &cancel,
    )
    .unwrap();
    let mut base = plan
        .presolve_domain_facts(&values, 10_000, &cancel)
        .unwrap();
    // Whole-case domain uncertainty remains an obligation; it does not make an
    // independent row's algebra nonlinear or grant the unknown domain admission.
    base.obligations
        .insert(id(9), ObligationStatus::Unestablished);
    let facts = plan
        .affine_row_facts(&values, &base, 10_000, &cancel)
        .unwrap();
    let row = |id| {
        plan.structure()
            .rows()
            .iter()
            .position(|row| row.id == id)
            .unwrap()
    };
    assert_eq!(
        facts.affine[row(id(10))].as_ref().unwrap().entries,
        BTreeMap::from([(0, 10.0)])
    );
    assert!(facts.affine[row(id(11))].is_none());
    assert_eq!(facts.class_status, ClassStatus::Unassessed);
    assert_eq!(facts.objective_degree, None);
    assert_eq!(facts.objective_linear, base.objective_linear);
    assert_eq!(facts.obligations, base.obligations);
    assert_eq!(facts.row_sources, base.row_sources);
    assert_eq!(facts.complete, base.complete);
    assert_eq!(facts.signs, base.signs);
    assert_eq!(facts.has_guards, base.has_guards);
    assert_ne!(facts.key, base.key);
    assert!(facts.proof_remaining < base.proof_remaining);
    assert!(base.affine.iter().all(Option::is_none));
    let mut incomplete = base.clone();
    incomplete.affine.pop();
    assert!(matches!(
        plan.affine_row_facts(&values, &incomplete, 10_000, &cancel),
        Err(crate::MathError::Contract(_))
    ));
    let consumed = base.proof_remaining - facts.proof_remaining;
    assert!(matches!(
        plan.affine_row_facts(&values, &base, consumed - 1, &cancel),
        Err(crate::MathError::WorkLimit { .. })
    ));
    assert!(
        plan.affine_row_facts(&values, &base, consumed, &cancel)
            .is_ok()
    );
    cancel.store(true, std::sync::atomic::Ordering::Release);
    assert!(matches!(
        plan.affine_row_facts(&values, &base, 10_000, &cancel),
        Err(crate::MathError::Cancelled)
    ));
}

#[test]
fn affine_row_demand_refuses_changed_original_assumptions() {
    let (plan, values) = fixture(false, true);
    let cancel = Arc::new(AtomicBool::new(false));
    let base = plan
        .presolve_domain_facts(&values, 10_000, &cancel)
        .unwrap();
    let mut changed = values.clone();
    changed.scalars.insert(id(1), 4.0);
    assert!(matches!(
        plan.affine_row_facts(&changed, &base, 10_000, &cancel),
        Err(crate::MathError::Contract(_))
    ));
    assert!(base.affine.iter().all(Option::is_none));
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
    let control = pse_diagnostics::Cancellation::new(false);
    let cancel = control.flag();
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
            ]
            .into();
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
    let mut worker = assembly.worker(BTreeMap::new(), control.clone());
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
    let mut other = assembly.worker(BTreeMap::new(), control.clone());
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
    let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
    let operation = hub.start("math.directional.control");
    let _entered = operation.context().enter();
    let registry = standard_registry().unwrap();
    let control = pse_diagnostics::Cancellation::new(false);
    let cancel = control.flag();
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
        let mut actual = action.worker(BTreeMap::new(), control.clone());
        let mut assembled = reference.worker(BTreeMap::new(), control.clone());
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
        let counts = actual
            .demand_statistics()
            .find(|row| {
                row.demand == EvaluationDemand::Directional
                    && row.requested == DerivativeOrder::First
            })
            .unwrap()
            .counts;
        assert_eq!(
            (counts.calls, counts.completed, counts.evaluations),
            (6, 6, 3)
        );
        assert_eq!((counts.cache_hits, counts.changed_input_misses), (3, 2));
        assert_eq!(counts.achieved(DerivativeOrder::First), 6);
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
                slots: (vec![
                    SlotBinding::new(&source, &formal, &registry).unwrap(),
                    SlotBinding::new(&source, &formal, &registry).unwrap(),
                    SlotBinding::new(&formal, &formal, &registry).unwrap(),
                ])
                .into(),
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
    let control = pse_diagnostics::Cancellation::new(false);
    let cancel = control.flag();
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
    // Independent physical oracle: -2 * ((F-32)*5/9 + 273.15 - K).
    // Aliases, nonzero gather offsets and contribution scales all precede row proof.
    let domain = plan
        .presolve_domain_facts(&values, 100_000, &cancel)
        .unwrap();
    let facts = plan
        .affine_row_facts(&values, &domain, 100_000, &cancel)
        .unwrap();
    let affine = facts.affine[0].as_ref().unwrap();
    assert!((affine.entries[&0] + 10.0 / 9.0).abs() < 1e-12);
    assert_eq!(affine.entries[&1], 2.0);
    assert!((affine.constant + 2.0 * (273.15 - 32.0 * 5.0 / 9.0)).abs() < 1e-12);
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
        .worker(BTreeMap::new(), control.clone())
        .jacobian_product(&values, &[9.0, 4.0], &mut expected)
        .unwrap();
    let mut output = [0.0];
    direct
        .worker(BTreeMap::new(), control.clone())
        .jacobian_product(&values, &[9.0, 4.0], &mut output)
        .unwrap();
    assert_eq!(output, expected);
    assert!((output[0] + 2.0).abs() < 1e-12);
}

#[test]
fn flat_factorable_construction_counts_source_and_refuses_opaque_expansion_premise() {
    let (assembly, values) = fixture(true, false);
    let limit = (16usize << 30) / 256;
    let mut request = crate::factorable::FactorableRequest::default();
    let demand = assembly
        .factorable_allocation_bound(&request, limit)
        .unwrap()
        .unwrap();
    assert!(demand < 1 << 20);
    let program = assembly
        .factorable_program(&values, &request, limit, &Arc::new(AtomicBool::new(false)))
        .unwrap();
    assert!(program.bytes() <= demand);
    request.envelopes.insert(
        pse_kernels::ProviderKey(ContentHash::from_bytes([9; 32])),
        vec![(0.0, 1.0)],
    );
    assert_eq!(
        assembly
            .factorable_allocation_bound(&request, limit)
            .unwrap(),
        None
    );
}

#[test]
fn support_upgrade_extent_covers_original_sparse_duplicate_contributions() {
    let (assembly, _) = fixture(true, false);
    let cancel = Arc::new(AtomicBool::new(false));
    let value = CasePlan::prepare(
        Arc::new(assembly.structure().clone()),
        assembly.bodies().clone(),
        &standard_registry().unwrap(),
        DerivativeOrder::Value,
        assembly.limits(),
        &cancel,
    )
    .unwrap();
    let evaluation = EvaluationLimits {
        derivative_components: 1_000_000,
        operations: 100_000_000,
        scratch_bytes: 4usize << 30,
        provider_calls: 1_000_000,
    };
    let demand = value
        .support_upgrade_allocation_bound(evaluation, DerivativeOrder::Second)
        .unwrap()
        .unwrap();
    assert!(demand < 4 << 20);
    let stronger = value
        .prepare_order(DerivativeOrder::Second, &cancel)
        .unwrap();
    assert!(stronger.retained_bytes() < demand);
    let directional = value.with_directional_actions(&cancel).unwrap();
    assert!(directional.has_directional_actions());
    assert!(directional.retained_bytes() < demand);
}

#[test]
fn conditional_blocks_share_original_coordinates_and_binding_inputs() {
    use std::collections::BTreeSet;
    let registry = standard_registry().unwrap();
    let quantity = ids::quantity("neutral");
    let port = |n| Port {
        id: id(n),
        quantity,
        unit: registry.quantity_type(quantity).unwrap().canonical_unit,
    };
    let control = pse_diagnostics::Cancellation::new(false);
    let cancel = control.flag();
    for count in [4u8, 24] {
        let mut builder = BodyBuilder::new(
            crate::initialize().unwrap(),
            &registry,
            &StandardInvariantChecker,
            3,
            BodyLimits::default(),
        )
        .unwrap();
        let x = builder
            .input(0, quantity, IndexSet::new(), id(240))
            .unwrap();
        let p = builder
            .input(1, quantity, IndexSet::new(), id(241))
            .unwrap();
        let _inactive = builder
            .input(2, quantity, IndexSet::new(), id(242))
            .unwrap();
        let sum = builder.binary(Binary::Add, x, p, None, id(243)).unwrap();
        let body = Arc::new(builder.prepare(&[sum]).unwrap());
        let key = ContentHash::from_bytes([count; 32]);
        let mut variables: Vec<_> = (1..=count)
            .map(|n| Variable {
                port: port(n),
                fixed: false,
                domain: ModelingVariableDomain::Continuous,
                lower: None,
                upper: None,
            })
            .collect();
        variables.push(Variable {
            port: port(250),
            fixed: true,
            domain: ModelingVariableDomain::Integer,
            lower: Some(0.0),
            upper: Some(10.0),
        });
        let bindings = (1..=count)
            .map(|n| InstanceBinding {
                instance: id(n + 100),
                body: key,
                checked_members: Default::default(),
                slots: vec![
                    SlotBinding::new(&port(n), &port(n), &registry).unwrap(),
                    SlotBinding::new(&port(n + 50), &port(n + 50), &registry).unwrap(),
                    SlotBinding::new(&port(250), &port(250), &registry).unwrap(),
                ]
                .into(),
                contributions: vec![
                    Contribution {
                        output: 0,
                        target: Target::Row(id(n + 150)),
                        scale: 2.0,
                    },
                    Contribution {
                        output: 0,
                        target: Target::Row(id(n + 150)),
                        scale: -1.0,
                    },
                ],
            })
            .collect();
        let structure = Arc::new(
            CaseStructure::new(
                variables,
                (1..=count).map(|n| port(n + 50)).collect(),
                bindings,
                (1..=count)
                    .map(|n| Row {
                        id: id(n + 150),
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
        let original = CasePlan::prepare(
            structure,
            BTreeMap::from([(key, body)]),
            &registry,
            DerivativeOrder::First,
            AssemblyLimits::default(),
            &cancel,
        )
        .unwrap();
        let blocks: Vec<_> = (1..=count)
            .map(|n| {
                original
                    .conditional(
                        &BTreeSet::from([id(n + 150)]),
                        &BTreeSet::from([id(n)]),
                        &registry,
                        &cancel,
                    )
                    .unwrap()
            })
            .collect();
        let universe = original.structure().coordinate_allocation();
        let mut coordinate_ids = BTreeSet::new();
        let mut selected_bytes = 0;
        let mut values = CaseValues {
            scalars: (1..=count)
                .flat_map(|n| [(id(n), f64::from(n)), (id(n + 50), 0.5)])
                .chain([(id(250), 3.0)])
                .collect(),
        };
        for (offset, block) in blocks.iter().enumerate() {
            let n = offset as u8 + 1;
            assert_eq!(block.structure().coordinate_allocation(), universe);
            coordinate_ids.insert(universe.0);
            selected_bytes += block.structure().selection_bytes();
            assert_eq!(block.columns(), &[id(n)]);
            assert_eq!(block.structure().variables().len(), usize::from(count) + 1);
            assert_eq!(block.structure().parameters().len(), usize::from(count));
            assert_eq!(
                block.structure().free_variables().collect::<Vec<_>>(),
                [id(n)]
            );
            assert!(Arc::ptr_eq(
                &block.structure().instances()[0].slots,
                &original.structure().instances()[offset].slots
            ));
            assert!(Arc::ptr_eq(
                &block.structure().instances()[0].checked_members,
                &original.structure().instances()[offset].checked_members
            ));
            assert_eq!(block.structure().instances()[0].contributions.len(), 2);
            block.structure().validate_values(&values).unwrap();
            let mut missing = values.clone();
            missing.scalars.remove(&id(if n == 1 { 2 } else { 1 }));
            assert!(block.structure().validate_values(&missing).is_err());
            missing = values.clone();
            missing.scalars.remove(&id(50 + if n == 1 { 2 } else { 1 }));
            assert!(block.structure().validate_values(&missing).is_err());
            missing = values.clone();
            missing.scalars.insert(id(250), 3.5);
            assert!(block.structure().validate_values(&missing).is_err());
            let explicit = CaseStructure::new(
                block.structure().variables().to_vec(),
                block.structure().parameters().to_vec(),
                block.structure().instances().to_vec(),
                block.structure().rows().to_vec(),
                None,
                CaseLimits::default(),
            )
            .unwrap();
            assert_eq!(block.structure().key(), explicit.key());
            assert_eq!(block.structure(), &explicit);
        }
        assert_eq!(coordinate_ids.len(), 1);
        // Index storage grows with selected coordinates, not original inventory times blocks.
        assert_eq!(selected_bytes, usize::from(count) * size_of::<usize>());
        let total = CasePlan::retained_group_bytes(blocks.iter()).unwrap();
        let independent: usize = blocks.iter().map(CasePlan::retained_bytes).sum();
        assert!(total <= independent - (usize::from(count) - 1) * universe.1);
        let assembly = Arc::new(
            Arc::new(blocks[0].clone())
                .compile(
                    Optimization::default(),
                    EvaluationLimits::default(),
                    &cancel,
                )
                .unwrap(),
        );
        let mut first = assembly.worker(BTreeMap::new(), control.clone());
        let mut second = assembly.worker(BTreeMap::new(), control.clone());
        assert_eq!(first.constraints(&values).unwrap(), [1.5]);
        values.scalars.insert(id(1), 8.0);
        assert_eq!(second.constraints(&values).unwrap(), [8.5]);
        assert_eq!(first.jacobian(&values).unwrap().val(), &[1.0]);
        let slots = Arc::downgrade(&original.structure().instances()[0].slots);
        control.cancel();
        assert!(matches!(
            original.conditional(
                &BTreeSet::from([id(151)]),
                &BTreeSet::from([id(1)]),
                &registry,
                &cancel
            ),
            Err(crate::MathError::Cancelled)
        ));
        cancel.store(false, std::sync::atomic::Ordering::Relaxed);
        drop(first);
        drop(second);
        drop(assembly);
        drop(original);
        assert!(slots.upgrade().is_some());
        drop(blocks);
        assert!(slots.upgrade().is_none());
    }
}

#[test]
fn dense_trials_preserve_aliases_axes_fixed_rebinding_and_original_assessment() {
    for alias in [false, true] {
        for fixed in [false, true] {
            let (assembly, values) = fixture(alias, fixed);
            let bound = assembly.bind_values(&values).unwrap();
            let coordinates: Vec<_> = assembly
                .columns()
                .iter()
                .map(|id| values.scalars[id])
                .collect();
            let trial = bound.trial(&coordinates).unwrap();
            let control = pse_diagnostics::Cancellation::new(false);
            let mut original = assembly.worker(BTreeMap::new(), control.clone());
            let mut dense = assembly.worker(BTreeMap::new(), control.clone());
            assert_eq!(
                dense.objective(trial).unwrap().to_bits(),
                original.objective(&values).unwrap().to_bits()
            );
            assert_eq!(
                dense.constraints(trial).unwrap(),
                original.constraints(&values).unwrap()
            );
            assert_eq!(
                dense.gradient(trial).unwrap(),
                original.gradient(&values).unwrap()
            );
            assert_eq!(
                dense.jacobian(trial).unwrap().val(),
                original.jacobian(&values).unwrap().val()
            );
            assert_eq!(
                dense.hessian(trial, 2.0, &[3.0, 5.0]).unwrap().val(),
                original.hessian(&values, 2.0, &[3.0, 5.0]).unwrap().val()
            );
            let mut changed = values.clone();
            changed.scalars.insert(id(1), -0.0);
            let rebound = assembly.bind_values(&changed).unwrap();
            let coordinates: Vec<_> = assembly
                .columns()
                .iter()
                .map(|id| changed.scalars[id])
                .collect();
            assert_eq!(
                dense
                    .objective(rebound.trial(&coordinates).unwrap())
                    .unwrap()
                    .to_bits(),
                original.objective(&changed).unwrap().to_bits()
            );
            assert!(bound.trial(&[f64::NAN]).is_err());
            assert!(
                bound
                    .trial(&vec![0.0; assembly.columns().len() + 1])
                    .is_err()
            );
            changed.scalars.remove(&id(1));
            assert!(assembly.bind_values(&changed).is_err());
        }
    }
}

#[test]
fn dense_trials_refuse_other_layout_and_failed_values_clear_observations() {
    let (assembly, values) = fixture(false, false);
    let (other, _) = fixture(false, false);
    let bound = other.bind_values(&values).unwrap();
    let mut worker = assembly.worker(BTreeMap::new(), pse_diagnostics::Cancellation::default());
    assert!(worker.objective(bound.trial(&[2.0, 3.0]).unwrap()).is_err());
    worker.objective(&values).unwrap();
    let mut missing = values;
    missing.scalars.remove(&id(1));
    assert!(worker.objective(&missing).is_err());
    assert!(worker.constraint_sources().is_err());
    assert!(worker.applicability_observations().is_empty());
}

#[test]
fn compiled_gathers_refuse_duplicate_and_unknown_formals_before_evaluation() {
    let (assembly, _) = fixture(true, false);
    let binding = &assembly.structure().instances()[0];
    let columns = BTreeMap::from([(id(1), 0), (id(2), 1)]);
    assert!(EvalBindings::new(binding, &[0, 0], &columns, &BTreeMap::new()).is_err());
    assert!(EvalBindings::new(binding, &[2], &columns, &BTreeMap::new()).is_err());
    assert!(EvalBindings::new(binding, &[0], &BTreeMap::new(), &BTreeMap::new()).is_err());
}

#[test]
fn assembly_first_order_callbacks_leave_second_order_workers_cold() {
    let (assembly, values) = fixture(false, false);
    let mut worker = assembly.worker(BTreeMap::new(), pse_diagnostics::Cancellation::default());
    assert!(worker.instantiated_orders().iter().all(Vec::is_empty));
    assert_eq!(worker.objective(&values).unwrap(), 6.0);
    assert_eq!(worker.gradient(&values).unwrap(), vec![3.0, 2.0]);
    assert_eq!(worker.constraints(&values).unwrap(), vec![25.0, 0.0]);
    assert_eq!(worker.jacobian(&values).unwrap().to_dense()[(0, 0)], 5.0);
    let orders = worker.instantiated_orders();
    assert_eq!(orders.iter().filter(|orders| !orders.is_empty()).count(), 2);
    assert!(
        orders
            .iter()
            .flatten()
            .all(|order| *order <= DerivativeOrder::First)
    );
    assert_eq!(
        worker
            .hessian(&values, 1.0, &[0.0, 0.0])
            .unwrap()
            .to_dense()[(1, 0)],
        1.0
    );
    let orders = worker.instantiated_orders();
    assert_eq!(
        orders
            .iter()
            .filter(|orders| orders.contains(&DerivativeOrder::Second))
            .count(),
        1
    );
    assert_eq!(orders.iter().filter(|orders| !orders.is_empty()).count(), 3);
    let mut independent = assembly.worker(BTreeMap::new(), pse_diagnostics::Cancellation::default());
    assert!(independent.instantiated_orders().iter().all(Vec::is_empty));
    assert_eq!(independent.gradient(&values).unwrap(), vec![3.0, 2.0]);
    assert!(
        independent
            .instantiated_orders()
            .iter()
            .flatten()
            .all(|order| *order == DerivativeOrder::First)
    );
}

#[test]
fn operation_demands_use_consumed_ceilings_and_upgrade_all_products() {
    let (assembly, _) = fixture(false, false);
    let demands = assembly.demands();
    assert_eq!(
        demands
            .iter()
            .filter(|d| d.order == DerivativeOrder::First)
            .count(),
        2
    );
    assert_eq!(
        demands
            .iter()
            .filter(|d| d.order == DerivativeOrder::Second)
            .count(),
        1
    );
    for (demand, support) in demands.iter().zip(assembly.supports()) {
        assert_eq!(demand.order, support.order());
    }
    let registry = standard_registry().unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let value = CasePlan::prepare(
        Arc::new(assembly.structure().clone()),
        assembly.bodies().clone(),
        &registry,
        DerivativeOrder::Value,
        AssemblyLimits::default(),
        &cancel,
    )
    .unwrap();
    let upgraded = value
        .prepare_order(DerivativeOrder::Second, &cancel)
        .unwrap();
    assert_eq!(upgraded.demands(), assembly.demands());
    for (demand, support) in upgraded.demands().iter().zip(upgraded.supports()) {
        assert_eq!(demand.order, support.order());
    }
    // Selecting only constraints makes Constraints and All the same actual demand.
    // Their single product must retain the maximum Second ceiling.
    let rows = assembly.structure().rows().to_vec();
    let mut bindings = assembly.structure().instances().to_vec();
    for binding in &mut bindings {
        binding
            .contributions
            .retain(|c| matches!(c.target, Target::Row(_)));
    }
    let structure = CaseStructure::new(
        assembly.structure().variables().to_vec(),
        vec![],
        bindings,
        rows,
        None,
        CaseLimits::default(),
    )
    .unwrap();
    let only_constraints = CasePlan::prepare(
        Arc::new(structure),
        assembly.bodies().clone(),
        &registry,
        DerivativeOrder::Second,
        AssemblyLimits::default(),
        &cancel,
    )
    .unwrap();
    assert_eq!(only_constraints.demands().len(), 1);
    assert_eq!(only_constraints.demands()[0].order, DerivativeOrder::Second);
}

#[test]
fn assembly_reuses_exact_compact_artifacts_after_original_source_expires() {
    let control = pse_diagnostics::Cancellation::new(false);
    let cancel = control.flag();
    let registry = standard_registry().unwrap();
    let (original, _) = fixture(false, false);
    let programs = original.programs().to_vec();
    let (current, mut values) = fixture(false, false);
    let key = current.demands()[0].body;
    assert_ne!(
        original.bodies()[&key].allocation_identity(),
        current.bodies()[&key].allocation_identity()
    );
    assert!(programs.iter().all(|program| {
        program
            .prepared_support()
            .matches_body(current.bodies()[&key].as_ref())
    }));
    drop(original);
    assert!(programs.iter().all(|program| {
        !program
            .prepared_support()
            .matches_body(current.bodies()[&key].as_ref())
    }));
    for (program, expected) in programs.iter().zip(current.supports()) {
        assert_eq!(program.prepared_support(), expected.as_ref());
    }
    let current_plan = Arc::new(CasePlan::clone(&current));
    let reassembled = Arc::new(current_plan.assemble(programs.clone()).unwrap());
    let mut worker = reassembled.worker(BTreeMap::new(), control.clone());
    assert_eq!(worker.objective(&values).unwrap(), 6.0);
    assert_eq!(worker.gradient(&values).unwrap(), [3.0, 2.0]);
    assert_eq!(worker.constraints(&values).unwrap(), [25.0, 0.0]);
    assert_eq!(
        worker
            .hessian(&values, 1.0, &[0.0, 0.0])
            .unwrap()
            .to_dense()[(1, 0)],
        1.0
    );
    values.scalars.insert(id(1), 4.0);
    assert_eq!(worker.objective(&values).unwrap(), 12.0);
    assert_eq!(worker.gradient(&values).unwrap(), [3.0, 4.0]);
    assert_eq!(worker.constraints(&values).unwrap(), [35.0, 0.0]);

    // Equal demand maps and derivative incidence are insufficient: an altered
    // selected value program must refuse even under the same supplied body key.
    let quantity = ids::quantity("neutral");
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
    let changed_product = builder
        .binary(Binary::Add, product, a, None, id(24))
        .unwrap();
    let changed = Arc::new(
        CasePlan::prepare(
            Arc::new(current.structure().clone()),
            BTreeMap::from([(
                key,
                Arc::new(builder.prepare(&[changed_product, sum]).unwrap()),
            )]),
            &registry,
            DerivativeOrder::Second,
            AssemblyLimits::default(),
            &cancel,
        )
        .unwrap(),
    );
    assert_eq!(changed.demands(), current.demands());
    for (old, changed) in current.supports().iter().zip(changed.supports()) {
        assert_eq!(old.support(), changed.support());
    }
    assert!(matches!(
        changed.assemble(programs),
        Err(crate::MathError::Contract(detail))
            if detail == "artifact does not match selected admitted demand"
    ));
}

#[test]
fn compact_refill_is_atomic_and_hashes_only_consumed_inputs() {
    for alias in [false, true] {
        let (assembly, values) = fixture(alias, true);
        let reads = std::cell::Cell::new(0usize);
        let mut bound = assembly
            .bind_inputs(|id| {
                reads.set(reads.get() + 1);
                values.scalars.get(&id).copied()
            })
            .unwrap();
        assert_eq!(reads.get(), if alias { 1 } else { 2 });
        let mut worker = assembly.worker(BTreeMap::new(), pse_diagnostics::Cancellation::default());
        let before = bound.input_identity();
        let expected = worker.objective(bound.trial(&[]).unwrap()).unwrap();
        let source = bound
            .input_ids()
            .iter()
            .map(|id| values.scalars[id])
            .collect::<Vec<_>>();
        let map = (0..source.len()).rev().collect::<Vec<_>>();
        let reversed = source.iter().rev().copied().collect::<Vec<_>>();
        bound.refill_mapped(&reversed, &map).unwrap();
        assert_eq!(bound.input_identity(), before);
        assert!(
            bound
                .refill_mapped(&source, &vec![usize::MAX; map.len()])
                .is_err()
        );
        assert!(bound.refill_mapped(&source, &[]).is_err());
        assert!(
            bound
                .refill_mapped(&vec![f64::NAN; source.len()], &map)
                .is_err()
        );
        assert_eq!(bound.input_identity(), before);
        assert!(bound.refill(&[(id(1), 7.0), (id(1), 8.0)]).is_err());
        assert!(bound.refill(&[(id(1), 7.0), (id(250), 8.0)]).is_err());
        assert!(bound.refill(&[(id(1), f64::NAN)]).is_err());
        assert_eq!(bound.input_identity(), before);
        assert_eq!(
            worker.objective(bound.trial(&[]).unwrap()).unwrap(),
            expected
        );
        if alias {
            bound.refill(&[(id(2), 99.0)]).unwrap();
            assert_eq!(bound.input_identity(), before);
        }
        bound.refill(&[(id(1), 7.0)]).unwrap();
        assert_ne!(bound.input_identity(), before);
        let mut original = values.clone();
        original.scalars.insert(id(1), 7.0);
        assert_eq!(
            worker.objective(bound.trial(&[]).unwrap()).unwrap(),
            worker.objective(&original).unwrap()
        );
        // A separate operation owner keeps its own compact frame.
        let other = assembly
            .bind_inputs(|id| values.scalars.get(&id).copied())
            .unwrap();
        assert_eq!(other.input_identity(), before);
    }
}

#[test]
fn exclusive_case_scratch_is_one_shot_exact_and_clears_trial_caches() {
    crate::initialize().unwrap();
    let (assembly, mut values) = fixture(false, false);
    let stop_control = pse_diagnostics::Cancellation::default();
    let stop = stop_control.flag();
    let old_stop = Arc::downgrade(&stop);
    #[derive(Debug)]
    struct DepartingProvider {
        spec: pse_kernels::ProviderSpec,
        dropped: Arc<AtomicBool>,
    }
    impl Drop for DepartingProvider {
        fn drop(&mut self) {
            self.dropped
                .store(true, std::sync::atomic::Ordering::Release);
        }
    }
    impl pse_kernels::Provider for DepartingProvider {
        fn spec(&self) -> &pse_kernels::ProviderSpec {
            &self.spec
        }
        fn evaluate(
            &mut self,
            _: &[f64],
            _: &pse_kernels::ProviderRequest,
            _: &pse_kernels::EvaluationContext<'_>,
        ) -> Result<pse_kernels::ProviderValues, pse_kernels::ProviderError> {
            Err(pse_kernels::ProviderError::Trial(
                "unused provider in arithmetic assembly".into(),
            ))
        }
    }
    let dropped = Arc::new(AtomicBool::new(false));
    let registry = standard_registry().unwrap();
    let quantity = ids::quantity("neutral");
    let port = |n| Port {
        id: id(n),
        quantity,
        unit: registry.quantity_type(quantity).unwrap().canonical_unit,
    };
    let spec = pse_kernels::ProviderSpec {
        id: id(90),
        revision: ContentHash::from_bytes([91; 32]),
        data: ContentHash::from_bytes([92; 32]),
        shapes: Default::default(),
        derivative_source: pse_kernels::DerivativeSource::Analytic,
        inputs: vec![port(93)],
        outputs: vec![port(94)],
        derivatives: DerivativeOrder::Value,
        smoothness: DerivativeOrder::Value,
    };
    let provider: Box<dyn pse_kernels::Provider> = Box::new(DepartingProvider {
        spec: spec.clone(),
        dropped: dropped.clone(),
    });
    let returned = Arc::new(std::sync::Mutex::new(None));
    let output = returned.clone();
    let mut worker = assembly.worker(BTreeMap::from([(spec.key(), provider)]), stop_control.clone());
    worker.recycle_on_drop(move |scratch| {
        assert!(
            dropped.load(std::sync::atomic::Ordering::Acquire),
            "provider destruction precedes detachment"
        );
        *output.lock().unwrap() = Some(scratch);
    });
    assert_eq!(worker.objective(&values).unwrap(), 6.0);
    worker.jacobian(&values).unwrap();
    worker.hessian(&values, 1., &[0., 0.]).unwrap();
    let warmed = worker.instantiated_orders();
    values.scalars.insert(id(1), f64::NAN);
    assert!(worker.objective(&values).is_err());
    drop(worker);
    drop(stop);
    drop(stop_control);
    assert!(old_stop.upgrade().is_none());
    let raw = returned.lock().unwrap().take().unwrap();
    assert!(raw.compatible(&assembly));
    let (other, _) = fixture(false, false);
    assert!(
        !raw.compatible(&other),
        "structural equality is insufficient"
    );
    let scratch = raw.reset().unwrap();
    let mut next = assembly
        .worker_from_scratch(
            scratch,
            BTreeMap::new(),
            pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
        )
        .unwrap();
    assert_eq!(next.instantiated_orders(), warmed);
    assert_eq!(
        next.demand_observation_status(),
        DemandObservationStatus::NotRequested
    );
    assert_eq!(next.demand_statistics().count(), 0);
    values.scalars.insert(id(1), 4.0);
    assert_eq!(next.objective(&values).unwrap(), 12.0);
    assert_eq!(next.constraints(&values).unwrap(), vec![35.0, 0.0]);
    let reference = assembly.worker(BTreeMap::new(), pse_diagnostics::Cancellation::default());
    assert!(reference.instantiated_orders().iter().all(Vec::is_empty));
    assert!(returned.lock().unwrap().is_none());
}

#[test]
fn affine_original_rows_cancel_across_bodies_before_proof_and_reuse_bound_outputs() {
    use crate::presolve::ClassStatus;
    let (original, mut values) = fixture(true, false);
    let registry = standard_registry().unwrap();
    let quantity = ids::quantity("neutral");
    let mut builder = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &StandardInvariantChecker,
        3,
        BodyLimits::default(),
    )
    .unwrap();
    let a = builder.input(0, quantity, IndexSet::new(), id(30)).unwrap();
    let b = builder.input(1, quantity, IndexSet::new(), id(31)).unwrap();
    let c = builder.input(2, quantity, IndexSet::new(), id(32)).unwrap();
    let square = builder
        .binary(Binary::Mul, a.clone(), b, None, id(33))
        .unwrap();
    let linear = builder.binary(Binary::Add, a, c, None, id(34)).unwrap();
    let residual = builder
        .binary(Binary::Add, square, linear, None, id(35))
        .unwrap();
    let second = Arc::new(builder.prepare(&[residual]).unwrap());
    let second_key = ContentHash::from_bytes([2; 32]);
    let mut variables = original.structure().variables().to_vec();
    let mut parameter = variables
        .iter()
        .find(|v| v.port.id == id(1))
        .unwrap()
        .clone();
    parameter.port.id = id(3);
    parameter.fixed = true;
    variables.push(parameter.clone());
    let mut first = original.structure().instances()[0].clone();
    first.contributions = vec![
        Contribution {
            output: 0,
            target: Target::Row(id(10)),
            scale: -0.5
        };
        2
    ];
    let x = variables
        .iter()
        .find(|v| v.port.id == id(1))
        .unwrap()
        .port
        .clone();
    let slots = vec![
        SlotBinding::new(&x, &x, &registry).unwrap(),
        SlotBinding::new(&x, &x, &registry).unwrap(),
        SlotBinding::new(&parameter.port, &parameter.port, &registry).unwrap(),
    ];
    let second_binding = InstanceBinding {
        checked_members: Default::default(),
        instance: id(8),
        body: second_key,
        slots: slots.into(),
        contributions: vec![Contribution {
            output: 0,
            target: Target::Row(id(10)),
            scale: 1.0,
        }],
    };
    let structure = CaseStructure::new(
        variables,
        vec![],
        vec![first, second_binding],
        original.structure().rows().to_vec(),
        None,
        CaseLimits::default(),
    )
    .unwrap();
    let mut bodies = original.bodies().clone();
    bodies.insert(second_key, second);
    values.scalars.insert(id(3), 3.0);
    let control = pse_diagnostics::Cancellation::new(false);
    let cancel = control.flag();
    let plan = Arc::new(
        CasePlan::prepare(
            Arc::new(structure),
            bodies,
            &registry,
            DerivativeOrder::Value,
            AssemblyLimits::default(),
            &cancel,
        )
        .unwrap(),
    );
    let base = plan
        .presolve_domain_facts(&values, 100_000, &cancel)
        .unwrap();
    let facts = plan
        .affine_row_facts(&values, &base, 100_000, &cancel)
        .unwrap();
    let row_index = plan
        .structure()
        .rows()
        .iter()
        .position(|row| row.id == id(10))
        .unwrap();
    let row = facts.affine[row_index].as_ref().unwrap();
    assert_eq!(row.entries, BTreeMap::from([(0, 1.0)]));
    assert_eq!(row.constant, 3.0);
    assert_eq!(facts.class_status, ClassStatus::Unassessed);
    assert_eq!(facts.obligations, base.obligations);
    assert_eq!(facts.row_sources, base.row_sources);
    let assembly = Arc::new(
        plan.compile(
            Optimization::default(),
            EvaluationLimits::default(),
            &cancel,
        )
        .unwrap(),
    );
    let mut worker = assembly.worker(BTreeMap::new(), control.clone());
    for x in [-2.0, 0.0, 5.0] {
        values.scalars.insert(id(1), x);
        assert!(facts.matches(&assembly, &values));
        assert_eq!(worker.constraints(&values).unwrap()[row_index], x + 3.0);
    }
    values.scalars.insert(id(3), 4.0);
    assert!(matches!(
        assembly.affine_row_facts(&values, &facts, 100_000, &cancel),
        Err(crate::MathError::Contract(_))
    ));
}

#[test]
fn affine_selected_row_work_is_local_when_an_independent_block_is_added() {
    use pse_diagnostics::observation::{ObservationHub, Policy, Value};
    use std::time::Duration;
    let (original, original_values) = fixture(false, false);
    let registry = standard_registry().unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let run = |extra: bool| {
        let mut variables = original.structure().variables().to_vec();
        let mut instances = original.structure().instances().to_vec();
        let mut rows = original.structure().rows().to_vec();
        let mut values = original_values.clone();
        if extra {
            let mut ports = Vec::new();
            for n in [3, 4] {
                let mut variable = variables[0].clone();
                variable.port.id = id(n);
                ports.push(variable.port.clone());
                variables.push(variable);
                values.scalars.insert(id(n), f64::from(n));
            }
            let mut unrelated = instances[0].clone();
            unrelated.instance = id(8);
            unrelated.slots = vec![
                SlotBinding::new(&ports[0], &ports[0], &registry).unwrap(),
                SlotBinding::new(&ports[1], &ports[1], &registry).unwrap(),
            ]
            .into();
            unrelated.contributions = vec![Contribution {
                output: 1,
                target: Target::Row(id(12)),
                scale: 1.0,
            }];
            instances.push(unrelated);
            let mut row = rows[0].clone();
            row.id = id(12);
            rows.push(row);
        }
        let structure = CaseStructure::new(
            variables,
            vec![],
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
        let domain = plan
            .presolve_domain_facts(&values, 100_000, &cancel)
            .unwrap();
        let mut base = plan
            .affine_row_facts(&values, &domain, 100_000, &cancel)
            .unwrap();
        let selected = plan
            .structure()
            .rows()
            .iter()
            .position(|row| row.id == id(10))
            .unwrap();
        base.affine[selected] = None;
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("selected-row-locality");
        let entered = operation.context().enter();
        let facts = plan
            .affine_row_facts(&values, &base, 100_000, &cancel)
            .unwrap();
        drop(entered);
        let snapshot = operation.reader().snapshot(Duration::from_secs(60));
        let counts = snapshot
            .metrics
            .iter()
            .map(|metric| {
                let Value::Count(count) = metric.cumulative else {
                    panic!("expected actual work count");
                };
                (metric.name.clone(), count)
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            facts.affine[selected].as_ref().unwrap().entries,
            BTreeMap::from([(0, 5.0), (1, 5.0)])
        );
        assert_eq!(counts["affine.selected_rows"], 1);
        assert_eq!(counts["affine.contributions_examined"], 2);
        assert_eq!(counts["affine.bound_output_reuses"], 1);
        assert_eq!(counts["affine.candidate_coordinates"], 2);
        assert_eq!(counts["affine.derivative_attempts"], 2);
        assert_eq!(counts["affine.formal_coordinates"], 2);
        assert_eq!(counts["affine.nonzero_coefficients"], 2);
        assert_eq!(counts["affine.established_rows"], 1);
        assert_eq!(counts["affine.withheld_rows"], 0);
        (counts, facts)
    };
    let (first, _) = run(false);
    let (extra, _) = run(true);
    for name in [
        "contributions_examined",
        "bound_output_reuses",
        "candidate_coordinates",
        "derivative_attempts",
        "formal_coordinates",
        "substitution_calls",
        "nonzero_coefficients",
    ] {
        assert_eq!(
            first[&format!("affine.{name}")],
            extra[&format!("affine.{name}")],
            "{name}"
        );
    }
}

#[test]
fn affine_row_coefficient_range_withholds_only_the_affected_original_row() {
    let (original, values) = fixture(false, false);
    let mut instances = original.structure().instances().to_vec();
    for contribution in &mut instances[0].contributions {
        if matches!(contribution.target, Target::Row(_)) {
            contribution.scale = f64::MAX;
        }
    }
    let structure = CaseStructure::new(
        original.structure().variables().to_vec(),
        vec![],
        instances,
        original.structure().rows().to_vec(),
        original.structure().objective().cloned(),
        CaseLimits::default(),
    )
    .unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let plan = CasePlan::prepare(
        Arc::new(structure),
        original.bodies().clone(),
        &standard_registry().unwrap(),
        DerivativeOrder::Value,
        AssemblyLimits::default(),
        &cancel,
    )
    .unwrap();
    let base = plan
        .presolve_domain_facts(&values, 100_000, &cancel)
        .unwrap();
    let facts = plan
        .affine_row_facts(&values, &base, 100_000, &cancel)
        .unwrap();
    let affected = plan
        .structure()
        .rows()
        .iter()
        .position(|row| row.id == id(10))
        .unwrap();
    let empty = plan
        .structure()
        .rows()
        .iter()
        .position(|row| row.id == id(11))
        .unwrap();
    assert!(facts.affine[affected].is_none());
    assert_eq!(facts.affine[empty].as_ref().unwrap().constant, 0.0);
    assert!(facts.affine[empty].as_ref().unwrap().entries.is_empty());
    assert_eq!(facts.class_status, base.class_status);
    assert_eq!(facts.obligations, base.obligations);
    assert_eq!(facts.row_sources, base.row_sources);
    assert!(base.affine.iter().all(Option::is_none));
}

#[test]
fn affine_plan_coordinates_can_select_fixed_variables_and_parameters_in_explicit_order() {
    let (original, mut values) = fixture(false, true);
    let registry = standard_registry().unwrap();
    let fixed = original
        .structure()
        .variables()
        .iter()
        .find(|v| v.port.id == id(1))
        .unwrap()
        .port
        .clone();
    let mut parameter = fixed.clone();
    parameter.id = id(3);
    let mut instances = original.structure().instances().to_vec();
    instances[0].slots = vec![
        SlotBinding::new(&fixed, &fixed, &registry).unwrap(),
        SlotBinding::new(&parameter, &parameter, &registry).unwrap(),
    ]
    .into();
    let structure = Arc::new(
        CaseStructure::new(
            original.structure().variables().to_vec(),
            vec![parameter],
            instances,
            original.structure().rows().to_vec(),
            original.structure().objective().cloned(),
            CaseLimits::default(),
        )
        .unwrap(),
    );
    values.scalars.insert(id(3), 3.0);
    let cancel = Arc::new(AtomicBool::new(false));
    for (coordinates, entries, constant) in [
        (vec![id(1)], BTreeMap::from([(0, 5.0)]), 15.0),
        (vec![id(3)], BTreeMap::from([(0, 5.0)]), 10.0),
        (
            vec![id(3), id(1)],
            BTreeMap::from([(0, 5.0), (1, 5.0)]),
            0.0,
        ),
    ] {
        let plan = CasePlan::prepare_with_coordinates(
            structure.clone(),
            original.bodies().clone(),
            &registry,
            DerivativeOrder::Value,
            AssemblyLimits::default(),
            &cancel,
            coordinates.clone(),
        )
        .unwrap();
        assert_eq!(plan.columns(), coordinates);
        assert!(plan.structure().free_variables().next().is_none());
        let domain = plan
            .presolve_domain_facts(&values, 100_000, &cancel)
            .unwrap();
        let facts = plan
            .affine_row_facts(&values, &domain, 100_000, &cancel)
            .unwrap();
        let row = plan
            .structure()
            .rows()
            .iter()
            .position(|row| row.id == id(10))
            .unwrap();
        assert_eq!(facts.affine[row].as_ref().unwrap().entries, entries);
        assert_eq!(facts.affine[row].as_ref().unwrap().constant, constant);
        for source in [id(1), id(3)] {
            let mut changed = values.clone();
            changed.scalars.insert(source, 7.0);
            assert_eq!(
                facts.matches(&plan, &changed),
                coordinates.contains(&source)
            );
            if coordinates.contains(&source) {
                let retained = plan
                    .affine_row_facts(&changed, &facts, 100_000, &cancel)
                    .unwrap();
                assert_eq!(retained.key, facts.key);
                assert_eq!(retained.affine, facts.affine);
            } else {
                assert!(matches!(
                    plan.affine_row_facts(&changed, &facts, 100_000, &cancel),
                    Err(crate::MathError::Contract(_))
                ));
            }
        }
    }
}

#[test]
fn affine_selected_output_does_not_examine_unrelated_inputs_of_a_wide_body() {
    use pse_diagnostics::observation::{ObservationHub, Policy, Value};
    use std::time::Duration;
    let registry = standard_registry().unwrap();
    let quantity = ids::quantity("neutral");
    let cancel = Arc::new(AtomicBool::new(false));
    let run = |width: usize| {
        let mut builder = BodyBuilder::new(
            crate::initialize().unwrap(),
            &registry,
            &StandardInvariantChecker,
            width,
            BodyLimits::default(),
        )
        .unwrap();
        let mut inputs = (0..width)
            .map(|slot| {
                builder
                    .input(slot, quantity, IndexSet::new(), id(20))
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let selected = inputs.remove(0);
        let mut unrelated = inputs.remove(0);
        for input in inputs {
            unrelated = builder
                .binary(Binary::Add, unrelated, input, None, id(21))
                .unwrap();
        }
        let body = Arc::new(builder.prepare(&[selected, unrelated]).unwrap());
        let key = ContentHash::from_bytes([1; 32]);
        let variables = (0..width)
            .map(|column| Variable {
                port: Port {
                    id: id(50 + u8::try_from(column).unwrap()),
                    quantity,
                    unit: registry.quantity_type(quantity).unwrap().canonical_unit,
                },
                fixed: false,
                domain: ModelingVariableDomain::Continuous,
                lower: None,
                upper: None,
            })
            .collect::<Vec<_>>();
        let mut ports = variables
            .iter()
            .map(|variable| variable.port.clone())
            .collect::<Vec<_>>();
        // Original slot zero binds to the last global column. The other local slots,
        // including one with that same formal number, belong only to the other output.
        ports.rotate_right(1);
        let slots = ports
            .iter()
            .map(|port| SlotBinding::new(port, port, &registry).unwrap())
            .collect::<Vec<_>>();
        let values = CaseValues {
            scalars: variables
                .iter()
                .map(|variable| (variable.port.id, 2.0))
                .collect(),
        };
        let structure = CaseStructure::new(
            variables,
            vec![],
            vec![InstanceBinding {
                checked_members: Default::default(),
                instance: id(9),
                body: key,
                slots: slots.into(),
                contributions: vec![
                    Contribution {
                        output: 0,
                        target: Target::Row(id(10)),
                        scale: 2.0,
                    },
                    Contribution {
                        output: 0,
                        target: Target::Row(id(10)),
                        scale: 3.0,
                    },
                ],
            }],
            vec![Row {
                id: id(10),
                quantity,
                lower: 0.0,
                upper: 100.0,
            }],
            None,
            CaseLimits::default(),
        )
        .unwrap();
        let plan = CasePlan::prepare(
            Arc::new(structure),
            BTreeMap::from([(key, body)]),
            &registry,
            DerivativeOrder::Value,
            AssemblyLimits::default(),
            &cancel,
        )
        .unwrap();
        let domain = plan
            .presolve_domain_facts(&values, 100_000, &cancel)
            .unwrap();
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("affine-wide-output-locality");
        let entered = operation.context().enter();
        let facts = plan
            .affine_row_facts(&values, &domain, 100_000, &cancel)
            .unwrap();
        drop(entered);
        assert_eq!(
            facts.affine[0].as_ref().unwrap().entries,
            BTreeMap::from([(width - 1, 5.0)])
        );
        assert_eq!(facts.affine[0].as_ref().unwrap().constant, 0.0);
        let counts = || {
            operation
                .reader()
                .snapshot(Duration::from_secs(60))
                .metrics
                .into_iter()
                .filter(|metric| metric.unit == "count" && metric.name.starts_with("affine."))
                .map(|metric| {
                    let Value::Count(count) = metric.cumulative else {
                        panic!("expected actual work count");
                    };
                    (metric.name, count)
                })
                .collect::<BTreeMap<_, _>>()
        };
        let first = counts();
        assert_eq!(counts(), first, "inspection must not execute another proof");
        for name in [
            "selected_rows",
            "input_symbols_examined",
            "input_slots_examined",
            "bound_output_reuses",
            "formal_coordinates",
            "candidate_coordinates",
            "derivative_attempts",
            "nonzero_coefficients",
            "established_rows",
        ] {
            assert_eq!(first[&format!("affine.{name}")], 1, "{name}");
        }
        assert_eq!(first["affine.contributions_examined"], 2);
        assert_eq!(first["affine.substitution_calls"], 2);
        assert_eq!(first["affine.withheld_rows"], 0);
        first
    };
    assert_eq!(run(2), run(64));
}

#[test]
fn selected_functions_prepare_once_keep_contributions_coordinates_and_complete_universe() {
    use pse_diagnostics::observation::{ObservationHub, Policy, Value};
    use std::time::Duration;
    let (original, values) = fixture(false, false);
    let registry = standard_registry().unwrap();
    let control = pse_diagnostics::Cancellation::new(false);
    let cancel = control.flag();
    let mut first = original.structure().instances()[0].clone();
    first.contributions = vec![
        Contribution {
            output: 0,
            target: Target::Row(id(10)),
            scale: 2.0,
        },
        Contribution {
            output: 0,
            target: Target::Row(id(10)),
            scale: -0.5,
        },
        Contribution {
            output: 1,
            target: Target::Row(id(11)),
            scale: 3.0,
        },
    ];
    let mut second = first.clone();
    second.instance = id(8);
    second.contributions = vec![
        Contribution {
            output: 0,
            target: Target::Row(id(10)),
            scale: 0.5,
        },
        Contribution {
            output: 1,
            target: Target::Row(id(11)),
            scale: -1.0,
        },
    ];
    let source = Arc::new(
        CaseStructure::new(
            original.structure().variables().to_vec(),
            vec![],
            vec![first, second],
            original.structure().rows().to_vec(),
            None,
            CaseLimits::default(),
        )
        .unwrap(),
    );
    let needed = CasePlan::prepare(
        source.clone(),
        original.bodies().clone(),
        &registry,
        DerivativeOrder::Value,
        AssemblyLimits::default(),
        &cancel,
    )
    .unwrap();
    for order in [
        DerivativeOrder::Value,
        DerivativeOrder::First,
        DerivativeOrder::Second,
    ] {
        let coordinates = vec![id(2), id(1)];
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("direct-selected-functions");
        let entered = operation.context().enter();
        let mut looked_up = Vec::new();
        let direct = CasePlan::prepare_functions(
            &source,
            |key| {
                looked_up.push(key);
                original.bodies().get(&key).cloned()
            },
            &[id(11), id(10)],
            coordinates.clone(),
            &registry,
            order,
            AssemblyLimits::default(),
            &cancel,
        )
        .unwrap();
        drop(entered);
        assert_eq!(
            looked_up,
            original.bodies().keys().copied().collect::<Vec<_>>()
        );
        let snapshot = operation.reader().snapshot(Duration::from_secs(60));
        for name in [
            "math.case_plan_constructions",
            "math.selected_function_constructions",
            "math.selected_function_bodies",
        ] {
            assert_eq!(
                snapshot
                    .metrics
                    .iter()
                    .find(|m| m.name == name)
                    .unwrap()
                    .cumulative,
                Value::Count(1)
            );
        }
        let projected = needed
            .functions(
                &[id(11), id(10)],
                coordinates.clone(),
                &registry,
                order,
                &cancel,
            )
            .unwrap();
        assert_eq!(direct.columns(), coordinates);
        assert_eq!(
            direct
                .structure()
                .rows()
                .iter()
                .map(|r| r.id)
                .collect::<Vec<_>>(),
            vec![id(10), id(11)]
        );
        assert_eq!(
            direct.structure().coordinate_allocation(),
            source.coordinate_allocation()
        );
        assert_eq!(direct.structure(), projected.structure());
        assert_eq!(direct.demands(), projected.demands());
        assert_eq!(
            direct
                .structure()
                .instances()
                .iter()
                .map(|i| i.contributions.len())
                .sum::<usize>(),
            5
        );
        let compile = |plan: CasePlan| {
            Arc::new(
                Arc::new(plan)
                    .compile(
                        Optimization::default(),
                        EvaluationLimits::default(),
                        &cancel,
                    )
                    .unwrap(),
            )
        };
        let direct = compile(direct);
        let projected = compile(projected);
        let mut worker = direct.worker(BTreeMap::new(), control.clone());
        let mut reference = projected.worker(BTreeMap::new(), control.clone());
        assert_eq!(worker.constraints(&values).unwrap(), vec![12.0, 10.0]);
        assert_eq!(
            worker.constraints(&values).unwrap(),
            reference.constraints(&values).unwrap()
        );
        if order >= DerivativeOrder::First {
            assert_eq!(
                worker.jacobian(&values).unwrap().val(),
                reference.jacobian(&values).unwrap().val()
            );
            assert_eq!(worker.jacobian(&values).unwrap().to_dense()[(0, 0)], 4.0);
            assert_eq!(worker.jacobian(&values).unwrap().to_dense()[(0, 1)], 6.0);
        }
        if order >= DerivativeOrder::Second {
            assert_eq!(
                worker.hessian(&values, 0.0, &[3.0, 2.0]).unwrap().val(),
                reference.hessian(&values, 0.0, &[3.0, 2.0]).unwrap().val()
            );
            assert_eq!(
                worker
                    .hessian(&values, 0.0, &[3.0, 2.0])
                    .unwrap()
                    .to_dense()[(1, 0)],
                6.0
            );
        }
    }
    for (rows, coordinates) in [
        (vec![id(99)], vec![id(1)]),
        (vec![id(10), id(10)], vec![id(1)]),
        (vec![id(10)], vec![id(1), id(1)]),
        (vec![id(10)], vec![id(99)]),
    ] {
        let result = CasePlan::prepare_functions(
            &source,
            |_| panic!("invalid selection acquired a body"),
            &rows,
            coordinates,
            &registry,
            DerivativeOrder::Second,
            AssemblyLimits::default(),
            &cancel,
        );
        assert!(matches!(result, Err(crate::MathError::Contract(_))));
    }
    assert!(matches!(
        CasePlan::prepare_functions(
            &source,
            |_| None,
            &[id(10)],
            vec![],
            &registry,
            DerivativeOrder::Value,
            AssemblyLimits::default(),
            &cancel,
        ),
        Err(crate::MathError::Contract(_))
    ));
}

#[test]
fn selected_functions_ignore_unselected_bad_guard_and_preserve_selected_guard() {
    let (original, mut values) = fixture(false, false);
    let registry = standard_registry().unwrap();
    let quantity = ids::quantity("neutral");
    let mut builder = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &StandardInvariantChecker,
        1,
        BodyLimits::default(),
    )
    .unwrap();
    let x = builder.input(0, quantity, IndexSet::new(), id(31)).unwrap();
    let logarithm = builder.unary(crate::Function::Log, x, id(32)).unwrap();
    let guarded = Arc::new(builder.prepare(&[logarithm]).unwrap());
    let key = ContentHash::from_bytes([2; 32]);
    let mut instances = original.structure().instances().to_vec();
    let port = &original.structure().variables().get(0).unwrap().port;
    instances.push(InstanceBinding {
        instance: id(8),
        body: key,
        checked_members: Default::default(),
        slots: vec![SlotBinding::new(port, port, &registry).unwrap()].into(),
        contributions: vec![Contribution {
            output: 0,
            target: Target::Row(id(11)),
            scale: 1.0,
        }],
    });
    let source = CaseStructure::new(
        original.structure().variables().to_vec(),
        vec![],
        instances,
        original.structure().rows().to_vec(),
        original.structure().objective().cloned(),
        CaseLimits::default(),
    )
    .unwrap();
    let mut bodies = original.bodies().clone();
    bodies.insert(key, guarded.clone());
    let control = pse_diagnostics::Cancellation::new(false);
    let cancel = control.flag();
    values.scalars.insert(id(1), -2.0);
    let prepare = |rows: &[SemanticId]| {
        let plan = CasePlan::prepare_functions(
            &source,
            |key| bodies.get(&key).cloned(),
            rows,
            vec![id(2)],
            &registry,
            DerivativeOrder::First,
            AssemblyLimits::default(),
            &cancel,
        )
        .unwrap();
        Arc::new(
            Arc::new(plan)
                .compile(
                    Optimization::default(),
                    EvaluationLimits::default(),
                    &cancel,
                )
                .unwrap(),
        )
    };
    let selected = prepare(&[id(10)]);
    assert_eq!(selected.bodies().len(), 1);
    let mut worker = selected.worker(BTreeMap::new(), control.clone());
    assert_eq!(worker.constraints(&values).unwrap(), vec![5.0]);
    let selected = prepare(&[id(11)]);
    assert_eq!(selected.bodies().len(), 1);
    assert!(Arc::ptr_eq(&selected.bodies()[&key], &guarded));
    let mut worker = selected.worker(BTreeMap::new(), control.clone());
    assert!(worker.constraints(&values).is_err());
}

#[test]
fn selected_functions_differentiate_fixed_variables_and_parameters_in_request_order() {
    let (original, mut values) = fixture(false, true);
    let registry = standard_registry().unwrap();
    let fixed = &original.structure().variables().get(0).unwrap().port;
    let mut parameter = fixed.clone();
    parameter.id = id(3);
    let mut instances = original.structure().instances().to_vec();
    instances[0].slots = vec![
        SlotBinding::new(fixed, fixed, &registry).unwrap(),
        SlotBinding::new(&parameter, &parameter, &registry).unwrap(),
    ]
    .into();
    let source = CaseStructure::new(
        original.structure().variables().to_vec(),
        vec![parameter],
        instances,
        original.structure().rows().to_vec(),
        original.structure().objective().cloned(),
        CaseLimits::default(),
    )
    .unwrap();
    values.scalars.insert(id(3), 7.0);
    let control = pse_diagnostics::Cancellation::new(false);
    let cancel = control.flag();
    let plan = CasePlan::prepare_functions(
        &source,
        |key| original.bodies().get(&key).cloned(),
        &[id(10)],
        vec![id(3), id(1)],
        &registry,
        DerivativeOrder::Second,
        AssemblyLimits::default(),
        &cancel,
    )
    .unwrap();
    assert_eq!(plan.columns(), &[id(3), id(1)]);
    assert!(plan.structure().free_variables().next().is_none());
    assert_eq!(
        plan.structure().coordinate_allocation(),
        source.coordinate_allocation()
    );
    let assembly = Arc::new(
        Arc::new(plan)
            .compile(
                Optimization::default(),
                EvaluationLimits::default(),
                &cancel,
            )
            .unwrap(),
    );
    let mut worker = assembly.worker(BTreeMap::new(), control.clone());
    assert_eq!(worker.constraints(&values).unwrap(), vec![45.0]);
    assert_eq!(worker.jacobian(&values).unwrap().to_dense()[(0, 0)], 5.0);
    assert_eq!(worker.jacobian(&values).unwrap().to_dense()[(0, 1)], 5.0);
}

#[test]
fn demand_observations_distinguish_order_projection_inputs_and_failed_trials() {
    let (assembly, mut values) = fixture(false, false);
    let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
    let operation = hub.start("math.demand.control");
    let _entered = operation.context().enter();
    let control = pse_diagnostics::Cancellation::new(false);
    let mut worker = assembly.worker(BTreeMap::new(), control.clone());
    let counts = |worker: &CaseWorker, requested| {
        *worker
            .demand_statistics()
            .find(|row| {
                row.instance == id(9)
                    && row.demand == EvaluationDemand::Objective
                    && row.requested == requested
            })
            .unwrap()
            .counts
    };
    assert_eq!(worker.objective(&values).unwrap(), 6.0);
    assert_eq!(worker.objective(&values).unwrap(), 6.0);
    assert_eq!(worker.gradient(&values).unwrap(), vec![3.0, 2.0]);
    assert_eq!(worker.objective(&values).unwrap(), 6.0);
    let value = counts(&worker, DerivativeOrder::Value);
    assert_eq!((value.calls, value.completed, value.evaluations), (3, 3, 1));
    assert_eq!((value.cache_hits, value.cache_projections), (2, 1));
    assert_eq!(value.achieved(DerivativeOrder::Value), 2);
    assert_eq!(value.achieved(DerivativeOrder::First), 1);
    let first = counts(&worker, DerivativeOrder::First);
    assert_eq!((first.evaluations, first.order_upgrade_misses), (1, 1));
    values.scalars.insert(id(1), 4.0);
    assert_eq!(worker.objective(&values).unwrap(), 12.0);
    values.scalars.remove(&id(1));
    assert!(worker.objective(&values).is_err());
    values.scalars.insert(id(1), 4.0);
    assert_eq!(worker.objective(&values).unwrap(), 12.0);
    worker.invalidate_trial();
    assert_eq!(worker.objective(&values).unwrap(), 12.0);
    let value = counts(&worker, DerivativeOrder::Value);
    assert_eq!((value.calls, value.completed, value.failed), (7, 6, 1));
    assert_eq!((value.evaluations, value.empty_misses), (4, 1));
    assert_eq!(
        (value.changed_input_misses, value.invalidated_misses),
        (1, 2)
    );
    assert!(!value.overflowed);
    control.cancel();
    assert!(worker.objective(&values).is_err());
    assert_eq!(
        counts(&worker, DerivativeOrder::Value),
        value,
        "pre-cancellation does not invent an occurrence invocation"
    );
}

#[test]
fn compact_census_borrows_actual_products_and_separates_addends_from_nnz() {
    let (assembly, _) = fixture(true, false);
    let census = assembly.census();
    assert_eq!(census.occurrences, 1);
    assert_eq!(census.templates, assembly.programs().len());
    assert_eq!(
        census.jacobian_addends, 4,
        "two independent axes in each of two authored row contributions"
    );
    assert_eq!(census.jacobian_nnz, 1);
    assert_eq!(census.numeric_worker_bytes, assembly.numeric_worker_bytes());
    for demand in assembly.demand_census() {
        assert!(std::ptr::eq(
            demand.demand,
            &assembly.demands()[demand.template]
        ));
        assert!(std::ptr::eq(
            demand.support,
            assembly.supports()[demand.template].as_ref()
        ));
        assert!(std::ptr::eq(
            demand.program,
            assembly.programs()[demand.template].as_ref()
        ));
        let orders = demand.program.census().collect::<Vec<_>>();
        assert_eq!(orders.len(), demand.demand.order as usize + 1);
        assert_eq!(orders[0].order, DerivativeOrder::Value);
        for order in orders {
            assert!(order.stages >= order.arithmetic_blocks + order.provider_stages);
            assert!(order.library_instructions > 0);
            assert!(order.instruction_storage_bytes >= order.library_instructions);
            assert!(order.frame_entries > 0);
            assert!(order.result_entries >= demand.demand.outputs.len());
            assert_eq!(order.native_arithmetic_blocks, 0);
            assert_eq!(order.provider_stages, 0);
        }
    }
}

#[test]
fn evaluator_census_leaves_unused_derivative_realizations_cold() {
    let (assembly, values) = fixture(false, false);
    let mut worker = assembly.worker(BTreeMap::new(), pse_diagnostics::Cancellation::default());
    assert!(worker.evaluator_census().all(|(_, census)| {
        census.frame_entries == 0
            && census.instantiated_orders == [false; 3]
            && census.used_allowance_bytes == 0
    }));
    worker.objective(&values).unwrap();
    assert!(worker.evaluator_census().all(|(_, census)| {
        !census.instantiated_orders[DerivativeOrder::First as usize]
            && !census.instantiated_orders[DerivativeOrder::Second as usize]
            && census.frame_capacity >= census.frame_entries
            && census.used_allowance_bytes <= census.admitted_bytes
    }));
    worker.gradient(&values).unwrap();
    assert!(
        worker
            .evaluator_census()
            .any(|(_, census)| { census.instantiated_orders[DerivativeOrder::First as usize] })
    );
    assert!(
        worker
            .evaluator_census()
            .all(|(_, census)| { !census.instantiated_orders[DerivativeOrder::Second as usize] })
    );
}

#[test]
fn demand_collection_refusal_and_operation_rebind_preserve_scientific_reuse() {
    use pse_diagnostics::observation::{ObservationHub, Policy};
    let (assembly, values) = fixture(false, false);
    let mut worker = assembly.worker(BTreeMap::new(), pse_diagnostics::Cancellation::default());
    assert_eq!(
        worker.demand_observation_status(),
        DemandObservationStatus::NotRequested
    );
    assert_eq!(worker.objective(&values).unwrap(), 6.0);
    assert_eq!(worker.demand_statistics().count(), 0);
    for policy in [
        Policy {
            enabled: false,
            ..Policy::default()
        },
        Policy {
            budget_bytes: 1,
            ..Policy::default()
        },
    ] {
        let hub = ObservationHub::new(policy);
        let operation = hub.start("math.collection.refused");
        let _entered = operation.context().enter();
        assert_eq!(worker.objective(&values).unwrap(), 6.0);
        assert_eq!(
            worker.demand_observation_status(),
            DemandObservationStatus::Unavailable
        );
        assert_eq!(worker.demand_statistics().count(), 0);
        assert_eq!(
            assembly.numeric_worker_bytes(),
            assembly.census().numeric_worker_bytes
        );
    }
    let hub = ObservationHub::new(Policy::default());
    for _ in 0..2 {
        let operation = hub.start("math.collection.new_operation");
        let _entered = operation.context().enter();
        assert_eq!(worker.objective(&values).unwrap(), 6.0);
        assert_eq!(
            worker.demand_observation_status(),
            DemandObservationStatus::Collected
        );
        let counts = worker
            .demand_statistics()
            .find(|row| {
                row.demand == EvaluationDemand::Objective && row.requested == DerivativeOrder::Value
            })
            .unwrap()
            .counts;
        assert_eq!(
            (
                counts.calls,
                counts.completed,
                counts.cache_hits,
                counts.evaluations
            ),
            (1, 1, 1, 0)
        );
        let snapshot = operation
            .reader()
            .snapshot(std::time::Duration::from_secs(60));
        assert!(snapshot.producers.iter().any(|producer| {
            producer.descriptor.role == pse_diagnostics::observation::ProducerRole::Evaluator
                && producer
                    .current
                    .as_ref()
                    .is_some_and(|record| record.family == "evaluator_demand_batch.v1")
        }));
        assert!(
            snapshot
                .producers
                .iter()
                .flat_map(|producer| &producer.recent)
                .all(|record| record.family != "evaluator_demand.v1"),
            "ordinary bounded observation does not emit one record per occurrence"
        );
    }
}

#[test]
fn selected_demand_rows_preserve_occurrences_and_actual_order_without_eager_work() {
    use pse_diagnostics::observation::{CoherentRecord, ObservationHub, RecordTap, Value};
    #[derive(Debug, Default)]
    struct Tap(std::sync::Mutex<Vec<CoherentRecord>>);
    impl RecordTap for Tap {
        fn selected(&self, family: &str) -> bool {
            matches!(family, "evaluator_demand.v1" | "evaluator_census.v1")
        }
        fn observe(&self, record: &CoherentRecord) {
            if self.selected(&record.family) {
                let mut records = self.0.lock().unwrap();
                assert!(records.len() < 128, "finite fixture collector exhausted");
                records.push(record.clone());
            }
        }
    }
    let (original, values) = fixture(false, false);
    let mut instances = original.structure().instances().to_vec();
    let mut second = instances[0].clone();
    second.instance = id(8);
    instances.push(second);
    let structure = CaseStructure::new(
        original.structure().variables().to_vec(),
        original.structure().parameters().to_vec(),
        instances,
        original.structure().rows().to_vec(),
        original.structure().objective().cloned(),
        CaseLimits::default(),
    )
    .unwrap();
    let control = pse_diagnostics::Cancellation::new(false);
    let cancel = control.flag();
    let assembly = Arc::new(
        Arc::new(
            CasePlan::prepare(
                Arc::new(structure),
                original.bodies().clone(),
                &standard_registry().unwrap(),
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
    assert_eq!(assembly.census().occurrences, 2);
    assert_eq!(assembly.census().templates, original.census().templates);
    let hub = ObservationHub::new(Default::default());
    let operation = hub.start("math.selected_demands.control");
    let tap = Arc::new(Tap::default());
    let _entered = operation.context().with_record_tap(tap.clone()).enter();
    let mut worker = assembly.worker(BTreeMap::new(), control.clone());
    assert_eq!(worker.gradient(&values).unwrap(), vec![6.0, 4.0]);
    assert_eq!(worker.objective(&values).unwrap(), 12.0);
    assert!(
        worker
            .evaluator_census()
            .all(|(_, census)| !census.instantiated_orders[2])
    );
    let records = tap.0.lock().unwrap();
    let rows = records
        .iter()
        .filter(|record| record.family == "evaluator_demand.v1")
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 4);
    let read = |record: &CoherentRecord, name: &str| {
        record
            .values
            .iter()
            .find(|field| field.name == name)
            .unwrap()
            .value
            .clone()
    };
    for instance in [id(8), id(9)] {
        let value = rows
            .iter()
            .find(|record| {
                read(record, "instance") == Value::Text(instance.to_string())
                    && read(record, "requested_order") == Value::Integer(0)
            })
            .unwrap();
        assert_eq!(read(value, "math.demand.evaluations"), Value::Count(0));
        assert_eq!(read(value, "math.demand.cache_hits"), Value::Count(1));
        assert_eq!(
            read(value, "math.demand.cache_projections"),
            Value::Count(1)
        );
        assert_eq!(read(value, "math.demand.achieved_first"), Value::Count(1));
    }
    assert!(
        records
            .iter()
            .any(|record| record.family == "evaluator_census.v1")
    );
    assert!(
        records
            .iter()
            .all(|record| record.producer.operation == operation.context().id())
    );
}

#[test]
fn indexed_row_pressure_keeps_admitted_aggregate_work_and_science_independent() {
    use pse_diagnostics::observation::{ObservationHub, Policy, Value};
    let (assembly, values) = fixture(false, false);
    let mut observed_aggregate_only = false;
    for kib in (32..=128).step_by(8) {
        let hub = ObservationHub::new(Policy {
            budget_bytes: kib * 1024,
            ..Policy::default()
        });
        let operation = hub.start("math.indexed_pressure.control");
        let _entered = operation.context().enter();
        let mut worker = assembly.worker(BTreeMap::new(), pse_diagnostics::Cancellation::default());
        assert_eq!(worker.objective(&values).unwrap(), 6.0);
        if worker.demand_observation_status() == DemandObservationStatus::AggregateOnly {
            observed_aggregate_only = true;
            assert_eq!(worker.demand_statistics().count(), 0);
            let snapshot = operation
                .reader()
                .snapshot(std::time::Duration::from_secs(60));
            assert!(snapshot.metrics.iter().any(|metric| {
                metric.name == "math.demand.calls" && metric.cumulative == Value::Count(1)
            }));
            assert!(
                !snapshot.metrics.iter().any(|metric| {
                    metric.name == "math.demand.empty_misses"
                        && metric.cumulative == Value::Count(1)
                }),
                "missing indexed history cannot establish a cold miss"
            );
        }
        assert!(hub.retained_bytes() <= kib * 1024);
    }
    assert!(
        observed_aggregate_only,
        "bounded pressure must exercise the aggregate-only branch"
    );
}

#[test]
fn complete_constraint_and_gradient_results_reuse_owned_storage_after_failed_trial() {
    let (assembly, mut values) = fixture(false, false);
    let mut worker = assembly.worker(BTreeMap::new(), pse_diagnostics::Cancellation::default());
    let constraints = worker.constraints(&values).unwrap().as_ptr();
    let gradient = worker.gradient(&values).unwrap().as_ptr();
    values.scalars.insert(id(1), 3.);
    assert_eq!(worker.constraints(&values).unwrap().as_ptr(), constraints);
    assert_eq!(worker.gradient(&values).unwrap().as_ptr(), gradient);
    values.scalars.insert(id(1), f64::NAN);
    assert!(worker.constraints(&values).is_err());
    assert!(worker.gradient(&values).is_err());
    values.scalars.insert(id(1), 2.);
    assert_eq!(worker.constraints(&values).unwrap().as_ptr(), constraints);
    assert_eq!(worker.gradient(&values).unwrap().as_ptr(), gradient);
    assert_eq!(worker.gradient(&values).unwrap(), &[3., 2.]);
}
