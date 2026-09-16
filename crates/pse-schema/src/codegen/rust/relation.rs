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
    let keys = spec.columns.iter().map(|column| ident(column.name()));
    let ids = spec.id.as_bytes();
    let fingerprint = spec.fingerprint.as_bytes();
    let name = spec.key.name;
    let namespace = format_ident!("{}", pascal(spec.key.namespace.as_str()));
    let version = spec.key.version;
    let support = support(spec, &row, &view, &builder)?;
    let allocation = super::columnar::allocation(spec);
    let minimum_row = spec
        .columns
        .iter()
        .map(|column| super::columnar::minimum_allocation(&column.value_type().data_type()))
        .sum::<usize>();
    let builder_allocation = crate::compiled_contract::relation(reg, spec)?
        .literal_spec()
        .len()
        .checked_mul(8)
        .and_then(|value| value.checked_add(minimum_row.checked_mul(128)?))
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
    let admission = admission(reg, spec)?;
    Ok(quote! {
        /// The declared relation identity.
        pub const RELATION_ID: pse_ids::SemanticId = pse_ids::SemanticId::from_bytes([#(#ids),*]);
        /// The declared name within its namespace.
        pub const NAME: &str = #name;
        /// The declared namespace.
        pub const NAMESPACE: pse_schema::model::Namespace = pse_schema::model::Namespace::#namespace;
        /// The schema generation.
        pub const VERSION: u32 = #version;
        /// The generated contract identity, not evidence of row validity.
        pub const FINGERPRINT: pse_ids::ContentHash = pse_ids::ContentHash::from_bytes([#(#fingerprint),*]);
        #(#declarations)*
        /// The concrete generated relation row.
        pub type Row = #row;
        /// The concrete generated relation view.
        pub type View<'a> = #view<'a>;
        /// The concrete generated relation builder.
        pub type Builder = #builder;
        impl #row {
            /// Values in exact declared column order.
            pub fn into_cells(self) -> Vec<pse_schema::model::Cell> {
                vec![#(crate::typed::CellCodec::into_cell(self.#keys)),*]
            }
            /// Decode a row after its enclosing batch has been admitted.
            /// # Errors
            /// A value-kind or row-width mismatch.
            pub fn from_cells(values: Vec<pse_schema::model::Cell>) -> Result<Self, crate::RelationError> {
                <Self as crate::typed::CellCodec>::from_cell(pse_schema::model::Cell::Struct(values))
            }
        }
        #admission
        /// The schema built from the authoritative declaration.
        /// # Errors
        /// An unavailable registry or incompatible generated contract.
        pub fn schema() -> Result<crate::SchemaRef, crate::RelationError> {
            let reg = pse_schema::registry()?;
            Ok(std::sync::Arc::new(pse_schema::arrow::relation_schema(reg, spec(reg)?)?))
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
            fn relation(registry: &pse_schema::Registry) -> Result<&pse_schema::model::RelationSpec, crate::RelationError> { spec(registry) }
            fn builder(registry: &pse_schema::Registry, capacity: usize) -> Result<Self::Builder, crate::RelationError> { #builder::with_registry(registry, capacity) }
            fn push(builder: &mut Self::Builder, row: Self) -> Result<(), crate::RelationError> { builder.push(row) }
            fn finish(builder: Self::Builder) -> Result<crate::columnar::FieldCheckedBatch, crate::RelationError> { builder.finish() }
            fn rows(batch: &crate::columnar::FieldCheckedBatch) -> Result<Vec<Self>, crate::RelationError> { #view::from_checked(batch)?.rows() }
            fn builder_allocation_size() -> usize { #builder_allocation + size_of::<Self::Builder>() }
            fn minimum_row_allocation_size() -> usize { #minimum_row }
            fn allocation_size(&self) -> Result<usize, crate::RelationError> { #allocation }
        }
        #support
    })
}

fn admission(reg: &Registry, spec: &RelationSpec) -> Result<TokenStream, SchemaError> {
    let declaration = crate::compiled_contract::relation(reg, spec)?.literal_spec();
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
            mod compiled_admission_test {
                #[test]
                fn equal_fingerprint_cannot_admit_a_changed_declaration() -> Result<(), crate::RelationError> {
                    let registry = pse_schema::registry()?;
                    let original = super::spec(registry)?;
                    let mut altered = original.clone();
                    altered.columns[0] = altered.columns[0].clone().with_nullable(!altered.columns[0].nullable());
                    assert_eq!(altered.fingerprint, super::FINGERPRINT);
                    assert!(matches!(super::check_declaration(registry, &altered), Err(crate::RelationError::Contract { .. })));
                    Ok(())
                }
            }
        }
    } else {
        TokenStream::new()
    };
    Ok(quote! {
        const COMPILED_DECLARATION: &str = #declaration;
        /// Resolves this exact generated contract in a runtime registry.
        /// # Errors
        /// A missing or incompatible declaration.
        pub fn spec(reg: &pse_schema::Registry) -> Result<&pse_schema::model::RelationSpec, crate::RelationError> {
            let spec = reg.relation_by_id(RELATION_ID).ok_or_else(|| crate::RelationError::UnknownRegistry { relation: #qualified.to_owned() })?;
            check_declaration(reg, spec)?;
            Ok(spec)
        }
        fn check_declaration(reg: &pse_schema::Registry, spec: &pse_schema::model::RelationSpec) -> Result<(), crate::RelationError> {
            if spec.fingerprint != FINGERPRINT {
                return Err(crate::RelationError::FingerprintMismatch { relation: #qualified.to_owned(), expected: FINGERPRINT, actual: spec.fingerprint });
            }
            if reg.compiled_declaration(spec)? != COMPILED_DECLARATION {
                return Err(crate::RelationError::Contract {
                    relation: #qualified.to_owned(),
                    reason: "runtime declaration differs from the complete generated contract".to_owned(),
                });
            }
            Ok(())
        }
        #test
    })
}

#[expect(
    clippy::too_many_lines,
    reason = "one relation declaration generates its matching borrowed columns and owned Arrow builder"
)]
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
    let checks = super::columnar::checks(spec)?;
    let row_index = (!checks.is_empty()).then(|| quote!(let row_index = self.columns.len();));
    Ok(quote! {
        /// The complete declared relation key.
        pub const RELATION_KEY: pse_schema::model::RelationKey = pse_schema::model::RelationKey {
            namespace: NAMESPACE, name: NAME, version: VERSION,
        };
        /// Stable field references projected from the declared column order.
        pub const COLUMNS: [crate::columnar::ColumnReference; #count] = [
            #(crate::columnar::ColumnReference { relation_id: RELATION_ID, name: #names, position: #positions },)*
        ];
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
                Self::borrow_columns(owner.for_declaration(RELATION_ID, COMPILED_DECLARATION)?)
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
            /// This performs no schema/value admission and creates no `Cell` intermediates.
            /// # Errors
            /// A physical value cannot be decoded.
            pub fn rows(&self) -> Result<Vec<#row>, crate::RelationError> {
                (0..self.len()).map(|index| self.row(index)).collect()
            }
        }
        /// Appends typed values directly into the declared Arrow column builders.
        /// Finish establishes layout and local value contracts, not relational validity.
        #[derive(Debug)]
        pub struct #builder { columns: crate::columnar::BatchBuilder }
        impl #builder {
            /// Opens empty Arrow builders under the generated declaration.
            /// # Errors
            /// The runtime registry or declared storage is incompatible.
            pub fn new() -> Result<Self, crate::RelationError> { Self::with_capacity(0) }
            /// Reserves initial Arrow column capacity.
            /// # Errors
            /// The runtime registry or declared storage is incompatible.
            pub fn with_capacity(capacity: usize) -> Result<Self, crate::RelationError> {
                Self::with_registry(pse_schema::registry()?, capacity)
            }
            /// Opens Arrow builders after checking the exact runtime declaration once.
            /// # Errors
            /// A generated declaration mismatch or unrepresentable storage capacity.
            pub fn with_registry(registry: &pse_schema::Registry, capacity: usize) -> Result<Self, crate::RelationError> {
                let schema = std::sync::Arc::new(pse_schema::arrow::relation_schema(registry, spec(registry)?)?);
                Ok(Self { columns: crate::columnar::BatchBuilder::new(RELATION_ID, COMPILED_DECLARATION, schema, capacity)? })
            }
            /// Number of appended rows, including retained bulk column chunks.
            pub const fn len(&self) -> usize { self.columns.len() }
            /// Whether the builder contains no rows.
            pub const fn is_empty(&self) -> bool { self.len() == 0 }
            /// Checks local logical values before appending directly to Arrow buffers.
            /// Local refusal does not modify the builder; storage failure closes it.
            /// # Errors
            /// An extension value, quantity sibling or Arrow storage violation.
            pub fn push(&mut self, row: #row) -> Result<(), crate::RelationError> {
                #row_index
                #(#checks)*
                self.columns.append(move |columns| {
                    let row = &row;
                    #(crate::columnar::ArrowValue::append(&row.#fields, columns[#positions].as_mut())?;)*
                    Ok(())
                })
            }
            /// Appends an admitted view's actual Arrow arrays without decoding rows.
            /// # Errors
            /// A different field contract, Arrow failure or row-count overflow.
            pub fn append_view(&mut self, view: &#view<'_>) -> Result<(), crate::RelationError> {
                self.columns.append_batch(view.batch())
            }
            /// Finishes Arrow buffers and retains private local field evidence.
            /// # Errors
            /// Arrow rejected the physical layout or a previous append failed.
            pub fn finish(self) -> Result<crate::columnar::FieldCheckedBatch, crate::RelationError> {
                self.columns.finish()
            }
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
