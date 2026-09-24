// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Borrowed column types and local checks mechanically projected for generated builders.

use arrow_schema::{DataType, TimeUnit};
use proc_macro2::TokenStream;
use quote::quote;

use crate::SchemaError;
use crate::model::{FieldContract, RelationSpec};

use super::types::ident;

pub(super) fn minimum_allocation(ty: &DataType) -> usize {
    match ty {
        DataType::Struct(fields) => {
            8 + fields
                .iter()
                .map(|field| minimum_allocation(field.data_type()))
                .sum::<usize>()
        }
        DataType::FixedSizeList(child, count) => {
            8 + usize::try_from(*count).unwrap_or(0) * minimum_allocation(child.data_type())
        }
        DataType::List(child) => 8 + minimum_allocation(child.data_type()),
        DataType::FixedSizeBinary(count) => 8 + usize::try_from(*count).unwrap_or(0),
        _ => 16,
    }
}

pub(super) fn allocation(spec: &RelationSpec) -> TokenStream {
    let fields = spec.columns.iter().map(|column| {
        let name = ident(column.name());
        allocation_field(column.field(), column.nullable(), &quote!(self.#name))
    });
    quote! { let mut bytes = 0usize; #(bytes = crate::columnar::allocation_add(bytes, #fields?)?;)* Ok(bytes) }
}

fn allocation_field(
    field: &arrow_schema::Field,
    nullable: bool,
    value: &TokenStream,
) -> TokenStream {
    let ty = field.data_type();
    if nullable {
        let size = allocation_field(field, false, &quote!(value));
        let present = if allocation_reads_value(ty) {
            quote!(let Some(value) = (#value).as_ref())
        } else {
            quote!((#value).is_some())
        };
        return quote! { if #present { crate::columnar::allocation_add(1, #size?) } else { Ok::<usize, crate::RelationError>(1) } };
    }
    if super::super::enum_name(field).is_some() {
        return quote!(crate::columnar::allocation_add(8, (#value).as_str().len()));
    }
    match ty {
        DataType::Utf8 => quote!(crate::columnar::allocation_add(8, (#value).len())),
        DataType::List(child) | DataType::FixedSizeList(child, _) => {
            let binding = if child.is_nullable() || allocation_reads_value(child.data_type()) {
                quote!(item)
            } else {
                quote!(_)
            };
            let child = allocation_field(child, child.is_nullable(), &quote!(item));
            quote! { (#value).iter().try_fold(8usize, |bytes, #binding| crate::columnar::allocation_add(bytes, #child?)) }
        }
        DataType::Struct(fields) => {
            let fields = fields.iter().map(|field| {
                let name = ident(field.name());
                allocation_field(field, field.is_nullable(), &quote!((#value).#name))
            });
            quote! { { let mut bytes = 1usize; #(bytes = crate::columnar::allocation_add(bytes, #fields?)?;)* Ok::<usize, crate::RelationError>(bytes) } }
        }
        DataType::FixedSizeBinary(size) => {
            let size = usize::try_from(*size).unwrap_or(0);
            quote!(Ok::<usize, crate::RelationError>(#size))
        }
        _ => quote!(Ok::<usize, crate::RelationError>(8usize)),
    }
}

fn allocation_reads_value(ty: &DataType) -> bool {
    match ty {
        DataType::Utf8 | DataType::List(_) | DataType::FixedSizeList(..) => true,
        DataType::Struct(fields) => fields
            .iter()
            .any(|field| field.is_nullable() || allocation_reads_value(field.data_type())),
        _ => false,
    }
}

pub(super) fn array_type(ty: &FieldContract) -> Result<TokenStream, SchemaError> {
    Ok(match ty.data_type() {
        DataType::Boolean => quote!(arrow_array::BooleanArray),
        DataType::Int16 => quote!(arrow_array::Int16Array),
        DataType::Int32 => quote!(arrow_array::Int32Array),
        DataType::Int64 => quote!(arrow_array::Int64Array),
        DataType::UInt8 => quote!(arrow_array::UInt8Array),
        DataType::UInt16 => quote!(arrow_array::UInt16Array),
        DataType::UInt32 => quote!(arrow_array::UInt32Array),
        DataType::UInt64 => quote!(arrow_array::UInt64Array),
        DataType::Float64 => quote!(arrow_array::Float64Array),
        DataType::Utf8 => quote!(arrow_array::StringArray),
        DataType::FixedSizeBinary(16 | 32) => quote!(arrow_array::FixedSizeBinaryArray),
        DataType::List(_) => quote!(arrow_array::ListArray),
        DataType::FixedSizeList(..) => quote!(arrow_array::FixedSizeListArray),
        DataType::Struct(_) => quote!(arrow_array::StructArray),
        DataType::Timestamp(TimeUnit::Nanosecond, _) => {
            quote!(arrow_array::TimestampNanosecondArray)
        }
        other => {
            return Err(super::error(format!(
                "no borrowed generated column type for {other}"
            )));
        }
    })
}
