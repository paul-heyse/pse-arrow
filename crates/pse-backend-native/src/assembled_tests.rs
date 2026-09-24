// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use pse_math::{
    assembly::{AssemblyLimits, CaseAssembly},
    binding::{
        CaseLimits, CaseStructure, CaseValues, Contribution, InstanceBinding, Row, SlotBinding,
        Target, VariableDomain,
    },
    jets::EvaluationLimits,
    library::Optimization,
    typed::{BodyBuilder, BodyLimits},
};
use pse_quantity::{
    IndexSet,
    standard::{StandardInvariantChecker, ids, standard_registry},
};
use std::collections::BTreeMap;
fn fixture(domain: VariableDomain) -> (Arc<CaseAssembly>, CaseValues) {
    let id = |n| SemanticId::from_bytes([n; 16]);
    let registry = standard_registry().unwrap();
    let quantity = ids::quantity("neutral");
    let port = pse_kernels::Port {
        id: id(1),
        quantity,
        unit: registry.quantity_type(quantity).unwrap().canonical_unit,
    };
    let mut builder = BodyBuilder::new(
        &registry,
        &StandardInvariantChecker,
        1,
        BodyLimits::default(),
    )
    .unwrap();
    let value = builder.input(0, quantity, IndexSet::new(), id(3)).unwrap();
    let body = Arc::new(builder.prepare(&[value]).unwrap());
    let key = ContentHash::from_bytes([1; 32]);
    let structure = Arc::new(
        CaseStructure::new(
            vec![pse_math::binding::Variable {
                port: port.clone(),
                fixed: false,
                domain,
                lower: Some(0.0),
                upper: Some(10.0),
            }],
            vec![],
            vec![InstanceBinding {
                instance: id(4),
                body: key,
                slots: vec![SlotBinding::new(&port, &port, &registry).unwrap()],
                contributions: vec![Contribution {
                    output: 0,
                    target: Target::Row(id(2)),
                    scale: 1.0,
                }],
            }],
            vec![Row {
                id: id(2),
                quantity,
                lower: 2.0,
                upper: 2.0,
            }],
            None,
            CaseLimits::default(),
        )
        .unwrap(),
    );
    let assembly = Arc::new(
        Arc::new(
            pse_math::assembly::CasePlan::prepare(
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
            scalars: BTreeMap::from([(id(1), 3.0)]),
        },
    )
}
#[test]
fn split_native_callbacks_and_equality_root_views() {
    let (a, values) = fixture(VariableDomain::Continuous);
    let worker = a.worker(BTreeMap::new(), Arc::new(AtomicBool::new(false)));
    let mut oracle = assembled::AlgebraicOracle::new(worker, values).unwrap();
    oracle.admit_nle().unwrap();
    let mut out = [-7.0];
    NleOracle::residual(&mut oracle, &[3.0], &mut out).unwrap();
    assert_eq!(out, [1.0]);
    NleOracle::jacobian(&mut oracle, &[3.0], &mut out).unwrap();
    assert_eq!(out, [1.0]);
    NleOracle::jacobian_product(&mut oracle, &[3.0], &[5.0], &mut out).unwrap();
    assert_eq!(out, [5.0]);
    assert_eq!(NlpOracle::objective(&mut oracle, &[3.0]).unwrap(), 0.0);
    NlpOracle::gradient(&mut oracle, &[3.0], &mut out).unwrap();
    assert_eq!(out, [0.0]);
    assert!(NlpOracle::constraints(&mut oracle, &[f64::NAN], &mut out).is_err());
    assert_eq!(out, [0.0]);
}
#[test]
fn integrality_is_explicit_and_continuous_oracles_reject_it() {
    let (a, v) = fixture(VariableDomain::Integer);
    let c = a
        .coefficients(
            &v,
            Optimization::default(),
            100,
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    let p = CoefficientProblem::from_plan(&a, c).unwrap();
    assert_eq!(p.domains, vec![VariableDomain::Integer]);
    p.validate_convex(None).unwrap();
    assert!(
        assembled::AlgebraicOracle::new(
            a.worker(BTreeMap::new(), Arc::new(AtomicBool::new(false))),
            v
        )
        .is_err()
    );
}
#[test]
fn explicit_cones_validate_storage_dimensions_parameters_and_psd() {
    let (a, _) = fixture(VariableDomain::Continuous);
    let q = faer::sparse::SparseColMat::try_new_from_triplets(
        1,
        1,
        &[faer::sparse::Triplet::new(0, 0, 2.0)],
    )
    .unwrap();
    let cert =
        GramCertificate::new(&q, 1.0, &faer::Mat::from_fn(1, 1, |_, _| 1.0), &[2.0], 10).unwrap();
    let mut p = ConicProblem {
        contract: assembled::contract(&a),
        quadratic: clarabel::algebra::CscMatrix::new(1, 1, vec![0, 1], vec![0], vec![2.0]),
        objective: vec![0.0],
        constraints: clarabel::algebra::CscMatrix::new(1, 1, vec![0, 1], vec![0], vec![1.0]),
        rhs: vec![1.0],
        cones: vec![clarabel::solver::SupportedConeT::NonnegativeConeT(1)],
        objective_constant: 0.0,
    };
    p.validate(&cert).unwrap();
    p.cones = vec![clarabel::solver::SupportedConeT::PowerConeT(0.0)];
    assert!(p.validate(&cert).is_err());
    p.cones = vec![clarabel::solver::SupportedConeT::ZeroConeT(1)];
    p.quadratic.nzval[0] = -2.0;
    assert!(p.validate(&cert).is_err());
    p.quadratic.nzval[0] = 2.0;
    p.constraints.rowval[0] = 2;
    assert!(p.validate(&cert).is_err());
}
