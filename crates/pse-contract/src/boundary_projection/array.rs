// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
/// Existing fixed-array projection naming, including its actual argument fallback.
pub fn fixed_array_name(element: &TypeRef, element_argument: &str, extent: usize) -> String {
    let argument = match element {
        TypeRef::Named(name) => name.clone(),
        TypeRef::Scalar(scalar) => format!("{scalar:?}"),
        _ => element_argument.into(),
    };
    format!("Array{extent}{argument}")
}
/// Existing fixed-array value layout, shared by its native owner and readers of
/// that declared shape. This cannot register a Rust owner or reconstruct values.
/// # Errors
/// An extent outside the native length-rule range refuses.
pub fn fixed_array_shape(element: TypeRef, extent: usize) -> Result<Shape, ProjectionError> {
    let length = u64::try_from(extent)
        .map_err(|_| ProjectionError::invalid("array", "array extent exceeds uint64"))?;
    Ok(Shape::Record(vec![Field {
        name: "values".into(),
        tag: 1,
        ty: sequence_type(element),
        direction: Direction::Both,
        rules: vec![LocalRule::Length {
            min: length,
            max: length,
        }],
        named_facets: BTreeMap::new(),
    }]))
}
impl<T: ProjectionType, const N: usize> ProjectionType for [T; N] {
    fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
        let element = T::projection(graph)?;
        let argument = if matches!(element, TypeRef::Named(_) | TypeRef::Scalar(_)) {
            String::new()
        } else {
            T::projection_argument_name(graph)?
        };
        let name = fixed_array_name(&element, &argument, N);
        let root = graph.declare::<Self>(&name, file!(), 1, Direction::Both, &[], &[], |_| {
            fixed_array_shape(element.clone(), N)
        })?;
        graph.array_container::<T, N>(&root, element)?;
        let element = T::projection_rust_path(graph)?;
        graph.public_path::<Self>(&root, &format!("[{element};{N}]"))?;
        Ok(root)
    }
    fn projection_rust_path(graph: &mut ProjectionGraph) -> Result<String, ProjectionError> {
        Ok(format!("[{};{N}]", T::projection_rust_path(graph)?))
    }
    fn projection_argument_name(graph: &mut ProjectionGraph) -> Result<String, ProjectionError> {
        Ok(array_argument_name(&T::projection_argument_name(graph)?, N))
    }
}
impl<T, const N: usize> ProjectionRecord for [T; N] {
    const VERSION: u32 = 1;
    const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[(1, "values")];
    type Fields = (Vec<T>,);
    fn into_projection_fields(self) -> Self::Fields {
        (self.into_iter().collect(),)
    }
    fn from_projection_fields((values,): Self::Fields) -> Result<Self, AdapterError> {
        values.try_into().map_err(|_: Vec<T>| {
            AdapterError::refused(
                "values",
                AdapterFailure::InvalidValue,
                format!("fixed array requires exactly {N} components"),
            )
        })
    }
}
impl<T: ValidateProjection, const N: usize> ValidateProjection for [T; N] {
    const SUPPORTS_FLOAT64_DOMAIN: bool = T::SUPPORTS_FLOAT64_DOMAIN;
    const NEEDS_RECURSIVE_VALIDATION: bool = T::NEEDS_RECURSIVE_VALIDATION;
    fn validate_at(&self, path: &str) -> Result<(), ValidationError> {
        if !Self::NEEDS_RECURSIVE_VALIDATION {
            return Ok(());
        }
        for (index, value) in self.iter().enumerate() {
            value.validate_at(&format!("{path}[{index}]"))?;
        }
        Ok(())
    }
    fn projection_length(&self) -> Option<usize> {
        Some(N)
    }
    fn projection_float64_domain(&self, domain: Float64Domain) -> Option<bool> {
        Self::SUPPORTS_FLOAT64_DOMAIN.then(|| {
            self.iter()
                .all(|value| value.projection_float64_domain(domain) == Some(true))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projection_struct;
    #[test]
    fn array_names_preserve_actual_container_arguments_after_byte_projection_folding() {
        let mut graph = ProjectionGraph::default();
        let plain = <[Option<Vec<u8>>; 2]>::projection(&mut graph).unwrap();
        let boxed = <[Option<Box<Vec<u8>>>; 2]>::projection(&mut graph).unwrap();
        assert_eq!(plain, TypeRef::Named("Array2OptionalListU8".into()));
        assert_eq!(boxed, TypeRef::Named("Array2OptionalBoxedListU8".into()));
        assert_ne!(plain, boxed);
        let element = TypeRef::Optional(Box::new(TypeRef::Scalar(Scalar::Bytes)));
        assert_eq!(
            fixed_array_name(&element, "OptionalBytes", 2),
            "Array2OptionalBytes"
        );
        assert_eq!(
            graph.definitions()["Array2OptionalListU8"].shape,
            fixed_array_shape(element, 2).unwrap()
        );
    }
    #[test]
    fn floating_domain_refuses_record_resemblance_and_non_binary64_leaves() {
        projection_struct! {
            struct NotAnArray {
                projection: { name: "NotAnArray", version: 1, direction: Direction::Both,
                    reserved_tags: [], reserved_names: [] };
                values: Vec<f64> => { tag: 1, direction: Direction::Both, rules: [] };
            }
        }
        let mut graph = ProjectionGraph::of::<NotAnArray>().unwrap();
        let root = NotAnArray::projection(&mut graph).unwrap();
        assert!(!float_collections::supports_float64_domain(
            &root,
            graph.definitions()
        ));
        assert!(
            graph
                .array_container::<f64, 2>(&root, TypeRef::Scalar(Scalar::F64Bits))
                .is_err()
        );
        assert!(!float_collections::supports_float64_domain(
            &TypeRef::Repeated(Box::new(TypeRef::Scalar(Scalar::U64))),
            graph.definitions()
        ));
    }
}
