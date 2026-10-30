// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Checkout-bound invocation evidence. Publication is exclusive; observation grants no control.

use crate::{
    HostError,
    envelope::{CliInvocationId, Envelope, ScientificReference, SubjectState},
    generation::PreparedGeneration,
    paths::CheckoutState,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const VERSION: u32 = 1;
const META_LIMIT: u64 = 4 * 1024 * 1024;

fn invalid(message: impl Into<String>) -> HostError {
    HostError::Ownership(format!("invocation evidence: {}", message.into()))
}

/// Inputs captured before the first launch effect. Operation details retain their owning type.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvocationSpec<O> {
    /// CLI identity, independent of product run/attempt and systemd identities.
    pub invocation_id: CliInvocationId,
    /// Literal CLI verb path, used to bind the terminal response.
    pub command: String,
    /// Session association, never used as a filesystem path.
    pub session: Option<String>,
    /// Parent invocation when explicitly supplied by its owner.
    pub parent: Option<CliInvocationId>,
    /// The actual resolved operation, including its own byte-exact command representation.
    pub operation: O,
    /// Immutable prepared tool reference; this record does not authenticate execution authority.
    pub control: PreparedGeneration,
    /// Digest of the captured environment; secret values are not stored here.
    pub environment_digest: String,
    /// Product-supplied references only.
    pub scientific: Vec<ScientificReference>,
}

/// Immutable selection and checkout binding; launch observations are separate owner evidence.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvocationMetadata<O> {
    /// Ledger format version, independent of the output and control protocols.
    pub version: u32,
    /// Inspected checkout identity, checked on every read.
    pub checkout_id: String,
    /// Shared Git repository relation, also checked on every read.
    pub repository_id: String,
    /// Creation time in UTC milliseconds since the Unix epoch.
    pub created_unix_ms: u64,
    /// Original captured selection.
    pub selection: InvocationSpec<O>,
}

/// One checkout's ledger root. Construction and reads create no paths.
#[derive(Clone, Debug)]
pub struct InvocationStore {
    state: CheckoutState,
}

/// A directory established by this store, not a manager mutation capability.
#[derive(Debug)]
pub struct InvocationDirectory {
    path: PathBuf,
    invocation_id: CliInvocationId,
    command: String,
}

fn directory(path: &Path, create: bool) -> Result<(), HostError> {
    if create {
        match fs::DirBuilder::new().mode(0o700).create(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.into()),
        }
    }
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_dir()
        || metadata.file_type().is_symlink()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o022 != 0
    {
        return Err(invalid(
            "ledger directory is not an owned, non-writable-by-others directory",
        ));
    }
    Ok(())
}

fn evidence_file(path: &Path) -> Result<File, HostError> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o022 != 0
    {
        return Err(invalid("evidence is not an owned regular file"));
    }
    Ok(file)
}

