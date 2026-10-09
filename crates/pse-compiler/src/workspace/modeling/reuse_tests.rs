// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Counted controls on the actual admitted-package production route.
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
    let executions = Arc::new(AtomicUsize::new(0));
    let observed = executions.clone();
    let event = Box::new(move |event: salsa::Event| {
        let event = format!("{:?}", event.kind);
        if event.contains("WillExecute") && event.contains("semantic_body") {
            observed.fetch_add(1, Ordering::Relaxed);
        }
    });
    let mut workspace = CompilerWorkspace::with_events(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
        Some(event),
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
    let first = executions.load(Ordering::Relaxed);
    assert!(first > 0);
    workspace
        .publish_modeling(rows(&format!("\n\n{text}")), PhysicalScope::default())
        .unwrap();
    let shifted = prepare(&mut workspace, root, Bindings::default());
    assert_eq!(executions.load(Ordering::Relaxed), first);
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
    assert_eq!(executions.load(Ordering::Relaxed), first + changed_bodies);
    assert!(a.admitted.bodies.iter().any(|(key, body)| {
        changed
            .admitted
            .bodies
            .get(key)
            .is_some_and(|next| Arc::ptr_eq(&body.math, &next.math))
    }));
    assert_eq!(a.occurrences(), original_occurrences);
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

fn counting_frontier_workspace(executions: Arc<AtomicUsize>) -> CompilerWorkspace {
    CompilerWorkspace::with_events(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
        Some(Box::new(move |event: salsa::Event| {
            let event = format!("{:?}", event.kind);
            if event.contains("WillExecute") && event.contains("semantic_body") {
                executions.fetch_add(1, Ordering::Relaxed);
            }
        })),
    )
    .unwrap()
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
    let executions = Arc::new(AtomicUsize::new(0));
    let retention = Arc::new(PureFrontierRetention::default());
    let mut workspace = counting_frontier_workspace(executions.clone());
    workspace.attach_body_retention(retention.clone()).unwrap();
    let root = publish_frontier_source(
        &mut workspace,
        "package p { def Root { var x:Scalar; eq a:x*x==1; eq b:x==2; } }",
    );
    let frontier = plan_frontier(&mut workspace, root);
    let requests = frontier.body_requests().collect::<BTreeSet<_>>();
    assert!(!requests.is_empty());
    assert!(frontier.retained_bytes() > 0);
    assert_eq!(executions.load(Ordering::Relaxed), 0);
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
    assert_eq!(executions.load(Ordering::Relaxed), requests.len());
    assert_eq!(retention.retains.load(Ordering::Relaxed), requests.len());
    let again = workspace
        .complete_modeling_cancellable(frontier, Arc::new(AtomicBool::new(false)))
        .unwrap();
    assert_eq!(again.admitted.case(), completed.admitted.case());
    assert_eq!(again.portable_bodies().count(), requests.len());
    assert_eq!(
        executions.load(Ordering::Relaxed),
        requests.len(),
        "body memos hit without losing the portable inventory"
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
    let executions = Arc::new(AtomicUsize::new(0));
    let mut workspace = counting_frontier_workspace(executions.clone());
    let root = publish_frontier_source(
        &mut workspace,
        "package p { def Root { param p:Scalar=2; implicit a { var selected_value:Scalar; eq selected:selected_value==p; annotation start selected_value(1); annotation bounds selected_value(0,4); } realize ra on a using nested; var y:Scalar; eq e:y==a.selected_value; } }",
    );
    let frontier = plan_frontier(&mut workspace, root);
    let requests = frontier.body_requests().collect::<BTreeSet<_>>();
    assert_eq!(
        executions.load(Ordering::Relaxed),
        0,
        "only existing direct implicit admission occurs during planning"
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
    let executed = executions.load(Ordering::Relaxed);
    let again = workspace
        .complete_modeling_cancellable(frontier, Arc::new(AtomicBool::new(false)))
        .unwrap();
    assert_eq!(again.portable_bodies().count(), requests.len());
    assert_eq!(executions.load(Ordering::Relaxed), executed);
    assert_eq!(
        again.original_equations().unwrap().admitted.as_ref(),
        original.admitted.as_ref()
    );
}

#[test]
fn owned_frontier_completion_refuses_changed_inventory_and_cancellation_is_retryable() {
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
