// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Entities as typed records, keyed kinds and static `Ref` dispatch (ADR-0123 Outcome 2,
//! Plan 23 KR4).
use crate::kernel_types::{physical, try_source};
use crate::specialize::Value;
use crate::*;
use pse_ids::SemanticId;

fn admitted(text: &str) -> Result<CheckedPackage> {
    let (registry, _) = physical();
    let context = TypeContext {
        admissions: None,
        formula_authority: None,
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
fn specialized(text: &str) -> Result<SpecializedModel> {
    let (registry, _) = physical();
    let context = TypeContext {
        admissions: None,
        formula_authority: None,
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        scope: &PhysicalScope::default(),
    };
    let p = check(&try_source(text)?, &context)?;
    specialize(
        &p,
        p.names["p.Root"],
        InstanceId::from_bytes([7; 16]),
        &Bindings::default(),
        Limits::default(),
    )
}
/// The admitted records of `kind`, by identity.
fn rows_of(p: &CheckedPackage, kind: &str) -> Vec<(DeclarationId, entity::Record)> {
    let kind = p.names[kind];
    p.entities
        .iter()
        .filter(|(_, r)| p.refines(r.kind, kind))
        .map(|(id, r)| (*id, r.clone()))
        .collect()
}

#[test]
fn constants_admit_named_reference_conditions_in_their_declared_quantity_types() {
    let text = r#"package p {
        entity kind source provenance {}
        entity source convention {}
        enum role { published }
        constant t0: Temperature = stock.temperature provenance(convention, role.published);
        constant p0: Pressure = stock.pressure provenance(convention, role.published);
        fn condition() -> Temperature = stock.temperature;
        def Root {
            require t0 == 298.15{K} and p0 == 100000{Pa} : "named conditions";
            var temperature: Temperature;
            eq named_condition: temperature == condition();
        }
    }"#;
    specialized(text).unwrap();
    for invalid in [
        text.replace("t0: Temperature", "t0: Pressure"),
        text.replace("stock.temperature", "stock.volume"),
        text.replace("stock.pressure", "unknown.pressure"),
    ] {
        assert!(admitted(&invalid).is_err());
    }
}

#[test]
fn coefficient_unit_refusal_names_attribute_expected_and_actual_quantity() {
    let error = refusal(
        r#"package p {
        entity kind form { attribute coefficient: Energy; }
        entity form invalid { coefficient = 1{s} }
    }"#,
    );
    for detail in [
        "attribute coefficient",
        "Energy",
        "actual quantity Time",
        "1.0{s}",
    ] {
        assert!(error.contains(detail), "{detail}: {error}");
    }
}

#[test]
fn native_fixture_options_admit_only_exact_primitives() {
    let source = |value: &str| {
        format!(
            "package p {{ def D {{var x:Scalar; eq e:x==1;}} test t fixture {{dof 0; route steady; procedure solve; policy {{options {{\"setting\" = {value};}};}}}} {{child root:D=D();}} }}"
        )
    };
    for value in ["0", "false", "1e-3", "\"method\""] {
        admitted(&source(value)).unwrap();
    }
    for value in [
        "1{s}",
        "1.0 ± standard(0.1)",
        "2147483648",
        "missing",
        "p.D",
    ] {
        assert!(refusal(&source(value)).contains("requires a unique name and an exact Boolean"));
    }
    assert!(
        refusal(&source("0").replace("procedure solve", "procedure check"))
            .contains("pure fixture")
    );
}

#[test]
fn qualified_record_attributes_resolve_the_record_and_preserve_lexical_shadowing() {
    specialized(
        r#"package p {
        entity kind family { attribute omega: Scalar; }
        entity family pr { omega = 0.45724 }
        fn coefficient() -> Scalar = p.pr.omega;
        def Child(p: family) {
            require p.omega == 0.45724 : "the lexical record shadows its package name";
        }
        def Root {
            require p.pr.omega == 0.45724 : "a qualified record's attribute";
            child binding: Child = Child(p = pr);
            var omega: Scalar;
            eq coefficient_binding: omega == coefficient();
        }
    }"#,
    )
    .unwrap();
}

