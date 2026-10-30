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
    time::{Duration, Instant},
};

#[derive(Default)]
struct Options {
    environment: Vec<OsString>,
    command: Vec<OsString>,
    docs: bool,
    read: bool,
    native: Option<String>,
    native_default: bool,
    no_scope: bool,
    cache: CacheMode,
    class: Option<OsString>,
    recipe_class: Option<OsString>,
}
const DEFAULT_NATIVE_CAPABILITIES: &str = "solver,klu,isolation,uno,petsc";
/// Passive native profile selection, never readiness or an implicit build.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct NativeExplanation {
    /// Explicit capability selection.
    pub native_profile: String,
    /// No readiness probe was performed.
    pub readiness: String,
    /// Personal shell selections were not sourced.
    pub personal_selections: String,
    /// Optional literal preparation guidance.
    pub prepare: Option<String>,
}
fn native_request(options: &Options, values: &Environment) -> Result<Option<String>, HostError> {
    let requested = if options.native_default {
        Some(
            values
                .get(OsStr::new("PSE_NATIVE_CAPABILITIES"))
                .map(|value| {
                    value
                        .to_str()
                        .ok_or_else(|| invalid("PSE_NATIVE_CAPABILITIES must be UTF-8"))
                })
                .transpose()?
                .unwrap_or(DEFAULT_NATIVE_CAPABILITIES)
                .to_owned(),
        )
    } else {
        options.native.clone()
    };
    if let Some(value) = &requested {
        crate::native::validate_capabilities(value)?;
    }
    Ok(requested)
}
fn invalid(message: impl Into<String>) -> HostError {
    HostError::Ownership(message.into())
}
#[cfg(test)]
fn parse(arguments: &[OsString]) -> Result<Options, HostError> {
    use clap::Parser as _;
    #[derive(clap::Parser)]
    struct Boundary {
        #[command(flatten)]
        options: crate::cli::BoundaryArgs,
    }
    let args = Boundary::try_parse_from(
        std::iter::once(OsString::from("pse-env")).chain(arguments.iter().cloned()),
    )
    .map_err(|error| invalid(error.to_string()))?;
    options(args.options)
}
fn options(args: crate::cli::BoundaryArgs) -> Result<Options, HostError> {
    let mut result = Options::default();
    result.docs = args.docs;
    result.read = args.print || args.explain;
    result.no_scope = args.no_scope;
    result.class = args.resource_class.map(OsString::from);
    result.recipe_class = args.recipe_default_class.map(OsString::from);
    result.cache = args.cache.into();
    result.command = args.command;
    result.native_default = args.native.as_ref().is_some_and(Option::is_none);
    result.native = args
        .native
        .map(|value| value.unwrap_or_else(|| DEFAULT_NATIVE_CAPABILITIES.into()));
    if let Some(native) = &result.native {
        crate::native::validate_capabilities(native)?;
    }
    for (enabled, name) in [
        (args.print, "--print"),
        (args.explain, "--explain"),
        (args.json, "--json"),
        (args.docs, "--docs"),
    ] {
        if enabled {
            result.environment.push(name.into());
        }
    }
    result.environment.extend([
        OsString::from("--cache"),
        OsString::from(match args.cache {
            crate::cli::CacheArg::Auto => "auto",
            crate::cli::CacheArg::On => "on",
            crate::cli::CacheArg::Off => "off",
        }),
    ]);
    Ok(result)
}
fn args_cache(cache: CacheMode) -> crate::cli::CacheArg {
    match cache {
        CacheMode::Auto => crate::cli::CacheArg::Auto,
        CacheMode::On => crate::cli::CacheArg::On,
        CacheMode::Off => crate::cli::CacheArg::Off,
    }
}
fn known_selections(options: &Options, values: &Environment) -> Result<(), HostError> {
    native_request(options, values)?;
    crate::admission::validate_request(values)
}

