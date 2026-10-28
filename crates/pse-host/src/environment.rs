// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pure child-environment selection. Personal shell evaluation belongs to the caller.
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::{OsStr, OsString},
    path::{Component, Path, PathBuf},
};

/// Environment values remain operating-system strings until an external text boundary.
pub type Environment = BTreeMap<OsString, OsString>;

/// Explicit compiler-cache intent belongs to the command owner, not another environment knob.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CacheMode {
    /// Use the native installed cache when supported, otherwise compile directly.
    #[default]
    Auto,
    /// Require the selected supported native cache.
    On,
    /// Disable compiler wrapping for this command.
    Off,
}

/// An invalid declaration or unsupported external environment selection.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("host environment: {0}")]
#[diagnostic(code(pse::host::environment))]
pub struct EnvironmentError(pub String);

/// Immutable filesystem selections established before pure composition.
#[derive(Debug, Clone)]
pub struct EnvironmentSettings {
    /// Checkout which owns paths and Cargo configuration.
    pub root: PathBuf,
    /// Declared interpreter selection for optional Python consumers.
    pub python: String,
    /// Declared ordinary Cargo width, not a host allocation ceiling.
    pub jobs: String,
    /// Native managed sccache executable when actually installed.
    pub cache_binary: Option<PathBuf>,
    /// Declared disk-cache size.
    pub cache_size: String,
    /// Declared native IPC frame bound.
    pub cache_frame: String,
    /// The selected cache mode can keep compiler execution with its caller.
    pub cache_compatible: bool,
    /// Existing optional solver toolchain prefix.
    pub solver_stack: Option<PathBuf>,
    /// Existing selected LLVM toolchain prefix.
    pub llvm_prefix: Option<PathBuf>,
    /// An unread personal file may change defaults.
    pub local_pending: bool,
}

fn failure(message: impl Into<String>) -> EnvironmentError {
    EnvironmentError(message.into())
}
fn get<'a>(env: &'a Environment, key: &str) -> Option<&'a OsStr> {
    env.get(OsStr::new(key)).map(OsString::as_os_str)
}
fn text<'a>(env: &'a Environment, key: &str) -> Result<Option<&'a str>, EnvironmentError> {
    get(env, key)
        .map(|value| {
            value
                .to_str()
                .ok_or_else(|| failure(format!("{key} must be UTF-8")))
        })
        .transpose()
}
fn insert(env: &mut Environment, key: &str, value: impl Into<OsString>) {
    env.insert(key.into(), value.into());
}
fn default(env: &mut Environment, key: &str, value: impl Into<OsString>) {
    env.entry(key.into()).or_insert_with(|| value.into());
}
fn read_toml(path: &Path) -> Result<toml::Value, EnvironmentError> {
    let value = std::fs::read_to_string(path)
        .map_err(|error| failure(format!("cannot read {}: {error}", path.display())))?;
    toml::from_str(&value).map_err(|error| failure(format!("invalid {}: {error}", path.display())))
}
fn field<'a>(value: &'a toml::Value, path: &[&str]) -> Result<&'a toml::Value, EnvironmentError> {
    path.iter().try_fold(value, |current, key| {
        current
            .get(*key)
            .ok_or_else(|| failure(format!("missing declaration {}", path.join("."))))
    })
}
fn string_field(value: &toml::Value, path: &[&str]) -> Result<String, EnvironmentError> {
    field(value, path)?
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| failure(format!("{} must be text", path.join("."))))
}
fn home(env: &Environment) -> Result<PathBuf, EnvironmentError> {
    get(env, "HOME")
        .map(PathBuf::from)
        .ok_or_else(|| failure("HOME is required for default user paths"))
}
fn base(env: &Environment, name: &str, suffix: &str) -> Result<PathBuf, EnvironmentError> {
    Ok(match get(env, name).filter(|value| !value.is_empty()) {
        Some(value) => PathBuf::from(value),
        None => home(env)?.join(suffix),
    })
}
fn prepend(env: &mut Environment, name: &str, path: &Path) -> Result<(), EnvironmentError> {
    let mut paths = vec![path.to_owned()];
    if let Some(old) = get(env, name) {
        paths.extend(
            std::env::split_paths(old)
                .filter(|entry| !entry.as_os_str().is_empty() && entry != path),
        );
    }
    let joined =
        std::env::join_paths(paths).map_err(|error| failure(format!("{name}: {error}")))?;
    insert(env, name, joined);
    Ok(())
}

fn lexical_path(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            other => result.push(other.as_os_str()),
        }
    }
    result
}

