// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native metadata failures, without a product or registry error dependency.

/// One retained domain diagnostic whose standard source chain exposes the actual
/// native value rather than stopping at the erased diagnostic box.
#[derive(Debug, miette::Diagnostic)]
#[diagnostic(forward(inner))]
pub struct DomainDiagnostic {
    inner: Box<dyn miette::Diagnostic + Send + Sync>,
}
impl std::fmt::Display for DomainDiagnostic {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(formatter)
    }
}
impl std::error::Error for DomainDiagnostic {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.inner.as_ref())
    }
}

/// A structural acquisition, schema, codec or explicitly selected domain refusal.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum MetadataError {
    /// Original schema or interpretation cannot be understood by this reader.
    #[error("contract metadata interpretation unavailable: {reason}")]
    #[diagnostic(code("schema.version_mismatch"))]
    Unavailable {
        /// Attributed unsupported meaning, without adopting a default.
        reason: String,
    },
    /// A selected world or schema cannot represent the required meaning.
    #[error("invalid contract metadata: {reason}")]
    #[diagnostic(code("workflow.input"))]
    Invalid {
        /// Specific unsupported or inconsistent fact.
        reason: String,
    },
    /// Original descriptor linker failure.
    #[error("contract metadata pool: {source}")]
    #[diagnostic(code("workflow.input"))]
    Pool {
        /// Original library error.
        #[from]
        source: buffa_descriptor::PoolError,
    },
    /// Original lossless decoder failure.
    #[error("contract metadata decoding: {source}")]
    #[diagnostic(code("workflow.input"))]
    Decode {
        /// Original library error.
        #[from]
        source: buffa::DecodeError,
    },
    /// Original fallible encoding failure.
    #[error("contract metadata encoding: {source}")]
    #[diagnostic(code("workflow.input"))]
    Encode {
        /// Original library error.
        #[from]
        source: buffa::EncodeError,
    },
    /// Original checked reflection mutation failure.
    #[error("contract metadata reflection: {source}")]
    #[diagnostic(code("workflow.input"))]
    Reflection {
        /// Original library error.
        #[from]
        source: buffa_descriptor::ReflectError,
    },
    /// Actual acquired source association, without an invented character span.
    #[error("{file}:{subject}: {source}")]
    #[diagnostic(forward(source))]
    At {
        /// Original descriptor filename.
        file: String,
        /// Original selected declaration or field.
        subject: String,
        /// Original native refusal, retaining its classification.
        #[source]
        source: Box<Self>,
    },
    /// Declared schema/member context when no physical source coordinate exists.
    #[error("{subject}: {source}")]
    #[diagnostic(forward(source))]
    Context {
        /// Typed schema or member identity.
        subject: String,
        /// Original native refusal.
        #[source]
        source: Box<Self>,
    },
    /// An explicit external owner policy refused its original typed value.
    #[error("domain metadata policy {policy}: {source}")]
    #[diagnostic(forward(source))]
    Domain {
        /// Fully qualified selected extension/policy.
        policy: String,
        /// Original domain diagnostic; no product dependency or string erasure.
        #[source]
        source: DomainDiagnostic,
    },
}

impl MetadataError {
    /// Whether this refusal retains an unavailable interpretation rather than malformed input.
    pub fn is_unavailable(&self) -> bool {
        match self {
            Self::Unavailable { .. } => true,
            Self::At { source, .. } | Self::Context { source, .. } => source.is_unavailable(),
            _ => false,
        }
    }
    /// Preserve an actual file/subject association around a typed native error.
    pub fn at(self, file: impl Into<String>, subject: impl Into<String>) -> Self {
        Self::At {
            file: file.into(),
            subject: subject.into(),
            source: Box::new(self),
        }
    }
    /// Add logical schema/member context without claiming a source coordinate.
    pub fn context(self, subject: impl Into<String>) -> Self {
        Self::Context {
            subject: subject.into(),
            source: Box::new(self),
        }
    }
    /// Retain the original typed diagnostic supplied by an external domain owner.
    pub fn domain(
        policy: impl Into<String>,
        source: impl miette::Diagnostic + Send + Sync + 'static,
    ) -> Self {
        Self::Domain {
            policy: policy.into(),
            source: DomainDiagnostic {
                inner: Box::new(source),
            },
        }
    }
}
