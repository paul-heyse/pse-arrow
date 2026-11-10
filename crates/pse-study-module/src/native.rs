// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Direct native value support for source-generated semantic adapters.
use std::collections::{BTreeMap, BTreeSet};
use surrealdb_types::{Number, Object, Value};

/// The supplied value cannot represent the exact source-owned product.
#[derive(Clone, Copy, Debug, thiserror::Error, miette::Diagnostic)]
#[error("invalid native study product")]
#[diagnostic(code(validation::invariant))]
pub struct NativeError;

/// Exact unsigned carrier; unlike the SDK primitive it emits native U64.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Unsigned(pub u64);
impl surrealdb_types::SurrealValue for Unsigned {
    fn kind_of() -> surrealdb_types::Kind {
        surrealdb_types::Kind::U64
    }
    fn into_value(self) -> Value {
        Value::Number(Number::U64(self.0))
    }
    fn from_value(value: Value) -> Result<Self, surrealdb_types::Error> {
        if !matches!(
            &value,
            Value::Number(Number::U64(_) | Number::Int(_) | Number::Decimal(_))
        ) {
            return Err(surrealdb_types::conversion_error(
                surrealdb_types::Kind::U64,
                Value::None,
            ));
        }
        u64::from_native(value)
            .map(Self)
            .map_err(|_| surrealdb_types::conversion_error(surrealdb_types::Kind::U64, Value::None))
    }
}

