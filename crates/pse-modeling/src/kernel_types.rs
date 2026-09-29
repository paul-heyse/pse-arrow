// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use crate::*;
use pse_ids::SemanticId;
use pse_quantity::{
    QuantityTypeId,
    scheme::{Scheme, Substitution},
};
use std::collections::BTreeMap;

pub(crate) fn physical() -> (
    pse_quantity::QuantityRegistry,
    BTreeMap<String, QuantityTypeId>,
) {
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let names = [
        ("Scalar", "dc255c612cf27e30cb835377c8dafcf4"),
        ("Temperature", "c64b96975a4a59755f8711d3bf628bc9"),
        ("DeltaTemperature", "459a933fd00837bbc50372e31ac9801c"),
        ("Flow", "6df479e3be3bda77c5938727311f1063"),
        ("MassFlow", "63a6d5d8add842d4848b7ee99b39b4d7"),
        ("Enthalpy", "553e33a36c6245619508d88d585c148b"),
        ("OtherEnthalpy", "1831d0d72dc74b299ba8ecb6d4da6f53"),
    ]
    .into_iter()
    .map(|(n, id)| {
        (
            n.into(),
            QuantityTypeId::from_id(SemanticId::parse_hex(id).unwrap()),
        )
    })
    .collect();
    (registry, names)
}
pub(crate) fn source(text: &str) -> Vec<Declaration> {
    try_source(text).unwrap()
}
/// A source the parser refuses is a refusal of the run under test, not a panic.
pub(crate) fn try_source(text: &str) -> Result<Vec<Declaration>> {
    pse_authoring::language::parse(
        text,
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .map_err(|e| invalid(DeclarationId::from(SemanticId::NIL), e.to_string()))
}
#[test]
fn polymorphic_smoothing_and_complete_substitution() {
    let (registry, names) = physical();
    let c = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        names: &names,
    };
    let rows = source(
        "package p { fn smooth<Q>(a: Q, b: Q, eps: Delta<Q>) -> Q = b + ((a-b) + sqrt((a-b)^2 + eps^2))/2; }",
    );
    let checked = check(&rows, &c).unwrap();
    assert_eq!(checked.functions.len(), 1);
    let mut bindings = Substitution::new();
    let q = Scheme::Variable("Q".into());
    q.bind(names["Temperature"], &registry, &mut bindings)
        .unwrap();
    Scheme::Delta(Box::new(q.clone()))
        .bind(names["DeltaTemperature"], &registry, &mut bindings)
        .unwrap();
    assert!(q.bind(names["Flow"], &registry, &mut bindings).is_err());
    let mut bindings = Substitution::new();
    q.bind(names["Enthalpy"], &registry, &mut bindings).unwrap();
    assert!(
        q.bind(names["OtherEnthalpy"], &registry, &mut bindings)
            .is_err()
    );
}

