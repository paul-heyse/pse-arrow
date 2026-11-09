// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit external-file profile over one original interpretation-three commitment.
use crate::{test_inventory::CapturedReport, test_workflow::execute::EvidenceScope};
use pse_contract::boundary_projection::{
    Direction, Field, ProjectionError, ProjectionGraph, ProjectionObservationRecord,
    ProjectionType, Shape, TypeRef, ValidateProjection, ValidationError,
};
use std::sync::Arc;

pse_contract::projection_enum! {
/// Versioned physical intent; no file membership or completeness grant is encoded here.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
pub enum DirectoryProfile {
    projection: { name: "WorkflowEvidenceDirectoryProfile", rust_path: crate::developer_evidence::directory::DirectoryProfile, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// All originally committed members occupy native ordinal-derived file names.
    OriginalFullV1 => 1;
}
}

/// Bounded passive metadata. All member facts remain inside the exact original
/// report; native policy fixes manifest.pb, descriptor.bin and members/<ordinal>.
pub(crate) struct DirectoryManifest {
    pub(crate) report: Arc<CapturedReport>,
    pub(crate) profile: DirectoryProfile,
}

impl ProjectionType for DirectoryManifest {
    fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
        let root = graph.declare::<Self>(
            "CapturedWorkflowDirectoryManifest",
            file!(),
            1,
            Direction::Output,
            &[],
            &[],
            |graph| {
                Ok(Shape::Record(vec![
                    Field::of::<CapturedReport>(graph, "report", 1, Direction::Output, vec![])?,
                    Field::of::<DirectoryProfile>(graph, "profile", 2, Direction::Output, vec![])?,
                ]))
            },
        )?;
        graph.public_path::<Self>(
            &root,
            "crate::developer_evidence::directory::DirectoryManifest",
        )?;
        graph.observation_record::<Self>(&root)?;
        Ok(root)
    }
}

impl ProjectionObservationRecord for DirectoryManifest {
    const VERSION: u32 = 1;
    const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[(1, "report"), (2, "profile")];
    type Fields = (Arc<CapturedReport>, DirectoryProfile);
    fn observation_fields(
        &self,
    ) -> Result<Self::Fields, pse_contract::boundary_projection::AdapterError> {
        Ok((Arc::clone(&self.report), self.profile))
    }
}

impl ValidateProjection for DirectoryManifest {
    fn validate_at(&self, path: &str) -> Result<(), ValidationError> {
        self.report.validate_at(path)?;
        if !original_profile(self.report.scope, self.report.files.is_empty()) {
            return Err(ValidationError::Rule {
                path: path.into(),
                rule: "directory.original_profile",
                reason: "directory metadata requires Report scope and no inline files".into(),
            });
        }
        Ok(())
    }
}

use super::{
    EvidenceError, MAX_INPUT_BYTES, Readback, VerifiedReport, admitted_options,
    adopt_member_meaning, adopt_report, finish_members, read_member_meaning,
    validate_member_meaning,
};
use crate::developer_evidence_contracts as contracts;
use crate::{HostError, invocation::InvocationStore, test_workflow::execute::evidence::OpenedFile};
use buffa::Message as _;
use contracts::pb::pse::developer::evidence::v1 as pb;
use protovalidate_buffa::Validate as _;
use pse_contract_proto::support::{decode_root, required, unknown_fields};
use std::{
    collections::BTreeSet,
    fs::File,
    io::{Read, Write},
    num::NonZeroU64,
    os::unix::fs::{MetadataExt as _, PermissionsExt as _},
    path::Path,
};

const MANIFEST: &str = "manifest.pb";
const DESCRIPTORS: &str = "descriptor.bin";
const MEMBERS: &str = "members";

/// Complete original-member verification and bytes accepted by Write.
/// This receipt establishes neither sink flush/sync nor destination publication.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct MemberCopy {
    /// Exact original dependency ordinal.
    pub member: usize,
    /// Complete original bytes transferred and verified, never a prefix commitment.
    pub content: crate::test_workflow::execute::ContentIdentity,
}

/// Exclusive publication of one verified member, retaining original acquisition meaning.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct MemberExport {
    /// Explicit destination published by the native no-replace/sync owner.
    pub output: crate::investigation::NativePath,
    /// Original complete member, separately verified during this transfer.
    pub copied: MemberCopy,
    /// Original report outcome and acquisition coverage; copying cannot upgrade them.
    pub readback: Readback,
}

