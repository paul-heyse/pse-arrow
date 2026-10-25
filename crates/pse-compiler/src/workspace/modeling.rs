// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Generic kernel queries in the existing compiler database.
use super::*;
use pse_authoring::language::Declaration;
use pse_modeling::document::{DataDocument, DocumentInventory, DocumentPlan, DocumentTable};
use pse_modeling::{
    Bindings, CheckedPackage, DeclarationId, InstanceId, Limits, PhysicalScope, SpecializedModel,
    TypeContext,
};
use salsa::Setter;
pub(super) mod granular;
mod shared_packages;
use shared_packages::{
    StaticFunctionPackageKey, read_static_package, root_package, select_package,
};

/// Bounded immutable body retention owned by the mathematics service. No database
/// handles or attempt workers cross this interface; cache state never defines meaning.
/// Implementations perform only immutable in-memory lookup and allocation accounting.
/// Receiving-host qualification, persisted acquisition and publication belong outside
/// tracked compiler calls and run explicitly for every current consumer.
pub trait ModelingBodyRetention: std::fmt::Debug + Send + Sync + std::panic::RefUnwindSafe {
    /// Capture a retention epoch before admission begins.
    fn generation(&self) -> u64;
    /// Look up an immutable body by its complete admitted semantic dependency key.
    fn get(
        &self,
        key: pse_ids::roles::SemanticBodyHash,
    ) -> std::result::Result<Option<Arc<AdmittedBody>>, MathError>;
    /// Retain actual allocation ownership on the returned body, whether or not cached.
    fn retain(
        &self,
        generation: u64,
        key: pse_ids::roles::SemanticBodyHash,
        body: Arc<AdmittedBody>,
    ) -> std::result::Result<Arc<AdmittedBody>, MathError>;
}

/// One interpreted package data document (ADR-0125), set when its admitted value changes.
#[salsa::input]
pub(super) struct DataDocumentInput {
    document: Arc<DataDocument>,
}
/// A dataset's admission plan, equal for equal declarations.
#[salsa::interned(heap_size = plan_heap)]
struct PlanKey<'db> {
    plan: Arc<DocumentPlan>,
}
fn plan_heap((plan,): &(Arc<DocumentPlan>,)) -> usize {
    plan.retained_bytes()
}
/// ADR-0125, §22.2: an admitted table's document rows are a tracked query over the
/// dataset's plan, drawn from its table declaration, and the document input. An unchanged
/// plan over an unchanged document reuses the admitted rows; nothing else caches them.
#[salsa::tracked(returns(clone), lru = 64, heap_size = document_table_heap)]
fn document_table<'db>(
    db: &'db dyn CompilerDb,
    inventory: Inventory,
    plan: PlanKey<'db>,
    document: DataDocumentInput,
) -> Result<Arc<DocumentTable>> {
    let _phase = pse_diagnostics::observation::phase("compiler.query.document_table");
    checkpoint(db);
    let table = pse_modeling::document::admit(
        plan.plan(db),
        document.document(db),
        inventory.quantities(db),
    )?;
    checkpoint(db);
    Ok(Arc::new(table))
}
fn document_table_heap(v: &Result<Arc<DocumentTable>>) -> usize {
    v.as_ref().map_or(0, |v| v.retained_bytes())
}
/// The workspace's answer to the checker's document reads: resolution in the published
/// inventory and admission through [`document_table`]. While a physical registry that is not
/// yet published is checked (`pending`), rows are admitted against it without a memo, so no
/// result of the previous registry is read.
struct TrackedDocuments<'a> {
    db: &'a dyn CompilerDb,
    inventory: Inventory,
    documents: &'a DocumentInventory,
    inputs: &'a BTreeMap<SemanticId, DataDocumentInput>,
    pending: Option<&'a QuantityRegistry>,
    admission_bytes: std::cell::Cell<usize>,
    admission_limit: usize,
    memo_bytes: std::cell::Cell<usize>,
    memo_limit: usize,
}
impl pse_modeling::document::Documents for TrackedDocuments<'_> {
    fn field_spans(
        &self,
        declaration: DeclarationId,
    ) -> Option<&BTreeMap<String, pse_authoring::SourceSpan>> {
        self.documents.field_spans.get(&declaration)
    }
    fn parsed_fields(
        &self,
        declaration: DeclarationId,
    ) -> Option<&Arc<pse_modeling::expression::occurrences::ParsedFields>> {
        self.documents.parsed_fields.get(&declaration)
    }
    fn preflight(
        &self,
        _at: DeclarationId,
        bytes: usize,
    ) -> std::result::Result<(), pse_modeling::ModelingError> {
        if self.db.cancel().load(Ordering::Relaxed) {
            return Err(pse_modeling::ModelingError::Cancelled);
        }
        let next = self.admission_bytes.get().saturating_add(bytes);
        if next > self.admission_limit {
            return Err(pse_modeling::ModelingError::Budget(
                "modeling admission bytes exceed the workspace allowance before expansion".into(),
            ));
        }
        self.admission_bytes.set(next);
        Ok(())
    }
    fn rows(&self, document: SemanticId) -> Option<usize> {
        self.documents
            .documents
            .get(&document)
            .map(|document| document.rows.rows())
    }
    fn bytes(&self, document: SemanticId) -> usize {
        self.documents
            .documents
            .get(&document)
            .map_or(0, |document| document.retained_bytes())
    }
    fn resolve(&self, source: SemanticId, path: &str) -> Option<SemanticId> {
        self.documents.resolve(source, path)
    }
    fn admit(
        &self,
        plan: &Arc<DocumentPlan>,
        document: SemanticId,
    ) -> std::result::Result<Arc<DocumentTable>, pse_modeling::ModelingError> {
        let input =
            self.inputs
                .get(&document)
                .ok_or_else(|| pse_modeling::ModelingError::Contract {
                    reason: pse_modeling::ModelingContractReason::Unclassified,
                    declaration: plan.dataset.into(),
                    message: format!("data document {document} is not published"),
                })?;
        let projection = plan.admission_bytes(input.document(self.db));
        self.preflight(plan.dataset, projection)?;
        let next_memo = self
            .memo_bytes
            .get()
            .saturating_add(plan.retained_bytes())
            .saturating_add(projection);
        if self.pending.is_none() && next_memo > self.memo_limit {
            return Err(pse_modeling::ModelingError::Contract {
                reason: pse_modeling::ModelingContractReason::Unclassified,
                declaration: plan.dataset.into(), message: "modeling document plan and memo bytes exceed the workspace allowance before retention".into(),
            });
        }
        self.memo_bytes.set(next_memo);
        if let Some(quantities) = self.pending {
            let document = input.document(self.db);
            return pse_modeling::document::admit(plan, document, quantities).map(Arc::new);
        }
        let key = PlanKey::new(self.db, Arc::clone(plan));
        let db = self.db;
        let inventory = self.inventory;
        salsa::Cancelled::catch(std::panic::AssertUnwindSafe(|| {
            document_table(db, inventory, key, *input)
        }))
        .map_err(|_| pse_modeling::ModelingError::Cancelled)?
        .map_err(|error| match error {
            CompileError::Modeling(error) => error,
            other => pse_modeling::ModelingError::Contract {
                reason: pse_modeling::ModelingContractReason::Unclassified,
                declaration: plan.dataset.into(),
                message: other.to_string(),
            },
        })
    }
}

