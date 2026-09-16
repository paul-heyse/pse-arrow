// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One immutable source inventory for native lookup, roles and semantic admission.

use std::{collections::BTreeMap, sync::Arc};

use datafusion::{
    catalog::{CatalogProvider, TableProvider},
    common::{DataFusionError, Result, TableReference},
};
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::model::RelationKey;

use super::{catalog::SnapshotCatalog, list::SnapshotCatalogList};

/// Semantic lookup role. SQL references are derived locators, not these identities.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum BindingKey {
    Catalog(String),
    Schema(String, String),
    Relation(RelationKey),
    Input(String),
    Computation(String),
    Native(TableReference),
    Output(String),
    SnapshotPort(String),
    Target(pse_schema::model::provider::ProviderScope),
    Resolution(String),
}

#[derive(Clone, Debug)]
enum BoundOwner {
    Namespace,
    Table(Arc<TableBinding>),
    Resolution(Arc<crate::session::resolution::ResolutionGeneration>),
}

/// Exact implementation and, when available, the actual checked source owner.
/// Construction is internal; external providers cannot manufacture checked facts.
#[derive(Clone, Debug)]
pub(crate) struct TableBinding {
    pub(crate) reference: TableReference,
    pub(crate) provider: Arc<dyn TableProvider>,
    pub(crate) relation: Option<RelationKey>,
    pub(crate) checked: Option<FieldCheckedBatch>,
    pub(crate) selection: Option<
        pse_relations::generated::runtime::publications::RuntimePublicationsFieldMembersItem,
    >,
    // Actual native view inputs admitted with an exact Delta member. These are
    // execution dependencies, not additional public namespace or policy aliases.
    pub(crate) dependencies: Vec<Arc<dyn TableProvider>>,
    pub(crate) effects: std::collections::BTreeSet<pse_schema::model::provider::OperationEffect>,
    pub(crate) mutation: Option<Arc<dyn crate::session::mutation::PrivateTableFactory>>,
}

impl TableBinding {
    pub(crate) fn new(
        reference: TableReference,
        provider: Arc<dyn TableProvider>,
        relation: Option<RelationKey>,
        checked: Option<FieldCheckedBatch>,
    ) -> Self {
        Self {
            reference,
            provider,
            relation,
            checked,
            selection: None,
            dependencies: Vec::new(),
            mutation: None,
            effects: [pse_schema::model::provider::OperationEffect::Read]
                .into_iter()
                .collect(),
        }
    }
    pub(crate) fn observed(mut self) -> Self {
        self.effects
            .insert(pse_schema::model::provider::OperationEffect::Observe);
        self
    }
}

/// Clones share immutable entries. A fork copies only the binding index; schemas,
/// data, implementations and checked owners remain shared. Native catalog objects
/// are created at operation preparation, not after every role addition.
#[derive(Clone, Debug, Default)]
pub(crate) struct Bindings {
    entries: Arc<BTreeMap<BindingKey, BoundOwner>>,
}

impl Bindings {
    pub(crate) fn resolutions_only(&self) -> Self {
        Self {
            entries: Arc::new(
                self.entries
                    .iter()
                    .filter(|(_, value)| matches!(value, BoundOwner::Resolution(_)))
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect(),
            ),
        }
    }
    pub(crate) fn resolution(
        &mut self,
        catalog: &str,
        generation: Arc<crate::session::resolution::ResolutionGeneration>,
    ) -> Result<()> {
        let key = BindingKey::Resolution(catalog.to_owned());
        if let Some(BoundOwner::Resolution(existing)) = self.entries.get(&key) {
            return if Arc::ptr_eq(existing, &generation) {
                Ok(())
            } else {
                Err(invalid(
                    "catalog already belongs to another resolution generation",
                ))
            };
        }
        Arc::make_mut(&mut self.entries).insert(key, BoundOwner::Resolution(generation));
        Ok(())
    }