#[derive(Debug, thiserror::Error)]
#[error("original member transfer cancelled at its native checkpoint")]
struct TransferCancelled;

struct TransferSink<'a, W> {
    output: &'a mut W,
    cancel: &'a tokio_util::sync::CancellationToken,
    accepted: u64,
    cancelled: bool,
}
impl<W: Write> TransferSink<'_, W> {
    fn checkpoint(&mut self) -> std::io::Result<()> {
        if self.cancel.is_cancelled() {
            self.cancelled = true;
            return Err(std::io::Error::other(TransferCancelled));
        }
        Ok(())
    }
}
impl<W: Write> Write for TransferSink<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.checkpoint()?;
        let count = self.output.write(bytes)?;
        self.accepted = self
            .accepted
            .checked_add(
                u64::try_from(count)
                    .map_err(|_| std::io::Error::other("accepted member extent exceeds u64"))?,
            )
            .ok_or_else(|| std::io::Error::other("accepted member extent exceeds u64"))?;
        Ok(count)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.checkpoint()?;
        self.output.flush()
    }
}

/// Copy one original member into an exclusively staged and synchronized destination.
/// No destination prefix is published on transfer failure. A parent-sync failure retains
/// the native publication-durability-unknown outcome and must not be blindly replayed.
/// # Errors
/// Original acquisition, transfer/cancellation, publication or synchronization fails.
pub fn export_directory_member(
    input: &Path,
    ordinal: usize,
    output: &Path,
    cancel: &tokio_util::sync::CancellationToken,
) -> Result<MemberExport, HostError> {
    let mut reader = open_directory(input, &super::binary_options())?;
    let mut copied = None;
    crate::invocation::publish_file(output, 0o400, |sink| {
        copied = Some(reader.copy_member_verified(ordinal, sink, cancel)?);
        Ok(())
    })?;
    Ok(MemberExport {
        output: std::path::absolute(output)?.into(),
        copied: copied.ok_or_else(|| invalid("member publication lost its verified transfer"))?,
        readback: reader.readback,
    })
}

/// Exclusive directory publication and the original observations actually verified.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct DirectoryExport {
    /// User-selected exclusively published directory.
    pub output: crate::investigation::NativePath,
    /// Actual generated passive metadata root.
    pub message: String,
    /// Exact bounded manifest digest.
    pub manifest_sha256: String,
    /// Exact manifest extent, separate from external members.
    pub manifest_bytes: usize,
    /// External member extent, accumulated with checked u64 arithmetic.
    pub member_bytes: u64,
    /// Versioned fixed native directory profile.
    pub profile: DirectoryProfile,
    /// The same report/role/completeness interpretation as compact evidence.
    pub readback: Readback,
}

fn invalid(message: impl Into<String>) -> HostError {
    EvidenceError::Invalid(message.into()).into()
}

fn original_profile(scope: EvidenceScope, inline_empty: bool) -> bool {
    scope == EvidenceScope::Report && inline_empty
}

fn require_profile(verified: &VerifiedReport) -> Result<(), HostError> {
    if !original_profile(verified.readback.scope, verified.files.is_empty()) {
        return Err(invalid(
            "directory metadata requires Report scope and no inline files",
        ));
    }
    Ok(())
}

pub(crate) fn acquire_original(original: &CapturedReport) -> Result<Vec<OpenedFile>, HostError> {
    if !original_profile(original.scope, original.files.is_empty()) {
        return Err(invalid(
            "opened acquisition requires Report scope and no inline files",
        ));
    }
    crate::test_workflow::execute::evidence::acquire(
        &original.parent,
        original
            .observation
            .evidence()
            .map_err(EvidenceError::from)?,
        |member, commitment| {
            let (meaning, identity) =
                member.with_reader(|input| read_member_meaning(commitment.role, input))?;
            validate_member_meaning(&original.observation, commitment.role, &identity, meaning)?;
            Ok(())
        },
    )
}

struct Tee<'a> {
    input: &'a mut dyn Read,
    output: &'a mut File,
}
impl Read for Tee<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let count = self.input.read(buffer)?;
        self.output.write_all(&buffer[..count])?;
        Ok(count)
    }
}

