// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Admitted physical context shared by authored models and explicit analysis requests.
use super::{Runtime, WorkflowError, contract, relation};
use pse_ids::ContentHash;
use pse_quantity::{PhysicalPreconditions, QuantityRegistry};
use pse_relations::columnar::FieldCheckedBatch;
use std::{collections::BTreeMap, sync::Arc};
/// Immutable physical declarations retaining their actual admitted source rows.
#[derive(Clone, Debug)]
pub struct PhysicalContext {
    pub(crate) quantities: Arc<QuantityRegistry>,
    pub(crate) preconditions: Arc<PhysicalPreconditions>,
    pub(crate) sources: BTreeMap<pse_schema::model::RelationKey, FieldCheckedBatch>,
    pub(crate) key: ContentHash,
    /// The package whose physical document declared the names (ADR-0123 Outcome 6),
    /// when the context was admitted from documents.
    pub(crate) package: Option<PhysicalPackage>,
    pub(super) _inventory: Option<Arc<crate::physical::PhysicalInventory>>,
    pub(super) _source_owner: Option<Arc<super::physical_cache::PhysicalOwner>>,
}
/// The package that declared a physical inventory's names.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PhysicalPackage {
    /// Its identity, which a depending manifest names.
    pub(crate) id: pse_ids::SemanticId,
    /// Its name, which qualifies the physical names.
    pub(crate) name: String,
    /// Its manifest header: a modeling closure that depends on the package resolves the
    /// dependency against the admitted context when the package's documents are not in it.
    pub(crate) header: pse_model::generated::authored::packages::Row,
}
impl PhysicalContext {
    /// Read the admitted registry used to resolve public physical type names.
    pub fn quantities(&self) -> &QuantityRegistry {
        &self.quantities
    }
    /// Actual immutable prerequisites admitted with this physical inventory.
    pub fn preconditions(&self) -> &PhysicalPreconditions {
        &self.preconditions
    }
    /// Custom contexts retain the very rows admitted by PhysicalInventory.
    pub fn admitted(inventory: Arc<crate::physical::PhysicalInventory>) -> Self {
        let quantities = Arc::new(inventory.quantities().clone());
        let preconditions = inventory.compiler_preconditions();
        let key = pse_compiler::workspace::physical_identity(&quantities, &preconditions);
        Self {
            quantities,
            preconditions,
            key,
            sources: inventory.source_batches().clone(),
            package: None,
            _inventory: Some(inventory),
            _source_owner: None,
        }
    }
    /// Full exact physical context identity, not a caller's revision assertion.
    pub fn identity(&self) -> ContentHash {
        self.key
    }
    pub(super) fn same_sources(&self, other: &Self) -> bool {
        self.key == other.key
            && Arc::ptr_eq(&self.quantities, &other.quantities)
            && Arc::ptr_eq(&self.preconditions, &other.preconditions)
            && self.package == other.package
            && self.sources.len() == other.sources.len()
            && self.sources.iter().all(|(key, batch)| {
                other
                    .sources
                    .get(key)
                    .is_some_and(|other| batch.same_source(other))
            })
    }
    pub(super) fn retained_bytes(&self) -> usize {
        use pse_model::HeapUsage;
        let sources = self.sources.values().fold(0_usize, |bytes, batch| {
            bytes
                .saturating_add(
                    batch
                        .owned()
                        .and_then(|owned| owned.retained_bytes().ok())
                        .unwrap_or_else(|| batch.batch().get_array_memory_size()),
                )
                .saturating_add(512)
        });
        sources
            .saturating_add(self.quantities.allocation_extent())
            .saturating_add(self.preconditions.allocation_extent())
            .saturating_add(
                self.package
                    .as_ref()
                    .map_or(0, |p| p.header.heap_bytes() + p.name.len() + 256),
            )
            .saturating_add(1024)
    }
}

impl Runtime {
    /// Admit the exact physical sources through the existing registry/session boundary.
    pub async fn physical_from_documents(
        &self,
        documents: &crate::authoring_driver::document::OwnedDocumentSet,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<PhysicalContext, WorkflowError> {
        PhysicalContext::from_documents(documents, self.registry.clone(), &self.sessions, cancel)
            .await
    }
}
impl PhysicalContext {
    /// Shared physical admission for workflows and standalone pure checks.
    pub(crate) async fn from_documents(
        documents: &crate::authoring_driver::document::OwnedDocumentSet,
        registry: Arc<pse_schema::Registry>,
        sessions: &pse_engine::session::EngineFactory,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Self, WorkflowError> {
        documents.validate_context(
            &registry,
            sessions.validation_context(&registry)?.as_ref(),
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
                return Err(contract(
                    "one package declares the physical inventory and its names",
                ));
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
            sessions.validation_context(&registry)?.as_ref(),
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
            .map_err(pse_relations::RelationError::from)
            .map_err(relation)?;
        let mut physical = BTreeMap::new();
        let mut retained_support = BTreeMap::new();
        for (id, batch) in batches {
            let spec = registry
                .relation_by_id(id)
                .ok_or_else(|| contract("unknown physical declaration"))?;
            if keys.contains(&spec.key) {
                physical.insert(spec.key, batch);
            } else if support.contains(&id) {
                retained_support.insert(
                    spec.key,
                    batch.retained(sessions.pool(), cancel).map_err(relation)?,
                );
            }
        }
        // Physical inventory consists of independent, bounded registry projections.
        // `execute_group` runs several of those scans concurrently; letting each
        // inherit the deployment's full target width would make every query try to
        // reserve the whole shared CPU semaphore. Keep the same factory owners and
        // implementations, but give each inventory scan its actual one-partition
        // demand. Other sessions and scientific work retain the deployment width.
        let inventory_sessions = sessions
            .clone()
            .with_target_partitions(std::num::NonZeroUsize::MIN);
        let session = inventory_sessions.candidate_checked(physical, registry.clone(), cancel)?;
        let inventory = crate::physical::PhysicalInventory::load(&session, &registry, cancel)
            .await
            .map_err(|e| pse_engine::EngineError::Semantic(Arc::new(e)))?;
        let mut context = PhysicalContext::admitted(Arc::new(inventory));
        context.package = package;
        context.sources.extend(retained_support);
        Ok(context)
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
        let validation = sessions.validation_context(&registry).unwrap();
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

        let inventory = PhysicalContext::from_documents(&documents, registry, &sessions, &cancel)
            .await
            .unwrap();
        let loaded = inventory._inventory.as_ref().unwrap().source_batches();
        assert!(
            loaded.len() > 1,
            "fixture must execute multiple physical relation scans in one group"
        );
        assert_eq!(permits.available_permits(), 16);
    }
}