fn read_metadata<O: DeserializeOwned>(path: &Path) -> Result<InvocationMetadata<O>, HostError> {
    let file = evidence_file(&path.join("meta.json"))?;
    if file.metadata()?.len() > META_LIMIT {
        return Err(invalid(
            "selection metadata exceeds its declared storage bound",
        ));
    }
    let mut bytes = Vec::new();
    file.take(META_LIMIT + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > META_LIMIT {
        return Err(invalid(
            "selection metadata changed beyond its declared bound",
        ));
    }
    serde_json::from_slice(&bytes)
        .map_err(|error| invalid(format!("invalid selection metadata: {error}")))
}

/// Serialize privately, sync, then publish once. A failed writer cannot expose partial JSON.
fn publish<T: Serialize>(
    directory: &Path,
    name: &str,
    value: &T,
    limit: Option<u64>,
) -> Result<(), HostError> {
    let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
    serde_json::to_writer(temporary.as_file_mut(), value)
        .map_err(|error| invalid(format!("evidence serialization failed: {error}")))?;
    temporary.as_file_mut().write_all(b"\n")?;
    if limit.is_some_and(|limit| {
        temporary
            .as_file()
            .metadata()
            .map_or(true, |metadata| metadata.len() > limit)
    }) {
        return Err(invalid(
            "serialized evidence exceeds its declared storage bound",
        ));
    }
    temporary.as_file().sync_all()?;
    rustix::fs::renameat_with(
        rustix::fs::CWD,
        temporary.path(),
        rustix::fs::CWD,
        directory.join(name),
        rustix::fs::RenameFlags::NOREPLACE,
    )
    .map_err(std::io::Error::from)?;
    File::open(directory)?.sync_all()?;
    Ok(())
}

impl InvocationStore {
    /// Bind the inspected checkout. This does not create state or discover a database.
    pub fn new(state: CheckoutState) -> Self {
        Self { state }
    }

    fn validate<O>(
        &self,
        id: &CliInvocationId,
        metadata: &InvocationMetadata<O>,
    ) -> Result<(), HostError> {
        if metadata.version != VERSION
            || metadata.checkout_id != self.state.checkout_id.as_str()
            || metadata.repository_id != self.state.repository_id.as_str()
            || &metadata.selection.invocation_id != id
        {
            return Err(invalid(
                "unsupported metadata or mismatched invocation/checkout binding",
            ));
        }
        Ok(())
    }

    /// Publish the captured selection before starting children or acquiring their resources.
    pub fn begin<O: Serialize>(
        &self,
        selection: InvocationSpec<O>,
    ) -> Result<InvocationDirectory, HostError> {
        let parent = self
            .state
            .root
            .parent()
            .ok_or_else(|| invalid("state root has no parent"))?;
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(parent)?;
        directory(parent, false)?;
        directory(&self.state.root, true)?;
        let invocations = self.state.invocations();
        directory(&invocations, true)?;
        let created_unix_ms = u64::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| invalid("system clock precedes the Unix epoch"))?
                .as_millis(),
        )
        .map_err(|_| invalid("creation timestamp exceeds the ledger representation"))?;
        let metadata = InvocationMetadata {
            version: VERSION,
            checkout_id: self.state.checkout_id.as_str().to_owned(),
            repository_id: self.state.repository_id.as_str().to_owned(),
            created_unix_ms,
            selection,
        };
        // No invocation is visible until its complete metadata exists. Private preparation
        // directories contain no launched work and are outside the committed ID namespace.
        let stage = tempfile::Builder::new()
            .prefix(".prepare-")
            .tempdir_in(&invocations)?;
        fs::set_permissions(stage.path(), fs::Permissions::from_mode(0o700))?;
        publish(stage.path(), "meta.json", &metadata, Some(META_LIMIT))?;
        let path = invocations.join(metadata.selection.invocation_id.as_str());
        rustix::fs::renameat_with(
            rustix::fs::CWD,
            stage.path(),
            rustix::fs::CWD,
            &path,
            rustix::fs::RenameFlags::NOREPLACE,
        )
        .map_err(std::io::Error::from)?;
        File::open(&invocations)?.sync_all()?;
        Ok(InvocationDirectory {
            path,
            invocation_id: metadata.selection.invocation_id,
            command: metadata.selection.command,
        })
    }

    /// Read the exact versioned selection; no unsupported record is silently omitted.
    pub fn metadata<O: DeserializeOwned>(
        &self,
        id: &CliInvocationId,
    ) -> Result<InvocationMetadata<O>, HostError> {
        directory(&self.state.root, false)?;
        directory(&self.state.invocations(), false)?;
        let path = self.state.invocations().join(id.as_str());
        directory(&path, false)?;
        let metadata = read_metadata(&path)?;
        self.validate(id, &metadata)?;
        Ok(metadata)
    }

    /// Select an exact ID, a unique prefix or latest within session, falling back to checkout.
    /// Missing state is an empty observation. Malformed evidence remains an error.
    pub fn resolve(
        &self,
        selector: &str,
        session: Option<&str>,
    ) -> Result<Option<CliInvocationId>, HostError> {
        if selector.is_empty()
            || (selector != "latest"
                && !selector
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() || byte == b'-'))
        {
            return Err(invalid(
                "selector must be latest, an invocation ID or its literal prefix",
            ));
        }
        let records = self.history::<Box<serde_json::value::RawValue>>()?;
        if selector == "latest" {
            let latest = |session_only: bool| {
                records
                    .iter()
                    .filter(|record| {
                        !session_only || record.selection.session.as_deref() == session
                    })
                    .max_by(|left, right| {
                        (left.created_unix_ms, left.selection.invocation_id.as_str()).cmp(&(
                            right.created_unix_ms,
                            right.selection.invocation_id.as_str(),
                        ))
                    })
                    .map(|record| record.selection.invocation_id.clone())
            };
            return Ok(session.and_then(|_| latest(true)).or_else(|| latest(false)));
        }
        let mut matches = records.into_iter().filter(|record| {
            record
                .selection
                .invocation_id
                .as_str()
                .starts_with(selector)
        });
        let selected = matches.next().map(|record| record.selection.invocation_id);
        if matches.next().is_some() {
            return Err(invalid("invocation prefix is ambiguous"));
        }
        Ok(selected)
    }

    /// Full source records, sorted by creation time and exact ID; callers own presentation pages.
    pub fn history<O: DeserializeOwned>(&self) -> Result<Vec<InvocationMetadata<O>>, HostError> {
        match fs::symlink_metadata(&self.state.invocations()) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error.into()),
            Ok(_) => {}
        }
        directory(&self.state.root, false)?;
        directory(&self.state.invocations(), false)?;
        let mut records = Vec::new();
        for entry in fs::read_dir(self.state.invocations())? {
            let entry = entry?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| invalid("invalid invocation directory name"))?;
            if name.starts_with(".prepare-") {
                continue;
            }
            let id = CliInvocationId::try_from(name)
                .map_err(|_| invalid("invalid invocation directory identity"))?;
            let path = entry.path();
            directory(&path, false)?;
            let metadata = read_metadata(&path)?;
            self.validate(&id, &metadata)?;
            records.push(metadata);
        }
        records.sort_by(|left, right| {
            (left.created_unix_ms, left.selection.invocation_id.as_str()).cmp(&(
                right.created_unix_ms,
                right.selection.invocation_id.as_str(),
            ))
        });
        Ok(records)
    }

    /// Absence means no terminal record was observed, never completed success.
    pub fn result<O: DeserializeOwned, T: DeserializeOwned>(
        &self,
        id: &CliInvocationId,
    ) -> Result<Option<Envelope<T>>, HostError> {
        let metadata: InvocationMetadata<O> = self.metadata(id)?;
        let path = self.state.invocations().join(id.as_str());
        let file = match evidence_file(&path.join("result.json")) {
            Err(HostError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(None);
            }
            Err(error) => return Err(error),
            Ok(file) => file,
        };
        let result: Envelope<T> = serde_json::from_reader(file)
            .map_err(|error| invalid(format!("invalid terminal response: {error}")))?;
        validate_result(id, &metadata.selection.command, &result)?;
        Ok(Some(result))
    }
}

