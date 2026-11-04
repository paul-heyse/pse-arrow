// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared projection of top-level and nested actual registry records.

use crate::{
    SchemaError,
    model::{BoundaryShape, FieldContract, TaggedAlternative},
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(super) fn render(
    shape: &BoundaryShape,
    columns: &[FieldContract],
    row: &syn::Ident,
    fields: &[(String, TokenStream, String)],
    field_stems: &[String],
    mode: super::super::native::Mode,
) -> Result<TokenStream, SchemaError> {
    if fields.len() != columns.len()
        || field_stems.len() != columns.len()
        || fields
            .iter()
            .zip(columns)
            .any(|((name, _, _), column)| name != column.name())
    {
        return Err(super::error(
            "registry boundary must project exactly its actual native fields".into(),
        ));
    }
    let name = shape.name.clone().unwrap_or_else(|| row.to_string());
    let source = &shape.source;
    let version = shape.version;
    let direction = format_ident!("{}", format!("{:?}", shape.direction));
    let reserved_tags = &shape.reserved_tags;
    let reserved_names = &shape.reserved_names;
    let mut definitions = Vec::new();
    let mut layout = Vec::new();
    let mut names = Vec::new();
    let mut types = Vec::new();
    let mut recursive = Vec::new();
    let mut field_rules = Vec::new();
    for (((field_name, ty, _), column), field_stem) in fields.iter().zip(columns).zip(field_stems) {
        // Present malformed annotations must not disappear through convenience defaults.
        column.checked_role()?;
        column.checked_extension()?;
        column.identity_contract()?;
        column.canonical_policy()?;
        column.expression_source_contract()?;
        let ty = super::semantic::model_type(ty)?;
        let native_projection = match mode {
            super::super::native::Mode::Domain => {
                super::types::projection_type(column, field_stem)?
            }
            super::super::native::Mode::Storage => {
                super::types::projection_type_in(column, field_stem, mode)?
            }
        };
        let native_projection = super::types::projection_tokens(&native_projection);
        if TaggedAlternative::from_field(column.field())?.is_some()
            && BoundaryShape::from_field(column.field())?.is_none()
        {
            return Err(super::error(format!(
                "{name}.{field_name}: native alternatives need explicit wire presentation"
            )));
        }
        let number = column.boundary_tag().ok_or_else(|| {
            super::error(format!(
                "{name}.{field_name} has no source-owned boundary number"
            ))
        })?;
        let field = super::types::ident(field_name);
        let rules = super::types::field_rules(column)?
            .into_iter()
            .map(|(_, tokens)| tokens)
            .collect::<Vec<_>>();
        field_rules.push(!rules.is_empty());
        let mut metadata = column
            .field()
            .metadata()
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<std::collections::BTreeMap<_, _>>();
        // Resolve scalar FK nullability at its actual native owner, not from a later wire type.
        if column.checked_fk()?.is_some()
            && let Some(reference) = crate::model::ReferenceContract::for_contract(column)?
        {
            metadata.insert(
                crate::model::reference::KEY_REFERENCE.into(),
                reference.canonical()?,
            );
        }
        let native_field = column.field().clone().with_metadata(
            metadata
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
        );
        metadata.insert(
            super::super::boundary_projection::REGISTRY_FIELD_SOURCE.into(),
            crate::model::field::render_field(&native_field)?,
        );
        let facets = metadata
            .into_iter()
            .map(|(key, value)| quote!((#key.into(), #value.into())))
            .collect::<Vec<_>>();
        definitions.push(
            quote!({
                let projected = pse_contract::boundary_projection::Field::of::<#ty>(graph, #field_name, #number,
                    pse_contract::boundary_projection::Direction::#direction, vec![#(#rules),*])?;
                if projected.ty != #native_projection {
                    return Err(pse_contract::boundary_projection::ProjectionError::invalid(#field_name,
                        "compiler-resolved native field differs from its source Rust/Arrow projection"));
                }
                projected.with_named_facets(std::collections::BTreeMap::from([#(#facets),*]))
            }),
        );
        layout.push(quote!((#number, #field_name)));
        names.push(field);
        types.push(ty);
        let field = super::types::ident(field_name);
        recursive.push(
            quote!(pse_contract::boundary_projection::validate_field(&self.#field,
            &pse_contract::boundary_projection::field_path(path, #field_name), &[#(#rules),*])?;),
        );
    }
    Ok(quote! {
        impl pse_contract::boundary_projection::ProjectionType for #row {
            fn projection(graph: &mut pse_contract::boundary_projection::ProjectionGraph)
                -> Result<pse_contract::boundary_projection::TypeRef, pse_contract::boundary_projection::ProjectionError> {
                graph.declare::<Self>(#name, #source, #version,
                    pse_contract::boundary_projection::Direction::#direction, &[#(#reserved_tags),*], &[#(#reserved_names),*],
                    |graph| Ok(pse_contract::boundary_projection::Shape::Record(vec![#(#definitions),*])))
            }
        }
        impl pse_contract::boundary_projection::ProjectionRecord for #row {
            const VERSION: u32 = #version;
            const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[#(#layout),*];
            type Fields = (#(#types,)*);
            fn into_projection_fields(self) -> Self::Fields { (#(self.#names,)*) }
            fn from_projection_fields((#(#names,)*): Self::Fields)
                -> Result<Self, pse_contract::boundary_projection::AdapterError> { Ok(Self { #(#names,)* }) }
        }
        impl pse_contract::boundary_projection::ValidateProjection for #row {
            const NEEDS_RECURSIVE_VALIDATION: bool = false #(|| #field_rules || <#types as pse_contract::boundary_projection::ValidateProjection>::NEEDS_RECURSIVE_VALIDATION)*;
            fn validate_at(&self, path: &str) -> Result<(), pse_contract::boundary_projection::ValidationError> {
                if Self::NEEDS_RECURSIVE_VALIDATION { #(#recursive)* }
                Ok(())
            }
        }
    })
}
