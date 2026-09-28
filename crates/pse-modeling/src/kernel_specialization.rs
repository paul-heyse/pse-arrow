// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use crate::kernel_types::{physical, source};
use crate::*;
use pse_ids::SemanticId;
use specialize::{Value, symbol_name};

fn run(text: &str, root: &str, bindings: Bindings) -> Result<SpecializedModel> {
    let (registry, names) = physical();
    let c = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        names: &names,
    };
    let p = check(&source(text), &c)?;
    specialize(
        &p,
        p.names[root],
        InstanceId::from_bytes([7; 16]),
        &bindings,
        Limits::default(),
    )
}

#[test]
fn implicit_members_remain_addressable_through_child_paths_and_indexed_arguments() {
    let text = "package p {
        entity kind item {}
        entity item a {}
        entity item b {}
        set items:Set<item>={a,b};
        fn first(xs:Scalar[item])->Scalar=xs[a];
        def Inner {
            implicit block {
                var x[j in items]:Scalar;
                var y:Scalar;
                eq indexed[j in items]:x[j]==2;
                eq scalar:y==4;
            }
            realize policy on block using inline;
            let selected:Scalar=block.x[a];
            let gathered:Scalar=first(block.x);
            let scalar_value:Scalar=block.y;
            annotation report block.x(\"implicit outputs\");
        }
        def Root {
            child inner:Inner=Inner();
            eq same:inner.selected+inner.gathered==inner.scalar_value;
        }
    }";
    let model = run(text, "p.Root", Bindings::default()).unwrap();
    assert_eq!(model.implicit.len(), 1);
    assert_eq!(model.equations.len(), 4);
    assert_eq!(model.annotations.len(), 2);
    assert_eq!(
        model
            .symbols
            .values()
            .filter(|symbol| {
                symbol.role == pse_model::generated::enums::ModelingDeclarationKind::Variable
                    && symbol.lineage.path.contains(".block.")
            })
            .count(),
        3
    );
    for invalid in [
        text.replace("block.x[a]", "block[a].x[a]"),
        text.replace("block.y;", "block.y.member;"),
    ] {
        assert!(run(&invalid, "p.Root", Bindings::default()).is_err());
    }
}

#[test]
fn multiplicative_literals_keep_operand_units_in_typed_and_static_expressions() {
    let (registry, names) = physical();
    let preconditions =
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .unwrap();
    let context = TypeContext {
        quantities: &registry,
        names: &names,
        preconditions: &preconditions,
    };
    let execute = |text: &str| -> Result<SpecializedModel> {
        let package = check(&source(text), &context)?;
        specialize(
            &package,
            package.names["p.Root"],
            InstanceId::from_id(SemanticId::NIL),
            &Bindings::default(),
            Limits::default(),
        )
    };
    let text = "package p {
        fn ratio()->Scalar=2{s}/4{s};
        fn frequency_product()->Scalar=2{s}*3{1/s};
        fn square()->Scalar=(2{s}/1{s})^2;
        def Child(r:Scalar,p:Scalar) {
            require r==0.5 and p==6 : \"canonical static operands\";
            var x:Scalar;
            eq e:x==ratio()+frequency_product()+square();
        }
        def Root {child inner:Child=Child(r=2{s}/4{s},p=2{s}*3{1/s});}
    }";
    let model = execute(text).unwrap();
    assert_eq!(model.equations.len(), 1);
    // The result contract does not authorize reinterpreting a time as scalar.
    let invalid = text.replace("fn ratio()->Scalar=2{s}/4{s}", "fn ratio()->Scalar=2{s}");
    assert!(execute(&invalid).is_err());
}

#[test]
fn kernel_diamond_keeps_the_most_specific_checked_member() {
    for bases in ["B,C", "C,B"] {
        let text = format!(
            "package p {{ interface A {{let x:Scalar;}} interface B extends A {{let x:Scalar=2;}} interface C extends A {{}} def Root:{bases} {{eq e:x==2;}} }}"
        );
        let (registry, names) = physical();
        let preconditions = pse_quantity::PhysicalPreconditions::new(vec![]).unwrap();
        let c = TypeContext {
            preconditions: &preconditions,
            quantities: &registry,
            names: &names,
        };
        let p = check(&source(&text), &c).unwrap();
        assert_eq!(
            p.members[&p.names["p.Root"]]["x"],
            p.members[&p.names["p.B"]]["x"]
        );
        run(&text, "p.Root", Bindings::default()).unwrap();
        let conflict = text.replace(
            "interface C extends A {}",
            "interface C extends A {let x:Scalar=3;}",
        );
        assert!(check(&source(&conflict), &c).is_err());
        let explicit = conflict.replace("eq e:x==2;", "override let x:Scalar=4; eq e:x==4;");
        assert!(check(&source(&explicit), &c).is_ok());
    }
}

