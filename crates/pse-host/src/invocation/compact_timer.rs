// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit local systemd user timer for bounded compact presentation custody.

use super::{directory, evidence_file, invalid, publish_bytes};
use crate::{
    HostError,
    environment::Environment,
    paths::{CheckoutState, HostPaths, UserDirectory},
};
use schemars::JsonSchema;
use serde::Serialize;
use std::{
    ffi::OsString,
    fs::{self, DirBuilder},
    io::Read,
    os::unix::{ffi::OsStringExt, fs::DirBuilderExt},
    path::{Path, PathBuf},
    process::{Command, Output},
};

const UNIT_LIMIT: usize = 16 * 1024;

/// Exact local timer preparation and activation result.
#[derive(Debug, Serialize, JsonSchema)]
pub struct CompactTimer {
    service: PathBuf,
    timer: PathBuf,
    activation_requested: bool,
}

fn word(value: &str) -> Result<String, HostError> {
    if value.chars().any(char::is_control) {
        return Err(invalid("compact timer input contains a control character"));
    }
    Ok(format!(
        "\"{}\"",
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('%', "%%")
    ))
}

fn quote_path(path: &Path) -> Result<String, HostError> {
    word(
        path.to_str()
            .ok_or_else(|| invalid("systemd unit path is not UTF-8"))?,
    )
}

fn unit_bytes(root: &Path, state_home: &Path, name: &str) -> Result<(String, String), HostError> {
    let program = quote_path(&root.join("scripts/pse"))?;
    let checkout = quote_path(root)?;
    let state = state_home
        .to_str()
        .ok_or_else(|| invalid("systemd state path is not UTF-8"))?;
    let environment = word(&format!("XDG_STATE_HOME={state}"))?;
    let service = format!(
        "[Unit]\nDescription=Release expired PSE compact developer results\n\n[Service]\nType=oneshot\nEnvironment={environment}\nExecStart=:{program} -C {checkout} tools sweep-results --registered --format json\n"
    );
    let timer = format!(
        "[Unit]\nDescription=Daily PSE compact developer result expiry\n\n[Timer]\nOnCalendar=daily\nPersistent=true\nUnit={name}.service\n\n[Install]\nWantedBy=timers.target\n"
    );
    Ok((service, timer))
}

fn unit_matches(path: &Path, content: &str) -> Result<bool, HostError> {
    match evidence_file(path) {
        Ok(file) => {
            let mut existing = Vec::new();
            file.take((UNIT_LIMIT + 1) as u64)
                .read_to_end(&mut existing)?;
            if existing != content.as_bytes() {
                return Err(invalid(
                    "compact timer unit differs from its declared owner",
                ));
            }
            Ok(true)
        }
        Err(HostError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn prepare_unit(path: &Path, content: &str) -> Result<(), HostError> {
    if unit_matches(path, content)? {
        return Ok(());
    }
    match publish_bytes(path, content.as_bytes(), Some(UNIT_LIMIT), 0o600) {
        Err(HostError::Io(error))
            if error.kind() == std::io::ErrorKind::AlreadyExists
                && unit_matches(path, content)? =>
        {
            Ok(())
        }
        result => result,
    }
}

fn systemctl(arguments: &[&str]) -> Result<Output, HostError> {
    let response = Command::new("/usr/bin/systemctl")
        .arg("--user")
        .args(arguments)
        .output()?;
    if !response.status.success() {
        return Err(invalid(format!(
            "compact timer systemctl {} failed with {}",
            arguments.first().copied().unwrap_or_default(),
            response.status
        )));
    }
    Ok(response)
}

fn check_fragment(name: &str, path: &Path) -> Result<(), HostError> {
    let response = systemctl(&["show", "--property=FragmentPath", "--value", name])?;
    let selected = std::str::from_utf8(&response.stdout)
        .map_err(|_| invalid("compact timer fragment path is not UTF-8"))?;
    if Path::new(selected.trim_end_matches('\n')) != path {
        return Err(invalid(
            "user manager selected a different compact timer unit owner",
        ));
    }
    let overrides = systemctl(&["show", "--property=DropInPaths", "--value", name])?;
    if !overrides.stdout.iter().all(u8::is_ascii_whitespace) {
        return Err(invalid("compact timer refuses user manager unit overrides"));
    }
    Ok(())
}

/// Prepare the single timer for the selected repository. Only an explicit enable
/// request activates it. Preparation never runs a sweep directly.
pub fn prepare_compact_timer(
    root: &Path,
    environment: &Environment,
    enable: bool,
) -> Result<CompactTimer, HostError> {
    let selected = CheckoutState::resolve(root, environment)?;
    let worktrees = crate::repository::observe_worktrees(root)?;
    let main = worktrees
        .entries
        .into_iter()
        .find(|entry| entry.main && !entry.bare)
        .and_then(|entry| entry.worktree)
        .ok_or_else(|| invalid("main checkout for the compact timer is unavailable"))?;
    let main = fs::canonicalize(PathBuf::from(OsString::from_vec(main.as_bytes().to_vec())))?;
    if CheckoutState::resolve(&main, environment)?.repository_id != selected.repository_id {
        return Err(invalid(
            "main checkout differs from the selected compact timer repository",
        ));
    }
    let paths = HostPaths::new(environment);
    let units = paths.base(UserDirectory::Config)?.join("systemd/user");
    if !units.is_absolute() {
        return Err(invalid("compact timer unit directory must be absolute"));
    }
    let state_home = paths.base(UserDirectory::State)?;
    let name = format!("pse-compact-results-{}", selected.repository_id.as_str());
    let (service, timer) = unit_bytes(&main, &state_home, &name)?;
    DirBuilder::new()
        .mode(0o700)
        .recursive(true)
        .create(&units)?;
    directory(&units, false)?;
    let service_name = format!("{name}.service");
    let timer_name = format!("{name}.timer");
    let service_path = units.join(&service_name);
    let timer_path = units.join(&timer_name);
    prepare_unit(&service_path, &service)?;
    prepare_unit(&timer_path, &timer)?;
    if enable {
        systemctl(&["daemon-reload"])?;
        check_fragment(&service_name, &service_path)?;
        check_fragment(&timer_name, &timer_path)?;
        if !unit_matches(&service_path, &service)? || !unit_matches(&timer_path, &timer)? {
            return Err(invalid("compact timer unit disappeared before enable"));
        }
        systemctl(&["enable", "--now", &timer_name])?;
    }
    Ok(CompactTimer {
        service: service_path,
        timer: timer_path,
        activation_requested: enable,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_has_fixed_scope_and_literal_paths() {
        let (service, timer) = unit_bytes(
            Path::new("/tmp/with \"quotes\" %n $HOME"),
            Path::new("/tmp/state"),
            "pse-test",
        )
        .unwrap();
        assert!(service.contains("ExecStart=:\"/tmp/with \\\"quotes\\\" %%n $HOME/scripts/pse\""));
        assert!(service.contains("tools sweep-results --registered --format json"));
        assert!(timer.contains("OnCalendar=daily\nPersistent=true\nUnit=pse-test.service"));
        assert!(quote_path(Path::new("/tmp/not\na unit")).is_err());
    }

    #[test]
    fn prepare_preserves_existing_foreign_units() {
        let root = tempfile::tempdir().unwrap();
        let unit = root.path().join("pse-test.timer");
        prepare_unit(&unit, "first").unwrap();
        prepare_unit(&unit, "first").unwrap();
        assert!(prepare_unit(&unit, "different").is_err());
        assert_eq!(fs::read(&unit).unwrap(), b"first");
    }
}
