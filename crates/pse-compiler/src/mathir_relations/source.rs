// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact generated Arrow views feed the one mathematical graph algorithm.

use super::malformed;
use crate::CompilerError;
use pse_catalog::session::{SnapshotSession, output::declare_relation_output};
use pse_ids::CancellationToken;
use pse_mathir::{
    MathIrError,
    relations::{MathRelationSink, MathRelationSource},
};
use pse_relations::{RecordBatch, columnar::FieldCheckedBatch};
use pse_schema::{
    Registry,
    model::{Namespace, RelationKey, RelationSpec},
};
use std::collections::BTreeMap;

/// The exact registered storage family being admitted.
#[derive(Clone, Copy, Debug)]
pub enum SourceFamily<'a> {
    /// Physically compiled contracts.
    Compiled,
    /// Instantiated actual references with pending physical requests.
    Inferred,
    /// One complete normalized expression family.
    Normalized {
        /// Registry-declared family prefix.
        prefix: &'a str,
    },
}

/// Checked column owners; graph and occurrence-typing algorithms follow typed replay.
#[derive(Debug)]
pub struct RelationSource<'a> {
    registry: &'a Registry,
    batches: BTreeMap<RelationKey, FieldCheckedBatch>,
}
impl<'a> RelationSource<'a> {
    /// Admit raw compiled fields once at an external adapter boundary.
    /// # Errors
    /// Unknown or malformed actual relation values.
    pub fn from_batches(
        rows: &BTreeMap<RelationKey, RecordBatch>,
        registry: &'a Registry,
    ) -> Result<Self, CompilerError> {
        Self::from_family(rows, registry, SourceFamily::Compiled)
    }
    /// Admit raw family columns once, without reconstructing a Cell inventory.
    /// Native callers should retain checked columns or use `load_family`.
    /// # Errors
    /// Missing complete normalized/inferred family or invalid local fields.
    pub fn from_family(
        rows: &BTreeMap<RelationKey, RecordBatch>,
        registry: &'a Registry,
        family: SourceFamily<'_>,
    ) -> Result<Self, CompilerError> {
        let mut batches = BTreeMap::new();
        for spec in family_specs(registry, family)? {
            let Some(batch) = rows.get(&spec.key) else {
                if matches!(family, SourceFamily::Compiled) {
                    continue;
                }
                return Err(malformed(format!("complete math input omits {}", spec.key)).into());
            };
            batches.insert(
                spec.key,
                FieldCheckedBatch::admit(registry, spec, batch.clone())?,
            );
        }
        Ok(Self { registry, batches })
    }
    /// Retain exact already checked family columns without rescanning local values.
    /// # Errors
    /// Different declarations or an incomplete inferred/normalized family.
    pub fn from_checked(
        rows: &BTreeMap<RelationKey, FieldCheckedBatch>,
        registry: &'a Registry,
        family: SourceFamily<'_>,
    ) -> Result<Self, CompilerError> {
        let mut batches = BTreeMap::new();
        for spec in family_specs(registry, family)? {
            let Some(batch) = rows.get(&spec.key) else {
                if matches!(family, SourceFamily::Compiled) {
                    continue;
                }
                return Err(malformed(format!("complete math input omits {}", spec.key)).into());
            };
            batch.check_declaration(registry, spec)?;
            batches.insert(spec.key, batch.clone());
        }
        Ok(Self { registry, batches })
    }
    /// Select and order complete graph relations using the actual native session.
    /// Payload fields and ordered arguments retain their exact generated declarations.
    /// # Errors
    /// Missing binding, native preparation/execution, cancellation or resource refusal.
    pub async fn load_family(
        session: &'a SnapshotSession,
        family: SourceFamily<'_>,
        cancel: &CancellationToken,
    ) -> Result<Self, CompilerError> {
        use datafusion::logical_expr::{LogicalPlanBuilder, col};
        let registry = session.registry();
        let mut batches = BTreeMap::new();
        for spec in family_specs(registry, family)? {
            cancel.checkpoint()?;
            if session.table_provider(&spec.key).is_none() {
                if matches!(family, SourceFamily::Compiled) {
                    continue;
                }
                return Err(malformed(format!("complete math input omits {}", spec.key)).into());
            }
            let plan = LogicalPlanBuilder::scan(
                session.table_reference(&spec.key)?,
                session.table_source(&spec.key)?,
                None,
            )
            .and_then(|plan| {
                plan.sort(
                    spec.primary_key
                        .iter()
                        .map(|name| col(*name).sort(true, false)),
                )
            })
            .and_then(LogicalPlanBuilder::build)
            .map_err(engine)?;
            let plan = declare_relation_output(plan, registry, spec).map_err(engine)?;
            let batch = session
                .prepare_rule_plan(plan, cancel)?
                .execute(cancel)
                .await?
                .into_checked_relation(registry, spec, cancel)?;
            batches.insert(spec.key, batch);
        }
        Ok(Self { registry, batches })
    }
    /// Exact retained output columns, suitable for subsequent native context joins.
    pub fn batches(&self) -> &BTreeMap<RelationKey, FieldCheckedBatch> {
        &self.batches
    }
}
impl MathRelationSource for RelationSource<'_> {
    fn read(&self, sink: &mut dyn MathRelationSink) -> Result<(), MathIrError> {
        for (key, batch) in &self.batches {
            let spec = self
                .registry
                .relation(&key.qualified_name())
                .ok_or_else(|| malformed("mathematical declaration disappeared"))?;
            batch
                .check_declaration(self.registry, spec)
                .map_err(|error| malformed(error.to_string()))?;
            crate::generated::mathir_source::replay(*key, batch, sink)?;
        }
        Ok(())
    }
}
fn family_specs<'a>(
    registry: &'a Registry,
    family: SourceFamily<'_>,
) -> Result<Vec<&'a RelationSpec>, CompilerError> {
    if let SourceFamily::Normalized { prefix } = family
        && pse_schema::catalog::expr_family::target_name(prefix, "math_expr_nodes").is_none()
    {
        return Err(malformed("unknown normalized mathematical family").into());
    }
    registry
        .relations()
        .iter()
        .filter(|spec| {
            spec.key.namespace == Namespace::Compiled
                && !matches!(spec.key.name, "math_implicit_systems" | "math_dae_links")
                && (spec.key.name.starts_with("math_") || spec.key.name == "kernel_bindings")
        })
        .filter_map(|source| {
            let (namespace, name) = match family {
                SourceFamily::Compiled => (Namespace::Compiled, source.key.name),
                SourceFamily::Inferred => (Namespace::Inferred, source.key.name),
                SourceFamily::Normalized { prefix } => (
                    Namespace::Normalized,
                    pse_schema::catalog::expr_family::target_name(prefix, source.key.name)?,
                ),
            };
            Some(
                registry
                    .relation(&format!("{namespace}.{name}"))
                    .ok_or_else(|| malformed("undeclared mathematical family projection").into()),
            )
        })
        .collect()
}
fn engine(error: datafusion::common::DataFusionError) -> CompilerError {
    pse_rules::errmap::classify(error, pse_catalog::PlanOrigin::RuleCompiler).into()
}
