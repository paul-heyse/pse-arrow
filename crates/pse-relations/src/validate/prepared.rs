// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Immutable session/registry owners prepare expressions once and evaluate native batches.
use crate::RelationError;
use crate::native::{
    arrow::{
        array::{Array, BooleanArray, RecordBatch, StringArray, UInt64Array},
        datatypes::{Field, Schema, SchemaRef},
    },
    common::{DFSchema, DataFusionError},
    logical_expr::Expr,
    physical_expr::PhysicalExpr,
};
use pse_schema::{
    Registry,
    model::{FieldContract, RelationSpec},
};
use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, Mutex},
};

/// An invocation's owned identity and captured timestamp. Equal clock values do
/// not confer permission to reuse scope-sensitive successful evidence.
#[derive(Clone, Debug)]
pub struct AdmissionScope {
    identity: Arc<()>,
    properties: datafusion_expr::execution_props::ExecutionProps,
}
impl Default for AdmissionScope {
    fn default() -> Self {
        Self::new()
    }
}
impl AdmissionScope {
    /// Capture one timestamp without constructing an engine or substitute configuration.
    pub fn new() -> Self {
        let mut properties = datafusion_expr::execution_props::ExecutionProps::new();
        properties.query_execution_start_time = Some(std::time::SystemTime::now().into());
        Self {
            identity: Arc::new(()),
            properties,
        }
    }
    /// Create an independent scope using an already selected native query timestamp.
    /// An absent timestamp is captured now; other execution properties are not imported.
    pub fn from_execution_props(
        properties: &datafusion_expr::execution_props::ExecutionProps,
    ) -> Self {
        let mut scope = Self::new();
        if properties.query_execution_start_time.is_some() {
            scope.properties.query_execution_start_time = properties.query_execution_start_time;
        }
        scope
    }
    /// The captured native timestamp, to install into a clone of the actual engine state.
    pub fn execution_props(&self) -> &datafusion_expr::execution_props::ExecutionProps {
        &self.properties
    }
    /// Compare owned invocation identities, independently of timestamp equality.
    pub fn same_invocation(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.identity, &other.identity)
    }
}

/// Pointer hits retain their SchemaRef, so allocation reuse cannot select another
/// schema. Equal independently supplied schemas still take exact structural lookup.
#[derive(Debug, Default)]
struct SchemaSlots {
    values: HashMap<SchemaRef, u64>,
    last: Option<(SchemaRef, u64)>,
}
impl SchemaSlots {
    fn slot(&mut self, schema: &SchemaRef) -> Result<u64, RelationError> {
        if let Some((previous, slot)) = &self.last
            && Arc::ptr_eq(previous, schema)
        {
            return Ok(*slot);
        }
        let next = u64::try_from(self.values.len()).map_err(|_| {
            super::mismatch("validation context", "preparation slot extent overflows")
        })?;
        let slot = *self.values.entry(schema.clone()).or_insert(next);
        self.last = Some((schema.clone(), slot));
        Ok(slot)
    }
}

