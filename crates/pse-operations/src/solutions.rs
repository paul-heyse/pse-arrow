// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The solution and warm-start store, keyed by the coordinate-compatibility stamp and the
//! preparation identity (ADR-0112 Outcome 17). Seeds are typed vectors in original source
//! coordinates, stored exactly. The solution identity is a seed identity that enters
//! lineage, so the runtime mints it.

use chrono::{DateTime, Utc};
use pse_ids::{ContentHash, SemanticId};
use pse_model::generated::enums::NativeBackend;
use sqlx::postgres::PgRow;
use sqlx::{FromRow, Row};

use crate::codec;
use crate::error::{Classify, OperationsError, Target};
use crate::store::Store;
pub use pse_model::generated::enums::StoredSeedKind;

/// The vectors of a stored seed, by native meaning. Which vectors exist is fixed by the
/// variant; the database enforces the same rule.
#[derive(Clone, Debug, PartialEq)]
pub enum SeedVectors {
    /// Root-system initial values.
    Root {
        /// Primal in source order.
        primal: Vec<f64>,
    },
    /// A primal with optional bound and row multipliers.
    Nlp {
        /// Primal in source order.
        primal: Vec<f64>,
        /// Lower and upper bound multipliers.
        bounds: Option<(Vec<f64>, Vec<f64>)>,
        /// Constraint multipliers.
        rows: Option<Vec<f64>>,
    },
    /// A `HiGHS` primal, column/row duals and simplex basis, any of them present.
    Highs {
        /// Primal in source order.
        primal: Option<Vec<f64>>,
        /// Column and row duals.
        dual: Option<(Vec<f64>, Vec<f64>)>,
        /// Native column and row basis statuses.
        basis: Option<(Vec<i32>, Vec<i32>)>,
    },
}

impl SeedVectors {
    /// The registry kind of these vectors.
    pub const fn kind(&self) -> StoredSeedKind {
        match self {
            Self::Root { .. } => StoredSeedKind::Root,
            Self::Nlp { .. } => StoredSeedKind::Nlp,
            Self::Highs { .. } => StoredSeedKind::Highs,
        }
    }
}

/// A stored solution: a portable seed and its compatibility evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct Solution {
    /// The seed identity, minted by the runtime.
    pub solution_id: SemanticId,
    /// The coordinate-compatibility (layout) stamp the vectors are valid for.
    pub compatibility_stamp: ContentHash,
    /// The preparation identity that produced the coordinates.
    pub preparation_identity: ContentHash,
    /// The backend whose payload vocabulary the vectors use.
    pub backend: NativeBackend,
    /// The native profile stamp of the producing attempt.
    pub profile_stamp: ContentHash,
    /// The numeric data stamp of the producing attempt.
    pub data_stamp: ContentHash,
    /// The vectors.
    pub vectors: SeedVectors,
    /// The attempt that produced it.
    pub created_by: Option<SemanticId>,
}

fn vectors(row: &PgRow) -> Result<SeedVectors, sqlx::Error> {
    let kind: StoredSeedKind = codec::parsed(row, "kind")?;
    let reals = |column: &str| row.try_get::<Option<Vec<f64>>, _>(column);
    let codes = |column: &str| row.try_get::<Option<Vec<i32>>, _>(column);
    let pair = |a: Option<Vec<f64>>, b: Option<Vec<f64>>| a.zip(b);
    let missing = || sqlx::Error::ColumnDecode {
        index: "primal".to_owned(),
        source: format!("a {} seed requires a primal", kind.as_str()).into(),
    };
    Ok(match kind {
        StoredSeedKind::Root => SeedVectors::Root {
            primal: reals("primal")?.ok_or_else(missing)?,
        },
        StoredSeedKind::Nlp => SeedVectors::Nlp {
            primal: reals("primal")?.ok_or_else(missing)?,
            bounds: pair(reals("lower_bound_duals")?, reals("upper_bound_duals")?),
            rows: reals("row_duals")?,
        },
        StoredSeedKind::Highs => SeedVectors::Highs {
            primal: reals("primal")?,
            dual: pair(reals("column_duals")?, reals("row_duals")?),
            basis: codes("basis_columns")?.zip(codes("basis_rows")?),
        },
    })
}

