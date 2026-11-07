// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Released IAPWS controls and native datum/derivative/branch observations.
use super::*;
use pse_kernels::{ProviderBuffer, ProviderRequest, ProviderValues};
use pse_quantity::PhysicalName;
use std::sync::atomic::AtomicBool;

fn saturation_partial(
    api: &CoolProp,
    state: &NativeState,
    of: c_long,
    pressure: c_long,
    second: bool,
) -> Result<f64, ProviderError> {
    let mut error = ErrorBuffer::default();
    // SAFETY: the original live handle and selected keys are retained; destinations are bounded.
    let value = unsafe {
        if second {
            api.AbstractState_second_saturation_deriv(
                state.handle,
                of,
                pressure,
                pressure,
                &raw mut error.code,
                error.message.as_mut_ptr().cast(),
                ERROR_CAPACITY as c_long,
            )
        } else {
            api.AbstractState_first_saturation_deriv(
                state.handle,
                of,
                pressure,
                &raw mut error.code,
                error.message.as_mut_ptr().cast(),
                ERROR_CAPACITY as c_long,
            )
        }
    };
    error.check("saturation derivative", true)?;
    finite(value)
}

#[test]
fn prepared_water_saturation_c_abi_preserves_both_endpoint_values_first_and_second() {
    ensure_library().unwrap();
    let api = COOLPROP.shared_access();
    authenticate(&api).unwrap();
    let keys = Keys::new(&api).unwrap();
    let pair = unsafe { api.get_input_pair_index(c"PQ_INPUTS".as_ptr()) };
    assert!(pair >= 0);
    let mut state = NativeState::new(&api).unwrap();
    let fields = [keys.temperature, keys.density, keys.h, keys.s, keys.u];
    let mut endpoints = Vec::new();
    for quality in [0.0, 1.0] {
        state.update(&api, pair, 140_000.0, quality).unwrap();
        // Released third turbine expectation; independent rounded source, not a new reference run.
        near(
            state.output(&api, keys.temperature).unwrap(),
            382.442,
            0.01,
            0.0,
        );
        let values = fields.map(|key| state.output(&api, key).unwrap());
        near(values[2] - values[4], 140_000.0 / values[1], 1e-6, 1e-10);
        let first =
            fields.map(|key| saturation_partial(&api, &state, key, keys.pressure, false).unwrap());
        let second =
            fields.map(|key| saturation_partial(&api, &state, key, keys.pressure, true).unwrap());
        // Independent numerical comparisons are test-only, not a production derivative path.
        let step = 10.0;
        state.update(&api, pair, 140_000.0 + step, quality).unwrap();
        let plus = fields.map(|key| state.output(&api, key).unwrap());
        let dplus =
            fields.map(|key| saturation_partial(&api, &state, key, keys.pressure, false).unwrap());
        state.update(&api, pair, 140_000.0 - step, quality).unwrap();
        let minus = fields.map(|key| state.output(&api, key).unwrap());
        let dminus =
            fields.map(|key| saturation_partial(&api, &state, key, keys.pressure, false).unwrap());
        for index in 0..fields.len() {
            near(
                first[index],
                (plus[index] - minus[index]) / (2.0 * step),
                1e-10,
                5e-6,
            );
            near(
                second[index],
                (dplus[index] - dminus[index]) / (2.0 * step),
                1e-12,
                3e-5,
            );
        }
        endpoints.push(values);
    }
    let [liquid, vapor] = endpoints.as_slice() else {
        panic!("both original endpoints")
    };
    assert!(liquid[1] > vapor[1]);
    assert!(vapor[2] > liquid[2] && vapor[3] > liquid[3]);
    state.update(&api, pair, 140_000.0, 0.0).unwrap();
    // Clapeyron correspondence independently composes original endpoint values.
    near(
        saturation_partial(&api, &state, keys.temperature, keys.pressure, false).unwrap(),
        liquid[0] * (1.0 / vapor[1] - 1.0 / liquid[1]) / (vapor[2] - liquid[2]),
        1e-12,
        1e-9,
    );
}

