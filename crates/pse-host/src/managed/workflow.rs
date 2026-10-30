// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Fixed native workflow input and required producer protocol, separate from physical drain.
use super::{payload::PayloadCommand, supervisor};
use crate::{
    HostError,
    envelope::CliInvocationId,
    generation::{HostTool, PreparedGeneration},
    invocation::{InvocationDirectory, InvocationEvidence},
    stream_records::{Frame, RecordFramer},
    test_workflow::{
        WorkflowError,
        execute::{Action, Input, TerminalRecord},
    },
};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use sha2::{Digest as _, Sha256};
use std::{
    ffi::OsString,
    fs::{self, OpenOptions},
    io::{self, Read},
    num::NonZeroUsize,
    os::unix::fs::{MetadataExt as _, OpenOptionsExt as _},
    path::{Path, PathBuf},
    sync::Arc,
};

/// The original fixed helper or its required transport failed.
#[derive(Debug, thiserror::Error)]
pub enum WorkflowOwnerError {
    /// Original local evidence or pipe failure.
    #[error("managed workflow I/O: {0}")]
    Io(#[from] io::Error),
    /// Original strict input/terminal/report decode failure.
    #[error("managed workflow JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// Original native ownership/identity failure.
    #[error(transparent)]
    Host(#[from] HostError),
    /// Original operation-owned construction or execution failure.
    #[error(transparent)]
    Operation(#[from] WorkflowError),
    /// A checked fixed input, frame or original artifact association failed.
    #[error("managed workflow: {0}")]
    Invalid(String),
}
pse_diagnostics::impl_diagnostic! {
    WorkflowOwnerError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure) },
    forward(this) { match this { Self::Host(error)=>Some(error), Self::Operation(error)=>Some(error), _=>None } },
    help(_this) { None }, related(_this) { None }, source(_this) { None }
}
fn invalid(message: impl Into<String>) -> WorkflowOwnerError {
    WorkflowOwnerError::Invalid(message.into())
}
fn host(error: impl Into<WorkflowOwnerError>) -> HostError {
    HostError::WorkflowOwner(Box::new(error.into()))
}
fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Metadata points at one authoritative encoded private input; it does not repeat selection.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct InputBinding {
    /// Original exact encoded input extent.
    bytes: u64,
    /// Original exact encoded input identity.
    sha256: String,
}
#[derive(Deserialize)]
struct Metadata {
    workflow: Option<InputBinding>,
}
/// Serialize the input once; RawValue publication preserves exactly these bytes.
pub(crate) fn encode_input(input: &Input) -> Result<(InputBinding, Box<RawValue>), HostError> {
    let value = serde_json::value::to_raw_value(input).map_err(host)?;
    if value.get().len() > 1024 * 1024 {
        return Err(host(invalid(
            "private input exceeds its 1 MiB protocol extent",
        )));
    }
    Ok((
        InputBinding {
            bytes: value.get().len() as u64,
            sha256: digest(value.get().as_bytes()),
        },
        value,
    ))
}
/// Only this fixed prepared executable/argument vector can produce a workflow verdict.
pub(crate) fn command(
    root: &Path,
    id: &CliInvocationId,
    control: &PreparedGeneration,
    environment: crate::environment::Environment,
) -> Result<PayloadCommand, HostError> {
    Ok(PayloadCommand {
        program: control.executable(root, HostTool::Host)?,
        arguments: arguments(root, id),
        cwd: root.to_owned(),
        environment,
    })
}
fn arguments(root: &Path, id: &CliInvocationId) -> Vec<OsString> {
    vec![
        "__workflow".into(),
        "--checkout-bytes".into(),
        super::request::encode_path(root).into(),
        "--invocation".into(),
        id.as_str().into(),
    ]
}
fn input(
    directory: &InvocationDirectory,
    binding: &InputBinding,
    id: &CliInvocationId,
    command: &str,
) -> Result<Input, HostError> {
    let value = directory
        .evidence::<Box<RawValue>>(InvocationEvidence::WorkflowInput)?
        .ok_or_else(|| host(invalid("original workflow input is absent")))?;
    if binding.bytes != value.get().len() as u64 || binding.sha256 != digest(value.get().as_bytes())
    {
        return Err(host(invalid(
            "original workflow input byte identity differs from metadata",
        )));
    }
    let input: Input = serde_json::from_str(value.get()).map_err(host)?;
    if input.version != 1
        || input.invocation != *id
        || input.action.command() != command
        || input.bounds.record_bytes == 0
        || input.bounds.grouping_capacity == 0
        || input.policy.write.is_zero()
        || input.policy.drain.is_zero()
    {
        return Err(host(invalid(
            "workflow input version, correlation, action or observation policy is invalid",
        )));
    }
    Ok(input)
}
/// Internal entry: authenticate the actual original payload leader before any compilation.
pub(crate) async fn run(root: PathBuf, id: CliInvocationId) -> Result<i32, HostError> {
    let environment = std::env::vars_os().collect();
    let leader: supervisor::PayloadLeader = supervisor::current_payload(&environment).await?;
    if leader.root() != root.canonicalize()? || leader.invocation_id() != &id {
        return Err(host(invalid(
            "workflow entry differs from authenticated native payload",
        )));
    }
    let metadata = leader.metadata::<Metadata>()?;
    let binding = metadata
        .selection
        .operation
        .workflow
        .ok_or_else(|| host(invalid("original workflow input binding is absent")))?;
    let input = input(
        leader.directory(),
        &binding,
        &id,
        &metadata.selection.command,
    )?;
    let destination = leader.directory().path().join("workflow");
    let report = crate::test_workflow::execute::run(&input, &root, &destination, environment)
        .await
        .map_err(host)?;
    let terminal = report.publish(&destination).map_err(host)?;
    terminal.validate(&id, input.action).map_err(host)?;
    let status = terminal.helper_exit();
    terminal.write(&mut io::stdout().lock()).map_err(host)?;
    Ok(status)
}

pub(super) enum Observer {
    Raw,
    Workflow(WorkflowObserver),
}
pub(super) struct WorkflowObserver {
    invocation: CliInvocationId,
    action: Action,
    framer: RecordFramer,
    next_offset: u64,
    records: u64,
    terminal: Option<TerminalRecord>,
    invalid: bool,
}
impl Observer {
    pub(super) fn for_payload(
        context: &supervisor::Context,
        root: &Path,
        command: &PayloadCommand,
    ) -> Result<Self, HostError> {
        let Some(action) = Action::from_command(&context.record.command) else {
            return Ok(Self::Raw);
        };
        let metadata = context.store.metadata::<Metadata>(context.directory.id())?;
        if command.program
            != metadata
                .selection
                .control
                .executable(root, HostTool::Host)?
            || command.arguments != arguments(root, context.directory.id())
        {
            return Err(host(invalid(
                "workflow payload is not the original fixed helper",
            )));
        }
        let binding = metadata
            .selection
            .operation
            .workflow
            .ok_or_else(|| host(invalid("workflow operation has no original input binding")))?;
        let input = input(
            &context.directory,
            &binding,
            context.directory.id(),
            &context.record.command,
        )?;
        Ok(Self::Workflow(WorkflowObserver {
            invocation: input.invocation,
            action,
            framer: RecordFramer::new(
                NonZeroUsize::new(input.bounds.record_bytes)
                    .ok_or_else(|| host(invalid("zero terminal frame extent")))?,
            ),
            next_offset: 0,
            records: 0,
            terminal: None,
            invalid: false,
        }))
    }
    pub(super) fn finish(self) -> Result<Option<ObservedTerminal>, HostError> {
        let Self::Workflow(mut observer) = self else {
            return Ok(None);
        };
        observer.framer.finish(|_| observer.invalid = true);
        if observer.invalid || observer.records != 1 {
            return Err(host(invalid(
                "required workflow terminal frame is missing, malformed, duplicated or truncated",
            )));
        }
        let terminal = observer
            .terminal
            .ok_or_else(|| host(invalid("required workflow terminal is absent")))?;
        terminal
            .validate(&observer.invocation, observer.action)
            .map_err(host)?;
        Ok(Some(ObservedTerminal { terminal }))
    }
}
impl super::streams::ChunkObserver for Observer {
    type Error = HostError;
    fn observe(
        &mut self,
        stream: crate::extraction::OutputStream,
        offset: u64,
        bytes: &[u8],
    ) -> Result<(), HostError> {
        let Self::Workflow(observer) = self else {
            return Ok(());
        };
        if stream == crate::extraction::OutputStream::Stderr {
            return Ok(());
        }
        if offset != observer.next_offset {
            observer.invalid = true;
            return Err(host(invalid(
                "workflow terminal byte delivery is discontinuous",
            )));
        }
        observer.next_offset = offset.saturating_add(bytes.len() as u64);
        let mut first = None;
        observer.framer.push(bytes, |frame| {
            let result = match frame {
                Frame::Complete(bytes) => {
                    observer.records = observer.records.saturating_add(1);
                    if observer.records != 1 {
                        Err(host(invalid(
                            "workflow emitted more than one required terminal record",
                        )))
                    } else {
                        match serde_json::from_slice::<TerminalRecord>(bytes) {
                            Ok(terminal) => {
                                match terminal.validate(&observer.invocation, observer.action) {
                                    Ok(()) => {
                                        observer.terminal = Some(terminal);
                                        Ok(())
                                    }
                                    Err(error) => Err(host(error)),
                                }
                            }
                            Err(error) => Err(host(error)),
                        }
                    }
                }
                Frame::Oversized { .. } | Frame::Truncated(_) => Err(host(invalid(
                    "workflow terminal record exceeds framing policy or is incomplete",
                ))),
            };
            if let Err(error) = result {
                observer.invalid = true;
                if first.is_none() {
                    first = Some(error);
                }
            }
        });
        first.map_or(Ok(()), Err)
    }
}

/// Constructed only after original helper transport and durable report bytes are checked.
pub(super) struct Validated {
    terminal: TerminalRecord,
    path: PathBuf,
}
impl Validated {
    pub(super) fn into_parts(self) -> (TerminalRecord, PathBuf) {
        (self.terminal, self.path)
    }
}
/// Only the fixed-helper observer can construct this; decoding transport data grants no proof.
pub(super) struct ObservedTerminal {
    terminal: TerminalRecord,
}
pub(super) async fn validate_report(
    directory: Arc<InvocationDirectory>,
    observed: ObservedTerminal,
    child_exit: Option<i32>,
) -> Result<Validated, HostError> {
    let terminal = observed.terminal;
    if child_exit != Some(terminal.helper_exit()) {
        return Err(host(invalid(
            "workflow terminal and original waited helper status disagree",
        )));
    }
    tokio::task::spawn_blocking(move || {
        let parent = directory.path().join("workflow");
        let metadata = fs::symlink_metadata(&parent)?;
        if !metadata.is_dir()
            || metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.mode() & 0o777 != 0o700
        {
            return Err(host(invalid(
                "workflow report directory is not original private evidence",
            )));
        }
        let path = parent.join(&terminal.report.path);
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
            .open(&path)?;
        let metadata = file.metadata()?;
        if !metadata.is_file()
            || metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.mode() & 0o022 != 0
            || metadata.len() != terminal.report.bytes
        {
            return Err(host(invalid(
                "workflow report file identity/extent differs from original terminal",
            )));
        }
        let mut input = HashingReader {
            input: file,
            sha: Sha256::new(),
            bytes: 0,
        };
        {
            let mut json = serde_json::Deserializer::from_reader(io::BufReader::new(&mut input));
            serde::de::IgnoredAny::deserialize(&mut json).map_err(host)?;
            json.end().map_err(host)?;
        }
        let sha = input
            .sha
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        if input.bytes != terminal.report.bytes || sha != terminal.report.sha256 {
            return Err(host(invalid(
                "durable workflow report differs from original terminal digest",
            )));
        }
        Ok(Validated { terminal, path })
    })
    .await?
}
struct HashingReader<R> {
    input: R,
    sha: Sha256,
    bytes: u64,
}
impl<R: Read> Read for HashingReader<R> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let count = self.input.read(bytes)?;
        self.sha.update(&bytes[..count]);
        self.bytes = self.bytes.saturating_add(count as u64);
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        managed::streams::ChunkObserver as _,
        test_workflow::execute::{Outcome, ReportReference},
    };
    use std::os::unix::fs::{DirBuilderExt as _, PermissionsExt as _};
    fn observer(id: &CliInvocationId) -> Observer {
        Observer::Workflow(WorkflowObserver {
            invocation: id.clone(),
            action: Action::Test,
            framer: RecordFramer::new(NonZeroUsize::new(4096).unwrap()),
            next_offset: 0,
            records: 0,
            terminal: None,
            invalid: false,
        })
    }
    fn terminal(id: &CliInvocationId, bytes: &[u8]) -> TerminalRecord {
        TerminalRecord {
            protocol: "pse.test-workflow".into(),
            version: 1,
            invocation: id.clone(),
            action: Action::Test,
            outcome: Outcome::TestsSucceeded,
            report: ReportReference {
                path: "report.json".into(),
                sha256: digest(bytes),
                bytes: bytes.len() as u64,
            },
            required_capture_complete: true,
        }
    }
    fn observed(id: &CliInvocationId, record: &TerminalRecord) -> ObservedTerminal {
        let mut observer = observer(id);
        let mut bytes = Vec::new();
        record.write(&mut bytes).unwrap();
        observer
            .observe(crate::extraction::OutputStream::Stdout, 0, &bytes)
            .unwrap();
        observer.finish().unwrap().unwrap()
    }
    #[test]
    fn required_terminal_is_single_complete_correlated_and_never_generic_authority() {
        let id = CliInvocationId::new();
        let record = terminal(&id, b"{}");
        let mut bytes = Vec::new();
        record.write(&mut bytes).unwrap();
        let mut valid = observer(&id);
        let split = bytes.len() / 2;
        valid
            .observe(crate::extraction::OutputStream::Stdout, 0, &bytes[..split])
            .unwrap();
        valid
            .observe(
                crate::extraction::OutputStream::Stdout,
                split as u64,
                &bytes[split..],
            )
            .unwrap();
        assert!(valid.finish().unwrap().is_some());
        let mut duplicate = observer(&id);
        duplicate
            .observe(crate::extraction::OutputStream::Stdout, 0, &bytes)
            .unwrap();
        assert!(
            duplicate
                .observe(
                    crate::extraction::OutputStream::Stdout,
                    bytes.len() as u64,
                    &bytes
                )
                .is_err()
        );
        assert!(duplicate.finish().is_err());
        let mut truncated = observer(&id);
        truncated
            .observe(
                crate::extraction::OutputStream::Stdout,
                0,
                &bytes[..bytes.len() - 1],
            )
            .unwrap();
        assert!(truncated.finish().is_err());
        let mut gap = observer(&id);
        assert!(
            gap.observe(crate::extraction::OutputStream::Stdout, 1, &bytes)
                .is_err()
        );
        assert!(gap.finish().is_err());
        let mut foreign = observer(&CliInvocationId::new());
        assert!(
            foreign
                .observe(crate::extraction::OutputStream::Stdout, 0, &bytes)
                .is_err()
        );
        assert!(foreign.finish().is_err());
        let mut raw = Observer::Raw;
        raw.observe(crate::extraction::OutputStream::Stdout, 0, &bytes)
            .unwrap();
        assert!(raw.finish().unwrap().is_none());
        let mut traversal = record;
        traversal.report.path = "../outside.json".into();
        let mut bytes = Vec::new();
        traversal.write(&mut bytes).unwrap();
        let mut reader = observer(&id);
        assert!(
            reader
                .observe(crate::extraction::OutputStream::Stdout, 0, &bytes)
                .is_err()
        );
    }
    #[tokio::test]
    async fn original_report_requires_private_complete_unchanged_bytes_and_waited_status() {
        use crate::{
            invocation::{InvocationSpec, InvocationStore},
            paths::CheckoutState,
        };
        let scratch = tempfile::tempdir().unwrap();
        let env = crate::environment::Environment::from([(
            "XDG_STATE_HOME".into(),
            scratch.path().join("state").into_os_string(),
        )]);
        let state = CheckoutState::from_paths(
            &scratch.path().join("checkout"),
            &scratch.path().join("common.git"),
            &env,
        )
        .unwrap();
        let id = CliInvocationId::new();
        let store = InvocationStore::new(state);
        let directory = Arc::new(
            store
                .begin(InvocationSpec {
                    invocation_id: id.clone(),
                    command: "test".into(),
                    session: None,
                    parent: None,
                    operation: (),
                    control: PreparedGeneration {
                        generation: "generation.fixture".into(),
                        protocol: 1,
                        manifest_digest: "a".repeat(64),
                    },
                    environment_digest: "b".repeat(64),
                    scientific: vec![],
                })
                .unwrap(),
        );
        let parent = directory.path().join("workflow");
        fs::DirBuilder::new().mode(0o700).create(&parent).unwrap();
        let path = parent.join("report.json");
        let original = b"{\"status\":\"original\"}";
        fs::write(&path, original).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        let record = terminal(&id, original);
        assert!(
            validate_report(directory.clone(), observed(&id, &record), Some(0))
                .await
                .is_ok()
        );
        assert!(
            validate_report(directory.clone(), observed(&id, &record), Some(1))
                .await
                .is_err()
        );
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&path, b"{\"status\":\"tampered\"}").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        assert!(
            validate_report(directory.clone(), observed(&id, &record), Some(0))
                .await
                .is_err()
        );
        fs::remove_file(&path).unwrap();
        std::os::unix::fs::symlink(scratch.path().join("outside.json"), &path).unwrap();
        assert!(
            validate_report(directory, observed(&id, &record), Some(0))
                .await
                .is_err()
        );
    }
}