#[salsa::input]
pub(super) struct Catalog {
    checked: Arc<CheckedPackage>,
}
#[salsa::input]
struct Request {
    root: DeclarationId,
    instance: InstanceId,
    bindings: Bindings,
    limits: Limits,
    structure: StructuralRequest,
}
#[salsa::input]
struct StructuralRequest {
    root: DeclarationId,
    instance: InstanceId,
    bindings: Bindings,
    limits: Limits,
}
/// A process-local bucket hint only. Complete Salsa input fields remain the
/// equality authority, and the generation owns every handle in the buckets.
fn request_hash(
    root: DeclarationId,
    instance: InstanceId,
    bindings: &Bindings,
    limits: Limits,
) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hash = rustc_hash::FxHasher::default();
    root.hash(&mut hash);
    instance.hash(&mut hash);
    bindings.arguments.hash(&mut hash);
    bindings.scope.hash(&mut hash);
    bindings.facts.hash(&mut hash);
    bindings.demand.hash(&mut hash);
    bindings.formulation.omitted.hash(&mut hash);
    bindings.formulation.elastic.hash(&mut hash);
    limits.depth.hash(&mut hash);
    limits.items.hash(&mut hash);
    limits.members.hash(&mut hash);
    limits.body_occurrences.hash(&mut hash);
    limits.body_slots.hash(&mut hash);
    hash.finish()
}
pub(super) struct State {
    pub(super) catalog: Catalog,
    requests: rustc_hash::FxHashMap<u64, Vec<Request>>,
    structures: rustc_hash::FxHashMap<u64, Vec<StructuralRequest>>,
    request_bytes: usize,
    pub(super) revision: Arc<ModelingRevision>,
    pub(super) input_bytes: usize,
}
/// Immutable source admission, bound to its complete physical environment.
#[derive(Clone, Debug)]
pub struct ModelingRevision {
    rows: Arc<Vec<Declaration>>,
    scope: Arc<PhysicalScope>,
    documents: Arc<DocumentInventory>,
    checked: Arc<CheckedPackage>,
    inputs: CompilerContext,
    input_bytes: usize,
}
impl ModelingRevision {
    /// Exact immutable compiler context retained for historical preparation.
    #[doc(hidden)]
    pub fn compiler_context(&self) -> CompilerContext {
        self.inputs.clone()
    }
    /// Original declarations in source order.
    pub fn declarations(&self) -> &[Declaration] {
        &self.rows
    }
    /// The package data documents the declarations were admitted with (ADR-0125).
    pub fn documents(&self) -> &Arc<DocumentInventory> {
        &self.documents
    }
    /// The admitted package: checked declarations, entities and tables.
    pub fn checked(&self) -> &Arc<CheckedPackage> {
        &self.checked
    }
    /// Which documents see the physical names, for immutable source edits.
    pub fn physical_scope(&self) -> &PhysicalScope {
        &self.scope
    }
    /// The source entity a test names as the source of its expected values (ADR-0123
    /// Outcome 5).
    pub fn oracle(&self, test: DeclarationId) -> Option<DeclarationId> {
        self.checked.oracle(test)
    }
    /// The release an oracle's values come from (Plan 23 H6).
    pub fn release_of(&self, oracle: DeclarationId) -> Option<DeclarationId> {
        self.checked.release_of(oracle)
    }
    /// The physical name bindings the admitted source resolves with (ADR-0123 Outcome 8).
    pub fn physical_bindings(&self) -> BTreeMap<String, SemanticId> {
        self.checked.physical_bindings()
    }
    /// What specializing `root` as `instance` solves: its model, case and instance
    /// (`pse_model::lineage`), or `None` when the revision admits no such root.
    pub fn solved(
        &self,
        root: DeclarationId,
        instance: InstanceId,
    ) -> Option<pse_model::lineage::Solved> {
        let kind = self.checked.declaration(root)?.value.kind;
        Some(pse_model::lineage::Solved::new(root, kind, instance))
    }
    /// Conservative retained source and checked-state extent.
    pub fn retained_bytes(&self) -> usize {
        self.input_bytes
    }
}
fn revision(
    rows: Arc<Vec<Declaration>>,
    scope: Arc<PhysicalScope>,
    documents: Arc<DocumentInventory>,
    checked: Arc<CheckedPackage>,
    inputs: CompilerContext,
) -> Arc<ModelingRevision> {
    use pse_model::HeapUsage;
    // The data documents' decoded rows and the admitted tables are charged beside the
    // declarations (ADR-0125).
    let input_bytes = rows
        .owned_bytes()
        .saturating_add(checked.retained_bytes())
        .saturating_add(provider_context_extent(&inputs))
        .saturating_add(scope_bytes(&scope))
        .saturating_add(documents.retained_bytes());
    Arc::new(ModelingRevision {
        rows,
        scope,
        documents,
        checked,
        inputs,
        input_bytes,
    })
}
fn scope_bytes(scope: &PhysicalScope) -> usize {
    scope.package.as_ref().map_or(0, String::capacity)
        + scope
            .documents
            .as_ref()
            .map_or(0, |d| d.len() * (size_of::<SemanticId>() + 32))
}
#[salsa::tracked(returns(clone),lru=64,heap_size=selected_heap)]
fn selected(
    db: &dyn CompilerDb,
    catalog: Catalog,
    root: DeclarationId,
) -> Result<Arc<CheckedPackage>> {
    let _phase = pse_diagnostics::observation::phase("compiler.query.selected");
    checkpoint(db);
    read_static_package(db, &root_package(db, catalog, root)?).map_err(Into::into)
}
// Catalog equality is filtered through selected() before these retained firewalls.
// Unrelated package edits may validate them, but cannot invalidate projection.
#[salsa::tracked(returns(clone),lru=256,heap_size=physical_scope_heap)]
fn physical_scope(
    db: &dyn CompilerDb,
    catalog: Catalog,
    root: DeclarationId,
) -> Result<Arc<PhysicalScope>> {
    let package = selected(db, catalog, root)?;
    Ok(Arc::new(package.context().scope.clone()))
}
fn physical_scope_heap(value: &Result<Arc<PhysicalScope>>) -> usize {
    value.as_ref().map_or(0, |scope| scope_bytes(scope))
}
#[salsa::tracked(returns(clone),lru=256,heap_size=selected_heap)]
fn function_contracts(
    db: &dyn CompilerDb,
    catalog: Catalog,
    root: DeclarationId,
) -> Result<Arc<CheckedPackage>> {
    selected(db, catalog, root)
}
#[derive(Clone, Debug, PartialEq)]
struct PreludeInput {
    premises: Arc<pse_modeling::specialize::prelude::PreludePremises>,
    bindings: pse_modeling::specialize::prelude::PreludeBindings,
}
impl Eq for PreludeInput {}
impl std::hash::Hash for PreludeInput {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // A bucket hint only: complete entity-local premises remain Eq authority.
        std::hash::Hash::hash(&self.bindings, state);
    }
}
#[salsa::interned(heap_size=prelude_key_heap)]
struct PreludeKey<'db> {
    input: Arc<PreludeInput>,
}
fn prelude_key_heap((input,): &(Arc<PreludeInput>,)) -> usize {
    input
        .premises
        .retained_bytes()
        .saturating_add(input.bindings.retained_bytes())
}
#[salsa::tracked(returns(clone),lru=256,heap_size=prelude_heap)]
fn instance_prelude(
    db: &dyn CompilerDb,
    key: PreludeKey<'_>,
) -> Result<Arc<pse_modeling::specialize::prelude::InstancePrelude>> {
    checkpoint(db);
    let input = key.input(db);
    let result = pse_modeling::specialize::prelude::derive(&input.premises, &input.bindings)?;
    checkpoint(db);
    Ok(Arc::new(result))
}
fn prelude_heap(value: &Result<Arc<pse_modeling::specialize::prelude::InstancePrelude>>) -> usize {
    value.as_ref().map_or(0, |prelude| prelude.retained_bytes())
}
#[salsa::interned(heap_size=function_fragment_key_heap)]
struct FunctionFragmentKey<'db> {
    package: StaticFunctionPackageKey<'db>,
    recipe: Arc<pse_modeling::specialize::function_fragments::FunctionFragmentRecipe>,
}
fn function_fragment_key_heap(
    (_package, recipe): &(
        StaticFunctionPackageKey<'_>,
        Arc<pse_modeling::specialize::function_fragments::FunctionFragmentRecipe>,
    ),
) -> usize {
    recipe.retained_bytes()
}
#[salsa::tracked(returns(clone),lru=256,heap_size=function_fragment_heap)]
fn function_fragment<'db>(
    db: &'db dyn CompilerDb,
    key: FunctionFragmentKey<'db>,
) -> Result<Arc<pse_modeling::specialize::function_fragments::FunctionFragment>> {
    checkpoint(db);
    let package = read_static_package(db, key.package(db))?;
    let input = pse_modeling::specialize::function_fragments::FunctionFragmentInput::from_parts(
        package,
        key.recipe(db).clone(),
    );
    let result = pse_modeling::specialize::function_fragments::derive(
        &input,
        Some(&QueryFragments {
            db,
            parent: *key.package(db),
        }),
        &|| db.cancel().load(Ordering::Acquire),
        &LibraryDiscretizer,
    )?;
    checkpoint(db);
    Ok(Arc::new(result))
}
fn function_fragment_heap(
    value: &Result<Arc<pse_modeling::specialize::function_fragments::FunctionFragment>>,
) -> usize {
    value
        .as_ref()
        .map_or(0, |fragment| fragment.retained_bytes())
}
#[salsa::interned(heap_size=field_fragment_key_heap)]
struct FieldFragmentKey<'db> {
    package: StaticFunctionPackageKey<'db>,
    recipe: Arc<pse_modeling::specialize::field_fragments::FieldFragmentRecipe>,
}
fn field_fragment_key_heap(
    (_package, recipe): &(
        StaticFunctionPackageKey<'_>,
        Arc<pse_modeling::specialize::field_fragments::FieldFragmentRecipe>,
    ),
) -> usize {
    recipe.retained_bytes()
}
#[salsa::tracked(returns(clone),lru=256,heap_size=field_fragment_heap)]
fn field_fragment<'db>(
    db: &'db dyn CompilerDb,
    key: FieldFragmentKey<'db>,
) -> Result<Arc<pse_modeling::specialize::field_fragments::FieldFragment>> {
    checkpoint(db);
    let _phase = pse_diagnostics::observation::phase("compiler.query.field_fragment");
    let input = pse_modeling::specialize::field_fragments::FieldFragmentInput::from_parts(
        read_static_package(db, key.package(db))?,
        key.recipe(db).clone(),
    );
    let result = pse_modeling::specialize::field_fragments::derive(
        &input,
        Some(&QueryFragments {
            db,
            parent: *key.package(db),
        }),
        &|| db.cancel().load(Ordering::Acquire),
        &LibraryDiscretizer,
    )?;
    checkpoint(db);
    Ok(Arc::new(result))
}
fn field_fragment_heap(
    value: &Result<Arc<pse_modeling::specialize::field_fragments::FieldFragment>>,
) -> usize {
    value
        .as_ref()
        .map_or(0, |fragment| fragment.retained_bytes())
}
fn fragment_error(error: CompileError) -> pse_modeling::ModelingError {
    match error {
        CompileError::Modeling(error) => error,
        CompileError::Cancelled => pse_modeling::ModelingError::Cancelled,
        error => pse_modeling::ModelingError::Contract {
            reason: pse_modeling::ModelingContractReason::Unclassified,
            declaration: SemanticId::NIL,
            message: error.to_string(),
        },
    }
}
struct QueryFragments<'db> {
    db: &'db dyn CompilerDb,
    parent: StaticFunctionPackageKey<'db>,
}
impl pse_modeling::specialize::prelude::Fragments for QueryFragments<'_> {
    fn field_fragment(
        &self,
        input: Arc<pse_modeling::specialize::field_fragments::FieldFragmentInput>,
    ) -> std::result::Result<
        Arc<pse_modeling::specialize::field_fragments::FieldFragment>,
        pse_modeling::ModelingError,
    > {
        let (_, recipe) = input.parts();
        let package = select_package(self.db, self.parent, input.selection().clone())
            .map_err(fragment_error)?;
        let key = FieldFragmentKey::new(self.db, package, recipe);
        field_fragment(self.db, key).map_err(|error| match error {
            CompileError::Modeling(error) => error,
            CompileError::Cancelled => pse_modeling::ModelingError::Cancelled,
            error => pse_modeling::ModelingError::Contract {
                reason: pse_modeling::ModelingContractReason::Unclassified,
                declaration: SemanticId::NIL,
                message: error.to_string(),
            },
        })
    }

    fn function_fragment(
        &self,
        input: Arc<pse_modeling::specialize::function_fragments::FunctionFragmentInput>,
    ) -> std::result::Result<
        Arc<pse_modeling::specialize::function_fragments::FunctionFragment>,
        pse_modeling::ModelingError,
    > {
        let (_, recipe) = input.parts();
        let package = select_package(self.db, self.parent, input.selection().clone())
            .map_err(fragment_error)?;
        let key = FunctionFragmentKey::new(self.db, package, recipe);
        function_fragment(self.db, key).map_err(|error| match error {
            CompileError::Modeling(error) => error,
            CompileError::Cancelled => pse_modeling::ModelingError::Cancelled,
            error => pse_modeling::ModelingError::Contract {
                reason: pse_modeling::ModelingContractReason::Unclassified,
                declaration: SemanticId::NIL,
                message: error.to_string(),
            },
        })
    }

    fn instance_prelude(
        &self,
        premises: Arc<pse_modeling::specialize::prelude::PreludePremises>,
        bindings: pse_modeling::specialize::prelude::PreludeBindings,
    ) -> std::result::Result<
        Arc<pse_modeling::specialize::prelude::InstancePrelude>,
        pse_modeling::ModelingError,
    > {
        let key = PreludeKey::new(self.db, Arc::new(PreludeInput { premises, bindings }));
        instance_prelude(self.db, key).map_err(|error| match error {
            CompileError::Modeling(error) => error,
            CompileError::Cancelled => pse_modeling::ModelingError::Cancelled,
            error => pse_modeling::ModelingError::Contract {
                reason: pse_modeling::ModelingContractReason::Unclassified,
                declaration: SemanticId::NIL,
                message: error.to_string(),
            },
        })
    }
}
#[salsa::tracked(returns(clone),lru=64,heap_size=specialized_heap)]
fn specialized(
    db: &dyn CompilerDb,
    catalog: Catalog,
    request: StructuralRequest,
) -> Result<Arc<SpecializedModel>> {
    let _phase = pse_diagnostics::observation::phase("compiler.query.specialized");
    checkpoint(db);
    let parent = root_package(db, catalog, *request.root(db))?;
    let package = read_static_package(db, &parent)?;
    let result = pse_modeling::specialize::specialize_with_fragments(
        &package,
        *request.root(db),
        *request.instance(db),
        request.bindings(db),
        *request.limits(db),
        &|| db.cancel().load(Ordering::Acquire),
        &LibraryDiscretizer,
        Some(&QueryFragments { db, parent }),
    );
    checkpoint(db);
    result.map(Arc::new).map_err(CompileError::from)
}
#[salsa::tracked(returns(clone),lru=256,heap_size=specialized_heap)]
fn observed_specialization(
    db: &dyn CompilerDb,
    catalog: Catalog,
    request: Request,
) -> Result<Arc<SpecializedModel>> {
    let model = specialized(db, catalog, *request.structure(db))?;
    let paths = model.observation_paths(*request.instance(db), &request.bindings(db).demand);
    if paths
        .iter()
        .all(|(path, id)| model.paths.get(path) == Some(id))
    {
        return Ok(model);
    }
    let mut observed = model.as_ref().clone();
    observed.paths.extend(paths);
    Ok(Arc::new(observed))
}
fn selected_heap(_v: &Result<Arc<CheckedPackage>>) -> usize {
    // The tracked root-package product owns the payload and shared physical inputs.
    0
}
fn specialized_heap(v: &Result<Arc<SpecializedModel>>) -> usize {
    v.as_ref().map_or(0, |v| v.retained_bytes())
}

