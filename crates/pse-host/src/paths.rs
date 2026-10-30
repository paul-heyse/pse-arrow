// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Host-owned user paths. Resolving a path never creates or removes its contents.

use crate::environment::{Environment, EnvironmentError};
use sha2::{Digest, Sha256};
use std::{
    ffi::{OsStr, OsString},
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    process::Command,
};

/// XDG directory selection with its declared home-relative default.
#[derive(Clone, Copy, Debug)]
pub enum UserDirectory {
    /// Durable user state, defaulting to `.local/state` beneath HOME.
    State,
    /// Installed user data, defaulting to `.local/share` beneath HOME.
    Data,
    /// Disposable caches, defaulting to `.cache` beneath HOME.
    Cache,
    /// User configuration, defaulting to `.config` beneath HOME.
    Config,
}

/// Captured environment selection; no process-global environment reads.
pub struct HostPaths<'a> {
    environment: &'a Environment,
}
impl std::fmt::Debug for HostPaths<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Captured values can contain credentials; a resolver's Debug is not a public projection.
        formatter.debug_struct("HostPaths").finish_non_exhaustive()
    }
}

impl<'a> HostPaths<'a> {
    /// Bind the caller's captured environment.
    pub fn new(environment: &'a Environment) -> Self {
        Self { environment }
    }

    /// Empty path variables have the same meaning as absent path variables.
    pub fn selected(&self, name: &str) -> Option<&'a OsStr> {
        self.environment
            .get(OsStr::new(name))
            .map(OsString::as_os_str)
            .filter(|value| !value.is_empty())
    }

    /// Resolve an XDG base, retaining existing path spellings.
    pub fn base(&self, directory: UserDirectory) -> Result<PathBuf, EnvironmentError> {
        let (name, suffix) = match directory {
            UserDirectory::State => ("XDG_STATE_HOME", ".local/state"),
            UserDirectory::Data => ("XDG_DATA_HOME", ".local/share"),
            UserDirectory::Cache => ("XDG_CACHE_HOME", ".cache"),
            UserDirectory::Config => ("XDG_CONFIG_HOME", ".config"),
        };
        self.selected(name)
            .map(PathBuf::from)
            .map(Ok)
            .unwrap_or_else(|| {
                self.selected("HOME")
                    .map(|home| PathBuf::from(home).join(suffix))
                    .ok_or_else(|| {
                        EnvironmentError("HOME is required for default user paths".into())
                    })
            })
    }

    /// Existing optional telemetry state location, not a new store migration.
    pub fn telemetry(&self) -> Result<PathBuf, EnvironmentError> {
        self.override_or("PSE_TELEMETRY_STATE", UserDirectory::State, "pse/telemetry")
    }
    /// Existing canonical service state location.
    pub fn surreal_state(&self) -> Result<PathBuf, EnvironmentError> {
        self.override_or(
            "PSE_SURREAL_STATE",
            UserDirectory::State,
            "pse-arrow/surreal-functional-v2",
        )
    }
    /// Existing native provider cache location.
    pub fn native_cache(&self) -> Result<PathBuf, EnvironmentError> {
        self.override_or("PSE_NATIVE_CACHE", UserDirectory::Cache, "pse-arrow/native")
    }
    /// Existing canonical service/tool-module installation root.
    pub fn surreal_tools(&self) -> Result<PathBuf, EnvironmentError> {
        self.override_or(
            "PSE_SURREAL_TOOLS",
            UserDirectory::Data,
            "pse-arrow/tools/surreal",
        )
    }
    /// Existing diagnostic-tool installations, including sccache.
    pub fn tools(&self) -> Result<PathBuf, EnvironmentError> {
        Ok(self.base(UserDirectory::Data)?.join("pse/tools"))
    }
    /// Existing compiler cache location.
    pub fn compiler_cache(&self) -> Result<PathBuf, EnvironmentError> {
        self.override_or("SCCACHE_DIR", UserDirectory::Cache, "pse-arrow/sccache")
    }
    /// Existing native cache-server endpoint.
    pub fn compiler_cache_socket(&self) -> Result<PathBuf, EnvironmentError> {
        if let Some(path) = self.selected("SCCACHE_SERVER_UDS") {
            return Ok(path.into());
        }
        match self.selected("XDG_RUNTIME_DIR") {
            Some(path) => Ok(PathBuf::from(path).join("pse-sccache.sock")),
            None => Ok(self
                .base(UserDirectory::Cache)?
                .join("pse-arrow/sccache.sock")),
        }
    }
    /// Admission requires a selected absolute runtime base; no home fallback is valid.
    pub fn runtime(&self) -> Result<PathBuf, EnvironmentError> {
        let path = self
            .selected("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .ok_or_else(|| {
                EnvironmentError("XDG_RUNTIME_DIR is required for shared host admission".into())
            })?;
        if !path.is_absolute() {
            return Err(EnvironmentError("XDG_RUNTIME_DIR must be absolute".into()));
        }
        Ok(path)
    }
    /// Existing systemd user-unit directory.
    pub fn user_units(&self) -> Result<PathBuf, EnvironmentError> {
        Ok(self.base(UserDirectory::Config)?.join("systemd/user"))
    }
    fn override_or(
        &self,
        name: &str,
        base: UserDirectory,
        suffix: &str,
    ) -> Result<PathBuf, EnvironmentError> {
        self.selected(name)
            .map(PathBuf::from)
            .map(Ok)
            .unwrap_or_else(|| Ok(self.base(base)?.join(suffix)))
    }
}

