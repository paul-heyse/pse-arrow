// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! A relation's generated identity, concrete row, view and builder.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::model::RelationSpec;
use crate::{Registry, SchemaError};

use super::types::{ident, logical, optional, pascal, structure};

pub(super) fn render(reg: &Registry, spec: &RelationSpec) -> Result<TokenStream, SchemaError> {
    let stem = format!(
        "{}{}",
        pascal(spec.key.namespace.as_str()),
        pascal(spec.key.name)
    );
    let row = format_ident!("{stem}Row");
    let view = format_ident!("{stem}View");
    let builder = format_ident!("{stem}Builder");
    let mut declarations = Vec::new();
    let fields = spec
        .columns
        .iter()
        .map(|column| {
            let ty = logical(
                &column.value_type(),
                &format!("{stem}Field{}", pascal(column.name())),
                &mut declarations,
            )?;
            Ok((
                column.name().to_owned(),
                optional(ty, column.nullable()),
                column.doc().to_owned(),
            ))
        })
        .collect::<Result<Vec<_>, SchemaError>>()?;
    declarations.push(structure(&format!("{stem}Row"), &fields));
    let ids = spec.id.as_bytes();
    let fingerprint = spec.fingerprint.as_bytes();
    let name = spec.key.name;
    let namespace = format_ident!("{}", pascal(spec.key.namespace.as_str()));
    let version = spec.key.version;
    let names = fields
        .iter()
        .map(|(name, _, _)| ident(name))
        .collect::<Vec<_>>();
    let positions = (0..names.len()).collect::<Vec<_>>();
    let support = support(spec, &row, &view, &builder)?;
    let allocation = super::columnar::allocation(spec);
    let minimum_row = spec
        .columns
        .iter()
        .map(|column| super::columnar::minimum_allocation(&column.value_type().data_type()))
        .sum::<usize>();
    let builder_allocation = minimum_row
        .checked_mul(128)
        .ok_or_else(|| super::error("generated builder allocation extent overflow".to_owned()))?;
    let digits = builder_allocation
        .to_string()
        .chars()
        .rev()
        .collect::<Vec<_>>();
    let grouped = digits
        .chunks(3)
        .rev()
        .map(|group| group.iter().rev().collect::<String>())
        .collect::<Vec<_>>()
        .join("_");
    let builder_allocation =
        syn::LitInt::new(&format!("{grouped}_usize"), proc_macro2::Span::call_site());
    let admission = admission(reg, spec);
    Ok(quote! {
        /// The declared relation identity.
        pub const RELATION_ID: pse_ids::SemanticId = pse_ids::SemanticId::from_bytes([#(#ids),*]);
        /// The declared name within its namespace.
        pub const NAME: &str = #name;
        /// The declared namespace.
        pub const NAMESPACE: pse_schema::model::Namespace = pse_schema::model::Namespace::#namespace;
        /// The schema generation.
        pub const VERSION: u32 = #version;
        /// Generated interchange fingerprint, not proof of semantic equivalence or row validity.
        pub const FINGERPRINT: pse_ids::ContentHash = pse_ids::ContentHash::from_bytes([#(#fingerprint),*]);
        #(#declarations)*
        /// The concrete generated relation row.
        pub type Row = #row;
        /// The concrete generated relation view.
        pub type View<'a> = #view<'a>;
        /// The concrete generated relation builder.
        pub type Builder = #builder;
        #admission
        /// The schema built from the authoritative declaration.
        /// # Errors
        /// An unavailable registry or incompatible generated contract.
        pub fn schema() -> Result<crate::SchemaRef, crate::RelationError> {
            let reg = pse_schema::registry()?;
            Ok(pse_schema::arrow::relation_schema_ref(reg, spec(reg)?)?)
        }
        /// Checks schema, recursive extension contracts and visible values.
        /// # Errors
        /// All independently actionable violations.
        pub fn validate(batch: &crate::RecordBatch) -> Result<(), Vec<crate::RelationError>> {
            let reg = pse_schema::registry().map_err(|error| vec![error.into()])?;
            crate::validate::validate_batch(reg, spec(reg).map_err(|error| vec![error])?, batch)
        }
        impl crate::columnar::RelationRow for #row {
            type Builder = #builder;
            fn append_columns(&self, columns: &mut [Box<dyn arrow_array::builder::ArrayBuilder>]) -> Result<(), crate::RelationError> {
                #(crate::columnar::ArrowValue::append(&self.#names, columns[#positions].as_mut())?;)*
                Ok(())
            }
            fn relation(registry: &pse_schema::Registry) -> Result<&pse_schema::model::RelationSpec, crate::RelationError> { spec(registry) }
            fn builder(registry: &pse_schema::Registry, capacity: usize) -> Result<Self::Builder, crate::RelationError> { #builder::with_registry(registry, capacity) }
            fn push(builder: &mut Self::Builder, row: Self) -> Result<(), crate::RelationError> { builder.push(row) }
            fn finish(builder: Self::Builder) -> Result<crate::columnar::FieldCheckedBatch, crate::RelationError> { builder.finish() }
            fn rows(batch: &crate::columnar::FieldCheckedBatch) -> Result<Vec<Self>, crate::RelationError> { #view::from_checked(batch)?.rows() }
            fn rows_at(batch: &crate::columnar::FieldCheckedBatch, positions: &[usize]) -> Result<Vec<Self>, crate::RelationError> {
                let view = #view::from_checked(batch)?;
                positions.iter().map(|&position| view.row(position)).collect()
            }
            fn builder_allocation_size() -> usize { #builder_allocation + size_of::<Self::Builder>() }
            fn minimum_row_allocation_size() -> usize { #minimum_row }
            fn allocation_size(&self) -> Result<usize, crate::RelationError> { #allocation }
        }
        #support
    })
}

fn admission(reg: &Registry, spec: &RelationSpec) -> TokenStream {
    let qualified = spec.qualified_name();
    // One representative generated test exercises the exact runtime acceptance branch.
    // The semantic mutation matrix belongs to the shared schema helper's own tests.
    let test = if reg
        .relations()
        .first()
        .is_some_and(|first| first.id == spec.id)
    {
        quote! {
            #[cfg(test)]
            mod consolidation_unit {
                #[test]
                fn equal_fingerprint_cannot_admit_a_changed_declaration() -> Result<(), crate::RelationError> {
                    let registry = pse_schema::registry()?;
                    let original = super::spec(registry)?;
                    let mut altered = original.clone();
                    altered.columns[0] = altered.columns[0].clone().with_nullable(!altered.columns[0].nullable());
                    assert_eq!(altered.fingerprint, super::FINGERPRINT);
                    assert!(super::check_declaration(registry, &altered).is_err());
                    Ok(())
                }
            }
        }
    } else {
        TokenStream::new()
    };
    quote! {
        /// Resolves this exact generated contract in a runtime registry.
        /// # Errors
        /// A missing or incompatible declaration.
        pub fn spec(reg: &pse_schema::Registry) -> Result<&pse_schema::model::RelationSpec, crate::RelationError> {
            let spec = reg.relation_by_id(RELATION_ID).ok_or_else(|| crate::RelationError::UnknownRegistry { relation: #qualified.to_owned() })?;
            check_declaration(reg, spec)?;
            Ok(spec)
        }
        fn check_declaration(reg: &pse_schema::Registry, spec: &pse_schema::model::RelationSpec) -> Result<(), crate::RelationError> {
            reg.contract(spec)?.require_generated(crate::generated::contracts::expected())?;
            Ok(())
        }
        #test
    }
}

fn support(
    spec: &RelationSpec,
    row: &proc_macro2::Ident,
    view: &proc_macro2::Ident,
    builder: &proc_macro2::Ident,
) -> Result<TokenStream, SchemaError> {
    let names = spec
        .columns
        .iter()
        .map(crate::model::FieldContract::name)
        .collect::<Vec<_>>();
    let fields = names.iter().map(|name| ident(name)).collect::<Vec<_>>();
    let positions = (0..fields.len()).collect::<Vec<_>>();
    let count = fields.len();
    let column_constants = names
        .iter()
        .map(|name| format_ident!("{}", name.to_ascii_uppercase()))
        .collect::<Vec<_>>();
    let array_types = spec
        .columns
        .iter()
        .map(|column| super::columnar::array_type(&column.value_type()))
        .collect::<Result<Vec<_>, _>>()?;
    let columns = names
        .iter()
        .map(|name| format_ident!("{name}_column"))
        .collect::<Vec<_>>();
    let field_refs = names
        .iter()
        .map(|name| format_ident!("{name}_field"))
        .collect::<Vec<_>>();
    let accessor_attributes = names
        .iter()
        .copied()
        .map(accessor_attributes)
        .collect::<Vec<_>>();
    Ok(quote! {
        /// The complete declared relation key.
        pub const RELATION_KEY: pse_schema::model::RelationKey = pse_schema::model::RelationKey {
            namespace: NAMESPACE, name: NAME, version: VERSION,
        };
        /// Stable field references projected from the declared column order.
        pub const COLUMNS: [crate::columnar::ColumnReference; #count] = [
            #(crate::columnar::ColumnReference { relation_id: RELATION_ID, name: #names, position: #positions },)*
        ];
        /// Named native column references derived from the declared field inventory.
        pub mod columns {
            #(
                #[doc = #names]
                pub const #column_constants: crate::columnar::ColumnReference = super::COLUMNS[#positions];
            )*
        }
        /// Borrowed Arrow columns with checked layout and local values.
        /// Keys, references and domain completeness require relational admission.
        #[derive(Debug)]
        pub struct #view<'a> {
            batch: &'a crate::RecordBatch,
            #(#columns: &'a #array_types,)*
        }
        impl<'a> #view<'a> {
            /// Admits a raw candidate's actual schema and visible local values.
            /// # Errors
            /// A schema, field contract or local value violation.
            pub fn try_from_batch(batch: &'a crate::RecordBatch) -> Result<Self, crate::RelationError> {
                Self::try_from_batch_with_registry(pse_schema::registry()?, batch)
            }
            /// Admits a raw candidate with an explicitly bound registry.
            /// # Errors
            /// A schema, field contract or local value violation.
            pub fn try_from_batch_with_registry(registry: &pse_schema::Registry, batch: &'a crate::RecordBatch) -> Result<Self, crate::RelationError> {
                crate::validate::validate_batch(registry, spec(registry)?, batch).map_err(|errors| crate::RelationError::Validation { errors })?;
                Self::borrow_columns(batch)
            }
            /// Borrows a checked owner without rescanning visible values.
            /// The private owner must carry this exact relation and complete declaration.
            /// # Errors
            /// A different generated declaration or an incompatible Arrow layout.
            pub fn from_checked(owner: &'a crate::columnar::FieldCheckedBatch) -> Result<Self, crate::RelationError> {
                Self::borrow_columns(owner.for_generated(RELATION_ID, crate::generated::contracts::expected())?)
            }
            fn borrow_columns(batch: &'a crate::RecordBatch) -> Result<Self, crate::RelationError> {
                Ok(Self { batch, #(#columns: crate::columnar::array::<#array_types>(batch.column(#positions).as_ref())?,)* })
            }
            /// The immutable batch, preserving its buffer owners and reservations.
            pub const fn batch(&self) -> &'a crate::RecordBatch { self.batch }
            /// The number of visible relation rows.
            pub fn len(&self) -> usize { self.batch.num_rows() }
            /// Whether this view has no relation rows.
            pub fn is_empty(&self) -> bool { self.len() == 0 }
            #(
                #[doc = concat!("Borrows the actual Arrow column `", #names, "`, including its offsets and validity bitmap.")]
                #accessor_attributes
                pub const fn #columns(&self) -> &'a #array_types { self.#columns }
                #[doc = concat!("Borrows the exact declared field for `", #names, "`.")]
                #accessor_attributes
                pub fn #field_refs(&self) -> &'a crate::FieldRef { &self.batch.schema_ref().fields()[#positions] }
            )*
            /// Decodes one row for an explicit scalar algorithm boundary.
            /// Columnar consumers should borrow the concrete column accessors.
            /// # Errors
            /// The row is out of range or its physical value cannot be decoded.
            pub fn row(&self, index: usize) -> Result<#row, crate::RelationError> {
                if index >= self.len() { return Err(crate::columnar::mismatch("a row within the generated view")); }
                Ok(#row { #(#fields: crate::columnar::ArrowValue::read(self.#columns, index)?,)* })
            }
            /// Decodes rows directly from Arrow for an explicit scalar algorithm boundary.
            /// This performs no schema/value admission and creates no `serde_json::Value` intermediates.
            /// # Errors
            /// A physical value cannot be decoded.
            pub fn rows(&self) -> Result<Vec<#row>, crate::RelationError> {
                (0..self.len()).map(|index| self.row(index)).collect()
            }
        }
        /// Shared native column builder for this declaration.
        pub type #builder = crate::columnar::RowBuilder<#row>;
        impl crate::columnar::RelationView for #view<'_> {
            type Row = #row;
            fn batch(&self) -> &crate::RecordBatch { self.batch }
        }

    })
}

fn accessor_attributes(name: &str) -> TokenStream {
    if name.starts_with("from_") {
        quote!(#[expect(clippy::wrong_self_convention, reason = "borrowed accessor name preserves the declared relation field")])
    } else {
        quote!()
    }
}
