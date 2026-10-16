// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit scalar C++ preparation; shared code and independently loaded scratch.
use crate::{MathError, library};
use std::{
    fs::File,
    io::Read,
    os::unix::process::CommandExt,
    process::{Child, Command, ExitStatus, Stdio},
    sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}},
    time::Duration,
};
use symbolica::evaluate::{CompiledRealEvaluator, ExportSettings, ExpressionEvaluator, InlineASM};

const FUNCTION: &str = "pse_value";
const COMPILER: &str = "/usr/bin/g++";
const FLAGS: &[&str] = &[
    "-std=c++17", "-O2", "-shared", "-fPIC", "-fno-fast-math",
    "-fno-unsafe-math-optimizations", "-ffp-contract=off", "-march=x86-64", "-mtune=generic",
];

pub(crate) struct Template {
    // Drop the loaded template before its source/executable directory. Workers
    // retain this owner as well as their library-owned executable reference.
    evaluator: Mutex<CompiledRealEvaluator>,
    _directory: tempfile::TempDir,
}
pub(crate) struct Evaluator {
    template: Arc<Template>,
    // Mutex supplies Sync for immutable program containers; evaluation obtains
    // exclusive mutable access, never a shared serialization lock across workers.
    worker: Option<Mutex<CompiledRealEvaluator>>,
}
impl Clone for Evaluator {
    fn clone(&self) -> Self {
        Self { template: self.template.clone(), worker: None }
    }
}
impl Evaluator {
    pub(crate) fn instantiate(&mut self) -> Result<(), MathError> {
        if self.worker.is_none() {
            let evaluator = self.template.evaluator.lock()
                .map_err(|_| MathError::Library("compiled template lock poisoned".into()))?
                .load_new_function(FUNCTION).map_err(MathError::Library)?;
            self.worker = Some(Mutex::new(evaluator));
        }
        Ok(())
    }
    pub(crate) fn reset(&mut self) {
        self.worker = None;
    }
    pub(crate) fn evaluate(&mut self, arguments: &[f64], values: &mut [f64]) -> Result<(), String> {
        let evaluator = self.worker.as_mut().ok_or("compiled worker not instantiated")?
            .get_mut().map_err(|_| "compiled worker lock poisoned")?;
        if evaluator.get_input_len() != arguments.len() || evaluator.get_output_len() != values.len() {
            return Err("compiled scalar input/output extent".into());
        }
        evaluator.evaluate(arguments, values);
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("math.native_scalar_evaluations", "count", 1);
        }
        Ok(())
    }
}