/// Execute the supported ordinary command or fixed, source-free environment read.
pub(crate) fn execute(
    args: crate::cli::BoundaryArgs,
    selected_root: Option<&std::path::Path>,
    presentation: &crate::envelope::OutputOptions,
) -> Result<i32, HostError> {
    let options = options(args)?;
    if options.native.is_some() && options.docs {
        return Err(invalid("--docs and --native select different environments"));
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
            let explanation = NativeExplanation {
                native_profile: capabilities,
                readiness: "not_observed".into(),
                personal_selections: "unresolved".into(),
                prepare,
            };
            let mut presentation = presentation.clone();
            if options.environment.iter().any(|value| value == "--json") {
                presentation.format = crate::envelope::OutputFormat::Json;
            }
            return Ok(presentation.emit(crate::envelope::Envelope::response(
                "boundary explain native",
                "native selection; readiness not observed",
                explanation,
            ))?);
        }
        return Ok(environment::execute(
            crate::cli::EnvironmentArgs {
                print: options.environment.iter().any(|value| value == "--print"),
                explain: options.environment.iter().any(|value| value == "--explain"),
                json: options.environment.iter().any(|value| value == "--json"),
                activation_data: false,
                docs: options.docs,
                root: None,
                cache: args_cache(options.cache),
            },
            selected_root,
            presentation,
        )?);
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
        &crate::cli::checkout_root(selected_root)?,
        &caller,
        &overrides,
        options.cache,
        options.docs,
    )?
    .values;
    known_selections(&options, &selected)?;
    let root = crate::cli::checkout_root(selected_root)?;
    let selection = crate::admission::Selection::load(
        &root,
        &selected,
        options
            .class
            .as_ref()
            .map(|value| {
                value
                    .to_str()
                    .ok_or_else(|| invalid("resource class must be UTF-8"))
            })
            .transpose()?,
        options
            .recipe_class
            .as_ref()
            .map(|value| {
                value
                    .to_str()
                    .ok_or_else(|| invalid("recipe class must be UTF-8"))
            })
            .transpose()?,
    )?;
    let coordinator = crate::admission::Coordinator::open(&root, &selected)?;
    if let Some(capabilities) = native_request(&options, &selected)? {
        selected = crate::native::select_capabilities(&root, &selected, &capabilities)?;
    }
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(foreground::run_with_admission(
            ForegroundRequest {
                command: options.command,
                environment: selected,
                cwd: std::env::current_dir()?,
                startup_deadline: Instant::now() + Duration::from_secs(30),
                drain_timeout: Duration::from_secs(30),
                require_enclosing: options.no_scope,
            },
            coordinator,
            selection,
        ))
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
        assert_eq!(
            native_request(&defaults, &Environment::new())
                .unwrap()
                .as_deref(),
            Some(DEFAULT_NATIVE_CAPABILITIES)
        );
        let caller = Environment::from([("PSE_NATIVE_CAPABILITIES".into(), "compiler,klu".into())]);
        assert_eq!(
            native_request(&defaults, &caller).unwrap().as_deref(),
            Some("compiler,klu")
        );
        let empty = parse(&args(&["--native=", "--", "tool"])).unwrap();
        assert_eq!(
            native_request(&empty, &caller).unwrap().as_deref(),
            Some("")
        );
        assert!(
            native_request(
                &defaults,
                &Environment::from([("PSE_NATIVE_CAPABILITIES".into(), "unknown".into())])
            )
            .is_err()
        );
        assert_eq!(
            native_request(
                &parse(&args(&["--native", "--native=compiler,klu"])).unwrap(),
                &Environment::new()
            )
            .unwrap()
            .as_deref(),
            Some("compiler,klu")
        );
        assert_eq!(
            native_request(
                &parse(&args(&["--native=compiler,klu", "--native"])).unwrap(),
                &Environment::new()
            )
            .unwrap()
            .as_deref(),
            Some(DEFAULT_NATIVE_CAPABILITIES)
        );
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
