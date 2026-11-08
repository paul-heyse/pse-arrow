// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Same-parent publication of a verified deliverable and its original owner receipt.
use super::handler::error::Result as ActionResult;
use super::handler::{self, ProjectOperationError};
use crate::{
    CancelSource,
    workflow::{
        ProjectDraftDocuments, ProjectExportRequest, ProjectSelection, Runtime, StudyCohortReceipt,
        StudyCohortRequest, StudyKpiRequest, WorkflowError, contract, portable_payload,
    },
};
use pse_columnar::CancellationToken;
use pse_host::investigation::NativePath;
use pse_model::document::Version;
use pse_operations::canonical::CanonicalStore;
use rustix::fs::{CWD, Mode, OFlags, RenameFlags, renameat_with};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    fs::{self, DirBuilder, File, OpenOptions},
    io::{self, Read as _, Write as _},
    os::unix::{
        ffi::OsStrExt as _,
        fs::{DirBuilderExt as _, MetadataExt as _, OpenOptionsExt as _},
    },
    path::{Path, PathBuf},
};

const STAGE_PREFIX: &str = ".pse-project-publication-";
const INTENT: &str = "intent.json";
const CONTENT: &str = "content";
const RECEIPT: &str = "publication.json";
const MAX_DOCUMENT: usize = 1 << 20;
const COHORT_FILE: &str = "cohort.ndjson";
const RESULT_FILE: &str = "result.arrow";
const ANALYSIS_FILE: &str = "analysis.arrow";
const INDEX: &str = "project-publications";

/// Finite authored alternatives for original product publication owners.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ExchangeRequest {
    /// Selected portable project pack and its explicit physical admission inputs.
    Pack {
        /// Exact project export selection and allowances.
        request: ProjectExportRequest,
        /// Required for executable export; descriptive export uses metadata only.
        physical: Option<ProjectDraftDocuments>,
    },
    /// Bounded NDJSON pages from one complete frozen cut.
    Cohort {
        /// Exact cohort selection and population bounds.
        request: StudyCohortRequest,
        /// Finite byte allowance for encoded pages and receipt.
        #[serde(default = "super::cohort_export_bytes")]
        maximum_bytes: u64,
    },
    /// One original member's exact result row interval from a frozen cut.
    Result {
        /// Exact cohort selection and population bounds.
        request: StudyCohortRequest,
        /// Original member position.
        member: usize,
        /// Registered scientific result relation.
        relation: String,
        /// Inclusive original row coordinate.
        start: u64,
        /// Exclusive original row coordinate.
        end: u64,
        /// Finite Arrow IPC byte allowance.
        #[serde(default = "super::cohort_export_bytes")]
        maximum_bytes: u64,
    },
    /// Exact aggregate of one registered output over a frozen cut.
    Analysis {
        /// Exact cohort selection and population bounds.
        request: StudyCohortRequest,
        /// Original output, partition and row selection.
        kpi: StudyKpiRequest,
    },
}

/// The operation and complete typed request selected before any delivery work.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OriginalPublication {
    /// Local publication document version.
    pub version: Version<1>,
    /// Exact canonical issuing store.
    pub issuing_store: String,
    /// Caller-selected original operation key.
    pub operation: String,
    /// Exact authored absolute native destination.
    pub destination: NativePath,
    /// Encoding checksum of the complete original typed request.
    pub request_digest: pse_ids::ContentHash,
}

/// Verified output bytes in the published directory.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PublishedFile {
    /// Fixed relative output name.
    pub name: String,
    /// Exact synchronized file extent.
    pub bytes: u64,
    /// Encoding checksum of the original bytes.
    pub digest: pse_ids::ContentHash,
}

/// Original product acknowledgement, independent of protection granted at execution.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Deliverable {
    /// Verified project pack, with its full manifest available as the original `project.json`.
    Pack {
        /// Published manifest bytes.
        manifest: PublishedFile,
        /// Exact selected origin.
        origin: Box<ProjectSelection>,
        /// Actual verified copied member count.
        members: usize,
        /// Original decoding allowance for a recovered complete staged pack.
        maximum_document_bytes: usize,
        /// Original offline completeness fact.
        offline_complete: bool,
        /// Original scientific completeness fact.
        executable_complete: bool,
    },
    /// Original protected cohort cut and exported bounded NDJSON pages.
    Cohort {
        /// Attribution only. It grants no ability to reopen the capture.
        cohort: StudyCohortReceipt,
        /// Exported cohort pages.
        output: PublishedFile,
    },
    /// Exact selected cohort member result.
    Result {
        /// Attribution only. It grants no ability to reopen the capture.
        cohort: StudyCohortReceipt,
        /// Exact original member position.
        member: usize,
        /// Exported Arrow IPC result.
        output: PublishedFile,
    },
    /// Exact selected cohort analysis and aggregate publication facts.
    Analysis {
        /// Attribution only. It grants no ability to reopen the capture.
        cohort: StudyCohortReceipt,
        /// Number of included original members.
        included: u64,
        /// Number of unavailable or empty original members.
        unavailable: u64,
        /// Exported Arrow IPC aggregate when there were batches.
        output: Option<PublishedFile>,
    },
}
impl Deliverable {
    fn file(&self) -> Option<&PublishedFile> {
        match self {
            Self::Pack { manifest, .. } => Some(manifest),
            Self::Cohort { output, .. } | Self::Result { output, .. } => Some(output),
            Self::Analysis { output, .. } => output.as_ref(),
        }
    }
}

