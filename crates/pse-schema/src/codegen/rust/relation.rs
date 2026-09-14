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
                &column.logical_type,
                &format!("{stem}Field{}", pascal(column.name)),
                &mut declarations,
            )?;
            Ok((
                column.name.to_owned(),
                optional(ty, column.nullable),
                column.doc.to_owned(),
            ))
        })
        .collect::<Result<Vec<_>, SchemaError>>()?;
    declarations.push(structure(&format!("{stem}Row"), &fields));
    let keys = spec.columns.iter().map(|column| ident(column.name));
    let ids = spec.id.as_bytes();
    let fingerprint = spec.fingerprint.as_bytes();
    let name = spec.key.name;
    let namespace = format_ident!("{}", pascal(spec.key.namespace.as_str()));
    let version = spec.key.version;
    let support = support(&row, &view, &builder);
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
                    altered.columns[0].nullable = !altered.columns[0].nullable;
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
            if pse_schema::compiled_contract::relation(reg, spec)?.literal_spec() != COMPILED_DECLARATION {
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

fn support(
    row: &proc_macro2::Ident,
    view: &proc_macro2::Ident,
    builder: &proc_macro2::Ident,
) -> TokenStream {
    quote! {
        /// A borrowed batch admitted against the complete generated contract.
        #[derive(Debug)]
        pub struct #view<'a> { batch: &'a crate::RecordBatch, registry: &'a pse_schema::Registry }
        impl<'a> #view<'a> {
            /// Admits the actual schema and visible values, including extension metadata.
            /// # Errors
            /// A schema, contract or value violation.
            pub fn try_from_batch(batch: &'a crate::RecordBatch) -> Result<Self, crate::RelationError> {
                Self::try_from_batch_with_registry(pse_schema::registry()?, batch)
            }
            /// Admits a batch with an explicitly bound registry.
            /// # Errors
            /// A schema, contract or value violation.
            pub fn try_from_batch_with_registry(registry: &'a pse_schema::Registry, batch: &'a crate::RecordBatch) -> Result<Self, crate::RelationError> {
                crate::validate::validate_batch(registry, spec(registry)?, batch).map_err(|errors| crate::RelationError::Validation { errors })?;
                Ok(Self { batch, registry })
            }
            /// The admitted batch, preserving its owners and reservations.
            pub const fn batch(&self) -> &'a crate::RecordBatch { self.batch }
            /// Decode dictionary values and nested fields into generated typed rows.
            /// # Errors
            /// A typed decoding error.
            pub fn rows(&self) -> Result<Vec<#row>, crate::RelationError> {
                crate::cells::cells_from_batch(self.registry, spec(self.registry)?, self.batch)?.into_iter().map(#row::from_cells).collect()
            }
        }
        /// Builds a batch under the declared schema, with admission before return.
        #[derive(Debug, Default)]
        pub struct #builder { rows: Vec<#row> }
        impl #builder {
            /// An empty builder.
            pub fn new() -> Self { Self::default() }
            /// Reserves row capacity.
            pub fn with_capacity(capacity: usize) -> Self { Self { rows: Vec::with_capacity(capacity) } }
            /// Checks one candidate row before adding it. Bundle constraints run at P2.
            /// # Errors
            /// An extension value or field contract violation.
            pub fn push(&mut self, row: #row) -> Result<(), crate::RelationError> {
                let reg = pse_schema::registry()?;
                crate::cells::batch_from_cells(reg, spec(reg)?, &[row.clone().into_cells()])?;
                self.rows.push(row);
                Ok(())
            }
            /// Builds and admits the complete batch.
            /// # Errors
            /// A schema, value or Arrow layout failure.
            pub fn finish(self) -> Result<crate::RecordBatch, crate::RelationError> {
                self.finish_with_registry(pse_schema::registry()?)
            }
            /// Builds under an explicitly bound runtime registry.
            /// # Errors
            /// A contract mismatch or batch admission failure.
            pub fn finish_with_registry(self, reg: &pse_schema::Registry) -> Result<crate::RecordBatch, crate::RelationError> {
                let cells = self.rows.into_iter().map(#row::into_cells).collect::<Vec<_>>();
                crate::cells::batch_from_cells(reg, spec(reg)?, &cells)
            }
        }
    }
}
