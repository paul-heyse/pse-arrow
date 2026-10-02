// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Mechanical projection of source-owned vocabulary into its generated Python enum.
use pyo3::prelude::*;
pub(crate) trait RegistryEnum {
    const NAME: &'static str;
    fn wire(&self) -> &'static str;
}
pub(crate) struct EnumValue<T>(pub(crate) T);
impl<T> From<T> for EnumValue<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}
impl<'py, T: RegistryEnum> IntoPyObject<'py> for EnumValue<T> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;
    const OUTPUT_TYPE: pyo3::inspect::PyStaticExpr =
        pyo3::type_hint_identifier!("pse.contracts.enums", T::NAME);
    fn into_pyobject(self, py: Python<'py>) -> PyResult<Self::Output> {
        py.import("pse.contracts.enums")?
            .getattr(T::NAME)?
            .call1((self.0.wire(),))
    }
}
macro_rules! vocabularies {
    ($($name:ident),* $(,)?) => {$(
        impl RegistryEnum for pse_model::generated::enums::$name {
            const NAME: &'static str = stringify!($name);
            fn wire(&self) -> &'static str { self.as_str() }
        }
    )*};
}
vocabularies!(
    NativeBackend,
    NativeTermination,
    NativeQualification,
    NativeRunState,
    NativeMetricKind,
    EvidenceUnavailableReason,
    NativeIneligibility,
    NativeProblemClass,
    NativeConstraintForm,
    ModelingStructuralRequirement,
    DiagnosticCode,
    FailureClass,
    NativeBoundaryClass,
    DiagnosticSeverity,
    DiagnosticStage,
    DiagnosticRule,
    ModelingElasticObservation,
    TrajectoryTermination,
    ModelingDiagnosticSampleStop,
    ModelingInitializationStep
);
