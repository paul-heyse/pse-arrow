// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::UnitObservation;
use std::{cell::RefCell, os::unix::fs::symlink};

struct Control {
    calls: RefCell<Vec<String>>,
    states: RefCell<BTreeMap<String, InstalledServiceState>>,
    fail_inspect: Option<String>,
    fail_stop: bool,
    fail_new_start: bool,
    original: String,
}
impl Control {
    fn new(config: &Configuration) -> Self {
        let states = TOOLS
            .into_iter()
            .map(|name| {
                let unit = config.units[name].clone();
                let state = InstalledServiceState {
                    observation: UnitObservation {
                        unit: unit.clone(),
                        invocation: vec![7; 16],
                        active_state: "active".into(),
                        sub_state: "running".into(),
                        main_pid: 37,
                        exit_code: 0,
                        exit_status: 0,
                        control_group: "/fixture".into(),
                    },
                    enabled: "disabled".into(),
                    process: Some(pse_diagnostics::process_facts::ProcessIdentity {
                        boot_id: "fixture".into(),
                        pid_namespace: "fixture".into(),
                        pid: 37,
                        start_ticks: 71,
                    }),
                };
                (unit, state)
            })
            .collect();
        Self {
            calls: RefCell::new(vec![]),
            states: RefCell::new(states),
            fail_inspect: None,
            fail_stop: false,
            fail_new_start: false,
            original: config.configuration_id.clone(),
        }
    }
}
impl ServiceControl for Control {
    async fn reload(&self) -> Result<(), HostError> {
        self.calls.borrow_mut().push("reload".into());
        Ok(())
    }
    async fn inspect(
        &self,
        spec: &InstalledServiceSpec,
        load: bool,
    ) -> Result<Option<InstalledServiceState>, HostError> {
        self.calls
            .borrow_mut()
            .push(format!("inspect:{}:{load}", spec.name));
        if self
            .fail_inspect
            .as_ref()
            .is_some_and(|name| spec.name.ends_with(name))
        {
            return Err(invalid("changed owner"));
        }
        Ok(self.states.borrow().get(&spec.name).cloned())
    }
    async fn start(&self, spec: &InstalledServiceSpec) -> Result<(), HostError> {
        self.calls.borrow_mut().push(format!("start:{}", spec.name));
        if self.fail_new_start && !spec.name.contains(&self.original) {
            return Err(invalid("candidate activation failed"));
        }
        Ok(())
    }
    async fn stop(
        &self,
        spec: &InstalledServiceSpec,
        expected: &InstalledServiceState,
    ) -> Result<(), HostError> {
        self.calls.borrow_mut().push(format!("stop:{}", spec.name));
        if self.fail_stop {
            return Err(invalid(format!("{} stop failed", spec.name)));
        }
        if self.states.borrow().get(&spec.name) != Some(expected) {
            return Err(invalid("changed incarnation"));
        }
        self.states.borrow_mut().remove(&spec.name);
        Ok(())
    }
}
async fn fixture() -> (tempfile::TempDir, Environment, Configuration) {
    let directory = tempfile::tempdir().unwrap();
    let state = directory.path().join("state");
    private_directory(&state, true).unwrap();
    let root = directory.path();
    fs::create_dir(root.join(".config")).unwrap();
    let binary = root.join("collector-fixture");
    fs::write(&binary, b"#!/bin/sh\nexit 0\n").unwrap();
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).unwrap();
    let hash = digest(&binary).unwrap();
    let declarations = format!(
        "[otelcol-contrib]\nversion='fixture'\nurl='https://example.invalid/collector'\nsha256='{}'\nbinary_sha256='{hash}'\nmember='collector'\narchive='tar.gz'\n[openobserve]\nversion='fixture'\nurl='https://example.invalid/backend'\nsha256='{}'\nbinary_sha256='{hash}'\nmember='backend'\narchive='tar.gz'\n",
        "0".repeat(64),
        "0".repeat(64)
    );
    fs::write(root.join(".config/diagnostic-tools.toml"), declarations).unwrap();
    let environment = Environment::from([
        ("HOME".into(), root.into()),
        (
            "XDG_CONFIG_HOME".into(),
            root.join("config").into_os_string(),
        ),
        (
            "DBUS_SESSION_BUS_ADDRESS".into(),
            "unix:path=/definitely/missing-bus".into(),
        ),
    ]);
    for tool in [Tool::Collector, Tool::OpenObserve] {
        let target = diagnostic_tools::binary_path(root, tool, &environment).unwrap();
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::copy(&binary, &target).unwrap();
    }
    let listeners: Vec<_> = (0..default_ports().len())
        .map(|_| TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap())
        .collect();
    let ports = default_ports()
        .into_keys()
        .zip(
            listeners
                .iter()
                .map(|listener| listener.local_addr().unwrap().port()),
        )
        .collect();
    drop(listeners);
    setup(
        root,
        &environment,
        Some(&state),
        &SetupOptions {
            ports,
            ..SetupOptions::default()
        },
    )
    .await
    .unwrap();
    let config = checked_configuration(&state).unwrap();
    (directory, environment, config)
}
#[test]
fn collector_retains_persistent_queues_retry_failure_metrics_and_alert_policy() {
    let config = collector_configuration(
        Path::new("/private/state"),
        &default_ports(),
        "default",
        &default_streams(),
        "generation",
    )
    .unwrap();
    for signal in ["traces", "metrics", "logs"] {
        let exporter = &config["exporters"][format!("otlp_http/{signal}")];
        assert_eq!(exporter["sending_queue"]["queue_size"], 1000);
        assert_eq!(exporter["sending_queue"]["storage"], "file_storage");
        assert_eq!(exporter["retry_on_failure"]["max_elapsed_time"], "300s");
        assert_eq!(
            exporter["headers"]["Authorization"],
            "${env:PSE_TELEMETRY_AUTHORIZATION}"
        );
    }
    assert_eq!(config["extensions"]["file_storage"]["fsync"], true);
    assert_eq!(
        config["service"]["telemetry"]["metrics"]["readers"][1]["pull"]["exporter"]["prometheus"]["port"],
        8888
    );
    assert_eq!(ALERT_CIDR, "127.0.0.1/32");
}
#[tokio::test]
async fn setup_keeps_private_credentials_sealed_generation_and_no_unit_publication() {
    let (_directory, environment, config) = fixture().await;
    assert!(!unit_directory(&environment).unwrap().exists());
    let public = fs::read(config.state.join("configuration.json")).unwrap();
    assert!(!String::from_utf8(public).unwrap().contains("Basic "));
    let values = environment_values(&config.env_file()).unwrap();
    assert_eq!(values["ZO_SSRF_ALLOWED_CIDRS"], ALERT_CIDR);
    assert_eq!(
        fs::metadata(config.env_file()).unwrap().mode() & 0o777,
        0o600
    );
    assert!(
        setup(
            Path::new("/unused"),
            &environment,
            Some(&config.state),
            &SetupOptions::default()
        )
        .await
        .is_err()
    );
    let observed = inspect(&environment, Some(&config.state)).await.unwrap();
    assert_eq!(
        observed["services"]["collector"]["ownership"],
        "unavailable"
    );
    assert!(!unit_directory(&environment).unwrap().exists());
}
#[tokio::test]
async fn passive_inspection_and_help_never_create_state_or_request_manager_effects() {
    let directory = tempfile::tempdir().unwrap();
    let environment = Environment::from([("HOME".into(), directory.path().into())]);
    let selected = directory.path().join("absent");
    let observed = inspect(&environment, Some(&selected)).await.unwrap();
    assert_eq!(observed["status"], "unconfigured");
    assert!(!selected.exists());
    command(directory.path(), &environment, &["--help".into()])
        .await
        .unwrap();
    assert!(!directory.path().join(".local").exists());
    let (_directory, environment, config) = fixture().await;
    let control = Control::new(&config);
    inspect_with(&config, &unit_directory(&environment).unwrap(), &control)
        .await
        .unwrap();
    assert!(
        control
            .calls
            .borrow()
            .iter()
            .all(|call| call.starts_with("inspect:") && call.ends_with(":false"))
    );
}
#[tokio::test]
async fn changed_binary_generation_and_symlink_parent_are_rejected() {
    let (_directory, _environment, config) = fixture().await;
    let collector = config.generation.join("collector.yaml");
    fs::write(&collector, b"{}").unwrap();
    assert!(checked_configuration(&config.state).is_err());
    let (directory, _environment, config) = fixture().await;
    fs::write(&config.binaries["collector"].path, b"changed").unwrap();
    assert!(checked_configuration(&config.state).is_err());
    let alias = directory.path().join("alias");
    symlink(&config.state, &alias).unwrap();
    assert!(state_directory(Some(&alias), &Environment::new()).is_err());
}
#[tokio::test]
async fn stop_settles_independently_authenticated_owner_and_keeps_all_failures() {
    let (_directory, environment, config) = fixture().await;
    let directory = unit_directory(&environment).unwrap();
    let mut control = Control::new(&config);
    control.fail_inspect = Some("openobserve.service".into());
    assert!(stop_with(&config, &directory, &control).await.is_err());
    {
        let calls = control.calls.borrow();
        let stops: Vec<_> = calls
            .iter()
            .filter(|call| call.starts_with("stop:"))
            .collect();
        assert_eq!(stops.len(), 1);
        assert!(stops[0].ends_with("collector.service"));
    }
    let mut control = Control::new(&config);
    control.fail_inspect = Some("openobserve.service".into());
    control.fail_stop = true;
    assert!(matches!(
        stop_with(&config, &directory, &control).await,
        Err(HostError::Settlement { .. })
    ));
    assert_eq!(
        control
            .calls
            .borrow()
            .iter()
            .filter(|call| call.starts_with("stop:"))
            .count(),
        1
    );
    let mut control = Control::new(&config);
    control.fail_stop = true;
    let error = stop_with(&config, &directory, &control).await.unwrap_err();
    assert!(matches!(error, HostError::Settlement { .. }));
    assert_eq!(
        control
            .calls
            .borrow()
            .iter()
            .filter(|call| call.starts_with("stop:"))
            .count(),
        2
    );
}
#[tokio::test]
async fn reconfiguration_keeps_credentials_queue_data_and_original_for_recovery() {
    let (_directory, environment, config) = fixture().await;
    fs::write(config.state.join("collector-queue/retained"), b"queue").unwrap();
    fs::write(config.state.join("openobserve-data/retained"), b"data").unwrap();
    let credentials = environment_values(&config.env_file()).unwrap();
    let directory = unit_directory(&environment).unwrap();
    let control = Control::new(&config);
    reconfigure_with(&config.state, &config, &environment, &directory, &control)
        .await
        .unwrap();
    let candidate = checked_configuration(&config.state).unwrap();
    assert_ne!(candidate.configuration_id, config.configuration_id);
    assert_eq!(
        environment_values(&candidate.env_file()).unwrap(),
        credentials
    );
    assert_eq!(
        fs::read(config.state.join("collector-queue/retained")).unwrap(),
        b"queue"
    );
    assert_eq!(
        fs::read(config.state.join("openobserve-data/retained")).unwrap(),
        b"data"
    );
    validate_configuration(&config.state, &config, &config.public_configuration).unwrap();
    let receipt: Reconfiguration = read_json(&config.state.join("reconfiguration.json")).unwrap();
    assert_eq!(receipt.phase, "activated");
    let calls = control.calls.borrow();
    let stops = calls
        .iter()
        .filter(|call| call.starts_with("stop:"))
        .count();
    assert_eq!(stops, 2);
    let starts: Vec<_> = calls
        .iter()
        .filter(|call| call.starts_with("start:"))
        .collect();
    assert!(starts[0].ends_with("openobserve.service"));
    assert!(starts[1].ends_with("collector.service"));
}
#[tokio::test]
async fn candidate_activation_failure_restores_previous_selection_with_receipt() {
    let (_directory, environment, config) = fixture().await;
    let directory = unit_directory(&environment).unwrap();
    let mut control = Control::new(&config);
    control.fail_new_start = true;
    assert!(
        reconfigure_with(&config.state, &config, &environment, &directory, &control)
            .await
            .is_err()
    );
    let selected = checked_configuration(&config.state).unwrap();
    assert_eq!(selected.configuration_id, config.configuration_id);
    let receipt: Reconfiguration = read_json(&config.state.join("reconfiguration.json")).unwrap();
    assert_eq!(receipt.phase, "rolled_back");
}
#[tokio::test]
async fn login_activation_refuses_before_reconfiguration_effects() {
    let (_directory, environment, config) = fixture().await;
    let directory = unit_directory(&environment).unwrap();
    let control = Control::new(&config);
    for state in control.states.borrow_mut().values_mut() {
        state.enabled = "enabled".into();
    }
    assert!(
        reconfigure_with(&config.state, &config, &environment, &directory, &control)
            .await
            .is_err()
    );
    assert!(!config.state.join("reconfiguration.json").exists());
    assert!(
        control
            .calls
            .borrow()
            .iter()
            .all(|call| call.starts_with("inspect:") && call.ends_with(":false"))
    );
}
#[test]
fn unit_escaping_and_port_validation_reject_ambiguous_input() {
    let mut options = SetupOptions::default();
    options.ports.insert("health".into(), 4318);
    assert!(options.validate().is_err());
    options.ports.clear();
    options.organization = "../tenant".into();
    assert!(options.validate().is_err());
    assert_eq!(
        unit_quote("/a%b$c\\d\"e").unwrap(),
        "\"/a%%b$$c\\\\d\\\"e\""
    );
    assert!(unit_quote("/bad\npath").is_err());
}
