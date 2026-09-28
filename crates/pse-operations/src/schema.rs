// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The store's schema: created from the registry-generated DDL, or refused
//! (ADR-0114 Outcome 23).
//!
//! The `pse_ops` schema is created in one transaction from the generated `schema.sql` and
//! the hand-written `physical.sql`, and its fingerprint is recorded as the schema's
//! comment. A store whose recorded fingerprint differs from this build's is refused with
//! [`OperationsError::SchemaMismatch`]; it is never migrated and never reset implicitly.
//! Its contents are regenerable, so the remedy is `just db-reset`.

use pse_operations_queries::queries::store as statements;

use crate::error::{Classify, OperationsError};
use crate::generated::{RECORD_SQL, SCHEMA_FINGERPRINT_HEX, SCHEMA_SQL};
use crate::store::Store;

/// The hand-written access paths, defaults and grants applied after the generated DDL.
pub const PHYSICAL_SQL: &str = include_str!("../physical.sql");

/// The recorded comment's prefix: the fingerprint's frame.
const RECORD_PREFIX: &str = "pse.ops.schema.v1 ";

/// Serializes schema creation and reset across processes (`pse_ops` in ASCII).
const SCHEMA_LOCK: i64 = 0x7073_655f_6f70_7300;

/// What a store's schema is, compared with this build.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SchemaStatus {
    /// There is no `pse_ops` schema; [`Store::open`] creates it.
    Absent,
    /// The schema records this build's fingerprint.
    Current,
    /// The schema records another fingerprint, or none (a store this build did not
    /// create); `just db-reset` recreates it.
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
        let client = self.client().await?;
        let comment = statements::schema_comment()
            .bind(&client)
            .opt()
            .await
            .classify(self.target())?;
        Ok(match comment {
            None => SchemaStatus::Absent,
            Some(comment) => match recorded(comment.as_deref()) {
                Some(fingerprint) if fingerprint == SCHEMA_FINGERPRINT_HEX => {
                    SchemaStatus::Current
                }
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
        self.apply(false).await
    }

    /// Drop the store's schema with everything in it, then create it afresh. Destructive
    /// by design: the store's contents are regenerable (ADR-0114 Outcome 23), and the
    /// `just db-reset` recipe asks before calling this.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn reset(&self) -> Result<Opened, OperationsError> {
        self.apply(true).await
    }

    async fn apply(&self, reset: bool) -> Result<Opened, OperationsError> {
        let target = self.target();
        let mut client = self.client().await?;
        let tx = client.transaction().await.classify(target)?;
        statements::advisory_lock()
            .bind(&tx, &SCHEMA_LOCK)
            .one()
            .await
            .classify(target)?;
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
                for sql in [SCHEMA_SQL, PHYSICAL_SQL, RECORD_SQL] {
                    tx.batch_execute(sql).await.classify(target)?;
                }
                tx.commit().await.classify(target)?;
                Ok(Opened::Created)
            }
            Some(comment) if recorded(comment.as_deref()) == Some(SCHEMA_FINGERPRINT_HEX) => {
                tx.rollback().await.classify(target)?;
                Ok(Opened::Current)
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
