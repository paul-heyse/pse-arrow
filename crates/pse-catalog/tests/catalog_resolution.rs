// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Remote resolution uses actual native handles, bounded discovery and explicit coverage.
#![allow(clippy::unwrap_used, reason = "fixed native resolver fixtures")]

use datafusion::{
    arrow::{
        array::{RecordBatch, UInt64Array},
        datatypes::{DataType, Field, Schema},
    },
    catalog::{AsyncCatalogProvider, AsyncCatalogProviderList, AsyncSchemaProvider, TableProvider},
    common::{DataFusionError, Result as NativeResult, TableReference},
    datasource::MemTable,
};
use pse_columnar::CancellationToken;
use pse_engine::{
    BoxFut, EngineError,
    session::{
        EngineSession,
        resolution::{CatalogRevision, CatalogSource, ResolutionConsistency, ResolutionRequest},
    },
};
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
};

#[derive(Debug, Default)]
struct Counters {
    opens: AtomicUsize,
    catalogs: AtomicUsize,
    schemas: AtomicUsize,
    tables: AtomicUsize,
}
#[derive(Debug)]
struct Source {
    value: Arc<AtomicU64>,
    counts: Arc<Counters>,
    supports_snapshot: bool,
    pending: bool,
}
impl Default for Source {
    fn default() -> Self {
        Self {
            value: Arc::new(AtomicU64::new(7)),
            counts: Arc::default(),
            supports_snapshot: true,
            pending: false,
        }
    }
}
impl CatalogSource for Source {
    fn open<'a>(
        &'a self,
        consistency: ResolutionConsistency,
        _: &'a EngineSession,
        _: &'a CancellationToken,
    ) -> BoxFut<'a, Result<Arc<dyn CatalogRevision>, EngineError>> {
        Box::pin(async move {
            self.counts.opens.fetch_add(1, Ordering::SeqCst);
            if self.pending {
                futures_util::future::pending::<()>().await;
            }
            if consistency == ResolutionConsistency::Snapshot && !self.supports_snapshot {
                return Err(EngineError::Admission {
                    path: "fixture.revision".into(),
                    reason: "coherent revision unavailable".into(),
                });
            }
            let value = self.value.load(Ordering::SeqCst);
            let revision: Arc<dyn CatalogRevision> = Arc::new(Revision {
                value,
                counts: Arc::clone(&self.counts),
                references: vec![reference("t"), reference("u")],
            });
            Ok(revision)
        })
    }
}
#[derive(Clone, Debug)]
struct Revision {
    value: u64,
    counts: Arc<Counters>,
    references: Vec<TableReference>,
}
impl CatalogRevision for Revision {
    fn all_references(&self) -> Option<&[TableReference]> {
        Some(&self.references)
    }
}
impl AsyncCatalogProviderList for Revision {
    fn catalog<'s, 'n, 'f>(
        &'s self,
        name: &'n str,
    ) -> BoxFut<'f, NativeResult<Option<Arc<dyn AsyncCatalogProvider>>>>
    where
        's: 'f,
        'n: 'f,
        Self: 'f,
    {
        Box::pin(async move {
            self.counts.catalogs.fetch_add(1, Ordering::SeqCst);
            Ok((name == "remote").then(|| {
                let provider: Arc<dyn AsyncCatalogProvider> = Arc::new(self.clone());
                provider
            }))
        })
    }
}
impl AsyncCatalogProvider for Revision {
    fn schema<'s, 'n, 'f>(
        &'s self,
        name: &'n str,
    ) -> BoxFut<'f, NativeResult<Option<Arc<dyn AsyncSchemaProvider>>>>
    where
        's: 'f,
        'n: 'f,
        Self: 'f,
    {
        Box::pin(async move {
            self.counts.schemas.fetch_add(1, Ordering::SeqCst);
            Ok((name == "data").then(|| {
                let provider: Arc<dyn AsyncSchemaProvider> = Arc::new(self.clone());
                provider
            }))
        })
    }
}
impl AsyncSchemaProvider for Revision {
    fn table<'s, 'n, 'f>(
        &'s self,
        name: &'n str,
    ) -> BoxFut<'f, NativeResult<Option<Arc<dyn TableProvider>>>>
    where
        's: 'f,
        'n: 'f,
        Self: 'f,
    {
        Box::pin(async move {
            self.counts.tables.fetch_add(1, Ordering::SeqCst);
            tokio::task::yield_now().await;
            if name == "failure" {
                return Err(DataFusionError::Execution("backend lookup failed".into()));
            }
            if name != "t" && name != "u" {
                return Ok(None);
            }
            let schema = Arc::new(Schema::new(vec![Field::new(
                "value",
                DataType::UInt64,
                false,
            )]));
            let batch = RecordBatch::try_new(
                Arc::clone(&schema),
                vec![Arc::new(UInt64Array::from(vec![self.value]))],
            )?;
            let provider: Arc<dyn TableProvider> =
                Arc::new(MemTable::try_new(schema, vec![vec![batch]])?);
            Ok(Some(provider))
        })
    }
}
fn session() -> EngineSession {
    pse_testkit::NativeFixture::new((64 << 20).try_into().unwrap())
        .unwrap()
        .into_factory()
        .candidate(
            BTreeMap::new(),
            Arc::new(pse_schema::RegistryBuilder::new().build().unwrap()),
            &CancellationToken::new(),
        )
        .unwrap()
}
fn reference(name: &str) -> TableReference {
    TableReference::full("remote", "data", name.to_owned())
}
async fn values(session: &EngineSession) -> Vec<u64> {
    let cancel = CancellationToken::new();
    session
        .prepare_sql(
            "SELECT value FROM remote.data.t UNION ALL SELECT value FROM remote.data.u",
            &cancel,
        )
        .await
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap()
        .batches()
        .iter()
        .flat_map(|batch| {
            batch
                .batch()
                .column(0)
                .as_any()
                .downcast_ref::<UInt64Array>()
                .unwrap()
                .values()
                .to_vec()
        })
        .collect()
}
#[tokio::test]
async fn resolution_deduplicates_negative_lookups_and_distinguishes_missing_coverage() {
    let source = Arc::new(Source::default());
    let session = session();
    let cancel = CancellationToken::new();
    let request = ResolutionRequest::References(vec![
        reference("t"),
        reference("t"),
        reference("missing"),
        reference("missing"),
    ]);
    drop(
        session
            .prepare_catalog_resolution(
                source.clone(),
                request.clone(),
                ResolutionConsistency::Snapshot,
                &cancel,
            )
            .unwrap(),
    );
    assert_eq!(source.counts.opens.load(Ordering::SeqCst), 0);
    let resolved = session
        .prepare_catalog_resolution(
            source.clone(),
            request,
            ResolutionConsistency::Snapshot,
            &cancel,
        )
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    assert_eq!(source.counts.opens.load(Ordering::SeqCst), 1);
    assert_eq!(source.counts.catalogs.load(Ordering::SeqCst), 1);
    assert_eq!(source.counts.schemas.load(Ordering::SeqCst), 1);
    assert_eq!(source.counts.tables.load(Ordering::SeqCst), 2);
    let covered = resolved
        .prepare_sql("SELECT * FROM remote.data.missing", &cancel)
        .await
        .unwrap_err();
    assert!(!format!("{covered:?}").contains("outside completed"));
    let unknown = resolved
        .prepare_sql("SELECT * FROM remote.data.u", &cancel)
        .await
        .unwrap_err();
    assert!(format!("{unknown:?}").contains("outside completed"));
    assert_eq!(source.counts.tables.load(Ordering::SeqCst), 2);
}
#[tokio::test]
async fn exhaustive_resolution_retains_actual_revision_after_backend_changes() {
    let source = Arc::new(Source::default());
    let session = session();
    let cancel = CancellationToken::new();
    let original = session
        .prepare_catalog_resolution(
            source.clone(),
            ResolutionRequest::All,
            ResolutionConsistency::Snapshot,
            &cancel,
        )
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    source.value.store(99, Ordering::SeqCst);
    let later = session
        .prepare_catalog_resolution(
            source.clone(),
            ResolutionRequest::All,
            ResolutionConsistency::Snapshot,
            &cancel,
        )
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    assert_eq!(values(&original).await, [7, 7]);
    assert_eq!(values(&later).await, [99, 99]);
    assert_eq!(source.counts.tables.load(Ordering::SeqCst), 4);
}
#[tokio::test]
async fn backend_failure_cancellation_and_unavailable_revision_never_publish_partial_bindings() {
    let session = session();
    let cancel = CancellationToken::new();
    let request = ResolutionRequest::References(vec![reference("t"), reference("failure")]);
    let error = session
        .prepare_catalog_resolution(
            Arc::new(Source::default()),
            request,
            ResolutionConsistency::Snapshot,
            &cancel,
        )
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap_err();
    assert!(format!("{error:?}").contains("backend lookup failed"));
    let source = Arc::new(Source {
        supports_snapshot: false,
        ..Source::default()
    });
    assert!(
        session
            .prepare_catalog_resolution(
                source.clone(),
                ResolutionRequest::All,
                ResolutionConsistency::Snapshot,
                &cancel
            )
            .unwrap()
            .execute(&cancel)
            .await
            .is_err()
    );
    let observed = session
        .prepare_catalog_resolution(
            source,
            ResolutionRequest::All,
            ResolutionConsistency::Observation,
            &cancel,
        )
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    assert_eq!(values(&observed).await, [7, 7]);
    let pending = session
        .prepare_catalog_resolution(
            Arc::new(Source {
                pending: true,
                ..Source::default()
            }),
            ResolutionRequest::All,
            ResolutionConsistency::Snapshot,
            &cancel,
        )
        .unwrap();
    let cancellation = cancel.clone();
    tokio::spawn(async move {
        tokio::task::yield_now().await;
        cancellation.cancel();
    });
    let error = tokio::time::timeout(std::time::Duration::from_secs(2), pending.execute(&cancel))
        .await
        .unwrap()
        .unwrap_err();
    assert_eq!(
        pse_diagnostics::TypedDiagnostic::diagnostic_code(&error),
        Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled)
    );
}
