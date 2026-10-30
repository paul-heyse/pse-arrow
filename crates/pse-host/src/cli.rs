// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One command grammar and native dispatch for the host entry points.
use crate::{HostError, environment::Environment};
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::{
    ffi::OsString,
    io::{self, Write},
    path::{Path, PathBuf},
    process::ExitCode,
    time::{Duration, Instant},
};

/// Selects the fixed executable protocol, never inferred from an executable filename.
#[derive(Clone, Copy, Debug)]
pub enum BinKind {
    /// Agent-facing command name.
    Pse,
    /// Native host and hook command name.
    Host,
    /// External Cargo command and raw adapter protocol.
    Correctness,
}

#[derive(Debug, Parser)]
#[command(
    name = "pse",
    about = "Native host capabilities; explicit preparation: scripts/bootstrap-host",
    disable_version_flag = true
)]
pub(crate) struct Cli {
    #[arg(short = 'C', global = true, id = "checkout")]
    pub(crate) root: Option<PathBuf>,
    #[arg(long, exclusive = true)]
    version: bool,
    #[command(subcommand)]
    command: Option<Command>,
}
#[derive(Debug, Subcommand)]
enum Command {
    #[command(name = "__host-generation-bootstrap", hide = true)]
    HostGenerationBootstrap,
    #[command(
        hide = true,
        before_help = "pse-env — native command and environment boundary"
    )]
    Boundary(BoundaryArgs),
    #[command(hide = true)]
    AgentHook {
        #[command(subcommand)]
        action: HookAction,
    },
    Doctor {
        #[arg(long)]
        json: bool,
    },
    CargoTools {
        names: Vec<String>,
    },
    DiagnosticTools {
        #[command(subcommand)]
        action: ToolAction,
    },
    TelemetryService {
        #[command(subcommand)]
        action: TelemetryAction,
    },
    Native {
        #[command(subcommand)]
        action: NativeAction,
    },
    Server {
        #[command(subcommand)]
        action: ServerAction,
    },
    StudyModule {
        #[command(subcommand)]
        action: StudyAction,
    },
    Env(EnvironmentArgs),
    Activity {
        unit: Option<String>,
    },
    Run {
        #[arg(trailing_var_arg = true, required = true)]
        command: Vec<OsString>,
    },
}
#[derive(Debug, Subcommand)]
pub(crate) enum HookAction {
    Guard,
    SessionEnv,
}
#[derive(Debug, Subcommand)]
enum ToolAction {
    Install { tool: String },
    Verify { tool: String },
}
#[derive(Debug, Subcommand)]
enum NativeAction {
    Prepare {
        #[arg(long, required = true, value_parser = capabilities)]
        capabilities: String,
    },
}
fn capabilities(value: &str) -> Result<String, String> {
    crate::native::validate_capabilities(value).map_err(|error| error.to_string())?;
    Ok(value.into())
}
#[derive(Debug, Subcommand)]
pub(crate) enum ServerAction {
    Prepare {
        #[arg(long)]
        from_release: Option<PathBuf>,
    },
    Select,
}
#[derive(Debug, Args)]
pub(crate) struct StudyVariant {
    #[arg(long)]
    pub(crate) qualification: bool,
    #[arg(long, requires = "qualification")]
    pub(crate) long_hold: bool,
}
#[derive(Debug, Subcommand)]
pub(crate) enum StudyAction {
    Prepare {
        #[arg(long)]
        tool_root: Option<PathBuf>,
        #[command(flatten)]
        variant: StudyVariant,
    },
    Select {
        #[arg(long)]
        receipt: PathBuf,
        #[command(flatten)]
        variant: StudyVariant,
    },
}
#[derive(Debug, Subcommand)]
pub(crate) enum TelemetryAction {
    Install,
    Setup(TelemetrySetup),
    Inspect(TelemetryState),
    Start(TelemetryState),
    Stop(TelemetryState),
    Reconfigure(TelemetryState),
    RecoverConfiguration(TelemetryState),
}
#[derive(Debug, Args)]
pub(crate) struct TelemetryState {
    #[arg(long)]
    pub(crate) state: Option<PathBuf>,
}
#[derive(Debug, Args)]
pub(crate) struct TelemetrySetup {
    #[arg(long)]
    pub(crate) state: Option<PathBuf>,
    #[arg(long = "port", value_parser = named_port)]
    pub(crate) ports: Vec<(String, u16)>,
    #[arg(long, default_value = "default", overrides_with = "organization")]
    pub(crate) organization: String,
    #[arg(long = "stream", value_parser = named_value)]
    pub(crate) streams: Vec<(String, String)>,
    #[arg(long, default_value_t = 7, overrides_with = "retention_days")]
    pub(crate) retention_days: u32,
}
fn named_value(value: &str) -> Result<(String, String), String> {
    value
        .split_once('=')
        .map(|(name, value)| (name.into(), value.into()))
        .ok_or_else(|| "expected NAME=VALUE".into())
}
fn named_port(value: &str) -> Result<(String, u16), String> {
    let (name, port) = named_value(value)?;
    Ok((
        name,
        port.parse()
            .map_err(|_| "port must be an integer from 1 through 65535")?,
    ))
}
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub(crate) enum CacheArg {
    #[default]
    Auto,
    On,
    Off,
}
impl From<CacheArg> for crate::environment::CacheMode {
    fn from(value: CacheArg) -> Self {
        match value {
            CacheArg::Auto => Self::Auto,
            CacheArg::On => Self::On,
            CacheArg::Off => Self::Off,
        }
    }
}
#[derive(Debug, Args)]
#[group(skip)]
#[command(args_override_self = true)]
#[command(group(clap::ArgGroup::new("operation").required(true).args(["print", "explain", "activation_data"])))]
pub(crate) struct EnvironmentArgs {
    #[arg(long)]
    pub(crate) print: bool,
    #[arg(long)]
    pub(crate) explain: bool,
    #[arg(long, requires = "explain")]
    pub(crate) json: bool,
    #[arg(long, hide = true)]
    pub(crate) activation_data: bool,
    #[arg(long)]
    pub(crate) docs: bool,
    #[arg(long, requires = "activation_data")]
    pub(crate) root: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "auto")]
    pub(crate) cache: CacheArg,
}
#[derive(Debug, Args)]
#[command(args_override_self = true)]
pub(crate) struct BoundaryArgs {
    #[arg(long)]
    pub(crate) print: bool,
    #[arg(long)]
    pub(crate) explain: bool,
    #[arg(long)]
    pub(crate) json: bool,
    #[arg(long)]
    pub(crate) docs: bool,
    #[arg(long)]
    pub(crate) no_scope: bool,
    #[arg(long, value_parser = resource_class)]
    pub(crate) resource_class: Option<String>,
    #[arg(long, value_parser = resource_class)]
    pub(crate) recipe_default_class: Option<String>,
    #[arg(long, value_enum, default_value = "auto")]
    pub(crate) cache: CacheArg,
    #[arg(long, num_args = 0..=1, require_equals = true)]
    pub(crate) native: Option<Option<String>>,
    #[arg(trailing_var_arg = true)]
    pub(crate) command: Vec<OsString>,
}
fn resource_class(value: &str) -> Result<String, String> {
    crate::admission::validate_class(value).map_err(|error| error.to_string())?;
    Ok(value.into())
}

