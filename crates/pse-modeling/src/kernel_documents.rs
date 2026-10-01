// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Package data documents admitted through a table's plan (ADR-0125, Plan 23 KR9): typed
//! columns, units only from the declaration, references by identity or identifier scheme.
use crate::document::{
    DataDocument, DocumentColumn, DocumentInventory, DocumentPlan, DocumentTable, Documents,
    RowSet, Values,
};
use crate::kernel_types::{physical, try_source};
use crate::specialize::Value;
use crate::*;
use pse_ids::SemanticId;
use std::sync::Arc;

const PACKAGE: SemanticId = SemanticId::from_bytes([9; 16]);

/// Admission without a memo: the pure admission every owner calls.
struct Direct<'a> {
    inventory: DocumentInventory,
    quantities: &'a pse_quantity::QuantityRegistry,
}
impl Documents for Direct<'_> {
    fn resolve(&self, source: SemanticId, path: &str) -> Option<SemanticId> {
        self.inventory.resolve(source, path)
    }
    fn admit(&self, plan: &Arc<DocumentPlan>, document: SemanticId) -> Result<Arc<DocumentTable>> {
        document::admit(plan, &self.inventory.documents[&document], self.quantities).map(Arc::new)
    }
}

fn column(name: &str, values: Values) -> DocumentColumn {
    DocumentColumn {
        name: name.into(),
        unit: None,
        values,
    }
}
fn text(values: &[Option<&str>]) -> Values {
    Values::Text(values.iter().map(|v| v.map(str::to_owned)).collect())
}
fn admitted(text: &str, columns: Vec<DocumentColumn>) -> Result<CheckedPackage> {
    let (registry, _) = physical();
    let context = TypeContext {
        admissions: None,
        formula_authority: None,
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        scope: &PhysicalScope::default(),
    };
    let document = DataDocument {
        id: pse_ids::named_id(PACKAGE, "data/bank.parquet"),
        path: "data/bank.parquet".into(),
        content_hash: pse_ids::encoding_checksum(b"bank").content_hash(),
        rows: RowSet::new(columns).map_err(|e| invalid(DeclarationId::from(SemanticId::NIL), e))?,
    };
    let documents = Direct {
        inventory: DocumentInventory {
            packages: [(SemanticId::NIL, PACKAGE)].into(),
            documents: [(document.id, Arc::new(document))].into(),
        },
        quantities: &registry,
    };
    check_with(&try_source(text)?, &context, &documents)
}
fn refusal(text: &str, columns: Vec<DocumentColumn>) -> String {
    match admitted(text, columns) {
        Ok(_) => panic!("admitted: {text}"),
        Err(error) => error.to_string(),
    }
}

const BANK: &str = r#"package p {
 identifier scheme cas;
 entity kind item { attribute cas: Id<cas>?; }
 entity item a { cas = Id<cas>("71-43-2") }
 entity item b { cas = Id<cas>("108-88-3") }
 enum phase { liquid, vapor }
 table bank[j: item by cas, ph: phase]: {h: Energy storage {kJ}, T: Temperature? storage {degC}, n: Integer, note: Text, ok: Boolean} missing optional;
 dataset banked: bank provenance(s, role.given) from "data/bank.parquet";
}"#;

fn bank() -> Vec<DocumentColumn> {
    vec![
        column("j", text(&[Some("71-43-2"), Some("108-88-3")])),
        column(
            "ph",
            Values::Member {
                members: vec!["vapor".into(), "liquid".into()],
                keys: vec![Some(1), Some(0)],
            },
        ),
        DocumentColumn {
            unit: Some("kJ".into()),
            ..column("h", Values::Magnitude(vec![Some(1.5), Some(-2.0)]))
        },
        column("T", Values::Magnitude(vec![Some(25.0), None])),
        column("n", Values::Integer(vec![Some(1), Some(2)])),
        column("note", text(&[Some("first"), Some("second")])),
        column("ok", Values::Boolean(vec![Some(true), Some(false)])),
    ]
}

