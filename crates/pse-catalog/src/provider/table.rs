// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Read-only tables over fully admitted snapshots, with truthful filter pushdown.

use super::BoxFut;
use crate::{CatalogError, LoadedRelation, Snapshot};
use datafusion::arrow::datatypes::SchemaRef;
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::catalog::{Session, TableProvider};
use datafusion::common::{Constraints, DFSchema, DataFusionError, Result, Statistics};
use datafusion::datasource::memory::MemorySourceConfig;
use datafusion::logical_expr::utils::conjunction;
use datafusion::logical_expr::{Expr, TableProviderFilterPushDown, TableType};
use datafusion::physical_plan::ExecutionPlan;
use datafusion::physical_plan::filter::FilterExecBuilder;
use datafusion::physical_plan::limit::GlobalLimitExec;
use datafusion_catalog::{ScanArgs, ScanResult};
use pse_ids::SemanticId;
use pse_schema::Registry;
use std::collections::BTreeSet;
use std::sync::Arc;

/// A provider whose primitive and semantic checks ran before its snapshot was minted.
#[derive(Debug)]
pub struct RelationTable {
    snapshot: Arc<Snapshot>,
    relation: Arc<LoadedRelation>,
    constraints: Constraints,
}
impl RelationTable {
    /// Resolve and recheck the actual member and schema, without trusting a digest.
    ///
    /// # Errors
    /// Missing or ambiguous membership, incompatible declarations, or invalid values.
    pub fn new(
        snapshot: Arc<Snapshot>,
        relation_id: SemanticId,
        registry: &Registry,
    ) -> Result<Self, CatalogError> {
        let spec = registry
            .relation_by_id(relation_id)
            .ok_or_else(|| invalid("unknown relation"))?;
        let relation = snapshot
            .relation(spec.key.namespace.as_str(), spec.key.name)
            .cloned()
            .ok_or_else(|| invalid("snapshot has no uniquely named member; repeated stage outputs require an explicit port binding"))?;
        Self::from_member(snapshot, relation, registry)
    }
    /// Bind one exact admitted output port, including repeated relation declarations.
    /// # Errors
    /// Missing port or a registry different from the actual admission owner.
    pub fn from_port(
        snapshot: Arc<Snapshot>,
        port: &str,
        registry: &Registry,
    ) -> Result<Self, CatalogError> {
        let relation = snapshot
            .relation_port(port)
            .cloned()
            .ok_or_else(|| invalid("snapshot port is absent"))?;
        Self::from_member(snapshot, relation, registry)
    }
    fn from_member(
        snapshot: Arc<Snapshot>,
        relation: Arc<LoadedRelation>,
        registry: &Registry,
    ) -> Result<Self, CatalogError> {
        let spec = registry
            .relation_by_id(relation.contract().canonical.relation_id)
            .ok_or_else(|| invalid("member declaration is absent"))?;
        if !std::ptr::eq(snapshot.admission.registry.as_ref(), registry) {
            return Err(invalid(
                "provider registry differs from the actual admission context",
            ));
        }
        relation
            .contract()
            .validate_against_registry(registry, spec)?;
        let constraints = relation.contract().constraints();
        Ok(Self {
            snapshot,
            relation,
            constraints,
        })
    }
    /// The immutable handle establishing this table's admission context.
    pub fn snapshot(&self) -> &Arc<Snapshot> {
        &self.snapshot
    }
    /// The actual admitted relation, not a manifest-only description.
    pub fn relation(&self) -> &Arc<LoadedRelation> {
        &self.relation
    }