#[derive(Parser)]
#[command(
    name = "cargo pse-correctness",
    about = "Source-build correctness; Cargo and native nextest arguments remain literal"
)]
struct CorrectnessCli {
    #[command(subcommand)]
    command: CorrectnessCommand,
}
#[derive(Subcommand)]
enum CorrectnessCommand {
    Check(CargoTail),
    Build(CargoTail),
    Test(CargoTail),
    Bench(CargoTail),
    Run(CargoTail),
    Clippy(CargoTail),
    Nextest {
        #[command(subcommand)]
        action: NextestCommand,
    },
}
#[derive(Args)]
struct CargoTail {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    arguments: Vec<OsString>,
}
#[derive(Subcommand)]
enum NextestCommand {
    Run(CargoTail),
    List(CargoTail),
}
pub(crate) fn validate_correctness(
    arguments: &[OsString],
) -> Result<bool, crate::correctness::CorrectnessError> {
    let mut argv = vec![OsString::from("cargo pse-correctness")];
    argv.extend_from_slice(arguments);
    match CorrectnessCli::try_parse_from(argv) {
        Ok(_) => Ok(true),
        Err(error) if !error.use_stderr() => {
            error.print()?;
            Ok(false)
        }
        Err(error) => Err(crate::correctness::CorrectnessError::Invalid(
            error.to_string(),
        )),
    }
}

