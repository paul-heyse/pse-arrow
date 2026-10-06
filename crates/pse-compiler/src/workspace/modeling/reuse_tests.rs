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