impl EnvironmentSettings {
    /// Read declared defaults/availability only; never source shell, start a service or install tools.
    pub fn load(root: &Path, inputs: &Environment) -> Result<Self, EnvironmentError> {
        let build = read_toml(&root.join(".config/build.toml"))?;
        let cargo = read_toml(&root.join(".cargo/config.toml"))?;
        let tools = read_toml(&root.join(".config/diagnostic-tools.toml"))?;
        let jobs = field(&cargo, &["build", "jobs"])?
            .as_integer()
            .filter(|jobs| *jobs > 0)
            .ok_or_else(|| failure("build.jobs must be positive"))?
            .to_string();
        let version = string_field(&tools, &["sccache", "version"])?;
        let member = string_field(&tools, &["sccache", "member"])?;
        let installed = base(inputs, "XDG_DATA_HOME", ".local/share")?
            .join("pse/tools/sccache")
            .join(version)
            .join(member);
        let explicit = get(inputs, "PSE_SCCACHE_BINARY");
        if explicit.is_some_and(OsStr::is_empty) {
            return Err(failure("PSE_SCCACHE_BINARY cannot be empty"));
        }
        let selected = explicit.map(PathBuf::from).unwrap_or(installed);
        if let Some(wrapper) = get(inputs, "RUSTC_WRAPPER") {
            if explicit.is_some()
                && !wrapper.is_empty()
                && Path::new(wrapper) != selected
                && Path::new(wrapper) != root.join("scripts/sccache")
            {
                return Err(failure(
                    "PSE_SCCACHE_BINARY conflicts with the explicit RUSTC_WRAPPER selection",
                ));
            }
        }
        let cache_wanted = explicit.is_some()
            || get(inputs, "RUSTC_WRAPPER").is_none_or(|wrapper| {
                Path::new(wrapper).file_name() == Some(OsStr::new("sccache"))
            });
        let cache_binary = if !cache_wanted {
            None
        } else if selected.exists() {
            use std::{io::Read as _, os::unix::fs::PermissionsExt as _};
            let metadata = selected
                .metadata()
                .map_err(|error| failure(error.to_string()))?;
            if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
                return Err(failure(
                    "selected compiler cache must be an executable regular file",
                ));
            }
            let mut magic = [0u8; 4];
            std::fs::File::open(&selected)
                .and_then(|mut file| file.read_exact(&mut magic))
                .map_err(|error| failure(error.to_string()))?;
            if magic != *b"\x7fELF" {
                return Err(failure(
                    "selected compiler cache must be a native ELF executable",
                ));
            }
            Some(selected)
        } else if explicit.is_some() {
            return Err(failure("selected compiler cache executable is missing"));
        } else {
            None
        };
        let config = get(inputs, "SCCACHE_CONF")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join(".config/sccache.toml"));
        let config = if cache_wanted {
            read_toml(&config)?
        } else {
            toml::Value::Table(Default::default())
        };
        let cache_compatible = get(inputs, "SCCACHE_LOG").is_none()
            && get(inputs, "SCCACHE_ERROR_LOG").is_none()
            && config
                .get("dist")
                .and_then(|value| value.get("scheduler_url"))
                .is_none();
        let llvm = get(inputs, "PSE_LLVM_PREFIX")
            .map(PathBuf::from)
            .or_else(|| {
                Path::new("/opt/llvm-current/bin/llvm-config")
                    .is_file()
                    .then(|| PathBuf::from("/opt/llvm-current"))
            });
        if let Some(prefix) = &llvm {
            if !prefix.join("bin/clang").is_file() || !prefix.join("lib/libclang.so").is_file() {
                return Err(failure("selected LLVM prefix lacks clang or libclang"));
            }
        }
        let frame = field(&build, &["cache_max_frame_length"])?
            .as_integer()
            .filter(|value| *value > 0 && *value <= i64::from(u32::MAX))
            .ok_or_else(|| failure("cache_max_frame_length must fit a positive u32"))?
            .to_string();
        Ok(Self {
            root: root.to_owned(),
            python: std::fs::read_to_string(root.join(".python-version"))
                .map_err(|error| failure(error.to_string()))?
                .trim()
                .to_owned(),
            jobs,
            cache_binary,
            cache_size: string_field(&build, &["cache_size"])?,
            cache_frame: frame,
            cache_compatible,
            solver_stack: Path::new("/opt/pse-solvers/bin")
                .is_dir()
                .then(|| PathBuf::from("/opt/pse-solvers")),
            llvm_prefix: llvm,
            local_pending: root.join(".envrc.local").is_file(),
        })
    }
}

/// A composed child mapping; no field mutates this process's global environment.
#[derive(Clone)]
pub struct EnvironmentPlan {
    /// Exact values for a future Command::envs call.
    pub values: Environment,
    /// Local shell selections were not supplied by the caller.
    pub local_unresolved: bool,
    root: PathBuf,
    local_keys: BTreeSet<OsString>,
    command_keys: BTreeSet<OsString>,
}

impl std::fmt::Debug for EnvironmentPlan {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EnvironmentPlan")
            .field("value_count", &self.values.len())
            .field("local_unresolved", &self.local_unresolved)
            .finish_non_exhaustive()
    }
}

/// Precedence is command options, caller, evaluated personal values, repository defaults.
pub fn compose(
    settings: &EnvironmentSettings,
    caller: &Environment,
    local: &Environment,
    command: &Environment,
) -> Result<EnvironmentPlan, EnvironmentError> {
    compose_with_cache(settings, caller, local, command, CacheMode::Auto)
}

