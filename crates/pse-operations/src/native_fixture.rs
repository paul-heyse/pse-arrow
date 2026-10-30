// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! A borrowed selection of one freshly owned command-scoped canonical fixture.
//! This descriptor grants no service shutdown or persistent-data disposal authority.
use crate::canonical::{CanonicalError, CanonicalOptions};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::Read,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
};

#[cfg(feature = "native-service")]
use std::io::Write;

/// Explicit child selection, issued only after the private native service is ready.
pub const SELECTION_ENV: &str = "PSE_NATIVE_CANONICAL_FIXTURE";
const SELECTION_FILE: &str = "test-child.json";
const PRIVATE_BYTES: u64 = 16 * 1024;
fn refused(message: impl Into<String>) -> CanonicalError {
    CanonicalError::Configuration(message.into())
}
/// Borrowed state identity; only the original NativeCanonicalService owns disposal.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureSelection {
    version: u32,
    state: PathBuf,
    device: u64,
    inode: u64,
    nonce: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DisposableIdentity {
    version: u32,
    nonce: String,
}
fn private_read<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, CanonicalError> {
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(|error| refused(format!("fixture selection read: {error}")))?;
    let metadata = file
        .metadata()
        .map_err(|error| refused(error.to_string()))?;
    if !metadata.is_file()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o077 != 0
        || metadata.len() > PRIVATE_BYTES
    {
        return Err(refused(
            "fixture selection requires bounded private owned bytes",
        ));
    }
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(PRIVATE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| refused(error.to_string()))?;
    if bytes.len() as u64 > PRIVATE_BYTES {
        return Err(refused("fixture selection bytes exceeded bound"));
    }
    serde_json::from_slice(&bytes)
        .map_err(|error| refused(format!("fixture selection document: {error}")))
}
impl FixtureSelection {
    #[cfg(feature = "native-service")]
    pub(crate) fn issue(state: &Path, nonce: &str) -> Result<Self, CanonicalError> {
        let metadata = fs::symlink_metadata(state).map_err(|error| refused(error.to_string()))?;
        let selection = Self {
            version: 1,
            state: state.to_owned(),
            device: metadata.dev(),
            inode: metadata.ino(),
            nonce: nonce.to_owned(),
        };
        selection.validate()?;
        let path = state.join(SELECTION_FILE);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
            .map_err(|error| refused(error.to_string()))?;
        file.write_all(
            &serde_json::to_vec(&selection).map_err(|error| refused(error.to_string()))?,
        )
        .map_err(|error| refused(error.to_string()))?;
        file.sync_all()
            .map_err(|error| refused(error.to_string()))?;
        Ok(selection)
    }
    fn validate(&self) -> Result<(), CanonicalError> {
        if self.version != 1 || !self.state.is_absolute() || self.nonce.is_empty() {
            return Err(refused("invalid native fixture selection"));
        }
        let metadata =
            fs::symlink_metadata(&self.state).map_err(|error| refused(error.to_string()))?;
        if !metadata.is_dir()
            || metadata.file_type().is_symlink()
            || metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.mode() & 0o077 != 0
            || (metadata.dev(), metadata.ino()) != (self.device, self.inode)
            || fs::canonicalize(&self.state).map_err(|error| refused(error.to_string()))?
                != self.state
        {
            return Err(refused("native fixture state identity changed"));
        }
        let disposable: DisposableIdentity = private_read(&self.state.join("disposable.json"))?;
        if disposable.version != 1 || disposable.nonce != self.nonce {
            return Err(refused("native fixture disposable identity differs"));
        }
        Ok(())
    }
    /// Read an explicitly selected original descriptor; no service is implicitly started.
    /// # Errors
    /// Missing, modified, unowned or otherwise invalid selection.
    pub fn selected() -> Result<Self, CanonicalError> {
        let path = std::env::var_os(SELECTION_ENV)
            .map(PathBuf::from)
            .ok_or_else(|| {
                refused("canonical fixtures require an explicitly owned native test command")
            })?;
        Self::read(&path)
    }
    /// Validate one explicit borrowed handoff, without environmental discovery.
    /// # Errors
    /// The private descriptor or its original state identity does not match.
    pub fn read(path: &Path) -> Result<Self, CanonicalError> {
        let selection: Self = private_read(path)?;
        selection.validate()?;
        if path != selection.descriptor() {
            return Err(refused(
                "fixture descriptor differs from its original state handoff",
            ));
        }
        Ok(selection)
    }
    /// Original private state borrowed by this child; not a cleanup capability.
    pub fn state(&self) -> &Path {
        &self.state
    }
    /// Exact private descriptor path for a child process environment.
    pub fn descriptor(&self) -> PathBuf {
        self.state.join(SELECTION_FILE)
    }
    /// Verify the selected isolated logical database before creating any remote state.
    /// Explicit transport relays are allowed while credentials/state/namespace remain owned.
    /// # Errors
    /// A different state, namespace or non-isolated database was selected.
    pub fn validate_options(&self, options: &CanonicalOptions) -> Result<(), CanonicalError> {
        self.validate()?;
        let configured = CanonicalOptions::from_state(&self.state)?;
        let suffix = options.database.rsplit('_').next().unwrap_or("");
        if options.state_path() != self.state.as_path()
            || options.namespace != configured.namespace
            || !options.database.starts_with("canonical_test_")
            || uuid::Uuid::parse_str(suffix).is_err()
        {
            return Err(refused(
                "canonical fixture requires an isolated UUID database on its borrowed native service",
            ));
        }
        Ok(())
    }
}

