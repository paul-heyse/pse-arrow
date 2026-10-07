// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed dense index spaces at coordinate boundaries (ADR-0115, TD05).
//!
//! A row or column index is only meaningful in the space that produced it: the rows and
//! columns of a problem before presolve differ from those after it, an instance's formal
//! input slots differ from the case's global columns, and a Jacobian entry `(row, column)`
//! differs from a Hessian entry `(column, column)`. Each space here is its own newtype, so
//! an index from one space used in another fails to compile instead of yielding plausible
//! wrong numbers.
//!
//! The types sit at boundaries: maps between spaces are [`TiVec`]s keyed by the source
//! space, and sparse coordinates are [`Entry`] or [`Triplet`] values. Library and FFI
//! interiors — faer's `usize` patterns, the `i32` of Ipopt and `HiGHS` — keep their native
//! indices, and an adapter converts with [`From`] at the edge.
//!
//! A presolve map from presolved to original columns accepts only presolved columns:
//!
//! ```
//! use pse_math::index::{OriginalCol, PresolvedCol, TiVec};
//!
//! let retained: TiVec<PresolvedCol, OriginalCol> = vec![OriginalCol::new(2)].into();
//! assert_eq!(retained[PresolvedCol::new(0)], OriginalCol::new(2));
//! ```
//!
//! ```compile_fail,E0277
//! use pse_math::index::{OriginalCol, PresolvedCol, TiVec};
//!
//! let retained: TiVec<PresolvedCol, OriginalCol> = vec![OriginalCol::new(2)].into();
//! // An original column is not an index of the presolved space.
//! let _ = retained[OriginalCol::new(0)];
//! ```
//!
//! A Hessian takes `(column, column)` entries, so a Jacobian's `(row, column)` entries are
//! refused where a Hessian's are expected:
//!
//! ```
//! use pse_math::index::{GlobalCol, GlobalRow, Triplet};
//!
//! fn lagrangian<R, C>(hessian: &[Triplet<C, C>], jacobian: &[Triplet<R, C>]) -> usize {
//!     hessian.len() + jacobian.len()
//! }
//! let hessian = [Triplet::new(GlobalCol::new(0), GlobalCol::new(0), 2.0)];
//! let jacobian = [Triplet::new(GlobalRow::new(0), GlobalCol::new(0), 1.0)];
//! assert_eq!(lagrangian(&hessian, &jacobian), 2);
//! ```
//!
//! ```compile_fail,E0308
//! use pse_math::index::{GlobalCol, GlobalRow, Triplet};
//!
//! fn lagrangian<R, C>(hessian: &[Triplet<C, C>], jacobian: &[Triplet<R, C>]) -> usize {
//!     hessian.len() + jacobian.len()
//! }
//! let jacobian = [Triplet::new(GlobalRow::new(0), GlobalCol::new(0), 1.0)];
//! // A Jacobian triplet is not a Hessian triplet.
//! let _ = lagrangian(&jacobian, &jacobian);
//! ```

pub use typed_index_collections::{TiSlice, TiVec};

/// Checked identity access tied to the source inventory's lifetime. Keys describe
/// the owner's correspondence rule; repeated requests remain distinct consumers.
#[derive(Debug)]
pub struct CheckedInventory<'a, K, T> {
    source: &'a [T],
    positions: std::collections::BTreeMap<K, usize>,
}

impl<'a, K: Ord, T> CheckedInventory<'a, K, T> {
    /// Refuse ambiguous source identities before resolving any consumer.
    pub fn new(source: &'a [T], key: impl Fn(&T) -> K) -> Result<Self, crate::MathError> {
        let mut positions = std::collections::BTreeMap::new();
        for (position, value) in source.iter().enumerate() {
            if positions.insert(key(value), position).is_some() {
                return Err(crate::MathError::Contract(
                    "duplicate projection source identity".into(),
                ));
            }
        }
        Ok(Self { source, positions })
    }

    /// Transfer checked ordinals to the layout owner that retains the source inventory.
    pub fn into_positions(self) -> std::collections::BTreeMap<K, usize> {
        self.positions
    }

