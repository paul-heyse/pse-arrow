// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Release only the enrolled members of one authenticated, settled test workflow.

use super::{
    InvocationDirectory, InvocationEvidence, InvocationStore, META_LIMIT, directory, evidence_file, invalid, publish,
    synchronize_publication_parent,
};
use crate::{
    HostError,
    artifacts::{DescriptorIdentity, descriptor_identity},
    envelope::CliInvocationId,
    test_inventory::captured_report_at_limit,
    test_workflow::execute::{
        ContentIdentity, EvidenceRole, EvidenceRoot, ReportReference, evidence,
    },
};
use rustix::fs::{AtFlags, FlockOperation, Mode, OFlags, flock, openat, unlinkat};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{self, Read},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
};

const ENROLLMENT: &str = "workflow-retention.json";
const ADMISSION: &str = "workflow-retention-admission.json";
const RETIRED: &str = "workflow-retired.json";
const WORKFLOW_READ_LIMIT: usize = 64 * 1024 * 1024;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Enrollment {
    invocation: CliInvocationId,
    checkout: String,
    report_sha256: String,
    report_byte_length: u64,
    members: Vec<Member>,
    /// A durable intent allows a later sweep to continue a partly completed release.
    releasing: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Member {
    path: String,
    content: ContentIdentity,
    identity: DescriptorIdentity,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Retirement {
    invocation: CliInvocationId,
    report_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    raw: Option<RetiredRaw>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RetiredRaw {
    receipt: DescriptorIdentity,
    stdout: bool,
    stderr: bool,
}

/// Exact bounded raw streams identified by the performing owner's open descriptors.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorkflowStreams {
    pub(crate) invocation: CliInvocationId,
    pub(crate) stdout: Option<DescriptorIdentity>,
    pub(crate) stderr: Option<DescriptorIdentity>,
}

pub(super) fn streams(original: &InvocationDirectory) -> Result<Option<WorkflowStreams>, HostError> {
    Ok(stream_receipt(original)?.map(|(record, _)| record))
}

fn stream_receipt(original: &InvocationDirectory) -> Result<Option<(WorkflowStreams, DescriptorIdentity)>, HostError> {
    let owner = private_root(original.path())?;
    let name = Path::new(InvocationEvidence::WorkflowStreams.filename());
    let file = match open_child(&owner, name) {
        Ok(file) => file,
        Err(HostError::Io(error)) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let receipt = descriptor_identity(&file)?;
    let mut bytes = Vec::new();
    (&file).take(4097).read_to_end(&mut bytes)?;
    if bytes.len() > 4096 || descriptor_identity(&file)? != receipt {
        return Err(invalid("raw stream receipt is unbounded or changed while observed"));
    }
    let record: WorkflowStreams = serde_json::from_slice(&bytes).map_err(io::Error::other)?;
    if &record.invocation != original.id()
        || record.members().any(|(_, identity)| identity.bytes > crate::managed::streams::WORKFLOW_PIPE_BYTES)
    {
        return Err(invalid("raw stream commitments differ from the original bounded operation"));
    }
    Ok(Some((record, receipt)))
}

impl WorkflowStreams {
    fn members(&self) -> impl Iterator<Item = (&'static str, &DescriptorIdentity)> {
        [("stdout.raw", self.stdout.as_ref()), ("stderr.raw", self.stderr.as_ref())]
            .into_iter()
            .filter_map(|(name, identity)| identity.map(|identity| (name, identity)))
    }

    pub(super) fn prepare_release(
        &self,
        parent: &File,
        missing_ok: bool,
    ) -> Result<Vec<RawRelease>, HostError> {
        let mut selected = Vec::new();
        for (name, expected) in self.members() {
            let file = match open_child(parent, Path::new(name)) {
                Ok(file) => file,
                Err(HostError::Io(error)) if missing_ok && error.kind() == io::ErrorKind::NotFound => {
                    continue;
                }
                Err(error) => return Err(error),
            };
            flock(&file, FlockOperation::NonBlockingLockExclusive).map_err(io::Error::from)?;
            if descriptor_identity(&file)? != *expected
                || descriptor_identity(&open_child(parent, Path::new(name))?)? != *expected
            {
                return Err(invalid("bounded raw stream was replaced before release"));
            }
            selected.push(RawRelease { name, original: file, identity: expected.clone() });
        }
        Ok(selected)
    }
}

pub(super) struct RawRelease {
    name: &'static str,
    original: File,
    identity: DescriptorIdentity,
}

impl RawRelease {
    pub(super) fn unlink(self, parent: &File) -> Result<u64, HostError> {
        let name = Path::new(self.name);
        if descriptor_identity(&self.original)? != self.identity
            || descriptor_identity(&open_child(parent, name)?)? != self.identity
        {
            return Err(invalid("bounded raw stream changed during release"));
        }
        unlinkat(parent, name, AtFlags::empty()).map_err(io::Error::from)?;
        parent.sync_all()?;
        Ok(self.identity.bytes)
    }
}

fn private_root(path: &Path) -> Result<File, HostError> {
    directory(path, false)?;
    let root = OpenOptions::new()
        .read(true)
        .custom_flags((OFlags::DIRECTORY | OFlags::NOFOLLOW).bits() as i32)
        .open(path)?;
    let metadata = root.metadata()?;
    let named = fs::symlink_metadata(path)?;
    if (metadata.dev(), metadata.ino()) != (named.dev(), named.ino()) {
        return Err(invalid("workflow evidence root changed during access"));
    }
    Ok(root)
}

fn workflow_lock(original: &InvocationDirectory, mode: FlockOperation) -> Result<File, HostError> {
    // Artifact readers borrow report custody, not the parent invocation lock
    // used by executable release while consuming the original report.
    let path = original.path().join("workflow");
    let file = private_root(&path)?;
    flock(&file, mode).map_err(io::Error::from)?;
    let named = fs::symlink_metadata(&path)?;
    let opened = file.metadata()?;
    if (opened.dev(), opened.ino()) != (named.dev(), named.ino()) {
        return Err(invalid("invocation root changed while fencing workflow evidence"));
    }
    Ok(file)
}

impl InvocationStore {
    /// Fence passive reads against release of original workflow members.
    pub(crate) fn borrow_workflow_evidence(&self, id: &CliInvocationId) -> Result<File, HostError> {
        workflow_lock(&self.directory(id)?, FlockOperation::LockShared)
    }

    /// Positive evidence of retirement; missing files alone never establish expiry.
    pub(crate) fn workflow_retirement_digest(
        &self,
        id: &CliInvocationId,
    ) -> Result<Option<String>, HostError> {
        let original = match self.directory(id) {
            Ok(original) => original,
            Err(HostError::Io(error)) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        let Some(retired) = read_retirement(&original)? else {
            return Ok(None);
        };
        if &retired.invocation != id {
            return Err(invalid("workflow retirement witness names another invocation"));
        }
        Ok(Some(retired.report_sha256))
    }
}

impl InvocationDirectory {
    /// Enroll a new producer before any test work, including discovery that may fail.
    pub(crate) fn admit_workflow_retention(&self) -> Result<(), HostError> {
        publish(self.path(), ADMISSION, self.id(), Some(256))
    }
}

pub(super) fn raw_stream_retired(
    original: &InvocationDirectory, stream: crate::extraction::OutputStream,
) -> Result<bool, HostError> {
    Ok(read_retirement(original)?
        .filter(|record| &record.invocation == original.id())
        .and_then(|record| record.raw)
        .is_some_and(|record| match stream {
        crate::extraction::OutputStream::Stdout => record.stdout,
        crate::extraction::OutputStream::Stderr => record.stderr,
    }))
}

fn read_retirement(original: &InvocationDirectory) -> Result<Option<Retirement>, HostError> {
    let file = match evidence_file(&original.path().join(RETIRED)) {
        Ok(file) => file,
        Err(HostError::Io(error)) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let mut bytes = Vec::new();
    file.take(1025).read_to_end(&mut bytes)?;
    if bytes.len() > 1024 {
        return Err(invalid("workflow retirement witness exceeds its bound"));
    }
    Ok(Some(serde_json::from_slice(&bytes).map_err(io::Error::other)?))
}

fn read_enrollment(
    store: &InvocationStore,
    original: &InvocationDirectory,
    report: &ReportReference,
) -> Result<Option<Enrollment>, HostError> {
    let file = match evidence_file(&original.path().join(ENROLLMENT)) {
        Ok(file) => file,
        Err(HostError::Io(error)) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let mut bytes = Vec::new();
    file.take(META_LIMIT + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > META_LIMIT {
        return Err(invalid("workflow retention enrollment exceeds its metadata bound"));
    }
    let record: Enrollment = serde_json::from_slice(&bytes).map_err(io::Error::other)?;
    if &record.invocation != original.id()
        || record.checkout != store.state.checkout_id.as_str()
        || record.report_sha256 != report.sha256
        || record.report_byte_length != report.byte_length
        || !record.members.last().is_some_and(|member| {
            member.path == "report.json"
                && member.content.sha256 == report.sha256
                && member.content.byte_length == report.byte_length
        })
    {
        return Err(invalid("workflow retention enrollment differs from the original report"));
    }
    Ok(Some(record))
}

fn open_member(root: &Path, path: &str) -> Result<(File, PathBuf, File), HostError> {
    if !evidence::relative_file(path) {
        return Err(invalid("invalid enrolled workflow member path"));
    }
    let mut parent = root.to_path_buf();
    let mut components = Path::new(path).components().peekable();
    while let Some(component) = components.next() {
        if components.peek().is_none() {
            let owner = private_root(&parent)?;
            let name = PathBuf::from(component.as_os_str());
            let member = open_child(&owner, &name)?;
            return Ok((owner, name, member));
        }
        parent.push(component.as_os_str());
        crate::test_inventory::checked_parent(&parent)?;
    }
    Err(invalid("enrolled member has no filename"))
}

fn open_child(parent: &File, name: &Path) -> Result<File, HostError> {
    let file = File::from(openat(
        parent,
        name,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    ).map_err(io::Error::from)?);
    let metadata = file.metadata()?;
    if !metadata.is_file()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o022 != 0
        || metadata.nlink() != 1
    {
        return Err(invalid("enrolled workflow member has no private, unaliased file custody"));
    }
    Ok(file)
}

fn observe(root: &Path, path: &str, content: &ContentIdentity) -> Result<Member, HostError> {
    let (_, _, file) = open_member(root, path)?;
    let identity = descriptor_identity(&file)?;
    if identity.bytes != content.byte_length {
        return Err(invalid("workflow member extent differs from original commitment"));
    }
    Ok(Member { path: path.into(), content: content.clone(), identity })
}

/// Only the current workflow classification calls enrollment, after original settlement.
pub(super) fn enroll(
    store: &InvocationStore,
    original: &InvocationDirectory,
    reference: &ReportReference,
) -> Result<(), HostError> {
    if read_retirement(original)?.is_some() {
        return Ok(());
    }
    if read_enrollment(store, original, reference)?.is_some() {
        return Ok(());
    }
    let admission = match evidence_file(&original.path().join(ADMISSION)) {
        Ok(file) => file,
        Err(HostError::Io(error)) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    let admitted: CliInvocationId = serde_json::from_reader(admission.take(256)).map_err(io::Error::other)?;
    if &admitted != original.id() {
        return Err(invalid("workflow retention admission differs from the original invocation"));
    }
    let report = captured_report_at_limit(
        store, original.id().as_str(), None, Some(WORKFLOW_READ_LIMIT),
    )?;
    let commitments = report.observation.evidence()
        .map_err(|error| HostError::Workflow(Box::new(error)))?;
    let mut members = Vec::new();
    for item in commitments {
        if item.root != EvidenceRoot::Workflow || item.role == EvidenceRole::Input {
            continue;
        }
        if let Some(content) = &item.content {
            if item.path == "report.json" {
                return Err(invalid("report cannot be enrolled as its own member"));
            }
            members.push(observe(&report.parent, &item.path, content)?);
        }
    }
    members.push(observe(
        &report.parent, "report.json",
        &ContentIdentity { sha256: reference.sha256.clone(), byte_length: reference.byte_length },
    )?);
    let record = Enrollment {
        invocation: original.id().clone(),
        checkout: store.state.checkout_id.as_str().to_owned(),
        report_sha256: reference.sha256.clone(),
        report_byte_length: reference.byte_length,
        members,
        releasing: false,
    };
    publish(original.path(), ENROLLMENT, &record, Some(META_LIMIT))
}

pub(super) fn pending_extent(
    store: &InvocationStore,
    original: &InvocationDirectory,
    report: &ReportReference,
) -> Result<(u64, Option<String>), HostError> {
    let Some(record) = read_enrollment(store, original, report)? else {
        return Ok((0, None));
    };
    let mut bytes = record.members.iter().try_fold(0_u64, |total, member| {
        total.checked_add(member.identity.bytes)
            .ok_or_else(|| invalid("enrolled workflow extent overflow"))
    })?;
    let mut digest = Sha256::new();
    digest.update(serde_json::to_vec(&record).map_err(io::Error::other)?);
    if let Some(raw) = streams(original)? {
        for (_, member) in raw.members() {
            bytes = bytes.checked_add(member.bytes)
                .ok_or_else(|| invalid("enrolled workflow extent overflow"))?;
        }
        digest.update(serde_json::to_vec(&raw).map_err(io::Error::other)?);
    }
    let digest = digest.finalize().iter().map(|byte| format!("{byte:02x}")).collect();
    Ok((bytes, Some(digest)))
}

/// Actual extents of the original members still available to an explicit reader.
#[derive(Debug, Default, Serialize, JsonSchema)]
pub(super) struct ObservedExtent {
    apparent_bytes: u64,
    allocated_bytes: u64,
    observed_members: usize,
    unavailable_members: usize,
}

impl ObservedExtent {
    fn add(&mut self, opened: Result<File, HostError>, expected: &DescriptorIdentity) {
        let allocated = opened.and_then(|file| {
            let metadata = file.metadata()?;
            if descriptor_identity(&file)? != *expected {
                return Err(invalid("workflow member no longer has its enrolled identity"));
            }
            Ok(metadata.blocks().saturating_mul(512))
        });
        match allocated {
            Ok(bytes) => {
                self.apparent_bytes = self.apparent_bytes.saturating_add(expected.bytes);
                self.allocated_bytes = self.allocated_bytes.saturating_add(bytes);
                self.observed_members += 1;
            }
            Err(_) => self.unavailable_members += 1,
        }
    }
}

/// Observe the enrolled file set without sweeping unrelated paths or refreshing expiry.
pub(super) fn observe_extent(
    store: &InvocationStore,
    original: &InvocationDirectory,
    report: &ReportReference,
) -> Result<Option<ObservedExtent>, HostError> {
    let Some(record) = read_enrollment(store, original, report)? else {
        return Ok(None);
    };
    let _fence = workflow_lock(original, FlockOperation::LockShared)?;
    let workflow = original.path().join("workflow");
    let mut extent = ObservedExtent::default();
    for member in &record.members {
        extent.add(
            open_member(&workflow, &member.path).map(|(_, _, file)| file),
            &member.identity,
        );
    }
    if let Some(raw) = streams(original)? {
        let parent = private_root(original.path())?;
        for (name, expected) in raw.members() {
            extent.add(open_child(&parent, Path::new(name)), expected);
        }
    }
    Ok(Some(extent))
}

fn save_release_intent(
    original: &InvocationDirectory, enrollment: &Enrollment,
) -> Result<(), HostError> {
    let path = original.path().join(ENROLLMENT);
    let mut staged = tempfile::NamedTempFile::new_in(original.path())?;
    serde_json::to_writer(&mut staged, enrollment).map_err(io::Error::other)?;
    if staged.as_file().metadata()?.len() > META_LIMIT {
        return Err(invalid("workflow release intent exceeds its metadata bound"));
    }
    staged.as_file().sync_all()?;
    staged.persist(&path).map_err(|error| error.error)?;
    synchronize_publication_parent(original.path(), &path)
}

fn verify(
    root: &Path,
    expected: &Member,
) -> Result<(File, PathBuf), HostError> {
    let (parent, name, mut file) = open_member(root, &expected.path)?;
    if descriptor_identity(&file)? != expected.identity {
        return Err(invalid("enrolled workflow member changed before release"));
    }
    let mut hash = Sha256::new();
    let limit = expected.identity.bytes.checked_add(1)
        .ok_or_else(|| invalid("enrolled workflow extent overflow"))?;
    let mut bytes = 0u64;
    let mut observed = (&mut file).take(limit);
    let mut buffer = [0u8; 32 * 1024];
    loop {
        let read = observed.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hash.update(&buffer[..read]);
        bytes += read as u64;
    }
    let digest: String = hash.finalize().iter().map(|byte| format!("{byte:02x}")).collect();
    if bytes != expected.content.byte_length || digest != expected.content.sha256
        || descriptor_identity(&file)? != expected.identity
    {
        return Err(invalid("enrolled workflow bytes differ from the original commitment"));
    }
    Ok((parent, name))
}

fn authenticate_members(root: &Path, enrollment: &Enrollment) -> Result<(), HostError> {
    let report = enrollment.members.last()
        .ok_or_else(|| invalid("enrollment lacks its original report"))?;
    let (_, _, file) = open_member(root, &report.path)?;
    if descriptor_identity(&file)? != report.identity {
        return Err(invalid("original report was replaced after enrollment"));
    }
    let mut opened = evidence::OpenedFile::bind(0, file, &report.content)?;
    let (observation, _) = opened.with_reader(|input| {
        serde_json::from_reader::<_, crate::test_workflow::execute::ReportObservation>(input)
            .map_err(|error| io::Error::other(error).into())
    })?;
    let mut expected = observation.evidence()
        .map_err(|error| HostError::Workflow(Box::new(error)))?
        .iter()
        .filter(|item| item.root == EvidenceRoot::Workflow && item.role != EvidenceRole::Input)
        .filter_map(|item| item.content.as_ref().map(|content| (item.path.as_str(), content)))
        .collect::<BTreeMap<_, _>>();
    for member in &enrollment.members[..enrollment.members.len() - 1] {
        if expected.remove(member.path.as_str()) != Some(&member.content) {
            return Err(invalid("enrolled member is not in the original report"));
        }
    }
    if !expected.is_empty() {
        return Err(invalid("workflow enrollment omits committed members"));
    }
    Ok(())
}

/// Caller owns the settled compact record and excludes holds. A busy reader is not a release.
pub(super) fn release(
    store: &InvocationStore,
    original: &InvocationDirectory,
    report: &ReportReference,
) -> Result<u64, HostError> {
    let Some(mut record) = read_enrollment(store, original, report)? else {
        return Ok(0);
    };
    let _fence = workflow_lock(original, FlockOperation::NonBlockingLockExclusive)?;
    let workflow = original.path().join("workflow");
    let owner = private_root(original.path())?;
    let raw_receipt = stream_receipt(original)?;
    let raw = raw_receipt.as_ref()
        .map(|(streams, _)| streams.prepare_release(&owner, record.releasing))
        .transpose()?
        .unwrap_or_default();
    let retired_raw = raw_receipt.map(|(streams, receipt)| RetiredRaw {
        receipt,
        stdout: streams.stdout.is_some(),
        stderr: streams.stderr.is_some(),
    });
    let original_report_present = match authenticate_members(&workflow, &record) {
        Ok(()) => true,
        Err(HostError::Io(error)) if error.kind() == io::ErrorKind::NotFound && record.releasing => false,
        Err(error) => return Err(error),
    };
    let mut present = Vec::new();
    for member in &record.members {
        match verify(&workflow, member) {
            Ok((parent, name)) => present.push((parent, name, member)),
            Err(HostError::Io(error)) if error.kind() == io::ErrorKind::NotFound && record.releasing => {}
            Err(error) => return Err(error),
        }
    }
    if !original_report_present && (!present.is_empty() || !raw.is_empty()) {
        return Err(invalid("cannot finish release of remaining members without the original report"));
    }
    if !record.releasing {
        record.releasing = true;
        save_release_intent(original, &record)?;
    }
    let mut released: u64 = 0;
    for member in raw {
        released = released.checked_add(member.unlink(&owner)?)
            .ok_or_else(|| invalid("released workflow extent overflow"))?;
    }
    for (parent, name, member) in present {
        if descriptor_identity(&open_child(&parent, &name)?)? != member.identity {
            return Err(invalid("enrolled workflow member changed during release"));
        }
        unlinkat(&parent, &name, AtFlags::empty()).map_err(io::Error::from)?;
        parent.sync_all()?;
        released = released.checked_add(member.identity.bytes)
            .ok_or_else(|| invalid("released workflow extent overflow"))?;
    }
    if let Some(retired) = read_retirement(original)? {
        if &retired.invocation != original.id() || retired.report_sha256 != report.sha256
            || retired.raw != retired_raw
        {
            return Err(invalid("workflow retirement witness differs from the released original"));
        }
    } else {
        publish(
            original.path(),
            RETIRED,
            &Retirement {
                invocation: original.id().clone(),
                report_sha256: report.sha256.clone(),
                raw: retired_raw,
            },
            Some(1024),
        )?;
    }
    let path = original.path().join(ENROLLMENT);
    fs::remove_file(&path)?;
    synchronize_publication_parent(original.path(), &path)?;
    Ok(released)
}

pub(super) fn forget(original: &InvocationDirectory) -> Result<(), HostError> {
    let path = original.path().join(RETIRED);
    if let Some(retired) = read_retirement(original)? {
        if &retired.invocation != original.id() {
            return Err(invalid("workflow retirement witness names another invocation"));
        }
        if let Some(raw) = retired.raw {
            let parent = private_root(original.path())?;
            let name = Path::new(InvocationEvidence::WorkflowStreams.filename());
            match open_child(&parent, name) {
                Ok(file) if descriptor_identity(&file)? == raw.receipt => {
                    unlinkat(&parent, name, AtFlags::empty()).map_err(io::Error::from)?;
                    parent.sync_all()?;
                }
                Ok(_) => return Err(invalid("retired raw stream receipt was replaced before expiry")),
                Err(HostError::Io(error)) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
        fs::remove_file(&path)?;
        synchronize_publication_parent(original.path(), &path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        environment::Environment,
        paths::CheckoutState,
        test_workflow::execute::{Action, EvidenceFile, Outcome, ReportObservation},
    };
    use std::os::unix::fs::DirBuilderExt as _;

    fn write_member(path: &Path, bytes: &[u8]) {
        use std::io::Write as _;
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
            .unwrap()
            .write_all(bytes)
            .unwrap();
    }

    fn setup() -> (
        tempfile::TempDir,
        InvocationStore,
        InvocationDirectory,
        ReportReference,
        PathBuf,
        PathBuf,
    ) {
        use std::os::unix::fs::PermissionsExt as _;
        let temporary = tempfile::Builder::new()
            .permissions(fs::Permissions::from_mode(0o700))
            .tempdir()
            .unwrap();
        let environment = Environment::from([(
            "XDG_STATE_HOME".into(), temporary.path().join("state").into_os_string(),
        )]);
        let state = CheckoutState::from_paths(
            &temporary.path().join("checkout"),
            &temporary.path().join("common.git"),
            &environment,
        ).unwrap();
        let store = InvocationStore::new(state);
        let original = InvocationDirectory {
            path: temporary.path().to_owned(),
            invocation_id: CliInvocationId::new(),
            command: "test".into(),
        };
        let root = original.path().join("workflow");
        fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        let member_path = root.join("phase.stdout");
        let unrelated = root.join("unrelated-owner");
        write_member(&member_path, b"the original");
        write_member(&unrelated, b"foreign");
        let member_content = ContentIdentity::of(b"the original");
        let report = ReportObservation {
            version: 3,
            invocation: original.id().clone(),
            action: Action::Test,
            outcome: Outcome::TestsSucceeded,
            inventory: None,
            selected_tests: None,
            required_capture_complete: true,
            failures: Vec::new(),
            inventory_content: None,
            evidence: Some(vec![EvidenceFile {
                root: EvidenceRoot::Workflow,
                path: "phase.stdout".into(),
                role: EvidenceRole::Stdout,
                group: None,
                content: Some(member_content.clone()),
                omission: None,
                failure: None,
            }]),
        };
        let report = serde_json::to_vec(&report).unwrap();
        write_member(&root.join("report.json"), &report);
        let content = ContentIdentity::of(&report);
        let reference = ReportReference {
            path: "report.json".into(),
            sha256: content.sha256.clone(),
            byte_length: content.byte_length,
        };
        let enrollment = Enrollment {
            invocation: original.id().clone(),
            checkout: store.state.checkout_id.as_str().into(),
            report_sha256: content.sha256.clone(),
            report_byte_length: content.byte_length,
            members: vec![
                observe(&root, "phase.stdout", &member_content).unwrap(),
                observe(&root, "report.json", &content).unwrap(),
            ],
            releasing: false,
        };
        publish(original.path(), ENROLLMENT, &enrollment, Some(META_LIMIT)).unwrap();
        (temporary, store, original, reference, member_path, unrelated)
    }

    #[test]
    fn enrolled_release_fences_live_readers_and_unlinks_only_original_members() {
        use crate::extraction::OutputStream;
        let (_temporary, store, original, report, member, unrelated) = setup();
        let outer = original.path().join("stdout.raw");
        write_member(&outer, b"outer");
        original.publish_evidence(InvocationEvidence::WorkflowStreams, &WorkflowStreams {
            invocation: original.id().clone(),
            stdout: Some(descriptor_identity(&evidence_file(&outer).unwrap()).unwrap()),
            stderr: None,
        }).unwrap();
        let reader = workflow_lock(&original, FlockOperation::LockShared).unwrap();
        assert!(matches!(
            release(&store, &original, &report),
            Err(HostError::Io(error)) if error.kind() == io::ErrorKind::WouldBlock
        ));
        drop(reader);
        assert_eq!(
            release(&store, &original, &report).unwrap(),
            report.byte_length + b"the originalouter".len() as u64,
        );
        assert!(!member.exists());
        assert!(!original.path().join("workflow/report.json").exists());
        assert!(!outer.exists());
        assert!(original.stream(OutputStream::Stdout).unwrap_err().to_string().contains("retired"));
        assert_eq!(fs::read(unrelated).unwrap(), b"foreign");
        assert!(read_retirement(&original).unwrap().is_some());
        assert_eq!(release(&store, &original, &report).unwrap(), 0);
        forget(&original).unwrap();
        assert!(read_retirement(&original).unwrap().is_none());
        assert!(!original.path().join(InvocationEvidence::WorkflowStreams.filename()).exists());
    }

    #[test]
    fn enrolled_release_refuses_replacement_even_with_original_bytes() {
        let (_temporary, store, original, report, member, unrelated) = setup();
        let replacement = member.with_extension("replacement");
        write_member(&replacement, b"the original");
        fs::rename(replacement, &member).unwrap();
        let observed = observe_extent(&store, &original, &report).unwrap().unwrap();
        assert_eq!(observed.apparent_bytes, report.byte_length);
        assert_eq!(observed.observed_members, 1);
        assert_eq!(observed.unavailable_members, 1);
        assert!(release(&store, &original, &report).is_err());
        assert_eq!(fs::read(member).unwrap(), b"the original");
        assert!(original.path().join("workflow/report.json").exists());
        assert!(unrelated.exists());
        assert!(read_retirement(&original).unwrap().is_none());
    }

    #[test]
    fn enrolled_outer_stream_uses_the_writer_identity_and_fences_readers() {
        use crate::extraction::OutputStream;
        let (_temporary, store, original, report, member, _) = setup();
        let path = original.path().join("stdout.raw");
        let bytes = b"exact outer capture";
        write_member(&path, bytes);
        let captured = evidence_file(&path).unwrap();
        let original_identity = descriptor_identity(&captured).unwrap();
        original.publish_evidence(InvocationEvidence::WorkflowStreams, &WorkflowStreams {
            invocation: original.id().clone(),
            stdout: Some(original_identity),
            stderr: None,
        }).unwrap();
        let reader = original.stream(OutputStream::Stdout).unwrap();
        assert!(matches!(
            release(&store, &original, &report),
            Err(HostError::Io(error)) if error.kind() == io::ErrorKind::WouldBlock
        ));
        assert!(member.exists());
        drop(reader);
        let replacement = path.with_extension("replacement");
        write_member(&replacement, bytes);
        let parked_original = path.with_extension("parked");
        fs::rename(&path, &parked_original).unwrap();
        fs::rename(&replacement, &path).unwrap();
        assert!(original.stream(OutputStream::Stdout).is_err());
        assert!(release(&store, &original, &report).is_err());
        assert!(member.exists());
        assert!(path.exists());
    }

    #[test]
    fn forgetting_workflow_refuses_replaced_raw_stream_receipt() {
        let (_temporary, store, original, report, _, _) = setup();
        let outer = original.path().join("stdout.raw");
        write_member(&outer, b"outer");
        original.publish_evidence(InvocationEvidence::WorkflowStreams, &WorkflowStreams {
            invocation: original.id().clone(),
            stdout: Some(descriptor_identity(&evidence_file(&outer).unwrap()).unwrap()),
            stderr: None,
        }).unwrap();
        release(&store, &original, &report).unwrap();
        let path = original.path().join(InvocationEvidence::WorkflowStreams.filename());
        let replacement = path.with_extension("replacement");
        write_member(&replacement, &fs::read(&path).unwrap());
        fs::rename(&replacement, &path).unwrap();
        assert!(forget(&original).is_err());
        assert!(path.exists());
        assert!(read_retirement(&original).unwrap().is_some());
    }
}
