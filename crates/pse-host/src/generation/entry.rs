// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Entry protection and authenticated new-work classification, separate from parsing.
use super::*;
use std::time::{Duration, Instant};

/// An explicit report by the performing owner; never inferred from an exit code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntrySettlement {
    /// No retained executable paths, children or effectful resource obligations were created.
    NoEffects,
    /// The performing owner positively settled every protected child/resource/finalizer.
    ResourcesSettled,
    /// Actual settlement is unavailable; keep the durable protective claim.
    Unresolved,
}

/// Actual executable generation protection, including entries with no invocation ledger row.
#[derive(Debug)]
pub struct EntryGuard {
    pub(super) usage: Option<GenerationUse>,
    pub(super) reference: Option<PreparedGeneration>,
}
impl EntryGuard {
    /// Acquire before ordinary entry dispatch. The foreground trampoline must first claim
    /// its inherited descriptors; cold seed and raw version probes have no published use.
    pub fn acquire_current() -> Result<Self, HostError> {
        Self::acquire_executable_with_handoff(
            &std::env::current_exe()?,
            std::env::var_os(transfer::HANDOFF).as_deref(),
        )
    }

    #[cfg(test)]
    fn acquire_executable(executable: &Path) -> Result<Self, HostError> {
        Self::acquire_executable_with_handoff(executable, None)
    }
    fn acquire_executable_with_handoff(
        executable: &Path,
        handoff: Option<&OsStr>,
    ) -> Result<Self, HostError> {
        let empty = || Self {
            usage: None,
            reference: None,
        };
        let Some(directory) = executable.parent() else {
            return Ok(empty());
        };
        let Some(tools) = directory
            .parent()
            .filter(|path| path.file_name().is_some_and(|name| name == "host-tools"))
        else {
            return Ok(empty());
        };
        if !matches!(
            executable.file_name().and_then(OsStr::to_str),
            Some("pse" | "pse-host" | "cargo-pse-correctness")
        ) {
            return Err(invalid("entry executable is not a fixed host tool"));
        }
        let generation = directory
            .file_name()
            .and_then(OsStr::to_str)
            .filter(|name| component(name) && name.starts_with("generation."))
            .ok_or_else(|| invalid("entry executable is outside a fixed generation"))?;
        let usage = match transfer::adopt(tools, generation, handoff)? {
            Some(usage) => usage,
            None => GenerationUse::acquire(tools, generation)?,
        };
        let reference = PreparedGeneration {
            generation: generation.into(),
            protocol: usage.manifest().control_protocol(),
            manifest_digest: usage.manifest().digest()?,
        };
        Ok(Self {
            usage: Some(usage),
            reference: Some(reference),
        })
    }

    /// Original generation identity, independent of a subsequently advanced current selector.
    pub fn prepared(&self) -> Option<&PreparedGeneration> {
        self.reference.as_ref()
    }
    /// Physical checkout owning this authenticated entry, independent of `-C` selection.
    /// Mutable source-built images have no prepared checkout identity.
    pub fn checkout(&self) -> Option<PathBuf> {
        let tools = self.usage.as_ref()?.record.parent()?.parent()?.parent()?;
        let target = tools
            .parent()
            .filter(|path| path.file_name().is_some_and(|name| name == "target"))?;
        Some(target.parent()?.to_owned())
    }

    /// Release only an explicit performing-owner settlement report. Drop, process loss,
    /// timeout, success/failure status and missing ledger rows establish no such report.
    pub fn complete(self, settlement: EntrySettlement) -> Result<(), HostError> {
        if settlement != EntrySettlement::Unresolved
            && let Some(usage) = self.usage
        {
            usage.settle()?;
        }
        Ok(())
    }
}

/// Freshness disposition for a parsed new-work verb. Passive/internal/control/adapter
/// paths do not call this function and therefore cannot trigger its rebuild decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NewWorkDecision {
    /// Recorded inputs agree; use the prepared generation.
    Current,
    /// Existing authenticated workload containment owns the nested call; never rebuild.
    PreserveNested,
    /// A stale top-level new-work call must rebuild and re-execute before its effect.
    Rebuild,
}

/// Classify only after parsing a new-work command. A nesting marker is not authority:
/// native workload containment is queried from the original user manager and kernel unit.
pub async fn new_work_decision(
    root: &Path,
    inputs: &Environment,
) -> Result<NewWorkDecision, HostError> {
    if freshness(root, inputs)? == Freshness::Current {
        return Ok(NewWorkDecision::Current);
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    let nested = tokio::time::timeout_at(deadline.into(), async {
        let manager = crate::HostManager::connect_user(None).await?;
        let current = manager.current_unit().await?;
        manager.native_workload(&current).await
    })
    .await
    .map_err(|_| HostError::Deadline("host rebuild containment authentication".into()))??;
    if nested {
        return Ok(NewWorkDecision::PreserveNested);
    }
    if transfer::was_reexecuted(inputs)? {
        return Err(invalid(
            "host inputs changed after rebuilding; retry the top-level command",
        ));
    }
    Ok(NewWorkDecision::Rebuild)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn installed_entries_pin_their_original_generation_and_require_explicit_settlement() {
        let (_fixture, tools) = crate::generation::tests::installed_fixture();
        let executable = tools.join("generation.fixture/pse-host");
        let entry = EntryGuard::acquire_executable(&executable).unwrap();
        let identity = entry.prepared().unwrap().clone();
        assert_eq!(identity.generation, "generation.fixture");
        assert_eq!(identity.protocol, 1);
        assert_eq!(
            GenerationUse::inspect(&tools, &identity.generation).unwrap(),
            GenerationUseState::Active
        );
        entry.complete(EntrySettlement::NoEffects).unwrap();
        assert_eq!(
            GenerationUse::inspect(&tools, &identity.generation).unwrap(),
            GenerationUseState::Available
        );
        EntryGuard::acquire_executable(&executable)
            .unwrap()
            .complete(EntrySettlement::ResourcesSettled)
            .unwrap();
        assert_eq!(
            GenerationUse::inspect(&tools, &identity.generation).unwrap(),
            GenerationUseState::Available
        );
        EntryGuard::acquire_executable(&executable)
            .unwrap()
            .complete(EntrySettlement::Unresolved)
            .unwrap();
        assert_eq!(
            GenerationUse::inspect(&tools, &identity.generation).unwrap(),
            GenerationUseState::Unsettled
        );
    }
    #[test]
    fn mutable_seed_paths_do_not_impersonate_installed_generations() {
        let (_fixture, tools) = crate::generation::tests::installed_fixture();
        let seed = tools.parent().unwrap().join("debug/pse-host");
        assert!(
            EntryGuard::acquire_executable(&seed)
                .unwrap()
                .prepared()
                .is_none()
        );
        assert!(
            EntryGuard::acquire_executable(&tools.join("generation.fixture/arbitrary")).is_err()
        );
        assert!(EntryGuard::acquire_executable(&tools.join("current/pse-host")).is_err());
    }
}
