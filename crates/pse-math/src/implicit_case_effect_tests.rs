// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Operational selector effects through actual enclosing assembly demands.
use super::*;
use crate::assembly::{AssemblyLimits, CaseAssembly, CasePlan, CaseWorker};
use crate::binding::{
    CaseLimits, CaseStructure, CaseValues, Contribution, InstanceBinding, Objective,
    ObjectiveSense, Row, SlotBinding, Target, Variable,
};
use pse_model::generated::enums::ModelingVariableDomain;

fn operational_factory(
    control: &pse_diagnostics::Cancellation,
    restriction: bool,
) -> (Factory, Arc<Work>, SelectorEffect) {
    let (mut factory, work) = factory(control);
    let mut spec = factory.spec.clone();
    spec.id = pse_ids::named_id(spec.id, "operational selector provider");
    let effect = SelectorEffect {
        spec: spec.clone(),
        value: Arc::new(AtomicU64::new(
            if restriction { 1_f64 } else { 0.9_f64 }.to_bits(),
        )),
        calls: Arc::new(AtomicUsize::new(0)),
        refuse: Arc::new(AtomicBool::new(false)),
    };
    factory.providers.insert(
        spec.key(),
        pse_kernels::Registration::new(
            Arc::new(effect.clone()),
            &pse_quantity::standard::standard_registry().unwrap(),
        )
        .unwrap(),
    );
    let order = if restriction {
        DerivativeOrder::First
    } else {
        DerivativeOrder::Value
    };
    let source = PreparedBody::new(
        3,
        4,
        vec![3],
        vec![Stage::Provider {
            spec,
            partial: vec![],
            inputs: vec![1, 2],
            outputs: vec![3],
            source: factory.spec.id,
        }],
        order,
    )
    .unwrap();
    let program = Arc::new(
        source
            .compile(
                &[0],
                &[0, 1, 2],
                order,
                Optimization::default(),
                EvaluationLimits::default(),
                &ExecutionScope::new(control.clone(), None),
            )
            .unwrap(),
    );
    if restriction {
        factory.selection.restriction = Some(program);
    } else {
        factory.selection.anchor = Some(program);
    }
    (factory, work, effect)
}

fn case(factory: &Factory) -> (Arc<CaseAssembly>, CaseValues) {
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let id = factory.spec.id;
    let body_key = ContentHash::from_bytes([46; 32]);
    let quantity = factory.spec.outputs[0].quantity;
    let row = pse_ids::named_id(id, "enclosing constraint");
    let mut body = PreparedBody::new(
        2,
        3,
        vec![2],
        vec![Stage::Provider {
            spec: factory.spec.clone(),
            partial: vec![],
            inputs: vec![0, 1],
            outputs: vec![2],
            source: id,
        }],
        DerivativeOrder::Second,
    )
    .unwrap();
    body.set_quantities(vec![Some(quantity); 2], vec![quantity]);
    let structure = Arc::new(
        CaseStructure::new(
            factory
                .spec
                .inputs
                .iter()
                .map(|port| Variable {
                    port: port.clone(),
                    fixed: false,
                    domain: ModelingVariableDomain::Continuous,
                    lower: None,
                    upper: None,
                })
                .collect(),
            vec![],
            vec![InstanceBinding {
                checked_members: Default::default(),
                instance: id,
                body: body_key,
                slots: factory
                    .spec
                    .inputs
                    .iter()
                    .map(|port| SlotBinding::new(port, port, &registry).unwrap())
                    .collect::<Vec<_>>()
                    .into(),
                contributions: vec![
                    Contribution {
                        output: 0,
                        target: Target::Row(row),
                        scale: 1.,
                    },
                    Contribution {
                        output: 0,
                        target: Target::PRIMARY,
                        scale: 1.,
                    },
                ],
            }],
            vec![Row {
                id: row,
                quantity,
                lower: 0.,
                upper: 2.,
            }],
            Some(Objective {
                quantity,
                sense: ObjectiveSense::Minimize,
            }),
            CaseLimits::default(),
        )
        .unwrap(),
    );
    let scope = ExecutionScope::new(factory.cancel.clone(), None);
    let plan = CasePlan::prepare(
        structure,
        BTreeMap::from([(body_key, Arc::new(body))]),
        &registry,
        DerivativeOrder::Second,
        AssemblyLimits::default(),
        &factory.cancel.flag(),
    )
    .unwrap()
    .with_directional_actions(&factory.cancel.flag())
    .unwrap()
    .with_hessian_actions(&factory.cancel.flag())
    .unwrap();
    let assembly = Arc::new(
        Arc::new(plan)
            .compile(Optimization::default(), EvaluationLimits::default(), &scope)
            .unwrap(),
    );
    let values = CaseValues {
        scalars: BTreeMap::from([
            (factory.spec.inputs[0].id, 287. / 1200.),
            (factory.spec.inputs[1].id, 0.05),
        ]),
    };
    (assembly, values)
}

