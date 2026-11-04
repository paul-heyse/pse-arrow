// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Compose three original native reports while the existing keeper retains its fixture.
use super::{Error, Failures, WorkerStart, signals, worker_allocation};
use clap::Parser;
use futures_util::FutureExt as _;
use pse_host::{
    discovery::{BuildCapture, Executable},
    envelope::{CliInvocationId, Completeness, Settlement},
    invocation::InvocationStore,
    managed::result::ManagedSummary,
    paths::CheckoutState,
    test_workflow::execute::{Action, EvidenceScope, Outcome},
};
use pse_runtime::{
    CancelSource,
    host_operations::engineering::{EngineeringFixtureReady, EngineeringFrontendReady},
};
use std::{
    ffi::OsString,
    fs::{File, OpenOptions},
    io::Read as _,
    os::unix::fs::{DirBuilderExt as _, MetadataExt as _, OpenOptionsExt as _},
    panic::AssertUnwindSafe,
    path::{Path, PathBuf},
    process::Stdio,
    time::{Duration, Instant},
};

const FILTER: &str =
    "test(/^study::prepared_engineering_study_client_three_phase_original_owner_journey$/)";
const FEATURES: &str = "engineering-service,native-study-tests,product-operators,solver-kinsol";

#[derive(Debug, Parser)]
#[command(
    about = "Qualify the original Study through submit, frontend replacement and completion; prepare nothing implicitly",
    long_about = "Run this coordination mode from the application shell, outside a native workload. The keeper uses the existing foreground boundary; the host owns frontend and test invocations. Inputs are completed source-built capture manifests, a prepared module and a validated scientific allocation."
)]
pub(super) struct Args {
    #[arg(long)]
    root: PathBuf,
    /// Existing native BuildCapture containing keeper, frontend and pse-worker.
    #[arg(long)]
    runtime_build: PathBuf,
    /// Existing native BuildCapture containing pse-canonical-service.
    #[arg(long)]
    canonical_build: PathBuf,
    #[arg(long)]
    module_receipt: PathBuf,
    #[arg(long)]
    worker_allocation: PathBuf,
    /// Fresh absolute private diagnostic directory; original reports stay in the host store.
    #[arg(long)]
    work_directory: PathBuf,
    #[arg(long, default_value_t = 120_000)]
    startup_ms: u64,
    #[arg(long, default_value_t = 600_000)]
    phase_ms: u64,
    #[arg(long, default_value_t = 90_000)]
    shutdown_ms: u64,
}

#[derive(Debug)]
pub(super) struct Failure {
    pub(super) interrupted: bool,
    failures: Failures,
}
impl std::fmt::Display for Failure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.interrupted {
            write!(
                formatter,
                "qualification interrupted after settlement attempts; "
            )?;
        }
        self.failures.fmt(formatter)
    }
}
impl std::error::Error for Failure {}

#[derive(Debug)]
struct NativeWaitFailure {
    invocation: CliInvocationId,
    failures: Vec<pse_host::envelope::ErrorRecord>,
}
impl std::fmt::Display for NativeWaitFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "original native wait {} reported incomplete or failed settlement",
            self.invocation
        )?;
        for failure in &self.failures {
            write!(formatter, "; {}", failure.message)?;
        }
        Ok(())
    }
}
impl std::error::Error for NativeWaitFailure {}

fn settled_original(
    id: &CliInvocationId,
    observed: &pse_host::envelope::Envelope<serde_json::Value>,
    original: &pse_host::envelope::Envelope<ManagedSummary>,
) -> Result<(), Error> {
    if observed.payload.get("original_invocation") != Some(&serde_json::to_value(id)?)
        || original.invocation_id != *id
        || observed.settlement != Settlement::Complete
        || observed.observation.completeness != Completeness::Complete
        || original.observation.completeness != Completeness::Complete
        || original.payload.child_exit != Some(0)
        || !observed.failures.is_empty()
    {
        return Err(Box::new(NativeWaitFailure {
            invocation: id.clone(),
            failures: observed.failures.clone(),
        }));
    }
    // The historical outcome may be Interrupted. Only this current native wait
    // establishes drain; its original report/outcome is never rewritten.
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Submit,
    Recover,
    Complete,
}
impl Phase {
    fn name(self) -> &'static str {
        match self {
            Self::Submit => "submit",
            Self::Recover => "recover",
            Self::Complete => "complete",
        }
    }
}

