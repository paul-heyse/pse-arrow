// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit pinned optional tool installation. Passive verification never downloads or executes.
use crate::{HostError, environment::Environment};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::Read,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
};

/// A declared optional tool, without arbitrary source URL or installation-path input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    /// Perfetto trace processor shell.
    TraceProcessor,
    /// Optional MCP language-server adapter.
    Mcpls,
    /// Complete rr recording/replay runtime.
    Rr,
    /// Native compiler cache.
    Sccache,
    /// OpenTelemetry contrib collector.
    Collector,
    /// Local OpenObserve backend.
    OpenObserve,
}
impl Tool {
    /// Stable declaration name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::TraceProcessor => "trace-processor",
            Self::Mcpls => "mcpls",
            Self::Rr => "rr",
            Self::Sccache => "sccache",
            Self::Collector => "otelcol-contrib",
            Self::OpenObserve => "openobserve",
        }
    }
    /// Parse only supported declarations, never the unrelated cargo-tools table.
    pub fn parse(value: &str) -> Result<Self, HostError> {
        match value {
            "trace-processor" => Ok(Self::TraceProcessor),
            "mcpls" => Ok(Self::Mcpls),
            "rr" => Ok(Self::Rr),
            "sccache" => Ok(Self::Sccache),
            "otelcol-contrib" | "collector" => Ok(Self::Collector),
            "openobserve" => Ok(Self::OpenObserve),
            _ => Err(invalid("unknown diagnostic tool")),
        }
    }
}
/// Exact pin from the checkout's sole tool declaration.
#[derive(Clone, Debug, Deserialize)]
pub struct ToolSpec {
    /// Exact declared release version.
    pub version: String,
    /// HTTPS release archive URL.
    pub url: String,
    /// Exact archive SHA-256.
    pub sha256: String,
    /// Executable checksum when the declaration provides one.
    pub binary_sha256: Option<String>,
    /// Selected executable member.
    pub member: String,
    /// TAR archive kind, or absent for ZIP.
    pub archive: Option<String>,
    /// Required root for a portable bundle.
    pub prefix: Option<String>,
    max_archive_bytes: Option<String>,
    max_binary_bytes: Option<String>,
}
#[derive(Deserialize, Serialize)]
struct BundleReceipt {
    version: String,
    archive_sha256: String,
    source: String,
    members: BTreeMap<String, String>,
}
#[derive(Deserialize, Serialize, PartialEq, Eq)]
struct BinaryReceipt {
    schema_version: u32,
    version: String,
    archive_sha256: String,
    binary_sha256: String,
    source: String,
    member: String,
}
fn invalid(message: impl Into<String>) -> HostError {
    HostError::Ownership(message.into())
}
fn hash_valid(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}
fn relative(value: &Path) -> bool {
    !value.as_os_str().is_empty()
        && value
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}
/// Reject symlinks in every existing path component, including parent aliases.
pub(crate) fn no_symlinks(path: &Path) -> Result<(), HostError> {
    if !path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::CurDir))
    {
        return Err(invalid("managed path must be absolute without traversal"));
    }
    for parent in path.ancestors() {
        match fs::symlink_metadata(parent) {
            Ok(info) if info.file_type().is_symlink() => {
                return Err(invalid("managed path must not traverse symlinks"));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}
pub(crate) fn digest(path: &Path) -> Result<String, HostError> {
    no_symlinks(path)?;
    let mut input = OpenOptions::new()
        .read(true)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(path)?;
    if !input.metadata()?.is_file() {
        return Err(invalid("managed input must be a regular file"));
    }
    let mut hash = Sha256::new();
    let mut bytes = [0_u8; 65536];
    loop {
        let count = input.read(&mut bytes)?;
        if count == 0 {
            break;
        }
        hash.update(&bytes[..count]);
    }
    Ok(hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}
impl ToolSpec {
    fn validate(&self) -> Result<(), HostError> {
        if !self.url.starts_with("https://")
            || !hash_valid(&self.sha256)
            || self
                .binary_sha256
                .as_deref()
                .is_some_and(|hash| !hash_valid(hash))
            || !relative(Path::new(&self.version))
            || Path::new(&self.version).components().count() != 1
            || !relative(Path::new(&self.member))
        {
            return Err(invalid("invalid diagnostic tool pin"));
        }
        if !matches!(
            self.archive.as_deref(),
            None | Some("tar.gz" | "portable-tar")
        ) {
            return Err(invalid("unsupported diagnostic tool archive"));
        }
        if self.archive.as_deref() == Some("portable-tar")
            && !self.prefix.as_deref().is_some_and(|value| {
                relative(Path::new(value)) && Path::new(value).components().count() == 1
            })
        {
            return Err(invalid("invalid portable tool release prefix"));
        }
        self.archive_limit()?;
        self.binary_limit()?;
        Ok(())
    }
    fn limit(value: Option<&str>) -> Result<u64, HostError> {
        value.map_or(Ok(256 << 20), |value| {
            value
                .parse::<u64>()
                .ok()
                .filter(|value| *value > 0)
                .ok_or_else(|| invalid("tool installation allowance must be a positive integer"))
        })
    }
    fn archive_limit(&self) -> Result<u64, HostError> {
        Self::limit(self.max_archive_bytes.as_deref())
    }
    fn binary_limit(&self) -> Result<u64, HostError> {
        Self::limit(self.max_binary_bytes.as_deref())
    }
}
/// Read only the selected exact tool declaration.
pub fn tool_spec(root: &Path, tool: Tool) -> Result<ToolSpec, HostError> {
    let document: toml::Value = toml::from_str(&fs::read_to_string(
        root.join(".config/diagnostic-tools.toml"),
    )?)
    .map_err(|_| invalid("invalid diagnostic tool declarations"))?;
    let spec: ToolSpec = document
        .get(tool.name())
        .ok_or_else(|| invalid("missing diagnostic tool declaration"))?
        .clone()
        .try_into()
        .map_err(|_| invalid("invalid diagnostic tool declaration"))?;
    spec.validate()?;
    Ok(spec)
}
/// Format requested Cargo install selections from the existing exact tool declarations.
/// Ordinary tools absent from that table retain their original caller selection.
pub fn cargo_tool_specs(root: &Path, names: &[String]) -> Result<Vec<String>, HostError> {
    let document: toml::Value = toml::from_str(&fs::read_to_string(
        root.join(".config/diagnostic-tools.toml"),
    )?)
    .map_err(|_| invalid("invalid diagnostic tool declarations"))?;
    let tools = document
        .get("cargo-tools")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| invalid("missing Cargo tool declarations"))?;
    names
        .iter()
        .map(|name| {
            if name.is_empty() || name.starts_with('-') || name.chars().any(char::is_whitespace) {
                return Err(invalid("invalid Cargo tool selection"));
            }
            match tools.get(name) {
                None => Ok(name.clone()),
                Some(value) => value
                    .as_str()
                    .filter(|version| {
                        !version.is_empty()
                            && version.bytes().all(|byte| {
                                byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'+')
                            })
                    })
                    .map(|version| format!("{name}@={version}"))
                    .ok_or_else(|| invalid("invalid Cargo tool version declaration")),
            }
        })
        .collect()
}
/// Resolve the existing versioned path without creating it.
pub fn binary_path(
    root: &Path,
    tool: Tool,
    environment: &Environment,
) -> Result<PathBuf, HostError> {
    let spec = tool_spec(root, tool)?;
    let base = match environment
        .get(std::ffi::OsStr::new("XDG_DATA_HOME"))
        .filter(|value| !value.is_empty())
    {
        Some(value) => PathBuf::from(value),
        None => PathBuf::from(
            environment
                .get(std::ffi::OsStr::new("HOME"))
                .ok_or_else(|| invalid("HOME is required for diagnostic tool paths"))?,
        )
        .join(".local/share"),
    };
    let result = base
        .join("pse/tools")
        .join(if tool == Tool::TraceProcessor {
            "perfetto"
        } else {
            tool.name()
        })
        .join(&spec.version)
        .join(&spec.member);
    no_symlinks(&result)?;
    Ok(result)
}
fn verify_bundle(root: &Path, spec: &ToolSpec) -> Result<(), HostError> {
    no_symlinks(root)?;
    let receipt = root.join("bundle.json");
    no_symlinks(&receipt)?;
    let mut bytes = Vec::new();
    File::open(&receipt)?.take(65537).read_to_end(&mut bytes)?;
    if bytes.len() > 65536 {
        return Err(invalid("portable tool receipt exceeds allowance"));
    }
    let record: BundleReceipt =
        serde_json::from_slice(&bytes).map_err(|_| invalid("invalid portable tool receipt"))?;
    if record.version != spec.version
        || record.archive_sha256 != spec.sha256
        || record.members.is_empty()
        || record.members.len() > 200
    {
        return Err(invalid("portable tool receipt differs from pin"));
    }
    for (member, expected) in &record.members {
        if !relative(Path::new(member))
            || !hash_valid(expected)
            || digest(&root.join(member))? != *expected
        {
            return Err(invalid("portable tool member changed or escaped"));
        }
    }
    for required in [
        spec.member.as_str(),
        "lib/rr/librrpreload.so",
        "lib/rr/librrpage.so",
        "bin/rr_exec_stub",
    ] {
        if !record.members.contains_key(required) {
            return Err(invalid("portable rr runtime member missing"));
        }
    }
    Ok(())
}
fn verify_plain_binary(target: &Path, spec: &ToolSpec) -> Result<(), HostError> {
    no_symlinks(target)?;
    let info = fs::metadata(target)?;
    if !info.is_file() || info.permissions().mode() & 0o111 == 0 {
        return Err(invalid(
            "managed diagnostic executable missing, nonregular or not executable",
        ));
    }
    if let Some(expected) = &spec.binary_sha256
        && digest(target)? != *expected
    {
        return Err(invalid("diagnostic executable checksum differs"));
    }
    Ok(())
}
fn verify_at(target: &Path, spec: &ToolSpec) -> Result<(), HostError> {
    verify_plain_binary(target, spec)?;
    if spec.archive.as_deref() == Some("portable-tar") {
        let root = target
            .ancestors()
            .nth(Path::new(&spec.member).components().count())
            .ok_or_else(|| invalid("portable tool root missing"))?;
        verify_bundle(root, spec)?;
    } else if spec.binary_sha256.is_none() {
        verify_provenance(target, spec)?;
    }
    Ok(())
}
fn provenance_paths(target: &Path) -> Result<(PathBuf, PathBuf), HostError> {
    let parent = target
        .parent()
        .ok_or_else(|| invalid("tool installation parent missing"))?;
    Ok((
        parent.join(".pse-source.archive"),
        parent.join(".pse-install.json"),
    ))
}
fn member_digest(source: &mut impl Read, size: u64, limit: u64) -> Result<String, HostError> {
    if size > limit {
        return Err(invalid("tool member exceeds expanded allowance"));
    }
    let mut hash = Sha256::new();
    let mut bytes = [0_u8; 65536];
    let mut total = 0_u64;
    loop {
        let count = source.read(&mut bytes)?;
        if count == 0 {
            break;
        }
        total = total
            .checked_add(count as u64)
            .ok_or_else(|| invalid("tool member size overflow"))?;
        if total > limit {
            return Err(invalid("tool member exceeds expanded allowance"));
        }
        hash.update(&bytes[..count]);
    }
    if total != size {
        return Err(invalid("tool member size differs"));
    }
    Ok(hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}
struct TarLimits {
    entries: usize,
    expanded: u64,
    decoded: u64,
}
fn tar_limits(spec: &ToolSpec) -> Result<TarLimits, HostError> {
    let portable = spec.archive.as_deref() == Some("portable-tar");
    let expanded = if portable {
        64 << 20
    } else {
        spec.binary_limit()?
            .checked_add(64 << 20)
            .ok_or_else(|| invalid("tool expanded allowance overflow"))?
    };
    // Bound decoded headers, padding and GNU/PAX metadata as well as payloads;
    // the library may consume those before yielding an entry to the caller.
    let decoded = expanded
        .checked_add(if portable { 1 << 20 } else { 4 << 20 })
        .and_then(|limit| limit.checked_add(1))
        .ok_or_else(|| invalid("tool decoded allowance overflow"))?;
    Ok(TarLimits {
        entries: if portable { 200 } else { 4096 },
        expanded,
        decoded,
    })
}
fn account_tar_entry(
    index: usize,
    size: u64,
    expanded: &mut u64,
    limits: &TarLimits,
) -> Result<(), HostError> {
    *expanded = expanded
        .checked_add(size)
        .ok_or_else(|| invalid("tool expanded archive size overflow"))?;
    if index >= limits.entries || *expanded > limits.expanded {
        return Err(invalid(
            "tool archive exceeds entry count or expanded allowance",
        ));
    }
    Ok(())
}
/// Re-read the authenticated archive with the same library member/type/size
/// constraints. A self-asserted binary hash in a local receipt is not sufficient.
fn archive_binary_digest(archive: &Path, spec: &ToolSpec) -> Result<String, HostError> {
    no_symlinks(archive)?;
    if fs::metadata(archive)?.len() > spec.archive_limit()? || digest(archive)? != spec.sha256 {
        return Err(invalid(
            "installed diagnostic source archive checksum or allowance differs",
        ));
    }
    let mut selected = None;
    if spec.archive.as_deref() == Some("tar.gz") {
        let limits = tar_limits(spec)?;
        let mut expanded = 0_u64;
        let mut bundle =
            tar::Archive::new(GzDecoder::new(File::open(archive)?).take(limits.decoded));
        for (index, entry) in bundle.entries()?.enumerate() {
            let mut entry = entry?;
            account_tar_entry(index, entry.size(), &mut expanded, &limits)?;
            if entry.path()?.file_name() != Path::new(&spec.member).file_name() {
                continue;
            }
            if selected.is_some() || !entry.header().entry_type().is_file() {
                return Err(invalid(
                    "tool binary member missing, ambiguous or nonregular",
                ));
            }
            let size = entry.size();
            selected = Some(member_digest(&mut entry, size, spec.binary_limit()?)?);
        }
        if bundle.into_inner().limit() == 0 {
            return Err(invalid("tool archive exceeds decoded stream allowance"));
        }
    } else if spec.archive.is_none() {
        let mut bundle = zip::ZipArchive::new(File::open(archive)?)
            .map_err(|_| invalid("invalid installed tool zip archive"))?;
        for index in 0..bundle.len() {
            let mut member = bundle
                .by_index(index)
                .map_err(|_| invalid("invalid installed tool zip member"))?;
            let path = member
                .enclosed_name()
                .ok_or_else(|| invalid("escaping tool zip member"))?;
            if path.file_name() != Path::new(&spec.member).file_name() {
                continue;
            }
            if selected.is_some()
                || member.is_dir()
                || member
                    .unix_mode()
                    .is_some_and(|mode| !matches!(mode & 0o170000, 0 | 0o100000))
            {
                return Err(invalid(
                    "tool binary member missing, ambiguous or nonregular",
                ));
            }
            let size = member.size();
            selected = Some(member_digest(&mut member, size, spec.binary_limit()?)?);
        }
    } else {
        return Err(invalid(
            "binary source provenance requires a single-executable archive",
        ));
    }
    selected.ok_or_else(|| invalid("installed diagnostic source binary member missing"))
}
fn verify_provenance(target: &Path, spec: &ToolSpec) -> Result<(), HostError> {
    let (archive, receipt) = provenance_paths(target)?;
    no_symlinks(&archive)?;
    no_symlinks(&receipt)?;
    if !archive.is_file() || !receipt.is_file() {
        return Err(invalid(
            "diagnostic executable lacks authenticated source provenance; explicitly remove the unreceipted installation and reinstall",
        ));
    }
    let mut bytes = Vec::new();
    File::open(&receipt)?.take(16385).read_to_end(&mut bytes)?;
    if bytes.len() > 16384 {
        return Err(invalid("diagnostic provenance receipt exceeds allowance"));
    }
    let record: BinaryReceipt = serde_json::from_slice(&bytes)
        .map_err(|_| invalid("invalid diagnostic provenance receipt"))?;
    if record.schema_version != 1
        || record.version != spec.version
        || record.archive_sha256 != spec.sha256
        || record.source != spec.url
        || record.member != spec.member
        || !hash_valid(&record.binary_sha256)
    {
        return Err(invalid(
            "diagnostic provenance receipt differs from exact pin",
        ));
    }
    let extracted = archive_binary_digest(&archive, spec)?;
    if extracted != record.binary_sha256 || digest(target)? != extracted {
        return Err(invalid(
            "diagnostic executable differs from authenticated archive member",
        ));
    }
    Ok(())
}
fn publish_provenance(
    archive: &Path,
    target: &Path,
    staged: &Path,
    spec: &ToolSpec,
) -> Result<(), HostError> {
    let (source, receipt) = provenance_paths(target)?;
    no_symlinks(&source)?;
    no_symlinks(&receipt)?;
    File::open(archive)?.sync_all()?;
    match fs::hard_link(archive, &source) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            if digest(&source)? != spec.sha256 {
                return Err(invalid(
                    "another source archive occupies diagnostic installation",
                ));
            }
        }
        Err(error) => return Err(error.into()),
    }
    let record = BinaryReceipt {
        schema_version: 1,
        version: spec.version.clone(),
        archive_sha256: spec.sha256.clone(),
        binary_sha256: digest(staged)?,
        source: spec.url.clone(),
        member: spec.member.clone(),
    };
    let bytes =
        serde_json::to_vec(&record).map_err(|_| invalid("cannot encode diagnostic provenance"))?;
    let mut temporary = tempfile::NamedTempFile::new_in(
        target
            .parent()
            .ok_or_else(|| invalid("tool parent missing"))?,
    )?;
    std::io::Write::write_all(&mut temporary, &bytes)?;
    temporary.as_file().sync_all()?;
    match temporary.persist_noclobber(&receipt) {
        Ok(_) => {}
        Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => {
            let mut existing = Vec::new();
            File::open(&receipt)?
                .take(16385)
                .read_to_end(&mut existing)?;
            if existing != bytes {
                return Err(invalid(
                    "another provenance receipt occupies diagnostic installation",
                ));
            }
        }
        Err(error) => return Err(error.error.into()),
    }
    Ok(())
}
/// Verify an installed tool without installation or execution.
pub fn verify(root: &Path, tool: Tool, environment: &Environment) -> Result<PathBuf, HostError> {
    let spec = tool_spec(root, tool)?;
    let target = binary_path(root, tool, environment)?;
    verify_at(&target, &spec)?;
    Ok(target)
}
fn copy_bounded(
    source: &mut impl Read,
    target: &Path,
    size: u64,
    limit: u64,
    executable: bool,
) -> Result<(), HostError> {
    if size > limit {
        return Err(invalid("tool member exceeds expanded allowance"));
    }
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(if executable { 0o755 } else { 0o644 })
        .open(target)?;
    let copied = std::io::copy(&mut source.take(limit.saturating_add(1)), &mut output)?;
    if copied != size {
        return Err(invalid("tool member size differs or exceeds allowance"));
    }
    output.sync_all()?;
    Ok(())
}
fn stage_tar(archive: &Path, scratch: &Path, spec: &ToolSpec) -> Result<PathBuf, HostError> {
    let portable = spec.archive.as_deref() == Some("portable-tar");
    let root = scratch.join("bundle");
    fs::create_dir(&root)?;
    let staged = scratch.join("tool");
    let limits = tar_limits(spec)?;
    let mut bundle = tar::Archive::new(GzDecoder::new(File::open(archive)?).take(limits.decoded));
    let mut names = BTreeSet::new();
    let mut hashes = BTreeMap::new();
    let mut expanded = 0_u64;
    let mut found = false;
    for (index, entry) in bundle.entries()?.enumerate() {
        let mut entry = entry?;
        let name = entry.path()?.into_owned();
        let kind = entry.header().entry_type();
        account_tar_entry(index, entry.size(), &mut expanded, &limits)?;
        if portable {
            if !relative(&name) || !names.insert(name.clone()) {
                return Err(invalid("duplicate or escaping tool archive member"));
            }
            if !(kind.is_file() || kind.is_dir()) {
                return Err(invalid("portable archive contains a nonregular member"));
            }
            let relative = name
                .strip_prefix(
                    spec.prefix
                        .as_deref()
                        .ok_or_else(|| invalid("portable prefix missing"))?,
                )
                .map_err(|_| invalid("portable archive escapes declared prefix"))?;
            let target = root.join(relative);
            if kind.is_dir() {
                fs::create_dir_all(&target)?;
                continue;
            }
            if relative.as_os_str().is_empty() {
                return Err(invalid("portable release root must be a directory"));
            }
            fs::create_dir_all(
                target
                    .parent()
                    .ok_or_else(|| invalid("tool member parent missing"))?,
            )?;
            let size = entry.size();
            let mode = entry.header().mode()?;
            copy_bounded(&mut entry, &target, size, 64 << 20, mode & 0o111 != 0)?;
            hashes.insert(
                relative
                    .to_str()
                    .ok_or_else(|| invalid("non-UTF8 portable member"))?
                    .to_owned(),
                digest(&target)?,
            );
        } else if name.file_name() == Path::new(&spec.member).file_name() {
            if found || !kind.is_file() {
                return Err(invalid(
                    "tool binary member missing, ambiguous or nonregular",
                ));
            }
            let size = entry.size();
            copy_bounded(&mut entry, &staged, size, spec.binary_limit()?, true)?;
            found = true;
        }
    }
    if bundle.into_inner().limit() == 0 {
        return Err(invalid("tool archive exceeds decoded stream allowance"));
    }
    if portable {
        let receipt = BundleReceipt {
            version: spec.version.clone(),
            archive_sha256: spec.sha256.clone(),
            source: spec.url.clone(),
            members: hashes,
        };
        fs::write(
            root.join("bundle.json"),
            serde_json::to_vec(&receipt)
                .map_err(|_| invalid("cannot serialize portable receipt"))?,
        )?;
        verify_bundle(&root, spec)?;
        Ok(root)
    } else if found {
        Ok(staged)
    } else {
        Err(invalid("tool binary member missing"))
    }
}
fn stage_zip(archive: &Path, scratch: &Path, spec: &ToolSpec) -> Result<PathBuf, HostError> {
    let mut bundle = zip::ZipArchive::new(File::open(archive)?)
        .map_err(|_| invalid("invalid tool zip archive"))?;
    let staged = scratch.join("tool");
    let mut found = false;
    for index in 0..bundle.len() {
        let mut member = bundle
            .by_index(index)
            .map_err(|_| invalid("invalid tool zip member"))?;
        let path = member
            .enclosed_name()
            .ok_or_else(|| invalid("escaping tool zip member"))?;
        if path.file_name() != Path::new(&spec.member).file_name() {
            continue;
        }
        if found
            || member.is_dir()
            || member
                .unix_mode()
                .is_some_and(|mode| !matches!(mode & 0o170000, 0 | 0o100000))
        {
            return Err(invalid(
                "tool binary member missing, ambiguous or nonregular",
            ));
        }
        let size = member.size();
        copy_bounded(&mut member, &staged, size, spec.binary_limit()?, true)?;
        found = true;
    }
    if found {
        Ok(staged)
    } else {
        Err(invalid("tool binary member missing"))
    }
}
fn publish(
    archive: &Path,
    target: &Path,
    spec: &ToolSpec,
    scratch: &Path,
) -> Result<(), HostError> {
    if fs::metadata(archive)?.len() > spec.archive_limit()? || digest(archive)? != spec.sha256 {
        return Err(invalid(
            "tool release archive allowance or checksum differs",
        ));
    }
    let portable = spec.archive.as_deref() == Some("portable-tar");
    let staged = if spec.archive.is_some() {
        stage_tar(archive, scratch, spec)?
    } else {
        stage_zip(archive, scratch, spec)?
    };
    if portable {
        let selected = target
            .ancestors()
            .nth(Path::new(&spec.member).components().count())
            .ok_or_else(|| invalid("portable root missing"))?;
        match rustix::fs::renameat_with(
            rustix::fs::CWD,
            &staged,
            rustix::fs::CWD,
            selected,
            rustix::fs::RenameFlags::NOREPLACE,
        ) {
            Ok(()) => {}
            Err(rustix::io::Errno::EXIST) => verify_bundle(selected, spec)?,
            Err(error) => return Err(std::io::Error::from(error).into()),
        }
    } else {
        verify_plain_binary(&staged, spec)?;
        if spec.binary_sha256.is_none() {
            publish_provenance(archive, target, &staged, spec)?;
        }
        match fs::hard_link(&staged, target) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                verify_at(target, spec)?;
                if digest(target)? != digest(&staged)? {
                    return Err(invalid(
                        "incompatible diagnostic installation already occupies path",
                    ));
                }
            }
            Err(error) => return Err(error.into()),
        }
    }
    verify_at(target, spec)?;
    File::open(
        target
            .parent()
            .ok_or_else(|| invalid("tool installation parent missing"))?,
    )?
    .sync_all()?;
    Ok(())
}
/// Explicit HTTPS download, bounded decode and no-replace publication of the exact pin.
pub fn install(root: &Path, tool: Tool, environment: &Environment) -> Result<PathBuf, HostError> {
    let spec = tool_spec(root, tool)?;
    let target = binary_path(root, tool, environment)?;
    if fs::symlink_metadata(&target).is_ok() {
        verify_at(&target, &spec)?;
        return Ok(target);
    }
    let selected = if spec.archive.as_deref() == Some("portable-tar") {
        target
            .ancestors()
            .nth(Path::new(&spec.member).components().count())
            .ok_or_else(|| invalid("portable root missing"))?
    } else {
        &target
    };
    let parent = selected
        .parent()
        .ok_or_else(|| invalid("tool installation parent missing"))?;
    no_symlinks(parent)?;
    fs::create_dir_all(parent)?;
    no_symlinks(parent)?;
    let scratch = tempfile::Builder::new()
        .prefix(".install-")
        .tempdir_in(parent)?;
    let archive = scratch.path().join("release.archive");
    let status = Command::new("curl")
        .env_clear()
        .envs(environment)
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--location",
            "--proto",
            "=https",
            "--proto-redir",
            "=https",
            "--max-time",
            "60",
            "--max-filesize",
            &spec.archive_limit()?.to_string(),
            "--user-agent",
            "pse-arrow-diagnostics",
            "--output",
        ])
        .arg(&archive)
        .arg(&spec.url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    if !status.success() {
        return Err(invalid(
            "diagnostic tool HTTPS download failed; verify network and curl availability",
        ));
    }
    publish(&archive, &target, &spec, scratch.path())?;
    Ok(target)
}

#[cfg(test)]
#[path = "diagnostic_tools_tests.rs"]
mod tests;
