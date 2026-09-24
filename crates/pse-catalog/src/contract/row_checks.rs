// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Delta property rendering; native predicate binding lives in relations.
use datafusion::{arrow::datatypes::Schema, common::Result};
use std::collections::BTreeMap;

pub(crate) fn properties(schema: &Schema) -> Result<BTreeMap<String, String>> {
    Ok(pse_schema::arrow::native_checks(schema)
        .map_err(pse_columnar::external)?
        .into_iter()
        .map(|(name, sql)| {
            (
                format!("delta.constraints.pse_declared_{name}"),
                format!("({sql}) IS TRUE"),
            )
        })
        .collect())
}
