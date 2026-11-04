// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use pse_contract::boundary_projection::*;
macro_rules! scalar { ($($ty:ty => $kind:ident),*) => { $(
 impl ProjectionType for $ty {
  fn projection(_: &mut ProjectionGraph) -> Result<TypeRef,ProjectionError> { Ok(TypeRef::Scalar(Scalar::$kind)) }
  fn projection_rust_path(_: &mut ProjectionGraph) -> Result<String,ProjectionError> {
   Ok(adapter_source_path::<$ty>(stringify!($ty)))
  }
 }
)* }; }
scalar!(crate::SemanticId => SemanticId, crate::ContentHash => ContentHash);
impl ValidateProjection for crate::SemanticId {
    const NEEDS_RECURSIVE_VALIDATION: bool = false;
    fn validate_at(&self, _: &str) -> Result<(), ValidationError> {
        Ok(())
    }
    fn projection_length(&self) -> Option<usize> {
        Some(16)
    }
}
impl ValidateProjection for crate::ContentHash {
    const NEEDS_RECURSIVE_VALIDATION: bool = false;
    fn validate_at(&self, _: &str) -> Result<(), ValidationError> {
        Ok(())
    }
    fn projection_length(&self) -> Option<usize> {
        Some(32)
    }
}
macro_rules! hash_role_projection {
    ($($ty:path => $name:literal),* $(,)?) => { $(
        impl ProjectionType for $ty {
            fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
                graph.declare::<Self>($name, "crates/pse-ids/src/roles.rs", 1, Direction::Both, &[], &[], |graph|
                    Ok(Shape::Record(vec![Field::of::<crate::ContentHash>(graph, "id", 1,
                        Direction::Both, vec![LocalRule::Required])?])))
            }
        }
        impl ProjectionRecord for $ty {
            const VERSION: u32 = 1;
            const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[(1, "id")];
            type Fields = (crate::ContentHash,);
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
    crate::roles::SourceRevisionHash => "SourceRevisionHash",
    crate::roles::BindingHash => "BindingHash",
);
