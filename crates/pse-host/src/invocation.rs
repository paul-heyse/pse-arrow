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

/// Fixed immutable resource-owner records; none is a terminal admission or mutation grant.
#[derive(Clone, Copy, Debug)]
pub enum InvocationEvidence {
    /// Original service launch selection and intent.
    Launch,
    /// Original initialization and physical resource identities.
    Custody,
    /// The actual resource owner's settlement facts, separate from a terminal projection.
    Settlement,
}
impl InvocationEvidence {
    fn filename(self) -> &'static str {
        match self {
            Self::Launch => "launch.json",
            Self::Custody => "custody.json",
            Self::Settlement => "settlement.json",
        }
    }
}

/// Held original-parent launch exclusion. Only this owner can publish the cutoff.
#[derive(Debug)]
pub struct InvocationLaunchGuard<'a> {
    directory: &'a InvocationDirectory,
    _lock: File,
}
impl InvocationLaunchGuard<'_> {
    /// Establish the original explicit cutoff while excluding concurrent child admission.
    pub fn close(&self) -> Result<(), HostError> {
        if !self.directory.launch_closed()? {
            publish(
                &self.directory.path,
                "launch.closed",
                &true,
                Some(META_LIMIT),
            )?;
        }
        Ok(())
    }
    /// Inspect the cutoff while retaining the same exclusion across a launch decision.
    pub fn is_closed(&self) -> Result<bool, HostError> {
        self.directory.launch_closed()
    }
}

/// One independent unreadable source in a history observation.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("invocation history source {path:?}: {error}")]
pub struct HistoryFailure {
    /// Original source path, retaining platform bytes.
    pub path: PathBuf,
    /// Parsed directory identity, when the source supplied a valid one.
    pub invocation_id: Option<CliInvocationId>,
    /// Original typed source failure.
    #[source]
    #[diagnostic_source]
    pub error: HostError,
}
impl pse_diagnostics::TypedDiagnostic for HistoryFailure {
    fn diagnostic_code(&self) -> Option<pse_diagnostics::DiagnosticCode> {
        self.error.diagnostic_code()
    }
    fn diagnostic_facts(&self) -> pse_diagnostics::DiagnosticFacts {
        use pse_diagnostics::DiagnosticObservation;
        let mut facts = self.error.diagnostic_facts();
        facts.observe(
            "evidence_path_platform_bytes",
            DiagnosticObservation::Text(
                self.path
                    .as_os_str()
                    .as_encoded_bytes()
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect(),
            ),
        );
        if let Some(id) = &self.invocation_id {
            facts.observe("invocation_id", DiagnosticObservation::Text(id.to_string()));
        }
        facts
    }
    fn diagnostic_children(
        &self,
    ) -> Option<Box<dyn Iterator<Item = &dyn pse_diagnostics::TypedDiagnostic> + '_>> {
        let error: &dyn pse_diagnostics::TypedDiagnostic = &self.error;
        Some(Box::new(std::iter::once(error)))
    }
}

/// Successfully authenticated records and every failed source in the same traversal.
#[derive(Debug)]
pub struct InvocationHistory<O> {
    /// Source records sorted by creation time and exact ID.
    pub records: Vec<InvocationMetadata<O>>,
    /// Independent failures; a nonempty set prevents a global latest claim.
    pub failures: Vec<HistoryFailure>,
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

