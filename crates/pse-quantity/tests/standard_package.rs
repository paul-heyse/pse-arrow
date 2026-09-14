// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![cfg(feature = "fixtures")]
//! Required standard physical compositions (blueprint §8.3).
use pse_quantity::infer::{OpRequest, Operand, OperationSelection, infer, infer_with_evidence};
use pse_quantity::standard::{StandardInvariantChecker, ids, standard_registry};
use pse_quantity::{IndexSet, ScaleKind, convert_spec, convert_value};
#[test]
fn six_required_compositions_and_swapped_multiplication_have_declared_results() {
    let registry = standard_registry().expect("admitted standard fixture");
    let scalar = IndexSet::new();
    for (request, left, right, result) in [
        (
            OpRequest::Mul,
            "molar_flow",
            "molar_enthalpy.point",
            "energy_flow",
        ),
        (
            OpRequest::Mul,
            "molar_flow",
            "mole_fraction",
            "component_flow",
        ),
        (
            OpRequest::Mul,
            "gas_constant",
            "temperature.point",
            "molar_energy",
        ),
        (
            OpRequest::Div,
            "pressure.absolute",
            "molar_energy",
            "molar_density",
        ),
        (
            OpRequest::Div,
            "temperature.point",
            "temperature_scale",
            "neutral",
        ),
        (
            OpRequest::Div,
            "activation_energy",
            "molar_energy",
            "neutral",
        ),
    ] {
        let operands = [
            Operand {
                quantity_type: ids::quantity(left),
                indices: &scalar,
            },
            Operand {
                quantity_type: ids::quantity(right),
                indices: &scalar,
            },
        ];
        let inferred =
            infer_with_evidence(&request, &operands, &registry, &StandardInvariantChecker)
                .expect("declared composition");
        assert_eq!(inferred.result, ids::quantity(result));
        if matches!(request, OpRequest::Mul) {
            let reverse = infer_with_evidence(
                &request,
                &[operands[1], operands[0]],
                &registry,
                &StandardInvariantChecker,
            )
            .expect("swapped matching");
            assert_eq!(reverse.result, inferred.result);
            assert!(
                matches!(reverse.selected,OperationSelection::Registered {operand_permutation,..} if operand_permutation==vec![1,0])
            );
        }
    }
}
#[test]
fn reference_basis_and_missing_rule_obligations_do_not_disappear() {
    let registry = standard_registry().expect("fixture");
    let scalar = IndexSet::new();
    let flow = Operand {
        quantity_type: ids::quantity("molar_flow"),
        indices: &scalar,
    };
    let h = Operand {
        quantity_type: ids::quantity("molar_enthalpy.point"),
        indices: &scalar,
    };
    assert!(infer(&OpRequest::Mul, &[flow, h], &registry).is_err());
    let energy = infer_with_evidence(
        &OpRequest::Mul,
        &[flow, h],
        &registry,
        &StandardInvariantChecker,
    )
    .expect("basis check");
    let ty = registry.quantity_type(energy.result).expect("type");
    assert_eq!(ty.key.reference_state, Some(ids::reference("standard")));
    assert_eq!(ty.key.basis, None);
    let absolute = Operand {
        quantity_type: ids::quantity("pressure.absolute"),
        indices: &scalar,
    };
    let gauge = Operand {
        quantity_type: ids::quantity("pressure.gauge"),
        indices: &scalar,
    };
    assert!(infer(&OpRequest::Sub, &[absolute, gauge], &registry).is_err());
    assert!(infer(&OpRequest::Mul, &[flow, absolute], &registry).is_err());
}
#[test]
fn log_composition_keeps_its_declared_subject_and_currency_ratios_are_explicit() {
    let registry = standard_registry().expect("fixture");
    let scalar = IndexSet::new();
    let operand = Operand {
        quantity_type: ids::quantity("mole_fraction"),
        indices: &scalar,
    };
    let inferred = infer(
        &OpRequest::Transcendental(pse_quantity::Opcode::Log),
        &[operand],
        &registry,
    )
    .expect("declared log result");
    assert_eq!(inferred.result, ids::quantity("log_mole_fraction"));
    assert_eq!(
        registry
            .quantity_type(inferred.result)
            .expect("result")
            .key
            .subject_kind,
        registry
            .quantity_type(operand.quantity_type)
            .expect("input")
            .key
            .subject_kind
    );
    let spec = convert_spec(
        registry.unit(ids::unit("USD_2001")).expect("year"),
        registry.unit(ids::unit("USD_2000")).expect("basis"),
        ScaleKind::Point,
    )
    .expect("synthetic indices500/1000");
    assert_eq!(convert_value(&spec, 3.0).to_bits(), 6.0_f64.to_bits());
}

#[test]
fn gauge_datum_is_applied_exactly_once_after_context_checked_representation_conversion() {
    let registry = standard_registry().expect("fixture");
    let gauge = registry
        .quantity_type(ids::quantity("pressure.gauge"))
        .expect("gauge contract");
    let absolute = registry
        .quantity_type(ids::quantity("pressure.absolute"))
        .expect("absolute contract");
    let psig = registry
        .unit(ids::unit("psig"))
        .expect("datum-bearing spelling");
    let pa = registry.unit(ids::unit("Pa")).expect("Pa");
    assert!(convert_spec(psig, pa, ScaleKind::Point).is_err());
    assert!(pse_quantity::convert_spec_for_type(psig, pa, &absolute.key).is_err());
    let representation =
        pse_quantity::convert_spec_for_type(psig, pa, &gauge.key).expect("same gauge context");
    assert_eq!(representation.offset.to_bits(), 0.0_f64.to_bits());
    let gauge_pa = convert_value(&representation, 0.0);
    assert_eq!(gauge_pa.to_bits(), 0.0_f64.to_bits());
    let datum = registry
        .conversion(ids::conversion("gauge_to_absolute"))
        .expect("explicit datum conversion");
    let absolute_pa = gauge_pa * datum.scale.expect("scale") + datum.offset.expect("offset");
    assert_eq!(absolute_pa.to_bits(), 101_325.0_f64.to_bits());
    // A unit-only conversion never applies this second semantic operation implicitly.
    assert_eq!(representation.to, pa.id);
    let restored = pse_quantity::convert_spec_for_type(pa, psig, &gauge.key)
        .expect("reverse representation in same datum");
    assert_eq!(
        convert_value(&restored, gauge_pa).to_bits(),
        0.0_f64.to_bits()
    );
}

