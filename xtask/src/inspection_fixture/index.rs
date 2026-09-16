// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit handles needed to reopen the current inspection fixture.

use anyhow::{Context, Result};
use pse_catalog::{Snapshot, snapshot::ManifestRef};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Index {
    pub snapshots: Vec<StoredSnapshot>,
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
