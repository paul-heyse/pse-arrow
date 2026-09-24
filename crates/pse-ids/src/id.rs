// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The identity types of blueprint §5.1: 128-bit semantic IDs, 256-bit content hashes,
//! the role newtypes that keep the three hash roles apart, and the two integer
//! newtypes that keep an artifact-local ordinal from being mistaken for an identity.
//!
//! `Display` is lowercase hexadecimal for both widths. `Debug` exists so a value can be
//! named in a message; it is never a hash input, because a `Debug` rendering is a
//! property of a formatter and three libraries in the dependency set leak hash-container
//! iteration order through one (§5.3 step 4 and §5.3's closing rule).

use std::fmt;

use crate::error::IdError;

/// Renders `bytes` as lowercase hexadecimal.
fn to_hex_string(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        // `write!` to a String cannot fail, and the crate denies `unwrap`.
        out.push(hex_digit(byte >> 4));
        out.push(hex_digit(byte & 0x0f));
    }
    out
}

/// The lowercase hexadecimal digit for a nibble; values above 15 cannot occur.
const fn hex_digit(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        10..=15 => (b'a' + (nibble - 10)) as char,
        // Unreachable for a nibble; a total function is cheaper than a panic policy escape.
        _ => '?',
    }
}

/// Parses exactly `WIDTH * 2` hexadecimal digits into `WIDTH` bytes.
fn parse_hex_bytes<const WIDTH: usize>(text: &str) -> Result<[u8; WIDTH], IdError> {
    let digits: Vec<char> = text.chars().collect();
    if digits.len() != WIDTH * 2 {
        return Err(IdError::HexLength {
            expected: WIDTH * 2,
            actual: digits.len(),
        });
    }
    let mut out = [0_u8; WIDTH];
    for (index, byte) in out.iter_mut().enumerate() {
        let high = nibble_of(digits[index * 2], index * 2)?;
        let low = nibble_of(digits[index * 2 + 1], index * 2 + 1)?;
        *byte = (high << 4) | low;
    }
    Ok(out)
}

/// One hexadecimal digit, upper or lower case, as a nibble.
fn nibble_of(found: char, position: usize) -> Result<u8, IdError> {
    found
        .to_digit(16)
        .and_then(|value| u8::try_from(value).ok())
        .ok_or(IdError::HexDigit { position, found })
}

/// A 128-bit semantic identity (blueprint §5.1, `pse.semantic_id`).
///
/// Assigned at authoring time (`UUIDv7` under the `explicit` policy, [`named_id`] under the
/// `named` policy) or derived by a pass from what the entity was created from. It survives
/// revisions, reordering, re-batching, projection and publication; it is never computed
/// from a name a rename could change, and never from a row position.
///
/// [`named_id`]: crate::derive::named_id
///
/// ```
/// use pse_ids::SemanticId;
///
/// let id = SemanticId::from_bytes([0xab; 16]);
/// assert_eq!(id.to_hex(), "abababababababababababababababab");
/// assert_eq!(SemanticId::parse_hex(&id.to_hex()), Ok(id));
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SemanticId([u8; SemanticId::WIDTH]);

impl SemanticId {
    /// Width in bytes.
    pub const WIDTH: usize = 16;

    /// The all-zero identity: the root package under which the registry names itself, and
    /// the only ID never assigned to an authored entity.
    pub const NIL: Self = Self([0_u8; Self::WIDTH]);

    /// Wraps raw bytes that are already an identity.
    pub const fn from_bytes(bytes: [u8; Self::WIDTH]) -> Self {
        Self(bytes)
    }

    /// The raw bytes, which are what the framing in [`crate::derive`] hashes.
    pub const fn as_bytes(&self) -> &[u8; Self::WIDTH] {
        &self.0
    }

    /// Reads an identity from a slice of exactly [`Self::WIDTH`] bytes.
    ///
    /// # Errors
    ///
    /// [`IdError::Length`] when the slice is not exactly [`Self::WIDTH`] bytes; a short
    /// slice is a truncated identity, never a zero-padded one.
    pub fn try_from_slice(bytes: &[u8]) -> Result<Self, IdError> {
        let sized: [u8; Self::WIDTH] = bytes.try_into().map_err(|_| IdError::Length {
            expected: Self::WIDTH,
            actual: bytes.len(),
        })?;
        Ok(Self(sized))
    }

    /// Lowercase hexadecimal, 32 digits.
    pub fn to_hex(self) -> String {
        to_hex_string(&self.0)
    }

    /// Reads an identity from 32 hexadecimal digits of either case.
    ///
    /// # Errors
    ///
    /// [`IdError::HexLength`] when the digit count is wrong and [`IdError::HexDigit`]
    /// when a character is not hexadecimal.
    pub fn parse_hex(text: &str) -> Result<Self, IdError> {
        parse_hex_bytes::<{ Self::WIDTH }>(text).map(Self)
    }
}

impl fmt::Display for SemanticId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

