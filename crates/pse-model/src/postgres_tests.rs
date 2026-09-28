// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! PostgreSQL values of generated registry enums and typed ids (ADR-0114 Outcome 25):
//! pure round trips against the types a server would describe.
#![allow(clippy::unwrap_used, reason = "value-protocol assertions")]

use postgres_types::{FromSql, IsNull, Kind, ToSql, Type};
use pse_ids::postgres::BytesMut;

use crate::generated::enums::{AttemptState, DiagnosticCode, RuntimeTermination};
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
    // A source-owned vocabulary maps through its owning crate's `postgres` feature, to
    // the ENUM of its snake-case name with exactly its members (X4's rule column).
    let codes = DiagnosticCode::ALL
        .iter()
        .map(|code| code.as_str())
        .collect::<Vec<_>>();
    let rule = enumeration("diagnostic_code", &codes, "pse_ops");
    for code in DiagnosticCode::ALL {
        assert_eq!(round_trip(code, &rule), *code);
    }
    assert!(!<DiagnosticCode as ToSql>::accepts(&enumeration(
        "diagnostic_code",
        &codes[..3],
        "pse_ops"
    )));
    assert!(!<DiagnosticCode as ToSql>::accepts(&enumeration(
        "failure_class",
        &codes,
        "pse_ops"
    )));
    assert!(!<DiagnosticCode as ToSql>::accepts(&Type::TEXT));
    assert!(DiagnosticCode::from_sql(&rule, b"solve.nonsense").is_err());
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
    // A store row is its table's composite row type: fields checked by name, typed ids
    // read through their domains, enums through their ENUM types, timestamps as µs.
    whole_rows_decode_from_composite_records();
}

/// One composite field as `record_send` writes it: type oid, length (-1 for NULL), bytes.
fn field(record: &mut Vec<u8>, ty: &Type, bytes: Option<&[u8]>) {
    record.extend_from_slice(&ty.oid().to_be_bytes());
    match bytes {
        Some(bytes) => {
            record.extend_from_slice(&i32::try_from(bytes.len()).unwrap().to_be_bytes());
            record.extend_from_slice(bytes);
        }
        None => record.extend_from_slice(&(-1_i32).to_be_bytes()),
    }
}

fn encoded<T: ToSql>(value: &T, ty: &Type) -> Vec<u8> {
    let mut out = BytesMut::new();
    assert!(matches!(value.to_sql_checked(ty, &mut out), Ok(IsNull::No)));
    out.to_vec()
}

fn whole_rows_decode_from_composite_records() {
    use crate::generated::runtime::operational_attempt_transitions::RuntimeOperationalAttemptTransitionsRow as Row;
    let state = enumeration(
        "attempt_state",
        &AttemptState::ALL.map(AttemptState::as_str),
        "pse_ops",
    );
    let attempt_domain = domain("attempt_id", Type::UUID);
    let columns = [
        ("attempt_id", attempt_domain.clone()),
        ("seq", Type::INT4),
        ("from_state", state.clone()),
        ("to_state", state.clone()),
        ("actor", Type::TEXT),
        ("reason", Type::TEXT),
        ("at", Type::TIMESTAMPTZ),
    ];
    let row_type = |columns: &[(&str, Type)]| {
        Type::new(
            "attempt_transitions".into(),
            91_003,
            Kind::Composite(
                columns
                    .iter()
                    .map(|(name, ty)| postgres_types::Field::new((*name).to_owned(), ty.clone()))
                    .collect(),
            ),
            "pse_ops".into(),
        )
    };
    let ty = row_type(&columns);
    let attempt = AttemptId::from_bytes([0x42; 16]);
    // 2026-01-01T00:00:00Z: 1_767_225_600 s after the Unix epoch.
    let unix_micros = 1_767_225_600_000_000_i64;
    let mut at = BytesMut::new();
    postgres_protocol::types::timestamp_to_sql(unix_micros - 946_684_800_000_000, &mut at);
    let mut record = 7_i32.to_be_bytes().to_vec();
    field(&mut record, &attempt_domain, Some(&encoded(&attempt, &attempt_domain)));
    field(&mut record, &Type::INT4, Some(&3_i32.to_be_bytes()));
    field(&mut record, &state, Some(b"queued"));
    field(&mut record, &state, Some(b"running"));
    field(&mut record, &Type::TEXT, Some(b"worker-1"));
    field(&mut record, &Type::TEXT, None);
    field(&mut record, &Type::TIMESTAMPTZ, Some(&at));
    assert!(<Row as FromSql>::accepts(&ty));
    let row = Row::from_sql(&ty, &record).unwrap();
    assert_eq!(row.attempt_id, attempt);
    assert_eq!(row.seq, 3);
    assert_eq!(row.from_state, Some(AttemptState::Queued));
    assert_eq!(row.to_state, AttemptState::Running);
    assert_eq!(row.actor.as_deref(), Some("worker-1"));
    // NULL decodes as None, never as a default.
    assert_eq!(row.reason, None);
    assert_eq!(row.at, unix_micros);
    // A NULL in a required column is refused.
    let mut nulled = 7_i32.to_be_bytes().to_vec();
    field(&mut nulled, &attempt_domain, None);
    assert!(Row::from_sql(&ty, &nulled).is_err());
    // A table whose fields differ from the registry row is refused, not read by position.
    let mut renamed = columns.clone();
    renamed[4].0 = "worker";
    assert!(Row::from_sql(&row_type(&renamed), &record).is_err());
    // Trailing bytes and a short field count are refused.
    let mut trailing = record.clone();
    trailing.push(0);
    assert!(Row::from_sql(&ty, &trailing).is_err());
    let mut short = record;
    short[..4].copy_from_slice(&6_i32.to_be_bytes());
    assert!(Row::from_sql(&ty, &short).is_err());
    assert!(!<Row as FromSql>::accepts(&Type::TEXT));
}