/// Complete staged deliverable and exact original attribution.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PublicationReceipt {
    /// Exact request/destination binding.
    pub original: OriginalPublication,
    /// Original native product result.
    pub deliverable: Deliverable,
}

/// Observed custody or publication. Absence provides no permission to retry an effect.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PublicationStatus {
    /// No exact operation record was observed.
    Unknown,
    /// The exact original was claimed, with no complete staged deliverable observed.
    Pending {
        /// Original bound intent.
        original: OriginalPublication,
    },
    /// A joined original producer failed before publication; its incomplete bytes were released.
    Failed {
        /// Original bound intent retained independently of temporary content.
        original: OriginalPublication,
    },
    /// Complete original bytes are staged and may be published without repeating capture.
    Ready {
        /// Original complete staged receipt.
        receipt: Box<PublicationReceipt>,
    },
    /// Original receipt and deliverable were observed together at the destination.
    Published {
        /// Exact observed publication.
        receipt: Box<PublicationReceipt>,
    },
}

fn io_error(source: io::Error) -> WorkflowError {
    pse_engine::EngineError::Infrastructure {
        op: "publish or observe exact project deliverable".into(),
        source: Box::new(source),
    }
    .into()
}
fn checkpoint(cancel: &CancellationToken) -> Result<(), WorkflowError> {
    Ok(cancel.checkpoint().map_err(pse_engine::EngineError::from)?)
}
fn directory(path: &Path) -> io::Result<File> {
    rustix::fs::open(
        path,
        OFlags::RDONLY | OFlags::CLOEXEC | OFlags::DIRECTORY | OFlags::NOFOLLOW,
        Mode::empty(),
    )
    .map(File::from)
    .map_err(Into::into)
}
fn optional_directory(path: &Path) -> Result<Option<File>, WorkflowError> {
    match directory(path) {
        Ok(dir) => Ok(Some(dir)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(io_error(error)),
    }
}
fn private_directory(path: &Path) -> Result<Option<File>, WorkflowError> {
    let Some(dir) = optional_directory(path)? else {
        return Ok(None);
    };
    let metadata = dir.metadata().map_err(io_error)?;
    if metadata.uid() != rustix::process::geteuid().as_raw() || metadata.mode() & 0o077 != 0 {
        return Err(contract(
            "publication index requires an owned private state directory",
        ));
    }
    Ok(Some(dir))
}
fn read_document<T: DeserializeOwned>(dir: &File, name: &str) -> Result<Option<T>, WorkflowError> {
    let mut file = match portable_payload::open_contained(dir, name) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(io_error(error)),
    };
    let metadata = file.metadata().map_err(io_error)?;
    if metadata.uid() != rustix::process::geteuid().as_raw() || metadata.mode() & 0o022 != 0 {
        return Err(contract(
            "publication receipt is not an owned non-writable-by-others document",
        ));
    }
    let before = pse_host::artifacts::descriptor_identity(&file)
        .map_err(|error| io_error(io::Error::other(error)))?;
    let mut bytes = Vec::new();
    (&mut file)
        .take(MAX_DOCUMENT as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(io_error)?;
    let after = pse_host::artifacts::descriptor_identity(&file)
        .map_err(|error| io_error(io::Error::other(error)))?;
    if before != after || bytes.len() > MAX_DOCUMENT {
        return Err(contract(
            "original publication document changed or exceeds its bound",
        ));
    }
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|error| contract(error.to_string()))
}
fn document(value: &impl Serialize) -> Result<Vec<u8>, WorkflowError> {
    let bytes = serde_json::to_vec(value).map_err(|error| contract(error.to_string()))?;
    if bytes.len() > MAX_DOCUMENT {
        return Err(contract("original publication document exceeds its bound"));
    }
    Ok(bytes)
}

struct OperationIndex {
    state: PathBuf,
    root: PathBuf,
    key: String,
    failed: String,
}

impl OperationIndex {
    fn select(state: &Path, operation: &str, issuer: &str) -> Result<Self, WorkflowError> {
        if !state.is_absolute() || private_directory(state)?.is_none() {
            return Err(contract(
                "the original publication state directory is unavailable",
            ));
        }
        let state = state.canonicalize().map_err(io_error)?;
        let key = pse_ids::encoding_checksum(&document(&(issuer, operation))?)
            .content_hash()
            .to_hex();
        let root = state.join(INDEX);
        Ok(Self {
            state,
            root,
            key: format!("{key}.json"),
            failed: format!("{key}.failed.json"),
        })
    }

