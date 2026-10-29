// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! User-manager operations with original unit and opened-cgroup ownership.
//!
//! Mutating capabilities cover freshly created private units with this controller as the
//! sole manager mutator. Concurrent external restart/reconfiguration is unsupported: systemd
//! has no atomic StopUnit-if-InvocationID operation. Arbitrary existing units are read-only.
use crate::HostError;
use futures_util::StreamExt;
use pse_diagnostics::process_facts::{ProcessIdentity, ProcessRead, ProcessReader};
use serde::Serialize;
use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    os::unix::fs::MetadataExt,
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use zbus::{Connection, OwnedObjectPath, OwnedVariant, Variant, proxy::CacheProperties};
use zbus_systemd::systemd1::{ManagerProxy, ScopeProxy, ServiceProxy, SliceProxy, UnitProxy};

static NEXT: AtomicU64 = AtomicU64::new(0);
const DESTINATION: &str = "org.freedesktop.systemd1";

/// Original intent, generated before the effect and retained for uncertain replies.
#[derive(Clone, Debug, Serialize)]
pub struct LaunchIntent {
    /// Unique unit name; not by itself permission to stop it.
    unit: String,
    /// Exact submitted description nonce.
    description: String,
}
impl LaunchIntent {
    /// Fresh native foreground scope identity.
    pub fn fresh_scope() -> Result<Self, HostError> {
        let mut value = Self::fresh()?;
        value.unit = value.unit.replace(".service", ".scope");
        Ok(value)
    }
    /// Original unique unit name, without construction authority.
    pub fn unit(&self) -> &str {
        &self.unit
    }
    /// Create a checkout-independent original launch identity without opening the manager.
    pub fn fresh() -> Result<Self, HostError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| HostError::Ownership(error.to_string()))?
            .as_nanos();
        let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
        let nonce = format!("{}-{now:x}-{sequence:x}", std::process::id());
        Ok(Self {
            unit: format!("pse-host-{nonce}.service"),
            description: format!("pse-host original {nonce}"),
        })
    }
}

/// Complete native service request. The executable is passed directly, without shell parsing.
#[derive(Clone, Debug)]
pub struct ServiceSpec {
    /// Original intent generated before dispatch.
    pub intent: LaunchIntent,
    /// Absolute executable path.
    pub executable: PathBuf,
    /// Arguments excluding argv[0].
    pub arguments: Vec<String>,
    /// Child environment assignments, already selected by the environment owner.
    pub environment: Vec<String>,
}

/// Observation only. Unknown properties are errors, never empty-success facts.
#[derive(Clone, Debug, Serialize)]
pub struct UnitObservation {
    /// Original name.
    pub unit: String,
    /// Manager incarnation's invocation identity.
    pub invocation: Vec<u8>,
    /// Actual manager active state.
    pub active_state: String,
    /// Actual manager sub-state.
    pub sub_state: String,
    /// Main process as observed, zero when absent.
    pub main_pid: u32,
    /// waitid-style original termination kind.
    pub exit_code: i32,
    /// Original exit status or signal according to exit_code.
    pub exit_status: i32,
    /// Manager-selected cgroup path.
    pub control_group: String,
}
impl UnitObservation {
    /// Original foreground status, only when the manager reports its main process ended.
    pub fn completed_status(&self) -> Option<i32> {
        if self.main_pid != 0
            || !(matches!(self.active_state.as_str(), "inactive" | "failed")
                || (self.active_state == "active" && self.sub_state == "exited"))
        {
            return None;
        }
        match self.exit_code {
            1 => Some(self.exit_status),
            2 | 3 => Some(128 + self.exit_status),
            _ => None,
        }
    }
}