#[test]
fn keyed_kind_document_rows_equal_inline_records_and_resolve_identifier_keys() {
    let source = r#"package p {
 identifier scheme cas;
 entity kind item { attribute cas: Id<cas> unique; }
 entity item a { cas = Id<cas>("71-43-2") }
 entity item b { cas = Id<cas>("108-88-3") }
 entity kind source provenance {} entity source s {} enum role { given }
 entity kind form { key subject: item by cas; key variant: Integer = 1; attribute h: Energy storage {kJ}; attribute note: Text?; }
 dataset banked: form provenance(s, role.given) from "data/bank.parquet";
}"#;
    let columns = || {
        vec![
            column("subject", text(&[Some("71-43-2"), Some("108-88-3")])),
            column("h", Values::Magnitude(vec![Some(1.5), Some(-2.0)])),
            column("note", text(&[Some("first"), None])),
        ]
    };
    let p = admitted(source, columns()).unwrap();
    let inline = source.replace(
        "from \"data/bank.parquet\";",
        "{[a]=[1.5{kJ}, \"first\"]; [b]=[-2{kJ}, missing];}",
    );
    let plain = admitted(&inline, columns()).unwrap();
    assert_eq!(p.entities, plain.entities);
    assert_eq!(p.provenance, plain.provenance);
    assert_eq!(
        p.entities
            .values()
            .filter(|r| r.origin == p.names["p.banked"])
            .count(),
        2
    );
    let mut duplicate = columns();
    duplicate[0].values = text(&[Some("71-43-2"), Some("71-43-2")]);
    assert!(refusal(source, duplicate).contains("already admitted"));
    let mut unknown = columns();
    unknown[0].values = text(&[Some("unknown"), Some("108-88-3")]);
    assert!(refusal(source, unknown).contains("unknown"));
    let mut extra = columns();
    extra.push(column(
        "undeclared",
        Values::Integer(vec![Some(0), Some(0)]),
    ));
    assert!(refusal(source, extra).contains("does not declare"));
    let mut wrong = columns();
    wrong[1].values = text(&[Some("1.5"), Some("-2.0")]);
    assert!(refusal(source, wrong).contains("column h"));
    let mut wrong_unit = columns();
    wrong_unit[1].unit = Some("kg".into());
    assert!(refusal(source, wrong_unit).contains("declared storage unit"));
}

#[test]
fn keyed_document_references_use_the_complete_identity_index() {
    let source = r#"package p {
 entity kind source provenance {} entity source s {} enum role { given }
 entity kind node {key n:Integer; attribute parent:node?;}
 dataset nodes:node provenance(s,role.given) from "data/bank.parquet";
}"#;
    let kind = try_source(source)
        .unwrap()
        .iter()
        .find(|row| row.name == "node")
        .unwrap()
        .declaration_id;
    let identity = |n| entity::keyed_identity(kind, &[Value::Integer(n)]);
    let columns = |parent| {
        vec![
            column("n", Values::Integer(vec![Some(1), Some(2)])),
            column("parent", Values::Identity(vec![Some(parent), None])),
        ]
    };
    let p = admitted(source, columns(identity(2).as_id())).unwrap();
    assert_eq!(
        p.record(identity(1)).unwrap().values["parent"],
        Value::Entity {
            id: identity(2),
            kind
        }
    );
    assert!(refusal(source, columns(identity(1).as_id())).contains("cycle"));
    assert!(refusal(source, columns(SemanticId::NIL)).contains("no admitted entity"));
}

/// A document's columns are typed once each through the table's plan: identifier-scheme
/// references resolve through the identifier index, a dictionary names members, magnitudes
/// convert from the declared storage unit, and a null of an optional column is absence.
#[test]
fn document_rows_admit_through_the_plan() {
    let p = admitted(BANK, bank()).unwrap();
    let table = &p.tables[&p.names["p.bank"]];
    assert_eq!(table.rows.len(), 2);
    let member = |name: &str| Value::Enum {
        enumeration: p.names["p.phase"],
        member: p.declarations[&p.names["p.phase"]]
            .value
            .enumeration
            .as_ref()
            .unwrap()
            .members
            .iter()
            .find(|m| m.name == name)
            .unwrap()
            .member_id,
    };
    let a = Value::Entity {
        id: p.names["p.a"],
        kind: p.names["p.item"],
    };
    let row = &table.rows[&vec![a, member("liquid")]];
    let scalar = |v: &Value| v.scalar(p.names["p.bank"]).unwrap();
    assert_eq!(scalar(&row.cells[0]), 1500.0);
    assert!((scalar(&row.cells[1]) - 298.15).abs() < 1e-9);
    assert_eq!(row.cells[2], Value::Integer(1));
    assert_eq!(row.cells[3], Value::Text("first".into()));
    assert_eq!(row.cells[4], Value::Boolean(true));
    let b = Value::Entity {
        id: p.names["p.b"],
        kind: p.names["p.item"],
    };
    let row = &table.rows[&vec![b, member("vapor")]];
    assert_eq!(scalar(&row.cells[0]), -2000.0);
    assert_eq!(row.cells[1], Value::Missing);
    assert_eq!(row.origin, p.names["p.banked"]);
    // The dataset carries no cells: the document is the rows.
    assert!(
        p.declarations[&p.names["p.banked"]]
            .value
            .dataset
            .as_ref()
            .unwrap()
            .rows
            .is_empty()
    );
}

