// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact current-finalizer claim binding. Original performing claims are never adopted here.
use super::*;
use crate::{LaunchIntent, envelope::CliInvocationId, manager::UnitBinding};
use pse_diagnostics::process_facts::ProcessIdentity;

/// Descriptive original scope supplied by the authenticated finalizer owner. Decoding this
/// value provides no native custody or settlement capability, and paths are never adopted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FinalizerUseScope {
    checkout: OsString,
    directory: OsString,
    invocation: CliInvocationId,
    manager: String,
    controller: LaunchIntent,
    controller_binding: UnitBinding,
    controller_process: ProcessIdentity,
}
impl FinalizerUseScope {
    pub(crate) fn new(
        checkout: PathBuf,
        directory: PathBuf,
        invocation: CliInvocationId,
        manager: String,
        controller: LaunchIntent,
        controller_binding: UnitBinding,
        controller_process: ProcessIdentity,
    ) -> Result<Self, HostError> {
        if !checkout.is_absolute() || !directory.is_absolute() || manager.is_empty() {
            return Err(invalid(
                "finalizer scope lacks original absolute provenance or manager",
            ));
        }
        Ok(Self {
            checkout: checkout.into_os_string(),
            directory: directory.into_os_string(),
            invocation,
            manager,
            controller,
            controller_binding,
            controller_process,
        })
    }
}

/// An owner-minted observation of one original claim. Serialization is diagnostic only;
/// neither a decoded locator nor its paths can construct or retire a bound use.
#[derive(Debug, Serialize)]
pub(crate) struct FinalizerClaimLocator {
    tools: OsString,
    generation: String,
    record: String,
    manifest_digest: String,
    owner: ProcessIdentity,
    scope: FinalizerUseScope,
}

/// Existing current-finalizer claim plus its exact immutable registration. Only the actual
/// local completion owner may call settlement after its independent native duties finish.
#[derive(Debug)]
pub(crate) struct BoundFinalizerUse {
    usage: GenerationUse,
    registered: UseRecord,
    locator: FinalizerClaimLocator,
}

impl GenerationUse {
    fn current_record(&self) -> Result<UseRecord, HostError> {
        let record = transfer::read_record(&self.record)?;
        if record.version != VERSION
            || record.owner != transfer::identity()?
            || record.manifest_digest != self.manifest.digest()?
        {
            return Err(invalid(
                "generation claim differs from the current performing owner and manifest",
            ));
        }
        let directory = self
            .record
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .ok_or_else(|| invalid("generation claim has no tool root"))?
            .join(
                self.record
                    .parent()
                    .and_then(Path::file_name)
                    .ok_or_else(|| invalid("generation claim has no generation"))?,
            );
        if GenerationManifest::load(&directory)? != self.manifest {
            return Err(invalid(
                "prepared manifest changed since generation claim acquisition",
            ));
        }
        Ok(record)
    }

    /// Bind only this actual process's existing claim. Native finalizer authentication is
    /// supplied by the caller; original launcher/supervisor/exec claims remain independent.
    pub(crate) fn bind_finalizer(
        self,
        scope: FinalizerUseScope,
    ) -> Result<BoundFinalizerUse, HostError> {
        let mut registered = self.current_record()?;
        if registered.handoff.is_some() || registered.finalizer.is_some() {
            return Err(invalid(
                "generation claim already has a handoff or finalizer binding",
            ));
        }
        let records = self
            .record
            .parent()
            .ok_or_else(|| invalid("generation claim has no parent"))?;
        let generation = records
            .file_name()
            .and_then(OsStr::to_str)
            .filter(|name| component(name) && name.starts_with("generation."))
            .ok_or_else(|| invalid("finalizer claim has no fixed generation"))?;
        let tools = records
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| invalid("finalizer claim has no fixed tool root"))?;
        if tools != Path::new(&scope.checkout).join("target/host-tools") {
            return Err(invalid(
                "finalizer scope differs from the claim's original checkout",
            ));
        }
        let record = self
            .record
            .file_stem()
            .and_then(OsStr::to_str)
            .filter(|value| value.len() == 36 && uuid::Uuid::parse_str(value).is_ok())
            .ok_or_else(|| invalid("generation claim has no original UUID"))?;
        let locator = FinalizerClaimLocator {
            tools: tools.as_os_str().to_owned(),
            generation: generation.into(),
            record: record.into(),
            manifest_digest: registered.manifest_digest.clone(),
            owner: registered.owner.clone(),
            scope: scope.clone(),
        };
        registered.finalizer = Some(scope);
        self.replace_record(&registered)?;
        Ok(BoundFinalizerUse {
            usage: self,
            registered,
            locator,
        })
    }
}

impl BoundFinalizerUse {
    pub(crate) fn locator(&self) -> &FinalizerClaimLocator {
        &self.locator
    }
    pub(crate) fn scope(&self) -> &FinalizerUseScope {
        &self.locator.scope
    }
    /// Retire this same current owner only after the finalizer's sealed local-completion
    /// owner permits it. This checks identity/binding, never reconstructs native settlement.
    pub(crate) fn settle_current(self) -> Result<(), HostError> {
        if self.usage.current_record()? != self.registered {
            return Err(invalid(
                "bound finalizer generation claim changed before local retirement",
            ));
        }
        self.usage.settle()
    }
}

#[cfg(test)]
mod tests;
