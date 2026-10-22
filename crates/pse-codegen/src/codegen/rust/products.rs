// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed scientific product membership projected from the registry profiles.
use crate::{Registry, SchemaError};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(super) fn render(reg: &Registry) -> Result<TokenStream, SchemaError> {
    let Some(kinds) = reg.enum_spec("ScientificArtifactKind") else {
        return Ok(TokenStream::new());
    };
    let families = reg
        .enum_spec("ScientificArtifactFamily")
        .ok_or_else(|| super::error("scientific artifact families are absent".into()))?;
    let mut declarations = Vec::new();
    let mut arms = Vec::new();
    for kind in &kinds.members {
        let variant = super::types::ident(&super::types::pascal(kind.name));
        for family in &families.members {
            let profile_name = format!("{}-{}", kind.name, family.name);
            let Some(profile) = reg.artifact_profile(&profile_name) else {
                continue;
            };
            let family_variant = super::types::ident(&super::types::pascal(family.name));
            let constant = format_ident!(
                "REQUIRED_{}",
                profile_name.replace('-', "_").to_ascii_uppercase()
            );
            let relations = profile.iter().map(|id| {
                let spec = reg.relation_by_id(*id)
                    .ok_or_else(|| super::error(format!("scientific profile relation {id} is missing")))?;
                let bytes = id.as_bytes();
                let qualified = spec.key.qualified_name();
                Ok(quote!(ScientificRelation { id: pse_ids::SemanticId::from_bytes([#(#bytes),*]), name: #qualified }))
            }).collect::<Result<Vec<_>, SchemaError>>()?;
            let count = relations.len();
            declarations
                .push(quote!(const #constant: [ScientificRelation; #count] = [#(#relations),*];));
            arms.push(quote!((Self::#variant, super::enums::ScientificArtifactFamily::#family_variant) => Some(&#constant)));
        }
    }
    Ok(quote! {
        /// Qualified identity of one required scientific product member.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct ScientificRelation {
            /// Exact registry relation identity.
            pub id: pse_ids::SemanticId,
            /// Qualified name used at external relation boundaries.
            pub name: &'static str,
        }
        #(#declarations)*
        impl super::enums::ScientificArtifactKind {
            /// Complete required membership, including selected empty relations.
            pub const fn required_relations(self, family: super::enums::ScientificArtifactFamily) -> Option<&'static [ScientificRelation]> {
                match (self, family) { #(#arms,)* _ => None }
            }
        }
    })
}
