// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual runtime ingress/edit/receiving contrasts as unrelated source grows.
use super::*;
use crate::math::modeling::ModelingPreparation;
use crate::workflow::{SourceDelta, replay::ReplaySourceDocument};
use std::time::Instant;

fn rows(size: usize, default: u8, nonlinear: bool) -> Vec<Declaration> {
    let expression = if nonlinear { "x*x==k" } else { "x==k" };
    let mut text = format!(
        "package p {{ def Root {{ param k:Scalar={default}; var x:Scalar; eq balance:{expression}; }}"
    );
    for i in 0..size {
        text.push_str(&format!("param spare{i}:Scalar={i};"));
    }
    text.push('}');
    pse_authoring::language::parse(
        &text,
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        Default::default(),
    )
    .unwrap()
}
async fn prepare(package: &ModelingPackage, root: DeclarationId) -> ModelingPreparation {
    package
        .prepare(
            root,
            pse_modeling::specialize::root_instance(root),
            Default::default(),
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn runtime_source_size_ladder_preserves_local_edits_noops_and_old_products() {
    for size in [8, 32, 128] {
        let runtime = crate::workflow::tests::runtime();
        let physical = crate::workflow::tests::physical();
        let service = runtime.shared.math();
        let cancel = crate::CancelSource::new();
        let original_rows = rows(size, 1, false);
        let root = original_rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let stable = original_rows
            .iter()
            .find(|row| row.name == "x")
            .unwrap()
            .declaration_id;
        let started = Instant::now();
        let original = runtime
            .modeling_package(original_rows.clone(), physical.clone())
            .await
            .unwrap();
        eprintln!(
            "source-size size={size} phase=typed-ingress elapsed_ns={} declarations={}",
            started.elapsed().as_nanos(),
            original_rows.len()
        );
        let before = service.preparation_effects();
        let started = Instant::now();
        let old_product = prepare(&original, root).await;
        let cold = service.preparation_effects().since(before);
        assert!(cold.workspaces > 0 && cold.frontiers > 0 && cold.completions > 0);
        eprintln!(
            "source-size size={size} phase=first-preparation elapsed_ns={} effects={cold:?}",
            started.elapsed().as_nanos()
        );
        let old_structure = old_product
            .compiled()
            .bound_structure(&BTreeMap::new())
            .unwrap()
            .structure
            .key();

        let before = service.preparation_effects();
        let started = Instant::now();
        let equal = original
            .with_declarations(original_rows.clone())
            .await
            .unwrap();
        let repeated = prepare(&equal, root).await;
        assert_eq!(equal.revision_id(), original.revision_id());
        assert!(pse_math::SharedAllocation::ptr_eq(
            &old_product.compiled().admitted,
            &repeated.compiled().admitted
        ));
        let warm = service.preparation_effects().since(before);
        assert_eq!(warm, crate::math::PreparationEffectCounts::default());
        eprintln!(
            "source-size size={size} phase=equal-preparation elapsed_ns={} effects={warm:?}",
            started.elapsed().as_nanos()
        );

        let before = service.preparation_effects();
        let started = Instant::now();
        let equal_ingress = runtime
            .modeling_package(original_rows.clone(), physical.clone())
            .await
            .unwrap();
        let equal_product = prepare(&equal_ingress, root).await;
        let effects = service.preparation_effects().since(before);
        eprintln!(
            "source-size size={size} phase=equal-fresh-ingress elapsed_ns={} effects={effects:?}",
            started.elapsed().as_nanos()
        );
        assert_eq!(
            equal_ingress.revision.identity(),
            original.revision.identity()
        );
        assert_ne!(equal_ingress.revision_id(), original.revision_id());
        assert_eq!(
            equal_product
                .compiled()
                .bound_structure(&BTreeMap::new())
                .unwrap()
                .structure
                .key(),
            old_structure
        );

        let mut current = equal;
        let mut expected = original_rows.clone();
        for phase in ["parameter-default", "span", "expression", "deletion"] {
            let mut delta = SourceDelta {
                parent: current.revision_id().into(),
                ..Default::default()
            };
            if phase == "deletion" {
                let removed = expected
                    .iter()
                    .find(|row| row.name == format!("spare{}", size - 1))
                    .unwrap()
                    .declaration_id;
                delta.removals.push(removed);
                expected.retain(|row| row.declaration_id != removed);
            } else {
                let replacement = match phase {
                    "parameter-default" => rows(size, 2, false)
                        .into_iter()
                        .find(|row| row.name == "k")
                        .unwrap(),
                    "expression" => rows(size, 2, true)
                        .into_iter()
                        .find(|row| row.name == "balance")
                        .unwrap(),
                    "span" => {
                        let mut row = expected
                            .iter()
                            .find(|row| row.name == "balance")
                            .unwrap()
                            .clone();
                        row.source_start += 17;
                        row.source_end += 17;
                        row
                    }
                    phase => panic!("unsupported source-edit fixture phase {phase}"),
                };
                *expected
                    .iter_mut()
                    .find(|row| row.declaration_id == replacement.declaration_id)
                    .unwrap() = replacement.clone();
                delta.replacements.push(replacement);
            }
            let prior_identity = current.revision.identity();
            let stable_owner = current.revision.admitted.objects[&logical(stable)].clone();
            let before = service.preparation_effects();
            let started = Instant::now();
            let next = current.with_delta(delta, &cancel).await.unwrap();
            let product = prepare(&next, root).await;
            let effects = service.preparation_effects().since(before);
            eprintln!(
                "source-size size={size} phase={phase} elapsed_ns={} effects={effects:?}",
                started.elapsed().as_nanos()
            );
            assert_ne!(next.revision.identity(), prior_identity);
            assert_eq!(&*next.declarations().await.unwrap(), expected.as_slice());
            assert!(Arc::ptr_eq(
                &stable_owner,
                &next.revision.admitted.objects[&logical(stable)]
            ));
            // An independent full ingress is the source identity/order oracle; the
            // local edit does not substitute map equality for receiving admission.
            let fresh = runtime
                .modeling_package(expected.clone(), physical.clone())
                .await
                .unwrap();
            assert_eq!(next.revision.identity(), fresh.revision.identity());
            assert_eq!(&*fresh.declarations().await.unwrap(), expected.as_slice());
            assert!(product.compiled().bound_structure(&BTreeMap::new()).is_ok());
            current = next;
        }
        assert_eq!(
            &*original.declarations().await.unwrap(),
            original_rows.as_slice()
        );
        assert_eq!(
            old_product
                .compiled()
                .bound_structure(&BTreeMap::new())
                .unwrap()
                .structure
                .key(),
            old_structure
        );
        let retained = prepare(&original, root).await;
        assert_eq!(
            retained
                .compiled()
                .bound_structure(&BTreeMap::new())
                .unwrap()
                .structure
                .key(),
            old_structure
        );
    }
}

#[tokio::test]
async fn runtime_source_size_ladder_byte_receivers_check_full_source_predicates() {
    for size in [8, 32, 128] {
        let runtime = crate::workflow::tests::runtime();
        let physical = crate::workflow::tests::physical();
        let expected = rows(size, 1, false);
        let root = expected
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let original = runtime
            .modeling_package(expected.clone(), physical.clone())
            .await
            .unwrap();
        let document = original.replay_source().unwrap();
        let bytes = serde_json::to_vec(&document).unwrap();
        let received: ReplaySourceDocument = serde_json::from_slice(&bytes).unwrap();
        let started = Instant::now();
        // Public receiving API runs the actual registry, references, source-frame,
        // physical/provider and generated-row predicates on deserialized bytes.
        let imported = received
            .reconstruct(&runtime, physical.clone(), BTreeMap::new())
            .unwrap();
        eprintln!(
            "source-size size={size} phase=byte-receive elapsed_ns={} wire_bytes={}",
            started.elapsed().as_nanos(),
            bytes.len()
        );
        assert_eq!(imported.revision.identity(), original.revision.identity());
        assert_ne!(imported.revision_id(), original.revision_id());
        assert_eq!(
            &*imported.declarations().await.unwrap(),
            expected.as_slice()
        );
        let before = runtime.shared.math().preparation_effects();
        let product = prepare(&imported, root).await;
        assert!(
            runtime
                .shared
                .math()
                .preparation_effects()
                .since(before)
                .workspaces
                > 0
        );
        assert!(product.compiled().bound_structure(&BTreeMap::new()).is_ok());
        let mut changed = received.clone();
        changed
            .objects
            .iter_mut()
            .find(|object| object.logical == logical(root))
            .unwrap()
            .payload
            .push(b' ');
        assert!(
            changed
                .reconstruct(&runtime, physical.clone(), BTreeMap::new())
                .is_err(),
            "same claimed identity cannot authorize changed actual source bytes"
        );
        assert_eq!(
            &*original.declarations().await.unwrap(),
            expected.as_slice()
        );
        assert_eq!(
            &*imported.declarations().await.unwrap(),
            expected.as_slice()
        );
    }
}