/// Direct conversion implemented by the semantic-owner generated projection.
pub trait NativeValue: Sized {
    /// Consume a typed semantic product without serializing a document.
    fn into_native(self) -> Result<Value, NativeError>;
    /// Construct a typed product with its original owner constraints.
    fn from_native(value: Value) -> Result<Self, NativeError>;
}
/// Refuse a stale generated tuple mapping, including reorders of equal Rust types.
pub fn check_layout(
    actual_version: u32,
    actual: &[(u32, &str)],
    expected_version: u32,
    expected: &[(u32, &str)],
) -> Result<(), NativeError> {
    if actual_version != expected_version || actual != expected {
        return Err(NativeError);
    }
    Ok(())
}
pub(crate) fn required(object: &mut Object, name: &str) -> Result<Value, NativeError> {
    object.remove(name).ok_or(NativeError)
}
pub(crate) fn exact_object(value: Value, names: &[&str]) -> Result<Object, NativeError> {
    let Value::Object(object) = value else {
        return Err(NativeError);
    };
    if object.keys().any(|name| !names.contains(&name.as_str())) {
        return Err(NativeError);
    }
    Ok(object)
}
pub(crate) fn bytes(value: Value) -> Result<Vec<u8>, NativeError> {
    match value {
        Value::Bytes(bytes) => Ok(bytes.to_vec()),
        _ => Err(NativeError),
    }
}
pub(crate) fn array(value: Value) -> Result<Vec<Value>, NativeError> {
    match value {
        Value::Array(array) => Ok(array.into_inner()),
        _ => Err(NativeError),
    }
}
pub(crate) fn map<K: Ord, V>(entries: Vec<(K, V)>) -> Result<BTreeMap<K, V>, NativeError> {
    let mut output = BTreeMap::new();
    for (key, value) in entries {
        if output.insert(key, value).is_some() {
            return Err(NativeError);
        }
    }
    Ok(output)
}
/// Adopt a declared set without silently dropping duplicate input members.
pub fn set<T: Ord>(entries: Vec<T>) -> Result<BTreeSet<T>, NativeError> {
    let mut output = BTreeSet::new();
    for value in entries {
        if !output.insert(value) {
            return Err(NativeError);
        }
    }
    Ok(output)
}
impl NativeValue for bool {
    fn into_native(self) -> Result<Value, NativeError> {
        Ok(Value::Bool(self))
    }
    fn from_native(value: Value) -> Result<Self, NativeError> {
        match value {
            Value::Bool(value) => Ok(value),
            _ => Err(NativeError),
        }
    }
}
impl NativeValue for String {
    fn into_native(self) -> Result<Value, NativeError> {
        Ok(Value::String(self))
    }
    fn from_native(value: Value) -> Result<Self, NativeError> {
        match value {
            Value::String(value) => Ok(value),
            _ => Err(NativeError),
        }
    }
}
impl<T: NativeValue> NativeValue for Box<T> {
    fn into_native(self) -> Result<Value, NativeError> {
        (*self).into_native()
    }
    fn from_native(value: Value) -> Result<Self, NativeError> {
        T::from_native(value).map(Box::new)
    }
}
impl NativeValue for u64 {
    fn into_native(self) -> Result<Value, NativeError> {
        Ok(Value::Number(Number::U64(self)))
    }
    fn from_native(value: Value) -> Result<Self, NativeError> {
        match value {
            Value::Number(Number::U64(value)) => Ok(value),
            Value::Number(Number::Int(value)) => value.try_into().map_err(|_| NativeError),
            Value::Number(Number::Decimal(value))
                if value.fract().is_zero() && value >= 0.into() && value <= u64::MAX.into() =>
            {
                value
                    .normalize()
                    .to_string()
                    .parse()
                    .map_err(|_| NativeError)
            }
            _ => Err(NativeError),
        }
    }
}
macro_rules! unsigned { ($($ty:ty),*)=> {$(impl NativeValue for $ty {
    fn into_native(self)->Result<Value,NativeError> { u64::from(self).into_native() }
    fn from_native(value:Value)->Result<Self,NativeError> { u64::from_native(value)?.try_into().map_err(|_|NativeError) }
})*}; }
unsigned!(u8, u16, u32);
macro_rules! signed { ($($ty:ty),*)=> {$(impl NativeValue for $ty {
    fn into_native(self)->Result<Value,NativeError> { Ok(Value::Number(Number::Int(i64::from(self)))) }
    fn from_native(value:Value)->Result<Self,NativeError> { match value {Value::Number(Number::Int(value))=>value.try_into().map_err(|_|NativeError),_=>Err(NativeError)} }
})*}; }
signed!(i8, i16, i32, i64);
impl NativeValue for i128 {
    fn into_native(self) -> Result<Value, NativeError> {
        Ok(Value::Bytes(self.to_le_bytes().to_vec().into()))
    }
    fn from_native(value: Value) -> Result<Self, NativeError> {
        Ok(Self::from_le_bytes(
            bytes(value)?.try_into().map_err(|_| NativeError)?,
        ))
    }
}
pub(crate) trait Bits64: Sized {
    fn into_bits(self) -> Value;
    fn from_bits(value: Value) -> Result<Self, NativeError>;
}
/// Source-declared binary32 bits, independent of floating conversion.
pub trait Bits32: Sized {
    /// Preserve the original four IEEE bytes.
    fn into_bits(self) -> Value;
    /// Recover exactly four IEEE bytes.
    fn from_bits(value: Value) -> Result<Self, NativeError>;
}
impl Bits64 for u64 {
    fn into_bits(self) -> Value {
        Value::Bytes(self.to_le_bytes().to_vec().into())
    }
    fn from_bits(value: Value) -> Result<Self, NativeError> {
        Ok(Self::from_le_bytes(
            bytes(value)?.try_into().map_err(|_| NativeError)?,
        ))
    }
}
impl Bits64 for f64 {
    fn into_bits(self) -> Value {
        self.to_bits().into_bits()
    }
    fn from_bits(value: Value) -> Result<Self, NativeError> {
        Ok(Self::from_bits(<u64 as Bits64>::from_bits(value)?))
    }
}
impl Bits32 for u32 {
    fn into_bits(self) -> Value {
        Value::Bytes(self.to_le_bytes().to_vec().into())
    }
    fn from_bits(value: Value) -> Result<Self, NativeError> {
        Ok(Self::from_le_bytes(
            bytes(value)?.try_into().map_err(|_| NativeError)?,
        ))
    }
}
impl Bits32 for f32 {
    fn into_bits(self) -> Value {
        self.to_bits().into_bits()
    }
    fn from_bits(value: Value) -> Result<Self, NativeError> {
        Ok(Self::from_bits(<u32 as Bits32>::from_bits(value)?))
    }
}
impl NativeValue for f64 {
    fn into_native(self) -> Result<Value, NativeError> {
        Ok(Value::Bytes(self.to_bits().to_le_bytes().to_vec().into()))
    }
    fn from_native(value: Value) -> Result<Self, NativeError> {
        let bits: [u8; 8] = bytes(value)?.try_into().map_err(|_| NativeError)?;
        Ok(Self::from_bits(u64::from_le_bytes(bits)))
    }
}
impl NativeValue for f32 {
    fn into_native(self) -> Result<Value, NativeError> {
        Ok(Value::Bytes(self.to_bits().to_le_bytes().to_vec().into()))
    }
    fn from_native(value: Value) -> Result<Self, NativeError> {
        let bits: [u8; 4] = bytes(value)?.try_into().map_err(|_| NativeError)?;
        Ok(Self::from_bits(u32::from_le_bytes(bits)))
    }
}

