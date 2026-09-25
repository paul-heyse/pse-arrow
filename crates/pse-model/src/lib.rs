// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Registry-generated semantic values without native execution dependencies.

/// Registry-generated semantic value declarations.
#[rustfmt::skip]
pub mod generated;
/// Exact durable artifact validity, independent of storage and incremental handles.
pub mod artifact;
/// Source-attributed public failure structure.
pub mod diagnostic;
/// Resolved numerical meaning, independent of native solver implementations.
pub mod numerics;
/// An invalid declared semantic enum member.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ModelError {
    /// A tagged value violates its declared alternatives.
    #[error("{0}")]
    Malformed(String),
    /// A wire value is outside the declared enumeration.
    #[error("field `{field}` holds `{value}`, which is not a member of `{enumeration}`")]
    EnumMember {
        /// Declared field.
        field: String,
        /// Declared enumeration.
        enumeration: String,
        /// Rejected value.
        value: String,
    },
}
pse_diagnostics::impl_diagnostic! {
    ModelError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::ValidationInvariant) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}

/// Refuse a malformed generated semantic value.
pub fn malformed(message: &str) -> ModelError {
    ModelError::Malformed(message.to_owned())
}

/// Exact declared value equality, including canonical float-bit semantics.
pub trait SemanticEq {
    /// Compare complete values, never their hash or residency.
    fn semantic_eq(&self, other: &Self) -> bool;
}
macro_rules! ordinary_equality { ($($ty:ty),*) => { $(impl SemanticEq for $ty { fn semantic_eq(&self, other: &Self) -> bool { self == other } })* }; }
ordinary_equality!(
    bool,
    String,
    i8,
    i16,
    i32,
    i64,
    i128,
    u8,
    u16,
    u32,
    u64,
    usize,
    pse_ids::SemanticId,
    pse_ids::ContentHash
);
impl SemanticEq for f64 {
    fn semantic_eq(&self, other: &Self) -> bool {
        pse_ids::canonical_f64_bits(*self) == pse_ids::canonical_f64_bits(*other)
    }
}
impl SemanticEq for f32 {
    fn semantic_eq(&self, other: &Self) -> bool {
        pse_ids::canonical_f32_bits(*self) == pse_ids::canonical_f32_bits(*other)
    }
}
impl<T: SemanticEq> SemanticEq for Option<T> {
    fn semantic_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Some(a), Some(b)) => a.semantic_eq(b),
            (None, None) => true,
            _ => false,
        }
    }
}
impl<T: SemanticEq> SemanticEq for Vec<T> {
    fn semantic_eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().zip(other).all(|(a, b)| a.semantic_eq(b))
    }
}
impl<T: SemanticEq, const N: usize> SemanticEq for [T; N] {
    fn semantic_eq(&self, other: &Self) -> bool {
        self.iter().zip(other).all(|(a, b)| a.semantic_eq(b))
    }
}

impl<K: Ord, V: SemanticEq> SemanticEq for std::collections::BTreeMap<K, V> {
    fn semantic_eq(&self, other: &Self) -> bool {
        self.len() == other.len()
            && self
                .iter()
                .zip(other)
                .all(|((ka, va), (kb, vb))| ka == kb && va.semantic_eq(vb))
    }
}

/// Logical owned allocation extent used for input admission, excluding allocator overhead.
/// Overflow saturates so finite admission limits fail closed.
pub trait HeapUsage {
    /// Retained out-of-line bytes, including collection capacity and nested payloads.
    fn heap_bytes(&self) -> usize;
    /// Inline value plus its owned out-of-line payload.
    fn owned_bytes(&self) -> usize
    where
        Self: Sized,
    {
        size_of::<Self>().saturating_add(self.heap_bytes())
    }
}
macro_rules! inline_usage { ($($ty:ty),*) => { $(impl HeapUsage for $ty { fn heap_bytes(&self) -> usize { 0 } })* }; }
inline_usage!(
    bool,
    i8,
    i16,
    i32,
    i64,
    i128,
    u8,
    u16,
    u32,
    u64,
    usize,
    f32,
    f64,
    pse_ids::SemanticId,
    pse_ids::ContentHash
);
impl HeapUsage for String {
    fn heap_bytes(&self) -> usize {
        self.capacity()
    }
}
impl<T: HeapUsage> HeapUsage for Option<T> {
    fn heap_bytes(&self) -> usize {
        self.as_ref().map_or(0, HeapUsage::heap_bytes)
    }
}
impl<T: HeapUsage> HeapUsage for Vec<T> {
    fn heap_bytes(&self) -> usize {
        self.iter()
            .fold(self.capacity().saturating_mul(size_of::<T>()), |n, v| {
                n.saturating_add(v.heap_bytes())
            })
    }
}
impl<T: HeapUsage, const N: usize> HeapUsage for [T; N] {
    fn heap_bytes(&self) -> usize {
        self.iter()
            .fold(0usize, |n, v| n.saturating_add(v.heap_bytes()))
    }
}

/// Stable typed value framing, independent of serde, Debug, allocation and process IDs.
pub trait SemanticFrame {
    /// Append the complete declared value to the shared framing owner.
    fn frame(&self, hash: &mut pse_ids::FramedHasher);
}
macro_rules! frame_integer { ($($ty:ty),*) => { $(impl SemanticFrame for $ty { fn frame(&self, hash: &mut pse_ids::FramedHasher) { hash.part(&self.to_le_bytes()); } })* }; }
frame_integer!(i8, i16, i32, i64, i128, u8, u16, u32, u64);
impl SemanticFrame for bool {
    fn frame(&self, hash: &mut pse_ids::FramedHasher) {
        hash.bool(*self);
    }
}
impl SemanticFrame for String {
    fn frame(&self, hash: &mut pse_ids::FramedHasher) {
        hash.str(self);
    }
}
impl SemanticFrame for f64 {
    fn frame(&self, hash: &mut pse_ids::FramedHasher) {
        hash.u64(pse_ids::canonical_f64_bits(*self));
    }
}
impl SemanticFrame for f32 {
    fn frame(&self, hash: &mut pse_ids::FramedHasher) {
        hash.u32(pse_ids::canonical_f32_bits(*self));
    }
}
impl SemanticFrame for pse_ids::SemanticId {
    fn frame(&self, hash: &mut pse_ids::FramedHasher) {
        hash.id(self);
    }
}
impl SemanticFrame for pse_ids::ContentHash {
    fn frame(&self, hash: &mut pse_ids::FramedHasher) {
        hash.hash(self);
    }
}
impl<T: SemanticFrame> SemanticFrame for Option<T> {
    fn frame(&self, hash: &mut pse_ids::FramedHasher) {
        hash.bool(self.is_some());
        if let Some(value) = self {
            value.frame(hash);
        }
    }
}
impl<T: SemanticFrame> SemanticFrame for Vec<T> {
    fn frame(&self, hash: &mut pse_ids::FramedHasher) {
        hash.u64(self.len() as u64);
        for value in self {
            value.frame(hash);
        }
    }
}
impl<K: Ord + SemanticFrame, V: SemanticFrame> SemanticFrame for std::collections::BTreeMap<K, V> {
    fn frame(&self, hash: &mut pse_ids::FramedHasher) {
        hash.u64(self.len() as u64);
        for (key, value) in self {
            key.frame(hash);
            value.frame(hash);
        }
    }
}
impl<T: SemanticFrame, const N: usize> SemanticFrame for [T; N] {
    fn frame(&self, hash: &mut pse_ids::FramedHasher) {
        hash.u64(N as u64);
        for value in self {
            value.frame(hash);
        }
    }
}
