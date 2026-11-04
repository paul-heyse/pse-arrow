use pse_contract::boundary_projection::*;
#[cfg(test)]
use pse_contract::{projection_struct,projection_enum};
impl ProjectionType for crate::Bytes {
 fn projection(_: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> { Ok(TypeRef::Scalar(Scalar::Bytes)) }
 fn projection_rust_path(_: &mut ProjectionGraph) -> Result<String, ProjectionError> { Ok("pse_model::Bytes".into()) }
 fn projection_argument_name(_: &mut ProjectionGraph) -> Result<String, ProjectionError> { Ok("Bytes".into()) }
}
impl ValidateProjection for crate::Bytes {
    const NEEDS_RECURSIVE_VALIDATION: bool = false;
    fn validate_at(&self, _: &str) -> Result<(), ValidationError> {
        Ok(())
    }
    fn projection_length(&self) -> Option<usize> {
        Some(self.as_slice().len())
    }
}

impl crate::diagnostic::DiagnosticProjection for ValidationError {}
impl crate::diagnostic::DiagnosticProjection for AdapterError {}
#[cfg(test)]
mod tests;
#[cfg(test)]
mod float_collections_tests;
#[cfg(test)]
mod extension_values_tests;
