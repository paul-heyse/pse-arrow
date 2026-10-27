// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Immutable all-mode state support shared by admission and the native adapter.
use super::{DynamicDemand, IdasLinear, SupportEntry};
use crate::ProblemError;
use crate::solve::Preconditioner;

/// Complete conservative state support, including mass diagonals and its transpose.
/// Parameter columns are excluded from the state Newton matrix. Indices use the
/// pinned SUNDIALS 64-bit sparse ABI and are checked before publication.
#[derive(Debug)]
pub struct PreparedLayout {
    pub(super) columns: Vec<i64>,
    pub(super) rows: Vec<i64>,
    pub(super) columns_b: Vec<i64>,
    pub(super) rows_b: Vec<i64>,
    pub(super) transpose_slots: Vec<usize>,
    pub(super) diagonal_slots: Vec<usize>,
    pub(super) diagonal_slots_b: Vec<usize>,
}
impl PreparedLayout {
    /// Prepare once from all admitted modes' complete support, under a finite
    /// allowance. Count the borrowed entries before allocating sorting scratch;
    /// charge scratch and the simultaneously live destination arrays together.
    pub fn new<I>(states: usize, support: I, allowance: usize) -> Result<Self, ProblemError>
    where
        I: Iterator<Item = SupportEntry> + Clone,
    {
        i64::try_from(states).map_err(|_| ProblemError::memory("IDAS state index extent"))?;
        let entries = support
            .clone()
            .filter(|e| e.col.get() < states && e.row.get() < states)
            .count()
            .checked_add(states)
            .ok_or_else(|| ProblemError::memory("IDAS support extent"))?;
        let construction = Self::construction_bytes(states, entries)
            .ok_or_else(|| ProblemError::memory("IDAS support construction extent"))?;
        if construction > allowance {
            return Err(ProblemError::memory(
                "IDAS support construction exceeds allowance",
            ));
        }
        let mut pairs = Vec::with_capacity(entries);
        pairs.extend((0..states).map(|i| (i, i)));
        pairs.extend(
            support
                .filter(|e| e.col.get() < states && e.row.get() < states)
                .map(|e| (e.col.get(), e.row.get())),
        );
        pairs.sort_unstable();
        pairs.dedup();
        let (columns, rows) = csc(states, &pairs)?;
        // Reuse the admitted scratch for the transpose instead of retaining a
        // second coordinate representation or a hash table.
        for (col, row) in &mut pairs {
            std::mem::swap(col, row);
        }
        pairs.sort_unstable();
        let (columns_b, rows_b) = csc(states, &pairs)?;
        let mut diagonal_slots = Vec::with_capacity(states);
        let mut diagonal_slots_b = Vec::with_capacity(states);
        for i in 0..states {
            diagonal_slots.push(
                slot(&columns, &rows, i, i)
                    .ok_or_else(|| ProblemError::internal("IDAS diagonal absent"))?,
            );
            diagonal_slots_b.push(
                slot(&columns_b, &rows_b, i, i)
                    .ok_or_else(|| ProblemError::internal("IDAS transpose diagonal absent"))?,
            );
        }
        let mut transpose_slots = Vec::with_capacity(rows.len());
        for col in 0..states {
            for &row in &rows[columns[col] as usize..columns[col + 1] as usize] {
                transpose_slots.push(
                    slot(&columns_b, &rows_b, row as usize, col)
                        .ok_or_else(|| ProblemError::internal("IDAS transpose entry absent"))?,
                );
            }
        }
        Ok(Self {
            columns,
            rows,
            columns_b,
            rows_b,
            transpose_slots,
            diagonal_slots,
            diagonal_slots_b,
        })
    }
    /// Number of admitted state coordinates.
    pub fn states(&self) -> usize {
        self.columns.len() - 1
    }
    /// Occupied all-mode state entries, including required diagonals.
    pub fn nonzeros(&self) -> usize {
        self.rows.len()
    }
    /// Destination of a declared state partial. Sparse column lookup also handles
    /// subset and noncanonical oracle output without another coordinate owner.
    #[cfg(any(feature = "idas", feature = "diffsol", test))]
    pub(super) fn get(&self, col: usize, row: usize) -> Option<usize> {
        slot(&self.columns, &self.rows, col, row)
    }
    /// Upper bound for layout construction with borrowed state-support entries,
    /// before adding required diagonals. Includes sorting scratch and simultaneous
    /// destination arrays; the caller separately owns any copied source support.
    pub fn construction_bound(states: usize, support_entries: usize) -> Option<usize> {
        Self::construction_bytes(states, support_entries.checked_add(states)?)
    }
    fn construction_bytes(states: usize, entries: usize) -> Option<usize> {
        states
            .checked_add(1)?
            .checked_mul(2 * size_of::<i64>())?
            .checked_add(entries.checked_mul(2 * size_of::<i64>() + size_of::<usize>())?)?
            .checked_add(states.checked_mul(2 * size_of::<usize>())?)?
            .checked_add(entries.checked_mul(size_of::<(usize, usize)>())?)?
            .checked_add(size_of::<Self>())
    }
    /// Retained Rust layout storage, including actual container capacities.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + (self.columns.capacity()
                + self.rows.capacity()
                + self.columns_b.capacity()
                + self.rows_b.capacity())
                * size_of::<i64>()
            + (self.transpose_slots.capacity()
                + self.diagonal_slots.capacity()
                + self.diagonal_slots_b.capacity())
                * size_of::<usize>()
    }
    /// Known prepared pattern/value storage and peak second-order construction.
    pub fn scatter_bytes(&self, demand: DynamicDemand) -> Result<usize, ProblemError> {
        let n = self.states();
        let entries = self.nonzeros();
        let base = self
            .retained_bytes()
            .checked_add(
                entries
                    .checked_mul(2 * size_of::<f64>())
                    .ok_or_else(|| ProblemError::memory("IDAS scatter values extent"))?,
            )
            .ok_or_else(|| ProblemError::memory("IDAS scatter extent"))?;
        if demand != DynamicDemand::ExactHessian {
            return Ok(base);
        }
        // 3 transposed blocks, maps, diagonal maps, sparse colouring and sorting
        // scratch overlap. Colouring stores occupied rows, never states² bits.
        (|| {
            base.checked_add(
                n.checked_mul(2)?
                    .checked_add(1)?
                    .checked_mul(size_of::<i64>())?,
            )?
            .checked_add(entries.checked_mul(3 * size_of::<i64>() + 6 * size_of::<usize>())?)?
            .checked_add(n.checked_mul(4 * size_of::<usize>() + 4 * size_of::<Vec<usize>>())?)?
            .checked_add(
                entries
                    .checked_add(n)?
                    .checked_mul(2 * size_of::<usize>())?,
            )?
            .checked_add(entries.checked_mul(3 * size_of::<(usize, usize)>())?)
        })()
        .ok_or_else(|| ProblemError::memory("IDAS second-order scatter extent"))
    }
    /// Selected forward linear workspace, with a separate finite opaque direct
    /// fill allowance. Krylov methods pay for basis/Hessenberg arrays, not KLU.
    pub fn linear_bytes(&self, linear: IdasLinear) -> Result<usize, ProblemError> {
        let n = self.states();
        linear.validate_for_states(n)?;
        let bytes = match linear {
            IdasLinear::Klu => Self::direct_bytes(n, self.nonzeros()),
            IdasLinear::Spgmr { dimension, .. }
            | IdasLinear::Spfgmr { dimension, .. }
            | IdasLinear::SpgmrBbd { dimension, .. } => {
                if matches!(
                    linear,
                    IdasLinear::Spgmr {
                        preconditioner: Preconditioner::BlockFactor,
                        ..
                    } | IdasLinear::Spfgmr {
                        preconditioner: Preconditioner::BlockFactor,
                        ..
                    }
                ) {
                    return Err(ProblemError::unsupported(
                        "IDAS block factor preconditioning is not supplied",
                    ));
                }
                let m = dimension.into_inner();
                // Exact native SPGMR/SPFGMR real workspace formulas (SUNDIALS
                // 7.9.0 SUNLinSolSpace), plus the session's state vectors.
                let solver_vectors = if matches!(
                    linear,
                    IdasLinear::Spgmr { .. } | IdasLinear::SpgmrBbd { .. }
                ) {
                    m.checked_add(5)
                } else {
                    m.checked_mul(2).and_then(|v| v.checked_add(4))
                };
                let vectors = solver_vectors.and_then(|v| v.checked_add(12));
                vectors
                    .and_then(|v| n.checked_mul(v))
                    .and_then(|v| v.checked_add(m.checked_add(5)?.checked_mul(m)?))
                    .and_then(|v| v.checked_add(2))
                    .and_then(|v| v.checked_mul(size_of::<f64>()))
                    .and_then(|v| v.checked_add(vectors?.checked_mul(512)?))
                    .and_then(|v| v.checked_add(4096))
                    .and_then(|v| v.checked_add(linear.bbd_bytes(n).ok()?))
            }
        };
        bytes.ok_or_else(|| ProblemError::memory("IDAS selected linear workspace extent"))
    }
    /// Rust-owned direction and complete result buffers for selected Krylov callbacks.
    /// Direct KLU creates no action workspace. Parameter axes are explicit because
    /// unused declared parameters cannot be inferred from Jacobian sparsity.
    /// # Errors
    /// The selected finite state/parameter storage extent overflows.
    pub fn action_bytes(
        &self,
        linear: IdasLinear,
        parameters: usize,
    ) -> Result<usize, ProblemError> {
        if linear == IdasLinear::Klu {
            return Ok(0);
        }
        self.states()
            .checked_mul(2)
            .and_then(|n| n.checked_add(parameters))
            .and_then(|n| n.checked_mul(size_of::<f64>()))
            .and_then(|n| n.checked_add(2 * size_of::<Vec<f64>>()))
            .ok_or_else(|| ProblemError::memory("IDAS action callback workspace extent"))
    }
    pub(super) fn direct_bytes(states: usize, entries: usize) -> Option<usize> {
        // Dense fill is a conservative opaque allowance, independent of sparse
        // input nonzeros. The known matrix/vectors remain explicitly represented.
        states
            .checked_mul(states)?
            .checked_mul(32)?
            .checked_add(entries.checked_mul(size_of::<i64>() + size_of::<f64>())?)?
            .checked_add(states.checked_add(1)?.checked_mul(size_of::<i64>())?)?
            .checked_add(states.checked_mul(12 * size_of::<f64>())?)?
            .checked_add(4096)
    }
}
/// CSC arrays from canonical `(column,row)` pairs. The pinned native ABI is i64.
#[allow(
    clippy::type_complexity,
    reason = "the canonical CSC pointer and row arrays are one layout"
)]
pub(super) fn csc(
    n: usize,
    pairs: &[(usize, usize)],
) -> Result<(Vec<i64>, Vec<i64>), ProblemError> {
    let mut columns = Vec::with_capacity(
        n.checked_add(1)
            .ok_or_else(|| ProblemError::memory("IDAS column extent"))?,
    );
    columns.push(0);
    let mut rows = Vec::with_capacity(pairs.len());
    let mut next = pairs.iter().peekable();
    for col in 0..n {
        while let Some(&(_, row)) = next.next_if(|(c, _)| *c == col) {
            rows.push(
                i64::try_from(row).map_err(|_| ProblemError::memory("IDAS row index extent"))?,
            );
        }
        columns.push(
            i64::try_from(rows.len())
                .map_err(|_| ProblemError::memory("IDAS entry index extent"))?,
        );
    }
    Ok((columns, rows))
}
pub(super) fn slot(columns: &[i64], rows: &[i64], col: usize, row: usize) -> Option<usize> {
    let range = *columns.get(col)? as usize..*columns.get(col.checked_add(1)?)? as usize;
    rows.get(range.clone())?
        .binary_search(&i64::try_from(row).ok()?)
        .ok()
        .map(|k| range.start + k)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamics::{Method, Profile, entries};
    use pse_model::scalars::PositiveCount;
    fn diagonal(n: usize) -> PreparedLayout {
        PreparedLayout::new(n, std::iter::empty(), usize::MAX).unwrap()
    }
    #[test]
    fn sparse_storage_follows_occupied_support_and_selected_method() {
        let a = diagonal(128);
        let b = diagonal(256);
        let sparse_a = a.scatter_bytes(DynamicDemand::Base).unwrap();
        let sparse_b = b.scatter_bytes(DynamicDemand::Base).unwrap();
        assert!(sparse_b < sparse_a * 3);
        for demand in [DynamicDemand::Base, DynamicDemand::ExactHessian] {
            assert!(b.scatter_bytes(demand).unwrap() < a.scatter_bytes(demand).unwrap() * 3);
        }
        for flexible in [false, true] {
            let dimension = PositiveCount::try_new(12).unwrap();
            let preconditioner = Preconditioner::Jacobi;
            let linear = if flexible {
                IdasLinear::Spfgmr {
                    dimension,
                    preconditioner,
                }
            } else {
                IdasLinear::Spgmr {
                    dimension,
                    preconditioner,
                }
            };
            assert!(b.linear_bytes(linear).unwrap() < a.linear_bytes(linear).unwrap() * 3);
            assert!(b.linear_bytes(linear).unwrap() < b.linear_bytes(IdasLinear::Klu).unwrap());
        }
    }
    #[test]
    fn all_mode_union_retains_transpose_and_rejects_unadmitted_support() {
        let first = entries([(0, 1), (1, 1)]);
        let other = entries([(2, 0), (0, 4)]); // Parameter support is not state support.
        let layout =
            PreparedLayout::new(3, first.iter().chain(&other).copied(), usize::MAX).unwrap();
        assert_eq!(layout.nonzeros(), 5);
        assert!(
            layout.get(0, 2).is_some(),
            "support present only in the other mode"
        );
        assert!(layout.get(2, 0).is_none());
        for col in 0..3 {
            for k in layout.columns[col] as usize..layout.columns[col + 1] as usize {
                let target = layout.transpose_slots[k];
                let row = layout.rows[k] as usize;
                assert_eq!(
                    slot(&layout.columns_b, &layout.rows_b, row, col),
                    Some(target)
                );
            }
            assert_eq!(layout.get(col, col), Some(layout.diagonal_slots[col]));
        }
    }
    #[test]
    fn support_construction_refuses_dense_overlap_and_index_overflow() {
        let dense = entries((0..128).flat_map(|col| (0..128).map(move |row| (row, col))));
        let allowance = diagonal(128).scatter_bytes(DynamicDemand::Base).unwrap() * 4;
        assert!(PreparedLayout::new(128, dense.iter().copied(), allowance).is_err());
        assert!(PreparedLayout::new(usize::MAX, std::iter::empty(), usize::MAX).is_err());
        let needed = PreparedLayout::construction_bytes(8, 8).unwrap();
        assert!(PreparedLayout::new(8, std::iter::empty(), needed - 1).is_err());
        assert!(PreparedLayout::new(8, std::iter::empty(), needed).is_ok());
    }
    #[test]
    fn forward_krylov_adjoint_still_charges_actual_backward_klu() {
        let layout = diagonal(128);
        let profile = Profile {
            method: Method::Idas,
            ..Profile::default()
        };
        let forward = profile
            .native_scatter_bytes(&layout, DynamicDemand::Base, 0)
            .unwrap();
        assert_eq!(layout.action_bytes(IdasLinear::Klu, usize::MAX).unwrap(), 0);
        let mut krylov = profile.clone();
        krylov.idas.linear = IdasLinear::Spgmr {
            dimension: PositiveCount::try_new(12).unwrap(),
            preconditioner: Preconditioner::None,
        };
        assert_eq!(
            layout.action_bytes(krylov.idas.linear, 100).unwrap()
                - layout.action_bytes(krylov.idas.linear, 0).unwrap(),
            100 * size_of::<f64>()
        );
        assert!(layout.action_bytes(krylov.idas.linear, usize::MAX).is_err());
        assert!(
            krylov
                .native_scatter_bytes(&layout, DynamicDemand::Base, 0)
                .unwrap()
                < forward
        );
        // The checkpoint method adds the backward KLU allowance on either forward
        // route; this difference contains only the selected forward workspace.
        let contract = crate::dynamics::Contract {
            identity: pse_ids::ContentHash::from_bytes([1; 32]),
            states: vec![pse_ids::SemanticId::from_bytes([1; 16])],
            differential: vec![true],
            parameters: vec![],
            outputs: vec![pse_ids::SemanticId::from_bytes([2; 16])],
            events: vec![vec![]],
            signs: vec![],
            quadratures: vec![],
            balances: vec![],
            derivatives: pse_kernels::DerivativeOrder::First,
        };
        let small = diagonal(contract.states.len());
        assert_eq!(
            profile.checkpoint_bytes(&contract, &small).unwrap()
                - profile
                    .native_scatter_bytes(&small, DynamicDemand::Base, 0)
                    .unwrap(),
            krylov.checkpoint_bytes(&contract, &small).unwrap()
                - krylov
                    .native_scatter_bytes(&small, DynamicDemand::Base, 0)
                    .unwrap()
        );
        assert!(
            profile.checkpoint_bytes(&contract, &small).unwrap()
                - profile
                    .native_scatter_bytes(&small, DynamicDemand::Base, 0)
                    .unwrap()
                > PreparedLayout::direct_bytes(small.states(), small.nonzeros()).unwrap()
        );
    }
    #[test]
    fn bbd_explicit_band_profile_and_replacement_extent_are_admitted_before_effects() {
        use crate::dynamics::IdasBand;
        let selected = IdasLinear::SpgmrBbd {
            dimension: PositiveCount::try_new(8).unwrap(),
            difference: IdasBand { upper: 3, lower: 2 },
            retained: IdasBand { upper: 1, lower: 1 },
        };
        let wire = serde_json::to_vec(&selected).unwrap();
        assert_eq!(
            serde_json::from_slice::<IdasLinear>(&wire).unwrap(),
            selected
        );
        let layout = diagonal(4);
        let plain = IdasLinear::Spgmr {
            dimension: selected.krylov_dimension().unwrap(),
            preconditioner: Preconditioner::None,
        };
        let bytes = selected.bbd_bytes(4).unwrap();
        // Native band PP = n*(min(n-1,upper+lower)+lower+1) cells;
        // four cloned vectors, column pointers, pivots and bounded headers.
        assert_eq!(bytes, 2 * (4 * 4 * 8 + 4 * (16 + 32) + 6 * 512 + 4096));
        assert_eq!(
            layout.linear_bytes(selected).unwrap() - layout.linear_bytes(plain).unwrap(),
            bytes
        );
        assert!(selected.validate_for_states(3).is_err());
        let escaping = IdasLinear::SpgmrBbd {
            dimension: selected.krylov_dimension().unwrap(),
            difference: IdasBand { upper: 0, lower: 1 },
            retained: IdasBand { upper: 1, lower: 1 },
        };
        assert!(layout.linear_bytes(escaping).is_err());
        let huge = IdasLinear::SpgmrBbd {
            dimension: selected.krylov_dimension().unwrap(),
            difference: IdasBand {
                upper: usize::MAX - 1,
                lower: 0,
            },
            retained: IdasBand {
                upper: usize::MAX - 1,
                lower: 0,
            },
        };
        assert!(huge.bbd_bytes(usize::MAX).is_err());
        let overflow = IdasLinear::SpgmrBbd {
            dimension: selected.krylov_dimension().unwrap(),
            difference: IdasBand {
                upper: 1_999_999_999,
                lower: 1_999_999_999,
            },
            retained: IdasBand {
                upper: 1_999_999_999,
                lower: 1_999_999_999,
            },
        };
        assert!(overflow.validate_for_states(2_000_000_000).is_ok());
        assert!(overflow.bbd_bytes(2_000_000_000).is_err());
        assert!(selected.bbd_bytes(0).is_err());
    }
}
