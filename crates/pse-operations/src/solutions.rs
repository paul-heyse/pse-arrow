// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The solution and warm-start store, keyed by the coordinate-compatibility stamp and the
//! preparation identity (ADR-0112 Outcome 17). The solution identity is a seed identity
//! that enters lineage, so the runtime mints it.

use chrono::{DateTime, Utc};
use pse_ids::{ContentHash, SemanticId};
use sqlx::postgres::PgRow;
use sqlx::{FromRow, Row};

use crate::codec;
use crate::error::{Classify, OperationsError, Target};
use crate::store::Store;

/// A stored solution (primal, dual, working set or basis).
#[derive(Clone, Debug, PartialEq)]
pub struct Solution {
    /// The seed identity, minted by the runtime.
    pub solution_id: SemanticId,
    /// The coordinate-compatibility stamp the payload is valid for.
    pub compatibility_stamp: ContentHash,
    /// The preparation identity that produced the coordinates.
    pub preparation_identity: ContentHash,
    /// What the payload holds (registry spelling, e.g. `primal`).
    pub kind: String,
    /// The payload encoding, e.g. `arrow-ipc`.
    pub payload_format: String,
    /// The payload format version.
    pub payload_version: i32,
    /// The encoded payload.
    pub payload: Vec<u8>,
    /// The attempt that produced it.
    pub created_by: Option<SemanticId>,
}

impl FromRow<'_, PgRow> for Solution {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            solution_id: codec::id(row, "solution_id")?,
            compatibility_stamp: codec::hash(row, "compatibility_stamp")?,
            preparation_identity: codec::hash(row, "preparation_identity")?,
            kind: row.try_get("kind")?,
            payload_format: row.try_get("payload_format")?,
            payload_version: row.try_get("payload_version")?,
            payload: row.try_get("payload")?,
            created_by: codec::opt_id(row, "created_by")?,
        })
    }
}

/// A stored solution with its storage time.
#[derive(Clone, Debug, PartialEq)]
pub struct StoredSolution {
    /// The solution.
    pub solution: Solution,
    /// When it was stored.
    pub created_at: DateTime<Utc>,
}

impl FromRow<'_, PgRow> for StoredSolution {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            solution: Solution::from_row(row)?,
            created_at: row.try_get("created_at")?,
        })
    }
}

macro_rules! solution_columns {
    () => {
        "solution_id, compatibility_stamp, preparation_identity, kind, payload_format, \
         payload_version, payload, created_by, created_at"
    };
}

/// The solution repository.
#[derive(Clone, Copy, Debug)]
pub struct Solutions<'s> {
    store: &'s Store,
}

impl<'s> Solutions<'s> {
    pub(crate) const fn new(store: &'s Store) -> Self {
        Self { store }
    }

    fn target(&self) -> &Target {
        self.store.target()
    }

    /// Store a solution. Solutions are immutable: storing an existing identity again is a
    /// [`OperationsError::Duplicate`].
    ///
    /// # Errors
    ///
    /// [`OperationsError::Duplicate`]; classified driver failures.
    pub async fn put(&self, solution: &Solution) -> Result<DateTime<Utc>, OperationsError> {
        sqlx::query_scalar(
            "INSERT INTO pse_ops.solutions (solution_id, compatibility_stamp, \
                 preparation_identity, kind, payload_format, payload_version, payload, created_by) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING created_at",
        )
        .bind(codec::uuid(solution.solution_id))
        .bind(codec::hash_bytes(&solution.compatibility_stamp))
        .bind(codec::hash_bytes(&solution.preparation_identity))
        .bind(&solution.kind)
        .bind(&solution.payload_format)
        .bind(solution.payload_version)
        .bind(&solution.payload)
        .bind(solution.created_by.map(codec::uuid))
        .fetch_one(self.store.pool())
        .await
        .classify(self.target())
    }

    /// Read one solution by identity.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn get(
        &self,
        solution: SemanticId,
    ) -> Result<Option<StoredSolution>, OperationsError> {
        sqlx::query_as(concat!(
            "SELECT ",
            solution_columns!(),
            " FROM pse_ops.solutions WHERE solution_id = $1"
        ))
        .bind(codec::uuid(solution))
        .fetch_optional(self.store.pool())
        .await
        .classify(self.target())
    }

    /// The newest solution of `kind` compatible with `stamp` under `preparation`: the seed
    /// a warm start may reuse.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn latest_compatible(
        &self,
        stamp: &ContentHash,
        preparation: &ContentHash,
        kind: &str,
    ) -> Result<Option<StoredSolution>, OperationsError> {
        sqlx::query_as(concat!(
            "SELECT ",
            solution_columns!(),
            " FROM pse_ops.solutions \
             WHERE compatibility_stamp = $1 AND preparation_identity = $2 AND kind = $3 \
             ORDER BY created_at DESC, solution_id DESC LIMIT 1"
        ))
        .bind(codec::hash_bytes(stamp))
        .bind(codec::hash_bytes(preparation))
        .bind(kind)
        .fetch_optional(self.store.pool())
        .await
        .classify(self.target())
    }
}
