// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The API acquisition owns its private Cargo target until the original native root drains.
use super::{Acquisition, Context};
use crate::{
    HostError,
    envelope::CliInvocationId,
    environment::Environment,
    invocation::{InvocationMetadata, InvocationStore},
    paths::CheckoutState,
    probe::{OutputRelease, outputs},
};
use rustix::fs::{self as unix, FlockOperation, Mode, OFlags};
use serde::{Deserialize, Serialize};
use std::{
    ffi::{OsStr, OsString},
    fs::File,
    io::{Read as _, Write as _},
    os::unix::fs::MetadataExt as _,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    version: u32,
    invocation: CliInvocationId,
    context: String,
    selection: String,
    target_identity: String,
}

#[derive(Debug, Deserialize)]
struct Operation {
    command: Vec<OsString>,
    cwd: OsString,
    detached: bool,
}

pub(crate) struct Target {
    acquisition: File,
    receipt: Receipt,
    path: PathBuf,
}

fn refusal(message: &str) -> HostError {
    HostError::Ownership(message.into())
}

fn lock(directory: File) -> Result<File, HostError> {
    let metadata = directory.metadata()?;
    if metadata.uid() != rustix::process::geteuid().as_raw() || metadata.mode() & 0o777 != 0o700 {
        return Err(refusal("API acquisition directory lacks private custody"));
    }
    unix::flock(&directory, FlockOperation::NonBlockingLockExclusive)
        .map_err(std::io::Error::from)?;
    Ok(directory)
}

