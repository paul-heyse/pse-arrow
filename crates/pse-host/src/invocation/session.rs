// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Session correlation selects metadata only; native ownership remains independently checked.
use crate::{HostError, environment::Environment};
use std::ffi::OsStr;

fn checked(value: String, source: &str) -> Result<String, HostError> {
    if value.is_empty() || value.chars().any(char::is_control) {
        return Err(HostError::Ownership(format!(
            "session identity from {source} must be nonempty text without control characters"
        )));
    }
    Ok(value)
}

/// Select explicit CLI identity, PSE_SESSION, an existing Codex thread, or an existing
/// Claude session, in that order. Empty environment values are unavailable. None means
/// checkout-wide correlation; no identity is synthesized and no hook or process is created.
pub fn select_session(
    explicit: Option<String>,
    environment: &Environment,
) -> Result<Option<String>, HostError> {
    if let Some(value) = explicit {
        return checked(value, "--session").map(Some);
    }
    for name in ["PSE_SESSION", "CODEX_THREAD_ID", "CLAUDE_SESSION_ID"] {
        let Some(value) = environment
            .get(OsStr::new(name))
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        let value = value.to_str().ok_or_else(|| {
            HostError::Ownership(format!("session identity from {name} is not UTF-8"))
        })?;
        return checked(value.to_owned(), name).map(Some);
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{ffi::OsString, os::unix::ffi::OsStringExt as _};
    #[test]
    fn session_precedence_preserves_explicit_override_and_existing_harness_identity() {
        let mut environment = Environment::from([
            ("PSE_SESSION".into(), "selected".into()),
            ("CODEX_THREAD_ID".into(), "codex".into()),
            ("CLAUDE_SESSION_ID".into(), "claude".into()),
        ]);
        assert_eq!(
            select_session(Some("explicit".into()), &environment)
                .unwrap()
                .as_deref(),
            Some("explicit")
        );
        assert_eq!(
            select_session(None, &environment).unwrap().as_deref(),
            Some("selected")
        );
        environment.remove(OsStr::new("PSE_SESSION"));
        assert_eq!(
            select_session(None, &environment).unwrap().as_deref(),
            Some("codex")
        );
        environment.insert("CODEX_THREAD_ID".into(), "".into());
        assert_eq!(
            select_session(None, &environment).unwrap().as_deref(),
            Some("claude")
        );
        assert_eq!(select_session(None, &Environment::new()).unwrap(), None);
        assert_eq!(
            select_session(Some("../opaque-correlation".into()), &Environment::new())
                .unwrap()
                .as_deref(),
            Some("../opaque-correlation")
        );
    }
    #[test]
    fn invalid_selected_session_never_silently_rebinds_to_another_harness() {
        let mut environment = Environment::from([
            ("PSE_SESSION".into(), OsString::from_vec(vec![255])),
            ("CODEX_THREAD_ID".into(), "healthy".into()),
        ]);
        assert!(select_session(None, &environment).is_err());
        assert_eq!(
            select_session(Some("explicit".into()), &environment)
                .unwrap()
                .as_deref(),
            Some("explicit")
        );
        environment.insert("PSE_SESSION".into(), "line\nbreak".into());
        assert!(select_session(None, &environment).is_err());
        assert!(select_session(Some(String::new()), &environment).is_err());
    }
}
