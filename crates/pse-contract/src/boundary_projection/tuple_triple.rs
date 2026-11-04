// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Intrinsic source owner for the actual standard `(A, B, C)` tuple.

use super::*;

const FIRST: (u32, &str) = (1, "first");
const SECOND: (u32, &str) = (2, "second");
const THIRD: (u32, &str) = (3, "third");

impl<A: ProjectionType, B: ProjectionType, C: ProjectionType> ProjectionType for (A, B, C) {
    fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
        let first = A::projection_argument_name(graph)?;
        let second = B::projection_argument_name(graph)?;
        let third = C::projection_argument_name(graph)?;
        let name = format!("Tuple3Of{first}And{second}And{third}");
        let root = graph.declare::<Self>(&name, file!(), 1, Direction::Both, &[], &[], |graph| {
            Ok(Shape::Record(vec![
                Field::of::<A>(graph, FIRST.1, FIRST.0, Direction::Both, vec![])?,
                Field::of::<B>(graph, SECOND.1, SECOND.0, Direction::Both, vec![])?,
                Field::of::<C>(graph, THIRD.1, THIRD.0, Direction::Both, vec![])?,
            ]))
        })?;
        let path = Self::projection_rust_path(graph)?;
        graph.public_path::<Self>(&root, &path)?;
        Ok(root)
    }
    fn projection_rust_path(graph: &mut ProjectionGraph) -> Result<String, ProjectionError> {
        Ok(format!("({},{},{})", A::projection_rust_path(graph)?, B::projection_rust_path(graph)?, C::projection_rust_path(graph)?))
    }
}
impl<A, B, C> ProjectionRecord for (A, B, C) {
    const VERSION: u32 = 1;
    const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[FIRST, SECOND, THIRD];
    type Fields = (A, B, C);
    fn into_projection_fields(self) -> Self::Fields { self }
    fn from_projection_fields(fields: Self::Fields) -> Result<Self, AdapterError> { Ok(fields) }
}
impl<A: ValidateProjection, B: ValidateProjection, C: ValidateProjection> ValidateProjection for (A, B, C) {
    const NEEDS_RECURSIVE_VALIDATION: bool = A::NEEDS_RECURSIVE_VALIDATION || B::NEEDS_RECURSIVE_VALIDATION || C::NEEDS_RECURSIVE_VALIDATION;
    fn validate_at(&self, path: &str) -> Result<(), ValidationError> {
        self.0.validate_at(&field_path(path, FIRST.1))?;
        self.1.validate_at(&field_path(path, SECOND.1))?;
        self.2.validate_at(&field_path(path, THIRD.1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intrinsic_triple_preserves_actual_order_public_paths_and_native_json() {
        let native = (3usize, vec!["x".to_owned()], 17_u64);
        let bytes = serde_json::to_vec(&native).unwrap();
        assert_eq!(bytes, br#"[3,["x"],17]"#);
        let restored = <(usize, Vec<String>, u64)>::from_projection_fields(native.clone().into_projection_fields()).unwrap();
        assert_eq!(restored, native);
        let graph = ProjectionGraph::of::<(usize, Vec<String>, u64)>().unwrap();
        let owner = &graph.definitions()["Tuple3OfUsizeAndListStringAndU64"];
        assert_eq!(owner.rust_type, core::any::type_name::<(usize, Vec<String>, u64)>());
        assert_eq!(owner.public_rust_path.as_deref(), Some("(usize,std::vec::Vec<String>,u64)"));
        let Shape::Record(fields) = &owner.shape else { panic!("actual triple record") };
        assert_eq!(fields.iter().map(|field| (field.tag, field.name.as_str())).collect::<Vec<_>>(), vec![(1, "first"), (2, "second"), (3, "third")]);
        assert_eq!(owner.construction, ConstructionRole::Data);
        assert!(owner.native_container.is_none());
    }
}
