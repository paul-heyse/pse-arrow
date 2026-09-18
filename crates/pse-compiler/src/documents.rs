// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The explicit parsing boundary over the actual typed document relation.
use crate::CompilerError;
use pse_authoring::document::OwnedDocumentSet;
use pse_catalog::session::{RelationFacts, SnapshotSession};
use pse_ids::{CancellationToken, SemanticId};
use std::collections::BTreeMap;

/// Parse owned document fields for the existing source-language algorithms.
/// No store lookup or predecessor artifact is needed to recover source text.
/// # Errors
/// A different declaration, duplicate package/path, syntax, budget or cancellation.
pub fn parse(
    facts: &RelationFacts,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<OwnedDocumentSet, CompilerError> {
    let view = pse_relations::generated::authored::documents::View::from_checked(facts.checked())?;
    let mut reservation = session
        .reserver()
        .open("compiler:source-document-inventory");
    reservation
        .try_grow(pse_ids::validation_extent(facts.checked().batch())?)
        .map_err(pse_catalog::CatalogError::from)?;
    let mut texts = BTreeMap::<SemanticId, BTreeMap<String, String>>::new();
    for index in 0..facts.checked().batch().num_rows() {
        cancel
            .checkpoint()
            .map_err(pse_catalog::CatalogError::from)?;
        let row = view.row(index)?;
        if texts
            .entry(row.package_id)
            .or_default()
            .insert(row.path, row.source_text)
            .is_some()
        {
            return Err(crate::passes::invalid("duplicate document path in package"));
        }
    }
    let mut bundles = Vec::with_capacity(texts.len());
    for texts in texts.into_values() {
        bundles.push(pse_authoring::document::load_package_sources_owned(
            texts
                .iter()
                .map(|(path, text)| (path.as_str(), text.as_bytes())),
            session.registry(),
            pse_authoring::ParseBudget::default(),
            session.reserver(),
            cancel,
        )?);
    }
    Ok(OwnedDocumentSet::try_from_bundles(
        bundles,
        session.reserver(),
        cancel,
    )?)
}