#[test]
fn kernel_anonymous_contracts_compose_across_inherited_and_guarded_scopes() {
    for bases in ["interface B extends A", "interface B"] {
        let root = if bases.ends_with(" A") { "B" } else { "A,B" };
        let text = format!(
            "package p {{ interface A {{var x:Scalar; annotation start x(1); expect x==1 tolerance 1e-8; when true {{require true : \"A required\";}}}} {bases} {{var y:Scalar; annotation start y(2); expect y==2 tolerance 1e-8; when true {{require true : \"B required\";}}}} def Root:{root} {{eq e:x+y==3;}}}}"
        );
        let model = run(&text, "p.Root", Bindings::default()).unwrap();
        assert_eq!(model.annotations.len(), 2);
        assert_eq!(model.expectations.len(), 2);
        let targets = model
            .annotations
            .iter()
            .map(|a| a.target)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(targets.len(), 2);
        for label in ["A", "B"] {
            let invalid = text.replace(
                &format!("require true : \"{label} required\""),
                &format!("require false : \"{label} required\""),
            );
            assert!(run(&invalid, "p.Root", Bindings::default()).is_err());
        }
    }
}

#[test]
fn kernel_child_family_annotations_use_target_membership_and_author_scope() {
    let text = "package p {entity kind item {} entity item a {} entity item b {} set all:Set<item>={a,b}; set chosen:Set<item>={b}; interface Child {param members:Set<item>; var x[j in members]:Scalar;} def Concrete(selected:Set<item>):Child {override param members:Set<item>=selected;} def Root {param seed[j in all]:Scalar=2; child inner:Child=Concrete(selected=chosen); eq e:inner.x[b]==3; annotation start inner.x(seed[j]+1);}}";
    let model = run(text, "p.Root", Bindings::default()).unwrap();
    assert_eq!(model.annotations.len(), 1);
    let target = &model.symbols[&model.annotations[0].target];
    assert_eq!(target.lineage.path, "Root.inner.x");
    assert!(
        model
            .symbols
            .values()
            .any(|s| s.lineage.path == "Root.seed")
    );
    assert!(
        run(
            &text.replace("inner.x(seed[j]+1)", "inner.x(seed[1]+1)"),
            "p.Root",
            Bindings::default()
        )
        .is_err()
    );
    assert!(
        run(
            &text.replace("inner.x(seed[j]+1)", "inner.missing(seed[j]+1)"),
            "p.Root",
            Bindings::default()
        )
        .is_err()
    );
}
#[test]
fn kernel_function_validity_is_typed_pure_and_roundtrips() {
    let text =
        "package p {fn f(x:Scalar)->Scalar valid(x>0)=x*x; def Root {var x:Scalar; eq e:f(x)==1;}}";
    let rows = source(text);
    let rendered = pse_authoring::language::render(&rows).unwrap();
    assert_eq!(
        rows.iter().map(|r| &r.value).collect::<Vec<_>>(),
        source(&rendered)
            .iter()
            .map(|r| &r.value)
            .collect::<Vec<_>>()
    );
    let model = run(text, "p.Root", Bindings::default()).unwrap();
    assert!(model.functions.values().any(|f| f.validity.is_some()));
    for invalid in [
        text.replace("x>0", "x>1{mol/s}"),
        text.replace("x>0", "f(x)>0"),
        text.replace("fn f", "param limit:Scalar=1; fn f")
            .replace("x>0", "x>limit"),
    ] {
        assert!(
            run(&invalid, "p.Root", Bindings::default()).is_err(),
            "{invalid}"
        );
    }
}
#[test]
fn kernel_regimes_share_unknowns_and_keep_alternative_equations_and_hints_separate() {
    let text = "package p { def Root { param target:Scalar=2; implicit roots select minimum((y-target)*(y-target),1e-8) { var y:Scalar; regime negative eligible(y<0) { eq root:y==-1; annotation start y(-1); annotation bounds y(-2,0); } regime positive eligible(y>0) { eq root:y==1; annotation start y(1); annotation bounds y(0,2); } } realize r on roots using nested; } }";
    let rows = source(text);
    let rendered = pse_authoring::language::render(&rows).unwrap();
    assert_eq!(
        rows.iter().map(|r| &r.value).collect::<Vec<_>>(),
        source(&rendered)
            .iter()
            .map(|r| &r.value)
            .collect::<Vec<_>>()
    );
    let model = run(text, "p.Root", Bindings::default()).unwrap();
    let selection = model.regimes.values().next().unwrap();
    assert_eq!(selection.alternatives.len(), 2);
    assert!(model.equations.is_empty());
    assert!(model.annotations.is_empty());
    let unknown = model
        .symbols
        .values()
        .find(|s| s.role == pse_model::generated::enums::ModelingDeclarationKind::Variable)
        .unwrap()
        .id;
    assert!(selection.alternatives.iter().all(|r| r.equations.len() == 1
        && r.annotations.len() == 2
        && r.annotations.iter().all(|a| a.target == unknown)));
    assert_ne!(selection.alternatives[0].id, selection.alternatives[1].id);
    for invalid in [
        text.replace(",1e-8)", ",1e-8{mol/s})"),
        text.replace("select minimum((y-target)*(y-target),1e-8)", ""),
        text.replace("eq root:y==1;", ""),
        text.replace("using nested", "using inline"),
        text.replace(
            "regime positive eligible(y>0) {",
            "regime positive eligible(y>0) { var duplicate:Scalar;",
        ),
    ] {
        assert!(
            run(&invalid, "p.Root", Bindings::default()).is_err(),
            "{invalid}"
        );
    }
}
#[test]
fn lazy_members_pull_only_owned_defining_rows() {
    let m=run("package p { def D { param p: Scalar = 2; var x: Scalar defined by x == p; var unused: Scalar defined by unused == p; eq demand: x == p; } }","p.D",Bindings::default()).unwrap();
    assert_eq!(m.symbols.len(), 2);
    assert_eq!(m.equations.len(), 2);
    assert!(
        m.symbols
            .values()
            .all(|s| !s.lineage.path.ends_with("unused"))
    );
    assert!(
        m.symbols
            .values()
            .find(|s| s.lineage.path.ends_with(".x"))
            .unwrap()
            .lineage
            .demand
            .len()
            >= 2
    );
}
#[test]
fn effective_default_and_override_determine_demand() {
    let m=run("package p { interface I { let answer: Scalar = source; var source: Scalar; } def D : I { override let answer: Scalar = other; var other: Scalar; eq e: answer == other; } }","p.D",Bindings::default()).unwrap();
    let answer = m
        .symbols
        .values()
        .find(|s| s.lineage.path.ends_with("answer"))
        .unwrap();
    assert!(answer.lineage.is_override);
    assert!(
        m.symbols
            .values()
            .any(|s| s.lineage.path.ends_with("other"))
    );
}
#[test]
fn recursion_is_separate_from_simultaneous_equations() {
    assert!(
        run(
            "package p { def D { let a: Scalar = b; let b: Scalar = a; eq e: a == b; } }",
            "p.D",
            Bindings::default()
        )
        .is_err()
    );
    let m=run("package p { def D { var a: Scalar defined by a == b; var b: Scalar defined by b == a; eq demand: a == 1; } }","p.D",Bindings::default()).unwrap();
    assert_eq!(m.symbols.len(), 2);
    assert_eq!(m.equations.len(), 3);
}
#[test]
fn indexed_dispatch_groups_implementations() {
    let m = run(
        r#"package p {
 entity kind thing {} entity thing a {} entity thing b {} entity thing c {}
 set things: Set<thing> = {a,b,c};
 interface I { var x: Scalar; }
 def A : I { var x: Scalar; } def B : I { var x: Scalar; }
 table method[j: thing]: I;
 dataset choices: method source "synthetic" { [a] = [A()]; [b] = [B()]; [c] = [A()]; }
 def Root { child items[j in things]: I = method[j]; }
 }"#,
        "p.Root",
        Bindings::default(),
    )
    .unwrap();
    assert_eq!(m.instances.len(), 4);
    let sizes = m
        .groups
        .values()
        .map(|g| g.instances.len())
        .collect::<Vec<_>>();
    assert_eq!(sizes.iter().sum::<usize>(), 4);
    assert!(sizes.contains(&2));
    assert_eq!(m.groups.len(), 3);
}
#[test]
fn accumulator_closure_and_transfer_controls() {
    let m=run("package p { def D { var x: Scalar; accumulate a: Scalar conservation tolerance 1e-8; accumulate b: Scalar accounting tolerance 1e-8; contribute a role transfer transfer pair out = x; contribute b role transfer transfer pair in = x; } }","p.D",Bindings::default()).unwrap();
    assert_eq!(m.closures.len(), 2);
    assert_eq!(m.equations.len(), 1);
    assert!(m.closures.values().all(|c| c.terms.len() == 1));
    assert!(run("package p { def D { var x: Scalar; accumulate a: Scalar conservation tolerance 1e-8; contribute a role transfer transfer pair out = x; } }","p.D",Bindings::default()).is_err());
    assert!(
        run(
            "package p { def D { accumulate a: Scalar conservation tolerance 1e-8; } }",
            "p.D",
            Bindings::default()
        )
        .is_err()
    );
    assert!(run("package p { def D { var x: MassFlow; accumulate a: Flow conservation tolerance 1e-8{mol/s}; contribute a role inflow = x; } }","p.D",Bindings::default()).is_err());
}
#[test]
fn static_guards_requirements_and_budget() {
    let text = "package p { def D(enabled: Boolean = false) { var x: Scalar; when enabled { eq extra: x == 1; } require enabled == false : \"expected disabled\"; } }";
    let m = run(text, "p.D", Bindings::default()).unwrap();
    assert!(m.equations.is_empty());
    let mut bindings = Bindings::default();
    bindings
        .arguments
        .insert("enabled".into(), Value::Boolean(true));
    assert!(run(text, "p.D", bindings).is_err());
    let (registry, names) = physical();
    let c = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        names: &names,
    };
    let p = check(&source(text), &c).unwrap();
    assert!(
        specialize(
            &p,
            p.names["p.D"],
            InstanceId::from_id(SemanticId::NIL),
            &Bindings::default(),
            Limits {
                items: 1,
                ..Limits::default()
            }
        )
        .is_err()
    );
    assert!(!symbol_name(SemanticId::NIL).is_empty());
}

