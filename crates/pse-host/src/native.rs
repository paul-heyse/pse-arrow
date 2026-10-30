// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit native provider preparation and source-free immutable generation selection.
use crate::{HostError, HostManager, environment::Environment};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    process::Command,
};

mod providers;
mod requirements;
pub use providers::{
    capability_names, prepare_capabilities, select_capabilities, validate_capabilities,
};
pub use requirements::requirements_from_backend_features;

const PROFILE: &str = "compiler,klu";
const OPTIONS: &[&str] = &[
    "CMAKE_INSTALL_LIBDIR=lib",
    "CMAKE_BUILD_TYPE=Release",
    "CMAKE_POSITION_INDEPENDENT_CODE=ON",
    "BUILD_SHARED_LIBS=OFF",
    "BUILD_STATIC_LIBS=ON",
    "SUITESPARSE_ENABLE_PROJECTS=suitesparse_config;amd;btf;colamd;klu",
    "KLU_USE_CHOLMOD=OFF",
    "SUITESPARSE_USE_CUDA=OFF",
    "SUITESPARSE_CONFIG_USE_OPENMP=OFF",
    "SUITESPARSE_DEMOS=OFF",
];
const REQUIRED: &[&str] = &[
    "include/suitesparse/klu.h",
    "lib/libklu.a",
    "lib/libamd.a",
    "lib/libbtf.a",
    "lib/libcolamd.a",
    "lib/libsuitesparseconfig.a",
];
const INPUTS: &[&str] = &[
    "CC",
    "CXX",
    "CMAKE",
    "CFLAGS",
    "CXXFLAGS",
    "FC",
    "FFLAGS",
    "CPPFLAGS",
    "LDFLAGS",
    "CMAKE_TOOLCHAIN_FILE",
    "SDKROOT",
    "MACOSX_DEPLOYMENT_TARGET",
    "CARGO_BUILD_TARGET",
    "CLANG_PATH",
    "LIBCLANG_PATH",
    "BINDGEN_EXTRA_CLANG_ARGS",
];
fn invalid(message: impl Into<String>) -> HostError {
    HostError::Ownership(format!("native preparation: {}", message.into()))
}
fn text<'a>(env: &'a Environment, key: &str) -> Result<Option<&'a str>, HostError> {
    env.get(OsStr::new(key))
        .map(|value| {
            value
                .to_str()
                .ok_or_else(|| invalid(format!("{key} must be UTF-8 for native preparation")))
        })
        .transpose()
}
fn json<T: Serialize>(value: &T) -> Result<Vec<u8>, HostError> {
    serde_json::to_vec(value).map_err(|_| invalid("cannot encode native artifact record"))
}
fn read_json<T: for<'a> Deserialize<'a>>(path: &Path) -> Result<T, HostError> {
    let bytes = fs::read(path)?;
    serde_json::from_slice(&bytes)
        .map_err(|_| invalid("invalid native artifact record; run explicit preparation"))
}
fn digest(path: &Path) -> Result<String, HostError> {
    let mut file = File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; 65536];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(hex(&hash.finalize()))
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn hash_bytes(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}
fn resolve_program(name: &OsStr, env: &Environment) -> Result<PathBuf, HostError> {
    let path = Path::new(name);
    let choices = if path.components().count() > 1 {
        vec![path.to_path_buf()]
    } else {
        std::env::split_paths(
            env.get(OsStr::new("PATH"))
                .map(OsString::as_os_str)
                .unwrap_or(OsStr::new("/usr/bin:/bin")),
        )
        .map(|directory| directory.join(path))
        .collect()
    };
    for path in choices {
        if path.is_file() && path.metadata()?.permissions().mode() & 0o111 != 0 {
            return Ok(path.canonicalize()?);
        }
    }
    Err(invalid("required native executable is unavailable"))
}
fn capture(program: &Path, args: &[&OsStr], env: &Environment) -> Result<Vec<u8>, HostError> {
    let output = Command::new(program)
        .args(args)
        .env_clear()
        .envs(env)
        .output()?;
    if !output.status.success() {
        return Err(invalid(format!(
            "tool probe failed for {}",
            program.display()
        )));
    }
    Ok(output.stdout)
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Tool {
    path: PathBuf,
    sha256: String,
    version: String,
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Identity {
    profile: String,
    source: String,
    options: Vec<String>,
    tools: BTreeMap<String, Tool>,
    inputs: BTreeMap<String, Option<String>>,
    selections: BTreeMap<String, String>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    version: u32,
    identity: Identity,
    files: BTreeMap<String, String>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pointer {
    version: u32,
    recipe: String,
    generation: String,
}
fn profile_root(env: &Environment) -> Result<PathBuf, HostError> {
    let root = crate::paths::HostPaths::new(env).native_cache()?;
    if !root.is_absolute() {
        return Err(invalid("native cache root must be absolute"));
    }
    Ok(root.join("rust-v1/kinsol"))
}
fn private_directory(path: &Path) -> Result<(), HostError> {
    if let Ok(metadata) = fs::symlink_metadata(path)
        && (metadata.file_type().is_symlink() || !metadata.is_dir())
    {
        return Err(invalid(
            "native cache directory must not be a symlink or file",
        ));
    }
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            private_directory(parent)?;
        } else if fs::symlink_metadata(parent)?.file_type().is_symlink() {
            return Err(invalid("native cache parent must not be a symlink"));
        }
    }
    fs::create_dir_all(path)?;
    if path.metadata()?.uid() != rustix::process::geteuid().as_raw() {
        return Err(invalid("native cache directory belongs to another user"));
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}
fn file_map(root: &Path) -> Result<BTreeMap<String, String>, HostError> {
    fn walk(
        root: &Path,
        directory: &Path,
        files: &mut BTreeMap<String, String>,
    ) -> Result<(), HostError> {
        let mut entries = fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)?;
            if path == root.join(".complete.json") {
                continue;
            }
            if metadata.is_dir() {
                walk(root, &path, files)?;
            } else if metadata.is_file() || metadata.file_type().is_symlink() {
                let actual = path.canonicalize()?;
                if !actual.starts_with(root.canonicalize()?) || !actual.is_file() {
                    return Err(invalid(
                        "native artifact contains an external or directory symlink",
                    ));
                }
                let name = path
                    .strip_prefix(root)
                    .map_err(|_| invalid("invalid artifact path"))?
                    .to_str()
                    .ok_or_else(|| invalid("native artifact path must be UTF-8"))?
                    .to_owned();
                files.insert(name, digest(&path)?);
            } else {
                return Err(invalid("native artifact contains a special file"));
            }
        }
        Ok(())
    }
    let mut files = BTreeMap::new();
    walk(root, root, &mut files)?;
    Ok(files)
}
fn seal(root: &Path) -> Result<(), HostError> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.is_dir() {
            seal(&path)?;
        } else if metadata.is_file() {
            fs::set_permissions(
                &path,
                fs::Permissions::from_mode(metadata.permissions().mode() & !0o222),
            )?;
        }
    }
    fs::set_permissions(
        root,
        fs::Permissions::from_mode(root.metadata()?.permissions().mode() & !0o222),
    )?;
    Ok(())
}
fn validate(prefix: &Path, identity: Option<&Identity>) -> Result<Receipt, HostError> {
    for path in prefix.ancestors().take(5) {
        if fs::symlink_metadata(path)?.file_type().is_symlink() {
            return Err(invalid("native generation must not traverse a symlink"));
        }
    }
    let record: Receipt = read_json(&prefix.join(".complete.json"))?;
    if record.version != 2
        || record.identity.profile != PROFILE
        || record.identity.options != OPTIONS
        || !component(&record.identity.source, 64)
        || record.identity.inputs.len() != INPUTS.len()
        || INPUTS
            .iter()
            .any(|key| !record.identity.inputs.contains_key(*key))
        || identity.is_some_and(|wanted| wanted != &record.identity)
        || record.files.is_empty()
        || REQUIRED
            .iter()
            .any(|name| !record.files.contains_key(*name))
    {
        return Err(invalid(
            "native artifact identity or required files differ; run explicit preparation",
        ));
    }
    if record.files != file_map(prefix)? {
        return Err(invalid(
            "native artifact bytes differ; run explicit preparation",
        ));
    }
    Ok(record)
}
fn component(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|value| value.is_ascii_hexdigit() && !value.is_ascii_uppercase())
}
fn selected_prefix(profile: &Path) -> Result<(PathBuf, Receipt), HostError> {
    let pointer: Pointer = read_json(&profile.join("current.json")).map_err(|_| invalid("no completed compiler/KLU profile; run scripts/pse-env -- scripts/pse-host native prepare --capabilities compiler,klu"))?;
    if pointer.version != 1
        || !component(&pointer.recipe, 64)
        || !component(&pointer.generation, 32)
    {
        return Err(invalid("invalid native selection"));
    }
    let prefix = profile
        .join("generations")
        .join(&pointer.recipe)
        .join(pointer.generation);
    let receipt = validate(&prefix, None)?;
    if hash_bytes(&json(&receipt.identity)?) != pointer.recipe {
        return Err(invalid(
            "native recipe identity differs from selected recipe",
        ));
    }
    Ok((prefix, receipt))
}
fn publish_pointer(profile: &Path, recipe: &str, generation: &str) -> Result<(), HostError> {
    let mut temporary = tempfile::NamedTempFile::new_in(profile)?;
    temporary.write_all(&json(&Pointer {
        version: 1,
        recipe: recipe.into(),
        generation: generation.into(),
    })?)?;
    temporary.as_file().sync_all()?;
    temporary
        .persist(profile.join("current.json"))
        .map_err(|error| HostError::Io(error.error))?;
    Ok(())
}
fn inputs(env: &Environment) -> Result<BTreeMap<String, Option<String>>, HostError> {
    INPUTS
        .iter()
        .map(|key| Ok(((*key).to_owned(), text(env, key)?.map(str::to_owned))))
        .collect()
}
type CompilerSelection = (BTreeMap<String, Tool>, BTreeMap<String, String>);

