// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual immutable row ordinals captured before native planning or partitioning.

use super::SnapshotSession;
use crate::provider::binding::{BindingKey, TableBinding};
use crate::{BoxFut, CatalogError};
use datafusion::{
    arrow::{
        array::{RecordBatch, UInt64Array},
        datatypes::{Schema, SchemaRef},
    },
    catalog::{Session, TableProvider},
    common::{Result, TableReference},
    datasource::memory::MemorySourceConfig,
    logical_expr::{Expr, TableType},
    physical_plan::ExecutionPlan,
};
use pse_ids::{CancellationToken, ReservationLease};
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::model::FieldContract;
use std::sync::Arc;

/// Only the checked-input plus ordinal constructor below can create this provider.
#[derive(Debug)]
pub(super) struct IndexedTable {
    batch: RecordBatch,
    _input: FieldCheckedBatch,
}
impl TableProvider for IndexedTable {
    fn schema(&self) -> SchemaRef {
        self.batch.schema()
    }
    fn table_type(&self) -> TableType {
        TableType::Temporary
    }
    fn scan<'s, 't, 'p, 'f, 'future>(
        &'s self,
        _state: &'t dyn Session,
        projection: Option<&'p Vec<usize>>,
        _filters: &'f [Expr],
        limit: Option<usize>,
    ) -> BoxFut<'future, Result<Arc<dyn ExecutionPlan>>>
    where
        's: 'future,
        't: 'future,
        'p: 'future,
        'f: 'future,
        Self: 'future,
    {
        Box::pin(async move {
            let batch = limit.map_or_else(
                || self.batch.clone(),
                |limit| self.batch.slice(0, limit.min(self.batch.num_rows())),
            );
            let plan: Arc<dyn ExecutionPlan> = MemorySourceConfig::try_new_exec(
                &[vec![batch]],
                self.schema(),
                projection.cloned(),
            )?;
            Ok(plan)
        })
    }
}
impl SnapshotSession {
    /// Capture the actual checked Arrow row position alongside each immutable row.
    /// The ordinal comes from the declared transient occurrence field. It is built
    /// before any native query can reorder or repartition these rows, so a subsequent
    /// native join can bind a specialized algorithm's exact occurrence witnesses.
    /// This temporary source advertises no relation keys or producer-validity claims.
    /// Use `scan_computation_role` to scan the resulting role.
    /// # Errors
    /// Missing/different declarations, duplicate role/ordinal field, capacity,
    /// cancellation, shared reservation refusal, or Arrow construction failure.
    pub fn with_indexed_checked_role(
        &self,
        role: impl Into<String>,
        input: &FieldCheckedBatch,
        cancel: &CancellationToken,
    ) -> Result<Self, CatalogError> {
        let role = role.into();
        if role.is_empty() || self.bindings.computation(&role).is_some() {
            return Err(invalid("indexed role is empty or already bound"));
        }
        let spec = self
            .registry
            .relation_by_id(input.relation_id())
            .ok_or_else(|| invalid("indexed input relation is undeclared"))?;
        input.check_declaration(&self.registry, spec)?;
        let ordinal = self
            .registry
            .relation("provenance.algorithm_source_occurrences")
            .and_then(|spec| spec.column("constructed_row_ordinal"))
            .ok_or_else(|| invalid("algorithm occurrence ordinal is undeclared"))?;
        if ordinal.value_type()
            != FieldContract::native(datafusion::arrow::datatypes::DataType::UInt64)
            || ordinal.nullable()
        {
            return Err(invalid(
                "algorithm occurrence ordinal must be nonnull UInt64",
            ));
        }
        if input
            .batch()
            .schema()
            .column_with_name(ordinal.name())
            .is_some()
        {
            return Err(invalid(
                "indexed input already contains the occurrence ordinal",
            ));
        }
        cancel.checkpoint()?;
        let input = input.retained(self.reserver.as_ref(), cancel)?;
        let rows = input.batch().num_rows();
        let end = u64::try_from(rows)
            .map_err(|_| invalid("algorithm occurrence count exceeds UInt64"))?;
        let extent = rows
            .checked_mul(size_of::<u64>())
            .and_then(|value| value.checked_add(128))
            .ok_or_else(|| invalid("algorithm occurrence capacity overflow"))?;
        let mut reservation = self.reserver.open("algorithm:row-ordinal-column");
        reservation.try_grow(extent)?;
        cancel.checkpoint()?;
        let field = pse_schema::arrow::field_for(&self.registry, ordinal)
            .map_err(pse_relations::RelationError::from)?;
        let ordinal_schema = Arc::new(Schema::new(vec![field.clone()]));
        let ordinal_batch = RecordBatch::try_new(
            ordinal_schema,
            vec![Arc::new(UInt64Array::from_iter_values(0..end))],
        )
        .map_err(pse_relations::RelationError::from)?;
        let ordinal_batch = pse_ids::owned_buffer::attach_reservation(
            ordinal_batch,
            ReservationLease::new(reservation),
        )?;
        let mut fields = input.batch().schema().fields().to_vec();
        fields.push(Arc::new(field));
        let mut columns = input.batch().columns().to_vec();
        columns.push(Arc::clone(ordinal_batch.column(0)));
        let batch = RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
            .map_err(pse_relations::RelationError::from)?;
        let table: Arc<dyn TableProvider> = Arc::new(IndexedTable {
            batch,
            _input: input,
        });
        let mut result = self.clone();
        result
            .bindings
            .insert(
                BindingKey::Computation(role.clone()),
                TableBinding::new(
                    TableReference::full("arguments", "indexed", role),
                    table,
                    None,
                    None,
                ),
            )
            .map_err(super::snapshot_session::engine)?;
        Ok(result)
    }
}
fn invalid(reason: &str) -> CatalogError {
    CatalogError::Admission {
        path: "session.indexed_argument".to_owned(),
        reason: reason.to_owned(),
    }
}
