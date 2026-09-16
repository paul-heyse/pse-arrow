// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Small physical declarations isolate actual read-seed, guard and method closure behavior.
use pse_authoring::{
    ParseBudget,
    document::{DocumentBundle, load_package_texts},
};
use pse_ids::SemanticId;
use pse_schema::Registry;
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(crate) fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}
fn symbol(value: u8, template: u8, name: &str) -> Value {
    json!({"id":id(value),"template_id":id(template),"name":name,"role":"variable","quantity_type_id":id(31),"indexed_by":[],"doc":"declared scalar property"})
}
fn template(value: u8, name: &str, kind: &str) -> Value {
    json!({"id":id(value),"name":name,"version":"1","kind":kind,"doc":"finite closure fixture"})
}
fn property(value: u8, name: &str) -> Value {
    json!({"property_kind_id":id(value),"idaes_name":name,"quantity_kind_id":id(21),"basis_id":null,"shape":[],"category":"thermo","doc":"declared scalar contract"})
}
fn method(value: u8, template: u8, property: u8, requires: &[u8], family: &str) -> Value {
    let required = requires.iter().map(|value| id(*value)).collect::<Vec<_>>();
    json!({"method_id":id(value),"family":family,"name":format!("method{value}"),"version":"1","provides":[id(property)],"requires":required,"parameter_kinds":[],"realization":"equation_template","template_id":id(template),"kernel_id":null,"validity":[],"doc":"actual finite dependency declaration"})
}
fn provision(method: u8, property: u8, symbol: u8) -> Value {
    json!({"method_id":id(method),"property_kind_id":id(property),"output_kind":"template_symbol","symbol_decl_id":id(symbol),"kernel_output_ordinal":null,"quantity_type_id":id(31),"natural_unit_id":id(10),"indexed_by":[]})
}
fn selection(value: u8, method: u8, property: u8) -> Value {
    json!({"selection_id":id(value),"is_default":false,"property_package_id":id(95),"scope_kind":"package","scope_ids":[],"property_kind_id":id(property),"family":"pure_component","method_id":id(method),"options":[]})
}
fn dependency(method: u8, target: u8) -> Value {
    json!({"method_id":id(method),"ordinal":0,"target_kind":"property","target_id":id(target),"scope_map":"same_state","index_map":[]})
}
pub(crate) fn source(registry: &Registry, ambiguous: bool) -> DocumentBundle {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let physical = pse_authoring::document::load_package(
        &root.join("tests/fixtures/packages/physical-primitives"),
        registry,
        ParseBudget::default(),
    )
    .unwrap();
    let mut texts = physical
        .documents
        .iter()
        .map(|document| (document.path.clone(), document.text.clone()))
        .collect::<BTreeMap<_, _>>();
    let symbols = vec![
        symbol(40, 60, "requested"),
        symbol(41, 60, "state_value"),
        symbol(42, 60, "dormant"),
        symbol(43, 61, "output"),
        symbol(44, 62, "output"),
        symbol(45, 63, "output"),
    ];
    let contracts=(40..=45).map(|value|json!({"symbol_decl_id":id(value),"solver_type":"continuous","semantic_role":"state"})).collect::<Vec<_>>();
    let properties=[(40,100),(41,102),(42,103)].into_iter().map(|(symbol,property)|json!({"symbol_decl_id":id(symbol),"property_kind_id":id(property),"scope_selector_id":id(65)})).collect::<Vec<_>>();
    texts.insert("templates/state.yaml".into(),json!({
        "templates":[template(60,"State","state_block"),template(61,"FirstMethod","helper"),template(62,"DependencyMethod","helper"),template(63,"AlternativeMethod","helper")],
        "template_symbols":symbols,"template_symbol_contracts":contracts,
        "template_symbol_properties":properties,
        "template_scopes":[{"template_id":id(60),"name":"self","scope_id":id(65)}],
        "template_equations":[{"id":id(64),"template_id":id(60),"name":"read","indexed_by":[],"expression":"if true then requested + requested == 1 else dormant == 2","sense":"eq","doc":"dead branch must not demand dormant"}]
    }).to_string());
    texts.insert("instances/state.yaml".into(),json!({
        "scopes":[{"scope_id":id(65),"root_term_id":id(66)}],
        "selector_terms":[{"term_id":id(66),"scope_id":id(65),"parent_term_id":null,"ordinal":0,"op":"self","entity_id":null,"entity_kind":null,"tag":null,"parameter_name":null}],
        "instances":[{"id":id(70),"template_id":id(60),"name":"state","property_package_id":id(95),"param_values":[],"feature_values":[],"doc":"actual singleton state"}]
    }).to_string());
    let mut methods = vec![
        method(80, 61, 100, &[101], "pure_component"),
        method(81, 62, 101, &[102], "pure_component"),
        method(82, 60, 102, &[], "state_definition"),
    ];
    let mut provisions = vec![
        provision(80, 100, 43),
        provision(81, 101, 44),
        provision(82, 102, 41),
    ];
    if ambiguous {
        methods.push(method(83, 63, 100, &[], "pure_component"));
        provisions.push(provision(83, 100, 45));
    }
    texts.insert("methods/closure.yaml".into(),json!({
        "property_kinds":[property(100,"requested"),property(101,"intermediate"),property(102,"state"),property(103,"dormant")],
        "method_specs":methods,"method_provisions":provisions,"method_dependencies":[dependency(80,101),dependency(81,102)],
        "method_precedence":[{"is_default":false,"property_specific":true,"scope_kind":"package","rank":100},{"is_default":false,"property_specific":false,"scope_kind":"package","rank":90}]
    }).to_string());
    let mut selections = vec![selection(110, 80, 100), selection(111, 81, 101)];
    if ambiguous {
        selections.push(selection(112, 83, 100));
    }
    texts.insert("properties/closure.yaml".into(),json!({
        "property_packages":[{"id":id(95),"name":"physical","material_system_id":id(92),"unit_set_id":id(9),"state_definition_method_id":id(82),"temperature_ref":298.15,"pressure_ref":100000.0,"include_enthalpy_of_formation":false,"bubble_dew_method_id":null,"doc":"actual selected package"}],
        "method_selections":selections
    }).to_string());
    texts.insert("materials/system.yaml".into(),json!({"material_systems":[{"id":id(92),"name":"scalar_system","species_ids":[],"phase_ids":[],"doc":"no material axes requested"}]}).to_string());
    load_package_texts(texts, registry, ParseBudget::default()).unwrap()
}