#[test]
fn powers_keep_the_exponent_quantity_in_the_authoritative_operation() {
    let (registry, names) = physical();
    let context = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        names: &names,
    };
    check(
        &source(
            "package p { fn square(x:Scalar)->Scalar=x^2; fn root(x:Scalar)->Scalar=x^(1/3); }",
        ),
        &context,
    )
    .unwrap();
    let scalar = names["Scalar"];
    let power = Scheme::Power(
        Box::new(Scheme::Variable("Q".into())),
        pse_quantity::Ratio::new(2, 1).unwrap(),
    );
    assert_eq!(
        power
            .resolve(&registry, &BTreeMap::from([("Q".into(), scalar)]))
            .unwrap(),
        scalar
    );
    assert!(
        check(
            &source("package p { fn bad(x:Scalar,p:Flow)->Scalar=x^p; }"),
            &context
        )
        .is_err()
    );
}
#[test]
fn default_override_and_diamond_conflict() {
    let (registry, names) = physical();
    let c = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        names: &names,
    };
    let text = "package p { interface I { let f: Scalar = 1; } def D : I { override let f: Scalar = 2; } }";
    let p = check(&source(text), &c).unwrap();
    let d = p.names["p.D"];
    assert_eq!(p.declarations[&p.members[&d]["f"]].parent_id, Some(d));
    for text in [
        "package p { interface I { let f: Scalar = 1; } def D : I { let f: Scalar = 2; } }",
        "package p { interface A { let f: Scalar = 1; } interface B { let f: Scalar = 2; } def D : A, B {} }",
        "package p { interface A extends B {} interface B extends A {} }",
    ] {
        assert!(check(&source(text), &c).is_err(), "{text}");
    }
}
#[test]
fn wrong_basis_reference_and_uninstantiated_definition_are_rejected() {
    let (registry, names) = physical();
    let c = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        names: &names,
    };
    for text in [
        "package p { def NeverUsed { var a: Flow; var b: MassFlow; eq invalid: a == b; } }",
        "package p { def NeverUsed { var a: Enthalpy; var b: OtherEnthalpy; eq invalid: a == b; } }",
        "package p { fn wrong(x: Flow) -> MassFlow = x; }",
        "package p { def D { var t: Temperature; eq bad: t+t == t; } }",
    ] {
        assert!(check(&source(text), &c).is_err(), "{text}");
    }
}
#[test]
fn arbitrary_entity_kinds_are_data() {
    let (registry, names) = physical();
    let c = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        names: &names,
    };
    let p=check(&source("package p { entity kind membrane { attribute area: Scalar; } entity membrane a { area = 1 } table permeability[j: membrane]: Scalar missing optional; }"),&c).unwrap();
    assert!(matches!(p.types[&p.names["p.a"]],Type::Entity(id) if id==p.names["p.membrane"]));
}

#[test]
fn every_dataset_row_is_checked_before_instantiation() {
    let (registry, names) = physical();
    let c = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        names: &names,
    };
    let text = r#"package p { entity kind item {} entity item a {} table coeff[j: item]: { flow: Flow, count: Integer }; dataset data: coeff source "synthetic" { [a] = [2{mol/s}, 9007199254740993]; } }"#;
    let p = check(&source(text), &c).unwrap();
    let table = &p.tables[&p.names["p.coeff"]];
    let specialize::Value::Row { fields, .. } = table.rows.values().next().unwrap() else {
        panic!("row expected")
    };
    assert_eq!(
        fields["count"],
        specialize::Value::Integer(9007199254740993)
    );
    for bad in [
        text.replace("2{mol/s}", "2{kg/s}"),
        text.replace(
            "[a] = [2{mol/s}, 9007199254740993];",
            "[a] = [2{mol/s}, 1]; [a] = [3{mol/s}, 2];",
        ),
        text.replace("[a] =", "[4] ="),
    ] {
        assert!(check(&source(&bad), &c).is_err());
    }
    let printed = pse_authoring::language::render(&source(text)).unwrap();
    assert!(check(&source(&printed), &c).is_ok());
}
#[test]
fn requirements_can_read_typed_rows_and_exact_integer_counts() {
    let text = r#"package p { entity kind item {} entity item a {} table coeff[j: item]: { flow: Flow, count: Integer, next: Integer }; dataset data: coeff source "synthetic" { [a] = [2{mol/s}, 9007199254740993, 9007199254740994]; } def Root { require coeff[a].count + 1 == coeff[a].next : "exact"; } }"#;
    let (registry, names) = physical();
    let c = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        names: &names,
    };
    let p = check(&source(text), &c).unwrap();
    specialize(
        &p,
        p.names["p.Root"],
        InstanceId::from_id(SemanticId::NIL),
        &Bindings::default(),
        Limits::default(),
    )
    .unwrap();
}

#[test]
fn recursive_functions_and_implicit_captures_are_refused_before_selection() {
    let (registry, names) = physical();
    let c = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        names: &names,
    };
    for text in [
        "package p { fn f(x: Scalar) -> Scalar = f(x); }",
        "package p { fn f(x: Scalar) -> Scalar = g(x); fn g(x: Scalar) -> Scalar = f(x); }",
        "package p { def D { var x: Scalar; fn f(y: Scalar) -> Scalar = x + y; } }",
    ] {
        assert!(check(&source(text), &c).is_err());
    }
    let text = "package p { fn f(x: Scalar) -> Scalar = (b+b) where a = x*x, b = a+1; }";
    assert!(check(&source(text), &c).is_ok());
}

