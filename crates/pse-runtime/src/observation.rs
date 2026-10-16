// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive operation reads and independently published same-host diagnostic checkpoints.
//!
//! No object here owns a scientific cancellation lease, native handle or result permission.

use pse_diagnostics::observation::Clock;
use pse_diagnostics::observation::{
    ObservationContext, ObservationHub, OperationId, OperationReader, OperationSnapshot, Policy,
};
use serde::{Deserialize, Serialize};
#[path = "observation_target.rs"]
mod target;
use std::{
    future::Future,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread::JoinHandle,
    time::{Duration, SystemTime},
};
pub use target::{ClockMapping, ProcessLocator, ThreadExecution};

const IMAGE_BYTES: usize = 1 << 20;
const METADATA_BYTES: usize = 16 * 1024;
const PUBLICATION_INTERVAL: Duration = Duration::from_secs(1);

/// Diagnostic-only checkpoint failure; it supplies no scientific conclusion.
#[derive(Debug, thiserror::Error)]
pub enum ObservationError {
    /// The optional checkpoint could not be read or written.
    #[error("diagnostic checkpoint I/O: {0}")]
    Io(#[from] std::io::Error),
    /// A bounded checkpoint was malformed or exceeded its declared format.
    #[error("diagnostic checkpoint format: {0}")]
    Format(String),
}
pse_diagnostics::impl_diagnostic! {
    ObservationError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}

/// One immutable process publication, never a scientific result or native restart state.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProcessCheckpoint {
    /// Format version of this diagnostic-only representation.
    pub version: u32,
    /// Unique process-instance locator; PID alone is never its identity.
    pub process: String,
    /// Monotonic publication sequence within this process instance.
    pub sequence: u64,
    /// Publication wall timestamp, distinct from each producer's checkpoint time.
    pub published_unix_micros: u128,
    /// Current/cumulative/recent facts, including their own scopes and coverage.
    pub operations: Vec<OperationSnapshot>,
    /// Operations omitted from this bounded external image.
    pub omitted: usize,
    /// Registry refusal/loss, including operations that could not obtain diagnostic storage.
    pub registry_coverage: pse_diagnostics::observation::Coverage,
    /// Process-wide samples, never a private operation's peak or exclusive CPU time.
    pub resources: ProcessResources,
    /// A parent diagnostic operation, when this process was explicitly dispatched by it.
    pub parent: Option<OperationId>,
    /// Actual operational attachment locator, not build provenance.
    pub locator: ProcessLocator,
    /// Mapping to the same-host monotonic clock, with bounded read uncertainty.
    pub clock: Option<ClockMapping>,
    /// Bounded actual native-thread execution scopes.
    pub threads: Vec<ThreadExecution>,
    /// Thread scopes were displaced or could not be observed.
    pub threads_partial: bool,
}

/// Independently sampled process scope. Missing values are unavailable, never zero-filled.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ProcessResources {
    /// Current resident bytes at publication, including observer/library allocations.
    pub resident_bytes: Option<u64>,
    /// Process-lifetime resident high-water mark; it is not reset for an operation.
    pub lifetime_peak_resident_bytes: Option<u64>,
    /// Process-lifetime user CPU clock ticks, with no inferred frequency or operation owner.
    pub user_cpu_ticks: Option<u64>,
    /// Process-lifetime system CPU clock ticks.
    pub system_cpu_ticks: Option<u64>,
}
impl ProcessResources {
    fn sample() -> Self {
        let status = std::fs::read_to_string("/proc/self/status").ok();
        let bytes = |name: &str| {
            status.as_ref().and_then(|status| {
                status.lines().find_map(|line| {
                    let mut fields = line.strip_prefix(name)?.split_whitespace();
                    let value = fields.next()?.parse::<u64>().ok()?;
                    (fields.next() == Some("kB") && fields.next().is_none())
                        .then(|| value.checked_mul(1024))
                        .flatten()
                })
            })
        };
        let statistics = std::fs::read_to_string("/proc/self/stat").ok();
        let ticks = statistics
            .as_ref()
            .and_then(|statistics| statistics.rsplit_once(')'))
            .map(|(_, statistics)| statistics.split_whitespace().collect::<Vec<_>>());
        Self {
            resident_bytes: bytes("VmRSS:"),
            lifetime_peak_resident_bytes: bytes("VmHWM:"),
            user_cpu_ticks: ticks
                .as_ref()
                .and_then(|fields| fields.get(11)?.parse().ok()),
            system_cpu_ticks: ticks
                .as_ref()
                .and_then(|fields| fields.get(12)?.parse().ok()),
        }
    }
}