    fn bound(&self) -> Result<Option<OriginalPublication>, WorkflowError> {
        let Some(root) = private_directory(&self.root)? else {
            return Ok(None);
        };
        read_document(&root, &self.key)
    }

    fn status(
        &self,
        publication: &Publication,
        operation: &str,
        issuer: &str,
        cancel: &CancellationToken,
    ) -> Result<PublicationStatus, WorkflowError> {
        checkpoint(cancel)?;
        let Some(root) = private_directory(&self.root)? else {
            return Ok(PublicationStatus::Unknown);
        };
        let Some(original) = read_document::<OriginalPublication>(&root, &self.key)? else {
            return Ok(PublicationStatus::Unknown);
        };
        publication.matches(&original, operation, issuer)?;
        if let Some(failed) = read_document::<OriginalPublication>(&root, &self.failed)? {
            if failed != original {
                return Err(contract(
                    "failed publication disposition differs from its bound operation",
                ));
            }
            return Ok(PublicationStatus::Failed { original });
        }
        let status = publication.status(operation, issuer, cancel)?;
        let retained = match &status {
            PublicationStatus::Unknown => return Ok(PublicationStatus::Pending { original }),
            PublicationStatus::Pending { original } | PublicationStatus::Failed { original } => {
                original
            }
            PublicationStatus::Ready { receipt } | PublicationStatus::Published { receipt } => {
                &receipt.original
            }
        };
        if retained != &original {
            return Err(contract(
                "destination publication differs from its bound original operation",
            ));
        }
        Ok(status)
    }

    fn write(&self, name: &str, original: &OriginalPublication) -> Result<bool, WorkflowError> {
        let created = match DirBuilder::new().mode(0o700).create(&self.root) {
            Ok(()) => true,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => false,
            Err(error) => return Err(io_error(error)),
        };
        let root = private_directory(&self.root)?
            .ok_or_else(|| contract("original publication index disappeared"))?;
        if created {
            directory(&self.state)
                .and_then(|dir| dir.sync_all())
                .map_err(io_error)?;
        }
        let mut temporary = tempfile::NamedTempFile::new_in(&self.root).map_err(io_error)?;
        temporary
            .write_all(&document(original)?)
            .map_err(io_error)?;
        temporary.as_file().sync_all().map_err(io_error)?;
        let added = match temporary.persist_noclobber(self.root.join(name)) {
            Ok(_) => true,
            Err(error) if error.error.kind() == io::ErrorKind::AlreadyExists => false,
            Err(error) => return Err(io_error(error.error)),
        };
        root.sync_all().map_err(io_error)?;
        Ok(added)
    }
}

fn checksum(
    file: File,
    maximum: u64,
    cancel: &CancellationToken,
) -> Result<(u64, pse_ids::ContentHash), WorkflowError> {
    let mut source = file.take(maximum.saturating_add(1));
    let mut digest = pse_ids::EncodingHasher::new();
    let mut buffer = [0; 64 << 10];
    let mut extent = 0u64;
    loop {
        checkpoint(cancel)?;
        let count = source.read(&mut buffer).map_err(io_error)?;
        if count == 0 {
            break;
        }
        extent += count as u64;
        if extent > maximum {
            return Err(contract(
                "original published file exceeds its byte allowance",
            ));
        }
        digest.update(&buffer[..count]);
    }
    Ok((extent, digest.finish().content_hash()))
}

#[derive(Debug)]
pub(super) struct Publication {
    original_destination: NativePath,
    destination: PathBuf,
    stage: PathBuf,
    parent: File,
}
impl Publication {
    pub(super) fn select(destination: NativePath) -> Result<Self, WorkflowError> {
        let path = destination.to_path_buf();
        let name = path
            .file_name()
            .filter(|name| !name.as_bytes().starts_with(STAGE_PREFIX.as_bytes()))
            .filter(|_| path.is_absolute())
            .ok_or_else(|| {
                contract("publication requires an explicit absolute destination directory name")
            })?;
        let parent_path = path
            .parent()
            .ok_or_else(|| contract("publication destination lacks a parent"))?
            .canonicalize()
            .map_err(io_error)?;
        let parent = directory(&parent_path).map_err(io_error)?;
        let key = pse_ids::encoding_checksum(name.as_bytes())
            .content_hash()
            .to_hex();
        Ok(Self {
            original_destination: destination,
            destination: parent_path.join(name),
            stage: parent_path.join(format!("{STAGE_PREFIX}{key}")),
            parent,
        })
    }

    pub(super) fn content(&self) -> PathBuf {
        self.stage.join(CONTENT)
    }

    pub(super) fn original(
        &self,
        operation: &str,
        issuer: &str,
        request: &impl Serialize,
    ) -> Result<OriginalPublication, WorkflowError> {
        Ok(OriginalPublication {
            version: Version,
            operation: operation.into(),
            issuing_store: issuer.into(),
            destination: self.original_destination.clone(),
            request_digest: pse_ids::encoding_checksum(&document(request)?).content_hash(),
        })
    }

