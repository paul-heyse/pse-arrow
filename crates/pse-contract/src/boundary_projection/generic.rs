// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public Rust path and stable shape arguments for actual generic owner declarations.

use super::{ProjectionError, ProjectionGraph, ProjectionType, TypeRef};

/// Resolve one generic argument's declared shape name and checked public Rust path.
/// These are naming/path mechanics; the argument's actual fields remain its owner.
/// # Errors
/// An unresolved recursive argument refuses rather than exposing a private Rust path.
pub fn generic_argument<T: ProjectionType>(
    graph: &mut ProjectionGraph,
) -> Result<(String, String), ProjectionError> {
    Ok((
        T::projection_argument_name(graph)?,
        T::projection_rust_path(graph)?,
    ))
}

/// Default projection argument name, shared with source type renderers.
pub fn argument_name(ty: &TypeRef) -> String {
    match ty {
        TypeRef::Named(name) => name.clone(),
        TypeRef::Scalar(scalar) => format!("{scalar:?}"),
        TypeRef::Optional(inner) => format!("Optional{}", argument_name(inner)),
        TypeRef::Repeated(inner) => format!("List{}", argument_name(inner)),
        TypeRef::Set(inner) => format!("Set{}", argument_name(inner)),
        TypeRef::Map(key, value) => format!("Map{}To{}", argument_name(key), argument_name(value)),
    }
}
/// Existing Option argument naming, independent of representation folding.
pub fn optional_argument_name(element: &str) -> String {
    format!("Optional{element}")
}
/// Existing Vec argument naming, including byte vectors' actual Rust container.
pub fn sequence_argument_name(element: &str) -> String {
    format!("List{element}")
}
/// Existing array argument naming, independent of its named projection shortcut.
pub fn array_argument_name(element: &str, extent: usize) -> String {
    format!("Array{extent}{element}")
}

/// Declare one actual generic record and its source metadata without a monomorphic
/// field-table twin. Each admitted argument gets its own checked owner identity.
#[macro_export]
macro_rules! projection_generic_struct {
    ($(#[$meta:meta])* $vis:vis struct $name:ident <$parameter:ident $(= $default:ty)?> {
        projection: { name: $wire:expr, rust_path: $($public:ident)::+, version: $version:expr, direction: $direction:expr,
            reserved_tags: [$($reserved:expr),* $(,)?], reserved_names: [$($reserved_name:expr),* $(,)?] };
        $($(#[$field_meta:meta])* $field_vis:vis $field:ident: $ty:ty => {
            tag: $tag:expr, direction: $field_direction:expr, rules: [$($rule:expr),* $(,)?] $(, adapter: $adapter:ty)? };)*
        $(@local { $($(#[$local_meta:meta])* $local_vis:vis $local:ident: $local_ty:ty = $local_default:expr => check: $local_check:expr;)* })?
    }) => {
        $(#[$meta])* $vis struct $name<$parameter $(= $default)?> {
            $($(#[$field_meta])* $field_vis $field: $ty,)*
            $($($(#[$local_meta])* $local_vis $local: $local_ty,)*)?
        }
        impl<$parameter: $crate::boundary_projection::ProjectionType> $crate::boundary_projection::ProjectionType for $name<$parameter> {
            fn projection(graph: &mut $crate::boundary_projection::ProjectionGraph) -> Result<$crate::boundary_projection::TypeRef, $crate::boundary_projection::ProjectionError> {
                #[allow(unused_qualifications, reason = "the public generic alias is checked at its source and retained for cross-crate generated paths")]
                let _identity: fn(Self) -> $($public)::+<$parameter> = |value| value;
                let (argument, path) = $crate::boundary_projection::generic_argument::<$parameter>(graph)?;
                let name = format!("{}{}", $wire, argument);
                let root = graph.declare::<Self>(&name, file!(), $version, $direction, &[$($reserved),*], &[$($reserved_name),*], |graph| {
                    let _ = &graph;
                    Ok($crate::boundary_projection::Shape::Record(vec![$(
                        $crate::boundary_projection::Field::of::<$crate::projection_struct!(@ty $ty $(, $adapter)?)>(graph,
                            stringify!($field), $tag, $field_direction, vec![$($rule),*])?,
                    )*]))
                })?;
                let public = $crate::boundary_projection::adapter_source_path::<Self>(stringify!($($public)::+));
                graph.public_path::<Self>(&root, &format!("{public}<{path}>"))?;
                Ok(root)
            }
        }
        impl<$parameter: $crate::boundary_projection::ValidateProjection> $crate::boundary_projection::ValidateProjection for $name<$parameter> {
            fn validate_at(&self, path: &str) -> Result<(), $crate::boundary_projection::ValidationError> {
                let _ = path;
                $($crate::projection_struct!(@validate &self.$field,
                    &$crate::boundary_projection::field_path(path, stringify!($field)), &[$($rule),*] $(, $adapter)?)?;)*
                $($(($local_check)(&self.$local, &$crate::boundary_projection::field_path(path, stringify!($local)))?;)*)?
                Ok(())
            }
        }
        impl<$parameter> $crate::boundary_projection::ProjectionRecord for $name<$parameter> {
            const VERSION: u32 = $version;
            const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[$(($tag, stringify!($field))),*];
            type Fields = ($($crate::projection_struct!(@ty $ty $(, $adapter)?),)*);
            fn into_projection_fields(self) -> Self::Fields { ($($crate::projection_struct!(@into self.$field $(, $adapter)?),)*) }
            fn from_projection_fields(fields: Self::Fields) -> Result<Self, $crate::boundary_projection::AdapterError> {
                let ($($field,)*) = fields;
                Ok(Self { $($field: $crate::projection_struct!(@from $field $(, $adapter)?),)* $($($local: $local_default,)*)? })
            }
        }
    };
}