/// Only the exact compiler driver below enters this owned process group. Its
/// children inherit the group and workload placement. Every unsuccessful exit
/// kills that group and joins the driver before directory/storage release.
struct CompilerChild { child: Child, joined: bool }
impl Drop for CompilerChild {
    fn drop(&mut self) {
        if !self.joined {
            if let Ok(raw) = i32::try_from(self.child.id())
                && let Some(group) = rustix::process::Pid::from_raw(raw)
            {
                let _ = rustix::process::kill_process_group(group, rustix::process::Signal::KILL);
            }
            let _ = self.child.wait();
        }
    }
}
fn wait(compiler: &mut CompilerChild, cancelled: &AtomicBool) -> Result<ExitStatus, MathError> {
    loop {
        if cancelled.load(Ordering::Acquire) { return Err(MathError::Cancelled); }
        if let Some(status) = compiler.child.try_wait().map_err(|e| MathError::Library(e.to_string()))? {
            compiler.joined = true;
            return Ok(status);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

pub(crate) fn compile(
    evaluator: &ExpressionEvaluator<f64>,
    module_bytes: usize,
    cancelled: &Arc<AtomicBool>,
) -> Result<Evaluator, MathError> {
    compile_with(evaluator, module_bytes, cancelled, COMPILER)
}
fn compile_with(
    evaluator: &ExpressionEvaluator<f64>,
    module_bytes: usize,
    cancelled: &Arc<AtomicBool>,
    compiler_path: &str,
) -> Result<Evaluator, MathError> {
    if !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        return Err(MathError::Contract("compiled scalar profile supports Linux x86-64".into()));
    }
    if cancelled.load(Ordering::Acquire) { return Err(MathError::Cancelled); }
    let _phase = pse_diagnostics::observation::phase("math.native_scalar_compilation");
    let storage = library::storage(evaluator)?;
    let scratch = storage.numeric_entries.checked_mul(size_of::<f64>())
        .ok_or(MathError::Limit("compiled template scratch"))?;
    let source = evaluator.export_cpp_str::<f64>(FUNCTION, ExportSettings::new().inline_asm(InlineASM::None))
        .map_err(MathError::Library)?;
    let metadata = size_of::<Template>() + size_of::<Evaluator>() + 4096;
    let before = source.len().checked_add(scratch).and_then(|n| n.checked_add(metadata))
        .filter(|n| *n <= module_bytes).ok_or(MathError::Limit("compiled source/module storage"))?;
    let directory = tempfile::Builder::new().prefix("pse-math-native-").tempdir()
        .map_err(|e| MathError::Library(e.to_string()))?;
    let source_path = directory.path().join("value.cpp");
    let library_path = directory.path().join("value.so");
    let stderr_path = directory.path().join("compiler.stderr");
    std::fs::write(&source_path, source).map_err(|e| MathError::Library(e.to_string()))?;
    let stderr = File::create(&stderr_path).map_err(|e| MathError::Library(e.to_string()))?;
    let mut command = Command::new(compiler_path);
    command.args(FLAGS).arg(&source_path).arg("-o").arg(&library_path)
        .stdin(Stdio::null()).stdout(Stdio::null()).stderr(stderr).process_group(0);
    let mut child = CompilerChild { child: command.spawn().map_err(|e| MathError::Library(e.to_string()))?, joined: false };
    let status = wait(&mut child, cancelled)?;
    if !status.success() {
        let mut detail = String::new();
        File::open(&stderr_path).map_err(|e| MathError::Library(e.to_string()))?
            .take(4096).read_to_string(&mut detail).map_err(|e| MathError::Library(e.to_string()))?;
        return Err(MathError::Library(format!("compiled scalar compiler {status}: {detail}")));
    }
    let code_bytes = usize::try_from(std::fs::metadata(&library_path)
        .map_err(|e| MathError::Library(e.to_string()))?.len())
        .map_err(|_| MathError::Limit("compiled executable extent"))?;
    before.checked_add(code_bytes).filter(|n| *n <= module_bytes)
        .ok_or(MathError::Limit("compiled executable/module storage"))?;
    if cancelled.load(Ordering::Acquire) { return Err(MathError::Cancelled); }
    let loaded = CompiledRealEvaluator::load(&library_path, FUNCTION).map_err(MathError::Library)?;
    if loaded.get_input_len() != evaluator.get_input_len() || loaded.get_output_len() != evaluator.get_output_len() {
        return Err(MathError::Contract("compiled scalar library dimensions".into()));
    }
    if cancelled.load(Ordering::Acquire) { return Err(MathError::Cancelled); }
    if let Some(context) = pse_diagnostics::observation::current() {
        context.count("math.native_scalar_compilations", "count", 1);
        context.gauge("math.native_scalar_executable_bytes", "bytes", code_bytes as f64);
    }
    Ok(Evaluator {
        template: Arc::new(Template { evaluator: Mutex::new(loaded), _directory: directory }),
        worker: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use symbolica::{atom::{Atom, AtomCore}, parse};

    fn arithmetic() -> ExpressionEvaluator<f64> {
        crate::initialize().unwrap();
        Atom::evaluator_multiple(&[parse!("pse_native_test::x^2+1")], &[parse!("pse_native_test::x")])
            .horner_iterations(0).cpe_iterations(Some(0)).build().unwrap()
            .map_coeff(&|coefficient| coefficient.to_real().unwrap().to_f64())
    }

    #[test]
    fn native_scalar_failed_compiler_and_pre_cancel_never_publish() {
        let evaluator = arithmetic();
        let cancelled = Arc::new(AtomicBool::new(false));
        assert!(matches!(compile_with(&evaluator, 1 << 20, &cancelled, "/bin/false"), Err(MathError::Library(_))));
        assert!(matches!(compile(&evaluator, 1, &cancelled), Err(MathError::Limit(_))));
        cancelled.store(true, Ordering::Release);
        assert!(matches!(compile(&evaluator, 1 << 20, &cancelled), Err(MathError::Cancelled)));
    }

    #[test]
    fn native_scalar_cancelled_compiler_drains_owned_group_and_removes_partial_output() {
        let fixture = tempfile::tempdir().unwrap();
        let marker = fixture.path().join("compiler-started");
        let compiler = fixture.path().join("compiler");
        let quoted_marker = marker.to_string_lossy().replace('\'', "'\"'\"'");
        std::fs::write(&compiler, format!("#!/bin/sh\nfor arg; do case \"$arg\" in *.cpp) printf '%s\\n' \"$arg\" > '{quoted_marker}';; esac; done\nsleep 60 &\nprintf '%s\\n' \"$!\" >> '{quoted_marker}'\nwait\n")).unwrap();
        std::fs::set_permissions(&compiler, std::fs::Permissions::from_mode(0o700)).unwrap();
        let cancelled = Arc::new(AtomicBool::new(false));
        let flag = cancelled.clone();
        let started = marker.clone();
        let observer = std::thread::spawn(move || {
            let deadline = std::time::Instant::now() + Duration::from_secs(10);
            loop {
                if let Ok(text) = std::fs::read_to_string(&started)
                    && text.lines().count() == 2
                {
                    flag.store(true, Ordering::Release);
                    return text;
                }
                if std::time::Instant::now() >= deadline {
                    flag.store(true, Ordering::Release);
                    panic!("owned compiler fixture did not start");
                }
                std::thread::sleep(Duration::from_millis(5));
            }
        });
        let result = compile_with(&arithmetic(), 1 << 20, &cancelled, compiler.to_str().unwrap());
        assert!(matches!(result, Err(MathError::Cancelled)));
        let text = observer.join().unwrap();
        let mut lines = text.lines();
        let source = std::path::Path::new(lines.next().unwrap());
        assert!(!source.parent().unwrap().exists(), "cancelled preparation retained partial source/output");
        let child = lines.next().unwrap();
        // A killed orphan can remain a zombie until the host reaps it; it must
        // never remain executing after the owned driver has been joined.
        for _ in 0..100 {
            let stat = std::fs::read_to_string(format!("/proc/{child}/stat"));
            if stat.as_ref().map_or(true, |stat| stat.rsplit_once(") ").is_some_and(|(_, tail)| tail.starts_with('Z'))) { return; }
            std::thread::sleep(Duration::from_millis(1));
        }
        panic!("owned compiler descendant still executing after cancellation");
    }
}