#[derive(Clone, Copy, Debug)]
enum Demand {
    Value,
    First,
    Weighted,
    HessianAction,
    JacobianAction,
}
impl Demand {
    fn visits(self) -> usize {
        if matches!(self, Self::HessianAction) {
            3
        } else {
            1
        }
    }
    fn evaluate(self, worker: &mut CaseWorker, values: &CaseValues) -> Result<Vec<f64>, MathError> {
        match self {
            Self::Value => Ok(worker.constraints(values)?.to_vec()),
            Self::First => Ok(worker.jacobian(values)?.val().to_vec()),
            Self::Weighted => Ok(worker.hessian(values, 2., &[3.])?.val().to_vec()),
            Self::HessianAction => {
                let mut out = vec![f64::NAN; 2];
                worker.hessian_product(values, 2., &[3.], &[5., 7.], &mut out)?;
                Ok(out)
            }
            Self::JacobianAction => {
                let mut out = vec![f64::NAN];
                worker.jacobian_product(values, &[5., 7.], &mut out)?;
                Ok(out)
            }
        }
    }
    fn expected(self, assembly: &CaseAssembly, spec: &ProviderSpec) -> Vec<f64> {
        let jet = oracle(287. / 1200., 0.05, 0.8);
        let axes = assembly
            .columns()
            .iter()
            .map(|port| {
                spec.inputs
                    .iter()
                    .position(|input| input.id == *port)
                    .unwrap()
            })
            .collect::<Vec<_>>();
        match self {
            Self::Value => jet.values,
            Self::First => axes.iter().map(|&i| jet.jacobian[i]).collect(),
            Self::Weighted => vec![
                5. * jet.hessians[axes[0] * 2 + axes[0]],
                5. * jet.hessians[axes[1] * 2 + axes[0]],
                5. * jet.hessians[axes[1] * 2 + axes[1]],
            ],
            Self::HessianAction => axes
                .iter()
                .map(|&i| {
                    5. * (jet.hessians[i * 2 + axes[0]] * 5. + jet.hessians[i * 2 + axes[1]] * 7.)
                })
                .collect(),
            Self::JacobianAction => vec![jet.jacobian[axes[0]] * 5. + jet.jacobian[axes[1]] * 7.],
        }
    }
}

fn assert_trial(error: &MathError) {
    match error {
        MathError::Instance { cause, .. } => assert_trial(cause),
        MathError::Provider {
            cause: pse_kernels::ProviderError::Trial(_),
            ..
        } => {}
        other => panic!("expected retained typed operational refusal, got {other:?}"),
    }
}

