// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Read-only native checkout diagnostics; no dependency or service preparation.
use crate::{
    diagnostic_tools::{self, Tool},
    environment::{self, Environment, EnvironmentSettings},
};
use serde::Serialize;
use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::io::AsyncReadExt as _;

/// One observed prerequisite, separate from optional tool readiness.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Check {
    /// Stable diagnostic subject.
    pub name: String,
    /// Whether the stated observation succeeded.
    pub ok: bool,
    /// Whether its failure prevents the ordinary native development route.
    pub blocking: bool,
    /// Observed fact or original unavailable/mismatch reason.
    pub detail: String,
    /// Explicit repair guidance; diagnosis never executes it.
    pub fix: String,
}
impl Check {
    fn new(
        name: impl Into<String>,
        blocking: bool,
        outcome: Result<String, String>,
        fix: impl Into<String>,
    ) -> Self {
        let ok = outcome.is_ok();
        Self {
            name: name.into(),
            ok,
            blocking,
            detail: outcome.unwrap_or_else(|error| error),
            fix: if ok { String::new() } else { fix.into() },
        }
    }
}
/// Point-in-time native readiness, with unavailable optional checks retained.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Report {
    /// Native diagnostic output interpretation.
    pub version: u32,
    /// Independent prerequisite observations in stable name order.
    pub checks: Vec<Check>,
}
impl Report {
    /// All required observations succeeded; optional absence is not hidden success.
    pub fn ready(&self) -> bool {
        self.checks.iter().all(|check| check.ok || !check.blocking)
    }
}

struct Probe {
    name: String,
    program: PathBuf,
    arguments: Vec<OsString>,
    expected: Option<String>,
    components: Option<Vec<String>>,
    cache_environment: Option<Environment>,
    blocking: bool,
    fix: String,
    timeout: Duration,
}
impl Probe {
    fn version(
        name: impl Into<String>,
        program: PathBuf,
        expected: Option<String>,
        blocking: bool,
        fix: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            program,
            arguments: vec!["--version".into()],
            expected,
            components: None,
            cache_environment: None,
            blocking,
            fix: fix.into(),
            timeout: Duration::from_secs(10),
        }
    }
    fn assess(&self, text: &str) -> Result<String, String> {
        let actual = text.trim();
        if actual.is_empty() {
            return Err("tool returned no version/availability observation".into());
        }
        if let Some(expected) = &self.expected {
            let present = actual.split_whitespace().any(|token| {
                token
                    .trim_matches(|character: char| {
                        !character.is_ascii_alphanumeric() && character != '.' && character != '-'
                    })
                    .trim_start_matches('v')
                    == expected
            });
            if !present {
                return Err(format!(
                    "expected exact version {expected}; observed {actual}"
                ));
            }
        }
        if let Some(required) = &self.components {
            let missing = required
                .iter()
                .filter(|name| {
                    !actual.split_whitespace().any(|item| {
                        item == name.as_str()
                            || item
                                .strip_prefix(name.as_str())
                                .is_some_and(|rest| rest.starts_with('-'))
                    })
                })
                .collect::<Vec<_>>();
            if !missing.is_empty() {
                return Err(format!("missing pinned toolchain components: {missing:?}"));
            }
        }
        Ok(actual.into())
    }
    async fn run(self, root: &Path) -> Check {
        let outcome = self.execute(root).await;
        Check::new(self.name, self.blocking, outcome, self.fix)
    }
    async fn execute(&self, root: &Path) -> Result<String, String> {
        let mut command = tokio::process::Command::new(&self.program);
        if let Some(environment) = &self.cache_environment {
            let endpoint = cache_endpoint(environment)?;
            if observed_cache_listeners(endpoint)? == 0 {
                return Err(format!("no live daemon observed at {}", endpoint.to_string_lossy()));
            }
            command.env_clear().envs(environment);
        }
        let mut child = command.args(&self.arguments)
            .current_dir(root)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|error| format!("{} unavailable: {error}", self.program.display()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "version stdout unavailable".to_owned())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "version stderr unavailable".to_owned())?;
        let wait = async {
            match tokio::time::timeout(self.timeout, child.wait()).await {
                Ok(status) => status.map_err(|error| error.to_string()),
                Err(_) => {
                    let killed = child.start_kill();
                    let drained = tokio::time::timeout(Duration::from_secs(5), child.wait()).await;
                    Err(format!(
                        "tool version observation timed out; stop={killed:?}; drain={drained:?}"
                    ))
                }
            }
        };
        // Drain both bounded pipes while waiting, so even broken optional tools
        // cannot block each other or fill an unread pipe indefinitely.
        let (status, stdout, stderr) = tokio::join!(
            wait,
            output(stdout, self.timeout + Duration::from_secs(5)),
            output(stderr, self.timeout + Duration::from_secs(5))
        );
        let status = status?;
        let (stdout, stderr) = (stdout?, stderr?);
        if !status.success() {
            if self.cache_environment.is_some() {
                return Err(format!("sccache daemon observation failed with {status}"));
            }
            return Err(format!(
                "{} exited {status}: {}",
                self.program.display(),
                String::from_utf8_lossy(&stderr).trim()
            ));
        }
        if let Some(environment) = &self.cache_environment {
            let observed = assess_cache_daemon(&stdout, environment, self.expected.as_deref())?;
            return match observed_cache_listeners(cache_endpoint(environment)?)? {
                1 => Ok(observed),
                0 => Err(format!("{observed}; listener no longer observed")),
                count => Err(format!("{observed}; {count} listening sockets claim this endpoint")),
            };
        }
        let mut text = stdout;
        text.extend_from_slice(&stderr);
        let observed = self.assess(&String::from_utf8_lossy(&text))?;
        if self.name == "compiler-cache" {
            return Ok(format!("{observed} at {}", self.program.display()));
        }
        Ok(observed)
    }
}

