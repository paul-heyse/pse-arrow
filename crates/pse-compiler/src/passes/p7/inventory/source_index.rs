// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Algorithm lookup layout derived from native declaration identity/key projections.

use super::{Loader, invalid, native_rows};
use crate::CompilerError;
use datafusion::{
    arrow::array::{Array, FixedSizeBinaryArray},
    logical_expr::{LogicalPlanBuilder, col},
};
use pse_catalog::session::scalar;
use pse_ids::{ContentHash, SemanticId};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub(in crate::passes::p7) struct SourceIndex {
    keys: BTreeMap<(SemanticId, SemanticId), ContentHash>,
    owners: BTreeMap<(SemanticId, ContentHash), SemanticId>,
}

impl SourceIndex {
    pub(in crate::passes::p7) fn key(
        &self,
        relation: SemanticId,
        id: SemanticId,
    ) -> Option<ContentHash> {
        self.keys.get(&(relation, id)).copied()
    }

    pub(in crate::passes::p7) fn owner(
        &self,
        relation: SemanticId,
        key: ContentHash,
    ) -> Option<SemanticId> {
        self.owners.get(&(relation, key)).copied()
    }

    pub(super) async fn load(
        loader: &mut Loader<'_>,
        sources: &[pse_relations::generated::normalized::expression_sources::Row],
    ) -> Result<Self, CompilerError> {
        let mut result = Self::default();
        let selected = sources
            .iter()
            .map(|source| source.source_relation_id)
            .collect::<BTreeSet<_>>();
        for (key, batch) in loader.inputs {
            if key.namespace != pse_schema::model::Namespace::Normalized {
                continue;
            }
            let Some(authored) = loader.registry.relation(&format!("authored.{}", key.name)) else {
                continue;
            };
            if !selected.contains(&authored.id) {
                continue;
            }
            let [name] = authored.primary_key.as_slice() else {
                continue;
            };
            if authored
                .column(name)
                .is_none_or(|field| field.value_type() != pse_schema::model::FieldContract::id())
            {
                continue;
            }
            let session = loader.session.with_checked_role_inputs(
                BTreeMap::from([("source_declaration".to_owned(), batch.clone())]),
                loader.cancel,
            )?;
            let plan = LogicalPlanBuilder::from(session.scan_role("source_declaration")?)
                .project([
                    col(*name).alias("declaration_id"),
                    scalar::key(authored.id, vec![(*name, col(*name))]).alias("source_key"),
                ])
                .and_then(LogicalPlanBuilder::build)
                .map_err(native_rows::engine)?;
            let completed = loader
                .arguments
                .execute(plan, &session, loader.cancel)
                .await?;
            for batch in completed.batches() {
                let ids = batch
                    .column(0)
                    .as_any()
                    .downcast_ref::<FixedSizeBinaryArray>()
                    .ok_or_else(|| invalid("source declaration identity storage differs"))?;
                let keys = batch
                    .column(1)
                    .as_any()
                    .downcast_ref::<FixedSizeBinaryArray>()
                    .ok_or_else(|| invalid("source declaration key storage differs"))?;
                for row in 0..batch.num_rows() {
                    loader.cancel.checkpoint()?;
                    if ids.is_null(row) || keys.is_null(row) {
                        return Err(invalid("source declaration identity or key is null"));
                    }
                    let id = SemanticId::try_from_slice(ids.value(row))
                        .map_err(|_| invalid("source declaration identity width differs"))?;
                    let token = native_rows::key_value(keys.value(row))?;
                    if result.keys.insert((authored.id, id), token).is_some() {
                        return Err(invalid("source declaration identity is ambiguous"));
                    }
                    if matches!(*name, "equation_decl_id" | "symbol_decl_id") {
                        result.owners.insert((authored.id, token), id);
                    }
                }
            }
        }
        Ok(result)
    }
}
