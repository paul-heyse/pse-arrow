// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use pse_kernels::Port;
use pse_math::binding::{
    Contribution, InstanceBinding, Objective, ObjectiveSense, Row, SlotBinding, Variable,
    VariableDomain,
};
fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
fn inputs() -> Inputs {
    let quantities = Arc::new(pse_quantity::standard::standard_registry().unwrap());
    let q = pse_quantity::standard::ids::quantity("neutral");
    let unit = quantities.quantity_type(q).unwrap().canonical_unit;
    let port = |n| Port {
        id: id(n),
        quantity: q,
        unit,
    };
    let definition = Definition {
        sources: vec!["x*p".into(), "x*x".into()],
        formals: vec![
            Formal {
                path: "x".into(),
                quantity: q,
            },
            Formal {
                path: "p".into(),
                quantity: q,
            },
        ],
        domains: vec![],
        groups: vec![],
        providers: vec![],
        units: BTreeMap::from([("1".into(), unit)]),
        literals: BTreeMap::new(),

        limits: BodyLimits::default(),
    };
    let structure = CaseStructure::new(
        vec![Variable {
            port: port(1),
            fixed: false,
            domain: VariableDomain::Continuous,
            lower: None,
            upper: None,
        }],
        vec![port(2)],
        vec![InstanceBinding {
            instance: id(8),
            body: ContentHash::from_bytes([0; 32]),
            slots: vec![
                SlotBinding::new(&port(1), &port(1), &quantities).unwrap(),
                SlotBinding::new(&port(2), &port(2), &quantities).unwrap(),
            ],
            contributions: vec![
                Contribution {
                    output: 0,
                    target: Target::Row(id(5)),
                    scale: 1.,
                },
                Contribution {
                    output: 1,
                    target: Target::Objective,
                    scale: 1.,
                },
            ],
        }],
        vec![Row {
            id: id(5),
            quantity: q,
            lower: 0.,
            upper: 0.,
        }],
        Some(Objective {
            quantity: q,
            sense: ObjectiveSense::Minimize,
        }),
        CaseLimits::default(),
    )
    .unwrap();
    Inputs {
        flows: BTreeMap::new(),
        quantities,
        preconditions: Arc::new(PhysicalPreconditions::new(vec![]).unwrap()),
        definitions: BTreeMap::from([(id(10), definition)]),
        domains: BTreeMap::new(),
        groups: BTreeMap::new(),
        providers: BTreeMap::new(),
        cases: BTreeMap::from([(
            id(9),
            Case {
                structure: Arc::new(structure),
                definitions: BTreeMap::from([(id(8), id(10))]),
            },
        )]),
        values: BTreeMap::from([(id(1), 2.), (id(2), 3.)]),
    }
}
#[test]
fn generous_workspace_preserves_retention_and_refuses_tiny_budget() {
    let mut source = inputs();
    let mut workspace = CompilerWorkspace::new(source.clone(), WorkspaceLimits::default()).unwrap();
    assert_eq!(workspace.limits.query_values, 64);
    let definition = source.definitions[&id(10)].clone();
    for n in 50..150 {
        source.definitions.insert(id(n), definition.clone());
    }
    workspace.publish(source.clone()).unwrap();
    assert_eq!(workspace.limits.query_values, 64);
    assert_eq!(workspace.generation, 0);
    // Published values still produce the same evaluator as a clean bounded workspace.
    let profile = Profile::default();
    let reused = workspace
        .prepare(id(9), DerivativeOrder::Second, profile, false)
        .unwrap();
    let clean = CompilerWorkspace::new(source, WorkspaceLimits::default())
        .unwrap()
        .prepare(id(9), DerivativeOrder::Second, profile, false)
        .unwrap();
    assert_eq!(reused.artifacts, clean.artifacts);
    let limits = WorkspaceLimits {
        input_bytes: 1,
        ..Default::default()
    };
    assert!(CompilerWorkspace::new(inputs(), limits).is_err());
}
fn prepare(w: &mut CompilerWorkspace) -> PreparedCase {
    w.prepare(id(9), DerivativeOrder::Second, Profile::default(), true)
        .unwrap()
}
fn compare_clean(w: &mut CompilerWorkspace, i: &Inputs) -> PreparedCase {
    let a = prepare(w);
    let mut clean = CompilerWorkspace::new(i.clone(), WorkspaceLimits::default()).unwrap();
    let b = prepare(&mut clean);
    assert_eq!(a.structure, b.structure);
    assert_eq!(a.plan.structure(), b.plan.structure());
    assert_eq!(a.artifacts, b.artifacts);
    assert_eq!(a.occurrences, b.occurrences);
    assert_eq!(
        a.coefficients.as_ref().unwrap().assumptions,
        b.coefficients.as_ref().unwrap().assumptions
    );
    a
}
#[test]
fn reuse_matches_clean_values_parameters_source_profiles_and_unrelated_edits() {
    let mut i = inputs();
    let mut w = CompilerWorkspace::new(i.clone(), WorkspaceLimits::default()).unwrap();
    let a = compare_clean(&mut w, &i);
    i.values.insert(id(1), 1234.);
    w.publish(i.clone()).unwrap();
    let b = compare_clean(&mut w, &i);
    assert!(Arc::ptr_eq(&a.plan, &b.plan));
    assert!(Arc::ptr_eq(
        &a.coefficients.unwrap(),
        b.coefficients.as_ref().unwrap()
    ));
    i.values.insert(id(2), 7.);
    w.publish(i.clone()).unwrap();
    let c = compare_clean(&mut w, &i);
    assert!(Arc::ptr_eq(&b.plan, &c.plan));
    assert_eq!(b.artifacts, c.artifacts);
    assert_ne!(
        b.coefficients.unwrap().assumptions,
        c.coefficients.as_ref().unwrap().assumptions
    );
    i.definitions.get_mut(&id(10)).unwrap().sources[0] = " x * p ".into();
    w.publish(i.clone()).unwrap();
    let d = compare_clean(&mut w, &i);
    assert!(Arc::ptr_eq(&c.plan, &d.plan));
    assert_eq!(c.artifacts, d.artifacts);
    assert_ne!(c.occurrences, d.occurrences);
    i.definitions.insert(id(50), i.definitions[&id(10)].clone());
    w.publish(i.clone()).unwrap();
    let e = compare_clean(&mut w, &i);
    assert!(Arc::ptr_eq(&d.plan, &e.plan));
    let changed = w
        .prepare(
            id(9),
            DerivativeOrder::Second,
            Profile {
                optimization: Optimization {
                    cores: 2,
                    ..Optimization::default()
                },
                ..Profile::default()
            },
            false,
        )
        .unwrap();
    assert!(Arc::ptr_eq(&e.plan, &changed.plan));
    assert_ne!(e.artifacts[0].key(), changed.artifacts[0].key());
    // Invalid evaluator options do not affect graph preparation: no evaluator is built.
    assert!(
        w.prepare(
            id(9),
            DerivativeOrder::Value,
            Profile {
                optimization: Optimization {
                    cores: 0,
                    ..Optimization::default()
                },
                ..Profile::default()
            },
            false
        )
        .is_ok()
    );
}
#[test]
fn absent_empty_membership_and_removed_definitions_are_observed() {
    let mut i = inputs();
    i.definitions
        .get_mut(&id(10))
        .unwrap()
        .domains
        .push("components".into());
    let mut w = CompilerWorkspace::new(i.clone(), WorkspaceLimits::default()).unwrap();
    assert!(matches!(
        w.prepare(id(9), DerivativeOrder::Value, Profile::default(), false),
        Err(CompileError::Missing(_))
    ));
    i.domains.insert(
        "components".into(),
        Domain {
            members: pse_math::binding::FiniteDomain::new(id(70), vec![], 10).unwrap(),
            kind: pse_quantity::DomainKind::Species,
        },
    );
    w.publish(i.clone()).unwrap();
    assert!(
        w.prepare(id(9), DerivativeOrder::Value, Profile::default(), false)
            .is_ok()
    );
    i.domains.remove("components");
    w.publish(i.clone()).unwrap();
    assert!(
        w.prepare(id(9), DerivativeOrder::Value, Profile::default(), false)
            .is_err()
    );
    i.domains.insert(
        "components".into(),
        Domain {
            members: pse_math::binding::FiniteDomain::new(id(70), vec![id(71)], 10).unwrap(),
            kind: pse_quantity::DomainKind::Species,
        },
    );
    w.publish(i.clone()).unwrap();
    assert!(
        w.prepare(id(9), DerivativeOrder::Value, Profile::default(), false)
            .is_ok()
    );
    let def = i.definitions.remove(&id(10)).unwrap();
    w.publish(i.clone()).unwrap();
    assert!(
        w.prepare(id(9), DerivativeOrder::Value, Profile::default(), false)
            .is_err()
    );
    i.definitions.insert(id(10), def);
    w.publish(i).unwrap();
    assert!(
        w.prepare(id(9), DerivativeOrder::Value, Profile::default(), false)
            .is_ok()
    );
}
#[test]
fn fixedness_bounds_aliases_and_physical_edits_reprepare() {
    let mut i = inputs();
    let mut w = CompilerWorkspace::new(i.clone(), WorkspaceLimits::default()).unwrap();
    let before = prepare(&mut w);
    let s = &i.cases[&id(9)].structure;
    let mut variables = s.variables().to_vec();
    variables[0].fixed = true;
    i.cases.get_mut(&id(9)).unwrap().structure = Arc::new(
        CaseStructure::new(
            variables,
            s.parameters().to_vec(),
            s.instances().to_vec(),
            s.rows().to_vec(),
            s.objective().cloned(),
            CaseLimits::default(),
        )
        .unwrap(),
    );
    w.publish(i.clone()).unwrap();
    let after = compare_clean(&mut w, &i);
    assert!(after.plan.columns().is_empty());
    assert_ne!(before.artifacts, after.artifacts);
    let mut i = inputs();
    let s = &i.cases[&id(9)].structure;
    let mut rows = s.rows().to_vec();
    rows[0].upper = 5.;
    i.cases.get_mut(&id(9)).unwrap().structure = Arc::new(
        CaseStructure::new(
            s.variables().to_vec(),
            s.parameters().to_vec(),
            s.instances().to_vec(),
            rows,
            s.objective().cloned(),
            CaseLimits::default(),
        )
        .unwrap(),
    );
    w.publish(i.clone()).unwrap();
    assert!(compare_clean(&mut w, &i).structure.matching.is_empty());
    let mut builder = i.quantities.to_builder();
    let mut unit = i.quantities.units().next().unwrap().clone();
    unit.symbol.push_str("_changed");
    unit.id = UnitId::from_id(id(98));
    builder.unit(unit);
    i.quantities = Arc::new(builder.build().unwrap());
    w.publish(i.clone()).unwrap();
    compare_clean(&mut w, &i);
}
#[test]
fn finite_generations_cancel_retry_and_invalid_batch_are_atomic() {
    let mut i = inputs();
    let mut w = CompilerWorkspace::new(
        i.clone(),
        WorkspaceLimits {
            revisions: 1,
            preparations: 2,
            ..WorkspaceLimits::default()
        },
    )
    .unwrap();
    let old = prepare(&mut w);
    for n in 0..6 {
        i.values.insert(id(2), f64::from(n) + 1.);
        w.publish(i.clone()).unwrap();
        prepare(&mut w);
    }
    assert!(w.generation() >= 2);
    assert_eq!(old.plan.columns(), &[id(1)]);
    let mut bad = i.clone();
    bad.values.insert(id(2), f64::NAN);
    assert!(w.publish(bad).is_err());
    compare_clean(&mut w, &i);
    let token = w.cancellation_token();
    token.cancel();
    assert!(matches!(
        w.prepare(id(9), DerivativeOrder::Value, Profile::default(), false),
        Err(CompileError::Cancelled)
    ));
    assert!(
        w.prepare(id(9), DerivativeOrder::Value, Profile::default(), false)
            .is_ok()
    );
    let cancelled = Arc::new(AtomicBool::new(true));
    assert!(matches!(
        w.prepare_cancellable(
            id(9),
            DerivativeOrder::Value,
            Profile::default(),
            false,
            cancelled
        ),
        Err(CompileError::Cancelled)
    ));
    assert!(
        w.prepare(id(9), DerivativeOrder::Value, Profile::default(), false)
            .is_ok()
    );
}

