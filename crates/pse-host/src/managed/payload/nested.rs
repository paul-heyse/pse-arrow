// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Nested raw payloads stay below their actual original member; parent ownership is borrowed.
use super::*;
use crate::envelope::{ErrorRecord, RedactionPolicy};
use crate::{CreationOutcome, OwnedSubgroup, RemovalReceipt, managed::supervisor::ManagedParent};

#[derive(Serialize)]
pub(in crate::managed) struct SubgroupView {
    kill: Result<(), ErrorRecord>,
    drain: Result<(), ErrorRecord>,
    removal: Result<Option<RemovalView>, ErrorRecord>,
}
#[derive(Serialize)]
struct RemovalView {
    original_device: u64,
    original_inode: u64,
    removed_directories: u64,
    original_removed: bool,
    failures: Vec<ErrorRecord>,
}

#[derive(Debug)]
pub(in crate::managed) struct SubgroupReport {
    kill: Result<(), HostError>,
    drain: Result<(), HostError>,
    removal: Result<Option<RemovalReceipt>, HostError>,
}
impl SubgroupReport {
    pub(in crate::managed) fn settled(&self) -> bool {
        self.kill.is_ok()
            && self.drain.is_ok()
            && self.removal.as_ref().is_ok_and(|receipt| {
                receipt.as_ref().is_none_or(|receipt| {
                    receipt.original_removed() && receipt.failures().is_empty()
                })
            })
    }
    pub(in crate::managed) fn view(&self, policy: &RedactionPolicy) -> SubgroupView {
        SubgroupView {
            kill: self
                .kill
                .as_ref()
                .copied()
                .map_err(|error| ErrorRecord::from_typed(error, policy)),
            drain: self
                .drain
                .as_ref()
                .copied()
                .map_err(|error| ErrorRecord::from_typed(error, policy)),
            removal: self
                .removal
                .as_ref()
                .map(|receipt| {
                    receipt.as_ref().map(|receipt| RemovalView {
                        original_device: receipt.original_identity().0,
                        original_inode: receipt.original_identity().1,
                        removed_directories: receipt.removed_directories(),
                        original_removed: receipt.original_removed(),
                        failures: receipt
                            .failures()
                            .iter()
                            .map(|error| ErrorRecord::from_typed(error, policy))
                            .collect(),
                    })
                })
                .map_err(|error| ErrorRecord::from_typed(error, policy)),
        }
    }
    pub(in crate::managed) fn into_failures(self) -> Vec<HostError> {
        let mut errors = Vec::new();
        errors.extend(self.kill.err());
        errors.extend(self.drain.err());
        match self.removal {
            Err(error) => errors.push(error),
            Ok(Some(receipt)) => {
                if !receipt.original_removed() {
                    errors.push(HostError::Ownership(
                        "original nested subgroup removal remains unresolved".into(),
                    ));
                }
                errors.extend(receipt.failures);
            }
            Ok(None) => {}
        }
        errors
    }
}
pub(in crate::managed) async fn settle(
    group: &mut OwnedSubgroup,
    deadline: Instant,
) -> SubgroupReport {
    let (kill, drain) = tokio::join!(group.kill(), group.wait_empty(deadline));
    let removal = group.remove_tree(deadline).await.map(Some);
    SubgroupReport {
        kill,
        drain,
        removal,
    }
}
pub(super) async fn run<O: ChunkObserver>(
    request: PayloadRequest<O>,
    mut parent: ManagedParent,
) -> Result<PayloadReport<O>, HostError> {
    let PayloadRequest {
        directory,
        controller,
        root,
        mut command,
        trampoline,
        policy,
        cancellation,
        observer,
        selection: _,
        parent: _,
    } = request;
    let startup_deadline = deadline(policy.startup)?;
    let manager = HostManager::connect_user(None).await?;
    let (original, _, _) = manager.managed_service_placement(&controller).await?;
    manager
        .require_nested_service(&controller, parent.member().binding())
        .await?;
    parent.validate(&manager).await?;
    let mut permit = parent.take_permit();
    let creation = parent
        .member()
        .creation_intent(&format!("pse-{}", directory.id()))?;
    directory.publish_evidence(
        InvocationEvidence::PayloadIntent,
        &PayloadIntent {
            version: 2,
            controller: controller.clone(),
            workload: parent.member().binding().intent().clone(),
            operation: None,
            subgroup: Some(creation.clone()),
        },
    )?;
    command.environment.insert(
        "PSE_MANAGED_INVOCATION".into(),
        directory.id().as_str().into(),
    );
    command.environment.insert(
        super::super::supervisor::CHECKOUT.into(),
        super::super::request::encode_path(&root).into(),
    );
    let state = crate::paths::HostPaths::new(&command.environment)
        .base(crate::paths::UserDirectory::State)?;
    command.environment.insert(
        "PSE_MANAGED_STATE_BASE_BYTES".into(),
        super::super::request::encode_path(&state).into(),
    );
    let (mut child, mut gate_write, capture) = captured_child(
        &directory,
        &command,
        &trampoline,
        observer,
        policy.write,
        startup_deadline,
    )
    .await?;
    let mut group: Option<OwnedSubgroup> = None;
    let mut unbound_creation = None;
    let mut cancelled = false;
    let startup = async {
        let _ancestors = parent.locks(&manager, startup_deadline).await?;
        let own = launch_lock(&directory, startup_deadline).await?;
        if own.is_closed()? {
            return Err(HostError::Ownership(
                "nested invocation already closed launches".into(),
            ));
        }
        match parent.member().create(&creation).await {
            CreationOutcome::Created(created) => group = Some(*created),
            CreationOutcome::NoEffect(error) => return Err(error),
            CreationOutcome::CreatedButUnbound {
                intent,
                opened,
                cause,
            } => {
                group = opened.map(|opened| opened.into_owned());
                if group.is_none() {
                    unbound_creation = Some(intent);
                }
                return Err(cause);
            }
        }
        let group = group.as_ref().ok_or_else(|| {
            HostError::Ownership("created original nested subgroup missing".into())
        })?;
        let attachment = group.attach(&mut child).await?;
        group.validate_attachment(&attachment, &child).await?;
        directory.publish_evidence(
            InvocationEvidence::Custody,
            &PayloadCustody {
                version: 3,
                manager: manager.identity_key(),
                workload: parent.member().binding().root().clone(),
                child: attachment.process().clone(),
                operation: None,
                reservation: None,
                subgroup: Some(group.binding()),
                creation: Some(creation.clone()),
            },
        )?;
        parent.validate(&manager).await?;
        manager
            .require_nested_service(&controller, parent.member().binding())
            .await?;
        let current = manager.managed_service_placement(&controller).await?.0;
        if current.invocation != original.invocation
            || current.main_pid != original.main_pid
            || current.control_group != original.control_group
        {
            return Err(HostError::Ownership(
                "original nested controller changed before payload gate".into(),
            ));
        }
        if cancellation.is_cancelled() {
            cancelled = true;
            return Ok(());
        }
        super::super::readiness::SupervisorNotifier::authenticate(&manager, &controller)
            .await?
            .send_ready("nested original subgroup custody and capture initialized")
            .await?;
        gate_write.write_all(&[1])?;
        Ok(())
    };
    let execution = match tokio::time::timeout_at(startup_deadline.into(), startup).await {
        Ok(Ok(())) if cancelled => Ok(None),
        Ok(Ok(())) => tokio::select! {
            result = child.wait() => result.map(Some).map_err(HostError::from),
            () = cancellation.cancelled() => { cancelled = true; Ok(None) },
        },
        Ok(Err(error)) => Err(error),
        Err(_) => Err(HostError::Deadline(
            "absolute original nested payload startup".into(),
        )),
    };
    drop(gate_write);
    let drain_deadline = deadline(policy.drain)?;
    let closure = match launch_lock(&directory, drain_deadline).await {
        Ok(guard) => guard.close(),
        Err(error) => Err(error),
    };
    let subgroup = match &mut group {
        Some(group) => settle(group, drain_deadline).await,
        None => SubgroupReport { kill: Ok(()), drain: Ok(()), removal: match unbound_creation {
            Some(_) => Err(HostError::Ownership("original nested creation committed without opened inode custody; protected intent retained".into())),
            None => Ok(None),
        } },
    };
    let kill = match &execution {
        Ok(Some(_)) => Ok(()),
        _ => match child.try_wait() {
            Ok(Some(_)) => Ok(()),
            Ok(None) => child.start_kill().map_err(HostError::from),
            Err(error) => Err(error.into()),
        },
    };
    let waited = match &execution {
        Ok(Some(status)) => Ok(*status),
        _ => tokio::time::timeout_at(drain_deadline.into(), child.wait())
            .await
            .map_err(|_| HostError::Deadline("original nested child reap".into()))
            .and_then(|result| result.map_err(HostError::from)),
    };
    let borrowed = permit.is_some();
    let permit_result = match &mut permit {
        Some(permit) => permit.release(&manager, drain_deadline).await,
        None => Ok(true),
    };
    let capture = capture.finish(drain_deadline).await;
    let (child, operation_error) = match execution {
        Ok(Some(status)) => (Ok(status), None),
        Ok(None) => (waited, None),
        Err(error) => (waited, Some(error)),
    };
    Ok(PayloadReport {
        child,
        capture,
        operation_error,
        closure,
        workload: Ok(None),
        operation: Ok(None),
        kill,
        permit: permit_result,
        borrowed,
        cancelled,
        subgroup: Some(subgroup),
        retained: RetainedResources {
            _workload: None,
            _operation: None,
            _permit: permit,
            _subgroup: group,
        },
    })
}