    pub(super) fn claim(&self, original: &OriginalPublication) -> Result<(), WorkflowError> {
        let parent = self
            .destination
            .parent()
            .ok_or_else(|| contract("publication destination lacks a parent"))?;
        let temporary = tempfile::Builder::new()
            .prefix(STAGE_PREFIX)
            .tempdir_in(parent)
            .map_err(io_error)?;
        let mut intent = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(temporary.path().join(INTENT))
            .map_err(io_error)?;
        intent.write_all(&document(original)?).map_err(io_error)?;
        intent.sync_all().map_err(io_error)?;
        directory(temporary.path())
            .and_then(|dir| dir.sync_all())
            .map_err(io_error)?;
        renameat_with(
            CWD,
            temporary.path(),
            CWD,
            &self.stage,
            RenameFlags::NOREPLACE,
        )
        .map_err(|error| io_error(error.into()))?;
        self.parent.sync_all().map_err(io_error)
    }

    pub(super) fn file(
        &self,
        name: &str,
        maximum: u64,
        cancel: &CancellationToken,
    ) -> Result<PublishedFile, WorkflowError> {
        let dir = directory(&self.content()).map_err(io_error)?;
        let file = portable_payload::open_contained(&dir, name).map_err(io_error)?;
        let (bytes, digest) = checksum(file, maximum, cancel)?;
        Ok(PublishedFile {
            name: name.into(),
            bytes,
            digest,
        })
    }

    fn matches(
        &self,
        original: &OriginalPublication,
        operation: &str,
        issuer: &str,
    ) -> Result<(), WorkflowError> {
        if original.operation != operation
            || original.issuing_store != issuer
            || original.destination != self.original_destination
        {
            return Err(contract(
                "publication destination belongs to a different original operation",
            ));
        }
        Ok(())
    }

    fn published(
        &self,
        operation: &str,
        issuer: &str,
        cancel: &CancellationToken,
    ) -> Result<Option<PublicationReceipt>, WorkflowError> {
        let Some(dir) = optional_directory(&self.destination)? else {
            return Ok(None);
        };
        let receipt: PublicationReceipt = read_document(&dir, RECEIPT)?.ok_or_else(|| {
            contract("destination already exists without an original publication receipt")
        })?;
        self.matches(&receipt.original, operation, issuer)?;
        verify_file(&dir, &receipt.deliverable, cancel)?;
        Ok(Some(receipt))
    }

    pub(super) fn status(
        &self,
        operation: &str,
        issuer: &str,
        cancel: &CancellationToken,
    ) -> Result<PublicationStatus, WorkflowError> {
        checkpoint(cancel)?;
        if let Some(receipt) = self.published(operation, issuer, cancel)? {
            return Ok(PublicationStatus::Published {
                receipt: Box::new(receipt),
            });
        }
        if let Some(stage) = optional_directory(&self.stage)? {
            if let Some(original) = read_document::<OriginalPublication>(&stage, INTENT)? {
                self.matches(&original, operation, issuer)?;
                if let Some(content) = optional_directory(&self.content())?
                    && let Some(receipt) = read_document::<PublicationReceipt>(&content, RECEIPT)?
                {
                    if original != receipt.original {
                        return Err(contract(
                            "staged publication receipt differs from original intent",
                        ));
                    }
                    verify_file(&content, &receipt.deliverable, cancel)?;
                    return Ok(PublicationStatus::Ready {
                        receipt: Box::new(receipt),
                    });
                }
                return Ok(match self.published(operation, issuer, cancel)? {
                    Some(receipt) => PublicationStatus::Published {
                        receipt: Box::new(receipt),
                    },
                    None => PublicationStatus::Pending { original },
                });
            }
        }
        Ok(match self.published(operation, issuer, cancel)? {
            Some(receipt) => PublicationStatus::Published {
                receipt: Box::new(receipt),
            },
            None => PublicationStatus::Unknown,
        })
    }

    pub(super) fn finish(
        &self,
        original: OriginalPublication,
        deliverable: Deliverable,
        cancel: &CancellationToken,
    ) -> Result<PublicationStatus, WorkflowError> {
        let receipt = PublicationReceipt {
            original,
            deliverable,
        };
        checkpoint(cancel)?;
        self.stage_receipt(&receipt)?;
        self.publish_ready(Box::new(receipt), cancel)
    }

    fn stage_receipt(&self, receipt: &PublicationReceipt) -> Result<(), WorkflowError> {
        use std::os::unix::fs::PermissionsExt as _;
        let content = self.content();
        fs::set_permissions(&content, fs::Permissions::from_mode(0o700)).map_err(io_error)?;
        let directory = directory(&content).map_err(io_error)?;
        let temporary = ".publication-tmp";
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(content.join(temporary))
            .map_err(io_error)?;
        output.write_all(&document(receipt)?).map_err(io_error)?;
        output.sync_all().map_err(io_error)?;
        renameat_with(
            &directory,
            temporary,
            &directory,
            RECEIPT,
            RenameFlags::NOREPLACE,
        )
        .map_err(|error| io_error(error.into()))?;
        directory.sync_all().map_err(io_error)?;
        Ok(())
    }

