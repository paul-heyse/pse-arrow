// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Cooperating source protocol over the existing authenticated command owner.
use super::{SourceRecordingLaunchDocument, SourceRecordingLaunchRequest};
use crate::tooling::capture::{SourceRecordingAcknowledgement, SourceRecordingRequest};
use crate::observation_detail::{CaptureError};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    os::{fd::AsFd, unix::fs::{MetadataExt, OpenOptionsExt}},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, atomic::{AtomicBool, Ordering}},
    thread::JoinHandle,
    time::{Duration, Instant},
};
fn invalid(message: impl Into<String>) -> CaptureError { CaptureError::Invalid(message.into()) }
fn bounded_document<T: serde::de::DeserializeOwned, const N: u32>(path: &Path) -> Result<T, CaptureError> {
    let mut bytes = Vec::new();
    File::open(path)?.take(crate::tooling::capture::SOURCE_RECORDING_PROTOCOL_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > crate::tooling::capture::SOURCE_RECORDING_PROTOCOL_BYTES { return Err(invalid("source recording protocol document extent")); }
    pse_model::document::decode_versioned::<T,N>(&bytes).map_err(|error| invalid(error.to_string()))
}
struct LaunchClaim { path: PathBuf, device: u64, inode: u64, _file: File }
impl LaunchClaim {
    fn acquire(acknowledgement: &Path) -> Result<Self, CaptureError> {
        let name = acknowledgement.file_name().ok_or_else(|| invalid("acknowledgement has no file name"))?;
        let mut claim_name = name.to_os_string(); claim_name.push(".launch");
        let path = acknowledgement.with_file_name(claim_name);
        let file = OpenOptions::new().write(true).create_new(true).mode(0o600).open(&path)?;
        let metadata = file.metadata()?;
        let mut claim = Self { path, device: metadata.dev(), inode: metadata.ino(), _file: file };
        writeln!(claim._file, "launch owner pid={}", std::process::id())?;
        claim._file.flush()?;
        Ok(claim)
    }
}
impl Drop for LaunchClaim {
    fn drop(&mut self) {
        if std::fs::symlink_metadata(&self.path).is_ok_and(|metadata| metadata.dev() == self.device && metadata.ino() == self.inode) {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}
#[derive(Default)]
struct LogResult { bytes: u64, truncated: bool, errors: Vec<String> }
struct LogDrain { stop: Arc<AtomicBool>, worker: Option<JoinHandle<LogResult>> }
impl LogDrain {
    fn start<P: Read + AsFd + Send + 'static>(mut pipe: P, path: PathBuf, maximum: u64) -> Result<Self, CaptureError> {
        let flags = rustix::fs::fcntl_getfl(&pipe).map_err(std::io::Error::from)?;
        rustix::fs::fcntl_setfl(&pipe, flags | rustix::fs::OFlags::NONBLOCK).map_err(std::io::Error::from)?;
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let worker = std::thread::Builder::new().name("pse-launch-log".into()).spawn(move || {
            let mut result = LogResult::default();
            let mut sink = match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => Some(file), Err(error) => { result.errors.push(format!("command log prefix unavailable: {error}")); None }
            };
            let identity = sink.as_ref().and_then(|file| file.metadata().ok())
                .map(|metadata| (metadata.dev(), metadata.ino()));
            let mut buffer = [0_u8; 65536];
            loop {
                if stopped.load(Ordering::Acquire) { result.truncated = true; break; }
                match pipe.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(length) => {
                        let keep = length.min(usize::try_from(maximum.saturating_sub(result.bytes)).unwrap_or(usize::MAX));
                        if let Some(file) = &mut sink && keep > 0 {
                            match file.write_all(&buffer[..keep]) {
                                Ok(()) => result.bytes += keep as u64,
                                Err(error) => { result.errors.push(format!("command log write unavailable: {error}")); sink = None; }
                            }
                        }
                        result.truncated |= keep < length || sink.is_none();
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {},
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(10)),
                    Err(error) => { result.errors.push(format!("command output drain unavailable: {error}")); result.truncated = true; break; }
                }
            }
            if let Some(mut file) = sink && let Err(error) = file.flush() { result.errors.push(format!("command log flush unavailable: {error}")); }
            match std::fs::symlink_metadata(path) {
                Ok(metadata) if identity == Some((metadata.dev(), metadata.ino())) => {
                    result.truncated |= metadata.len() < result.bytes;
                    result.bytes = result.bytes.min(metadata.len());
                },
                Ok(_) => { result.bytes = 0; result.truncated = true; result.errors.push("command log prefix identity changed".into()); },
                Err(error) => { result.bytes = 0; result.errors.push(format!("command log prefix unavailable: {error}")); }
            }
            result
        })?;
        Ok(Self { stop, worker: Some(worker) })
    }
    fn finished(&self) -> bool { self.worker.as_ref().is_none_or(JoinHandle::is_finished) }
    fn finish(mut self) -> LogResult {
        self.stop.store(true, Ordering::Release);
        match self.worker.take().map(JoinHandle::join) {
            Some(Ok(result)) => result,
            _ => LogResult { truncated: true, errors: vec!["command log drain worker failed".into()], ..Default::default() },
        }
    }
}
impl Drop for LogDrain {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() { let _ = worker.join(); }
    }
}
fn json_value<T: serde::Serialize>(value: &T) -> Result<serde_json::Value, CaptureError> {
    serde_json::to_value(value).map_err(std::io::Error::other).map_err(Into::into)
}
fn acknowledgement(path: &Path, leader: u32, selection: &SourceRecordingRequest) -> Result<SourceRecordingAcknowledgement, CaptureError> {
    let ack: SourceRecordingAcknowledgement = bounded_document::<_,2>(path)?;
    let belongs = ack.pid == leader || i32::try_from(ack.pid).ok().and_then(rustix::process::Pid::from_raw)
        .and_then(|pid| rustix::process::getpgid(Some(pid)).ok()).is_some_and(|group| group.as_raw_nonzero().get() as u32 == leader);
    let mut admitted = ack.selection.clone();
    admitted.expiry_monotonic_ns = selection.detail.expiry_monotonic_ns;
    let detail_matches = json_value(&admitted)? == json_value(&selection.detail)?;
    let review_matches = match (&selection.review, &ack.review) {
        (None,None) => true,
        (Some(request),Some(ack)) => json_value(request)? == json_value(&ack.request)?,
        _ => false,
    };
    if !belongs || !detail_matches || !review_matches { return Err(invalid("source acknowledgement receiver/selection does not belong to this launch")); }
    Ok(ack)
}
/// Launch literal argv under its own finite command clock and cooperating source request.
/// Source activation, command success and actual group drain remain independent facts.
/// # Errors
/// Invalid/oversize protocol, unavailable command/paths, or failed result publication.
pub fn record_launch(request: SourceRecordingLaunchRequest, cancel: &AtomicBool) -> Result<SourceRecordingLaunchDocument, CaptureError> {
    if request.argv.is_empty() || request.argv[0].is_empty() || request.argv.len() > 4096
        || request.argv.iter().any(|value| value.contains('\0'))
        || request.argv.iter().try_fold(0_usize, |n,value| n.checked_add(value.len())).is_none_or(|n| n > 1 << 20)
        || request.command_timeout.is_zero() || request.acknowledgement_timeout.is_zero() || !(2..=1<<30).contains(&request.output_bytes) {
        return Err(invalid("record-launch requires bounded literal argv, positive independent clocks and bounded output"));
    }
    let start = Instant::now();
    let deadline = start.checked_add(request.command_timeout).ok_or_else(|| invalid("command deadline extent"))?;
    let ack_deadline = start.checked_add(request.acknowledgement_timeout).ok_or_else(|| invalid("acknowledgement deadline extent"))?;
    let request_file = std::fs::canonicalize(&request.request_file)?;
    let selection: SourceRecordingRequest = bounded_document::<_,2>(&request_file)?;
    if !selection.acknowledgement.is_absolute() || !selection.detail.output.is_absolute() { return Err(invalid("launch source output and acknowledgement must be absolute explicit paths")); }
    let _claim = LaunchClaim::acquire(&selection.acknowledgement)?;
    if selection.acknowledgement.try_exists()? { return Err(invalid("source acknowledgement already exists; refusing stale substitution")); }
    if cancel.load(Ordering::Acquire) || Instant::now() >= deadline {
        return Err(invalid("source launch cancelled or expired before command construction"));
    }
    std::fs::create_dir_all(request.output.parent().unwrap_or_else(|| Path::new(".")))?;
    std::fs::create_dir(&request.output)?;
    let mut result = SourceRecordingLaunchDocument {
        version: pse_model::document::Version, request,
        acknowledgement: None, command_status: None, timed_out: false, cancelled: false, drained: false,
        stdout_prefix_bytes: 0, stderr_prefix_bytes: 0, logs_truncated: false,
        limitations: vec!["Selection acknowledgement is not final family/managed-child coverage; inspect the original detail stream.".into()],
    };
    let mut command = Command::new(&result.request.argv[0]);
    command.args(&result.request.argv[1..]).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .env(crate::tooling::capture::SOURCE_RECORDING_REQUEST_ENV, request_file);
    if let Some(cwd) = &result.request.cwd { command.current_dir(cwd); }
    use std::os::unix::process::CommandExt as _;
    command.process_group(0);
    if cancel.load(Ordering::Acquire) || Instant::now() >= deadline {
        return Err(invalid("source launch cancelled or expired before spawn"));
    }
    let mut command = crate::tooling::capture::spawn_owned_command(&mut command)?;
    let leader = command.locator().pid;
    let stdout = command.take_stdout().ok_or_else(|| invalid("owned command stdout unavailable"))?;
    let stderr = command.take_stderr().ok_or_else(|| invalid("owned command stderr unavailable"))?;
    let logs = [
        LogDrain::start(stdout, result.request.output.join("stdout.txt"), result.request.output_bytes / 2)?,
        LogDrain::start(stderr, result.request.output.join("stderr.txt"), result.request.output_bytes / 2)?,
    ];
    let mut ack_failed = false;
    loop {
        if !ack_failed && result.acknowledgement.is_none() && Instant::now() <= ack_deadline && selection.acknowledgement.is_file() {
            match acknowledgement(&selection.acknowledgement, leader, &selection) {
                Ok(ack) => result.acknowledgement = Some(ack),
                Err(error) => { ack_failed = true; result.limitations.push(format!("Source acknowledgement unavailable: {error}")); }
            }
        }
        match command.try_wait() {
            Ok(Some(_)) => { result.drained = true; break; },
            Ok(None) => {},
            Err(error) => { result.limitations.push(format!("Owned command drain unavailable: {error}")); break; }
        }
        if cancel.load(Ordering::Acquire) || Instant::now() >= deadline {
            result.cancelled = cancel.load(Ordering::Acquire);
            result.timed_out = Instant::now() >= deadline;
            match command.cancel() {
                Ok(_) => result.drained = true,
                Err(error) => result.limitations.push(format!("Owned command drain unavailable: {error}")),
            }
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    use std::os::unix::process::ExitStatusExt as _;
    result.command_status = command.retained_status().map(|status| status.code().unwrap_or_else(|| -status.signal().unwrap_or(0)));
    let drain_deadline = Instant::now() + Duration::from_secs(2);
    while !logs.iter().all(LogDrain::finished) && Instant::now() < drain_deadline { std::thread::sleep(Duration::from_millis(10)); }
    let [stdout,stderr] = logs.map(LogDrain::finish);
    result.stdout_prefix_bytes = stdout.bytes; result.stderr_prefix_bytes = stderr.bytes;
    result.logs_truncated = stdout.truncated || stderr.truncated;
    result.limitations.extend(stdout.errors); result.limitations.extend(stderr.errors);
    if result.acknowledgement.is_none() { result.limitations.push("No preconstruction acknowledgement observed inside the requested window; source recording is unavailable, independent of command success.".into()); }
    let encoded = serde_json::to_vec(&result).map_err(std::io::Error::other)?;
    OpenOptions::new().write(true).create_new(true).open(result.request.output.join("launch-result.json"))?.write_all(&encoded)?;
    Ok(result)
}

#[cfg(test)]
mod tests;
