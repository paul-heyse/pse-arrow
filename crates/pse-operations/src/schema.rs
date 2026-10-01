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
    /// There is no `pse_ops` schema; [`Store::open`] creates it.
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

/// What [`Store::open`] did.
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

    /// Create the schema on a store that has none, or confirm that the store's schema is
    /// this build's. Concurrent callers serialize on a transaction-scoped advisory lock;
    /// creation is one transaction, so a failed creation leaves no schema behind.
    ///
    /// # Errors
    ///
    /// [`OperationsError::SchemaMismatch`] when the store records another fingerprint
    /// (or none); classified driver failures, including a DDL failure.
    pub async fn open(&self) -> Result<Opened, OperationsError> {
        let began = std::time::Instant::now();
        loop {
            // Existing read-only admission takes a shared generation lease, never a DDL lock.
            let client = self.admin_client().await?;
            let comment = statements::schema_comment()
                .bind(&client)
                .opt()
                .await
                .classify(self.target())?;
            drop(client);
            match comment {
                Some(comment) if recorded(comment.as_deref()) == Some(SCHEMA_FINGERPRINT_HEX) => {
                    self.retain_schema_lease().await?;
                    return Ok(Opened::Current);
                }
                Some(comment)
                    if comment
                        .as_deref()
                        .is_some_and(|value| value.starts_with(PENDING_PREFIX)) =>
                {
                    return Err(refuse(
                        MigrationRefusal::NotReady,
                        "a committed schema transition is incomplete; explicitly resume it",
                    ));
                }
                Some(comment) => {
                    return Err(OperationsError::SchemaMismatch {
                        target: self.target().clone(),
                        recorded: recorded(comment.as_deref()).map(str::to_owned),
                        expected: SCHEMA_FINGERPRINT_HEX,
                    });
                }
                None => match self.apply(false).await {
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
                },
            }
        }
    }

    /// Drop the store's schema with everything in it, then create it afresh. Destructive
    /// by design: the store's contents are regenerable (ADR-0114 Outcome 23), and the
    /// `just db-reset` recipe asks before calling this.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn reset(&self) -> Result<Opened, OperationsError> {
        if self.has_schema_lease().await {
            return Err(refuse(
                MigrationRefusal::ActiveGeneration,
                "close opened generations before destructive maintenance",
            ));
        }
        self.apply(true).await
    }

    async fn apply(&self, reset: bool) -> Result<Opened, OperationsError> {
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
        if reset {
            // Every object of the store goes with its schema; cached statements naming
            // them go too.
            tx.batch_execute("DROP SCHEMA IF EXISTS pse_ops CASCADE")
                .await
                .classify(target)?;
            self.forget_statements();
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

/// Typed transition refusals; no reason string is interpreted as policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MigrationRefusal {
    /// Recorded source is not an explicitly supported predecessor.
    UnknownSource,
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
        vec![migration(
            "V1__catalog_identity",
            include_str!("../migrations/V1__catalog_identity.sql"),
        )?],
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
    if rows.iter().any(|r| r.get::<_, String>(0) == "catalog") {
        let source = client
            .query_one(
                "SELECT source FROM pse_ops.schema_support_state WHERE history='catalog'",
                &[],
            )
            .await
            .classify(target)?
            .get::<_, String>(0);
        if source != "fresh" {
            let (catalog, operations) = transitions()?;
            verify_history(client, &catalog, CATALOG_HISTORY, target).await?;
            verify_history(client, &operations, OPERATIONS_HISTORY, target).await?;
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
    let rows = client
        .query(
            &format!("SELECT version,name,checksum FROM {table} ORDER BY version"),
            &[],
        )
        .await
        .classify(target)?;
    if rows.len() != expected.len()
        || rows.iter().zip(expected).any(|(row, migration)| {
            row.get::<_, i32>(0) != migration.version()
                || row.get::<_, String>(1) != migration.name()
                || row.get::<_, String>(2) != migration.checksum().to_string()
        })
    {
        return Err(refuse(
            MigrationRefusal::ChecksumConflict,
            "committed migration history differs from immutable declarations",
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
    let present = client
        .query_one("SELECT to_regclass($1) IS NOT NULL", &[&table])
        .await
        .classify(target)?
        .get::<_, bool>(0);
    if !present {
        return Ok(0);
    }
    let rows = client
        .query(
            &format!("SELECT version,name,checksum FROM {table} ORDER BY version"),
            &[],
        )
        .await
        .classify(target)?;
    if rows.len() > expected.len()
        || rows.iter().zip(expected).any(|(row, migration)| {
            row.get::<_, i32>(0) != migration.version()
                || row.get::<_, String>(1) != migration.name()
                || row.get::<_, String>(2) != migration.checksum().to_string()
        })
    {
        return Err(refuse(
            MigrationRefusal::ChecksumConflict,
            "committed history is not an exact declared prefix",
        ));
    }
    Ok(rows.len())
}
async fn verify_layout(
    client: &tokio_postgres::Client,
    target: &crate::error::Target,
    legacy: bool,
) -> Result<(), OperationsError> {
    verify_progress_layout(client, target, !legacy, !legacy, !legacy).await
}
async fn verify_progress_layout(
    client: &tokio_postgres::Client,
    target: &crate::error::Target,
    support: bool,
    node_limit: bool,
    shooting: bool,
) -> Result<(), OperationsError> {
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
    let expected = crate::generated::layout::COLUMNS
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
    let mut expected = crate::generated::layout::CONSTRAINTS
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
    let actual = constraints
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
    let frozen = if support {
        include_str!("../migrations/plan25e-constraint-signatures.json")
    } else {
        include_str!("../migrations/plan25d-constraint-signatures.json")
    };
    let expected: Vec<(String, String, String, String)> =
        serde_json::from_str(frozen).map_err(|error| {
            refuse(
                MigrationRefusal::Drift,
                format!("invalid immutable constraint declaration: {error}"),
            )
        })?;
    if actual != expected {
        return Err(refuse(
            MigrationRefusal::Drift,
            "canonical constraint/domain definitions differ from the immutable declared transition",
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
    let frozen = if support {
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
    let frozen = if support {
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
    for line in PHYSICAL_SQL.lines() {
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
    let mut expected = crate::generated::layout::ENUMS
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
    for (name, members) in crate::generated::layout::ENUMS {
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

impl Store {
    /// Explicitly upgrade the known predecessor after draining workers and closing generations.
    /// The namespace session lock survives every migration transaction and both histories.
    /// Failed steps retain NotReady and matching committed history; retries resume without reset.
    ///
    /// # Errors
    /// Unknown source, history conflict, active generation, drift or classified PostgreSQL failure.
    pub async fn migrate(&self) -> Result<Opened, OperationsError> {
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
        let comment = statements::schema_comment()
            .bind(&session.client)
            .opt()
            .await
            .classify(self.target())?;
        let source = comment.as_ref().and_then(|c| recorded(c.as_deref()));
        if source == Some(SCHEMA_FINGERPRINT_HEX) {
            verify_ready(&session.client, self.target()).await?;
            return Ok(Opened::Current);
        }
        let pending = format!(
            "{PENDING_PREFIX}{MIGRATION_SOURCE} {}",
            SCHEMA_FINGERPRINT_HEX
        );
        let is_pending = comment.as_ref().and_then(|c| c.as_deref()) == Some(pending.as_str());
        if source != Some(MIGRATION_SOURCE) && !is_pending {
            return Err(refuse(
                MigrationRefusal::UnknownSource,
                "no declared transition from the exact recorded source",
            ));
        }
        // Target declarations are frozen in SQL transitions; a later schema needs another version.
        if !include_str!("../migrations/V4__operations_ready.sql").contains(SCHEMA_FINGERPRINT_HEX)
            || !include_str!("../migrations/V1__catalog_identity.sql")
                .contains(crate::generated::CATALOG_FINGERPRINT_HEX)
            || !include_str!("../migrations/V4__operations_ready.sql")
                .contains(crate::generated::OPERATIONS_FINGERPRINT_HEX)
        {
            return Err(refuse(
                MigrationRefusal::Drift,
                "build target has no immutable declared transition",
            ));
        }
        let (catalog, operations) = transitions()?;
        let catalog_prefix =
            history_prefix(&session.client, &catalog, CATALOG_HISTORY, self.target()).await?;
        let operations_prefix = history_prefix(
            &session.client,
            &operations,
            OPERATIONS_HISTORY,
            self.target(),
        )
        .await?;
        if operations_prefix > 0 && catalog_prefix == 0 {
            return Err(refuse(
                MigrationRefusal::ChecksumConflict,
                "operations history lacks its committed catalog support prerequisite",
            ));
        }
        if !is_pending && (catalog_prefix > 0 || operations_prefix > 0) {
            return Err(refuse(
                MigrationRefusal::ChecksumConflict,
                "committed transition history disagrees with the recorded source",
            ));
        }
        verify_progress_layout(
            &session.client,
            self.target(),
            catalog_prefix > 0,
            operations_prefix >= 2,
            operations_prefix >= 3,
        )
        .await?;
        // Readiness changes only after both histories and their exact live intermediate layout agree.
        if !is_pending {
            session
                .client
                .batch_execute(&format!("COMMENT ON SCHEMA pse_ops IS '{pending}'"))
                .await
                .classify(self.target())?;
        }
        run_history(&mut session.client, &catalog, CATALOG_HISTORY).await?;
        let mut runner = refinery::Runner::new(&operations);
        runner.set_migration_table_name(OPERATIONS_HISTORY);
        runner = runner.set_target(refinery::Target::Version(3));
        runner
            .run_async(&mut session.client)
            .await
            .map_err(migration_error)?;
        verify_layout(&session.client, self.target(), false).await?;
        run_history(&mut session.client, &operations, OPERATIONS_HISTORY).await?;
        verify_ready(&session.client, self.target()).await?;
        self.forget_statements();
        Ok(Opened::Current)
    }
}

#[cfg(test)]
mod schema_unit {
    use super::*;

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
