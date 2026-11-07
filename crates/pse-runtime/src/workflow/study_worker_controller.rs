// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Test-owned barriers at the actual managed native session entry.
#![allow(
    clippy::unwrap_used,
    reason = "the subprocess test reports any invalid handoff through its original test exit"
)]

#[path = "../bin/support/native-stack.rs"]
mod native_stack;
#[path = "../bin/support/signals.rs"]
mod signals;
#[path = "../bin/support/study-worker.rs"]
mod worker;

use super::WorkflowError;
use crate::CancelSource;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

pub(super) fn run() {
    use super::native_worker::{TEST_WORKER_CONTROLLER, TEST_WORKER_INPUT};
    worker::ensure_openmp_environment().unwrap();
    let directory = PathBuf::from(std::env::var_os(TEST_WORKER_CONTROLLER).unwrap());
    let cli = worker::Cli {
        native_input: Some(PathBuf::from(std::env::var_os(TEST_WORKER_INPUT).unwrap())),
        lease_seconds: worker::LEASE_SECONDS,
        heartbeat_ms: worker::HEARTBEAT_MS,
        ..Default::default()
    };
    let stack = native_stack::selected().unwrap();
    std::thread::Builder::new()
        .name("pse-study-test-worker-root".into())
        .stack_size(stack)
        .spawn(move || {
            native_stack::runtime(stack)
                .unwrap()
                .block_on(receive(cli, directory))
        })
        .unwrap()
        .join()
        .unwrap()
        .unwrap();
}

