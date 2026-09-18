// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Active checks at every platform logical-plan boundary, including subqueries.

use datafusion::arrow::datatypes::{DataType, Field, SchemaRef};
use datafusion::catalog::TableProvider;
use datafusion::common::{
    DataFusionError, Result,
    tree_node::{Transformed, TreeNode, TreeNodeRecursion},
};
use datafusion::datasource::source_as_provider;
use datafusion::logical_expr::{Expr, ExprSchemable, LogicalPlan};
use pse_ids::{CancellationToken, MemoryReserver};
use pse_schema::Registry;
use std::sync::Arc;

mod derive;
pub(super) use derive::{NodeIdentity, identity};

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
    // Native expression/schema rules own ordinary Arrow intermediates; declared
    // relation materialization adds its explicit domain obligations.
    let mut fields = FieldAdmissions::new(registry, reserver);
    let plan = derive::plans(
        std::slice::from_ref(plan),
        registry,
        tables,
        &mut fields,
        reserver,
        cancel,
    )?
    .remove(0);
    admit_scoped(
        &plan,
        registry,
        tables,
        &mut Vec::new(),
        cancel,
        &mut fields,
        0,
    )
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
    restore_semantic_fields_many(&[plan], registry, tables, reserver, cancel)?
        .pop()
        .ok_or_else(|| DataFusionError::Internal("admission result absent".into()))
}

