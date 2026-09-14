// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Allocation forecast from borrowed source values, before constructing tagged copies.
use crate::{CompilerError, PassContext, passes::dag::invalid};
use pse_schema::model::Cell;

pub(super) fn forecast(ctx: &PassContext<'_>, engine: bool) -> Result<usize, CompilerError> {
    let mut bytes = 4096;
    for (key, rows) in ctx.registry.schema_rows_ref() {
        ctx.cancel.checkpoint()?;
        bytes = add(bytes, add(key.name.len(), 256)?)?;
        for row in rows {
            bytes = add(bytes, 64)?;
            for value in row {
                bytes = add(bytes, literal(value)?.1)?;
            }
        }
    }
    let mut sources = 0;
    for document in ctx
        .documents
        .bundles()
        .iter()
        .flat_map(|bundle| &bundle.documents)
    {
        ctx.cancel.checkpoint()?;
        sources = add(sources, add(document.text.len(), document.path.len())?)?;
        bytes = add(
            bytes,
            add(document.text.len(), add(document.path.len(), 256)?)?,
        )?;
    }
    if sources > 16 << 20 {
        return Err(invalid(
            "actual sources exceed the durable control envelope",
        ));
    }
    for (name, policy) in &ctx.policies.0 {
        bytes = add(bytes, add(name.len(), 4096)?)?;
        bytes = add(
            bytes,
            pse_catalog::store::membership::validation_extent(policy.input.relation().batch())?,
        )?;
    }
    if engine {
        let session = ctx
            .session
            .ok_or_else(|| invalid("engine context missing"))?;
        bytes = add(bytes, session.semantic_inputs_extent()?)?;
    }
    Ok(bytes)
}
fn add(left: usize, right: usize) -> Result<usize, CompilerError> {
    left.checked_add(right)
        .ok_or_else(|| invalid("durable context allocation extent overflow"))
}
fn multiply(value: usize, count: usize) -> Result<usize, CompilerError> {
    value
        .checked_mul(count)
        .ok_or_else(|| invalid("durable context allocation extent overflow"))
}
// Bound both the escaped output and the temporary child strings/vector/join/final
// rendering used by Cell::literal_spec. Nested children are measured recursively.
fn literal(value: &Cell) -> Result<(usize, usize), CompilerError> {
    let mut temporary = 0;
    let length = match value {
        Cell::Text(text) => add(multiply(text.len(), 6)?, 128)?,
        Cell::Enum(text) => add(multiply(text.len(), 6)?, 128)?,
        Cell::List(values) | Cell::Struct(values) => {
            let mut length = 128;
            for value in values {
                let (child, work) = literal(value)?;
                length = add(length, add(child, 1)?)?;
                temporary = add(temporary, add(work, 64)?)?;
            }
            length
        }
        _ => 128,
    };
    Ok((length, add(multiply(length, 4)?, temporary)?))
}