#[test]
fn smoothing_tolerances_scale_declared_units_without_adding_affine_origins() {
    use pse_quantity::{Opcode, smoothing::resolve_epsilon};
    let original = standard_registry().unwrap();
    let mut builder = original.to_builder();
    let mut pressure = original
        .quantity_type(ids::quantity("pressure.absolute"))
        .unwrap()
        .clone();
    pressure.id = ids::quantity("pressure.difference");
    pressure.key.scale_kind = ScaleKind::Difference;
    builder.quantity_type(pressure);
    let mut kpa = original.unit(ids::unit("Pa")).unwrap().clone();
    kpa.id = ids::unit("kPa");
    kpa.symbol = "kPa".to_owned();
    kpa.scale_to_canonical = 1000.0;
    builder.unit(kpa);
    let registry = builder.build().unwrap();
    let temperature = ids::quantity("temperature.point");
    assert!(
        (resolve_epsilon(
            Opcode::SmoothMax,
            temperature,
            1.8,
            Some(ids::unit("degF")),
            &registry
        )
        .unwrap()
            - 1.0)
            .abs()
            < 1e-12
    );
    assert_eq!(
        resolve_epsilon(
            Opcode::SmoothMin,
            temperature,
            2.0,
            Some(ids::unit("degC")),
            &registry
        )
        .unwrap()
        .to_bits(),
        2.0_f64.to_bits()
    );
    assert_eq!(
        resolve_epsilon(
            Opcode::SmoothMax,
            ids::quantity("pressure.absolute"),
            0.25,
            Some(ids::unit("kPa")),
            &registry
        )
        .unwrap()
        .to_bits(),
        250.0_f64.to_bits()
    );
    assert_eq!(
        resolve_epsilon(Opcode::SmoothAbs, temperature, 3.0, None, &registry)
            .unwrap()
            .to_bits(),
        3.0_f64.to_bits()
    );
    assert!(
        resolve_epsilon(
            Opcode::SmoothAbs,
            temperature,
            1.0,
            Some(ids::unit("Pa")),
            &registry
        )
        .is_err()
    );
    assert!(resolve_epsilon(Opcode::SafeLog, temperature, 1.0, None, &registry).is_err());
    assert_eq!(
        resolve_epsilon(
            Opcode::SafeSqrt,
            temperature,
            1.8,
            Some(ids::unit("degF")),
            &registry
        )
        .unwrap()
        .to_bits(),
        1.0_f64.to_bits()
    );
    for value in [0.0, -1.0, f64::INFINITY, f64::NAN] {
        assert!(
            resolve_epsilon(
                Opcode::SmoothAbs,
                temperature,
                value,
                Some(ids::unit("K")),
                &registry
            )
            .is_err()
        );
    }
}

#[test]
fn smoothing_never_selects_tolerance_by_dimension_instead_of_exact_kind_basis_and_reference() {
    use pse_quantity::{
        Opcode,
        smoothing::{resolve_epsilon, tolerance_type},
    };
    let original = standard_registry().unwrap();
    let mut builder = original.to_builder();
    let mut point = original
        .quantity_type(ids::quantity("temperature.point"))
        .unwrap()
        .clone();
    point.id = ids::quantity("referenced_temperature.point");
    point.key.reference_state = Some(ids::reference("standard"));
    let point_id = point.id;
    builder.quantity_type(point.clone());
    let mut expected = point;
    expected.id = ids::quantity("referenced_temperature.difference");
    expected.key.scale_kind = ScaleKind::Difference;
    let mut wrong_kind = original.kind(ids::kind("temperature")).unwrap().clone();
    wrong_kind.id = ids::kind("different_temperature_kind");
    builder.kind(wrong_kind.clone());
    let mut wrong = expected.clone();
    wrong.id = ids::quantity("wrong_tolerance.kind");
    wrong.key.kind = wrong_kind.id;
    builder.quantity_type(wrong);
    let mut wrong = expected.clone();
    wrong.id = ids::quantity("wrong_tolerance.basis");
    wrong.key.basis = Some(ids::basis("molar"));
    builder.quantity_type(wrong);
    // The original difference has the wrong (absent) reference; none of these qualify.
    let registry = builder.build().unwrap();
    assert!(tolerance_type(Opcode::SmoothMax, point_id, &registry).is_err());
    assert!(
        resolve_epsilon(
            Opcode::SmoothMax,
            point_id,
            1.0,
            Some(ids::unit("K")),
            &registry
        )
        .is_err()
    );
    let mut builder = registry.to_builder();
    let expected_id = expected.id;
    builder.quantity_type(expected);
    let registry = builder.build().unwrap();
    assert_eq!(
        tolerance_type(Opcode::SmoothMax, point_id, &registry).unwrap(),
        expected_id
    );
    assert_eq!(
        resolve_epsilon(
            Opcode::SmoothMax,
            point_id,
            2.0,
            Some(ids::unit("degC")),
            &registry
        )
        .unwrap()
        .to_bits(),
        2.0_f64.to_bits()
    );
}
