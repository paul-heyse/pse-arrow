// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Private store-free execution placement for a closed authored implicit-root factory.
//! One child owns one fresh serial KINSOL solve. Parent admission and original screening
//! remain authoritative; this is neither continuation nor distributed decomposition.
use super::{MathRuntimeError, MathService, Submission, solves::SolveHandle};
use pse_columnar::AllocationLease;
use pse_compiler::{
    typed_math::{AdmittedBody, portable_payload_hash, reconstruct_portable},
    workspace::CompilerContext,
};
use pse_diagnostics::Cancellation;
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_kernels::{DerivativeOrder, ExecutionScope};
use pse_math::{
    implicit::{InnerSolver, Options, Problem, Unknown},
    jets::EvaluationLimits,
};
use pse_model::HeapUsage;
use pse_model::diagnostic::{BoundaryDiagnostic, DiagnosticProjection};
use serde::{Deserialize, Serialize};
use std::{
    io::{self, Read, Write},
    os::{fd::AsFd, unix::net::UnixStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::Arc,
    time::{Duration, Instant},
};

const PROTOCOL: &str = "pse.private-native-root.v1";
const REQUEST_LIMIT: usize = 16 << 20;
const RESPONSE_LIMIT: usize = 1 << 20;
/// Explicit child native-owner stack, charged to its parent's finite worker allocation.
pub const NATIVE_CHILD_STACK_BYTES: usize = pse_structural::incidence::MATCHING_STACK;

fn contract(message: impl Into<String>) -> MathRuntimeError {
    pse_backend_native::ProblemError::Contract(message.into()).into()
}
fn infrastructure(error: impl std::fmt::Display) -> MathRuntimeError {
    MathRuntimeError::Infrastructure(format!("native child: {error}"))
}
fn checkpoint(scope: &ExecutionScope) -> Result<(), MathRuntimeError> {
    scope
        .check()
        .map_err(pse_math::MathError::Scope)
        .map_err(Into::into)
}
fn digest(bytes: &[u8]) -> ContentHash {
    let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
    h.str(PROTOCOL).part(bytes);
    h.finish_hash()
}
fn native_profile() -> Result<ContentHash, MathRuntimeError> {
    pse_backend_native::execution::adapter(pse_backend_native::solve::Backend::Kinsol)
        .build()
        .ok_or_else(|| {
            pse_backend_native::ProblemError::Unsupported(
                "native child requires available bundled-SUNDIALS/KLU profile evidence".into(),
            )
            .into()
        })
}

/// Actual closed source for the existing authored implicit-root mechanism.
/// Arbitrary provider descriptors cannot supply executable factory reconstruction.
#[derive(Clone, Debug)]
pub struct PortableRootSource {
    body: Arc<AdmittedBody>,
    context: CompilerContext,
    unknowns: Vec<Unknown>,
    rows: Vec<SemanticId>,
    parameters: Vec<f64>,
    options: Options,
    evaluation: EvaluationLimits,
    max_entries: usize,
}
impl PortableRootSource {
    /// Bind actual admitted syntax, physical declarations and original-coordinate controls.
    /// Full PreparedSolve/opaque provider closures are deliberately unsupported.
    #[expect(
        clippy::too_many_arguments,
        reason = "this closed factory binds the existing complete implicit problem and its compiler/physical source"
    )]
    pub fn new(
        body: Arc<AdmittedBody>,
        context: CompilerContext,
        unknowns: Vec<Unknown>,
        rows: Vec<SemanticId>,
        parameters: Vec<f64>,
        options: Options,
        evaluation: EvaluationLimits,
        max_entries: usize,
    ) -> Result<Self, MathRuntimeError> {
        if !body.math().providers().is_empty()
            || !body.spec().providers.is_empty()
            || !context.providers.is_empty()
        {
            return Err(contract(
                "native child requires a closed provider-free executable factory",
            ));
        }
        if body.spec().physical
            != pse_compiler::workspace::physical_identity(
                &context.quantities,
                &context.preconditions,
            )
            || body.math().input_count() != unknowns.len() + parameters.len()
            || body.math().output_count() != rows.len()
            || rows.len() != unknowns.len()
            || unknowns.is_empty()
            || max_entries == 0
            || unknowns
                .iter()
                .any(|u| !u.lower.is_finite() || !u.upper.is_finite() || u.lower > u.upper)
            || parameters.iter().any(|value| !value.is_finite())
            || options.iterations == 0
            || options.time_limit.is_zero()
        {
            return Err(contract(
                "native child source physical context, extent or finite interval differs",
            ));
        }
        body.portable_payload_bytes()?;
        Ok(Self {
            body,
            context,
            unknowns,
            rows,
            parameters,
            options,
            evaluation,
            max_entries,
        })
    }
    fn wire(&self) -> Result<RootWire, MathRuntimeError> {
        let registry =
            serde_json::to_vec(&self.context.quantities.to_builder()).map_err(infrastructure)?;
        Ok(RootWire {
            body: self.body.portable_payload()?,
            spec: self.body.spec().clone(),
            registry_digest: digest(&registry),
            registry,
            unknowns: self.unknowns.clone(),
            rows: self.rows.clone(),
            parameters: self.parameters.clone(),
            options: self.options.clone(),
            evaluation: self.evaluation,
            max_entries: self.max_entries,
        })
    }
    fn problem(&self, scope: &ExecutionScope) -> Result<Arc<Problem>, MathRuntimeError> {
        compile_problem(
            &self.body,
            &self.unknowns,
            &self.rows,
            self.parameters.len(),
            self.evaluation,
            self.max_entries,
            scope,
        )
    }
}

