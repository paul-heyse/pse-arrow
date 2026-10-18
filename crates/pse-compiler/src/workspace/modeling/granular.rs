// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exclusive candidate phase publication. No database handle or candidate product escapes.
use super::*;
use pse_modeling::check::phases::{
    self, Phases, ResolutionEnvironment, ResolvedSignature, Signature,
};
use pse_modeling::document::Documents;
use pse_modeling::expression::occurrences::{CheckedExpression, OccurrenceKey, ParsedFields};
use std::cell::{Cell, RefCell};
type PhaseResult<T> = std::result::Result<T, pse_modeling::ModelingError>;

#[salsa::input]
struct Source {
    row: Arc<Declaration>,
    signature: Arc<Declaration>,
    fields: Option<BTreeMap<String, pse_authoring::SourceSpan>>,
    parsed: Option<Arc<ParsedFields>>,
}
#[salsa::input]
struct TypeInput {
    source: Source,
    environment: ResolutionEnvironment,
    scope: Arc<PhysicalScope>,
}
#[salsa::input]
struct CompleteInput {
    package: Arc<CheckedPackage>,
    documents: Arc<DocumentInventory>,
    inputs: BTreeMap<SemanticId, DataDocumentInput>,
    scope: Arc<PhysicalScope>,
    input_limit: usize,
    memo_limit: usize,
}
#[salsa::input]
struct OccurrenceInput {
    namespace: phases::Namespace,
    key: OccurrenceKey,
    expression: CheckedExpression,
}
#[salsa::tracked(returns(clone), lru=64, heap_size=parsed_heap)]
fn parsed(db: &dyn CompilerDb, source: Source) -> Result<Arc<ParsedFields>> {
    let _phase = pse_diagnostics::observation::phase("compiler.query.parsed");
    checkpoint(db);
    if let Some(parsed) = source.parsed(db)
        && parsed.matches(source.row(db), source.fields(db).as_ref())
    {
        return Ok(parsed.clone());
    }
    let value = ParsedFields::parse(source.row(db), source.fields(db).as_ref())?;
    checkpoint(db);
    Ok(Arc::new(value))
}
fn parsed_heap(v: &Result<Arc<ParsedFields>>) -> usize {
    v.as_ref().map_or(0, |p| p.retained_bytes())
}
#[salsa::tracked(returns(clone), lru=64, heap_size=parsed_heap)]
fn signature_syntax(db: &dyn CompilerDb, source: Source) -> Result<Arc<ParsedFields>> {
    let _phase = pse_diagnostics::observation::phase("compiler.query.signature_syntax");
    checkpoint(db);
    let value = parsed(db, source)?.signature_syntax();
    checkpoint(db);
    Ok(Arc::new(value))
}
#[salsa::tracked(returns(clone), lru=64, heap_size=resolved_heap)]
fn resolved(
    db: &dyn CompilerDb,
    inventory: Inventory,
    input: TypeInput,
) -> Result<Arc<ResolvedSignature>> {
    let _phase = pse_diagnostics::observation::phase("compiler.query.resolved");
    checkpoint(db);
    let source = input.source(db);
    let context = TypeContext {
        admissions: None,
        formula_authority: None,
        quantities: inventory.quantities(db),
        preconditions: inventory.preconditions(db),
        scope: input.scope(db),
    };
    let value = input
        .environment(db)
        .resolve(source.signature(db), &context)?;
    checkpoint(db);
    Ok(Arc::new(value))
}
fn resolved_heap(v: &Result<Arc<ResolvedSignature>>) -> usize {
    v.as_ref().map_or(0, |p| p.retained_bytes())
}
#[salsa::tracked(returns(clone), lru=64, heap_size=signature_heap)]
fn typed(db: &dyn CompilerDb, inventory: Inventory, input: TypeInput) -> Result<Arc<Signature>> {
    let _phase = pse_diagnostics::observation::phase("compiler.query.typed");
    checkpoint(db);
    let resolution = resolved(db, inventory, input)?;
    let source = *input.source(db);
    let syntax = signature_syntax(db, source)?;
    let context = TypeContext {
        admissions: None,
        formula_authority: None,
        quantities: inventory.quantities(db),
        preconditions: inventory.preconditions(db),
        scope: input.scope(db),
    };
    let mut value = phases::signature(source.signature(db), &syntax, &resolution, &context)?;
    value.admit_reduced(&input.environment(db).namespace)?;
    checkpoint(db);
    Ok(Arc::new(value))
}
fn signature_heap(v: &Result<Arc<Signature>>) -> usize {
    v.as_ref().map_or(0, |p| p.retained_bytes())
}
#[salsa::tracked(returns(clone), lru=64, heap_size=complete_heap)]
fn complete(
    db: &dyn CompilerDb,
    inventory: Inventory,
    input: CompleteInput,
) -> Result<Arc<CheckedPackage>> {
    let _phase = pse_diagnostics::observation::phase("compiler.query.complete");
    checkpoint(db);
    let context = TypeContext {
        admissions: None,
        formula_authority: None,
        quantities: inventory.quantities(db),
        preconditions: inventory.preconditions(db),
        scope: input.scope(db),
    };
    let documents = TrackedDocuments {
        db,
        inventory,
        documents: input.documents(db),
        inputs: input.inputs(db),
        pending: None,
        admission_bytes: Cell::new(0),
        admission_limit: *input.input_limit(db),
        memo_bytes: Cell::new(0),
        memo_limit: *input.memo_limit(db),
    };
    let value = phases::finish(input.package(db).as_ref().clone(), &context, &documents)?;
    checkpoint(db);
    Ok(Arc::new(value))
}
fn complete_heap(v: &Result<Arc<CheckedPackage>>) -> usize {
    v.as_ref().map_or(0, |p| p.retained_bytes())
}
#[salsa::tracked(returns(clone), lru=64, heap_size=occurrence_heap)]
fn occurrence(db: &dyn CompilerDb, input: OccurrenceInput) -> CheckedExpression {
    let _phase = pse_diagnostics::observation::phase("compiler.query.occurrence");
    checkpoint(db);
    let value = input
        .namespace(db)
        .bind(input.key(db), input.expression(db).clone());
    checkpoint(db);
    value
}
fn occurrence_heap(value: &CheckedExpression) -> usize {
    pse_modeling::expression::occurrences::field_retained_bytes(
        &OccurrenceKey {
            declaration: DeclarationId::from_id(SemanticId::NIL),
            role: String::new(),
            position: 0,
        },
        value,
        true,
    )
}

