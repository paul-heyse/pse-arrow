// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Counted controls on the actual admitted-package production route.
use super::test_observation::{BodyObservations, Query, observed_workspace};
use super::*;
use std::sync::atomic::AtomicUsize;
fn rows(text: &str) -> Vec<Declaration> {
    pse_authoring::language::parse(
        text,
        SemanticId::from_bytes([91; 16]),
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap()
}
fn prepare(
    workspace: &mut CompilerWorkspace,
    root: DeclarationId,
    bindings: Bindings,
) -> PreparedModeling {
    workspace
        .prepare_modeling_cancellable(
            root,
            InstanceId::from_id(SemanticId::NIL),
            bindings,
            Limits::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
}
#[test]
fn complete_body_queries_reuse_unrelated_math_and_refresh_revision_attribution() {
    let executions = BodyObservations::new();
    let _entered = executions.enter();
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    let text = "package p { def Root { var x: Scalar; eq a: x*x == 1; eq b: x == 2; } }";
    let initial = rows(text);
    let root = initial
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    workspace
        .publish_modeling(initial, PhysicalScope::default())
        .unwrap();
    let a = prepare(&mut workspace, root, Bindings::default());
    let original_occurrences = a.occurrences();
    let first = executions.admissions();
    assert!(first > 0);
    workspace
        .publish_modeling(rows(&format!("\n\n{text}")), PhysicalScope::default())
        .unwrap();
    let shifted = prepare(&mut workspace, root, Bindings::default());
    assert_eq!(executions.admissions(), first);
    assert_ne!(a.occurrences(), shifted.occurrences());
    for (key, body) in &a.admitted.bodies {
        assert!(Arc::ptr_eq(&body.math, &shifted.admitted.bodies[key].math));
    }
    workspace
        .publish_modeling(
            rows(&text.replace("x*x == 1", "x*x == 3")),
            PhysicalScope::default(),
        )
        .unwrap();
    let changed = prepare(&mut workspace, root, Bindings::default());
    let changed_bodies = changed
        .admitted
        .bodies
        .keys()
        .filter(|key| !a.admitted.bodies.contains_key(*key))
        .count();
    assert!(changed_bodies > 0);
    assert_eq!(executions.admissions(), first + changed_bodies);
    assert!(a.admitted.bodies.iter().any(|(key, body)| {
        changed
            .admitted
            .bodies
            .get(key)
            .is_some_and(|next| Arc::ptr_eq(&body.math, &next.math))
    }));
    assert_eq!(a.occurrences(), original_occurrences);
    assert_eq!(executions.completed(), executions.admissions());
    let mut fresh = CompilerWorkspace::new(workspace.inputs.clone(), workspace.limits).unwrap();
    fresh
        .publish_modeling(
            rows(&text.replace("x*x == 1", "x*x == 3")),
            PhysicalScope::default(),
        )
        .unwrap();
    let independent = prepare(&mut fresh, root, Bindings::default());
    assert_eq!(changed.admitted.as_ref(), independent.admitted.as_ref());
    assert!(!pse_math::SharedAllocation::ptr_eq(
        &changed.admitted,
        &independent.admitted
    ));
}
#[test]
fn immutable_binding_inputs_allow_a_b_a_and_more_selections_than_old_root_cap() {
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits {
            query_values: 2,
            ..WorkspaceLimits::default()
        },
    )
    .unwrap();
    let declarations = rows("package p { def Root { var x: Scalar; eq a:x*x==1; } }");
    let root = declarations
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    workspace
        .publish_modeling(declarations, PhysicalScope::default())
        .unwrap();
    let a_bindings = Bindings {
        demand: vec!["x".into()],
        ..Bindings::default()
    };
    let a = prepare(&mut workspace, root, a_bindings.clone());
    let _b = prepare(&mut workspace, root, Bindings::default());
    let again = prepare(&mut workspace, root, a_bindings);
    assert!(pse_math::SharedAllocation::ptr_eq(&a.model, &again.model));
    for n in 1..=8 {
        workspace
            .prepare_modeling_cancellable(
                root,
                InstanceId::from_id(SemanticId::from_bytes([n; 16])),
                Bindings::default(),
                Limits::default(),
                Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
    }
}

#[test]
fn request_hash_buckets_compare_every_field_and_reuse_a_after_collisions() {
    use pse_modeling::specialize::Value;
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    let root = publish_frontier_source(
        &mut workspace,
        "package p { def Root { var x:Scalar; eq a:x==1; } }",
    );
    let instance = InstanceId::from_id(SemanticId::NIL);
    let bindings = Bindings::default();
    let limits = Limits::default();
    let base_hash = request_hash(root, instance, &bindings, limits);
    let (_, original) = workspace
        .modeling_request_in_bucket(root, instance, bindings.clone(), limits, 0)
        .unwrap();
    let mut variants = vec![
        (
            DeclarationId::from_id(SemanticId::from_bytes([19; 16])),
            instance,
            bindings.clone(),
            limits,
        ),
        (
            root,
            InstanceId::from_id(SemanticId::from_bytes([20; 16])),
            bindings.clone(),
            limits,
        ),
    ];
    for field in 0..6 {
        let mut changed = bindings.clone();
        match field {
            0 => {
                changed.arguments.insert("n".into(), Value::Integer(2));
            }
            1 => {
                changed.scope.insert("n".into(), Value::Integer(2));
            }
            2 => {
                changed.facts.insert(
                    pse_modeling::analysis::Fact::Stage("s".into()),
                    Value::Boolean(true),
                );
            }
            3 => changed.demand.push("x".into()),
            4 => {
                changed
                    .formulation
                    .omitted
                    .insert(SemanticId::from_bytes([21; 16]));
            }
            _ => {
                changed
                    .formulation
                    .elastic
                    .insert(SemanticId::from_bytes([22; 16]), Value::Integer(2));
            }
        }
        variants.push((root, instance, changed, limits));
    }
    for field in 0..5 {
        let mut changed = limits;
        match field {
            0 => changed.depth += 1,
            1 => changed.items += 1,
            2 => changed.members += 1,
            3 => changed.body_occurrences = Some(17),
            _ => changed.body_slots = Some(23),
        }
        variants.push((root, instance, bindings.clone(), changed));
    }
    let mut handles = vec![original];
    for (root, instance, bindings, limits) in variants {
        assert_ne!(request_hash(root, instance, &bindings, limits), base_hash);
        let (_, request) = workspace
            .modeling_request_in_bucket(root, instance, bindings.clone(), limits, 0)
            .unwrap();
        assert!(
            !handles.contains(&request),
            "a colliding request must keep its own complete meaning"
        );
        let (_, again) = workspace
            .modeling_request_in_bucket(root, instance, bindings, limits, 0)
            .unwrap();
        assert!(again == request);
        handles.push(request);
    }
    let (_, again) = workspace
        .modeling_request_in_bucket(root, instance, bindings, limits, 0)
        .unwrap();
    assert!(again == original);
    let state = workspace.modeling.as_ref().unwrap();
    assert_eq!(state.requests.len(), 1);
    assert!(state.requests[&0] == handles);
    assert!(state.request_bytes > handles.len() * size_of::<Bindings>());
}

#[test]
fn request_bucket_bytes_rotate_generation_without_invalidating_escaped_products() {
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    let root = publish_frontier_source(
        &mut workspace,
        "package p { def Root { var x:Scalar; eq a:x==1; } }",
    );
    let escaped = prepare(&mut workspace, root, Bindings::default());
    let occurrences = escaped.occurrences();
    let state = workspace.modeling.as_ref().unwrap();
    let per_request = state.request_bytes;
    assert!(per_request > 0);
    workspace.limits.input_bytes = state.input_bytes
        + workspace.inputs.quantities.allocation_extent()
        + workspace.inputs.preconditions.allocation_extent()
        + 2 * per_request;
    let available = workspace.limits.input_bytes - state.input_bytes;
    let generation = workspace.generation();
    for n in 1..=available / per_request + 1 {
        let mut id = [0; 16];
        id[..8].copy_from_slice(&(n as u64).to_le_bytes());
        workspace
            .modeling_request(
                root,
                InstanceId::from_id(SemanticId::from_bytes(id)),
                Bindings::default(),
                Limits::default(),
            )
            .unwrap();
        if workspace.generation() != generation {
            break;
        }
    }
    assert_eq!(workspace.generation(), generation + 1);
    let state = workspace.modeling.as_ref().unwrap();
    assert_eq!(state.requests.values().map(Vec::len).sum::<usize>(), 1);
    assert_eq!(state.request_bytes, per_request);
    drop(workspace);
    assert_eq!(escaped.occurrences(), occurrences);
    assert!(!escaped.model.symbols.is_empty());
}

#[derive(Debug)]
struct RefuseOnce(AtomicBool);
impl ModelingBodyRetention for RefuseOnce {
    fn generation(&self) -> u64 {
        0
    }
    fn get(
        &self,
        _: pse_ids::roles::SemanticBodyHash,
    ) -> std::result::Result<Option<Arc<AdmittedBody>>, MathError> {
        Ok(None)
    }
    fn retain(
        &self,
        _: u64,
        _: pse_ids::roles::SemanticBodyHash,
        body: Arc<AdmittedBody>,
    ) -> std::result::Result<Arc<AdmittedBody>, MathError> {
        if self.0.swap(false, Ordering::Relaxed) {
            Err(MathError::Contract("transient allocation refusal".into()))
        } else {
            Ok(body)
        }
    }
}
#[test]
fn effect_refusal_does_not_poison_incremental_body_admission() {
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    workspace
        .attach_body_retention(Arc::new(RefuseOnce(AtomicBool::new(true))))
        .unwrap();
    let declarations = rows("package p { def Root { var x: Scalar; eq a:x*x==1; } }");
    let root = declarations
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    workspace
        .publish_modeling(declarations, PhysicalScope::default())
        .unwrap();
    let request = |workspace: &mut CompilerWorkspace| {
        workspace.prepare_modeling_cancellable(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings::default(),
            Limits::default(),
            Arc::new(AtomicBool::new(false)),
        )
    };
    let refusal = request(&mut workspace).unwrap_err();
    assert!(refusal.to_string().contains("transient allocation refusal"));
    assert!(
        request(&mut workspace).is_ok(),
        "same inputs retry after an allocation refusal"
    );
    assert!(
        workspace
            .attach_body_retention(Arc::new(RefuseOnce(AtomicBool::new(false))))
            .is_err(),
        "retention cannot change behind already admitted Salsa inputs"
    );
}

#[derive(Debug, Default)]
struct WrongBodyOnce {
    body: std::sync::Mutex<Option<Arc<AdmittedBody>>>,
    poisoned: AtomicBool,
}
impl ModelingBodyRetention for WrongBodyOnce {
    fn generation(&self) -> u64 {
        0
    }
    fn get(
        &self,
        key: pse_ids::roles::SemanticBodyHash,
    ) -> std::result::Result<Option<Arc<AdmittedBody>>, MathError> {
        let Some(body) = self.body.lock().unwrap().clone() else {
            return Ok(None);
        };
        if body.semantic_identity() != Some(key) && !self.poisoned.swap(true, Ordering::Relaxed) {
            Ok(Some(body))
        } else {
            Ok(None)
        }
    }
    fn retain(
        &self,
        _: u64,
        _: pse_ids::roles::SemanticBodyHash,
        body: Arc<AdmittedBody>,
    ) -> std::result::Result<Arc<AdmittedBody>, MathError> {
        let mut first = self.body.lock().unwrap();
        if first.is_none() {
            *first = Some(body.clone());
        }
        Ok(body)
    }
}
#[test]
fn foreign_retention_cannot_supply_a_body_for_another_dependency_closure() {
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    workspace
        .attach_body_retention(Arc::new(WrongBodyOnce::default()))
        .unwrap();
    let declarations = rows("package p { def Root { var x:Scalar; eq a:x*x==1; eq b:x==2; } }");
    let root = declarations
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    workspace
        .publish_modeling(declarations, PhysicalScope::default())
        .unwrap();
    let request = |workspace: &mut CompilerWorkspace| {
        workspace.prepare_modeling_cancellable(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings::default(),
            Limits::default(),
            Arc::new(AtomicBool::new(false)),
        )
    };
    assert!(
        request(&mut workspace)
            .unwrap_err()
            .to_string()
            .contains("different immutable dependency closure")
    );
    assert!(
        request(&mut workspace).is_ok(),
        "foreign cache refusal is not memoized as admitted mathematics"
    );
}

fn plan_frontier(
    workspace: &mut CompilerWorkspace,
    root: DeclarationId,
) -> ModelingPreparationFrontier {
    workspace
        .plan_modeling_cancellable(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings::default(),
            Limits::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
}

fn publish_frontier_source(workspace: &mut CompilerWorkspace, text: &str) -> DeclarationId {
    let declarations = rows(text);
    let root = declarations
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    workspace
        .publish_modeling(declarations, PhysicalScope::default())
        .unwrap();
    root
}

#[derive(Debug, Default)]
struct PureFrontierRetention {
    bodies: std::sync::Mutex<BTreeMap<pse_ids::roles::SemanticBodyHash, Arc<AdmittedBody>>>,
    gets: AtomicUsize,
    retains: AtomicUsize,
}
impl ModelingBodyRetention for PureFrontierRetention {
    fn generation(&self) -> u64 {
        0
    }
    fn get(
        &self,
        key: pse_ids::roles::SemanticBodyHash,
    ) -> std::result::Result<Option<Arc<AdmittedBody>>, MathError> {
        self.gets.fetch_add(1, Ordering::Relaxed);
        Ok(self.bodies.lock().unwrap().get(&key).cloned())
    }
    fn retain(
        &self,
        _: u64,
        key: pse_ids::roles::SemanticBodyHash,
        body: Arc<AdmittedBody>,
    ) -> std::result::Result<Arc<AdmittedBody>, MathError> {
        self.retains.fetch_add(1, Ordering::Relaxed);
        self.bodies.lock().unwrap().insert(key, body.clone());
        Ok(body)
    }
}

#[test]
fn owned_frontier_plans_before_body_retention_and_completes_the_same_requests() {
    let executions = BodyObservations::new();
    let _entered = executions.enter();
    let retention = Arc::new(PureFrontierRetention::default());
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    workspace.attach_body_retention(retention.clone()).unwrap();
    let root = publish_frontier_source(
        &mut workspace,
        "package p { def Root { var x:Scalar; eq a:x*x==1; eq b:x==2; } }",
    );
    let frontier = plan_frontier(&mut workspace, root);
    let requests = frontier.body_requests().collect::<BTreeSet<_>>();
    assert!(!requests.is_empty());
    assert!(frontier.retained_bytes() > 0);
    assert_eq!(executions.admissions(), 0);
    assert_eq!(retention.gets.load(Ordering::Relaxed), 0);
    assert_eq!(retention.retains.load(Ordering::Relaxed), 0);
    let completed = workspace
        .complete_modeling_cancellable(frontier.clone(), Arc::new(AtomicBool::new(false)))
        .unwrap();
    let identities = completed
        .portable_bodies()
        .map(|body| body.semantic_identity().unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(identities, requests);
    assert_eq!(executions.admissions(), requests.len());
    assert_eq!(retention.retains.load(Ordering::Relaxed), requests.len());
    let again = workspace
        .complete_modeling_cancellable(frontier, Arc::new(AtomicBool::new(false)))
        .unwrap();
    assert_eq!(again.admitted.case(), completed.admitted.case());
    assert_eq!(again.portable_bodies().count(), requests.len());
    assert_eq!(
        executions.admissions(),
        requests.len(),
        "repeated completion admits no new bodies and preserves the portable inventory"
    );
    let ordinary = workspace
        .admit_modeling(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings::default(),
            Limits::default(),
        )
        .unwrap();
    assert_eq!(ordinary.as_ref(), completed.admitted.as_ref());
}

#[test]
fn owned_frontier_survives_rotation_and_completion_in_an_independent_workspace() {
    let mut source = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    let root = publish_frontier_source(
        &mut source,
        "package p { def Root { var x:Scalar; eq a:x*x==1; } }",
    );
    let frontier = plan_frontier(&mut source, root);
    let reference = source
        .complete_modeling_cancellable(frontier.clone(), Arc::new(AtomicBool::new(false)))
        .unwrap();
    let retention = Arc::new(PureFrontierRetention::default());
    for body in reference.portable_bodies() {
        retention
            .bodies
            .lock()
            .unwrap()
            .insert(body.semantic_identity().unwrap(), body.clone());
    }
    let generation = source.generation();
    source.rebuild(source.inputs.clone()).unwrap();
    assert!(source.generation() > generation);
    drop(source);
    let mut receiver = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    receiver.attach_body_retention(retention.clone()).unwrap();
    let completed = receiver
        .complete_modeling_cancellable(frontier, Arc::new(AtomicBool::new(false)))
        .unwrap();
    assert!(
        receiver.modeling.is_none(),
        "completion does not specialize a receiving catalog"
    );
    assert_eq!(completed.admitted.as_ref(), reference.admitted.as_ref());
    assert_eq!(
        retention.gets.load(Ordering::Relaxed),
        completed.portable_bodies().count()
    );
    assert_eq!(
        retention.retains.load(Ordering::Relaxed),
        0,
        "qualified owned bodies supply pure completion"
    );
    for body in completed.portable_bodies() {
        assert!(
            reference
                .portable_bodies()
                .any(|old| Arc::ptr_eq(old, body))
        );
    }
    drop(receiver);
    drop(retention);
    drop(reference);
    assert!(completed.retained_bytes() > 0);
    assert!(!completed.admitted.case().instances().is_empty());
    assert!(!completed.model.symbols.is_empty());
}

#[test]
fn owned_frontier_inventory_includes_promoted_original_and_retains_direct_implicit_bodies() {
    let executions = BodyObservations::new();
    let _entered = executions.enter();
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    let root = publish_frontier_source(
        &mut workspace,
        "package p { def Root { param p:Scalar=2; implicit a { var selected_value:Scalar; eq selected:selected_value==p; annotation start selected_value(1); annotation bounds selected_value(0,4); } realize ra on a using nested; var y:Scalar; eq e:y==a.selected_value; } }",
    );
    let frontier = plan_frontier(&mut workspace, root);
    let requests = frontier.body_requests().collect::<BTreeSet<_>>();
    assert_eq!(
        executions.admissions(),
        0,
        "planning performs no grouped semantic-body admission"
    );
    let completed = workspace
        .complete_modeling_cancellable(frontier.clone(), Arc::new(AtomicBool::new(false)))
        .unwrap();
    let original = completed.original_equations().unwrap();
    let primary = completed
        .admitted
        .bodies
        .values()
        .map(|body| body.semantic_identity().unwrap())
        .collect::<BTreeSet<_>>();
    let original_ids = original
        .admitted
        .bodies
        .values()
        .map(|body| body.semantic_identity().unwrap())
        .collect::<BTreeSet<_>>();
    assert!(
        original_ids
            .iter()
            .any(|identity| !primary.contains(identity)),
        "original equations have their own requested bodies"
    );
    assert_eq!(requests, primary.union(&original_ids).copied().collect());
    assert_eq!(
        requests,
        completed
            .portable_bodies()
            .map(|body| body.semantic_identity().unwrap())
            .collect()
    );
    assert_eq!(requests.len(), completed.portable_bodies().count());
    assert_eq!(
        original_ids,
        original
            .portable_bodies()
            .map(|body| body.semantic_identity().unwrap())
            .collect()
    );
    let first_inventory = completed
        .portable_bodies()
        .map(Arc::as_ptr)
        .collect::<Vec<_>>();
    assert_eq!(
        first_inventory,
        completed
            .clone()
            .portable_bodies()
            .map(Arc::as_ptr)
            .collect::<Vec<_>>()
    );
    assert!(
        completed
            .portable_bodies()
            .map(|body| body.semantic_identity().unwrap())
            .collect::<Vec<_>>()
            .windows(2)
            .all(|pair| pair[0] < pair[1])
    );
    let supplier = completed.admitted.implicit_systems().next().unwrap();
    assert!(supplier.retained_bytes() > 0);
    assert!(
        supplier
            .bodies()
            .all(|body| body.semantic_identity().is_none()),
        "direct residuals have no fabricated semantic portability identity"
    );
    let executed = executions.admissions();
    let again = workspace
        .complete_modeling_cancellable(frontier, Arc::new(AtomicBool::new(false)))
        .unwrap();
    assert_eq!(again.portable_bodies().count(), requests.len());
    assert_eq!(executions.admissions(), executed);
    assert_eq!(
        again.original_equations().unwrap().admitted.as_ref(),
        original.admitted.as_ref()
    );
}

#[test]
fn owned_frontier_completion_uses_consumed_inventory_and_cancellation_is_retryable() {
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    let root = publish_frontier_source(
        &mut workspace,
        "package p { def Root { var x:Scalar; eq a:x*x==1; } }",
    );
    let frontier = plan_frontier(&mut workspace, root);
    assert!(matches!(
        workspace.complete_modeling_cancellable(frontier.clone(), Arc::new(AtomicBool::new(true))),
        Err(CompileError::Cancelled)
    ));
    workspace
        .complete_modeling_cancellable(frontier.clone(), Arc::new(AtomicBool::new(false)))
        .unwrap();
    let mut changed = workspace.inputs.clone();
    let mut physical = changed.quantities.to_builder();
    physical.entity_kind(pse_quantity::EntityKind {
        id: pse_quantity::EntityKindId::from_id(SemanticId::from_bytes([243; 16])),
        name: "frontier_other_kind".into(),
    });
    changed.quantities = Arc::new(physical.build().unwrap());
    let mut receiver = CompilerWorkspace::new(changed.clone(), WorkspaceLimits::default()).unwrap();
    let received = receiver
        .complete_modeling_cancellable(frontier.clone(), Arc::new(AtomicBool::new(false)))
        .unwrap();
    assert!(Arc::ptr_eq(
        &received.compiler_context().quantities,
        &changed.quantities
    ));
    let scalar = serde_json::to_value(changed.quantities.neutral_dimensionless().unwrap()).unwrap();
    let mut declared = serde_json::to_value(changed.quantities.to_builder()).unwrap();
    let declaration = declared["quantity_types"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|declaration| declaration["id"] == scalar)
        .unwrap();
    declaration["nominal_magnitude"] = serde_json::json!(17.0);
    let declarations: pse_quantity::QuantityRegistryBuilder =
        serde_json::from_value(declared).unwrap();
    changed.quantities = Arc::new(declarations.build().unwrap());
    let mut receiver = CompilerWorkspace::new(changed, WorkspaceLimits::default()).unwrap();
    let error = receiver
        .complete_modeling_cancellable(frontier.clone(), Arc::new(AtomicBool::new(false)))
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("different immutable compiler inventory")
    );
    workspace
        .inventory
        .set_environment(&mut workspace.db)
        .to(ContentHash::from_bytes([255; 32]));
    let error = workspace
        .complete_modeling_cancellable(frontier, Arc::new(AtomicBool::new(false)))
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("different immutable compiler inventory")
    );
}

#[derive(Debug)]
struct RefuseFrontierLookupOnce(AtomicBool);
impl ModelingBodyRetention for RefuseFrontierLookupOnce {
    fn generation(&self) -> u64 {
        0
    }
    fn get(
        &self,
        _: pse_ids::roles::SemanticBodyHash,
    ) -> std::result::Result<Option<Arc<AdmittedBody>>, MathError> {
        if self.0.swap(false, Ordering::Relaxed) {
            Err(MathError::Contract("transient pure lookup refusal".into()))
        } else {
            Ok(None)
        }
    }
    fn retain(
        &self,
        _: u64,
        _: pse_ids::roles::SemanticBodyHash,
        body: Arc<AdmittedBody>,
    ) -> std::result::Result<Arc<AdmittedBody>, MathError> {
        Ok(body)
    }
}

#[test]
fn owned_frontier_retries_the_same_request_after_a_pure_retention_lookup_refusal() {
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    workspace
        .attach_body_retention(Arc::new(RefuseFrontierLookupOnce(AtomicBool::new(true))))
        .unwrap();
    let root = publish_frontier_source(
        &mut workspace,
        "package p { def Root { var x:Scalar; eq a:x*x==1; } }",
    );
    let frontier = plan_frontier(&mut workspace, root);
    let error = workspace
        .complete_modeling_cancellable(frontier.clone(), Arc::new(AtomicBool::new(false)))
        .unwrap_err();
    assert!(error.to_string().contains("transient pure lookup refusal"));
    workspace
        .complete_modeling_cancellable(frontier, Arc::new(AtomicBool::new(false)))
        .unwrap();
}

#[test]
fn observation_distinguishes_value_product_reuse_from_consumed_value_change() {
    use pse_diagnostics::observation::{ObservationHub, Policy, Value};
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    let source =
        rows("package p { def Root { param p: Scalar = 2; var x: Scalar; eq e: x*p == 6; } }");
    let root = source
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    workspace
        .publish_modeling(source, PhysicalScope::default())
        .unwrap();
    let model = prepare(
        &mut workspace,
        root,
        Bindings {
            demand: vec!["p".into(), "x".into()],
            ..Bindings::default()
        },
    );
    let (x, p) = (model.model.paths["x"], model.model.paths["p"]);
    let values = |x_value, p_value| CaseValues {
        scalars: BTreeMap::from([(x, x_value), (p, p_value)]),
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let first = workspace
        .prepare_modeling_view(
            &model,
            model.bound_structure(&BTreeMap::new()).unwrap().structure,
            &values(1.0, 2.0),
            DerivativeOrder::First,
            Profile::default(),
            &cancel,
        )
        .unwrap();
    let hub = ObservationHub::new(Policy::default());
    let operation = hub.start("rebind");
    let context = operation.context();
    let _entered = context.enter();
    let start = first.rebind(&values(5.0, 2.0), &cancel).unwrap();
    assert!(Arc::ptr_eq(&first.plan, &start.plan));
    assert!(pse_math::SharedAllocation::ptr_eq(
        &first.presolve,
        &start.presolve
    ));
    let changed = first.rebind(&values(1.0, 3.0), &cancel).unwrap();
    assert!(Arc::ptr_eq(&first.plan, &changed.plan));
    assert!(!pse_math::SharedAllocation::ptr_eq(
        &first.presolve,
        &changed.presolve
    ));
    let snapshot = context
        .reader()
        .snapshot(std::time::Duration::from_secs(60));
    let count = |name: &str| {
        snapshot
            .metrics
            .iter()
            .find(|metric| metric.name == name)
            .map(|metric| metric.cumulative.clone())
    };
    assert_eq!(count("compiler.value_rebinds"), Some(Value::Count(2)));
    assert_eq!(
        count("compiler.value_rebinds_completed"),
        Some(Value::Count(2))
    );
    assert_eq!(
        count("compiler.value_product_reuses"),
        Some(Value::Count(1))
    );
    assert_eq!(
        count("compiler.product_consumed_value_changes"),
        Some(Value::Count(1))
    );
    assert_eq!(count("math.compiled_body_constructions"), None);
}

#[test]
fn bound_lookup_preserves_complete_public_view_identity_and_exact_attribution() {
    use pse_kernels::DerivativeOrder;
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    let source = "package p {def Root {var x:Scalar; eq e:x==2;}}";
    let declarations = rows(source);
    let root = declarations
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    workspace
        .publish_modeling(declarations, PhysicalScope::default())
        .unwrap();
    let model = prepare(&mut workspace, root, Bindings::default());
    let x = model
        .model
        .symbols
        .values()
        .find(|symbol| symbol.lineage.path.ends_with(".x"))
        .unwrap()
        .id;
    let context = ContentHash::from_bytes([61; 32]);
    let profile = Profile::default();
    let empty = BTreeMap::new();
    let lookup = model
        .bound_lookup(&empty, DerivativeOrder::First, profile, &context)
        .unwrap();
    let original_key = lookup.key.clone();
    let expanded = model.expand_bound_lookup(lookup).unwrap();
    let direct = model.bound_structure(&empty).unwrap();
    assert_eq!(expanded.structure.key(), direct.structure.key());
    assert_eq!(
        model.view_key(
            &expanded.structure,
            DerivativeOrder::First,
            profile,
            &context
        ),
        model.view_key(&direct.structure, DerivativeOrder::First, profile, &context)
    );
    for state in [
        ModelingVariableState {
            fixed: Some(true),
            ..Default::default()
        },
        ModelingVariableState {
            lower: Some(Some(0.)),
            upper: Some(Some(9.)),
            ..Default::default()
        },
    ] {
        let states = BTreeMap::from([(x, state)]);
        assert_ne!(
            model
                .bound_lookup(&states, DerivativeOrder::First, profile, &context)
                .unwrap()
                .key,
            original_key
        );
    }
    let mut changed_profile = profile;
    changed_profile.class_proof_work += 1;
    assert_ne!(
        model
            .bound_lookup(&empty, DerivativeOrder::First, changed_profile, &context)
            .unwrap()
            .key,
        original_key
    );
    assert_ne!(
        model
            .bound_lookup(&empty, DerivativeOrder::Value, profile, &context)
            .unwrap()
            .key,
        original_key
    );
    assert_ne!(
        model
            .bound_lookup(
                &empty,
                DerivativeOrder::First,
                profile,
                &ContentHash::from_bytes([62; 32])
            )
            .unwrap()
            .key,
        original_key
    );
    for bounds in [(f64::NAN, 1.), (2., 1.), (0., f64::INFINITY)] {
        let state = ModelingVariableState {
            lower: Some(Some(bounds.0)),
            upper: Some(Some(bounds.1)),
            ..Default::default()
        };
        assert!(
            model
                .bound_lookup(
                    &BTreeMap::from([(x, state)]),
                    DerivativeOrder::First,
                    profile,
                    &context
                )
                .is_err()
        );
    }
    workspace
        .publish_modeling(rows(&format!("\n\n{source}")), PhysicalScope::default())
        .unwrap();
    let shifted = prepare(&mut workspace, root, Bindings::default());
    assert_eq!(
        shifted
            .bound_lookup(&empty, DerivativeOrder::First, profile, &context)
            .unwrap()
            .key,
        original_key
    );
    assert_ne!(shifted.occurrence_identity(), model.occurrence_identity());
    assert!(Arc::ptr_eq(
        &model.shared_occurrences(),
        &model.shared_occurrences()
    ));
    let mut relocated = rows(source);
    for row in &mut relocated {
        row.document_id = SemanticId::from_bytes([93; 16]);
    }
    workspace
        .publish_modeling(relocated, PhysicalScope::default())
        .unwrap();
    let relocated = prepare(&mut workspace, root, Bindings::default());
    assert_eq!(
        relocated
            .bound_lookup(&empty, DerivativeOrder::First, profile, &context)
            .unwrap()
            .key,
        original_key
    );
    assert_eq!(
        relocated.occurrences(),
        model.occurrences(),
        "offset-only map alone cannot establish document identity"
    );
    assert_ne!(relocated.occurrence_identity(), model.occurrence_identity());
    workspace
        .publish_modeling(
            rows(&source.replace("x==2", "x*x==3")),
            PhysicalScope::default(),
        )
        .unwrap();
    let edited = prepare(&mut workspace, root, Bindings::default());
    assert_ne!(
        edited
            .bound_lookup(&empty, DerivativeOrder::First, profile, &context)
            .unwrap()
            .key,
        original_key
    );
}

#[test]
fn reference_three_units_retains_direct_bodies_and_backdates_unrelated_edits() {
    let body_observations = BodyObservations::new();
    let _entered = body_observations.enter();
    let (mut workspace, counts) = observed_workspace(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    );
    let reference = format!(
        "{}\n{}\npackage unrelated {{ def Other {{ let value:Scalar=1; }} }}",
        crate::authored_transfer_tests::reference_family_sources("process-reference"),
        crate::process_composition_tests::COMPOSE
    );
    let declarations = crate::authored_transfer_tests::rows(&reference);
    let root =
        crate::authored_transfer_tests::root(&declarations, "process_composition", "ThreeUnits");
    workspace
        .publish_modeling(declarations, PhysicalScope::default())
        .unwrap();
    let generation = workspace.generation();
    assert_eq!(
        generation,
        0,
        "ordinary reference publication retains its generation; retained={:?}",
        {
            let s = workspace.observe_retention();
            (s.entries, s.known_bytes)
        }
    );
    let prepare_reference = |workspace: &mut CompilerWorkspace| {
        workspace
            .prepare_modeling_cancellable(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default().with_analysis(pse_modeling::analysis::Route::Steady),
                Limits::default(),
                Arc::new(AtomicBool::new(false)),
            )
            .unwrap()
    };
    let first = prepare_reference(&mut workspace);
    // The historical textual-normalizer population was 143. The direct producer
    // has 126 exact scalar products; complete current row/instance coverage is checked below.
    assert_eq!(first.admitted.bodies.len(), 126);
    assert_eq!(first.admitted.case.rows().len(), 856);
    assert_eq!(first.admitted.case.instances().len(), 845);
    let rows = first
        .admitted
        .case
        .rows()
        .iter()
        .map(|row| row.id)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        rows,
        first
            .admitted
            .outputs
            .iter()
            .map(ModelingOutput::row_id)
            .chain(
                first
                    .admitted
                    .term_outputs
                    .values()
                    .flatten()
                    .map(|(id, _)| *id)
            )
            .collect()
    );
    let mut contributed = BTreeSet::new();
    let mut bodies = BTreeSet::new();
    let mut instances = BTreeSet::new();
    let ports = first
        .admitted
        .case
        .variables()
        .iter()
        .map(|value| value.port.id)
        .chain(first.admitted.case.parameters().iter().map(|port| port.id))
        .collect::<BTreeSet<_>>();
    for instance in first.admitted.case.instances() {
        assert!(
            instances.insert(instance.instance),
            "each actual selected output has a distinct receiving instance"
        );
        let body = &first.admitted.bodies[&instance.body];
        bodies.insert(instance.body);
        assert_eq!(instance.slots.len(), body.math.input_count());
        assert!(
            instance
                .slots
                .iter()
                .all(|slot| ports.contains(&slot.source()))
        );
        pse_math::SourceBindings::new(
            instance.instance,
            &body.math,
            (*instance.checked_members).clone(),
            (*instance.checked_sources).clone(),
            (*instance.occurrences).clone(),
        )
        .unwrap();
        for contribution in &instance.contributions {
            assert!(contribution.output < body.math.output_count());
            if let Target::Row(row) = contribution.target {
                assert!(rows.contains(&row));
                contributed.insert(row);
            }
        }
    }
    assert_eq!(
        contributed, rows,
        "every original projected output is assembled"
    );
    assert_eq!(
        bodies,
        first.admitted.bodies.keys().copied().collect(),
        "every retained body is actually reached"
    );
    let first_counts = [
        counts.count(Query::Selected),
        counts.count(Query::Specialized),
        counts.count(Query::Projection),
        body_observations.admissions(),
    ];
    assert_eq!(
        first_counts[3], 126,
        "all distinct direct bodies were admitted"
    );
    let warm = prepare_reference(&mut workspace);
    assert_eq!(body_observations.admissions(), first_counts[3]);
    workspace
        .publish_modeling(
            crate::authored_transfer_tests::rows(
                &reference.replace("let value:Scalar=1", "let value:Scalar=2"),
            ),
            PhysicalScope::default(),
        )
        .unwrap();
    let unrelated = prepare_reference(&mut workspace);
    assert_eq!(workspace.generation(), generation);
    for (actual, expected) in [
        counts.count(Query::Specialized),
        counts.count(Query::Projection),
    ]
    .into_iter()
    .zip(&first_counts[1..3])
    {
        assert_eq!(
            actual, *expected,
            "selected firewall backdates preparation queries"
        );
    }
    assert_eq!(
        body_observations.admissions(),
        first_counts[3],
        "unrelated edits require no body readmission"
    );
    for (key, body) in &first.admitted.bodies {
        assert!(Arc::ptr_eq(&body.math, &warm.admitted.bodies[key].math));
        assert!(Arc::ptr_eq(
            &body.math,
            &unrelated.admitted.bodies[key].math
        ));
    }
    let (entries, bytes) = {
        let s = workspace.observe_retention();
        (s.entries, s.known_bytes)
    };
    assert!(entries <= workspace.limits.retained_entries, "{entries}");
    assert!(bytes <= workspace.limits.retained_bytes, "{bytes}");
}

#[test]
fn owned_frontier_completion_reclaims_history_under_unchanged_receiving_limits() {
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    let retention = Arc::new(PureFrontierRetention::default());
    workspace.attach_body_retention(retention).unwrap();
    let root = publish_frontier_source(
        &mut workspace,
        "package p {def Root {var x:Scalar; eq e:x*x==1;}}",
    );
    let escaped = prepare(&mut workspace, root, Bindings::default());
    let frontier = plan_frontier(&mut workspace, root);
    let incoming_bytes = frontier.retained_bytes();
    let incoming_count = frontier.body_requests().count();
    let mut receiver = CompilerWorkspace::new(workspace.inputs.clone(), workspace.limits).unwrap();
    receiver
        .publish_modeling_revision(workspace.modeling.as_ref().unwrap().revision.clone())
        .unwrap();
    let fresh_bytes = receiver.observe_retention().known_bytes;
    for n in 1..=512u64 {
        let mut id = [0; 16];
        id[..8].copy_from_slice(&n.to_le_bytes());
        workspace
            .modeling_request(
                root,
                InstanceId::from_id(SemanticId::from_bytes(id)),
                Bindings::default(),
                Limits::default(),
            )
            .unwrap();
    }
    let resident = workspace.observe_retention().known_bytes;
    assert!(resident > fresh_bytes + incoming_bytes);
    workspace.limits.retained_bytes = resident + incoming_bytes / 2;
    let limits = workspace.limits;
    assert!(resident < limits.retained_bytes);
    assert!(resident + incoming_bytes > limits.retained_bytes);
    assert!(fresh_bytes + incoming_bytes < limits.retained_bytes);
    assert!(incoming_count.max(limits.query_values) <= limits.retained_entries);
    let generation = workspace.generation();
    assert!(matches!(
        workspace.complete_modeling_cancellable(frontier.clone(), Arc::new(AtomicBool::new(true))),
        Err(CompileError::Cancelled)
    ));
    assert_eq!(workspace.generation(), generation);
    let received = workspace
        .complete_modeling_cancellable(frontier, Arc::new(AtomicBool::new(false)))
        .unwrap();
    assert_eq!(workspace.generation(), generation + 1);
    assert_eq!(workspace.limits.retained_bytes, limits.retained_bytes);
    assert_eq!(workspace.limits.retained_entries, limits.retained_entries);
    assert_eq!(received.admitted.as_ref(), escaped.admitted.as_ref());
    assert_eq!(received.occurrences(), escaped.occurrences());
    for (key, body) in &escaped.admitted.bodies {
        assert!(Arc::ptr_eq(body, &received.admitted.bodies[key]));
    }
}

#[test]
fn owned_frontier_completion_counts_mandatory_receiving_inventory() {
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    let root = publish_frontier_source(
        &mut workspace,
        "package p {def Root {var x:Scalar; eq e:x*x==1;}}",
    );
    let frontier = plan_frontier(&mut workspace, root);
    let incoming_count = frontier.body_requests().count();
    let mut receiver = CompilerWorkspace::new(workspace.inputs.clone(), workspace.limits).unwrap();
    receiver
        .publish_modeling_revision(workspace.modeling.as_ref().unwrap().revision.clone())
        .unwrap();
    let mandatory = receiver.observe_retention();
    assert!(mandatory.entries > incoming_count + 1);
    // Body requests fit, and bytes are unconstrained here; the receiver's
    // mandatory Inventory/Catalog/input entries alone prevent count admission.
    workspace.limits.retained_entries = mandatory.entries - 1;
    workspace.limits.query_values = 1;
    assert!(frontier.retained_bytes() + mandatory.known_bytes < workspace.limits.retained_bytes);
    let generation = workspace.generation();
    assert!(matches!(
        workspace.complete_modeling_cancellable(frontier, Arc::new(AtomicBool::new(false))),
        Err(CompileError::Limit("live root working set"))
    ));
    assert_eq!(workspace.generation(), generation + 1);
}

#[test]
fn owned_frontier_completion_refuses_new_mandatory_entries_without_second_rotation() {
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    let root = publish_frontier_source(
        &mut workspace,
        "package p {def Root {var x:Scalar; eq e:x*x==1;}}",
    );
    let frontier = plan_frontier(&mut workspace, root);
    let incoming_count = frontier.body_requests().count();
    let mut receiver = CompilerWorkspace::new(workspace.inputs.clone(), workspace.limits).unwrap();
    receiver
        .publish_modeling_revision(workspace.modeling.as_ref().unwrap().revision.clone())
        .unwrap();
    let mandatory = receiver.observe_retention();
    // Fresh preflight fits exactly. Actual completion additionally owns live
    // indexes and non-evictable body keys; those cannot justify another retry.
    workspace.limits.query_values = 1;
    workspace.limits.retained_entries = mandatory.entries + incoming_count;
    assert!(workspace.observe_retention().entries > workspace.limits.retained_entries);
    let generation = workspace.generation();
    assert!(matches!(
        workspace.complete_modeling_cancellable(frontier, Arc::new(AtomicBool::new(false))),
        Err(CompileError::Limit("live root working set"))
    ));
    assert_eq!(workspace.generation(), generation + 1);
}

#[test]
fn owned_frontier_completion_preserves_typed_refusal_after_receiving_rotation() {
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    workspace
        .attach_body_retention(Arc::new(RefuseFrontierLookupOnce(AtomicBool::new(true))))
        .unwrap();
    let root = publish_frontier_source(
        &mut workspace,
        "package p {def Root {var x:Scalar; eq e:x*x==1;}}",
    );
    let frontier = plan_frontier(&mut workspace, root);
    let mut receiver = CompilerWorkspace::new(workspace.inputs.clone(), workspace.limits).unwrap();
    receiver
        .publish_modeling_revision(workspace.modeling.as_ref().unwrap().revision.clone())
        .unwrap();
    workspace.limits.query_values = 1;
    workspace.limits.retained_entries = receiver.observe_retention().entries + frontier.body_requests().count();
    let generation = workspace.generation();
    let error = workspace
        .complete_modeling_cancellable(frontier, Arc::new(AtomicBool::new(false)))
        .unwrap_err();
    assert!(error.to_string().contains("transient pure lookup refusal"));
    assert_eq!(workspace.generation(), generation + 1);
    assert!(workspace.observe_retention().entries > workspace.limits.retained_entries);
}

#[derive(Debug)]
struct RequireReceivingRotation(pse_diagnostics::observation::ObservationContext);
impl ModelingBodyRetention for RequireReceivingRotation {
    fn generation(&self) -> u64 {
        0
    }
    fn get(
        &self,
        _: pse_ids::roles::SemanticBodyHash,
    ) -> std::result::Result<Option<Arc<AdmittedBody>>, MathError> {
        assert_eq!(
            super::test_observation::metric_count(&self.0, "compiler.workspace_rebuilds"),
            1,
            "receiving admission must discard excess history before body acquisition"
        );
        Ok(None)
    }
    fn retain(
        &self,
        _: u64,
        _: pse_ids::roles::SemanticBodyHash,
        body: Arc<AdmittedBody>,
    ) -> std::result::Result<Arc<AdmittedBody>, MathError> {
        Ok(body)
    }
}

#[test]
fn owned_frontier_completion_reclaims_resident_count_without_byte_pressure() {
    use pse_diagnostics::observation::{ObservationHub, Policy};
    let hub = ObservationHub::new(Policy::default());
    let operation = hub.start("count-only receiving admission");
    let context = operation.context();
    context.count("compiler.workspace_rebuilds", "count", 0);
    let _entered = context.enter();
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    workspace
        .attach_body_retention(Arc::new(RequireReceivingRotation(context)))
        .unwrap();
    let root = publish_frontier_source(
        &mut workspace,
        "package p {def Root {var x:Scalar; eq e:x*x==1;}}",
    );
    let frontier = plan_frontier(&mut workspace, root);
    let incoming_count = frontier.body_requests().count();
    for n in 1..=512u64 {
        let mut id = [0; 16];
        id[..8].copy_from_slice(&n.to_le_bytes());
        workspace
            .modeling_request(
                root,
                InstanceId::from_id(SemanticId::from_bytes(id)),
                Bindings::default(),
                Limits::default(),
            )
            .unwrap();
    }
    let resident = workspace.observe_retention();
    workspace.limits.retained_entries = resident.entries;
    assert!(resident.entries + incoming_count > workspace.limits.retained_entries);
    assert!(resident.known_bytes + frontier.retained_bytes() < workspace.limits.retained_bytes);
    let limits = workspace.limits;
    let generation = workspace.generation();
    workspace
        .complete_modeling_cancellable(frontier, Arc::new(AtomicBool::new(false)))
        .unwrap();
    assert_eq!(workspace.generation(), generation + 1);
    assert_eq!(workspace.limits.retained_entries, limits.retained_entries);
    assert_eq!(workspace.limits.retained_bytes, limits.retained_bytes);
    assert!(workspace.observe_retention().entries <= limits.retained_entries);
}

#[test]
fn owned_frontier_completion_retries_only_once_and_preserves_wrong_environment_refusal() {
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    let root = publish_frontier_source(
        &mut workspace,
        "package p {def Root {var x:Scalar; eq e:x*x==1;}}",
    );
    let frontier = plan_frontier(&mut workspace, root);
    // The frontier itself fits, but its mandatory receiving inventory does not.
    workspace.limits.retained_bytes = frontier.retained_bytes() + 1;
    let generation = workspace.generation();
    let environment = *workspace.inventory.environment(&workspace.db);
    workspace
        .inventory
        .set_environment(&mut workspace.db)
        .to(ContentHash::from_bytes([255; 32]));
    let error = workspace
        .complete_modeling_cancellable(frontier.clone(), Arc::new(AtomicBool::new(false)))
        .unwrap_err();
    assert!(error.to_string().contains("different immutable compiler inventory"));
    assert_eq!(workspace.generation(), generation);
    workspace
        .inventory
        .set_environment(&mut workspace.db)
        .to(environment);
    assert!(matches!(
        workspace.complete_modeling_cancellable(frontier, Arc::new(AtomicBool::new(false))),
        Err(CompileError::Limit("live root working set"))
    ));
    assert_eq!(workspace.generation(), generation + 1);
}

#[test]
fn owned_frontier_completion_does_not_rotate_for_an_oversize_incoming_count() {
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    let root = publish_frontier_source(
        &mut workspace,
        "package p {def Root {var x:Scalar; eq e:x*x==1; eq f:x==2;}}",
    );
    let frontier = plan_frontier(&mut workspace, root);
    assert!(frontier.body_requests().count() > 1);
    workspace.limits.query_values = 1;
    workspace.limits.retained_entries = 1;
    let generation = workspace.generation();
    assert!(matches!(
        workspace.complete_modeling_cancellable(frontier.clone(), Arc::new(AtomicBool::new(false))),
        Err(CompileError::Limit("live root working set"))
    ));
    assert_eq!(workspace.generation(), generation);
    workspace.limits.retained_entries = frontier.body_requests().count();
    workspace.limits.query_values = workspace.limits.retained_entries + 1;
    assert!(matches!(
        workspace.complete_modeling_cancellable(frontier, Arc::new(AtomicBool::new(false))),
        Err(CompileError::Limit("live root working set"))
    ));
    assert_eq!(workspace.generation(), generation);
}

#[test]
fn live_root_admission_refuses_oversize_without_invalidating_escaped_products() {
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    let root = publish_frontier_source(
        &mut workspace,
        "package p {def Root {var x:Scalar; eq e:x*x==1;}}",
    );
    let first = prepare(&mut workspace, root, Bindings::default());
    let frontier = plan_frontier(&mut workspace, root);
    workspace.limits.retained_bytes = frontier.retained_bytes() - 1;
    let generation = workspace.generation();
    let result =
        workspace.complete_modeling_cancellable(frontier, Arc::new(AtomicBool::new(false)));
    assert!(matches!(
        result,
        Err(CompileError::Limit("live root working set"))
    ));
    assert_eq!(workspace.generation(), generation);
    assert!(!first.admitted.bodies.is_empty());
}

#[test]
fn generation_bounds_survive_edit_deletion_and_root_switch_churn() {
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits {
            retained_entries: 600,
            granular_query_values: 16,
            query_values: 4,
            ..WorkspaceLimits::default()
        },
    )
    .unwrap();
    let mut escaped = Vec::new();
    for n in 0..80 {
        let text = format!(
            "package p {{ def Root {{var x:Scalar; let v{n}:Scalar=x*{n}; eq e:x=={n};}} def Other {{var y:Scalar; eq f:y==1;}} }}"
        );
        let declarations = rows(&text);
        let root = crate::authored_transfer_tests::root(
            &declarations,
            "p",
            if n % 2 == 0 { "Root" } else { "Other" },
        );
        workspace
            .publish_modeling(declarations, PhysicalScope::default())
            .unwrap();
        let product = prepare(&mut workspace, root, Bindings::default());
        if n % 20 == 0 {
            escaped.push(product);
        }
        let (entries, bytes) = {
            let s = workspace.observe_retention();
            (s.entries, s.known_bytes)
        };
        assert!(
            entries <= workspace.limits.retained_entries,
            "iteration {n}: {entries}"
        );
        assert!(bytes <= workspace.limits.retained_bytes);
    }
    assert!(
        workspace.generation() > 0,
        "non-evictable metadata eventually rotates"
    );
    assert!(
        escaped
            .iter()
            .all(|product| !product.admitted.bodies.is_empty())
    );
}

#[test]
fn receiving_model_refreshes_worker_error_locations_while_sharing_numerical_programs() {
    use pse_model::diagnostic::DiagnosticProjection;
    let source = "package p {def Root {var x:Scalar; eq e:1/x==1;}}";
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    let root = publish_frontier_source(&mut workspace, source);
    let first = prepare(&mut workspace, root, Bindings::default());
    let x = first
        .model
        .symbols
        .values()
        .find(|symbol| symbol.lineage.path.ends_with(".x"))
        .unwrap()
        .id;
    let values = CaseValues {
        scalars: BTreeMap::from([(x, 1.0)]),
    };
    let cancel = pse_diagnostics::Cancellation::default();
    let profile = Profile::default();
    let view = workspace
        .prepare_modeling_view(
            &first,
            first.bound_structure(&BTreeMap::new()).unwrap().structure,
            &values,
            DerivativeOrder::First,
            profile,
            &cancel.flag(),
        )
        .unwrap();
    let assembly = Arc::new(
        view.plan
            .compile(
                profile.optimization,
                profile.evaluation,
                &pse_kernels::ExecutionScope::new(cancel.clone(), None),
            )
            .unwrap(),
    );
    let invalid = CaseValues {
        scalars: BTreeMap::from([(x, 0.0)]),
    };
    let old = assembly
        .worker(pse_kernels::ProviderBindings::new(), cancel.clone())
        .constraints(&invalid)
        .unwrap_err()
        .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Evaluation);
    assert!(!old.locations.is_empty());
    workspace
        .publish_modeling(rows(&format!("\n\n{source}")), PhysicalScope::default())
        .unwrap();
    let current = prepare(&mut workspace, root, Bindings::default());
    let received = view.receive_modeling_context(&current).unwrap();
    let refreshed = Arc::new(
        assembly
            .receive_source_context(received.plan.clone())
            .unwrap(),
    );
    for (old, new) in assembly.programs().iter().zip(refreshed.programs()) {
        assert!(Arc::ptr_eq(old, new));
    }
    for (key, body) in view.plan.bodies() {
        assert!(Arc::ptr_eq(body, &received.plan.bodies()[key]));
    }
    let diagnostic = refreshed
        .worker(pse_kernels::ProviderBindings::new(), cancel)
        .constraints(&invalid)
        .unwrap_err()
        .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Evaluation);
    assert_eq!(diagnostic.sources, old.sources);
    assert!(!diagnostic.locations.is_empty());
    let actual = current
        .admitted
        .case
        .instances()
        .iter()
        .flat_map(|binding| binding.occurrences.values())
        .collect::<Vec<_>>();
    for location in &diagnostic.locations {
        assert!(
            actual.contains(&location),
            "worker must use the current admitted occurrence owner"
        );
        assert!(
            !old.locations.contains(location),
            "the cached program must not retain the old source range"
        );
    }
}

