// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Allocation and retention controls through the admitted ModelingPackage production route.
#[cfg(feature = "solver-kinsol")]
#[path = "reuse_journey_tests.rs"]
mod reuse_journey_tests;

use super::*;
use std::{collections::BTreeSet, sync::Arc};
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
        .reserve(
            "test:held-allocation",
            runtime.shared.budget().memory_limit_bytes.get() - pool.reserved() - 1024,
        )
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
async fn local_edits_preserve_unchanged_products_parent_and_true_noops() {
    let runtime = super::super::tests::runtime();
    let original = "package p { def Root { var x: Scalar; eq a:x*x==1; } def Other { var z:Scalar; eq b:z==2; } }";
    let first = runtime
        .modeling_package(declarations(original), super::super::tests::physical())
        .await
        .unwrap();
    assert!(first.canonical_revision().is_none());
    let before = prepared(&first).await;
    let noop = first
        .with_declarations(declarations(original))
        .await
        .unwrap();
    assert_eq!(first.revision_id(), noop.revision_id());
    let effects = runtime.shared.math().preparation_effects();
    let again = prepared(&noop).await;
    let effects = runtime.shared.math().preparation_effects().since(effects);
    assert_eq!(effects.workspaces, 0);
    assert!(pse_math::SharedAllocation::ptr_eq(
        &before.compiled().admitted,
        &again.compiled().admitted
    ));
    let changed = first.with_declarations(declarations("package p { def Root { var x: Scalar; eq a:x*x==1; } def Other { var z:Scalar; eq b:z==3; } }")).await.unwrap();
    assert_ne!(first.revision_id(), changed.revision_id());
    let after = prepared(&changed).await;
    for (key, body) in &before.compiled().admitted.bodies {
        assert!(Arc::ptr_eq(
            body.math(),
            after.compiled().admitted.bodies[key].math()
        ));
    }
    let removed = changed
        .with_declarations(declarations(
            "package p { def Root { var x: Scalar; eq a:x*x==1; } }",
        ))
        .await
        .unwrap();
    assert!(
        !removed
            .declarations()
            .await
            .unwrap()
            .iter()
            .any(|row| row.name == "Other")
    );
    assert!(
        first
            .declarations()
            .await
            .unwrap()
            .iter()
            .any(|row| row.name == "Other")
    );
    let historical = prepared(&first).await;
    assert_eq!(before.solved(), historical.solved());
    runtime.shared.math().clear_program_cache();
}

