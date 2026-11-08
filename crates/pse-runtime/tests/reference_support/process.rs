// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use pse_diagnostics::SupervisedChild;
use std::{
    fmt::Write as _,
    io::{self, Read as _},
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

pub(crate) const DIAGNOSTIC_BYTES: usize = 1 << 20;

/// Budget the fixed scientific and environment records plus each case candidate's comparison.
#[allow(
    dead_code,
    reason = "ordinary reference has its own binary wire extent"
)]
pub(crate) fn legacy_response_limit(candidates: usize) -> io::Result<usize> {
    if !(1..=32).contains(&candidates) {
        return Err(io::Error::other(
            "reference candidate population outside its finite extent",
        ));
    }
    Ok((64 << 10) + candidates * (8 << 10))
}

pub(crate) struct Execution {
    pub(crate) diagnostics: String,
    pub(crate) refusal: Option<(&'static str, String)>,
    _scratch: Option<tempfile::TempDir>,
}

impl Execution {
    #[allow(
        dead_code,
        reason = "the ordinary binary exchange owns its own diagnostic errors"
    )]
    pub(crate) fn failure(
        &self,
        phase: &str,
        error: impl std::fmt::Display,
    ) -> Box<dyn std::error::Error + Send + Sync> {
        format!("reference {phase}: {error}\n{}", self.diagnostics).into()
    }

    #[allow(
        dead_code,
        reason = "the ordinary reference decodes the binary wire format"
    )]
    pub(crate) fn decode_legacy<T: serde::de::DeserializeOwned>(
        &self,
        path: &Path,
        limit: usize,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        let payload =
            read_response(path, limit).map_err(|error| self.failure("response refused", error))?;
        serde_json::from_slice(&payload).map_err(|error| self.failure("response refused", error))
    }
}

pub(crate) fn execute(
    command: &mut Command,
    scratch: tempfile::TempDir,
    diagnostic_bytes: usize,
) -> Execution {
    let work_end = Instant::now() + Duration::from_secs(240);
    let mut child = match SupervisedChild::spawn_captured(
        command.stdin(Stdio::null()),
        diagnostic_bytes / 2,
        diagnostic_bytes - diagnostic_bytes / 2,
        work_end + Duration::from_secs(30),
    ) {
        Ok(child) => child,
        Err(error) => {
            return Execution {
                diagnostics: format!("scratch retained at {}", scratch.keep().display()),
                refusal: Some(("process admission failed", error.to_string())),
                _scratch: None,
            };
        }
    };
    let work = child.wait_captured(None, work_end);
    let settlement = if work.is_ok() {
        child.wait()
    } else {
        child.cancel()
    };
    let raw_exit = child.retained_status();
    let scratch = if raw_exit.is_some() {
        Some(scratch)
    } else {
        let path = scratch.keep();
        return Execution {
            diagnostics: format!(
                "unresolved physical settlement for {:?}; scratch retained at {}\n{}",
                child.identity(),
                path.display(),
                capture_text(child.captured_output()),
            ),
            refusal: Some((
                "physical settlement unresolved",
                format!("work={work:?}; drain={settlement:?}"),
            )),
            _scratch: None,
        };
    };
    let Some(capture) = child.captured_output() else {
        return Execution {
            diagnostics: format!("exit={raw_exit:?}; physical settlement=complete"),
            refusal: Some((
                "diagnostic capture refused",
                "bounded capture absent".to_owned(),
            )),
            _scratch: scratch,
        };
    };
    let overflow = capture.stdout.overflow || capture.stderr.overflow;
    let complete = capture.stdout.complete && capture.stderr.complete;
    let mut diagnostics = capture_text(Some(capture));
    let refusal = match (work, settlement) {
        (Err(error), settlement) => Some((
            "did not complete",
            format!("{error}; capture settlement={settlement:?}"),
        )),
        (Ok(()), Err(error)) => Some(("capture settlement failed", error.to_string())),
        (Ok(()), Ok(status)) if overflow || !complete => Some((
            "diagnostic capture refused",
            format!("exit={status}; settlement=complete; overflow={overflow}; EOF={complete}"),
        )),
        (Ok(()), Ok(status)) if !status.success() => {
            Some(("refused", format!("exit={status}; settlement=complete")))
        }
        (Ok(()), Ok(_)) => None,
    };
    if refusal.is_some() {
        let _ = write!(
            diagnostics,
            "exit={raw_exit:?}; physical settlement=complete"
        );
    }
    Execution {
        diagnostics,
        refusal,
        _scratch: scratch,
    }
}

fn capture_text(capture: Option<&pse_diagnostics::CapturedOutput>) -> String {
    let Some(capture) = capture else {
        return "bounded capture absent".into();
    };
    format!(
        "[stdout overflow={} complete={}]\n{}\n[stderr overflow={} complete={}]\n{}\n",
        capture.stdout.overflow,
        capture.stdout.complete,
        String::from_utf8_lossy(&capture.stdout.prefix),
        capture.stderr.overflow,
        capture.stderr.complete,
        String::from_utf8_lossy(&capture.stderr.prefix),
    )
}

pub(crate) fn read_response(path: &Path, limit: usize) -> io::Result<Vec<u8>> {
    use rustix::fs::{Mode, OFlags, open};
    let fd = open(
        path,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )?;
    let mut file = std::fs::File::from(fd);
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(io::Error::other("reference response is not a regular file"));
    }
    let length = metadata.len();
    let limit = u64::try_from(limit).map_err(io::Error::other)?;
    if length == 0 || length > limit {
        return Err(io::Error::other(
            "reference response outside its finite byte extent",
        ));
    }
    let mut payload = vec![0; usize::try_from(length).map_err(io::Error::other)?];
    file.read_exact(&mut payload)?;
    if file.read(&mut [0])? != 0 {
        return Err(io::Error::other("reference response changed during read"));
    }
    Ok(payload)
}
