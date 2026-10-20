// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Physical document admission retaining source, package and allocation owners.

use super::{PhysicalError, PhysicalInventory};
use pse_relations::columnar::FieldCheckedBatch;
use std::{collections::BTreeMap, sync::Arc};

/// Optional caller-owned copy destination; never retains the performing native session.
#[derive(Clone, Debug)]
pub struct PhysicalAdmissionReceipt {
    maximum_bytes: usize,
    observations:
        Option<Arc<pse_columnar::Leased<pse_engine::session::CopiedExecutionObservations>>>,
    unavailable: Option<String>,
    retained_bytes: usize,
}
impl PhysicalAdmissionReceipt {
    /// Select a finite copy allowance independently of scientific admission.
    pub fn new(maximum_bytes: usize) -> Self {
        Self {
            maximum_bytes,
            observations: None,
            unavailable: Some("no physical session was established".into()),
            retained_bytes: 0,
        }
    }
    /// Clear a reused destination before a new admission can fail or establish a session.
    pub fn reset(&mut self) {
        self.observations = None;
        self.retained_bytes = 0;
        self.unavailable = Some("no physical session was established".into());
    }
    /// Actual performed-session facts, retained with their copied allocation only.
    pub fn observations(&self) -> Option<&pse_engine::session::CopiedExecutionObservations> {
        self.observations.as_deref().map(|facts| &**facts)
    }
    /// Actual retained copy allocation, independent of scientific source ownership.
    pub fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }
    /// Why optional copying was unavailable; this never changes the admission result.
    pub fn unavailable(&self) -> Option<&str> {
        self.unavailable.as_deref()
    }
    fn capture(&mut self, session: &pse_engine::session::EngineSession) {
        self.observations = None;
        self.retained_bytes = 0;
        let copy = || -> Result<_, String> {
            if self.maximum_bytes < 4096 || self.maximum_bytes > 64 << 20 {
                return Err("physical receipt requires 4096 bytes through 64 MiB allowance".into());
            }
            // Optional copying owns an independent finite pool and cannot consume
            // the executing scientific pool or change its admission outcome.
            let pool: Arc<dyn pse_columnar::MemoryPool> =
                Arc::new(pse_columnar::GreedyMemoryPool::new(self.maximum_bytes));
            let reservation =
                pse_columnar::MemoryConsumer::new("physical:performed-session-receipt")
                    .register(&pool);
            reservation
                .try_grow(self.maximum_bytes)
                .map_err(|error| error.to_string())?;
            let (facts, retained_bytes) = session
                .copy_execution_observations_bounded(self.maximum_bytes)
                .map_err(|error| error.to_string())?;
            Ok((
                Arc::new(pse_columnar::Leased::new(
                    Arc::new(facts),
                    pse_columnar::AllocationLease::new(reservation.split(retained_bytes)),
                )),
                retained_bytes,
            ))
        };
        match copy() {
            Ok((facts, bytes)) => {
                self.observations = Some(facts);
                self.retained_bytes = bytes;
                self.unavailable = None;
            }
            Err(error) => self.unavailable = Some(error),
        }
    }
}
struct ReceiptGuard<'a> {
    session: &'a pse_engine::session::EngineSession,
    receipt: Option<&'a mut PhysicalAdmissionReceipt>,
}
impl Drop for ReceiptGuard<'_> {
    fn drop(&mut self) {
        if let Some(receipt) = self.receipt.as_deref_mut() {
            receipt.capture(self.session);
        }
    }
}
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
        Self::from_documents_with_receipt(documents, model, cancel, None).await
    }
    /// Admit sources while copying actual performed session facts into an optional
    /// caller-owned destination, including an error or a dropped admission future.
    /// Copy refusal never replaces the scientific outcome.
    pub async fn from_documents_with_receipt(
        documents: &crate::authoring_driver::document::OwnedDocumentSet,
        model: &pse_engine::session::BoundModel,
        cancel: &pse_columnar::CancellationToken,
        mut receipt: Option<&mut PhysicalAdmissionReceipt>,
    ) -> Result<Self, PhysicalError> {
        if let Some(receipt) = receipt.as_deref_mut() {
            receipt.reset();
        }
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
        let _receipt_guard = ReceiptGuard {
            session: &session,
            receipt,
        };
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
        let permits = Arc::new(pse_engine::resources::ReleaseNotifyingSemaphore::new(16));
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

        use pse_engine::session::{CopiedObservationPolicy, assurance::ObservationPolicy};
        for (policy, copied_policy) in [
            (ObservationPolicy::Off, CopiedObservationPolicy::Off),
            (
                ObservationPolicy::Contract,
                CopiedObservationPolicy::Contract,
            ),
            (
                ObservationPolicy::Diagnostic,
                CopiedObservationPolicy::Diagnostic,
            ),
        ] {
            let model = Arc::new(sessions.clone().with_observation(policy))
                .bind_model(registry.clone())
                .unwrap();
            let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
            let operation = hub.start("physical.performed-admission");
            let mut receipt = PhysicalAdmissionReceipt::new(16 << 20);
            let inventory = pse_diagnostics::observation::scope(
                Some(operation.context()),
                PhysicalDocuments::from_documents_with_receipt(
                    &documents,
                    &model,
                    &cancel,
                    Some(&mut receipt),
                ),
            )
            .await
            .unwrap();
            assert!(
                inventory.inventory.source_batches().len() > 1,
                "fixture executes multiple actual physical scans in one group"
            );
            let observations = receipt.observations().unwrap();
            assert_eq!(observations.policy, Some(copied_policy));
            assert_eq!(observations.operation, Some(operation.context().id()));
            if policy == ObservationPolicy::Off {
                assert!(observations.observations.is_empty());
                assert!(receipt.retained_bytes() < 16384);
            } else {
                assert!(!observations.observations.is_empty());
                assert!(
                    observations
                        .observations
                        .iter()
                        .all(|o| o.policy == copied_policy)
                );
                assert!(
                    observations
                        .observations
                        .iter()
                        .any(|o| o.execution.is_some())
                );
            }
            let early_cancel = CancellationToken::new();
            early_cancel.cancel();
            assert!(
                PhysicalDocuments::from_documents_with_receipt(
                    &documents,
                    &model,
                    &early_cancel,
                    Some(&mut receipt)
                )
                .await
                .is_err()
            );
            assert!(
                receipt.observations().is_none(),
                "early refusal must not reuse previous session facts"
            );
            assert_eq!(receipt.retained_bytes(), 0);
            assert_eq!(permits.available_permits(), 16);
            // Refusing the selected diagnostic copy must preserve science success.
            let mut refused = PhysicalAdmissionReceipt::new(1);
            PhysicalDocuments::from_documents_with_receipt(
                &documents,
                &model,
                &cancel,
                Some(&mut refused),
            )
            .await
            .unwrap();
            assert!(refused.observations().is_none());
            assert!(refused.unavailable().is_some());
            // A valid but insufficient copy allowance also preserves admission;
            // its full collector is sized before payload strings/metrics are cloned.
            let mut bounded = PhysicalAdmissionReceipt::new(4096);
            PhysicalDocuments::from_documents_with_receipt(
                &documents,
                &model,
                &cancel,
                Some(&mut bounded),
            )
            .await
            .unwrap();
            assert!(bounded.observations().is_none());
            assert_eq!(bounded.retained_bytes(), 0);
            assert!(
                bounded
                    .unavailable()
                    .unwrap()
                    .contains("allocation allowance")
            );
        }
        // A dropped caller future still copies the already performed session facts
        // through the same scope guard, without retaining a session in the receipt.
        let session = model.candidate_checked(BTreeMap::new(), &cancel).unwrap();
        let mut receipt = PhysicalAdmissionReceipt::new(1 << 20);
        let mut waiting = Box::pin(async {
            let _guard = ReceiptGuard {
                session: &session,
                receipt: Some(&mut receipt),
            };
            std::future::pending::<()>().await;
        });
        let mut context = std::task::Context::from_waker(std::task::Waker::noop());
        assert!(Future::poll(waiting.as_mut(), &mut context).is_pending());
        drop(waiting);
        drop(session);
        assert_eq!(
            receipt.observations().unwrap().policy,
            Some(CopiedObservationPolicy::Off)
        );
    }
}
