// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Verified recorded fields retained by an actual immutable read provider.
use datafusion::{
    arrow::datatypes::{Field, SchemaRef},
    catalog::{Session, TableProvider},
    common::Result,
    logical_expr::{Expr, TableProviderFilterPushDown, TableType},
    physical_plan::ExecutionPlan,
};
use pse_schema::compatibility::VerifiedRecordedContract;
use std::sync::Arc;

/// An actual provider with independently verified recorded fields. It offers no writes.
#[derive(Debug)]
pub struct RecordedProvider {
    inner: Arc<dyn TableProvider>,
    fields: Arc<[Arc<Field>]>,
}
impl RecordedProvider {
    /// Verify the provider's actual schema before retaining its original field environment.
    /// # Errors
    /// Recorded meaning/layout and the provider's fields disagree.
    pub fn new(
        inner: Arc<dyn TableProvider>,
        recorded: &VerifiedRecordedContract,
        relation: pse_ids::SemanticId,
    ) -> Result<Self> {
        let schema = inner.schema();
        let encoding =
            pse_schema::fingerprint::encoding_fields(schema.fields().iter().map(AsRef::as_ref))
                .map_err(pse_columnar::external)?;
        recorded
            .verify_fields(relation, &schema, encoding)
            .map_err(pse_columnar::external)?;
        let mut pending: Vec<_> = schema.fields().to_vec();
        let mut fields = Vec::new();
        while let Some(field) = pending.pop() {
            match field.data_type() {
                datafusion::arrow::datatypes::DataType::List(child)
                | datafusion::arrow::datatypes::DataType::LargeList(child)
                | datafusion::arrow::datatypes::DataType::ListView(child)
                | datafusion::arrow::datatypes::DataType::LargeListView(child)
                | datafusion::arrow::datatypes::DataType::FixedSizeList(child, _)
                | datafusion::arrow::datatypes::DataType::Map(child, _) => {
                    pending.push(child.clone())
                }
                datafusion::arrow::datatypes::DataType::Struct(children) => {
                    pending.extend(children.iter().cloned())
                }
                datafusion::arrow::datatypes::DataType::Union(children, _) => {
                    pending.extend(children.iter().map(|(_, child)| child.clone()))
                }
                datafusion::arrow::datatypes::DataType::RunEndEncoded(runs, values) => {
                    pending.push(runs.clone());
                    pending.push(values.clone());
                }
                _ => {}
            }
            fields.push(field);
        }
        Ok(Self {
            inner,
            fields: fields.into(),
        })
    }
    pub(crate) fn fields(&self) -> Arc<[Arc<Field>]> {
        self.fields.clone()
    }
}
/// Accept only an observed recorded field's meaning and physical representation.
/// Aliasing and native nullability inference affect occurrences, not the value domain.
pub(crate) fn admits(fields: &[Arc<Field>], field: &Field) -> bool {
    fields.iter().any(|observed| {
        observed
            .as_ref()
            .clone()
            .with_name(field.name())
            .with_nullable(field.is_nullable())
            == *field
    })
}
#[async_trait::async_trait]
impl TableProvider for RecordedProvider {
    fn schema(&self) -> SchemaRef {
        self.inner.schema()
    }
    fn table_type(&self) -> TableType {
        self.inner.table_type()
    }
    fn supports_filters_pushdown(
        &self,
        filters: &[&Expr],
    ) -> Result<Vec<TableProviderFilterPushDown>> {
        self.inner.supports_filters_pushdown(filters)
    }
    fn get_logical_plan(
        &'_ self,
    ) -> Option<std::borrow::Cow<'_, datafusion::logical_expr::LogicalPlan>> {
        self.inner.get_logical_plan()
    }
    async fn scan(
        &self,
        state: &dyn Session,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        self.inner.scan(state, projection, filters, limit).await
    }
}

#[cfg(test)]
mod durability_unit {
    use super::*;
    use datafusion::{
        arrow::array::RecordBatch,
        datasource::{MemTable, provider_as_source},
        logical_expr::LogicalPlanBuilder,
    };
    use pse_schema::model::{
        Authority, EnumDecl, EnumMember, FieldContract, Namespace, RelationDecl, SnapshotClass,
    };
    #[test]
    fn historical_unknown_domain_needs_the_actual_verified_source_environment() {
        let mut builder = pse_schema::RegistryBuilder::new();
        builder.declare_enum(EnumDecl::platform(
            "HistoricalChoice",
            vec![EnumMember::new("old", "old meaning")],
        ));
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                "historical_choice",
                1,
                Authority::Authored,
                SnapshotClass::Model,
                "recorded source",
            )
            .pk(&["id"])
            .columns(vec![
                FieldContract::key("id", FieldContract::nonnegative(100), "identity"),
                FieldContract::payload(
                    "choice",
                    FieldContract::enumeration("HistoricalChoice"),
                    "recorded domain",
                ),
            ]),
        );
        let historical = builder.build().unwrap();
        let relation = historical.relation("authored.historical_choice").unwrap();
        let schema = pse_schema::arrow::relation_schema_ref(&historical, relation).unwrap();
        let recorded = VerifiedRecordedContract::from_contract(
            pse_schema::fingerprint::SemanticContract::new(&historical, &[relation.id].into())
                .unwrap(),
        )
        .unwrap();
        let inner: Arc<dyn TableProvider> = Arc::new(
            MemTable::try_new(
                schema.clone(),
                vec![vec![RecordBatch::new_empty(schema.clone())]],
            )
            .unwrap(),
        );
        let ordinary = pse_schema::RegistryBuilder::new().build().unwrap();
        assert!(crate::session::admission::admit_field(&ordinary, schema.field(1)).is_err());
        let verified: Arc<dyn TableProvider> =
            Arc::new(RecordedProvider::new(inner.clone(), &recorded, relation.id).unwrap());
        let pool: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(32 << 20));
        let cancel = pse_columnar::CancellationToken::new();
        let plan = LogicalPlanBuilder::scan("source", provider_as_source(verified.clone()), None)
            .unwrap()
            .project(vec![
                datafusion::logical_expr::col("choice").alias("renamed"),
            ])
            .unwrap()
            .build()
            .unwrap();
        crate::session::admission::admit_plan(&plan, &ordinary, &[verified], &pool, &cancel)
            .unwrap();
        let unverified =
            LogicalPlanBuilder::scan("source", provider_as_source(inner.clone()), None)
                .unwrap()
                .build()
                .unwrap();
        assert!(
            crate::session::admission::admit_plan(&unverified, &ordinary, &[inner], &pool, &cancel)
                .is_err()
        );
        let forged = schema
            .field(1)
            .clone()
            .with_data_type(datafusion::arrow::datatypes::DataType::Null);
        assert!(!admits(&[schema.fields()[1].clone()], &forged));
    }
}
