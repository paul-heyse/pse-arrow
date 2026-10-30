// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Same-process source refresh and fixed-image re-execution, without GC claim handoff.
use super::*;
use std::{
    os::unix::process::CommandExt,
    process::{Command, Stdio},
};
pub(super) fn identity() -> Result<pse_diagnostics::process_facts::ProcessIdentity, HostError> {
    match pse_diagnostics::process_facts::ProcessReader::default().process(std::process::id()) {
        pse_diagnostics::process_facts::ProcessRead::Present(facts) => Ok(facts.identity),
        _ => Err(invalid(
            "current process identity is unavailable for executable transfer",
        )),
    }
}
pub(super) fn was_reexecuted(inputs: &Environment) -> Result<bool, HostError> {
    let Some(value) = inputs
        .get(OsStr::new("PSE_HOST_REEXEC"))
        .filter(|value| value.len() <= 4096)
    else {
        return Ok(false);
    };
    let Ok(captured) =
        serde_json::from_slice::<pse_diagnostics::process_facts::ProcessIdentity>(value.as_bytes())
    else {
        return Ok(false);
    };
    Ok(captured == identity()?)
}

/// Refresh authorized top-level new work and execute the authenticated replacement image.
/// Native child ownership stays with the calling lifetime; no persisted claim transfers.
pub fn rebuild_and_reexec(
    root: &Path,
    tool: HostTool,
    arguments: &[OsString],
) -> Result<i32, HostError> {
    let root = root.canonicalize()?;
    let stderr = rustix::io::dup(std::io::stderr()).map_err(std::io::Error::from)?;
    let result = Command::new(root.join("scripts/bootstrap-host"))
        .current_dir(&root)
        .stdout(Stdio::from(stderr))
        .status()?;
    if !result.success() {
        return Ok(result.code().unwrap_or(125));
    }
    let reference = prepared(&root)?
        .ok_or_else(|| invalid("bootstrap did not publish a prepared generation"))?;
    let executable = reference.executable(&root, tool)?;
    let marker = String::from_utf8(encoded(&identity()?)?)
        .map_err(|_| invalid("process identity is not UTF-8"))?;
    Err(Command::new(executable)
        .args(arguments)
        .env("PSE_HOST_REEXEC", marker)
        .env_remove("PSE_HOST_GENERATION_HANDOFF")
        .exec()
        .into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reexec_marker_is_bound_to_actual_original_process() {
        let mut inputs = Environment::new();
        assert!(!was_reexecuted(&inputs).unwrap());
        inputs.insert(
            "PSE_HOST_REEXEC".into(),
            String::from_utf8(encoded(&identity().unwrap()).unwrap())
                .unwrap()
                .into(),
        );
        assert!(was_reexecuted(&inputs).unwrap());
        let mut other = identity().unwrap();
        other.start_ticks = other.start_ticks.wrapping_add(1);
        inputs.insert(
            "PSE_HOST_REEXEC".into(),
            String::from_utf8(encoded(&other).unwrap()).unwrap().into(),
        );
        assert!(!was_reexecuted(&inputs).unwrap());
    }
}
