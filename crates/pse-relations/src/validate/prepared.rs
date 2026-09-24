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

/// One successful expression compilation, shared without holding the map lock.
type PreparedCell = Arc<Mutex<Option<Arc<PreparedLocalContract>>>>;

/// An immutable native implementation owner. Changing state requires a new context.
#[derive(Debug)]
pub struct ValidationContext {
    owner: Arc<()>,
    state: Arc<dyn super::planner::ValidationPlanner>,
    prepared: Mutex<HashMap<SchemaRef, PreparedCell>>,
    columns: Mutex<HashMap<Field, Arc<PreparedLocalContract>>>,
    relations: Mutex<BTreeMap<pse_ids::SemanticId, Arc<PreparedLocalContract>>>,
}
// The private key prevents another subsystem installing a different default policy.
struct DefaultValidationContext {
    local: Arc<ValidationContext>,
    native: std::sync::OnceLock<Arc<ValidationContext>>,
}
impl ValidationContext {
    /// Capture the actual function, rewrite and configuration bindings for preparation.
    pub fn new(
        registry: &Registry,
        state: impl super::planner::ValidationPlanner + 'static,
    ) -> Self {
        Self {
            owner: registry.implementation_owner(),
            state: Arc::new(state),
            prepared: Mutex::new(HashMap::new()),
            columns: Mutex::new(HashMap::new()),
            relations: Mutex::new(BTreeMap::new()),
        }
    }
    /// Registry-owned native defaults for generated construction and raw relation admission.
    /// Explicit sessions use `new`, never this default cache.
    /// # Errors
    /// A poisoned registry cache.
    pub fn for_registry(registry: &Registry) -> Result<Arc<Self>, RelationError> {
        let owner = Self::defaults(registry)?;
        Ok(owner.native.get().unwrap_or(&owner.local).clone())
    }
    fn defaults(registry: &Registry) -> Result<Arc<DefaultValidationContext>, RelationError> {
        Ok(registry.derived_implementation(|| {
            Ok(DefaultValidationContext {
                local: Arc::new(Self::new(registry, super::planner::LocalPlanner)),
                native: std::sync::OnceLock::new(),
            })
        })?)
    }
    /// Install the registry's immutable native SQL binding at its engine-owned boundary.
    /// Repeated installation keeps the first owner; explicit sessions use `new`.
    /// # Errors
    /// Registry cache failure.
    pub fn install_default(
        registry: &Registry,
        planner: impl super::planner::ValidationPlanner + 'static,
    ) -> Result<(), RelationError> {
        let defaults = Self::defaults(registry)?;
        defaults
            .native
            .get_or_init(|| Arc::new(Self::new(registry, planner)));
        Ok(())
    }
    /// Prepare the actual declared relation once inside this context.
    /// # Errors
    /// Foreign registry, invalid declaration or native planning failure.
    pub fn relation(
        &self,
        registry: &Registry,
        spec: &RelationSpec,
    ) -> Result<Arc<PreparedLocalContract>, RelationError> {
        let contract = registry.contract(spec)?;
        if !Arc::ptr_eq(&self.owner, &registry.implementation_owner()) {
            return Err(super::mismatch(
                "validation context",
                "context belongs to another registry",
            ));
        }
        let relations = self
            .relations
            .lock()
            .map_err(|_| super::mismatch("validation context", "relation cache lock poisoned"))?;
        if let Some(prepared) = relations.get(&contract.relation_id()) {
            return Ok(prepared.clone());
        }
        drop(relations);
        let prepared = self.prepare(registry, contract.schema().clone())?;
        self.relations
            .lock()
            .map_err(|_| super::mismatch("validation context", "relation cache lock poisoned"))?
            .insert(contract.relation_id(), prepared.clone());
        Ok(prepared)
    }
    /// Prepare a column contract in this same native context.
    /// # Errors
    /// Invalid field declaration, foreign registry or native planning failure.
    pub fn column(
        &self,
        registry: &Registry,
        field: &Field,
    ) -> Result<Arc<PreparedLocalContract>, RelationError> {
        if !Arc::ptr_eq(&self.owner, &registry.implementation_owner()) {
            return Err(super::mismatch(
                "validation context",
                "context belongs to another registry",
            ));
        }
        let columns = self
            .columns
            .lock()
            .map_err(|_| super::mismatch("validation context", "column cache lock poisoned"))?;
        if let Some(prepared) = columns.get(field) {
            return Ok(prepared.clone());
        }
        drop(columns);
        let prepared = self.prepare(registry, Arc::new(Schema::new(vec![field.clone()])))?;
        self.columns
            .lock()
            .map_err(|_| super::mismatch("validation context", "column cache lock poisoned"))?
            .insert(field.clone(), prepared.clone());
        Ok(prepared)
    }
    /// Prepare external native fields in this context's actual immutable session.
    /// # Errors
    /// A foreign registry, invalid fields, or a native expression binding failure.
    pub fn prepare(
        &self,
        registry: &Registry,
        schema: SchemaRef,
    ) -> Result<Arc<PreparedLocalContract>, RelationError> {
        if !Arc::ptr_eq(&self.owner, &registry.implementation_owner()) {
            return Err(super::mismatch(
                "validation context",
                "context belongs to another registry",
            ));
        }
        let cell = self
            .prepared
            .lock()
            .map_err(|_| super::mismatch("validation context", "prepared cache lock poisoned"))?
            .entry(schema.clone())
            .or_default()
            .clone();
        let mut prepared = cell
            .lock()
            .map_err(|_| super::mismatch("validation context", "prepared entry lock poisoned"))?;
        if let Some(found) = prepared.as_ref() {
            return Ok(found.clone());
        }
        for field in schema.fields() {
            super::validate_field(registry, field)
                .map_err(|errors| RelationError::Validation { errors })?;
        }
        let native = DFSchema::try_from(schema.as_ref().clone()).map_err(engine)?;
        let mut predicates = Vec::new();
        let nodes = schema
            .fields()
            .iter()
            .map(|field| {
                super::occurrences::Node::prepare(registry, &self.state, Arc::clone(field))
            })
            .collect::<Result<_, _>>()?;
        for (name, expression) in super::row_checks::bind(&schema, &self.state).map_err(engine)? {
            predicates.push(self.predicate(&native, format!("check:{name}"), expression)?);
        }
        let contract = Arc::new(PreparedLocalContract {
            schema,
            predicates,
            nodes,
            evaluations: std::sync::atomic::AtomicUsize::new(0),
        });
        *prepared = Some(Arc::clone(&contract));
        Ok(contract)
    }
    fn predicate(
        &self,
        schema: &DFSchema,
        rule: String,
        expression: Expr,
    ) -> Result<PreparedPredicate, RelationError> {
        let physical = prepare_expression(&self.state, expression, schema)?;
        Ok(PreparedPredicate { rule, physical })
    }
    /// Number of exact schema preparations, useful for deterministic reuse assertions.
    /// # Errors
    /// A poisoned preparation lock.
    pub fn prepared_count(&self) -> Result<usize, RelationError> {
        let cells: Vec<_> = self
            .prepared
            .lock()
            .map_err(|_| super::mismatch("validation context", "prepared cache lock poisoned"))?
            .values()
            .cloned()
            .collect();
        cells.iter().try_fold(0, |count, cell| {
            let prepared = cell.lock().map_err(|_| {
                super::mismatch("validation context", "prepared entry lock poisoned")
            })?;
            Ok(count + usize::from(prepared.is_some()))
        })
    }
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
#[derive(Debug)]
struct PreparedPredicate {
    rule: String,
    physical: Arc<dyn PhysicalExpr>,
}

/// Prepared expressions and exact input fields. No planning occurs during evaluation.
#[derive(Debug)]
pub struct PreparedLocalContract {
    schema: SchemaRef,
    predicates: Vec<PreparedPredicate>,
    nodes: Vec<super::occurrences::Node>,
    evaluations: std::sync::atomic::AtomicUsize,
}
impl PreparedLocalContract {
    /// Bind this prepared implementation as a visible native logical-plan predicate.
    pub fn predicate(self: &Arc<Self>, arguments: Vec<Expr>) -> Expr {
        predicate::expression(Arc::clone(self), arguments)
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
        self.evaluate_selected(batch, maximum_findings, cancel, None)
    }