/// A user-manager connection bound to the observed daemon owner and boot.
#[derive(Clone, Debug)]
pub struct HostManager {
    connection: Connection,
    owner: String,
    boot: String,
    manager_process: ProcessIdentity,
    cgroup_namespace: PathBuf,
    pid_namespace: String,
    manager_group: String,
    mount_device: String,
    cgroup_root: (u64, u64),
}
impl HostManager {
    /// Connect only to a user/session bus, or its explicitly selected address.
    pub async fn connect_user(address: Option<&str>) -> Result<Self, HostError> {
        tokio::time::timeout(Duration::from_secs(5), Self::connect_user_inner(address))
            .await
            .map_err(|_| HostError::Deadline("user-manager connection and identity".into()))?
    }
    async fn connect_user_inner(address: Option<&str>) -> Result<Self, HostError> {
        let connection = match address {
            Some(address) => zbus::connection::Builder::address(address).build().await?,
            None => Connection::session().await?,
        };
        let bus = zbus::fdo::DBusProxy::new(&connection).await?;
        let owner = bus
            .get_name_owner(DESTINATION.try_into()?)
            .await
            .map_err(zbus::Error::from)?
            .to_string();
        let manager_pid = bus
            .get_connection_unix_process_id(owner.as_str().try_into()?)
            .await
            .map_err(zbus::Error::from)?;
        let manager_uid = bus
            .get_connection_unix_user(owner.as_str().try_into()?)
            .await
            .map_err(zbus::Error::from)?;
        if manager_pid == 1 || manager_uid != rustix::process::geteuid().as_raw() {
            return Err(HostError::Ownership(
                "selected bus owner is not this user's manager".into(),
            ));
        }
        let reader = ProcessReader::default();
        let observer_namespace = match reader.observer_namespace() {
            ProcessRead::Present(namespace) => namespace,
            _ => {
                return Err(HostError::Ownership(
                    "observer PID namespace unavailable".into(),
                ));
            }
        };
        let boot = std::fs::read_to_string("/proc/sys/kernel/random/boot_id")?
            .trim()
            .to_owned();
        let manager_process = manager_process(manager_pid, &observer_namespace, &boot)?;
        let mount_device = cgroup_mount(&std::fs::read_to_string("/proc/self/mountinfo")?)?;
        let other_device = cgroup_mount(&std::fs::read_to_string(format!(
            "/proc/{manager_pid}/mountinfo"
        ))?)?;
        if mount_device != other_device {
            return Err(HostError::Ownership(
                "manager cgroup mount device differs".into(),
            ));
        }
        let manager = ManagerProxy::builder(&connection)
            .destination(owner.as_str())
            .build()
            .await?;
        let manager_scope = manager.get_unit_by_pid(manager_pid).await?;
        let scope = ScopeProxy::builder(&connection)
            .destination(owner.as_str())
            .path(manager_scope)
            .cache_properties(CacheProperties::No)
            .build()
            .await?;
        let manager_group = scope.control_group().await?;
        // Subscribe belongs to the bus connection, shared by concurrent owners.
        // Per-launch signal streams are still installed before each effect.
        manager.subscribe().await?;
        if manager_group != process_group(manager_pid)? {
            return Err(HostError::Ownership(
                "manager cgroup coordinates differ".into(),
            ));
        }
        let cgroup_namespace = std::fs::read_link("/proc/thread-self/ns/cgroup")?;
        let root = File::open("/sys/fs/cgroup")?;
        if rustix::fs::fstatfs(&root).map_err(io::Error::from)?.f_type != 0x6367_7270 {
            return Err(HostError::Ownership(
                "host cgroup root is not cgroup-v2".into(),
            ));
        }
        let meta = root.metadata()?;
        if mount_device
            != format!(
                "{}:{}",
                rustix::fs::major(meta.dev()),
                rustix::fs::minor(meta.dev())
            )
        {
            return Err(HostError::Ownership(
                "actual cgroup root device differs from mount metadata".into(),
            ));
        }
        let boot = manager_process.boot_id.clone();
        let pid_namespace = observer_namespace;
        let result = Self {
            connection,
            owner,
            boot,
            manager_process,
            cgroup_namespace,
            pid_namespace,
            manager_group,
            mount_device,
            cgroup_root: (meta.dev(), meta.ino()),
        };
        result.validate_view()?;
        Ok(result)
    }
    fn validate_view(&self) -> Result<(), HostError> {
        if self.connection.is_closed() {
            return Err(HostError::Ownership("original manager connection closed".into()));
        }
        if std::fs::read_link("/proc/thread-self/ns/cgroup")? != self.cgroup_namespace {
            return Err(HostError::Ownership(
                "observer cgroup namespace changed".into(),
            ));
        }
        let meta = std::fs::metadata("/sys/fs/cgroup")?;
        if (meta.dev(), meta.ino()) != self.cgroup_root
            || cgroup_mount(&std::fs::read_to_string("/proc/self/mountinfo")?)? != self.mount_device
            || cgroup_mount(&std::fs::read_to_string(format!(
                "/proc/{}/mountinfo",
                self.manager_process.pid
            ))?)?
                != self.mount_device
            || process_group(self.manager_process.pid)? != self.manager_group
        {
            return Err(HostError::Ownership(
                "manager/observer cgroup mount views changed".into(),
            ));
        }
        if manager_process(self.manager_process.pid, &self.pid_namespace, &self.boot)?
            != self.manager_process
        {
            return Err(HostError::Ownership(
                "user-manager process identity changed".into(),
            ));
        }
        Ok(())
    }
    async fn manager(&self) -> Result<ManagerProxy<'_>, HostError> {
        Ok(ManagerProxy::builder(&self.connection)
            .destination(self.owner.as_str())
            .build()
            .await?)
    }
    async fn unit(&self, name: &str) -> Result<UnitProxy<'_>, HostError> {
        let path = self.manager().await?.get_unit(name.to_owned()).await?;
        Ok(UnitProxy::builder(&self.connection)
            .destination(self.owner.as_str())
            .path(path)
            .cache_properties(CacheProperties::No)
            .build()
            .await?)
    }
    /// Obtain an observation-only handle. No stop permission is granted by a name.
    pub fn borrow(&self, name: impl Into<String>) -> BorrowedUnit {
        BorrowedUnit {
            manager: self.clone(),
            name: name.into(),
        }
    }
    /// Observe current authoritative manager facts without workload admission.
    pub async fn observe(&self, name: &str) -> Result<UnitObservation, HostError> {
        tokio::time::timeout(Duration::from_secs(5), self.observe_inner(name))
            .await
            .map_err(|_| HostError::Deadline("user-manager observation".into()))?
    }
    async fn observe_inner(&self, name: &str) -> Result<UnitObservation, HostError> {
        self.validate_view()?;
        if std::fs::read_to_string("/proc/sys/kernel/random/boot_id")?.trim() != self.boot {
            return Err(HostError::Ownership("kernel boot changed".into()));
        }
        let unit = self.unit(name).await?;
        let invocation = unit.invocation_id().await?;
        let (main_pid, exit_code, exit_status, control_group) = if name.ends_with(".scope") {
            let scope = ScopeProxy::builder(&self.connection)
                .destination(self.owner.as_str())
                .path(unit.inner().path().to_owned())
                .cache_properties(CacheProperties::No)
                .build()
                .await?;
            (0, 0, 0, scope.control_group().await?)
        } else {
            let service = ServiceProxy::builder(&self.connection)
                .destination(self.owner.as_str())
                .path(unit.inner().path().to_owned())
                .cache_properties(CacheProperties::No)
                .build()
                .await?;
            (
                service.main_pid().await?,
                service.exec_main_code().await?,
                service.exec_main_status().await?,
                service.control_group().await?,
            )
        };
        let observation = UnitObservation {
            unit: name.to_owned(),
            active_state: unit.active_state().await?,
            sub_state: unit.sub_state().await?,
            main_pid,
            exit_code,
            exit_status,
            control_group,
            invocation: invocation.clone(),
        };
        if unit.invocation_id().await? != invocation {
            return Err(HostError::Ownership(
                "unit changed during observation".into(),
            ));
        }
        Ok(observation)
    }
    /// Authenticate the current process membership against the same native manager.
    pub async fn current_unit(&self) -> Result<UnitObservation, HostError> {
        self.validate_view()?;
        let path = self
            .manager()
            .await?
            .get_unit_by_pid(std::process::id())
            .await?;
        let unit = UnitProxy::builder(&self.connection)
            .destination(self.owner.as_str())
            .path(path)
            .cache_properties(CacheProperties::No)
            .build()
            .await?;
        let observation = self.observe(&unit.id().await?).await?;
        if observation.control_group != process_group(std::process::id())? {
            return Err(HostError::Ownership(
                "current process and manager membership disagree".into(),
            ));
        }
        Ok(observation)
    }
    /// Resolve the prior scope's actual slice and manager's ordinary task limit.
    pub async fn foreground_placement(
        &self,
        observed: &UnitObservation,
    ) -> Result<(String, String, u64), HostError> {
        if !observed.unit.ends_with(".scope") {
            return Err(HostError::Ownership(
                "foreground entry requires an application scope".into(),
            ));
        }
        let unit = self.unit(&observed.unit).await?;
        let scope = ScopeProxy::builder(&self.connection)
            .destination(self.owner.as_str())
            .path(unit.inner().path().to_owned())
            .cache_properties(CacheProperties::No)
            .build()
            .await?;
        let slice = scope.slice().await?;
        let slice_unit = self.unit(&slice).await?;
        let slice_proxy = SliceProxy::builder(&self.connection)
            .destination(self.owner.as_str())
            .path(slice_unit.inner().path().to_owned())
            .cache_properties(CacheProperties::No)
            .build()
            .await?;
        let group = slice_proxy.control_group().await?;
        if Path::new(&observed.control_group).parent() != Some(Path::new(&group)) {
            return Err(HostError::Ownership(
                "prior application scope does not have the selected slice as its direct parent"
                    .into(),
            ));
        }
        Ok((
            slice,
            group,
            self.manager().await?.default_tasks_max().await?,
        ))
    }
    /// Authenticate the native workload role using manager dependency facts;
    /// an environment marker alone never establishes enclosing containment.
    pub async fn native_workload(&self, observed: &UnitObservation) -> Result<bool, HostError> {
        let Some(nonce) = observed
            .unit
            .strip_prefix("pse-host-")
            .and_then(|value| value.strip_suffix(".scope"))
        else {
            return Ok(false);
        };
        let unit = self.unit(&observed.unit).await?;
        if unit.description().await? != format!("pse-host original {nonce}") {
            return Ok(false);
        }
        for parent in unit.binds_to().await? {
            let Some(parent_nonce) = parent
                .strip_prefix("pse-host-")
                .and_then(|value| value.strip_suffix(".scope"))
            else {
                continue;
            };
            let owner = self.unit(&parent).await?;
            if owner.description().await? == format!("pse-host original {parent_nonce}")
                && unit.after().await?.contains(&parent)
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
    /// Reconcile a generated intent, distinguishing authoritative absence from uncertainty.
    pub async fn reconcile_if_present(
        &self,
        intent: &LaunchIntent,
        deadline: Instant,
    ) -> Result<Option<OwnedUnit>, HostError> {
        tokio::time::timeout_at(deadline.into(), async {
            self.validate_view()?;
            match self.unit(intent.unit()).await {
                Err(HostError::Manager(zbus::Error::MethodError(name, _, _)))
                    if name.as_str() == "org.freedesktop.systemd1.NoSuchUnit" =>
                {
                    Ok(None)
                }
                Err(error) => Err(error),
                Ok(_) => self.reconcile(intent, deadline).await.map(Some),
            }
        })
        .await
        .map_err(|_| HostError::Deadline("original scope reconciliation".into()))?
    }
    /// Create one private scope from a kernel process descriptor, never a recycled PID.
    pub async fn start_scope(
        &self,
        intent: &LaunchIntent,
        pidfd: std::os::fd::OwnedFd,
        binds_to: &str,
        slice: &str,
        deadline: Instant,
        stop_timeout: Duration,
    ) -> Result<OwnedUnit, HostError> {
        if Instant::now() >= deadline {
            return Err(HostError::Deadline(
                "scope launch expired before dispatch".into(),
            ));
        }
        tokio::time::timeout_at(deadline.into(), async {
            self.validate_view()?;
            let manager = self.manager().await?;
            let mut removed = manager.receive_job_removed().await?;
            let fd = zbus::wire::Fd::from(pidfd);
            let properties = vec![
                property("AddRef", true)?,
                property("Description", intent.description.clone())?,
                property("Slice", slice.to_owned())?,
                property("PIDFDs", vec![fd])?,
                property("KillMode", "control-group".to_owned())?,
                property("CollectMode", "inactive-or-failed".to_owned())?,
                property(
                    "TimeoutStopUSec",
                    u64::try_from(stop_timeout.as_micros()).map_err(|_| {
                        HostError::Ownership("scope timeout exceeds manager range".into())
                    })?,
                )?,
                property("BindsTo", vec![binds_to.to_owned()])?,
                property("After", vec![binds_to.to_owned()])?,
            ];
            let result = launch_job(
                intent,
                manager.start_transient_unit(intent.unit.clone(), "fail".into(), properties, vec![]),
                removed.by_ref().map(|signal| {
                    let args = signal.args()?;
                    Ok(JobEvent {
                        job: args.job().clone(),
                        unit: args.unit().to_owned(),
                        result: args.result().to_owned(),
                    })
                }),
            )
            .await?;
            if result != "done" {
                return Err(unknown_launch(intent, format!("scope activation: {result}")));
            }
            self.reconcile_inner(intent).await.map_err(|error| unknown_launch(intent, error))
        })
        .await
        .map_err(|_| HostError::UnknownLaunch {
            unit: intent.unit.clone(),
            detail: "absolute scope startup deadline".into(),
        })?
    }
    /// Start a new native service, binding its identity before returning a performing capability.
    pub async fn start_service(
        &self,
        spec: &ServiceSpec,
        deadline: Instant,
    ) -> Result<OwnedUnit, HostError> {
        if Instant::now() >= deadline {
            return Err(HostError::Deadline(
                "launch deadline already expired; not submitted".into(),
            ));
        }
        tokio::time::timeout_at(deadline.into(), self.start_service_inner(spec))
            .await
            .map_err(|_| HostError::UnknownLaunch {
                unit: spec.intent.unit.clone(),
                detail: "absolute launch deadline".into(),
            })?
    }
    async fn start_service_inner(
        &self,
        spec: &ServiceSpec,
    ) -> Result<OwnedUnit, HostError> {
        self.validate_view()?;
        if !spec.executable.is_absolute() {
            return Err(HostError::Ownership(
                "service executable must be absolute".into(),
            ));
        }
        let executable = spec
            .executable
            .to_str()
            .ok_or_else(|| HostError::Ownership("D-Bus executable path must be UTF-8".into()))?;
        let mut argv = vec![executable.to_owned()];
        argv.extend(spec.arguments.iter().cloned());
        let manager = self.manager().await?;
        let mut removed = manager.receive_job_removed().await?;
        let properties = vec![
            property("AddRef", true)?,
            property("Description", spec.intent.description.clone())?,
            property("Type", "exec".to_owned())?,
            property("KillMode", "control-group".to_owned())?,
            property("Restart", "no".to_owned())?,
            property("RemainAfterExit", true)?,
            property("CollectMode", "inactive-or-failed".to_owned())?,
            property("ExecStart", vec![(executable.to_owned(), argv, false)])?,
            property("Environment", spec.environment.clone())?,
        ];
        let job_result = launch_job(
            &spec.intent,
            manager.start_transient_unit(spec.intent.unit.clone(), "fail".into(), properties, vec![]),
            removed.by_ref().map(|signal| {
                let args = signal.args()?;
                Ok(JobEvent {
                    job: args.job().clone(),
                    unit: args.unit().to_owned(),
                    result: args.result().to_owned(),
                })
            }),
        )
        .await?;
        let mut owned =
            self.reconcile_inner(&spec.intent)
                .await
                .map_err(|error| HostError::UnknownLaunch {
                    unit: spec.intent.unit.clone(),
                    detail: format!("job result {job_result}; {error}"),
                })?;
        owned.launch_job_result = Some(job_result);
        Ok(owned)
    }
    async fn require_reference(&self, unit: &UnitProxy<'_>) -> Result<(), HostError> {
        let client = self
            .connection
            .unique_name()
            .ok_or_else(|| HostError::Ownership("native bus client identity absent".into()))?;
        require_original_reference(Some(client.as_str()), &unit.refs().await?)
    }
    /// Recover the exact original intent through its existing launch-client reference.
    /// Reconciliation never repeats launch or increments systemd's recursive references.
    pub async fn reconcile(
        &self,
        intent: &LaunchIntent,
        deadline: Instant,
    ) -> Result<OwnedUnit, HostError> {
        tokio::time::timeout_at(deadline.into(), self.reconcile_inner(intent))
            .await
            .map_err(|_| HostError::UnknownLaunch {
                unit: intent.unit.clone(),
                detail: "absolute reconciliation deadline".into(),
            })?
    }
    async fn reconcile_inner(&self, intent: &LaunchIntent) -> Result<OwnedUnit, HostError> {
        self.validate_view()?;
        let unit = self.unit(&intent.unit).await?;
        if unit.description().await? != intent.description {
            return Err(HostError::Ownership(
                "original launch description differs".into(),
            ));
        }
        let observation = self.observe(&intent.unit).await?;
        if observation.invocation.len() != 16
            || observation.invocation.iter().all(|byte| *byte == 0)
        {
            return Err(HostError::Ownership(
                "unit invocation identity absent".into(),
            ));
        }
        // StartTransientUnit atomically establishes this exact bus client's
        // reference (AddRef). Reconciliation is observational: RefUnit counts
        // recursively, so calling it here would leak on retries and failures.
        self.require_reference(&unit).await?;
        let process = if observation.main_pid == 0 {
            None
        } else {
            match ProcessReader::default().process(observation.main_pid) {
                ProcessRead::Present(facts) if facts.identity.boot_id == self.boot => {
                    Some(facts.identity)
                }
                _ => {
                    return Err(HostError::Ownership(
                        "original main process identity unavailable".into(),
                    ));
                }
            }
        };
        let group = Group::open(
            &observation.control_group,
            observation.completed_status().is_some()
                || matches!(observation.active_state.as_str(), "inactive" | "failed"),
        )?;
        self.validate_view()?;
        require_original_identity(intent, &unit.description().await?, &observation.invocation, &unit.invocation_id().await?)?;
        self.require_reference(&unit).await?;
        Ok(OwnedUnit {
            manager: self.clone(),
            intent: intent.clone(),
            invocation: observation.invocation,
            process,
            group,
            launch_job_result: None,
            released: false,
            reference_released: false,
        })
    }
}

fn unknown_launch(intent: &LaunchIntent, detail: impl std::fmt::Display) -> HostError {
    HostError::UnknownLaunch { unit: intent.unit.clone(), detail: detail.to_string() }
}

struct JobEvent {
    job: OwnedObjectPath,
    unit: String,
    result: String,
}

// A small protocol seam: the real signal subscription is installed before dispatch.
// Tests feed recorded job inputs; no alternate manager or launch mechanism is introduced.
async fn launch_job(
    intent: &LaunchIntent,
    dispatch: impl Future<Output = Result<OwnedObjectPath, zbus::Error>>,
    events: impl futures_util::Stream<Item = Result<JobEvent, zbus::Error>>,
) -> Result<String, HostError> {
    let job = dispatch.await.map_err(|error| unknown_launch(intent, error))?;
    futures_util::pin_mut!(events);
    while let Some(event) = events.next().await {
        let event = event.map_err(|error| unknown_launch(intent, error))?;
        if event.job != job {
            continue;
        }
        if event.unit != intent.unit {
            return Err(unknown_launch(intent, "correlated job names a different unit"));
        }
        // Job outcome does not replace the original process outcome. Reconciliation
        // must still recover and settle an already-execed nonzero child.
        return Ok(event.result);
    }
    Err(unknown_launch(intent, "job signal stream closed"))
}

fn require_original_reference(client: Option<&str>, refs: &[String]) -> Result<(), HostError> {
    if !client.is_some_and(|client| refs.iter().any(|name| name == client)) {
        return Err(HostError::Ownership("original launch client's unit reference absent".into()));
    }
    Ok(())
}

fn require_original_identity(
    intent: &LaunchIntent,
    description: &str,
    expected: &[u8],
    actual: &[u8],
) -> Result<(), HostError> {
    if description != intent.description || expected.len() != 16
        || expected.iter().all(|byte| *byte == 0) || expected != actual
    {
        return Err(HostError::Ownership("original unit description/invocation changed or absent".into()));
    }
    Ok(())
}

fn singleton_pid(status: &str, expected: u32) -> Result<(), HostError> {
    let rows: Vec<_> = status
        .lines()
        .filter_map(|line| line.strip_prefix("NSpid:"))
        .collect();
    let [row] = rows.as_slice() else {
        return Err(HostError::Ownership(
            "manager PID namespace fact absent/duplicated".into(),
        ));
    };
    let fields: Vec<_> = row.split_whitespace().collect();
    if fields.len() != 1 || fields[0].parse::<u32>().ok() != Some(expected) {
        return Err(HostError::Ownership(
            "manager PID view differs or is nested".into(),
        ));
    }
    Ok(())
}
fn manager_process(pid: u32, namespace: &str, boot: &str) -> Result<ProcessIdentity, HostError> {
    let reader = ProcessReader::default();
    let before = match reader.stat(pid) {
        ProcessRead::Present(stat) => stat.start_ticks,
        _ => {
            return Err(HostError::Ownership(
                "manager process stat unavailable".into(),
            ));
        }
    };
    singleton_pid(
        &std::fs::read_to_string(format!("/proc/{pid}/status"))?,
        pid,
    )?;
    let after = match reader.stat(pid) {
        ProcessRead::Present(stat) => stat.start_ticks,
        _ => {
            return Err(HostError::Ownership(
                "manager process stat changed/unavailable".into(),
            ));
        }
    };
    if before != after {
        return Err(HostError::Ownership("manager PID identity changed".into()));
    }
    Ok(ProcessIdentity {
        boot_id: boot.to_owned(),
        pid_namespace: namespace.to_owned(),
        pid,
        start_ticks: before,
    })
}
fn cgroup_mount(contents: &str) -> Result<String, HostError> {
    let mut devices = Vec::new();
    for line in contents.lines() {
        let Some((before, after)) = line.split_once(" - ") else {
            continue;
        };
        let fields: Vec<_> = before.split_whitespace().collect();
        if fields.get(4) == Some(&"/sys/fs/cgroup") {
            if fields.get(3) != Some(&"/") || after.split_whitespace().next() != Some("cgroup2") {
                return Err(HostError::Ownership(
                    "cgroup mount is not the unified hierarchy root".into(),
                ));
            }
            if let Some(device) = fields.get(2) {
                devices.push((*device).to_owned());
            }
        }
    }
    match devices.as_slice() {
        [device] => Ok(device.clone()),
        _ => Err(HostError::Ownership(
            "unambiguous cgroup-v2 mount unavailable".into(),
        )),
    }
}
fn process_group(pid: u32) -> Result<String, HostError> {
    let text = std::fs::read_to_string(format!("/proc/{pid}/cgroup"))?;
    let groups: Vec<_> = text
        .lines()
        .filter_map(|line| line.strip_prefix("0::"))
        .collect();
    match groups.as_slice() {
        [group] if group.starts_with('/') => Ok((*group).to_owned()),
        _ => Err(HostError::Ownership(
            "manager unified group unavailable".into(),
        )),
    }
}

fn property<T>(name: &str, value: T) -> Result<(String, OwnedVariant), HostError>
where
    T: Into<Variant<'static>>,
{
    Ok((name.to_owned(), OwnedVariant::try_from(value.into())?))
}

/// A borrowed unit has read authority only, with no method to stop it.
#[derive(Clone, Debug)]
pub struct BorrowedUnit {
    manager: HostManager,
    name: String,
}
impl BorrowedUnit {
    /// Read live facts; this does not imply physical release or grant mutation permission.
    pub async fn observe(&self) -> Result<UnitObservation, HostError> {
        self.manager.observe(&self.name).await
    }
}

/// Performing ownership of an original private service or scope; failure retains this capability.
#[must_use = "owned units require explicit stop/drain; dropping a handle does not establish release"]
#[derive(Debug)]
pub struct OwnedUnit {
    manager: HostManager,
    intent: LaunchIntent,
    invocation: Vec<u8>,
    process: Option<ProcessIdentity>,
    group: Group,
    launch_job_result: Option<String>,
    released: bool,
    reference_released: bool,
}
impl OwnedUnit {
    /// Correlated management-job result, distinct from the actual child exit.
    /// Reconciliation without the original signal leaves this unknown.
    pub fn launch_job_result(&self) -> Option<&str> {
        self.launch_job_result.as_deref()
    }
    /// Original launch intent for reconciliation and diagnostics.
    pub fn intent(&self) -> &LaunchIntent {
        &self.intent
    }
    /// Observe only while the original unit invocation remains selected.
    pub async fn observe(&self) -> Result<UnitObservation, HostError> {
        if self.reference_released {
            return Err(HostError::Ownership(
                "owned unit capability already relinquished".into(),
            ));
        }
        let result = self.manager.observe(&self.intent.unit).await?;
        let unit = self.manager.unit(&self.intent.unit).await?;
        self.manager.require_reference(&unit).await?;
        require_original_identity(&self.intent, &unit.description().await?, &self.invocation, &result.invocation)?;
        if unit.invocation_id().await? != result.invocation {
            return Err(HostError::Ownership("unit changed after ownership observation".into()));
        }
        if result.main_pid != 0 {
            let current = ProcessReader::default().process(result.main_pid);
            if !matches!(current, ProcessRead::Present(facts) if self.process.as_ref() == Some(&facts.identity))
            {
                return Err(HostError::Ownership(
                    "main process identity changed or unavailable".into(),
                ));
            }
        }
        self.group.check_path(&result.control_group)?;
        Ok(result)
    }
    /// Request stop and prove the original cgroup empty before reporting release.
    pub async fn stop_and_drain(
        &mut self,
        deadline: Instant,
    ) -> Result<UnitObservation, HostError> {
        if Instant::now() >= deadline {
            return Err(HostError::Deadline(
                "stop deadline already expired; ownership retained".into(),
            ));
        }
        tokio::time::timeout_at(deadline.into(), self.stop_and_drain_inner(deadline))
            .await
            .map_err(|_| {
                HostError::Deadline(
                    "absolute stop/drain deadline; original ownership retained".into(),
                )
            })?
    }
    async fn stop_and_drain_inner(
        &mut self,
        deadline: Instant,
    ) -> Result<UnitObservation, HostError> {
        self.observe().await?;
        let manager = self.manager.manager().await?;
        let unit = self.manager.unit(&self.intent.unit).await?;
        let properties = zbus::fdo::PropertiesProxy::builder(&self.manager.connection)
            .destination(self.manager.owner.as_str())
            .path(unit.inner().path().to_owned())
            .cache_properties(CacheProperties::No)
            .build()
            .await?;
        // Install both wake sources before StopUnit. Signals only trigger uncached
        // rereads: neither a job completion nor a population notification proves release.
        let mut changed = properties.receive_properties_changed().await?;
        let events = self.group.watch()?;
        self.observe().await?;
        tokio::time::timeout_at(
            deadline.into(),
            manager.stop_unit(self.intent.unit.clone(), "replace".into()),
        )
        .await
        .map_err(|_| HostError::Deadline("stop reply; original ownership retained".into()))??;
        loop {
            let observation = self.observe().await?;
            if matches!(observation.active_state.as_str(), "inactive" | "failed")
                && self.group.empty()?
            {
                manager.unref_unit(self.intent.unit.clone()).await?;
                self.released = true;
                self.reference_released = true;
                return Ok(observation);
            }
            tokio::select! {
                signal = changed.next() => {
                    signal.ok_or_else(|| HostError::Ownership("original unit notification stream closed; ownership retained".into()))?.args()?;
                }
                result = GroupEvents::changed(events.as_ref()) => result?,
                () = self.manager.connection.closed() => {
                    return Err(HostError::Ownership("original manager connection closed; ownership retained".into()));
                }
            }
        }
    }

    /// Establish the exact initial scope population before releasing the exec gate.
    pub async fn assert_population(&self, expected: &[u32]) -> Result<(), HostError> {
        self.observe().await?;
        let directory = self
            .group
            .directory
            .as_ref()
            .ok_or_else(|| HostError::Ownership("initial scope directory absent".into()))?;
        let fd = rustix::fs::openat(
            directory,
            "cgroup.procs",
            rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC | rustix::fs::OFlags::NOFOLLOW,
            rustix::fs::Mode::empty(),
        )
        .map_err(io::Error::from)?;
        let mut contents = String::new();
        File::from(fd).read_to_string(&mut contents)?;
        let mut actual = contents
            .lines()
            .map(|line| {
                line.parse::<u32>().map_err(|error| {
                    HostError::Ownership(format!("invalid cgroup population: {error}"))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        actual.sort_unstable();
        let mut expected = expected.to_vec();
        expected.sort_unstable();
        if actual != expected {
            return Err(HostError::Ownership(
                "initial scope population differs".into(),
            ));
        }
        Ok(())
    }
    /// Release the manager reference without stopping the controller's own scope.
    /// Scope lifetime remains its population; dependent workloads bind to that lifetime.
    pub async fn release_reference(&mut self, deadline: Instant) -> Result<(), HostError> {
        tokio::time::timeout_at(deadline.into(), async {
            self.observe().await?;
            self.manager
                .manager()
                .await?
                .unref_unit(self.intent.unit.clone())
                .await?;
            self.reference_released = true;
            Ok(())
        })
        .await
        .map_err(|_| HostError::Deadline("owner reference settlement".into()))?
    }
    /// Whether explicit physical settlement succeeded, never inferred from Drop.
    pub fn released(&self) -> bool {
        self.released
    }
}

#[derive(Debug)]
struct Group {
    directory: Option<File>,
    path: PathBuf,
    device: u64,
    inode: u64,
}
impl Group {
    fn path(value: &str) -> Result<PathBuf, HostError> {
        let relative = value
            .strip_prefix('/')
            .ok_or_else(|| HostError::Ownership("manager cgroup path is not absolute".into()))?;
        if relative.is_empty()
            || relative.split('/').any(|part| part == "." || part == "..")
            || Path::new(relative)
                .components()
                .any(|part| !matches!(part, Component::Normal(_)))
        {
            return Err(HostError::Ownership("invalid manager cgroup path".into()));
        }
        Ok(Path::new("/sys/fs/cgroup").join(relative))
    }
    fn open(value: &str, completed: bool) -> Result<Self, HostError> {
        // Immediate completion can retire the cgroup before the launch reply. Only
        // the authenticated original completed invocation in a verified view may
        // construct this state; a missing path by itself grants no authority.
        if value.is_empty() && completed {
            return Ok(Self {
                directory: None,
                path: PathBuf::new(),
                device: 0,
                inode: 0,
            });
        }
        let path = Self::path(value)?;
        let descriptor = match rustix::fs::open(
            &path,
            rustix::fs::OFlags::RDONLY
                | rustix::fs::OFlags::DIRECTORY
                | rustix::fs::OFlags::NOFOLLOW
                | rustix::fs::OFlags::CLOEXEC,
            rustix::fs::Mode::empty(),
        ) {
            Ok(fd) => fd,
            Err(rustix::io::Errno::NOENT) if completed => {
                return Ok(Self {
                    directory: None,
                    path,
                    device: 0,
                    inode: 0,
                });
            }
            Err(error) => return Err(io::Error::from(error).into()),
        };
        let directory = File::from(descriptor);
        // Linux uapi/linux/magic.h: CGROUP2_SUPER_MAGIC. The pinned fd must be
        // actual cgroup-v2 before mandatory-file disappearance proves removal.
        let filesystem = rustix::fs::fstatfs(&directory).map_err(io::Error::from)?;
        if filesystem.f_type != 0x6367_7270 {
            return Err(HostError::Ownership(
                "selected group is not cgroup-v2".into(),
            ));
        }
        let meta = directory.metadata()?;
        Ok(Self {
            directory: Some(directory),
            path,
            device: meta.dev(),
            inode: meta.ino(),
        })
    }
    fn check_path(&self, value: &str) -> Result<(), HostError> {
        if value.is_empty() && self.empty()? {
            return Ok(());
        }
        if Self::path(value)? != self.path {
            return Err(HostError::Ownership("unit cgroup changed".into()));
        }
        if self.directory.is_none() {
            return if self.path.try_exists()? {
                Err(HostError::Ownership(
                    "previously absent cgroup appeared".into(),
                ))
            } else {
                Ok(())
            };
        }
        match std::fs::metadata(&self.path) {
            Ok(meta) if meta.dev() == self.device && meta.ino() == self.inode => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound && self.empty()? => Ok(()),
            _ => Err(HostError::Ownership(
                "cgroup path identity changed or unavailable".into(),
            )),
        }
    }
    #[allow(unsafe_code, reason = "Tokio registration owns the original cgroup.events File for its whole lifetime")]
    fn watch(&self) -> Result<Option<GroupEvents>, HostError> {
        let Some(directory) = self.directory.as_ref() else { return Ok(None); };
        let fd = match rustix::fs::openat(
            directory,
            "cgroup.events",
            rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC | rustix::fs::OFlags::NONBLOCK | rustix::fs::OFlags::NOFOLLOW,
            rustix::fs::Mode::empty(),
        ) {
            Ok(fd) => fd,
            Err(rustix::io::Errno::NOENT) if self.empty()? => return Ok(None),
            Err(error) => return Err(io::Error::from(error).into()),
        };
        // Establish the kernfs notification generation before epoll registration.
        // Later changes are acknowledged by seeking and reading this same file.
        let mut file = File::from(fd);
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        parse_population(&contents)?;
        // SAFETY: AsyncFd takes exclusive ownership of this File. No raw fd is
        // exposed, replaced or closed until AsyncFd unregisters and drops it.
        let descriptor = unsafe {
            tokio::io::unix::AsyncFd::register_with_interest(file, tokio::io::Interest::PRIORITY)
        }.map_err(io::Error::from)?;
        Ok(Some(GroupEvents { descriptor }))
    }
    fn empty(&self) -> Result<bool, HostError> {
        let Some(directory) = self.directory.as_ref() else {
            return Ok(self.path.as_os_str().is_empty() || !self.path.try_exists()?);
        };
        let fd = match rustix::fs::openat(
            directory,
            "cgroup.events",
            rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC,
            rustix::fs::Mode::empty(),
        ) {
            Ok(fd) => fd,
            // cgroup.events is a mandatory kernel-owned cgroup-v2 file. It cannot be
            // unlinked independently; ENOENT through the original open directory
            // proves that original cgroup was removed, which requires no members.
            Err(rustix::io::Errno::NOENT) if !self.path.try_exists()? => return Ok(true),
            Err(error) => return Err(io::Error::from(error).into()),
        };
        let mut contents = String::new();
        File::from(fd).read_to_string(&mut contents)?;
        parse_population(&contents)
    }
}
#[derive(Debug)]
struct GroupEvents {
    descriptor: tokio::io::unix::AsyncFd<File>,
}
impl GroupEvents {
    fn read(&self) -> Result<(), HostError> {
        let mut file = self.descriptor.get_ref();
        let mut contents = String::new();
        let read = file.seek(SeekFrom::Start(0))
            .and_then(|_| file.read_to_string(&mut contents));
        match read {
            Ok(_) => { parse_population(&contents)?; Ok(()) }
            // Kernfs invalidates the original opened file on group removal. It is
            // still only a wakeup; Group::empty validates original removal next.
            Err(error) if error.raw_os_error() == Some(rustix::io::Errno::NODEV.raw_os_error()) => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
    async fn changed(events: Option<&Self>) -> Result<(), HostError> {
        let Some(events) = events else { return std::future::pending().await; };
        let mut ready = events.descriptor.ready(tokio::io::Interest::PRIORITY | tokio::io::Interest::ERROR).await?;
        ready.clear_ready();
        // Read this same opened file to acknowledge its kernel notification generation.
        events.read()
    }
}

fn parse_population(contents: &str) -> Result<bool, HostError> {
    let entries: Vec<_> = contents
        .lines()
        .filter_map(|line| line.strip_prefix("populated "))
        .collect();
    match entries.as_slice() {
        ["0"] => Ok(true),
        ["1"] => Ok(false),
        _ => Err(HostError::Ownership(
            "cgroup populated fact unavailable or malformed".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn job_event(job: &str, unit: &str, result: &str) -> Result<JobEvent, zbus::Error> {
        Ok(JobEvent { job: job.try_into().unwrap(), unit: unit.to_owned(), result: result.to_owned() })
    }

    #[tokio::test]
    async fn job_completion_before_ack_ignores_unrelated_jobs_and_retains_nonzero_outcome() {
        let intent = LaunchIntent::fresh().unwrap();
        let (send, receive) = tokio::sync::mpsc::unbounded_channel();
        let dispatch = async {
            // The manager completes and emits signals before the method reply arrives.
            send.send(job_event("/job/2", "unrelated.service", "done")).unwrap();
            send.send(job_event("/job/1", intent.unit(), "failed")).unwrap();
            tokio::task::yield_now().await;
            Ok("/job/1".try_into().unwrap())
        };
        let events = futures_util::stream::unfold(receive, |mut receive| async {
            receive.recv().await.map(|event| (event, receive))
        });
        assert_eq!(launch_job(&intent, dispatch, events).await.unwrap(), "failed");
    }

    #[tokio::test]
    async fn lost_launch_ack_retains_original_intent_without_replaying() {
        let intent = LaunchIntent::fresh().unwrap();
        let dispatched = AtomicU64::new(0);
        let dispatch = async {
            dispatched.fetch_add(1, Ordering::Relaxed);
            Err(zbus::Error::Failure("acknowledgement lost after dispatch".into()))
        };
        let result = launch_job(&intent, dispatch, futures_util::stream::empty()).await;
        assert!(matches!(result, Err(HostError::UnknownLaunch { unit, .. }) if unit == intent.unit));
        assert_eq!(dispatched.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn disconnect_or_mismatched_job_never_confers_launch_authority() {
        let intent = LaunchIntent::fresh().unwrap();
        for events in [
            vec![],
            vec![Err(zbus::Error::Failure("connection closed".into()))],
            vec![job_event("/job/1", "reused.service", "done")],
        ] {
            let result = launch_job(
                &intent,
                async { Ok("/job/1".try_into().unwrap()) },
                futures_util::stream::iter(events),
            ).await;
            assert!(matches!(result, Err(HostError::UnknownLaunch { unit, .. }) if unit == intent.unit));
        }
    }

    #[test]
    fn original_reference_requires_exact_live_launch_client() {
        let refs = vec![":1.21".to_owned(), ":1.22".to_owned()];
        for _ in 0..3 {
            assert!(require_original_reference(Some(":1.21"), &refs).is_ok());
        }
        assert_eq!(refs.len(), 2);
        for client in [None, Some(":1.2"), Some(":1.23")] {
            assert!(require_original_reference(client, &refs).is_err());
        }
        assert!(require_original_reference(Some(":1.21"), &[]).is_err());
    }

    #[test]
    fn original_identity_rejects_unit_reuse_and_missing_invocation() {
        let intent = LaunchIntent::fresh().unwrap();
        let original = [1_u8; 16];
        assert!(require_original_identity(&intent, &intent.description, &original, &original).is_ok());
        assert!(require_original_identity(&intent, "another launch", &original, &original).is_err());
        for replacement in [vec![], vec![0; 16], vec![2; 16], vec![1; 15]] {
            assert!(require_original_identity(&intent, &intent.description, &original, &replacement).is_err());
        }
        assert!(require_original_identity(&intent, &intent.description, &[0; 16], &[0; 16]).is_err());
    }

    #[test]
    fn absent_cgroup_requires_original_terminal_observation() {
        assert!(Group::open("", false).is_err());
        let absent = Group::open("", true).unwrap();
        assert!(absent.empty().unwrap());
        assert!(absent.check_path("").is_ok());
        assert!(absent.check_path("/unrelated.scope").is_err());
        // A regular directory never supplies kernel population authority.
        let fixture = tempfile::tempdir().unwrap();
        let file = File::open(fixture.path()).unwrap();
        assert_ne!(rustix::fs::fstatfs(&file).unwrap().f_type, 0x6367_7270);
    }

    #[tokio::test]
    async fn real_user_manager_view_changes_refuse_before_launch() {
        let manager = HostManager::connect_user(None).await.unwrap();
        let spec = ServiceSpec {
            intent: LaunchIntent::fresh().unwrap(),
            executable: "/bin/sleep".into(), arguments: vec!["30".into()], environment: vec![],
        };
        let mut denied = manager.clone();
        denied.cgroup_root.1 = denied.cgroup_root.1.wrapping_add(1);
        assert!(matches!(denied.start_service(&spec, Instant::now() + Duration::from_secs(5)).await, Err(HostError::Ownership(_))));
        let mut reused = manager.clone();
        reused.manager_process.start_ticks = reused.manager_process.start_ticks.wrapping_add(1);
        assert!(matches!(reused.start_service(&spec, Instant::now() + Duration::from_secs(5)).await, Err(HostError::Ownership(_))));
        assert!(manager.reconcile_if_present(&spec.intent, Instant::now() + Duration::from_secs(5)).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn real_user_manager_disconnect_refuses_launch_without_effect() {
        let manager = HostManager::connect_user(None).await.unwrap();
        manager.connection.clone().close().await.unwrap();
        let spec = ServiceSpec {
            intent: LaunchIntent::fresh().unwrap(),
            executable: "/bin/sleep".into(), arguments: vec!["30".into()], environment: vec![],
        };
        assert!(matches!(manager.start_service(&spec, Instant::now() + Duration::from_secs(5)).await, Err(HostError::Ownership(_))));
    }

    #[tokio::test]
    async fn real_original_cgroup_notifies_population_change() {
        let manager = HostManager::connect_user(None).await.unwrap();
        let spec = ServiceSpec {
            intent: LaunchIntent::fresh().unwrap(),
            executable: "/bin/sleep".into(), arguments: vec!["0.1".into()], environment: vec![],
        };
        let mut owned = manager.start_service(&spec, Instant::now() + Duration::from_secs(10)).await.unwrap();
        let checked: Result<(), HostError> = async {
            if owned.group.empty()? {
                return Err(HostError::Ownership("sleep fixture was already empty".into()));
            }
            let events = owned.group.watch()?;
            if events.is_none() {
                return Err(HostError::Ownership("original cgroup watcher absent".into()));
            }
            // The fixture exits after the kernel's cgroup notification coalescing
            // interval. Immediate StopUnit may remove the group before that timer
            // fires; the production path also listens for manager state changes.
            let notified = tokio::time::timeout(Duration::from_secs(5), GroupEvents::changed(events.as_ref())).await;
            notified.map_err(|_| HostError::Deadline("original cgroup kernel notification".into()))??;
            if !owned.group.empty()? || owned.released() {
                return Err(HostError::Ownership("kernel notification lacks original empty population or falsely released ownership".into()));
            }
            // Removal can race the acknowledgement at either seek or read.
            if let Some(events) = events.as_ref() { events.read()?; }
            Ok(())
        }.await;
        let cleanup = owned.stop_and_drain(Instant::now() + Duration::from_secs(10)).await;
        assert!(checked.is_ok(), "{checked:?}");
        assert!(cleanup.is_ok(), "{cleanup:?}");
    }

    #[tokio::test]
    async fn real_user_manager_owns_and_drains_escaped_descendants() {
        let manager = HostManager::connect_user(None).await.unwrap();
        let spec = ServiceSpec {
            intent: LaunchIntent::fresh().unwrap(),
            executable: "/bin/sh".into(),
            arguments: vec!["-c".into(), "setsid /bin/sleep 30 & wait".into()],
            environment: vec!["PATH=/usr/bin:/bin".into()],
        };
        let mut owned = manager
            .start_service(&spec, Instant::now() + Duration::from_secs(10))
            .await
            .unwrap();
        let timed_out = owned.stop_and_drain(Instant::now()).await;
        let retained = !owned.released();
        let observed = owned.observe().await;
        let borrowed = manager.borrow(spec.intent.unit.clone()).observe().await;
        let cleanup = owned
            .stop_and_drain(Instant::now() + Duration::from_secs(10))
            .await;
        assert!(matches!(timed_out, Err(HostError::Deadline(_))));
        assert!(retained);
        assert!(observed.is_ok(), "{observed:?}");
        assert!(borrowed.is_ok(), "{borrowed:?}");
        assert!(cleanup.is_ok(), "{cleanup:?}");
        assert!(owned.released());
    }
    #[tokio::test]
    async fn real_user_manager_keeps_original_child_exit() {
        let manager = HostManager::connect_user(None).await.unwrap();
        let spec = ServiceSpec {
            intent: LaunchIntent::fresh().unwrap(),
            executable: "/bin/sh".into(),
            arguments: vec!["-c".into(), "exit 7".into()],
            environment: vec!["PATH=/usr/bin:/bin".into()],
        };
        let mut owned = manager
            .start_service(&spec, Instant::now() + Duration::from_secs(10))
            .await
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        let observed: Result<i32, HostError> = async {
            loop {
                if let Some(code) = owned.observe().await?.completed_status() {
                    break Ok(code);
                }
                if Instant::now() >= deadline {
                    break Err(HostError::Deadline("test original child exit".into()));
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
        .await;
        let cleanup = owned.stop_and_drain(deadline).await;
        assert_eq!(observed.unwrap(), 7);
        assert!(cleanup.is_ok(), "{cleanup:?}");
    }
    #[tokio::test]
    async fn concurrent_fast_failures_retain_each_original_exit_and_settle_all() {
        let manager = HostManager::connect_user(None).await.unwrap();
        let mut cases = tokio::task::JoinSet::new();
        for _ in 0..16 {
            let manager = manager.clone();
            cases.spawn(async move {
                let spec = ServiceSpec {
                    intent: LaunchIntent::fresh().map_err(|error| error.to_string())?,
                    executable: "/bin/sh".into(),
                    arguments: vec!["-c".into(), "exit 7".into()],
                    environment: vec!["PATH=/usr/bin:/bin".into()],
                };
                let mut owned = match manager
                    .start_service(&spec, Instant::now() + Duration::from_secs(10))
                    .await
                {
                    Ok(owned) => owned,
                    Err(operation) => {
                        // Even a failing case attempts original-intent settlement;
                        // other cases remain independently owned and are all joined.
                        let cleanup = async {
                            let mut original = manager
                                .reconcile(&spec.intent, Instant::now() + Duration::from_secs(10))
                                .await?;
                            original
                                .stop_and_drain(Instant::now() + Duration::from_secs(10))
                                .await
                        }
                        .await;
                        return Err(format!(
                            "{}: {operation}; cleanup {cleanup:?}",
                            spec.intent.unit
                        ));
                    }
                };
                let observed: Result<i32, HostError> = async {
                    let deadline = Instant::now() + Duration::from_secs(10);
                    loop {
                        if let Some(code) = owned.observe().await?.completed_status() {
                            break Ok(code);
                        }
                        if Instant::now() >= deadline {
                            break Err(HostError::Deadline(
                                "concurrent original completion".into(),
                            ));
                        }
                        tokio::task::yield_now().await;
                    }
                }
                .await;
                let job_result = owned.launch_job_result().map(str::to_owned);
                let cleanup = owned
                    .stop_and_drain(Instant::now() + Duration::from_secs(10))
                    .await;
                match (observed, cleanup) {
                    (Ok(7), Ok(_)) if job_result.is_some() && owned.released() => Ok(()),
                    (outcome, cleanup) => Err(format!(
                        "{} job={job_result:?} outcome={outcome:?} cleanup={cleanup:?}",
                        spec.intent.unit
                    )),
                }
            });
        }
        let mut failures = Vec::new();
        while let Some(result) = cases.join_next().await {
            match result {
                Ok(Ok(())) => {}
                other => failures.push(format!("{other:?}")),
            }
        }
        assert!(failures.is_empty(), "{failures:#?}");
    }
    #[tokio::test]
    async fn repeated_original_reconciliation_never_adds_manager_references() {
        let manager = HostManager::connect_user(None).await.unwrap();
        let spec = ServiceSpec {
            intent: LaunchIntent::fresh().unwrap(),
            executable: "/bin/sh".into(),
            arguments: vec!["-c".into(), "exit 7".into()],
            environment: vec![],
        };
        let deadline = Instant::now() + Duration::from_secs(15);
        let mut original = manager.start_service(&spec, deadline).await.unwrap();
        let duplicate = manager.reconcile(&spec.intent, deadline).await.unwrap();
        let second = manager.reconcile(&spec.intent, deadline).await.unwrap();
        original.stop_and_drain(deadline).await.unwrap();
        // One unref must remove the original sender completely, regardless of
        // how often that same original intent was reconciled.
        let retained = match manager.unit(spec.intent.unit()).await {
            Ok(unit) => unit.refs().await.unwrap().iter().any(|name| {
                Some(name.as_str()) == manager.connection.unique_name().map(|value| value.as_str())
            }),
            Err(HostError::Manager(zbus::Error::MethodError(name, _, _)))
                if name.as_str() == "org.freedesktop.systemd1.NoSuchUnit" =>
            {
                false
            }
            Err(error) => panic!("original unit reference observation failed: {error}"),
        };
        assert!(
            !retained,
            "reconciliation added an extra numeric sender reference"
        );
        assert!(duplicate.observe().await.is_err());
        assert!(second.observe().await.is_err());
    }
    #[test]
    fn manager_view_rejects_nested_or_ambiguous_pid_mounts() {
        assert!(singleton_pid("NSpid:\t2381\n", 2381).is_ok());
        for invalid in ["", "NSpid: 2381 1", "NSpid: 7", "NSpid: 2381\nNSpid: 2381"] {
            assert!(singleton_pid(invalid, 2381).is_err());
        }
        assert_eq!(
            cgroup_mount("37 28 0:31 / /sys/fs/cgroup rw - cgroup2 cgroup2 rw").unwrap(),
            "0:31"
        );
        for invalid in [
            "",
            "37 28 0:31 /user.slice /sys/fs/cgroup rw - cgroup2 cgroup2 rw",
            "37 28 0:31 / /sys/fs/cgroup rw - tmpfs tmpfs rw",
            "37 28 0:31 / /other rw - cgroup2 cgroup2 rw",
        ] {
            assert!(cgroup_mount(invalid).is_err());
        }
    }
    #[test]
    fn population_never_infers_empty_from_unknown() {
        assert!(parse_population("populated 0\nfrozen 0\n").unwrap());
        assert!(!parse_population("populated 1\n").unwrap());
        for invalid in ["", "populated 2", "populated 0\npopulated 1", "frozen 0"] {
            assert!(parse_population(invalid).is_err());
        }
    }
    #[test]
    fn cgroup_paths_cannot_escape_kernel_root() {
        for invalid in ["", "/", "relative", "/x/../y", "/x/./y"] {
            assert!(Group::path(invalid).is_err(), "{invalid}");
        }
        assert_eq!(
            Group::path("/user.slice/test.scope").unwrap(),
            Path::new("/sys/fs/cgroup/user.slice/test.scope")
        );
    }
    #[test]
    fn original_launch_ids_are_distinct() {
        let a = LaunchIntent::fresh().unwrap();
        let b = LaunchIntent::fresh().unwrap();
        assert_ne!(a.unit, b.unit);
        assert_ne!(a.description, b.description);
    }
    #[test]
    fn exit_status_never_turns_running_into_success() {
        let mut value = UnitObservation {
            unit: "test".into(),
            invocation: vec![],
            active_state: "active".into(),
            sub_state: "running".into(),
            main_pid: 1,
            exit_code: 1,
            exit_status: 0,
            control_group: "/test".into(),
        };
        assert_eq!(value.completed_status(), None);
        value.active_state = "failed".into();
        value.main_pid = 0;
        value.exit_status = 7;
        assert_eq!(value.completed_status(), Some(7));
        value.exit_code = 2;
        value.exit_status = 15;
        assert_eq!(value.completed_status(), Some(143));
    }
}
