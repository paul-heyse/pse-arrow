// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pure child-environment selection. Personal shell evaluation belongs to the caller.
use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    path::{Component, Path, PathBuf},
};

/// Environment values remain operating-system strings until an external text boundary.
pub type Environment = BTreeMap<OsString, OsString>;

/// Explicit compiler-cache intent belongs to the command owner, not another environment knob.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
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
    /// Finite libtest worker stack, independent of host admission or CPU width.
    pub rust_stack_bytes: usize,
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
    // TOML errors can quote private configuration source, including credentials.
    toml::from_str(&value).map_err(|_| failure(format!("invalid configuration {}", path.display())))
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

fn rust_stack_bytes(value: &OsStr) -> Result<usize, EnvironmentError> {
    let value = value
        .to_str()
        .ok_or_else(|| failure("RUST_MIN_STACK must be a positive finite byte count"))?;
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(failure(
            "RUST_MIN_STACK must be a positive finite byte count",
        ));
    }
    value
        .parse::<usize>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| failure("RUST_MIN_STACK must be a positive finite byte count"))
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
        let rust_stack_bytes = match get(inputs, "RUST_MIN_STACK") {
            Some(value) => rust_stack_bytes(value)?,
            None => {
                let capacity = read_toml(&root.join(".config/agent-capacity.toml"))?;
                field(&capacity, &["test-runner", "rust_thread_stack_mib"])?.as_integer()
                    .and_then(|mib| usize::try_from(mib).ok())
                    .and_then(|mib| mib.checked_mul(1 << 20))
                    .filter(|bytes| *bytes > 0)
                    .ok_or_else(|| failure("test-runner.rust_thread_stack_mib must be a positive finite integer MiB"))?
            }
        };
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
        if let Some(wrapper) = get(inputs, "RUSTC_WRAPPER")
            && explicit.is_some()
            && !wrapper.is_empty()
            && Path::new(wrapper) != selected
            && Path::new(wrapper) != root.join("scripts/sccache")
        {
            return Err(failure(
                "PSE_SCCACHE_BINARY conflicts with the explicit RUSTC_WRAPPER selection",
            ));
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
        if let Some(prefix) = &llvm
            && (!prefix.join("bin/clang").is_file() || !prefix.join("lib/libclang.so").is_file())
        {
            return Err(failure("selected LLVM prefix lacks clang or libclang"));
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
            rust_stack_bytes,
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
    default(
        &mut env,
        "RUST_MIN_STACK",
        settings.rust_stack_bytes.to_string(),
    );
    rust_stack_bytes(
        get(&env, "RUST_MIN_STACK").ok_or_else(|| failure("missing Rust stack selection"))?,
    )?;
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
    if cache == CacheMode::On && !sccache {
        return Err(failure(
            "requested cache conflicts with the selected compiler wrapper",
        ));
    }
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
    resolve_with_options(root, caller, local, command, CacheMode::Auto, false)
}

/// Resolve a native child mapping with explicit cache and documentation selections.
pub fn resolve_with_options(
    root: &Path,
    caller: &Environment,
    local: Option<&Environment>,
    command: &Environment,
    cache: CacheMode,
    docs: bool,
) -> Result<EnvironmentPlan, EnvironmentError> {
    let mut command = command.clone();
    if docs {
        insert(
            &mut command,
            "UV_PROJECT_ENVIRONMENT",
            root.join(".venv-docs").into_os_string(),
        );
    }
    let command = &command;
    let empty = Environment::new();
    let selected_local = local.unwrap_or(&empty);
    let mut selected = selected_local.clone();
    selected.extend(caller.clone());
    selected.extend(command.clone());
    if cache == CacheMode::Off {
        selected.remove(OsStr::new("PSE_SCCACHE_BINARY"));
        insert(&mut selected, "RUSTC_WRAPPER", "");
    }
    let mut settings = EnvironmentSettings::load(root, &selected)?;
    settings.local_pending &= local.is_none();
    let mut plan = compose_with_cache(&settings, caller, selected_local, command, cache)?;
    crate::telemetry::configure(&mut plan.values);
    Ok(plan)
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
    activate_with_options(root, caller, command, CacheMode::Auto, false)
}

/// Explicit activation; documentation selection does not create or synchronize an environment.
pub fn activate_with_options(
    root: &Path,
    caller: &Environment,
    command: &Environment,
    cache: CacheMode,
    docs: bool,
) -> Result<EnvironmentPlan, EnvironmentError> {
    let local = evaluate_local(root, caller)?;
    resolve_with_options(root, caller, Some(local.values()), command, cache, docs)
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
    /// Serialize changes only into a private NUL transfer, never shell program text.
    pub fn activation_transfer(&self, caller: &Environment) -> Result<Vec<u8>, EnvironmentError> {
        use std::os::unix::ffi::OsStrExt as _;
        if self.local_unresolved {
            return Err(failure("personal selections have not been evaluated"));
        }
        let mut bytes = b"pse-activation-1\0".to_vec();
        for (key, value) in &self.values {
            if caller.get(key) == Some(value) || key.as_bytes().starts_with(b"PSE_NATIVE_") {
                continue;
            }
            identifier(key)?;
            if value.as_bytes().contains(&0) {
                return Err(failure("environment values cannot contain NUL bytes"));
            }
            bytes.push(b'+');
            bytes.extend_from_slice(key.as_bytes());
            bytes.push(b'=');
            bytes.extend_from_slice(value.as_bytes());
            bytes.push(0);
        }
        for key in caller.keys().filter(|key| !self.values.contains_key(*key)) {
            identifier(key)?;
            bytes.push(b'-');
            bytes.extend_from_slice(key.as_bytes());
            bytes.push(0);
        }
        bytes.extend_from_slice(b"pse-activation-end\0\0");
        Ok(bytes)
    }
}

/// Fixed Bash program. Private values are read once from a pipe after the caller evaluates it.
pub fn activation_script(
    executable: &Path,
    root: &Path,
    docs: bool,
    cache: CacheMode,
) -> Result<String, EnvironmentError> {
    let command = format!(
        "{} -C {} env --activation-data --cache {}{}",
        quote(executable.as_os_str())?,
        quote(root.as_os_str())?,
        match cache {
            CacheMode::Auto => "auto",
            CacheMode::On => "on",
            CacheMode::Off => "off",
        },
        if docs { " --docs" } else { "" }
    );
    Ok(ACTIVATION_SCRIPT.replace("@PRODUCER@", &command))
}

// The payload is data, never eval input. Read and wait before touching the caller mapping.
const ACTIVATION_SCRIPT: &str = r#"__pse_activate() {
    local __pse_trace=0
    case $- in *x*) __pse_trace=1; set +x;; esac
    local __pse_fd __pse_pid __pse_status=0 __pse_record __pse_key __pse_decl __pse_flags
    local -a __pse_records=()
    local -A __pse_seen=()
    if exec {__pse_fd}< <(@PRODUCER@); then
        __pse_pid=$!
        if ! mapfile -d '' -t __pse_records <&"$__pse_fd"; then __pse_status=125; fi
        exec {__pse_fd}<&-
        if ! wait "$__pse_pid"; then __pse_status=125; fi
    else
        __pse_status=125
    fi
    if (( __pse_status == 0 )); then
        if (( ${#__pse_records[@]} < 3 )) || [[ ${__pse_records[0]} != pse-activation-1 || ${__pse_records[-2]} != pse-activation-end || -n ${__pse_records[-1]} ]]; then
            __pse_status=125
        else
            unset '__pse_records[-1]' '__pse_records[-1]' '__pse_records[0]'
            for __pse_record in "${__pse_records[@]}"; do
                case $__pse_record in
                    +*=*) __pse_key=${__pse_record%%=*}; __pse_key=${__pse_key:1};;
                    -*) __pse_key=${__pse_record:1};;
                    *) __pse_status=125; break;;
                esac
                if [[ ! $__pse_key =~ ^[a-zA-Z_][a-zA-Z_0-9]*$ || $__pse_key == __pse_* || ${__pse_seen[$__pse_key]+present} ]]; then
                    __pse_status=125; break
                fi
                __pse_seen[$__pse_key]=1
                if __pse_decl=$(declare -p "$__pse_key" 2>/dev/null); then
                    __pse_flags=${__pse_decl#declare -}; __pse_flags=${__pse_flags%% *}
                    case $__pse_flags in *r*|*a*|*A*|*n*) __pse_status=125; break;; esac
                fi
            done
        fi
    fi
    if (( __pse_status == 0 )); then
        for __pse_record in "${__pse_records[@]}"; do
            case $__pse_record in
                +*) export "${__pse_record:1}";;
                -*) unset "${__pse_record:1}";;
            esac
        done
    fi
    if (( __pse_trace )); then set -x; fi
    return "$__pse_status"
}
if __pse_activate; then
    unset -f __pse_activate