    /// Resolve an ordinal without treating inventory set equality as positional equality.
    pub fn position(&self, key: &K) -> Option<usize> {
        self.positions.get(key).copied()
    }

    /// Access the admitted source value with its complete contract intact.
    pub fn get(&self, key: &K) -> Option<&'a T> {
        self.position(key).map(|position| &self.source[position])
    }
}

/// Declares one dense index space: a `usize` newtype that `TiVec`/`TiSlice` accept as a key.
macro_rules! index_space {
    ($($(#[$attr:meta])* $name:ident;)*) => {$(
        $(#[$attr])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(transparent)]
        pub struct $name(usize);

        impl $name {
            /// The index at `position` of this space.
            #[must_use]
            pub const fn new(position: usize) -> Self {
                Self(position)
            }

            /// The position within this space, for a library or FFI interior.
            #[must_use]
            pub const fn get(self) -> usize {
                self.0
            }
        }

        impl From<usize> for $name {
            fn from(position: usize) -> Self {
                Self(position)
            }
        }

        impl From<$name> for usize {
            fn from(index: $name) -> Self {
                index.0
            }
        }
    )*};
}

index_space! {
    /// A row of a problem as its oracle states it, before presolve.
    OriginalRow;
    /// A column of a problem as its oracle states it, before presolve.
    OriginalCol;
    /// A row of the presolved problem a native solver receives.
    PresolvedRow;
    /// A column of the presolved problem a native solver receives.
    PresolvedCol;
    /// A row of a reduced subproblem, such as the active rows of a KKT block.
    ReducedRow;
    /// A column of a reduced subproblem, such as the free columns of a KKT block.
    ReducedCol;
    /// A formal input slot of one body binding: an instance-local coordinate.
    Slot;
    /// A row of an assembled case, in its selected row order.
    GlobalRow;
    /// A derivative column of an assembled case, in its selected column order.
    GlobalCol;
    /// One addend of an assembly matrix: an entry in the order its entries were declared.
    /// Duplicate entries stay distinct addends and accumulate into one canonical nonzero.
    Addend;
}

/// One structural sparse entry: its row in space `R` and its column in space `C`.
///
/// A Jacobian entry is `Entry<Row, Col>` and a lower-triangle Hessian entry is
/// `Entry<Col, Col>`, so the two do not substitute for each other.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entry<R, C> {
    /// The entry's row.
    pub row: R,
    /// The entry's column.
    pub col: C,
}

impl<R, C> Entry<R, C> {
    /// The entry at `(row, col)`.
    pub const fn new(row: R, col: C) -> Self {
        Self { row, col }
    }
}

/// One valued sparse entry: its row in space `R`, its column in space `C` and its value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Triplet<R, C> {
    /// The entry's row.
    pub row: R,
    /// The entry's column.
    pub col: C,
    /// The entry's value.
    pub value: f64,
}

impl<R, C> Triplet<R, C> {
    /// The entry at `(row, col)` with `value`.
    pub const fn new(row: R, col: C, value: f64) -> Self {
        Self { row, col, value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_inventory_preserves_order_multiplicity_and_refuses_ambiguity() {
        let source = [("row", 7), ("variable", 7), ("row", 2)];
        let index = CheckedInventory::new(&source, |value| *value).unwrap();
        assert_eq!(index.position(&("row", 2)), Some(2));
        assert_eq!(index.position(&("variable", 7)), Some(1));
        assert_eq!(index.get(&("row", 7)), index.get(&("row", 7)));
        assert_eq!(index.position(&("row", 9)), None);
        assert!(CheckedInventory::new(&[(1, 2), (1, 3)], |value| value.0).is_err());
    }

    #[test]
    fn index_spaces_round_trip_their_position() {
        assert_eq!(usize::from(OriginalCol::from(7)), 7);
        assert_eq!(PresolvedRow::new(3).get(), 3);
        let map: TiVec<PresolvedCol, OriginalCol> =
            vec![OriginalCol::new(4), OriginalCol::new(1)].into();
        assert_eq!(
            map.iter_enumerated()
                .map(|(p, o)| (p.get(), o.get()))
                .collect::<Vec<_>>(),
            [(0, 4), (1, 1)]
        );
    }
}
