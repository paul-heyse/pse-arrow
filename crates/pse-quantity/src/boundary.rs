// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-owned typed access for downstream boundary projections, without a wire dependency.

/// Shared boundary direction without depending on any model or protobuf implementation.
pub use pse_contract::BoundaryDirection as Direction;

mod passive;
pub use passive::{
    BoundaryAlternative, BoundaryAlternativeField, BoundaryAlternatives, BoundaryDocument,
    BoundaryFieldEncoding, PassiveFieldEncoding,
};

/// Actual record fields and checked reconstruction at their defining owner.
pub trait BoundaryRecord: Sized {
    /// Explicit interpretation name.
    const NAME: &'static str;
    /// Nonzero interpretation version.
    const VERSION: u32;
    /// Original declaration location.
    const SOURCE: &'static str;
    /// Source-declared ingress/egress scope.
    const DIRECTION: Direction;
    /// Removed stable tags that cannot be reassigned.
    const RESERVED_TAGS: &'static [u32];
    /// Removed public field names that cannot be reused.
    const RESERVED_NAMES: &'static [&'static str];
    /// Actual field tuple, rather than a generic document representation.
    type Fields;
    /// Consume the original declared fields.
    fn into_boundary_fields(self) -> Self::Fields;
    /// Reconstruct through the original owner's invariants.
    /// # Errors
    /// Malformed canonical rational components refuse rather than normalize.
    fn from_boundary_fields(fields: Self::Fields) -> Result<Self, crate::DimensionError>;
}

/// An actual field selected by its source-declared stable tag.
pub trait BoundaryField<const TAG: u32> {
    /// The actual Rust field type.
    type Type;
    /// Borrow the actual field without copying its declaration.
    fn boundary_field(&self) -> &Self::Type;
}

/// Declare an ordinary native record and export its tagged field callback together.
/// The callback carries names/tags only; downstream consumers resolve field types from
/// [`BoundaryField`], so this does not introduce a second physical schema.
#[macro_export]
macro_rules! quantity_boundary_record {
    (callback: $callback:ident; name: $wire:literal; version: $version:literal;
     direction: $direction:ident; reserved_tags: [$($reserved:literal),* $(,)?]; reserved_names: [$($reserved_name:literal),* $(,)?];
     $(#[$meta:meta])* $vis:vis struct $name:ident ($ty:ty) => $field:ident : $tag:literal;) => {
        $(#[$meta])* $vis struct $name($ty);
        impl $crate::boundary::BoundaryRecord for $name {
            const NAME: &'static str = $wire;
            const VERSION: u32 = $version;
            const SOURCE: &'static str = file!();
            const DIRECTION: $crate::boundary::Direction = $crate::boundary::Direction::$direction;
            const RESERVED_TAGS: &'static [u32] = &[$($reserved),*];
            const RESERVED_NAMES: &'static [&'static str] = &[$($reserved_name),*];
            type Fields = ($ty,);
            fn into_boundary_fields(self) -> Self::Fields { (self.0,) }
            fn from_boundary_fields((value,): Self::Fields) -> Result<Self, $crate::DimensionError> { Ok(Self(value)) }
        }
        impl $crate::boundary::BoundaryField<$tag> for $name {
            type Type = $ty;
            fn boundary_field(&self) -> &Self::Type { &self.0 }
        }
        #[doc = concat!("Visit the source-owned `", stringify!($name), "` boundary declaration.")]
        #[macro_export]
        macro_rules! $callback { ($consumer:ident) => { $consumer! { $crate::$name; $field => $tag; } }; }
    };
    (callback: $callback:ident; name: $wire:literal; version: $version:literal;
     direction: $direction:ident; reserved_tags: [$($reserved:literal),* $(,)?]; reserved_names: [$($reserved_name:literal),* $(,)?];
     construct: [$($construct:expr)?];
     $(#[$meta:meta])* $vis:vis struct $name:ident {
        $($(#[$field_meta:meta])* $field_vis:vis $field:ident: $ty:ty => $tag:literal;)*
     }) => {
        $(#[$meta])* $vis struct $name { $($(#[$field_meta])* $field_vis $field: $ty,)* }
        impl $crate::boundary::BoundaryRecord for $name {
            const NAME: &'static str = $wire;
            const VERSION: u32 = $version;
            const SOURCE: &'static str = file!();
            const DIRECTION: $crate::boundary::Direction = $crate::boundary::Direction::$direction;
            const RESERVED_TAGS: &'static [u32] = &[$($reserved),*];
            const RESERVED_NAMES: &'static [&'static str] = &[$($reserved_name),*];
            type Fields = ($($ty,)*);
            fn into_boundary_fields(self) -> Self::Fields { ($(self.$field,)*) }
            fn from_boundary_fields(($($field,)*): Self::Fields) -> Result<Self, $crate::DimensionError> {
                $crate::quantity_boundary_record!(@construct $name; ($($field),*); $($construct)?)
            }
        }
        $(impl $crate::boundary::BoundaryField<$tag> for $name {
            type Type = $ty;
            fn boundary_field(&self) -> &Self::Type { &self.$field }
        })*
        #[doc = concat!("Visit the source-owned `", stringify!($name), "` boundary declaration.")]
        #[macro_export]
        macro_rules! $callback {
            ($consumer:ident) => { $consumer! { $crate::$name; $($field => $tag;)* } };
        }
    };
    (@construct $name:ident; ($($field:ident),*); $construct:expr) => { ($construct)($($field),*) };
    (@construct $name:ident; ($($field:ident),*);) => { Ok($name { $($field,)* }) };
}
