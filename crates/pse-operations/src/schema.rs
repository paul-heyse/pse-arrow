// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact writer admission and explicit, preservation-first schema transitions (ADR-0146).
//! Ordinary opening verifies existing readiness; upgrades require a quiescent explicit migrate.

use pse_operations_queries::queries::store as statements;

use crate::error::{Classify, OperationsError};
use crate::generated::{RECORD_SQL, SCHEMA_FINGERPRINT_HEX, SCHEMA_SQL};
use crate::store::Store;

/// The hand-written access paths, defaults and grants applied after the generated DDL.
pub const PHYSICAL_SQL: &str = include_str!("../physical.sql");

/// The recorded comment's prefix: the fingerprint's frame.
const RECORD_PREFIX: &str = "pse.ops.schema.v1 ";

/// Serializes schema creation and reset across processes (`pse_ops` in ASCII).
pub(crate) const SCHEMA_LOCK: i64 = 0x7073_655f_6f70_7300;

/// What a store's schema is, compared with this build.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SchemaStatus {
    /// There is no `pse_ops` schema; only explicit [`Store::create`] creates it.
    Absent,
    /// The schema records this build's fingerprint.
    Current,
    /// The schema records another fingerprint, or none (a store this build did not
    /// create); an explicit supported migration is required.
    Mismatch {
        /// The recorded fingerprint, if any.
        recorded: Option<String>,
    },
}

/// The result of explicit [`Store::create`] or validation-only [`Store::open`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opened {
    /// The schema was absent and has been created.
    Created,
    /// The schema already records this build's fingerprint.
    Current,
}

/// The fingerprint a schema comment records, if it has the recorded form.
fn recorded(comment: Option<&str>) -> Option<&str> {
    comment.and_then(|comment| comment.strip_prefix(RECORD_PREFIX))
}

impl Store {
    /// The fingerprint this build creates and accepts, as lowercase hexadecimal.
    pub const fn expected_schema() -> &'static str {
        SCHEMA_FINGERPRINT_HEX
    }

    /// Compare the store's schema with this build without changing anything.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn schema_status(&self) -> Result<SchemaStatus, OperationsError> {
        let client = self.admin_client().await?;
        let comment = statements::schema_comment()
            .bind(&client)
            .opt()
            .await
            .classify(self.target())?;
        Ok(match comment {
            None => SchemaStatus::Absent,
            Some(comment) => match recorded(comment.as_deref()) {
                Some(fingerprint) if fingerprint == SCHEMA_FINGERPRINT_HEX => SchemaStatus::Current,
                Some(fingerprint) => SchemaStatus::Mismatch {
                    recorded: Some(fingerprint.to_owned()),
                },
                None => SchemaStatus::Mismatch { recorded: None },
            },
        })
    }

    /// Validate an existing schema without creating or upgrading it.
    /// # Errors
    /// Typed absence, mismatch, incomplete readiness or classified driver failures.
    pub async fn open(&self) -> Result<Opened, OperationsError> {
        let client = self.admin_client().await?;
        let comment = statements::schema_comment()
            .bind(&client)
            .opt()
            .await
            .classify(self.target())?;
        match comment {
            None => Err(OperationsError::SchemaAbsent {
                target: self.target().clone(),
            }),
            Some(comment) if recorded(comment.as_deref()) == Some(SCHEMA_FINGERPRINT_HEX) => {
                drop(client);
                self.retain_schema_lease().await?;
                Ok(Opened::Current)
            }
            Some(comment)
                if comment
                    .as_deref()
                    .is_some_and(|value| value.starts_with(PENDING_PREFIX)) =>
            {
                Err(refuse(
                    MigrationRefusal::NotReady,
                    "explicitly resume the committed incomplete transition",
                ))
            }
            Some(comment) => Err(OperationsError::SchemaMismatch {
                target: self.target().clone(),
                recorded: recorded(comment.as_deref()).map(str::to_owned),
                expected: SCHEMA_FINGERPRINT_HEX,
            }),
        }
    }

    /// Explicitly create an absent schema, or validate an already current schema.
    /// # Errors
    /// Mismatch, active generation or classified driver failures.
    pub async fn create(&self) -> Result<Opened, OperationsError> {
        let began = std::time::Instant::now();
        loop {
            if self.schema_status().await? != SchemaStatus::Absent {
                return self.open().await;
            }
            match self.create_schema().await {
                Ok(opened) => {
                    self.retain_schema_lease().await?;
                    return Ok(opened);
                }
                Err(OperationsError::MigrationRefused {
                    reason: MigrationRefusal::ActiveGeneration,
                    ..
                }) if began.elapsed() < self.acquire_timeout() => {
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await
                }
                Err(error) => return Err(error),
            }
        }
    }

    async fn create_schema(&self) -> Result<Opened, OperationsError> {
        let target = self.target();
        let mut client = self.admin_client().await?;
        let tx = client.transaction().await.classify(target)?;
        let locked = tx
            .query_one("SELECT pg_try_advisory_xact_lock($1)", &[&SCHEMA_LOCK])
            .await
            .classify(target)?
            .get::<_, bool>(0);
        if !locked {
            tx.rollback().await.classify(target)?;
            return Err(refuse(
                MigrationRefusal::ActiveGeneration,
                "a generation or competing creator owns the namespace",
            ));
        }
        let comment = statements::schema_comment()
            .bind(&tx)
            .opt()
            .await
            .classify(target)?;
        match comment {
            None => {
                // The generated DDL, the access paths and the fingerprint record: one
                // transaction, so a failed creation leaves no schema behind.
                for sql in [SCHEMA_SQL, PHYSICAL_SQL] {
                    tx.batch_execute(sql).await.classify(target)?;
                }
                tx.execute("INSERT INTO pse_ops.schema_support_state(history, shared_version, source, target, ready) VALUES ('catalog', $1, 'fresh', $2, true), ('operations', $1, 'fresh', $3, true)", &[&crate::generated::SHARED_VERSION, &crate::generated::CATALOG_FINGERPRINT_HEX, &crate::generated::OPERATIONS_FINGERPRINT_HEX]).await.classify(target)?;
                tx.batch_execute(RECORD_SQL).await.classify(target)?;
                tx.commit().await.classify(target)?;
                Ok(Opened::Created)
            }
            Some(comment) if recorded(comment.as_deref()) == Some(SCHEMA_FINGERPRINT_HEX) => {
                tx.rollback().await.classify(target)?;
                verify_ready(&client, target).await?;
                Ok(Opened::Current)
            }
            Some(comment)
                if comment
                    .as_deref()
                    .is_some_and(|value| value.starts_with(PENDING_PREFIX)) =>
            {
                tx.rollback().await.classify(target)?;
                Err(refuse(
                    MigrationRefusal::NotReady,
                    "a committed schema transition is incomplete; explicitly resume it",
                ))
            }
            Some(comment) => {
                tx.rollback().await.classify(target)?;
                Err(OperationsError::SchemaMismatch {
                    target: target.clone(),
                    recorded: recorded(comment.as_deref()).map(str::to_owned),
                    expected: SCHEMA_FINGERPRINT_HEX,
                })
            }
        }
    }
}

