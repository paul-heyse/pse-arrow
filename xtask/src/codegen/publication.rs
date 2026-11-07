// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Guarded publication of the original generator's exact owned file population.
use super::{GeneratedTree, inventory, reject_symlinks, validate_tree};
use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, Permissions},
    io::{self, Read as _, Write as _},
    path::{Path, PathBuf},
};

type Identity = [u8; 32];

struct ObservedFile {
    identity: Identity,
    permissions: Permissions,
}

fn observe(root: &Path, relative: &Path) -> Result<Option<ObservedFile>> {
    reject_symlinks(root, relative)?;
    let path = root.join(relative);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.is_file() => metadata,
        Ok(_) => bail!(
            "generated destination is not a regular file: {}",
            path.display()
        ),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).with_context(|| format!("inspecting {}", path.display())),
    };
    let mut file = File::open(&path).with_context(|| format!("reading {}", path.display()))?;
    let mut digest = Sha256::new();
    let mut buffer = [0; 32 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(Some(ObservedFile {
        identity: digest.finalize().into(),
        permissions: metadata.permissions(),
    }))
}

struct Entry<'a> {
    relative: PathBuf,
    before: Option<Identity>,
    after: Option<StagedFile<'a>>,
}

struct StagedFile<'a> {
    identity: Identity,
    bytes: &'a [u8],
}

impl Entry<'_> {
    fn pending(&self, current: Option<&ObservedFile>) -> Result<bool> {
        let current = current.map(|file| file.identity);
        if current == self.after.as_ref().map(|file| file.identity) {
            return Ok(false);
        }
        if current == self.before {
            return Ok(true);
        }
        bail!(
            "generated destination changed since staging: {}",
            self.relative.display()
        )
    }

    fn publish(&self, root: &Path) -> Result<()> {
        let current = observe(root, &self.relative)?;
        if !self.pending(current.as_ref())? {
            return Ok(());
        }
        let destination = root.join(&self.relative);
        let parent = destination
            .parent()
            .context("generated file has no parent")?;
        match &self.after {
            Some(file) => {
                fs::create_dir_all(parent)?;
                let mut builder = tempfile::Builder::new();
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt as _;
                    builder.permissions(Permissions::from_mode(0o666));
                }
                let mut staged = builder.tempfile_in(parent)?;
                staged.write_all(file.bytes)?;
                if let Some(original) = &current {
                    // Preserve the existing writer's requirement to open the old file for writing.
                    File::options().write(true).open(&destination)?;
                    staged
                        .as_file()
                        .set_permissions(original.permissions.clone())?;
                }
                staged.as_file().sync_all()?;
                if !self.pending(observe(root, &self.relative)?.as_ref())? {
                    return Ok(());
                }
                if current.is_some() {
                    staged.persist(&destination).map_err(|error| error.error)?;
                } else {
                    staged
                        .persist_noclobber(&destination)
                        .map_err(|error| error.error)?;
                }
            }
            None => fs::remove_file(&destination)?,
        }
        #[cfg(unix)]
        File::open(parent)?.sync_all()?;
        Ok(())
    }
}

/// Bytes borrowed from the generator, with the selected checkout's owned preimages.
/// Entries include unchanged files so a later conflict cannot leave a mixed tree.
struct Delta<'a> {
    tree: &'a GeneratedTree,
    entries: Vec<Entry<'a>>,
}

impl<'a> Delta<'a> {
    fn stage(root: &Path, tree: &'a GeneratedTree) -> Result<Self> {
        validate_tree(tree)?;
        let mut existing = inventory(root, tree)?;
        let mut entries = Vec::with_capacity(tree.files.len().max(existing.len()));
        for (relative, bytes) in &tree.files {
            existing.remove(relative);
            entries.push(Entry {
                relative: relative.clone(),
                before: observe(root, relative)?.map(|file| file.identity),
                after: Some(StagedFile {
                    identity: Sha256::digest(bytes).into(),
                    bytes,
                }),
            });
        }
        for relative in existing {
            let before = observe(root, &relative)?.map(|file| file.identity);
            entries.push(Entry {
                relative,
                before,
                after: None,
            });
        }
        Ok(Self { tree, entries })
    }

    fn apply(&self, root: &Path) -> Result<()> {
        let mut unexpected = inventory(root, self.tree)?;
        let mut pending = Vec::new();
        for entry in &self.entries {
            unexpected.remove(&entry.relative);
            if entry.pending(observe(root, &entry.relative)?.as_ref())? {
                pending.push(entry);
            }
        }
        if let Some(path) = unexpected.first() {
            bail!(
                "generated output appeared since staging: {}",
                path.display()
            );
        }
        for entry in pending {
            entry
                .publish(root)
                .with_context(|| format!("publishing {}", entry.relative.display()))?;
        }
        Ok(())
    }
}

pub(super) fn write_tree(root: &Path, tree: &GeneratedTree) -> Result<()> {
    // Serialize generator publications in this checkout without a tracked lock file.
    #[cfg(unix)]
    let _lease = {
        let directory = File::open(root)?;
        directory.lock()?;
        directory
    };
    Delta::stage(root, tree)?.apply(root)
}

#[cfg(test)]
mod tests;
