// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit native compiler generation and authored compiler probes.
use super::{GenerationIntent, HostError, OutputOptions, checkout_root};
use crate::{compiler_investigation::artifact, generation::EntryGuard, paths::CheckoutState};
use clap::Subcommand;
use std::path::Path;

#[derive(Debug, Subcommand)]
pub(super) enum Action {
    /// Explicitly generate native selected-unit/body artifacts through correctness admission.
    Generate {
        /// One Cargo package; default target is its library unless --bin is selected.
        package: String,
        #[arg(long, value_enum)]
        kind: artifact::Kind,
        /// Original investigation question, retained without inferring an answer.
        #[arg(long)]
        question: String,
        /// MIR substring, or exact owning body path for facts (including nested closures/coroutines).
        #[arg(long)]
        body: Option<String>,
        /// Native query family for typed facts; other artifact kinds require the all default.
        #[arg(long, value_enum, default_value = "all")]
        fact_focus: artifact::FactFocus,
        /// Native MIR pass substrings; select before/after artifacts at their actual phase.
        #[arg(long = "pass")]
        passes: Vec<String>,
        #[arg(long)]
        bin: Option<String>,
        #[arg(long, conflicts_with = "bin")]
        lib: bool,
        /// Explicit Cargo target; no component is installed automatically.
        #[arg(long)]
        target: Option<String>,
        #[command(flatten)]
        features: clap_cargo::Features,
    },
    /// Explicitly evaluate an authored probe or generate native artifacts from its isolated unit.
    Probe {
        name: String,
        #[arg(long, value_enum, default_value = "evaluate")]
        kind: artifact::Kind,
        /// Original constant/layout/ownership/instance question; authored source owns stdout meaning.
        #[arg(long)]
        question: String,
        #[arg(long)]
        body: Option<String>,
        /// Native query family for typed facts.
        #[arg(long, value_enum, default_value = "all")]
        fact_focus: artifact::FactFocus,
        #[arg(long = "pass")]
        passes: Vec<String>,
    },
}
impl Action {
    pub(super) fn intent(&self) -> GenerationIntent {
        match self {
            Self::Generate { .. } | Self::Probe { .. } => GenerationIntent::NewWork,
        }
    }
}

pub(super) fn dispatch(
    action: Action,
    selected: Option<&Path>,
    output: &OutputOptions,
    entry: &EntryGuard,
) -> Result<i32, HostError> {
    let root = checkout_root(selected)?;
    let store = super::invocation_store(selected)?;
    match action {
        Action::Generate {
            package,
            kind,
            question,
            body,
            fact_focus,
            passes,
            bin,
            lib: _,
            target,
            features,
        } => {
            let environment = std::env::vars_os().collect();
            let state = CheckoutState::resolve(&root, &environment)?;
            let host=entry.prepared().ok_or_else(||HostError::Ownership("compiler generation requires explicitly prepared native host tools; run scripts/bootstrap-host".into()))?;
            let mut feature_arguments = Vec::new();
            if features.no_default_features {
                feature_arguments.push("--no-default-features".into());
            }
            if features.all_features {
                feature_arguments.push("--all-features".into());
            }
            for feature in features.features {
                feature_arguments.extend(["--features".into(), feature.into()]);
            }
            let control_root = entry.checkout().ok_or_else(|| {
                HostError::Ownership("original compiler control checkout is absent".into())
            })?;
            let prepared = artifact::prepare(
                &root,
                &control_root,
                &state,
                &output.invocation_id,
                artifact::Request {
                    kind,
                    package,
                    binary: bin,
                    target,
                    feature_arguments,
                    body,
                    passes,
                    question,
                    fact_focus,
                },
                &environment,
                host,
            )?;
            prepared.verify()?;
            super::launch_managed_finalize(
                super::ManagedCommand {
                    parent: None,
                    root: root.clone(),
                    cwd: root.clone(),
                    environment: prepared.environment.clone(),
                    command: prepared.command.clone(),
                    detach: false,
                    session: None,
                    resource_class: None,
                    verb: "compiler generate",
                    tool: None,
                    workflow: None,
                },
                entry,
                output,
                prepared.references(),
                vec![prepared.provider()?],
                || artifact::finish(&prepared, &store),
            )
        }
        Action::Probe {
            name,
            kind,
            question,
            body,
            fact_focus,
            passes,
        } => {
            let environment = std::env::vars_os().collect();
            let state = CheckoutState::resolve(&root, &environment)?;
            let host=entry.prepared().ok_or_else(||HostError::Ownership("compiler probe requires explicitly prepared native host tools; run scripts/bootstrap-host".into()))?;
            let control_root = entry.checkout().ok_or_else(|| {
                HostError::Ownership("original compiler control checkout is absent".into())
            })?;
            let (prepared, run) = artifact::prepare_probe(
                &root,
                &control_root,
                &state,
                &output.invocation_id,
                artifact::ProbeRequest {
                    name,
                    kind,
                    question,
                    body,
                    passes,
                    fact_focus,
                },
                &environment,
                host,
            )?;
            prepared.verify()?;
            run.record_managed(
                &root,
                &prepared.environment,
                crate::probe::ManagedRun {
                    invocation: output.invocation_id.clone(),
                    command_name: "compiler probe".into(),
                    command: prepared.command.clone(),
                    control: host.clone(),
                    control_checkout: control_root,
                },
            )
            .map_err(|error| HostError::Probe(Box::new(error)))?;
            let status = super::launch_managed_finalize(
                super::ManagedCommand {
                    parent: None,
                    root: root.clone(),
                    cwd: run.probe.directory.clone(),
                    environment: prepared.environment.clone(),
                    command: prepared.command.clone(),
                    detach: false,
                    session: None,
                    resource_class: None,
                    verb: "compiler probe",
                    tool: None,
                    workflow: None,
                },
                entry,
                output,
                prepared.references(),
                vec![prepared.provider()?],
                || artifact::finish(&prepared, &store),
            );
            drop(run.lease);
            status
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser as _;
    #[test]
    fn compiler_generation_and_probe_are_explicit_and_require_original_questions() {
        let generated = super::super::Cli::try_parse_from([
            "pse",
            "compiler",
            "generate",
            "case",
            "--kind",
            "mir",
            "--body",
            "partial",
            "--pass",
            "ElaborateDrops",
            "--question",
            "remaining field drops",
            "--lib",
        ])
        .unwrap();
        assert_eq!(
            generated.command.as_ref().unwrap().generation_intent(),
            GenerationIntent::NewWork
        );
        let probe = super::super::Cli::try_parse_from([
            "pse",
            "compiler",
            "probe",
            "layout",
            "--question",
            "concrete layout",
        ])
        .unwrap();
        assert_eq!(
            probe.command.as_ref().unwrap().generation_intent(),
            GenerationIntent::NewWork
        );
        assert!(
            super::super::Cli::try_parse_from([
                "pse", "compiler", "generate", "case", "--kind", "mir"
            ])
            .is_err()
        );
    }
}
