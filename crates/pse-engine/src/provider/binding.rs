// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One immutable source inventory for native lookup, roles and semantic admission.

use std::{collections::BTreeMap, sync::Arc};

use datafusion::{
    catalog::{CatalogProviderList, TableProvider},
    common::{DataFusionError, Result, TableReference},
};
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::model::RelationKey;

use super::{catalog::SnapshotCatalog, list::SnapshotCatalogList};

/// Semantic lookup role. SQL references are derived locators, not these identities.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BindingKey {
    /// An unambiguous declared relation.
    Relation(RelationKey),
    /// A named declared input port.
    Input(String),
    /// A named computed input.
    Computation(String),
    /// An explicitly qualified native source.
    Native(TableReference),
}

/// Exact implementation and, when available, the actual checked source owner.
/// Construction is internal; external providers cannot manufacture checked facts.
#[derive(Clone, Debug)]
pub struct TableBinding {
    /// Fully qualified native lookup name.
    pub reference: TableReference,
    /// The actual retained provider implementation.
    pub provider: Arc<dyn TableProvider>,
    /// Declared relation identity, when this source has one.
    pub relation: Option<RelationKey>,
    /// Actual checked field owner, never a reconstructed validation claim.
    pub checked: Option<FieldCheckedBatch>,
    /// Allocation provenance for checked relations and captured native query chunks.
    pub ownership: Option<pse_columnar::owned_buffer::AllocationScope>,
    /// Owner-defined source evidence retained independently of optimizer elimination.
    pub witness: Option<super::witness::SourceWitness>,
    // Actual native view inputs admitted with an exact Delta member. These are
    // execution dependencies, not additional public namespace or policy aliases.
    /// Actual native view inputs retained for semantic source admission.
    pub dependencies: Vec<Arc<dyn TableProvider>>,
    // Exact durable inputs survive native view expansion and private query scopes.
    // They describe source dependencies, never the identity of a derived result.
    /// Source witnesses inherited through native view expansion.
    pub source_dependencies: Vec<super::witness::SourceWitness>,
    /// Declared operation effects admitted by the effective policy.
    pub effects: std::collections::BTreeSet<pse_schema::model::provider::OperationEffect>,
    /// Actual factory for an isolated private mutation target.
    pub mutation: Option<Arc<dyn crate::session::mutation::PrivateTableFactory>>,
}

impl TableBinding {
    /// Bind an actual provider and optional declaration/checked owner; adds no constraints.
    pub fn new(
        reference: TableReference,
        provider: Arc<dyn TableProvider>,
        relation: Option<RelationKey>,
        checked: Option<FieldCheckedBatch>,
    ) -> Self {
        let ownership = checked
            .as_ref()
            .and_then(FieldCheckedBatch::owned)
            .map(|batch| batch.ownership().clone());
        Self {
            reference,
            provider,
            relation,
            checked,
            ownership,
            witness: None,
            dependencies: Vec::new(),
            source_dependencies: Vec::new(),
            mutation: None,
            effects: [pse_schema::model::provider::OperationEffect::Read]
                .into_iter()
                .collect(),
        }
    }
    /// Mark this source as an observation requiring fresh execution.
    #[must_use]
    pub fn observed(mut self) -> Self {
        self.effects
            .insert(pse_schema::model::provider::OperationEffect::Observe);
        self
    }
}

/// The native hierarchy owns every provider and its established semantic facts.
/// Roles contain locators only. Copy-on-write forks namespace nodes, sharing the
/// actual immutable table and buffer owners with earlier sessions.
#[derive(Clone, Debug, Default)]
pub struct Bindings {
    catalogs: Arc<SnapshotCatalogList>,
    roles: Arc<BTreeMap<BindingKey, TableReference>>,
    resolutions: Arc<BTreeMap<String, Arc<crate::session::resolution::ResolutionGeneration>>>,
    targets: Arc<std::collections::BTreeSet<pse_schema::model::provider::ProviderScope>>,
}

