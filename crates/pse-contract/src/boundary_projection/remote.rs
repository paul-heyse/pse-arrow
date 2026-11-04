// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Checked bindings extend an existing remote serialization declaration in place.

/// Declare a foreign record's existing Serde remote definition and typed projection
/// from one field table. `target` and `rust_path` are checked public Rust wiring;
/// the remote Serde attribute continues to check the actual library fields.
#[macro_export]
macro_rules! projection_remote {
    ($(#[$meta:meta])* $vis:vis struct $name:ident for $target:path {
        projection: { name: $wire:expr, rust_path: $public:path, $(target_path: $native_public:ty,)? version: $version:expr, direction: $direction:expr,
            reserved_tags: [$($reserved:expr),* $(,)?], reserved_names: [$($reserved_name:expr),* $(,)?] };
        $($(#[$field_meta:meta])* $field_vis:vis $field:ident: $ty:ty => {
            tag: $tag:expr, direction: $field_direction:expr, rules: [$($rule:expr),* $(,)?] $(, adapter: $adapter:ty)? };)*
    }) => {
        $(#[$meta])* $vis struct $name {
            $($(#[$field_meta])* $field_vis $field: $ty,)*
        }
        #[allow(unused_qualifications, reason = "the source's public adapter alias must remain exact in generated cross-crate paths")]
        const _: fn($name) -> $public = |value| value;
        $(#[allow(unused_qualifications, reason = "the public native alias is checked against the actual foreign target for generated downstream paths")]
        const _: fn($target) -> $native_public = |value| value;)?
        impl $crate::boundary_projection::ProjectionAdapter<$target> for $name {
            const NAME: &'static str = $wire;
            const SOURCE: &'static str = file!();
            const VERSION: u32 = $version;
            const DIRECTION: $crate::boundary_projection::Direction = $direction;
            const RESERVED_TAGS: &'static [u32] = &[$($reserved),*];
            const RESERVED_NAMES: &'static [&'static str] = &[$($reserved_name),*];
            const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[$(($tag, stringify!($field))),*];
            type Fields = ($($crate::projection_struct!(@ty $ty $(, $adapter)?),)*);
            fn public_path() -> Option<String> {
                Some(format!("pse_contract::boundary_projection::Adapted<{},{}>",
                    <Self as $crate::boundary_projection::ProjectionAdapter<$target>>::native_path(),
                    <Self as $crate::boundary_projection::ProjectionAdapter<$target>>::adapter_path()))
            }
            fn native_path() -> String {
                $crate::boundary_projection::adapter_source_path::<Self>(
                    $crate::projection_remote!(@native $target $(, $native_public)?))
            }
            fn adapter_path() -> String { $crate::boundary_projection::adapter_source_path::<Self>(stringify!($public)) }
            fn fields(graph: &mut $crate::boundary_projection::ProjectionGraph) -> Result<Vec<$crate::boundary_projection::Field>, $crate::boundary_projection::ProjectionError> {
                let _ = &graph;
                Ok(vec![$(
                    $crate::boundary_projection::Field::of::<$crate::projection_struct!(@ty $ty $(, $adapter)?)>(graph,
                        stringify!($field), $tag, $field_direction, vec![$($rule),*])?,
                )*])
            }
            fn validate(value: &$target, path: &str) -> Result<(), $crate::boundary_projection::ValidationError> {
                let _ = (value, path);
                $($crate::projection_struct!(@validate &value.$field,
                    &$crate::boundary_projection::field_path(path, stringify!($field)), &[$($rule),*] $(, $adapter)?)?;)*
                Ok(())
            }
            fn try_fields(value: &$target) -> Result<Self::Fields, $crate::boundary_projection::AdapterError> {
                <Self as $crate::boundary_projection::ProjectionAdapter<$target>>::validate(value, "")?;
                Ok(($($crate::projection_struct!(@into $crate::boundary_projection::clone_adapter_field(&value.$field) $(, $adapter)?),)*))
            }
            fn from_fields(fields: Self::Fields) -> Result<$target, $crate::boundary_projection::AdapterError> {
                let ($($field,)*) = fields;
                type Target = $target;
                let value = Target { $($field: $crate::projection_struct!(@from $field $(, $adapter)?),)* };
                <Self as $crate::boundary_projection::ProjectionAdapter<$target>>::validate(&value, "")?;
                Ok(value)
            }
        }
    };
    (@native $target:path, $public:ty) => { stringify!($public) };
    (@native $target:path) => { stringify!($target) };
}
