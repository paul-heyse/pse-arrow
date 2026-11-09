// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original performing-helper dependency commitments, distinct from retention now.
use super::ContentIdentity;
use crate::test_workflow::WorkflowError;
use crate::{
    HostError,
    envelope::{ErrorRecord, RedactionPolicy},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use std::{
    collections::BTreeSet,
    fs::{File, OpenOptions},
    io::{Read as _, Seek as _},
    os::unix::fs::{MetadataExt as _, OpenOptionsExt as _, PermissionsExt as _},
    path::{Component, Path, PathBuf},
};
mod opened;
pub(crate) use opened::{OpenedFile, acquire};

pse_contract::projection_enum! {
/// Exact requested developer observation scope; none grants execution authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceScope {
    projection: { name: "WorkflowEvidenceScope", rust_path: crate::test_workflow::execute::EvidenceScope, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Original report and manager-recorded observations only.
    Report => 1;
    /// Original report and complete inventory association, without raw dependencies.
    Selection => 2;
    /// Original dependency account and all originally committed file bytes.
    Full => 3;
}
}
pse_contract::projection_enum! {
/// Fixed native custody roots for this workflow family, never arbitrary export paths.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRoot {
    projection: { name: "WorkflowEvidenceRoot", rust_path: crate::test_workflow::execute::EvidenceRoot, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Original performing helper's private workflow directory.
    Workflow => 1;
    /// Original invocation directory; only the fixed authenticated input is eligible.
    Invocation => 2;
}
}
pse_contract::projection_enum! {
/// Source-owned file responsibilities; control receipts and executable custody are excluded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRole {
    projection: { name: "WorkflowEvidenceRole", rust_path: crate::test_workflow::execute::EvidenceRole, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Exact private input accepted through the original metadata binding.
    Input => 1;
    /// Original native listing inventory, when produced.
    Inventory => 2;
    /// Raw stdout of an actually reached phase.
    Stdout => 3;
    /// Raw stderr of an actually reached phase.
    Stderr => 4;
    /// Original completed runner artifact, including assertion output.
    RunnerReport => 5;
    /// Original compiler-unit callback context.
    CompilerContext => 6;
    /// Exact captured or derived runner policy.
    RunnerConfiguration => 7;
    /// Original passive fixture outcome, without service capabilities.
    FixtureOutcome => 8;
    /// Original discovery metadata consumed by native listing/execution.
    DiscoveryContext => 9;
}
}
pse_contract::projection_enum! {
/// Original cutoff omissions; none describes later expiry, removal or corruption.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceOmission {
    projection: { name: "WorkflowEvidenceOmission", rust_path: crate::test_workflow::execute::EvidenceOmission, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// The original writer/capture did not establish stable required bytes.
    Unsettled => 1;
    /// The originally expected file was absent at the performing cutoff.
    Missing => 2;
    /// Original custody or input-identity checks refused.
    Refused => 3;
    /// Original required reading failed for another I/O reason.
    Unreadable => 4;
}
}
pse_contract::projection_struct! {
/// One immutable original promise or omission, committed inside report interpretation three.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceFile {
    projection: { name: "WorkflowEvidenceFile", rust_path: crate::test_workflow::execute::EvidenceFile, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Fixed original custody root.
    pub root: EvidenceRoot => { tag: 1, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Normalized relative original file address, never an extraction destination.
    pub path: String => { tag: 2, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Responsibility supplied by the actual native producer.
    pub role: EvidenceRole => { tag: 3, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original compiler group, when applicable.
    pub group: Option<usize> => { tag: 4, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Present only when the source owner established exact original bytes.
    pub content: Option<ContentIdentity> => { tag: 5, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Present only for an original omission; never filled from a later read.
    pub omission: Option<EvidenceOmission> => { tag: 6, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original typed custody/read refusal, independent of the workload's observed outcome.
    pub failure: Option<ErrorRecord> => { tag: 7, direction: pse_contract::BoundaryDirection::Output, rules: [] };
}
}
pse_contract::projection_struct! {
/// Passive exact file data, addressed by the original report's dependency ordinal.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CapturedFile {
    projection: { name: "WorkflowCapturedFile", rust_path: crate::test_workflow::execute::CapturedFile, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Zero-based original dependency entry; paths and commitments are not repeated.
    pub dependency: usize => { tag: 1, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Exact original bytes, with no newline, text or redaction transformation.
    pub original: Vec<u8> => { tag: 2, direction: pse_contract::BoundaryDirection::Output, rules: [] };
}
}

#[derive(Debug)]
pub(super) struct SourceFile {
    pub root: EvidenceRoot,
    pub path: String,
    pub role: EvidenceRole,
    pub group: Option<usize>,
    pub settled: bool,
}
fn invalid(message: impl Into<String>) -> HostError {
    HostError::Ownership(format!("original workflow evidence: {}", message.into()))
}
pub(crate) fn relative_file(path: &str) -> bool {
    !path.is_empty()
        && !path.as_bytes().contains(&0)
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
        && Path::new(path)
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}

fn address(workflow: &Path, root: EvidenceRoot, path: &str) -> Result<PathBuf, HostError> {
    if !relative_file(path) {
        return Err(invalid(
            "dependency address is not a normalized relative file",
        ));
    }
    let base = match root {
        EvidenceRoot::Workflow => workflow,
        EvidenceRoot::Invocation if path == "workflow-input.json" => workflow
            .parent()
            .ok_or_else(|| invalid("original invocation root is absent"))?,
        EvidenceRoot::Invocation => {
            return Err(invalid(
                "invocation dependency is not the fixed workflow input",
            ));
        }
    };
    crate::test_inventory::checked_parent(base)?;
    let mut current = base.to_owned();
    let mut components = Path::new(path).components().peekable();
    while let Some(component) = components.next() {
        current.push(component.as_os_str());
        if components.peek().is_some() {
            crate::test_inventory::checked_parent(&current)?;
        }
    }
    Ok(current)
}
fn identity(workflow: &Path, source: &SourceFile) -> Result<ContentIdentity, HostError> {
    let mut file =
        crate::test_inventory::open_private(&address(workflow, source.root, &source.path)?)?;
    hash_file(&mut file)
}
fn hash_file(file: &mut File) -> Result<ContentIdentity, HostError> {
    let mut hash = Sha256::new();
    let mut byte_length = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
        byte_length = byte_length
            .checked_add(count as u64)
            .ok_or_else(|| invalid("dependency extent overflow"))?;
    }
    Ok(ContentIdentity {
        sha256: hash
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
        byte_length,
    })
}
fn owned_runner_entry(path: &Path, file: &File) -> Result<(), HostError> {
    let entry = std::fs::symlink_metadata(path)?;
    let opened = file.metadata()?;
    let user = rustix::process::geteuid().as_raw();
    if !entry.is_file()
        || !opened.is_file()
        || entry.uid() != user
        || opened.uid() != user
        || entry.nlink() != 1
        || opened.nlink() != 1
        || entry.dev() != opened.dev()
        || entry.ino() != opened.ino()
    {
        return Err(invalid(
            "original runner artifact is not the same unaliased owned regular file",
        ));
    }
    Ok(())
}
fn finalize_runner_descriptor(path: &Path, mut file: File) -> Result<ContentIdentity, HostError> {
    owned_runner_entry(path, &file)?;
    file.set_permissions(std::fs::Permissions::from_mode(0o400))?;
    file.sync_all()?;
    let sealed = crate::artifacts::descriptor_identity(&file)?;
    file.rewind()?;
    let content = hash_file(&mut file)?;
    owned_runner_entry(path, &file)?;
    if crate::artifacts::descriptor_identity(&file)? != sealed
        || content.byte_length != sealed.bytes
    {
        return Err(invalid(
            "original runner artifact changed during producer finalization",
        ));
    }
    Ok(content)
}
fn finalize_runner_report(
    workflow: &Path,
    source: &SourceFile,
) -> Result<ContentIdentity, HostError> {
    if source.root != EvidenceRoot::Workflow || source.role != EvidenceRole::RunnerReport {
        return Err(invalid(
            "only the declared workflow runner artifact can be finalized here",
        ));
    }
    let path = address(workflow, source.root, &source.path)?;
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(&path)?;
    finalize_runner_descriptor(&path, file)
}
/// Seal from actual settled producer facts once; retain every original typed cause.
pub(super) fn seal(
    workflow: &Path,
    sources: Vec<SourceFile>,
) -> (Vec<EvidenceFile>, Vec<WorkflowError>) {
    let mut files = Vec::new();
    let mut causes = Vec::new();
    for source in sources {
        let mut file = EvidenceFile {
            root: source.root,
            path: source.path.clone(),
            role: source.role,
            group: source.group,
            content: None,
            omission: None,
            failure: None,
        };
        if !source.settled {
            file.omission = Some(EvidenceOmission::Unsettled);
        } else {
            match if source.role == EvidenceRole::RunnerReport {
                finalize_runner_report(workflow, &source)
            } else {
                identity(workflow, &source)
            } {
                Ok(content) => file.content = Some(content),
                Err(error) => {
                    file.omission = Some(match &error {
                        HostError::Io(error) if error.kind() == std::io::ErrorKind::NotFound => {
                            EvidenceOmission::Missing
                        }
                        HostError::Io(_) => EvidenceOmission::Unreadable,
                        _ => EvidenceOmission::Refused,
                    });
                    file.failure = Some(ErrorRecord::from_typed(
                        &error,
                        &RedactionPolicy::from_environment(),
                    ));
                    causes.push(error.into());
                }
            }
        }
        files.push(file);
    }
    (files, causes)
}
/// Validate passive declarations without creating custody, verdicts or extraction paths.
pub(crate) fn validate(files: &[EvidenceFile]) -> Result<(), HostError> {
    let mut addresses = BTreeSet::new();
    for file in files {
        if !relative_file(&file.path) || !addresses.insert((file.root, &file.path)) {
            return Err(invalid("dependency addresses are invalid or repeated"));
        }
        if file.root == EvidenceRoot::Invocation
            && (file.path != "workflow-input.json" || file.role != EvidenceRole::Input)
        {
            return Err(invalid(
                "only the authenticated input may address invocation custody",
            ));
        }
        if file.role == EvidenceRole::Input && file.root != EvidenceRoot::Invocation {
            return Err(invalid(
                "input commitment has the wrong original custody root",
            ));
        }
        if file.role == EvidenceRole::Inventory
            && (file.root != EvidenceRoot::Workflow || file.path != "inventory/inventory.json")
        {
            return Err(invalid(
                "inventory commitment has the wrong original address",
            ));
        }
        if (matches!(file.role, EvidenceRole::Input | EvidenceRole::Inventory)
            && file.group.is_some())
            || (matches!(
                file.role,
                EvidenceRole::RunnerReport
                    | EvidenceRole::CompilerContext
                    | EvidenceRole::FixtureOutcome
                    | EvidenceRole::DiscoveryContext
            ) && file.group.is_none())
        {
            return Err(invalid(
                "dependency role and original compiler-group association disagree",
            ));
        }
        if file.content.is_some() == file.omission.is_some()
            || (file.content.is_some() && file.failure.is_some())
        {
            return Err(invalid(
                "dependency must have exactly one original commitment or omission",
            ));
        }
        if file.content.as_ref().is_some_and(|content| {
            content.sha256.len() != 64
                || !content
                    .sha256
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        }) {
            return Err(invalid("dependency digest is not canonical SHA-256"));
        }
        if file
            .omission
            .is_some_and(|omission| omission != EvidenceOmission::Unsettled)
            && file.failure.is_none()
        {
            return Err(invalid(
                "original dependency refusal lacks its original typed diagnostic",
            ));
        }
    }
    Ok(())
}
/// Recheck original promises; original omissions stay omissions even if a file appears later.
pub(super) fn revalidate(workflow: &Path, files: &[EvidenceFile]) -> Result<(), HostError> {
    validate(files)?;
    for file in files {
        if let Some(expected) = &file.content {
            let actual = identity(
                workflow,
                &SourceFile {
                    root: file.root,
                    path: file.path.clone(),
                    role: file.role,
                    group: file.group,
                    settled: true,
                },
            )?;
            if &actual != expected {
                return Err(invalid(format!(
                    "dependency {} differs from its original commitment",
                    file.path
                )));
            }
        }
    }
    Ok(())
}
/// Acquire a bounded portable closure from the same original native custody reader.
pub(crate) fn capture(
    workflow: &Path,
    files: &[EvidenceFile],
    budget: usize,
) -> Result<Vec<CapturedFile>, HostError> {
    validate(files)?;
    let mut remaining = budget as u64;
    for expected in files.iter().filter_map(|file| file.content.as_ref()) {
        if expected.byte_length > remaining {
            return Err(invalid(
                "full dependency closure exceeds the selected portable extent",
            ));
        }
        remaining -= expected.byte_length;
    }
    let mut captured = Vec::new();
    for (dependency, commitment) in files.iter().enumerate() {
        let Some(mut member) = opened::open_member(workflow, dependency, commitment)? else {
            continue;
        };
        let mut original = Vec::new();
        member.copy_verified(&mut original)?;
        captured.push(CapturedFile {
            dependency: member.dependency(),
            original,
        });
    }
    Ok(captured)
}

/// Read one committed phase stream without acquiring or copying unrelated members.
pub(crate) fn phase_window(
    workflow: &Path,
    files: &[EvidenceFile],
    ordinal: usize,
    role: EvidenceRole,
    offset: u64,
    length: std::num::NonZeroU64,
    tail: bool,
) -> Result<crate::artifacts::DescriptorRead, HostError> {
    file_window(workflow, files, ordinal, Some(role), offset, length, tail)
}

/// Read one committed workflow-owned file without including inputs or other members.
pub(crate) fn file_window(
    workflow: &Path,
    files: &[EvidenceFile],
    ordinal: usize,
    role: Option<EvidenceRole>,
    offset: u64,
    length: std::num::NonZeroU64,
    tail: bool,
) -> Result<crate::artifacts::DescriptorRead, HostError> {
    let commitment = files
        .get(ordinal)
        .filter(|file| file.root == EvidenceRoot::Workflow && file.role != EvidenceRole::Input
            && role.is_none_or(|role| file.role == role))
        .ok_or_else(|| invalid("selector does not name an original workflow member of the requested kind"))?;
    let mut member = opened::open_member(workflow, ordinal, commitment)?
        .ok_or_else(|| invalid("selected member was not captured by the original owner"))?;
    let observed = commitment.content.as_ref()
        .ok_or_else(|| invalid("selected member has no original extent"))?
        .byte_length;
    let start = if tail { observed.saturating_sub(length.get()) } else { offset };
    member.read_range(start, length)
}
/// One original ordinal/content account for inline bytes and physically opened members.
pub(crate) fn validate_members(
    files: &[EvidenceFile],
    observed: impl IntoIterator<Item = (usize, ContentIdentity)>,
) -> Result<(), HostError> {
    validate(files)?;
    let expected = files
        .iter()
        .enumerate()
        .filter_map(|(index, file)| file.content.as_ref().map(|content| (index, content)));
    let mut observed = observed.into_iter();
    for (index, content) in expected {
        let Some((ordinal, actual)) = observed.next() else {
            return Err(invalid(
                "included files do not cover the original dependency commitments",
            ));
        };
        if ordinal != index || &actual != content {
            return Err(invalid(
                "included file bytes or dependency association differ from the original report",
            ));
        }
    }
    if observed.next().is_some() {
        return Err(invalid(
            "included files exceed the original dependency commitments",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, os::unix::fs::symlink};
    fn private_root() -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        root
    }
    fn source(path: &str, settled: bool) -> SourceFile {
        SourceFile {
            root: EvidenceRoot::Workflow,
            path: path.into(),
            role: EvidenceRole::Stdout,
            group: Some(0),
            settled,
        }
    }
    #[test]
    fn exact_phase_window_refuses_wrong_stream_and_changed_bytes() {
        let root = private_root();
        let path = root.path().join("run-0.stdout");
        fs::write(&path, b"original").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        let (files, causes) = seal(root.path(), vec![source("run-0.stdout", true)]);
        assert!(causes.is_empty());
        let length = std::num::NonZeroU64::new(3).unwrap();
        let read = phase_window(root.path(), &files, 0, EvidenceRole::Stdout, 0, length, true).unwrap();
        assert_eq!(read.byte_range, 5..8);
        assert!(matches!(read.content, crate::artifacts::Content::Utf8(value) if value == "nal"));
        assert!(phase_window(root.path(), &files, 0, EvidenceRole::Stderr, 0, length, false).is_err());
        fs::remove_file(&path).unwrap();
        fs::write(&path, b"replaced").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        assert!(phase_window(root.path(), &files, 0, EvidenceRole::Stdout, 0, length, false).is_err());
    }
    #[test]
    fn sealed_original_dependencies_refuse_same_shape_mutation_missing_and_truncated_portable_closure()
     {
        let root = private_root();
        let path = root.path().join("run-0.stdout");
        let original = [0xff, b'\n', b'a'];
        fs::write(&path, original).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        let (files, causes) = seal(root.path(), vec![source("run-0.stdout", true)]);
        assert!(causes.is_empty());
        let captured = capture(root.path(), &files, original.len()).unwrap();
        assert_eq!(captured[0].original, original);
        validate_members(
            &files,
            captured
                .iter()
                .map(|file| (file.dependency, ContentIdentity::of(&file.original))),
        )
        .unwrap();
        assert!(validate_members(&files, []).is_err());
        let mut changed = captured.clone();
        changed[0].original[2] = b'b';
        assert!(
            validate_members(
                &files,
                changed
                    .iter()
                    .map(|file| (file.dependency, ContentIdentity::of(&file.original)))
            )
            .is_err()
        );
        let mut changed = captured.clone();
        changed.push(captured[0].clone());
        assert!(
            validate_members(
                &files,
                changed
                    .iter()
                    .map(|file| (file.dependency, ContentIdentity::of(&file.original)))
            )
            .is_err()
        );
        assert!(capture(root.path(), &files, original.len() - 1).is_err());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&path, [0xff, b'\n', b'b']).unwrap();
        assert!(revalidate(root.path(), &files).is_err());
        assert!(capture(root.path(), &files, 32).is_err());
        fs::remove_file(path).unwrap();
        assert!(capture(root.path(), &files, 32).is_err());
        assert_eq!(
            files[0].content.as_ref().unwrap(),
            &ContentIdentity::of(&original)
        );
    }
    #[test]
    fn original_omissions_do_not_become_commitments_when_files_appear_later() {
        let root = private_root();
        let (files, causes) = seal(
            root.path(),
            vec![
                source("unsettled.stdout", false),
                source("absent.stdout", true),
            ],
        );
        assert_eq!(causes.len(), 1);
        assert_eq!(files[0].omission, Some(EvidenceOmission::Unsettled));
        assert_eq!(files[1].omission, Some(EvidenceOmission::Missing));
        assert!(files[1].failure.is_some());
        for name in ["unsettled.stdout", "absent.stdout"] {
            fs::write(root.path().join(name), b"later bytes").unwrap();
        }
        revalidate(root.path(), &files).unwrap();
        assert!(capture(root.path(), &files, 1024).unwrap().is_empty());
        validate_members(&files, []).unwrap();
        assert!(files.iter().all(|file| file.content.is_none()));
    }
    #[test]
    fn native_closure_refuses_private_request_addresses_and_intermediate_symlinks() {
        let root = private_root();
        let outside = tempfile::tempdir().unwrap();
        fs::write(outside.path().join("secret"), b"private credential").unwrap();
        symlink(outside.path(), root.path().join("alias")).unwrap();
        let (files, causes) = seal(root.path(), vec![source("alias/secret", true)]);
        assert_eq!(causes.len(), 1);
        assert_eq!(files[0].omission, Some(EvidenceOmission::Refused));
        assert!(files[0].content.is_none());
        assert!(capture(root.path(), &files, 1024).unwrap().is_empty());
        let request = EvidenceFile {
            root: EvidenceRoot::Invocation,
            path: "request.json".into(),
            role: EvidenceRole::Input,
            group: None,
            content: None,
            omission: Some(EvidenceOmission::Refused),
            failure: None,
        };
        assert!(validate(&[request]).is_err());
        for path in [
            "../secret",
            "/absolute",
            "nested/../secret",
            "nested//secret",
            "nested/./secret",
            "nul\0file",
        ] {
            let mut file = files[0].clone();
            file.path = path.into();
            assert!(validate(&[file]).is_err());
        }
    }
    #[test]
    fn settled_runner_writer_finalizes_owned_descriptor_once_and_readers_never_repair_it() {
        let root = private_root();
        let path = root.path().join("junit.xml");
        fs::write(&path, b"<original-report/>").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o664)).unwrap();
        let mut declared = source("junit.xml", true);
        declared.role = EvidenceRole::RunnerReport;
        let (files, causes) = seal(root.path(), vec![declared]);
        assert!(causes.is_empty());
        assert_eq!(fs::metadata(&path).unwrap().mode() & 0o777, 0o400);
        let captured = capture(root.path(), &files, 1024).unwrap();
        assert_eq!(captured[0].original, b"<original-report/>");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o664)).unwrap();
        assert!(capture(root.path(), &files, 1024).is_err());
        assert!(revalidate(root.path(), &files).is_err());
        assert_eq!(
            fs::metadata(&path).unwrap().mode() & 0o777,
            0o664,
            "retained readers cannot finalize or repair a producer artifact"
        );
    }
    #[test]
    fn unresolved_runner_writer_remains_original_omission_without_chmod_or_hash() {
        let root = private_root();
        let path = root.path().join("junit.xml");
        fs::write(&path, b"partial original bytes").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o664)).unwrap();
        let mut declared = source("junit.xml", false);
        declared.role = EvidenceRole::RunnerReport;
        let (files, causes) = seal(root.path(), vec![declared]);
        assert!(causes.is_empty());
        assert_eq!(files[0].omission, Some(EvidenceOmission::Unsettled));
        assert!(files[0].content.is_none());
        assert_eq!(fs::metadata(&path).unwrap().mode() & 0o777, 0o664);
    }
    #[test]
    fn runner_finalization_refuses_hardlinks_and_path_replacement_before_chmod() {
        let root = private_root();
        let path = root.path().join("junit.xml");
        fs::write(&path, b"original bytes").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o664)).unwrap();
        let alias = root.path().join("alias.xml");
        fs::hard_link(&path, &alias).unwrap();
        let mut declared = source("junit.xml", true);
        declared.role = EvidenceRole::RunnerReport;
        let (files, causes) = seal(root.path(), vec![declared]);
        assert_eq!(causes.len(), 1);
        assert_eq!(files[0].omission, Some(EvidenceOmission::Refused));
        assert_eq!(fs::metadata(&path).unwrap().mode() & 0o777, 0o664);
        fs::remove_file(alias).unwrap();
        let opened = File::open(&path).unwrap();
        let original = root.path().join("original.xml");
        fs::rename(&path, &original).unwrap();
        fs::write(&path, b"replacement bytes").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o664)).unwrap();
        assert!(finalize_runner_descriptor(&path, opened).is_err());
        assert_eq!(fs::metadata(&path).unwrap().mode() & 0o777, 0o664);
        assert_eq!(fs::metadata(&original).unwrap().mode() & 0o777, 0o664);
        assert_eq!(fs::read(path).unwrap(), b"replacement bytes");
    }
}
