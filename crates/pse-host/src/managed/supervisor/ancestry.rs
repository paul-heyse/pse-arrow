// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authenticated original-member handoff and ordered ancestor launch exclusion.
use super::*;
use crate::{
    PayloadParent, PayloadParentBinding,
    admission::{Coordinator, Permit, PermitBorrow},
    invocation::InvocationLaunchGuard,
};
use std::{
    collections::BTreeSet,
    ffi::OsString,
    os::unix::ffi::{OsStrExt, OsStringExt},
    time::Duration,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct InvocationLink {
    checkout: Vec<u8>,
    state: Vec<u8>,
    pub(super) invocation: CliInvocationId,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ParentRecord {
    pub(super) link: InvocationLink,
    member: PayloadParentBinding,
    origin: ProcessIdentity,
    reservation: Option<PermitBorrow>,
}
struct Ancestor {
    directory: InvocationDirectory,
    launch: LaunchRecord,
    controller: ControllerRecord,
}
/// Actual member authority plus its checked root-to-leaf launch ancestry. No parent stop or
/// reservation-release authority is lent to a nested invocation.
pub struct ManagedParent {
    pub(super) record: ParentRecord,
    ancestors: Vec<Ancestor>,
    pub(super) member: PayloadParent,
    pub(super) permit: Option<Permit>,
}
impl std::fmt::Debug for ManagedParent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ManagedParent")
            .field("invocation", self.invocation_id())
            .field("ancestor_count", &self.ancestors.len())
            .finish_non_exhaustive()
    }
}
impl ManagedParent {
    /// Immediate original parent; metadata records this relation before child launch effects.
    pub fn invocation_id(&self) -> &CliInvocationId {
        &self.record.link.invocation
    }
    pub(in crate::managed) async fn locks(
        &self,
        manager: &HostManager,
        deadline: Instant,
    ) -> Result<Vec<InvocationLaunchGuard<'_>>, HostError> {
        let mut guards = Vec::new();
        for ancestor in &self.ancestors {
            loop {
                match ancestor.directory.try_launch_lock() {
                    Ok(guard) => {
                        if guard.is_closed()? {
                            return Err(invalid("an original ancestor closed launches"));
                        }
                        guards.push(guard);
                        break;
                    }
                    Err(HostError::Io(error)) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        if Instant::now() >= deadline {
                            return Err(HostError::Deadline(
                                "original ancestor launch exclusion".into(),
                            ));
                        }
                        tokio::time::sleep(Duration::from_millis(5)).await;
                    }
                    Err(error) => return Err(error),
                }
            }
        }
        self.validate(manager).await?;
        Ok(guards)
    }
    pub(in crate::managed) async fn validate(
        &self,
        manager: &HostManager,
    ) -> Result<(), HostError> {
        for ancestor in &self.ancestors {
            validate_controller(manager, ancestor).await?;
        }
        self.member.validate_origin(&self.record.origin).await
    }
    pub(super) async fn reopen(
        record: ParentRecord,
        manager: &HostManager,
        selection: &Selection,
    ) -> Result<Self, HostError> {
        let ancestors = ancestors(&record.link, manager, &record.member, None).await?;
        let member = manager.recover_payload_parent(&record.member).await?;
        member.validate_origin(&record.origin).await?;
        let permit = match &record.reservation {
            Some(record) => Some(
                Permit::from_borrow_record(record, manager, member.binding().root(), selection)
                    .await?,
            ),
            None if !selection.profile().ordinary() => {
                return Err(invalid(
                    "nested finite request has no authenticated enclosing reservation",
                ));
            }
            None => None,
        };
        Ok(Self {
            record,
            ancestors,
            member,
            permit,
        })
    }
    pub(in crate::managed) fn member(&self) -> &PayloadParent {
        &self.member
    }
    pub(in crate::managed) fn take_permit(&mut self) -> Option<Permit> {
        self.permit.take()
    }
}
async fn validate_controller(manager: &HostManager, ancestor: &Ancestor) -> Result<(), HostError> {
    validate_live_controller(manager, &ancestor.launch, &ancestor.controller).await
}
pub(super) async fn validate_live_controller(
    manager: &HostManager,
    launch: &LaunchRecord,
    controller: &ControllerRecord,
) -> Result<(), HostError> {
    if !matches!(ProcessReader::default().process(controller.process.pid), ProcessRead::Present(facts)
        if facts.identity == controller.process && !matches!(facts.stat.state, 'Z' | 'X'))
    {
        return Err(invalid(
            "original ancestor controller is absent or changed for launch",
        ));
    }
    let unit = manager
        .recorded_original(
            &launch.controller,
            &controller.manager,
            &controller.binding,
            Some(&controller.process),
        )
        .await?
        .ok_or_else(|| invalid("original ancestor controller is gone for launch"))?;
    let observed = unit.observe().await?;
    if observed.active_state != "active" || observed.main_pid != controller.process.pid {
        return Err(invalid("original ancestor is no longer live for launch"));
    }
    Ok(())
}
async fn ancestors(
    link: &InvocationLink,
    manager: &HostManager,
    member_binding: &PayloadParentBinding,
    membership: Option<&crate::manager::CurrentMembership>,
) -> Result<Vec<Ancestor>, HostError> {
    let mut seen = BTreeSet::new();
    let mut current = Some(link.clone());
    let mut values = Vec::new();
    while let Some(link) = current {
        if !seen.insert((
            link.checkout.clone(),
            link.state.clone(),
            link.invocation.to_string(),
        )) {
            return Err(invalid("original managed ancestry contains a cycle"));
        }
        let checkout = PathBuf::from(OsString::from_vec(link.checkout));
        let mut environment = Environment::new();
        environment.insert("XDG_STATE_HOME".into(), OsString::from_vec(link.state));
        let store = InvocationStore::new(CheckoutState::resolve(&checkout, &environment)?);
        let directory = store.directory(&link.invocation)?;
        let metadata: crate::invocation::InvocationMetadata<Box<serde_json::value::RawValue>> =
            store.metadata(&link.invocation)?;
        let launch: LaunchRecord = directory
            .evidence(InvocationEvidence::Launch)?
            .ok_or_else(|| invalid("original ancestor launch absent"))?;
        launch.validate(&link.invocation, &metadata)?;
        let executable = launch.verified_executable(&checkout)?;
        let controller: ControllerRecord = directory
            .evidence(InvocationEvidence::Controller)?
            .ok_or_else(|| invalid("original ancestor controller absent"))?;
        if std::fs::read_link(format!("/proc/{}/exe", controller.process.pid))? != executable {
            return Err(invalid(
                "original ancestor executable differs from captured control generation",
            ));
        }
        let custody: payload::PayloadCustody = directory
            .evidence(InvocationEvidence::Custody)?
            .ok_or_else(|| invalid("original ancestor payload custody absent"))?;
        if controller.version != 1
            || custody.manager != manager.identity_key()
            || custody.workload != *member_binding.root()
            || !matches!(custody.version, 2 | 3)
        {
            return Err(invalid(
                "original ancestor differs from actual native workload root",
            ));
        }
        if let Some(subgroup) = &custody.subgroup {
            if !member_binding.contains_original(subgroup) {
                return Err(invalid(
                    "recorded nested origin is outside an ancestor payload subgroup",
                ));
            }
            if let Some(member) = membership {
                if !member.contains_original(subgroup) {
                    return Err(invalid(
                        "actual caller is outside original parent payload subgroup",
                    ));
                }
            }
        } else if custody.version != 2 {
            return Err(invalid("original nested subgroup custody absent"));
        }
        current = launch.parent.as_ref().map(|parent| parent.link.clone());
        let ancestor = Ancestor {
            directory,
            launch,
            controller,
        };
        validate_controller(manager, &ancestor).await?;
        values.push(ancestor);
    }
    values.reverse();
    Ok(values)
}
/// Capture nested authority from actual native membership and protected parent records.
/// Environment hints only route checked opens; they cannot establish membership or ownership.
pub async fn parent(
    environment: &Environment,
    selection: &Selection,
    deadline: Instant,
) -> Result<Option<ManagedParent>, HostError> {
    let manager = HostManager::connect_user(None).await?;
    let membership = manager.current_membership().await?;
    let native = manager.native_workload(membership.root()).await?;
    let Some(id) = environment.get(std::ffi::OsStr::new("PSE_MANAGED_INVOCATION")) else {
        return if native {
            Err(invalid(
                "native nested entry has no protected managed parent",
            ))
        } else {
            Ok(None)
        };
    };
    if !native {
        return Err(invalid(
            "managed parent hint does not match actual native membership",
        ));
    }
    let id = CliInvocationId::try_from(
        id.to_str()
            .ok_or_else(|| invalid("managed parent id is not UTF-8"))?
            .to_owned(),
    )?;
    let hint = |key| {
        environment
            .get(std::ffi::OsStr::new(key))
            .and_then(|value| value.to_str())
            .ok_or_else(|| invalid("managed parent byte routing hint absent"))
    };
    let checkout = request::decode_path(hint(CHECKOUT)?)?;
    let state = request::decode_path(hint(STATE_BASE)?)?;
    let link = InvocationLink {
        checkout: checkout.as_os_str().as_bytes().to_vec(),
        state: state.as_os_str().as_bytes().to_vec(),
        invocation: id,
    };
    let member = membership.borrow_payload_parent(&manager).await?;
    let ancestors = ancestors(&link, &manager, member.binding(), Some(&membership)).await?;
    let coordinator = Coordinator::ordinary(environment)?;
    let permit = coordinator
        .inherit(
            environment,
            &manager,
            Some(selection),
            membership.root(),
            deadline,
        )
        .await?;
    if permit.is_none() && !selection.profile().ordinary() {
        return Err(invalid("nested finite demand has no enclosing reservation"));
    }
    let reservation = permit
        .as_ref()
        .map(|permit| permit.borrow_record(member.binding().root()))
        .transpose()?;
    let record = ParentRecord {
        link,
        member: member.binding().clone(),
        origin: current_identity()?,
        reservation,
    };
    Ok(Some(ManagedParent {
        record,
        ancestors,
        member,
        permit,
    }))
}
#[cfg(test)]
mod tests;
