// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Compact presentation custody. Original invocations and artifacts have independent owners.

use super::{
    InvocationDirectory, InvocationEvidence, InvocationStore, META_LIMIT, directory, evidence_file,
    invalid, publish, synchronize_publication_parent, unix_millis,
};
use crate::{
    HostError,
    envelope::{CliInvocationId, Envelope, RedactionPolicy, Settlement, SubjectState},
    environment::Environment,
    invocation_view::ResultSummary,
    managed::result::{ManagedSummary, TerminalResult},
    paths::{CheckoutState, RetentionClass, RetentionEvidence},
    selection::IgnoredPolicy,
    test_workflow::execute::{Action, Input, Outcome, ReportReference},
};
use rustix::fs::{FlockOperation, OFlags, flock};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::Read,
    os::unix::{
        ffi::OsStringExt,
        fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Path, PathBuf},
    time::SystemTime,
};

const WEEK_MS: u64 = 7 * 24 * 60 * 60 * 1000;
const CLASSIFICATION: &str = "compact-presentation.json";
const COMPACT_LIMIT: u64 = 64 * 1024;
const SWEEP_LIMIT: usize = 128;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    checkout: String,
    invocation: CliInvocationId,
    state: RecordState,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "disposition", rename_all = "snake_case", deny_unknown_fields)]
enum RecordState {
    Active {
        settled_unix_ms: u64,
        summary: Box<ResultSummary>,
        original_report: ReportReference,
        holds: BTreeMap<String, String>,
    },
    Retired {
        released_unix_ms: u64,
    },
}

pub(crate) enum CompactObservation {
    Original,
    Available(ResultSummary),
    Retired,
    Expired,
}

/// Exact preview or disposition from the compact presentation owner.
#[derive(Debug, Serialize, JsonSchema)]
pub struct CompactRetirement {
    invocation: CliInvocationId,
    disposition: &'static str,
    eligible_after_unix_ms: Option<u64>,
    holds: BTreeMap<String, String>,
    /// Pass this token with the exact ID to release the reviewed compact record.
    pub release: Option<String>,
}

/// Result of one bounded checkout-local automatic release pass.
#[derive(Debug, Default, Serialize, JsonSchema)]
pub struct CompactSweep {
    released: usize,
    forgotten: usize,
    limited: bool,
    busy: usize,
    checkouts: usize,
    unavailable: usize,
}

/// Sweep only accessible, registered checkouts of the selected Git repository.
pub fn sweep_registered_compact_results(
    root: &Path,
    environment: &Environment,
) -> Result<CompactSweep, HostError> {
    let selected = CheckoutState::resolve(root, environment)?;
    let inventory = crate::repository::observe_worktrees(root)?;
    let mut result = CompactSweep {
        unavailable: inventory.failures.len(),
        ..CompactSweep::default()
    };
    for checkout in inventory.entries {
        let Some(path) = checkout.worktree else {
            continue;
        };
        let path = PathBuf::from(std::ffi::OsString::from_vec(path.as_bytes().to_vec()));
        let Ok(state) = CheckoutState::resolve(&path, environment) else {
            result.unavailable += 1;
            continue;
        };
        if state.repository_id != selected.repository_id {
            result.unavailable += 1;
            continue;
        }
        let sweep = InvocationStore::new(state).sweep_compact_results()?;
        result.checkouts += sweep.checkouts;
        result.released += sweep.released;
        result.forgotten += sweep.forgotten;
        result.limited |= sweep.limited;
        result.busy += sweep.busy;
    }
    Ok(result)
}

fn read<T: DeserializeOwned>(path: &Path) -> Result<Option<T>, HostError> {
    let file = match evidence_file(path) {
        Err(HostError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(None);
        }
        result => result?,
    };
    let mut bytes = Vec::new();
    file.take(COMPACT_LIMIT + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > COMPACT_LIMIT {
        return Err(invalid("compact record exceeds its storage bound"));
    }
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|error| invalid(format!("invalid compact record: {error}")))
}

fn replace(path: &Path, record: &Record) -> Result<(), HostError> {
    let parent = path
        .parent()
        .ok_or_else(|| invalid("compact record has no parent"))?;
    let mut staged = tempfile::NamedTempFile::new_in(parent)?;
    serde_json::to_writer(&mut staged, record)
        .map_err(|error| invalid(format!("compact record encoding failed: {error}")))?;
    if staged.as_file().metadata()?.len() > COMPACT_LIMIT {
        return Err(invalid("compact record exceeds its storage bound"));
    }
    staged.as_file().sync_all()?;
    staged.persist(path).map_err(|error| error.error)?;
    synchronize_publication_parent(parent, path)
}