#[test]
fn canonical_numeric_admission_matches_inline_columns_and_uncertainty_scale() {
    use pse_authoring::dsl::Number;
    use pse_quantity::{UnitProduct, scheme::Scheme};
    let (registry, names) = physical();
    let context = TypeContext {
        admissions: None,
        formula_authority: None,
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        scope: &PhysicalScope::default(),
    };
    let p = admitted(BANK, bank()).unwrap();
    let table = &p.tables[&p.names["p.bank"]];
    let at = p.names["p.bank"];
    for (value, symbol, quantity_name, cell, scale) in [
        (1.5, "kJ", "Energy", 0, 1000.0),
        (25.0, "degC", "Temperature", 1, 1.0),
    ] {
        let expected = Type::Quantity(Scheme::Concrete(names[quantity_name]));
        let (inline, uncertainty_scale) = specialize::value::number(
            &context,
            at,
            &Number {
                value,
                exact_integer: None,
                unit: Some(UnitProduct::symbol(symbol)),
            },
            Some(&expected),
        )
        .unwrap();
        assert!(table.rows.values().any(|row| row.cells[cell] == inline));
        assert_eq!(uncertainty_scale, scale);
    }
    assert!(
        table
            .rows
            .values()
            .any(|row| row.cells[1] == Value::Missing)
    );
    let scalar = Type::Quantity(Scheme::Concrete(names["Scalar"]));
    for value in [f64::NAN, f64::INFINITY] {
        let error = specialize::value::number(
            &context,
            at,
            &Number {
                value,
                exact_integer: None,
                unit: None,
            },
            Some(&scalar),
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("nonfinite magnitude"), "{error}");
    }
    let (negative_zero, _) = specialize::value::number(
        &context,
        at,
        &Number {
            value: -0.0,
            exact_integer: None,
            unit: None,
        },
        Some(&scalar),
    )
    .unwrap();
    assert!(matches!(negative_zero, Value::Number { bits, .. } if bits == (-0.0_f64).to_bits()));
}

#[test]
fn canonical_numeric_overflow_refuses_with_document_row_and_inline_attribution() {
    let mut columns = bank();
    columns[2].values = Values::Magnitude(vec![Some(1.0), Some(f64::MAX)]);
    let error = refusal(BANK, columns);
    for detail in [
        "column h",
        "data/bank.parquet",
        "row 1",
        "nonfinite converted magnitude",
    ] {
        assert!(error.contains(detail), "{detail}: {error}");
    }
    let (registry, names) = physical();
    let context = TypeContext {
        admissions: None,
        formula_authority: None,
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        scope: &PhysicalScope::default(),
    };
    let at = DeclarationId::from(SemanticId::from_bytes([0x7a; 16]));
    let expected = Type::Quantity(pse_quantity::scheme::Scheme::Concrete(names["Energy"]));
    let error = specialize::value::number(
        &context,
        at,
        &pse_authoring::dsl::Number {
            value: f64::MAX,
            exact_integer: None,
            unit: Some(pse_quantity::UnitProduct::symbol("kJ")),
        },
        Some(&expected),
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("nonfinite converted magnitude"), "{error}");
    assert!(error.contains(&at.to_string()), "{error}");
}

