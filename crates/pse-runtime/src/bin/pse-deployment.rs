// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded pre-effect product deployment materialization and validation.
#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "product CLI reports structured bounded results and actionable refusal"
)]

use pse_runtime::{
    boundary_documents::Versioned,
    workflow::{DeploymentRequest, WorkflowError},
};
use std::io::{Read, Write};

fn run() -> Result<(), WorkflowError> {
    if std::env::args_os().len() != 1 {
        return Err(WorkflowError::Input(
            "pse-deployment accepts one versioned JSON request on stdin".into(),
        ));
    }
    let mut input = Vec::new();
    std::io::stdin()
        .take(1024 * 1024 + 1)
        .read_to_end(&mut input)
        .map_err(|error| WorkflowError::Input(error.to_string()))?;
    if input.len() > 1024 * 1024 {
        return Err(WorkflowError::Input(
            "deployment request exceeds its bounded envelope".into(),
        ));
    }
    let request: Versioned<DeploymentRequest, 1> =
        serde_json::from_slice(&input).map_err(|error| WorkflowError::Input(error.to_string()))?;
    let response = Versioned::<_, 1>::new(request.body.materialize()?);
    let mut output = std::io::stdout().lock();
    serde_json::to_writer(&mut output, &response)
        .map_err(|error| WorkflowError::Input(error.to_string()))?;
    output
        .write_all(b"\n")
        .map_err(|error| WorkflowError::Input(error.to_string()))?;
    Ok(())
}
fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("pse-deployment: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
