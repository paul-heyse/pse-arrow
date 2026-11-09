// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Prepared image checks supplement original managed process custody across exec.
use super::denied;
use pse_diagnostics::process_facts::{ProcessIdentity, ProcessRead, ProcessReader};

pub(super) fn authenticate_image(
    original: &ProcessIdentity,
    expected: &pse_buildinfo::FileObservation,
    build: pse_ids::ContentHash,
) -> std::io::Result<()> {
    let process = ProcessReader::default();
    let check_process = || match process.process(original.pid) {
        ProcessRead::Present(facts) if facts.identity == *original => Ok(()),
        _ => Err(denied(
            "original payload process changed during image observation",
        )),
    };
    check_process()?;
    expected.verify_scoped(&check_process)?;
    let path = std::path::PathBuf::from(format!("/proc/{}/exe", original.pid));
    let observed = pse_buildinfo::FileObservation::capture_scoped(&path, &check_process)?;
    if observed.sha256 != expected.sha256
        || observed.canonical != expected.canonical
        || observed.mode != expected.mode
    {
        return Err(denied(
            "original payload now executes another prepared image",
        ));
    }
    let mut file = std::fs::File::open(&path)?;
    let mut digest = pse_ids::EncodingHasher::new();
    std::io::copy(&mut file, &mut digest)?;
    if digest.finish().content_hash() != build {
        return Err(denied("original payload executable attestation differs"));
    }
    observed.verify_scoped(&check_process)?;
    check_process()
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        reason = "actual same-PID exec image control"
    )]
    use super::*;
    use std::{
        io::{Read as _, Write as _},
        process::{Command, Stdio},
        time::{Duration, Instant},
    };

    struct Child(std::process::Child);
    impl Child {
        fn fail(&mut self, reason: &str) -> ! {
            let observed = self.0.try_wait();
            let stopped = self.0.kill();
            let joined = self.0.wait();
            let mut stdout = String::new();
            let mut stderr = String::new();
            let stdout_read = self
                .0
                .stdout
                .take()
                .map(|pipe| pipe.take(64 << 10).read_to_string(&mut stdout));
            let stderr_read = self
                .0
                .stderr
                .take()
                .map(|pipe| pipe.take(64 << 10).read_to_string(&mut stderr));
            panic!(
                "{reason}; original child observed={observed:?}, stop={stopped:?}, joined={joined:?}, stdout_read={stdout_read:?}, stderr_read={stderr_read:?}, stdout={stdout:?}, stderr={stderr:?}"
            );
        }
    }
    impl Drop for Child {
        fn drop(&mut self) {
            let _stopped = self.0.kill();
            let _joined = self.0.wait();
        }
    }

    #[test]
    fn engineering_image_authentication_refuses_same_pid_exec_replacement() {
        // The shell executes on its main thread. A libtest function runs on a
        // worker thread, whose exec would invoke Linux de_thread instead.
        let executable = std::path::PathBuf::from("/bin/sh");
        let expected = pse_buildinfo::FileObservation::capture(&executable).unwrap();
        let mut digest = pse_ids::EncodingHasher::new();
        std::io::copy(&mut std::fs::File::open(&executable).unwrap(), &mut digest).unwrap();
        let build = digest.finish().content_hash();
        let mut child = Child(
            Command::new(&executable)
                .args(["-c", "read gate; exec /bin/sleep 30"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap(),
        );
        let ProcessRead::Present(facts) = ProcessReader::default().process(child.0.id()) else {
            panic!("original child process identity unavailable")
        };
        authenticate_image(&facts.identity, &expected, build).unwrap();
        child
            .0
            .stdin
            .take()
            .unwrap()
            .write_all(b"replace\n")
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if Instant::now() >= deadline {
                child.fail("original child did not publish a replacement image");
            }
            match child.0.try_wait() {
                Ok(None) => {}
                _ => child.fail("original child exited before replacement observation"),
            }
            match std::fs::read_link(format!("/proc/{}/exe", child.0.id())) {
                Ok(path) if path != expected.canonical => break,
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => child.fail("original child image observation failed"),
            }
            std::thread::yield_now();
        }
        let ProcessRead::Present(replaced) = ProcessReader::default().process(child.0.id()) else {
            panic!("exec child process identity unavailable")
        };
        assert_eq!(
            facts.identity, replaced.identity,
            "PID/startticks custody survives exec"
        );
        assert!(authenticate_image(&facts.identity, &expected, build).is_err());
    }
}