/// One immutable migration step with its owning qualified history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigrationStep {
    /// Qualified history table.
    pub history: String,
    /// Immutable migration version.
    pub version: i32,
    /// Immutable migration name.
    pub name: String,
    /// Refinery checksum of the immutable SQL.
    pub checksum: u64,
}
/// Exact history declaration and committed prefix.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigrationHistoryPlan {
    /// Qualified history table (distinct for each owner).
    pub table: String,
    /// Target support identity.
    pub target: String,
    /// Number of committed, checksum-verified steps.
    pub committed: usize,
    /// Complete immutable steps consumed by this lineage.
    pub steps: Vec<MigrationStep>,
}
/// Read-only inspection bound to exact source, target, histories and shared support.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigrationPlan {
    /// Exact supported source provenance.
    pub source: String,
    /// Exact combined target identity.
    pub target: String,
    /// Whether a committed incomplete transition is being resumed.
    pub pending: bool,
    /// Catalog/control-owned history.
    pub catalog: MigrationHistoryPlan,
    /// Operations-owned history.
    pub operations: MigrationHistoryPlan,
    /// Required shared support version, owned by catalog/control.
    pub shared_version: i32,
    /// Whether exact target readiness is already verified.
    pub ready: bool,
}
impl MigrationPlan {
    /// Ordered uncommitted steps, catalog owner first.
    pub fn pending_steps(&self) -> Vec<MigrationStep> {
        if self.ready {
            return Vec::new();
        }
        let mut steps = [&self.catalog, &self.operations]
            .into_iter()
            .flat_map(|h| h.steps.iter().skip(h.committed).cloned())
            .collect::<Vec<_>>();
        steps.sort_by_key(|s| {
            if s.history == CATALOG_HISTORY {
                if s.version == 1 { 0 } else { 6 }
            } else {
                s.version
            }
        });
        steps
    }
    fn require_expected(&self, expected: &Self) -> Result<(), OperationsError> {
        if self == expected {
            Ok(())
        } else {
            Err(refuse(
                MigrationRefusal::PlanChanged,
                "live source, histories or support changed since inspection",
            ))
        }
    }
}
/// Applied immutable steps and freshly verified final readiness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigrationReport {
    /// Steps committed by this invocation; a resumed committed prefix is excluded.
    pub applied: Vec<MigrationStep>,
    /// Final exact target support and history inspection.
    pub final_ready: MigrationPlan,
}
fn history_plan(
    table: &str,
    target: &str,
    committed: usize,
    steps: &[refinery::Migration],
) -> MigrationHistoryPlan {
    MigrationHistoryPlan {
        table: table.to_owned(),
        target: target.to_owned(),
        committed,
        steps: steps
            .iter()
            .map(|m| MigrationStep {
                history: table.to_owned(),
                version: m.version(),
                name: m.name().to_owned(),
                checksum: m.checksum(),
            })
            .collect(),
    }
}
async fn current_plan(
    client: &tokio_postgres::Client,
    target: &crate::error::Target,
) -> Result<MigrationPlan, OperationsError> {
    let (mut catalog, mut operations) = transitions()?;
    let sources = client
        .query(
            "SELECT history,source FROM pse_ops.schema_support_state ORDER BY history",
            &[],
        )
        .await
        .classify(target)?;
    for row in sources {
        let history: String = row.try_get(0).classify(target)?;
        let source: String = row.try_get(1).classify(target)?;
        if history == "catalog" && source == "fresh-v25f" {
            catalog.drain(..1);
        }
        if history == "operations" && source == "fresh-v25h" {
            operations.drain(..7);
        } else if history == "operations" && source == "fresh-v25g" {
            operations.drain(..6);
        } else if history == "operations" && source == "fresh-v25f" {
            operations.drain(..5);
        } else if history == "operations" && source == "fresh-v25e" {
            operations.drain(..4);
        }
    }
    let cp = history_prefix(client, &catalog, CATALOG_HISTORY, target).await?;
    let op = history_prefix(client, &operations, OPERATIONS_HISTORY, target).await?;
    Ok(MigrationPlan {
        source: SCHEMA_FINGERPRINT_HEX.into(),
        target: SCHEMA_FINGERPRINT_HEX.into(),
        pending: false,
        catalog: history_plan(
            CATALOG_HISTORY,
            crate::generated::CATALOG_FINGERPRINT_HEX,
            cp,
            &catalog,
        ),
        operations: history_plan(
            OPERATIONS_HISTORY,
            crate::generated::OPERATIONS_FINGERPRINT_HEX,
            op,
            &operations,
        ),
        shared_version: crate::generated::SHARED_VERSION,
        ready: true,
    })
}

/// Typed transition refusals; no reason string is interpreted as policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MigrationRefusal {
    /// Recorded source is not an explicitly supported predecessor.
    UnknownSource,
    /// Live inspection no longer agrees with the reviewed exact plan.
    PlanChanged,
    /// Committed history differs from the immutable transition declaration.
    ChecksumConflict,
    /// An opened generation still owns runtime schema admission.
    ActiveGeneration,
    /// Schema shape or target support does not agree with its record.
    Drift,
    /// Readiness is missing, incomplete or its owner connection has ended.
    NotReady,
}
fn refuse(reason: MigrationRefusal, detail: impl Into<String>) -> OperationsError {
    OperationsError::MigrationRefused {
        reason,
        detail: detail.into(),
    }
}
/// Exact committed Plan 25d operational baseline; unrelated or interim sources are refused.
pub const MIGRATION_SOURCE: &str =
    "a660d5317c58a235a0717b2d24e808153d833d6bb41753ad850cf22217c3da0d";
/// Exact completed Plan 25e baseline; its committed histories must be catalog 1 / operations 4.
const STUDY_MIGRATION_SOURCE: &str =
    "60293f46ba51830bb2a376d9554cae582e4f715234fa7163c38e372994de915b";
const INVENTORY_MIGRATION_SOURCE: &str =
    "bb4d7025a28c62d4cef788de61277b31b4c1ae88fa6f0160d94dcc6f521cd759";
const JOB_IDENTITY_MIGRATION_SOURCE: &str =
    "7a1649ecfeec229fd3fa87f96e8eb7239003146c66783edb6a0f9b66ea6f3d4e";
const NATIVE_BACKENDS_MIGRATION_SOURCE: &str =
    "13d5852cb2f9124849aef19386746094eacb72cf6d5768eddc6f35e2958b0b64";
const PLAN25H_OPERATIONS: &str = "ffac44aa0986c9e8b1fa2546e12ca6dfe7604c32e16ff1b7f68eef263b7994d7";
const PLAN25F_OPERATIONS: &str = "13dd4f91810792475382a021425807a8ef42551bc2eaea71d6f9a8702009d00a";
const FRESH_PLAN25E_ORIGIN: &str =
    "fresh-v25e:60293f46ba51830bb2a376d9554cae582e4f715234fa7163c38e372994de915b";