/// Stateless bridge from declaration contracts to the initialized mathematics library.
struct LibraryDiscretizer;
impl pse_modeling::continuous::Discretizer for LibraryDiscretizer {
    fn element(
        &self,
        scheme: pse_modeling::continuous::Scheme<'_>,
        order: usize,
        at: DeclarationId,
    ) -> std::result::Result<pse_modeling::continuous::ElementStencil, pse_modeling::ModelingError>
    {
        use pse_modeling::continuous::{ElementStencil, FiniteDifference, Scheme};
        scheme.validate(at)?;
        let Scheme::Collocation(parameters) = scheme else {
            return FiniteDifference.element(scheme, order, at);
        };
        let stencil = pse_math::collocation::element(
            order,
            parameters.alpha,
            parameters.beta,
            parameters.right_endpoint,
        )
        .map_err(|e| pse_modeling::ModelingError::Contract {
            reason: pse_modeling::ModelingContractReason::Unclassified,
            declaration: at.into(),
            message: e.to_string(),
        })?;
        Ok(ElementStencil {
            nodes: stencil.nodes,
            derivative: stencil.derivative,
            integral: stencil.integral,
            endpoint: stencil.endpoint,
            lattice: None,
        })
    }
}
impl CompilerWorkspace {
    /// Atomically admit a complete generic package inventory without data documents into
    /// the existing workspace.
    /// # Errors
    /// Invalid source contracts leave the previous package selection unchanged.
    pub fn publish_modeling(
        &mut self,
        rows: impl Into<Arc<Vec<Declaration>>>,
        scope: impl Into<Arc<PhysicalScope>>,
    ) -> Result<Arc<ModelingRevision>> {
        self.publish_modeling_with(rows, scope, Arc::new(DocumentInventory::default()))
    }
    /// Atomically admit a complete generic package inventory and its package data
    /// documents (ADR-0125). A document's admitted rows are reused while its plan and bytes
    /// are unchanged; their bytes are charged to the workspace limits.
    /// # Errors
    /// Invalid source contracts leave the previous package selection unchanged.
    pub fn publish_modeling_with(
        &mut self,
        rows: impl Into<Arc<Vec<Declaration>>>,
        scope: impl Into<Arc<PhysicalScope>>,
        documents: Arc<DocumentInventory>,
    ) -> Result<Arc<ModelingRevision>> {
        self.publish_modeling_with_cancellable(
            rows,
            scope,
            documents,
            Arc::new(AtomicBool::new(false)),
        )
    }
    /// Stage and publish one candidate under the caller's cooperative cancellation scope.
    /// # Errors
    /// Cancellation and all normal admission refusals preserve the preceding head.
    pub fn publish_modeling_with_cancellable(
        &mut self,
        rows: impl Into<Arc<Vec<Declaration>>>,
        scope: impl Into<Arc<PhysicalScope>>,
        documents: Arc<DocumentInventory>,
        cancel: Arc<AtomicBool>,
    ) -> Result<Arc<ModelingRevision>> {
        self.publish_modeling_with_validation(rows, scope, documents, cancel, |_| Ok(()))
    }
    /// Validate consumer-owned source obligations before the admitted head commits.
    /// The validation receives an immutable complete candidate and cannot query this lane.
    /// # Errors
    /// Semantic, consumer-validation and cancellation failures preserve the previous head.
    pub fn publish_modeling_with_validation(
        &mut self,
        rows: impl Into<Arc<Vec<Declaration>>>,
        scope: impl Into<Arc<PhysicalScope>>,
        documents: Arc<DocumentInventory>,
        cancel: Arc<AtomicBool>,
        validate_candidate: impl FnOnce(&CheckedPackage) -> Result<()>,
    ) -> Result<Arc<ModelingRevision>> {
        let _phase = pse_diagnostics::observation::current().map(|context| {
            context.count("compiler.publication_demands", "count", 1);
            context.phase("compiler.publish_modeling")
        });
        if cancel.load(Ordering::Acquire) {
            return Err(CompileError::Cancelled);
        }
        self.db.cancel = cancel;

        use pse_model::HeapUsage;
        let rows = rows.into();
        let scope = scope.into();
        if let Some(state) = &self.modeling
            && state.revision.rows == rows
            && state.revision.scope == scope
            && state.revision.documents == documents
            && state.revision.inputs == self.inputs
            && Arc::ptr_eq(&state.revision.inputs.quantities, &self.inputs.quantities)
            && Arc::ptr_eq(
                &state.revision.inputs.preconditions,
                &self.inputs.preconditions,
            )
        {
            validate_candidate(&state.revision.checked)?;
            if let Some(context) = pse_diagnostics::observation::current() {
                context.count("compiler.unchanged_revision_reuses", "count", 1);
            }
            return Ok(state.revision.clone());
        }
        if rows
            .owned_bytes()
            .saturating_add(scope_bytes(&scope))
            .saturating_add(documents.retained_bytes())
            > self.limits.input_bytes
        {
            return Err(CompileError::Limit("modeling input bytes"));
        }
        if rows.len() > self.limits.query_values.saturating_mul(1024) {
            return Err(CompileError::Limit("modeling declarations"));
        }
        let prior = self.modeling.as_ref().map(|state| state.revision.clone());
        let candidate = (|| {
            let checked = Arc::new(self.check_modeling(&rows, &scope, &documents)?);
            validate_candidate(&checked)?;
            let revision = revision(rows, scope, documents, checked, self.inputs.clone());
            if revision.input_bytes > self.limits.input_bytes {
                return Err(CompileError::Limit("modeling input bytes"));
            }
            if self.db.cancel.load(Ordering::Acquire) {
                return Err(CompileError::Cancelled);
            }
            self.publish_modeling_revision(revision.clone())?;
            self.db.trigger_lru_eviction();
            if self.retention_exceeded() {
                if let Some(context) = pse_diagnostics::observation::current() {
                    context.count("compiler.retention_limit_rotations", "count", 1);
                }
                self.rebuild(self.inputs.clone())?;
            }
            if self.db.cancel.load(Ordering::Acquire) {
                return Err(CompileError::Cancelled);
            }
            Ok(revision)
        })();
        if candidate.is_err() {
            // Setters are not a transaction. A failed or cancelled batch cannot
            // expose candidate-only inputs: rotate, reset cancellation and republish
            // the exact detached admitted head without running the checker again.
            let generation = self.generation.saturating_add(1);
            let _phase = pse_diagnostics::observation::current().map(|context| {
                context.count("compiler.failed_candidate_rotations", "count", 1);
                context.phase("compiler.failed_candidate_restore")
            });
            let mut restored = Self::with_shared_events(
                prior
                    .as_ref()
                    .map_or_else(|| self.inputs.clone(), |revision| revision.inputs.clone()),
                self.limits,
                self.events.clone(),
            )?;
            restored.db.body_retention = self.db.body_retention.clone();
            if let Some(prior) = prior {
                restored.publish_modeling_revision(prior)?;
            }
            restored.generation = generation;
            restored.retention_snapshot = None;
            *self = restored;
        }
        candidate
    }
    /// Whether this generation currently selects the exact detached admitted source.
    pub fn selects_modeling_revision(&self, revision: &Arc<ModelingRevision>) -> bool {
        self.modeling.as_ref().is_some_and(|state| {
            Arc::ptr_eq(&state.revision, revision)
                || state.revision.rows == revision.rows
                    && state.revision.scope == revision.scope
                    && state.revision.documents == revision.documents
                    && state.revision.inputs == revision.inputs
                    && Arc::ptr_eq(
                        &state.revision.inputs.quantities,
                        &revision.inputs.quantities,
                    )
                    && Arc::ptr_eq(
                        &state.revision.inputs.preconditions,
                        &revision.inputs.preconditions,
                    )
        })
    }
    /// Whether a generation has an admitted live head, independently of retained query entries.
    pub fn has_modeling_revision(&self) -> bool {
        self.modeling.is_some()
    }
    /// Complete immutable physical and provider context for a separately charged historical lane.
    pub fn context(&self) -> CompilerContext {
        self.inputs.clone()
    }
    /// Atomically check incoming source under its receiving physical context. The
    /// preceding source need not be valid in the incoming interpretation.
    #[doc(hidden)]
    pub fn publish_modeling_context_with_validation(
        &mut self,
        inputs: CompilerContext,
        rows: impl Into<Arc<Vec<Declaration>>>,
        scope: impl Into<Arc<PhysicalScope>>,
        documents: Arc<DocumentInventory>,
        cancel: Arc<AtomicBool>,
        validate_candidate: impl FnOnce(&CheckedPackage) -> Result<()>,
    ) -> Result<Arc<ModelingRevision>> {
        self.with_modeling_context(inputs, cancel.clone(), |workspace| {
            workspace.publish_modeling_with_validation(
                rows,
                scope,
                documents,
                cancel,
                validate_candidate,
            )
        })
    }
    /// Refresh physical inputs under the existing exclusive compiler writer. Failed
    /// admission destroys all candidate setters and restores the preceding context/head.
    #[doc(hidden)]
    pub fn refresh_modeling_context(
        &mut self,
        inputs: CompilerContext,
        cancel: Arc<AtomicBool>,
    ) -> Result<()> {
        let prior = self.modeling.as_ref().map(|state| state.revision.clone());
        self.with_modeling_context(inputs, cancel.clone(), |workspace| match prior {
            Some(prior) => workspace
                .publish_modeling_with_cancellable(
                    prior.rows.clone(),
                    prior.scope.clone(),
                    prior.documents.clone(),
                    cancel,
                )
                .map(|_| ()),
            None => Ok(()),
        })
    }
    fn with_modeling_context<T>(
        &mut self,
        inputs: CompilerContext,
        cancel: Arc<AtomicBool>,
        admit: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        if cancel.load(Ordering::Acquire) {
            return Err(CompileError::Cancelled);
        }
        if self.inputs == inputs
            && Arc::ptr_eq(&self.inputs.quantities, &inputs.quantities)
            && Arc::ptr_eq(&self.inputs.preconditions, &inputs.preconditions)
        {
            return admit(self);
        }
        validate(&inputs, self.limits)?;
        let prior_inputs = self.inputs.clone();
        let prior = self.modeling.as_ref().map(|state| state.revision.clone());
        let generation = self.generation.saturating_add(1);
        self.inventory
            .set_quantities(&mut self.db)
            .to(inputs.quantities.clone());
        self.inventory
            .set_preconditions(&mut self.db)
            .to(inputs.preconditions.clone());
        self.inventory
            .set_providers(&mut self.db)
            .to(inputs.providers.clone());
        self.inputs = inputs;
        let result = admit(self);
        if result.is_err() {
            let mut restored =
                Self::with_shared_events(prior_inputs, self.limits, self.events.clone())?;
            restored.db.body_retention = self.db.body_retention.clone();
            if let Some(prior) = prior {
                restored.publish_modeling_revision(prior)?;
            }
            restored.generation = generation;
            restored.retention_snapshot = None;
            *self = restored;
        }
        result
    }
    /// Stage document inputs while the candidate exclusively owns this generation.
    fn check_modeling(
        &mut self,
        rows: &[Declaration],
        scope: &Arc<PhysicalScope>,
        documents: &Arc<DocumentInventory>,
    ) -> Result<CheckedPackage> {
        for (id, document) in &documents.documents {
            match self.documents.get(id) {
                Some(input) if input.document(&self.db) == document => {}
                Some(input) => {
                    input.set_document(&mut self.db).to(document.clone());
                }
                None => {
                    self.documents
                        .insert(*id, DataDocumentInput::new(&self.db, document.clone()));
                }
            }
        }
        granular::configure(&mut self.db, self.limits.granular_query_values);
        let checked = salsa::Cancelled::catch(std::panic::AssertUnwindSafe(|| {
            granular::check(self, rows, scope, documents)
        }))
        .map_err(|_| CompileError::Cancelled)??;
        checked
            .retain_physical_owners(
                self.inputs.quantities.clone(),
                self.inputs.preconditions.clone(),
            )
            .map_err(CompileError::from)
    }
    /// Select an already admitted revision without rechecking its declarations.
    /// # Errors
    /// A different physical context or insufficient workspace budget refuses publication.
    pub fn publish_modeling_revision(&mut self, revision: Arc<ModelingRevision>) -> Result<()> {
        if revision.inputs != self.inputs
            || !Arc::ptr_eq(&revision.inputs.quantities, &self.inputs.quantities)
            || !Arc::ptr_eq(&revision.inputs.preconditions, &self.inputs.preconditions)
            || revision.checked.context().quantities != self.inputs.quantities.as_ref()
            || revision.checked.context().preconditions != self.inputs.preconditions.as_ref()
        {
            return Err(CompileError::Missing(
                "modeling revision physical context differs".into(),
            ));
        }
        if revision.rows.len() > self.limits.query_values.saturating_mul(1024) {
            return Err(CompileError::Limit("modeling declarations"));
        }
        let input_bytes = revision.input_bytes;
        let checked = revision.checked.clone();
        validate(
            &self.inputs,
            WorkspaceLimits {
                input_bytes: self.limits.input_bytes.saturating_sub(input_bytes),
                ..self.limits
            },
        )?;
        if let Some(state) = &mut self.modeling {
            if state.catalog.checked(&self.db).as_ref() != checked.as_ref() {
                state.catalog.set_checked(&mut self.db).to(checked);
            }
            state.revision = revision;
            state.input_bytes = input_bytes;
        } else {
            let catalog = Catalog::new(&self.db, checked);
            selected::set_lru_capacity(&mut self.db, self.limits.query_values);
            shared_packages::configure(&mut self.db, self.limits.query_values);
            specialized::set_lru_capacity(&mut self.db, self.limits.query_values);
            observed_specialization::set_lru_capacity(&mut self.db, self.limits.query_values);
            instance_prelude::set_lru_capacity(&mut self.db, self.limits.query_values);
            function_fragment::set_lru_capacity(&mut self.db, self.limits.query_values);
            field_fragment::set_lru_capacity(&mut self.db, self.limits.query_values);
            physical_scope::set_lru_capacity(&mut self.db, self.limits.query_values);
            function_contracts::set_lru_capacity(&mut self.db, self.limits.query_values);
            document_table::set_lru_capacity(&mut self.db, self.limits.query_values);
            executable::configure(&mut self.db, self.limits.query_values);
            self.modeling = Some(State {
                catalog,
                requests: rustc_hash::FxHashMap::default(),
                structures: rustc_hash::FxHashMap::default(),
                request_bytes: 0,
                revision,
                input_bytes,
            });
        }
        Ok(())
    }
    /// Specialize a finite root through tracked queries without constructing a solver or store.
    /// # Errors
    /// Missing package inputs, invalid bindings, limits or cancelled compilation.
    pub fn specialize_modeling(
        &mut self,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
    ) -> Result<Arc<SpecializedModel>> {
        self.db.cancel = Arc::new(AtomicBool::new(false));
        let (catalog, request) = self.modeling_request(root, instance, bindings, limits)?;
        let result =
            salsa::Cancelled::catch(|| observed_specialization(&self.db, catalog, request))
                .map_err(|_| CompileError::Cancelled)?;
        self.trim_queries()?;
        result
    }
    /// Prepare physically checked process meaning without executable arithmetic.
    /// # Errors
    /// Invalid specialization, physical coordinates, cancellation or finite limits.
    pub fn prepare_semantic_modeling_cancellable(
        &mut self,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        cancel: Arc<AtomicBool>,
    ) -> Result<SemanticModeling> {
        if cancel.load(Ordering::Acquire) {
            return Err(CompileError::Cancelled);
        }
        self.db.cancel = cancel;
        let (catalog, request) = self.modeling_request(root, instance, bindings, limits)?;
        let result = salsa::Cancelled::catch(|| {
            let model = observed_specialization(&self.db, catalog, request)?;
            SemanticModeling::admit(
                model.into(),
                self.inventory.quantities(&self.db),
                self.inventory.preconditions(&self.db),
            )
        })
        .map_err(|_| CompileError::Cancelled)?;
        self.trim_queries()?;
        result
    }
    fn modeling_request(
        &mut self,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
    ) -> Result<(Catalog, Request)> {
        let hash = request_hash(root, instance, &bindings, limits);
        if bindings.demand.is_empty() {
            return self.modeling_request_in_bucket(root, instance, bindings, limits, hash);
        }
        let mut base_bindings = bindings.clone();
        base_bindings.demand.clear();
        let base_hash = request_hash(root, instance, &base_bindings, limits);
        let (catalog, base) =
            self.modeling_request_in_bucket(root, instance, base_bindings, limits, base_hash)?;
        let base =
            salsa::Cancelled::catch(|| specialized(&self.db, catalog, *base.structure(&self.db)))
                .map_err(|_| CompileError::Cancelled)?;
        // A demand may make an otherwise absent construction reachable. If the
        // baseline refuses, retain every demand as structural and let the ordinary
        // producer establish the exact outcome, rather than guessing from syntax.
        let mut structural = bindings.clone();
        if let Ok(base) = base {
            let observations = base.observation_paths(instance, &bindings.demand);
            structural
                .demand
                .retain(|path| !observations.contains_key(path));
        }
        self.modeling_request_with_structure(root, instance, bindings, limits, hash, structural)
    }
    fn modeling_request_in_bucket(
        &mut self,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        hash: u64,
    ) -> Result<(Catalog, Request)> {
        self.modeling_request_with_structure(
            root,
            instance,
            bindings.clone(),
            limits,
            hash,
            bindings,
        )
    }
    fn modeling_request_with_structure(
        &mut self,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        hash: u64,
        structural: Bindings,
    ) -> Result<(Catalog, Request)> {
        if self.retention_exceeded() {
            if let Some(context) = pse_diagnostics::observation::current() {
                context.count("compiler.retention_limit_rotations", "count", 1);
            }
            self.rebuild(self.inputs.clone())?;
        }

        let state = self
            .modeling
            .as_mut()
            .ok_or_else(|| CompileError::Missing("modeling package".into()))?;
        let binding_bytes = bindings
            .arguments
            .iter()
            .chain(&bindings.scope)
            .map(|(n, v)| n.capacity() + v.retained_bytes() + 128)
            .chain(
                bindings
                    .facts
                    .iter()
                    .map(|(f, v)| f.member().len() + v.retained_bytes() + 128),
            )
            .sum::<usize>()
            .saturating_add(
                bindings
                    .demand
                    .iter()
                    .map(String::capacity)
                    .sum::<usize>()
                    .saturating_add(bindings.demand.capacity() * size_of::<String>()),
            );
        let binding_bytes = binding_bytes
            .saturating_add(bindings.formulation.omitted.len() * 64)
            .saturating_add(
                bindings
                    .formulation
                    .elastic
                    .values()
                    .map(|v| v.retained_bytes() + 128)
                    .sum::<usize>(),
            );
        // CompilerContext are immutable by complete root/instance/binding/limit meaning. A/B/A
        // can fetch the original memo rather than overwriting a singleton root slot.
        if let Some(request) = state.requests.get(&hash).into_iter().flatten().find(|r| {
            *r.root(&self.db) == root
                && *r.instance(&self.db) == instance
                && r.bindings(&self.db) == &bindings
                && *r.limits(&self.db) == limits
        }) {
            if let Some(context) = pse_diagnostics::observation::current() {
                context.count("compiler.selection_request_reuses", "count", 1);
            }
            return Ok((state.catalog, *request));
        }
        // Charge the map's minimum table/growth slack and control bytes, plus
        // collision-vector spare capacity. New vectors start at one handle;
        // subsequent growth retains at most twice their occupied handles. The
        // conservative table allowance applies even when a hash bucket exists.
        let index_bytes =
            4 * (size_of::<(u64, Vec<Request>)>() + 1) + 2 * size_of::<Request>() + 128;
        let bytes = binding_bytes
            .saturating_mul(2)
            .saturating_add(
                2 * (size_of::<Bindings>() + size_of::<Limits>() + size_of::<Request>() + 128),
            )
            .saturating_add(index_bytes.saturating_mul(2));
        if bytes > self.limits.input_bytes.saturating_sub(state.input_bytes) {
            return Err(CompileError::Limit("modeling selection bindings"));
        }
        if state.request_bytes.saturating_add(bytes)
            > self.limits.input_bytes.saturating_sub(state.input_bytes)
        {
            // Rotation releases only local handles; owned products remain valid.
            if let Some(context) = pse_diagnostics::observation::current() {
                context.count("compiler.request_byte_limit_rotations", "count", 1);
            }
            self.rebuild(self.inputs.clone())?;
            return self.modeling_request_with_structure(
                root, instance, bindings, limits, hash, structural,
            );
        }
        let structural_hash = request_hash(root, instance, &structural, limits);
        let structure = state
            .structures
            .get(&structural_hash)
            .into_iter()
            .flatten()
            .find(|r| {
                *r.root(&self.db) == root
                    && *r.instance(&self.db) == instance
                    && r.bindings(&self.db) == &structural
                    && *r.limits(&self.db) == limits
            })
            .copied()
            .unwrap_or_else(|| {
                let request = StructuralRequest::new(&self.db, root, instance, structural, limits);
                state
                    .structures
                    .entry(structural_hash)
                    .or_default()
                    .push(request);
                request
            });
        let request = Request::new(&self.db, root, instance, bindings, limits, structure);
        state
            .requests
            .entry(hash)
            .or_insert_with(|| Vec::with_capacity(1))
            .push(request);
        state.request_bytes = state.request_bytes.saturating_add(bytes);
        Ok((state.catalog, request))
    }
}