#[test]
fn lexical_locals_are_shared_and_shadow_without_capture() {
    let m=run("package p { def D { var x: Scalar; let result: Scalar = ((b + a) where a = 3) where a = x, b = a + 1; eq e: result == x + 4; } }","p.D",Bindings::default()).unwrap();
    let result = m
        .symbols
        .values()
        .find(|s| s.lineage.path.ends_with(".result"))
        .unwrap();
    let text = pse_authoring::dsl::render_expr(result.expression.as_ref().unwrap());
    assert!(text.contains("__local_1"));
    assert!(text.contains("__local_2"));
    assert!(text.contains("__local_3"));
    assert_eq!(text.matches("where").count(), 2);
}

#[test]
fn raw_closure_detects_a_violation_independent_of_residual_values() {
    let m=run("package p { def D { var x: Scalar; var y: Scalar; accumulate total: Scalar conservation tolerance 0.01; contribute total role inflow = x; contribute total role outflow = y; } }","p.D",Bindings::default()).unwrap();
    let terms = &m.closures.values().next().unwrap().terms;
    let mut magnitudes = std::collections::BTreeMap::from([(terms[0].id, 2.0), (terms[1].id, 2.0)]);
    assert_eq!(
        m.assess_closure(&magnitudes).unwrap()[0].satisfied,
        Some(true)
    );
    magnitudes.insert(terms[1].id, 1.5);
    assert_eq!(
        m.assess_closure(&magnitudes).unwrap()[0].satisfied,
        Some(false)
    );
    magnitudes.remove(&terms[0].id);
    assert!(m.assess_closure(&magnitudes).is_err());
}