/// Explicit private executable and finite opaque-work allowance.
/// The launcher observes this exact file again before spawn and verifies the child's
/// running executable, native version and compiled product during its handshake.
#[derive(Clone, Debug)]
pub struct NativeChild {
    executable: PathBuf,
    observation: pse_buildinfo::FileObservation,
    /// Separately charged allowance for SUNDIALS/KLU/private allocator, registry and
    /// compiled-program populations and startup/main/foreign stack unknowns. This is
    /// admission accounting, not an allocator-enforced complete process/RSS maximum.
    opaque_bytes: usize,
    /// Explicit strict whole-child memory selection; currently refused before launch.
    pub require_complete_memory_bound: bool,
    #[cfg(test)]
    test_mode: Option<(String, PathBuf)>,
}
impl NativeChild {
    /// Select a trusted deployed private worker. There is no PATH search or auto-build.
    pub fn qualify(
        executable: impl AsRef<Path>,
        opaque_bytes: usize,
    ) -> Result<Self, MathRuntimeError> {
        if opaque_bytes == 0 {
            return Err(contract(
                "native child requires a finite positive opaque allowance",
            ));
        }
        native_profile()?;
        let observation =
            pse_buildinfo::FileObservation::capture(executable.as_ref()).map_err(infrastructure)?;
        Ok(Self {
            executable: observation.canonical.clone(),
            observation,
            opaque_bytes,
            require_complete_memory_bound: false,
            #[cfg(test)]
            test_mode: None,
        })
    }
}

