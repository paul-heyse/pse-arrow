// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact binary content: the value of a registry `Binary` column (ADR-0125).

/// Exact bytes, compared, framed and stored byte for byte. A package data document's
/// content is one; text is never implied.
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bytes(Vec<u8>);

impl Bytes {
    /// Own `bytes` without copying.
    pub const fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
    /// The exact bytes.
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
    /// Release the owned bytes.
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl std::ops::Deref for Bytes {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Debug for Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bytes({} bytes)", self.0.len())
    }
}

impl crate::SemanticEq for Bytes {
    fn semantic_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl crate::HeapUsage for Bytes {
    fn heap_bytes(&self) -> usize {
        self.0.capacity()
    }
}

impl crate::SemanticFrame for Bytes {
    fn frame(&self, hash: &mut pse_ids::FramedHasher) {
        hash.part(&self.0);
    }
}

#[cfg(feature = "postgres")]
mod store {
    use super::Bytes;
    use postgres_types::{FromSql, IsNull, ToSql, Type, to_sql_checked};
    use pse_ids::postgres::BytesMut;

    type BoxError = Box<dyn std::error::Error + Sync + Send>;

    /// PostgreSQL `bytea`.
    impl ToSql for Bytes {
        fn to_sql(
            &self,
            ty: &Type,
            out: &mut BytesMut,
        ) -> Result<IsNull, BoxError> {
            self.as_slice().to_sql(ty, out)
        }
        fn accepts(ty: &Type) -> bool {
            <&[u8] as ToSql>::accepts(ty)
        }
        to_sql_checked!();
    }

    impl<'a> FromSql<'a> for Bytes {
        fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, BoxError> {
            Vec::<u8>::from_sql(ty, raw).map(Self)
        }
        fn accepts(ty: &Type) -> bool {
            <Vec<u8> as FromSql>::accepts(ty)
        }
    }
}
