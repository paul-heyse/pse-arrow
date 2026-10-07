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

#[tokio::test]
async fn checked_selected_admission_rechecks_fresh_protection_and_retains_a_b_a() {
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
    runtime
        .canonical
        .store()
        .release(a.read.selection())
        .await
        .unwrap();
    let hit = first.checked_selection(&[root], &cancel).await.unwrap();
    assert!(held.same_admission(&hit.admission.revision));
    // The old pin is closed: success must have acquired independent current authority.
    assert!(
        runtime
            .canonical
            .store()
            .recheck_selection_dependencies(&mut a.read.clone(), &a.admission.dependencies)
            .await
            .is_err()
    );
    runtime
        .canonical
        .store()
        .release(hit.read.selection())
        .await
        .unwrap();
    let unrelated = first.with_declarations(declarations("package p { def Root { var x:Scalar; eq a:x*x==1; } def Other { var z:Scalar; eq b:z==3; } }")).await.unwrap();
    let unchanged = unrelated.checked_selection(&[root], &cancel).await.unwrap();
    assert!(
        held.same_admission(&unchanged.admission.revision),
        "a changed unconsumed body preserves checked selected input"
    );
    runtime
        .canonical
        .store()
        .release(unchanged.read.selection())
        .await
        .unwrap();
    let changed = unrelated.with_declarations(declarations("package p { def Root { var x:Scalar; eq a:x*x==2; } def Other { var z:Scalar; eq b:z==3; } }")).await.unwrap();
    let b = changed.checked_selection(&[root], &cancel).await.unwrap();
    assert!(
        !held.same_admission(&b.admission.revision),
        "structural literal changes must re-admit"
    );
    runtime
        .canonical
        .store()
        .release(b.read.selection())
        .await
        .unwrap();
    let restored = changed
        .with_declarations(declarations(source))
        .await
        .unwrap();
    let again = restored.checked_selection(&[root], &cancel).await.unwrap();
    assert!(
        held.same_admission(&again.admission.revision),
        "A/B/A retains complete eligible candidates"
    );
    runtime
        .canonical
        .store()
        .release(again.read.selection())
        .await
        .unwrap();
    runtime.shared.math().clear_program_cache();
    let fresh = restored.checked_selection(&[root], &cancel).await.unwrap();
    assert!(
        !held.same_admission(&fresh.admission.revision),
        "clear forces current admission without invalidating escaped immutable owners"
    );
    runtime
        .canonical
        .store()
        .release(fresh.read.selection())
        .await
        .unwrap();
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
    runtime
        .canonical
        .store()
        .release(before.read.selection())
        .await
        .unwrap();
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
    assert!(
        !runtime
            .canonical
            .store()
            .recheck_selection_dependencies(&mut after.read.clone(), &before.admission.dependencies)
            .await
            .unwrap(),
        "an absent-to-present rule invalidates the complete cached receipt"
    );
    runtime
        .canonical
        .store()
        .release(after.read.selection())
        .await
        .unwrap();
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
    runtime
        .canonical
        .store()
        .release(before.read.selection())
        .await
        .unwrap();
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
    assert!(
        !runtime
            .canonical
            .store()
            .recheck_selection_dependencies(&mut after.read.clone(), &before.admission.dependencies)
            .await
            .unwrap(),
        "unchanged outer membership cannot authorize reuse after a nearer name appears"
    );
    runtime
        .canonical
        .store()
        .release(after.read.selection())
        .await
        .unwrap();
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
    runtime
        .canonical
        .store()
        .release(again.read.selection())
        .await
        .unwrap();
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
    let changed = runtime
        .modeling_revision(
            first.canonical_revision().clone(),
            physical,
            BTreeMap::from([("law".into(), provider_b)]),
        )
        .await
        .unwrap();
    assert_eq!(first.canonical_revision(), changed.canonical_revision());
    let cancel = crate::CancelSource::new();
    let a = first.checked_selection(&[root], &cancel).await.unwrap();
    runtime
        .canonical
        .store()
        .release(a.read.selection())
        .await
        .unwrap();
    let b = changed.checked_selection(&[root], &cancel).await.unwrap();
    assert_ne!(a.admission.request, b.admission.request);
    assert!(!a.admission.revision.same_admission(&b.admission.revision));
    runtime
        .canonical
        .store()
        .release(b.read.selection())
        .await
        .unwrap();
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
    runtime
        .canonical
        .store()
        .release(wider.read.selection())
        .await
        .unwrap();
    let again = first.checked_selection(&[root], &cancel).await.unwrap();
    assert!(
        a.admission
            .revision
            .same_admission(&again.admission.revision),
        "configuration and root variants preserve the independently eligible original request"
    );
    runtime
        .canonical
        .store()
        .release(again.read.selection())
        .await
        .unwrap();
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
    probe_runtime
        .canonical
        .store()
        .release(probe.read.selection())
        .await
        .unwrap();
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
    runtime
        .canonical
        .store()
        .release(a.read.selection())
        .await
        .unwrap();
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
    runtime
        .canonical
        .store()
        .release(b.read.selection())
        .await
        .unwrap();
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
    runtime
        .canonical
        .store()
        .release(fresh.read.selection())
        .await
        .unwrap();
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
    runtime
        .canonical
        .store()
        .release(retry.read.selection())
        .await
        .unwrap();
    service.clear_program_cache();
    drop(retry);
    drop(package);
    assert_eq!(pool.reserved(), baseline);
}