#[test]
fn retention_observation_is_exclusive_owned_and_explicitly_fresh() {
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    assert_eq!(workspace.retention_snapshot(), None);
    let first = workspace.observe_retention();
    assert!(first.entries > 0);
    assert!(first.metadata_bytes > 0);
    workspace.db.synthetic_write(salsa::Durability::LOW);
    // Read-only diagnostics expose the old observation rather than cancelling readers.
    assert_eq!(workspace.retention_snapshot(), Some(first));
    assert_eq!(
        workspace.retention_usage(),
        Some((first.entries, first.known_bytes))
    );
    assert_eq!(workspace.metadata_bytes(), Some(first.metadata_bytes));
    let next = workspace.observe_retention();
    assert_ne!(next.revision, first.revision);
    assert_eq!(next.observation, first.observation + 1);
    assert_eq!(next.generation, workspace.generation());
    // Admission refreshes even when diagnostics were already available.
    workspace.admit_live_root(0, 0).unwrap();
    assert!(workspace.retention_snapshot().unwrap().observation > next.observation);
}

#[test]
fn exclusive_retention_observation_cancels_and_drains_cloned_reader() {
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    let reader = workspace.db.clone();
    let (ready, started) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        ready.send(()).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while salsa::Cancelled::catch(std::panic::AssertUnwindSafe(|| {
            reader.unwind_if_revision_cancelled();
        }))
        .is_ok()
        {
            assert!(
                std::time::Instant::now() < deadline,
                "observation did not cancel reader"
            );
            std::thread::yield_now();
        }
        drop(reader);
    });
    started.recv().unwrap();
    let observed = workspace.observe_retention();
    worker.join().unwrap();
    assert_eq!(workspace.retention_snapshot(), Some(observed));
}

