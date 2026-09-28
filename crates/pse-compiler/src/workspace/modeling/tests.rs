// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use pse_authoring::{
    ParseBudget,
    language::{IdentityPolicy, parse},
};
use pse_modeling::specialize::root_instance;
use std::collections::BTreeSet;
fn source(text: &str) -> Vec<Declaration> {
    parse(
        text,
        SemanticId::from_bytes([81; 16]),
        IdentityPolicy::Named,
        ParseBudget::default(),
    )
    .unwrap()
}
fn setup(
    text: &str,
) -> (
    CompilerWorkspace,
    Vec<Declaration>,
    BTreeMap<String, QuantityTypeId>,
    DeclarationId,
) {
    let input = super::super::tests::inputs();
    let names = BTreeMap::from([(
        "Scalar".into(),
        input.quantities.neutral_dimensionless().unwrap(),
    )]);
    let rows = source(text);
    let root = rows
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let mut workspace = CompilerWorkspace::new(input, WorkspaceLimits::default()).unwrap();
    workspace
        .publish_modeling(rows.clone(), names.clone())
        .unwrap();
    (workspace, rows, names, root)
}
fn admit(workspace: &mut CompilerWorkspace, root: DeclarationId) -> Arc<AdmittedModeling> {
    workspace
        .admit_modeling(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings::default(),
            Limits::default(),
        )
        .unwrap()
}
#[test]
fn kernel_body_construction_limits_are_tracked_without_changing_mathematics() {
    let (mut workspace, _, _, root) = setup(
        "package p {fn square(x:Scalar)->Scalar=x*x; def Root {var x:Scalar; eq e:square(x)+x==2;}}",
    );
    let original = admit(&mut workspace, root);
    let generous = workspace
        .admit_modeling(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings::default(),
            Limits {
                body_occurrences: Some(65536),
                ..Limits::default()
            },
        )
        .unwrap();
    assert_eq!(original.case.key(), generous.case.key());
    assert!(
        workspace
            .admit_modeling(
                root,
                InstanceId::from_id(SemanticId::NIL),
                Bindings::default(),
                Limits {
                    body_occurrences: Some(1),
                    ..Limits::default()
                }
            )
            .is_err()
    );
    assert_eq!(original.case.key(), admit(&mut workspace, root).case.key());
}

#[test]
fn kernel_queries_reuse_math_across_values_unrelated_edits_and_clean_rebuild() {
    let text = "package p { def Root { param p: Scalar = 2; var x: Scalar; eq e: x * p == 6; } def Other { var y: Scalar; eq e: y == 1; } }";
    let (initial, mut rows, names, root) = setup(text);
    let events = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let observed = events.clone();
    let mut workspace = CompilerWorkspace::with_events(
        initial.inputs.clone(),
        initial.limits,
        Some(Box::new(move |event| {
            if matches!(event.kind, salsa::EventKind::WillExecute { .. }) {
                observed.fetch_add(1, Ordering::Relaxed);
            }
        })),
    )
    .unwrap();
    workspace
        .publish_modeling(rows.clone(), names.clone())
        .unwrap();
    let original = admit(&mut workspace, root);
    let executed = events.load(Ordering::Relaxed);
    assert!(executed > 0);
    let catalog = workspace.modeling.as_ref().unwrap().catalog;
    let checked = catalog.checked(&workspace.db).clone();
    workspace
        .publish_modeling(rows.clone(), names.clone())
        .unwrap();
    assert!(Arc::ptr_eq(&checked, catalog.checked(&workspace.db)));
    assert!(Arc::ptr_eq(&original, &admit(&mut workspace, root)));
    assert_eq!(events.load(Ordering::Relaxed), executed);
    let parameter = rows
        .iter_mut()
        .find(|r| r.name == "p" && r.value.binding.is_some())
        .unwrap();
    parameter.value.binding.as_mut().unwrap().expression = Some("3".into());
    workspace
        .publish_modeling(rows.clone(), names.clone())
        .unwrap();
    let values = admit(&mut workspace, root);
    assert!(events.load(Ordering::Relaxed) > executed);
    assert!(Arc::ptr_eq(&original, &values));
    let other = rows
        .iter()
        .find(|r| r.name == "Other")
        .unwrap()
        .declaration_id;
    rows.iter_mut()
        .find(|r| r.parent_id == Some(other) && r.name == "e")
        .unwrap()
        .value
        .equation
        .as_mut()
        .unwrap()
        .expression = "y == 9".into();
    workspace
        .publish_modeling(rows.clone(), names.clone())
        .unwrap();
    assert!(Arc::ptr_eq(&values, &admit(&mut workspace, root)));
    let mut clean = CompilerWorkspace::new(workspace.inputs.clone(), workspace.limits).unwrap();
    clean.publish_modeling(rows.clone(), names.clone()).unwrap();
    assert_eq!(values, admit(&mut clean, root));
    rows.iter_mut()
        .find(|r| r.parent_id == Some(root) && r.name == "e")
        .unwrap()
        .value
        .equation
        .as_mut()
        .unwrap()
        .expression = "x * p == 7".into();
    workspace.publish_modeling(rows, names).unwrap();
    let changed = admit(&mut workspace, root);
    assert!(!Arc::ptr_eq(&values, &changed));
    assert_ne!(values.case.key(), changed.case.key());
}
#[test]
fn kernel_publication_rejects_invalid_batch_atomically() {
    let (mut workspace, mut rows, names, root) =
        setup("package p { def Root { var x: Scalar; eq e: x == 2; } }");
    let old = admit(&mut workspace, root);
    rows.iter_mut()
        .find(|r| r.name == "e")
        .unwrap()
        .value
        .equation
        .as_mut()
        .unwrap()
        .expression = "missing == 2".into();
    assert!(workspace.publish_modeling(rows, names).is_err());
    assert!(Arc::ptr_eq(&old, &admit(&mut workspace, root)));
}
#[test]
fn kernel_admitted_revisions_reuse_checked_state_and_refuse_context_substitution() {
    let (mut workspace, rows, names, root) =
        setup("package p {def Root {var x:Scalar; eq e:x==2;}}");
    let first = workspace
        .publish_modeling(rows.clone(), names.clone())
        .unwrap();
    let mut changed = rows;
    changed
        .iter_mut()
        .find(|r| r.name == "e")
        .unwrap()
        .value
        .equation
        .as_mut()
        .unwrap()
        .expression = "x==3".into();
    let second = workspace.publish_modeling(changed, names).unwrap();
    let catalog = workspace.modeling.as_ref().unwrap().catalog;
    for revision in [&first, &second, &first] {
        workspace
            .publish_modeling_revision(revision.clone())
            .unwrap();
        assert!(Arc::ptr_eq(
            catalog.checked(&workspace.db),
            &revision.checked
        ));
        admit(&mut workspace, root);
    }
    let mut other = workspace.inputs.clone();
    let mut physical = other.quantities.to_builder();
    physical.entity_kind(pse_quantity::EntityKind {
        id: pse_quantity::EntityKindId::from_id(SemanticId::from_bytes([243; 16])),
        name: "extra_kind".into(),
    });
    other.quantities = Arc::new(physical.build().unwrap());
    let mut other = CompilerWorkspace::new(other, WorkspaceLimits::default()).unwrap();
    assert!(other.publish_modeling_revision(first).is_err());
    assert!(other.modeling.is_none());
}

#[test]
fn kernel_physical_prerequisites_survive_admission_and_context_republication() {
    use pse_quantity::{
        InvariantId, PhysicalPrecondition, PhysicalPreconditions, PhysicalRequirement,
    };
    let mut inputs = super::super::tests::inputs();
    let scalar = inputs.quantities.neutral_dimensionless().unwrap();
    let neutral = inputs.quantities.quantity_type(scalar).unwrap();
    let mut kind = inputs.quantities.kind(neutral.key.kind).unwrap().clone();
    kind.id = pse_quantity::QuantityKindId::from_id(SemanticId::from_bytes([244; 16]));
    let mut quantity = neutral.clone();
    quantity.id = QuantityTypeId::from_id(SemanticId::from_bytes([245; 16]));
    quantity.key.kind = kind.id;
    let qualified = quantity.id;
    let prerequisite = InvariantId::from_id(SemanticId::from_bytes([246; 16]));
    let mut operation = inputs
        .quantities
        .operations()
        .find(|op| {
            op.opcode == pse_quantity::Opcode::Pow
                && op.input_kinds == vec![neutral.key.kind, neutral.key.kind]
        })
        .unwrap()
        .clone();
    operation.id = pse_quantity::OperationId::from_id(SemanticId::from_bytes([247; 16]));
    operation.opcode = pse_quantity::Opcode::Div;
    operation.input_kinds = vec![kind.id, kind.id];
    operation.precondition_invariants = vec![prerequisite];
    let mut builder = inputs.quantities.to_builder();
    builder
        .kind(kind)
        .quantity_type(quantity)
        .operation(operation);
    inputs.quantities = Arc::new(builder.build().unwrap());
    let prerequisites = |required| {
        Arc::new(
            PhysicalPreconditions::new(vec![PhysicalPrecondition {
                id: prerequisite,
                operand_positions: vec![0, 1],
                requirement: PhysicalRequirement::OperandQuantityContract {
                    required,
                    match_shape: true,
                },
            }])
            .unwrap(),
        )
    };
    let rows = source(
        "package p { fn ratio(x:Qualified,y:Qualified)->Scalar=x/y; def Root {var x:Qualified; var y:Qualified; eq e:ratio(x,y)==1;} }",
    );
    let names = BTreeMap::from([("Scalar".into(), scalar), ("Qualified".into(), qualified)]);
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let mut workspace = CompilerWorkspace::new(inputs.clone(), WorkspaceLimits::default()).unwrap();
    assert!(
        workspace
            .publish_modeling(rows.clone(), names.clone())
            .is_err()
    );
    inputs.preconditions = prerequisites(qualified);
    workspace.publish(inputs.clone()).unwrap();
    let revision = workspace.publish_modeling(rows, names).unwrap();
    admit(&mut workspace, root);
    inputs.preconditions = prerequisites(scalar);
    assert!(workspace.publish(inputs.clone()).is_err());
    assert!(Arc::ptr_eq(
        &workspace.modeling.as_ref().unwrap().revision,
        &revision
    ));
    let mut foreign = CompilerWorkspace::new(inputs, WorkspaceLimits::default()).unwrap();
    assert!(foreign.publish_modeling_revision(revision).is_err());
}
#[test]
fn kernel_functions_and_repeated_partials_use_library_derivatives() {
    let (mut workspace, _, _, root) = setup(
        "package p { fn cube(x: Scalar) -> Scalar = x*x*x; def Root { var x: Scalar; eq value: cube(x) == 0; eq first: partial(cube, x)(x) == 0; eq second: partial(cube, x, x)(x) == 0; } }",
    );
    let admitted = admit(&mut workspace, root);
    let cancel = Arc::new(AtomicBool::new(false));
    let artifact = fixture(
        &admitted,
        workspace.inputs.quantities.clone(),
        DerivativeOrder::Second,
        &cancel,
    )
    .unwrap();
    let result = artifact
        .worker()
        .evaluate(
            &[2.],
            DerivativeOrder::Second,
            &mut BTreeMap::new(),
            &cancel,
        )
        .unwrap();
    let mut values = result.equations(&admitted);
    values.sort_by(f64::total_cmp);
    assert_eq!(values, vec![8., 12., 12.]);
}
#[test]
fn kernel_partial_preserves_erased_division_guard() {
    let (mut workspace, _, _, root) = setup(
        "package p { fn quotient(x: Scalar) -> Scalar = x/x; def Root { var x: Scalar; eq e: partial(quotient, x)(x) == 0; } }",
    );
    let admitted = admit(&mut workspace, root);
    let cancel = Arc::new(AtomicBool::new(false));
    let artifact = fixture(
        &admitted,
        workspace.inputs.quantities.clone(),
        DerivativeOrder::Value,
        &cancel,
    )
    .unwrap();
    let mut worker = artifact.worker();
    assert!(
        worker
            .evaluate(&[0.], DerivativeOrder::Value, &mut BTreeMap::new(), &cancel)
            .is_err()
    );
    assert_eq!(
        worker
            .evaluate(&[2.], DerivativeOrder::Value, &mut BTreeMap::new(), &cancel)
            .unwrap()
            .equations(&admitted),
        vec![0.]
    );
}

#[test]
fn kernel_indexed_functions_and_coordinate_partials() {
    let (mut workspace, _, _, root) = setup(
        "package p { entity kind item {} entity item a {} entity item b {} set items: Set<item> = {a,b}; fn squares(x: Scalar[item], members: Set<item>) -> Scalar = sum(j in members | x[j]*x[j]); fn nested(x: Scalar[item], members: Set<item>) -> Scalar = squares(x, members); def Root { var x[j in items]: Scalar; eq value: nested(x, items) == 0; eq first: partial(squares, x[a])(x, items) == 0; eq repeated: partial(squares, x[a], x[a])(x, items) == 0; eq mixed: partial(squares, x[a], x[b])(x, items) == 0; } }",
    );
    let admitted = admit(&mut workspace, root);
    assert_eq!(admitted.inputs.len(), 2);
    let cancel = Arc::new(AtomicBool::new(false));
    let artifact = fixture(
        &admitted,
        workspace.inputs.quantities.clone(),
        DerivativeOrder::Second,
        &cancel,
    )
    .unwrap();
    let result = artifact
        .worker()
        .evaluate(
            &[3., 3.],
            DerivativeOrder::Second,
            &mut BTreeMap::new(),
            &cancel,
        )
        .unwrap();
    let mut values = result.equations(&admitted);
    values.sort_by(f64::total_cmp);
    assert_eq!(values, vec![0., 2., 6., 18.]);
}

