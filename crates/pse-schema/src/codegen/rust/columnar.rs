// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Borrowed column types and local checks mechanically projected for generated builders.

use arrow_schema::{DataType, TimeUnit};
use proc_macro2::TokenStream;
use quote::quote;

use crate::SchemaError;
use crate::model::{ExtensionUse, FieldContract, QuantityContract, RelationSpec};

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

pub(super) fn checks(spec: &RelationSpec) -> Result<Vec<TokenStream>, SchemaError> {
    let mut checks = spec
        .columns
        .iter()
        .map(|column| {
            let name = ident(column.name());
            check(
                &column.value_type(),
                column.nullable(),
                &quote!(row.#name),
                column.name(),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    checks.retain(|check| !check.is_empty());
    for column in &spec.columns {
        if column.quantity() != QuantityContract::PerRow {
            continue;
        }
        let sibling_name = column.per_row_quantity_sibling();
        let Some(sibling) = spec
            .columns
            .iter()
            .find(|candidate| candidate.name() == sibling_name)
        else {
            continue;
        };
        // Registry admission requires the sibling to be an ID. A non-nullable generated
        // ID already satisfies this local implication for every visible measure.
        if !sibling.nullable() {
            continue;
        }
        let name = ident(column.name());
        let sibling = ident(sibling.name());
        let path = column.name();
        let visible = if column.nullable() {
            quote!(row.#name.is_some())
        } else {
            quote!(true)
        };
        checks.push(quote! {
            if #visible && row.#sibling.is_none() {
                return Err(crate::columnar::value_error(#path, row_index, "visible per-row quantity requires a non-null sibling quantity identity"));
            }
        });
    }
    Ok(checks)
}

fn check(
    ty: &FieldContract,
    nullable: bool,
    value: &TokenStream,
    path: &str,
) -> Result<TokenStream, SchemaError> {
    let inner = if nullable {
        quote!(value)
    } else {
        value.clone()
    };
    if let Some(range) = crate::model::IntegerRange::from_field(ty.field())? {
        let minimum = range.minimum;
        let maximum = range.maximum;
        return Ok(local_check(
            &quote!((#minimum..=#maximum).contains(&(#inner).to_owned())),
            &quote!("value outside declared integer domain"),
            nullable,
            value,
            path,
        ));
    }
    let body = match (ty.extension(), ty.data_type()) {
        (None, DataType::List(child) | DataType::FixedSizeList(child, _)) => {
            let child = check(
                &FieldContract::from_field((*child).clone()),
                child.is_nullable(),
                &quote!(item),
                path,
            )?;
            if child.is_empty() {
                return Ok(TokenStream::new());
            }
            quote!((#inner).iter().try_for_each(|item| { #child Ok::<(), crate::RelationError>(()) })?;)
        }
        (None, DataType::Struct(fields)) => {
            let mut children = fields
                .iter()
                .map(|field| {
                    let name = field.name();
                    let ty = FieldContract::from_field((**field).clone());
                    let nullable = field.is_nullable();
                    let field = ident(name);
                    check(
                        &ty,
                        nullable,
                        &quote!((#inner).#field),
                        &format!("{path}.{name}"),
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            children.retain(|tokens| !tokens.is_empty());
            if children.is_empty() {
                return Ok(TokenStream::new());
            }
            quote!(#(#children)*)
        }
        (Some(ExtensionUse::DimensionVector), _) => {
            return Ok(local_check(
                &quote!((#inner).iter().all(|exponent| crate::validate::local_values::dimension_exponent(i64::from(exponent.num), i64::from(exponent.den)))),
                &quote!(crate::validate::local_values::DIMENSION_ERROR),
                nullable,
                value,
                path,
            ));
        }
        (Some(ExtensionUse::Bound), _) => {
            return Ok(local_check(
                &quote!(crate::validate::local_values::bound((#inner).kind.as_str(), (#inner).value)),
                &quote!(crate::validate::local_values::BOUND_ERROR),
                nullable,
                value,
                path,
            ));
        }
        (Some(ExtensionUse::SourceSpan), _) => {
            return Ok(local_check(
                &quote!(crate::validate::local_values::source_span((#inner).start, (#inner).end)),
                &quote!(crate::validate::local_values::SPAN_ERROR),
                nullable,
                value,
                path,
            ));
        }
        (Some(ExtensionUse::QuantityValue), _) => {
            return Ok(local_check(
                &quote!(crate::validate::local_values::quantity((#inner).value)),
                &quote!(crate::validate::local_values::QUANTITY_ERROR),
                nullable,
                value,
                path,
            ));
        }
        _ => return Ok(TokenStream::new()),
    };
    Ok(if nullable {
        quote!(if let Some(value) = (#value).as_ref() { #body })
    } else {
        body
    })
}

fn local_check(
    valid: &TokenStream,
    message: &TokenStream,
    nullable: bool,
    value: &TokenStream,
    path: &str,
) -> TokenStream {
    let invalid = if nullable {
        quote!(let Some(value) = (#value).as_ref() && !(#valid))
    } else {
        quote!(!(#valid))
    };
    quote! { if #invalid { return Err(crate::columnar::value_error(#path, row_index, #message)); } }
}