fn cache_endpoint(environment: &Environment) -> Result<&OsStr, String> {
    environment
        .get(OsStr::new("SCCACHE_SERVER_UDS"))
        .map(OsString::as_os_str)
        .ok_or_else(|| "sccache Unix endpoint unavailable".into())
}

fn observed_cache_listeners(endpoint: &OsStr) -> Result<usize, String> {
    let table = std::fs::read("/proc/net/unix")
        .map_err(|error| format!("sccache listener ownership unavailable: {error}"))?;
    Ok(unix_listener_count(&table, endpoint))
}

fn unix_listener_count(table: &[u8], endpoint: &OsStr) -> usize {
    use std::os::unix::ffi::OsStrExt as _;
    let endpoint = endpoint.as_bytes();
    let abstract_name = endpoint.strip_prefix(b"\\x00");
    table
        .split(|byte| *byte == b'\n')
        .filter(|line| {
            // Procfs labels live AF_UNIX stream listeners with SO_ACCEPTCON
            // and prints an abstract address with '@' in place of its first NUL.
            let mut fields = line.splitn(8, |byte| *byte == b' ');
            if fields.nth(3) != Some(&b"00010000"[..])
                || fields.next() != Some(&b"0001"[..])
            {
                return false;
            }
            let Some(path) = fields.nth(2) else { return false };
            match abstract_name {
                Some(name) => path.strip_prefix(b"@") == Some(name),
                None => path == endpoint,
            }
        })
        .count()
}

#[derive(serde::Deserialize)]
struct CacheDaemonStats {
    version: String,
    effective_cache_mode: Option<String>,
    cache_location: String,
    cache_size: Option<u64>,
    max_cache_size: Option<u64>,
    use_preprocessor_cache_mode: bool,
}

