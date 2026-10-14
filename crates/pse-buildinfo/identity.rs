// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! External artifact association and fresh outer observation framing.
use std::{fs, path::Path};

/// Full synchronous observation envelope, including simultaneously live inventories,
/// bounded proc/configuration buffers, traversal frames and streamed encoding scratch.
/// Runtime jobs reserve this allowance before entering the observation owner.
pub const OBSERVATION_WORK_BYTES: usize =
    2 * SOURCE_METADATA_BYTES + (4 << 20) + 2 * MAPPING_BYTES;
// Complete imported native trees are source inputs too. Reconfirmation temporarily
// retains two inventories; each has this independently enforced finite allowance.
const SOURCE_METADATA_BYTES: usize = 16 << 20;
/// Supported composition roots bound configuration generation before Rust allocation.
pub const MAX_CONFIGURATION_BYTES: usize = 64 << 10;
const METADATA_BYTES: usize = 1 << 20;
const PATH_BYTES: usize = 4096;
// Scientific Python imports many file-backed extensions before composing a runtime.
// Two mapping buffers can coexist while resolving and verifying the actual module;
// reserve both above, independently of the small kernel/environment input limits.
const MAPPING_BYTES: usize = 1 << 20;
const ENVIRONMENT_BYTES: usize = 64 << 10;
const SOURCE_DEPTH: usize = 64;

// Cumulative conservative growth accounting deliberately does not credit removed
// entries: container spare capacity and overlapping clones remain covered. Each
// node charge includes allocator/BTree overhead, vector growth and value headers.
struct MetadataBudget {
    used: usize,
    limit: usize,
}
impl Default for MetadataBudget {
    fn default() -> Self {
        Self {
            used: 0,
            limit: METADATA_BYTES,
        }
    }
}
impl MetadataBudget {
    fn take(&mut self, bytes: usize) -> std::io::Result<()> {
        let next = self
            .used
            .checked_add(bytes)
            .ok_or_else(|| std::io::Error::other("observation metadata extent overflow"))?;
        if next > self.limit {
            return Err(std::io::Error::other("observation metadata byte limit"));
        }
        self.used = next;
        Ok(())
    }
    fn path(&mut self, path: &Path, copies: usize) -> std::io::Result<()> {
        let bytes = path.as_os_str().len();
        if bytes > PATH_BYTES {
            return Err(std::io::Error::other("observation path byte limit"));
        }
        self.take(
            (bytes + 256)
                .checked_mul(copies)
                .ok_or_else(|| std::io::Error::other("observation path extent overflow"))?,
        )
    }
}

fn bounded_read(
    path: &Path,
    limit: usize,
    checkpoint: &dyn Fn() -> std::io::Result<()>,
) -> std::io::Result<Vec<u8>> {
    use std::io::Read;
    checkpoint()?;
    let mut file = fs::File::open(path)?;
    let mut bytes = Vec::with_capacity(limit);
    let mut chunk = [0_u8; 4096];
    loop {
        checkpoint()?;
        let count = file.read(&mut chunk)?;
        if count == 0 {
            break;
        }
        if count > limit - bytes.len() {
            return Err(std::io::Error::other("observation input byte limit"));
        }
        bytes.extend_from_slice(&chunk[..count]);
    }
    checkpoint()?;
    Ok(bytes)
}
fn bounded_maps(checkpoint: &dyn Fn() -> std::io::Result<()>) -> std::io::Result<String> {
    String::from_utf8(bounded_read(
        Path::new("/proc/self/maps"),
        MAPPING_BYTES,
        checkpoint,
    )?)
    .map_err(std::io::Error::other)
}

struct CountWriter<'a> {
    bytes: u64,
    checkpoint: &'a dyn Fn() -> std::io::Result<()>,
}
impl std::io::Write for CountWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        (self.checkpoint)()?;
        self.bytes = self
            .bytes
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| std::io::Error::other("observation encoding extent overflow"))?;
        Ok(bytes.len())
    }
    fn write_all(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        // Interrupted is the original operation's stop, not a retryable write.
        self.write(bytes).map(|_| ())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        (self.checkpoint)()
    }
}
fn framed_json(
    hash: &mut pse_ids::FramedHasher,
    value: &impl serde::Serialize,
    checkpoint: &dyn Fn() -> std::io::Result<()>,
) -> std::io::Result<()> {
    let mut count = CountWriter {
        bytes: 0,
        checkpoint,
    };
    serde_json::to_writer(&mut count, value).map_err(json_error)?;
    hash.part_writer(count.bytes, checkpoint, |writer| {
        serde_json::to_writer(writer, value).map_err(json_error)
    })?;
    Ok(())
}
fn json_error(error: serde_json::Error) -> std::io::Error {
    std::io::Error::new(
        error
            .io_error_kind()
            .unwrap_or(std::io::ErrorKind::InvalidData),
        error,
    )
}
fn json_value_digest(
    value: &impl serde::Serialize,
    checkpoint: &dyn Fn() -> std::io::Result<()>,
) -> std::io::Result<String> {
    use sha2::Digest;
    struct DigestWriter<'a> {
        hash: sha2::Sha256,
        checkpoint: &'a dyn Fn() -> std::io::Result<()>,
    }
    impl std::io::Write for DigestWriter<'_> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            (self.checkpoint)()?;
            self.hash.update(bytes);
            Ok(bytes.len())
        }
        fn write_all(&mut self, bytes: &[u8]) -> std::io::Result<()> {
            // Hash writes are complete; retrying a cancelled checkpoint would
            // conceal cancellation and can loop forever inside serde_json.
            self.write(bytes).map(|_| ())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            (self.checkpoint)()
        }
    }
    let mut writer = DigestWriter {
        hash: sha2::Sha256::new(),
        checkpoint,
    };
    serde_json::to_writer(&mut writer, value).map_err(json_error)?;
    checkpoint()?;
    Ok(writer
        .hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

/// Current external deployment receipt interpretation. Scientific ProducerV1 keys
/// are unchanged; this version describes association and admission observations.
pub const DEPLOYMENT_RECEIPT_VERSION: u32 = 2;

/// Canonical Cargo-selected compilation unit in the established ProducerV1 payload.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProductionUnit {
    /// Stable compilation-unit key.
    pub key: String,
    /// Selected package source identity.
    pub package_id: String,
    /// Cargo target name.
    pub target_name: String,
    /// Cargo target kinds.
    pub target_kind: Vec<String>,
    /// Rust crate output types.
    pub crate_types: Vec<String>,
    /// Rust edition.
    pub edition: String,
    /// Cargo compilation mode.
    pub mode: String,
    /// Optional target platform.
    pub platform: Option<String>,
    /// Actual Cargo profile contract.
    pub profile: serde_json::Value,
    /// Actual selected features.
    pub features: Vec<String>,
    /// External dependency names and compilation-unit keys.
    pub dependencies: std::collections::BTreeMap<String, String>,
}
/// Established scientific consumed-input projection, independent of raw admission.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConsumedInput {
    /// Stable logical input path.
    pub path: String,
    /// Existing framed scientific input identity.
    pub identity: String,
    /// Source-selected representation interpretation.
    pub representation: String,
    /// Capture-local actual bytes; excluded from scientific serialization.
    #[serde(skip)]
    pub observation: Option<FileObservation>,
}
/// Exact established tuple/serde framing. Association and outer context never enter it.
pub fn scientific_producer_identity(
    package: &str,
    units: &[ProductionUnit],
    locks: &[serde_json::Value],
    inputs: &std::collections::BTreeMap<String, Vec<ConsumedInput>>,
    environment: &std::collections::BTreeMap<String, String>,
    native_abi: &Option<String>,
) -> std::io::Result<pse_ids::ContentHash> {
    let bytes = serde_json::to_vec(&(package, units, locks, inputs, environment, native_abi))
        .map_err(std::io::Error::other)?;
    let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::ProducerV1);
    hash.part(b"producer").part(&bytes);
    Ok(hash.finish_hash())
}
/// Recompute the current scientific payload with its original wire types and frame.
/// This establishes internal consistency; operator-reviewed completeness is separate.
pub fn verify_scientific_producer_identity(
    value: &serde_json::Value,
) -> std::io::Result<pse_ids::ContentHash> {
    if value
        .get("receipt_version")
        .and_then(serde_json::Value::as_u64)
        != Some(u64::from(DEPLOYMENT_RECEIPT_VERSION))
    {
        return Err(std::io::Error::other(
            "unsupported deployment receipt interpretation",
        ));
    }
    if value.get("frame").and_then(serde_json::Value::as_str)
        != Some(pse_ids::Frame::ProducerV1.as_str())
    {
        return Err(std::io::Error::other("unsupported producer interpretation"));
    }
    #[derive(serde::Deserialize)]
    struct Payload {
        package: String,
        units: Vec<ProductionUnit>,
        selected_lock_records: Vec<serde_json::Value>,
        consumed_inputs: std::collections::BTreeMap<String, Vec<ConsumedInput>>,
        declared_environment: std::collections::BTreeMap<String, String>,
        native_abi: Option<String>,
    }
    let payload: Payload = serde_json::from_value(value.clone()).map_err(std::io::Error::other)?;
    let actual = scientific_producer_identity(
        &payload.package,
        &payload.units,
        &payload.selected_lock_records,
        &payload.consumed_inputs,
        &payload.declared_environment,
        &payload.native_abi,
    )?;
    if value.get("identity").and_then(serde_json::Value::as_str) != Some(actual.to_hex().as_str()) {
        return Err(std::io::Error::other(
            "scientific producer identity differs from captured input payload",
        ));
    }
    Ok(actual)
}