/// Completion-owned child proposal. The parent has checked original physical residuals
/// and intervals; enclosing scientific consumers retain their normal final acceptance.
#[derive(Debug)]
pub struct PortableRootResult {
    /// Original-coordinate proposal, absent after an attributed child refusal.
    pub proposal: Option<Vec<f64>>,
    /// Complete typed native/evaluation diagnostic, never reclassified from error text.
    pub refusal: Option<BoundaryDiagnostic>,
    /// Actual reaped child process identity.
    pub child_pid: u32,
    /// Attempt identity echoed only over the private inherited channel.
    pub attempt: SemanticId,
    /// Actual executable byte identity.
    pub executable_sha256: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RootWire {
    body: Vec<u8>,
    spec: pse_math::binding::BodySpec,
    registry: Vec<u8>,
    registry_digest: ContentHash,
    unknowns: Vec<Unknown>,
    rows: Vec<SemanticId>,
    parameters: Vec<f64>,
    options: Options,
    evaluation: EvaluationLimits,
    max_entries: usize,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Hello {
    diagnostic_operation: pse_diagnostics::observation::OperationId,
    protocol: String,
    attempt: SemanticId,
    pid: u32,
    parent: u32,
    executable: pse_buildinfo::FileObservation,
    build: pse_buildinfo::BuildInfo,
    native_build: ContentHash,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    diagnostic_parent: Option<pse_diagnostics::observation::OperationId>,
    attempt: SemanticId,
    execution: ContentHash,
    request: ContentHash,
    /// Remaining parent allowance sent after startup; parent retains its original clock.
    remaining: Duration,
    source: Vec<u8>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Response {
    attempt: SemanticId,
    execution: ContentHash,
    request: ContentHash,
    result: Result<Vec<f64>, BoundaryDiagnostic>,
}

impl MathService {
    /// Execute a complete portable authored root on one independent serial process lane.
    /// Queue, startup, IPC, native work, original screen and actual reap share one deadline.
    pub fn solve_portable_root(
        self: &Arc<Self>,
        source: PortableRootSource,
        child: NativeChild,
        cancel: Cancellation,
        original_deadline: Option<Instant>,
    ) -> Result<SolveHandle<(PortableRootResult, Arc<AllocationLease>)>, MathRuntimeError> {
        let deadline = Instant::now()
            .checked_add(source.options.time_limit)
            .ok_or_else(|| contract("native child deadline extent"))?;
        let deadline = original_deadline.map_or(deadline, |outer| outer.min(deadline));
        let scope = ExecutionScope::new(cancel.clone(), Some(deadline));
        checkpoint(&scope)?;
        if child.require_complete_memory_bound {
            return Err(pse_backend_native::ProblemError::Unsupported(
                "strict complete child memory/RSS bounds are not qualified for SUNDIALS/KLU/private allocations".into()).into());
        }
        // The existing submission owner holds this finite opaque-work allowance and
        // population/CPU admission through kill/wait and original screening. Known
        // buffers and the explicit native-owner stack have a separate shared-pool lease.
        let submission = Submission {
            cancel,
            progress: Arc::new(pse_backend_native::solve::Progress::new(0)),
            admitted: None,
            deadline: Some(deadline),
        };
        let bytes = child.opaque_bytes;
        let recipe_extent = source.body.portable_payload_bytes()?;
        let replay_extent =
            pse_ids::scientific_replay::QualifiedScientificRecipe::scratch_bytes(recipe_extent)
                .map_err(infrastructure)?;
        let known_bytes = replay_extent
            .checked_mul(2)
            .and_then(|n| n.checked_add(4 * REQUEST_LIMIT + 2 * RESPONSE_LIMIT))
            .and_then(|n| n.checked_add(NATIVE_CHILD_STACK_BYTES))
            .ok_or_else(|| contract("native child construction extent"))?;
        let service = self.clone();
        self.submit_source_timed(1, bytes, submission, move |flag, _| {
            let scope = ExecutionScope::new(flag, Some(deadline));
            checkpoint(&scope)?;
            // Immediate pool refusal cannot create admission hold-and-wait. This lease
            // remains live across every child exit/drain and all original evaluation.
            let _known = service.reserve("math:native-child-known-extents", known_bytes)?;
            let wire = source.wire()?;
            let payload = serde_json::to_vec(&wire).map_err(infrastructure)?;
            if payload.len() > REQUEST_LIMIT {
                return Err(contract("native child request frame limit"));
            }
            let original = source.problem(&scope)?;
            original.validate_options(&source.options)?;
            let result = launch(&child, payload, &scope)?;
            checkpoint(&scope)?;
            if let Some(point) = &result.proposal {
                original.verify(
                    &source.parameters,
                    point,
                    &source.options,
                    scope.cancellation(),
                )?;
                checkpoint(&scope)?;
            }
            let retained = size_of::<PortableRootResult>()
                + result.executable_sha256.capacity()
                + result
                    .proposal
                    .as_ref()
                    .map_or(0, |point| point.capacity() * size_of::<f64>())
                + result.refusal.as_ref().map_or(0, HeapUsage::heap_bytes);
            Ok((result, retained))
        })
    }
}

fn compile_problem(
    body: &AdmittedBody,
    unknowns: &[Unknown],
    rows: &[SemanticId],
    parameters: usize,
    limits: EvaluationLimits,
    max_entries: usize,
    scope: &ExecutionScope,
) -> Result<Arc<Problem>, MathRuntimeError> {
    checkpoint(scope)?;
    let source = body.math().compile(
        &(0..rows.len()).collect::<Vec<_>>(),
        &[],
        DerivativeOrder::Value,
        pse_math::library::Optimization::default(),
        limits,
        scope,
    )?;
    let unknown = body.math().compile(
        &(0..rows.len()).collect::<Vec<_>>(),
        &(0..unknowns.len()).collect::<Vec<_>>(),
        DerivativeOrder::First,
        pse_math::library::Optimization::default(),
        limits,
        scope,
    )?;
    checkpoint(scope)?;
    Ok(Arc::new(Problem::new(
        rows[0],
        body.spec().key(),
        unknowns.to_vec(),
        rows.to_vec(),
        parameters,
        pse_math::implicit::ResidualBodies {
            unknown: Arc::new(unknown),
            source: Arc::new(source),
        },
        max_entries,
    )?))
}

struct OwnedChild {
    process: Child,
    process_fd: Option<std::os::fd::OwnedFd>,
    channel: UnixStream,
    reaped: bool,
}
impl OwnedChild {
    fn reap(&mut self, scope: &ExecutionScope, abort: bool) -> Result<(), MathRuntimeError> {
        if abort {
            self.process.kill().map_err(infrastructure)?;
            self.process.wait().map_err(infrastructure)?;
            self.reaped = true;
            return Ok(());
        }
        loop {
            if let Some(status) = self.process.try_wait().map_err(infrastructure)? {
                self.reaped = true;
                return if !abort && !status.success() {
                    Err(infrastructure(format!("process exited {status}")))
                } else {
                    Ok(())
                };
            }
            let readiness = self
                .process_fd
                .as_ref()
                .ok_or_else(|| infrastructure("missing owned process descriptor"))
                .and_then(|fd| wait_ready(scope, fd, rustix::event::PollFlags::IN));
            if let Err(error) = readiness {
                let _ = self.process.kill();
                self.process.wait().map_err(infrastructure)?;
                self.reaped = true;
                return Err(error);
            }
        }
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if !self.reaped {
            let _ = self.process.kill();
            let _ = self.process.wait();
        }
    }
}
fn launch(
    child: &NativeChild,
    payload: Vec<u8>,
    scope: &ExecutionScope,
) -> Result<PortableRootResult, MathRuntimeError> {
    checkpoint(scope)?;
    let observed = pse_buildinfo::FileObservation::capture_scoped(&child.executable, &|| {
        scope.check().map_err(|e| io::Error::other(e.to_string()))
    });
    checkpoint(scope)?;
    let actual = observed.map_err(infrastructure)?;
    if actual.sha256 != child.observation.sha256 || actual.canonical != child.observation.canonical
    {
        return Err(contract(
            "native child executable changed after qualification",
        ));
    }
    let attempt: SemanticId = pse_operations::mint_id();
    let (channel, inherited) = UnixStream::pair().map_err(infrastructure)?;
    let mut command = Command::new(&child.executable);
    #[cfg(test)]
    if let Some((mode, marker)) = &child.test_mode {
        command
            .args([
                "--exact",
                "math::native_child::tests::private_native_fake_process",
                "--nocapture",
            ])
            .env("PSE_PRIVATE_NATIVE_TEST_MODE", mode)
            .env("PSE_PRIVATE_NATIVE_TEST_MARKER", marker)
            .env("PSE_PRIVATE_NATIVE_TEST_ATTEMPT", attempt.to_string());
    } else {
        command.args([
            "--private-native-root",
            &attempt.to_string(),
            &std::process::id().to_string(),
        ]);
    }
    #[cfg(not(test))]
    command.args([
        "--private-native-root",
        &attempt.to_string(),
        &std::process::id().to_string(),
    ]);
    let process = command
        .stdin(Stdio::from(std::os::fd::OwnedFd::from(inherited)))
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .env("OMP_NUM_THREADS", "1")
        .env("OPENBLAS_NUM_THREADS", "1")
        .env("MKL_NUM_THREADS", "1")
        .env("RAYON_NUM_THREADS", "1")
        .spawn()
        .map_err(infrastructure)?;
    let mut owned = OwnedChild {
        process,
        process_fd: None,
        channel,
        reaped: false,
    };
    owned.process_fd = Some(owned_process_fd(owned.process.id())?);
    owned
        .channel
        .set_nonblocking(true)
        .map_err(infrastructure)?;
    let result = (|| {
        let hello: Hello = receive(&mut owned.channel, RESPONSE_LIMIT, scope)?;
        if hello.protocol != PROTOCOL
            || hello.attempt != attempt
            || hello.pid != owned.process.id()
            || hello.parent != std::process::id()
            || hello.executable.sha256 != child.observation.sha256
            || hello.executable.canonical != child.executable
            || hello.native_build != native_profile()?
            || serde_json::to_vec(&hello.build).map_err(infrastructure)?
                != serde_json::to_vec(&pse_buildinfo::BuildInfo::captured())
                    .map_err(infrastructure)?
        {
            return Err(contract(
                "native child executable/build/profile/attempt handshake differs",
            ));
        }
        if let Some(context) = pse_diagnostics::observation::current() {
            context.link_id(
                hello.diagnostic_operation,
                pse_diagnostics::observation::Relationship::Continuation,
            );
        }
        let request = digest(&payload);
        let remaining = scope
            .remaining(Duration::MAX)
            .map_err(pse_math::MathError::Scope)?;
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
        h.str(PROTOCOL)
            .id(&attempt)
            .hash(&request)
            .str(&hello.executable.sha256)
            .part(&serde_json::to_vec(&hello.native_build).map_err(infrastructure)?)
            .u64(remaining.as_secs())
            .u64(u64::from(remaining.subsec_nanos()));
        let execution = h.finish_hash();
        let envelope = Request {
            diagnostic_parent: pse_diagnostics::observation::current().map(|context| context.id()),
            attempt,
            execution,
            request,
            remaining,
            source: payload,
        };
        send(&mut owned.channel, &envelope, REQUEST_LIMIT, scope)?;
        let response: Response = receive(&mut owned.channel, RESPONSE_LIMIT, scope)?;
        if response.attempt != attempt
            || response.execution != execution
            || response.request != request
        {
            return Err(contract(
                "native child response belongs to another attempt/profile",
            ));
        }
        let (proposal, refusal) = match response.result {
            Ok(point) => (Some(point), None),
            Err(cause) => (None, Some(cause)),
        };
        Ok(PortableRootResult {
            proposal,
            refusal,
            child_pid: hello.pid,
            attempt,
            executable_sha256: hello.executable.sha256,
        })
    })();
    let drained = owned.reap(scope, result.is_err());
    checkpoint(scope)?;
    let result = result?;
    drained?;
    Ok(result)
}

#[cfg(target_os = "linux")]
fn owned_process_fd(pid: u32) -> Result<std::os::fd::OwnedFd, MathRuntimeError> {
    let pid = rustix::process::Pid::from_raw(pid.try_into().map_err(infrastructure)?)
        .ok_or_else(|| infrastructure("invalid owned child PID"))?;
    rustix::process::pidfd_open(pid, rustix::process::PidfdFlags::empty()).map_err(infrastructure)
}
#[cfg(not(target_os = "linux"))]
fn owned_process_fd(_pid: u32) -> Result<std::os::fd::OwnedFd, MathRuntimeError> {
    Err(pse_backend_native::ProblemError::Unsupported(
        "private native child readiness requires Linux pidfd".into(),
    )
    .into())
}

#[cfg(target_os = "linux")]
fn wait_ready(
    scope: &ExecutionScope,
    fd: &impl AsFd,
    events: rustix::event::PollFlags,
) -> Result<(), MathRuntimeError> {
    use rustix::{
        event::{EventfdFlags, PollFd, PollFlags, Timespec},
        io::Errno,
    };
    struct Signal(std::os::fd::OwnedFd);
    impl std::task::Wake for Signal {
        fn wake(self: Arc<Self>) {
            self.wake_by_ref();
        }
        fn wake_by_ref(self: &Arc<Self>) {
            while let Err(Errno::INTR) = rustix::io::write(&self.0, &1_u64.to_ne_bytes()) {}
        }
    }
    let signal = Arc::new(Signal(
        rustix::event::eventfd(0, EventfdFlags::CLOEXEC | EventfdFlags::NONBLOCK)
            .map_err(infrastructure)?,
    ));
    let waker = std::task::Waker::from(signal.clone());
    let _watch = scope.cancellation_control().watch(&waker);
    loop {
        checkpoint(scope)?;
        let timeout = scope
            .deadline()
            .map(|deadline| Timespec::try_from(deadline.saturating_duration_since(Instant::now())))
            .transpose()
            .map_err(infrastructure)?;
        let mut descriptors = [
            PollFd::new(fd, events),
            PollFd::new(&signal.0, PollFlags::IN),
        ];
        match rustix::event::poll(&mut descriptors, timeout.as_ref()) {
            Err(Errno::INTR) => continue,
            Err(error) => return Err(infrastructure(error)),
            Ok(_) => {}
        }
        checkpoint(scope)?;
        let ready = descriptors[0].revents();
        if ready.contains(PollFlags::NVAL) {
            return Err(infrastructure("owned readiness descriptor invalid"));
        }
        if ready.intersects(events | PollFlags::HUP | PollFlags::ERR) {
            return Ok(());
        }
    }
}
#[cfg(not(target_os = "linux"))]
fn wait_ready(
    _scope: &ExecutionScope,
    _fd: &impl AsFd,
    _events: rustix::event::PollFlags,
) -> Result<(), MathRuntimeError> {
    Err(pse_backend_native::ProblemError::Unsupported(
        "private native child readiness requires Linux eventfd".into(),
    )
    .into())
}

fn io_until(
    scope: &ExecutionScope,
    channel: &mut UnixStream,
    events: rustix::event::PollFlags,
    mut operation: impl FnMut(&mut UnixStream) -> io::Result<usize>,
) -> Result<usize, MathRuntimeError> {
    loop {
        checkpoint(scope)?;
        match operation(channel) {
            Ok(count) => return Ok(count),
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock
                        | io::ErrorKind::TimedOut
                        | io::ErrorKind::Interrupted
                ) =>
            {
                if error.kind() != io::ErrorKind::Interrupted {
                    wait_ready(scope, channel, events)?;
                }
            }
            Err(error) => return Err(infrastructure(error)),
        }
    }
}
fn write_bytes(
    channel: &mut UnixStream,
    mut bytes: &[u8],
    scope: &ExecutionScope,
) -> Result<(), MathRuntimeError> {
    while !bytes.is_empty() {
        let count = io_until(scope, channel, rustix::event::PollFlags::OUT, |channel| {
            channel.write(bytes)
        })?;
        if count == 0 {
            return Err(infrastructure("closed request channel"));
        }
        bytes = &bytes[count..];
    }
    Ok(())
}
fn read_bytes(
    channel: &mut UnixStream,
    mut bytes: &mut [u8],
    scope: &ExecutionScope,
) -> Result<(), MathRuntimeError> {
    while !bytes.is_empty() {
        let count = io_until(scope, channel, rustix::event::PollFlags::IN, |channel| {
            channel.read(bytes)
        })?;
        if count == 0 {
            return Err(infrastructure("child died or closed its private channel"));
        }
        bytes = &mut bytes[count..];
    }
    Ok(())
}
fn send(
    channel: &mut UnixStream,
    value: &impl Serialize,
    limit: usize,
    scope: &ExecutionScope,
) -> Result<(), MathRuntimeError> {
    struct Count {
        bytes: usize,
        limit: usize,
    }
    impl Write for Count {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.bytes = self
                .bytes
                .checked_add(bytes.len())
                .filter(|count| *count <= self.limit)
                .ok_or_else(|| io::Error::other("native child outgoing frame limit"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    checkpoint(scope)?;
    let mut count = Count { bytes: 0, limit };
    serde_json::to_writer(&mut count, value).map_err(|e| contract(e.to_string()))?;
    let bytes = serde_json::to_vec(value).map_err(infrastructure)?;
    if bytes.len() != count.bytes {
        return Err(contract("native child frame extent changed"));
    }
    write_bytes(channel, &(bytes.len() as u64).to_le_bytes(), scope)?;
    write_bytes(channel, &bytes, scope)
}
fn receive<T: serde::de::DeserializeOwned>(
    channel: &mut UnixStream,
    limit: usize,
    scope: &ExecutionScope,
) -> Result<T, MathRuntimeError> {
    let mut size = [0; 8];
    read_bytes(channel, &mut size, scope)?;
    let size = usize::try_from(u64::from_le_bytes(size)).map_err(infrastructure)?;
    if size > limit {
        return Err(contract("native child incoming frame limit"));
    }
    let mut bytes = vec![0; size];
    read_bytes(channel, &mut bytes, scope)?;
    checkpoint(scope)?;
    serde_json::from_slice(&bytes).map_err(infrastructure)
}

/// Entry point of the private native worker, never a public stdin DTO admission API.
/// # Safety
/// The controlled deployment launcher must own this inherited Unix socket and provide
/// direct compiler-issued source bytes. It must authenticate the executable/build/attempt,
/// retain the original deadline and charges, screen original coordinates, and reap the child.
#[allow(
    unsafe_code,
    reason = "ADR-0185 trusted private child composition root forwards direct admitted-source authority"
)]
pub unsafe fn serve_private(attempt: SemanticId, parent: u32) -> Result<(), MathRuntimeError> {
    let mut channel = UnixStream::from(
        io::stdin()
            .as_fd()
            .try_clone_to_owned()
            .map_err(infrastructure)?,
    );
    // A public pipe/file/terminal cannot stand in for the private inherited socket.
    channel.peer_addr().map_err(infrastructure)?;
    let parent_record = std::fs::read_to_string("/proc/self/status").map_err(infrastructure)?;
    let actual_parent = parent_record
        .lines()
        .find_map(|line| line.strip_prefix("PPid:"))
        .and_then(|v| v.trim().parse::<u32>().ok());
    if actual_parent != Some(parent) {
        return Err(contract("native child parent identity differs"));
    }
    let startup = ExecutionScope::new(Cancellation::default(), None);
    let executable =
        pse_buildinfo::FileObservation::capture(&std::env::current_exe().map_err(infrastructure)?)
            .map_err(infrastructure)?;
    let diagnostics = crate::observation::process_diagnostics();
    let diagnostic = diagnostics.hub().start("native.child");
    let hello = Hello {
        diagnostic_operation: diagnostic.context().id(),
        protocol: PROTOCOL.into(),
        attempt,
        pid: std::process::id(),
        parent,
        executable,
        build: pse_buildinfo::BuildInfo::captured(),
        native_build: native_profile()?,
    };
    send(&mut channel, &hello, RESPONSE_LIMIT, &startup)?;
    let request: Request = receive(&mut channel, REQUEST_LIMIT, &startup)?;
    if request.attempt != attempt
        || request.remaining.is_zero()
        || digest(&request.source) != request.request
    {
        return Err(contract("native child admitted request identity differs"));
    }
    let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
    h.str(PROTOCOL)
        .id(&attempt)
        .hash(&request.request)
        .str(&hello.executable.sha256)
        .part(&serde_json::to_vec(&hello.native_build).map_err(infrastructure)?)
        .u64(request.remaining.as_secs())
        .u64(u64::from(request.remaining.subsec_nanos()));
    if h.finish_hash() != request.execution {
        return Err(contract("native child execution profile identity differs"));
    }
    let deadline = Instant::now()
        .checked_add(request.remaining)
        .ok_or_else(|| contract("native child receiving deadline extent"))?;
    let scope = ExecutionScope::new(Cancellation::default(), Some(deadline));
    let registration = diagnostics.register_child(request.diagnostic_parent);
    let context = diagnostic.context();
    if let Some(parent) = request.diagnostic_parent {
        context.link_id(
            parent,
            pse_diagnostics::observation::Relationship::Continuation,
        );
    }
    context.source_ref("native.attempt", attempt.to_string().as_bytes());
    let _entered = context.enter();
    let result = reconstruct_and_solve(&request, &scope)
        .map_err(|error| error.boundary_diagnostic(pse_diagnostics::DiagnosticStage::Native));
    let solve_success = result.is_ok();
    context.boolean("native.solve.return.success", solve_success);
    let sending = context.phase("native.response.send");
    let sent = send(
        &mut channel,
        &Response {
            attempt,
            execution: request.execution,
            request: request.request,
            result,
        },
        RESPONSE_LIMIT,
        &scope,
    );
    drop(sending);
    context.boolean("native.response.sent", sent.is_ok());
    diagnostic.finish(if solve_success && sent.is_ok() {
        pse_diagnostics::observation::Outcome::Completed
    } else {
        pse_diagnostics::observation::Outcome::Failed
    });
    drop(registration);
    diagnostics.stop_publisher();
    sent
}

#[allow(
    unsafe_code,
    reason = "ADR-0185 private inherited-channel scope issues the restricted exact admitted recipe authority"
)]
fn reconstruct_and_solve(
    request: &Request,
    scope: &ExecutionScope,
) -> Result<Vec<f64>, MathRuntimeError> {
    checkpoint(scope)?;
    let wire: RootWire = serde_json::from_slice(&request.source).map_err(infrastructure)?;
    if digest(&wire.registry) != wire.registry_digest || !wire.spec.providers.is_empty() {
        return Err(contract(
            "native child physical registry or provider closure differs",
        ));
    }
    let registry: pse_quantity::QuantityRegistryBuilder =
        serde_json::from_slice(&wire.registry).map_err(infrastructure)?;
    let registry = registry.build().map_err(pse_math::MathError::from)?;
    pse_math::initialize()?;
    checkpoint(scope)?;
    // SAFETY: this function is reached only by serve_private's trusted inherited socket
    // role after exact executable/build/attempt and payload binding. The parent captured
    // this recipe directly from AdmittedBody and bound its actual physical context.
    let permit = unsafe {
        pse_ids::scientific_replay::QualifiedScientificRecipe::from_admitted_process_message(
            wire.body.clone(),
            portable_payload_hash(&wire.body),
            request.execution,
            request.request,
        )
    }
    .map_err(infrastructure)?;
    let body = reconstruct_portable(&permit, &registry, scope.cancellation())?;
    if body.spec() != &wire.spec || !body.math().providers().is_empty() {
        return Err(contract("native child reconstructed source differs"));
    }
    let problem = compile_problem(
        &body,
        &wire.unknowns,
        &wire.rows,
        wire.parameters.len(),
        wire.evaluation,
        wire.max_entries,
        scope,
    )?;
    let mut options = wire.options;
    options.time_limit = scope
        .remaining(options.time_limit)
        .map_err(pse_math::MathError::Scope)?;
    let mut worker = pse_backend_native::implicit::Kinsol.worker();
    let point = worker.solve(
        problem,
        &wire.parameters,
        &options,
        scope.cancellation_control(),
    )?;
    checkpoint(scope)?;
    Ok(point)
}

#[cfg(test)]
#[path = "native_child_tests.rs"]
mod tests;
