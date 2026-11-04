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
macro_rules! identity_bytes { ($($ty:ty),* $(,)?) => { $(
    impl ProjectionBytes for $ty {
        fn into_projection_bytes(self) -> Vec<u8> { self.as_bytes().to_vec() }
        fn from_projection_bytes(bytes: Vec<u8>, path: &str) -> Result<Self, AdapterError> {
            let bytes = bytes.try_into().map_err(|_| AdapterError::refused(path, AdapterFailure::InvalidValue, "identity byte width does not match its actual nominal owner"))?;
            Ok(Self::from_bytes(bytes))
        }
        fn from_projection_slice(bytes: &[u8], path: &str) -> Result<Self, AdapterError> {
            let bytes = bytes.try_into().map_err(|_| AdapterError::refused(path, AdapterFailure::InvalidValue, "identity byte width does not match its actual nominal owner"))?;
            Ok(Self::from_bytes(bytes))
        }
    }
)* }; }
identity_bytes!(crate::SemanticId, crate::ContentHash);
#[cfg(test)]
mod byte_tests {
    use super::*;
    #[test]
    fn nominal_byte_access_uses_actual_owner_width_and_source_path() {
        let id = crate::SemanticId::from_bytes([17; 16]);
        let bytes = id.into_projection_bytes();
        assert_eq!(
            crate::SemanticId::from_projection_bytes(bytes.clone(), "request.id").unwrap(),
            id
        );
        assert_eq!(
            crate::SemanticId::from_projection_slice(&bytes, "request.id").unwrap(),
            id
        );
        assert!(matches!(
            crate::SemanticId::from_projection_slice(&bytes[..15], "request.id"),
            Err(AdapterError::Refused {
                kind: AdapterFailure::InvalidValue,
                ..
            })
        ));
        assert!(
            matches!(crate::SemanticId::from_projection_bytes(bytes[..bytes.len()-1].to_vec(), "request.id"), Err(AdapterError::Refused { path, kind: AdapterFailure::InvalidValue, .. }) if path == "request.id")
        );
        let hash = crate::ContentHash::from_bytes([29; 32]);
        assert_eq!(
            Box::<crate::ContentHash>::from_projection_bytes(hash.into_projection_bytes(), "hash")
                .unwrap()
                .into_projection_bytes(),
            hash.as_bytes()
        );
        assert_eq!(
            Box::<crate::ContentHash>::from_projection_slice(hash.as_bytes(), "hash")
                .unwrap()
                .into_projection_bytes(),
            hash.as_bytes()
        );
    }
}
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
    crate::roles::SemanticBodyHash => "SemanticBodyHash",
    crate::roles::BindingHash => "BindingHash",
);
