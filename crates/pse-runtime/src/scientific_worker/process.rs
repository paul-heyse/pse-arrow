// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact executable selection and child custody through actual reap.
use super::{WorkerError, infrastructure};
use pse_buildinfo::FileObservation;
use pse_columnar::PayloadOwner;
use pse_kernels::{ExecutionScope, ProviderError};
use std::{
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    sync::Arc,
    time::{Duration, Instant},
};

/// Explicit deployed image and honest opaque allocation accounting.
#[derive(Clone, Debug)]
pub struct PreparedExecutable {
    observation: FileObservation,
    opaque_bytes: usize,
    #[cfg(all(test, feature = "property-coolprop"))]
    test_entry: Option<(&'static str, String, PathBuf)>,
    #[cfg(all(test, feature = "property-coolprop"))]
    measurements: Option<Arc<std::sync::Mutex<Vec<ChildMeasurement>>>>,
}
impl PreparedExecutable {
    /// Observe an existing image; never build, download or search PATH.
    pub fn observe(path: impl AsRef<Path>, opaque_bytes: usize) -> Result<Self, WorkerError> {
        if opaque_bytes == 0 {
            return Err(ProviderError::Contract(
                "scientific worker opaque allowance is zero".into(),
            )
            .into());
        }
        Ok(Self {
            observation: FileObservation::capture(path.as_ref()).map_err(infrastructure)?,
            opaque_bytes,
            #[cfg(all(test, feature = "property-coolprop"))]
            test_entry: None,
            #[cfg(all(test, feature = "property-coolprop"))]
            measurements: None,
        })
    }
    /// Canonical explicitly selected image.
    pub fn canonical_path(&self) -> &Path {
        &self.observation.canonical
    }
    /// Exact byte observation bound at explicit preparation.
    pub fn observation(&self) -> &FileObservation {
        &self.observation
    }
    /// Accounted opaque/native allowance; this is no whole-process RSS bound.
    pub fn opaque_bytes(&self) -> usize {
        self.opaque_bytes
    }
    #[cfg(all(test, feature = "property-coolprop"))]
    pub(super) fn test_entry(mut self, entry: &'static str, mode: &str, marker: PathBuf) -> Self {
        self.test_entry = Some((entry, mode.into(), marker));
        self
    }
    #[cfg(all(test, feature = "property-coolprop"))]
    pub(crate) fn measured(
        mut self,
        observations: Arc<std::sync::Mutex<Vec<ChildMeasurement>>>,
    ) -> Self {
        self.measurements = Some(observations);
        self
    }

