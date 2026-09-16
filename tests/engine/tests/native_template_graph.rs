// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Target template outcomes through actual source admission and P3-P10 execution.
#![allow(
    clippy::unwrap_used,
    reason = "explicit source and expected graph assertions"
)]
#[path = "../../support/native_pipeline.rs"]
mod native_pipeline;
#[path = "../../support/physical_source.rs"]
mod physical_source;
use pse_authoring::{
    ParseBudget,
    document::{DocumentBundle, load_package_texts},
};
use pse_ids::SemanticId;
use pse_relations::generated::{compiled, enums::Sense, inferred};
use pse_schema::Registry;
use std::collections::BTreeSet;
fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}

fn source(registry: &Registry, second: bool) -> DocumentBundle {
    let mut texts = physical_source::physical_texts(registry);
    let templates = format!(
        "templates:\n- id: '{}'\n  name: alpha\n  version: '1'\n  kind: unit\n  doc: ''\n- id: '{}'\n  name: beta\n  version: '1'\n  kind: unit\n  doc: ''\ntemplate_symbols:\n- id: '{}'\n  template_id: '{}'\n  name: x\n  role: variable\n  quantity_type_id: '{}'\n  indexed_by: []\n  doc: ''\n- id: '{}'\n  template_id: '{}'\n  name: y\n  role: expression\n  quantity_type_id: '{}'\n  indexed_by: []\n  doc: ''\ntemplate_symbol_contracts:\n- symbol_decl_id: '{}'\n  solver_type: continuous\n  semantic_role: state\n- symbol_decl_id: '{}'\n  solver_type: continuous\n  semantic_role: reporting_only\ntemplate_symbol_expressions:\n- symbol_decl_id: '{}'\n  template_id: '{}'\n  expression: '2 + 3'\ntemplate_equations:\n- id: '{}'\n  template_id: '{}'\n  name: balance\n  indexed_by: []\n  expression: 'x == 7'\n  sense: eq\n  doc: ''\n",
        id(61),
        id(62),
        id(63),
        id(61),
        id(31),
        id(64),
        id(62),
        id(31),
        id(63),
        id(64),
        id(64),
        id(62),
        id(65),
        id(61)
    );
    texts.insert("templates/models.yaml".to_owned(), templates);
    let mut instances = format!(
        "instances:\n- id: '{}'\n  template_id: '{}'\n  name: first\n  param_values: []\n  feature_values: []\n  doc: ''\n",
        id(71),
        id(61)
    );
    if second {
        use std::fmt::Write;
        writeln!(instances, "- id: '{}'\n  template_id: '{}'\n  name: second\n  param_values: []\n  feature_values: []\n  doc: ''", id(72), id(62)).unwrap();
    }
    texts.insert("instances/models.yaml".to_owned(), instances);
    load_package_texts(texts, registry, ParseBudget::default()).unwrap()
}

#[tokio::test]
async fn two_templates_produce_declared_symbols_expressions_and_equations() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("pse_compiler=info")
        .with_test_writer()
        .try_init();
    let mut fixture = native_pipeline::Fixture::new();
    let input = source(&fixture.registry, true);
    let committed = fixture.commit(vec![input]).await;
    let report = fixture
        .run(committed, "P10")
        .await
        .unwrap_or_else(|error| panic!("source-to-P10 failed: {error}"));
    assert_eq!(
        report
            .stages
            .iter()
            .map(|stage| stage.pass.as_str())
            .collect::<Vec<_>>(),
        ["P3", "P4", "P5", "P6", "P7", "P8", "P9", "P10"]
    );
    let result = &report
        .stages
        .iter()
        .find(|stage| stage.pass == "P7")
        .unwrap()
        .snapshot;
    let symbols = compiled::symbols::View::from_checked(
        result.relation("compiled", "symbols").unwrap().checked(),
    )
    .unwrap()
    .rows()
    .unwrap();
    assert_eq!(symbols.len(), 2);
    assert_eq!(
        symbols
            .iter()
            .map(|row| row.ordinal)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([0, 1])
    );
    let expressions = compiled::symbol_expressions::View::from_checked(
        result
            .relation("compiled", "symbol_expressions")
            .unwrap()
            .checked(),
    )
    .unwrap()
    .rows()
    .unwrap();
    assert_eq!(expressions.len(), 1);
    let roots = compiled::expression_roots::View::from_checked(
        result
            .relation("compiled", "expression_roots")
            .unwrap()
            .checked(),
    )
    .unwrap()
    .rows()
    .unwrap();
    assert!(!roots.is_empty());
    let equations = inferred::math_indexed_equations::View::from_checked(
        result
            .relation("inferred", "math_indexed_equations")
            .unwrap()
            .checked(),
    )
    .unwrap()
    .rows()
    .unwrap();
    assert_eq!(equations.len(), 1);
    assert_eq!(equations[0].owner_instance_id, id(71));
    assert_eq!(equations[0].equation_decl_id, Some(id(65)));
    assert_eq!(equations[0].sense, Sense::Eq);

    let canonical = &report.stages.last().unwrap().snapshot;
    let equations = compiled::math_indexed_equations::View::from_checked(
        canonical
            .relation("compiled", "math_indexed_equations")
            .unwrap()
            .checked(),
    )
    .unwrap()
    .rows()
    .unwrap();
    assert_eq!(equations.len(), 1);
    assert_eq!(equations[0].owner_instance_id, id(71));
    assert_eq!(equations[0].equation_decl_id, Some(id(65)));
    assert_eq!(equations[0].sense, Sense::Eq);
    assert_eq!(equations[0].residual_quantity_type_id, Some(id(31)));
}