#[test]
fn prepared_saturation_c_abi_preserves_native_handle_key_and_bounded_error_refusals() {
    ensure_library().unwrap();
    let api = COOLPROP.shared_access();
    let keys = Keys::new(&api).unwrap();
    let mut state = NativeState::new(&api).unwrap();
    let pair = unsafe { api.get_input_pair_index(c"PQ_INPUTS".as_ptr()) };
    state.update(&api, pair, 140_000.0, 1.0).unwrap();
    for (handle, of, wrt) in [
        (-1, keys.h, keys.pressure),
        (state.handle, -1, keys.pressure),
        (state.handle, keys.h, keys.temperature),
    ] {
        let mut code = 0;
        let mut guarded = [0xa5_u8; 18];
        // SAFETY: only the middle bounded writable region is offered; invalid inputs test the C exception boundary.
        unsafe {
            api.AbstractState_second_saturation_deriv(
                handle,
                of,
                wrt,
                wrt,
                &raw mut code,
                guarded[1..17].as_mut_ptr().cast(),
                16,
            );
        }
        assert_ne!(code, 0);
        assert_eq!(guarded[0], 0xa5);
        assert_eq!(guarded[17], 0xa5);
    }
    assert!(state.update(&api, pair, -1.0, 0.0).is_err());
}

fn factory(regime: WaterRegime) -> WaterFactory {
    WaterFactory::prepare(
        &pse_quantity::standard::standard_registry().unwrap(),
        regime,
    )
    .unwrap()
}
fn evaluate(
    factory: &WaterFactory,
    point: [f64; 2],
    order: DerivativeOrder,
) -> Result<ProviderValues, ProviderError> {
    let mut binding = factory.create()?;
    let request = binding.prepare(ProviderRequest::all(binding.spec(), order))?;
    let mut buffer = ProviderBuffer::new(&request);
    let cancelled = AtomicBool::new(false);
    let context = EvaluationContext {
        cancelled: &cancelled,
        max_result_bytes: 4096,
    };
    let value = buffer.fill(
        &mut binding,
        &point,
        &request,
        ProviderRuntime::default(),
        &context,
    )?;
    Ok(ProviderValues {
        values: value.values.to_vec(),
        jacobian: value.jacobian.to_vec(),
        hessians: value.hessians.to_vec(),
    })
}
fn near(actual: f64, expected: f64, absolute: f64, relative: f64) {
    assert!(
        (actual - expected).abs() <= absolute + relative * expected.abs(),
        "actual={actual:.16e}, expected={expected:.16e}"
    );
}

#[test]
fn fixed_si_factory_refuses_valid_registries_with_changed_consumed_unit_meaning() {
    let registry = pse_quantity::standard::standard_registry().unwrap();
    descriptor(&registry, WaterRegime::Vapor).unwrap();
    for name in [
        "Temperature",
        "Density",
        "Pressure",
        "MolarEnthalpy",
        "MolarEntropy",
    ] {
        let Some(PhysicalName::QuantityType(quantity)) = registry.physical_name(name) else {
            panic!("physical quantity {name}")
        };
        let ty = registry.quantity_type(quantity).unwrap();
        let canonical = registry.unit(ty.canonical_unit).unwrap();
        let atomic = canonical
            .definition
            .as_ref()
            .map_or(canonical.id, |factors| factors[0].unit);
        let mut declarations = serde_json::to_value(registry.to_builder()).unwrap();
        let units = declarations["units"].as_array_mut().unwrap();
        let mut changed = 0;
        for unit in units {
            if unit["id"] == atomic.to_string() {
                let scale = unit["scale_to_canonical"].as_f64().unwrap();
                unit["scale_to_canonical"] = serde_json::json!(scale * 2.0);
                changed += 1;
            }
        }
        assert_eq!(changed, 1, "{name}");
        let builder: pse_quantity::QuantityRegistryBuilder =
            serde_json::from_value(declarations).unwrap();
        let changed = builder.build().unwrap();
        // Native inventory admission establishes internal consistency, not the ABI's units.
        assert!(
            matches!(
                descriptor(&changed, WaterRegime::Vapor),
                Err(ProviderError::Contract(_))
            ),
            "{name}"
        );
        assert!(
            matches!(
                WaterFactory::prepare(&changed, WaterRegime::Vapor),
                Err(ProviderError::Contract(_))
            ),
            "{name}"
        );
    }
}