    /// Launch only after admission; the returned owner retains it until reap.
    pub fn launch(
        &self,
        endpoint: &Path,
        session: pse_ids::SemanticId,
        scope: &ExecutionScope,
        admission: Arc<dyn PayloadOwner>,
        require_complete_memory_bound: bool,
    ) -> Result<ProcessCustody, WorkerError> {
        scope.check()?;
        if require_complete_memory_bound {
            return Err(ProviderError::Contract(
                "scientific worker native allocator cannot establish a strict whole-child bound"
                    .into(),
            )
            .into());
        }
        let actual = FileObservation::capture_scoped(self.canonical_path(), &|| {
            scope
                .check()
                .map_err(|error| std::io::Error::other(error.to_string()))
        })
        .map_err(infrastructure)?;
        scope.check()?;
        if actual.sha256 != self.observation.sha256
            || actual.canonical != self.observation.canonical
        {
            return Err(infrastructure(
                "scientific worker executable changed after preparation",
            ));
        }
        let mut command = Command::new(self.canonical_path());
        #[cfg(all(test, feature = "property-coolprop"))]
        if let Some((entry, mode, marker)) = &self.test_entry {
            command
                .args(["--exact", entry, "--nocapture"])
                .env("PSE_SCIENCE_TEST_MODE", mode)
                .env("PSE_SCIENCE_TEST_MARKER", marker)
                .env("PSE_SCIENCE_TEST_ENDPOINT", endpoint)
                .env("PSE_SCIENCE_TEST_SESSION", session.to_string())
                .env("PSE_SCIENCE_TEST_PARENT", std::process::id().to_string());
        } else {
            command
                .arg("--scientific-worker")
                .arg(endpoint)
                .arg(session.to_string())
                .arg(std::process::id().to_string());
        }
        #[cfg(not(all(test, feature = "property-coolprop")))]
        command
            .arg("--scientific-worker")
            .arg(endpoint)
            .arg(session.to_string())
            .arg(std::process::id().to_string());
        let process = command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(infrastructure)?;
        Ok(ProcessCustody {
            process,
            reaped: false,
            admission: Some(admission),
            executable: self.observation.clone(),
            #[cfg(all(test, feature = "property-coolprop"))]
            measurement: self
                .measurements
                .clone()
                .map(|observations| PendingMeasurement {
                    observations,
                    identity: None,
                    peak_resident_bytes: None,
                    error: None,
                    after_close_ack: false,
                }),
        })
    }
}

/// Sole process owner; dropping the transport cannot release native admission.
#[derive(Debug)]
pub struct ProcessCustody {
    process: Child,
    reaped: bool,
    admission: Option<Arc<dyn PayloadOwner>>,
    executable: FileObservation,
    #[cfg(all(test, feature = "property-coolprop"))]
    measurement: Option<PendingMeasurement>,
}
impl ProcessCustody {
    #[cfg(feature = "property-coolprop")]
    pub(super) fn admission_guard(&self) -> Option<Arc<dyn PayloadOwner>> {
        self.admission.clone()
    }
    /// PID comes from the actual owned spawn, never from RPC metadata.
    pub fn pid(&self) -> u32 {
        self.process.id()
    }
    #[cfg(feature = "property-coolprop")]
    pub(super) fn check_running(&mut self) -> Result<(), WorkerError> {
        match self.process.try_wait().map_err(infrastructure)? {
            Some(status) => Err(infrastructure(format!(
                "scientific worker exited before readiness: {status}"
            ))),
            None => Ok(()),
        }
    }
    /// Verify the currently running executable against the prepared bytes.
    pub fn authenticate_image(&mut self) -> Result<(), WorkerError> {
        if self.reaped || self.process.try_wait().map_err(infrastructure)?.is_some() {
            return Err(infrastructure(
                "scientific worker exited before authentication",
            ));
        }
        let image = FileObservation::capture(&PathBuf::from(format!("/proc/{}/exe", self.pid())))
            .map_err(infrastructure)?;
        if image.sha256 != self.executable.sha256 || image.canonical != self.executable.canonical {
            return Err(infrastructure(
                "running scientific worker executable differs",
            ));
        }
        #[cfg(all(test, feature = "property-coolprop"))]
        if let Some(measurement) = &mut self.measurement {
            match pse_diagnostics::process_facts::ProcessReader::default()
                .process(self.process.id())
            {
                pse_diagnostics::process_facts::ProcessRead::Present(facts) => {
                    measurement.identity = Some(facts.identity)
                }
                other => {
                    measurement.error = Some(format!(
                        "authenticated child identity observation unavailable: {other:?}"
                    ))
                }
            }
        }
        Ok(())
    }
    /// Finite graceful allowance followed by unconditional owned kill and wait.
    /// Admission is released only after wait has observed actual process exit.
    pub fn drain(&mut self, grace: Duration) -> Result<ExitStatus, WorkerError> {
        let end = Instant::now()
            .checked_add(grace)
            .ok_or_else(|| infrastructure("scientific worker drain extent"))?;
        loop {
            if let Some(status) = self.process.try_wait().map_err(infrastructure)? {
                self.reaped = true;
                #[cfg(all(test, feature = "property-coolprop"))]
                self.finish_measurement(status);
                self.admission.take();
                return Ok(status);
            }
            if Instant::now() >= end {
                break;
            }
            std::thread::sleep(
                Duration::from_millis(2).min(end.saturating_duration_since(Instant::now())),
            );
        }
        // A concurrent natural exit may make kill fail. Wait still establishes
        // settlement and its actual outcome; do not release admission early.
        let killed = self.process.kill();
        let status = self
            .process
            .wait()
            .map_err(|error| infrastructure(format!("kill={killed:?}; wait={error}")))?;
        self.reaped = true;
        #[cfg(all(test, feature = "property-coolprop"))]
        self.finish_measurement(status);
        self.admission.take();
        Ok(status)
    }
    /// Retain the original dispatch/stop cause if cleanup also fails.
    pub fn settle_failure(&mut self, cause: WorkerError) -> WorkerError {
        match self.drain(Duration::from_millis(100)) {
            Ok(_) => cause,
            Err(cleanup) => WorkerError::Cleanup {
                cause: Box::new(cause),
                cleanup: cleanup.to_string(),
            },
        }
    }
}

#[cfg(all(test, feature = "property-coolprop"))]
#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct ChildMeasurement {
    pub(crate) pid: u32,
    pub(crate) identity: Option<pse_diagnostics::process_facts::ProcessIdentity>,
    /// Actual kernel VmHWM after native close acknowledgement and before transport shutdown.
    pub(crate) peak_resident_bytes_at_close: Option<u64>,
    pub(crate) after_close_ack: bool,
    pub(crate) exit_code: Option<i32>,
    pub(crate) signal: Option<i32>,
    pub(crate) error: Option<String>,
}
#[cfg(all(test, feature = "property-coolprop"))]
#[derive(Debug)]
struct PendingMeasurement {
    observations: Arc<std::sync::Mutex<Vec<ChildMeasurement>>>,
    identity: Option<pse_diagnostics::process_facts::ProcessIdentity>,
    peak_resident_bytes: Option<u64>,
    error: Option<String>,
    after_close_ack: bool,
}
#[cfg(all(test, feature = "property-coolprop"))]
impl ProcessCustody {
    pub(super) fn observe_close(&mut self, acknowledged: bool) {
        use pse_diagnostics::process_facts::{ProcessRead, ProcessReader};
        let pid = self.process.id();
        let Some(measurement) = &mut self.measurement else {
            return;
        };
        measurement.after_close_ack = acknowledged;
        let reader = ProcessReader::default();
        match (reader.process(pid), reader.status(pid), reader.process(pid)) {
            (
                ProcessRead::Present(before),
                ProcessRead::Present(status),
                ProcessRead::Present(after),
            ) if Some(&before.identity) == measurement.identity.as_ref()
                && before.identity == after.identity
                && status.pid == pid
                && status.parent_pid == std::process::id() =>
            {
                measurement.peak_resident_bytes = status.peak_resident_bytes;
            }
            other => {
                measurement.error = Some(format!(
                    "child close memory observation unavailable/changed: {other:?}"
                ))
            }
        }
    }
    fn finish_measurement(&mut self, status: ExitStatus) {
        use std::os::unix::process::ExitStatusExt as _;
        if let Some(measurement) = self.measurement.take() {
            measurement
                .observations
                .lock()
                // Recording an already-settled child's facts must not panic
                // during teardown. Keep the poison marker for the reader.
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(ChildMeasurement {
                    pid: self.process.id(),
                    identity: measurement.identity,
                    peak_resident_bytes_at_close: measurement.peak_resident_bytes,
                    after_close_ack: measurement.after_close_ack,
                    exit_code: status.code(),
                    signal: status.signal(),
                    error: measurement.error,
                });
        }
    }
}
impl Drop for ProcessCustody {
    fn drop(&mut self) {
        if !self.reaped {
            let _ = self.process.kill();
            // This destructor deliberately blocks: outstanding child work must
            // never outlive its native admission/opaque allocation ownership.
            if self.process.wait().is_ok() {
                self.reaped = true;
                self.admission.take();
            }
        }
    }
}

