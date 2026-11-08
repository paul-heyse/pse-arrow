// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Read-only identity of the selected SurrealKit schema rollout inputs.

use super::{CanonicalError, DefinitionInput, invalid, is_sha256, parse_definition_input, sha256};
use rustix::fs::{self, Mode, OFlags, ResolveFlags};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs::File,
    io::Read as _,
    os::unix::fs::MetadataExt as _,
    path::{Component, Path},
};
use surrealdb_types::{Object, RecordId, Value};

const MAX_STEPS: usize = 256;
const MAX_MANIFEST_BYTES: usize = 256 * 1024;

/// The exact authored schema manifest and SQL selected for preview.
///
/// The hashes attest bytes and the supported action shape. Chain lint, live
/// predecessor verification and execution authority are separate owner facts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct FrozenRollout {
    /// Stable SurrealKit rollout identifier.
    id: String,
    /// SHA-256 of the original manifest bytes.
    manifest_sha256: String,
    /// SurrealKit source schema hash, distinct from a canonical marker digest.
    source_schema_hash: String,
    /// SurrealKit target schema hash, distinct from a canonical marker digest.
    target_schema_hash: String,
    /// Authored step order, including rollback for input identity.
    steps: Vec<FrozenRolloutStep>,
}

/// An ordered entry in the exact frozen rollout manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
struct FrozenRolloutStep {
    /// Original step identifier, unique across the manifest.
    id: String,
    /// Original step phase.
    phase: RolloutPhase,
    /// Exact pinned tool action kind.
    kind: &'static str,
    /// This operation can remove existing stored values or records.
    requires_independent_data_witness: bool,
    /// SHA-256 of the pinned tool's journal step serialization.
    checksum: String,
    /// Frozen SQL files in dispatch order.
    files: Vec<FrozenRolloutFile>,
    /// Declared removal targets in dispatch order.
    removed: Vec<Entity>,
}

/// Frozen, checked SQL path and exact byte digest.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct FrozenRolloutFile {
    /// Path inside the directory of this rollout's frozen copies.
    path: String,
    /// SHA-256 of the authored and verified bytes.
    hash: String,
}

/// The pinned rollout phase vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RolloutPhase {
    /// Expand before cutover.
    Start,
    /// Contract after cutover.
    Complete,
    /// Undo a start in the tool's own protocol.
    Rollback,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    id: String,
    name: String,
    source_schema_hash: String,
    target_schema_hash: String,
    compatibility: String,
    #[serde(default)]
    renames: Vec<toml::Value>,
    steps: Vec<Step>,
}

