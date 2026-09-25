// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use crate::{
    Function,
    binding::*,
    library::Optimization,
    typed::{Binary, BodyBuilder, BodyLimits},
};
use pse_ids::{ContentHash, SemanticId};
use pse_kernels::{DerivativeOrder, Port};
use pse_quantity::{
    IndexSet,
    standard::{StandardInvariantChecker, ids, standard_registry},
};
use std::sync::Arc;
use std::{collections::BTreeMap, sync::atomic::AtomicBool};
fn source() -> SemanticId {
    SemanticId::from_bytes([1; 16])
}

#[test]
fn physical_point_subtraction_precedes_normalization() {
    crate::initialize().unwrap();
    let registry = standard_registry().unwrap();
    let mut builder = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &StandardInvariantChecker,
        2,
        BodyLimits::default(),
    )
    .unwrap();
    let left = builder
        .input(
            0,
            ids::quantity("temperature.point"),
            IndexSet::new(),
            source(),
        )
        .unwrap();
    let right = builder
        .input(
            1,
            ids::quantity("temperature.point"),
            IndexSet::new(),
            source(),
        )
        .unwrap();
    assert!(
        builder
            .binary(Binary::Add, left.clone(), right.clone(), None, source())
            .is_err()
    );
    let difference = builder
        .binary(Binary::Sub, left, right, None, source())
        .unwrap();
    assert_eq!(difference.quantity, ids::quantity("temperature.difference"));
    let mut artifact = builder
        .finish(
            &[difference],
            DerivativeOrder::Second,
            Optimization::default(),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
        .worker();
    assert_eq!(
        artifact
            .evaluate(
                &[300.0, 290.0],
                DerivativeOrder::Value,
                &mut BTreeMap::new(),
                &Arc::new(AtomicBool::new(false))
            )
            .unwrap()
            .values,
        vec![10.0]
    );
}

