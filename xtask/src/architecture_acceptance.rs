// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Active-plan CLI adapter. Coverage and execution receipts are owned by scripts.
use anyhow::{Context, Result, ensure};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone, Debug, clap::Args)]
pub(crate) struct RunOptions {
    /// Select the active implementation plan.
    #[arg(long, default_value_t = 0)]
    plan: u8,
    /// Link a prior campaign; retain its successful, unaffected gates.
    #[arg(long)]
    resume_from: Option<PathBuf>,
    /// Invalidate and rerun a named gate when continuing after a repair.
    #[arg(long, requires = "resume_from")]
    rerun: Vec<String>,
    /// Explain source changes and why the explicitly retained evidence still applies.
    #[arg(long, requires = "resume_from")]
    change_reason: Option<String>,
    /// Execute functional qualification or the subsequent measurement phase.
    #[arg(long, default_value = "functional", value_parser = ["functional", "performance"])]
    phase: String,
    /// Qualified current functional campaign required before performance execution.
    #[arg(long)]
    functional_from: Option<PathBuf>,
    /// Retain measurements with review collection explicitly pending.
    #[arg(long, value_parser = ["plan14-measure"])]
    stop_after: Option<String>,
}
impl Default for RunOptions {
    fn default() -> Self {
        Self {
            plan: 0,
            resume_from: None,
            rerun: Vec::new(),
            change_reason: None,
            phase: "functional".into(),
            functional_from: None,
            stop_after: None,
        }
    }
}
pub(crate) fn run(root: &Path, output: &Path, options: &RunOptions) -> Result<()> {
    let plan = active_plan(root)?;
    ensure!(
        options.plan == 0 || options.plan == plan,
        "historical plan cannot authorize current execution"
    );
    let mut command = Command::new(root.join(if cfg!(windows) {
        ".venv/Scripts/python.exe"
    } else {
        ".venv/bin/python"
    }));
    command
        .current_dir(root)
        .args(["-m", "scripts.validation", "--output"])
        .arg(output)
        .args(["--phase", &options.phase, "--plan", &plan.to_string()]);
    if let Some(functional) = &options.functional_from {
        command.arg("--functional-from").arg(functional);
    }
    if let Some(gate) = &options.stop_after {
        command.arg("--stop-after").arg(gate);
    }
    if let Some(parent) = &options.resume_from {
        command.arg("--resume-from").arg(parent);
    }
    if let Some(reason) = &options.change_reason {
        command.arg("--change-reason").arg(reason);
    }
    for gate in &options.rerun {
        command.args(["--rerun", gate]);
    }
    ensure!(
        command.status()?.success(),
        "active-plan assessment has unresolved required evidence"
    );
    Ok(())
}

fn active_plan(root: &Path) -> Result<u8> {
    let manifest: toml::Value = toml::from_str(&std::fs::read_to_string(root.join("Cargo.toml"))?)?;
    let plan = manifest["workspace"]["metadata"]["pse"]["execution"]["active-plan"]
        .as_integer()
        .context("missing active execution plan")?;
    ensure!(plan == 14, "unsupported active execution plan");
    Ok(u8::try_from(plan)?)
}

#[cfg(test)]
mod consolidation_unit;
