// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! PostgreSQL values of generated registry enums and typed ids (ADR-0114 Outcome 25):
//! pure round trips against the types a server would describe.
#![allow(clippy::unwrap_used, reason = "value-protocol assertions")]

use postgres_types::{FromSql, IsNull, Kind, ToSql, Type};
use pse_ids::postgres::BytesMut;

use crate::generated::enums::{AttemptState, RuntimeTermination};
use crate::generated::identities::{AttemptId, SourceBundleId};

fn enumeration(name: &str, labels: &[&str], schema: &str) -> Type {
    Type::new(
        name.into(),
        91_001,
        Kind::Enum(labels.iter().map(|label| (*label).to_owned()).collect()),
        schema.into(),
    )
}

fn domain(name: &str, over: Type) -> Type {
    Type::new(name.into(), 91_002, Kind::Domain(over), "pse_ops".into())
}

fn round_trip<T>(value: &T, ty: &Type) -> T
where
    T: ToSql + for<'a> FromSql<'a>,
{
    let mut out = BytesMut::new();
    assert!(matches!(value.to_sql_checked(ty, &mut out), Ok(IsNull::No)));
    T::from_sql(ty, &out).unwrap()
}

#[test]
fn postgres_value_mapping_round_trips() {
    // A registry enum is its ENUM type: by name, schema and exactly its members.
    let labels = AttemptState::ALL.map(AttemptState::as_str);
    let state = enumeration("attempt_state", &labels, "pse_ops");
    for member in AttemptState::ALL {
        assert_eq!(round_trip(&member, &state), member);
    }
    assert!(!<AttemptState as ToSql>::accepts(&enumeration(
        "attempt_state",
        &labels[..3],
        "pse_ops"
    )));
    assert!(!<AttemptState as ToSql>::accepts(&enumeration(
        "attempt_state",
        &labels,
        "public"
    )));
    assert!(!<AttemptState as ToSql>::accepts(&Type::TEXT));
    let runtime = enumeration(
        "runtime_termination",
        &RuntimeTermination::ALL.map(RuntimeTermination::as_str),
        "pse_ops",
    );
    assert_eq!(
        round_trip(&RuntimeTermination::Infrastructure, &runtime),
        RuntimeTermination::Infrastructure
    );
    assert!(AttemptState::from_sql(&state, b"runing").is_err());
    // A typed id is its identity domain, or the base type a result column arrives as.
    let attempt = AttemptId::from_bytes([0x42; 16]);
    let attempt_domain = domain("attempt_id", Type::UUID);
    assert_eq!(round_trip(&attempt, &attempt_domain), attempt);
    assert_eq!(round_trip(&attempt, &Type::UUID), attempt);
    assert!(!<AttemptId as ToSql>::accepts(&domain("run_id", Type::UUID)));
    // A content-addressed identity is a domain over the checked content_hash domain.
    let bundle = SourceBundleId::from_bytes([0x17; 32]);
    let bundle_domain = domain("source_bundle_id", domain("content_hash", Type::BYTEA));
    assert_eq!(round_trip(&bundle, &bundle_domain), bundle);
    assert_eq!(round_trip(&bundle, &Type::BYTEA), bundle);
    assert!(SourceBundleId::from_sql(&Type::BYTEA, &[0; 31]).is_err());
}
