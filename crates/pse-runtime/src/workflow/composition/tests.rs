// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::workflow::{
    ModelBuilder,
    tests::{id, physical, runtime},
};

fn fixture() -> ModelBuilder {
    let physical = physical();
    let q = pse_quantity::standard::ids::quantity("neutral").as_id();
    let row = serde_json::from_value(serde_json::json!({
        "model_id":id(1),"name":"composition","definitions":[],"domains":[],"groups":[],
        "cases":[{"case_id":id(2),"name":"selected","variables":[],"parameters":[],"instances":[],"rows":[],"objective":null,"values":[]}]
    })).unwrap();
    let mut builder = ModelBuilder::from_declaration(runtime(), row, physical);
    builder.sources.composition = serde_json::from_value(serde_json::json!({
        "roots":[{"model_id":id(1),"root_instance_id":id(3)}],
        "templates":[{"template_id":id(4),"package_id":id(5),"name":"unit","version":"1","kind":"unit","default_initializer_template_id":null,"default_scaler_template_id":null,"idaes_class":null,"doc":"source"}],
        "instances":[{"instance_id":id(3),"parent_instance_id":null,"template_id":id(4),"name":"unit","param_values":[],"feature_values":[],"property_package_id":null,"reaction_package_id":null,"doc":"source"}],
        "symbols":[
            {"template_id":id(4),"symbol_decl_id":id(6),"name":"x","role":"variable","quantity_type_id":q,"indexed_by":[],"default_lower":null,"default_upper":null,"default_initial":1.0,"reference_to":null,"wrt_domain":null,"guard_id":null,"idaes_name":null,"doc":"x"},
            {"template_id":id(4),"symbol_decl_id":id(7),"name":"a","role":"parameter","quantity_type_id":q,"indexed_by":[],"default_lower":null,"default_upper":null,"default_initial":2.0,"reference_to":null,"wrt_domain":null,"guard_id":null,"idaes_name":null,"doc":"a"}],
        "equations":[{"template_id":id(4),"equation_decl_id":id(8),"name":"closure","indexed_by":[],"filter":null,"expression":"x == a","sense":"eq","guard_id":null,"idaes_name":null,"doc":"closure"}]
    })).unwrap();
    builder
}