#[test]
fn lazy_definition_cannot_capture_the_demanders_local() {
    let m=run("package p { def D { var x: Scalar; let a: Scalar = x; let b: Scalar = a where x = 17; eq demand: b == x; } }","p.D",Bindings::default()).unwrap();
    let x = m
        .symbols
        .values()
        .find(|s| s.lineage.path.ends_with(".x"))
        .unwrap();
    let a = m
        .symbols
        .values()
        .find(|s| s.lineage.path.ends_with(".a"))
        .unwrap();
    assert_eq!(
        pse_authoring::dsl::render_expr(a.expression.as_ref().unwrap()),
        symbol_name(x.id)
    );
}

#[test]
fn indexed_attributes_rows_enums_and_optional_guards() {
    let text = r#"package p {
 entity kind item { attribute count: Integer = 2; }
 entity item a {} entity item b { count = 3 }
 set items: Set<item> = {a,b};
 enum Choice { small, large }
 table coeff[j:item]: {count: Integer, value: Scalar} missing optional;
 dataset data: coeff source "synthetic" { [a] = [2,4]; }
 def Root(choice: Choice = Choice.small) {
  var x[j in items]: Scalar;
  eq e[j in items]: x[j] == x[j];
  require a.count == 2 : "default attribute";
  require b.count == 3 : "explicit attribute";
  when present(coeff[a]) { require coeff[a].count == a.count : "row"; }
  when present(coeff[b]) { require coeff[b].count == b.count : "missing guard"; }
  when choice == Choice.small { eq selected: x[a] == 4; }
 }
 }"#;
    let model = run(text, "p.Root", Bindings::default()).unwrap();
    assert_eq!(model.symbols.len(), 2);
    assert_eq!(model.equations.len(), 3);
    assert!(
        run(
            &text.replace("when present(coeff[b]) { require", "when true { require"),
            "p.Root",
            Bindings::default()
        )
        .is_err()
    );
    assert!(
        run(
            &text.replace("x[j] == x[j]", "x[4] == x[j]"),
            "p.Root",
            Bindings::default()
        )
        .is_err()
    );
}

