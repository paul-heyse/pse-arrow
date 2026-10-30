// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authenticate the executing image independently from source freshness and native lifetimes.
use super::*;
use std::time::{Duration, Instant};

/// Immutable executing-image identity. It creates no GC claims or resource-settlement duties.
#[derive(Debug)]
pub struct EntryGuard {
    pub(super) reference: Option<PreparedGeneration>,
    pub(super) checkout: Option<PathBuf>,
}
impl EntryGuard {
    /// Verify a fixed prepared executable before dispatch. Mutable bootstrap seeds have no
    /// published image identity; their narrow construction route retains its existing policy.
    pub fn acquire_current() -> Result<Self, HostError> {
        Self::acquire_executable(&std::env::current_exe()?)
    }
    fn acquire_executable(executable: &Path) -> Result<Self, HostError> {
        let empty = || Self {
            reference: None,
            checkout: None,
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
        let manifest = GenerationManifest::load(directory)?;
        let name = executable
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or_else(|| invalid("entry executable name is not UTF-8"))?;
        if !manifest.binaries.contains_key(name) {
            return Err(invalid(
                "entry executable is absent from its authenticated manifest",
            ));
        }
        let reference = PreparedGeneration {
            generation: generation.into(),
            protocol: manifest.control_protocol(),
            manifest_digest: manifest.digest()?,
        };
        let checkout = tools
            .parent()
            .filter(|path| path.file_name().is_some_and(|name| name == "target"))
            .and_then(Path::parent)
            .map(Path::to_owned);
        Ok(Self {
            reference: Some(reference),
            checkout,
        })
    }
    /// Original image identity, independent of a subsequently advanced current selector.
    pub fn prepared(&self) -> Option<&PreparedGeneration> {
        self.reference.as_ref()
    }
    /// Authenticated prepared checkout; mutable source-built seeds have none.
    pub fn checkout(&self) -> Option<PathBuf> {
        self.checkout.clone()
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
    if reexec::was_reexecuted(inputs)? {
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
    fn installed_image_authentication_leaves_unknown_gc_claims_untouched() {
        let (_fixture, tools) = crate::generation::tests::installed_fixture();
        let unknown = tools.join(".uses/unknown-legacy");
        fs::create_dir_all(unknown.parent().unwrap()).unwrap();
        fs::write(&unknown, b"unknown claim").unwrap();
        let entry =
            EntryGuard::acquire_executable(&tools.join("generation.fixture/pse-host")).unwrap();
        assert_eq!(entry.prepared().unwrap().generation, "generation.fixture");
        assert_eq!(entry.prepared().unwrap().protocol, 1);
        drop(entry);
        assert_eq!(fs::read(unknown).unwrap(), b"unknown claim");
        assert_eq!(fs::read_dir(tools.join(".uses")).unwrap().count(), 1);
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
        fs::write(
            tools.join("generation.fixture/pse"),
            b"unlisted executable bytes",
        )
        .unwrap();
        assert!(EntryGuard::acquire_executable(&tools.join("generation.fixture/pse")).is_err());
    }
}
