// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Invocation-local algorithm inventories keyed by the actual immutable relation owners.

use crate::{
    CompilerError, InputBundle, passes::dag::invalid, quantity_relations::PhysicalInventory,
};
use pse_catalog::{LoadedRelation, session::SnapshotSession};
use pse_ids::CancellationToken;
use pse_schema::{Registry, model::RelationKey};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
struct Entry {
    inputs: Vec<(RelationKey, Option<Arc<LoadedRelation>>)>,
    value: Arc<PhysicalInventory>,
}

#[derive(Debug, Default)]
pub(crate) struct PhysicalInventories(Mutex<Vec<Entry>>);

impl PhysicalInventories {
    pub(crate) async fn get(
        &self,
        inputs: &InputBundle,
        session: &SnapshotSession,
        registry: &Registry,
        cancel: &CancellationToken,
    ) -> Result<Arc<PhysicalInventory>, CompilerError> {
        let keys = crate::quantity_relations::inventory::input_keys(registry);
        let selected = keys
            .into_iter()
            .map(|key| {
                let spec = registry
                    .relation_by_key(key)
                    .ok_or_else(|| invalid("physical inventory declaration absent"))?;
                let mut candidates = inputs
                    .ports
                    .values()
                    .flatten()
                    .filter(|input| input.relation_id() == spec.id);
                let relation = candidates.next().map(|input| Arc::clone(input.relation()));
                if candidates.next().is_some() {
                    return Err(invalid(
                        "physical inventory requires one selected role per physical declaration",
                    ));
                }
                Ok((key, relation))
            })
            .collect::<Result<Vec<_>, CompilerError>>()?;
        let mut entries = self.0.lock().await;
        cancel
            .checkpoint()
            .map_err(pse_catalog::CatalogError::from)?;
        if let Some(entry) = entries.iter().find(|entry| same(&entry.inputs, &selected)) {
            return Ok(Arc::clone(&entry.value));
        }
        let value = Arc::new(PhysicalInventory::load(session, registry, cancel).await?);
        entries.push(Entry {
            inputs: selected,
            value: Arc::clone(&value),
        });
        Ok(value)
    }
}

fn same(
    left: &[(RelationKey, Option<Arc<LoadedRelation>>)],
    right: &[(RelationKey, Option<Arc<LoadedRelation>>)],
) -> bool {
    left.len() == right.len()
        && left.iter().zip(right).all(|((ak, a), (bk, b))| {
            ak == bk
                && match (a, b) {
                    (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                    (None, None) => true,
                    _ => false,
                }
        })
}