    fn scan_plan(
        &self,
        state: &dyn Session,
        projection: Option<&[usize]>,
        filters: &[Expr],
        limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        if filters
            .iter()
            .any(|filter| !super::pushdown::supported(filter, self.relation.contract()))
        {
            return Err(DataFusionError::Plan(
                "unsupported predicate was sent to exact provider".to_owned(),
            ));
        }
        native_scan(
            &crate::session::query_schema::batch(self.relation.batch())?,
            &self.constraints,
            state,
            projection,
            filters,
            limit,
        )
    }
}
impl TableProvider for RelationTable {
    fn schema(&self) -> SchemaRef {
        crate::session::query_schema::schema(self.relation.batch().schema().as_ref())
    }
    fn table_type(&self) -> TableType {
        TableType::Base
    }
    fn constraints(&self) -> Option<&Constraints> {
        Some(&self.constraints)
    }
    fn statistics(&self) -> Option<Statistics> {
        Some(super::statistics::cached(&self.relation))
    }
    fn supports_filters_pushdown(
        &self,
        filters: &[&Expr],
    ) -> Result<Vec<TableProviderFilterPushDown>> {
        Ok(filters
            .iter()
            .map(|filter| {
                if super::pushdown::supported(filter, self.relation.contract()) {
                    TableProviderFilterPushDown::Exact
                } else {
                    TableProviderFilterPushDown::Unsupported
                }
            })
            .collect())
    }
    fn scan<'s, 't, 'p, 'f, 'future>(
        &'s self,
        state: &'t dyn Session,
        projection: Option<&'p Vec<usize>>,
        filters: &'f [Expr],
        limit: Option<usize>,
    ) -> BoxFut<'future, Result<Arc<dyn ExecutionPlan>>>
    where
        's: 'future,
        't: 'future,
        'p: 'future,
        'f: 'future,
        Self: 'future,
    {
        Box::pin(
            async move { self.scan_plan(state, projection.map(Vec::as_slice), filters, limit) },
        )
    }
    fn scan_with_args<'a, 's, 't, 'future>(
        &'s self,
        state: &'t dyn Session,
        args: ScanArgs<'a>,
    ) -> BoxFut<'future, Result<ScanResult>>
    where
        'a: 'future,
        's: 'future,
        't: 'future,
        Self: 'future,
    {
        Box::pin(async move {
            Ok(ScanResult::new(self.scan_plan(
                state,
                args.projection(),
                args.filters().unwrap_or(&[]),
                args.limit(),
            )?))
        })
    }
}
fn invalid(reason: &str) -> CatalogError {
    CatalogError::ConfigInvalid {
        key: "provider.admission".to_owned(),
        reason: reason.to_owned(),
    }
}

/// Construct native operators without evaluating any input rows. The source retains
/// admitted Arrow buffer owners; native filter allocations follow DataFusion's own
/// accounting scope, rather than a second PSE filtering implementation.
fn native_scan(
    batch: &RecordBatch,
    constraints: &Constraints,
    state: &dyn Session,
    projection: Option<&[usize]>,
    filters: &[Expr],
    limit: Option<usize>,
) -> Result<Arc<dyn ExecutionPlan>> {
    let predicate = conjunction(filters.iter().cloned());
    let source_projection = required_columns(batch, projection, predicate.as_ref())?;
    let source_constraints = constraints.project(&source_projection).unwrap_or_default();
    let source = MemorySourceConfig::try_new_exec(
        &[vec![batch.clone()]],
        batch.schema(),
        Some(source_projection.clone()),
    )?;
    let mut plan: Arc<dyn ExecutionPlan> =
        Arc::new(Arc::unwrap_or_clone(source).with_constraints(source_constraints));
    if let Some(predicate) = predicate {
        let schema = DFSchema::try_from(plan.schema().as_ref().clone())?;
        let physical = state.create_physical_expr(predicate, &schema)?;
        let output_projection = projection
            .map(|indices| {
                indices
                    .iter()
                    .map(|index| {
                        source_projection.binary_search(index).map_err(|_| {
                            DataFusionError::Internal(
                                "required output column is absent from scan".to_owned(),
                            )
                        })
                    })
                    .collect::<Result<Vec<_>>>()
            })
            .transpose()?;
        plan = Arc::new(
            FilterExecBuilder::new(physical, plan)
                .with_batch_size(state.config_options().execution.batch_size.into())
                .with_default_selectivity(
                    state.config_options().optimizer.default_filter_selectivity,
                )
                .apply_projection(output_projection)?
                .build()?,
        );
    }
    if let Some(limit) = limit {
        plan = Arc::new(GlobalLimitExec::new(plan, 0, Some(limit)));
    }
    Ok(plan)
}

fn required_columns(
    batch: &RecordBatch,
    projection: Option<&[usize]>,
    predicate: Option<&Expr>,
) -> Result<Vec<usize>> {
    // With no filter, keep the requested order and repeated output columns exactly.
    let Some(predicate) = predicate else {
        return Ok(projection.map_or_else(|| (0..batch.num_columns()).collect(), <[usize]>::to_vec));
    };
    let mut required: BTreeSet<_> = projection.map_or_else(
        || (0..batch.num_columns()).collect(),
        |indices| indices.iter().copied().collect(),
    );
    for column in predicate.column_refs() {
        required.insert(batch.schema().index_of(column.name())?);
    }
    Ok(required.into_iter().collect())
}

#[cfg(test)]
mod tests;