pub(super) fn restore_semantic_fields_many(
    plans: &[LogicalPlan],
    registry: &Registry,
    tables: &[Arc<dyn TableProvider>],
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> Result<Vec<LogicalPlan>> {
    let mut fields = FieldAdmissions::new(registry, reserver);
    let restored = derive::plans(plans, registry, tables, &mut fields, reserver, cancel)?;
    for plan in &restored {
        admit_scoped(
            plan,
            registry,
            tables,
            &mut Vec::new(),
            cancel,
            &mut fields,
            0,
        )?;
    }
    Ok(restored)
}

fn admit_scoped(
    plan: &LogicalPlan,
    registry: &Registry,
    tables: &[Arc<dyn TableProvider>],
    scope: &mut Vec<(String, SchemaRef)>,
    cancel: &CancellationToken,
    fields: &mut FieldAdmissions<'_>,
    scope_id: usize,
) -> Result<()> {
    plan.apply_with_subqueries(|node| {
        cancel.checkpoint().map_err(|error| {
            DataFusionError::External(Box::new(crate::CatalogError::from(error)))
        })?;
        if super::cache::admitted(node, registry, tables) {
            return Ok(TreeNodeRecursion::Jump);
        }
        // Ephemeral Subquery wrappers have no stable object address. Every other
        // node remains owned by the retained native roots for this operation.
        if !matches!(node, LogicalPlan::Subquery(_)) {
            let key = (identity(node), scope_id);
            if fields.visited.contains(&key) {
                return Ok(TreeNodeRecursion::Jump);
            }
            if fields.reservation.try_grow(64).is_ok() {
                fields.visited.insert(key);
            }
        }
        if let LogicalPlan::RecursiveQuery(query) = node {
            for field in query.schema.fields() {
                admit_field(registry, field)?;
            }
            admit_scoped(
                &query.static_term,
                registry,
                tables,
                scope,
                cancel,
                fields,
                scope_id,
            )?;
            scope.push((
                query.name.clone(),
                Arc::clone(query.static_term.schema().inner()),
            ));
            fields.scope_sequence = fields.scope_sequence.checked_add(1).ok_or_else(|| {
                DataFusionError::Internal("recursive scope counter overflow".into())
            })?;
            let recursive_scope = fields.scope_sequence;
            let result = admit_scoped(
                &query.recursive_term,
                registry,
                tables,
                scope,
                cancel,
                fields,
                recursive_scope,
            );
            scope.pop();
            result?;
            return Ok(TreeNodeRecursion::Jump);
        }
        let admitted = node.schema().as_ref();
        for field in admitted.fields() {
            fields.intermediate(field)?;
        }
        if let LogicalPlan::TableScan(scan) = node {
            admit_scan(scan, registry, tables, scope)?;
        }
        if let LogicalPlan::Dml(command) = node {
            let target = source_as_provider(&command.target)?;
            if !tables.iter().any(|bound| Arc::ptr_eq(bound, &target)) {
                return Err(DataFusionError::Plan(
                    "DML target is not an actual bound provider".to_owned(),
                ));
            }
            for field in target.schema().fields() {
                fields.intermediate(field)?;
            }
        }
        if let LogicalPlan::Join(join) = node {
            let left = join.left.schema().as_ref();
            let right = join.right.schema().as_ref();
            // Each equijoin key belongs to its own input schema. Combining those
            // schemas can introduce false ambiguity for a left anti/semi join.
            for (a, b) in &join.on {
                admit_expression(a, left, registry, fields)?;
                admit_expression(b, right, registry, fields)?;
            }
            if let Some(filter) = &join.filter {
                let schema = left.join(right).map_err(|error| {
                    DataFusionError::Plan(format!("semantic join-filter scope: {error}"))
                })?;
                admit_expression(filter, &schema, registry, fields)?;
            }
            return Ok(TreeNodeRecursion::Continue);
        }
        let schema = expression_schema(node)?;
        node.apply_expressions(|expression| {
            admit_expression(expression, &schema, registry, fields)?;
            Ok(TreeNodeRecursion::Continue)
        })?;
        Ok(TreeNodeRecursion::Continue)
    })?;
    Ok(())
}
pub(super) fn admit_scan(
    scan: &datafusion::logical_expr::TableScan,
    _registry: &Registry,
    tables: &[Arc<dyn TableProvider>],
    scope: &[(String, SchemaRef)],
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
        if expected.is_none_or(|schema| {
            let actual = work.schema();
            // Native Rust builders retain anchor nullability; SQL creates a
            // nullable worktable. Both preserve the exact anchor field meaning.
            super::query_schema::schema(schema).metadata()
                != super::query_schema::schema(&actual).metadata()
                || schema.fields().len() != actual.fields().len()
                || schema
                    .fields()
                    .iter()
                    .zip(actual.fields())
                    .any(|(anchor, field)| {
                        (anchor.is_nullable() && !field.is_nullable())
                            || anchor.as_ref().clone().with_nullable(field.is_nullable()) != **field
                    })
        }) {
            return Err(DataFusionError::Plan(format!(
                "unbound or schema-incompatible recursive work table {}: expected {expected:?}, actual {:?}",
                work.name(),
                work.schema()
            )));
        }
        return Ok(());
    }
    if !tables.iter().any(|table| Arc::ptr_eq(table, &provider)) {
        return Err(DataFusionError::Plan(
            "table source is outside the pinned session inventory".to_owned(),
        ));
    }
    // Registered bindings retain the actual implementations and their source
    // contracts. Concrete Rust types do not decide semantic admission here.
    // Field/relational facts are established by construction or output obligations;
    // passing this structural check cannot mint a completed/admitted result.

    Ok(())
}

