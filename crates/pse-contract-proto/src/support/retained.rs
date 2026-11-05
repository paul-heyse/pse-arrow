// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original crossing-failure custody, including owned field/rule selectors.
use super::BoundaryProjectionError;
use protovalidate_buffa::{FieldPath, Subscript};
use pse_contract::boundary_projection::{AdapterError, ValidationError};
use std::borrow::Cow;

/// A retained failure cannot be represented by the native byte allowance.
#[derive(Clone, Copy, Debug, thiserror::Error, miette::Diagnostic, PartialEq, Eq)]
#[error("crossing failure retained extent overflow")]
#[diagnostic(code("runtime.resource_limit"))]
pub struct RetainedExtentError;

fn add(a: usize, b: usize) -> Result<usize, RetainedExtentError> {
    a.checked_add(b).ok_or(RetainedExtentError)
}
fn slots<T>(capacity: usize) -> Result<usize, RetainedExtentError> {
    capacity
        .checked_mul(size_of::<T>())
        .ok_or(RetainedExtentError)
}
trait RetainedStringCapacity {
    fn owned_capacity(&self) -> usize;
}
impl RetainedStringCapacity for Cow<'_, str> {
    fn owned_capacity(&self) -> usize {
        match self {
            Cow::Borrowed(_) => 0,
            Cow::Owned(value) => value.capacity(),
        }
    }
}
fn rule(error: &ValidationError) -> Result<usize, RetainedExtentError> {
    match error {
        ValidationError::Rule { path, reason, .. } => add(path.capacity(), reason.capacity()),
    }
}
fn adapter(error: &AdapterError) -> Result<usize, RetainedExtentError> {
    match error {
        AdapterError::Refused { path, reason, .. } => add(path.capacity(), reason.capacity()),
        AdapterError::Validation(error) => rule(error),
    }
}
fn path(path: &FieldPath) -> Result<usize, RetainedExtentError> {
    path.elements.iter().try_fold(
        slots::<protovalidate_buffa::FieldPathElement>(path.elements.capacity())?,
        |bytes, element| {
            let bytes = add(
                bytes,
                element
                    .field_name
                    .as_ref()
                    .map_or(0, RetainedStringCapacity::owned_capacity),
            )?;
            add(
                bytes,
                match &element.subscript {
                    Some(Subscript::StringKey(key)) => key.owned_capacity(),
                    _ => 0,
                },
            )
        },
    )
}
fn validation(error: &protovalidate_buffa::ValidationError) -> Result<usize, RetainedExtentError> {
    let bytes = add(
        slots::<protovalidate_buffa::Violation>(error.violations.capacity())?,
        error.compile_error.as_ref().map_or(0, String::capacity),
    )?;
    let bytes = add(
        bytes,
        error.runtime_error.as_ref().map_or(0, String::capacity),
    )?;
    error.violations.iter().try_fold(bytes, |bytes, violation| {
        let bytes = add(bytes, path(&violation.field)?)?;
        let bytes = add(bytes, path(&violation.rule)?)?;
        add(
            add(bytes, violation.rule_id.owned_capacity())?,
            violation.message.owned_capacity(),
        )
    })
}
impl BoundaryProjectionError {
    /// Actual inline and owned allocation capacities, excluding shared allocation/allocator overhead.
    /// Original paths, selectors and causes stay untouched; unknown/infinite extents never become zero.
    pub fn retained_bytes(&self) -> Result<usize, RetainedExtentError> {
        add(
            size_of::<Self>(),
            match self {
                Self::Adoption(error) | Self::OutgoingAdoption(error) => adapter(error)?,
                Self::OutgoingNative(error) => rule(error)?,
                Self::WireValidation(error) | Self::OutgoingWire(error) => validation(error)?,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use protovalidate_buffa::{FieldPathElement, Violation};
    fn owned(text: &str, capacity: usize) -> Cow<'static, str> {
        let mut value = String::with_capacity(capacity);
        value.push_str(text);
        Cow::Owned(value)
    }
    #[test]
    fn cow_retained_extent_follows_owned_capacity() {
        let borrowed: Cow<'_, str> = Cow::Borrowed("original static selector");
        assert_eq!(borrowed.owned_capacity(), 0);
        let mut storage = String::with_capacity(128);
        storage.push_str("selector");
        let capacity = storage.capacity();
        let pointer = storage.as_ptr();
        let owned: Cow<'_, str> = Cow::Owned(storage);
        assert!(capacity > owned.len());
        assert_eq!(owned.owned_capacity(), capacity);
        assert_eq!(owned.as_ptr(), pointer);
    }
    #[test]
    fn retained_crossing_failure_counts_capacities_and_preserves_original_selectors() {
        let mut field = FieldPath {
            elements: Vec::with_capacity(4),
        };
        field.elements.push(FieldPathElement {
            field_name: Some(owned("map", 21)),
            subscript: Some(Subscript::StringKey(owned("actual\0key", 37))),
            ..Default::default()
        });
        let mut rule_path = FieldPath {
            elements: Vec::with_capacity(2),
        };
        rule_path.elements.push(FieldPathElement {
            field_name: Some(Cow::Borrowed("const")),
            ..Default::default()
        });
        let mut errors = protovalidate_buffa::ValidationError {
            violations: Vec::with_capacity(3),
            compile_error: Some(String::with_capacity(49)),
            runtime_error: Some(String::with_capacity(53)),
        };
        errors.violations.push(Violation {
            field,
            rule: rule_path,
            rule_id: owned("bytes.const", 29),
            message: owned("actual original failure", 79),
            for_key: true,
        });
        let expected = size_of::<BoundaryProjectionError>()
            + errors.violations.capacity() * size_of::<Violation>()
            + errors.violations[0].field.elements.capacity() * size_of::<FieldPathElement>()
            + errors.violations[0].rule.elements.capacity() * size_of::<FieldPathElement>()
            + 21
            + 37
            + 49
            + 53
            + 29
            + 79;
        let error = BoundaryProjectionError::WireValidation(errors);
        assert_eq!(error.retained_bytes().unwrap(), expected);
        assert!(
            matches!(&error.wire_validation().unwrap().violations[0].field.elements[0].subscript,
            Some(Subscript::StringKey(key)) if key == "actual\0key")
        );
        assert!(error.wire_validation().unwrap().violations[0].for_key);
        assert_eq!(add(usize::MAX, 1), Err(RetainedExtentError));
        assert_eq!(slots::<Violation>(usize::MAX), Err(RetainedExtentError));
    }
}