#[test]
fn fixed_si_factory_refuses_valid_inventory_with_changed_molar_basis_meaning() {
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let Some(PhysicalName::QuantityType(density)) = registry.physical_name("Density") else {
        panic!("Density")
    };
    let basis = registry.quantity_type(density).unwrap().key.basis.unwrap();
    let mut declarations = serde_json::to_value(registry.to_builder()).unwrap();
    let bases = declarations["bases"].as_array_mut().unwrap();
    let mut changed = 0;
    for declaration in bases {
        if declaration["id"] == basis.to_string() {
            declaration["kind"] = serde_json::json!("mass");
            changed += 1;
        }
    }
    assert_eq!(changed, 1);
    let builder: pse_quantity::QuantityRegistryBuilder =
        serde_json::from_value(declarations).unwrap();
    let changed = builder.build().unwrap();
    assert!(matches!(
        WaterFactory::prepare(&changed, WaterRegime::Vapor),
        Err(ProviderError::Contract(_))
    ));
}

#[test]
fn fixed_si_factory_refuses_same_dimension_foreign_nominal_kind() {
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let Some(PhysicalName::QuantityType(temperature)) = registry.physical_name("Temperature")
    else {
        panic!("Temperature")
    };
    let original = registry.quantity_type(temperature).unwrap();
    let mut foreign = registry.kind(original.key.kind).unwrap().clone();
    foreign.id = pse_quantity::QuantityKindId::from_bytes([0xee; 16]);
    let foreign_id = foreign.id;
    let mut builder = registry.to_builder();
    builder.kind(foreign);
    let mut declarations = serde_json::to_value(builder).unwrap();
    let mut changed = 0;
    for ty in declarations["quantity_types"].as_array_mut().unwrap() {
        if ty["id"] == temperature.to_string() {
            ty["key"]["kind"] = serde_json::to_value(foreign_id).unwrap();
            changed += 1;
        }
    }
    assert_eq!(changed, 1);
    let builder: pse_quantity::QuantityRegistryBuilder =
        serde_json::from_value(declarations).unwrap();
    let changed = builder.build().unwrap();
    assert_eq!(
        registry.unit(original.canonical_unit).unwrap().dimension,
        changed.unit(original.canonical_unit).unwrap().dimension
    );
    assert!(matches!(
        WaterFactory::prepare(&changed, WaterRegime::Vapor),
        Err(ProviderError::Contract(_))
    ));
}

#[test]
fn parameterized_stock_conditions_keep_native_roles_and_change_only_datum_configuration() {
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let original = WaterFactory::prepare(&registry, WaterRegime::Vapor).unwrap();
    let reference = evaluate(&original, [500.0, 250.0], DerivativeOrder::Second).unwrap();
    for (axis, increment) in [("temperature", 1.0), ("pressure", 1000.0)] {
        let mut declarations = serde_json::to_value(registry.to_builder()).unwrap();
        let mut changed = 0;
        for datum in declarations["reference_states"].as_array_mut().unwrap() {
            if datum["id"] == WATER_STOCK_DATUM.to_string() {
                let value = datum[axis]["value"].as_f64().unwrap();
                datum[axis]["value"] = serde_json::json!(value + increment);
                changed += 1;
            }
        }
        assert_eq!(changed, 1);
        let builder: pse_quantity::QuantityRegistryBuilder =
            serde_json::from_value(declarations).unwrap();
        let changed = builder.build().unwrap();
        let factory = WaterFactory::prepare(&changed, WaterRegime::Vapor).unwrap();
        assert_eq!(original.spec(), factory.spec());
        assert_ne!(original.configuration_key(), factory.configuration_key());
        let value = evaluate(&factory, [500.0, 250.0], DerivativeOrder::Second).unwrap();
        assert_eq!(reference.values[0], value.values[0]);
        assert_ne!(&reference.values[1..], &value.values[1..]);
        assert_eq!(reference.jacobian, value.jacobian);
        assert_eq!(reference.hessians, value.hessians);
    }
}

