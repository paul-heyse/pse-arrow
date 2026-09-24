// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Shared real authored scalar model for compiler and solver journeys.
#![allow(clippy::unwrap_used, reason = "explicit source fixture construction")]
#[path = "physical_source.rs"]
mod physical_source;
use pse_ids::SemanticId;
use pse_schema::Registry;
use std::collections::BTreeMap;
fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}

pub(crate) fn source_texts(registry: &Registry, second: bool) -> BTreeMap<String, String> {
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
    texts
}
