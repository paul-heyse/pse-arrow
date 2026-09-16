// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual immutable rows and source rows behind compiler input bindings.
use crate::{
    CompilerError,
    passes::{BoundInput, InputBundle, dag::invalid},
};
use pse_authoring::{
    AuthoringError, ParseBudget, change_set::AuthoredReader, document::OwnedDocumentSet,
};
use pse_catalog::{Catalog, Snapshot, store::membership::validation_extent};
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::{RecordBatch, generated::authored};
use pse_schema::{
    Registry,
    model::{Authority, RelationKey},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// Exact commit base, including original bytes read from its typed document rows.
#[derive(Clone, Debug)]
pub struct BaseReader {
    pub(crate) revision: SemanticId,
    pub(crate) rows: BTreeMap<SemanticId, RecordBatch>,
    pub(crate) checked: Option<pse_authoring::document::Batches>,
    pub(crate) documents: OwnedDocumentSet,
}
impl AuthoredReader for BaseReader {
    fn checked_relations(&self) -> Option<&pse_authoring::document::Batches> {
        self.checked.as_ref()
    }
    fn revision_id(&self) -> SemanticId {
        self.revision
    }
    fn relations(&self) -> Result<BTreeMap<SemanticId, RecordBatch>, AuthoringError> {
        Ok(self.rows.clone())
    }
    fn source_documents(&self) -> Result<BTreeMap<SemanticId, String>, AuthoringError> {
        Ok(self
            .documents
            .bundles()
            .iter()
            .flat_map(|bundle| {
                bundle
                    .documents
                    .iter()
                    .map(|document| (document.id, document.text.clone()))
            })
            .collect())
    }
}
/// Find complete immutable ancestors without identifying them by hashes.
pub(crate) fn inventory(
    root: &Arc<Snapshot>,
    registry: &Registry,
) -> Result<BTreeMap<RelationKey, BoundInput>, CompilerError> {
    let mut found = BTreeMap::new();
    let mut stack = vec![Arc::clone(root)];
    let mut visited = BTreeSet::new();
    while let Some(snapshot) = stack.pop() {
        if !visited.insert(Arc::as_ptr(&snapshot)) {
            continue;
        }
        for relation in snapshot.relations().values() {
            let spec = registry
                .relation_by_id(relation.contract().canonical.relation_id)
                .ok_or_else(|| invalid("snapshot declaration absent"))?;
            if let Some(prior) = found.get(&spec.key) {
                let prior: &BoundInput = prior;
                if prior.relation().batch() != relation.batch() {
                    return Err(invalid("ambiguous actual ancestor relation bindings"));
                }
            } else {
                found.insert(
                    spec.key,
                    BoundInput::bind(Arc::clone(&snapshot), spec.key, registry)?,
                );
            }
        }
        stack.extend(snapshot.parents().values().cloned());
    }
    Ok(found)
}
pub(crate) fn row_inventory(
    inputs: &BTreeMap<RelationKey, BoundInput>,
) -> BTreeMap<RelationKey, RecordBatch> {
    inputs
        .iter()
        .map(|(key, input)| (*key, input.relation().batch().clone()))
        .collect()
}
pub(crate) fn documents(
    catalog: &Catalog,
    rows: &BTreeMap<RelationKey, RecordBatch>,
    cancel: &CancellationToken,
) -> Result<OwnedDocumentSet, CompilerError> {
    let registry = catalog.registry();
    let spec = registry
        .relation("authored.documents")
        .ok_or_else(|| invalid("document contract absent"))?;
    let Some(batch) = rows.get(&spec.key) else {
        return Ok(OwnedDocumentSet::default());
    };
    cancel.checkpoint()?;
    let mut metadata = catalog
        .reserver()
        .open("compiler:source-document-inventory");
    metadata
        .try_grow(validation_extent(batch)?)
        .map_err(|error| CompilerError::Catalog(error.into()))?;
    let mut texts = BTreeMap::<SemanticId, BTreeMap<_, _>>::new();
    for row in pse_relations::cells::cells_from_batch(registry, spec, batch)? {
        cancel.checkpoint()?;
        let row = authored::documents::Row::from_cells(row)?;
        if texts
            .entry(row.package_id)
            .or_default()
            .insert(row.path, row.source_text)
            .is_some()
        {
            return Err(invalid("duplicate document path in package"));
        }
    }
    let mut bundles = Vec::new();
    for (_, texts) in texts {
        bundles.push(pse_authoring::document::load_package_sources_owned(
            texts
                .iter()
                .map(|(path, bytes)| (path.as_str(), bytes.as_bytes())),
            registry,
            ParseBudget::default(),
            catalog.reserver().as_ref(),
            cancel,
        )?);
    }
    Ok(OwnedDocumentSet::try_from_bundles(
        bundles,
        catalog.reserver().as_ref(),
        cancel,
    )?)
}
pub(crate) fn primitive_rows(
    rows: &BTreeMap<RelationKey, RecordBatch>,
    registry: &Registry,
) -> BTreeMap<SemanticId, RecordBatch> {
    rows.iter()
        .filter_map(|(key, batch)| {
            registry
                .relation(&key.qualified_name())
                .filter(|spec| matches!(spec.authority, Authority::Authored | Authority::Reference))
                .map(|spec| (spec.id, batch.clone()))
        })
        .collect()
}
pub(super) fn bind(
    spec: &pse_schema::model::PassSpec,
    registry: &Registry,
    pinned: &BTreeMap<RelationKey, BoundInput>,
    stages: &BTreeMap<String, Arc<Snapshot>>,
) -> Result<InputBundle, CompilerError> {
    let mut bundle = InputBundle::new();
    for port in &spec.inputs {
        let relation = registry
            .relation(&port.relation)
            .ok_or_else(|| invalid("pass relation absent"))?;
        let input = match port.source {
            pse_schema::model::PortSource::Pinned => pinned.get(&relation.key).cloned(),
            pse_schema::model::PortSource::Derived { pass, port } => stages
                .get(pass)
                .map(|snapshot| {
                    BoundInput::bind_port(Arc::clone(snapshot), relation.key, port, registry)
                })
                .transpose()?,
        };
        bundle.ports.insert(port.port, input);
    }
    bundle.validate(spec, registry)?;
    Ok(bundle)
}
