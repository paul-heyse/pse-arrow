// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Immutable Arrow codecs used inside native plans for diagnostic and support labels.
//! These functions preserve exact values. Their identifiers never establish validity,
//! equality, uniqueness, truth, membership or dependency coverage.

pub(crate) mod checked_value;
pub(super) mod element;
mod encode;
mod fields;
mod key;
pub use key::key;
mod nonnull;
pub(crate) use nonnull::nullable;
pub use nonnull::{refine_filtered_fields, require_nonnull};
mod index_tuple;
mod list_concat;
mod list_field;
mod preserve_field;
mod retain_metadata;
pub(super) mod structure;
pub(crate) use preserve_field::{
    materialize as materialize_nested_fields, nested_metadata, proven_native_layout,
};
pub(crate) use retain_metadata::{retain as retain_metadata, union as union_fields};

use datafusion::arrow::array::{
    Array, FixedSizeBinaryArray, StringArray,
    builder::{FixedSizeBinaryBuilder, ListBuilder, StringBuilder},
};
use datafusion::arrow::datatypes::{DataType, FieldRef};
use datafusion::common::{DataFusionError, Result};
use datafusion::execution::session_state::SessionStateBuilder;
use datafusion::logical_expr::{
    ColumnarValue, Expr, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDF, ScalarUDFImpl, Signature,
    TypeSignature, Volatility,
};
use pse_ids::SemanticId;
use pse_schema::model::FieldContract;
use std::sync::{Arc, LazyLock};

static LITERAL: LazyLock<Arc<ScalarUDF>> =
    LazyLock::new(|| Arc::new(ScalarUDF::from(Codec::new(Kind::Literal))));
static NAMED_ID: LazyLock<Arc<ScalarUDF>> =
    LazyLock::new(|| Arc::new(ScalarUDF::from(Codec::new(Kind::NamedId))));
static ID_LIST: LazyLock<Arc<ScalarUDF>> =
    LazyLock::new(|| Arc::new(ScalarUDF::from(Codec::new(Kind::IdList))));

/// Adds the actual platform codec objects before sealing a native session assembly.
/// Other configured functions remain present.
pub fn register(
    mut builder: SessionStateBuilder,
    reserver: Arc<dyn pse_ids::MemoryReserver>,
) -> SessionStateBuilder {
    builder
        .scalar_functions()
        .get_or_insert_with(Vec::new)
        .extend([
            key::function(),
            Arc::clone(&LITERAL),
            Arc::clone(&NAMED_ID),
            Arc::clone(&ID_LIST),
            nonnull::function(),
            nonnull::nullable_function(),
            list_field::function(Arc::clone(&reserver)),
            list_concat::function(Arc::clone(&reserver)),
            preserve_field::function(Arc::clone(&reserver)),
            retain_metadata::function(),
            index_tuple::function(reserver),
        ]);
    builder
}

/// Encodes an actual typed value without `Cell` reconstruction or float normalization.
pub fn literal(value: Expr) -> Expr {
    LITERAL.call(vec![value])
}

/// Assigns an identity to an explicit namespace and exact text or typed row-key payload.
/// Consumers must compare actual fact/support columns rather than these labels.
pub fn named_id(namespace: Expr, payload: Expr) -> Expr {
    NAMED_ID.call(vec![namespace, payload])
}

