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
    collections::BTreeMap,
    ffi::OsString,
    fs::{self, DirBuilder},
    io::{ErrorKind, Read},
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

/// Passive observation of the selected repository's expiry timer.
#[derive(Debug, Serialize, JsonSchema)]
pub struct CompactTimerStatus {
    service: PathBuf,
    timer: PathBuf,
    state: ExpiryState,
    detail: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum ExpiryState {
    Unprepared,
    Inactive,
    Active,
    ManagerUnavailable,
    Unverified,
}

struct Unit {
    name: String,
    path: PathBuf,
    content: String,
}

impl Unit {
    fn new(directory: &Path, name: String, content: String) -> Self {
        Self {
            path: directory.join(&name),
            name,
            content,
        }
    }

    fn matches(&self) -> Result<bool, HostError> {
        unit_matches(&self.path, &self.content)
    }
}

struct Selection {
    directory: PathBuf,
    service: Unit,
    timer: Unit,
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
        Err(HostError::Io(error)) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn prepare_unit(path: &Path, content: &str) -> Result<(), HostError> {
    if unit_matches(path, content)? {
        return Ok(());
    }
    match publish_bytes(path, content.as_bytes(), Some(UNIT_LIMIT), 0o600) {
        Err(HostError::Io(error))
            if error.kind() == ErrorKind::AlreadyExists && unit_matches(path, content)? =>
        {
            Ok(())
        }
        result => result,
    }
}

fn systemctl_command() -> Command {
    let mut command = Command::new("/usr/bin/systemctl");
    command.arg("--user");
    command
}

fn systemctl(arguments: &[&str]) -> Result<Output, HostError> {
    let response = systemctl_command().args(arguments).output()?;
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

fn select(root: &Path, environment: &Environment) -> Result<Selection, HostError> {
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
    let directory = paths.base(UserDirectory::Config)?.join("systemd/user");
    if !directory.is_absolute() {
        return Err(invalid("compact timer unit directory must be absolute"));
    }
    let state_home = paths.base(UserDirectory::State)?;
    let name = format!("pse-compact-results-{}", selected.repository_id.as_str());
    let (service, timer) = unit_bytes(&main, &state_home, &name)?;
    Ok(Selection {
        service: Unit::new(&directory, format!("{name}.service"), service),
        timer: Unit::new(&directory, format!("{name}.timer"), timer),
        directory,
    })
}

/// Prepare the single timer for the selected repository. Only an explicit enable
/// request activates it. Preparation never runs a sweep directly.
pub fn prepare_compact_timer(
    root: &Path,
    environment: &Environment,
    enable: bool,
) -> Result<CompactTimer, HostError> {
    let Selection {
        directory: units,
        service,
        timer,
    } = select(root, environment)?;
    DirBuilder::new()
        .mode(0o700)
        .recursive(true)
        .create(&units)?;
    directory(&units, false)?;
    prepare_unit(&service.path, &service.content)?;
    prepare_unit(&timer.path, &timer.content)?;
    if enable {
        systemctl(&["daemon-reload"])?;
        check_fragment(&service.name, &service.path)?;
        check_fragment(&timer.name, &timer.path)?;
        if !service.matches()? || !timer.matches()? {
            return Err(invalid("compact timer unit disappeared before enable"));
        }
        systemctl(&["enable", "--now", &timer.name])?;
    }
    Ok(CompactTimer {
        service: service.path,
        timer: timer.path,
        activation_requested: enable,
    })
}

#[derive(Debug, PartialEq, Eq)]
enum ManagerUnit<'a> {
    NotFound,
    Exact(&'a str),
}

fn manager_unit<'a>(unit: &Unit, output: &'a [u8]) -> Option<ManagerUnit<'a>> {
    let mut properties = BTreeMap::new();
    for line in std::str::from_utf8(output).ok()?.lines() {
        let (name, value) = line.split_once('=')?;
        if properties.insert(name, value).is_some() {
            return None;
        }
    }
    let value = |name| properties.get(name).copied();
    if value("Id")? != unit.name
        || !value("DropInPaths")?.is_empty()
        || value("NeedDaemonReload")? != "no"
    {
        return None;
    }
    let fragment = value("FragmentPath")?;
    let active = value("ActiveState")?;
    match value("LoadState")? {
        "not-found" if fragment.is_empty() && active == "inactive" => Some(ManagerUnit::NotFound),
        "loaded" if Path::new(fragment) == unit.path.as_path() => Some(ManagerUnit::Exact(active)),
        _ => None,
    }
}

fn observe_selection(
    selection: &Selection,
    mut show: impl FnMut(&str) -> Option<Vec<u8>>,
) -> (ExpiryState, &'static str) {
    const UNPREPARED: (ExpiryState, &str) = (
        ExpiryState::Unprepared,
        "No expiry units prepared in the selected configuration",
    );
    match directory(&selection.directory, false) {
        Ok(()) => {}
        Err(HostError::Io(error)) if error.kind() == ErrorKind::NotFound => return UNPREPARED,
        Err(_) => {
            return (
                ExpiryState::Unverified,
                "Cannot verify the selected user unit directory",
            );
        }
    }
    match (selection.service.matches(), selection.timer.matches()) {
        (Ok(false), Ok(false)) => return UNPREPARED,
        (Ok(true), Ok(true)) => {}
        _ => {
            return (
                ExpiryState::Unverified,
                "Cannot verify both prepared expiry unit files",
            );
        }
    }

    let Some(timer) = show(&selection.timer.name) else {
        return (
            ExpiryState::ManagerUnavailable,
            "Cannot read expiry timer from the user manager",
        );
    };
    let timer_state = match manager_unit(&selection.timer, &timer) {
        Some(ManagerUnit::NotFound) => {
            return (
                ExpiryState::Inactive,
                "The user manager has not loaded the prepared timer",
            );
        }
        Some(ManagerUnit::Exact(state)) => state,
        None => {
            return (
                ExpiryState::Unverified,
                "Cannot verify the user manager's timer owner",
            );
        }
    };
    let Some(service) = show(&selection.service.name) else {
        return (
            ExpiryState::ManagerUnavailable,
            "Cannot read expiry service from the user manager",
        );
    };
    if !matches!(
        manager_unit(&selection.service, &service),
        Some(ManagerUnit::Exact(_))
    ) || !selection.service.matches().unwrap_or(false)
        || !selection.timer.matches().unwrap_or(false)
    {
        return (
            ExpiryState::Unverified,
            "Cannot verify the user manager's current expiry units",
        );
    }

    match timer_state {
        "active" => (
            ExpiryState::Active,
            "The exact prepared expiry timer is active",
        ),
        "inactive" => (
            ExpiryState::Inactive,
            "The exact prepared expiry timer is inactive",
        ),
        "failed" => (
            ExpiryState::Inactive,
            "The exact prepared expiry timer is in a failed state",
        ),
        _ => (
            ExpiryState::Unverified,
            "The exact timer has no confirmed active or inactive state",
        ),
    }
}

/// Read prepared files and the selected user manager without activating cleanup.
pub fn compact_timer_status(
    root: &Path,
    environment: &Environment,
) -> Result<CompactTimerStatus, HostError> {
    let selection = select(root, environment)?;
    let (state, detail) = observe_selection(&selection, |name| {
        systemctl_command()
            .env_clear()
            .envs(environment)
            .args([
                "show",
                "--all",
                "--property=Id,LoadState,FragmentPath,DropInPaths,NeedDaemonReload,ActiveState",
                "--",
                name,
            ])
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|output| output.stdout)
    });
    Ok(CompactTimerStatus {
        service: selection.service.path,
        timer: selection.timer.path,
        state,
        detail,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn private_root() -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        root
    }

    fn fixture(directory: PathBuf) -> Selection {
        let name = "pse-test";
        let (service, timer) =
            unit_bytes(Path::new("/tmp/checkout"), Path::new("/tmp/state"), name).unwrap();
        Selection {
            service: Unit::new(&directory, format!("{name}.service"), service),
            timer: Unit::new(&directory, format!("{name}.timer"), timer),
            directory,
        }
    }

    fn installed(selection: &Selection) {
        for unit in [&selection.service, &selection.timer] {
            prepare_unit(&unit.path, &unit.content).unwrap();
        }
    }

    fn properties(unit: &Unit, active: &str) -> String {
        format!(
            "Id={}\nLoadState=loaded\nActiveState={active}\nFragmentPath={}\nDropInPaths=\nNeedDaemonReload=no\n",
            unit.name,
            unit.path.display()
        )
    }

    #[test]
    fn status_observes_only_prepared_units_and_never_creates_them() {
        let root = private_root();
        let selection = fixture(root.path().join("absent"));
        let state = || observe_selection(&selection, |_| panic!("manager must not be queried")).0;
        assert_eq!(state(), ExpiryState::Unprepared);
        assert!(!selection.directory.exists());

        DirBuilder::new()
            .mode(0o700)
            .create(&selection.directory)
            .unwrap();
        assert_eq!(state(), ExpiryState::Unprepared);
        assert_eq!(fs::read_dir(&selection.directory).unwrap().count(), 0);

        prepare_unit(&selection.service.path, &selection.service.content).unwrap();
        assert_eq!(state(), ExpiryState::Unverified);
        fs::write(&selection.timer.path, b"foreign timer").unwrap();
        assert_eq!(state(), ExpiryState::Unverified);
        assert_eq!(fs::read(&selection.timer.path).unwrap(), b"foreign timer");
    }

    #[test]
    fn status_distinguishes_exact_activity_from_unverified_manager_evidence() {
        let root = private_root();
        let selection = fixture(root.path().to_owned());
        installed(&selection);
        let timer = properties(&selection.timer, "active");
        let service = properties(&selection.service, "inactive");
        let state = |timer: &str, service: &str| {
            observe_selection(&selection, |name| {
                if name == selection.timer.name {
                    Some(timer.as_bytes().to_vec())
                } else {
                    assert_eq!(name, selection.service.name);
                    Some(service.as_bytes().to_vec())
                }
            })
            .0
        };
        assert_eq!(state(&timer, &service), ExpiryState::Active);
        assert_eq!(
            state(&properties(&selection.timer, "inactive"), &service),
            ExpiryState::Inactive
        );
        assert_eq!(
            state(
                "Id=pse-test.timer\nLoadState=not-found\nActiveState=inactive\nFragmentPath=\nDropInPaths=\nNeedDaemonReload=no\n",
                &service
            ),
            ExpiryState::Inactive
        );
        assert_eq!(
            observe_selection(&selection, |_| None).0,
            ExpiryState::ManagerUnavailable
        );
        for conflicting_service in [
            service.replace("DropInPaths=", "DropInPaths=/tmp/foreign.conf"),
            service.replace("NeedDaemonReload=no", "NeedDaemonReload=yes"),
            service.replace("FragmentPath=", "FragmentPath=/tmp/foreign/"),
        ] {
            assert_eq!(state(&timer, &conflicting_service), ExpiryState::Unverified);
        }
        assert_eq!(
            state("not systemctl properties", &service),
            ExpiryState::Unverified
        );
    }

    #[test]
    fn status_rechecks_unit_files_after_reading_manager_ownership() {
        let root = private_root();
        let selection = fixture(root.path().to_owned());
        installed(&selection);
        let observed = observe_selection(&selection, |name| {
            if name == selection.timer.name {
                Some(properties(&selection.timer, "active").into_bytes())
            } else {
                assert_eq!(name, selection.service.name);
                fs::write(&selection.service.path, b"replaced after initial check").unwrap();
                Some(properties(&selection.service, "inactive").into_bytes())
            }
        });
        assert_eq!(observed.0, ExpiryState::Unverified);
        assert_eq!(
            fs::read(&selection.service.path).unwrap(),
            b"replaced after initial check"
        );
    }

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
