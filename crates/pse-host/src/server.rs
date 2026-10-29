// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit native construction of the canonical source server.
//!
//! This capability establishes executable identity only. Application composition owns
//! schema applicability, authentication, migrations and permission to open writes.
use crate::{HostError, HostManager, environment::Environment};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::{OsStr, OsString},
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
};

mod release_import;
use release_import::{import_inner, origin_complete};

const VERSION: &str = "3.3.4";
const REVISION: &str = "1ac2b1dba7cd177621a4cf5d507caebeb2414395";
const ORIGIN: &str = "https://github.com/surrealdb/surrealdb.git";
const ENGINE: &str = "1.0.0";
const SURREALISM: &str = "0.6.4";
const FEATURES: &str = "storage-mem,cli,mcp,surrealism";
const BUILD_CONFIG: &str = "profile.release.panic=\"unwind\"";
const DISKANN_SHA256: &str = "3ead115d8a4f3917c1ae9c3db0aa4dd56251841864ef8637329969f8e5145dd3";
const PATCHES: &[&str] = &[
    "mx-1.0-minimal.patch", "mx-1.0-controls.patch", "grpc-connection-ownership.patch",
    "pinned-compiler.patch", "canonical-profile.patch", "grpc-session-admission.patch",
    "grpc-status-details.patch", "api-failure-rollback.patch", "surrealism-locked-build.patch",
    "surrealism-build-features.patch", "surrealism-global-controllers.patch",
    "surrealism-memory-count.patch", "surrealism-controller-deadline.patch",
];
const BUILD_ENV: &[&str] = &[
    "HOME", "PATH", "USER", "LOGNAME", "TMPDIR", "LANG", "LC_ALL", "TZ", "CARGO_HOME",
    "RUSTUP_HOME", "CARGO_BUILD_JOBS", "CARGO_BUILD_TARGET", "CARGO_INCREMENTAL",
    "CARGO_TERM_COLOR", "RUSTC", "RUSTDOC", "RUSTC_WRAPPER", "RUSTFLAGS", "RUSTDOCFLAGS",
    "CARGO_ENCODED_RUSTFLAGS", "CARGO_ENCODED_RUSTDOCFLAGS", "CC", "CXX", "AR", "RANLIB",
    "CMAKE_PREFIX_PATH", "CMAKE_TOOLCHAIN_FILE", "CMAKE_GENERATOR", "LIBCLANG_PATH",
    "CLANG_PATH", "LLVM_CONFIG_PATH", "LLVM_DIR", "LIBRARY_PATH", "LD_LIBRARY_PATH",
    "PKG_CONFIG_PATH", "PKG_CONFIG_LIBDIR", "PKG_CONFIG_SYSROOT_DIR", "BINDGEN_EXTRA_CLANG_ARGS",
    "PSE_LLVM_PREFIX", "PSE_SCCACHE_BINARY", "SCCACHE_DIR", "SCCACHE_CACHE_SIZE",
    "SCCACHE_MAX_FRAME_LENGTH", "SCCACHE_CONF", "SCCACHE_SERVER_UDS", "SCCACHE_CLIENT_SIDE",
    "SCCACHE_DIRECT", "SCCACHE_NO_DAEMON", "PSE_NATIVE_COMPILER_CACHE", "SOURCE_DATE_EPOCH",
];
const NATIVE_PATHS: &[&str] = &[
    "CMAKE_PREFIX_PATH", "CMAKE_TOOLCHAIN_FILE", "LIBCLANG_PATH", "LLVM_DIR", "LIBRARY_PATH",
    "LD_LIBRARY_PATH", "PKG_CONFIG_PATH", "PKG_CONFIG_LIBDIR", "PKG_CONFIG_SYSROOT_DIR",
    "PSE_LLVM_PREFIX", "SCCACHE_CONF",
];

