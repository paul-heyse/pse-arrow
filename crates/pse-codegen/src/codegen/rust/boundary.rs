// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Registry boundary traits from the same resolved fields that emit the native row.
use crate::{SchemaError, model::RelationSpec};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(super) fn record(
    spec: &RelationSpec,
    row: &syn::Ident,
    fields: &[(String, TokenStream, String)],
) -> Result<TokenStream, SchemaError> {
    let Some(record) = spec.boundary else {
        return Ok(TokenStream::new());
    };
    let name = record.name;
    let source = record.source;
    let version = record.version;
    let direction = format_ident!("{}", format!("{:?}", record.direction));
    let reserved_tags = record.reserved_tags;
    let reserved_names = record.reserved_names;
    let mut definitions = Vec::new();
    let mut layout = Vec::new();
    let mut names = Vec::new();
    let mut types = Vec::new();
    let mut recursive = Vec::new();
    for ((name, ty, _), column) in fields.iter().zip(&spec.columns) {
        if crate::model::IntegerRange::from_field(column.field())?.is_some()
            || crate::model::TaggedAlternative::from_field(column.field())?.is_some()
            || crate::model::CollectionContract::from_field(column.field())?
                .is_some_and(|contract| contract != crate::model::CollectionContract::SEQUENCE)
        {
            return Err(super::error(format!(
                "{}.{name}: selected registry boundary requires explicit lowering of its local integer, alternative or collection contract",
                spec.key
            )));
        }
        let number = column.boundary_tag().ok_or_else(|| {
            super::error(format!(
                "{}.{name} has no source-owned boundary number",
                spec.key
            ))
        })?;
        let field = super::types::ident(name);
        let facets = column
            .field()
            .metadata()
            .iter()
            .collect::<std::collections::BTreeMap<_, _>>()
            .into_iter()
            .map(|(key, value)| quote!((#key.into(), #value.into())))
            .collect::<Vec<_>>();
        definitions.push(
            quote!(crate::boundary_projection::Field::of::<#ty>(graph, #name, #number,
            crate::boundary_projection::Direction::#direction, vec![])?
            .with_named_facets(std::collections::BTreeMap::from([#(#facets),*]))),
        );
        layout.push(quote!((#number, #name)));
        names.push(field);
        types.push(ty);
        let field = super::types::ident(name);
        recursive.push(
            quote!(crate::boundary_projection::ValidateProjection::validate_at(&self.#field,
            &crate::boundary_projection::field_path(path, #name))?;),
        );
    }
    Ok(quote! {
        impl crate::boundary_projection::ProjectionType for #row {
            fn projection(graph: &mut crate::boundary_projection::ProjectionGraph)
                -> Result<crate::boundary_projection::TypeRef, crate::boundary_projection::ProjectionError> {
                graph.declare::<Self>(#name, #source, #version,
                    crate::boundary_projection::Direction::#direction, &[#(#reserved_tags),*], &[#(#reserved_names),*],
                    |graph| Ok(crate::boundary_projection::Shape::Record(vec![#(#definitions),*])))
            }
        }
        impl crate::boundary_projection::ProjectionRecord for #row {
            const VERSION: u32 = #version;
            const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[#(#layout),*];
            type Fields = (#(#types,)*);
            fn into_projection_fields(self) -> Self::Fields { (#(self.#names,)*) }
            fn from_projection_fields((#(#names,)*): Self::Fields)
                -> Result<Self, crate::boundary_projection::AdapterError> { Ok(Self { #(#names,)* }) }
        }
        impl crate::boundary_projection::ValidateProjection for #row {
            const NEEDS_RECURSIVE_VALIDATION: bool = false #(|| <#types as crate::boundary_projection::ValidateProjection>::NEEDS_RECURSIVE_VALIDATION)*;
            fn validate_at(&self, path: &str) -> Result<(), crate::boundary_projection::ValidationError> {
                if Self::NEEDS_RECURSIVE_VALIDATION { #(#recursive)* }
                Ok(())
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolved_registry_owner_drives_numbers_links_and_exact_native_fields() {
        let registry = crate::catalog::assemble().unwrap();
        let mut owner = registry
            .relation("reference.quantity_types")
            .unwrap()
            .clone();
        owner.columns[0] = owner.columns[0].clone().with_boundary_tag(37);
        let mut declarations = Vec::new();
        let fields = owner
            .columns
            .iter()
            .map(|field| {
                let ty = match field.identity() {
                    Some(identity) => super::super::identities::path(identity),
                    None => super::super::types::logical(
                        &field.value_type(),
                        "Probe",
                        &mut declarations,
                    )
                    .unwrap(),
                };
                (
                    field.name().into(),
                    super::super::types::optional(ty, field.nullable()),
                    field.doc().into(),
                )
            })
            .collect::<Vec<_>>();
        let emitted = record(&owner, &format_ident!("ReferenceQuantityTypesRow"), &fields).unwrap();
        syn::parse2::<syn::File>(emitted.clone()).unwrap();
        let text = emitted.to_string();
        assert!(text.contains("RegistryPhysicalQuantityType"));
        assert!(text.contains("37u32") || text.contains("37"));
        assert!(text.contains("pse.domain.fk.relation") && text.contains("reference.units"));
        assert!(text.contains("ProjectionRecord") && text.contains("Direction :: Output"));
        assert!(text.contains("Result < Self , crate :: boundary_projection :: AdapterError >"));
    }

    #[test]
    fn unsupported_local_contract_refuses_instead_of_dropping_validation() {
        let registry = crate::catalog::assemble().unwrap();
        let mut owner = registry.relation("reference.bases").unwrap().clone();
        owner.columns[0] = crate::model::FieldContract::nonnegative(128)
            .with_name("basis_id")
            .with_boundary_tag(1);
        let fields = owner
            .columns
            .iter()
            .map(|field| (field.name().into(), quote!(i64), String::new()))
            .collect::<Vec<_>>();
        assert!(record(&owner, &format_ident!("RefusedRow"), &fields).is_err());
    }
}