#[test]
fn case_worker_same_key_visits_operational_anchor_and_restriction_for_partials_and_products() {
    for restriction in [false, true] {
        for demand in [
            Demand::Value,
            Demand::First,
            Demand::Weighted,
            Demand::HessianAction,
            Demand::JacobianAction,
        ] {
            let control = pse_diagnostics::Cancellation::default();
            let (factory, work, effect) = operational_factory(&control, restriction);
            let (assembly, values) = case(&factory);
            let expected = demand.expected(&assembly, &factory.spec);
            let mut worker = assembly.worker(
                [(factory.spec.key(), factory.create().unwrap())]
                    .into_iter()
                    .collect(),
                control.clone(),
            );
            for _ in 0..2 {
                close(&demand.evaluate(&mut worker, &values).unwrap(), &expected);
            }
            assert_eq!(
                effect.calls.load(Ordering::Relaxed),
                2 * demand.visits(),
                "{restriction} {demand:?}"
            );
            assert_eq!(work.solves.load(Ordering::Relaxed), 1);
            if !restriction {
                effect.value.store(0.8_f64.to_bits(), Ordering::Release);
                close(&demand.evaluate(&mut worker, &values).unwrap(), &expected);
                assert_eq!(
                    work.solves.load(Ordering::Relaxed),
                    2,
                    "new anchor reaches numerical owner"
                );
            } else {
                effect.value.store(0_f64.to_bits(), Ordering::Release);
                assert!(
                    demand.evaluate(&mut worker, &values).is_err(),
                    "changed restriction rejects the previous root"
                );
                effect.value.store(1_f64.to_bits(), Ordering::Release);
                close(&demand.evaluate(&mut worker, &values).unwrap(), &expected);
                assert_eq!(
                    work.solves.load(Ordering::Relaxed),
                    2,
                    "restriction refusal invalidates numerical completion"
                );
            }
            let before = effect.calls.load(Ordering::Relaxed);
            effect.refuse.store(true, Ordering::Release);
            assert_trial(&demand.evaluate(&mut worker, &values).unwrap_err());
            assert_eq!(effect.calls.load(Ordering::Relaxed), before + 1);
            effect.refuse.store(false, Ordering::Release);
            close(&demand.evaluate(&mut worker, &values).unwrap(), &expected);
            assert_eq!(
                effect.calls.load(Ordering::Relaxed),
                before + 1 + demand.visits()
            );
            let before = effect.calls.load(Ordering::Relaxed);
            control.cancel();
            assert!(demand.evaluate(&mut worker, &values).is_err());
            assert_eq!(
                effect.calls.load(Ordering::Relaxed),
                before,
                "pre-cancelled request has no provider effects"
            );
            control.flag().store(false, Ordering::Release);
            close(&demand.evaluate(&mut worker, &values).unwrap(), &expected);
            assert_eq!(
                effect.calls.load(Ordering::Relaxed),
                before + demand.visits()
            );
        }
    }
}

#[derive(Debug)]
struct CurrentConfiguration {
    unknowns: Vec<Unknown>,
    options: Options,
    start: Arc<AtomicU64>,
    refuse: Arc<AtomicBool>,
    calls: Arc<AtomicUsize>,
}
impl HintResolver for CurrentConfiguration {
    fn identity(&self) -> ContentHash {
        ContentHash::from_bytes([47; 32])
    }
    fn retained_bytes(&self) -> usize {
        size_of::<Self>()
    }
    fn time_limit(&self) -> Duration {
        self.options.time_limit
    }
    fn resolve(
        &self,
        _: &[f64],
        _: Option<&[f64]>,
        _: Option<&[f64]>,
    ) -> Result<(Vec<Unknown>, Options), MathError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        if self.refuse.load(Ordering::Acquire) {
            return Err(MathError::Limit("current configuration allowance"));
        }
        let mut options = self.options.clone();
        options.start[0] = f64::from_bits(self.start.load(Ordering::Acquire));
        Ok((self.unknowns.clone(), options))
    }
}

