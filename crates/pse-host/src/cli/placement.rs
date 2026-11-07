// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Enter an authenticated application scope before an unplaced CLI can admit effects.
use crate::{HostError, HostManager};
use std::{ffi::OsString, os::unix::process::CommandExt as _, process::Command, time::Duration};

const REENTRY_PARENT: &str = "PSE_SCOPE_REENTRY";
// This shell waits for a foreground child. Unlike an async list, it does not
// redirect stdin or ignore SIGINT for the child. Every caller argument is data.
const WAIT_FOR_ORIGINAL: &str = "PSE_SCOPE_REENTRY=$$ \"$@\"; status=$?; exit \"$status\"";

pub(super) fn ensure(
    arguments: &[OsString],
    runtime: &tokio::runtime::Runtime,
) -> Result<(), HostError> {
    let membership = runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(5), async {
            let manager = HostManager::connect_user(None).await?;
            manager.current_membership().await
        })
        .await
        .map_err(|_| HostError::Deadline("original user scope authentication".into()))?
    });
    let error = match membership {
        Ok(_) => return Ok(()),
        Err(error) => error,
    };
    let outside_manager = matches!(&error, HostError::Manager(zbus::Error::MethodError(name, _, _))
        if name.as_str() == "org.freedesktop.systemd1.NoUnitForPID");
    // This marker can only suppress another launch. A child always authenticates
    // its actual membership above before reaching the ordinary dispatch.
    let reentry = std::env::var(REENTRY_PARENT)
        .ok()
        .and_then(|value| value.parse::<i32>().ok())
        == rustix::process::getppid().map(rustix::process::Pid::as_raw_pid);
    if !outside_manager || reentry {
        return Err(error);
    }
    let executable = std::env::current_exe()?;
    let error = Command::new("systemd-run")
        .args(["--user", "--scope", "--quiet", "--collect"])
        .args(["/bin/sh", "-c", WAIT_FOR_ORIGINAL, "pse-launch"])
        .arg(executable)
        .args(arguments)
        .exec();
    Err(HostError::Ownership(format!(
        "could not enter a user application scope before admission: {error}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Write as _, process::Stdio};

    #[test]
    fn waiting_parent_preserves_stdin_literal_argv_and_original_exit() {
        let mut parent = Command::new("/bin/sh")
            .args([
                "-c",
                WAIT_FOR_ORIGINAL,
                "pse-launch",
                "/bin/sh",
                "-c",
                "[ \"$PSE_SCOPE_REENTRY\" = \"$PPID\" ] || exit 91; cat; printf '%s\\n' \"$1\"; exit 23",
                "payload",
                "one ; two",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        parent.stdin.take().unwrap().write_all(b"stdin\n").unwrap();
        let output = parent.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(23));
        assert_eq!(output.stdout, b"stdin\none ; two\n");
    }
}