#[test]
fn abstract_kinds_refuse_records_without_poisoning_concrete_refinements() {
    let text = r#"package p {
 entity kind base abstract { attribute value: Scalar; }
 entity kind concrete extends base {}
 entity concrete admitted { value = 1 }
 entity kind keyed abstract { key name: Text; attribute value: Scalar; }
 entity kind row extends keyed {}
 entity kind source provenance {} entity source origin {}
 enum role { published }
 dataset bank: row provenance(origin, role.published) { ["a"] = [2]; }
}"#;
    let p = admitted(text).unwrap();
    assert!(p.kinds[&p.names["p.base"]].is_abstract);
    assert!(!p.kinds[&p.names["p.concrete"]].is_abstract);
    assert_eq!(rows_of(&p, "p.keyed").len(), 1);
    for invalid in [
        text.replace("entity concrete admitted", "entity base admitted"),
        text.replace("bank: row", "bank: keyed"),
        text.replace(
            "concrete extends base {}",
            "concrete extends base abstract {}",
        ),
    ] {
        let error = refusal(&invalid);
        assert!(
            error.contains("abstract kind") && error.contains("cannot supply"),
            "{error}"
        );
    }
}

/// A kind refines one kind and inherits its attributes; a member of the refinement is a
/// member of every kind it refines: in a set, as a function argument and as a coordinate.
#[test]
fn kind_extends_kind_and_subkind_members_conform() {
    let text = r#"package p {
 entity kind species { attribute charge: Scalar = 0; }
 entity kind ion extends species { attribute label: Text = "ion"; }
 entity species water {}
 entity ion sodium { charge = 1 }
 set members: Set<species> = {water, sodium};
 set ions: Set<ion> = {sodium};
 fn charge_of(j: species) -> Scalar = j.charge;
 def Root { var x[j in members]: Scalar; eq e[j in members]: x[j] == charge_of(j); eq f[j in ions]: x[j] == 1; }
}"#;
    let p = admitted(text).unwrap();
    let (species, ion) = (p.names["p.species"], p.names["p.ion"]);
    assert_eq!(p.kinds[&ion].base, Some(species));
    assert!(p.refines(ion, species) && !p.refines(species, ion));
    let sodium = p.record(p.names["p.sodium"]).unwrap();
    assert_eq!(sodium.kind, ion);
    // Inherited and own attributes, supplied and defaulted.
    assert_eq!(
        sodium.values.keys().collect::<Vec<_>>(),
        ["charge", "label"]
    );
    assert_eq!(sodium.values["label"], Value::Text("ion".into()));
    assert!(p.members[&ion].contains_key("charge"));
    let model = specialized(text).unwrap();
    assert_eq!(model.equations.len(), 3);
    // Controls: a species is not an ion, as a set member or as an argument.
    let error = specialized(&text.replace("{sodium};", "{sodium, water};"))
        .unwrap_err()
        .to_string();
    assert!(error.contains("does not satisfy"), "{error}");
    assert!(
        refusal(&text.replace(
            "def Root {",
            "fn only_ion(j: ion) -> Scalar = j.charge; def Bad { var y: Scalar; eq g: y == only_ion(water); } def Root {"
        ))
        .contains("function argument type")
    );
}

/// Refinement is single, acyclic and between kinds only.
#[test]
fn kind_refinement_cycle_refused() {
    for (text, expected) in [
        (
            "package p { entity kind a extends b {} entity kind b extends a {} }",
            "recursive kind refinement",
        ),
        (
            "package p { entity kind a extends a {} }",
            "recursive kind refinement",
        ),
        (
            "package p { interface I {} entity kind k extends I {} }",
            "only an entity kind is the base of an entity kind",
        ),
        (
            "package p { entity kind k {} interface I : k {} }",
            "is the base of entity kinds only",
        ),
        (
            "package p { entity kind a {} entity kind b {} entity kind c extends a, b {} }",
            "at most one kind",
        ),
    ] {
        let error = refusal(text);
        assert!(error.contains(expected), "{text}: {error}");
    }
    // Control: a chain of refinements admits.
    let p = admitted(
        "package p { entity kind a {} entity kind b extends a {} entity kind c extends b {} }",
    )
    .unwrap();
    assert!(p.refines(p.names["p.c"], p.names["p.a"]));
}

