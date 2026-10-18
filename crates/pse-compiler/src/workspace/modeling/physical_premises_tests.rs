// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Context-only edits exercise real source checking and retained mathematical products.

use super::*;
use pse_modeling::specialize::root_instance;
use pse_quantity::{EntityKind, EntityKindId, QuantityRegistryBuilder, QuantityTypeId};

const SOURCE: &str = "package physical_edit { def Root { var x:Scalar; eq e:x*x==1; } }";

fn setup() -> (CompilerWorkspace, Arc<ModelingRevision>, DeclarationId, CompilerContext) {
    setup_source(SOURCE)
}

fn setup_source(source: &str) -> (CompilerWorkspace, Arc<ModelingRevision>, DeclarationId, CompilerContext) {
    let inputs = crate::authored_transfer_tests::context();
    let rows = crate::authored_transfer_tests::rows(source);
    let root = crate::authored_transfer_tests::root(&rows, "physical_edit", "Root");
    let mut workspace = CompilerWorkspace::new(inputs.clone(), WorkspaceLimits::default()).unwrap();
    let revision = workspace.publish_modeling(rows, PhysicalScope::default()).unwrap();
    (workspace, revision, root, inputs)
}

fn prepare(workspace: &mut CompilerWorkspace, root: DeclarationId) -> PreparedModeling {
    workspace.prepare_modeling_cancellable(root, root_instance(root), Bindings::default(),
        Limits::default(), Arc::new(AtomicBool::new(false))).unwrap()
}

fn with_unrelated_named_type(inputs: &CompilerContext) -> CompilerContext {
    let mut changed = inputs.clone();
    let mut declarations = inputs.quantities.to_builder();
    let axis = EntityKindId::from_id(SemanticId::from_bytes([211; 16]));
    declarations.entity_kind(EntityKind { id: axis, name: "unused_physical_axis".into() });
    let mut quantity = inputs.quantities.quantity_type(inputs.quantities.neutral_dimensionless().unwrap()).unwrap().clone();
    quantity.id = QuantityTypeId::from_id(SemanticId::from_bytes([212; 16]));
    quantity.name = Some("UnusedPhysicalVector".into());
    quantity.key.shape = vec![axis];
    declarations.quantity_type(quantity);
    changed.quantities = Arc::new(declarations.build().unwrap());
    changed
}

/// Change one admitted declaration through its ordinary serializable builder, then
/// re-admit the complete inventory. Tests never mutate an admitted registry in place.
fn change_scalar(inputs: &CompilerContext, edit: impl FnOnce(&mut serde_json::Value)) -> CompilerContext {
    let scalar = inputs.quantities.neutral_dimensionless().unwrap();
    let id = serde_json::to_value(scalar).unwrap();
    let mut declarations = serde_json::to_value(inputs.quantities.to_builder()).unwrap();
    let row = declarations["quantity_types"].as_array_mut().unwrap().iter_mut()
        .find(|row| row["id"] == id).unwrap();
    edit(row);
    let builder: QuantityRegistryBuilder = serde_json::from_value(declarations).unwrap();
    let mut changed = inputs.clone();
    changed.quantities = Arc::new(builder.build().unwrap());
    changed
}

fn full_identity(inputs: &CompilerContext) -> ContentHash {
    physical_identity(&inputs.quantities, &inputs.preconditions)
}

fn assert_same_mathematics(old: &PreparedModeling, current: &PreparedModeling) {
    assert!(!old.admitted.bodies.is_empty());
    assert_eq!(old.admitted.bodies.len(), current.admitted.bodies.len());
    for (key, body) in &old.admitted.bodies {
        let now = current.admitted.bodies.get(key).unwrap_or_else(|| panic!("unrelated context retained the body identity: old {:?} semantic {:?}; current {:?}", body.spec(), body.semantic_identity(), current.admitted.bodies.values().map(|body| (body.spec(), body.semantic_identity())).collect::<Vec<_>>()));
        assert!(Arc::ptr_eq(body.math(), now.math()), "unchanged mathematics was reconstructed");
        assert_eq!(body.spec().physical, now.spec().physical);
    }
}

#[test]
fn context_only_unrelated_named_type_preserves_math_and_renews_checked_owners() {
    let (mut workspace, prior, root, inputs) = setup();
    let old = prepare(&mut workspace, root);
    let generation = workspace.generation();
    let changed = with_unrelated_named_type(&inputs);
    assert_ne!(full_identity(&inputs), full_identity(&changed));
    workspace.refresh_modeling_context(changed.clone(), Arc::new(AtomicBool::new(false))).unwrap();
    let current = prepare(&mut workspace, root);
    assert_same_mathematics(&old, &current);
    assert_eq!(old.consumed_physical_identity(), current.consumed_physical_identity(),
        "an unrelated declaration changed the complete preparation's physical key");
    assert_eq!(workspace.generation(), generation, "unrelated physical edit rotated the compiler lane");
    let head = &workspace.modeling.as_ref().unwrap().revision;
    assert!(!Arc::ptr_eq(head, &prior), "current admission reused the previous checked wrapper");
    assert!(std::ptr::eq(head.checked().context().quantities, changed.quantities.as_ref()));
    assert!(std::ptr::eq(prior.checked().context().quantities, inputs.quantities.as_ref()));
    assert!(Arc::ptr_eq(&workspace.context().quantities, &changed.quantities));
    // The source declarations remain exactly equal; refreshing context is not a source edit.
    assert_eq!(head.declarations(), prior.declarations());
}

