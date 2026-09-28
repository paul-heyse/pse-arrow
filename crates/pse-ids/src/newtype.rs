// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed identities: transparent wrappers over [`SemanticId`] or [`ContentHash`] that make
//! the compiler refuse a substitution the design refuses (blueprint §5.1, ADR-0115).
//!
//! A `UnitId` where a `QuantityTypeId` belongs, or an attempt identity where a run identity
//! belongs, produces a plausible-looking lookup miss rather than a type error when both are
//! bare identities. A typed id is its base value and nothing else at runtime: its bytes,
//! its serde form, its `Display` and its hash framing are the base value's, so wrapping an
//! identity never changes what is stored or hashed.
//!
//! [`SemanticId`]: crate::SemanticId
//! [`ContentHash`]: crate::ContentHash

/// Declares typed identities over [`SemanticId`] (the default) or [`ContentHash`]
/// (`Name: ContentHash`).
///
/// Each generated type derives `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`,
/// `Hash` and `Debug`; carries `from_id`/`as_id`, `const` `from_bytes`/`as_bytes` and the
/// two `From` conversions; renders the base value through `Display`; and serializes
/// exactly as its base value does.
///
/// ```
/// pse_ids::semantic_id_newtype! {
///     /// Identifies a widget.
///     WidgetId,
///     /// A content-addressed widget bundle.
///     BundleId: ContentHash,
/// }
///
/// let id = WidgetId::from_id(pse_ids::SemanticId::from_bytes([7; 16]));
/// assert_eq!(id.as_id().to_hex(), "07070707070707070707070707070707");
/// assert_eq!(BundleId::from_bytes([1; 32]).as_bytes(), &[1; 32]);
/// ```
///
/// [`SemanticId`]: crate::SemanticId
/// [`ContentHash`]: crate::ContentHash
#[macro_export]
macro_rules! semantic_id_newtype {
    () => {};
    (
        $(#[$meta:meta])*
        $name:ident : ContentHash
        $(, $($rest:tt)*)?
    ) => {
        $crate::__typed_identity! { $(#[$meta])* $name, $crate::ContentHash }
        $($crate::semantic_id_newtype! { $($rest)* })?
    };
    (
        $(#[$meta:meta])*
        $name:ident
        $(, $($rest:tt)*)?
    ) => {
        $crate::__typed_identity! { $(#[$meta])* $name, $crate::SemanticId }
        $($crate::semantic_id_newtype! { $($rest)* })?
    };
}

/// One typed identity over `$base`; the implementation of [`semantic_id_newtype!`].
#[doc(hidden)]
#[macro_export]
macro_rules! __typed_identity {
    ($(#[$meta:meta])* $name:ident, $base:path) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        pub struct $name($base);

        impl $name {
            #[doc = concat!("Wraps the base identity as a [`", stringify!($name), "`].")]
            pub const fn from_id(id: $base) -> Self {
                Self(id)
            }

            #[doc = concat!("The base identity inside this [`", stringify!($name), "`].")]
            pub const fn as_id(self) -> $base {
                self.0
            }

            #[doc = concat!("Wraps raw identity bytes as a [`", stringify!($name), "`].")]
            pub const fn from_bytes(bytes: [u8; <$base>::WIDTH]) -> Self {
                Self(<$base>::from_bytes(bytes))
            }

            /// The raw identity bytes, which are what storage and framing see.
            pub const fn as_bytes(&self) -> &[u8; <$base>::WIDTH] {
                self.0.as_bytes()
            }
        }

        impl ::core::convert::From<$base> for $name {
            fn from(id: $base) -> Self {
                Self(id)
            }
        }

        impl ::core::convert::From<$name> for $base {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl ::core::fmt::Display for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Display::fmt(&self.0, f)
            }
        }

        impl $crate::__serde::Serialize for $name {
            fn serialize<S: $crate::__serde::Serializer>(
                &self,
                serializer: S,
            ) -> ::core::result::Result<S::Ok, S::Error> {
                $crate::__serde::Serialize::serialize(&self.0, serializer)
            }
        }

        impl<'de> $crate::__serde::Deserialize<'de> for $name {
            fn deserialize<D: $crate::__serde::Deserializer<'de>>(
                deserializer: D,
            ) -> ::core::result::Result<Self, D::Error> {
                <$base as $crate::__serde::Deserialize<'de>>::deserialize(deserializer).map(Self)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::{ContentHash, SemanticId};

    crate::semantic_id_newtype! {
        /// A unit, for the tests.
        UnitId,
        /// A quantity type, for the tests.
        QuantityTypeId,
        /// A content-addressed bundle, for the tests.
        BundleId: ContentHash
    }

    #[test]
    fn newtype_round_trips_through_its_identity() {
        let raw = SemanticId::from_bytes([0x11; 16]);
        let unit = UnitId::from_id(raw);
        assert_eq!(unit.as_id(), raw);
        assert_eq!(SemanticId::from(unit), raw);
        assert_eq!(UnitId::from(raw), unit);
        assert_eq!(UnitId::from_bytes([0x11; 16]), unit);
        assert_eq!(unit.as_bytes(), raw.as_bytes());
    }

    #[test]
    fn newtype_display_renders_the_underlying_hexadecimal() {
        let id = QuantityTypeId::from_id(SemanticId::from_bytes([0xab; 16]));
        assert_eq!(id.to_string(), "abababababababababababababababab");
        let bundle = BundleId::from_bytes([0xcd; 32]);
        assert_eq!(bundle.to_string(), "cd".repeat(32));
    }

    /// Ordering follows the identity bytes, so a `BTreeMap` keyed by one of these is
    /// deterministic without a separate comparator.
    #[test]
    fn newtype_ordering_follows_the_identity_bytes() {
        let low = UnitId::from_id(SemanticId::from_bytes([0x00; 16]));
        let high = UnitId::from_id(SemanticId::from_bytes([0x01; 16]));
        assert!(low < high);
    }

    /// Wrapping never changes the stored form: serde delegates to the base value.
    #[test]
    fn newtype_serializes_exactly_as_its_base() {
        let raw = SemanticId::from_bytes([0x5a; 16]);
        let typed = serde_json::to_string(&UnitId::from_id(raw)).unwrap();
        assert_eq!(typed, serde_json::to_string(&raw).unwrap());
        assert_eq!(
            serde_json::from_str::<UnitId>(&typed).unwrap(),
            UnitId::from_id(raw)
        );
        let hash = ContentHash::from_bytes([0x3c; 32]);
        let typed = serde_json::to_string(&BundleId::from_id(hash)).unwrap();
        assert_eq!(typed, serde_json::to_string(&hash).unwrap());
        assert_eq!(
            serde_json::from_str::<BundleId>(&typed).unwrap().as_id(),
            hash
        );
        assert!(serde_json::from_str::<BundleId>("\"not a hash\"").is_err());
    }

    #[test]
    fn newtype_bytes_are_const() {
        const ID: UnitId = UnitId::from_bytes([3; 16]);
        const FIRST: u8 = ID.as_bytes()[0];
        assert_eq!(FIRST, 3);
        assert_eq!(ID.as_bytes(), &[3; 16]);
    }
}
