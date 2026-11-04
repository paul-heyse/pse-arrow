// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source scalar declarations also emit their exact native and wire domain metadata.

// The existing nutype attribute remains the checked construction/Serde authority.
// Matching its declared validation tokens supplies projection metadata from that
// same declaration; unsupported domains fail macro matching instead of disappearing.
macro_rules! projected_scalar {
    ($(#[doc = $doc:expr])* #[nutype(
        const_fn,
        validate($($validation:tt)*),
        derive($($derive:tt)*),
        derive_unchecked($schema:path)
    )]
    pub struct $name:ident ($(#[$field:meta])* $ty:ident);) => {
        $(#[doc = $doc])*
        #[nutype(const_fn, validate($($validation)*), derive($($derive)*), derive_unchecked($schema))]
        pub struct $name($(#[$field])* $ty);

        impl crate::boundary_projection::ProjectionType for $name {
            fn projection(graph: &mut crate::boundary_projection::ProjectionGraph) -> Result<crate::boundary_projection::TypeRef, crate::boundary_projection::ProjectionError> {
                #[allow(unused_qualifications, reason = "nutype's private defining module requires this exact checked public alias for cross-crate adapters")]
                let _identity: fn(Self) -> crate::scalars::$name = |value| value;
                let root = graph.declare::<Self>(stringify!($name), file!(), 1, crate::boundary_projection::Direction::Both,
                    &[], &[], |graph| Ok(crate::boundary_projection::Shape::Record(vec![
                        crate::boundary_projection::Field::of::<$ty>(graph, "value", 1,
                            crate::boundary_projection::Direction::Both,
                            vec![projected_scalar!(@rule $ty; $($validation)*)])?,
                    ])))?;
                graph.public_path::<Self>(&root, concat!("crate::scalars::", stringify!($name)))?;
                Ok(root)
            }
        }
        impl crate::boundary_projection::ValidateProjection for $name {
            const NEEDS_RECURSIVE_VALIDATION: bool = false;
            fn validate_at(&self, path: &str) -> Result<(), crate::boundary_projection::ValidationError> {
                crate::boundary_projection::validate_field(&self.into_inner(),
                    &crate::boundary_projection::field_path(path, "value"),
                    &[projected_scalar!(@rule $ty; $($validation)*)])
            }
        }
        impl crate::boundary_projection::ProjectionRecord for $name {
            const VERSION: u32 = 1;
            const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[(1, "value")];
            type Fields = ($ty,);
            fn into_projection_fields(self) -> Self::Fields { (self.into_inner(),) }
            fn from_projection_fields(fields: Self::Fields) -> Result<Self, crate::boundary_projection::AdapterError> {
                Self::try_new(fields.0).map_err(|error| crate::boundary_projection::AdapterError::refused(
                    "value", crate::boundary_projection::AdapterFailure::InvalidValue, error.to_string()))
            }
        }
    };
    (@rule f64; finite) => { crate::boundary_projection::LocalRule::Float64Domain(crate::boundary_projection::Float64Domain::Finite) };
    (@rule f64; finite, greater = 0.0) => { crate::boundary_projection::LocalRule::Float64Domain(crate::boundary_projection::Float64Domain::PositiveFinite) };
    (@rule f64; finite, greater = 0.0, less_or_equal = 1.0) => { crate::boundary_projection::LocalRule::Float64Domain(crate::boundary_projection::Float64Domain::PositiveUnit) };
    (@rule usize; greater = 0) => { crate::boundary_projection::LocalRule::UnsignedRange { min: 1, max: u64::MAX } };
}
