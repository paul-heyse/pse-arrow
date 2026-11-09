// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Admission and release of original managed campaign observations.
use super::{
    InvocationDirectory, InvocationEvidence, InvocationStore, META_LIMIT, directory,
    evidence_file, invalid, publish, unix_millis,
    workflow_evidence::{RawRelease, WorkflowStreams, open_child, stream_receipt},
};
use crate::{
    HostError,
    artifacts::{DescriptorIdentity, descriptor_identity},
    envelope::{ArtifactReference, CliInvocationId, Envelope, Settlement},
    extraction::OutputStream,
    managed::{result::ManagedSummary, workflow::digest},
};
use rustix::fs::{AtFlags, FlockOperation, Mode, OFlags, RenameFlags, flock, openat, renameat_with, unlinkat};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs::{self, File, OpenOptions}, io::{self, Read}, os::unix::fs::{MetadataExt, OpenOptionsExt}, path::{Path, PathBuf}, time::SystemTime};

pub(crate) const SNAPSHOT: &str = "conformance-progress.json";
const STAGING: &str = "conformance-progress.pending";
const RESERVATION_BYTES: u64 = 4 * 1024 * 1024;
const CHECKOUT_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const WEEK_MS: u64 = 7 * 24 * 60 * 60 * 1000;
const SEAL: &str = "conformance_worker_drain.json";
const RELEASE_INTENT: &str = "conformance-release-intent.json";
const RETIRED: &str = "conformance-retired.json";
const CHECKPOINT_KIND: &str = "conformance_checkpoint";
const RAW_KIND: &str = "conformance_raw_streams";

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CampaignRelease {
    invocation: CliInvocationId,
    reservation: DescriptorIdentity,
    checkpoints: BTreeMap<String, CheckpointCommitment>,
    raw: Option<RawCommitment>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckpointCommitment {
    identity: DescriptorIdentity,
    fingerprint: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCommitment {
    streams: WorkflowStreams,
    receipt: DescriptorIdentity,
}

impl RawCommitment {
    fn read(original: &InvocationDirectory) -> Result<Option<Self>, HostError> {
        Ok(stream_receipt(original)?.map(|(streams, receipt)| Self { streams, receipt }))
    }

    fn fingerprint(&self) -> Result<String, HostError> {
        Ok(digest(&serde_json::to_vec(self).map_err(io::Error::other)?))
    }
}

fn checkpoint_commitments(artifacts: &[ArtifactReference]) -> Result<BTreeMap<&str, &str>, HostError> {
    let mut commitments = BTreeMap::new();
    for artifact in artifacts.iter().filter(|artifact| artifact.kind == CHECKPOINT_KIND) {
        let name = artifact.location.as_str();
        let expected = artifact.digest.as_deref().ok_or_else(|| invalid("campaign checkpoint lacks a terminal identity"))?;
        if ![SNAPSHOT, STAGING].contains(&name) || commitments.insert(name, expected).is_some() {
            return Err(invalid("invalid or duplicate terminal campaign checkpoint commitment"));
        }
    }
    Ok(commitments)
}

fn raw_commitment(artifacts: &[ArtifactReference]) -> Result<Option<&str>, HostError> {
    let mut entries = artifacts.iter().filter(|artifact| artifact.kind == RAW_KIND);
    let Some(artifact) = entries.next() else { return Ok(None); };
    if entries.next().is_some() || artifact.location != InvocationEvidence::WorkflowStreams.filename() {
        return Err(invalid("invalid or duplicate terminal campaign raw stream commitment"));
    }
    artifact.digest.as_deref().map(Some).ok_or_else(|| invalid("campaign raw streams lack a terminal identity"))
}

impl CampaignRelease {
    fn authenticate(&self, id: &CliInvocationId, artifacts: &[ArtifactReference]) -> Result<(), HostError> {
        let checkpoint_match = checkpoint_commitments(artifacts)? == self.checkpoints.iter()
            .map(|(name, member)| (name.as_str(), member.fingerprint.as_str())).collect();
        let raw = self.raw.as_ref().map(RawCommitment::fingerprint).transpose()?;
        if &self.invocation != id || !checkpoint_match
            || self.raw.as_ref().is_some_and(|member| &member.streams.invocation != id)
            || raw_commitment(artifacts)? != raw.as_deref()
        {
            return Err(invalid("campaign release record differs from the original terminal commitments"));
        }
        Ok(())
    }

    fn token(&self) -> Result<String, HostError> {
        Ok(digest(&serde_json::to_vec(self).map_err(io::Error::other)?))
    }

    fn prepare_raw(
        &self,
        owner: &File,
        original: &InvocationDirectory,
        replay: bool,
    ) -> Result<(Vec<RawRelease>, bool), HostError> {
        let Some(raw) = &self.raw else { return Ok((Vec::new(), false)); };
        let current = RawCommitment::read(original)?;
        if current.as_ref().is_some_and(|current| current != raw) || current.is_none() && !replay {
            return Err(invalid("original campaign raw stream receipt is missing or replaced"));
        }
        let selected = raw.streams.prepare_release(owner, replay)?;
        if current.is_none() && !selected.is_empty() {
            return Err(invalid("campaign raw stream receipt disappeared before its original members"));
        }
        Ok((selected, current.is_some()))
    }
}

fn optional_member(owner: &File, name: &str) -> Result<Option<File>, HostError> {
    match open_child(owner, Path::new(name)) {
        Ok(file) => Ok(Some(file)),
        Err(HostError::Io(error)) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn read_release(owner: &File, name: &str) -> Result<Option<(CampaignRelease, DescriptorIdentity)>, HostError> {
    let Some(file) = optional_member(owner, name)? else { return Ok(None); };
    let identity = descriptor_identity(&file)?;
    let mut bytes = Vec::new();
    (&file).take(META_LIMIT + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > META_LIMIT || descriptor_identity(&file)? != identity {
        return Err(invalid("campaign release record is unbounded or changed while observed"));
    }
    let release = serde_json::from_slice(&bytes).map_err(io::Error::other)?;
    Ok(Some((release, identity)))
}

fn unlink_exact(parent: &File, name: &str, expected: &DescriptorIdentity) -> Result<(), HostError> {
    if descriptor_identity(&open_child(parent, Path::new(name))?)? != *expected {
        return Err(invalid("original campaign member was replaced before release"));
    }
    unlinkat(parent, name, AtFlags::empty()).map_err(io::Error::from)?;
    Ok(())
}

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
    identity: DescriptorIdentity,
    allocated: u64,
    fingerprint: String,
    bytes: Vec<u8>,
}

impl CampaignMember {
    fn committed_by(&self, name: &str, artifacts: &[ArtifactReference]) -> bool {
        artifacts.iter().any(|artifact| {
            artifact.kind == CHECKPOINT_KIND && artifact.location == name
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
    let identity = descriptor_identity(&file)?;
    let mut bytes = Vec::new();
    (&file).take(RESERVATION_BYTES / 2 + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 != metadata.len() || descriptor_identity(&file)? != identity {
        return Err(invalid("conformance evidence changed while calculating release identity"));
    }
    let fingerprint = format!("{}:{}:{}", identity.device, identity.inode, digest(&bytes));
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
        if self.retired_campaign(&owner)?.is_some_and(|release| release.checkpoints.contains_key(SNAPSHOT)) {
            return Ok(None);
        }
        let Some(member) = selected_file(&owner, SNAPSHOT)? else {
            if terminal.is_some_and(|artifacts| artifacts.iter().any(|artifact| {
                artifact.kind == CHECKPOINT_KIND && artifact.location == SNAPSHOT
            })) {
                return Err(invalid("committed campaign checkpoint is missing without a retirement witness"));
            }
            return Ok(None);
        };
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
                    kind: CHECKPOINT_KIND.into(),
                    location: name.into(),
                    digest: Some(member.fingerprint),
                });
            }
        }
        Ok(artifacts)
    }

    pub(super) fn borrow_conformance(&self) -> Result<File, HostError> {
        lock(self.path(), FlockOperation::LockShared)
    }

    pub(crate) fn conformance_raw_artifact(&self) -> Result<ArtifactReference, HostError> {
        let raw = RawCommitment::read(self)?.ok_or_else(|| invalid("campaign raw stream receipt is absent"))?;
        Ok(ArtifactReference {
            kind: RAW_KIND.into(),
            location: InvocationEvidence::WorkflowStreams.filename().into(),
            digest: Some(raw.fingerprint()?),
        })
    }

    fn campaign_terminal(&self, owner: &File) -> Result<Option<Envelope<serde::de::IgnoredAny>>, HostError> {
        let Some(file) = optional_member(owner, "result.json")? else { return Ok(None); };
        let terminal = serde_json::from_reader(file).map_err(io::Error::other)?;
        super::validate_result(self.id(), &self.command, &terminal)?;
        Ok(Some(terminal))
    }

    fn retired_campaign(&self, owner: &File) -> Result<Option<CampaignRelease>, HostError> {
        let Some((release, _)) = read_release(owner, RETIRED)? else { return Ok(None); };
        let terminal = self.campaign_terminal(owner)?.ok_or_else(|| invalid("retired campaign lacks its original terminal result"))?;
        release.authenticate(self.id(), &terminal.artifacts)?;
        Ok(Some(release))
    }

    pub(crate) fn conformance_checkpoint_retired(&self) -> Result<bool, HostError> {
        if !self.is_conformance() {
            return Ok(false);
        }
        let owner = self.borrow_conformance()?;
        Ok(self.retired_campaign(&owner)?.is_some_and(|release| release.checkpoints.contains_key(SNAPSHOT)))
    }

    pub(super) fn conformance_raw_retired(&self, owner: &File, stream: OutputStream) -> Result<bool, HostError> {
        Ok(self.retired_campaign(owner)?.and_then(|release| release.raw).is_some_and(|raw| {
            match stream {
                OutputStream::Stdout => raw.streams.stdout.is_some(),
                OutputStream::Stderr => raw.streams.stderr.is_some(),
            }
        }))
    }

    pub(super) fn conformance_raw_streams(&self, owner: &File) -> Result<Option<WorkflowStreams>, HostError> {
        let Some(terminal) = self.campaign_terminal(owner)? else {
            return Ok(None);
        };
        let Some(expected) = raw_commitment(&terminal.artifacts)? else {
            return Ok(None);
        };
        let current = RawCommitment::read(self)?.ok_or_else(|| invalid("original campaign raw stream receipt is absent"))?;
        if expected != current.fingerprint()? {
            return Err(invalid("campaign raw stream receipt differs from its original terminal commitment"));
        }
        Ok(Some(current.streams))
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

    fn campaign_holds(index: &File, id: &CliInvocationId) -> Result<BTreeMap<String, String>, HostError> {
        let Some(file) = optional_member(index, &format!("{id}.holds"))? else {
            return Ok(BTreeMap::new());
        };
        let mut bytes = Vec::new();
        file.take(65_537).read_to_end(&mut bytes)?;
        if bytes.len() > 65_536 {
            return Err(invalid("campaign holds exceed their metadata bound"));
        }
        serde_json::from_slice(&bytes).map_err(io::Error::other).map_err(HostError::from)
    }

    pub(crate) fn hold_conformance(&self, id: &CliInvocationId, consumer: &str, condition: Option<&str>) -> Result<(), HostError> {
        let original = self.directory(id)?;
        if !original.is_conformance() || consumer.is_empty() || consumer.len() > 128
            || consumer.chars().any(char::is_control)
            || condition.is_some_and(|text| text.is_empty() || text.len() > 1024 || text.chars().any(char::is_control))
        {
            return Err(invalid("a campaign hold requires an original run, consumer, and bounded condition"));
        }
        let root = self.conformance_root();
        let index = lock(&root, FlockOperation::LockExclusive)?;
        let owner = lock(original.path(), FlockOperation::LockShared)?;
        if optional_member(&owner, RELEASE_INTENT)?.is_some()
            || optional_member(&owner, RETIRED)?.is_some()
        {
            return Err(invalid("original campaign evidence release has already started"));
        }
        open_child(&index, Path::new(id.as_str()))?;
        let mut holds = Self::campaign_holds(&index, id)?;
        if let Some(condition) = condition {
            holds.insert(consumer.to_owned(), condition.to_owned());
        } else {
            holds.remove(consumer);
        }
        let hold_name = format!("{id}.holds");
        if holds.is_empty() {
            if let Some(file) = optional_member(&index, &hold_name)? {
                unlink_exact(&index, &hold_name, &descriptor_identity(&file)?)?;
                index.sync_all()?;
            }
            return Ok(());
        }
        let bytes = serde_json::to_vec(&holds).map_err(io::Error::other)?;
        if bytes.len() > 65_536 {
            return Err(invalid("campaign holds exceed their metadata bound"));
        }
        use std::io::Write as _;
        let mut file = tempfile::NamedTempFile::new_in(&root)?;
        file.write_all(&bytes)?;
        file.as_file().sync_all()?;
        file.persist(root.join(hold_name)).map_err(|error| error.error)?;
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
        let index = lock(&root, FlockOperation::LockExclusive)?;
        let owner = lock(original.path(), FlockOperation::NonBlockingLockExclusive)?;
        let mut preview = ConformanceRetirement {
            invocation: id.clone(),
            disposition: "unsettled",
            allocated_bytes: 0,
            eligible_after_unix_ms: None,
            holds: Self::campaign_holds(&index, id)?,
            release: None,
        };
        if let Some(released) = original.retired_campaign(&owner)? {
            if let Some(reservation) = optional_member(&index, id.as_str())? {
                if descriptor_identity(&reservation)? != released.reservation {
                    return Err(invalid("retired campaign reservation has been replaced"));
                }
                // Complete interrupted index cleanup only on an authorized release or sweep.
                if sweep || token == Some(released.token()?.as_str()) {
                    owner.sync_all()?;
                    unlink_exact(&index, id.as_str(), &released.reservation)?;
                    index.sync_all()?;
                }
            }
            preview.disposition = "retired";
            return Ok(preview);
        }
        let reservation = open_child(&index, Path::new(id.as_str()))?;
        let reservation_identity = descriptor_identity(&reservation)?;
        let terminal = self.result::<serde::de::IgnoredAny, ManagedSummary>(id)?;
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
        if !preview.holds.is_empty() {
            preview.disposition = "held";
            if token.is_some() {
                return Err(invalid("original conformance evidence is held"));
            }
            return Ok(preview);
        }
        let current_checkpoints = [SNAPSHOT, STAGING].into_iter()
            .filter_map(|name| selected_file(&owner, name).transpose().map(|result| result.map(|member| (name, member))))
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        let intent = read_release(&owner, RELEASE_INTENT)?;
        let intent_identity = intent.as_ref().map(|(_, identity)| identity.clone());
        let replay = intent.is_some();
        let release = match intent {
            Some((intent, _)) => intent,
            None => CampaignRelease {
                invocation: id.clone(),
                reservation: reservation_identity.clone(),
                checkpoints: current_checkpoints.iter().map(|(&name, member)| (name.into(), CheckpointCommitment {
                    identity: member.identity.clone(),
                    fingerprint: member.fingerprint.clone(),
                })).collect(),
                raw: if raw_commitment(&terminal.artifacts)?.is_some() {
                    RawCommitment::read(&original)?
                } else {
                    None
                },
            },
        };
        release.authenticate(id, &terminal.artifacts)?;
        if release.reservation != reservation_identity {
            return Err(invalid("campaign release does not match the original reservation"));
        }
        for (&name, member) in &current_checkpoints {
            if !release.checkpoints.get(name).is_some_and(|expected| {
                expected.identity == member.identity && expected.fingerprint == member.fingerprint
            }) {
                return Err(invalid("original campaign checkpoint was replaced before release"));
            }
            preview.allocated_bytes = preview.allocated_bytes.saturating_add(member.allocated);
        }
        let (raw, remove_receipt) = release.prepare_raw(&owner, &original, replay)?;
        for member in &raw {
            preview.allocated_bytes = preview.allocated_bytes.saturating_add(member.allocated_bytes()?);
        }
        let release_token = release.token()?;
        preview.disposition = if replay { "release_pending" } else { "settled" };
        preview.release = Some(release_token.clone());
        if !sweep && token.is_none() {
            return Ok(preview);
        }
        if !sweep && token != Some(release_token.as_str()) {
            return Err(invalid("conformance release token no longer matches the original evidence"));
        }
        let intent_identity = if let Some(identity) = intent_identity {
            identity
        } else {
            publish(original.path(), RELEASE_INTENT, &release, Some(META_LIMIT))?;
            let (intent, identity) = read_release(&owner, RELEASE_INTENT)?
                .ok_or_else(|| invalid("published campaign release intent is absent"))?;
            if intent != release {
                return Err(invalid("published campaign release intent was replaced"));
            }
            identity
        };
        // A prior publisher may have failed after rename and before synchronizing the intent.
        owner.sync_all()?;
        for (name, member) in current_checkpoints {
            unlink_exact(&owner, name, &member.identity)?;
        }
        for member in raw {
            member.unlink(&owner)?;
        }
        if remove_receipt && let Some(raw) = &release.raw {
            unlink_exact(&owner, InvocationEvidence::WorkflowStreams.filename(), &raw.receipt)?;
        }
        // Retain the authenticated record as positive expiry only after member removals are durable.
        owner.sync_all()?;
        if descriptor_identity(&open_child(&owner, Path::new(RELEASE_INTENT))?)? != intent_identity {
            return Err(invalid("campaign release intent was replaced before retirement"));
        }
        renameat_with(&owner, RELEASE_INTENT, &owner, RETIRED, RenameFlags::NOREPLACE).map_err(io::Error::from)?;
        owner.sync_all()?;
        unlink_exact(&index, id.as_str(), &release.reservation)?;
        index.sync_all()?;
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
    use crate::{generation::PreparedGeneration, invocation::InvocationSpec, paths::CheckoutState};
    use std::io::Write as _;

    struct Fixture {
        _temp: tempfile::TempDir,
        store: InvocationStore,
        original: InvocationDirectory,
        raw: RawCommitment,
    }

    fn write(path: &Path, bytes: &[u8]) -> File {
        let mut file = OpenOptions::new().write(true).create_new(true).mode(0o600).open(path).unwrap();
        file.write_all(bytes).unwrap();
        file
    }

    impl Fixture {
        fn new() -> Self {
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
            store.reserve_conformance(&original).unwrap();
            write(&original.path().join(SNAPSHOT), b"original");
            let stdout = write(&original.path().join("stdout.raw"), b"original raw stdout");
            let stderr = write(&original.path().join("stderr.raw"), b"original raw stderr");
            original.publish_evidence(InvocationEvidence::WorkflowStreams, &WorkflowStreams {
                invocation: original.id().clone(),
                stdout: Some(descriptor_identity(&stdout).unwrap()),
                stderr: Some(descriptor_identity(&stderr).unwrap()),
            }).unwrap();
            let raw = RawCommitment::read(&original).unwrap().unwrap();
            Self { _temp: temp, store, original, raw }
        }

        fn finish(&self, foreground_drained: bool) -> Vec<ArtifactReference> {
            let mut terminal = Envelope::response("conformance run", "fixture complete", ManagedSummary {
                bundle: None,
                native_lifetime_pending: Some(!foreground_drained),
                workflow: None,
                child_exit: Some(0),
                stdout: None,
                stderr: None,
                borrowed_reservation: false,
                service_result: None,
            });
            terminal.invocation_id = self.original.id().clone();
            terminal.settlement = Settlement::Complete;
            terminal.artifacts = self.original.conformance_artifacts().unwrap();
            terminal.artifacts.push(self.original.conformance_raw_artifact().unwrap());
            publish(self.original.path(), "result.json", &terminal, None).unwrap();
            terminal.artifacts
        }

        fn path(&self, name: &str) -> PathBuf {
            self.original.path().join(name)
        }

        fn retire(&self, token: Option<&str>) -> Result<ConformanceRetirement, HostError> {
            self.store.retire_conformance(self.original.id(), token)
        }
    }

    #[test]
    fn campaign_release_requires_drain_hold_clear_and_exact_member() {
        let fixture = Fixture::new();
        let id = fixture.original.id();
        assert_eq!(fixture.retire(None).unwrap().disposition, "unsettled");
        let artifacts = fixture.finish(false);
        assert_eq!(fixture.retire(None).unwrap().disposition, "unsettled");
        fixture.original.record_conformance_worker_drain().unwrap();

        let lease = fixture.original.borrow_conformance().unwrap();
        assert!(fixture.retire(None).is_err());
        drop(lease);
        let preview = fixture.retire(None).unwrap();
        let token = preview.release.unwrap();
        assert!(preview.allocated_bytes > 0);
        let raw_reader = fixture.original.stream(OutputStream::Stdout).unwrap();
        assert!(fixture.retire(Some(&token)).is_err());
        assert!(!fixture.path(RELEASE_INTENT).exists());
        drop(raw_reader);

        fixture.store.hold_conformance(id, "review", Some("until reviewed")).unwrap();
        assert!(fixture.retire(Some(&token)).is_err());
        fixture.store.hold_conformance(id, "review", None).unwrap();
        let unowned = fixture.path("other");
        write(&unowned, b"do not remove");
        assert_eq!(fixture.retire(Some(&token)).unwrap().disposition, "retired");
        assert_eq!(fixture.retire(None).unwrap().disposition, "retired");
        assert!(!fixture.path(SNAPSHOT).exists());
        assert!(!fixture.path("stdout.raw").exists());
        assert!(!fixture.path("stderr.raw").exists());
        assert!(!fixture.path(InvocationEvidence::WorkflowStreams.filename()).exists());
        assert!(fixture.path("result.json").exists());
        assert!(unowned.exists());
        assert!(fixture.original.conformance_checkpoint(Some(&artifacts)).unwrap().is_none());
        assert!(fixture.original.conformance_checkpoint_retired().unwrap());
        write(&fixture.path("stdout.raw"), b"foreign after expiry");
        assert!(fixture.original.stream(OutputStream::Stdout).unwrap_err().to_string().contains("retired"));
        assert!(fixture.path("stdout.raw").exists());
    }

    #[test]
    fn campaign_release_refuses_unexplained_missing_checkpoint_or_raw() {
        let fixture = Fixture::new();
        let artifacts = fixture.finish(true);
        let token = fixture.retire(None).unwrap().release.unwrap();
        fs::remove_file(fixture.path(SNAPSHOT)).unwrap();
        assert!(fixture.original.conformance_checkpoint(Some(&artifacts)).is_err());
        assert!(!fixture.original.conformance_checkpoint_retired().unwrap());
        assert!(fixture.retire(Some(&token)).is_err());
        assert!(!fixture.path(RELEASE_INTENT).exists());
        assert!(fixture.path("stdout.raw").exists());

        let fixture = Fixture::new();
        fixture.finish(true);
        let token = fixture.retire(None).unwrap().release.unwrap();
        fs::remove_file(fixture.path("stdout.raw")).unwrap();
        assert!(fixture.retire(Some(&token)).is_err());
        assert!(!fixture.path(RELEASE_INTENT).exists());
        assert!(fixture.path("stderr.raw").exists());
        assert!(fixture.path(SNAPSHOT).exists());
    }

    #[test]
    fn campaign_release_replays_original_members_without_removing_replacements() {
        let fixture = Fixture::new();
        fixture.finish(true);
        let id = fixture.original.id();
        let owner = fixture.original.borrow_conformance().unwrap();
        let checkpoint = selected_file(&owner, SNAPSHOT).unwrap().unwrap();
        let release = CampaignRelease {
            invocation: id.clone(),
            reservation: descriptor_identity(&evidence_file(&fixture.store.conformance_root().join(id.as_str())).unwrap()).unwrap(),
            checkpoints: BTreeMap::from([(SNAPSHOT.into(), CheckpointCommitment {
                identity: checkpoint.identity,
                fingerprint: checkpoint.fingerprint,
            })]),
            raw: Some(RawCommitment {
                streams: fixture.raw.streams.clone(),
                receipt: fixture.raw.receipt.clone(),
            }),
        };
        let token = fixture.retire(None);
        assert!(token.is_err());
        drop(owner);
        let token = fixture.retire(None).unwrap().release.unwrap();
        assert_eq!(token, release.token().unwrap());
        publish(fixture.original.path(), RELEASE_INTENT, &release, Some(META_LIMIT)).unwrap();
        fs::remove_file(fixture.path(SNAPSHOT)).unwrap();
        fs::remove_file(fixture.path("stdout.raw")).unwrap();
        write(&fixture.path("stdout.raw"), b"unrelated replacement");
        assert!(fixture.retire(Some(&token)).is_err());
        assert!(fixture.path("stdout.raw").exists());
        assert!(fixture.path("stderr.raw").exists());
        assert!(!fixture.path(RETIRED).exists());
        fs::remove_file(fixture.path("stdout.raw")).unwrap();

        let receipt = fixture.path(InvocationEvidence::WorkflowStreams.filename());
        // Same bytes in another inode cannot replace the original writer receipt.
        let saved = fs::read(&receipt).unwrap();
        let original = evidence_file(&receipt).unwrap();
        fs::remove_file(&receipt).unwrap();
        write(&receipt, &saved);
        assert!(fixture.retire(Some(&token)).is_err());
        assert!(receipt.exists());
        assert!(fixture.path("stderr.raw").exists());
        // Simulate interruption after the authenticated raw member and receipt were removed.
        fs::remove_file(&receipt).unwrap();
        drop(original);
        fs::remove_file(fixture.path("stderr.raw")).unwrap();
        let retry = fixture.retire(None).unwrap();
        assert_eq!(retry.disposition, "release_pending");
        assert_eq!(retry.release.as_deref(), Some(token.as_str()));
        assert!(fixture.store.hold_conformance(id, "late", Some("would be too late")).is_err());
        assert_eq!(fixture.retire(Some(&token)).unwrap().disposition, "retired");
        assert!(!fixture.path(RELEASE_INTENT).exists());
        assert!(fixture.path(RETIRED).exists());
        assert!(!fixture.store.conformance_root().join(id.as_str()).exists());
    }
}
