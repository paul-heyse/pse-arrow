// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Shared predicate adapter for SQL lambdas and execution/storage type restoration.
//! The value predicate and its execution are entirely native DataFusion expressions.
//! Only fields needing native restoration or lambdas cross this adapter; no `row/serde_json::Value` validator exists here.

use datafusion::{
    arrow::{
        array::RecordBatch,
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
pub(super) struct FieldCheck {
    name: String,
    schema: SchemaRef,
    storage: Arc<Field>,
    predicate: Expr,
    signature: Signature,
    physical: Option<Arc<dyn PhysicalExpr>>,
}
impl FieldCheck {
    pub(super) fn new(name: String, storage: Field, field: Field, predicate: Expr) -> Result<Self> {
        let schema = Arc::new(Schema::new(vec![field]));
        let predicate = predicate
            .resolve_lambda_variables(&DFSchema::try_from(schema.as_ref().clone())?)?
            .data;
        Ok(Self {
            name,
            schema,
            storage: Arc::new(storage),
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
        bound.physical = Some(
            pse_relations::validate::prepare_expression(
                &pse_engine::validation::NativeValidation(state.clone()),
                self.predicate.clone(),
                &DFSchema::try_from(self.schema.as_ref().clone())?,
            )
            .map_err(pse_columnar::external)?,
        );
        Ok(bound.function())
    }
}
impl PartialEq for FieldCheck {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.schema == other.schema
            && self.storage == other.storage
            && self.predicate == other.predicate
    }
}
impl Eq for FieldCheck {}
impl Hash for FieldCheck {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.schema.hash(state);
        self.storage.hash(state);
        self.predicate.hash(state);
    }
}
impl ScalarUDFImpl for FieldCheck {
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
                "field CHECK requires one field".into(),
            ));
        };
        let field = self
            .storage
            .as_ref()
            .clone()
            .with_data_type(data_type.clone());
        pse_schema::field_contract::delta_scan_schema(
            &Schema::new(vec![field]),
            &Schema::new(vec![self.storage.clone()]),
        )
        .map_err(|error| DataFusionError::External(Box::new(error)))?;
        Ok(args.to_vec())
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        let [value] = args.args.as_slice() else {
            return Err(DataFusionError::Execution(
                "field CHECK requires one field".into(),
            ));
        };
        let physical = self.physical.as_ref().ok_or_else(|| {
            DataFusionError::Execution("field CHECK requires native expression binding".into())
        })?;
        let array = super::layout::restore_array(
            &value.to_array(args.number_rows)?,
            self.storage.data_type(),
            &self.schema.fields()[0],
            true,
        )?;
        let result = physical.evaluate(&RecordBatch::try_new(
            Arc::clone(&self.schema),
            vec![array],
        )?)?;
        // An empty batch has no invalid rows. A native CASE can return scalar
        // false on zero rows; Delta's scalar CHECK branch would reject it even
        // though no values were evaluated. Preserve the actual batch cardinality.
        Ok(ColumnarValue::Array(result.into_array(args.number_rows)?))
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

#[cfg(test)]
mod delta_boundary_unit {
    use super::*;
    use datafusion::{
        arrow::array::{BooleanArray, Decimal128Array},
        execution::context::SessionContext,
        logical_expr::{col, lit},
    };

    #[test]
    fn empty_mapped_check_retains_zero_rows_instead_of_a_scalar_failure() {
        let execution = Field::new("value", DataType::UInt64, false);
        let layout = super::super::layout::DurableLayout::new(Arc::new(Schema::new(vec![
            execution.clone(),
        ])))
        .unwrap();
        let registry = pse_schema::RegistryBuilder::new().build().unwrap();
        let predicate = pse_relations::validate::predicates::field_value(
            &registry,
            &execution,
            col("value"),
            0,
        )
        .unwrap();
        let adapter = FieldCheck::new(
            "empty_domain".into(),
            layout.storage_schema().field(0).clone(),
            execution,
            predicate,
        )
        .unwrap();
        let state = SessionContext::new().state();
        let expression = adapter.bind(&state).unwrap().call(vec![col("value")]);
        let physical = state
            .create_physical_expr(
                expression,
                &DFSchema::try_from(layout.storage_schema().as_ref().clone()).unwrap(),
            )
            .unwrap();
        let value = physical
            .evaluate(&RecordBatch::new_empty(layout.storage_schema().clone()))
            .unwrap();
        let ColumnarValue::Array(value) = value else {
            panic!("CHECK must retain batch cardinality")
        };
        assert_eq!(value.len(), 0);
    }

    #[test]
    fn mapped_check_restores_unsigned_execution_values_before_predicate_evaluation() {
        let execution = Field::new("value", DataType::UInt64, false);
        let schema = Arc::new(Schema::new(vec![execution.clone()]));
        let layout = super::super::layout::DurableLayout::new(schema).unwrap();
        let storage = layout.storage_schema();
        let adapter = FieldCheck::new(
            "unsigned_domain".into(),
            storage.field(0).clone(),
            execution,
            col("value").gt_eq(lit(u64::MAX - 1)),
        )
        .unwrap();
        let state = SessionContext::new().state();
        let native = adapter.bind(&state).unwrap();
        let physical = state
            .create_physical_expr(
                native.call(vec![col("value")]),
                &DFSchema::try_from(storage.as_ref().clone()).unwrap(),
            )
            .unwrap();
        let batch = RecordBatch::try_new(
            storage.clone(),
            vec![Arc::new(
                Decimal128Array::from(vec![i128::from(u64::MAX), 0])
                    .with_precision_and_scale(20, 0)
                    .unwrap(),
            )],
        )
        .unwrap();
        let result = physical.evaluate(&batch).unwrap().to_array(2).unwrap();
        let result = result.as_any().downcast_ref::<BooleanArray>().unwrap();
        assert!(result.value(0));
        assert!(!result.value(1));
        let invalid = RecordBatch::try_new(
            storage.clone(),
            vec![Arc::new(
                Decimal128Array::from(vec![-1])
                    .with_precision_and_scale(20, 0)
                    .unwrap(),
            )],
        )
        .unwrap();
        assert!(physical.evaluate(&invalid).is_err());
    }
}
