// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact source occurrences for the finite configuration algorithm. These keys
//! are association data; native joins to retained owners establish membership.

use super::{Configuration, Configured, Source, invalid};
use crate::{
    CompilerError,
    passes::{
        native_outputs::{SourceKey, Sources},
        native_rows::engine,
    },
};
use datafusion::{
    arrow::array::{Array, StringArray, UInt64Array},
    logical_expr::{LogicalPlanBuilder, col},
};
use pse_catalog::session::scalar;
use pse_ids::SemanticId;
use pse_relations::columnar::{FieldCheckedBatch, RelationRow};
use pse_schema::model::RelationKey;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Origin {
    relation: RelationKey,
    key: String,
}
pub(super) type Origins = BTreeSet<Origin>;

impl Configuration<'_> {
    pub(super) async fn project_domains(
        &mut self,
        source: SemanticId,
        target: SemanticId,
    ) -> Result<(FieldCheckedBatch, Vec<Origins>), CompilerError> {
        let batch = self
            .binding_batches
            .get(&source)
            .ok_or_else(|| invalid("configuration domain inventory absent"))?
            .clone();
        let spec = self
            .registry
            .relation_by_id(target)
            .ok_or_else(|| invalid("normalized domain declaration absent"))?;
        let session = self.session.with_checked_role_inputs(
            BTreeMap::from([("configuration_domains".to_owned(), batch)]),
            self.cancel,
        )?;
        let plan = pse_catalog::session::output::declare_relation_projection(
            session.scan_role("configuration_domains")?,
            self.registry,
            spec,
            vec![
                scalar::key(
                    spec.primary_key
                        .iter()
                        .map(|name| (*name, col(*name)))
                        .collect(),
                )
                .alias("source_key"),
            ],
        )
        .map_err(engine)?;
        let completed = self.arguments.execute(plan, &session, self.cancel).await?;
        let positions = (0..spec.columns.len()).collect::<Vec<_>>();
        let mut pieces = Vec::new();
        let mut origins = Vec::new();
        for batch in completed.batches() {
            let keys = batch
                .column(spec.columns.len())
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| invalid("configuration domain source key is not Utf8"))?;
            for index in 0..batch.num_rows() {
                self.cancel.checkpoint()?;
                if keys.is_null(index) {
                    return Err(invalid("configuration domain source key is null"));
                }
                origins.push(self.original(source, keys.value(index))?);
            }
            pieces.push(FieldCheckedBatch::admit_owned_projection(
                self.registry,
                spec,
                batch,
                &positions,
            )?);
        }
        Ok((
            FieldCheckedBatch::concat_reserved(
                self.registry,
                spec,
                &pieces,
                self.reserver,
                self.cancel,
            )?,
            origins,
        ))
    }

    pub(super) fn origin<T: RelationRow>(
        &self,
        source: &Source<T>,
    ) -> Result<Origins, CompilerError> {
        self.original(T::relation(self.registry)?.id, &source.key)
    }

    pub(super) fn original(
        &self,
        relation: SemanticId,
        key: &str,
    ) -> Result<Origins, CompilerError> {
        let relation = self
            .registry
            .relation_by_id(relation)
            .ok_or_else(|| invalid("configuration source declaration absent"))?
            .key;
        Ok(self
            .generated_origins
            .get(&(relation, key.to_owned()))
            .cloned()
            .unwrap_or_else(|| {
                BTreeSet::from([Origin {
                    relation,
                    key: key.to_owned(),
                }])
            }))
    }

    pub(super) fn owner(&self, instance: SemanticId) -> Result<Origins, CompilerError> {
        self.instance_origins
            .get(&instance)
            .cloned()
            .ok_or_else(|| invalid("prospective instance has no actual source correspondence"))
    }

    pub(super) fn push<T: RelationRow>(
        &mut self,
        row: T,
        origins: Origins,
    ) -> Result<(), CompilerError> {
        if origins.is_empty() {
            return Err(invalid("configuration output lacks actual sources"));
        }
        let relation = T::relation(self.registry)?.key;
        self.arguments
            .reserve(origins.iter().try_fold(256usize, |bytes, source| {
                bytes
                    .checked_add(source.key.len())
                    .and_then(|bytes| bytes.checked_add(128))
                    .ok_or_else(|| invalid("configuration provenance extent overflow"))
            })?)?;
        self.columns.push(row)?;
        self.origins.entry(relation).or_default().push(origins);
        Ok(())
    }

    /// Retain generated keys with their pre-execution row ordinals. Native operators
    /// may reorder this projection; the explicit ordinal keeps each origin attached.
    pub(super) async fn remember_generated(
        &mut self,
        batch: &FieldCheckedBatch,
        origins: &[Origins],
    ) -> Result<(), CompilerError> {
        if batch.batch().num_rows() != origins.len() {
            return Err(invalid(
                "generated configuration source occurrence cardinality differs",
            ));
        }
        let spec = self
            .registry
            .relation_by_id(batch.relation_id())
            .ok_or_else(|| invalid("generated source declaration absent"))?;
        let session = self.session.with_indexed_checked_role(
            "configuration_generated",
            batch,
            self.cancel,
        )?;
        let plan =
            LogicalPlanBuilder::from(session.scan_computation_role("configuration_generated")?)
                .project([
                    col("constructed_row_ordinal"),
                    scalar::key(
                        spec.primary_key
                            .iter()
                            .map(|name| (*name, col(*name)))
                            .collect(),
                    )
                    .alias("source_key"),
                ])
                .and_then(LogicalPlanBuilder::build)
                .map_err(engine)?;
        let completed = self.arguments.execute(plan, &session, self.cancel).await?;
        for batch in completed.batches() {
            let ordinal = batch
                .column(0)
                .as_any()
                .downcast_ref::<UInt64Array>()
                .ok_or_else(|| invalid("generated occurrence ordinal is not UInt64"))?;
            let key = batch
                .column(1)
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| invalid("generated occurrence key is not Utf8"))?;
            for index in 0..batch.num_rows() {
                self.cancel.checkpoint()?;
                if ordinal.is_null(index) || key.is_null(index) {
                    return Err(invalid("generated occurrence is null"));
                }
                let origin = origins
                    .get(
                        usize::try_from(ordinal.value(index))
                            .map_err(|_| invalid("generated ordinal exceeds address space"))?,
                    )
                    .ok_or_else(|| invalid("generated occurrence escaped its actual output"))?;
                if self
                    .generated_origins
                    .insert((spec.key, key.value(index).to_owned()), origin.clone())
                    .is_some()
                {
                    return Err(invalid(
                        "generated configuration key collides with an earlier output",
                    ));
                }
            }
        }
        Ok(())
    }
}

impl Configured {
    pub(crate) fn captured(
        &self,
        sources: &Sources,
        registry: &pse_schema::Registry,
        reserver: &dyn pse_ids::MemoryReserver,
        cancel: &pse_ids::CancellationToken,
    ) -> Result<crate::passes::native_outputs::GeneratedOutputs, CompilerError> {
        super::super::provenance::capture(
            self.batches.clone(),
            |relation, ordinal| {
                self.origins
                    .get(&relation)
                    .and_then(|rows| rows.get(ordinal))
                    .ok_or_else(|| {
                        invalid("configuration output omitted its exact source occurrences")
                    })?
                    .iter()
                    .map(|source| {
                        let (port, _) = sources.get(&source.relation).ok_or_else(|| {
                            invalid("configuration source has no unique immutable input role")
                        })?;
                        Ok(SourceKey {
                            relation: source.relation,
                            port: port.clone(),
                            key: source.key.clone(),
                        })
                    })
                    .collect()
            },
            registry,
            reserver,
            cancel,
        )
    }
}

pub(super) type OutputOrigins = BTreeMap<RelationKey, Vec<Origins>>;
