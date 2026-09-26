// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Independently compiled semantic closure and execution encoding expectations.

use crate::{Registry, SchemaError};
use proc_macro2::TokenStream;
use quote::quote;

pub(super) fn render(reg: &Registry) -> Result<TokenStream, SchemaError> {
    let entries = reg
        .relations()
        .iter()
        .map(|spec| {
            let expected = crate::resolved_contract::ExpectedContract::capture(reg, spec)?;
            let id = expected.relation.as_bytes();
            let version = expected.version;
            let semantic_version = expected.semantic_version;
            let semantics = expected.semantics.as_bytes();
            let encoding = expected.encoding.as_bytes();
            Ok(quote! {
                pse_schema::resolved_contract::ExpectedContract {
                    relation: pse_ids::SemanticId::from_bytes([#(#id),*]),
                    version: #version,
                    semantic_version: #semantic_version,
                    semantics: pse_ids::ContentHash::from_bytes([#(#semantics),*]),
                    encoding: pse_ids::ContentHash::from_bytes([#(#encoding),*]),
                }
            })
        })
        .collect::<Result<Vec<_>, SchemaError>>()?;
    Ok(quote! {
        //! Compact expectations generated independently of the receiving registry.
        static EXPECTED: &[pse_schema::resolved_contract::ExpectedContract] = &[#(#entries),*];
        pub(crate) fn expected() -> &'static [pse_schema::resolved_contract::ExpectedContract] {
            EXPECTED
        }
    })
}
