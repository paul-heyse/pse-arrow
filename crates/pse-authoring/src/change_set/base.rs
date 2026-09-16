// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Borrow existing field admission, or establish it once for an external raw reader.
use super::{AuthoredReader, contract};
use crate::{AuthoringError, document::Batches};
use pse_catalog::session::SnapshotSession;
use pse_ids::SemanticId;
use pse_relations::{RecordBatch, columnar::FieldCheckedBatch};
use std::{borrow::Cow, collections::BTreeMap};

pub(crate) fn checked<'a>(
    base: &'a dyn AuthoredReader,
    session: &SnapshotSession,
) -> Result<Cow<'a, Batches>, AuthoringError> {
    if let Some(rows) = base.checked_relations() {
        for (id, batch) in rows {
            let spec = session
                .registry()
                .relation_by_id(*id)
                .ok_or_else(|| contract("base relation absent"))?;
            batch.check_declaration(session.registry(), spec)?;
        }
        return Ok(Cow::Borrowed(rows));
    }
    let rows = base
        .relations()?
        .into_iter()
        .map(|(id, batch)| {
            let spec = session
                .registry()
                .relation_by_id(id)
                .ok_or_else(|| contract("base relation absent"))?;
            Ok((
                id,
                FieldCheckedBatch::admit(session.registry(), spec, batch)?,
            ))
        })
        .collect::<Result<_, AuthoringError>>()?;
    Ok(Cow::Owned(rows))
}

pub(crate) struct CheckedBase<'a> {
    pub(crate) base: &'a dyn AuthoredReader,
    pub(crate) rows: &'a Batches,
}
impl AuthoredReader for CheckedBase<'_> {
    fn revision_id(&self) -> SemanticId {
        self.base.revision_id()
    }
    fn checked_relations(&self) -> Option<&Batches> {
        Some(self.rows)
    }
    fn relations(&self) -> Result<BTreeMap<SemanticId, RecordBatch>, AuthoringError> {
        Ok(crate::p1::raw_batches(self.rows))
    }
    fn source_documents(&self) -> Result<BTreeMap<SemanticId, String>, AuthoringError> {
        self.base.source_documents()
    }
}