/// Publish exact original files through a private staged directory. Compact export
/// never falls back to this explicitly selected external profile.
pub fn export_directory(
    store: &InvocationStore,
    selector: &str,
    session: Option<&str>,
    output: &Path,
) -> Result<DirectoryExport, HostError> {
    let crate::test_inventory::OpenedReport {
        original,
        mut files,
    } = crate::test_inventory::opened_report(store, selector, session)?;
    let manifest = DirectoryManifest {
        report: Arc::new(original),
        profile: DirectoryProfile::OriginalFullV1,
    };
    let wire = contracts::encode_captured_workflow_directory_manifest(&manifest)
        .map_err(EvidenceError::from)?;
    let bytes = wire.try_encode_to_vec().map_err(EvidenceError::from)?;
    let mut verified =
        adopt_report(required(wire.report.into_option(), "report").map_err(EvidenceError::from)?)?;
    require_profile(&verified)?;
    bounded(&bytes)?;
    bounded(contracts::DESCRIPTORS)?;
    let member_bytes = crate::invocation::publish_directory(output, |stage| {
        let members = stage.join(MEMBERS);
        std::fs::create_dir(&members)?;
        std::fs::set_permissions(&members, std::fs::Permissions::from_mode(0o700))?;
        let mut identities = Vec::new();
        let mut total = 0u64;
        for member in &mut files {
            let ordinal = member.dependency();
            let role = verified
                .observation
                .evidence()
                .map_err(EvidenceError::from)?[ordinal]
                .role;
            let mut adopted = None;
            crate::invocation::publish_file(&members.join(ordinal.to_string()), 0o400, |output| {
                adopted =
                    Some(member.with_reader(|input| {
                        read_member_meaning(role, &mut Tee { input, output })
                    })?);
                Ok(())
            })?;
            let (meaning, identity) =
                adopted.ok_or_else(|| invalid("member publication lost its verified read"))?;
            adopt_member_meaning(&mut verified, role, &identity, meaning)?;
            total = total
                .checked_add(identity.byte_length)
                .ok_or_else(|| invalid("external dependency total exceeds u64"))?;
            identities.push((ordinal, identity));
        }
        finish_members(&mut verified, &identities)?;
        crate::invocation::publish_bytes(
            &stage.join(DESCRIPTORS),
            contracts::DESCRIPTORS,
            Some(MAX_INPUT_BYTES),
            0o400,
        )?;
        // The metadata name appears only after all members and descriptors are synced.
        crate::invocation::publish_bytes(
            &stage.join(MANIFEST),
            &bytes,
            Some(MAX_INPUT_BYTES),
            0o400,
        )?;
        Ok(total)
    })?;
    Ok(DirectoryExport {
        output: std::path::absolute(output)?.into(),
        message: <pb::CapturedWorkflowDirectoryManifest as buffa::MessageName>::FULL_NAME.into(),
        manifest_sha256: super::digest(&bytes),
        manifest_bytes: bytes.len(),
        member_bytes,
        profile: DirectoryProfile::OriginalFullV1,
        readback: verified.readback,
    })
}

fn bounded(bytes: &[u8]) -> Result<(), HostError> {
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(EvidenceError::InputLimit {
            observed_at_least: bytes.len(),
            maximum: MAX_INPUT_BYTES,
        }
        .into());
    }
    Ok(())
}

/// Native retained descriptor owner. Decoded metadata alone never creates this
/// value, and this owner cannot be serialized into a custody/completeness grant.
pub struct DirectoryReader {
    readback: Readback,
    files: Vec<OpenedFile>,
    member_bytes: u64,
}

/// Verified whole acquisition plus an optional explicitly bounded member range.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct DirectoryReadback {
    /// Original complete acquisition observation, independent of a returned range.
    pub readback: Readback,
    /// External extent verified at acquisition.
    pub member_bytes: u64,
    /// Selected original ordinal; absent for metadata-only output.
    pub member: Option<usize>,
    /// Exact requested bounded original bytes; this range is not the full acquisition.
    pub range: Option<crate::artifacts::DescriptorRead>,
}

