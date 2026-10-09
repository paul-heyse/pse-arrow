// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Current selected effects around independently retained pure preparation.
use super::*;
use crate::math::{modeling::PreparedBasis, portable};
use pse_operations::canonical_selection::SelectedRead;
use std::{
    future::Future,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

struct StopWatch(tokio::task::JoinHandle<()>);
impl Drop for StopWatch {
    fn drop(&mut self) {
        self.0.abort();
    }
}
fn watch(cancel: &crate::CancelSource) -> (Arc<AtomicBool>, StopWatch) {
    let flag = Arc::new(AtomicBool::new(cancel.token().is_cancelled()));
    let watched = flag.clone();
    let token = cancel.token();
    (
        flag,
        StopWatch(tokio::spawn(async move {
            token.cancelled().await;
            watched.store(true, Ordering::Release);
        })),
    )
}

fn preparation_time_limit() -> WorkflowError {
    crate::math::MathRuntimeError::Solve(pse_backend_native::ProblemError::Limit {
        kind: pse_backend_native::LimitKind::Time,
        detail: "scientific preparation deadline".into(),
    })
    .into()
}

/// Every effect and each caller's flight wait consumes the original operation clock.
/// Dropping a pure flight waiter leaves surviving consumers and completion owners intact.
async fn scoped<T>(
    cancel: &crate::CancelSource,
    deadline: Instant,
    operation: impl Future<Output = Result<T, WorkflowError>>,
) -> Result<T, WorkflowError> {
    let deadline = cancel
        .deadline()
        .map_or(deadline, |parent| parent.min(deadline));
    if cancel.token().is_cancelled() {
        return Err(crate::math::MathRuntimeError::Cancelled.into());
    }
    if Instant::now() >= deadline {
        return Err(preparation_time_limit());
    }
    tokio::select! {
        biased;
        () = cancel.cancelled() => Err(crate::math::MathRuntimeError::Cancelled.into()),
        () = tokio::time::sleep_until(deadline.into()) => Err(preparation_time_limit()),
        result = operation => result,
    }
}

/// Private receiving construction must drain before its source protection is released.
/// This differs from a shared pure flight waiter, which may detach independently.
async fn receiving<T>(
    cancel: &crate::CancelSource,
    flag: &AtomicBool,
    deadline: Instant,
    operation: impl Future<Output = Result<T, WorkflowError>>,
) -> Result<T, WorkflowError> {
    let deadline = cancel
        .deadline()
        .map_or(deadline, |parent| parent.min(deadline));
    if cancel.token().is_cancelled() {
        return Err(crate::math::MathRuntimeError::Cancelled.into());
    }
    if Instant::now() >= deadline {
        return Err(preparation_time_limit());
    }
    tokio::pin!(operation);
    tokio::select! {
        biased;
        () = cancel.cancelled() => {
            flag.store(true, Ordering::Release);
            let _ = operation.await;
            Err(crate::math::MathRuntimeError::Cancelled.into())
        },
        () = tokio::time::sleep_until(deadline.into()) => {
            flag.store(true, Ordering::Release);
            let _ = operation.await;
            Err(preparation_time_limit())
        },
        result = &mut operation => result,
    }
}

fn portable(error: portable::PortableError) -> WorkflowError {
    portable::runtime_error(error).into()
}

impl ModelingPackage {
    /// Stop selection cooperatively and await its cleanup, retaining source protection
    /// through the last bounded in-flight store request rather than abandoning a pin.
    pub(super) async fn checked_selection_scoped(
        &self,
        roots: &[DeclarationId],
        cancel: &crate::CancelSource,
        deadline: Instant,
    ) -> Result<canonical::CheckedSelection, WorkflowError> {
        let phase = cancel.child_with_deadline(Some(deadline));
        let deadline = phase.deadline().unwrap_or(deadline);
        if cancel.token().is_cancelled() {
            return Err(crate::math::MathRuntimeError::Cancelled.into());
        }
        if Instant::now() >= deadline {
            return Err(preparation_time_limit());
        }
        let operation = self.checked_selection(roots, &phase);
        tokio::pin!(operation);
        tokio::select! {
            biased;
            () = cancel.cancelled() => {
                phase.cancel();
                if let Ok(selected) = operation.await {
                    self.runtime.canonical.store().release(selected.read.selection()).await?;
                }
                Err(crate::math::MathRuntimeError::Cancelled.into())
            },
            () = tokio::time::sleep_until(deadline.into()) => {
                phase.cancel();
                if let Ok(selected) = operation.await {
                    self.runtime.canonical.store().release(selected.read.selection()).await?;
                }
                Err(preparation_time_limit())
            },
            result = &mut operation => result,
        }
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "one current consumer owns the exact selected specialization request, stop and original clock"
    )]
    pub(super) async fn prepare_selected(
        &self,
        selected: &mut canonical::CheckedSelection,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        cancel: &crate::CancelSource,
        deadline: Instant,
    ) -> Result<ModelingPreparation, WorkflowError> {
        let math = self.runtime.shared.math();
        let store = self.runtime.canonical.store();
        let context = compiler_context(&self.physical, &self.providers);
        let generation = math.modeling_cache.generation();
        let key = math.basis_key(&selected.admission, root, instance, bindings, limits)?;
        let (flag, _watch) = watch(cancel);
        let mut receiving_checked = false;
        let mut namespace = None;
        let cached = math.modeling_cache.basis(&key);
        // checked_selection already proves and merges the admission's full
        // inventory into this protected read. Only additional acquisition
        // premises require another store qualification here.
        let basis = if let Some(basis) = cached
            && (basis.acquisition_dependencies == selected.admission.dependencies
                || scoped(cancel, deadline, async {
                    Ok(store
                        .recheck_selection_dependencies(
                            &mut selected.read,
                            &basis.acquisition_dependencies,
                        )
                        .await?)
                })
                .await?)
        {
            basis
        } else {
            let workspace = self.pure_workspace()?;
            let frontier = scoped(cancel, deadline, async {
                Ok(math
                    .plan_frontier_keyed(
                        key.clone(),
                        workspace.clone(),
                        selected.admission.revision.clone(),
                        cancel,
                        deadline,
                    )
                    .await?)
            })
            .await?;
            for identity in frontier.body_requests() {
                if math.modeling_cache.body(identity).is_some() {
                    continue;
                }
                let Some(producer) = self.runtime.canonical.producer() else {
                    continue;
                };
                let present = scoped(cancel, deadline, async {
                    portable::product_candidate_presence(
                        store,
                        &selected.read,
                        producer,
                        identity,
                        &context,
                        "",
                    )
                    .await
                    .map_err(portable)
                })
                .await?;
                if !present {
                    continue;
                }
                if !receiving_checked {
                    // Native capture owns its cancellation/join; do not put it inside an
                    // async timeout that could abandon its explicit settlement wait.
                    namespace =
                        portable::qualify_replay(math, Some(producer.clone()), deadline, cancel)
                            .await?;
                    receiving_checked = true;
                }
                let Some(current) = &namespace else {
                    continue;
                };
                let received = receiving(cancel, &flag, deadline, async {
                    portable::reuse_body_current(
                        math,
                        store,
                        &mut selected.read,
                        current,
                        identity,
                        &context,
                        &flag,
                    )
                    .await
                    .map_err(portable)
                })
                .await?;
                if let Some(body) = received {
                    math.retain_received_body(generation, identity, Arc::new(body))?;
                }
            }
            let dependencies = Arc::new(selected.read.snapshot_dependencies()?);
            scoped(cancel, deadline, async {
                Ok(math
                    .complete_frontier_keyed(
                        key.clone(),
                        dependencies,
                        workspace,
                        frontier,
                        cancel,
                        deadline,
                    )
                    .await?)
            })
            .await?
        };

        let descriptions = self
            .settle_preparation(
                &basis,
                &selected.read,
                &context,
                &mut namespace,
                &mut receiving_checked,
                cancel,
                deadline,
            )
            .await?;
        let retained = Arc::new(basis.with_descriptions(math, descriptions)?);
        math.modeling_cache
            .retain_basis(generation, key, retained.clone());
        Ok(retained
            .bind(&selected.admission.revision, root, instance)?
            .with_consumed_source_versions(
                selected.admission.versions.clone(),
                selected.admission.metadata.clone(),
            ))
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "exact selected publication shares one receiving admission and original consumer stop/clock across its complete body inventory"
    )]
    async fn settle_preparation(
        &self,
        basis: &PreparedBasis,
        read: &SelectedRead,
        context: &CompilerContext,
        namespace: &mut Option<portable::ValidatedReplayNamespace>,
        receiving_checked: &mut bool,
        cancel: &crate::CancelSource,
        deadline: Instant,
    ) -> Result<Vec<Arc<portable::BodyDescription>>, WorkflowError> {
        let math = self.runtime.shared.math();
        let store = self.runtime.canonical.store();
        let mut descriptions = Vec::new();
        for body in basis.compiled().portable_bodies() {
            let identity = body
                .semantic_identity()
                .ok_or_else(|| contract("portable body is not sealed"))?;
            if let Some(old) = basis.description(identity)
                && old.description.matches_read(read)?
                && scoped(cancel, deadline, async {
                    Ok(store
                        .acknowledge_description_until(&old.description, deadline.into())
                        .await?
                        .is_some())
                })
                .await?
            {
                descriptions.push(old.clone());
                continue;
            }
            if !*receiving_checked {
                *namespace = portable::qualify_replay(
                    math,
                    self.runtime.canonical.producer().cloned(),
                    deadline,
                    cancel,
                )
                .await?;
                *receiving_checked = true;
            }
            let producer = namespace.as_ref().map_or_else(
                || {
                    format!(
                        "pse.unqualified-producer.v1:{}",
                        self.runtime.canonical.attestation().build.to_hex()
                    )
                },
                |current| current.producer_key().to_owned(),
            );
            let description = scoped(cancel, deadline, async {
                portable::describe_body(math, read, producer, body, context).map_err(portable)
            })
            .await?;
            scoped(cancel, deadline, async {
                portable::publish_description(store, read, &description, deadline.into())
                    .await
                    .map_err(portable)
            })
            .await?;
            descriptions.push(description);
        }
        Ok(descriptions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn preparation_effects_preserve_original_deadline_and_do_not_poll_after_expiry() {
        let driver = crate::CancelSource::new();
        let polled = AtomicBool::new(false);
        let deadline = Instant::now() - std::time::Duration::from_millis(1);
        let result = scoped(&driver, deadline, async {
            polled.store(true, Ordering::Relaxed);
            Ok(())
        })
        .await;
        assert!(matches!(
            result,
            Err(WorkflowError::Math(crate::math::MathRuntimeError::Solve(
                pse_backend_native::ProblemError::Limit {
                    kind: pse_backend_native::LimitKind::Time,
                    ..
                }
            )))
        ));
        assert!(!polled.load(Ordering::Relaxed));
    }
    #[tokio::test]
    async fn preparation_receiving_effect_inherits_expired_driver_clock_before_polling() {
        let driver = crate::CancelSource::new()
            .child_with_deadline(Some(Instant::now() - std::time::Duration::from_millis(1)));
        let polled = AtomicBool::new(false);
        let stop = AtomicBool::new(false);
        let result = receiving(
            &driver,
            &stop,
            Instant::now() + std::time::Duration::from_secs(10),
            async {
                polled.store(true, Ordering::Relaxed);
                Ok(())
            },
        )
        .await;
        assert!(matches!(
            result,
            Err(WorkflowError::Math(crate::math::MathRuntimeError::Solve(
                pse_backend_native::ProblemError::Limit {
                    kind: pse_backend_native::LimitKind::Time,
                    ..
                }
            )))
        ));
        assert!(!polled.load(Ordering::Relaxed));
        assert!(!driver.token().is_cancelled());
    }
    #[tokio::test]
    async fn preparation_effect_cancellation_drops_pending_effect_without_polling_new_work() {
        struct PendingOwner(Arc<AtomicBool>);
        impl Drop for PendingOwner {
            fn drop(&mut self) {
                self.0.store(true, Ordering::Release);
            }
        }
        let driver = crate::CancelSource::new();
        let entered = Arc::new(tokio::sync::Notify::new());
        let dropped = Arc::new(AtomicBool::new(false));
        let owner = PendingOwner(dropped.clone());
        let signal = entered.clone();
        let trigger = driver.clone();
        let cancel = tokio::spawn(async move {
            signal.notified().await;
            trigger.cancel();
        });
        let result = scoped(
            &driver,
            Instant::now() + std::time::Duration::from_secs(10),
            async move {
                let _owner = owner;
                entered.notify_one();
                std::future::pending::<Result<(), WorkflowError>>().await
            },
        )
        .await;
        cancel.await.unwrap();
        assert!(matches!(
            result,
            Err(WorkflowError::Math(
                crate::math::MathRuntimeError::Cancelled
            ))
        ));
        assert!(dropped.load(Ordering::Acquire));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn private_receiving_cancellation_keeps_selection_and_native_owners_until_drain() {
        let runtime = super::super::super::tests::runtime();
        let rows = pse_authoring::language::parse(
            "package p { def Root { var x:Scalar; eq a:x*x==1; } }",
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(rows, super::super::super::tests::physical())
            .await
            .unwrap();
        let driver = crate::CancelSource::new();
        let mut selected = package.checked_selection(&[root], &driver).await.unwrap();
        let service = runtime.shared.math();
        let baseline = runtime.shared.pool().reserved();
        let flag = Arc::new(AtomicBool::new(false));
        let native_stop = flag.clone();
        let (entered, entry) = tokio::sync::oneshot::channel();
        let (exit, gate) = std::sync::mpsc::channel();
        let native = service.job_scoped(1, 4096, Default::default(), None, move |_| {
            let _ = entered.send(());
            gate.recv().map_err(|error| {
                crate::math::MathRuntimeError::Infrastructure(error.to_string())
            })?;
            assert!(native_stop.load(Ordering::Acquire));
            Err::<(), _>(crate::math::MathRuntimeError::Cancelled)
        });
        let mut operation = Box::pin(receiving(
            &driver,
            &flag,
            Instant::now() + std::time::Duration::from_secs(10),
            async { Ok(native.await?) },
        ));
        assert!(futures_util::poll!(operation.as_mut()).is_pending());
        entry.await.unwrap();
        assert!(runtime.shared.pool().reserved() > baseline);
        driver.cancel();
        assert!(
            futures_util::poll!(operation.as_mut()).is_pending(),
            "private receiving waits for native join"
        );
        assert!(flag.load(Ordering::Acquire));
        assert!(
            runtime.shared.pool().reserved() > baseline,
            "active native ownership cannot release at cancellation"
        );
        runtime
            .canonical
            .store()
            .resolve_logicals(&mut selected.read, &["protection-drain-probe".into()])
            .await
            .unwrap();
        exit.send(()).unwrap();
        assert!(matches!(
            operation.await,
            Err(WorkflowError::Math(
                crate::math::MathRuntimeError::Cancelled
            ))
        ));
        assert_eq!(runtime.shared.pool().reserved(), baseline);
        runtime
            .canonical
            .store()
            .release(selected.read.selection())
            .await
            .unwrap();
        assert!(
            runtime
                .canonical
                .store()
                .resolve_logicals(&mut selected.read, &["after-release-probe".into()])
                .await
                .is_err()
        );
    }
}
