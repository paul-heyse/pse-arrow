// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Admission and release for the managed campaign's bounded original checkpoint.
use super::{InvocationDirectory, InvocationStore, directory, evidence_file, invalid, unix_millis, workflow_evidence};
use crate::{HostError, envelope::{ArtifactReference, CliInvocationId, Settlement}, managed::result::ManagedSummary};
use rustix::fs::{FlockOperation, Mode, OFlags, flock, openat, unlinkat};
use schemars::JsonSchema;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs::{self, File, OpenOptions}, io::{self, Read}, os::unix::fs::{MetadataExt, OpenOptionsExt}, path::{Path, PathBuf}, time::SystemTime};

pub(crate) const SNAPSHOT: &str = "conformance-progress.json";
const STAGING: &str = "conformance-progress.pending";
const RESERVATION_BYTES: u64 = 4 * 1024 * 1024;
const CHECKOUT_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const WEEK_MS: u64 = 7 * 24 * 60 * 60 * 1000;
const SEAL: &str = "conformance_worker_drain.json";

#[derive(Debug, Serialize, JsonSchema)]
/// Exact release preview for one original managed campaign observation.
pub struct ConformanceRetirement {
    invocation: CliInvocationId,
    disposition: &'static str,
    allocated_bytes: u64,
    eligible_after_unix_ms: Option<u64>,
    holds: BTreeMap<String, String>,
    release: Option<String>,
}

fn lock(path: &Path, mode: FlockOperation) -> Result<File, HostError> {
    directory(path, false)?;
    let opened = OpenOptions::new().read(true)
        .custom_flags((OFlags::DIRECTORY | OFlags::NOFOLLOW).bits() as i32).open(path)?;
    flock(&opened, mode).map_err(io::Error::from)?;
    let descriptor = opened.metadata()?;
    let entry = fs::symlink_metadata(path)?;
    if (descriptor.dev(), descriptor.ino()) != (entry.dev(), entry.ino()) {
        return Err(invalid("conformance evidence owner changed during access"));
    }
    Ok(opened)
}

struct CampaignMember {
    identity: crate::artifacts::DescriptorIdentity,
    allocated: u64,
    fingerprint: String,
    bytes: Vec<u8>,
}

impl CampaignMember {
    fn committed_by(&self, name: &str, artifacts: &[ArtifactReference]) -> bool {
        artifacts.iter().any(|artifact| {
            artifact.kind == "conformance_checkpoint" && artifact.location == name
                && artifact.digest.as_deref() == Some(self.fingerprint.as_str())
        })
    }
}

