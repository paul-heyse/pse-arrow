// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The one Python document-content boundary; workers retain these owned bytes.
use pyo3::prelude::*;
use pyo3::type_hint_union;
use pyo3::types::{PyBytes, PyString};

pub(crate) struct DocumentContent(pub(crate) Vec<u8>);

impl FromPyObject<'_, '_> for DocumentContent {
    type Error = PyErr;

    const INPUT_TYPE: pyo3::inspect::PyStaticExpr = pyo3::type_hint_union!(
        pyo3::type_hint_identifier!("builtins", "str"),
        pyo3::type_hint_identifier!("builtins", "bytes")
    );

    fn extract(value: Borrowed<'_, '_, PyAny>) -> PyResult<Self> {
        if let Ok(text) = value.cast::<PyString>() {
            return Ok(Self(text.to_str()?.as_bytes().to_vec()));
        }
        if let Ok(bytes) = value.cast::<PyBytes>() {
            return Ok(Self(bytes.as_bytes().to_vec()));
        }
        Err(pyo3::exceptions::PyTypeError::new_err(
            "document content must be str or bytes",
        ))
    }
}

/// Decode an owned operation document exactly once, before native contextual admission.
pub(crate) fn decode<T: serde::de::DeserializeOwned>(
    py: Python<'_>,
    operation: &str,
    bytes: &[u8],
    allowance: usize,
) -> PyResult<T> {
    if bytes.len() > allowance {
        return Err(crate::workflow::invalid(
            py,
            format!("{operation} document extent"),
        ));
    }
    serde_json::from_slice(bytes)
        .map_err(|error| crate::workflow::invalid(py, format!("{operation}: {error}")))
}
/// Decode a registered standalone owner using its catalogue-selected actual envelope.
pub(crate) fn decode_document<T: serde::de::DeserializeOwned + DocumentType>(
    py: Python<'_>,
    operation: &str,
    bytes: &[u8],
    allowance: usize,
) -> PyResult<T> {
    if bytes.len() > allowance {
        return Err(crate::workflow::invalid(
            py,
            format!("{operation} document extent"),
        ));
    }
    if let Some(version) = T::VERSION {
        let field = match T::WIRE {
            pse_runtime::boundary_documents::Wire::Native => "version",
            pse_runtime::boundary_documents::Wire::FlatV1
            | pse_runtime::boundary_documents::Wire::ValueV1 => "document_version",
        };
        pse_runtime::boundary_documents::check_required_header(bytes, field, version)
            .map_err(|error| crate::workflow::invalid(py, format!("{operation}: {error}")))?;
    }
    T::WIRE
        .decode_slice(bytes)
        .map_err(|error| crate::workflow::invalid(py, format!("{operation}: {error}")))
}

/// Shared native header admission for Python's original JSON slices.
#[pyfunction]
pub(crate) fn check_document_header(bytes: &[u8], field: &str, expected: u32) -> PyResult<()> {
    let field = match field {
        "version" => "version",
        "document_version" => "document_version",
        _ => {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "unknown document version header",
            ));
        }
    };
    pse_runtime::boundary_documents::check_required_header(bytes, field, expected)
        .map_err(|error| pyo3::exceptions::PyValueError::new_err(error.to_string()))
}

/// Admit interpretation before decoding nested current document fields.
pub(crate) fn decode_versioned<T: serde::de::DeserializeOwned, const N: u32>(
    py: Python<'_>,
    operation: &str,
    bytes: &[u8],
    allowance: usize,
) -> PyResult<T> {
    if bytes.len() > allowance {
        return Err(crate::workflow::invalid(
            py,
            format!("{operation} document extent"),
        ));
    }
    pse_model::document::decode_versioned::<T, N>(bytes)
        .map_err(|error| crate::workflow::invalid(py, format!("{operation}: {error}")))
}
/// Infer current interpretation from the document's declared version field.
pub(crate) fn decode_current<T: serde::de::DeserializeOwned, const N: u32>(
    py: Python<'_>,
    operation: &str,
    bytes: &[u8],
    allowance: usize,
    _version: fn(&T) -> &pse_model::document::Version<N>,
) -> PyResult<T> {
    decode_versioned::<T, N>(py, operation, bytes, allowance)
}
/// Read the immutable study contract through its owning version-first admission.
pub(crate) fn study_definition(
    py: Python<'_>,
    bytes: &[u8],
    allowance: usize,
) -> PyResult<pse_runtime::workflow::StudyDefinition> {
    if bytes.len() > allowance {
        return Err(crate::workflow::invalid(py, "operation document extent"));
    }
    let document = std::str::from_utf8(bytes)
        .map_err(|error| crate::workflow::invalid(py, format!("operation: {error}")))?;
    pse_runtime::workflow::StudyDefinition::readmission(document)
        .map_err(|error| crate::inspection::errors::diagnostic(py, &error))
}
/// Omission is resolved by the Rust operation's default implementation.
pub(crate) fn controls<T: serde::de::DeserializeOwned + Default + DocumentType>(
    py: Python<'_>,
    operation: &str,
    bytes: Option<&[u8]>,
    allowance: usize,
) -> PyResult<T> {
    bytes.map_or_else(
        || Ok(T::default()),
        |bytes| decode_document(py, operation, bytes, allowance),
    )
}

