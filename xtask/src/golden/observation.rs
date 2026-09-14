// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Lossless row observations and explicit parent handles for stored fixtures.

use super::Environment;
use anyhow::{Context, Result, ensure};
use pse_catalog::{Snapshot, snapshot::ManifestRef};
use pse_relations::generated::authored;
use pse_schema::{Registry, model::Cell};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};

type LiteralRows = Vec<Vec<String>>;
type RelationRows = BTreeMap<String, LiteralRows>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Index {
    pub snapshots: Vec<StoredSnapshot>,
    pub sources: BTreeMap<String, String>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StoredSnapshot {
    pub label: String,
    pub reference: ManifestRef,
    pub parents: BTreeMap<String, String>,
    pub stage_pass: Option<pse_ids::SemanticId>,
}
impl StoredSnapshot {
    pub(super) fn new(
        label: &str,
        snapshot: &Snapshot,
        known: &[(&str, &Arc<Snapshot>)],
    ) -> Result<Self> {
        let parents = snapshot
            .parents()
            .iter()
            .map(|(role, parent)| {
                let (label, _) = known
                    .iter()
                    .find(|(_, candidate)| Arc::ptr_eq(candidate, parent))
                    .with_context(|| format!("{role}: parent has no explicit stored label"))?;
                Ok((role.clone(), (*label).to_owned()))
            })
            .collect::<Result<_>>()?;
        Ok(Self {
            label: label.to_owned(),
            reference: snapshot.manifest_ref(),
            parents,
            stage_pass: snapshot.stage_pass(),
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Observation {
    pub snapshots: BTreeMap<String, RelationRows>,
    pub identities: BTreeMap<String, SnapshotIdentity>,
    pub queries: BTreeMap<String, LiteralRows>,
    pub sources: BTreeMap<String, String>,
    pub p10: Option<super::p10::Observation>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SnapshotIdentity {
    snapshot: String,
    relations: BTreeMap<String, String>,
}
pub(super) fn identity(snapshot: &Snapshot) -> SnapshotIdentity {
    SnapshotIdentity {
        snapshot: snapshot.snapshot_id().to_string(),
        relations: snapshot
            .relations()
            .values()
            .map(|relation| {
                (
                    format!("{}.{}", relation.member().namespace, relation.member().name),
                    relation.member().logical_hash.to_string(),
                )
            })
            .collect(),
    }
}

pub(super) fn rows(snapshot: &Snapshot, registry: &Registry) -> Result<RelationRows> {
    snapshot
        .relations()
        .values()
        .map(|relation| {
            let spec = registry
                .relation_by_id(relation.contract().canonical.relation_id)
                .context("admitted relation has no declaration")?;
            let rows = pse_relations::cells::cells_from_batch(registry, spec, relation.batch())?;
            Ok((spec.key.qualified_name(), literals(rows)))
        })
        .collect()
}

pub(super) fn normalization_budget_control(
    opened: &BTreeMap<String, Arc<Snapshot>>,
    registry: &Registry,
) -> Result<()> {
    if !opened.contains_key("P3") {
        return Ok(());
    }
    let model = opened.get("model").context("P3 fixture model absent")?;
    let inputs = model
        .relations()
        .values()
        .map(|relation| {
            let spec = registry
                .relation_by_id(relation.contract().canonical.relation_id)
                .context("reopened model relation undeclared")?;
            Ok((spec.key, relation.batch().clone()))
        })
        .collect::<Result<_>>()?;
    let budget = pse_ids::FixedBudget::new(1);
    let outcome = pse_compiler::passes::p3::normalize_owned(
        &inputs,
        &pse_authoring::document::OwnedDocumentSet::default(),
        registry,
        budget.as_ref(),
        &pse_ids::CancellationToken::new(),
    );
    ensure!(
        matches!(
            outcome,
            Err(pse_compiler::CompilerError::ResourceLimit { .. })
        ),
        "reopened P3 inputs must refuse a one-byte workspace before decoding"
    );
    ensure!(budget.reserved() == 0, "failed P3 reservation leaked bytes");
    Ok(())
}
fn literals(rows: Vec<Vec<Cell>>) -> LiteralRows {
    rows.into_iter()
        .map(|row| row.iter().map(Cell::literal_spec).collect())
        .collect()
}

pub(super) async fn query(
    environment: &Environment,
    opened: &BTreeMap<String, Arc<Snapshot>>,
    observed: &mut Observation,
) -> Result<()> {
    let mut queries = vec![(
        "model",
        "reference.schema_relations",
        "namespace, name, version",
    )];
    if opened.contains_key("P3") {
        queries.extend([
            ("case", "authored.cases", "case_id"),
            ("P3", "normalized.expression_sources", "source_id"),
            ("P3", "normalized.template_expr_int_constants", "node_id"),
        ]);
    }
    for (label, relation, order) in queries {
        let snapshot = opened.get(label).context("query snapshot absent")?;
        let session = environment.sessions.open_session(
            vec![Arc::clone(snapshot)],
            Arc::clone(&environment.registry),
            &environment.cancel,
        )?;
        let sql = format!("SELECT * FROM {relation} ORDER BY {order}");
        let batches = session.sql(&sql, &environment.cancel).await?;
        let values = batches
            .iter()
            .map(|batch| pse_relations::cells::decode_columns(&environment.registry, batch))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();
        let values = literals(values);
        ensure!(
            !values.is_empty(),
            "golden query must exercise actual rows: {sql}"
        );
        observed.queries.insert(sql, values);
    }
    Ok(())
}

pub(super) async fn sources(
    environment: &Environment,
    opened: &BTreeMap<String, Arc<Snapshot>>,
    expected: &BTreeMap<String, String>,
) -> Result<()> {
    let model = opened.get("model").context("model absent")?;
    let spec = environment
        .registry
        .relation("authored.documents")
        .context("document declaration absent")?;
    let relation = model
        .relation("authored", "documents")
        .context("complete model omitted documents")?;
    let mut actual = BTreeMap::new();
    for row in
        pse_relations::cells::cells_from_batch(&environment.registry, spec, relation.batch())?
    {
        let document = authored::documents::Row::from_cells(row)?;
        let bytes = environment
            .catalog
            .read_document(
                document.document_id,
                document.content_hash,
                &environment.cancel,
            )
            .await?;
        let text = std::str::from_utf8(&bytes)?.to_owned();
        ensure!(
            actual
                .insert(format!("{}/{}", document.package_id, document.path), text)
                .is_none(),
            "duplicate source path"
        );
    }
    ensure!(
        &actual == expected,
        "original source inventory or actual stored bytes differ"
    );
    Ok(())
}