/// Generation-local input identities; bounded with the enclosing generation.
#[derive(Default)]
pub(in crate::workspace) struct State {
    sources: BTreeMap<DeclarationId, Source>,
    types: BTreeMap<DeclarationId, TypeInput>,
    occurrences: BTreeMap<OccurrenceKey, OccurrenceInput>,
    complete: Option<CompleteInput>,
    inventory: Option<InventoryInput>,
    members: BTreeMap<DeclarationId, MemberPhase>,
    expressions: BTreeMap<DeclarationId, ExpressionPhase>,
    expression_environment: Option<phases::expressions::Environment>,
    references: BTreeMap<DeclarationId, ReferencePhase>,
}
impl State {
    pub(in crate::workspace) fn entries(&self) -> usize {
        self.sources.len()
            + self.types.len()
            + self.occurrences.len()
            + self.members.len()
            + self.expressions.len()
            + self.references.len()
            + usize::from(self.expression_environment.is_some())
            + usize::from(self.complete.is_some())
            + usize::from(self.inventory.is_some())
    }
    pub(in crate::workspace) fn known_bytes(&self, db: &CompilerDatabase) -> usize {
        use pse_model::HeapUsage;
        let mut census = phases::RetainedInputs::default();
        // Salsa 0.28.4 input::Value::memory_usage reports metadata and inline fields.
        // These input declarations supply no heap_size callback: their live heap
        // roots are counted here. Tracked query outputs retain their own callbacks
        // and remain in CompilerWorkspace::retention_usage independently.
        census.add(self.entries().saturating_mul(256));
        census.add(self.occurrences.keys().map(|key| key.role.capacity()).sum());
        for input in self.sources.values() {
            for row in [input.row(db), input.signature(db)] {
                if census.include_shared(row) { census.add(row.owned_bytes()); }
            }
            if let Some(fields) = input.fields(db) {
                census.add(fields.keys().map(|key| key.capacity() + size_of::<pse_authoring::SourceSpan>() + 128).sum());
            }
            if let Some(parsed) = input.parsed(db) {
                if census.include_shared(parsed) { census.add(parsed.retained_bytes()); }
            }
        }
        for input in self.types.values() {
            input.environment(db).collect_retained_inputs(&mut census);
            if census.include_shared(input.scope(db)) { census.add(scope_bytes(input.scope(db))); }
        }
        for input in self.occurrences.values() {
            input.namespace(db).collect_retained_inputs(&mut census);
            census.add(pse_modeling::expression::occurrences::field_retained_bytes(input.key(db), input.expression(db), true));
        }
        for phase in self.members.values() { phase.input(db).collect_retained_inputs(&mut census); }
        for phase in self.expressions.values() {
            phase.input(db).collect_retained_inputs(&mut census);
            if census.include_shared(phase.scope(db)) { census.add(scope_bytes(phase.scope(db))); }
        }
        for phase in self.references.values() { phase.input(db).collect_retained_inputs(&mut census); }
        if let Some(environment) = &self.expression_environment { environment.collect_retained_inputs(&mut census); }
        if let Some(input) = self.complete {
            if census.include_shared(input.package(db)) { census.add(input.package(db).retained_bytes()); }
            if census.include_shared(input.documents(db)) { census.add(input.documents(db).retained_bytes()); }
            if census.include_shared(input.scope(db)) { census.add(scope_bytes(input.scope(db))); }
            collect_document_inputs(db, input.inputs(db), &mut census);
        }
        if let Some(input) = self.inventory {
            if census.include_shared(input.rows(db)) { census.add(input.rows(db).owned_bytes()); }
            if census.include_shared(input.documents(db)) { census.add(input.documents(db).retained_bytes()); }
            if census.include_shared(input.scope(db)) { census.add(scope_bytes(input.scope(db))); }
            census.add(input.sources(db).capacity().saturating_mul(size_of::<Source>()));
            collect_document_inputs(db, input.inputs(db), &mut census);
        }
        census.bytes()
    }
}
fn collect_document_inputs<'a>(
    db: &'a CompilerDatabase,
    inputs: &BTreeMap<SemanticId, DataDocumentInput>,
    census: &mut phases::RetainedInputs<'a>,
) {
    census.add(inputs.len().saturating_mul(size_of::<(SemanticId, DataDocumentInput)>() + 128));
    // DataDocumentInput also has no Salsa heap callback. A retained handle may
    // still select rows absent from the latest DocumentInventory.
    for input in inputs.values() {
        let document = input.document(db);
        if census.include_shared(document) { census.add(document.retained_bytes()); }
    }
}
pub(super) fn configure(db: &mut CompilerDatabase, capacity: usize) {
    enclosing_inventory::set_lru_capacity(db, capacity);
    parsed::set_lru_capacity(db, capacity);
    signature_syntax::set_lru_capacity(db, capacity);
    resolved::set_lru_capacity(db, capacity);
    typed::set_lru_capacity(db, capacity);
    complete::set_lru_capacity(db, capacity);
    occurrence::set_lru_capacity(db, capacity);
    effective_members::set_lru_capacity(db, capacity);
    checked_expressions::set_lru_capacity(db, capacity);
    expression_references::set_lru_capacity(db, capacity);
}
fn modeling_error(error: CompileError) -> pse_modeling::ModelingError {
    match error {
        CompileError::Modeling(error) => error,
        CompileError::Cancelled => pse_modeling::ModelingError::Cancelled,
        error => pse_modeling::ModelingError::Contract {
            declaration: SemanticId::NIL,
            message: error.to_string(),
        },
    }
}
/// A pure checker drives ordered phases. Setters run only between tracked calls under
/// exclusive candidate ownership, and every failed candidate rotates this generation.
struct Driver<'a> {
    db: RefCell<&'a mut CompilerDatabase>,
    state: RefCell<&'a mut State>,
    inventory: Inventory,
    documents: &'a Arc<DocumentInventory>,
    inputs: &'a BTreeMap<SemanticId, DataDocumentInput>,
    scope: &'a Arc<PhysicalScope>,
    bytes: Cell<usize>,
    limit: usize,
    memo_limit: usize,
}
impl Documents for Driver<'_> {
    fn field_spans(
        &self,
        id: DeclarationId,
    ) -> Option<&BTreeMap<String, pse_authoring::SourceSpan>> {
        self.documents.field_spans.get(&id)
    }
    fn parsed_fields(&self, id: DeclarationId) -> Option<&Arc<ParsedFields>> {
        self.documents.parsed_fields.get(&id)
    }
    fn resolve(&self, source: SemanticId, path: &str) -> Option<SemanticId> {
        self.documents.resolve(source, path)
    }
    fn rows(&self, id: SemanticId) -> Option<usize> {
        self.documents
            .documents
            .get(&id)
            .map(|document| document.rows.rows())
    }
    fn bytes(&self, id: SemanticId) -> usize {
        self.documents
            .documents
            .get(&id)
            .map_or(0, |document| document.retained_bytes())
    }
    fn preflight(&self, _: DeclarationId, bytes: usize) -> PhaseResult<()> {
        let db = self.db.borrow();
        if db.cancel.load(Ordering::Acquire) {
            return Err(pse_modeling::ModelingError::Cancelled);
        }
        let next = self.bytes.get().saturating_add(bytes);
        if next > self.limit {
            return Err(pse_modeling::ModelingError::Budget(
                "granular modeling candidate exceeds precharged workspace allowance".into(),
            ));
        }
        self.bytes.set(next);
        if let Some(context) = pse_diagnostics::observation::current() {
            context.gauge("compiler.candidate_preflight_bytes", "bytes", next as f64);
        }
        Ok(())
    }
    fn admit(
        &self,
        plan: &Arc<DocumentPlan>,
        document: SemanticId,
    ) -> PhaseResult<Arc<DocumentTable>> {
        let db = self.db.borrow();
        let tracked = TrackedDocuments {
            db: &**db,
            inventory: self.inventory,
            documents: self.documents,
            inputs: self.inputs,
            pending: None,
            admission_bytes: Cell::new(self.bytes.get()),
            admission_limit: self.limit,
            memo_bytes: Cell::new(0),
            memo_limit: self.memo_limit,
        };
        let value = tracked.admit(plan, document)?;
        self.bytes.set(tracked.admission_bytes.get());
        Ok(value)
    }
}
impl Phases for Driver<'_> {
    fn parse(&self, row: &Declaration, _: &dyn Documents) -> PhaseResult<Arc<ParsedFields>> {
        use pse_model::HeapUsage;
        self.preflight(row.declaration_id, row.owned_bytes().saturating_mul(3))?;
        let mut db = self.db.borrow_mut();
        let mut state = self.state.borrow_mut();
        let fields = self.documents.field_spans.get(&row.declaration_id).cloned();
        let supplied = self
            .documents
            .parsed_fields
            .get(&row.declaration_id)
            .cloned();
        let mut signature = row.clone();
        signature.source_start = 0;
        signature.source_end = 0;
        let input = if let Some(input) = state.sources.get(&row.declaration_id).copied() {
            if input.row(&**db).as_ref() != row {
                input.set_row(&mut **db).to(Arc::new(row.clone()));
            }
            if input.signature(&**db).as_ref() != &signature {
                input.set_signature(&mut **db).to(Arc::new(signature));
            }
            if input.fields(&**db) != &fields {
                input.set_fields(&mut **db).to(fields);
            }
            if input.parsed(&**db) != &supplied {
                input.set_parsed(&mut **db).to(supplied);
            }
            input
        } else {
            let input = Source::new(
                &**db,
                Arc::new(row.clone()),
                Arc::new(signature),
                fields,
                supplied,
            );
            state.sources.insert(row.declaration_id, input);
            input
        };
        parsed(&**db, input).map_err(modeling_error)
    }
    fn inventory(
        &self,
        rows: &[Declaration],
        _: &TypeContext<'_>,
        _: &dyn Documents,
    ) -> PhaseResult<phases::InventoryProduct> {
        use pse_model::HeapUsage;
        self.preflight(
            DeclarationId::from_id(SemanticId::NIL),
            rows.iter()
                .map(HeapUsage::owned_bytes)
                .sum::<usize>()
                .saturating_mul(2)
                .saturating_add(self.documents.retained_bytes()),
        )?;
        let mut db = self.db.borrow_mut();
        let mut state = self.state.borrow_mut();
        let sources = rows
            .iter()
            .map(|row| state.sources[&row.declaration_id])
            .collect::<Vec<_>>();
        let input = if let Some(input) = state.inventory {
            if input.rows(&**db).as_slice() != rows {
                input.set_rows(&mut **db).to(Arc::new(rows.to_vec()));
            }
            if input.documents(&**db) != self.documents {
                input.set_documents(&mut **db).to(self.documents.clone());
            }
            if input.sources(&**db) != &sources {
                input.set_sources(&mut **db).to(sources);
            }
            if input.inputs(&**db) != self.inputs {
                input.set_inputs(&mut **db).to(self.inputs.clone());
            }
            if input.scope(&**db) != self.scope {
                input.set_scope(&mut **db).to(self.scope.clone());
            }
            input
        } else {
            let input = InventoryInput::new(
                &**db,
                Arc::new(rows.to_vec()),
                self.documents.clone(),
                sources,
                self.inputs.clone(),
                self.scope.clone(),
                self.limit,
                self.memo_limit,
            );
            state.inventory = Some(input);
            input
        };
        enclosing_inventory(&**db, self.inventory, input)
            .map(|p| p.as_ref().clone())
            .map_err(modeling_error)
    }
    fn signature(
        &self,
        row: &Declaration,
        _: &Arc<ParsedFields>,
        environment: &ResolutionEnvironment,
        _: &TypeContext<'_>,
    ) -> PhaseResult<Signature> {
        let mut db = self.db.borrow_mut();
        let mut state = self.state.borrow_mut();
        let source = state.sources[&row.declaration_id];
        let input = if let Some(input) = state.types.get(&row.declaration_id).copied() {
            if input.environment(&**db) != environment {
                input.set_environment(&mut **db).to(environment.clone());
            }
            if input.scope(&**db) != self.scope {
                input.set_scope(&mut **db).to(self.scope.clone());
            }
            input
        } else {
            let input = TypeInput::new(&**db, source, environment.clone(), self.scope.clone());
            state.types.insert(row.declaration_id, input);
            input
        };
        typed(&**db, self.inventory, input)
            .map(|p| p.as_ref().clone())
            .map_err(modeling_error)
    }
    fn members(&self, input: &phases::MemberInput) -> PhaseResult<phases::MemberProduct> {
        self.preflight(input.declaration(), input.owned_contract_bytes())?;
        let mut db = self.db.borrow_mut();
        let mut state = self.state.borrow_mut();
        let id = input.declaration();
        let phase = if let Some(phase) = state.members.get(&id).copied() {
            if phase.input(&**db) != input {
                phase.set_input(&mut **db).to(input.clone());
            }
            phase
        } else {
            let phase = MemberPhase::new(&**db, input.clone());
            state.members.insert(id, phase);
            phase
        };
        effective_members(&**db, phase)
            .map(|p| p.as_ref().clone())
            .map_err(modeling_error)
    }
    fn finish(
        &self,
        package: CheckedPackage,
        _: &TypeContext<'_>,
        _: &dyn Documents,
    ) -> PhaseResult<CheckedPackage> {
        self.preflight(
            DeclarationId::from_id(SemanticId::NIL),
            package.retained_bytes().saturating_mul(2),
        )?;
        let mut db = self.db.borrow_mut();
        let mut state = self.state.borrow_mut();
        let input = if let Some(input) = state.complete {
            if input.package(&**db).as_ref() != &package {
                input.set_package(&mut **db).to(Arc::new(package));
            }
            if input.documents(&**db) != self.documents {
                input.set_documents(&mut **db).to(self.documents.clone());
            }
            if input.inputs(&**db) != self.inputs {
                input.set_inputs(&mut **db).to(self.inputs.clone());
            }
            if input.scope(&**db) != self.scope {
                input.set_scope(&mut **db).to(self.scope.clone());
            }
            input
        } else {
            let input = CompleteInput::new(
                &**db,
                Arc::new(package),
                self.documents.clone(),
                self.inputs.clone(),
                self.scope.clone(),
                self.limit,
                self.memo_limit,
            );
            state.complete = Some(input);
            input
        };
        complete(&**db, self.inventory, input)
            .map(|p| p.as_ref().clone())
            .map_err(modeling_error)
    }
    fn bind(&self, package: &mut CheckedPackage) -> PhaseResult<()> {
        self.preflight(
            DeclarationId::from_id(SemanticId::NIL),
            package.retained_bytes(),
        )?;
        let namespace = phases::Namespace::new(package);
        let expressions = package.take_unbound_occurrences();
        let mut output = BTreeMap::new();
        for (key, expression) in expressions {
            let mut db = self.db.borrow_mut();
            let mut state = self.state.borrow_mut();
            let input = if let Some(input) = state.occurrences.get(&key).copied() {
                if input.namespace(&**db) != &namespace {
                    input.set_namespace(&mut **db).to(namespace.clone());
                }
                if input.expression(&**db) != &expression {
                    input.set_expression(&mut **db).to(expression);
                }
                input
            } else {
                let input = OccurrenceInput::new(&**db, namespace.clone(), key.clone(), expression);
                state.occurrences.insert(key.clone(), input);
                input
            };
            output.insert(key, occurrence(&**db, input));
        }
        package.set_bound_occurrences(output);
        Ok(())
    }
    fn expressions(
        &self,
        package: &mut CheckedPackage,
        _: &TypeContext<'_>,
        _: &dyn Documents,
    ) -> PhaseResult<()> {
        let mut environment = phases::expressions::Environment::new(package, self)?;
        {
            let mut state = self.state.borrow_mut();
            if let Some(previous) = &state.expression_environment {
                environment.reuse_metadata(previous);
            }
            state.expression_environment = Some(environment.clone());
        }
        let mut references = BTreeMap::new();
        for id in package.declarations().map(|row| row.declaration_id) {
            self.preflight(id, 256)?;
            let input = environment.references(id)?;
            let mut db = self.db.borrow_mut();
            let mut state = self.state.borrow_mut();
            let phase = if let Some(phase) = state.references.get(&id).copied() {
                if phase.input(&**db) != &input {
                    phase.set_input(&mut **db).to(input);
                }
                phase
            } else {
                let phase = ReferencePhase::new(&**db, input);
                state.references.insert(id, phase);
                phase
            };
            references.insert(id, expression_references(&**db, phase));
        }
        let mut products = phases::expressions::Product::default();
        for id in package.declarations().map(|row| row.declaration_id) {
            let input = phases::expressions::Input::new(&environment, id, &references, self)?;
            let mut db = self.db.borrow_mut();
            let mut state = self.state.borrow_mut();
            let phase = if let Some(phase) = state.expressions.get(&id).copied() {
                if phase.input(&**db) != &input {
                    phase.set_input(&mut **db).to(input);
                }
                if phase.scope(&**db) != self.scope {
                    phase.set_scope(&mut **db).to(self.scope.clone());
                }
                phase
            } else {
                let phase = ExpressionPhase::new(&**db, input, self.scope.clone());
                state.expressions.insert(id, phase);
                phase
            };
            let product =
                checked_expressions(&**db, self.inventory, phase).map_err(modeling_error)?;
            products.merge(product.as_ref().clone(), id)?;
        }
        products.install(package)
    }
}
/// Stage exact source and evaluate pure tracked phases; caller publishes only on success.
pub(super) fn check(
    workspace: &mut CompilerWorkspace,
    rows: &[Declaration],
    scope: &Arc<PhysicalScope>,
    documents: &Arc<DocumentInventory>,
) -> Result<CheckedPackage> {
    let context = TypeContext {
        admissions: None,
        formula_authority: None,
        quantities: &workspace.inputs.quantities,
        preconditions: &workspace.inputs.preconditions,
        scope,
    };
    let driver = Driver {
        db: RefCell::new(&mut workspace.db),
        state: RefCell::new(&mut workspace.granular),
        inventory: workspace.inventory,
        documents,
        inputs: &workspace.documents,
        scope,
        bytes: Cell::new(0),
        limit: workspace.limits.input_bytes,
        memo_limit: workspace.limits.retained_bytes,
    };
    pse_modeling::check::check_with_phases(rows, &context, &driver, &driver)
        .map_err(CompileError::from)
}