#[test]
fn case_worker_unchanged_key_configuration_change_refusal_and_recovery() {
    for demand in [
        Demand::Value,
        Demand::First,
        Demand::Weighted,
        Demand::HessianAction,
        Demand::JacobianAction,
    ] {
        let control = pse_diagnostics::Cancellation::default();
        let (mut factory, work) = factory(&control);
        let Configuration::Fixed(unknowns, options) = factory.configuration.clone() else {
            panic!("fixed fixture");
        };
        let resolver = Arc::new(CurrentConfiguration {
            unknowns,
            options,
            start: Arc::new(AtomicU64::new(0.9_f64.to_bits())),
            refuse: Arc::new(AtomicBool::new(false)),
            calls: Arc::new(AtomicUsize::new(0)),
        });
        factory.configuration = Configuration::Hints(resolver.clone());
        let (assembly, values) = case(&factory);
        let expected = demand.expected(&assembly, &factory.spec);
        let mut worker = assembly.worker(
            [(factory.spec.key(), factory.create().unwrap())]
                .into_iter()
                .collect(),
            control.clone(),
        );
        for _ in 0..2 {
            close(&demand.evaluate(&mut worker, &values).unwrap(), &expected);
        }
        assert_eq!(resolver.calls.load(Ordering::Relaxed), 2 * demand.visits());
        assert_eq!(work.solves.load(Ordering::Relaxed), 1);
        resolver.start.store(0.8_f64.to_bits(), Ordering::Release);
        close(&demand.evaluate(&mut worker, &values).unwrap(), &expected);
        assert_eq!(resolver.calls.load(Ordering::Relaxed), 3 * demand.visits());
        assert_eq!(work.solves.load(Ordering::Relaxed), 2);
        resolver.refuse.store(true, Ordering::Release);
        let error = demand.evaluate(&mut worker, &values).unwrap_err();
        let MathError::Instance { cause, .. } = error else {
            panic!("instance attribution");
        };
        assert!(matches!(
            *cause,
            MathError::Provider {
                cause: pse_kernels::ProviderError::Limit("current configuration allowance"),
                ..
            }
        ));
        resolver.refuse.store(false, Ordering::Release);
        close(&demand.evaluate(&mut worker, &values).unwrap(), &expected);
        assert_eq!(
            resolver.calls.load(Ordering::Relaxed),
            4 * demand.visits() + 1,
            "resident completion cannot authorize bypassing current configuration"
        );
    }
}

#[test]
fn case_worker_operational_order_upgrades_and_cross_demand_keep_current_effects() {
    for restriction in [false, true] {
        let control = pse_diagnostics::Cancellation::default();
        let (factory, _, effect) = operational_factory(&control, restriction);
        let (assembly, values) = case(&factory);
        let mut worker = assembly.worker(
            [(factory.spec.key(), factory.create().unwrap())]
                .into_iter()
                .collect(),
            control,
        );
        close(worker.constraints(&values).unwrap(), &[0.8]);
        worker.jacobian(&values).unwrap();
        assert_eq!(
            effect.calls.load(Ordering::Relaxed),
            2,
            "Value to First upgrade visits selection"
        );
        worker.objective(&values).unwrap();
        assert_eq!(
            effect.calls.load(Ordering::Relaxed),
            3,
            "provider cross-demand projection remains excluded"
        );
        effect.refuse.store(true, Ordering::Release);
        assert_trial(&worker.hessian(&values, 2., &[3.]).unwrap_err());
        effect.refuse.store(false, Ordering::Release);
        worker.hessian(&values, 2., &[3.]).unwrap();
        assert_eq!(
            effect.calls.load(Ordering::Relaxed),
            5,
            "Second upgrade refusal then recovery both visit selection"
        );
        effect.refuse.store(true, Ordering::Release);
        assert_trial(&worker.constraints(&values).unwrap_err());
        effect.refuse.store(false, Ordering::Release);
        close(worker.constraints(&values).unwrap(), &[0.8]);
        assert_eq!(effect.calls.load(Ordering::Relaxed), 7);
    }
}