/// An identifier value is held by one entity across every package admitted together.
#[test]
fn identifier_unique_within_closure() {
    let text = r#"package lib {
 identifier scheme cas; identifier scheme inchikey;
 entity kind species { attribute cas: Id<cas>?; attribute inchi: Id<inchikey>?; }
 entity species benzene { cas = Id<cas>("71-43-2") }
}
package app {
 use lib @ "1.0.0";
 entity lib.species toluene { cas = Id<lib.cas>("108-88-3"), inchi = Id<lib.inchikey>("71-43-2") }
}"#;
    let p = admitted(text).unwrap();
    let cas = p.names["lib.cas"];
    assert_eq!(p.identified(cas, "71-43-2"), Some(p.names["lib.benzene"]));
    assert_eq!(p.identified(cas, "108-88-3"), Some(p.names["app.toluene"]));
    // The same value under another scheme is another identifier.
    assert_eq!(
        p.identified(p.names["lib.inchikey"], "71-43-2"),
        Some(p.names["app.toluene"])
    );
    let error = refusal(&text.replace("108-88-3", "71-43-2"));
    for part in [
        "identifier cas \"71-43-2\" is held by",
        "benzene",
        "toluene",
    ] {
        assert!(error.contains(part), "{part}: {error}");
    }
    // Keyed rows share the scope.
    let error = refusal(&text.replace(
        "entity species benzene",
        "entity kind named { key id: Id<cas>; } dataset rows: named provenance(s, role.given) { [Id<cas>(\"71-43-2\")] = []; } entity species benzene",
    ));
    assert!(error.contains("is held by"), "{error}");
}

