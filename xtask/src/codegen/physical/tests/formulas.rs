// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Numerical probes evaluate the actual authored syntax, independently of fixture equality.

use super::*;
use serde_json::Value;

mod evaluate;

struct Source {
    templates: Value,
    methods: Value,
    data: Value,
    properties: Value,
    physical: Physical,
}
impl Source {
    fn new() -> Self {
        let registry = pse_schema::registry().expect("registry");
        // These are ordinary strict document admissions, not arbitrary JSON fixtures.
        for name in ["methods", "thermo-examples"] {
            load_package(
                &root().join("packages/reference").join(name),
                registry,
                ParseBudget::default(),
            )
            .expect("actual package admission");
        }
        Self {
            templates: document("methods/templates/correlations.yaml"),
            methods: document("methods/methods/correlations.yaml"),
            data: document("thermo-examples/materials/species-and-data.yaml"),
            properties: document("thermo-examples/properties/packages.yaml"),
            physical: load(root(), registry).expect("admitted actual physical registry"),
        }
    }
    fn value(&self, method: &str, package: &str, species: &str, temperature: f64) -> f64 {
        let method = rows(&self.methods, "method_specs")
            .iter()
            .find(|row| row["name"] == method)
            .expect("declared method");
        let provision = rows(&self.methods, "method_provisions")
            .iter()
            .find(|row| row["method_id"] == method["method_id"])
            .expect("actual output provision");
        let expression = rows(&self.templates, "template_symbol_expressions")
            .iter()
            .find(|row| row["symbol_decl_id"] == provision["symbol_decl_id"])
            .expect("actual provision expression")["expression"]
            .as_str()
            .expect("source text");
        let species_id = &rows(&self.data, "species")
            .iter()
            .find(|row| row["name"] == species)
            .expect("actual species")["species_id"];
        let owner = &rows(&self.properties, "property_packages")
            .iter()
            .find(|row| row["name"] == package)
            .expect("actual package")["property_package_id"];
        let mut values =
            BTreeMap::from([("T".to_owned(), temperature), ("P".to_owned(), 100_000.0)]);
        for row in rows(&self.data, "parameter_values") {
            if row["owner_entity_id"] != *owner {
                continue;
            }
            let index = row["index"].as_array().expect("declared index tuple");
            if !(index.is_empty() || index.as_slice() == [species_id.clone()]) {
                continue;
            }
            let unit_id = row["unit_id"].as_str().expect("unit ID");
            let unit = self
                .physical
                .quantities()
                .units()
                .find(|unit| unit.id.as_id().to_string() == unit_id)
                .expect("admitted parameter unit");
            let value = row["value"].as_f64().expect("finite parameter");
            values.insert(
                row["parameter_kind"]
                    .as_str()
                    .expect("declared name")
                    .to_owned(),
                value * unit.scale_to_canonical + unit.offset_to_canonical,
            );
        }
        let expression = pse_authoring::dsl::parse_expr(expression).expect("actual DSL syntax");
        evaluate::expression(&expression, &values, self.physical.quantities())
    }
}
fn document(path: &str) -> Value {
    let text =
        std::fs::read_to_string(root().join("packages/reference").join(path)).expect("source");
    serde_json::from_str(&text.lines().skip(2).collect::<Vec<_>>().join("\n"))
        .expect("JSON-shaped YAML")
}
fn rows<'a>(value: &'a Value, name: &str) -> &'a Vec<Value> {
    value[name]
        .as_array()
        .expect("declared repeated source section")
}
fn close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        actual.is_finite() && (actual - expected).abs() <= tolerance,
        "actual {actual}, expected {expected}, tolerance {tolerance}"
    );
}

#[test]
fn sourced_correlations_match_independent_values_and_integral_identities() {
    let source = Source::new();
    // NIST table anchors are independent of the authored fitted coefficient vector.
    for (species, values) in [
        ("benzene", [82.44, 113.52, 139.35, 160.09]),
        ("toluene", [103.7, 139.9, 170.8, 196.2]),
    ] {
        for (temperature, expected) in [298.15, 400.0, 500.0, 600.0].into_iter().zip(values) {
            close(
                source.value("RPP4.cp", "BT_vapor", species, temperature),
                expected,
                1e-10,
            );
        }
    }
    // Perry Table 2-30 reports these densities at its stated lower endpoints.
    close(
        source.value("Perry.liquid.density", "BT_liquid", "benzene", 278.68),
        11_421.0,
        5.0,
    );
    close(
        source.value("Perry.liquid.density", "BT_liquid", "toluene", 178.18),
        10_495.0,
        5.0,
    );
    // NIST measured liquid benzene Cp at 298.15 K: 135.69 J/(mol K).
    close(
        source.value("Perry.liquid.cp", "BT_liquid", "benzene", 298.15),
        135.69,
        0.2,
    );
    for (family, package, species, temperature) in [
        ("NIST.Shomate", "N2_vapor", "nitrogen", 350.0),
        ("RPP4", "BT_vapor", "benzene", 400.0),
        ("Perry.liquid", "BT_liquid", "toluene", 325.0),
    ] {
        integral_identity(&source, family, package, species, temperature);
    }
    close(
        source.value("NIST.Shomate.h", "N2_vapor", "nitrogen", 298.15),
        0.0,
        1e-12,
    );
    close(
        source.value("NIST.Shomate.s", "N2_vapor", "nitrogen", 298.15),
        191.609,
        1e-12,
    );
    close(
        source.value("Ideal.gas.density", "N2_vapor", "nitrogen", 300.0),
        40.090_785_014_242_016,
        1e-10,
    );
}

#[test]
fn shomate_matches_published_nist_janaf_table_values() {
    let source = Source::new();
    // Primary numerical table, independent of the coefficient projection and integrator:
    // https://webbook.nist.gov/cgi/cbook.cgi?ID=C7727379&Table=on&Type=JANAFG
    // NIST publishes Cp to .01 J/(mol K), S to .1 J/(mol K), and H to .01 kJ/mol.
    // The source method subtracts its actual 298.15 K fit value before adding Href.
    for (temperature, cp, entropy, enthalpy) in [
        (300.0, 29.12, 191.8, 50.0),
        (400.0, 29.25, 200.2, 2970.0),
        (500.0, 29.58, 206.7, 5910.0),
    ] {
        close(
            source.value("NIST.Shomate.cp", "N2_vapor", "nitrogen", temperature),
            cp,
            0.005,
        );
        close(
            source.value("NIST.Shomate.s", "N2_vapor", "nitrogen", temperature),
            entropy,
            0.05,
        );
        close(
            source.value("NIST.Shomate.h", "N2_vapor", "nitrogen", temperature),
            enthalpy,
            5.1,
        );
    }
}
fn integral_identity(
    source: &Source,
    family: &str,
    package: &str,
    species: &str,
    temperature: f64,
) {
    let delta = 0.001;
    let cp = source.value(&format!("{family}.cp"), package, species, temperature);
    let derivative = |suffix: &str| {
        (source.value(
            &format!("{family}.{suffix}"),
            package,
            species,
            temperature + delta,
        ) - source.value(
            &format!("{family}.{suffix}"),
            package,
            species,
            temperature - delta,
        )) / (2.0 * delta)
    };
    close(derivative("h"), cp, 1e-6);
    // The BT vapor example deliberately does not claim an absolute gas entropy datum.
    if family != "RPP4" {
        close(derivative("s"), cp / temperature, 1e-8);
    }
}
