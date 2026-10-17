// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit scalar C++ preparation; shared code and independently loaded scratch.
use crate::{MathError, library};
use std::{
    io::Read,
    os::unix::process::CommandExt,
    process::{Child, Command, ExitStatus, Stdio},
    sync::{
        Arc, Mutex,
    },
    time::Instant,
};
use symbolica::{
    atom::Symbol,
    evaluate::{
        CompiledRealEvaluator, ExportSettings, ExportedInstructions, ExpressionEvaluator,
        InlineASM, Instruction,
    },
};

const FUNCTION: &str = "pse_value";
const COMPILER: &str = "/usr/bin/g++";
const FLAGS: &[&str] = &[
    "-std=c++17",
    "-O2",
    "-pipe",
    "-fPIC",
    "-fno-fast-math",
    "-fno-unsafe-math-optimizations",
    "-ffp-contract=off",
    "-march=x86-64",
    "-mtune=generic",
];

pub(crate) struct Template {
    // Drop the loaded template before its source/executable directory. Workers
    // retain this owner as well as their library-owned executable reference.
    evaluator: Mutex<CompiledRealEvaluator>,
    _directory: tempfile::TempDir,
}
pub(crate) struct Evaluator {
    // Mutex supplies Sync for immutable program containers; evaluation obtains
    // exclusive mutable access, never a shared serialization lock across workers.
    worker: Option<Mutex<CompiledRealEvaluator>>,
    // Rust drops fields in declaration order: unload private native scratch
    // before the last template can remove its directory and shared ownership.
    template: Arc<Template>,
}
impl Clone for Evaluator {
    fn clone(&self) -> Self {
        Self {
            template: self.template.clone(),
            worker: None,
        }
    }
}
impl Evaluator {
    pub(crate) fn instantiate(&mut self) -> Result<(), MathError> {
        if self.worker.is_none() {
            let evaluator = self
                .template
                .evaluator
                .lock()
                .map_err(|_| MathError::Library("compiled template lock poisoned".into()))?
                .load_new_function(FUNCTION)
                .map_err(MathError::Library)?;
            self.worker = Some(Mutex::new(evaluator));
        }
        Ok(())
    }
    pub(crate) fn reset(&mut self) {
        self.worker = None;
    }
    pub(crate) fn evaluate(&mut self, arguments: &[f64], values: &mut [f64]) -> Result<(), String> {
        let evaluator = self
            .worker
            .as_mut()
            .ok_or("compiled worker not instantiated")?
            .get_mut()
            .map_err(|_| "compiled worker lock poisoned")?;
        if evaluator.get_input_len() != arguments.len()
            || evaluator.get_output_len() != values.len()
        {
            return Err("compiled scalar input/output extent".into());
        }
        evaluator.evaluate(arguments, values);
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("math.native_scalar_evaluations", "count", 1);
        }
        Ok(())
    }
}

