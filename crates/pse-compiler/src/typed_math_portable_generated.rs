// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Private passive Recipe correspondence; production qualification still consumes JSON v3.
#![allow(
    dead_code,
    reason = "Private generated Recipe correspondence is compiled before production codec activation"
)]

use pse_contract::boundary_projection::{AdapterError, AdapterFailure};
use pse_contract_proto::support::*;

#[path = "generated/scientific-recipe/mod.rs"]
#[rustfmt::skip]
#[allow(unreachable_pub, elided_lifetimes_in_paths, trivial_numeric_casts, missing_debug_implementations, missing_docs, clippy::upper_case_acronyms, reason = "Pinned protobuf generator surface remains inside the private Recipe owner")]
pub(super) mod pb;

#[path = "generated/scientific-recipe/validation/mod.rs"]
#[rustfmt::skip]
#[allow(unused_qualifications, elided_lifetimes_in_paths, trivial_numeric_casts, unreachable_pub, missing_docs, reason = "Pinned validator generator packaging")]
mod validation;

#[path = "generated/scientific-recipe/native-adapters.rs"]
#[rustfmt::skip]
mod adapters;

#[path = "generated/scientific-recipe/native-view-adapters.rs"]
#[rustfmt::skip]
mod view_adapters;

#[path = "generated/scientific-recipe/native-policy.rs"]
#[rustfmt::skip]
#[allow(unreachable_pub, reason = "Source-derived policy stays private with its passive Recipe owner")]
mod source_policy;

