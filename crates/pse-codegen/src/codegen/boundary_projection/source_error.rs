// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native source and wire metadata error composition at the generator boundary.

/// A generator refusal retaining its registry or metadata source and diagnostic.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum ProjectionCodegenError {
    /// Actual source registry refusal.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Registry(#[from] pse_schema::SchemaError),
    /// Actual shared Proto metadata refusal.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Metadata(#[from] pse_contract_proto::MetadataError),
    /// Original native projection-owner refusal.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Projection(#[from] pse_contract::boundary_projection::ProjectionError),
    /// Original Arrow conversion or purpose projection refusal.
    #[error("native field IPC: {0}")]
    #[diagnostic(code("schema.invalid_declaration"))]
    Arrow(#[from] arrow_schema::ArrowError),
    /// Original verified FlatBuffer acquisition refusal.
    #[error("native field IPC flatbuffer: {0}")]
    #[diagnostic(code("schema.invalid_declaration"))]
    Flatbuffer(#[from] flatbuffers::InvalidFlatbuffer),
    /// Original native source-only field carrier decode refusal.
    #[error("native field source: {0}")]
    #[diagnostic(code("schema.invalid_declaration"))]
    Json(#[from] serde_json::Error),
    /// Original native source association, without flattening its typed cause.
    #[error("{file}:{subject}: {source}")]
    #[diagnostic(forward(source))]
    At {
        /// Native declaration source file.
        file: String,
        /// Native owner or method identity.
        subject: String,
        /// Original typed refusal and diagnostic category.
        #[source]
        source: Box<Self>,
    },
}
impl ProjectionCodegenError {
    pub(super) fn at(self, file: impl Into<String>, subject: impl Into<String>) -> Self {
        Self::At {
            file: file.into(),
            subject: subject.into(),
            source: Box::new(self),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use miette::Diagnostic;

    #[test]
    fn source_and_metadata_categories_remain_typed_at_codegen_boundary() {
        let source = pse_schema::SchemaError::UnknownReference {
            context: "native field".into(),
            reference: "absent owner".into(),
        };
        let original_code = source.code().unwrap().to_string();
        let wrapped = ProjectionCodegenError::from(source);
        assert_eq!(wrapped.code().unwrap().to_string(), original_code);
        assert!(matches!(
            wrapped,
            ProjectionCodegenError::Registry(pse_schema::SchemaError::UnknownReference { .. })
        ));
        let source = pse_contract_proto::MetadataError::Invalid {
            reason: "selected option absent".into(),
        };
        let original_code = source.code().unwrap().to_string();
        let wrapped = ProjectionCodegenError::from(source);
        assert_eq!(wrapped.code().unwrap().to_string(), original_code);
        assert!(matches!(
            wrapped,
            ProjectionCodegenError::Metadata(pse_contract_proto::MetadataError::Invalid { .. })
        ));
    }
}
