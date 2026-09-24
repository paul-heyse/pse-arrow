// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Construct a local ordered index value from actual semantic identities. Domain
//! membership and the existence of referenced entities remain separate obligations.

use datafusion::{
    arrow::datatypes::{DataType, FieldRef},
    common::{DataFusionError, Result},
    logical_expr::{
        ColumnarValue, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDF, ScalarUDFImpl, Signature,
        Volatility,
    },
};
use pse_columnar::MemoryPool;
use pse_schema::model::{ExtensionUse, FieldContract};
use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

pub(super) fn function(pool: Arc<dyn MemoryPool>) -> Arc<ScalarUDF> {
    Arc::new(ScalarUDF::from(IndexTuple {
        signature: Signature::any(1, Volatility::Immutable),
        pool,
    }))
}

#[derive(Debug)]
struct IndexTuple {
    signature: Signature,
    pool: Arc<dyn MemoryPool>,
}
impl PartialEq for IndexTuple {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.pool, &other.pool)
    }
}
impl Eq for IndexTuple {}
impl Hash for IndexTuple {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(Arc::as_ptr(&self.pool), state);
    }
}
impl ScalarUDFImpl for IndexTuple {
    fn name(&self) -> &'static str {
        "pse_index_tuple"
    }
    fn signature(&self) -> &Signature {
        &self.signature
    }
    fn return_type(&self, _args: &[DataType]) -> Result<DataType> {
        Ok(FieldContract::extended(ExtensionUse::IndexTuple).data_type())
    }
    fn return_field_from_args(&self, args: ReturnFieldArgs<'_>) -> Result<FieldRef> {
        let [source] = args.arg_fields else {
            return Err(invalid("expected one ordered semantic-identity list"));
        };
        let DataType::List(child) = source.data_type() else {
            return Err(invalid("input must be an ordered List"));
        };
        let registry = crate::validation::registry().map_err(external)?;
        match source
            .metadata()
            .get(pse_schema::arrow::KEY_EXTENSION_NAME)
            .map(String::as_str)
        {
            Some("pse.index_tuple") => {
                let field = output_field(source.is_nullable())?;
                super::super::output::check_field_output(source, &field)?;
            }
            None => {
                let id = pse_schema::arrow::field_for(
                    registry,
                    &FieldContract::payload(
                        "item",
                        FieldContract::id(),
                        "Actual member semantic identity.",
                    ),
                )
                .map_err(external)?
                .with_nullable(true);
                super::super::output::check_field_output(child, &id)?;
            }
            Some(_) => return Err(invalid("input has a different semantic list contract")),
        }
        output_field(source.is_nullable())
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        if args.args.len() != 1 {
            return Err(invalid("expected one input value"));
        }
        super::list_field::cast_field(args, &self.pool, "query:index-tuple")
    }
}

pub(super) fn output_field(nullable: bool) -> Result<FieldRef> {
    let registry = crate::validation::registry().map_err(external)?;
    Ok(Arc::new(
        pse_schema::arrow::field_for(
            registry,
            &FieldContract::payload(
                "pse_index_tuple",
                FieldContract::extended(ExtensionUse::IndexTuple),
                "Ordered actual semantic identities with non-null members.",
            ),
        )
        .map_err(external)?
        .with_nullable(nullable),
    ))
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(format!("pse_index_tuple: {reason}"))
}
fn external(error: impl Into<DataFusionError>) -> DataFusionError {
    error.into()
}

#[cfg(test)]
mod tests;
