// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete record agreement and independent scientific checks (ADR-0064).

#![allow(
    clippy::float_cmp,
    reason = "test fixture construction and exact independent value assertions"
)]

use super::*;
use pse_quantity::{BaseDimension, QuantityOperation, QuantityRegistry};

mod formulas;

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace parent")
}

#[test]
fn standard_fixture_matches_yaml() {
    let source = load(root(), pse_schema::registry().expect("registry")).expect("package data");
    let fixture = pse_quantity::standard::standard_registry().expect("generated fixture");
    assert_quantities_equal(source.quantities(), &fixture);
    assert_eq!(
        source.preconditions(),
        pse_quantity::generated::standard_preconditions()
    );
    let elements = pse_material::generated::standard_elements().expect("generated elements");
    assert_eq!(
        source.elements().elements().collect::<Vec<_>>(),
        elements.elements().collect::<Vec<_>>()
    );
}

fn assert_quantities_equal(a: &QuantityRegistry, b: &QuantityRegistry) {
    assert_eq!(a.neutral_dimensionless(), b.neutral_dimensionless());
    let units = |r: &QuantityRegistry| {
        r.units()
            .map(|x| {
                (
                    x.id,
                    x.symbol.clone(),
                    x.dimension,
                    x.scale_to_canonical.to_bits(),
                    x.offset_to_canonical.to_bits(),
                    x.is_affine,
                    x.reference_state,
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(units(a), units(b));
    let kinds = |r: &QuantityRegistry| {
        r.kinds()
            .map(|x| (x.id, x.dimension, x.extensive, x.addition_kind))
            .collect::<Vec<_>>()
    };
    assert_eq!(kinds(a), kinds(b));
    let bases = |r: &QuantityRegistry| {
        r.bases()
            .map(|x| {
                (
                    x.id,
                    x.kind,
                    x.composition_basis,
                    x.rate_basis,
                    x.reference_conditions,
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(bases(a), bases(b));
    let references = |r: &QuantityRegistry| {
        r.reference_states()
            .map(|x| {
                (
                    x.id,
                    x.kind,
                    x.temperature.map(f64::to_bits),
                    x.pressure.map(f64::to_bits),
                    x.include_enthalpy_of_formation,
                    x.phase,
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(references(a), references(b));
    let types = |r: &QuantityRegistry| {
        r.quantity_types()
            .map(|x| {
                (
                    x.id,
                    x.key.clone(),
                    x.canonical_unit,
                    x.nominal_magnitude.map(f64::to_bits),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(types(a), types(b));
    let conversions = |r: &QuantityRegistry| {
        r.conversions()
            .map(|x| {
                (
                    x.id,
                    x.from,
                    x.to,
                    x.kind,
                    x.kernel,
                    x.required_parameters.clone(),
                    x.scale.map(f64::to_bits),
                    x.offset.map(f64::to_bits),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(conversions(a), conversions(b));
    let sets = |r: &QuantityRegistry| r.unit_sets().map(|x| (x.id, x.base)).collect::<Vec<_>>();
    assert_eq!(sets(a), sets(b));
    assert_eq!(a.operations().len(), b.operations().len());
    for (left, right) in a.operations().zip(b.operations()) {
        assert_operation_equal(left, right);
    }
}

fn assert_operation_equal(a: &QuantityOperation, b: &QuantityOperation) {
    assert_eq!(
        (a.id, a.opcode, &a.input_kinds, a.result_kind),
        (b.id, b.opcode, &b.input_kinds, b.result_kind)
    );
    assert_eq!(
        (
            a.basis_rule,
            a.reference_rule,
            a.scale_rule,
            a.shape_rule,
            a.subject_rule
        ),
        (
            b.basis_rule,
            b.reference_rule,
            b.scale_rule,
            b.shape_rule,
            b.subject_rule
        )
    );
    assert_eq!(
        (
            a.basis_source,
            a.reference_source,
            a.scale_source,
            a.shape_source,
            a.subject_source
        ),
        (
            b.basis_source,
            b.reference_source,
            b.scale_source,
            b.shape_source,
            b.subject_source
        )
    );
    assert_eq!(
        (
            a.result_basis,
            a.result_reference_state,
            a.result_subject_kind
        ),
        (
            b.result_basis,
            b.result_reference_state,
            b.result_subject_kind
        )
    );
    assert_eq!(a.precondition_invariants, b.precondition_invariants);
    let conversions = |x: &QuantityOperation| {
        x.input_conversions
            .iter()
            .map(|x| (x.operand, x.conversion))
            .collect::<Vec<_>>()
    };
    assert_eq!(conversions(a), conversions(b));
}

#[test]
fn reference_package_admission_scientific_units_and_elements() {
    let source = load(root(), pse_schema::registry().expect("registry")).expect("package data");
    // BIPM SI bases and NIST SP 811 Appendix B.8; checked independently of generator.
    let units = source.quantities();
    let kelvin = units.unit_by_symbol("K").expect("K");
    assert_eq!(
        kelvin.dimension,
        pse_quantity::DimensionVector::base(BaseDimension::Temperature)
    );
    let celsius = units.unit_by_symbol("degC").expect("degC");
    assert_eq!(celsius.offset_to_canonical, 273.15);
    let fahrenheit = units.unit_by_symbol("degF").expect("degF");
    let point = pse_quantity::convert_spec(fahrenheit, kelvin, pse_quantity::ScaleKind::Point)
        .expect("point");
    assert!((pse_quantity::convert_value(&point, 32.0) - 273.15).abs() < 1e-12);
    let difference =
        pse_quantity::convert_spec(fahrenheit, kelvin, pse_quantity::ScaleKind::Difference)
            .expect("difference");
    assert_eq!(pse_quantity::convert_value(&difference, 18.0), 10.0);
    let psi = units.unit_by_symbol("psig").expect("psig");
    let exact_factor = 0.453_592_37 * 9.806_65 / (0.0254 * 0.0254);
    assert!((psi.scale_to_canonical - exact_factor).abs() < 1e-9);
    let reference = units
        .reference_state(psi.reference_state.expect("explicit datum"))
        .expect("datum");
    assert_eq!(
        (reference.temperature, reference.pressure),
        (Some(298.15), Some(101_325.0))
    );
    let unit_set = units
        .unit_sets()
        .find(|s| s.base[7].is_none())
        .expect("physical SI");
    unit_set.validate(units).expect("all seven bases valid");
    // CIAAW abridged 2024: C = 12.011 ± .002 and H = 1.0080 ± .0002 g/mol.
    let carbon = source
        .elements()
        .elements()
        .find(|e| e.symbol == "C")
        .expect("carbon");
    let hydrogen = source
        .elements()
        .elements()
        .find(|e| e.symbol == "H")
        .expect("hydrogen");
    assert!((carbon.atomic_mass - 0.012_011).abs() <= 0.012_011 * 1e-15);
    assert!((hydrogen.atomic_mass - 0.001_008).abs() <= 0.001_008 * 1e-15);
    let benzene = pse_material::molecular_weight(
        source.elements(),
        &[
            pse_material::ElementCount {
                element: carbon.id,
                count: 6.0,
            },
            pse_material::ElementCount {
                element: hydrogen.id,
                count: 6.0,
            },
        ],
    )
    .expect("composition")
    .expect("present");
    assert!((benzene - 0.078_114).abs() <= 6.0 * (0.000_002 + 0.000_000_2));
}

fn mutated_package(
    package: &str,
    mutate: impl FnOnce(&mut serde_json::Value),
) -> tempfile::TempDir {
    let scratch = tempfile::tempdir().expect("scratch");
    let destination = scratch.path().join("packages/reference");
    std::fs::create_dir_all(&destination).expect("reference root");
    std::fs::copy(
        root().join("packages/reference/fixture-projection.toml"),
        destination.join("fixture-projection.toml"),
    )
    .expect("projection selection");
    for package in ["physical", "elements", "fixture-currency"] {
        let source = root().join("packages/reference").join(package);
        let target = destination.join(package);
        std::fs::create_dir_all(target.join("materials")).expect("material directory");
        for relative in ["package.toml", "materials/physical.yaml"] {
            std::fs::copy(source.join(relative), target.join(relative)).expect("source copy");
        }
    }
    let path = destination.join(package).join("materials/physical.yaml");
    let text = std::fs::read_to_string(&path).expect("original source");
    let mut body: serde_json::Value = serde_json::from_str(
        &text
            .lines()
            .filter(|line| !line.starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .expect("JSON syntax is also YAML");
    mutate(&mut body);
    std::fs::write(
        path,
        serde_json::to_vec_pretty(&body).expect("source encoding"),
    )
    .expect("source edit");
    scratch
}

#[test]
fn altered_physical_facts_are_admitted_from_values_or_refused() {
    let registry = pse_schema::registry().expect("registry");
    for (section, field, value) in [
        ("units", "scale_to_canonical", serde_json::json!(0.0)),
        ("elements", "atomic_mass", serde_json::json!(-1.0)),
        ("reference_states", "pressure", serde_json::json!(-1.0)),
    ] {
        let package = if section == "elements" {
            "elements"
        } else {
            "physical"
        };
        let scratch = mutated_package(package, |body| body[section][0][field] = value);
        assert!(load(scratch.path(), registry).is_err(), "{section}.{field}");
    }
    let dimensions = mutated_package("physical", |body| {
        body["quantity_kinds"][0]["dimension"][0]["num"] = serde_json::json!(7);
    });
    assert!(
        load(dimensions.path(), registry).is_err(),
        "kind dimension disagrees with actual quantity canonical unit"
    );
    let original = load(root(), registry).expect("original");
    let changed = mutated_package("elements", |body| {
        let element = body["elements"]
            .as_array_mut()
            .expect("elements")
            .iter_mut()
            .find(|row| row["symbol"] == "C")
            .expect("carbon");
        element["atomic_mass"] = serde_json::json!(0.013);
    });
    let altered = load(changed.path(), registry).expect("valid changed declaration");
    let carbon = original
        .elements()
        .elements()
        .find(|e| e.symbol == "C")
        .expect("carbon")
        .id;
    let composition = [pse_material::ElementCount {
        element: carbon,
        count: 6.0,
    }];
    assert_ne!(
        pse_material::molecular_weight(original.elements(), &composition).expect("old result"),
        pse_material::molecular_weight(altered.elements(), &composition).expect("changed result"),
    );
}