/// Independently observed actual file bytes and topology at a consumed path.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct FileObservation {
    /// Exact consumed or deployed path.
    pub path: std::path::PathBuf,
    /// External SHA256 checksum of observed bytes.
    pub sha256: String,
    /// Actual resolved target, retaining path redirection.
    pub canonical: std::path::PathBuf,
    /// Observed platform file mode.
    pub mode: u32,
}
impl FileObservation {
    /// Observe bytes through one handle and refuse replacement while reading.
    pub fn capture(path: &Path) -> std::io::Result<Self> {
        Self::capture_scoped(path, &|| Ok(()))
    }
    /// Observe through one handle, checking the owning operation before each read
    /// chunk and before sealing. The checkpoint retains cancellation/deadline identity.
    pub fn capture_scoped(
        path: &Path,
        checkpoint: &dyn Fn() -> std::io::Result<()>,
    ) -> std::io::Result<Self> {
        checkpoint()?;
        MetadataBudget::default().path(path, 1)?;
        use sha2::{Digest, Sha256};
        use std::io::Read;
        let mut file = fs::File::open(path)?;
        let before = file.metadata()?;
        if !before.is_file() {
            return Err(std::io::Error::other(
                "artifact/input is not a regular file",
            ));
        }
        let mut hash = Sha256::new();
        let mut bytes = [0u8; 65_536];
        let mut extent = 0u64;
        loop {
            checkpoint()?;
            let count = file.read(&mut bytes)?;
            if count == 0 {
                break;
            }
            extent += count as u64;
            hash.update(&bytes[..count]);
        }
        checkpoint()?;
        let after = file.metadata()?;
        if extent != before.len()
            || before.len() != after.len()
            || before.modified()? != after.modified()?
        {
            return Err(std::io::Error::other(
                "artifact/input changed during observation",
            ));
        }
        #[cfg(unix)]
        let mode = {
            use std::os::unix::fs::MetadataExt;
            before.mode()
        };
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let current = fs::metadata(path)?;
            if (before.dev(), before.ino()) != (current.dev(), current.ino()) {
                return Err(std::io::Error::other("file replaced during observation"));
            }
        }
        #[cfg(not(unix))]
        let mode = u32::from(before.permissions().readonly());
        let canonical = path.canonicalize()?;
        MetadataBudget::default().path(&canonical, 1)?;
        Ok(Self {
            path: path.to_path_buf(),
            canonical,
            mode,
            sha256: hash
                .finalize()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
        })
    }
    /// Verify the current bytes, mode and resolved path.
    pub fn verify(&self) -> std::io::Result<()> {
        self.verify_scoped(&|| Ok(()))
    }
    /// Reobserve with the same operation checkpoint throughout the full capture.
    pub fn verify_scoped(
        &self,
        checkpoint: &dyn Fn() -> std::io::Result<()>,
    ) -> std::io::Result<()> {
        if Self::capture_scoped(&self.path, checkpoint)? != *self {
            return Err(std::io::Error::other(format!(
                "consumed file changed: {}",
                self.path.display()
            )));
        }
        Ok(())
    }
}

/// Post-build association to the actual selected role, never another scientific key.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeploymentAssociation {
    /// Scientific product key observed from the actual selected post-build capture.
    pub producer_identity: String,
    /// Actual unit-DAG production root associated with this emitted artifact.
    pub selected_root: String,
    /// Checkout that supplied the actual reviewed closure.
    pub workspace_root: std::path::PathBuf,
    /// Actual deployed role file, after admitted installation transformations.
    pub artifact: FileObservation,
    /// Original Cargo-emitted selected-root artifact.
    pub built_artifact: FileObservation,
    /// Exact current raw observations of consumed files.
    pub files: Vec<FileObservation>,
    /// Reviewed absence premises of the consumed closure.
    pub absent_files: Vec<std::path::PathBuf>,
    /// Complete consumed search namespace observations.
    pub namespaces: std::collections::BTreeMap<std::path::PathBuf, String>,
    /// Dynamic loader inputs; values are digested rather than published.
    pub native_environment: std::collections::BTreeMap<String, Option<String>>,
    /// Reviewed absent executable startup environment families.
    pub native_absent_environment_prefixes: Vec<String>,
}
impl DeploymentAssociation {
    /// Read current version before decoding its association shape. Historical keys
    /// and receipts are never upgraded by a decoder.
    pub fn decode(bytes: &[u8]) -> std::io::Result<Self> {
        let value: serde_json::Value =
            serde_json::from_slice(bytes).map_err(std::io::Error::other)?;
        if value
            .get("receipt_version")
            .and_then(serde_json::Value::as_u64)
            != Some(u64::from(DEPLOYMENT_RECEIPT_VERSION))
        {
            return Err(std::io::Error::other(
                "unsupported deployment receipt interpretation",
            ));
        }
        serde_json::from_value(
            value
                .get("deployment")
                .cloned()
                .ok_or_else(|| std::io::Error::other("deployment association absent"))?,
        )
        .map_err(std::io::Error::other)
    }
    /// Revalidate actual consumed inputs and artifact, without recapturing Cargo or
    /// turning incidental current compiler/tool inventory into a scientific key.
    pub fn verify_current(&self, observed: &Path) -> std::io::Result<FileObservation> {
        self.verify_current_scoped(observed, &|| Ok(()))
    }
    /// Revalidate a strict receipt under its original operation clock and cancellation.
    pub fn verify_current_scoped(
        &self,
        observed: &Path,
        checkpoint: &dyn Fn() -> std::io::Result<()>,
    ) -> std::io::Result<FileObservation> {
        checkpoint()?;
        self.artifact.verify_scoped(checkpoint)?;
        self.built_artifact.verify_scoped(checkpoint)?;
        let actual = FileObservation::capture_scoped(observed, checkpoint)?;
        if actual.sha256 != self.artifact.sha256 || actual.canonical != self.artifact.canonical {
            return Err(std::io::Error::other(
                "actual deployed artifact differs from selected capture",
            ));
        }
        for file in &self.files {
            file.verify_scoped(checkpoint)?;
        }
        for path in &self.absent_files {
            checkpoint()?;
            match fs::symlink_metadata(path) {
                Ok(_) => return Err(std::io::Error::other("reviewed absent input is present")),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
        for (path, expected) in &self.namespaces {
            if namespace_digest_scoped(path, checkpoint)? != *expected {
                return Err(std::io::Error::other("consumed search namespace changed"));
            }
        }
        for (name, expected) in &self.native_environment {
            checkpoint()?;
            let actual = std::env::var(name).ok();
            if actual.as_ref().map(|value| value_digest(value.as_bytes())) != *expected {
                return Err(std::io::Error::other(format!(
                    "consumed native environment changed: {name}"
                )));
            }
        }
        for prefix in &self.native_absent_environment_prefixes {
            checkpoint()?;
            if std::env::vars_os().any(|(name, _)| name.to_string_lossy().starts_with(prefix)) {
                return Err(std::io::Error::other(
                    "reviewed absent native environment family is present",
                ));
            }
        }
        checkpoint()?;
        Ok(actual)
    }
}

/// External SHA256 observation of bytes, separate from scientific identity.
pub fn value_digest(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Bounded raw namespace observation retains names, modes, link topology and file
/// bytes. Following each canonical directory once closes cycles without losing links.
pub fn namespace_digest(root: &Path) -> std::io::Result<String> {
    namespace_digest_scoped(root, &|| Ok(()))
}
/// Observe every namespace member with the original operation checkpoint.
pub fn namespace_digest_scoped(
    root: &Path,
    checkpoint: &dyn Fn() -> std::io::Result<()>,
) -> std::io::Result<String> {
    use std::collections::{BTreeMap, BTreeSet};
    let mut budget = MetadataBudget::default();
    budget.path(root, 2)?;
    let mut pending = vec![root.to_path_buf()];
    let mut visited = BTreeSet::new();
    let mut entries = BTreeMap::new();
    let mut extent = 0u64;
    #[cfg(unix)]
    let mut aliases = BTreeMap::<(u64, u64), Vec<std::path::PathBuf>>::new();
    while let Some(path) = pending.pop() {
        checkpoint()?;
        // Reserve all retained entry keys, JSON nodes and worst-case aliases before
        // constructing them. File/canonical path scratch is independently path-bounded.
        budget.path(&path, 8)?;
        budget.take(4096)?;
        if entries.len() + pending.len() >= 262_144 {
            return Err(std::io::Error::other("deployment namespace entry limit"));
        }
        let metadata = match fs::symlink_metadata(&path) {
            Ok(value) => value,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                entries.insert(
                    path.strip_prefix(root)
                        .map_err(std::io::Error::other)?
                        .to_path_buf(),
                    serde_json::json!({"absent":true}),
                );
                continue;
            }
            Err(error) => return Err(error),
        };
        let link = metadata
            .is_symlink()
            .then(|| fs::read_link(&path))
            .transpose()?;
        let target = match fs::metadata(&path) {
            Ok(value) => Some(value),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error),
        };
        #[cfg(unix)]
        let mode = {
            use std::os::unix::fs::MetadataExt;
            (metadata.mode(), metadata.uid(), metadata.gid())
        };
        #[cfg(not(unix))]
        let mode = (u32::from(metadata.permissions().readonly()), 0, 0);
        let content = if target.as_ref().is_some_and(fs::Metadata::is_file) {
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                if let Some(metadata) = &target {
                    aliases
                        .entry((metadata.dev(), metadata.ino()))
                        .or_default()
                        .push(path.clone());
                }
            }
            extent = extent
                .checked_add(target.as_ref().map_or(0, fs::Metadata::len))
                .ok_or_else(|| std::io::Error::other("namespace extent overflow"))?;
            if extent > 16 * 1024 * 1024 * 1024 {
                return Err(std::io::Error::other("deployment namespace content limit"));
            }
            Some(FileObservation::capture_scoped(&path, checkpoint)?)
        } else {
            None
        };
        let canonical = target.as_ref().map(|_| path.canonicalize()).transpose()?;
        if let Some(canonical) = &canonical {
            budget.path(canonical, 4)?;
        }
        if let Some(link) = &link {
            budget.path(link, 2)?;
        }
        if let Some(file) = &content {
            budget.path(&file.canonical, 4)?;
        }
        let directory = target.as_ref().is_some_and(fs::Metadata::is_dir);
        if directory
            && canonical
                .as_ref()
                .is_some_and(|value| visited.insert(value.clone()))
        {
            for entry in fs::read_dir(&path)? {
                checkpoint()?;
                if entries.len() + pending.len() >= 262_144 {
                    return Err(std::io::Error::other("deployment namespace entry limit"));
                }
                let entry = entry?;
                let name = entry.file_name();
                let length = path
                    .as_os_str()
                    .len()
                    .checked_add(name.len() + 1)
                    .ok_or_else(|| std::io::Error::other("namespace path extent overflow"))?;
                if length > PATH_BYTES {
                    return Err(std::io::Error::other("observation path byte limit"));
                }
                budget.take(2 * (length + 256))?;
                pending.push(path.join(name));
            }
        }
        if target.is_some() && content.is_none() && !directory {
            return Err(std::io::Error::other("unsupported namespace special file"));
        }
        entries.insert(path.strip_prefix(root).map_err(std::io::Error::other)?.to_path_buf(), serde_json::json!({"mode":mode,"link":link,"canonical":canonical,"directory":directory,"file":content}));
    }
    #[cfg(unix)]
    let mut groups = aliases
        .into_values()
        .filter(|paths| paths.len() > 1)
        .collect::<Vec<_>>();
    #[cfg(not(unix))]
    let mut groups = Vec::<Vec<std::path::PathBuf>>::new();
    for paths in &mut groups {
        paths.sort();
    }
    groups.sort();
    checkpoint()?;
    json_value_digest(&(entries, groups), checkpoint)
}

