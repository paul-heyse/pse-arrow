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
    no_scope: bool,
    cache: CacheMode,
    class: Option<OsString>,
    recipe_class: Option<OsString>,
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
                if !matches!(
                    value.to_str(),
                    Some(
                        "light"
                            | "build"
                            | "compile"
                            | "functional"
                            | "wide"
                            | "exclusive"
                            | "timing"
                            | "exclusive-observer"
                    )
                ) {
                    return Err(invalid("unknown resource class"));
                }
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
            Some("--native" | "--store" | "--bind-allocation") => {
                return Err(unsupported(argument.to_str().unwrap_or("selection")));
            }
            Some(value) if value.starts_with("--native=") => {
                return Err(unsupported("native solver/provider selection"));
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
    if values
        .get(OsStr::new("PSE_BUILD_MODE"))
        .is_some_and(|value| value != "unbounded")
    {
        return Err(unsupported(
            "bounded host admission; select PSE_BUILD_MODE=unbounded for ordinary work",
        ));
    }
    for key in ["PSE_MEMORY_MAX", "PSE_BUILD_MEASUREMENT_TIMING"] {
        if values.contains_key(OsStr::new(key)) {
            return Err(unsupported(key));
        }
    }
    // Explicit resource classes keep their meaning; recipe hints may select ordinary
    // unbounded work, as before. No numeric resource policy is reconstructed here.
    if options
        .class
        .as_ref()
        .or_else(|| values.get(OsStr::new("PSE_RESOURCE_CLASS")))
        .is_some_and(|value| value != "build")
    {
        return Err(unsupported("explicit finite/scientific resource class"));
    }
    if options
        .recipe_class
        .as_ref()
        .is_some_and(|value| matches!(value.to_str(), Some("timing" | "exclusive-observer")))
    {
        return Err(unsupported("reference/timing recipe"));
    }
    if values
        .get(OsStr::new("PSE_SLICE"))
        .is_some_and(|value| !value.is_empty() && value != "none")
    {
        return Err(unsupported("custom slice placement"));
    }
    Ok(())
}
fn ordinary(options: &Options, values: &mut Environment) -> Result<(), HostError> {
    known_selections(options, values)?;
    if !values.contains_key(OsStr::new("PSE_BUILD_MODE")) {
        return Err(unsupported(
            "bounded host admission; select PSE_BUILD_MODE=unbounded for ordinary work",
        ));
    }
    values.remove(OsStr::new("PSE_RESOURCE_CLASS"));
    Ok(())
}

/// Execute the supported ordinary command or fixed, source-free environment read.
pub fn run_cli(arguments: &[OsString]) -> Result<i32, HostError> {
    let options = parse(arguments)?;
    if arguments.is_empty() || options.help {
        writeln!(
            std::io::stdout(),
            "pse-env [--docs] [--cache auto|on|off] -- command [args...]\npse-env --print | --explain [--json]\nNative foreground ownership; explicit preparation: scripts/bootstrap-host.\nFinite/scientific admission and database/provider setup await their Plan58 owners."
        )?;
        return Ok(0);
    }
    if options.read {
        if options.no_scope || options.class.is_some() || options.recipe_class.is_some() {
            return Err(invalid(
                "resource placement options do not apply to environment reads",
            ));
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
    ordinary(&options, &mut selected)?;
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(foreground::run(ForegroundRequest {
            command: options.command,
            environment: selected,
            cwd: std::env::current_dir()?,
            startup_deadline: Instant::now() + Duration::from_secs(30),
            drain_timeout: Duration::from_secs(30),
            require_enclosing: options.no_scope,
        }))
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
    fn boundary_refuses_unimplemented_modes_before_execution() {
        for mode in ["--native", "--native=", "--store", "--bind-allocation"] {
            assert!(parse(&args(&[mode, "--", "/bin/true"])).is_err());
        }
        let mut values = Environment::from([("PSE_BUILD_MODE".into(), "unbounded".into())]);
        assert!(
            ordinary(
                &parse(&args(&["--recipe-default-class", "compile", "--", "tool"])).unwrap(),
                &mut values
            )
            .is_ok()
        );
        assert!(
            ordinary(
                &parse(&args(&["--resource-class", "compile", "--", "tool"])).unwrap(),
                &mut values
            )
            .is_err()
        );
        values.insert("PSE_MEMORY_MAX".into(), "".into());
        assert!(ordinary(&Options::default(), &mut values).is_err());
    }
}