fn runtime() -> Result<tokio::runtime::Runtime, HostError> {
    Ok(tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?)
}
/// Find an explicitly selected checkout, its immutable executable owner, or current directory.
pub fn checkout_root(selected: Option<&Path>) -> Result<PathBuf, HostError> {
    fn find(mut path: PathBuf) -> Option<PathBuf> {
        loop {
            if path.join(".config/diagnostic-tools.toml").is_file() {
                return Some(path);
            }
            if !path.pop() {
                return None;
            }
        }
    }
    if let Some(path) = selected {
        return find(path.to_path_buf())
            .ok_or_else(|| HostError::Ownership("-C must select a pse-arrow checkout".into()));
    }
    if let Some(executable) = std::env::current_exe().ok().filter(|path| {
        path.parent()
            .and_then(Path::parent)
            .and_then(Path::file_name)
            .is_some_and(|name| name == "host-tools")
    }) && let Some(root) = find(executable)
    {
        return Ok(root);
    }
    find(std::env::current_dir()?)
        .ok_or_else(|| HostError::Ownership("run inside a pse-arrow checkout".into()))
}
fn dispatch(cli: Cli) -> Result<i32, HostError> {
    let selected = cli.root.as_deref();
    let Some(command) = cli.command else {
        return Ok(0);
    };
    match command {
        Command::HostGenerationBootstrap => crate::generation::bootstrap(&checkout_root(selected)?),
        Command::Boundary(options) => crate::boundary::execute(options, selected),
        Command::AgentHook { action } => crate::agent_hooks::execute(action),
        Command::Doctor { json } => {
            runtime()?.block_on(crate::doctor::execute(&checkout_root(selected)?, json))
        }
        Command::CargoTools { names } => {
            for spec in
                crate::diagnostic_tools::cargo_tool_specs(&checkout_root(selected)?, &names)?
            {
                writeln!(io::stdout(), "{spec}")?;
            }
            Ok(0)
        }
        Command::DiagnosticTools { action } => {
            let (install, name) = match action {
                ToolAction::Install { tool } => (true, tool),
                ToolAction::Verify { tool } => (false, tool),
            };
            let tool = crate::diagnostic_tools::Tool::parse(&name)?;
            let values = std::env::vars_os().collect();
            let root = checkout_root(selected)?;
            let path = if install {
                crate::diagnostic_tools::install(&root, tool, &values)?
            } else {
                crate::diagnostic_tools::verify(&root, tool, &values)?
            };
            writeln!(io::stdout(), "{}", path.display())?;
            Ok(0)
        }
        Command::TelemetryService { action } => {
            let value = runtime()?.block_on(crate::telemetry_service::execute(
                &checkout_root(selected)?,
                &std::env::vars_os().collect(),
                action,
            ))?;
            serde_json::to_writer_pretty(io::stdout(), &value).map_err(io::Error::other)?;
            writeln!(io::stdout())?;
            Ok(0)
        }
        Command::Native {
            action: NativeAction::Prepare { capabilities },
        } => crate::native::execute(&checkout_root(selected)?, &capabilities),
        Command::Server { action } => crate::server::execute(&checkout_root(selected)?, action),
        Command::StudyModule { action } => {
            crate::study_module::execute(&checkout_root(selected)?, action)
        }
        Command::Env(options) => Ok(crate::environment::execute(options, selected)?),
        Command::Activity { unit } => runtime()?.block_on(async {
            let value = crate::activity::ActivityObservation::read(unit.as_deref()).await;
            serde_json::to_writer(io::stdout(), &value).map_err(io::Error::other)?;
            writeln!(io::stdout())?;
            Ok(if value.complete() { 0 } else { 125 })
        }),
        Command::Run { command } => runtime()?.block_on(crate::foreground::run(
            crate::foreground::ForegroundRequest {
                command,
                environment: std::env::vars_os().collect::<Environment>(),
                cwd: selected.map_or_else(std::env::current_dir, |path| Ok(path.to_path_buf()))?,
                require_enclosing: false,
                startup_deadline: Instant::now() + Duration::from_secs(30),
                drain_timeout: Duration::from_secs(30),
            },
        )),
    }
}