/// Complete checkout context during this explicit observation operation. A runtime
/// may retain it as startup context; it does not certify a later run or loaded build.
/// These hashes are neither a scientific producer nor compiled all-tree input.
#[derive(Clone, Debug, serde::Serialize)]
pub struct OuterObservation {
    /// Current dirty source inventory when its authored checkout is available.
    /// An installed artifact without that checkout explicitly has no source observation.
    pub source: Option<pse_ids::ContentHash>,
    /// Current configuration and commit context observation.
    pub build: pse_ids::ContentHash,
    /// Current contextual commit, or source distribution marker.
    pub git_sha: String,
}
/// Observe the complete current source/configuration inventory at admission.
pub fn observe_outer(root: &Path) -> std::io::Result<OuterObservation> {
    observe_outer_scoped(root, &|| Ok(()))
}
/// Stream complete dirty checkout bytes under the original stop/clock checkpoint.
/// Inventory ordering and length framing exactly match the unstreamed v2 digest.
pub fn observe_outer_scoped(
    root: &Path,
    checkpoint: &dyn Fn() -> std::io::Result<()>,
) -> std::io::Result<OuterObservation> {
    checkpoint()?;
    let paths = source_paths(root, checkpoint)?;
    let mut source = pse_ids::FramedHasher::new(pse_ids::Frame::BuildInputsV1);
    for (name, path) in &paths {
        checkpoint()?;
        source.str(name);
        stream_file(&mut source, path, checkpoint)?;
    }
    if source_paths(root, checkpoint)? != paths {
        return Err(std::io::Error::other(
            "outer source inventory changed during observation",
        ));
    }
    let source = source.finish_hash();
    let git_sha = git_context(root, checkpoint)?;
    let mut configuration = std::collections::BTreeMap::new();
    configuration.insert("interpretation", None);
    configuration.insert("git-context", None);
    for name in [
        "Cargo.toml",
        "Cargo.lock",
        "uv.lock",
        "pyproject.toml",
        "rust-toolchain.toml",
        ".cargo/config.toml",
        ".config/build.toml",
        ".config/native-cache.cmake",
        ".config/sccache.toml",
    ] {
        configuration.insert(name, Some(root.join(name)));
    }
    let mut build = pse_ids::FramedHasher::new(pse_ids::Frame::BuildInputsV1);
    for (name, path) in configuration {
        checkpoint()?;
        build.str(name);
        match path {
            Some(path) => {
                stream_file(&mut build, &path, checkpoint)?;
            }
            None if name == "interpretation" => {
                build.part(b"deployment-outer-observation.v2");
            }
            None => {
                build.part(git_sha.as_bytes());
            }
        }
    }
    checkpoint()?;
    Ok(OuterObservation {
        source: Some(source),
        build: build.finish_hash(),
        git_sha,
    })
}

fn git_context(
    root: &Path,
    checkpoint: &dyn Fn() -> std::io::Result<()>,
) -> std::io::Result<String> {
    checkpoint()?;
    let mut child = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "HEAD"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    loop {
        if let Err(error) = checkpoint() {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(10)),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error);
            }
        }
    }
    let output = child.wait_with_output()?;
    checkpoint()?;
    Ok(if output.status.success() {
        String::from_utf8_lossy(&output.stdout).trim().to_owned()
    } else {
        "sdist".into()
    })
}

fn stream_file(
    hash: &mut pse_ids::FramedHasher,
    path: &Path,
    checkpoint: &dyn Fn() -> std::io::Result<()>,
) -> std::io::Result<()> {
    checkpoint()?;
    let mut file = fs::File::open(path)?;
    let before = file.metadata()?;
    let canonical = path.canonicalize()?;
    if !before.is_file() || fs::symlink_metadata(path)?.is_symlink() {
        return Err(std::io::Error::other("unqualified outer source file"));
    }
    hash.part_reader(before.len(), &mut file, checkpoint)?;
    let after = file.metadata()?;
    let current = fs::symlink_metadata(path)?;
    if before.len() != after.len()
        || before.modified()? != after.modified()?
        || current.is_symlink()
        || canonical != path.canonicalize()?
    {
        return Err(std::io::Error::other(
            "outer file changed during observation",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let identity = |m: &fs::Metadata| (m.dev(), m.ino(), m.mode(), m.ctime(), m.ctime_nsec());
        if identity(&before) != identity(&after) || identity(&before) != identity(&current) {
            return Err(std::io::Error::other(
                "outer file replaced or changed during observation",
            ));
        }
    }
    checkpoint()
}
/// Locate the checkout containing the actual artifact without embedding its path.
pub fn workspace_root(path: &Path) -> std::io::Result<std::path::PathBuf> {
    path.ancestors()
        .find(|root| {
            root.join("Cargo.toml").is_file()
                && root.join("rust-toolchain.toml").is_file()
                && root.join("crates/pse-buildinfo/Cargo.toml").is_file()
                && root.join("crates/pse-runtime/Cargo.toml").is_file()
        })
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "complete outer workspace observation unavailable",
            )
        })
}

/// Observe a deployment without requiring installed artifacts to retain a source checkout.
/// Missing source evidence never grants a scientific producer qualification.
pub fn observe_deployment(artifact: &Path) -> std::io::Result<OuterObservation> {
    observe_deployment_scoped(artifact, &|| Ok(()))
}
/// Capture startup checkout context or explicit installed source-unavailable evidence
/// with cooperative chunk cancellation. No observation survives a failed checkpoint.
pub fn observe_deployment_scoped(
    artifact: &Path,
    checkpoint: &dyn Fn() -> std::io::Result<()>,
) -> std::io::Result<OuterObservation> {
    checkpoint()?;
    match workspace_root(artifact) {
        Ok(root) => observe_outer_scoped(&root, checkpoint),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let observed = FileObservation::capture_scoped(artifact, checkpoint)?;
            checkpoint()?;
            Ok(OuterObservation {
                source: None,
                build: digest(vec![
                    (
                        "interpretation".into(),
                        b"installed-artifact-outer-observation.v1".to_vec(),
                    ),
                    (
                        "actual-artifact".into(),
                        serde_json::to_vec(&observed).map_err(std::io::Error::other)?,
                    ),
                ]),
                git_sha: "source-unavailable".into(),
            })
        }
        Err(error) => Err(error),
    }
}

/// Verify that the imported path is the currently mapped Linux module, rather
/// than a replacement at the same path or a Python metadata-only association.
pub fn verify_loaded_module(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::MetadataExt;
        let path = path.canonicalize()?;
        let metadata = fs::metadata(&path)?;
        let device = metadata.dev();
        let major = ((device >> 8) & 0xfff) | ((device >> 32) & 0xfffff000);
        let minor = (device & 0xff) | ((device >> 12) & 0xffffff00);
        let maps = bounded_maps(&|| Ok(()))?;
        let mapped = maps.lines().any(|line| {
            let mut parts = line
                .splitn(6, char::is_whitespace)
                .filter(|part| !part.is_empty());
            let _address = parts.next();
            let _mode = parts.next();
            let _offset = parts.next();
            let device = parts.next();
            let inode = parts.next();
            let name = parts.next().map(str::trim);
            let expected_device = format!("{major:02x}:{minor:02x}");
            device == Some(expected_device.as_str())
                && inode.and_then(|value| value.parse::<u64>().ok()) == Some(metadata.ino())
                && name == path.to_str()
        });
        if !mapped {
            return Err(std::io::Error::other(
                "imported module is not the actual currently mapped artifact",
            ));
        }
        Ok(())
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = path;
        Err(std::io::Error::other(
            "loaded artifact association is qualified only on Linux",
        ))
    }
}

/// Resolve the mapping containing an actual code address supplied by the module's
/// composition root. Python metadata alone cannot select this artifact.
pub fn loaded_module_path(anchor: usize) -> std::io::Result<std::path::PathBuf> {
    #[cfg(target_os = "linux")]
    {
        let maps = bounded_maps(&|| Ok(()))?;
        for line in maps.lines() {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            let Some((start, end)) = parts.first().and_then(|range| range.split_once('-')) else {
                continue;
            };
            let (Ok(start), Ok(end)) = (
                usize::from_str_radix(start, 16),
                usize::from_str_radix(end, 16),
            ) else {
                continue;
            };
            if start <= anchor
                && anchor < end
                && parts.get(1).is_some_and(|mode| mode.contains('x'))
            {
                if parts.len() != 6 || parts[5].starts_with('[') {
                    return Err(std::io::Error::other(
                        "loaded code has no unambiguous live file mapping",
                    ));
                }
                let path = std::path::PathBuf::from(parts[5]);
                verify_loaded_module(&path)?;
                return Ok(path);
            }
        }
        Err(std::io::Error::other(
            "native composition root is not mapped",
        ))
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = anchor;
        Err(std::io::Error::other(
            "loaded artifact association is qualified only on Linux",
        ))
    }
}

/// Interpretation of exact process-local replay observations; separate from receipts.
pub const LOCAL_RUNTIME_VERSION: u32 = 1;

/// Actual composition role, retained separately from the observed artifact bytes.
#[derive(Clone, Copy, Debug, serde::Serialize, PartialEq, Eq)]
pub enum LocalRuntimeRole {
    /// Native supervised worker: the anchor must belong to the running executable.
    Worker,
    /// Imported native extension, with the actual running interpreter observed as well.
    Python,
}

