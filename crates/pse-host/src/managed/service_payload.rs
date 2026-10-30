// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Child and capture ownership within one detached native service, without external subtrees.
use super::{
    payload::{self, PayloadReport, PayloadRequest},
    streams::ChunkObserver,
};
use crate::{HostError, HostManager, admission::Coordinator, invocation::InvocationEvidence};
use serde::{Deserialize, Serialize};
use std::{io::Write as _, time::Instant};

/// Protected original child binding; decoding this record lends no stop or release authority.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ServiceCustody {
    pub(super) version: u32,
    pub(super) manager: String,
    pub(super) service: crate::manager::UnitBinding,
    pub(super) child: pse_diagnostics::process_facts::ProcessIdentity,
}

pub(super) async fn run<O: ChunkObserver>(
    request: PayloadRequest<O>,
) -> Result<PayloadReport<O>, HostError> {
    let PayloadRequest {
        directory,
        controller,
        root,
        mut command,
        selection,
        trampoline,
        policy,
        cancellation,
        observer,
    } = request;
    let startup = Instant::now()
        .checked_add(policy.startup)
        .ok_or_else(|| HostError::Ownership("managed startup horizon overflow".into()))?;
    let manager = HostManager::connect_user(None).await?;
    let original = manager.authenticate_service_controller(&controller).await?;
    let coordinator = Coordinator::open(&root, &command.environment)?;
    if selection.profile().ordinary() {
        coordinator
            .admit_ordinary(&manager, &original, startup)
            .await?;
    } else {
        coordinator
            .bind_service(&manager, &selection, &controller, startup)
            .await?;
        // Prove both the marker and actual service/allocation ancestry before payload effects.
        if coordinator
            .inherit(
                &command.environment,
                &manager,
                Some(&selection),
                &original,
                startup,
            )
            .await?
            .is_none()
        {
            return Err(HostError::Ownership(
                "detached finite service has no actual original reservation".into(),
            ));
        }
    }
    command
        .environment
        .insert("PSE_HOST_WORKLOAD_UNIT".into(), controller.unit().into());
    command.environment.insert(
        "PSE_MANAGED_INVOCATION".into(),
        directory.id().as_str().into(),
    );
    command.environment.insert(
        super::supervisor::CHECKOUT.into(),
        super::request::encode_path(&root).into(),
    );
    command.environment.insert(
        "PSE_MANAGED_STATE_BASE_BYTES".into(),
        super::request::encode_path(
            &crate::paths::HostPaths::new(&command.environment)
                .base(crate::paths::UserDirectory::State)?,
        )
        .into(),
    );
    let (mut child, mut gate, capture) = payload::captured_child(
        &directory,
        &command,
        &trampoline,
        observer,
        policy.write,
        startup,
    )
    .await?;
    let mut cancelled = false;
    let setup = async {
        let pid = child
            .id()
            .ok_or_else(|| HostError::Ownership("detached original child already reaped".into()))?;
        let identity = match pse_diagnostics::process_facts::ProcessReader::default().process(pid) {
            pse_diagnostics::process_facts::ProcessRead::Present(facts) => facts.identity,
            _ => {
                return Err(HostError::Ownership(
                    "detached original child identity unavailable".into(),
                ));
            }
        };
        let service = manager.service_controller_binding(&controller).await?;
        let member = std::fs::read_to_string(format!("/proc/{pid}/cgroup"))?;
        if !member
            .lines()
            .any(|line| line.strip_prefix("0::") == Some(service.group.as_str()))
        {
            return Err(HostError::Ownership(
                "detached child did not inherit its original service cgroup".into(),
            ));
        }
        directory.publish_evidence(
            InvocationEvidence::Custody,
            &ServiceCustody {
                version: 5,
                manager: manager.identity_key(),
                service,
                child: identity,
            },
        )?;
        super::readiness::SupervisorNotifier::authenticate(&manager, &controller)
            .await?
            .send_ready("same-service child, admission and capture initialized")
            .await?;
        gate.write_all(&[1])?;
        Ok::<_, HostError>(())
    };
    let execution = match tokio::time::timeout_at(startup.into(), setup).await {
        Ok(Ok(())) => tokio::select! {
            result = child.wait() => result.map(Some).map_err(HostError::from),
            () = cancellation.cancelled() => { cancelled = true; Ok(None) },
        },
        Ok(Err(error)) => Err(error),
        Err(_) => Err(HostError::Deadline(
            "detached service payload initialization".into(),
        )),
    };
    drop(gate);
    let deadline = Instant::now()
        .checked_add(policy.drain)
        .ok_or_else(|| HostError::Ownership("managed drain horizon overflow".into()))?;
    let kill = if matches!(execution, Ok(Some(_))) {
        Ok(())
    } else {
        child.start_kill().map_err(HostError::from)
    };
    // Native whole-population TERM reaches escaped sessions too. It includes this wrapper,
    // whose signal handler is already registered, instead of reconstructing process trees.
    let population = manager
        .terminate_service_population(&controller, deadline)
        .await;
    // Each independent attempt is retained, even when cancellation or another attempt fails.
    let waited = match &execution {
        Ok(Some(status)) => Ok(*status),
        _ => tokio::time::timeout_at(deadline.into(), child.wait())
            .await
            .map_err(|_| HostError::Deadline("detached original child reap".into()))
            .and_then(|result| result.map_err(HostError::from)),
    };
    let capture = capture.finish(deadline).await;
    let kill = match (kill, population) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
        (Err(operation), Err(cleanup)) => Err(HostError::Settlement {
            operation: Box::new(operation),
            cleanup: Box::new(cleanup),
        }),
    };
    let operation_error = execution.err();
    Ok(PayloadReport {
        native_pending: true,
        native_settled: false,
        child: waited,
        capture,
        operation_error,
        kill,
        borrowed: false,
        cancelled,
    })
}
