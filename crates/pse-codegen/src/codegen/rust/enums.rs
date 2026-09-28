// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed string enumerations, retaining the authoritative member spellings.
//!
//! A vocabulary whose declaration names its hand-written source type
//! ([`crate::model::EnumSpec::source`]) is re-exported rather than emitted a second time,
//! so it has one Rust type (ADR-0115 Outcome 3). The source type provides `ALL`, `as_str`
//! and `parse`; the generator adds only the value traits the generated rows need.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{Registry, SchemaError};

pub(super) fn render(reg: &Registry) -> Result<TokenStream, SchemaError> {
    let enums = reg
        .enums()
        .iter()
        .map(|spec| match spec.source {
            Some(source) => sourced(spec.name, source),
            None => Ok(declared(spec)),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(quote!(#(#enums)*))
}

/// A re-export of the vocabulary's source type, with the generated value traits.
fn sourced(name: &str, source: &str) -> Result<TokenStream, SchemaError> {
    let path = syn::parse_str::<syn::Path>(source)
        .map_err(|error| super::error(format!("enum {name} source `{source}`: {error}")))?;
    let name = format_ident!("{}", super::types::pascal(name));
    Ok(quote! {
        /// A vocabulary owned by its source type; the registry declares its members.
        pub type #name = #path;
        impl crate::SemanticEq for #name { fn semantic_eq(&self, other: &Self) -> bool { self == other } }
        impl crate::HeapUsage for #name { fn heap_bytes(&self) -> usize { 0 } }
        impl crate::SemanticFrame for #name { fn frame(&self, hash: &mut pse_ids::FramedHasher) { hash.str(self.as_str()); } }
        impl crate::columnar::ArrowValue for #name {
            fn append(&self, output: &mut dyn arrow_array::builder::ArrayBuilder) -> Result<(), crate::RelationError> {
                crate::columnar::append_string(output, Some(self.as_str()))
            }
            fn append_null(output: &mut dyn arrow_array::builder::ArrayBuilder) -> Result<(), crate::RelationError> {
                crate::columnar::append_string(output, None)
            }
            fn read(input: &dyn arrow_array::Array, index: usize) -> Result<Self, crate::RelationError> {
                let value = crate::columnar::read_string(input, index)?;
                Self::parse(value).ok_or_else(|| crate::RelationError::EnumMember {
                    field: stringify!(#name).to_owned(), enumeration: stringify!(#name).to_owned(), value: value.to_owned(),
                })
            }
        }
    })
}

/// A registry-owned enumeration, emitted from its declaration.
fn declared(spec: &crate::model::EnumSpec) -> TokenStream {
    let name = format_ident!("{}", super::types::pascal(spec.name));
    let variants = spec
        .members
        .iter()
        .map(|member| format_ident!("{}", super::types::pascal(member.name)))
        .collect::<Vec<_>>();
    let members = spec
        .members
        .iter()
        .map(|member| member.name)
        .collect::<Vec<_>>();
    let docs = spec
        .members
        .iter()
        .map(|member| member.doc)
        .collect::<Vec<_>>();
    let idaes = spec
        .members
        .iter()
        .map(|member| {
            if let Some(name) = member.idaes_name {
                quote!(Some(#name))
            } else {
                quote!(None)
            }
        })
        .collect::<Vec<_>>();
    let ordinals = (0..variants.len()).collect::<Vec<_>>();
    let length = variants.len();
    quote! {
        /// A string enumeration projected from the registry.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
        #[allow(clippy::enum_variant_names, reason = "closed enum spellings preserve registry and sanctioned parity names")]
        pub enum #name { #(#[doc = #docs] #[serde(rename = #members)] #variants,)* }
        impl crate::SemanticEq for #name { fn semantic_eq(&self, other: &Self) -> bool { self == other } }
        impl #name {
            /// All members in declaration order; the ordinal is presentation only.
            pub const ALL: [Self; #length] = [#(Self::#variants),*];
            /// The declared member spelling.
            pub const fn as_str(self) -> &'static str { match self { #(Self::#variants => #members,)* } }
            /// The presentation ordinal, never a semantic identity.
            pub const fn ordinal(self) -> usize { match self { #(Self::#variants => #ordinals,)* } }
            /// The sanctioned IDAES member name, where applicable.
            #[allow(clippy::match_same_arms, clippy::unnecessary_wraps, reason = "uniform optional parity-name projection follows one member declaration per arm")]
            pub const fn idaes_name(self) -> Option<&'static str> { match self { #(Self::#variants => #idaes,)* } }
        }
        /// A boundary document states this vocabulary as its registry spellings, which are
        /// its serde spellings (ADR-0116 Outcome 7).
        impl schemars::JsonSchema for #name {
            fn schema_name() -> std::borrow::Cow<'static, str> { std::borrow::Cow::Borrowed(stringify!(#name)) }
            fn schema_id() -> std::borrow::Cow<'static, str> { std::borrow::Cow::Borrowed(concat!("pse_model::", stringify!(#name))) }
            fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
                schemars::json_schema!({ "type": "string", "enum": [#(#members),*] })
            }
        }
        impl core::str::FromStr for #name {
            type Err = crate::RelationError;
            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value { #(#members => Ok(Self::#variants),)*
                    _ => Err(crate::RelationError::EnumMember { field: stringify!(#name).to_owned(), enumeration: stringify!(#name).to_owned(), value: value.to_owned() })
                }
            }
        }
        impl crate::columnar::ArrowValue for #name {
            fn append(&self, output: &mut dyn arrow_array::builder::ArrayBuilder) -> Result<(), crate::RelationError> {
                crate::columnar::append_string(output, Some(self.as_str()))
            }
            fn append_null(output: &mut dyn arrow_array::builder::ArrayBuilder) -> Result<(), crate::RelationError> {
                crate::columnar::append_string(output, None)
            }
            fn read(input: &dyn arrow_array::Array, index: usize) -> Result<Self, crate::RelationError> {
                crate::columnar::read_string(input, index)?.parse().map_err(Into::into)
            }
        }
    }
}
