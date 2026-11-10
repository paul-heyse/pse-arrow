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
use surrealdb_expr::{
    expr::{Expr, statements::define::DefineStatement},
    parser::TopLevelExpr,
};
use surrealdb_types::{Object, RecordId, ToSql, Value};

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

/// Data-affecting actions derived from the exact frozen inputs, in execution order.
/// These are preview facts, never proof of completed effects or execution authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RolloutEffect {
    /// A named conversion may change every existing value of this field.
    FieldMigration {
        /// Selected table.
        table: String,
        /// Selected scalar field.
        field: String,
        /// Exact declared target kind, resolved against separately witnessed named types.
        target_kind: String,
        /// Table-scoped one-shot migration name.
        name: String,
        /// Exact engine-normalized transform over the old value.
        using: String,
        /// Explicit fallback; acceptance is a separate domain obligation.
        fallback: Option<String>,
        /// Identity of the complete declaration, including constraints.
        declaration_sha256: String,
    },
    /// Irreversibly discard undeclared top-level values on a table.
    TableCleanup {
        /// Selected table.
        table: String,
        /// Table-scoped migration name.
        name: String,
        /// Exact admitted cleanup statement identity.
        declaration_sha256: String,
    },
    /// Explicit removal. Definition reversal cannot restore discarded values.
    Removal {
        /// Native entity family.
        family: String,
        /// Parent table for table-scoped entities.
        scope: Option<String>,
        /// Selected entity name.
        name: String,
    },
    /// Supplier completion barrier, distinct from activation and index readiness.
    AwaitMigrations {
        /// Tables whose background migrations must finish.
        tables: Vec<String>,
        /// Original wait budget. Expiry is not rollback.
        timeout_secs: Option<u64>,
    },
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
    /// Data effects captured before catalog normalization can discard duplicates.
    effects: Vec<RolloutEffect>,
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
    ApplyFiles {
        files: Vec<FrozenRolloutFile>,
    },
    RemoveEntities {
        entities: Vec<Entity>,
    },
    AwaitMigrations {
        tables: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        timeout_secs: Option<u64>,
    },
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

    /// Forward effects in the same phase order used by the supplier executor.
    pub fn effects(&self) -> impl Iterator<Item = &RolloutEffect> {
        [RolloutPhase::Start, RolloutPhase::Complete]
            .into_iter()
            .flat_map(|phase| {
                self.steps
                    .iter()
                    .filter(move |step| step.phase == phase)
                    .flat_map(|step| &step.effects)
            })
    }
    /// Effects of one supplier phase, before any execution or authority is granted.
    pub fn phase_effects(&self, phase: RolloutPhase) -> impl Iterator<Item = &RolloutEffect> {
        self.steps
            .iter()
            .filter(move |step| step.phase == phase)
            .flat_map(|step| &step.effects)
    }

    /// Digest of exact original manifest bytes, distinct from any installed schema hash.
    pub fn manifest_sha256(&self) -> &str {
        &self.manifest_sha256
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
            let (kind, files, removed, requires_independent_data_witness, effects) = match step
                .action
            {
                Action::ApplyFiles { files } => {
                    if files.is_empty() || files.len() > MAX_STEPS {
                        return Err(invalid("frozen schema rollout file list differs"));
                    }
                    let mut effects = Vec::new();
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
                        effects.extend(frozen_effects(&sql)?);
                    }
                    let data_witness = !effects.is_empty();
                    ("apply_files", files, Vec::new(), data_witness, effects)
                }
                Action::RemoveEntities { entities } => {
                    if entities.is_empty()
                        || entities.len() > MAX_STEPS
                        || step.phase == RolloutPhase::Start
                        || entities.iter().any(|entity| {
                            !(if matches!(entity.kind.as_str(), "function" | "module") {
                                entity.name.split("::").all(identifier)
                            } else {
                                identifier(&entity.name)
                            }) || match entity.kind.as_str() {
                                "table" | "type" | "module" => entity.scope.is_some(),
                                "function" => entity.scope.is_some(),
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
                    let effects = entities
                        .iter()
                        .map(|entity| RolloutEffect::Removal {
                            family: entity.kind.clone(),
                            scope: entity.scope.clone(),
                            name: entity.name.clone(),
                        })
                        .collect();
                    (
                        "remove_entities",
                        Vec::new(),
                        entities,
                        requires_independent_data_witness,
                        effects,
                    )
                }
                Action::AwaitMigrations {
                    tables,
                    timeout_secs,
                } => {
                    let unique = tables.iter().collect::<BTreeSet<_>>();
                    if tables.is_empty()
                        || tables.len() > MAX_STEPS
                        || unique.len() != tables.len()
                        || tables.iter().any(|table| !identifier(table))
                        || timeout_secs == Some(0)
                    {
                        return Err(invalid("unsupported frozen migration completion barrier"));
                    }
                    let effect = RolloutEffect::AwaitMigrations {
                        tables,
                        timeout_secs,
                    };
                    (
                        "await_migrations",
                        Vec::new(),
                        Vec::new(),
                        false,
                        vec![effect],
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
                effects,
            });
        }
        if !steps
            .iter()
            .any(|step| !step.files.is_empty() && step.phase != RolloutPhase::Rollback)
        {
            return Err(invalid("frozen rollout requires forward frozen SQL"));
        }
        let rollout = Self {
            id: manifest.id,
            manifest_sha256: sha256(raw.as_bytes()),
            source_schema_hash: manifest.source_schema_hash,
            target_schema_hash: manifest.target_schema_hash,
            steps,
        };
        rollout.validate_migration_barriers()?;
        Ok(rollout)
    }

    fn validate_migration_barriers(&self) -> Result<(), CanonicalError> {
        let mut pending = BTreeSet::new();
        let mut names = BTreeSet::new();
        for effect in self.effects() {
            match effect {
                RolloutEffect::FieldMigration { table, name, .. }
                | RolloutEffect::TableCleanup { table, name, .. } => {
                    if !names.insert((table, name)) {
                        return Err(invalid("frozen migration name is reused"));
                    }
                    pending.insert(table);
                }
                RolloutEffect::AwaitMigrations { tables, .. } => {
                    for table in tables {
                        pending.remove(table);
                    }
                }
                RolloutEffect::Removal { .. } => {}
            }
        }
        if !pending.is_empty() {
            return Err(invalid(
                "frozen migration requires a subsequent completion barrier",
            ));
        }
        Ok(())
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

fn frozen_effects(sql: &str) -> Result<Vec<RolloutEffect>, CanonicalError> {
    let ast =
        surrealdb_expr::parser::parse(sql).map_err(|_| invalid("frozen migration parse failed"))?;
    if ast.expressions.is_empty() {
        return Err(invalid("frozen rollout SQL is empty"));
    }
    let mut effects = Vec::new();
    for expression in ast.expressions {
        if let TopLevelExpr::Expr(Expr::Alter(statement)) = &expression {
            use surrealdb_expr::expr::{
                Literal,
                statements::alter::{AlterStatement, AlterTableStatement},
            };
            let AlterStatement::Table(table) = statement.as_ref() else {
                return Err(invalid("unsupported frozen alteration"));
            };
            let Some(migration) = &table.migrate else {
                return Err(invalid(
                    "frozen table alteration requires an explicit cleanup",
                ));
            };
            let name = table.name.to_sql();
            let mut remaining = table.clone();
            remaining.name = Expr::Literal(Literal::None);
            remaining.migrate = None;
            if !identifier(&name)
                || !identifier(&migration.name)
                || remaining != AlterTableStatement::default()
            {
                return Err(invalid(
                    "frozen cleanup cannot carry unrelated table effects",
                ));
            }
            effects.push(RolloutEffect::TableCleanup {
                table: name,
                name: migration.name.clone(),
                declaration_sha256: sha256(statement.to_sql().as_bytes()),
            });
            continue;
        }
        let TopLevelExpr::Expr(Expr::Define(statement)) = expression else {
            return Err(invalid("unsupported frozen rollout statement"));
        };
        parse_definition_input(&statement.to_sql(), DefinitionInput::FrozenRollout)
            .map_err(|_| invalid("frozen rollout SQL is not a supported schema declaration"))?;
        let DefineStatement::Field(field) = statement.as_ref() else {
            continue;
        };
        let Some(migration) = &field.migrate else {
            continue;
        };
        let table = field.what.to_sql();
        let name = field.name.to_sql();
        if !identifier(&table)
            || !identifier(&name)
            || !identifier(&migration.name)
            || matches!(name.as_str(), "id" | "in" | "out")
            || field.inline
            || field.reference.is_some()
            || field.computed.is_some()
        {
            return Err(invalid("unsupported frozen native migration field"));
        }
        effects.push(RolloutEffect::FieldMigration {
            table,
            field: name,
            target_kind: field
                .field_kind
                .as_ref()
                .ok_or_else(|| invalid("native migration requires a target kind"))?
                .to_sql(),
            name: migration.name.clone(),
            using: migration.using.to_sql(),
            fallback: migration.fallback.as_ref().map(ToSql::to_sql),
            declaration_sha256: sha256(statement.to_sql().as_bytes()),
        });
    }
    Ok(effects)
}

fn string_is(object: &Object, name: &str, expected: &str) -> bool {
    matches!(object.get(name), Some(Value::String(value)) if value == expected)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn migration_fixture(sql: &str, barrier: &str) -> tempfile::TempDir {
        let project = tempfile::tempdir().unwrap();
        let frozen = project.path().join("rollout/schema");
        std::fs::create_dir_all(&frozen).unwrap();
        std::fs::write(frozen.join("fixture.surql"), sql).unwrap();
        std::fs::write(
            project.path().join("rollout.toml"),
            format!(
                r#"
id = "rollout"
name = "unsigned ordinal"
source_schema_hash = "{source}"
target_schema_hash = "{target}"
compatibility = "phased"
[[steps]]
id = "convert"
phase = "start"
kind = "apply_files"
files = [{{ path = "schema/fixture.surql", hash = "{digest}" }}]
{barrier}
"#,
                source = sha256(b"source"),
                target = sha256(b"target"),
                digest = sha256(sql.as_bytes())
            ),
        )
        .unwrap();
        project
    }

    #[test]
    fn frozen_native_migration_captures_effect_and_requires_later_conformance() {
        let sql = "DEFINE FIELD OVERWRITE ordinal ON canonical_result_batches TYPE u64 MIGRATE EXISTING ROWS AS ordinal_u64 USING <u64>$value FALLBACK 0u64;";
        let barrier = r#"
[[steps]]
id = "conform"
phase = "complete"
kind = "await_migrations"
tables = ["canonical_result_batches"]
timeout_secs = 30
"#;
        let project = migration_fixture(sql, barrier);
        let rollout = FrozenRollout::read(&project.path().join("rollout.toml")).unwrap();
        assert!(rollout.requires_independent_data_witness());
        let effects = rollout.effects().collect::<Vec<_>>();
        assert!(
            matches!(effects[0], RolloutEffect::FieldMigration { table, field, name, fallback: Some(_), .. }
            if table == "canonical_result_batches" && field == "ordinal" && name == "ordinal_u64")
        );
        assert!(matches!(
            effects[1],
            RolloutEffect::AwaitMigrations {
                timeout_secs: Some(30),
                ..
            }
        ));
        let unguarded = migration_fixture(sql, "");
        assert!(FrozenRollout::read(&unguarded.path().join("rollout.toml")).is_err());
        let wrong = migration_fixture(
            sql,
            &barrier.replace("canonical_result_batches", "canonical_other"),
        );
        assert!(FrozenRollout::read(&wrong.path().join("rollout.toml")).is_err());
        let repeated = migration_fixture(&format!("{sql}{sql}"), barrier);
        assert!(FrozenRollout::read(&repeated.path().join("rollout.toml")).is_err());
    }

    #[test]
    fn frozen_cleanup_is_explicit_destructive_and_cannot_smuggle_other_alterations() {
        let barrier = r#"
[[steps]]
id = "conform"
phase = "complete"
kind = "await_migrations"
tables = ["canonical_probe"]
"#;
        let project = migration_fixture(
            "ALTER TABLE canonical_probe MIGRATE EXISTING ROWS AS remove_extra REMOVE UNDECLARED FIELDS;",
            barrier,
        );
        let rollout = FrozenRollout::read(&project.path().join("rollout.toml")).unwrap();
        assert!(rollout.requires_independent_data_witness());
        assert!(
            matches!(rollout.effects().next(), Some(RolloutEffect::TableCleanup {table, name, ..}) if table=="canonical_probe" && name=="remove_extra")
        );
        for sql in [
            "ALTER TABLE canonical_probe SCHEMALESS;",
            "ALTER TABLE canonical_probe SCHEMALESS MIGRATE EXISTING ROWS AS remove_extra REMOVE UNDECLARED FIELDS;",
            "ALTER TABLE canonical_probe COMPACT;",
        ] {
            let project = migration_fixture(sql, barrier);
            assert!(
                FrozenRollout::read(&project.path().join("rollout.toml")).is_err(),
                "{sql}"
            );
        }
    }

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