#[test]
fn native_batch_preserves_independent_iapws_points_mixed_refusals_and_selected_order() {
    let factory = factory(WaterRegime::Liquid);
    let mut binding = factory.create().unwrap();
    let request = binding
        .prepare(ProviderRequest {
            outputs: vec![2, 0, 1],
            order: DerivativeOrder::Value,
            demand: ProviderDemand::CompleteJets,
        })
        .unwrap();
    assert!(binding.supports_batch(&request));
    let derivative = binding
        .prepare(ProviderRequest::all(binding.spec(), DerivativeOrder::First))
        .unwrap();
    assert!(!binding.supports_batch(&derivative));
    let points = [
        ProviderBatchPoint::values(
            SemanticId::from_bytes([1; 16]),
            vec![300.0, 996.556 / MOLAR_MASS],
        ),
        ProviderBatchPoint::values(
            SemanticId::from_bytes([2; 16]),
            vec![f64::from_bits(0x7ff8_0000_0000_1234), 50000.0],
        ),
        ProviderBatchPoint::values(SemanticId::from_bytes([3; 16]), vec![250.0, 50000.0]),
        ProviderBatchPoint::values(
            SemanticId::from_bytes([4; 16]),
            vec![500.0, 0.435 / MOLAR_MASS],
        ),
        ProviderBatchPoint::values(SemanticId::from_bytes([5; 16]), vec![310.0, 56000.0]),
    ];
    let cancelled = AtomicBool::new(false);
    let context = EvaluationContext {
        cancelled: &cancelled,
        max_result_bytes: 65536,
    };
    let calls = BATCH_NATIVE_CALLS.with(std::cell::Cell::get);
    let results = binding.evaluate_batch(&points, &request, &context).unwrap();
    assert_eq!(BATCH_NATIVE_CALLS.with(std::cell::Cell::get), calls + 1);
    assert_eq!(
        results.iter().map(|point| point.id).collect::<Vec<_>>(),
        points.iter().map(|point| point.id).collect::<Vec<_>>()
    );
    let first = results[0].result.as_ref().unwrap();
    near(first.values[1], 99_241.835_2, 0.02, 1e-9);
    near(
        (first.values[0] - factory.reference.s_offset) / MOLAR_MASS,
        393.062_643,
        2e-5,
        1e-9,
    );
    assert!(results[1..4].iter().all(|point| point.result.is_err()));
    let last = results[4].result.as_ref().unwrap();
    let scalar = evaluate(&factory, [310.0, 56000.0], DerivativeOrder::Value).unwrap();
    for (&actual, expected) in
        last.values
            .iter()
            .zip([scalar.values[2], scalar.values[0], scalar.values[1]])
    {
        near(actual, expected, 1e-7, 1e-12);
    }
    let mut duplicate = points[..1].to_vec();
    duplicate.push(points[0].clone());
    let calls = BATCH_NATIVE_CALLS.with(std::cell::Cell::get);
    assert!(
        binding
            .evaluate_batch(&duplicate, &request, &context)
            .is_err()
    );
    assert!(
        binding
            .evaluate_batch(&points[1..3], &request, &context)
            .unwrap()
            .iter()
            .all(|point| point.result.is_err())
    );
    assert_eq!(BATCH_NATIVE_CALLS.with(std::cell::Cell::get), calls);
}

