// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Array selection carries the actual input child field through the native kernel.
use datafusion::{
    arrow::datatypes::{DataType, Field, FieldRef},
    common::{DataFusionError, Result},
    functions_nested::extract::array_element_udf,
    logical_expr::{
        ColumnarValue, Expr, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDF, ScalarUDFImpl,
        Signature,
    },
};
use std::sync::{Arc, LazyLock};

static ELEMENT: LazyLock<Arc<ScalarUDF>> = LazyLock::new(|| {
    Arc::new(ScalarUDF::from(Element {
        native: array_element_udf(),
    }))
});
pub(super) fn function() -> Arc<ScalarUDF> {
    Arc::clone(&ELEMENT)
}

/// Select an element using DataFusion's one-based/negative-index and null rules.
/// The result retains the list child's actual field metadata.
pub fn array_element(array: Expr, index: Expr) -> Expr {
    ELEMENT.call(vec![array, index])
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Element {
    native: Arc<ScalarUDF>,
}
impl ScalarUDFImpl for Element {
    fn name(&self) -> &'static str {
        "pse_array_element"
    }
    fn signature(&self) -> &Signature {
        self.native.signature()
    }
    fn return_type(&self, args: &[DataType]) -> Result<DataType> {
        self.native.return_type(args)
    }
    fn return_field_from_args(&self, args: ReturnFieldArgs<'_>) -> Result<FieldRef> {
        let [array, _] = args.arg_fields else {
            return Err(DataFusionError::Plan(
                "pse_array_element requires an array and index".into(),
            ));
        };
        // IndexTuple declares ordered semantic identities in its extension
        // contract; the canonical storage child is a bare 16-byte value.
        if array
            .metadata()
            .get(pse_schema::arrow::KEY_EXTENSION_NAME)
            .is_some_and(|name| name == "pse.index_tuple")
        {
            let expected = super::index_tuple::output_field(array.is_nullable())?;
            super::super::output::check_field_output(array, &expected)?;
            let mut element = pse_schema::model::FieldContract::payload(
                "pse_array_element",
                pse_schema::model::FieldContract::id(),
                "Actual index-tuple member identity.",
            );
            element = element.optional();
            let registry = pse_schema::registry()
                .map_err(|error| DataFusionError::External(Box::new(error)))?;
            return pse_schema::arrow::field_for(registry, &element)
                .map(Arc::new)
                .map_err(|error| DataFusionError::External(Box::new(error)));
        }
        match array.data_type() {
            DataType::List(child) | DataType::LargeList(child) => Ok(Arc::new(
                child
                    .as_ref()
                    .clone()
                    .with_name(self.name())
                    .with_nullable(true),
            )),
            DataType::Null => Ok(Arc::new(Field::new(self.name(), DataType::Null, true))),
            _ => Err(DataFusionError::Plan(
                "pse_array_element requires a native List or LargeList".into(),
            )),
        }
    }
    fn invoke_with_args(&self, mut args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        // Keep the native kernel on its array path. DF 55.1's generic scalar
        // extraction rebuilds nested list fields without their child metadata.
        args.args = args
            .args
            .into_iter()
            .map(|value| value.into_array(args.number_rows).map(ColumnarValue::Array))
            .collect::<Result<_>>()?;
        self.native.inner().invoke_with_args(args)
    }
}
