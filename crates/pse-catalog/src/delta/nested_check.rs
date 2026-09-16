// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Adapter for Delta's pinned CHECK parser, which cannot parse SQL lambdas.
//! The value predicate and its execution are entirely native DataFusion expressions.
//! Only affected collection fields cross this adapter; no row/Cell validator exists here.

use datafusion::{
    arrow::{
        array::RecordBatch,
        compute::{CastOptions, cast_with_options},
        datatypes::{DataType, Field, Schema, SchemaRef},
    },
    common::{DFSchema, DataFusionError, Result},
    execution::session_state::SessionState,
    logical_expr::{
        ColumnarValue, Expr, ScalarFunctionArgs, ScalarUDF, ScalarUDFImpl, Signature, Volatility,
    },
    physical_expr::PhysicalExpr,
};
use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

#[derive(Debug, Clone)]
pub(super) struct NestedCheck {
    name: String,
    schema: SchemaRef,
    predicate: Expr,
    signature: Signature,
    physical: Option<Arc<dyn PhysicalExpr>>,
}
impl NestedCheck {
    pub(super) fn new(name: String, field: Field, predicate: Expr) -> Result<Self> {
        let schema = Arc::new(Schema::new(vec![field]));
        let predicate = predicate
            .resolve_lambda_variables(&DFSchema::try_from(schema.as_ref().clone())?)?
            .data;
        Ok(Self {
            name,
            schema,
            predicate,
            signature: Signature::user_defined(Volatility::Immutable),
            physical: None,
        })
    }
    pub(super) fn function(&self) -> ScalarUDF {
        ScalarUDF::from(self.clone())
    }
    pub(super) fn bind(&self, state: &SessionState) -> Result<ScalarUDF> {
        let mut bound = self.clone();
        bound.physical = Some(state.create_physical_expr(
            self.predicate.clone(),
            &DFSchema::try_from(self.schema.as_ref().clone())?,
        )?);
        Ok(bound.function())
    }
}
impl PartialEq for NestedCheck {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.schema == other.schema && self.predicate == other.predicate
    }
}
impl Eq for NestedCheck {}
impl Hash for NestedCheck {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.schema.hash(state);
        self.predicate.hash(state);
    }
}
impl ScalarUDFImpl for NestedCheck {
    fn name(&self) -> &str {
        &self.name
    }
    fn signature(&self) -> &Signature {
        &self.signature
    }
    fn return_type(&self, _: &[DataType]) -> Result<DataType> {
        Ok(DataType::Boolean)
    }
    fn coerce_types(&self, args: &[DataType]) -> Result<Vec<DataType>> {
        let [data_type] = args else {
            return Err(DataFusionError::Plan(
                "nested CHECK requires one field".into(),
            ));
        };
        let field = self
            .schema
            .field(0)
            .clone()
            .with_data_type(data_type.clone());
        pse_schema::field_contract::delta_scan_schema(&Schema::new(vec![field]), &self.schema)
            .map_err(|error| DataFusionError::External(Box::new(error)))?;
        Ok(args.to_vec())
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        let [value] = args.args.as_slice() else {
            return Err(DataFusionError::Execution(
                "nested CHECK requires one field".into(),
            ));
        };
        let physical = self.physical.as_ref().ok_or_else(|| {
            DataFusionError::Execution("nested CHECK requires native expression binding".into())
        })?;
        let array = cast_with_options(
            &value.to_array(args.number_rows)?,
            self.schema.field(0).data_type(),
            &CastOptions {
                safe: false,
                ..CastOptions::default()
            },
        )?;
        physical.evaluate(&RecordBatch::try_new(
            Arc::clone(&self.schema),
            vec![array],
        )?)
    }
}

pub(super) fn contains_collection(field: &Field) -> bool {
    contains_collection_type(field.data_type())
}
fn contains_collection_type(data_type: &DataType) -> bool {
    match data_type {
        DataType::List(_)
        | DataType::LargeList(_)
        | DataType::ListView(_)
        | DataType::LargeListView(_)
        | DataType::FixedSizeList(..)
        | DataType::Map(..) => true,
        DataType::Struct(fields) => fields.iter().any(|field| contains_collection(field)),
        DataType::Dictionary(_, values) => contains_collection_type(values),
        _ => false,
    }
}