/// The shared recursive field validator, used before and after engine transformations.
/// # Errors
/// Invalid storage, nested fields, extension identity/version/metadata, or semantic tags.
pub fn admit_field(registry: &Registry, field: &Field) -> Result<()> {
    if field
        .metadata()
        .keys()
        .any(|key| key.starts_with("pse.layout."))
    {
        // A native Delta leaf may use view arrays while its stored descriptor
        // names the execution representation. Validate that exact named mapping
        // before applying domain checks to the reconstructed execution field.
        let execution =
            pse_schema::delta::execution_schema(&datafusion::arrow::datatypes::Schema::new(vec![
                field.clone(),
            ]))
            .map_err(|error| DataFusionError::External(Box::new(error)))?;
        for decoded in execution.fields() {
            admit_field(registry, decoded)?;
        }
        return Ok(());
    }
    // Platform field predicates constrain declared PSE meanings. Untagged native
    // temporaries are governed by Arrow/DataFusion's own type universe, not the
    // smaller set of persisted relation declarations.
    if field.metadata().keys().any(|key| key.starts_with("pse."))
        || field
            .metadata()
            .get(pse_schema::arrow::KEY_EXTENSION_NAME)
            .is_some_and(|name| name.starts_with("pse."))
    {
        return pse_relations::validate::validate_field(registry, field)
            .map_err(|errors| invalid_rows(&errors));
    }
    match field.data_type() {
        DataType::List(child)
        | DataType::LargeList(child)
        | DataType::ListView(child)
        | DataType::LargeListView(child)
        | DataType::FixedSizeList(child, _)
        | DataType::Map(child, _) => admit_field(registry, child)?,
        DataType::Struct(children) => {
            for child in children {
                admit_field(registry, child)?;
            }
        }
        DataType::Union(children, _) => {
            for (_, child) in children.iter() {
                admit_field(registry, child)?;
            }
        }
        DataType::RunEndEncoded(runs, values) => {
            admit_field(registry, runs)?;
            admit_field(registry, values)?;
        }
        DataType::Dictionary(_, value) => admit_field(
            registry,
            &Field::new("dictionary_value", value.as_ref().clone(), true),
        )?,
        _ => {}
    }
    Ok(())
}
fn invalid_rows(errors: &[pse_relations::RelationError]) -> DataFusionError {
    DataFusionError::Plan(format!("semantic admission failed: {errors:?}"))
}
/// One admission traversal can encounter the same immutable tuple fields many
/// times through native view expansion. Cache successful full-field checks only
/// within this registry-bound traversal; source ownership and expression transfer
/// are still checked at every use. Budget pressure simply bypasses retention.
struct FieldAdmissions<'a> {
    registry: &'a Registry,
    accepted: std::collections::HashMap<usize, Arc<Field>>,
    accepted_values: std::collections::HashSet<AdmittedField>,
    visited: std::collections::HashSet<(NodeIdentity, usize)>,
    scope_sequence: usize,
    reservation: Box<dyn pse_ids::Reservation>,
}
/// Cheap bucket selection, followed by Arrow's complete field equality. Hashing
/// every nested metadata tree for every expression costs more than the lookup;
/// collisions never establish admission because Eq still compares all fields.
#[derive(PartialEq, Eq)]
struct AdmittedField(Arc<Field>);
impl std::hash::Hash for AdmittedField {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(self.0.name(), state);
        std::hash::Hash::hash(&std::mem::discriminant(self.0.data_type()), state);
    }
}
impl<'a> FieldAdmissions<'a> {
    fn new(registry: &'a Registry, reserver: &dyn MemoryReserver) -> Self {
        Self {
            registry,
            accepted: std::collections::HashMap::new(),
            accepted_values: std::collections::HashSet::new(),
            visited: std::collections::HashSet::new(),
            scope_sequence: 0,
            reservation: reserver.open("session:field-admission"),
        }
    }
    fn intermediate(&mut self, field: &Arc<Field>) -> Result<()> {
        let identity = Arc::as_ptr(field) as usize;
        if self.accepted.contains_key(&identity)
            || self
                .accepted_values
                .contains(&AdmittedField(Arc::clone(field)))
        {
            return Ok(());
        }
        let declared = field.metadata().keys().any(|key| key.starts_with("pse."))
            || field
                .metadata()
                .get(pse_schema::arrow::KEY_EXTENSION_NAME)
                .is_some_and(|name| name.starts_with("pse."));
        if declared {
            admit_intermediate_field(self.registry, field)?;
        } else {
            // Native tuple/UNNEST fields frequently get a new outer Field while
            // retaining the exact immutable child declarations. Admit those
            // children through the same cache instead of rewalking the tuple.
            match field.data_type() {
                DataType::List(child)
                | DataType::LargeList(child)
                | DataType::ListView(child)
                | DataType::LargeListView(child)
                | DataType::FixedSizeList(child, _)
                | DataType::Map(child, _) => self.intermediate(child)?,
                DataType::Struct(children) => {
                    for child in children {
                        self.intermediate(child)?;
                    }
                }
                DataType::Union(children, _) => {
                    for (_, child) in children.iter() {
                        self.intermediate(child)?;
                    }
                }
                DataType::RunEndEncoded(runs, values) => {
                    self.intermediate(runs)?;
                    self.intermediate(values)?;
                }
                DataType::Dictionary(_, value) => self.intermediate(&Arc::new(Field::new(
                    "dictionary_value",
                    value.as_ref().clone(),
                    true,
                )))?,
                _ => return Ok(()),
            }
        }
        // Charge shared nested fields conservatively as well as hash-table slack.
        if self
            .reservation
            .try_grow(field.size().saturating_add(256))
            .is_ok()
        {
            // Retaining the actual immutable Field prevents address reuse and
            // forces mutations through Arc::make_mut to acquire another identity.
            self.accepted.insert(identity, Arc::clone(field));
            // Expr::to_field can allocate a fresh Arc for the same complete
            // native declaration. Structural equality includes every metadata
            // entry and nested child; names alone never establish admission.
            self.accepted_values
                .insert(AdmittedField(Arc::clone(field)));
        }
        Ok(())
    }
}
fn admit_intermediate_field(registry: &Registry, field: &Field) -> Result<()> {
    if let Some(descriptor) = field.metadata().get(pse_schema::delta::KEY_EXECUTION_FIELD) {
        // Native expressions rename values and outer joins widen nullability.
        // Those are expression properties, not changes to the persisted layout.
        // Validate the original descriptor with its original name/nullability;
        // actual provider and durable-boundary schemas still use admit_field.
        let declared: Field = serde_json::from_str(descriptor)
            .map_err(|error| DataFusionError::External(Box::new(error)))?;
        return admit_field(
            registry,
            &field
                .clone()
                .with_name(declared.name())
                .with_nullable(declared.is_nullable()),
        );
    }
    admit_field(registry, field)
}