/// Perform an offline acquisition and optionally read one bounded original range.
pub fn inspect_directory(
    input: &Path,
    member: Option<usize>,
    offset: u64,
    length: NonZeroU64,
) -> Result<DirectoryReadback, HostError> {
    let mut reader = open_directory(input, &super::binary_options())?;
    let range = member
        .map(|ordinal| reader.read_member(ordinal, offset, length))
        .transpose()?;
    Ok(DirectoryReadback {
        readback: reader.readback,
        member_bytes: reader.member_bytes,
        member,
        range,
    })
}
impl std::fmt::Debug for DirectoryReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DirectoryReader")
            .field("readback", &self.readback)
            .field("member_bytes", &self.member_bytes)
            .finish_non_exhaustive()
    }
}
impl DirectoryReader {
    /// Transfer an original member sequentially with bounded library copy buffers.
    /// Emitted bytes remain provisional until EOF, fresh identity and whole digest agree.
    /// Cancellation is observed between writes and at completion; blocked regular-file
    /// I/O settles before return. The caller owns sink flush/sync and any publication.
    /// # Errors
    /// Missing original member, custody/digest change, sink failure or cancellation.
    pub fn copy_member_verified(
        &mut self,
        ordinal: usize,
        output: &mut impl Write,
        cancel: &tokio_util::sync::CancellationToken,
    ) -> Result<MemberCopy, HostError> {
        let index = self
            .files
            .binary_search_by_key(&ordinal, OpenedFile::dependency)
            .map_err(|_| invalid("selected ordinal has no originally committed member"))?;
        let mut sink = TransferSink {
            output,
            cancel,
            accepted: 0,
            cancelled: false,
        };
        let operation = (|| {
            sink.checkpoint()?;
            let content = self.files[index].copy_verified(&mut sink)?;
            sink.checkpoint()?;
            Ok::<_, HostError>(content)
        })();
        match operation {
            Ok(content) => Ok(MemberCopy {
                member: ordinal,
                content,
            }),
            Err(source) => Err(EvidenceError::Transfer {
                member: ordinal,
                accepted_bytes: sink.accepted,
                cancelled: sink.cancelled,
                source: Box::new(source),
            }
            .into()),
        }
    }
    /// Complete acquisition observation, preserving original omissions and outcome facts.
    pub fn readback(&self) -> &Readback {
        &self.readback
    }
    /// External bytes actually verified at acquisition, independently of metadata bounds.
    pub fn member_bytes(&self) -> u64 {
        self.member_bytes
    }
    /// Bounded original bytes through the retained descriptor, with fresh whole-file
    /// digest/identity checks. Path replacement/unlink cannot redirect the fd; same-inode
    /// changes refuse. This is neither snapshot isolation nor directory retention.
    pub fn read_member(
        &mut self,
        ordinal: usize,
        offset: u64,
        length: NonZeroU64,
    ) -> Result<crate::artifacts::DescriptorRead, HostError> {
        let index = self
            .files
            .binary_search_by_key(&ordinal, OpenedFile::dependency)
            .map_err(|_| invalid("selected ordinal has no originally committed member"))?;
        self.files[index].read_range(offset, length)
    }
}

fn open_at(parent: &File, name: &str, directory: bool) -> Result<File, HostError> {
    let mut flags = rustix::fs::OFlags::RDONLY
        | rustix::fs::OFlags::CLOEXEC
        | rustix::fs::OFlags::NOFOLLOW
        | rustix::fs::OFlags::NONBLOCK;
    if directory {
        flags |= rustix::fs::OFlags::DIRECTORY;
    }
    let file: File = rustix::fs::openat(parent, name, flags, rustix::fs::Mode::empty())
        .map_err(std::io::Error::from)?
        .into();
    let metadata = file.metadata()?;
    if metadata.uid() != rustix::process::geteuid().as_raw()
        || (directory && (!metadata.is_dir() || metadata.mode() & 0o777 != 0o700))
        || (!directory && (!metadata.is_file() || metadata.mode() & 0o022 != 0))
    {
        return Err(invalid(
            "directory bundle member lacks private native custody",
        ));
    }
    Ok(file)
}

