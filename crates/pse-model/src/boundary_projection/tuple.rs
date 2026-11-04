// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Named source fields retain existing tuple document and native alternative forms.

/// Declare a named native record while preserving its existing tuple JSON encoding.
/// Do not derive Serde or JsonSchema on the record: these implementations delegate
/// to the original field tuple, in its explicitly declared order.
#[macro_export]
macro_rules! projection_tuple_struct {
    ($(#[$meta:meta])* $vis:vis struct $name:ident {
        projection: { $($projection:tt)* };
        $($(#[$field_meta:meta])* $field_vis:vis $field:ident: $ty:ty => {
            tag: $tag:expr, direction: $direction:expr, rules: [$($rule:expr),* $(,)?] };)*
    }) => {
        $crate::projection_struct! {
            $(#[$meta])* $vis struct $name {
                projection: { $($projection)* };
                $($(#[$field_meta])* $field_vis $field: $ty => { tag: $tag, direction: $direction, rules: [$($rule),*] };)*
            }
        }
        impl ::serde::Serialize for $name {
            fn serialize<S: ::serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                ::serde::Serialize::serialize(&($( &self.$field,)*), serializer)
            }
        }
        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let ($($field,)*) = <($($ty,)*) as ::serde::Deserialize<'de>>::deserialize(deserializer)?;
                Ok(Self { $($field,)* })
            }
        }
        impl ::schemars::JsonSchema for $name {
            fn schema_name() -> ::std::borrow::Cow<'static, str> { stringify!($name).into() }
            fn json_schema(generator: &mut ::schemars::SchemaGenerator) -> ::schemars::Schema {
                <($($ty,)*) as ::schemars::JsonSchema>::json_schema(generator)
            }
        }
    };
}

/// Declare actual unit/tuple enum alternatives with explicitly named and tagged
/// payload components. Native tuple constructors and Serde payload order are retained.
#[macro_export]
macro_rules! projection_tuple_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident {
        projection: { name: $wire:expr, $(rust_path: $public:path,)? version: $version:expr, direction: $direction:expr,
            reserved_tags: [$($reserved:expr),* $(,)?], reserved_names: [$($reserved_name:expr),* $(,)?] };
        $($(#[$variant_meta:meta])* $variant:ident $([
            $($(#[$field_meta:meta])* $field:ident: $ty:ty => {
                tag: $tag:expr, direction: $field_direction:expr, rules: [$($rule:expr),* $(,)?] };)+
        ])? => $variant_tag:expr;)*
    }) => {
        $(#[$meta])* $vis enum $name { $($(#[$variant_meta])* $variant $(($($(#[$field_meta])* $ty,)+))?,)* }
        $(#[allow(unused_qualifications, reason = "the source public tuple-enum alias is checked verbatim for downstream adapters")]
        const _: fn($name) -> $public = |value| value;)?
        impl $crate::boundary_projection::ProjectionType for $name {
            fn projection(graph: &mut $crate::boundary_projection::ProjectionGraph) -> Result<$crate::boundary_projection::TypeRef, $crate::boundary_projection::ProjectionError> {
                let root = graph.declare::<Self>($wire, file!(), $version, $direction, &[$($reserved),*], &[$($reserved_name),*], |graph| {
                    let _ = &graph;
                    Ok($crate::boundary_projection::Shape::Alternatives(vec![$(
                        $crate::boundary_projection::Alternative { name: stringify!($variant).into(), tag: $variant_tag, fields: vec![
                            $($($crate::boundary_projection::Field::of::<$ty>(graph, stringify!($field), $tag, $field_direction, vec![$($rule),*])?,)+)?
                        ] },
                    )*]))
                })?;
                $(graph.public_path::<Self>(&root, stringify!($public))?;)?
                Ok(root)
            }
        }
        impl $crate::boundary_projection::ValidateProjection for $name {
            fn validate_at(&self, path: &str) -> Result<(), $crate::boundary_projection::ValidationError> {
                let _ = path;
                $($(#[allow(irrefutable_let_patterns, reason = "a source owner can intentionally expose a single tuple alternative")]
                if let Self::$variant($($field,)+) = self {
                    let alternative = $crate::boundary_projection::field_path(path, stringify!($variant));
                    $($crate::boundary_projection::validate_field($field,
                        &$crate::boundary_projection::field_path(&alternative, stringify!($field)), &[$($rule),*])?;)+
                })?)*
                Ok(())
            }
        }
        $($crate::projection_tuple_enum!(@access $name, $version, $variant, $variant_tag; $($($field: $ty => $tag,)*)?);)*
    };
    (@access $name:ident, $version:expr, $variant:ident, $tag:expr;) => {
        $crate::projection_enum!(@access $name, $version, $variant, $tag;);
    };
    (@access $name:ident, $version:expr, $variant:ident, $tag:expr; $($field:ident: $ty:ty => $field_tag:expr,)+) => {
        impl $crate::boundary_projection::ProjectionAlternative<$tag> for $name {
            const VERSION: u32 = $version;
            const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[$(($field_tag, stringify!($field))),+];
            type Fields = ($($ty,)+);
            #[allow(unreachable_patterns, reason = "a source owner can intentionally expose a single tuple alternative")]
            fn into_projection_alternative(self) -> Result<Self::Fields, Self> {
                match self { Self::$variant($($field,)+) => Ok(($($field,)+)), other => Err(other) }
            }
            fn from_projection_alternative(($($field,)+): Self::Fields) -> Self { Self::$variant($($field,)+) }
        }
    };
}