// These are source bounds for Symbolica 3.0.1 export.rs, not compiler RSS
// estimates. Generic scalar export spells usize/i64 indices in <=20 digits,
// finite f64 constants as scientific notation, and the fixed wrappers below.
// Registered constants/subevaluators and opaque function/control channels refuse.
// This is not a validator for arbitrary Symbolica evaluators: its sole production
// caller receives library::bounded_evaluator products. library::exact_evaluator
// uses Atom::evaluator_multiple's empty FunctionMap and never registers functions;
// ScalarProducts has no public insertion route. This producer boundary excludes
// unused external bodies, which export_cpp emits but export_instructions omits.
fn export_bound(export: &ExportedInstructions<f64>) -> Result<usize, MathError> {
    if !export.constant_functions.is_empty()
        || !export.sub_evaluators.is_empty()
        || export.constants.iter().any(|value| !value.is_finite())
    {
        return Err(MathError::Contract(
            "compiled scalar export requires finite provider-free constants".into(),
        ));
    }
    let mut bytes = 4096usize
        .checked_add(
            export
                .output_count
                .checked_mul(128)
                .ok_or(MathError::Limit("compiled output export extent"))?,
        )
        .ok_or(MathError::Limit("compiled output export extent"))?;
    for instruction in &export.instructions {
        let operands = match instruction {
            Instruction::Add(_, arguments, _) | Instruction::Mul(_, arguments, _) => {
                arguments.len()
            }
            Instruction::Pow(..) | Instruction::Assign(..) => 1,
            Instruction::Powf(..) => 2,
            Instruction::Fun(_, call, _) => {
                let (symbol, tags, arguments) = call.as_ref();
                if !tags.is_empty()
                    || arguments.len() != 1
                    || ![
                        Symbol::EXP,
                        Symbol::LOG,
                        Symbol::SIN,
                        Symbol::COS,
                        Symbol::SQRT,
                        Symbol::ABS,
                        Symbol::CONJ,
                    ]
                    .contains(symbol)
                {
                    return Err(MathError::Contract(
                        "compiled scalar export refuses opaque function channels".into(),
                    ));
                }
                1
            }
            _ => {
                return Err(MathError::Contract(
                    "compiled scalar export refuses opaque instruction channels".into(),
                ));
            }
        };
        bytes = bytes
            .checked_add(
                operands
                    .checked_mul(64)
                    .and_then(|n| n.checked_add(256))
                    .ok_or(MathError::Limit("compiled instruction export extent"))?,
            )
            .ok_or(MathError::Limit("compiled instruction export extent"))?;
    }
    Ok(bytes)
}

const CAPTURE: usize = 4096;
const READER_STACK: usize = 64 << 10;