/// Pure composition with the command owner's explicit cache selection.
pub fn compose_with_cache(
    settings: &EnvironmentSettings,
    caller: &Environment,
    local: &Environment,
    command: &Environment,
    cache: CacheMode,
) -> Result<EnvironmentPlan, EnvironmentError> {
    let mut env = local.clone();
    env.extend(caller.clone());
    env.extend(command.clone());
    default(&mut env, "PSE_PYTHON", &settings.python);
    default(&mut env, "UV_PROJECT_ENVIRONMENT", ".venv");
    let state =
        base(&env, "XDG_STATE_HOME", ".local/state")?.join("pse-arrow/surreal-functional-v2");
    default(&mut env, "PSE_SURREAL_STATE", state.into_os_string());
    prepend(&mut env, "PATH", &settings.root.join("scripts/cargo-bin"))?;
    let venv = PathBuf::from(
        get(&env, "UV_PROJECT_ENVIRONMENT")
            .ok_or_else(|| failure("missing environment selection"))?,
    );
    let venv = if venv.is_absolute() {
        venv
    } else {
        settings.root.join(venv)
    };
    prepend(&mut env, "PATH", &venv.join("bin"))?;
    if let Some(stack) = &settings.solver_stack {
        prepend(&mut env, "PATH", &stack.join("bin"))?;
        prepend(&mut env, "PKG_CONFIG_PATH", &stack.join("lib/pkgconfig"))?;
    }
    if let Some(prefix) = &settings.llvm_prefix {
        insert(&mut env, "PSE_LLVM_PREFIX", prefix.as_os_str());
        insert(
            &mut env,
            "CLANG_PATH",
            prefix.join("bin/clang").into_os_string(),
        );
        insert(
            &mut env,
            "LLVM_CONFIG_PATH",
            prefix.join("bin/llvm-config").into_os_string(),
        );
        default(
            &mut env,
            "LIBCLANG_PATH",
            prefix.join("lib").into_os_string(),
        );
        prepend(&mut env, "PATH", &prefix.join("bin"))?;
    }
    let inherited = get(&env, "PSE_NEXTEST_DEFAULT_THREADS").map(OsStr::to_owned);
    env.remove(OsStr::new("PSE_NEXTEST_DEFAULT_THREADS"));
    if inherited
        .as_deref()
        .is_some_and(|value| get(&env, "NEXTEST_TEST_THREADS") == Some(value))
    {
        env.remove(OsStr::new("NEXTEST_TEST_THREADS"));
    }
    let inherited = get(&env, "PSE_CARGO_DEFAULT_JOBS").map(OsStr::to_owned);
    if inherited
        .as_deref()
        .is_some_and(|value| get(&env, "CARGO_BUILD_JOBS") == Some(value))
        && !command.contains_key(OsStr::new("CARGO_BUILD_JOBS"))
    {
        env.remove(OsStr::new("CARGO_BUILD_JOBS"));
    }
    env.remove(OsStr::new("PSE_CARGO_DEFAULT_JOBS"));
    if get(&env, "CARGO_BUILD_JOBS").is_none() {
        insert(&mut env, "CARGO_BUILD_JOBS", &settings.jobs);
        insert(&mut env, "PSE_CARGO_DEFAULT_JOBS", &settings.jobs);
    }
    let mode = text(&env, "PSE_BUILD_MODE")?.unwrap_or("bounded");
    if !matches!(mode, "bounded" | "unbounded") {
        return Err(failure("PSE_BUILD_MODE must be bounded or unbounded"));
    }
    // An inherited checkout-generated Python wrapper is a retired default, not a custom tool.
    if get(&env, "RUSTC_WRAPPER")
        .is_some_and(|value| Path::new(value) == settings.root.join("scripts/sccache"))
    {
        env.remove(OsStr::new("RUSTC_WRAPPER"));
    }
    if cache == CacheMode::Off {
        insert(&mut env, "RUSTC_WRAPPER", "");
        insert(&mut env, "RUSTC_WORKSPACE_WRAPPER", "");
    } else if get(&env, "RUSTC_WRAPPER").is_none()
        && (cache == CacheMode::On || !get(&env, "CI").is_some_and(|v| !v.is_empty()))
    {
        if let Some(binary) = &settings.cache_binary {
            insert(&mut env, "RUSTC_WRAPPER", binary.as_os_str());
            insert(&mut env, "PSE_SCCACHE_BINARY", binary.as_os_str());
        } else if cache == CacheMode::On {
            return Err(failure(
                "cache requested but the native executable is unavailable",
            ));
        }
    }
    if cache == CacheMode::On && get(&env, "RUSTC_WRAPPER").is_none_or(OsStr::is_empty) {
        return Err(failure(
            "explicit wrapper selection disables requested cache",
        ));
    }
    let sccache = get(&env, "RUSTC_WRAPPER")
        .is_some_and(|value| Path::new(value).file_name() == Some(OsStr::new("sccache")));
    if sccache {
        if !settings.cache_compatible {
            if cache == CacheMode::On {
                return Err(failure(
                    "selected logging/distribution settings disable supported client-side compilation",
                ));
            }
            insert(&mut env, "RUSTC_WRAPPER", "");
            insert(&mut env, "RUSTC_WORKSPACE_WRAPPER", "");
        } else {
            insert(&mut env, "SCCACHE_CLIENT_SIDE", "1");
            insert(&mut env, "SCCACHE_DIRECT", "false");
            default(
                &mut env,
                "SCCACHE_CONF",
                settings.root.join(".config/sccache.toml").into_os_string(),
            );
            default(&mut env, "SCCACHE_MAX_FRAME_LENGTH", &settings.cache_frame);
            let directory = base(&env, "XDG_CACHE_HOME", ".cache")?.join("pse-arrow/sccache");
            default(&mut env, "SCCACHE_DIR", directory.into_os_string());
            default(&mut env, "SCCACHE_CACHE_SIZE", &settings.cache_size);
            let endpoint = match get(&env, "XDG_RUNTIME_DIR").filter(|value| !value.is_empty()) {
                Some(path) => PathBuf::from(path).join("pse-sccache.sock"),
                None => base(&env, "XDG_CACHE_HOME", ".cache")?.join("pse-arrow/sccache.sock"),
            };
            default(&mut env, "SCCACHE_SERVER_UDS", endpoint.into_os_string());
        }
    }
    // A foreign inherited target must not join two checkouts' Cargo state.
    if let Some(selected) = get(&env, "PSE_CARGO_TARGET_DIR")
        .filter(|value| !value.is_empty())
        .map(OsStr::to_owned)
    {
        insert(&mut env, "CARGO_TARGET_DIR", selected);
    } else if let Some(target) = get(&env, "CARGO_TARGET_DIR").map(PathBuf::from) {
        let target = if target.is_absolute() {
            target
        } else {
            settings.root.join(target)
        };
        let target = lexical_path(&target);
        let root = lexical_path(&settings.root);
        if !target.starts_with(&root) || target == root.join("target") {
            env.remove(OsStr::new("CARGO_TARGET_DIR"));
        }
    }
    Ok(EnvironmentPlan {
        values: env,
        local_unresolved: settings.local_pending && local.is_empty(),
        root: settings.root.clone(),
        local_keys: local.keys().cloned().collect(),
        command_keys: command.keys().cloned().collect(),
    })
}