#[test]
fn bounded_expansion_observes_caller_cancellation() {
    let (registry, names) = physical();
    let context = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        names: &names,
    };
    let package = check(
        &source("package p { def D { var a: Scalar; var b: Scalar; var c: Scalar; } }"),
        &context,
    )
    .unwrap();
    let calls = std::cell::Cell::new(0);
    let cancelled = || {
        calls.set(calls.get() + 1);
        calls.get() > 3
    };
    let result = specialize::specialize_cancellable(
        &package,
        package.names["p.D"],
        InstanceId::from_id(SemanticId::NIL),
        &Bindings::default(),
        Limits::default(),
        &cancelled,
    );
    assert!(matches!(result, Err(ModelingError::Cancelled)));
}

#[test]
fn finite_set_projection_filter_product_and_relation_image() {
    let text = r#"package p {
 entity kind item {} entity item a {} entity item b {} entity item c {}
 set left: Set<item> = {a,b}; set right: Set<item> = {b,c};
 set all: Set<item> = union(left,right);
 set pairs: Set<Tuple<item,item>> = product(left,right);
 set selected: Set<item> = {j for j in all where j != c};
 table edge[from:item,to:item]: Scalar;
 dataset graph: edge source "synthetic" { [a,b] = [1]; [a,c] = [1]; [b,c] = [1]; }
 set image: Set<item> = {at(pair,1) for pair in keys(edge) where at(pair,0) == a};
 def Root {
  require size(all) == 3 : "union";
  require size(pairs) == 4 : "product";
  require size(selected) == 2 : "filter";
  require size(image) == 2 : "image";
  var x[j in image]: Scalar;
 }
 }"#;
    let model = run(text, "p.Root", Bindings::default()).unwrap();
    assert_eq!(model.symbols.len(), 2);
    assert!(
        run(
            &text.replace("at(pair,1)", "at(pair,2)"),
            "p.Root",
            Bindings::default()
        )
        .is_err()
    );
}

