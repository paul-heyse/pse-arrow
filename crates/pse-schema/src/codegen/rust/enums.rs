// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed closed dictionaries, retaining the authoritative member spellings.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::Registry;

pub(super) fn render(reg: &Registry) -> TokenStream {
    let enums = reg.enums().iter().map(|spec| {
        let name = format_ident!("{}", super::types::pascal(spec.name));
        let variants = spec.members.iter().map(|member| format_ident!("{}", super::types::pascal(member.name))).collect::<Vec<_>>();
        let members = spec.members.iter().map(|member| member.name).collect::<Vec<_>>();
        let docs = spec.members.iter().map(|member| member.doc).collect::<Vec<_>>();
        let idaes = spec.members.iter().map(|member| if let Some(name) = member.idaes_name { quote!(Some(#name)) } else { quote!(None) }).collect::<Vec<_>>();
        let ordinals = (0..variants.len()).collect::<Vec<_>>();
        let length = variants.len();
        quote! {
            /// A closed dictionary projected from the registry.
            #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
            #[allow(clippy::enum_variant_names, reason = "closed enum spellings preserve registry and sanctioned parity names")]
            pub enum #name { #(#[doc = #docs] #[serde(rename = #members)] #variants,)* }
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
            impl core::str::FromStr for #name {
                type Err = crate::RelationError;
                fn from_str(value: &str) -> Result<Self, Self::Err> {
                    match value { #(#members => Ok(Self::#variants),)*
                        _ => Err(crate::RelationError::EnumMember { field: stringify!(#name).to_owned(), enumeration: stringify!(#name).to_owned(), value: value.to_owned() })
                    }
                }
            }
            impl crate::typed::CellCodec for #name {
                fn into_cell(self) -> pse_schema::model::Cell { pse_schema::model::Cell::Enum(self.as_str()) }
                fn from_cell(cell: pse_schema::model::Cell) -> Result<Self, crate::RelationError> {
                    match cell { pse_schema::model::Cell::Enum(value) => value.parse(), _ => Err(crate::typed::mismatch(stringify!(#name))) }
                }
            }
        }
    });
    quote!(#(#enums)*)
}