const PLAN25E_CATALOG: &str = "3aa8a326827f2b06cde68942b520e1480b0512b1112e91b15ba33f0fdff83927";
const PLAN25E_OPERATIONS: &str = "b3e1ba34cd0f964ee1941021e52371f3d83b12373bd6b2b3656f4eebeea1c9d6";
mod plan25e_layout {
    include!("../test-fixtures/plan25e/layout.rs");
}
mod plan25f_layout {
    include!("../test-fixtures/plan25f/layout.rs");
}
mod plan25g_layout {
    include!("../test-fixtures/plan25g/layout.rs");
}
const PENDING_PREFIX: &str = "pse.ops.transition.v1 ";
const CATALOG_HISTORY: &str = "pse_ops.catalog_schema_history";
const OPERATIONS_HISTORY: &str = "pse_ops.operations_schema_history";
fn migration_error(error: refinery::Error) -> OperationsError {
    match error.kind() {
        refinery::error::Kind::DivergentVersion(..) | refinery::error::Kind::MissingVersion(..) => {
            refuse(MigrationRefusal::ChecksumConflict, error.to_string())
        }
        _ => OperationsError::Internal {
            target: crate::error::Target::migration(),
            source: crate::error::DriverError::new(error),
        },
    }
}
fn migration(name: &str, sql: &str) -> Result<refinery::Migration, OperationsError> {
    refinery::Migration::unapplied(name, sql).map_err(migration_error)
}
fn transitions() -> Result<(Vec<refinery::Migration>, Vec<refinery::Migration>), OperationsError> {
    Ok((
        vec![
            migration(
                "V1__catalog_identity",
                include_str!("../migrations/V1__catalog_identity.sql"),
            )?,
            migration(
                "V2__catalog_retirement_inventory",
                include_str!("../migrations/V2__catalog_retirement_inventory.sql"),
            )?,
        ],
        vec![
            migration(
                "V1__operations_identity",
                include_str!("../migrations/V1__operations_identity.sql"),
            )?,
            migration(
                "V2__native_node_limit",
                "ALTER TYPE pse_ops.native_termination ADD VALUE IF NOT EXISTS 'node_limit' AFTER 'iteration_limit';",
            )?,
            migration(
                "V3__shooting_attempt",
                "ALTER TYPE pse_ops.attempt_kind ADD VALUE IF NOT EXISTS 'shooting' AFTER 'simulation';",
            )?,
            migration(
                "V4__operations_ready",
                include_str!("../migrations/V4__operations_ready.sql"),
            )?,
            migration(
                "V5__study_occurrence_policy",
                include_str!("../migrations/V5__study_occurrence_policy.sql"),
            )?,
            migration(
                "V6__retirement_ready",
                include_str!("../migrations/V6__retirement_ready.sql"),
            )?,
            migration(
                "V7__operational_job_identity",
                include_str!("../migrations/V7__operational_job_identity.sql"),
            )?,
            migration(
                "V8__native_backends",
                include_str!("../migrations/V8__native_backends.sql"),
            )?,
        ],
    ))
}
async fn run_history(
    client: &mut tokio_postgres::Client,
    migrations: &[refinery::Migration],
    table: &str,
) -> Result<(), OperationsError> {
    let mut runner = refinery::Runner::new(migrations);
    runner.set_migration_table_name(table);
    runner.run_async(client).await.map_err(migration_error)?;
    Ok(())
}
/// Verify owned support declarations, independently of unrelated registry vocabulary.
pub(crate) async fn verify_ready(
    client: &tokio_postgres::Client,
    target: &crate::error::Target,
) -> Result<(), OperationsError> {
    let comment = statements::schema_comment()
        .bind(client)
        .opt()
        .await
        .classify(target)?;
    if comment
        .as_ref()
        .and_then(|comment| recorded(comment.as_deref()))
        != Some(SCHEMA_FINGERPRINT_HEX)
    {
        return Err(refuse(
            MigrationRefusal::NotReady,
            "schema record changed before shared generation admission",
        ));
    }
    let present = client
        .query_one(
            "SELECT to_regclass('pse_ops.schema_support_state') IS NOT NULL",
            &[],
        )
        .await
        .classify(target)?;
    if !present.get::<_, bool>(0) {
        return Err(refuse(
            MigrationRefusal::NotReady,
            "explicit migration must establish schema support and readiness",
        ));
    }
    let rows = client.query("SELECT history,shared_version,target,ready FROM pse_ops.schema_support_state ORDER BY history", &[]).await.classify(target)?;
    let expected = [
        ("catalog", crate::generated::CATALOG_FINGERPRINT_HEX),
        ("operations", crate::generated::OPERATIONS_FINGERPRINT_HEX),
    ];
    if rows.len() != expected.len()
        || rows
            .iter()
            .zip(expected)
            .any(|(r, (history, fingerprint))| {
                r.get::<_, String>(0) != history
                    || r.get::<_, i32>(1) != crate::generated::SHARED_VERSION
                    || r.get::<_, String>(2) != fingerprint
                    || !r.get::<_, bool>(3)
            })
    {
        return Err(refuse(
            MigrationRefusal::NotReady,
            "owned schema support is missing, incomplete or incompatible",
        ));
    }
    verify_layout(client, target, false).await?;
    let provenance = client
        .query(
            "SELECT history,source FROM pse_ops.schema_support_state ORDER BY history",
            &[],
        )
        .await
        .classify(target)?;
    let (catalog, operations) = transitions()?;
    for row in provenance {
        let history = row.get::<_, String>(0);
        let source = row.get::<_, String>(1);
        let known_source = if history == "catalog" {
            matches!(source.as_str(), "fresh" | "fresh-v25f" | MIGRATION_SOURCE)
        } else {
            matches!(
                source.as_str(),
                "fresh"
                    | "fresh-v25e"
                    | "fresh-v25f"
                    | "fresh-v25g"
                    | "fresh-v25h"
                    | STUDY_MIGRATION_SOURCE
            )
        };
        if !known_source {
            return Err(refuse(
                MigrationRefusal::NotReady,
                "schema support provenance is not a declared lineage",
            ));
        }
        let (table, declared) = if history == "catalog" {
            (
                CATALOG_HISTORY,
                if source == "fresh-v25f" {
                    &catalog[1..]
                } else {
                    catalog.as_slice()
                },
            )
        } else if source == "fresh-v25h" {
            (OPERATIONS_HISTORY, &operations[7..])
        } else if source == "fresh-v25g" {
            (OPERATIONS_HISTORY, &operations[6..])
        } else if source == "fresh-v25f" {
            (OPERATIONS_HISTORY, &operations[5..])
        } else if source == "fresh-v25e" {
            (OPERATIONS_HISTORY, &operations[4..])
        } else {
            (OPERATIONS_HISTORY, operations.as_slice())
        };
        if source == "fresh" {
            if history_prefix(client, declared, table, target).await? != 0 {
                return Err(refuse(
                    MigrationRefusal::ChecksumConflict,
                    "fresh schema lineage must not claim migration history",
                ));
            }
        } else {
            verify_history(client, declared, table, target).await?;
        }
    }
    Ok(())
}
async fn verify_history(
    client: &tokio_postgres::Client,
    expected: &[refinery::Migration],
    table: &str,
    target: &crate::error::Target,
) -> Result<(), OperationsError> {
    if history_prefix(client, expected, table, target).await? != expected.len() {
        return Err(refuse(
            MigrationRefusal::ChecksumConflict,
            "committed history is incomplete",
        ));
    }
    Ok(())
}
/// Full typed preflight prevents refinery's unchecked timestamp/row decoding from seeing malformed input.
fn checked_history_row(
    row: &tokio_postgres::Row,
    migration: &refinery::Migration,
) -> Result<(), OperationsError> {
    let malformed = |detail: String| refuse(MigrationRefusal::ChecksumConflict, detail);
    let version: i32 = row
        .try_get(0)
        .map_err(|e| malformed(format!("invalid history version: {e}")))?;
    let name: String = row
        .try_get(1)
        .map_err(|e| malformed(format!("invalid history name: {e}")))?;
    let checksum: String = row
        .try_get(2)
        .map_err(|e| malformed(format!("invalid history checksum: {e}")))?;
    let applied: String = row
        .try_get(3)
        .map_err(|e| malformed(format!("invalid history applied_on: {e}")))?;
    validate_history_values(version, &name, &checksum, &applied, migration)
}
fn validate_history_values(
    version: i32,
    name: &str,
    checksum: &str,
    applied: &str,
    migration: &refinery::Migration,
) -> Result<(), OperationsError> {
    if version != migration.version()
        || name != migration.name()
        || checksum != migration.checksum().to_string()
        || time::OffsetDateTime::parse(applied, &time::format_description::well_known::Rfc3339)
            .is_err()
    {
        return Err(refuse(
            MigrationRefusal::ChecksumConflict,
            "history version/name/checksum/timestamp is not an exact declared valid record",
        ));
    }
    Ok(())
}
/// Read both committed prefixes before changing readiness or any schema object.
async fn history_prefix(
    client: &tokio_postgres::Client,
    expected: &[refinery::Migration],
    table: &str,
    target: &crate::error::Target,
) -> Result<usize, OperationsError> {
    if !client
        .query_one("SELECT to_regclass($1) IS NOT NULL", &[&table])
        .await
        .classify(target)?
        .get::<_, bool>(0)
    {
        return Ok(0);
    }
    let relation = client
        .query_one(
            "SELECT relkind::text FROM pg_class WHERE oid=to_regclass($1)",
            &[&table],
        )
        .await
        .classify(target)?;
    if relation.try_get::<_, String>(0).classify(target)? != "r" {
        return Err(refuse(
            MigrationRefusal::ChecksumConflict,
            "history is not an owned ordinary table",
        ));
    }
    let columns=client.query("SELECT attname::text,format_type(atttypid,atttypmod),attnotnull FROM pg_attribute WHERE attrelid=to_regclass($1) AND attnum>0 AND NOT attisdropped ORDER BY attnum",&[&table]).await.classify(target)?;
    let expected_columns = [
        ("version", "integer", true),
        ("name", "character varying(255)", false),
        ("applied_on", "character varying(255)", false),
        ("checksum", "character varying(255)", false),
    ];
    if columns.len() != expected_columns.len() {
        return Err(refuse(
            MigrationRefusal::ChecksumConflict,
            "history column inventory differs from refinery's owned declaration",
        ));
    }
    for (row, (name, ty, required)) in columns.iter().zip(expected_columns) {
        if row.try_get::<_, String>(0).classify(target)? != name
            || row.try_get::<_, String>(1).classify(target)? != ty
            || row.try_get::<_, bool>(2).classify(target)? != required
        {
            return Err(refuse(
                MigrationRefusal::ChecksumConflict,
                "history field encoding differs from refinery's owned declaration",
            ));
        }
    }
    let primary=client.query("SELECT pg_get_constraintdef(oid,true) FROM pg_constraint WHERE conrelid=to_regclass($1) AND contype='p' AND convalidated AND conenforced",&[&table]).await.classify(target)?;
    if primary.len() != 1
        || primary[0].try_get::<_, String>(0).classify(target)? != "PRIMARY KEY (version)"
    {
        return Err(refuse(
            MigrationRefusal::ChecksumConflict,
            "history version key differs from refinery's owned declaration",
        ));
    }
    let rows = client
        .query(
            &format!("SELECT version,name,checksum,applied_on FROM {table} ORDER BY version"),
            &[],
        )
        .await
        .classify(target)?;
    if rows.len() > expected.len() {
        return Err(refuse(
            MigrationRefusal::ChecksumConflict,
            "history exceeds declared prefix",
        ));
    }
    for (row, migration) in rows.iter().zip(expected) {
        checked_history_row(row, migration)?;
    }
    Ok(rows.len())
}
async fn verify_layout(
    client: &tokio_postgres::Client,
    target: &crate::error::Target,
    legacy: bool,
) -> Result<(), OperationsError> {
    verify_progress_layout(
        client,
        target,
        LayoutCheckpoint {
            support: !legacy,
            inventory: !legacy,
            operations_version: if legacy { 0 } else { 8 },
        },
    )
    .await
}