#[test]
fn annotations_resolve_existing_targets_and_physical_expressions() {
    let text = r#"package p { def Root { var x: Flow; var y: Flow;
 annotation start x(2{mol/s}); annotation start y(x);
 annotation bounds x(0{mol/s}, 10{mol/s}); annotation report x("flow");
 annotation valid x(0{mol/s}, 20{mol/s}, reject); annotation check x(x >= 0{mol/s});
 } }"#;
    let model = run(text, "p.Root", Bindings::default()).unwrap();
    assert_eq!(model.symbols.len(), 2);
    assert_eq!(model.annotations.len(), 6);
    assert!(
        model
            .annotations
            .iter()
            .all(|a| model.symbols.contains_key(&a.target))
    );
    assert!(
        run(
            &text.replace("2{mol/s}", "2{kg/s}"),
            "p.Root",
            Bindings::default()
        )
        .is_err()
    );
}

#[test]
fn shared_dispatch_bodies_exclude_numeric_initial_values() {
    let text = "package p { def D { param p: Scalar = 2; var x: Scalar; eq e: x == p; } def Root { child a = D(); child b = D(); } }";
    let before = run(text, "p.Root", Bindings::default()).unwrap();
    let after = run(
        &text.replace("Scalar = 2", "Scalar = 3"),
        "p.Root",
        Bindings::default(),
    )
    .unwrap();
    assert_eq!(
        before.groups.keys().collect::<Vec<_>>(),
        after.groups.keys().collect::<Vec<_>>()
    );
    let group = before
        .groups
        .values()
        .find(|g| g.instances.len() == 2)
        .unwrap();
    assert_eq!(group.body.equations.len(), 1);
    let (expressions, equations) = before.bound_bodies().unwrap();
    assert!(expressions.is_empty());
    assert_eq!(equations.len(), 2);
    for row in &before.equations {
        assert!(row.equation.structural_eq(&equations[&row.id]));
    }
    assert!(
        before
            .symbols
            .values()
            .zip(after.symbols.values())
            .any(|(a, b)| a.initial != b.initial)
    );
}
#[test]
fn typed_ports_connect_existing_coordinates_across_children() {
    let text = "package p { interface I { var flow: Flow; port inlet: Flow = flow; annotation connectivity inlet(1,0); port outlet: Flow = flow; annotation connectivity outlet(0,1); } def D : I {} def Root { child a: I = D(); child b: I = D(); connect a.outlet -> b.inlet; } }";
    let m = run(text, "p.Root", Bindings::default()).unwrap();
    assert_eq!(m.symbols.len(), 2);
    assert_eq!(m.ports.len(), 4);
    assert_eq!(m.connections.len(), 1);
    let connection = m.connections.values().next().unwrap();
    assert_ne!(
        m.ports[&connection.from].id,
        m.ports[&connection.from].symbol
    );
    assert_ne!(
        m.ports[&connection.from].lineage.instance,
        m.ports[&connection.to].lineage.instance
    );
    assert_eq!(m.equations[0].id, connection.id);
    assert_eq!(m.equations.len(), 1);
    assert!(
        run(
            &text.replace("port inlet: Flow = flow", "port inlet: MassFlow = flow"),
            "p.Root",
            Bindings::default()
        )
        .is_err()
    );
}

#[test]
fn structural_integer_literals_keep_all_bits_and_reject_overflow() {
    let text = r#"package p { def Root(n: Integer = 9007199254740993, minimum: Integer = -9223372036854775808) {
 require n == 9007199254740993 : "all bits";
 require n + 1 == 9007199254740994 : "exact arithmetic";
 require minimum + 1 == -9223372036854775807 : "signed minimum";
 } }"#;
    run(text, "p.Root", Bindings::default()).unwrap();
    let changed = text.replace("n + 1 == 9007199254740994", "n + 1 == 9007199254740993");
    assert!(run(&changed, "p.Root", Bindings::default()).is_err());
    let overflow = text
        .replace(
            "n: Integer = 9007199254740993",
            "n: Integer = 9223372036854775807",
        )
        .replace("require n == 9007199254740993 : \"all bits\";", "");
    assert!(run(&overflow, "p.Root", Bindings::default()).is_err());
}