mod conditional;
pub use conditional::{AutomaticCausalUnit, ConditionalUnitInventory};
pub(super) mod executable;
mod flow;
pub use executable::{
    AdmittedImplicit, AdmittedModeling, AnnotationObservation, BoundLookup, BoundLookupKey,
    BoundStructure, CompilerView, Derivation, Derived, ImplicitAlgorithm, ImplicitCapabilities,
    ImplicitMeaning, ImplicitProgramRequest, ImplicitPrograms, ImplicitScale, ImplicitSelection,
    ModelingCaseBindings, ModelingExpectationResult, ModelingHint, ModelingOccurrenceIdentity,
    ModelingOutput, ModelingPointChecks, ModelingPreparationFrontier, ModelingTestValue,
    ModelingValidityResult, ModelingVariableState, NumericalAdmission, NumericalAnnotation, NumericalAnnotationMode,
    NumericalAnnotations, NumericalLocation, NumericalTargetFact, NumericalTargetRole, NumericalScale, NumericalStartSource, NumericalStartValues,
    ObjectiveBound, PreparedModeling, ScalingObservations, SelectionEquivalence,
    SelectionNeighborhood, ViewChoice, ViewPolicy, ViewRoles, ViewReason,
};
pub use flow::{
    FlowConnectionDocument, FlowSelectionDocument, ModelingFlowSelection, SemanticModeling,
};

#[cfg(test)]
mod document_tests;
#[cfg(test)]
mod domain_tests;
#[cfg(test)]
mod forms_tests;
#[cfg(test)]
mod objective_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod reuse_tests;

#[cfg(test)]
mod granular_tests;

#[cfg(test)]
mod physical_premises_tests;

#[cfg(test)]
mod test_observation;