#[test]
fn polymorphic_functions_compose_without_dimension_only_substitution() {
    let (registry, names) = physical();
    let c = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        names: &names,
    };
    let simple =
        "package p { fn same<Q>(x:Q) -> Q = x; fn composed<R>(x:R) -> R = same(same(x)); }";
    assert!(check(&source(simple), &c).is_ok());
    let bad = "package p { fn same<Q>(x:Q,y:Q) -> Q = x; fn bad<R,S>(x:R,y:S) -> R = same(x,y); }";
    assert!(check(&source(bad), &c).is_err());
}

#[test]
fn indexed_function_types_and_partial_coordinates_are_checked_before_selection() {
    let (registry, names) = physical();
    let c = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        names: &names,
    };
    let prelude = "package p { entity kind item {} entity kind other {} entity item a {} entity other b {} set items: Set<item> = {a}; fn total<Q>(x: Q[item], members: Set<item>)->Q=sum(j in members | x[j]); ";
    assert!(check(&source(&format!("{prelude} def D {{ var x[j in items]: Flow; eq e: total(x,items) == 0{{mol/s}}; }} }}")),&c).is_ok());
    for call in [
        "partial(total,x[b])(x,items)",
        "partial(total,x)(x,items)",
        "partial(total,x[a,a])(x,items)",
    ] {
        assert!(
            check(
                &source(&format!(
                    "{prelude} def D {{ var x[j in items]: Scalar; eq e: {call} == 0; }} }}"
                )),
                &c
            )
            .is_err(),
            "{call}"
        );
    }
}

#[test]
fn indirect_function_calls_use_only_visible_immutable_package_tables() {
    let (registry, names) = physical();
    let c = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        names: &names,
    };
    let text = r#"package p { entity kind item {} entity item a {} fn double(x:Scalar)->Scalar=x*2;
 table methods[j:item]: Fn(x:Scalar)->Scalar; dataset choices: methods source "test" {[a]=[double];}
 fn captured(x:Scalar,j:item)->Scalar=methods[j](x); }"#;
    assert!(check(&source(text), &c).is_ok());
    let hidden = text.replace(
        "fn captured(x:Scalar,j:item)",
        "} package consumer { fn captured(x:Scalar,j:p.item)",
    );
    assert!(check(&source(&hidden), &c).is_err());
}

#[test]
fn package_visibility_requires_an_import_for_functions_and_types() {
    let (registry, names) = physical();
    let context = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        names: &names,
    };
    let library = "package library { entity kind item {} fn twice(x:Scalar)->Scalar=x*2; }";
    for body in [
        "fn f(x:Scalar)->Scalar=library.twice(x);",
        "def Root(x:library.item) {}",
    ] {
        assert!(
            check(
                &source(&format!("{library} package consumer {{ {body} }}")),
                &context
            )
            .is_err()
        );
    }
    let imported = format!(
        "{library} package consumer {{ use library @\"1.0.0\" as lib; fn f(x:Scalar)->Scalar=lib.twice(x); def Root(x:lib.item) {{}} }}"
    );
    let checked = check(&source(&imported), &context).unwrap();
    let consumer = checked.entry("consumer").unwrap();
    assert_eq!(
        checked.resolve(consumer, "lib.twice"),
        checked.entry("library.twice")
    );
    assert_eq!(checked.resolve(consumer, "library.twice"), None);
    let selected = checked
        .select(checked.entry("consumer.f").unwrap())
        .unwrap();
    assert_eq!(
        selected.resolve(consumer, "lib.twice"),
        checked.entry("library.twice")
    );
    let private_id = checked.entry("library.twice").unwrap();
    let bypass = format!(
        "{library} package consumer {{ fn f(x:Scalar)->Scalar=f_{}(x); }}",
        private_id.as_id().to_hex()
    );
    assert!(check(&source(&bypass), &context).is_err());
}

