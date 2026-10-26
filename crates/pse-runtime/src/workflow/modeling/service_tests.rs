// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Scientific sources retain their values while each caller supplies its effect capabilities.
use super::*;

#[tokio::test]
async fn immutable_source_uses_explicit_receiver_after_original_caller_closes() {
    let runtime = crate::workflow::tests::runtime();
    let receiver =
        Runtime::from_shared_local(runtime.shared.clone(), runtime.model.clone()).unwrap();
    let declarations = pse_authoring::language::parse(
        "package p { def Root { param a:Scalar=2; var x:Scalar; eq e:x==a; } }",
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        Default::default(),
    )
    .unwrap();
    let root = declarations
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime
        .modeling_package(declarations, crate::workflow::tests::physical())
        .await
        .unwrap();
    let revision = package.revision_id().to_owned();
    let original = runtime.preparation_services();
    runtime.close();
    // The rejected original effect owner does not erase the immutable scientific input.
    let refused = package
        .prepare(
            &original,
            root,
            pse_modeling::specialize::root_instance(root),
            Bindings::default(),
            Limits::default(),
            &crate::CancelSource::new(),
        )
        .await;
    assert!(matches!(refused, Err(WorkflowError::CallerClosed)));
    let prepared = package
        .prepare(
            &receiver.preparation_services(),
            root,
            pse_modeling::specialize::root_instance(root),
            Bindings::default(),
            Limits::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert_eq!(
        prepared.solved().instance(),
        pse_modeling::specialize::root_instance(root)
    );
    assert_eq!(package.revision_id(), revision);
    assert!(package.source_issuer().is_none());
    // A retained admission lease carries release state, without recovering an effect owner.
    let scope = engagement::HeldPreparationScope::new();
    let protected = scope.package(&package);
    scope.freeze().unwrap();
    scope.close().await.unwrap();
    let expired = protected
        .prepare(
            &receiver.preparation_services(),
            root,
            pse_modeling::specialize::root_instance(root),
            Bindings::default(),
            Limits::default(),
            &crate::CancelSource::new(),
        )
        .await;
    assert!(expired.unwrap_err().to_string().contains("closing"));
}