// Keep field order aligned with the selected rc.1 journal checksum contract.
#[derive(Deserialize, Serialize)]
struct Step {
    id: String,
    phase: RolloutPhase,
    #[serde(flatten)]
    action: Action,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Action {
    ApplyFiles { files: Vec<FrozenRolloutFile> },
    RemoveEntities { entities: Vec<Entity> },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct Entity {
    kind: String,
    scope: Option<String>,
    name: String,
}

pub(super) fn bounded_file(
    path: &Path,
    limit: usize,
    private: bool,
) -> Result<String, CanonicalError> {
    if path
        .components()
        .any(|component| component == Component::ParentDir)
    {
        return Err(invalid("frozen rollout path traverses a parent directory"));
    }
    let file = File::from(
        fs::openat2(
            fs::CWD,
            path,
            OFlags::RDONLY | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
            ResolveFlags::NO_SYMLINKS,
        )
        .map_err(|_| invalid("frozen rollout input is unavailable or traverses a symbolic link"))?,
    );
    let metadata = file
        .metadata()
        .map_err(|_| invalid("frozen rollout input metadata is unavailable"))?;
    if !metadata.is_file() || metadata.nlink() != 1 {
        return Err(invalid("frozen rollout input must be one regular file"));
    }
    if private
        && (metadata.mode() & 0o077 != 0 || metadata.uid() != rustix::process::geteuid().as_raw())
    {
        return Err(invalid("rollout settlement must be private and owned"));
    }
    if metadata.len() > limit as u64 {
        return Err(CanonicalError::PayloadLimit);
    }
    let mut value = String::new();
    file.take(limit as u64 + 1)
        .read_to_string(&mut value)
        .map_err(|_| invalid("frozen rollout input must be UTF-8"))?;
    if value.len() > limit {
        return Err(CanonicalError::PayloadLimit);
    }
    Ok(value)
}

fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

impl FrozenRollout {
    /// The exact persisted record selector for this rollout.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Forward removals of a field or table require independent evidence of the
    /// old data and its required postconditions, even when the journal and
    /// installed catalog agree.
    pub fn requires_independent_data_witness(&self) -> bool {
        self.steps.iter().any(|step| {
            step.phase != RolloutPhase::Rollback && step.requires_independent_data_witness
        })
    }

    /// Read one default-module phased schema rollout from its authored manifest.
    ///
    /// Only hashed `.surql` copies and explicit schema removals are selected.
    /// Template substitution, arbitrary SQL, rename hints and data migrations
    /// require their own independent effect contracts.
    pub fn read(manifest_path: &Path) -> Result<Self, CanonicalError> {
        let raw = bounded_file(manifest_path, MAX_MANIFEST_BYTES, false)?;
        // A parse error can include arbitrary authored input. Report shape only.
        let manifest: Manifest = toml::from_str(&raw)
            .map_err(|_| invalid("unsupported frozen schema rollout manifest"))?;
        if !identifier(&manifest.id)
            || manifest_path.extension().and_then(|value| value.to_str()) != Some("toml")
            || manifest_path.file_stem().and_then(|value| value.to_str()) != Some(&manifest.id)
            || manifest.name.is_empty()
            || manifest.compatibility != "phased"
            || !manifest.renames.is_empty()
            || !is_sha256(&manifest.source_schema_hash)
            || !is_sha256(&manifest.target_schema_hash)
            || manifest.steps.is_empty()
            || manifest.steps.len() > MAX_STEPS
        {
            return Err(invalid("frozen schema rollout manifest identity differs"));
        }
        let frozen = manifest_path.with_extension("");
        let mut ids = BTreeSet::new();
        let mut remaining_bytes = super::super::PAYLOAD_BYTES;
        let mut steps = Vec::with_capacity(manifest.steps.len());
        for step in manifest.steps {
            if !identifier(&step.id) || !ids.insert(step.id.clone()) {
                return Err(invalid("frozen schema rollout step identity differs"));
            }
            let checksum = sha256(
                &serde_json::to_vec(&step)
                    .map_err(|_| invalid("cannot encode frozen schema rollout step"))?,
            );
            let (kind, files, removed, requires_independent_data_witness) = match step.action {
                Action::ApplyFiles { files } => {
                    if files.is_empty() || files.len() > MAX_STEPS {
                        return Err(invalid("frozen schema rollout file list differs"));
                    }
                    for file in &files {
                        let path = Path::new(&file.path);
                        if !file.path.starts_with("schema/")
                            || path.extension().and_then(|value| value.to_str()) != Some("surql")
                            || file.path.split('/').any(|part| {
                                part.is_empty() || matches!(part, "." | "..") || part.contains('\\')
                            })
                            || !is_sha256(&file.hash)
                        {
                            return Err(invalid(
                                "frozen schema rollout SQL path or digest differs",
                            ));
                        }
                        let sql = bounded_file(&frozen.join(path), remaining_bytes, false)?;
                        remaining_bytes -= sql.len();
                        if sha256(sql.as_bytes()) != file.hash || sql.contains("${") {
                            return Err(invalid("frozen rollout SQL or effective bytes differ"));
                        }
                        if parse_definition_input(&sql, DefinitionInput::FrozenRollout)
                            .map_err(|_| {
                                invalid("frozen rollout SQL is not a supported schema declaration")
                            })?
                            .is_empty()
                        {
                            return Err(invalid("frozen rollout SQL is empty"));
                        }
                    }
                    ("apply_files", files, Vec::new(), false)
                }
                Action::RemoveEntities { entities } => {
                    if entities.is_empty()
                        || entities.len() > MAX_STEPS
                        || step.phase == RolloutPhase::Start
                        || entities.iter().any(|entity| {
                            !identifier(&entity.name)
                                || match entity.kind.as_str() {
                                    "table" => entity.scope.is_some(),
                                    "field" | "event" | "index" => {
                                        !entity.scope.as_deref().is_some_and(identifier)
                                    }
                                    _ => true,
                                }
                        })
                    {
                        return Err(invalid("unsupported frozen rollout schema removal"));
                    }
                    let requires_independent_data_witness = entities
                        .iter()
                        .any(|entity| matches!(entity.kind.as_str(), "field" | "table"));
                    (
                        "remove_entities",
                        Vec::new(),
                        entities,
                        requires_independent_data_witness,
                    )
                }
            };
            steps.push(FrozenRolloutStep {
                id: step.id,
                phase: step.phase,
                kind,
                requires_independent_data_witness,
                checksum,
                files,
                removed,
            });
        }
        if !steps
            .iter()
            .any(|step| !step.files.is_empty() && step.phase != RolloutPhase::Rollback)
        {
            return Err(invalid("frozen rollout requires forward frozen SQL"));
        }
        Ok(Self {
            id: manifest.id,
            manifest_sha256: sha256(raw.as_bytes()),
            source_schema_hash: manifest.source_schema_hash,
            target_schema_hash: manifest.target_schema_hash,
            steps,
        })
    }

    /// Compare an exact journal row to the frozen manifest and phase steps.
    ///
    /// The caller reads the row by this rollout ID on the selected deployment.
    /// A matching journal is one settlement premise. Observed effects and
    /// physical drain belong to the maintenance owner.
    pub fn journal_records_phase(&self, journal: &Object, phase: RolloutPhase) -> bool {
        let status = match phase {
            RolloutPhase::Start => "ready_to_complete",
            RolloutPhase::Complete => "completed",
            RolloutPhase::Rollback => return false,
        };
        let Some(Value::Array(actual)) = journal.get("steps") else {
            return false;
        };
        let mut expected = self
            .steps
            .iter()
            .filter(|step| step.phase == RolloutPhase::Start)
            .chain(self.steps.iter().filter(|step| {
                phase == RolloutPhase::Complete && step.phase == RolloutPhase::Complete
            }));
        journal.get("id")
            == Some(&Value::RecordId(RecordId::new(
                "__rollout",
                self.id.clone(),
            )))
            && string_is(journal, "status", status)
            && string_is(journal, "manifest_checksum", &self.manifest_sha256)
            && string_is(journal, "source_schema_hash", &self.source_schema_hash)
            && string_is(journal, "target_schema_hash", &self.target_schema_hash)
            && actual
                .iter()
                .all(|observed| match (expected.next(), observed) {
                    (Some(step), Value::Object(row)) => {
                        string_is(row, "step_id", &step.id)
                            && string_is(
                                row,
                                "phase",
                                if step.phase == RolloutPhase::Start {
                                    "start"
                                } else {
                                    "complete"
                                },
                            )
                            && string_is(row, "kind", step.kind)
                            && string_is(row, "checksum", &step.checksum)
                            && string_is(row, "status", "completed")
                    }
                    _ => false,
                })
            && expected.next().is_none()
    }
}

fn string_is(object: &Object, name: &str, expected: &str) -> bool {
    matches!(object.get(name), Some(Value::String(value)) if value == expected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_rollout_refuses_substitution_unsupported_sql_and_linked_inputs() {
        let project = tempfile::tempdir().unwrap();
        let manifest = project.path().join("rollout.toml");
        let frozen = project.path().join("rollout/schema");
        let schema = frozen.join("fixture.surql");
        std::fs::create_dir_all(&frozen).unwrap();
        let stage = |sql: &str, path: &str| {
            std::fs::write(&schema, sql).unwrap();
            std::fs::write(
                &manifest,
                format!(
                    r#"
id = "rollout"
name = "rollout"
source_schema_hash = "{source}"
target_schema_hash = "{target}"
compatibility = "phased"
[[steps]]
id = "expand"
phase = "start"
kind = "apply_files"
[[steps.files]]
path = "{path}"
hash = "{digest}"
"#,
                    source = sha256(b"source"),
                    target = sha256(b"target"),
                    digest = sha256(sql.as_bytes()),
                ),
            )
            .unwrap();
        };
        let statement = "DEFINE TABLE canonical_probe SCHEMAFULL;";
        let path = "schema/fixture.surql";
        stage(statement, path);
        assert_eq!(FrozenRollout::read(&manifest).unwrap().id(), "rollout");
        for sql in [
            "DEFINE TABLE canonical_probe COMMENT '${INPUT}';",
            "DEFINE TABLE canonical_probe COMMENT '$${ESCAPED}';",
            "DEFINE TABLE canonical_probe; CREATE canonical_probe:test;",
            "DEFINE TABLE canonical_probe DROP SCHEMAFULL;",
            "DEFINE FIELD OVERWRITE value ON canonical_probe TYPE int MIGRATE EXISTING ROWS AS to_int USING <int>$value; DEFINE FIELD OVERWRITE value ON canonical_probe TYPE int;",
        ] {
            stage(sql, path);
            assert!(FrozenRollout::read(&manifest).is_err(), "{sql}");
        }
        stage(statement, "schema/../schema/fixture.surql");
        assert!(FrozenRollout::read(&manifest).is_err());
        stage(statement, path);
        let elsewhere = project.path().join("outside.surql");
        std::fs::rename(&schema, &elsewhere).unwrap();
        std::os::unix::fs::symlink(&elsewhere, &schema).unwrap();
        assert!(FrozenRollout::read(&manifest).is_err());
    }
}
