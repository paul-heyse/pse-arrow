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
    let custody: super::super::service_payload::ServiceCustody = directory
        .evidence(InvocationEvidence::Custody)?
        .ok_or_else(|| invalid("same-service original child custody absent"))?;
    let manager = HostManager::connect_user(None).await?;
    let membership = manager.current_membership().await?;
    if custody.version != 5
        || custody.service != controller.binding
        || custody.manager != manager.identity_key()
        || current_identity()? != custody.child
        || !membership.matches_root(&custody.service)
        || membership.root().unit != launch.controller.unit()
        || !manager.native_workload(membership.root()).await?
    {
        return Err(invalid(
            "current process is not the original same-service payload child",
        ));
    }
    manager
        .validate_binding(&launch.controller, &custody.service)
        .await?;
    Ok(PayloadLeader {
        root,
        id,
        store,
        _membership: membership,
        _manager: manager,
    })
}