#[test]
fn presets_bind_before_demand_and_retain_lineage() {
    let text = "package p { def Base(enabled: Boolean = false) { var x: Scalar; when enabled { eq e: x == 1; } } preset Active = Base(enabled=true); def Root { child d = Active(); } }";
    let m = run(text, "p.Root", Bindings::default()).unwrap();
    assert_eq!(m.equations.len(), 1);
    assert_eq!(m.equations[0].lineage.presets.len(), 1);
    let mut bindings = Bindings::default();
    bindings
        .arguments
        .insert("enabled".into(), Value::Boolean(false));
    assert!(
        run(text, "p.Active", bindings)
            .unwrap()
            .equations
            .is_empty()
    );
    let (registry, names) = physical();
    let c = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        names: &names,
    };
    assert!(check(&source("package p { preset A = B(); preset B = A(); }"), &c).is_err());
}

#[test]
fn presets_preserve_the_bound_definition_contract() {
    let text = "package p { def Base(enabled:Boolean=false) {var x:Scalar; when enabled {eq e:x==1;}} preset Active=Base(enabled=true); preset Twice=Active(); def Root {child d:Base=Twice();} }";
    let model = run(text, "p.Root", Bindings::default()).unwrap();
    assert_eq!(model.equations.len(), 1);
    assert_eq!(model.equations[0].lineage.presets.len(), 2);
    let different = text
        .replace("def Root", "def Other {} def Root")
        .replace("child d:Base", "child d:Other");
    assert!(run(&different, "p.Root", Bindings::default()).is_err());
}

#[test]
fn requirements_reduce_typed_tables_and_reject_bad_data() {
    let text = r#"package p { entity kind item {} entity item a {} entity item b {} set items: Set<item>={a,b};
 table amount[j:item]: Scalar; dataset rows: amount source "synthetic" {[a]=[2];[b]=[3];}
 def Root { require abs(sum(j in items | amount[j])-5) < 0.01 : "table closure"; }
 }"#;
    assert!(
        run(text, "p.Root", Bindings::default())
            .unwrap()
            .symbols
            .is_empty()
    );
    assert!(
        run(
            &text.replace("[b]=[3]", "[b]=[4]"),
            "p.Root",
            Bindings::default()
        )
        .is_err()
    );
}

#[test]
fn expression_expansion_spends_the_shared_item_budget() {
    let text = "package p { entity kind item {} entity item a {} entity item b {} set items: Set<item>={a,b}; def Root { eq e:sum(i in items | sum(j in items | sum(k in items | 1))) == 8; } }";
    let (registry, names) = physical();
    let c = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        names: &names,
    };
    let p = check(&source(text), &c).unwrap();
    assert!(matches!(
        specialize(
            &p,
            p.names["p.Root"],
            InstanceId::from_id(SemanticId::NIL),
            &Bindings::default(),
            Limits {
                items: 8,
                ..Limits::default()
            }
        ),
        Err(ModelingError::Budget(_))
    ));
    assert!(
        specialize(
            &p,
            p.names["p.Root"],
            InstanceId::from_id(SemanticId::NIL),
            &Bindings::default(),
            Limits::default()
        )
        .is_ok()
    );
}

#[test]
fn kernel_fixture_specs_are_physical_data_and_oracle_links_are_portable() {
    let text = r#"package p { def D { var t: Temperature; eq e: t == 300{K}; }
 test sample source "idaes-oracle:example::0" revision "2.13.0@source-sha" fixture {
   dof 0; value root.t = 26.85{degC}; lower root.t = 250{K}; upper root.t = 400{K}; free root.t;
 } { child root: D = D(); expect root.t == 300{K} tolerance 0.001{K}; } }"#;
    let rows = source(text);
    let rendered = pse_authoring::language::render(&rows).unwrap();
    let restored = source(&rendered);
    assert_eq!(
        rows.iter().map(|r| &r.value).collect::<Vec<_>>(),
        restored.iter().map(|r| &r.value).collect::<Vec<_>>()
    );
    let m = run(text, "p.sample", Bindings::default()).unwrap();
    let f = m.fixtures.values().next().unwrap();
    let v = &f.specifications["root.t"];
    assert!((v.value.unwrap() - 300.).abs() < 1e-12);
    assert_eq!(v.lower, Some(250.));
    assert_eq!(v.fixed, Some(false));
    assert_eq!(
        f.oracle.as_ref().unwrap().reference,
        "idaes-oracle:example::0"
    );
    for bad in [
        text.replace("free root.t;", "fix root.t; free root.t;"),
        text.replace("26.85{degC}", "1{mol/s}"),
        text.replace("idaes-oracle:example::0", "skill://local/oracle"),
    ] {
        assert!(run(&bad, "p.sample", Bindings::default()).is_err());
    }
}

