// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! PostgreSQL values of identities (ADR-0114 Outcome 25), behind the `postgres` feature.
//!
//! A [`SemanticId`] is a `uuid` (16 bytes on the wire); a [`ContentHash`] is 32 bytes of
//! `bytea`, stored through a `content_hash` domain whose CHECK fixes the width, and
//! refused on decode when the width differs. Typed ids generated from the registry accept
//! their identity domain as well as the base type: a parameter is inferred as the domain,
//! while a result column of a domain type arrives as its base type.
//!
//! This is the value protocol only, never a driver.

use postgres_types::{FromSql, IsNull, Kind, ToSql, Type, to_sql_checked};

use crate::{ContentHash, SemanticId};

/// The buffer `ToSql` writes into, re-exported so generated impls need no second pin.
pub use bytes::BytesMut;

type BoxError = Box<dyn std::error::Error + Sync + Send>;

/// The name of the domain every content hash is stored as.
pub const CONTENT_HASH_DOMAIN: &str = "content_hash";

/// The base type beneath any chain of domains, or `ty` itself.
pub fn base(ty: &Type) -> &Type {
    match ty.kind() {
        Kind::Domain(inner) => base(inner),
        _ => ty,
    }
}

/// Whether `ty` is a type `T` accepts, or the domain `schema.name` over one.
pub fn accepts_domain<T: ToSql>(ty: &Type, schema: &str, name: &str) -> bool {
    T::accepts(ty)
        || (ty.schema() == schema
            && ty.name() == name
            && matches!(ty.kind(), Kind::Domain(inner) if T::accepts(inner)))
}

impl ToSql for SemanticId {
    fn to_sql(&self, _: &Type, out: &mut BytesMut) -> Result<IsNull, BoxError> {
        out.extend_from_slice(self.as_bytes());
        Ok(IsNull::No)
    }
    fn accepts(ty: &Type) -> bool {
        *ty == Type::UUID
    }
    to_sql_checked!();
}

impl<'a> FromSql<'a> for SemanticId {
    fn from_sql(_: &Type, raw: &'a [u8]) -> Result<Self, BoxError> {
        Ok(Self::try_from_slice(raw)?)
    }
    fn accepts(ty: &Type) -> bool {
        *ty == Type::UUID
    }
}

impl ToSql for ContentHash {
    fn to_sql(&self, _: &Type, out: &mut BytesMut) -> Result<IsNull, BoxError> {
        out.extend_from_slice(self.as_bytes());
        Ok(IsNull::No)
    }
    fn accepts(ty: &Type) -> bool {
        *ty == Type::BYTEA
            || (ty.name() == CONTENT_HASH_DOMAIN
                && matches!(ty.kind(), Kind::Domain(inner) if *inner == Type::BYTEA))
    }
    to_sql_checked!();
}

impl<'a> FromSql<'a> for ContentHash {
    fn from_sql(_: &Type, raw: &'a [u8]) -> Result<Self, BoxError> {
        Ok(Self::try_from_slice(raw)?)
    }
    fn accepts(ty: &Type) -> bool {
        <Self as ToSql>::accepts(ty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn domain(name: &str, over: Type) -> Type {
        Type::new(name.into(), 90_001, Kind::Domain(over), "pse_ops".into())
    }

    fn round_trip<T>(value: T, ty: &Type) -> T
    where
        T: ToSql + for<'a> FromSql<'a> + std::fmt::Debug,
    {
        let mut out = BytesMut::new();
        assert!(matches!(value.to_sql_checked(ty, &mut out), Ok(IsNull::No)));
        T::from_sql(ty, &out).unwrap_or_else(|error| panic!("decode failed: {error}"))
    }

    #[test]
    fn postgres_value_mapping_round_trips() {
        let id = SemanticId::from_bytes([0x5a; 16]);
        assert_eq!(round_trip(id, &Type::UUID), id);
        let hash = ContentHash::from_bytes([0xc3; 32]);
        assert_eq!(round_trip(hash, &Type::BYTEA), hash);
        let content = domain(CONTENT_HASH_DOMAIN, Type::BYTEA);
        assert_eq!(round_trip(hash, &content), hash);
        // A bytea of the wrong width is refused on decode, never truncated or padded.
        assert!(ContentHash::from_sql(&Type::BYTEA, &[0; 31]).is_err());
        assert!(SemanticId::from_sql(&Type::UUID, &[0; 17]).is_err());
        // Identity domains resolve to their base; other types are refused.
        let attempt = domain("attempt_id", Type::UUID);
        assert!(accepts_domain::<SemanticId>(&attempt, "pse_ops", "attempt_id"));
        assert!(accepts_domain::<SemanticId>(&Type::UUID, "pse_ops", "attempt_id"));
        assert!(!accepts_domain::<SemanticId>(&attempt, "pse_ops", "run_id"));
        assert!(!<SemanticId as ToSql>::accepts(&Type::TEXT));
        let bundle = domain("source_bundle_id", content);
        assert_eq!(base(&bundle), &Type::BYTEA);
        assert!(accepts_domain::<ContentHash>(&bundle, "pse_ops", "source_bundle_id"));
    }
}
