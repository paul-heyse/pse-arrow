// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native configuration selection with an explicit finite instance-graph algorithm.
mod choices;
mod instances;
mod material_domains;
mod native;
mod provenance;
pub(crate) mod selected;
mod syntax;
mod value;
mod values;

use super::invalid;
use crate::{
    CompilerError, passes::native_rows::AlgorithmInputs, quantity_relations::PhysicalInventory,
};
use datafusion::logical_expr::{Expr, LogicalPlanBuilder, col};
use native::{Source, engine};
use provenance::{Origins, OutputOrigins};
use pse_authoring::document::OwnedDocumentSet;
use pse_catalog::session::SnapshotSession;
use pse_catalog::session::scalar::array_element;
use pse_ids::{CancellationToken, MemoryReserver, SemanticId};
use pse_relations::{
    columnar::{Collection, FieldCheckedBatch, RelationRow},
    generated::{authored, normalized},
};
use pse_schema::{Registry, model::RelationKey};
use std::collections::BTreeMap;

type Value = normalized::config_values::NormalizedConfigValuesFieldValue;

pub(crate) struct Configured {
    pub(crate) batches: BTreeMap<RelationKey, FieldCheckedBatch>,
    pub(crate) _arguments: AlgorithmInputs,
    origins: OutputOrigins,
}

pub(super) struct Configuration<'a> {
    syntax: syntax::ConfigurationSyntax<'a>,
    binding_batches: BTreeMap<SemanticId, FieldCheckedBatch>,
    instances: Vec<instances::Prospective>,
    values: BTreeMap<(SemanticId, String), Value>,
    domains: BTreeMap<(SemanticId, String), SemanticId>,
    generated_domains: BTreeMap<SemanticId, FieldCheckedBatch>,
    columns: Collection<'a>,
    arguments: AlgorithmInputs,
    origins: OutputOrigins,
    generated_origins: BTreeMap<(RelationKey, String), Origins>,
    instance_origins: BTreeMap<SemanticId, Origins>,
    value_origins: BTreeMap<(SemanticId, String), Origins>,
    domain_origins: BTreeMap<(SemanticId, String), Origins>,
    session: &'a SnapshotSession,
    registry: &'a Registry,
    physical: &'a PhysicalInventory,
    reserver: &'a dyn MemoryReserver,
    cancel: &'a CancellationToken,
}

impl<'a> Configuration<'a> {
    pub(super) async fn build(
        source: &BTreeMap<SemanticId, FieldCheckedBatch>,
        documents: &'a OwnedDocumentSet,
        session: &'a SnapshotSession,
        physical: &'a PhysicalInventory,
        cancel: &'a CancellationToken,
    ) -> Result<Self, CompilerError> {
        Self::build_selected(source, documents, session, physical, cancel, &[]).await
    }

    async fn build_selected(
        source: &BTreeMap<SemanticId, FieldCheckedBatch>,
        documents: &'a OwnedDocumentSet,
        session: &'a SnapshotSession,
        physical: &'a PhysicalInventory,
        cancel: &'a CancellationToken,
        selected: &[selected::SelectedRoot],
    ) -> Result<Self, CompilerError> {
        let registry = session.registry();
        let reserver = session.reserver();
        let mut arguments = AlgorithmInputs::new(reserver, "configuration-arguments");
        let syntax = syntax::ConfigurationSyntax::new(documents, registry, &mut arguments, cancel)?;
        let mut result = Self {
            syntax,
            binding_batches: source.clone(),
            instances: Vec::new(),
            values: BTreeMap::new(),
            domains: BTreeMap::new(),
            generated_domains: BTreeMap::new(),
            columns: Collection::new(registry, reserver, cancel),
            arguments,
            origins: BTreeMap::new(),
            generated_origins: BTreeMap::new(),
            instance_origins: BTreeMap::new(),
            value_origins: BTreeMap::new(),
            domain_origins: BTreeMap::new(),
            session,
            registry,
            physical,
            reserver,
            cancel,
        };
        result.columns.ensure::<normalized::config_values::Row>()?;
        result
            .columns
            .ensure::<normalized::feature_inheritance::Row>()?;
        result
            .columns
            .ensure::<normalized::material_domain_members::Row>()?;
        result
            .columns
            .ensure::<normalized::instance_domain_bindings::Row>()?;
        instances::expand(&mut result, selected).await?;
        let mut prospective = Collection::new(registry, reserver, cancel);
        prospective.ensure::<normalized::instance_bindings::Row>()?;
        for instance in &result.instances {
            prospective.push(instance.binding())?;
        }
        for (_, batch) in prospective.finish()? {
            result.binding_batches.insert(batch.relation_id(), batch);
        }
        let columns = std::mem::replace(
            &mut result.columns,
            Collection::new(registry, reserver, cancel),
        );
        for (_, batch) in columns.finish()? {
            result.binding_batches.insert(batch.relation_id(), batch);
        }
        Ok(result)
    }