/// Child-side parent death guard; process termination contains indivisible FFI.
/// Normal shutdown joins this observer before returning from the executable.
#[cfg(feature = "property-coolprop")]
pub(super) struct ParentGuard {
    stopping: Arc<std::sync::atomic::AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}
#[cfg(feature = "property-coolprop")]
impl ParentGuard {
    /// Contain an uninterruptible foreign call after its authenticated parent
    /// connection is lost. Only this standalone child process owner may exit;
    /// the parent still observes and settles the actual process lifetime.
    #[allow(
        clippy::exit,
        reason = "the isolated process owner must terminate an abandoned indivisible native call without running destructors against it"
    )]
    pub(super) fn terminate_unsettled_native(self) -> ! {
        std::process::exit(125)
    }

    pub(super) fn start(parent: u32) -> Result<Self, WorkerError> {
        use std::sync::atomic::{AtomicBool, Ordering};
        let expected =
            rustix::process::Pid::from_raw(i32::try_from(parent).map_err(infrastructure)?)
                .ok_or_else(|| infrastructure("invalid scientific worker parent PID"))?;
        if rustix::process::getppid() != Some(expected) {
            return Err(infrastructure("scientific worker actual parent differs"));
        }
        let parent_fd = rustix::process::pidfd_open(expected, rustix::process::PidfdFlags::empty())
            .map_err(infrastructure)?;
        if rustix::process::getppid() != Some(expected) {
            return Err(infrastructure(
                "scientific worker parent departed during admission",
            ));
        }
        let stopping = Arc::new(AtomicBool::new(false));
        let stop = stopping.clone();
        let thread = std::thread::Builder::new()
            .name("pse-science-parent".into())
            .stack_size(2 << 20)
            .spawn(move || {
                let mut events = [rustix::event::PollFd::new(
                    &parent_fd,
                    rustix::event::PollFlags::IN,
                )];
                while !stop.load(Ordering::Acquire) {
                    let timeout = rustix::time::Timespec {
                        tv_sec: 0,
                        tv_nsec: 50_000_000,
                    };
                    match rustix::event::poll(&mut events, Some(&timeout)) {
                        Ok(0) => {}
                        Err(rustix::io::Errno::INTR) => {}
                        // Parent loss or an unavailable lifetime witness terminates
                        // the owned process, including any currently executing FFI.
                        Ok(_) | Err(_) => std::process::exit(125),
                    }
                }
            })
            .map_err(infrastructure)?;
        Ok(Self {
            stopping,
            thread: Some(thread),
        })
    }
}
#[cfg(feature = "property-coolprop")]
impl Drop for ParentGuard {
    fn drop(&mut self) {
        self.stopping
            .store(true, std::sync::atomic::Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