/// A 256-bit BLAKE3 content hash (blueprint §5.1, `pse.content_hash`).
///
/// Produced by [`crate::derive::derive_hash`] under a `derive_key` context, by
/// `pse.canon.v2` logical hashing, or by a plain hash over stored bytes. The role
/// newtypes [`LogicalHash`] and [`EncodingChecksum`] exist because
/// ADR-0045 separates those three roles and a bare `ContentHash` would let one stand in
/// for another.
///
/// ```
/// use pse_ids::ContentHash;
///
/// let hash = ContentHash::from_bytes([0x01; 32]);
/// assert!(hash.to_prefixed().starts_with("blake3:"));
/// assert_eq!(ContentHash::parse_prefixed(&hash.to_prefixed()), Ok(hash));
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ContentHash([u8; ContentHash::WIDTH]);

impl ContentHash {
    /// Width in bytes.
    pub const WIDTH: usize = 32;

    /// The algorithm prefix of the textual form; the algorithm is part of the value.
    pub const PREFIX: &'static str = "blake3:";

    /// The all-zero hash. Not a hash of anything; a placeholder a caller must replace.
    pub const NIL: Self = Self([0_u8; Self::WIDTH]);

    /// Wraps raw digest bytes.
    pub const fn from_bytes(bytes: [u8; Self::WIDTH]) -> Self {
        Self(bytes)
    }

    /// The raw digest bytes.
    pub const fn as_bytes(&self) -> &[u8; Self::WIDTH] {
        &self.0
    }

    /// Reads a hash from a slice of exactly [`Self::WIDTH`] bytes.
    ///
    /// # Errors
    ///
    /// [`IdError::Length`] when the slice is not exactly [`Self::WIDTH`] bytes.
    pub fn try_from_slice(bytes: &[u8]) -> Result<Self, IdError> {
        let sized: [u8; Self::WIDTH] = bytes.try_into().map_err(|_| IdError::Length {
            expected: Self::WIDTH,
            actual: bytes.len(),
        })?;
        Ok(Self(sized))
    }

    /// Lowercase hexadecimal, 64 digits, with no algorithm prefix.
    pub fn to_hex(self) -> String {
        to_hex_string(&self.0)
    }

    /// Reads a hash from 64 hexadecimal digits of either case.
    ///
    /// # Errors
    ///
    /// [`IdError::HexLength`] when the digit count is wrong and [`IdError::HexDigit`]
    /// when a character is not hexadecimal.
    pub fn parse_hex(text: &str) -> Result<Self, IdError> {
        parse_hex_bytes::<{ Self::WIDTH }>(text).map(Self)
    }

    /// `blake3:` followed by 64 lowercase hexadecimal digits: the form a manifest stores.
    pub fn to_prefixed(self) -> String {
        let mut out = String::with_capacity(Self::PREFIX.len() + Self::WIDTH * 2);
        out.push_str(Self::PREFIX);
        out.push_str(&self.to_hex());
        out
    }

    /// Reads the prefixed form written by [`Self::to_prefixed`].
    ///
    /// # Errors
    ///
    /// [`IdError::Prefix`] when the `blake3:` prefix is absent — an unprefixed digest may
    /// be another algorithm, and guessing is how a hash contract stops being one — plus
    /// the [`Self::parse_hex`] errors for the digits.
    pub fn parse_prefixed(text: &str) -> Result<Self, IdError> {
        let digits = text.strip_prefix(Self::PREFIX).ok_or(IdError::Prefix {
            expected: Self::PREFIX,
        })?;
        Self::parse_hex(digits)
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl serde::Serialize for SemanticId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> serde::Deserialize<'de> for SemanticId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = <String as serde::Deserialize>::deserialize(deserializer)?;
        Self::parse_hex(&text).map_err(serde::de::Error::custom)
    }
}

impl serde::Serialize for ContentHash {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_prefixed())
    }
}

impl<'de> serde::Deserialize<'de> for ContentHash {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = <String as serde::Deserialize>::deserialize(deserializer)?;
        Self::parse_prefixed(&text).map_err(serde::de::Error::custom)
    }
}

/// Declares a role newtype over [`ContentHash`] with a `Display` that delegates.
macro_rules! hash_role {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        pub struct $name(pub ContentHash);

        impl $name {
            /// The underlying digest, for framing and for storage.
            pub const fn content_hash(&self) -> ContentHash {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0, f)
            }
        }
    };
}

hash_role! {
    /// The identity of a relation's logical content under its contract
    /// (`logical_hash`, blueprint §5.3 step 6).
    ///
    /// Equal declared content hashes equally across row and batch layout, hidden null
    /// payload, dictionary encoding and supported file round trips. It is never a
    /// statement about any particular stored object.
    LogicalHash
}

hash_role! {
    /// The integrity check of one stored object's finished bytes
    /// (`encoding_checksum`, blueprint §5.3 step 6).
    ///
    /// Two encodings of the same logical relation have different checksums and the same
    /// [`LogicalHash`]; ADR-0045 refuses to let either substitute for the other.
    EncodingChecksum
}

