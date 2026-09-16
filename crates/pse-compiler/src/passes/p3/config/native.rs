// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native selection of configuration declarations and their exact source keys.
use super::invalid;
use crate::{CompilerError, passes::native_rows};
use datafusion::{common::ScalarValue, logical_expr::Expr};
use pse_catalog::session::output::checked_literal;
use pse_ids::SemanticId;
use pse_schema::{Registry, model::RelationSpec};

pub(super) use native_rows::{column, engine, join, scan};

pub(super) use crate::passes::native_rows::{Keyed as Source, keyed_rows as rows};

pub(super) fn identity(
    registry: &Registry,
    spec: &RelationSpec,
    field: &str,
    value: SemanticId,
) -> Result<Expr, CompilerError> {
    checked_literal(
        registry,
        spec.column(field)
            .ok_or_else(|| invalid("configuration identity field absent"))?,
        ScalarValue::FixedSizeBinary(16, Some(value.as_bytes().to_vec())),
    )
    .map_err(engine)
}