#[test]
fn observation_demand_reuses_structure_but_new_let_demand_changes_it() {
    let (mut workspace, executions) = observed_workspace(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    );
    let declarations =
        rows("package p {def Root {var x:Scalar; var y:Scalar; let lazy:Scalar=x+y; eq e:x==1;}}");
    let root = declarations
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    workspace
        .publish_modeling(declarations, PhysicalScope::default())
        .unwrap();
    let instance = InstanceId::from_id(SemanticId::NIL);
    let specialize = |workspace: &mut CompilerWorkspace, paths: &[&str]| {
        workspace
            .specialize_modeling(
                root,
                instance,
                Bindings {
                    demand: paths.iter().map(|path| (*path).to_owned()).collect(),
                    ..Bindings::default()
                },
                Limits::default(),
            )
            .unwrap()
    };
    let base = specialize(&mut workspace, &[]);
    assert_eq!(executions.count(Query::Specialized), 1);
    let x = specialize(&mut workspace, &["x"]);
    let y = specialize(&mut workspace, &["y"]);
    let x_again = specialize(&mut workspace, &["x"]);
    assert!(
        Arc::ptr_eq(&x, &x_again),
        "A/B/A reuses the exact observation product"
    );
    assert_eq!(executions.count(Query::Specialized), 1);
    assert_eq!(x.symbols, base.symbols);
    assert_eq!(y.symbols, base.symbols);
    assert!(x.paths.contains_key("x"));
    assert!(!x.paths.contains_key("y"));
    assert!(y.paths.contains_key("y"));
    let lazy = specialize(&mut workspace, &["lazy"]);
    assert_eq!(executions.count(Query::Specialized), 2);
    assert_eq!(lazy.symbols.len(), base.symbols.len() + 1);
    assert!(lazy.paths.contains_key("lazy"));
}