/// Load filesystem facts after merging selection inputs, then perform pure composition.
pub fn resolve(
    root: &Path,
    caller: &Environment,
    local: &Environment,
    command: &Environment,
) -> Result<EnvironmentPlan, EnvironmentError> {
    resolve_with_local(root, caller, Some(local), command)
}

/// A missing local mapping is unresolved; an explicitly evaluated empty map is complete.
pub fn resolve_with_local(
    root: &Path,
    caller: &Environment,
    local: Option<&Environment>,
    command: &Environment,
) -> Result<EnvironmentPlan, EnvironmentError> {
    let empty = Environment::new();
    let selected_local = local.unwrap_or(&empty);
    let mut selected = selected_local.clone();
    selected.extend(caller.clone());
    selected.extend(command.clone());
    let mut settings = EnvironmentSettings::load(root, &selected)?;
    settings.local_pending &= local.is_none();
    compose(&settings, caller, selected_local, command)
}

/// Personal shell exports obtained only through an explicitly requested activation.
#[derive(Clone)]
pub struct EvaluatedLocal {
    values: Environment,
}
impl std::fmt::Debug for EvaluatedLocal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EvaluatedLocal")
            .field("value_count", &self.values.len())
            .finish_non_exhaustive()
    }
}
impl EvaluatedLocal {
    /// Exact exported selections, including operating-system byte values.
    pub fn values(&self) -> &Environment {
        &self.values
    }
}