    fn recover(
        &self,
        status: PublicationStatus,
        cancel: &CancellationToken,
    ) -> Result<PublicationStatus, WorkflowError> {
        match status {
            PublicationStatus::Ready { receipt } => {
                if let Deliverable::Pack {
                    manifest,
                    maximum_document_bytes,
                    ..
                } = &receipt.deliverable
                {
                    let pack = crate::workflow::ProjectBundleReader::open(
                        &self.content(),
                        *maximum_document_bytes,
                    )?;
                    if pack.digest() != manifest.digest.to_hex() {
                        return Err(contract(
                            "verified staged pack differs from its original publication receipt",
                        ));
                    }
                }
                self.publish_ready(receipt, cancel)
            }
            PublicationStatus::Published { receipt } => {
                self.parent.sync_all().map_err(io_error)?;
                self.release_stage(&receipt.original)?;
                Ok(PublicationStatus::Published { receipt })
            }
            original => Ok(original),
        }
    }

    fn publish_ready(
        &self,
        receipt: Box<PublicationReceipt>,
        cancel: &CancellationToken,
    ) -> Result<PublicationStatus, WorkflowError> {
        checkpoint(cancel)?;
        let Some(stage) = optional_directory(&self.stage)? else {
            if let Some(actual) = self.published(
                &receipt.original.operation,
                &receipt.original.issuing_store,
                cancel,
            )? && actual.original == receipt.original
            {
                self.parent.sync_all().map_err(io_error)?;
                return Ok(PublicationStatus::Published {
                    receipt: Box::new(actual),
                });
            }
            return Err(contract(
                "complete original stage disappeared before publication",
            ));
        };
        stage.sync_all().map_err(io_error)?;
        let destination = self
            .destination
            .file_name()
            .ok_or_else(|| contract("publication destination name disappeared"))?;
        let receipt = match renameat_with(
            &stage,
            CONTENT,
            &self.parent,
            destination,
            RenameFlags::NOREPLACE,
        ) {
            Ok(()) => receipt,
            Err(error) => Box::new(
                self.published(
                    &receipt.original.operation,
                    &receipt.original.issuing_store,
                    cancel,
                )?
                .filter(|actual| actual.original == receipt.original)
                .ok_or_else(|| io_error(error.into()))?,
            ),
        };
        stage.sync_all().map_err(io_error)?;
        self.parent.sync_all().map_err(io_error)?;
        self.release_stage(&receipt.original)?;
        Ok(PublicationStatus::Published { receipt })
    }

    fn release_stage(&self, original: &OriginalPublication) -> Result<(), WorkflowError> {
        if let Some(stage) = optional_directory(&self.stage)? {
            if let Some(bound) = read_document::<OriginalPublication>(&stage, INTENT)? {
                if bound != *original {
                    return Err(contract(
                        "staging custody differs from the published original",
                    ));
                }
                stage.sync_all().map_err(io_error)?;
                remove_if_present(fs::remove_file(self.stage.join(INTENT)))?;
            }
            remove_if_present(fs::remove_dir(&self.stage))?;
            self.parent.sync_all().map_err(io_error)?;
        }
        Ok(())
    }