#[test]
fn kernel_inherited_guarded_members_have_contracts_without_early_activation() {
    let text = "package p { interface Optional {param enabled:Boolean=false; when enabled {var x:Scalar;}} def Child(flag:Boolean):Optional {override param enabled:Boolean=flag; when enabled {eq value:x==2;}} def Root {child yes:Child=Child(flag=true); child no:Child=Child(flag=false); let observed:Scalar=yes.x; annotation report observed(\"selected value\"); annotation report yes.x(\"active child\");} }";
    let model = run(text, "p.Root", Bindings::default()).unwrap();
    assert_eq!(model.equations.len(), 1);
    assert_eq!(
        model
            .symbols
            .values()
            .filter(|s| s.role == pse_model::generated::enums::ModelingDeclarationKind::Variable)
            .count(),
        1
    );
    assert!(
        run(
            &text.replace("observed:Scalar=yes.x", "observed:Scalar=no.x"),
            "p.Root",
            Bindings::default()
        )
        .is_err()
    );
    let ambiguous = text.replace(
        "when enabled {var x:Scalar;}",
        "when enabled {var x:Scalar;} when not enabled {var x:Flow;}",
    );
    assert!(run(&ambiguous, "p.Root", Bindings::default()).is_err());
}

#[test]
fn kernel_connectivity_limits_keep_direction_multiplicity_and_indexed_port_identity() {
    let source = r#"package p {
        entity kind item {} entity item a {} entity item b {} set items:Set<item>={a,b};
        def Source {var x[j in items]:Scalar; port output[j in items]:Scalar=x[j]; annotation connectivity output(0,1);}
        def Sink {var x[j in items]:Scalar; port input[j in items]:Scalar=x[j]; annotation connectivity input(1,0);}
        def Root {child source=Source(); child first=Sink(); child second=Sink();
            connect source.output[a] -> first.input[a];
            connect source.output[b] -> first.input[b];
        }
    }"#;
    let model = run(source, "p.Root", Bindings::default()).unwrap();
    assert_eq!(model.connections.len(), 2);
    assert_eq!(model.connectivity.len(), 6);
    assert_eq!(model.equations.len(), 2);
    let fanout = source.replace(
        "connect source.output[b] -> first.input[b];",
        "connect source.output[a] -> second.input[a];",
    );
    assert!(
        run(&fanout, "p.Root", Bindings::default())
            .unwrap_err()
            .to_string()
            .contains("exceeds declared connectivity")
    );
    // An authored signal contract can allow fanout without changing its quantity type.
    let signal = fanout.replace("connectivity output(0,1)", "connectivity output(0,many)");
    assert_eq!(
        run(&signal, "p.Root", Bindings::default())
            .unwrap()
            .connections
            .len(),
        2
    );
    let duplicate = source.replace(
        "connect source.output[b] -> first.input[b];",
        "connect source.output[b] -> first.input[a];",
    );
    assert!(run(&duplicate, "p.Root", Bindings::default()).is_err());
    let reversed = source.replace(
        "source.output[a] -> first.input[a]",
        "first.input[a] -> source.output[a]",
    );
    assert!(run(&reversed, "p.Root", Bindings::default()).is_err());
    for invalid in [
        source.replace("annotation connectivity output(0,1);", ""),
        source.replace("connectivity output(0,1)", "connectivity x(0,1)"),
        source.replace("connectivity output(0,1)", "connectivity output(-1,1)"),
        source.replace("connectivity output(0,1)", "connectivity output(0.5,1)"),
        source.replace(
            "annotation connectivity output(0,1);",
            "annotation connectivity output(0,1); annotation connectivity output(0,many);",
        ),
    ] {
        assert!(
            run(&invalid, "p.Root", Bindings::default()).is_err(),
            "{invalid}"
        );
    }
    let individually = source.replace(
        "annotation connectivity output(0,1);",
        "annotation connectivity output[a](0,1); annotation connectivity output[b](0,many);",
    );
    assert_eq!(
        run(&individually, "p.Root", Bindings::default())
            .unwrap()
            .connectivity
            .len(),
        6
    );
}