#[test]
fn batch_slot_admission_checks_every_guard_and_preserves_native_physical_policy() {
    let factory = factory(WaterRegime::Liquid);
    let api = COOLPROP.shared_access();
    let mut state = NativeState::new(&api).unwrap();
    let density = 996.556 / MOLAR_MASS;
    state
        .update(&api, factory.keys.density_temperature, density, 300.0)
        .unwrap();
    let row = [
        state.output(&api, factory.keys.pressure).unwrap(),
        state.output(&api, factory.keys.h).unwrap(),
        state.output(&api, factory.keys.s).unwrap(),
        state.output(&api, factory.keys.u).unwrap(),
        f64::from(state.phase(&api).unwrap()),
        state
            .first(
                &api,
                factory.keys.pressure,
                factory.keys.density,
                factory.keys.temperature,
            )
            .unwrap(),
    ];
    assert!(qualified_water_row(&factory, density, &row).is_ok());
    for missing in 0..row.len() {
        let mut partial = row;
        partial[missing] = f64::NAN;
        assert!(qualified_water_row(&factory, density, &partial).is_err());
    }
    let mut unstable = row;
    unstable[5] = -1.0;
    assert!(qualified_water_row(&factory, density, &unstable).is_err());
    let mut inconsistent = row;
    inconsistent[3] += 1.0;
    assert!(matches!(
        qualified_water_row(&factory, density, &inconsistent),
        Err(ProviderError::Terminal(_))
    ));
}

#[test]
fn released_iapws_liquid_and_vapor_controls_preserve_the_identified_source_datum() {
    // IAPWS R6-95(2018), Table 7 (not generated by this provider):
    // https://iapws.org/public/documents/k3PSK/IAPWS95-2018.pdf
    for (regime, temperature, mass_density, pressure, specific_entropy) in [
        (
            WaterRegime::Liquid,
            300.0,
            996.556,
            99_241.835_2,
            393.062_643,
        ),
        (WaterRegime::Vapor, 500.0, 0.435, 99_967.942_3, 7_944.882_71),
    ] {
        let factory = factory(regime);
        let values = evaluate(
            &factory,
            [temperature, mass_density / MOLAR_MASS],
            DerivativeOrder::Second,
        )
        .unwrap();
        near(values.values[0], pressure, 0.02, 1e-9);
        // Undo the explicitly retained stock translation before comparing the
        // published entropy, whose source convention is different.
        near(
            (values.values[2] - factory.reference.s_offset) / MOLAR_MASS,
            specific_entropy,
            2e-5,
            1e-9,
        );
        assert!(
            values
                .jacobian
                .iter()
                .chain(&values.hessians)
                .all(|x| x.is_finite())
        );
    }
}

#[test]
fn ideal_only_anchor_is_zero_and_identical_h_u_shift_preserves_energy_relation() {
    let factory = factory(WaterRegime::Vapor);
    let api = COOLPROP.shared_access();
    let mut state = NativeState::new(&api).unwrap();
    let mut error = ErrorBuffer::default();
    // SAFETY: test owns the live handle; bounded error storage and literal outlive the call.
    unsafe {
        api.AbstractState_specify_phase(
            state.handle,
            c"phase_gas".as_ptr(),
            &raw mut error.code,
            error.message.as_mut_ptr().cast(),
            ERROR_CAPACITY as c_long,
        );
    }
    error
        .check("independent ideal anchor control", false)
        .unwrap();
    let reference = factory.reference;
    state
        .update(
            &api,
            factory.keys.density_temperature,
            reference.pressure / (reference.gas_constant * reference.temperature),
            reference.temperature,
        )
        .unwrap();
    near(
        state.output(&api, factory.keys.ideal_h).unwrap() + reference.h_offset,
        0.0,
        1e-10,
        0.0,
    );
    near(
        state.output(&api, factory.keys.ideal_s).unwrap() + reference.s_offset,
        0.0,
        1e-12,
        0.0,
    );
    // A new, unforced physical state, distinct from the ideal-only anchor.
    drop(api);
    drop(state);
    let api = COOLPROP.shared_access();
    let mut physical = NativeState::new(&api).unwrap();
    physical
        .update(&api, factory.keys.density_temperature, 250.0, 500.0)
        .unwrap();
    let pressure = physical.output(&api, factory.keys.pressure).unwrap();
    let h = physical.output(&api, factory.keys.h).unwrap() + reference.h_offset;
    let u = physical.output(&api, factory.keys.u).unwrap() + reference.h_offset;
    near(h - u, pressure / 250.0, 1e-9, 1e-12);
    drop(api);
    drop(physical);
}