/// Process-owned registry. Readers hold only the low-level copied observation store.
#[derive(Debug)]
pub struct Diagnostics {
    hub: Arc<ObservationHub>,
    process: String,
    root: PathBuf,
    sequence: AtomicU64,
    publication: Mutex<()>,
    publisher: Mutex<Option<Publisher>>,
    writer_failure: Mutex<Option<String>>,
    registrations: Mutex<usize>,
    publishing: AtomicBool,
    final_publication: AtomicBool,
    parent: Mutex<Option<OperationId>>,
    clock: Arc<target::ProcessClock>,
    locator: ProcessLocator,
    threads: Mutex<Vec<ThreadExecution>>,
    threads_partial: AtomicBool,
}
#[derive(Debug)]
struct Publisher {
    stop: Arc<AtomicBool>,
    join: JoinHandle<()>,
}

impl Diagnostics {
    /// Construct a diagnostic owner without starting a writer or opening a scientific store.
    pub fn new(policy: Policy, root: PathBuf) -> Arc<Self> {
        // Retained facts and one read copy share the hub allowance. Keep space for
        // the bounded assembled image and the one candidate serialization buffer.
        let clock = Arc::new(target::ProcessClock::new());
        let hub = ObservationHub::with_clock(
            Policy {
                budget_bytes: policy
                    .budget_bytes
                    .saturating_sub(2 * IMAGE_BYTES + METADATA_BYTES),
                ..policy
            },
            clock.clone(),
        );
        let instance = hub.process_instance();
        let process = format!("{:016x}-{:016x}", instance[0], instance[1]);
        Arc::new(Self {
            hub,
            process,
            root,
            sequence: AtomicU64::new(0),
            publication: Mutex::new(()),
            publisher: Mutex::new(None),
            writer_failure: Mutex::new(None),
            registrations: Mutex::new(0),
            publishing: AtomicBool::new(false),
            final_publication: AtomicBool::new(false),
            parent: Mutex::new(None),
            clock,
            locator: ProcessLocator::current(),
            threads: Mutex::new(Vec::new()),
            threads_partial: AtomicBool::new(false),
        })
    }
    /// Shared pure observation registry; it has no runtime or scientific resource owner.
    pub fn hub(&self) -> &Arc<ObservationHub> {
        &self.hub
    }
    /// Copy the actual same-host process attachment locator.
    pub fn locator(&self) -> ProcessLocator {
        self.locator.clone()
    }
    /// Map the process observation clock to Linux monotonic time.
    pub fn clock_mapping(&self) -> Option<ClockMapping> {
        self.clock.mapping()
    }
    /// Register one actual native-thread scope without taking native ownership.
    pub(crate) fn execution(
        self: &Arc<Self>,
        operation: Option<OperationId>,
    ) -> Option<ThreadScope> {
        let operation = operation?;
        let Some((tid, start_ticks)) = target::thread_identity() else {
            self.threads_partial.store(true, Ordering::Relaxed);
            return None;
        };
        let started_at = self.clock.now();
        let Ok(mut threads) = self.threads.try_lock() else {
            self.threads_partial.store(true, Ordering::Relaxed);
            return None;
        };
        if threads.len() >= 32 {
            if let Some(index) = threads.iter().position(|entry| entry.finished_at.is_some()) {
                threads.remove(index);
            } else {
                self.threads_partial.store(true, Ordering::Relaxed);
                return None;
            }
            self.threads_partial.store(true, Ordering::Relaxed);
        }
        threads.push(ThreadExecution {
            operation,
            tid,
            start_ticks,
            started_at,
            finished_at: None,
        });
        Some(ThreadScope {
            diagnostics: self.clone(),
            operation,
            tid,
            started_at,
        })
    }
    /// Register an actual runtime owner; diagnostic readers never acquire this token.
    pub(crate) fn register(self: &Arc<Self>) -> DiagnosticRegistration {
        let mut registrations = self
            .registrations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *registrations == 0 {
            self.publishing.store(true, Ordering::Release);
            self.start_publisher();
        }
        *registrations = registrations.saturating_add(1);
        DiagnosticRegistration(self.clone())
    }
    /// Register the actual native-child process and its diagnostic-only continuation.
    #[cfg(any(all(unix, feature = "solver-kinsol"), test))]
    pub(crate) fn register_child(
        self: &Arc<Self>,
        parent: Option<OperationId>,
    ) -> DiagnosticRegistration {
        *self
            .parent
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = parent;
        self.register()
    }
    /// Non-owning reader for an exact operation, including preparation before a run exists.
    pub fn reader(&self, id: OperationId) -> Option<OperationReader> {
        self.hub.reader(id)
    }
    /// Exact path written by this owner. Never infer cleanup authority from its age.
    pub fn checkpoint_path(&self) -> PathBuf {
        self.root.join(format!("{}.json", self.process))
    }
    /// Latest optional writer failure, independent of scientific completion.
    pub fn writer_failure(&self) -> Option<String> {
        self.writer_failure
            .lock()
            .ok()
            .and_then(|value| value.clone())
    }
    /// Publish already-observed state. This runs outside numerical callbacks.
    pub fn publish(&self) -> Result<(), ObservationError> {
        let _publication = self
            .publication
            .lock()
            .map_err(|_| ObservationError::Format("publisher lock poisoned".into()))?;
        let mut page = self.hub.list(0, usize::MAX);
        // An old completed operation must never displace newer currently active work.
        page.operations.sort_by_key(|operation| {
            (
                operation.finished_at.is_some(),
                std::cmp::Reverse(operation.started_at),
                std::cmp::Reverse(operation.id.sequence),
            )
        });
        let mut image = ProcessCheckpoint {
            version: 2,
            process: self.process.clone(),
            sequence: self.sequence.fetch_add(1, Ordering::Relaxed),
            published_unix_micros: unix_micros(),
            operations: Vec::new(),
            omitted: 0,
            registry_coverage: page.coverage,
            resources: ProcessResources::sample(),
            locator: self.locator.clone(),
            clock: self.clock.mapping(),
            threads: self
                .threads
                .lock()
                .map(|threads| threads.clone())
                .unwrap_or_default(),
            threads_partial: self.threads_partial.load(Ordering::Relaxed),
            parent: *self
                .parent
                .lock()
                .map_err(|_| ObservationError::Format("parent lock poisoned".into()))?,
        };
        // Encode each candidate at most twice; avoid repeated whole-image serialization under pressure.
        // Reserve metadata independently so refusal cannot overrun the image bound.
        let mut operations = Vec::with_capacity(IMAGE_BYTES - METADATA_BYTES);
        let mut candidate = BoundedImage {
            bytes: Vec::with_capacity(IMAGE_BYTES - METADATA_BYTES),
            limit: 0,
        };
        for operation in page.operations {
            let Some(reader) = self.hub.reader(operation.id) else {
                image.omitted = image.omitted.saturating_add(1);
                continue;
            };
            let remaining = IMAGE_BYTES.saturating_sub(METADATA_BYTES + operations.len() + 1);
            let mut sample = reader.snapshot(Duration::from_secs(60));
            candidate.limit = remaining;
            candidate.bytes.clear();
            if serde_json::to_writer(&mut candidate, &sample).is_err() {
                // Keep current/cumulative facts when rich recent point/detail history cannot
                // fit this transport. Local readers still retain the full bounded history.
                sample.coverage.curtailed = true;
                sample.coverage.sampled = true;
                sample.detail_partial = true;
                sample.coverage.dropped_detail = sample
                    .coverage
                    .dropped_detail
                    .saturating_add(sample.detail.len() as u64);
                sample.detail.clear();
                for metric in &mut sample.metrics {
                    metric.history_partial = true;
                    metric.history_dropped = metric
                        .history_dropped
                        .saturating_add(metric.history.len() as u64);
                    metric.history.clear();
                    metric.counter_window = None;
                }
                candidate.bytes.clear();
                if serde_json::to_writer(&mut candidate, &sample).is_err() {
                    image.omitted = image.omitted.saturating_add(1);
                    continue;
                }
            }
            if !operations.is_empty() {
                operations.push(b',');
            }
            operations.extend_from_slice(&candidate.bytes);
        }
        drop(candidate);
        let metadata = serde_json::to_vec(&image)
            .map_err(|error| ObservationError::Format(error.to_string()))?;
        if metadata.len() > METADATA_BYTES {
            return Err(ObservationError::Format(
                "checkpoint metadata exceeds diagnostic bound".into(),
            ));
        }
        let marker = b"\"operations\":[]";
        let position = metadata
            .windows(marker.len())
            .position(|part| part == marker)
            .ok_or_else(|| ObservationError::Format("checkpoint operation field missing".into()))?
            + marker.len()
            - 1;
        let mut writer = BoundedImage {
            bytes: Vec::with_capacity(IMAGE_BYTES),
            limit: IMAGE_BYTES,
        };
        writer.write_all(&metadata[..position])?;
        writer.write_all(&operations)?;
        writer.write_all(&metadata[position..])?;
        let bytes = writer.bytes;
        std::fs::create_dir_all(&self.root)?;
        let path = self.checkpoint_path();
        let temporary = path.with_extension("pending");
        {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&temporary)?;
            file.write_all(&bytes)?;
        }
        std::fs::rename(temporary, path)?;
        Ok(())
    }
    /// Start one independent publisher. Failure is reported diagnostically, never as solve failure.
    pub fn start_publisher(self: &Arc<Self>) {
        self.publishing.store(true, Ordering::Release);
        let Ok(mut publisher) = self.publisher.lock() else {
            return;
        };
        if publisher.is_some() {
            return;
        }
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let owner = Arc::downgrade(self);
        match std::thread::Builder::new()
            .name("pse-observations".into())
            .spawn(move || {
                while !stopped.load(Ordering::Acquire) {
                    let Some(owner) = owner.upgrade() else {
                        break;
                    };
                    if (owner.publishing.load(Ordering::Acquire)
                        || owner.final_publication.swap(false, Ordering::AcqRel))
                        && let Err(error) = owner.publish()
                        && let Ok(mut failure) = owner.writer_failure.lock()
                    {
                        *failure = Some(error.to_string());
                    }
                    drop(owner);
                    std::thread::park_timeout(PUBLICATION_INTERVAL);
                }
            }) {
            Ok(join) => {
                *publisher = Some(Publisher { stop, join });
            }
            Err(error) => {
                if let Ok(mut failure) = self.writer_failure.lock() {
                    *failure = Some(error.to_string());
                }
            }
        }
    }
    /// Stop and join this diagnostic publisher. Scientific owners are unaffected.
    pub fn stop_publisher(&self) {
        self.publishing.store(false, Ordering::Release);
        let publisher = self
            .publisher
            .lock()
            .ok()
            .and_then(|mut publisher| publisher.take());
        if let Some(publisher) = publisher {
            publisher.stop.store(true, Ordering::Release);
            publisher.join.thread().unpark();
            let _joined = publisher.join.join();
            if let Err(error) = self.publish()
                && let Ok(mut failure) = self.writer_failure.lock()
            {
                *failure = Some(error.to_string());
            }
        }
    }
    /// Release only this exact owner's files, after its publisher has joined.
    pub fn release(&self) -> Result<(), ObservationError> {
        let registrations = self
            .registrations
            .lock()
            .map_err(|_| ObservationError::Format("registration lock poisoned".into()))?;
        if *registrations != 0 {
            return Err(ObservationError::Format(
                "diagnostic owners are still registered".into(),
            ));
        }
        self.stop_publisher();
        for path in [
            self.checkpoint_path(),
            self.checkpoint_path().with_extension("pending"),
        ] {
            match std::fs::remove_file(path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct DiagnosticRegistration(Arc<Diagnostics>);
impl Drop for DiagnosticRegistration {
    fn drop(&mut self) {
        let mut registrations = self
            .0
            .registrations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *registrations = registrations.saturating_sub(1);
        if *registrations == 0 {
            // Optional I/O never joins or publishes while a scientific caller holds its lock.
            self.0.publishing.store(false, Ordering::Release);
            self.0.final_publication.store(true, Ordering::Release);
            if let Ok(publisher) = self.0.publisher.lock()
                && let Some(publisher) = publisher.as_ref()
            {
                publisher.join.thread().unpark();
            }
        }
    }
}

struct BoundedImage {
    bytes: Vec<u8>,
    limit: usize,
}
impl Write for BoundedImage {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other("diagnostic image bound"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Read an already-published bounded image while its scientific process remains occupied.
pub fn read_checkpoint(path: &Path) -> Result<ProcessCheckpoint, ObservationError> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take((IMAGE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > IMAGE_BYTES {
        return Err(ObservationError::Format(
            "checkpoint exceeds diagnostic bound".into(),
        ));
    }
    #[derive(Deserialize)]
    struct Header {
        version: u32,
    }
    let header: Header = serde_json::from_slice(&bytes)
        .map_err(|error| ObservationError::Format(error.to_string()))?;
    if header.version != 2 {
        return Err(ObservationError::Format(
            "unsupported diagnostic checkpoint version".into(),
        ));
    }
    let image: ProcessCheckpoint = serde_json::from_slice(&bytes)
        .map_err(|error| ObservationError::Format(error.to_string()))?;
    if image.version != 2 {
        return Err(ObservationError::Format(
            "unsupported diagnostic checkpoint version".into(),
        ));
    }
    Ok(image)
}

fn unix_micros() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros()
}

/// The process's one default lightweight diagnostic registry.
pub fn process_diagnostics() -> &'static Arc<Diagnostics> {
    static PROCESS: OnceLock<Arc<Diagnostics>> = OnceLock::new();
    PROCESS.get_or_init(|| {
        let root = crate::inspection::default_directory();
        let policy = Policy {
            enabled: std::env::var("PSE_DIAGNOSTICS").as_deref() != Ok("off"),
            ..Policy::default()
        };
        Diagnostics::new(policy, root)
    })
}

/// Poll with the supplied diagnostic context; no thread-local guard crosses an await.
pub(crate) fn scope<F: Future>(
    context: Option<ObservationContext>,
    future: F,
) -> impl Future<Output = F::Output> {
    pse_diagnostics::observation::scope(context, future)
}
/// Capture context before spawning, independently of scientific continuation ownership.
pub(crate) fn inherit<F: Future>(future: F) -> impl Future<Output = F::Output> {
    scope(pse_diagnostics::observation::current(), future)
}

/// A real asynchronous producer scope, distinct from its caller's waiting/accounting scope.
pub(crate) fn child<F: Future>(kind: &'static str, future: F) -> impl Future<Output = F::Output> {
    let operation = pse_diagnostics::observation::current()
        .map(|context| context.child(kind, pse_diagnostics::observation::Relationship::Nested));
    let context = operation.as_ref().map(|operation| operation.context());
    let future = futures_util::FutureExt::inspect(future, move |_| {
        // Completed describes this observed scope, not scientific permission of its output.
        if let Some(operation) = operation {
            operation.finish(pse_diagnostics::observation::Outcome::Completed);
        }
    });
    scope(context, future)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checkpoint_preserves_actual_thread_entry_and_return() {
        let directory = tempfile::tempdir().unwrap();
        let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
        let operation = owner.hub().start("native.thread");
        let scope = owner.execution(Some(operation.context().id())).unwrap();
        owner.publish().unwrap();
        let active = read_checkpoint(&owner.checkpoint_path()).unwrap();
        assert!(active.locator.is_current());
        assert!(active.clock.is_some());
        assert_eq!(active.threads.len(), 1);
        assert_eq!(active.threads[0].operation, operation.context().id());
        assert!(active.threads[0].start_ticks.is_some());
        assert!(active.threads[0].finished_at.is_none());
        drop(scope);
        owner.publish().unwrap();
        let returned = read_checkpoint(&owner.checkpoint_path()).unwrap();
        assert!(returned.threads[0].finished_at.unwrap() >= returned.threads[0].started_at);
        assert!(
            operation
                .reader()
                .snapshot(Duration::from_secs(60))
                .outcome
                .is_none(),
            "thread return must not create scientific outcome"
        );
        owner.release().unwrap();
    }
    #[test]
    fn checkpoint_reads_are_independent_and_bounded() {
        let directory = tempfile::tempdir().unwrap();
        let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
        let operation = owner.hub().start("busy.preparation");
        operation.context().count("constructed", "calls", 2);
        owner.publish().unwrap();
        let image = read_checkpoint(&owner.checkpoint_path()).unwrap();
        assert_eq!(image.operations.len(), 1);
        assert_eq!(image.omitted, 0);
        assert!(read_checkpoint(&owner.checkpoint_path()).is_ok());
        drop(image);
        assert!(owner.reader(operation.context().id()).is_some());
        owner.release().unwrap();
        assert!(!owner.checkpoint_path().exists());
    }
    #[test]
    fn checkpoint_failure_does_not_end_observed_work() {
        let directory = tempfile::tempdir().unwrap();
        let obstacle = directory.path().join("file");
        std::fs::write(&obstacle, b"not a directory").unwrap();
        let owner = Diagnostics::new(Policy::default(), obstacle);
        let operation = owner.hub().start("science");
        assert!(owner.publish().is_err());
        assert!(owner.reader(operation.context().id()).is_some());
    }
    #[test]
    fn checkpoint_pressure_preserves_active_work_before_completed_history() {
        let directory = tempfile::tempdir().unwrap();
        let owner = Diagnostics::new(
            Policy {
                budget_bytes: 64 * 1024 * 1024,
                ..Policy::default()
            },
            directory.path().to_owned(),
        );
        let root = owner.hub().start("oldest.active.root");
        let active = root.context().child(
            "latest.active.lane",
            pse_diagnostics::observation::Relationship::Nested,
        );
        active.context().count("actual.requests", "calls", 3);
        for _ in 0..110 {
            let completed = owner.hub().start("completed.history");
            for index in 0..64 {
                completed
                    .context()
                    .text(format!("field.{index}").as_str(), &"history".repeat(18));
            }
            completed.finish(pse_diagnostics::observation::Outcome::Completed);
        }
        owner.publish().unwrap();
        let image = read_checkpoint(&owner.checkpoint_path()).unwrap();
        assert!(
            image.omitted > 0,
            "fixture must actually reach the image bound"
        );
        assert!(
            image
                .operations
                .iter()
                .any(|operation| operation.id == active.context().id())
        );
        let retained_root = image
            .operations
            .iter()
            .find(|operation| operation.id == root.context().id())
            .unwrap();
        assert!(retained_root.outcome.is_none());
        assert_eq!(retained_root.children.len(), 1);
        assert_eq!(retained_root.children[0].id, active.context().id());
        assert!(std::fs::metadata(owner.checkpoint_path()).unwrap().len() <= IMAGE_BYTES as u64);
        owner.release().unwrap();
    }
    #[test]
    fn registered_child_publishes_while_its_owner_is_busy_and_readers_do_not_own_it() {
        let directory = tempfile::tempdir().unwrap();
        let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
        let parent_hub = ObservationHub::new(Policy::default());
        let parent = parent_hub.start("caller");
        let registration = owner.register_child(Some(parent.context().id()));
        let operation = owner.hub().start("busy.child");
        let phase = operation.context().phase("native.factor.open");
        let reader = operation.reader();
        let deadline = std::time::Instant::now() + Duration::from_secs(4);
        loop {
            if let Ok(image) = read_checkpoint(&owner.checkpoint_path())
                && image.operations.iter().any(|sample| {
                    sample.id == operation.context().id() && !sample.phases.is_empty()
                })
            {
                assert_eq!(image.parent, Some(parent.context().id()));
                assert_ne!(
                    owner.hub().process_instance(),
                    parent_hub.process_instance()
                );
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "independent publisher did not make progress"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(
            owner.release().is_err(),
            "registered scientific owner still active"
        );
        drop(phase);
        drop(operation);
        drop(registration);
        owner.release().unwrap();
        assert_eq!(
            reader.snapshot(Duration::from_secs(60)).outcome,
            Some(pse_diagnostics::observation::Outcome::Interrupted)
        );
        assert!(
            !owner.checkpoint_path().exists(),
            "reader must not keep publisher/files alive"
        );
    }
    #[test]
    fn last_registration_drop_does_not_wait_for_optional_publication() {
        let directory = tempfile::tempdir().unwrap();
        let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
        let registration = owner.register();
        // Simulate a writer occupied on optional I/O. Destruction must not try to join it.
        let publication = owner.publication.lock().unwrap();
        let (sent, received) = std::sync::mpsc::channel();
        let dropping = std::thread::spawn(move || {
            drop(registration);
            sent.send(()).unwrap();
        });
        received.recv_timeout(Duration::from_secs(2)).unwrap();
        drop(publication);
        dropping.join().unwrap();
        owner.release().unwrap();
    }
    #[tokio::test]
    async fn poll_scopes_restore_unrelated_context() {
        let owner = Diagnostics::new(Policy::default(), PathBuf::new());
        let first = owner.hub().start("first");
        let second = owner.hub().start("second");
        let first_context = first.context();
        let second_context = second.context();
        let (a, b) = tokio::join!(
            scope(Some(first_context.clone()), async {
                tokio::task::yield_now().await;
                pse_diagnostics::observation::current().unwrap().id()
            }),
            scope(Some(second_context.clone()), async {
                tokio::task::yield_now().await;
                pse_diagnostics::observation::current().unwrap().id()
            })
        );
        assert_eq!(a, first_context.id());
        assert_eq!(b, second_context.id());
        assert!(pse_diagnostics::observation::current().is_none());
    }
}

/// An observed execution interval; dropping it records return without owning native drain.
pub(crate) struct ThreadScope {
    diagnostics: Arc<Diagnostics>,
    operation: OperationId,
    tid: u32,
    started_at: Duration,
}
impl Drop for ThreadScope {
    fn drop(&mut self) {
        let Ok(mut threads) = self.diagnostics.threads.try_lock() else {
            self.diagnostics
                .threads_partial
                .store(true, Ordering::Relaxed);
            return;
        };
        if let Some(entry) = threads.iter_mut().find(|entry| {
            entry.operation == self.operation
                && entry.tid == self.tid
                && entry.started_at == self.started_at
        }) {
            entry.finished_at = Some(self.diagnostics.clock.now());
        } else {
            self.diagnostics
                .threads_partial
                .store(true, Ordering::Relaxed);
        }
    }
}
/// Enter an actual native-thread execution scope; this does not infer async thread affinity.
pub(crate) fn native_execution_scope() -> Option<ThreadScope> {
    let operation = pse_diagnostics::observation::current().map(|context| context.id());
    process_diagnostics().execution(operation)
}