/// Identifier values are opaque: kept and compared byte for byte, never trimmed,
/// normalized or checked, and never read as text or numbers.
#[test]
fn identifier_values_are_never_interpreted() {
    let values = [
        "71-43-2",
        "0071-43-2",
        "71-43-2 ",
        "not a CAS number",
        "",
        "\u{212b}",
        "A\u{30a}",
    ];
    let entities = values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            format!(
                "entity species s{i} {{ cas = Id<cas>({}) }}",
                pse_authoring::language::render_cell(&pse_authoring::language::cell(
                    pse_authoring::language::CellValue::from_text(
                        pse_authoring::language::CellText {
                            value: (*v).to_owned()
                        }
                    )
                ))
                .unwrap()
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    let text = format!(
        "package p {{ identifier scheme cas; identifier scheme other; entity kind species {{ attribute cas: Id<cas>; }} {entities} }}"
    );
    let p = admitted(&text).unwrap();
    let cas = p.names["p.cas"];
    for (i, value) in values.iter().enumerate() {
        let id = p.names[&format!("p.s{i}")];
        assert_eq!(p.identified(cas, value), Some(id), "{value:?}");
        assert_eq!(
            p.record(id).unwrap().values["cas"],
            Value::Identifier {
                scheme: cas,
                value: (*value).to_owned()
            }
        );
    }
    assert_eq!(p.identified(cas, "71-43-2  "), None);
    for (cell, expected) in [
        ("\"71-43-2\"", "a text cell"),
        ("71", "an integer cell"),
        ("Id<other>(\"71-43-2\")", "scheme other where scheme cas"),
    ] {
        let error = refusal(&text.replace("Id<cas>(\"71-43-2\")", cell));
        assert!(error.contains(expected), "{cell}: {error}");
    }
    // A scheme is a type only as `Id<scheme>`.
    assert!(
        refusal("package p { identifier scheme cas; entity kind k { attribute c: cas; } }")
            .contains("written Id<cas>")
    );
}

/// Attribute values and defaults are cells typed once at admission, before any
/// specialization, and admitted in canonical units.
#[test]
fn attribute_cells_type_checked_at_admission() {
    let text = r#"package p {
 entity kind other {} entity other z {}
 entity kind k { attribute m: Energy; attribute peer: k? = missing; attribute tags: Set<Choice> = {first}; attribute x: MoleFraction = 0.5{1}; }
 enum Choice { first, second }
 entity k a { m = 2{kJ} ± standard(1) }
 entity k b { m = 1{J}, peer = a, tags = {first, Choice.second} }
}"#;
    let p = admitted(text).unwrap();
    let a = p.record(p.names["p.a"]).unwrap();
    assert!(matches!(a.values["m"], Value::Number { bits, .. } if f64::from_bits(bits) == 2000.0));
    assert_eq!(a.uncertainties["m"].magnitude, 1000.0);
    assert_eq!(a.values["peer"], Value::Missing);
    let b = p.record(p.names["p.b"]).unwrap();
    assert!(matches!(&b.values["peer"], Value::Entity { id, .. } if *id == p.names["p.a"]));
    assert!(matches!(&b.values["tags"], Value::Set(members) if members.len() == 2));
    for (from, to, expected) in [
        ("m = 1{J}, peer", "m = 1{s}, peer", "unit"),
        ("peer = a,", "peer = z,", "not an entity of kind k"),
        ("m = 1{J}, peer", "m = \"1 J\", peer", "a text cell"),
        (
            "m = 1{J}, peer",
            "mass = 1{J}, peer",
            "unknown attribute mass",
        ),
        (
            "{first, Choice.second}",
            "{first, third}",
            "third is not a member of Choice",
        ),
        ("± standard(1)", "± bound(-1)", "finite and nonnegative"),
        // A bare number is the neutral scalar; a written `{1}` takes the expected type.
        (
            "MoleFraction = 0.5{1};",
            "MoleFraction = 0.5;",
            "does not satisfy",
        ),
        (
            "attribute peer: k? = missing;",
            "attribute peer: k? = missing; attribute n: Energy = 2{s};",
            "unit",
        ),
        (
            "attribute peer: k? = missing;",
            "attribute peer: k? = missing; attribute label: Text = \"x\" ± standard(1);",
            "numeric value only",
        ),
    ] {
        let changed = text.replace(from, to);
        assert_ne!(changed, text, "{from}");
        let error = refusal(&changed);
        assert!(error.contains(expected), "{to}: {error}");
    }
}

/// A required attribute has a supplied value, a kind binding or a default; absence is a
/// value only of an optional attribute.
#[test]
fn missing_required_attribute_refused_at_admission() {
    let text = r#"package p {
 entity kind k { attribute m: Energy; attribute n: Energy?; attribute d: Energy = 1{kJ}; }
 entity k a { m = 2{J} }
}"#;
    let p = admitted(text).unwrap();
    let a = p.record(p.names["p.a"]).unwrap();
    assert_eq!(a.values["n"], Value::Missing);
    assert!(matches!(a.values["d"], Value::Number { bits, .. } if f64::from_bits(bits) == 1000.0));
    let error = refusal(&text.replace("{ m = 2{J} }", "{}"));
    assert!(error.contains("missing attribute m of kind k"), "{error}");
    let error = refusal(&text.replace("m = 2{J}", "m = missing"));
    assert!(error.contains("missing value where"), "{error}");
    // A keyed row omits a trailing value only where the attribute has a default or is
    // optional.
    let keyed = r#"package p {
 entity kind item {} entity item a {}
 entity kind form { key subject: item; attribute m: Energy; attribute n: Energy?; }
 dataset rows: form provenance(s, role.given) { [a] = [1{J}]; }
}"#;
    admitted(keyed).unwrap();
    let error = refusal(&keyed.replace("[1{J}]", "[]"));
    assert!(
        error.contains("missing attribute m of kind form"),
        "{error}"
    );
}

