// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native declared predicates bind once against the invocation's real state.

use super::{EngineSession, engine};
use crate::EngineError;
use datafusion::logical_expr::Expr;
use pse_schema::model::RelationKey;
use std::collections::BTreeMap;

impl EngineSession {
    /// Bind named row checks for an exact retained relation. No rows are executed
    /// and no field, key or cross-relation obligation is certified by this binding.
    /// # Errors
    /// Missing source/declaration or invalid native SQL/function/type binding.
    pub fn row_check_expressions(
        &self,
        key: RelationKey,
    ) -> Result<BTreeMap<String, Expr>, EngineError> {
        self.table_source(&key)?;
        let spec = self
            .registry()
            .relation_by_key(key)
            .ok_or_else(|| EngineError::Admission {
                path: key.to_string(),
                reason: "native row checks require the exact declaration".into(),
            })?;
        let schema = pse_schema::arrow::relation_schema(self.registry(), spec)
            .map_err(|error| EngineError::Semantic(std::sync::Arc::new(error)))?;
        pse_relations::validate::row_checks::bind(
            &schema,
            &crate::validation::NativeValidation(self.bound_state()?),
        )
        .map_err(engine)
    }
}

/// Shared native local-field checks for owned intermediate Arrow arguments.
/// This establishes neither cross-relation references nor declared keys.
/// # Errors
/// Undeclared fields or native validation-plan construction failure.
pub fn local_field_violations(
    registry: &pse_schema::Registry,
    input: datafusion::logical_expr::LogicalPlan,
) -> datafusion::common::Result<datafusion::logical_expr::LogicalPlan> {
    let predicate =
        pse_relations::validate::predicates::relation(registry, input.schema().as_arrow())?;
    datafusion::logical_expr::LogicalPlanBuilder::from(input)
        .filter(predicate.is_not_true())?
        .limit(0, Some(1))?
        .build()
}