#[test]
fn admitted_physical_context_survives_caller_changes() {
    let (registry, mut names) = physical();
    let checked = check(
        &source("package p { def Root {var x:Scalar; eq e:x==2;} }"),
        &TypeContext {
            preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
            quantities: &registry,
            names: &names,
        },
    )
    .unwrap();
    let original = names["Scalar"];
    names.insert("Scalar".into(), names["Temperature"]);
    assert_eq!(checked.context().names["Scalar"], original);
    let model = specialize(
        &checked,
        checked.entry("p.Root").unwrap(),
        InstanceId::from_id(SemanticId::NIL),
        &Bindings::default(),
        Limits::default(),
    )
    .unwrap();
    assert_eq!(model.equations.len(), 1);
}

#[test]
fn physical_aliases_are_scoped_and_compose_in_quantity_schemes() {
    let (registry, standard) = physical();
    let names = BTreeMap::from([
        ("a.Measure".into(), standard["Temperature"]),
        ("b.Measure".into(), standard["Scalar"]),
    ]);
    let context = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        names: &names,
    };
    let source_text = "package a { fn difference(x:Measure,y:Measure)->Delta<Measure>=x-y; } package b {fn identity(x:Measure)->Measure=x;}";
    let checked = check(&source(source_text), &context).unwrap();
    assert_ne!(
        checked.functions[&checked.entry("a.difference").unwrap()].arguments[0].1,
        checked.functions[&checked.entry("b.identity").unwrap()].arguments[0].1
    );
    for ty in ["a.Measure", "Delta<a.Measure>"] {
        assert!(
            check(
                &source(&format!(
                    "package a {{}} package b {{def Root {{var x:{ty};}}}}"
                )),
                &context
            )
            .is_err()
        );
    }
    assert!(check(&source("package a {} package b {use a @\"1.0.0\" as imported; def Root {var x:imported.Measure; var y:Delta<imported.Measure>;}}"), &context).is_ok());
}

#[test]
fn constraint_forms_are_placed_and_typed_at_declaration() {
    let (registry, mut names) = physical();
    names.insert(
        "Indicator".into(),
        QuantityTypeId::from_id(SemanticId::parse_hex("b5d9e1c4a7f2483e9d6c1b0a5e8f3d27").unwrap()),
    );
    let context = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        names: &names,
    };
    let admitted = check(
        &source(
            "package p { def Root { var on: Indicator in binary; var x: Flow; eq e when on: x <= x; when true { logic l: on implies on; } disjunction d { alternative a { eq k: x == x; } alternative b { eq k: x >= x; } } } }",
        ),
        &context,
    )
    .unwrap();
    // An alternative is typed by the indicator quantity, as its binary selection.
    let alternative = admitted
        .declarations
        .values()
        .find(|r| r.name == "a")
        .unwrap()
        .declaration_id;
    assert_eq!(
        admitted.types[&alternative],
        Type::Quantity(Scheme::Concrete(names["Indicator"]))
    );
    for invalid in [
        // Forms never enter an implicit residual, a regime, a stage or an alternative.
        "package p { def Root { var on: Indicator in binary; var x: Flow; implicit i { eq e when on: x <= x; } } }",
        "package p { def Root { var on: Indicator in binary; stage s { logic l: on; } } }",
        "package p { def Root { var on: Indicator in binary; var x: Flow; disjunction d { alternative a { logic l: on; } alternative b { eq k: x >= x; } } } }",
        // An alternative lives in a disjunction, and a condition names an indicator.
        "package p { def Root { alternative a { } } }",
        "package p { def Root { var y: Flow; var x: Flow; eq e when y: x <= x; } }",
        "package p { def Root { var x: Flow; piecewise f: x == x at (x, x); } }",
        "package p { def Root { var on: Indicator in binary; logic l: on implies; } }",
    ] {
        assert!(check(&source(invalid), &context).is_err(), "{invalid}");
    }
}
