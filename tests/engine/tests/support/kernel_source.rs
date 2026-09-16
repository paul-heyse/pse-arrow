// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! A complete source descriptor advertises binding and deliberately has no implementation.
use super::demand_source::{id, source};
use pse_authoring::{
    ParseBudget,
    document::{DocumentBundle, load_package_texts},
};
use pse_relations::generated::reference;
use pse_schema::Registry;
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(crate) fn kernel_source(registry: &Registry, missing_input: bool) -> DocumentBundle {
    let bundle = source(registry, false);
    let spec = registry.relation("reference.units").unwrap();
    let mut percent = reference::units::View::from_checked(&bundle.batches[&spec.id])
        .unwrap()
        .rows()
        .unwrap()
        .into_iter()
        .find(|row| row.unit_id == id(10))
        .unwrap();
    percent.unit_id = id(11);
    percent.symbol = "percent_kernel".into();
    percent.name = "percent kernel fixture".into();
    percent.scale_to_canonical = 0.01;
    let mut texts = bundle
        .documents
        .iter()
        .map(|document| (document.path.clone(), document.text.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut methods: Value = serde_json::from_str(&texts["methods/closure.yaml"]).unwrap();
    methods["method_specs"][0]["realization"] = json!("kernel");
    methods["method_specs"][0]["template_id"] = Value::Null;
    methods["method_specs"][0]["kernel_id"] = json!(id(120));
    methods["method_specs"][0]["parameter_kinds"] = json!([
        {"name":"coefficient","quantity_kind_id":id(21),"indexed_by":[],"required":true}
    ]);
    methods["method_provisions"][0]["output_kind"] = json!("kernel_output");
    methods["method_provisions"][0]["symbol_decl_id"] = Value::Null;
    methods["method_provisions"][0]["kernel_output_ordinal"] = json!(0);
    methods["method_provisions"][0]["natural_unit_id"] = json!(id(11));
    methods["method_parameters"] = json!([
        {"method_id":id(80),"name":"coefficient","quantity_type_id":id(31),"natural_unit_id":id(11),"indexed_by":[],"required":true}
    ]);
    methods["method_kernel_inputs"] = if missing_input {
        json!([])
    } else {
        json!([
            {"method_id":id(80),"input_name":"x","dependency_ordinal":0}
        ])
    };
    methods["kernel_specs"] = json!([{
        "kernel_id":id(120),"name":"descriptor_only","version":"1","provider":"fixture",
        "artifact_digest":pse_ids::ContentHash::from_bytes([19;32]),"behavior":"explicit",
        "inputs":[{"name":"x","quantity_type_id":id(31),"natural_unit_id":id(11),"nullable":false,"shape":[]}],
        "outputs":[{"name":"out","quantity_type_id":id(31),"natural_unit_id":id(11),"shape":[]}],
        "parameters":[{"name":"coefficient","quantity_type_id":id(31),"natural_unit_id":id(11),"indexed_by":[]}],
        "null_input_policy":"reject","failure_policy":"typed_error","tolerant_batch":false,
        "effects":"pure","argument_evaluation":"eager","smoothness":"c_infinity",
        "validity":[],"monotonicity":[],"convexity":"unknown","derivative_bindings":[],"execution_forms":[],
        "thread_safe":true,"failure_classes":[],"bindings":[],"nl_function_name":null,
        "doc":"complete descriptor without an installed algorithm","test_suite":"fixture::descriptor_only"
    }]);
    texts.insert("methods/closure.yaml".into(), methods.to_string());
    texts.insert("materials/kernel-units.yaml".into(), json!({"units":[percent],"parameter_values":[{
        "owner_entity_id":id(95),"parameter_kind":"coefficient","index":[],"value":2.0,"unit_id":id(10),
        "source":null,"std_dev":null,"estimable":false
    }]}).to_string());
    load_package_texts(texts, registry, ParseBudget::default()).unwrap()
}