/// Independently observed Linux executable/native context. This is observation, not
/// eligibility: a stable-use owner must protect the interval through reconstruction.
/// It cannot be deserialized or assembled from claimed receipt filenames.
#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
pub struct LocalRuntimeObservation {
    version: u32,
    role: LocalRuntimeRole,
    kernel_release: Vec<u8>,
    kernel_version: Vec<u8>,
    artifact: FileObservation,
    executable: FileObservation,
    loaded: Vec<FileObservation>,
    loader_files: Vec<(std::path::PathBuf, Option<FileObservation>)>,
    startup_environment: std::collections::BTreeMap<String, String>,
    current_environment: std::collections::BTreeMap<String, String>,
}
impl LocalRuntimeObservation {
    /// Owned observation metadata; file contents are streamed rather than retained.
    pub fn retained_bytes(&self) -> usize {
        fn file(value: &FileObservation) -> usize {
            size_of::<FileObservation>()
                + value.path.capacity()
                + value.canonical.capacity()
                + value.sha256.capacity()
                + 128
        }
        size_of::<Self>()
            + 256
            + self.kernel_release.capacity()
            + self.kernel_version.capacity()
            + file(&self.artifact)
            + file(&self.executable)
            + self.loaded.capacity() * size_of::<FileObservation>()
            + self.loaded.iter().map(file).sum::<usize>()
            + self.loader_files.capacity()
                * size_of::<(std::path::PathBuf, Option<FileObservation>)>()
            + self
                .loader_files
                .iter()
                .map(|(path, value)| path.capacity() + value.as_ref().map_or(0, file) + 128)
                .sum::<usize>()
            + self
                .startup_environment
                .iter()
                .chain(&self.current_environment)
                .map(|(key, value)| key.capacity() + value.capacity() + 128)
                .sum::<usize>()
    }
    /// Observe the actual anchor, executable, every executable-backed ELF and effective
    /// loader/runtime environment. Addresses/inodes establish association but do not enter
    /// restart identity. Unknown executable mappings refuse this narrower guarantee.
    /// # Safety
    /// The actual controlled composition root supplied an independently captured
    /// owned environment snapshot outside the loader scope and retains its native
    /// execution support contract. Capture itself never borrows libc environment.
    #[allow(
        unsafe_code,
        reason = "controlled root forwards its independently owned observation premises"
    )]
    pub unsafe fn capture(
        role: LocalRuntimeRole,
        anchor: usize,
        environment: &CurrentEnvironmentSnapshot,
    ) -> std::io::Result<Self> {
        // SAFETY: forwarded unchanged from this actual controlled root.
        unsafe { Self::capture_scoped(role, anchor, environment, &|| Ok(())) }
    }
    /// Capture the receiving boundary under its original stop/clock checkpoint.
    /// Foreign loader-lock acquisition itself remains outside cooperative interruption.
    /// # Safety
    /// The same controlled-root observation premises as [`Self::capture`] apply.
    #[allow(
        unsafe_code,
        reason = "ADR-0164 controlled root supplies independently owned observation premises"
    )]
    pub unsafe fn capture_scoped(
        role: LocalRuntimeRole,
        anchor: usize,
        environment: &CurrentEnvironmentSnapshot,
        checkpoint: &dyn Fn() -> std::io::Result<()>,
    ) -> std::io::Result<Self> {
        checkpoint()?;
        #[cfg(target_os = "linux")]
        {
            let before = executable_mapping_files(&bounded_maps(checkpoint)?)?;
            // Several authority paths normally name the same running ELF: the
            // caller's anchor, current_exe, and its executable mapping. Observe
            // each exact path once, then retain the independent end-of-capture
            // verification below. Hashing the same large worker binary for each
            // alias needlessly lengthens the loader exclusion held by the caller.
            let mut observations =
                std::collections::BTreeMap::<std::path::PathBuf, FileObservation>::new();
            let mut budget = MetadataBudget::default();
            let mut observe = |path: &Path| -> std::io::Result<FileObservation> {
                if let Some(observation) = observations.get(path) {
                    budget.path(path, 2)?;
                    budget.path(&observation.canonical, 2)?;
                    return Ok(observation.clone());
                }
                budget.path(path, 4)?;
                budget.take(512)?;
                let observation = FileObservation::capture_scoped(path, checkpoint)?;
                budget.path(&observation.canonical, 4)?;
                observations.insert(path.to_path_buf(), observation.clone());
                checkpoint()?;
                Ok(observation)
            };
            let artifact = observe(&loaded_module_path(anchor)?)?;
            let executable = observe(&std::env::current_exe()?)?;
            verify_loaded_module(&executable.canonical)?;
            if role == LocalRuntimeRole::Worker && artifact.canonical != executable.canonical {
                return Err(std::io::Error::other(
                    "worker anchor belongs to another mapped module",
                ));
            }
            let loaded = before
                .iter()
                .map(|path| {
                    checkpoint()?;
                    use std::io::Read;
                    let mut magic = [0; 4];
                    fs::File::open(path)?.read_exact(&mut magic)?;
                    if magic != *b"\x7fELF" {
                        return Err(std::io::Error::other(
                            "executable mapping is not a supported ELF artifact",
                        ));
                    }
                    verify_loaded_module(path)?;
                    observe(path)
                })
                .collect::<std::io::Result<Vec<_>>>()?;
            drop(observe);
            let mut loader_files = Vec::new();
            for path in ["/etc/ld.so.cache", "/etc/ld.so.preload"] {
                checkpoint()?;
                let path = std::path::PathBuf::from(path);
                let observation = match fs::symlink_metadata(&path) {
                    Ok(_) => Some(FileObservation::capture_scoped(&path, checkpoint)?),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                    Err(error) => return Err(error),
                };
                loader_files.push((path, observation));
            }
            let observation = Self {
                version: LOCAL_RUNTIME_VERSION,
                role,
                // Kernel supplies vDSO/vsyscall executable context. These are actual
                // process-visible kernel facts, not caller/platform receipt markers.
                kernel_release: bounded_read(
                    Path::new("/proc/sys/kernel/osrelease"),
                    4096,
                    checkpoint,
                )?,
                kernel_version: bounded_read(
                    Path::new("/proc/sys/kernel/version"),
                    4096,
                    checkpoint,
                )?,
                artifact,
                executable,
                loaded,
                loader_files,
                startup_environment: runtime_environment(
                    bounded_read(
                        Path::new("/proc/self/environ"),
                        ENVIRONMENT_BYTES,
                        checkpoint,
                    )?
                    .split(|byte| *byte == 0)
                    .filter(|entry| !entry.is_empty()),
                )?,
                // Owned snapshot captured by the actual root outside loader scope.
                // Python ordinary environment mutation cannot invalidate its memory.
                current_environment: environment.values.clone(),
            };
            checkpoint()?;
            if before != executable_mapping_files(&bounded_maps(checkpoint)?)? {
                return Err(std::io::Error::other(
                    "loaded executable context changed during observation",
                ));
            }
            let mut verified_paths = std::collections::BTreeSet::new();
            for file in &observation.loaded {
                if verified_paths.insert(&file.path) {
                    file.verify_scoped(checkpoint)?;
                }
            }
            checkpoint()?;
            Ok(observation)
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (role, anchor);
            Err(std::io::Error::other(
                "local replay observations are supported only on Linux",
            ))
        }
    }
    /// Reobserve the complete context. This closes changed files/configuration and
    /// persistent late loading, but does not by itself exclude transient late loading.
    /// # Safety
    /// The actual root supplies a newly observed owned environment snapshot and
    /// retains the controlled native execution support contract.
    #[allow(
        unsafe_code,
        reason = "verification retains the actual controlled-root observation premises"
    )]
    pub unsafe fn verify(
        &self,
        anchor: usize,
        environment: &CurrentEnvironmentSnapshot,
    ) -> std::io::Result<()> {
        // SAFETY: the caller supplies the same environment lifetime.
        unsafe { self.verify_scoped(anchor, environment, &|| Ok(())) }
    }
    /// Verify receiving inputs without resetting the enclosing operation clock.
    /// # Safety
    /// The same owned-observation premises as [`Self::verify`] are required.
    #[allow(
        unsafe_code,
        reason = "full verification consumes the actual controlled-root snapshot"
    )]
    pub unsafe fn verify_scoped(
        &self,
        anchor: usize,
        environment: &CurrentEnvironmentSnapshot,
        checkpoint: &dyn Fn() -> std::io::Result<()>,
    ) -> std::io::Result<()> {
        // SAFETY: the verification caller retains the controlled root through capture.
        if unsafe { Self::capture_scoped(self.role, anchor, environment, checkpoint)? } != *self {
            return Err(std::io::Error::other("local runtime observation changed"));
        }
        Ok(())
    }
    /// Exact versioned context identity, without a checkout or compiler-source capture.
    pub fn identity(&self) -> std::io::Result<pse_ids::ContentHash> {
        let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::BuildInputsV1);
        hash.str("local-runtime-observation.v1");
        framed_json(&mut hash, self, &|| Ok(()))?;
        Ok(hash.finish_hash())
    }
}

// A finite supported runtime configuration boundary, not arbitrary build environment.
// Startup and current values both matter: glibc can consume a startup setting which a
// Python host later removes from os.environ. Digests keep secret values out of records.
fn runtime_environment<E: AsRef<[u8]>>(
    entries: impl Iterator<Item = E>,
) -> std::io::Result<std::collections::BTreeMap<String, String>> {
    let mut result = std::collections::BTreeMap::new();
    let mut raw = 0usize;
    let mut retained = 0usize;
    for entry in entries {
        insert_environment(&mut result, entry.as_ref(), &mut raw, &mut retained)?;
    }
    Ok(result)
}
fn insert_environment(
    result: &mut std::collections::BTreeMap<String, String>,
    entry: &[u8],
    raw: &mut usize,
    retained: &mut usize,
) -> std::io::Result<()> {
    *raw = raw
        .checked_add(entry.len() + 1)
        .ok_or_else(|| std::io::Error::other("environment extent overflow"))?;
    if *raw > ENVIRONMENT_BYTES {
        return Err(std::io::Error::other("runtime environment byte limit"));
    }
    let Some(equal) = entry.iter().position(|byte| *byte == b'=') else {
        return Err(std::io::Error::other("malformed runtime environment"));
    };
    let name = std::str::from_utf8(&entry[..equal]).map_err(std::io::Error::other)?;
    if [
        "LD_",
        "GLIBC_",
        "OMP_",
        "MKL_",
        "KMP_",
        "TBB_",
        "SYMBOLICA_",
        "NUMERICA_",
        "PYTHON",
    ]
    .iter()
    .any(|prefix| name.starts_with(prefix))
        || matches!(name, "LANG" | "LC_ALL" | "LC_NUMERIC" | "TZ")
    {
        let next = retained
            .checked_add(name.len() + 512)
            .ok_or_else(|| std::io::Error::other("environment metadata overflow"))?;
        if next > ENVIRONMENT_BYTES {
            return Err(std::io::Error::other(
                "runtime environment metadata byte limit",
            ));
        }
        *retained = next;
        if result
            .insert(name.to_owned(), value_digest(&entry[equal + 1..]))
            .is_some()
        {
            return Err(std::io::Error::other("duplicate runtime environment input"));
        }
    }
    Ok(())
}

/// Owned exact supported current-environment observation, captured by its actual
/// exclusion owner before any loader scope. It is evidence, never replay permission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurrentEnvironmentSnapshot {
    values: std::collections::BTreeMap<String, String>,
}
impl CurrentEnvironmentSnapshot {
    /// # Safety
    /// Exclude all libc environment mutation for this complete bounded walk.
    /// A worker uses its actual no-writers lifetime. An interpreter owner must
    /// exclude writers in every interpreter domain as well as foreign native
    /// writers. Holding one conventional GIL or attaching to a free-threaded
    /// interpreter does not establish this process-wide exclusion.
    #[allow(
        unsafe_code,
        reason = "actual root supplies a genuine environment exclusion before borrowing libc"
    )]
    pub unsafe fn capture_scoped(
        checkpoint: &dyn Fn() -> std::io::Result<()>,
    ) -> std::io::Result<Self> {
        #[cfg(target_os = "linux")]
        {
            Ok(Self {
                // SAFETY: this method's caller excludes every libc environment
                // writer for the complete bounded traversal.
                values: unsafe { current_environment(checkpoint)? },
            })
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = checkpoint;
            Err(std::io::Error::other(
                "current environment observation requires the supported Linux owner",
            ))
        }
    }
    /// Retained metadata upper bound; construction enforces this before growth.
    pub fn retained_bytes(&self) -> usize {
        self.values
            .iter()
            .map(|(name, value)| name.capacity() + value.capacity() + 512)
            .sum()
    }
}