#[tokio::test]
async fn local_package_admission_preserves_source_identity_and_accelerators_on_edits() {
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
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let second = runtime
        .package_from_sources(
            std::slice::from_ref(&texts),
            super::super::tests::physical(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert_eq!(first.revision.identity(), second.revision.identity());
    assert_eq!(
        first.revision_id(),
        second.revision_id(),
        "the ingress cache retains its owned immutable local revision"
    );
    let mut other_context = runtime.clone();
    other_context.model = Arc::new(runtime.sessions().as_ref().clone())
        .bind_model(runtime.registry().clone())
        .unwrap();
    let scoped = other_context
        .package_from_sources(
            std::slice::from_ref(&texts),
            super::super::tests::physical(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert_eq!(first.revision.identity(), scoped.revision.identity());
    assert_ne!(
        first.revision_id(),
        scoped.revision_id(),
        "exact validation context owns ingress reuse"
    );
    let edited = first.with_declarations(declarations(source)).await.unwrap();
    assert!(Arc::ptr_eq(&first.accelerators, &edited.accelerators));
    runtime.shared.math().clear_program_cache();
    let fresh = runtime
        .package_from_sources(
            std::slice::from_ref(&texts),
            super::super::tests::physical(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert_eq!(first.revision.identity(), fresh.revision.identity());
    assert_ne!(first.revision_id(), fresh.revision_id());
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

#[tokio::test]
async fn checked_local_admission_rechecks_exact_premises_and_retains_a_b_a() {
    let runtime = super::super::tests::runtime();
    let source = "package p { def Root { var x:Scalar; eq a:x*x==1; } def Other { var z:Scalar; eq b:z==2; } }";
    let first = runtime
        .modeling_package(declarations(source), super::super::tests::physical())
        .await
        .unwrap();
    let root = declarations(source)
        .into_iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let cancel = crate::CancelSource::new();
    let a = first.checked_selection(&[root], &cancel).await.unwrap();
    let held = a.admission.revision.clone();
    let hit = first.checked_selection(&[root], &cancel).await.unwrap();
    assert!(held.same_admission(&hit.admission.revision));
    let unrelated = first.with_declarations(declarations("package p { def Root { var x:Scalar; eq a:x*x==1; } def Other { var z:Scalar; eq b:z==3; } }")).await.unwrap();
    let unchanged = unrelated.checked_selection(&[root], &cancel).await.unwrap();
    assert!(
        held.same_admission(&unchanged.admission.revision),
        "a changed unconsumed body preserves checked selected input"
    );
    let changed = unrelated.with_declarations(declarations("package p { def Root { var x:Scalar; eq a:x*x==2; } def Other { var z:Scalar; eq b:z==3; } }")).await.unwrap();
    let b = changed.checked_selection(&[root], &cancel).await.unwrap();
    assert!(
        !held.same_admission(&b.admission.revision),
        "structural literal changes must re-admit"
    );
    let restored = changed
        .with_declarations(declarations(source))
        .await
        .unwrap();
    let again = restored.checked_selection(&[root], &cancel).await.unwrap();
    assert!(
        held.same_admission(&again.admission.revision),
        "A/B/A retains complete eligible candidates"
    );
    runtime.shared.math().clear_program_cache();
    let fresh = restored.checked_selection(&[root], &cancel).await.unwrap();
    assert!(
        !held.same_admission(&fresh.admission.revision),
        "clear forces current admission without invalidating escaped immutable owners"
    );
}

#[tokio::test]
async fn checked_selected_admission_finds_oldest_after_more_than_one_candidate_window() {
    let runtime = super::super::tests::runtime();
    let source = "package p { def Root { var x:Scalar; eq a:x*x==1; } }";
    let rows = declarations(source);
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let mut package = runtime
        .modeling_package(rows, super::super::tests::physical())
        .await
        .unwrap();
    let cancel = crate::CancelSource::new();
    let first = package.checked_selection(&[root], &cancel).await.unwrap();
    let original = first.admission.revision.clone();
    let request = first.admission.request.clone();
    for literal in 2..=10 {
        package = package
            .with_declarations(declarations(&format!(
                "package p {{ def Root {{ var x:Scalar; eq a:x*x=={literal}; }} }}"
            )))
            .await
            .unwrap();
        let selected = package.checked_selection(&[root], &cancel).await.unwrap();
        assert!(!original.same_admission(&selected.admission.revision));
    }
    assert!(
        runtime
            .shared
            .math()
            .modeling_cache
            .selected(&request)
            .unwrap()
            .len()
            >= 10,
        "the fixture retains history beyond one bounded lookup window"
    );
    package = package
        .with_declarations(declarations(source))
        .await
        .unwrap();
    let before = runtime.shared.math().preparation_effects();
    let restored = package.checked_selection(&[root], &cancel).await.unwrap();
    assert!(
        original.same_admission(&restored.admission.revision),
        "lookup must continue beyond the newest eight rejected admissions"
    );
    assert_eq!(
        runtime
            .shared
            .math()
            .preparation_effects()
            .since(before)
            .workspaces,
        0,
        "the retained oldest candidate needs no new compiler workspace"
    );
}

#[tokio::test]
async fn checked_selected_admission_invalidates_absent_package_rule_inventory() {
    let runtime = super::super::tests::runtime();
    let rows = declarations(
        "package p {use policy @\"1.0.0\"; def Root {var T:Temperature; eq temperature:T==280{K};}} package policy {entity kind source provenance {attribute title:Text;} enum role {published} entity source maintainer {title=\"engineering policy\"} constant temperature:DeltaTemperature=0.1{K} provenance(maintainer,role.published); annotation engineering_rule policy.temperature;}",
    );
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let marker = rows
        .iter()
        .find(|row| row.value.annotation.is_some())
        .unwrap()
        .declaration_id;
    // Keep source locations and all positive dependencies exact. Only the recorded
    // empty engineering-rule inventory becomes populated.
    let unmarked = rows
        .iter()
        .filter(|row| row.declaration_id != marker)
        .cloned()
        .collect();
    let first = runtime
        .modeling_package(unmarked, super::super::tests::physical())
        .await
        .unwrap();
    let cancel = crate::CancelSource::new();
    let before = first.checked_selection(&[root], &cancel).await.unwrap();
    assert!(
        !before
            .admission
            .revision
            .declarations()
            .iter()
            .any(|row| row.declaration_id == marker)
    );
    let changed = first.with_declarations(rows).await.unwrap();
    let after = changed.checked_selection(&[root], &cancel).await.unwrap();
    assert_eq!(before.admission.request, after.admission.request);
    assert!(
        !before
            .admission
            .revision
            .same_admission(&after.admission.revision)
    );
    assert!(
        after
            .admission
            .revision
            .declarations()
            .iter()
            .any(|row| row.declaration_id == marker)
    );
    let model = prepared(&changed).await;
    assert_eq!(model.compiled().model.engineering_rules.len(), 1);
    assert_eq!(model.compiled().model.engineering_rules[0].marker, marker);
}

#[tokio::test]
async fn checked_selected_admission_invalidates_nearer_name_shadowing() {
    let runtime = super::super::tests::runtime();
    let rows = declarations(
        "package p {param ambient:Scalar=8; def Root {var x:Scalar; eq e:x==ambient; param ambient:Scalar=2;}} ",
    );
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let inner = rows
        .iter()
        .find(|row| row.name == "ambient" && row.parent_id == Some(root))
        .unwrap()
        .declaration_id;
    let outer = rows
        .iter()
        .find(|row| row.name == "ambient" && row.parent_id != Some(root))
        .unwrap()
        .declaration_id;
    let unshadowed = rows
        .iter()
        .filter(|row| row.declaration_id != inner)
        .cloned()
        .collect::<Vec<_>>();
    let first = runtime
        .modeling_package(unshadowed.clone(), super::super::tests::physical())
        .await
        .unwrap();
    let cancel = crate::CancelSource::new();
    let before = first.checked_selection(&[root], &cancel).await.unwrap();
    assert_eq!(
        before.admission.revision.checked().resolve(root, "ambient"),
        Some(outer)
    );
    let shadowed = first.with_declarations(rows).await.unwrap();
    let after = shadowed.checked_selection(&[root], &cancel).await.unwrap();
    assert_eq!(before.admission.request, after.admission.request);
    assert!(
        !before
            .admission
            .revision
            .same_admission(&after.admission.revision)
    );
    assert_eq!(
        after.admission.revision.checked().resolve(root, "ambient"),
        Some(inner)
    );
    let restored = shadowed.with_declarations(unshadowed).await.unwrap();
    let again = restored.checked_selection(&[root], &cancel).await.unwrap();
    assert!(
        before
            .admission
            .revision
            .same_admission(&again.admission.revision)
    );
    assert_eq!(
        again.admission.revision.checked().resolve(root, "ambient"),
        Some(outer)
    );
}

/// Descriptor-only fixture: selected admission must not construct execution state.
#[derive(Debug)]
struct SelectedProviderConfiguration {
    spec: pse_kernels::ProviderSpec,
    configuration: pse_ids::ContentHash,
}
impl pse_kernels::ProviderFactory for SelectedProviderConfiguration {
    fn spec(&self) -> &pse_kernels::ProviderSpec {
        &self.spec
    }
    fn configuration_key(&self) -> pse_ids::ContentHash {
        self.configuration
    }
    fn create(&self) -> Result<Box<dyn pse_kernels::Provider>, pse_kernels::ProviderError> {
        Err(pse_kernels::ProviderError::Contract(
            "selected admission must not instantiate a provider worker".into(),
        ))
    }
}

#[tokio::test]
async fn checked_selected_admission_separates_provider_configuration_and_root_requests() {
    use pse_kernels::{
        AdmittedProvider, DerivativeOrder, DerivativeSource, Port, ProviderShapes, ProviderSpec,
        Registration,
    };
    let runtime = super::super::tests::runtime();
    let physical = super::super::tests::physical();
    let quantity = physical.quantities.neutral_dimensionless().unwrap();
    let unit = physical
        .quantities
        .quantity_type(quantity)
        .unwrap()
        .canonical_unit;
    let port = |n| Port {
        id: SemanticId::from_bytes([n; 16]),
        quantity,
        unit,
    };
    let spec = ProviderSpec {
        shapes: ProviderShapes::default(),
        derivative_source: DerivativeSource::Analytic,
        id: SemanticId::from_bytes([91; 16]),
        revision: pse_ids::ContentHash::from_bytes([92; 32]),
        data: pse_ids::ContentHash::from_bytes([93; 32]),
        inputs: vec![port(94)],
        outputs: vec![port(95)],
        derivatives: DerivativeOrder::First,
        smoothness: DerivativeOrder::First,
    };
    let registration = |configuration| {
        Registration::bind(
            AdmittedProvider::new(spec.clone(), &physical.quantities).unwrap(),
            Arc::new(SelectedProviderConfiguration {
                spec: spec.clone(),
                configuration: pse_ids::ContentHash::from_bytes([configuration; 32]),
            }),
        )
        .unwrap()
    };
    let provider_a = registration(96);
    let provider_b = registration(97);
    assert_eq!(provider_a.spec().key(), provider_b.spec().key());
    assert_ne!(
        provider_a.configuration_key(),
        provider_b.configuration_key()
    );
    let rows = declarations(&format!(
        "package p {{fn law(x:Scalar)->Scalar external \"law\" revision \"{}\" data \"{}\" output 0 derivatives 1 source analytic smoothness 1; def Root {{var x:Scalar; eq e:law(x)==2;}} def Other {{var y:Scalar; eq e:y==3;}}}}",
        spec.revision, spec.data
    ));
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let other = rows
        .iter()
        .find(|row| row.name == "Other")
        .unwrap()
        .declaration_id;
    let first = runtime
        .modeling_package_registered(
            rows,
            physical.clone(),
            BTreeMap::from([("law".into(), provider_a.clone())]),
        )
        .await
        .unwrap();
    let mut changed = first.clone();
    changed.providers = Arc::new(BTreeMap::from([("law".into(), provider_b.clone())]));
    assert_eq!(first.revision_id(), changed.revision_id());
    assert!(changed.canonical_revision().is_none());
    let cancel = crate::CancelSource::new();
    let a = first.checked_selection(&[root], &cancel).await.unwrap();
    let b = changed.checked_selection(&[root], &cancel).await.unwrap();
    assert_ne!(a.admission.request, b.admission.request);
    assert!(!Arc::ptr_eq(&a.admission, &b.admission));
    assert!(
        a.admission.revision.same_admission(&b.admission.revision),
        "equal descriptors retain the same checked topology independently of native configuration"
    );
    let math = runtime.shared.math();
    let instance = pse_modeling::specialize::root_instance(root);
    assert_ne!(
        math.basis_key(
            &a.admission,
            root,
            instance,
            Bindings::default(),
            Limits::default()
        )
        .unwrap(),
        math.basis_key(
            &b.admission,
            root,
            instance,
            Bindings::default(),
            Limits::default()
        )
        .unwrap(),
        "the complete staged request still separates configured preparation bases"
    );
    assert_eq!(
        first.registrations()[&spec.key()].configuration_key(),
        provider_a.configuration_key()
    );
    assert_eq!(
        changed.registrations()[&spec.key()].configuration_key(),
        provider_b.configuration_key()
    );
    for (package, expected) in [(&first, &provider_a), (&changed, &provider_b)] {
        let model = prepared(package).await;
        let registrations = package
            .inner_registrations(
                model,
                &pse_compiler::workspace::ModelingCaseBindings::default(),
                &crate::math::solves::NumericalInputs::default(),
                &pse_model::numerics::NumericalPolicy::default(),
                &pse_backend_native::solve::Controls::default(),
                DerivativeOrder::First,
                pse_compiler::workspace::Profile::default(),
                &cancel,
                implicit::ProviderDemand::Observations(None),
            )
            .await
            .unwrap();
        assert_eq!(
            registrations[&spec.key()].configuration_key(),
            expected.configuration_key(),
            "native provider handoff retains the package's actual configured factory"
        );
    }
    let wider = first
        .checked_selection(&[root, other], &cancel)
        .await
        .unwrap();
    assert_ne!(a.admission.request, wider.admission.request);
    assert!(
        !a.admission
            .revision
            .same_admission(&wider.admission.revision)
    );
    assert!(
        wider
            .admission
            .revision
            .declarations()
            .iter()
            .any(|row| row.declaration_id == other)
    );
    let again = first.checked_selection(&[root], &cancel).await.unwrap();
    assert!(
        a.admission
            .revision
            .same_admission(&again.admission.revision),
        "configuration and root variants preserve the independently eligible original request"
    );
}

#[tokio::test]
async fn checked_selected_admission_evicts_candidates_at_the_byte_limit() {
    let source = "package p {def Root {var x:Scalar; eq e:x*x==1;}}";
    let rows = declarations(source);
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let cancel = crate::CancelSource::new();
    let probe_runtime = super::super::tests::runtime();
    let probe_package = probe_runtime
        .modeling_package(rows.clone(), super::super::tests::physical())
        .await
        .unwrap();
    let probe = probe_package
        .checked_selection(&[root], &cancel)
        .await
        .unwrap();
    // Allow the key and entry overhead as well as one actual admitted payload;
    // keep the limit strictly below two payloads. No cache internals are replaced.
    let artifact_bytes =
        probe.admission.retained_bytes() + probe.admission.request.retained_bytes() + 4096;
    assert!(artifact_bytes < 2 * probe.admission.retained_bytes());
    probe_runtime.shared.math().clear_program_cache();
    drop(probe);
    drop(probe_package);
    drop(probe_runtime);
    let runtime = super::super::tests::runtime_on(
        512 << 20,
        crate::math::MathPolicy {
            artifact_bytes,
            workspace_bytes: 16 << 20,
            foreign_bytes: 1 << 20,
            worker_bytes: 8 << 20,
            ..Default::default()
        },
    );
    let pool = runtime.shared.pool();
    let baseline = pool.reserved();
    let first = runtime
        .modeling_package(rows, super::super::tests::physical())
        .await
        .unwrap();
    let a = first.checked_selection(&[root], &cancel).await.unwrap();
    let held = a.admission.revision.clone();
    let request = a.admission.request.clone();
    assert_eq!(
        runtime
            .shared
            .math()
            .modeling_cache
            .selected(&request)
            .unwrap()
            .len(),
        1
    );
    drop(a);
    let changed = first
        .with_declarations(declarations(
            "package p {def Root {var x:Scalar; eq e:x*x==2;}}",
        ))
        .await
        .unwrap();
    let b = changed.checked_selection(&[root], &cancel).await.unwrap();
    let retained = runtime
        .shared
        .math()
        .modeling_cache
        .selected(&request)
        .unwrap();
    assert_eq!(
        retained.len(),
        1,
        "the bounded cache must discard the older candidate"
    );
    assert!(retained[0].revision.same_admission(&b.admission.revision));
    assert!(!held.same_admission(&retained[0].revision));
    drop(retained);
    drop(b);
    let restored = changed
        .with_declarations(declarations(source))
        .await
        .unwrap();
    let fresh = restored.checked_selection(&[root], &cancel).await.unwrap();
    assert!(
        !held.same_admission(&fresh.admission.revision),
        "evicted A requires a fresh admission even though its escaped immutable owner remains live"
    );
    assert_eq!(held.identity(), fresh.admission.revision.identity());
    runtime.shared.math().clear_program_cache();
    drop(fresh);
    drop(restored);
    drop(changed);
    drop(first);
    assert!(
        pool.reserved() > baseline,
        "escaped evicted admission retains its charged allocation"
    );
    drop(held);
    assert_eq!(pool.reserved(), baseline);
}

#[tokio::test(flavor = "current_thread")]
async fn checked_selected_admission_cancelled_before_native_entry_retains_owners_until_drain() {
    use std::task::Poll;
    let runtime = super::super::tests::runtime();
    let service = runtime.shared.math();
    let pool = runtime.shared.pool();
    let baseline = pool.reserved();
    let rows = declarations("package p {def Root {var x:Scalar; eq e:x*x==1;}}");
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime
        .modeling_package(rows, super::super::tests::physical())
        .await
        .unwrap();
    let (admitted, admission) = tokio::sync::oneshot::channel();
    let (exit, gate) = std::sync::mpsc::channel::<()>();
    let mut submission = crate::math::Submission::ephemeral();
    submission.admitted = Some(admitted);
    // Existing production admission holds every CPU permit. Dropping `exit` on
    // assertion failure also opens the gate, so the native blocker cannot escape.
    let blocker = service
        .submit_with(2, 0, submission, move |_, _| {
            gate.recv().map_err(|error| {
                crate::math::MathRuntimeError::Infrastructure(error.to_string())
            })?;
            Ok(((), 0))
        })
        .unwrap();
    admission.await.unwrap();
    let before_selection = pool.reserved();
    let cancel = crate::CancelSource::new();
    let roots = [root];
    let mut selection = Box::pin(package.checked_selection(&roots, &cancel));
    // The timeout is a deadlock watchdog; the gate and actual flight table, rather
    // than elapsed time or a delay, establish that native entry is unavailable.
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        while service.selected_flights.active() == 0 {
            assert!(futures_util::poll!(selection.as_mut()).is_pending());
            if service.selected_flights.active() == 0 {
                tokio::task::yield_now().await;
            }
        }
    })
    .await
    .unwrap();
    assert_eq!(service.selected_flights.active(), 1);
    assert!(
        pool.reserved() > before_selection,
        "the actual pending selected flight owns its source, request and admission allocations"
    );
    cancel.cancel();
    // Poll cancellation inline: on this current-thread executor the detached
    // supervisor cannot run between waiter departure and these observations.
    let cancelled = futures_util::poll!(selection.as_mut());
    assert_eq!(
        service.selected_flights.active(),
        1,
        "departing last waiter cannot remove the completion-owned entry"
    );
    assert!(
        pool.reserved() > before_selection,
        "caller cancellation cannot prematurely release the pending job's input owners"
    );
    let result = match cancelled {
        Poll::Ready(result) => result,
        Poll::Pending => selection.as_mut().await,
    };
    assert!(matches!(
        result,
        Err(WorkflowError::Math(
            crate::math::MathRuntimeError::Cancelled
        ))
    ));
    drop(selection);
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        while service.selected_flights.active() != 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert_eq!(
        pool.reserved(),
        before_selection,
        "the selected supervisor releases its owners on actual drain, independently of the unrelated blocked job"
    );
    exit.send(()).unwrap();
    blocker.finish().await.unwrap();
    let retry = package
        .checked_selection(&[root], &crate::CancelSource::new())
        .await
        .unwrap();
    assert!(!retry.admission.revision.declarations().is_empty());
    service.clear_program_cache();
    drop(retry);
    drop(package);
    assert_eq!(pool.reserved(), baseline);
}

async fn retained_preparation_basis(
    package: &ModelingPackage,
    root: DeclarationId,
    instance: InstanceId,
    bindings: Bindings,
    limits: Limits,
) -> Option<Arc<crate::math::modeling::PreparedBasis>> {
    let selected = package
        .checked_selection(&[root], &crate::CancelSource::new())
        .await
        .unwrap();
    let math = package.runtime.shared.math();
    let key = math
        .basis_key(&selected.admission, root, instance, bindings, limits)
        .unwrap();
    let basis = math.modeling_cache.basis(&key);
    basis
}

#[tokio::test]
async fn ordinary_preparation_basis_separates_actual_instance_and_complete_bindings_and_limits() {
    use pse_modeling::specialize::Value;
    let runtime = super::super::tests::runtime();
    let rows = declarations("package p {def Root {var x:Scalar; eq e:x*x==1;}}");
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let instance = pse_modeling::specialize::root_instance(root);
    let package = runtime
        .modeling_package(rows, super::super::tests::physical())
        .await
        .unwrap();
    let first = prepared(&package).await;
    let selected = package
        .checked_selection(&[root], &crate::CancelSource::new())
        .await
        .unwrap();
    let math = runtime.shared.math();
    let original = math
        .basis_key(
            &selected.admission,
            root,
            instance,
            Bindings::default(),
            Limits::default(),
        )
        .unwrap();
    assert!(math.modeling_cache.basis(&original).is_some());
    let mut variants = Vec::new();
    let mut arguments = Bindings::default();
    arguments
        .arguments
        .insert("explicit".into(), Value::Integer(1));
    variants.push(arguments);
    let mut scope = Bindings::default();
    scope.scope.insert("ancestor".into(), Value::Boolean(true));
    variants.push(scope);
    let mut facts = Bindings::default();
    facts.facts.insert(
        pse_modeling::analysis::Fact::Stage("selected".into()),
        Value::Boolean(true),
    );
    variants.push(facts);
    let demand = Bindings {
        demand: vec!["x".into()],
        ..Bindings::default()
    };
    variants.push(demand.clone());
    let mut omission = Bindings::default();
    omission
        .formulation
        .omitted
        .insert(SemanticId::from_bytes([91; 16]));
    variants.push(omission);
    let mut elastic = Bindings::default();
    elastic
        .formulation
        .elastic
        .insert(SemanticId::from_bytes([91; 16]), Value::Integer(1));
    variants.push(elastic);
    for bindings in variants {
        let changed = math
            .basis_key(
                &selected.admission,
                root,
                instance,
                bindings,
                Limits::default(),
            )
            .unwrap();
        assert_ne!(changed, original);
        assert!(
            math.modeling_cache.basis(&changed).is_none(),
            "every complete binding field must prevent a hit on the different retained request"
        );
    }
    let default = Limits::default();
    let limits = [
        Limits {
            depth: default.depth - 1,
            ..default
        },
        Limits {
            items: default.items - 1,
            ..default
        },
        Limits {
            members: default.members - 1,
            ..default
        },
        Limits {
            body_occurrences: Some(100_000),
            ..default
        },
        Limits {
            body_slots: Some(100_000),
            ..default
        },
    ];
    for limit in limits {
        let changed = math
            .basis_key(
                &selected.admission,
                root,
                instance,
                Bindings::default(),
                limit,
            )
            .unwrap();
        assert_ne!(changed, original);
        assert!(
            math.modeling_cache.basis(&changed).is_none(),
            "every limit remains part of the exact retained request"
        );
    }
    drop(selected);
    let other_instance = InstanceId::from_id(SemanticId::from_bytes([92; 16]));
    assert!(
        retained_preparation_basis(&package, root, other_instance, Bindings::default(), default)
            .await
            .is_none()
    );
    let other = package
        .prepare(
            root,
            other_instance,
            Bindings::default(),
            default,
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert_eq!(other.solved().instance(), other_instance);
    assert_ne!(other.solved(), first.solved());
    assert!(!pse_math::SharedAllocation::ptr_eq(
        &other.compiled().admitted,
        &first.compiled().admitted
    ));
    let demanded = package
        .prepare(
            root,
            instance,
            demand.clone(),
            default,
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert!(
        retained_preparation_basis(&package, root, instance, demand, default)
            .await
            .is_some()
    );
    assert_eq!(demanded.solved(), first.solved());
    let bounded = package
        .prepare(
            root,
            instance,
            Bindings::default(),
            limits[1],
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert_eq!(bounded.solved(), first.solved());
    assert!(
        retained_preparation_basis(&package, root, instance, Bindings::default(), limits[1])
            .await
            .is_some()
    );
    let refused = package
        .prepare(
            root,
            instance,
            Bindings::default(),
            Limits {
                items: 0,
                ..default
            },
            &crate::CancelSource::new(),
        )
        .await;
    assert!(
        refused.is_err(),
        "a warm basis cannot bypass a current finite specialization refusal"
    );
    math.clear_program_cache();
}

#[tokio::test]
async fn ordinary_preparation_source_spans_and_exact_dependencies_prevent_stale_basis_hits() {
    let runtime = super::super::tests::runtime();
    let source = "package p {def Root {var x:Scalar; eq e:x*x==1;}}";
    let rows = declarations(source);
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let instance = pse_modeling::specialize::root_instance(root);
    let package = runtime
        .modeling_package(rows, super::super::tests::physical())
        .await
        .unwrap();
    let first = prepared(&package).await;
    let shifted = package
        .with_declarations(declarations(&format!("\n\n{source}")))
        .await
        .unwrap();
    assert!(
        retained_preparation_basis(
            &shifted,
            root,
            instance,
            Bindings::default(),
            Limits::default()
        )
        .await
        .is_none()
    );
    let rebound = prepared(&shifted).await;
    assert!(!pse_math::SharedAllocation::ptr_eq(
        &first.compiled().admitted,
        &rebound.compiled().admitted
    ));
    assert_ne!(
        first.compiled().occurrences(),
        rebound.compiled().occurrences()
    );
    assert_ne!(
        first.consumed_source_versions(),
        rebound.consumed_source_versions()
    );
    for (key, body) in &first.compiled().admitted.bodies {
        assert!(
            Arc::ptr_eq(body.math(), rebound.compiled().admitted.bodies[key].math()),
            "source attribution changes do not force duplication of unchanged immutable body math"
        );
    }
    let changed = shifted
        .with_declarations(declarations(
            "\n\npackage p {def Root {var x:Scalar; eq e:x*x==2;}}",
        ))
        .await
        .unwrap();
    assert!(
        retained_preparation_basis(
            &changed,
            root,
            instance,
            Bindings::default(),
            Limits::default()
        )
        .await
        .is_none()
    );
    let actual = prepared(&changed).await;
    assert!(!pse_math::SharedAllocation::ptr_eq(
        &rebound.compiled().admitted,
        &actual.compiled().admitted
    ));
    assert_ne!(
        rebound.consumed_source_versions(),
        actual.consumed_source_versions()
    );
    // This inventory contains the complete residual and independent original terms.
    // The unchanged x*x term may share mathematics; the consumed changed residual may not.
    fn body_for_row(
        model: &ModelingPreparation,
        row: SemanticId,
    ) -> &Arc<pse_compiler::typed_math::AdmittedBody> {
        let mut contributing =
            model
                .compiled()
                .admitted
                .case()
                .instances()
                .iter()
                .filter(|instance| {
                    instance.contributions.iter().any(|contribution| {
                        contribution.target == pse_math::binding::Target::Row(row)
                    })
                });
        let instance = contributing
            .next()
            .expect("fixture row has one consumed mathematical body");
        assert!(
            contributing.next().is_none(),
            "fixture row must identify its exact body"
        );
        &model.compiled().admitted.bodies[&instance.body]
    }
    fn equation(model: &ModelingPreparation) -> SemanticId {
        let mut equations = model
            .compiled()
            .admitted
            .outputs
            .iter()
            .filter_map(|output| match output {
                pse_compiler::workspace::ModelingOutput::Equation { id, .. } => Some(*id),
                _ => None,
            });
        let id = equations
            .next()
            .expect("fixture has its authored equality residual");
        assert!(equations.next().is_none());
        id
    }
    let row = equation(&rebound);
    assert_eq!(equation(&actual), row);
    let old_residual = body_for_row(&rebound, row);
    let new_residual = body_for_row(&actual, row);
    assert_ne!(
        old_residual.semantic_identity(),
        new_residual.semantic_identity(),
        "x*x-1 and x*x-2 are different complete scientific residual descriptions"
    );
    assert!(
        !Arc::ptr_eq(old_residual.math(), new_residual.math()),
        "the changed equality residual cannot reuse the old immutable mathematical body"
    );
    let old_term = rebound.compiled().admitted.term_outputs[&row][0].0;
    let new_term = actual.compiled().admitted.term_outputs[&row][0].0;
    assert!(
        Arc::ptr_eq(
            body_for_row(&rebound, old_term).math(),
            body_for_row(&actual, new_term).math()
        ),
        "the independent unchanged x*x inspection term remains eligible for mathematical sharing"
    );
    let x = rebound
        .compiled()
        .model
        .symbols
        .values()
        .find(|symbol| symbol.lineage.path == "x" || symbol.lineage.path.ends_with(".x"))
        .unwrap()
        .id;
    let values = pse_math::binding::CaseValues {
        scalars: BTreeMap::from([(x, 2.0)]),
    };
    assert_eq!(rebound.compiled().admitted.term_outputs[&row][0].1, 1.0);
    assert_eq!(actual.compiled().admitted.term_outputs[&row][0].1, 1.0);
    let old_value = shifted
        .observe(
            rebound.clone(),
            BTreeSet::from([row, old_term]),
            values.clone(),
            super::super::tests::compiler_profile(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let new_value = changed
        .observe(
            actual.clone(),
            BTreeSet::from([row, new_term]),
            values.clone(),
            super::super::tests::compiler_profile(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert_eq!(
        old_value[&old_term], 4.0,
        "the unchanged positive original term evaluates independently as x*x"
    );
    assert_eq!(new_value[&new_term], 4.0);
    assert_eq!(
        old_value[&row], 3.0,
        "the original residual at x=2 is independently x*x-1"
    );
    assert_eq!(
        new_value[&row], 2.0,
        "the current consumed scientific residual is independently x*x-2"
    );
    let fresh_runtime = super::super::tests::runtime();
    let fresh = fresh_runtime
        .modeling_package(
            declarations("\n\npackage p {def Root {var x:Scalar; eq e:x*x==2;}}"),
            super::super::tests::physical(),
        )
        .await
        .unwrap();
    let fresh_model = prepared(&fresh).await;
    assert!(
        !Arc::ptr_eq(new_residual.math(), body_for_row(&fresh_model, row).math()),
        "the oracle uses a separate runtime and independently admitted body"
    );
    let fresh_value = fresh
        .observe(
            fresh_model,
            BTreeSet::from([row, new_term]),
            values,
            super::super::tests::compiler_profile(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert_eq!(&*new_value, &*fresh_value);
    fresh_runtime.shared.math().clear_program_cache();
    let selected = changed
        .checked_selection(&[root], &crate::CancelSource::new())
        .await
        .unwrap();
    assert_eq!(
        actual.consumed_source_versions(),
        selected.admission.versions.as_ref()
    );
    runtime.shared.math().clear_program_cache();
}

#[tokio::test]
async fn ordinary_preparation_basis_prehash_preserves_exact_identity_under_forced_collisions() {
    use std::hash::{Hash, Hasher};
    #[derive(Debug, Default)]
    struct HashWrites {
        writes: usize,
        bytes: usize,
    }
    impl Hasher for HashWrites {
        fn finish(&self) -> u64 {
            0
        }
        fn write(&mut self, bytes: &[u8]) {
            self.writes += 1;
            self.bytes += bytes.len();
        }
    }
    fn hash(key: &crate::math::modeling::BasisKey) -> u64 {
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hash);
        hash.finish()
    }
    let runtime = super::super::tests::runtime();
    let original_rows = declarations("package p {def Root {var x:Scalar; eq e:x*x==1;}}");
    let root = original_rows
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let instance = pse_modeling::specialize::root_instance(root);
    let package = runtime
        .modeling_package(original_rows, super::super::tests::physical())
        .await
        .unwrap();
    let first = prepared(&package).await;
    let first_basis = retained_preparation_basis(
        &package,
        root,
        instance,
        Bindings::default(),
        Limits::default(),
    )
    .await
    .unwrap();
    let changed = package
        .with_declarations(declarations(
            "package p {def Root {var x:Scalar; eq e:x*x==2;}}",
        ))
        .await
        .unwrap();
    let second = prepared(&changed).await;
    let second_basis = retained_preparation_basis(
        &changed,
        root,
        instance,
        Bindings::default(),
        Limits::default(),
    )
    .await
    .unwrap();
    assert!(
        !Arc::ptr_eq(&first_basis, &second_basis),
        "the collision control must use two distinct prepared products"
    );
    let selected = package
        .checked_selection(&[root], &crate::CancelSource::new())
        .await
        .unwrap();
    let different = changed
        .checked_selection(&[root], &crate::CancelSource::new())
        .await
        .unwrap();
    let service = runtime.shared.math();
    let mut first_key = service
        .basis_key(
            &selected.admission,
            root,
            instance,
            Bindings::default(),
            Limits::default(),
        )
        .unwrap();
    let mut equal_key = service
        .basis_key(
            &selected.admission,
            root,
            instance,
            Bindings::default(),
            Limits::default(),
        )
        .unwrap();
    let mut second_key = service
        .basis_key(
            &different.admission,
            root,
            instance,
            Bindings::default(),
            Limits::default(),
        )
        .unwrap();
    assert_eq!(first_key, equal_key);
    assert_eq!(hash(&first_key), hash(&equal_key));
    assert_ne!(
        first_key, second_key,
        "changed exact source dependencies remain unequal"
    );
    let mut writes = HashWrites::default();
    first_key.hash(&mut writes);
    assert_eq!(
        (writes.writes, writes.bytes),
        (1, size_of::<u64>()),
        "table callbacks consume the cached hash rather than traversing source fields"
    );
    // Install only forced keys in the cleared cache: every equal test key gets the
    // same forced prehash before publication, preserving Hash/Eq's table contract.
    for key in [&mut first_key, &mut equal_key, &mut second_key] {
        Arc::get_mut(key).unwrap().force_prehash_for_test(7);
    }
    assert_eq!(hash(&first_key), hash(&second_key));
    assert_eq!(first_key, equal_key);
    assert_ne!(first_key, second_key);
    service.clear_program_cache();
    let generation = service.modeling_cache.generation();
    service
        .modeling_cache
        .retain_basis(generation, first_key.clone(), first_basis.clone());
    service
        .modeling_cache
        .retain_basis(generation, second_key.clone(), second_basis.clone());
    assert!(Arc::ptr_eq(
        &service.modeling_cache.basis(&first_key).unwrap(),
        &first_basis
    ));
    assert!(Arc::ptr_eq(
        &service.modeling_cache.basis(&equal_key).unwrap(),
        &first_basis
    ));
    assert!(Arc::ptr_eq(
        &service.modeling_cache.basis(&second_key).unwrap(),
        &second_basis
    ));
    drop((first, second));
    service.clear_program_cache();
}
