// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! PostgreSQL values of registry types (ADR-0114 Outcome 25), generated into `pse-model`
//! behind its `postgres` feature: every store enumeration maps to its ENUM type by name
//! and member set, every typed id to its identity domain (a result column arrives as the
//! base type, so the base type is accepted too), and every store row decodes from its
//! table's composite row type (Plan 22 X7), field names checked against the registry.

use arrow_schema::{DataType, TimeUnit};
use quote::{format_ident, quote};

use crate::Registry;
use crate::model::{FieldContract, IdentityBase};

/// How a store column is read from its composite field.
fn read(column: &FieldContract) -> proc_macro2::TokenStream {
    let timestamp = matches!(
        column.data_type(),
        DataType::Timestamp(TimeUnit::Microsecond, Some(ref zone)) if zone.as_ref() == "UTC"
    );
    let document = column.document() == Some(crate::model::field::JSON_DOCUMENT);
    match (timestamp, document, column.nullable()) {
        (true, _, false) => quote!(record.micros()?),
        (true, _, true) => quote!(record.opt_micros()?),
        (_, true, false) => quote!(record.json()?),
        (_, true, true) => quote!(record.opt_json()?),
        _ => quote!(record.value()?),
    }
}

const MODEL: &str = "crates/pse-model/src/generated/postgres.rs";

pub(super) fn emit(
    tree: &mut crate::codegen::GeneratedTree,
    reg: &Registry,
) -> Result<(), crate::SchemaError> {
    let schema = pse_schema::store::SCHEMA;
    let mut items = Vec::new();
    for name in crate::codegen::postgres::store_enums(reg) {
        let spec = reg
            .enum_spec(name)
            .ok_or_else(|| super::error(format!("unknown enum {name}")))?;
        let ty = format_ident!("{}", super::types::pascal(name));
        let type_name = crate::codegen::postgres::names::enum_type(name);
        let members = spec.members.iter().map(|member| member.name);
        items.push(quote! {
            impl ToSql for crate::generated::enums::#ty {
                fn to_sql(&self, _: &Type, out: &mut BytesMut) -> Result<IsNull, BoxError> {
                    out.extend_from_slice(self.as_str().as_bytes());
                    Ok(IsNull::No)
                }
                fn accepts(ty: &Type) -> bool {
                    enum_type(ty, #type_name, &[#(#members),*])
                }
                postgres_types::to_sql_checked!();
            }
            impl<'a> FromSql<'a> for crate::generated::enums::#ty {
                fn from_sql(_: &Type, raw: &'a [u8]) -> Result<Self, BoxError> {
                    Ok(postgres_protocol::types::text_from_sql(raw)?.parse()?)
                }
                fn accepts(ty: &Type) -> bool {
                    <Self as ToSql>::accepts(ty)
                }
            }
        });
    }
    for identity in reg.identities() {
        let ty = format_ident!("{}", super::identities::type_name(identity.name));
        let domain = crate::codegen::postgres::names::identity_domain(identity.name);
        let base = match identity.base {
            IdentityBase::SemanticId => quote!(pse_ids::SemanticId),
            IdentityBase::ContentHash => quote!(pse_ids::ContentHash),
        };
        items.push(quote! {
            impl ToSql for crate::generated::identities::#ty {
                fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, BoxError> {
                    ToSql::to_sql(&self.as_id(), pse_ids::postgres::base(ty), out)
                }
                fn accepts(ty: &Type) -> bool {
                    pse_ids::postgres::accepts_domain::<#base>(ty, #schema, #domain)
                }
                postgres_types::to_sql_checked!();
            }
            impl<'a> FromSql<'a> for crate::generated::identities::#ty {
                fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, BoxError> {
                    <#base as FromSql<'a>>::from_sql(pse_ids::postgres::base(ty), raw).map(Self::from_id)
                }
                fn accepts(ty: &Type) -> bool {
                    <Self as ToSql>::accepts(ty)
                }
            }
        });
    }
    for (table, spec) in pse_schema::store::relations(reg) {
        let namespace = super::types::ident(spec.key.namespace.as_str());
        let module = super::types::ident(spec.key.name);
        let row = format_ident!(
            "{}{}Row",
            super::types::pascal(spec.key.namespace.as_str()),
            super::types::pascal(spec.key.name)
        );
        let names = spec.columns.iter().map(FieldContract::name);
        let fields = spec.columns.iter().map(|column| {
            let field = super::types::ident(column.name());
            let read = read(column);
            quote!(#field: #read)
        });
        items.push(quote! {
            impl<'a> FromSql<'a> for crate::generated::#namespace::#module::#row {
                fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, BoxError> {
                    let mut record = crate::postgres::Record::read(ty, raw, &[#(#names),*])?;
                    let row = Self { #(#fields,)* };
                    record.finish()?;
                    Ok(row)
                }
                fn accepts(ty: &Type) -> bool {
                    crate::postgres::composite(ty, #schema, #table)
                }
            }
        });
    }
    super::emit(
        tree,
        MODEL,
        quote! {
            //! PostgreSQL values of the operational store's registry enums, typed ids and
            //! rows.
            use pse_ids::postgres::BytesMut;
            use postgres_types::{FromSql, IsNull, Kind, ToSql, Type};
            type BoxError = Box<dyn std::error::Error + Sync + Send>;
            /// An ENUM type of the store schema with exactly the registry's members, in order.
            fn enum_type(ty: &Type, name: &str, members: &[&str]) -> bool {
                ty.schema() == #schema
                    && ty.name() == name
                    && matches!(ty.kind(), Kind::Enum(labels)
                        if labels.len() == members.len() && labels.iter().zip(members).all(|(label, member)| label == member))
            }
            #(#items)*
        },
    )
}