/// Physical generation at one admitted catalog/operations history checkpoint.
/// Catalog support and inventory evolve independently of the operations history.
struct LayoutCheckpoint {
    support: bool,
    inventory: bool,
    operations_version: i32,
}

async fn verify_progress_layout(
    client: &tokio_postgres::Client,
    target: &crate::error::Target,
    checkpoint: LayoutCheckpoint,
) -> Result<(), OperationsError> {
    let LayoutCheckpoint {
        support,
        inventory,
        operations_version,
    } = checkpoint;
    let node_limit = operations_version >= 2;
    let shooting = operations_version >= 3;
    let study_policy = operations_version >= 5;
    let job_identity = operations_version >= 7;
    let rows = client.query("SELECT c.relname::text,a.attname::text,a.attnum::int,format_type(a.atttypid,a.atttypmod),NOT a.attnotnull FROM pg_attribute a JOIN pg_class c ON c.oid=a.attrelid JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='pse_ops' AND c.relkind IN ('r','p') AND a.attnum>0 AND NOT a.attisdropped AND c.relname <> ALL($1) ORDER BY c.relname,a.attnum", &[&vec!["catalog_schema_history", "operations_schema_history"]]).await.classify(target)?;
    let actual = rows
        .iter()
        .map(|r| {
            (
                r.get::<_, String>(0),
                r.get::<_, String>(1),
                r.get::<_, i32>(2),
                r.get::<_, String>(3),
                r.get::<_, bool>(4),
            )
        })
        .collect::<Vec<_>>();
    let columns = if job_identity {
        crate::generated::layout::COLUMNS
    } else if inventory {
        plan25g_layout::COLUMNS
    } else if study_policy {
        plan25f_layout::COLUMNS
    } else {
        plan25e_layout::COLUMNS
    };
    let expected = columns
        .iter()
        .filter(|(table, ..)| support || *table != "schema_support_state")
        .map(|(t, c, o, ty, n)| (t.to_string(), c.to_string(), *o, ty.to_string(), *n))
        .collect::<Vec<_>>();
    if actual != expected {
        return Err(refuse(
            MigrationRefusal::Drift,
            "physical column layout differs from declared source/target",
        ));
    }
    let constraints=client.query("SELECT c.relname::text,k.conname::text,k.contype::text FROM pg_constraint k JOIN pg_class c ON c.oid=k.conrelid JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='pse_ops' AND c.relname <> ALL($1) AND k.convalidated AND k.conenforced AND k.contype IN ('p','u','c','f') ORDER BY c.relname,k.conname", &[&vec!["catalog_schema_history","operations_schema_history"]]).await.classify(target)?;
    let actual = constraints
        .iter()
        .map(|row| {
            (
                row.get::<_, String>(0),
                row.get::<_, String>(1),
                row.get::<_, String>(2),
            )
        })
        .collect::<Vec<_>>();
    let declared_constraints = if job_identity {
        crate::generated::layout::CONSTRAINTS
    } else if inventory {
        plan25g_layout::CONSTRAINTS
    } else if study_policy {
        plan25f_layout::CONSTRAINTS
    } else {
        plan25e_layout::CONSTRAINTS
    };
    let mut expected = declared_constraints
        .iter()
        .filter(|(table, ..)| support || *table != "schema_support_state")
        .map(|(table, name, kind)| (table.to_string(), name.to_string(), kind.to_string()))
        .collect::<Vec<_>>();
    expected.sort();
    if actual != expected {
        return Err(refuse(
            MigrationRefusal::Drift,
            "required generated constraints differ or are not validated/enforced",
        ));
    }
    let constraints=client.query("SELECT COALESCE(c.relname,t.typname),k.conname::text,k.contype::text,pg_get_constraintdef(k.oid,true) FROM pg_constraint k LEFT JOIN pg_class c ON c.oid=k.conrelid LEFT JOIN pg_type t ON t.oid=k.contypid JOIN pg_namespace n ON n.oid=k.connamespace WHERE n.nspname='pse_ops' AND k.convalidated AND k.conenforced AND COALESCE(c.relname,t.typname) <> ALL($1) ORDER BY COALESCE(c.relname,t.typname),k.conname", &[&vec!["catalog_schema_history","operations_schema_history"]]).await.classify(target)?;
    let mut actual = constraints
        .iter()
        .map(|row| {
            (
                row.get::<_, String>(0),
                row.get::<_, String>(1),
                row.get::<_, String>(2),
                row.get::<_, String>(3),
            )
        })
        .collect::<Vec<_>>();
    let frozen = if job_identity {
        include_str!("../migrations/plan25h-constraint-signatures.json")
    } else if inventory {
        include_str!("../migrations/plan25g-constraint-signatures.json")
    } else if study_policy {
        include_str!("../migrations/plan25f-constraint-signatures.json")
    } else if support {
        include_str!("../migrations/plan25e-constraint-signatures.json")
    } else {
        include_str!("../migrations/plan25d-constraint-signatures.json")
    };
    let mut expected: Vec<(String, String, String, String)> = serde_json::from_str(frozen)
        .map_err(|error| {
            refuse(
                MigrationRefusal::Drift,
                format!("invalid immutable constraint declaration: {error}"),
            )
        })?;
    // PostgreSQL 18 assigns these six NOT NULL names a `1` suffix when immutable
    // V5 recreates study_points while its predecessor table still exists. Admit only
    // that exact historical naming artifact; V7 normalizes it in the same transaction.
    if study_policy && !job_identity {
        for (table, name, kind, definition) in &mut actual {
            for column in [
                "binding_hash",
                "job_id",
                "point_index",
                "state",
                "study_id",
                "updated_at",
            ] {
                let canonical = format!("study_points_{column}_not_null");
                if table == "study_points"
                    && kind == "n"
                    && *name == format!("{canonical}1")
                    && *definition == format!("NOT NULL {column}")
                {
                    *name = canonical;
                }
            }
        }
    }
    actual.sort();
    expected.sort();
    if actual != expected {
        return Err(refuse(
            MigrationRefusal::Drift,
            format!(
                "canonical constraint/domain definitions differ from the immutable declared transition; unexpected={:?}; missing={:?}",
                actual
                    .iter()
                    .filter(|row| !expected.contains(row))
                    .take(3)
                    .collect::<Vec<_>>(),
                expected
                    .iter()
                    .filter(|row| !actual.contains(row))
                    .take(3)
                    .collect::<Vec<_>>()
            ),
        ));
    }
    let domains=client.query("SELECT t.typname::text,format_type(t.typbasetype,t.typtypmod),t.typnotnull FROM pg_type t JOIN pg_namespace n ON n.oid=t.typnamespace WHERE n.nspname='pse_ops' AND t.typtype='d' ORDER BY t.typname", &[]).await.classify(target)?;
    let actual = domains
        .iter()
        .map(|row| {
            (
                row.get::<_, String>(0),
                row.get::<_, String>(1),
                row.get::<_, bool>(2),
            )
        })
        .collect::<Vec<_>>();
    let frozen = if inventory {
        include_str!("../migrations/plan25g-domain-signatures.json")
    } else if study_policy {
        include_str!("../migrations/plan25f-domain-signatures.json")
    } else if support {
        include_str!("../migrations/plan25e-domain-signatures.json")
    } else {
        include_str!("../migrations/plan25d-domain-signatures.json")
    };
    let expected: Vec<(String, String, bool)> = serde_json::from_str(frozen).map_err(|error| {
        refuse(
            MigrationRefusal::Drift,
            format!("invalid immutable domain declaration: {error}"),
        )
    })?;
    if actual != expected {
        return Err(refuse(
            MigrationRefusal::Drift,
            "identity domain definitions differ from the immutable declared transition",
        ));
    }
    let indexes=client.query("SELECT t.relname::text,c.relname::text,pg_get_indexdef(i.indexrelid),i.indisvalid,i.indisready FROM pg_index i JOIN pg_class c ON c.oid=i.indexrelid JOIN pg_class t ON t.oid=i.indrelid JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='pse_ops' AND t.relname <> ALL($1) ORDER BY t.relname,c.relname", &[&vec!["catalog_schema_history","operations_schema_history"]]).await.classify(target)?;
    let actual = indexes
        .iter()
        .map(|row| {
            (
                row.get::<_, String>(0),
                row.get::<_, String>(1),
                row.get::<_, String>(2),
                row.get::<_, bool>(3),
                row.get::<_, bool>(4),
            )
        })
        .collect::<Vec<_>>();
    let frozen = if inventory {
        include_str!("../migrations/plan25g-index-signatures.json")
    } else if study_policy {
        include_str!("../migrations/plan25f-index-signatures.json")
    } else if support {
        include_str!("../migrations/plan25e-index-signatures.json")
    } else {
        include_str!("../migrations/plan25d-index-signatures.json")
    };
    let expected: Vec<(String, String, String, bool, bool)> = serde_json::from_str(frozen)
        .map_err(|error| {
            refuse(
                MigrationRefusal::Drift,
                format!("invalid immutable index declaration: {error}"),
            )
        })?;
    if actual != expected {
        return Err(refuse(
            MigrationRefusal::Drift,
            "access path definitions differ from the immutable declared transition",
        ));
    }
    // The hand-authored physical declaration owns these access paths and defaults.
    let mut table = None;
    let physical = if study_policy {
        PHYSICAL_SQL
    } else {
        include_str!("../test-fixtures/plan25e/physical.sql")
    };
    for line in physical.lines() {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        if fields.starts_with(&["CREATE", "INDEX"]) {
            let name = fields[2];
            let valid=client.query_one("SELECT EXISTS(SELECT 1 FROM pg_index i JOIN pg_class c ON c.oid=i.indexrelid JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='pse_ops' AND c.relname=$1 AND i.indisvalid AND i.indisready)", &[&name]).await.classify(target)?.get::<_,bool>(0);
            if !valid {
                return Err(refuse(
                    MigrationRefusal::Drift,
                    format!("required access path {name} is absent or invalid"),
                ));
            }
        }
        if fields.starts_with(&["ALTER", "TABLE"]) {
            table = fields
                .get(2)
                .map(|name| name.trim_start_matches("pse_ops.").to_string());
        }
        if fields.starts_with(&["ALTER", "COLUMN"])
            && fields.get(3) == Some(&"SET")
            && fields.get(4) == Some(&"DEFAULT")
        {
            let table = table.as_deref().ok_or_else(|| {
                refuse(
                    MigrationRefusal::Drift,
                    "default declaration has no table owner",
                )
            })?;
            let column = fields[2];
            let expected = fields[5..].join(" ");
            let expected = expected.trim_end_matches([',', ';']);
            let actual=client.query_opt("SELECT pg_get_expr(d.adbin,d.adrelid) FROM pg_attrdef d JOIN pg_attribute a ON a.attrelid=d.adrelid AND a.attnum=d.adnum JOIN pg_class c ON c.oid=a.attrelid JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='pse_ops' AND c.relname=$1 AND a.attname=$2", &[&table,&column]).await.classify(target)?;
            if actual.map(|row| row.get::<_, String>(0)).as_deref() != Some(expected) {
                return Err(refuse(
                    MigrationRefusal::Drift,
                    format!("required default {table}.{column} differs"),
                ));
            }
        }
    }
    let enum_names=client.query("SELECT t.typname::text FROM pg_type t JOIN pg_namespace n ON n.oid=t.typnamespace WHERE n.nspname='pse_ops' AND t.typtype='e' ORDER BY t.typname", &[]).await.classify(target)?;
    let actual = enum_names
        .iter()
        .map(|row| row.get::<_, String>(0))
        .collect::<Vec<_>>();
    let enums = if operations_version >= 8 {
        crate::generated::layout::ENUMS
    } else if inventory {
        plan25g_layout::ENUMS
    } else if study_policy {
        plan25f_layout::ENUMS
    } else {
        plan25e_layout::ENUMS
    };
    let mut expected = enums
        .iter()
        .map(|(name, _)| name.to_string())
        .collect::<Vec<_>>();
    expected.sort();
    if actual != expected {
        return Err(refuse(
            MigrationRefusal::Drift,
            "owned enum domain inventory differs",
        ));
    }
    for (name, members) in enums {
        let qualified = format!("pse_ops.{name}");
        let rows = client.query("SELECT enumlabel::text FROM pg_enum WHERE enumtypid=$1::text::regtype ORDER BY enumsortorder", &[&qualified]).await.classify(target)?;
        let actual = rows
            .iter()
            .map(|r| r.get::<_, String>(0))
            .collect::<Vec<_>>();
        let expected = members
            .iter()
            .filter(|member| {
                !(!node_limit && *name == "native_termination" && **member == "node_limit"
                    || !shooting && *name == "attempt_kind" && **member == "shooting")
            })
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        if actual != expected {
            return Err(refuse(
                MigrationRefusal::Drift,
                format!("{name} differs from declared source/target domain"),
            ));
        }
    }
    Ok(())
}