#[test]
fn instance_prelude_is_local_to_membership_and_static_meaning() {
    let (mut workspace, executions) = observed_workspace(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    );
    let original = "package p {def Root {var x:Scalar; eq a:x==1;}}";
    let mut root = None;
    for (source, expected) in [
        (original, 1),
        ("package p {def Root {var x:Scalar; eq a:x==2;}}", 1),
        (
            "package p {def Root {var x:Scalar; eq a:x==2; eq b:x==3;}}",
            2,
        ),
        (original, 2),
    ] {
        let declarations = rows(source);
        let selected_root = declarations
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        assert_eq!(*root.get_or_insert(selected_root), selected_root);
        let revision = workspace
            .publish_modeling(declarations, PhysicalScope::default())
            .unwrap();
        let bindings = Bindings::default();
        let instance = pse_modeling::specialize::root_instance(selected_root);
        let incremental = workspace
            .specialize_modeling(selected_root, instance, bindings.clone(), Limits::default())
            .unwrap();
        let clean = pse_modeling::specialize::specialize_with_discretizer(
            revision.checked(),
            selected_root,
            instance,
            &bindings,
            Limits::default(),
            &|| false,
            &LibraryDiscretizer,
        )
        .unwrap();
        assert_eq!(incremental.as_ref(), &clean);
        assert_eq!(executions.count(Query::InstancePrelude), expected);
    }
}

