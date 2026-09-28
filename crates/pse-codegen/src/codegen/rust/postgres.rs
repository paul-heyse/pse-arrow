// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! PostgreSQL values of registry types (ADR-0114 Outcome 25), generated into `pse-model`
//! behind its `postgres` feature: every store enumeration maps to its ENUM type by name
//! and member set, and every typed id to its identity domain (a result column arrives as
//! the base type, so the base type is accepted too).

use quote::{format_ident, quote};

use crate::Registry;
use crate::model::IdentityBase;

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
    super::emit(
        tree,
        MODEL,
        quote! {
            //! PostgreSQL values of the operational store's registry enums and typed ids.
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
