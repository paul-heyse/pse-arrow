// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Array selection carries the actual input child field through the native kernel.
use datafusion::{
    arrow::datatypes::{DataType, Field, FieldRef},
    common::{DataFusionError, Result},
    logical_expr::{
        ColumnarValue, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDF, ScalarUDFImpl, Signature,
    },
};
use std::sync::Arc;

pub(in crate::session) fn function(native: Arc<ScalarUDF>) -> Arc<ScalarUDF> {
    Arc::new(ScalarUDF::from(Element {
        origin: native.clone(),
        native,
    }))
}

crate::session::native_hooks::native_identity!(Element);

#[derive(Debug)]
struct Element {
    origin: Arc<ScalarUDF>,
    native: Arc<ScalarUDF>,
}
impl ScalarUDFImpl for Element {
    crate::session::native_hooks::scalar_hooks!();
    fn with_updated_config(
        &self,
        config: &datafusion::common::config::ConfigOptions,
    ) -> Option<ScalarUDF> {
        self.native
            .inner()
            .with_updated_config(config)
            .map(|native| {
                ScalarUDF::from(Self {
                    origin: self.origin.clone(),
                    native: Arc::new(native),
                })
            })
    }

    fn name(&self) -> &'static str {
        "array_element"
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
                "array_element requires an array and index".into(),
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
                "array_element",
                pse_schema::model::FieldContract::id(),
                "Actual index-tuple member identity.",
            );
            element = element.optional();
            let registry = crate::validation::registry().map_err(pse_columnar::external)?;
            return pse_schema::arrow::field_for(registry, &element)
                .map(Arc::new)
                .map_err(pse_columnar::external);
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
                "array_element requires a native List or LargeList".into(),
            )),
        }
    }
    fn invoke_with_args(&self, mut args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        // Only nested scalar extraction has a pinned field-loss gap. Ordinary
        // scalar and mixed array inputs retain the native dispatch fast paths.
        if super::nested_metadata(args.return_field.data_type())
            && args
                .args
                .iter()
                .all(|arg| matches!(arg, ColumnarValue::Scalar(_)))
        {
            args.number_rows = 1;
            args.args = args
                .args
                .into_iter()
                .map(|value| value.into_array(1).map(ColumnarValue::Array))
                .collect::<Result<_>>()?;
            let value = self.native.invoke_with_args(args)?.into_array(1)?;
            return pse_schema::literal::scalar_from_array(&value)
                .map(ColumnarValue::Scalar)
                .map_err(pse_columnar::external);
        }
        self.native.invoke_with_args(args)
    }
}

pub(in crate::session) fn native(function: &ScalarUDF) -> Option<&Arc<ScalarUDF>> {
    function
        .inner()
        .downcast_ref::<Element>()
        .map(|value| &value.origin)
}
