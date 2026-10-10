// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use crate::specialize::Value;

fn checked(text: &str) -> Result<CheckedPackage> {
    let (quantities, _) = crate::kernel_types::physical();
    let preconditions = pse_quantity::PhysicalPreconditions::new(vec![]).unwrap();
    check(
        &crate::kernel_types::try_source(text)?,
        &TypeContext {
            admissions: None,
            formula_authority: None,
            quantities: &quantities,
            preconditions: &preconditions,
            scope: &crate::PhysicalScope::default(),
        },
    )
}

fn names(package: &CheckedPackage) -> BTreeSet<&str> {
    package.names.keys().map(String::as_str).collect()
}

#[test]
fn selected_closure_ranges_follow_reached_declarations_not_unrelated_occurrences() {
    for reached in [1, 5, 17] {
        for unrelated in [0, 7, 40] {
            let equations = (0..reached)
                .map(|i| format!("eq e{i}:x=={i};"))
                .collect::<String>();
            let functions = (0..unrelated)
                .map(|i| format!("fn unused{i}(x:Scalar)->Scalar=x+{i};"))
                .collect::<String>();
            let package = checked(&format!(
                "package p {{def Root {{var x:Scalar; {equations}}} {functions}}}"
            ))
            .unwrap();
            let root = package.entry("p.Root").unwrap();
            let x = package.entry("p.Root.x").unwrap();
            let selected = package.select(root).unwrap();
            let expected = ["p".to_owned(), "p.Root".into(), "p.Root.x".into()]
                .into_iter()
                .chain((0..reached).map(|i| format!("p.Root.e{i}")))
                .collect::<BTreeSet<_>>();
            assert_eq!(
                selected.names.keys().cloned().collect::<BTreeSet<_>>(),
                expected
            );
            assert_eq!(selected.declarations.len(), reached + 3);
            assert_eq!(selected.expressions.len(), reached);
            for i in 0..reached {
                let equation = package.entry(&format!("p.Root.e{i}")).unwrap();
                let occurrence = selected
                    .expression_occurrence(equation, "equation.expression", 0)
                    .unwrap();
                assert_eq!(occurrence.dependencies, BTreeSet::from([x]));
                assert_eq!(
                    occurrence,
                    package
                        .expression_occurrence(equation, "equation.expression", 0)
                        .unwrap()
                );
            }
            // The projected indexes support another selection without consulting the source.
            assert_eq!(selected.select(root).unwrap(), selected);
        }
    }
}

#[test]
fn selected_closure_keeps_imports_inherited_defaults_and_lexical_shadowing() {
    let package = checked(r#"package library {
        interface Base {param inherited:Scalar=2;}
        fn helper(x:Scalar)->Scalar=x;
        param duplicate:Scalar=7;
        param unused:Scalar=99;
    }
    package consumer {
        use library @"1.0.0" as lib;
        def Root:lib.Base {param duplicate:Scalar=3; var x:Scalar; eq e:x==lib.helper(duplicate+inherited);}
    }"#).unwrap();
    let root = package.entry("consumer.Root").unwrap();
    let selected = package.select(root).unwrap();
    assert_eq!(
        names(&selected),
        BTreeSet::from([
            "library",
            "library.Base",
            "library.Base.inherited",
            "library.helper",
            "consumer",
            "consumer.library",
            "consumer.Root",
            "consumer.Root.duplicate",
            "consumer.Root.x",
            "consumer.Root.e",
        ])
    );
    assert_eq!(
        selected.resolve(root, "duplicate"),
        package.entry("consumer.Root.duplicate")
    );
    assert_eq!(
        selected.resolve(root, "inherited"),
        package.entry("library.Base.inherited")
    );
    assert_eq!(
        selected.resolve(root, "lib.helper"),
        package.entry("library.helper")
    );
    assert_eq!(selected.resolve(root, "library.helper"), None);
    assert_eq!(
        selected.members[&root]["inherited"],
        package.entry("library.Base.inherited").unwrap()
    );
    assert_eq!(selected.select(root).unwrap(), selected);
}

#[test]
fn selected_closure_rechecks_membership_additions_deletions_and_absent_names() {
    let source = |local: &str, value: &str| {
        format!(
            "package p {{param amount:Scalar={value}; def Root {{ {local} var x:Scalar; eq e:x==amount;}} }}"
        )
    };
    let inherited = checked(&source("", "1")).unwrap();
    let root = inherited.entry("p.Root").unwrap();
    let outer = inherited.entry("p.amount").unwrap();
    let before = inherited.select(root).unwrap();
    assert!(before.declarations.contains_key(&outer));

    let shadowed = checked(&source("param amount:Scalar=2;", "1")).unwrap();
    let local = shadowed.entry("p.Root.amount").unwrap();
    let after = shadowed.select(root).unwrap();
    assert!(after.declarations.contains_key(&local));
    assert!(!after.declarations.contains_key(&outer));
    assert_eq!(after.resolve(root, "amount"), Some(local));
    let equation = after.entry("p.Root.e").unwrap();
    assert!(
        after
            .expression_occurrence(equation, "equation.expression", 0)
            .unwrap()
            .dependencies
            .contains(&local)
    );

    let deleted = checked(&source("", "1")).unwrap().select(root).unwrap();
    assert_eq!(deleted, before);
    let changed = checked(&source("", "3")).unwrap().select(root).unwrap();
    assert_ne!(changed, before);
    assert!(checked("package p {def Root {var x:Scalar; eq e:x==amount;}}").is_err());
    let removed = checked("package p {param amount:Scalar=1;}").unwrap();
    assert!(
        removed
            .select(root)
            .unwrap_err()
            .to_string()
            .contains("selected root absent")
    );
}

