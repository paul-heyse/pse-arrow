// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Correlation forms with dimensioned coefficients type by monomial against declared kinds
//! (ADR-0124, Plan 23 KR2).
use crate::kernel_types::{physical, source};
use crate::*;
use pse_ids::SemanticId;
use pse_quantity::{KindFactor, QuantityRegistry, QuantityTypeId, Ratio, scheme::Scheme};
use std::collections::{BTreeMap, BTreeSet};

fn id(hex: &str) -> QuantityTypeId {
    QuantityTypeId::from_id(SemanticId::parse_hex(hex).unwrap())
}
const MOLAR_CP: &str = "cd653ba98fa94d16b5d66b363f21c3d6";
const DELTA_H: &str = "d5bb3d48b9804f2f8d5a6f0a7cadaee8";

fn names() -> (QuantityRegistry, BTreeMap<String, QuantityTypeId>) {
    let (registry, mut names) = physical();
    names.insert("MolarCp".into(), id(MOLAR_CP));
    names.insert("DeltaH".into(), id(DELTA_H));
    (registry, names)
}
fn preconditions() -> pse_quantity::PhysicalPreconditions {
    pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
        .unwrap()
}
/// The canonical monomial of a quantity type's kind, over (kind name, exponent).
fn monomial(registry: &QuantityRegistry, ty: &Type) -> Vec<KindFactor> {
    let Type::Quantity(Scheme::Concrete(quantity)) = ty else {
        panic!("{ty:?} is not a concrete quantity");
    };
    registry
        .kind_monomial(registry.quantity_type(*quantity).unwrap().key.kind)
        .unwrap()
}
fn heat_capacity_over_temperature(registry: &QuantityRegistry, power: i32) -> Vec<KindFactor> {
    let temperature = registry
        .quantity_type(id("c64b96975a4a59755f8711d3bf628bc9"))
        .unwrap()
        .key
        .kind;
    let cp = registry.quantity_type(id(MOLAR_CP)).unwrap().key.kind;
    let mut factors = vec![
        KindFactor {
            kind: cp,
            exponent: Ratio::ONE,
        },
        KindFactor {
            kind: temperature,
            exponent: Ratio::new(-power, 1).unwrap(),
        },
    ];
    factors.sort();
    factors
}

const FORMS: &str = "package forms {
fn dippr100_cp(T: Temperature, c1: MolarCp, c2: MolarCp/Temperature, c3: MolarCp/Temperature^2, c4: MolarCp/Temperature^3, c5: MolarCp/Temperature^4) -> MolarCp
  = c1 + c2*T + c3*T^2 + c4*T^3 + c5*T^4;
fn dippr100_dh(T0: Temperature, T: Temperature, c1: MolarCp, c2: MolarCp/Temperature, c3: MolarCp/Temperature^2, c4: MolarCp/Temperature^3, c5: MolarCp/Temperature^4) -> DeltaH
  = c1*(T - T0) + c2*T^2/2 - c2*T0^2/2 + c3*T^3/3 - c3*T0^3/3 + c4*T^4/4 - c4*T0^4/4 + c5*T^5/5 - c5*T0^5/5;
fn shomate_cp(T: Temperature, a: MolarCp, b: MolarCp/Temperature, c: MolarCp/Temperature^2, d: MolarCp/Temperature^3, e: MolarCp*Temperature^2) -> MolarCp
  = a + b*T + c*T^2 + d*T^3 + e/T^2;
fn shomate_dh(T0: Temperature, T: Temperature, a: MolarCp, b: MolarCp/Temperature, c: MolarCp/Temperature^2, d: MolarCp/Temperature^3, e: MolarCp*Temperature^2) -> DeltaH
  = a*(T - T0) + b*T^2/2 - b*T0^2/2 + c*T^3/3 - c*T0^3/3 + d*T^4/4 - d*T0^4/4 - e/T + e/T0;
fn rpp4_cp(T: Temperature, a: MolarCp, b: MolarCp/Temperature, c: MolarCp/Temperature^2, d: MolarCp/Temperature^3) -> MolarCp
  = a + b*T + c*T^2 + d*T^3;
fn rpp4_dh(T0: Temperature, T: Temperature, a: MolarCp, b: MolarCp/Temperature, c: MolarCp/Temperature^2, d: MolarCp/Temperature^3) -> DeltaH
  = a*T - a*T0 + b*T^2/2 - b*T0^2/2 + c*T^3/3 - c*T0^3/3 + d*T^4/4 - d*T0^4/4;
}";

