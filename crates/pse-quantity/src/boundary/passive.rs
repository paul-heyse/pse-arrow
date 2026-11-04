// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Neutral facets for existing passive scientific documents.

/// Existing native document representation, separate from admitted physical authority.
pub trait BoundaryDocument: Sized {
    /// Stable native document-view interpretation name.
    const NAME: &'static str;
    /// Actual source owner location.
    const SOURCE: &'static str;
    /// Nonzero view interpretation version.
    const VERSION: u32;
    /// View's explicitly selected input/output scope.
    const DIRECTION: crate::boundary::Direction;
    /// Actual passive record type declared at its original source owner.
    type Document;
    /// Capture ordinary observed data without publishing performing authority.
    fn boundary_document(&self) -> Self::Document;
    /// Adopt syntax through the original contextual owner.
    /// # Errors
    /// A retained resolved leaf must satisfy its original qualified receipt check.
    fn from_boundary_document(document: Self::Document) -> Result<Self, crate::QuantityError>;
}

/// Source-owned alternative metadata independent of any downstream protobuf layer.
pub trait BoundaryAlternatives {
    /// Stable public interpretation name.
    const NAME: &'static str;
    /// Actual source declaration location.
    const SOURCE: &'static str;
    /// Nonzero interpretation version.
    const VERSION: u32;
    /// Explicit input/output scope.
    const DIRECTION: crate::boundary::Direction;
}

/// Typed actual payload selected by explicit source alternative and field tags.
pub trait BoundaryAlternativeField<const ALTERNATIVE: u32, const FIELD: u32> {
    /// Actual source Rust field type.
    type Type;
}

/// Extract and reconstruct only passive data; this trait conveys no admitted contract.
pub trait BoundaryAlternative<const TAG: u32>: Sized {
    /// Actual field tuple in native tuple/struct order.
    type Fields;
    /// Borrow the same original payload tuple without cloning passive records.
    type BorrowedFields<'a>
    where
        Self: 'a;
    /// Borrow original payloads only for this selected alternative.
    fn boundary_alternative_fields(&self) -> Option<Self::BorrowedFields<'_>>;
    /// Retain the original alternative when a different tag is requested.
    fn into_boundary_alternative(self) -> Result<Self::Fields, Self>;
    /// Construct passive data, never an admitted physical proof.
    fn from_boundary_alternative(fields: Self::Fields) -> Self;
}

/// Declare the actual passive tuple enum and source tags once.
/// Its downstream callback carries names/tags, resolving types through associated fields.
#[macro_export]
macro_rules! quantity_passive_enum {
    (callback:$callback:ident;owner:$($public:ident)::+;name:$wire:literal;version:$version:literal;direction:$direction:ident;
        $(#[$meta:meta])* $vis:vis enum $name:ident {
            $($(#[$variant_meta:meta])* $variant:ident [
                $($(#[$field_meta:meta])* $field:ident:$ty:ty=>$field_tag:literal;)+
            ]=>$tag:literal;)*
        }
    ) => {
        $(#[$meta])* $vis enum $name { $($(#[$variant_meta])* $variant($($(#[$field_meta])* $ty,)+),)* }
        impl $crate::boundary::BoundaryAlternatives for $name {
            const NAME:&'static str=$wire; const SOURCE:&'static str=file!(); const VERSION:u32=$version;
            const DIRECTION:$crate::boundary::Direction=$crate::boundary::Direction::$direction;
        }
        const _: fn($name) -> $crate::$($public)::+ = |value| value;
        $(
            $(impl $crate::boundary::BoundaryAlternativeField<$tag,$field_tag> for $name {type Type=$ty;})+
            impl $crate::boundary::BoundaryAlternative<$tag> for $name {
                type Fields=($($ty,)+);
                type BorrowedFields<'a>=($(&'a $ty,)+) where Self:'a;
                fn boundary_alternative_fields(&self)->Option<Self::BorrowedFields<'_>>{
                    match self {Self::$variant($($field,)+)=>Some(($($field,)+)),_=>None}
                }
                fn into_boundary_alternative(self)->Result<Self::Fields,Self>{
                    match self {Self::$variant($($field,)+)=>Ok(($($field,)+)),other=>Err(other)}
                }
                fn from_boundary_alternative(($($field,)+):Self::Fields)->Self{Self::$variant($($field,)+)}
            }
        )*
        #[doc=concat!("Visit the actual passive alternatives of ",stringify!($name),".")]
        #[macro_export]
        macro_rules! $callback {($consumer:ident)=>{$consumer!{
            $crate::$($public)::+; $($variant=>$tag{$($field=>$field_tag;)+};)*
        }};}
    };
}

