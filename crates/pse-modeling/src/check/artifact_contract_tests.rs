// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded A05 artifact-granularity probes; these introduce no persistent cache API.
use super::*;
use crate::portable::FunctionRecord;

const SOURCE: &str = r#"package p {
 entity kind source provenance {attribute title:Text;}
 enum role {given}
 entity source s {title="artifact probe"}
 constant datum:Energy=2{reading} provenance(s, role.given);
 table bank[n:Integer]:Energy complete_over(n in 1..1);
 dataset supplied:bank provenance(s, role.given) {[1]=[3{J}];}
 fn law(x:Energy)->Energy=x+datum+bank[1];
 def Unit(offset:Energy=1{J}) {var x:Energy;eq e:x==law(offset);}
}
package q {use p @"1.0.0" as imported;def Root {child unit:imported.Unit=imported.Unit();}}
"#;

fn registry(scale: f64) -> pse_quantity::QuantityRegistry {
    let (registry, _) = crate::kernel_types::physical();
    let mut reading = registry
        .units()
        .find(|unit| unit.symbol == "J")
        .unwrap()
        .clone();
    reading.id = pse_quantity::UnitId::from_id(SemanticId::from_bytes([185; 16]));
    reading.symbol = "reading".into();
    reading.scale_to_canonical = scale;
    let mut builder = registry.to_builder();
    builder.unit(reading);
    builder.build().unwrap()
}
fn checked(text: &str, quantities: &pse_quantity::QuantityRegistry) -> CheckedPackage {
    let preconditions =
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .unwrap();
    let scope = crate::PhysicalScope::default();
    check(
        &crate::kernel_types::source(text),
        &TypeContext {
            admissions: None,
            formula_authority: None,
            quantities,
            preconditions: &preconditions,
            scope: &scope,
        },
    )
    .unwrap()
}
fn record(package: &CheckedPackage) -> FunctionRecord {
    FunctionRecord::capture(&package.functions[&package.entry("p.law").unwrap()])
}
fn model(package: &CheckedPackage) -> crate::SpecializedModel {
    crate::specialize(
        package,
        package.entry("q.Root").unwrap(),
        crate::InstanceId::from_bytes([186; 16]),
        &crate::Bindings::default(),
        crate::Limits::default(),
    )
    .unwrap()
}

#[test]
fn portable_function_roundtrip_retains_body_but_not_consumed_defaults_tables_or_physics() {
    let quantities = registry(1000.0);
    let package = checked(SOURCE, &quantities);
    let original = record(&package);
    let bytes = serde_json::to_vec(&original).unwrap();
    let restored: FunctionRecord = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(restored, original);
    assert!(
        restored
            .restore()
            .unwrap_err()
            .to_string()
            .contains("qualified reconstruction recipe")
    );
    let changed_body = checked(&SOURCE.replace("x+datum+", "x-datum+"), &quantities);
    let id = package.entry("p.law").unwrap();
    assert_eq!(package.types[&id], changed_body.types[&id]);
    assert_ne!(original, record(&changed_body));
    let before = model(&package);
    for (label, changed) in [
        (
            "default",
            checked(
                &SOURCE.replace("offset:Energy=1{J}", "offset:Energy=4{J}"),
                &quantities,
            ),
        ),
        (
            "table",
            checked(&SOURCE.replace("[3{J}]", "[8{J}]"), &quantities),
        ),
        (
            "physical unit interpretation",
            checked(SOURCE, &registry(2000.0)),
        ),
    ] {
        assert_eq!(
            original,
            record(&changed),
            "{label}: a function record is not the complete consumed package"
        );
        assert_ne!(package, changed, "{label}");
        let after = model(&changed);
        assert!(
            before.equations != after.equations
                || before.functions != after.functions
                || before.symbols != after.symbols,
            "{label}: the imported scientific consumer must observe the changed input"
        );
    }
}

#[test]
fn portable_function_roundtrip_does_not_supply_absent_namespace_permission() {
    let quantities = registry(1000.0);
    let package = checked(SOURCE, &quantities);
    let original = record(&package);
    let missing = SOURCE.replace(
        "child unit:imported.Unit=imported.Unit();",
        "child unit:imported.Unit=imported.Unit();var value:Energy;eq missing:value==absent;",
    );
    let context = package.context();
    let refusal = check(&crate::kernel_types::source(&missing), &context)
        .unwrap_err()
        .to_string();
    assert!(refusal.contains("absent"), "{refusal}");
    let repaired = checked(
        &missing.replace(
            "var value:Energy;",
            "param absent:Energy=1{J};var value:Energy;",
        ),
        &quantities,
    );
    assert_eq!(original, record(&repaired));
    assert!(repaired.entry("q.Root.absent").is_some());
    let bytes = serde_json::to_vec(&record(&repaired)).unwrap();
    assert_eq!(
        serde_json::from_slice::<FunctionRecord>(&bytes).unwrap(),
        original
    );
}
