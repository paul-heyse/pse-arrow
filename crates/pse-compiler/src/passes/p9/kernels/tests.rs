// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native descriptor binding from typed component inputs; no graph or upstream receipt fixture.
use super::*;
use crate::PolicySet;
use crate::passes::native_test::{Inputs, put, read, session};
use pse_authoring::document::OwnedDocumentSet;
use pse_ids::{CancellationToken, FixedBudget};
use pse_relations::{
    columnar::FieldCheckedBatch,
    generated::{
        enums::{DomainKind, QuantityAdditionKind, ScaleKind},
        extension_values::ExtensionDimensionVectorItem,
    },
};
use pse_schema::Registry;
use serde_json::json;
use std::sync::Arc;

fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}

#[tokio::test]
async fn kernel_reader_uses_only_the_declared_p9_input_inventory() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let spec = registry.pass("P9@1").unwrap();
    let rows = spec
        .inputs
        .iter()
        .map(|port| {
            let relation = registry.relation(&port.relation).unwrap();
            (
                relation.key,
                FieldCheckedBatch::concat(&registry, relation, &[]).unwrap(),
            )
        })
        .collect();
    let budget = FixedBudget::new(256 << 20);
    let cancel = CancellationToken::new();
    let (session, sources) = session(&registry, rows, &budget, &cancel).unwrap();
    let ctx = PassContext {
        physical: None,
        registry: &registry,
        documents: &OwnedDocumentSet::default(),
        policies: &PolicySet::default(),
        cancel: &cancel,
        reserver: budget.as_ref(),
        session: &session,
    };
    let mut arguments = AlgorithmInputs::new(budget.as_ref(), "fixture:p9-reader");
    Inventory::new(&mut arguments, &session, &sources, &ctx)
        .await
        .unwrap();
}
fn descriptor() -> reference::kernel_specs::Row {
    serde_json::from_value(json!({
        "kernel_id":id(81),"name":"scalar_fixture","version":"1","provider":"explicit_fixture",
        "artifact_digest":pse_ids::ContentHash::from_bytes([19;32]),"behavior":"explicit",
        "inputs":[{"name":"x","quantity_type_id":id(31),"natural_unit_id":id(11),"nullable":false,"shape":[]}],
        "outputs":[{"name":"out","quantity_type_id":id(31),"natural_unit_id":id(11),"shape":[]}],
        "parameters":[{"name":"coefficient","quantity_type_id":id(31),"natural_unit_id":id(11),"indexed_by":[]}],
        "null_input_policy":"reject","failure_policy":"typed_error","tolerant_batch":false,
        "effects":"pure","argument_evaluation":"eager","smoothness":"c_infinity",
        "validity":[{"input":"x","lower":{"kind":"finite","value":0.0},"upper":{"kind":"unbounded","value":null}}],
        "monotonicity":[],"convexity":"unknown","derivative_bindings":[],"execution_forms":[],
        "thread_safe":true,"failure_classes":[],"bindings":[],"nl_function_name":null,
        "doc":"descriptor binding has no implemented execution backend","test_suite":"fixture::descriptor"
    })).unwrap()
}
fn inputs(registry: &Registry) -> Inputs {
    let mut rows = Inputs::new();
    put::<reference::method_state_parameters::Row>(&mut rows, registry, vec![]);
    put::<normalized::template_params::Row>(&mut rows, registry, vec![]);
    put::<reference::method_parameter_axes::Row>(&mut rows, registry, vec![]);
    put::<normalized::domains::Row>(&mut rows, registry, vec![]);
    put::<normalized::domain_members::Row>(&mut rows, registry, vec![]);
    put::<normalized::material_domain_members::Row>(&mut rows, registry, vec![]);
    put::<inferred::dependency_key_maps::Row>(&mut rows, registry, vec![]);
    physical_rows(&mut rows, registry);
    put(&mut rows, registry, vec![descriptor()]);
    put::<reference::method_specs::Row>(&mut rows, registry, serde_json::from_value(json!([
        {"method_id":id(80),"family":"pure_component","name":"fixture_method","version":"1",
         "provides":[id(82)],"requires":[],"parameter_kinds":[{"name":"coefficient","quantity_kind_id":id(21),"indexed_by":[],"required":true}],
         "realization":"kernel","template_id":null,"kernel_id":id(81),"validity":[],"doc":"declared method"}
    ])).unwrap());
    put::<reference::method_provisions::Row>(&mut rows, registry, serde_json::from_value(json!([
        {"method_id":id(80),"property_kind_id":id(82),"output_kind":"kernel_output","symbol_decl_id":null,"kernel_output_ordinal":0,
         "quantity_type_id":id(31),"natural_unit_id":id(11),"indexed_by":[]}
    ])).unwrap());
    put::<inferred::property_requirements::Row>(&mut rows, registry, serde_json::from_value(json!([
        {"requirement_id":id(83),"state_scope_id":id(84),"property_kind_id":id(82),"index":[],"derivation_id":id(99)}
    ])).unwrap());
    put::<inferred::method_resolutions::Row>(&mut rows, registry, serde_json::from_value(json!([
        {"requirement_id":id(83),"method_id":id(80),"realization":"kernel","template_id":null,"status":"resolved","derivation_id":id(99)}
    ])).unwrap());
    put::<inferred::state_scopes::Row>(&mut rows, registry, serde_json::from_value(json!([
        {"state_scope_id":id(84),"state_instance_id":id(85),"property_package_id":id(86),"derivation_id":id(99)}
    ])).unwrap());
    put::<inferred::instances::Row>(&mut rows, registry, serde_json::from_value(json!([
        {"instance_id":id(85),"parent_instance_id":null,"template_id":id(87),"path":"fixture_state","index":[],"derivation_id":id(99)}
    ])).unwrap());
    put::<reference::method_parameters::Row>(&mut rows, registry, serde_json::from_value(json!([
        {"method_id":id(80),"name":"coefficient","quantity_type_id":id(31),"natural_unit_id":id(11),"indexed_by":[],"required":true}
    ])).unwrap());
    put::<normalized::parameter_values::Row>(&mut rows, registry, serde_json::from_value(json!([
        {"owner_entity_id":id(86),"parameter_kind":"coefficient","index":[],"value":2.0,"unit_id":id(10),"source":null,"std_dev":null,"estimable":false}
    ])).unwrap());
    put::<reference::method_dependencies::Row>(&mut rows, registry, serde_json::from_value(json!([
        {"method_id":id(80),"ordinal":0,"target_kind":"state_symbol","target_id":id(88),"scope_map":"same_state","index_map":[]}
    ])).unwrap());
    put(
        &mut rows,
        registry,
        vec![reference::method_kernel_inputs::Row {
            method_id: id(80),
            input_name: "x".into(),
            dependency_ordinal: 0,
        }],
    );
    put::<inferred::state_dependency_keys::Row>(&mut rows, registry, serde_json::from_value(json!([
        {"requirement_id":id(83),"method_id":id(80),"dependency_ordinal":0,"symbol_decl_id":id(88),"product_id":pse_templates::identity::domain_product_id(&[]),"index":[],"derivation_id":id(99)}
    ])).unwrap());
    rows
}
fn physical_rows(rows: &mut Inputs, registry: &Registry) {
    put(
        rows,
        registry,
        [(10, 1.0), (11, 0.01)]
            .into_iter()
            .map(|(value, scale)| reference::units::Row {
                unit_id: id(value),
                symbol: format!("unit_{value}"),
                name: format!("unit_{value}"),
                dimension: std::array::from_fn(|_| ExtensionDimensionVectorItem { num: 0, den: 1 }),
                scale_to_canonical: scale,
                offset_to_canonical: 0.0,
                is_affine: false,
                reference_state_id: None,
                system: "component".into(),
                doc: String::new(),
            })
            .collect(),
    );
    put(
        rows,
        registry,
        [21, 22]
            .into_iter()
            .map(|kind| reference::quantity_kinds::Row {
                quantity_kind_id: id(kind),
                name: format!("kind_{kind}"),
                dimension: std::array::from_fn(|_| ExtensionDimensionVectorItem { num: 0, den: 1 }),
                extensive: false,
                addition_kind: QuantityAdditionKind::Additive,
                doc: String::new(),
            })
            .collect(),
    );
    put(
        rows,
        registry,
        [(31, 21), (30, 22)]
            .into_iter()
            .map(|(value, kind)| reference::quantity_types::Row {
                quantity_type_id: id(value),
                quantity_kind_id: id(kind),
                basis_id: None,
                reference_state_id: None,
                scale_kind: ScaleKind::Point,
                shape: vec![],
                subject_kind: None,
                canonical_unit_id: id(10),
                nominal_magnitude: None,
                doc: String::new(),
            })
            .collect(),
    );
}

