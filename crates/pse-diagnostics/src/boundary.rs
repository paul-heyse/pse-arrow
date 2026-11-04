// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source callbacks expose original diagnostic data without a model dependency.

/// Actual field types stay at their defining owner, independent of a downstream schema.
pub trait BoundaryField<const TAG: u32> {
    /// Actual native field type.
    type Type;
    /// Borrow original data without copying its declaration.
    fn boundary_field(&self) -> &Self::Type;
}
/// Original declaration provenance, retained through a downstream callback expansion.
pub trait BoundaryDeclaration {
    /// Original owner source file.
    const SOURCE: &'static str;
}

/// Closed string vocabulary metadata emitted by its actual spelling declaration.
pub trait BoundaryVocabulary: BoundaryDeclaration {
    /// Actual source vocabulary name.
    const NAME: &'static str;
    /// Checked public alias, relative to this native owner's crate.
    const PUBLIC_PATH: &'static str;
    /// Canonical member spellings from the original `ALL` and `as_str` contracts.
    fn boundary_members() -> Vec<&'static str>;
}

/// Declare a plain diagnostic record and its typed field callback together. The
/// selected records expose public data, so reconstruction grants no performing authority.
#[macro_export]
macro_rules! diagnostic_boundary_record {
    (callback: $callback:ident; owner: $($public:ident)::+; name: $wire:literal; version: $version:literal;
     direction: $direction:ident; reserved_tags: [$($reserved:literal),* $(,)?]; reserved_names: [$($reserved_name:literal),* $(,)?];
     $(#[$meta:meta])* $vis:vis struct $name:ident {
        $($(#[$field_meta:meta])* pub $field:ident: $ty:ty => $tag:literal;)*
     }) => {
        $(#[$meta])* $vis struct $name { $($(#[$field_meta])* pub $field: $ty,)* }
        impl $crate::boundary::BoundaryDeclaration for $name { const SOURCE: &'static str = file!(); }
        $(impl $crate::boundary::BoundaryField<$tag> for $name {
            type Type = $ty;
            fn boundary_field(&self) -> &Self::Type { &self.$field }
        })*
        const _: fn($name) -> $crate::$($public)::+ = |value| value;
        #[doc = concat!("Visit the source-owned `", stringify!($name), "` diagnostic declaration.")]
        #[macro_export]
        macro_rules! $callback {
            ($consumer:ident) => { $consumer! {
                $crate::$($public)::+; concat!("crate::", stringify!($($public)::+)); $wire; $version; $direction; [$($reserved),*]; [$($reserved_name),*];
                $($field => $tag;)*
            } };
        }
    };
}

/// Declare an actual unit vocabulary and its stable alternative callback. Direction
/// stays a source token because the shared vocabulary itself depends on diagnostics.
#[macro_export]
macro_rules! diagnostic_boundary_enum {
    (callback: $callback:ident; owner: $($public:ident)::+; name: $wire:literal; version: $version:literal;
     direction: $direction:ident; reserved_tags: [$($reserved:literal),* $(,)?]; reserved_names: [$($reserved_name:literal),* $(,)?];
     $(#[$meta:meta])* $vis:vis enum $name:ident {
        $($(#[$variant_meta:meta])* $variant:ident => $tag:literal;)*
     }) => {
        $(#[$meta])* $vis enum $name { $($(#[$variant_meta])* $variant,)* }
        impl $crate::boundary::BoundaryDeclaration for $name { const SOURCE: &'static str = file!(); }
        const _: fn($name) -> $crate::$($public)::+ = |value| value;
        #[doc = concat!("Visit the source-owned `", stringify!($name), "` diagnostic alternatives.")]
        #[macro_export]
        macro_rules! $callback {
            ($consumer:ident) => { $consumer! {
                $crate::$($public)::+; concat!("crate::", stringify!($($public)::+)); $wire; $version; $direction; [$($reserved),*]; [$($reserved_name),*];
                $($variant => $tag;)*
            } };
        }
    };
}
