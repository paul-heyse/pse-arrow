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

#[tokio::test]
async fn unrelated_services_refuse_source_preparation_and_single_step_start_before_effects() {
    let runtime = crate::workflow::tests::runtime();
    let receiver =
        Runtime::from_shared_local(runtime.shared.clone(), runtime.model.clone()).unwrap();
    let unrelated = crate::workflow::tests::runtime();
    let rows = pse_authoring::language::parse(
        "package p { def Root { param a:Scalar=2; annotation check a(a>0); } }",
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        Default::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime
        .modeling_package(rows, crate::workflow::tests::physical())
        .await
        .unwrap();
    let before = unrelated.native().preparation_effects();
    let reserved = unrelated.shared.pool().reserved();
    assert!(
        package
            .prepare(
                &unrelated.preparation_services(),
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                &crate::CancelSource::new(),
            )
            .await
            .is_err()
    );
    assert_eq!(unrelated.native().preparation_effects(), before);
    assert_eq!(unrelated.shared.pool().reserved(), reserved);
    let prepared = package
        .prepare_solve(
            &runtime.preparation_services(),
            root,
            pse_modeling::specialize::root_instance(root),
            Bindings::default(),
            Limits::default(),
            Default::default(),
            pse_kernels::DerivativeOrder::Second,
            crate::workflow::tests::compiler_profile(),
            crate::workflow::tests::profile(),
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let before = unrelated.native().test_admission_snapshot();
    assert!(prepared.start(&unrelated.execution_services()).is_err());
    assert_eq!(unrelated.native().test_admission_snapshot(), before);
    assert_eq!(unrelated.shared.pool().reserved(), reserved);
    // Caller identity may change while the admitted registry/native bindings remain exact.
    runtime.close();
    let result = prepared
        .start(&receiver.execution_services())
        .unwrap()
        .wait()
        .await
        .unwrap();
    assert!(result.usable(), "{:?}", result.report());
}

#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn prepared_fit_refuses_unrelated_execution_bindings_before_worker_admission() {
    use crate::workflow::fitting::regression;
    let runtime = crate::workflow::tests::runtime();
    let unrelated = crate::workflow::tests::runtime();
    let package = regression::package(&runtime, regression::declared(), [1.; 4]).await;
    let prepared = package
        .prepare_fit(
            &runtime.preparation_services(),
            crate::workflow::tests::id(32).into(),
            regression::profile(pse_backend_native::solve::HessianMode::LimitedMemory, None),
            crate::workflow::tests::compiler_profile(),
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let before = unrelated.native().test_admission_snapshot();
    let reserved = unrelated.shared.pool().reserved();
    assert!(prepared.start(&unrelated.execution_services()).is_err());
    assert_eq!(unrelated.native().test_admission_snapshot(), before);
    assert_eq!(unrelated.shared.pool().reserved(), reserved);
}
