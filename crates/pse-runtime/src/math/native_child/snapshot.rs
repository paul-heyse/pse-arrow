// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual received-population planning and fixed-profile restoration.
use super::*;
use pse_math::{
    AttributionTransferError, MathError,
    implicit::transfer::{self as root_transfer, RootRestoreExtents, RootTransferError},
    transfer::{self, F64TransferError, TransferError},
};
use std::convert::Infallible;

#[derive(Debug, thiserror::Error)]
enum SnapshotFailure {
    #[error("native numerical transfer: {0}")]
    Numerical(#[source] TransferError<Infallible>),
    #[error("native root transfer: {0}")]
    Root(#[source] RootTransferError<Infallible>),
}
fn numerical_code(error: &TransferError<Infallible>) -> Option<pse_diagnostics::DiagnosticCode> {
    use pse_diagnostics::{DiagnosticCode as C, TypedDiagnostic as _};
    match error {
        TransferError::Unsupported(_)
        | TransferError::Supplier(F64TransferError::Unsupported(_)) => Some(C::CapabilityBackend),
        TransferError::Overflow(_)
        | TransferError::Allocation(_)
        | TransferError::Supplier(F64TransferError::Overflow(_)) => Some(C::RuntimeResourceLimit),
        TransferError::Math(error)
        | TransferError::Layout(pse_math::jets::LayoutSnapshotError::Math(error)) => {
            error.diagnostic_code()
        }
        TransferError::Invalid(_) | TransferError::Supplier(_) | TransferError::Layout(_) => {
            Some(C::InternalInvariant)
        }
        TransferError::Admission(never) => match *never {},
    }
}
pse_diagnostics::impl_diagnostic! {
    SnapshotFailure,
    code(this) { match this {
        Self::Numerical(error) => numerical_code(error),
        Self::Root(error) => match error {
            RootTransferError::Unsupported(_) => Some(pse_diagnostics::DiagnosticCode::CapabilityBackend),
            RootTransferError::Allocation(_) | RootTransferError::Attribution(AttributionTransferError::Extent) => Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit),
            RootTransferError::Numerical(error) => numerical_code(error),
            RootTransferError::Math(error) => { use pse_diagnostics::TypedDiagnostic as _; error.diagnostic_code() },
            RootTransferError::Requirements(error) => { use pse_diagnostics::TypedDiagnostic as _; error.diagnostic_code() },
            RootTransferError::Invalid(_) | RootTransferError::Attribution(_) => Some(pse_diagnostics::DiagnosticCode::InternalInvariant),
            RootTransferError::Admission(never) => match *never {},
        }
    } },
    forward(_this) { None },
    help(_this) { None }, related(_this) { None }, source(_this) { None },
    facts(_this) { pse_diagnostics::DiagnosticFacts::default() }
}
impl DiagnosticProjection for SnapshotFailure {}
impl SnapshotFailure {
    fn into_runtime(self) -> MathRuntimeError {
        // Only fixed-backed structural/supplier/allocation variants reach here.
        // Original mathematical/provider and caller errors are forwarded below.
        MathError::Typed {
            retained: size_of::<Self>() + 2 * size_of::<usize>(),
            cause: pse_model::diagnostic::DiagnosticCause::new(self),
        }
        .into()
    }
}
fn numerical_error<E: std::fmt::Debug + std::fmt::Display>(
    error: TransferError<E>,
    admission: impl FnOnce(E) -> MathRuntimeError,
) -> MathRuntimeError {
    let error = match error {
        TransferError::Admission(error) => return admission(error),
        TransferError::Math(error)
        | TransferError::Layout(pse_math::jets::LayoutSnapshotError::Math(error)) => {
            return error.into();
        }
        TransferError::Invalid(error) => TransferError::Invalid(error),
        TransferError::Unsupported(error) => TransferError::Unsupported(error),
        TransferError::Overflow(error) => TransferError::Overflow(error),
        TransferError::Supplier(error) => TransferError::Supplier(error),
        TransferError::Layout(error) => TransferError::Layout(error),
        TransferError::Allocation(error) => TransferError::Allocation(error),
    };
    SnapshotFailure::Numerical(error).into_runtime()
}
fn root_error<E: std::fmt::Debug + std::fmt::Display>(
    error: RootTransferError<E>,
    admission: impl FnOnce(E) -> MathRuntimeError,
) -> MathRuntimeError {
    let error = match error {
        RootTransferError::Admission(error) => return admission(error),
        RootTransferError::Math(error) => return error.into(),
        RootTransferError::Requirements(error) => return MathError::Scope(error).into(),
        RootTransferError::Numerical(error) => return numerical_error(error, admission),
        RootTransferError::Invalid(error) => RootTransferError::Invalid(error),
        RootTransferError::Unsupported(error) => RootTransferError::Unsupported(error),
        RootTransferError::Attribution(error) => RootTransferError::Attribution(error),
        RootTransferError::Allocation(error) => RootTransferError::Allocation(error),
    };
    SnapshotFailure::Root(error).into_runtime()
}
pub(super) fn root_transfer_error(error: RootTransferError<MathRuntimeError>) -> MathRuntimeError {
    root_error(error, std::convert::identity)
}

pub(super) fn prepare(
    request: &Request,
    scope: &ExecutionScope,
) -> Result<(RootInput, RootRestoreExtents), MathRuntimeError> {
    checkpoint(scope)?;
    let input: RootInput = wire::decode(&request.source, REQUEST_LIMIT)?;
    if !input.spec.providers.is_empty()
        || input.execution.definition.inputs != input.parameters.len()
        || input.parameters.iter().any(|value| !value.is_finite())
    {
        return Err(contract(
            "native child closed execution or parameter correspondence differs",
        ));
    }
    let facts = input
        .execution
        .restore_extents()
        .map_err(|error| root_error(error, |never| match never {}))?;
    checkpoint(scope)?;
    Ok((input, facts))
}
pub(super) fn restore(
    input: RootInput,
    admission: Arc<admission::RestorationAdmission>,
    scope: &ExecutionScope,
) -> Result<(Arc<pse_math::implicit::RootExecution>, Vec<f64>, Options), MathRuntimeError> {
    let RootInput {
        execution,
        parameters,
        mut options,
        ..
    } = input;
    // DTO decoding/conversion was covered by the original launch admission.
    // This anchor preserves that phase's lifetime; restoration separately debits
    // the actual granted numerical and metadata populations before allocation.
    let input_owner: Arc<dyn pse_math::AllocationOwner> = admission.clone();
    let (numerical, metadata) = execution.into_restore_parts(input_owner);
    let checked = transfer::restore(numerical, scope.clone(), admission.clone(), |facts| {
        admission.charge(facts.restore_peak_bytes)
    })
    .map_err(|error| numerical_error(error, std::convert::identity))?;
    let root = root_transfer::restore(
        checked,
        metadata,
        scope.clone(),
        admission.clone(),
        |facts| {
            let bytes = facts
                .metadata_input_bytes
                .checked_add(facts.construction_bytes)
                .and_then(|value| value.checked_add(facts.validation_bytes))
                .ok_or(MathRuntimeError::Limit(
                    "native root restoration metadata extent",
                ))?;
            admission.charge(bytes)
        },
    )
    .map_err(root_transfer_error)?;
    options.time_limit = scope
        .remaining(options.time_limit)
        .map_err(MathError::Scope)?;
    root.validate_options(&options)?;
    Ok((root, parameters, options))
}
pub(super) fn restore_and_solve(
    input: RootInput,
    admission: Arc<admission::RestorationAdmission>,
    scope: &ExecutionScope,
) -> Result<Vec<f64>, MathRuntimeError> {
    let (root, parameters, options) = restore(input, admission, scope)?;
    let mut worker = pse_backend_native::implicit::Kinsol;
    let point = worker.solve_root(root, &parameters, &options, scope.cancellation_control())?;
    checkpoint(scope)?;
    Ok(point)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_model::diagnostic::BoundaryClass;
    #[test]
    fn transfer_boundary_preserves_original_admission_and_mathematical_control() {
        let original = "original pool refusal source".to_owned();
        let allocation = original.as_ptr();
        let error = root_transfer_error(RootTransferError::Admission(
            MathRuntimeError::Infrastructure(original),
        ));
        let MathRuntimeError::Infrastructure(original) = error else {
            panic!("admission cause changed");
        };
        assert_eq!(original.as_ptr(), allocation);
        let error = numerical_error::<MathRuntimeError>(
            TransferError::Math(MathError::Scope(pse_kernels::ProviderError::Deadline)),
            std::convert::identity,
        );
        assert!(matches!(
            error,
            MathRuntimeError::Math(MathError::Scope(pse_kernels::ProviderError::Deadline))
        ));
    }
    #[test]
    fn transfer_boundary_distinguishes_unsupported_resource_and_malformed_without_display_parsing()
    {
        for (original, class) in [
            (
                TransferError::<MathRuntimeError>::Unsupported("native evaluator"),
                BoundaryClass::Unsupported,
            ),
            (
                TransferError::Overflow("stack backing"),
                BoundaryClass::ResourceLimit,
            ),
            (
                TransferError::Invalid("output index"),
                BoundaryClass::Internal,
            ),
            (
                TransferError::Supplier(F64TransferError::Unsupported("external function")),
                BoundaryClass::Unsupported,
            ),
        ] {
            let error = numerical_error(original, std::convert::identity);
            assert_eq!(
                error
                    .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Native)
                    .class,
                class
            );
            let mut cause: Option<&dyn std::error::Error> = Some(&error);
            let mut found = false;
            while let Some(current) = cause {
                found |= current
                    .downcast_ref::<TransferError<Infallible>>()
                    .is_some();
                cause = current.source();
            }
            assert!(found, "original typed numerical cause was erased");
        }
        let error = root_transfer_error(RootTransferError::Attribution(
            AttributionTransferError::Extent,
        ));
        assert_eq!(
            error
                .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Native)
                .class,
            BoundaryClass::ResourceLimit
        );
    }
}