async fn prepare_rows(
    rows: &Inputs,
    registry: &Arc<Registry>,
    cancel: &CancellationToken,
) -> Result<Vec<KernelMethod>, CompilerError> {
    let budget = FixedBudget::new(2usize << 30);
    let (native, sources) = session(registry, rows.clone(), &budget, cancel)?;
    let (selection, _selection_arguments) =
        super::super::selections::select(&native, registry, budget.as_ref(), cancel).await?;
    let physical = Arc::new(
        crate::quantity_relations::PhysicalInventory::load(&native, registry, cancel).await?,
    );
    let (methods, _arguments) = prepare(
        &selection,
        &PassContext {
            registry,
            documents: &OwnedDocumentSet::default(),
            policies: &PolicySet::default(),
            cancel,
            reserver: budget.as_ref(),
            session: &native,
            physical: Some(&physical),
        },
        &native,
        &sources,
    )
    .await?;
    Ok(methods)
}

#[tokio::test]
async fn complete_kernel_descriptor_binds_exact_values_without_claiming_execution() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let rows = inputs(&registry);
    let output = prepare_rows(&rows, &registry, &CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(output.len(), 1);
    assert_eq!(output[0].descriptor, descriptor());
    assert!(output[0].descriptor.bindings.is_empty());
    assert!(output[0].descriptor.execution_forms.is_empty());
    assert_eq!(
        output[0].inputs,
        vec![(
            "x".into(),
            pse_ids::symbol_instance_id(id(85), id(88), IndexTuple(&[]))
        )]
    );
    assert_eq!(
        output[0].parameters,
        vec![(
            "coefficient".into(),
            None,
            Some(200.0),
            Some(UnitId::from_id(id(11)))
        )]
    );
    assert!(
        output[0]
            .support
            .iter()
            .any(|source| source.relation.qualified_name() == "normalized.parameter_values")
    );
}