/// Reject excessive native trees before constructing nested owned semantic products.
pub fn admit(value: &Value, maximum_bytes: usize, maximum_depth: usize) -> Result<(), NativeError> {
    fn visit(value: &Value, budget: &mut usize, depth: usize) -> Result<(), NativeError> {
        if depth == 0 {
            return Err(NativeError);
        }
        let payload = match value {
            Value::String(value) => value.len(),
            Value::Bytes(value) => value.len(),
            Value::Number(Number::BigDecimal(_)) => return Err(NativeError),
            Value::None
            | Value::Null
            | Value::Bool(_)
            | Value::Number(_)
            | Value::Array(_)
            | Value::Object(_) => 0,
            _ => return Err(NativeError),
        };
        let cost = 16usize.checked_add(payload).ok_or(NativeError)?;
        *budget = budget.checked_sub(cost).ok_or(NativeError)?;
        match value {
            Value::Object(object) => {
                for (key, value) in object.iter() {
                    *budget = budget.checked_sub(key.len()).ok_or(NativeError)?;
                    visit(value, budget, depth - 1)?;
                }
            }
            Value::Array(array) => {
                for value in array.iter() {
                    visit(value, budget, depth - 1)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    let mut budget = maximum_bytes;
    visit(value, &mut budget, maximum_depth)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_unsigned_values_preserve_width_and_checked_wire_widening() {
        for value in [0, i64::MAX as u64, i64::MAX as u64 + 1, u64::MAX] {
            let native = value.into_native().unwrap();
            assert!(matches!(native, Value::Number(Number::U64(_))));
            assert_eq!(u64::from_native(native).unwrap(), value);
            assert_eq!(
                u64::from_native(Value::Number(Number::Decimal(value.into()))).unwrap(),
                value
            );
        }
        assert!(u64::from_native(Value::Number(Number::Int(-1))).is_err());
        assert!(u64::from_native(Value::Number(Number::Float(1.0))).is_err());
    }
    #[test]
    fn native_tree_admission_refuses_aggregate_and_depth_before_decoding() {
        let value =
            Value::Array(vec![Value::String("x".repeat(64)), Value::String("y".repeat(64))].into());
        assert!(admit(&value, 128, 8).is_err());
        assert!(admit(&value, 256, 1).is_err());
        assert!(admit(&value, 256, 2).is_ok());
    }
    #[test]
    fn empty_native_leaves_still_consume_aggregate_admission() {
        for leaf in [
            Value::String(String::new()),
            Value::Bytes(Vec::new().into()),
        ] {
            let value = Value::Array(vec![leaf; 65537].into());
            assert!(admit(&value, 1024 * 1024, 128).is_err());
            assert!(<Unsigned as surrealdb_types::SurrealValue>::from_value(value).is_err());
        }
        let arbitrary_precision = Value::Number(Number::BigDecimal(Box::new("1".parse().unwrap())));
        assert!(admit(&arbitrary_precision, 1024 * 1024, 128).is_err());
    }
    #[test]
    fn stale_native_tuple_mapping_refuses_even_when_field_types_could_still_compile() {
        assert!(
            check_layout(
                1,
                &[(2, "seed_permission"), (1, "usable")],
                1,
                &[(1, "usable"), (2, "seed_permission")]
            )
            .is_err()
        );
        assert!(check_layout(2, &[(1, "usable")], 1, &[(1, "usable")]).is_err());
    }
}