/// Run a native host entrypoint. The trampoline and raw Cargo protocol bypass parsing.
pub fn main(kind: BinKind) -> ExitCode {
    let arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
    let result = if matches!(kind, BinKind::Correctness) {
        return match crate::correctness::run(arguments) {
            Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(125)),
            Err(error) => {
                let _ = writeln!(io::stderr(), "arrow-validation: {error}");
                ExitCode::from(2)
            }
        };
    } else if arguments
        .first()
        .is_some_and(|arg| arg == "__foreground-exec")
    {
        crate::foreground::trampoline(&arguments[1..])
    } else {
        let mut argv = vec![OsString::from(if matches!(kind, BinKind::Pse) {
            "pse"
        } else {
            "pse-host"
        })];
        argv.extend(arguments.clone());
        match Cli::try_parse_from(argv) {
            Ok(cli) if cli.version => {
                let name = if matches!(kind, BinKind::Pse) {
                    "pse"
                } else {
                    "pse-host"
                };
                let _ = writeln!(
                    io::stdout(),
                    "{name} {} protocol 1",
                    env!("CARGO_PKG_VERSION")
                );
                Ok(0)
            }
            Ok(cli) if cli.command.is_none() => {
                use clap::CommandFactory as _;
                let _ = Cli::command().print_help();
                let _ = writeln!(io::stdout());
                Ok(0)
            }
            Ok(cli) => dispatch(cli),
            Err(error) => {
                if error.use_stderr() && arguments.first().is_some_and(|arg| arg == "boundary") {
                    let _ = writeln!(io::stderr(), "pse-env: {error}");
                    Ok(125)
                } else if error.use_stderr()
                    && arguments.first().is_some_and(|arg| arg == "agent-hook")
                {
                    let _ = writeln!(io::stderr(), "BLOCKED: {error}");
                    Ok(2)
                } else {
                    let code = error.exit_code();
                    let _ = error.print();
                    Ok(code)
                }
            }
        }
    };
    match result {
        Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(125)),
        Err(error) => {
            let prefix = if arguments.first().is_some_and(|arg| arg == "boundary") {
                "pse-env"
            } else {
                "pse-host"
            };
            let _ = writeln!(io::stderr(), "{prefix}: {error}");
            ExitCode::from(125)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory as _;
    #[test]
    fn command_tree_is_consistent() {
        Cli::command().debug_assert();
        CorrectnessCli::command().debug_assert();
    }
    #[test]
    fn boundary_tail_is_literal_and_native_is_last_wins() {
        let cli = Cli::try_parse_from([
            "pse-host",
            "boundary",
            "--native=compiler,klu",
            "--native",
            "tool",
            "--native=",
            "-h",
        ])
        .unwrap();
        let Some(Command::Boundary(options)) = cli.command else {
            panic!("boundary")
        };
        assert_eq!(options.native, Some(None));
        assert_eq!(
            options.command,
            vec![OsString::from("tool"), "--native=".into(), "-h".into()]
        );
    }
    #[test]
    fn run_preserves_child_options_and_env_rejects_conflicts() {
        let cli = Cli::try_parse_from([
            "pse",
            "run",
            "--",
            "/bin/true",
            "--help",
            "--format",
            "json",
        ])
        .unwrap();
        let Some(Command::Run { command }) = cli.command else {
            panic!("run")
        };
        assert_eq!(
            command,
            vec![
                OsString::from("/bin/true"),
                "--help".into(),
                "--format".into(),
                "json".into()
            ]
        );
        assert!(Cli::try_parse_from(["pse", "env", "--print", "--explain"]).is_err());
        assert!(
            Cli::try_parse_from([
                "pse",
                "env",
                "--explain",
                "--json",
                "--docs",
                "--cache",
                "off"
            ])
            .is_ok()
        );
        assert!(
            Cli::try_parse_from([
                "pse",
                "env",
                "--activation-data",
                "--root",
                "/checkout",
                "--cache",
                "auto"
            ])
            .is_ok()
        );
        let cli =
            Cli::try_parse_from(["pse-host", "run", "/bin/true", "--help", "--format", "json"])
                .unwrap();
        let Some(Command::Run { command }) = cli.command else {
            panic!("run")
        };
        assert_eq!(
            command,
            vec![
                OsString::from("/bin/true"),
                "--help".into(),
                "--format".into(),
                "json".into()
            ]
        );
    }
}
