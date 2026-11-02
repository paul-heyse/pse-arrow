// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Synchronization inside the original library's reference evaluation and cancellation result.
use super::*;
use std::sync::atomic::{AtomicBool, AtomicU64};
use tracing::{
    Metadata, Subscriber,
    span::{Attributes, Id, Record},
};

struct ReferenceEvaluation {
    spans: Mutex<BTreeMap<u64, &'static str>>,
    next: AtomicU64,
    gate: Mutex<
        Option<(
            tokio::sync::oneshot::Sender<()>,
            std::sync::mpsc::Receiver<()>,
        )>,
    >,
    observed_native_cancellation: Arc<AtomicBool>,
}
impl Subscriber for ReferenceEvaluation {
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }
    fn new_span(&self, attrs: &Attributes<'_>) -> Id {
        let id = self.next.fetch_add(1, Ordering::SeqCst) + 1;
        self.spans
            .lock()
            .unwrap()
            .insert(id, attrs.metadata().name());
        Id::from_u64(id)
    }
    fn record(&self, _: &Id, _: &Record<'_>) {}
    fn record_follows_from(&self, _: &Id, _: &Id) {}
    fn event(&self, event: &tracing::Event<'_>) {
        if event
            .metadata()
            .fields()
            .field("native_cancellation")
            .is_some()
        {
            self.observed_native_cancellation
                .store(true, Ordering::SeqCst);
        }
    }
    fn enter(&self, id: &Id) {
        // This exact span is emitted inside upstream FindUsages::search, reached
        // from the real Analysis::find_all_refs call on its cloned database.
        let evaluating =
            self.spans.lock().unwrap().get(&id.into_u64()).copied() == Some("FindUsages:search");
        if evaluating && let Some((entered, release)) = self.gate.lock().unwrap().take() {
            entered.send(()).unwrap();
            release
                .recv_timeout(std::time::Duration::from_secs(30))
                .unwrap();
        }
    }
    fn exit(&self, _: &Id) {}
}

#[tokio::test]
async fn in_flight_reference_evaluation_observes_snapshot_hook_and_original_salsa_cancellation() {
    let (root, configuration, file) = tests::fixture();
    let engine = Engine::start(root.path(), configuration).unwrap();
    let (entered, observed) = tokio::sync::oneshot::channel();
    let (release, blocked) = std::sync::mpsc::channel();
    let native_cancelled = Arc::new(AtomicBool::new(false));
    engine.observe_next_query_dispatch(tracing::Dispatch::new(ReferenceEvaluation {
        spans: Mutex::new(BTreeMap::new()),
        next: AtomicU64::new(0),
        gate: Mutex::new(Some((entered, blocked))),
        observed_native_cancellation: native_cancelled.clone(),
    }));
    let mut selected = engine.start_query(
        tests::query(&file, "original()->", Kind::Refs),
        CancellationToken::new(),
    );
    tokio::time::timeout(std::time::Duration::from_secs(30), observed)
        .await
        .unwrap()
        .unwrap();
    let mut sibling = engine.start_query(
        tests::query(&file, "original()->", Kind::Refs),
        CancellationToken::new(),
    );
    selected.cancel();
    assert!(
        engine.active.load(Ordering::SeqCst) >= 1,
        "the blocked original native reference evaluation remains owned"
    );
    release.send(()).unwrap();
    let result = tokio::time::timeout(std::time::Duration::from_secs(30), selected.settle())
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(result, Err(Error::Cancelled)), "{result:?}");
    assert!(
        native_cancelled.load(Ordering::SeqCst),
        "the original IDE result must observe Salsa Local cancellation; an outer cancelled-token check is insufficient"
    );
    assert!(sibling.settle().await.unwrap().is_ok());
    assert!(
        engine
            .start_query(
                tests::query(&file, "original()->", Kind::Refs),
                CancellationToken::new()
            )
            .settle()
            .await
            .unwrap()
            .is_ok()
    );
    assert_eq!(engine.active.load(Ordering::SeqCst), 0);
}
