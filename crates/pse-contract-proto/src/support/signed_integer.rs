// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact signed-integer representation, independent of scientific membership profiles.
use super::{NativeFrom, NativeInto, refused};
use pse_contract::boundary_projection::{AdapterError, AdapterFailure};

impl NativeInto<i32> for i8 {
    fn into_wire(self, _: &str) -> Result<i32, AdapterError> {
        Ok(i32::from(self))
    }
}
impl NativeFrom<i32> for i8 {
    fn from_wire(wire: i32, path: &str) -> Result<Self, AdapterError> {
        Self::try_from(wire).map_err(|_| {
            refused(
                path,
                AdapterFailure::InvalidValue,
                "sint32 does not fit the actual signed octet",
            )
        })
    }
}

impl NativeInto<Vec<u8>> for i128 {
    fn into_wire(self, _: &str) -> Result<Vec<u8>, AdapterError> {
        Ok(self.to_le_bytes().to_vec())
    }
}
impl NativeInto<Vec<u8>> for &i128 {
    fn into_wire(self, path: &str) -> Result<Vec<u8>, AdapterError> {
        (*self).into_wire(path)
    }
}
impl NativeFrom<&[u8]> for i128 {
    fn from_wire(wire: &[u8], path: &str) -> Result<Self, AdapterError> {
        let bytes = <[u8; 16]>::try_from(wire).map_err(|_| {
            refused(
                path,
                AdapterFailure::InvalidValue,
                "signed i128 representation requires exactly sixteen little-endian bytes",
            )
        })?;
        Ok(Self::from_le_bytes(bytes))
    }
}
impl NativeFrom<Vec<u8>> for i128 {
    fn from_wire(wire: Vec<u8>, path: &str) -> Result<Self, AdapterError> {
        Self::from_wire(wire.as_slice(), path)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{native_from, native_into};
    use super::*;
    #[test]
    fn signed_i128_exact_little_endian_owned_and_borrowed_boundaries() {
        for value in [
            i128::MIN,
            i128::MIN + 1,
            -(1_i128 << 64),
            i128::from(i64::MIN) - 1,
            i128::from(i64::MIN),
            -1,
            0,
            1,
            i128::from(i64::MAX),
            i128::from(i64::MAX) + 1,
            i128::from(u64::MAX),
            i128::from(u64::MAX) + 1,
            i128::MAX - 1,
            i128::MAX,
        ] {
            let bytes = native_into::<i128, Vec<u8>>(value, "integer").unwrap();
            assert_eq!(bytes, value.to_le_bytes());
            assert_eq!(
                native_into::<&i128, Vec<u8>>(&value, "integer").unwrap(),
                bytes
            );
            assert_eq!(
                native_from::<i128, &[u8]>(&bytes, "integer").unwrap(),
                value
            );
            assert_eq!(
                native_from::<i128, Vec<u8>>(bytes, "integer").unwrap(),
                value
            );
        }
        assert_eq!(
            native_from::<i128, &[u8]>(&[255; 16], "integer").unwrap(),
            -1
        );
        let mut minimum = [0; 16];
        minimum[15] = 128;
        assert_eq!(
            native_from::<i128, &[u8]>(&minimum, "integer").unwrap(),
            i128::MIN
        );
        let mut pattern = [0; 16];
        pattern[0] = 1;
        pattern[8] = 1;
        assert_eq!(
            native_from::<i128, &[u8]>(&pattern, "integer").unwrap(),
            (1_i128 << 64) + 1
        );
    }
    #[test]
    fn signed_i128_refuses_short_and_long_representations_without_truncation() {
        for length in [0, 1, 8, 15, 17, 32] {
            let bytes = vec![255; length];
            for error in [
                native_from::<i128, &[u8]>(&bytes, "number.exact_integer").unwrap_err(),
                native_from::<i128, Vec<u8>>(bytes, "number.exact_integer").unwrap_err(),
            ] {
                assert!(
                    matches!(error,AdapterError::Refused{path,kind:AdapterFailure::InvalidValue,..} if path=="number.exact_integer")
                );
            }
        }
    }
    #[test]
    fn signed_octet_uses_checked_sint32_adoption() {
        for value in [i8::MIN, -1, 0, 1, i8::MAX] {
            assert_eq!(
                native_from::<i8, i32>(i32::from(value), "factor").unwrap(),
                value
            );
            assert_eq!(
                native_into::<i8, i32>(value, "factor").unwrap(),
                i32::from(value)
            );
        }
        for value in [i32::MIN, -129, 128, i32::MAX] {
            assert!(
                matches!(native_from::<i8,i32>(value,"operation.factor"),Err(AdapterError::Refused {kind:AdapterFailure::InvalidValue,path,..}) if path=="operation.factor")
            );
        }
    }
}