#[test]
fn typed_function_fragments_reuse_local_meaning_and_refresh_source_attribution() {
    let (mut workspace, executions) = observed_workspace(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    );
    let original =
        "package p {fn law(x:Scalar)->Scalar=x*x; def Root {var x:Scalar;eq e:law(x)==1;}}";
    for (source, expected) in [
        (original.to_owned(), 1),
        (original.replace("==1", "==2"), 1),
        (original.replace("x*x", "x+x"), 2),
        (format!("\n{original}"), 3),
    ] {
        let declarations = rows(&source);
        let root = declarations
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let revision = workspace
            .publish_modeling(declarations, PhysicalScope::default())
            .unwrap();
        let instance = pse_modeling::specialize::root_instance(root);
        let incremental = workspace
            .specialize_modeling(root, instance, Bindings::default(), Limits::default())
            .unwrap();
        let clean = pse_modeling::specialize::specialize_with_discretizer(
            revision.checked(),
            root,
            instance,
            &Bindings::default(),
            Limits::default(),
            &|| false,
            &LibraryDiscretizer,
        )
        .unwrap();
        assert_eq!(incremental.as_ref(), &clean);
        assert_eq!(executions.count(Query::FunctionFragment), expected);
    }
}

#[test]
fn shared_typed_function_math_preserves_distinct_original_callable_selections() {
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    let declarations = rows(
        "package p {fn first(x:Scalar)->Scalar=x; fn second(x:Scalar)->Scalar=x; fn apply(method:Fn(x:Scalar)->Scalar,x:Scalar)->Scalar=method(x); def Root {var x:Scalar;eq a:apply(first,x)==1;eq b:apply(second,x)==1;}}",
    );
    let root = declarations
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let apply = declarations
        .iter()
        .find(|row| row.name == "apply")
        .unwrap()
        .declaration_id;
    workspace
        .publish_modeling(declarations, PhysicalScope::default())
        .unwrap();
    let model = workspace
        .specialize_modeling(
            root,
            pse_modeling::specialize::root_instance(root),
            Bindings::default(),
            Limits::default(),
        )
        .unwrap();
    let inspections = model
        .functions
        .values()
        .filter(|function| function.id == apply)
        .collect::<Vec<_>>();
    assert_eq!(
        inspections.len(),
        2,
        "equivalent arithmetic must retain the two selected original callables"
    );
    assert_ne!(inspections[0].body, inspections[1].body);
}