async fn receive(cli: worker::Cli, directory: PathBuf) -> Result<(), WorkflowError> {
    let stop = CancelSource::new();
    let interrupt = signals::ProcessInterrupt::new(stop.clone()).map_err(|error| {
        WorkflowError::Internal(format!("native worker stop listener: {error}"))
    })?;
    let (runtime, selected) = worker::runtime(&cli, &stop)
        .await
        .map_err(WorkflowError::Internal)?;
    let diagnostics = runtime.observations().clone();
    let result = async {
        let controller =
            Controller::open(&directory, &selected).map_err(WorkflowError::Internal)?;
        let observer = controller.clone();
        crate::math::install_qualified_native_entry(Arc::new(move |flag| observer.enter(flag)))?;
        selected.publish_ready(&runtime)?;
        let work = runtime.serve(selected.settings(), &stop);
        tokio::pin!(work);
        tokio::select! {
            result = &mut work => result,
            requested = controller.wait_stop() => {
                stop.cancel();
                worker::combine(work.await, requested.map_err(WorkflowError::Internal))
            }
        }
    }
    .await;
    stop.cancel();
    let result = worker::finish(&runtime, &selected, result).await;
    interrupt.finish().await;
    diagnostics.stop_publisher();
    result.map(|_| ())
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    nonce: String,
    canonical_database: String,
    entries: usize,
    entry_timeout_ms: u64,
}
#[derive(Default)]
struct Entries {
    owners: BTreeSet<String>,
    observations: Vec<serde_json::Value>,
    active: usize,
    maximum: usize,
}
struct Controller {
    directory: PathBuf,
    request: Request,
    fallback_deadline: Instant,
    entries: Mutex<Entries>,
}
fn private_metadata(path: &Path, directory: bool) -> Result<std::fs::Metadata, String> {
    use std::os::unix::fs::MetadataExt;
    let metadata = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    let owner = std::fs::metadata("/proc/self")
        .map_err(|error| error.to_string())?
        .uid();
    if metadata.file_type().is_symlink()
        || metadata.uid() != owner
        || metadata.mode() & 0o077 != 0
        || if directory {
            !metadata.is_dir()
        } else {
            !metadata.is_file() || metadata.len() > 8192
        }
    {
        return Err(
            "qualification control must be private, bounded and owned by the receiver user".into(),
        );
    }
    Ok(metadata)
}
fn bounded_json(path: &Path) -> Result<Vec<u8>, String> {
    private_metadata(path, false)?;
    std::fs::read(path).map_err(|error| error.to_string())
}
fn write_private(path: &Path, value: &impl serde::Serialize) -> Result<(), String> {
    use std::{io::Write, os::unix::fs::OpenOptionsExt};
    let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    if bytes.len() > 8192 {
        return Err("qualification observation extent".into());
    }
    let temporary = path.with_extension(format!(
        "{}-{:?}-temporary",
        std::process::id(),
        std::thread::current().id()
    ));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)
        .map_err(|error| error.to_string())?;
    let result = (|| {
        file.write_all(&bytes).map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
        std::fs::rename(&temporary, path).map_err(|error| error.to_string())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}
impl Controller {
    fn open(
        directory: &Path,
        selected: &pse_runtime::workflow::NativeStudyWorkerInputs,
    ) -> Result<Arc<Self>, String> {
        let state = std::fs::canonicalize(selected.state()).map_err(|error| error.to_string())?;
        private_metadata(directory, true)?;
        let directory = std::fs::canonicalize(directory).map_err(|error| error.to_string())?;
        if directory.parent() != Some(state.as_path()) {
            return Err("qualification control must be beneath the selected private state".into());
        }
        let profile = selected
            .allocation()
            .execution
            .ok_or("qualification requires the selected reference profile")?;
        if profile.pool_memory_bytes != 128 << 30
            || profile.cpu_threads != 16
            || profile.case_lanes != 16
            || profile.math_jobs != 32
        {
            return Err("qualification requires the original 128 GiB sixteen-lane profile".into());
        }
        let request: Request =
            serde_json::from_slice(&bounded_json(&directory.join("request.json"))?)
                .map_err(|error| error.to_string())?;
        if !(1..=profile.case_lanes).contains(&request.entries)
            || request.nonce.len() != 32
            || !request.nonce.bytes().all(|byte| byte.is_ascii_hexdigit())
            || request.canonical_database != selected.database()
            || request.entry_timeout_ms == 0
            || request.entry_timeout_ms > 90_000
        {
            return Err(
                "qualification extent, original database association or deadline differs".into(),
            );
        }
        let startup = Duration::from_secs(45);
        let entry = Duration::from_secs(90);
        let operation = Duration::from_secs(90);
        let drain = Duration::from_secs(45);
        let fallback_deadline = Instant::now()
            // A crashed observer still has a finite receiver owner. Startup,
            // entry, public operation and drain keep their separate bounds;
            // startup cannot consume an actual native owner's entry clock.
            .checked_add(startup + entry + operation + drain)
            .ok_or("qualification deadline overflow")?;
        let controller = Arc::new(Self {
            directory,
            request,
            fallback_deadline,
            entries: Mutex::new(Entries::default()),
        });
        controller.snapshot(&Entries::default())?;
        Ok(controller)
    }
    fn snapshot(&self, entries: &Entries) -> Result<(), String> {
        write_private(
            &self.directory.join("entries.json"),
            &serde_json::json!({
                "nonce": self.request.nonce, "canonical_database": self.request.canonical_database,
                "pid": std::process::id(), "entries": entries.observations,
                "active": entries.active, "maximum": entries.maximum,
            }),
        )
    }
    fn signal(&self, name: &str) -> Result<bool, String> {
        let path = self.directory.join(name);
        match std::fs::symlink_metadata(&path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error.to_string()),
            Ok(_) => {
                let value: String = serde_json::from_slice(&bounded_json(&path)?)
                    .map_err(|error| error.to_string())?;
                if value != self.request.nonce {
                    return Err("qualification signal association differs".into());
                }
                Ok(true)
            }
        }
    }
    fn observe(&self, stop: &AtomicBool) -> Result<(), String> {
        let entry_deadline = Instant::now()
            .checked_add(Duration::from_millis(self.request.entry_timeout_ms))
            .ok_or("qualification native entry deadline overflow")?;
        let thread = format!("{:?}", std::thread::current().id());
        let ordinal = {
            let mut entries = self
                .entries
                .lock()
                .map_err(|_| "qualification observation lock poisoned")?;
            if entries.owners.contains(&thread) {
                return Ok(());
            }
            if entries.owners.len() >= self.request.entries {
                return Err("qualified native owner entry extent exceeded".into());
            }
            entries.owners.insert(thread.clone());
            let ordinal = entries.observations.len();
            entries.observations.push(
                serde_json::json!({ "thread": thread, "released": false, "stop_observed": false }),
            );
            entries.active += 1;
            entries.maximum = entries.maximum.max(entries.active);
            self.snapshot(&entries)?;
            ordinal
        };
        let waited = (|| {
            loop {
                if stop.load(Ordering::Acquire) {
                    return Ok(true);
                }
                if self.signal("release.json")? {
                    return Ok(false);
                }
                if Instant::now() >= entry_deadline {
                    return Err("qualification native entry release deadline expired".into());
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        })();
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| "qualification observation lock poisoned")?;
        entries.active -= 1;
        entries.observations[ordinal]["released"] = serde_json::json!(true);
        entries.observations[ordinal]["stop_observed"] =
            serde_json::json!(waited.as_ref().is_ok_and(|value| *value));
        self.snapshot(&entries)?;
        waited.map(|_| ())
    }
    fn enter(&self, stop: &AtomicBool) {
        if let Err(error) = self.observe(stop) {
            stop.store(true, Ordering::Release);
            let _ = write_private(&self.directory.join("failure.json"), &error);
        }
    }
    async fn wait_stop(&self) -> Result<(), String> {
        loop {
            if self.signal("stop.json")? {
                return Ok(());
            }
            if self.directory.join("failure.json").exists() {
                return Err(String::from_utf8_lossy(&bounded_json(
                    &self.directory.join("failure.json"),
                )?)
                .into_owned());
            }
            if Instant::now() >= self.fallback_deadline {
                return Err("qualification receiver stop deadline expired".into());
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }
}
