// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Kernel subgroup effects. The managed lifecycle separately closes/serializes launches.
use super::*;
use std::{ffi::OsString, io::Write, os::unix::ffi::OsStrExt};

/// Exact observed directory coordinates, never a pathname-only destructive capability.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DirectoryBinding {
    pub(crate) path: PathBuf,
    pub(crate) device: u64,
    pub(crate) inode: u64,
}
impl DirectoryBinding {
    /// Recorded coordinates; this inert snapshot grants no mutation or adoption authority.
    pub fn path(&self) -> &Path {
        &self.path
    }
    /// Original opened directory's device and inode.
    pub fn identity(&self) -> (u64, u64) {
        (self.device, self.inode)
    }
    fn from_group(group: &Group) -> Self {
        Self {
            path: group.path.clone(),
            device: group.device,
            inode: group.inode,
        }
    }
    fn matches(&self, group: &Group) -> bool {
        self == &Self::from_group(group)
    }
}
/// Inert original root/member snapshot, suitable for protected evidence storage.
/// Production recovery requires a protected ledger owner and is not implemented yet.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PayloadParentBinding {
    manager: String,
    intent: LaunchIntent,
    root: UnitBinding,
    ancestry: Vec<DirectoryBinding>,
}
/// Persist this unique intent before asking the kernel to create its child.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubgroupCreationIntent {
    parent: PayloadParentBinding,
    name: String,
}
/// Borrowed native root/member authority. It cannot stop the root or release its reservation.
#[derive(Debug)]
pub struct PayloadParent {
    manager: HostManager,
    binding: PayloadParentBinding,
    ancestry: Vec<Group>,
}

impl CurrentMembership {
    /// Lend only this actual member of an authenticated live native workload role.
    /// No pathname, environment marker or caller-supplied launch intent grants it.
    pub async fn borrow_payload_parent(
        &self,
        manager: &HostManager,
    ) -> Result<PayloadParent, HostError> {
        self.validate(manager).await?;
        if self.root.active_state != "active" || !manager.native_workload(&self.root).await? {
            return Err(HostError::Ownership(
                "actual current member is not a live native workload role".into(),
            ));
        }
        let unit = manager.unit(&self.root.unit).await?;
        let intent = LaunchIntent {
            unit: self.root.unit.clone(),
            description: unit.description().await?,
        };
        require_original_identity(
            &intent,
            &unit.description().await?,
            &self.root.invocation,
            &unit.invocation_id().await?,
        )?;
        let original = &self.ancestry[0];
        let root = UnitBinding {
            invocation: self.root.invocation.clone(),
            group: self.root.control_group.clone(),
            device: original.device,
            inode: original.inode,
        };
        let ancestry = self
            .ancestry
            .iter()
            .map(copy_group)
            .collect::<Result<Vec<_>, _>>()?;
        let binding = PayloadParentBinding {
            manager: manager.identity_key(),
            intent,
            root,
            ancestry: ancestry.iter().map(DirectoryBinding::from_group).collect(),
        };
        let value = PayloadParent {
            manager: manager.clone(),
            binding,
            ancestry,
        };
        value.require_launchable().await?;
        self.validate(manager).await?;
        if !manager
            .native_workload(&manager.observe(&self.root.unit).await?)
            .await?
        {
            return Err(HostError::Ownership(
                "native workload dependency role changed while lending its member".into(),
            ));
        }
        Ok(value)
    }
}

