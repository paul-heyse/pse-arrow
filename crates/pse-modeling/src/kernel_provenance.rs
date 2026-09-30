// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed provenance and test-only taint (ADR-0123 Outcome 5, Plan 23 KR6).
use crate::kernel_types::{physical, try_source};
use crate::*;
use pse_model::generated::enums::{ModelingDataFacet, ModelingLineageKind};

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
fn root(text: &str, name: &str) -> Result<SpecializedModel> {
    let p = admitted(text)?;
    specialize(
        &p,
        p.names[&format!("p.{name}")],
        InstanceId::from_bytes([7; 16]),
        &Bindings::default(),
        Limits::default(),
    )
}
fn refused(text: &str, name: &str) -> String {
    match root(text, name) {
        Ok(_) => panic!("{name} specialized"),
        Err(error) => {
            assert!(
                matches!(&error, ModelingError::TestOnly { .. }),
                "{name}: {error:?}"
            );
            error.to_string()
        }
    }
}

/// A package-declared provenance schema: source kinds, a note kind without the facet, a
/// role enumeration whose members declare their facets, and two sources.
const SCHEMA: &str = r#"
 entity kind source provenance { attribute title: Text; }
 entity kind publication extends source { attribute year: Integer; }
 entity kind note { attribute title: Text; }
 enum role { published, fitted facets(requires_lineage), oracle_input facets(test_only) }
 entity publication handbook { title = "Handbook", year = 1997 }
 entity source upstream { title = "Upstream release tests" }
 entity note remark { title = "a remark" }
 entity kind item {} entity item a {} entity item b {}
 set items: Set<item> = {a, b}; set first: Set<item> = {a}; set second: Set<item> = {b};
 table cp[j: item]: Scalar complete_over(j);"#;

#[test]
fn entity_and_attribute_origins_are_retained_and_cannot_launder_oracle_data() {
    let text = format!(
        r#"package p {{ {SCHEMA}
 entity kind datum {{ attribute value: Scalar; }}
 entity datum good provenance(handbook, role.published) {{ value = 2 }}
 entity datum explicit {{ value = 3 provenance(handbook, role.published) }}
 entity datum oracle provenance(upstream, role.oracle_input) {{ value = 4 provenance(handbook, role.published) }}
 entity datum attributed provenance(handbook, role.published) {{ value = 5 provenance(upstream, role.oracle_input) }}
 entity kind holder {{ attribute held: datum; }}
 entity holder laundering provenance(handbook, role.published) {{ held = oracle }}
 entity source derived_origin provenance(handbook, role.published, lineage(source upstream)) {{ title = "derived" }}
 def Good {{ require good.value + explicit.value > 0 : "positive"; }}
 def Oracle {{ require oracle.value > 0 : "positive"; }}
 def Attributed {{ require attributed.value > 0 : "positive"; }}
 def Laundering {{ require laundering.held.value > 0 : "positive"; }}
 test compared {{ require good.value + oracle.value + attributed.value > 0 : "positive"; }}
}}"#
    );
    let p = admitted(&text).unwrap();
    let published = p.names["p.handbook"];
    assert_eq!(p.provenance(p.names["p.good"]).unwrap().source, published);
    assert_eq!(
        p.attribute_provenance(p.names["p.good"], "value")
            .unwrap()
            .source,
        published
    );
    assert_eq!(
        p.attribute_provenance(p.names["p.oracle"], "value")
            .unwrap()
            .source,
        published
    );
    assert_eq!(
        p.attribute_provenance(p.names["p.attributed"], "value")
            .unwrap()
            .source,
        p.names["p.upstream"]
    );
    root(&text, "Good").unwrap();
    for name in ["Oracle", "Attributed", "Laundering"] {
        refused(&text, name);
    }
    root(&text, "compared").unwrap();
    // Selection retains entity and attribute source/role dependencies.
    let selected = p.select(p.names["p.explicit"]).unwrap();
    assert!(selected.declarations.contains_key(&published));
    assert!(
        selected
            .attribute_provenance(selected.names["p.explicit"], "value")
            .is_some()
    );
    let error = refusal(&text.replace(
        "value = 3 provenance(handbook",
        "value = 3 provenance(remark",
    ));
    assert!(error.contains("does not"), "{error}");
}