/// Keep the group leader waitable (WNOWAIT) until every owned group member has
/// stopped. Its reserved PID prevents signalling a replacement numeric PGID.
struct CompilerChild {
    child: Child,
    group: rustix::process::Pid,
    joined: bool,
}
impl CompilerChild {
    fn drain(&mut self) -> std::io::Result<ExitStatus> {
        loop {
            // The leader remains unreaped, reserving the group identity while
            // every actual member is discovered, signalled and awaited by pidfd.
            if let Err(error) = rustix::process::kill_process_group(self.group, rustix::process::Signal::KILL)
                && error != rustix::io::Errno::SRCH
            { retain_unknown_drain(error); }
            let members = match group_members(self.group.as_raw_pid()) {
                Ok(members) => members,
                Err(error) => retain_unknown_drain(error),
            };
            if members.is_empty() { break; }
            for member in &members {
                if let Err(error) = rustix::process::pidfd_send_signal(member, rustix::process::Signal::KILL)
                    && error != rustix::io::Errno::SRCH
                { retain_unknown_drain(error); }
            }
            for member in &members {
                if let Err(error) = wait_exit(member, None) { retain_unknown_drain(error); }
            }
            // A member may have forked before its kill. Re-inventory after all
            // known exits; a child/leader exit is never whole-group drain.
        }
        let status = match self.child.wait() {
            Ok(status) => status,
            Err(error) => retain_unknown_drain(error),
        };
        self.joined = true;
        Ok(status)
    }
}
impl Drop for CompilerChild {
    fn drop(&mut self) {
        if !self.joined { let _ = self.drain(); }
    }
}
/// No safe readiness source exists after an unproved inventory/drain fault.
/// Keep this stack, child, stderr reader, directory and admitted caller alive.
/// Cancellation is not subscribed here: an already requested stop must not spin
/// or release unknown issued work. There is deliberately no reaper or timer retry.
fn retain_unknown_drain(error: impl std::fmt::Display) -> ! {
    if let Some(context) = pse_diagnostics::observation::current() {
        context.text("math.native_compiler_drain_fault", &error.to_string());
        context.text("math.native_compiler_state", "blocked-unproved-drain");
    }
    loop { std::thread::park(); }
}
fn proc_identity(stat: &str) -> std::io::Result<(bool, i32, u64)> {
    let fields = stat.rsplit_once(") ")
        .map(|(_, tail)| tail.split_whitespace().collect::<Vec<_>>())
        .ok_or_else(|| std::io::Error::other("malformed compiler group stat"))?;
    let group = fields.get(2).and_then(|value| value.parse().ok())
        .ok_or_else(|| std::io::Error::other("missing compiler group identity"))?;
    let start = fields.get(19).and_then(|value| value.parse().ok())
        .ok_or_else(|| std::io::Error::other("missing compiler member start identity"))?;
    Ok((!matches!(fields.first().copied(), Some("Z" | "X")), group, start))
}
fn group_members(group: i32) -> std::io::Result<Vec<std::os::fd::OwnedFd>> {
    let mut members = Vec::new();
    for entry in std::fs::read_dir("/proc")? {
        let entry = entry?;
        let Some(pid) = entry.file_name().to_str().and_then(|name| name.parse::<i32>().ok()).and_then(rustix::process::Pid::from_raw) else { continue; };
        let stat_path = entry.path().join("stat");
        let before = match std::fs::read_to_string(&stat_path) {
            Ok(stat) => proc_identity(&stat)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        if !before.0 || before.1 != group { continue; }
        let fd = match rustix::process::pidfd_open(pid, rustix::process::PidfdFlags::empty()) {
            Ok(fd) => fd,
            Err(rustix::io::Errno::SRCH) => continue,
            Err(error) => return Err(error.into()),
        };
        // Re-read after pidfd_open: numeric PID reuse cannot turn an unrelated
        // member into permission to signal it. The descriptor owns that identity.
        let after = match std::fs::read_to_string(&stat_path) {
            Ok(stat) => proc_identity(&stat)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        if after.0 && after.1 == group && after.2 == before.2 { members.push(fd); }
    }
    Ok(members)
}
fn wait_exit(fd: &std::os::fd::OwnedFd, scope: Option<&pse_kernels::ExecutionScope>) -> Result<(), MathError> {
    use rustix::{event::{EventfdFlags, PollFd, PollFlags, Timespec}, io::Errno};
    struct Signal(std::os::fd::OwnedFd);
    impl std::task::Wake for Signal {
        fn wake(self: Arc<Self>) { self.wake_by_ref(); }
        fn wake_by_ref(self: &Arc<Self>) {
            while let Err(Errno::INTR) = rustix::io::write(&self.0, &1_u64.to_ne_bytes()) {}
        }
    }
    let signal = scope.map(|_| rustix::event::eventfd(0, EventfdFlags::CLOEXEC | EventfdFlags::NONBLOCK)
        .map(|fd| Arc::new(Signal(fd))).map_err(|error| MathError::Library(error.to_string()))).transpose()?;
    let _watch = scope.zip(signal.as_ref()).map(|(scope, signal)| scope.cancellation_control().watch(&std::task::Waker::from(signal.clone())));
    loop {
        if let Some(scope) = scope { scope.check().map_err(crate::error::scope_error)?; }
        let timeout = scope.and_then(pse_kernels::ExecutionScope::deadline)
            .map(|deadline| Timespec::try_from(deadline.saturating_duration_since(Instant::now())))
            .transpose().map_err(|error| MathError::Library(error.to_string()))?;
        let mut descriptors = vec![PollFd::new(fd, PollFlags::IN)];
        if let Some(signal) = &signal { descriptors.push(PollFd::new(&signal.0, PollFlags::IN)); }
        match rustix::event::poll(&mut descriptors, timeout.as_ref()) {
            Err(Errno::INTR) => continue,
            Err(error) => return Err(MathError::Library(error.to_string())),
            Ok(_) => {}
        }
        if let Some(scope) = scope { scope.check().map_err(crate::error::scope_error)?; }
        let ready = descriptors[0].revents();
        if ready.contains(PollFlags::NVAL) { return Err(MathError::Library("compiler pidfd invalid".into())); }
        if ready.intersects(PollFlags::IN | PollFlags::HUP | PollFlags::ERR) { return Ok(()); }
    }
}
fn wait(compiler: &CompilerChild, scope: &pse_kernels::ExecutionScope) -> Result<(), MathError> {
    let fd = rustix::process::pidfd_open(compiler.group, rustix::process::PidfdFlags::empty())
        .map_err(|error| MathError::Library(error.to_string()))?;
    wait_exit(&fd, Some(scope))?;
    // Observe without reaping. The leader continues to anchor the owned PGID
    // until drain has independently checked every descendant.
    rustix::process::waitid(rustix::process::WaitId::Pid(compiler.group), rustix::process::WaitIdOptions::EXITED | rustix::process::WaitIdOptions::NOWAIT)
        .map_err(|error| MathError::Library(error.to_string()))?;
    scope.check().map_err(crate::error::scope_error)
}
fn run_compiler(mut command: Command, scope: &pse_kernels::ExecutionScope) -> Result<(), MathError> {
    scope.check().map_err(crate::error::scope_error)?;
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .process_group(0);
    let child = command
        .spawn()
        .map_err(|e| MathError::Library(e.to_string()))?;
    let group = rustix::process::Pid::from_child(&child);
    let mut compiler = CompilerChild {
        child,
        group,
        joined: false,
    };
    let mut stderr = compiler
        .child
        .stderr
        .take()
        .ok_or_else(|| MathError::Library("missing compiler stderr pipe".into()))?;
    // Drain all bytes so a noisy compiler cannot block; retain only a bounded
    // prefix. The thread stack, capture and conversion overlap were prepaid.
    let reader = std::thread::Builder::new()
        .name("pse-cpp-stderr".into())
        .stack_size(READER_STACK)
        .spawn(move || -> std::io::Result<Vec<u8>> {
            let mut prefix = Vec::with_capacity(CAPTURE);
            let mut chunk = [0u8; 1024];
            loop {
                let size = stderr.read(&mut chunk)?;
                if size == 0 {
                    return Ok(prefix);
                }
                let keep = size.min(CAPTURE - prefix.len());
                prefix.extend_from_slice(&chunk[..keep]);
            }
        })
        .map_err(|e| MathError::Library(e.to_string()))?;
    let observed = wait(&compiler, scope);
    // Always drain before releasing temporary outputs, including driver failure
    // and a nominally successful driver that left an executing child behind.
    let status = compiler.drain();
    let diagnostics = reader
        .join()
        .map_err(|_| MathError::Library("compiler stderr reader panicked".into()))?
        .map_err(|e| MathError::Library(e.to_string()));
    scope.check().map_err(crate::error::scope_error)?;
    observed?;
    let status = status.map_err(|e| MathError::Library(e.to_string()))?;
    let diagnostics = diagnostics?;
    if !status.success() {
        return Err(MathError::Library(format!(
            "compiled scalar compiler {status}: {}",
            String::from_utf8_lossy(&diagnostics)
        )));
    }
    Ok(())
}
fn compiler_command(path: &str, limit: usize, directory: &std::path::Path) -> Command {
    // prlimit installs the hard per-output file limit before exec, inherited by
    // cc1plus/as/ld. Separate compile/link phases own one named output each.
    let mut command = Command::new("/usr/bin/prlimit");
    command
        .arg(format!("--fsize={limit}:{limit}"))
        .arg("--core=0:0")
        .arg("--")
        .arg(path)
        .env("TMPDIR", directory);
    command
}

pub(crate) fn compile(
    evaluator: &ExpressionEvaluator<f64>,
    storage: &library::Storage,
    export: &ExportedInstructions<f64>,
    module_bytes: usize,
    scope: &pse_kernels::ExecutionScope,
) -> Result<Evaluator, MathError> {
    compile_prepared_with(
        evaluator,
        storage,
        export,
        module_bytes,
        scope,
        COMPILER,
    )
}
fn compile_prepared_with(
    evaluator: &ExpressionEvaluator<f64>,
    storage: &library::Storage,
    export: &ExportedInstructions<f64>,
    module_bytes: usize,
    scope: &pse_kernels::ExecutionScope,
    compiler_path: &str,
) -> Result<Evaluator, MathError> {
    if !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        return Err(MathError::Contract(
            "compiled scalar profile supports Linux x86-64".into(),
        ));
    }
    scope.check().map_err(crate::error::scope_error)?;
    let _phase = pse_diagnostics::observation::phase("math.native_scalar_compilation");
    // The caller borrows its already source-admitted instruction inspection,
    // also used for scratch/reset eligibility. No native-module inspection clone
    // precedes the module's own preflight or duplicates that source population.
    let scratch = storage
        .numeric_entries
        .checked_mul(size_of::<f64>())
        .ok_or(MathError::Limit("compiled template scratch"))?;
    let source_bound = export_bound(export)?;
    // export.rs builds argument String vectors, joined expressions and formatted
    // assignments while the growing result String remains live. Eight complete
    // source populations cover these capacities/temporaries for the admitted
    // fixed-width channels, plus the separately counted instruction view.
    let export_peak = source_bound
        .checked_mul(8)
        .and_then(|n| n.checked_add(storage.instruction_bytes))
        .ok_or(MathError::Limit("compiled export construction extent"))?;
    let fixed = scratch
        .checked_add(size_of::<Template>() + size_of::<Evaluator>() + (64 << 10))
        .and_then(|n| n.checked_add(READER_STACK + 16 * CAPTURE))
        .ok_or(MathError::Limit("compiled module metadata extent"))?;
    let output_capacity = module_bytes
        .checked_sub(export_peak)
        .and_then(|n| n.checked_sub(fixed))
        .and_then(|n| n.checked_sub(source_bound))
        .filter(|n| *n >= 3)
        .ok_or(MathError::Limit(
            "compiled source/output preflight allowance",
        ))?;
    // Object file, shared file and its loaded file-backed image coexist. The
    // fixed allowance above separately covers page rounding/loader metadata.
    let object_limit = output_capacity / 3;
    let code_limit = output_capacity / 3;
    let source = evaluator
        .export_cpp_str::<f64>(FUNCTION, ExportSettings::new().inline_asm(InlineASM::None))
        .map_err(MathError::Library)?;
    if source.len() > source_bound || source.capacity() > source_bound.saturating_mul(2) {
        return Err(MathError::Contract(
            "compiled export exceeded its source-qualified bound".into(),
        ));
    }
    let directory = tempfile::Builder::new()
        .prefix("pse-math-native-")
        .tempdir()
        .map_err(|e| MathError::Library(e.to_string()))?;
    let source_path = directory.path().join("value.cpp");
    let library_path = directory.path().join("value.so");
    let object_path = directory.path().join("value.o");
    std::fs::write(&source_path, source).map_err(|e| MathError::Library(e.to_string()))?;
    let mut compile = compiler_command(compiler_path, object_limit, directory.path());
    compile
        .args(FLAGS)
        .arg("-c")
        .arg(&source_path)
        .arg("-o")
        .arg(&object_path);
    run_compiler(compile, scope)?;
    let mut link = compiler_command(compiler_path, code_limit, directory.path());
    link.arg("-shared")
        .arg("-Wl,-z,max-page-size=4096")
        .arg(&object_path)
        .arg("-o")
        .arg(&library_path);
    run_compiler(link, scope)?;
    let code_bytes = usize::try_from(
        std::fs::metadata(&library_path)
            .map_err(|e| MathError::Library(e.to_string()))?
            .len(),
    )
    .map_err(|_| MathError::Limit("compiled executable extent"))?;
    if code_bytes > code_limit {
        return Err(MathError::Contract(
            "compiler exceeded enforced output file limit".into(),
        ));
    }
    scope.check().map_err(crate::error::scope_error)?;
    let loaded =
        CompiledRealEvaluator::load(&library_path, FUNCTION).map_err(MathError::Library)?;
    if loaded.get_input_len() != evaluator.get_input_len()
        || loaded.get_output_len() != evaluator.get_output_len()
    {
        return Err(MathError::Contract(
            "compiled scalar library dimensions".into(),
        ));
    }
    scope.check().map_err(crate::error::scope_error)?;
    if let Some(context) = pse_diagnostics::observation::current() {
        context.count("math.native_scalar_compilations", "count", 1);
        context.gauge(
            "math.native_scalar_executable_bytes",
            "bytes",
            code_bytes as f64,
        );
    }
    Ok(Evaluator {
        template: Arc::new(Template {
            evaluator: Mutex::new(loaded),
            _directory: directory,
        }),
        worker: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{os::unix::fs::PermissionsExt, time::Duration};
    use symbolica::{
        atom::{Atom, AtomCore},
        parse,
    };

    // Fixture construction/inspection belongs to the already constructed test
    // evaluator, matching the source-owned inspection in compile_stages.
    fn compile_with(
        evaluator: &ExpressionEvaluator<f64>,
        module_bytes: usize,
        scope: &pse_kernels::ExecutionScope,
        compiler: &str,
    ) -> Result<Evaluator, MathError> {
        let storage = library::storage(evaluator)?;
        let export = evaluator.export_instructions();
        compile_prepared_with(
            evaluator,
            &storage,
            &export,
            module_bytes,
            scope,
            compiler,
        )
    }
    fn compile(
        evaluator: &ExpressionEvaluator<f64>,
        module_bytes: usize,
        scope: &pse_kernels::ExecutionScope,
    ) -> Result<Evaluator, MathError> {
        compile_with(evaluator, module_bytes, scope, COMPILER)
    }

    fn arithmetic() -> ExpressionEvaluator<f64> {
        crate::initialize().unwrap();
        Atom::evaluator_multiple(
            &[parse!("pse_native_test::x^2+1")],
            &[parse!("pse_native_test::x")],
        )
        .horner_iterations(0)
        .cpe_iterations(Some(0))
        .build()
        .unwrap()
        .map_coeff(&|coefficient| coefficient.to_real().unwrap().to_f64())
    }

    #[test]
    fn native_scalar_failed_compiler_and_pre_cancel_never_publish() {
        let evaluator = arithmetic();
        let cancelled = pse_diagnostics::Cancellation::default();
        assert!(matches!(
            compile_with(&evaluator, 1 << 20, &pse_kernels::ExecutionScope::new(cancelled.clone(), None), "/bin/false"),
            Err(MathError::Library(_))
        ));
        assert!(matches!(
            compile(&evaluator, 1, &pse_kernels::ExecutionScope::new(cancelled.clone(), None)),
            Err(MathError::Limit(_))
        ));
        cancelled.cancel();
        assert!(matches!(
            compile(&evaluator, 1 << 20, &pse_kernels::ExecutionScope::new(cancelled.clone(), None)),
            Err(MathError::Cancelled)
        ));
    }

    #[test]
    fn native_scalar_driver_exit_drains_live_children_before_return() {
        for exit in [0, 7] {
            let fixture = tempfile::tempdir().unwrap();
            let marker = fixture.path().join("child");
            let quoted = marker.to_string_lossy().replace('\'', "'\"'\"'");
            let mut command = Command::new("/bin/sh");
            command.arg("-c").arg(format!(
                "sleep 60 &\nprintf '%s\\n' \"$!\" > '{quoted}'\nexit {exit}\n"
            ));
            let result = run_compiler(command, &pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None));
            assert_eq!(result.is_ok(), exit == 0);
            let child = std::fs::read_to_string(marker).unwrap();
            assert_stopped_at_return(child.trim());
        }
    }

    #[test]
    fn native_scalar_export_bound_covers_supported_scalar_channels_and_refuses_unknowns() {
        crate::initialize().unwrap();
        let x = parse!("pse_native_bound::x");
        let y = parse!("pse_native_bound::y");
        let expressions = [
            &x + &y,
            &x * &y,
            x.pow(-1),
            x.pow(3),
            x.pow(&y),
            x.exp(),
            x.log(),
            x.sin(),
            x.cos(),
            x.sqrt(),
            x.abs(),
            x.conj(),
        ];
        let evaluator = Atom::evaluator_multiple(&expressions, &[x, y])
            .horner_iterations(0)
            .cpe_iterations(Some(0))
            .build()
            .unwrap()
            .map_coeff(&|coefficient| coefficient.to_real().unwrap().to_f64());
        let export = evaluator.export_instructions();
        let bound = export_bound(&export).unwrap();
        let source = evaluator
            .export_cpp_str::<f64>(FUNCTION, ExportSettings::new().inline_asm(InlineASM::None))
            .unwrap();
        assert!(source.len() <= bound && source.capacity() <= 2 * bound);
        let mut compiled = compile(&evaluator, 1 << 20, &pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None)).unwrap();
        compiled.instantiate().unwrap();
        let (x, y): (f64, f64) = (1.25, 2.5);
        let expected = [
            x + y,
            x * y,
            x.recip(),
            x.powi(3),
            x.powf(y),
            x.exp(),
            x.ln(),
            x.sin(),
            x.cos(),
            x.sqrt(),
            x.abs(),
            x,
        ];
        let mut actual = [0.; 12];
        compiled.evaluate(&[x, y], &mut actual).unwrap();
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert!(
                (actual - expected).abs() <= 32. * f64::EPSILON * expected.abs().max(1.),
                "compiled builtin {actual} differs from independent scalar {expected}"
            );
        }
        for value in [
            f64::MAX,
            f64::MIN,
            f64::MIN_POSITIVE,
            f64::from_bits(1),
            -0.,
        ] {
            assert!(format!("{value:e}").len() + 3 <= 64);
        }
        let mut tagged = evaluator.export_instructions();
        let Some(Instruction::Fun(_, call, _)) = tagged
            .instructions
            .iter_mut()
            .find(|instruction| matches!(instruction, Instruction::Fun(..)))
        else {
            panic!("builtin function export");
        };
        call.1.push("opaque".into());
        assert!(export_bound(&tagged).is_err());
        let mut control = evaluator.export_instructions();
        control.instructions.push(Instruction::Goto(0));
        assert!(export_bound(&control).is_err());
        let mut nonfinite = evaluator.export_instructions();
        nonfinite.constants.push(f64::INFINITY);
        assert!(export_bound(&nonfinite).is_err());
        let mut overflow = evaluator.export_instructions();
        overflow.output_count = usize::MAX;
        assert!(matches!(export_bound(&overflow), Err(MathError::Limit(_))));
    }

    #[test]
    fn native_scalar_export_preflight_covers_wide_argument_temporary_overlap() {
        crate::initialize().unwrap();
        let parameters = (0..256)
            .map(|i| library::formal(i).unwrap())
            .collect::<Vec<_>>();
        let sum = parameters
            .iter()
            .fold(Atom::num(0), |value, parameter| value + parameter);
        let product = parameters
            .iter()
            .fold(Atom::num(1), |value, parameter| value * parameter);
        let evaluator = Atom::evaluator_multiple(&[sum, product], &parameters)
            .horner_iterations(0)
            .cpe_iterations(Some(0))
            .build()
            .unwrap()
            .map_coeff(&|coefficient| coefficient.to_real().unwrap().to_f64());
        let bound = export_bound(&evaluator.export_instructions()).unwrap();
        let source = evaluator
            .export_cpp_str::<f64>(FUNCTION, ExportSettings::new().inline_asm(InlineASM::None))
            .unwrap();
        assert!(source.len() <= bound && source.capacity() <= 2 * bound);
        // A too-small module refuses before invoking even a deliberately invalid
        // compiler path; the effect boundary is behind export/output preflight.
        assert!(matches!(
            compile_with(
                &evaluator,
                bound,
                &pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
                "/no/compiler"
            ),
            Err(MathError::Limit(
                "compiled source/output preflight allowance"
            ))
        ));
    }

    #[test]
    fn native_scalar_compiler_diagnostics_and_file_output_are_bounded() {
        let fixture = tempfile::tempdir().unwrap();
        let noisy = fixture.path().join("noisy");
        std::fs::write(
            &noisy,
            "#!/bin/sh\nhead -c 1048576 /dev/zero | tr '\\000' x >&2\nexit 1\n",
        )
        .unwrap();
        std::fs::set_permissions(&noisy, std::fs::Permissions::from_mode(0o700)).unwrap();
        let error = compile_with(
            &arithmetic(),
            1 << 20,
            &pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
            noisy.to_str().unwrap(),
        );
        let Err(MathError::Library(detail)) = error else {
            panic!("noisy compiler must fail");
        };
        assert!(detail.len() < CAPTURE + 128);
        let oversized = fixture.path().join("oversized");
        std::fs::write(&oversized, "#!/bin/sh\nprev=\nfor arg; do if [ \"$prev\" = -o ]; then out=$arg; fi; prev=$arg; done\ndd if=/dev/zero of=\"$out\" bs=4096 count=1024 2>/dev/null\n").unwrap();
        std::fs::set_permissions(&oversized, std::fs::Permissions::from_mode(0o700)).unwrap();
        assert!(matches!(
            compile_with(
                &arithmetic(),
                1 << 20,
                &pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
                oversized.to_str().unwrap()
            ),
            Err(MathError::Library(_))
        ));
    }

    fn assert_stopped_at_return(child: &str) {
        let stat = std::fs::read_to_string(format!("/proc/{child}/stat"));
        assert!(
            stat.as_ref()
                .map_or(true, |stat| stat.rsplit_once(") ").is_some_and(
                    |(_, tail)| matches!(tail.as_bytes().first(), Some(b'Z' | b'X'))
                )),
            "owned compiler descendant still executing at return"
        );
    }

    #[test]
    fn native_scalar_cancelled_compiler_drains_owned_group_and_removes_partial_output() {
        let fixture = tempfile::tempdir().unwrap();
        let marker = fixture.path().join("compiler-started");
        let compiler = fixture.path().join("compiler");
        let quoted_marker = marker.to_string_lossy().replace('\'', "'\"'\"'");
        std::fs::write(&compiler, format!("#!/bin/sh\nfor arg; do case \"$arg\" in *.cpp) printf '%s\\n' \"$arg\" > '{quoted_marker}';; esac; done\nsleep 60 &\nprintf '%s\\n' \"$!\" >> '{quoted_marker}'\nwait\n")).unwrap();
        std::fs::set_permissions(&compiler, std::fs::Permissions::from_mode(0o700)).unwrap();
        let cancelled = pse_diagnostics::Cancellation::default();
        let flag = cancelled.clone();
        let started = marker.clone();
        let observer = std::thread::spawn(move || {
            let deadline = std::time::Instant::now() + Duration::from_secs(10);
            loop {
                if let Ok(text) = std::fs::read_to_string(&started)
                    && text.lines().count() == 2
                {
                    flag.cancel();
                    return text;
                }
                if std::time::Instant::now() >= deadline {
                    flag.cancel();
                    panic!("owned compiler fixture did not start");
                }
                std::thread::sleep(Duration::from_millis(5));
            }
        });
        let result = compile_with(
            &arithmetic(),
            1 << 20,
            &pse_kernels::ExecutionScope::new(cancelled.clone(), None),
            compiler.to_str().unwrap(),
        );
        assert!(matches!(result, Err(MathError::Cancelled)));
        let text = observer.join().unwrap();
        let mut lines = text.lines();
        let source = std::path::Path::new(lines.next().unwrap());
        assert!(
            !source.parent().unwrap().exists(),
            "cancelled preparation retained partial source/output"
        );
        let child = lines.next().unwrap();
        // Drain is part of compile_with: the descendant must already be dead
        // at return, not merely stop during a later grace-period poll.
        assert_stopped_at_return(child);
    }
}