fn confirm_root(path: &Path, opened: &File) -> Result<(), HostError> {
    directory(path, false)?;
    let opened = opened.metadata()?;
    let named = fs::symlink_metadata(path)?;
    if (opened.dev(), opened.ino()) != (named.dev(), named.ino()) {
        return Err(invalid("compact result store changed during access"));
    }
    Ok(())
}

fn lock_root(
    path: &Path,
    operation: FlockOperation,
    create: bool,
) -> Result<Option<File>, HostError> {
    match directory(path, create) {
        Err(HostError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(None);
        }
        result => result?,
    }
    let opened = OpenOptions::new()
        .read(true)
        .custom_flags((OFlags::DIRECTORY | OFlags::NOFOLLOW).bits() as i32)
        .open(path)?;
    flock(&opened, operation).map_err(std::io::Error::from)?;
    confirm_root(path, &opened)?;
    Ok(Some(opened))
}

fn settled_test(result: &Envelope<ManagedSummary>) -> Option<&ReportReference> {
    let workflow = result.payload.workflow.as_ref()?;
    (workflow.terminal.action == Action::Test
        && matches!(
            workflow.terminal.outcome,
            Outcome::TestsSucceeded | Outcome::Failed
        )
        && workflow.terminal.required_capture_complete
        && result.settlement == Settlement::Complete
        && result.payload.native_lifetime_pending == Some(false)
        && !result.payload.borrowed_reservation
        && matches!(
            result.subject_state,
            SubjectState::Succeeded | SubjectState::Failed
        ))
    .then_some(&workflow.terminal.report)
}

impl InvocationStore {
    fn compact_root(&self) -> PathBuf {
        self.state.root.join("compact-results")
    }

    fn compact_path(&self, id: &CliInvocationId) -> PathBuf {
        self.compact_root().join(format!("{id}.json"))
    }

    fn read_compact(&self, id: &CliInvocationId) -> Result<Option<Record>, HostError> {
        let record: Option<Record> = read(&self.compact_path(id))?;
        if let Some(record) = &record
            && (record.checkout != self.state.checkout_id.as_str() || &record.invocation != id)
        {
            return Err(invalid(
                "compact record differs from its checkout or original ID",
            ));
        }
        Ok(record)
    }

    fn require_compact_class(
        &self,
        id: &CliInvocationId,
    ) -> Result<InvocationDirectory, HostError> {
        let original = self.directory(id)?;
        let classified: Option<CliInvocationId> = read(&original.path.join(CLASSIFICATION))?;
        if classified.as_ref() != Some(id) {
            return Err(invalid("original compact presentation class is unproved"));
        }
        Ok(original)
    }

    /// Publish the disposable presentation of a positively settled original test.
    /// The original ledger and report remain with the performing owner.
    pub fn publish_compact_test_result(
        &self,
        original: &InvocationDirectory,
        terminal: &TerminalResult<ManagedSummary>,
        policy: &RedactionPolicy,
    ) -> Result<(), HostError> {
        let response = terminal.response();
        let Some(report) = settled_test(response) else {
            return Ok(());
        };
        if self.directory(original.id())?.path != original.path {
            return Err(invalid(
                "original directory differs from compact result owner",
            ));
        }
        let input: Input = original
            .evidence(InvocationEvidence::WorkflowInput)?
            .ok_or_else(|| invalid("original ordinary-test input is absent"))?;
        input
            .validate_original(original.id(), &original.command)
            .map_err(|error| invalid(error.to_string()))?;
        if input.profile != "default"
            || input.request.filters.ignore_default_filter
            || input.request.ignored != IgnoredPolicy::Default
        {
            return Ok(());
        }
        Self::require_original_report(original, report)?;
        self.publish_compact(
            original,
            ResultSummary::from_terminal(response, policy),
            report.clone(),
            unix_millis(SystemTime::now())?,
        )
    }

    fn require_original_report(
        original: &InvocationDirectory,
        report: &ReportReference,
    ) -> Result<(), HostError> {
        let result: Envelope<ManagedSummary> =
            serde_json::from_slice(&original.paired_settlement_bytes()?)
                .map_err(|error| invalid(format!("invalid original settlement: {error}")))?;
        super::validate_result(original.id(), &original.command, &result)?;
        let observed = settled_test(&result)
            .ok_or_else(|| invalid("original test is no longer a positively settled terminal"))?;
        if observed.path != report.path
            || observed.sha256 != report.sha256
            || observed.byte_length != report.byte_length
        {
            return Err(invalid(
                "original settled test report differs from compact classification",
            ));
        }
        Ok(())
    }