/// A generated Python document projected from the actual Rust serde owner.
#[derive(Clone, Debug)]
pub(crate) struct DocumentValue<T>(pub(crate) T);
pub(crate) trait DocumentType {
    const INPUT_NAME: &'static str;
    const OUTPUT_NAME: &'static str;
    const DISTINCT_DIRECTIONS: bool;
    const VERSION: Option<u32>;
    const WIRE: pse_runtime::boundary_documents::Wire;
}
#[path = "generated/documents.rs"]
#[rustfmt::skip]
mod catalogue;
// These are explicit owning serializable projections, rather than new catalogue roots.
macro_rules! projection {
    ($owner:ty => $document:ty) => {
        impl $crate::documents::DocumentType for $owner {
            const INPUT_NAME: &'static str =
                <$document as $crate::documents::DocumentType>::INPUT_NAME;
            const OUTPUT_NAME: &'static str =
                <$document as $crate::documents::DocumentType>::OUTPUT_NAME;
            const DISTINCT_DIRECTIONS: bool =
                <$document as $crate::documents::DocumentType>::DISTINCT_DIRECTIONS;
            const VERSION: Option<u32> = <$document as $crate::documents::DocumentType>::VERSION;
            const WIRE: pse_runtime::boundary_documents::Wire =
                <$document as $crate::documents::DocumentType>::WIRE;
        }
    };
}
pub(crate) use projection;
projection!(pse_runtime::workflow::ModelingSourceMap => pse_runtime::workflow::ModelingSourceMapDocument);
projection!(pse_runtime::workflow::ReviewContext => pse_runtime::workflow::ReviewContextDocument);
projection!(pse_runtime::invocation::InvocationEvidence => pse_runtime::invocation::InvocationEvidenceDocument);
projection!(pse_runtime::workflow::AttemptSummary => pse_runtime::workflow::AttemptSummaryDocument);
projection!(pse_runtime::tooling::review_analysis::Analysis => pse_runtime::tooling::review_analysis::AnalysisDocument);
projection!(crate::workflow::StoredCompletionDocument => pse_runtime::workflow::Completion);
projection!(pse_backend_native::solve::SubmittedStartRef<'_> => pse_backend_native::solve::SubmittedStartRecord);

impl<T: DocumentType> DocumentType for std::sync::Arc<T> {
    const INPUT_NAME: &'static str = T::INPUT_NAME;
    const OUTPUT_NAME: &'static str = T::OUTPUT_NAME;
    const DISTINCT_DIRECTIONS: bool = T::DISTINCT_DIRECTIONS;
    const VERSION: Option<u32> = T::VERSION;
    const WIRE: pse_runtime::boundary_documents::Wire = T::WIRE;
}
impl<T: DocumentType> DocumentType for &T {
    const INPUT_NAME: &'static str = T::INPUT_NAME;
    const OUTPUT_NAME: &'static str = T::OUTPUT_NAME;
    const DISTINCT_DIRECTIONS: bool = T::DISTINCT_DIRECTIONS;
    const VERSION: Option<u32> = T::VERSION;
    const WIRE: pse_runtime::boundary_documents::Wire = T::WIRE;
}
fn require_document<T: DocumentType>(value: Borrowed<'_, '_, PyAny>) -> PyResult<()> {
    let module = value.py().import("pse.contracts.documents")?;
    let input = module.getattr(T::INPUT_NAME)?;
    if value.is_instance(&input)? {
        return Ok(());
    }
    if T::DISTINCT_DIRECTIONS && value.is_instance(&module.getattr(T::OUTPUT_NAME)?)? {
        return Ok(());
    }
    Err(pyo3::exceptions::PyTypeError::new_err(format!(
        "expected {} document",
        T::INPUT_NAME
    )))
}
impl<'py, T: serde::Serialize + DocumentType> IntoPyObject<'py> for DocumentValue<T> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;
    const OUTPUT_TYPE: pyo3::inspect::PyStaticExpr =
        pyo3::type_hint_identifier!("pse.contracts.documents", T::OUTPUT_NAME);
    fn into_pyobject(self, py: Python<'py>) -> PyResult<Self::Output> {
        let bytes = T::WIRE
            .to_vec(&self.0)
            .map_err(|error| crate::workflow::invalid(py, error.to_string()))?;
        let target = py
            .import("pse.contracts.documents")?
            .getattr(T::OUTPUT_NAME)?;
        py.import("pse.codec")?
            .call_method1("decode_json", (PyBytes::new(py, &bytes), target))
    }
}