#[cfg(target_os = "linux")]
#[allow(
    unsafe_code,
    reason = "controlled root excludes every libc environment mutation during bounded borrowed traversal"
)]
unsafe fn current_environment(
    checkpoint: &dyn Fn() -> std::io::Result<()>,
) -> std::io::Result<std::collections::BTreeMap<String, String>> {
    unsafe extern "C" {
        static mut environ: *mut *mut libc::c_char;
    }
    let mut result = std::collections::BTreeMap::new();
    let mut raw = 0usize;
    let mut retained = 0usize;
    // SAFETY: the actual root excludes mutation of the libc vector, its entries
    // and their terminating bytes for this entire operation. The vector is
    // NUL-terminated by libc; each non-null member is a valid C string.
    let base = unsafe { environ };
    if base.is_null() {
        return Ok(result);
    }
    for index in 0..ENVIRONMENT_BYTES {
        checkpoint()?;
        // SAFETY: preceding non-null entries establish that this is still within
        // libc's stable NUL-terminated vector; the count is independently bounded.
        let slot = unsafe { base.add(index) };
        // SAFETY: the slot above is within the stable libc environment vector.
        let entry = unsafe { *slot };
        if entry.is_null() {
            return Ok(result);
        }
        let remaining = ENVIRONMENT_BYTES - raw;
        let mut length = 0usize;
        loop {
            if length >= remaining {
                return Err(std::io::Error::other("runtime environment byte limit"));
            }
            if length.is_multiple_of(4096) {
                checkpoint()?;
            }
            // SAFETY: the root lends the stable valid C string through its NUL.
            let byte = unsafe { entry.add(length) };
            // SAFETY: the byte above is within the borrowed C string through its NUL.
            if unsafe { *byte } == 0 {
                break;
            }
            length += 1;
        }
        // SAFETY: the bounded scan established every byte before the terminator.
        let bytes = unsafe { std::slice::from_raw_parts(entry.cast::<u8>(), length) };
        insert_environment(&mut result, bytes, &mut raw, &mut retained)?;
    }
    Err(std::io::Error::other("runtime environment entry limit"))
}

fn executable_mapping_files(
    maps: &str,
) -> std::io::Result<std::collections::BTreeSet<std::path::PathBuf>> {
    if maps.len() > MAPPING_BYTES {
        return Err(std::io::Error::other("mapping observation byte limit"));
    }
    let mut files = std::collections::BTreeSet::new();
    let mut budget = MetadataBudget::default();
    for line in maps.lines() {
        let mut fields = line.split_whitespace();
        let parts = std::array::from_fn::<_, 6, _>(|_| fields.next());
        if parts[..5].iter().any(Option::is_none) {
            return Err(std::io::Error::other("malformed Linux mapping"));
        }
        if !parts[1].is_some_and(|part| part.contains('x')) {
            continue;
        }
        // vDSO/vsyscall are kernel-supplied code, governed by the platform support
        // boundary. They have no file-backed artifact to hash and are not JIT code.
        if fields.clone().next().is_none() && matches!(parts[5], Some("[vdso]" | "[vsyscall]")) {
            continue;
        }
        if parts[1].is_some_and(|part| part.contains('w'))
            || fields.next().is_some()
            || !parts[5].is_some_and(|part| part.starts_with('/'))
            || parts[4] == Some("0")
        {
            return Err(std::io::Error::other(
                "unknown, writable, deleted or ambiguous executable mapping",
            ));
        }
        let path =
            Path::new(parts[5].ok_or_else(|| std::io::Error::other("missing executable path"))?);
        budget.path(path, 2)?;
        files.insert(path.to_path_buf());
    }
    if files.is_empty() {
        return Err(std::io::Error::other("no supported loaded ELF context"));
    }
    Ok(files)
}

/// Execute bounded synchronous observation/reconstruction while the supported glibc
/// loader holds its loaded-object write lock. No guard escapes into cached bodies.
/// Panics are caught inside the C callback and resumed after glibc releases its lock.
/// Reentrant same-thread loading is refused using monotonic loader counters.
///
/// The supported controlled root excludes executable mmap/JIT, opaque plugins and
/// concurrent runtime configuration changes. The operation must not import/load
/// modules, invoke provider/plugin callbacks, spawn threads, or wait for work which
/// can acquire a loader lock. Additional effective Python configuration is observed
/// before/after this scope, so the GIL is never acquired under the loader lock.
/// External/privileged file mutation is
/// outside the existing generation-use contract; file/mapping checks detect ordinary
/// stale deployments. This function itself does not mint scientific eligibility.
pub fn with_local_runtime_scope<T>(
    operation: impl FnOnce() -> std::io::Result<T>,
) -> std::io::Result<T> {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    {
        linux_loader_scope::run(operation)
    }
    #[cfg(not(all(target_os = "linux", target_env = "gnu")))]
    {
        let _ = operation;
        Err(std::io::Error::other(
            "local replay requires the supported Linux glibc loader",
        ))
    }
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
use crate::loader_scope as linux_loader_scope;

/// Deployment owners observe their artifact first, and revalidate reviewed
/// consumed inputs whenever a receipt claims scientific eligibility.
pub fn verify_receipt_artifact(bytes: &[u8], path: &Path) -> std::io::Result<FileObservation> {
    verify_receipt_artifact_scoped(bytes, path, &|| Ok(()))
}
/// Verify strict source/artifact association with cooperative full-content checks.
pub fn verify_receipt_artifact_scoped(
    bytes: &[u8],
    path: &Path,
    checkpoint: &dyn Fn() -> std::io::Result<()>,
) -> std::io::Result<FileObservation> {
    checkpoint()?;
    let header: serde_json::Value = serde_json::from_slice(bytes).map_err(std::io::Error::other)?;
    if header
        .get("receipt_version")
        .and_then(serde_json::Value::as_u64)
        != Some(u64::from(DEPLOYMENT_RECEIPT_VERSION))
    {
        return Err(std::io::Error::other(
            "unsupported deployment receipt interpretation",
        ));
    }
    if header
        .get("persistent_reuse_eligible")
        .and_then(serde_json::Value::as_bool)
        == Some(false)
    {
        return FileObservation::capture_scoped(path, checkpoint);
    }
    let scientific = verify_scientific_producer_identity(&header)?;
    let association = DeploymentAssociation::decode(bytes)?;
    if header
        .get("selected_root")
        .and_then(serde_json::Value::as_str)
        != Some(association.selected_root.as_str())
    {
        return Err(std::io::Error::other(
            "actual artifact association names another selected root",
        ));
    }
    if association.producer_identity != scientific.to_hex() {
        return Err(std::io::Error::other(
            "actual artifact association names another scientific producer",
        ));
    }
    checkpoint()?;
    association.verify_current_scoped(path, checkpoint)
}

/// Complete outer product sources, including the executable worker composition root.
/// Relevant scientific producer inputs are selected separately by their actual unit DAG.
#[cfg(test)]
pub(crate) fn source_files(root: &Path) -> std::io::Result<Vec<(String, Vec<u8>)>> {
    source_paths(root, &|| Ok(()))?
        .into_iter()
        .map(|(name, path)| Ok((name, fs::read(path)?)))
        .collect()
}

fn source_paths(
    root: &Path,
    checkpoint: &dyn Fn() -> std::io::Result<()>,
) -> std::io::Result<Vec<(String, std::path::PathBuf)>> {
    let mut files = Vec::new();
    let mut budget = MetadataBudget {
        used: 0,
        limit: SOURCE_METADATA_BYTES,
    };
    budget.path(root, 1)?;
    for name in ["crates", "vendor", "xtask", "python", "scripts"] {
        let directory = root.join(name);
        // Retired source overrides leave no vendor directory in a fresh checkout.
        // Observe its reappearance so a future actual override still rekeys the build.
        if matches!(name, "vendor" | "python" | "scripts") && !directory.try_exists()? {
            continue;
        }
        checkpoint()?;
        collect_sources(root, &directory, &mut files, &mut budget, 0, checkpoint)?;
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(files)
}

fn collect_sources(
    root: &Path,
    directory: &Path,
    files: &mut Vec<(String, std::path::PathBuf)>,
    budget: &mut MetadataBudget,
    depth: usize,
    checkpoint: &dyn Fn() -> std::io::Result<()>,
) -> std::io::Result<()> {
    if depth >= SOURCE_DEPTH {
        return Err(std::io::Error::other(
            "source inventory traversal depth limit",
        ));
    }
    for entry in fs::read_dir(directory)? {
        checkpoint()?;
        let entry = entry?;
        let name = entry.file_name();
        if directory
            .as_os_str()
            .len()
            .checked_add(name.len() + 1)
            .is_none_or(|length| length > PATH_BYTES)
        {
            return Err(std::io::Error::other("observation path byte limit"));
        }
        let path = directory.join(name);
        let kind = entry.file_type()?;
        if kind.is_dir() {
            if !matches!(
                entry.file_name().to_str(),
                Some("target" | "__pycache__" | ".git" | ".pytest_cache" | ".ruff_cache")
            ) {
                collect_sources(root, &path, files, budget, depth + 1, checkpoint)?;
            }
        } else if kind.is_file() {
            // Installed native extension bytes are bound as actual role artifacts;
            // they are outputs, not authored dirty Python source.
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("_native") && name.ends_with(".so"))
            {
                continue;
            }
            let relative_path = path.strip_prefix(root).map_err(std::io::Error::other)?;
            // Lossy UTF-8 replacement can expand each raw byte threefold. Include
            // that temporary and the replaced name plus vector spare capacity.
            budget.take(
                6 * relative_path.as_os_str().len()
                    + path.as_os_str().len()
                    + 4 * size_of::<(String, std::path::PathBuf)>()
                    + 128,
            )?;
            let relative = relative_path.to_string_lossy().replace('\\', "/");
            files.push((relative, path));
        } else if kind.is_symlink() {
            return Err(std::io::Error::other(format!(
                "unqualified source symlink {}",
                path.display()
            )));
        }
    }
    Ok(())
}

pub(crate) fn digest(mut entries: Vec<(String, Vec<u8>)>) -> pse_ids::ContentHash {
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::BuildInputsV1);
    for (name, bytes) in entries {
        hash.str(&name).part(&bytes);
    }
    hash.finish_hash()
}
#[cfg(test)]
mod foundation_unit {
    use super::fs;
    #[test]
    fn streamed_observation_json_preserves_exact_bytes_and_original_stop_kind() {
        let value = serde_json::json!({"paths":["a\\b", "α/β", "\u{0000}\n\""],"absent":null,"mode":[1,2,3]});
        let encoded = serde_json::to_vec(&value).unwrap();
        assert_eq!(
            super::json_value_digest(&value, &|| Ok(())).unwrap(),
            super::value_digest(&encoded)
        );
        let mut actual = pse_ids::FramedHasher::new(pse_ids::Frame::BuildInputsV1);
        actual.str("local-runtime-observation.v1");
        let mut expected = actual.clone();
        expected.part(&encoded);
        super::framed_json(&mut actual, &value, &|| Ok(())).unwrap();
        assert_eq!(actual.finish_hash(), expected.finish_hash());
        for kind in [
            std::io::ErrorKind::Interrupted,
            std::io::ErrorKind::TimedOut,
        ] {
            let stop = || Err(std::io::Error::new(kind, "original observation stop"));
            assert_eq!(
                super::json_value_digest(&value, &stop).unwrap_err().kind(),
                kind
            );
            let mut framed = pse_ids::FramedHasher::new(pse_ids::Frame::BuildInputsV1);
            let original = framed.clone().finish_hash();
            assert_eq!(
                super::framed_json(&mut framed, &value, &stop)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(framed.finish_hash(), original);
        }
    }
    #[test]
    fn scientific_process_mapping_extent_preserves_code_checks_and_finite_limit() {
        // Many non-executable segments must not hide the actual extension, even
        // when their aggregate exceeds the small kernel-input buffer allowance.
        let segment = "1000-2000 r--p 00000000 08:01 42 /lib/scientific.so\n";
        let mut maps = segment.repeat((192 << 10) / segment.len());
        assert!(maps.len() > 128 << 10);
        maps.push_str("2000-3000 r-xp 00000000 08:01 43 /lib/actual.so\n");
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("maps");
        fs::write(&path, &maps).unwrap();
        let read = super::bounded_read(&path, super::MAPPING_BYTES, &|| Ok(())).unwrap();
        assert_eq!(read, maps.as_bytes());
        let files = super::executable_mapping_files(std::str::from_utf8(&read).unwrap())
            .unwrap();
        assert_eq!(files, [std::path::PathBuf::from("/lib/actual.so")].into());

        maps.push_str("3000-4000 rwxp 00000000 08:01 44 /lib/writable.so\n");
        assert!(super::executable_mapping_files(&maps).is_err());
        maps = segment.repeat(super::MAPPING_BYTES / segment.len() + 1);
        assert!(super::executable_mapping_files(&maps).is_err());
        fs::write(&path, maps).unwrap();
        assert!(super::bounded_read(&path, super::MAPPING_BYTES, &|| Ok(())).is_err());
    }
    #[test]
    fn observation_metadata_and_input_limits_refuse_before_retained_growth() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(directory.path().join("input"), [7; 65]).unwrap();
        assert_eq!(
            super::bounded_read(&directory.path().join("input"), 65, &|| Ok(())).unwrap(),
            [7; 65]
        );
        assert!(super::bounded_read(&directory.path().join("input"), 64, &|| Ok(())).is_err());
        let mut budget = super::MetadataBudget {
            used: 1023,
            limit: 1024,
        };
        let mut files = Vec::new();
        assert!(
            super::collect_sources(
                directory.path(),
                directory.path(),
                &mut files,
                &mut budget,
                0,
                &|| Ok(())
            )
            .is_err()
        );
        assert!(files.is_empty());
        assert_eq!(budget.used, 1023);
        assert!(
            super::collect_sources(
                directory.path(),
                directory.path(),
                &mut files,
                &mut super::MetadataBudget::default(),
                super::SOURCE_DEPTH,
                &|| Ok(())
            )
            .is_err()
        );
        let oversized = format!("LD_LONG={}", "x".repeat(super::ENVIRONMENT_BYTES));
        assert!(super::runtime_environment([oversized.as_bytes()].into_iter()).is_err());
        let entries = (0..256).map(|n| format!("LD_{n}=x"));
        assert!(super::runtime_environment(entries).is_err());
    }
    fn outer_fixture() -> tempfile::TempDir {
        let directory = tempfile::tempdir().unwrap();
        for name in ["crates", "xtask", ".cargo", ".config"] {
            fs::create_dir(directory.path().join(name)).unwrap();
        }
        for name in [
            "Cargo.toml",
            "Cargo.lock",
            "uv.lock",
            "pyproject.toml",
            "rust-toolchain.toml",
            ".cargo/config.toml",
            ".config/build.toml",
            ".config/native-cache.cmake",
            ".config/sccache.toml",
        ] {
            fs::write(directory.path().join(name), name.as_bytes()).unwrap();
        }
        directory
    }