/// Source records and committed prefixes are one admission fact; no historical row is rewritten.
fn admit_prefix(
    origin: &str,
    pending: bool,
    catalog: usize,
    operations: usize,
) -> Result<(), OperationsError> {
    let admitted = match (origin, pending) {
        (MIGRATION_SOURCE, false) => catalog == 0 && operations == 0,
        (MIGRATION_SOURCE, true) => {
            catalog <= 2
                && operations <= 8
                && (operations == 0 || catalog >= 1)
                && (catalog < 2 || operations >= 5)
                && (operations < 6 || catalog == 2)
        }
        (STUDY_MIGRATION_SOURCE, false) => catalog == 1 && operations == 4,
        (STUDY_MIGRATION_SOURCE, true) => {
            (1..=2).contains(&catalog)
                && (4..=8).contains(&operations)
                && (catalog < 2 || operations >= 5)
                && (operations < 6 || catalog == 2)
        }
        (FRESH_PLAN25E_ORIGIN, false) => catalog == 0 && operations == 0,
        (FRESH_PLAN25E_ORIGIN, true) => {
            catalog <= 1
                && operations <= 4
                && (catalog == 0 || operations >= 1)
                && (operations < 2 || catalog == 1)
        }
        _ => false,
    };
    if admitted {
        Ok(())
    } else {
        Err(refuse(
            MigrationRefusal::ChecksumConflict,
            "committed transition history disagrees with the exact recorded source",
        ))
    }
}
async fn verify_plan25e_support(
    client: &tokio_postgres::Client,
    target: &crate::error::Target,
) -> Result<bool, OperationsError> {
    let support_present = client
        .query_one(
            "SELECT to_regclass('pse_ops.schema_support_state') IS NOT NULL",
            &[],
        )
        .await
        .classify(target)?
        .get::<_, bool>(0);
    if !support_present {
        return Err(refuse(
            MigrationRefusal::NotReady,
            "Plan 25e support readiness is absent",
        ));
    }
    let rows = client.query("SELECT history,shared_version,target,ready,source FROM pse_ops.schema_support_state ORDER BY history", &[]).await.classify(target)?;
    let expected = [
        ("catalog", PLAN25E_CATALOG),
        ("operations", PLAN25E_OPERATIONS),
    ];
    if rows.len() != expected.len()
        || rows
            .iter()
            .zip(expected)
            .any(|(r, (history, fingerprint))| {
                r.get::<_, String>(0) != history
                    || r.get::<_, i32>(1) != 1
                    || r.get::<_, String>(2) != fingerprint
                    || !r.get::<_, bool>(3)
            })
    {
        return Err(refuse(
            MigrationRefusal::NotReady,
            "Plan 25e support must be complete before the follow-on transition",
        ));
    }
    let fresh = rows.iter().all(|r| r.get::<_, String>(4) == "fresh");
    if !fresh
        && !rows
            .iter()
            .all(|r| r.get::<_, String>(4) == MIGRATION_SOURCE)
    {
        return Err(refuse(
            MigrationRefusal::NotReady,
            "Plan 25e support provenance is not a declared source lineage",
        ));
    }
    Ok(fresh)
}

/// The shared support rows are part of every supported committed intermediate baseline.
async fn verify_intermediate_support(
    client: &tokio_postgres::Client,
    target: &crate::error::Target,
    fresh25e: bool,
    operation_version: i32,
    inventory: bool,
) -> Result<(), OperationsError> {
    let rows=client.query("SELECT history,shared_version,target,ready,source FROM pse_ops.schema_support_state ORDER BY history",&[]).await.classify(target)?;
    let operations_present = fresh25e || operation_version > 0;
    if rows.len() != if operations_present { 2 } else { 1 } {
        return Err(refuse(
            MigrationRefusal::NotReady,
            "shared support row inventory differs from committed prefix",
        ));
    }
    for row in rows {
        let history: String = row.try_get(0).classify(target)?;
        let shared: i32 = row.try_get(1).classify(target)?;
        let recorded: String = row.try_get(2).classify(target)?;
        let ready: bool = row.try_get(3).classify(target)?;
        let source: String = row.try_get(4).classify(target)?;
        let (expected, provenance, expected_ready) = match history.as_str() {
            "catalog" => (
                if inventory {
                    crate::generated::CATALOG_FINGERPRINT_HEX
                } else {
                    PLAN25E_CATALOG
                },
                if fresh25e {
                    if inventory { "fresh-v25f" } else { "fresh" }
                } else {
                    MIGRATION_SOURCE
                },
                !inventory || operation_version >= 6,
            ),
            "operations" => (
                if operation_version >= 8 {
                    crate::generated::OPERATIONS_FINGERPRINT_HEX
                } else if operation_version >= 7 {
                    PLAN25H_OPERATIONS
                } else if operation_version >= 5 {
                    PLAN25F_OPERATIONS
                } else {
                    PLAN25E_OPERATIONS
                },
                if fresh25e {
                    if operation_version >= 5 {
                        "fresh-v25e"
                    } else {
                        "fresh"
                    }
                } else if operation_version >= 5 {
                    STUDY_MIGRATION_SOURCE
                } else {
                    MIGRATION_SOURCE
                },
                operation_version >= 4 && (!inventory || operation_version >= 6),
            ),
            _ => {
                return Err(refuse(
                    MigrationRefusal::NotReady,
                    "unexpected support owner",
                ));
            }
        };
        if shared != crate::generated::SHARED_VERSION
            || recorded != expected
            || source != provenance
            || ready != expected_ready
        {
            return Err(refuse(
                MigrationRefusal::NotReady,
                "shared support target/provenance/readiness disagrees with committed immutable prefix",
            ));
        }
    }
    Ok(())
}

