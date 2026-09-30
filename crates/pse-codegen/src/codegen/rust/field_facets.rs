// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Data-only projection of schema-owned facet purposes into the columnar dependency.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(super) fn render() -> TokenStream {
    let mut constants = Vec::new();
    let mut rows = Vec::new();
    for facet in crate::model::field_facets::FACETS {
        let constant = format_ident!("{}", facet.constant);
        let key = facet.key;
        let execution = facet.execution;
        let value = facet.value;
        let storage_root = facet.storage_root;
        let type_identity = facet.type_identity;
        let admission = match facet.admission {
            crate::model::field_facets::Admission::Ignore => quote!(Admission::Ignore),
            crate::model::field_facets::Admission::Exact => quote!(Admission::Exact),
            crate::model::field_facets::Admission::Preserve => quote!(Admission::Preserve),
        };
        constants.push(quote! {
            #[doc = concat!("Declared field facet `", #key, "`.")]
            pub const #constant: &str = #key;
        });
        rows.push(quote! {
            Facet { key: #constant, execution: #execution, value: #value,
                storage_root: #storage_root, type_identity: #type_identity,
                admission: #admission }
        });
    }
    quote! {
        //! Schema-declared field purposes. No registry or upper-layer dependency.
        /// Directional target admission policy for one metadata spelling.
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum Admission {
            /// Occurrence usage and presentation may differ.
            Ignore,
            /// Both sides must declare the same meaning.
            Exact,
            /// New checked meaning may be attached; existing meaning must match.
            Preserve,
        }
        /// Data-only facet projection.
        #[derive(Clone, Copy, Debug)]
        pub struct Facet {
            /// Metadata spelling.
            pub key: &'static str,
            /// Retained in execution identity.
            pub execution: bool,
            /// Retained in value identity.
            pub value: bool,
            /// Retained at the root of logical/storage type.
            pub storage_root: bool,
            /// Retained in logical type identity.
            pub type_identity: bool,
            /// Directional target admission.
            pub admission: Admission,
        }
        #(#constants)*
        /// Schema-owned policies. Unlisted keys remain significant.
        pub const FACETS: &[Facet] = &[#(#rows),*];
    }
}