const KEYED: &str = r#"package p {
 entity kind species {} entity species benzene {}
 enum Phase { @id("0aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa") liquid, vapor }
 entity kind parameter_set { key subject: species; key phase: Phase; key variant: Integer = 1; }
 entity kind constant_cp extends parameter_set { attribute c: Scalar; }
 dataset d: constant_cp provenance(s, role.given) { [benzene, liquid] = [2.0]; [benzene, vapor, 2] = [3.0]; }
 def Root { require constant_cp[benzene, Phase.liquid].c == 2 : "by keys"; require parameter_set[benzene, Phase.vapor, 2].variant == 2 : "through the key-declaring kind"; }
}"#;

/// A keyed row's identity is framed over the key-declaring kind and its ordered, typed
/// key values, defaults included; an enumeration member enters by its identity.
#[test]
fn keyed_identity_is_the_key_declaring_kind_and_typed_keys() {
    let p = admitted(KEYED).unwrap();
    let (parameter_set, constant_cp) = (p.names["p.parameter_set"], p.names["p.constant_cp"]);
    let phase = p.names["p.Phase"];
    let liquid = Value::Enum {
        enumeration: phase,
        member: SemanticId::parse_hex("0aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap(),
    };
    let benzene = Value::Entity {
        id: p.names["p.benzene"],
        kind: p.names["p.species"],
    };
    let identity = entity::keyed_identity(
        parameter_set,
        &[benzene.clone(), liquid.clone(), Value::Integer(1)],
    );
    let row = p.record(identity).unwrap();
    assert_eq!(row.kind, constant_cp);
    assert_eq!(row.origin, p.names["p.d"]);
    assert_eq!(row.values["variant"], Value::Integer(1));
    // Not the concrete kind's frame, and not without the default.
    assert!(!p.entities.contains_key(&entity::keyed_identity(
        constant_cp,
        &[benzene.clone(), liquid.clone(), Value::Integer(1)]
    )));
    assert!(
        !p.entities
            .contains_key(&entity::keyed_identity(parameter_set, &[benzene, liquid]))
    );
    // The declared lookup finds both rows (a pure fixture of static requirements).
    specialized(KEYED).unwrap();
    // A default is part of the identity; an explicit key equal to it is the same row.
    let changed =
        admitted(&KEYED.replace("key variant: Integer = 1;", "key variant: Integer = 3;")).unwrap();
    assert!(!changed.entities.contains_key(&identity));
    let explicit =
        admitted(&KEYED.replace("[benzene, liquid] =", "[benzene, liquid, 1] =")).unwrap();
    assert!(explicit.entities.contains_key(&identity));
    // Renaming a member that has an identity re-keys nothing.
    let renamed = admitted(
        &KEYED
            .replace(") liquid,", ") fluid,")
            .replace("[benzene, liquid] =", "[benzene, fluid] =")
            .replace(
                "constant_cp[benzene, Phase.liquid]",
                "constant_cp[benzene, Phase.fluid]",
            ),
    )
    .unwrap();
    assert!(renamed.entities.contains_key(&identity));
    // A key is a scalar with a byte-exact framing.
    assert!(
        refusal(&KEYED.replace("key subject: species;", "key subject: Set<species>;"))
            .contains("a key is a reference")
    );
    // One kind of a lineage declares the keys.
    assert!(
        refusal(&KEYED.replace(
            "attribute c: Scalar;",
            "attribute c: Scalar; key extra: Text;"
        ))
        .contains("declared by one kind, here parameter_set")
    );
}

const FORMS: &str = r#"package p {
 entity kind item {} entity item a {} entity item b {}
 entity kind form { key subject: item; key source: Text; }
 entity kind doubling extends form {} entity kind tripling extends form {}
 dataset d: doubling bind(source = "bank") provenance(s, role.given) { [a] = []; }
 dataset t: tripling bind(source = "bank") provenance(s, role.given) { [b] = []; }
}"#;