/// One SQL binding and immutable expression preparation per actual owner/schema.
struct TemplateCell(Result<Arc<ContractTemplate>, Arc<RelationError>>);
struct PreparedCell(Result<Arc<PreparedLocalContract>, Arc<RelationError>>);
#[derive(Debug)]
struct ContractTemplate {
    immutable: Arc<PreparedLocalContract>,
    scoped: Vec<(String, Expr)>,
}
#[derive(Debug)]
struct ValidationOwner {
    owner: Arc<()>,
    native_owner: Arc<()>,
    state: Arc<dyn super::planner::ValidationPlanner>,
    domains: super::predicates::DomainEnvironment,
    prepared: Mutex<SchemaSlots>,
    implementations: pse_schema::ImplementationCache,
    successful: std::sync::atomic::AtomicUsize,
}
#[derive(Debug, Default)]
struct ScopedContracts {
    implementations: pse_schema::ImplementationCache,
    columns: Mutex<HashMap<Field, Arc<PreparedLocalContract>>>,
    relations: Mutex<BTreeMap<pse_ids::SemanticId, Arc<PreparedLocalContract>>>,
}
/// A scoped admission view over an immutable actual native implementation owner.
/// Historical scope caches are owned by their views, never accumulated on the owner.
#[derive(Clone, Debug)]
pub struct ValidationContext {
    shared: Arc<ValidationOwner>,
    scope: AdmissionScope,
    scoped: Arc<ScopedContracts>,
}
impl ValidationContext {
    /// Capture actual function/configuration bindings and one initial admission scope.
    pub fn new(
        registry: &Registry,
        state: impl super::planner::ValidationPlanner + 'static,
    ) -> Self {
        Self {
            shared: Arc::new(ValidationOwner {
                owner: registry.implementation_owner(),
                native_owner: Arc::new(()),
                state: Arc::new(state),
                domains: super::predicates::DomainEnvironment::current(registry),
                prepared: Mutex::new(SchemaSlots::default()),
                implementations: Default::default(),
                successful: std::sync::atomic::AtomicUsize::new(0),
            }),
            scope: AdmissionScope::new(),
            scoped: Arc::new(ScopedContracts::default()),
        }
    }
    /// Eligibility of this exact receiving planner for generated modeling values.
    /// A true result establishes no facts about a row or newly constructed array.
    pub fn typed_modeling_local_eligible(&self) -> bool {
        self.shared.state.typed_modeling_local_eligible()
    }
    /// Compare retained actual registry and native implementation owners, independently
    /// of the admission invocation. Content equality never establishes this relation.
    pub fn same_bindings(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.shared, &other.shared)
    }
    /// Begin another operation while retaining exact bindings and immutable templates.
    pub fn new_scope(&self) -> Self {
        self.for_scope(AdmissionScope::new())
    }
    /// Retain an explicit enclosing operation's admission scope.
    pub fn for_scope(&self, scope: AdmissionScope) -> Self {
        if self.scope.same_invocation(&scope) {
            return self.clone();
        }
        Self {
            shared: self.shared.clone(),
            scope,
            scoped: Arc::new(ScopedContracts::default()),
        }
    }
    /// The operation identity and timestamp carried by child admissions.
    pub fn admission_scope(&self) -> &AdmissionScope {
        &self.scope
    }
    /// Deliberately local validation: SQL obligations require an actual engine context.
    /// # Errors
    /// Registry memo construction fails.
    pub fn local(registry: &Registry) -> Result<Arc<Self>, RelationError> {
        struct Local(Arc<ValidationContext>);
        Ok(registry
            .derived_implementation(|| {
                Ok(Local(Arc::new(Self::new(
                    registry,
                    super::planner::LocalPlanner,
                ))))
            })?
            .0
            .clone())
    }
    fn require_registry(&self, registry: &Registry) -> Result<(), RelationError> {
        if !Arc::ptr_eq(&self.shared.owner, &registry.implementation_owner()) {
            return Err(super::mismatch(
                "validation context",
                "context belongs to another registry",
            ));
        }
        Ok(())
    }
    /// Prepare the actual declared relation once in this admission scope.
    /// # Errors
    /// Foreign registry, invalid declaration or native planning failure.
    pub fn relation(
        &self,
        registry: &Registry,
        spec: &RelationSpec,
    ) -> Result<Arc<PreparedLocalContract>, RelationError> {
        let contract = registry.contract(spec)?;
        self.require_registry(registry)?;
        let relations =
            self.scoped.relations.lock().map_err(|_| {
                super::mismatch("validation context", "relation cache lock poisoned")
            })?;
        if let Some(prepared) = relations.get(&contract.relation_id()) {
            return Ok(prepared.clone());
        }
        drop(relations);
        let prepared = self.prepare(registry, contract.schema().clone())?;
        self.scoped
            .relations
            .lock()
            .map_err(|_| super::mismatch("validation context", "relation cache lock poisoned"))?
            .insert(contract.relation_id(), prepared.clone());
        Ok(prepared)
    }
    /// Prepare a column in this same actual native context and scope.
    /// # Errors
    /// Invalid declaration, foreign registry or native planning failure.
    pub fn column(
        &self,
        registry: &Registry,
        field: &Field,
    ) -> Result<Arc<PreparedLocalContract>, RelationError> {
        self.require_registry(registry)?;
        let columns = self
            .scoped
            .columns
            .lock()
            .map_err(|_| super::mismatch("validation context", "column cache lock poisoned"))?;
        if let Some(prepared) = columns.get(field) {
            return Ok(prepared.clone());
        }
        drop(columns);
        let prepared = self.prepare(registry, Arc::new(Schema::new(vec![field.clone()])))?;
        self.scoped
            .columns
            .lock()
            .map_err(|_| super::mismatch("validation context", "column cache lock poisoned"))?
            .insert(field.clone(), prepared.clone());
        Ok(prepared)
    }
    /// Bind SQL once under the actual owner; scope-sensitive lowering is invocation-relative.
    /// # Errors
    /// Foreign registry, invalid declaration, native failure or preparation cycle.
    pub fn prepare(
        &self,
        registry: &Registry,
        schema: SchemaRef,
    ) -> Result<Arc<PreparedLocalContract>, RelationError> {
        self.require_registry(registry)?;
        let key = {
            let mut slots = self.shared.prepared.lock().map_err(|_| {
                super::mismatch("validation context", "prepared cache lock poisoned")
            })?;
            slots.slot(&schema)?
        };
        let template = self
            .shared
            .implementations
            .get_at(key, || {
                let result = self.construct(registry, schema);
                if result.is_ok() {
                    self.shared
                        .successful
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
                Ok(TemplateCell(result.map_err(Arc::new)))
            })?
            .0
            .clone()
            .map_err(RelationError::Preparation)?;
        if template.scoped.is_empty() {
            return Ok(template.immutable.clone());
        }
        let prepared = self.scoped.implementations.get_at(key, || {
            Ok(PreparedCell(
                self.prepare_scoped(&template).map_err(Arc::new),
            ))
        })?;
        prepared.0.clone().map_err(RelationError::Preparation)
    }
    fn construct(
        &self,
        registry: &Registry,
        schema: SchemaRef,
    ) -> Result<Arc<ContractTemplate>, RelationError> {
        for field in schema.fields() {
            super::validate_field(registry, field)
                .map_err(|errors| RelationError::Validation { errors })?;
        }
        let native = DFSchema::try_from(schema.as_ref().clone()).map_err(engine)?;
        let nodes = schema
            .fields()
            .iter()
            .map(|field| {
                super::occurrences::Node::prepare(
                    &self.shared.domains,
                    &self.shared.state,
                    Arc::clone(field),
                )
            })
            .collect::<Result<_, _>>()?;
        let mut predicates = Vec::new();
        let mut scoped = Vec::new();
        for (name, expression) in
            super::row_checks::bind(&schema, &self.shared.state).map_err(engine)?
        {
            let rule = format!("check:{name}");
            // Classify the retained bound expression before simplification can fold Stable calls.
            if scope_sensitive(&expression).map_err(engine)? {
                scoped.push((rule, expression));
            } else {
                predicates.push(PreparedPredicate {
                    scoped: false,
                    rule,
                    physical: prepare_expression(&self.shared.state, expression, &native)?,
                });
            }
        }
        Ok(Arc::new(ContractTemplate {
            immutable: Arc::new(PreparedLocalContract {
                native_owner: self.shared.native_owner.clone(),
                scope: None,
                immutable: None,
                logical_scoped: Vec::new(),
                schema,
                predicates: Arc::new(predicates),
                nodes: Arc::new(nodes),
                evaluations: std::sync::atomic::AtomicUsize::new(0),
                #[cfg(test)]
                structural_scans: std::sync::atomic::AtomicUsize::new(0),
            }),
            scoped,
        }))
    }
    fn prepare_scoped(
        &self,
        template: &ContractTemplate,
    ) -> Result<Arc<PreparedLocalContract>, RelationError> {
        let base = &template.immutable;
        let native = DFSchema::try_from(base.schema.as_ref().clone()).map_err(engine)?;
        let mut predicates = base.predicates.as_ref().clone();
        for (rule, expression) in &template.scoped {
            predicates.push(PreparedPredicate {
                scoped: true,
                rule: rule.clone(),
                physical: self
                    .shared
                    .state
                    .prepare_in_scope(expression.clone(), &native, &self.scope)
                    .map_err(engine)?,
            });
        }
        Ok(Arc::new(PreparedLocalContract {
            native_owner: base.native_owner.clone(),
            scope: Some(self.scope.clone()),
            immutable: Some(base.clone()),
            logical_scoped: template
                .scoped
                .iter()
                .map(|(_, expression)| expression.clone())
                .collect(),
            schema: base.schema.clone(),
            predicates: Arc::new(predicates),
            nodes: base.nodes.clone(),
            evaluations: std::sync::atomic::AtomicUsize::new(0),
            #[cfg(test)]
            structural_scans: std::sync::atomic::AtomicUsize::new(0),
        }))
    }
    /// Exact schema binding/template preparations, independent of invocation lowering.
    /// # Errors
    /// A poisoned preparation lock.
    pub fn prepared_count(&self) -> Result<usize, RelationError> {
        Ok(self
            .shared
            .successful
            .load(std::sync::atomic::Ordering::Relaxed))
    }
}
fn scope_sensitive(expression: &Expr) -> datafusion_common::Result<bool> {
    use datafusion_common::tree_node::TreeNode;
    use datafusion_expr::Volatility;
    expression.exists(|expression| {
        Ok(match expression {
            Expr::ScalarFunction(value) => {
                value.func.signature().volatility != Volatility::Immutable
            }
            Expr::AggregateFunction(value) => {
                value.func.signature().volatility != Volatility::Immutable
            }
            Expr::WindowFunction(value) => {
                value.fun.signature().volatility != Volatility::Immutable
            }
            Expr::HigherOrderFunction(value) => {
                value.func.signature().volatility != Volatility::Immutable
            }
            Expr::ScalarVariable(..) => true,
            _ => false,
        })
    })
}