fn selected_file(owner: &File, name: &str) -> Result<Option<CampaignMember>, HostError> {
    let file = match openat(owner, name, OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC, Mode::empty()) {
        Ok(fd) => File::from(fd),
        Err(rustix::io::Errno::NOENT) => return Ok(None),
        Err(error) => return Err(io::Error::from(error).into()),
    };
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.nlink() != 1
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o077 != 0
        || metadata.len() > RESERVATION_BYTES / 2 {
        return Err(invalid("conformance evidence is not a bounded original owner file"));
    }
    let identity = crate::artifacts::descriptor_identity(&file)?;
    let mut bytes = Vec::new();
    (&file).take(RESERVATION_BYTES / 2 + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 != metadata.len() || crate::artifacts::descriptor_identity(&file)? != identity {
        return Err(invalid("conformance evidence changed while calculating release identity"));
    }
    let fingerprint = format!("{}:{}:{}", identity.device, identity.inode, crate::managed::workflow::digest(&bytes));
    Ok(Some(CampaignMember {
        identity,
        allocated: metadata.blocks().saturating_mul(512),
        fingerprint,
        bytes,
    }))
}

impl InvocationDirectory {
    /// Read provisional bytes, or authenticate them against a terminal's exact commitment.
    pub(crate) fn conformance_checkpoint(
        &self,
        terminal: Option<&[ArtifactReference]>,
    ) -> Result<Option<Vec<u8>>, HostError> {
        if !self.is_conformance() { return Ok(None); }
        let owner = lock(self.path(), FlockOperation::LockShared)?;
        let Some(member) = selected_file(&owner, SNAPSHOT)? else { return Ok(None); };
        if terminal.is_some_and(|artifacts| !member.committed_by(SNAPSHOT, artifacts)) {
            return Err(invalid("campaign checkpoint differs from the original terminal commitment"));
        }
        Ok(Some(member.bytes))
    }

    /// Freeze the exact checkpoint after the original child has been reaped.
    pub(crate) fn conformance_artifacts(&self) -> Result<Vec<ArtifactReference>, HostError> {
        if !self.is_conformance() { return Ok(Vec::new()); }
        let owner = lock(self.path(), FlockOperation::LockShared)?;
        let mut artifacts = Vec::new();
        for name in [SNAPSHOT, STAGING] {
            if let Some(member) = selected_file(&owner, name)? {
                artifacts.push(ArtifactReference {
                    kind: "conformance_checkpoint".into(),
                    location: name.into(),
                    digest: Some(member.fingerprint),
                });
            }
        }
        Ok(artifacts)
    }

    /// Only the authenticated detached finalizer records positive worker drain.
    pub(crate) fn record_conformance_worker_drain(&self) -> Result<(), HostError> {
        super::publish(self.path(), SEAL, self.id(), None)
    }
}

impl InvocationStore {
    fn conformance_root(&self) -> PathBuf {
        self.state.root.join("conformance-results")
    }

    /// Reserve room for both the live checkpoint and its atomic staging file.
    /// An unsettled invocation continues to occupy its exact reservation.
    pub(crate) fn reserve_conformance(&self, original: &InvocationDirectory) -> Result<(), HostError> {
        if !original.is_conformance() {
            return Err(invalid("checkpoint reservation requires the campaign owner"));
        }
        let root = self.conformance_root();
        directory(&root, true)?;
        let _sweep = self.sweep_conformance(unix_millis(SystemTime::now())?)?;
        let _guard = lock(&root, FlockOperation::LockExclusive)?;
        let mut reserved = 0_u64;
        for entry in fs::read_dir(&root)? {
            let entry = entry?;
            let Some(id) = entry.file_name().to_str().and_then(|name| CliInvocationId::try_from(name.to_owned()).ok()) else {
                continue;
            };
            let _reservation = evidence_file(&root.join(id.as_str()))?;
            reserved = reserved.saturating_add(RESERVATION_BYTES);
            if reserved >= CHECKOUT_BYTES {
                return Err(invalid("conformance evidence capacity exhausted; inspect or release settled results with pse tools retire-result"));
            }
        }
        let file = OpenOptions::new().write(true).create_new(true).mode(0o600)
            .custom_flags(OFlags::NOFOLLOW.bits() as i32)
            .open(root.join(original.id().as_str()))?;
        file.sync_all()?;
        File::open(root)?.sync_all()?;
        Ok(())
    }

    fn campaign_holds(&self, id: &CliInvocationId) -> Result<BTreeMap<String, String>, HostError> {
        let file = match evidence_file(&self.conformance_root().join(format!("{id}.holds"))) {
            Ok(file) => file,
            Err(HostError::Io(error)) if error.kind() == io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
            Err(error) => return Err(error),
        };
        let mut bytes = Vec::new();
        file.take(65_537).read_to_end(&mut bytes)?;
        if bytes.len() > 65_536 {
            return Err(invalid("campaign holds exceed their metadata bound"));
        }
        serde_json::from_slice(&bytes).map_err(io::Error::other).map_err(HostError::from)
    }

    pub(crate) fn hold_conformance(&self, id: &CliInvocationId, consumer: &str, condition: Option<&str>) -> Result<(), HostError> {
        if !self.directory(id)?.is_conformance() || consumer.is_empty() || consumer.len() > 128
            || consumer.chars().any(char::is_control)
            || condition.is_some_and(|text| text.is_empty() || text.len() > 1024 || text.chars().any(char::is_control))
        {
            return Err(invalid("a campaign hold requires an original run, consumer, and bounded condition"));
        }
        let root = self.conformance_root();
        let _index = lock(&root, FlockOperation::LockExclusive)?;
        evidence_file(&root.join(id.as_str()))?;
        let mut holds = self.campaign_holds(id)?;
        if let Some(condition) = condition {
            holds.insert(consumer.to_owned(), condition.to_owned());
        } else {
            holds.remove(consumer);
        }
        let bytes = serde_json::to_vec(&holds).map_err(io::Error::other)?;
        if bytes.len() > 65_536 {
            return Err(invalid("campaign holds exceed their metadata bound"));
        }
        use std::io::Write as _;
        let mut file = tempfile::NamedTempFile::new_in(&root)?;
        file.write_all(&bytes)?;
        file.as_file().sync_all()?;
        file.persist(root.join(format!("{id}.holds"))).map_err(|error| error.error)?;
        File::open(root)?.sync_all()?;
        Ok(())
    }

    pub(crate) fn retire_conformance(
        &self,
        id: &CliInvocationId,
        token: Option<&str>,
    ) -> Result<ConformanceRetirement, HostError> {
        self.retire_conformance_at(id, token, false, unix_millis(SystemTime::now())?)
    }

    fn retire_conformance_at(
        &self,
        id: &CliInvocationId,
        token: Option<&str>,
        sweep: bool,
        now: u64,
    ) -> Result<ConformanceRetirement, HostError> {
        let original = self.directory(id)?;
        if !original.is_conformance() {
            return Err(invalid("release requires an original managed campaign"));
        }
        let root = self.conformance_root();
        let _index = lock(&root, FlockOperation::LockExclusive)?;
        let reservation_path = root.join(id.as_str());
        let reservation = evidence_file(&reservation_path)?;
        let owner = lock(original.path(), FlockOperation::NonBlockingLockExclusive)?;
        let terminal = self.result::<serde::de::IgnoredAny, ManagedSummary>(id)?;
        let mut preview = ConformanceRetirement {
            invocation: id.clone(),
            disposition: "unsettled",
            allocated_bytes: 0,
            eligible_after_unix_ms: None,
            holds: self.campaign_holds(id)?,
            release: None,
        };
        let Some(terminal) = terminal else { return Ok(preview); };
        let native_drained = terminal.settlement == Settlement::Complete
            && terminal.payload.native_lifetime_pending == Some(false)
            && !terminal.payload.borrowed_reservation;
        let detached_drain = match evidence_file(&original.path().join(SEAL)) {
            Ok(file) => {
                let witness: CliInvocationId = serde_json::from_reader(file.take(256)).map_err(io::Error::other)?;
                if &witness != id { return Err(invalid("campaign drain witness names another invocation")); }
                true
            }
            Err(HostError::Io(error)) if error.kind() == io::ErrorKind::NotFound => false,
            Err(error) => return Err(error),
        };
        if !native_drained && !detached_drain {
            return Ok(preview);
        }
        let clock = if native_drained { "result.json" } else { SEAL };
        let settled = unix_millis(evidence_file(&original.path().join(clock))?.metadata()?.modified()?)?;
        let expiry = settled.saturating_add(WEEK_MS);
        preview.eligible_after_unix_ms = Some(expiry);
        if sweep && now < expiry {
            return Ok(preview);
        }
        let mut digest = Sha256::new();
        digest.update(id.as_str().as_bytes());
        let reservation_identity = crate::artifacts::descriptor_identity(&reservation)?;
        digest.update(reservation_identity.inode.to_be_bytes());
        let raw = if let Some(streams) = workflow_evidence::streams(&original)? {
            digest.update(serde_json::to_vec(&streams).map_err(io::Error::other)?);
            streams.prepare_release(&owner, true)?
        } else {
            Vec::new()
        };
        for member in &raw {
            preview.allocated_bytes = preview.allocated_bytes.saturating_add(member.allocated()?);
        }
        let mut selected = Vec::new();
        for name in [SNAPSHOT, STAGING] {
            let Some(member) = selected_file(&owner, name)? else { continue; };
            if !member.committed_by(name, &terminal.artifacts) {
                return Err(invalid("campaign checkpoint differs from the original terminal commitment"));
            }
            digest.update(name.as_bytes());
            digest.update(member.fingerprint.as_bytes());
            preview.allocated_bytes = preview.allocated_bytes.saturating_add(member.allocated);
            selected.push((name, member.identity));
        }
        let release = digest.finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        if !preview.holds.is_empty() {
            preview.disposition = "held";
            if token.is_some() {
                return Err(invalid("original conformance evidence is held"));
            }
            return Ok(preview);
        }
        preview.disposition = "settled";
        preview.release = Some(release.clone());
        if !sweep && token.is_none() || sweep && now < expiry {
            return Ok(preview);
        }
        if !sweep && token != Some(release.as_str()) {
            return Err(invalid("conformance release token no longer matches the original evidence"));
        }
        for member in raw {
            member.unlink(&owner)?;
        }
        for (name, identity) in selected {
            let current = File::from(openat(&owner, name, OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC, Mode::empty()).map_err(io::Error::from)?);
            if crate::artifacts::descriptor_identity(&current)? != identity {
                return Err(invalid("conformance evidence was replaced before release"));
            }
            unlinkat(&owner, name, rustix::fs::AtFlags::empty()).map_err(io::Error::from)?;
        }
        owner.sync_all()?;
        fs::remove_file(reservation_path)?;
        let hold_path = root.join(format!("{id}.holds"));
        match fs::remove_file(hold_path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        File::open(root)?.sync_all()?;
        preview.disposition = "retired";
        preview.release = None;
        Ok(preview)
    }

    pub(super) fn sweep_conformance(&self, now: u64) -> Result<(usize, usize), HostError> {
        let entries = match fs::read_dir(self.conformance_root()) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok((0, 0)),
            Err(error) => return Err(error.into()),
        };
        let mut released = 0;
        let mut blocked = 0;
        for entry in entries {
            let entry = entry?;
            let Some(id) = entry.file_name().to_str().and_then(|name| CliInvocationId::try_from(name.to_owned()).ok()) else {
                continue;
            };
            match self.retire_conformance_at(&id, None, true, now) {
                Ok(preview) if preview.disposition == "retired" => released += 1,
                Ok(_) => {}
                Err(HostError::Io(error)) if error.kind() == io::ErrorKind::NotFound => {}
                // Each failed owner remains reserved and can be diagnosed by its exact ID.
                Err(_) => blocked += 1,
            }
            if released == 128 {
                break;
            }
        }
        Ok((released, blocked))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{envelope::Envelope, generation::PreparedGeneration, invocation::InvocationSpec, paths::CheckoutState};

    #[test]
    fn campaign_release_requires_drain_hold_clear_and_exact_member() {
        let temp = tempfile::tempdir().unwrap();
        let environment = BTreeMap::from([("XDG_STATE_HOME".into(), temp.path().join("state").into_os_string())]);
        let state = CheckoutState::from_paths(&temp.path().join("checkout"), &temp.path().join("common.git"), &environment).unwrap();
        let store = InvocationStore::new(state);
        let original = store.begin(InvocationSpec {
            invocation_id: CliInvocationId::new(),
            command: "conformance run".into(),
            session: None,
            parent: None,
            operation: (),
            control: PreparedGeneration {
                generation: "generation.fixture".into(),
                protocol: 1,
                manifest_digest: "a".repeat(64),
            },
            environment_digest: "b".repeat(64),
            scientific: Vec::new(),
        }).unwrap();
        let id = original.id();
        store.reserve_conformance(&original).unwrap();
        let checkpoint = original.path().join(SNAPSHOT);
        let owned = OpenOptions::new().write(true).create_new(true).mode(0o600).open(&checkpoint).unwrap();
        std::io::Write::write_all(&mut &owned, b"original").unwrap();
        assert_eq!(store.retire_conformance(id, None).unwrap().disposition, "unsettled");
        let mut terminal = Envelope::response("conformance run", "fixture complete", ManagedSummary {
            bundle: None,
            native_lifetime_pending: Some(false),
            workflow: None,
            child_exit: Some(0),
            stdout: None,
            stderr: None,
            borrowed_reservation: false,
            service_result: None,
        });
        terminal.invocation_id = id.clone();
        terminal.settlement = Settlement::Complete;
        terminal.artifacts = original.conformance_artifacts().unwrap();
        super::super::publish(original.path(), "result.json", &terminal, None).unwrap();
        let lease = lock(original.path(), FlockOperation::LockShared).unwrap();
        assert!(store.retire_conformance(id, None).is_err());
        drop(lease);
        let token = store.retire_conformance(id, None).unwrap().release.unwrap();
        store.hold_conformance(id, "review", Some("until reviewed")).unwrap();
        assert!(store.retire_conformance(id, Some(&token)).is_err());
        store.hold_conformance(id, "review", None).unwrap();
        assert_eq!(store.retire_conformance(id, Some(&token)).unwrap().disposition, "retired");
        assert!(!checkpoint.exists());
        assert!(original.path().join("result.json").exists());
    }
}