    #[test]
    fn outer_streaming_preserves_dirty_source_and_configuration_frames() {
        let directory = outer_fixture();
        let root = directory.path();
        fs::write(root.join("crates/large.rs"), vec![37; 3 * 65_536 + 1]).unwrap();
        fs::write(root.join("xtask/worker.rs"), b"worker").unwrap();
        let observed = super::observe_outer_scoped(root, &|| Ok(())).unwrap();
        assert_eq!(
            observed.source,
            Some(super::digest(super::source_files(root).unwrap()))
        );
        let mut configuration = vec![
            (
                "interpretation".into(),
                b"deployment-outer-observation.v2".to_vec(),
            ),
            ("git-context".into(), observed.git_sha.as_bytes().to_vec()),
        ];
        for name in [
            "Cargo.toml",
            "Cargo.lock",
            "uv.lock",
            "pyproject.toml",
            "rust-toolchain.toml",
            ".cargo/config.toml",
            ".config/build.toml",
            ".config/native-cache.cmake",
            ".config/sccache.toml",
        ] {
            configuration.push((name.into(), fs::read(root.join(name)).unwrap()));
        }
        assert_eq!(observed.build, super::digest(configuration));
        // Same-inode and same-length edits must still consume and distinguish bytes.
        fs::write(root.join("crates/large.rs"), vec![38; 3 * 65_536 + 1]).unwrap();
        assert_ne!(observed.source, super::observe_outer(root).unwrap().source);
        // Retained startup context remains its original observation.
        assert_eq!(observed.git_sha, "sdist");
    }