// Full F03 receiving admission is deliberately absent. These compiler-private
// receivers are compiled only for exact-root/membership prerequisite controls.
#[cfg(test)]
#[allow(
    unsafe_code,
    reason = "ADR-0245 isolated exact-byte compiler receiver controls; production protobuf activation remains withheld"
)]
mod native_receiving {
    use super::{pb::pse::scientific_recipe::v4 as wire, view_adapters};
    use crate::typed_math::AdmittedBody;
    use crate::typed_math::portable::{Recipe, contract, reconstruct_recipe};
    use pse_ids::{
        ContentHash,
        roles::SemanticBodyHash,
        scientific_replay::{
            NATIVE_RECIPE_INTERPRETATION, NativeMembershipLimits, NativeRecipeScope,
            NativeRecipeSource, RecordAuthority, count_native_membership,
            native_recipe_payload_hash,
        },
    };
    use pse_math::{MathError, binding::BodySpec};
    use pse_quantity::QuantityRegistry;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    /// Traversal-only preadmission. This does not cover the decoded root, views,
    /// complete validation, native adoption, receipt copies or reconstruction.
    #[derive(Clone, Copy)]
    struct TraversalAllowance {
        limits: NativeMembershipLimits,
        count_bytes: usize,
        index_bytes: usize,
        query_bytes: usize,
    }
    struct IndexedNative {
        recipe: Recipe,
        records: RecordAuthority,
        payload: Arc<[u8]>,
    }
    struct ReceivedNative {
        body: AdmittedBody,
        payload: Arc<[u8]>,
    }
    impl IndexedNative {
        fn reconstruct(
            self,
            registry: &QuantityRegistry,
            cancelled: &Arc<AtomicBool>,
        ) -> Result<ReceivedNative, MathError> {
            // Move the SAME indexed root into the existing strict reconstruction.
            let mut body = reconstruct_recipe(self.recipe, &self.records, registry, cancelled)?;
            // Never put binary bytes through the still-active JSON-v3 portable leaf.
            body.portable = None;
            Ok(ReceivedNative {
                body,
                payload: self.payload,
            })
        }
    }
    /// # Safety
    /// The caller must establish the canonical or private origin premise required
    /// by its monomorphic wrapper, with the original request/qualification and
    /// physical owners. This test-only route supplies no enclosing F03 admission.
    unsafe fn index_exact_recipe(
        payload: Arc<[u8]>,
        scope: NativeRecipeScope,
        expected: &BodySpec,
        expected_identity: Option<SemanticBodyHash>,
        cancelled: &Arc<AtomicBool>,
        allowance: TraversalAllowance,
    ) -> Result<IndexedNative, MathError> {
        if cancelled.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        super::super::check_payload(&payload)?;
        if native_recipe_payload_hash(&payload) != scope.payload_digest {
            return Err(contract(
                "native recipe differs from original exact payload digest",
            ));
        }
        let view = buffa::DecodeOptions::new()
            .decode_view::<wire::__buffa::view::ScientificRecipeView<'_>>(&payload)
            .map_err(contract)?;
        let recipe = view_adapters::decode_scientific_recipe_view(&view).map_err(contract)?;
        drop(view);
        if recipe.interpretation != NATIVE_RECIPE_INTERPRETATION {
            return Err(contract("unsupported native recipe interpretation"));
        }
        if recipe.spec != *expected
            || recipe.physical != expected.physical
            || recipe.structure != expected.structure
            || recipe.semantic_identity != expected_identity
        {
            return Err(contract(
                "native recipe differs from original specification, identity or physical context",
            ));
        }
        let counts = count_native_membership(&recipe, allowance.limits, allowance.count_bytes)
            .map_err(pse_quantity::QuantityError::RecordMembership)?;
        // SAFETY: This private compiler receiver decoded these verified immutable
        // bytes once. Its wrapper's audited origin premise binds the original
        // scope/context. Index a borrow now, then move this same root to replay.
        let records = unsafe {
            RecordAuthority::from_authenticated_native_root(
                &recipe,
                scope,
                counts,
                allowance.index_bytes,
                allowance.query_bytes,
            )
        }
        .map_err(pse_quantity::QuantityError::RecordMembership)?;
        Ok(IndexedNative {
            recipe,
            records,
            payload,
        })
    }
    /// Receive a compiler-issued canonical product through its original qualification.
    /// # Safety
    /// The original canonical owner must authenticate eligible producer, namespace,
    /// dependencies, qualification and exact selected request, with its original
    /// physical/semantic owners and operation/cancellation lifetime. A caller hash,
    /// decoded DTO or validator success is insufficient. This test-only prerequisite
    /// does not establish full receiving/decode/adoption/reconstruction admission.
    unsafe fn receive_canonical(
        payload: Arc<[u8]>,
        mut scope: NativeRecipeScope,
        expected: &BodySpec,
        expected_identity: Option<SemanticBodyHash>,
        registry: &QuantityRegistry,
        cancelled: &Arc<AtomicBool>,
        allowance: TraversalAllowance,
    ) -> Result<ReceivedNative, MathError> {
        scope.source = NativeRecipeSource::CanonicalProduct;
        // SAFETY: the caller retains the original canonical premise above.
        unsafe {
            index_exact_recipe(
                payload,
                scope,
                expected,
                expected_identity,
                cancelled,
                allowance,
            )
        }?
        .reconstruct(registry, cancelled)
    }
    /// Receive the trusted parent's immutable admission on its private child channel.
    /// # Safety
    /// The original child owner must authenticate image/build, attempt, profile,
    /// deadline, request/execution and complete physical/factory context while the
    /// parent retains scientific authority and original settlement ownership. Public
    /// stdin, decoded DTOs or self-supplied hashes are insufficient. This test-only
    /// prerequisite does not establish the full original workspace/F03 envelope.
    unsafe fn receive_private(
        payload: Arc<[u8]>,
        mut scope: NativeRecipeScope,
        expected: &BodySpec,
        expected_identity: Option<SemanticBodyHash>,
        registry: &QuantityRegistry,
        cancelled: &Arc<AtomicBool>,
        allowance: TraversalAllowance,
    ) -> Result<ReceivedNative, MathError> {
        scope.source = NativeRecipeSource::AdmittedProcessMessage;
        // SAFETY: the caller retains the original private-process premise above.
        unsafe {
            index_exact_recipe(
                payload,
                scope,
                expected,
                expected_identity,
                cancelled,
                allowance,
            )
        }?
        .reconstruct(registry, cancelled)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use buffa::Message as _;
        use pse_contract::boundary_projection::ProjectionRecord;

        fn fixture() -> (Recipe, QuantityRegistry) {
            let owner = crate::typed_math::direct_tests::fixture(
                crate::authored_transfer_tests::context(),
                "package direct { fn doubled(x:Scalar)->Scalar=x+x; def Root { var x:Scalar; let out:Scalar=doubled(x)+doubled(x*x); } }",
                &["out"],
            ).unwrap();
            let body = owner.admit(&Arc::new(AtomicBool::new(false))).unwrap();
            let mut recipe = Recipe::decode(&body.portable_payload().unwrap()).unwrap();
            recipe.interpretation = NATIVE_RECIPE_INTERPRETATION.into();
            (recipe, (*owner.context.quantities).clone())
        }
        fn allowance(recipe: &Recipe) -> TraversalAllowance {
            let limits = NativeMembershipLimits {
                nodes: 100_000,
                key_bytes: 4 * 1024 * 1024,
                helper_bytes: 4096,
                depth: 512,
            };
            let count_bytes = limits.count_scratch_bytes().unwrap();
            let counts = count_native_membership(recipe, limits, count_bytes).unwrap();
            TraversalAllowance {
                limits,
                count_bytes,
                index_bytes: counts.authority_scratch_bytes().unwrap(),
                query_bytes: counts.query_scratch_bytes().unwrap(),
            }
        }
        fn payload(recipe: &Recipe) -> Arc<[u8]> {
            super::super::adapters::encode_scientific_recipe(recipe.clone())
                .unwrap()
                .encode_to_vec()
                .into()
        }
        fn scope(payload: &[u8]) -> NativeRecipeScope {
            NativeRecipeScope {
                source: NativeRecipeSource::CanonicalProduct,
                payload_digest: native_recipe_payload_hash(payload),
                qualification: ContentHash::from_bytes([17; 32]),
                request: ContentHash::from_bytes([18; 32]),
            }
        }
        #[test]
        fn scientific_recipe_native_receivers_decode_index_and_consume_same_actual_root() {
            let (recipe, registry) = fixture();
            let bytes = payload(&recipe);
            let cancel = Arc::new(AtomicBool::new(false));
            // SAFETY: isolated admitted compiler fixture exercises each origin
            // premise locally; actual runtime authentication gates remain open.
            let canonical = unsafe {
                receive_canonical(
                    bytes.clone(),
                    scope(&bytes),
                    &recipe.spec,
                    recipe.semantic_identity,
                    &registry,
                    &cancel,
                    allowance(&recipe),
                )
            }
            .unwrap();
            let private = unsafe {
                receive_private(
                    bytes.clone(),
                    scope(&bytes),
                    &recipe.spec,
                    recipe.semantic_identity,
                    &registry,
                    &cancel,
                    allowance(&recipe),
                )
            }
            .unwrap();
            for received in [canonical, private] {
                assert_eq!(received.body.spec(), &recipe.spec);
                assert_eq!(received.body.quantities, recipe.outputs);
                assert_eq!(received.payload.as_ref(), bytes.as_ref());
                assert!(!received.body.has_portable_payload());
            }
        }
        #[test]
        fn scientific_recipe_native_actual_complete_surviving_nodes_match_dom_membership() {
            let (recipe, _) = fixture();
            let bytes = payload(&recipe);
            let cancel = Arc::new(AtomicBool::new(false));
            // SAFETY: compiler-owned exact bytes/root in an isolated fixture.
            let indexed = unsafe {
                index_exact_recipe(
                    bytes.clone(),
                    scope(&bytes),
                    &recipe.spec,
                    recipe.semantic_identity,
                    &cancel,
                    allowance(&recipe),
                )
            }
            .unwrap();
            let dom = serde_json::to_value(&recipe).unwrap();
            let json = serde_json::to_vec(&recipe).unwrap();
            let digest = pse_ids::scientific_replay::admitted_recipe_payload_hash(&json);
            // SAFETY: independent legacy DOM oracle over the same fixture meaning.
            let oracle = unsafe {
                pse_ids::scientific_replay::QualifiedScientificRecipe::from_canonical_product(
                    json,
                    digest,
                    scope(&bytes).qualification,
                    scope(&bytes).request,
                )
            }
            .unwrap();
            let mut nodes = vec![&dom];
            let mut visited = 0;
            while let Some(node) = nodes.pop() {
                assert_eq!(
                    indexed.records.admits(node).unwrap(),
                    oracle.records().admits(node).unwrap()
                );
                assert!(indexed.records.admits(node).unwrap());
                visited += 1;
                match node {
                    serde_json::Value::Array(values) => nodes.extend(values),
                    serde_json::Value::Object(values) => nodes.extend(values.values()),
                    _ => {}
                }
            }
            assert!(visited > 100);
            assert!(indexed.records.admits(&indexed.recipe).unwrap());
            let foreign = serde_json::json!({"foreign_record_for_native_membership": 977331});
            assert!(!indexed.records.admits(&foreign).unwrap());
            assert!(!oracle.records().admits(&foreign).unwrap());
        }
        #[test]
        fn scientific_recipe_native_receiver_refuses_digest_interpretation_identity_context_and_cancel()
         {
            let (recipe, registry) = fixture();
            let bytes = payload(&recipe);
            let cancel = Arc::new(AtomicBool::new(false));
            let mut wrong_digest = scope(&bytes);
            wrong_digest.payload_digest = ContentHash::from_bytes([77; 32]);
            // SAFETY: isolated negative fixtures preserve caller origin; corrupt
            // content is deliberately offered to prove local receiver refusal.
            assert!(
                unsafe {
                    receive_canonical(
                        bytes.clone(),
                        wrong_digest,
                        &recipe.spec,
                        recipe.semantic_identity,
                        &registry,
                        &cancel,
                        allowance(&recipe),
                    )
                }
                .is_err()
            );
            let mut wrong = recipe.clone();
            wrong.interpretation = "pse.admitted-body.v3".into();
            let encoded = payload(&wrong);
            assert!(
                unsafe {
                    receive_private(
                        encoded.clone(),
                        scope(&encoded),
                        &recipe.spec,
                        recipe.semantic_identity,
                        &registry,
                        &cancel,
                        allowance(&recipe),
                    )
                }
                .is_err()
            );
            let mut context = recipe.spec.clone();
            context.physical = ContentHash::from_bytes([78; 32]);
            assert!(
                unsafe {
                    receive_canonical(
                        bytes.clone(),
                        scope(&bytes),
                        &context,
                        recipe.semantic_identity,
                        &registry,
                        &cancel,
                        allowance(&recipe),
                    )
                }
                .is_err()
            );
            assert!(
                unsafe {
                    receive_private(
                        bytes.clone(),
                        scope(&bytes),
                        &recipe.spec,
                        Some(SemanticBodyHash::from_bytes([79; 32])),
                        &registry,
                        &cancel,
                        allowance(&recipe),
                    )
                }
                .is_err()
            );
            cancel.store(true, Ordering::Relaxed);
            assert!(matches!(
                unsafe {
                    receive_canonical(
                        bytes.clone(),
                        scope(&bytes),
                        &recipe.spec,
                        recipe.semantic_identity,
                        &registry,
                        &cancel,
                        allowance(&recipe),
                    )
                },
                Err(MathError::Cancelled)
            ));
        }
        #[test]
        fn scientific_recipe_native_nested_operation_and_boundary_streams_keep_complete_ordered_drain()
         {
            let (recipe, registry) = fixture();
            let cancel = Arc::new(AtomicBool::new(false));
            let quantity = registry.neutral_dimensionless().unwrap();
            let contract = pse_quantity::resolved::ResolvedPhysicalContract::named(
                quantity,
                pse_quantity::IndexSet::new(),
                &registry,
            )
            .unwrap();
            let journey = || {
                for operation in [
                    pse_quantity::infer::OpRequest::Add,
                    pse_quantity::infer::OpRequest::Mul,
                ] {
                    pse_quantity::resolved::infer_operation(
                        &operation,
                        &[contract.clone(), contract.clone()],
                        None,
                        &registry,
                        &pse_quantity::infer::NoInvariantFacts,
                    )?;
                }
                for identity in [47, 48] {
                    pse_quantity::resolved::AdmittedOutputBoundary::declared(
                        pse_ids::SemanticId::from_bytes([identity; 16]),
                        contract.clone(),
                        quantity,
                        &registry,
                    )?;
                }
                Ok::<_, pse_quantity::QuantityError>(())
            };
            let (_, stream) = pse_quantity::resolved::receipts::capture(journey).unwrap();
            let (version, entries, boundaries) = stream.into_projection_fields();
            assert_eq!(entries.len(), 2);
            assert_eq!(boundaries.len(), 2);
            assert_ne!(entries[0], entries[1]);
            assert_ne!(boundaries[0], boundaries[1]);
            let mut missing = entries.clone();
            missing.pop();
            let mut extra = entries.clone();
            extra.push(entries[0].clone());
            let mut reordered = entries.clone();
            reordered.swap(0, 1);
            let mut missing_boundary = boundaries.clone();
            missing_boundary.pop();
            let mut extra_boundary = boundaries.clone();
            extra_boundary.push(boundaries[0].clone());
            let mut reordered_boundary = boundaries.clone();
            reordered_boundary.swap(0, 1);
            for (entries, boundaries, accepted) in [
                (entries.clone(), boundaries.clone(), true),
                (missing, boundaries.clone(), false),
                (extra, boundaries.clone(), false),
                (reordered, boundaries.clone(), false),
                (entries.clone(), missing_boundary, false),
                (entries.clone(), extra_boundary, false),
                (entries.clone(), reordered_boundary, false),
            ] {
                let mut changed = recipe.clone();
                changed.receipts =
                    pse_quantity::resolved::receipts::ReceiptStream::from_projection_fields((
                        version, entries, boundaries,
                    ))
                    .unwrap();
                let bytes = payload(&changed);
                // SAFETY: isolated component fixture places these actual captured
                // outcomes in the actual private Recipe and indexes its exact root.
                // This qualifies nested receipt mechanics, not a claim that the
                // scalar body's normal reconstruction consumed these operations.
                let indexed = unsafe {
                    index_exact_recipe(
                        bytes.clone(),
                        scope(&bytes),
                        &recipe.spec,
                        recipe.semantic_identity,
                        &cancel,
                        allowance(&changed),
                    )
                }
                .unwrap();
                let result = pse_quantity::resolved::receipts::replay(
                    &indexed.recipe.receipts,
                    &indexed.records,
                    journey,
                );
                assert_eq!(result.is_ok(), accepted);
            }
            let mut changed = recipe.clone();
            let (version, mut entries) = changed.proofs.into_projection_fields();
            entries.push(
                pse_math::typed::receipts::Proof::from_projection_fields((
                    ContentHash::from_bytes([83; 32]),
                    vec![],
                ))
                .unwrap(),
            );
            changed.proofs =
                pse_math::typed::receipts::ProofStream::from_projection_fields((version, entries))
                    .unwrap();
            let bytes = payload(&changed);
            assert!(
                unsafe {
                    receive_private(
                        bytes.clone(),
                        scope(&bytes),
                        &recipe.spec,
                        recipe.semantic_identity,
                        &registry,
                        &cancel,
                        allowance(&changed),
                    )
                }
                .is_err()
            );
        }
        #[test]
        fn scientific_recipe_native_physical_record_queries_propagate_typed_profile_and_budget_failures()
         {
            let (recipe, _) = fixture();
            let bytes = payload(&recipe);
            let cancel = Arc::new(AtomicBool::new(false));
            let budget = allowance(&recipe);
            // SAFETY: complete compiler-owned immutable isolated fixture.
            let indexed = unsafe {
                index_exact_recipe(
                    bytes.clone(),
                    scope(&bytes),
                    &recipe.spec,
                    recipe.semantic_identity,
                    &cancel,
                    budget,
                )
            }
            .unwrap();
            let profile = pse_quantity::resolved::receipts::replay(
                &indexed.recipe.receipts,
                &indexed.records,
                || pse_quantity::resolved::receipts::require_record(&i128::MAX),
            )
            .unwrap_err();
            assert!(matches!(
                profile,
                pse_quantity::QuantityError::RecordMembership(
                    pse_ids::IdError::ScientificMembershipProfile { .. }
                )
            ));
            let oversized = vec![0_u8; budget.limits.nodes + 1];
            let limit = pse_quantity::resolved::receipts::replay(
                &indexed.recipe.receipts,
                &indexed.records,
                || pse_quantity::resolved::receipts::require_record(&oversized),
            )
            .unwrap_err();
            assert!(matches!(
                limit,
                pse_quantity::QuantityError::RecordMembership(
                    pse_ids::IdError::ScientificMembershipLimit { .. }
                )
            ));
            let foreign = pse_modeling::portable::ValueRecord::capture(
                &pse_modeling::specialize::Value::Tuple(vec![
                    pse_modeling::specialize::Value::Integer(0); budget.limits.nodes + 1
                ]),
            );
            let declaration = pse_ids::SemanticId::from_bytes([92; 16]);
            let restored = pse_quantity::resolved::receipts::replay(
                &indexed.recipe.receipts,
                &indexed.records,
                || {
                    foreign
                        .restore(declaration.into())
                        .map_err(crate::typed_math::portable::reconstruction_model_error)
                },
            )
            .unwrap_err();
            let MathError::Typed { cause, .. } = restored else {
                panic!("retain modeling record query source")
            };
            use pse_diagnostics::TypedDiagnostic as _;
            assert_eq!(
                cause.diagnostic_code(),
                Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit)
            );
            let diagnostic = cause
                .as_ref()
                .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Modeling);
            assert!(diagnostic.sources.contains(&declaration));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::Recipe;
    use super::{adapters, pb::pse::scientific_recipe::v4 as wire, view_adapters};
    use buffa::Message as _;
    use pse_contract::boundary_projection::{AdapterError, AdapterFailure};
    use pse_contract_proto::support::BoundaryProjectionError;

