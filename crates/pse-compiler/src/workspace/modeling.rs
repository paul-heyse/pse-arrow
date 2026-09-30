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

/// One package data document (ADR-0125), set only when its bytes change.
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
    db: &'a CompilerDatabase,
    inventory: Inventory,
    documents: &'a DocumentInventory,
    inputs: &'a BTreeMap<SemanticId, DataDocumentInput>,
    pending: Option<&'a QuantityRegistry>,
}
impl pse_modeling::document::Documents for TrackedDocuments<'_> {
    fn resolve(&self, source: SemanticId, path: &str) -> Option<SemanticId> {
        self.documents.resolve(source, path)
    }
    fn admit(
        &self,
        plan: &Arc<DocumentPlan>,
        document: SemanticId,
    ) -> std::result::Result<Arc<DocumentTable>, pse_modeling::ModelingError> {
        let input = self.inputs.get(&document).ok_or_else(|| {
            pse_modeling::ModelingError::Contract {
                declaration: plan.dataset.into(),
                message: format!("data document {document} is not published"),
            }
        })?;
        if let Some(quantities) = self.pending {
            let document = input.document(self.db);
            return pse_modeling::document::admit(plan, document, quantities).map(Arc::new);
        }
        let key = PlanKey::new(self.db, Arc::clone(plan));
        salsa::Cancelled::catch(|| document_table(self.db, self.inventory, key, *input))
            .map_err(|_| pse_modeling::ModelingError::Cancelled)?
            .map_err(|error| match error {
                CompileError::Modeling(error) => error,
                other => pse_modeling::ModelingError::Contract {
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
}
pub(super) struct State {
    pub(super) catalog: Catalog,
    requests: BTreeMap<(DeclarationId, InstanceId), Request>,
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
    input_bytes: usize,
}
impl ModelingRevision {
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
) -> Arc<ModelingRevision> {
    use pse_model::HeapUsage;
    // The data documents' decoded rows and the admitted tables are charged beside the
    // declarations (ADR-0125).
    let input_bytes = rows
        .owned_bytes()
        .saturating_add(checked.retained_bytes())
        .saturating_add(scope_bytes(&scope))
        .saturating_add(documents.retained_bytes());
    Arc::new(ModelingRevision {
        rows,
        scope,
        documents,
        checked,
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
    checkpoint(db);
    Ok(Arc::new(catalog.checked(db).select(root)?))
}
#[salsa::tracked(returns(clone),lru=64,heap_size=specialized_heap)]
fn specialized(
    db: &dyn CompilerDb,
    catalog: Catalog,
    request: Request,
) -> Result<Arc<SpecializedModel>> {
    checkpoint(db);
    let package = selected(db, catalog, *request.root(db))?;
    let result = pse_modeling::specialize::specialize_with_discretizer(
        &package,
        *request.root(db),
        *request.instance(db),
        request.bindings(db),
        *request.limits(db),
        &|| db.cancel().load(Ordering::Acquire),
        &LibraryDiscretizer,
    );
    checkpoint(db);
    result.map(Arc::new).map_err(CompileError::from)
}
fn selected_heap(v: &Result<Arc<CheckedPackage>>) -> usize {
    v.as_ref().map_or(0, |v| v.retained_bytes())
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
        use pse_model::HeapUsage;
        let rows = rows.into();
        let scope = scope.into();
        if let Some(state) = &self.modeling
            && state.revision.rows == rows
            && state.revision.scope == scope
            && state.revision.documents == documents
        {
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
        let checked = Arc::new(self.check_modeling(&rows, &scope, &documents, None)?);
        let revision = revision(rows, scope, documents, checked);
        if revision.input_bytes > self.limits.input_bytes {
            return Err(CompileError::Limit("modeling input bytes"));
        }
        self.publish_modeling_revision(revision.clone())?;
        Ok(revision)
    }
    /// Check `rows` with their data documents, each document an input set only when its
    /// bytes change, so equal plans reuse their admitted rows.
    fn check_modeling(
        &mut self,
        rows: &[Declaration],
        scope: &PhysicalScope,
        documents: &DocumentInventory,
        physical: Option<(&QuantityRegistry, &PhysicalPreconditions)>,
    ) -> Result<CheckedPackage> {
        for (id, document) in &documents.documents {
            match self.documents.get(id) {
                Some(input) if input.document(&self.db).content_hash == document.content_hash => {}
                Some(input) => {
                    input.set_document(&mut self.db).to(Arc::clone(document));
                }
                None => {
                    let input = DataDocumentInput::new(&self.db, Arc::clone(document));
                    self.documents.insert(*id, input);
                }
            }
        }
        let (quantities, preconditions) = physical.unwrap_or((
            self.inputs.quantities.as_ref(),
            self.inputs.preconditions.as_ref(),
        ));
        let context = TypeContext {
            quantities,
            preconditions,
            scope,
        };
        let tracked = TrackedDocuments {
            db: &self.db,
            inventory: self.inventory,
            documents,
            inputs: &self.documents,
            pending: physical.map(|(quantities, _)| quantities),
        };
        Ok(pse_modeling::check_with(rows, &context, &tracked)?)
    }
    /// Select an already admitted revision without rechecking its declarations.
    /// # Errors
    /// A different physical context or insufficient workspace budget refuses publication.
    pub fn publish_modeling_revision(&mut self, revision: Arc<ModelingRevision>) -> Result<()> {
        if revision.checked.context().quantities != self.inputs.quantities.as_ref()
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
            specialized::set_lru_capacity(&mut self.db, self.limits.query_values);
            document_table::set_lru_capacity(&mut self.db, self.limits.query_values);
            executable::configure(&mut self.db, self.limits.query_values);
            self.modeling = Some(State {
                catalog,
                requests: BTreeMap::new(),
                revision,
                input_bytes,
            });
        }
        Ok(())
    }
    pub(super) fn recheck_modeling(
        &mut self,
        quantities: &QuantityRegistry,
        preconditions: &PhysicalPreconditions,
    ) -> Result<Option<(Arc<ModelingRevision>, usize)>> {
        let Some(old) = self.modeling.as_ref().map(|state| state.revision.clone()) else {
            return Ok(None);
        };
        // A document's plan converts through the next physical registry; its rows are
        // admitted again only where that plan differs.
        let checked = Arc::new(self.check_modeling(
            &old.rows,
            &old.scope,
            &old.documents,
            Some((quantities, preconditions)),
        )?);
        let next = revision(
            old.rows.clone(),
            old.scope.clone(),
            old.documents.clone(),
            checked,
        );
        let bytes = next.input_bytes;
        Ok(Some((next, bytes)))
    }
    pub(super) fn set_checked_modeling(&mut self, checked: Option<(Arc<ModelingRevision>, usize)>) {
        if let (Some(state), Some((revision, bytes))) = (&mut self.modeling, checked) {
            if state.catalog.checked(&self.db) != &revision.checked {
                state
                    .catalog
                    .set_checked(&mut self.db)
                    .to(revision.checked.clone());
            }
            state.revision = revision;
            state.input_bytes = bytes;
        }
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
        let result = salsa::Cancelled::catch(|| specialized(&self.db, catalog, request))
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
        if self.retention_exceeded() {
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
                    .map(|s| s.capacity() + size_of::<String>())
                    .sum::<usize>(),
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
        if binding_bytes
            > self.limits.input_bytes.saturating_sub(state.input_bytes) / self.limits.query_values
        {
            return Err(CompileError::Limit("modeling selection bindings"));
        }
        let key = (root, instance);
        let request = if let Some(request) = state.requests.get(&key).copied() {
            if request.bindings(&self.db) != &bindings {
                request.set_bindings(&mut self.db).to(bindings);
            }
            if request.limits(&self.db) != &limits {
                request.set_limits(&mut self.db).to(limits);
            }
            request
        } else {
            if state.requests.len() >= self.limits.query_values {
                return Err(CompileError::Limit("modeling root selections"));
            }
            let request = Request::new(&self.db, root, instance, bindings, limits);
            state.requests.insert(key, request);
            request
        };
        Ok((state.catalog, request))
    }
}

mod executable;
mod flow;
pub use executable::{
    AdmittedImplicit, AdmittedModeling, BoundStructure, Derivation, Derived, ImplicitAlgorithm,
    ImplicitScale, ModelingCaseBindings, ModelingExpectationResult, ModelingHint, ModelingOutput,
    ModelingPointChecks, ModelingTestValue, ModelingValidityResult, ModelingVariableState,
    ObjectiveBound, PreparedModeling,
};
pub use flow::ModelingFlowSelection;

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