#[test]
fn typed_ordinary_fields_reuse_local_bodies_and_preserve_lazy_source_obligations() {
    let (mut workspace, executions) = observed_workspace(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    );
    let original = "package p {def Root {var x:Scalar;let z:Scalar=x+x;let unused:Scalar=x*x;eq a:z==1;eq b:x*x==2;}}";
    let mut counts = Vec::new();
    for source in [
        original.to_owned(),
        original.replace("eq b:x*x", "eq b:x+x"),
        original.to_owned(),
        original.replace("eq b:x*x", "eq b:x+x*x"),
        format!("\n{original}"),
    ] {
        let declarations = rows(&source);
        let root = declarations
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let revision = workspace
            .publish_modeling(declarations, PhysicalScope::default())
            .unwrap();
        let instance = pse_modeling::specialize::root_instance(root);
        let incremental = workspace
            .specialize_modeling(root, instance, Bindings::default(), Limits::default())
            .unwrap();
        let clean = pse_modeling::specialize::specialize_with_discretizer(
            revision.checked(),
            root,
            instance,
            &Bindings::default(),
            Limits::default(),
            &|| false,
            &LibraryDiscretizer,
        )
        .unwrap();
        assert_eq!(incremental.as_ref(), &clean);
        assert!(
            !incremental
                .symbols
                .values()
                .any(|symbol| symbol.lineage.path.ends_with(".unused"))
        );
        counts.push(executions.count(Query::FieldFragment));
    }
    assert_eq!(
        counts[0], 4,
        "two original equation operands own four local producers"
    );
    assert_eq!(
        counts[1] - counts[0],
        2,
        "only changed equation's two fields rebuild; demanded lazy alias belongs to first field"
    );
    assert_eq!(counts[2], counts[1], "A/B/A reuses original local products");
    assert_eq!(
        counts[3] - counts[2],
        2,
        "different-sized unrelated body does not change another field's local key"
    );
    assert_eq!(
        counts[4] - counts[3],
        4,
        "original source shift refreshes each field's original attribution"
    );
}

