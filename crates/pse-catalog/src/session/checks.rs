// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native declared predicates bind once against the invocation's real state.

use super::{SnapshotSession, engine};
use crate::CatalogError;
use datafusion::logical_expr::Expr;
use pse_schema::model::RelationKey;
use std::collections::BTreeMap;

impl SnapshotSession {
    /// Bind named row checks for an exact retained relation. No rows are executed
    /// and no field, key or cross-relation obligation is certified by this binding.
    /// # Errors
    /// Missing source/declaration or invalid native SQL/function/type binding.
    pub fn row_check_expressions(
        &self,
        key: RelationKey,
    ) -> Result<BTreeMap<String, Expr>, CatalogError> {
        self.table_source(&key)?;
        let spec = self
            .registry()
            .relation_by_key(key)
            .ok_or_else(|| CatalogError::Admission {
                path: key.to_string(),
                reason: "native row checks require the exact declaration".into(),
            })?;
        let schema = pse_schema::arrow::relation_schema(self.registry(), spec)
            .map_err(|error| CatalogError::Semantic(std::sync::Arc::new(error)))?;
        crate::contract::row_checks::bind(&schema, &self.bound_state()?).map_err(engine)
    }
}