/// A dataset names a source entity whose kind, or an ancestor kind, carries the provenance
/// facet, and a role that is a member of a declared enumeration.
#[test]
fn dataset_source_requires_provenance_kind() {
    let text = format!(
        "package p {{ {SCHEMA}
 dataset data: cp complete_over(j in items) provenance(handbook, role.published) {{ [a] = [1.0]; [b] = [2.0]; }}
}}"
    );
    // A refinement of a provenance kind carries the facet.
    let p = admitted(&text).unwrap();
    let data = p.provenance(p.names["p.data"]).unwrap();
    assert_eq!(data.source, p.names["p.handbook"]);
    assert!(data.role.facets.is_empty() && data.lineage.is_empty());
    assert!(p.kinds[&p.names["p.publication"]].provenance);
    assert!(!p.kinds[&p.names["p.note"]].provenance);
    // The kernel provides each row's origin role.
    let row = &p.tables[&p.names["p.cp"]].rows[&vec![specialize::Value::Entity {
        id: p.names["p.a"],
        kind: p.names["p.item"],
    }]];
    assert_eq!(p.origin_role(row.origin), Some(&data.role));
    for (spelling, expected) in [
        (
            "provenance(remark, role.published)",
            "remark is a note, which does not",
        ),
        (
            "provenance(items, role.published)",
            "items is not an entity",
        ),
        (
            "provenance(nowhere, role.published)",
            "unknown entity nowhere",
        ),
        (
            "provenance(handbook, role.estimated)",
            "estimated is not a member of role",
        ),
        (
            "provenance(handbook, published)",
            "a role names its enumeration and member",
        ),
        (
            "provenance(handbook, handbook.published)",
            "is not a member of a declared enumeration",
        ),
    ] {
        let error = refusal(&text.replace("provenance(handbook, role.published)", spelling));
        assert!(error.contains(expected), "{spelling}: {error}");
    }
    // Provenance is required, and the text form is gone.
    for spelling in ["", "source \"Handbook\""] {
        let error = refusal(&text.replace("provenance(handbook, role.published)", spelling));
        assert!(error.contains("provenance"), "{spelling}: {error}");
    }
}