#[test]
fn typed_ordinary_fields_preserve_inactive_aliases_and_membership_changes() {
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    let original = "package p {def Root(enabled:Boolean=false) {var x:Scalar;let inactive:Scalar=inactive;when enabled {let chosen:Scalar=x+x;} eq a:(if enabled then chosen else x)==1;}}";
    for source in [
        original.to_owned(),
        original.replace("=false", "=true"),
        original.to_owned(),
    ] {
        let declarations = rows(&source);
        let root = declarations
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let revision = workspace
            .publish_modeling(declarations, PhysicalScope::default())
            .unwrap();
        let instance = pse_modeling::specialize::root_instance(root);
        let incremental = workspace
            .specialize_modeling(root, instance, Bindings::default(), Limits::default())
            .unwrap();
        let clean = pse_modeling::specialize::specialize_with_discretizer(
            revision.checked(),
            root,
            instance,
            &Bindings::default(),
            Limits::default(),
            &|| false,
            &LibraryDiscretizer,
        )
        .unwrap();
        assert_eq!(incremental.as_ref(), &clean);
        assert!(
            !incremental
                .symbols
                .values()
                .any(|symbol| symbol.lineage.path.ends_with(".inactive"))
        );
        assert_eq!(
            incremental
                .symbols
                .values()
                .any(|symbol| symbol.lineage.path.ends_with(".chosen")),
            source.contains("=true")
        );
    }
}