fn read_private(directory: &File, name: &std::ffi::CStr) -> Result<Vec<u8>, HostError> {
    let mut file: File = unix::openat(
        directory,
        name,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(std::io::Error::from)?
    .into();
    let facts = file.metadata()?;
    if !facts.is_file()
        || facts.uid() != rustix::process::geteuid().as_raw()
        || facts.mode() & 0o077 != 0
    {
        return Err(refusal("API acquisition record lacks private custody"));
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

impl Target {
    pub(crate) fn prepare(
        acquisition: &Acquisition,
        invocation: &CliInvocationId,
    ) -> Result<Self, HostError> {
        let directory: File = unix::open(
            &acquisition.directory,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(std::io::Error::from)?
        .into();
        let directory = lock(directory)?;
        unix::mkdirat(&directory, c"target", Mode::RUSR | Mode::WUSR | Mode::XUSR)
            .map_err(std::io::Error::from)?;
        let target = outputs::directory(&directory, c"target")?;
        let receipt = Receipt {
            version: 1,
            invocation: invocation.clone(),
            context: acquisition.context.key().map_err(|error| HostError::Api(Box::new(error)))?,
            selection: acquisition.query.clone(),
            target_identity: outputs::identity(&target)?,
        };
        let mut record: File = unix::openat(
            &directory,
            c"build-output.json",
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::CLOEXEC,
            Mode::RUSR,
        )
        .map_err(std::io::Error::from)?
        .into();
        record.write_all(&serde_json::to_vec(&receipt).map_err(std::io::Error::other)?)?;
        record.sync_all()?;
        directory.sync_all()?;
        Ok(Self {
            acquisition: directory,
            receipt,
            path: acquisition.target_directory.clone(),
        })
    }

    /// The performing owner already holds the acquisition lease while committing the artifact.
    /// A borrowed operation may leave the target for explicit retry after the enclosing drain.
    pub(crate) async fn release_if_drained(
        &self,
        root: &Path,
        environment: &Environment,
    ) -> Result<(), HostError> {
        if crate::foreground::observe_recorded_drain(root, &self.receipt.invocation, environment)
            .await?
        {
            self.output(Some(&self.receipt.target_identity))?;
        }
        Ok(())
    }

    fn output(&self, release: Option<&str>) -> Result<OutputRelease, HostError> {
        if release.is_some_and(|expected| expected != self.receipt.target_identity) {
            return Err(refusal("API build output identity differs from the selected release"));
        }
        Ok(outputs::target_output(
            &self.acquisition,
            self.path.clone(),
            Some(&self.receipt.target_identity),
            release.is_some(),
        )?)
    }
}

fn original_target(
    operation: &InvocationMetadata<Operation>,
    root: &Path,
    state: &CheckoutState,
) -> Result<PathBuf, HostError> {
    let original = &operation.selection.operation;
    if !matches!(operation.selection.command.as_str(), "api prepare" | "api extract-examples")
        || Path::new(&original.cwd) != root
        || original.detached
    {
        return Err(refusal("invocation is not an original foreground API acquisition"));
    }
    let argv = original.command.split(|argument| argument == OsStr::new("--")).next().unwrap_or(&[]);
    let mut paths = argv.windows(2).filter(|pair| pair[0] == OsStr::new("--target-dir"));
    let Some(path) = paths.next().map(|pair| PathBuf::from(&pair[1])) else {
        return Err(refusal("original API acquisition does not name a Cargo target"));
    };
    if paths.next().is_some() || path.strip_prefix(state.api()).is_err() {
        return Err(refusal("original API acquisition target is outside its checkout owner"));
    }
    Ok(path)
}

pub(crate) async fn inspect_or_release(
    root: &Path,
    environment: &Environment,
    selection: &str,
    invocation: &CliInvocationId,
    release: Option<&str>,
) -> Result<OutputRelease, HostError> {
    let root = root.canonicalize()?;
    let state = CheckoutState::resolve(&root, environment)?;
    let store = InvocationStore::new(state.clone());
    let operation = store.metadata(invocation)?;
    let path = original_target(&operation, &root, &state)?;
    let relative = path.strip_prefix(&state.root).map_err(|_| refusal("unknown API target"))?;
    let parts: Vec<_> = relative.components().map(|part| part.as_os_str()).collect();
    let [api, context, acquisition, target] = parts.as_slice() else {
        return Err(refusal("invalid API acquisition path"));
    };
    if *api != "api"
        || *target != "target"
        || context.as_encoded_bytes().len() != 64
        || !context.as_encoded_bytes().iter().all(u8::is_ascii_hexdigit)
        || acquisition.to_str().and_then(|value| uuid::Uuid::parse_str(value).ok()).is_none()
    {
        return Err(refusal("invalid API acquisition identity"));
    }
    let checkout: File = unix::open(
        &state.root,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(std::io::Error::from)?
    .into();
    let directory = outputs::directory(&checkout, *api)?;
    let directory = outputs::directory(&directory, *context)?;
    let directory = lock(outputs::directory(&directory, *acquisition)?)?;
    let receipt: Receipt = serde_json::from_slice(&read_private(&directory, c"build-output.json")?)
        .map_err(std::io::Error::other)?;
    let recorded: Context = serde_json::from_slice(&read_private(&directory, c"context.json")?)
        .map_err(std::io::Error::other)?;
    if receipt.version != 1
        || &receipt.invocation != invocation
        || receipt.selection != selection
        || OsStr::new(&receipt.context) != *context
        || recorded.source_root != root
        || recorded.key().map_err(|error| HostError::Api(Box::new(error)))? != receipt.context
    {
        return Err(refusal("API Cargo output differs from its original acquisition receipt"));
    }
    if !crate::foreground::observe_recorded_drain(&root, invocation, environment).await? {
        return Err(refusal("original API native root remains live or unsettled"));
    }
    Target { acquisition: directory, receipt, path }.output(release)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, os::unix::fs::DirBuilderExt as _};

    #[test]
    fn target_retirement_holds_its_lease_and_rejects_a_recreated_target() {
        let temporary = tempfile::tempdir().unwrap();
        let parent = temporary.path().join("acquisition");
        fs::DirBuilder::new()
            .mode(0o700)
            .create(&parent)
            .unwrap();
        let path = parent.join("target");
        fs::create_dir(&path).unwrap();
        fs::write(path.join("artifact"), b"compiled output").unwrap();
        fs::write(parent.join("document.json"), b"published document").unwrap();
        let directory = lock(File::open(&parent).unwrap()).unwrap();
        let target = outputs::directory(&directory, c"target").unwrap();
        let selected = Target {
            acquisition: directory,
            receipt: Receipt {
                version: 1,
                invocation: CliInvocationId::new(),
                context: "context".into(),
                selection: "example".into(),
                target_identity: outputs::identity(&target).unwrap(),
            },
            path: path.clone(),
        };
        assert!(lock(File::open(&parent).unwrap()).is_err());
        let preview = selected.output(None).unwrap();
        assert!(preview.allocated_bytes > 0);
        let identity = preview.identity.as_deref();
        let moved = parent.join("moved");
        fs::rename(&path, &moved).unwrap();
        fs::create_dir(&path).unwrap();
        assert!(selected.output(identity).is_err());
        fs::remove_dir(&path).unwrap();
        fs::rename(&moved, &path).unwrap();
        assert!(selected.output(identity).unwrap().released);
        assert!(!selected.output(identity).unwrap().released);
        assert_eq!(fs::read(parent.join("document.json")).unwrap(), b"published document");
    }
}
