// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Kernel gaps the thermodynamic domain schema closes (Plan 23 D0): kind-level derived
//! attributes, kind-level requirements and attribute-level uniqueness. KR5 gave tables these
//! constraints; entity kinds now have them too. An admission-time expression ranges over a
//! kind's admitted extent by the kind's name, and `missing` states absence.
use crate::kernel_types::{physical, try_source};
use crate::specialize::Value;
use crate::*;

fn admitted(text: &str) -> Result<CheckedPackage> {
    let (registry, _) = physical();
    let context = TypeContext {formula_authority: None,
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        scope: &PhysicalScope::default(),
    };
    check(&try_source(text)?, &context)
}
fn refusal(text: &str) -> String {
    match admitted(text) {
        Ok(_) => panic!("admitted: {text}"),
        Err(error) => error.to_string(),
    }
}
fn specialized(text: &str, root: &str) -> Result<SpecializedModel> {
    let p = admitted(text)?;
    specialize(
        &p,
        p.names[root],
        InstanceId::from_bytes([7; 16]),
        &Bindings::default(),
        Limits::default(),
    )
}
fn magnitude(value: &Value) -> f64 {
    value.scalar(DeclarationId::from(SemanticId::NIL)).unwrap()
}

const FORMULA: &str = r#"package p {
 entity kind element { attribute weight: MolarMass?; }
 entity element h { weight = 0.001{kg/mol} }
 entity element o { weight = 0.016{kg/mol} }
 entity element x {}
 entity kind species {
  attribute charge: Scalar = 0;
  derived mass: MolarMass? = if sum(e in element | formula[self, e]) > 0{1} then sum(e in element | if present(e.weight) then formula[self, e] * e.weight else 0{kg/mol}) else missing;
 }
 entity kind ion extends species {}
 table formula[j: species, e: element]: Count missing default 0{1} require present(e.weight);
 entity species water {}
 entity ion hydroxide { charge = -1 }
 entity species unknown {}
 dataset f: formula provenance(s, role.given) { [water, h] = [2{1}]; [water, o] = [1{1}]; [hydroxide, h] = [1{1}]; [hydroxide, o] = [1{1}]; }
}"#;

/// A derived attribute is evaluated once per entity of its kind and its refinements, after
/// every table is admitted, with the entity as `self`, its attributes by name and each
/// visible kind's extent by the kind's name. No entity supplies it and no kind binds it;
/// derivations that read one another are refused, and `missing` states an absent value.
#[test]
fn kind_derived_attribute_is_evaluated_once_per_entity() {
    let p = admitted(FORMULA).unwrap();
    let mass = |name: &str| p.record(p.names[name]).unwrap().values["mass"].clone();
    assert!((magnitude(&mass("p.water")) - 0.018).abs() < 1e-15);
    // The refinement inherits the derivation.
    assert!((magnitude(&mass("p.hydroxide")) - 0.017).abs() < 1e-15);
    assert_eq!(mass("p.unknown"), Value::Missing);
    // A root reads the derived value like any attribute.
    let root = FORMULA.replace(
        "entity species water {}",
        "entity species water {} fn weight_of(j: species) -> MolarMass = if present(j.mass) then j.mass else 0{kg/mol}; def Root { var y: MolarMass; eq e: y == 2 * weight_of(water); }",
    );
    assert_eq!(specialized(&root, "p.Root").unwrap().equations.len(), 1);
    for (from, to, expected) in [
        // Supplied by an entity, or bound by a refinement.
        (
            "entity species unknown {}",
            "entity species unknown { mass = 0.001{kg/mol} }",
            "attribute mass is derived by kind species; no entity supplies it",
        ),
        (
            "entity kind ion extends species {}",
            "entity kind ion extends species { mass = 0.001{kg/mol}; }",
            "attribute mass is derived by kind species; a kind binds none",
        ),
        // Typed before any entity is evaluated.
        (
            "derived mass: MolarMass? =",
            "derived mass: Temperature? =",
            "derived attribute mass of kind species",
        ),
        // A requirement of the table it reads still holds.
        (
            "entity element x {}",
            "entity element x {} dataset g: formula provenance(s, role.given) { [unknown, x] = [1{1}]; }",
            "violates the requirement `present(e.weight)`",
        ),
    ] {
        let changed = FORMULA.replacen(from, to, 1);
        assert_ne!(changed, FORMULA, "{from}");
        let error = refusal(&changed);
        assert!(error.contains(expected), "{to}: {error}");
    }
    // Derivations that read one another are refused with the attributes named.
    let error = refusal(
        "package p { entity kind k { derived a: Scalar = b + 1; derived b: Scalar = a + 1; } entity k z {} }",
    );
    assert!(
        error.contains("derived attributes k.a, k.b derive from one another"),
        "{error}"
    );
    // One derivation may read another, of its own entity or of another.
    let chained = admitted(
        "package p { entity kind k { attribute v: Scalar; derived b: Scalar = a + v; derived a: Scalar = 2 * v; } entity k z { v = 1 } }",
    )
    .unwrap();
    assert_eq!(
        magnitude(&chained.record(chained.names["p.z"]).unwrap().values["b"]),
        3.
    );
    // A table's derived column is evaluated before the kinds' derived attributes.
    let error = refusal(&FORMULA.replace(
        "entity species unknown {}",
        "entity species unknown {} table heavy[j: species]: {derived m: MolarMass? = j.mass} complete_over(j); dataset hv: heavy provenance(s, role.given) { [water] = []; }",
    ));
    assert!(
        error.contains(
            "derived attribute mass of kind species is evaluated once every table is admitted"
        ),
        "{error}"
    );
    // A kind's extent is admission data; a root's model does not range over it.
    let error = specialized(
        &FORMULA.replace(
            "entity species unknown {}",
            "entity species unknown {} def Root { var y: Scalar; eq e: y == sum(j in species | j.charge); }",
        ),
        "p.Root",
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("reduction domain"), "{error}");
    // `missing` needs an optional context.
    let error =
        refusal("package p { entity kind k { derived a: Scalar = missing; } entity k z {} }");
    assert!(error.contains("derived attribute a of kind k"), "{error}");
}