#[test]
fn inherited_numerical_body_edits_preserve_structural_preludes() {
    let (mut workspace, executions) = observed_workspace(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    );
    let original = "package p {interface Base {var x:Scalar;eq inherited:x==2;} def Root:Base {eq local:x==1;}}";
    for source in [
        original.to_owned(),
        original.replace("x==2", "x==3"),
        original.to_owned(),
    ] {
        let declarations = rows(&source);
        let root = declarations
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let revision = workspace
            .publish_modeling(declarations, PhysicalScope::default())
            .unwrap();
        let instance = pse_modeling::specialize::root_instance(root);
        let incremental = workspace
            .specialize_modeling(root, instance, Bindings::default(), Limits::default())
            .unwrap();
        let clean = pse_modeling::specialize::specialize_with_discretizer(
            revision.checked(),
            root,
            instance,
            &Bindings::default(),
            Limits::default(),
            &|| false,
            &LibraryDiscretizer,
        )
        .unwrap();
        assert_eq!(incremental.as_ref(), &clean);
        assert_eq!(
            executions.count(Query::InstancePrelude),
            1,
            "an interface's numerical child body is not a static construction premise"
        );
    }
}

#[test]
fn ordinary_field_premises_follow_actual_instances_and_forwarded_sources() {
    let (mut workspace, executions) = observed_workspace(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    );
    let original = "package p {def Unit(k:Scalar) {var x:Scalar;eq e:x==k;} def Root {child a:Unit=Unit(k=1);child b:Unit=Unit(k=2);}}";
    let mut counts = Vec::new();
    for source in [
        original.to_owned(),
        original.replace("k=2", "k=3"),
        original.to_owned(),
    ] {
        let declarations = rows(&source);
        let root = declarations
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let revision = workspace
            .publish_modeling(declarations, PhysicalScope::default())
            .unwrap();
        let instance = pse_modeling::specialize::root_instance(root);
        let incremental = workspace
            .specialize_modeling(root, instance, Bindings::default(), Limits::default())
            .unwrap();
        let clean = pse_modeling::specialize::specialize_with_discretizer(
            revision.checked(),
            root,
            instance,
            &Bindings::default(),
            Limits::default(),
            &|| false,
            &LibraryDiscretizer,
        )
        .unwrap();
        assert_eq!(incremental.as_ref(), &clean);
        counts.push(executions.count(Query::FieldFragment));
    }
    assert_eq!(counts[0], 4);
    assert_eq!(
        counts[1] - counts[0],
        2,
        "another instance of the same definition is not this field's premise"
    );
    assert_eq!(
        counts[2], counts[1],
        "A/B/A retains each actual instance's original source product"
    );
}

#[test]
fn typed_fields_keep_actual_exchange_premises_and_refuse_changed_endpoints() {
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    let original = "package p {def Cell {boundary surface;var heat:Transfer<EnergyTransferRate,surface,Into>;} def Root {child hot:Cell=Cell();child cold:Cell=Cell();child warm:Cell=Cell();exchange pair between hot.surface and cold.surface;eq e:hot.heat==reflect(pair,cold.heat,Into);}}";
    for source in [
        original.to_owned(),
        original.replace("and cold.surface", "and warm.surface"),
        original.to_owned(),
    ] {
        let declarations = rows(&source);
        let root = declarations
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let revision = workspace
            .publish_modeling(declarations, PhysicalScope::default())
            .unwrap();
        let instance = pse_modeling::specialize::root_instance(root);
        let incremental =
            workspace.specialize_modeling(root, instance, Bindings::default(), Limits::default());
        let clean = pse_modeling::specialize::specialize_with_discretizer(
            revision.checked(),
            root,
            instance,
            &Bindings::default(),
            Limits::default(),
            &|| false,
            &LibraryDiscretizer,
        );
        if source == original {
            assert_eq!(incremental.unwrap().as_ref(), &clean.unwrap());
        } else {
            assert!(
                incremental.is_err(),
                "reflection cannot reuse an exchange that no longer contains its actual source boundary"
            );
            assert!(clean.is_err());
        }
    }
}

#[test]
fn typed_fields_keep_logical_temporal_parameters_across_actual_replicas() {
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    let original = "package p {difference backward order(1) offsets(-1,0) weights(-1,1) quadrature(0,1);def Cell(times:Set<Time>={}) {param gain:Scalar=2;var x:Time;when analysis.dynamic {eq ode[i in times]:d(x)/di==gain;}when not analysis.dynamic {eq stationary:x==gain*1{s};}port out:Time=x;}def Root {domain t:Time from 0{s} to 2{s};when analysis.route==analysis.steady {discretize mesh on t using stationary(elements=1,order=1);}when analysis.route==analysis.integrated {discretize mesh on t using integrated(elements=1,order=1);}when analysis.route==analysis.simultaneous {discretize mesh on t using backward(elements=2,order=1);}child cell:Cell=Cell();evolve time_state on cell using t bind times;when analysis.dynamic {eq initial:cell[0{s}].x==0{s};}}}";
    let declarations = rows(original);
    let root = declarations
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let revision = workspace
        .publish_modeling(declarations, PhysicalScope::default())
        .unwrap();
    let instance = pse_modeling::specialize::root_instance(root);
    use pse_modeling::analysis::Route;
    for route in [
        Route::Steady,
        Route::Integrated,
        Route::Simultaneous,
        Route::Integrated,
        Route::Steady,
    ] {
        let bindings = Bindings::default().with_analysis(route);
        let incremental = workspace
            .specialize_modeling(root, instance, bindings.clone(), Limits::default())
            .unwrap();
        let clean = pse_modeling::specialize::specialize_with_discretizer(
            revision.checked(),
            root,
            instance,
            &bindings,
            Limits::default(),
            &|| false,
            &LibraryDiscretizer,
        )
        .unwrap();
        assert_eq!(incremental.as_ref(), &clean);
        assert_eq!(
            incremental
                .symbols
                .values()
                .filter(|symbol| symbol.lineage.path.ends_with(".gain"))
                .count(),
            1
        );
    }
    let varying = original.replace(
        "param gain:Scalar=2",
        "param gain:Scalar=sum(i in times | i/1{s})",
    );
    let revision = workspace
        .publish_modeling(rows(&varying), PhysicalScope::default())
        .unwrap();
    let bindings = Bindings::default().with_analysis(Route::Simultaneous);
    let incremental = workspace
        .specialize_modeling(root, instance, bindings.clone(), Limits::default())
        .unwrap_err();
    let clean = pse_modeling::specialize::specialize_with_discretizer(
        revision.checked(),
        root,
        instance,
        &bindings,
        Limits::default(),
        &|| false,
        &LibraryDiscretizer,
    )
    .unwrap_err();
    assert!(
        incremental
            .to_string()
            .contains("scalar temporal parameter defaults must agree"),
        "{incremental}"
    );
    assert!(
        clean
            .to_string()
            .contains("scalar temporal parameter defaults must agree"),
        "{clean}"
    );
}