    #[test]
    fn outer_observation_retains_complete_large_native_supplier_inventory() {
        let directory = outer_fixture();
        let root = directory.path();
        let native = root.join("vendor/supplier/native/include");
        fs::create_dir_all(&native).unwrap();
        for n in 0..8192 {
            fs::write(
                native.join(format!("complete-source-input-{n:05}.h")),
                n.to_string(),
            )
            .unwrap();
        }
        let paths = super::source_paths(root, &|| Ok(())).unwrap();
        assert_eq!(paths.len(), 8192);
        assert!(paths.windows(2).all(|pair| pair[0].0 < pair[1].0));
        let observed = super::observe_outer_scoped(root, &|| Ok(())).unwrap();
        assert_eq!(
            observed.source,
            Some(super::digest(super::source_files(root).unwrap()))
        );
        fs::write(
            native.join("complete-source-input-08191.h"),
            b"changed final input",
        )
        .unwrap();
        assert_ne!(
            observed.source,
            super::observe_outer_scoped(root, &|| Ok(()))
                .unwrap()
                .source
        );
        let cancelled = || {
            Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "original stop",
            ))
        };
        assert_eq!(
            super::observe_outer_scoped(root, &cancelled)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::Interrupted
        );
    }

    #[test]
    fn outer_streaming_cancellation_and_original_deadline_never_seal() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let directory = outer_fixture();
        let path = directory.path().join("crates/large.rs");
        fs::write(&path, vec![29; 4 * 65_536]).unwrap();
        let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::BuildInputsV1);
        let polls = AtomicUsize::new(0);
        let error = super::stream_file(&mut hash, &path, &|| {
            if polls.fetch_add(1, Ordering::Relaxed) == 3 {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "original stop",
                ))
            } else {
                Ok(())
            }
        })
        .unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::Interrupted);
        assert_eq!(polls.load(Ordering::Relaxed), 4);
        let expired = || {
            Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "original clock",
            ))
        };
        let error =
            super::observe_deployment_scoped(std::path::Path::new("missing-artifact"), &expired)
                .unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
        assert!(super::observe_outer_scoped(directory.path(), &|| Ok(())).is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn outer_streaming_refuses_link_and_mode_changes_during_capture() {
        use std::{
            os::unix::fs::{PermissionsExt, symlink},
            sync::atomic::{AtomicUsize, Ordering},
        };
        let directory = outer_fixture();
        let path = directory.path().join("crates/source.rs");
        fs::write(&path, vec![29; 2 * 65_536]).unwrap();
        let mode = fs::metadata(&path).unwrap().permissions().mode();
        let polls = AtomicUsize::new(0);
        let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::BuildInputsV1);
        assert!(
            super::stream_file(&mut hash, &path, &|| {
                if polls.fetch_add(1, Ordering::Relaxed) == 3 {
                    fs::set_permissions(&path, fs::Permissions::from_mode(mode ^ 0o100))?;
                }
                Ok(())
            })
            .is_err()
        );
        let alias = directory.path().join("crates/alias.rs");
        symlink(&path, alias).unwrap();
        assert!(
            super::observe_outer_scoped(directory.path(), &|| Ok(()))
                .unwrap_err()
                .to_string()
                .contains("symlink")
        );
    }
    #[test]
    fn file_observation_cancellation_checks_each_read_chunk_and_never_seals() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("receiving-input");
        fs::write(&path, vec![37; 4 * 65_536]).unwrap();
        let polls = AtomicUsize::new(0);
        let error = super::FileObservation::capture_scoped(&path, &|| {
            if polls.fetch_add(1, Ordering::Relaxed) == 3 {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "stopped",
                ))
            } else {
                Ok(())
            }
        })
        .unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::Interrupted);
        assert_eq!(polls.load(Ordering::Relaxed), 4);
        let complete = super::FileObservation::capture_scoped(&path, &|| Ok(())).unwrap();
        assert_eq!(complete, super::FileObservation::capture(&path).unwrap());
    }
    #[test]
    fn receiving_observation_preserves_expired_operation_clock_before_io() {
        let expired = || {
            Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "original deadline",
            ))
        };
        #[allow(
            unsafe_code,
            reason = "expired operation control never reaches borrowed environment"
        )]
        // SAFETY: the original clock refuses before any borrowed environment read.
        let error = unsafe {
            super::LocalRuntimeObservation::capture_scoped(
                super::LocalRuntimeRole::Worker,
                0,
                &super::CurrentEnvironmentSnapshot {
                    values: Default::default(),
                },
                &expired,
            )
        }
        .unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
        let error = super::FileObservation::capture_scoped(
            std::path::Path::new("absent-is-not-the-cause"),
            &expired,
        )
        .unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
    }
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    #[test]
    #[allow(
        unsafe_code,
        reason = "isolated actual glibc sibling-load and current-module TLS ordering control"
    )]
    fn local_runtime_scope_releases_for_sibling_loading_after_owned_module_tls_use() {
        use std::sync::mpsc;
        let (start_tx, start_rx) = mpsc::channel();
        let (started_tx, started_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();
        // Thread/TLS allocation is outside the loader callback.
        let sibling = std::thread::spawn(move || {
            start_rx.recv().unwrap();
            started_tx.send(()).unwrap(); // Acknowledgment precedes dlopen, never waits on loader.
            // SAFETY: balanced system-library handle, no executable authority escapes.
            let handle = unsafe {
                libc::dlopen(c"libutil.so.1".as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL)
            };
            assert!(!handle.is_null());
            // SAFETY: handle is the live, uniquely owned dlopen result above.
            assert_eq!(unsafe { libc::dlclose(handle) }, 0);
            done_tx.send(()).unwrap();
        });
        super::with_local_runtime_scope(|| {
            start_tx.send(()).unwrap();
            started_rx
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap();
            // Exercise a current owning-module TLS access while a sibling may be
            // waiting in dlopen. No new thread/module/provider is used by this callback.
            super::linux_loader_scope::touch_current_module_tls();
            Ok(())
        })
        .unwrap();
        // Wait/join happens after scope releases the glibc loaded-object lock.
        done_rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        sibling.join().unwrap();
        super::with_local_runtime_scope(|| {
            super::linux_loader_scope::touch_current_module_tls();
            Ok(())
        })
        .unwrap();
    }
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    #[test]
    fn local_runtime_scope_contains_panic_and_releases_loader_lock() {
        let panic = std::panic::catch_unwind(|| {
            super::with_local_runtime_scope::<()>(|| panic!("scope control"))
        });
        assert!(panic.is_err());
        assert_eq!(super::with_local_runtime_scope(|| Ok(7)).unwrap(), 7);
    }
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    #[test]
    #[allow(
        unsafe_code,
        reason = "isolated actual glibc transient-load refusal control"
    )]
    fn local_runtime_scope_refuses_reentrant_load_and_unload_even_when_mapping_snapshot_matches() {
        let before =
            super::executable_mapping_files(&fs::read_to_string("/proc/self/maps").unwrap())
                .unwrap();
        assert!(
            !before
                .iter()
                .any(|path| path.file_name().is_some_and(|name| name == "libutil.so.1")),
            "control requires unloaded libutil"
        );
        let result = super::with_local_runtime_scope(|| {
            // SAFETY: NUL-terminated system library name; handle is used only for
            // balanced dlclose, never as arbitrary executable/data authority.
            let handle = unsafe {
                libc::dlopen(c"libutil.so.1".as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL)
            };
            assert!(
                !handle.is_null(),
                "supported glibc libutil control unavailable"
            );
            // SAFETY: this call closes exactly the non-null handle just opened.
            assert_eq!(unsafe { libc::dlclose(handle) }, 0);
            Ok(())
        });
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("reentrant loading")
        );
        assert_eq!(
            before,
            super::executable_mapping_files(&fs::read_to_string("/proc/self/maps").unwrap())
                .unwrap()
        );
        assert_eq!(super::with_local_runtime_scope(|| Ok(9)).unwrap(), 9);
    }
    #[test]
    fn local_runtime_mapping_controls_refuse_unknown_writable_deleted_and_ambiguous_code() {
        let supported = "1000-2000 r-xp 00000000 08:01 42 /lib/actual.so\n2000-3000 r--p 00001000 08:01 42 /lib/actual.so\n3000-4000 r-xp 00000000 00:00 0 [vdso]";
        let expected =
            std::collections::BTreeSet::from([std::path::PathBuf::from("/lib/actual.so")]);
        assert_eq!(
            super::executable_mapping_files(supported).unwrap(),
            expected
        );
        for code in [
            "1000-2000 r-xp 00000000 00:00 0",
            "1000-2000 rwxp 00000000 08:01 42 /lib/actual.so",
            "1000-2000 r-xp 00000000 08:01 42 /lib/actual.so (deleted)",
            "1000-2000 r-xp 00000000 08:01 42 /lib/space name.so",
            "1000-2000 r-xp 00000000 00:00 0 [anon:jit]",
        ] {
            assert!(super::executable_mapping_files(code).is_err(), "{code}");
        }
        assert!(super::executable_mapping_files("1000-2000 r-xp 00000000 00:00 0 [vdso]").is_err());
    }
    #[test]
    fn local_runtime_environment_controls_include_startup_loader_and_runtime_values() {
        let observe = |entries: &[&str]| {
            super::runtime_environment(entries.iter().map(|entry| entry.as_bytes().to_vec()))
                .unwrap()
        };
        let baseline = observe(&[
            "LD_LIBRARY_PATH=/native",
            "MKL_NUM_THREADS=2",
            "PATH=/compiler",
        ]);
        assert_eq!(
            baseline,
            observe(&[
                "PATH=/unrelated",
                "MKL_NUM_THREADS=2",
                "LD_LIBRARY_PATH=/native"
            ])
        );
        assert_ne!(
            baseline,
            observe(&["LD_LIBRARY_PATH=/changed", "MKL_NUM_THREADS=2"])
        );
        assert_ne!(
            baseline,
            observe(&["LD_LIBRARY_PATH=/native", "MKL_NUM_THREADS=3"])
        );
        assert_ne!(baseline, observe(&["MKL_NUM_THREADS=2"]));
        assert!(
            super::runtime_environment(
                [b"LD_PRELOAD=a".to_vec(), b"LD_PRELOAD=b".to_vec()].into_iter()
            )
            .is_err()
        );
    }
    #[cfg(target_os = "linux")]
    #[test]
    #[allow(
        unsafe_code,
        reason = "actual controlled test process excludes environment writers during observation"
    )]
    fn local_runtime_observation_binds_actual_anchor_role_and_loaded_bytes_without_checkout() {
        // SAFETY: this control does not mutate the process environment or admit
        // external callbacks during any actual capture or verification below.
        let environment =
            unsafe { super::CurrentEnvironmentSnapshot::capture_scoped(&|| Ok(())) }.unwrap();
        fn anchor() {}
        let anchor = anchor as *const () as usize;
        // SAFETY: the controlled process excludes environment writers throughout this capture.
        let observation = unsafe {
            super::LocalRuntimeObservation::capture(
                super::LocalRuntimeRole::Worker,
                anchor,
                &environment,
            )
        }
        .unwrap();
        assert_eq!(observation.artifact, observation.executable);
        assert!(
            observation
                .loaded
                .iter()
                .any(|file| file.canonical == observation.artifact.canonical)
        );
        assert_eq!(
            observation.identity().unwrap(),
            // SAFETY: the same controlled environment exclusion remains in force.
            unsafe {
                super::LocalRuntimeObservation::capture(
                    super::LocalRuntimeRole::Worker,
                    anchor,
                    &environment,
                )
            }
            .unwrap()
            .identity()
            .unwrap()
        );
        // SAFETY: this verification uses the same controlled environment exclusion.
        unsafe { observation.verify(anchor, &environment) }.unwrap();
        assert!(
            // SAFETY: the same controlled environment exclusion remains in force.
            unsafe {
                super::LocalRuntimeObservation::capture(
                    super::LocalRuntimeRole::Worker,
                    0,
                    &environment,
                )
            }
            .is_err()
        );
        let mut changed = observation.clone();
        changed.role = super::LocalRuntimeRole::Python;
        assert_ne!(changed.identity().unwrap(), observation.identity().unwrap());
        changed = observation.clone();
        changed.loaded[0].sha256 = "0".repeat(64);
        assert_ne!(changed.identity().unwrap(), observation.identity().unwrap());
        // SAFETY: only owned observation metadata changed; environment exclusion is unchanged.
        assert!(unsafe { changed.verify(anchor, &environment) }.is_err());
        changed = observation.clone();
        changed.kernel_release.push(b'!');
        assert_ne!(changed.identity().unwrap(), observation.identity().unwrap());
        // SAFETY: only owned observation metadata changed; environment exclusion is unchanged.
        assert!(unsafe { changed.verify(anchor, &environment) }.is_err());
        changed = observation.clone();
        changed.kernel_version.push(b'!');
        assert_ne!(changed.identity().unwrap(), observation.identity().unwrap());
        // SAFETY: only owned observation metadata changed; environment exclusion is unchanged.
        assert!(unsafe { changed.verify(anchor, &environment) }.is_err());
        changed = observation.clone();
        changed
            .current_environment
            .insert("LD_PRELOAD".into(), "different".into());
        assert_ne!(changed.identity().unwrap(), observation.identity().unwrap());
        // SAFETY: only owned observation metadata changed; environment exclusion is unchanged.
        assert!(unsafe { changed.verify(anchor, &environment) }.is_err());
    }
    #[test]
    fn scientific_producer_payload_refuses_identity_substitution_and_preserves_projection() {
        let inputs = std::collections::BTreeMap::from([(
            "source".into(),
            vec![super::ConsumedInput {
                path: "src/lib.rs".into(),
                identity: "known-input".into(),
                representation: "active-source".into(),
                observation: None,
            }],
        )]);
        let environment =
            std::collections::BTreeMap::from([("RUSTFLAGS".into(), "known-digest".into())]);
        let abi = Some("reviewed-ABI".into());
        let identity =
            super::scientific_producer_identity("fixture", &[], &[], &inputs, &environment, &abi)
                .unwrap();
        // Preserve exact original tuple and field order, independently of receipt
        // metadata, association and current dirty outer observation.
        let wire=br#"["fixture",[],[],{"source":[{"path":"src/lib.rs","identity":"known-input","representation":"active-source"}]},{"RUSTFLAGS":"known-digest"},"reviewed-ABI"]"#;
        let mut original = pse_ids::FramedHasher::new(pse_ids::Frame::ProducerV1);
        original.part(b"producer").part(wire);
        assert_eq!(identity, original.finish_hash());
        let mut receipt = serde_json::json!({"receipt_version":super::DEPLOYMENT_RECEIPT_VERSION,"frame":pse_ids::Frame::ProducerV1.as_str(),"package":"fixture","units":[],"selected_lock_records":[],"consumed_inputs":inputs,"declared_environment":environment,"native_abi":abi,"identity":identity.to_hex(),"deployment":{"arbitrary":"separate association"}});
        assert_eq!(
            super::verify_scientific_producer_identity(&receipt).unwrap(),
            identity
        );
        receipt["identity"] = "a".repeat(64).into();
        assert!(super::verify_scientific_producer_identity(&receipt).is_err());
        receipt["identity"] = identity.to_hex().into();
        receipt["consumed_inputs"]["source"][0]["identity"] = "changed-input".into();
        assert!(super::verify_scientific_producer_identity(&receipt).is_err());
    }

    #[test]
    fn deployment_association_refuses_current_consumed_and_artifact_changes_but_not_outer_edits() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let artifact = root.join("artifact");
        let source = root.join("consumed.rs");
        let unrelated = root.join("unrelated.rs");
        fs::write(&artifact, b"actual artifact").unwrap();
        fs::write(&source, b"consumed source").unwrap();
        fs::write(&unrelated, b"outer source").unwrap();
        let association = super::DeploymentAssociation {
            producer_identity: "isolated-file-association-fixture".into(),
            selected_root: "isolated-file-root".into(),
            workspace_root: root.into(),
            artifact: super::FileObservation::capture(&artifact).unwrap(),
            built_artifact: super::FileObservation::capture(&artifact).unwrap(),
            files: vec![super::FileObservation::capture(&source).unwrap()],
            absent_files: vec![root.join("absent")],
            namespaces: Default::default(),
            native_environment: Default::default(),
            native_absent_environment_prefixes: vec![],
        };
        association.verify_current(&artifact).unwrap();
        fs::write(&unrelated, b"changed dirty outer source").unwrap();
        association.verify_current(&artifact).unwrap();
        fs::write(&source, b"dirty consumed source").unwrap();
        assert!(association.verify_current(&artifact).is_err());
        fs::write(&source, b"consumed source").unwrap();
        fs::write(root.join("absent"), b"new consumed presence").unwrap();
        assert!(association.verify_current(&artifact).is_err());
        fs::remove_file(root.join("absent")).unwrap();
        fs::write(&artifact, b"replacement at same path").unwrap();
        assert!(association.verify_current(&artifact).is_err());
        let historical = br#"{"receipt_version":1,"deployment":{"wrong":"shape"}}"#;
        assert!(
            super::DeploymentAssociation::decode(historical)
                .unwrap_err()
                .to_string()
                .contains("interpretation")
        );
    }
    #[test]
    fn deployment_namespace_observes_new_names_links_and_hardlink_equivalence() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        fs::write(root.join("a"), b"native").unwrap();
        let initial = super::namespace_digest(root).unwrap();
        fs::hard_link(root.join("a"), root.join("b")).unwrap();
        let linked = super::namespace_digest(root).unwrap();
        assert_ne!(initial, linked);
        fs::remove_file(root.join("b")).unwrap();
        fs::write(root.join("b"), b"native").unwrap();
        assert_ne!(linked, super::namespace_digest(root).unwrap());
    }
    #[test]
    fn loaded_code_mapping_is_independent_of_metadata_only_paths() {
        let actual = super::loaded_module_path(
            loaded_code_mapping_is_independent_of_metadata_only_paths as *const () as usize,
        )
        .unwrap();
        super::verify_loaded_module(&actual).unwrap();
        let scratch = tempfile::NamedTempFile::new().unwrap();
        assert!(super::verify_loaded_module(scratch.path()).is_err());
    }
    #[test]
    #[cfg(target_os = "linux")]
    fn installed_artifact_without_checkout_has_explicitly_incomplete_source_evidence() {
        const CHILD: &str = "PSE_INSTALLED_ARTIFACT_OBSERVATION_CHILD";
        if std::env::var_os(CHILD).is_some() {
            let executable = std::env::current_exe().unwrap();
            super::verify_loaded_module(&executable).unwrap();
            assert!(super::workspace_root(&executable).is_err());
            let observed = super::observe_deployment(&executable).unwrap();
            assert_eq!(observed.source, None);
            assert_eq!(observed.git_sha, "source-unavailable");
            assert_eq!(
                observed.build,
                super::observe_deployment(&executable).unwrap().build
            );
            assert_eq!(
                serde_json::to_value(&observed).unwrap()["source"],
                serde_json::Value::Null
            );
            return;
        }
        let directory = tempfile::tempdir().unwrap();
        // An installed package may live inside another Rust project's environment.
        // Its manifests do not make that project this artifact's authored checkout.
        fs::write(
            directory.path().join("Cargo.toml"),
            b"[package]\nname='other-project'\n",
        )
        .unwrap();
        fs::write(
            directory.path().join("rust-toolchain.toml"),
            b"[toolchain]\nchannel='unrelated'\n",
        )
        .unwrap();
        let executable = directory.path().join("installed-artifact");
        fs::copy(std::env::current_exe().unwrap(), &executable).unwrap();
        let child = std::process::Command::new(executable)
            .args(["--exact", "identity::foundation_unit::installed_artifact_without_checkout_has_explicitly_incomplete_source_evidence", "--nocapture"])
            .env(CHILD, "1").output().unwrap();
        assert!(
            child.status.success(),
            "{}",
            String::from_utf8_lossy(&child.stderr)
        );
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn loaded_artifact_same_path_replacement_refuses_even_identical_bytes() {
        const CHILD: &str = "PSE_LOADED_ARTIFACT_REPLACEMENT_UNIT_CHILD";
        if std::env::var_os(CHILD).is_some() {
            let executable = std::env::current_exe().unwrap();
            super::verify_loaded_module(&executable).unwrap();
            let replacement = executable.with_extension("replacement");
            fs::copy(&executable, &replacement).unwrap();
            fs::rename(&replacement, &executable).unwrap();
            assert!(super::verify_loaded_module(&executable).is_err());
            assert!(
                super::loaded_module_path(
                    loaded_artifact_same_path_replacement_refuses_even_identical_bytes as *const ()
                        as usize
                )
                .is_err()
            );
            return;
        }
        let directory = tempfile::tempdir().unwrap();
        let executable = directory.path().join("actual-mapped-fixture");
        fs::copy(std::env::current_exe().unwrap(), &executable).unwrap();
        let child=std::process::Command::new(executable).args(["--exact","identity::foundation_unit::loaded_artifact_same_path_replacement_refuses_even_identical_bytes","--nocapture"]).env(CHILD,"1").output().unwrap();
        assert!(
            child.status.success(),
            "{}",
            String::from_utf8_lossy(&child.stderr)
        );
    }

    #[test]
    fn absent_vendor_is_valid_and_later_vendored_sources_rekey() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        for name in ["crates/a/src", "xtask/src"] {
            fs::create_dir_all(root.join(name)).unwrap();
        }
        fs::write(root.join("crates/a/src/lib.rs"), b"library").unwrap();
        fs::write(root.join("xtask/src/main.rs"), b"worker").unwrap();
        let original = super::digest(super::source_files(root).unwrap());
        fs::create_dir(root.join("vendor")).unwrap();
        assert_eq!(original, super::digest(super::source_files(root).unwrap()));
        fs::write(root.join("vendor/override.rs"), b"actual override").unwrap();
        assert_ne!(original, super::digest(super::source_files(root).unwrap()));
    }

    #[test]
    fn actual_outer_inventory_rekeys_dirty_worker_sources_and_excludes_build_outputs() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        for name in ["crates/a/src", "vendor", "xtask/src", "xtask/target"] {
            fs::create_dir_all(root.join(name)).unwrap();
        }
        fs::write(root.join("crates/a/src/lib.rs"), b"library").unwrap();
        let worker = root.join("xtask/src/worker.rs");
        fs::write(&worker, b"worker one").unwrap();
        fs::write(root.join("xtask/target/generated.rs"), b"build output").unwrap();
        let entries = super::source_files(root).unwrap();
        assert_eq!(entries.len(), 2);
        assert!(
            entries
                .iter()
                .any(|(name, bytes)| name == "xtask/src/worker.rs" && bytes == b"worker one")
        );
        let original = super::digest(entries);
        fs::write(&worker, b"worker two").unwrap();
        assert_ne!(original, super::digest(super::source_files(root).unwrap()));
        let changed = super::digest(super::source_files(root).unwrap());
        fs::create_dir_all(root.join("python/pse")).unwrap();
        fs::write(root.join("python/pse/workflow.py"), b"authored Python").unwrap();
        let python_changed = super::digest(super::source_files(root).unwrap());
        assert_ne!(changed, python_changed);
        fs::write(
            root.join("python/pse/_native.example.so"),
            b"deployed artifact",
        )
        .unwrap();
        assert_eq!(
            python_changed,
            super::digest(super::source_files(root).unwrap())
        );
        fs::write(
            root.join("xtask/target/generated.rs"),
            b"different build output",
        )
        .unwrap();
        assert_eq!(
            python_changed,
            super::digest(super::source_files(root).unwrap())
        );
    }

    #[test]
    fn outer_build_observation_tracks_cache_configuration_without_rekeying_source() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        for name in ["crates", "xtask", ".cargo", ".config"] {
            fs::create_dir(root.join(name)).unwrap();
        }
        for name in [
            "Cargo.toml",
            "Cargo.lock",
            "uv.lock",
            "pyproject.toml",
            "rust-toolchain.toml",
            ".cargo/config.toml",
            ".config/build.toml",
            ".config/native-cache.cmake",
            ".config/sccache.toml",
        ] {
            fs::write(root.join(name), b"original configuration").unwrap();
        }
        let original = super::observe_outer(root).unwrap();
        fs::write(root.join(".config/sccache.toml"), b"changed cache mode").unwrap();
        let changed = super::observe_outer(root).unwrap();
        assert_ne!(original.build, changed.build);
        assert_eq!(original.source, changed.source);
    }

    #[test]
    fn dirty_bytes_names_and_configuration_affect_identity_but_enumeration_order_does_not() {
        let a = ("crates/a/src/lib.rs".into(), b"source".to_vec());
        let b = ("feature:ipopt".into(), b"false".to_vec());
        let original = super::digest(vec![a.clone(), b.clone()]);
        assert_eq!(original, super::digest(vec![b.clone(), a.clone()]));
        assert_ne!(
            original,
            super::digest(vec![(a.0.clone(), b"dirty source".to_vec()), b.clone()])
        );
        assert_ne!(
            original,
            super::digest(vec![("crates/other.rs".into(), a.1.clone()), b.clone()])
        );
        assert_ne!(original, super::digest(vec![a, (b.0, b"true".to_vec())]));
    }
}
