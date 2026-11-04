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

        impl pse_contract::boundary_projection::ProjectionType for $name {
            fn projection(graph: &mut pse_contract::boundary_projection::ProjectionGraph) -> Result<pse_contract::boundary_projection::TypeRef, pse_contract::boundary_projection::ProjectionError> {
                #[allow(unused_qualifications, reason = "nutype's private defining module requires this exact checked public alias for cross-crate adapters")]
                let _identity: fn(Self) -> crate::scalars::$name = |value| value;
                let root = graph.declare::<Self>(stringify!($name), file!(), 1, pse_contract::boundary_projection::Direction::Both,
                    &[], &[], |graph| Ok(pse_contract::boundary_projection::Shape::Record(vec![
                        pse_contract::boundary_projection::Field::of::<$ty>(graph, "value", 1,
                            pse_contract::boundary_projection::Direction::Both,
                            vec![projected_scalar!(@rule $ty; $($validation)*)])?,
                    ])))?;
                projected_scalar!(@mark $ty, graph, root);
                graph.public_path::<Self>(&root, concat!("crate::scalars::", stringify!($name)))?;
                Ok(root)
            }
        }
        impl pse_contract::boundary_projection::ValidateProjection for $name {
            const SUPPORTS_FLOAT64_DOMAIN: bool = projected_scalar!(@float $ty);
            const NEEDS_RECURSIVE_VALIDATION: bool = false;
            fn projection_float64(&self) -> Option<f64> { projected_scalar!(@value $ty, self) }
            fn validate_at(&self, path: &str) -> Result<(), pse_contract::boundary_projection::ValidationError> {
                pse_contract::boundary_projection::validate_field(&self.into_inner(),
                    &pse_contract::boundary_projection::field_path(path, "value"),
                    &[projected_scalar!(@rule $ty; $($validation)*)])
            }
        }
        impl pse_contract::boundary_projection::ProjectionRecord for $name {
            const VERSION: u32 = 1;
            const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[(1, "value")];
            type Fields = ($ty,);
            fn into_projection_fields(self) -> Self::Fields { (self.into_inner(),) }
            fn from_projection_fields(fields: Self::Fields) -> Result<Self, pse_contract::boundary_projection::AdapterError> {
                Self::try_new(fields.0).map_err(|error| pse_contract::boundary_projection::AdapterError::refused(
                    "value", pse_contract::boundary_projection::AdapterFailure::InvalidValue, error.to_string()))
            }
        }
        projected_scalar!(@nominal $name, $ty);
    };
    (@nominal $name:ident, f64) => { impl pse_contract::boundary_projection::NominalFloat64 for $name {} };
    (@nominal $name:ident, usize) => {};
    (@mark f64, $graph:ident, $root:ident) => { $graph.nominal_float64::<Self>(&$root)? };
    (@mark usize, $graph:ident, $root:ident) => {};
    (@float f64) => { true };
    (@float usize) => { false };
    (@value f64, $self:ident) => { Some($self.into_inner()) };
    (@value usize, $self:ident) => { None };
    (@rule f64; finite) => { pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite) };
    (@rule f64; finite, greater = 0.0) => { pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::PositiveFinite) };
    (@rule f64; finite, greater = 0.0, less_or_equal = 1.0) => { pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::PositiveUnit) };
    (@rule usize; greater = 0) => { pse_contract::boundary_projection::LocalRule::UnsignedRange { min: 1, max: u64::MAX } };
}
