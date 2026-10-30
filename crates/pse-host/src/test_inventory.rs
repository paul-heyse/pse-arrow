// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive pages of an original captured population; never execution or fixture authority.
use crate::{
    HostError,
    discovery::Inventory,
    envelope::{CliInvocationId, Envelope},
    invocation::InvocationStore,
    managed::result::ManagedSummary,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use std::{
    collections::BTreeSet,
    fs::{self, OpenOptions},
    io::{self, Read as _},
    num::NonZeroUsize,
    os::unix::fs::{MetadataExt as _, OpenOptionsExt as _},
    path::Path,
};

/// One indivisible actual selected test, preserving its original compiler context.
#[derive(Debug, Serialize, JsonSchema)]
pub struct TestRow {
    /// Original group, package, binary and exact harness ID.
    pub identity: crate::discovery::TestIdentity,
    /// Same-build target name, independent of the test's spelling.
    pub target: String,
    /// Original Cargo target kinds.
    pub target_kinds: Vec<String>,
    /// Actual compiler-artifact features, not guessed from declarations.
    pub features: BTreeSet<String>,
    /// Original listed ignored status.
    pub ignored: bool,
}

/// A source-bound whole-record window, independent of original workload acceptance.
#[derive(Debug, Serialize, JsonSchema)]
pub struct TestPage {
    /// Selected original operation, not this observer.
    pub original_invocation: CliInvocationId,
    /// Exact bytes read from the selected retained inventory.
    pub inventory_sha256: String,
    /// Actual selected population size in this inventory.
    pub selected_tests: usize,
    /// Complete rows only; a byte budget never truncates a test identity.
    pub records: Vec<TestRow>,
    /// Explicit continuation, bound to these exact inventory bytes.
    pub next: Option<Cursor>,
    /// A next record exceeding the byte budget stays unread rather than being split.
    pub next_record_bytes: Option<usize>,
}

/// Read position only. Decoding this value grants no process or execution authority.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Cursor {
    /// Original operation identity.
    pub invocation: CliInvocationId,
    /// Original inventory byte identity.
    pub inventory_sha256: String,
    /// Zero-based selected-test index, never a name-only continuation.
    pub offset: usize,
}
fn invalid(message: &str) -> HostError {
    HostError::Ownership(format!("test inventory: {message}"))
}
fn read_private(path: &Path) -> Result<Vec<u8>, HostError> {
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o022 != 0
    {
        return Err(invalid(
            "retained input is not an owned regular evidence file",
        ));
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}
fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn checked_parent(path: &Path) -> Result<(), HostError> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_dir()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o777 != 0o700
    {
        return Err(invalid(
            "evidence directory is not private original custody",
        ));
    }
    Ok(())
}