// The sequence owns only composition. Each implementation retains its performing
// owners even if a step fails after dispatch or loses its acknowledgement.
trait Driver {
    async fn keeper(&mut self) -> Result<(), Error>;
    async fn frontend(&mut self, replacement: bool) -> Result<(), Error>;
    async fn phase(&mut self, phase: Phase) -> Result<(), Error>;
    async fn gate(&mut self) -> Result<(), Error>;
    async fn retire_frontends_and_tests(&mut self) -> Result<(), Error>;
    async fn finish_keeper(&mut self) -> Result<(), Error>;
}
fn checkpoint(stop: &CancelSource) -> Result<(), Error> {
    if stop.token().is_cancelled() {
        return Err("qualification stop requested".into());
    }
    Ok(())
}
async fn stop_cli(child: &mut tokio::process::Child, allowance: Duration) -> Vec<Error> {
    let mut failures: Vec<Error> = Vec::new();
    if let Err(error) = child.start_kill() {
        failures.push(error.into());
    }
    match tokio::time::timeout(allowance, child.wait()).await {
        Ok(Ok(_)) => {}
        Ok(Err(error)) => failures.push(error.into()),
        Err(_) => failures.push(
            "CLI child join deadline expired; original managed effect remains unresolved".into(),
        ),
    }
    failures
}
async fn sequence(driver: &mut impl Driver, stop: &CancelSource) -> Result<(), Error> {
    checkpoint(stop)?;
    driver.keeper().await?;
    checkpoint(stop)?;
    driver.frontend(false).await?;
    checkpoint(stop)?;
    driver.phase(Phase::Submit).await?;
    driver.retire_frontends_and_tests().await?;
    checkpoint(stop)?;
    driver.frontend(true).await?;
    checkpoint(stop)?;
    driver.phase(Phase::Recover).await?;
    checkpoint(stop)?;
    driver.gate().await?;
    checkpoint(stop)?;
    driver.phase(Phase::Complete).await
}
async fn execute(driver: &mut impl Driver, stop: &CancelSource) -> Result<(), Error> {
    let action = AssertUnwindSafe(sequence(driver, stop))
        .catch_unwind()
        .await;
    let (primary, panic) = match action {
        Ok(result) => (result.err(), None),
        Err(panic) => (None, Some(panic)),
    };
    // Both teardowns are required independently; cancellation is repropagated only afterward.
    let frontends = AssertUnwindSafe(driver.retire_frontends_and_tests())
        .catch_unwind()
        .await;
    let keeper = AssertUnwindSafe(driver.finish_keeper())
        .catch_unwind()
        .await;
    let mut causes = Vec::new();
    if let Some(error) = primary {
        causes.push(error);
    }
    let mut panic = panic;
    for cleanup in [frontends, keeper] {
        match cleanup {
            Ok(Ok(())) => {}
            Ok(Err(error)) => causes.push(error),
            Err(payload) => {
                causes.push("independent cleanup panicked".into());
                if panic.is_none() {
                    panic = Some(payload);
                }
            }
        }
    }
    if let Some(panic) = panic {
        eprintln!(
            "Study qualification panic; independent settlement failures: {}",
            Failures(causes)
        );
        std::panic::resume_unwind(panic);
    }
    if !causes.is_empty() || stop.token().is_cancelled() {
        return Err(Box::new(Failure {
            interrupted: stop.token().is_cancelled(),
            failures: Failures(causes),
        }));
    }
    Ok(())
}

fn read(path: &Path, maximum: usize) -> Result<Vec<u8>, Error> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags((rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK).bits() as i32)
        .open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.uid() != rustix::process::geteuid().as_raw() {
        return Err("qualification input is not an original user-owned regular file".into());
    }
    let mut bytes = Vec::new();
    file.take(
        u64::try_from(maximum)?
            .checked_add(1)
            .ok_or("input bound overflow")?,
    )
    .read_to_end(&mut bytes)?;
    if bytes.len() > maximum {
        return Err("qualification input exceeds its byte bound".into());
    }
    Ok(bytes)
}
fn image(
    capture: &BuildCapture,
    root: &Path,
    package: &str,
    name: &str,
    source: &str,
    required_features: &[&str],
) -> Result<Executable, Error> {
    if capture.version != 1 || capture.cargo_exit != 0 || !capture.incomplete.is_empty() {
        return Err("source build or original capture is incomplete".into());
    }
    let selected = capture
        .artifacts
        .iter()
        .filter(|artifact| {
            artifact.cargo.target.name == name
                && !artifact.cargo.profile.test
                && artifact
                    .cargo
                    .target
                    .kind
                    .iter()
                    .any(|kind| kind.to_string() == "bin")
        })
        .collect::<Vec<_>>();
    let [artifact] = selected.as_slice() else {
        return Err("capture must select exactly one requested binary".into());
    };
    if artifact.cargo.target.src_path.as_std_path() != root.join(source)
        || !required_features.iter().all(|required| {
            artifact
                .cargo
                .features
                .iter()
                .any(|feature| feature == required)
        })
        || !capture.resolution.units.iter().any(|unit| {
            unit.root
                && unit.package_name == package
                && unit.package_id == artifact.cargo.package_id.repr
                && unit.target_name == name
                && unit.mode == "build"
                && unit.source_path == artifact.cargo.target.src_path.as_str()
        })
        || artifact
            .cargo
            .executable
            .as_ref()
            .is_none_or(|path| path.as_std_path() != artifact.executable.original)
    {
        return Err("captured image differs from its actual native source/build context".into());
    }
    artifact.executable.verify()?;
    Ok(artifact.executable.clone())
}