impl Store {
    /// Read-only exact transition inspection. A temporary shared session fence prevents
    /// concurrent migration while both histories and live support are inspected.
    /// # Errors
    /// Unknown source, checksum conflict, drift, active maintenance or driver failures.
    pub async fn migration_plan(&self) -> Result<MigrationPlan, OperationsError> {
        let session = self.schema_session().await?;
        let locked = session
            .client
            .query_one("SELECT pg_try_advisory_lock_shared($1)", &[&SCHEMA_LOCK])
            .await
            .classify(self.target())?
            .get::<_, bool>(0);
        if !locked {
            return Err(refuse(
                MigrationRefusal::ActiveGeneration,
                "maintenance owns the namespace",
            ));
        }
        self.inspect_migration(&session.client).await
    }

    async fn inspect_migration(
        &self,
        client: &tokio_postgres::Client,
    ) -> Result<MigrationPlan, OperationsError> {
        let comment = statements::schema_comment()
            .bind(client)
            .opt()
            .await
            .classify(self.target())?;
        let source = comment.as_ref().and_then(|c| recorded(c.as_deref()));
        if source == Some(SCHEMA_FINGERPRINT_HEX) {
            verify_ready(client, self.target()).await?;
            return current_plan(client, self.target()).await;
        }
        let pending_source = comment
            .as_ref()
            .and_then(|c| c.as_deref())
            .and_then(|c| c.strip_prefix(PENDING_PREFIX))
            .and_then(|c| c.split_once(' '))
            .filter(|(_, target)| *target == SCHEMA_FINGERPRINT_HEX)
            .map(|(source, _)| source);
        if source == Some(NATIVE_BACKENDS_MIGRATION_SOURCE)
            || pending_source == Some(NATIVE_BACKENDS_MIGRATION_SOURCE)
        {
            return self
                .inspect_native_backends_transition(client, pending_source.is_some())
                .await;
        }
        if source == Some(JOB_IDENTITY_MIGRATION_SOURCE)
            || pending_source == Some(JOB_IDENTITY_MIGRATION_SOURCE)
        {
            return self
                .inspect_job_identity_transition(client, pending_source.is_some())
                .await;
        }
        if source == Some(INVENTORY_MIGRATION_SOURCE)
            || pending_source == Some(INVENTORY_MIGRATION_SOURCE)
        {
            return self
                .inspect_inventory_transition(client, pending_source.is_some())
                .await;
        }
        let mut origin = source
            .filter(|source| matches!(*source, MIGRATION_SOURCE | STUDY_MIGRATION_SOURCE))
            .or_else(|| {
                pending_source.filter(|source| {
                    matches!(
                        *source,
                        MIGRATION_SOURCE | STUDY_MIGRATION_SOURCE | FRESH_PLAN25E_ORIGIN
                    )
                })
            })
            .ok_or_else(|| {
                refuse(
                    MigrationRefusal::UnknownSource,
                    "no declared transition from the exact recorded source",
                )
            })?;
        if source == Some(STUDY_MIGRATION_SOURCE)
            && verify_plan25e_support(client, self.target()).await?
        {
            origin = FRESH_PLAN25E_ORIGIN;
        }
        let is_pending = pending_source.is_some();
        if !include_str!("../migrations/V8__native_backends.sql").contains(SCHEMA_FINGERPRINT_HEX)
            || !include_str!("../migrations/V2__catalog_retirement_inventory.sql")
                .contains(crate::generated::CATALOG_FINGERPRINT_HEX)
        {
            return Err(refuse(
                MigrationRefusal::Drift,
                "build target has no immutable declared transition",
            ));
        }
        let (mut catalog, mut operations) = transitions()?;
        let fresh25e = origin == FRESH_PLAN25E_ORIGIN;
        if fresh25e && !is_pending {
            let history_present = client
                .query_one(
                    "SELECT to_regclass($1) IS NOT NULL OR to_regclass($2) IS NOT NULL",
                    &[&CATALOG_HISTORY, &OPERATIONS_HISTORY],
                )
                .await
                .classify(self.target())?
                .get::<_, bool>(0);
            if history_present {
                return Err(refuse(
                    MigrationRefusal::ChecksumConflict,
                    "fresh Plan 25e source must have absent migration histories",
                ));
            }
        }
        if fresh25e {
            catalog.drain(..1);
            operations.drain(..4);
        }
        let catalog_prefix =
            history_prefix(client, &catalog, CATALOG_HISTORY, self.target()).await?;
        let operations_prefix =
            history_prefix(client, &operations, OPERATIONS_HISTORY, self.target()).await?;
        admit_prefix(origin, is_pending, catalog_prefix, operations_prefix)?;
        if fresh25e && operations_prefix == 0 {
            if !verify_plan25e_support(client, self.target()).await? {
                return Err(refuse(
                    MigrationRefusal::NotReady,
                    "fresh Plan 25e transition requires fresh support provenance",
                ));
            }
        } else if origin == STUDY_MIGRATION_SOURCE
            && operations_prefix == 4
            && verify_plan25e_support(client, self.target()).await?
        {
            return Err(refuse(
                MigrationRefusal::NotReady,
                "upgraded Plan 25e transition cannot claim fresh support provenance",
            ));
        }
        let inventory = if fresh25e {
            catalog_prefix >= 1
        } else {
            catalog_prefix >= 2
        };
        let op_version = operations
            .get(operations_prefix.saturating_sub(1))
            .filter(|_| operations_prefix > 0)
            .map_or(if fresh25e { 4 } else { 0 }, refinery::Migration::version);
        if fresh25e || catalog_prefix > 0 {
            verify_intermediate_support(client, self.target(), fresh25e, op_version, inventory)
                .await?;
        }
        verify_progress_layout(
            client,
            self.target(),
            LayoutCheckpoint {
                support: fresh25e || catalog_prefix > 0,
                inventory,
                operations_version: op_version,
            },
        )
        .await?;
        Ok(MigrationPlan {
            source: origin.to_owned(),
            target: SCHEMA_FINGERPRINT_HEX.to_owned(),
            pending: is_pending,
            catalog: history_plan(
                CATALOG_HISTORY,
                crate::generated::CATALOG_FINGERPRINT_HEX,
                catalog_prefix,
                &catalog,
            ),
            operations: history_plan(
                OPERATIONS_HISTORY,
                crate::generated::OPERATIONS_FINGERPRINT_HEX,
                operations_prefix,
                &operations,
            ),
            shared_version: crate::generated::SHARED_VERSION,
            ready: false,
        })
    }

