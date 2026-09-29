// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Relations with constraints: three-phase admission, completeness, symmetry, uniqueness,
//! integer-range keys, derived columns, row requirements and keyed-row references
//! (ADR-0123 Outcome 3, Plan 23 KR5).
use crate::kernel_types::{physical, try_source};
use crate::specialize::Value;
use crate::*;

fn admitted(text: &str) -> Result<CheckedPackage> {
    let (registry, _) = physical();
    let context = TypeContext {
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
    let p = admitted(text)?;
    specialize(
        &p,
        p.names["p.Root"],
        InstanceId::from_bytes([7; 16]),
        &Bindings::default(),
        Limits::default(),
    )
}
fn entity(p: &CheckedPackage, name: &str, kind: &str) -> Value {
    Value::Entity {
        id: p.names[name],
        kind: p.names[kind],
    }
}
fn number(p: &CheckedPackage, table: &str, keys: &[Value]) -> f64 {
    let id = p.names[table];
    p.tables[&id].rows[keys].cells[0].scalar(id).unwrap()
}

const ITEMS: &str = "entity kind item {} entity item a {} entity item b {} entity item c {}
 set items: Set<item> = {a, b};";

/// A row reference `table[keys]` must name an admitted row of that table, and an entity
/// reference an admitted entity or keyed row.
#[test]
fn row_reference_requires_existing_target_row() {
    let text = format!(
        r#"package p {{ {ITEMS}
 table link[j: item]: {{target: Row<base>, label: Text}} complete_over(j in items);
 dataset l: link source "s" {{ [a] = [base[b], "first"]; [b] = [base[a], "second"]; }}
 table base[j: item]: {{x: Scalar}} complete_over(j in items);
 dataset d: base source "s" {{ [a] = [1.0]; [b] = [2.0]; }}
 def Root {{ require link[a].target.x == 2 : "resolved through the reference"; }}
}}"#
    );
    // The referencing table precedes its target in the source: identities need no order.
    let p = admitted(&text).unwrap();
    let link = &p.tables[&p.names["p.link"]];
    let Value::Row { table, fields, .. } = &link.rows[&vec![entity(&p, "p.a", "p.item")]].cells[0]
    else {
        panic!("row reference expected")
    };
    assert_eq!(*table, p.names["p.base"]);
    assert_eq!(fields[0].scalar(*table).unwrap(), 2.);
    specialized(&text).unwrap();
    // An entity without a row of the target table.
    let error = refusal(&text.replace("[a] = [base[b], ", "[a] = [base[c], "));
    assert!(error.contains("row reference base[c] names no admitted row of base"), "{error}");
    // An undeclared entity, and a row of another table.
    assert!(refusal(&text.replace("base[b], \"first\"", "base[z], \"first\"")).contains("unknown entity z"));
    assert!(
        refusal(&text.replace("base[b], \"first\"", "link[b], \"first\"")).contains("is not a row of table base")
    );
    // A keyed-row reference names an admitted keyed row.
    let keyed = r#"package p { entity kind item {} entity item a {}
 entity kind form { key subject: item; key variant: Integer = 1; }
 dataset f: form source "s" { [a] = []; }
 set items: Set<item> = {a};
 table choice[j: item]: form complete_over(j in items);
 dataset d: choice source "s" { [a] = [form[a, 2]]; }
}"#;
    let error = refusal(keyed);
    assert!(error.contains("no form row has the keys [a, 2]"), "{error}");
    admitted(&keyed.replace("form[a, 2]", "form[a]")).unwrap();
}