#[test]
fn kernel_structure_follows_lazy_specialization_and_cancellation_is_transient() {
    let (mut workspace, _, _, root) = setup(
        "package p { def Root { param p: Scalar = 2; var x: Scalar defined by x == p; var unused: Scalar defined by unused == p; } }",
    );
    let bindings = Bindings {
        demand: vec!["x".into()],
        ..Bindings::default()
    };
    let cancel = Arc::new(AtomicBool::new(true));
    assert!(matches!(
        workspace.prepare_modeling_cancellable(
            root,
            InstanceId::from_id(SemanticId::NIL),
            bindings.clone(),
            Limits::default(),
            cancel
        ),
        Err(CompileError::Cancelled)
    ));
    let prepared = workspace
        .prepare_modeling_cancellable(
            root,
            InstanceId::from_id(SemanticId::NIL),
            bindings,
            Limits::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert_eq!(prepared.structure.square.rows.len(), 1);
    assert_eq!(prepared.structure.square.columns.len(), 1);
    assert!(prepared.structure.under.columns.is_empty());
    assert!(prepared.structure.over.rows.is_empty());
    assert_eq!(prepared.model.symbols.len(), 2);
    assert!(prepared.retained_bytes() > 0);
}

#[test]
fn kernel_literals_keep_physical_context_after_source_spans_are_removed() {
    let mut input = super::super::tests::inputs();
    input.quantities = Arc::new(pse_quantity::standard::standard_registry().unwrap());
    let temperature =
        QuantityTypeId::from_id(SemanticId::parse_hex("c64b96975a4a59755f8711d3bf628bc9").unwrap());
    let names = BTreeMap::from([("Temperature".into(), temperature)]);
    let rows = source(
        "package p { fn reference() -> Temperature = 300{K}; def Root { var t: Temperature; let initial: Temperature = 300{K}; eq e: t == initial; eq f: t == reference(); } }",
    );
    let root = rows
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let mut workspace = CompilerWorkspace::new(input, WorkspaceLimits::default()).unwrap();
    workspace.publish_modeling(rows, names).unwrap();
    let admitted = admit(&mut workspace, root);
    assert_eq!(admitted.inputs.len(), 1);
    assert_eq!(admitted.outputs.len(), 4);
    let flag = Arc::new(AtomicBool::new(false));
    let artifact = fixture(
        &admitted,
        workspace.inputs.quantities.clone(),
        DerivativeOrder::Value,
        &flag,
    )
    .unwrap();
    let mut values = artifact
        .worker()
        .evaluate(&[300.], DerivativeOrder::Value, &mut BTreeMap::new(), &flag)
        .unwrap()
        .values;
    values.sort_by(f64::total_cmp);
    assert_eq!(values, vec![0., 0., 300., 300.]);
}

#[test]
fn kernel_compiled_original_terms_feed_independent_closure() {
    let (mut workspace, _, _, root) = setup(
        "package p { def Root { var x: Scalar; var y: Scalar; accumulate total: Scalar conservation tolerance 0.01; contribute total role inflow = x; contribute total role outflow = y; } }",
    );
    let flag = Arc::new(AtomicBool::new(false));
    let prepared = workspace
        .prepare_modeling_cancellable(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings::default(),
            Limits::default(),
            flag.clone(),
        )
        .unwrap();
    let artifact = fixture(
        &prepared.admitted,
        workspace.inputs.quantities.clone(),
        DerivativeOrder::Value,
        &flag,
    )
    .unwrap();
    let input = prepared
        .admitted
        .inputs
        .iter()
        .map(|id| {
            if prepared.model.symbols[id].lineage.path.ends_with(".x") {
                2.0
            } else {
                1.0
            }
        })
        .collect::<Vec<_>>();
    let mut values = artifact
        .worker()
        .evaluate(&input, DerivativeOrder::Value, &mut BTreeMap::new(), &flag)
        .unwrap()
        .values;
    for (meaning, value) in prepared.admitted.outputs.iter().zip(&mut values) {
        if matches!(meaning, ModelingOutput::Equation { .. }) {
            *value = 0.;
        }
    }
    let checks = prepared.assess_closure(&values).unwrap();
    assert_eq!(checks[0].net, 1.);
    assert_eq!(checks[0].satisfied, Some(false));
}

#[test]
fn kernel_child_contract_refinement_preserves_inherited_members() {
    let source = "package p {
        interface Basic {var x:Scalar;}
        interface Rich extends Basic {var y:Scalar;}
        def State:Rich {}
        interface Vessel {child state:Basic; eq inherited:state.x==2;}
        def Root:Vessel {override child state:Rich=State; eq extra:state.y==3;}
    }";
    let (mut workspace, _, _, root) = setup(source);
    let admitted = admit(&mut workspace, root);
    assert_eq!(admitted.case.variables().len(), 2);
    assert_eq!(
        admitted
            .case
            .rows()
            .iter()
            .filter(|r| r.lower == 0.0 && r.upper == 0.0)
            .count(),
        2
    );
    // The replacement must implement the promised refinement; common member names
    // alone do not grant the nominal interface or erase physical contracts.
    for source in [
        source.replace("def State:Rich {}", "def State:Basic {var y:Scalar;}"),
        source.replace("interface Rich extends Basic", "interface Rich"),
        "package p {interface Basic {} interface Rich extends Basic {} interface Base {param state:Basic;} def Root:Base {override param state:Rich;}}".into(),
    ] {
        let input = super::super::tests::inputs();
        let names = BTreeMap::from([("Scalar".into(),input.quantities.neutral_dimensionless().unwrap())]);
        let mut workspace = CompilerWorkspace::new(input,WorkspaceLimits::default()).unwrap();
        let declarations = self::source(&source);
        let root = declarations.iter().find(|r|r.name=="Root").unwrap().declaration_id;
        if workspace.publish_modeling(declarations,names).is_ok() {
            assert!(workspace.admit_modeling(root,InstanceId::from_id(SemanticId::NIL),Bindings::default(),Limits::default()).is_err(), "{source}");
        }
    }
}

#[test]
fn kernel_conservation_assembles_local_derivatives() {
    let source = "package p { def Root { var x:Scalar; var y:Scalar;
            accumulate total:Scalar conservation tolerance 0.01;
            contribute total role inflow=x*x; contribute total role outflow=y*y*y;
            } }";
    let (mut workspace, _, _, root) = setup(source);
    let flag = Arc::new(AtomicBool::new(false));
    let prepared = workspace
        .prepare_modeling_cancellable(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings::default(),
            Limits::default(),
            flag.clone(),
        )
        .unwrap();
    let a = &prepared.admitted;
    let equation = a
        .outputs
        .iter()
        .find_map(|o| {
            if let ModelingOutput::Equation { id, .. } = o {
                Some(*id)
            } else {
                None
            }
        })
        .unwrap();
    let terms = a
        .case
        .instances()
        .iter()
        .filter(|i| {
            i.contributions
                .iter()
                .any(|c| c.target == Target::Row(equation))
        })
        .collect::<Vec<_>>();
    assert_eq!(terms.len(), 2);
    assert!(terms.iter().all(|i| i.slots.len() == 1));
    let plan = a
        .plan(
            &workspace.inputs.quantities,
            DerivativeOrder::Second,
            AssemblyLimits::default(),
            &flag,
        )
        .unwrap();
    let assembly = Arc::new(
        plan.compile(Default::default(), Default::default(), &flag)
            .unwrap(),
    );
    let mut worker = assembly.worker(BTreeMap::new(), flag);
    let values = CaseValues {
        scalars: a
            .inputs
            .iter()
            .map(|id| {
                (
                    *id,
                    if prepared.model.symbols[id].lineage.path.ends_with(".x") {
                        3.0
                    } else {
                        2.0
                    },
                )
            })
            .collect(),
    };
    let row = a.case.rows().iter().position(|r| r.id == equation).unwrap();
    assert_eq!(worker.constraints(&values).unwrap()[row], 1.0);
    let jacobian = worker.jacobian(&values).unwrap().to_dense();
    for (column, id) in plan.columns().iter().enumerate() {
        let x = prepared.model.symbols[id].lineage.path.ends_with(".x");
        assert_eq!(jacobian[(row, column)], if x { 6.0 } else { -12.0 });
    }
    let mut weights = vec![0.0; a.case.rows().len()];
    weights[row] = 1.0;
    let hessian = worker.hessian(&values, 0.0, &weights).unwrap().to_dense();
    for (column, id) in plan.columns().iter().enumerate() {
        let x = prepared.model.symbols[id].lineage.path.ends_with(".x");
        assert_eq!(hessian[(column, column)], if x { 2.0 } else { -12.0 });
    }
}

#[test]
fn kernel_empty_indexed_polymorphic_sum_retains_physical_type() {
    let mut input = super::super::tests::inputs();
    input.quantities = Arc::new(pse_quantity::standard::standard_registry().unwrap());
    let names = BTreeMap::from([(
        "Flow".into(),
        QuantityTypeId::from_id(SemanticId::parse_hex("6df479e3be3bda77c5938727311f1063").unwrap()),
    )]);
    let rows = source(
        "package p { entity kind item {} set items: Set<item> = {}; fn total<Q>(x: Q[item], members: Set<item>)->Q=sum(j in members | x[j]); def Root { var x[j in items]: Flow; eq e: total(x,items) == 0{mol/s}; } }",
    );
    let root = rows
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let mut workspace = CompilerWorkspace::new(input, WorkspaceLimits::default()).unwrap();
    workspace.publish_modeling(rows, names).unwrap();
    let admitted = admit(&mut workspace, root);
    assert!(admitted.inputs.is_empty());
    let cancel = Arc::new(AtomicBool::new(false));
    let artifact = fixture(
        &admitted,
        workspace.inputs.quantities.clone(),
        DerivativeOrder::Value,
        &cancel,
    )
    .unwrap();
    assert_eq!(
        artifact
            .worker()
            .evaluate(&[], DerivativeOrder::Value, &mut BTreeMap::new(), &cancel)
            .unwrap()
            .values,
        vec![0.]
    );
}

#[test]
fn kernel_table_function_references_dispatch_and_share_specializations() {
    let (mut workspace, _, _, root) = setup(
        r#"package p {
 entity kind item {} entity item a {} entity item b {} entity item c {}
 set items: Set<item> = {a,b,c};
 fn double(x: Scalar)->Scalar=x*2; fn triple(x: Scalar)->Scalar=x*3;
 fn apply(method: Fn(x: Scalar)->Scalar, x: Scalar)->Scalar=method(x);
 table methods[j: item]: Fn(x: Scalar)->Scalar;
 dataset choices: methods source "synthetic" { [a]=[double]; [b]=[triple]; [c]=[double]; }
 def Root { var x[j in items]: Scalar; eq e[j in items]: apply(methods[j],x[j]) == 0; }
 }"#,
    );
    let prepared = workspace
        .prepare_modeling_cancellable(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings::default(),
            Limits::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert_eq!(prepared.model.functions.len(), 4); // two selected methods and two apply specializations
    let body = prepared.admitted;
    let cancel = Arc::new(AtomicBool::new(false));
    let artifact = fixture(
        &body,
        workspace.inputs.quantities.clone(),
        DerivativeOrder::Value,
        &cancel,
    )
    .unwrap();
    let mut values = artifact
        .worker()
        .evaluate(
            &[2., 2., 2.],
            DerivativeOrder::Value,
            &mut BTreeMap::new(),
            &cancel,
        )
        .unwrap()
        .equations(&body);
    values.sort_by(f64::total_cmp);
    assert_eq!(values, vec![4., 4., 6.]);
}

#[test]
fn kernel_import_alias_resolves_functions_in_selected_package() {
    let (mut workspace, _, _, root) = setup(
        "package library { fn double(x: Scalar)->Scalar=x*2; } package p { use library @ \"1.0.0\" as m; def Root { var x: Scalar; eq e: m.double(x) == 4; } }",
    );
    assert_eq!(admit(&mut workspace, root).inputs.len(), 1);
}

#[test]
fn kernel_original_terms_survive_exact_cancellation_for_scaling() {
    let (mut workspace, _, _, root) = setup(
        "package p { def Root { var x: Scalar; eq balance: x-x == 0; annotation scale balance(inverseSum); } }",
    );
    let flag = Arc::new(AtomicBool::new(false));
    let prepared = workspace
        .prepare_modeling_cancellable(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings::default(),
            Limits::default(),
            flag.clone(),
        )
        .unwrap();
    let admitted = &prepared.admitted;
    let f = fixture(
        admitted,
        workspace.inputs.quantities.clone(),
        DerivativeOrder::Value,
        &flag,
    )
    .unwrap();
    let mut worker = f.assembly.worker(BTreeMap::new(), flag);
    let observations = worker
        .constraints(&CaseValues {
            scalars: BTreeMap::from([(admitted.inputs[0], 4.0)]),
        })
        .unwrap();
    assert_eq!(
        equation_values(admitted, admitted.ordered_values(&observations).unwrap()),
        vec![0.0]
    );
    let schemes = prepared
        .model
        .annotations
        .iter()
        .filter_map(|a| {
            if let pse_modeling::annotation::AnnotationValue::Scale(s) = a.value {
                Some((a.target, s))
            } else {
                None
            }
        })
        .collect();
    let scales = admitted.derived_scales(&schemes, &observations).unwrap();
    assert_eq!(scales.values().copied().collect::<Vec<_>>(), vec![0.125]);
}

