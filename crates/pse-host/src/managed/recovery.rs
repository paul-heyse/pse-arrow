// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original payload/allocation recovery from protected performing-owner custody.
use super::{
    payload::{PayloadCustody, PayloadIntent},
    supervisor::ControllerRecord,
};
use crate::{
    HostError, HostManager, LaunchIntent, UnitObservation,
    admission::{Permit, RecoveredPermit},
    envelope::{ErrorRecord, RedactionPolicy},
    invocation::{InvocationDirectory, InvocationEvidence},
};
use serde::Serialize;
use std::time::Instant;

#[derive(Debug)]
pub(super) struct RecoveryReport {
    workload: Result<Option<UnitObservation>, HostError>,
    operation: Result<Option<UnitObservation>, HostError>,
    reservation: Result<Option<bool>, HostError>,
    _workload: Option<crate::manager::RecordedUnit>,
    _operation: Option<crate::manager::RecordedUnit>,
    _reservation: Option<RecoveredPermit>,
}
#[derive(Serialize)]
pub(super) struct RecoveryView {
    workload: Result<Option<UnitObservation>, ErrorRecord>,
    operation: Result<Option<UnitObservation>, ErrorRecord>,
    reservation: Result<Option<bool>, ErrorRecord>,
}
#[derive(Debug)]
pub(super) struct RecoveryCustody {
    _workload: Option<crate::manager::RecordedUnit>,
    _operation: Option<crate::manager::RecordedUnit>,
    _reservation: Option<RecoveredPermit>,
}
impl RecoveryReport {
    pub(super) fn view(&self, policy: &RedactionPolicy) -> RecoveryView {
        RecoveryView {
            workload: self
                .workload
                .as_ref()
                .map(Clone::clone)
                .map_err(|error| ErrorRecord::from_typed(error, policy)),
            operation: self
                .operation
                .as_ref()
                .map(Clone::clone)
                .map_err(|error| ErrorRecord::from_typed(error, policy)),
            reservation: self
                .reservation
                .as_ref()
                .copied()
                .map_err(|error| ErrorRecord::from_typed(error, policy)),
        }
    }
    pub(super) fn finish(
        self,
        policy: &RedactionPolicy,
    ) -> (RecoveryView, Vec<HostError>, RecoveryCustody) {
        let view = self.view(policy);
        let mut failures = Vec::new();
        for result in [self.workload, self.operation] {
            if let Err(error) = result {
                failures.push(error);
            }
        }
        match self.reservation {
            Err(error) => failures.push(error),
            Ok(Some(false)) => failures.push(HostError::Ownership(
                "original recovered finite charge remains retained".into(),
            )),
            Ok(None | Some(true)) => {}
        }
        (
            view,
            failures,
            RecoveryCustody {
                _workload: self._workload,
                _operation: self._operation,
                _reservation: self._reservation,
            },
        )
    }
    #[cfg(test)]
    pub(super) fn resources_settled(&self) -> bool {
        self.workload.is_ok()
            && self.operation.is_ok()
            && matches!(self.reservation, Ok(None | Some(true)))
    }
}

/// Caller must authenticate the actual original finalizer and close this invocation's
/// launch authority first. Recorded process exit supplies no child reap/capture authority.
pub(super) async fn recover(
    directory: &InvocationDirectory,
    manager: &HostManager,
    controller: &LaunchIntent,
    original: &ControllerRecord,
    deadline: Instant,
) -> Result<RecoveryReport, HostError> {
    let intent: PayloadIntent = directory
        .evidence(InvocationEvidence::PayloadIntent)?
        .ok_or_else(|| HostError::Ownership("original payload creation intent absent".into()))?;
    let custody: PayloadCustody = directory
        .evidence(InvocationEvidence::Custody)?
        .ok_or_else(|| {
            HostError::Ownership("original initialized payload custody absent".into())
        })?;
    if intent.version != 1
        || custody.version != 2
        || intent.controller != *controller
        || custody.manager != manager.identity_key()
        || original.manager != custody.manager
    {
        return Err(HostError::Ownership(
            "original payload recovery custody differs".into(),
        ));
    }
    let mut workload = None;
    let mut operation = None;
    let (workload_result, operation_result) = tokio::join!(
        async {
            workload = manager
                .recorded_original(
                    &intent.workload,
                    &custody.manager,
                    &custody.workload,
                    Some(&custody.child),
                )
                .await?;
            match &mut workload {
                Some(workload) => workload.stop_and_drain(deadline).await,
                None => Ok(None),
            }
        },
        async {
            match (&intent.operation, &custody.operation) {
                (Some(intent), Some(binding)) => {
                    operation = manager
                        .recorded_original(intent, &custody.manager, binding, None)
                        .await?;
                    match &mut operation {
                        Some(operation) => operation.stop_and_drain(deadline).await,
                        None => Ok(None),
                    }
                }
                (None, None) => Ok(None),
                _ => Err(HostError::Ownership(
                    "original operation recovery binding missing".into(),
                )),
            }
        }
    );
    let mut reservation = None;
    let reservation_result = match &custody.reservation {
        Some(record) => {
            async {
                reservation = Some(
                    Permit::recover(
                        record,
                        manager,
                        controller,
                        &original.binding,
                        &original.process,
                        &intent.workload,
                        &custody.workload,
                    )
                    .await?,
                );
                reservation
                    .as_mut()
                    .ok_or_else(|| {
                        HostError::Ownership("original recovered permit missing".into())
                    })?
                    .release(manager, deadline)
                    .await
                    .map(Some)
            }
            .await
        }
        None => Ok(None),
    };
    Ok(RecoveryReport {
        workload: workload_result,
        operation: operation_result,
        reservation: reservation_result,
        _workload: workload,
        _operation: operation,
        _reservation: reservation,
    })
}

#[cfg(test)]
mod tests;