    fn discard_partial(
        &self,
        original: &OriginalPublication,
        claimed: bool,
    ) -> Result<bool, WorkflowError> {
        if let Some(final_directory) = optional_directory(&self.destination)?
            && read_document::<PublicationReceipt>(&final_directory, RECEIPT)?
                .is_some_and(|receipt| receipt.original == *original)
        {
            return Ok(false);
        }
        let Some(stage) = private_directory(&self.stage)? else {
            return Ok(true);
        };
        if read_document::<OriginalPublication>(&stage, INTENT)?.as_ref() != Some(original) {
            if !claimed {
                return Ok(true);
            }
            return Err(contract(
                "partial staging is not bound to the settled original producer",
            ));
        }
        let content = self.content();
        if let Some(directory) = optional_directory(&content)? {
            if read_document::<PublicationReceipt>(&directory, RECEIPT)?.is_some() {
                return Ok(false);
            }
            fs::remove_dir_all(&content).map_err(io_error)?;
        }
        self.release_stage(original)?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publication_recovery_preserves_the_exact_original_and_never_publishes_a_partial_file() {
        let root = tempfile::tempdir().unwrap();
        let destination = root.path().join("delivery");
        let target = Publication::select(destination.clone().into()).unwrap();
        let cancel = CancellationToken::new();
        let original = target
            .original("operation", "issuer", &"original request")
            .unwrap();
        let changed = target
            .original("operation", "issuer", &"changed request")
            .unwrap();
        assert_ne!(original.request_digest, changed.request_digest);
        assert!(matches!(
            target.status("operation", "issuer", &cancel).unwrap(),
            PublicationStatus::Unknown
        ));
        target.claim(&original).unwrap();
        assert!(target.claim(&changed).is_err());
        assert!(target.status("other", "issuer", &cancel).is_err());
        assert!(target.status("operation", "other", &cancel).is_err());
        let content = target.content();
        fs::create_dir(&content).unwrap();
        let payload = content.join(COHORT_FILE);
        fs::write(&payload, b"complete original bytes").unwrap();
        File::open(&payload).unwrap().sync_all().unwrap();
        let pending = target
            .recover(
                target.status("operation", "issuer", &cancel).unwrap(),
                &cancel,
            )
            .unwrap();
        assert!(
            matches!(pending, PublicationStatus::Pending { original: retained } if retained == original)
        );
        assert!(!destination.exists());
        let receipt = PublicationReceipt {
            original: original.clone(),
            deliverable: Deliverable::Cohort {
                cohort: StudyCohortReceipt {
                    issuing_store: "issuer".into(),
                    capture: "original-cut".into(),
                    request: StudyCohortRequest {
                        filter: Default::default(),
                        maximum_studies: 1,
                        maximum_members: 1,
                    },
                    header_count: 0,
                    member_count: 0,
                    complete: true,
                },
                output: target.file(COHORT_FILE, 256, &cancel).unwrap(),
            },
        };
        target.stage_receipt(&receipt).unwrap();
        let stopped = CancellationToken::new();
        stopped.cancel();
        assert!(
            target
                .recover(
                    target.status("operation", "issuer", &cancel).unwrap(),
                    &stopped
                )
                .is_err()
        );
        assert!(!destination.exists());
        assert!(matches!(
            target.status("operation", "issuer", &cancel).unwrap(),
            PublicationStatus::Ready { .. }
        ));
        for _ in 0..2 {
            let PublicationStatus::Published { receipt: retained } = target
                .recover(
                    target.status("operation", "issuer", &cancel).unwrap(),
                    &cancel,
                )
                .unwrap()
            else {
                panic!("the complete original must publish without taking another cut");
            };
            assert_eq!(
                document(retained.as_ref()).unwrap(),
                document(&receipt).unwrap()
            );
        }
        assert!(!target.stage.exists());
        assert_eq!(
            fs::read(destination.join(COHORT_FILE)).unwrap(),
            b"complete original bytes"
        );
        fs::write(destination.join(COHORT_FILE), b"changed").unwrap();
        assert!(target.status("operation", "issuer", &cancel).is_err());
    }

    #[test]
    fn existing_destination_with_no_receipt_never_becomes_an_original_publication() {
        let root = tempfile::tempdir().unwrap();
        let destination = root.path().join("existing");
        fs::create_dir(&destination).unwrap();
        let target = Publication::select(destination.into()).unwrap();
        assert!(
            target
                .status("operation", "issuer", &CancellationToken::new())
                .is_err()
        );
        assert!(!target.stage.exists());
    }

    #[test]
    fn publication_index_rejects_changed_destination_and_retains_settled_failure_without_partial_bytes()
     {
        let root = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir_in(root.path()).unwrap();
        let target = Publication::select(root.path().join("original").into()).unwrap();
        let different = Publication::select(root.path().join("changed").into()).unwrap();
        let index = OperationIndex::select(state.path(), "operation", "issuer").unwrap();
        let token = CancellationToken::new();
        let original = target.original("operation", "issuer", &"request").unwrap();
        let changed = different
            .original("operation", "issuer", &"request")
            .unwrap();
        assert!(index.write(&index.key, &original).unwrap());
        assert!(!index.write(&index.key, &changed).unwrap());
        assert!(
            index
                .status(&different, "operation", "issuer", &token)
                .is_err()
        );
        assert!(!different.destination.exists());
        assert!(!different.stage.exists());
        target.claim(&original).unwrap();
        fs::create_dir(target.content()).unwrap();
        fs::write(
            target.content().join("cohort.ndjson.incomplete"),
            b"partial",
        )
        .unwrap();
        assert!(target.discard_partial(&original, true).unwrap());
        assert!(index.write(&index.failed, &original).unwrap());
        assert!(!target.stage.exists());
        assert!(!target.destination.exists());
        let recovered = index
            .status(&target, "operation", "issuer", &token)
            .unwrap();
        assert!(
            matches!(recovered, PublicationStatus::Failed { original: retained } if retained == original)
        );
        assert!(
            index
                .status(&different, "operation", "issuer", &token)
                .is_err()
        );
    }
}

fn remove_if_present(result: io::Result<()>) -> Result<(), WorkflowError> {
    match result {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_error(error)),
    }
}

