// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Current original raw-payload leader authentication for internal workflow consumers.
use super::*;
use crate::invocation::InvocationMetadata;
use serde::de::DeserializeOwned;
use std::path::Path;

/// Checked current original child and ledger reads. This lends no process stop,
/// reservation retirement, arbitrary-directory adoption or terminal-publication authority.
pub(crate) struct PayloadLeader {
    root: PathBuf,
    id: CliInvocationId,
    store: InvocationStore,
    #[cfg(test)]
    directory: Arc<InvocationDirectory>,
    _membership: crate::manager::CurrentMembership,
    _manager: HostManager,
}
impl PayloadLeader {
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }
    pub(crate) fn invocation_id(&self) -> &CliInvocationId {
        &self.id
    }
    #[cfg(test)]
    pub(crate) fn directory(&self) -> &Arc<InvocationDirectory> {
        &self.directory
    }
    /// Reopen only this original store/id, checking its ordinary metadata and filesystem bindings.
    pub(crate) fn metadata<O: DeserializeOwned>(&self) -> Result<InvocationMetadata<O>, HostError> {
        self.store.metadata(&self.id)
    }
}

/// Authenticate the current original leader, rather than any descendant carrying its hints.
/// Checkout/state/id values route checked opens; physical/process custody supplies authority.
pub(crate) async fn current_payload(environment: &Environment) -> Result<PayloadLeader, HostError> {
    let hint = |key| {
        environment
            .get(std::ffi::OsStr::new(key))
            .and_then(|value| value.to_str())
            .ok_or_else(|| invalid("current payload routing hint absent"))
    };
    let root = std::fs::canonicalize(request::decode_path(hint(CHECKOUT)?)?)?;
    let state = request::decode_path(hint(STATE_BASE)?)?;
    let id = CliInvocationId::try_from(hint("PSE_MANAGED_INVOCATION")?.to_owned())?;
    let mut routed = Environment::new();
    routed.insert("XDG_STATE_HOME".into(), state.into_os_string());
    let store = InvocationStore::new(CheckoutState::resolve(&root, &routed)?);
    let directory = Arc::new(store.directory(&id)?);
    let metadata: InvocationMetadata<Box<serde_json::value::RawValue>> = store.metadata(&id)?;
    let launch: LaunchRecord = directory
        .evidence(InvocationEvidence::Launch)?
        .ok_or_else(|| invalid("current payload original launch absent"))?;
    launch.validate(&id, &metadata)?;
    let executable = launch.verified_executable(&root)?;
    let controller: ControllerRecord = directory
        .evidence(InvocationEvidence::Controller)?
        .ok_or_else(|| invalid("current payload original controller absent"))?;
    if controller.version != 1
        || std::fs::read_link(format!("/proc/{}/exe", controller.process.pid))? != executable
    {
        return Err(invalid(
            "current payload original controller generation differs",
        ));
    }
    let custody: payload::PayloadCustody = directory
        .evidence(InvocationEvidence::Custody)?
        .ok_or_else(|| invalid("current payload original custody absent"))?;
    let intent: payload::PayloadIntent = directory
        .evidence(InvocationEvidence::PayloadIntent)?
        .ok_or_else(|| invalid("current payload original creation intent absent"))?;
    let manager = HostManager::connect_user(None).await?;
    let membership = manager.current_membership().await?;
    if current_identity()? != custody.child
        || custody.manager != manager.identity_key()
        || intent.controller != launch.controller
        || !membership.matches_root(&custody.workload)
        || membership.root().unit != intent.workload.unit()
        || !manager.native_workload(membership.root()).await?
    {
        return Err(invalid(
            "current process is not the original managed payload leader",
        ));
    }
    match (
        intent.version,
        custody.version,
        &custody.subgroup,
        &custody.creation,
    ) {
        (1, 2, None, None)
            if intent.subgroup.is_none()
                && membership.member()
                    == Path::new("/sys/fs/cgroup")
                        .join(custody.workload.group.trim_start_matches('/')) => {}
        (2, 3, Some(group), Some(creation))
            if membership.member() == group.path()
                && membership.contains_original(group)
                && intent.subgroup.as_ref() == Some(creation)
                && creation.parent_binding().root() == &custody.workload => {}
        _ => {
            return Err(invalid(
                "current payload original subgroup/root custody differs",
            ));
        }
    }
    ancestry::validate_live_controller(&manager, &launch, &controller).await?;
    manager
        .validate_binding(&intent.workload, &custody.workload)
        .await?;
    membership.validate(&manager).await?;
    if current_identity()? != custody.child {
        return Err(invalid(
            "current payload leader changed during authentication",
        ));
    }
    Ok(PayloadLeader {
        root,
        id,
        store,
        #[cfg(test)]
        directory,
        _membership: membership,
        _manager: manager,
    })
}