// Native constructors own schema/nullability/functional-dependency derivation.
// The only extra transfer here repairs the pinned engine's lost list-child metadata.
#[cfg(test)]
fn intermediate_schema(
    plan: &LogicalPlan,
    registry: &Registry,
) -> Result<datafusion::common::DFSchema> {
    let budget = pse_ids::FixedBudget::new(64 * 1024 * 1024);
    let mut fields = FieldAdmissions::new(registry, budget.as_ref());
    Ok(derive::plans(
        std::slice::from_ref(plan),
        registry,
        &[],
        &mut fields,
        budget.as_ref(),
        &CancellationToken::new(),
    )?
    .remove(0)
    .schema()
    .as_ref()
    .clone())
}

fn derive_native_node(
    mut node: LogicalPlan,
    registry: &Registry,
    fields: &mut FieldAdmissions<'_>,
) -> Result<Transformed<LogicalPlan>> {
    let original = node.clone();
    let offered = Arc::clone(node.schema());
    let mut changed = super::field_transfer::node(&mut node)?;
    if let LogicalPlan::Unnest(value) = &node {
        // Validate dependency indices and depth against the native constructor,
        // rather than trusting fields a caller can populate by hand.
        let rebuilt = node.clone().recompute_schema()?;
        let LogicalPlan::Unnest(native) = &rebuilt else {
            return Ok(Transformed::yes(rebuilt));
        };
        if value.dependency_indices != native.dependency_indices
            || value.list_type_columns != native.list_type_columns
            || value.struct_type_columns != native.struct_type_columns
        {
            return Err(DataFusionError::Plan(
                "unnest mapping differs from the native constructor".to_owned(),
            ));
        }
    }
    let mut native_before = node.clone().recompute_schema().map_err(|error| {
        DataFusionError::Plan(format!(
            "native schema reconstruction for {}: {error}",
            node.display()
        ))
    })?;
    if let LogicalPlan::Unnest(value) = &mut native_before {
        value.schema = Arc::new(unnest_schema(value, registry)?);
    }
    admit_derived_schema(&offered, native_before.schema().fields())?;
    let node = normalize_casts(node)?;
    changed |= node.transformed;
    let mut rebuilt = match node.data {
        LogicalPlan::Union(value) => {
            let before = value.inputs.clone();
            let result = super::scalar::union_fields(value.inputs)?;
            if let LogicalPlan::Union(after) = &result {
                changed |= before.len() != after.inputs.len()
                    || before
                        .iter()
                        .zip(&after.inputs)
                        .any(|(a, b)| !Arc::ptr_eq(a, b));
            }
            result
        }
        other if node.transformed => other.recompute_schema()?,
        _ => native_before,
    };
    if let LogicalPlan::Unnest(value) = &mut rebuilt {
        value.schema = Arc::new(unnest_schema(value, registry)?);
    }
    if let LogicalPlan::RecursiveQuery(query) = &mut rebuilt {
        // Native recursive output metadata comes from the anchor. Terms can
        // come from different relation owners, and physical joins can drop
        // schema metadata. Explicit native projections transport the recursive
        // output declaration. Field metadata, types and nullability remain
        // unchanged and are checked by RecursiveQueryExec.
        for term in [&mut query.static_term, &mut query.recursive_term] {
            if matches!(term.as_ref(), LogicalPlan::Projection(projection)
                if projection.schema.metadata() == query.schema.metadata())
            {
                continue;
            }
            changed = true;
            let expressions = term
                .schema()
                .columns()
                .into_iter()
                .map(Expr::Column)
                .collect();
            let schema = datafusion::common::DFSchema::new_with_metadata(
                term.schema()
                    .iter()
                    .map(|(qualifier, field)| (qualifier.cloned(), Arc::clone(field)))
                    .collect(),
                query.schema.metadata().clone(),
            )?;
            *term = Arc::new(LogicalPlan::Projection(
                datafusion::logical_expr::Projection::try_new_with_schema(
                    expressions,
                    Arc::clone(term),
                    Arc::new(schema),
                )?,
            ));
        }
    }
    for field in rebuilt.schema().fields() {
        fields.intermediate(field)?;
    }
    changed |= offered != *rebuilt.schema() || original.expressions() != rebuilt.expressions();
    Ok(Transformed::new_transformed(
        if changed { rebuilt } else { original },
        changed,
    ))
}