fn verify_file(
    dir: &File,
    deliverable: &Deliverable,
    cancel: &CancellationToken,
) -> Result<(), WorkflowError> {
    let name = match deliverable {
        Deliverable::Pack { .. } => "project.json",
        Deliverable::Cohort { .. } => COHORT_FILE,
        Deliverable::Result { .. } => RESULT_FILE,
        Deliverable::Analysis { .. } => ANALYSIS_FILE,
    };
    if let Some(expected) = deliverable.file() {
        if expected.name != name {
            return Err(contract("original publication output name differs"));
        }
        let file = portable_payload::open_contained(dir, name).map_err(io_error)?;
        let (bytes, digest) = checksum(file, expected.bytes, cancel)?;
        if expected.bytes != bytes || expected.digest != digest {
            return Err(contract("original published file differs from its receipt"));
        }
    } else {
        match portable_payload::open_contained(dir, name) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Ok(_) => return Err(contract("unexpected original analysis output")),
            Err(error) => return Err(io_error(error)),
        }
    }
    Ok(())
}

/// Read back the exact original local publication without a service connection.
/// # Errors
/// Wrong original identity, unsafe destination or changed receipt/output bytes.
pub fn publication_status(
    state: &Path,
    destination: NativePath,
    operation: &str,
    issuer: &str,
    cancel: &CancellationToken,
) -> Result<PublicationStatus, WorkflowError> {
    let index = OperationIndex::select(state, operation, issuer)?;
    index.status(
        &Publication::select(destination)?,
        operation,
        issuer,
        cancel,
    )
}

/// Finish an exact complete staged publication without reopening protected payloads.
/// # Errors
/// Wrong original identity, changed original bytes, or publication and synchronization failure.
pub fn recover_publication(
    state: &Path,
    destination: NativePath,
    operation: &str,
    issuer: &str,
    cancel: &CancellationToken,
) -> Result<PublicationStatus, WorkflowError> {
    let index = OperationIndex::select(state, operation, issuer)?;
    let publication = Publication::select(destination)?;
    publication.recover(
        index.status(&publication, operation, issuer, cancel)?,
        cancel,
    )
}

/// Original local publication and its independently attempted cleanup.
#[derive(Debug)]
pub struct ExportOutcome {
    /// The original operation or its refusal after the custody attempt.
    pub result: Result<PublicationStatus, ProjectOperationError>,
    /// Independently attempted scientific runtime drain.
    pub cleanup: Option<crate::math::MathRuntimeError>,
    /// Failure to dispose of an exact known partial delivery after its producer joined.
    pub publication_cleanup: Option<WorkflowError>,
}

/// Claim and publish an exact authored product deliverable once.
/// # Errors
/// Outer errors precede the publication claim. Inner errors retain possible stage or publication.
pub async fn export(
    store: &CanonicalStore,
    operation: &str,
    destination: NativePath,
    request: ExchangeRequest,
    cancel: &CancelSource,
) -> Result<ExportOutcome, ProjectOperationError> {
    let token = cancel.token();
    let selected: ActionResult<_> = (|| {
        checkpoint(&token)?;
        let issuer = store.issuing_store()?;
        if let ExchangeRequest::Pack { request, physical } = &request {
            if request.selection.issuing_store != issuer {
                return Err(
                    contract("project export selection belongs to a different issuer").into(),
                );
            }
            if physical.is_some() == request.descriptive_only {
                return Err(contract(
                    "only executable project export requires explicit physical documents",
                )
                .into());
            }
        }
        let publication = Publication::select(destination)?;
        let original = publication.original(operation, issuer, &request)?;
        let index = OperationIndex::select(store.deployment_state(), operation, issuer)?;
        let status = if let Some(bound) = index.bound()? {
            if bound != original {
                return Err(contract(
                    "original project publication was issued with different inputs or destination",
                )
                .into());
            }
            index.status(&publication, operation, issuer, &token)?
        } else {
            if !matches!(
                publication.status(operation, issuer, &token)?,
                PublicationStatus::Unknown
            ) {
                return Err(contract(
                    "destination retains a publication without its original operation index",
                )
                .into());
            }
            PublicationStatus::Unknown
        };
        Ok((index, publication, original, status))
    })();
    let (index, publication, original, status) = selected.map_err(ProjectOperationError::action)?;
    if !matches!(status, PublicationStatus::Unknown) {
        return Ok(ExportOutcome {
            result: Ok(status),
            cleanup: None,
            publication_cleanup: None,
        });
    }

    match index.write(&index.key, &original) {
        Ok(true) => {}
        Ok(false) => {
            let bound = index
                .bound()
                .map_err(|error| ProjectOperationError::action(error.into()))?;
            if bound.as_ref() != Some(&original) {
                return Err(ProjectOperationError::action(contract("original project publication was issued with different inputs or destination").into()));
            }
            let result = index
                .status(&publication, operation, &original.issuing_store, &token)
                .map_err(|error| ProjectOperationError::action(error.into()));
            return Ok(ExportOutcome {
                result,
                cleanup: None,
                publication_cleanup: None,
            });
        }
        Err(error) => {
            return Ok(ExportOutcome {
                result: Err(ProjectOperationError::action(error.into())),
                cleanup: None,
                publication_cleanup: None,
            });
        }
    }
    let (_spill, shared, runtime) = match handler::prepared_runtime(store) {
        Ok(runtime) => runtime,
        Err(error) => {
            return Ok(ExportOutcome {
                result: Err(ProjectOperationError::action(error)),
                cleanup: None,
                publication_cleanup: index.write(&index.failed, &original).err(),
            });
        }
    };
    let mut owns_stage = false;
    let result = Box::pin(async {
        checkpoint(&token)?;
        publication.claim(&original)?;
        owns_stage = true;
        deliver(
            &runtime,
            &shared,
            &publication,
            original.clone(),
            request,
            cancel,
        )
        .await
    })
    .await
    .map_err(ProjectOperationError::action);
    runtime.close();
    let cleanup = shared.math().shutdown_native_lanes().await.err();
    let publication_cleanup = if result.is_err() && cleanup.is_none() {
        (|| {
            if publication.discard_partial(&original, owns_stage)? {
                index.write(&index.failed, &original)?;
            }
            Ok::<_, WorkflowError>(())
        })()
        .err()
    } else {
        None
    };
    Ok(ExportOutcome {
        result,
        cleanup,
        publication_cleanup,
    })
}

