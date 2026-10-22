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
    let entries = kinds.members.iter().map(|kind| {
        let name = &kind.name;
        let variant = super::types::ident(&super::types::pascal(name));
        let constant = format_ident!("REQUIRED_{}", name.replace('-', "_").to_ascii_uppercase());
        let profile = reg.artifact_profile(name)
            .ok_or_else(|| super::error(format!("scientific artifact {name} has no profile")))?;
        let relations = profile.iter().map(|id| {
            let spec = reg.relation_by_id(*id)
                .ok_or_else(|| super::error(format!("scientific profile relation {id} is missing")))?;
            let bytes = id.as_bytes();
            let qualified = spec.key.qualified_name();
            Ok(quote!(ScientificRelation { id: pse_ids::SemanticId::from_bytes([#(#bytes),*]), name: #qualified }))
        }).collect::<Result<Vec<_>, SchemaError>>()?;
        let count = relations.len();
        Ok((quote!(const #constant: [ScientificRelation; #count] = [#(#relations),*];), quote!(Self::#variant => &#constant)))
    }).collect::<Result<Vec<_>, SchemaError>>()?;
    let (declarations, arms): (Vec<_>, Vec<_>) = entries.into_iter().unzip();
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
            pub const fn required_relations(self) -> &'static [ScientificRelation] {
                match self { #(#arms,)* }
            }
        }
    })
}