    /// Append native profiles to the exact V7 predecessor, retaining its owned histories.
    async fn inspect_native_backends_transition(
        &self,
        client: &tokio_postgres::Client,
        pending: bool,
    ) -> Result<MigrationPlan, OperationsError> {
        let rows = client.query("SELECT history,shared_version,target,ready,source FROM pse_ops.schema_support_state ORDER BY history", &[]).await.classify(self.target())?;
        if rows.len() != 2 {
            return Err(refuse(
                MigrationRefusal::NotReady,
                "V7 support owners are missing",
            ));
        }
        let (mut catalog, mut operations) = transitions()?;
        for (row, history) in rows.iter().zip(["catalog", "operations"]) {
            let owner: String = row.try_get(0).classify(self.target())?;
            let shared: i32 = row.try_get(1).classify(self.target())?;
            let target: String = row.try_get(2).classify(self.target())?;
            let ready: bool = row.try_get(3).classify(self.target())?;
            let origin: String = row.try_get(4).classify(self.target())?;
            if owner != history
                || shared != crate::generated::SHARED_VERSION
                || !ready
                || target
                    != if history == "catalog" {
                        crate::generated::CATALOG_FINGERPRINT_HEX
                    } else {
                        PLAN25H_OPERATIONS
                    }
            {
                return Err(refuse(
                    MigrationRefusal::NotReady,
                    "V7 owned support differs from the recorded predecessor",
                ));
            }
            if history == "catalog" {
                match origin.as_str() {
                    "fresh" => catalog.clear(),
                    "fresh-v25f" => {
                        catalog.drain(..1);
                    }
                    MIGRATION_SOURCE => {}
                    _ => {
                        return Err(refuse(
                            MigrationRefusal::UnknownSource,
                            "unsupported V7 catalog lineage",
                        ));
                    }
                }
            } else {
                let retained_from = match origin.as_str() {
                    "fresh" => 7,
                    "fresh-v25g" => 6,
                    "fresh-v25f" => 5,
                    "fresh-v25e" => 4,
                    STUDY_MIGRATION_SOURCE => 0,
                    _ => {
                        return Err(refuse(
                            MigrationRefusal::UnknownSource,
                            "unsupported V7 operations lineage",
                        ));
                    }
                };
                operations.drain(..retained_from);
            }
        }
        let cp = history_prefix(client, &catalog, CATALOG_HISTORY, self.target()).await?;
        let op = history_prefix(client, &operations, OPERATIONS_HISTORY, self.target()).await?;
        if cp != catalog.len() || op != operations.len() - 1 {
            return Err(refuse(
                MigrationRefusal::ChecksumConflict,
                "V7 histories differ from source provenance",
            ));
        }
        verify_progress_layout(
            client,
            self.target(),
            LayoutCheckpoint {
                support: true,
                inventory: true,
                operations_version: 7,
            },
        )
        .await?;
        Ok(MigrationPlan {
            source: NATIVE_BACKENDS_MIGRATION_SOURCE.into(),
            target: SCHEMA_FINGERPRINT_HEX.into(),
            pending,
            catalog: history_plan(
                CATALOG_HISTORY,
                crate::generated::CATALOG_FINGERPRINT_HEX,
                cp,
                &catalog,
            ),
            operations: history_plan(
                OPERATIONS_HISTORY,
                crate::generated::OPERATIONS_FINGERPRINT_HEX,
                op,
                &operations,
            ),
            shared_version: crate::generated::SHARED_VERSION,
            ready: false,
        })
    }

    /// Inspect the V6 predecessor without retagging its recorded digest provenance.
    async fn inspect_job_identity_transition(
        &self,
        client: &tokio_postgres::Client,
        pending: bool,
    ) -> Result<MigrationPlan, OperationsError> {
        let rows = client.query("SELECT history,shared_version,target,ready,source FROM pse_ops.schema_support_state ORDER BY history", &[]).await.classify(self.target())?;
        if rows.len() != 2 {
            return Err(refuse(
                MigrationRefusal::NotReady,
                "V6 support owners are missing",
            ));
        }
        let (mut catalog, mut operations) = transitions()?;
        for (row, history) in rows.iter().zip(["catalog", "operations"]) {
            let owner: String = row.try_get(0).classify(self.target())?;
            let shared: i32 = row.try_get(1).classify(self.target())?;
            let target: String = row.try_get(2).classify(self.target())?;
            let ready: bool = row.try_get(3).classify(self.target())?;
            let origin: String = row.try_get(4).classify(self.target())?;
            if owner != history
                || shared != crate::generated::SHARED_VERSION
                || !ready
                || target
                    != if history == "catalog" {
                        crate::generated::CATALOG_FINGERPRINT_HEX
                    } else {
                        PLAN25F_OPERATIONS
                    }
            {
                return Err(refuse(
                    MigrationRefusal::NotReady,
                    "V6 owned support differs from recorded predecessor",
                ));
            }
            if history == "catalog" {
                match origin.as_str() {
                    "fresh" => catalog.clear(),
                    "fresh-v25f" => {
                        catalog.drain(..1);
                    }
                    MIGRATION_SOURCE => {}
                    _ => {
                        return Err(refuse(
                            MigrationRefusal::UnknownSource,
                            "unsupported V6 catalog lineage",
                        ));
                    }
                }
            } else {
                match origin.as_str() {
                    "fresh" => {
                        operations.drain(..6);
                    }
                    "fresh-v25f" => {
                        operations.drain(..5);
                    }
                    "fresh-v25e" => {
                        operations.drain(..4);
                    }
                    STUDY_MIGRATION_SOURCE => {}
                    _ => {
                        return Err(refuse(
                            MigrationRefusal::UnknownSource,
                            "unsupported V6 operations lineage",
                        ));
                    }
                }
            }
        }
        let cp = history_prefix(client, &catalog, CATALOG_HISTORY, self.target()).await?;
        let op = history_prefix(client, &operations, OPERATIONS_HISTORY, self.target()).await?;
        if cp != catalog.len()
            || op < operations.iter().take_while(|m| m.version() < 7).count()
            || (!pending && op != operations.iter().take_while(|m| m.version() < 7).count())
        {
            return Err(refuse(
                MigrationRefusal::ChecksumConflict,
                "V6 committed histories differ from source provenance",
            ));
        }
        verify_progress_layout(
            client,
            self.target(),
            LayoutCheckpoint {
                support: true,
                inventory: true,
                operations_version: 6,
            },
        )
        .await?;
        Ok(MigrationPlan {
            source: JOB_IDENTITY_MIGRATION_SOURCE.into(),
            target: SCHEMA_FINGERPRINT_HEX.into(),
            pending,
            catalog: history_plan(
                CATALOG_HISTORY,
                crate::generated::CATALOG_FINGERPRINT_HEX,
                cp,
                &catalog,
            ),
            operations: history_plan(
                OPERATIONS_HISTORY,
                crate::generated::OPERATIONS_FINGERPRINT_HEX,
                op,
                &operations,
            ),
            shared_version: crate::generated::SHARED_VERSION,
            ready: false,
        })
    }

    /// Exact Plan 25f predecessor, including fresh and previously upgraded lineage.
    async fn inspect_inventory_transition(
        &self,
        client: &tokio_postgres::Client,
        pending: bool,
    ) -> Result<MigrationPlan, OperationsError> {
        let rows=client.query("SELECT history,shared_version,target,ready,source FROM pse_ops.schema_support_state ORDER BY history", &[]).await.classify(self.target())?;
        if rows.len() != 2 {
            return Err(refuse(
                MigrationRefusal::NotReady,
                "exact Plan 25f support rows are missing",
            ));
        }
        let (mut catalog, mut operations) = transitions()?;
        for (row, history) in rows.iter().zip(["catalog", "operations"]) {
            let recorded_history: String = row.try_get(0).classify(self.target())?;
            let shared: i32 = row.try_get(1).classify(self.target())?;
            let ready: bool = row.try_get(3).classify(self.target())?;
            let source: String = row.try_get(4).classify(self.target())?;
            if recorded_history != history
                || shared != crate::generated::SHARED_VERSION
                || (!pending && !ready)
            {
                return Err(refuse(
                    MigrationRefusal::NotReady,
                    "Plan 25f shared support or readiness differs",
                ));
            }
            if history == "catalog" {
                match source.as_str() {
                    "fresh" | "fresh-v25f" => {
                        catalog.drain(..1);
                    }
                    MIGRATION_SOURCE => {}
                    _ => {
                        return Err(refuse(
                            MigrationRefusal::UnknownSource,
                            "unsupported catalog provenance",
                        ));
                    }
                }
            } else {
                match source.as_str() {
                    "fresh" | "fresh-v25f" => {
                        operations.drain(..5);
                    }
                    "fresh-v25e" => {
                        operations.drain(..4);
                    }
                    STUDY_MIGRATION_SOURCE => {}
                    _ => {
                        return Err(refuse(
                            MigrationRefusal::UnknownSource,
                            "unsupported operations provenance",
                        ));
                    }
                }
            }
        }
        let cp = history_prefix(client, &catalog, CATALOG_HISTORY, self.target()).await?;
        let op = history_prefix(client, &operations, OPERATIONS_HISTORY, self.target()).await?;
        let initial_catalog = catalog.len() - 1;
        let initial_operations = operations.iter().take_while(|m| m.version() < 6).count();
        if cp < initial_catalog
            || op < initial_operations
            || (!pending && (cp != initial_catalog || op != initial_operations))
            || (op == operations.len() && cp != catalog.len())
        {
            return Err(refuse(
                MigrationRefusal::ChecksumConflict,
                "Plan 25f source and committed prefixes disagree",
            ));
        }
        let inventory = cp == catalog.len();
        for (row, expected) in rows.iter().zip([
            if inventory {
                crate::generated::CATALOG_FINGERPRINT_HEX
            } else {
                PLAN25E_CATALOG
            },
            PLAN25F_OPERATIONS,
        ]) {
            let actual: String = row.try_get(2).classify(self.target())?;
            if actual != expected {
                return Err(refuse(
                    MigrationRefusal::NotReady,
                    "Plan 25f consumed support target differs",
                ));
            }
        }
        verify_progress_layout(
            client,
            self.target(),
            LayoutCheckpoint {
                support: true,
                inventory,
                operations_version: operations
                    .get(op.saturating_sub(1))
                    .filter(|_| op > 0)
                    .map_or(5, refinery::Migration::version),
            },
        )
        .await?;
        Ok(MigrationPlan {
            source: INVENTORY_MIGRATION_SOURCE.into(),
            target: SCHEMA_FINGERPRINT_HEX.into(),
            pending,
            catalog: history_plan(
                CATALOG_HISTORY,
                crate::generated::CATALOG_FINGERPRINT_HEX,
                cp,
                &catalog,
            ),
            operations: history_plan(
                OPERATIONS_HISTORY,
                crate::generated::OPERATIONS_FINGERPRINT_HEX,
                op,
                &operations,
            ),
            shared_version: crate::generated::SHARED_VERSION,
            ready: false,
        })
    }

