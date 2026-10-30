// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Opened original roots and actual members. These are kernel facts, not launch cutoff proof.
use super::*;
use std::ffi::OsStr;

pub(crate) mod payload;

/// Observation-only current placement. A descendant never replaces the manager's unit root.
#[derive(Debug)]
pub(crate) struct CurrentMembership {
    root: UnitObservation,
    manager: String,
    process: ProcessIdentity,
    /// Every component is opened from its original parent, including the original root.
    ancestry: Vec<Group>,
}
impl CurrentMembership {
    pub(crate) fn root(&self) -> &UnitObservation {
        &self.root
    }
    pub(super) fn into_root(self) -> UnitObservation {
        self.root
    }
    pub(super) async fn authenticate(
        manager: &HostManager,
        root: UnitObservation,
    ) -> Result<Self, HostError> {
        let process = current_process(manager)?;
        let member = process_group(process.pid)?;
        let original = Group::open(&root.control_group, false)?;
        let ancestry = open_chain(&original, &member)?;
        if ancestry.len() > 1 && !delegation(manager, &root).await?.0 {
            return Err(HostError::Ownership(
                "current subgroup is not beneath a delegated unit root".into(),
            ));
        }
        let value = Self {
            root,
            manager: manager.identity_key(),
            process,
            ancestry,
        };
        value.validate(manager).await?;
        Ok(value)
    }
    /// Exact actual member coordinates, separate from `root.control_group`.
    pub(crate) fn member(&self) -> &Path {
        &self.ancestry[self.ancestry.len() - 1].path
    }
    pub(crate) async fn validate(&self, manager: &HostManager) -> Result<(), HostError> {
        manager.validate_view()?;
        if self.manager != manager.identity_key() || current_process(manager)? != self.process {
            return Err(HostError::Ownership(
                "current member manager/process identity changed".into(),
            ));
        }
        let observed = manager.observe(&self.root.unit).await?;
        if observed.invocation != self.root.invocation
            || observed.control_group != self.root.control_group
        {
            return Err(HostError::Ownership(
                "current member original unit root changed".into(),
            ));
        }
        validate_chain(&self.ancestry)?;
        if Group::path(&process_group(self.process.pid)?)? != self.member() {
            return Err(HostError::Ownership(
                "actual process membership changed".into(),
            ));
        }
        Ok(())
    }
    /// Check actual controller-bearing ancestors; a core-only member has no memory files.
    pub(crate) fn require_memory_ceiling(
        &self,
        manager: &HostManager,
        requested: u64,
    ) -> Result<(), HostError> {
        validate_chain(&self.ancestry)?;
        let mut current = copy_group(&self.ancestry[self.ancestry.len() - 1])?;
        let mut finite = false;
        loop {
            if current.path == Path::new("/sys/fs/cgroup") {
                if (current.device, current.inode) != manager.cgroup_root {
                    return Err(HostError::Ownership(
                        "finite ancestry differs from the authenticated unified root".into(),
                    ));
                }
                break;
            }
            let parent = parent_group(&current)?;
            match current.control("memory.max")? {
                Some(value) if value.trim() == "max" => {}
                Some(value) => {
                    let limit = value.trim().parse::<u64>().map_err(|_| {
                        HostError::Ownership("actual memory ceiling is malformed".into())
                    })?;
                    finite = true;
                    if requested > limit {
                        return Err(HostError::Ownership(
                            "nested demand exceeds an actual enclosing memory ceiling".into(),
                        ));
                    }
                }
                None => {
                    let controllers =
                        parent.control("cgroup.subtree_control")?.ok_or_else(|| {
                            HostError::Ownership(
                                "actual ancestor controller state unavailable".into(),
                            )
                        })?;
                    if controllers
                        .split_whitespace()
                        .any(|value| value == "memory")
                    {
                        return Err(HostError::Ownership(
                            "enabled memory controller has no actual ceiling file".into(),
                        ));
                    }
                }
            }
            validate_parent(&parent, &current)?;
            current = parent;
        }
        if !finite {
            return Err(HostError::Ownership(
                "inherited finite allocation has no proven actual finite ceiling".into(),
            ));
        }
        validate_chain(&self.ancestry)
    }
}

fn current_process(manager: &HostManager) -> Result<ProcessIdentity, HostError> {
    match ProcessReader::default().process(std::process::id()) {
        ProcessRead::Present(facts) if facts.identity.boot_id == manager.boot => Ok(facts.identity),
        _ => Err(HostError::Ownership(
            "current original process identity unavailable".into(),
        )),
    }
}