/// Explicit field transport interpretation declared beside its actual Rust storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PassiveFieldEncoding {
    /// Ordinary typed field projection.
    Native,
    /// A u64 retaining exact IEEE binary64 bits.
    Binary64Bits,
}
/// Source interpretation for an actual passive-record field.
pub trait BoundaryFieldEncoding<const TAG: u32> {
    /// Native storage's declared transport interpretation.
    const ENCODING: PassiveFieldEncoding;
}

/// Declare actual passive record fields and their stable typed callback together.
/// Reconstructing this record grants no physical authority.
#[macro_export]
macro_rules! quantity_passive_record {
    (callback:$callback:ident;owner:$($public:ident)::+;name:$wire:literal;version:$version:literal;direction:$direction:ident;
        $(#[$meta:meta])* $vis:vis struct $name:ident($ty:ty)=>$field:ident:$tag:literal,$encoding:ident;
    ) => {
        $(#[$meta])* $vis struct $name($ty);
        impl $crate::boundary::BoundaryRecord for $name {
            const NAME:&'static str=$wire;const VERSION:u32=$version;const SOURCE:&'static str=file!();
            const DIRECTION:$crate::boundary::Direction=$crate::boundary::Direction::$direction;
            const RESERVED_TAGS:&'static[u32]=&[];const RESERVED_NAMES:&'static[&'static str]=&[];
            type Fields=($ty,);
            fn into_boundary_fields(self)->Self::Fields{(self.0,)}
            fn from_boundary_fields((value,):Self::Fields)->Result<Self,$crate::DimensionError>{Ok(Self(value))}
        }
        impl $crate::boundary::BoundaryField<$tag> for $name {
            type Type=$ty;fn boundary_field(&self)->&Self::Type{&self.0}
        }
        impl $crate::boundary::BoundaryFieldEncoding<$tag> for $name {
            const ENCODING:$crate::boundary::PassiveFieldEncoding=$crate::boundary::PassiveFieldEncoding::$encoding;
        }
        const _: fn($name) -> $crate::$($public)::+ = |value| value;
        #[doc=concat!("Visit the actual passive record ",stringify!($name),".")]
        #[macro_export]
        macro_rules! $callback {($consumer:ident)=>{$consumer!{$crate::$($public)::+;$field=>$tag;}};}
    };
    (callback:$callback:ident;owner:$($public:ident)::+;name:$wire:literal;version:$version:literal;direction:$direction:ident;
        $(#[$meta:meta])* $vis:vis struct $name:ident {
            $($(#[$field_meta:meta])* $field_vis:vis $field:ident:$ty:ty=>$tag:literal,$encoding:ident;)*
        }
    ) => {
        $(#[$meta])* $vis struct $name {$($(#[$field_meta])* $field_vis $field:$ty,)*}
        impl $crate::boundary::BoundaryRecord for $name {
            const NAME:&'static str=$wire;const VERSION:u32=$version;const SOURCE:&'static str=file!();
            const DIRECTION:$crate::boundary::Direction=$crate::boundary::Direction::$direction;
            const RESERVED_TAGS:&'static[u32]=&[];const RESERVED_NAMES:&'static[&'static str]=&[];
            type Fields=($($ty,)*);
            fn into_boundary_fields(self)->Self::Fields{($(self.$field,)*)}
            fn from_boundary_fields(($($field,)*):Self::Fields)->Result<Self,$crate::DimensionError>{Ok(Self{$($field,)*})}
        }
        $(impl $crate::boundary::BoundaryField<$tag> for $name {
            type Type=$ty;fn boundary_field(&self)->&Self::Type{&self.$field}
        }
        impl $crate::boundary::BoundaryFieldEncoding<$tag> for $name {
            const ENCODING:$crate::boundary::PassiveFieldEncoding=$crate::boundary::PassiveFieldEncoding::$encoding;
        })*
        const _: fn($name) -> $crate::$($public)::+ = |value| value;
        #[doc=concat!("Visit the actual passive record ",stringify!($name),".")]
        #[macro_export]
        macro_rules! $callback {($consumer:ident)=>{$consumer!{
            $crate::$($public)::+;$($field=>$tag;)*
        }};}
    };
}
