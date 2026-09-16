// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Configuration values and source lists cross native execution in the same rows.
use super::{Configuration, Source, invalid};
use crate::{
    CompilerError,
    passes::{
        native_outputs::{
            OutputRows, SourceKey,
            support::{self, SupportedBatch},
        },
        native_rows::{engine, key_value},
    },
};
use datafusion::{
    arrow::array::{Array, FixedSizeBinaryArray},
    logical_expr::{LogicalPlanBuilder, col},
};
use pse_catalog::session::scalar;
use pse_ids::SemanticId;
use pse_relations::columnar::{FieldCheckedBatch, RelationRow};
use pse_schema::model::RelationKey;
use std::collections::{BTreeMap, BTreeSet};

pub(super) type Origins = BTreeSet<SourceKey>;

impl Configuration<'_> {
    pub(super) async fn project_domains(
        &mut self,
        source: SemanticId,
        target: SemanticId,
    ) -> Result<(), CompilerError> {
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
                    source,
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
        let mut output = OutputRows::new(self.registry, self.reserver, self.cancel)?;
        // An empty native execution still produces the declared empty output.
        let empty = FieldCheckedBatch::concat_reserved(
            self.registry,
            spec,
            &[],
            self.reserver,
            self.cancel,
        )?;
        output.append_checked(&empty, |_, _| Err(invalid("empty domain output has a row")))?;
        for batch in completed.batches() {
            let keys = batch
                .column(spec.columns.len())
                .as_any()
                .downcast_ref::<FixedSizeBinaryArray>()
                .ok_or_else(|| invalid("configuration domain source key is not a typed key"))?;
            let payload =
                FieldCheckedBatch::admit_owned_projection(self.registry, spec, batch, &positions)?;
            output.append_checked(&payload, |_, row| {
                if keys.is_null(row) {
                    return Err(invalid("configuration domain source key is null"));
                }
                self.original(source, &key_value(keys.value(row))?)
            })?;
        }
        for (key, batch) in output.finish()?.columns {
            self.columns.append_supported(key, batch)?;
        }
        Ok(())
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
        key: &pse_ids::ContentHash,
    ) -> Result<Origins, CompilerError> {
        let relation = self
            .registry
            .relation_by_id(relation)
            .ok_or_else(|| invalid("configuration source declaration absent"))?
            .key;
        if let Some(origins) = self.generated_origins.get(&(relation, *key)) {
            return Ok(origins.clone());
        }
        self.sources.locate([(relation, *key)])
    }

    pub(super) fn owner(&self, instance: SemanticId) -> Result<Origins, CompilerError> {
        self.instances
            .get(&instance)
            .map(|instance| instance.origins.clone())
            .ok_or_else(|| invalid("prospective instance has no actual source correspondence"))
    }

    /// Native key calculation projects the support list beside its actual row.
    /// No execution-order or construction-ordinal lookup can reattach another row's sources.
    pub(super) async fn remember_generated(
        &mut self,
        relation: RelationKey,
        batch: &SupportedBatch,
    ) -> Result<(), CompilerError> {
        let spec = self
            .registry
            .relation_by_key(relation)
            .ok_or_else(|| invalid("generated source declaration absent"))?;
        let session = self
            .session
            .with_columnar_argument("configuration_generated", batch.data.clone(), self.cancel)
            .await?;
        let plan =
            LogicalPlanBuilder::from(session.scan_computation_role("configuration_generated")?)
                .project([
                    col(support::COLUMN),
                    scalar::key(
                        spec.id,
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
            let keys = batch
                .column_by_name("source_key")
                .and_then(|array| array.as_any().downcast_ref::<FixedSizeBinaryArray>())
                .ok_or_else(|| invalid("generated occurrence key is not a typed key"))?;
            for row in 0..batch.num_rows() {
                self.cancel.checkpoint()?;
                if keys.is_null(row) {
                    return Err(invalid("generated occurrence key is null"));
                }
                let origins = support::row_sources(batch, row, self.registry)?;
                if self
                    .generated_origins
                    .insert((relation, key_value(keys.value(row))?), origins)
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
