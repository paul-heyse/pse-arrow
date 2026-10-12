// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Live selected permission belongs to an explicit consumer, independently of pure caches.
use super::*;
use crate::math::modeling::PreparedBasis;
use std::{
    future::Future,
    sync::atomic::{AtomicU8, AtomicUsize, Ordering},
    time::Duration,
};

const ESTABLISHING: u8 = 0;
const ACTIVE: u8 = 1;
const CLOSING: u8 = 2;
const RELEASED: u8 = 3;
const HEARTBEAT: Duration = Duration::from_secs(10);
// Source read protection is independent of the durable attempt's execution lease.
const SOURCE_LEASE: Duration = Duration::from_secs(3600);

/// One exact settled preparation and its actual live source permission. Its lane
/// explicitly closes it after dispatched work drains; it is never a cache entry.
#[derive(Debug)]
pub(in crate::workflow) struct PreparationEngagement {
    source: ModelingPackage,
    root: DeclarationId,
    instance: InstanceId,
    bindings: Bindings,
    limits: Limits,
    selected: canonical::CheckedSelection,
    model: ModelingPreparation,
    _basis: Arc<PreparedBasis>,
    _owner: Arc<pse_columnar::AllocationLease>,
    state: AtomicU8,
    active: AtomicUsize,
    drained: tokio::sync::Notify,
}