#[test]
fn nested_domain_dependencies_execute_before_division_and_log() {
    crate::initialize().unwrap();
    let registry = standard_registry().unwrap();
    let mut builder = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &StandardInvariantChecker,
        2,
        BodyLimits::default(),
    )
    .unwrap();
    let one = builder
        .input(0, ids::quantity("neutral"), IndexSet::new(), source())
        .unwrap();
    let x = builder
        .input(1, ids::quantity("neutral"), IndexSet::new(), source())
        .unwrap();
    let inverse = builder.binary(Binary::Div, one, x, None, source()).unwrap();
    let logarithm = builder.unary(Function::Log, inverse, source()).unwrap();
    let mut artifact = builder
        .finish(
            &[logarithm],
            DerivativeOrder::Second,
            Optimization::default(),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
        .worker();
    for invalid in [0.0, -1.0] {
        assert!(
            artifact
                .evaluate(
                    &[1.0, invalid],
                    DerivativeOrder::Value,
                    &mut BTreeMap::new(),
                    &Arc::new(AtomicBool::new(false))
                )
                .is_err()
        );
    }
    let value = artifact
        .evaluate(
            &[1.0, 2.0],
            DerivativeOrder::Second,
            &mut BTreeMap::new(),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
        .values[0];
    assert!((value - 0.5_f64.ln()).abs() < 1e-14);
}

#[test]
fn datum_mismatch_is_not_a_representation_conversion() {
    crate::initialize().unwrap();
    let registry = standard_registry().unwrap();
    let absolute = Port {
        id: source(),
        quantity: ids::quantity("pressure.absolute"),
        unit: ids::unit("Pa"),
    };
    let gauge = Port {
        quantity: ids::quantity("pressure.gauge"),
        ..absolute.clone()
    };
    assert!(SlotBinding::new(&gauge, &absolute, &registry).is_err());
}

#[test]
fn semantic_domains_reject_duplicates_and_preserve_empty_sets() {
    crate::initialize().unwrap();
    let a = SemanticId::from_bytes([1; 16]);
    let b = SemanticId::from_bytes([2; 16]);
    assert!(FiniteDomain::new(a, vec![a, a], 2).is_err());
    assert_eq!(
        FiniteDomain::new(a, vec![b, a], 2).unwrap().members(),
        &[a, b]
    );
    assert!(
        FiniteDomain::new(a, vec![], 2)
            .unwrap()
            .members()
            .is_empty()
    );
    assert!(FiniteDomain::new(a, vec![], 0).is_err());
}

#[test]
fn body_identity_tracks_semantics_not_symbol_registration_or_values() {
    crate::initialize().unwrap();
    let h = ContentHash::from_bytes([1; 32]);
    let spec = BodySpec {
        definition: h,
        structure: h,
        physical: h,
        providers: vec![],
        policy: h,
    };
    let key = spec.key();
    let _ = crate::library::formal(100).unwrap();
    assert_eq!(key, spec.key());
    let mut changed = spec.clone();
    changed.structure = ContentHash::from_bytes([2; 32]);
    assert_ne!(key, changed.key());
    changed = spec.clone();
    changed.physical = ContentHash::from_bytes([2; 32]);
    assert_ne!(key, changed.key());
}

#[test]
fn aliases_and_affine_unit_bindings_preserve_body_reuse() {
    crate::initialize().unwrap();
    let registry = standard_registry().unwrap();
    let quantity = ids::quantity("temperature.point");
    let source_port = Port {
        id: source(),
        quantity,
        unit: ids::unit("degC"),
    };
    let target_port = Port {
        id: SemanticId::from_bytes([2; 16]),
        quantity,
        unit: ids::unit("K"),
    };
    let slot = SlotBinding::new(&source_port, &target_port, &registry).unwrap();
    let h = ContentHash::from_bytes([1; 32]);
    let instance = InstanceBinding {
        instance: SemanticId::from_bytes([3; 16]),
        body: h,
        slots: vec![slot.clone(), slot],
        contributions: vec![Contribution {
            output: 0,
            target: Target::Row(SemanticId::from_bytes([4; 16])),
            scale: 1.0,
        }],
    };
    let mut values = CaseValues {
        scalars: BTreeMap::from([(source(), 20.0)]),
    };
    assert_eq!(instance.values(&values).unwrap(), vec![293.15, 293.15]);
    values.scalars.insert(source(), 30.0);
    assert_eq!(instance.values(&values).unwrap(), vec![303.15, 303.15]);
    assert_eq!(instance.body, h);
    let mut variable = Variable {
        port: source_port,
        fixed: false,
        domain: VariableDomain::Continuous,
        lower: None,
        upper: None,
    };
    let free = CaseStructure::new(
        vec![variable.clone()],
        vec![],
        vec![instance.clone()],
        vec![Row {
            id: SemanticId::from_bytes([4; 16]),
            quantity,
            lower: 0.0,
            upper: 0.0,
        }],
        None,
        CaseLimits::default(),
    )
    .unwrap();
    assert_eq!(free.free_variables().collect::<Vec<_>>(), vec![source()]);
    variable.fixed = true;
    let fixed = CaseStructure::new(
        vec![variable],
        vec![],
        vec![instance.clone()],
        vec![Row {
            id: SemanticId::from_bytes([4; 16]),
            quantity,
            lower: 0.0,
            upper: 0.0,
        }],
        None,
        CaseLimits::default(),
    )
    .unwrap();
    assert_eq!(fixed.free_variables().count(), 0);
    assert_eq!(free.instances()[0].body, fixed.instances()[0].body);
    assert!(
        CaseStructure::new(
            vec![],
            vec![],
            vec![instance.clone(), instance],
            vec![],
            None,
            CaseLimits::default()
        )
        .is_err()
    );
}

#[test]
fn connection_equation_binds_distinct_units_before_point_subtraction() {
    crate::initialize().unwrap();
    let registry = standard_registry().unwrap();
    let quantity = ids::quantity("temperature.point");
    let hot = Port {
        id: source(),
        quantity,
        unit: ids::unit("degC"),
    };
    let cold = Port {
        id: SemanticId::from_bytes([2; 16]),
        quantity,
        unit: ids::unit("K"),
    };
    assert!(SlotBinding::new(&cold, &hot, &registry).is_err());
    let instance = InstanceBinding {
        instance: SemanticId::from_bytes([3; 16]),
        body: ContentHash::from_bytes([1; 32]),
        slots: vec![
            SlotBinding::new(&hot, &cold, &registry).unwrap(),
            SlotBinding::new(&cold, &cold, &registry).unwrap(),
        ],
        contributions: vec![Contribution {
            output: 0,
            target: Target::Row(SemanticId::from_bytes([4; 16])),
            scale: 1.0,
        }],
    };
    let mut builder = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &StandardInvariantChecker,
        2,
        BodyLimits::default(),
    )
    .unwrap();
    let a = builder
        .input(0, quantity, IndexSet::new(), source())
        .unwrap();
    let b = builder
        .input(1, quantity, IndexSet::new(), source())
        .unwrap();
    let residual = builder.binary(Binary::Sub, a, b, None, source()).unwrap();
    assert_eq!(residual.quantity, ids::quantity("temperature.difference"));
    let cancel = Arc::new(AtomicBool::new(false));
    let mut body = builder
        .finish(
            &[residual],
            DerivativeOrder::Second,
            Optimization::default(),
            &cancel,
        )
        .unwrap()
        .worker();
    let mut values = CaseValues {
        scalars: BTreeMap::from([(hot.id, 20.0), (cold.id, 293.15)]),
    };
    assert_eq!(
        body.evaluate(
            &instance.values(&values).unwrap(),
            DerivativeOrder::Value,
            &mut BTreeMap::new(),
            &cancel
        )
        .unwrap()
        .values,
        vec![0.0]
    );
    values.scalars.insert(cold.id, 283.15);
    assert_eq!(
        body.evaluate(
            &instance.values(&values).unwrap(),
            DerivativeOrder::Value,
            &mut BTreeMap::new(),
            &cancel
        )
        .unwrap()
        .values,
        vec![10.0]
    );
}

