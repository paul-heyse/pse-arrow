// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use crate::*;
use pse_ids::SemanticId;
use pse_quantity::{
    QuantityTypeId,
    scheme::{Scheme, Substitution},
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn physical() -> (
    pse_quantity::QuantityRegistry,
    BTreeMap<String, QuantityTypeId>,
) {
    // The standard registry names its quantity types in its physical document (ADR-0123
    // Outcome 6); tests read those names back.
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let names = registry
        .physical_names()
        .filter_map(|(name, value)| match value {
            pse_quantity::PhysicalName::QuantityType(id) => Some((name.to_owned(), id)),
            pse_quantity::PhysicalName::ReferenceState(_) => None,
        })
        .collect();
    (registry, names)
}
pub(crate) fn source(text: &str) -> Vec<Declaration> {
    try_source(text).unwrap()
}
/// The provenance every kernel-test dataset names unless its test is about provenance
/// (ADR-0123 Outcome 5): a source kind carrying the provenance facet, a role enumeration
/// whose `given` member declares no facet, and one source `s`. [`try_source`] declares it in
/// package `p` when the text names `provenance(s, role.given)` and declares no roles itself.
pub(crate) const PROVENANCE: &str = "entity kind source provenance { attribute title: Text; } enum role { given } entity source s { title = \"kernel test data\" }";
/// A source the parser refuses is a refusal of the run under test, not a panic.
pub(crate) fn try_source(text: &str) -> Result<Vec<Declaration>> {
    let text = if text.contains("provenance(s, role.given)") && !text.contains("enum role") {
        text.replacen("package p {", &format!("package p {{ {PROVENANCE}"), 1)
    } else {
        text.to_owned()
    };
    pse_authoring::language::parse(
        &text,
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
        scope: &PhysicalScope::default(),
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
    q.bind(names["MolarEnthalpy"], &registry, &mut bindings).unwrap();
    assert!(
        q.bind(names["PrOracleEnthalpy"], &registry, &mut bindings)
            .is_err()
    );
}

#[test]
fn powers_keep_the_exponent_quantity_in_the_authoritative_operation() {
    let (registry, names) = physical();
    let context = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        scope: &PhysicalScope::default(),
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
    let (registry, _) = physical();
    let c = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        scope: &PhysicalScope::default(),
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
    let (registry, _) = physical();
    let c = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        scope: &PhysicalScope::default(),
    };
    for text in [
        "package p { def NeverUsed { var a: Flow; var b: VolumeFlow; eq invalid: a == b; } }",
        "package p { def NeverUsed { var a: MolarEnthalpy; var b: PrOracleEnthalpy; eq invalid: a == b; } }",
        "package p { fn wrong(x: Flow) -> VolumeFlow = x; }",
        "package p { def D { var t: Temperature; eq bad: t+t == t; } }",
    ] {
        assert!(check(&source(text), &c).is_err(), "{text}");
    }
}
#[test]
fn arbitrary_entity_kinds_are_data() {
    let (registry, _) = physical();
    let c = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        scope: &PhysicalScope::default(),
    };
    let p=check(&source("package p { entity kind membrane { attribute area: Scalar; } entity membrane a { area = 1 } table permeability[j: membrane]: Scalar missing optional; }"),&c).unwrap();
    assert!(matches!(p.types[&p.names["p.a"]],Type::Entity(id) if id==p.names["p.membrane"]));
}

#[test]
fn every_dataset_row_is_checked_before_instantiation() {
    let (registry, _) = physical();
    let c = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        scope: &PhysicalScope::default(),
    };
    let text = r#"package p { entity kind item {} entity item a {} set items: Set<item> = {a}; table coeff[j: item]: { flow: Flow, count: Integer } complete_over(j in items); dataset data: coeff provenance(s, role.given) { [a] = [2{mol/s}, 9007199254740993]; } }"#;
    let p = check(&source(text), &c).unwrap();
    let table = &p.tables[&p.names["p.coeff"]];
    // Rows are positional cells in column order.
    assert_eq!(&*table.names, ["flow", "count"]);
    assert_eq!(
        table.rows.values().next().unwrap().cells[1],
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
    let text = r#"package p { entity kind item {} entity item a {} set items: Set<item> = {a}; table coeff[j: item]: { flow: Flow, count: Integer, next: Integer } complete_over(j in items); dataset data: coeff provenance(s, role.given) { [a] = [2{mol/s}, 9007199254740993, 9007199254740994]; } def Root { require coeff[a].count + 1 == coeff[a].next : "exact"; } }"#;
    let (registry, _) = physical();
    let c = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        scope: &PhysicalScope::default(),
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
    let (registry, _) = physical();
    let c = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        scope: &PhysicalScope::default(),
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
    let (registry, _) = physical();
    let c = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        scope: &PhysicalScope::default(),
    };
    let simple =
        "package p { fn same<Q>(x:Q) -> Q = x; fn composed<R>(x:R) -> R = same(same(x)); }";
    assert!(check(&source(simple), &c).is_ok());
    let bad = "package p { fn same<Q>(x:Q,y:Q) -> Q = x; fn bad<R,S>(x:R,y:S) -> R = same(x,y); }";
    assert!(check(&source(bad), &c).is_err());
}

