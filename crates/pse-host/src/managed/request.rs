// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One-use private process input. Unix byte transport is independent of D-Bus control strings.
use super::payload::{PayloadCommand, PayloadPolicy};
use crate::{HostError, admission::Selection, environment::Environment};
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsString,
    os::unix::ffi::{OsStrExt, OsStringExt},
    path::PathBuf,
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct UnixCommand {
    program: Vec<u8>,
    arguments: Vec<Vec<u8>>,
    cwd: Vec<u8>,
    environment: Vec<(Vec<u8>, Vec<u8>)>,
}

/// No Debug implementation: full secret values are transient transport, never diagnostics.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PrivateRequest {
    version: u32,
    command: UnixCommand,
    selection: Selection,
    policy: PayloadPolicy,
}
impl PrivateRequest {
    pub(crate) fn digest(&self) -> Result<String, HostError> {
        use sha2::Digest;
        let bytes =
            serde_json::to_vec(self).map_err(|_| invalid("private request cannot be encoded"))?;
        Ok(sha2::Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect())
    }
    pub(crate) fn new(
        command: PayloadCommand,
        selection: Selection,
        policy: PayloadPolicy,
    ) -> Result<Self, HostError> {
        validate_command(&command)?;
        Ok(Self {
            version: 1,
            command: UnixCommand {
                program: command.program.as_os_str().as_bytes().to_vec(),
                arguments: command
                    .arguments
                    .iter()
                    .map(|arg| arg.as_bytes().to_vec())
                    .collect(),
                cwd: command.cwd.as_os_str().as_bytes().to_vec(),
                environment: command
                    .environment
                    .iter()
                    .map(|(key, value)| (key.as_bytes().to_vec(), value.as_bytes().to_vec()))
                    .collect(),
            },
            selection,
            policy,
        })
    }
    pub(crate) fn into_parts(
        self,
    ) -> Result<(PayloadCommand, Selection, PayloadPolicy), HostError> {
        if self.version != 1 {
            return Err(invalid("unsupported private request version"));
        }
        let mut environment = Environment::new();
        for (key, value) in self.command.environment {
            if environment
                .insert(OsString::from_vec(key), OsString::from_vec(value))
                .is_some()
            {
                return Err(invalid("duplicate private environment key"));
            }
        }
        let command = PayloadCommand {
            program: PathBuf::from(OsString::from_vec(self.command.program)),
            arguments: self
                .command
                .arguments
                .into_iter()
                .map(OsString::from_vec)
                .collect(),
            cwd: PathBuf::from(OsString::from_vec(self.command.cwd)),
            environment,
        };
        validate_command(&command)?;
        Ok((command, self.selection, self.policy))
    }
}
fn invalid(message: &str) -> HostError {
    HostError::Ownership(format!("managed private request: {message}"))
}
fn validate_command(command: &PayloadCommand) -> Result<(), HostError> {
    if command.program.as_os_str().is_empty()
        || !command.cwd.is_absolute()
        || command.program.as_os_str().as_bytes().contains(&0)
        || command.cwd.as_os_str().as_bytes().contains(&0)
        || command
            .arguments
            .iter()
            .any(|argument| argument.as_bytes().contains(&0))
        || command.environment.iter().any(|(key, value)| {
            key.is_empty()
                || key.as_bytes().contains(&0)
                || key.as_bytes().contains(&b'=')
                || value.as_bytes().contains(&0)
        })
    {
        return Err(invalid(
            "invalid native program, working directory, argument or environment",
        ));
    }
    Ok(())
}

pub(crate) fn encode_path(path: &std::path::Path) -> String {
    path.as_os_str()
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
pub(crate) fn decode_path(value: &str) -> Result<PathBuf, HostError> {
    if value.is_empty() || value.len() % 2 != 0 {
        return Err(invalid("invalid encoded native path"));
    }
    let bytes = value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let text =
                std::str::from_utf8(pair).map_err(|_| invalid("invalid encoded native path"))?;
            u8::from_str_radix(text, 16).map_err(|_| invalid("invalid encoded native path"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let path = PathBuf::from(OsString::from_vec(bytes));
    if !path.is_absolute() || path.as_os_str().as_bytes().contains(&0) {
        return Err(invalid("invalid encoded native path"));
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn private_json_round_trip_preserves_non_utf8_program_argv_environment_and_cwd() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_owned();
        let selection = Selection::load(
            &root,
            &[("PSE_BUILD_MODE".into(), "unbounded".into())]
                .into_iter()
                .collect(),
            None,
            None,
        )
        .unwrap();
        let command = PayloadCommand {
            program: PathBuf::from(OsString::from_vec(b"/bin/program-\xff".to_vec())),
            arguments: vec![OsString::from_vec(b"argument-\xfe".to_vec())],
            cwd: PathBuf::from(OsString::from_vec(b"/directory-\xfd".to_vec())),
            environment: [(
                OsString::from_vec(b"KEY_\xfc".to_vec()),
                OsString::from_vec(b"secret-\xfb".to_vec()),
            )]
            .into_iter()
            .collect(),
        };
        let program = command.program.clone();
        let arguments = command.arguments.clone();
        let cwd = command.cwd.clone();
        let environment = command.environment.clone();
        let request = PrivateRequest::new(
            command,
            selection,
            PayloadPolicy {
                startup: Duration::from_secs(10),
                drain: Duration::from_secs(10),
                write: Duration::from_secs(1),
            },
        )
        .unwrap();
        let bytes = serde_json::to_vec(&request).unwrap();
        let request: PrivateRequest = serde_json::from_slice(&bytes).unwrap();
        let (command, _, _) = request.into_parts().unwrap();
        assert_eq!(command.program, program);
        assert_eq!(command.arguments, arguments);
        assert_eq!(command.cwd, cwd);
        assert_eq!(command.environment, environment);
        assert_eq!(decode_path(&encode_path(&cwd)).unwrap(), cwd);
    }

    #[test]
    fn malformed_control_path_never_becomes_a_lossy_path() {
        for path in ["", "0", "zz", "00", "72656c6174697665", "2f00"] {
            assert!(decode_path(path).is_err());
        }
    }
}
