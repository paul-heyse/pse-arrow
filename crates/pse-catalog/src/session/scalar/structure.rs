// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Field-preserving adapter for the pinned native named-struct constructor.
//! Native argument validation and Arrow construction remain in DataFusion.

use datafusion::{
    arrow::datatypes::{DataType, FieldRef},
    common::{DataFusionError, Result, ScalarValue},
    logical_expr::{
        ColumnarValue, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDF, ScalarUDFImpl, Signature,
        StructFieldMapping,
    },
};
use std::sync::Arc;

pub(in crate::session) fn function(native: Arc<ScalarUDF>) -> Arc<ScalarUDF> {
    Arc::new(ScalarUDF::from(FieldStruct { native }))
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct FieldStruct {
    native: Arc<ScalarUDF>,
}

impl ScalarUDFImpl for FieldStruct {
    fn name(&self) -> &'static str {
        "named_struct"
    }
    fn signature(&self) -> &Signature {
        self.native.signature()
    }
    fn return_type(&self, arguments: &[DataType]) -> Result<DataType> {
        self.native.return_type(arguments)
    }
    fn return_field_from_args(&self, args: ReturnFieldArgs<'_>) -> Result<FieldRef> {
        let native = self.native.return_field_from_args(ReturnFieldArgs {
            arg_fields: args.arg_fields,
            scalar_arguments: args.scalar_arguments,
        })?;
        let DataType::Struct(fields) = native.data_type() else {
            return Err(DataFusionError::Internal(
                "native named struct returned a non-struct field".to_owned(),
            ));
        };
        let fields = fields
            .iter()
            .zip(args.arg_fields.iter().skip(1).step_by(2))
            .map(|(named, actual)| Arc::new(actual.as_ref().clone().with_name(named.name())))
            .collect::<Vec<_>>();
        // The pinned implementation returns StructArray(..., None). Nullable
        // arguments are represented solely by their own child validity bitmaps.
        Ok(Arc::new(
            native
                .as_ref()
                .clone()
                .with_data_type(DataType::Struct(fields.into()))
                .with_nullable(false),
        ))
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        self.native.invoke_with_args(args)
    }
    fn struct_field_mapping(
        &self,
        literal_args: &[Option<ScalarValue>],
    ) -> Option<StructFieldMapping> {
        // Retaining field meaning does not change the native constructor's
        // argument correspondence or the optimizer's ordering proof.
        self.native.struct_field_mapping(literal_args)
    }
}

pub(in crate::session) fn native(function: &ScalarUDF) -> Option<&Arc<ScalarUDF>> {
    function
        .inner()
        .downcast_ref::<FieldStruct>()
        .map(|value| &value.native)
}