impl FromRow<'_, PgRow> for Solution {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            solution_id: codec::id(row, "solution_id")?,
            compatibility_stamp: codec::hash(row, "compatibility_stamp")?,
            preparation_identity: codec::hash(row, "preparation_identity")?,
            backend: codec::parsed(row, "backend")?,
            profile_stamp: codec::hash(row, "profile_stamp")?,
            data_stamp: codec::hash(row, "data_stamp")?,
            vectors: vectors(row)?,
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
        "solution_id, compatibility_stamp, preparation_identity, kind, backend, profile_stamp, \
         data_stamp, primal, lower_bound_duals, upper_bound_duals, column_duals, row_duals, \
         basis_columns, basis_rows, created_by, created_at"
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
    /// [`OperationsError::Duplicate`]; classified driver failures, including a CHECK
    /// violation for nonconforming vectors.
    pub async fn put(&self, solution: &Solution) -> Result<DateTime<Utc>, OperationsError> {
        let none = None::<Vec<f64>>;
        let (primal, lower, upper, columns, rows, basis) = match &solution.vectors {
            SeedVectors::Root { primal } => (Some(primal.clone()), None, None, None, None, None),
            SeedVectors::Nlp {
                primal,
                bounds,
                rows,
            } => (
                Some(primal.clone()),
                bounds.as_ref().map(|b| b.0.clone()),
                bounds.as_ref().map(|b| b.1.clone()),
                none.clone(),
                rows.clone(),
                None,
            ),
            SeedVectors::Highs {
                primal,
                dual,
                basis,
            } => (
                primal.clone(),
                None,
                None,
                dual.as_ref().map(|d| d.0.clone()),
                dual.as_ref().map(|d| d.1.clone()),
                basis.clone(),
            ),
        };
        let (basis_columns, basis_rows) = basis.unzip();
        sqlx::query_scalar(
            "INSERT INTO pse_ops.solutions (solution_id, compatibility_stamp, \
                 preparation_identity, kind, backend, profile_stamp, data_stamp, primal, \
                 lower_bound_duals, upper_bound_duals, column_duals, row_duals, \
                 basis_columns, basis_rows, created_by) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15) \
             RETURNING created_at",
        )
        .bind(codec::uuid(solution.solution_id))
        .bind(codec::hash_bytes(&solution.compatibility_stamp))
        .bind(codec::hash_bytes(&solution.preparation_identity))
        .bind(solution.vectors.kind().as_str())
        .bind(solution.backend.as_str())
        .bind(codec::hash_bytes(&solution.profile_stamp))
        .bind(codec::hash_bytes(&solution.data_stamp))
        .bind(primal)
        .bind(lower)
        .bind(upper)
        .bind(columns)
        .bind(rows)
        .bind(basis_columns)
        .bind(basis_rows)
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

    /// The newest solution for `backend` compatible with `stamp` under `preparation`: the
    /// seed a warm start may reuse. Coordinates and backend must both match for a seed to
    /// be consumed.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn latest_compatible(
        &self,
        stamp: &ContentHash,
        preparation: &ContentHash,
        backend: NativeBackend,
    ) -> Result<Option<StoredSolution>, OperationsError> {
        sqlx::query_as(concat!(
            "SELECT ",
            solution_columns!(),
            " FROM pse_ops.solutions \
             WHERE compatibility_stamp = $1 AND preparation_identity = $2 AND backend = $3 \
             ORDER BY created_at DESC, solution_id DESC LIMIT 1"
        ))
        .bind(codec::hash_bytes(stamp))
        .bind(codec::hash_bytes(preparation))
        .bind(backend.as_str())
        .fetch_optional(self.store.pool())
        .await
        .classify(self.target())
    }
}
