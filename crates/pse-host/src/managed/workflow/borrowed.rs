// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Handoff of managed workflow and campaign custody to an enclosing native workload owner.
use super::{
    complete_capture, digest, host, invalid, release_interrupted_listing, release_original_images,
};
use crate::{
    HostError, HostManager,
    envelope::{CliInvocationId, RedactionPolicy, Settlement},
    environment::Environment,
    foreground::ForegroundCustody,
    invocation::{DeferredTestDrain, InvocationDirectory, InvocationEvidence, InvocationStore},
    manager::UnitBinding,
    paths::CheckoutState,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Read as _},
    os::unix::fs::{DirBuilderExt as _, MetadataExt as _},
    path::{Path, PathBuf},
    time::Instant,
};

const RECORD_LIMIT: usize = 16 * 1024;

/// Routing data only. Actual manager, cgroup and performing-child identity stay
/// in the original foreground custody record, independently of this handoff.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Route {
    checkout: String,
    state_base: String,
}
impl Route {
    pub(crate) fn new(checkout: &Path, state_base: &Path) -> Self {
        Self {
            checkout: super::super::request::encode_path(checkout),
            state_base: super::super::request::encode_path(state_base),
        }
    }
    fn store(&self) -> Result<InvocationStore, HostError> {
        let checkout = super::super::request::decode_path(&self.checkout)?;
        let base = super::super::request::decode_path(&self.state_base)?;
        let environment = Environment::from([("XDG_STATE_HOME".into(), base.into_os_string())]);
        Ok(InvocationStore::new(CheckoutState::resolve(
            &checkout,
            &environment,
        )?))
    }
}

fn queue(runtime: &Path, manager: &str, binding: &UnitBinding) -> Result<PathBuf, HostError> {
    let key = serde_json::to_vec(&(manager, binding)).map_err(host)?;
    Ok(runtime.join("pse-workflow-release").join(digest(&key)))
}