fn frontend_identity(
    ready: &EngineeringFrontendReady,
    root: &Path,
    invocation: &CliInvocationId,
    socket: &Path,
    image: &pse_buildinfo::FileObservation,
    build: pse_ids::ContentHash,
    previous: Option<&CliInvocationId>,
) -> Result<(), Error> {
    if ready.root != root
        || &ready.invocation != invocation
        || ready.socket != socket
        || ready.process == 0
        || ready.build != build
        || ready.artifact != *image
        || previous == Some(invocation)
    {
        return Err("frontend readiness differs from original managed image/identity".into());
    }
    Ok(())
}
struct Managed {
    session: String,
    id: Option<CliInvocationId>,
    settled: bool,
}
struct Native {
    args: Args,
    keeper_image: Executable,
    frontend_image: Executable,
    worker_image: Executable,
    canonical_image: Executable,
    store: InvocationStore,
    stop: CancelSource,
    keeper: Option<tokio::process::Child>,
    keeper_bytes: Option<Vec<u8>>,
    ready: Option<EngineeringFixtureReady>,
    original_frontend: Option<CliInvocationId>,
    endpoint: Option<PathBuf>,
    managed: Vec<Managed>,
    serial: usize,
    cleaning: bool,
    completed: bool,
}
impl Native {
    fn path(&self, name: &str) -> PathBuf {
        self.args.work_directory.join(name)
    }
    fn duration(&self) -> Duration {
        Duration::from_millis(self.args.startup_ms)
    }
    fn log(&self, name: &str) -> Result<File, Error> {
        Ok(OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(self.path(name))?)
    }
    async fn cli(
        &mut self,
        arguments: Vec<OsString>,
        timeout: Duration,
        require_success: bool,
    ) -> Result<serde_json::Value, Error> {
        self.serial = self
            .serial
            .checked_add(1)
            .ok_or("command ordinal overflow")?;
        let prefix = format!("command-{}", self.serial);
        let output = self.path(&format!("{prefix}.json"));
        let mut child = tokio::process::Command::new(self.args.root.join("scripts/pse"))
            .current_dir(&self.args.root)
            .args(arguments)
            .stdout(Stdio::from(self.log(&format!("{prefix}.json"))?))
            .stderr(Stdio::from(self.log(&format!("{prefix}.stderr"))?))
            .kill_on_drop(false)
            .spawn()?;
        let waited = tokio::select! {
            status = child.wait() => Some(status?),
            _ = tokio::time::sleep(timeout) => None,
            _ = self.stop.cancelled(), if !self.cleaning => None,
        };
        let Some(status) = waited else {
            let mut failures: Vec<Error> = vec!["qualification CLI observation interrupted or expired; original managed owner still requires settlement".into()];
            failures
                .extend(stop_cli(&mut child, Duration::from_millis(self.args.shutdown_ms)).await);
            return Err(Box::new(Failures(failures)));
        };
        let value = serde_json::from_slice(&read(&output, 16 << 20)?)?;
        if require_success && !status.success() {
            return Err(format!(
                "native command exited {status}; original output {}",
                output.display()
            )
            .into());
        }
        Ok(value)
    }
    async fn await_file(&mut self, path: &Path) -> Result<Vec<u8>, Error> {
        let deadline = Instant::now()
            .checked_add(self.duration())
            .ok_or("startup clock overflow")?;
        loop {
            checkpoint(&self.stop)?;
            if let Some(child) = &mut self.keeper
                && child.try_wait()?.is_some()
            {
                return Err("original keeper exited before readiness".into());
            }
            match read(path, 1 << 20) {
                Ok(bytes) => return Ok(bytes),
                Err(error)
                    if error
                        .downcast_ref::<std::io::Error>()
                        .is_some_and(|error| error.kind() == std::io::ErrorKind::NotFound) => {}
                Err(error) => return Err(error),
            }
            if Instant::now() >= deadline {
                return Err("original readiness deadline expired".into());
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }
    fn unchanged_keeper(&mut self, open: bool) -> Result<(), Error> {
        if self.keeper_bytes.as_ref() != Some(&read(&self.path("keeper.json"), 1 << 20)?) {
            return Err("original keeper identity changed".into());
        }
        if self.path("start.json").try_exists()? != open
            || self.path("started.json").try_exists()? != open
        {
            return Err("original worker gate/acknowledgement is in the wrong phase".into());
        }
        if let Some(child) = &mut self.keeper
            && child.try_wait()?.is_some()
        {
            return Err("original keeper lifetime ended".into());
        }
        Ok(())
    }
    fn issue(&mut self) -> Result<usize, Error> {
        let session = format!("engineering-study-{}", CliInvocationId::new());
        if self.store.resolve("latest", Some(&session))?.is_some() {
            return Err("qualification session already exists".into());
        }
        let index = self.managed.len();
        self.managed.push(Managed {
            session,
            id: None,
            settled: false,
        });
        Ok(index)
    }
    fn identify(&mut self, index: usize) -> Result<CliInvocationId, Error> {
        let action = &mut self.managed[index];
        let id = self
            .store
            .resolve("latest", Some(&action.session))?
            .ok_or("original managed invocation was never published")?;
        if action.id.as_ref().is_some_and(|original| original != &id) {
            return Err("original managed identity changed".into());
        }
        action.id = Some(id.clone());
        Ok(id)
    }
    async fn wait_original(
        &mut self,
        id: &CliInvocationId,
        timeout: Duration,
    ) -> Result<(), Error> {
        let seconds = timeout.as_secs().saturating_add(1).to_string();
        let observed = self
            .cli(
                vec![
                    "wait".into(),
                    "--format".into(),
                    "json".into(),
                    "--timeout".into(),
                    seconds.into(),
                    id.as_str().into(),
                ],
                timeout.saturating_add(Duration::from_secs(5)),
                false,
            )
            .await?;
        let observed: pse_host::envelope::Envelope<serde_json::Value> =
            serde_json::from_value(observed)?;
        let original = self
            .store
            .result::<serde::de::IgnoredAny, ManagedSummary>(id)?
            .ok_or("original managed result unavailable after wait")?;
        settled_original(id, &observed, &original)
    }
}

impl Driver for Native {
    async fn keeper(&mut self) -> Result<(), Error> {
        for image in [
            &self.keeper_image,
            &self.worker_image,
            &self.canonical_image,
        ] {
            image.verify()?;
        }
        let lifetime = self
            .args
            .phase_ms
            .checked_mul(3)
            .and_then(|n| n.checked_add(self.args.startup_ms.checked_mul(4)?))
            .and_then(|n| n.checked_add(self.args.shutdown_ms.checked_mul(3)?))
            .ok_or("keeper lifetime overflow")?;
        let mut command = tokio::process::Command::new(self.args.root.join("scripts/pse-env"));
        command
            .current_dir(&self.args.root)
            .args(["--native=compiler,klu", "--"])
            .arg(&self.keeper_image.captured)
            .arg("--service-helper")
            .arg(&self.canonical_image.captured)
            .arg("--module-receipt")
            .arg(&self.args.module_receipt)
            .arg("--study-worker")
            .arg(&self.worker_image.captured)
            .arg("--worker-allocation")
            .arg(&self.args.worker_allocation)
            .arg("--study-worker-start")
            .arg(self.path("start.json"))
            .arg("--study-worker-started")
            .arg(self.path("started.json"))
            .arg("--finish-request")
            .arg(self.path("finish.json"))
            .arg("--dispose-request")
            .arg(self.path("dispose.json"))
            .arg("--state-parent")
            .arg(&self.args.work_directory)
            .arg("--ready")
            .arg(self.path("keeper.json"))
            .arg("--startup-ms")
            .arg(self.args.startup_ms.to_string())
            .arg("--lifetime-ms")
            .arg(lifetime.to_string())
            .arg("--shutdown-ms")
            .arg(self.args.shutdown_ms.to_string())
            .stdout(Stdio::from(self.log("keeper.stdout")?))
            .stderr(Stdio::from(self.log("keeper.stderr")?))
            .kill_on_drop(false);
        self.keeper = Some(command.spawn()?);
        let bytes = self.await_file(&self.path("keeper.json")).await?;
        let ready: EngineeringFixtureReady = serde_json::from_slice(&bytes)?;
        let _version = ready.version;
        let _selection = &ready.fixture;
        if !ready.state.is_absolute() || ready.database.is_empty() {
            return Err("keeper routing receipt is incomplete".into());
        }
        self.ready = Some(ready);
        self.keeper_bytes = Some(bytes);
        self.unchanged_keeper(false)
    }
    async fn frontend(&mut self, replacement: bool) -> Result<(), Error> {
        self.unchanged_keeper(false)?;
        self.frontend_image.verify()?;
        let ordinal = if replacement { "two" } else { "one" };
        let socket = self.path(&format!("frontend-{ordinal}.sock"));
        let ready = self.path(&format!("frontend-{ordinal}.json"));
        let spill = self.path(&format!("spill-{ordinal}"));
        std::fs::create_dir(&spill)?;
        let index = self.issue()?;
        let keeper = self.ready.as_ref().ok_or("keeper is not ready")?;
        self.cli(
            vec![
                "run".into(),
                "--detach".into(),
                "--format".into(),
                "json".into(),
                "--session".into(),
                self.managed[index].session.clone().into(),
                "--".into(),
                self.frontend_image.captured.as_os_str().to_owned(),
                "--root".into(),
                self.args.root.as_os_str().to_owned(),
                "--state".into(),
                keeper.state.as_os_str().to_owned(),
                "--database".into(),
                keeper.database.clone().into(),
                "--socket".into(),
                socket.as_os_str().to_owned(),
                "--ready".into(),
                ready.as_os_str().to_owned(),
                "--allow-uid".into(),
                rustix::process::geteuid().as_raw().to_string().into(),
                "--spill-dir".into(),
                spill.into_os_string(),
                "--shutdown-ms".into(),
                self.args.shutdown_ms.to_string().into(),
            ],
            self.duration(),
            true,
        )
        .await?;
        let id = self.identify(index)?;
        let observed: EngineeringFrontendReady =
            serde_json::from_slice(&self.await_file(&ready).await?)?;
        let _version = observed.version;
        let image = pse_buildinfo::FileObservation::capture(&self.frontend_image.captured)?;
        let mut digest = pse_ids::EncodingHasher::new();
        std::io::copy(&mut File::open(&self.frontend_image.captured)?, &mut digest)?;
        let build = digest.finish().content_hash();
        if replacement && self.original_frontend.is_none() {
            return Err("replacement has no original frontend witness".into());
        }
        frontend_identity(
            &observed,
            &self.args.root,
            &id,
            &socket,
            &image,
            build,
            if replacement {
                self.original_frontend.as_ref()
            } else {
                None
            },
        )?;
        if !replacement {
            self.original_frontend = Some(id);
        }
        self.endpoint = Some(ready);
        Ok(())
    }
    async fn phase(&mut self, phase: Phase) -> Result<(), Error> {
        self.unchanged_keeper(phase == Phase::Complete)?;
        let index = self.issue()?;
        let session = self.managed[index].session.clone();
        let endpoint = self
            .endpoint
            .clone()
            .ok_or("original frontend endpoint missing")?;
        // Explicit environment only for this child; no process-global state is mutated.
        let mut command = tokio::process::Command::new(self.args.root.join("scripts/pse"));
        let prefix = format!("{}.start", phase.name());
        command
            .current_dir(&self.args.root)
            .args([
                "test",
                "pse-runtime:engineering_service",
                "--detach",
                "--format",
                "json",
                "--session",
                &session,
                "--features",
                FEATURES,
                "-E",
                FILTER,
            ])
            .env("PSE_ENGINEERING_ENDPOINT", endpoint)
            .env("PSE_ENGINEERING_KEEPER_READY", self.path("keeper.json"))
            .env("PSE_ENGINEERING_STUDY_GATE", self.path("start.json"))
            .env("PSE_ENGINEERING_WORKER_STARTED", self.path("started.json"))
            .env("PSE_ENGINEERING_STUDY_HANDOFF", self.path("handoff.json"))
            .env("PSE_ENGINEERING_STUDY_PHASE", phase.name())
            .stdout(Stdio::from(self.log(&format!("{prefix}.json"))?))
            .stderr(Stdio::from(self.log(&format!("{prefix}.stderr"))?))
            .kill_on_drop(false);
        let mut child = command.spawn()?;
        let status = tokio::select! {
            status = tokio::time::timeout(self.duration(), child.wait()) => Some(status),
            _ = self.stop.cancelled() => None,
        };
        match status {
            Some(Ok(Ok(status))) if status.success() => {}
            Some(Ok(Ok(status))) => {
                return Err(format!("native phase launch exited {status}").into());
            }
            Some(Ok(Err(error))) => return Err(error.into()),
            Some(Err(_)) | None => {
                let mut failures: Vec<Error> =
                    vec!["native phase launch observation interrupted or expired".into()];
                failures.extend(
                    stop_cli(&mut child, Duration::from_millis(self.args.shutdown_ms)).await,
                );
                return Err(Box::new(Failures(failures)));
            }
        }
        let id = self.identify(index)?;
        self.wait_original(&id, Duration::from_millis(self.args.phase_ms))
            .await?;
        let original = self
            .store
            .result::<serde::de::IgnoredAny, ManagedSummary>(&id)?
            .ok_or("phase result absent")?;
        let workflow = original
            .payload
            .workflow
            .ok_or("phase has no original native test report")?;
        workflow.terminal.validate(&id, Action::Test)?;
        if workflow.terminal.outcome != Outcome::TestsSucceeded
            || !workflow.terminal.required_capture_complete
        {
            return Err("phase did not complete its original native test population".into());
        }
        let page = pse_host::test_inventory::read(
            &self.store,
            id.as_str(),
            Some(&session),
            None,
            std::num::NonZeroUsize::new(2).ok_or("positive population window")?,
            std::num::NonZeroUsize::new(1 << 20).ok_or("positive population byte bound")?,
        )?;
        let [selected] = page.payload.records.as_slice() else {
            return Err("phase must select exactly one native test identity".into());
        };
        if page.payload.selected_tests != 1
            || page.payload.next.is_some()
            || selected.target != "engineering_service"
            || selected.identity.test_name
                != "study::prepared_engineering_study_client_three_phase_original_owner_journey"
        {
            return Err("phase selected another original native test population".into());
        }
        let output = self.path(&format!("{}.selection.bin", phase.name()));
        let export = pse_host::developer_evidence::export(
            &self.store,
            id.as_str(),
            Some(&session),
            &output,
            EvidenceScope::Selection,
        )?;
        use buffa::Message as _;
        let descriptors =
            buffa_descriptor::generated::descriptor::FileDescriptorSet::decode_from_slice(
                pse_host::developer_evidence_contracts::DESCRIPTORS,
            )?;
        let facts = pse_host::developer_evidence::inspect(
            &descriptors,
            &export.message,
            &read(&output, 64 << 20)?,
        )?;
        if facts.invocation != id
            || facts.selected_tests != Some(1)
            || facts.selected_population_complete != Some(true)
        {
            return Err("phase lacks the complete single original selected-test inventory".into());
        }
        self.managed[index].settled = true;
        self.unchanged_keeper(phase == Phase::Complete)?;
        if phase == Phase::Complete {
            self.completed = true;
        }
        Ok(())
    }
    async fn gate(&mut self) -> Result<(), Error> {
        self.unchanged_keeper(false)?;
        super::publish_worker_started(&self.path("start.json"))?;
        let acknowledgement = self.await_file(&self.path("started.json")).await?;
        let _: WorkerStart = serde_json::from_slice(&acknowledgement)?;
        self.unchanged_keeper(true)
    }
    async fn retire_frontends_and_tests(&mut self) -> Result<(), Error> {
        self.cleaning = true;
        let mut failures = Vec::new();
        for index in (0..self.managed.len()).rev() {
            if self.managed[index].settled {
                continue;
            }
            let id = match self.identify(index) {
                Ok(id) => id,
                Err(error) => {
                    failures.push(error);
                    continue;
                }
            };
            let timeout = Duration::from_millis(self.args.shutdown_ms);
            let seconds = timeout.as_secs().saturating_add(1).to_string();
            if let Err(error) = self
                .cli(
                    vec![
                        "cancel".into(),
                        "--format".into(),
                        "json".into(),
                        "--timeout".into(),
                        seconds.into(),
                        id.as_str().into(),
                    ],
                    timeout.saturating_add(Duration::from_secs(5)),
                    false,
                )
                .await
            {
                failures.push(error);
            }
            match self.wait_original(&id, timeout).await {
                Ok(()) => self.managed[index].settled = true,
                Err(error) => failures.push(error),
            }
        }
        self.cleaning = false;
        if failures.is_empty() {
            Ok(())
        } else {
            Err(Box::new(Failures(failures)))
        }
    }
    async fn finish_keeper(&mut self) -> Result<(), Error> {
        let Some(child) = self.keeper.as_mut() else {
            return Ok(());
        };
        if let Some(status) = child.try_wait()? {
            return if status.success() {
                Ok(())
            } else {
                Err(format!("keeper exited {status}; original fixture/logs retained").into())
            };
        }
        let mut failures = Vec::new();
        if self.completed
            && self.managed.iter().all(|owner| owner.settled)
            && let Err(error) = super::publish_worker_started(&self.path("dispose.json"))
        {
            failures.push(error);
        }
        if let Err(error) = super::publish_worker_started(&self.path("finish.json")) {
            failures.push(error);
        }
        let child = self.keeper.as_mut().ok_or("keeper owner missing")?;
        match tokio::time::timeout(Duration::from_millis(self.args.shutdown_ms), child.wait()).await
        {
            Ok(Ok(status)) if status.success() => {}
            Ok(Ok(status)) => failures
                .push(format!("keeper exited {status}; original fixture/logs retained").into()),
            Ok(Err(error)) => failures.push(error.into()),
            Err(_) => failures.push(
                "keeper settlement deadline expired; original foreground lifetime/fixture retained"
                    .into(),
            ),
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(Box::new(Failures(failures)))
        }
    }
}

pub(super) async fn run(mut args: Args) -> Result<(), Error> {
    args.root = args.root.canonicalize()?;
    if !args.work_directory.is_absolute()
        || args.work_directory.try_exists()?
        || args.startup_ms == 0
        || args.phase_ms == 0
        || args.shutdown_ms == 0
    {
        return Err(
            "qualification requires a fresh absolute directory and positive deadlines".into(),
        );
    }
    let environment = std::env::vars_os().collect();
    let manager = pse_host::HostManager::connect_user(None).await?;
    let unit = manager.current_unit().await?;
    if manager.native_workload(&unit).await? {
        return Err("qualification coordinator must run outside an enclosing native workload; each child has its own existing boundary".into());
    }
    let _module = pse_host::study_module::select(
        &args.module_receipt,
        pse_host::study_module::Variant::Production,
    )?;
    let _allocation = worker_allocation(&args.worker_allocation)?;
    let runtime: BuildCapture = serde_json::from_slice(&read(&args.runtime_build, 64 << 20)?)?;
    let canonical: BuildCapture = serde_json::from_slice(&read(&args.canonical_build, 64 << 20)?)?;
    let keeper_image = image(
        &runtime,
        &args.root,
        "pse-runtime",
        "pse-engineering-fixture",
        "crates/pse-runtime/src/bin/pse-engineering-fixture.rs",
        &["native-study-tests", "product-reader"],
    )?;
    let frontend_image = image(
        &runtime,
        &args.root,
        "pse-runtime",
        "pse-engineering",
        "crates/pse-runtime/src/bin/pse-engineering.rs",
        &["engineering-service", "product-reader", "solver-kinsol"],
    )?;
    let worker_image = image(
        &runtime,
        &args.root,
        "pse-runtime",
        "pse-worker",
        "crates/pse-runtime/src/bin/pse-worker.rs",
        &["product-operators", "solver-kinsol"],
    )?;
    let canonical_image = image(
        &canonical,
        &args.root,
        "pse-operations",
        "pse-canonical-service",
        "crates/pse-operations/src/bin/pse-canonical-service.rs",
        &["native-service"],
    )?;
    if std::env::current_exe()?.canonicalize()? != keeper_image.captured.canonicalize()? {
        return Err("qualification must execute its original captured fixture image, not a mutable Cargo output".into());
    }
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&args.work_directory)?;
    let stop = CancelSource::new();
    let signals = signals::ProcessInterrupt::new(stop.clone())?;
    let mut native = Native {
        store: InvocationStore::new(CheckoutState::resolve(&args.root, &environment)?),
        args,
        keeper_image,
        frontend_image,
        worker_image,
        canonical_image,
        stop: stop.clone(),
        keeper: None,
        keeper_bytes: None,
        ready: None,
        original_frontend: None,
        endpoint: None,
        managed: Vec::new(),
        serial: 0,
        cleaning: false,
        completed: false,
    };
    let result = execute(&mut native, &stop).await;
    signals.finish().await;
    eprintln!(
        "Study qualification original command outputs and handoff: {}",
        native.args.work_directory.display()
    );
    result
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "controlled qualification fixtures require valid local setup"
    )]
    use super::*;
    #[derive(Default)]
    struct Probe {
        calls: Vec<String>,
        fail: Option<&'static str>,
        cleanup_failure: bool,
        stop_at: Option<&'static str>,
        panic_at: Option<&'static str>,
        stop: CancelSource,
    }
    impl Probe {
        fn enter(&mut self, name: &'static str) -> Result<(), Error> {
            self.calls.push(name.into());
            assert_ne!(self.panic_at, Some(name), "original controlled panic");
            if self.stop_at == Some(name) {
                self.stop.cancel();
            }
            if self.fail == Some(name) {
                return Err(format!("original {name} failure").into());
            }
            Ok(())
        }
    }
    impl Driver for Probe {
        async fn keeper(&mut self) -> Result<(), Error> {
            self.enter("keeper")
        }
        async fn frontend(&mut self, replacement: bool) -> Result<(), Error> {
            self.enter(if replacement {
                "replacement"
            } else {
                "frontend"
            })
        }
        async fn phase(&mut self, phase: Phase) -> Result<(), Error> {
            self.enter(phase.name())
        }
        async fn gate(&mut self) -> Result<(), Error> {
            self.enter("gate")
        }
        async fn retire_frontends_and_tests(&mut self) -> Result<(), Error> {
            self.enter("retire")
        }
        async fn finish_keeper(&mut self) -> Result<(), Error> {
            self.enter("finish")?;
            if self.cleanup_failure {
                return Err("independent keeper release failure".into());
            }
            Ok(())
        }
    }
    #[tokio::test]
    async fn original_study_phase_order_places_recovery_before_worker_gate() {
        let mut driver = Probe::default();
        let stop = driver.stop.clone();
        execute(&mut driver, &stop).await.unwrap();
        assert_eq!(
            driver.calls,
            [
                "keeper",
                "frontend",
                "submit",
                "retire",
                "replacement",
                "recover",
                "gate",
                "complete",
                "retire",
                "finish"
            ]
        );
    }
    #[tokio::test]
    async fn failed_recovery_never_opens_gate_and_keeps_independent_cleanup_causes() {
        let mut driver = Probe {
            fail: Some("recover"),
            cleanup_failure: true,
            ..Default::default()
        };
        let stop = driver.stop.clone();
        let error = execute(&mut driver, &stop).await.unwrap_err();
        assert!(
            !driver
                .calls
                .iter()
                .any(|name| name == "gate" || name == "complete")
        );
        assert!(error.to_string().contains("original recover failure"));
        assert!(
            error
                .to_string()
                .contains("independent keeper release failure")
        );
        assert!(driver.calls.ends_with(&["retire".into(), "finish".into()]));
    }
    #[tokio::test]
    async fn interruption_settles_both_owners_before_repropagating_cancellation() {
        let mut driver = Probe {
            stop_at: Some("submit"),
            cleanup_failure: true,
            ..Default::default()
        };
        let stop = driver.stop.clone();
        let error = execute(&mut driver, &stop).await.unwrap_err();
        assert!(error.downcast_ref::<Failure>().unwrap().interrupted);
        assert!(!driver.calls.iter().any(|name| name == "replacement"));
        assert!(driver.calls.ends_with(&["retire".into(), "finish".into()]));
    }
    #[tokio::test]
    async fn panicking_frontend_cleanup_still_attempts_keeper_settlement() {
        let mut driver = Probe {
            fail: Some("submit"),
            panic_at: Some("retire"),
            ..Default::default()
        };
        let stop = driver.stop.clone();
        assert!(
            AssertUnwindSafe(execute(&mut driver, &stop))
                .catch_unwind()
                .await
                .is_err()
        );
        assert!(driver.calls.ends_with(&["retire".into(), "finish".into()]));
    }
    #[tokio::test]
    async fn interrupted_cli_child_is_actually_joined_before_observation_returns() {
        let directory = tempfile::tempdir().unwrap();
        let mut child = tokio::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "signals::tests::process_signals_request_stop_and_keep_drain_owned",
                "--nocapture",
            ])
            .env("PSE_SIGNAL_DRAIN_CONTROL", directory.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        tokio::time::timeout(Duration::from_secs(10), async {
            while !directory.path().join("ready").exists() {
                assert!(child.try_wait().unwrap().is_none());
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap();
        assert!(
            stop_cli(&mut child, Duration::from_secs(10))
                .await
                .is_empty()
        );
        assert!(child.try_wait().unwrap().is_some());
    }
    #[test]
    fn replacement_identity_cannot_reuse_original_invocation_or_changed_image() {
        let scratch = tempfile::tempdir().unwrap();
        let executable = scratch.path().join("image");
        std::fs::write(&executable, b"independent original frontend image").unwrap();
        let image = pse_buildinfo::FileObservation::capture(&executable).unwrap();
        let mut digest = pse_ids::EncodingHasher::new();
        std::io::copy(&mut File::open(&executable).unwrap(), &mut digest).unwrap();
        let build = digest.finish().content_hash();
        let original = CliInvocationId::new();
        let replacement = CliInvocationId::new();
        let socket = scratch.path().join("new.sock");
        let mut ready = EngineeringFrontendReady {
            version: Default::default(),
            root: scratch.path().to_owned(),
            invocation: replacement.clone(),
            socket: socket.clone(),
            process: 1,
            build,
            artifact: image.clone(),
        };
        assert!(
            frontend_identity(
                &ready,
                scratch.path(),
                &replacement,
                &socket,
                &image,
                build,
                Some(&original)
            )
            .is_ok()
        );
        assert!(
            frontend_identity(
                &ready,
                scratch.path(),
                &replacement,
                &socket,
                &image,
                build,
                Some(&replacement)
            )
            .is_err()
        );
        ready.invocation = original;
        assert!(
            frontend_identity(
                &ready,
                scratch.path(),
                &replacement,
                &socket,
                &image,
                build,
                None
            )
            .is_err()
        );
        ready.invocation = replacement.clone();
        ready.artifact.sha256 = "00".repeat(32);
        assert!(
            frontend_identity(
                &ready,
                scratch.path(),
                &replacement,
                &socket,
                &image,
                build,
                None
            )
            .is_err()
        );
        ready.artifact = image.clone();
        ready.build = pse_ids::ContentHash::NIL;
        assert!(
            frontend_identity(
                &ready,
                scratch.path(),
                &replacement,
                &socket,
                &image,
                build,
                None
            )
            .is_err()
        );
    }
    #[test]
    fn interrupted_historical_outcome_needs_current_drain_and_preserves_native_failures() {
        use pse_host::envelope::{CommandOutcome, Envelope, ErrorRecord, RedactionPolicy};
        let id = CliInvocationId::new();
        let mut original = Envelope::response(
            "run",
            "historical interrupted frontend",
            ManagedSummary {
                bundle: None,
                native_lifetime_pending: Some(true),
                workflow: None,
                child_exit: Some(0),
                stdout: None,
                stderr: None,
                borrowed_reservation: false,
                service_result: None,
            },
        );
        original.invocation_id = id.clone();
        original.outcome = CommandOutcome::Interrupted;
        original.settlement = Settlement::Unknown;
        let mut observed = Envelope::response(
            "wait",
            "current original native drain",
            serde_json::json!({"original_invocation":id}),
        );
        observed.settlement = Settlement::Complete;
        assert!(settled_original(&id, &observed, &original).is_ok());
        observed.settlement = Settlement::Unknown;
        assert!(settled_original(&id, &observed, &original).is_err());
        observed.settlement = Settlement::Complete;
        original.payload.child_exit = Some(3);
        assert!(settled_original(&id, &observed, &original).is_err());
        original.payload.child_exit = Some(0);
        observed.failures.push(ErrorRecord::from_typed(
            &pse_host::HostError::Ownership("independent native release failed".into()),
            &RedactionPolicy::from_environment(),
        ));
        let error = settled_original(&id, &observed, &original).unwrap_err();
        let retained = error.downcast_ref::<NativeWaitFailure>().unwrap();
        assert_eq!(retained.failures.len(), 1);
        assert!(
            error
                .to_string()
                .contains("independent native release failed")
        );
        assert_eq!(original.outcome, CommandOutcome::Interrupted);
        assert_eq!(original.settlement, Settlement::Unknown);
    }
}