    /// Explicitly upgrade the known predecessor after draining workers and closing generations.
    /// The namespace session lock survives every migration transaction and both histories.
    /// Failed steps retain NotReady and matching committed history; retries resume without reset.
    ///
    /// # Errors
    /// Unknown source, history conflict, active generation, drift or classified PostgreSQL failure.
    pub async fn migrate(
        &self,
        expected: &MigrationPlan,
    ) -> Result<MigrationReport, OperationsError> {
        if self.has_schema_lease().await {
            return Err(refuse(
                MigrationRefusal::ActiveGeneration,
                "close this opened generation and connect an administration handle",
            ));
        }
        let mut session = self.schema_session().await?;
        let locked = session
            .client
            .query_one("SELECT pg_try_advisory_lock($1)", &[&SCHEMA_LOCK])
            .await
            .classify(self.target())?
            .get::<_, bool>(0);
        if !locked {
            return Err(refuse(
                MigrationRefusal::ActiveGeneration,
                "another opened generation or maintenance session owns the namespace",
            ));
        }
        let actual = self.inspect_migration(&session.client).await?;
        actual.require_expected(expected)?;
        if actual.ready {
            return Ok(MigrationReport {
                applied: Vec::new(),
                final_ready: actual,
            });
        }
        let origin = actual.source.as_str();
        let is_pending = actual.pending;
        let pending = format!("{PENDING_PREFIX}{origin} {SCHEMA_FINGERPRINT_HEX}");
        let (declared_catalog, declared_operations) = transitions()?;
        let catalog = declared_catalog
            .into_iter()
            .filter(|m| {
                actual
                    .catalog
                    .steps
                    .iter()
                    .any(|s| s.version == m.version())
            })
            .collect::<Vec<_>>();
        let operations = declared_operations
            .into_iter()
            .filter(|m| {
                actual
                    .operations
                    .steps
                    .iter()
                    .any(|s| s.version == m.version())
            })
            .collect::<Vec<_>>();
        // Keep the shared namespace session lock through every history and transaction.
        if !is_pending {
            session
                .client
                .batch_execute(&format!("COMMENT ON SCHEMA pse_ops IS '{pending}'"))
                .await
                .classify(self.target())?;
        }
        let old_catalog = catalog
            .iter()
            .filter(|m| m.version() == 1)
            .cloned()
            .collect::<Vec<_>>();
        if actual
            .catalog
            .steps
            .iter()
            .skip(actual.catalog.committed)
            .any(|s| s.version == 1)
        {
            run_history(&mut session.client, &old_catalog, CATALOG_HISTORY).await?;
        }
        let through4 = operations
            .iter()
            .filter(|m| m.version() <= 4)
            .cloned()
            .collect::<Vec<_>>();
        if !through4.is_empty()
            && actual
                .operations
                .steps
                .iter()
                .skip(actual.operations.committed)
                .any(|s| s.version <= 4)
        {
            run_history(&mut session.client, &through4, OPERATIONS_HISTORY).await?;
            verify_progress_layout(
                &session.client,
                self.target(),
                LayoutCheckpoint {
                    support: true,
                    inventory: false,
                    operations_version: 4,
                },
            )
            .await?;
        }
        let through5 = operations
            .iter()
            .filter(|m| m.version() <= 5)
            .cloned()
            .collect::<Vec<_>>();
        if !through5.is_empty()
            && actual
                .operations
                .steps
                .iter()
                .skip(actual.operations.committed)
                .any(|s| s.version <= 5)
        {
            run_history(&mut session.client, &through5, OPERATIONS_HISTORY).await?;
            verify_progress_layout(
                &session.client,
                self.target(),
                LayoutCheckpoint {
                    support: true,
                    inventory: false,
                    operations_version: 5,
                },
            )
            .await?;
            // V5's immutable ready record is the exact supported 25f checkpoint. Restore the
            // current in-progress record before the appended catalog step.
            session
                .client
                .batch_execute(&format!("COMMENT ON SCHEMA pse_ops IS '{pending}'"))
                .await
                .classify(self.target())?;
        }
        run_history(&mut session.client, &catalog, CATALOG_HISTORY).await?;
        run_history(&mut session.client, &operations, OPERATIONS_HISTORY).await?;
        verify_ready(&session.client, self.target()).await?;
        self.forget_statements();
        let applied = actual.pending_steps();
        let final_ready = self.inspect_migration(&session.client).await?;
        Ok(MigrationReport {
            applied,
            final_ready,
        })
    }
}

#[cfg(test)]
mod schema_unit {
    use super::*;

    #[test]
    fn history_preflight_refuses_malformed_timestamp_without_panicking() {
        let (catalog, _) = transitions().unwrap();
        let step = &catalog[0];
        for timestamp in ["", "2026-01-01", "not-a-timestamp", "2026-99-99T99:00:00Z"] {
            assert!(matches!(
                validate_history_values(
                    step.version(),
                    step.name(),
                    &step.checksum().to_string(),
                    timestamp,
                    step
                ),
                Err(OperationsError::MigrationRefused {
                    reason: MigrationRefusal::ChecksumConflict,
                    ..
                })
            ));
        }
        assert!(
            validate_history_values(
                step.version(),
                step.name(),
                &step.checksum().to_string(),
                "2026-01-01T00:00:00Z",
                step
            )
            .is_ok()
        );
    }

    #[test]
    fn migration_plan_compares_all_owned_support_and_reports_only_pending_steps() {
        let (catalog, operations) = transitions().unwrap();
        let plan = MigrationPlan {
            source: MIGRATION_SOURCE.into(),
            target: SCHEMA_FINGERPRINT_HEX.into(),
            pending: false,
            catalog: history_plan(
                CATALOG_HISTORY,
                crate::generated::CATALOG_FINGERPRINT_HEX,
                1,
                &catalog,
            ),
            operations: history_plan(
                OPERATIONS_HISTORY,
                crate::generated::OPERATIONS_FINGERPRINT_HEX,
                3,
                &operations,
            ),
            shared_version: crate::generated::SHARED_VERSION,
            ready: false,
        };
        assert_eq!(
            plan.pending_steps()
                .iter()
                .map(|s| s.version)
                .collect::<Vec<_>>(),
            [4, 5, 2, 6, 7, 8]
        );
        for mutated in 0..5 {
            let mut changed = plan.clone();
            match mutated {
                0 => changed.source.push('x'),
                1 => changed.target.push('x'),
                2 => changed.catalog.steps[0].checksum += 1,
                3 => changed.operations.committed += 1,
                _ => changed.shared_version += 1,
            }
            assert!(matches!(
                changed.require_expected(&plan),
                Err(OperationsError::MigrationRefused {
                    reason: MigrationRefusal::PlanChanged,
                    ..
                })
            ));
        }
        assert!(plan.require_expected(&plan).is_ok());
        let mut ready = plan;
        ready.ready = true;
        assert!(ready.pending_steps().is_empty());
    }

    #[test]
    fn the_record_statement_carries_the_expected_fingerprint() {
        assert_eq!(
            RECORD_SQL,
            format!("COMMENT ON SCHEMA pse_ops IS '{RECORD_PREFIX}{SCHEMA_FINGERPRINT_HEX}'")
        );
        assert_eq!(
            recorded(Some(&format!("{RECORD_PREFIX}{SCHEMA_FINGERPRINT_HEX}"))),
            Some(SCHEMA_FINGERPRINT_HEX)
        );
        assert_eq!(recorded(Some("hand-made schema")), None);
        assert_eq!(recorded(None), None);
        assert_eq!(
            crate::generated::SCHEMA_FINGERPRINT.to_hex(),
            SCHEMA_FINGERPRINT_HEX
        );
    }
}

#[cfg(test)]
#[path = "migration_tests.rs"]
mod migration_tests;