fn private_directory(path: &Path, create: bool) -> Result<bool, HostError> {
    if create {
        match fs::DirBuilder::new().mode(0o700).create(path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.into()),
        }
    }
    match fs::symlink_metadata(path) {
        Ok(metadata)
            if metadata.is_dir()
                && metadata.uid() == rustix::process::geteuid().as_raw()
                && metadata.mode() & 0o777 == 0o700 =>
        {
            Ok(true)
        }
        Ok(_) => Err(host(invalid(
            "workflow release queue lacks private custody",
        ))),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn queue_directory(path: &Path, create: bool) -> Result<bool, HostError> {
    let parent = path
        .parent()
        .ok_or_else(|| host(invalid("workflow queue parent absent")))?;
    Ok(private_directory(parent, create)? && private_directory(path, create)?)
}

/// Register the exact original invocation before releasing its child gate.
pub(crate) fn register(
    runtime: &Path,
    directory: &InvocationDirectory,
    route: &Route,
    custody: &ForegroundCustody,
) -> Result<(), HostError> {
    let queue = queue(runtime, &custody.manager, &custody.root)?;
    queue_directory(&queue, true)?;
    let bytes = serde_json::to_vec(route).map_err(host)?;
    crate::invocation::publish_bytes(
        &queue.join(directory.id().as_str()),
        &bytes,
        Some(RECORD_LIMIT),
        0o400,
    )
}

fn release_recorded(
    store: &InvocationStore,
    directory: &InvocationDirectory,
    borrowed: bool,
    authenticated_custody: &str,
) -> Result<(), HostError> {
    let result = store
        .result::<serde::de::IgnoredAny, crate::managed::result::ManagedSummary>(directory.id())?
        .ok_or_else(|| host(invalid("original foreground result is absent")))?;
    if directory.is_conformance() {
        if !borrowed || !result.payload.borrowed_reservation {
            return Err(host(invalid(
                "original campaign did not borrow this enclosing workload",
            )));
        }
        return directory.record_conformance_enclosing_drain(authenticated_custody);
    }
    if result.payload.borrowed_reservation == borrowed
        && release_interrupted_listing(directory, &result)?
    {
        return Ok(());
    }
    let workflow = result.payload.workflow.as_ref().ok_or_else(|| {
        host(invalid(
            "original foreground workflow transport is incomplete",
        ))
    })?;
    if result.payload.borrowed_reservation != borrowed
        || result.payload.native_lifetime_pending != Some(false)
        || result.settlement != Settlement::Complete
        || !complete_capture(&result)
        || !workflow.terminal.required_capture_complete
    {
        return Err(host(invalid(
            "original foreground workflow capture is incomplete",
        )));
    }
    let released = release_original_images(store, directory, workflow.terminal.action)?;
    if borrowed {
        released.publish_compact(
            store,
            directory,
            DeferredTestDrain::Borrowed,
            &RedactionPolicy::from_environment(),
        )?;
    }
    Ok(())
}

fn custody(directory: &InvocationDirectory) -> Result<(ForegroundCustody, String), HostError> {
    let bytes = directory
        .evidence_bytes(InvocationEvidence::Custody)?
        .ok_or_else(|| host(invalid("original foreground workflow custody is absent")))?;
    let custody: ForegroundCustody = serde_json::from_slice(&bytes).map_err(host)?;
    if custody.version != 4 {
        return Err(host(invalid("unsupported foreground workflow custody")));
    }
    Ok((custody, digest(&bytes)))
}

fn remove_queue(path: &Path) -> Result<(), HostError> {
    match fs::remove_dir(path) {
        Ok(()) => Ok(()),
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::NotFound | io::ErrorKind::DirectoryNotEmpty
            ) =>
        {
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

pub(crate) fn enclosing_pending(
    runtime: &Path,
    manager: &str,
    binding: &UnitBinding,
) -> Result<bool, HostError> {
    queue_directory(&queue(runtime, manager, binding)?, false)
}

/// Called by the actual enclosing owner after positive workload drain. A stop-post
/// caller must first prove it is the sole surviving process in that root.
pub(crate) fn reconcile_enclosing(
    runtime: &Path,
    manager: &str,
    unit: &str,
    binding: &UnitBinding,
    deadline: Instant,
) -> Result<(), HostError> {
    let path = queue(runtime, manager, binding)?;
    if !queue_directory(&path, false)? {
        return Ok(());
    }
    let mut first_error = None;
    for (index, entry) in fs::read_dir(&path)?.enumerate() {
        if index >= 4096 || Instant::now() >= deadline {
            first_error.get_or_insert_with(|| {
                HostError::Deadline("borrowed workflow release handoff".into())
            });
            break;
        }
        let reconcile = || -> Result<(), HostError> {
            let entry = entry?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| host(invalid("invalid borrowed workflow queue identity")))?;
            let id = CliInvocationId::try_from(name.clone())
                .map_err(|_| host(invalid("invalid borrowed workflow queue identity")))?;
            if id.as_str() != name {
                return Err(host(invalid(
                    "noncanonical borrowed workflow queue identity",
                )));
            }
            let record = crate::test_inventory::open_private(&entry.path())?;
            let original = record.metadata()?;
            if original.mode() & 0o777 != 0o400 || original.nlink() != 1 {
                return Err(host(invalid(
                    "borrowed workflow queue entry lacks private custody",
                )));
            }
            let mut bytes = Vec::new();
            (&record)
                .take(RECORD_LIMIT as u64 + 1)
                .read_to_end(&mut bytes)?;
            if bytes.len() > RECORD_LIMIT {
                return Err(host(invalid(
                    "borrowed workflow queue entry exceeds its bound",
                )));
            }
            let route: Route = serde_json::from_slice(&bytes).map_err(host)?;
            let store = route.store()?;
            let directory = store.directory(&id)?;
            let (original_custody, authenticated_custody) = custody(&directory)?;
            if original_custody.manager != manager
                || original_custody.unit != unit
                || original_custody.root != *binding
            {
                return Err(host(invalid(
                    "borrowed workflow belongs to a different enclosing workload",
                )));
            }
            release_recorded(&store, &directory, true, &authenticated_custody)?;
            match fs::symlink_metadata(entry.path()) {
                Ok(current)
                    if (original.dev(), original.ino()) == (current.dev(), current.ino()) =>
                {
                    fs::remove_file(entry.path())?
                }
                Ok(_) => return Err(host(invalid("borrowed workflow queue entry changed"))),
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
            Ok(())
        };
        if let Err(error) = reconcile() {
            first_error.get_or_insert(error);
        }
    }
    remove_queue(&path)?;
    first_error.map_or(Ok(()), Err)
}

/// Explicit retry for one original foreground workflow or campaign. An inactive name alone
/// never grants release, and the enclosing workload is independently owned.
pub(crate) async fn reconcile_foreground(
    store: &InvocationStore,
    directory: &InvocationDirectory,
    borrowed_runtime: Option<&Path>,
) -> Result<bool, HostError> {
    let (custody, authenticated_custody) = custody(directory)?;
    let manager = HostManager::connect_user(None).await?;
    if !manager
        .foreground_binding_drained(
            &custody.manager,
            &custody.unit,
            &custody.root,
            &custody.child,
        )
        .await?
    {
        return Ok(false);
    }
    release_recorded(
        store,
        directory,
        borrowed_runtime.is_some(),
        &authenticated_custody,
    )?;
    let Some(runtime) = borrowed_runtime else {
        return Ok(true);
    };
    let path = queue(runtime, &custody.manager, &custody.root)?;
    if queue_directory(&path, false)? {
        let record = path.join(directory.id().as_str());
        match fs::symlink_metadata(&record) {
            Ok(metadata)
                if metadata.is_file()
                    && metadata.uid() == rustix::process::geteuid().as_raw()
                    && metadata.mode() & 0o777 == 0o400
                    && metadata.nlink() == 1 =>
            {
                match fs::remove_file(record) {
                    Ok(()) => {}
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error.into()),
                }
            }
            Ok(_) => {
                return Err(host(invalid(
                    "borrowed workflow queue entry lacks private custody",
                )));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        remove_queue(&path)?;
    }
    Ok(true)
}
