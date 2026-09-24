// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One native-expression field transfer boundary. Ordinary built-ins keep their
//! names and kernels; only proven gaps at this exact pin install physical adapters.
//! Matching uses the actual implementation, never a function's SQL spelling.
use datafusion::{
    common::{
        Result,
        tree_node::{Transformed, TreeNode},
    },
    logical_expr::{Expr, LogicalPlan, ScalarUDF, WindowFunctionDefinition},
};
use std::sync::Arc;

pub(super) fn node(node: &mut LogicalPlan) -> Result<bool> {
    let mut changed = false;
    let expressions = node
        .expressions()
        .into_iter()
        .map(|expression| {
            let result = expression.transform_up(|mut expression| {
                let changed = match &mut expression {
                    Expr::AggregateFunction(call) => aggregate(&mut call.func),
                    Expr::WindowFunction(call) => match &mut call.fun {
                        WindowFunctionDefinition::AggregateUDF(function) => aggregate(function),
                        WindowFunctionDefinition::WindowUDF(_) => false,
                    },
                    Expr::ScalarFunction(call) => scalar(&mut call.func),
                    _ => false,
                };
                Ok(Transformed::new(
                    expression,
                    changed,
                    datafusion::common::tree_node::TreeNodeRecursion::Continue,
                ))
            })?;
            changed |= result.transformed;
            Ok(result.data)
        })
        .collect::<Result<Vec<_>>>()?;
    if changed {
        *node = node.with_new_exprs(expressions, node.inputs().into_iter().cloned().collect())?;
    }
    Ok(changed)
}
fn aggregate(function: &mut Arc<datafusion::logical_expr::AggregateUDF>) -> bool {
    if let Some(adapted) = super::aggregate::adapt(function) {
        *function = adapted;
        true
    } else {
        false
    }
}
fn scalar(function: &mut Arc<ScalarUDF>) -> bool {
    if function.as_ref() == datafusion::functions::core::named_struct().as_ref() {
        *function = super::scalar::structure::function(Arc::clone(function));
        true
    } else if function.as_ref()
        == datafusion::functions_nested::extract::array_element_udf().as_ref()
    {
        *function = super::scalar::element::function(Arc::clone(function));
        true
    } else {
        false
    }
}
pub(super) fn native_scalar(function: &ScalarUDF) -> Option<&Arc<ScalarUDF>> {
    super::scalar::structure::native(function).or_else(|| super::scalar::element::native(function))
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::{
        arrow::{
            array::{Int64Array, RecordBatch},
            datatypes::{DataType, Field},
        },
        execution::runtime_env::RuntimeEnv,
    };
    use pse_columnar::CancellationToken;
    use pse_schema::{
        RegistryBuilder,
        model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass},
    };
    use std::collections::{BTreeMap, HashMap};

    #[tokio::test]
    async fn ordinary_native_functions_carry_input_fields_without_parallel_sql_names() {
        let mut registry = RegistryBuilder::new();
        let field = Field::new("value", DataType::Int64, false).with_metadata(HashMap::from([(
            "unit-test-meaning".into(),
            "source".into(),
        )]));
        registry.declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                "inputs",
                1,
                Authority::Authored,
                SnapshotClass::Model,
                "isolated field transfer input",
            )
            .pk(&["value"])
            .columns(vec![FieldContract::from_field(field)]),
        );
        let registry = Arc::new(registry.build().unwrap());
        let spec = registry.relation("authored.inputs").unwrap();
        let batch = RecordBatch::try_new(
            Arc::new(pse_schema::arrow::relation_schema(&registry, spec).unwrap()),
            vec![Arc::new(Int64Array::from(vec![9, 3, 7]))],
        )
        .unwrap();
        let cancel = CancellationToken::new();
        let session = crate::session::EngineFactory::new(
            Arc::new(RuntimeEnv::default()),
            Arc::new(pse_columnar::GreedyMemoryPool::new(32 << 20)),
            crate::session::ExecutionSettings::default(),
            crate::session::ThreadBudget {
                pool_threads: 1.try_into().unwrap(),
                target_partitions: 1.try_into().unwrap(),
            },
            crate::session::native_engine_profile(),
        )
        .unwrap()
        .candidate(BTreeMap::from([(spec.key, batch)]), registry, &cancel)
        .unwrap();
        let batches = session.sql("SELECT min(value) AS lo, max(value) AS hi, array_agg(value ORDER BY value) AS items FROM authored.inputs", &cancel).await.unwrap();
        let schema = batches[0].schema();
        for name in ["lo", "hi"] {
            assert_eq!(
                schema
                    .field_with_name(name)
                    .unwrap()
                    .metadata()
                    .get("unit-test-meaning")
                    .map(String::as_str),
                Some("source")
            );
        }
        let DataType::List(child) = schema.field_with_name("items").unwrap().data_type() else {
            panic!("native list expected")
        };
        assert_eq!(
            child
                .metadata()
                .get("unit-test-meaning")
                .map(String::as_str),
            Some("source")
        );
        let nested = session.sql("SELECT named_struct('selected', array_element(array_agg(value ORDER BY value), 1)) AS item FROM authored.inputs", &cancel).await.unwrap();
        let nested_schema = nested[0].schema();
        let DataType::Struct(fields) = nested_schema.field(0).data_type() else {
            panic!("native struct expected")
        };
        assert_eq!(
            fields[0]
                .metadata()
                .get("unit-test-meaning")
                .map(String::as_str),
            Some("source")
        );
    }
}