/// A derivation that reads test-only data makes its entity test-only, and whatever
/// references it: a production root reading it is refused, a test fixture reads it.
#[test]
fn derived_attribute_reading_test_only_data_is_test_only() {
    let text = FORMULA
        .replace(
            "package p {",
            "package p { entity kind source provenance { attribute title: Text; } enum role { given, oracle facets(test_only) } entity source s { title = \"kernel test data\" }",
        )
        .replace("dataset f: formula provenance(s, role.given)", "dataset f: formula provenance(s, role.oracle)")
        .replace(
            "entity species unknown {}",
            "entity species unknown {} table pick[k: species]: species missing optional; dataset chosen: pick provenance(s, role.given) { [unknown] = [water]; }
 fn weight_of(j: species) -> MolarMass = if present(j.mass) then j.mass else 0{kg/mol};
 def Root { var y: MolarMass; eq e: y == weight_of(water); }
 test Check { var y: MolarMass; eq e: y == weight_of(water); }",
        );
    let p = admitted(&text).unwrap();
    assert!(p.is_test_only(p.names["p.water"]));
    assert!(!p.is_test_only(p.names["p.unknown"]));
    let pick = &p.tables[&p.names["p.pick"]];
    assert!(pick.rows.values().all(|row| row.test_only));
    let error = specialized(&text, "p.Root").unwrap_err().to_string();
    assert!(
        error.contains("test-only data is read outside a test fixture: entity water referencing test-only data, read by root Root"),
        "{error}"
    );
    specialized(&text, "p.Check").unwrap();
}

/// A requirement declared in a kind holds for every entity of the kind and its refinements,
/// declared entities and keyed rows alike; a refusal names the entity, the kind, the clause
/// and its message.
#[test]
fn kind_requirement_names_the_entity_and_clause() {
    let text = r#"package p {
 entity kind species { attribute charge: Scalar = 0; }
 entity kind apparent extends species {
  require sum(j in species | split[self, j] * j.charge) == charge : "the products' charges sum to the apparent charge";
 }
 entity kind salt extends apparent {}
 table split[a: apparent, j: species]: Count missing default 0{1};
 entity species na { charge = 1 }
 entity species cl { charge = -1 }
 entity salt nacl {}
 dataset d: split provenance(s, role.given) { [nacl, na] = [1{1}]; [nacl, cl] = [1{1}]; }
 entity kind band { key name: Text; attribute low: Scalar; attribute high: Scalar; require low < high : "bounds are ordered"; }
 dataset b: band provenance(s, role.given) { ["a"] = [1, 2]; }
}"#;
    admitted(text).unwrap();
    let error = refusal(&text.replace("[nacl, cl] = [1{1}]", "[nacl, cl] = [2{1}]"));
    for part in [
        "nacl of kind apparent violates the requirement `sum(j in species | split[self, j] * j.charge) == charge`",
        "the products' charges sum to the apparent charge",
    ] {
        assert!(error.contains(part), "{part}: {error}");
    }
    let error = refusal(&text.replace("[\"a\"] = [1, 2];", "[\"a\"] = [1, 2]; [\"b\"] = [3, 2];"));
    assert!(
        error.contains(
            "band[\"b\"] of kind band violates the requirement `low < high`: bounds are ordered"
        ),
        "{error}"
    );
    // Typed before any entity is checked.
    let error = refusal(&text.replace("require low < high", "require lower < high"));
    assert!(
        error.contains("requirement `lower < high` of kind band"),
        "{error}"
    );
}

/// An attribute declared unique holds distinct values across every entity of its kind and
/// its refinements, declared entities and keyed rows alike; absent values never collide.
#[test]
fn attribute_unique_rejects_a_shared_value() {
    let text = r#"package p {
 entity kind element { attribute symbol: Text? unique; attribute z: Integer? unique; }
 entity kind isotope extends element {}
 entity element h { symbol = "H", z = 1 }
 entity element c { symbol = "C", z = 6 }
 entity element u {}
 entity element v {}
 entity isotope d { symbol = "D" }
 entity kind form { key name: Text; attribute code: Integer unique; }
 dataset f: form provenance(s, role.given) { ["a"] = [1]; ["b"] = [2]; }
}"#;
    admitted(text).unwrap();
    for (from, to, expected) in [
        (
            "symbol = \"D\"",
            "symbol = \"H\"",
            &[
                "h and d",
                "d and h",
                "share symbol = \"H\", which kind element declares unique",
            ][..],
        ),
        (
            "[\"b\"] = [2]",
            "[\"b\"] = [1]",
            &[
                "form[\"a\"] and form[\"b\"]",
                "form[\"b\"] and form[\"a\"]",
                "share code = 1, which kind form declares unique",
            ],
        ),
        (
            "attribute z: Integer? unique;",
            "attribute z: Set<element> unique;",
            &["unique attribute z is a reference"],
        ),
    ] {
        let error = refusal(&text.replacen(from, to, 1));
        let (pair, rest) = expected.split_at(expected.len().saturating_sub(1).min(2));
        assert!(
            pair.is_empty() || pair.iter().any(|p| error.contains(p)),
            "{to}: {error}"
        );
        assert!(rest.iter().all(|p| error.contains(p)), "{to}: {error}");
    }
}
