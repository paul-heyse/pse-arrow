// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source syntax and provenance through ordinary admitted native P3 execution.
#[path = "../../support/native_pipeline.rs"]
mod native_pipeline;
#[path = "../../support/physical_source.rs"]
mod physical_source;

use pse_authoring::{
    ParseBudget,
    document::{DocumentBundle, load_package_texts},
};
use pse_catalog::Snapshot;
use pse_ids::SemanticId;
use pse_relations::{
    columnar::RelationRow,
    generated::{
        authored,
        enums::{EquationSyntax, ExpressionSyntax, PredicateComparison, Sense},
        normalized,
    },
};
use pse_schema::Registry;
use serde_json::json;
use std::sync::Arc;

fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}

fn source_texts(
    registry: &Registry,
    expression: &str,
) -> std::collections::BTreeMap<String, String> {
    let mut texts = physical_source::physical_texts(registry);
    texts.insert("templates/model.yaml".into(), json!({
        "templates": [{"id":id(91),"name":"model","version":"1.0.0","kind":"unit","doc":""}],
        "template_symbols": [{"id":id(92),"template_id":id(91),"name":"x","role":"variable","quantity_type_id":id(31),"indexed_by":[],"doc":""}],
        "template_symbol_contracts": [{"symbol_decl_id":id(92),"solver_type":"continuous","semantic_role":"state"}],
        "template_equations": [{"id":id(94),"template_id":id(91),"name":"equation","indexed_by":[],"expression":expression,"sense":"eq","doc":""}]
    }).to_string());
    texts
}

fn source(registry: &Registry, expression: &str) -> DocumentBundle {
    load_package_texts(
        source_texts(registry, expression),
        registry,
        ParseBudget::default(),
    )
    .unwrap()
}

fn rows<T: RelationRow>(snapshot: &Snapshot, registry: &Registry) -> Vec<T> {
    let spec = T::relation(registry).unwrap();
    T::rows(
        snapshot
            .relation(spec.key.namespace.as_str(), spec.key.name)
            .unwrap()
            .checked(),
    )
    .unwrap()
}

async fn compile(expression: &str) -> (native_pipeline::Fixture, Arc<Snapshot>) {
    let mut fixture = native_pipeline::Fixture::new();
    let input = source(&fixture.registry, expression);
    let committed = fixture.commit(vec![input]).await;
    let report = fixture
        .run(committed, "P3")
        .await
        .unwrap_or_else(|error| panic!("P3 failed: {error}"));
    let snapshot = Arc::clone(&report.stages.last().unwrap().snapshot);
    (fixture, snapshot)
}

#[tokio::test]
async fn conditional_equations_keep_declared_senses_branches_and_source_key() {
    let (fixture, output) = compile("if x > 0 then x <= 3 else x >= -2").await;
    let sources = rows::<normalized::expression_sources::Row>(&output, &fixture.registry);
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].syntax, ExpressionSyntax::Equation);
    assert_eq!(
        sources[0].source_relation_id,
        authored::template_equations::RELATION_ID
    );
    assert_eq!(sources[0].source_key.len(), 1);
    assert_eq!(sources[0].source_key[0].semantic_id, Some(id(94)));
    let equations = rows::<normalized::equation_nodes::Row>(&output, &fixture.registry);
    assert_eq!(equations.len(), 3);
    assert!(equations.iter().any(|row| row.sense == Some(Sense::Le)));
    assert!(equations.iter().any(|row| row.sense == Some(Sense::Ge)));
    let conditional = equations
        .iter()
        .find(|row| row.kind == EquationSyntax::Conditional)
        .unwrap();
    assert!(conditional.guard_predicate.is_some());
    assert_ne!(conditional.then_equation, conditional.else_equation);
    let predicates = rows::<normalized::predicate_nodes::Row>(&output, &fixture.registry);
    assert_eq!(predicates.len(), 2);
    let guard = predicates
        .iter()
        .find(|row| Some(row.predicate_id) == conditional.guard_predicate)
        .unwrap();
    assert_eq!(guard.comparison, Some(PredicateComparison::Gt));
    let otherwise = predicates
        .iter()
        .find(|row| row.predicate_id != guard.predicate_id)
        .unwrap();
    assert_eq!(
        otherwise.kind,
        pse_relations::generated::enums::PredicateKind::Not
    );
    assert_eq!(otherwise.left_predicate, Some(guard.predicate_id));
}