#[test]
fn indexed_function_types_and_partial_coordinates_are_checked_before_selection() {
    let (registry, _) = physical();
    let c = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        scope: &PhysicalScope::default(),
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
    let (registry, _) = physical();
    let c = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        scope: &PhysicalScope::default(),
    };
    let text = r#"package p { entity kind item {} entity item a {} fn double(x:Scalar)->Scalar=x*2;
 set items: Set<item> = {a};
 table methods[j:item]: Fn(x:Scalar)->Scalar complete_over(j in items); dataset choices: methods provenance(s, role.given) {[a]=[double];}
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
    let (registry, _) = physical();
    let context = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        scope: &PhysicalScope::default(),
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
    let (registry, _) = physical();
    let mut scope = PhysicalScope {
        package: Some("pse.physical".into()),
        documents: None,
    };
    let checked = check(
        &source("package p { def Root {var x:Scalar; eq e:x==2;} }"),
        &TypeContext {
            preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
            quantities: &registry,
            scope: &scope,
        },
    )
    .unwrap();
    scope.documents = Some(BTreeSet::new());
    assert_eq!(checked.context().scope.documents, None);
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

/// ADR-0123 Outcome 6: a package sees the physical names when its document is in scope,
/// unqualified and qualified by the declaring package; a package outside it does not.
#[test]
fn physical_names_are_scoped_by_document() {
    let (registry, names) = physical();
    let seen = SemanticId::from_bytes([1; 16]);
    let unseen = SemanticId::from_bytes([2; 16]);
    let scope = PhysicalScope {
        package: Some("pse.physical".into()),
        documents: Some(BTreeSet::from([seen])),
    };
    let context = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        scope: &scope,
    };
    let rows = |document: SemanticId, text: &str| {
        pse_authoring::language::parse(
            text,
            document,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap()
    };
    for ty in ["Temperature", "pse.physical.Temperature", "Delta<Temperature>"] {
        let text = format!("package a {{ fn f(x: {ty}) -> {ty} = x; }}");
        let checked = check(&rows(seen, &text), &context).unwrap();
        if ty != "Delta<Temperature>" {
            assert_eq!(
                checked.functions[&checked.entry("a.f").unwrap()].result,
                Type::Quantity(Scheme::Concrete(names["Temperature"])),
                "{ty}"
            );
        }
        let refused = check(&rows(unseen, &text), &context).unwrap_err();
        assert!(refused.to_string().contains("unknown type"), "{ty}: {refused}");
    }
}

/// ADR-0123 Outcome 6: a package that sees the physical names cannot declare one of them;
/// the name would be ambiguous.
#[test]
fn ambiguous_quantity_name_refused() {
    let (registry, _) = physical();
    let context = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        scope: &PhysicalScope::default(),
    };
    for declaration in [
        "enum Temperature { low, high }",
        "def Pressure {}",
        "fn stock() -> Scalar = 1;",
        "use other @\"1.0.0\" as Scalar;",
    ] {
        let text = format!("package other {{}} package p {{ {declaration} }}");
        let error = check(&source(&text), &context).unwrap_err().to_string();
        assert!(error.contains("ambiguous name"), "{declaration}: {error}");
    }
    // Control: a member of a definition is lexical and shadows the physical name.
    assert!(check(&source("package p { def D { param Temperature: Scalar = 1; } }"), &context).is_ok());
}

#[test]
fn constraint_forms_are_placed_and_typed_at_declaration() {
    let (registry, names) = physical();
    let context = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        scope: &PhysicalScope::default(),
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
