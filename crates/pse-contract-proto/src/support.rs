// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Standard typed mechanics for source-generated adapters; no product/native owner dependency.
use pse_contract::boundary_projection::{AdapterError, AdapterFailure, ValidateProjection};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
mod borrowed;
mod retained;
mod signed_integer;
pub use borrowed::{decode_view_root, unknown_fields_view};
pub use retained::RetainedExtentError;

/// Recognize typed resource refusals from the maintained Buffa decode profile.
/// This observes the original cause; it grants no decode or native admission.
pub fn decode_resource_refusal(error: &buffa::DecodeError) -> bool {
    matches!(
        error,
        buffa::DecodeError::MessageTooLarge
            | buffa::DecodeError::RecursionLimitExceeded
            | buffa::DecodeError::UnknownFieldLimitExceeded
            | buffa::DecodeError::ElementMemoryLimitExceeded
            | buffa::DecodeError::AllocationLimitExceeded
            | buffa::DecodeError::AllocationLayoutOverflow
            | buffa::DecodeError::AllocationReserveFailed
    )
}

/// Original typed crossing failure; native permission and consumer presentation stay external.
#[derive(Debug, thiserror::Error)]
pub enum BoundaryProjectionError {
    /// Actual native mechanical/local rule refusal.
    #[error("{0}")]
    Adoption(#[from] AdapterError),
    /// Original generated validation failure, with all violations and infrastructure detail.
    #[error("{0}")]
    WireValidation(#[from] protovalidate_buffa::ValidationError),
    /// Original outgoing native producer envelope violation.
    #[error("outgoing native projection: {0}")]
    OutgoingNative(#[source] pse_contract::boundary_projection::ValidationError),
    /// Original outgoing source correspondence failure.
    #[error("outgoing native adoption: {0}")]
    OutgoingAdoption(#[source] AdapterError),
    /// Original outgoing generated validation failure.
    #[error("outgoing wire projection: {0}")]
    OutgoingWire(#[source] protovalidate_buffa::ValidationError),
}
impl miette::Diagnostic for BoundaryProjectionError {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        let code = match self {
            Self::Adoption(error) => return miette::Diagnostic::code(error),
            Self::OutgoingNative(_) | Self::OutgoingAdoption(_) | Self::OutgoingWire(_) => {
                "workflow.internal"
            }
            Self::WireValidation(error)
                if error.compile_error.is_some() || error.runtime_error.is_some() =>
            {
                "workflow.internal"
            }
            Self::WireValidation(_) => "workflow.input",
        };
        Some(Box::new(code))
    }
    fn diagnostic_source(&self) -> Option<&dyn miette::Diagnostic> {
        match self {
            Self::Adoption(error) | Self::OutgoingAdoption(error) => Some(error),
            Self::OutgoingNative(error) => Some(error),
            _ => None,
        }
    }
}
impl BoundaryProjectionError {
    /// Original maintained-validator details, never flattened into a rendered message.
    pub fn wire_validation(&self) -> Option<&protovalidate_buffa::ValidationError> {
        match self {
            Self::WireValidation(error) | Self::OutgoingWire(error) => Some(error),
            _ => None,
        }
    }
}

/// Mechanical native-to-wire correspondence; root validation remains explicit.
/// A local scope permits the crossing owner to pair types from two dependencies.
pub trait NativeInto<W, Scope = ()> {
    /// Convert the actual selected native value.
    fn into_wire(self, path: &str) -> Result<W, AdapterError>;
}
/// Checked mechanical wire-to-native correspondence, excluding sealed observations.
pub trait NativeFrom<W, Scope = ()>: Sized {
    /// Construct only this actual declared data value.
    fn from_wire(wire: W, path: &str) -> Result<Self, AdapterError>;
}

/// Apply one actual native mechanical conversion.
pub fn native_into<N: NativeInto<W>, W>(value: N, path: &str) -> Result<W, AdapterError> {
    native_into_in_scope::<N, W, ()>(value, path)
}
/// Apply the crossing owner's mechanical conversion.
pub fn native_into_in_scope<N: NativeInto<W, Scope>, W, Scope>(
    value: N,
    path: &str,
) -> Result<W, AdapterError> {
    value.into_wire(path)
}
/// Apply one checked actual native inverse.
pub fn native_from<N: NativeFrom<W>, W>(value: W, path: &str) -> Result<N, AdapterError> {
    native_from_in_scope::<N, W, ()>(value, path)
}
/// Apply the crossing owner's checked mechanical inverse.
pub fn native_from_in_scope<N: NativeFrom<W, Scope>, W, Scope>(
    value: W,
    path: &str,
) -> Result<N, AdapterError> {
    N::from_wire(value, path)
}
/// Attribute a child field using the shared native path convention.
pub fn child_path(parent: &str, field: &str) -> String {
    pse_contract::boundary_projection::field_path(parent, field)
}
/// Retain a typed native mechanical refusal.
pub fn refused(path: &str, kind: AdapterFailure, reason: &str) -> AdapterError {
    AdapterError::refused(path, kind, reason)
}
/// Refuse adopting binary meaning absent from the actual native owner.
pub fn unknown_fields(fields: &buffa::UnknownFields, path: &str) -> Result<(), AdapterError> {
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
/// Require the actual owner version and stable tuple layout.
pub fn layout(
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
/// Preserve required presence rather than fabricating a default.
pub fn required<T>(value: Option<T>, path: &str) -> Result<T, AdapterError> {
    value.ok_or_else(|| {
        refused(
            path,
            AdapterFailure::InvalidValue,
            "required projected value is absent",
        )
    })
}

/// Validate the native producer, mechanically convert, then validate outgoing wire.
pub fn encode_root<N: NativeInto<W> + ValidateProjection, W: protovalidate_buffa::Validate>(
    native: N,
    path: &str,
) -> Result<W, BoundaryProjectionError> {
    encode_root_in_scope::<N, W, ()>(native, path)
}

/// Validate and convert an outgoing root under its crossing owner's scope.
pub fn encode_root_in_scope<N, W, Scope>(native: N, path: &str) -> Result<W, BoundaryProjectionError>
where
    N: NativeInto<W, Scope> + ValidateProjection,
    W: protovalidate_buffa::Validate,
{
    native
        .validate_projection()
        .map_err(BoundaryProjectionError::OutgoingNative)?;
    let wire = native_into_in_scope::<N, W, Scope>(native, path)
        .map_err(BoundaryProjectionError::OutgoingAdoption)?;
    wire.validate()
        .map_err(BoundaryProjectionError::OutgoingWire)?;
    Ok(wire)
}

/// Validate uncontrolled wire, construct actual data, then validate native rules.
pub fn decode_root<N: NativeFrom<W> + ValidateProjection, W: protovalidate_buffa::Validate>(
    wire: W,
    path: &str,
) -> Result<N, BoundaryProjectionError> {
    decode_root_in_scope::<N, W, ()>(wire, path)
}

/// Validate and adopt an incoming root under its crossing owner's scope.
pub fn decode_root_in_scope<N, W, Scope>(wire: W, path: &str) -> Result<N, BoundaryProjectionError>
where
    N: NativeFrom<W, Scope> + ValidateProjection,
    W: protovalidate_buffa::Validate,
{
    wire.validate()?;
    let native = native_from_in_scope::<N, W, Scope>(wire, path)?;
    native.validate_projection().map_err(AdapterError::from)?;
    Ok(native)
}

macro_rules! scalar_identity { ($($ty:ty),* $(,)?) => { $(
    impl NativeInto<$ty> for $ty { fn into_wire(self, _: &str) -> Result<$ty, AdapterError> { Ok(self) } }
    impl NativeFrom<$ty> for $ty { fn from_wire(wire: $ty, _: &str) -> Result<Self, AdapterError> { Ok(wire) } }
)* }; }
scalar_identity!(bool, String, u32, u64, i32, i64);
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
    fn into_wire(self, _: &str) -> Result<u32, AdapterError> {
        Ok(self.into())
    }
}
impl NativeFrom<u32> for u16 {
    fn from_wire(wire: u32, path: &str) -> Result<Self, AdapterError> {
        Self::try_from(wire).map_err(|_| {
            refused(
                path,
                AdapterFailure::InvalidValue,
                "uint32 does not fit the actual unsigned 16-bit component",
            )
        })
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
impl<T: NativeInto<W, Scope>, W, Scope> NativeInto<W, Scope> for Box<T> {
    fn into_wire(self, path: &str) -> Result<W, AdapterError> {
        (*self).into_wire(path)
    }
}
impl<T: NativeFrom<W, Scope>, W, Scope> NativeFrom<W, Scope> for Box<T> {
    fn from_wire(wire: W, path: &str) -> Result<Self, AdapterError> {
        T::from_wire(wire, path).map(Box::new)
    }
}

/// Source container extraction, retaining declared order/uniqueness semantics.
pub trait ProjectionItems {
    /// Original native element type.
    type Item;
    /// Original extraction iterator.
    type Items: Iterator<Item = Self::Item>;
    /// Consume the selected native collection.
    fn projection_items(self) -> Self::Items;
}
impl<T> ProjectionItems for Vec<T> {
    type Item = T;
    type Items = std::vec::IntoIter<T>;
    fn projection_items(self) -> Self::Items {
        self.into_iter()
    }
}
/// Owning source-order iteration over a shared slice, cloning only yielded values.
#[derive(Debug)]
pub struct ArcProjectionItems<T> {
    values: Arc<[T]>,
    remaining: std::ops::Range<usize>,
}
impl<T: Clone> Iterator for ArcProjectionItems<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.remaining
            .next()
            .map(|index| self.values[index].clone())
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.remaining.size_hint()
    }
}
impl<T: Clone> ProjectionItems for Arc<[T]> {
    type Item = T;
    type Items = ArcProjectionItems<T>;
    fn projection_items(self) -> Self::Items {
        ArcProjectionItems {
            remaining: 0..self.len(),
            values: self,
        }
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
/// Iterate the original container through its actual owner.
pub fn items<T: ProjectionItems>(value: T) -> T::Items {
    value.projection_items()
}

/// Checked construction of the actual selected native container.
pub trait FromProjectionItems<T>: Sized {
    /// Construct the native container under its checked contract.
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
/// Construct the actual native collection without silently dropping duplicates.
pub fn from_items<T, C: FromProjectionItems<T>>(
    items: Vec<T>,
    path: &str,
) -> Result<C, AdapterError> {
    C::from_items(items, path)
}

/// Construct a native map while refusing duplicate projected keys.
pub fn unique_pairs<K: Ord, V>(
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

/// Build the declared wire map from original typed pairs.
pub fn wire_map<K, V, M: FromIterator<(K, V)>>(pairs: impl Iterator<Item = (K, V)>) -> M {
    pairs.collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use miette::Diagnostic;

    #[test]
    fn decode_resource_refusals_cover_the_maintained_profile_without_reclassifying_wire_failures() {
        for error in [
            buffa::DecodeError::MessageTooLarge,
            buffa::DecodeError::RecursionLimitExceeded,
            buffa::DecodeError::UnknownFieldLimitExceeded,
            buffa::DecodeError::ElementMemoryLimitExceeded,
            buffa::DecodeError::AllocationLimitExceeded,
            buffa::DecodeError::AllocationLayoutOverflow,
            buffa::DecodeError::AllocationReserveFailed,
        ] {
            assert!(decode_resource_refusal(&error), "{error:?}");
        }
        for error in [
            buffa::DecodeError::UnexpectedEof,
            buffa::DecodeError::InvalidUtf8,
            buffa::DecodeError::UnaccountedView,
        ] {
            assert!(!decode_resource_refusal(&error), "{error:?}");
        }
    }

    #[test]
    fn arc_projection_items_clone_on_demand_and_retain_source() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        #[derive(Debug)]
        struct Tracked {
            value: usize,
            clones: Arc<AtomicUsize>,
            drops: Arc<AtomicUsize>,
        }
        impl Clone for Tracked {
            fn clone(&self) -> Self {
                self.clones.fetch_add(1, Ordering::Relaxed);
                Self {
                    value: self.value,
                    clones: self.clones.clone(),
                    drops: self.drops.clone(),
                }
            }
        }
        impl Drop for Tracked {
            fn drop(&mut self) {
                self.drops.fetch_add(1, Ordering::Relaxed);
            }
        }
        for consumed in [1, 3] {
            let clones = Arc::new(AtomicUsize::new(0));
            let drops = Arc::new(AtomicUsize::new(0));
            let source: Arc<[Tracked]> = (0..3)
                .map(|value| Tracked {
                    value,
                    clones: clones.clone(),
                    drops: drops.clone(),
                })
                .collect();
            let weak = Arc::downgrade(&source);
            let mut iterator = items(source);
            assert_eq!(clones.load(Ordering::Relaxed), 0);
            assert_eq!(iterator.size_hint(), (3, Some(3)));
            assert!(weak.upgrade().is_some());
            for expected in 0..consumed {
                let value = iterator.next().unwrap();
                assert_eq!(value.value, expected);
                assert_eq!(clones.load(Ordering::Relaxed), expected + 1);
                assert_eq!(iterator.size_hint(), (2 - expected, Some(2 - expected)));
            }
            if consumed == 3 {
                assert!(iterator.next().is_none());
                assert!(iterator.next().is_none());
            }
            assert_eq!(drops.load(Ordering::Relaxed), consumed);
            assert!(weak.upgrade().is_some());
            drop(iterator);
            assert!(weak.upgrade().is_none());
            assert_eq!(clones.load(Ordering::Relaxed), consumed);
            assert_eq!(drops.load(Ordering::Relaxed), 3 + consumed);
        }
    }

    #[test]
    fn original_rule_infrastructure_and_outgoing_failures_remain_distinct() {
        let ordinary = BoundaryProjectionError::WireValidation(
            protovalidate_buffa::ValidationError::default(),
        );
        assert_eq!(ordinary.code().unwrap().to_string(), "workflow.input");
        let original = ordinary.wire_validation().unwrap();
        let cause = std::error::Error::source(&ordinary)
            .unwrap()
            .downcast_ref::<protovalidate_buffa::ValidationError>()
            .unwrap();
        assert!(std::ptr::eq(original, cause));
        for error in [
            BoundaryProjectionError::WireValidation(protovalidate_buffa::ValidationError {
                compile_error: Some("original rule compile refusal".into()),
                ..Default::default()
            }),
            BoundaryProjectionError::WireValidation(protovalidate_buffa::ValidationError {
                runtime_error: Some("original rule precondition refusal".into()),
                ..Default::default()
            }),
            BoundaryProjectionError::OutgoingWire(protovalidate_buffa::ValidationError::default()),
        ] {
            assert_eq!(error.code().unwrap().to_string(), "workflow.internal");
            assert!(error.wire_validation().is_some());
        }
    }

    #[test]
    fn actual_scalar_bits_widths_and_collection_uniqueness_are_preserved() {
        for bits in [0_u64, (-0.0_f64).to_bits(), 0x7ff8_0000_0000_0042] {
            let native = native_from::<f64, u64>(bits, "value").unwrap();
            assert_eq!(native_into::<f64, u64>(native, "value").unwrap(), bits);
        }
        assert!(native_from::<u8, u32>(256, "value.octet").is_err());
        assert!(matches!(
            from_items::<_, BTreeSet<u64>>(vec![1, 1], "values"),
            Err(AdapterError::Refused {
                kind: AdapterFailure::DuplicateKey,
                ..
            })
        ));
        assert!(matches!(
            unique_pairs(vec![(1, "first"), (1, "second")], "pairs"),
            Err(AdapterError::Refused {
                kind: AdapterFailure::DuplicateKey,
                ..
            })
        ));
    }
}