impl Bindings {
    /// Compare retained selection owners, not catalog names or mutable provider addresses.
    pub(crate) fn same_selection(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.catalogs, &other.catalogs)
            && Arc::ptr_eq(&self.roles, &other.roles)
            && Arc::ptr_eq(&self.resolutions, &other.resolutions)
            && Arc::ptr_eq(&self.targets, &other.targets)
    }
    /// Borrow the retained completed metadata generations.
    pub fn resolutions(
        &self,
    ) -> impl Iterator<
        Item = (
            &String,
            &Arc<crate::session::resolution::ResolutionGeneration>,
        ),
    > {
        self.resolutions.iter()
    }
    pub(crate) fn resolutions_only(&self) -> Self {
        Self {
            resolutions: Arc::clone(&self.resolutions),
            ..Self::default()
        }
    }
    pub(crate) fn resolution(
        &mut self,
        catalog: &str,
        generation: Arc<crate::session::resolution::ResolutionGeneration>,
    ) -> Result<()> {
        if let Some(existing) = self.resolutions.get(catalog) {
            return if Arc::ptr_eq(existing, &generation) {
                Ok(())
            } else {
                Err(invalid(
                    "catalog already belongs to another resolution generation",
                ))
            };
        }
        Arc::make_mut(&mut self.resolutions).insert(catalog.to_owned(), generation);
        Ok(())
    }
    pub(crate) fn check_resolution(&self, reference: &TableReference) -> Result<()> {
        if self.lookup(reference).is_some() {
            return Ok(());
        }
        for (catalog, generation) in self.resolutions.iter() {
            if (catalog.is_empty() || Some(catalog.as_str()) == reference.catalog())
                && !generation.covers(reference)
            {
                return Err(invalid(&format!(
                    "reference {} is outside completed {:?} resolution coverage",
                    reference.to_quoted_string(),
                    generation.consistency
                )));
            }
        }
        Ok(())
    }
    fn lookup(&self, reference: &TableReference) -> Option<Arc<TableBinding>> {
        self.catalogs
            .catalog(reference.catalog()?)?
            .schema(reference.schema()?)?
            .downcast_ref::<super::schema::SnapshotSchema>()?
            .binding(reference.table())
    }
    fn install(&mut self, binding: TableBinding) -> Result<()> {
        let catalogs = Arc::make_mut(&mut self.catalogs);
        let catalog = catalogs.ensure_catalog(binding.reference.catalog().unwrap_or_default());
        let catalog = catalog
            .downcast_ref::<SnapshotCatalog>()
            .ok_or_else(|| invalid("bound catalog owner changed"))?;
        let schema = catalog.ensure_schema(binding.reference.schema().unwrap_or_default());
        let schema = schema
            .downcast_ref::<super::schema::SnapshotSchema>()
            .ok_or_else(|| invalid("bound schema owner changed"))?;
        schema.bind(binding);
        Ok(())
    }

    pub(crate) fn mutation_factory(
        &mut self,
        reference: &TableReference,
        factory: Arc<dyn crate::session::mutation::PrivateTableFactory>,
    ) -> Result<()> {
        let mut binding = self
            .lookup(reference)
            .ok_or_else(|| invalid("mutable capture has no exact native binding"))?
            .as_ref()
            .clone();
        binding.mutation = Some(factory);
        self.install(binding)?;
        Ok(())
    }
    pub(crate) fn replace_table(
        &mut self,
        reference: &TableReference,
        replacement: &TableBinding,
    ) -> Result<()> {
        if self.lookup(reference).is_none() || *reference != replacement.reference {
            return Err(invalid(
                "replacement requires its exact existing native name",
            ));
        }
        self.install(replacement.clone())
    }
    /// Resolve a semantic role to its actual retained binding.
    pub fn get(&self, key: &BindingKey) -> Option<Arc<TableBinding>> {
        self.lookup(self.roles.get(key)?)
    }
    /// Visit declared roles and their retained providers.
    pub fn iter(&self) -> impl Iterator<Item = (&BindingKey, Arc<TableBinding>)> {
        self.roles
            .iter()
            .filter_map(|(key, reference)| self.lookup(reference).map(|binding| (key, binding)))
    }
    pub(crate) fn namespace(&mut self, catalog: &str, schema: Option<&str>) -> Result<()> {
        let catalog = Arc::make_mut(&mut self.catalogs).ensure_catalog(catalog);
        if let Some(name) = schema {
            catalog
                .downcast_ref::<SnapshotCatalog>()
                .ok_or_else(|| invalid("bound catalog owner changed"))?
                .ensure_schema(name);
        }
        Ok(())
    }
    pub(crate) fn resolved_metadata(&mut self, reference: &TableReference) -> Result<()> {
        if let Some(binding) = self.lookup(reference) {
            let mut binding = binding.as_ref().clone();
            binding
                .effects
                .remove(&pse_schema::model::provider::OperationEffect::Observe);
            self.install(binding)?;
        }
        Ok(())
    }
    pub(crate) fn target(&mut self, scope: pse_schema::model::provider::ProviderScope) {
        Arc::make_mut(&mut self.targets).insert(scope);
    }
    pub(crate) fn targets(
        &self,
    ) -> impl Iterator<Item = &pse_schema::model::provider::ProviderScope> {
        self.targets.iter()
    }
    pub(crate) fn scopes(&self) -> impl Iterator<Item = (String, Option<String>)> {
        let mut scopes = Vec::new();
        for name in self.catalogs.catalog_names() {
            scopes.push((name.clone(), None));
            if let Some(catalog) = self.catalogs.catalog(&name) {
                scopes.extend(
                    catalog
                        .schema_names()
                        .into_iter()
                        .map(|schema| (name.clone(), Some(schema))),
                );
            }
        }
        scopes.into_iter()
    }
    pub(crate) fn insert(&mut self, key: BindingKey, binding: TableBinding) -> Result<()> {
        if binding.reference.table().is_empty()
            || binding.reference.catalog().is_none_or(str::is_empty)
            || binding.reference.schema().is_none_or(str::is_empty)
        {
            return Err(invalid(
                "a binding requires an exact nonempty catalog/schema/table",
            ));
        }
        if self.roles.contains_key(&key) {
            return Err(invalid(&format!("binding {key:?} is already present")));
        }
        if let Some(existing) = self.lookup(&binding.reference) {
            if !Arc::ptr_eq(&existing.provider, &binding.provider) {
                return Err(invalid(
                    "one native name cannot identify different source owners",
                ));
            }
            if existing.relation != binding.relation
                || existing.witness != binding.witness
                || existing.effects != binding.effects
            {
                return Err(invalid(
                    "one native name cannot carry conflicting semantic facts",
                ));
            }
        } else {
            self.install(binding.clone())?;
        }
        Arc::make_mut(&mut self.roles).insert(key, binding.reference);
        Ok(())
    }
    pub(crate) fn retain(&mut self, mut predicate: impl FnMut(&BindingKey) -> bool) {
        Arc::make_mut(&mut self.roles).retain(|key, _| predicate(key));
        let catalogs = Arc::make_mut(&mut self.catalogs);
        for catalog_name in catalogs.catalog_names() {
            if let Some(catalog) = catalogs.catalog(&catalog_name) {
                for schema_name in catalog.schema_names() {
                    if let Some(schema) = catalog.schema(&schema_name)
                        && let Some(schema) = schema.downcast_ref::<super::schema::SnapshotSchema>()
                    {
                        schema.retain(|table| {
                            self.roles.values().any(|reference| {
                                reference.catalog() == Some(catalog_name.as_str())
                                    && reference.schema() == Some(schema_name.as_str())
                                    && reference.table() == table
                            })
                        });
                    }
                }
            }
        }
    }
    pub(crate) fn providers(&self) -> Vec<Arc<dyn TableProvider>> {
        self.iter()
            .flat_map(|(_, binding)| {
                std::iter::once(Arc::clone(&binding.provider)).chain(binding.dependencies.clone())
            })
            .collect()
    }
    pub(crate) fn contains_provider(&self, provider: &Arc<dyn TableProvider>) -> bool {
        self.iter().any(|(_, entry)| {
            Arc::ptr_eq(&entry.provider, provider)
                || entry
                    .dependencies
                    .iter()
                    .any(|input| Arc::ptr_eq(input, provider))
        })
    }
    pub(crate) fn relation(&self, key: RelationKey) -> Option<Arc<TableBinding>> {
        self.get(&BindingKey::Relation(key))
    }
    /// Resolve a named input port without constructing another provider.
    pub fn input(&self, role: &str) -> Option<Arc<TableBinding>> {
        self.get(&BindingKey::Input(role.to_owned()))
    }
    pub(crate) fn computation(&self, role: &str) -> Option<Arc<TableBinding>> {
        self.get(&BindingKey::Computation(role.to_owned()))
    }
    /// Fork the actual native namespace for one execution; table owners remain shared.
    pub(crate) fn catalogs(
        &self,
        default_catalog: &str,
        default_schema: &str,
    ) -> Result<SnapshotCatalogList> {
        let catalogs = self.catalogs.as_ref().clone();
        if catalogs.catalog(default_catalog).is_none() {
            let catalog = catalogs.ensure_catalog(default_catalog);
            catalog
                .downcast_ref::<SnapshotCatalog>()
                .ok_or_else(|| invalid("bound catalog owner changed"))?
                .ensure_schema(default_schema);
        }
        Ok(catalogs)
    }
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(format!("provider binding: {reason}"))
}

#[cfg(test)]
mod tests;