async fn deliver(
    runtime: &Runtime,
    shared: &crate::SharedRuntime,
    publication: &Publication,
    original: OriginalPublication,
    request: ExchangeRequest,
    cancel: &CancelSource,
) -> ActionResult<PublicationStatus> {
    let token = cancel.token();
    let content = publication.content();
    if !matches!(request, ExchangeRequest::Pack { .. }) {
        fs::create_dir(&content).map_err(io_error)?;
    }
    match request {
        ExchangeRequest::Pack { request, physical } => {
            let manifest = if let Some(physical) = physical {
                let physical =
                    handler::admit_project_physical(runtime, shared, &physical, cancel).await?;
                runtime
                    .project_export(&content, &request, physical, cancel)
                    .await?
            } else {
                runtime
                    .project_export_metadata(&content, &request, cancel)
                    .await?
            };
            directory(&content.join("members"))
                .and_then(|dir| dir.sync_all())
                .map_err(io_error)?;
            fs::remove_dir(content.join(".staging")).map_err(io_error)?;
            let file = publication.file(
                "project.json",
                request.maximum_document_bytes as u64,
                &token,
            )?;
            Ok(publication.finish(
                original,
                Deliverable::Pack {
                    manifest: file,
                    origin: Box::new(manifest.origin),
                    members: manifest.members.len(),
                    maximum_document_bytes: request.maximum_document_bytes,
                    offline_complete: manifest.offline_complete,
                    executable_complete: manifest.executable_complete,
                },
                &token,
            )?)
        }
        ExchangeRequest::Cohort {
            request,
            maximum_bytes,
        } => {
            let cohort = runtime.freeze_study_cohort(request).await?;
            let bytes = cohort
                .export_pages(&content.join(COHORT_FILE), maximum_bytes, &token)
                .await?;
            let output = publication.file(COHORT_FILE, bytes, &token)?;
            if output.bytes != bytes {
                return Err(
                    contract("cohort output differs from the original writer extent").into(),
                );
            }
            Ok(publication.finish(
                original,
                Deliverable::Cohort {
                    cohort: cohort.receipt().clone(),
                    output,
                },
                &token,
            )?)
        }
        ExchangeRequest::Result {
            request,
            member,
            relation,
            start,
            end,
            maximum_bytes,
        } => {
            let cohort = runtime.freeze_study_cohort(request).await?;
            let mut reader = cohort
                .result(member, &relation, start, end, token.clone())
                .await?;
            let bytes = reader
                .export_ipc_bounded(&content.join(RESULT_FILE), maximum_bytes, &token)
                .await?;
            let output = publication.file(RESULT_FILE, bytes, &token)?;
            if output.bytes != bytes {
                return Err(
                    contract("result output differs from the original writer extent").into(),
                );
            }
            Ok(publication.finish(
                original,
                Deliverable::Result {
                    cohort: cohort.receipt().clone(),
                    member,
                    output,
                },
                &token,
            )?)
        }
        ExchangeRequest::Analysis { request, kpi } => {
            let cohort = runtime.freeze_study_cohort(request).await?;
            let analysis = cohort
                .export_analysis(&kpi, &content.join(ANALYSIS_FILE), &token)
                .await?;
            let output = analysis
                .output
                .as_ref()
                .map(|_| publication.file(ANALYSIS_FILE, 4 << 20, &token))
                .transpose()?;
            Ok(publication.finish(
                original,
                Deliverable::Analysis {
                    cohort: analysis.cohort,
                    included: analysis.included,
                    unavailable: analysis.unavailable,
                    output,
                },
                &token,
            )?)
        }
    }
}
