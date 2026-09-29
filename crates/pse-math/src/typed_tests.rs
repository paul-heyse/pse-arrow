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
use pse_model::generated::enums::ModelingVariableDomain;
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
        domain: ModelingVariableDomain::Continuous,
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
}

/// The symbol of a registered pool slot and its Symbolica registration rank.
fn formal_symbol(slot: usize) -> symbolica::atom::Symbol {
    match crate::library::formal(slot).unwrap().as_view() {
        symbolica::atom::AtomView::Var(v) => v.get_symbol(),
        _ => panic!("a formal is a variable"),
    }
}

#[test]
fn formal_pool_extends_to_the_declared_limit_and_refuses_beyond() {
    use crate::{MathError, library::FORMAL_CHUNK};
    let context = crate::initialize().unwrap();
    let registry = standard_registry().unwrap();
    // A declared allowance beyond the first two chunks: every slot but the last is a formal
    // input, and preparation materializes the output into the last one.
    let declared = 2 * FORMAL_CHUNK + 3;
    let inputs = declared - 1;
    let prepare = |slots| {
        let mut builder = BodyBuilder::new(
            context,
            &registry,
            &StandardInvariantChecker,
            inputs,
            BodyLimits {
                slots,
                occurrences: 16384,
            },
        )?;
        let x = builder.input(inputs - 1, ids::quantity("neutral"), IndexSet::new(), source())?;
        builder.prepare(&[x])
    };
    assert!(matches!(
        prepare(declared - 1),
        Err(MathError::SlotLimit { required, available }) if required == declared && available == declared - 1
    ));
    assert!(matches!(
        prepare(inputs - 1),
        Err(MathError::SlotLimit { required, available }) if required == inputs && available == inputs - 1
    ));
    let error = prepare(declared - 1).unwrap_err();
    assert_eq!(
        pse_diagnostics::TypedDiagnostic::diagnostic_code(&error),
        Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit)
    );
    assert!(error.to_string().contains("body slots"), "{error}");
    let body = prepare(declared).unwrap();
    // Preparation reports the slots and construction occurrences it used.
    assert_eq!(body.slot_count(), declared);
    assert_eq!(body.occurrence_count(), 1);
    // The pool grew in whole chunks to cover the declared allowance.
    let pool = crate::library::formal_pool_len().unwrap();
    assert!(pool >= declared && pool % FORMAL_CHUNK == 0, "{pool}");
    let mut values = vec![0.0; inputs];
    values[inputs - 1] = 2.5;
    let jet = body
        .compile(
            &[0],
            &[inputs - 1],
            DerivativeOrder::First,
            Optimization::default(),
            Default::default(),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
        .worker()
        .evaluate(
            &values,
            DerivativeOrder::First,
            &mut BTreeMap::new(),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert_eq!(jet.values, vec![2.5]);
    assert_eq!(jet.jacobian, vec![1.0]);
}

#[test]
fn formal_symbols_are_stable_across_pool_extension() {
    use crate::library::FORMAL_CHUNK;
    crate::initialize().unwrap();
    let first = [0, 1, FORMAL_CHUNK - 1];
    let before = first.map(formal_symbol);
    let function = |slot| crate::context().unwrap().pool.function(slot).unwrap();
    let last_function = function(FORMAL_CHUNK - 1);
    // Concurrent first requests beyond the pool extend it once, in its fixed order.
    let far = 3 * FORMAL_CHUNK + 7;
    let jobs: Vec<_> = (0..4)
        .map(|i| std::thread::spawn(move || (formal_symbol(far - i), formal_symbol(FORMAL_CHUNK + i))))
        .collect();
    let extended: Vec<_> = jobs.into_iter().map(|job| job.join().unwrap()).collect();
    for (i, (far_symbol, next_symbol)) in extended.into_iter().enumerate() {
        assert_eq!(far_symbol, formal_symbol(far - i));
        assert_eq!(next_symbol, formal_symbol(FORMAL_CHUNK + i));
    }
    // The same slot is the same symbol before and after growth, and registration by name
    // returns it.
    assert_eq!(first.map(formal_symbol), before);
    assert_eq!(function(FORMAL_CHUNK - 1), last_function);
    for slot in [0, FORMAL_CHUNK - 1, FORMAL_CHUNK, far] {
        let named = symbolica::atom::SymbolBuilder::new(
            symbolica::atom::NamespacedSymbol::try_from(format!("pse_math::slot_{slot}").as_str())
                .unwrap(),
        )
        .build()
        .unwrap();
        assert_eq!(named, formal_symbol(slot));
    }
    // Registration order, Symbolica's canonical term order, is fixed: slot order within a
    // family, and each chunk's formals, then its functions, before the next chunk.
    for chunk in 0..4 {
        let start = chunk * FORMAL_CHUNK;
        let end = start + FORMAL_CHUNK - 1;
        assert!(formal_symbol(start) < formal_symbol(end));
        assert!(formal_symbol(end) < function(start));
        assert!(function(start) < function(end));
        if chunk > 0 {
            assert!(function(start - 1) < formal_symbol(start));
        }
    }
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

#[test]
fn shared_let_blocks_bound_repeated_nonlinear_tree_growth() {
    crate::initialize().unwrap();
    let registry = standard_registry().unwrap();
    let mut builder = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &StandardInvariantChecker,
        1,
        BodyLimits::default(),
    )
    .unwrap();
    let mut value = builder
        .input(0, ids::quantity("neutral"), IndexSet::new(), source())
        .unwrap();
    let (mut expected, mut first, mut second) = (0.2_f64, 1.0, 0.0);
    for _ in 0..32 {
        let sin = builder
            .unary(Function::Sin, value.clone(), source())
            .unwrap();
        let cos = builder.unary(Function::Cos, value, source()).unwrap();
        value = builder
            .binary(Binary::Add, sin, cos, None, source())
            .unwrap();
        value = builder.bind(value).unwrap();
        second = -(expected.sin() + expected.cos()) * first * first
            + (expected.cos() - expected.sin()) * second;
        first *= expected.cos() - expected.sin();
        expected = expected.sin() + expected.cos();
    }
    let body = builder.prepare(&[value]).unwrap();
    assert!(body.expression(0).is_none()); // optional flattening stopped; the shared program remains exact
    let compiled = body
        .compile(
            &[0],
            &[0],
            DerivativeOrder::Second,
            Optimization::default(),
            crate::jets::EvaluationLimits::default(),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    let result = compiled
        .worker()
        .evaluate(
            &[0.2],
            DerivativeOrder::Second,
            &mut BTreeMap::new(),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert!((result.values[0] - expected).abs() < 1e-12);
    assert!((result.jacobian[0] - first).abs() < 1e-12);
    assert!((result.hessians[0] - second).abs() < 1e-12);
}

#[test]
fn body_construction_allowance_can_be_explicitly_larger_than_default() {
    let registry = standard_registry().unwrap();
    let q = ids::quantity("neutral");
    for allowance in [BodyLimits::default().occurrences, 32768] {
        let mut builder = BodyBuilder::new(
            crate::initialize().unwrap(),
            &registry,
            &StandardInvariantChecker,
            1,
            BodyLimits {
                occurrences: allowance,
                ..BodyLimits::default()
            },
        )
        .unwrap();
        let x = builder.input(0, q, IndexSet::new(), source()).unwrap();
        let mut value = x.clone();
        let mut failure = None;
        for _ in 0..16385 {
            match builder.binary(Binary::Add, value.clone(), x.clone(), None, source()) {
                Ok(next) => value = next,
                Err(error) => {
                    failure = Some(error);
                    break;
                }
            }
        }
        if allowance == BodyLimits::default().occurrences {
            assert!(matches!(
                failure,
                Some(crate::MathError::Limit("body occurrences"))
            ));
        } else {
            assert!(failure.is_none());
            let cancel = Arc::new(AtomicBool::new(false));
            let mut worker = builder
                .finish(
                    &[value],
                    DerivativeOrder::First,
                    Optimization::default(),
                    &cancel,
                )
                .unwrap()
                .worker();
            let jet = worker
                .evaluate(&[2.], DerivativeOrder::First, &mut BTreeMap::new(), &cancel)
                .unwrap();
            assert_eq!(jet.values, vec![32772.]);
            assert_eq!(jet.jacobian, vec![16386.]);
        }
    }
}

#[test]
fn shared_nonlinear_coefficients_preserve_affine_rate_support() {
    let registry = standard_registry().unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    for nonlinear_rate in [false, true] {
        let mut builder = BodyBuilder::new(
            crate::initialize().unwrap(),
            &registry,
            &StandardInvariantChecker,
            2,
            BodyLimits::default(),
        )
        .unwrap();
        let rate = builder
            .input(0, ids::quantity("neutral"), IndexSet::new(), source())
            .unwrap();
        let parameter = builder
            .input(1, ids::quantity("neutral"), IndexSet::new(), source())
            .unwrap();
        let mut coefficient = builder.unary(Function::Log, parameter, source()).unwrap();
        let mut expected = 0.2_f64;
        for _ in 0..32 {
            let sin = builder
                .unary(Function::Sin, coefficient.clone(), source())
                .unwrap();
            let cos = builder.unary(Function::Cos, coefficient, source()).unwrap();
            coefficient = builder
                .binary(Binary::Add, sin, cos, None, source())
                .unwrap();
            coefficient = builder.bind(coefficient).unwrap();
            expected = expected.sin() + expected.cos();
        }
        let product = builder
            .binary(Binary::Mul, rate.clone(), coefficient, None, source())
            .unwrap();
        let output = if nonlinear_rate {
            builder
                .binary(Binary::Mul, product, rate, None, source())
                .unwrap()
        } else {
            product
        };
        let body = builder.prepare(&[output]).unwrap();
        if nonlinear_rate {
            assert!(body.support().second[0].contains(&(0, 0)));
            assert!(crate::implicit::Affine::new(&body, 1).is_err());
            continue;
        }
        assert!(body.expression(0).is_none());
        assert!(!body.support().second[0].contains(&(0, 0)));
        assert!(body.support().second[0].contains(&(0, 1)));
        assert!(body.support().second[0].contains(&(1, 1)));
        crate::implicit::Affine::new(&body, 1).unwrap();
        let mut worker = body
            .compile(
                &[0],
                &[0, 1],
                DerivativeOrder::Second,
                Optimization::default(),
                crate::jets::EvaluationLimits::default(),
                &cancel,
            )
            .unwrap()
            .worker();
        let jet = worker
            .evaluate(
                &[3., 0.2_f64.exp()],
                DerivativeOrder::Second,
                &mut BTreeMap::new(),
                &cancel,
            )
            .unwrap();
        assert!((jet.values[0] - 3. * expected).abs() < 1e-12);
        assert!((jet.jacobian[0] - expected).abs() < 1e-12);
        assert_eq!(jet.hessians[0], 0.);
        assert!(
            worker
                .evaluate(
                    &[3., -1.],
                    DerivativeOrder::Second,
                    &mut BTreeMap::new(),
                    &cancel
                )
                .is_err()
        );
    }
}

#[test]
fn exponential_of_nested_logarithms_preserves_values_and_derivatives() {
    crate::initialize().unwrap();
    let registry = standard_registry().unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    for fixed in [false, true] {
        let mut b = BodyBuilder::new(
            crate::initialize().unwrap(),
            &registry,
            &StandardInvariantChecker,
            usize::from(!fixed),
            BodyLimits::default(),
        )
        .unwrap();
        let literal = |b: &mut BodyBuilder<'_>, value| {
            b.literal(
                value,
                registry
                    .quantity_type(ids::quantity("neutral"))
                    .unwrap()
                    .canonical_unit,
                pse_quantity::literal::LiteralContext::Explicit {
                    quantity_type: ids::quantity("neutral"),
                },
                source(),
            )
            .unwrap()
        };
        let x = if fixed {
            literal(&mut b, 2.)
        } else {
            b.input(0, ids::quantity("neutral"), IndexSet::new(), source())
                .unwrap()
        };
        let logarithm = b.unary(Function::Log, x, source()).unwrap();
        let quarter = literal(&mut b, 0.25);
        let term = b
            .binary(Binary::Mul, logarithm.clone(), quarter, None, source())
            .unwrap();
        let one = literal(&mut b, 1.);
        let term = b.binary(Binary::Sub, term, one, None, source()).unwrap();
        let term = b
            .binary(Binary::Mul, logarithm, term, None, source())
            .unwrap();
        let three = literal(&mut b, 3.);
        let exponent = b.binary(Binary::Add, three, term, None, source()).unwrap();
        let exponential = b.unary(Function::Exp, exponent.clone(), source()).unwrap();
        let mut worker = b
            .finish(
                &[exponential, exponent],
                DerivativeOrder::Second,
                Optimization::default(),
                &cancel,
            )
            .unwrap()
            .worker();
        for x in if fixed {
            vec![2.0_f64]
        } else {
            vec![2.0_f64, 5., 20.]
        } {
            let g = 3. + x.ln() * (x.ln() / 4. - 1.);
            let dg = (x.ln() / 2. - 1.) / x;
            let ddg = (1.5 - x.ln() / 2.) / (x * x);
            for order in [
                DerivativeOrder::Value,
                DerivativeOrder::First,
                DerivativeOrder::Second,
            ] {
                let result = worker
                    .evaluate(
                        &if fixed { vec![] } else { vec![x] },
                        order,
                        &mut BTreeMap::new(),
                        &cancel,
                    )
                    .unwrap();
                assert!((result.values[0] - g.exp()).abs() < 1e-11, "{result:?}");
                assert!((result.values[1] - g).abs() < 1e-12);
                if !fixed && order >= DerivativeOrder::First {
                    assert!((result.jacobian[0] - g.exp() * dg).abs() < 1e-11);
                    assert!((result.jacobian[1] - dg).abs() < 1e-12);
                }
                if !fixed && order >= DerivativeOrder::Second {
                    assert!((result.hessians[0] - g.exp() * (dg * dg + ddg)).abs() < 1e-11);
                    assert!((result.hessians[1] - ddg).abs() < 1e-12);
                }
            }
        }
    }
}

#[test]
fn shared_aliases_reuse_slots_but_partial_arguments_remain_independent() {
    let registry = standard_registry().unwrap();
    let mut builder = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &StandardInvariantChecker,
        1,
        BodyLimits {
            slots: 8,
            occurrences: 128,
        },
    )
    .unwrap();
    let mut x = builder
        .input(0, ids::quantity("neutral"), IndexSet::new(), source())
        .unwrap();
    for _ in 0..64 {
        x = builder.bind(x).unwrap();
    }
    let a = builder.independent(x.clone()).unwrap();
    let b = builder.independent(x).unwrap();
    let scope = builder.function_scope();
    let product = builder
        .binary(Binary::Mul, a.clone(), b, None, source())
        .unwrap();
    let derivative = builder.partial(scope, product, &[a], source()).unwrap();
    let body = builder.prepare(&[derivative]).unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let compiled = body
        .compile(
            &[0],
            &[0],
            DerivativeOrder::Second,
            Optimization::default(),
            crate::jets::EvaluationLimits::default(),
            &cancel,
        )
        .unwrap();
    let result = compiled
        .worker()
        .evaluate(
            &[3.],
            DerivativeOrder::Second,
            &mut BTreeMap::new(),
            &cancel,
        )
        .unwrap();
    assert_eq!(result.values, vec![3.]);
    assert_eq!(result.jacobian, vec![1.]);
}
