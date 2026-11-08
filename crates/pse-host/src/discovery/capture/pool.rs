// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Checkout-local immutable test images. Private invocation links are live leases.
use super::{Executable, FileIdentity, invalid, private_directory, read_file};
use crate::correctness::CorrectnessError;
use rustix::fs::{FlockOperation, Mode, OFlags, flock, open};
use std::{
    fs::{self, File},
    io,
    os::unix::fs::{DirBuilderExt, MetadataExt},
    path::{Path, PathBuf},
};

fn location(path: &Path) -> Option<PathBuf> {
    let inventory = path.ancestors().find(|part| {
        part.file_name().is_some_and(|name| name == "inventory")
            && part
                .parent()
                .and_then(Path::file_name)
                .is_some_and(|name| name == "workflow")
    })?;
    let invocation = inventory.parent()?.parent()?;
    let name = invocation.file_name()?.to_str()?;
    if uuid::Uuid::parse_str(name).ok()?.to_string() != name {
        return None;
    }
    let invocations = invocation.parent()?;
    (invocations.file_name()? == "invocations")
        .then(|| invocations.parent().map(|root| root.join("test-images")))?
}

pub(super) struct Pool {
    root: PathBuf,
    directory: File,
}

impl Pool {
    pub(super) fn lock(path: &Path, create: bool) -> Result<Option<Self>, CorrectnessError> {
        let Some(root) = location(path) else {
            return Ok(None);
        };
        if create {
            private_directory(
                root.parent()
                    .ok_or_else(|| invalid("image store lacks checkout state"))?,
            )?;
            match fs::DirBuilder::new().mode(0o700).create(&root) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error.into()),
            }
        }
        let directory = match open(
            &root,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        ) {
            Ok(fd) => File::from(fd),
            Err(rustix::io::Errno::NOENT) if !create => return Ok(None),
            Err(error) => return Err(io::Error::from(error).into()),
        };
        let meta = directory.metadata()?;
        if meta.uid() != rustix::process::geteuid().as_raw() || meta.mode() & 0o777 != 0o700 {
            return Err(invalid("test image store lacks private custody"));
        }
        flock(&directory, FlockOperation::LockExclusive).map_err(io::Error::from)?;
        Ok(Some(Self { root, directory }))
    }

    pub(super) fn object(&self, sha256: &str) -> Result<PathBuf, CorrectnessError> {
        if sha256.len() != 64
            || !sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(invalid("test image digest is invalid"));
        }
        Ok(self.root.join(sha256))
    }

    pub(super) fn link(&self, object: &Path, destination: &Path) -> Result<(), CorrectnessError> {
        fs::hard_link(object, destination)?;
        Ok(())
    }

    pub(super) fn validate_link(
        &self,
        executable: &Executable,
        image: &File,
    ) -> Result<(), CorrectnessError> {
        let canonical = read_file(&self.object(&executable.sha256)?)?;
        if FileIdentity::read(&canonical)? != FileIdentity::read(image)? {
            return Err(invalid(
                "test image lease differs from the original immutable image",
            ));
        }
        Ok(())
    }

    pub(super) fn collect(&self, sha256: &str) -> Result<(), CorrectnessError> {
        let path = self.object(sha256)?;
        let file = match read_file(&path) {
            Ok(file) => file,
            Err(CorrectnessError::Io(error)) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(());
            }
            Err(error) => return Err(error),
        };
        let meta = file.metadata()?;
        if meta.uid() != rustix::process::geteuid().as_raw() || meta.mode() & 0o777 != 0o500 {
            return Err(invalid("test image object lacks private custody"));
        }
        if meta.nlink() == 1 {
            fs::remove_file(path)?;
            self.directory.sync_all()?;
        }
        Ok(())
    }

    pub(super) fn sync(&self) -> io::Result<()> {
        self.directory.sync_all()
    }
}

pub(crate) fn collect_unused(path: &Path) -> Result<(), CorrectnessError> {
    let Some(pool) = Pool::lock(path, false)? else {
        return Ok(());
    };
    for entry in fs::read_dir(&pool.root)? {
        let entry = entry?;
        let name = entry.file_name();
        let sha256 = name
            .to_str()
            .ok_or_else(|| invalid("test image object has an invalid name"))?;
        pool.collect(sha256)?;
    }
    Ok(())
}