#[test]
fn event_backed_query_reuse_and_metadata_reconstruction() {
    let i = inputs();
    let events = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let observed = events.clone();
    let mut w = CompilerWorkspace::with_events(
        i.clone(),
        WorkspaceLimits::default(),
        Some(Box::new(move |e| {
            if matches!(e.kind, salsa::EventKind::WillExecute { .. }) {
                observed.fetch_add(1, Ordering::Relaxed);
            }
        })),
    )
    .unwrap();
    let a = prepare(&mut w);
    let count = events.load(Ordering::Relaxed);
    assert!(count > 0);
    let b = prepare(&mut w);
    assert_eq!(events.load(Ordering::Relaxed), count);
    assert!(Arc::ptr_eq(&a.plan, &b.plan));
    assert!(w.metadata_bytes() > 0);
    let mut changed = i;
    changed
        .definitions
        .insert(id(50), changed.definitions[&id(10)].clone());
    w.publish(changed).unwrap();
    let c = prepare(&mut w);
    assert!(events.load(Ordering::Relaxed) > count);
    assert!(Arc::ptr_eq(&b.plan, &c.plan));
}

#[derive(Clone, Debug)]
struct Declared(pse_kernels::ProviderSpec);
impl pse_kernels::Provider for Declared {
    fn spec(&self) -> &pse_kernels::ProviderSpec {
        &self.0
    }
    fn evaluate(
        &mut self,
        _: &[f64],
        _: &pse_kernels::ProviderRequest,
        _: &pse_kernels::EvaluationContext<'_>,
    ) -> std::result::Result<pse_kernels::ProviderValues, pse_kernels::ProviderError> {
        Err(pse_kernels::ProviderError::Trial(
            "not executed during compiler preparation".into(),
        ))
    }
}
impl pse_kernels::ProviderFactory for Declared {
    fn spec(&self) -> &pse_kernels::ProviderSpec {
        &self.0
    }
    fn create(
        &self,
    ) -> std::result::Result<Box<dyn pse_kernels::Provider>, pse_kernels::ProviderError> {
        Ok(Box::new(self.clone()))
    }
}
#[test]
fn consumed_provider_revisions_invalidate_but_unrelated_providers_backdate() {
    let mut i = inputs();
    let port = i.cases[&id(9)].structure.variables()[0].port.clone();
    let h = ContentHash::from_bytes([0; 32]);
    let spec = pse_kernels::ProviderSpec {
        envelope: None,
        id: id(80),
        revision: h,
        data: h,
        components: vec![id(90)],
        phase: pse_kernels::Phase {
            id: id(81),
            revision: h,
        },
        inputs: vec![port.clone()],
        outputs: vec![port],
        derivatives: DerivativeOrder::Second,
        smoothness: DerivativeOrder::Second,
    };
    let call = |spec| ProviderCall {
        descriptor: pse_kernels::Registration::new(Arc::new(Declared(spec)), &i.quantities)
            .unwrap()
            .descriptor(),
        output: 0,
    };
    i.providers.insert("property".into(), call(spec.clone()));
    i.definitions
        .get_mut(&id(10))
        .unwrap()
        .providers
        .push("property".into());
    i.definitions.get_mut(&id(10)).unwrap().sources[0] = "kernel.property(x)".into();
    let mut w = CompilerWorkspace::new(i.clone(), WorkspaceLimits::default()).unwrap();
    let get = |w: &mut CompilerWorkspace| {
        w.prepare(id(9), DerivativeOrder::Second, Profile::default(), false)
            .unwrap()
    };
    let a = get(&mut w);
    i.providers.insert("unused".into(), call(spec.clone()));
    w.publish(i.clone()).unwrap();
    let b = get(&mut w);
    assert!(Arc::ptr_eq(&a.plan, &b.plan));
    for spec in [
        pse_kernels::ProviderSpec {
            data: ContentHash::from_bytes([1; 32]),
            ..spec.clone()
        },
        pse_kernels::ProviderSpec {
            phase: pse_kernels::Phase {
                id: id(82),
                revision: ContentHash::from_bytes([2; 32]),
            },
            ..spec.clone()
        },
        pse_kernels::ProviderSpec {
            components: vec![id(91), id(90)],
            ..spec.clone()
        },
        pse_kernels::ProviderSpec {
            derivatives: DerivativeOrder::First,
            ..spec
        },
    ] {
        let prior = get(&mut w);
        i.providers.insert("property".into(), call(spec));
        w.publish(i.clone()).unwrap();
        let reused = get(&mut w);
        let clean =
            get(&mut CompilerWorkspace::new(i.clone(), WorkspaceLimits::default()).unwrap());
        assert_eq!(reused.artifacts, clean.artifacts);
        assert_ne!(prior.artifacts, reused.artifacts);
    }
}
#[test]
fn finite_membership_and_ragged_bindings_match_clean_preparation() {
    let mut i = inputs();
    let scalar = i.definitions[&id(10)].formals[0].quantity;
    let mut shaped = i.quantities.quantity_type(scalar).unwrap().clone();
    shaped.key.shape = vec![pse_quantity::DomainKind::Species];
    shaped.id = QuantityTypeId::from_id(id(91));
    let shaped_id = i.quantities.resolve_key(&shaped.key).unwrap_or(shaped.id);
    if shaped_id == shaped.id {
        let mut builder = i.quantities.to_builder();
        builder.quantity_type(shaped);
        i.quantities = Arc::new(builder.build().unwrap());
    }
    let d = i.definitions.get_mut(&id(10)).unwrap();
    d.sources[0] = "sum(j in members | amounts[j])".into();
    d.domains = vec!["members".into()];
    d.groups = vec!["amounts".into()];
    i.groups.insert(
        "amounts".into(),
        Group {
            quantity: shaped_id,
            axes: vec!["members".into()],
            slots: BTreeMap::from([(vec![id(71)], 0), (vec![id(72)], 1)]),
        },
    );
    let domain = |members: Vec<SemanticId>| Domain {
        members: pse_math::binding::FiniteDomain::new(id(70), members, 10).unwrap(),
        kind: pse_quantity::DomainKind::Species,
    };
    i.domains.insert("members".into(), domain(vec![id(71)]));
    let mut w = CompilerWorkspace::new(i.clone(), WorkspaceLimits::default()).unwrap();
    let a = compare_clean(&mut w, &i);
    i.domains
        .insert("members".into(), domain(vec![id(71), id(72)]));
    w.publish(i.clone()).unwrap();
    let b = compare_clean(&mut w, &i);
    assert_ne!(a.artifacts, b.artifacts);
    i.groups
        .get_mut("amounts")
        .unwrap()
        .slots
        .remove(&vec![id(72)]);
    w.publish(i.clone()).unwrap();
    assert!(
        w.prepare(id(9), DerivativeOrder::Second, Profile::default(), false)
            .is_err()
    );
    i.domains.insert("members".into(), domain(vec![]));
    w.publish(i.clone()).unwrap();
    let empty = compare_clean(&mut w, &i);
    assert!(empty.structure.matching.is_empty());
}

