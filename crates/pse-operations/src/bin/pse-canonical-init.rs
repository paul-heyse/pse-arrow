// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Ordinary product canonical initialization, independent of fixture and evidence composition.
#![allow(
    clippy::print_stderr,
    reason = "product initializer reports an actionable terminal refusal"
)]
use clap::Parser;
use pse_operations::{
    canonical::{CanonicalError, CanonicalOptions, CanonicalStore},
    deployment::{CanonicalInitializationPhase, CanonicalInitializationReport},
};
use std::{io::{Read, Write}, path::{Path, PathBuf}};

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// Describe the initializer protocol without reading a deployment or starting an executor.
    #[arg(long, conflicts_with = "state")]
    describe: bool,
    /// Provision an explicitly requested control actor in an existing initialized database.
    /// The bounded private JSON file contains the password; it is never a command-line value.
    #[arg(long, value_name = "PRIVATE_JSON", conflicts_with = "describe", requires = "state")]
    issue_control_actor: Option<PathBuf>,
    /// Select an existing fixture database for actor provisioning, never schema installation.
    #[arg(long, requires = "issue_control_actor")]
    database: Option<String>,
    /// Existing explicitly owned deployment state.
    #[arg(required_unless_present = "describe")]
    state: Option<PathBuf>,
}
const ACTOR_REQUEST_BYTES: u64 = 16 * 1024;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ActorRequest {
    document_version: pse_model::document::Version<1>,
    key: String,
    password: String,
    browse: bool,
    cancel: bool,
}

fn actor_request(path: &Path) -> Result<ActorRequest, CanonicalError> {
    let invalid = || CanonicalError::Configuration("control actor request must be a bounded private regular file owned by the current user".into());
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    }
    let file = options.open(path).map_err(|_| invalid())?;
    let metadata = file.metadata().map_err(|_| invalid())?;
    if !metadata.is_file() || metadata.len() > ACTOR_REQUEST_BYTES {
        return Err(invalid());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.mode() & 0o077 != 0 {
            return Err(invalid());
        }
        #[cfg(target_os = "linux")]
        if metadata.uid() != std::fs::metadata("/proc/self").map_err(|_| invalid())?.uid() {
            return Err(invalid());
        }
    }
    let mut bytes = Vec::new();
    file.take(ACTOR_REQUEST_BYTES + 1).read_to_end(&mut bytes).map_err(|_| invalid())?;
    if u64::try_from(bytes.len()).map_err(|_| invalid())? > ACTOR_REQUEST_BYTES {
        return Err(invalid());
    }
    serde_json::from_slice(&bytes).map_err(|_| CanonicalError::Configuration("invalid control actor request document".into()))
}

fn write_report(report: &impl serde::Serialize) -> Result<(), CanonicalError> {
    let mut output = std::io::stdout().lock();
    serde_json::to_writer(&mut output, report)
        .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
    output.write_all(b"\n").map_err(|error| CanonicalError::Configuration(error.to_string()))
}

