// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Binary `COPY ... FROM STDIN` of every store table (ADR-0114 Outcome 26): the
//! statement with the registry's column list, and a `WHERE false` probe whose result
//! columns carry the types the server expects in the binary stream (a domain column
//! arrives as its base type, an ENUM column as its ENUM type).

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::Registry;

use super::names::{qualified, quote as quoted};

pub(super) fn render(reg: &Registry) -> TokenStream {
    let tables = pse_schema::store::relations(reg)
        .into_iter()
        .map(|(table, spec)| {
            let name = format_ident!("{}", table.to_ascii_uppercase());
            let columns: Vec<&str> = spec.columns.iter().map(|column| column.name()).collect();
            let list = columns
                .iter()
                .map(|column| quoted(column))
                .collect::<Vec<_>>()
                .join(", ");
            let relation = qualified(&quoted(table));
            let statement = format!("COPY {relation} ({list}) FROM STDIN (FORMAT binary)");
            let probe = format!("SELECT {list} FROM {relation} WHERE false");
            let doc = format!(" The binary copy into `{}` ({}).", table, spec.qualified_name());
            quote! {
                #[doc = #doc]
                pub const #name: CopyIn = CopyIn {
                    table: #table,
                    statement: #statement,
                    probe: #probe,
                    columns: &[#(#columns),*],
                };
            }
        });
    quote! {
        //! Binary `COPY` of every store table, generated from the registry
        //! (ADR-0114 Outcome 26).
        /// A binary copy into one store table.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct CopyIn {
            /// The table, in the `pse_ops` schema.
            pub table: &'static str,
            /// `COPY ... FROM STDIN (FORMAT binary)` with the registry's column list.
            pub statement: &'static str,
            /// A statement returning no row whose columns have the copied types.
            pub probe: &'static str,
            /// The copied columns, in the order every row writes them.
            pub columns: &'static [&'static str],
        }
        #(#tables)*
    }
}