fn assess_cache_daemon(
    output: &[u8],
    environment: &Environment,
    expected: Option<&str>,
) -> Result<String, String> {
    let stats: CacheDaemonStats = serde_json::from_slice(output)
        .map_err(|_| "sccache daemon statistics did not match the 0.19 format".to_owned())?;
    let selected = |name: &str| {
        environment.get(OsStr::new(name)).and_then(|value| value.to_str()).unwrap_or("unselected")
    };
    let endpoint = selected("SCCACHE_SERVER_UDS");
    let mode = stats.effective_cache_mode.as_deref().ok_or_else(|| {
        format!("no live daemon observed at {endpoint}; sccache returned configured defaults")
    })?;
    if let Some(expected) = expected
        && stats.version != expected
    {
        return Err(format!("daemon at {endpoint} has version {}; expected {expected}", stats.version));
    }
    // Remote location metadata can contain endpoint details. Report the backend
    // name only. The local path is checked against the selected directory.
    let backend = if stats.cache_location.starts_with("Local disk: ") {
        let directory = Path::new(selected("SCCACHE_DIR"));
        if stats.cache_location != format!("Local disk: {directory:?}") {
            return Err(format!("daemon at {endpoint} uses a different local cache directory"));
        }
        "disk"
    } else if stats.cache_location.starts_with("Multi-level (") {
        "multi-level"
    } else {
        stats.cache_location.split_once(',').map_or("unknown", |(name, _)| name)
    };
    let extent = |bytes: Option<u64>| bytes.map_or_else(|| "unreported".into(), |bytes| bytes.to_string());
    let request = if environment.contains_key(OsStr::new("SCCACHE_NO_CACHE")) {
        "bypass"
    } else if environment.contains_key(OsStr::new("SCCACHE_RECACHE")) {
        "recache"
    } else {
        "normal"
    };
    Ok(format!(
        "sccache {} at {endpoint}; backend {backend}; mode {mode}; current bytes {}; maximum bytes {}; daemon direct preprocessing {}; client-side compilation {}; client frame bytes {}; request {request}",
        stats.version,
        extent(stats.cache_size),
        extent(stats.max_cache_size),
        stats.use_preprocessor_cache_mode,
        selected("SCCACHE_CLIENT_SIDE"),
        selected("SCCACHE_MAX_FRAME_LENGTH"),
    ))
}
async fn output(
    input: impl tokio::io::AsyncRead + Unpin,
    timeout: Duration,
) -> Result<Vec<u8>, String> {
    const LIMIT: u64 = 1024 * 1024;
    let mut bytes = Vec::new();
    tokio::time::timeout(timeout, input.take(LIMIT + 1).read_to_end(&mut bytes))
        .await
        .map_err(|_| "version output did not drain".to_owned())?
        .map_err(|error| error.to_string())?;
    if bytes.len() > LIMIT as usize {
        return Err("version output exceeds the diagnostic allowance".into());
    }
    Ok(bytes)
}
fn load(path: &Path) -> Result<toml::Value, String> {
    toml::from_str(
        &std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?,
    )
    .map_err(|error| format!("{}: {error}", path.display()))
}
fn executable(name: &str, environment: &Environment) -> PathBuf {
    environment
        .get(OsStr::new("PATH"))
        .and_then(|paths| {
            std::env::split_paths(paths)
                .map(|path| path.join(name))
                .find(|path| path.is_file())
        })
        .unwrap_or_else(|| PathBuf::from(name))
}

const CACHE_FIX: &str = "prepare the declared sccache or select direct compilation explicitly";

fn compiler_cache_probes(root: &Path, values: Environment) -> Result<[Probe; 2], String> {
    let wrapper = values.get(OsStr::new("RUSTC_WRAPPER"));
    let Some(wrapper) = wrapper.filter(|wrapper| !wrapper.is_empty()) else {
        return Err(if wrapper.is_some() {
            "effective compiler route is direct".into()
        } else {
            "managed compiler cache was not selected by the effective environment".into()
        });
    };
    let binary = PathBuf::from(wrapper);
    if binary.file_name() != Some(OsStr::new("sccache")) {
        return Err(format!("effective compiler wrapper is {}", binary.display()));
    }
    let expected = diagnostic_tools::tool_spec(root, Tool::Sccache)
        .map_err(|error| error.to_string())?
        .version;
    if binary == diagnostic_tools::binary_path(root, Tool::Sccache, &values)
        .map_err(|error| error.to_string())?
    {
        diagnostic_tools::verify(root, Tool::Sccache, &values)
            .map_err(|error| error.to_string())?;
    }
    let binary = if binary == Path::new("sccache") {
        executable("sccache", &values)
    } else {
        binary
    };
    let version = Probe::version("compiler-cache", binary.clone(), Some(expected.clone()), false, CACHE_FIX);
    let mut daemon = Probe::version(
        "compiler-cache-daemon",
        binary,
        Some(expected),
        false,
        "use the effective compiler-cache endpoint and inspect its existing daemon",
    );
    daemon.arguments = ["--show-stats", "--stats-format=json"].map(OsString::from).to_vec();
    daemon.cache_environment = Some(values);
    Ok([version, daemon])
}

