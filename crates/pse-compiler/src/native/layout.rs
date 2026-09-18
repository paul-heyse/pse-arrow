// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One transient tuple exposes heterogeneous results through ordinary native UNNEST.
use crate::{AlgorithmOutput, CompilerError, passes::invalid};
use datafusion::{
    arrow::{
        array::{Array, ArrayRef, LargeListArray, RecordBatch, StructArray},
        buffer::OffsetBuffer,
        datatypes::{DataType, Field, Schema, SchemaRef},
    },
    common::{Column, NullHandling, UnnestOptions},
    logical_expr::{Expr, LogicalPlan, LogicalPlanBuilder, Projection},
};
use pse_catalog::session::RelationPlan;
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::{Registry, model::AlgorithmSpec};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug)]
pub(super) struct Layout {
    pub schema: SchemaRef,
    members: BTreeMap<String, (pse_ids::SemanticId, SchemaRef)>,
}
impl Layout {
    pub(super) fn new(registry: &Registry, spec: &AlgorithmSpec) -> Result<Self, CompilerError> {
        let mut members = BTreeMap::new();
        for (port, relation) in spec
            .outputs
            .iter()
            .map(|output| (output.port.as_str(), output.relation.as_str()))
            .chain([
                ("__findings", "runtime.diagnostics_findings"),
                ("__derivations", "provenance.derivations"),
            ])
        {
            let spec = registry
                .relation(relation)
                .ok_or_else(|| invalid("algorithm result declaration absent"))?;
            let schema = Arc::new(pse_schema::arrow::relation_schema(registry, spec)?);
            if members.insert(port.to_owned(), (spec.id, schema)).is_some() {
                return Err(invalid("algorithm output name repeated"));
            }
        }
        let fields = members
            .iter()
            .map(|(name, (_, schema))| {
                Field::new(
                    name,
                    DataType::LargeList(Arc::new(Field::new(
                        "item",
                        DataType::Struct(schema.fields().clone()),
                        false,
                    ))),
                    false,
                )
            })
            .collect::<Vec<_>>();
        Ok(Self {
            schema: Arc::new(Schema::new(fields)),
            members,
        })
    }
    pub(super) fn unpack_plans(
        &self,
        tuple: &LogicalPlan,
        registry: &Registry,
    ) -> Result<BTreeMap<String, RelationPlan>, CompilerError> {
        self.members
            .iter()
            .map(|(name, (relation, schema))| {
                // These columns are already resolved by this exact tuple layout.
                // Native Projection checks their fields without the SQL builder's
                // repeated USING-column search through the whole producer graph.
                let plan = LogicalPlan::Projection(
                    Projection::try_new(
                        vec![Expr::Column(Column::from_name(name))],
                        Arc::new(tuple.clone()),
                    )
                    .map_err(engine)?,
                );
                let plan = LogicalPlanBuilder::from(plan)
                    .unnest_column_with_options(
                        Column::from_name(name),
                        UnnestOptions::new().with_null_handling(NullHandling::Drop),
                    )
                    .map_err(engine)?
                    .unnest_column(Column::from_name(name))
                    .map_err(engine)?
                    .build()
                    .map_err(engine)?;
                let expressions = schema
                    .fields()
                    .iter()
                    .map(|field| {
                        Expr::Column(Column::from_name(format!("{name}.{}", field.name())))
                            .alias(field.name())
                    })
                    .collect();
                let plan = LogicalPlan::Projection(
                    Projection::try_new(expressions, Arc::new(plan)).map_err(engine)?,
                );
                Ok((
                    name.clone(),
                    RelationPlan::derived(*relation, plan, registry)?,
                ))
            })
            .collect()
    }
    pub(super) fn member(
        &self,
        name: &str,
        tuple: &RecordBatch,
        registry: &Registry,
    ) -> Result<FieldCheckedBatch, CompilerError> {
        if tuple.num_rows() != 1 || tuple.schema() != self.schema {
            return Err(invalid("algorithm result tuple differs"));
        }
        let (relation, schema) = self
            .members
            .get(name)
            .ok_or_else(|| invalid("algorithm tuple member undeclared"))?;
        let column = tuple
            .column_by_name(name)
            .ok_or_else(|| invalid("algorithm tuple member absent"))?;
        let list = column
            .as_any()
            .downcast_ref::<LargeListArray>()
            .filter(|list| list.null_count() == 0)
            .ok_or_else(|| invalid("algorithm tuple list invalid"))?;
        let value = list.value(0);
        let value = value
            .as_any()
            .downcast_ref::<StructArray>()
            .filter(|value| value.null_count() == 0)
            .ok_or_else(|| invalid("algorithm tuple struct invalid"))?;
        let rows = RecordBatch::try_new(Arc::clone(schema), value.columns().to_vec())
            .map_err(|error| invalid(error.to_string()))?;
        let spec = registry
            .relation_by_id(*relation)
            .ok_or_else(|| invalid("tuple relation undeclared"))?;
        Ok(FieldCheckedBatch::admit(registry, spec, rows)?)
    }
    pub(super) fn pack(
        &self,
        mut output: AlgorithmOutput,
        session: &pse_catalog::session::SnapshotSession,
        cancel: &pse_ids::CancellationToken,
    ) -> Result<RecordBatch, CompilerError> {
        let mut columns = Vec::with_capacity(self.members.len());
        for (name, (relation, schema)) in &self.members {
            let spec = session
                .registry()
                .relation_by_id(*relation)
                .ok_or_else(|| invalid("algorithm result declaration absent"))?;
            let checked = match name.as_str() {
                "__findings" | "__derivations" => {
                    let batches = if name == "__findings" {
                        &output.findings
                    } else {
                        &output.derivations
                    };
                    let batches = batches
                        .iter()
                        .map(|batch| {
                            FieldCheckedBatch::admit(session.registry(), spec, batch.clone())
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    FieldCheckedBatch::concat_reserved(
                        session.registry(),
                        spec,
                        &batches,
                        session.reserver(),
                        cancel,
                    )?
                }
                _ => output
                    .outputs
                    .remove(name)
                    .ok_or_else(|| invalid(format!("algorithm omitted result {name}")))?,
            };
            checked.check_declaration(session.registry(), spec)?;
            if checked.batch().schema().fields() != schema.fields() {
                return Err(invalid("algorithm output fields differ"));
            }
            columns.push(pack(checked.batch())?);
        }
        if !output.outputs.is_empty() {
            return Err(invalid("algorithm returned undeclared outputs"));
        }
        RecordBatch::try_new(Arc::clone(&self.schema), columns)
            .map_err(|error| invalid(error.to_string()))
    }
}
fn pack(rows: &RecordBatch) -> Result<ArrayRef, CompilerError> {
    let values = StructArray::try_new(
        rows.schema().fields().clone(),
        rows.columns().to_vec(),
        None,
    )
    .map_err(|error| invalid(error.to_string()))?;
    let size = i64::try_from(rows.num_rows())
        .map_err(|_| invalid("algorithm output exceeds Arrow list extent"))?;
    Ok(Arc::new(
        LargeListArray::try_new(
            Arc::new(Field::new("item", values.data_type().clone(), false)),
            OffsetBuffer::new(vec![0_i64, size].into()),
            Arc::new(values),
            None,
        )
        .map_err(|error| invalid(error.to_string()))?,
    ))
}
use crate::passes::native_rows::engine;