    pub(crate) fn check_resolution(&self, reference: &TableReference) -> Result<()> {
        if self
            .iter()
            .any(|(_, binding)| binding.reference == *reference)
        {
            return Ok(());
        }
        for (key, owner) in self.entries.iter() {
            if let (BindingKey::Resolution(catalog), BoundOwner::Resolution(generation)) =
                (key, owner)
                && (catalog.is_empty() || Some(catalog.as_str()) == reference.catalog())
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

    pub(crate) fn mutation_factory(
        &mut self,
        reference: &TableReference,
        factory: Arc<dyn crate::session::mutation::PrivateTableFactory>,
    ) -> Result<()> {
        let Some(BoundOwner::Table(binding)) =
            Arc::make_mut(&mut self.entries).get_mut(&BindingKey::Native(reference.clone()))
        else {
            return Err(invalid("mutable capture has no exact native binding"));
        };
        Arc::make_mut(binding).mutation = Some(factory);
        Ok(())
    }

    pub(crate) fn replace_table(&mut self, reference: &TableReference, replacement: &TableBinding) {
        for owner in Arc::make_mut(&mut self.entries).values_mut() {
            if let BoundOwner::Table(binding) = owner
                && binding.reference == *reference
            {
                *binding = Arc::new(replacement.clone());
            }
        }
    }

    pub(crate) fn get(&self, key: &BindingKey) -> Option<&TableBinding> {
        match self.entries.get(key) {
            Some(BoundOwner::Table(table)) => Some(table.as_ref()),
            _ => None,
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&BindingKey, &TableBinding)> {
        self.entries.iter().filter_map(|(key, owner)| match owner {
            BoundOwner::Table(table) => Some((key, table.as_ref())),
            BoundOwner::Namespace | BoundOwner::Resolution(_) => None,
        })
    }

    pub(crate) fn namespace(&mut self, catalog: &str, schema: Option<&str>) {
        let key = schema.map_or_else(
            || BindingKey::Catalog(catalog.to_owned()),
            |schema| BindingKey::Schema(catalog.to_owned(), schema.to_owned()),
        );
        Arc::make_mut(&mut self.entries).insert(key, BoundOwner::Namespace);
    }
    pub(crate) fn resolved_metadata(&mut self, reference: &TableReference) {
        if let Some(BoundOwner::Table(table)) =
            Arc::make_mut(&mut self.entries).get_mut(&BindingKey::Native(reference.clone()))
        {
            Arc::make_mut(table)
                .effects
                .remove(&pse_schema::model::provider::OperationEffect::Observe);
        }
    }

    pub(crate) fn target(&mut self, scope: pse_schema::model::provider::ProviderScope) {
        Arc::make_mut(&mut self.entries).insert(BindingKey::Target(scope), BoundOwner::Namespace);
    }

    pub(crate) fn targets(
        &self,
    ) -> impl Iterator<Item = &pse_schema::model::provider::ProviderScope> {
        self.entries.keys().filter_map(|key| match key {
            BindingKey::Target(scope) => Some(scope),
            _ => None,
        })
    }

    pub(crate) fn scopes(&self) -> impl Iterator<Item = (&str, Option<&str>)> {
        self.entries.keys().filter_map(|key| match key {
            BindingKey::Catalog(c) => Some((c.as_str(), None)),
            BindingKey::Schema(c, s) => Some((c.as_str(), Some(s.as_str()))),
            _ => None,
        })
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
        if self.entries.contains_key(&key) {
            return Err(invalid(&format!("binding {key:?} is already present")));
        }
        if self.iter().any(|(_, entry)| {
            entry.reference == binding.reference && !Arc::ptr_eq(&entry.provider, &binding.provider)
        }) {
            return Err(invalid(
                "one native name cannot identify different source owners",
            ));
        }
        Arc::make_mut(&mut self.entries).insert(key, BoundOwner::Table(Arc::new(binding)));
        Ok(())
    }

    pub(crate) fn retain(&mut self, mut predicate: impl FnMut(&BindingKey) -> bool) {
        Arc::make_mut(&mut self.entries).retain(|key, _| predicate(key));
    }

    pub(crate) fn providers(&self) -> Vec<Arc<dyn TableProvider>> {
        self.iter()
            .flat_map(|(_, binding)| {
                std::iter::once(Arc::clone(&binding.provider))
                    .chain(binding.dependencies.iter().cloned())
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

    pub(crate) fn relation(&self, key: RelationKey) -> Option<&TableBinding> {
        self.get(&BindingKey::Relation(key))
    }

    pub(crate) fn input(&self, role: &str) -> Option<&TableBinding> {
        self.get(&BindingKey::Input(role.to_owned()))
    }

    pub(crate) fn computation(&self, role: &str) -> Option<&TableBinding> {
        self.get(&BindingKey::Computation(role.to_owned()))
    }

    /// Projection of the same entries into the native hierarchy. Even an empty
    /// session has its configured default catalog/schema for scalar-only queries.
    pub(crate) fn catalogs(
        &self,
        default_catalog: &str,
        default_schema: &str,
    ) -> SnapshotCatalogList {
        let mut catalogs =
            BTreeMap::<String, BTreeMap<(String, String), Arc<dyn TableProvider>>>::new();
        catalogs.entry(default_catalog.to_owned()).or_default();
        for (catalog, _) in self.scopes() {
            catalogs.entry(catalog.to_owned()).or_default();
        }
        for (_, binding) in self.iter() {
            catalogs
                .entry(
                    binding
                        .reference
                        .catalog()
                        .unwrap_or(default_catalog)
                        .to_owned(),
                )
                .or_default()
                .insert(
                    (
                        binding
                            .reference
                            .schema()
                            .unwrap_or(default_schema)
                            .to_owned(),
                        binding.reference.table().to_owned(),
                    ),
                    Arc::clone(&binding.provider),
                );
        }
        let generation = Arc::new(std::sync::atomic::AtomicU64::new(0));
        SnapshotCatalogList::with_generation(
            catalogs
                .into_iter()
                .map(|(name, tables)| {
                    let catalog: Arc<dyn CatalogProvider> = Arc::new(SnapshotCatalog::from_tables(
                        &tables,
                        (name == default_catalog
                            && !self
                                .entries
                                .contains_key(&BindingKey::Catalog(name.clone())))
                        .then_some(default_schema),
                        self.scopes().filter_map(|(catalog, schema)| {
                            (catalog == name).then_some(schema).flatten()
                        }),
                        Arc::clone(&generation),
                    ));
                    (name, catalog)
                })
                .collect(),
            generation,
        )
    }
}

fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(format!("provider binding: {reason}"))
}
