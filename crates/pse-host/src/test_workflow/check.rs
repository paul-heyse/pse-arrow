// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Concrete compiler-check stages and required Cargo observation, without process launch.
use super::{Context, Explanation, GroupCommand, Stage, WorkflowError, pending};
use crate::{
    catalog::description::Information,
    correctness, discovery,
    envelope::RedactionPolicy,
    extraction::{
        CargoExtractor, CargoRecord, CargoSummary, CompilerSpan, ExtractionError, OutputStream,
        RecordOrigin,
    },
    generation::PreparedGeneration,
    managed::streams::ChunkObserver,
    selection::{self, Plan, Request, Resolved},
    stream_records::{Frame, RecordFramer},
};
use std::{fs, num::NonZeroUsize, os::unix::fs::DirBuilderExt as _, path::Path};

/// Compiler intent from the same captured declaration/environment context as test intent.
#[derive(Debug)]
pub struct CheckIntent {
    context: Context,
    plan: Plan,
}
impl CheckIntent {
    /// Normalize only. Test filters and ignored policies are refused by the selection owner.
    pub fn new(
        metadata: cargo_metadata::Metadata,
        request: Request,
        environment: crate::environment::Environment,
    ) -> Result<Self, WorkflowError> {
        let (plan, _) = super::source::resolve(&metadata, request, String::new(), &environment)?;
        if plan.operation() != selection::Operation::Check {
            return Err(WorkflowError::Invalid(
                "check workflow requires check intent".into(),
            ));
        }
        let context = Context::capture(metadata, &plan, String::new(), environment)?;
        Ok(Self { context, plan })
    }
    /// Pure projection; compiler units remain pending until actual correctness admission.
    pub fn explain(&self) -> Explanation<'_> {
        Explanation {
            stage: Stage::Intent,
            selection: &self.plan,
            admission: pending("correctness admission not requested"),
            selected_tests: pending("compiler checks do not select a runnable test population"),
            fixture_requirements: Information::Available(&[]),
            pending: vec![
                "correctness_admission",
                "native_provider_readiness",
                "captured_control",
                "physical_settlement",
            ],
        }
    }
    /// Admit in the explicit selected checkout/environment, without compiling or preparing.
    pub fn admit(self) -> Result<AdmittedCheck, WorkflowError> {
        let resolved = self.context.admit(self.plan)?;
        Ok(AdmittedCheck {
            context: self.context,
            resolved,
        })
    }
}
/// Actual check compiler units and features, before immutable control capture or compilation.
#[derive(Debug)]
pub struct AdmittedCheck {
    context: Context,
    resolved: Resolved,
}
impl AdmittedCheck {
    pub(super) fn select_native(&mut self) -> Result<(), WorkflowError> {
        self.context.select_native(&self.resolved)
    }
    /// Original correctness-owned admission, with no invented test IDs or binaries.
    pub fn resolved(&self) -> &Resolved {
        &self.resolved
    }
    /// Explain the actual compiler stage and explicit later obligations.
    pub fn explain(&self) -> Explanation<'_> {
        Explanation {
            stage: Stage::Admitted,
            selection: self.resolved.plan(),
            admission: Information::Available(self.resolved.admitted()),
            selected_tests: pending("compiler checks do not select a runnable test population"),
            fixture_requirements: Information::Available(&[]),
            pending: vec![
                "native_provider_readiness",
                "captured_control",
                "physical_settlement",
            ],
        }
    }
    /// Explicitly retain the prepared correctness image. This copies no mutable Cargo target
    /// outputs, launches no child and grants no provider/fixture readiness or settlement.
    pub fn prepare(self, destination: &Path) -> Result<PreparedCheck, WorkflowError> {
        self.context.verify()?;
        if !destination.is_absolute() {
            return Err(WorkflowError::Invalid(
                "check capture destination must be absolute".into(),
            ));
        }
        let prepared = crate::generation::prepared(&self.context.checkout)?.ok_or_else(|| {
            WorkflowError::Invalid("native host is unprepared; run scripts/bootstrap-host".into())
        })?;
        let source = prepared.executable(
            &self.context.checkout,
            crate::generation::HostTool::Correctness,
        )?;
        fs::DirBuilder::new().mode(0o700).create(destination)?;
        let control =
            discovery::capture::executable(&source, &destination.join("cargo-pse-correctness"))?;
        self.context.verify()?;
        Ok(PreparedCheck {
            context: self.context,
            resolved: self.resolved,
            control,
            generation: prepared,
        })
    }
}
/// Captured original correctness control, independent of the original generation's lifetime.
#[derive(Debug)]
pub struct PreparedCheck {
    context: Context,
    resolved: Resolved,
    control: discovery::Executable,
    generation: PreparedGeneration,
}
impl PreparedCheck {
    /// Provenance only, not manager mutation authority or a child-settlement assertion.
    pub fn generation(&self) -> &PreparedGeneration {
        &self.generation
    }
    /// Construct one exact admitted check command. The native correctness adapter remains
    /// force-validation authority; provider requirements remain pending their actual owner.
    pub fn command(&self, group: usize) -> Result<GroupCommand, WorkflowError> {
        self.context.verify()?;
        self.control.verify()?;
        let admitted = self
            .resolved
            .admitted()
            .get(group)
            .ok_or_else(|| WorkflowError::Invalid("admitted check group is absent".into()))?;
        if !admitted
            .native_capabilities
            .is_subset(&self.context.selected_native)
        {
            return Err(WorkflowError::NativePending(
                admitted.native_capabilities.clone(),
            ));
        }
        let mut environment =
            correctness::standalone_adapter_environment(&self.context.environment)?;
        environment.insert("PSE_CORRECTNESS_ADAPTER".into(), "1".into());
        environment.insert(
            "PSE_CORRECTNESS_EXECUTABLE".into(),
            self.control.captured.as_os_str().to_owned(),
        );
        let mut arguments = admitted.arguments.clone();
        arguments.push("--keep-going".into());
        arguments.push("--message-format=json".into());
        Ok(GroupCommand {
            program: self.control.captured.clone(),
            arguments,
            cwd: self.context.checkout.clone(),
            environment,
            group,
            runtime_inputs: discovery::RuntimeInputObservation::NotCapturedByDiscovery,
        })
    }
}
/// A selected diagnostic address. Original rendered context and all duplicates stay in raw logs.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
pub struct CompilerFailureReference {
    /// Exact opaque Cargo package identity.
    pub package_id: String,
    /// Native compiler target name.
    pub target: String,
    /// Original coded diagnostic, when supplied.
    pub code: Option<String>,
    /// Complete primary native locations, without guessed panic coordinates.
    pub primary: Vec<CompilerSpan>,
    /// Native message after selected presentation redaction.
    pub message: String,
    /// Exact original record location in retained stdout.
    pub origin: RecordOrigin,
    /// Optional grouping fact only; original repeated records are never discarded.
    pub duplicate: bool,
}
/// Owner-constructed compiler result plus an explicitly budgeted failure projection.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct CompilerObservation {
    /// Native compiler reconciliation, independent of resource settlement.
    pub summary: CargoSummary,
    /// Whole diagnostic references selected under the caller's record budget.
    pub failures: Vec<CompilerFailureReference>,
    /// Explicit requested failure-record budget, never a population or raw-log cap.
    pub failure_budget: usize,
    /// Original failure occurrences outside that view; raw custody still includes them all.
    pub omitted_failure_records: usize,
}
/// Required framing/decode error, retaining original native cause and byte attribution.
#[derive(Debug, thiserror::Error)]
#[error("compiler observation at {origin:?}: {error}")]
pub struct CompilerObservationError {
    /// Original raw stdout record/fragment address.
    pub origin: RecordOrigin,
    /// Original strict decode or framing failure.
    #[source]
    pub error: ExtractionError,
}
pse_diagnostics::impl_diagnostic! {
    CompilerObservationError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure) },
    forward(this) { Some(&this.error) }, help(_this) { None }, related(_this) { None }, source(_this) { None },
    facts(this) { let mut facts = pse_diagnostics::DiagnosticFacts::default(); facts.observe("stdout_byte_offset", pse_diagnostics::DiagnosticObservation::Text(this.origin.byte_offset.to_string())); facts.observe("stdout_byte_length", pse_diagnostics::DiagnosticObservation::Text(this.origin.byte_length.to_string())); facts }
}
/// Concrete Cargo interpreter over independently retained pipes; no process lifecycle lives here.
#[derive(Debug)]
pub struct CompilerObserver {
    framer: RecordFramer,
    extractor: CargoExtractor,
    policy: RedactionPolicy,
    next_offset: u64,
    frame_offset: u64,
    ordinal: u64,
    failure_budget: usize,
    failures: Vec<CompilerFailureReference>,
    omitted: usize,
}
impl CompilerObserver {
    /// Bounds apply to one decode, optional duplicate grouping and requested view records only.
    /// The supervisor's original raw retention is independent of every one of these bounds.
    pub fn new(
        record_limit: NonZeroUsize,
        grouping_capacity: NonZeroUsize,
        failure_budget: usize,
        policy: RedactionPolicy,
    ) -> Self {
        Self {
            framer: RecordFramer::new(record_limit),
            extractor: CargoExtractor::new(grouping_capacity),
            policy,
            next_offset: 0,
            frame_offset: 0,
            ordinal: 0,
            failure_budget,
            failures: Vec::new(),
            omitted: 0,
        }
    }
    fn frames(&mut self, bytes: &[u8], eof: bool) -> Option<CompilerObservationError> {
        let Self {
            framer,
            extractor,
            policy,
            frame_offset,
            ordinal,
            failure_budget,
            failures,
            omitted,
            ..
        } = self;
        let mut first_error = None;
        let mut consume = |frame: Frame<'_>| {
            let (length, terminal) = match frame {
                Frame::Complete(bytes) | Frame::Truncated(bytes) => (bytes.len() as u64, true),
                Frame::Oversized { bytes, last, .. } => (bytes.len() as u64, last),
            };
            let origin = RecordOrigin {
                stream: OutputStream::Stdout,
                byte_offset: *frame_offset,
                byte_length: length,
                ordinal: *ordinal,
            };
            let result = match frame {
                Frame::Complete(bytes) => {
                    extractor
                        .consume(bytes, origin.clone(), policy)
                        .map(|record| {
                            if let CargoRecord::Diagnostic(diagnostic) = record
                                && diagnostic.level.starts_with("error")
                            {
                                if failures.len() < *failure_budget {
                                    failures.push(CompilerFailureReference {
                                        package_id: diagnostic.package_id,
                                        target: diagnostic.target,
                                        code: diagnostic.code,
                                        primary: diagnostic.primary,
                                        message: diagnostic.message,
                                        origin: diagnostic.origin,
                                        duplicate: diagnostic.duplicate,
                                    });
                                } else {
                                    *omitted = omitted.saturating_add(1);
                                }
                            }
                        })
                }
                Frame::Oversized { first, .. } => {
                    if first {
                        extractor.mark_incomplete();
                    }
                    Err(ExtractionError::Framing(
                        "compiler record exceeds the selected decoding bound; raw custody is independent",
                    ))
                }
                Frame::Truncated(_) => {
                    extractor.mark_incomplete();
                    Err(ExtractionError::Framing(
                        "compiler stdout ended before a required record delimiter",
                    ))
                }
            };
            if let Err(error) = result
                && first_error.is_none()
            {
                first_error = Some(CompilerObservationError { origin, error });
            }
            *frame_offset = frame_offset.saturating_add(length);
            if terminal {
                *ordinal = ordinal.saturating_add(1);
            }
        };
        if eof {
            framer.finish(&mut consume);
        } else {
            framer.push(bytes, &mut consume);
        }
        first_error
    }
    /// Reconcile only from an original child status. Required retention/delivery failure cannot
    /// be erased by exit0; physical settlement remains with PayloadReport/TerminalResult.
    pub fn finish(
        mut self,
        child_exit: i32,
        required_capture_complete: bool,
    ) -> (CompilerObservation, Option<CompilerObservationError>) {
        let error = self.frames(&[], true);
        if !required_capture_complete {
            self.extractor.mark_incomplete();
        }
        (
            CompilerObservation {
                summary: self.extractor.finish(child_exit),
                failures: self.failures,
                failure_budget: self.failure_budget,
                omitted_failure_records: self.omitted,
            },
            error,
        )
    }
}
impl ChunkObserver for CompilerObserver {
    type Error = CompilerObservationError;
    fn observe(
        &mut self,
        stream: OutputStream,
        byte_offset: u64,
        bytes: &[u8],
    ) -> Result<(), Self::Error> {
        if stream == OutputStream::Stderr {
            return Ok(());
        }
        if byte_offset != self.next_offset {
            self.extractor.mark_incomplete();
            return Err(CompilerObservationError {
                origin: RecordOrigin {
                    stream,
                    byte_offset,
                    byte_length: bytes.len() as u64,
                    ordinal: self.ordinal,
                },
                error: ExtractionError::Framing("compiler stdout byte delivery is discontinuous"),
            });
        }
        self.next_offset = self.next_offset.saturating_add(bytes.len() as u64);
        self.frames(bytes, false).map_or(Ok(()), Err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        extraction::CompilerOutcome,
        selection::{Features, Filters, IgnoredPolicy, PackageIntent, Scope, TargetIntent},
    };
    fn fixture() -> (tempfile::TempDir, CheckIntent) {
        let directory = tempfile::tempdir().unwrap();
        fs::create_dir(directory.path().join("src")).unwrap();
        fs::write(
            directory.path().join("Cargo.toml"),
            "[package]\nname=\"check-case\"\nversion=\"0.0.0\"\nedition=\"2024\"\n",
        )
        .unwrap();
        fs::write(
            directory.path().join("src/lib.rs"),
            "pub fn case() { let _: u32 = \"type error\"; }\n",
        )
        .unwrap();
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap();
        fs::copy(
            root.join("rust-toolchain.toml"),
            directory.path().join("rust-toolchain.toml"),
        )
        .unwrap();
        let generated = std::process::Command::new(correctness::real_cargo().unwrap())
            .current_dir(directory.path())
            .args(["generate-lockfile", "--offline"])
            .output()
            .unwrap();
        assert!(generated.status.success());
        let metadata = cargo_metadata::MetadataCommand::new()
            .current_dir(directory.path())
            .no_deps()
            .exec()
            .unwrap();
        let request = Request {
            operation: selection::Operation::Check,
            scope: Scope::Packages(vec![PackageIntent {
                package: "check-case".into(),
                target: Some(TargetIntent {
                    name: "check_case".into(),
                    kind: Some("lib".into()),
                }),
                features: Features::default(),
            }]),
            platform: None,
            filters: Filters::default(),
            ignored: IgnoredPolicy::Default,
        };
        let intent = CheckIntent::new(metadata, request, std::env::vars_os().collect()).unwrap();
        (directory, intent)
    }
    fn framed(
        bytes: &[u8],
        budget: usize,
    ) -> (CompilerObservation, Option<CompilerObservationError>) {
        let mut observer = CompilerObserver::new(
            NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
            NonZeroUsize::new(32).unwrap(),
            budget,
            RedactionPolicy::default(),
        );
        let mut offset = 0;
        for chunk in bytes.chunks(13) {
            let _ = observer.observe(OutputStream::Stdout, offset, chunk);
            offset += chunk.len() as u64;
        }
        observer.finish(0, true)
    }
    #[test]
    fn check_explain_and_changed_input_refusal_remain_before_compilation() {
        let (directory, intent) = fixture();
        assert!(matches!(
            intent.explain().admission,
            Information::Unavailable { .. }
        ));
        fs::write(directory.path().join("Cargo.toml"), "changed declaration").unwrap();
        assert!(matches!(intent.admit(), Err(WorkflowError::Changed(_))));
        assert!(!directory.path().join("target/debug/deps").exists());
    }
    #[test]
    fn compiler_observer_never_promotes_missing_terminal_or_lost_required_evidence() {
        let success = b"{\"reason\":\"build-finished\",\"success\":true}\n";
        assert_eq!(
            framed(b"", 0).0.summary.outcome(),
            CompilerOutcome::Incomplete
        );
        assert_eq!(
            framed(success, 0).0.summary.outcome(),
            CompilerOutcome::Succeeded
        );
        assert_eq!(
            framed(&success[..success.len() - 1], 0).0.summary.outcome(),
            CompilerOutcome::Incomplete
        );
        let mut observer = CompilerObserver::new(
            NonZeroUsize::new(4096).unwrap(),
            NonZeroUsize::new(1).unwrap(),
            0,
            RedactionPolicy::default(),
        );
        observer.observe(OutputStream::Stdout, 0, success).unwrap();
        assert_eq!(
            observer.finish(0, false).0.summary.outcome(),
            CompilerOutcome::Incomplete
        );
    }
    #[test]
    fn actual_captured_correctness_command_retains_original_compiler_failure_address() {
        let (directory, intent) = fixture();
        let admitted = intent.admit().unwrap();
        assert!(admitted.resolved().native_capabilities().is_empty());
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap();
        let generation = crate::generation::prepared(root).unwrap().unwrap();
        let retained = directory.path().join("control");
        fs::DirBuilder::new().mode(0o700).create(&retained).unwrap();
        // The fixture uses the actual prepared host's control bytes without bootstrapping a
        // second host in its tiny source checkout. Public capture/lease behavior is separate.
        let control = discovery::capture::executable(
            &root
                .join("target/host-tools")
                .join(&generation.generation)
                .join("cargo-pse-correctness"),
            &retained.join("cargo-pse-correctness"),
        )
        .unwrap();
        let mut ready = PreparedCheck {
            context: admitted.context,
            resolved: admitted.resolved,
            control,
            generation,
        };
        for key in [
            "CARGO_HACK_CARGO_SRC",
            "PSE_CORRECTNESS_BUILD_CAPTURE",
            "PSE_FIXTURE_CARGO_METADATA_CAPTURE",
        ] {
            ready
                .context
                .environment
                .insert(key.into(), "another invocation's callback".into());
        }
        let request = ready.command(0).unwrap();
        for key in [
            "CARGO_HACK_CARGO_SRC",
            "PSE_CORRECTNESS_BUILD_CAPTURE",
            "PSE_FIXTURE_CARGO_METADATA_CAPTURE",
        ] {
            assert!(!request.environment.contains_key(std::ffi::OsStr::new(key)));
        }
        assert!(
            request
                .arguments
                .iter()
                .any(|arg| arg == "--message-format=json")
        );
        assert!(
            request
                .arguments
                .iter()
                .all(|arg| !arg.to_string_lossy().contains("force_validate")),
            "pure scratch closure has no Arrow target"
        );
        let output = std::process::Command::new(&request.program)
            .args(&request.arguments)
            .current_dir(&request.cwd)
            .env_clear()
            .envs(&request.environment)
            .output()
            .unwrap();
        assert!(
            !output.status.success(),
            "the authored fixture must fail compilation"
        );
        let mut observer = CompilerObserver::new(
            NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
            NonZeroUsize::new(32).unwrap(),
            1,
            RedactionPolicy::default(),
        );
        observer
            .observe(OutputStream::Stdout, 0, &output.stdout)
            .unwrap();
        observer
            .observe(OutputStream::Stderr, 0, &output.stderr)
            .unwrap();
        let (observation, eof) = observer.finish(output.status.code().unwrap(), true);
        assert!(eof.is_none());
        assert_eq!(observation.summary.outcome(), CompilerOutcome::Failed);
        assert!(observation.summary.complete());
        assert_eq!(observation.summary.unique_primary_failures(), Some(1));
        assert_eq!(observation.failures.len(), 1);
        assert_eq!(observation.failures[0].code.as_deref(), Some("E0308"));
        assert_eq!(observation.failures[0].primary[0].line_start, 1);
        let origin = &observation.failures[0].origin;
        let raw: serde_json::Value = serde_json::from_slice(
            &output.stdout
                [origin.byte_offset as usize..(origin.byte_offset + origin.byte_length) as usize],
        )
        .unwrap();
        assert_eq!(raw["message"]["code"]["code"], "E0308");
        let (zero, _) = framed(&output.stdout, 0);
        assert_eq!(zero.summary.outcome(), CompilerOutcome::Failed);
        assert!(zero.summary.complete());
        assert_eq!(
            zero.summary.error_records(),
            observation.summary.error_records()
        );
        assert!(zero.failures.is_empty());
        assert_eq!(zero.omitted_failure_records, zero.summary.error_records());
    }
}
