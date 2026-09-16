// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Meaningful rendering of extension storage. Rendering never substitutes for admission.

use arrow::util::display::{
    ArrayFormatter, ArrayFormatterFactory, DisplayIndex, FormatOptions, FormatResult,
};
use arrow_array::{
    Array, FixedSizeBinaryArray, FixedSizeListArray, Int16Array, Int64Array, ListArray, StructArray,
};
use arrow_schema::{ArrowError, Field};
use pse_ids::{ContentHash, SemanticId};
use std::borrow::Cow;
use std::fmt::Write;

/// Arrow formatter shared with the engine's extension registration. Identities render
/// explicitly; resolving an owner or unit symbol requires the caller's model context.
#[derive(Debug, Default)]
pub struct PseFormatterFactory;

impl ArrayFormatterFactory for PseFormatterFactory {
    fn create_array_formatter<'a>(
        &self,
        array: &'a dyn Array,
        options: &FormatOptions<'a>,
        field: Option<&'a Field>,
    ) -> Result<Option<ArrayFormatter<'a>>, ArrowError> {
        let Some(field) = field else {
            return Ok(None);
        };
        create_formatter(array, options, Cow::Borrowed(field))
    }
}

/// Create an extension formatter that owns its field descriptor. Engine extension
/// registries can drop their temporary field while the returned display still borrows
/// the array safely.
///
/// # Errors
/// Invalid extension metadata or a field whose storage differs from the array.
pub fn create_owned_formatter<'a>(
    array: &'a dyn Array,
    options: &FormatOptions<'a>,
    field: Field,
) -> Result<Option<ArrayFormatter<'a>>, ArrowError> {
    create_formatter(array, options, Cow::Owned(field))
}
fn create_formatter<'a>(
    array: &'a dyn Array,
    options: &FormatOptions<'a>,
    field: Cow<'a, Field>,
) -> Result<Option<ArrayFormatter<'a>>, ArrowError> {
    let Some(name) = field.metadata().get(pse_schema::arrow::KEY_EXTENSION_NAME) else {
        return Ok(None);
    };
    if !name.starts_with("pse.") {
        return Ok(None);
    }
    super::validate_extension(&field)?;
    if field.data_type() != array.data_type() {
        return Err(invalid("formatter field and array storage disagree"));
    }
    Ok(Some(ArrayFormatter::new(
        Box::new(PseDisplay {
            array,
            field,
            null: options.null().to_owned(),
        }),
        options.safe(),
    )))
}

struct PseDisplay<'a> {
    array: &'a dyn Array,
    field: Cow<'a, Field>,
    null: String,
}
impl DisplayIndex for PseDisplay<'_> {
    fn write(&self, index: usize, output: &mut dyn Write) -> FormatResult {
        if index >= self.array.len() {
            return Err(invalid("formatter index outside array").into());
        }
        if self.array.is_null(index) {
            output.write_str(&self.null)?;
            return Ok(());
        }
        output.write_str(&render(self.array, &self.field, index)?)?;
        Ok(())
    }
}
fn invalid(reason: &str) -> ArrowError {
    ArrowError::InvalidArgumentError(reason.to_owned())
}
fn cast<T: 'static>(array: &dyn Array) -> Result<&T, ArrowError> {
    array
        .as_any()
        .downcast_ref()
        .ok_or_else(|| invalid("formatter array implementation differs from storage"))
}
fn plain(array: &dyn Array, index: usize) -> Result<String, ArrowError> {
    ArrayFormatter::try_new(array, &FormatOptions::default())?
        .value(index)
        .try_to_string()
}
fn identity(array: &dyn Array, index: usize) -> Result<String, ArrowError> {
    let bytes = cast::<FixedSizeBinaryArray>(array)?.value(index);
    let id = SemanticId::from_bytes(
        bytes
            .try_into()
            .map_err(|_| invalid("identity width differs"))?,
    );
    Ok(format!("id:{}", id.to_hex()))
}
fn render(array: &dyn Array, field: &Field, index: usize) -> Result<String, ArrowError> {
    let name = field
        .metadata()
        .get(pse_schema::arrow::KEY_EXTENSION_NAME)
        .ok_or_else(|| invalid("extension name missing"))?;
    match name.as_str() {
        "pse.semantic_id" => identity(array, index),
        "pse.content_hash" => {
            let bytes = cast::<FixedSizeBinaryArray>(array)?.value(index);
            Ok(ContentHash::from_bytes(
                bytes
                    .try_into()
                    .map_err(|_| invalid("hash width differs"))?,
            )
            .to_prefixed())
        }
        "pse.quantity_value" => {
            let value = cast::<StructArray>(array)?;
            Ok(format!(
                "{} [unit:{}; quantity:{}]",
                plain(value.column(0).as_ref(), index)?,
                identity(value.column(2).as_ref(), index)?,
                identity(value.column(1).as_ref(), index)?
            ))
        }
        "pse.bound" => {
            let value = cast::<StructArray>(array)?;
            let kind = plain(value.column(0).as_ref(), index)?;
            match kind.as_str() {
                "finite" => Ok(format!(
                    "finite({})",
                    plain(value.column(1).as_ref(), index)?
                )),
                "unbounded" => Ok("unbounded".to_owned()),
                _ => Err(invalid("unknown bound kind")),
            }
        }
        "pse.dimension_vector" => dimension(array, index),
        "pse.index_tuple" => {
            let values = cast::<ListArray>(array)?.value(index);
            let ids = (0..values.len())
                .map(|row| identity(values.as_ref(), row))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(format!("[{}]", ids.join(", ")))
        }
        "pse.source_span" => {
            let value = cast::<StructArray>(array)?;
            Ok(format!(
                "{}:{}..{}",
                identity(value.column(0).as_ref(), index)?,
                plain(value.column(1).as_ref(), index)?,
                plain(value.column(2).as_ref(), index)?
            ))
        }
        "pse.ordinal_ref" => {
            let metadata = super::ExtMetadata::parse(
                &pse_schema::model::EXTENSION_TYPES[6],
                &field.metadata()[pse_schema::arrow::KEY_EXTENSION_METADATA],
            )?;
            let target = metadata
                .target_relation_id
                .ok_or_else(|| invalid("ordinal target missing"))?;
            Ok(format!(
                "{}#{}",
                target.to_hex(),
                cast::<Int64Array>(array)?.value(index)
            ))
        }
        "pse.enum" | "pse.expr_dsl" | "pse.target_path" => plain(array, index),
        _ => Err(invalid("extension is not registered")),
    }
}
fn dimension(array: &dyn Array, index: usize) -> Result<String, ArrowError> {
    let values = cast::<FixedSizeListArray>(array)?.value(index);
    let pairs = cast::<StructArray>(values.as_ref())?;
    let nums = cast::<Int16Array>(pairs.column(0).as_ref())?;
    let dens = cast::<Int16Array>(pairs.column(1).as_ref())?;
    let axes = [
        "length",
        "mass",
        "time",
        "temperature",
        "amount",
        "current",
        "luminous_intensity",
        "currency",
    ];
    let mut terms = Vec::new();
    for (row, axis) in axes.iter().enumerate() {
        if nums.value(row) != 0 {
            terms.push(format!("{axis}^{}/{}", nums.value(row), dens.value(row)));
        }
    }
    Ok(if terms.is_empty() {
        "dimensionless".to_owned()
    } else {
        terms.join(" ")
    })
}
