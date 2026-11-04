// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact standard-library and low-level nominal leaves of native requests.

use super::*;

impl ProjectionType for std::time::Duration {
    fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
        graph.declare::<Self>("NonnegativeDuration", file!(), 1, Direction::Both, &[], &[], |graph|
            Ok(Shape::Record(vec![
                Field::of::<u64>(graph, "secs", 1, Direction::Both, vec![])?,
                Field::of::<u32>(graph, "nanos", 2, Direction::Both,
                    vec![LocalRule::UnsignedRange { min: 0, max: 999_999_999 }])?,
            ])))
    }
}
impl ProjectionRecord for std::time::Duration {
    const VERSION: u32 = 1;
    const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[(1, "secs"), (2, "nanos")];
    type Fields = (u64, u32);
    fn into_projection_fields(self) -> Self::Fields { (self.as_secs(), self.subsec_nanos()) }
    fn from_projection_fields((secs, nanos): Self::Fields) -> Result<Self, AdapterError> {
        // Duration::new normalizes an out-of-domain nanos component, and may then
        // overflow secs. Boundary data must already have its exact canonical shape.
        if nanos >= 1_000_000_000 {
            return Err(AdapterError::refused("nanos", AdapterFailure::InvalidValue,
                "duration nanoseconds must be below one second"));
        }
        Ok(Self::new(secs, nanos))
    }
}
impl ValidateProjection for std::time::Duration {
    const NEEDS_RECURSIVE_VALIDATION: bool = false;
    fn validate_at(&self, _: &str) -> Result<(), ValidationError> { Ok(()) }
}

macro_rules! hash_role_projection {
    ($($ty:path => $name:literal),* $(,)?) => { $(
        impl ProjectionType for $ty {
            fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
                graph.declare::<Self>($name, "crates/pse-ids/src/roles.rs", 1, Direction::Both, &[], &[], |graph|
                    Ok(Shape::Record(vec![Field::of::<pse_ids::ContentHash>(graph, "id", 1,
                        Direction::Both, vec![LocalRule::Required])?])))
            }
        }
        impl ProjectionRecord for $ty {
            const VERSION: u32 = 1;
            const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[(1, "id")];
            type Fields = (pse_ids::ContentHash,);
            fn into_projection_fields(self) -> Self::Fields { (self.as_id(),) }
            fn from_projection_fields((id,): Self::Fields) -> Result<Self, AdapterError> { Ok(Self::from_id(id)) }
        }
        impl ValidateProjection for $ty {
            const NEEDS_RECURSIVE_VALIDATION: bool = false;
            fn validate_at(&self, _: &str) -> Result<(), ValidationError> { Ok(()) }
        }
    )* };
}
hash_role_projection!(
    pse_ids::roles::SourceRevisionHash => "SourceRevisionHash",
    pse_ids::roles::BindingHash => "BindingHash",
);