#[salsa::input]
struct MemberPhase {
    input: phases::MemberInput,
}
#[salsa::tracked(returns(clone),lru=64,heap_size=members_heap)]
fn effective_members(
    db: &dyn CompilerDb,
    phase: MemberPhase,
) -> Result<Arc<phases::MemberProduct>> {
    let _phase = pse_diagnostics::observation::phase("compiler.query.effective_members");
    checkpoint(db);
    let value = phase.input(db).check()?;
    checkpoint(db);
    Ok(Arc::new(value))
}
fn members_heap(value: &Result<Arc<phases::MemberProduct>>) -> usize {
    value.as_ref().map_or(0, |value| value.retained_bytes())
}

#[salsa::input]
struct ExpressionPhase {
    input: phases::expressions::Input,
    scope: Arc<PhysicalScope>,
}
#[salsa::tracked(returns(clone),lru=64,heap_size=expression_heap)]
fn checked_expressions(
    db: &dyn CompilerDb,
    inventory: Inventory,
    phase: ExpressionPhase,
) -> Result<Arc<phases::expressions::Product>> {
    let _phase = pse_diagnostics::observation::phase("compiler.query.checked_expressions");
    checkpoint(db);
    let context = TypeContext {
        admissions: None,
        formula_authority: None,
        quantities: inventory.quantities(db),
        preconditions: inventory.preconditions(db),
        scope: phase.scope(db),
    };
    let product = phase.input(db).check(&context)?;
    checkpoint(db);
    Ok(Arc::new(product))
}
fn expression_heap(value: &Result<Arc<phases::expressions::Product>>) -> usize {
    value.as_ref().map_or(0, |value| value.retained_bytes())
}