/// Key uniqueness holds at the key-declaring kind across every refinement.
#[test]
fn same_key_in_two_forms_is_refused() {
    admitted(FORMS).unwrap();
    let error = refusal(&FORMS.replace("{ [b] = []; }", "{ [a] = []; }"));
    for name in ["d (kind doubling)", "t (kind tripling)", "key of kind form"] {
        assert!(error.contains(name), "{name}: {error}");
    }
    // Twice in one dataset.
    let error = refusal(&FORMS.replace("{ [a] = []; }", "{ [a] = []; [a] = []; }"));
    assert!(
        error.contains("supplies one key of kind form twice"),
        "{error}"
    );
    // A dataset-supplied key is a declared binding of a key.
    let error = refusal(&FORMS.replace(
        "bind(source = \"bank\") provenance(s, role.given) { [a]",
        "bind(label = \"x\") provenance(s, role.given) { [a]",
    ));
    assert!(
        error.contains("binding label names no attribute"),
        "{error}"
    );
    // The binding is part of the key: another source is another row.
    admitted(&FORMS.replace(
        "{ [b] = []; }",
        "{ [b] = []; } dataset e: tripling bind(source = \"other\") provenance(s, role.given) { [a] = []; }",
    ))
    .unwrap();
}

/// The concrete refinement is content: a row moved to another form keeps its identity.
#[test]
fn moving_a_row_between_forms_keeps_its_identity() {
    let before = admitted(FORMS).unwrap();
    let after = admitted(
        &FORMS
            .replace("dataset d: doubling", "dataset d: tripling")
            .replace("dataset t: tripling", "dataset t: doubling"),
    )
    .unwrap();
    let identities = |p: &CheckedPackage| {
        rows_of(p, "p.form")
            .into_iter()
            .map(|(id, record)| (id, p.declarations[&record.kind].name.clone()))
            .collect::<Vec<_>>()
    };
    let (before, after) = (identities(&before), identities(&after));
    assert_eq!(before.len(), 2);
    assert_eq!(
        before.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        after.iter().map(|(id, _)| *id).collect::<Vec<_>>()
    );
    assert_ne!(before, after, "the forms are content and moved");
}

const BINDINGS: &str = r#"package p {
 entity kind item {} entity item a {}
 entity kind form { key subject: item; attribute cp: Fn(x: Scalar, s: form) -> Scalar; }
 entity kind linear extends form { attribute slope: Scalar; cp = linear_cp; }
 fn linear_cp(x: Scalar, s: linear) -> Scalar = s.slope * x;
 dataset rows: linear provenance(s, role.given) { [a] = [2.0]; }
}"#;

/// A refinement binds an inherited function attribute for itself and its refinements;
/// the most-derived binding wins, and rebinding one says `override`.
#[test]
fn refined_kind_binds_inherited_function_attribute() {
    let p = admitted(BINDINGS).unwrap();
    let (_, row) = rows_of(&p, "p.form").remove(0);
    assert_eq!(row.values["cp"], Value::Function(p.names["p.linear_cp"]));
    assert!(
        matches!(row.values["slope"], Value::Number { bits, .. } if f64::from_bits(bits) == 2.0)
    );
    let steep = BINDINGS.replace(
        "dataset rows: linear",
        "entity kind steep extends linear { override cp = steep_cp; } fn steep_cp(x: Scalar, s: steep) -> Scalar = 2 * s.slope * x; dataset rows: steep",
    );
    let p = admitted(&steep).unwrap();
    let (_, row) = rows_of(&p, "p.form").remove(0);
    assert_eq!(row.values["cp"], Value::Function(p.names["p.steep_cp"]));
    for (from, to, expected) in [
        (
            "override cp = steep_cp;",
            "cp = steep_cp;",
            "requires override",
        ),
        (
            "cp = linear_cp;",
            "cp = scaled;",
            "does not have the bound attribute's signature",
        ),
        (
            "cp = linear_cp;",
            "cp = linear_cp; heat = linear_cp;",
            "binds no attribute inherited",
        ),
        (
            "cp = linear_cp;",
            "cp = unrelated;",
            "does not have the bound attribute's signature",
        ),
        (
            "cp = linear_cp;",
            "cp = linear_cp; subject = a;",
            "supplied by each row",
        ),
    ] {
        let source = steep.replace(
            "dataset rows:",
            "fn scaled(x: Energy, s: linear) -> Scalar = s.slope; entity kind loose {} fn unrelated(x: Scalar, s: loose) -> Scalar = x; dataset rows:",
        );
        let changed = source.replacen(from, to, 1);
        assert_ne!(changed, source, "{from}");
        let error = refusal(&changed);
        assert!(error.contains(expected), "{to}: {error}");
    }
    // A kind-bound attribute is not supplied per entity.
    let error = refusal(
        "package p { entity kind form { attribute cp: Fn(x: Scalar) -> Scalar; } entity kind linear extends form { cp = f; } fn f(x: Scalar) -> Scalar = x; fn g(x: Scalar) -> Scalar = x; entity linear z { cp = g } }",
    );
    assert!(
        error.contains("attribute cp is bound by kind linear"),
        "{error}"
    );
}

