// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Thin checkout command boundary composed from native environment and foreground owners.
use crate::{
    HostError,
    environment::{self, CacheMode, Environment},
    foreground::{self, ForegroundRequest},
};
use std::{
    ffi::{OsStr, OsString},
    io::Write as _,
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Default)]
struct Options {
    environment: Vec<OsString>,
    command: Vec<OsString>,
    docs: bool,
    read: bool,
    help: bool,
    native: Option<String>,
    native_default: bool,
    no_scope: bool,
    cache: CacheMode,
    class: Option<OsString>,
    recipe_class: Option<OsString>,
}
const DEFAULT_NATIVE_CAPABILITIES: &str = "solver,klu,isolation,uno,petsc";
fn native_request(options: &Options, values: &Environment) -> Result<Option<String>, HostError> {
    let requested = if options.native_default {
        Some(values.get(OsStr::new("PSE_NATIVE_CAPABILITIES"))
            .map(|value| value.to_str().ok_or_else(|| invalid("PSE_NATIVE_CAPABILITIES must be UTF-8")))
            .transpose()?.unwrap_or(DEFAULT_NATIVE_CAPABILITIES).to_owned())
    } else { options.native.clone() };
    if let Some(value) = &requested { crate::native::validate_capabilities(value)?; }
    Ok(requested)
}
fn invalid(message: impl Into<String>) -> HostError {
    HostError::Ownership(message.into())
}
fn unsupported(selection: &str) -> HostError {
    invalid(format!(
        "{selection} is unavailable during the Plan58 native cutover; no command or compatibility controller was started (58b/58f)"
    ))
}
fn parse(arguments: &[OsString]) -> Result<Options, HostError> {
    let mut result = Options::default();
    let mut args = arguments.iter();
    while let Some(argument) = args.next() {
        match argument.to_str() {
            Some("--") => {
                result.command.extend(args.cloned());
                break;
            }
            Some("--help" | "-h") => {
                result.help = true;
                break;
            }
            Some("--print" | "--explain") => {
                result.read = true;
                result.environment.push(argument.clone());
            }
            Some("--json") => result.environment.push(argument.clone()),
            Some("--docs") => {
                result.docs = true;
                result.environment.push(argument.clone());
            }
            Some("--no-scope") => result.no_scope = true,
            Some("--resource-class" | "--recipe-default-class") => {
                let value = args
                    .next()
                    .ok_or_else(|| invalid("resource class requires a value"))?
                    .clone();
                crate::admission::validate_class(value.to_str().ok_or_else(||invalid("resource class must be UTF-8"))?)?;
                if argument == "--resource-class" {
                    result.class = Some(value);
                } else {
                    result.recipe_class = Some(value);
                }
            }
            Some("--cache") => {
                let value = args
                    .next()
                    .ok_or_else(|| invalid("--cache requires auto, on or off"))?;
                result.cache = match value.to_str() {
                    Some("auto") => CacheMode::Auto,
                    Some("on") => CacheMode::On,
                    Some("off") => CacheMode::Off,
                    _ => return Err(invalid("--cache requires auto, on or off")),
                };
                result.environment.extend([argument.clone(), value.clone()]);
            }
            Some("--native") => { result.native = Some(DEFAULT_NATIVE_CAPABILITIES.into()); result.native_default = true; },
            Some("--store" | "--bind-allocation") => {
                return Err(unsupported(argument.to_str().unwrap_or("selection")));
            }
            Some(value) if value.starts_with("--native=") => {
                let capabilities = value.trim_start_matches("--native=");
                crate::native::validate_capabilities(capabilities)?;
                result.native = Some(capabilities.to_owned());
                result.native_default = false;
            }
            Some(value) if value.starts_with('-') => {
                return Err(invalid("unknown boundary option; use --help"));
            }
            _ => {
                result.command.push(argument.clone());
                result.command.extend(args.cloned());
                break;
            }
        }
    }
    Ok(result)
}
fn root() -> Result<PathBuf, HostError> {
    let mut root = std::env::current_dir()?;
    while !root.join(".config/diagnostic-tools.toml").is_file() {
        if !root.pop() {
            return Err(invalid("run inside a pse-arrow checkout"));
        }
    }
    Ok(root)
}
fn known_selections(options: &Options, values: &Environment) -> Result<(), HostError> {
    native_request(options, values)?;
    crate::admission::validate_request(values)
}

