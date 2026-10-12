// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Physical document admission retaining source, package and allocation owners.

use super::{PhysicalError, PhysicalInventory};
use pse_relations::columnar::FieldCheckedBatch;
use std::{collections::BTreeMap, sync::Arc};

/// The package that declared a physical inventory's names.
#[derive(Clone, Debug, PartialEq)]
pub struct PhysicalPackage {
    /// Its identity, which a depending manifest names.
    pub id: pse_ids::SemanticId,
    /// Its name, which qualifies the physical names.
    pub name: String,
    /// Its manifest header: a modeling closure that depends on the package resolves the
    /// dependency against the admitted context when the package's documents are not in it.
    pub header: pse_model::generated::authored::packages::Row,
}
/// Physical declarations admitted from owned documents under the actual bound model.
#[derive(Debug)]
pub struct PhysicalDocuments {
    /// Exact physical inventory retaining its selected source allocations.
    inventory: Arc<PhysicalInventory>,
    /// Physical inputs and support closure retaining their original checked owners.
    sources: BTreeMap<pse_schema::model::RelationKey, FieldCheckedBatch>,
    /// Package that declared public physical names, when supplied.
    package: Option<PhysicalPackage>,
}

impl PhysicalDocuments {
    /// Actual admitted physical inventory.
    pub fn inventory(&self) -> &Arc<PhysicalInventory> {
        &self.inventory
    }

    /// Transfer the admitted inventory, checked support sources and declaring package together.
    pub fn into_parts(
        self,
    ) -> (
        Arc<PhysicalInventory>,
        BTreeMap<pse_schema::model::RelationKey, FieldCheckedBatch>,
        Option<PhysicalPackage>,
    ) {
        (self.inventory, self.sources, self.package)
    }