/// Rows of one relation may reference one another, in any declaration order, when the
/// references are acyclic at row level; a cycle is refused with its rows named. This holds
/// for keyed kinds (fitted parameter-set lineage) and for tables (reference columns).
#[test]
fn self_referential_lineage_admits_and_cycles_are_refused() {
    let lineage = r#"package p {
 entity kind parameter_set { key name: Text; attribute lineage: parameter_set?; attribute value: Scalar; }
 dataset sets: parameter_set source "s" {
  ["refit"] = [parameter_set["fitted"], 3.0];
  ["fitted"] = [parameter_set["base"], 2.0];
  ["base"] = [missing, 1.0];
 }
}"#;
    let p = admitted(lineage).unwrap();
    let kind = p.names["p.parameter_set"];
    let row = |name: &str| entity::keyed_identity(kind, &[Value::Text(name.into())]);
    assert_eq!(
        p.record(row("refit")).unwrap().values["lineage"],
        Value::Entity {
            id: row("fitted"),
            kind
        }
    );
    assert_eq!(p.record(row("base")).unwrap().values["lineage"], Value::Missing);
    let error = refusal(&lineage.replace("[missing, 1.0]", "[parameter_set[\"refit\"], 1.0]"));
    for name in [
        "rows reference one another in a cycle",
        "parameter_set[\"base\"]",
        "parameter_set[\"fitted\"]",
        "parameter_set[\"refit\"]",
    ] {
        assert!(error.contains(name), "{name}: {error}");
    }
    // A table whose rows reference their own table derives in reference order.
    let tree = format!(
        r#"package p {{ {ITEMS}
 set all: Set<item> = {{a, b, c}};
 table tree[j: item]: {{parent: Row<tree>?, derived depth: Integer = if present(parent) then parent.depth + 1 else 0}} complete_over(j in all);
 dataset t: tree source "s" {{ [c] = [tree[b]]; [b] = [tree[a]]; [a] = [missing]; }}
}}"#
    );
    let p = admitted(&tree).unwrap();
    let table = &p.tables[&p.names["p.tree"]];
    let depth = |name: &str| table.rows[&vec![entity(&p, name, "p.item")]].cells[1].clone();
    assert_eq!(
        [depth("p.a"), depth("p.b"), depth("p.c")],
        [Value::Integer(0), Value::Integer(1), Value::Integer(2)]
    );
    let error = refusal(&tree.replace("[a] = [missing]", "[a] = [tree[c]]"));
    assert!(error.contains("rows reference one another in a cycle"), "{error}");
    for row in ["tree[a]", "tree[b]", "tree[c]"] {
        assert!(error.contains(row), "{row}: {error}");
    }
}

/// A required table is complete over the declared sets of its completeness, checked at
/// admission: the table's own claim over every row, a dataset's claim for open keys over
/// that dataset's rows. Rows outside the sets are admitted.
#[test]
fn completeness_over_declared_sets_checked_at_admission() {
    let text = format!(
        r#"package p {{ {ITEMS}
 enum Phase {{ liquid, vapor }}
 table t[j: item, q: Phase, k: 0..1]: Scalar complete_over(j in items, q in Phase, k in 0..1);
 dataset d: t source "s" {{
  [a, liquid, 0] = [1.0]; [a, liquid, 1] = [1.0]; [a, vapor, 0] = [1.0]; [a, vapor, 1] = [1.0];
  [b, liquid, 0] = [1.0]; [b, liquid, 1] = [1.0]; [b, vapor, 0] = [1.0]; [b, vapor, 1] = [1.0];
  [c, liquid, 0] = [9.0];
 }}
}}"#
    );
    let p = admitted(&text).unwrap();
    assert_eq!(p.tables[&p.names["p.t"]].rows.len(), 9);
    let error = refusal(&text.replace("[b, vapor, 1] = [1.0];", ""));
    assert!(
        error.contains("t is declared complete over its sets by t, but no row has the keys [b, vapor, 1]"),
        "{error}"
    );
    // An open key is claimed by each dataset over its own rows.
    let open = format!(
        r#"package p {{ {ITEMS}
 set first: Set<item> = {{a}};
 table u[j: item]: Scalar complete_over(j);
 dataset e: u complete_over(j in items) source "s" {{ [a] = [1.0]; [b] = [2.0]; }}
 dataset f: u complete_over(j in first) source "s" {{ [c] = [3.0]; }}
}}"#
    );
    let error = refusal(&open);
    assert!(
        error.contains("but no row of dataset f has the keys [a]"),
        "{error}"
    );
    let p = admitted(&open.replace("j in first", "j in singles").replace(
        "set first: Set<item> = {a};",
        "set singles: Set<item> = {c};",
    ))
    .unwrap();
    let Some(data::Absence::Required(claims)) = p.tables.get(&p.names["p.u"]).map(|t| &t.absence)
    else {
        panic!("required")
    };
    assert_eq!(claims.len(), 2);
    // A dataset claims only open keys, and names each of them.
    assert!(refusal(&text.replace("dataset d: t source", "dataset d: t complete_over(j in items) source")).contains("claims only open keys"));
    assert!(refusal(&open.replace("complete_over(j in first)", "complete_over(k in first)")).contains("not an open key"));
    // A completeness set's members are keys of the key's type.
    assert!(refusal(&text.replace("q in Phase", "q in items")).contains("is not a key q"));
}