/// Read the selected protected original inventory without preparing or verifying executable
/// eligibility. This scans/decodes the retained inventory; the window bounds output, not scan cost.
pub fn read(
    store: &InvocationStore,
    selector: &str,
    session: Option<&str>,
    cursor: Option<&Cursor>,
    limit: NonZeroUsize,
    byte_budget: NonZeroUsize,
) -> Result<Envelope<TestPage>, HostError> {
    let id = store
        .resolve(selector, session)?
        .ok_or_else(|| invalid("original invocation is absent"))?;
    let result = store
        .result::<serde::de::IgnoredAny, ManagedSummary>(&id)?
        .ok_or_else(|| invalid("original workflow has no published terminal evidence"))?;
    let workflow = result
        .payload
        .workflow
        .ok_or_else(|| invalid("selected operation is not a captured test workflow"))?;
    let action = crate::test_workflow::execute::Action::from_command(&result.command)
        .ok_or_else(|| invalid("original workflow command is unsupported"))?;
    if action == crate::test_workflow::execute::Action::Check {
        return Err(invalid(
            "compiler checks have no actual selected test population",
        ));
    }
    workflow
        .terminal
        .validate(&id, action)
        .map_err(|error| HostError::Workflow(Box::new(error)))?;
    let directory = store.directory(&id)?;
    let parent = directory.path().join("workflow");
    checked_parent(&parent)?;
    let report = read_private(&parent.join("report.json"))?;
    if report.len() as u64 != workflow.terminal.report.bytes
        || digest(&report) != workflow.terminal.report.sha256
    {
        return Err(invalid("original report bytes changed"));
    }
    #[derive(Deserialize)]
    struct Address {
        invocation: CliInvocationId,
        inventory: Option<String>,
    }
    let address: Address = serde_json::from_slice(&report).map_err(io::Error::other)?;
    if address.invocation != id || address.inventory.as_deref() != Some("inventory/inventory.json")
    {
        return Err(invalid(
            "original report has no supported inventory association",
        ));
    }
    let parent = parent.join("inventory");
    checked_parent(&parent)?;
    let bytes = read_private(&parent.join("inventory.json"))?;
    let sha = digest(&bytes);
    let inventory: Inventory = serde_json::from_slice(&bytes).map_err(io::Error::other)?;
    if !inventory.capture_complete() {
        return Err(invalid(
            "original inventory lacks complete supported discovery capture",
        ));
    }
    let total = inventory.selected_tests_iter().count();
    let offset = position(cursor, &id, &sha, total)?;
    let mut records = Vec::new();
    let mut used = 0usize;
    let mut next_record_bytes = None;
    for identity in inventory
        .selected_tests_iter()
        .skip(offset)
        .take(limit.get())
    {
        let group = &inventory.groups[identity.group];
        let suite = group
            .listing
            .rust_suites
            .get(&nextest_metadata::RustBinaryId::new(&identity.binary_id))
            .ok_or_else(|| invalid("selected binary is absent"))?;
        let artifact = group
            .build
            .artifacts
            .iter()
            .find(|artifact| artifact.executable.captured == suite.binary.binary_path.as_std_path())
            .ok_or_else(|| invalid("selected binary has no same-build artifact"))?;
        let case = suite
            .test_cases
            .get(&nextest_metadata::TestCaseName::new(&identity.test_name))
            .ok_or_else(|| invalid("selected test is absent"))?;
        let row = TestRow {
            identity,
            target: artifact.cargo.target.name.clone(),
            target_kinds: artifact
                .cargo
                .target
                .kind
                .iter()
                .map(ToString::to_string)
                .collect(),
            features: artifact.cargo.features.iter().cloned().collect(),
            ignored: case.ignored,
        };
        let extent = serde_json::to_vec(&row).map_err(io::Error::other)?.len();
        if used.saturating_add(extent) > byte_budget.get() {
            next_record_bytes = Some(extent);
            break;
        }
        used += extent;
        records.push(row);
    }
    let next_offset = offset + records.len();
    let next = (next_offset < total).then(|| Cursor {
        invocation: id.clone(),
        inventory_sha256: sha.clone(),
        offset: next_offset,
    });
    let mut response = Envelope::response(
        "result tests",
        format!("{} of {total} original selected tests", records.len()),
        TestPage {
            original_invocation: id,
            inventory_sha256: sha,
            selected_tests: total,
            records,
            next,
            next_record_bytes,
        },
    );
    response.observation.scope = "retained_native_test_inventory_page".into();
    response.observation.limitations=vec!["Listing evidence only: fixture readiness, executable reuse eligibility, physical settlement and test acceptance remain with their original owners.".into(),"The full retained inventory is decoded; page limits bound whole-record output, not underlying scan cost.".into()];
    Ok(response)
}
fn position(
    cursor: Option<&Cursor>,
    id: &CliInvocationId,
    digest: &str,
    total: usize,
) -> Result<usize, HostError> {
    match cursor {
        None => Ok(0),
        Some(cursor)
            if &cursor.invocation == id
                && cursor.inventory_sha256 == digest
                && cursor.offset <= total =>
        {
            Ok(cursor.offset)
        }
        Some(_) => Err(invalid(
            "continuation identity, inventory bytes or extent changed",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn continuation_cannot_rebind_original_identity_or_changed_inventory() {
        let id = CliInvocationId::new();
        let cursor = Cursor {
            invocation: id.clone(),
            inventory_sha256: "original".into(),
            offset: 2,
        };
        assert_eq!(position(Some(&cursor), &id, "original", 3).unwrap(), 2);
        assert!(position(Some(&cursor), &CliInvocationId::new(), "original", 3).is_err());
        assert!(position(Some(&cursor), &id, "changed", 3).is_err());
        assert!(position(Some(&cursor), &id, "original", 1).is_err());
    }
    #[test]
    fn evidence_reads_refuse_symlinked_files_and_nonprivate_directories() {
        use std::os::unix::fs::{DirBuilderExt as _, PermissionsExt as _};
        let scratch = tempfile::tempdir().unwrap();
        let path = scratch.path().join("private");
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        checked_parent(&path).unwrap();
        let original = path.join("original.json");
        fs::write(&original, b"{\"original\":true}").unwrap();
        fs::set_permissions(&original, fs::Permissions::from_mode(0o400)).unwrap();
        assert_eq!(read_private(&original).unwrap(), b"{\"original\":true}");
        std::os::unix::fs::symlink(&original, path.join("alias.json")).unwrap();
        assert!(read_private(&path.join("alias.json")).is_err());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(checked_parent(&path).is_err());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        fs::set_permissions(&original, fs::Permissions::from_mode(0o666)).unwrap();
        assert!(read_private(&original).is_err());
    }
}
