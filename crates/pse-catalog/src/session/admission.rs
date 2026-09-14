// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Active checks at every platform logical-plan boundary, including subqueries.

use super::candidate::CandidateTable;
use crate::provider::table::RelationTable;
use datafusion::arrow::datatypes::{DataType, Field, SchemaRef};
use datafusion::catalog::TableProvider;
use datafusion::common::{
    DataFusionError, Result,
    config::ConfigOptions,
    tree_node::{Transformed, TreeNode, TreeNodeRecursion},
};
use datafusion::datasource::source_as_provider;
use datafusion::logical_expr::{Expr, ExprSchemable, LogicalPlan};
use datafusion::optimizer::AnalyzerRule;
use pse_ids::{CancellationToken, MemoryReserver};
use pse_schema::Registry;
use std::sync::Arc;

/// Validate fields and actual provider ownership; hashes never admit a foreign source.
/// # Errors
/// Foreign or mutated input, invalid nested field/extension metadata, or a mutation plan.
pub fn admit_plan(
    plan: &LogicalPlan,
    registry: &Registry,
    tables: &[Arc<dyn TableProvider>],
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> Result<()> {
    admit_scoped(plan, registry, tables, &mut Vec::new(), reserver, cancel)
}

/// Restore only metadata proved by actual expressions and admitted source fields.
///
/// This runs after optimization and before observation or physical planning. The
/// original plan is checked first so a child rewrite cannot erase forged metadata.
pub(super) fn restore_semantic_fields(
    plan: LogicalPlan,
    registry: &Registry,
    tables: &[Arc<dyn TableProvider>],
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> Result<LogicalPlan> {
    admit_plan(&plan, registry, tables, reserver, cancel)?;
    let restored = plan
        .transform_up_with_subqueries(|mut node| {
            let expected = intermediate_schema(&node, registry)?;
            if node.schema().as_ref() == &expected {
                return Ok(Transformed::no(node));
            }
            let target = match &mut node {
                LogicalPlan::Projection(value) => &mut value.schema,
                LogicalPlan::Aggregate(value) => &mut value.schema,
                LogicalPlan::Unnest(value) => &mut value.schema,
                LogicalPlan::SubqueryAlias(value) => &mut value.schema,
                LogicalPlan::Join(value) => &mut value.schema,
                LogicalPlan::Union(value) => &mut value.schema,
                _ => {
                    return Err(DataFusionError::Plan(
                        "derived schema has no supported restoration boundary".into(),
                    ));
                }
            };
            *target = Arc::new(expected);
            Ok(Transformed::yes(node))
        })?
        .data;
    admit_plan(&restored, registry, tables, reserver, cancel)?;
    Ok(restored)
}
fn admit_scoped(
    plan: &LogicalPlan,
    registry: &Registry,
    tables: &[Arc<dyn TableProvider>],
    scope: &mut Vec<(String, SchemaRef)>,
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> Result<()> {
    plan.apply_with_subqueries(|node| {
        cancel.checkpoint().map_err(|error| {
            DataFusionError::External(Box::new(crate::CatalogError::from(error)))
        })?;
        if let LogicalPlan::RecursiveQuery(query) = node {
            for field in query.schema.fields() {
                admit_field(registry, field)?;
            }
            admit_scoped(
                &query.static_term,
                registry,
                tables,
                scope,
                reserver,
                cancel,
            )?;
            scope.push((
                query.name.clone(),
                Arc::new(query.schema.as_arrow().clone()),
            ));
            let result = admit_scoped(
                &query.recursive_term,
                registry,
                tables,
                scope,
                reserver,
                cancel,
            );
            scope.pop();
            result?;
            return Ok(TreeNodeRecursion::Jump);
        }
        if matches!(
            node,
            LogicalPlan::Dml(_)
                | LogicalPlan::Ddl(_)
                | LogicalPlan::Copy(_)
                | LogicalPlan::Statement(_)
        ) {
            return Err(DataFusionError::Plan(
                "snapshot sessions accept read-only logical plans".to_owned(),
            ));
        }
        let admitted = intermediate_schema(node, registry)?;
        for field in admitted.fields() {
            admit_field(registry, field)?;
        }
        if let LogicalPlan::TableScan(scan) = node {
            admit_scan(scan, registry, tables, scope, reserver, cancel)?;
        }
        if let LogicalPlan::Join(join) = node {
            let left = intermediate_schema(&join.left, registry)?;
            let right = intermediate_schema(&join.right, registry)?;
            // Each equijoin key belongs to its own input schema. Combining those
            // schemas can introduce false ambiguity for a left anti/semi join.
            for (a, b) in &join.on {
                admit_expression(a, &left, registry)?;
                admit_expression(b, &right, registry)?;
            }
            if let Some(filter) = &join.filter {
                admit_expression(filter, &left.join(&right)?, registry)?;
            }
            return Ok(TreeNodeRecursion::Continue);
        }
        let mut inputs = node.inputs().into_iter();
        let schema = inputs.next().map_or_else(
            || Ok(admitted),
            |input| intermediate_schema(input, registry),
        )?;
        for expression in node.expressions() {
            admit_expression(&expression, &schema, registry)?;
        }
        Ok(TreeNodeRecursion::Continue)
    })?;
    Ok(())
}
fn admit_scan(
    scan: &datafusion::logical_expr::TableScan,
    registry: &Registry,
    tables: &[Arc<dyn TableProvider>],
    scope: &[(String, SchemaRef)],
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> Result<()> {
    let provider = source_as_provider(&scan.source)?;
    if let Some(work) = provider
        .as_ref()
        .downcast_ref::<datafusion_catalog::cte_worktable::CteWorkTable>()
    {
        let expected = scope
            .iter()
            .rev()
            .find(|(name, _)| name == work.name())
            .map(|(_, schema)| schema);
        if expected.is_none_or(|schema| schema.as_ref() != work.schema().as_ref()) {
            return Err(DataFusionError::Plan(
                "unbound or schema-incompatible recursive work table".to_owned(),
            ));
        }
        return Ok(());
    }
    if !tables.iter().any(|table| Arc::ptr_eq(table, &provider)) {
        return Err(DataFusionError::Plan(
            "table source is outside the pinned session inventory".to_owned(),
        ));
    }
    if let Some(table) = provider.as_ref().downcast_ref::<RelationTable>() {
        let relation = table.relation();
        let spec = registry
            .relation_by_id(relation.contract().canonical.relation_id)
            .ok_or_else(|| DataFusionError::Plan("unknown admitted relation".to_owned()))?;
        relation
            .contract()
            .validate_against_registry(registry, spec)
            .map_err(|error| DataFusionError::External(Box::new(error)))?;
        validate_batch(registry, spec, relation.batch(), reserver, cancel)
            .map_err(|error| DataFusionError::External(Box::new(error)))?;
    } else if let Some(table) = provider.as_ref().downcast_ref::<CandidateTable>() {
        let spec = registry
            .relation(&table.key.qualified_name())
            .ok_or_else(|| DataFusionError::Plan("unknown candidate relation".to_owned()))?;
        validate_batch(registry, spec, &table.batch, reserver, cancel)
            .map_err(|error| DataFusionError::External(Box::new(error)))?;
        if provider.constraints().is_some() {
            return Err(DataFusionError::Plan(
                "unpublished candidates cannot advertise constraints".to_owned(),
            ));
        }
    } else {
        return Err(DataFusionError::Plan(
            "foreign provider implementation".to_owned(),
        ));
    }
    Ok(())
}

/// Scratch ownership covers decoded cells, nested values and validation findings.
/// The input's own buffer lease remains separate; no valid digest skips this check.
pub(super) fn validate_batch(
    registry: &Registry,
    spec: &pse_schema::model::RelationSpec,
    batch: &datafusion::arrow::array::RecordBatch,
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> std::result::Result<(), crate::CatalogError> {
    cancel.checkpoint()?;
    let mut scratch = reserver.open("session:validate-batch");
    scratch.try_grow(crate::store::membership::validation_extent(batch)?)?;
    cancel.checkpoint()?;
    pse_relations::validate::validate_batch(registry, spec, batch).map_err(|errors| {
        crate::CatalogError::Admission {
            path: spec.key.qualified_name(),
            reason: format!("schema/value admission failed: {errors:?}"),
        }
    })?;
    cancel.checkpoint()?;
    Ok(())
}
/// The shared recursive field validator, used before and after engine transformations.
/// # Errors
/// Invalid storage, nested fields, extension identity/version/metadata, or semantic tags.
pub fn admit_field(registry: &Registry, field: &Field) -> Result<()> {
    pse_relations::validate::validate_field(registry, field).map_err(|errors| invalid_rows(&errors))
}
fn invalid_rows(errors: &[pse_relations::RelationError]) -> DataFusionError {
    DataFusionError::Plan(format!("semantic admission failed: {errors:?}"))
}
#[derive(Debug)]
pub(crate) struct AdmissionRule {
    pub name: &'static str,
    pub registry: Arc<Registry>,
    pub tables: Vec<Arc<dyn TableProvider>>,
    pub(super) functions: Arc<super::functions::Functions>,
    pub(super) reserver: Arc<dyn MemoryReserver>,
    pub(super) cancel: CancellationToken,
}
impl AnalyzerRule for AdmissionRule {
    fn name(&self) -> &str {
        self.name
    }
    fn analyze(&self, plan: LogicalPlan, _config: &ConfigOptions) -> Result<LogicalPlan> {
        self.functions.admit_plan(&plan)?;
        admit_plan(
            &plan,
            &self.registry,
            &self.tables,
            self.reserver.as_ref(),
            &self.cancel,
        )?;
        Ok(plan)
    }
}

fn lossless_integer_temporary(source: &DataType, target: &DataType) -> bool {
    let digits = match source {
        DataType::Int8 | DataType::UInt8 => 3,
        DataType::Int16 | DataType::UInt16 => 5,
        DataType::Int32 | DataType::UInt32 => 10,
        DataType::Int64 => 19,
        DataType::UInt64 => 20,
        _ => return false,
    };
    matches!(target,DataType::Decimal128(precision,0) if *precision>=digits)
}

fn lossless_text_encoding(
    cast: &datafusion::logical_expr::expr::Cast,
    schema: &datafusion::common::DFSchema,
) -> Result<bool> {
    let (_, child) = cast.expr.to_field(schema)?;
    Ok(child.data_type() == &DataType::Utf8
        && matches!(cast.field.data_type(), DataType::Dictionary(key, value) if key.as_ref() == &DataType::Int32 && value.as_ref() == &DataType::Utf8)
        && child
            .metadata()
            .get(pse_schema::arrow::KEY_LOGICAL_TYPE)
            .is_some_and(|kind| kind == "text")
        && (cast.field.metadata().is_empty() || cast.field.metadata() == child.metadata()))
}

// DataFusion's Unnest constructor discards list-child metadata. The input child and
// explicit dependency/depth map provide the type derivation; storage alone does not.
fn intermediate_schema(
    plan: &LogicalPlan,
    registry: &Registry,
) -> Result<datafusion::common::DFSchema> {
    use datafusion::logical_expr::Distinct;
    match plan {
        LogicalPlan::Unnest(unnest) => unnest_schema(unnest, registry),
        LogicalPlan::Projection(project) => {
            let input = intermediate_schema(&project.input, registry)?;
            let fields = project
                .expr
                .iter()
                .map(|expression| expression.to_field(&input).map(|(_, field)| field))
                .collect::<Result<Vec<_>>>()?;
            derived_schema(plan.schema(), &fields, registry)
        }
        LogicalPlan::Aggregate(aggregate)
            if !aggregate
                .group_expr
                .iter()
                .any(|expr| matches!(expr, Expr::GroupingSet(_))) =>
        {
            let input = intermediate_schema(&aggregate.input, registry)?;
            let fields = aggregate
                .group_expr
                .iter()
                .chain(&aggregate.aggr_expr)
                .map(|expression| expression.to_field(&input).map(|(_, field)| field))
                .collect::<Result<Vec<_>>>()?;
            derived_schema(plan.schema(), &fields, registry)
        }
        LogicalPlan::Filter(filter) => inherited_schema(plan, &filter.input, registry),
        LogicalPlan::Sort(sort) => inherited_schema(plan, &sort.input, registry),
        LogicalPlan::Limit(limit) => inherited_schema(plan, &limit.input, registry),
        LogicalPlan::Repartition(repartition) => {
            inherited_schema(plan, &repartition.input, registry)
        }
        LogicalPlan::SubqueryAlias(alias) => inherited_schema(plan, &alias.input, registry),
        LogicalPlan::Distinct(Distinct::All(input)) => inherited_schema(plan, input, registry),
        LogicalPlan::Join(join) => {
            let left = intermediate_schema(&join.left, registry)?;
            let right = intermediate_schema(&join.right, registry)?;
            let expected = datafusion::logical_expr::logical_plan::builder::build_join_schema(
                &left,
                &right,
                &join.join_type,
            )?;
            derived_schema(plan.schema(), expected.fields(), registry)
        }
        LogicalPlan::Union(union) => {
            let mut inputs = union.inputs.iter();
            let first = intermediate_schema(
                inputs
                    .next()
                    .ok_or_else(|| DataFusionError::Plan("empty union input inventory".into()))?,
                registry,
            )?;
            for input in inputs {
                let input = intermediate_schema(input, registry)?;
                if input.fields().len() != first.fields().len()
                    || input.fields().iter().zip(first.fields()).any(|(a, b)| {
                        a.data_type() != b.data_type() || a.metadata() != b.metadata()
                    })
                {
                    return Err(DataFusionError::Plan(
                        "union input semantic contracts disagree".into(),
                    ));
                }
            }
            derived_schema(plan.schema(), first.fields(), registry)
        }
        _ => Ok(plan.schema().as_ref().clone()),
    }
}
fn inherited_schema(
    plan: &LogicalPlan,
    input: &LogicalPlan,
    registry: &Registry,
) -> Result<datafusion::common::DFSchema> {
    let expected = intermediate_schema(input, registry)?;
    derived_schema(plan.schema(), expected.fields(), registry)
}
fn derived_schema(
    actual: &datafusion::common::DFSchema,
    expected: &[Arc<Field>],
    registry: &Registry,
) -> Result<datafusion::common::DFSchema> {
    if actual.fields().len() != expected.len() {
        return Err(DataFusionError::Plan(
            "derived field inventory differs from actual plan schema".into(),
        ));
    }
    let fields = actual
        .iter()
        .zip(expected)
        .map(|((qualifier, actual), expected)| {
            if actual.data_type() != expected.data_type()
                || (!actual.metadata().is_empty() && actual.metadata() != expected.metadata())
            {
                return Err(DataFusionError::Plan(format!(
                    "derived field {} differs from its actual expression/source contract",
                    actual.name()
                )));
            }
            let field = actual
                .as_ref()
                .clone()
                .with_metadata(expected.metadata().clone());
            admit_field(registry, &field)?;
            Ok((qualifier.cloned(), Arc::new(field)))
        })
        .collect::<Result<Vec<_>>>()?;
    datafusion::common::DFSchema::new_with_metadata(fields, actual.metadata().clone())
}
fn unnest_schema(
    unnest: &datafusion::logical_expr::Unnest,
    registry: &Registry,
) -> Result<datafusion::common::DFSchema> {
    if !unnest.struct_type_columns.is_empty()
        || unnest.dependency_indices.len() != unnest.schema.fields().len()
    {
        return Err(DataFusionError::Plan(
            "unsupported or malformed unnest dependency map".to_owned(),
        ));
    }
    let input = intermediate_schema(&unnest.input, registry)?;
    let mut fields = Vec::new();
    for ((qualifier, actual), index) in unnest.schema.iter().zip(&unnest.dependency_indices) {
        let source = input
            .fields()
            .get(*index)
            .ok_or_else(|| DataFusionError::Plan("unnest source index out of range".to_owned()))?;
        let mut declared = source.as_ref();
        if let Some((_, list)) = unnest
            .list_type_columns
            .iter()
            .find(|(input, list)| *input == *index && list.output_column.name == *actual.name())
        {
            for _ in 0..list.depth {
                declared = match declared.data_type() {
                    DataType::List(child)
                    | DataType::LargeList(child)
                    | DataType::FixedSizeList(child, _) => child.as_ref(),
                    _ => {
                        return Err(DataFusionError::Plan(
                            "unnest depth does not follow declared list children".to_owned(),
                        ));
                    }
                };
            }
            if actual.data_type() != declared.data_type()
                || (!actual.metadata().is_empty() && actual.metadata() != declared.metadata())
            {
                return Err(DataFusionError::Plan(
                    "unnest output differs from exact source child contract".to_owned(),
                ));
            }
            let field = actual
                .as_ref()
                .clone()
                .with_metadata(declared.metadata().clone());
            admit_field(registry, &field)?;
            fields.push((qualifier.cloned(), Arc::new(field)));
        } else {
            if actual.as_ref() != declared {
                return Err(DataFusionError::Plan(
                    "unnest changed an unrelated source field".to_owned(),
                ));
            }
            fields.push((qualifier.cloned(), Arc::clone(actual)));
        }
    }
    datafusion::common::DFSchema::new_with_metadata(fields, unnest.schema.metadata().clone())
}

fn admit_expression(
    expression: &Expr,
    schema: &datafusion::common::DFSchema,
    registry: &Registry,
) -> Result<()> {
    expression.apply(|expression| {
        if let Expr::Alias(alias) = expression {
            let (_, child) = alias.expr.to_field(schema)?;
            let (_, output) = expression.to_field(schema)?;
            let meaning = [
                pse_schema::arrow::KEY_LOGICAL_TYPE,
                pse_schema::arrow::KEY_QUANTITY_TYPE,
                pse_schema::arrow::KEY_ENUM,
                pse_schema::arrow::KEY_EXTENSION_NAME,
                pse_schema::arrow::KEY_EXTENSION_METADATA,
            ];
            if !child.metadata().is_empty()
                && meaning
                    .iter()
                    .any(|key| child.metadata().get(*key) != output.metadata().get(*key))
            {
                return Err(DataFusionError::Plan(
                    "alias changed its actual expression's semantic contract".into(),
                ));
            }
        }
        // The engine may widen integer comparisons to a decimal temporary.
        // Prove that representation lossless over the full source domain; the
        // child retains its admitted meaning and an exported cast still passes
        // the strict node-schema check above.
        let intermediate = if let Expr::Cast(cast) = expression {
            (cast.field.metadata().is_empty()
                && lossless_integer_temporary(&cast.expr.get_type(schema)?, cast.field.data_type()))
                || lossless_text_encoding(cast, schema)?
        } else {
            false
        };
        if !intermediate {
            let (_, field) = expression.to_field(schema)?;
            if let Expr::Literal(datafusion::common::ScalarValue::Dictionary(key, value), _) =
                expression
                && key.as_ref() == &DataType::Int32
                && matches!(value.as_ref(), datafusion::common::ScalarValue::Utf8(_))
                && field
                    .metadata()
                    .get(pse_schema::arrow::KEY_LOGICAL_TYPE)
                    .is_some_and(|kind| kind == "text")
            {
                // Dictionary encoding is reversible storage for this literal.
                // Validate its decoded text contract; exported fields above
                // remain strict and an enum column retains its own membership.
                admit_field(
                    registry,
                    &field.as_ref().clone().with_data_type(DataType::Utf8),
                )?;
            } else if matches!(
                expression,
                Expr::Literal(datafusion::common::ScalarValue::Null, _)
            ) && field
                .metadata()
                .get(pse_schema::arrow::KEY_LOGICAL_TYPE)
                .is_some_and(|kind| kind == "bool")
            {
                // Simplification erases the physical type of a Boolean NULL.
                // Its retained Boolean declaration still denotes only unknown;
                // this does not admit a stored Null array as a Boolean column.
                admit_field(
                    registry,
                    &field.as_ref().clone().with_data_type(DataType::Boolean),
                )?;
            } else {
                admit_field(registry, &field)?;
            }
        }
        Ok(TreeNodeRecursion::Continue)
    })?;
    Ok(())
}

#[cfg(test)]
mod tests;
