// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source lists are columns of the same immutable rows as algorithm values.
use super::{CompilerError, SourceKey, invalid};
use datafusion::arrow::{
    array::{
        Array, ArrayRef, FixedSizeBinaryArray, ListArray, RecordBatch, StringArray, StructArray,
        builder::{FixedSizeBinaryBuilder, ListBuilder, StringBuilder, StructBuilder},
    },
    datatypes::{DataType, Schema},
};
use pse_ids::{CancellationToken, MemoryReserver, owned_buffer::OwnedRecordBatch};
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::{
    Registry,
    model::{FieldContract, RelationSpec},
};
use std::{collections::BTreeSet, sync::Arc};

pub(crate) use pse_schema::model::field::SOURCE_SUPPORT_COLUMN as COLUMN;

pub(super) struct Builder(ListBuilder<StructBuilder>);

impl Builder {
    pub(super) fn new(registry: &Registry) -> Result<Self, CompilerError> {
        let field = pse_schema::arrow::field_for(registry, &FieldContract::source_support())
            .map_err(pse_relations::RelationError::from)?;
        let DataType::List(child) = field.data_type() else {
            return Err(invalid("source support declaration is not a list"));
        };
        let DataType::Struct(fields) = child.data_type() else {
            return Err(invalid("source support member declaration is not a struct"));
        };
        Ok(Self(
            ListBuilder::with_capacity(StructBuilder::from_fields(fields.clone(), 0), 0)
                .with_field(Arc::clone(child)),
        ))
    }

    pub(super) fn push(
        &mut self,
        sources: &BTreeSet<SourceKey>,
        registry: &Registry,
    ) -> Result<(), CompilerError> {
        if sources.is_empty() {
            return Err(invalid("algorithm output has no actual source occurrence"));
        }
        for source in sources {
            let relation = registry
                .relation_by_key(source.relation)
                .ok_or_else(|| invalid("source support relation absent"))?;
            let values = self.0.values();
            values
                .field_builder::<StringBuilder>(0)
                .ok_or_else(|| invalid("source support port is not text"))?
                .append_value(&source.port);
            values
                .field_builder::<FixedSizeBinaryBuilder>(1)
                .ok_or_else(|| invalid("source support identity is not fixed binary"))?
                .append_value(relation.id.as_bytes())
                .map_err(pse_relations::RelationError::from)?;
            values
                .field_builder::<FixedSizeBinaryBuilder>(2)
                .ok_or_else(|| invalid("source support key is not fixed binary"))?
                .append_value(source.key.as_bytes())
                .map_err(pse_relations::RelationError::from)?;
            values.append(true);
        }
        self.0.append(true);
        Ok(())
    }

    pub(super) fn finish(mut self) -> ArrayRef {
        Arc::new(self.0.finish())
    }
}

#[derive(Clone)]
pub(crate) struct SupportedBatch {
    pub(crate) data: OwnedRecordBatch,
}

impl SupportedBatch {
    pub(super) fn new(
        input: &FieldCheckedBatch,
        support: ArrayRef,
        registry: &Registry,
        reserver: &dyn MemoryReserver,
        cancel: &CancellationToken,
    ) -> Result<Self, CompilerError> {
        if support.len() != input.batch().num_rows()
            || input.batch().schema().column_with_name(COLUMN).is_some()
        {
            return Err(invalid(
                "source support must be one co-located list per algorithm row",
            ));
        }
        let mut fields = input.batch().schema().fields().to_vec();
        fields.push(Arc::new(
            pse_schema::arrow::field_for(registry, &FieldContract::source_support())
                .map_err(pse_relations::RelationError::from)?,
        ));
        let mut columns = input.batch().columns().to_vec();
        columns.push(support);
        let batch = RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
            .map_err(pse_relations::RelationError::from)?;
        Ok(Self {
            data: OwnedRecordBatch::export(batch, reserver, cancel)?,
        })
    }

    pub(crate) fn payload(
        &self,
        registry: &Registry,
        spec: &RelationSpec,
    ) -> Result<FieldCheckedBatch, CompilerError> {
        Ok(FieldCheckedBatch::admit_owned_projection(
            registry,
            spec,
            &self.data,
            &(0..spec.columns.len()).collect::<Vec<_>>(),
        )?)
    }
}

/// Read association data from the same row as the algorithm value. Native joins
/// still establish membership against the actual immutable source owners.
pub(crate) fn row_sources(
    batch: &RecordBatch,
    row: usize,
    registry: &Registry,
) -> Result<BTreeSet<SourceKey>, CompilerError> {
    let lists = batch
        .column_by_name(COLUMN)
        .and_then(|array| array.as_any().downcast_ref::<ListArray>())
        .ok_or_else(|| invalid("algorithm support is not a list"))?;
    if row >= lists.len() || lists.is_null(row) || lists.value_length(row) == 0 {
        return Err(invalid("algorithm row has no source support"));
    }
    let values = lists.value(row);
    let values = values
        .as_any()
        .downcast_ref::<StructArray>()
        .ok_or_else(|| invalid("algorithm support member is not a struct"))?;
    let ports = values
        .column_by_name("source_port")
        .and_then(|array| array.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| invalid("algorithm source port is not text"))?;
    let relations = values
        .column_by_name("source_relation_id")
        .and_then(|array| array.as_any().downcast_ref::<FixedSizeBinaryArray>())
        .ok_or_else(|| invalid("algorithm source relation is not an identity"))?;
    let keys = values
        .column_by_name("source_key")
        .and_then(|array| array.as_any().downcast_ref::<FixedSizeBinaryArray>())
        .ok_or_else(|| invalid("algorithm source key is not fixed binary"))?;
    (0..values.len())
        .map(|index| {
            if values.is_null(index)
                || ports.is_null(index)
                || relations.is_null(index)
                || keys.is_null(index)
                || ports.value(index).is_empty()
            {
                return Err(invalid("algorithm source member is incomplete"));
            }
            let id = pse_ids::SemanticId::from_bytes(
                relations
                    .value(index)
                    .try_into()
                    .map_err(|_| invalid("algorithm source identity width differs"))?,
            );
            Ok(SourceKey {
                relation: registry
                    .relation_by_id(id)
                    .ok_or_else(|| invalid("algorithm source relation is undeclared"))?
                    .key,
                port: ports.value(index).to_owned(),
                key: crate::passes::native_rows::key_value(keys.value(index))?,
            })
        })
        .collect()
}