#[salsa::input]
struct ReferencePhase {
    input: phases::expressions::ReferenceInput,
}
#[salsa::tracked(returns(clone),lru=64,heap_size=reference_heap)]
fn expression_references(
    db: &dyn CompilerDb,
    phase: ReferencePhase,
) -> Arc<BTreeSet<DeclarationId>> {
    let _phase = pse_diagnostics::observation::phase("compiler.query.expression_references");
    checkpoint(db);
    let value = phase.input(db).resolve();
    checkpoint(db);
    Arc::new(value)
}
fn reference_heap(value: &Arc<BTreeSet<DeclarationId>>) -> usize {
    value.len().saturating_mul(size_of::<DeclarationId>() + 128)
}

#[salsa::input]
struct InventoryInput {
    rows: Arc<Vec<Declaration>>,
    documents: Arc<DocumentInventory>,
    sources: Vec<Source>,
    inputs: BTreeMap<SemanticId, DataDocumentInput>,
    scope: Arc<PhysicalScope>,
    input_limit: usize,
    memo_limit: usize,
}
#[salsa::tracked(returns(clone),lru=64,heap_size=inventory_heap)]
fn enclosing_inventory(
    db: &dyn CompilerDb,
    inventory: Inventory,
    input: InventoryInput,
) -> Result<Arc<phases::InventoryProduct>> {
    let _phase = pse_diagnostics::observation::phase("compiler.query.enclosing_inventory");
    checkpoint(db);
    let mut documents = input.documents(db).as_ref().clone();
    for source in input.sources(db) {
        documents
            .parsed_fields
            .insert(source.row(db).declaration_id, parsed(db, *source)?);
    }
    let context = TypeContext {
        admissions: None,
        formula_authority: None,
        quantities: inventory.quantities(db),
        preconditions: inventory.preconditions(db),
        scope: input.scope(db),
    };
    let tracked = TrackedDocuments {
        db,
        inventory,
        documents: &documents,
        inputs: input.inputs(db),
        pending: None,
        admission_bytes: Cell::new(0),
        admission_limit: *input.input_limit(db),
        memo_bytes: Cell::new(0),
        memo_limit: *input.memo_limit(db),
    };
    let value = phases::inventory(input.rows(db), &context, &tracked)?;
    checkpoint(db);
    Ok(Arc::new(value))
}
fn inventory_heap(v: &Result<Arc<phases::InventoryProduct>>) -> usize {
    v.as_ref().map_or(0, |p| p.retained_bytes())
}