    fn actual_recipe() -> (Recipe, Vec<u8>) {
        let owner = crate::typed_math::direct_tests::fixture(
            crate::authored_transfer_tests::context(),
            "package direct { fn doubled(x:Scalar)->Scalar=x+x; def Root { var x:Scalar; let out:Scalar=doubled(x); } }",
            &["out"],
        )
        .unwrap();
        let body = owner
            .admit(&std::sync::Arc::new(std::sync::atomic::AtomicBool::new(
                false,
            )))
            .unwrap();
        let payload = body.portable_payload().unwrap();
        (Recipe::decode(&payload).unwrap(), payload)
    }

    fn both_refusals(projected: wire::ScientificRecipe) -> [BoundaryProjectionError; 2] {
        let bytes = projected.encode_to_vec();
        let owned = wire::ScientificRecipe::decode_from_slice(&bytes).unwrap();
        let view = buffa::DecodeOptions::new()
            .decode_view::<wire::__buffa::view::ScientificRecipeView<'_>>(&bytes)
            .unwrap();
        [
            adapters::decode_scientific_recipe(owned).unwrap_err(),
            view_adapters::decode_scientific_recipe_view(&view).unwrap_err(),
        ]
    }

    #[test]
    fn scientific_recipe_owned_and_borrowed_binary_adoption_preserve_actual_native_meaning() {
        let (recipe, payload) = actual_recipe();
        let projected = adapters::encode_scientific_recipe(recipe.clone()).unwrap();
        let bytes = projected.encode_to_vec();
        let decoded = wire::ScientificRecipe::decode_from_slice(&bytes).unwrap();
        let owned = adapters::decode_scientific_recipe(decoded).unwrap();
        let view = buffa::DecodeOptions::new()
            .decode_view::<wire::__buffa::view::ScientificRecipeView<'_>>(&bytes)
            .unwrap();
        let borrowed = view_adapters::decode_scientific_recipe_view(&view).unwrap();
        for adopted in [owned, borrowed] {
            assert_eq!(adopted, recipe);
            assert_eq!(adopted.encode().unwrap().as_ref(), payload.as_slice());
            assert_eq!(adopted.interpretation, "pse.admitted-body.v3");
            // Passive binary adoption cannot grant original reconstruction authority.
            assert!(adopted.syntax.restore(adopted.occurrences).is_err());
        }
        assert!(
            super::source_policy::INTERPRETATIONS
                .contains(&("pse.scientific_recipe.v4.ScientificRecipe", 1))
        );
    }

