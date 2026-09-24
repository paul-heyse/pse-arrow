// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact declared value equality inside native plans. Arrow's equality compares
//! float bytes, nested validity and dictionary values, not dictionary key codes.

use datafusion::{
    arrow::{
        array::BooleanArray,
        datatypes::{DataType, FieldRef},
    },
    common::{DataFusionError, Result},
    logical_expr::{
        ColumnarValue, Expr, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDF, ScalarUDFImpl,
        Signature, Volatility,
    },
};
use std::sync::{Arc, LazyLock};

static FUNCTION: LazyLock<Arc<ScalarUDF>> = LazyLock::new(|| {
    Arc::new(ScalarUDF::from(Exact {
        signature: Signature::any(2, Volatility::Immutable),
    }))
});

pub(super) fn function() -> Arc<ScalarUDF> {
    Arc::clone(&FUNCTION)
}
pub(crate) fn equal(left: Expr, right: Expr) -> Expr {
    FUNCTION.call(vec![left, right])
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Exact {
    signature: Signature,
}

impl ScalarUDFImpl for Exact {
    fn name(&self) -> &'static str {
        "pse_exact_value_equal"
    }
    fn signature(&self) -> &Signature {
        &self.signature
    }
    fn return_type(&self, arguments: &[DataType]) -> Result<DataType> {
        let [left, right] = arguments else {
            return Err(invalid("requires two values"));
        };
        if left != right {
            return Err(invalid("requires exactly equal storage types"));
        }
        Ok(DataType::Boolean)
    }
    fn return_field_from_args(&self, args: ReturnFieldArgs<'_>) -> Result<FieldRef> {
        let [left, right] = args.arg_fields else {
            return Err(invalid("requires two fields"));
        };
        if left.data_type() != right.data_type() || left.metadata() != right.metadata() {
            return Err(invalid("requires the same complete declared value type"));
        }
        let registry = pse_engine::validation::registry().map_err(pse_columnar::external)?;
        let column = pse_schema::model::FieldContract::payload(
            "equal",
            pse_schema::model::FieldContract::native(DataType::Boolean),
            "Exact declared value equality, including two nulls.",
        );
        pse_schema::arrow::field_for(registry, &column)
            .map(Arc::new)
            .map_err(pse_columnar::external)
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        let [left, right] = args.args.as_slice() else {
            return Err(invalid("requires two inputs"));
        };
        let left = left.clone().into_array(args.number_rows)?;
        let right = right.clone().into_array(args.number_rows)?;
        if left.data_type() != right.data_type() {
            return Err(invalid("runtime types differ"));
        }
        let result = BooleanArray::from(
            ((0..args.number_rows)
                .map(|row| left.slice(row, 1).to_data() == right.slice(row, 1).to_data()))
            .collect::<Vec<_>>(),
        );
        Ok(ColumnarValue::Array(Arc::new(result)))
    }
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(format!("exact before-image equality {reason}"))
}