else
    unset -f __pse_activate
    false
fi
"#;

/// The declaration that marks a checkout root.
const CHECKOUT_MARKER: &str = ".config/diagnostic-tools.toml";

/// The checkout that owns a published host-tools generation executable
/// (`<checkout>/target/host-tools/<generation>/<tool>`), if `executable` is one.
fn generation_checkout(executable: &Path) -> Option<PathBuf> {
    let generation = executable.parent()?;
    let tools = generation.parent()?;
    let target = tools.parent()?;
    let root = target.parent()?;
    (tools.file_name() == Some(OsStr::new("host-tools"))
        && target.file_name() == Some(OsStr::new("target"))
        && root.join(CHECKOUT_MARKER).is_file())
    .then(|| root.to_path_buf())
}

/// The nearest enclosing checkout root of `start`, walking up to the checkout marker.
fn enclosing_checkout(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|path| path.join(CHECKOUT_MARKER).is_file())
        .map(Path::to_path_buf)
}

/// Locate the checkout: an explicit selection (walked up to its root), then the checkout
/// owning the running host-tools generation, then the current directory walked up.
/// Pure over its inputs; `checkout_root` supplies the process facts.
pub fn locate_checkout(
    explicit: Option<&Path>,
    executable: Option<&Path>,
    cwd: &Path,
) -> Result<PathBuf, EnvironmentError> {
    if let Some(explicit) = explicit {
        let start = if explicit.is_absolute() {
            explicit.to_path_buf()
        } else {
            cwd.join(explicit)
        };
        return enclosing_checkout(&start).ok_or_else(|| {
            failure(format!(
                "{} is not inside a pse-arrow checkout",
                explicit.display()
            ))
        });
    }
    executable
        .and_then(generation_checkout)
        .or_else(|| enclosing_checkout(cwd))
        .ok_or_else(|| failure("run inside a pse-arrow checkout or select one with -C"))
}