#[test]
fn composition_unit_template_lowering_is_shared_and_reorder_stable() {
    let a = fixture().freeze().unwrap();
    let mut b = a.edit();
    b.sources.composition.symbols.reverse();
    let b = b.freeze().unwrap();
    assert_eq!(a.identity(), b.identity());
    assert!(a.declaration().definitions.is_empty());
    assert_eq!(a.projection().definitions.len(), 1);
    let case = &a.0.cases[&id(2)];
    assert_eq!(case.flows[&id(3)].nodes.len(), 1);
    assert_eq!(case.cases[&id(2)].structure.parameters().len(), 1);
    let mut workspace =
        pse_compiler::workspace::CompilerWorkspace::new(case.as_ref().clone(), Default::default())
            .unwrap();
    let prepared = workspace.admit_selected_case(id(2)).unwrap();
    assert_eq!(prepared.structure().rows().len(), 1);
}

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn selected_tear_causal_map_converges_through_public_strategy() {
    use pse_backend_native::solve::*;
    let mut builder = fixture();
    let c = &mut builder.sources.composition;
    c.equations[0].expression = "x/2+a == 0".into();
    for (name, direction) in [("out", "outlet"), ("in", "inlet")] {
        c.ports.push(serde_json::from_value(serde_json::json!({"template_id":id(4),"name":name,"kind":"material","direction":direction,"bound_to":"x","guard_id":null,"doc":""})).unwrap());
    }
    c.connection_rules.push(
        serde_json::from_value(
            serde_json::json!({"rule_template_id":id(60),"expansion":"equality"}),
        )
        .unwrap(),
    );
    c.connections.push(serde_json::from_value(serde_json::json!({"connection_id":id(61),"from_port_id":port_id(id(3),"out"),"to_port_id":port_id(id(3),"in"),"rule_template_id":id(60),"tear_cost":2.0,"tear_policy":"mandatory","tear_group":null,"doc":""})).unwrap());
    let revision = builder.freeze().unwrap();
    let flow = revision.prepare_flow(id(2), id(3)).await.unwrap();
    let witness = revision
        .0
        .runtime
        .native()
        .select_tears(
            flow,
            crate::math::flows::TearMethod::UnweightedHeuristic,
            Controls::default(),
        )
        .unwrap()
        .finish()
        .await
        .unwrap()
        .selected
        .unwrap();
    assert_eq!(witness.cost, 2.0);
    let request = crate::workflow::RecycleRequest {
        case: id(2),
        flow: id(3),
        tears: witness.decisions,
        units: vec![crate::workflow::CausalUnitRequest {
            node: id(3),
            case: id(2),
            inputs: BTreeMap::from([(
                symbol_id(port_id(id(3), "in"), symbol_id(id(3), id(6), &[]), &[]),
                symbol_id(id(3), id(6), &[]),
            )]),
            outputs: BTreeMap::from([(
                symbol_id(port_id(id(3), "out"), symbol_id(id(3), id(6), &[]), &[]),
                symbol_id(id(3), id(8), &[]),
            )]),
        }],
        anderson: 1,
        damping: 1.0,
    };
    let profile = crate::math::solves::SolverProfile {
        intent: SolveIntent::Root,
        selection: SolverSelection::Auto,
        controls: Controls::default(),
        presolve: Default::default(),
        numerics: Default::default(),
        convexity: Default::default(),
        backend: crate::math::solves::BackendSettings::Default,
    };
    let prepared = revision
        .prepare_recycle(
            request,
            profile,
            crate::workflow::tests::compiler_profile(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert_eq!(prepared.order().unwrap(), vec![id(3)]);
    let result = prepared.start().unwrap().finish().await.unwrap();
    assert_eq!(
        result.report.qualification,
        Qualification::Feasible,
        "{:?}",
        result.report
    );
    assert!((result.report.candidate.as_ref().unwrap().primal[0] - 4.0).abs() < 1e-6);
    assert!(
        result
            .report
            .metrics
            .contains_key("KINGetNumNonlinSolvIters")
    );
}

#[test]
fn composition_unit_case_specs_are_resolved_with_selected_scaling() {
    let mut a = fixture();
    let spec: case_specs::Row = serde_json::from_value(serde_json::json!({
        "spec_id":id(20),"case_id":id(2),"target":"unit.x","treatment":"fixed","value":3.0,"unit_id":null,"initial":null,"lower":null,"upper":null,"scaling_factor":null,"priority":0,"source_span":{"document_id":id(21),"start":0,"end":6}
    })).unwrap();
    a.sources.composition.case_specs.push(spec);
    let a = a.freeze().unwrap();
    assert!(a.0.cases[&id(2)].cases[&id(2)].structure.variables()[0].fixed);
    assert_eq!(a.0.cases[&id(2)].values[&symbol_id(id(3), id(6), &[])], 3.0);
    let mut edit = a.edit();
    edit.sources.composition.case_specs[0].scaling_factor = Some(2.0);
    let scaled = edit.freeze().unwrap();
    assert_eq!(scaled.0.resolved_sources.numerics.len(), 1);
    assert_eq!(
        scaled.0.resolved_sources.numerics[0].scaling_factor,
        Some(2.0)
    );
    // Unrelated cases are explicitly nonexecuting, even when their meaning is unsupported.
    let mut edit = a.edit();
    edit.sources.composition.case_specs[0].case_id = id(99);
    edit.sources.composition.case_specs[0].scaling_factor = Some(2.0);
    let admitted = edit.freeze().unwrap();
    assert!(
        admitted
            .admission()
            .iter()
            .any(|r| r.relation == "authored.case_specs" && r.nonexecuting == 1)
    );
}

#[test]
fn composition_unit_guards_select_formulations_and_refuse_unknown_values() {
    let mut a = fixture();
    a.sources.composition.features.push(serde_json::from_value(serde_json::json!({"template_id":id(4),"name":"active","kind":"bool","enum_id":null,"default":"true","inherit_from":null,"doc":""})).unwrap());
    a.sources.composition.guards.push(serde_json::from_value(serde_json::json!({"guard_id":id(30),"template_id":id(4),"predicate":"active","doc":""})).unwrap());
    a.sources.composition.equations[0].guard_id = Some(id(30));
    let a = a.freeze().unwrap();
    assert_eq!(a.projection().definitions.len(), 1);
    let mut b = a.edit();
    b.sources.composition.features[0].default = Some("false".into());
    assert!(b.freeze().unwrap().projection().definitions.is_empty());
    let mut b = a.edit();
    b.sources.composition.features[0].default = None;
    assert!(matches!(b.freeze(), Err(WorkflowError::Boundary(_))));
}

#[test]
fn composition_unit_integer_parameters_preserve_exact_declared_values() {
    let mut builder = fixture();
    let registry = pse_schema::registry().unwrap();
    let logical = registry
        .logical_types()
        .iter()
        .find(|t| t.arrow_storage == "\"Int64\"")
        .unwrap();
    builder.sources.composition.parameters.push(serde_json::from_value(serde_json::json!({"template_id":id(4),"name":"factor","logical_type_id":logical.id,"enum_id":null,"required":true,"default":"3","domain_spec":null,"doc":""})).unwrap());
    builder.sources.composition.equations[0].expression = "x == factor * a".into();
    let revision = builder.freeze().unwrap();
    let mut changed = revision.edit();
    changed.sources.composition.parameters[0].default = Some("9007199254740993".into());
    assert!(
        changed
            .freeze()
            .unwrap_err()
            .to_string()
            .contains("exact declared representation")
    );
    let mut changed = revision.edit();
    changed.sources.composition.parameters[0].default = Some("4".into());
    assert_ne!(revision.identity(), changed.freeze().unwrap().identity());
}

#[test]
fn composition_unit_scalar_law_owns_contributions_and_heater_variant() {
    let mut draft = fixture();
    let q = draft.sources.composition.symbols[0].quantity_type_id;
    draft.sources.composition.equations.clear();
    let registry = pse_schema::registry().unwrap();
    let enumeration = registry
        .enums()
        .iter()
        .find(|e| e.name == "MaterialBalanceType")
        .unwrap();
    let member = enumeration
        .members
        .iter()
        .find(|m| m.name != "none")
        .unwrap();
    draft.sources.composition.laws.push(serde_json::from_value(serde_json::json!({"template_id":id(4),"law_instance_decl_id":id(40),"law_template_id":id(41),"scope":"unit","options":[{"key":"tolerance","value":"1e-8"}],"guard_id":null})).unwrap());
    draft.sources.composition.law_contracts.push(serde_json::from_value(serde_json::json!({"law_instance_decl_id":id(40),"indexed_by":[],"quantity_type_id":q,"balance_enum_id":enumeration.id,"default_balance":null,"coordinates":{"member":null,"phase":null}})).unwrap());
    draft.sources.composition.law_bindings.push(serde_json::from_value(serde_json::json!({"law_template_id":id(41),"balance_enum_id":enumeration.id,"balance_member":member.name,"family":"material","source_family":"material","subject_projection":"identity","expansion":"conservation","subject_kind":"total"})).unwrap());
    for (n, expression, orientation) in [(42, "a", "into_scope"), (43, "x", "out_of_scope")] {
        draft.sources.composition.contributions.push(serde_json::from_value(serde_json::json!({"template_id":id(4),"contribution_decl_id":id(n),"name":expression,"law_family":"material","expression":expression,"orientation":orientation,"scope":"unit","guard_id":null,"doc":""})).unwrap());
    }
    let original = draft.freeze().unwrap();
    assert_eq!(original.resolved_balances().len(), 1);
    assert!(original.source_declarations().balances.is_empty());
    assert_eq!(original.resolved_balances()[0].terms.len(), 2);
    let mut multiple = original.edit();
    let mut second = multiple.row.cases[0].clone();
    second.case_id = id(45);
    multiple.row.cases.push(second);
    let multiple = multiple.freeze().unwrap();
    assert_eq!(multiple.resolved_balances().len(), 2);
    assert_ne!(
        multiple.resolved_balances()[0].balance_id,
        multiple.resolved_balances()[1].balance_id
    );
    let mut tolerance = original.edit();
    tolerance.sources.composition.laws[0].options[0].value = "1e-7".into();
    let tolerance = tolerance.freeze().unwrap();
    assert_ne!(original.identity(), tolerance.identity());
    assert_eq!(
        original.case_identity(id(2)),
        tolerance.case_identity(id(2))
    );
    // An ordinary additional input uses the existing contribution vocabulary.
    let mut heater = original.edit();
    let mut heat = heater.sources.composition.contributions[0].clone();
    heat.contribution_decl_id = id(44);
    heat.name = "additional-input".into();
    heat.expression = "2 * a".into();
    heater.sources.composition.contributions.push(heat);
    let heater = heater.freeze().unwrap();
    assert_eq!(heater.resolved_balances()[0].terms.len(), 3);
    assert_ne!(original.identity(), heater.identity());
    let mut duplicate = heater.edit();
    let option = duplicate.sources.composition.laws[0].options[0].clone();
    duplicate.sources.composition.laws[0].options.push(option);
    assert!(duplicate.freeze().is_err());
}

#[test]
fn composition_unit_nested_selected_rows_cannot_fall_through() {
    use datafusion::arrow::{
        array::{ArrayRef, FixedSizeBinaryArray, ListArray},
        datatypes::{DataType, Field},
    };
    use std::sync::Arc;
    let ids =
        FixedSizeBinaryArray::try_from_iter([id(3).as_bytes().as_slice()].into_iter()).unwrap();
    let child = pse_schema::model::FieldContract::id().with_name("item");
    let field = pse_schema::model::FieldContract::from_field(Field::new(
        "owners",
        DataType::List(Arc::new(child.field().clone())),
        false,
    ));
    let ids: ArrayRef = Arc::new(ids);
    let array = ListArray::new(
        Arc::new(child.field().clone()),
        datafusion::arrow::buffer::OffsetBuffer::new(vec![0, 1].into()),
        ids,
        None,
    );
    let mut found = BTreeSet::new();
    selected_ids(&field, &array, 0, &BTreeSet::from([id(3)]), &mut found).unwrap();
    assert_eq!(found, BTreeSet::from([id(3)]));
}

#[test]
fn composition_unit_finite_axes_preserve_physical_shape_and_binding() {
    let mut builder = fixture();
    let mut physical = physical();
    let scalar = physical
        .quantities
        .quantity_type(pse_quantity::standard::ids::quantity("neutral"))
        .unwrap();
    let mut shaped = scalar.clone();
    shaped.id = id(100).into();
    shaped.key.shape = vec![pse_quantity::DomainKind::PortSet];
    let shape_id = physical
        .quantities
        .quantity_types()
        .find(|q| q.key == shaped.key)
        .unwrap()
        .id
        .as_id();
    physical.key =
        pse_compiler::workspace::physical_identity(&physical.quantities, &physical.preconditions);
    let mut target = ModelBuilder::from_declaration(runtime(), builder.row.clone(), physical);
    let c = &mut builder.sources.composition;
    for symbol in &mut c.symbols {
        symbol.quantity_type_id = shape_id;
        symbol.indexed_by = vec!["j".into()];
    }
    c.equations[0].indexed_by = vec!["j".into()];
    c.equations[0].expression = "x[j] == a[j]".into();
    c.template_domains.push(serde_json::from_value(serde_json::json!({"template_id":id(4),"name":"j","kind":"port_set","continuous":false,"members_from":null,"bounds":null,"unit_id":null})).unwrap());
    c.instance_domains.push(
        serde_json::from_value(
            serde_json::json!({"instance_id":id(3),"domain_name":"j","domain_id":id(101)}),
        )
        .unwrap(),
    );
    c.domains.push(serde_json::from_value(serde_json::json!({"domain_id":id(101),"owner_entity_id":id(3),"kind":"port_set","continuous":false,"unit_id":null,"parent_domain_id":null,"doc":""})).unwrap());
    for (ordinal, label) in ["feed", "recycle"].into_iter().enumerate() {
        c.members.push(serde_json::from_value(serde_json::json!({"domain_id":id(101),"member_id":id(102+ordinal as u8),"ordinal":ordinal,"label":label,"coordinate":null,"ref_entity_id":null})).unwrap());
    }
    target.sources = builder.sources;
    let revision = target.freeze().unwrap();
    assert_eq!(revision.scalar_bindings().len(), 4);
    assert_eq!(revision.projection().cases[0].rows.len(), 2);
    let mut changed = revision.edit();
    changed.sources.composition.template_domains[0].kind =
        pse_relations::generated::enums::DomainKind::Species;
    assert!(changed.freeze().is_err());
}

#[test]
fn composition_unit_documents_match_typed_selection_and_prose_is_neutral() {
    let mut builder = fixture();
    builder.sources.composition.templates[0].name = "reusable_unit".into();
    builder.sources.composition.templates[0].package_id =
        SemanticId::parse_hex("01991d6a13a070008000000000000001").unwrap();
    let rt = runtime();
    let c = builder.sources.composition.clone();
    let row = builder.row.clone();
    let typed = builder.freeze().unwrap();
    let texts=BTreeMap::from([
        ("package.toml".into(),include_str!("../../../../../tests/fixtures/packages/minimal_explicit/package.toml").into()),
        ("computation_models/model.yaml".into(),serde_json::to_string(&serde_json::json!({"computation_models":[row],"model_compositions":c.roots})).unwrap()),
        ("templates/unit.yaml".into(),serde_json::to_string(&serde_json::json!({"templates":c.templates,"template_symbols":c.symbols,"template_equations":c.equations})).unwrap()),
        ("instances/unit.yaml".into(),serde_json::to_string(&serde_json::json!({"instances":c.instances})).unwrap()),
    ]);
    let cancel = pse_columnar::CancellationToken::new();
    let pool = rt.shared.pool();
    let bundle = crate::authoring_driver::document::load_package_texts_owned(
        &texts,
        &rt.registry,
        pse_authoring::ParseBudget::default(),
        &pool,
        &cancel,
    )
    .unwrap();
    let documents = crate::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
        vec![bundle],
        &pool,
        &cancel,
    )
    .unwrap();
    let document = rt
        .models_from_documents(&documents, physical())
        .unwrap()
        .remove(0)
        .freeze()
        .unwrap();
    assert_eq!(typed.identity(), document.identity());
    assert_eq!(typed.case_identity(id(2)), document.case_identity(id(2)));
    let mut changed = document.edit();
    changed.sources.composition.templates[0].doc = "editorial correction".into();
    changed.sources.composition.symbols[0].doc = "clarified meaning".into();
    assert_eq!(changed.freeze().unwrap().identity(), typed.identity());
}

#[test]
fn composition_unit_connections_own_real_ports_and_refuse_multiple_inlets_and_tears() {
    let mut builder = fixture();
    let c = &mut builder.sources.composition;
    for (n, name) in [(50, "middle"), (51, "sink")] {
        let mut child = c.instances[0].clone();
        child.instance_id = id(n);
        child.parent_instance_id = Some(id(3));
        child.name = name.into();
        c.instances.push(child);
    }
    for (name, direction) in [("out", "outlet"), ("in", "inlet")] {
        c.ports.push(serde_json::from_value(serde_json::json!({"template_id":id(4),"name":name,"kind":"signal","direction":direction,"bound_to":"x","guard_id":null,"doc":""})).unwrap());
    }
    c.connection_rules.push(
        serde_json::from_value(
            serde_json::json!({"rule_template_id":id(60),"expansion":"equality"}),
        )
        .unwrap(),
    );
    for (n, from, to) in [(61, 3, 50), (62, 3, 51)] {
        c.connections.push(serde_json::from_value(serde_json::json!({"connection_id":id(n),"from_port_id":port_id(id(from),"out"),"to_port_id":port_id(id(to),"in"),"rule_template_id":id(60),"tear_cost":null,"doc":""})).unwrap());
    }
    let revision = builder.freeze().unwrap();
    assert_eq!(revision.0.cases[&id(2)].flows[&id(3)].nodes.len(), 3);
    assert!(
        revision.0.cases[&id(2)].flows[&id(3)]
            .connections
            .is_empty()
    ); // Signals remain mathematical assignments.
    let mut duplicate = revision.edit();
    duplicate.sources.composition.connections[1].to_port_id = port_id(id(50), "in");
    assert!(
        duplicate
            .freeze()
            .unwrap_err()
            .to_string()
            .contains("multiple assignments")
    );
    let mut tear = revision.edit();
    tear.sources.composition.connections[0].tear_cost = Some(2.0);
    assert_eq!(
        tear.freeze()
            .unwrap()
            .source_declarations()
            .composition
            .connections[0]
            .tear_cost,
        Some(2.0)
    );
    let mut physical = revision.edit();
    for port in &mut physical.sources.composition.ports {
        port.kind = pse_relations::generated::enums::PortKind::Material;
    }
    let revision = physical.freeze().unwrap();
    assert_eq!(revision.0.cases[&id(2)].flows[&id(3)].connections.len(), 2);
    let mut extensive = revision.edit();
    let registry = crate::workflow::tests::physical().quantities;
    let quantity = registry
        .quantity_types()
        .find(|q| {
            q.key.shape.is_empty()
                && q.key.scale_kind == pse_quantity::ScaleKind::Point
                && registry.kind(q.key.kind).unwrap().extensive
        })
        .unwrap()
        .id
        .as_id();
    for symbol in &mut extensive.sources.composition.symbols {
        symbol.quantity_type_id = quantity;
    }
    assert!(
        extensive
            .freeze()
            .unwrap_err()
            .to_string()
            .contains("fan-out")
    );
}