/// A root outside a test fixture that reads a test-only row is refused, whether the row's
/// keys are literal or bound at specialization; taint follows resolved references from
/// keyed rows, rows, derived columns, entities and constants. A test fixture reads them all.
#[test]
fn production_root_reading_an_oracle_row_is_refused() {
    let text = format!(
        r#"package p {{ {SCHEMA}
 dataset frozen: cp complete_over(j in items) provenance(upstream, role.oracle_input) {{ [a] = [1.0]; [b] = [2.0]; }}
 entity kind form {{ key subject: item; attribute value: Scalar; }}
 dataset forms: form provenance(upstream, role.oracle_input) {{ [a] = [3.0]; }}
 table choice[j: item]: form complete_over(j in first);
 dataset chosen: choice provenance(handbook, role.published) {{ [a] = [form[a]]; }}
 table doubled[j: item]: {{factor: Scalar, derived v: Scalar = factor * cp[j]}} complete_over(j in first);
 dataset scaled: doubled provenance(handbook, role.published) {{ [a] = [2.0]; }}
 entity kind holder {{ attribute held: form; }}
 entity holder h {{ held = form[a] }}
 constant c0: Scalar = 4.0 provenance(upstream, role.oracle_input);
 def Reads(j: item) {{ require cp[j] > 0 : "positive"; }}
 def Literal {{ require cp[a] > 0 : "positive"; }}
 def Bound {{ child r: Reads = Reads(j = b); }}
 def Keyed {{ require form[a].value > 0 : "positive"; }}
 def Referenced {{ require choice[a].value > 0 : "positive"; }}
 def Derived {{ require doubled[a].v > 0 : "positive"; }}
 def Held {{ require h.held.value > 0 : "positive"; }}
 def Constant {{ require c0 > 0 : "positive"; }}
 test fixture {{ child r: Reads = Reads(j = b); require cp[a] + form[a].value + choice[a].value + doubled[a].v + h.held.value + c0 > 0 : "positive"; }}
}}"#
    );
    let p = admitted(&text).unwrap();
    let a = specialize::Value::Entity {
        id: p.names["p.a"],
        kind: p.names["p.item"],
    };
    for table in ["p.cp", "p.choice", "p.doubled"] {
        assert!(
            p.tables[&p.names[table]].rows[&vec![a.clone()]].test_only,
            "{table}"
        );
    }
    assert!(p.is_test_only(p.names["p.h"]) && p.is_test_only(p.names["p.c0"]));
    // A literal key resolves its row statically, a bound key at specialization.
    let error = refused(&text, "Literal");
    for part in [
        "test-only data is read outside a test fixture",
        "row cp[a] supplied by frozen with role oracle_input",
        "read by root Literal",
    ] {
        assert!(error.contains(part), "{part}: {error}");
    }
    assert!(refused(&text, "Bound").contains("row cp[b]"));
    assert!(refused(&text, "Keyed").contains("form[a] supplied by forms with role oracle_input"));
    assert!(refused(&text, "Referenced").contains("row choice[a] supplied by chosen"));
    assert!(refused(&text, "Derived").contains("row doubled[a]"));
    assert!(refused(&text, "Held").contains("entity h referencing test-only data"));
    assert!(
        refused(&text, "Constant").contains("constant c0 supplied by c0 with role oracle_input")
    );
    root(&text, "fixture").unwrap();
    // The same roots read the same data once its role is not test-only.
    let published = text.replace("role.oracle_input", "role.published");
    for name in [
        "Literal",
        "Bound",
        "Keyed",
        "Referenced",
        "Derived",
        "Held",
        "Constant",
    ] {
        root(&published, name).unwrap();
    }
}

/// A relation holding production and test-only rows is tainted row by row: a production
/// root reads the production rows, is refused an oracle row, and a fixture reads both.
#[test]
fn shared_relation_with_oracle_rows_keeps_production_roots_admissible() {
    let text = format!(
        r#"package p {{ {SCHEMA}
 dataset bank: cp complete_over(j in first) provenance(handbook, role.published) {{ [a] = [1.0]; }}
 dataset oracle: cp complete_over(j in second) provenance(upstream, role.oracle_input) {{ [b] = [2.0]; }}
 def Reads(j: item) {{ require cp[j] > 0 : "positive"; }}
 def Production {{ child r: Reads = Reads(j = a); }}
 def Leaky {{ child r: Reads = Reads(j = b); }}
 test compared {{ child bank: Reads = Reads(j = a); child oracle: Reads = Reads(j = b); }}
}}"#
    );
    let p = admitted(&text).unwrap();
    let rows = &p.tables[&p.names["p.cp"]].rows;
    assert_eq!(rows.values().filter(|row| row.test_only).count(), 1);
    root(&text, "Production").unwrap();
    let error = refused(&text, "Leaky");
    assert!(
        error.contains("row cp[b] supplied by oracle with role oracle_input"),
        "{error}"
    );
    root(&text, "compared").unwrap();
}