#[test]
fn context_only_same_quantity_id_with_changed_meaning_invalidates_consumed_body() {
    let (mut workspace, prior, root, inputs) = setup();
    let old = prepare(&mut workspace, root);
    let changed = change_scalar(&inputs, |row| row["nominal_magnitude"] = serde_json::json!(17.0));
    assert_ne!(full_identity(&inputs), full_identity(&changed));
    workspace.refresh_modeling_context(changed.clone(), Arc::new(AtomicBool::new(false))).unwrap();
    let current = prepare(&mut workspace, root);
    assert!(!old.admitted.bodies.is_empty());
    for body in old.admitted.bodies.values() {
        assert!(current.admitted.bodies.values().all(|now|
            body.spec().physical != now.spec().physical && !Arc::ptr_eq(body.math(), now.math())));
    }
    assert!(std::ptr::eq(prior.checked().context().quantities, inputs.quantities.as_ref()));
    // Repeating the new context has its own stable products, rather than disabling all reuse.
    let repeated = prepare(&mut workspace, root);
    assert_same_mathematics(&current, &repeated);
}

#[test]
fn failed_context_refresh_restores_prior_checked_source_and_usable_mathematics() {
    let (mut workspace, prior, root, inputs) = setup();
    let old = prepare(&mut workspace, root);
    let incompatible = change_scalar(&inputs, |row| row["name"] = serde_json::json!("RenamedScalar"));
    assert!(workspace.refresh_modeling_context(incompatible, Arc::new(AtomicBool::new(false))).is_err());
    assert!(Arc::ptr_eq(&workspace.context().quantities, &inputs.quantities));
    assert!(Arc::ptr_eq(&workspace.modeling.as_ref().unwrap().revision, &prior));
    assert!(std::ptr::eq(prior.checked().context().quantities, inputs.quantities.as_ref()));
    let restored = prepare(&mut workspace, root);
    assert!(!restored.admitted.bodies.is_empty());
    assert_eq!(old.admitted.inputs, restored.admitted.inputs);
    assert_eq!(old.admitted.outputs, restored.admitted.outputs);
    // A later valid refresh must still work after setters and failed admission were discarded.
    workspace.refresh_modeling_context(with_unrelated_named_type(&inputs), Arc::new(AtomicBool::new(false))).unwrap();
    let recovered = prepare(&mut workspace, root);
    assert_same_mathematics(&restored, &recovered);
}

#[test]
fn cancelled_context_refresh_leaves_prior_context_and_checked_head_usable() {
    let (mut workspace, prior, root, inputs) = setup();
    let old = prepare(&mut workspace, root);
    let changed = with_unrelated_named_type(&inputs);
    assert!(matches!(workspace.refresh_modeling_context(changed.clone(), Arc::new(AtomicBool::new(true))), Err(CompileError::Cancelled)));
    assert!(Arc::ptr_eq(&workspace.context().quantities, &inputs.quantities));
    assert!(Arc::ptr_eq(&workspace.modeling.as_ref().unwrap().revision, &prior));
    let restored = prepare(&mut workspace, root);
    assert_same_mathematics(&old, &restored);
    workspace.refresh_modeling_context(changed, Arc::new(AtomicBool::new(false))).unwrap();
    let current = prepare(&mut workspace, root);
    assert_same_mathematics(&old, &current);
}

#[test]
fn unrelated_named_type_preserves_finite_function_math_after_cached_admission() {
    let (mut workspace, _, root, inputs) = setup_source(
        "package physical_edit { fn square(x:Scalar)->Scalar=x*x; def Root { var x:Scalar; eq e:square(x)+x==2; } }",
    );
    let old = prepare(&mut workspace, root);
    // Warm the existing function-admission path before changing its physical context.
    assert_same_mathematics(&old, &prepare(&mut workspace, root));
    workspace.refresh_modeling_context(with_unrelated_named_type(&inputs), Arc::new(AtomicBool::new(false))).unwrap();
    let current = prepare(&mut workspace, root);
    assert_same_mathematics(&old, &current);
}

#[test]
fn equal_content_new_physical_owners_refresh_checked_shell_and_reuse_mathematics() {
    let (mut workspace, prior, root, inputs) = setup();
    let old = prepare(&mut workspace, root);
    let mut current = inputs.clone();
    current.quantities = Arc::new(inputs.quantities.as_ref().clone());
    current.preconditions = Arc::new(inputs.preconditions.as_ref().clone());
    assert_eq!(full_identity(&inputs), full_identity(&current));
    workspace.refresh_modeling_context(current.clone(), Arc::new(AtomicBool::new(false))).unwrap();
    let head = &workspace.modeling.as_ref().unwrap().revision;
    assert!(!Arc::ptr_eq(head, &prior));
    assert!(std::ptr::eq(head.checked().context().quantities, current.quantities.as_ref()));
    assert!(std::ptr::eq(head.checked().context().preconditions, current.preconditions.as_ref()));
    let fresh = prepare(&mut workspace, root);
    assert_same_mathematics(&old, &fresh);
    assert_eq!(old.consumed_physical_identity(), fresh.consumed_physical_identity());
}