    #[test]
    fn scientific_recipe_owned_and_borrowed_adoption_refuse_root_and_nested_unknown_meaning() {
        let (recipe, _) = actual_recipe();
        let projected = adapters::encode_scientific_recipe(recipe).unwrap();
        for nested in [false, true] {
            let mut changed = projected.clone();
            let fields = if nested {
                &mut changed
                    .syntax
                    .as_option_mut()
                    .unwrap()
                    .__buffa_unknown_fields
            } else {
                &mut changed.__buffa_unknown_fields
            };
            fields.push(buffa::UnknownField {
                number: 99,
                data: buffa::UnknownFieldData::Varint(7),
            });
            let bytes = changed.encode_to_vec();
            let decoded = wire::ScientificRecipe::decode_from_slice(&bytes).unwrap();
            assert_eq!(decoded.encode_to_vec(), bytes);
            let expected_path = if nested {
                "ScientificRecipe.syntax"
            } else {
                "ScientificRecipe"
            };
            for error in both_refusals(decoded) {
                assert!(matches!(
                    error,
                    BoundaryProjectionError::Adoption(AdapterError::Refused {
                        path,
                        kind: AdapterFailure::UnknownField,
                        ..
                    }) if path == expected_path
                ));
            }
        }
    }

    #[test]
    fn scientific_recipe_owned_and_borrowed_adoption_refuse_malformed_nested_shape_and_digest() {
        let (recipe, _) = actual_recipe();
        let projected = adapters::encode_scientific_recipe(recipe).unwrap();
        let mut missing = projected.clone();
        missing.syntax.as_option_mut().unwrap().syntax = Default::default();
        for error in both_refusals(missing) {
            assert!(matches!(
                error,
                BoundaryProjectionError::Adoption(AdapterError::Refused {
                    path,
                    kind: AdapterFailure::InvalidValue,
                    ..
                }) if path == "ScientificRecipe.syntax.syntax"
            ));
        }
        let mut invalid = projected;
        invalid.physical.pop().unwrap();
        for error in both_refusals(invalid) {
            let BoundaryProjectionError::WireValidation(error) = error else {
                panic!("invalid digest width must retain the generated validator refusal")
            };
            assert!(error.violations.iter().any(|violation| {
                violation.rule_id == "bytes.len"
                    && violation.field.elements[0].field_name.as_deref() == Some("physical")
            }));
        }
    }
}