fn normalize_casts(mut node: LogicalPlan) -> Result<Transformed<LogicalPlan>> {
    // Equijoin keys belong to their respective input schemas.
    if let LogicalPlan::Join(join) = &mut node {
        let mut changed = false;
        for (left, right) in &mut join.on {
            let result = left
                .clone()
                .transform_up(|expr| normalize_primitive_cast(expr, join.left.schema()))?;
            changed |= result.transformed;
            *left = result.data;
            let result = right
                .clone()
                .transform_up(|expr| normalize_primitive_cast(expr, join.right.schema()))?;
            changed |= result.transformed;
            *right = result.data;
        }
        if let Some(filter) = join.filter.take() {
            let schema = join
                .left
                .schema()
                .join(join.right.schema())
                .map_err(|error| {
                    DataFusionError::Plan(format!("semantic cast join-filter scope: {error}"))
                })?;
            let result = filter.transform_up(|expr| normalize_primitive_cast(expr, &schema))?;
            changed |= result.transformed;
            join.filter = Some(result.data);
        }
        return Ok(Transformed::new(node, changed, TreeNodeRecursion::Continue));
    }
    let input_schema = expression_schema(&node)?;
    node.map_expressions(|expression| {
        expression.transform_up(|expression| normalize_primitive_cast(expression, &input_schema))
    })
}
fn expression_schema(node: &LogicalPlan) -> Result<datafusion::common::DFSchemaRef> {
    if let LogicalPlan::Dml(command) = node
        && matches!(
            command.op,
            datafusion::logical_expr::dml::WriteOp::MergeInto(_)
        )
    {
        // Native MERGE expressions refer to both the target and USING input.
        // DataFusion canonicalizes target aliases to the actual table reference.
        let target = datafusion::common::DFSchema::try_from_qualified_schema(
            command.table_name.clone(),
            &command.target.schema(),
        )?;
        return Ok(Arc::new(target.join(command.input.schema())?));
    }
    Ok(node.inputs().first().map_or_else(
        || Arc::clone(node.schema()),
        |input| Arc::clone(input.schema()),
    ))
}
fn normalize_primitive_cast(
    expression: Expr,
    schema: &datafusion::common::DFSchema,
) -> Result<Transformed<Expr>> {
    let (child, target) = match &expression {
        Expr::Cast(cast) => (&cast.expr, &cast.field),
        Expr::TryCast(cast) => (&cast.expr, &cast.field),
        _ => return Ok(Transformed::no(expression)),
    };
    let (_, source) = child.to_field(schema)?;
    // A native physical cast may carry primitive input metadata into its new
    // physical type. That representation change cannot carry role/FK assertions.
    // Extension and quantity conversions require their registered semantic functions.
    let type_only =
        target.name().is_empty() && target.is_nullable() && target.metadata().is_empty();
    if source.data_type() == target.data_type()
        || !type_only
        || !source.metadata().keys().any(|key| key.starts_with("pse."))
    {
        return Ok(Transformed::no(expression));
    }
    // DataFusion's type-only cast deliberately inherits non-extension metadata.
    // An explicit target field requests native physical storage without carrying
    // a declaration for the source type into that different representation.
    let mut field = target
        .as_ref()
        .clone()
        .with_name("cast_value")
        .with_nullable(matches!(expression, Expr::TryCast(_)) || source.is_nullable())
        .with_metadata(source.metadata().clone());
    field.metadata_mut().retain(|key, _| {
        !key.starts_with("pse.")
            && key != pse_schema::arrow::KEY_EXTENSION_NAME
            && key != pse_schema::arrow::KEY_EXTENSION_METADATA
    });
    let mut expression = expression;
    match &mut expression {
        Expr::Cast(cast) => cast.field = Arc::new(field),
        Expr::TryCast(cast) => cast.field = Arc::new(field),
        _ => {}
    }
    Ok(Transformed::yes(expression))
}

