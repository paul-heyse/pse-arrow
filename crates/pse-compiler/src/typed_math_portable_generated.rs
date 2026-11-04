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
