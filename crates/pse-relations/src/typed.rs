// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Mechanical value conversion for generated rows; admission remains in `validate`.

use pse_ids::{ContentHash, SemanticId};
use pse_schema::model::Cell;

use crate::RelationError;

/// A generated row field's lossless representation in the declared cell model.
pub trait CellCodec: Sized {
    /// Encodes the value without inferring a schema.
    fn into_cell(self) -> Cell;
    /// Decodes the exact value kind, rejecting mismatches and integer overflow.
    ///
    /// # Errors
    /// A typed value error on a kind, width or range mismatch.
    fn from_cell(cell: Cell) -> Result<Self, RelationError>;
}

/// A mismatch discovered while decoding a generated value.
pub fn mismatch(expected: &str) -> RelationError {
    RelationError::Value {
        field: expected.to_owned(),
        row: 0,
        reason: "cell does not match the generated field type".to_owned(),
    }
}

macro_rules! scalar {
    ($ty:ty, $variant:ident) => {
        impl CellCodec for $ty {
            fn into_cell(self) -> Cell {
                Cell::$variant(self)
            }
            fn from_cell(cell: Cell) -> Result<Self, RelationError> {
                match cell {
                    Cell::$variant(value) => Ok(value),
                    _ => Err(mismatch(stringify!($ty))),
                }
            }
        }
    };
}
scalar!(bool, Bool);
scalar!(i64, I64);
scalar!(u64, U64);
scalar!(f64, F64);
scalar!(String, Text);
scalar!(SemanticId, Id);
scalar!(ContentHash, Hash);

macro_rules! integer {
    ($ty:ty, $variant:ident, $wide:ty) => {
        impl CellCodec for $ty {
            fn into_cell(self) -> Cell {
                Cell::$variant(<$wide>::from(self))
            }
            fn from_cell(cell: Cell) -> Result<Self, RelationError> {
                match cell {
                    Cell::$variant(value) => {
                        Self::try_from(value).map_err(|_| mismatch(stringify!($ty)))
                    }
                    _ => Err(mismatch(stringify!($ty))),
                }
            }
        }
    };
}
integer!(i16, I64, i64);
integer!(i32, I64, i64);
integer!(u8, U64, u64);
integer!(u16, U64, u64);
integer!(u32, U64, u64);

impl<T: CellCodec> CellCodec for Option<T> {
    fn into_cell(self) -> Cell {
        self.map_or(Cell::Null, T::into_cell)
    }
    fn from_cell(cell: Cell) -> Result<Self, RelationError> {
        if matches!(cell, Cell::Null) {
            Ok(None)
        } else {
            T::from_cell(cell).map(Some)
        }
    }
}

impl<T: CellCodec> CellCodec for Vec<T> {
    fn into_cell(self) -> Cell {
        Cell::List(self.into_iter().map(T::into_cell).collect())
    }
    fn from_cell(cell: Cell) -> Result<Self, RelationError> {
        match cell {
            Cell::List(values) => values.into_iter().map(T::from_cell).collect(),
            _ => Err(mismatch("list")),
        }
    }
}

impl<T: CellCodec, const N: usize> CellCodec for [T; N] {
    fn into_cell(self) -> Cell {
        Cell::List(self.into_iter().map(T::into_cell).collect())
    }
    fn from_cell(cell: Cell) -> Result<Self, RelationError> {
        Vec::<T>::from_cell(cell)?
            .try_into()
            .map_err(|_| mismatch("fixed list width"))
    }
}