fn metadata_bytes(parent: &File, name: &str) -> Result<Vec<u8>, HostError> {
    let file = open_at(parent, name, false)?;
    let before = crate::artifacts::descriptor_identity(&file)?;
    let mut bytes = Vec::new();
    (&file)
        .take(MAX_INPUT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    bounded(&bytes)?;
    if before != crate::artifacts::descriptor_identity(&file)? || bytes.len() as u64 != before.bytes
    {
        return Err(invalid("directory metadata changed while reading"));
    }
    Ok(bytes)
}

fn exact_names(parent: &File, expected: BTreeSet<String>) -> Result<(), HostError> {
    let mut observed = BTreeSet::new();
    for entry in rustix::fs::Dir::read_from(parent).map_err(std::io::Error::from)? {
        let entry = entry.map_err(std::io::Error::from)?;
        let name = entry.file_name().to_bytes();
        if name == b"." || name == b".." {
            continue;
        }
        let name = std::str::from_utf8(name)
            .map_err(|_| invalid("unexpected non-UTF8 bundle member name"))?;
        if !expected.contains(name) || !observed.insert(name.into()) {
            return Err(invalid("unexpected or duplicate directory bundle member"));
        }
    }
    if observed != expected {
        return Err(invalid(
            "directory bundle is missing an originally committed member",
        ));
    }
    Ok(())
}

/// Independently acquire and verify one explicit offline directory. There is no
/// reconstruction of native execution inputs or source-file extraction paths.
pub fn open_directory(
    input: &Path,
    requested: &buffa::DecodeOptions,
) -> Result<DirectoryReader, HostError> {
    let root: File = rustix::fs::open(
        input,
        rustix::fs::OFlags::RDONLY
            | rustix::fs::OFlags::DIRECTORY
            | rustix::fs::OFlags::NOFOLLOW
            | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )
    .map_err(std::io::Error::from)?
    .into();
    let metadata = root.metadata()?;
    if metadata.uid() != rustix::process::geteuid().as_raw() || metadata.mode() & 0o777 != 0o700 {
        return Err(invalid("directory bundle root is not private custody"));
    }
    exact_names(
        &root,
        [MANIFEST.into(), DESCRIPTORS.into(), MEMBERS.into()].into(),
    )?;
    let descriptors = metadata_bytes(&root, DESCRIPTORS)?;
    let descriptors: buffa_descriptor::generated::descriptor::FileDescriptorSet =
        super::binary_options()
            .decode_from_slice(&descriptors)
            .map_err(EvidenceError::from)?;
    let bytes = metadata_bytes(&root, MANIFEST)?;
    let options = admitted_options(&descriptors, bytes.len(), requested)?;
    let wire: pb::CapturedWorkflowDirectoryManifest = options
        .decode_from_slice(&bytes)
        .map_err(EvidenceError::from)?;
    wire.validate().map_err(EvidenceError::from)?;
    unknown_fields(
        &wire.__buffa_unknown_fields,
        "CapturedWorkflowDirectoryManifest",
    )
    .map_err(EvidenceError::from)?;
    let profile: DirectoryProfile = decode_root(
        required(wire.profile.into_option(), "profile").map_err(EvidenceError::from)?,
        "profile",
    )
    .map_err(EvidenceError::from)?;
    match profile {
        DirectoryProfile::OriginalFullV1 => {}
    }
    let mut verified =
        adopt_report(required(wire.report.into_option(), "report").map_err(EvidenceError::from)?)?;
    require_profile(&verified)?;
    let members = open_at(&root, MEMBERS, true)?;
    let commitments = verified
        .observation
        .evidence()
        .map_err(EvidenceError::from)?;
    exact_names(
        &members,
        commitments
            .iter()
            .enumerate()
            .filter(|(_, file)| file.content.is_some())
            .map(|(ordinal, _)| ordinal.to_string())
            .collect(),
    )?;
    let mut files = Vec::new();
    let mut identities = Vec::new();
    let mut meanings = Vec::new();
    let mut member_bytes = 0u64;
    for (ordinal, commitment) in commitments.iter().enumerate() {
        let Some(expected) = &commitment.content else {
            continue;
        };
        let file = open_at(&members, &ordinal.to_string(), false)?;
        let mut member = OpenedFile::bind(ordinal, file, expected)?;
        let (meaning, identity) =
            member.with_reader(|input| read_member_meaning(commitment.role, input))?;
        member_bytes = member_bytes
            .checked_add(identity.byte_length)
            .ok_or_else(|| invalid("external dependency total exceeds u64"))?;
        identities.push((ordinal, identity));
        meanings.push((commitment.role, meaning));
        files.push(member);
    }
    finish_members(&mut verified, &identities)?;
    for ((role, meaning), (_, identity)) in meanings.into_iter().zip(&identities) {
        adopt_member_meaning(&mut verified, role, identity, meaning)?;
    }
    verified.readback.limitations.push("Retained descriptors protect against pathname replacement and unlink, not same-inode mutation, directory deletion, or atomic filesystem snapshots; every later range is verified again.".into());
    Ok(DirectoryReader {
        readback: verified.readback,
        files,
        member_bytes,
    })
}