#[tokio::test]
async fn lexical_binding_leaves_only_the_outer_symbol_reference() {
    let (fixture, output) = compile("x == (x + 1 where x = 3)").await;
    let references = rows::<normalized::template_expr_symbol_refs::Row>(&output, &fixture.registry);
    assert_eq!(references.len(), 1);
    assert_eq!(references[0].symbol_id, Some(id(92)));
    assert!(references[0].bound_index_id.is_none());
}

#[tokio::test]
async fn cached_configuration_ast_binds_each_parent_and_preserves_full_width_literals() {
    let mut fixture = native_pipeline::Fixture::new();
    let mut texts = source_texts(&fixture.registry, "x == 0");
    let mut model: serde_json::Value =
        serde_json::from_str(&texts["templates/model.yaml"]).unwrap();
    model["templates"].as_array_mut().unwrap().push(json!({
        "id": id(95), "name": "child", "version": "1.0.0", "kind": "unit", "doc": ""
    }));
    let logical = fixture.registry.logical_type("u64").unwrap().id;
    model["template_params"] = json!([
        {"template_id":id(91),"name":"count","logical_type_id":logical,"required":true,"doc":""},
        {"template_id":id(95),"name":"count","logical_type_id":logical,"required":true,"doc":""},
        {"template_id":id(95),"name":"limit","logical_type_id":logical,"required":true,"doc":""}
    ]);
    model["template_submodels"] = json!([{
        "template_id":id(91), "name":"child", "child_template_id":id(95),
        "bindings":[
            {"child_param":"count","value":" (parent . count) "},
            {"child_param":"limit","value":"18446744073709551615"}
        ]
    }]);
    texts.insert("templates/model.yaml".into(), model.to_string());
    let expected = [(id(96), 9_007_199_254_740_993_u64), (id(97), u64::MAX)];
    texts.insert(
        "instances/model.yaml".into(),
        json!({"instances": expected.map(|(parent, value)| json!({
            "id":parent,"template_id":id(91),"name":format!("parent_{value}"),
            "param_values":[{"name":"count","value":value.to_string()}],"feature_values":[],"doc":""
        }))})
        .to_string(),
    );
    let source = load_package_texts(texts, &fixture.registry, ParseBudget::default()).unwrap();
    let committed = fixture.commit(vec![source]).await;
    let report = fixture.run(committed, "P3").await.unwrap();
    let output = &report.stages.last().unwrap().snapshot;
    let instances = rows::<normalized::instance_bindings::Row>(output, &fixture.registry);
    let values = rows::<normalized::config_values::Row>(output, &fixture.registry);
    for (parent, expected) in expected {
        let children = instances
            .iter()
            .filter(|row| row.parent_instance_id == Some(parent))
            .collect::<Vec<_>>();
        assert_eq!(children.len(), 1);
        for (name, expected) in [("count", expected), ("limit", u64::MAX)] {
            let actual = values
                .iter()
                .filter(|row| row.owner_id == children[0].instance_id && row.name == name)
                .collect::<Vec<_>>();
            assert_eq!(actual.len(), 1);
            assert_eq!(
                actual[0].value.unsigned.as_ref().map(|arm| arm.value),
                Some(expected)
            );
            assert_eq!(
                actual[0].source_relation_id,
                authored::template_submodels::RELATION_ID
            );
        }
    }
}

#[tokio::test]
async fn indexed_reads_keep_the_declared_order_without_inventing_binders() {
    let mut fixture = native_pipeline::Fixture::new();
    let mut texts = source_texts(&fixture.registry, "x[1, 2] == x[2, 1]");
    let mut model: serde_json::Value =
        serde_json::from_str(&texts["templates/model.yaml"]).unwrap();
    model["template_symbols"][0]["indexed_by"] = json!(["i", "j"]);
    model["template_domains"] = json!([
        {"template_id":id(91),"name":"i","kind":"custom","continuous":false},
        {"template_id":id(91),"name":"j","kind":"custom","continuous":false}
    ]);
    texts.insert("templates/model.yaml".into(), model.to_string());
    let source = load_package_texts(texts, &fixture.registry, ParseBudget::default()).unwrap();
    let committed = fixture.commit(vec![source]).await;
    let report = fixture
        .run(committed, "P3")
        .await
        .unwrap_or_else(|error| panic!("P3 failed: {error}"));
    let output = &report.stages.last().unwrap().snapshot;
    let gathers = rows::<normalized::template_expr_gathers::Row>(output, &fixture.registry);
    assert_eq!(gathers.len(), 2);
    assert!(gathers.iter().all(|row| row.coordinate_map.is_none()
        && row.gather_state == pse_relations::generated::enums::GatherState::Pending));
    let first = gathers[0].index_nodes.as_ref().unwrap();
    let second = gathers[1].index_nodes.as_ref().unwrap();
    assert_eq!(
        first.iter().rev().collect::<Vec<_>>(),
        second.iter().collect::<Vec<_>>()
    );
    assert!(
        rows::<normalized::expression_index_bindings::Row>(output, &fixture.registry).is_empty()
    );
    let constants = rows::<normalized::template_expr_int_constants::Row>(output, &fixture.registry);
    assert_eq!(
        constants
            .iter()
            .map(|row| row.value)
            .collect::<std::collections::BTreeSet<_>>(),
        [1, 2].into_iter().collect()
    );
}