#[test]
fn complete_native_jets_match_independent_centered_observations() {
    let factory = factory(WaterRegime::Vapor);
    let point = [500.0, 250.0];
    let base = evaluate(&factory, point, DerivativeOrder::Second).unwrap();
    for axis in 0..2 {
        let step = 0.002;
        let mut plus = point;
        let mut minus = point;
        plus[axis] += step;
        minus[axis] -= step;
        let plus = evaluate(&factory, plus, DerivativeOrder::First).unwrap();
        let minus = evaluate(&factory, minus, DerivativeOrder::First).unwrap();
        for output in 0..3 {
            near(
                base.jacobian[output * 2 + axis],
                (plus.values[output] - minus.values[output]) / (2.0 * step),
                1e-6,
                2e-7,
            );
            for first in 0..2 {
                near(
                    base.hessians[output * 4 + first * 2 + axis],
                    (plus.jacobian[output * 2 + first] - minus.jacobian[output * 2 + first])
                        / (2.0 * step),
                    1e-7,
                    2e-6,
                );
            }
            near(
                base.hessians[output * 4 + 1],
                base.hessians[output * 4 + 2],
                1e-10,
                1e-10,
            );
        }
    }
}

#[test]
fn native_selected_columns_use_requested_partials_orders_and_regimes() {
    for (regime, point) in [
        (WaterRegime::Liquid, [300.0, 996.556 / MOLAR_MASS]),
        (WaterRegime::Vapor, [500.0, 0.435 / MOLAR_MASS]),
        (WaterRegime::Supercritical, [650.0, 500.0 / MOLAR_MASS]),
    ] {
        let factory = factory(regime);
        for order in [DerivativeOrder::First, DerivativeOrder::Second] {
            let full = evaluate(&factory, point, order).unwrap();
            for columns in [vec![0], vec![1], vec![1, 0]] {
                let mut binding = factory.create().unwrap();
                let outputs = vec![2, 0];
                let request = binding
                    .prepare(ProviderRequest {
                        outputs: outputs.clone(),
                        order,
                        demand: ProviderDemand::SelectedColumns {
                            columns: columns.clone(),
                        },
                    })
                    .unwrap();
                assert!(!binding.supports_batch(&request));
                let mut buffer = ProviderBuffer::new(&request);
                let flag = AtomicBool::new(false);
                let selected = buffer
                    .fill(
                        &mut binding,
                        &point,
                        &request,
                        ProviderRuntime::default(),
                        &EvaluationContext {
                            cancelled: &flag,
                            max_result_bytes: 4096,
                        },
                    )
                    .unwrap();
                assert_eq!(
                    selected.values,
                    outputs
                        .iter()
                        .map(|&row| full.values[row])
                        .collect::<Vec<_>>()
                );
                let mut first = Vec::new();
                for &row in &outputs {
                    for &column in &columns {
                        first.push(full.jacobian[row * 2 + column]);
                    }
                }
                assert_eq!(selected.jacobian, first);
                if order == DerivativeOrder::Second {
                    let mut second = Vec::new();
                    for &row in &outputs {
                        for &a in &columns {
                            for &b in &columns {
                                second.push(full.hessians[row * 4 + a * 2 + b]);
                            }
                        }
                    }
                    assert_eq!(selected.hessians, second);
                } else {
                    assert!(selected.hessians.is_empty());
                }
            }
        }
        let mut binding = factory.create().unwrap();
        for (order, columns) in [
            (DerivativeOrder::Value, vec![0]),
            (DerivativeOrder::First, vec![]),
            (DerivativeOrder::First, vec![2]),
            (DerivativeOrder::Second, vec![0, 0]),
        ] {
            assert!(matches!(
                binding.prepare(ProviderRequest {
                    outputs: vec![0],
                    order,
                    demand: ProviderDemand::SelectedColumns { columns }
                }),
                Err(ProviderError::Contract(_))
            ));
        }
        for demand in [
            ProviderDemand::JacobianAction { directions: 1 },
            ProviderDemand::TransposeAction { cotangents: 1 },
            ProviderDemand::WeightedSecondAction { directions: 1 },
        ] {
            assert!(!factory.supports(&demand));
        }
    }
}

