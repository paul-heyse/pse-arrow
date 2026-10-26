// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Held local preparation owns exact source/context and joins its actual consumers.
use super::*;
use crate::math::modeling::PreparedBasis;
use std::{
    future::Future,
    sync::atomic::{AtomicU8, AtomicUsize, Ordering},
};

const ESTABLISHING: u8 = 0;
const ACTIVE: u8 = 1;
const CLOSING: u8 = 2;
const RELEASED: u8 = 3;
/// One exact locally admitted preparation. Its lane closes admission and joins
/// actual consumers; immutable products retain their own source and byte owners.
#[derive(Debug)]
pub(in crate::workflow) struct PreparationEngagement {
    source: ModelingPackage,
    root: DeclarationId,
    instance: InstanceId,
    bindings: Bindings,
    limits: Limits,
    _selected: canonical::CheckedSelection,
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
        services: &crate::workflow::PreparationServices,
        source: &ModelingPackage,
        root: DeclarationId,
        instance: InstanceId,
        bindings: &Bindings,
        limits: Limits,
    ) -> bool {
        self.state.load(Ordering::Acquire) == ACTIVE
            && self._selected.carries(source, services)
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
        Ok(admission)
    }
    /// Hold exact source/product ownership through an admitted operation.
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
        operation.await
    }
    /// Stop admission, await actual in-flight consumer drain, then release this
    /// held owner. No detached renewal or release task owns this lifecycle.
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
        self.state.store(RELEASED, Ordering::Release);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn local_engagement_retains_actual_owners_through_cache_pressure() {
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
        let cancel = crate::CancelSource::new();
        let engagement = package
            .engage(
                &runtime.preparation_services(),
                root,
                instance,
                Bindings::default(),
                Limits::default(),
                &cancel,
            )
            .await
            .unwrap();
        let original = engagement.model().compiled().model.allocation_identity();
        math.clear_program_cache();
        for _ in 0..20 {
            assert!(engagement.matches(
                &runtime.preparation_services(),
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
        let mut foreign = package.clone();
        foreign.providers = Arc::new((*package.providers).clone());
        assert!(
            !engagement.matches(
                &runtime.preparation_services(),
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
            &runtime.preparation_services(),
            &package,
            root,
            instance,
            &Bindings::default(),
            Limits::default()
        ));
        assert!(engagement.during(&cancel, async { Ok(()) }).await.is_err());
        engagement.close().await.unwrap();
    }
    async fn held_fixture(runtime: &Runtime) -> (ModelingPackage, Arc<HeldPreparationScope>) {
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
                &runtime.preparation_services(),
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
    #[tokio::test(flavor = "current_thread")]
    async fn held_preparation_reuse_preserves_cancelled_and_expired_caller_clocks() {
        let runtime = crate::workflow::tests::runtime();
        let (package, scope) = held_fixture(&runtime).await;
        let (root, instance, original) = {
            let entries = scope.entries.lock().await;
            let view = &entries.views[0];
            (
                view.root,
                view.instance,
                view.model.compiled().model.allocation_identity(),
            )
        };
        let cancelled = crate::CancelSource::new();
        cancelled.cancel();
        assert!(matches!(
            package
                .prepare(
                    &runtime.preparation_services(),
                    root,
                    instance,
                    Bindings::default(),
                    Limits::default(),
                    &cancelled
                )
                .await,
            Err(WorkflowError::Math(
                crate::math::MathRuntimeError::Cancelled
            ))
        ));
        let expired = crate::CancelSource::new().with_deadline(Some(
            std::time::Instant::now() - std::time::Duration::from_millis(1),
        ));
        assert!(matches!(
            package
                .prepare(
                    &runtime.preparation_services(),
                    root,
                    instance,
                    Bindings::default(),
                    Limits::default(),
                    &expired
                )
                .await,
            Err(WorkflowError::Math(crate::math::MathRuntimeError::Solve(
                pse_backend_native::ProblemError::Limit {
                    kind: pse_backend_native::LimitKind::Time,
                    ..
                }
            )))
        ));
        assert!(!expired.token().is_cancelled());
        let caller = crate::CancelSource::new();
        let guard = scope.entries.lock().await;
        let preparation_services = runtime.preparation_services();
        let mut queued = Box::pin(package.prepare(
            &preparation_services,
            root,
            instance,
            Bindings::default(),
            Limits::default(),
            &caller,
        ));
        assert!(futures_util::poll!(queued.as_mut()).is_pending());
        caller.cancel();
        assert!(matches!(
            queued.await,
            Err(WorkflowError::Math(
                crate::math::MathRuntimeError::Cancelled
            ))
        ));
        drop(guard);
        let valid = package
            .prepare(
                &runtime.preparation_services(),
                root,
                instance,
                Bindings::default(),
                Limits::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert_eq!(valid.compiled().model.allocation_identity(), original);
        scope.close().await.unwrap();
    }
    #[tokio::test]
    async fn closed_scope_releases_engagement_payloads_while_source_package_is_retained() {
        let runtime = crate::workflow::tests::runtime();
        let (retained_source, scope) = held_fixture(&runtime).await;
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
    #[tokio::test(flavor = "current_thread")]
    async fn public_preparation_parent_cancellation_drains_queued_specialization() {
        let runtime = crate::workflow::tests::runtime();
        let (mut package, scope) = held_fixture(&runtime).await;
        let (root, instance) = {
            let entries = scope.entries.lock().await;
            (entries.views[0].root, entries.views[0].instance)
        };
        scope.close().await.unwrap();
        package.held_preparation = None;
        let service = runtime.shared.math();
        service.clear_program_cache();
        // Establish source admission before occupying CPU, so the issued public
        // request reaches the actual specialization producer rather than selection.
        drop(
            package
                .checked_selection(
                    &runtime.preparation_services(),
                    &[root],
                    &crate::CancelSource::new(),
                )
                .await
                .unwrap(),
        );
        let before = service.preparation_effects();
        let (started, entered) = tokio::sync::oneshot::channel();
        let (release, gate) = std::sync::mpsc::channel();
        let mut submission = crate::math::Submission::ephemeral();
        submission.admitted = Some(started);
        let blocker = service
            .submit_with(service.cores(), 0, submission, move |_, _| {
                gate.recv_timeout(std::time::Duration::from_secs(10))
                    .map_err(|error| {
                        crate::math::MathRuntimeError::Infrastructure(error.to_string())
                    })?;
                Ok(((), 0))
            })
            .unwrap();
        entered.await.unwrap();
        let available = service.test_admission_snapshot().2;
        let cancel = crate::CancelSource::new();
        let preparation_services = runtime.preparation_services();
        let mut preparation = Box::pin(package.prepare(
            &preparation_services,
            root,
            instance,
            Bindings::default(),
            Limits::default(),
            &cancel,
        ));
        let queued = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                assert!(futures_util::poll!(preparation.as_mut()).is_pending());
                if service.frontier_flights.active() == 1 {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await;
        cancel.cancel();
        let outcome =
            tokio::time::timeout(std::time::Duration::from_secs(5), preparation.as_mut()).await;
        drop(preparation);
        let drained = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while service.frontier_flights.active() != 0 {
                tokio::task::yield_now().await;
            }
        })
        .await;
        let remaining = service.test_admission_snapshot().2;
        release.send(()).unwrap();
        blocker.finish().await.unwrap();
        queued.unwrap();
        assert!(matches!(
            outcome.unwrap(),
            Err(WorkflowError::Math(
                crate::math::MathRuntimeError::Cancelled
            ))
        ));
        drained.unwrap();
        assert_eq!(remaining, available);
        assert_eq!(
            service.preparation_effects().frontiers,
            before.frontiers,
            "cancellation drained the queued specialization without entering its compiler work"
        );
    }
}
impl ModelingPackage {
    pub(in crate::workflow) async fn engage(
        &self,
        services: &crate::workflow::PreparationServices,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        cancel: &crate::CancelSource,
    ) -> Result<Arc<PreparationEngagement>, WorkflowError> {
        let activity = services.admit_activity()?;
        activity
            .scope(async {
                let state = AtomicU8::new(ESTABLISHING);
                let deadline = services.math().preparation_deadline(cancel)?;
                let mut selected = self
                    .checked_selection_scoped(services, &[root], cancel, deadline)
                    .await?;
                let phase = cancel.child_with_deadline(Some(deadline));
                let result = async {
                    let bytes = size_of::<PreparationEngagement>()
                        + crate::math::modeling::bindings_bytes(&bindings)
                        + 1024;
                    let owner = services.math().reserve("modeling:live-engagement", bytes)?;
                    let (model, basis) = self
                        .prepare_selected_basis(
                            services,
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
                    _selected: selected,
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

/// An explicit lane's finite set of locally admitted views and allocation protections.
/// This lease owns admission/release state but no runtime or execution capability; every
/// effectful prepare/join operation receives its services explicitly. Retained immutable
/// products own their actual bytes independently of release.
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
        services: &crate::workflow::PreparationServices,
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
            // Earlier views remain owned throughout multi-view establishment.
            let _admission = view.admit().await?;
        }
        if let Some(view) = entries
            .views
            .iter()
            .find(|view| view.matches(services, source, root, instance, &bindings, limits))
        {
            return Ok(view.model().clone());
        }
        if state != ESTABLISHING {
            return Err(contract(
                "held preparation differs from the admitted lane request",
            ));
        }
        let owner = services.math().reserve(
            "modeling:held-view-entry",
            size_of::<Arc<PreparationEngagement>>()
                + size_of::<Arc<pse_columnar::AllocationLease>>()
                + 128,
        )?;
        let view = source
            .engage(services, root, instance, bindings, limits, cancel)
            .await?;
        let model = view.model().clone();
        entries.views.push(view);
        entries.owners.push(owner);
        Ok(model)
    }
    pub(in crate::workflow) async fn check_active(&self) -> Result<(), WorkflowError> {
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
    /// refusal and admission refusal. No issued attempt is discarded unpolled.
    pub(in crate::workflow) async fn joined<T>(
        &self,
        services: &crate::workflow::PreparationServices,
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
                Some(_) => Some(services.math().reserve(
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
        // The admission guards retain the selected local products until the
        // actual operation drains. There is no store source heartbeat to renew.
        let _views = views;
        (operation.await, None)
    }

    pub(in crate::workflow) async fn during<T>(
        &self,
        services: &crate::workflow::PreparationServices,
        cancel: &crate::CancelSource,
        operation: impl Future<Output = Result<T, WorkflowError>>,
    ) -> Result<T, WorkflowError> {
        let (result, renewal) = self.joined(services, cancel, operation).await;
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
#[derive(Debug)]
pub(crate) struct OriginalPreparationInputs {
    pub(crate) root: DeclarationId,
    pub(crate) instance: InstanceId,
    pub(crate) bindings: Bindings,
    pub(crate) limits: Limits,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl OriginalPreparationInputs {
    pub(crate) fn retained_bytes(&self) -> usize {
        self._owner.size()
    }
    pub(in crate::workflow) fn capture(
        services: &crate::workflow::PreparationServices,
        root: DeclarationId,
        instance: InstanceId,
        bindings: &Bindings,
        limits: Limits,
    ) -> Result<Arc<Self>, WorkflowError> {
        let owner = services.math().reserve(
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
    source.revision.revision_id() == current.revision.revision_id()
        && source.revision.identity() == current.revision.identity()
        && source.physical.key == current.physical.key
        && Arc::ptr_eq(&source.physical.quantities, &current.physical.quantities)
        && Arc::ptr_eq(
            &source.physical.preconditions,
            &current.physical.preconditions,
        )
        && Arc::ptr_eq(&source.providers, &current.providers)
        && Arc::ptr_eq(&source.accelerators, &current.accelerators)
}
