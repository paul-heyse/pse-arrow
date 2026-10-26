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
use std::{io::Write, path::PathBuf};

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// Describe the initializer protocol without reading a deployment or starting an executor.
    #[arg(long, conflicts_with = "state")]
    describe: bool,
    /// Existing explicitly owned deployment state.
    #[arg(required_unless_present = "describe")]
    state: Option<PathBuf>,
}
fn run(cli: Cli) -> Result<(), CanonicalError> {
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
    let mut output = std::io::stdout().lock();
    serde_json::to_writer(&mut output, &report)
        .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
    output
        .write_all(b"\n")
        .map_err(|error| CanonicalError::Configuration(error.to_string()))
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
}
