// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded structural V1 traversal prerequisite; hashes/data never grant authority.
//!
//! This module is test-only until ADR-0245's compiler association and full receiving
//! admission are implemented. Bounds below cover this traversal's buffers, including
//! its tiny pinned-library normalization helpers. Decoded roots, caller Serialize
//! implementations/stacks, receiving validation and scientific reconstruction remain
//! separate admission obligations. No universal arbitrary-Serde equivalence is claimed.
use crate::{ContentHash, Frame, FramedHasher};
use serde::{Serialize, ser};
use std::{borrow::Cow, cell::Cell, collections::TryReserveError, fmt, ops::Range};

const RAW_SENTINEL: &str = "$serde_json::private::RawValue";
const NUMBER_SENTINEL: &str = "$serde_json::private::Number";

#[derive(Debug, thiserror::Error)]
pub(super) enum Error {
    #[error("structural membership allowance exhausted: {0}")]
    Limit(&'static str),
    #[error("unsupported structural membership source profile: {0}")]
    Profile(&'static str),
    #[error("native serialization changed between count and hash")]
    ChangedShape,
    #[error("structural membership serialization: {0}")]
    Json(#[from] serde_json::Error),
    #[error("structural membership allocation: {0}")]
    Allocation(#[from] TryReserveError),
    #[error("structural membership serialization: {0}")]
    Custom(String),
}
pse_diagnostics::impl_diagnostic! {
    Error,
    code(this) { Some(match this {
        Self::Limit(_) | Self::Allocation(_) => pse_diagnostics::DiagnosticCode::RuntimeResourceLimit,
        Self::ChangedShape => pse_diagnostics::DiagnosticCode::InternalInvariant,
        Self::Profile(_) | Self::Json(_) | Self::Custom(_) => pse_diagnostics::DiagnosticCode::ValidationInvariant,
    }) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}
impl ser::Error for Error {
    fn custom<T: fmt::Display>(message: T) -> Self {
        let mut output = Text::new(1024);
        match fmt::write(&mut output, format_args!("{message}")) {
            Ok(()) => Self::Custom(output.value),
            Err(_) => Self::Limit("serializer error text"),
        }
    }
}
type Result<T> = std::result::Result<T, Error>;

/// Logical source/query limits, not a workspace lease or a scientific permit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Limits {
    pub nodes: usize,
    pub key_bytes: usize,
    pub helper_bytes: usize,
    pub depth: usize,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Emissions {
    nodes: usize,
    edges: usize,
    entries: usize,
    key_bytes: usize,
    depth: usize,
}
#[derive(Clone, Copy, Debug)]
pub(super) struct Counts {
    limits: Limits,
    actual: Emissions,
}
#[derive(Debug)]
pub(super) struct Index {
    pub root: ContentHash,
    pub nodes: Vec<ContentHash>,
}

fn add(a: usize, b: usize) -> Result<usize> {
    a.checked_add(b).ok_or(Error::Limit("extent overflow"))
}
fn mul(a: usize, b: usize) -> Result<usize> {
    a.checked_mul(b).ok_or(Error::Limit("extent overflow"))
}
fn admitted(required: usize, available: usize) -> Result<()> {
    if required > available { Err(Error::Limit("preadmitted scratch bytes")) } else { Ok(()) }
}

impl Limits {
    /// Pre-admit this before count or query normalization. Only one helper is live.
    pub(super) fn count_scratch_bytes(self) -> Result<usize> {
        // One map-to-null normalization object. The pinned std BTree node holds
        // eleven entries; this also covers IndexMap's one-entry initial tables.
        // Include headers/alignment, bounded Display text and the copied key. This
        // prices logical library buffers, not allocator-private overhead or RSS.
        let fixed = add(mul(12, size_of::<(String, serde_json::Value)>())?, mul(32, size_of::<usize>())?)?;
        add(add(fixed, mul(2, self.helper_bytes)?)?, 1024)
    }
}
impl Counts {
    /// Conservative simultaneous traversal buffers; no byte-to-root multiplier.
    pub(super) fn index_scratch_bytes(self) -> Result<usize> {
        let events = mul(self.actual.nodes, size_of::<ContentHash>())?;
        let dead = mul(mul(2, self.actual.entries)?, size_of::<Range<usize>>())?;
        let arrays = mul(mul(2, self.actual.edges)?, size_of::<Summary>())?;
        let objects = mul(mul(2, self.actual.entries)?, size_of::<Entry>())?;
        add(add(add(add(events, dead)?, arrays)?, objects)?,
            add(self.actual.key_bytes, self.limits.count_scratch_bytes()?)?)
    }
    /// A concrete query uses child/key buffers but never the global event/index log.
    pub(super) fn query_scratch_bytes(self) -> Result<usize> {
        let arrays = mul(mul(2, self.actual.edges)?, size_of::<Summary>())?;
        let objects = mul(mul(2, self.actual.entries)?, size_of::<Entry>())?;
        add(add(arrays, objects)?, add(self.actual.key_bytes, self.limits.count_scratch_bytes()?)?)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode { Count, Index, Query }
#[derive(Clone, Copy, Debug)]
struct Summary { digest: ContentHash, start: usize, end: usize }
#[derive(Debug)]
struct Entry { key: Cow<'static, str>, value: Summary, ordinal: usize }
struct State {
    mode: Mode,
    limits: Limits,
    bounds: Option<Emissions>,
    actual: Emissions,
    binding: ContentHash,
    events: Vec<ContentHash>,
    dead: Vec<Range<usize>>,
}
impl State {
    fn new(mode: Mode, limits: Limits, binding: ContentHash, bounds: Option<Emissions>) -> Self {
        Self { mode, limits, bounds, binding, actual: Emissions::default(), events: Vec::new(), dead: Vec::new() }
    }
    fn visit<T: ?Sized + Serialize>(&mut self, value: &T, depth: usize) -> Result<Summary> {
        if depth > self.limits.depth { return Err(Error::Limit("native serialization depth")); }
        self.actual.depth = self.actual.depth.max(depth);
        value.serialize(Visitor { state: self, depth })
    }
    fn edge(&mut self, object: bool) -> Result<()> {
        self.actual.edges = add(self.actual.edges, 1)?;
        if object { self.actual.entries = add(self.actual.entries, 1)?; }
        let bound = self.bounds.map_or(self.limits.nodes, |v| v.edges);
        if self.actual.edges > bound { return Err(Error::Limit("child summaries")); }
        if let Some(bound) = self.bounds {
            if self.actual.entries > bound.entries { return Err(Error::Limit("object summaries")); }
        }
        Ok(())
    }
    fn key(&mut self, key: &str) -> Result<()> {
        self.actual.key_bytes = add(self.actual.key_bytes, key.len())?;
        let bound = self.bounds.map_or(self.limits.key_bytes, |v| v.key_bytes);
        if self.actual.key_bytes > bound { return Err(Error::Limit("rendered key bytes")); }
        Ok(())
    }
    fn emit(&mut self, start: usize, body: impl FnOnce(&mut FramedHasher)) -> Result<Summary> {
        self.actual.nodes = add(self.actual.nodes, 1)?;
        let bound = self.bounds.map_or(self.limits.nodes, |v| v.nodes);
        if self.actual.nodes > bound { return Err(Error::Limit("normalized nodes")); }
        let digest = if self.mode == Mode::Count { ContentHash::from_bytes([0; 32]) } else {
            let mut hash = FramedHasher::new(Frame::MathReplayAuthorityV1);
            hash.str("record").hash(&self.binding);
            body(&mut hash);
            hash.finish_hash()
        };
        if self.mode == Mode::Index { self.events.push(digest); }
        Ok(Summary { digest, start, end: self.actual.nodes })
    }
    fn null(&mut self) -> Result<Summary> {
        self.emit(self.actual.nodes, |h| { h.str("null"); })
    }
    fn string(&mut self, value: &str) -> Result<Summary> {
        self.emit(self.actual.nodes, |h| { h.str("string").str(value); })
    }
    fn number<T: Serialize>(&mut self, value: T) -> Result<Summary> {
        // Only numeric primitives reach this helper. The pinned public conversion
        // supplies f32 widening, signed zero, nonfinite/null and integer fallibility.
        match serde_json::to_value(value)? {
            serde_json::Value::Number(number) => {
                let text = number.to_string();
                self.emit(self.actual.nodes, |h| { h.str("number").str(&text); })
            }
            serde_json::Value::Null => self.null(),
            _ => Err(Error::Profile("numeric primitive normalization")),
        }
    }
    fn wrapper(&mut self, start: usize, variant: &'static str, child: Summary) -> Result<Summary> {
        self.key(variant)?;
        self.edge(true)?;
        self.emit(start, |h| { h.str("object").u64(1).str(variant).hash(&child.digest); })
    }
}

fn supported_profile() -> Result<()> {
    if serde_json::to_value(u128::MAX).is_ok() {
        return Err(Error::Profile("arbitrary_precision"));
    }
    Ok(())
}
pub(super) fn count<T: ?Sized + Serialize>(value: &T, limits: Limits, helper_scratch: usize) -> Result<Counts> {
    admitted(limits.count_scratch_bytes()?, helper_scratch)?;
    supported_profile()?;
    let mut state = State::new(Mode::Count, limits, ContentHash::from_bytes([0; 32]), None);
    state.visit(value, 0)?;
    Ok(Counts { limits, actual: state.actual })
}
pub(super) fn build_index<T: ?Sized + Serialize>(value: &T, binding: ContentHash, counts: Counts, scratch: usize) -> Result<Index> {
    admitted(counts.index_scratch_bytes()?, scratch)?;
    supported_profile()?;
    let mut state = State::new(Mode::Index, counts.limits, binding, Some(counts.actual));
    state.events.try_reserve_exact(counts.actual.nodes)?;
    let root = state.visit(value, 0)?;
    if state.actual != counts.actual { return Err(Error::ChangedShape); }
    state.dead.sort_unstable_by_key(|range| (range.start, range.end));
    let mut merged = 0;
    for read in 0..state.dead.len() {
        let range = state.dead[read].clone();
        if merged > 0 && range.start <= state.dead[merged - 1].end {
            state.dead[merged - 1].end = state.dead[merged - 1].end.max(range.end);
        } else {
            state.dead[merged] = range;
            merged += 1;
        }
    }
    state.dead.truncate(merged);
    let mut position = 0;
    let mut discarded = 0;
    state.events.retain(|_| {
        while discarded < state.dead.len() && position >= state.dead[discarded].end { discarded += 1; }
        let live = discarded == state.dead.len() || position < state.dead[discarded].start;
        position += 1;
        live
    });
    // Reuse the event allocation as the final index: no hidden boxed-slice/Arc copy.
    state.events.sort_unstable();
    state.events.dedup();
    Ok(Index { root: root.digest, nodes: state.events })
}
pub(super) fn query_hash<T: ?Sized + Serialize>(value: &T, binding: ContentHash, counts: Counts, scratch: usize) -> Result<ContentHash> {
    admitted(counts.query_scratch_bytes()?, scratch)?;
    supported_profile()?;
    let mut state = State::new(Mode::Query, counts.limits, binding, Some(counts.actual));
    Ok(state.visit(value, 0)?.digest)
}

/// Grow only this concrete buffer, by at most doubling actual emitted elements.
/// No length hint can trigger allocation. The caller admitted twice counted slots.
fn push<T>(values: &mut Vec<T>, value: T) -> Result<()> {
    if values.len() == values.capacity() {
        let target = values.capacity().checked_mul(2).ok_or(Error::Limit("buffer extent overflow"))?.max(1);
        let additional = target.checked_sub(values.len()).ok_or(Error::Limit("buffer extent"))?;
        values.try_reserve_exact(additional)?;
    }
    values.push(value);
    Ok(())
}

struct Visitor<'a> { state: &'a mut State, depth: usize }
struct Sequence<'a> { state: &'a mut State, depth: usize, start: usize, length: usize, children: Vec<Summary>, variant: Option<&'static str> }
struct Object<'a> { state: &'a mut State, depth: usize, start: usize, entries: Vec<Entry>, pending: Option<Cow<'static, str>>, ordinal: usize, variant: Option<&'static str> }

impl Sequence<'_> {
    fn element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.state.edge(false)?;
        self.length = add(self.length, 1)?;
        let value = self.state.visit(value, add(self.depth, 1)?)?;
        if self.state.mode != Mode::Count { push(&mut self.children, value)?; }
        Ok(())
    }
    fn finish(self) -> Result<Summary> {
        let child = self.state.emit(self.start, |h| {
            h.str("array").u64(self.length as u64);
            for value in &self.children { h.hash(&value.digest); }
        })?;
        match self.variant { Some(variant) => self.state.wrapper(self.start, variant, child), None => Ok(child) }
    }
}
impl Object<'_> {
    fn field<T: ?Sized + Serialize>(&mut self, key: Cow<'static, str>, value: &T) -> Result<()> {
        self.state.edge(true)?;
        // Count never retains ancestor keys/child buffers while descending.
        let key = if self.state.mode == Mode::Count { drop(key); Cow::Borrowed("") } else { key };
        let value = self.state.visit(value, add(self.depth, 1)?)?;
        if self.state.mode != Mode::Count {
            push(&mut self.entries, Entry { key, value, ordinal: self.ordinal })?;
        }
        self.ordinal = add(self.ordinal, 1)?;
        Ok(())
    }
    fn finish(mut self) -> Result<Summary> {
        // Sorting emission ordinal last preserves last rendered key wins, including
        // differently typed native keys that the pinned library renders equally.
        self.entries.sort_unstable_by(|a, b| a.key.cmp(&b.key).then(a.ordinal.cmp(&b.ordinal)));
        let mut live = 0;
        for read in 0..self.entries.len() {
            if read + 1 < self.entries.len() && self.entries[read].key == self.entries[read + 1].key {
                if self.state.mode == Mode::Index {
                    let value = self.entries[read].value;
                    push(&mut self.state.dead, value.start..value.end)?;
                }
            } else {
                self.entries.swap(live, read);
                live += 1;
            }
        }
        let child = self.state.emit(self.start, |h| {
            h.str("object").u64(live as u64);
            for entry in &self.entries[..live] { h.str(&entry.key).hash(&entry.value.digest); }
        })?;
        match self.variant { Some(variant) => self.state.wrapper(self.start, variant, child), None => Ok(child) }
    }
}

macro_rules! number_methods { ($($name:ident($ty:ty)),* $(,)?) => { $(
    fn $name(self, value: $ty) -> Result<Summary> { self.state.number(value) }
)* }; }
impl<'a> ser::Serializer for Visitor<'a> {
    type Ok = Summary;
    type Error = Error;
    type SerializeSeq = Sequence<'a>;
    type SerializeTuple = Sequence<'a>;
    type SerializeTupleStruct = Sequence<'a>;
    type SerializeTupleVariant = Sequence<'a>;
    type SerializeMap = Object<'a>;
    type SerializeStruct = Object<'a>;
    type SerializeStructVariant = Object<'a>;
    number_methods!(serialize_i8(i8), serialize_i16(i16), serialize_i32(i32), serialize_i64(i64),
        serialize_u8(u8), serialize_u16(u16), serialize_u32(u32), serialize_u64(u64), serialize_f32(f32), serialize_f64(f64));
    fn serialize_i128(self, value: i128) -> Result<Summary> {
        if value < i128::from(i64::MIN) || value > i128::from(u64::MAX) { return Err(Error::Profile("i128 outside combined i64/u64 range")); }
        self.state.number(value)
    }
    fn serialize_u128(self, value: u128) -> Result<Summary> {
        if value > u128::from(u64::MAX) { return Err(Error::Profile("u128 outside u64 range")); }
        self.state.number(value)
    }
    fn serialize_bool(self, value: bool) -> Result<Summary> { self.state.emit(self.state.actual.nodes, |h| { h.str("bool").bool(value); }) }
    fn serialize_str(self, value: &str) -> Result<Summary> { self.state.string(value) }
    fn serialize_char(self, value: char) -> Result<Summary> { self.state.string(value.encode_utf8(&mut [0; 4])) }
    fn serialize_bytes(self, value: &[u8]) -> Result<Summary> {
        let mut seq = self.serialize_seq(None)?;
        for byte in value { seq.element(byte)?; }
        seq.finish()
    }
    fn serialize_none(self) -> Result<Summary> { self.state.null() }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Summary> { self.state.visit(value, add(self.depth, 1)?) }
    fn serialize_unit(self) -> Result<Summary> { self.state.null() }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Summary> { self.state.null() }
    fn serialize_unit_variant(self, _name: &'static str, _tag: u32, value: &'static str) -> Result<Summary> { self.state.string(value) }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(self, name: &'static str, value: &T) -> Result<Summary> {
        sentinel(name)?;
        self.state.visit(value, add(self.depth, 1)?)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(self, _name: &'static str, _tag: u32, variant: &'static str, value: &T) -> Result<Summary> {
        let start = self.state.actual.nodes;
        let child = self.state.visit(value, add(self.depth, 1)?)?;
        self.state.wrapper(start, variant, child)
    }
    fn serialize_seq(self, _hint: Option<usize>) -> Result<Sequence<'a>> {
        let start = self.state.actual.nodes;
        Ok(Sequence { state: self.state, depth: self.depth, start, length: 0, children: Vec::new(), variant: None })
    }
    fn serialize_tuple(self, _hint: usize) -> Result<Sequence<'a>> { self.serialize_seq(None) }
    fn serialize_tuple_struct(self, _name: &'static str, _hint: usize) -> Result<Sequence<'a>> { self.serialize_seq(None) }
    fn serialize_tuple_variant(self, _name: &'static str, _tag: u32, variant: &'static str, _hint: usize) -> Result<Sequence<'a>> {
        let mut seq = self.serialize_seq(None)?;
        seq.variant = Some(variant);
        Ok(seq)
    }
    fn serialize_map(self, _hint: Option<usize>) -> Result<Object<'a>> {
        let start = self.state.actual.nodes;
        Ok(Object { state: self.state, depth: self.depth, start, entries: Vec::new(), pending: None, ordinal: 0, variant: None })
    }
    fn serialize_struct(self, name: &'static str, _hint: usize) -> Result<Object<'a>> { sentinel(name)?; self.serialize_map(None) }
    fn serialize_struct_variant(self, _name: &'static str, _tag: u32, variant: &'static str, _hint: usize) -> Result<Object<'a>> {
        let mut object = self.serialize_map(None)?;
        object.variant = Some(variant);
        Ok(object)
    }
    fn collect_str<T: ?Sized + fmt::Display>(self, value: &T) -> Result<Summary> {
        let mut text = Text::new(self.state.limits.helper_bytes);
        fmt::write(&mut text, format_args!("{value}")).map_err(|_| Error::Limit("formatted scalar bytes"))?;
        self.state.string(&text.value)
    }
    fn is_human_readable(&self) -> bool { true }
}
fn sentinel(name: &str) -> Result<()> {
    if matches!(name, RAW_SENTINEL | NUMBER_SENTINEL) { Err(Error::Profile("private raw/number sentinel")) } else { Ok(()) }
}

macro_rules! sequence_impl { ($trait:ident, $method:ident) => {
    impl ser::$trait for Sequence<'_> {
        type Ok = Summary; type Error = Error;
        fn $method<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> { self.element(value) }
        fn end(self) -> Result<Summary> { self.finish() }
    }
}; }
sequence_impl!(SerializeSeq, serialize_element);
sequence_impl!(SerializeTuple, serialize_element);
sequence_impl!(SerializeTupleStruct, serialize_field);
sequence_impl!(SerializeTupleVariant, serialize_field);
impl ser::SerializeMap for Object<'_> {
    type Ok = Summary; type Error = Error;
    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<()> {
        let key = normalize_key(key, self.state.limits.helper_bytes, self.state.limits.depth)?;
        self.state.key(&key)?;
        self.pending = Some(if self.state.mode == Mode::Count { drop(key); Cow::Borrowed("") } else { Cow::Owned(key) });
        Ok(())
    }
    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        let key = self.pending.take().ok_or(Error::Profile("map value without key"))?;
        self.field(key, value)
    }
    fn end(self) -> Result<Summary> { self.finish() }
}
macro_rules! object_impl { ($trait:ident) => {
    impl ser::$trait for Object<'_> {
        type Ok = Summary; type Error = Error;
        fn serialize_field<T: ?Sized + Serialize>(&mut self, key: &'static str, value: &T) -> Result<()> {
            self.state.key(key)?;
            self.field(Cow::Borrowed(key), value)
        }
        fn end(self) -> Result<Summary> { self.finish() }
    }
}; }
object_impl!(SerializeStruct);
object_impl!(SerializeStructVariant);