struct Admission(Arc<PreparationEngagement>);
impl Drop for Admission {
    fn drop(&mut self) {
        if self.0.active.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.0.drained.notify_waiters();
        }
    }
}
impl PreparationEngagement {
    pub(in crate::workflow) fn model(&self) -> &ModelingPreparation {
        &self.model
    }
    /// Complete source/context ownership is retained, so local identity comparisons
    /// cannot accidentally match a later allocation that reused an address.
    pub(in crate::workflow) fn matches(
        &self,
        source: &ModelingPackage,
        root: DeclarationId,
        instance: InstanceId,
        bindings: &Bindings,
        limits: Limits,
    ) -> bool {
        self.state.load(Ordering::Acquire) == ACTIVE
            && same_source(&self.source, source)
            && self.root == root
            && self.instance == instance
            && &self.bindings == bindings
            && self.limits == limits
    }
    async fn admit(self: &Arc<Self>) -> Result<Admission, WorkflowError> {
        if self.state.load(Ordering::Acquire) != ACTIVE {
            return Err(contract("preparation engagement is closing or released"));
        }
        self.active.fetch_add(1, Ordering::AcqRel);
        let admission = Admission(self.clone());
        if self.state.load(Ordering::Acquire) != ACTIVE {
            return Err(contract("preparation engagement closed during admission"));
        }
        if let Err(error) = self.selected.read.selection().renew(SOURCE_LEASE).await {
            self.state.store(CLOSING, Ordering::Release);
            return Err(error.into());
        }
        if self.state.load(Ordering::Acquire) != ACTIVE {
            return Err(contract("preparation engagement closed during renewal"));
        }
        Ok(admission)
    }
    /// Join renewal with the actual operation. On lease loss, stop new admission,
    /// signal this operation's child cancellation and await its private/native drain.
    /// Every operation retains its own clock; renewal changes no issued effect expiry.
    #[cfg(test)]
    pub(in crate::workflow) async fn during<T>(
        self: &Arc<Self>,
        cancel: &crate::CancelSource,
        operation: impl Future<Output = Result<T, WorkflowError>>,
    ) -> Result<T, WorkflowError> {
        if cancel.token().is_cancelled() {
            return Err(crate::math::MathRuntimeError::Cancelled.into());
        }
        let _admission = self.admit().await?;
        tokio::pin!(operation);
        loop {
            tokio::select! {
                result=&mut operation => return result,
                ()=tokio::time::sleep(HEARTBEAT) => {
                    if let Err(error)=self.selected.read.selection().renew(SOURCE_LEASE).await {
                        self.state.store(CLOSING, Ordering::Release);
                        cancel.cancel();
                        let _=operation.await;
                        return Err(error.into());
                    }
                }
            }
        }
    }
    /// Stop admission, await actual in-flight consumer drain, then release this
    /// exact pin. No detached renewal or release task owns this lifecycle.
    pub(in crate::workflow) async fn close(&self) -> Result<(), WorkflowError> {
        if self.state.load(Ordering::Acquire) == RELEASED {
            return Ok(());
        }
        self.state.store(CLOSING, Ordering::Release);
        loop {
            let notified = self.drained.notified();
            if self.active.load(Ordering::Acquire) == 0 {
                break;
            }
            notified.await;
        }
        self.source
            .runtime
            .canonical
            .store()
            .release(self.selected.read.selection())
            .await?;
        self.state.store(RELEASED, Ordering::Release);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn live_engagement_settles_once_and_retains_actual_owners_through_cache_pressure() {
        let runtime = crate::workflow::tests::runtime();
        let rows = pse_authoring::language::parse(
            "package p { def Root { var x:Scalar; eq a:x*x==1; eq b:x==2; } }",
            SemanticId::from_bytes([41; 16]),
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let instance = pse_modeling::specialize::root_instance(root);
        let package = runtime
            .modeling_package(rows, crate::workflow::tests::physical())
            .await
            .unwrap();
        let math = runtime.shared.math();
        let before = math.preparation_effects();
        let cancel = crate::CancelSource::new();
        let engagement = package
            .engage(
                root,
                instance,
                Bindings::default(),
                Limits::default(),
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(math.preparation_effects().since(before).settlements, 1);
        let original = engagement.model().compiled().model.allocation_identity();
        math.clear_program_cache();
        for _ in 0..20 {
            assert!(engagement.matches(
                &package,
                root,
                instance,
                &Bindings::default(),
                Limits::default()
            ));
            let phase = cancel.child_with_deadline(None);
            let actual = engagement
                .during(&phase, async {
                    Ok(engagement.model().compiled().model.allocation_identity())
                })
                .await
                .unwrap();
            assert_eq!(actual, original);
        }
        assert_eq!(
            math.preparation_effects().since(before).settlements,
            1,
            "live use never repeats B-body acknowledgment or receiving establishment"
        );
        let mut foreign = package.clone();
        foreign.providers = Arc::new((*package.providers).clone());
        assert!(
            !engagement.matches(
                &foreign,
                root,
                instance,
                &Bindings::default(),
                Limits::default()
            ),
            "equal visible provider values do not replace the actual retained context owner"
        );
        engagement.close().await.unwrap();
        assert!(!engagement.matches(
            &package,
            root,
            instance,
            &Bindings::default(),
            Limits::default()
        ));
        assert!(engagement.during(&cancel, async { Ok(()) }).await.is_err());
        engagement.close().await.unwrap();
    }
    async fn held_fixture(runtime: Runtime) -> (ModelingPackage, Arc<HeldPreparationScope>) {
        let rows = pse_authoring::language::parse(
            "package p { def Root { var x:Scalar; eq a:x*x==1; eq b:x==2; } }",
            SemanticId::from_bytes([43; 16]),
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(rows, crate::workflow::tests::physical())
            .await
            .unwrap();
        let scope = HeldPreparationScope::new();
        let package = scope.package(&package);
        package
            .prepare(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        (package, scope)
    }
    #[tokio::test]
    async fn closed_scope_releases_engagement_payloads_while_source_package_is_retained() {
        let (retained_source, scope) = held_fixture(crate::workflow::tests::runtime()).await;
        let weak = Arc::downgrade(&scope.entries.lock().await.views[0]);
        scope.close().await.unwrap();
        assert!(
            retained_source.held_preparation.is_some(),
            "reports may retain their source package"
        );
        assert!(
            weak.upgrade().is_none(),
            "closed source scope retains no engagement-only payload"
        );
        let entries = scope.entries.lock().await;
        assert!(entries.views.is_empty() && entries.owners.is_empty());
    }
    #[cfg(all(feature = "canonical-tests", feature = "solver-diffsol"))]
    #[tokio::test]
    async fn joined_preentry_refusals_abandon_and_drain_actual_started_attempts() {
        use crate::workflow::{Durability, durable::DurableAttempt};
        for refusal in 0..3 {
            let runtime = crate::workflow::durable_tests::durable_runtime();
            let (package, scope) = held_fixture(runtime.clone()).await;
            let Durability::Durable(operations) = runtime.durability() else {
                panic!("durable fixture");
            };
            let mut attempt = DurableAttempt::new(operations).unwrap();
            attempt
                .register_horizon(
                    pse_operations::mint_id(),
                    &runtime,
                    std::slice::from_ref(&package),
                    pse_ids::ContentHash::from_bytes([refusal; 32]),
                    b"{}",
                )
                .await
                .unwrap();
            let cancel = crate::CancelSource::new();
            let stop = cancel.clone();
            attempt
                .start(Arc::new(move || stop.cancel()))
                .await
                .unwrap();
            let pool = runtime.shared.pool();
            let filler = if refusal == 1 {
                // Pressure may evict idle compiler/cache allocations. Remove those
                // before filling the pool so this fixture actually refuses entry.
                runtime.shared.math().clear_program_cache();
                let filler = pse_columnar::MemoryConsumer::new("test:held-preentry-pressure")
                    .register(&pool);
                filler
                    .try_grow(runtime.shared.budget().memory_limit_bytes.get() - pool.reserved())
                    .unwrap();
                Some(filler)
            } else {
                None
            };
            if refusal == 0 {
                cancel.cancel();
            }
            if refusal == 2 {
                let view = scope.entries.lock().await.views[0].clone();
                runtime
                    .canonical_store()
                    .release(view.selected.read.selection())
                    .await
                    .unwrap();
            }
            let (record, source_refusal) = scope
                .joined(&cancel, async {
                    assert!(cancel.token().is_cancelled(), "preentry refusal {refusal}");
                    drop(filler);
                    attempt
                        .abandon(&Arc::new(WorkflowError::from(
                            crate::math::MathRuntimeError::Cancelled,
                        )))
                        .await
                })
                .await;
            assert!(
                source_refusal.is_some(),
                "actual cancellation/pool/released-pin refusal is observed"
            );
            assert!(
                record.attempt.is_ok() && record.completion.is_some(),
                "original claim settlement joins stream and heartbeat: {record:?}"
            );
            scope.close().await.unwrap();
        }
    }

    #[cfg(all(feature = "canonical-tests", feature = "solver-diffsol"))]
    #[tokio::test]
    async fn joined_source_renewal_spans_actual_completion_export_and_drain() {
        use crate::workflow::{Durability, durable::DurableAttempt};
        let runtime = crate::workflow::durable_tests::durable_runtime();
        let (package, scope) = held_fixture(runtime.clone()).await;
        let Durability::Durable(operations) = runtime.durability() else {
            panic!("durable fixture");
        };
        let mut attempt = DurableAttempt::new(operations).unwrap();
        attempt
            .register_horizon(
                pse_operations::mint_id(),
                &runtime,
                std::slice::from_ref(&package),
                pse_ids::ContentHash::from_bytes([44; 32]),
                b"{}",
            )
            .await
            .unwrap();
        let cancel = crate::CancelSource::new();
        let stop = cancel.clone();
        attempt
            .start(Arc::new(move || stop.cancel()))
            .await
            .unwrap();
        let (reached, release) = attempt.pause_completion_export();
        let view = scope.entries.lock().await.views[0].clone();
        let operation = scope.joined(&cancel, async {
            attempt
                .abandon(&Arc::new(contract(
                    "test:completion export before source loss",
                )))
                .await
        });
        let observer = async {
            reached.await.unwrap();
            assert_eq!(
                view.active.load(Ordering::Acquire),
                1,
                "source admission still belongs to the actual final publication owner"
            );
            runtime
                .canonical_store()
                .release(view.selected.read.selection())
                .await
                .unwrap();
            tokio::time::timeout(HEARTBEAT * 2, cancel.cancelled())
                .await
                .unwrap();
            assert_eq!(
                view.state.load(Ordering::Acquire),
                CLOSING,
                "joined source heartbeat observes the exact released pin during publication"
            );
            let close = scope.close();
            tokio::pin!(close);
            assert!(
                tokio::time::timeout(Duration::from_millis(20), &mut close)
                    .await
                    .is_err(),
                "source close joins actual publication rather than releasing its owner early"
            );
            release.send(()).unwrap();
            close.await.unwrap();
        };
        let ((record, refusal), ()) = tokio::join!(operation, observer);
        assert!(
            refusal.is_some(),
            "publication-time source loss closes further admission"
        );
        assert!(
            record.attempt.is_ok() && record.completion.is_some(),
            "the original rooted completion and heartbeat are drained: {record:?}"
        );
        assert_eq!(view.active.load(Ordering::Acquire), 0);
        assert!(scope.entries.lock().await.views.is_empty());
    }
}
impl ModelingPackage {
    pub(in crate::workflow) async fn engage(
        &self,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        cancel: &crate::CancelSource,
    ) -> Result<Arc<PreparationEngagement>, WorkflowError> {
        let activity = self.runtime.canonical_store().admit_activity()?;
        activity
            .scope(async {
                let state = AtomicU8::new(ESTABLISHING);
                let deadline = preparation_deadline(cancel)?;
                let mut selected = self
                    .checked_selection_scoped(&[root], cancel, deadline)
                    .await?;
                let phase = cancel.child_with_deadline(Some(deadline));
                let result = async {
                    let bytes = size_of::<PreparationEngagement>()
                        + crate::math::modeling::bindings_bytes(&bindings)
                        + 1024;
                    let owner = self
                        .runtime
                        .shared
                        .math()
                        .reserve("modeling:live-engagement", bytes)?;
                    let (model, basis) = self
                        .prepare_selected_basis(
                            &mut selected,
                            root,
                            instance,
                            bindings.clone(),
                            limits,
                            &phase,
                            deadline,
                        )
                        .await?;
                    Ok::<_, WorkflowError>((model, basis, owner))
                }
                .await;
                let (model, basis, owner) = match result {
                    Ok(value) => value,
                    Err(error) => {
                        state.store(CLOSING, Ordering::Release);
                        let release = self
                            .runtime
                            .canonical
                            .store()
                            .release(selected.read.selection())
                            .await;
                        release?;
                        return Err(error);
                    }
                };
                state.store(ACTIVE, Ordering::Release);
                let mut source = self.clone();
                source.held_preparation = None;
                Ok(Arc::new(PreparationEngagement {
                    source,
                    root,
                    instance,
                    bindings,
                    limits,
                    selected,
                    model,
                    _basis: basis,
                    _owner: owner,
                    state,
                    active: AtomicUsize::new(0),
                    drained: tokio::sync::Notify::new(),
                }))
            })
            .await
    }
}

/// An explicit lane's finite set of settled views. Membership belongs to the
/// operation admission; this owner retains only actual source permissions.
#[derive(Debug)]
pub(in crate::workflow) struct HeldPreparationScope {
    entries: tokio::sync::Mutex<HeldEntries>,
    close_serial: tokio::sync::Mutex<()>,
    state: AtomicU8,
}
#[derive(Debug, Default)]
struct HeldEntries {
    views: Vec<Arc<PreparationEngagement>>,
    owners: Vec<Arc<pse_columnar::AllocationLease>>,
}
impl HeldPreparationScope {
    pub(in crate::workflow) fn new() -> Arc<Self> {
        Arc::new(Self {
            entries: tokio::sync::Mutex::new(HeldEntries::default()),
            close_serial: tokio::sync::Mutex::new(()),
            state: AtomicU8::new(ESTABLISHING),
        })
    }
    pub(in crate::workflow) fn package(
        self: &Arc<Self>,
        source: &ModelingPackage,
    ) -> ModelingPackage {
        let mut source = source.clone();
        source.held_preparation = Some(self.clone());
        source
    }
    pub(in crate::workflow) fn is_open(&self) -> bool {
        matches!(self.state.load(Ordering::Acquire), ESTABLISHING | ACTIVE)
    }
    pub(in crate::workflow) fn freeze(&self) -> Result<(), WorkflowError> {
        self.state
            .compare_exchange(ESTABLISHING, ACTIVE, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| ())
            .or_else(|state| {
                if state == ACTIVE {
                    Ok(())
                } else {
                    Err(contract("held preparation scope is closing"))
                }
            })
    }
    pub(in crate::workflow) async fn prepare(
        &self,
        source: &ModelingPackage,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingPreparation, WorkflowError> {
        let mut entries = self.entries.lock().await;
        let state = self.state.load(Ordering::Acquire);
        if state != ESTABLISHING && state != ACTIVE {
            return Err(contract("held preparation scope is closing"));
        }
        for view in &entries.views {
            // Earlier views remain protected throughout multi-view establishment.
            let _admission = view.admit().await?;
        }
        if let Some(view) = entries
            .views
            .iter()
            .find(|view| view.matches(source, root, instance, &bindings, limits))
        {
            return Ok(view.model().clone());
        }
        if state != ESTABLISHING {
            return Err(contract(
                "held preparation differs from the admitted lane request",
            ));
        }
        let owner = source.runtime.shared.math().reserve(
            "modeling:held-view-entry",
            size_of::<Arc<PreparationEngagement>>()
                + size_of::<Arc<pse_columnar::AllocationLease>>()
                + 128,
        )?;
        let view = source
            .engage(root, instance, bindings, limits, cancel)
            .await?;
        let model = view.model().clone();
        entries.views.push(view);
        entries.owners.push(owner);
        Ok(model)
    }
    pub(in crate::workflow) async fn renew(&self) -> Result<(), WorkflowError> {
        if !self.is_open() {
            return Err(contract("held preparation scope is closing"));
        }
        let entries = self.entries.lock().await;
        for view in &entries.views {
            if let Err(error) = view.admit().await {
                self.state.store(CLOSING, Ordering::Release);
                return Err(error);
            }
        }
        Ok(())
    }
    /// Cleanup always owns its future, including preentry cancellation, memory
    /// refusal and renewal refusal. No issued attempt is discarded unpolled.
    pub(in crate::workflow) async fn joined<T>(
        &self,
        cancel: &crate::CancelSource,
        operation: impl Future<Output = T>,
    ) -> (T, Option<WorkflowError>) {
        let admitted = async {
            if cancel.token().is_cancelled() {
                return Err(crate::math::MathRuntimeError::Cancelled.into());
            }
            self.freeze()?;
            let entries = self.entries.lock().await;
            let owner = match entries.views.first() {
                Some(view) => Some(view.source.runtime.shared.math().reserve(
                    "modeling:held-operation-views",
                    entries.views.len()
                        * (size_of::<Arc<PreparationEngagement>>() + size_of::<Admission>())
                        + 128,
                )?),
                None => None,
            };
            let views = entries.views.clone();
            drop(entries);
            let mut admissions = Vec::with_capacity(views.len());
            for view in &views {
                admissions.push(view.admit().await?);
            }
            Ok::<_, WorkflowError>((views, admissions, owner))
        }
        .await;
        let (views, _admissions, _owner) = match admitted {
            Ok(admitted) => admitted,
            Err(error) => {
                cancel.cancel();
                return (operation.await, Some(error));
            }
        };
        tokio::pin!(operation);
        loop {
            tokio::select! {
                result = &mut operation => return (result, None),
                () = tokio::time::sleep(HEARTBEAT) => {
                    for view in &views {
                        if let Err(error) = view.selected.read.selection().renew(SOURCE_LEASE).await {
                            view.state.store(CLOSING, Ordering::Release);
                            self.state.store(CLOSING, Ordering::Release);
                            cancel.cancel();
                            return (operation.await, Some(error.into()));
                        }
                    }
                }
            }
        }
    }
    pub(in crate::workflow) async fn during<T>(
        &self,
        cancel: &crate::CancelSource,
        operation: impl Future<Output = Result<T, WorkflowError>>,
    ) -> Result<T, WorkflowError> {
        let (result, renewal) = self.joined(cancel, operation).await;
        match renewal {
            Some(error) => Err(error),
            None => result,
        }
    }
    pub(in crate::workflow) async fn close(&self) -> Result<(), WorkflowError> {
        let _serial = self.close_serial.lock().await;
        if self.state.load(Ordering::Acquire) == RELEASED {
            return Ok(());
        }
        self.state.store(CLOSING, Ordering::Release);
        // Move the existing allocation. Release must remain available under pool
        // pressure, and the contents lock must not block an operation's drain.
        let mut detached = {
            let mut entries = self.entries.lock().await;
            std::mem::take(&mut *entries)
        };
        let mut failure = None;
        for view in &detached.views {
            if let Err(error) = view.close().await
                && failure.is_none()
            {
                failure = Some(error);
            }
        }
        // Released views must not remain reachable through report source packages.
        // On uncertainty the old container allocation still needs its metadata charge;
        // only exact unreleased pin owners remain in that container.
        detached
            .views
            .retain(|view| view.state.load(Ordering::Acquire) != RELEASED);
        if detached.views.is_empty() {
            detached = HeldEntries::default();
        }
        *self.entries.lock().await = detached;
        if let Some(error) = failure {
            return Err(error);
        }
        self.state.store(RELEASED, Ordering::Release);
        Ok(())
    }
}

/// Captured at actual construction, never inferred from a prepared model hash.
#[cfg(feature = "solver-diffsol")]
#[derive(Debug)]
pub(in crate::workflow) struct OriginalPreparationInputs {
    pub(in crate::workflow) root: DeclarationId,
    pub(in crate::workflow) instance: InstanceId,
    pub(in crate::workflow) bindings: Bindings,
    pub(in crate::workflow) limits: Limits,
    _owner: Arc<pse_columnar::AllocationLease>,
}
#[cfg(feature = "solver-diffsol")]
impl OriginalPreparationInputs {
    pub(in crate::workflow) fn capture(
        source: &ModelingPackage,
        root: DeclarationId,
        instance: InstanceId,
        bindings: &Bindings,
        limits: Limits,
    ) -> Result<Arc<Self>, WorkflowError> {
        let owner = source.runtime.shared.math().reserve(
            "modeling:original-preparation-inputs",
            size_of::<Self>() + crate::math::modeling::bindings_bytes(bindings) + 128,
        )?;
        Ok(Arc::new(Self {
            root,
            instance,
            bindings: bindings.clone(),
            limits,
            _owner: owner,
        }))
    }
}

pub(in crate::workflow) fn same_source(
    source: &ModelingPackage,
    current: &ModelingPackage,
) -> bool {
    source.revision.canonical == current.revision.canonical
        && source.revision.identity() == current.revision.identity()
        && Arc::ptr_eq(&source.runtime.shared, &current.runtime.shared)
        && Arc::ptr_eq(source.runtime.sessions(), current.runtime.sessions())
        && Arc::ptr_eq(source.runtime.registry(), current.runtime.registry())
        && source.physical.key == current.physical.key
        && Arc::ptr_eq(&source.physical.quantities, &current.physical.quantities)
        && Arc::ptr_eq(
            &source.physical.preconditions,
            &current.physical.preconditions,
        )
        && Arc::ptr_eq(&source.providers, &current.providers)
        && Arc::ptr_eq(&source.accelerators, &current.accelerators)
}