/// Required is complete: a required table declares its completeness, and completeness
/// belongs to a required table.
#[test]
fn required_without_completeness_refused() {
    let base = format!("package p {{ {ITEMS} table t[j: item]: Scalar POLICY; }}");
    for policy in ["", "missing required"] {
        let error = refusal(&base.replace("POLICY", policy));
        assert!(error.contains("t is required but declares no completeness"), "{error}");
    }
    // An open key needs no dataset; a closed completeness needs its rows.
    for policy in ["missing optional", "missing default 0.0", "complete_over(j)"] {
        admitted(&base.replace("POLICY", policy)).unwrap();
    }
    assert!(refusal(&base.replace("POLICY", "complete_over(j in items)")).contains("no row has the keys [a]"));
    for policy in [
        "complete_over(j in items) missing optional",
        "complete_over(j in items) missing default 0.0",
    ] {
        assert!(refusal(&base.replace("POLICY", policy)).contains("completeness belongs to a required table"));
    }
    // Completeness names every key exactly once.
    let pair = format!("package p {{ {ITEMS} table t[i: item, j: item]: Scalar COMPLETE; }}");
    assert!(refusal(&pair.replace("COMPLETE", "complete_over(i in items)")).contains("names every key"));
    assert!(refusal(&pair.replace("COMPLETE", "complete_over(i in items, i)")).contains("twice"));
}

/// A symmetric pair is stored once, in canonical orientation, and a lookup answers both
/// orders; a table excluding its diagonal answers its default there.
#[test]
fn symmetric_pair_answers_both_orientations() {
    let text = format!(
        r#"package p {{ {ITEMS}
 table kij[i: item, j: item]: Scalar symmetric(i, j) diagonal excluded missing default 0.0;
 dataset d: kij source "s" {{ [b, a] = [0.5]; [c, b] = [0.25]; }}
 def Root {{
  require kij[a, b] == 0.5 : "canonical";
  require kij[b, a] == 0.5 : "reversed";
  require kij[b, c] == 0.25 : "other pair";
  require kij[a, a] == 0 : "diagonal takes the default";
 }}
}}"#
    );
    let p = admitted(&text).unwrap();
    assert_eq!(p.tables[&p.names["p.kij"]].rows.len(), 2);
    specialized(&text).unwrap();
    // A required symmetric table is complete over unordered pairs, the diagonal excluded.
    let required = format!(
        r#"package p {{ {ITEMS}
 table pair[i: item, j: item]: Scalar symmetric(i, j) diagonal excluded complete_over(i in items, j in items);
 dataset d: pair source "s" {{ [b, a] = [0.5]; }}
 def Root {{ require pair[a, b] == pair[b, a] : "both orders"; }}
}}"#
    );
    specialized(&required).unwrap();
    let error = specialized(&required.replace("pair[a, b] == pair[b, a]", "pair[a, a] == 0.5"))
        .unwrap_err()
        .to_string();
    assert!(error.contains("outside the completeness of pair"), "{error}");
    // With the diagonal allowed, completeness covers it.
    let error = refusal(&required.replace("diagonal excluded", "diagonal allowed"));
    assert!(error.contains("no row has the keys [a, a]"), "{error}");
    // A symmetric pair names two keys of one type.
    assert!(
        refusal(&format!("package p {{ {ITEMS} table t[i: item, k: Integer]: Scalar symmetric(i, k) missing optional; }}"))
            .contains("two distinct keys of one type")
    );
}

