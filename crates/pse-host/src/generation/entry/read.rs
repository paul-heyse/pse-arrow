// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Process-held protection for reads, promoted before any retained effect or path handoff.
use super::*;

/// A prepared generation held for this process's bounded reads. Dropping this guard only
/// closes its shared kernel lock. It grants no descendant or exported-path protection;
/// those obligations require successful durable promotion before they are created.
#[derive(Debug)]
pub struct GenerationReadGuard {
    _lock: File,
    pub(super) tools: PathBuf,
    generation: String,
    manifest: GenerationManifest,
}
impl GenerationReadGuard {
    /// Authenticate a fixed generation under publication exclusion and retain its shared
    /// use lock. This creates no durable claim, so an interrupted passive reader leaves
    /// no unresolved resource assertion behind.
    pub fn acquire(tools: &Path, generation: &str) -> Result<Self, HostError> {
        if !component(generation) {
            return Err(invalid("generation name is not a path component"));
        }
        let _update = update_lock(tools)?;
        let (lock, _) = use_paths(tools, generation)?;
        let lock = owned_file(&lock, true)?;
        rustix::fs::flock(&lock, rustix::fs::FlockOperation::LockShared)
            .map_err(std::io::Error::from)?;
        let manifest = GenerationManifest::load(&tools.join(generation))?;
        Ok(Self {
            _lock: lock,
            tools: tools.to_owned(),
            generation: generation.into(),
            manifest,
        })
    }

    /// Authenticated bytes for the selected prepared image, independent of current source.
    pub fn manifest(&self) -> &GenerationManifest {
        &self.manifest
    }

    /// Establish and synchronize an original durable claim while this shared guard still
    /// excludes removal. The caller must keep the returned owner until actual obligations
    /// settle or transfer it to their authenticated native owner. Failure authorizes no effect.
    /// Recheck prepared integrity rather than treating a shared use lock as a mutation seal.
    pub fn promote(&self) -> Result<GenerationUse, HostError> {
        let usage = GenerationUse::acquire(&self.tools, &self.generation)?;
        if usage.manifest() != &self.manifest {
            let operation = invalid("generation manifest changed before durable promotion");
            return Err(match usage.settle() {
                Ok(()) => operation,
                Err(cleanup) => HostError::Settlement {
                    operation: Box::new(operation),
                    cleanup: Box::new(cleanup),
                },
            });
        }
        Ok(usage)
    }
}

#[cfg(test)]
mod tests;
