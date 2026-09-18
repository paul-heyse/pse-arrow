// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native SQL is the row-predicate representation. The declaration is also persisted
//! as Delta CHECK, with identical false/null refusal and actual caller bindings.

use datafusion::{
    arrow::datatypes::{DataType, Schema},
    common::{DFSchema, DataFusionError, Result},
    execution::session_state::SessionState,
    logical_expr::{Expr, ExprSchemable},
};
use std::collections::BTreeMap;

pub(crate) fn properties(schema: &Schema) -> Result<BTreeMap<String, String>> {
    Ok(pse_schema::arrow::native_checks(schema)
        .map_err(external)?
        .into_iter()
        .map(|(name, sql)| {
            (
                format!("delta.constraints.pse_declared_{name}"),
                format!("({sql}) IS TRUE"),
            )
        })
        .collect())
}

/// Bind every declared predicate in the actual native function/configuration context.
/// The returned expressions accept only SQL true; false and unknown both violate.
/// # Errors
/// Malformed declarations, unbound columns/functions, non-Boolean or volatile checks.
pub fn bind(schema: &Schema, state: &SessionState) -> Result<BTreeMap<String, Expr>> {
    let native = DFSchema::try_from(schema.clone())?;
    pse_schema::arrow::native_checks(schema)
        .map_err(external)?
        .into_iter()
        .map(|(name, sql)| {
            let expression = state.create_logical_expr(&sql, &native)?;
            if expression.get_type(&native)? != DataType::Boolean || expression.is_volatile() {
                return Err(DataFusionError::Plan(format!(
                    "declared check {name} must be a nonvolatile Boolean expression"
                )));
            }
            Ok((name, expression.is_true()))
        })
        .collect()
}
fn external(error: impl std::error::Error + Send + Sync + 'static) -> DataFusionError {
    DataFusionError::External(Box::new(error))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "native predicate binding assertions")]
    use super::*;
    use datafusion::{arrow::datatypes::Field, execution::context::SessionContext};

    #[test]
    fn row_checks_bind_boolean_nonvolatile_expressions_in_the_actual_session() {
        let state = SessionContext::new().state();
        for (sql, valid) in [
            ("id > 0", true),
            ("id", false),
            ("random() > 0.1", false),
            ("missing_column > 0", false),
            ("unregistered_function(id)", false),
        ] {
            let checks = BTreeMap::from([("check".to_owned(), sql.to_owned())]);
            let schema = Schema::new_with_metadata(
                vec![Field::new("id", DataType::Int64, false)],
                [(
                    pse_schema::arrow::KEY_CHECKS.into(),
                    serde_json::to_string(&checks).unwrap(),
                )]
                .into(),
            );
            assert_eq!(bind(&schema, &state).is_ok(), valid, "{sql}");
        }
    }
}