/// Bounded Display formatting, including custom collect_str. Allocation is capped
/// before reserve; the formatter never builds an unbounded intermediate String.
struct Text { value: String, limit: usize }
impl Text { fn new(limit: usize) -> Self { Self { value: String::new(), limit } } }
impl fmt::Write for Text {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let total = self.value.len().checked_add(text.len()).filter(|n| *n <= self.limit).ok_or(fmt::Error)?;
        if total > self.value.capacity() {
            let target = self.value.capacity().saturating_mul(2).max(total).min(self.limit);
            self.value.try_reserve_exact(target - self.value.len()).map_err(|_| fmt::Error)?;
        }
        self.value.push_str(text);
        Ok(())
    }
}

/// The actual pinned library owns accepted key forms, number formatting and errors.
/// The adapter caps strings/collect_str before that library may copy them. It does
/// not translate key types or invent a second key serialization format.
struct Key<'a, T: ?Sized> { value: &'a T, limit: usize, depth: usize, exhausted: &'a Cell<bool> }
struct KeySerializer<'a, S> { inner: S, limit: usize, depth: usize, exhausted: &'a Cell<bool> }
impl<T: ?Sized + Serialize> Serialize for Key<'_, T> {
    fn serialize<S: ser::Serializer>(&self, inner: S) -> std::result::Result<S::Ok, S::Error> {
        if self.depth == 0 { self.exhausted.set(true); return Err(ser::Error::custom("map key depth allowance")); }
        self.value.serialize(KeySerializer { inner, limit: self.limit, depth: self.depth, exhausted: self.exhausted })
    }
}
struct OneKey<'a, T: ?Sized>(&'a Key<'a, T>);
impl<T: ?Sized + Serialize> Serialize for OneKey<'_, T> {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(1))?;
        ser::SerializeMap::serialize_entry(&mut map, self.0, &())?;
        ser::SerializeMap::end(map)
    }
}
fn normalize_key<T: ?Sized + Serialize>(value: &T, limit: usize, depth: usize) -> Result<String> {
    let exhausted = Cell::new(false);
    let key = Key { value, limit, depth, exhausted: &exhausted };
    let normalized = serde_json::to_value(OneKey(&key));
    if exhausted.get() { return Err(Error::Limit("map key normalization")); }
    let serde_json::Value::Object(map) = normalized? else { return Err(Error::Profile("map key normalization")); };
    let key = map.into_iter().next().map(|(key, _)| key).ok_or(Error::Profile("map key normalization"))?;
    // Primitive numeric keys are fixed-size tiny normalizations but must still fit
    // the selected rendered-key profile, independently of integer VALUE support.
    if key.len() > limit { return Err(Error::Limit("map key normalization")); }
    Ok(key)
}
macro_rules! key_methods { ($($name:ident($ty:ty)),* $(,)?) => { $(
    fn $name(self, value: $ty) -> std::result::Result<Self::Ok, Self::Error> { self.inner.$name(value) }
)* }; }
impl<S: ser::Serializer> ser::Serializer for KeySerializer<'_, S> {
    type Ok = S::Ok; type Error = S::Error;
    type SerializeSeq = S::SerializeSeq; type SerializeTuple = S::SerializeTuple;
    type SerializeTupleStruct = S::SerializeTupleStruct; type SerializeTupleVariant = S::SerializeTupleVariant;
    type SerializeMap = S::SerializeMap; type SerializeStruct = S::SerializeStruct; type SerializeStructVariant = S::SerializeStructVariant;
    key_methods!(serialize_i8(i8), serialize_i16(i16), serialize_i32(i32), serialize_i64(i64), serialize_i128(i128),
        serialize_u8(u8), serialize_u16(u16), serialize_u32(u32), serialize_u64(u64), serialize_u128(u128),
        serialize_f32(f32), serialize_f64(f64), serialize_bool(bool));
    fn serialize_str(self, value: &str) -> std::result::Result<Self::Ok, Self::Error> {
        if value.len() > self.limit { self.exhausted.set(true); return Err(ser::Error::custom("map key byte allowance")); }
        self.inner.serialize_str(value)
    }
    fn serialize_char(self, value: char) -> std::result::Result<Self::Ok, Self::Error> {
        if value.len_utf8() > self.limit { self.exhausted.set(true); return Err(ser::Error::custom("map key byte allowance")); }
        self.inner.serialize_char(value)
    }
    fn serialize_bytes(self, value: &[u8]) -> std::result::Result<Self::Ok, Self::Error> { self.inner.serialize_bytes(value) }
    fn serialize_none(self) -> std::result::Result<Self::Ok, Self::Error> { self.inner.serialize_none() }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> std::result::Result<Self::Ok, Self::Error> { self.inner.serialize_some(value) }
    fn serialize_unit(self) -> std::result::Result<Self::Ok, Self::Error> { self.inner.serialize_unit() }
    fn serialize_unit_struct(self, name: &'static str) -> std::result::Result<Self::Ok, Self::Error> { self.inner.serialize_unit_struct(name) }
    fn serialize_unit_variant(self, name: &'static str, tag: u32, value: &'static str) -> std::result::Result<Self::Ok, Self::Error> {
        if value.len() > self.limit { self.exhausted.set(true); return Err(ser::Error::custom("map key byte allowance")); }
        self.inner.serialize_unit_variant(name, tag, value)
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(self, name: &'static str, value: &T) -> std::result::Result<Self::Ok, Self::Error> {
        self.inner.serialize_newtype_struct(name, &Key { value, limit: self.limit, depth: self.depth - 1, exhausted: self.exhausted })
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(self, name: &'static str, tag: u32, variant: &'static str, value: &T) -> std::result::Result<Self::Ok, Self::Error> { self.inner.serialize_newtype_variant(name, tag, variant, value) }
    fn serialize_seq(self, hint: Option<usize>) -> std::result::Result<Self::SerializeSeq, Self::Error> { self.inner.serialize_seq(hint) }
    fn serialize_tuple(self, hint: usize) -> std::result::Result<Self::SerializeTuple, Self::Error> { self.inner.serialize_tuple(hint) }
    fn serialize_tuple_struct(self, name: &'static str, hint: usize) -> std::result::Result<Self::SerializeTupleStruct, Self::Error> { self.inner.serialize_tuple_struct(name, hint) }
    fn serialize_tuple_variant(self, name: &'static str, tag: u32, variant: &'static str, hint: usize) -> std::result::Result<Self::SerializeTupleVariant, Self::Error> { self.inner.serialize_tuple_variant(name, tag, variant, hint) }
    fn serialize_map(self, hint: Option<usize>) -> std::result::Result<Self::SerializeMap, Self::Error> { self.inner.serialize_map(hint) }
    fn serialize_struct(self, name: &'static str, hint: usize) -> std::result::Result<Self::SerializeStruct, Self::Error> { self.inner.serialize_struct(name, hint) }
    fn serialize_struct_variant(self, name: &'static str, tag: u32, variant: &'static str, hint: usize) -> std::result::Result<Self::SerializeStructVariant, Self::Error> { self.inner.serialize_struct_variant(name, tag, variant, hint) }
    fn collect_str<T: ?Sized + fmt::Display>(self, value: &T) -> std::result::Result<Self::Ok, Self::Error> {
        let mut text = Text::new(self.limit);
        if fmt::write(&mut text, format_args!("{value}")).is_err() { self.exhausted.set(true); return Err(ser::Error::custom("map key formatted allowance")); }
        self.inner.serialize_str(&text.value)
    }
    fn is_human_readable(&self) -> bool { self.inner.is_human_readable() }
}

#[cfg(test)]
mod tests;
