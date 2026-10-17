// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use pse_math::{
    assembly::{AssemblyLimits, CaseAssembly},
    binding::{
        CaseLimits, CaseStructure, CaseValues, Contribution, InstanceBinding, Row, SlotBinding,
        Target,
    },
    jets::EvaluationLimits,
    library::Optimization,
    typed::{BodyBuilder, BodyLimits},
};
use pse_model::generated::enums::ModelingVariableDomain;
use pse_quantity::{
    IndexSet,
    standard::{StandardInvariantChecker, ids, standard_registry},
};
use std::collections::BTreeMap;
fn fixture(domain: ModelingVariableDomain) -> (Arc<CaseAssembly>, CaseValues) {
    let id = |n| SemanticId::from_bytes([n; 16]);
    let registry = standard_registry().unwrap();
    let quantity = ids::quantity("neutral");
    let port = pse_kernels::Port {
        id: id(1),
        quantity,
        unit: registry.quantity_type(quantity).unwrap().canonical_unit,
    };
    let mut builder = BodyBuilder::new(
        pse_math::initialize().unwrap(),
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
                checked_members: Default::default(),
                instance: id(4),
                body: key,
                slots: (vec![SlotBinding::new(&port, &port, &registry).unwrap()]).into(),
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
                &pse_diagnostics::Cancellation::default().flag(),
            )
            .unwrap()
            .with_directional_actions(&pse_diagnostics::Cancellation::default().flag())
            .unwrap(),
        )
        .compile(
            Optimization::default(),
            EvaluationLimits::default(),
            &pse_diagnostics::Cancellation::default().flag(),
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
    pse_math::initialize().unwrap();
    let (a, values) = fixture(ModelingVariableDomain::Continuous);
    let worker = a.worker(BTreeMap::new(), pse_diagnostics::Cancellation::default());
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
fn native_dense_trial_preserves_atomic_failure_and_recovers() {
    let (assembly, values) = fixture(ModelingVariableDomain::Continuous);
    let mut oracle = assembled::AlgebraicOracle::new(
        assembly.worker(BTreeMap::new(), pse_diagnostics::Cancellation::default()),
        values.clone(),
    )
    .unwrap();
    let mut output = [99.0];
    NleOracle::residual(&mut oracle, &[7.0], &mut output).unwrap();
    assert_eq!(output, [5.0]);
    assert!(NleOracle::residual(&mut oracle, &[f64::INFINITY], &mut output).is_err());
    assert_eq!(output, [5.0]);
    assert!(NlpOracle::constraint_sources(&oracle).is_err());
    NleOracle::residual(&mut oracle, &[4.0], &mut output).unwrap();
    assert_eq!(output, [2.0]);
}
#[test]
fn integrality_is_explicit_and_continuous_oracles_reject_it() {
    pse_math::initialize().unwrap();
    let (a, v) = fixture(ModelingVariableDomain::Integer);
    // Include the bounded construction of the original coefficient-class proof.
    let c = a
        .coefficients(&v, 1024, &pse_diagnostics::Cancellation::default().flag())
        .unwrap();
    let p = CoefficientProblem::from_plan(&a, c).unwrap();
    assert_eq!(p.domains, vec![ModelingVariableDomain::Integer]);
    p.validate_convex(None).unwrap();
    assert!(
        assembled::AlgebraicOracle::new(
            a.worker(BTreeMap::new(), pse_diagnostics::Cancellation::default()),
            v
        )
        .is_err()
    );
}
#[test]
fn explicit_cones_validate_storage_dimensions_parameters_and_psd() {
    pse_math::initialize().unwrap();
    let (a, _) = fixture(ModelingVariableDomain::Continuous);
    let q = faer::sparse::SparseColMat::try_new_from_triplets(
        1,
        1,
        &[faer::sparse::Triplet::new(0, 0, 2.0)],
    )
    .unwrap();
    let cert = solver_tests::certify(&q, 1.0);
    let mut p = ConicProblem {
        contract: assembled::contract(&a),
        quadratic: conic::SparseMatrix::new(1, 1, vec![0, 1], vec![0], vec![2.0]),
        objective: vec![0.0],
        constraints: conic::SparseMatrix::new(1, 1, vec![0, 1], vec![0], vec![1.0]),
        rhs: vec![1.0],
        cones: vec![conic::Cone::Nonnegative { dimension: 1 }],
        objective_constant: 0.0,
    };
    p.validate(&cert).unwrap();
    p.cones = vec![conic::Cone::Power { alpha: 0.0 }];
    assert!(p.validate(&cert).is_err());
    p.cones = vec![conic::Cone::Zero { dimension: 1 }];
    p.quadratic.values[0] = -2.0;
    assert!(p.validate(&cert).is_err());
    p.quadratic.values[0] = 2.0;
    p.constraints.row_indices[0] = 2;
    assert!(p.validate(&cert).is_err());
}

#[test]
fn shared_conditional_oracle_lends_one_worker_and_keeps_other_owners_private() {
    let (assembly, values) = fixture(ModelingVariableDomain::Continuous);
    let create = || {
        assembled::SharedRootOracle::new(
            assembled::AlgebraicOracle::compact(
                assembly.worker(BTreeMap::new(), pse_diagnostics::Cancellation::default()),
                |id| values.scalars.get(&id).copied(),
            )
            .unwrap(),
        )
        .unwrap()
    };
    let mut one = create();
    let mut loan = one.clone();
    let mut other = create();
    let mut output = [99.0];
    NleOracle::residual(&mut one, &[3.0], &mut output).unwrap();
    assert_eq!(output, [1.0]);
    NleOracle::residual(&mut other, &[8.0], &mut output).unwrap();
    assert_eq!(output, [6.0]);
    // A loan sees the same current source witness, the independent owner does not.
    let loan_observation = NleOracle::observe(&loan, vec![1.0]).unwrap();
    let other_observation = NleOracle::observe(&other, vec![6.0]).unwrap();
    assert_eq!(loan_observation.sources[0].value, 3.0);
    assert_eq!(other_observation.sources[0].value, 8.0);
    assert!(
        one.refill_inputs(&[(SemanticId::from_bytes([250; 16]), 0.0)])
            .is_err()
    );
    assert!(NleOracle::observe(&loan, vec![1.0]).is_err());
    NleOracle::residual(&mut loan, &[4.0], &mut output).unwrap();
    assert_eq!(output, [2.0]);
    assert!(NleOracle::observe(&one, vec![2.0]).is_ok());
    NleOracle::jacobian_product(&mut loan, &[4.0], &[5.0], &mut output).unwrap();
    assert_eq!(output, [5.0]);
}
