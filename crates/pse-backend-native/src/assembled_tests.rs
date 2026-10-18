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
    fixture_provider(domain, None)
}
fn fixture_provider(
    domain: ModelingVariableDomain,
    provider: Option<&pse_kernels::ProviderSpec>,
) -> (Arc<CaseAssembly>, CaseValues) {
    fixture_provider_order(domain, provider, DerivativeOrder::Second)
}
fn fixture_provider_order(
    domain: ModelingVariableDomain,
    provider: Option<&pse_kernels::ProviderSpec>,
    order: DerivativeOrder,
) -> (Arc<CaseAssembly>, CaseValues) {
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
    let value = if let Some(spec) = provider {
        builder
            .provider(
                &pse_kernels::AdmittedProvider::new(spec.clone(), &registry).unwrap(),
                &[value],
                id(99),
            )
            .unwrap()
            .remove(0)
    } else {
        value
    };
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
                order,
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
            &pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
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
    let worker = a.worker(
        pse_kernels::ProviderBindings::new(),
        pse_diagnostics::Cancellation::default(),
    );
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
        assembly.worker(
            pse_kernels::ProviderBindings::new(),
            pse_diagnostics::Cancellation::default(),
        ),
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
            a.worker(
                pse_kernels::ProviderBindings::new(),
                pse_diagnostics::Cancellation::default()
            ),
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
                assembly.worker(
                    pse_kernels::ProviderBindings::new(),
                    pse_diagnostics::Cancellation::default(),
                ),
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

#[test]
fn repeated_native_completed_callbacks_preserve_point_direction_and_atomic_boundaries() {
    let (assembly, values) = fixture(ModelingVariableDomain::Continuous);
    let mut oracle = assembled::AlgebraicOracle::new(
        assembly.worker(
            pse_kernels::ProviderBindings::new(),
            pse_diagnostics::Cancellation::default(),
        ),
        values,
    )
    .unwrap();
    let mut output = [999.];
    for _ in 0..3 {
        NlpOracle::constraints(&mut oracle, &[3.], &mut output).unwrap();
        assert_eq!(output, [3.]);
        NleOracle::residual(&mut oracle, &[3.], &mut output).unwrap();
        assert_eq!(output, [1.]);
        NlpOracle::jacobian(&mut oracle, &[3.], &mut output).unwrap();
        assert_eq!(output, [1.]);
        NlpOracle::gradient(&mut oracle, &[3.], &mut output).unwrap();
        assert_eq!(output, [0.]);
        NleOracle::jacobian_product(&mut oracle, &[3.], &[5.], &mut output).unwrap();
        assert_eq!(output, [5.]);
    }
    NleOracle::jacobian_product(&mut oracle, &[3.], &[7.], &mut output).unwrap();
    assert_eq!(output, [7.]);
    assert!(
        NleOracle::jacobian_product(&mut oracle, &[3.], &[f64::INFINITY], &mut output).is_err()
    );
    assert_eq!(output, [7.]);
    assert!(matches!(
        NleOracle::jacobian_product(&mut oracle, &[f64::NAN], &[5.], &mut output),
        Err(ProblemError::Numerical { .. })
    ));
    assert_eq!(output, [7.]);
    NlpOracle::jacobian(&mut oracle, &[4.], &mut output).unwrap();
    NleOracle::jacobian_product(&mut oracle, &[4.], &[5.], &mut output).unwrap();
    assert_eq!(output, [5.]);
    // Final assessment independently returns original, unshifted values.
    NlpOracle::constraints(&mut oracle, &[4.], &mut output).unwrap();
    assert_eq!(output, [4.]);
    assert_eq!(NlpOracle::constraint_sources(&oracle).unwrap()[0].value, 4.);
}

#[test]
fn fresh_native_oracle_prepares_provider_demands_before_any_callback_effect() {
    use pse_kernels::{
        EvaluationContext, PreparedProviderRequest, Provider, ProviderDemand, ProviderError,
        ProviderOutput, ProviderRuntime,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};
    #[derive(Debug)]
    struct Prepared {
        preparations: Arc<AtomicUsize>,
        effects: Arc<AtomicUsize>,
        orders: Arc<AtomicUsize>,
    }
    impl Provider for Prepared {
        fn supports(&self, _: &ProviderDemand) -> bool {
            true
        }
        fn prepare_demand(
            &mut self,
            request: &PreparedProviderRequest,
        ) -> Result<(), ProviderError> {
            self.preparations.fetch_add(1, Ordering::Relaxed);
            self.orders
                .fetch_or(1 << request.order as u8, Ordering::Relaxed);
            Ok(())
        }
        fn fill(
            &mut self,
            x: &[f64],
            request: &PreparedProviderRequest,
            runtime: ProviderRuntime<'_>,
            output: ProviderOutput<'_>,
            _: &EvaluationContext<'_>,
        ) -> Result<(), ProviderError> {
            assert_ne!(
                self.orders.load(Ordering::Relaxed) & (1 << request.order as u8),
                0,
                "native callback cannot prepare a new demand"
            );
            self.effects.fetch_add(1, Ordering::Relaxed);
            output.values[0] = x[0];
            output.jacobian.fill(1.);
            output.hessians.fill(0.);
            if matches!(request.demand, ProviderDemand::JacobianAction { .. }) {
                output.jacobian[0] = runtime.directions[0];
            }
            Ok(())
        }
    }
    let registry = standard_registry().unwrap();
    let quantity = ids::quantity("neutral");
    let unit = registry.quantity_type(quantity).unwrap().canonical_unit;
    let port = |n| pse_kernels::Port {
        id: SemanticId::from_bytes([n; 16]),
        quantity,
        unit,
    };
    let spec = pse_kernels::ProviderSpec {
        id: SemanticId::from_bytes([85; 16]),
        revision: ContentHash::from_bytes([86; 32]),
        data: ContentHash::from_bytes([87; 32]),
        inputs: vec![port(88)],
        outputs: vec![port(89)],
        shapes: Default::default(),
        derivative_source: pse_kernels::DerivativeSource::Analytic,
        derivatives: DerivativeOrder::Second,
        smoothness: DerivativeOrder::Second,
    };
    for order in [DerivativeOrder::First, DerivativeOrder::Second] {
        let (assembly, values) =
            fixture_provider_order(ModelingVariableDomain::Continuous, Some(&spec), order);
        let preparations = Arc::new(AtomicUsize::new(0));
        let effects = Arc::new(AtomicUsize::new(0));
        let orders = Arc::new(AtomicUsize::new(0));
        let binding = pse_kernels::ProviderBinding::new(
            spec.clone(),
            Box::new(Prepared {
                preparations: preparations.clone(),
                effects: effects.clone(),
                orders: orders.clone(),
            }),
        );
        let worker = assembly.worker(
            pse_kernels::ProviderBindings::from([(spec.key(), binding)]),
            pse_diagnostics::Cancellation::default(),
        );
        assert_eq!(preparations.load(Ordering::Relaxed), 0);
        let mut oracle = assembled::AlgebraicOracle::new(worker, values).unwrap();
        let prepared = preparations.load(Ordering::Relaxed);
        assert!(prepared > 0);
        assert_eq!(effects.load(Ordering::Relaxed), 0);
        if order == DerivativeOrder::First {
            assert_eq!(
                orders.load(Ordering::Relaxed) & (1 << DerivativeOrder::Second as u8),
                0,
                "a First-only bound program never prepares unused Second capability"
            );
        }
        let mut result = [91.];
        NleOracle::residual(&mut oracle, &[3.], &mut result).unwrap();
        assert_eq!(result, [1.]);
        NleOracle::jacobian_product(&mut oracle, &[3.], &[5.], &mut result).unwrap();
        assert_eq!(result, [5.]);
        assert_eq!(
            preparations.load(Ordering::Relaxed),
            prepared,
            "callbacks only use precompiled demand set"
        );
        assert!(effects.load(Ordering::Relaxed) >= 2);
    }
}

#[test]
fn shared_conditional_bindings_keep_current_facts_and_lease_without_unread_free_values() {
    let (base, _) = fixture(ModelingVariableDomain::Continuous);
    let id = |n| SemanticId::from_bytes([n; 16]);
    let registry = standard_registry().unwrap();
    let mut variables = base.structure().variables().to_vec();
    let mut unread = variables[0].clone();
    unread.port.id = id(8);
    variables.push(unread);
    let mut frozen = variables[0].clone();
    frozen.port.id = id(10);
    frozen.fixed = true;
    variables.push(frozen);
    let mut parameter = variables[0].port.clone();
    parameter.id = id(9);
    let structure = Arc::new(
        CaseStructure::new(
            variables,
            vec![parameter],
            base.structure().instances().to_vec(),
            base.structure().rows().to_vec(),
            None,
            CaseLimits::default(),
        )
        .unwrap(),
    );
    let stop = pse_diagnostics::Cancellation::default();
    let original = pse_math::assembly::CasePlan::prepare(
        structure,
        base.bodies().clone(),
        &registry,
        DerivativeOrder::Second,
        AssemblyLimits::default(),
        &stop.flag(),
    )
    .unwrap();
    let plan = Arc::new(
        original
            .conditional(
                &[id(2)].into_iter().collect(),
                &[id(1)].into_iter().collect(),
                &registry,
                &stop.flag(),
            )
            .unwrap(),
    );
    let compact = CaseValues {
        scalars: BTreeMap::from([(id(1), 3.0), (id(9), 7.0), (id(10), 4.0)]),
    };
    assert!(
        plan.structure().validate_values(&compact).is_err(),
        "full-universe public admission remains unchanged"
    );
    assert!(
        plan.structure().validate_frozen_values(&compact).is_err(),
        "effective conditional frozen admission remains unchanged"
    );
    assert_eq!(
        plan.structure().source_frozen_ids().collect::<Vec<_>>(),
        vec![id(9), id(10)]
    );
    plan.structure()
        .validate_source_frozen_values(&compact)
        .unwrap();
    let facts = Arc::new(
        plan.presolve_domain_facts(&compact, 10000, &stop.flag())
            .unwrap(),
    );
    let assembly = Arc::new(
        plan.compile(
            Optimization::default(),
            EvaluationLimits::default(),
            &pse_kernels::ExecutionScope::new(stop.clone(), None),
        )
        .unwrap(),
    );
    let worker = || assembly.worker(pse_kernels::ProviderBindings::new(), stop.clone());
    for bad in [
        CaseValues {
            scalars: BTreeMap::from([(id(1), 3.), (id(10), 4.)]),
        },
        CaseValues {
            scalars: BTreeMap::from([(id(9), 7.), (id(10), 4.)]),
        },
        CaseValues {
            scalars: BTreeMap::from([(id(1), 3.), (id(9), 7.)]),
        },
        CaseValues {
            scalars: BTreeMap::from([(id(1), f64::NAN), (id(9), 7.), (id(10), 4.)]),
        },
    ] {
        assert!(assembled::AlgebraicOracle::conditional(worker(), Arc::new(bad).into()).is_err());
    }
    #[derive(Debug)]
    struct Owner(Arc<std::sync::atomic::AtomicBool>);
    impl Drop for Owner {
        fn drop(&mut self) {
            self.0.store(true, std::sync::atomic::Ordering::Release);
        }
    }
    let released = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let shared = pse_math::SharedAllocation::from(Arc::new(compact))
        .with_owner(Arc::new(Owner(released.clone())));
    let mut oracle = assembled::AlgebraicOracle::conditional(worker(), shared.clone())
        .unwrap()
        .with_presolve_facts(facts.into())
        .unwrap();
    drop(shared);
    assert!(!released.load(std::sync::atomic::Ordering::Acquire));
    let mut residual = [0.];
    NleOracle::residual(&mut oracle, &[3.], &mut residual).unwrap();
    assert_eq!(residual, [1.]);
    drop(oracle);
    assert!(released.load(std::sync::atomic::Ordering::Acquire));
}