    fn publish_compact(
        &self,
        original: &InvocationDirectory,
        summary: ResultSummary,
        original_report: ReportReference,
        settled_unix_ms: u64,
    ) -> Result<(), HostError> {
        let root = self.compact_root();
        let _guard = lock_root(&root, FlockOperation::LockExclusive, true)?
            .ok_or_else(|| invalid("compact result store disappeared during publication"))?;
        let id = original.id();
        let classified: Option<CliInvocationId> = read(&original.path.join(CLASSIFICATION))?;
        if classified.as_ref() == Some(id) {
            return Ok(());
        }
        if classified.is_some() {
            return Err(invalid(
                "compact presentation class is bound to a different original",
            ));
        }
        if self.read_compact(id)?.is_none() {
            publish(
                &root,
                &format!("{id}.json"),
                &Record {
                    checkout: self.state.checkout_id.as_str().to_owned(),
                    invocation: id.clone(),
                    state: RecordState::Active {
                        settled_unix_ms,
                        summary: Box::new(summary),
                        original_report,
                        holds: BTreeMap::new(),
                    },
                },
                Some(COMPACT_LIMIT),
            )?;
        }
        publish(&original.path, CLASSIFICATION, id, Some(META_LIMIT))
    }

    pub(crate) fn observe_compact(
        &self,
        id: &CliInvocationId,
    ) -> Result<(CompactObservation, Option<File>), HostError> {
        let original = self.directory(id)?;
        let borrower = lock_root(&self.compact_root(), FlockOperation::LockShared, false)?;
        let classified: Option<CliInvocationId> = read(&original.path.join(CLASSIFICATION))?;
        if classified.is_none() {
            return Ok((CompactObservation::Original, None));
        }
        if classified.as_ref() != Some(id) {
            return Err(invalid(
                "compact presentation class is bound to a different original",
            ));
        }
        let now = unix_millis(SystemTime::now())?;
        let observed = match self.read_compact(id)?.map(|record| record.state) {
            Some(RecordState::Active { summary, .. }) => CompactObservation::Available(*summary),
            Some(RecordState::Retired { released_unix_ms })
                if now < released_unix_ms.saturating_add(WEEK_MS) =>
            {
                CompactObservation::Retired
            }
            Some(RecordState::Retired { .. }) | None => CompactObservation::Expired,
        };
        Ok((observed, borrower))
    }

    fn preview_record(&self, record: &Record, now: u64) -> Result<CompactRetirement, HostError> {
        let mut result = CompactRetirement {
            invocation: record.invocation.clone(),
            disposition: "retired",
            eligible_after_unix_ms: None,
            holds: BTreeMap::new(),
            release: None,
        };
        if let RecordState::Active {
            settled_unix_ms,
            original_report,
            holds,
            ..
        } = &record.state
        {
            let eligible = settled_unix_ms.saturating_add(WEEK_MS);
            result.eligible_after_unix_ms = Some(eligible);
            result.holds = holds.clone();
            if now < eligible {
                result.disposition = "retained";
            } else if RetentionClass::DisposablePresentation.eligible(RetentionEvidence {
                terminal: true,
                settled: true,
                generation_in_use: false,
                owner_retains: !holds.is_empty(),
            }) {
                let original = self.require_compact_class(&record.invocation)?;
                Self::require_original_report(&original, original_report)?;
                let encoded = serde_json::to_vec(record)
                    .map_err(|error| invalid(format!("compact record encoding failed: {error}")))?;
                result.disposition = "eligible";
                result.release = Some(
                    Sha256::digest(encoded)
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect(),
                );
            } else {
                result.disposition = "held";
            }
        }
        Ok(result)
    }

    /// Preview an exact compact record, or release it after verifying the preview token.
    pub fn retire_compact_result(
        &self,
        id: &CliInvocationId,
        release: Option<&str>,
    ) -> Result<CompactRetirement, HostError> {
        let lock = if release.is_some() {
            FlockOperation::NonBlockingLockExclusive
        } else {
            FlockOperation::LockShared
        };
        let _guard = lock_root(&self.compact_root(), lock, false)?
            .ok_or_else(|| invalid("compact result store is unavailable"))?;
        let mut record = self
            .read_compact(id)?
            .ok_or_else(|| invalid("compact result is unavailable"))?;
        let now = unix_millis(SystemTime::now())?;
        let mut preview = self.preview_record(&record, now)?;
        if let Some(token) = release {
            if preview.disposition == "retired" {
                return Ok(preview);
            }
            if preview.release.as_deref() != Some(token) {
                return Err(invalid(
                    "compact release token no longer matches eligible evidence",
                ));
            }
            self.release_record(&mut record, now)?;
            preview.disposition = "retired";
            preview.release = None;
        }
        Ok(preview)
    }