    pub(super) fn binding_batches(&self) -> &BTreeMap<SemanticId, FieldCheckedBatch> {
        &self.binding_batches
    }

    async fn select<T: RelationRow>(
        &mut self,
        predicates: Vec<Expr>,
    ) -> Result<Vec<Source<T>>, CompilerError> {
        let spec = T::relation(self.registry)?;
        let batch = self
            .binding_batches
            .get(&spec.id)
            .ok_or_else(|| invalid("configuration source relation absent"))?
            .clone();
        let session = self.session.with_checked_role_inputs(
            BTreeMap::from([("configuration_source".to_owned(), batch)]),
            self.cancel,
        )?;
        let mut plan = LogicalPlanBuilder::from(session.scan_role("configuration_source")?)
            .alias("d")
            .map_err(engine)?;
        for predicate in predicates {
            plan = plan.filter(predicate).map_err(engine)?;
        }
        native::rows(
            &mut self.arguments,
            plan.build().map_err(engine)?,
            &session,
            self.registry,
            self.cancel,
        )
        .await
    }

    async fn by_id<T: RelationRow>(
        &mut self,
        field: &str,
        id: SemanticId,
    ) -> Result<Vec<Source<T>>, CompilerError> {
        let literal = native::identity(self.registry, T::relation(self.registry)?, field, id)?;
        self.select::<T>(vec![col(field).eq(literal)]).await
    }

    async fn one<T: RelationRow>(
        &mut self,
        field: &str,
        id: SemanticId,
    ) -> Result<Source<T>, CompilerError> {
        let mut rows = self.by_id::<T>(field, id).await?;
        if rows.len() != 1 {
            return Err(invalid("configuration reference is missing or ambiguous"));
        }
        rows.pop()
            .ok_or_else(|| invalid("configuration selection disappeared"))
    }

    async fn retain_generated<T: RelationRow>(
        &mut self,
        row: T,
        origins: Origins,
    ) -> Result<FieldCheckedBatch, CompilerError> {
        let mut columns = Collection::new(self.registry, self.reserver, self.cancel);
        columns.push(row)?;
        let spec = T::relation(self.registry)?;
        let batch = columns
            .finish()?
            .remove(&spec.key)
            .ok_or_else(|| invalid("generated configuration row absent"))?;
        self.remember_generated(&batch, &[origins]).await?;
        self.merge_generated(batch.clone()).await?;
        Ok(batch)
    }

    async fn merge_generated(&mut self, batch: FieldCheckedBatch) -> Result<(), CompilerError> {
        let spec = self
            .registry
            .relation_by_id(batch.relation_id())
            .ok_or_else(|| invalid("generated declaration absent"))?;
        let joined = if let Some(previous) = self.binding_batches.get(&spec.id) {
            let session = self.session.with_checked_role_inputs(
                BTreeMap::from([
                    ("configuration_previous".to_owned(), previous.clone()),
                    ("configuration_additions".to_owned(), batch),
                ]),
                self.cancel,
            )?;
            let plan = LogicalPlanBuilder::from(session.scan_role("configuration_previous")?)
                .union(session.scan_role("configuration_additions")?)
                .map_err(engine)?
                .build()
                .map_err(engine)?;
            let plan =
                pse_catalog::session::output::declare_relation_output(plan, self.registry, spec)
                    .map_err(engine)?;
            let completed = session
                .prepare_rule_plan(plan, self.cancel)?
                .execute(self.cancel)
                .await?;
            completed.into_checked_relation(self.registry, spec, self.cancel)?
        } else {
            batch
        };
        self.binding_batches.insert(spec.id, joined);
        Ok(())
    }

