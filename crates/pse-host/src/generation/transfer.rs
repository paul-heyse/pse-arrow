// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Same-process executable handoff using the original owned use records.
use super::*;
use std::{
    os::unix::process::CommandExt,
    process::{Command, Stdio},
};

pub(super) const HANDOFF: &str = "PSE_HOST_GENERATION_HANDOFF";
const RECORD_LIMIT: u64 = 16 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Role {
    Source,
    Target,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct UseHandoff {
    id: String,
    target: PreparedGeneration,
    role: Role,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Claim {
    #[serde(with = "os_path")]
    tools: PathBuf,
    generation: String,
    record: String,
}

mod os_path {
    use super::*;
    pub(super) fn serialize<S: serde::Serializer>(
        value: &Path,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        value.as_os_str().serialize(serializer)
    }
    pub(super) fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<PathBuf, D::Error> {
        OsString::deserialize(deserializer).map(PathBuf::from)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Handoff {
    version: u32,
    id: String,
    target: Claim,
    source: Option<Claim>,
}

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
pub(super) fn read_record(path: &Path) -> Result<UseRecord, HostError> {
    let file = owned_file(path, false)?;
    if file.metadata()?.len() > RECORD_LIMIT {
        return Err(invalid("generation use record exceeds its bound"));
    }
    let mut bytes = Vec::new();
    file.take(RECORD_LIMIT + 1).read_to_end(&mut bytes)?;
    serde_json::from_slice(&bytes).map_err(|_| invalid("generation use record is invalid"))
}

impl GenerationUse {
    fn claim(&self) -> Result<Claim, HostError> {
        Ok(Claim {
            tools: self
                .record
                .parent()
                .and_then(Path::parent)
                .and_then(Path::parent)
                .ok_or_else(|| invalid("generation use claim has no tool root"))?
                .to_owned(),
            generation: self
                .record
                .parent()
                .and_then(Path::file_name)
                .and_then(OsStr::to_str)
                .ok_or_else(|| invalid("generation use claim has no component"))?
                .into(),
            record: self
                .record
                .file_stem()
                .and_then(OsStr::to_str)
                .ok_or_else(|| invalid("generation use claim has no record ID"))?
                .into(),
        })
    }
    fn set_handoff(&self, handoff: Option<UseHandoff>) -> Result<(), HostError> {
        let mut record = read_record(&self.record)?;
        if record.version != VERSION
            || record.owner != identity()?
            || record.manifest_digest != self.manifest.digest()?
            || record.finalizer.is_some()
        {
            return Err(invalid(
                "generation claim does not belong to the performing process and manifest",
            ));
        }
        record.handoff = handoff;
        self.replace_record(&record)
    }
    pub(super) fn replace_record(&self, record: &UseRecord) -> Result<(), HostError> {
        let bytes = encoded(record)?;
        if u64::try_from(bytes.len()).map_or(true, |length| length > RECORD_LIMIT) {
            return Err(invalid("generation use record exceeds its bound"));
        }
        let parent = self
            .record
            .parent()
            .ok_or_else(|| invalid("generation claim has no parent"))?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        temporary.write_all(&bytes)?;
        temporary.as_file().sync_all()?;
        temporary
            .persist(&self.record)
            .map_err(|error| error.error)?;
        File::open(parent)?.sync_all()?;
        Ok(())
    }
}

fn claim_path(root: &Path, claim: &Claim) -> Result<(PathBuf, PathBuf), HostError> {
    if !component(&claim.generation)
        || !claim.generation.starts_with("generation.")
        || uuid::Uuid::parse_str(&claim.record).is_err()
        || claim.record.len() != 36
    {
        return Err(invalid(
            "executable handoff claim is not a fixed relative reference",
        ));
    }
    let uses = root.join(".uses");
    private_directory(&uses, false)?;
    let records = uses.join(&claim.generation);
    private_directory(&records, false)?;
    Ok((
        uses.join(format!("{}.lock", claim.generation)),
        records.join(format!("{}.json", claim.record)),
    ))
}

fn adopt_claim(
    root: &Path,
    claim: &Claim,
    expected: &UseHandoff,
) -> Result<Option<GenerationUse>, HostError> {
    if claim.tools != root
        || root.file_name().is_none_or(|name| name != "host-tools")
        || root
            .parent()
            .and_then(Path::file_name)
            .is_none_or(|name| name != "target")
    {
        return Err(invalid(
            "executable claim is outside a fixed checkout tool root",
        ));
    }
    private_directory(root, false)?;
    let update = owned_file(&root.join(".update.lock"), false)?;
    rustix::fs::flock(&update, rustix::fs::FlockOperation::LockShared)
        .map_err(std::io::Error::from)?;
    let (lock, record) = claim_path(root, claim)?;
    let lock = owned_file(&lock, false)?;
    rustix::fs::flock(&lock, rustix::fs::FlockOperation::LockShared)
        .map_err(std::io::Error::from)?;
    let captured = match read_record(&record) {
        Err(HostError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(None);
        }
        result => result?,
    };
    // Inherited/completed markers are locators, never authority for another process.
    if captured.owner != identity()? || captured.handoff.is_none() {
        return Ok(None);
    }
    let manifest = GenerationManifest::load(&root.join(&claim.generation))?;
    if captured.version != VERSION
        || captured.handoff.as_ref() != Some(expected)
        || captured.manifest_digest != manifest.digest()?
        || captured.finalizer.is_some()
    {
        return Err(invalid(
            "executable handoff differs from the original owned claim",
        ));
    }
    Ok(Some(GenerationUse {
        lock,
        record,
        manifest,
    }))
}

pub(super) fn adopt(
    root: &Path,
    generation: &str,
    value: Option<&OsStr>,
) -> Result<Option<GenerationUse>, HostError> {
    let Some(value) = value
        .and_then(OsStr::to_str)
        .filter(|value| value.len() <= 4096)
    else {
        return Ok(None);
    };
    let Ok(handoff) = serde_json::from_str::<Handoff>(value) else {
        return Ok(None);
    };
    if handoff.version != VERSION
        || uuid::Uuid::parse_str(&handoff.id).is_err()
        || handoff.target.generation != generation
        || handoff.target.tools != root
    {
        return Ok(None);
    }
    let _update = update_lock(root)?;
    let manifest = GenerationManifest::load(&root.join(generation))?;
    let target = PreparedGeneration {
        generation: generation.into(),
        protocol: manifest.control_protocol(),
        manifest_digest: manifest.digest()?,
    };
    let expected = UseHandoff {
        id: handoff.id.clone(),
        target: target.clone(),
        role: Role::Target,
    };
    let Some(usage) = adopt_claim(root, &handoff.target, &expected)? else {
        return Ok(None);
    };
    if let Some(source) = &handoff.source {
        let expected = UseHandoff {
            id: handoff.id,
            target,
            role: Role::Source,
        };
        let source = adopt_claim(&source.tools, source, &expected)?.ok_or_else(|| {
            invalid("source executable claim cannot establish same-process transfer")
        })?;
        // The target's actual bytes, process identity and original target claim are now
        // authenticated. The source declared no remaining resource obligations before exec.
        source.settle()?;
    }
    usage.set_handoff(None)?;
    Ok(Some(usage))
}

fn begin(
    entry: &EntryGuard,
    target: &GenerationUse,
    reference: &PreparedGeneration,
) -> Result<Handoff, HostError> {
    let id = uuid::Uuid::new_v4().to_string();
    target.set_handoff(Some(UseHandoff {
        id: id.clone(),
        target: reference.clone(),
        role: Role::Target,
    }))?;
    let source = if let Some(source) = &entry.usage {
        source.set_handoff(Some(UseHandoff {
            id: id.clone(),
            target: reference.clone(),
            role: Role::Source,
        }))?;
        Some(source.claim()?)
    } else {
        None
    };
    Ok(Handoff {
        version: VERSION,
        id,
        target: target.claim()?,
        source,
    })
}

fn cancel(entry: &EntryGuard, target: GenerationUse) -> Result<(), HostError> {
    let restore = entry
        .usage
        .as_ref()
        .map_or(Ok(()), |source| source.set_handoff(None));
    let settled = target.settle();
    match (restore, settled) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(operation), Err(cleanup)) => Err(HostError::Settlement {
            operation: Box::new(operation),
            cleanup: Box::new(cleanup),
        }),
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
    }
}

fn failed(entry: &EntryGuard, target: GenerationUse, operation: HostError) -> HostError {
    match cancel(entry, target) {
        Ok(()) => operation,
        Err(cleanup) => HostError::Settlement {
            operation: Box::new(operation),
            cleanup: Box::new(cleanup),
        },
    }
}

/// Rebuild only after `new_work_decision` returned Rebuild, before the parsed command's
/// effects. Both old and new claims remain protective until the replacement image proves
/// same-process transfer. Exec failure retains the source guard and settles the unused target.
pub fn rebuild_and_reexec(
    root: &Path,
    tool: HostTool,
    arguments: &[OsString],
    entry: &mut EntryGuard,
) -> Result<i32, HostError> {
    let root = root.canonicalize()?;
    entry.promote()?;
    let stderr = rustix::io::dup(std::io::stderr()).map_err(std::io::Error::from)?;
    let result = Command::new(root.join("scripts/bootstrap-host"))
        .current_dir(&root)
        .stdout(Stdio::from(stderr))
        .status()?;
    if !result.success() {
        return Ok(result.code().unwrap_or(125));
    }
    // The replacement may intentionally implement a newer fingerprint policy. The old
    // image authenticates prepared bytes, never interprets their source freshness using
    // its superseded policy. The new image checks freshness before command dispatch;
    // its same-process loop marker refuses actual post-publication input drift.
    let reference = prepared(&root)?
        .ok_or_else(|| invalid("bootstrap did not publish a prepared generation"))?;
    let executable = reference.executable(&root, tool)?;
    let target = GenerationUse::acquire(&root.join("target/host-tools"), &reference.generation)?;
    // Bootstrap returned successfully and all of its synchronous build/probe children were
    // reaped; the requested new-work command has not dispatched or handed off any paths yet.
    let handoff = match begin(entry, &target, &reference) {
        Ok(value) => value,
        Err(error) => return Err(failed(entry, target, error)),
    };
    let prepared = (|| {
        let payload =
            String::from_utf8(encoded(&handoff)?).map_err(|_| invalid("handoff is not UTF-8"))?;
        if payload.len() > 4096 {
            return Err(invalid("executable handoff locator exceeds its bound"));
        }
        let marker = String::from_utf8(encoded(&identity()?)?)
            .map_err(|_| invalid("process identity is not UTF-8"))?;
        Ok((payload, marker))
    })();
    let (payload, marker) = match prepared {
        Ok(value) => value,
        Err(error) => return Err(failed(entry, target, error)),
    };
    let error = Command::new(executable)
        .args(arguments)
        .env("PSE_HOST_REEXEC", marker)
        .env(HANDOFF, payload)
        .exec();
    // exec returned, so no replacement process exists. Keep the original performing guard.
    Err(failed(entry, target, error.into()))
}

#[cfg(test)]
mod tests;