/// A reference by identity names an admitted entity of the declared kind.
#[test]
fn document_identity_references_name_admitted_entities() {
    let text = BANK.replace("item by cas,", "item,");
    let p = admitted(&text, bank()).map(|_| ()).unwrap_err().to_string();
    assert!(p.contains("declares no identifier scheme"), "{p}");
    let mut columns = bank();
    let a = admitted_names(&text)["p.a"].as_id();
    columns[0] = column("j", Values::Identity(vec![Some(a), Some(a)]));
    columns[1] = column(
        "ph",
        Values::Member {
            members: vec!["liquid".into(), "vapor".into()],
            keys: vec![Some(0), Some(1)],
        },
    );
    let p = admitted(&text, columns.clone()).unwrap();
    assert_eq!(p.tables[&p.names["p.bank"]].rows.len(), 2);
    columns[0] = column(
        "j",
        Values::Identity(vec![Some(a), Some(SemanticId::from_bytes([3; 16]))]),
    );
    let error = refusal(&text, columns);
    assert!(
        error.contains("column j") && error.contains("no admitted entity"),
        "{error}"
    );
}

fn admitted_names(text: &str) -> std::collections::BTreeMap<String, DeclarationId> {
    let (registry, _) = physical();
    let context = TypeContext {
        admissions: None,
        formula_authority: None,
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        scope: &PhysicalScope::default(),
    };
    let text = text.replace(
        r#" dataset banked: bank provenance(s, role.given) from "data/bank.parquet";"#,
        "",
    );
    check(&try_source(&text).unwrap(), &context).unwrap().names
}

/// Every refusal names the column and its declared type: a missing, undeclared or
/// mistyped column, a stated unit other than the declared storage unit, a null in a
/// required column, an unknown identifier value, a member outside the enumeration and a
/// quantity column without a storage unit.
#[test]
fn document_refusals_name_the_column_and_declared_type() {
    type Mutation = Box<dyn Fn(&mut Vec<DocumentColumn>)>;
    let cases: Vec<(Mutation, &str)> = vec![
        (
            Box::new(|c| {
                c.remove(4);
            }),
            "has no column n",
        ),
        (
            Box::new(|c| c.push(column("extra", Values::Integer(vec![Some(1), Some(2)])))),
            "has column extra",
        ),
        (
            Box::new(|c| c[4] = column("n", Values::Magnitude(vec![Some(1.0), Some(2.0)]))),
            "stores it as Float64, where Int64 is expected",
        ),
        (
            Box::new(|c| c[2].unit = Some("J".into())),
            "states unit J, which disagrees with the declared storage unit",
        ),
        (
            Box::new(|c| c[4].unit = Some("K".into())),
            "only a quantity column has one",
        ),
        (
            Box::new(|c| c[5] = column("note", text(&[Some("x"), None]))),
            "row 1 is null",
        ),
        (
            Box::new(|c| c[0] = column("j", text(&[Some("71-43-2"), Some("0-00-0")]))),
            "\"0-00-0\", which no admitted entity",
        ),
        (
            Box::new(|c| {
                c[1] = column(
                    "ph",
                    Values::Member {
                        members: vec!["solid".into()],
                        keys: vec![Some(0), Some(0)],
                    },
                )
            }),
            "names solid, which is not a member",
        ),
        (
            Box::new(|c| c[2] = column("h", Values::Magnitude(vec![Some(f64::NAN), Some(1.0)]))),
            "nonfinite magnitude",
        ),
    ];
    for (edit, expected) in cases {
        let mut columns = bank();
        edit(&mut columns);
        let error = refusal(BANK, columns);
        assert!(error.contains(expected), "{expected}: {error}");
        assert!(
            error.contains("declared") || error.contains("declare"),
            "{error}"
        );
    }
    let error = refusal(&BANK.replace("Energy storage {kJ}", "Energy"), bank());
    assert!(
        error.contains("column h declared") && error.contains("declares no storage unit"),
        "{error}"
    );
    let error = refusal(
        &BANK.replace("n: Integer,", "n: Integer storage {kJ},"),
        bank(),
    );
    assert!(error.contains("only a quantity has"), "{error}");
    let error = refusal(
        &BANK.replace("data/bank.parquet", "data/other.parquet"),
        bank(),
    );
    assert!(
        error.contains("which its package does not carry"),
        "{error}"
    );
}