#[tokio::test]
async fn missing_mapping_parameter_and_pre_cancel_refuse_binding() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    for name in [
        "reference.method_kernel_inputs",
        "normalized.parameter_values",
    ] {
        let mut rows = inputs(&registry);
        let spec = registry.relation(name).unwrap();
        rows.insert(
            spec.key,
            FieldCheckedBatch::concat_reserved(
                &registry,
                spec,
                &[],
                FixedBudget::new(1 << 20).as_ref(),
                &CancellationToken::new(),
            )
            .unwrap(),
        );
        assert!(
            prepare_rows(&rows, &registry, &CancellationToken::new())
                .await
                .is_err(),
            "{name}"
        );
    }
    let cancel = CancellationToken::new();
    cancel.cancel();
    assert!(
        prepare_rows(&inputs(&registry), &registry, &cancel)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn indexed_parameter_and_wrong_physical_contract_cannot_use_scalar_carrier() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let mut rows = inputs(&registry);
    let mut descriptor = descriptor();
    descriptor.parameters[0].indexed_by = vec![DomainKind::Species];
    descriptor.parameters[0].quantity_type_id = id(32);
    let mut types = read::<reference::quantity_types::Row>(&rows, &registry);
    let mut indexed = types
        .iter()
        .find(|row| row.quantity_type_id == id(31))
        .unwrap()
        .clone();
    indexed.quantity_type_id = id(32);
    indexed.shape = vec![DomainKind::Species];
    types.push(indexed);
    put(&mut rows, &registry, types);
    let mut parameters = read::<reference::method_parameters::Row>(&rows, &registry);
    parameters[0].quantity_type_id = id(32);
    parameters[0].indexed_by = vec![DomainKind::Species];
    put(&mut rows, &registry, parameters);
    put(&mut rows, &registry, vec![descriptor]);
    assert!(matches!(
        prepare_rows(&rows, &registry, &CancellationToken::new()).await,
        Err(CompilerError::KernelUnbound { .. })
    ));
    let mut rows = inputs(&registry);
    let mut parameters = read::<reference::method_parameters::Row>(&rows, &registry);
    parameters[0].quantity_type_id = id(30);
    put(&mut rows, &registry, parameters);
    assert!(
        prepare_rows(&rows, &registry, &CancellationToken::new())
            .await
            .is_err()
    );
}