async fn delegation(
    manager: &HostManager,
    root: &UnitObservation,
) -> Result<(bool, Vec<String>), HostError> {
    let unit = manager.unit(&root.unit).await?;
    if root.unit.ends_with(".scope") {
        let proxy = ScopeProxy::builder(&manager.connection)
            .destination(manager.owner.as_str())
            .path(unit.inner().path().to_owned())
            .cache_properties(CacheProperties::No)
            .build()
            .await?;
        Ok((proxy.delegate().await?, proxy.delegate_controllers().await?))
    } else if root.unit.ends_with(".service") {
        let proxy = ServiceProxy::builder(&manager.connection)
            .destination(manager.owner.as_str())
            .path(unit.inner().path().to_owned())
            .cache_properties(CacheProperties::No)
            .build()
            .await?;
        Ok((proxy.delegate().await?, proxy.delegate_controllers().await?))
    } else {
        Err(HostError::Ownership(
            "only scope/service roots support delegation".into(),
        ))
    }
}

fn directory(group: &Group) -> Result<&File, HostError> {
    group
        .directory
        .as_ref()
        .ok_or_else(|| HostError::Ownership("original subgroup directory unavailable".into()))
}
fn copy_group(group: &Group) -> Result<Group, HostError> {
    Group::from_directory(directory(group)?.try_clone()?, group.path.clone())
}
fn parent_group(child: &Group) -> Result<Group, HostError> {
    let path = child
        .path
        .parent()
        .filter(|path| path.starts_with("/sys/fs/cgroup"))
        .ok_or_else(|| {
            HostError::Ownership("opened cgroup ancestry escaped its unified root".into())
        })?;
    let fd = rustix::fs::openat(
        directory(child)?,
        "..",
        rustix::fs::OFlags::RDONLY
            | rustix::fs::OFlags::DIRECTORY
            | rustix::fs::OFlags::CLOEXEC
            | rustix::fs::OFlags::NOFOLLOW,
        rustix::fs::Mode::empty(),
    )
    .map_err(io::Error::from)?;
    let parent = Group::from_directory(File::from(fd), path.to_owned())?;
    validate_parent(&parent, child)?;
    Ok(parent)
}
fn open_child(parent: &Group, component: &OsStr) -> Result<Group, HostError> {
    if Path::new(component).components().count() != 1
        || !matches!(
            Path::new(component).components().next(),
            Some(Component::Normal(_))
        )
    {
        return Err(HostError::Ownership(
            "subgroup is not one strict path component".into(),
        ));
    }
    parent.require_present()?;
    let fd = rustix::fs::openat(
        directory(parent)?,
        component,
        rustix::fs::OFlags::RDONLY
            | rustix::fs::OFlags::DIRECTORY
            | rustix::fs::OFlags::CLOEXEC
            | rustix::fs::OFlags::NOFOLLOW,
        rustix::fs::Mode::empty(),
    )
    .map_err(io::Error::from)?;
    let child = Group::from_directory(File::from(fd), parent.path.join(component))?;
    parent.require_present()?;
    validate_parent(parent, &child)?;
    Ok(child)
}
fn validate_parent(parent: &Group, child: &Group) -> Result<(), HostError> {
    parent.require_present()?;
    child.require_present()?;
    let fd = rustix::fs::openat(
        directory(child)?,
        "..",
        rustix::fs::OFlags::RDONLY
            | rustix::fs::OFlags::DIRECTORY
            | rustix::fs::OFlags::CLOEXEC
            | rustix::fs::OFlags::NOFOLLOW,
        rustix::fs::Mode::empty(),
    )
    .map_err(io::Error::from)?;
    let meta = File::from(fd).metadata()?;
    if (meta.dev(), meta.ino()) != (parent.device, parent.inode) || child.device != parent.device {
        return Err(HostError::Ownership(
            "opened subgroup parent identity changed".into(),
        ));
    }
    Ok(())
}
fn open_chain(root: &Group, member: &str) -> Result<Vec<Group>, HostError> {
    let path = Group::path(member)?;
    let relative = path.strip_prefix(&root.path).map_err(|_| {
        HostError::Ownership("actual member is outside its original unit root".into())
    })?;
    let mut ancestry = vec![copy_group(root)?];
    for component in relative.components() {
        let Component::Normal(component) = component else {
            return Err(HostError::Ownership(
                "invalid actual membership component".into(),
            ));
        };
        let next = open_child(&ancestry[ancestry.len() - 1], component)?;
        ancestry.push(next);
    }
    validate_chain(&ancestry)?;
    Ok(ancestry)
}
fn validate_chain(ancestry: &[Group]) -> Result<(), HostError> {
    if ancestry.is_empty() {
        return Err(HostError::Ownership(
            "original subgroup ancestry absent".into(),
        ));
    }
    ancestry[0].require_present()?;
    for pair in ancestry.windows(2) {
        validate_parent(&pair[0], &pair[1])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