    pub(super) async fn emit(
        mut self,
        expression_sources: &FieldCheckedBatch,
    ) -> Result<Configured, CompilerError> {
        let mut output = BTreeMap::new();
        let bindings = std::mem::take(&mut self.instances);
        self.columns
            .ensure::<normalized::instance_bindings::Row>()?;
        self.binding_batches
            .insert(expression_sources.relation_id(), expression_sources.clone());
        for instance in bindings {
            let mut binding = instance.binding();
            let mut origins = self.owner(binding.instance_id)?;
            if let Some(guard) = instance.guard {
                let guard = self
                    .one::<authored::template_guards::Row>("guard_id", guard)
                    .await?;
                origins.extend(self.origin(&guard)?);
                let source = self.guard_source(guard.row.guard_id).await?;
                origins.extend(self.origin(&source)?);
                binding.guard_source_id = Some(source.row.source_id);
                binding.guard_node_id = Some(source.row.root_id);
            }
            self.push(binding, origins)?;
        }
        let columns = std::mem::replace(
            &mut self.columns,
            Collection::new(self.registry, self.reserver, self.cancel),
        );
        output.extend(columns.finish()?);
        for id in [
            normalized::config_values::RELATION_ID,
            normalized::feature_inheritance::RELATION_ID,
            normalized::material_domain_members::RELATION_ID,
            normalized::instance_domain_bindings::RELATION_ID,
        ] {
            let spec = self
                .registry
                .relation_by_id(id)
                .ok_or_else(|| invalid("configuration output undeclared"))?;
            output.insert(
                spec.key,
                self.binding_batches
                    .get(&id)
                    .ok_or_else(|| invalid("configuration output absent"))?
                    .clone(),
            );
        }
        for (source, target) in [
            (
                authored::domains::RELATION_ID,
                normalized::domains::RELATION_ID,
            ),
            (
                authored::domain_members::RELATION_ID,
                normalized::domain_members::RELATION_ID,
            ),
        ] {
            let spec = self
                .registry
                .relation_by_id(target)
                .ok_or_else(|| invalid("normalized domain declaration absent"))?;
            let (batch, origins) = self.project_domains(source, target).await?;
            output.insert(spec.key, batch);
            self.origins.insert(spec.key, origins);
        }
        Ok(Configured {
            batches: output,
            _arguments: self.arguments,
            origins: self.origins,
        })
    }

    async fn guard_source(
        &mut self,
        guard: SemanticId,
    ) -> Result<Source<normalized::expression_sources::Row>, CompilerError> {
        use datafusion::functions::core::expr_fn::get_field;
        use datafusion::logical_expr::lit;
        let spec = normalized::expression_sources::Row::relation(self.registry)?;
        let relation = native::identity(
            self.registry,
            spec,
            "source_relation_id",
            authored::template_guards::RELATION_ID,
        )?;
        let guard_value = native::identity(
            self.registry,
            authored::template_guards::Row::relation(self.registry)?,
            "guard_id",
            guard,
        )?;
        let key = array_element(col("source_key"), lit(1_i64));
        let rows = self
            .select::<normalized::expression_sources::Row>(vec![
                col("source_relation_id").eq(relation),
                get_field(key.clone(), "column_name").eq(lit("guard_id")),
                get_field(key, "semantic_id").eq(guard_value),
            ])
            .await?;
        let [row] = rows.as_slice() else {
            return Err(invalid("prospective guard source missing or ambiguous"));
        };
        Ok(row.clone())
    }
}