/// A symmetric pair is written once: both orientations, in one dataset or two, and a row
/// on an excluded diagonal are refused.
#[test]
fn symmetric_pair_with_both_orientations_refused() {
    let text = format!(
        r#"package p {{ {ITEMS}
 table kij[i: item, j: item]: Scalar symmetric(i, j) diagonal excluded missing default 0.0;
 dataset d: kij source "s" {{ [a, b] = [0.5]; }}
 dataset e: kij source "s" {{ [c, a] = [0.1]; }}
}}"#
    );
    admitted(&text).unwrap();
    for changed in [
        text.replace("[a, b] = [0.5];", "[a, b] = [0.5]; [b, a] = [0.5];"),
        text.replace("[c, a] = [0.1];", "[b, a] = [0.5];"),
    ] {
        let error = refusal(&changed);
        assert!(error.contains("symmetric pair kij[a, b] is written in both orientations"), "{error}");
    }
    let error = refusal(&text.replace("[c, a] = [0.1];", "[c, c] = [0.1];"));
    assert!(error.contains("lies on the diagonal, which kij excludes"), "{error}");
    // The same orientation twice is a duplicate, not a symmetry violation.
    let error = refusal(&text.replace("[c, a] = [0.1];", "[a, b] = [0.1];"));
    assert!(error.contains("kij[a, b] is supplied twice"), "{error}");
}

/// A uniqueness constraint names keys and supplied columns; two rows sharing its tuple are
/// refused, and an absent value never collides.
#[test]
fn unique_constraint_rejects_duplicate_tuple() {
    let text = format!(
        r#"package p {{ {ITEMS}
 set all: Set<item> = {{a, b, c}};
 table code[j: item]: {{cas: Text?, rank: Integer}} unique(cas) unique(rank, j) complete_over(j in all);
 dataset d: code source "s" {{ [a] = ["71-43-2", 1]; [b] = ["108-88-3", 1]; [c] = [missing, 2]; }}
}}"#
    );
    admitted(&text).unwrap();
    admitted(&text.replace("[c] = [missing, 2]", "[c] = [missing, 3]").replace("\"108-88-3\"", "missing")).unwrap();
    let error = refusal(&text.replace("\"108-88-3\"", "\"71-43-2\""));
    assert!(
        error.contains("rows code[a] and code[b] share (cas) = (\"71-43-2\"), which unique(cas) declares unique"),
        "{error}"
    );
    assert!(refusal(&text.replace("unique(cas)", "unique(weight)")).contains("names no key or supplied column"));
}

/// A derived column is evaluated once per row at admission, with the row's keys and
/// columns bound, in the order of its value dependencies: derived columns of one row by
/// what they read, and tables by the tables they read, whatever their declaration order.
#[test]
fn derived_column_evaluated_once_per_row() {
    let text = format!(
        r#"package p {{ {ITEMS}
 table total[j: item]: {{n: Integer, derived doubled: Integer = scaled * 2, derived scaled: Integer = n * factor[j].k}} complete_over(j in items);
 dataset d: total source "s" {{ [a] = [1]; [b] = [2]; }}
 table factor[j: item]: {{base: Integer, derived k: Integer = base + 1}} complete_over(j in items);
 dataset f: factor source "s" {{ [a] = [2]; [b] = [3]; }}
 def Root {{ require total[b].doubled == 16 : "derived once, read by lookup"; }}
}}"#
    );
    let p = admitted(&text).unwrap();
    let table = &p.tables[&p.names["p.total"]];
    let row = |name: &str| table.rows[&vec![entity(&p, name, "p.item")]].cells.to_vec();
    // The derived values are stored in the admitted rows: a lookup reads them.
    assert_eq!(row("p.a"), [Value::Integer(1), Value::Integer(6), Value::Integer(3)]);
    assert_eq!(row("p.b"), [Value::Integer(2), Value::Integer(16), Value::Integer(8)]);
    specialized(&text).unwrap();
    // A dataset supplies only the columns that are not derived.
    assert!(refusal(&text.replace("[a] = [1];", "[a] = [1, 6, 3];")).contains("supplies 1 columns"));
    // Derived columns that read one another in a cycle are refused with their names.
    let error = refusal(&text.replace("n * factor[j].k", "doubled + n"));
    assert!(error.contains("derived columns doubled, scaled of total derive from one another"), "{error}");
    // Tables deriving from one another are refused with their names.
    let error = refusal(&text.replace("base + 1", "base + total[j].n"));
    assert!(error.contains("tables factor, total derive their values from one another"), "{error}");
    // A derived value of the wrong type is refused before any row is evaluated.
    let error = refusal(&text.replace("base + 1", "1.5"));
    assert!(error.contains("derived column k of factor is "), "{error}");
}