#[test]
fn actual_phase_domain_and_cancellation_refuse_without_phase_imposition() {
    let liquid = factory(WaterRegime::Liquid);
    let vapor = factory(WaterRegime::Vapor);
    let supercritical = factory(WaterRegime::Supercritical);
    assert!(matches!(
        evaluate(
            &vapor,
            [300.0, 996.556 / MOLAR_MASS],
            DerivativeOrder::Value
        ),
        Err(ProviderError::Trial(_))
    ));
    assert!(matches!(
        evaluate(&liquid, [500.0, 250.0], DerivativeOrder::Value),
        Err(ProviderError::Trial(_))
    ));
    // At 400 K and 10 kg/m3 the equilibrium water state is two phase.
    for factory in [&liquid, &vapor, &supercritical] {
        assert!(matches!(
            evaluate(factory, [400.0, 10.0 / MOLAR_MASS], DerivativeOrder::Value),
            Err(ProviderError::Trial(_))
        ));
    }
    assert!(matches!(
        evaluate(&vapor, [250.0, 250.0], DerivativeOrder::Value),
        Err(ProviderError::OutsideEnvelope { .. })
    ));
    let sc = evaluate(
        &supercritical,
        [650.0, 500.0 / MOLAR_MASS],
        DerivativeOrder::Second,
    )
    .unwrap();
    assert!(sc.values[0] > 22_064_000.0);
    // Independent industrial formulation, IAPWS R7-97(2012), Table 33:
    // https://www.iapws.org/relguide/IF97-Rev.pdf . This deliberately allows
    // 0.02% cross-formulation agreement, not exact IF97 realization by HEOS95.
    near(sc.values[0], 25_583_701.8, 0.0, 2e-4);
    near(
        (sc.values[1] - supercritical.reference.h_offset) / MOLAR_MASS,
        1_863_430.19,
        0.0,
        2e-4,
    );
    near(
        (sc.values[2] - supercritical.reference.s_offset) / MOLAR_MASS,
        4_054.272_73,
        0.0,
        2e-4,
    );
    let mut worker = vapor.create().unwrap();
    let request = worker
        .prepare(ProviderRequest::all(worker.spec(), DerivativeOrder::Value))
        .unwrap();
    let mut buffer = ProviderBuffer::new(&request);
    let cancelled = AtomicBool::new(true);
    let context = EvaluationContext {
        cancelled: &cancelled,
        max_result_bytes: 4096,
    };
    assert!(matches!(
        buffer.fill(
            &mut worker,
            &[500.0, 250.0],
            &request,
            ProviderRuntime::default(),
            &context
        ),
        Err(ProviderError::Cancelled)
    ));
}

#[test]
fn fresh_workers_preserve_point_independence_and_prepared_configuration_identity() {
    let factory = factory(WaterRegime::Vapor);
    let first = evaluate(&factory, [500.0, 250.0], DerivativeOrder::Second).unwrap();
    let other = evaluate(&factory, [800.0, 1500.0], DerivativeOrder::Second).unwrap();
    assert_ne!(first.values, other.values);
    assert_eq!(
        first,
        evaluate(&factory, [500.0, 250.0], DerivativeOrder::Second).unwrap()
    );
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let prepared = registration(&registry, WaterRegime::Vapor).unwrap();
    assert_eq!(prepared.spec().key(), factory.spec.key());
    assert_eq!(prepared.configuration_key(), factory.configuration_key());
}
