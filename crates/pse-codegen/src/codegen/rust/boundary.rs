// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Registry boundary traits from the same resolved fields that emit the native row.
use crate::{SchemaError, model::RelationSpec};
use proc_macro2::TokenStream;
#[cfg(test)]
use quote::{format_ident, quote};

pub(super) fn record(
    spec: &RelationSpec,
    row: &syn::Ident,
    fields: &[(String, TokenStream, String)],
    stem: &str,
) -> Result<TokenStream, SchemaError> {
    let Some(record) = spec.boundary else {
        return Ok(TokenStream::new());
    };
    super::boundary_record::render(
        &crate::model::BoundaryShape::record(record),
        &spec.columns,
        row,
        fields,
        &spec
            .columns
            .iter()
            .map(|field| super::types::relation_field_stem(stem, field.name()))
            .collect::<Vec<_>>(),
        super::super::native::Mode::Domain,
    )
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
        let emitted = record(
            &owner,
            &format_ident!("ReferenceQuantityTypesRow"),
            &fields,
            "ReferenceQuantityTypes",
        )
        .unwrap();
        syn::parse2::<syn::File>(emitted.clone()).unwrap();
        let text = emitted.to_string();
        assert!(text.contains("RegistryPhysicalQuantityType"));
        assert!(text.contains("37u32") || text.contains("37"));
        assert!(text.contains("pse.domain.fk.relation") && text.contains("reference.units"));
        assert!(text.contains("ProjectionRecord") && text.contains("Direction :: Output"));
        assert!(
            text.contains("Result < Self , pse_contract :: boundary_projection :: AdapterError >")
        );
    }

    #[test]
    fn signed_integer_domain_is_lowered_from_the_original_registry_field() {
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
        let emitted = record(&owner, &format_ident!("BoundedRow"), &fields, "Bounded")
            .unwrap()
            .to_string();
        assert!(emitted.contains("SignedRange") && emitted.contains("128"));
        assert!(emitted.contains("validate_field"));
    }
}