/// Collects the present explicitly selected semantic identities in argument order.
/// Null inputs are absent references, not list members; no entity existence is claimed.
pub fn id_list(values: Vec<Expr>) -> Expr {
    ID_LIST.call(values)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Kind {
    Literal,
    NamedId,
    IdList,
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Codec {
    kind: Kind,
    signature: Signature,
}

impl Codec {
    fn new(kind: Kind) -> Self {
        let signature = match kind {
            Kind::IdList => Signature::one_of(
                vec![TypeSignature::Nullary, TypeSignature::VariadicAny],
                Volatility::Immutable,
            ),
            Kind::Literal => Signature::any(1, Volatility::Immutable),
            Kind::NamedId => Signature::one_of(
                vec![
                    TypeSignature::Exact(vec![DataType::FixedSizeBinary(16), DataType::Utf8]),
                    TypeSignature::Exact(vec![
                        DataType::FixedSizeBinary(16),
                        DataType::FixedSizeBinary(32),
                    ]),
                ],
                Volatility::Immutable,
            ),
        };
        Self { kind, signature }
    }
}

impl ScalarUDFImpl for Codec {
    fn name(&self) -> &str {
        match self.kind {
            Kind::Literal => "pse_literal",
            Kind::NamedId => "pse_named_id",
            Kind::IdList => "pse_id_list",
        }
    }
    fn signature(&self) -> &Signature {
        &self.signature
    }
    fn return_type(&self, _arguments: &[DataType]) -> Result<DataType> {
        Ok(match self.kind {
            Kind::NamedId => DataType::FixedSizeBinary(16),
            Kind::IdList => FieldContract::list(FieldContract::id()).data_type(),
            Kind::Literal => DataType::Utf8,
        })
    }
    fn return_field_from_args(&self, args: ReturnFieldArgs<'_>) -> Result<FieldRef> {
        match self.kind {
            Kind::Literal => {
                let field = args
                    .arg_fields
                    .first()
                    .ok_or_else(|| invalid("literal requires one value"))?;
                encode::admit_type(field.data_type())?;
            }
            Kind::NamedId => {
                if args.arg_fields.len() != 2
                    || args.arg_fields[0].data_type() != &DataType::FixedSizeBinary(16)
                    || !matches!(
                        args.arg_fields[1].data_type(),
                        DataType::Utf8 | DataType::FixedSizeBinary(32)
                    )
                {
                    return Err(invalid(
                        "named identity requires a 16-byte namespace and text or typed-key payload",
                    ));
                }
            }
            Kind::IdList => {
                for field in args.arg_fields {
                    if field.data_type() != &DataType::FixedSizeBinary(16)
                        || field
                            .metadata()
                            .get(pse_schema::arrow::KEY_EXTENSION_NAME)
                            .is_none_or(|value| value != "pse.semantic_id")
                    {
                        return Err(invalid(
                            "identity collection requires actual semantic-ID arguments",
                        ));
                    }
                }
            }
        }
        let nullable =
            self.kind == Kind::NamedId && args.arg_fields.iter().any(|field| field.is_nullable());
        fields::output(self.kind, nullable)
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        let arrays = args
            .args
            .iter()
            .map(|value| value.clone().into_array(args.number_rows))
            .collect::<Result<Vec<_>>>()?;
        if self.kind == Kind::IdList {
            return collect_ids(&args, &arrays);
        }
        if self.kind == Kind::NamedId {
            let namespace = arrays
                .first()
                .and_then(|array| array.as_any().downcast_ref::<FixedSizeBinaryArray>())
                .ok_or_else(|| invalid("named identity namespace layout differs"))?;
            let payload = arrays
                .get(1)
                .ok_or_else(|| invalid("named identity payload absent"))?;
            let mut output = FixedSizeBinaryBuilder::with_capacity(args.number_rows, 16);
            for row in 0..args.number_rows {
                if namespace.is_null(row) || payload.is_null(row) {
                    output.append_null();
                    continue;
                }
                let bytes = namespace
                    .value(row)
                    .try_into()
                    .map_err(|_| invalid("identity namespace width differs"))?;
                let payload = if let Some(text) = payload.as_any().downcast_ref::<StringArray>() {
                    text.value(row).as_bytes()
                } else if let Some(key) = payload.as_any().downcast_ref::<FixedSizeBinaryArray>() {
                    key.value(row)
                } else {
                    return Err(invalid("named identity payload layout differs"));
                };
                output.append_value(
                    pse_ids::derive_id(
                        pse_ids::derive::context::NAMED,
                        &[SemanticId::from_bytes(bytes).as_bytes(), payload],
                    )
                    .as_bytes(),
                )?;
            }
            return Ok(ColumnarValue::Array(Arc::new(output.finish())));
        }
        let mut output = StringBuilder::with_capacity(args.number_rows, 0);
        let mut text = String::new();
        for row in 0..args.number_rows {
            text.clear();
            let array = arrays
                .first()
                .ok_or_else(|| invalid("literal argument absent"))?;
            encode::value(array.as_ref(), &args.arg_fields[0], row, &mut text)?;
            let length = output
                .values_slice()
                .len()
                .checked_add(text.len())
                .ok_or_else(|| invalid("diagnostic text capacity overflow"))?;
            i32::try_from(length)
                .map_err(|_| invalid("diagnostic text exceeds Arrow Utf8 offset capacity"))?;
            output.append_value(&text);
        }
        Ok(ColumnarValue::Array(Arc::new(output.finish())))
    }
}

fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(format!("PSE diagnostic codec: {reason}"))
}

fn collect_ids(
    args: &ScalarFunctionArgs,
    arrays: &[datafusion::arrow::array::ArrayRef],
) -> Result<ColumnarValue> {
    let DataType::List(field) = args.return_type() else {
        return Err(invalid("identity collection output is not a list"));
    };
    let capacity = args
        .number_rows
        .checked_mul(arrays.len())
        .ok_or_else(|| invalid("identity collection capacity overflow"))?;
    i32::try_from(capacity)
        .map_err(|_| invalid("identity collection exceeds Arrow list offsets"))?;
    let mut output = ListBuilder::with_capacity(
        FixedSizeBinaryBuilder::with_capacity(capacity, 16),
        args.number_rows,
    )
    .with_field(Arc::clone(field));
    let inputs = arrays
        .iter()
        .map(|array| {
            array
                .as_any()
                .downcast_ref::<FixedSizeBinaryArray>()
                .ok_or_else(|| invalid("identity collection input layout differs"))
        })
        .collect::<Result<Vec<_>>>()?;
    for row in 0..args.number_rows {
        for input in &inputs {
            if input.is_valid(row) {
                output.values().append_value(input.value(row))?;
            }
        }
        output.append(true);
    }
    Ok(ColumnarValue::Array(Arc::new(output.finish())))
}
