// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Selected method templates use the ordinary source/finite/realization pipeline.
#[path = "support/demand_source.rs"]
mod demand_source;
#[path = "support/kernel_source.rs"]
mod kernel_source;
#[path = "../../support/native_pipeline.rs"]
mod native_pipeline;
use demand_source::id;
use pse_authoring::{ParseBudget, document::load_package_texts};
use pse_ids::CancellationToken;
use pse_relations::generated::{compiled, inferred};
use pse_schema::model::Cell;
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn source(registry: &pse_schema::Registry) -> pse_authoring::document::DocumentBundle {
    let mut texts = demand_source::source(registry, false)
        .documents
        .iter()
        .map(|document| (document.path.clone(), document.text.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut templates: Value = serde_json::from_str(&texts["templates/state.yaml"]).unwrap();
    for symbol in templates["template_symbols"].as_array_mut().unwrap() {
        if symbol["id"] == json!(id(43)) || symbol["id"] == json!(id(44)) {
            symbol["role"] = json!("expression");
        }
    }
    templates["template_symbols"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "id":id(46),"template_id":id(61),"name":"coefficient","role":"parameter",
            "quantity_type_id":id(31),"indexed_by":[],"doc":"explicit method coefficient"
        }));
    templates["template_symbol_contracts"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "symbol_decl_id":id(46),"solver_type":"continuous","semantic_role":"reporting_only"
        }));
    templates["template_symbol_expressions"] = json!([
        {"symbol_decl_id":id(43),"template_id":id(61),"expression":"coefficient + 3"},
        {"symbol_decl_id":id(44),"template_id":id(62),"expression":"2"}
    ]);
    texts.insert("templates/state.yaml".into(), templates.to_string());
    let mut methods: Value = serde_json::from_str(&texts["methods/closure.yaml"]).unwrap();
    methods["method_specs"][0]["parameter_kinds"] = json!([
        {"name":"coefficient","quantity_kind_id":id(21),"indexed_by":[],"required":true}
    ]);
    methods["method_parameters"] = json!([
        {"method_id":id(80),"name":"coefficient","quantity_type_id":id(31),"natural_unit_id":id(10),"indexed_by":[],"required":true}
    ]);
    texts.insert("methods/closure.yaml".into(), methods.to_string());
    texts.insert("materials/coefficients.yaml".into(), json!({"parameter_values":[{
        "owner_entity_id":id(95),"parameter_kind":"coefficient","index":[],"value":4.0,"unit_id":id(10),"source":null,"std_dev":null,"estimable":false
    }]}).to_string());
    load_package_texts(texts, registry, ParseBudget::default()).unwrap()
}

#[tokio::test]
async fn selected_methods_bind_actual_coefficients_and_preserve_seed_on_reopen() {
    let mut fixture = native_pipeline::Fixture::new();
    let documents = vec![source(&fixture.registry)];
    let model = fixture.commit(documents).await;
    let report = fixture
        .run(model, "P9")
        .await
        .expect("actual selected P0 through P9 realization");
    let stage = &report.stages.last().unwrap().snapshot;
    let read = |snapshot: &pse_catalog::Snapshot, name: &str| {
        let spec = fixture.registry.relation(name).unwrap();
        let batch = snapshot
            .relation(spec.key.namespace.as_str(), spec.key.name)
            .unwrap();
        pse_relations::cells::cells_from_batch(&fixture.registry, spec, batch.batch()).unwrap()
    };
    let parameters = read(stage, "compiled.method_parameter_bindings")
        .into_iter()
        .map(|row| compiled::method_parameter_bindings::Row::from_cells(row).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(parameters.len(), 1);
    assert_eq!(parameters[0].value.to_bits(), 4_f64.to_bits());
    assert_eq!(parameters[0].source_owner, id(95));
    let realized = read(stage, "compiled.method_realizations")
        .into_iter()
        .map(|row| compiled::method_realizations::Row::from_cells(row).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(realized.len(), 3);
    assert!(
        realized
            .iter()
            .all(|row| row.output_symbol_id.is_some() && row.kernel_binding_id.is_none())
    );
    let instances = read(stage, "inferred.instances")
        .into_iter()
        .map(|row| inferred::instances::Row::from_cells(row).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        instances.len(),
        3,
        "existing state plus exactly two selected method instances"
    );
    let reopened = fixture
        .catalog
        .read_pinned_manifest(stage.manifest_ref(), &CancellationToken::new())
        .await
        .unwrap();
    for name in [
        "compiled.symbols",
        "compiled.symbol_expressions",
        "compiled.method_realizations",
        "compiled.method_parameter_bindings",
        "inferred.math_expr_nodes",
    ] {
        let values = |rows: Vec<Vec<Cell>>| {
            let mut values = rows
                .into_iter()
                .map(|row| Cell::Struct(row).literal_spec())
                .collect::<Vec<_>>();
            values.sort();
            values
        };
        assert_eq!(
            values(read(stage, name)),
            values(read(&reopened, name)),
            "{name}"
        );
    }
}

#[tokio::test]
async fn kernel_descriptor_compiles_exact_natural_units_without_inventing_execution() {
    let mut fixture = native_pipeline::Fixture::new();
    let documents = vec![kernel_source::kernel_source(&fixture.registry, false)];
    let model = fixture.commit(documents).await;
    let result = fixture
        .run(model, "P10")
        .await
        .expect("complete descriptor binding and physical admission");
    let stage = &result.stages.last().unwrap().snapshot;
    let binding = stage.relation("compiled", "kernel_bindings").unwrap();
    let bindings = compiled::kernel_bindings::View::try_from_batch_with_registry(
        &fixture.registry,
        binding.batch(),
    )
    .unwrap()
    .rows()
    .unwrap();
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0].kernel_id, id(120));
    assert_eq!(bindings[0].parameter_bindings[0].value, Some(200.0));
    assert_eq!(bindings[0].parameter_bindings[0].unit_id, Some(id(11)));
    let conversions = stage.relation("compiled", "math_unit_converts").unwrap();
    let conversions = compiled::math_unit_converts::View::try_from_batch_with_registry(
        &fixture.registry,
        conversions.batch(),
    )
    .unwrap()
    .rows()
    .unwrap();
    assert!(conversions.iter().any(|row| row.from_unit_id == id(10)
        && row.to_unit_id == id(11)
        && row.scale.to_bits() == 100_f64.to_bits()));
    assert!(conversions.iter().any(|row| row.from_unit_id == id(11)
        && row.to_unit_id == id(10)
        && row.scale.to_bits() == 0.01_f64.to_bits()));
    let reopened = fixture
        .catalog
        .read_pinned_manifest(stage.manifest_ref(), &CancellationToken::new())
        .await
        .expect("producer replay revalidates descriptor sources");
    assert_eq!(
        reopened
            .relation("compiled", "kernel_bindings")
            .unwrap()
            .batch(),
        binding.batch()
    );
}

#[tokio::test]
async fn missing_kernel_input_mapping_cannot_publish_a_realization() {
    let mut fixture = native_pipeline::Fixture::new();
    let documents = vec![kernel_source::kernel_source(&fixture.registry, true)];
    let model = fixture.commit(documents).await;
    assert!(fixture.run(model, "P9").await.is_err());
}