const DISPATCH: &str = r#"package p {
 entity kind item {} entity item a {} entity item b {}
 entity kind form { key subject: item; attribute cp: Fn(x: Scalar) -> Scalar; }
 entity kind doubling extends form { cp = double; }
 fn double(x: Scalar) -> Scalar = x*2;
 dataset d: doubling provenance(s, role.given) { [a] = []; [b] = []; }
 fn through(s: form, x: Scalar) -> Scalar = s.cp(x);
 def Root { var x: Scalar; eq e: 0 == STATIC; }
}"#;

/// A `Ref` is static at specialization: a call through one whose value depends on a
/// runtime value is refused, never deferred to the solve.
#[test]
fn non_static_ref_is_refused() {
    // Controls: a static reference, bound locally or passed as an argument.
    for call in ["s.cp(x) where s = form[a]", "through(form[b], x)"] {
        let model = specialized(&DISPATCH.replace("STATIC", call)).unwrap();
        assert_eq!(model.equations.len(), 1, "{call}");
    }
    for (call, expected) in [
        (
            "s.cp(x) where s = if x > 0 then form[a] else form[b]",
            "s must be static at specialization",
        ),
        (
            "through(if x > 0 then form[a] else form[b], x)",
            "Ref argument s of through must be static at specialization",
        ),
    ] {
        let error = specialized(&DISPATCH.replace("STATIC", call))
            .expect_err(call)
            .to_string();
        assert!(error.contains(expected), "{call}: {error}");
        assert!(error.contains("runtime value x"), "{call}: {error}");
    }
}

/// A constant is a typed declaration whose value is a cell, admitted in canonical units and
/// read as immutable package data.
#[test]
fn constants_are_typed_declarations() {
    let text = r#"package p {
 constant m0: Energy = 2{kJ} ± standard(1) provenance(s, role.given);
 fn shifted(y: Energy) -> Energy = y + m0;
 def Root { var x: Energy; eq e: x == shifted(m0); }
}"#;
    let p = admitted(text).unwrap();
    let m0 = p.names["p.m0"];
    assert!(matches!(p.types[&m0], Type::Quantity(_)));
    let constant = &p.constants[&m0];
    assert!(matches!(constant.value, Value::Number { bits, .. } if f64::from_bits(bits) == 2000.0));
    assert_eq!(constant.uncertainty.unwrap().magnitude, 1000.0);
    assert!(
        !p.functions.contains_key(&m0),
        "a constant is not a function"
    );
    assert_eq!(specialized(text).unwrap().equations.len(), 1);
    for (value, expected) in [
        ("2{s}", "unit"),
        ("\"2 kJ\"", "a text cell"),
        ("x", "not a named reference-state condition"),
    ] {
        let error = refusal(&text.replace("2{kJ} ± standard(1)", value));
        assert!(error.contains(expected), "{value}: {error}");
    }
}