/// The single checkout self-location used by every host verb.
pub fn checkout_root(explicit: Option<&Path>) -> Result<PathBuf, EnvironmentError> {
    let cwd = std::env::current_dir().map_err(|error| failure(error.to_string()))?;
    locate_checkout(explicit, std::env::current_exe().ok().as_deref(), &cwd)
}

/// One fixed, source-free environment read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentRead {
    /// The fixed Bash activation program; it never contains private values.
    Print,
    /// The selected values as JSON, without evaluating personal code.
    Explain,
    /// The private NUL activation transfer the printed program reads from a pipe.
    ActivationData,
}

/// Fixed environment output: explanation and script generation never evaluate personal code.
pub fn read(
    root: &Path,
    caller: &Environment,
    operation: EnvironmentRead,
    docs: bool,
    cache: CacheMode,
) -> Result<Vec<u8>, EnvironmentError> {
    Ok(match operation {
        EnvironmentRead::Print => activation_script(
            &std::env::current_exe().map_err(|error| failure(error.to_string()))?,
            root,
            docs,
            cache,
        )?
        .into_bytes(),
        EnvironmentRead::ActivationData => {
            use std::os::unix::fs::FileTypeExt as _;
            let kind = std::fs::metadata("/proc/self/fd/1")
                .map_err(|error| failure(error.to_string()))?
                .file_type();
            if !kind.is_fifo() && !kind.is_socket() {
                return Err(failure("activation data requires a private pipe"));
            }
            activate_with_options(root, caller, &Environment::new(), cache, docs)?
                .activation_transfer(caller)?
        }
        EnvironmentRead::Explain => {
            let plan = resolve_with_options(root, caller, None, &Environment::new(), cache, docs)?;
            let selected: [&str; 9] = [
                "PSE_BUILD_MODE",
                "CARGO_BUILD_JOBS",
                "RUSTC_WRAPPER",
                "SCCACHE_CLIENT_SIDE",
                "SCCACHE_DIR",
                "PSE_SURREAL_STATE",
                "UV_PROJECT_ENVIRONMENT",
                "PSE_PYTHON",
                "PSE_TELEMETRY_CONFIGURATION_STATUS",
            ];
            let values: BTreeMap<_, _> = selected
                .into_iter()
                .map(|name| (name, get(&plan.values, name).and_then(OsStr::to_str)))
                .collect();
            (serde_json::to_string_pretty(
                &serde_json::json!({"local_unresolved":plan.local_unresolved,"selections":values}),
            )
            .map_err(|error| failure(error.to_string()))?
                + "\n")
                .into_bytes()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checkout_location_prefers_explicit_then_generation_then_cwd() {
        let base = tempfile::tempdir().unwrap();
        let checkout = |name: &str| {
            let root = base.path().join(name);
            std::fs::create_dir_all(root.join(".config")).unwrap();
            std::fs::create_dir_all(root.join("target/host-tools/generation.abc")).unwrap();
            std::fs::create_dir_all(root.join("target/debug")).unwrap();
            std::fs::create_dir_all(root.join("crates/x")).unwrap();
            std::fs::write(root.join(CHECKOUT_MARKER), "").unwrap();
            root
        };
        let (own, other, explicit) = (checkout("own"), checkout("other"), checkout("explicit"));
        let generation = own.join("target/host-tools/generation.abc/pse");
        let cwd = other.join("crates/x");
        assert_eq!(
            locate_checkout(Some(&explicit.join("crates/x")), Some(&generation), &cwd).unwrap(),
            explicit
        );
        assert_eq!(
            locate_checkout(Some(Path::new("crates")), None, &explicit).unwrap(),
            explicit
        );
        assert!(locate_checkout(Some(base.path()), Some(&generation), &cwd).is_err());
        assert_eq!(locate_checkout(None, Some(&generation), &cwd).unwrap(), own);
        // A Cargo-built executable is not a published generation: the cwd decides.
        assert_eq!(
            locate_checkout(None, Some(&own.join("target/debug/pse")), &cwd).unwrap(),
            other
        );
        assert!(locate_checkout(None, None, base.path()).is_err());
    }
    #[test]
    fn environment_invalid_toml_does_not_quote_private_configuration() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("private.toml");
        std::fs::write(&path, "credential = 'private-credential-value\n").unwrap();
        let error = read_toml(&path).unwrap_err().to_string();
        assert!(!error.contains("private-credential-value"));
        assert!(error.contains("invalid configuration"));
    }
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
            rust_stack_bytes: 16 << 20,
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
    fn local_secret_is_private_transfer_and_unresolved_transfer_refuses() {
        let caller = map(&[("HOME", "/home/test")]);
        let mut settings = settings();
        settings.local_pending = true;
        let local = map(&[("API_TOKEN", "secret-do-not-render")]);
        let plan = compose(&settings, &caller, &local, &Environment::new()).unwrap();
        let rendered = activation_script(
            Path::new("/native/host"),
            Path::new("/checkout"),
            false,
            CacheMode::Auto,
        )
        .unwrap();
        assert!(!rendered.contains("secret-do-not-render"));
        assert!(
            plan.activation_transfer(&caller)
                .unwrap()
                .split(|byte| *byte == 0)
                .any(|part| part == b"+API_TOKEN=secret-do-not-render")
        );
        assert!(
            compose(&settings, &caller, &Environment::new(), &Environment::new())
                .unwrap()
                .activation_transfer(&caller)
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
        std::fs::write(
            root.path().join(".config/agent-capacity.toml"),
            "[test-runner]\nrust_thread_stack_mib=16\n",
        )
        .unwrap();
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
    fn cache_off_skips_stale_cache_selection_and_on_refuses_foreign_wrapper() {
        let root = declaration_fixture();
        let caller = map(&[
            ("HOME", root.path().to_str().unwrap()),
            ("PSE_SCCACHE_BINARY", "/missing/cache"),
        ]);
        let off = resolve_with_options(
            root.path(),
            &caller,
            Some(&Environment::new()),
            &Environment::new(),
            CacheMode::Off,
            false,
        )
        .unwrap();
        assert_eq!(get(&off.values, "RUSTC_WRAPPER"), Some(OsStr::new("")));
        assert!(
            compose_with_cache(
                &settings(),
                &map(&[
                    ("HOME", "/home/test"),
                    ("RUSTC_WRAPPER", "/foreign/wrapper")
                ]),
                &Environment::new(),
                &Environment::new(),
                CacheMode::On
            )
            .is_err()
        );
    }

    #[test]
    fn rust_stack_uses_one_policy_scalar_and_preserves_valid_explicit_bytes() {
        let root = declaration_fixture();
        let mut caller = map(&[("HOME", root.path().to_str().unwrap())]);
        let defaults = resolve(
            root.path(),
            &caller,
            &Environment::new(),
            &Environment::new(),
        )
        .unwrap();
        assert_eq!(
            get(&defaults.values, "RUST_MIN_STACK"),
            Some(OsStr::new("16777216"))
        );
        insert(&mut caller, "RUST_MIN_STACK", "0004096");
        assert_eq!(
            get(
                &resolve(
                    root.path(),
                    &caller,
                    &Environment::new(),
                    &Environment::new()
                )
                .unwrap()
                .values,
                "RUST_MIN_STACK"
            ),
            Some(OsStr::new("0004096"))
        );
        for invalid in ["", "0", "-1", "1.5", " 1024", "18446744073709551616"] {
            insert(&mut caller, "RUST_MIN_STACK", invalid);
            assert!(
                resolve(
                    root.path(),
                    &caller,
                    &Environment::new(),
                    &Environment::new()
                )
                .is_err()
            );
        }
        caller.remove(OsStr::new("RUST_MIN_STACK"));
        std::fs::write(
            root.path().join(".config/agent-capacity.toml"),
            "[test-runner]\nrust_thread_stack_mib=0\n",
        )
        .unwrap();
        assert!(
            resolve(
                root.path(),
                &caller,
                &Environment::new(),
                &Environment::new()
            )
            .is_err()
        );
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
        let shell = activation_script(
            Path::new("/native/host"),
            root.path(),
            false,
            CacheMode::Auto,
        )
        .unwrap();
        assert!(!shell.contains("private-local-secret"));
        assert!(plan.activation_transfer(&caller).is_ok());
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
        assert!(plan.activation_transfer(&caller).is_ok());
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
            rust_stack_bytes: 16 << 20,
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