impl OwnedUnit {
    /// Lend subgroup authority without transferring the original unit/reference/reservation.
    pub async fn payload_parent(&self) -> Result<PayloadParent, HostError> {
        let binding = self.binding().await?;
        let root = self.observe().await?;
        let ancestry = vec![copy_group(&self.group)?];
        let value = PayloadParent {
            manager: self.manager.clone(),
            binding: PayloadParentBinding {
                manager: self.manager.identity_key(),
                intent: self.intent.clone(),
                root: binding,
                ancestry: ancestry.iter().map(DirectoryBinding::from_group).collect(),
            },
            ancestry,
        };
        require_core(&self.manager, &root, &value.ancestry).await?;
        value.validate().await?;
        Ok(value)
    }
}
impl HostManager {
    /// Reopen an exact recorded parent after supervisor handoff; no original-unit stop is lent.
    /// Fixture feasibility only until a protected production ledger owner supplies authority.
    #[cfg(test)]
    pub(crate) async fn recover_payload_parent(
        &self,
        binding: &PayloadParentBinding,
    ) -> Result<PayloadParent, HostError> {
        self.validate_binding(&binding.intent, &binding.root)
            .await?;
        if binding.manager != self.identity_key() {
            return Err(HostError::Ownership(
                "recorded subgroup manager identity changed".into(),
            ));
        }
        let root = Group::open(&binding.root.group, false)?;
        let member = binding
            .ancestry
            .last()
            .ok_or_else(|| HostError::Ownership("recorded actual subgroup parent absent".into()))?;
        let member = member
            .path
            .strip_prefix("/sys/fs/cgroup")
            .map_err(|_| HostError::Ownership("recorded member escaped unified root".into()))?;
        let member = format!(
            "/{}",
            member.to_str().ok_or_else(|| HostError::Ownership(
                "recorded member coordinates are not UTF-8".into()
            ))?
        );
        let ancestry = open_chain(&root, &member)?;
        if ancestry.len() != binding.ancestry.len()
            || !binding
                .ancestry
                .iter()
                .zip(&ancestry)
                .all(|(expected, actual)| expected.matches(actual))
        {
            return Err(HostError::Ownership(
                "recorded subgroup component identity changed".into(),
            ));
        }
        let value = PayloadParent {
            manager: self.clone(),
            binding: binding.clone(),
            ancestry,
        };
        value.validate().await?;
        Ok(value)
    }
}
async fn require_core(
    manager: &HostManager,
    root: &UnitObservation,
    ancestry: &[Group],
) -> Result<(), HostError> {
    if delegation(manager, root).await? != (true, vec![]) {
        return Err(HostError::Ownership(
            "original root is not core-only delegated".into(),
        ));
    }
    for group in ancestry {
        let controllers = group.control("cgroup.subtree_control")?.ok_or_else(|| {
            HostError::Ownership("original subtree controller state unavailable".into())
        })?;
        if !controllers.trim().is_empty() {
            return Err(HostError::Ownership(
                "core-only subgroup has enabled descendant controllers".into(),
            ));
        }
        if group.control("cgroup.type")?.as_deref().map(str::trim) != Some("domain") {
            return Err(HostError::Ownership(
                "payload subgroup is not a domain cgroup".into(),
            ));
        }
    }
    Ok(())
}
impl PayloadParent {
    /// Inert evidence snapshot. Deserializing it never grants a production capability.
    pub fn binding(&self) -> &PayloadParentBinding {
        &self.binding
    }
    /// Select the actual member under this exact original root, never a sibling scope.
    pub async fn at_current_member(
        &self,
        membership: &CurrentMembership,
    ) -> Result<Self, HostError> {
        membership.validate(&self.manager).await?;
        self.validate().await?;
        if membership.root.unit != self.binding.intent.unit
            || membership.root.invocation != self.binding.root.invocation
            || membership.root.control_group != self.binding.root.group
        {
            return Err(HostError::Ownership(
                "actual member is not in the borrowed original root".into(),
            ));
        }
        let ancestry = membership
            .ancestry
            .iter()
            .map(copy_group)
            .collect::<Result<Vec<_>, _>>()?;
        let mut binding = self.binding.clone();
        binding.ancestry = ancestry.iter().map(DirectoryBinding::from_group).collect();
        let value = Self {
            manager: self.manager.clone(),
            binding,
            ancestry,
        };
        value.validate().await?;
        Ok(value)
    }
    async fn validate(&self) -> Result<(), HostError> {
        if self.binding.manager != self.manager.identity_key() {
            return Err(HostError::Ownership(
                "subgroup manager binding changed".into(),
            ));
        }
        self.manager
            .validate_binding(&self.binding.intent, &self.binding.root)
            .await?;
        validate_chain(&self.ancestry)?;
        if self.ancestry.len() != self.binding.ancestry.len()
            || !self
                .binding
                .ancestry
                .iter()
                .zip(&self.ancestry)
                .all(|(expected, actual)| expected.matches(actual))
        {
            return Err(HostError::Ownership(
                "original subgroup ancestry differs".into(),
            ));
        }
        let root = self.manager.observe(self.binding.intent.unit()).await?;
        require_core(&self.manager, &root, &self.ancestry).await
    }
    async fn require_launchable(&self) -> Result<(), HostError> {
        self.validate().await?;
        if self
            .manager
            .observe(self.binding.intent.unit())
            .await?
            .active_state
            != "active"
        {
            return Err(HostError::Ownership(
                "original payload root is no longer active for launch".into(),
            ));
        }
        Ok(())
    }
    /// Select a strict child component; persist its unique intent before creation effects.
    pub fn creation_intent(&self, name: &str) -> Result<SubgroupCreationIntent, HostError> {
        if name.is_empty()
            || name.len() > 128
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            return Err(HostError::Ownership(
                "subgroup name is not one strict component".into(),
            ));
        }
        Ok(SubgroupCreationIntent {
            parent: self.binding.clone(),
            name: name.to_owned(),
        })
    }
    /// Recover only an actually recorded original child inode, including unbound-open custody.
    /// Fixture feasibility only; production sibling adoption needs a protected ledger owner.
    #[cfg(test)]
    pub(crate) async fn recover_created(
        &self,
        intent: &SubgroupCreationIntent,
        binding: &DirectoryBinding,
    ) -> Result<OwnedSubgroup, HostError> {
        if intent.parent != self.binding {
            return Err(HostError::Ownership(
                "recorded child creation parent differs".into(),
            ));
        }
        self.creation_intent(&intent.name)?;
        self.validate().await?;
        let parent = &self.ancestry[self.ancestry.len() - 1];
        let group = open_child(parent, OsStr::new(&intent.name))?;
        if !binding.matches(&group) {
            return Err(HostError::Ownership(
                "recorded original child inode changed".into(),
            ));
        }
        let group = OwnedSubgroup {
            parent: self.copy()?,
            group,
            name: intent.name.clone(),
            removed: false,
        };
        group.validate().await?;
        Ok(group)
    }
    /// Does not release any payload gate. Errors retain known effects and opened custody.
    pub async fn create(&self, intent: &SubgroupCreationIntent) -> CreationOutcome {
        if intent.parent.manager != self.binding.manager
            || intent.parent.ancestry != self.binding.ancestry
            || intent.parent.intent != self.binding.intent
            || intent.parent.root != self.binding.root
        {
            return CreationOutcome::NoEffect(HostError::Ownership(
                "subgroup creation intent differs from original parent".into(),
            ));
        }
        if let Err(cause) = self
            .creation_intent(&intent.name)
            .and_then(|_| validate_chain(&self.ancestry))
        {
            return CreationOutcome::NoEffect(cause);
        }
        if let Err(cause) = self.require_launchable().await {
            return CreationOutcome::NoEffect(cause);
        }
        // Preserve the complete live parent capability before the first creation effect.
        let owned_parent = match self.copy() {
            Ok(parent) => parent,
            Err(cause) => return CreationOutcome::NoEffect(cause),
        };
        let parent = &self.ancestry[self.ancestry.len() - 1];
        let mkdir = directory(parent).and_then(|fd| {
            rustix::fs::mkdirat(fd, intent.name.as_str(), rustix::fs::Mode::RWXU)
                .map_err(io::Error::from)
                .map_err(HostError::from)
        });
        if let Err(cause) = mkdir {
            return CreationOutcome::NoEffect(cause);
        }
        let group = match open_child(parent, OsStr::new(&intent.name)) {
            Ok(group) => group,
            Err(cause) => {
                return CreationOutcome::CreatedButUnbound {
                    intent: intent.clone(),
                    opened: None,
                    cause,
                };
            }
        };
        let subgroup = OwnedSubgroup {
            parent: owned_parent,
            group,
            name: intent.name.clone(),
            removed: false,
        };
        match subgroup.validate().await {
            Ok(()) => CreationOutcome::Created(Box::new(subgroup)),
            Err(cause) => CreationOutcome::CreatedButUnbound {
                intent: intent.clone(),
                opened: Some(Box::new(OpenedSubgroupCustody { group: subgroup })),
                cause,
            },
        }
    }
    fn copy(&self) -> Result<Self, HostError> {
        Ok(Self {
            manager: self.manager.clone(),
            binding: self.binding.clone(),
            ancestry: self
                .ancestry
                .iter()
                .map(copy_group)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}
/// All possible creation effect boundaries; no contradictory boolean/optional state.
#[derive(Debug)]
#[must_use = "creation outcomes retain committed subgroup effects and opened custody"]
pub enum CreationOutcome {
    /// The subgroup was not created by this attempt.
    NoEffect(HostError),
    /// The fresh subgroup is fully bound to its original parent and inode.
    Created(Box<OwnedSubgroup>),
    /// Creation committed but complete authentication did not finish. No payload gate opened.
    CreatedButUnbound {
        /// Persisted pre-effect identity and parent snapshot.
        intent: SubgroupCreationIntent,
        /// Actual fresh inode custody, when opening succeeded, never reconstructed from data.
        opened: Option<Box<OpenedSubgroupCustody>>,
        /// Original authentication/opening error.
        cause: HostError,
    },
}
/// Original opened inode retained even if post-creation authentication could not complete.
#[derive(Debug)]
pub struct OpenedSubgroupCustody {
    group: OwnedSubgroup,
}
impl OpenedSubgroupCustody {
    /// Original fresh inode evidence, independent of failed post-creation validation.
    pub fn binding(&self) -> DirectoryBinding {
        self.group.binding()
    }
    /// Return only the actual fresh capability for revalidated cleanup/retry.
    /// This never adopts a directory from serialized coordinates.
    pub fn into_owned(self) -> OwnedSubgroup {
        self.group
    }
}
/// One original child cgroup. Drop never asserts removal or managed settlement.
#[derive(Debug)]
#[must_use = "original subgroups require explicit recursive drain and removal"]
pub struct OwnedSubgroup {
    parent: PayloadParent,
    group: Group,
    name: String,
    removed: bool,
}
impl OwnedSubgroup {
    async fn validate(&self) -> Result<(), HostError> {
        if self.removed {
            return Err(HostError::Ownership(
                "original subgroup already removed".into(),
            ));
        }
        self.parent.validate().await?;
        validate_parent(
            &self.parent.ancestry[self.parent.ancestry.len() - 1],
            &self.group,
        )?;
        require_core(
            &self.parent.manager,
            &self
                .parent
                .manager
                .observe(self.parent.binding.intent.unit())
                .await?,
            std::slice::from_ref(&self.group),
        )
        .await
    }
    /// Original child coordinates, never permission to act on a replacement at that path.
    pub fn path(&self) -> &Path {
        &self.group.path
    }
    /// Inert original inode evidence, not a production adoption constructor.
    pub fn binding(&self) -> DirectoryBinding {
        DirectoryBinding::from_group(&self.group)
    }
    /// Lend this owned child as the exact parent for further descendants.
    pub async fn as_parent(&self) -> Result<PayloadParent, HostError> {
        self.validate().await?;
        let mut parent = self.parent.copy()?;
        parent.ancestry.push(copy_group(&self.group)?);
        parent
            .binding
            .ancestry
            .push(DirectoryBinding::from_group(&self.group));
        Ok(parent)
    }
    /// Original kernel population, not a launch-cutoff or workload-settlement assertion.
    pub fn population_empty(&self) -> Result<bool, HostError> {
        self.group.require_present()?;
        self.group.empty()
    }
    /// Attach only a locally owned unreaped child. Its execution gate remains with the caller.
    pub async fn attach(
        &self,
        child: &mut tokio::process::Child,
    ) -> Result<AttachedChild, HostError> {
        self.validate().await?;
        self.parent.require_launchable().await?;
        let pid = child
            .id()
            .ok_or_else(|| HostError::Ownership("subgroup child was already reaped".into()))?;
        let pid_value = i32::try_from(pid)
            .ok()
            .and_then(rustix::process::Pid::from_raw)
            .ok_or_else(|| HostError::Ownership("invalid original child PID".into()))?;
        let pidfd = rustix::process::pidfd_open(pid_value, rustix::process::PidfdFlags::empty())
            .map_err(io::Error::from)?;
        let process = child_identity(pid, &self.parent.manager)?;
        require_live(&pidfd)?;
        self.validate().await?;
        write_control(&self.group, "cgroup.procs", pid.to_string().as_bytes())?;
        require_live(&pidfd)?;
        if child_identity(pid, &self.parent.manager)? != process
            || Group::path(&process_group(pid)?)? != self.group.path
        {
            return Err(HostError::Ownership(
                "original child did not enter its selected subgroup".into(),
            ));
        }
        self.validate().await?;
        Ok(AttachedChild { process, pidfd })
    }
    /// Revalidate immediately before the actual lifecycle releases its still-held gate.
    pub async fn validate_attachment(
        &self,
        attachment: &AttachedChild,
        child: &tokio::process::Child,
    ) -> Result<(), HostError> {
        self.validate().await?;
        self.parent.require_launchable().await?;
        require_live(&attachment.pidfd)?;
        if child.id() != Some(attachment.process.pid)
            || child_identity(attachment.process.pid, &self.parent.manager)? != attachment.process
            || Group::path(&process_group(attachment.process.pid)?)? != self.group.path
        {
            return Err(HostError::Ownership(
                "gated child attachment changed before release".into(),
            ));
        }
        Ok(())
    }
    /// Recursive kernel kill. The supervisor separately prevents future attachments.
    pub async fn kill(&self) -> Result<(), HostError> {
        self.validate().await?;
        write_control(&self.group, "cgroup.kill", b"1")
    }
    /// Positive recursive drain observation, independent of any kill error or child reap.
    pub async fn wait_empty(&self, deadline: Instant) -> Result<(), HostError> {
        self.validate().await?;
        let events = self.group.watch()?;
        loop {
            self.group.require_present()?;
            if self.group.empty()? {
                return Ok(());
            }
            tokio::time::timeout_at(deadline.into(), GroupEvents::changed(events.as_ref()))
                .await
                .map_err(|_| HostError::Deadline("original subgroup population drain".into()))??;
        }
    }
    /// Remove every original directory after positive drain. Root replacement refuses.
    /// Removal is a kernel fact; caller must also close launch authority before final settlement.
    pub async fn remove_tree(&mut self, deadline: Instant) -> Result<RemovalReceipt, HostError> {
        self.validate().await?;
        if !self.group.empty()? {
            return Err(HostError::Ownership(
                "original subgroup remains populated".into(),
            ));
        }
        let (removed, mut failures) = remove_descendants(&self.group, deadline);
        let original = (self.group.device, self.group.inode);
        if Instant::now() >= deadline {
            failures.push(HostError::Deadline(
                "original subgroup removal cutoff".into(),
            ));
            return Ok(RemovalReceipt {
                removed_directories: removed,
                original_device: original.0,
                original_inode: original.1,
                original_removed: false,
                failures,
            });
        }
        if let Err(error) = self
            .parent
            .validate()
            .await
            .and_then(|()| self.group.require_present())
        {
            failures.push(error);
            return Ok(RemovalReceipt {
                removed_directories: removed,
                original_device: original.0,
                original_inode: original.1,
                original_removed: false,
                failures,
            });
        }
        let parent = &self.parent.ancestry[self.parent.ancestry.len() - 1];
        let removal = validate_parent(parent, &self.group)
            .and_then(|()| unlink_original(parent, &self.group, OsStr::new(&self.name)));
        match removal {
            Ok(()) => {
                self.removed = true;
                Ok(RemovalReceipt {
                    removed_directories: removed + 1,
                    original_device: original.0,
                    original_inode: original.1,
                    original_removed: true,
                    failures,
                })
            }
            Err(error) => {
                failures.push(error);
                Ok(RemovalReceipt {
                    removed_directories: removed,
                    original_device: original.0,
                    original_inode: original.1,
                    original_removed: false,
                    failures,
                })
            }
        }
    }
}
/// Retained original child identity/pidfd, independent of child exit and group removal.
#[derive(Debug)]
pub struct AttachedChild {
    pub(crate) process: ProcessIdentity,
    pub(crate) pidfd: std::os::fd::OwnedFd,
}
/// Original-inode removal facts and independent failures, without a lifecycle settlement claim.
#[derive(Debug)]
#[must_use = "removal receipts and every independent failure must reach the lifecycle owner"]
pub struct RemovalReceipt {
    pub(crate) removed_directories: u64,
    pub(crate) original_device: u64,
    pub(crate) original_inode: u64,
    pub(crate) original_removed: bool,
    pub(crate) failures: Vec<HostError>,
}
impl AttachedChild {
    /// Original child start identity. Retained pidfd custody is internal to attachment checks.
    pub fn process(&self) -> &ProcessIdentity {
        &self.process
    }
}
impl RemovalReceipt {
    /// Number of original directories positively removed by this attempt.
    pub fn removed_directories(&self) -> u64 {
        self.removed_directories
    }
    /// Original root of this subgroup removal, independent of any later pathname occupant.
    pub fn original_identity(&self) -> (u64, u64) {
        (self.original_device, self.original_inode)
    }
    /// Positive original-inode removal, not managed launch closure or terminal settlement.
    pub fn original_removed(&self) -> bool {
        self.original_removed
    }
    /// Every retained independent traversal/removal failure.
    pub fn failures(&self) -> &[HostError] {
        &self.failures
    }
}
fn child_identity(pid: u32, manager: &HostManager) -> Result<ProcessIdentity, HostError> {
    match ProcessReader::default().process(pid) {
        ProcessRead::Present(facts)
            if facts.identity.boot_id == manager.boot
                && facts.stat.parent_pid == std::process::id()
                && !matches!(facts.stat.state, 'Z' | 'X') =>
        {
            Ok(facts.identity)
        }
        _ => Err(HostError::Ownership(
            "original locally owned live child identity unavailable".into(),
        )),
    }
}
fn require_live(fd: &std::os::fd::OwnedFd) -> Result<(), HostError> {
    let mut poll = [rustix::event::PollFd::new(fd, rustix::event::PollFlags::IN)];
    if rustix::event::poll(
        &mut poll,
        Some(&rustix::event::Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        }),
    )
    .map_err(io::Error::from)?
        != 0
    {
        return Err(HostError::Ownership(
            "original child exited before subgroup attachment completed".into(),
        ));
    }
    Ok(())
}
fn write_control(group: &Group, name: &str, bytes: &[u8]) -> Result<(), HostError> {
    group.require_present()?;
    let fd = rustix::fs::openat(
        directory(group)?,
        name,
        rustix::fs::OFlags::WRONLY | rustix::fs::OFlags::CLOEXEC | rustix::fs::OFlags::NOFOLLOW,
        rustix::fs::Mode::empty(),
    )
    .map_err(io::Error::from)?;
    File::from(fd).write_all(bytes)?;
    Ok(())
}
fn unlink_original(parent: &Group, child: &Group, name: &OsStr) -> Result<(), HostError> {
    validate_parent(parent, child)?;
    if !child.empty()? {
        return Err(HostError::Ownership(
            "descendant subgroup became populated during removal".into(),
        ));
    }
    rustix::fs::unlinkat(directory(parent)?, name, rustix::fs::AtFlags::REMOVEDIR)
        .map_err(io::Error::from)?;
    if child.control("cgroup.events")?.is_some() || child.path.try_exists()? {
        return Err(HostError::Ownership(
            "original subgroup removal not positively observed or pathname replaced".into(),
        ));
    }
    Ok(())
}
fn remove_descendants(root: &Group, deadline: Instant) -> (u64, Vec<HostError>) {
    let start = copy_group(root).and_then(|group| {
        directory(&group)
            .and_then(|fd| {
                rustix::fs::Dir::read_from(fd)
                    .map_err(io::Error::from)
                    .map_err(HostError::from)
            })
            .map(|entries| (group, entries))
    });
    let mut stack = match start {
        Ok(frame) => vec![frame],
        Err(error) => return (0, vec![error]),
    };
    let mut removed = 0;
    let mut failures = Vec::new();
    loop {
        if Instant::now() >= deadline {
            failures.push(HostError::Deadline(
                "original subgroup recursive removal".into(),
            ));
            break;
        }
        let Some((group, entries)) = stack.last_mut() else {
            break;
        };
        if let Err(error) = group.require_present() {
            failures.push(error);
            stack.pop();
            continue;
        }
        match entries.next() {
            Some(Ok(entry))
                if entry.file_type() == rustix::fs::FileType::Directory
                    && entry.file_name().to_bytes() != b"."
                    && entry.file_name().to_bytes() != b".." =>
            {
                let name = OsString::from(OsStr::from_bytes(entry.file_name().to_bytes()));
                match open_child(group, &name).and_then(|child| {
                    directory(&child)
                        .and_then(|fd| {
                            rustix::fs::Dir::read_from(fd)
                                .map_err(io::Error::from)
                                .map_err(HostError::from)
                        })
                        .map(|entries| (child, entries))
                }) {
                    Ok(frame) => stack.push(frame),
                    Err(error) => failures.push(error),
                }
            }
            Some(Ok(_)) => {}
            Some(Err(error)) => {
                failures.push(io::Error::from(error).into());
                stack.pop();
            }
            None => {
                let Some((child, _)) = stack.pop() else {
                    failures.push(HostError::Ownership(
                        "original subgroup traversal state lost".into(),
                    ));
                    break;
                };
                if let Some((parent, _)) = stack.last() {
                    let removal = child
                        .path
                        .file_name()
                        .ok_or_else(|| HostError::Ownership("original child name absent".into()))
                        .and_then(|name| unlink_original(parent, &child, name));
                    match removal {
                        Ok(()) => removed += 1,
                        Err(error) => failures.push(error),
                    }
                }
            }
        }
    }
    (removed, failures)
}

#[cfg(test)]
mod tests;