fn admit_derived_schema(
    actual: &datafusion::common::DFSchema,
    expected: &[Arc<Field>],
) -> Result<()> {
    if actual.fields().len() != expected.len() {
        return Err(DataFusionError::Plan(
            "derived field inventory differs from actual plan schema".into(),
        ));
    }
    for (actual, expected) in actual.fields().iter().zip(expected) {
        if Arc::ptr_eq(actual, expected) {
            continue;
        }
        if !super::scalar::proven_native_layout(actual.data_type(), expected.data_type())
            || (!actual.is_nullable() && expected.is_nullable())
            || (!actual.metadata().is_empty() && !same_value_metadata(actual, expected))
        {
            return Err(DataFusionError::Plan(format!(
                "derived field {} differs from its actual expression/source contract: offered={actual:?}, native={expected:?}",
                actual.name()
            )));
        }
    }
    Ok(())
}
pub(crate) fn same_value_metadata(actual: &Field, expected: &Field) -> bool {
    if std::ptr::eq(actual, expected) {
        return true;
    }
    // Role and foreign-key annotations declare relation obligations. They do not
    // change a column value's type or grant native optimizer constraints. An
    // equijoin may select either equivalent key as the projected expression.
    let meaning =
        |key: &str| key != pse_schema::arrow::KEY_ROLE && key != pse_schema::arrow::KEY_FK;
    actual
        .metadata()
        .iter()
        .filter(|(key, _)| meaning(key))
        .all(|(key, value)| expected.metadata().get(key) == Some(value))
        && expected
            .metadata()
            .iter()
            .filter(|(key, _)| meaning(key))
            .all(|(key, value)| actual.metadata().get(key) == Some(value))
}
fn unnest_schema(
    unnest: &datafusion::logical_expr::Unnest,
    registry: &Registry,
) -> Result<datafusion::common::DFSchema> {
    if unnest.dependency_indices.len() != unnest.schema.fields().len() {
        return Err(DataFusionError::Plan(
            "unsupported or malformed unnest dependency map".to_owned(),
        ));
    }
    let input = unnest.input.schema();
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
                // A single expansion with Drop emits only actual child values.
                // Multiple expansions zip to the longest list and can pad with NULL.
                .with_nullable(
                    declared.is_nullable()
                        || unnest.options.preserve_nulls()
                        || unnest.list_type_columns.len() != 1,
                )
                .with_metadata(declared.metadata().clone());
            admit_field(registry, &field)?;
            fields.push((qualifier.cloned(), Arc::new(field)));
        } else if unnest.struct_type_columns.contains(index) {
            // Native struct expansion derives the child's field directly.
            admit_field(registry, actual)?;
            fields.push((qualifier.cloned(), Arc::clone(actual)));
        } else {
            if actual.as_ref() != declared {
                return Err(DataFusionError::Plan(
                    "unnest changed an unrelated source field".to_owned(),
                ));
            }
            fields.push((qualifier.cloned(), Arc::clone(actual)));
        }
    }
    datafusion::common::DFSchema::new_with_metadata(fields, unnest.schema.metadata().clone())?
        .with_functional_dependencies(unnest.schema.functional_dependencies().clone())
}