    fn release_record(&self, record: &mut Record, now: u64) -> Result<(), HostError> {
        // One atomic replacement discards the compact bytes and publishes the witness.
        // No other member or generation is unlinked by this operation.
        record.state = RecordState::Retired {
            released_unix_ms: now,
        };
        replace(&self.compact_path(&record.invocation), record)
    }

    /// Set a named consumer's hold condition, or remove that consumer's hold.
    pub fn hold_compact_result(
        &self,
        id: &CliInvocationId,
        consumer: &str,
        condition: Option<&str>,
    ) -> Result<(), HostError> {
        if consumer.is_empty()
            || consumer.len() > 128
            || consumer.chars().any(char::is_control)
            || condition.is_some_and(|value| {
                value.is_empty() || value.len() > 1024 || value.chars().any(char::is_control)
            })
        {
            return Err(invalid(
                "compact hold requires a consumer and release condition",
            ));
        }
        let _guard = lock_root(&self.compact_root(), FlockOperation::LockExclusive, false)?
            .ok_or_else(|| invalid("compact result store is unavailable"))?;
        self.require_compact_class(id)?;
        let mut record = self
            .read_compact(id)?
            .ok_or_else(|| invalid("compact result is unavailable"))?;
        let RecordState::Active { holds, .. } = &mut record.state else {
            return Err(invalid("compact result is already retired"));
        };
        if let Some(condition) = condition {
            holds.insert(consumer.to_owned(), condition.to_owned());
        } else {
            holds.remove(consumer);
        }
        replace(&self.compact_path(id), &record)
    }

    /// Release eligible records and forget expired witnesses for this authenticated checkout.
    pub fn sweep_compact_results(&self) -> Result<CompactSweep, HostError> {
        self.sweep_at(unix_millis(SystemTime::now())?)
    }

