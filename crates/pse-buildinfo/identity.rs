// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! External artifact association and fresh outer observation framing.
use std::{fs, path::Path};

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
        Ok(Self {
            path: path.to_path_buf(),
            canonical: path.canonicalize()?,
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
        self.artifact.verify()?;
        self.built_artifact.verify()?;
        let actual = FileObservation::capture(observed)?;
        if actual.sha256 != self.artifact.sha256 || actual.canonical != self.artifact.canonical {
            return Err(std::io::Error::other(
                "actual deployed artifact differs from selected capture",
            ));
        }
        for file in &self.files {
            file.verify()?;
        }
        for path in &self.absent_files {
            match fs::symlink_metadata(path) {
                Ok(_) => return Err(std::io::Error::other("reviewed absent input is present")),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
        for (path, expected) in &self.namespaces {
            if namespace_digest(path)? != *expected {
                return Err(std::io::Error::other("consumed search namespace changed"));
            }
        }
        for (name, expected) in &self.native_environment {
            let actual = std::env::var(name).ok();
            if actual.as_ref().map(|value| value_digest(value.as_bytes())) != *expected {
                return Err(std::io::Error::other(format!(
                    "consumed native environment changed: {name}"
                )));
            }
        }
        for prefix in &self.native_absent_environment_prefixes {
            if std::env::vars_os().any(|(name, _)| name.to_string_lossy().starts_with(prefix)) {
                return Err(std::io::Error::other(
                    "reviewed absent native environment family is present",
                ));
            }
        }
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
    use std::collections::{BTreeMap, BTreeSet};
    let mut pending = vec![root.to_path_buf()];
    let mut visited = BTreeSet::new();
    let mut entries = BTreeMap::new();
    let mut extent = 0u64;
    #[cfg(unix)]
    let mut aliases = BTreeMap::<(u64, u64), Vec<std::path::PathBuf>>::new();
    while let Some(path) = pending.pop() {
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
            Some(FileObservation::capture(&path)?)
        } else {
            None
        };
        let canonical = target.as_ref().map(|_| path.canonicalize()).transpose()?;
        let directory = target.as_ref().is_some_and(fs::Metadata::is_dir);
        if directory
            && canonical
                .as_ref()
                .is_some_and(|value| visited.insert(value.clone()))
        {
            for entry in fs::read_dir(&path)? {
                if entries.len() + pending.len() >= 262_144 {
                    return Err(std::io::Error::other("deployment namespace entry limit"));
                }
                pending.push(entry?.path());
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
    Ok(value_digest(
        &serde_json::to_vec(&(entries, groups)).map_err(std::io::Error::other)?,
    ))
}

/// Fresh complete outer observation at deployment/run admission. These hashes
/// record provenance; neither is a scientific producer or compiled all-tree input.
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
    let source = digest(source_files(root)?);
    let mut configuration = Vec::new();
    configuration.push((
        "interpretation".into(),
        b"deployment-outer-observation.v2".to_vec(),
    ));
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
        configuration.push((name.into(), fs::read(root.join(name))?));
    }
    let git = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "HEAD"])
        .output()?;
    let git_sha = if git.status.success() {
        String::from_utf8_lossy(&git.stdout).trim().to_owned()
    } else {
        "sdist".into()
    };
    configuration.push(("git-context".into(), git_sha.as_bytes().to_vec()));
    Ok(OuterObservation {
        source: Some(source),
        build: digest(configuration),
        git_sha,
    })
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
    match workspace_root(artifact) {
        Ok(root) => observe_outer(&root),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let observed = FileObservation::capture(artifact)?;
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
        let mapped = fs::read_to_string("/proc/self/maps")?.lines().any(|line| {
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
        let maps = fs::read_to_string("/proc/self/maps")?;
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
    pub fn capture(role: LocalRuntimeRole, anchor: usize) -> std::io::Result<Self> {
        Self::capture_scoped(role, anchor, &|| Ok(()))
    }
    /// Capture the receiving boundary under its original stop/clock checkpoint.
    /// Foreign loader-lock acquisition itself remains outside cooperative interruption.
    pub fn capture_scoped(
        role: LocalRuntimeRole,
        anchor: usize,
        checkpoint: &dyn Fn() -> std::io::Result<()>,
    ) -> std::io::Result<Self> {
        checkpoint()?;
        #[cfg(target_os = "linux")]
        {
            let before = executable_mapping_files(&fs::read_to_string("/proc/self/maps")?)?;
            // Several authority paths normally name the same running ELF: the
            // caller's anchor, current_exe, and its executable mapping. Observe
            // each exact path once, then retain the independent end-of-capture
            // verification below. Hashing the same large worker binary for each
            // alias needlessly lengthens the loader exclusion held by the caller.
            let mut observations =
                std::collections::BTreeMap::<std::path::PathBuf, FileObservation>::new();
            let mut observe = |path: &Path| -> std::io::Result<FileObservation> {
                if let Some(observation) = observations.get(path) {
                    return Ok(observation.clone());
                }
                let observation = FileObservation::capture_scoped(path, checkpoint)?;
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
                kernel_release: fs::read("/proc/sys/kernel/osrelease")?,
                kernel_version: fs::read("/proc/sys/kernel/version")?,
                artifact,
                executable,
                loaded,
                loader_files,
                startup_environment: runtime_environment(
                    fs::read("/proc/self/environ")?
                        .split(|byte| *byte == 0)
                        .filter(|entry| !entry.is_empty())
                        .map(|entry| entry.to_vec()),
                )?,
                current_environment: runtime_environment(std::env::vars_os().map(
                    |(name, value)| {
                        use std::os::unix::ffi::OsStrExt;
                        let mut entry = name.as_bytes().to_vec();
                        entry.push(b'=');
                        entry.extend_from_slice(value.as_bytes());
                        entry
                    },
                ))?,
            };
            checkpoint()?;
            if before != executable_mapping_files(&fs::read_to_string("/proc/self/maps")?)? {
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
    pub fn verify(&self, anchor: usize) -> std::io::Result<()> {
        self.verify_scoped(anchor, &|| Ok(()))
    }
    /// Verify receiving inputs without resetting the enclosing operation clock.
    pub fn verify_scoped(
        &self,
        anchor: usize,
        checkpoint: &dyn Fn() -> std::io::Result<()>,
    ) -> std::io::Result<()> {
        if Self::capture_scoped(self.role, anchor, checkpoint)? != *self {
            return Err(std::io::Error::other("local runtime observation changed"));
        }
        Ok(())
    }
    /// Exact versioned context identity, without a checkout or compiler-source capture.
    pub fn identity(&self) -> std::io::Result<pse_ids::ContentHash> {
        Ok(digest(vec![(
            "local-runtime-observation.v1".into(),
            serde_json::to_vec(self).map_err(std::io::Error::other)?,
        )]))
    }
}

// A finite supported runtime configuration boundary, not arbitrary build environment.
// Startup and current values both matter: glibc can consume a startup setting which a
// Python host later removes from os.environ. Digests keep secret values out of records.
fn runtime_environment(
    entries: impl Iterator<Item = Vec<u8>>,
) -> std::io::Result<std::collections::BTreeMap<String, String>> {
    let mut result = std::collections::BTreeMap::new();
    for entry in entries {
        let Some(equal) = entry.iter().position(|byte| *byte == b'=') else {
            return Err(std::io::Error::other("malformed runtime environment"));
        };
        let name = std::str::from_utf8(&entry[..equal]).map_err(std::io::Error::other)?;
        if ([
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
            || matches!(name, "LANG" | "LC_ALL" | "LC_NUMERIC" | "TZ"))
            && result
                .insert(name.to_owned(), value_digest(&entry[equal + 1..]))
                .is_some()
        {
            return Err(std::io::Error::other("duplicate runtime environment input"));
        }
    }
    Ok(result)
}

fn executable_mapping_files(
    maps: &str,
) -> std::io::Result<std::collections::BTreeSet<std::path::PathBuf>> {
    let mut files = std::collections::BTreeSet::new();
    for line in maps.lines() {
        let parts = line.split_whitespace().collect::<Vec<_>>();
        if parts.len() < 5 {
            return Err(std::io::Error::other("malformed Linux mapping"));
        }
        if !parts[1].contains('x') {
            continue;
        }
        // vDSO/vsyscall are kernel-supplied code, governed by the platform support
        // boundary. They have no file-backed artifact to hash and are not JIT code.
        if parts.len() == 6 && matches!(parts[5], "[vdso]" | "[vsyscall]") {
            continue;
        }
        if parts[1].contains('w')
            || parts.len() != 6
            || !parts[5].starts_with('/')
            || parts[4] == "0"
        {
            return Err(std::io::Error::other(
                "unknown, writable, deleted or ambiguous executable mapping",
            ));
        }
        files.insert(std::path::PathBuf::from(parts[5]));
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
        return FileObservation::capture(path);
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
    association.verify_current(path)
}

/// Complete outer product sources, including the executable worker composition root.
/// Relevant scientific producer inputs are selected separately by their actual unit DAG.
pub(crate) fn source_files(root: &Path) -> std::io::Result<Vec<(String, Vec<u8>)>> {
    let mut files = Vec::new();
    for name in ["crates", "vendor", "xtask", "python", "scripts"] {
        let directory = root.join(name);
        // Retired source overrides leave no vendor directory in a fresh checkout.
        // Observe its reappearance so a future actual override still rekeys the build.
        if matches!(name, "vendor" | "python" | "scripts") && !directory.try_exists()? {
            continue;
        }
        collect_sources(root, &directory, &mut files)?;
    }
    Ok(files)
}

fn collect_sources(
    root: &Path,
    directory: &Path,
    files: &mut Vec<(String, Vec<u8>)>,
) -> std::io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let kind = entry.file_type()?;
        if kind.is_dir() {
            if !matches!(
                entry.file_name().to_str(),
                Some("target" | "__pycache__" | ".git" | ".pytest_cache" | ".ruff_cache")
            ) {
                collect_sources(root, &path, files)?;
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
            let relative = path
                .strip_prefix(root)
                .map_err(std::io::Error::other)?
                .to_string_lossy()
                .replace('\\', "/");
            files.push((relative, fs::read(path)?));
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
        let error = super::LocalRuntimeObservation::capture_scoped(
            super::LocalRuntimeRole::Worker,
            0,
            &expired,
        )
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
    fn local_runtime_observation_binds_actual_anchor_role_and_loaded_bytes_without_checkout() {
        fn anchor() {}
        let anchor = anchor as *const () as usize;
        let observation =
            super::LocalRuntimeObservation::capture(super::LocalRuntimeRole::Worker, anchor)
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
            super::LocalRuntimeObservation::capture(super::LocalRuntimeRole::Worker, anchor)
                .unwrap()
                .identity()
                .unwrap()
        );
        observation.verify(anchor).unwrap();
        assert!(
            super::LocalRuntimeObservation::capture(super::LocalRuntimeRole::Worker, 0).is_err()
        );
        let mut changed = observation.clone();
        changed.role = super::LocalRuntimeRole::Python;
        assert_ne!(changed.identity().unwrap(), observation.identity().unwrap());
        changed = observation.clone();
        changed.loaded[0].sha256 = "0".repeat(64);
        assert_ne!(changed.identity().unwrap(), observation.identity().unwrap());
        assert!(changed.verify(anchor).is_err());
        changed = observation.clone();
        changed.kernel_release.push(b'!');
        assert_ne!(changed.identity().unwrap(), observation.identity().unwrap());
        assert!(changed.verify(anchor).is_err());
        changed = observation.clone();
        changed.kernel_version.push(b'!');
        assert_ne!(changed.identity().unwrap(), observation.identity().unwrap());
        assert!(changed.verify(anchor).is_err());
        changed = observation.clone();
        changed
            .current_environment
            .insert("LD_PRELOAD".into(), "different".into());
        assert_ne!(changed.identity().unwrap(), observation.identity().unwrap());
        assert!(changed.verify(anchor).is_err());
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
