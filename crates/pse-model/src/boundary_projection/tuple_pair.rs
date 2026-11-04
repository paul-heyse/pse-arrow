// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Intrinsic owner for the actual standard `(A, B)` tuple.
//! No named-record resemblance or function-argument-specific surrogate is used.

use super::*;

const FIRST: (u32, &str) = (1, "first");
const SECOND: (u32, &str) = (2, "second");

impl<A: ProjectionType, B: ProjectionType> ProjectionType for (A, B) {
    fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
        let first = A::projection_argument_name(graph)?;
        let second = B::projection_argument_name(graph)?;
        let name = format!("Tuple2Of{first}And{second}");
        let root =
            graph.declare::<Self>(&name, file!(), 1, Direction::Both, &[], &[], |graph| {
                Ok(Shape::Record(vec![
                    Field::of::<A>(graph, FIRST.1, FIRST.0, Direction::Both, vec![])?,
                    Field::of::<B>(graph, SECOND.1, SECOND.0, Direction::Both, vec![])?,
                ]))
            })?;
        let path = Self::projection_rust_path(graph)?;
        graph.public_path::<Self>(&root, &path)?;
        Ok(root)
    }
    fn projection_rust_path(graph: &mut ProjectionGraph) -> Result<String, ProjectionError> {
        // Recursive arguments must supply their explicitly checked public alias
        // while still being built. Never substitute a potentially private type_name.
        Ok(format!(
            "({},{})",
            A::projection_rust_path(graph)?,
            B::projection_rust_path(graph)?
        ))
    }
}
impl<A, B> ProjectionRecord for (A, B) {
    const VERSION: u32 = 1;
    const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[FIRST, SECOND];
    type Fields = (A, B);
    fn into_projection_fields(self) -> Self::Fields {
        self
    }
    fn from_projection_fields(fields: Self::Fields) -> Result<Self, AdapterError> {
        Ok(fields)
    }
}
impl<A: ValidateProjection, B: ValidateProjection> ValidateProjection for (A, B) {
    const NEEDS_RECURSIVE_VALIDATION: bool =
        A::NEEDS_RECURSIVE_VALIDATION || B::NEEDS_RECURSIVE_VALIDATION;
    fn validate_at(&self, path: &str) -> Result<(), ValidationError> {
        self.0.validate_at(&field_path(path, FIRST.1))?;
        self.1.validate_at(&field_path(path, SECOND.1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intrinsic_pair_retains_actual_tuple_order_types_and_native_json() {
        let pair = ("x".to_owned(), 17_u64);
        assert_eq!(serde_json::to_vec(&pair).unwrap(), br#"["x",17]"#);
        let fields = pair.clone().into_projection_fields();
        assert_eq!(fields, pair);
        assert_eq!(
            <(String, u64)>::from_projection_fields(fields).unwrap(),
            pair
        );
        assert_eq!(
            <(String, u64)>::FIELD_LAYOUT,
            &[(1, "first"), (2, "second")]
        );
        let graph = ProjectionGraph::of::<(String, u64)>().unwrap();
        let owner = &graph.definitions()["Tuple2OfStringAndU64"];
        assert_eq!(owner.rust_type, core::any::type_name::<(String, u64)>());
        assert_eq!(owner.public_rust_path.as_deref(), Some("(String,u64)"));
        let Shape::Record(fields) = &owner.shape else {
            panic!("actual tuple record")
        };
        assert_eq!(fields[0].ty, TypeRef::Scalar(Scalar::String));
        assert_eq!(fields[1].ty, TypeRef::Scalar(Scalar::U64));
        assert!(
            owner.native_container.is_none(),
            "a heterogeneous tuple has no numeric-container meaning"
        );
    }
}
