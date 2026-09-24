// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Versioned key tokens over the shared native semantic framing. Keys remain
//! scoped by their relation and selected input; tokens are not membership proofs.

use crate::native::{
    arrow::datatypes::{DataType, FieldRef},
    common::{DataFusionError, Result},
    logical_expr::{
        ColumnarValue, Expr, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDF, ScalarUDFImpl,
        Signature, Volatility, lit,
    },
};
use pse_schema::model::FieldContract;
use std::sync::{Arc, LazyLock, OnceLock};

const NAME: &str = "pse_row_key";
static OUTPUT_FIELD: OnceLock<FieldRef> = OnceLock::new();

static FUNCTION: LazyLock<Arc<ScalarUDF>> = LazyLock::new(|| {
    Arc::new(ScalarUDF::from(Key {
        signature: Signature::variadic_any(Volatility::Immutable),
    }))
});

/// Native row-token expression adapter.
pub fn function() -> Arc<ScalarUDF> {
    Arc::clone(&FUNCTION)
}

/// Compute a typed token from ordered declared primary-key names and actual values.
/// Payload columns and batch positions do not participate. The selected relation and
/// revision remain explicit in the surrounding reference, never inferred from a token.
pub fn key(relation: pse_ids::SemanticId, columns: Vec<(&str, Expr)>) -> Expr {
    FUNCTION.call(
        std::iter::once(lit(crate::native::common::ScalarValue::FixedSizeBinary(
            16,
            Some(relation.as_bytes().to_vec()),
        )))
        .chain(
            columns
                .into_iter()
                .flat_map(|(name, value)| [lit(name), value]),
        )
        .collect(),
    )
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Key {
    signature: Signature,
}

impl ScalarUDFImpl for Key {
    fn name(&self) -> &'static str {
        NAME
    }
    fn signature(&self) -> &Signature {
        &self.signature
    }
    fn return_type(&self, _arguments: &[DataType]) -> Result<DataType> {
        Ok(DataType::FixedSizeBinary(32))
    }
    fn return_field_from_args(&self, args: ReturnFieldArgs<'_>) -> Result<FieldRef> {
        if args.arg_fields.is_empty()
            || !((args.arg_fields.len() - 1).is_multiple_of(2))
            || args.arg_fields[0].data_type() != &DataType::FixedSizeBinary(16)
        {
            return Err(invalid("ordered name/value pairs required"));
        }
        if !matches!(args.scalar_arguments.first().copied().flatten(),
            Some(crate::native::common::ScalarValue::FixedSizeBinary(16, Some(scope))) if scope.len() == 16)
        {
            return Err(invalid("relation scope must be a literal identity"));
        }
        for (i, pair) in args.arg_fields[1..].as_chunks::<2>().0.iter().enumerate() {
            if args
                .scalar_arguments
                .get(i * 2 + 1)
                .copied()
                .flatten()
                .and_then(|v| v.try_as_str().flatten())
                .is_none()
            {
                return Err(invalid("key names must be non-null text literals"));
            }
            pse_columnar::native_value::admit_type(pair[1].data_type())
                .map_err(pse_columnar::external)?;
        }
        output_field()
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        let Some(ColumnarValue::Scalar(crate::native::common::ScalarValue::FixedSizeBinary(
            16,
            Some(scope),
        ))) = args.args.first()
        else {
            return Err(invalid("relation scope must be a literal identity"));
        };
        let scope = pse_ids::SemanticId::from_bytes(
            scope
                .as_slice()
                .try_into()
                .map_err(|_| invalid("relation scope width differs"))?,
        );
        let mut columns = Vec::new();
        for (pair, declared) in args.args[1..]
            .as_chunks::<2>()
            .0
            .iter()
            .zip(args.arg_fields[1..].as_chunks::<2>().0)
        {
            let ColumnarValue::Scalar(name) = &pair[0] else {
                return Err(invalid("key name is not literal"));
            };
            let name = name
                .try_as_str()
                .flatten()
                .ok_or_else(|| invalid("key name is not text"))?;
            columns.push((
                name,
                Arc::clone(&declared[1]),
                pair[1].clone().into_array(args.number_rows)?,
            ));
        }
        let output = pse_columnar::row_token::tokens(scope, &columns, args.number_rows)
            .map_err(pse_columnar::external)?;
        Ok(ColumnarValue::Array(Arc::new(output)))
    }
}
fn output_field() -> Result<FieldRef> {
    if let Some(field) = OUTPUT_FIELD.get() {
        return Ok(Arc::clone(field));
    }
    // This intrinsic output contract has no caller, argument or session state.
    // Keep its exact generated declaration without serializing it at every native
    // expression-field visit. Failed initialization is never cached.
    let field = Arc::new(
        pse_schema::arrow::field_for(
            pse_schema::registry().map_err(external)?,
            &FieldContract::row_key().with_name(NAME),
        )
        .map_err(external)?,
    );
    let _ = OUTPUT_FIELD.set(Arc::clone(&field));
    Ok(field)
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(format!("row key: {reason}"))
}
fn external(error: impl Into<DataFusionError>) -> DataFusionError {
    error.into()
}
