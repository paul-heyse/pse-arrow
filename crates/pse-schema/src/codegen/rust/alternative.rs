// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed selection and construction projected from the native arm declaration.
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::types::{ident, pascal};
use crate::{
    SchemaError,
    model::{FieldContract, TaggedAlternative},
};

pub(super) fn accessors(field: &FieldContract, stem: &str) -> Result<TokenStream, SchemaError> {
    let Some(alternative) = TaggedAlternative::from_field(field.field())? else {
        return Ok(TokenStream::new());
    };
    let name = format_ident!("{stem}");
    let selected = format_ident!("{stem}Selected");
    let discriminator = ident(&alternative.discriminator);
    let children = field.children();
    let tag_field = children
        .iter()
        .find(|field| field.name() == alternative.discriminator)
        .ok_or_else(|| super::error("alternative discriminator absent".into()))?;
    let payloads = alternative.payloads();
    let names = payloads.iter().map(|name| ident(name)).collect::<Vec<_>>();
    let lifetime = (!payloads.is_empty()).then(|| quote!(<'a>));
    let borrowed = (!payloads.is_empty()).then(|| quote!(<'_>));
    let mut variants = Vec::new();
    let mut selections = Vec::new();
    let mut constructors = Vec::new();
    for (tag, arm) in &alternative.arms {
        let variant = format_ident!("{}", pascal(tag));
        let constructor = format_ident!("from_{tag}");
        let patterns = payloads
            .iter()
            .map(|name| {
                if Some(*name) == arm.as_deref() {
                    quote!(Some(value))
                } else {
                    quote!(None)
                }
            })
            .collect::<Vec<_>>();
        let fields = payloads.iter().map(|name| {
            let field = ident(name);
            if Some(*name) == arm.as_deref() {
                quote!(#field: Some(value))
            } else {
                quote!(#field: None)
            }
        });
        let tag_value = if let Some(enumeration) = tag_field.enum_name() {
            let enumeration = format_ident!("{}", pascal(enumeration));
            quote!(crate::generated::enums::#enumeration::#variant)
        } else {
            quote!(#tag.to_owned())
        };
        let argument = if let Some(arm) = arm {
            let arm_field = children
                .iter()
                .find(|field| field.name() == arm)
                .ok_or_else(|| super::error("alternative arm absent".into()))?;
            let payload = super::types::logical(
                arm_field,
                &format!("{stem}{}", pascal(arm)),
                &mut Vec::new(),
            )?;
            variants.push(quote!(#[doc = #tag] #variant(&'a #payload)));
            selections.push(quote!((#tag, #(#patterns,)*) => Ok(#selected::#variant(value))));
            quote!(value: #payload)
        } else {
            variants.push(quote!(#[doc = #tag] #variant));
            selections.push(quote!((#tag, #(#patterns,)*) => Ok(#selected::#variant)));
            quote!()
        };
        constructors.push(quote! {
            #[doc = concat!("Construct the ", #tag, " arm with every other arm absent.")]
            pub fn #constructor(#argument) -> Self {
                Self { #discriminator: #tag_value, #(#fields,)* }
            }
        });
    }
    Ok(quote! {
        /// The declared selected payload, borrowed without a second row representation.
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum #selected #lifetime { #(#variants,)* }
        impl #name {
            #(#constructors)*
            /// Select exactly the declared payload.
            /// # Errors
            /// Unknown tag, missing selected arm or any overlapping arm.
            pub fn selected(&self) -> Result<#selected #borrowed, crate::RelationError> {
                match (self.#discriminator.as_str(), #(self.#names.as_ref(),)*) {
                    #(#selections,)*
                    _ => Err(crate::typed::mismatch("tagged value requires exactly its selected arm")),
                }
            }
        }
    })
}