#[test]
fn selected_closure_keeps_refined_dataset_records_identifiers_provenance_and_taint() {
    let package = checked(
        r#"package p {
        identifier scheme code;
        entity kind source provenance {} entity source origin {}
        enum role {published, oracle facets(test_only)}
        entity kind base {key name:Integer; attribute code:Id<code>; attribute amount:Scalar;}
        entity kind derived extends base {}
        dataset bank:derived provenance(origin,role.oracle) {[1]=[Id<code>("A"),2];}
        entity kind other {key name:Text; attribute code:Id<code>;}
        dataset ignored:other provenance(origin,role.published) {["other"]=[Id<code>("B")];}
        def Root {require base[1].amount>0 : "positive";}
    }"#,
    )
    .unwrap();
    let root = package.entry("p.Root").unwrap();
    let selected = package.select(root).unwrap();
    let base = package.entry("p.base").unwrap();
    let derived = package.entry("p.derived").unwrap();
    let bank = package.entry("p.bank").unwrap();
    let code = package.entry("p.code").unwrap();
    let origin = package.entry("p.origin").unwrap();
    let row = crate::entity::keyed_identity(base, &[Value::Integer(1)]);
    assert_eq!(
        selected.entities.keys().copied().collect::<BTreeSet<_>>(),
        BTreeSet::from([row, origin])
    );
    assert_eq!(selected.entities[&row].kind, derived);
    assert_eq!(selected.entities[&row].origin, bank);
    assert_eq!(selected.identified(code, "A"), Some(row));
    assert_eq!(selected.identified(code, "B"), None);
    assert_eq!(selected.provenance(bank), package.provenance(bank));
    assert!(selected.test_only.contains(&row));
    assert!(selected.test_only_data.contains(&bank));
    assert!(selected.entry("p.ignored").is_none());
    assert!(selected.entry("p.other").is_none());
    assert!(matches!(
        crate::specialize(
            &selected,
            root,
            crate::InstanceId::from_bytes([7; 16]),
            &crate::Bindings::default(),
            crate::Limits::default(),
        ),
        Err(crate::ModelingError::TestOnly { .. })
    ));
    assert_eq!(selected.select(root).unwrap(), selected);
}

#[test]
fn selected_closure_keeps_every_table_supplier_and_excludes_unrelated_tables() {
    let package = checked(
        r#"package p {
        entity kind item {} entity item a {} entity item b {}
        table values[j:item]:Scalar complete_over(j);
        dataset first:values provenance(s, role.given) {[a]=[1];}
        dataset second:values provenance(s, role.given) {[b]=[2];}
        table ignored[j:item]:Scalar complete_over(j);
        dataset unused:ignored provenance(s, role.given) {[a]=[99];}
    }"#,
    )
    .unwrap();
    let table = package.entry("p.values").unwrap();
    let selected = package.select(table).unwrap();
    assert_eq!(selected.tables[&table], package.tables[&table]);
    assert_eq!(selected.tables[&table].rows.len(), 2);
    for name in [
        "p.first", "p.second", "p.a", "p.b", "p.item", "p.s", "p.role",
    ] {
        assert!(selected.entry(name).is_some(), "missing {name}");
    }
    assert!(selected.entry("p.ignored").is_none());
    assert!(selected.entry("p.unused").is_none());
    assert_eq!(selected.select(table).unwrap(), selected);
}

#[test]
fn selected_closure_refreshes_refinement_supplier_membership_after_addition_and_deletion() {
    let source = |suppliers: &str| {
        format!(
            r#"package p {{
        entity kind base {{key name:Text; attribute value:Scalar;}}
        entity kind refined extends base {{}}
        entity kind other {{key name:Text; attribute value:Scalar;}}
        dataset first:refined provenance(s, role.given) {{["one"]=[1];}}
        {suppliers}
    }}"#
        )
    };
    let baseline = checked(&source("")).unwrap();
    let base = baseline.entry("p.base").unwrap();
    let one = crate::entity::keyed_identity(base, &[Value::Text("one".into())]);
    let two = crate::entity::keyed_identity(base, &[Value::Text("two".into())]);
    let before = baseline.select(base).unwrap();
    assert!(before.entities.contains_key(&one));
    assert!(!before.entities.contains_key(&two));

    let extra = r#"dataset second:refined provenance(s, role.given) {["two"]=[2];}"#;
    let added = checked(&source(extra)).unwrap().select(base).unwrap();
    assert!(added.entry("p.second").is_some());
    assert!(added.entities.contains_key(&one));
    assert!(added.entities.contains_key(&two));

    // A supplier moved to a different kind no longer supplies the selected ancestor.
    let moved = checked(&source(&extra.replace("second:refined", "second:other")))
        .unwrap()
        .select(base)
        .unwrap();
    assert!(moved.entry("p.second").is_none());
    assert!(moved.entry("p.other").is_none());
    assert!(moved.entities.contains_key(&one));
    assert!(!moved.entities.contains_key(&two));
    // The enclosing package's source range changes; the selected supplier products do not.
    assert_eq!(moved.names, before.names);
    assert_eq!(moved.entities, before.entities);
    assert_eq!(moved.provenance, before.provenance);
    assert_eq!(moved.selection_index, before.selection_index);
    assert_eq!(checked(&source("")).unwrap().select(base).unwrap(), before);
}