    pub(crate) fn evaluate_missing_checks(
        &self,
        batch: &RecordBatch,
        missing: &std::collections::BTreeSet<String>,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<ValidationReport, RelationError> {
        self.evaluate_selected(batch, 16, cancel, Some(missing))
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
        for column in batch.columns() {
            column.to_data().validate_full()?;
        }
        let mut findings = Findings::new(batch, maximum_findings, cancel);
        for start in (0..batch.num_rows()).step_by(super::occurrences::CHUNK) {
            cancel.checkpoint()?;
            let input = batch.slice(
                start,
                (batch.num_rows() - start).min(super::occurrences::CHUNK),
            );
            if missing.is_none() {
                for (node, column) in self.nodes.iter().zip(input.columns()) {
                    node.root(column, start, &mut findings)?;
                }
            }
            for predicate in &self.predicates {
                if missing.is_some_and(|names| !names.contains(&predicate.rule)) {
                    continue;
                }
                cancel.checkpoint()?;
                let result = predicate
                    .physical
                    .evaluate(&input)
                    .and_then(|value| value.into_array(input.num_rows()))
                    .map_err(engine)?;
                let result = result
                    .as_any()
                    .downcast_ref::<BooleanArray>()
                    .ok_or_else(|| {
                        super::mismatch("local validation", "predicate did not return Boolean")
                    })?;
                for row in 0..result.len() {
                    if result.is_null(row) || !result.value(row) {
                        findings.row_check(&predicate.rule, start + row)?;
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
        location: &super::occurrences::Location,
    ) -> Result<(), RelationError> {
        self.invalid_rows[location.row] = true;
        if self.count()? {
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
