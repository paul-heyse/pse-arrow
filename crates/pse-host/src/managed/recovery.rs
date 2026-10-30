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
    subgroup: Option<super::payload::nested::SubgroupReport>,
    _subgroup: Option<crate::OwnedSubgroup>,
}
#[derive(Serialize)]
pub(super) struct RecoveryView {
    workload: Result<Option<UnitObservation>, ErrorRecord>,
    operation: Result<Option<UnitObservation>, ErrorRecord>,
    reservation: Result<Option<bool>, ErrorRecord>,
    subgroup: Option<super::payload::nested::SubgroupView>,
}
#[derive(Debug)]
pub(super) struct RecoveryCustody {
    _workload: Option<crate::manager::RecordedUnit>,
    _operation: Option<crate::manager::RecordedUnit>,
    _reservation: Option<RecoveredPermit>,
    _subgroup: Option<crate::OwnedSubgroup>,
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
            subgroup: self.subgroup.as_ref().map(|report| report.view(policy)),
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
        if let Some(report) = self.subgroup {
            failures.extend(report.into_failures());
        }
        (
            view,
            failures,
            RecoveryCustody {
                _workload: self._workload,
                _operation: self._operation,
                _reservation: self._reservation,
                _subgroup: self._subgroup,
            },
        )
    }
    #[cfg(test)]
    pub(super) fn resources_settled(&self) -> bool {
        self.workload.is_ok()
            && self.operation.is_ok()
            && matches!(self.reservation, Ok(None | Some(true)))
            && self
                .subgroup
                .as_ref()
                .is_none_or(super::payload::nested::SubgroupReport::settled)
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
    if intent.version == 2 && custody.version == 3 {
        if intent.controller != *controller
            || custody.manager != manager.identity_key()
            || original.manager != custody.manager
            || intent.operation.is_some()
            || custody.operation.is_some()
            || custody.reservation.is_some()
            || intent.subgroup != custody.creation
        {
            return Err(HostError::Ownership(
                "original nested recovery binding differs".into(),
            ));
        }
        let creation = custody
            .creation
            .as_ref()
            .ok_or_else(|| HostError::Ownership("original nested creation missing".into()))?;
        if creation.parent_binding().root() != &custody.workload
            || creation.parent_binding().intent() != &intent.workload
        {
            return Err(HostError::Ownership(
                "original nested workload root differs".into(),
            ));
        }
        let binding = custody
            .subgroup
            .as_ref()
            .ok_or_else(|| HostError::Ownership("original nested inode custody missing".into()))?;
        let parent = manager
            .recover_payload_parent(creation.parent_binding())
            .await?;
        let mut subgroup = parent.recover_created(creation, binding).await?;
        let report = match &mut subgroup {
            Some(group) => Some(super::payload::nested::settle(group, deadline).await),
            None => None,
        };
        return Ok(RecoveryReport {
            workload: Ok(None),
            operation: Ok(None),
            reservation: Ok(None),
            _workload: None,
            _operation: None,
            _reservation: None,
            subgroup: report,
            _subgroup: subgroup,
        });
    }
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
        subgroup: None,
        _subgroup: None,
    })
}

#[cfg(test)]
mod tests;