#[cfg(feature = "native-service")]
mod command {
    use super::*;
    use crate::native_service::{NativeCanonicalService, NativeServiceError, NativeServiceSpec};
    use std::{
        ffi::OsString,
        process::ExitStatus,
        time::{Duration, Instant},
    };
    /// Source-built tests from one actual correctness-adapter invocation.
    /// No public constructor admits cached or caller-authored metadata paths.
    #[derive(Debug)]
    pub struct NativeTestCommand {
        program: PathBuf,
        binaries: PathBuf,
        cargo: PathBuf,
        arguments: Vec<OsString>,
        directory: PathBuf,
        _built: Option<tempfile::TempDir>,
    }
    /// Disposal is initially safe before spawn. Once a child exists, only its
    /// observed ordinary exit can establish the positive disposal premise.
    #[derive(Clone, Default)]
    struct ChildExitPremise(std::sync::Arc<std::sync::atomic::AtomicU8>);
    impl ChildExitPremise {
        fn spawned(&self) {
            self.0.store(1, std::sync::atomic::Ordering::Release);
        }
        fn ordinary_exit(&self) {
            self.0.store(2, std::sync::atomic::Ordering::Release);
        }
        fn permits_disposal(&self) -> bool {
            matches!(self.0.load(std::sync::atomic::Ordering::Acquire), 0 | 2)
        }
    }
    async fn settle_child(
        child: &mut tokio::process::Child,
        final_end: Instant,
        state: &Path,
        mut failures: Vec<NativeServiceError>,
    ) -> NativeServiceError {
        // Killing and waiting are independent attempts: a failed signal must
        // never skip the original wait, and all failures retain their causes.
        if let Err(error) = child.start_kill() {
            failures.push(error.into());
        }
        match tokio::time::timeout_at(final_end.into(), child.wait()).await {
            Ok(Ok(_)) => {}
            Ok(Err(error)) => failures.push(error.into()),
            Err(_) => failures.push(NativeServiceError::Refused(
                "original nextest child did not settle",
            )),
        }
        NativeServiceError::Settlement {
            state: state.to_owned(),
            failures,
        }
    }
    async fn child_status(
        command: tokio::process::Command,
        execution_end: Instant,
        final_end: Instant,
        state: &Path,
        premise: ChildExitPremise,
    ) -> Result<ExitStatus, NativeServiceError> {
        child_status_with_signal(
            command,
            execution_end,
            final_end,
            state,
            premise,
            tokio::signal::ctrl_c(),
        )
        .await
    }
    async fn child_status_with_signal<S>(
        mut command: tokio::process::Command,
        execution_end: Instant,
        final_end: Instant,
        state: &Path,
        premise: ChildExitPremise,
        signal: S,
    ) -> Result<ExitStatus, NativeServiceError>
    where
        S: Future<Output = std::io::Result<()>> + Send,
    {
        if Instant::now() >= execution_end {
            return Err(NativeServiceError::Refused(
                "test command deadline expired before launch",
            ));
        }
        let mut child = command.kill_on_drop(true).spawn()?;
        // This precedes every waiter or signal future, including immediate errors.
        premise.spawned();
        enum Observed {
            Wait(std::io::Result<ExitStatus>),
            Signal(std::io::Result<()>),
            Expired,
        }
        let observed = tokio::select! {status=child.wait()=>Observed::Wait(status),_=tokio::time::sleep_until(execution_end.into())=>Observed::Expired,signal=signal=>Observed::Signal(signal)};
        let failure = match observed {
            Observed::Wait(Ok(status)) if status.code().is_some() => {
                premise.ordinary_exit();
                return Ok(status);
            }
            Observed::Wait(Ok(_)) => NativeServiceError::Refused(
                "signal-terminated test runner cannot prove scientific joins",
            ),
            Observed::Wait(Err(error)) | Observed::Signal(Err(error)) => error.into(),
            Observed::Signal(Ok(())) => {
                NativeServiceError::Refused("native fixture test command cancelled")
            }
            Observed::Expired => NativeServiceError::Refused("native fixture test command expired"),
        };
        Err(settle_child(&mut child, final_end, state, vec![failure]).await)
    }
    impl NativeTestCommand {
        /// Compile/list through the source correctness adapter before creating a service.
        /// The same invocation captures nextest's actual Cargo metadata into owned bytes.
        /// # Errors
        /// Invalid prepared tools, build/capture failure or unsupported run selectors.
        pub fn prepare(
            correctness: PathBuf,
            program: PathBuf,
            build_arguments: Vec<OsString>,
            arguments: Vec<OsString>,
            directory: PathBuf,
        ) -> Result<Self, NativeServiceError> {
            for path in [&correctness, &program] {
                if !path.is_absolute() || !path.is_file() {
                    return Err(NativeServiceError::Refused(
                        "explicit prepared correctness adapter and nextest executable required",
                    ));
                }
            }
            if !directory.is_absolute() || !directory.is_dir() {
                return Err(NativeServiceError::Refused(
                    "explicit nextest working directory required",
                ));
            }
            if correctness.canonicalize()?
                != directory
                    .join("target/host-tools/current/cargo-pse-correctness")
                    .canonicalize()?
            {
                return Err(NativeServiceError::Refused(
                    "fixture source build requires this checkout's prepared correctness adapter",
                ));
            }
            for argument in &arguments {
                let value = argument.to_string_lossy();
                if [
                    "--binaries-metadata",
                    "--cargo-metadata",
                    "--archive-file",
                    "--manifest-path",
                    "--package",
                    "--workspace",
                    "--all",
                    "--lib",
                    "--bins",
                    "--tests",
                    "--all-targets",
                    "--features",
                    "--all-features",
                    "--no-default-features",
                    "--no-run",
                ]
                .iter()
                .any(|flag| value == *flag || value.starts_with(&format!("{flag}=")))
                    || value == "-p"
                {
                    return Err(NativeServiceError::Refused(
                        "native fixture accepts run filters over its source-built binaries, not new build or metadata selectors",
                    ));
                }
            }
            for argument in &build_arguments {
                let value = argument.to_string_lossy();
                if ["--list-type", "--message-format"]
                    .iter()
                    .any(|flag| value == *flag || value.starts_with(&format!("{flag}=")))
                {
                    return Err(NativeServiceError::Refused(
                        "fixture constructor owns nextest metadata output modes",
                    ));
                }
            }
            let built = tempfile::Builder::new()
                .prefix("pse-native-built-tests-")
                .tempdir()?;
            let cargo = built.path().join("cargo-metadata.json");
            let binaries = built.path().join("binaries-metadata.json");
            let output = std::process::Command::new(correctness)
                .current_dir(&directory)
                .args([
                    "nextest",
                    "list",
                    "--list-type",
                    "binaries-only",
                    "--message-format",
                    "json",
                ])
                .args(&build_arguments)
                .env("PSE_FIXTURE_CARGO_METADATA_CAPTURE", &cargo)
                .stderr(std::process::Stdio::inherit())
                .output()?;
            if !output.status.success() {
                return Err(NativeServiceError::Command {
                    stage: "source build/list",
                    status: output.status,
                });
            }
            let _: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|_| {
                NativeServiceError::Refused("source nextest list did not produce binary metadata")
            })?;
            let _: serde_json::Value =
                serde_json::from_slice(&fs::read(&cargo)?).map_err(|_| {
                    NativeServiceError::Refused("same-invocation Cargo metadata capture invalid")
                })?;
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&binaries)?;
            file.write_all(&output.stdout)?;
            file.sync_all()?;
            Ok(Self {
                program,
                binaries,
                cargo,
                arguments,
                directory,
                _built: Some(built),
            })
        }
        /// Adopt the host's same-invocation build/list capture without compiling again.
        /// The host owns source admission and immutable capture; this owner supplies only
        /// the fresh service, child execution and physical settlement.
        /// # Errors
        /// Missing/incomplete capture, changed binaries or run-only argument violations.
        pub fn from_source_capture(
            inventory_path: &Path,
            group: usize,
            arguments: Vec<OsString>,
            directory: PathBuf,
        ) -> Result<Self, NativeServiceError> {
            let mut file = OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
                .open(inventory_path)?;
            let metadata = file.metadata()?;
            if !metadata.is_file()
                || metadata.uid() != rustix::process::geteuid().as_raw()
                || metadata.mode() & 0o077 != 0
                || metadata.len() > 64 * 1024 * 1024
            {
                return Err(NativeServiceError::Refused(
                    "source inventory requires bounded private owned bytes",
                ));
            }
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            let inventory: pse_host::discovery::Inventory = serde_json::from_slice(&bytes)
                .map_err(|_| NativeServiceError::Refused("source inventory is invalid"))?;
            inventory.verify_executables().map_err(|_| {
                NativeServiceError::Refused("source inventory capture is incomplete or changed")
            })?;
            let selected = inventory
                .groups
                .get(group)
                .ok_or(NativeServiceError::Refused(
                    "source inventory group is absent",
                ))?;
            let parent = inventory_path.parent().ok_or(NativeServiceError::Refused(
                "source inventory has no capture directory",
            ))?;
            let capture = parent.join(format!("group-{group}"));
            let binaries = capture.join("binaries.json");
            let cargo = capture.join("cargo-metadata.json");
            let native: serde_json::Value = serde_json::from_slice(&fs::read(&binaries)?)
                .map_err(|_| NativeServiceError::Refused("captured binary metadata is invalid"))?;
            let expected = serde_json::to_value(&selected.binaries)
                .map_err(|_| NativeServiceError::Refused("source binary inventory is invalid"))?;
            if native != expected || !directory.is_absolute() || !directory.is_dir() {
                return Err(NativeServiceError::Refused(
                    "captured binary metadata or working directory differs",
                ));
            }
            let metadata: serde_json::Value = serde_json::from_slice(&fs::read(&cargo)?)
                .map_err(|_| NativeServiceError::Refused("captured Cargo metadata is invalid"))?;
            if metadata
                .get("workspace_root")
                .and_then(serde_json::Value::as_str)
                != directory.to_str()
            {
                return Err(NativeServiceError::Refused(
                    "captured Cargo workspace differs from selected checkout",
                ));
            }
            for argument in &arguments {
                let value = argument.to_string_lossy();
                if [
                    "--binaries-metadata",
                    "--cargo-metadata",
                    "--archive-file",
                    "--manifest-path",
                    "--package",
                    "--workspace",
                    "--all",
                    "--lib",
                    "--bins",
                    "--tests",
                    "--all-targets",
                    "--features",
                    "--all-features",
                    "--no-default-features",
                    "--no-run",
                ]
                .iter()
                .any(|flag| value == *flag || value.starts_with(&format!("{flag}=")))
                    || value == "-p"
                {
                    return Err(NativeServiceError::Refused(
                        "captured fixture accepts run policy, not new build selectors",
                    ));
                }
            }
            Ok(Self {
                program: inventory.nextest.captured,
                binaries,
                cargo,
                arguments,
                directory,
                _built: None,
            })
        }
        /// Create a fresh physical owner only after reuse inputs have been admitted.
        /// Every terminal child outcome attempts original service drain and disposal.
        /// # Errors
        /// Startup, child execution or any independent settlement fails.
        pub async fn run(
            self,
            spec: NativeServiceSpec,
            execution: Duration,
            settlement: Duration,
            report_path: Option<PathBuf>,
        ) -> Result<ExitStatus, NativeServiceError> {
            if execution.is_zero() || settlement.is_zero() {
                return Err(NativeServiceError::Refused(
                    "native fixture requires positive execution and settlement allowances",
                ));
            }
            let start = Instant::now();
            let execution_end = start
                .checked_add(execution)
                .ok_or(NativeServiceError::Refused(
                    "fixture execution clock overflow",
                ))?;
            let final_end =
                execution_end
                    .checked_add(settlement)
                    .ok_or(NativeServiceError::Refused(
                        "fixture settlement clock overflow",
                    ))?;
            let mut service = NativeCanonicalService::new(spec)?;
            let premise = ChildExitPremise::default();
            let child_premise = premise.clone();
            let runner_exit = std::sync::Arc::new(std::sync::atomic::AtomicI32::new(i32::MIN));
            let observed_exit = runner_exit.clone();
            let outcome = service
                .run(final_end, |service| {
                    Box::pin(async move {
                        service
                            .launch(execution_end.min(Instant::now() + Duration::from_secs(90)))
                            .await?;
                        let selection = service.fixture_selection()?;
                        let mut command = tokio::process::Command::new(&self.program);
                        command
                            .current_dir(&self.directory)
                            .args(["nextest", "run", "--binaries-metadata"])
                            .arg(&self.binaries)
                            .arg("--cargo-metadata")
                            .arg(&self.cargo)
                            .args(&self.arguments)
                            .env(SELECTION_ENV, selection.descriptor())
                            .env("PSE_SURREAL_STATE", selection.state())
                            .env_remove("PSE_TEST_INVOCATION")
                            .env_remove("PSE_TEST_RESOURCE_PYTHON")
                            .env_remove("PSE_TEST_EXECUTION_PROFILE")
                            .env_remove("PSE_FIXTURE_CARGO_METADATA_CAPTURE")
                            .kill_on_drop(true);
                        let status = child_status(
                            command,
                            execution_end,
                            final_end,
                            service.state(),
                            child_premise,
                        )
                        .await?;
                        if let Some(exit) = status.code() {
                            observed_exit.store(exit, std::sync::atomic::Ordering::Release);
                        }
                        Ok(status)
                    })
                })
                .await;
            if !premise.permits_disposal() {
                let mut failures = vec![outcome.err().unwrap_or(NativeServiceError::Refused(
                    "cancelled test process cannot prove native joins",
                ))];
                if let Err(error) = publish_run_report(
                    report_path.as_deref(),
                    runner_exit.load(std::sync::atomic::Ordering::Acquire),
                    false,
                ) {
                    failures.push(error);
                }
                return Err(NativeServiceError::Settlement {
                    state: service.state().to_owned(),
                    failures,
                });
            }
            let disposal = service.dispose();
            let publication = publish_run_report(
                report_path.as_deref(),
                runner_exit.load(std::sync::atomic::Ordering::Acquire),
                outcome.is_ok() && disposal.is_ok(),
            );
            let result = match (outcome, disposal) {
                (Ok(status), Ok(())) => Ok(status),
                (Err(error), Ok(())) => Err(error),
                (Ok(_), Err(error)) => Err(error),
                (Err(error), Err(cleanup)) => Err(NativeServiceError::Settlement {
                    state: service.state().to_owned(),
                    failures: vec![error, cleanup],
                }),
            };
            match (result, publication) {
                (Ok(status), Ok(())) => Ok(status),
                (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
                (Err(error), Err(publication)) => Err(NativeServiceError::Settlement {
                    state: service.state().to_owned(),
                    failures: vec![error, publication],
                }),
            }
        }
    }
    fn publish_run_report(
        path: Option<&Path>,
        exit: i32,
        fixture_settled: bool,
    ) -> Result<(), NativeServiceError> {
        let Some(path) = path else {
            return Ok(());
        };
        let report = pse_host::fixtures::CanonicalRunReport {
            version: 1,
            runner_exit: (exit != i32::MIN).then_some(exit),
            fixture_settled,
        };
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)?;
        serde_json::to_writer(&mut output, &report)
            .map_err(|_| NativeServiceError::Refused("fixture outcome encoding failed"))?;
        output.sync_all()?;
        if let Some(parent) = path.parent() {
            fs::File::open(parent)?.sync_all()?;
        }
        Ok(())
    }
    #[cfg(test)]
    mod tests {
        #![allow(
            clippy::unwrap_used,
            clippy::panic,
            reason = "native child settlement controls require exact terminal status"
        )]
        use super::*;
        #[test]
        fn fixture_report_keeps_native_success_distinct_from_owner_failure() {
            let directory = tempfile::tempdir().unwrap();
            let path = directory.path().join("outcome.json");
            publish_run_report(Some(&path), 0, false).unwrap();
            let report: pse_host::fixtures::CanonicalRunReport =
                serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            assert_eq!(report.runner_exit, Some(0));
            assert!(!report.fixture_settled);
            assert!(
                publish_run_report(Some(&path), 0, true).is_err(),
                "original outcome cannot be overwritten"
            );
        }
        #[test]
        fn captured_inventory_refuses_before_service_creation() {
            let directory = tempfile::tempdir().unwrap();
            let inventory = directory.path().join("inventory.json");
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&inventory)
                .unwrap();
            file.write_all(b"{}").unwrap();
            let error = NativeTestCommand::from_source_capture(
                &inventory,
                0,
                Vec::new(),
                directory.path().to_owned(),
            )
            .unwrap_err();
            assert!(matches!(
                error,
                NativeServiceError::Refused("source inventory is invalid")
            ));
            assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
        }
        #[tokio::test]
        async fn unsuccessful_test_status_is_retained_after_actual_child_exit() {
            let mut command = tokio::process::Command::new("/bin/sh");
            command.args(["-c", "exit 7"]);
            let premise = ChildExitPremise::default();
            let deadline = Instant::now() + Duration::from_secs(5);
            let status = child_status(
                command,
                deadline,
                deadline,
                Path::new("/fixture-test"),
                premise.clone(),
            )
            .await
            .unwrap();
            assert_eq!(status.code(), Some(7));
            assert!(premise.permits_disposal());
        }
        #[tokio::test]
        async fn expired_child_refuses_before_any_process_effect() {
            let directory = tempfile::tempdir().unwrap();
            let witness = directory.path().join("started");
            let mut command = tokio::process::Command::new("/bin/sh");
            command
                .args(["-c", "touch -- \"$1\"", "fixture"])
                .arg(&witness);
            let deadline = Instant::now() - Duration::from_millis(1);
            let premise = ChildExitPremise::default();
            assert!(
                child_status(
                    command,
                    deadline,
                    deadline + Duration::from_secs(5),
                    directory.path(),
                    premise.clone()
                )
                .await
                .is_err()
            );
            assert!(!witness.exists());
            assert!(premise.permits_disposal());
        }
        #[tokio::test]
        async fn timeout_kills_and_joins_original_child() {
            let mut command = tokio::process::Command::new("/bin/sh");
            command.args(["-c", "exec sleep 30"]);
            let premise = ChildExitPremise::default();
            let deadline = Instant::now() + Duration::from_millis(40);
            assert!(
                child_status(
                    command,
                    deadline,
                    deadline + Duration::from_secs(5),
                    Path::new("/fixture-test"),
                    premise.clone()
                )
                .await
                .is_err()
            );
            assert!(!premise.permits_disposal());
        }
        #[tokio::test]
        async fn signal_registration_error_settles_child_and_retains_unknown_scientific_joins() {
            let mut command = tokio::process::Command::new("/bin/sh");
            command.args(["-c", "exec sleep 30"]);
            let premise = ChildExitPremise::default();
            let deadline = Instant::now() + Duration::from_secs(5);
            let error = child_status_with_signal(
                command,
                deadline,
                deadline,
                Path::new("/fixture-test"),
                premise.clone(),
                std::future::ready(Err(std::io::Error::other(
                    "injected signal registration failure",
                ))),
            )
            .await
            .unwrap_err();
            assert!(!premise.permits_disposal());
            let NativeServiceError::Settlement { failures, .. } = error else {
                panic!("all original child settlement errors must be retained")
            };
            assert!(
                failures
                    .iter()
                    .any(|error| error.to_string().contains("injected signal"))
            );
            assert_eq!(
                failures.len(),
                1,
                "successful original kill+wait adds no invented failure"
            );
        }
        #[tokio::test]
        async fn signal_terminated_child_never_establishes_scientific_join_disposal() {
            let mut command = tokio::process::Command::new("/bin/sh");
            command.args(["-c", "kill -TERM $$"]);
            let premise = ChildExitPremise::default();
            let deadline = Instant::now() + Duration::from_secs(5);
            assert!(
                child_status(
                    command,
                    deadline,
                    deadline,
                    Path::new("/fixture-test"),
                    premise.clone()
                )
                .await
                .is_err()
            );
            assert!(!premise.permits_disposal());
        }
        #[tokio::test]
        async fn waiter_error_settlement_attempts_original_child_join_without_disposal_premise() {
            let mut command = tokio::process::Command::new("/bin/sh");
            command.args(["-c", "exec sleep 30"]);
            let mut child = command.kill_on_drop(true).spawn().unwrap();
            let premise = ChildExitPremise::default();
            premise.spawned();
            let error = settle_child(
                &mut child,
                Instant::now() + Duration::from_secs(5),
                Path::new("/fixture-test"),
                vec![std::io::Error::other("injected original waiter failure").into()],
            )
            .await;
            assert!(
                child.id().is_none(),
                "the same original process must actually be reaped"
            );
            assert!(!premise.permits_disposal());
            let NativeServiceError::Settlement { failures, .. } = error else {
                panic!("original waiter error must be retained")
            };
            assert_eq!(failures.len(), 1);
            assert!(
                failures[0]
                    .to_string()
                    .contains("injected original waiter failure")
            );
        }
    }
}
#[cfg(feature = "native-service")]
pub use command::NativeTestCommand;

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "private fixture identity controls fail on unexpected filesystem outcomes"
    )]
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    fn fixture() -> (tempfile::TempDir, FixtureSelection) {
        let directory = tempfile::tempdir().unwrap();
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let metadata = fs::metadata(directory.path()).unwrap();
        let selection = FixtureSelection {
            version: 1,
            state: directory.path().canonicalize().unwrap(),
            device: metadata.dev(),
            inode: metadata.ino(),
            nonce: "original-private-nonce".into(),
        };
        let marker = selection.state.join("disposable.json");
        fs::write(
            &marker,
            br#"{"version":1,"nonce":"original-private-nonce"}"#,
        )
        .unwrap();
        fs::set_permissions(marker, fs::Permissions::from_mode(0o600)).unwrap();
        let path = selection.descriptor();
        fs::write(&path, serde_json::to_vec(&selection).unwrap()).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
        (directory, selection)
    }
    #[test]
    fn explicit_borrow_selection_pins_original_directory_and_marker() {
        let (_directory, selection) = fixture();
        assert_eq!(
            FixtureSelection::read(&selection.descriptor())
                .unwrap()
                .state(),
            selection.state()
        );
        fs::write(
            selection.state.join("disposable.json"),
            br#"{"version":1,"nonce":"changed"}"#,
        )
        .unwrap();
        assert!(FixtureSelection::read(&selection.descriptor()).is_err());
    }
    #[test]
    fn descriptor_copies_symlinks_and_public_bytes_are_not_admitted() {
        let (directory, selection) = fixture();
        let copy = directory.path().join("copied.json");
        fs::copy(selection.descriptor(), &copy).unwrap();
        assert!(FixtureSelection::read(&copy).is_err());
        let link = directory.path().join("link.json");
        std::os::unix::fs::symlink(selection.descriptor(), &link).unwrap();
        assert!(FixtureSelection::read(&link).is_err());
        fs::set_permissions(selection.descriptor(), fs::Permissions::from_mode(0o644)).unwrap();
        assert!(FixtureSelection::read(&selection.descriptor()).is_err());
    }
}