fn compiler(root: &Path, env: &Environment) -> Result<CompilerSelection, HostError> {
    let mut tools = BTreeMap::new();
    let mut selected = BTreeMap::new();
    for (key, default) in [
        ("CC", "cc"),
        ("CXX", "c++"),
        ("CMAKE", "cmake"),
        ("CLANG_PATH", "clang"),
    ] {
        let path = resolve_program(
            env.get(OsStr::new(key))
                .map(OsString::as_os_str)
                .unwrap_or(OsStr::new(default)),
            env,
        )?;
        let version = String::from_utf8(capture(&path, &[OsStr::new("--version")], env)?)
            .map_err(|_| invalid("native tool version must be UTF-8"))?;
        selected.insert(
            key.to_owned(),
            path.to_str()
                .ok_or_else(|| invalid("native executable path must be UTF-8"))?
                .to_owned(),
        );
        tools.insert(
            key.to_owned(),
            Tool {
                sha256: digest(&path)?,
                path,
                version,
            },
        );
    }
    for key in ["CC", "CXX"] {
        let target = capture(&tools[key].path, &[OsStr::new("-dumpmachine")], env)?;
        let target = String::from_utf8(target).map_err(|_| invalid("invalid compiler target"))?;
        if !matches!(
            target.trim(),
            "x86_64-linux-gnu" | "x86_64-pc-linux-gnu" | "x86_64-unknown-linux-gnu"
        ) {
            return Err(invalid(
                "compiler/KLU preparation supports only native x86-64 GNU/Linux compilers",
            ));
        }
    }
    let clang = tools["CLANG_PATH"].path.clone();
    let resource = String::from_utf8(capture(&clang, &[OsStr::new("-print-resource-dir")], env)?)
        .map_err(|_| invalid("invalid Clang resource directory"))?;
    let resource = PathBuf::from(resource.trim());
    let header = resource.join("include/stddef.h");
    if !resource.is_absolute() || !header.is_file() {
        return Err(invalid("selected Clang lacks development resource headers"));
    }
    tools.insert(
        "clang-resource".into(),
        Tool {
            path: header.clone(),
            sha256: digest(&header)?,
            version: String::new(),
        },
    );
    let arguments = text(env, "BINDGEN_EXTRA_CLANG_ARGS")?.unwrap_or("");
    let resource_option = format!("-resource-dir={}", resource.display());
    if arguments
        .split_whitespace()
        .any(|argument| argument.contains("resource-dir") && argument != resource_option)
    {
        return Err(invalid(
            "select CLANG_PATH rather than overriding Clang resource-dir",
        ));
    }
    selected.insert(
        "BINDGEN_EXTRA_CLANG_ARGS".into(),
        if arguments
            .split_whitespace()
            .any(|argument| argument == resource_option)
        {
            arguments.to_owned()
        } else {
            format!("{arguments} {resource_option}").trim().to_owned()
        },
    );
    let library = env
        .get(OsStr::new("LIBCLANG_PATH"))
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            clang
                .parent()
                .and_then(Path::parent)
                .unwrap_or(root)
                .join("lib")
        });
    let library_file = if library.is_dir() {
        library.join("libclang.so")
    } else {
        library.clone()
    };
    if !library_file.is_file() {
        return Err(invalid("selected libclang is unavailable"));
    }
    tools.insert(
        "libclang".into(),
        Tool {
            path: library_file.clone(),
            sha256: digest(&library_file)?,
            version: String::new(),
        },
    );
    selected.insert(
        "LIBCLANG_PATH".into(),
        library
            .to_str()
            .ok_or_else(|| invalid("libclang path must be UTF-8"))?
            .to_owned(),
    );
    let key = "CMAKE_TOOLCHAIN_FILE";
    if let Some(path) = env.get(OsStr::new(key)) {
        let path = PathBuf::from(path);
        if !path.is_absolute() {
            return Err(invalid("CMAKE_TOOLCHAIN_FILE must be absolute"));
        }
        tools.insert(
            key.into(),
            Tool {
                sha256: digest(&path)?,
                path,
                version: String::new(),
            },
        );
    }
    Ok((tools, selected))
}
fn cache_bridge(root: &Path, env: &mut Environment) {
    if env
        .get(OsStr::new("RUSTC_WRAPPER"))
        .is_some_and(|value| Path::new(value).file_name() == Some(OsStr::new("sccache")))
    {
        if let Some(wrapper) = env.get(OsStr::new("RUSTC_WRAPPER")).cloned() {
            env.insert("PSE_NATIVE_COMPILER_CACHE".into(), wrapper);
        }
        env.entry("CMAKE_TOOLCHAIN_FILE".into())
            .or_insert_with(|| root.join(".config/native-cache.cmake").into_os_string());
    } else {
        env.remove(OsStr::new("PSE_NATIVE_COMPILER_CACHE"));
    }
}
fn source(root: &Path, env: &Environment) -> Result<PathBuf, HostError> {
    let mut metadata = cargo_metadata::MetadataCommand::new();
    metadata
        .current_dir(root)
        .features(cargo_metadata::CargoOpt::SomeFeatures(vec![
            "pse-runtime/solver-kinsol".into(),
        ]))
        .other_options(vec!["--locked".into()]);
    if let Some(cargo) = env
        .get(OsStr::new("PSE_CORRECTNESS_REAL_CARGO"))
        .or_else(|| env.get(OsStr::new("CARGO")))
    {
        metadata.cargo_path(PathBuf::from(cargo));
    }
    let mut command = metadata.cargo_command();
    command.env_clear().envs(env);
    let output = command.output()?;
    if !output.status.success() {
        return Err(invalid("locked Cargo metadata failed"));
    }
    let output =
        String::from_utf8(output.stdout).map_err(|_| invalid("Cargo metadata is not UTF-8"))?;
    let metadata = cargo_metadata::MetadataCommand::parse(&output)
        .map_err(|_| invalid("invalid Cargo metadata"))?;
    let packages: Vec<_> = metadata
        .packages
        .iter()
        .filter(|package| package.name.as_str() == "suitesparse_sys")
        .collect();
    if packages.len() != 1 {
        return Err(invalid("expected one resolved SuiteSparse source"));
    }
    let path = packages[0]
        .manifest_path
        .parent()
        .ok_or_else(|| invalid("SuiteSparse manifest has no parent"))?
        .join("vendor")
        .into_std_path_buf();
    if !path.join("CMakeLists.txt").is_file() {
        return Err(invalid("resolved SuiteSparse source is unavailable"));
    }
    Ok(path)
}
fn run(program: &Path, args: &[OsString], env: &Environment) -> Result<(), HostError> {
    let status = Command::new(program)
        .args(args)
        .env_clear()
        .envs(env)
        .status()?;
    if !status.success() {
        return Err(invalid(format!("native construction failed with {status}")));
    }
    Ok(())
}
fn preflight(env: &Environment) -> Result<(), HostError> {
    if std::env::consts::OS != "linux"
        || std::env::consts::ARCH != "x86_64"
        || text(env, "CARGO_BUILD_TARGET")?
            .is_some_and(|target| target != "x86_64-unknown-linux-gnu")
    {
        return Err(invalid(
            "compiler/KLU preparation supports only native x86-64 GNU/Linux",
        ));
    }
    Ok(())
}
fn immutable(root: &Path) -> Result<(), HostError> {
    let metadata = fs::symlink_metadata(root)?;
    if metadata.file_type().is_symlink() {
        return Err(invalid("native generation cannot contain mutable symlinks"));
    }
    if metadata.permissions().mode() & 0o222 != 0 {
        return Err(invalid("native generation contains writable paths"));
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(root)? {
            immutable(&entry?.path())?;
        }
    }
    Ok(())
}
fn prepare(root: &Path, env: &Environment) -> Result<PathBuf, HostError> {
    preflight(env)?;
    if env
        .get(OsStr::new("CMAKE_TOOLCHAIN_FILE"))
        .is_some_and(|path| Path::new(path) != root.join(".config/native-cache.cmake"))
    {
        return Err(invalid(
            "custom CMake toolchains are outside the native compiler/KLU profile",
        ));
    }
    for key in [
        "SUITESPARSE_INCLUDE_DIR",
        "SUITESPARSE_LIBRARY_DIR",
        "PSE_KLU_PROVIDER_RECEIPT",
        "SUNDIALS_INCLUDE_DIR",
        "SUNDIALS_LIBRARY_DIR",
    ] {
        if env.contains_key(OsStr::new(key)) {
            return Err(invalid(format!(
                "{key} conflicts with managed compiler/KLU preparation"
            )));
        }
    }
    let profile = profile_root(env)?;
    private_directory(&profile)?;
    let original_inputs = inputs(env)?;
    let (mut tools, selections) = compiler(root, env)?;
    let mut environment = env.clone();
    for (key, value) in &selections {
        environment.insert(key.into(), value.into());
    }
    cache_bridge(root, &mut environment);
    if let Some(path) = environment.get(OsStr::new("CMAKE_TOOLCHAIN_FILE")) {
        let path = PathBuf::from(path);
        tools.insert(
            "effective-cmake-toolchain".into(),
            Tool {
                sha256: digest(&path)?,
                path,
                version: String::new(),
            },
        );
    }
    let source = source(root, &environment)?;
    let identity = Identity {
        profile: PROFILE.into(),
        source: hash_bytes(&json(&file_map(&source)?)?),
        options: OPTIONS.iter().map(|value| (*value).into()).collect(),
        tools,
        inputs: original_inputs,
        selections,
    };
    let recipe = hash_bytes(&json(&identity)?);
    let generations = profile.join("generations").join(&recipe);
    private_directory(&generations)?;
    let locks = profile.join("locks");
    private_directory(&locks)?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .mode(0o600)
        .open(locks.join(&recipe))?;
    lock.lock()?;
    let mut candidates = fs::read_dir(&generations)?.collect::<Result<Vec<_>, _>>()?;
    candidates.sort_by_key(|entry| std::cmp::Reverse(entry.file_name()));
    for candidate in candidates {
        if !candidate
            .file_name()
            .to_str()
            .is_some_and(|name| component(name, 32))
        {
            continue;
        }
        if validate(&candidate.path(), Some(&identity)).is_ok()
            && immutable(&candidate.path()).is_ok()
        {
            let generation = candidate.file_name();
            let generation = generation
                .to_str()
                .ok_or_else(|| invalid("invalid generation"))?;
            publish_pointer(&profile, &recipe, generation)?;
            return Ok(candidate.path());
        }
    }
    let generation = uuid::Uuid::now_v7().simple().to_string();
    let destination = generations.join(&generation);
    let staging = profile.join("staging");
    private_directory(&staging)?;
    let scratch = tempfile::Builder::new()
        .prefix("prepare-")
        .tempdir_in(staging)?;
    let work = scratch.path().join("build");
    let destdir = scratch.path().join("dest");
    let install = destdir.join(
        destination
            .strip_prefix("/")
            .map_err(|_| invalid("native cache is not absolute"))?,
    );
    let cmake = &identity.tools["CMAKE"].path;
    let mut install_option = OsString::from("-DCMAKE_INSTALL_PREFIX=");
    install_option.push(&destination);
    let mut options: Vec<OsString> = vec![
        "-S".into(),
        source.into_os_string(),
        "-B".into(),
        work.clone().into_os_string(),
        install_option,
    ];
    options.extend(OPTIONS.iter().map(|option| format!("-D{option}").into()));
    run(cmake, &options, &environment)?;
    let jobs = text(env, "CARGO_BUILD_JOBS")?
        .ok_or_else(|| invalid("native preparation needs selected CARGO_BUILD_JOBS"))?;
    if jobs
        .parse::<usize>()
        .ok()
        .filter(|jobs| *jobs > 0)
        .is_none()
    {
        return Err(invalid("native compiler width must be a positive integer"));
    }
    run(
        cmake,
        &[
            "--build".into(),
            work.clone().into_os_string(),
            "--parallel".into(),
            jobs.into(),
        ],
        &environment,
    )?;
    environment.insert("DESTDIR".into(), destdir.into_os_string());
    run(
        cmake,
        &["--install".into(), work.into_os_string()],
        &environment,
    )?;
    let files = file_map(&install)?;
    if REQUIRED.iter().any(|name| !files.contains_key(*name)) {
        return Err(invalid("native installation is incomplete"));
    }
    fs::write(
        install.join(".complete.json"),
        json(&Receipt {
            version: 2,
            identity: identity.clone(),
            files,
        })?,
    )?;
    publish_generation(&install, &destination, &identity)?;
    publish_pointer(&profile, &recipe, &generation)?;
    Ok(destination)
}
fn publish_generation(
    install: &Path,
    destination: &Path,
    identity: &Identity,
) -> Result<(), HostError> {
    validate(install, Some(identity))?;
    seal(install)
        .map_err(|error| invalid(format!("sealing staged installation failed: {error}")))?;
    // Moving a directory between parents needs write access to update its '..' entry.
    // The generation stays unselected until its root is sealed again after the move.
    fs::set_permissions(install, fs::Permissions::from_mode(0o700))?;
    fs::rename(install, destination)
        .map_err(|error| invalid(format!("publishing generation directory failed: {error}")))?;
    fs::set_permissions(destination, fs::Permissions::from_mode(0o500))?;
    immutable(destination)?;
    Ok(())
}
/// Select only a completed immutable compiler/KLU generation. This never prepares sources.
pub fn select(root: &Path, environment: &Environment) -> Result<Environment, HostError> {
    let (prefix, receipt) = selected_prefix(&profile_root(environment)?)?;
    preflight(environment)?;
    immutable(&prefix)?;
    for (key, expected) in &receipt.identity.inputs {
        let actual = text(environment, key)?;
        let installed = receipt.identity.selections.get(key).map(String::as_str);
        let bridge = if key == "CMAKE_TOOLCHAIN_FILE" {
            receipt
                .identity
                .tools
                .get("effective-cmake-toolchain")
                .and_then(|tool| tool.path.to_str())
        } else {
            None
        };
        if actual != expected.as_deref()
            && !installed.is_some_and(|value| actual == Some(value))
            && !bridge.is_some_and(|value| actual == Some(value))
        {
            return Err(invalid(format!("{key} changed; run explicit preparation")));
        }
    }
    for tool in receipt.identity.tools.values() {
        if digest(&tool.path)? != tool.sha256 {
            return Err(invalid(
                "selected native tool changed; run explicit preparation",
            ));
        }
    }
    let mut selected = environment.clone();
    for (key, value) in &receipt.identity.selections {
        selected.insert(key.into(), value.into());
    }
    for (key, value) in [
        (
            "SUITESPARSE_INCLUDE_DIR",
            prefix.join("include/suitesparse"),
        ),
        ("SUITESPARSE_LIBRARY_DIR", prefix.join("lib")),
        ("PSE_KLU_PROVIDER_RECEIPT", prefix.join(".complete.json")),
    ] {
        if selected
            .get(OsStr::new(key))
            .is_some_and(|current| Path::new(current) != value)
        {
            return Err(invalid(format!(
                "{key} conflicts with selected native generation"
            )));
        }
        selected.insert(key.into(), value.into_os_string());
    }
    for key in ["SUNDIALS_INCLUDE_DIR", "SUNDIALS_LIBRARY_DIR"] {
        if selected.contains_key(OsStr::new(key)) {
            return Err(invalid(
                "external SUNDIALS selection is outside the bundled KINSOL profile",
            ));
        }
    }
    cache_bridge(root, &mut selected);
    if receipt
        .identity
        .tools
        .get("effective-cmake-toolchain")
        .map(|tool| tool.path.as_os_str())
        != selected
            .get(OsStr::new("CMAKE_TOOLCHAIN_FILE"))
            .map(OsString::as_os_str)
    {
        return Err(invalid(
            "effective CMake toolchain changed; run explicit preparation",
        ));
    }
    Ok(selected)
}
/// Execute the narrow explicit native preparation command within authenticated containment.
pub(crate) fn execute(
    root: &Path,
    capabilities: &str,
) -> Result<BTreeMap<String, PathBuf>, HostError> {
    providers::capabilities(capabilities)?;
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async {
            let manager = HostManager::connect_user(None).await?;
            let current = manager.current_unit().await?;
            if !manager.native_workload(&current).await? {
                return Err(invalid(
                    "explicit preparation requires scripts/pse-env containment",
                ));
            }
            Ok::<(), HostError>(())
        })?;
    let prepared = prepare_capabilities(&root, &std::env::vars_os().collect(), capabilities)?;
    Ok(prepared.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::ffi::OsStringExt;

    struct Fixture {
        directory: tempfile::TempDir,
        profile: PathBuf,
        prefix: PathBuf,
        identity: Identity,
        environment: Environment,
    }
    fn fixture() -> Fixture {
        let directory = tempfile::tempdir().unwrap();
        let mut environment = Environment::new();
        environment.insert("PSE_NATIVE_CACHE".into(), directory.path().into());
        let profile = profile_root(&environment).unwrap();
        private_directory(&profile).unwrap();
        let identity = Identity {
            profile: PROFILE.into(),
            source: "a".repeat(64),
            options: OPTIONS.iter().map(|value| (*value).into()).collect(),
            tools: BTreeMap::new(),
            inputs: inputs(&environment).unwrap(),
            selections: BTreeMap::new(),
        };
        let recipe = hash_bytes(&json(&identity).unwrap());
        let generation = "1".repeat(32);
        let prefix = profile.join("generations").join(&recipe).join(&generation);
        let staged = directory.path().join("stage");
        fs::create_dir_all(&staged).unwrap();
        for name in REQUIRED {
            let path = staged.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, b"native artifact fixture").unwrap();
        }
        let receipt = Receipt {
            version: 2,
            identity: identity.clone(),
            files: file_map(&staged).unwrap(),
        };
        fs::write(staged.join(".complete.json"), json(&receipt).unwrap()).unwrap();
        fs::create_dir_all(prefix.parent().unwrap()).unwrap();
        publish_generation(&staged, &prefix, &identity).unwrap();
        publish_pointer(&profile, &recipe, &generation).unwrap();
        Fixture {
            directory,
            profile,
            prefix,
            identity,
            environment,
        }
    }
    fn writable(path: &Path) {
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fn restore(path: &Path) {
                if path.is_dir() {
                    writable(path);
                    for entry in fs::read_dir(path).unwrap() {
                        restore(&entry.unwrap().path());
                    }
                }
            }
            restore(self.directory.path());
        }
    }

    #[test]
    fn native_selection_preserves_raw_values_and_reuses_immutable_generation() {
        let fixture = fixture();
        let mut environment = fixture.environment.clone();
        let raw = OsString::from_vec(vec![b'x', 0xff]);
        environment.insert("UNRELATED_NATIVE_FIXTURE".into(), raw.clone());
        let first = select(fixture.directory.path(), &environment).unwrap();
        let second = select(fixture.directory.path(), &environment).unwrap();
        assert_eq!(first, second);
        assert_eq!(first[OsStr::new("UNRELATED_NATIVE_FIXTURE")], raw);
        assert_eq!(
            first[OsStr::new("SUITESPARSE_LIBRARY_DIR")],
            fixture.prefix.join("lib")
        );
        assert!(validate(&fixture.prefix, Some(&fixture.identity)).is_ok());
    }
    #[test]
    fn native_selection_is_idempotent_for_verified_installed_environment() {
        let fixture = fixture();
        let mut receipt: Receipt = read_json(&fixture.prefix.join(".complete.json")).unwrap();
        receipt
            .identity
            .selections
            .insert("CC".into(), "/verified/compiler".into());
        receipt.identity.selections.insert(
            "BINDGEN_EXTRA_CLANG_ARGS".into(),
            "-resource-dir=/verified/resource".into(),
        );
        let recipe = hash_bytes(&json(&receipt.identity).unwrap());
        let destination = fixture
            .profile
            .join("generations")
            .join(&recipe)
            .join("2".repeat(32));
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        let stage = fixture.directory.path().join("idempotent-stage");
        fs::create_dir(&stage).unwrap();
        for name in REQUIRED {
            let path = stage.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, b"native artifact fixture").unwrap();
        }
        fs::write(stage.join(".complete.json"), json(&receipt).unwrap()).unwrap();
        publish_generation(&stage, &destination, &receipt.identity).unwrap();
        publish_pointer(&fixture.profile, &recipe, &"2".repeat(32)).unwrap();
        let first = select(fixture.directory.path(), &fixture.environment).unwrap();
        assert_eq!(select(fixture.directory.path(), &first).unwrap(), first);
    }
    #[test]
    fn native_identity_corruption_cannot_preserve_selected_recipe() {
        let fixture = fixture();
        let path = fixture.prefix.join(".complete.json");
        writable(&path);
        let mut receipt: Receipt = read_json(&path).unwrap();
        receipt
            .identity
            .selections
            .insert("CC".into(), "/wrong/compiler".into());
        fs::write(&path, json(&receipt).unwrap()).unwrap();
        seal(&fixture.prefix).unwrap();
        assert!(select(fixture.directory.path(), &fixture.environment).is_err());
    }
    #[test]
    fn native_file_corruption_and_writable_children_are_rejected() {
        let fixture = fixture();
        let path = fixture.prefix.join("lib/libklu.a");
        writable(&path);
        assert!(select(fixture.directory.path(), &fixture.environment).is_err());
        fs::write(&path, b"corrupt bytes").unwrap();
        seal(&fixture.prefix).unwrap();
        assert!(select(fixture.directory.path(), &fixture.environment).is_err());
    }
    #[test]
    fn native_failed_publication_keeps_previous_selection() {
        let fixture = fixture();
        let original = fs::read(fixture.profile.join("current.json")).unwrap();
        let stage = fixture.directory.path().join("incomplete");
        fs::create_dir(&stage).unwrap();
        fs::write(stage.join("partial"), b"incomplete").unwrap();
        assert!(
            publish_generation(
                &stage,
                &fixture.directory.path().join("unpublished"),
                &fixture.identity
            )
            .is_err()
        );
        assert_eq!(
            fs::read(fixture.profile.join("current.json")).unwrap(),
            original
        );
        assert!(select(fixture.directory.path(), &fixture.environment).is_ok());
    }
    #[test]
    fn native_external_provider_and_changed_flags_are_rejected() {
        let fixture = fixture();
        let mut environment = fixture.environment.clone();
        environment.insert("SUITESPARSE_LIBRARY_DIR".into(), "/external/lib".into());
        assert!(select(fixture.directory.path(), &environment).is_err());
        environment.remove(OsStr::new("SUITESPARSE_LIBRARY_DIR"));
        environment.insert("CFLAGS".into(), "-march=other".into());
        assert!(select(fixture.directory.path(), &environment).is_err());
    }
    #[test]
    fn native_cross_target_is_rejected_before_cache_effects() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("must-not-exist");
        let mut environment = Environment::new();
        environment.insert("PSE_NATIVE_CACHE".into(), root.clone().into_os_string());
        environment.insert(
            "CARGO_BUILD_TARGET".into(),
            "aarch64-unknown-linux-gnu".into(),
        );
        assert!(prepare(directory.path(), &environment).is_err());
        assert!(!root.exists());
    }
    #[test]
    fn native_current_pointer_publication_is_atomic_under_concurrency() {
        let fixture = fixture();
        let recipe = hash_bytes(&json(&fixture.identity).unwrap());
        std::thread::scope(|scope| {
            for _ in 0..8 {
                let profile = &fixture.profile;
                let recipe = &recipe;
                scope.spawn(move || {
                    for _ in 0..8 {
                        publish_pointer(profile, recipe, &"1".repeat(32)).unwrap();
                        selected_prefix(profile).unwrap();
                    }
                });
            }
        });
        assert!(select(fixture.directory.path(), &fixture.environment).is_ok());
    }
    #[test]
    fn native_generation_symlinks_are_rejected() {
        let fixture = fixture();
        let alias = fixture.directory.path().join("alias");
        std::os::unix::fs::symlink(&fixture.prefix, &alias).unwrap();
        assert!(validate(&alias, Some(&fixture.identity)).is_err());
    }
}
