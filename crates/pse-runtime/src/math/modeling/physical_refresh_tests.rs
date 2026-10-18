// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Same-lineage physical refresh keeps current and detached interpretation owners distinct.
use super::*;
use pse_compiler::workspace::{CompilerContext, PreparedModeling};
use pse_modeling::{PhysicalScope, specialize::root_instance};
use std::sync::atomic::AtomicBool;

fn context() -> CompilerContext {
    let physical = crate::workflow::tests::physical();
    CompilerContext {
        quantities: physical.quantities,
        preconditions: physical.preconditions,
        providers: BTreeMap::new(),
    }
}

fn extend(inputs: &CompilerContext) -> CompilerContext {
    let mut rows = inputs.quantities.to_builder();
    let axis = pse_quantity::EntityKindId::from_id(SemanticId::from_bytes([231; 16]));
    rows.entity_kind(pse_quantity::EntityKind { id: axis, name: "unconsumed_axis".into() });
    let mut quantity = inputs.quantities.quantity_type(inputs.quantities.neutral_dimensionless().unwrap()).unwrap().clone();
    quantity.id = pse_quantity::QuantityTypeId::from_id(SemanticId::from_bytes([232; 16]));
    quantity.name = Some("UnconsumedVector".into());
    quantity.key.shape = vec![axis];
    rows.quantity_type(quantity);
    CompilerContext { quantities: Arc::new(rows.build().unwrap()), ..inputs.clone() }
}

fn prepare(service: &Arc<MathService>, workspace: &Workspace, revision: &ModelingRevision,
    root: DeclarationId) -> PreparedModeling {
    service.with_modeling_compiler(
        super::super::CompilerAuthority::private(workspace.clone()).unwrap(), revision,
        |compiler| Ok(compiler.prepare_modeling_cancellable(root, root_instance(root),
            Bindings::default(), Limits::default(), Arc::new(AtomicBool::new(false)))?),
    ).unwrap()
}

#[tokio::test]
async fn physical_refresh_keeps_one_live_lane_and_historical_context_without_owner_leaks() {
    let runtime = crate::workflow::tests::runtime_on(1 << 30, super::super::MathPolicy {
        workspace_bytes: 32 << 20, worker_bytes: 64 << 20, foreign_bytes: 1 << 20,
        inner_session_bytes: 0, ..Default::default()
    });
    let service = runtime.shared.math();
    let baseline = service.pool.reserved();
    let original = context();
    let changed = extend(&original);
    let lineage = CompilerLineage::new("physical-refresh-control", "current-interpretation",
        runtime.sessions(), runtime.registry().clone(), runtime.model.clone()).unwrap();
    let first = service.compiler_lane(lineage.clone(), original.clone()).unwrap();
    let rows = pse_authoring::language::parse(
        "package physical_refresh { def Root { var x:Scalar; eq e:x*x==1; } }",
        SemanticId::from_bytes([230; 16]), pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default()).unwrap();
    let root = rows.iter().find(|row| row.name == "Root").unwrap().declaration_id;
    let original_physical = pse_compiler::workspace::physical_identity(&original.quantities, &original.preconditions);
    let changed_physical = pse_compiler::workspace::physical_identity(&changed.quantities, &changed.preconditions);
    assert_ne!(original_physical, changed_physical);
    let old = service.modeling_revision(&first, rows.clone(), PhysicalScope::default(),
        Arc::default(), &original_physical, &crate::CancelSource::new()).await.unwrap();
    let old_math = prepare(service, &first, &old, root);
    let second = {
        let held = first.compiler.lock().unwrap();
        let second = service.compiler_lane(lineage, changed.clone()).unwrap();
        assert!(Arc::ptr_eq(&first.compiler, &second.compiler), "context-only edit created a second live lane");
        assert!(Arc::ptr_eq(&held.context().quantities, &original.quantities),
            "selecting desired context mutated the active writer");
        second
    };
    let current = service.modeling_revision(&second, rows, PhysicalScope::default(), Arc::default(),
        &changed_physical, &crate::CancelSource::new()).await.unwrap();
    assert_ne!(old.identity(), current.identity(), "full source identity lost physical admission context");
    assert!(Arc::ptr_eq(&second.compiler.lock().unwrap().context().quantities, &changed.quantities));
    assert!(std::ptr::eq(old.checked().context().quantities, original.quantities.as_ref()));
    assert!(std::ptr::eq(current.checked().context().quantities, changed.quantities.as_ref()));
    let current_math = prepare(service, &second, &current, root);
    assert_eq!(old_math.consumed_physical_identity(), current_math.consumed_physical_identity());
    assert!(!old_math.admitted.bodies.is_empty());
    for (key, body) in &old_math.admitted.bodies {
        assert!(Arc::ptr_eq(body.math(), current_math.admitted.bodies[key].math()),
            "same-lineage context refresh rebuilt eligible mathematics");
    }
    let historical_math = prepare(service, &second, &old, root);
    assert!(Arc::ptr_eq(&historical_math.compiler_context().quantities, &original.quantities));
    assert!(Arc::ptr_eq(&second.compiler.lock().unwrap().context().quantities, &changed.quantities));
    assert!(second.compiler.lock().unwrap().selects_modeling_revision(&current.admitted));
    assert_eq!(service.compiler_lanes.lock().unwrap().iter().filter(|lane| lane.historical.is_none()).count(), 1);
    assert_eq!(historical_math.consumed_physical_identity(), old_math.consumed_physical_identity());
    drop((old_math, current_math, historical_math, old, current, first, second));
    service.clear_program_cache();
    service.trim_compiler_lanes();
    assert!(service.compiler_lanes.lock().unwrap().is_empty());
    assert_eq!(service.pool.reserved(), baseline);
}
