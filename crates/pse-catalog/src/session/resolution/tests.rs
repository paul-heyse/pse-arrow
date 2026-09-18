// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Empty in-memory metadata source unit; no external storage or model journey.
use super::*;
use datafusion::{
    catalog::AsyncCatalogProvider,
    execution::{runtime_env::RuntimeEnv, session_state::SessionStateBuilder},
};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
struct EmptySource(Arc<AtomicUsize>);
impl CatalogSource for EmptySource {
    fn open<'a>(
        &'a self,
        _: ResolutionConsistency,
        _: &'a SnapshotSession,
        _: &'a CancellationToken,
    ) -> BoxFut<'a, Result<Arc<dyn CatalogRevision>, CatalogError>> {
        Box::pin(async move {
            self.0.fetch_add(1, Ordering::SeqCst);
            let revision: Arc<dyn CatalogRevision> = Arc::new(EmptyRevision);
            Ok(revision)
        })
    }
}
#[derive(Debug)]
struct EmptyRevision;
impl CatalogRevision for EmptyRevision {
    fn all_references(&self) -> Option<&[TableReference]> {
        Some(&[])
    }
}
impl AsyncCatalogProviderList for EmptyRevision {
    fn catalog<'s, 'n, 'f>(
        &'s self,
        _: &'n str,
    ) -> BoxFut<'f, datafusion::common::Result<Option<Arc<dyn AsyncCatalogProvider>>>>
    where
        's: 'f,
        'n: 'f,
        Self: 'f,
    {
        Box::pin(async { Ok(None) })
    }
}

#[tokio::test]
async fn native_metadata_source_is_lazy_and_does_not_retain_its_own_result() {
    let cancel = CancellationToken::new();
    let factory = super::super::SessionFactory::from_builder(
        Arc::new(RuntimeEnv::default()),
        pse_ids::FixedBudget::new(8 << 20),
        "metadata-unit",
        SessionStateBuilder::new_with_default_features(),
    );
    let session = factory
        .candidate_checked(
            std::collections::BTreeMap::new(),
            Arc::new(pse_schema::builder::RegistryBuilder::new().build().unwrap()),
            &cancel,
        )
        .unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let prepared = session
        .prepare_catalog_resolution(
            Arc::new(EmptySource(Arc::clone(&calls))),
            ResolutionRequest::All,
            ResolutionConsistency::Snapshot,
            &cancel,
        )
        .unwrap();
    let weak = Arc::downgrade(&prepared.operation);
    let native = prepared.computation().clone();
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(
        native
            .original_plan()
            .display_indent()
            .to_string()
            .contains("TableScan")
    );
    drop(prepared);
    let completion = native.execute(&cancel).await.unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(weak.upgrade().unwrap().output.lock().unwrap().is_some());
    drop(completion);
    assert!(
        weak.upgrade().is_none(),
        "the resolved context must not retain its own source provider"
    );
}
