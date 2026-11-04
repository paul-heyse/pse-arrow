// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-owned nominal wrappers preserve native tuple constructors and Serde shape.

/// The native record layout shared by nominal identity owners and their source renderer.
pub fn identity_shape(base: super::TypeRef) -> super::Shape {
    super::Shape::Record(vec![super::Field {
        name: "id".into(), tag: 1, ty: base, direction: super::Direction::Both,
        rules: vec![super::LocalRule::Required], named_facets: Default::default(),
    }])
}

/// Declare a plain nominal tuple wrapper together with its stable value field.
/// Invariant-bearing wrappers must instead retain their checked native constructor.
#[macro_export]
macro_rules! projection_newtype {
    ($(#[$meta:meta])* $vis:vis struct $name:ident ($field_vis:vis $ty:ty) {
        projection: { name: $wire:expr, version: $version:expr, direction: $direction:expr,
            reserved_tags: [$($reserved:expr),* $(,)?], reserved_names: [$($reserved_name:expr),* $(,)?] };
        value: { tag: $tag:expr, rules: [$($rule:expr),* $(,)?] };
    }) => {
        $(#[$meta])* $vis struct $name($field_vis $ty);
        impl $crate::boundary_projection::ProjectionType for $name {
            fn projection(graph: &mut $crate::boundary_projection::ProjectionGraph) -> Result<$crate::boundary_projection::TypeRef, $crate::boundary_projection::ProjectionError> {
                graph.declare::<Self>($wire, file!(), $version, $direction, &[$($reserved),*], &[$($reserved_name),*], |graph| {
                    Ok($crate::boundary_projection::Shape::Record(vec![
                        $crate::boundary_projection::Field::of::<$ty>(graph, "value", $tag, $direction, vec![$($rule),*])?,
                    ]))
                })
            }
        }
        impl $crate::boundary_projection::ValidateProjection for $name {
            fn validate_at(&self, path: &str) -> Result<(), $crate::boundary_projection::ValidationError> {
                $crate::boundary_projection::validate_field(&self.0,
                    &$crate::boundary_projection::field_path(path, "value"), &[$($rule),*])
            }
        }
        impl $crate::boundary_projection::ProjectionRecord for $name {
            const VERSION: u32 = $version;
            const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[($tag, "value")];
            type Fields = ($ty,);
            fn into_projection_fields(self) -> Self::Fields { (self.0,) }
            fn from_projection_fields(fields: Self::Fields) -> Result<Self, $crate::boundary_projection::AdapterError> {
                let value = Self(fields.0);
                $crate::boundary_projection::ValidateProjection::validate_projection(&value)?;
                Ok(value)
            }
        }
    };
}