fn declarations(root: &Path, environment: &Environment) -> (Vec<Check>, Vec<Probe>) {
    let mut checks = Vec::new();
    let mut probes = Vec::new();
    let configured = EnvironmentSettings::load(root, environment).and_then(|settings| {
        let plan = environment::compose(
            &settings,
            environment,
            &Environment::new(),
            &Environment::new(),
        )?;
        Ok((settings, plan))
    });
    match configured {
        Ok((settings, plan)) => {
            checks.push(Check::new(
                "environment",
                true,
                Ok(format!(
                    "native composition available; Cargo jobs {}; local file unresolved={}",
                    settings.jobs, plan.local_unresolved
                )),
                "",
            ));
            match compiler_cache_probes(root, plan.values) {
                Ok(cache) => probes.extend(cache),
                Err(reason) => checks.push(Check::new("compiler-cache", false, Err(reason), CACHE_FIX)),
            }
        }
        Err(error) => checks.push(Check::new(
            "environment",
            true,
            Err(error.to_string()),
            "inspect declared checkout configuration and explicit environment selections",
        )),
    }
    let admission = crate::admission::validate_request(environment)
        .and_then(|()| crate::admission::Selection::load(root, environment, None, None))
        .map(|selection| {
            format!(
                "selected profile {}; policy loaded without admission or resource mutation",
                selection.profile().name
            )
        })
        .map_err(|error| error.to_string());
    checks.push(Check::new(
        "admission-configuration",
        true,
        admission,
        "inspect .config/agent-capacity.toml and explicit placement selections",
    ));
    for name in ["pse-host", "cargo-pse-correctness"] {
        probes.push(Probe::version(
            format!("prepared-{name}"),
            root.join("target/host-tools/current").join(name),
            Some(env!("CARGO_PKG_VERSION").into()),
            true,
            "run scripts/bootstrap-host explicitly",
        ));
    }
    let toolchain = load(&root.join("rust-toolchain.toml")).and_then(|value| {
        let settings = value
            .get("toolchain")
            .ok_or_else(|| "toolchain declaration unavailable".to_owned())?;
        let channel = settings
            .get("channel")
            .and_then(toml::Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "toolchain.channel must be nonempty".to_owned())?
            .to_owned();
        let components = settings
            .get("components")
            .and_then(toml::Value::as_array)
            .ok_or_else(|| "toolchain.components must be an array".to_owned())?
            .iter()
            .map(|component| {
                component
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| "toolchain components must be strings".into())
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok((channel, components))
    });
    match toolchain {
        Ok((channel, components)) => {
            let rustup = executable("rustup", environment);
            for tool in ["rustc", "cargo"] {
                let mut probe = Probe::version(
                    format!("pinned-{tool}"),
                    rustup.clone(),
                    None,
                    true,
                    format!("install the declared {channel} toolchain explicitly"),
                );
                probe.arguments = ["run", &channel, tool, "--version"]
                    .map(OsString::from)
                    .into();
                probes.push(probe);
            }
            let mut probe = Probe::version(
                "rust-components",
                rustup.clone(),
                None,
                true,
                "install the components declared in rust-toolchain.toml",
            );
            probe.arguments = ["component", "list", "--toolchain", &channel, "--installed"]
                .map(OsString::from)
                .into();
            probe.components = Some(components);
            probes.push(probe);
            match load(&root.join(".config/diagnostic-tools.toml")) {
                Ok(tools) => {
                    if let Some(cargo_tools) =
                        tools.get("cargo-tools").and_then(toml::Value::as_table)
                    {
                        for (name, wanted) in cargo_tools {
                            let Some(wanted) = wanted.as_str() else {
                                checks.push(Check::new(
                                    name,
                                    true,
                                    Err("declared tool version must be a string".into()),
                                    "fix .config/diagnostic-tools.toml",
                                ));
                                continue;
                            };
                            let Some(subcommand) = name.strip_prefix("cargo-") else {
                                checks.push(Check::new(
                                    name,
                                    true,
                                    Err("Cargo tool declaration must use cargo- prefix".into()),
                                    "fix .config/diagnostic-tools.toml",
                                ));
                                continue;
                            };
                            let mut probe = Probe::version(
                                name,
                                rustup.clone(),
                                Some(wanted.into()),
                                true,
                                format!("install {name} at declared version {wanted} explicitly"),
                            );
                            probe.arguments = ["run", &channel, "cargo", subcommand, "--version"]
                                .map(OsString::from)
                                .into();
                            probes.push(probe);
                        }
                    } else {
                        checks.push(Check::new(
                            "cargo-tools",
                            true,
                            Err("cargo-tools declaration unavailable".into()),
                            "fix .config/diagnostic-tools.toml",
                        ));
                    }
                    for (name, tool) in tools
                        .as_table()
                        .into_iter()
                        .flatten()
                        .filter(|(name, _)| *name != "cargo-tools" && *name != "sccache")
                    {
                        if let (Some(version), Some(_member)) = (
                            tool.get("version").and_then(toml::Value::as_str),
                            tool.get("member").and_then(toml::Value::as_str),
                        ) {
                            match Tool::parse(name)
                                .and_then(|tool| diagnostic_tools::verify(root, tool, environment))
                            {
                                Ok(binary) => probes.push(Probe::version(format!("optional-{name}"), binary, Some(version.into()), false, "prepare the selected optional diagnostic tool explicitly")),
                                Err(error) => checks.push(Check::new(format!("optional-{name}"), false, Err(error.to_string()), "prepare the selected optional diagnostic tool explicitly; doctor never repairs an unverified installation")),
                            }
                        } else {
                            checks.push(Check::new(
                                format!("optional-{name}"),
                                false,
                                Err("optional tool requires declared version and member strings"
                                    .into()),
                                "fix .config/diagnostic-tools.toml",
                            ));
                        }
                    }
                }
                Err(error) => checks.push(Check::new(
                    "cargo-tools",
                    true,
                    Err(error),
                    "fix .config/diagnostic-tools.toml",
                )),
            }
        }
        Err(error) => checks.push(Check::new(
            "rust-toolchain",
            true,
            Err(error),
            "restore the declared rust-toolchain.toml",
        )),
    }
    match load(&root.join("docs/site.toml")) {
        Ok(site) => {
            if let Some(tools) = site.get("tools").and_then(toml::Value::as_table) {
                for (name, version) in tools {
                    if let Some(version) = version.as_str() {
                        probes.push(Probe::version(
                            format!("optional-docs-{name}"),
                            executable(name, environment),
                            Some(version.into()),
                            false,
                            "prepare the documentation tools declared in docs/site.toml explicitly",
                        ));
                    } else {
                        checks.push(Check::new(
                            format!("optional-docs-{name}"),
                            false,
                            Err("documentation tool version must be a string".into()),
                            "fix docs/site.toml",
                        ));
                    }
                }
            } else {
                checks.push(Check::new(
                    "optional-docs",
                    false,
                    Err("documentation tools declaration unavailable".into()),
                    "fix docs/site.toml",
                ));
            }
        }
        Err(error) => checks.push(Check::new(
            "optional-docs",
            false,
            Err(error),
            "restore docs/site.toml for documentation publication",
        )),
    }
    for name in ["actionlint", "ast-grep", "shellcheck"] {
        probes.push(Probe::version(
            format!("optional-linter-{name}"),
            executable(name, environment),
            None,
            false,
            "install the selected repository linter explicitly",
        ));
    }
    checks.push(Check::new("native-providers", false, Err("readiness not observed; doctor does not select source profiles, build native libraries or initialize scientific callers".into()), "prepare requested capabilities explicitly with pse-host native prepare --capabilities; the workload boundary selects the completed profile"));
    checks.push(Check::new("canonical-readiness", false, Err("readiness not observed; each explicit application/fixture owns its schema-ready canonical capability".into()), "construct the selected native canonical caller explicitly"));
    (checks, probes)
}

/// Diagnose independent native prerequisites concurrently without activation side effects.
pub async fn inspect(root: &Path, environment: &Environment) -> Report {
    let (mut checks, mut probes) = declarations(root, environment);
    // `rustup run` without --install refuses an absent channel. Establish that
    // premise before component commands, which must never prepare a missing SDK.
    if let Some(index) = probes.iter().position(|probe| probe.name == "pinned-rustc") {
        let rustc = probes.remove(index).run(root).await;
        if !rustc.ok {
            probes.retain(|probe| {
                if probe.arguments.first().is_some_and(|argument| argument == "run" || argument == "component") {
                    checks.push(Check::new(&probe.name, probe.blocking, Err("pinned toolchain unavailable; dependent version/component query was not executed".into()), &probe.fix));
                    false
                } else { true }
            });
        }
        checks.push(rustc);
    }
    let mut pending = tokio::task::JoinSet::new();
    for probe in probes {
        let root = root.to_owned();
        pending.spawn(async move { probe.run(&root).await });
    }
    pending.spawn(async {
        let outcome = match tokio::time::timeout(Duration::from_secs(10), crate::activity::ActivityObservation::read(None)).await {
            Ok(observation) if observation.complete() => Ok("user-manager and observer kernel facts available; no workload was admitted".into()),
            Ok(observation) => Err(format!("manager unavailable: {}; observer unavailable: {}", observation.manager_unavailable.as_deref().unwrap_or("not established"), observation.observer.unavailable.join("; "))),
            Err(_) => Err("user-manager observation timed out".into()),
        };
        Check::new("user-manager", true, outcome, "restore this user's systemd session bus; use pse-host activity for independent diagnosis")
    });
    while let Some(result) = pending.join_next().await {
        match result {
            Ok(check) => checks.push(check),
            Err(error) => checks.push(Check::new(
                "diagnostic-task",
                true,
                Err(error.to_string()),
                "inspect the failed diagnostic task",
            )),
        }
    }
    checks.sort_by(|left, right| left.name.cmp(&right.name));
    Report { version: 1, checks }
}

#[cfg(test)]
#[path = "doctor_tests.rs"]
mod tests;
