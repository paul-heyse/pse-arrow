// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Conversions between platform identities and their PostgreSQL columns.
//!
//! Semantic identities are `uuid` columns (the runtime mints them on its `UUIDv7` path) and
//! content hashes are 32-byte `bytea` columns. Rows decode through these helpers inside
//! hand-written `FromRow` implementations, so a corrupt value is a typed failure.

use std::str::FromStr;
use std::time::Duration;

use pse_ids::{ContentHash, SemanticId};
use sqlx::Row;
use sqlx::postgres::PgRow;
use uuid::Uuid;

/// The `uuid` column value of an identity.
pub(crate) const fn uuid(id: SemanticId) -> Uuid {
    Uuid::from_bytes(*id.as_bytes())
}

/// The `bytea` column value of a content hash.
pub(crate) fn hash_bytes(hash: &ContentHash) -> Vec<u8> {
    hash.as_bytes().to_vec()
}

/// A duration PostgreSQL `interval` can represent exactly (microsecond resolution).
pub(crate) fn interval(duration: Duration) -> Duration {
    Duration::from_micros(u64::try_from(duration.as_micros()).unwrap_or(u64::MAX))
}

fn decode_error(column: &str, detail: impl ToString) -> sqlx::Error {
    sqlx::Error::ColumnDecode {
        index: column.to_owned(),
        source: detail.to_string().into(),
    }
}

/// A non-null identity column.
pub(crate) fn id(row: &PgRow, column: &str) -> Result<SemanticId, sqlx::Error> {
    let value: Uuid = row.try_get(column)?;
    Ok(SemanticId::from_bytes(value.into_bytes()))
}

/// A nullable identity column.
pub(crate) fn opt_id(row: &PgRow, column: &str) -> Result<Option<SemanticId>, sqlx::Error> {
    let value: Option<Uuid> = row.try_get(column)?;
    Ok(value.map(|value| SemanticId::from_bytes(value.into_bytes())))
}

fn to_hash(column: &str, bytes: &[u8]) -> Result<ContentHash, sqlx::Error> {
    let sized: [u8; ContentHash::WIDTH] = bytes.try_into().map_err(|_| {
        decode_error(
            column,
            format!(
                "expected {} bytes, found {}",
                ContentHash::WIDTH,
                bytes.len()
            ),
        )
    })?;
    Ok(ContentHash::from_bytes(sized))
}

/// A non-null content-hash column.
pub(crate) fn hash(row: &PgRow, column: &str) -> Result<ContentHash, sqlx::Error> {
    let bytes: Vec<u8> = row.try_get(column)?;
    to_hash(column, &bytes)
}

/// A text column holding a Rust enumeration's spelling.
pub(crate) fn parsed<T>(row: &PgRow, column: &str) -> Result<T, sqlx::Error>
where
    T: FromStr,
    T::Err: ToString,
{
    let text: String = row.try_get(column)?;
    text.parse()
        .map_err(|error: T::Err| decode_error(column, error))
}

#[cfg(test)]
mod codec_unit {
    use super::*;

    #[test]
    fn intervals_drop_sub_microsecond_precision() {
        assert_eq!(
            interval(Duration::from_nanos(1_500_999)),
            Duration::from_micros(1_500)
        );
    }

    #[test]
    fn short_hash_is_a_decode_error() {
        assert!(matches!(
            to_hash("h", &[0; 31]),
            Err(sqlx::Error::ColumnDecode { .. })
        ));
        assert_eq!(
            to_hash("h", &[7; 32]).ok(),
            Some(ContentHash::from_bytes([7; 32]))
        );
    }
}