/// The `@N` version of a relation's declared schema (blueprint §4.1).
///
/// A `u32` because that is the width the `pse.canon.v2` and `pse.snapshot.v2` frames
/// reserve for it (§5.3 steps 4 and 7).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct SchemaVersion(pub u32);

impl SchemaVersion {
    /// The version as four little-endian bytes, which is how it enters a hash frame.
    pub const fn to_le_bytes(self) -> [u8; 4] {
        self.0.to_le_bytes()
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nil_is_all_zero_and_renders_as_zeros() {
        assert_eq!(SemanticId::NIL.as_bytes(), &[0_u8; 16]);
        assert_eq!(SemanticId::NIL.to_hex(), "0".repeat(32));
    }

    #[test]
    fn semantic_id_hex_round_trips_every_byte_value() {
        let mut bytes = [0_u8; 16];
        for (index, byte) in bytes.iter_mut().enumerate() {
            *byte = u8::try_from(index * 17).unwrap_or(0);
        }
        let id = SemanticId::from_bytes(bytes);
        assert_eq!(SemanticId::parse_hex(&id.to_hex()), Ok(id));
        assert_eq!(id.to_string(), id.to_hex());
    }

    #[test]
    fn content_hash_hex_and_prefixed_round_trip() {
        let hash = ContentHash::from_bytes([0xfe; 32]);
        assert_eq!(hash.to_hex(), "fe".repeat(32));
        assert_eq!(ContentHash::parse_hex(&hash.to_hex()), Ok(hash));
        assert_eq!(hash.to_prefixed(), format!("blake3:{}", "fe".repeat(32)));
        assert_eq!(ContentHash::parse_prefixed(&hash.to_prefixed()), Ok(hash));
    }

    #[test]
    fn uppercase_hex_parses_and_renders_lowercase() {
        let parsed = SemanticId::parse_hex("ABCDEF01234567890000000000000000");
        assert_eq!(
            parsed.map(SemanticId::to_hex),
            Ok("abcdef01234567890000000000000000".to_owned())
        );
    }

    #[test]
    fn try_from_slice_reports_the_offered_length() {
        assert_eq!(
            SemanticId::try_from_slice(&[0_u8; 15]),
            Err(IdError::Length {
                expected: 16,
                actual: 15
            })
        );
        assert_eq!(
            SemanticId::try_from_slice(&[0_u8; 17]),
            Err(IdError::Length {
                expected: 16,
                actual: 17
            })
        );
        assert_eq!(
            ContentHash::try_from_slice(&[0_u8; 16]),
            Err(IdError::Length {
                expected: 32,
                actual: 16
            })
        );
        assert_eq!(
            SemanticId::try_from_slice(&[7_u8; 16]).map(|id| *id.as_bytes()),
            Ok([7_u8; 16])
        );
    }

    #[test]
    fn hex_parsing_reports_length_and_digit_faults() {
        assert_eq!(
            SemanticId::parse_hex("abcd"),
            Err(IdError::HexLength {
                expected: 32,
                actual: 4
            })
        );
        let short = "0".repeat(31);
        assert_eq!(
            SemanticId::parse_hex(&short),
            Err(IdError::HexLength {
                expected: 32,
                actual: 31
            })
        );
        let leading_z = format!("z{}", "0".repeat(31));
        assert_eq!(
            SemanticId::parse_hex(&leading_z),
            Err(IdError::HexDigit {
                position: 0,
                found: 'z'
            })
        );
        let trailing_z = format!("{}z", "0".repeat(31));
        assert_eq!(
            SemanticId::parse_hex(&trailing_z),
            Err(IdError::HexDigit {
                position: 31,
                found: 'z'
            })
        );
    }

    #[test]
    fn a_missing_prefix_is_refused_rather_than_guessed() {
        assert_eq!(
            ContentHash::parse_prefixed(&"ab".repeat(32)),
            Err(IdError::Prefix {
                expected: "blake3:"
            })
        );
    }

    #[test]
    fn role_newtypes_expose_their_digest_without_being_interchangeable() {
        let digest = ContentHash::from_bytes([0x5a; 32]);
        assert_eq!(LogicalHash(digest).content_hash(), digest);
        assert_eq!(EncodingChecksum(digest).content_hash(), digest);
    }

    #[test]
    fn ordinal_and_schema_version_frame_little_endian() {
        assert_eq!(SchemaVersion(1).to_le_bytes(), [1, 0, 0, 0]);
        assert_eq!(SchemaVersion(3).to_string(), "3");
    }

    #[test]
    fn ids_order_by_raw_bytes() {
        let low = SemanticId::from_bytes([0x00; 16]);
        let high = SemanticId::from_bytes([0x01; 16]);
        assert!(low < high);
        let mut sorted = vec![high, low];
        sorted.sort_unstable();
        assert_eq!(sorted, vec![low, high]);
    }
}