    /// Reopen only the original checkout-bound directory. This grants evidence access,
    /// never systemd reference, process, permit or finalization authority.
    pub fn directory(&self, id: &CliInvocationId) -> Result<InvocationDirectory, HostError> {
        let metadata: InvocationMetadata<serde::de::IgnoredAny> = self.metadata(id)?;
        Ok(InvocationDirectory {
            path: self.state.invocations().join(id.as_str()),
            invocation_id: id.clone(),
            command: metadata.selection.command,
        })
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
    /// Exact/prefix access validates only the chosen record; latest requires valid full history.
    /// Missing state is an empty observation. Malformed chosen evidence remains an error.
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
        if selector == "latest" {
            let history = self.history::<serde::de::IgnoredAny>()?;
            if !history.failures.is_empty() {
                return Err(HostError::IncompleteInvocationHistory {
                    failures: history.failures,
                });
            }
            let records = history.records;
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
        match fs::symlink_metadata(self.state.invocations()) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
            Ok(_) => {}
        }
        directory(&self.state.root, false)?;
        directory(&self.state.invocations(), false)?;
        let selected = if let Ok(id) = CliInvocationId::try_from(selector.to_owned()) {
            // A full ID does not depend on any other invocation's name or evidence.
            match fs::symlink_metadata(self.state.invocations().join(id.as_str())) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(error) => return Err(error.into()),
                Ok(_) => Some(id),
            }
        } else {
            let mut selected = None;
            for entry in fs::read_dir(self.state.invocations())? {
                let name = entry?
                    .file_name()
                    .into_string()
                    .map_err(|_| invalid("invalid invocation directory name"))?;
                if name.starts_with(".prepare-") {
                    continue;
                }
                let id = CliInvocationId::try_from(name)
                    .map_err(|_| invalid("invalid invocation directory identity"))?;
                if id.as_str().starts_with(selector) {
                    if selected.is_some() {
                        return Err(invalid("invocation prefix is ambiguous"));
                    }
                    selected = Some(id);
                }
            }
            selected
        };
        if let Some(id) = &selected {
            // Establish prefix ambiguity from identities before opening any candidate.
            // Missing or malformed metadata in the chosen directory remains an error.
            self.metadata::<serde::de::IgnoredAny>(id)?;
        }
        Ok(selected)
    }

    /// Full source records, sorted by creation time and exact ID; callers own presentation pages.
    pub fn history<O: DeserializeOwned>(&self) -> Result<InvocationHistory<O>, HostError> {
        match fs::symlink_metadata(self.state.invocations()) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(InvocationHistory {
                    records: Vec::new(),
                    failures: Vec::new(),
                });
            }
            Err(error) => return Err(error.into()),
            Ok(_) => {}
        }
        directory(&self.state.root, false)?;
        directory(&self.state.invocations(), false)?;
        let mut records = Vec::new();
        let mut failures = Vec::new();
        for entry in fs::read_dir(self.state.invocations())? {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    failures.push(HistoryFailure {
                        path: self.state.invocations(),
                        invocation_id: None,
                        error: error.into(),
                    });
                    continue;
                }
            };
            let name = entry.file_name();
            if name.as_encoded_bytes().starts_with(b".prepare-") {
                continue;
            }
            let path = entry.path();
            let id = name
                .into_string()
                .ok()
                .and_then(|name| CliInvocationId::try_from(name).ok());
            let record = (|| {
                let id = id
                    .as_ref()
                    .ok_or_else(|| invalid("invalid invocation directory identity"))?;
                directory(&path, false)?;
                let metadata = read_metadata(&path)?;
                self.validate(id, &metadata)?;
                Ok(metadata)
            })();
            match record {
                Ok(record) => records.push(record),
                Err(error) => failures.push(HistoryFailure {
                    path,
                    invocation_id: id,
                    error,
                }),
            }
        }
        records.sort_by(|left, right| {
            (left.created_unix_ms, left.selection.invocation_id.as_str()).cmp(&(
                right.created_unix_ms,
                right.selection.invocation_id.as_str(),
            ))
        });
        failures.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(InvocationHistory { records, failures })
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
    /// Open an exact original stream through the protected local evidence boundary.
    /// The descriptor is a live mutable observation, never an atomic snapshot.
    pub fn stream(&self, stream: crate::extraction::OutputStream) -> Result<File, HostError> {
        directory(&self.path, false)?;
        evidence_file(&self.path.join(match stream {
            crate::extraction::OutputStream::Stdout => "stdout.raw",
            crate::extraction::OutputStream::Stderr => "stderr.raw",
        }))
    }
    /// Publish original owner evidence exactly once. Facts must be constructed and
    /// authenticated by the performing owner; serialization conveys no authority.
    pub fn publish_evidence<T: Serialize>(
        &self,
        record: InvocationEvidence,
        value: &T,
    ) -> Result<(), HostError> {
        directory(&self.path, false)?;
        publish(&self.path, record.filename(), value, Some(META_LIMIT))
    }
    /// Read one exact owner record. Missing evidence is absent, never settled success.
    pub fn evidence<T: DeserializeOwned>(
        &self,
        record: InvocationEvidence,
    ) -> Result<Option<T>, HostError> {
        self.read_record(record.filename())
    }
    fn read_record<T: DeserializeOwned>(&self, name: &str) -> Result<Option<T>, HostError> {
        directory(&self.path, false)?;
        let file = match evidence_file(&self.path.join(name)) {
            Err(HostError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(None);
            }
            result => result?,
        };
        if file.metadata()?.len() > META_LIMIT {
            return Err(invalid("owner record exceeds declared bound"));
        }
        let mut bytes = Vec::new();
        file.take(META_LIMIT + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > META_LIMIT {
            return Err(invalid("owner record grew beyond declared bound"));
        }
        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|error| invalid(format!("owner record decode failed: {error}")))
    }
    /// Publish ephemeral private service input. Secrets belong only here, never immutable
    /// metadata, diagnostics or launch records. The supervisor consumes it before readiness.
    pub fn publish_request<T: Serialize>(&self, request: &T) -> Result<(), HostError> {
        directory(&self.path, false)?;
        publish(&self.path, "request.json", request, Some(META_LIMIT))
    }
    /// Consume private input once. A decode/removal/sync failure refuses readiness and is
    /// not permission to replay a possibly submitted launch. No plaintext is retained on success.
    pub fn take_request<T: DeserializeOwned>(&self) -> Result<T, HostError> {
        let request = self
            .read_record("request.json")?
            .ok_or_else(|| invalid("private service request absent or already consumed"))?;
        fs::remove_file(self.path.join("request.json"))?;
        File::open(&self.path)?.sync_all()?;
        Ok(request)
    }
    /// Try the original parent launch exclusion without blocking an async runtime thread.
    /// The owner holds this descriptor across admission and closure/creation effects.
    pub fn try_launch_lock(&self) -> Result<InvocationLaunchGuard<'_>, HostError> {
        directory(&self.path, false)?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
            .open(self.path.join("launch.lock"))?;
        let metadata = file.metadata()?;
        if !metadata.is_file()
            || metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.mode() & 0o022 != 0
        {
            return Err(invalid(
                "launch exclusion is not an owned private regular file",
            ));
        }
        rustix::fs::flock(&file, rustix::fs::FlockOperation::NonBlockingLockExclusive)
            .map_err(std::io::Error::from)?;
        Ok(InvocationLaunchGuard {
            directory: self,
            _lock: file,
        })
    }
    /// This original owner's explicit cutoff, never inferred from process state.
    pub fn launch_closed(&self) -> Result<bool, HostError> {
        match self.read_record::<bool>("launch.closed")? {
            Some(true) => Ok(true),
            None => Ok(false),
            Some(false) => Err(invalid("invalid launch closure record")),
        }
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