#[test]
fn seed_forms_type_without_intermediate_kinds() {
    let (registry, names) = names();
    let preconditions = preconditions();
    let context = TypeContext {
        preconditions: &preconditions,
        quantities: &registry,
        names: &names,
    };
    let checked = check(&source(FORMS), &context).unwrap();
    let function = |name: &str| &checked.functions[&checked.entry(name).unwrap()];
    for (name, result) in [
        ("forms.dippr100_cp", MOLAR_CP),
        ("forms.dippr100_dh", DELTA_H),
        ("forms.shomate_cp", MOLAR_CP),
        ("forms.shomate_dh", DELTA_H),
        ("forms.rpp4_cp", MOLAR_CP),
        ("forms.rpp4_dh", DELTA_H),
    ] {
        assert_eq!(
            function(name).result,
            Type::Quantity(Scheme::Concrete(id(result))),
            "{name}"
        );
    }
    // The coefficients are dimensioned: each is a declared derived kind's type.
    let dippr = function("forms.dippr100_cp");
    for (position, power) in [(2, 1), (3, 2), (4, 3), (5, 4)] {
        assert_eq!(
            monomial(&registry, &dippr.arguments[position].1),
            heat_capacity_over_temperature(&registry, power),
            "c{position}"
        );
    }
    assert_eq!(
        monomial(&registry, &function("forms.shomate_cp").arguments[5].1),
        heat_capacity_over_temperature(&registry, -2)
    );
    // No intermediate kind exists for the powers of temperature the terms pass through.
    let temperature = heat_capacity_over_temperature(&registry, 1)
        .into_iter()
        .find(|f| f.exponent != Ratio::ONE)
        .unwrap()
        .kind;
    for power in 2..=5 {
        assert_eq!(
            registry.kind_by_monomial(&[KindFactor {
                kind: temperature,
                exponent: Ratio::new(power, 1).unwrap(),
            }]),
            None
        );
    }
    // Standing alone, such a power is refused with its monomial, never synthesized.
    let error = check(
        &source("package cube { fn cube(T: Temperature) -> Scalar = T^3; }"),
        &context,
    )
    .unwrap_err()
    .to_string();
    assert!(
        error.contains("no declared quantity kind has the monomial"),
        "{error}"
    );
}

#[test]
fn type_expression_resolves_by_monomial() {
    let (registry, names) = names();
    let preconditions = preconditions();
    let context = TypeContext {
        preconditions: &preconditions,
        quantities: &registry,
        names: &names,
    };
    let at = DeclarationId::from(SemanticId::NIL);
    let resolve = |text: &str| context.resolve(text, &BTreeSet::new(), &BTreeMap::new(), at);
    let squared = resolve("MolarCp/Temperature^2").unwrap();
    assert_eq!(
        monomial(&registry, &squared),
        heat_capacity_over_temperature(&registry, 2)
    );
    // Spellings of one monomial are one type, with the factors' molar basis.
    assert_eq!(resolve("MolarCp/Temperature/Temperature").unwrap(), squared);
    assert_eq!(resolve("MolarCp*Temperature^-2").unwrap(), squared);
    let Type::Quantity(Scheme::Concrete(quantity)) = &squared else {
        panic!("{squared:?}");
    };
    assert_eq!(
        registry.quantity_type(*quantity).unwrap().key.basis,
        registry.quantity_type(id(MOLAR_CP)).unwrap().key.basis
    );
    assert_eq!(
        monomial(&registry, &resolve("MolarCp*Temperature^2").unwrap()),
        heat_capacity_over_temperature(&registry, -2)
    );
    // The declared molar enthalpy kind types the caloric product as an increment.
    assert_eq!(
        resolve("MolarCp*Temperature").unwrap(),
        Type::Quantity(Scheme::Concrete(id(DELTA_H)))
    );
    let error = resolve("Temperature^2").unwrap_err().to_string();
    assert!(
        error.contains("no declared quantity kind has the monomial"),
        "{error}"
    );
}

#[test]
fn static_chains_evaluate_as_a_whole() {
    // Static evaluation types a chain once at its root: T³ alone names no kind, while
    // c3·T³/3 is a declared enthalpy increment (3·(400³ − 300³)/3 J/mol = 3.7e7 J/mol).
    let (registry, names) = names();
    let preconditions = preconditions();
    let context = TypeContext {
        preconditions: &preconditions,
        quantities: &registry,
        names: &names,
    };
    let execute = |h: &str| {
        let text = format!(
            "package p {{
            def Child(c3: MolarCp/Temperature^2, T0: Temperature, T: Temperature, h: DeltaH) {{
                require h == c3*T^3/3 - c3*T0^3/3 : \"increment\";
                var x: Scalar; eq e: x == 1;
            }}
            def Root {{ child inner: Child = Child(c3=3{{J/(K^3*mol)}}, T0=300{{K}}, T=400{{K}}, h={h}); }}
        }}"
        );
        let package = check(&source(&text), &context)?;
        specialize(
            &package,
            package.names["p.Root"],
            InstanceId::from_id(SemanticId::NIL),
            &Bindings::default(),
            Limits::default(),
        )
    };
    execute("37000000{J/mol}").unwrap();
    assert!(execute("1{J/mol}").is_err());
}