fn validate_result<T>(
    id: &CliInvocationId,
    command: &str,
    result: &Envelope<T>,
) -> Result<(), HostError> {
    if &result.invocation_id != id || result.command != command || result.version != VERSION {
        return Err(invalid(
            "terminal response does not match its original invocation",
        ));
    }
    if matches!(
        result.subject_state,
        SubjectState::Launched | SubjectState::Running
    ) {
        return Err(invalid(
            "nonterminal acknowledgement cannot be published as a terminal result",
        ));
    }
    if result.view.truncated || result.view.fields.is_some() {
        return Err(invalid(
            "a presentation projection cannot replace the retained full response",
        ));
    }
    Ok(())
}

impl InvocationDirectory {
    /// Original ledger identity.
    pub fn id(&self) -> &CliInvocationId {
        &self.invocation_id
    }
    /// Existing authenticated directory for the original output/observation owners.
    pub fn path(&self) -> &Path {
        &self.path
    }
    /// Publish exactly one immutable full response. This does not establish its stated facts.
    pub fn publish_result<T: Serialize>(&self, result: &Envelope<T>) -> Result<(), HostError> {
        validate_result(&self.invocation_id, &self.command, result)?;
        directory(&self.path, false)?;
        publish(&self.path, "result.json", result, None)
    }
}

#[cfg(test)]
mod tests;
