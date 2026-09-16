// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P0 package plans share the frozen native execution session.
use crate::{CompilerError, passes::dag::invalid};
use pse_catalog::session::SnapshotSession;
use pse_ids::CancellationToken;
use pse_relations::{RecordBatch, columnar::FieldCheckedBatch};
use pse_schema::model::RelationKey;
use std::collections::BTreeMap;

/// Resolve actual package fields through native relational/dependency construction.
/// # Errors
/// Missing headers, incompatible exact versions, cyclic dependencies or native failure.
pub async fn resolve(
    rows: &BTreeMap<RelationKey, RecordBatch>,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<RecordBatch, CompilerError> {
    let registry = session.registry();
    let spec = registry
        .relation("authored.packages")
        .ok_or_else(|| invalid("P0 packages undeclared"))?;
    let batch = rows
        .get(&spec.key)
        .ok_or_else(|| invalid("P0 packages absent"))?;
    let checked = FieldCheckedBatch::admit(registry, spec, batch.clone())?;
    Ok(pse_authoring::p0::resolve(&checked, session, cancel)
        .await?
        .into_batch())
}