#[test]
fn integral_power_growth_is_bounded_before_cas_construction() {
    crate::initialize().unwrap();
    let registry = standard_registry().unwrap();
    let mut builder = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &StandardInvariantChecker,
        2,
        BodyLimits::default(),
    )
    .unwrap();
    let a = builder
        .input(0, ids::quantity("neutral"), IndexSet::new(), source())
        .unwrap();
    let b = builder
        .input(1, ids::quantity("neutral"), IndexSet::new(), source())
        .unwrap();
    let exponent = pse_quantity::Ratio::new(1025, 1).unwrap();
    assert!(matches!(
        builder.binary(Binary::Pow, a, b, Some(exponent), source()),
        Err(crate::MathError::Limit("integral power degree"))
    ));
}

#[test]
fn symbolic_context_registers_complete_vocabulary_before_concurrent_jobs() {
    let context = crate::initialize().unwrap();
    assert!(!context.environment.gmp.is_empty());
    assert!(!context.environment.mpfr.is_empty());
    let identity = context.environment.identity();
    let jobs: Vec<_> = [false, true]
        .into_iter()
        .map(|reverse| {
            std::thread::spawn(move || {
                let slots: Vec<_> = if reverse {
                    vec![4095, 0, 1]
                } else {
                    vec![0, 1, 4095]
                };
                let mut symbols = BTreeMap::new();
                for slot in slots {
                    symbols.insert(slot, crate::library::formal(slot).unwrap());
                }
                symbols
            })
        })
        .collect();
    let results: Vec<_> = jobs.into_iter().map(|job| job.join().unwrap()).collect();
    assert_eq!(results[0], results[1]);
    assert_eq!(identity, crate::context().unwrap().environment.identity());
    assert!(crate::library::formal(crate::library::MAX_FORMAL_SYMBOLS).is_err());
}

#[test]
fn symbolic_order_child() {
    let Ok(order) = std::env::var("PSE_SYMBOLIC_TEST_ORDER") else {
        return;
    };
    let context = crate::initialize().unwrap();
    let registry = standard_registry().unwrap();
    let mut outcomes = BTreeMap::new();
    for n in if order == "reverse" {
        vec![3, 2]
    } else {
        vec![2, 3]
    } {
        let mut builder = BodyBuilder::new(
            context,
            &registry,
            &StandardInvariantChecker,
            1,
            BodyLimits::default(),
        )
        .unwrap();
        let x = builder
            .input(0, ids::quantity("neutral"), IndexSet::new(), source())
            .unwrap();
        let mut y = x.clone();
        for _ in 1..n {
            y = builder
                .binary(Binary::Mul, y, x.clone(), None, source())
                .unwrap();
        }
        let body = builder.prepare(&[y]).unwrap();
        let compiled = body
            .compile(
                &[0],
                &[0],
                DerivativeOrder::Second,
                Optimization::default(),
                Default::default(),
                &Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
        let jet = compiled
            .worker()
            .evaluate(
                &[2.0],
                DerivativeOrder::Second,
                &mut BTreeMap::new(),
                &Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
        outcomes.insert(
            n,
            (
                body.expression(0).unwrap().to_string(),
                jet.values,
                jet.jacobian,
                jet.hessians,
            ),
        );
    }
    println!(
        "SYMBOLIC_CONTROL:{}",
        serde_json::to_string(&outcomes).unwrap()
    );
}
#[test]
fn symbolic_admission_order_agrees_across_fresh_processes() {
    let run = |order| {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "typed_tests::symbolic_order_child",
                "--nocapture",
            ])
            .env("PSE_SYMBOLIC_TEST_ORDER", order)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .find_map(|s| s.strip_prefix("SYMBOLIC_CONTROL:").map(str::to_owned))
            .unwrap()
    };
    assert_eq!(run("forward"), run("reverse"));
}
