// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Generic kernel queries in the existing compiler database.
use super::*;
use pse_authoring::language::Declaration;
use pse_modeling::{
    Bindings, CheckedPackage, DeclarationId, InstanceId, Limits, SpecializedModel, TypeContext,
};
use pse_quantity::QuantityTypeId;
use salsa::Setter;

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
    names: Arc<BTreeMap<String, QuantityTypeId>>,
    checked: Arc<CheckedPackage>,
    input_bytes: usize,
}
impl ModelingRevision {
    /// Original declarations in source order.
    pub fn declarations(&self) -> &[Declaration] {
        &self.rows
    }
    /// Admitted physical aliases for immutable source edits.
    pub fn quantity_names(&self) -> &BTreeMap<String, QuantityTypeId> {
        &self.names
    }
    /// What specializing `root` as `instance` solves: its model, case and instance
    /// (`pse_model::lineage`), or `None` when the revision admits no such root.
    pub fn solved(&self, root: DeclarationId, instance: InstanceId) -> Option<pse_model::lineage::Solved> {
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
    names: Arc<BTreeMap<String, QuantityTypeId>>,
    checked: Arc<CheckedPackage>,
) -> Arc<ModelingRevision> {
    use pse_model::HeapUsage;
    let input_bytes = rows
        .owned_bytes()
        .saturating_add(checked.retained_bytes())
        .saturating_add(
            names
                .iter()
                .map(|(name, _)| name.capacity() + 128)
                .sum::<usize>(),
        );
    Arc::new(ModelingRevision {
        rows,
        names,
        checked,
        input_bytes,
    })
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
    /// Atomically admit a complete generic package inventory into the existing workspace.
    /// # Errors
    /// Invalid source contracts leave the previous package selection unchanged.
    pub fn publish_modeling(
        &mut self,
        rows: impl Into<Arc<Vec<Declaration>>>,
        names: impl Into<Arc<BTreeMap<String, QuantityTypeId>>>,
    ) -> Result<Arc<ModelingRevision>> {
        use pse_model::HeapUsage;
        let rows = rows.into();
        let names = names.into();
        if let Some(state) = &self.modeling {
            if state.revision.rows == rows && state.revision.names == names {
                return Ok(state.revision.clone());
            }
        }
        if rows
            .owned_bytes()
            .saturating_add(names.keys().map(|n| n.capacity()).sum::<usize>())
            > self.limits.input_bytes
        {
            return Err(CompileError::Limit("modeling input bytes"));
        }
        if rows.len() > self.limits.query_values.saturating_mul(1024) {
            return Err(CompileError::Limit("modeling declarations"));
        }
        let context = TypeContext {
            quantities: &self.inputs.quantities,
            preconditions: &self.inputs.preconditions,
            names: &names,
        };
        let checked = Arc::new(pse_modeling::check(&rows, &context)?);
        let revision = revision(rows, names, checked);
        self.publish_modeling_revision(revision.clone())?;
        Ok(revision)
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
        &self,
        quantities: &QuantityRegistry,
        preconditions: &pse_quantity::PhysicalPreconditions,
    ) -> Result<Option<(Arc<ModelingRevision>, usize)>> {
        self.modeling
            .as_ref()
            .map(|state| {
                let old = &state.revision;
                let checked = Arc::new(pse_modeling::check(
                    &old.rows,
                    &TypeContext {
                        quantities,
                        preconditions,
                        names: &old.names,
                    },
                )?);
                let next = revision(old.rows.clone(), old.names.clone(), checked);
                let bytes = next.input_bytes;
                Ok((next, bytes))
            })
            .transpose()
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
            .chain(&bindings.facts)
            .map(|(n, v)| n.capacity() + v.retained_bytes() + 128)
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
    AdmittedImplicit, AdmittedModeling, Derivation, Derived, ImplicitAlgorithm, ImplicitScale,
    ModelingCaseBindings, ModelingExpectationResult, ModelingHint, ModelingOutput,
    ModelingTestValue, ModelingVariableState, PreparedModeling,
};
pub use flow::ModelingFlowSelection;

#[cfg(test)]
mod domain_tests;
#[cfg(test)]
mod forms_tests;
#[cfg(test)]
mod tests;