/// A row requirement is a typed predicate over the row, evaluated at admission; a refusal
/// names the row, its dataset and the clause.
#[test]
fn row_requirement_names_row_and_clause() {
    let text = format!(
        r#"package p {{ {ITEMS}
 table limits[j: item]: {{lower: Scalar, upper: Scalar}} complete_over(j in items) require lower <= upper require j in items;
 dataset d: limits source "s" {{ [a] = [1.0, 2.0]; [b] = [2.0, 3.0]; }}
}}"#
    );
    admitted(&text).unwrap();
    let error = refusal(&text.replace("[b] = [2.0, 3.0]", "[b] = [3.0, 2.0]"));
    assert!(
        error.contains("row limits[b] of dataset d violates the requirement `lower <= upper`"),
        "{error}"
    );
    // A requirement over an enumeration-valued relation and its keys' attributes.
    let roles = r#"package p {
 entity kind species { attribute charge: Scalar = 0; }
 entity species water {} entity species sodium { charge = 1 } entity species chloride { charge = -1 }
 enum ComponentType { solvent, cation, anion }
 set members: Set<species> = {water, sodium, chloride};
 table component_role[j: species]: ComponentType complete_over(j in members)
  require not (value == ComponentType.cation) or j.charge > 0
  require not (value == ComponentType.anion) or j.charge < 0;
 dataset roles: component_role source "s" { [water] = [solvent]; [sodium] = [cation]; [chloride] = [anion]; }
}"#;
    admitted(roles).unwrap();
    let error = refusal(&roles.replace("[chloride] = [anion]", "[chloride] = [cation]"));
    assert!(error.contains("row component_role[chloride] of dataset roles violates the requirement `not (value == ComponentType.cation) or j.charge > 0`"), "{error}");
    // A requirement is a Boolean predicate, checked even without rows.
    assert!(admitted(&text.replace("require j in items", "require lower + upper")).is_err());
}

/// A lookup of a required table outside its completeness is refused on the declarations,
/// before any row is read: here the row of `c` exists, but `c` lies outside the set.
#[test]
fn lookup_outside_completeness_set_refused_before_evaluation() {
    let text = format!(
        r#"package p {{ {ITEMS}
 set more: Set<item> = {{a, c}};
 table w[j: item]: Scalar complete_over(j in items);
 dataset d: w source "s" {{ [a] = [1.0]; [b] = [2.0]; [c] = [3.0]; }}
 def Root {{ var x[j in DOMAIN]: Scalar; eq e[j in DOMAIN]: x[j] == w[j]; }}
}}"#
    );
    let model = specialized(&text.replace("DOMAIN", "items")).unwrap();
    assert_eq!(model.equations.len(), 2);
    let error = specialized(&text.replace("DOMAIN", "more"))
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("lookup w[c] is outside the completeness of w"),
        "{error}"
    );
    // An optional table answers absence there instead; its lookups are guarded.
    let p = admitted(
        &text
            .replace("eq e[j in DOMAIN]: x[j] == w[j];", "")
            .replace("DOMAIN", "items")
            .replace("complete_over(j in items)", "missing optional"),
    )
    .unwrap();
    assert!(p.tables[&p.names["p.w"]].optional());
    assert_eq!(number(&p, "p.w", &[entity(&p, "p.c", "p.item")]), 3.);
}