fn invalid(message: impl Into<String>) -> HostError {
    HostError::Ownership(format!("server preparation: {}", message.into()))
}
fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, HostError> {
    serde_json::to_vec(value).map_err(|error| invalid(format!("cannot encode record: {error}")))
}
fn read_record<T: for<'a> Deserialize<'a>>(path: &Path) -> Result<T, HostError> {
    regular(path)?;
    serde_json::from_slice(&fs::read(path)?)
        .map_err(|error| invalid(format!("invalid {}: {error}", path.display())))
}
fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}
fn digest(path: &Path) -> Result<String, HostError> {
    let mut file = File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; 65536];
    loop {
        let size = file.read(&mut buffer)?;
        if size == 0 { break; }
        hash.update(&buffer[..size]);
    }
    Ok(hash.finalize().iter().map(|byte| format!("{byte:02x}")).collect())
}
fn text<'a>(environment: &'a Environment, name: &str) -> Result<Option<&'a str>, HostError> {
    environment.get(OsStr::new(name)).map(|value| {
        value.to_str().ok_or_else(|| invalid(format!("{name} must be UTF-8")))
    }).transpose()
}
fn regular(path: &Path) -> Result<(), HostError> {
    for ancestor in path.ancestors() {
        if fs::symlink_metadata(ancestor)?.file_type().is_symlink() {
            return Err(invalid(format!("artifact path traverses a symlink: {}", path.display())));
        }
    }
    Ok(())
}
fn private_directory(path: &Path) -> Result<(), HostError> {
    if let Some(parent) = path.parent() {
        if !parent.exists() { private_directory(parent)?; }
    }
    if !path.exists() { fs::create_dir(path)?; }
    regular(path)?;
    let metadata = fs::metadata(path)?;
    if !metadata.is_dir() || metadata.uid() != rustix::process::geteuid().as_raw() {
        return Err(invalid("server cache must be a directory owned by the current user"));
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}
fn profile_root(environment: &Environment) -> Result<PathBuf, HostError> {
    let root = if let Some(root) = environment.get(OsStr::new("PSE_SURREAL_TOOLS")) {
        PathBuf::from(root)
    } else {
        let data = if let Some(data) = environment.get(OsStr::new("XDG_DATA_HOME")) {
            PathBuf::from(data)
        } else {
            PathBuf::from(environment.get(OsStr::new("HOME"))
                .ok_or_else(|| invalid("HOME or PSE_SURREAL_TOOLS is required"))?)
                .join(".local/share")
        };
        data.join("pse-arrow/tools/surreal")
    };
    if !root.is_absolute() { return Err(invalid("server tools root must be absolute")); }
    Ok(root.join("rust-v1"))
}
fn hexadecimal(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[derive(Clone, Debug)]
/// A selected immutable executable; this is not a schema-ready database capability.
pub struct PreparedServer {
    generation: PathBuf,
    recipe: String,
}
impl PreparedServer {
    /// Native executable validated at construction. Carry this capability into launch.
    pub fn executable(&self) -> PathBuf { self.generation.join("surreal") }
    /// Immutable installation directory; legacy installations are never modified.
    pub fn generation(&self) -> &Path { &self.generation }
    /// Exact prepared source and build identity.
    pub fn recipe(&self) -> &str { &self.recipe }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceIdentity {
    version: String,
    revision: String,
    origin: String,
    engine: String,
    surrealism: String,
    features: String,
    profile: String,
    build_config: String,
    patches: BTreeMap<String, String>,
    diskann_archive: String,
    diskann_patch: String,
    metadata: String,
    platform: String,
}
impl SourceIdentity {
    fn supported(&self) -> bool {
        self.version == VERSION && self.revision == REVISION && self.origin == ORIGIN
            && self.engine == ENGINE && self.surrealism == SURREALISM && self.features == FEATURES
            && self.profile == "release" && self.build_config == BUILD_CONFIG
            && self.diskann_archive == DISKANN_SHA256 && hexadecimal(&self.diskann_patch, 64)
            && self.patches.len() == PATCHES.len()
            && PATCHES.iter().all(|name| self.patches.get(*name).is_some_and(|value| hexadecimal(value, 64)))
            && self.metadata == metadata(&self.patches)
            && self.platform == format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
    }
}
fn metadata(patches: &BTreeMap<String, String>) -> String {
    // Preserve the original recipe's sorted Python JSON spelling in version metadata.
    let pairs: Vec<_> = patches.iter().map(|(name, value)| format!("\"{name}\": \"{value}\"")).collect();
    format!("pse.mx.{ENGINE}.{}", &hash(format!("{{{}}}", pairs.join(", ")).as_bytes())[..16])
}
fn source_identity(root: &Path) -> Result<SourceIdentity, HostError> {
    let inputs = root.join("scripts/surrealdb-source");
    let mut patches = BTreeMap::new();
    for name in PATCHES { patches.insert((*name).to_owned(), digest(&inputs.join(name))?); }
    Ok(SourceIdentity {
        version: VERSION.into(), revision: REVISION.into(), origin: ORIGIN.into(),
        engine: ENGINE.into(), surrealism: SURREALISM.into(), features: FEATURES.into(),
        profile: "release".into(), build_config: BUILD_CONFIG.into(), metadata: metadata(&patches), patches,
        diskann_archive: DISKANN_SHA256.into(), diskann_patch: digest(&inputs.join("diskann-0.56.0.patch"))?,
        platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
    })
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Tool { path: PathBuf, sha256: String }
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BuildContext {
    root: PathBuf,
    target: Option<String>,
    configuration: BTreeMap<PathBuf, Option<String>>,
    toolchain_declaration: String,
    tools: BTreeMap<String, Tool>,
    toolchain: BTreeMap<String, String>,
    native: BTreeMap<PathBuf, String>,
    environment: BTreeMap<String, String>,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Identity { source: SourceIdentity, origin: ConstructionOrigin }
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum ConstructionOrigin {
    Built { context: Box<BuildContext> },
    ImportedRelease { release_sha256: String },
}
impl Identity {
    fn context(&self) -> Result<&BuildContext, HostError> {
        match &self.origin {
            ConstructionOrigin::Built { context } => Ok(context),
            ConstructionOrigin::ImportedRelease { .. } => Err(invalid("imported releases have original provenance, not a current build context")),
        }
    }
    #[cfg(test)]
    fn context_mut(&mut self) -> &mut BuildContext {
        match &mut self.origin {
            ConstructionOrigin::Built { context } => context,
            ConstructionOrigin::ImportedRelease { .. } => panic!("fixture requires a built identity"),
        }
    }
}
impl BuildContext {
    fn complete(&self) -> bool {
        self.root.is_absolute() && hexadecimal(&self.toolchain_declaration, 64)
            && self.target.as_ref().is_none_or(|target| target == &format!("{}-unknown-linux-gnu", std::env::consts::ARCH))
            && !self.configuration.is_empty() && !self.native.is_empty()
            && ["cargo", "rustc", "rustdoc", "git", "curl", "tar", "CC", "CXX", "CLANG_PATH"].iter().all(|name| self.tools.contains_key(*name))
            && ["cargo", "rustc", "rustdoc"].iter().all(|name| self.toolchain.get(*name).is_some_and(|value| !value.is_empty()))
            && ["HOME", "PATH"].iter().all(|name| self.environment.contains_key(*name))
            && self.tools.values().all(|tool| tool.path.is_absolute() && hexadecimal(&tool.sha256, 64))
            && self.native.values().all(|value| hexadecimal(value, 64))
            && self.environment.values().all(|value| hexadecimal(value, 64))
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt { schema: u32, identity: Identity, files: BTreeMap<String, String> }
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pointer { schema: u32, recipe: String, generation: String, receipt_sha256: String }

fn resolve(name: &OsStr, root: &Path, environment: &Environment) -> Result<PathBuf, HostError> {
    let path = Path::new(name);
    let choices = if path.components().count() > 1 {
        vec![if path.is_absolute() { path.to_path_buf() } else { root.join(path) }]
    } else {
        std::env::split_paths(environment.get(OsStr::new("PATH"))
            .ok_or_else(|| invalid("PATH is required"))?).map(|directory| directory.join(path)).collect()
    };
    for path in choices {
        if path.is_file() && path.metadata()?.permissions().mode() & 0o111 != 0 {
            return Ok(path.canonicalize()?);
        }
    }
    Err(invalid(format!("required native executable unavailable: {}", name.to_string_lossy())))
}
fn capture(program: &Path, arguments: &[OsString], root: &Path, environment: &Environment) -> Result<String, HostError> {
    let output = Command::new(program).args(arguments).current_dir(root).env_clear().envs(environment).output()?;
    if !output.status.success() {
        return Err(invalid(format!("{} failed with {}: {}", program.display(), output.status,
            String::from_utf8_lossy(&output.stderr).chars().take(12000).collect::<String>())));
    }
    String::from_utf8(output.stdout).map_err(|_| invalid("tool response is not UTF-8"))
}
fn args(values: &[&str]) -> Vec<OsString> { values.iter().map(OsString::from).collect() }
fn tree(path: &Path) -> Result<String, HostError> {
    fn visit(path: &Path, entries: &mut BTreeMap<PathBuf, String>, visited: &mut BTreeSet<PathBuf>) -> Result<(), HostError> {
        if !path.exists() {
            entries.insert(path.to_path_buf(), "absent".into());
            return Ok(());
        }
        let resolved = path.canonicalize()?;
        if path.is_file() {
            entries.insert(path.to_path_buf(), format!("{}:{}", resolved.display(), digest(&resolved)?));
        } else if path.is_dir() {
            entries.insert(path.to_path_buf(), format!("directory:{}", resolved.display()));
            if visited.insert(resolved) {
                let mut children = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
                children.sort_by_key(fs::DirEntry::file_name);
                for child in children { visit(&child.path(), entries, visited)?; }
            }
        } else { return Err(invalid("unsupported special native build input")); }
        Ok(())
    }
    let mut entries = BTreeMap::new();
    visit(path, &mut entries, &mut BTreeSet::new())?;
    Ok(hash(&encode(&entries)?))
}
fn controlled_environment(caller: &Environment) -> Result<Environment, HostError> {
    for name in ["CARGO_TARGET_DIR", "PSE_CARGO_TARGET_DIR", "RUSTC_WORKSPACE_WRAPPER", "CFLAGS", "CXXFLAGS", "FFLAGS", "CPPFLAGS", "LDFLAGS", "FC", "SDKROOT", "MACOSX_DEPLOYMENT_TARGET"] {
        if text(caller, name)?.is_some_and(|value| !value.is_empty()) {
            return Err(invalid(format!("unsupported external server build control {name}")));
        }
    }
    for name in caller.keys() {
        let name = name.to_str().ok_or_else(|| invalid("build environment key is not UTF-8"))?;
        if (name.starts_with("CARGO_") || name.starts_with("RUSTC_")) && !BUILD_ENV.contains(&name)
            && !name.starts_with("CARGO_PKG_") && !name.starts_with("CARGO_MANIFEST_") {
            return Err(invalid(format!("unsupported Cargo environment control {name}")));
        }
    }
    if text(caller, "RUSTC_WRAPPER")?.is_some_and(|value| !value.is_empty() && Path::new(value).file_name() != Some(OsStr::new("sccache"))) {
        return Err(invalid("server preparation requires the native sccache compiler wrapper"));
    }
    Ok(caller.iter().filter(|(name, _)| name.to_str().is_some_and(|name| BUILD_ENV.contains(&name)))
        .map(|(name, value)| (name.clone(), value.clone())).collect())
}
fn flags(arguments: &[String]) -> Result<(), HostError> {
    let mut index = 0;
    while index < arguments.len() {
        let argument = &arguments[index];
        if ["-C", "--cfg", "--check-cfg", "-Z"].contains(&argument.as_str()) {
            index += 1;
            let value = arguments.get(index).ok_or_else(|| invalid("incomplete compiler flag"))?;
            if (argument == "-C" && !["link-arg=-fuse-ld=mold", "link-arg=-fuse-ld=bfd", "link-arg=-fuse-ld=lld"].contains(&value.as_str())
                && !["opt-level=", "debuginfo=", "codegen-units=", "panic=unwind", "lto=", "target-feature="].iter().any(|prefix| value.starts_with(prefix)))
                || (argument == "-Z" && !value.starts_with("threads=")) { return Err(invalid("unsupported compiler input")); }
        } else if !["-Copt-level=", "-Cdebuginfo=", "--cfg=", "--check-cfg=", "-Zthreads="].iter().any(|prefix| argument.starts_with(prefix)) {
            return Err(invalid("unsupported compiler input"));
        }
        if argument.contains('@') || argument.contains("target-cpu") || argument.contains("sysroot") || argument.contains("plugin") {
            return Err(invalid("unsupported external compiler input"));
        }
        index += 1;
    }
    Ok(())
}
fn configuration(root: &Path, environment: &Environment) -> Result<(BTreeMap<PathBuf, Option<String>>, Vec<(OsString, PathBuf)>), HostError> {
    let home = environment.get(OsStr::new("CARGO_HOME")).map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(environment.get(OsStr::new("HOME")).cloned().unwrap_or_default()).join(".cargo"));
    let mut locations = vec![home];
    let mut parents: Vec<_> = root.ancestors().map(|parent| parent.join(".cargo")).collect();
    parents.reverse();
    locations.extend(parents);
    let mut records = BTreeMap::new();
    let mut tools = Vec::new();
    for directory in locations {
        let selected = if directory.join("config").is_file() { directory.join("config") } else { directory.join("config.toml") };
        if !selected.exists() { records.insert(selected, None); continue; }
        let document: toml::Value = toml::from_str(&fs::read_to_string(&selected)?).map_err(|error| invalid(format!("invalid Cargo configuration: {error}")))?;
        let table = document.as_table().ok_or_else(|| invalid("Cargo configuration must be a table"))?;
        for (name, settings) in table {
            let allowed: &[&str] = match name.as_str() {
                "alias" => continue,
                "build" => &["jobs", "rustc", "rustdoc", "rustc-wrapper", "rustc-workspace-wrapper", "rustflags", "rustdocflags", "incremental", "fingerprint"],
                "unstable" => &["feature-unification", "fine-grain-locking", "checksum-freshness"],
                "resolver" => &["feature-unification"],
                "future-incompat-report" => &["frequency"],
                "target" => &[],
                _ => return Err(invalid(format!("unsupported Cargo configuration {name}"))),
            };
            let settings = settings.as_table().ok_or_else(|| invalid("invalid Cargo configuration table"))?;
            let groups = if name == "target" {
                settings.values().map(|value| value.as_table().ok_or_else(|| invalid("invalid Cargo target table"))).collect::<Result<Vec<_>, _>>()?
            } else { vec![settings] };
            for group in groups {
                let allowed = if name == "target" { &["linker", "rustflags", "rustdocflags"][..] } else { allowed };
                for (key, value) in group {
                    if !allowed.contains(&key.as_str()) { return Err(invalid(format!("unsupported Cargo configuration {name}.{key}"))); }
                    if ["linker", "rustc", "rustdoc", "rustc-wrapper", "rustc-workspace-wrapper"].contains(&key.as_str()) {
                        let program = value.as_str().ok_or_else(|| invalid("configured tool must be a string"))?;
                        if key.contains("wrapper") && !program.is_empty() && Path::new(program).file_name() != Some(OsStr::new("sccache")) {
                            return Err(invalid("non-native configured compiler wrapper refused"));
                        }
                        if !program.is_empty() { tools.push((program.into(), directory.parent().unwrap_or(root).to_path_buf())); }
                    }
                    if key == "rustflags" || key == "rustdocflags" {
                        let values: Vec<String> = if let Some(value) = value.as_str() { value.split_whitespace().map(str::to_owned).collect() }
                            else { value.as_array().ok_or_else(|| invalid("invalid compiler flag array"))?.iter().map(|value| value.as_str().map(str::to_owned).ok_or_else(|| invalid("invalid compiler flag"))).collect::<Result<Vec<_>, _>>()? };
                        flags(&values)?;
                    }
                }
            }
        }
        records.insert(selected, Some(hash(&encode(&document)?)));
    }
    Ok((records, tools))
}
fn context(root: &Path, environment: &Environment) -> Result<BuildContext, HostError> {
    let (configuration, configured) = configuration(root, environment)?;
    let mut tools = BTreeMap::new();
    let mut toolchain = BTreeMap::new();
    let rustup = resolve(OsStr::new("rustup"), root, environment)?;
    let declaration = fs::read_to_string(root.join("rust-toolchain.toml"))?;
    let declared: toml::Value = toml::from_str(&declaration).map_err(|error| invalid(format!("invalid toolchain declaration: {error}")))?;
    let channel = declared.get("toolchain").and_then(|value| value.get("channel")).and_then(toml::Value::as_str)
        .ok_or_else(|| invalid("toolchain declaration has no channel"))?;
    let active = capture(&rustup, &args(&["show", "active-toolchain"]), root, environment)?;
    if !active.split_whitespace().next().is_some_and(|active| active.starts_with(&format!("{channel}-"))) {
        return Err(invalid("server construction requires the declared pinned toolchain"));
    }
    tools.insert("rustup".into(), Tool { path: rustup.clone(), sha256: digest(&rustup)? });
    for name in ["cargo", "rustc", "rustdoc"] {
        let path = PathBuf::from(capture(&rustup, &args(&["which", name]), root, environment)?.trim());
        let path = path.canonicalize()?;
        toolchain.insert(name.into(), capture(&path, &args(&["--version", "--verbose"]), root, environment)?);
        tools.insert(name.into(), Tool { sha256: digest(&path)?, path });
    }
    for (name, default) in [("CC", "cc"), ("CXX", "c++"), ("AR", "ar"), ("RANLIB", "ranlib"), ("CLANG_PATH", "clang"), ("LLVM_CONFIG_PATH", "llvm-config")] {
        let path = resolve(environment.get(OsStr::new(name)).map(OsString::as_os_str).unwrap_or(OsStr::new(default)), root, environment)?;
        tools.insert(name.into(), Tool { sha256: digest(&path)?, path });
    }
    for name in ["RUSTC", "RUSTDOC", "RUSTC_WRAPPER", "PSE_SCCACHE_BINARY", "PSE_NATIVE_COMPILER_CACHE"] {
        if let Some(value) = environment.get(OsStr::new(name)).filter(|value| !value.is_empty()) {
            let path = resolve(value, root, environment)?;
            tools.insert(name.into(), Tool { sha256: digest(&path)?, path });
        }
    }
    for (index, (name, base)) in configured.iter().enumerate() {
        let path = resolve(name, base, environment)?;
        tools.insert(format!("config-{index}"), Tool { sha256: digest(&path)?, path });
    }
    for name in ["mold", "ld", "ld.lld", "pkg-config", "cmake", "make", "ninja", "git", "sh", "bash", "env", "perl", "clang++", "curl", "tar", "ldd"] {
        if let Ok(path) = resolve(OsStr::new(name), root, environment) {
            tools.insert(name.into(), Tool { sha256: digest(&path)?, path });
        }
    }
    for name in ["git", "curl", "tar", "ldd"] {
        if !tools.contains_key(name) { return Err(invalid(format!("{name} is required for source preparation"))); }
    }
    for name in ["RUSTFLAGS", "RUSTDOCFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_ENCODED_RUSTDOCFLAGS"] {
        if let Some(value) = text(environment, name)? {
            let arguments: Vec<String> = if name.starts_with("CARGO_ENCODED_") { value.split('\u{1f}').map(str::to_owned).collect() }
                else { value.split_whitespace().map(str::to_owned).collect() };
            flags(&arguments)?;
        }
    }
    let mut roots = BTreeSet::from([
        PathBuf::from("/etc/ld.so.cache"), PathBuf::from("/etc/ld.so.conf"), PathBuf::from("/etc/ld.so.conf.d"),
        PathBuf::from("/usr/include"), PathBuf::from("/usr/local/include"), PathBuf::from("/usr/local/lib"),
        PathBuf::from("/usr/lib/gcc"), PathBuf::from("/usr/lib64"),
        PathBuf::from(if std::env::consts::ARCH == "x86_64" { "/usr/lib/x86_64-linux-gnu" } else { "/usr/lib/aarch64-linux-gnu" }),
    ]);
    roots.insert(PathBuf::from(capture(&tools["rustc"].path, &args(&["--print", "sysroot"]), root, environment)?.trim()));
    let resource = PathBuf::from(capture(&tools["CLANG_PATH"].path, &args(&["-print-resource-dir"]), root, environment)?.trim());
    if !resource.is_absolute() || !resource.join("include/stddef.h").is_file() { return Err(invalid("selected Clang resource headers are unavailable")); }
    if let Some(value) = text(environment, "BINDGEN_EXTRA_CLANG_ARGS")?.filter(|value| !value.is_empty()) {
        if value != format!("-resource-dir={}", resource.display()) { return Err(invalid("only selected Clang resource-dir is supported for bindgen flags")); }
    }
    roots.insert(resource);
    if let Some(value) = environment.get(OsStr::new("CMAKE_TOOLCHAIN_FILE")) {
        let selected = PathBuf::from(value);
        if selected != root.join(".config/native-cache.cmake") { return Err(invalid("only the selected native cache CMake toolchain is supported")); }
        regular(&selected)?;
    }
    for name in NATIVE_PATHS {
        if let Some(value) = environment.get(OsStr::new(name)) {
            for path in std::env::split_paths(value).filter(|path| !path.as_os_str().is_empty()) {
                roots.insert(if path.is_absolute() { path } else { root.join(path) });
            }
        }
    }
    // Capture actual loader-resolved tool dependencies without unstable ASLR addresses.
    for tool in tools.values() {
        let output = Command::new(&tools["ldd"].path).arg(&tool.path).current_dir(root).env_clear().envs(environment).output()?;
        let linked = String::from_utf8_lossy(&output.stdout);
        if linked.contains("not found") || !matches!(output.status.code(), Some(0 | 1)) { return Err(invalid("cannot close native build-tool loader dependencies")); }
        for word in linked.split_whitespace().filter(|word| word.starts_with('/')) {
            let path = PathBuf::from(word);
            if path.is_file() { roots.insert(path); }
        }
    }
    let mut native = BTreeMap::new();
    for path in roots { native.insert(path.clone(), tree(&path)?); }
    let mut inputs = BTreeMap::new();
    for (name, value) in environment {
        let name = name.to_str().ok_or_else(|| invalid("invalid build environment key"))?;
        let value = value.to_str().ok_or_else(|| invalid("invalid build environment value"))?;
        inputs.insert(name.to_owned(), hash(value.as_bytes()));
    }
    let target = text(environment, "CARGO_BUILD_TARGET")?.filter(|value| !value.is_empty()).map(str::to_owned);
    Ok(BuildContext { root: root.to_path_buf(), target, configuration, toolchain_declaration: hash(declaration.as_bytes()), tools, toolchain, native, environment: inputs })
}
fn run_logged(program: &Path, arguments: &[OsString], root: &Path, environment: &Environment, log: &Path) -> Result<(), HostError> {
    let mut output = OpenOptions::new().create(true).append(true).mode(0o600).open(log)?;
    let status = Command::new(program).args(arguments).current_dir(root).env_clear().envs(environment)
        .stdout(Stdio::from(output.try_clone()?)).stderr(Stdio::from(output.try_clone()?)).status()?;
    output.sync_all()?;
    if !status.success() {
        let mut tail = File::open(log)?;
        tail.seek(SeekFrom::Start(tail.metadata()?.len().saturating_sub(12000)))?;
        let mut bytes = Vec::new();
        tail.read_to_end(&mut bytes)?;
        return Err(invalid(format!("{} failed with {status}; {}: {}", program.display(), log.display(), String::from_utf8_lossy(&bytes))));
    }
    output.flush()?;
    Ok(())
}
fn atomic_record<T: Serialize>(path: &Path, value: &T) -> Result<(), HostError> {
    let parent = path.parent().ok_or_else(|| invalid("record has no parent"))?;
    let mut pending = tempfile::NamedTempFile::new_in(parent)?;
    pending.write_all(&encode(value)?)?;
    pending.as_file().sync_all()?;
    pending.persist(path).map_err(|error| HostError::Io(error.error))?;
    File::open(parent)?.sync_all()?;
    Ok(())
}
fn file_map(root: &Path) -> Result<BTreeMap<String, String>, HostError> {
    fn visit(root: &Path, path: &Path, files: &mut BTreeMap<String, String>) -> Result<(), HostError> {
        let metadata = fs::symlink_metadata(path)?;
        if metadata.file_type().is_symlink() { return Err(invalid("source/artifact tree contains a symlink")); }
        if metadata.is_dir() {
            for entry in fs::read_dir(path)? { visit(root, &entry?.path(), files)?; }
        } else if metadata.is_file() {
            let name = path.strip_prefix(root).map_err(|_| invalid("artifact escapes its root"))?.to_str().ok_or_else(|| invalid("artifact name is not UTF-8"))?;
            files.insert(name.to_owned(), digest(path)?);
        } else { return Err(invalid("source/artifact tree contains a special file")); }
        Ok(())
    }
    let mut files = BTreeMap::new();
    visit(root, root, &mut files)?;
    Ok(files)
}
fn archive_names(listing: &str, verbose: &str) -> Result<(), HostError> {
    let names: Vec<_> = listing.lines().collect();
    let kinds: Vec<_> = verbose.lines().collect();
    if names.is_empty() || names.len() != kinds.len() { return Err(invalid("DiskANN archive listing is incomplete")); }
    for (name, kind) in names.iter().zip(kinds) {
        let path = Path::new(name);
        if !kind.starts_with(['-', 'd']) || !name.starts_with("diskann-0.56.0/")
            || path.components().any(|part| !matches!(part, Component::Normal(_))) {
            return Err(invalid("DiskANN archive contains an unexpected source entry"));
        }
    }
    Ok(())
}
fn acquire(root: &Path, work: &Path, identity: &Identity, environment: &Environment) -> Result<(), HostError> {
    let source = work.join("source");
    if source.exists() { return verify_source(root, work, identity, environment); }
    let scratch = tempfile::Builder::new().prefix("acquire-").tempdir_in(work)?;
    let pending = scratch.path().join("source");
    let git = &identity.context()?.tools["git"].path;
    let log = work.join("build.log");
    let run = |program: &Path, arguments: Vec<OsString>| run_logged(program, &arguments, root, environment, &log);
    run(git, vec!["init".into(), pending.clone().into_os_string()])?;
    run(git, vec!["-C".into(), pending.clone().into_os_string(), "fetch".into(), "--depth".into(), "1".into(), ORIGIN.into(), REVISION.into()])?;
    run(git, vec!["-C".into(), pending.clone().into_os_string(), "checkout".into(), "--detach".into(), "FETCH_HEAD".into()])?;
    for name in PATCHES {
        run(git, vec!["-C".into(), pending.clone().into_os_string(), "apply".into(), "--index".into(), root.join("scripts/surrealdb-source").join(name).into_os_string()])?;
    }
    let archive = scratch.path().join("diskann-0.56.0.crate");
    run(&identity.context()?.tools["curl"].path, vec!["--fail".into(), "--location".into(), "--proto".into(), "=https".into(), "--output".into(), archive.clone().into_os_string(), "https://static.crates.io/crates/diskann/diskann-0.56.0.crate".into()])?;
    if digest(&archive)? != DISKANN_SHA256 { return Err(invalid("DiskANN archive checksum differs from pinned source")); }
    let tar = &identity.context()?.tools["tar"].path;
    let listing = capture(tar, &[OsString::from("-tzf"), archive.clone().into_os_string()], root, environment)?;
    let verbose = capture(tar, &[OsString::from("-tvzf"), archive.clone().into_os_string()], root, environment)?;
    archive_names(&listing, &verbose)?;
    let dependencies = pending.join(".pse-deps");
    fs::create_dir(&dependencies)?;
    run(tar, vec!["-xzf".into(), archive.into_os_string(), "--no-same-owner".into(), "--no-same-permissions".into(), "-C".into(), dependencies.clone().into_os_string()])?;
    fs::rename(dependencies.join("diskann-0.56.0"), dependencies.join("diskann"))?;
    run(git, vec!["-C".into(), pending.clone().into_os_string(), "apply".into(), "--directory=.pse-deps/diskann".into(), root.join("scripts/surrealdb-source/diskann-0.56.0.patch").into_os_string()])?;
    let indexed = capture(git, &[OsString::from("-C"), pending.clone().into_os_string(), "diff".into(), "--cached".into(), "--name-only".into()], root, environment)?;
    let mut files = BTreeMap::new();
    for name in indexed.lines() {
        if Path::new(name).components().any(|part| !matches!(part, Component::Normal(_))) { return Err(invalid("invalid patched source path")); }
        files.insert(name.to_owned(), digest(&pending.join(name))?);
    }
    atomic_record(&work.join("source-files.json"), &files)?;
    atomic_record(&work.join("dependency-files.json"), &file_map(&dependencies.join("diskann"))?)?;
    fs::rename(pending, &source)?;
    verify_source(root, work, identity, environment)
}
fn verify_source(root: &Path, work: &Path, identity: &Identity, environment: &Environment) -> Result<(), HostError> {
    let source = work.join("source");
    regular(&source)?;
    let git = &identity.context()?.tools["git"].path;
    let command = |arguments: &[&str]| {
        let mut selected = vec![OsString::from("-C"), source.clone().into_os_string()];
        selected.extend(args(arguments));
        capture(git, &selected, root, environment)
    };
    if command(&["rev-parse", "HEAD"])?.trim() != REVISION || !command(&["diff", "--name-only"])?.trim().is_empty() {
        return Err(invalid("acquired source revision or worktree changed"));
    }
    let mut expected = BTreeSet::new();
    for name in PATCHES {
        for line in fs::read_to_string(root.join("scripts/surrealdb-source").join(name))?.lines() {
            if let Some(name) = line.strip_prefix("+++ b/") { expected.insert(name.to_owned()); }
        }
    }
    let indexed: BTreeSet<_> = command(&["diff", "--cached", "--name-only"])?.lines().map(str::to_owned).collect();
    let recorded: BTreeMap<String, String> = read_record(&work.join("source-files.json"))?;
    if indexed != expected || recorded.keys().cloned().collect::<BTreeSet<_>>() != expected {
        return Err(invalid("acquired index differs from the owned patch set"));
    }
    for (name, expected) in recorded {
        let path = source.join(name);
        regular(&path)?;
        if digest(&path)? != expected { return Err(invalid("patched source bytes changed")); }
    }
    let dependencies: BTreeMap<String, String> = read_record(&work.join("dependency-files.json"))?;
    if file_map(&source.join(".pse-deps/diskann"))? != dependencies { return Err(invalid("checked DiskANN source bytes changed")); }
    let manifest = fs::read_to_string(source.join("Cargo.toml"))?;
    let lock = fs::read_to_string(source.join("Cargo.lock"))?;
    if !manifest.contains("surrealmx = \"=1.0.0\"") || !lock.contains("name = \"surrealmx\"\nversion = \"1.0.0\"")
        || !lock.contains(&format!("name = \"surrealism-runtime\"\nversion = \"{SURREALISM}\"")) {
        return Err(invalid("source does not retain the pinned MX/Surrealism resolution"));
    }
    Ok(())
}
fn seal(path: &Path) -> Result<(), HostError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() { return Err(invalid("immutable generation contains a symlink")); }
    if metadata.is_dir() { for entry in fs::read_dir(path)? { seal(&entry?.path())?; } }
    fs::set_permissions(path, fs::Permissions::from_mode(metadata.permissions().mode() & !0o222))?;
    Ok(())
}
fn immutable(path: &Path) -> Result<(), HostError> {
    regular(path)?;
    let metadata = fs::metadata(path)?;
    if metadata.uid() != rustix::process::geteuid().as_raw() || metadata.permissions().mode() & 0o222 != 0 {
        return Err(invalid("selected server generation is writable or has another owner"));
    }
    if metadata.is_dir() { for entry in fs::read_dir(path)? { immutable(&entry?.path())?; } }
    Ok(())
}
fn validate(generation: &Path, recipe: &str, expected: Option<&Identity>) -> Result<Receipt, HostError> {
    immutable(generation)?;
    let receipt: Receipt = read_record(&generation.join(".complete.json"))?;
    if receipt.schema != 1 || !receipt.identity.source.supported() || !origin_complete(generation, &receipt.identity, &receipt.files)? || hash(&encode(&receipt.identity)?) != recipe
        || expected.is_some_and(|expected| expected != &receipt.identity)
        || receipt.files.keys().map(String::as_str).collect::<BTreeSet<_>>() != match &receipt.identity.origin {
            ConstructionOrigin::Built { .. } => BTreeSet::from(["surreal", "Cargo.lock"]),
            ConstructionOrigin::ImportedRelease { .. } => BTreeSet::from(["surreal", "Cargo.lock", "release.json"]),
        }
        || generation.join("surreal").metadata()?.permissions().mode() & 0o111 == 0 {
        return Err(invalid("selected server identity is incomplete or unsupported"));
    }
    let mut files = file_map(generation)?;
    files.remove(".complete.json");
    if files != receipt.files { return Err(invalid("selected server artifact bytes changed")); }
    Ok(receipt)
}
fn publish_pointer(profile: &Path, generation: &PreparedServer) -> Result<(), HostError> {
    let name = generation.generation.file_name().and_then(OsStr::to_str).ok_or_else(|| invalid("invalid server generation name"))?;
    atomic_record(&profile.join("current.json"), &Pointer { schema: 1, recipe: generation.recipe.clone(), generation: name.into(), receipt_sha256: digest(&generation.generation.join(".complete.json"))? })
}
/// Reopen an exact generation at an external process boundary without following
/// the mutable current selection or consulting source inputs.
pub fn reopen_generation(generation: &Path, recipe: &str) -> Result<PreparedServer, HostError> {
    let parent = generation.parent();
    if !generation.is_absolute()
        || generation.components().any(|part| !matches!(part, Component::RootDir | Component::Normal(_)))
        || !hexadecimal(recipe, 64)
        || !generation.file_name().and_then(OsStr::to_str).is_some_and(|name| hexadecimal(name, 32))
        || parent.and_then(Path::file_name) != Some(OsStr::new(recipe))
        || parent.and_then(Path::parent).and_then(Path::file_name) != Some(OsStr::new("generations"))
    {
        return Err(invalid("invalid exact server generation identity"));
    }
    validate(generation, recipe, None)?;
    Ok(PreparedServer { generation: generation.to_path_buf(), recipe: recipe.to_owned() })
}
/// Select a completed immutable executable without running tools or reading source inputs.
/// Missing or corrupt selection fails closed; it never builds or chooses a fallback.
pub fn select(environment: &Environment) -> Result<PreparedServer, HostError> {
    let profile = profile_root(environment)?;
    let pointer: Pointer = read_record(&profile.join("current.json"))
        .map_err(|error| invalid(format!("no valid selected server; explicitly run server prepare: {error}")))?;
    if pointer.schema != 1 || !hexadecimal(&pointer.recipe, 64) || !hexadecimal(&pointer.generation, 32)
        || !hexadecimal(&pointer.receipt_sha256, 64) { return Err(invalid("invalid server selection")); }
    let generation = profile.join("generations").join(&pointer.recipe).join(pointer.generation);
    regular(&generation)?;
    if digest(&generation.join(".complete.json"))? != pointer.receipt_sha256 { return Err(invalid("server receipt differs from atomic selection")); }
    validate(&generation, &pointer.recipe, None)?;
    Ok(PreparedServer { generation, recipe: pointer.recipe })
}
fn cargo_arguments(source: &Path) -> Vec<OsString> {
    vec!["build".into(), "--manifest-path".into(), source.join("Cargo.toml").into_os_string(),
        "--locked".into(), "--no-default-features".into(), "--features".into(), FEATURES.into(),
        "--profile".into(), "release".into(), "--bin".into(), "surreal".into(), "--config".into(), BUILD_CONFIG.into()]
}
fn compiled_artifact(work: &Path, identity: &Identity) -> Result<PathBuf, HostError> {
    let target = work.join("source/target");
    let target = match &identity.context()?.target { Some(triple) => target.join(triple), None => target };
    Ok(target.join("release/surreal"))
}
fn compile(root: &Path, work: &Path, identity: &Identity, environment: &Environment) -> Result<(), HostError> {
    if text(environment, "CARGO_BUILD_TARGET")?.filter(|value| !value.is_empty()) != identity.context()?.target.as_deref() {
        return Err(invalid("Cargo target differs from the selected build request"));
    }
    let source = work.join("source");
    verify_source(root, work, identity, environment)?;
    let request = work.join("build-request.json");
    let target = source.join("target");
    if target.exists() {
        regular(&target)?;
        let previous: Identity = read_record(&request)?;
        if &previous != identity { return Err(invalid("existing server target has a different build request; target preserved")); }
    }
    atomic_record(&request, identity)?;
    let mut selected = environment.clone();
    selected.insert("SURREAL_BUILD_VERSION".into(), VERSION.into());
    selected.insert("SURREAL_BUILD_METADATA".into(), identity.source.metadata.clone().into());
    run_logged(&identity.context()?.tools["cargo"].path, &cargo_arguments(&source), root, &selected, &work.join("build.log"))?;
    let binary = compiled_artifact(work, identity)?;
    regular(&binary)?;
    let version = capture(&binary, &args(&["version"]), root, &selected)?;
    if !version.contains(VERSION) || !version.contains(&identity.source.metadata) {
        return Err(invalid("built executable version differs from selected source"));
    }
    capture(&binary, &args(&["module", "build", "--help"]), root, &selected)?;
    verify_source(root, work, identity, environment)
}
fn publish(profile: &Path, work: &Path, identity: &Identity) -> Result<PreparedServer, HostError> {
    let staging = profile.join("staging");
    private_directory(&staging)?;
    let scratch = tempfile::Builder::new().prefix("publish-").tempdir_in(staging)?;
    let install = scratch.path().join("install");
    fs::create_dir(&install)?;
    for (source, name) in [(compiled_artifact(work, identity)?, "surreal"), (work.join("source/Cargo.lock"), "Cargo.lock")] {
        regular(&source)?;
        fs::copy(source, install.join(name))?;
        File::open(install.join(name))?.sync_all()?;
    }
    publish_install(profile, &install, identity)
}
fn publish_install(profile: &Path, install: &Path, identity: &Identity) -> Result<PreparedServer, HostError> {
    let recipe = hash(&encode(identity)?);
    let generations = profile.join("generations").join(&recipe);
    private_directory(&generations)?;
    let generation = uuid::Uuid::now_v7().simple().to_string();
    let destination = generations.join(&generation);
    fs::set_permissions(install.join("surreal"), fs::Permissions::from_mode(0o555))?;
    atomic_record(&install.join(".complete.json"), &Receipt { schema: 1, identity: identity.clone(), files: file_map(&install)? })?;
    seal(&install)?;
    // Directory relocation needs write access on its root until the rename completes.
    fs::set_permissions(&install, fs::Permissions::from_mode(0o700))?;
    fs::rename(&install, &destination)?;
    fs::set_permissions(&destination, fs::Permissions::from_mode(0o500))?;
    File::open(&generations)?.sync_all()?;
    validate(&destination, &recipe, Some(identity))?;
    let prepared = PreparedServer { generation: destination, recipe };
    publish_pointer(profile, &prepared)?;
    Ok(prepared)
}
fn prepare_inner(root: &Path, caller: &Environment) -> Result<PreparedServer, HostError> {
    if std::env::consts::OS != "linux" || !matches!(std::env::consts::ARCH, "x86_64" | "aarch64") {
        return Err(invalid("source server construction supports native Linux x86-64/arm64 only"));
    }
    let root = root.canonicalize()?;
    let environment = controlled_environment(caller)?;
    if text(&environment, "CARGO_BUILD_TARGET")?.is_some_and(|value| !value.is_empty() && value != format!("{}-unknown-linux-gnu", std::env::consts::ARCH)) {
        return Err(invalid("cross-compilation is outside the native source server profile"));
    }
    let identity = Identity { source: source_identity(&root)?, origin: ConstructionOrigin::Built { context: Box::new(context(&root, &environment)?) } };
    if !identity.context()?.complete() { return Err(invalid("server construction context is incomplete")); }
    let recipe = hash(&encode(&identity)?);
    let profile = profile_root(caller)?;
    private_directory(&profile)?;
    let locks = profile.join("locks");
    private_directory(&locks)?;
    let lock_path = locks.join(&recipe);
    if lock_path.exists() { regular(&lock_path)?; }
    let lock = OpenOptions::new().create(true).truncate(false).read(true).write(true).mode(0o600).open(lock_path)?;
    lock.lock()?;
    if let Ok(current) = select(caller) {
        if current.recipe == recipe { return Ok(current); }
    }
    let generations = profile.join("generations").join(&recipe);
    if generations.exists() {
        regular(&generations)?;
        let mut candidates = fs::read_dir(&generations)?.collect::<Result<Vec<_>, _>>()?;
        candidates.sort_by_key(|entry| std::cmp::Reverse(entry.file_name()));
        for entry in candidates {
            if entry.file_name().to_str().is_some_and(|name| hexadecimal(name, 32)) && validate(&entry.path(), &recipe, Some(&identity)).is_ok() {
                let prepared = PreparedServer { generation: entry.path(), recipe };
                publish_pointer(&profile, &prepared)?;
                return Ok(prepared);
            }
        }
    }
    let mut work = profile.join("builds").join(&recipe);
    private_directory(&work)?;
    if work.join("source").exists() && (verify_source(&root, &work, &identity, &environment).is_err()
        || (work.join("source/target").exists() && read_record::<Identity>(&work.join("build-request.json")).ok().as_ref() != Some(&identity))) {
        // Explicit preparation may recover in a fresh work directory; preserve the old
        // source/target and its diagnostics rather than repairing unknown mutable input.
        work = profile.join("builds").join(format!("{recipe}-{}", uuid::Uuid::now_v7().simple()));
        private_directory(&work)?;
    }
    acquire(&root, &work, &identity, &environment)?;
    compile(&root, &work, &identity, &environment)?;
    if source_identity(&root)? != identity.source || context(&root, &environment)? != *identity.context()? {
        return Err(invalid("server build premises changed during compilation; no selection published"));
    }
    publish(&profile, &work, &identity)
}
/// Explicitly construct the pinned server inside authenticated native containment.
/// Once work starts, its blocking join is awaited; caller containment owns physical drain.
pub async fn prepare(root: &Path, environment: &Environment) -> Result<PreparedServer, HostError> {
    authenticated_preparation().await?;
    let root = root.to_path_buf();
    let environment = environment.clone();
    tokio::task::spawn_blocking(move || prepare_inner(&root, &environment)).await
        .map_err(|error| invalid(format!("server preparation worker failed: {error}")))?
}
async fn authenticated_preparation() -> Result<(), HostError> {
    let manager = HostManager::connect_user(None).await?;
    let current = manager.current_unit().await?;
    if !manager.native_workload(&current).await? {
        return Err(invalid("explicit server preparation requires scripts/pse-env containment"));
    }
    Ok(())
}
/// Explicitly snapshot an admitted source release, preserving its original provenance.
/// This imports executable identity only; it neither starts nor adopts a database.
pub async fn prepare_from_release(root: &Path, environment: &Environment, release: &Path) -> Result<PreparedServer, HostError> {
    authenticated_preparation().await?;
    let root = root.to_path_buf();
    let environment = environment.clone();
    let release = release.to_path_buf();
    tokio::task::spawn_blocking(move || import_inner(&root, &environment, &release)).await
        .map_err(|error| invalid(format!("server release import worker failed: {error}")))?
}
/// Narrow standalone construction/selection surface. Service startup belongs to composition.
pub fn run_cli(arguments: &[OsString]) -> Result<i32, HostError> {
    if arguments == [OsString::from("--help")] || arguments == [OsString::from("-h")] {
        writeln!(std::io::stdout(), "server prepare [--from-release PATH] | select\nExplicit preparation builds the pinned native server or imports a verified source release; selection never builds.\nThis executable capability does not establish schema readiness or write admission.")?;
        return Ok(0);
    }
    let environment: Environment = std::env::vars_os().collect();
    let selected = if arguments == [OsString::from("select")] { select(&environment)? }
    else if arguments == [OsString::from("prepare")]
        || (arguments.len() == 3 && arguments[0] == "prepare" && arguments[1] == "--from-release") {
        let mut root = std::env::current_dir()?;
        while !root.join(".config/diagnostic-tools.toml").is_file() {
            if !root.pop() { return Err(invalid("run explicit preparation inside the selected checkout")); }
        }
        let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
        if arguments.len() == 3 {runtime.block_on(prepare_from_release(&root, &environment, Path::new(&arguments[2])))?}
        else {runtime.block_on(prepare(&root, &environment))?}
    } else { return Err(invalid("expected server prepare [--from-release PATH] or server select")); };
    writeln!(std::io::stdout(), "{}", selected.executable().display())?;
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture {
        directory: tempfile::TempDir,
        root: PathBuf,
        profile: PathBuf,
        work: PathBuf,
        identity: Identity,
        environment: Environment,
    }
    fn executable(path: &Path, contents: &str) {
        fs::write(path, contents).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    fn restore(path: &Path) {
        if let Ok(metadata) = fs::symlink_metadata(path) {
            if metadata.is_dir() {
                fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
                for entry in fs::read_dir(path).unwrap() { restore(&entry.unwrap().path()); }
            }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) { restore(self.directory.path()); }
    }
    fn fixture() -> Fixture {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("checkout");
        fs::create_dir(&root).unwrap();
        let inputs = root.join("scripts/surrealdb-source");
        fs::create_dir_all(&inputs).unwrap();
        for name in PATCHES { fs::write(inputs.join(name), "--- a/Cargo.toml\n+++ b/Cargo.toml\n").unwrap(); }
        fs::write(inputs.join("diskann-0.56.0.patch"), "fixture DiskANN patch").unwrap();
        let source = source_identity(&root).unwrap();
        let profile = directory.path().join("tools/rust-v1");
        private_directory(&profile).unwrap();
        let work = profile.join("builds/fixture");
        private_directory(&work).unwrap();
        let acquired = work.join("source");
        fs::create_dir_all(acquired.join(".pse-deps/diskann")).unwrap();
        fs::write(acquired.join("Cargo.toml"), "surrealmx = \"=1.0.0\"\n").unwrap();
        fs::write(acquired.join("Cargo.lock"), "name = \"surrealmx\"\nversion = \"1.0.0\"\nname = \"surrealism-runtime\"\nversion = \"0.6.4\"\n").unwrap();
        fs::write(acquired.join(".pse-deps/diskann/lib.rs"), "fixture checked dependency").unwrap();
        atomic_record(&work.join("source-files.json"), &BTreeMap::from([("Cargo.toml", digest(&acquired.join("Cargo.toml")).unwrap())])).unwrap();
        atomic_record(&work.join("dependency-files.json"), &file_map(&acquired.join(".pse-deps/diskann")).unwrap()).unwrap();
        let git = root.join("git-fixture");
        executable(&git, &format!("#!/bin/sh\nif [ \"$3\" = rev-parse ]; then printf '%s\\n' {REVISION}; elif [ \"$3\" = diff ] && [ \"$4\" = --cached ]; then printf '%s\\n' Cargo.toml; fi\n"));
        let cargo = root.join("cargo-fixture");
        executable(&cargo, &format!("#!/bin/sh\nset -eu\nmkdir -p \"$PSE_SERVER_BUILD/source/target/release\"\ncat > \"$PSE_SERVER_BUILD/source/target/release/surreal\" <<'BINARY'\n#!/bin/sh\nif [ \"$1\" = version ]; then printf '%s\\n' 'surreal {VERSION}+{}'; fi\nBINARY\nchmod 755 \"$PSE_SERVER_BUILD/source/target/release/surreal\"\n", source.metadata));
        let mut context = BuildContext { root: root.clone(), target: None, configuration: BTreeMap::from([(root.join(".cargo/config.toml"), None)]), toolchain_declaration: "a".repeat(64), tools: BTreeMap::from([
            ("git".into(), Tool { sha256: digest(&git).unwrap(), path: git }),
            ("cargo".into(), Tool { sha256: digest(&cargo).unwrap(), path: cargo }),
        ]), toolchain: BTreeMap::from([("cargo".into(), "fixture Cargo".into()), ("rustc".into(), "fixture rustc".into()), ("rustdoc".into(), "fixture rustdoc".into())]), native: BTreeMap::from([(root.clone(), "b".repeat(64))]), environment: BTreeMap::from([("HOME".into(), "c".repeat(64)), ("PATH".into(), "d".repeat(64))]) };
        for name in ["rustc", "rustdoc", "curl", "tar", "CC", "CXX", "CLANG_PATH"] { context.tools.insert(name.into(), context.tools["git"].clone()); }
        let mut environment = Environment::new();
        environment.insert("PATH".into(), "/usr/bin:/bin".into());
        environment.insert("PSE_SURREAL_TOOLS".into(), profile.parent().unwrap().into());
        environment.insert("PSE_SERVER_BUILD".into(), work.clone().into());
        Fixture { directory, root, profile, work, identity: Identity { source, origin: ConstructionOrigin::Built { context: Box::new(context) } }, environment }
    }
    fn prepared(fixture: &Fixture) -> PreparedServer {
        compile(&fixture.root, &fixture.work, &fixture.identity, &fixture.environment).unwrap();
        publish(&fixture.profile, &fixture.work, &fixture.identity).unwrap()
    }
    #[test]
    fn server_native_command_build_publishes_immutable_source_free_selection() {
        let fixture = fixture();
        let prepared = prepared(&fixture);
        assert!(prepared.executable().is_file());
        assert_eq!(prepared.recipe(), hash(&encode(&fixture.identity).unwrap()));
        immutable(prepared.generation()).unwrap();
        fs::remove_dir_all(&fixture.root).unwrap();
        let selected = select(&fixture.environment).unwrap();
        assert_eq!(selected.generation(), prepared.generation());
        assert!(capture(&selected.executable(), &args(&["version"]), fixture.directory.path(), &fixture.environment).unwrap().contains(VERSION));
    }
    #[test]
    fn server_reopen_retains_exact_generation_without_current_selection_or_sources() {
        let fixture = fixture();
        let prepared = prepared(&fixture);
        fs::remove_file(fixture.profile.join("current.json")).unwrap();
        fs::remove_dir_all(&fixture.root).unwrap();
        let reopened = reopen_generation(prepared.generation(), prepared.recipe()).unwrap();
        assert_eq!(reopened.executable(), prepared.executable());
        assert!(reopen_generation(prepared.generation(), &"0".repeat(64)).is_err());
        assert!(reopen_generation(&prepared.generation().join(".."), prepared.recipe()).is_err());
        let executable = prepared.executable();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(reopen_generation(prepared.generation(), prepared.recipe()).is_err());
    }
    #[test]
    fn server_failed_native_build_preserves_selection_and_diagnostics() {
        let fixture = fixture();
        let prepared = prepared(&fixture);
        let before = fs::read(fixture.profile.join("current.json")).unwrap();
        executable(&fixture.identity.context().unwrap().tools["cargo"].path, "#!/bin/sh\nprintf '%s\\n' 'fixture compiler error' >&2\nexit 42\n");
        let error = compile(&fixture.root, &fixture.work, &fixture.identity, &fixture.environment).unwrap_err().to_string();
        assert!(error.contains("42"), "{error}");
        assert!(error.contains("fixture compiler error"));
        assert_eq!(fs::read(fixture.profile.join("current.json")).unwrap(), before);
        assert_eq!(select(&fixture.environment).unwrap().generation(), prepared.generation());
        assert!(fixture.work.join("build.log").is_file());
    }
    #[test]
    fn server_native_target_selects_its_artifact_instead_of_stale_host_output() {
        let mut fixture = fixture();
        let triple = format!("{}-unknown-linux-gnu", std::env::consts::ARCH);
        fixture.environment.insert("CARGO_BUILD_TARGET".into(), triple.clone().into());
        fixture.identity.context_mut().target = Some(triple);
        atomic_record(&fixture.work.join("build-request.json"), &fixture.identity).unwrap();
        let stale = fixture.work.join("source/target/release/surreal");
        fs::create_dir_all(stale.parent().unwrap()).unwrap();
        executable(&stale, "#!/bin/sh\nprintf 'stale host output\\n'\n");
        executable(&fixture.work.join("image"), &format!("#!/bin/sh\nprintf '%s\\n' 'surreal {VERSION}+{}'\n", fixture.identity.source.metadata));
        executable(&fixture.identity.context().unwrap().tools["cargo"].path, "#!/bin/sh\nset -eu\nout=\"$PSE_SERVER_BUILD/source/target/$CARGO_BUILD_TARGET/release\"\nmkdir -p \"$out\"\ncp \"$PSE_SERVER_BUILD/image\" \"$out/surreal\"\n");
        let selected = prepared(&fixture);
        assert_eq!(fs::read(selected.executable()).unwrap(), fs::read(fixture.work.join("image")).unwrap());
        assert_ne!(fs::read(selected.executable()).unwrap(), fs::read(stale).unwrap());
    }
    #[test]
    fn server_changed_source_or_target_request_refuses_before_native_build() {
        let fixture = fixture();
        compile(&fixture.root, &fixture.work, &fixture.identity, &fixture.environment).unwrap();
        let log = fs::read(fixture.work.join("build.log")).unwrap();
        fs::write(fixture.work.join("build-request.json"), b"{}").unwrap();
        assert!(compile(&fixture.root, &fixture.work, &fixture.identity, &fixture.environment).is_err());
        fs::write(fixture.work.join("source/Cargo.toml"), b"unowned change").unwrap();
        assert!(compile(&fixture.root, &fixture.work, &fixture.identity, &fixture.environment).is_err());
        assert_eq!(fs::read(fixture.work.join("build.log")).unwrap(), log);
        assert!(!fixture.profile.join("current.json").exists());
    }
    #[test]
    fn server_publication_failure_preserves_completed_generation_without_selection() {
        let fixture = fixture();
        compile(&fixture.root, &fixture.work, &fixture.identity, &fixture.environment).unwrap();
        fs::create_dir(fixture.profile.join("current.json")).unwrap();
        assert!(publish(&fixture.profile, &fixture.work, &fixture.identity).is_err());
        assert!(select(&fixture.environment).is_err());
        let recipe = hash(&encode(&fixture.identity).unwrap());
        let generations = fs::read_dir(fixture.profile.join("generations").join(&recipe)).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(generations.len(), 1);
        validate(&generations[0].path(), &recipe, Some(&fixture.identity)).unwrap();
    }
    #[test]
    fn server_corrupt_pointer_explicit_republication_repairs_without_deleting_generations() {
        let fixture = fixture();
        let prepared = prepared(&fixture);
        fs::write(fixture.profile.join("current.json"), b"interrupted foreign selection").unwrap();
        assert!(select(&fixture.environment).is_err());
        // Same publication used by explicit preparation's valid-generation reuse.
        publish_pointer(&fixture.profile, &prepared).unwrap();
        assert_eq!(select(&fixture.environment).unwrap().generation(), prepared.generation());
        assert!(fixture.work.join("source/target/release/surreal").is_file());
    }
    #[test]
    fn server_corrupt_or_writable_artifact_refuses_without_fallback() {
        let fixture = fixture();
        let prepared = prepared(&fixture);
        let binary = prepared.executable();
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(select(&fixture.environment).is_err());
        fs::write(&binary, b"changed executable").unwrap();
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o555)).unwrap();
        assert!(select(&fixture.environment).is_err());
        assert_eq!(fs::read_dir(fixture.profile.join("generations")).unwrap().count(), 1);
    }
    #[test]
    fn server_archive_entries_refuse_links_traversal_and_unexpected_roots() {
        archive_names("diskann-0.56.0/\ndiskann-0.56.0/Cargo.toml\n", "drwxr-xr-x directory\n-rw-r--r-- file\n").unwrap();
        for (names, types) in [
            ("diskann-0.56.0/../outside\n", "-rw-r--r-- file\n"),
            ("diskann-0.56.0/link\n", "lrwxrwxrwx link\n"),
            ("diskann-0.56.0/hard\n", "hrwxrwxrwx hard\n"),
            ("/diskann-0.56.0/file\n", "-rw-r--r-- file\n"),
            ("another/file\n", "-rw-r--r-- file\n"),
        ] { assert!(archive_names(names, types).is_err()); }
    }
    #[test]
    fn server_retained_recipe_keeps_patches_features_lock_and_unwind() {
        let fixture = fixture();
        assert_eq!(fixture.identity.source.patches.len(), 13);
        assert!(fixture.identity.source.supported());
        let arguments = cargo_arguments(&fixture.work.join("source"));
        assert!(arguments.contains(&OsString::from("--locked")));
        assert!(arguments.contains(&OsString::from("--no-default-features")));
        assert!(arguments.contains(&OsString::from(FEATURES)));
        assert!(arguments.contains(&OsString::from(BUILD_CONFIG)));
        assert!(!arguments.iter().any(|argument| argument.to_string_lossy().contains("python")));
    }
    #[test]
    fn server_controlled_environment_removes_unrelated_values_and_refuses_external_builds() {
        let mut environment = Environment::new();
        environment.insert("PATH".into(), "/usr/bin:/bin".into());
        environment.insert("BASH_ENV".into(), "/untrusted/startup".into());
        environment.insert("TOKEN".into(), "private".into());
        let controlled = controlled_environment(&environment).unwrap();
        assert!(!controlled.contains_key(OsStr::new("TOKEN")));
        assert!(!controlled.contains_key(OsStr::new("BASH_ENV")));
        for (name, value) in [("RUSTC_WRAPPER", "/repository/python-wrapper"), ("CARGO_TARGET_DIR", "/shared/target"), ("CARGO_BUILD_RUSTFLAGS", "external"), ("CFLAGS", "@external")] {
            let mut selected = environment.clone();
            selected.insert(name.into(), value.into());
            assert!(controlled_environment(&selected).is_err(), "{name}");
        }
    }
    #[test]
    fn server_build_configuration_reads_real_table_documents_and_checks_supported_controls() {
        let fixture = fixture();
        let directory = fixture.root.join(".cargo");
        fs::create_dir(&directory).unwrap();
        let path = directory.join("config.toml");
        fs::write(&path, "# Cargo table document\n[build]\njobs = 32\n[unstable]\nfeature-unification = \"workspace\"\n").unwrap();
        let mut environment = fixture.environment.clone();
        environment.insert("HOME".into(), fixture.root.join("home").into_os_string());
        let (configuration, tools) = configuration(&fixture.root, &environment).unwrap();
        assert!(configuration[&path].is_some());
        assert!(tools.is_empty());
        fs::write(&path, "[build]\ntarget-dir = '/unowned-target'\n").unwrap();
        assert!(super::configuration(&fixture.root, &environment).unwrap_err().to_string().contains("build.target-dir"));
    }
    #[test]
    fn server_build_toolchain_reads_real_table_document_before_enforcing_declared_channel() {
        let fixture = fixture();
        fs::write(fixture.root.join("rust-toolchain.toml"), "# Pinned toolchain table document\n[toolchain]\nchannel = \"fixture-nightly\"\nprofile = \"minimal\"\ncomponents = [\n  \"rust-src\",\n  \"clippy\",\n]\n").unwrap();
        let rustup = fixture.root.join("rustup");
        executable(&rustup, "#!/bin/sh\nprintf '%s\\n' 'another-channel-x86_64-unknown-linux-gnu (default)'\n");
        let mut environment = fixture.environment.clone();
        environment.insert("PATH".into(), fixture.root.clone().into_os_string());
        environment.insert("HOME".into(), fixture.root.join("home").into_os_string());
        let error = match context(&fixture.root, &environment) { Ok(_) => panic!("mismatched toolchain must refuse"), Err(error) => error.to_string() };
        assert!(error.contains("requires the declared pinned toolchain"), "{error}");
    }
}