/// Direct expression planning needs the native simplifier as well as coercion.
/// In particular, DataFusion's `coalesce` has no executable scalar kernel.
/// # Errors
/// Lambda binding, coercion, simplification or physical planning fails.
pub fn prepare_expression(
    state: &dyn super::planner::ValidationPlanner,
    expression: Expr,
    schema: &DFSchema,
) -> Result<Arc<dyn PhysicalExpr>, RelationError> {
    state.prepare(expression, schema).map_err(engine)
}
#[derive(Clone, Debug)]
struct PreparedPredicate {
    scoped: bool,
    rule: String,
    physical: Arc<dyn PhysicalExpr>,
}

/// Prepared expressions and exact input fields. No planning occurs during evaluation.
#[derive(Debug)]
pub struct PreparedLocalContract {
    native_owner: Arc<()>,
    scope: Option<AdmissionScope>,
    immutable: Option<Arc<PreparedLocalContract>>,
    logical_scoped: Vec<Expr>,
    schema: SchemaRef,
    predicates: Arc<Vec<PreparedPredicate>>,
    nodes: Arc<Vec<super::occurrences::Node>>,
    evaluations: std::sync::atomic::AtomicUsize,
    #[cfg(test)]
    structural_scans: std::sync::atomic::AtomicUsize,
}
impl PreparedLocalContract {
    pub(crate) fn same_native_owner(&self, other: &Self) -> bool {
        self.same_bindings(other)
            && match (&self.scope, &other.scope) {
                (None, None) => true,
                (Some(left), Some(right)) => left.same_invocation(right),
                _ => false,
            }
    }
    pub(crate) fn same_bindings(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.native_owner, &other.native_owner)
    }
    /// Whether successful row evidence depends on the actual admission invocation.
    pub fn has_scope_checks(&self) -> bool {
        self.predicates.iter().any(|predicate| predicate.scoped)
    }
    pub(crate) fn scope_checks(&self) -> std::collections::BTreeSet<String> {
        self.predicates
            .iter()
            .filter(|predicate| predicate.scoped)
            .map(|predicate| predicate.rule.clone())
            .collect()
    }
    /// Bind immutable obligations plus visible invocation-relative logical checks.
    /// Stable checks remain native expressions so prepared-query freshness can see
    /// them and physical planning uses the actual execution timestamp.
    /// # Errors
    /// Argument extent or a bound column differs from the exact prepared schema.
    pub fn predicate(self: &Arc<Self>, arguments: Vec<Expr>) -> datafusion_common::Result<Expr> {
        use datafusion_common::tree_node::{Transformed, TreeNode};
        if arguments.len() != self.schema.fields().len() {
            return Err(DataFusionError::Plan(
                "local contract argument extent differs".into(),
            ));
        }
        let immutable = self.immutable.as_ref().unwrap_or(self);
        let mut result = predicate::expression(Arc::clone(immutable), arguments.clone());
        for expression in &self.logical_scoped {
            let projected = expression
                .clone()
                .transform_up(|expression| {
                    if let Expr::Column(column) = expression {
                        let index = self.schema.index_of(&column.name)?;
                        let argument = arguments.get(index).ok_or_else(|| {
                            DataFusionError::Plan("local contract argument absent".into())
                        })?;
                        Ok(Transformed::yes(argument.clone()))
                    } else {
                        Ok(Transformed::no(expression))
                    }
                })?
                .data;
            result = result.and(projected);
        }
        Ok(result)
    }
    /// Exact declared input schema.
    pub fn schema(&self) -> &SchemaRef {
        &self.schema
    }
    /// Evaluate all local obligations with bounded retained samples and cancellation.
    /// False and SQL null both violate. Structural safety is checked first.
    /// # Errors
    /// Schema mismatch, malformed arrays, cancellation or native execution failure.
    pub fn evaluate(
        &self,
        batch: &RecordBatch,
        maximum_findings: usize,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<ValidationReport, RelationError> {
        self.evaluate_selected(batch, maximum_findings, cancel, None, None, false)
    }

    pub(crate) fn evaluate_missing_checks(
        &self,
        checked: &crate::columnar::FieldCheckedBatch,
        missing: &std::collections::BTreeSet<String>,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<ValidationReport, RelationError> {
        self.evaluate_selected(
            checked.structurally_trusted_batch(),
            16,
            cancel,
            Some(missing),
            None,
            true,
        )
    }

    // Only the private storage owner can supply this structural premise.
    pub(crate) fn evaluate_retained(
        &self,
        checked: &crate::columnar::FieldCheckedBatch,
        maximum_findings: usize,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<ValidationReport, RelationError> {
        self.evaluate_selected(
            checked.structurally_trusted_batch(),
            maximum_findings,
            cancel,
            None,
            None,
            true,
        )
    }

    pub(crate) fn evaluate_retained_ranges(
        &self,
        checked: &crate::columnar::FieldCheckedBatch,
        ranges: &[(usize, usize)],
        maximum_findings: usize,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<ValidationReport, RelationError> {
        self.evaluate_selected(
            checked.structurally_trusted_batch(),
            maximum_findings,
            cancel,
            None,
            Some(ranges),
            true,
        )
    }

    /// Actual local admission attempts, including rejected batches.
    pub fn evaluation_count(&self) -> usize {
        self.evaluations.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn evaluate_selected(
        &self,
        batch: &RecordBatch,
        maximum_findings: usize,
        cancel: &pse_columnar::CancellationToken,
        missing: Option<&std::collections::BTreeSet<String>>,
        ranges: Option<&[(usize, usize)]>,
        structurally_trusted: bool,
    ) -> Result<ValidationReport, RelationError> {
        self.evaluations
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if !Arc::ptr_eq(&self.schema, &batch.schema())
            && !same_schema(&self.schema, &batch.schema())
        {
            return Err(super::mismatch(
                "local validation",
                "prepared and actual schema differ",
            ));
        }
        cancel.checkpoint()?;
        if !structurally_trusted {
            #[cfg(test)]
            self.structural_scans
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            for column in batch.columns() {
                cancel.checkpoint()?;
                column.to_data().validate_full()?;
            }
        }
        let mut findings = Findings::new(batch, maximum_findings, cancel);
        let all = [(0, batch.num_rows())];
        for &(offset, length) in ranges.unwrap_or(&all) {
            let end = offset
                .checked_add(length)
                .filter(|end| *end <= batch.num_rows())
                .ok_or_else(|| {
                    super::mismatch(
                        "local validation",
                        "evaluation segment outside retained storage",
                    )
                })?;
            for start in (offset..end).step_by(super::occurrences::CHUNK) {
                cancel.checkpoint()?;
                let input = batch.slice(start, (end - start).min(super::occurrences::CHUNK));
                if missing.is_none() {
                    for (node, column) in self.nodes.iter().zip(input.columns()) {
                        node.root(column, start, &mut findings)?;
                    }
                }
                for predicate in self.predicates.iter() {
                    if missing.is_some_and(|names| !names.contains(&predicate.rule)) {
                        continue;
                    }
                    cancel.checkpoint()?;
                    let result = predicate
                        .physical
                        .evaluate(&input)
                        .and_then(|value| value.into_array(input.num_rows()))
                        .map_err(engine)?;
                    let result =
                        result
                            .as_any()
                            .downcast_ref::<BooleanArray>()
                            .ok_or_else(|| {
                                super::mismatch(
                                    "local validation",
                                    "predicate did not return Boolean",
                                )
                            })?;
                    for row in 0..result.len() {
                        if result.is_null(row) || !result.value(row) {
                            findings.row_check(&predicate.rule, start + row)?;
                        }
                    }
                }
            }
        }
        findings.finish()
    }
}

/// Native findings and explicit sampling completeness; truncation never means validity.
#[derive(Debug)]
pub struct ValidationReport {
    /// Typed findings with exact row locators and lossless observations.
    pub findings: RecordBatch,
    /// Total violations in the completely evaluated input.
    pub violations: usize,
    /// The retained findings are a bounded sample rather than the complete set.
    pub truncated: bool,
    /// Complete per-root-row validity, independent of the bounded finding sample.
    pub valid_rows: BooleanArray,
}
impl ValidationReport {
    /// A completed report is valid only when no predicate failed.
    pub const fn is_valid(&self) -> bool {
        self.violations == 0 && !self.truncated
    }
    /// Refuse invalid/partial reports while retaining their native findings.
    /// # Errors
    /// One or more obligations failed or findings were truncated.
    pub fn require_valid(self) -> Result<(), RelationError> {
        if self.is_valid() {
            Ok(())
        } else {
            Err(RelationError::LocalFindings {
                report: Box::new(self),
            })
        }
    }
}
fn same_schema(left: &Schema, right: &Schema) -> bool {
    left.metadata() == right.metadata()
        && left.fields().len() == right.fields().len()
        && left
            .fields()
            .iter()
            .zip(right.fields())
            .all(|(left, right)| {
                FieldContract::from_field(left.as_ref().clone())
                    == FieldContract::from_field(right.as_ref().clone())
            })
}
pub(super) fn engine(error: DataFusionError) -> RelationError {
    pse_columnar::classify(error, pse_columnar::PlanOrigin::RuleCompiler).into()
}

pub(super) struct Findings<'a> {
    batch: &'a RecordBatch,
    maximum: usize,
    pub cancel: &'a pse_columnar::CancellationToken,
    violations: usize,
    invalid_rows: Vec<bool>,
    rules: Vec<String>,
    rows: Vec<u64>,
    paths: Vec<String>,
    observations: Vec<Option<String>>,
    fields: Vec<Option<String>>,
    keys: Vec<Option<String>>,
}
impl<'a> Findings<'a> {
    fn new(
        batch: &'a RecordBatch,
        maximum: usize,
        cancel: &'a pse_columnar::CancellationToken,
    ) -> Self {
        Self {
            batch,
            maximum,
            cancel,
            violations: 0,
            invalid_rows: vec![false; batch.num_rows()],
            rules: vec![],
            rows: vec![],
            paths: vec![],
            observations: vec![],
            fields: vec![],
            keys: vec![],
        }
    }
    pub(super) fn add(
        &mut self,
        rule: &str,
        field: &Field,
        array: &arrow::array::ArrayRef,
        index: usize,
        traversal: &super::occurrences::Traversal,
    ) -> Result<(), RelationError> {
        let row = traversal.row(index);
        self.invalid_rows[row] = true;
        if self.count()? {
            let location = traversal.location(index);
            let literal = pse_schema::literal::to_json(array.as_ref(), field, index)?;
            let field = pse_columnar::native_field::canonical_json(field)
                .map_err(|error| super::mismatch("validation findings", error.to_string()))?;
            self.retain(
                rule,
                location.row,
                &location.path.render(),
                Some(literal),
                Some(field),
            )?;
        }
        Ok(())
    }
    fn count(&mut self) -> Result<bool, RelationError> {
        self.violations = self
            .violations
            .checked_add(1)
            .ok_or_else(|| super::mismatch("validation findings", "count overflow"))?;
        Ok(self.rows.len() < self.maximum)
    }
    fn row_check(&mut self, rule: &str, row: usize) -> Result<(), RelationError> {
        self.invalid_rows[row] = true;
        if self.count()? {
            self.retain(rule, row, "", None, None)?;
        }
        Ok(())
    }
    fn retain(
        &mut self,
        rule: &str,
        row: usize,
        path: &str,
        observed: Option<String>,
        field: Option<String>,
    ) -> Result<(), RelationError> {
        let mut keys = BTreeMap::new();
        for (field, values) in self
            .batch
            .schema()
            .fields()
            .iter()
            .zip(self.batch.columns())
        {
            if field
                .metadata()
                .get(pse_schema::arrow::KEY_ROLE)
                .is_some_and(|role| role == "key")
            {
                keys.insert(
                    field.name().clone(),
                    pse_schema::literal::to_json(values.as_ref(), field, row)?,
                );
            }
        }
        self.keys.push(if keys.is_empty() {
            None
        } else {
            Some(
                serde_json::to_string(&keys)
                    .map_err(|error| super::mismatch("validation findings", error.to_string()))?,
            )
        });
        self.rules.push(rule.into());
        self.rows.push(
            u64::try_from(row)
                .map_err(|_| super::mismatch("validation findings", "row overflow"))?,
        );
        self.paths.push(path.into());
        self.observations.push(observed);
        self.fields.push(field);
        Ok(())
    }
    fn finish(self) -> Result<ValidationReport, RelationError> {
        let retained = self.rows.len();
        let schema = self.batch.schema();
        let relation = schema
            .metadata()
            .get(pse_schema::arrow::KEY_CONTRACT_ID)
            .map(String::as_str);
        let findings = RecordBatch::try_new(
            pse_schema::validation::findings_schema()?,
            vec![
                Arc::new(StringArray::from(self.rules)),
                Arc::new(StringArray::from(
                    vec![pse_diagnostics::DiagnosticCode::ValidationInvariant.as_str(); retained],
                )),
                Arc::new(
                    arrow_array::FixedSizeBinaryArray::try_from_sparse_iter_with_size(
                        std::iter::repeat_n(
                            relation
                                .map(pse_ids::SemanticId::parse_hex)
                                .transpose()
                                .map_err(|error| engine(pse_columnar::external(error)))?
                                .map(|id| *id.as_bytes()),
                            retained,
                        ),
                        16,
                    )?,
                ),
                Arc::new(UInt64Array::from(self.rows)),
                Arc::new(StringArray::from(self.keys)),
                Arc::new(StringArray::from(self.paths)),
                Arc::new(StringArray::from(self.fields)),
                Arc::new(StringArray::from(self.observations)),
            ],
        )?;
        Ok(ValidationReport {
            findings,
            violations: self.violations,
            truncated: retained < self.violations,
            valid_rows: BooleanArray::from_iter(
                self.invalid_rows.into_iter().map(|invalid| Some(!invalid)),
            ),
        })
    }
}

#[cfg(test)]
mod consolidation_unit;

mod predicate;

#[cfg(test)]
mod owner_tests {
    use super::*;
    #[test]
    fn retained_validation_bindings_do_not_identify_an_invocation_or_equal_registry() {
        let registry = pse_schema::registry().unwrap();
        let original = ValidationContext::new(registry, super::super::planner::LocalPlanner);
        let child = original.new_scope();
        let foreign = ValidationContext::new(registry, super::super::planner::LocalPlanner);
        assert!(original.same_bindings(&original.clone()));
        assert!(original.same_bindings(&child));
        assert!(
            !original
                .admission_scope()
                .same_invocation(child.admission_scope())
        );
        assert!(!original.same_bindings(&foreign));
    }
}