#[derive(Debug, thiserror::Error)]
enum InitializerError {
    #[error(transparent)]
    Canonical(#[from] CanonicalError),
    #[error("control actor provisioning failed: {issue}; physical disconnect also failed: {drain}")]
    ProvisionAndDrain {
        #[source]
        issue: CanonicalError,
        drain: CanonicalError,
    },
}

fn settle_provision<T>(issue: Result<T, CanonicalError>, drain: Result<(), CanonicalError>) -> Result<T, InitializerError> {
    match (issue, drain) {
        (Ok(report), Ok(())) => Ok(report),
        (Err(issue), Ok(())) => Err(issue.into()),
        (Ok(_), Err(drain)) => Err(drain.into()),
        (Err(issue), Err(drain)) => Err(InitializerError::ProvisionAndDrain {issue, drain}),
    }
}

fn run(cli: Cli) -> Result<(), InitializerError> {
    if let Some(request_path) = cli.issue_control_actor {
        let request = actor_request(&request_path)?;
        let _version = request.document_version;
        let state = cli.state.ok_or_else(|| CanonicalError::Configuration("actor provisioning requires state".into()))?;
        let options = CanonicalOptions::from_state_for_database(&state, cli.database.as_deref())?;
        let executor = tokio::runtime::Builder::new_multi_thread().enable_all().build()
            .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
        let report = executor.block_on(async {
            let store = CanonicalStore::connect(&options).await?;
            let issued = async {
                store.open().await?;
                store.issue_control_actor(&request.key, &request.password, request.browse, request.cancel).await
            }.await;
            let drained = store.disconnect().await;
            settle_provision(issued, drained)
        })?;
        return write_report(&report).map_err(Into::into);
    }
    let phase = if cli.describe {
        CanonicalInitializationPhase::Description
    } else {
        let state = cli
            .state
            .ok_or_else(|| CanonicalError::Configuration("initializer requires state".into()))?;
        let options = CanonicalOptions::from_state(&state)?;
        let executor = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
        executor.block_on(CanonicalStore::initialize_database(&options))?;
        CanonicalInitializationPhase::Initialized
    };
    let report = CanonicalInitializationReport {
        version: pse_model::document::Version,
        canonical_initialize: true,
        phase,
    };
    write_report(&report).map_err(Into::into)
}
fn main() -> std::process::ExitCode {
    match run(Cli::parse()) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("pse-canonical-init: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn description_needs_no_deployment_and_is_distinct_from_initialization() {
        assert!(Cli::try_parse_from(["pse-canonical-init"]).is_err());
        assert!(Cli::try_parse_from(["pse-canonical-init", "--describe", "/state"]).is_err());
        let description = Cli::try_parse_from(["pse-canonical-init", "--describe"]).unwrap();
        assert!(description.describe);
        assert!(description.state.is_none());
        let initialize = Cli::try_parse_from(["pse-canonical-init", "/state"]).unwrap();
        assert!(!initialize.describe);
        assert_eq!(initialize.state, Some(PathBuf::from("/state")));
    }
    #[test]
    fn actor_provisioning_requires_state_and_excludes_description() {
        assert!(Cli::try_parse_from(["pse-canonical-init", "--issue-control-actor", "/private.json"]).is_err());
        assert!(Cli::try_parse_from(["pse-canonical-init", "--describe", "--issue-control-actor", "/private.json"]).is_err());
        let request = Cli::try_parse_from(["pse-canonical-init", "--issue-control-actor", "/private.json", "/state"]).unwrap();
        assert_eq!(request.issue_control_actor, Some(PathBuf::from("/private.json")));
        assert!(!request.describe);
        assert!(Cli::try_parse_from(["pse-canonical-init", "--database", "foreign", "/state"]).is_err());
        let selected = Cli::try_parse_from(["pse-canonical-init", "--issue-control-actor", "/private.json", "--database", "canonical_test_control", "/state"]).unwrap();
        assert_eq!(selected.database.as_deref(), Some("canonical_test_control"));
    }

    #[test]
    #[cfg(unix)]
    fn actor_request_refuses_public_symlink_oversized_and_unknown_documents() {
        use std::{os::unix::fs::PermissionsExt, io::Write};
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("actor.json");
        let secret = br#"{"document_version":1,"key":"browser","password":"not-an-output-secret","browse":true,"cancel":false}"#;
        std::fs::write(&path, secret).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        let request = actor_request(&path).unwrap();
        assert_eq!(request.key, "browser");
        assert!(request.browse && !request.cancel);
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(actor_request(&path).is_err());
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        let alias = directory.path().join("alias.json");
        std::os::unix::fs::symlink(&path, &alias).unwrap();
        assert!(actor_request(&alias).is_err());
        let mut output = std::fs::File::create(&path).unwrap();
        output.write_all(&vec![b' '; usize::try_from(ACTOR_REQUEST_BYTES).unwrap() + 1]).unwrap();
        assert!(actor_request(&path).is_err());
        std::fs::write(&path, br#"{"document_version":1,"key":"browser","password":"private","browse":true,"cancel":false,"extra":1}"#).unwrap();
        let error = actor_request(&path).err().unwrap();
        assert_eq!(error.to_string(), "canonical store configuration: invalid control actor request document");
        assert!(!format!("{error:?}").contains("private"));
        std::fs::write(&path, br#"{"document_version":2,"key":"browser","password":"private","browse":true,"cancel":false}"#).unwrap();
        assert!(actor_request(&path).is_err());
    }

    #[test]
    fn provisioning_and_disconnect_failures_both_survive_settlement() {
        let error = settle_provision::<()>(Err(CanonicalError::PayloadLimit), Err(CanonicalError::Timeout)).unwrap_err();
        assert!(matches!(error, InitializerError::ProvisionAndDrain { issue: CanonicalError::PayloadLimit, drain: CanonicalError::Timeout }));
        assert!(settle_provision(Ok(7), Ok(())).is_ok());
        assert!(matches!(settle_provision::<()>(Ok(()), Err(CanonicalError::Timeout)), Err(InitializerError::Canonical(CanonicalError::Timeout))));
    }

}
