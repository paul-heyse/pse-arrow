// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Generate one frozen native contract graph, independent of runtime assembly.

use crate::model::FieldContract;
use crate::model::{EnumSpec, RelationSpec};
use crate::resolved_contract::{ResolvedExtensionContract, ResolvedRelationContract};
use crate::{Registry, SchemaError};
use arrow_schema::{DataType, Field};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::BTreeMap;

pub(super) fn render(reg: &Registry) -> Result<TokenStream, SchemaError> {
    let graph = reg.generated_contracts()?;
    let mut pool = FieldPool::default();
    for node in graph.relations() {
        for field in node.declaration.columns.iter().chain(&node.fields) {
            pool.intern(field.field());
        }
    }
    for extension in graph.extensions() {
        pool.intern(extension.storage.field());
    }
    let mut field_builders = Vec::new();
    let mut field_calls = Vec::new();
    for (index, value) in pool.fields.iter().enumerate() {
        let name = format_ident!("field_{index}");
        let value = field_definition(value, &pool)?;
        field_builders.push(quote! {
            fn #name(fields: &[std::sync::Arc<arrow_schema::Field>]) -> Result<arrow_schema::Field, pse_schema::SchemaError> { let _ = fields; Ok(#value) }
        });
        field_calls.push(name);
    }
    let mut declarations = Vec::new();
    let mut calls = Vec::new();
    for (index, contract) in graph.relations().enumerate() {
        let name = format_ident!("relation_{index}");
        let value = relation(contract, &pool)?;
        declarations.push(quote! {
            fn #name(fields: &[std::sync::Arc<arrow_schema::Field>]) -> pse_schema::resolved_contract::ResolvedRelationContract { #value }
        });
        calls.push(name);
    }
    let mut enum_builders = Vec::new();
    let mut enum_calls = Vec::new();
    for (index, value) in graph.enums().enumerate() {
        let name = format_ident!("enum_{index}");
        let value = enumeration(value);
        enum_builders.push(quote!(fn #name() -> pse_schema::model::EnumSpec { #value }));
        enum_calls.push(name);
    }
    let extensions = graph
        .extensions()
        .map(|e| extension(e, &pool))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(quote! {
        //! Frozen generated expectations; these never resolve against the receiving registry.
        #![allow(clippy::unnecessary_wraps, reason = "native field constructor table shares one fallible signature across all Arrow types")]
        static EXPECTED: std::sync::OnceLock<Result<pse_schema::resolved_contract::GeneratedContracts, pse_schema::SchemaError>> = std::sync::OnceLock::new();
        pub(crate) fn expected() -> Result<&'static pse_schema::resolved_contract::GeneratedContracts, crate::RelationError> {
            EXPECTED.get_or_init(build).as_ref().map_err(|error| error.clone().into())
        }
        type FieldBuilder = fn(&[std::sync::Arc<arrow_schema::Field>]) -> Result<arrow_schema::Field, pse_schema::SchemaError>;
        type RelationBuilder = fn(&[std::sync::Arc<arrow_schema::Field>]) -> pse_schema::resolved_contract::ResolvedRelationContract;
        static FIELDS: &[FieldBuilder] = &[#(#field_calls),*];
        static RELATIONS: &[RelationBuilder] = &[#(#calls),*];
        static ENUMS: &[fn() -> pse_schema::model::EnumSpec] = &[#(#enum_calls),*];
        fn build() -> Result<pse_schema::resolved_contract::GeneratedContracts, pse_schema::SchemaError> {
            let mut fields: Vec<std::sync::Arc<arrow_schema::Field>> = Vec::with_capacity(FIELDS.len());
            for constructor in FIELDS { fields.push(std::sync::Arc::new(constructor(&fields)?)); }
            let relations = RELATIONS.iter().map(|constructor| constructor(&fields)).collect();
            let enums = ENUMS.iter().map(|constructor| constructor()).collect();
            pse_schema::resolved_contract::GeneratedContracts::new(relations, enums, vec![#(#extensions),*])
        }
        #(#field_builders)*
        #(#enum_builders)*
        #(#declarations)*
    })
}

fn id(id: pse_ids::SemanticId) -> TokenStream {
    let bytes = id.as_bytes();
    quote!(pse_ids::SemanticId::from_bytes([#(#bytes),*]))
}

fn optional(value: Option<&str>) -> TokenStream {
    value.map_or_else(|| quote!(None), |value| quote!(Some(#value)))
}

fn map(values: impl IntoIterator<Item = (String, String)>) -> TokenStream {
    let mut values = values.into_iter().collect::<Vec<_>>();
    values.sort();
    let pairs = values
        .iter()
        .map(|(key, value)| quote!((#key.to_owned(), #value.to_owned())));
    quote!([#(#pairs),*].into_iter().collect())
}

fn relation(
    value: &ResolvedRelationContract,
    pool: &FieldPool,
) -> Result<TokenStream, SchemaError> {
    let declaration = declaration(&value.declaration, pool)?;
    let fields = value
        .fields
        .iter()
        .map(|f| {
            field(f.field(), pool).map(|f| quote!(pse_schema::model::FieldContract::from_field(#f)))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let references = value.references.iter().copied().map(id);
    let enums = value.enums.iter().copied().map(id);
    let extensions = &value.extensions;
    Ok(
        quote!(pse_schema::resolved_contract::ResolvedRelationContract {
            declaration: #declaration, fields: vec![#(#fields),*], references: vec![#(#references),*], enums: vec![#(#enums),*], extensions: vec![#(#extensions.to_owned()),*],
        }),
    )
}

fn declaration(spec: &RelationSpec, pool: &FieldPool) -> Result<TokenStream, SchemaError> {
    let id = id(spec.id);
    let namespace = format_ident!("{}", format!("{:?}", spec.key.namespace));
    let name = spec.key.name;
    let version = spec.key.version;
    let authority = format_ident!("{}", format!("{:?}", spec.authority));
    let snapshot = format_ident!("{}", format!("{:?}", spec.snapshot_class));
    let stability = format_ident!("{}", format!("{:?}", spec.stability));
    let granularity = spec.derivation_granularity.map_or_else(
        || quote!(None),
        |g| {
            let g = format_ident!("{}", format!("{g:?}"));
            quote!(Some(pse_schema::model::DerivationGranularity::#g))
        },
    );
    let keys = &spec.primary_key;
    let columns = spec
        .columns
        .iter()
        .map(|f| {
            field(f.field(), pool).map(|f| quote!(pse_schema::model::FieldContract::from_field(#f)))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let checks = map(spec.checks.clone());
    let properties = map(spec.delta_properties.clone());
    Ok(quote!(pse_schema::model::RelationSpec {
        id: #id, key: pse_schema::model::RelationKey { namespace: pse_schema::model::Namespace::#namespace, name: #name, version: #version },
        authority: pse_schema::model::Authority::#authority, snapshot_class: pse_schema::model::SnapshotClass::#snapshot,
        derivation_granularity: #granularity, stability: pse_schema::model::Stability::#stability,
        primary_key: vec![#(#keys),*], columns: vec![#(#columns),*], checks: #checks, delta_properties: #properties,
        doc: "", fingerprint: pse_ids::ContentHash::NIL,
    }))
}

fn enumeration(value: &EnumSpec) -> TokenStream {
    let id = id(value.id);
    let name = value.name;
    let source = optional(value.idaes_source);
    let members = value.members.iter().map(|member| {
        let name = member.name; let alias = optional(member.idaes_name); let deprecated = member.deprecated;
        quote!(pse_schema::model::EnumMember { name: #name, idaes_name: #alias, deprecated: #deprecated, doc: "" })
    });
    quote!(pse_schema::model::EnumSpec { id: #id, name: #name, idaes_source: #source, members: vec![#(#members),*] })
}

fn extension(
    value: &ResolvedExtensionContract,
    pool: &FieldPool,
) -> Result<TokenStream, SchemaError> {
    let name = &value.name;
    let storage = field(value.storage.field(), pool)?;
    let kind = &value.metadata_kind;
    let version = value.version;
    Ok(
        quote!(pse_schema::resolved_contract::ResolvedExtensionContract {
            name: #name.to_owned(), storage: pse_schema::model::FieldContract::from_field(#storage), metadata_kind: #kind.to_owned(), version: #version,
        }),
    )
}

fn field_definition(value: &Field, pool: &FieldPool) -> Result<TokenStream, SchemaError> {
    let name = value.name();
    let ty = data_type(value.data_type(), pool)?;
    let nullable = value.is_nullable();
    let metadata = map(value.metadata().iter().map(|(k, v)| (k.clone(), v.clone())));
    let ordered = value.dict_is_ordered().unwrap_or(false);
    Ok(
        quote!(arrow_schema::Field::new(#name, #ty, #nullable).with_metadata(#metadata).with_dict_is_ordered(#ordered)),
    )
}

#[expect(
    clippy::too_many_lines,
    reason = "exhaustive native Arrow variant construction is kept in one generator match"
)]
fn data_type(value: &DataType, pool: &FieldPool) -> Result<TokenStream, SchemaError> {
    let result = match value {
        DataType::Timestamp(unit, zone) => {
            let unit = format_ident!("{}", format!("{unit:?}"));
            let zone = zone.as_ref().map_or_else(
                || quote!(None),
                |s| {
                    let s = s.as_ref();
                    quote!(Some(std::sync::Arc::from(#s)))
                },
            );
            quote!(arrow_schema::DataType::Timestamp(arrow_schema::TimeUnit::#unit, #zone))
        }
        DataType::Time32(unit) | DataType::Time64(unit) | DataType::Duration(unit) => {
            let name = match value {
                DataType::Time32(_) => "Time32",
                DataType::Time64(_) => "Time64",
                _ => "Duration",
            };
            let name = format_ident!("{name}");
            let unit = format_ident!("{}", format!("{unit:?}"));
            quote!(arrow_schema::DataType::#name(arrow_schema::TimeUnit::#unit))
        }
        DataType::Interval(unit) => {
            let unit = format_ident!("{}", format!("{unit:?}"));
            quote!(arrow_schema::DataType::Interval(arrow_schema::IntervalUnit::#unit))
        }
        DataType::FixedSizeBinary(size) => quote!(arrow_schema::DataType::FixedSizeBinary(#size)),
        DataType::List(f)
        | DataType::LargeList(f)
        | DataType::ListView(f)
        | DataType::LargeListView(f) => {
            let name = match value {
                DataType::List(_) => "List",
                DataType::LargeList(_) => "LargeList",
                DataType::ListView(_) => "ListView",
                _ => "LargeListView",
            };
            let name = format_ident!("{name}");
            let f = field(f, pool)?;
            quote!(arrow_schema::DataType::#name(std::sync::Arc::new(#f)))
        }
        DataType::FixedSizeList(f, n) => {
            let f = field(f, pool)?;
            quote!(arrow_schema::DataType::FixedSizeList(std::sync::Arc::new(#f), #n))
        }
        DataType::Map(f, sorted) => {
            let f = field(f, pool)?;
            quote!(arrow_schema::DataType::Map(std::sync::Arc::new(#f), #sorted))
        }
        DataType::Struct(fields) => {
            let fields = fields
                .iter()
                .map(|f| field(f, pool))
                .collect::<Result<Vec<_>, _>>()?;
            quote!(arrow_schema::DataType::Struct(arrow_schema::Fields::from(Vec::<arrow_schema::Field>::from([#(#fields),*]))))
        }
        DataType::Union(fields, mode) => {
            let ids = fields.iter().map(|(id, _)| id);
            let fields = fields
                .iter()
                .map(|(_, f)| field(f, pool))
                .collect::<Result<Vec<_>, _>>()?;
            let mode = format_ident!("{}", format!("{mode:?}"));
            quote!(arrow_schema::DataType::Union(arrow_schema::UnionFields::try_new(Vec::<i8>::from([#(#ids),*]),Vec::<arrow_schema::Field>::from([#(#fields),*])).map_err(|error|pse_schema::SchemaError::InvalidDeclaration { context: "generated union".into(), reason: error.to_string() })?, arrow_schema::UnionMode::#mode))
        }
        DataType::Dictionary(key, value) => {
            let key = data_type(key, pool)?;
            let value = data_type(value, pool)?;
            quote!(arrow_schema::DataType::Dictionary(Box::new(#key),Box::new(#value)))
        }
        DataType::RunEndEncoded(run, value) => {
            let run = field(run, pool)?;
            let value = field(value, pool)?;
            quote!(arrow_schema::DataType::RunEndEncoded(std::sync::Arc::new(#run),std::sync::Arc::new(#value)))
        }
        DataType::Decimal32(p, s)
        | DataType::Decimal64(p, s)
        | DataType::Decimal128(p, s)
        | DataType::Decimal256(p, s) => {
            let name = match value {
                DataType::Decimal32(..) => "Decimal32",
                DataType::Decimal64(..) => "Decimal64",
                DataType::Decimal128(..) => "Decimal128",
                _ => "Decimal256",
            };
            let name = format_ident!("{name}");
            quote!(arrow_schema::DataType::#name(#p,#s))
        }
        DataType::Null
        | DataType::Boolean
        | DataType::Int8
        | DataType::Int16
        | DataType::Int32
        | DataType::Int64
        | DataType::UInt8
        | DataType::UInt16
        | DataType::UInt32
        | DataType::UInt64
        | DataType::Float16
        | DataType::Float32
        | DataType::Float64
        | DataType::Date32
        | DataType::Date64
        | DataType::Binary
        | DataType::LargeBinary
        | DataType::BinaryView
        | DataType::Utf8
        | DataType::LargeUtf8
        | DataType::Utf8View => {
            let name = format_ident!("{}", format!("{value:?}"));
            quote!(arrow_schema::DataType::#name)
        }
    };
    Ok(result)
}

#[derive(Default)]
struct FieldPool {
    indices: BTreeMap<FieldContract, usize>,
    fields: Vec<Field>,
}
impl FieldPool {
    fn intern(&mut self, value: &Field) {
        let key = FieldContract::from_field(value.clone());
        if self.indices.contains_key(&key) {
            return;
        }
        for child in crate::model::field::child_fields(value.data_type()) {
            self.intern(child);
        }
        self.indices.insert(key, self.fields.len());
        self.fields.push(value.clone());
    }
}
fn field(value: &Field, pool: &FieldPool) -> Result<TokenStream, SchemaError> {
    let index = pool
        .indices
        .get(&FieldContract::from_field(value.clone()))
        .ok_or_else(|| super::error("native generated field was not interned".into()))?;
    Ok(quote!(fields[#index].as_ref().clone()))
}