    fn sweep_at(&self, now: u64) -> Result<CompactSweep, HostError> {
        let root = self.compact_root();
        let lock = match lock_root(&root, FlockOperation::NonBlockingLockExclusive, false) {
            Err(HostError::Io(error)) if error.kind() == std::io::ErrorKind::WouldBlock => {
                return Ok(CompactSweep {
                    busy: 1,
                    checkouts: 1,
                    ..CompactSweep::default()
                });
            }
            result => result?,
        };
        let Some(_guard) = lock else {
            return Ok(CompactSweep::default());
        };
        let mut result = CompactSweep {
            checkouts: 1,
            ..CompactSweep::default()
        };
        for entry in fs::read_dir(&root)? {
            let path = entry?.path();
            let Some(id) = path
                .file_name()
                .and_then(|name| name.to_str())
                .and_then(|name| name.strip_suffix(".json"))
                .and_then(|name| CliInvocationId::try_from(name.to_owned()).ok())
            else {
                continue;
            };
            let Some(mut record) = self.read_compact(&id)? else {
                continue;
            };
            if result.released + result.forgotten == SWEEP_LIMIT {
                result.limited = true;
                break;
            }
            match &record.state {
                RecordState::Retired { released_unix_ms }
                    if now >= released_unix_ms.saturating_add(WEEK_MS) =>
                {
                    self.require_compact_class(&id)?;
                    fs::remove_file(&path)?;
                    synchronize_publication_parent(&root, &path)?;
                    result.forgotten += 1;
                }
                RecordState::Active { .. }
                    if self.preview_record(&record, now)?.release.is_some() =>
                {
                    self.release_record(&mut record, now)?;
                    result.released += 1;
                }
                _ => {}
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        invocation::InvocationSpec, managed::result::WorkflowSummary,
        test_workflow::execute::TerminalRecord,
    };

    fn fixture() -> (tempfile::TempDir, InvocationStore, InvocationDirectory, u64) {
        let temporary = tempfile::tempdir().unwrap();
        let environment = Environment::from([(
            "XDG_STATE_HOME".into(),
            temporary.path().join("state").into_os_string(),
        )]);
        let state = CheckoutState::from_paths(
            &temporary.path().join("checkout"),
            &temporary.path().join("common.git"),
            &environment,
        )
        .unwrap();
        let store = InvocationStore::new(state);
        let invocation = CliInvocationId::new();
        let directory = store
            .begin(InvocationSpec {
                invocation_id: invocation.clone(),
                command: "test".into(),
                session: Some("fixture".into()),
                parent: None,
                operation: (),
                control: crate::generation::PreparedGeneration {
                    generation: "generation.fixture".into(),
                    protocol: 1,
                    manifest_digest: "a".repeat(64),
                },
                environment_digest: "b".repeat(64),
                scientific: Vec::new(),
            })
            .unwrap();
        let report = ReportReference {
            path: "report.json".into(),
            sha256: "c".repeat(64),
            byte_length: 12,
        };
        let mut original = Envelope::response(
            "test",
            "original terminal",
            ManagedSummary {
                bundle: None,
                native_lifetime_pending: Some(false),
                workflow: Some(WorkflowSummary {
                    terminal: TerminalRecord {
                        protocol: "pse.test-workflow".into(),
                        version: 1,
                        invocation: invocation.clone(),
                        action: Action::Test,
                        outcome: Outcome::TestsSucceeded,
                        report: report.clone(),
                        required_capture_complete: true,
                    },
                    report: "owner report reference".into(),
                }),
                child_exit: Some(0),
                stdout: None,
                stderr: None,
                borrowed_reservation: false,
                service_result: None,
            },
        );
        original.invocation_id = invocation;
        original.subject_state = SubjectState::Succeeded;
        original.settlement = Settlement::Complete;
        for name in ["settlement.json", "result.json"] {
            publish(directory.path(), name, &original, Some(META_LIMIT)).unwrap();
        }
        let settled = unix_millis(SystemTime::now()).unwrap() - WEEK_MS - 1_000;
        store
            .publish_compact(
                &directory,
                ResultSummary::from_terminal(&original, &RedactionPolicy::default()),
                report,
                settled,
            )
            .unwrap();
        (temporary, store, directory, settled)
    }

    #[test]
    fn automatic_release_uses_settlement_clock_holds_and_live_borrowers() {
        let (_temporary, store, directory, settled) = fixture();
        let id = directory.id();
        assert_eq!(store.sweep_at(settled + WEEK_MS - 1).unwrap().released, 0);
        let token = store
            .retire_compact_result(id, None)
            .unwrap()
            .release
            .unwrap();
        store
            .hold_compact_result(id, "assessment", Some("until review closes"))
            .unwrap();
        assert!(store.retire_compact_result(id, Some(&token)).is_err());
        assert_eq!(store.sweep_at(settled + WEEK_MS).unwrap().released, 0);
        store.hold_compact_result(id, "assessment", None).unwrap();

        let (view, borrower) = store.observe_compact(id).unwrap();
        assert!(matches!(view, CompactObservation::Available(_)));
        assert_eq!(store.sweep_at(settled + WEEK_MS).unwrap().busy, 1);
        drop(borrower);
        assert_eq!(store.sweep_at(settled + WEEK_MS).unwrap().released, 1);
        assert!(matches!(
            store.observe_compact(id).unwrap().0,
            CompactObservation::Retired
        ));
        assert!(directory.path().join("result.json").exists());
        assert_eq!(
            store.sweep_at(settled + 2 * WEEK_MS - 1).unwrap().forgotten,
            0
        );
        assert_eq!(store.sweep_at(settled + 2 * WEEK_MS).unwrap().forgotten, 1);
        assert!(matches!(
            store.observe_compact(id).unwrap().0,
            CompactObservation::Expired
        ));
        assert!(directory.path().join("settlement.json").exists());
    }

    #[test]
    fn changed_settlement_fences_release_and_repeat_keeps_minimal_witness() {
        let (_temporary, store, directory, _settled) = fixture();
        let id = directory.id();
        let token = store
            .retire_compact_result(id, None)
            .unwrap()
            .release
            .unwrap();
        let original = fs::read(directory.path().join("settlement.json")).unwrap();
        fs::write(directory.path().join("settlement.json"), b"changed").unwrap();
        assert!(store.retire_compact_result(id, Some(&token)).is_err());
        fs::write(directory.path().join("settlement.json"), original).unwrap();
        let first = store.retire_compact_result(id, Some(&token)).unwrap();
        assert_eq!(first.disposition, "retired");
        let witness = fs::read(store.compact_path(id)).unwrap();
        let repeat = store.retire_compact_result(id, Some(&token)).unwrap();
        assert_eq!(repeat.disposition, "retired");
        assert_eq!(fs::read(store.compact_path(id)).unwrap(), witness);
        assert!(!String::from_utf8(witness).unwrap().contains("summary"));
    }
}