#[tokio::test]
async fn distinct_compound_literals_use_each_actual_component_unit() {
    let mut fixture = native_pipeline::Fixture::new();
    let mut texts = source_texts(&fixture.registry, "x == 2{cm/s} + 4{m/s}");
    let physical_text = texts["materials/physical.yaml"]
        .lines()
        .filter(|line| !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let mut physical: serde_json::Value = serde_json::from_str(&physical_text).unwrap();
    let mut centimetre = physical["units"][0].clone();
    centimetre["unit_id"] = json!(id(40));
    centimetre["symbol"] = json!("cm");
    centimetre["name"] = json!("centimetre");
    centimetre["scale_to_canonical"] = json!(0.01);
    physical["units"].as_array_mut().unwrap().push(centimetre);
    texts.insert("materials/physical.yaml".into(), physical.to_string());
    let source = load_package_texts(texts, &fixture.registry, ParseBudget::default()).unwrap();
    let committed = fixture.commit(vec![source]).await;
    let report = fixture
        .run(committed, "P3")
        .await
        .unwrap_or_else(|error| panic!("P3 failed: {error}"));
    let output = &report.stages.last().unwrap().snapshot;
    let mut values =
        rows::<normalized::template_expr_float_constants::Row>(output, &fixture.registry)
            .iter()
            .map(|row| row.value)
            .collect::<Vec<_>>();
    values.sort_by(f64::total_cmp);
    assert_eq!(values, [0.02, 4.0]);
}

#[tokio::test]
async fn cold_stage_reopen_retains_selected_policy_and_rejects_missing_invocation() {
    use pse_catalog::store::membership::AdmissionContext;
    use pse_compiler::{BoundInput, PolicySet, driver::PipelineRequest, passes::PolicyBinding};
    use pse_ids::CancellationToken;
    use std::collections::BTreeMap;

    let mut fixture = native_pipeline::Fixture::new();
    let input = source(&fixture.registry, "x == 7");
    let committed = fixture.commit(vec![input]).await;
    let declaration = fixture
        .registry
        .relation("reference.quantity_types")
        .unwrap();
    let mut owners = vec![committed.clone()];
    let owner = loop {
        let owner = owners
            .pop()
            .expect("actual policy owner in committed input graph");
        if owner
            .relations()
            .values()
            .any(|relation| relation.contract().canonical.relation_id == declaration.id)
        {
            break owner;
        }
        owners.extend(owner.parents().values().cloned());
    };
    let policy = BoundInput::bind(owner, declaration.key, &fixture.registry).unwrap();
    let policies = PolicySet(BTreeMap::from([(
        "physical_scope".to_owned(),
        PolicyBinding {
            policy_id: id(31),
            input: policy,
        },
    )]));
    let cancel = CancellationToken::new();
    let report = fixture
        .driver
        .run(
            PipelineRequest {
                through: "P3".into(),
                snapshot: committed,
                policies,
                reuse: false,
            },
            &cancel,
        )
        .await
        .unwrap();
    let output = &report.stages.last().unwrap().snapshot;
    let reader = fixture.reader();
    let restored = reader
        .read_pinned_manifest(output.manifest_ref(), &cancel)
        .await
        .unwrap();
    let invocation = restored.invocation().unwrap();
    assert_eq!(invocation.policies["physical_scope"].policy_id, id(31));
    assert_eq!(invocation.policies.len(), 1);
    assert_eq!(invocation.engine, output.invocation().unwrap().engine);
    for (port, relation) in restored.relations() {
        assert_eq!(relation.batch(), output.relations()[port].batch());
    }
    let missing = AdmissionContext {
        traversal: Arc::default(),
        parents: restored.parents().clone(),
        stage_pass: restored.stage_pass(),
        invocation: None,
    };
    assert!(
        reader
            .read_manifest(restored.manifest_ref(), &missing, &cancel)
            .await
            .is_err(),
        "an explicit context cannot omit the stored invocation and replay defaults"
    );
}
