// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Thin checkout command boundary composed from native environment and foreground owners.
//! The grammar is `cli::boundary`; this module owns the launch and its refusals.
use crate::{
    HostError, admission,
    environment::{self, CacheMode, Environment},
    foreground::{self, ForegroundRequest},
    native,
};
use std::{
    ffi::OsString,
    path::Path,
    time::{Duration, Instant},
};

/// One parsed boundary launch.
#[derive(Debug)]
pub struct Launch {
    /// Executable followed by its raw arguments, never interpreted by the boundary.
    pub command: Vec<OsString>,
    /// Select the documentation environment instead of the ordinary one.
    pub docs: bool,
    /// Explicit compiler-cache intent.
    pub cache: CacheMode,
    /// Requested native capability profile, if any.
    pub native: Option<native::Request>,
    /// Require authenticated enclosing containment instead of creating a scope.
    pub require_enclosing: bool,
    /// Explicit resource class.
    pub class: Option<String>,
    /// Recipe default resource class, below caller and explicit selections.
    pub recipe_class: Option<String>,
}

fn invalid(message: impl Into<String>) -> HostError {
    HostError::Ownership(message.into())
}

/// Refusal for a selection the boundary names but does not support.
pub fn unsupported(selection: &str) -> HostError {
    invalid(format!(
        "{selection} is unavailable during the Plan58 native cutover; no command or compatibility controller was started (58b/58f)"
    ))
}

/// Source-free explanation of a native profile; readiness is never observed here.
pub fn explain_native(capabilities: &str) -> serde_json::Value {
    let prepare = (!capabilities.is_empty()).then(|| {
        format!("scripts/pse-env -- scripts/pse-host native prepare --capabilities {capabilities}")
    });
    serde_json::json!({"native_profile":capabilities, "readiness":"not_observed", "personal_selections":"unresolved", "prepare":prepare})
}

fn known_selections(
    native: Option<&native::Request>,
    values: &Environment,
) -> Result<(), HostError> {
    if let Some(request) = native {
        request.resolve(values)?;
    }
    admission::validate_request(values)
}

/// Activate the checkout environment and run the command in an owned foreground scope.
/// Returns the child's status, 128+signal, or 127/126 for exec failures.
pub fn launch(root: &Path, caller: &Environment, launch: Launch) -> Result<i32, HostError> {
    known_selections(launch.native.as_ref(), caller)?;
    let mut overrides = Environment::new();
    if let Some(class) = &launch.class {
        overrides.insert("PSE_RESOURCE_CLASS".into(), class.into());
    }
    let mut selected =
        environment::activate_with_options(root, caller, &overrides, launch.cache, launch.docs)?
            .values;
    known_selections(launch.native.as_ref(), &selected)?;
    let selection = admission::Selection::load(
        root,
        &selected,
        launch.class.as_deref(),
        launch.recipe_class.as_deref(),
    )?;
    let coordinator = admission::Coordinator::open(root, &selected)?;
    if let Some(request) = &launch.native {
        let capabilities = request.resolve(&selected)?;
        selected = native::select_capabilities(root, &selected, &capabilities)?;
    }
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(foreground::run_with_admission(
            ForegroundRequest {
                command: launch.command,
                environment: selected,
                cwd: std::env::current_dir()?,
                startup_deadline: Instant::now() + Duration::from_secs(30),
                drain_timeout: Duration::from_secs(30),
                require_enclosing: launch.require_enclosing,
            },
            coordinator,
            selection,
        ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn known_selections_reject_explicit_empty_memory_before_execution() {
        let mut values = Environment::from([("PSE_BUILD_MODE".into(), "unbounded".into())]);
        assert!(known_selections(None, &values).is_ok());
        values.insert("PSE_MEMORY_MAX".into(), "".into());
        assert!(known_selections(None, &values).is_err());
    }
    #[test]
    fn caller_capabilities_apply_only_to_a_bare_native_request() {
        let caller = Environment::from([("PSE_NATIVE_CAPABILITIES".into(), "compiler,klu".into())]);
        assert_eq!(
            native::Request::Default
                .resolve(&Environment::new())
                .unwrap(),
            native::DEFAULT_CAPABILITIES
        );
        assert_eq!(
            native::Request::Default.resolve(&caller).unwrap(),
            "compiler,klu"
        );
        assert_eq!(
            native::Request::Explicit(String::new())
                .resolve(&caller)
                .unwrap(),
            ""
        );
        assert!(
            native::Request::Default
                .resolve(&Environment::from([(
                    "PSE_NATIVE_CAPABILITIES".into(),
                    "unknown".into()
                )]))
                .is_err()
        );
    }
}