/// Evaluate an explicitly selected checkout's personal file in an isolated native shell.
/// Reading/explaining an environment never calls this operation.
pub fn evaluate_local(
    root: &Path,
    caller: &Environment,
) -> Result<EvaluatedLocal, EnvironmentError> {
    use pse_diagnostics::{DrainFailurePolicy, SupervisedChild};
    use std::{
        io::Read as _,
        os::unix::ffi::OsStringExt as _,
        process::{Command, Stdio},
        time::{Duration, Instant},
    };
    let file = root.join(".envrc.local");
    if !file.is_file() {
        return Ok(EvaluatedLocal {
            values: Environment::new(),
        });
    }
    let mut clean = Environment::new();
    insert(
        &mut clean,
        "HOME",
        get(caller, "HOME").ok_or_else(|| failure("HOME is required for personal activation"))?,
    );
    insert(&mut clean, "PATH", "/usr/bin:/bin");
    let mut command = Command::new("/bin/bash");
    command
        .args([
            "--noprofile",
            "--norc",
            "-c",
            ". \"$1\" >/dev/null 2>&1 || exit $?; /usr/bin/env -0",
            "pse-host-personal",
        ])
        .arg(&file)
        .env_clear()
        .envs(&clean)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = SupervisedChild::spawn(
        &mut command,
        DrainFailurePolicy::Refuse(Duration::from_secs(2)),
        None,
    )
    .map_err(|_| failure("personal activation could not start its native shell"))?;
    let output = child
        .take_stdout()
        .ok_or_else(|| failure("personal activation output unavailable"))?;
    let reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        output
            .take(4 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map(|_| bytes)
    });
    let completed = child.wait_ready(None, Some(Instant::now() + Duration::from_secs(10)));
    let status = if completed.is_ok() {
        child.wait()
    } else {
        child.cancel()
    };
    let bytes = reader
        .join()
        .map_err(|_| failure("personal activation reader failed"))?
        .map_err(|_| failure("personal activation output could not be read"))?;
    if completed.is_err() {
        return Err(failure(
            "personal activation did not complete within its deadline",
        ));
    }
    if !status
        .map_err(|_| failure("personal activation shell could not settle"))?
        .success()
    {
        return Err(failure("personal activation shell refused configuration"));
    }
    if bytes.len() > 4 * 1024 * 1024 || bytes.last() != Some(&0) {
        return Err(failure(
            "personal activation did not return a complete bounded environment",
        ));
    }
    let mut values = Environment::new();
    for item in bytes[..bytes.len() - 1].split(|byte| *byte == 0) {
        let equals = item
            .iter()
            .position(|byte| *byte == b'=')
            .ok_or_else(|| failure("personal activation returned an invalid environment entry"))?;
        if equals == 0 {
            return Err(failure(
                "personal activation returned an empty environment key",
            ));
        }
        let key = OsString::from_vec(item[..equals].to_vec());
        let value = OsString::from_vec(item[equals + 1..].to_vec());
        if ["_", "SHLVL", "PWD", "OLDPWD"]
            .into_iter()
            .any(|name| key == OsStr::new(name))
            || clean.get(&key) == Some(&value)
        {
            continue;
        }
        if values.insert(key, value).is_some() {
            return Err(failure(
                "personal activation returned a duplicate environment key",
            ));
        }
    }
    Ok(EvaluatedLocal { values })
}

/// Explicit activation resolves exported personal selections before applying caller precedence.
pub fn activate(
    root: &Path,
    caller: &Environment,
    command: &Environment,
) -> Result<EnvironmentPlan, EnvironmentError> {
    let local = evaluate_local(root, caller)?;
    resolve(root, caller, local.values(), command)
}

fn quote(value: &OsStr) -> Result<String, EnvironmentError> {
    let value = value.to_str().ok_or_else(|| {
        failure(
            "shell rendering requires UTF-8; use the native environment mapping for opaque values",
        )
    })?;
    Ok(format!("'{}'", value.replace('\'', "'\\''")))
}
fn identifier(value: &OsStr) -> Result<&str, EnvironmentError> {
    let value = value
        .to_str()
        .ok_or_else(|| failure("environment key must be UTF-8"))?;
    if value.is_empty()
        || !value.bytes().enumerate().all(|(index, byte)| {
            byte == b'_' || byte.is_ascii_alphabetic() || (index > 0 && byte.is_ascii_digit())
        })
    {
        return Err(failure("invalid shell environment identifier"));
    }
    Ok(value)
}
impl EnvironmentPlan {
    /// Render personal values as deferred reads, never as literal secrets.
    pub fn render_exports(&self, caller: &Environment) -> Result<String, EnvironmentError> {
        if self.local_unresolved {
            return Err(failure(
                "personal selections are unresolved; an explicit activation owner must supply the evaluated local map before rendering exports",
            ));
        }
        for name in [
            "HOME",
            "PATH",
            "UV_PROJECT_ENVIRONMENT",
            "XDG_STATE_HOME",
            "XDG_DATA_HOME",
            "XDG_CACHE_HOME",
            "PSE_LLVM_PREFIX",
        ] {
            if self.local_keys.contains(OsStr::new(name))
                && !caller.contains_key(OsStr::new(name))
                && !self.command_keys.contains(OsStr::new(name))
            {
                return Err(failure(
                    "personal path selections require the native child mapping; shell rendering cannot disclose derived private values",
                ));
            }
        }
        let mut lines = Vec::new();
        for (key, value) in &self.values {
            if caller.get(key) == Some(value) && !self.command_keys.contains(key) {
                continue;
            }
            let name = identifier(key)?;
            if name.starts_with("PSE_NATIVE_") {
                continue;
            }
            if self.local_keys.contains(key)
                && !caller.contains_key(key)
                && !self.command_keys.contains(key)
            {
                lines.push(format!("[ -n \"${{{name}+x}}\" ] || {{ {name}=\"$(. {} >/dev/null 2>&1; printf %s \"${{{name}-}}\")\" && export {name}; }}",quote(self.root.join(".envrc.local").as_os_str())?));
            } else {
                lines.push(format!("export {name}={}", quote(value)?));
            }
        }
        for key in caller.keys().filter(|key| !self.values.contains_key(*key)) {
            lines.push(format!("unset {}", identifier(key)?));
        }
        Ok(lines.join("\n") + "\n")
    }
}