/// A keyed-row reference cell names a keyed kind's row by its key cells, resolved at
/// admission to the row's identity and concrete kind: a selection points at a parameter
/// set, through its refined form or the key-declaring kind, trailing defaults omitted.
#[test]
fn keyed_row_reference_cell_resolves_by_key() {
    let text = r#"package p {
 entity kind species {} entity species benzene {} entity species toluene {}
 enum PhaseType { liquid, vapor }
 entity kind parameter_set { key subject: species; key phase: PhaseType; key variant: Integer = 1; }
 entity kind constant_cp extends parameter_set { attribute c: Scalar; }
 dataset forms: constant_cp source "s" { [benzene, liquid] = [2.0]; [toluene, liquid, 2] = [3.0]; }
 set components: Set<species> = {benzene, toluene};
 table selection[j: species]: parameter_set complete_over(j in components)
  require value.subject == j;
 dataset chosen: selection source "s" { [benzene] = [constant_cp[benzene, liquid]]; [toluene] = [parameter_set[toluene, liquid, 2]]; }
 def Root { require selection[toluene].variant == 2 : "the selected row"; }
}"#;
    let p = admitted(text).unwrap();
    let benzene = entity(&p, "p.benzene", "p.species");
    let liquid = Value::Enum {
        enumeration: p.names["p.PhaseType"],
        member: p.declarations[&p.names["p.PhaseType"]]
            .value
            .enumeration
            .as_ref()
            .unwrap()
            .members[0]
            .member_id,
    };
    let identity = entity::keyed_identity(
        p.names["p.parameter_set"],
        &[benzene.clone(), liquid, Value::Integer(1)],
    );
    assert_eq!(
        p.tables[&p.names["p.selection"]].rows[&vec![benzene]].cells[0],
        Value::Entity {
            id: identity,
            kind: p.names["p.constant_cp"]
        }
    );
    specialized(text).unwrap();
    // The row must exist, with keys of the key-declaring kind's types.
    let error = refusal(&text.replace("constant_cp[benzene, liquid]", "constant_cp[benzene, vapor]"));
    assert!(error.contains("no constant_cp row has the keys [benzene, vapor, 1]"), "{error}");
    assert!(refusal(&text.replace("constant_cp[benzene, liquid]", "constant_cp[liquid, benzene]")).contains("unknown entity liquid"));
    // The row requirement checks the reference against its keys.
    let error = refusal(&text.replace("[toluene] = [parameter_set[toluene, liquid, 2]]", "[toluene] = [constant_cp[benzene, liquid]]"));
    assert!(error.contains("row selection[toluene] of dataset chosen violates the requirement `value.subject == j`"), "{error}");
    // A kind without keys, and a declared entity, are not keyed rows.
    assert!(refusal(&text.replace("constant_cp[benzene, liquid]", "species[benzene]")).contains("has no keys"));
}

/// An integer-range key admits only its declared integers, in rows, completeness and
/// lookups.
#[test]
fn integer_range_key_admits_declared_range_only() {
    let text = r#"package p {
 entity kind cubic_family {} entity cubic_family pr {} entity cubic_family srk {}
 set peng_robinson: Set<cubic_family> = {pr};
 table kappa[f: cubic_family, k: 0..2]: Scalar complete_over(f, k in 0..2);
 dataset pr_kappa: kappa complete_over(f in peng_robinson) source "Peng and Robinson 1976" { [pr, 0] = [0.37464]; [pr, 1] = [1.54226]; [pr, 2] = [-0.26992]; }
 def Root { require kappa[pr, INDEX] < 0 : "the quadratic coefficient"; }
}"#;
    let p = admitted(&text.replace("INDEX", "2")).unwrap();
    assert_eq!(
        p.tables[&p.names["p.kappa"]].keys[1].range,
        Some((0, 2))
    );
    specialized(&text.replace("INDEX", "2")).unwrap();
    let error = refusal(&text.replace("[pr, 2] = [-0.26992];", "[pr, 2] = [-0.26992]; [pr, 3] = [0.1];"));
    assert!(error.contains("key k = 3 of kappa is outside its declared range 0..2"), "{error}");
    let error = specialized(&text.replace("INDEX", "3")).unwrap_err().to_string();
    assert!(error.contains("key k = 3 of table kappa is outside its declared range 0..2"), "{error}");
    // Completeness over a range lies within the key's range, and the claim is checked.
    assert!(refusal(&text.replace("k in 0..2)", "k in 0..3)")).contains("lies outside the key's integers"));
    let error = refusal(&text.replace(" [pr, 1] = [1.54226];", ""));
    assert!(error.contains("no row of dataset pr_kappa has the keys [pr, 1]"), "{error}");
    // Only an integer key is complete over a range.
    assert!(
        refusal("package p { table g[x: Text]: Scalar complete_over(x in 0..1); }")
            .contains("outside the key's integers")
    );
}
