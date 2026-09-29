// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete record agreement and independent scientific checks (ADR-0064).

#![allow(
    clippy::float_cmp,
    reason = "test fixture construction and exact independent value assertions"
)]

use super::*;
use pse_quantity::{BaseDimension, QuantityOperation, QuantityRegistry};

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
                    x.definition.clone(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(units(a), units(b));
    let kinds = |r: &QuantityRegistry| {
        r.kinds()
            .map(|x| {
                (
                    x.id,
                    x.dimension,
                    x.extensive,
                    x.addition_kind,
                    x.category,
                    x.definition.clone(),
                )
            })
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
                    x.name.clone(),
                    x.kind,
                    x.temperature,
                    x.pressure,
                    x.include_enthalpy_of_formation,
                    x.subject,
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
                    x.name.clone(),
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
fn reference_package_admission_scientific_units() {
    let source = load(root(), pse_schema::registry().expect("registry")).expect("package data");
    // BIPM SI bases and NIST SP 811 Appendix B.8; checked independently of generator.
    let units = source.quantities();
    let atomic = |symbol: &str| {
        units
            .compose(&pse_quantity::UnitProduct::symbol(symbol))
            .expect(symbol)
    };
    let kelvin = atomic("K");
    assert_eq!(
        kelvin.dimension,
        pse_quantity::DimensionVector::base(BaseDimension::Temperature)
    );
    let celsius = atomic("degC");
    assert_eq!(celsius.offset_to_canonical, 273.15);
    let fahrenheit = atomic("degF");
    let point = pse_quantity::convert_spec(&fahrenheit, &kelvin, pse_quantity::ScaleKind::Point)
        .expect("point");
    assert!((pse_quantity::convert_value(&point, 32.0) - 273.15).abs() < 1e-12);
    let difference =
        pse_quantity::convert_spec(&fahrenheit, &kelvin, pse_quantity::ScaleKind::Difference)
            .expect("difference");
    assert_eq!(pse_quantity::convert_value(&difference, 18.0), 10.0);
    let psi = atomic("psig");
    let exact_factor = 0.453_592_37 * 9.806_65 / (0.0254 * 0.0254);
    assert!((psi.scale_to_canonical - exact_factor).abs() < 1e-9);
    let reference = units
        .reference_state(psi.reference_state.expect("explicit datum"))
        .expect("datum");
    // Typed conditions, read in their quantity types' canonical units (ADR-0123 Outcome 6).
    let condition = |value: Option<pse_quantity::ReferenceCondition>| {
        units.reference_condition(&value.expect("typed condition")).expect("condition")
    };
    assert_eq!(
        (condition(reference.temperature), condition(reference.pressure)),
        (298.15, 101_325.0)
    );
    let unit_set = units
        .unit_sets()
        .find(|s| s.base[7].is_none())
        .expect("physical SI");
    unit_set.validate(units).expect("all seven bases valid");
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
    for package in ["physical", "fixture-currency"] {
        let source = root().join("packages/reference").join(package);
        let target = destination.join(package);
        std::fs::create_dir_all(target.join("materials")).expect("material directory");
        for relative in ["package.toml", "materials/physical.yaml"] {
            std::fs::copy(source.join(relative), target.join(relative)).expect("source copy");
        }
    }
    let source_models = root().join("packages/reference/physical/models");
    let target_models = destination.join("physical/models");
    std::fs::create_dir_all(&target_models).expect("model directory");
    for entry in std::fs::read_dir(source_models).expect("models") {
        let entry = entry.expect("model source");
        std::fs::copy(entry.path(), target_models.join(entry.file_name())).expect("model copy");
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
    let unchanged = mutated_package("physical", |_| {});
    load(unchanged.path(), registry).expect("unchanged fixture closure admits");
    for (section, field, value) in [
        ("units", "scale_to_canonical", serde_json::json!(0.0)),
        ("reference_states", "pressure", serde_json::json!(-1.0)),
    ] {
        let scratch = mutated_package("physical", |body| body[section][0][field] = value);
        assert!(load(scratch.path(), registry).is_err(), "{section}.{field}");
    }
    // A defined unit's identity is its product identity; a changed exponent is refused.
    let composition = mutated_package("physical", |body| {
        let defined = body["units"]
            .as_array_mut()
            .expect("units")
            .iter_mut()
            .find(|unit| unit["definition"].as_array().is_some_and(|d| !d.is_empty()))
            .expect("a defined unit");
        defined["definition"][0]["num"] = serde_json::json!(5);
    });
    assert!(
        load(composition.path(), registry).is_err(),
        "defined unit composition disagrees with its identity"
    );
    let dimensions = mutated_package("physical", |body| {
        body["quantity_kinds"][0]["dimension"][0]["num"] = serde_json::json!(7);
    });
    assert!(
        load(dimensions.path(), registry).is_err(),
        "kind dimension disagrees with actual quantity canonical unit"
    );
}
