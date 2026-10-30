// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native readiness transport for the authenticated original service main process.
use crate::{HostError, HostManager, LaunchIntent, UnitObservation};
use sd_notify::NotifyState;
use std::{ffi::OsString, path::Path};

/// Readiness belongs to the original service controller; notification transport grants no authority.
#[derive(Debug)]
pub struct SupervisorNotifier {
    manager: HostManager,
    intent: LaunchIntent,
    original: UnitObservation,
    endpoint: OsString,
}
impl SupervisorNotifier {
    /// Authenticate the actual service main process and its configured notification transport.
    /// Merely setting `NOTIFY_SOCKET` cannot establish a service-controller role.
    pub async fn authenticate(
        manager: &HostManager,
        intent: &LaunchIntent,
    ) -> Result<Self, HostError> {
        let original = manager.authenticate_service_controller(intent).await?;
        Ok(Self {
            manager: manager.clone(),
            intent: intent.clone(),
            original,
            endpoint: endpoint()?,
        })
    }
    async fn validate(&self) -> Result<(), HostError> {
        let actual = self
            .manager
            .authenticate_service_controller(&self.intent)
            .await?;
        if actual.invocation != self.original.invocation
            || actual.control_group != self.original.control_group
        {
            return Err(HostError::Ownership(
                "notification service incarnation changed".into(),
            ));
        }
        if endpoint()? != self.endpoint {
            return Err(HostError::Ownership(
                "service notification endpoint changed".into(),
            ));
        }
        Ok(())
    }
    /// Send native readiness after owner initialization. The launcher must still observe the
    /// matching systemd job/unit; successful datagram transmission is not manager acknowledgement.
    pub async fn send_ready(&self, status: &str) -> Result<(), HostError> {
        self.validate().await?;
        send(&self.endpoint, &[NotifyState::Ready], status)
    }
    /// Announce cooperative stopping; callers must independently settle payloads and capture.
    pub async fn send_stopping(&self, status: &str) -> Result<(), HostError> {
        self.validate().await?;
        send(&self.endpoint, &[NotifyState::Stopping], status)
    }
    /// Publish one current status, without inventing a historical observation or changing results.
    pub async fn send_status(&self, status: &str) -> Result<(), HostError> {
        self.validate().await?;
        send(&self.endpoint, &[], status)
    }
}
fn endpoint() -> Result<OsString, HostError> {
    let value = std::env::var_os("NOTIFY_SOCKET").ok_or_else(|| {
        HostError::Ownership("managed notify service has no NOTIFY_SOCKET".into())
    })?;
    // sd-notify0.5 uses pathname UnixDatagram::connect. Do not silently pretend its
    // optional no-op or an unsupported abstract/vsock address provides readiness.
    if !Path::new(&value).is_absolute() {
        return Err(HostError::Ownership(
            "sd-notify0.5 requires an absolute pathname notification socket".into(),
        ));
    }
    Ok(value)
}
fn send(expected: &OsString, states: &[NotifyState<'_>], status: &str) -> Result<(), HostError> {
    if status.contains(['\n', '\r', '\0']) {
        return Err(HostError::Ownership(
            "service status must be a single protocol-safe line".into(),
        ));
    }
    if &endpoint()? != expected {
        return Err(HostError::Ownership(
            "notification endpoint no longer matches the authenticated service".into(),
        ));
    }
    let mut states = states.to_vec();
    states.push(NotifyState::Status(status));
    sd_notify::notify(&states)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        os::unix::net::UnixDatagram,
        process::{Command, Stdio},
        time::Duration,
    };

    #[test]
    fn native_notification_transport_and_required_endpoint_failures() {
        const CASE: &str = "PSE_NATIVE_NOTIFY_TEST_CASE";
        if let Ok(case) = std::env::var(CASE) {
            match case.as_str() {
                "ready" => {
                    let endpoint = endpoint().unwrap();
                    send(
                        &endpoint,
                        &[NotifyState::Ready],
                        "owner and origin lease initialized",
                    )
                    .unwrap();
                    assert!(send(&endpoint, &[], "line\nREADY=1").is_err());
                }
                "missing" => assert!(endpoint().is_err()),
                "abstract" => assert!(endpoint().is_err()),
                "unavailable" => {
                    let endpoint = endpoint().unwrap();
                    assert!(send(&endpoint, &[NotifyState::Ready], "not delivered").is_err());
                }
                _ => panic!("unknown test mode"),
            }
            return;
        }
        let directory = tempfile::tempdir().unwrap();
        let socket_path = directory.path().join("notify.sock");
        let socket = UnixDatagram::bind(&socket_path).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let executable = std::env::current_exe().unwrap();
        for case in ["ready", "missing", "abstract", "unavailable"] {
            let mut command = Command::new(&executable);
            command.args(["--exact", "managed::readiness::tests::native_notification_transport_and_required_endpoint_failures", "--nocapture"])
                .env(CASE, case).stdout(Stdio::piped()).stderr(Stdio::piped());
            match case {
                "ready" => {
                    command.env("NOTIFY_SOCKET", &socket_path);
                }
                "missing" => {
                    command.env_remove("NOTIFY_SOCKET");
                }
                "abstract" => {
                    command.env("NOTIFY_SOCKET", "@unsupported");
                }
                "unavailable" => {
                    command.env("NOTIFY_SOCKET", directory.path().join("missing.sock"));
                }
                _ => panic!("unsupported notification fixture mode: {case}"),
            }
            let output = command.output().unwrap();
            assert!(output.status.success(), "case {case}: {:?}", output);
            if case == "ready" {
                let mut bytes = [0; 256];
                let count = socket.recv(&mut bytes).unwrap();
                assert_eq!(
                    &bytes[..count],
                    b"READY=1\nSTATUS=owner and origin lease initialized\n"
                );
            }
        }
    }
}