#[test]
fn kernel_inline_implicit_block_has_explicit_capture_gathers() {
    let (mut workspace, _, _, root) = setup(
        "package p { def Root { var x: Scalar; implicit root { var y: Scalar; eq residual: y*y == x; } realize root_policy on root using inline; eq pin: root.y == 2; } }",
    );
    let prepared = workspace
        .prepare_modeling_cancellable(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings::default(),
            Limits::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert_eq!(prepared.model.instances.len(), 2);
    assert_eq!(prepared.structure.square.rows.len(), 2);
    assert_eq!(prepared.structure.square.columns.len(), 2);
    let inputs = prepared
        .admitted
        .inputs
        .iter()
        .map(|id| {
            if prepared.model.symbols[id].lineage.path.ends_with(".x") {
                4.0
            } else {
                2.0
            }
        })
        .collect::<Vec<_>>();
    let flag = Arc::new(AtomicBool::new(false));
    let f = fixture(
        &prepared.admitted,
        workspace.inputs.quantities.clone(),
        DerivativeOrder::Second,
        &flag,
    )
    .unwrap();
    assert!(
        f.worker()
            .evaluate(
                &inputs,
                DerivativeOrder::Second,
                &mut BTreeMap::new(),
                &flag
            )
            .unwrap()
            .equations(&prepared.admitted)
            .iter()
            .all(|v| v.abs() < 1e-14)
    );
}

#[test]
fn kernel_repeated_consumers_share_one_library_body_and_local_gathers() {
    let (mut workspace, _, _, root) = setup(
        "package p { entity kind item {} entity item a {} entity item b {} set items: Set<item> = {a,b}; def Root { var x[i in items]: Scalar; eq e[i in items]: x[i]*x[i] == 4; } }",
    );
    let admitted = admit(&mut workspace, root);
    let occurrences = admitted
        .outputs
        .iter()
        .filter(|output| matches!(output, ModelingOutput::Equation { .. }))
        .map(|o| {
            admitted
                .case
                .instances()
                .iter()
                .find(|i| i.instance == o.row_id())
                .unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(occurrences.len(), 2);
    assert_eq!(occurrences[0].body, occurrences[1].body);
    assert!(occurrences.iter().all(|i| i.slots.len() == 1));
    assert_ne!(
        occurrences[0].slots[0].source(),
        occurrences[1].slots[0].source()
    );
    let cancel = Arc::new(AtomicBool::new(false));
    let f = fixture(
        &admitted,
        workspace.inputs.quantities.clone(),
        DerivativeOrder::Second,
        &cancel,
    )
    .unwrap();
    assert_eq!(
        f.worker()
            .evaluate(
                &[2., 2.],
                DerivativeOrder::Second,
                &mut BTreeMap::new(),
                &cancel
            )
            .unwrap()
            .equations(&admitted),
        vec![0., 0.]
    );
}

#[test]
fn kernel_collocation_preserves_equations_continuity_and_quadrature() {
    for scheme in ["radau", "legendre"] {
        let text = format!(
            "package p {{ collocation radau alpha(1) beta(0) right(true); collocation legendre alpha(0) beta(0) right(false); def Root {{ domain t: Scalar from 0 to 2; discretize mesh on t using {scheme}(elements = 2, order = 3); var x[i in t]: Scalar; eq ode[i in t]: d(x[i])/di == 0; eq initial: x[0] == 3; let area: Scalar = integral(i in t | x[i]); eq output: area == 6; }} }}"
        );
        let (mut workspace, _, _, root) = setup(&text);
        let admitted = admit(&mut workspace, root);
        let cancel = Arc::new(AtomicBool::new(false));
        let f = fixture(
            &admitted,
            workspace.inputs.quantities.clone(),
            DerivativeOrder::Second,
            &cancel,
        )
        .unwrap();
        let values = f
            .worker()
            .evaluate(
                &vec![3.; admitted.inputs.len()],
                DerivativeOrder::Second,
                &mut BTreeMap::new(),
                &cancel,
            )
            .unwrap()
            .values;
        for (output, value) in admitted.outputs.iter().zip(values) {
            if matches!(output, ModelingOutput::Equation { .. }) {
                assert!(value.abs() < 1e-10, "{scheme}: {output:?} = {value}");
            }
        }
        assert_eq!(
            admitted
                .outputs
                .iter()
                .filter(|o| matches!(o, ModelingOutput::Equation { .. }))
                .count(),
            if scheme == "radau" { 8 } else { 10 }
        );
    }
}

// Exercise the production CasePlan owner; no whole-root evaluator survives in this fixture.
#[test]
fn kernel_function_validity_guards_survive_simplification_and_partial_derivatives() {
    for body in ["f(x)-f(x)", "partial(f,x)(x)"] {
        let text = format!(
            "package p {{ fn zero()->Scalar=0; fn upper()->Scalar=10; fn f(x:Scalar)->Scalar valid(x>zero() and log(x)<upper()) = x*x; def Root {{var x:Scalar; eq e:{body}==0;}} }}"
        );
        let (mut workspace, _, _, root) = setup(&text);
        let admitted = admit(&mut workspace, root);
        let cancel = Arc::new(AtomicBool::new(false));
        let f = fixture(
            &admitted,
            workspace.inputs.quantities.clone(),
            DerivativeOrder::Second,
            &cancel,
        )
        .unwrap();
        let mut worker = f.assembly.worker(BTreeMap::new(), cancel);
        let values = |x| CaseValues {
            scalars: BTreeMap::from([(admitted.inputs[0], x)]),
        };
        let expected = if body.starts_with("partial") { 4. } else { 0. };
        let all = admitted
            .ordered_values(&worker.constraints(&values(2.)).unwrap())
            .unwrap();
        let actual = admitted
            .outputs
            .iter()
            .zip(all)
            .find_map(|(o, v)| matches!(o, ModelingOutput::Equation { .. }).then_some(v))
            .unwrap();
        assert!(
            (actual - expected).abs() < 1e-12,
            "{body}: {actual} != {expected}"
        );
        worker.jacobian(&values(2.)).unwrap();
        worker
            .hessian(&values(2.), 0., &vec![1.; admitted.case.rows().len()])
            .unwrap();
        for x in [0., -1., 1e6] {
            assert!(worker.constraints(&values(x)).is_err());
        }
    }
}
#[test]
fn kernel_nested_hints_reject_self_dependencies_and_select_regime_overrides() {
    for expression in ["y", "y-y+1", "if y>0 then 1 else 2"] {
        let source = format!(
            "package p {{ def Root {{ implicit a {{var y:Scalar; eq e:y==2; annotation start y({expression});}} realize r on a using nested; eq e:a.y==2; }} }}"
        );
        let (mut workspace, _, _, root) = setup(&source);
        let error = workspace
            .admit_modeling(
                root,
                InstanceId::from_id(SemanticId::NIL),
                Bindings::default(),
                Limits::default(),
            )
            .expect_err("self-dependent start must be rejected");
        assert!(error.to_string().contains("own unknowns"), "{error}");
    }
    let source = "package p { def Root { var target:Scalar; implicit roots select minimum((y-target)*(y-target),1e-8) {var y:Scalar; annotation start y(y); regime negative {eq root:y==-1; annotation start y(-1);} regime positive {eq root:y==1; annotation start y(1);} } realize r on roots using nested; eq selected:roots.y==target; } }";
    let (mut workspace, _, _, root) = setup(source);
    let admitted = admit(&mut workspace, root);
    let inner = admitted.implicit.values().next().unwrap();
    assert_eq!(inner.residuals.len(), 2);
    assert!(inner.residuals.iter().all(|r| r.hint_targets.len() == 1));
    assert_eq!(admitted.implicit_order().unwrap().len(), 1);
}
#[test]
fn kernel_nested_implicit_provider_projects_values_and_ift_derivatives() {
    use pse_kernels::ProviderFactory;
    use pse_math::implicit::{InnerSolver, Options, Problem, Unknown};
    #[derive(Debug)]
    struct AnalyticRoot;
    impl InnerSolver for AnalyticRoot {
        fn identity(&self) -> ContentHash {
            pse_math::implicit::solver_identity("test.analytic-root.v1")
        }
        fn solve(
            &self,
            _problem: Arc<Problem>,
            inputs: &[f64],
            _options: &Options,
            _cancel: &Arc<AtomicBool>,
        ) -> std::result::Result<Vec<f64>, MathError> {
            Ok(vec![inputs[0].sqrt()])
        }
    }
    for policy in [
        "nested",
        "accelerated(cubic_roots)",
        "accelerated(test_polynomial)",
    ] {
        let (mut workspace, _, _, root) = setup(
        &"package p { def Root { var x: Scalar; implicit root { var y: Scalar; eq residual: y*y == x; } realize policy on root using nested; eq pin: root.y == 2; } }".replace("using nested",&format!("using {policy}")),
    );
        let admitted = admit(&mut workspace, root);
        assert_eq!(admitted.inputs.len(), 1);
        assert_eq!(admitted.implicit.len(), 1);
        let cancel = Arc::new(AtomicBool::new(false));
        let inner = admitted.implicit.values().next().unwrap();
        let mut accelerators = pse_math::implicit::accelerators::Accelerators::standard();
        accelerators
            .register(
                "test_polynomial".into(),
                Arc::new(pse_math::implicit::accelerators::Cubic),
            )
            .unwrap();
        let factory = inner
            .factory(
                BTreeMap::from([(
                    inner.residuals[0].id,
                    pse_math::implicit::Configuration::Fixed(
                        inner
                            .unknowns
                            .iter()
                            .map(|id| Unknown {
                                id: *id,
                                lower: 0.5,
                                upper: 2.5,
                            })
                            .collect(),
                        Options {
                            start: vec![1.0],
                            variable_nominals: vec![2.0],
                            variable_tolerance: vec![1e-9],
                            residual_tolerance: vec![1e-10],
                            iterations: 50,
                            time_limit: std::time::Duration::from_secs(2),
                            derivative_tolerance: 1e-10,
                        },
                    ),
                )]),
                Arc::new(AnalyticRoot),
                &accelerators,
                cancel.clone(),
                EvaluationLimits::default(),
            )
            .unwrap();
        let f = fixture(
            &admitted,
            workspace.inputs.quantities.clone(),
            DerivativeOrder::Second,
            &cancel,
        )
        .unwrap();
        let mut worker = f.assembly.worker(
            BTreeMap::from([(inner.descriptor.spec().key(), factory.create().unwrap())]),
            cancel.clone(),
        );
        let values = CaseValues {
            scalars: BTreeMap::from([(admitted.inputs[0], 4.0)]),
        };
        let rows = worker.constraints(&values).unwrap();
        let public = admitted.ordered_values(&rows).unwrap();
        for (meaning, value) in admitted.outputs.iter().zip(public) {
            match meaning {
                ModelingOutput::Equation { .. } => assert!(value.abs() < 1e-12),
                ModelingOutput::Member(id) => assert_eq!(
                    value,
                    if admitted.inputs.contains(id) {
                        4.0
                    } else {
                        2.0
                    }
                ),
                _ => {}
            }
        }
        let jacobian = worker.jacobian(&values).unwrap();
        assert!(jacobian.val().iter().any(|v| (*v - 0.25).abs() < 1e-12));
        worker
            .hessian(&values, 0.0, &vec![1.0; rows.len()])
            .unwrap();
        assert!(
            worker
                .constraints(&CaseValues {
                    scalars: BTreeMap::from([(admitted.inputs[0], 9.0)])
                })
                .is_err()
        );
        cancel.store(true, Ordering::Release);
        assert!(worker.constraints(&values).is_err());
    }
}

#[test]
fn kernel_elastic_variants_preserve_originals_and_normalize_penalties() {
    let (mut workspace, _, _, root) = setup(
        "package p { def Root { var x: Scalar; eq equality: x == 4; eq lower: x >= 3; eq upper: x <= 1; relax a on equality nominal 2; relax b on lower nominal 1; relax c on upper nominal 4; param t: Scalar = 0; continue ramp on t from 0 to 1; } }",
    );
    let prepared = workspace
        .prepare_modeling_cancellable(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings::default(),
            Limits::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    let a = &prepared.admitted;
    assert_eq!(prepared.model.elastic.len(), 3);
    assert_eq!(
        a.case
            .variables()
            .iter()
            .filter(|v| v.lower == Some(0.0))
            .count(),
        4
    );
    let cancel = Arc::new(AtomicBool::new(false));
    let f = fixture(
        a,
        workspace.inputs.quantities.clone(),
        DerivativeOrder::Second,
        &cancel,
    )
    .unwrap();
    let mut values = a
        .inputs
        .iter()
        .map(|id| {
            (
                *id,
                if prepared.model.symbols[id].lineage.path.ends_with(".x") {
                    2.0
                } else if let Some(pse_modeling::specialize::Value::Number { bits, .. }) =
                    prepared.model.symbols[id].initial
                {
                    f64::from_bits(bits)
                } else {
                    0.0
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    for elastic in prepared.model.elastic.values() {
        match elastic.original.lineage.path.rsplit('.').next().unwrap() {
            "equality" => {
                values.insert(elastic.slacks[1], 2.0);
            }
            "lower" => {
                values.insert(elastic.slacks[0], 1.0);
            }
            "upper" => {
                values.insert(elastic.slacks[0], 1.0);
            }
            other => panic!("unexpected elastic row {other}"),
        }
    }
    let mut worker = f.assembly.worker(BTreeMap::new(), cancel);
    let values = CaseValues { scalars: values };
    let rows = worker.constraints(&values).unwrap();
    let public = a.ordered_values(&rows).unwrap();
    for (output, value) in a.outputs.iter().zip(public) {
        if matches!(output, ModelingOutput::Equation { .. }) {
            assert!(value.abs() < 1e-12);
        }
        if matches!(output, ModelingOutput::OriginalEquation(_)) {
            assert!(value.abs() >= 1.0);
        }
    }
    assert!((worker.objective(&values).unwrap() - 2.25).abs() < 1e-12);
    let continuation = prepared.model.continuation.values().next().unwrap();
    assert!(
        matches!(continuation.at(0.5).unwrap(),pse_modeling::specialize::Value::Number{bits,..} if f64::from_bits(bits)==0.5)
    );
    assert!(continuation.at(1.1).is_err());
}

#[test]
fn kernel_stage_variants_preserve_variables_and_restore_final_structure() {
    let (mut workspace, _, _, root) = setup(
        "package p { def Root { var x: Scalar; eq e: x*x == 4; stage linear { override eq e: x == 2; } } }",
    );
    let normal = admit(&mut workspace, root);
    let bindings = Bindings {
        facts: BTreeMap::from([(
            "stage.linear".into(),
            pse_modeling::specialize::Value::Boolean(true),
        )]),
        ..Bindings::default()
    };
    let stage = workspace
        .admit_modeling(
            root,
            InstanceId::from_id(SemanticId::NIL),
            bindings,
            Limits::default(),
        )
        .unwrap();
    assert_eq!(normal.inputs, stage.inputs);
    assert_ne!(normal.case.key(), stage.case.key());
    assert_eq!(normal, admit(&mut workspace, root));
    let cancel = Arc::new(AtomicBool::new(false));
    for (a, expected) in [(&normal, 5.0), (&stage, 1.0)] {
        let f = fixture(
            a,
            workspace.inputs.quantities.clone(),
            DerivativeOrder::Second,
            &cancel,
        )
        .unwrap();
        assert_eq!(
            f.worker()
                .evaluate(
                    &[3.0],
                    DerivativeOrder::Second,
                    &mut BTreeMap::new(),
                    &cancel
                )
                .unwrap()
                .equations(a),
            vec![expected]
        );
    }
}

#[test]
fn kernel_piecewise_proves_boundary_jets_and_keeps_lazy_regions() {
    let (mut workspace, _, _, root) = setup(
        "package p { fn f(x: Scalar, a: Scalar) -> Scalar piecewise 2 = if x < 0 then 0 else a*x*x*x; def Root { var x: Scalar; param a: Scalar = 2; eq e: f(x,a) == 0; } }",
    );
    let a = admit(&mut workspace, root);
    let cancel = Arc::new(AtomicBool::new(false));
    let f = fixture(
        &a,
        workspace.inputs.quantities.clone(),
        DerivativeOrder::Second,
        &cancel,
    )
    .unwrap();
    for x in [-1.0, 0.0, 2.0] {
        let prepared = workspace
            .prepare_modeling_cancellable(
                root,
                InstanceId::from_id(SemanticId::NIL),
                Bindings::default(),
                Limits::default(),
                cancel.clone(),
            )
            .unwrap();
        let values = a
            .inputs
            .iter()
            .map(|id| {
                if prepared.model.symbols[id].lineage.path.ends_with(".x") {
                    x
                } else {
                    2.0
                }
            })
            .collect::<Vec<_>>();
        let values = f
            .worker()
            .evaluate(
                &values,
                DerivativeOrder::Second,
                &mut BTreeMap::new(),
                &cancel,
            )
            .unwrap()
            .equations(&a);
        assert_eq!(values, vec![if x > 0.0 { 2.0 * x * x * x } else { 0.0 }]);
    }
    for body in ["if x < 0 then 0 else x", "if x < 0 then 0 else x*x"] {
        let (mut workspace, _, _, root) = setup(&format!(
            "package p {{ fn f(x: Scalar) -> Scalar piecewise 2 = {body}; def Root {{ var x: Scalar; eq e: f(x) == 0; }} }}"
        ));
        assert!(
            workspace
                .admit_modeling(
                    root,
                    InstanceId::from_id(SemanticId::NIL),
                    Bindings::default(),
                    Limits::default()
                )
                .is_err()
        );
    }
}

#[test]
fn kernel_external_vector_shapes_derivatives_and_revisions_are_checked() {
    use pse_kernels::*;
    #[derive(Debug)]
    struct Vector(ProviderSpec);
    impl Provider for Vector {
        fn spec(&self) -> &ProviderSpec {
            &self.0
        }
        fn evaluate(
            &mut self,
            x: &[f64],
            r: &ProviderRequest,
            c: &EvaluationContext<'_>,
        ) -> std::result::Result<ProviderValues, ProviderError> {
            r.validate(&self.0, c)?;
            if x.len() != 2 {
                return Err(ProviderError::Contract("vector input extent".into()));
            }
            let mut out = ProviderValues {
                values: vec![],
                jacobian: vec![],
                hessians: vec![],
            };
            for i in &r.outputs {
                out.values.push(if *i == 0 {
                    x[0] * x[0] + x[1]
                } else {
                    x[0] * x[1]
                });
                if r.order >= DerivativeOrder::First {
                    out.jacobian.extend(if *i == 0 {
                        [2.0 * x[0], 1.0]
                    } else {
                        [x[1], x[0]]
                    });
                }
                if r.order >= DerivativeOrder::Second {
                    out.hessians.extend(if *i == 0 {
                        [2.0, 0.0, 0.0, 0.0]
                    } else {
                        [0.0, 1.0, 1.0, 0.0]
                    });
                }
            }
            out.validate(&self.0, r)?;
            Ok(out)
        }
    }
    let revision = ContentHash::from_bytes([11; 32]);
    let data = ContentHash::from_bytes([12; 32]);
    let text = format!(
        "package p {{ entity kind item {{}} entity item a {{}} entity item b {{}} set items: Set<item> = {{a,b}}; fn vector(x: Scalar[item], output: Integer) -> Scalar external \"vector\" revision \"{revision}\" data \"{data}\" output output derivatives 2 source analytic smoothness 2; fn composed(x: Scalar[item], output: Integer) -> Scalar = vector(x,output)*vector(x,output); def Root {{ var x[j in items]: Scalar; eq a: vector(x,0) == 0; eq b: vector(x,1) == 0; }} }}"
    );
    let (mut w, rows, _, root) = setup(&text);
    let model = w
        .specialize_modeling(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings::default(),
            Limits::default(),
        )
        .unwrap();
    let contract = model
        .functions
        .values()
        .find_map(|f| f.external.as_ref())
        .unwrap();
    assert_eq!(contract.shapes.len(), 1);
    let shape = &contract.shapes[0];
    let mut expected = rows
        .iter()
        .filter(|r| r.value.entity.is_some())
        .map(|r| vec![r.declaration_id.as_id()])
        .collect::<Vec<_>>();
    expected.sort();
    assert_eq!(shape.coordinates, expected);
    let q = w.inputs.quantities.neutral_dimensionless().unwrap();
    let unit = w.inputs.quantities.quantity_type(q).unwrap().canonical_unit;
    let port = |v| Port {
        id: SemanticId::from_bytes([v; 16]),
        quantity: q,
        unit,
    };
    let spec = ProviderSpec {
        shapes: ProviderShapes {
            inputs: vec![ProviderShape {
                id: shape.argument,
                axes: shape.axes.clone(),
                coordinates: shape.coordinates.clone(),
                cells: vec![port(60).id, port(61).id],
            }],
            outputs: vec![ProviderShape {
                id: SemanticId::from_bytes([63; 16]),
                axes: shape.axes.clone(),
                coordinates: shape.coordinates.clone(),
                cells: vec![port(62).id, port(63).id],
            }],
        },
        derivative_source: DerivativeSource::Analytic,

        id: SemanticId::from_bytes([64; 16]),
        revision,
        data,

        inputs: vec![port(60), port(61)],
        outputs: vec![port(62), port(63)],
        derivatives: DerivativeOrder::Second,
        smoothness: DerivativeOrder::Second,
    };
    let mut input = w.inputs.clone();
    input.providers.insert(
        "vector".into(),
        ProviderCall {
            descriptor: AdmittedProvider::new(spec.clone(), &input.quantities).unwrap(),
            output: 0,
        },
    );
    w.publish(input).unwrap();
    let a = admit(&mut w, root);
    let cancel = Arc::new(AtomicBool::new(false));
    let f = fixture(
        &a,
        w.inputs.quantities.clone(),
        DerivativeOrder::Second,
        &cancel,
    )
    .unwrap();
    let provider: Box<dyn Provider> = Box::new(Vector(spec.clone()));
    let mut worker = f
        .assembly
        .worker(BTreeMap::from([(spec.key(), provider)]), cancel);
    let point = CaseValues {
        scalars: a.inputs.iter().map(|id| (*id, 2.0)).collect(),
    };
    let mut values = a
        .ordered_values(&worker.constraints(&point).unwrap())
        .unwrap();
    values = equation_values(&a, values);
    values.sort_by(f64::total_cmp);
    assert_eq!(values, vec![4.0, 6.0]);
    worker.jacobian(&point).unwrap();
    worker
        .hessian(&point, 0.0, &vec![1.0; a.case.rows().len()])
        .unwrap();
    for (expression, order, expected) in [
        ("partial(composed,x[p.a])(x,1)", DerivativeOrder::First, 16.),
        (
            "partial(composed,x[p.a],x[p.b])(x,1)",
            DerivativeOrder::Value,
            16.,
        ),
        ("partial(vector,x[p.a])(x,1)", DerivativeOrder::First, 2.),
        (
            "partial(vector,x[p.a],x[p.b])(x,1)",
            DerivativeOrder::Value,
            1.,
        ),
    ] {
        let mut rows = rows.clone();
        for row in &mut rows {
            if let Some(e) = &mut row.value.equation {
                e.expression = format!("{expression} == 0");
            }
        }
        let names = BTreeMap::from([("Scalar".into(), q)]);
        w.publish_modeling(rows, names).unwrap();
        let a = admit(&mut w, root);
        let cancel = Arc::new(AtomicBool::new(false));
        let f = fixture(&a, w.inputs.quantities.clone(), order, &cancel).unwrap();
        let provider: Box<dyn Provider> = Box::new(Vector(spec.clone()));
        let mut worker = f
            .assembly
            .worker(BTreeMap::from([(spec.key(), provider)]), cancel.clone());
        let point = CaseValues {
            scalars: a.inputs.iter().map(|id| (*id, 2.)).collect(),
        };
        let values = a
            .ordered_values(&worker.constraints(&point).unwrap())
            .unwrap();
        let values = equation_values(&a, values);
        assert_eq!(values.len(), 2);
        assert!(values.iter().all(|v| (*v - expected).abs() < 1e-12));
        if order == DerivativeOrder::First {
            let j = worker.jacobian(&point).unwrap();
            let expected = if expression.contains("composed") {
                8.
            } else {
                1.
            };
            assert!(j.val().iter().any(|v| (*v - expected).abs() < 1e-12));
        }
        assert!(
            fixture(
                &a,
                w.inputs.quantities.clone(),
                DerivativeOrder::Second,
                &cancel
            )
            .is_err()
        );
    }
    let mut unsupported = rows.clone();
    for row in &mut unsupported {
        if let Some(e) = &mut row.value.equation {
            e.expression = "partial(composed,x[p.a],x[p.b],x[p.b])(x,1) == 0".into();
        }
    }
    w.publish_modeling(unsupported, BTreeMap::from([("Scalar".into(), q)]))
        .unwrap();
    assert!(
        w.admit_modeling(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings::default(),
            Limits::default()
        )
        .is_err()
    );
    w.publish_modeling(rows, BTreeMap::from([("Scalar".into(), q)]))
        .unwrap();
    let mut changed = spec;
    changed.data = revision;
    let mut input = w.inputs.clone();
    input.providers.insert(
        "vector".into(),
        ProviderCall {
            descriptor: AdmittedProvider::new(changed, &input.quantities).unwrap(),
            output: 0,
        },
    );
    w.publish(input).unwrap();
    assert!(
        w.admit_modeling(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings::default(),
            Limits::default()
        )
        .is_err()
    );
}

#[test]
fn kernel_case_projection_excludes_observations_and_preserves_specification() {
    let (mut w, _, _, root) = setup(
        "package p { def Root { var x: Scalar; var y: Scalar; eq e: x+y == 3; let bad: Scalar = log(-1); } }",
    );
    let case = ModelingCaseBindings {
        values: BTreeMap::from([("x".into(), 1.0), ("y".into(), 2.0)]),
        variables: BTreeMap::from([(
            "y".into(),
            ModelingVariableState {
                fixed: Some(true),
                ..Default::default()
            },
        )]),
    };
    let bindings = Bindings {
        demand: vec!["bad".into()],
        ..Bindings::default()
    };
    let (model, solve, values) = w
        .prepare_modeling_case_cancellable(
            root,
            InstanceId::from_id(SemanticId::NIL),
            bindings,
            Limits::default(),
            &case,
            DerivativeOrder::Second,
            Profile::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert!(model.admitted.case.rows().len() > 1);
    assert_eq!(solve.plan.structure().rows().len(), 1);
    assert_eq!(solve.facts.variables, 1);
    assert!(solve.facts.equalities && solve.facts.coefficients);
    assert!(!model.admitted.case.variables().iter().any(|v| v.fixed));
    let assembly = Arc::new(
        solve
            .plan
            .compile(
                Optimization::default(),
                EvaluationLimits::default(),
                &Arc::new(AtomicBool::new(false)),
            )
            .unwrap(),
    );
    assert_eq!(
        assembly
            .worker(BTreeMap::new(), Arc::new(AtomicBool::new(false)))
            .constraints(&values)
            .unwrap(),
        vec![0.0]
    );
    let mut invalid = case;
    invalid.variables.get_mut("y").unwrap().lower = Some(Some(4.0));
    assert!(
        w.prepare_modeling_case_cancellable(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings::default(),
            Limits::default(),
            &invalid,
            DerivativeOrder::Second,
            Profile::default(),
            Arc::new(AtomicBool::new(false))
        )
        .is_err()
    );
}

#[test]
fn value_rebind_shares_structure_and_rebuilds_only_consumed_values() {
    let (mut w, _, _, root) =
        setup("package p { def Root { param p: Scalar = 2; var x: Scalar; eq e: x*p == 6; } }");
    let cancel = Arc::new(AtomicBool::new(false));
    let bindings = Bindings {
        demand: vec!["p".into(), "x".into()],
        ..Bindings::default()
    };
    let model = w
        .prepare_modeling_cancellable(
            root,
            InstanceId::from_id(SemanticId::NIL),
            bindings,
            Limits::default(),
            cancel.clone(),
        )
        .unwrap();
    let (x, p) = (model.model.paths["x"], model.model.paths["p"]);
    let values = |x_value: f64, p_value: f64| CaseValues {
        scalars: BTreeMap::from([(x, x_value), (p, p_value)]),
    };
    let structure = model.bound_structure(&BTreeMap::new()).unwrap();
    let context = ContentHash::from_bytes([7; 32]);
    let key = model.view_key(
        &structure,
        DerivativeOrder::First,
        Profile::default(),
        &context,
    );
    let first = w
        .prepare_modeling_view(
            &model,
            structure.clone(),
            &values(1.0, 2.0),
            DerivativeOrder::First,
            Profile::default(),
            &cancel,
        )
        .unwrap();
    // A changed free start is not among the consumed values: everything is shared.
    assert!(first.values_match(&values(5.0, 2.0)));
    let start = first.rebind(&values(5.0, 2.0), &cancel).unwrap();
    assert!(Arc::ptr_eq(&start.presolve, &first.presolve));
    assert!(Arc::ptr_eq(&start.plan, &first.plan));
    // A changed parameter rebuilds only the value-dependent products.
    assert!(!first.values_match(&values(1.0, 3.0)));
    let rebound = first.rebind(&values(1.0, 3.0), &cancel).unwrap();
    assert!(Arc::ptr_eq(&rebound.plan, &first.plan));
    assert!(Arc::ptr_eq(&rebound.structure, &first.structure));
    assert!(Arc::ptr_eq(&rebound.artifacts, &first.artifacts));
    assert!(rebound.values_match(&values(1.0, 3.0)));
    let fresh = w
        .prepare_modeling_view(
            &model,
            structure,
            &values(1.0, 3.0),
            DerivativeOrder::First,
            Profile::default(),
            &cancel,
        )
        .unwrap();
    assert_eq!(rebound.coefficient_values, fresh.coefficient_values);
    assert_eq!(
        rebound.coefficients.as_ref().map(|c| c.assumptions),
        fresh.coefficients.as_ref().map(|c| c.assumptions)
    );
    assert_eq!(rebound.presolve.values, fresh.presolve.values);
    assert_eq!(rebound.facts, fresh.facts);
    // Structure, not values, keys the view.
    let fixed = model
        .bound_structure(&BTreeMap::from([(
            x,
            ModelingVariableState {
                fixed: Some(true),
                ..Default::default()
            },
        )]))
        .unwrap();
    assert_eq!(
        key,
        model.view_key(
            &model.bound_structure(&BTreeMap::new()).unwrap(),
            DerivativeOrder::First,
            Profile::default(),
            &context
        )
    );
    assert_ne!(
        key,
        model.view_key(&fixed, DerivativeOrder::First, Profile::default(), &context)
    );
    // Values that do not bind the structure are refused.
    assert!(
        first
            .rebind(
                &CaseValues {
                    scalars: BTreeMap::from([(x, 1.0)]),
                },
                &cancel,
            )
            .is_err()
    );
}

struct Fixture {
    admitted: AdmittedModeling,
    assembly: Arc<pse_math::assembly::CaseAssembly>,
    cancel: Arc<AtomicBool>,
}
struct FixtureWorker<'a> {
    fixture: &'a Fixture,
    worker: pse_math::assembly::CaseWorker,
}
struct FixtureResult {
    values: Vec<f64>,
}
impl FixtureResult {
    fn equations(self, admitted: &AdmittedModeling) -> Vec<f64> {
        equation_values(admitted, self.values)
    }
}
fn equation_values(admitted: &AdmittedModeling, values: Vec<f64>) -> Vec<f64> {
    assert_eq!(admitted.outputs.len(), values.len());
    admitted
        .outputs
        .iter()
        .zip(values)
        .filter_map(|(output, value)| {
            matches!(output, ModelingOutput::Equation { .. }).then_some(value)
        })
        .collect()
}
fn fixture(
    admitted: &AdmittedModeling,
    registry: Arc<QuantityRegistry>,
    order: DerivativeOrder,
    cancel: &Arc<AtomicBool>,
) -> Result<Fixture> {
    let plan = admitted.plan(&registry, order, AssemblyLimits::default(), cancel)?;
    let assembly =
        Arc::new(plan.compile(Optimization::default(), EvaluationLimits::default(), cancel)?);
    Ok(Fixture {
        admitted: admitted.clone(),
        assembly,
        cancel: cancel.clone(),
    })
}
impl Fixture {
    fn worker(&self) -> FixtureWorker<'_> {
        FixtureWorker {
            fixture: self,
            worker: self.assembly.worker(BTreeMap::new(), self.cancel.clone()),
        }
    }
}
impl FixtureWorker<'_> {
    fn evaluate(
        &mut self,
        values: &[f64],
        order: DerivativeOrder,
        _providers: &mut BTreeMap<pse_kernels::ProviderKey, Box<dyn pse_kernels::Provider>>,
        _cancel: &Arc<AtomicBool>,
    ) -> Result<FixtureResult> {
        let case = CaseValues {
            scalars: self
                .fixture
                .admitted
                .inputs
                .iter()
                .copied()
                .zip(values.iter().copied())
                .collect(),
        };
        let values = self.worker.constraints(&case)?;
        if order >= DerivativeOrder::First {
            self.worker.jacobian(&case)?;
        }
        if order >= DerivativeOrder::Second {
            self.worker.hessian(&case, 0.0, &vec![1.0; values.len()])?;
        }
        Ok(FixtureResult {
            values: self.fixture.admitted.ordered_values(&values)?,
        })
    }
}

#[test]
fn kernel_validity_obligations_survive_cancellation_and_cover_derived_members() {
    for expression in ["x-x", "z-z"] {
        let text = format!(
            "package p {{ def Root {{ var x: Scalar; let z: Scalar = 2*x; eq e: {expression} == 0; annotation valid x(1,3,reject); annotation valid z(2,6,reject); }} }}"
        );
        let (mut workspace, _, _, root) = setup(&text);
        let admitted = admit(&mut workspace, root);
        let cancel = Arc::new(AtomicBool::new(false));
        let artifact = fixture(
            &admitted,
            workspace.inputs.quantities.clone(),
            DerivativeOrder::Second,
            &cancel,
        )
        .unwrap();
        let mut worker = artifact.worker();
        assert!(
            worker
                .evaluate(
                    &[0.],
                    DerivativeOrder::Second,
                    &mut BTreeMap::new(),
                    &cancel
                )
                .is_err()
        );
        assert!(
            worker
                .evaluate(
                    &[2.],
                    DerivativeOrder::Second,
                    &mut BTreeMap::new(),
                    &cancel
                )
                .is_ok()
        );
        assert!(
            worker
                .evaluate(
                    &[4.],
                    DerivativeOrder::Second,
                    &mut BTreeMap::new(),
                    &cancel
                )
                .is_err()
        );
    }
}

#[test]
fn kernel_pure_authored_tests_use_existing_math_without_runtime_or_solver() {
    let (mut workspace, rows, _, _) = setup(
        "package p { fn square(x:Scalar)->Scalar=x*x; def Root {} test squared { expect square(2) == 4 tolerance 1e-12; expect partial(square,x)(3) == 6 tolerance 1e-12; expect square(3) == 8 tolerance 1e-12; } }",
    );
    let test = rows
        .iter()
        .find(|r| r.name == "squared")
        .unwrap()
        .declaration_id;
    let results = workspace
        .check_modeling_expectations(
            test,
            root_instance(test),
            Bindings::default(),
            Limits::default(),
            &CaseValues {
                scalars: BTreeMap::new(),
            },
            Profile::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(results.iter().filter(|r| r.passed).count(), 2);
    assert!(
        results
            .iter()
            .any(|r| !r.passed && r.actual == 9. && r.expected == 8.)
    );
}

#[test]
fn kernel_expectations_combine_physical_and_relative_tolerances() {
    let source_text = "package p {def Root {} test values {expect 1001==1000 tolerance 0.25 relative 0.001; expect 1.2==1 tolerance 0.01 relative 0.001; expect 0==0 tolerance 0 relative 0.001;}}";
    let (mut workspace, rows, names, _) = setup(source_text);
    let rendered = pse_authoring::language::render(&rows).unwrap();
    assert_eq!(
        rows.iter().map(|r| &r.value).collect::<Vec<_>>(),
        source(&rendered)
            .iter()
            .map(|r| &r.value)
            .collect::<Vec<_>>()
    );
    let test = rows
        .iter()
        .find(|r| r.name == "values")
        .unwrap()
        .declaration_id;
    let run = |workspace: &mut CompilerWorkspace| {
        workspace.check_modeling_expectations(
            test,
            root_instance(test),
            Bindings::default(),
            Limits::default(),
            &CaseValues {
                scalars: BTreeMap::new(),
            },
            Profile::default(),
            Arc::new(AtomicBool::new(false)),
        )
    };
    let results = run(&mut workspace).unwrap();
    assert_eq!(results.iter().filter(|r| r.passed).count(), 2);
    let scaled = results.iter().find(|r| r.expected == 1000.).unwrap();
    assert_eq!(scaled.absolute_tolerance, 0.25);
    assert_eq!(scaled.relative_tolerance, 0.001);
    assert_eq!(scaled.tolerance, 1.25);
    for (absolute, relative) in [("-1", "0.1"), ("0", "-1"), ("0", "0")] {
        let mut invalid = rows.clone();
        for row in &mut invalid {
            if let Some(expected) = &mut row.value.expectation {
                expected.tolerance = absolute.into();
                expected.relative_tolerance = Some(relative.into());
            }
        }
        workspace.publish_modeling(invalid, names.clone()).unwrap();
        assert!(run(&mut workspace).is_err());
    }
    assert!(
        workspace
            .publish_modeling(
                source(&source_text.replace("relative 0.001", "relative 1{s}")),
                names
            )
            .is_err()
    );
}

#[test]
fn kernel_pure_expectations_apply_authored_fixture_values_without_solving() {
    let (mut workspace, rows, _, _) = setup(
        "package p { def Root { var x:Scalar; } test local fixture { dof 1; value root.x=3; } { child root:Root=Root(); expect root.x*root.x==9 tolerance 1e-12; } }",
    );
    let test = rows
        .iter()
        .find(|r| r.name == "local")
        .unwrap()
        .declaration_id;
    let instance = InstanceId::from_bytes([66; 16]);
    let model = workspace
        .prepare_modeling_cancellable(
            test,
            instance,
            Bindings::default(),
            Limits::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    let x = model.model.paths["root.x"];
    let run = |workspace: &mut CompilerWorkspace, scalars, cancelled| {
        workspace.check_modeling_expectations(
            test,
            instance,
            Bindings::default(),
            Limits::default(),
            &CaseValues { scalars },
            Profile::default(),
            Arc::new(AtomicBool::new(cancelled)),
        )
    };
    let result = run(&mut workspace, BTreeMap::new(), false).unwrap();
    assert_eq!(result.len(), 1);
    assert!(result[0].passed);
    let failed = run(&mut workspace, BTreeMap::from([(x, 4.)]), false).unwrap();
    assert!(!failed[0].passed);
    assert_eq!(failed[0].actual, 16.);
    assert!(
        run(
            &mut workspace,
            BTreeMap::from([(SemanticId::NIL, 3.)]),
            false
        )
        .is_err()
    );
    assert!(run(&mut workspace, BTreeMap::new(), true).is_err());
    assert!(run(&mut workspace, BTreeMap::new(), false).unwrap()[0].passed);
}

#[test]
fn kernel_piecewise_chained_breakpoints_and_explicit_partials_keep_proved_order() {
    let source = "package p { fn f(x: Scalar) -> Scalar piecewise 2 = if x < 0 then 0 else if x < 1 then x*x*x else x*x*x+(x-1)*(x-1)*(x-1); def Root { var x: Scalar; eq e: partial(f,x)(x) == 0; } }";
    let (mut workspace, _, _, root) = setup(source);
    let admitted = admit(&mut workspace, root);
    let cancel = Arc::new(AtomicBool::new(false));
    let compiled = fixture(
        &admitted,
        workspace.inputs.quantities.clone(),
        DerivativeOrder::First,
        &cancel,
    )
    .unwrap();
    for x in [-1.0, 0.0, 0.5, 1.0, 2.0] {
        let jet = compiled
            .worker()
            .evaluate(&[x], DerivativeOrder::First, &mut BTreeMap::new(), &cancel)
            .unwrap();
        let expected = if x < 0. {
            0.
        } else {
            3. * x * x + if x > 1. { 3. * (x - 1.) * (x - 1.) } else { 0. }
        };
        assert!((jet.values[0] - expected).abs() < 1e-12);
        let expected = if x < 0. {
            0.
        } else {
            6. * x + if x > 1. { 6. * (x - 1.) } else { 0. }
        };
        let mut worker = compiled.assembly.worker(BTreeMap::new(), cancel.clone());
        let jacobian = worker
            .jacobian(&CaseValues {
                scalars: BTreeMap::from([(admitted.inputs[0], x)]),
            })
            .unwrap();
        let row = compiled
            .assembly
            .structure()
            .rows()
            .iter()
            .position(|r| r.id == admitted.outputs[0].row_id())
            .unwrap();
        assert!((jacobian.get(row, 0).copied().unwrap_or(0.) - expected).abs() < 1e-12);
    }
    assert!(
        fixture(
            &admitted,
            workspace.inputs.quantities.clone(),
            DerivativeOrder::Second,
            &cancel
        )
        .is_err()
    );
    let invalid = source.replace("x*x*x+(x-1)*(x-1)*(x-1)", "x*x*x+(x-1)*(x-1)");
    let (mut workspace, _, _, root) = setup(&invalid);
    assert!(
        workspace
            .admit_modeling(
                root,
                InstanceId::from_id(SemanticId::NIL),
                Bindings::default(),
                Limits::default()
            )
            .is_err()
    );
}

#[test]
fn kernel_authored_math_replaces_composite_native_functions() {
    let package = include_str!("../../../../../packages/reference/physical/models/math.pse");
    let kinds = include_str!("../../../../../packages/reference/physical/models/kinds.pse");
    for expectation in [
        "math.log10(100) == 2",
        "math.tan(0) == 0",
        "partial(math.sinh,x)(0) == 1",
        "partial(math.sigmoid,x)(0) == 0.25",
        "math.sigmoid(1000) == 1",
        "math.sigmoid(-1000) == 0",
        "math.softplus(1000) == 1000",
    ] {
        let source = format!(
            "{kinds} {package} package p {{ use math @ \"1.0.0\"; def Root {{}} test pure {{ expect {expectation} tolerance 1e-12; }} }}"
        );
        let (mut workspace, rows, _, _) = setup(&source);
        let root = rows
            .iter()
            .find(|r| r.name == "pure")
            .unwrap()
            .declaration_id;
        let result = workspace
            .check_modeling_expectations(
                root,
                root_instance(root),
                Bindings::default(),
                Limits::default(),
                &CaseValues {
                    scalars: BTreeMap::new(),
                },
                Profile::default(),
                Arc::new(AtomicBool::new(false)),
            )
            .unwrap_or_else(|e| panic!("{expectation}: {e}"));
        assert_eq!(result.len(), 1);
        assert!(result.iter().all(|r| r.passed), "{expectation}: {result:?}");
    }
}

#[test]
fn kernel_integrated_axis_retains_symbolic_time_and_original_derivative_lineage() {
    let mut input = super::super::tests::inputs();
    input.preconditions = Arc::new(
        PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions()).unwrap(),
    );
    let time =
        QuantityTypeId::from_id(SemanticId::parse_hex("e2ccf6d0a394403db967f4f35b83cb7c").unwrap());
    let names = BTreeMap::from([
        (
            "Scalar".into(),
            input.quantities.neutral_dimensionless().unwrap(),
        ),
        ("Time".into(), time),
    ]);
    let rows = source(
        "package p { def Cell { var x:Time; } def Root { domain t: Time from 0{s} to 2{s}; discretize mesh on t using integrated(elements=1,order=1); child cell[i in t]:Cell=Cell(); eq ode[i in t]: d(cell[i].x)/di == 1; eq initial: cell[0{s}].x == 0{s}; eq clock[i in t]: cell[i].x == i; annotation start cell[0{s}].x(0{s}); } }",
    );
    let ode = rows
        .iter()
        .find(|r| r.name == "ode")
        .unwrap()
        .declaration_id;
    let root = rows
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let mut workspace = CompilerWorkspace::new(input, WorkspaceLimits::default()).unwrap();
    workspace.publish_modeling(rows, names).unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let model = workspace
        .prepare_modeling_cancellable(
            root,
            root_instance(root),
            Bindings::default(),
            Limits::default(),
            cancel.clone(),
        )
        .unwrap();
    assert_eq!(model.model.integrated.len(), 1);
    assert_eq!(model.model.derivatives.len(), 1);
    assert_eq!(model.model.initial_equations.len(), 1);
    let axis = model.model.integrated.values().next().unwrap();
    let derivative = model.model.derivatives.values().next().unwrap();
    assert_eq!(axis.upper, 2.);
    assert_eq!(derivative.axis, axis.id);
    assert_ne!(derivative.rate, derivative.state);
    let rates = model
        .admitted
        .implicit
        .values()
        .find(|i| i.algorithm == ImplicitAlgorithm::AffineRates)
        .unwrap();
    assert_eq!(rates.unknowns, [derivative.rate]);
    assert!(!model.admitted.inputs.contains(&derivative.rate));
    let equations = model
        .admitted
        .outputs
        .iter()
        .find_map(|o| match o {
            ModelingOutput::DynamicRate { state, equations } if *state == derivative.state => {
                Some(equations)
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(equations, &rates.residuals[0].rows);
    assert_eq!(equations.len(), 1);
    assert!(
        model
            .model
            .equations
            .iter()
            .any(|e| e.id == equations[0] && e.lineage.declaration == ode)
    );
    assert!(
        matches!(model.model.symbols[&axis.time].initial,Some(pse_modeling::specialize::Value::Number{bits,..}) if f64::from_bits(bits)==0.)
    );
}

#[test]
fn kernel_analysis_facts_are_typed_and_select_source_guards() {
    use pse_modeling::{analysis::Route, specialize::Value};
    let text = "package p { def Root { var x: Scalar; when analysis.dynamic { eq transient: x == 2; } when not analysis.dynamic { eq stationary: x == 1; } when analysis.route == analysis.simultaneous { let marker: Scalar = 3; } } }";
    let (mut workspace, _, _, root) = setup(text);
    for (route, name) in [
        (Route::Steady, "stationary"),
        (Route::Integrated, "transient"),
        (Route::Simultaneous, "transient"),
    ] {
        let model = workspace
            .prepare_modeling_cancellable(
                root,
                root_instance(root),
                Bindings::default().with_analysis(route),
                Limits::default(),
                Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
        assert_eq!(model.model.equations.len(), 1);
        assert!(model.model.equations[0].lineage.path.ends_with(name));
    }
    let mut invalid = Bindings::default().with_analysis(Route::Steady);
    invalid
        .facts
        .insert("analysis.dynamic".into(), Value::Boolean(true));
    assert!(
        workspace
            .specialize_modeling(root, root_instance(root), invalid, Limits::default())
            .unwrap_err()
            .to_string()
            .contains("disagrees")
    );
}

#[test]
fn kernel_regime_derivatives_are_regular_local_branch_jets() {
    use pse_kernels::{EvaluationContext, ProviderFactory, ProviderRequest};
    use pse_math::implicit::{Configuration, InnerSolver, Options, Problem, Unknown};
    #[derive(Debug)]
    struct Roots;
    impl InnerSolver for Roots {
        fn identity(&self) -> ContentHash {
            pse_math::implicit::solver_identity("test.polynomial-roots.v1")
        }
        fn solve(
            &self,
            _: Arc<Problem>,
            inputs: &[f64],
            options: &Options,
            _: &Arc<AtomicBool>,
        ) -> std::result::Result<Vec<f64>, MathError> {
            Ok(vec![options.start[0].signum() * inputs[0].sqrt()])
        }
    }
    let source = "package p { def Root { var x:Scalar; implicit roots select minimum(y,1e-8) { var y:Scalar; regime negative eligible(y<0) {eq root:y*y==x;} regime positive eligible(y>0) {eq root:y*y==x;} } realize policy on roots using nested; eq pin:roots.y==-2; } }";
    let (mut workspace, _, _, root) = setup(source);
    let admitted = admit(&mut workspace, root);
    let inner = admitted.implicit.values().next().unwrap();
    assert_eq!(inner.descriptor.spec().derivatives, DerivativeOrder::Second);
    let cancel = Arc::new(AtomicBool::new(false));
    let configs = inner
        .residuals
        .iter()
        .enumerate()
        .map(|(index, r)| {
            (
                r.id,
                Configuration::Fixed(
                    inner
                        .unknowns
                        .iter()
                        .map(|id| Unknown {
                            id: *id,
                            lower: -3.0,
                            upper: 3.0,
                        })
                        .collect(),
                    Options {
                        start: vec![if index == 0 { -1.0 } else { 1.0 }],
                        variable_nominals: vec![1.0],
                        variable_tolerance: vec![1e-9],
                        residual_tolerance: vec![1e-9],
                        iterations: 10,
                        time_limit: std::time::Duration::from_secs(2),
                        derivative_tolerance: 1e-10,
                    },
                ),
            )
        })
        .collect();
    // Residual order is semantic-ID order; align starts with each authored eligibility.
    let mut configs: BTreeMap<_, _> = configs;
    for residual in &inner.residuals {
        let mut eligibility = residual
            .assessment
            .as_ref()
            .unwrap()
            .eligibility
            .math
            .compile(
                &[0],
                &[0, 1],
                DerivativeOrder::Value,
                Optimization::default(),
                EvaluationLimits::default(),
                &cancel,
            )
            .unwrap()
            .worker();
        let negative = eligibility
            .evaluate(
                &[-1.0, 1.0],
                DerivativeOrder::Value,
                &mut BTreeMap::new(),
                &cancel,
            )
            .unwrap()
            .values[0]
            == 1.0;
        if let Configuration::Fixed(_, options) = configs.get_mut(&residual.id).unwrap() {
            options.start[0] = if negative { -1.0 } else { 1.0 };
        }
    }
    let factory = inner
        .factory(
            configs,
            Arc::new(Roots),
            &pse_math::implicit::accelerators::Accelerators::standard(),
            cancel.clone(),
            EvaluationLimits::default(),
        )
        .unwrap();
    let mut provider = factory.create().unwrap();
    let request = ProviderRequest {
        outputs: vec![0],
        order: DerivativeOrder::Second,
    };
    let context = EvaluationContext {
        cancelled: &cancel,
        max_result_bytes: 1024,
    };
    let jet = provider.evaluate(&[4.0], &request, &context).unwrap();
    assert_eq!(jet.values, vec![-2.0]);
    assert!((jet.jacobian[0] + 0.25).abs() < 1e-12);
    assert!((jet.hessians[0] - 0.03125).abs() < 1e-12);
    assert!(provider.evaluate(&[0.0], &request, &context).is_err());
    assert!(provider.evaluate(&[9.0], &request, &context).is_err());
}

#[test]
fn kernel_child_indices_resolve_in_the_authors_import_scope() {
    let text = r#"package kinds {entity kind item {}}
    package data {use kinds @"1.0.0"; entity kinds.item a {} set items:Set<kinds.item>={a};}
    package library {
      use kinds @"1.0.0";
      def Model(selected:Set<kinds.item>) {var x[j in selected]:Scalar;}
    }
    package p {
      use data @"1.0.0" as local;
      use library @"1.0.0";
      def Root {}
      test run fixture {dof 0; run pure; fix root.x[local.a]=5;} {
        child root:library.Model=library.Model(selected=local.items);
        expect root.x[local.a]==5 tolerance 1e-12;
      }
    }"#;
    let (mut workspace, rows, _, _) = setup(text);
    let fixture = rows
        .iter()
        .find(|row| row.name == "run")
        .unwrap()
        .declaration_id;
    let results = workspace
        .check_modeling_expectations(
            fixture,
            root_instance(fixture),
            Bindings::default(),
            Limits::default(),
            &CaseValues::default(),
            Profile::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert!(results.iter().all(|check| check.passed));
}

#[test]
fn kernel_structural_interface_parameters_bind_effective_members_before_guards() {
    let text = r#"package p {
      entity kind item {} entity item a {} entity item b {}
      set items:Set<item>={a,b};
      set other:Set<item>={a};
      interface Indexed {
        param members:Set<item>;
        let indexed[j in members]:Scalar;
      }
      interface Aggregate extends Indexed {
        param enabled:Boolean;
        param x[j in members]:Scalar=2;
        override let indexed[j in members]:Scalar=x[j];
        let total:Scalar=sum(j in members | x[j]);
        when enabled {eq closure:total==4;}
      }
      def Root(selected:Set<item>,active:Boolean):Aggregate {
        override param members:Set<item>=selected;
        override param enabled:Boolean=active;
        override let indexed[j in members]:Scalar=x[j]+1;
      }
      test run {
        child root:Aggregate=Root(selected=items,active=true);
        expect root.total==4 tolerance 1e-12;
        expect root.indexed[a]==3 tolerance 1e-12;
      }
    }"#;
    let (mut workspace, mut rows, names, root) = setup(text);
    let fixture = rows
        .iter()
        .find(|r| r.name == "run")
        .unwrap()
        .declaration_id;
    let checks = workspace
        .check_modeling_expectations(
            fixture,
            root_instance(fixture),
            Bindings::default(),
            Limits::default(),
            &CaseValues::default(),
            Profile::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert!(checks.iter().all(|check| check.passed));
    let model = workspace
        .specialize_modeling(
            fixture,
            root_instance(fixture),
            Bindings::default(),
            Limits::default(),
        )
        .unwrap();
    assert_eq!(model.equations.len(), 1);
    assert!(
        model
            .symbols
            .values()
            .all(|symbol| !symbol.lineage.path.ends_with("members")
                && !symbol.lineage.path.ends_with("enabled"))
    );
    rows.iter_mut()
        .find(|row| row.parent_id == Some(root) && row.name == "indexed")
        .unwrap()
        .value
        .binding
        .as_mut()
        .unwrap()
        .indices[0]
        .domain = "other".into();
    assert!(workspace.publish_modeling(rows, names).is_err());
}

#[test]
fn kernel_definition_projection_shares_constructor_defaults_and_effective_members() {
    let text = r#"package data {
      entity kind item {} entity item a {} entity item b {}
      set both:Set<item>={a,b}; set one:Set<item>={b};
      interface Indexed {param members:Set<item>; var x[j in members]:Scalar;}
      interface Default extends Indexed {override param members:Set<item>=both;}
      def Base(selected:Set<item>=one):Default {override param members:Set<item>=selected;}
      preset Single=Base(selected=one);
    }
    package p {
      use data @"1.0.0";
      def Root(pkg:data.Indexed=data.Base) {
        param members:Set<data.item>=pkg.members;
        child state:data.Indexed=pkg;
        eq values[j in members]:state.x[j]==2;
        let count:Scalar=sum(j in members | 1);
      }
      test run {
        child single:Root=Root();
        child pair:Root=Root(pkg=data.Base(selected=data.both));
        child preset:Root=Root(pkg=data.Single);
        expect single.count==1 tolerance 1e-12;
        expect pair.count==2 tolerance 1e-12;
        expect preset.count==1 tolerance 1e-12;
        expect sum(j in data.Base.members | 1)==1 tolerance 1e-12;
      }
    }"#;
    let (mut workspace, _, _, root) = setup(text);
    let model = workspace
        .specialize_modeling(
            root,
            root_instance(root),
            Bindings::default(),
            Limits::default(),
        )
        .unwrap();
    assert_eq!(model.equations.len(), 1);
    let fixture = source(text)
        .into_iter()
        .find(|r| r.name == "run")
        .unwrap()
        .declaration_id;
    let checks = workspace
        .check_modeling_expectations(
            fixture,
            root_instance(fixture),
            Bindings::default(),
            Limits::default(),
            &CaseValues::default(),
            Profile::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert!(checks.iter().all(|c| c.passed));
    let missing = text.replace("selected:Set<item>=one", "selected:Set<item>");
    let (mut workspace, _, _, root) = setup(&missing);
    assert!(
        workspace
            .specialize_modeling(
                root,
                root_instance(root),
                Bindings::default(),
                Limits::default()
            )
            .unwrap_err()
            .to_string()
            .contains("missing argument selected")
    );
    let recursive = text.replace(
        "override param members:Set<item>=selected",
        "override param members:Set<item>=Base.members",
    );
    let (mut workspace, _, _, root) = setup(&recursive);
    assert!(
        workspace
            .specialize_modeling(
                root,
                root_instance(root),
                Bindings::default(),
                Limits::default()
            )
            .unwrap_err()
            .to_string()
            .contains("recursive structural definition")
    );
}

#[test]
fn kernel_fixture_paths_bind_the_selected_implementation_physical_contract() {
    let text = r#"package p {
      interface I {var x:Scalar;}
      def Concrete:I {param hidden:Scalar=3; var secret:Scalar; eq balance:x==secret+hidden;}
      def Root {}
      test run fixture {dof 0; value state.hidden=7; fix state.secret=2;} {
        child state:I=Concrete;
        expect state.x==9 tolerance 1e-12;
      }
    }"#;
    let (mut workspace, rows, _, _) = setup(text);
    let fixture = rows
        .iter()
        .find(|r| r.name == "run")
        .unwrap()
        .declaration_id;
    let model = workspace
        .specialize_modeling(
            fixture,
            root_instance(fixture),
            Bindings::default(),
            Limits::default(),
        )
        .unwrap();
    let specifications = &model.fixtures.values().next().unwrap().specifications;
    assert_eq!(specifications["state.hidden"].value, Some(7.));
    assert_eq!(specifications["state.secret"].value, Some(2.));
    assert_eq!(specifications["state.secret"].fixed, Some(true));
    let invalid = text.replace("state.secret=2", "state.absent=2");
    let (mut workspace, rows, _, _) = setup(&invalid);
    let fixture = rows
        .iter()
        .find(|r| r.name == "run")
        .unwrap()
        .declaration_id;
    assert!(
        workspace
            .specialize_modeling(
                fixture,
                root_instance(fixture),
                Bindings::default(),
                Limits::default()
            )
            .is_err()
    );
}

#[test]
fn kernel_immutable_function_data_is_visible_differentiable_and_invalidated() {
    let text = r#"package data {
      entity kind item {} entity item a {} entity item b {}
      set items:Set<item>={a,b};
      table coefficients[j:item]:{value:Scalar};
      dataset values:coefficients source "synthetic" {[a]=[3];[b]=[4];}
      fn energy(x:Scalar,n:Scalar[item])->Scalar=sum(j in items | coefficients[j].value*x*n[j]);
    }
    package p {
      use data @"1.0.0";
      def Root {}
      test run {
        param n[j in data.items]:Scalar=if j==data.a then 1 else 2;
        expect data.energy(2,n)==22 tolerance 1e-12;
        expect partial(data.energy,x)(2,n)==11 tolerance 1e-12;
      }
    }"#;
    let (mut workspace, mut rows, names, _) = setup(text);
    let fixture = rows
        .iter()
        .find(|r| r.name == "run")
        .unwrap()
        .declaration_id;
    let evaluate = |workspace: &mut CompilerWorkspace| {
        workspace
            .check_modeling_expectations(
                fixture,
                root_instance(fixture),
                Bindings::default(),
                Limits::default(),
                &CaseValues::default(),
                Profile::default(),
                Arc::new(AtomicBool::new(false)),
            )
            .unwrap()
    };
    assert!(evaluate(&mut workspace).iter().all(|check| check.passed));
    rows.iter_mut()
        .find(|row| row.name == "values")
        .unwrap()
        .value
        .dataset
        .as_mut()
        .unwrap()
        .rows[0]
        .values[0] = "5".into();
    workspace
        .publish_modeling(rows.clone(), names.clone())
        .unwrap();
    let changed = evaluate(&mut workspace);
    assert_eq!(changed.iter().filter(|check| !check.passed).count(), 2);
    let mut actual = changed.iter().map(|check| check.actual).collect::<Vec<_>>();
    actual.sort_by(f64::total_cmp);
    assert_eq!(actual, vec![13., 26.]);
    let mut clean = CompilerWorkspace::new(workspace.inputs.clone(), workspace.limits).unwrap();
    clean.publish_modeling(rows, names).unwrap();
    assert_eq!(
        changed.iter().map(|check| check.actual).collect::<Vec<_>>(),
        evaluate(&mut clean)
            .iter()
            .map(|check| check.actual)
            .collect::<Vec<_>>()
    );
}

#[test]
fn kernel_imported_table_paths_preserve_rows_columns_and_visibility() {
    let text = r#"package data {
        entity kind item {} entity item a {}
        table coefficients[j:item]:{value:Scalar};
        dataset values:coefficients source "synthetic" {[a]=[7];}
        fn read(row:Row<coefficients>)->Scalar=row.value;
    }
    package p {
        use data @"1.0.0" as imported;
        def Root {}
        test run {
            expect imported.read(imported.coefficients[imported.a])==7 tolerance 1e-12;
            expect imported.coefficients[imported.a].value==7 tolerance 1e-12;
        }
    }"#;
    let (mut workspace, mut rows, names, _) = setup(text);
    let fixture = rows
        .iter()
        .find(|row| row.name == "run")
        .unwrap()
        .declaration_id;
    let checks = workspace
        .check_modeling_expectations(
            fixture,
            root_instance(fixture),
            Bindings::default(),
            Limits::default(),
            &CaseValues::default(),
            Profile::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert_eq!(checks.len(), 2);
    assert!(checks.iter().all(|check| check.passed));
    rows.retain(|row| row.value.import.is_none());
    assert!(workspace.publish_modeling(rows, names).is_err());
}

#[test]
fn kernel_explicit_primitive_functions_preserve_references_and_derivative_checks() {
    let text = r#"package p {
        fn f(x:Scalar)->Scalar=2+x;
        fn a(x:Scalar)->Scalar=2*x+x*x/2;
        fn b(x:Scalar)->Scalar=2*log(x)+x;
        interface PrimitivePair {
            param first:Fn(x:Scalar)->Scalar;
            param second:Fn(x:Scalar)->Scalar;
            param x:Scalar;
            param reference:Scalar;
            param first_reference:Scalar;
            param second_reference:Scalar;
            let first_value:Scalar=first_reference+first(x)-first(reference);
            let second_value:Scalar=second_reference+second(x)-second(reference);
        }
        def Root:PrimitivePair {
            override param first:Fn(x:Scalar)->Scalar=a;
            override param second:Fn(x:Scalar)->Scalar=b;
            override param x:Scalar=3;
            override param reference:Scalar=1;
            override param first_reference:Scalar=10;
            override param second_reference:Scalar=20;
        }
        test identities {
            child root:PrimitivePair=Root();
            expect root.first_value==18 tolerance 1e-12;
            expect root.second_value==22+2*log(3) tolerance 1e-12;
            expect partial(a,x)(3)==f(3) tolerance 1e-12;
            expect partial(b,x)(3)==f(3)/3 tolerance 1e-12;
        }
    }"#;
    let (mut workspace, rows, _, _) = setup(text);
    let id = rows
        .iter()
        .find(|r| r.name == "identities")
        .unwrap()
        .declaration_id;
    let results = workspace
        .check_modeling_expectations(
            id,
            root_instance(id),
            Bindings::default(),
            Limits::default(),
            &CaseValues {
                scalars: BTreeMap::new(),
            },
            Profile::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert_eq!(results.len(), 4);
    assert!(results.iter().all(|r| r.passed), "{results:?}");
    let mut bad = rows;
    bad.iter_mut()
        .find(|r| r.name == "a" && r.value.function.is_some())
        .unwrap()
        .value
        .function
        .as_mut()
        .unwrap()
        .body = Some("2*x+x*x".into());
    workspace
        .publish_modeling(
            bad,
            BTreeMap::from([(
                "Scalar".into(),
                workspace.inputs.quantities.neutral_dimensionless().unwrap(),
            )]),
        )
        .unwrap();
    let results = workspace
        .check_modeling_expectations(
            id,
            root_instance(id),
            Bindings::default(),
            Limits::default(),
            &CaseValues {
                scalars: BTreeMap::new(),
            },
            Profile::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert_eq!(results.iter().filter(|r| !r.passed).count(), 2);
}

#[test]
fn kernel_function_slots_select_overrides_indexed_methods_and_forward_arguments() {
    let text = r#"package p {
      entity kind item {} entity item a {} entity item b {}
      set items:Set<item>={a,b};
      fn double(x:Scalar)->Scalar=2*x;
      fn triple(x:Scalar)->Scalar=3*x;
      fn apply(method:Fn(x:Scalar)->Scalar,x:Scalar)->Scalar=method(x);
      table methods[j:item]:Fn(x:Scalar)->Scalar;
      dataset choices:methods source "synthetic" {[a]=[double];[b]=[triple];}
      interface Port {
        param method:Fn(x:Scalar)->Scalar=double;
        param alias:Fn(x:Scalar)->Scalar=method;
        let value:Scalar=apply(alias,2);
      }
      def Root:Port {
        override param method:Fn(x:Scalar)->Scalar=triple;
        param indexed[j in items]:Fn(x:Scalar)->Scalar=methods[j];
        let total:Scalar=sum(j in items | indexed[j](2));
      }
      test run {
        child root:Root=Root();
        expect root.value==6 tolerance 1e-12;
        expect root.total==10 tolerance 1e-12;
      }
    }"#;
    let (mut workspace, rows, _, root) = setup(text);
    let test = rows
        .iter()
        .find(|row| row.name == "run")
        .unwrap()
        .declaration_id;
    let checks = workspace
        .check_modeling_expectations(
            test,
            root_instance(test),
            Bindings::default(),
            Limits::default(),
            &CaseValues::default(),
            Profile::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert!(checks.iter().all(|check| check.passed));
    let mut cyclic = rows;
    cyclic
        .iter_mut()
        .find(|row| row.is_override && row.name == "method")
        .unwrap()
        .value
        .binding
        .as_mut()
        .unwrap()
        .expression = Some("alias".into());
    workspace
        .publish_modeling(
            cyclic,
            BTreeMap::from([(
                "Scalar".into(),
                workspace.inputs.quantities.neutral_dimensionless().unwrap(),
            )]),
        )
        .unwrap();
    let error = workspace
        .check_modeling_expectations(
            test,
            root_instance(test),
            Bindings::default(),
            Limits::default(),
            &CaseValues::default(),
            Profile::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap_err();
    assert!(
        error.to_string().contains("recursive function parameter"),
        "{error}"
    );
    let _ = root;
}

#[test]
fn kernel_shared_partials_preserve_lets_and_mixed_derivatives() {
    let mut bindings = vec!["u0=x*y".to_owned()];
    for i in 1..=14 {
        bindings.push(format!("u{i}=sin(u{})+u{}", i - 1, i - 1));
    }
    let text = format!(
        "package p {{ fn repeated(x:Scalar,y:Scalar)->Scalar=u14 where {}; def Root {{var x:Scalar; var y:Scalar; eq value:repeated(x,y)==0; eq first:partial(repeated,x)(x,y)==0; eq mixed:partial(repeated,x,y)(x,y)==0; eq second:partial(repeated,x,x)(x,y)==0;}} }}",
        bindings.join(",")
    );
    let (mut workspace, _, _, root) = setup(&text);
    let model = workspace
        .specialize_modeling(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings::default(),
            Limits::default(),
        )
        .unwrap();
    let admitted = admit(&mut workspace, root);
    let cancel = Arc::new(AtomicBool::new(false));
    let artifact = fixture(
        &admitted,
        workspace.inputs.quantities.clone(),
        DerivativeOrder::Second,
        &cancel,
    )
    .unwrap();
    let values = admitted
        .inputs
        .iter()
        .map(|id| {
            if model.symbols[id].lineage.path.ends_with(".x") {
                0.
            } else {
                1.
            }
        })
        .collect::<Vec<_>>();
    let result = artifact
        .worker()
        .evaluate(
            &values,
            DerivativeOrder::Second,
            &mut BTreeMap::new(),
            &cancel,
        )
        .unwrap();
    let mut outputs = result.equations(&admitted);
    outputs.sort_by(f64::total_cmp);
    assert_eq!(outputs, vec![0., 0., 16384., 16384.]);
}

#[test]
fn kernel_implicit_functions_read_enclosing_indexed_members() {
    let text = "package p {entity kind item {} entity item a {} entity item b {} set items:Set<item>={a,b}; fn total(x:Scalar[item],members:Set<item>)->Scalar=sum(j in members | x[j]); interface Contract {param selected:Set<item>; param x[j in selected]:Scalar=2; implicit root {var y:Scalar; eq e:y==total(x,selected);}} def Root:Contract {override param selected:Set<item>=items;} }";
    let (mut workspace, _, _, root) = setup(text);
    let model = workspace
        .specialize_modeling(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings::default(),
            Limits::default(),
        )
        .unwrap();
    let admitted = admit(&mut workspace, root);
    assert_eq!(model.equations.len(), 1);
    let cancel = Arc::new(AtomicBool::new(false));
    let artifact = fixture(
        &admitted,
        workspace.inputs.quantities.clone(),
        DerivativeOrder::First,
        &cancel,
    )
    .unwrap();
    let values = admitted
        .inputs
        .iter()
        .map(|id| {
            if model.symbols[id].lineage.path.ends_with(".y") {
                4.
            } else {
                2.
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(
        artifact
            .worker()
            .evaluate(
                &values,
                DerivativeOrder::First,
                &mut BTreeMap::new(),
                &cancel
            )
            .unwrap()
            .equations(&admitted),
        vec![0.]
    );
}

#[test]
fn kernel_negative_literals_keep_the_expected_physical_contract() {
    let inputs = super::super::tests::inputs();
    let energy =
        QuantityTypeId::from_id(SemanticId::parse_hex("d5bb3d48b9804f2f8d5a6f0a7cadaee8").unwrap());
    let names = BTreeMap::from([("EnergyDifference".into(), energy)]);
    let rows = source(
        "package p {fn negative()->EnergyDifference=-2{J/mol}; test physical fixture {dof 0; run pure;} {expect negative()==-2{J/mol} tolerance 1e-10{J/mol};}} ",
    );
    let id = rows
        .iter()
        .find(|r| r.name == "physical")
        .unwrap()
        .declaration_id;
    let mut workspace = CompilerWorkspace::new(inputs, WorkspaceLimits::default()).unwrap();
    workspace.publish_modeling(rows, names).unwrap();
    let checks = workspace
        .check_modeling_expectations(
            id,
            root_instance(id),
            Bindings::default(),
            Limits::default(),
            &CaseValues::default(),
            Profile::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert_eq!(checks.len(), 1);
    assert!(checks[0].passed);
}

#[test]
fn kernel_generic_normalization_requires_the_concrete_physical_operation() {
    for (quantity, value, accepted) in [
        ("dc255c612cf27e30cb835377c8dafcf4", "2", true),
        ("459a933fd00837bbc50372e31ac9801c", "2{K}", true),
        ("1831d0d72dc74b299ba8ecb6d4da6f53", "2{J/mol}", false),
    ] {
        let mut inputs = super::super::tests::inputs();
        inputs.preconditions = Arc::new(
            PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions()).unwrap(),
        );
        let names = BTreeMap::from([
            (
                "Scalar".into(),
                inputs.quantities.neutral_dimensionless().unwrap(),
            ),
            (
                "Value".into(),
                QuantityTypeId::from_id(SemanticId::parse_hex(quantity).unwrap()),
            ),
        ]);
        let rows = source(&format!(
            "package p {{fn norm<Q>(a:Q,b:Q)->Scalar=a/b; fn value()->Value={value}; test trial fixture {{dof 0; run pure;}} {{expect norm(value(),value())==1 tolerance 1e-12;}}}}"
        ));
        let id = rows
            .iter()
            .find(|r| r.name == "trial")
            .unwrap()
            .declaration_id;
        let mut workspace = CompilerWorkspace::new(inputs, WorkspaceLimits::default()).unwrap();
        workspace.publish_modeling(rows, names).unwrap();
        let result = workspace.check_modeling_expectations(
            id,
            root_instance(id),
            Bindings::default(),
            Limits::default(),
            &CaseValues::default(),
            Profile::default(),
            Arc::new(AtomicBool::new(false)),
        );
        if accepted {
            assert!(result.unwrap().iter().all(|check| check.passed));
        } else {
            assert!(
                result.is_err(),
                "a generic signature must not create a physical operation"
            );
        }
    }
}

#[test]
fn kernel_finite_reductions_retain_domains_prototypes_and_derivatives() {
    let mut inputs = super::super::tests::inputs();
    inputs.preconditions = Arc::new(
        PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions()).unwrap(),
    );
    let quantity = |id| QuantityTypeId::from_id(SemanticId::parse_hex(id).unwrap());
    let names = BTreeMap::from([
        (
            "Scalar".into(),
            inputs.quantities.neutral_dimensionless().unwrap(),
        ),
        (
            "ComponentFlow".into(),
            quantity("8ce2f0977877ae56c3712f710201be89"),
        ),
        ("Flow".into(), quantity("6df479e3be3bda77c5938727311f1063")),
    ]);
    let text = r#"package p {
      @id("1c998211e5d74955863f377662f56526") entity kind species {}
      entity species a {} entity species b {}
      set species_set:Set<species>={a,b};
      set empty:Set<species>={};
      fn guarded(value:ComponentFlow)->ComponentFlow valid(value>=0{mol/s})=value;
      fn total(values:ComponentFlow[species],selected:Set<species>)->Flow=sum(j in selected | guarded(values[j]));
      def Root {
        var values[j in species_set]:ComponentFlow;
        eq balance:total(values,species_set)==7{mol/s};
        eq empty_balance:total(values,empty)==0{mol/s};
        eq filtered:sum(j in species_set where j in empty | values[j])==0{mol/s};
      }
    }"#;
    let rows = source(text);
    let root = rows
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let mut workspace = CompilerWorkspace::new(inputs.clone(), WorkspaceLimits::default()).unwrap();
    workspace.publish_modeling(rows, names.clone()).unwrap();
    let admitted = admit(&mut workspace, root);
    let cancel = Arc::new(AtomicBool::new(false));
    let compiled = fixture(
        &admitted,
        workspace.inputs.quantities.clone(),
        DerivativeOrder::Second,
        &cancel,
    )
    .unwrap();
    let mut worker = compiled.worker();
    let mut residual = worker
        .evaluate(
            &[3., 4.],
            DerivativeOrder::Second,
            &mut BTreeMap::new(),
            &cancel,
        )
        .unwrap()
        .equations(&admitted);
    residual.sort_by(f64::total_cmp);
    assert_eq!(residual, vec![0., 0., 0.]);
    let case = CaseValues {
        scalars: admitted.inputs.iter().copied().zip([3., 4.]).collect(),
    };
    let mut numeric = compiled.assembly.worker(BTreeMap::new(), cancel);
    let jacobian = numeric.jacobian(&case).unwrap();
    let (rows, columns, values) = (
        jacobian.row_idx().to_vec(),
        jacobian.col_ptr().to_vec(),
        jacobian.val().to_vec(),
    );
    let row_count = jacobian.nrows();
    for (column, id) in admitted.inputs.iter().enumerate() {
        let mut plus = case.clone();
        let mut minus = case.clone();
        *plus.scalars.get_mut(id).unwrap() += 1e-4;
        *minus.scalars.get_mut(id).unwrap() -= 1e-4;
        let a = numeric.constraints(&plus).unwrap().to_vec();
        let b = numeric.constraints(&minus).unwrap().to_vec();
        let mut analytic = vec![0.; row_count];
        for position in columns[column]..columns[column + 1] {
            analytic[rows[position]] = values[position];
        }
        for ((a, b), derivative) in a.iter().zip(&b).zip(analytic) {
            assert!(((a - b) / 2e-4 - derivative).abs() < 1e-9);
        }
    }
    // Same dimensions and body syntax cannot replace the consumed species kind.
    let wrong = text.replace(
        "1c998211e5d74955863f377662f56526",
        "7d986668d824439f9b5828046038365d",
    );
    assert!(
        workspace
            .publish_modeling(source(&wrong), names.clone())
            .is_err()
    );
    // An empty reduction still rejects an affine-point prototype.
    let mut bad_names = names;
    bad_names.insert(
        "ComponentFlow".into(),
        quantity("c64b96975a4a59755f8711d3bf628bc9"),
    );
    let invalid = text.replace(
        "fn total(values:ComponentFlow[species],selected:Set<species>)->Flow",
        "fn total(values:ComponentFlow[species],selected:Set<species>)->ComponentFlow",
    );
    let mut other = CompilerWorkspace::new(inputs, WorkspaceLimits::default()).unwrap();
    assert!(other.publish_modeling(source(&invalid), bad_names).is_err());
    let static_source = text.replace("def Root {", "table flow_data[j:species]:ComponentFlow; dataset values_data:flow_data source \"synthetic component values\" {[a]=[2{mol/s}];[b]=[2{mol/s}];} def Root {param total_static:Flow=sum(j in species_set | flow_data[j]); param empty_static:Flow=sum(j in empty | flow_data[j]);");
    workspace
        .publish_modeling(
            source(&static_source),
            BTreeMap::from([
                (
                    "Scalar".into(),
                    workspace.inputs.quantities.neutral_dimensionless().unwrap(),
                ),
                (
                    "ComponentFlow".into(),
                    quantity("8ce2f0977877ae56c3712f710201be89"),
                ),
                ("Flow".into(), quantity("6df479e3be3bda77c5938727311f1063")),
            ]),
        )
        .unwrap();
    let model = workspace
        .specialize_modeling(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings {
                demand: vec!["total_static".into(), "empty_static".into()],
                ..Bindings::default()
            },
            Limits::default(),
        )
        .unwrap();
    for (path, expected) in [("total_static", 4.), ("empty_static", 0.)] {
        let value = model.symbols[&model.paths[path]].initial.as_ref().unwrap();
        assert!(
            matches!(value,pse_modeling::specialize::Value::Number{bits,quantity:q} if f64::from_bits(*bits)==expected && *q==quantity("6df479e3be3bda77c5938727311f1063"))
        );
    }
}

#[test]
fn kernel_finite_folds_compose_source_functions_and_library_partials() {
    let text = r#"package p {entity kind item {} entity item a {} entity item b {} entity item c {}
      set items:Set<item>={a,b,c}; set empty:Set<item>={};
      fn fold_product(values:Scalar[item],members:Set<item>)->Scalar=fold(acc,value; j in members | values[j]; acc*value);
      fn filtered(values:Scalar[item],members:Set<item>)->Scalar=fold(acc,value; j in members where j!=b | values[j]; acc*value);
      fn single(values:Scalar[item],members:Set<item>)->Scalar=fold(acc,value; j in members where j==a | values[j]; acc*value);
      fn shadowed(values:Scalar[item],members:Set<item>,acc:Scalar)->Scalar=fold(acc,value; j in members | values[j]+acc; acc*value);
      def Root {}
      test values fixture {dof 0;run pure;} {param v[j in items]:Scalar=2;
        expect fold_product(v,items)==8 tolerance 1e-12;
        expect partial(fold_product,values[a])(v,items)==4 tolerance 1e-12;
        expect partial(fold_product,values[a],values[b])(v,items)==2 tolerance 1e-12;
        expect filtered(v,items)==4 tolerance 1e-12;
        expect single(v,items)==2 tolerance 1e-12;
        expect shadowed(v,items,10)==1728 tolerance 1e-10;
      }}"#;
    let (mut workspace, rows, names, _) = setup(text);
    let case = rows
        .iter()
        .find(|r| r.name == "values" && r.value.scope.is_some())
        .unwrap()
        .declaration_id;
    let run = |workspace: &mut CompilerWorkspace| {
        workspace.check_modeling_expectations(
            case,
            root_instance(case),
            Bindings::default(),
            Limits::default(),
            &CaseValues::default(),
            Profile::default(),
            Arc::new(AtomicBool::new(false)),
        )
    };
    let checks = run(&mut workspace).unwrap();
    assert_eq!(checks.len(), 6);
    assert!(checks.iter().all(|v| v.passed));
    let empty = text.replace(
        "expect fold_product(v,items)==8",
        "expect fold_product(v,empty)==8",
    );
    workspace
        .publish_modeling(source(&empty), names.clone())
        .unwrap();
    assert!(
        run(&mut workspace)
            .unwrap_err()
            .to_string()
            .contains("at least one selected member")
    );
    let duplicate = text.replace("fold(acc,value;", "fold(acc,acc;");
    assert!(
        workspace
            .publish_modeling(source(&duplicate), names)
            .is_err()
    );
}

#[test]
fn kernel_indexed_accounting_values_are_readable_in_checks_and_ports() {
    let (mut workspace, _, _, root) = setup(
        "package p {entity kind item {} entity item a {} entity item b {} set items:Set<item>={a,b}; def Root {param x[j in items]:Scalar=2; accumulate total[j in items]:Scalar accounting tolerance 0.01; contribute [j in items] total[j] role positive=x[j]; contribute [j in items] total[j] role negative=1; port result[j in items]:Scalar=total[j]; annotation check total(total[j]==1); let combined:Scalar=sum(j in items | total[j]); annotation report combined(\"combined\");}}",
    );
    let cancel = Arc::new(AtomicBool::new(false));
    let prepared = workspace
        .prepare_modeling_cancellable(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings::default(),
            Limits::default(),
            cancel.clone(),
        )
        .unwrap();
    let a = &prepared.admitted;
    let artifact = fixture(
        a,
        workspace.inputs.quantities.clone(),
        DerivativeOrder::Second,
        &cancel,
    )
    .unwrap();
    let inputs = vec![2.0; a.inputs.len()];
    let values = artifact
        .worker()
        .evaluate(
            &inputs,
            DerivativeOrder::Second,
            &mut BTreeMap::new(),
            &cancel,
        )
        .unwrap()
        .values;
    for (output, value) in a.outputs.iter().zip(values) {
        match output {
            ModelingOutput::Hint {
                kind: ModelingHint::Check,
                ..
            } => assert_eq!(value, 1.0),
            ModelingOutput::Member(id)
                if prepared.model.symbols[id]
                    .lineage
                    .path
                    .ends_with(".combined") =>
            {
                assert_eq!(value, 2.0)
            }
            _ => {}
        }
    }
    assert_eq!(
        prepared
            .model
            .annotations
            .iter()
            .filter(|a| matches!(a.value, pse_modeling::annotation::AnnotationValue::Check(_)))
            .count(),
        2
    );
}

#[test]
fn kernel_qualified_enumeration_arguments_select_structure() {
    let text = r#"package options { enum Mode { a,b } def Selected(mode:Mode=Mode.a) {
        var x:Scalar; when mode==Mode.a { eq a:x==1; } when mode==Mode.b { eq b:x==2; }
    } } package consumer { use options @"1.0.0"; def Root {
        child selected:options.Selected=options.Selected(mode=options.Mode.b);
    } }"#;
    let (mut workspace, _, _, root) = setup(text);
    let model = workspace
        .specialize_modeling(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings::default(),
            Limits::default(),
        )
        .unwrap();
    assert_eq!(model.equations.len(), 1);
    assert!(
        model
            .equations
            .iter()
            .any(|e| e.lineage.path.ends_with(".b"))
    );
}

#[test]
fn kernel_pure_function_names_do_not_capture_caller_members() {
    let text = r#"package p {
      entity kind item {} entity item a {} entity item b {}
      table data[j:item]:Scalar; dataset values:data source "synthetic" {[a]=[5];[b]=[7];}
      fn helper(x:Scalar)->Scalar=3*x;
      fn read(x:Scalar,j:item)->Scalar=data[j]+helper(x);
      interface Port {
        param method:Fn(x:Scalar,j:item)->Scalar=p.read;
        let value:Scalar=method(2,b);
      }
      def Root:Port { param data:Scalar=999; let helper:Scalar=111; }
      test run { child root:Root=Root; expect root.value==13 tolerance 1e-12; }
    }"#;
    let (mut workspace, rows, _, _) = setup(text);
    let case = rows
        .iter()
        .find(|r| r.name == "run")
        .unwrap()
        .declaration_id;
    let checks = workspace
        .check_modeling_expectations(
            case,
            root_instance(case),
            Bindings::default(),
            Limits::default(),
            &CaseValues::default(),
            Profile::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert!(checks.iter().all(|c| c.passed));
}

#[test]
fn kernel_partial_distinguishes_equal_argument_values_through_aliases() {
    let (mut workspace, _, _, root) = setup(
        "package p { fn pair(a:Scalar,b:Scalar)->Scalar=a*b; fn alias(x:Scalar)->Scalar=x; def Root {var x:Scalar; eq first:partial(pair,a)(alias(x),alias(x))==0; eq mixed:partial(pair,a,b)(alias(x),alias(x))==0;} }",
    );
    let admitted = admit(&mut workspace, root);
    let cancel = Arc::new(AtomicBool::new(false));
    let artifact = fixture(
        &admitted,
        workspace.inputs.quantities.clone(),
        DerivativeOrder::Second,
        &cancel,
    )
    .unwrap();
    let result = artifact
        .worker()
        .evaluate(
            &[3.],
            DerivativeOrder::Second,
            &mut BTreeMap::new(),
            &cancel,
        )
        .unwrap();
    let mut values = result.equations(&admitted);
    values.sort_by(f64::total_cmp);
    assert_eq!(values, vec![1., 3.]);
}

#[test]
fn kernel_objective_selection_preserves_values_derivatives_and_penalty_direction() {
    use pse_math::binding::{CaseValues, ObjectiveSense};
    for (sense, sign) in [("minimize", 1.0), ("maximize", -1.0)] {
        for elastic in [false, true] {
            let relaxation = if elastic {
                "eq e:x==4; relax r on e nominal 2;"
            } else {
                ""
            };
            let (mut workspace, _, _, root) = setup(&format!(
                "package p {{def Root {{var x:Scalar; let cost:Scalar=x*x; annotation objective cost({sense}); {relaxation} }} }}"
            ));
            let prepared = workspace
                .prepare_modeling_cancellable(
                    root,
                    InstanceId::from_id(SemanticId::NIL),
                    Bindings::default(),
                    Limits::default(),
                    Arc::new(AtomicBool::new(false)),
                )
                .unwrap();
            let a = &prepared.admitted;
            assert_eq!(
                a.case.objective().unwrap().sense,
                if sign > 0.0 {
                    ObjectiveSense::Minimize
                } else {
                    ObjectiveSense::Maximize
                }
            );
            let cancel = Arc::new(AtomicBool::new(false));
            let f = fixture(
                a,
                workspace.inputs.quantities.clone(),
                DerivativeOrder::Second,
                &cancel,
            )
            .unwrap();
            let x = prepared
                .model
                .symbols
                .values()
                .find(|s| s.lineage.path.ends_with(".x"))
                .unwrap()
                .id;
            let mut values = CaseValues {
                scalars: a
                    .inputs
                    .iter()
                    .map(|id| {
                        (
                            *id,
                            if *id == x {
                                2.0
                            } else if let Some(pse_modeling::specialize::Value::Number {
                                bits,
                                ..
                            }) = prepared.model.symbols[id].initial
                            {
                                f64::from_bits(bits)
                            } else {
                                0.0
                            },
                        )
                    })
                    .collect(),
            };
            if elastic {
                values.scalars.insert(
                    prepared.model.elastic.values().next().unwrap().slacks[1],
                    2.0,
                );
            }
            let mut worker = f.assembly.worker(BTreeMap::new(), cancel);
            assert!(
                (worker.objective(&values).unwrap()
                    - (sign * 4.0 + if elastic { 1.0 } else { 0.0 }))
                .abs()
                    < 1e-12
            );
            let index = f.assembly.columns().iter().position(|id| *id == x).unwrap();
            assert!((worker.gradient(&values).unwrap()[index] - sign * 4.0).abs() < 1e-12);
            let hessian = worker
                .hessian(&values, 1.0, &vec![0.0; a.case.rows().len()])
                .unwrap();
            assert!((*hessian.get(index, index).unwrap() - sign * 2.0).abs() < 1e-12);
        }
    }
    for text in [
        "package p {def Root {var x:Scalar; annotation objective x(minimize); annotation objective x(maximize);}}",
        "package p {entity kind I {} entity I a {} entity I b {} set items:Set<I>={a,b}; def Root {var x[j in items]:Scalar; annotation objective x(minimize);}}",
    ] {
        let (mut workspace, _, _, root) = setup(text);
        assert!(
            workspace
                .admit_modeling(
                    root,
                    InstanceId::from_id(SemanticId::NIL),
                    Bindings::default(),
                    Limits::default()
                )
                .is_err()
        );
    }
}

#[test]
fn kernel_dimensional_objectives_require_normalization_before_elastic_combination() {
    for elastic in [false, true] {
        let mut input = super::super::tests::inputs();
        input.quantities = Arc::new(pse_quantity::standard::standard_registry().unwrap());
        let temperature = QuantityTypeId::from_id(
            SemanticId::parse_hex("c64b96975a4a59755f8711d3bf628bc9").unwrap(),
        );
        let mut names = BTreeMap::from([("Temperature".into(), temperature)]);
        names.insert(
            "Scalar".into(),
            input.quantities.neutral_dimensionless().unwrap(),
        );
        let relaxation = if elastic {
            "var x:Scalar; eq e:x==1; relax r on e nominal 1;"
        } else {
            ""
        };
        let rows = source(&format!(
            "package p {{def Root {{var T:Temperature; annotation objective T(minimize); {relaxation} }} }}"
        ));
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let mut workspace = CompilerWorkspace::new(input, WorkspaceLimits::default()).unwrap();
        workspace.publish_modeling(rows, names).unwrap();
        let admitted = workspace.admit_modeling(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings::default(),
            Limits::default(),
        );
        if elastic {
            assert!(
                admitted
                    .unwrap_err()
                    .to_string()
                    .contains("normalized dimensionless objective")
            );
        } else {
            assert_eq!(
                admitted.unwrap().case.objective().unwrap().quantity,
                temperature
            );
        }
    }
}

#[test]
fn kernel_flow_projection_preserves_ports_isolates_and_explicit_tear_policies() {
    use pse_structural::flowsheet::{Decision, Policy};
    let (mut workspace, _, _, root) = setup(
        "package p {def Unit {var x:Scalar; port inlet:Scalar=x; annotation connectivity inlet(1,0); port outlet:Scalar=x; annotation connectivity outlet(0,1);} def Root {child a=Unit(); child b=Unit(); child isolated=Unit(); connect a.outlet -> b.inlet; connect b.outlet -> a.inlet;}}",
    );
    let prepared = workspace
        .prepare_modeling_cancellable(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings::default(),
            Limits::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    let model = &prepared.model;
    let selection = ModelingFlowSelection {
        nodes: model.ports.values().map(|p| p.lineage.instance).collect(),
        connections: model
            .connections
            .keys()
            .map(|id| {
                (
                    *id,
                    Decision {
                        id: *id,
                        cost: 1.0,
                        policy: Policy::Free,
                    },
                )
            })
            .collect(),
    };
    let graph = prepared
        .flow_graph(&selection, &workspace.inputs.quantities)
        .unwrap();
    assert_eq!(graph.declaration().nodes.len(), 3);
    assert_eq!(graph.declaration().connections.len(), 2);
    assert!(graph.witness(&BTreeSet::new()).is_err());
    for id in model.connections.keys() {
        assert!(graph.witness(&BTreeSet::from([*id])).is_ok());
    }
    let mut partial = selection.clone();
    partial.connections.pop_first();
    assert!(
        prepared
            .flow_graph(&partial, &workspace.inputs.quantities)
            .is_err()
    );
    let mut cut = selection.clone();
    cut.nodes.remove(
        &model.ports[&model.connections.values().next().unwrap().from]
            .lineage
            .instance,
    );
    assert!(
        prepared
            .flow_graph(&cut, &workspace.inputs.quantities)
            .is_err()
    );
}
#[test]
fn nested_implicit_factorable_definition_exports_residual_exactly() {
    use pse_math::factorable::{FactorableRequest, Fidelity};
    let (mut workspace, _, _, root) = setup(
        "package p { def Root { var x: Scalar; implicit root { var y: Scalar; eq residual: y*y == x; annotation bounds y(0.5, 10); } realize policy on root using nested; eq pin: root.y == 2; } }",
    );
    let admitted = admit(&mut workspace, root);
    let inner = admitted.implicit.values().next().unwrap();
    let (key, definition) = inner.factorable_definition().unwrap();
    assert_eq!(key, inner.descriptor.spec().key());
    assert_eq!(
        definition.residual.input_count(),
        1 + inner.descriptor.spec().inputs.len()
    );
    let bounds = definition.bounds.as_ref().unwrap();
    assert!(bounds.lower[0].is_some() && bounds.upper[0].is_some());
    let cancel = Arc::new(AtomicBool::new(false));
    let f = fixture(
        &admitted,
        workspace.inputs.quantities.clone(),
        DerivativeOrder::Value,
        &cancel,
    )
    .unwrap();
    let values = CaseValues {
        scalars: admitted.inputs.iter().map(|id| (*id, 4.0)).collect(),
    };
    let relaxed = f
        .assembly
        .factorable_program(&values, &FactorableRequest::default(), 10_000, &cancel)
        .unwrap();
    assert!(relaxed.rows.iter().any(|r| r.fidelity == Fidelity::Relaxed));
    let request = FactorableRequest {
        implicit: BTreeMap::from([(key, definition)]),
        ..Default::default()
    };
    let program = f
        .assembly
        .factorable_program(&values, &request, 10_000, &cancel)
        .unwrap();
    assert_eq!(program.implicit.len(), 1);
    assert_eq!(program.implicit[0].fidelity, Fidelity::Exact);
    let unknown = &program.auxiliaries[program.implicit[0].unknowns[0]];
    assert_eq!((unknown.lower, unknown.upper), (0.5, 10.0));
    assert!(program.rows.iter().all(|r| r.fidelity == Fidelity::Exact));
    // A regime selection has no single residual; it stays a provider output.
    let (mut workspace, _, _, root) = setup(
        "package p { def Root { var target:Scalar; implicit roots select minimum((y-target)*(y-target),1e-8) {var y:Scalar; annotation start y(y); regime negative {eq root:y==-1; annotation start y(-1);} regime positive {eq root:y==1; annotation start y(1);} } realize r on roots using nested; eq selected:roots.y==target; } }",
    );
    let admitted = admit(&mut workspace, root);
    assert!(
        admitted
            .implicit
            .values()
            .all(|i| i.factorable_definition().is_none())
    );
}
