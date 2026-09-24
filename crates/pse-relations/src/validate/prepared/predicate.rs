// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Logical visibility for the same prepared native validator used by raw admission.
use super::PreparedLocalContract;
use crate::native::{
    arrow::{array::RecordBatch, datatypes::DataType},
    common::Result,
    logical_expr::{
        ColumnarValue, Expr, ScalarFunctionArgs, ScalarUDF, ScalarUDFImpl, Signature, Volatility,
    },
};
use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

pub(super) fn expression(prepared: Arc<PreparedLocalContract>, arguments: Vec<Expr>) -> Expr {
    let signature = Signature::exact(
        prepared
            .schema()
            .fields()
            .iter()
            .map(|field| field.data_type().clone())
            .collect(),
        Volatility::Immutable,
    );
    ScalarUDF::from(PreparedPredicate {
        prepared,
        signature,
    })
    .call(arguments)
}
#[derive(Debug)]
struct PreparedPredicate {
    prepared: Arc<PreparedLocalContract>,
    signature: Signature,
}
impl PartialEq for PreparedPredicate {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.prepared, &other.prepared)
    }
}
impl Eq for PreparedPredicate {}
impl Hash for PreparedPredicate {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.prepared).hash(state);
    }
}
impl ScalarUDFImpl for PreparedPredicate {
    fn name(&self) -> &'static str {
        "pse_local_contract"
    }
    fn signature(&self) -> &Signature {
        &self.signature
    }
    fn return_type(&self, _: &[DataType]) -> Result<DataType> {
        Ok(DataType::Boolean)
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        let columns = args
            .args
            .into_iter()
            .map(|value| value.to_array(args.number_rows))
            .collect::<Result<Vec<_>>>()?;
        let batch = RecordBatch::try_new(Arc::clone(self.prepared.schema()), columns)?;
        let report = self
            .prepared
            .evaluate(&batch, 0, &pse_columnar::CancellationToken::new())
            .map_err(pse_columnar::external)?;
        Ok(ColumnarValue::Array(Arc::new(report.valid_rows)))
    }
}
