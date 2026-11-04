// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared typed scalar/container mechanics for source-generated native boundary adapters.

#[cfg(feature = "engineering-service")]
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::Arc;

use pse_model::boundary_projection::{AdapterError, AdapterFailure, ValidateProjection};

/// A typed crossing refusal, preserving validator compilation/evaluation and rule failures.
#[derive(Debug, thiserror::Error)]
pub enum BoundaryProjectionError {
    /// Mechanical adoption or the actual owner's native local rule refused.
    #[error(transparent)]
    Adoption(#[from] AdapterError),
    /// Original generated validator result, including all field/rule paths and categories.
    #[error(transparent)]
    WireValidation(#[from] protovalidate_buffa::ValidationError),
    /// An outgoing native producer violated its shared local envelope.
    #[error("outgoing native projection: {0}")]
    OutgoingNative(#[source] pse_model::boundary_projection::ValidationError),
    /// An outgoing mapping could not preserve its actual native meaning.
    #[error("outgoing native adoption: {0}")]
    OutgoingAdoption(#[source] AdapterError),
    /// Original outgoing wire-rule failure, distinguished from uncontrolled ingress.
    #[error("outgoing wire projection: {0}")]
    OutgoingWire(#[source] protovalidate_buffa::ValidationError),
}
pse_diagnostics::impl_diagnostic! {
    BoundaryProjectionError,
    code(this) { match this {
        Self::Adoption(_) => None,
        Self::OutgoingNative(_) | Self::OutgoingAdoption(_) | Self::OutgoingWire(_) => Some(pse_diagnostics::DiagnosticCode::WorkflowInternal),
        Self::WireValidation(error) if error.compile_error.is_some() || error.runtime_error.is_some() => Some(pse_diagnostics::DiagnosticCode::WorkflowInternal),
        Self::WireValidation(_) => Some(pse_diagnostics::DiagnosticCode::WorkflowInput),
    } },
    forward(this) { match this { Self::Adoption(error) | Self::OutgoingAdoption(error) => Some(error), Self::OutgoingNative(error) => Some(error), _ => None } },
    help(_this) { None }, related(_this) { None }, source(_this) { None },
    facts(this) {
        let mut facts = pse_diagnostics::DiagnosticFacts::default();
        if let Self::WireValidation(error) | Self::OutgoingWire(error) = this {
            if let Some(detail) = &error.compile_error { facts.observe("rule_compilation_error", pse_diagnostics::DiagnosticObservation::Text(detail.clone())); }
            if let Some(detail) = &error.runtime_error { facts.observe("rule_evaluation_error", pse_diagnostics::DiagnosticObservation::Text(detail.clone())); }
            for (index, violation) in error.violations.iter().enumerate() {
                facts.observe(format!("violation_{index}_field_path"), pse_diagnostics::DiagnosticObservation::Text(violation.field.to_string()));
                facts.observe(format!("violation_{index}_rule_id"), pse_diagnostics::DiagnosticObservation::Text(violation.rule_id.to_string()));
                facts.observe(format!("violation_{index}_detail"), pse_diagnostics::DiagnosticObservation::Text(violation.message.to_string()));
            }
        }
        facts
    }
}
impl pse_model::diagnostic::DiagnosticProjection for BoundaryProjectionError {}

pub(crate) trait NativeInto<W> {
    fn into_wire(self, path: &str) -> Result<W, AdapterError>;
}
pub(crate) trait NativeFrom<W>: Sized {
    fn from_wire(wire: W, path: &str) -> Result<Self, AdapterError>;
}

pub(crate) fn native_into<N: NativeInto<W>, W>(value: N, path: &str) -> Result<W, AdapterError> {
    value.into_wire(path)
}
pub(crate) fn native_from<N: NativeFrom<W>, W>(value: W, path: &str) -> Result<N, AdapterError> {
    N::from_wire(value, path)
}
pub(crate) fn child_path(parent: &str, field: &str) -> String {
    pse_model::boundary_projection::field_path(parent, field)
}
pub(crate) fn refused(path: &str, kind: AdapterFailure, reason: &str) -> AdapterError {
    AdapterError::refused(path, kind, reason)
}
#[cfg(feature = "engineering-service")]
pub(crate) fn unknown_fields(
    fields: &buffa::UnknownFields,
    path: &str,
) -> Result<(), AdapterError> {
    if fields.is_empty() {
        Ok(())
    } else {
        Err(refused(
            path,
            AdapterFailure::UnknownField,
            "binary fields are retained by Buffa but this native owner cannot adopt their unknown meaning",
        ))
    }
}
pub(crate) fn layout(
    actual_version: u32,
    actual: &[(u32, &str)],
    version: u32,
    expected: &[(u32, &str)],
    path: &str,
) -> Result<(), AdapterError> {
    if actual_version == version && actual == expected {
        Ok(())
    } else {
        Err(refused(
            path,
            AdapterFailure::LayoutMismatch,
            "regenerate adapters from the actual owner before crossing this interpretation",
        ))
    }
}
pub(crate) fn required<T>(value: Option<T>, path: &str) -> Result<T, AdapterError> {
    value.ok_or_else(|| {
        refused(
            path,
            AdapterFailure::InvalidValue,
            "required projected value is absent",
        )
    })
}

pub(crate) fn encode_root<
    N: NativeInto<W> + ValidateProjection,
    W: protovalidate_buffa::Validate,
>(
    native: N,
    path: &str,
) -> Result<W, BoundaryProjectionError> {
    native
        .validate_projection()
        .map_err(BoundaryProjectionError::OutgoingNative)?;
    let wire = native_into(native, path).map_err(BoundaryProjectionError::OutgoingAdoption)?;
    wire.validate()
        .map_err(BoundaryProjectionError::OutgoingWire)?;
    Ok(wire)
}

pub(crate) fn decode_root<
    N: NativeFrom<W> + ValidateProjection,
    W: protovalidate_buffa::Validate,
>(
    wire: W,
    path: &str,
) -> Result<N, BoundaryProjectionError> {
    wire.validate()?;
    let native: N = native_from(wire, path)?;
    native.validate_projection().map_err(AdapterError::from)?;
    Ok(native)
}

macro_rules! scalar_identity { ($($ty:ty),* $(,)?) => { $(
    impl NativeInto<$ty> for $ty { fn into_wire(self, _: &str) -> Result<$ty, AdapterError> { Ok(self) } }
    impl NativeFrom<$ty> for $ty { fn from_wire(wire: $ty, _: &str) -> Result<Self, AdapterError> { Ok(wire) } }
)* }; }
scalar_identity!(bool, String, u32, u64, i32, i64, Vec<u8>);
impl NativeInto<u32> for u8 {
    fn into_wire(self, _: &str) -> Result<u32, AdapterError> {
        Ok(self.into())
    }
}
impl NativeFrom<u32> for u8 {
    fn from_wire(wire: u32, path: &str) -> Result<Self, AdapterError> {
        Self::try_from(wire).map_err(|_| {
            refused(
                path,
                AdapterFailure::InvalidValue,
                "uint32 does not fit the actual octet",
            )
        })
    }
}
impl NativeInto<u32> for u16 {
    fn into_wire(self, _: &str) -> Result<u32, AdapterError> { Ok(self.into()) }
}
impl NativeFrom<u32> for u16 {
    fn from_wire(wire: u32, path: &str) -> Result<Self, AdapterError> {
        Self::try_from(wire).map_err(|_| refused(path, AdapterFailure::InvalidValue,
            "uint32 does not fit the actual unsigned 16-bit component"))
    }
}
impl NativeInto<i32> for i16 {
    fn into_wire(self, _: &str) -> Result<i32, AdapterError> {
        Ok(self.into())
    }
}
impl NativeFrom<i32> for i16 {
    fn from_wire(wire: i32, path: &str) -> Result<Self, AdapterError> {
        Self::try_from(wire).map_err(|_| {
            refused(
                path,
                AdapterFailure::InvalidValue,
                "int32 does not fit the actual signed 16-bit component",
            )
        })
    }
}
impl NativeInto<u64> for usize {
    fn into_wire(self, path: &str) -> Result<u64, AdapterError> {
        u64::try_from(self).map_err(|_| {
            refused(
                path,
                AdapterFailure::InvalidValue,
                "native count exceeds uint64",
            )
        })
    }
}
impl NativeFrom<u64> for usize {
    fn from_wire(wire: u64, path: &str) -> Result<Self, AdapterError> {
        Self::try_from(wire).map_err(|_| {
            refused(
                path,
                AdapterFailure::InvalidValue,
                "uint64 does not fit the native count",
            )
        })
    }
}
impl NativeInto<u64> for f64 {
    fn into_wire(self, _: &str) -> Result<u64, AdapterError> {
        Ok(self.to_bits())
    }
}
impl NativeFrom<u64> for f64 {
    fn from_wire(wire: u64, _: &str) -> Result<Self, AdapterError> {
        Ok(Self::from_bits(wire))
    }
}
impl NativeInto<u32> for f32 {
    fn into_wire(self, _: &str) -> Result<u32, AdapterError> {
        Ok(self.to_bits())
    }
}
impl NativeFrom<u32> for f32 {
    fn from_wire(wire: u32, _: &str) -> Result<Self, AdapterError> {
        Ok(Self::from_bits(wire))
    }
}
impl NativeInto<Vec<u8>> for pse_model::Bytes {
    fn into_wire(self, _: &str) -> Result<Vec<u8>, AdapterError> {
        Ok(self.into_vec())
    }
}
impl NativeFrom<Vec<u8>> for pse_model::Bytes {
    fn from_wire(wire: Vec<u8>, _: &str) -> Result<Self, AdapterError> {
        Ok(Self::new(wire))
    }
}
macro_rules! identity_bytes { ($($ty:ty),* $(,)?) => { $(
    impl NativeInto<Vec<u8>> for $ty { fn into_wire(self, _: &str) -> Result<Vec<u8>, AdapterError> { Ok(self.as_bytes().to_vec()) } }
    impl NativeFrom<Vec<u8>> for $ty {
        fn from_wire(wire: Vec<u8>, path: &str) -> Result<Self, AdapterError> {
            let bytes = wire.try_into().map_err(|_| refused(path, AdapterFailure::InvalidValue, "identity byte width does not match its actual nominal owner"))?;
            Ok(Self::from_bytes(bytes))
        }
    }
)* }; }
identity_bytes!(pse_ids::SemanticId, pse_ids::ContentHash);
impl<T: NativeInto<W>, W> NativeInto<W> for Box<T> {
    fn into_wire(self, path: &str) -> Result<W, AdapterError> {
        (*self).into_wire(path)
    }
}
impl<T: NativeFrom<W>, W> NativeFrom<W> for Box<T> {
    fn from_wire(wire: W, path: &str) -> Result<Self, AdapterError> {
        T::from_wire(wire, path).map(Box::new)
    }
}

pub(crate) trait ProjectionItems {
    type Item;
    type Items: Iterator<Item = Self::Item>;
    fn projection_items(self) -> Self::Items;
}
impl<T> ProjectionItems for Vec<T> {
    type Item = T;
    type Items = std::vec::IntoIter<T>;
    fn projection_items(self) -> Self::Items {
        self.into_iter()
    }
}
impl<T: Clone> ProjectionItems for Arc<[T]> {
    type Item = T;
    type Items = std::vec::IntoIter<T>;
    fn projection_items(self) -> Self::Items {
        self.to_vec().into_iter()
    }
}
impl<T> ProjectionItems for BTreeSet<T> {
    type Item = T;
    type Items = std::collections::btree_set::IntoIter<T>;
    fn projection_items(self) -> Self::Items {
        self.into_iter()
    }
}
impl<T: ProjectionItems> ProjectionItems for Box<T> {
    type Item = T::Item;
    type Items = T::Items;
    fn projection_items(self) -> Self::Items {
        (*self).projection_items()
    }
}
pub(crate) fn items<T: ProjectionItems>(value: T) -> T::Items {
    value.projection_items()
}

pub(crate) trait FromProjectionItems<T>: Sized {
    fn from_items(items: Vec<T>, path: &str) -> Result<Self, AdapterError>;
}
impl<T> FromProjectionItems<T> for Vec<T> {
    fn from_items(items: Vec<T>, _: &str) -> Result<Self, AdapterError> {
        Ok(items)
    }
}
impl<T> FromProjectionItems<T> for Arc<[T]> {
    fn from_items(items: Vec<T>, _: &str) -> Result<Self, AdapterError> {
        Ok(items.into())
    }
}
impl<T: Ord> FromProjectionItems<T> for BTreeSet<T> {
    fn from_items(items: Vec<T>, path: &str) -> Result<Self, AdapterError> {
        let mut values = Self::new();
        for value in items {
            if !values.insert(value) {
                return Err(refused(
                    path,
                    AdapterFailure::DuplicateKey,
                    "duplicate projected set member",
                ));
            }
        }
        Ok(values)
    }
}
impl<T, C: FromProjectionItems<T>> FromProjectionItems<T> for Box<C> {
    fn from_items(items: Vec<T>, path: &str) -> Result<Self, AdapterError> {
        C::from_items(items, path).map(Box::new)
    }
}
pub(crate) fn from_items<T, C: FromProjectionItems<T>>(
    items: Vec<T>,
    path: &str,
) -> Result<C, AdapterError> {
    C::from_items(items, path)
}

#[cfg(feature = "engineering-service")]
pub(crate) fn unique_pairs<K: Ord, V>(
    pairs: Vec<(K, V)>,
    path: &str,
) -> Result<BTreeMap<K, V>, AdapterError> {
    let mut map = BTreeMap::new();
    for (key, value) in pairs {
        if map.insert(key, value).is_some() {
            return Err(refused(
                path,
                AdapterFailure::DuplicateKey,
                "duplicate projected map key",
            ));
        }
    }
    Ok(map)
}

#[cfg(feature = "engineering-service")]
pub(crate) fn wire_map<K, V, M: FromIterator<(K, V)>>(pairs: impl Iterator<Item = (K, V)>) -> M {
    pairs.collect()
}
