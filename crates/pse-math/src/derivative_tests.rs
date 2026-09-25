// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use crate::{
    MathError,
    guarded::*,
    jets::EvaluationLimits,
    library::{self, Optimization},
};
use pse_ids::{ContentHash, SemanticId};
use pse_kernels::*;
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};
use symbolica::atom::{Atom, AtomCore};
fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
fn compile(
    inputs: usize,
    slots: usize,
    stages: Vec<Stage>,
    outputs: Vec<usize>,
    order: DerivativeOrder,
) -> CompiledBody {
    let prepared =
        PreparedBody::new(inputs, slots, outputs, stages, DerivativeOrder::Second).unwrap();
    prepared
        .compile(
            &(0..prepared.output_count()).collect::<Vec<_>>(),
            &(0..inputs).collect::<Vec<_>>(),
            order,
            Optimization::default(),
            EvaluationLimits::default(),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
}
fn block(expressions: Vec<Atom>, outputs: Vec<usize>) -> Stage {
    Stage::Block {
        expressions,
        outputs,
        source: id(1),
    }
}

#[test]
fn numerica_raw_diagonal_mixed_and_multi_output_derivatives() {
    crate::initialize().unwrap();
    let x = library::formal(0).unwrap();
    let y = library::formal(1).unwrap();
    let body = compile(
        2,
        4,
        vec![block(vec![&x * &x * &y, x.sin()], vec![2, 3])],
        vec![2, 3],
        DerivativeOrder::Second,
    );
    let cancel = Arc::new(AtomicBool::new(false));
    let mut worker = body.worker();
    let mut providers = BTreeMap::new();
    let v = worker
        .evaluate(
            &[2.0, 3.0],
            DerivativeOrder::Second,
            &mut providers,
            &cancel,
        )
        .unwrap();
    assert_eq!(v.values[0], 12.0);
    assert_eq!(&v.jacobian[..2], &[12.0, 4.0]);
    assert_eq!(&v.hessians[..4], &[6.0, 4.0, 4.0, 0.0]);
    assert!((v.hessians[4] + 2.0_f64.sin()).abs() < 1e-14);
    let v = worker
        .evaluate(&[2.0, 3.0], DerivativeOrder::Value, &mut providers, &cancel)
        .unwrap();
    assert!(v.jacobian.is_empty() && v.hessians.is_empty());
    assert_eq!(
        body.support().second[0],
        [(0, 0), (0, 1)].into_iter().collect()
    );
}

#[derive(Debug)]
struct Cubic {
    spec: ProviderSpec,
    calls: Arc<AtomicUsize>,
}
impl Provider for Cubic {
    fn spec(&self) -> &ProviderSpec {
        &self.spec
    }
    fn evaluate(
        &mut self,
        inputs: &[f64],
        request: &ProviderRequest,
        context: &EvaluationContext<'_>,
    ) -> Result<ProviderValues, ProviderError> {
        request.validate(&self.spec, context)?;
        self.calls.fetch_add(1, Ordering::Relaxed);
        if inputs[0] <= 0.0 {
            return Err(ProviderError::Trial("positive cubic input".into()));
        }
        let x = inputs[0];
        Ok(ProviderValues {
            values: request.outputs.iter().map(|_| x * x * x).collect(),
            jacobian: if request.order >= DerivativeOrder::First {
                vec![3.0 * x * x; request.outputs.len()]
            } else {
                vec![]
            },
            hessians: if request.order >= DerivativeOrder::Second {
                vec![6.0 * x; request.outputs.len()]
            } else {
                vec![]
            },
        })
    }
}
fn provider() -> (ProviderSpec, Box<dyn Provider>, Arc<AtomicUsize>) {
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let q = pse_quantity::standard::ids::quantity("neutral");
    let u = registry.quantity_type(q).unwrap().canonical_unit;
    let spec = ProviderSpec {
        envelope: None,
        id: id(10),
        revision: ContentHash::from_bytes([1; 32]),
        data: ContentHash::from_bytes([2; 32]),
        components: vec![],
        phase: Phase {
            id: id(11),
            revision: ContentHash::from_bytes([3; 32]),
        },
        inputs: vec![Port {
            id: id(12),
            quantity: q,
            unit: u,
        }],
        outputs: vec![Port {
            id: id(13),
            quantity: q,
            unit: u,
        }],
        derivatives: DerivativeOrder::Second,
        smoothness: DerivativeOrder::Second,
    };
    let calls = Arc::new(AtomicUsize::new(0));
    (
        spec.clone(),
        Box::new(Cubic {
            spec,
            calls: calls.clone(),
        }),
        calls,
    )
}
#[test]
fn symbolica_composes_provider_partials_after_arithmetic_barriers() {
    crate::initialize().unwrap();
    let (spec, provider, calls) = provider();
    let x = library::formal(0).unwrap();
    let stages = vec![
        block(vec![&x * &x], vec![1]),
        Stage::Provider {
            spec: spec.clone(),
            inputs: vec![1],
            outputs: vec![2],
            source: id(21),
        },
        block(vec![library::formal(2).unwrap() + &x], vec![3]),
    ];
    let body = compile(1, 4, stages, vec![3], DerivativeOrder::Second);
    let mut worker = body.worker();
    let cancel = Arc::new(AtomicBool::new(false));
    let mut providers = BTreeMap::from([(spec.key(), provider)]);
    let v = worker
        .evaluate(&[2.0], DerivativeOrder::Second, &mut providers, &cancel)
        .unwrap();
    assert_eq!(v.values, vec![66.0]);
    assert_eq!(v.jacobian, vec![193.0]);
    assert_eq!(v.hessians, vec![480.0]);
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert!(
        matches!(worker.evaluate(&[0.0],DerivativeOrder::Second,&mut providers,&cancel),Err(MathError::Provider{source_id,..}) if source_id==id(21))
    );
    let mut other = body.worker();
    assert_eq!(
        other
            .evaluate(&[1.0], DerivativeOrder::Second, &mut providers, &cancel)
            .unwrap()
            .hessians,
        vec![30.0]
    );
    assert_eq!(
        worker
            .evaluate(&[2.0], DerivativeOrder::Second, &mut providers, &cancel)
            .unwrap()
            .hessians,
        vec![480.0]
    );
}

#[test]
fn derivative_budget_cancellation_and_numeric_zero_support() {
    crate::initialize().unwrap();
    let x = library::formal(0).unwrap();
    let p = PreparedBody::new(
        1,
        2,
        vec![1],
        vec![block(vec![&x * &x * &x], vec![1])],
        DerivativeOrder::Second,
    )
    .unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let budget = EvaluationLimits {
        derivative_components: 2,
        ..EvaluationLimits::default()
    };
    assert!(matches!(
        p.compile(
            &[0],
            &[0],
            DerivativeOrder::Second,
            Optimization::default(),
            budget,
            &cancel
        ),
        Err(MathError::Limit(_))
    ));
    let body = p
        .compile(
            &[0],
            &[0],
            DerivativeOrder::Second,
            Optimization::default(),
            EvaluationLimits::default(),
            &cancel,
        )
        .unwrap();
    let mut worker = body.worker();
    assert_eq!(
        worker
            .evaluate(
                &[0.0],
                DerivativeOrder::Second,
                &mut BTreeMap::new(),
                &cancel
            )
            .unwrap()
            .hessians,
        vec![0.0]
    );
    assert!(body.support().second[0].contains(&(0, 0)));
    cancel.store(true, Ordering::Relaxed);
    assert!(matches!(
        worker.evaluate(
            &[2.0],
            DerivativeOrder::Value,
            &mut BTreeMap::new(),
            &cancel
        ),
        Err(MathError::Cancelled)
    ));
}

#[test]
fn parameter_only_switches_preserve_all_branch_support() {
    crate::initialize().unwrap();
    let x = library::formal(0).unwrap();
    let p = library::formal(1).unwrap();
    let body = PreparedBody::new(
        2,
        4,
        vec![3],
        vec![
            block(vec![Atom::num(0)], vec![2]),
            Stage::Branch {
                comparison: Comparison::Lt,
                left: 2,
                right: 1,
                then: vec![block(vec![&x * &x], vec![3])],
                otherwise: vec![block(vec![&x * &x * &x], vec![3])],
            },
        ],
        DerivativeOrder::Second,
    )
    .unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    assert!(
        body.compile(
            &[0],
            &[0, 1],
            DerivativeOrder::First,
            Optimization::default(),
            EvaluationLimits::default(),
            &cancel
        )
        .is_err()
    );
    let mut worker = body
        .compile(
            &[0],
            &[0],
            DerivativeOrder::Second,
            Optimization::default(),
            EvaluationLimits::default(),
            &cancel,
        )
        .unwrap()
        .worker();
    let a = worker
        .evaluate(
            &[2.0, 1.0],
            DerivativeOrder::Second,
            &mut BTreeMap::new(),
            &cancel,
        )
        .unwrap();
    assert_eq!(a.hessians, vec![2.0]);
    let b = worker
        .evaluate(
            &[2.0, -1.0],
            DerivativeOrder::Second,
            &mut BTreeMap::new(),
            &cancel,
        )
        .unwrap();
    assert_eq!(b.hessians, vec![12.0]);
    assert!(body.support().controls.contains(&1));
    assert!(!body.support().first[0].contains(&1));
    assert_eq!(body.support().second[0], [(0, 0)].into_iter().collect());
    let _ = p;
}
#[test]
fn multi_input_provider_lift_preserves_mixed_raw_partials() {
    crate::initialize().unwrap();
    #[derive(Debug)]
    struct Product {
        spec: ProviderSpec,
    }
    impl Provider for Product {
        fn spec(&self) -> &ProviderSpec {
            &self.spec
        }
        fn evaluate(
            &mut self,
            x: &[f64],
            r: &ProviderRequest,
            c: &EvaluationContext<'_>,
        ) -> Result<ProviderValues, ProviderError> {
            r.validate(&self.spec, c)?;
            Ok(ProviderValues {
                values: vec![x[0] * x[1]],
                jacobian: if r.order >= DerivativeOrder::First {
                    vec![x[1], x[0]]
                } else {
                    vec![]
                },
                hessians: if r.order >= DerivativeOrder::Second {
                    vec![0.0, 1.0, 1.0, 0.0]
                } else {
                    vec![]
                },
            })
        }
    }
    let (mut spec, _, _) = provider();
    let mut second = spec.inputs[0].clone();
    second.id = id(77);
    spec.inputs.push(second);
    let x = library::formal(0).unwrap();
    let y = library::formal(1).unwrap();
    let key = spec.key();
    let body = compile(
        2,
        5,
        vec![
            block(vec![&x * &x, &x * &y], vec![2, 3]),
            Stage::Provider {
                spec: spec.clone(),
                inputs: vec![2, 3],
                outputs: vec![4],
                source: id(9),
            },
        ],
        vec![4],
        DerivativeOrder::Second,
    );
    let provider: Box<dyn Provider> = Box::new(Product { spec });
    let mut providers: BTreeMap<_, Box<dyn Provider>> = BTreeMap::from([(key, provider)]);
    let v = body
        .worker()
        .evaluate(
            &[2.0, 3.0],
            DerivativeOrder::Second,
            &mut providers,
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert_eq!(v.values, vec![24.0]);
    assert_eq!(v.jacobian, vec![36.0, 8.0]);
    assert_eq!(v.hessians, vec![36.0, 12.0, 12.0, 0.0]);
}
#[test]
fn typed_output_demand_coalesces_calls_and_keeps_canceled_obligations() {
    crate::initialize().unwrap();
    use crate::typed::{Binary, BodyBuilder, BodyLimits};
    use pse_quantity::{
        IndexSet,
        standard::{StandardInvariantChecker, ids, standard_registry},
    };
    #[derive(Debug)]
    struct Factory {
        spec: ProviderSpec,
        requests: Arc<std::sync::Mutex<Vec<ProviderRequest>>>,
    }
    #[derive(Debug)]
    struct Properties {
        spec: ProviderSpec,
        requests: Arc<std::sync::Mutex<Vec<ProviderRequest>>>,
    }
    impl Provider for Properties {
        fn spec(&self) -> &ProviderSpec {
            &self.spec
        }
        fn evaluate(
            &mut self,
            x: &[f64],
            r: &ProviderRequest,
            c: &EvaluationContext<'_>,
        ) -> Result<ProviderValues, ProviderError> {
            r.validate(&self.spec, c)?;
            self.requests.lock().unwrap().push(r.clone());
            if x[0] <= 0.0 {
                return Err(ProviderError::Trial("positive property input".into()));
            }
            let mut result = ProviderValues {
                values: vec![],
                jacobian: vec![],
                hessians: vec![],
            };
            for &o in &r.outputs {
                result
                    .values
                    .push(if o == 0 { x[0] * x[0] } else { x[0].ln() });
                if r.order >= DerivativeOrder::First {
                    result
                        .jacobian
                        .push(if o == 0 { 2.0 * x[0] } else { 1.0 / x[0] });
                }
                if r.order >= DerivativeOrder::Second {
                    result
                        .hessians
                        .push(if o == 0 { 2.0 } else { -1.0 / (x[0] * x[0]) });
                }
            }
            Ok(result)
        }
    }
    impl ProviderFactory for Factory {
        fn spec(&self) -> &ProviderSpec {
            &self.spec
        }
        fn create(&self) -> Result<Box<dyn Provider>, ProviderError> {
            Ok(Box::new(Properties {
                spec: self.spec.clone(),
                requests: self.requests.clone(),
            }))
        }
    }
    let registry = standard_registry().unwrap();
    let (mut spec, _, _) = provider();
    let mut port = spec.outputs[0].clone();
    port.id = id(88);
    spec.outputs.push(port);
    let key = spec.key();
    let requests = Arc::new(std::sync::Mutex::new(vec![]));
    let registration = Registration::new(
        Arc::new(Factory {
            spec,
            requests: requests.clone(),
        }),
        &registry,
    )
    .unwrap();
    let mut b = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &StandardInvariantChecker,
        1,
        BodyLimits::default(),
    )
    .unwrap();
    let x = b
        .input(0, ids::quantity("neutral"), IndexSet::new(), id(1))
        .unwrap();
    let first = b
        .provider(&registration.descriptor(), std::slice::from_ref(&x), id(2))
        .unwrap();
    let second = b
        .provider(&registration.descriptor(), std::slice::from_ref(&x), id(3))
        .unwrap();
    let canceled = b
        .binary(Binary::Sub, first[0].clone(), first[0].clone(), None, id(4))
        .unwrap();
    let prepared = b.prepare(&[x, canceled, second[1].clone()]).unwrap();
    assert_eq!(prepared.providers().len(), 1);
    let cancel = Arc::new(AtomicBool::new(false));
    let mut workers = BTreeMap::from([(key, registration.worker().unwrap())]);
    let compile = |outputs: &[usize]| {
        prepared
            .compile(
                outputs,
                &[0],
                DerivativeOrder::Second,
                Optimization::default(),
                EvaluationLimits::default(),
                &cancel,
            )
            .unwrap()
            .worker()
    };
    assert_eq!(
        compile(&[0])
            .evaluate(&[-2.0], DerivativeOrder::Value, &mut workers, &cancel)
            .unwrap()
            .values,
        vec![-2.0]
    );
    assert!(requests.lock().unwrap().is_empty());
    assert!(
        compile(&[1])
            .evaluate(&[-2.0], DerivativeOrder::Value, &mut workers, &cancel)
            .is_err()
    );
    requests.lock().unwrap().clear();
    let result = compile(&[1, 2])
        .evaluate(&[2.0], DerivativeOrder::Second, &mut workers, &cancel)
        .unwrap();
    assert_eq!(result.values, vec![0.0, 2.0_f64.ln()]);
    assert_eq!(result.hessians, vec![0.0, -0.25]);
    assert_eq!(requests.lock().unwrap().len(), 1);
    assert_eq!(requests.lock().unwrap()[0].outputs, vec![0, 1]);
    requests.lock().unwrap().clear();
    compile(&[2])
        .evaluate(&[2.0], DerivativeOrder::Value, &mut workers, &cancel)
        .unwrap();
    assert_eq!(
        requests.lock().unwrap()[0],
        ProviderRequest {
            outputs: vec![1],
            order: DerivativeOrder::Value
        }
    );
    use crate::assembly::{AssemblyLimits, CasePlan};
    use crate::binding::*;
    let quantity = ids::quantity("neutral");
    let port = Port {
        id: id(1),
        quantity,
        unit: registry.quantity_type(quantity).unwrap().canonical_unit,
    };
    let body_key = ContentHash::from_bytes([9; 32]);
    let structure = Arc::new(
        CaseStructure::new(
            vec![Variable {
                port: port.clone(),
                fixed: false,
                domain: VariableDomain::Continuous,
                lower: Some(0.1),
                upper: Some(10.0),
            }],
            vec![],
            vec![InstanceBinding {
                instance: id(5),
                body: body_key,
                slots: vec![SlotBinding::new(&port, &port, &registry).unwrap()],
                contributions: vec![
                    Contribution {
                        output: 2,
                        target: Target::Objective,
                        scale: 1.0,
                    },
                    Contribution {
                        output: 1,
                        target: Target::Row(id(6)),
                        scale: 1.0,
                    },
                ],
            }],
            vec![Row {
                id: id(6),
                quantity,
                lower: 0.0,
                upper: 0.0,
            }],
            Some(Objective {
                quantity,
                sense: ObjectiveSense::Minimize,
            }),
            CaseLimits::default(),
        )
        .unwrap(),
    );
    let case = Arc::new(
        Arc::new(
            CasePlan::prepare(
                structure,
                BTreeMap::from([(body_key, Arc::new(prepared))]),
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
    let mut worker = case.worker(workers, cancel);
    let values = CaseValues {
        scalars: BTreeMap::from([(id(1), 2.0)]),
    };
    requests.lock().unwrap().clear();
    assert_eq!(worker.hessian(&values, 2.0, &[3.0]).unwrap().val(), &[-0.5]);
    assert_eq!(worker.hessian(&values, 4.0, &[9.0]).unwrap().val(), &[-1.0]);
    assert_eq!(requests.lock().unwrap().len(), 1);
    let bad = CaseValues {
        scalars: BTreeMap::from([(id(1), -2.0)]),
    };
    let error = worker.hessian(&bad, 1.0, &[0.0]).unwrap_err();
    assert!(
        matches!(error,MathError::Instance{instance,cause} if instance==id(5) && matches!(*cause,MathError::Provider{source_id,..} if source_id==id(2)))
    );
}
