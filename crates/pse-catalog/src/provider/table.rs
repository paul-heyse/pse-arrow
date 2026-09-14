// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Read-only tables over fully admitted snapshots, with truthful filter pushdown.

use super::BoxFut;
use crate::{CatalogError, LoadedRelation, Snapshot};
use datafusion::arrow::array::BooleanArray;
use datafusion::arrow::compute::filter_record_batch;
use datafusion::arrow::datatypes::SchemaRef;
use datafusion::catalog::{Session, TableProvider};
use datafusion::common::{Constraints, DFSchema, DataFusionError, Result, Statistics};
use datafusion::datasource::memory::MemorySourceConfig;
use datafusion::logical_expr::{Expr, TableProviderFilterPushDown, TableType};
use datafusion::physical_plan::ExecutionPlan;
use datafusion_catalog::{ScanArgs, ScanResult};
use pse_ids::{MemoryReserver, ReservationLease, SemanticId};
use pse_schema::Registry;
use std::sync::Arc;

/// A provider whose primitive and semantic checks ran before its snapshot was minted.
#[derive(Debug)]
pub struct RelationTable {
    snapshot: Arc<Snapshot>,
    relation: Arc<LoadedRelation>,
    constraints: Constraints,
    reserver: Arc<dyn MemoryReserver>,
    pushdown: bool,
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
        reserver: Arc<dyn MemoryReserver>,
    ) -> Result<Self, CatalogError> {
        let spec = registry
            .relation_by_id(relation_id)
            .ok_or_else(|| invalid("unknown relation"))?;
        let relation = snapshot
            .relation(spec.key.namespace.as_str(), spec.key.name)
            .cloned()
            .ok_or_else(|| invalid("snapshot has no uniquely named member; repeated stage outputs require an explicit port binding"))?;
        let mut validation = reserver.open("provider:semantic-admission");
        validation.try_grow(crate::store::membership::validation_extent(
            relation.batch(),
        )?)?;
        relation
            .contract()
            .validate_against_registry(registry, spec)?;
        pse_relations::validate::validate_batch(registry, spec, relation.batch())
            .map_err(|errors| invalid(&format!("invalid table rows: {errors:?}")))?;
        let constraints = relation.contract().constraints();
        Ok(Self {
            snapshot,
            relation,
            constraints,
            reserver,
            pushdown: true,
        })
    }
    /// Disable source filtering for the independent completeness oracle.
    #[must_use]
    pub const fn without_pushdown(mut self) -> Self {
        self.pushdown = false;
        self
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
        if filters.iter().any(|filter| {
            !self.pushdown || !super::pushdown::supported(filter, self.relation.contract())
        }) {
            return Err(DataFusionError::Plan(
                "unsupported predicate was sent to exact provider".to_owned(),
            ));
        }
        let mut batch = self.relation.batch().clone();
        for filter in filters {
            let extent = pse_ids::owned_buffer::retained_buffer_bytes(&batch).map_err(external)?;
            let envelope = extent
                .checked_mul(4)
                .and_then(|n| {
                    batch
                        .num_rows()
                        .checked_mul(64)
                        .and_then(|rows| n.checked_add(rows))
                })
                .and_then(|n| n.checked_add(4096))
                .ok_or_else(|| {
                    DataFusionError::ResourcesExhausted("filter extent overflow".to_owned())
                })?;
            let mut reservation = self.reserver.open("provider:exact-filter");
            reservation.try_grow(envelope).map_err(external)?;
            let schema = DFSchema::try_from(batch.schema().as_ref().clone())?;
            let physical = state.create_physical_expr(filter.clone(), &schema)?;
            let values = physical.evaluate(&batch)?.into_array(batch.num_rows())?;
            let predicate = values
                .as_any()
                .downcast_ref::<BooleanArray>()
                .ok_or_else(|| {
                    DataFusionError::Plan("filter did not produce Boolean storage".to_owned())
                })?;
            let filtered = filter_record_batch(&batch, predicate)?;
            drop(values);
            drop(physical);
            let retained =
                pse_ids::owned_buffer::retained_buffer_bytes(&filtered).map_err(external)?;
            reservation.shrink(reservation.size().saturating_sub(retained));
            let lease = ReservationLease::new(reservation);
            batch = pse_ids::owned_buffer::attach_reservation(filtered, lease).map_err(external)?;
        }
        if let Some(limit) = limit {
            batch = batch.slice(0, limit.min(batch.num_rows()));
        }
        let projection = projection.map(<[usize]>::to_vec);
        let constraints = projection.as_ref().map_or_else(
            || Some(self.constraints.clone()),
            |indices| self.constraints.project(indices),
        );
        let source = MemorySourceConfig::try_new_exec(&[vec![batch]], self.schema(), projection)?;
        let source = Arc::unwrap_or_clone(source).with_constraints(constraints.unwrap_or_default());
        Ok(Arc::new(source))
    }
}
impl TableProvider for RelationTable {
    fn schema(&self) -> SchemaRef {
        self.relation.batch().schema()
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
                if self.pushdown && super::pushdown::supported(filter, self.relation.contract()) {
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
fn external(error: impl std::error::Error + Send + Sync + 'static) -> DataFusionError {
    DataFusionError::External(Box::new(error))
}