fn admit_expression(
    expression: &Expr,
    schema: &datafusion::common::DFSchema,
    registry: &Registry,
    fields: &mut FieldAdmissions<'_>,
) -> Result<()> {
    expression.apply(|expression| {
        if let Expr::Alias(alias) = expression {
            let (_, child) = alias.expr.to_field(schema)?;
            let (_, output) = expression.to_field(schema)?;
            // An alias names a value; it cannot invent an extension, quantity or
            // enumeration meaning. Primitive logical labels follow native types.
            for key in [
                pse_schema::arrow::KEY_QUANTITY_TYPE,
                pse_schema::arrow::KEY_ENUM,
                pse_schema::arrow::KEY_EXTENSION_NAME,
                pse_schema::arrow::KEY_EXTENSION_METADATA,
            ] {
                if child.metadata().get(key) != output.metadata().get(key) {
                    return Err(DataFusionError::Plan(
                        format!("alias {} changed semantic key {key}: source={:?}, output={:?}, expression={:?}", alias.name, child.metadata().get(key), output.metadata().get(key), alias.expr),
                    ));
                }
            }
        }
        let (_, field) = expression.to_field(schema)?;
        // Native scalar simplification can retain a Boolean declaration on NULL.
        if matches!(
            expression,
            Expr::Literal(datafusion::common::ScalarValue::Null, _)
        ) && field
            .metadata()
            .get(pse_schema::arrow::KEY_LOGICAL_TYPE)
            .is_some_and(|kind| kind == "bool")
        {
            admit_field(
                registry,
                &field.as_ref().clone().with_data_type(DataType::Boolean),
            )?;
        } else if matches!(field.data_type(), DataType::Dictionary(_, _))
            && field
                .metadata()
                .get(pse_schema::arrow::KEY_LOGICAL_TYPE)
                .is_some_and(|kind| kind == "text")
        {
            admit_field(
                registry,
                &field.as_ref().clone().with_data_type(DataType::Utf8),
            )?;
        } else {
            fields.intermediate(&field)?;
        }
        Ok(TreeNodeRecursion::Continue)
    })?;
    Ok(())
}

#[cfg(test)]
mod tests;
