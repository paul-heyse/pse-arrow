// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! A transient Arrow tuple for the finite algorithm's heterogeneous outputs.
//! Each member is a list of its exact declared struct; ordinary UNNEST/projection
//! exposes individual relations. This is neither storage nor a model authority.
use crate::{RuleError, errmap::internal};
use datafusion::arrow::{
    array::{Array, ArrayRef, LargeListArray, RecordBatch, StructArray, UInt16Array, UInt32Array},
    buffer::OffsetBuffer,
    datatypes::{DataType, Field, Schema, SchemaRef},
};
use pse_catalog::session::SnapshotSession;
use pse_ids::owned_buffer::OwnedRecordBatch;
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::{Registry, model::RelationKey};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[derive(Debug)]
pub(super) struct Layout {
    pub(super) schema: SchemaRef,
    relations: Vec<(RelationKey, SchemaRef)>,
    derivations: Option<(RelationKey, SchemaRef)>,
}
pub(in crate::strata) struct Settled {
    pub(in crate::strata) relations: BTreeMap<RelationKey, FieldCheckedBatch>,
    pub(in crate::strata) derivations: Vec<RecordBatch>,
    pub(in crate::strata) rounds: BTreeMap<u16, u32>,
}
impl Layout {
    pub(super) fn new(
        registry: &Registry,
        outputs: &BTreeSet<RelationKey>,
    ) -> Result<Self, RuleError> {
        let relations = outputs
            .iter()
            .map(|key| {
                let spec = super::super::native_state::spec(registry, *key)?;
                let schema = pse_schema::arrow::relation_schema(registry, spec)
                    .map_err(|e| internal(e.to_string()))?;
                Ok((*key, Arc::new(schema)))
            })
            .collect::<Result<Vec<_>, RuleError>>()?;
        let derivations = registry
            .relation("provenance.derivations")
            .map(|spec| {
                pse_schema::arrow::relation_schema(registry, spec)
                    .map(|schema| (spec.key, Arc::new(schema)))
                    .map_err(|e| internal(e.to_string()))
            })
            .transpose()?;
        let mut fields = relations
            .iter()
            .chain(derivations.iter())
            .map(|(key, schema)| list_field(&key.qualified_name(), schema))
            .collect::<Vec<_>>();
        fields.push(list_field("rounds", &round_schema()));
        Ok(Self {
            schema: Arc::new(Schema::new(fields)),
            relations,
            derivations,
        })
    }
    pub(super) fn pack(&self, settled: &Settled) -> Result<RecordBatch, RuleError> {
        let mut columns = Vec::new();
        for (key, _) in &self.relations {
            let rows = settled
                .relations
                .get(key)
                .ok_or_else(|| internal("fixed-point result member absent"))?;
            columns.push(pack(rows.batch())?);
        }
        if let Some((_, schema)) = &self.derivations {
            let rows = match settled.derivations.as_slice() {
                [] => RecordBatch::new_empty(Arc::clone(schema)),
                [rows] => rows.clone(),
                _ => return Err(internal("fixed-point derivation output repeated")),
            };
            columns.push(pack(&rows)?);
        }
        let rounds = RecordBatch::try_new(
            round_schema(),
            vec![
                Arc::new(UInt16Array::from_iter_values(
                    settled.rounds.keys().copied(),
                )),
                Arc::new(UInt32Array::from_iter_values(
                    settled.rounds.values().copied(),
                )),
            ],
        )
        .map_err(|e| internal(e.to_string()))?;
        columns.push(pack(&rounds)?);
        RecordBatch::try_new(Arc::clone(&self.schema), columns).map_err(|e| internal(e.to_string()))
    }
    pub(super) fn unpack(
        &self,
        batch: &OwnedRecordBatch,
        session: &SnapshotSession,
    ) -> Result<Settled, RuleError> {
        if batch.num_rows() != 1 || batch.schema() != self.schema {
            return Err(internal("fixed-point result tuple differs"));
        }
        let mut relations = BTreeMap::new();
        for (index, (key, schema)) in self.relations.iter().enumerate() {
            let rows = unpack(batch.column(index), schema)?;
            let spec = super::super::native_state::spec(session.registry(), *key)?;
            relations.insert(
                *key,
                FieldCheckedBatch::admit(session.registry(), spec, rows)?,
            );
        }
        let derivations = self
            .derivations
            .as_ref()
            .map(|(_, schema)| unpack(batch.column(self.relations.len()), schema))
            .transpose()?
            .into_iter()
            .collect();
        let rounds = unpack(batch.column(batch.num_columns() - 1), &round_schema())?;
        let strata = rounds
            .column(0)
            .as_any()
            .downcast_ref::<UInt16Array>()
            .ok_or_else(|| internal("stratum type differs"))?;
        let counts = rounds
            .column(1)
            .as_any()
            .downcast_ref::<UInt32Array>()
            .ok_or_else(|| internal("round type differs"))?;
        let mut result = BTreeMap::new();
        for row in 0..rounds.num_rows() {
            if result
                .insert(strata.value(row), counts.value(row))
                .is_some()
            {
                return Err(internal("stratum repeated"));
            }
        }
        Ok(Settled {
            relations,
            derivations,
            rounds: result,
        })
    }
}
fn list_field(name: &str, schema: &SchemaRef) -> Field {
    Field::new(
        name,
        DataType::LargeList(Arc::new(Field::new(
            "item",
            DataType::Struct(schema.fields().clone()),
            false,
        ))),
        false,
    )
}
fn round_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("stratum", DataType::UInt16, false),
        Field::new("rounds", DataType::UInt32, false),
    ]))
}
fn pack(rows: &RecordBatch) -> Result<ArrayRef, RuleError> {
    let values = StructArray::try_new(
        rows.schema().fields().clone(),
        rows.columns().to_vec(),
        None,
    )
    .map_err(|e| internal(e.to_string()))?;
    let size = i64::try_from(rows.num_rows())
        .map_err(|_| internal("fixed-point output exceeds Arrow list extent"))?;
    Ok(Arc::new(
        LargeListArray::try_new(
            Arc::new(Field::new("item", values.data_type().clone(), false)),
            OffsetBuffer::new(vec![0_i64, size].into()),
            Arc::new(values),
            None,
        )
        .map_err(|e| internal(e.to_string()))?,
    ))
}
fn unpack(column: &ArrayRef, schema: &SchemaRef) -> Result<RecordBatch, RuleError> {
    let list = column
        .as_any()
        .downcast_ref::<LargeListArray>()
        .filter(|v| v.len() == 1 && v.null_count() == 0)
        .ok_or_else(|| internal("invalid fixed-point member list"))?;
    let values = list.value(0);
    let rows = values
        .as_any()
        .downcast_ref::<StructArray>()
        .filter(|v| v.null_count() == 0)
        .ok_or_else(|| internal("invalid fixed-point member struct"))?;
    RecordBatch::try_new(Arc::clone(schema), rows.columns().to_vec())
        .map_err(|e| internal(e.to_string()))
}