/// Fixed environment command: explanation is read-only; export is explicit personal activation.
pub fn run_cli(arguments: &[OsString]) -> Result<i32, EnvironmentError> {
    let print = arguments.iter().any(|argument| argument == "--print");
    let explain = arguments.iter().any(|argument| argument == "--explain");
    let json = arguments.iter().any(|argument| argument == "--json");
    if arguments
        .iter()
        .any(|argument| argument != "--print" && argument != "--explain" && argument != "--json")
        || (print && explain)
        || (json && !explain)
    {
        return Err(failure("expected --print or --explain [--json]"));
    }
    let mut root = std::env::current_dir().map_err(|error| failure(error.to_string()))?;
    while !root.join(".config/diagnostic-tools.toml").is_file() {
        if !root.pop() {
            return Err(failure(
                "run environment selection inside a pse-arrow checkout",
            ));
        }
    }
    let caller = std::env::vars_os().collect();
    let plan = if print {
        activate(&root, &caller, &Environment::new())?
    } else {
        resolve_with_local(&root, &caller, None, &Environment::new())?
    };
    let output = if print {
        plan.render_exports(&caller)?
    } else if arguments
        .iter()
        .all(|argument| argument == "--explain" || argument == "--json")
    {
        let selected: [&str; 8] = [
            "PSE_BUILD_MODE",
            "CARGO_BUILD_JOBS",
            "RUSTC_WRAPPER",
            "SCCACHE_CLIENT_SIDE",
            "SCCACHE_DIR",
            "PSE_SURREAL_STATE",
            "UV_PROJECT_ENVIRONMENT",
            "PSE_PYTHON",
        ];
        let values: BTreeMap<_, _> = selected
            .into_iter()
            .map(|name| (name, get(&plan.values, name).and_then(OsStr::to_str)))
            .collect();
        serde_json::to_string_pretty(
            &serde_json::json!({"local_unresolved":plan.local_unresolved,"selections":values}),
        )
        .map_err(|error| failure(error.to_string()))?
            + "\n"
    } else {
        return Err(failure("expected --print or --explain [--json]"));
    };
    use std::io::Write as _;
    std::io::stdout()
        .lock()
        .write_all(output.as_bytes())
        .map_err(|error| failure(error.to_string()))?;
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn map(values: &[(&str, &str)]) -> Environment {
        values
            .iter()
            .map(|(key, value)| ((*key).into(), (*value).into()))
            .collect()
    }
    fn settings() -> EnvironmentSettings {
        EnvironmentSettings {
            root: "/checkout".into(),
            python: "3.14.8".into(),
            jobs: "32".into(),
            cache_binary: Some("/tools/sccache".into()),
            cache_size: "100G".into(),
            cache_frame: "268435456".into(),
            cache_compatible: true,
            solver_stack: None,
            llvm_prefix: None,
            local_pending: false,
        }
    }
    #[test]
    fn precedence_and_explicit_empty_are_preserved() {
        let plan = compose(
            &settings(),
            &map(&[
                ("HOME", "/home/test"),
                ("X", "caller"),
                ("RUSTC_WRAPPER", ""),
            ]),
            &map(&[("X", "local"), ("Y", "local")]),
            &map(&[("X", "command")]),
        )
        .unwrap();
        assert_eq!(get(&plan.values, "X"), Some(OsStr::new("command")));
        assert_eq!(get(&plan.values, "Y"), Some(OsStr::new("local")));
        assert_eq!(get(&plan.values, "RUSTC_WRAPPER"), Some(OsStr::new("")));
    }
    #[test]
    fn native_cache_has_no_python_wrapper() {
        let plan = compose(
            &settings(),
            &map(&[("HOME", "/home/test")]),
            &Environment::new(),
            &Environment::new(),
        )
        .unwrap();
        assert_eq!(
            get(&plan.values, "RUSTC_WRAPPER"),
            Some(OsStr::new("/tools/sccache"))
        );
        assert_eq!(
            get(&plan.values, "CARGO_BUILD_JOBS"),
            Some(OsStr::new("32"))
        );
    }
    #[test]
    fn custom_wrapper_and_unbounded_finite_selection_survive() {
        let caller = map(&[
            ("HOME", "/home/test"),
            ("RUSTC_WRAPPER", "/custom/wrapper"),
            ("PSE_BUILD_MODE", "unbounded"),
            ("PSE_MEMORY_MAX", "8G"),
            ("CARGO_BUILD_JOBS", "17"),
        ]);
        let plan = compose(
            &settings(),
            &caller,
            &Environment::new(),
            &Environment::new(),
        )
        .unwrap();
        for key in [
            "RUSTC_WRAPPER",
            "PSE_BUILD_MODE",
            "PSE_MEMORY_MAX",
            "CARGO_BUILD_JOBS",
        ] {
            assert_eq!(get(&plan.values, key), get(&caller, key));
        }
    }
    #[test]
    fn incompatible_cache_auto_falls_back_but_on_refuses() {
        let mut settings = settings();
        settings.cache_compatible = false;
        let caller = map(&[("HOME", "/home/test")]);
        assert_eq!(
            get(
                &compose(&settings, &caller, &Environment::new(), &Environment::new())
                    .unwrap()
                    .values,
                "RUSTC_WRAPPER"
            ),
            Some(OsStr::new(""))
        );
        assert!(
            compose_with_cache(
                &settings,
                &caller,
                &Environment::new(),
                &Environment::new(),
                CacheMode::On
            )
            .is_err()
        );
    }
    #[test]
    fn inherited_default_width_refreshes_but_explicit_width_survives() {
        let caller = map(&[
            ("HOME", "/home/test"),
            ("CARGO_BUILD_JOBS", "4"),
            ("PSE_CARGO_DEFAULT_JOBS", "4"),
        ]);
        assert_eq!(
            get(
                &compose(
                    &settings(),
                    &caller,
                    &Environment::new(),
                    &Environment::new()
                )
                .unwrap()
                .values,
                "CARGO_BUILD_JOBS"
            ),
            Some(OsStr::new("32"))
        );
        assert_eq!(
            get(
                &compose(
                    &settings(),
                    &caller,
                    &Environment::new(),
                    &map(&[("CARGO_BUILD_JOBS", "4")])
                )
                .unwrap()
                .values,
                "CARGO_BUILD_JOBS"
            ),
            Some(OsStr::new("4"))
        );
    }
    #[test]
    fn local_secret_is_deferred_and_unresolved_render_refuses() {
        let caller = map(&[("HOME", "/home/test")]);
        let mut settings = settings();
        settings.local_pending = true;
        let local = map(&[("API_TOKEN", "secret-do-not-render")]);
        let plan = compose(&settings, &caller, &local, &Environment::new()).unwrap();
        let rendered = plan.render_exports(&caller).unwrap();
        assert!(!rendered.contains("secret-do-not-render"));
        assert!(rendered.contains("API_TOKEN="));
        assert!(
            compose(&settings, &caller, &Environment::new(), &Environment::new())
                .unwrap()
                .render_exports(&caller)
                .is_err()
        );
    }
    #[test]
    fn composition_is_idempotent_and_does_not_touch_global_environment() {
        let settings = settings();
        let caller = map(&[("HOME", "/home/test"), ("PATH", "/usr/bin")]);
        let first = compose(&settings, &caller, &Environment::new(), &Environment::new()).unwrap();
        let second = compose(
            &settings,
            &first.values,
            &Environment::new(),
            &Environment::new(),
        )
        .unwrap();
        assert_eq!(first.values, second.values);
        assert_eq!(get(&caller, "PATH"), Some(OsStr::new("/usr/bin")));
    }
    #[test]
    fn cache_off_and_foreign_target_removal_are_explicit() {
        let plan = compose_with_cache(
            &settings(),
            &map(&[
                ("HOME", "/home/test"),
                ("CARGO_TARGET_DIR", "/checkout/../other/target"),
            ]),
            &Environment::new(),
            &Environment::new(),
            CacheMode::Off,
        )
        .unwrap();
        assert!(get(&plan.values, "CARGO_TARGET_DIR").is_none());
        assert_eq!(get(&plan.values, "RUSTC_WRAPPER"), Some(OsStr::new("")));
    }
    fn declaration_fixture() -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join(".config")).unwrap();
        std::fs::create_dir(root.path().join(".cargo")).unwrap();
        std::fs::write(root.path().join(".python-version"), "3.14.8").unwrap();
        std::fs::write(root.path().join(".cargo/config.toml"), "[build]\njobs=32\n").unwrap();
        std::fs::write(
            root.path().join(".config/build.toml"),
            "cache_size='100G'\ncache_max_frame_length=268435456\n",
        )
        .unwrap();
        std::fs::write(
            root.path().join(".config/diagnostic-tools.toml"),
            "[sccache]\nversion='0.19.0'\nmember='sccache'\n",
        )
        .unwrap();
        std::fs::write(root.path().join(".config/sccache.toml"), "").unwrap();
        root
    }
    #[test]
    fn explicit_cache_selection_rejects_empty_conflicting_missing_and_non_native_files() {
        use std::os::unix::fs::PermissionsExt as _;
        let root = declaration_fixture();
        let mut caller = map(&[
            ("HOME", root.path().to_str().unwrap()),
            ("PSE_SCCACHE_BINARY", ""),
        ]);
        assert!(EnvironmentSettings::load(root.path(), &caller).is_err());
        let binary = std::env::current_exe().unwrap();
        insert(&mut caller, "PSE_SCCACHE_BINARY", binary.as_os_str());
        insert(&mut caller, "RUSTC_WRAPPER", "/other/wrapper");
        assert!(EnvironmentSettings::load(root.path(), &caller).is_err());
        caller.remove(OsStr::new("RUSTC_WRAPPER"));
        assert_eq!(
            EnvironmentSettings::load(root.path(), &caller)
                .unwrap()
                .cache_binary,
            Some(binary)
        );
        insert(
            &mut caller,
            "PSE_SCCACHE_BINARY",
            root.path().join("absent").into_os_string(),
        );
        assert!(EnvironmentSettings::load(root.path(), &caller).is_err());
        let script = root.path().join("script");
        std::fs::write(&script, "#!/bin/sh\nexit 0\n").unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o700)).unwrap();
        insert(&mut caller, "PSE_SCCACHE_BINARY", script.into_os_string());
        assert!(EnvironmentSettings::load(root.path(), &caller).is_err());
    }
    #[test]
    fn empty_evaluated_local_map_is_not_an_unread_file() {
        let root = declaration_fixture();
        std::fs::write(root.path().join(".envrc.local"), "# no exported values\n").unwrap();
        let caller = map(&[("HOME", root.path().to_str().unwrap())]);
        assert!(
            resolve_with_local(root.path(), &caller, None, &Environment::new())
                .unwrap()
                .local_unresolved
        );
        assert!(
            !resolve(
                root.path(),
                &caller,
                &Environment::new(),
                &Environment::new()
            )
            .unwrap()
            .local_unresolved
        );
    }

    #[test]
    fn explanation_does_not_evaluate_personal_code_but_explicit_activation_does() {
        let root = declaration_fixture();
        std::fs::write(root.path().join(".envrc.local"),"printf hit >> \"$HOME/activation-counter\"\nexport SHARED=local\nexport API_TOKEN='private-local-secret'\n").unwrap();
        let caller = map(&[
            ("HOME", root.path().to_str().unwrap()),
            ("SHARED", "caller"),
        ]);
        let read = resolve_with_local(root.path(), &caller, None, &Environment::new()).unwrap();
        assert!(read.local_unresolved);
        assert!(!root.path().join("activation-counter").exists());
        let plan = activate(root.path(), &caller, &map(&[("SHARED", "command")])).unwrap();
        assert!(!plan.local_unresolved);
        assert_eq!(get(&plan.values, "SHARED"), Some(OsStr::new("command")));
        assert_eq!(
            get(&plan.values, "API_TOKEN"),
            Some(OsStr::new("private-local-secret"))
        );
        assert_eq!(
            std::fs::read(root.path().join("activation-counter")).unwrap(),
            b"hit"
        );
        let shell = plan.render_exports(&caller).unwrap();
        assert!(!shell.contains("private-local-secret"));
        assert!(shell.contains("API_TOKEN="));
    }
    #[test]
    fn activation_keeps_private_derived_paths_in_the_typed_mapping() {
        let root = declaration_fixture();
        std::fs::write(
            root.path().join(".envrc.local"),
            "export XDG_STATE_HOME=\"$HOME/private-state\"\n",
        )
        .unwrap();
        let caller = map(&[("HOME", root.path().to_str().unwrap())]);
        let plan = activate(root.path(), &caller, &Environment::new()).unwrap();
        assert_eq!(
            Path::new(get(&plan.values, "PSE_SURREAL_STATE").unwrap()),
            root.path()
                .join("private-state/pse-arrow/surreal-functional-v2")
        );
        assert!(plan.render_exports(&caller).is_err());
    }
    #[test]
    fn activation_does_not_expose_shell_error_output_or_accept_early_success_exit() {
        let root = declaration_fixture();
        let caller = map(&[("HOME", root.path().to_str().unwrap())]);
        std::fs::write(
            root.path().join(".envrc.local"),
            "echo private-error >&2\nreturn 12\n",
        )
        .unwrap();
        let error = match activate(root.path(), &caller, &Environment::new()) {
            Ok(_) => panic!("failed personal configuration was accepted"),
            Err(error) => error,
        };
        assert!(!error.to_string().contains("private-error"));
        std::fs::write(root.path().join(".envrc.local"), "exit 0\n").unwrap();
        assert!(activate(root.path(), &caller, &Environment::new()).is_err());
    }

    #[test]
    fn debug_output_does_not_disclose_personal_credentials() {
        let local = EvaluatedLocal {
            values: map(&[("API_TOKEN", "credential-marker-never-log")]),
        };
        let plan = compose(
            &settings(),
            &map(&[("HOME", "/home/test")]),
            local.values(),
            &Environment::new(),
        )
        .unwrap();
        assert!(!format!("{local:?} {plan:?}").contains("credential-marker-never-log"));
    }
}

#[cfg(test)]
mod native_wrapper_reentry {
    use super::*;
    #[test]
    fn retired_checkout_default_cannot_reenter_python() {
        let caller: Environment = [
            ("HOME", "/home/test"),
            ("RUSTC_WRAPPER", "/checkout/scripts/sccache"),
        ]
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .collect();
        let settings = EnvironmentSettings {
            root: "/checkout".into(),
            python: "3.14.8".into(),
            jobs: "32".into(),
            cache_binary: Some("/tools/sccache".into()),
            cache_size: "100G".into(),
            cache_frame: "268435456".into(),
            cache_compatible: true,
            solver_stack: None,
            llvm_prefix: None,
            local_pending: false,
        };
        let plan = compose(&settings, &caller, &Environment::new(), &Environment::new()).unwrap();
        assert_eq!(
            get(&plan.values, "RUSTC_WRAPPER"),
            Some(OsStr::new("/tools/sccache"))
        );
    }
}
