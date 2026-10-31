// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original evidence reads and explicitly requested native compiler generation.
use super::{GenerationIntent, HostError, OutputOptions, checkout_root};
use crate::{
    compiler_investigation::{self, DetailCursor, DiagnosticAddress, DiagnosticCursor, artifact},
    envelope::DetailView,
    generation::EntryGuard,
    paths::CheckoutState,
};
use clap::Subcommand;
use std::{
    num::{NonZeroU64, NonZeroUsize},
    path::Path,
};

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
        /// Native MIR body substring; native filter operators and 'all' are refused.
        #[arg(long)]
        body: Option<String>,
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
    /// Observe the original compiler-artifact context, status and completed file digests.
    Artifacts { invocation: String },
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
        #[arg(long = "pass")]
        passes: Vec<String>,
    },
    /// Read one original indexed opaque file after whole-file SHA verification; never compile.
    File {
        invocation: String,
        file: usize,
        #[arg(long, default_value = "0")]
        offset: u64,
        #[arg(long, default_value = "4096")]
        length: NonZeroU64,
    },
    /// Page original retained check failures; related native detail is a separate observation.
    Diagnostics {
        invocation: String,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Read one addressed original rustc diagnostic, its suggestions and related spans.
    Diagnostic {
        /// Opaque original address returned by compiler diagnostics.
        #[arg(long)]
        address: String,
        #[arg(long)]
        cursor: Option<String>,
        /// Include exact original JSON as opaque evidence, preserving original numeric lexemes.
        #[arg(long)]
        raw: bool,
        /// Compare retained native source lines to independently acquired current candidates.
        #[arg(long)]
        source: bool,
        /// Whole-record decoding bound; oversized originals stay in their raw artifact.
        #[arg(long, default_value = "4194304")]
        max_record_bytes: NonZeroUsize,
    },
}
impl Action {
    pub(super) fn intent(&self) -> GenerationIntent {
        match self {
            Self::Generate { .. } | Self::Probe { .. } => GenerationIntent::NewWork,
            _ => GenerationIntent::Passive,
        }
    }
}

fn decoded<T: serde::de::DeserializeOwned>(text: &str) -> Result<T, HostError> {
    serde_json::from_str(text)
        .map_err(std::io::Error::other)
        .map_err(HostError::from)
}
pub(super) fn dispatch(
    action: Action,
    selected: Option<&Path>,
    output: &OutputOptions,
    entry: &EntryGuard,
) -> Result<i32, HostError> {
    let root = checkout_root(selected)?;
    let store = super::invocation_store(selected)?;
    let limit = NonZeroUsize::new(output.presentation.budget.unwrap_or(20))
        .ok_or_else(|| HostError::Ownership("compiler detail allowance must be positive".into()))?;
    match action {
        Action::Generate {
            package,
            kind,
            question,
            body,
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
        Action::Artifacts { invocation } => {
            let environment = std::env::vars_os().collect();
            let state = CheckoutState::resolve(&root, &environment)?;
            Ok(output.emit(artifact::observe(&root, &state, &invocation)?)?)
        }
        Action::Probe {
            name,
            kind,
            question,
            body,
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
        Action::File {
            invocation,
            file,
            offset,
            length,
        } => {
            let environment = std::env::vars_os().collect();
            let state = CheckoutState::resolve(&root, &environment)?;
            Ok(output.emit(artifact::read_file(
                &root,
                &state,
                &invocation,
                file,
                offset,
                length,
            )?)?)
        }
        Action::Diagnostics {
            invocation,
            session,
            cursor,
        } => {
            let environment = std::env::vars_os().collect();
            let session = crate::invocation::select_session(session, &environment)?;
            let cursor: Option<DiagnosticCursor> = cursor.as_deref().map(decoded).transpose()?;
            let response = compiler_investigation::diagnostics(
                &store,
                &invocation,
                session.as_deref(),
                cursor.as_ref(),
                limit,
                &root,
            )?;
            let omitted = response
                .payload
                .retained_references
                .saturating_sub(response.payload.records.len());
            Ok(output.emit_page(
                response,
                DetailView {
                    budget: Some(limit.get()),
                    omitted_records: omitted,
                    truncated: omitted > 0,
                    ..DetailView::default()
                },
            )?)
        }
        Action::Diagnostic {
            address,
            cursor,
            raw,
            source,
            max_record_bytes,
        } => {
            let address: DiagnosticAddress = decoded(&address)?;
            let cursor: Option<DetailCursor> = cursor.as_deref().map(decoded).transpose()?;
            let evidence = compiler_investigation::diagnostic(&store, &address, max_record_bytes)?;
            let response = evidence.page(&root, cursor.as_ref(), limit, raw, source)?;
            let omitted = response
                .payload
                .total_related_records
                .saturating_sub(response.payload.records.len());
            Ok(output.emit_page(
                response,
                DetailView {
                    budget: Some(limit.get()),
                    omitted_records: omitted,
                    truncated: omitted > 0,
                    ..DetailView::default()
                },
            )?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser as _;
    #[test]
    fn original_evidence_is_passive_and_requires_explicit_record_selection() {
        let cli = super::super::Cli::try_parse_from([
            "pse",
            "compiler",
            "diagnostic",
            "--address",
            "{}",
            "--source",
            "--raw",
            "--budget",
            "2",
        ])
        .unwrap();
        assert_eq!(
            cli.command.as_ref().unwrap().generation_intent(),
            GenerationIntent::Passive
        );
        let Some(super::super::Command::Compiler {
            action: Action::Diagnostic { source, raw, .. },
        }) = cli.command
        else {
            panic!("selected original reader")
        };
        assert!(source && raw);
        assert!(super::super::Cli::try_parse_from(["pse", "compiler", "diagnostic"]).is_err());
        assert!(
            super::super::Cli::try_parse_from([
                "pse",
                "compiler",
                "diagnostics",
                "latest",
                "--detach"
            ])
            .is_err()
        );
    }
    #[test]
    fn compiler_generation_is_explicit_and_original_file_reads_remain_passive() {
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
        for arguments in [
            vec!["pse", "compiler", "artifacts", "latest"],
            vec!["pse", "compiler", "file", "latest", "0", "--length", "64"],
        ] {
            let cli = super::super::Cli::try_parse_from(arguments).unwrap();
            assert_eq!(
                cli.command.as_ref().unwrap().generation_intent(),
                GenerationIntent::Passive
            );
        }
        assert!(
            super::super::Cli::try_parse_from([
                "pse", "compiler", "generate", "case", "--kind", "mir"
            ])
            .is_err()
        );
    }
}