/// Execute the supported ordinary command or fixed, source-free environment read.
pub fn run_cli(arguments: &[OsString]) -> Result<i32, HostError> {
    let options = parse(arguments)?;
    if options.native.is_some() && options.docs {
        return Err(invalid("--docs and --native select different environments"));
    }
    if arguments.is_empty() || options.help {
        writeln!(
            std::io::stdout(),
            "pse-env [--docs] [--cache auto|on|off] [--native[=CAPABILITIES]] [--resource-class CLASS] -- command [args...]\npse-env --print | --explain [--json]\nNative foreground ownership; explicit preparation: scripts/bootstrap-host.\nNative providers require explicit preparation; store setup remains unavailable."
        )?;
        return Ok(0);
    }
    if options.read {
        if options.no_scope || options.class.is_some() || options.recipe_class.is_some() {
            return Err(invalid(
                "resource placement options do not apply to environment reads",
            ));
        }
        if let Some(capabilities) = native_request(&options, &std::env::vars_os().collect())? {
            if !options.environment.iter().any(|value| value == "--explain")
                || options.environment.iter().any(|value| value == "--print")
            {
                return Err(invalid(
                    "native profile reads require --explain; native shell exports are not supported",
                ));
            }
            let prepare = (!capabilities.is_empty()).then(|| format!("scripts/pse-env -- scripts/pse-host native prepare --capabilities {capabilities}"));
            let explanation = serde_json::json!({"native_profile":capabilities, "readiness":"not_observed", "personal_selections":"unresolved", "prepare":prepare});
            writeln!(std::io::stdout(), "{explanation}")?;
            return Ok(0);
        }
        return Ok(environment::run_cli(&options.environment)?);
    }
    if options.environment.iter().any(|value| value == "--json") {
        return Err(invalid("--json requires --explain"));
    }
    if options.command.is_empty() {
        return Err(invalid("give a command after --, or use --print/--explain"));
    }
    let caller = std::env::vars_os().collect();
    known_selections(&options, &caller)?;
    let mut overrides = Environment::new();
    if let Some(class) = &options.class {
        overrides.insert("PSE_RESOURCE_CLASS".into(), class.clone());
    }
    let mut selected = environment::activate_with_options(
        &root()?,
        &caller,
        &overrides,
        options.cache,
        options.docs,
    )?
    .values;
    known_selections(&options, &selected)?;
    let root = root()?;
    let selection = crate::admission::Selection::load(&root, &selected,
        options.class.as_ref().map(|value| value.to_str().ok_or_else(||invalid("resource class must be UTF-8"))).transpose()?,
        options.recipe_class.as_ref().map(|value| value.to_str().ok_or_else(||invalid("recipe class must be UTF-8"))).transpose()?)?;
    let coordinator = crate::admission::Coordinator::open(&root, &selected)?;
    if let Some(capabilities) = native_request(&options, &selected)? {
        selected = crate::native::select_capabilities(&root, &selected, &capabilities)?;
    }
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(foreground::run_with_admission(ForegroundRequest {
            command: options.command,
            environment: selected,
            cwd: std::env::current_dir()?,
            startup_deadline: Instant::now() + Duration::from_secs(30),
            drain_timeout: Duration::from_secs(30),
            require_enclosing: options.no_scope,
        }, coordinator, selection))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }
    #[test]
    fn boundary_keeps_command_options_as_data() {
        let options = parse(&args(&["--", "tool", "--native", "--store"])).unwrap();
        assert_eq!(options.command, args(&["tool", "--native", "--store"]));
    }
    #[test]
    fn native_default_preserves_full_profile_caller_override_and_explicit_empty() {
        let defaults = parse(&args(&["--native", "--", "tool"])).unwrap();
        assert_eq!(native_request(&defaults, &Environment::new()).unwrap().as_deref(), Some(DEFAULT_NATIVE_CAPABILITIES));
        let caller = Environment::from([("PSE_NATIVE_CAPABILITIES".into(), "compiler,klu".into())]);
        assert_eq!(native_request(&defaults, &caller).unwrap().as_deref(), Some("compiler,klu"));
        let empty = parse(&args(&["--native=", "--", "tool"])).unwrap();
        assert_eq!(native_request(&empty, &caller).unwrap().as_deref(), Some(""));
        assert!(native_request(&defaults, &Environment::from([("PSE_NATIVE_CAPABILITIES".into(), "unknown".into())])).is_err());
        assert_eq!(native_request(&parse(&args(&["--native", "--native=compiler,klu"])).unwrap(), &Environment::new()).unwrap().as_deref(), Some("compiler,klu"));
        assert_eq!(native_request(&parse(&args(&["--native=compiler,klu", "--native"])).unwrap(), &Environment::new()).unwrap().as_deref(), Some(DEFAULT_NATIVE_CAPABILITIES));
    }
    #[test]
    fn boundary_refuses_unimplemented_modes_before_execution() {
        for mode in ["--native=unknown", "--store", "--bind-allocation"] {
            assert!(parse(&args(&[mode, "--", "/bin/true"])).is_err());
        }
        let mut values = Environment::from([("PSE_BUILD_MODE".into(), "unbounded".into())]);
        assert!(known_selections(&Options::default(), &values).is_ok());
        values.insert("PSE_MEMORY_MAX".into(), "".into());
        assert!(known_selections(&Options::default(), &values).is_err());
    }
}
