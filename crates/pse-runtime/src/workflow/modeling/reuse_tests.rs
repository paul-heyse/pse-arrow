// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Allocation and retention controls through the admitted ModelingPackage production route.
use super::*;
use std::sync::Arc;
fn declarations(source: &str) -> Vec<Declaration> {
    pse_authoring::language::parse(
        source,
        SemanticId::from_bytes([77; 16]),
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap()
}
async fn prepared(package: &ModelingPackage) -> ModelingPreparation {
    let root = package
        .declarations()
        .await
        .unwrap()
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    package
        .prepare(
            root,
            pse_modeling::specialize::root_instance(root),
            Bindings::default(),
            Limits::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap()
}
#[tokio::test]
async fn shared_bodies_escape_fresh_workspaces_and_cache_invalidation_with_owned_allocations() {
    let runtime = super::super::tests::runtime();
    let service = runtime.shared.math();
    let pool = runtime.shared.pool();
    let baseline = pool.reserved();
    let source = "package p { def Root { var x: Scalar; eq a:x*x==1; eq b:x==2; } }";
    let first = runtime
        .modeling_package(declarations(source), super::super::tests::physical())
        .await
        .unwrap();
    let a = prepared(&first).await;
    let second = runtime
        .modeling_package(
            declarations(&format!("\n\n{source}")),
            super::super::tests::physical(),
        )
        .await
        .unwrap();
    let b = prepared(&second).await;
    assert_ne!(a.compiled().occurrences(), b.compiled().occurrences());
    for (key, body) in &a.compiled().admitted.bodies {
        assert!(Arc::ptr_eq(
            body.math(),
            b.compiled().admitted.bodies[key].math()
        ));
    }
    let escaped = Arc::new(
        a.compiled()
            .admitted
            .bodies
            .values()
            .next()
            .unwrap()
            .math()
            .as_ref()
            .clone()
            .with_owner(Arc::new(())),
    );
    let attribution = a.compiled().model.clone();
    service.clear_program_cache();
    drop(a);
    drop(b);
    drop(first);
    drop(second);
    assert!(
        pool.reserved() > baseline,
        "escaped math retains actual allocation lease after invalidation and workspace release"
    );
    drop(escaped);
    assert!(
        pool.reserved() > baseline,
        "escaped model alias retains revision attribution and its original product owner"
    );
    drop(attribution);
    assert_eq!(pool.reserved(), baseline);
}
#[tokio::test]
async fn refused_allocation_leaves_the_same_admitted_package_retryable() {
    let runtime = super::super::tests::runtime();
    let service = runtime.shared.math();
    let pool = runtime.shared.pool();
    let baseline = pool.reserved();
    let package = runtime
        .modeling_package(
            declarations("package p { def Root { var x:Scalar; eq a:x*x==1; } }"),
            super::super::tests::physical(),
        )
        .await
        .unwrap();
    let root = package
        .declarations()
        .await
        .unwrap()
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let pressure = service
        .reserve("test:held-allocation", (512 << 20) - pool.reserved() - 1024)
        .unwrap();
    let refusal = package
        .prepare(
            root,
            pse_modeling::specialize::root_instance(root),
            Bindings::default(),
            Limits::default(),
            &crate::CancelSource::new(),
        )
        .await;
    assert!(
        refusal.is_err(),
        "a live allocation exhausts the actual bounded pool"
    );
    drop(pressure);
    let retry = prepared(&package).await;
    assert!(!retry.compiled().admitted.bodies.is_empty());
    service.clear_program_cache();
    drop(retry);
    drop(package);
    assert_eq!(
        pool.reserved(),
        baseline,
        "refusal retains neither admission nor orphaned allocations"
    );
}
#[tokio::test]
async fn escaped_nested_body_retains_its_admission_owner_after_eviction() {
    let runtime = super::super::tests::runtime();
    let service = runtime.shared.math();
    let pool = runtime.shared.pool();
    let baseline = pool.reserved();
    let package=runtime.modeling_package(declarations("package p { def Root { implicit inner {var y:Scalar; eq root:y==2; annotation start y(1);} realize r on inner using nested; eq pin:inner.y==2; } }"),super::super::tests::physical()).await.unwrap();
    let model = prepared(&package).await;
    let attached = model.compiled().clone().with_owner(Arc::new(()));
    assert!(
        pse_math::SharedAllocation::ptr_eq(&attached.admitted, &model.compiled().admitted),
        "repeated attachment shares already owned nested descriptors"
    );
    let nested = attached.implicit_order().unwrap();
    assert_eq!(nested.len(), 1);
    let escaped = nested[0].residuals[0].body.math().clone();
    service.clear_program_cache();
    drop(nested);
    drop(attached);
    drop(model);
    drop(package);
    assert!(
        pool.reserved() > baseline,
        "the nested math alias retains the original admission and its charged attachment wrappers"
    );
    assert_eq!(escaped.output_count(), 1);
    drop(escaped);
    assert_eq!(pool.reserved(), baseline);
}
#[tokio::test]
async fn shared_body_cache_invalidates_the_complete_physical_closure() {
    let runtime = super::super::tests::runtime();
    let service = runtime.shared.math();
    let pool = runtime.shared.pool();
    let baseline = pool.reserved();
    let source = "package p { def Root { var x:Scalar; eq a:x*x==1; } }";
    let original = super::super::tests::physical();
    let mut changed = original.clone();
    let mut quantities = changed.quantities.to_builder();
    quantities.entity_kind(pse_quantity::EntityKind {
        id: pse_quantity::EntityKindId::from_id(SemanticId::from_bytes([249; 16])),
        name: "cache_context_control".into(),
    });
    changed.quantities = Arc::new(quantities.build().unwrap());
    changed.key =
        pse_compiler::workspace::physical_identity(&changed.quantities, &changed.preconditions);
    assert_ne!(original.key, changed.key);
    let first = runtime
        .modeling_package(declarations(source), original)
        .await
        .unwrap();
    let a = prepared(&first).await;
    let second = runtime
        .modeling_package(declarations(source), changed)
        .await
        .unwrap();
    let b = prepared(&second).await;
    assert!(!a.compiled().admitted.bodies.is_empty());
    for (key, body) in &a.compiled().admitted.bodies {
        assert!(!b.compiled().admitted.bodies.contains_key(key));
        assert!(
            b.compiled()
                .admitted
                .bodies
                .values()
                .all(|other| body.spec().physical != other.spec().physical
                    && !Arc::ptr_eq(body.math(), other.math()))
        );
    }
    service.clear_program_cache();
    drop(a);
    drop(b);
    drop(first);
    drop(second);
    assert_eq!(pool.reserved(), baseline);
}
#[tokio::test]
async fn canonical_edits_preserve_unchanged_memberships_and_skip_true_noops() {
    let runtime = super::super::tests::runtime();
    let original = "package p { def Root { var x: Scalar; eq a:x*x==1; } def Other { var z:Scalar; eq b:z==2; } }";
    let first = runtime
        .modeling_package(declarations(original), super::super::tests::physical())
        .await
        .unwrap();
    let noop = first
        .with_declarations(declarations(original))
        .await
        .unwrap();
    assert_eq!(first.canonical_revision(), noop.canonical_revision());
    let store = runtime.canonical.store();
    let pin = store
        .protect(
            first.canonical_revision().clone(),
            std::time::Duration::from_secs(60),
        )
        .await
        .unwrap();
    let root = declarations(original)
        .into_iter()
        .find(|row| row.name == "Root")
        .unwrap();
    let before = store
        .select_names(&pin, &canonical::scope(root.parent_id), &["Root".into()])
        .await
        .unwrap();
    let changed = first.with_declarations(declarations("package p { def Root { var x: Scalar; eq a:x*x==1; } def Other { var z:Scalar; eq b:z==3; } }")).await.unwrap();
    assert_eq!(
        changed.canonical_revision().sequence,
        first.canonical_revision().sequence + 1
    );
    let current = store
        .protect(
            changed.canonical_revision().clone(),
            std::time::Duration::from_secs(60),
        )
        .await
        .unwrap();
    let after = store
        .select_names(
            &current,
            &canonical::scope(root.parent_id),
            &["Root".into()],
        )
        .await
        .unwrap();
    assert_eq!(
        before, after,
        "unrelated edits preserve the exact membership interval"
    );
    let removed = changed
        .with_declarations(declarations(
            "package p { def Root { var x: Scalar; eq a:x*x==1; } }",
        ))
        .await
        .unwrap();
    let final_pin = store
        .protect(
            removed.canonical_revision().clone(),
            std::time::Duration::from_secs(60),
        )
        .await
        .unwrap();
    assert!(
        store
            .select_names(
                &final_pin,
                &canonical::scope(root.parent_id),
                &["Other".into()]
            )
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        store
            .select_names(&pin, &canonical::scope(root.parent_id), &["Root".into()])
            .await
            .unwrap(),
        before
    );
    store.release(&pin).await.unwrap();
    store.release(&current).await.unwrap();
    store.release(&final_pin).await.unwrap();
}

#[tokio::test]
async fn canonical_package_admission_preserves_source_identity_and_accelerators_on_edits() {
    let runtime = super::super::tests::runtime();
    let source = "package p { def Root { var x: Scalar; eq a:x*x==1; } }";
    let texts = BTreeMap::from([
        (
            "package.toml".into(),
            include_bytes!("../../../../../tests/fixtures/packages/minimal_explicit/package.toml")
                .to_vec(),
        ),
        (
            "models/kernel.pse".into(),
            pse_authoring::language::render(&declarations(source))
                .unwrap()
                .into_bytes(),
        ),
    ]);
    let first = runtime
        .package_from_sources(
            std::slice::from_ref(&texts),
            super::super::tests::physical(),
        )
        .await
        .unwrap();
    let second = runtime
        .package_from_sources(
            std::slice::from_ref(&texts),
            super::super::tests::physical(),
        )
        .await
        .unwrap();
    assert_eq!(first.revision.identity(), second.revision.identity());
    assert_eq!(
        first.canonical_revision(),
        second.canonical_revision(),
        "the ingress cache retains a compact immutable revision handle"
    );
    let mut other_context = runtime.clone();
    other_context.sessions = Arc::new(runtime.sessions.as_ref().clone());
    let scoped = other_context
        .package_from_sources(
            std::slice::from_ref(&texts),
            super::super::tests::physical(),
        )
        .await
        .unwrap();
    assert_eq!(first.revision.identity(), scoped.revision.identity());
    assert_ne!(
        first.canonical_revision().problem,
        scoped.canonical_revision().problem,
        "exact validation context owns ingress reuse"
    );
    let edited = first.with_declarations(declarations(source)).await.unwrap();
    assert!(Arc::ptr_eq(&first.accelerators, &edited.accelerators));
    runtime.shared.math().clear_program_cache();
    let fresh = runtime
        .package_from_sources(
            std::slice::from_ref(&texts),
            super::super::tests::physical(),
        )
        .await
        .unwrap();
    assert_eq!(first.revision.identity(), fresh.revision.identity());
    assert_ne!(
        first.canonical_revision().problem,
        fresh.canonical_revision().problem
    );
}

#[tokio::test]
async fn solver_view_rebinds_share_plan_storage_and_charge_fresh_wrappers() {
    let runtime = super::super::tests::runtime();
    let service = runtime.shared.math();
    let pool = runtime.shared.pool();
    let baseline = pool.reserved();
    let package = runtime
        .modeling_package(
            declarations("package p { def Root { param p:Scalar; var x: Scalar; eq a:x*p==1; } }"),
            super::super::tests::physical(),
        )
        .await
        .unwrap();
    let model = prepared(&package).await;
    let symbol = |name: &str| {
        model
            .compiled()
            .model
            .symbols
            .values()
            .find(|symbol| {
                symbol.lineage.path == name || symbol.lineage.path.ends_with(&format!(".{name}"))
            })
            .unwrap()
            .id
    };
    let x = symbol("x");
    let p = symbol("p");
    let values = pse_math::binding::CaseValues {
        scalars: BTreeMap::from([(x, 1.0), (p, 1.0)]),
    };
    let compiler = super::super::tests::compiler_profile();
    let cancel = crate::CancelSource::new();
    let first = package
        .bound_case(
            &model,
            values.clone(),
            &BTreeMap::new(),
            pse_kernels::DerivativeOrder::Second,
            compiler,
            &cancel,
        )
        .await
        .unwrap();
    let before = pool.reserved();
    let second = package
        .bound_case(
            &model,
            pse_math::binding::CaseValues {
                scalars: BTreeMap::from([(x, 1.0), (p, 2.0)]),
            },
            &BTreeMap::new(),
            pse_kernels::DerivativeOrder::Second,
            compiler,
            &cancel,
        )
        .await
        .unwrap();
    assert!(Arc::ptr_eq(
        &first.case.compiled().plan,
        &second.case.compiled().plan
    ));
    assert!(pse_math::SharedAllocation::ptr_eq(
        &first.case.compiled().occurrences,
        &second.case.compiled().occurrences
    ));
    assert!(
        pool.reserved() > before,
        "each escaping rebind owns its new wrapper allocation"
    );
    let shifted_package = runtime
        .modeling_package(
            declarations(
                "

package p { def Root { param p:Scalar; var x: Scalar; eq a:x*p==1; } }",
            ),
            super::super::tests::physical(),
        )
        .await
        .unwrap();
    let shifted_model = prepared(&shifted_package).await;
    let shifted = shifted_package
        .bound_case(
            &shifted_model,
            pse_math::binding::CaseValues {
                scalars: BTreeMap::from([(x, 1.0), (p, 2.0)]),
            },
            &BTreeMap::new(),
            pse_kernels::DerivativeOrder::Second,
            compiler,
            &cancel,
        )
        .await
        .unwrap();
    assert_ne!(
        first.case.compiled().occurrences.as_ref(),
        shifted.case.compiled().occurrences.as_ref()
    );
    let escaped = Arc::new(
        first
            .case
            .compiled()
            .plan
            .as_ref()
            .clone()
            .with_owner(Arc::new(())),
    );
    let binding = second.case.compiled().coefficient_values.clone();
    let attribution = shifted.case.compiled().occurrences.clone();
    let derived = second.case.compiled().derived.clone();
    let witness = second.case.structural_witness();
    let physical = second
        .case
        .compiled()
        .quantities
        .clone()
        .with_owner(Arc::new(()));
    let erased = physical.allocation_payload();
    service.clear_program_cache();
    drop(first);
    drop(second);
    drop(shifted);
    drop(shifted_model);
    drop(shifted_package);
    drop(model);
    drop(package);
    assert!(pool.reserved() > baseline);
    drop(escaped);
    assert!(
        pool.reserved() > baseline,
        "escaped binding/attribution aliases retain the rebind owner"
    );
    assert!(
        binding
            .iter()
            .any(|(id, bits)| *id == p && *bits == 2.0_f64.to_bits())
    );
    assert!(!attribution.is_empty());
    drop(binding);
    assert!(pool.reserved() > baseline);
    drop(attribution);
    drop(derived);
    drop(witness);
    assert!(
        pool.reserved() > baseline,
        "escaped physical registry retains its owner"
    );
    drop(physical);
    assert!(
        pool.reserved() > baseline,
        "erased payload alias retains original accounting owner"
    );
    drop(erased);
    assert_eq!(pool.reserved(), baseline);
}
