// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit perf process sampling and save-only Samply conversion; no UI/server or target stop.

use super::{
    CaptureError, CaptureOutput, CaptureOutputKind, CaptureRequest, CaptureStatus, Shared,
    document, failure, source_partial, target,
};
use crate::observation::{ProcessCheckpoint, read_checkpoint};
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

/// A live Child is the ownership proof. Never signal a cached PID after reaping it.
struct Tool {
    child: Child,
}
impl Tool {
    fn poll(&mut self) -> std::io::Result<Option<ExitStatus>> {
        self.child.try_wait()
    }
    fn interrupt_and_join(&mut self) -> std::io::Result<(ExitStatus, bool)> {
        if let Some(status) = self.poll()? {
            return Ok((status, false));
        }
        #[cfg(unix)]
        let owner_sigint_sent =
            rustix::process::Pid::from_raw(self.child.id() as i32).is_some_and(|pid| {
                rustix::process::kill_process(pid, rustix::process::Signal::INT).is_ok()
            });
        #[cfg(not(unix))]
        let owner_sigint_sent = false;
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(3) {
            if let Some(status) = self.poll()? {
                return Ok((status, owner_sigint_sent));
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        self.child.kill()?;
        self.child.wait().map(|status| (status, owner_sigint_sent))
    }
}
impl Drop for Tool {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}
struct Sampler {
    tool: Tool,
    raw: PathBuf,
    log: PathBuf,
    finished: Option<ExitStatus>,
    owner_sigint_sent: bool,
    reservation: u64,
}
fn exit_signal(status: ExitStatus) -> Option<i32> {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        status.signal()
    }
    #[cfg(not(unix))]
    {
        let _ = status;
        None
    }
}
pub(super) fn expected_finish(status: ExitStatus, owner_sigint_sent: bool) -> bool {
    status.success() || owner_sigint_sent && exit_signal(status) == Some(2)
}
pub(super) fn append_finish(
    log: &Path,
    limit: u64,
    status: ExitStatus,
    owner_sigint_sent: bool,
) -> std::io::Result<bool> {
    let receipt = format!(
        "\npse-capture collector-exit code={:?} signal={:?} owner_sigint_sent={} expected={}\n",
        status.code(),
        exit_signal(status),
        owner_sigint_sent,
        expected_finish(status, owner_sigint_sent)
    );
    if size(log).saturating_add(receipt.len() as u64) > limit {
        return Ok(false);
    }
    OpenOptions::new()
        .append(true)
        .open(log)?
        .write_all(receipt.as_bytes())?;
    Ok(true)
}
const MAX_FILES: usize = 16 * 6;
struct OwnedFile {
    path: PathBuf,
    reserved: u64,
}
/// All file reservations live independently of the bounded public inventory.
/// Live tools can create at most their two reserved hard-limited files.
pub(super) struct OwnedFiles {
    files: Vec<OwnedFile>,
    limit: u64,
}
impl OwnedFiles {
    pub(super) fn new(limit: u64) -> Self {
        Self {
            files: Vec::new(),
            limit,
        }
    }
    pub(super) fn available(&self) -> u64 {
        self.limit
            .saturating_sub(self.files.iter().map(|file| file.reserved).sum())
    }
    pub(super) fn reserve_pair(
        &mut self,
        first: &Path,
        second: &Path,
        each: u64,
    ) -> Result<(), CaptureError> {
        if first == second
            || self.len() + 2 > MAX_FILES
            || each.saturating_mul(2) > self.available()
            || self
                .files
                .iter()
                .any(|file| file.path == first || file.path == second)
        {
            return Err(CaptureError::Invalid(
                "capture file reservation exhausted or duplicated".into(),
            ));
        }
        self.files.push(OwnedFile {
            path: first.to_owned(),
            reserved: each,
        });
        self.files.push(OwnedFile {
            path: second.to_owned(),
            reserved: each,
        });
        Ok(())
    }
    pub(super) fn seal(&mut self, path: &Path) {
        if let Some(file) = self.files.iter_mut().find(|file| file.path == path) {
            file.reserved = size(path);
        }
    }
    pub(super) fn bytes(&self) -> u64 {
        self.files.iter().map(|file| size(&file.path)).sum()
    }
    pub(super) fn len(&self) -> usize {
        self.files.len()
    }
}
fn size(path: &Path) -> u64 {
    std::fs::metadata(path).map_or(0, |metadata| metadata.len())
}
pub(super) fn output(state: &Shared, path: &Path, kind: CaptureOutputKind, readable: bool) {
    let bytes = size(path);
    let mut state = document(state);
    let entry = CaptureOutput {
        path: path.to_owned(),
        kind,
        bytes,
        readable_prefix_bytes: if readable { bytes } else { 0 },
        readable,
    };
    if let Some(existing) = state.outputs.iter_mut().find(|entry| entry.path == path) {
        *existing = entry;
    } else if state.outputs.len() < MAX_FILES {
        state.outputs.push(entry);
    } else {
        state.loss.refused_items += 1;
    }
}
fn log_prefix(path: &Path) -> String {
    let mut bytes = Vec::new();
    if let Ok(file) = File::open(path) {
        let _ = file.take(512).read_to_end(&mut bytes);
    }
    String::from_utf8_lossy(&bytes).into_owned()
}
pub(super) fn finish_tool_result(
    state: &Shared,
    path: &Path,
    kind: CaptureOutputKind,
    log: &Path,
    result: Result<ExitStatus, CaptureError>,
) -> Result<ExitStatus, CaptureError> {
    // Inventory the actual partial files before propagating an interrupted/failed tool.
    // Only the successful caller may subsequently establish format readability.
    output(state, path, kind, false);
    output(state, log, CaptureOutputKind::ToolLog, log.is_file());
    result.inspect_err(|_| failure(state, format!("capture tool log: {}", log_prefix(log))))
}
fn command(program: &str, limit: u64, log: &Path) -> Result<Command, CaptureError> {
    let mut command = Command::new("prlimit");
    command
        .arg(format!("--fsize={limit}:{limit}"))
        .arg("--")
        .arg(program)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(OpenOptions::new().write(true).create_new(true).open(log)?);
    Ok(command)
}
fn start_sampler(
    request: &CaptureRequest,
    image: &ProcessCheckpoint,
    allowance: u64,
    files: &mut OwnedFiles,
    state: &Shared,
) -> Result<Sampler, CaptureError> {
    // Recheck immediately before spawning; a stale retained operation is not attach authority.
    if !image.locator.is_current() {
        return Err(CaptureError::Invalid(
            "CPU process exited or its PID was reused before attachment".into(),
        ));
    }
    start_sampler_for(request.output.as_path(), image.locator.pid, image.locator.start_ticks, allowance, files, state, None, "perf")
}
fn start_sampler_for(
    output_path: &Path, pid: u32, ticks: Option<u64>, allowance: u64,
    files: &mut OwnedFiles, state: &Shared, control: Option<(&Path, &Path)>, sampler_program: &str,
) -> Result<Sampler, CaptureError> {
    let label = format!(
        "{}-{:016x}",
        pid,
        ticks.unwrap_or(0)
    );
    let raw = output_path.join(format!("{label}.perf.data"));
    let log = output_path.join(format!("{label}.perf.log"));
    // Two independently hard-limited files fit this sampler's reservation.
    let file_limit = allowance / 2;
    files.reserve_pair(&raw, &log, file_limit)?;
    output(state, &raw, CaptureOutputKind::PerfData, false);
    output(state, &log, CaptureOutputKind::ToolLog, false);
    let mut command = command(sampler_program, file_limit, &log)?;
    // Default inheritance covers threads born after attachment, and also forked descendants.
    // Those samples remain inclusive process/task evidence, never exclusive operation CPU.
    command
        .args([
            "record",
            "-e",
            "cpu-clock:u",
            "-F",
            "99",
            "--call-graph",
            "dwarf,32768",
            "--clockid",
            "mono",
            "--no-buildid-cache",
            "-m",
            "1M",
            "--max-size",
        ])
        .arg(format!("{file_limit}B"))
        .arg("-p")
        .arg(pid.to_string())
        .arg("-o")
        .arg(&raw);
    if let Some((control, ack)) = control {
        let arguments: Vec<_> = command.get_args().map(std::ffi::OsStr::to_owned).collect();
        let program = command.get_program().to_owned();
        // Open the control descriptors only in a freshly executed shell. Parent descriptors
        // remain CLOEXEC; there is no pre_exec callback in a multithreaded Python process.
        let mut controlled = Command::new("/bin/sh");
        controlled.args(["-c", "exec 3<> \"$1\"; exec 4<> \"$2\"; shift 2; exec \"$@\" --delay=-1 --control fd:3,4", "pse-perf-control"])
            .arg(control).arg(ack).arg(program).args(arguments)
            .stdin(Stdio::null()).stdout(Stdio::null())
            .stderr(OpenOptions::new().append(true).open(&log)?);
        command = controlled;
    }
    let tool = Tool { child: command.spawn()? };
    Ok(Sampler {
        tool,
        raw,
        log,
        finished: None,
        owner_sigint_sent: false,
        reservation: allowance,
    })
}
fn tool_until(
    mut command: Command,
    timeout: Duration,
    output_path: &Path,
    output_limit: u64,
) -> Result<ExitStatus, CaptureError> {
    let mut tool = Tool {
        child: command.spawn()?,
    };
    let started = Instant::now();
    loop {
        if let Some(status) = tool.poll()? {
            return Ok(status);
        }
        if started.elapsed() >= timeout || size(output_path) >= output_limit {
            let _ = tool.interrupt_and_join();
            return Err(CaptureError::Invalid(
                "capture conversion/decoder exceeded its time or file allowance".into(),
            ));
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}
fn convert(
    sampler: &Sampler,
    state: &Shared,
    timeout: Duration,
    files: &mut OwnedFiles,
) -> Result<(), CaptureError> {
    let allowance = files.available() / 2;
    if allowance < 4096 {
        document(state).loss.output_exhausted = true;
        return Ok(());
    }
    let profile = sampler.raw.with_extension("profile.json.gz");
    let log = sampler.raw.with_extension("samply.log");
    files.reserve_pair(&profile, &log, allowance)?;
    output(state, &profile, CaptureOutputKind::Samply, false);
    output(state, &log, CaptureOutputKind::ToolLog, false);
    let mut command = command("samply", allowance, &log)?;
    command
        .arg("import")
        .arg(&sampler.raw)
        .arg("--save-only")
        .arg("--output")
        .arg(&profile);
    let result = tool_until(command, timeout, &profile, allowance);
    files.seal(&profile);
    files.seal(&log);
    let status = finish_tool_result(state, &profile, CaptureOutputKind::Samply, &log, result)?;
    output(
        state,
        &profile,
        CaptureOutputKind::Samply,
        status.success() && size(&profile) > 0,
    );
    output(state, &log, CaptureOutputKind::ToolLog, true);
    if !status.success() {
        return Err(CaptureError::Invalid(format!(
            "Samply save-only import failed ({status}): {}",
            log_prefix(&log)
        )));
    }
    Ok(())
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Decode {
    Checked,
    Skipped,
}
pub(super) fn symbols(
    raw: &Path,
    state: &Shared,
    timeout: Duration,
    files: &mut OwnedFiles,
) -> Result<Decode, CaptureError> {
    let allowance = (files.available() / 2).min(4 << 20);
    if allowance < 4096 {
        document(state).loss.output_exhausted = true;
        return Ok(Decode::Skipped);
    }
    let path = raw.with_extension("symbols.txt");
    let log = raw.with_extension("symbols.log");
    files.reserve_pair(&path, &log, allowance)?;
    output(state, &path, CaptureOutputKind::Symbols, false);
    output(state, &log, CaptureOutputKind::ToolLog, false);
    let mut command = command("perf", allowance, &log)?;
    // Frame names suffice here; inline addr2line expansion is outside this bounded decoder.
    // The unchanged raw recording and Samply import retain the complete input evidence.
    command
        .args(["script", "--no-inline", "-i"])
        .arg(raw)
        .args(["-F", "ip,sym,dso"])
        .stdout(
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)?,
        );
    let result = tool_until(command, timeout, &path, allowance);
    files.seal(&path);
    files.seal(&log);
    let status = finish_tool_result(state, &path, CaptureOutputKind::Symbols, &log, result)?;
    output(state, &path, CaptureOutputKind::Symbols, status.success());
    output(state, &log, CaptureOutputKind::ToolLog, true);
    if !status.success() {
        return Err(CaptureError::Invalid(format!(
            "perf frame decoding failed ({status}): {}",
            log_prefix(&log)
        )));
    }
    let reader = BufReader::new(File::open(path)?.take(allowance));
    let mut state = document(state);
    for line in reader.split(b'\n') {
        let line = line?;
        let text = String::from_utf8_lossy(&line);
        let Some((address, frame)) = text.trim().split_once(char::is_whitespace) else {
            continue;
        };
        if !address
            .chars()
            .all(|character| character.is_ascii_hexdigit())
        {
            continue;
        }
        let frame = frame.trim();
        if frame.starts_with("[unknown]") || frame.starts_with("0x") || frame.is_empty() {
            state.symbols.unresolved_frames = state.symbols.unresolved_frames.saturating_add(1);
        } else {
            state.symbols.resolved_frames = state.symbols.resolved_frames.saturating_add(1);
            let frame: String = frame.chars().take(256).collect();
            if state.symbols.names.len() < 16 && !state.symbols.names.contains(&frame) {
                state.symbols.names.push(frame);
            }
        }
    }
    state.symbols.checked = true;
    Ok(Decode::Checked)
}
pub(super) fn discover(
    request: &CaptureRequest,
    image: &ProcessCheckpoint,
    selected: &mut Vec<pse_diagnostics::observation::OperationId>,
    paths: &mut Vec<PathBuf>,
    state: &Shared,
) {
    let mut growing = true;
    while growing {
        growing = false;
        for operation in &image.operations {
            if !selected.contains(&operation.id) {
                continue;
            }
            for child in &operation.children {
                if !selected.contains(&child.id) {
                    if selected.len() < 512 {
                        selected.push(child.id);
                        growing = true;
                    } else {
                        let mut state = document(state);
                        state.loss.refused_items = state.loss.refused_items.saturating_add(1);
                        state.loss.source_partial = true;
                    }
                }
                if child.id.process_instance != operation.id.process_instance
                    && let Some(parent) = request.checkpoint.parent()
                {
                    let path = parent.join(format!(
                        "{:016x}-{:016x}.json",
                        child.id.process_instance[0], child.id.process_instance[1]
                    ));
                    if !paths.contains(&path) {
                        if paths.len() < 16 {
                            paths.push(path);
                        } else {
                            let mut state = document(state);
                            state.loss.refused_items = state.loss.refused_items.saturating_add(1);
                            state.loss.source_partial = true;
                        }
                    }
                }
            }
        }
    }
}
pub(super) fn run(
    request: &CaptureRequest,
    initial: ProcessCheckpoint,
    stop: &AtomicBool,
    state: &Shared,
) -> Result<(), CaptureError> {
    document(state).loss.sampling_loss_unknown = true;
    document(state).loss.sampling_inherited_tasks = true;
    let started = Instant::now();
    let mut selected = vec![request.operation];
    let mut paths = vec![request.checkpoint.clone()];
    let mut attached = Vec::<String>::new();
    let mut samplers = Vec::<Sampler>::new();
    let mut files = OwnedFiles::new(request.budget.output_bytes);
    let mut initial = Some(initial);
    let mut last_publish = started;
    loop {
        let mut index = 0;
        while index < paths.len() {
            let image = if index == 0 && initial.is_some() {
                initial.take()
            } else {
                match read_checkpoint(&paths[index]) {
                    Ok(image) => Some(image),
                    Err(error) => {
                        failure(state, error);
                        None
                    }
                }
            };
            if let Some(image) = image {
                if !image
                    .operations
                    .iter()
                    .any(|operation| selected.contains(&operation.id))
                {
                    failure(state, "linked CPU checkpoint has no selected operation");
                    index += 1;
                    continue;
                }
                discover(request, &image, &mut selected, &mut paths, state);
                if !attached.contains(&image.process) && samplers.len() < 16 {
                    attached.push(image.process.clone());
                    let reserved: u64 = samplers.iter().map(|sampler| sampler.reservation).sum();
                    let allowance =
                        (request.budget.output_bytes / 4 * 3).saturating_sub(reserved) / 2;
                    if allowance < 4096 {
                        document(state).loss.output_exhausted = true;
                        break;
                    }
                    match start_sampler(request, &image, allowance, &mut files, state) {
                        Ok(sampler) => {
                            output(state, &sampler.raw, CaptureOutputKind::PerfData, false);
                            output(state, &sampler.log, CaptureOutputKind::ToolLog, false);
                            samplers.push(sampler);
                            document(state).loss.sampling_overlap_possible |= samplers.len() > 1;
                        }
                        Err(error) => failure(state, error),
                    }
                } else if !attached.contains(&image.process) {
                    let mut state = document(state);
                    state.loss.refused_items = state.loss.refused_items.saturating_add(1);
                    state.loss.source_partial = true;
                }
                let entry = target(&image, &selected);
                let mut state = document(state);
                state.loss.source_partial |= source_partial(&image, &selected);
                if let Some(existing) = state
                    .targets
                    .iter_mut()
                    .find(|target| target.process == entry.process)
                {
                    *existing = entry;
                } else if state.targets.len() < 16 {
                    state.targets.push(entry);
                } else {
                    state.loss.refused_items = state.loss.refused_items.saturating_add(1);
                    state.loss.source_partial = true;
                }
            }
            index += 1;
        }
        let mut alive = false;
        for sampler in &mut samplers {
            if sampler.finished.is_none() {
                sampler.finished = sampler.tool.poll()?;
            }
            alive |= sampler.finished.is_none();
        }
        if last_publish.elapsed() >= Duration::from_secs(1) {
            for sampler in &samplers {
                output(state, &sampler.raw, CaptureOutputKind::PerfData, false);
                output(state, &sampler.log, CaptureOutputKind::ToolLog, true);
            }
            document(state).observed_duration = started.elapsed();
            last_publish = Instant::now();
        }
        if files.bytes() >= request.budget.output_bytes {
            document(state).loss.output_exhausted = true;
            break;
        }
        if stop.load(Ordering::Acquire) || started.elapsed() >= request.duration || !alive {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    document(state).observed_duration = started.elapsed();
    finalize(&mut samplers, state, &mut files)
}
fn finalize(samplers: &mut [Sampler], state: &Shared, files: &mut OwnedFiles) -> Result<(), CaptureError> {
    for sampler in samplers.iter_mut() {
        if sampler.finished.is_none() {
            let (status, owner_sigint_sent) = sampler.tool.interrupt_and_join()?;
            sampler.finished = Some(status);
            sampler.owner_sigint_sent = owner_sigint_sent;
        }
        if let Some(status) = sampler.finished {
            match append_finish(
                &sampler.log,
                sampler.reservation / 2,
                status,
                sampler.owner_sigint_sent,
            ) {
                Ok(true) => {}
                Ok(false) => document(state).loss.output_exhausted = true,
                Err(error) => {
                    document(state).loss.output_exhausted = true;
                    failure(state, error);
                }
            }
        }
        files.seal(&sampler.raw);
        files.seal(&sampler.log);
    }
    let finalizing = Instant::now();
    for sampler in samplers.iter() {
        let success = sampler
            .finished
            .is_some_and(|status| expected_finish(status, sampler.owner_sigint_sent));
        output(state, &sampler.raw, CaptureOutputKind::PerfData, false);
        output(state, &sampler.log, CaptureOutputKind::ToolLog, true);
        if !success {
            failure(
                state,
                format!("perf recording failed: {}", log_prefix(&sampler.log)),
            );
        }
        // Preserve raw evidence regardless of conversion/symbol success, including permission failures.
        if size(&sampler.raw) > 0 {
            let remaining = Duration::from_secs(20).saturating_sub(finalizing.elapsed());
            if remaining.is_zero() {
                failure(state, "capture finalization interval exhausted");
                break;
            }
            match symbols(&sampler.raw, state, remaining, files) {
                Ok(Decode::Checked) => {
                    output(state, &sampler.raw, CaptureOutputKind::PerfData, true)
                }
                Ok(Decode::Skipped) => {}
                Err(error) => failure(state, error),
            }
            let remaining = Duration::from_secs(20).saturating_sub(finalizing.elapsed());
            if remaining.is_zero() {
                failure(state, "capture finalization interval exhausted");
                break;
            }
            if let Err(error) = convert(sampler, state, remaining, files) {
                failure(state, error);
            }
        }
    }
    let mut state = document(state);
    if state.symbols.resolved_frames == 0 {
        state.status = CaptureStatus::Failed;
        if state.errors.len() < 16 {
            state.errors.push("no useful resolved frames were established; file creation/decoding alone is insufficient".into());
        }
    } else if !state.errors.is_empty() || state.loss.output_exhausted {
        state.status = CaptureStatus::Partial;
    }
    Ok(())
}

// Only fresh capture-owned FIFO nodes; removal never touches raw evidence or science.
struct ControlPaths([PathBuf; 2]);
impl Drop for ControlPaths {
    fn drop(&mut self) {
        for path in &self.0 { let _ = std::fs::remove_file(path); }
    }
}

/// Start disabled, acknowledge actual sampling activation, then allow caller-owned exec.
pub(super) fn run_launched(
    request: &super::launch::LaunchCaptureRequest,
    locator: &crate::observation::ProcessLocator,
    stop: &AtomicBool, state: &Shared,
    ready: &std::sync::mpsc::SyncSender<Result<(), String>>,
) -> Result<(), CaptureError> {
    run_launched_program(request, locator, stop, state, ready, "perf")
}
pub(super) fn run_launched_program(
    request: &super::launch::LaunchCaptureRequest,
    locator: &crate::observation::ProcessLocator,
    stop: &AtomicBool, state: &Shared,
    ready: &std::sync::mpsc::SyncSender<Result<(), String>>,
    sampler_program: &str,
) -> Result<(), CaptureError> {
    use std::os::unix::fs::OpenOptionsExt;
    let control_path = request.output.join("perf-control.fifo");
    let ack_path = request.output.join("perf-ack.fifo");
    let _control_paths = ControlPaths([control_path.clone(), ack_path.clone()]);
    for path in [&control_path, &ack_path] {
        rustix::fs::mkfifoat(rustix::fs::CWD, path, rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR).map_err(std::io::Error::from)?;
    }
    let open = |path: &Path| OpenOptions::new().read(true).write(true)
        .custom_flags(rustix::fs::OFlags::NONBLOCK.bits() as i32).open(path);
    let mut control = open(&control_path)?;
    let mut ack = open(&ack_path)?;
    let mut files = OwnedFiles::new(request.budget.output_bytes);
    let allowance = request.budget.output_bytes / 4 * 3;
    let mut sampler = start_sampler_for(&request.output, locator.pid, locator.start_ticks,
        allowance, &mut files, state, Some((&control_path, &ack_path)), sampler_program)?;
    let result = (|| {
        control.write_all(b"enable\n")?;
        let started = Instant::now();
        let mut response = Vec::with_capacity(4);
        loop {
            let mut bytes = [0_u8; 4];
            match ack.read(&mut bytes) {
                Ok(0) => {},
                Ok(n) => response.extend_from_slice(&bytes[..n]),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {},
                Err(error) => return Err(error.into()),
            }
            if response == b"ack\n" { break; }
            if !b"ack\n".starts_with(&response) {
                return Err(CaptureError::Invalid("invalid perf enable acknowledgement".into()));
            }
            if sampler.tool.poll()?.is_some() {
                return Err(CaptureError::Invalid(format!("perf exited before enable acknowledgement: {}", log_prefix(&sampler.log))));
            }
            if started.elapsed() >= request.startup_timeout {
                return Err(CaptureError::Invalid("perf startup acknowledgement timed out".into()));
            }
            let remaining = request.startup_timeout.saturating_sub(started.elapsed()).min(Duration::from_millis(50));
            let timeout = rustix::event::Timespec { tv_sec: remaining.as_secs() as i64, tv_nsec: remaining.subsec_nanos() as i64 };
            let mut descriptors = [rustix::event::PollFd::new(&ack, rustix::event::PollFlags::IN)];
            rustix::event::poll(&mut descriptors, Some(&timeout)).map_err(std::io::Error::from)?;
        }
        document(state).loss.sampling_loss_unknown = true;
        document(state).loss.sampling_inherited_tasks = true;
        // The receiver alone owns the command gate. Sending carries no cancellation lease.
        let _ = ready.send(Ok(()));
        let started = Instant::now();
        loop {
            sampler.finished = sampler.tool.poll()?;
            output(state, &sampler.raw, CaptureOutputKind::PerfData, false);
            output(state, &sampler.log, CaptureOutputKind::ToolLog, true);
            document(state).observed_duration = started.elapsed();
            if files.bytes() >= request.budget.output_bytes {
                document(state).loss.output_exhausted = true;
                break;
            }
            if sampler.finished.is_some() || stop.load(Ordering::Acquire) || started.elapsed() >= request.duration { break; }
            std::thread::sleep(Duration::from_millis(50));
        }
        Ok(())
    })();
    // Closing/removing the control endpoints affects only perf, never the gated command.
    drop(control); drop(ack);
    // Capture-owned FIFO nodes are removed by ControlPaths on every return path.
    if let Err(error) = result {
        let message = error.to_string();
        failure(state, error);
        document(state).status = CaptureStatus::Failed;
        let _ = ready.send(Err(message));
    }
    finalize(std::slice::from_mut(&mut sampler), state, &mut files)
}
