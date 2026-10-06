// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native protocol values: authoritative bits and checked decimal integers.

use surrealdb::types::{Bytes, Decimal, Number, Object, Value};

/// Refusal of an incompatible or corrupt scientific wire value.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[diagnostic(code(validation::invariant))]
pub enum CodecError {
    /// A required field is absent; absence is never fabricated as zero.
    #[error("missing canonical field {0}")]
    Missing(&'static str),
    /// The native value kind/domain differs from its declaration.
    #[error("incompatible canonical value: expected {0}")]
    Domain(&'static str),
    /// Unknown fields require a compatible reader interpretation.
    #[error("unknown canonical fields")]
    UnknownFields,
}
impl pse_diagnostics::TypedDiagnostic for CodecError {
    fn diagnostic_code(&self) -> Option<pse_diagnostics::DiagnosticCode> {
        Some(pse_diagnostics::DiagnosticCode::ValidationInvariant)
    }
}

pub(crate) fn required(object: &mut Object, field: &'static str) -> Result<Value, CodecError> {
    object.remove(field).ok_or(CodecError::Missing(field))
}

/// Unsigned values never pass through the SDK's signed ordinary integer conversion.
pub fn encode_uint(value: u64) -> Result<Value, CodecError> {
    Ok(Value::Number(Number::Decimal(Decimal::from(value))))
}
/// Validate integrality before conversion, since numeric conversions can truncate.
pub fn decode_uint(value: Value) -> Result<u64, CodecError> {
    let Value::Number(Number::Decimal(value)) = value else {
        return Err(CodecError::Domain("native unsigned decimal integer"));
    };
    if !value.fract().is_zero() || value < Decimal::ZERO || value > Decimal::from(u64::MAX) {
        return Err(CodecError::Domain("decimal integer in the u64 domain"));
    }
    value
        .normalize()
        .to_string()
        .parse()
        .map_err(|_| CodecError::Domain("decimal integer in the u64 domain"))
}
/// Encode declared text without record-ID coercion.
pub fn encode_string(value: String) -> Result<Value, CodecError> {
    Ok(Value::String(value))
}
/// Decode declared text without record-ID coercion.
pub fn decode_string(value: Value) -> Result<String, CodecError> {
    match value {
        Value::String(value) => Ok(value),
        _ => Err(CodecError::Domain("string")),
    }
}
/// Native record identity is distinct from declared text; projections derive it exactly.
pub fn record_link(table: &str, key: &str) -> Value {
    Value::RecordId(surrealdb::types::RecordId::new(table, key))
}
/// Refuse string coercion, a wrong table, a numeric key or a mismatched declared key.
pub fn check_record_identity(value: Value, table: &str, key: &str) -> Result<(), CodecError> {
    if value == record_link(table, key) {
        Ok(())
    } else {
        Err(CodecError::Domain("matching native record identity"))
    }
}
/// Validate a generated graph endpoint against its authoritative reference field.
pub fn check_record_link(
    object: &mut Object,
    field: &'static str,
    table: &str,
    key: &str,
) -> Result<(), CodecError> {
    check_record_identity(required(object, field)?, table, key)
}
/// Encode opaque bytes using the native binary value kind.
pub fn encode_bytes(value: pse_model::Bytes) -> Result<Value, CodecError> {
    Ok(Value::Bytes(Bytes::from(value.as_slice().to_vec())))
}
/// Decode native binary values without JSON conversion.
pub fn decode_bytes(value: Value) -> Result<pse_model::Bytes, CodecError> {
    match value {
        Value::Bytes(value) => Ok(value.to_vec().into()),
        _ => Err(CodecError::Domain("bytes")),
    }
}
/// Encode a signed integer or microsecond timestamp.
pub fn encode_int(value: i64) -> Result<Value, CodecError> {
    Ok(Value::Number(Number::Int(value)))
}
/// Decode the exact signed native integer kind.
pub fn decode_int(value: Value) -> Result<i64, CodecError> {
    match value {
        Value::Number(Number::Int(value)) => Ok(value),
        _ => Err(CodecError::Domain("signed integer")),
    }
}
/// Encode a declared predicate.
pub fn encode_boolean(value: bool) -> Result<Value, CodecError> {
    Ok(Value::Bool(value))
}
/// Decode a declared predicate without truthiness coercion.
pub fn decode_boolean(value: Value) -> Result<bool, CodecError> {
    match value {
        Value::Bool(value) => Ok(value),
        _ => Err(CodecError::Domain("boolean")),
    }
}
/// Raw diagnostics preserve all IEEE bit patterns, without asserting physical validity.
pub fn encode_diagnostic_bits(bits: u64) -> Value {
    Value::Bytes(Bytes::from(bits.to_be_bytes().to_vec()))
}
/// Decode the exact raw IEEE diagnostic payload.
pub fn decode_diagnostic_bits(value: Value) -> Result<u64, CodecError> {
    let bytes = decode_bytes(value)?;
    let bytes: [u8; 8] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| CodecError::Domain("eight IEEE-754 bytes"))?;
    Ok(u64::from_be_bytes(bytes))
}
/// Projection is queryable; authoritative bytes preserve signed zero.
pub fn encode_finite(value: f64) -> Result<Value, CodecError> {
    if !value.is_finite() {
        return Err(CodecError::Domain("finite scientific f64"));
    }
    let mut object = Object::new();
    object.insert("bits", encode_diagnostic_bits(value.to_bits()));
    object.insert("projection", Value::Number(Number::Float(value)));
    Ok(Value::Object(object))
}
/// Check the finite domain and projection against authoritative bits.
pub fn decode_finite(value: Value) -> Result<f64, CodecError> {
    let Value::Object(mut object) = value else {
        return Err(CodecError::Domain("exact scientific cell"));
    };
    let bits = decode_diagnostic_bits(required(&mut object, "bits")?)?;
    let value = f64::from_bits(bits);
    let projection = required(&mut object, "projection")?;
    if !value.is_finite() {
        return Err(CodecError::Domain("finite scientific f64"));
    }
    let Value::Number(Number::Float(projected)) = projection else {
        return Err(CodecError::Domain("finite numeric projection"));
    };
    // The server may normalize signed zero in a projection; bits retain that distinction.
    if projected != value || !object.is_empty() {
        return Err(CodecError::Domain("consistent finite numeric projection"));
    }
    Ok(value)
}

#[cfg(test)]
mod canonical_codec_unit {
    use super::*;
    #[test]
    fn unsigned_native_boundaries_and_invalid_values() {
        for value in [
            0,
            (1u64 << 63) - 1,
            1u64 << 63,
            (1u64 << 63) + 1,
            u64::MAX - 1,
            u64::MAX,
        ] {
            assert_eq!(decode_uint(encode_uint(value).unwrap()).unwrap(), value);
        }
        for value in ["-1", "0.5", "18446744073709551616"] {
            let decimal: Decimal = value.parse().unwrap();
            assert!(decode_uint(Value::Number(Number::Decimal(decimal))).is_err());
        }
        assert!(decode_uint(Value::Number(Number::Int(1))).is_err());
    }
    #[test]
    fn finite_cells_and_raw_diagnostics_have_distinct_domains() {
        for value in [0.0f64, -0.0, f64::MIN_POSITIVE, f64::MAX, -17.25] {
            assert_eq!(
                decode_finite(encode_finite(value).unwrap())
                    .unwrap()
                    .to_bits(),
                value.to_bits()
            );
        }
        for bits in [
            f64::INFINITY.to_bits(),
            f64::NEG_INFINITY.to_bits(),
            0x7ff8_0000_0000_0042,
        ] {
            assert_eq!(
                decode_diagnostic_bits(encode_diagnostic_bits(bits)).unwrap(),
                bits
            );
            assert!(encode_finite(f64::from_bits(bits)).is_err());
        }
        assert!(decode_finite(Value::None).is_err());
        assert!(decode_finite(Value::Null).is_err());
        let mut value = encode_finite(4.0).unwrap();
        if let Value::Object(ref mut object) = value {
            object.insert("projection", 5.0);
        }
        assert!(decode_finite(value).is_err());
    }
}