/// A generated Python input document decoded into its actual Rust owner.
pub(crate) struct DocumentInput<T>(pub(crate) T);
impl<T: serde::de::DeserializeOwned + DocumentType> FromPyObject<'_, '_> for DocumentInput<T> {
    type Error = PyErr;
    const INPUT_TYPE: pyo3::inspect::PyStaticExpr = if T::DISTINCT_DIRECTIONS {
        pyo3::type_hint_union!(
            pyo3::type_hint_identifier!("pse.contracts.documents", T::INPUT_NAME),
            pyo3::type_hint_identifier!("pse.contracts.documents", T::OUTPUT_NAME)
        )
    } else {
        pyo3::type_hint_identifier!("pse.contracts.documents", T::INPUT_NAME)
    };
    fn extract(value: Borrowed<'_, '_, PyAny>) -> PyResult<Self> {
        let py = value.py();
        require_document::<T>(value)?;
        let encoded = py
            .import("pse.codec")?
            .call_method1("encode_json", (value,))?;
        let bytes = encoded.cast::<PyBytes>()?;
        let bytes = bytes.as_bytes();
        decode_document(py, T::INPUT_NAME, bytes, 1 << 20).map(Self)
    }
}

/// Version-first factual ingress, with no runtime/admitted-owner deserialization.
pub(crate) trait RecordedDocument: DocumentType + serde::de::DeserializeOwned {
    fn validate_recorded(&self) -> Result<(), pse_runtime::observation::ObservationError> {
        Ok(())
    }
}
pub(crate) struct RecordedInput<T>(pub(crate) T);
impl<T: RecordedDocument> FromPyObject<'_, '_> for RecordedInput<T> {
    type Error = PyErr;
    const INPUT_TYPE: pyo3::inspect::PyStaticExpr = if T::DISTINCT_DIRECTIONS {
        pyo3::type_hint_union!(
            pyo3::type_hint_identifier!("pse.contracts.documents", T::INPUT_NAME),
            pyo3::type_hint_identifier!("pse.contracts.documents", T::OUTPUT_NAME)
        )
    } else {
        pyo3::type_hint_identifier!("pse.contracts.documents", T::INPUT_NAME)
    };
    fn extract(value: Borrowed<'_, '_, PyAny>) -> PyResult<Self> {
        let py = value.py();
        require_document::<T>(value)?;
        let encoded = py
            .import("pse.codec")?
            .call_method1("encode_json", (value,))?;
        let bytes = encoded.cast::<PyBytes>()?;
        let bytes = bytes.as_bytes();
        if bytes.len() > pse_runtime::tooling::frozen_inspection::MAXIMUM_DOCUMENT_BYTES {
            return Err(crate::workflow::invalid(py, "recorded document extent"));
        }
        let document: T = decode_document(
            py,
            T::INPUT_NAME,
            bytes,
            pse_runtime::tooling::frozen_inspection::MAXIMUM_DOCUMENT_BYTES,
        )?;
        document
            .validate_recorded()
            .map_err(|error| crate::inspection::errors::diagnostic(py, &error))?;
        Ok(Self(document))
    }
}
impl RecordedDocument for pse_runtime::tooling::frozen_inspection::FrozenEvidenceSet {
    fn validate_recorded(&self) -> Result<(), pse_runtime::observation::ObservationError> {
        self.validate()
    }
}
impl RecordedDocument for pse_runtime::tooling::frozen_inspection::FrozenInspectionRequest {}
impl RecordedDocument for pse_runtime::tooling::frozen_inspection::FrozenInspectionDocument {}
impl RecordedDocument for pse_runtime::workflow::ReviewContextDocument {}
impl RecordedDocument for pse_runtime::workflow::ReviewInventorySelection {}
impl RecordedDocument for pse_runtime::workflow::ReviewInventoryPage {}
impl RecordedDocument for pse_runtime::invocation::InvocationEvidenceDocument {
    fn validate_recorded(&self) -> Result<(), pse_runtime::observation::ObservationError> {
        self.validate()
    }
}

impl RecordedDocument for pse_runtime::tooling::review_analysis::AnalysisRequest {}

impl RecordedDocument for pse_runtime::tooling::review_analysis::AnalysisDocument {
    fn validate_recorded(&self) -> Result<(), pse_runtime::observation::ObservationError> {
        self.validate()
    }
}

impl RecordedDocument for pse_runtime::tooling::review_pack::PackSelection {}

impl RecordedDocument for pse_runtime::tooling::review_pack::PackIndex {}

impl RecordedDocument for pse_runtime::tooling::review_query::ReviewQueryRequest {}

impl RecordedDocument for pse_runtime::tooling::review_query::ReviewQueryDocument {}

impl RecordedDocument for pse_runtime::tooling::review_query::ReviewQuerySpecification {}

impl RecordedDocument for pse_runtime::tooling::capture::SourceRecordingRequest {}

impl RecordedDocument for pse_runtime::tooling::capture::SourceRecordingAcknowledgement {}

impl RecordedDocument for pse_runtime::tooling::review_pack::PackExportRequest {}

impl RecordedDocument for pse_runtime::workflow::DiagnosticExecutionRequest {}
impl RecordedDocument for pse_runtime::workflow::DiagnosticReproducer {}
impl RecordedDocument for pse_runtime::workflow::ReproducerExtractionRequest {}

impl RecordedDocument for pse_runtime::tooling::review_query::ReviewTableRequest {}

impl RecordedDocument for pse_runtime::tooling::capture::ReviewHandoffReceipt {}