#[test]
fn conditional_blocks_drop_objective_demands_and_reuse_pure_salsa_products() {
    let i = inputs();
    let mut w = CompilerWorkspace::new(i.clone(), WorkspaceLimits::default()).unwrap();
    let first = w
        .prepare_initialization_blocks(id(9), Profile::default())
        .unwrap();
    assert_eq!(first.len(), 1);
    assert!(first[0].plan.structure().objective().is_none());
    assert_eq!(first[0].plan.demands()[0].outputs, vec![0]);
    let again = w
        .prepare_initialization_blocks(id(9), Profile::default())
        .unwrap();
    assert!(Arc::ptr_eq(&first, &again));
    let mut next = i;
    next.values.insert(id(2), 42.0);
    w.publish(next).unwrap();
    let unchanged = w
        .prepare_initialization_blocks(id(9), Profile::default())
        .unwrap();
    assert!(Arc::ptr_eq(&first, &unchanged));
}
#[test]
fn flow_membership_absence_and_isolated_nodes_are_tracked() {
    use pse_structural::flowsheet::{Declaration, Node};
    let mut i = inputs();
    let mut w = CompilerWorkspace::new(i.clone(), WorkspaceLimits::default()).unwrap();
    assert!(w.prepare_flow(id(90)).is_err());
    i.flows.insert(
        id(90),
        Declaration {
            nodes: vec![Node {
                id: id(91),
                ports: vec![],
            }],
            connections: vec![],
            decisions: vec![],
        },
    );
    w.publish(i.clone()).unwrap();
    let first = w.prepare_flow(id(90)).unwrap();
    assert_eq!(first.components(), vec![vec![id(91)]]);
    assert!(Arc::ptr_eq(&first, &w.prepare_flow(id(90)).unwrap()));
    i.flows.get_mut(&id(90)).unwrap().nodes.push(Node {
        id: id(92),
        ports: vec![],
    });
    w.publish(i).unwrap();
    let next = w.prepare_flow(id(90)).unwrap();
    assert_eq!(next.components().len(), 2);
    assert_ne!(first.key(), next.key());
}