/// A role that requires lineage needs a nonempty lineage; lineage entries name datasets
/// and sources, and the lineage between datasets is acyclic.
#[test]
fn lineage_facet_requires_acyclic_lineage() {
    let text = format!(
        "package p {{ {SCHEMA}
 dataset base: cp complete_over(j in first) provenance(handbook, role.published) {{ [a] = [1.0]; }}
 dataset refit: cp complete_over(j in second) provenance(handbook, role.fitted, lineage(dataset base, source upstream)) {{ [b] = [2.0]; }}
}}"
    );
    let p = admitted(&text).unwrap();
    let refit = p.provenance(p.names["p.refit"]).unwrap();
    assert!(
        refit
            .role
            .facets
            .contains(&ModelingDataFacet::RequiresLineage)
    );
    assert_eq!(
        refit.lineage,
        [
            (ModelingLineageKind::Dataset, p.names["p.base"]),
            (ModelingLineageKind::Source, p.names["p.upstream"]),
        ]
    );
    let error = refusal(&text.replace(", lineage(dataset base, source upstream)", ""));
    assert!(
        error.contains("role role.fitted requires lineage; refit names none"),
        "{error}"
    );
    // A cycle, through another dataset or directly, is refused with its datasets named.
    let cycle = text.replace(
        "provenance(handbook, role.published)",
        "provenance(handbook, role.published, lineage(dataset refit))",
    );
    let error = refusal(&cycle);
    assert!(
        error.contains("lineage is cyclic: base, refit derive from one another"),
        "{error}"
    );
    let error = refusal(&text.replace("lineage(dataset base,", "lineage(dataset refit,"));
    assert!(error.contains("lineage is cyclic: refit"), "{error}");
    // Entries name what their kind says.
    let error = refusal(&text.replace("dataset base,", "dataset items,"));
    assert!(
        error.contains("lineage entry items is not a dataset"),
        "{error}"
    );
    let error = refusal(&text.replace("source upstream)", "source remark)"));
    assert!(
        error.contains("remark is a note, which does not"),
        "{error}"
    );
}

/// Data derived from test-only data is test-only, whatever its own role: a dataset whose
/// lineage reaches test-only data, through other datasets or through a source that itself
/// references test-only data, supplies test-only rows, so a derived dataset cannot launder
/// oracle data into production. A fixture reads it; lineage to published data taints nothing.
#[test]
fn lineage_from_test_only_data_is_test_only() {
    let text = format!(
        r#"package p {{ {SCHEMA}
 table fit[j: item]: Scalar complete_over(j);
 table refit[j: item]: Scalar complete_over(j);
 dataset frozen: cp complete_over(j in items) provenance(upstream, role.oracle_input) {{ [a] = [1.0]; [b] = [2.0]; }}
 dataset derived: fit complete_over(j in first) provenance(handbook, role.fitted, lineage(dataset frozen)) {{ [a] = [3.0]; }}
 dataset again: refit complete_over(j in first) provenance(handbook, role.published, lineage(dataset derived)) {{ [a] = [4.0]; }}
 entity kind form {{ key subject: item; attribute value: Scalar; }}
 dataset forms: form provenance(upstream, role.oracle_input) {{ [a] = [5.0]; }}
 entity kind digest extends source {{ attribute basis: form; }}
 entity digest summary {{ title = "a summary of oracle rows", basis = form[a] }}
 dataset summarized: fit complete_over(j in second) provenance(handbook, role.fitted, lineage(source summary)) {{ [b] = [6.0]; }}
 constant k: Scalar = 7.0 provenance(handbook, role.fitted, lineage(dataset again));
 def Derived {{ require fit[a] > 0 : "positive"; }}
 def Again {{ require refit[a] > 0 : "positive"; }}
 def Summarized {{ require fit[b] > 0 : "positive"; }}
 def Constant {{ require k > 0 : "positive"; }}
 test fixture {{ require fit[a] + refit[a] + fit[b] + k > 0 : "positive"; }}
}}"#
    );
    let p = admitted(&text).unwrap();
    for data in [
        "p.frozen",
        "p.derived",
        "p.again",
        "p.forms",
        "p.summarized",
        "p.k",
    ] {
        assert!(p.supplies_test_only(p.names[data]), "{data}");
    }
    assert!(p.is_test_only(p.names["p.summary"]) && p.is_test_only(p.names["p.k"]));
    // Each root outside a fixture reading derived data is refused, naming its dataset.
    let error = refused(&text, "Derived");
    assert!(
        error.contains(
            "row fit[a] supplied by derived with role fitted, whose lineage reaches test-only data"
        ),
        "{error}"
    );
    assert!(refused(&text, "Again").contains(
        "row refit[a] supplied by again with role published, whose lineage reaches test-only data"
    ));
    assert!(refused(&text, "Summarized").contains("row fit[b] supplied by summarized"));
    assert!(refused(&text, "Constant").contains("constant k supplied by k with role fitted"));
    root(&text, "fixture").unwrap();
    // Lineage to published data taints nothing.
    let published = text.replace("role.oracle_input", "role.published");
    let p = admitted(&published).unwrap();
    assert!(
        ["p.derived", "p.again", "p.summarized", "p.k"]
            .iter()
            .all(|data| !p.supplies_test_only(p.names[*data]))
    );
    for name in ["Derived", "Again", "Summarized", "Constant"] {
        root(&published, name).unwrap();
    }
}