/// Checkout-local operational state, distinct from the shared repository relation.
#[derive(Clone, Debug)]
pub struct CheckoutState {
    /// Content-derived identity of the canonical checkout location and Git common directory.
    pub checkout_id: CheckoutId,
    /// Common-directory identity shared by worktrees of one repository.
    pub repository_id: RepositoryId,
    /// New state lives beneath the pse-arrow prefix; existing resources retain their paths.
    pub root: PathBuf,
}

/// Identity of one checkout's inspected canonical location and Git common directory.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(transparent)]
pub struct CheckoutId(String);
impl CheckoutId {
    /// Stable path component for this checkout binding.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Shared repository relation, intentionally not interchangeable with a checkout identity.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(transparent)]
pub struct RepositoryId(String);
impl RepositoryId {
    /// Stable textual relation key for inspected worktrees of this repository.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn identity(paths: &[&Path]) -> String {
    let mut digest = Sha256::new();
    for path in paths {
        digest.update(path.as_os_str().as_bytes());
        digest.update([0]);
    }
    digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

impl CheckoutState {
    /// Inspect checkout identity explicitly without creating state or starting a service.
    pub fn resolve(root: &Path, environment: &Environment) -> Result<Self, EnvironmentError> {
        let root = std::fs::canonicalize(root)
            .map_err(|_| EnvironmentError("checkout path cannot be resolved".into()))?;
        let output = Command::new("git")
            .env_clear()
            .envs(environment)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_COMMON_DIR")
            .arg("-C")
            .arg(&root)
            .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
            .output()
            .map_err(|_| EnvironmentError("Git common directory cannot be inspected".into()))?;
        if !output.status.success() {
            return Err(EnvironmentError(
                "checkout has no Git common directory".into(),
            ));
        }
        let mut bytes = output.stdout;
        if bytes.last() == Some(&b'\n') {
            bytes.pop();
        }
        let common = PathBuf::from(OsStr::from_bytes(&bytes));
        Self::from_paths(&root, &common, environment)
    }
    /// Bind already inspected absolute paths, useful when one operation shares discovery.
    pub fn from_paths(
        root: &Path,
        common: &Path,
        environment: &Environment,
    ) -> Result<Self, EnvironmentError> {
        if !root.is_absolute() || !common.is_absolute() {
            return Err(EnvironmentError(
                "checkout and Git common directory must be absolute".into(),
            ));
        }
        let checkout_id = CheckoutId(identity(&[root, common]));
        let repository_id = RepositoryId(identity(&[common]));
        let state = HostPaths::new(environment)
            .base(UserDirectory::State)?
            .join("pse-arrow")
            .join(checkout_id.as_str());
        if !state.is_absolute() {
            return Err(EnvironmentError(
                "checkout state root must be absolute".into(),
            ));
        }
        Ok(Self {
            checkout_id,
            repository_id,
            root: state,
        })
    }
    /// Invocation ledger directories are owned by the invocation subsystem.
    pub fn invocations(&self) -> PathBuf {
        self.root.join("invocations")
    }
    /// Actual fixture owners govern retained fixture state and disposal.
    pub fn fixtures(&self) -> PathBuf {
        self.root.join("fixtures")
    }
    /// Context-addressed API artifacts.
    pub fn api(&self) -> PathBuf {
        self.root.join("api")
    }
    /// Authored probe inputs must not become disposable presentation logs.
    pub fn probes(&self) -> PathBuf {
        self.root.join("probes")
    }
    /// Session scratch root; callers validate session identifiers before appending them.
    pub fn scratch(&self) -> PathBuf {
        self.root.join("scratch")
    }
    /// Derived handoff records with separately retained authored intent.
    pub fn handoff(&self) -> PathBuf {
        self.root.join("handoff")
    }
}

/// Resource meaning, not an age-based deletion decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetentionClass {
    /// Terminal presentation logs may become eligible once their owners settle.
    DisposablePresentation,
    /// An artifact has its own explicit retention owner.
    ProtectedArtifact,
    /// Uniquely authored probe input is not a disposable log.
    AuthoredProbe,
    /// Scientific data retains canonical/product ownership.
    ScientificData,
    /// Fixture evidence requires its actual owner's disposal decision.
    FixtureEvidence,
}

/// Positive owner evidence required even for otherwise disposable presentation.
#[derive(Clone, Copy, Debug)]
pub struct RetentionEvidence {
    /// Original invocation reached a terminal outcome.
    pub terminal: bool,
    /// Actual protected owners/children settled.
    pub settled: bool,
    /// A prepared generation still has a live user.
    pub generation_in_use: bool,
    /// Independent resource owner retains this evidence.
    pub owner_retains: bool,
}
impl RetentionClass {
    /// Eligibility only; this function never removes data or establishes owner facts.
    pub fn eligible(self, evidence: RetentionEvidence) -> bool {
        self == Self::DisposablePresentation
            && evidence.terminal
            && evidence.settled
            && !evidence.generation_in_use
            && !evidence.owner_retains
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn env() -> Environment {
        Environment::from([("HOME".into(), "/home/test".into())])
    }
    #[test]
    fn path_selection_preserves_existing_locations_and_ignores_empty_overrides() {
        let mut values = env();
        for key in [
            "XDG_STATE_HOME",
            "XDG_DATA_HOME",
            "XDG_CACHE_HOME",
            "XDG_CONFIG_HOME",
            "PSE_TELEMETRY_STATE",
            "PSE_SURREAL_STATE",
            "PSE_NATIVE_CACHE",
            "PSE_SURREAL_TOOLS",
            "SCCACHE_DIR",
            "SCCACHE_SERVER_UDS",
        ] {
            values.insert(key.into(), "".into());
        }
        let paths = HostPaths::new(&values);
        assert_eq!(
            paths.telemetry().unwrap(),
            Path::new("/home/test/.local/state/pse/telemetry")
        );
        assert_eq!(
            paths.surreal_state().unwrap(),
            Path::new("/home/test/.local/state/pse-arrow/surreal-functional-v2")
        );
        assert_eq!(
            paths.native_cache().unwrap(),
            Path::new("/home/test/.cache/pse-arrow/native")
        );
        assert_eq!(
            paths.surreal_tools().unwrap(),
            Path::new("/home/test/.local/share/pse-arrow/tools/surreal")
        );
        assert_eq!(
            paths.tools().unwrap(),
            Path::new("/home/test/.local/share/pse/tools")
        );
        assert_eq!(
            paths.compiler_cache().unwrap(),
            Path::new("/home/test/.cache/pse-arrow/sccache")
        );
        assert_eq!(
            paths.compiler_cache_socket().unwrap(),
            Path::new("/home/test/.cache/pse-arrow/sccache.sock")
        );
        assert_eq!(
            paths.user_units().unwrap(),
            Path::new("/home/test/.config/systemd/user")
        );
        assert!(paths.runtime().is_err());
        for (key, value) in [
            ("XDG_STATE_HOME", "/state"),
            ("XDG_DATA_HOME", "/data"),
            ("XDG_CACHE_HOME", "/cache"),
            ("XDG_CONFIG_HOME", "/config"),
            ("XDG_RUNTIME_DIR", "/runtime"),
        ] {
            values.insert(key.into(), value.into());
        }
        let paths = HostPaths::new(&values);
        assert_eq!(
            paths.telemetry().unwrap(),
            Path::new("/state/pse/telemetry")
        );
        assert_eq!(paths.tools().unwrap(), Path::new("/data/pse/tools"));
        assert_eq!(
            paths.native_cache().unwrap(),
            Path::new("/cache/pse-arrow/native")
        );
        assert_eq!(
            paths.user_units().unwrap(),
            Path::new("/config/systemd/user")
        );
        assert_eq!(
            paths.compiler_cache_socket().unwrap(),
            Path::new("/runtime/pse-sccache.sock")
        );
        assert_eq!(paths.runtime().unwrap(), Path::new("/runtime"));
        for key in [
            "PSE_TELEMETRY_STATE",
            "PSE_SURREAL_STATE",
            "PSE_NATIVE_CACHE",
            "PSE_SURREAL_TOOLS",
            "SCCACHE_DIR",
            "SCCACHE_SERVER_UDS",
        ] {
            values.insert(key.into(), "/selected".into());
        }
        let paths = HostPaths::new(&values);
        for path in [
            paths.telemetry(),
            paths.surreal_state(),
            paths.native_cache(),
            paths.surreal_tools(),
            paths.compiler_cache(),
            paths.compiler_cache_socket(),
        ] {
            assert_eq!(path.unwrap(), Path::new("/selected"));
        }
    }
    #[test]
    fn checkout_paths_distinguish_worktrees_and_share_repository_relation() {
        let main = CheckoutState::from_paths(
            Path::new("/repo/main"),
            Path::new("/repo/main/.git"),
            &env(),
        )
        .unwrap();
        let work = CheckoutState::from_paths(
            Path::new("/repo/work"),
            Path::new("/repo/main/.git"),
            &env(),
        )
        .unwrap();
        assert_ne!(main.checkout_id, work.checkout_id);
        assert_eq!(main.repository_id, work.repository_id);
        assert_eq!(main.invocations(), main.root.join("invocations"));
        assert!(main.root.starts_with("/home/test/.local/state/pse-arrow"));
    }
    #[test]
    fn checkout_identity_reads_real_git_worktrees_without_creating_state() {
        let fixture = tempfile::tempdir().unwrap();
        let main = fixture.path().join("main");
        let work = fixture.path().join("work");
        assert!(
            Command::new("git")
                .args(["init", "--quiet"])
                .arg(&main)
                .status()
                .unwrap()
                .success()
        );
        assert!(
            Command::new("git")
                .arg("-C")
                .arg(&main)
                .args([
                    "-c",
                    "user.name=path test",
                    "-c",
                    "user.email=paths@example.invalid",
                    "-c",
                    "commit.gpgsign=false",
                    "-c",
                    "core.hooksPath=/dev/null",
                    "commit",
                    "--quiet",
                    "--allow-empty",
                    "-m",
                    "fixture"
                ])
                .status()
                .unwrap()
                .success()
        );
        assert!(
            Command::new("git")
                .arg("-C")
                .arg(&main)
                .args(["worktree", "add", "--quiet", "--detach"])
                .arg(&work)
                .status()
                .unwrap()
                .success()
        );
        let environment = Environment::from([(
            "XDG_STATE_HOME".into(),
            fixture.path().join("state").into_os_string(),
        )]);
        let main_state = CheckoutState::resolve(&main, &environment).unwrap();
        let work_state = CheckoutState::resolve(&work, &environment).unwrap();
        assert_ne!(main_state.checkout_id, work_state.checkout_id);
        assert_eq!(main_state.repository_id, work_state.repository_id);
        assert!(!main_state.root.exists());
        assert!(!work_state.root.exists());
    }
    #[test]
    fn retention_requires_positive_terminal_settlement_and_owner_release() {
        let safe = RetentionEvidence {
            terminal: true,
            settled: true,
            generation_in_use: false,
            owner_retains: false,
        };
        assert!(RetentionClass::DisposablePresentation.eligible(safe));
        for evidence in [
            RetentionEvidence {
                terminal: false,
                ..safe
            },
            RetentionEvidence {
                settled: false,
                ..safe
            },
            RetentionEvidence {
                generation_in_use: true,
                ..safe
            },
            RetentionEvidence {
                owner_retains: true,
                ..safe
            },
        ] {
            assert!(!RetentionClass::DisposablePresentation.eligible(evidence));
        }
        for class in [
            RetentionClass::ProtectedArtifact,
            RetentionClass::AuthoredProbe,
            RetentionClass::ScientificData,
            RetentionClass::FixtureEvidence,
        ] {
            assert!(!class.eligible(safe));
        }
    }
}