    /// Shared physical admission for workflows and standalone pure checks.
    ///
    /// # Errors
    /// Document/reference closure refusal, incompatible physical definitions, native
    /// execution failure, cancellation or finite allocation exhaustion.
    pub async fn from_documents(
        documents: &crate::authoring_driver::document::OwnedDocumentSet,
        model: &pse_engine::session::BoundModel,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Self, PhysicalError> {
        let registry = model.registry().clone();
        let sessions = model.factory();
        documents.validate_context(
            &registry,
            model.validation_context().as_ref(),
            sessions.pool(),
            cancel,
        )?;
        let headers = documents
            .bundles()
            .iter()
            .map(|bundle| bundle.package.clone())
            .collect::<Vec<_>>();
        let limits = pse_authoring::p0::GraphLimits {
            nodes: headers.len(),
            edges: crate::authoring_driver::work::sources(documents.bundles())? / 128,
        };
        crate::authoring_driver::p1::admit_package_closure(documents.bundles(), &headers, limits)?;
        // ADR-0123 Outcome 6: the package whose physical document declares quantity types
        // declares their names; one package declares the inventory (register R-51).
        let declaring = documents
            .bundles()
            .iter()
            .filter(|bundle| {
                bundle
                    .batches
                    .get(&pse_relations::generated::reference::quantity_types::RELATION_ID)
                    .is_some_and(|batch| batch.batch().num_rows() > 0)
            })
            .map(|bundle| PhysicalPackage {
                id: bundle.package.package_id.as_id(),
                name: bundle.package.name.clone(),
                header: bundle.package.clone(),
            })
            .collect::<Vec<_>>();
        let package = match declaring.as_slice() {
            [] => None,
            [one] => Some(one.clone()),
            _ => {
                return Err(crate::authoring_driver::contract(
                    "one package declares the physical inventory and its names",
                )
                .into());
            }
        };
        let mut batches = crate::authoring_driver::p1::source_context_batches(
            documents.bundles(),
            &registry,
            &headers,
            limits,
        )?;
        if let Some(entities) = crate::authoring_driver::p1::physical_declaration_batch(
            documents.bundles(),
            &registry,
            model.validation_context().as_ref(),
            sessions.pool(),
            cancel,
        )? {
            batches.insert(
                pse_relations::generated::authored::modeling_declarations::RELATION_ID,
                entities,
            );
        }
        let keys = crate::physical::input_keys(&registry);
        let roots = keys
            .iter()
            .filter_map(|key| registry.relation(&key.qualified_name()).map(|spec| spec.id))
            .collect();
        let support = pse_schema::product::support_closure(&registry, &roots)
            .map_err(pse_relations::RelationError::from)?;
        let mut physical = BTreeMap::new();
        let mut retained_support = BTreeMap::new();
        for (id, batch) in batches {
            let spec = registry
                .relation_by_id(id)
                .ok_or_else(|| crate::authoring_driver::contract("unknown physical declaration"))?;
            if keys.contains(&spec.key) {
                physical.insert(spec.key, batch);
            } else if support.contains(&id) {
                retained_support.insert(spec.key, batch.retained(sessions.pool(), cancel)?);
            }
        }
        // Physical inventory consists of independent, bounded registry projections.
        // `execute_group` runs several of those scans concurrently; letting each
        // inherit the deployment's full target width would make every query try to
        // reserve the whole shared CPU semaphore. Keep the same factory owners and
        // implementations, but give each inventory scan its actual one-partition
        // demand. Other sessions and scientific work retain the deployment width.
        let inventory_sessions = sessions
            .as_ref()
            .clone()
            .with_target_partitions(std::num::NonZeroUsize::MIN);
        let inventory_model = Arc::new(inventory_sessions).bind_model(registry.clone())?;
        let session = inventory_model.candidate_checked(physical, cancel)?;
        let inventory = Arc::new(PhysicalInventory::load(&session, &registry, cancel).await?);
        let mut sources = inventory.source_batches().clone();
        sources.extend(retained_support);
        Ok(Self {
            inventory,
            sources,
            package,
        })
    }
}

#[cfg(test)]
mod admission_tests {
    #![allow(
        clippy::unwrap_used,
        reason = "the physical fixture is a checked local package required to exercise grouped inventory admission"
    )]

    use super::*;
    use crate::authoring_driver::document::{OwnedDocumentSet, load_package_documents_owned};
    use pse_columnar::CancellationToken;
    use pse_engine::{
        cache_service::{CacheBudget, NativeCacheService},
        resources::CpuAdmission,
        session::{EngineFactory, ExecutionSettings, ThreadBudget, native_engine_profile},
    };
    use std::{collections::BTreeMap, num::NonZeroUsize, path::Path, sync::Arc};

    fn collect_documents(root: &Path, at: &Path, rows: &mut BTreeMap<String, Vec<u8>>) {
        for entry in std::fs::read_dir(at).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect_documents(root, &path, rows);
            } else if matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("toml" | "yaml" | "yml" | "pse" | "parquet")
            ) {
                rows.insert(
                    path.strip_prefix(root)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .replace('\\', "/"),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }

    #[tokio::test]
    async fn cold_multirelation_inventory_shares_cpu_without_fullwidth_sibling_refusal() {
        let registry = pse_schema::shared_registry().unwrap();
        let pool: Arc<dyn datafusion::execution::memory_pool::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(512 << 20));
        let mut cache_policy = CacheBudget::for_memory(512 << 20);
        cache_policy.concurrent_outputs = NonZeroUsize::new(4).unwrap();
        let caches = NativeCacheService::new(cache_policy, &pool).unwrap();
        let permits = Arc::new(tokio::sync::Semaphore::new(16));
        let sessions = EngineFactory::new(
            Arc::new(datafusion::execution::runtime_env::RuntimeEnv::default()),
            pool.clone(),
            ExecutionSettings::default(),
            ThreadBudget {
                pool_threads: NonZeroUsize::new(16).unwrap(),
                target_partitions: NonZeroUsize::new(16).unwrap(),
            },
            native_engine_profile(),
        )
        .unwrap()
        .with_cache_service(caches.clone())
        .with_extension(caches)
        .with_extension(Arc::new(CpuAdmission {
            permits: permits.clone(),
            workers: 16.try_into().unwrap(),
        }));
        assert_eq!(
            sessions
                .native_state()
                .config()
                .options()
                .execution
                .target_partitions,
            16
        );

        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/packages/physical-primitives");
        let mut source = BTreeMap::new();
        collect_documents(&root, &root, &mut source);
        let cancel = CancellationToken::new();
        let model = Arc::new(sessions.clone())
            .bind_model(registry.clone())
            .unwrap();
        let validation = model.validation_context();
        let bundle = load_package_documents_owned(
            &source,
            &registry,
            Default::default(),
            &pool,
            &cancel,
            &validation,
        )
        .unwrap();
        let documents = OwnedDocumentSet::try_from_bundles(vec![bundle], &pool, &cancel).unwrap();

        let inventory = PhysicalDocuments::from_documents(&documents, &model, &cancel)
            .await
            .unwrap();
        let loaded = inventory.inventory.source_batches();
        assert!(
            loaded.len() > 1,
            "fixture must execute multiple physical relation scans in one group"
        );
        assert_eq!(permits.available_permits(), 16);
    }
}