/// A role is a package enumeration member: a new one, with any name, is a package edit, and
/// the kernel acts only on the facets it declares.
#[test]
fn new_role_is_a_package_only_edit() {
    let text = format!(
        r#"package p {{ {SCHEMA}
 enum bank_role {{ predicted }}
 dataset guess: cp complete_over(j in items) provenance(handbook, bank_role.predicted) {{ [a] = [1.0]; [b] = [2.0]; }}
 def Production {{ require cp[a] > 0 : "positive"; }}
}}"#
    );
    let p = admitted(&text).unwrap();
    let role = &p.provenance(p.names["p.guess"]).unwrap().role;
    assert_eq!(role.enumeration, p.names["p.bank_role"]);
    root(&text, "Production").unwrap();
    // Declaring a facet on the new member is what makes the kernel act, under any name.
    let frozen = text.replace("{ predicted }", "{ predicted facets(test_only) }");
    assert!(refused(&frozen, "Production").contains("with role predicted"));
    let renamed = frozen.replace("predicted", "frozen_reference");
    assert!(refused(&renamed, "Production").contains("with role frozen_reference"));
    let fitted = text.replace("{ predicted }", "{ predicted facets(requires_lineage) }");
    assert!(refusal(&fitted).contains("requires lineage"));
}

/// A constant is data with provenance: it names its source, role and lineage, and a
/// test-only constant is read by test fixtures only.
#[test]
fn constants_carry_provenance() {
    let text = format!(
        r#"package p {{ {SCHEMA}
 constant r: Scalar = 8.314 provenance(handbook, role.published);
 constant fitted: Scalar = 2.0 provenance(handbook, role.fitted, lineage(source upstream));
 def Root {{ require r * fitted > 0 : "positive"; }}
}}"#
    );
    let p = admitted(&text).unwrap();
    let r = p.provenance(p.names["p.r"]).unwrap();
    assert_eq!(r.source, p.names["p.handbook"]);
    assert!(!p.is_test_only(p.names["p.r"]));
    assert_eq!(
        p.provenance(p.names["p.fitted"]).unwrap().lineage,
        [(ModelingLineageKind::Source, p.names["p.upstream"])]
    );
    root(&text, "Root").unwrap();
    let error = refusal(&text.replace(", lineage(source upstream)", ""));
    assert!(
        error.contains("requires lineage; fitted names none"),
        "{error}"
    );
    let error = refusal(&text.replace(" provenance(handbook, role.published);", ";"));
    assert!(error.contains("provenance"), "{error}");
    let error = refusal(&text.replace(
        "provenance(handbook, role.published)",
        "provenance(remark, role.published)",
    ));
    assert!(
        error.contains("remark is a note, which does not"),
        "{error}"
    );
    let frozen = text.replace(
        "provenance(handbook, role.published)",
        "provenance(upstream, role.oracle_input)",
    );
    assert!(refused(&frozen, "Root").contains("constant r supplied by r with role oracle_input"));
}
