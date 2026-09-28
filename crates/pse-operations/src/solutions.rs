// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The solution and warm-start store, keyed by the coordinate-compatibility stamp and the
//! preparation identity (ADR-0114 Outcome 17). Seeds are typed vectors in original source
//! coordinates, stored exactly; reads return the `runtime.operational_solutions` row, and
//! [`SeedVectors::of`] gives its vectors their native meaning. The solution identity is a
//! seed identity that enters lineage, so the runtime mints it.

use pse_ids::ContentHash;
use pse_model::generated::enums::NativeBackend;
use pse_operations_queries::client::Params as _;
use pse_operations_queries::queries::solutions as statements;

use crate::attempts::AttemptId;
use crate::error::{Classify, OperationsError, Target};
use crate::store::Store;
pub use pse_model::generated::enums::StoredSeedKind;
pub use pse_model::generated::identities::SolutionId;
pub use pse_model::generated::runtime::operational_solutions::RuntimeOperationalSolutionsRow;

/// The vectors of a stored seed, by native meaning. Which vectors exist is fixed by the
/// variant; the database enforces the same rule.
#[derive(Clone, Debug, PartialEq)]
pub enum SeedVectors {
    /// Root-system initial values.
    Root {
        /// Primal in source order.
        primal: Vec<f64>,
    },
    /// A primal with optional bound and row multipliers and the final barrier parameter of
    /// its interior-point producer.
    Nlp {
        /// Primal in source order.
        primal: Vec<f64>,
        /// Lower and upper bound multipliers.
        bounds: Option<(Vec<f64>, Vec<f64>)>,
        /// Constraint multipliers.
        rows: Option<Vec<f64>>,
        /// Final barrier parameter, in authored objective units.
        barrier: Option<f64>,
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

    /// The vectors a stored solution holds, by its kind.
    ///
    /// # Errors
    /// [`OperationsError::CorruptValue`] for a root or NLP seed without its primal (the
    /// store's row check refuses one).
    pub fn of(row: &RuntimeOperationalSolutionsRow) -> Result<Self, OperationsError> {
        let primal = || {
            row.primal.clone().ok_or_else(|| OperationsError::CorruptValue {
                column: "solutions.primal",
                detail: format!("a {} seed requires a primal", row.kind.as_str()),
            })
        };
        Ok(match row.kind {
            StoredSeedKind::Root => Self::Root { primal: primal()? },
            StoredSeedKind::Nlp => Self::Nlp {
                primal: primal()?,
                bounds: row
                    .lower_bound_duals
                    .clone()
                    .zip(row.upper_bound_duals.clone()),
                rows: row.row_duals.clone(),
                barrier: row.barrier,
            },
            StoredSeedKind::Highs => Self::Highs {
                primal: row.primal.clone(),
                dual: row.column_duals.clone().zip(row.row_duals.clone()),
                basis: row.basis_columns.clone().zip(row.basis_rows.clone()),
            },
        })
    }
}

/// A seed to store: its identity, its compatibility evidence and its vectors.
#[derive(Clone, Debug, PartialEq)]
pub struct NewSolution {
    /// The seed identity, minted by the runtime.
    pub solution_id: SolutionId,
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
    pub created_by: Option<AttemptId>,
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

    /// Store a solution and return it as stored. Solutions are immutable: storing an
    /// existing identity again is a [`OperationsError::Duplicate`].
    ///
    /// # Errors
    ///
    /// [`OperationsError::Duplicate`]; [`OperationsError::InvariantViolation`] for
    /// nonconforming vectors; classified driver failures.
    pub async fn put(
        &self,
        solution: &NewSolution,
    ) -> Result<RuntimeOperationalSolutionsRow, OperationsError> {
        let none: Option<&[f64]> = None;
        let (primal, lower, upper, columns, rows, barrier, basis) = match &solution.vectors {
            SeedVectors::Root { primal } => {
                (Some(primal.as_slice()), none, none, none, none, None, None)
            }
            SeedVectors::Nlp {
                primal,
                bounds,
                rows,
                barrier,
            } => (
                Some(primal.as_slice()),
                bounds.as_ref().map(|b| b.0.as_slice()),
                bounds.as_ref().map(|b| b.1.as_slice()),
                none,
                rows.as_deref(),
                *barrier,
                None,
            ),
            SeedVectors::Highs {
                primal,
                dual,
                basis,
            } => (
                primal.as_deref(),
                none,
                none,
                dual.as_ref().map(|d| d.0.as_slice()),
                dual.as_ref().map(|d| d.1.as_slice()),
                None,
                basis.as_ref(),
            ),
        };
        let client = self.store.client().await?;
        statements::insert_solution()
            .params(
                &client,
                &statements::InsertSolutionParams {
                    solution_id: solution.solution_id,
                    compatibility_stamp: solution.compatibility_stamp,
                    preparation_identity: solution.preparation_identity,
                    kind: solution.vectors.kind(),
                    backend: solution.backend,
                    profile_stamp: solution.profile_stamp,
                    data_stamp: solution.data_stamp,
                    primal,
                    lower_bound_duals: lower,
                    upper_bound_duals: upper,
                    column_duals: columns,
                    row_duals: rows,
                    barrier,
                    basis_columns: basis.map(|b| b.0.as_slice()),
                    basis_rows: basis.map(|b| b.1.as_slice()),
                    created_by: solution.created_by,
                },
            )
            .one()
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
        solution: SolutionId,
    ) -> Result<Option<RuntimeOperationalSolutionsRow>, OperationsError> {
        let client = self.store.client().await?;
        statements::solution()
            .bind(&client, &solution)
            .opt()
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
    ) -> Result<Option<RuntimeOperationalSolutionsRow>, OperationsError> {
        let client = self.store.client().await?;
        statements::latest_compatible()
            .params(
                &client,
                &statements::LatestCompatibleParams {
                    compatibility_stamp: *stamp,
                    preparation_identity: *preparation,
                    backend,
                },
            )
            .opt()
            .await
            .classify(self.target())
    }
}
