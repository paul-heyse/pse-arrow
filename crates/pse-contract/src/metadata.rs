// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One wire-independent checking operation for normalized contract metadata.
//!
//! Descriptor acquisition owns decoding and approval of a world. This module checks
//! its borrowed facts; neither a successful check nor semantic compatibility admits
//! scientific meaning, creates native capabilities or establishes wire compatibility.

use std::collections::BTreeSet;
use crate::{BoundaryDirection, boundary_projection::ConstructionRole, operation_effects::{EffectFacts, EffectScope}};

/// Current interoperability metadata version; distinct from owner interpretation.
pub const VERSION: u32 = 1;

/// Explicit known/unknown enum decoding; an unknown number never becomes a default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnumValue<T> {
    /// A number resolved to the actual native vocabulary.
    Known(T),
    /// Unspecified or unavailable meaning, retained for attribution.
    Unknown(i32),
}

/// Provenance supplied by the original native declaration, not an exact source span.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Owner<'a> {
    /// Actual native type identity.
    pub rust_type: &'a str,
    /// Original declaration file, without an invented span.
    pub declaration_file: &'a str,
}

/// Normalized typed message options, including presence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MessageMetadata<'a> {
    /// Interoperability schema version.
    pub metadata_version: Option<u32>,
    /// Native declaration provenance.
    pub owner: Option<Owner<'a>>,
    /// Owner's declared interpretation version.
    pub interpretation_version: Option<u32>,
    /// Crossing direction independent of construction authority.
    pub direction: Option<EnumValue<BoundaryDirection>>,
    /// Reconstruction or borrowed observation.
    pub construction: Option<EnumValue<ConstructionRole>>,
    /// Optional original source action intent, separate from RPC method effects.
    pub effects: Option<Effects>,
}

/// Normalized typed field options.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldMetadata<'a> {
    /// Interoperability schema version.
    pub metadata_version: Option<u32>,
    /// Original field or alternative name.
    pub native_field: Option<&'a str>,
    /// Actual owner-declared direction.
    pub direction: Option<EnumValue<BoundaryDirection>>,
}

/// Presence-preserving normalized operation intent. It never grants permission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Effects {
    /// Original native scope, without a method-name inference.
    pub scope: Option<EnumValue<EffectScope>>,
    /// Explicit observation classification.
    pub read_only: Option<bool>,
    /// Exact-request repeatability.
    pub idempotent: Option<bool>,
    /// Explicit release of retained custody.
    pub destructive: Option<bool>,
}
impl From<EffectFacts> for Effects {
    fn from(value: EffectFacts) -> Self {
        Self { scope: Some(EnumValue::Known(value.scope)), read_only: Some(value.read_only), idempotent: Some(value.idempotent), destructive: Some(value.destructive) }
    }
}

/// Normalized typed method options.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MethodMetadata {
    /// Interoperability schema version.
    pub metadata_version: Option<u32>,
    /// Explicit owner facts; absent effects are not assumed to be observation.
    pub effects: Option<Effects>,
}

/// Complete acquisition file and its declared import closure.
#[derive(Clone, Copy, Debug)]
pub struct File<'a> {
    /// Relative acquisition name.
    pub name: &'a str,
    /// Original declared imports.
    pub imports: &'a [&'a str],
}

/// One selected owner-derived message; synthetic wrappers are not native owners.
#[derive(Clone, Copy, Debug)]
pub struct Message<'a> {
    /// Fully qualified world symbol.
    pub name: &'a str,
    /// Acquired descriptor file.
    pub file: &'a str,
    /// Required selected options, retaining missing presence.
    pub metadata: Option<MessageMetadata<'a>>,
}

/// One selected owner-derived field, including its referenced message when present.
#[derive(Clone, Copy, Debug)]
pub struct Field<'a> {
    /// Fully qualified field symbol.
    pub name: &'a str,
    /// Acquired descriptor file.
    pub file: &'a str,
    /// Required native correspondence.
    pub metadata: Option<FieldMetadata<'a>>,
    /// Referenced world symbol, without scalar pseudo-references.
    pub reference: Option<&'a str>,
}

/// One selected service method.
#[derive(Clone, Copy, Debug)]
pub struct Method<'a> {
    /// Fully qualified method symbol.
    pub name: &'a str,
    /// Acquired descriptor file.
    pub file: &'a str,
    /// Required selected options.
    pub metadata: Option<MethodMetadata>,
    /// Actual request symbol.
    pub input: &'a str,
    /// Actual response symbol.
    pub output: &'a str,
}

/// An extension allocation, including imported unknown-but-forwardable extensions.
#[derive(Clone, Copy, Debug)]
pub struct Extension<'a> {
    /// Fully qualified extension name.
    pub name: &'a str,
    /// Fully qualified extendee; allocation numbers are scoped to this symbol.
    pub extendee: &'a str,
    /// Original field number.
    pub number: u32,
    /// Acquisition provenance.
    pub file: &'a str,
}

/// Owner-specific typed meaning normalized by an external policy adapter.
#[derive(Clone, Copy, Debug)]
pub struct Facet<'a> {
    /// Attributed selected field or message.
    pub subject: &'a str,
    /// Acquisition provenance.
    pub file: &'a str,
    /// Fully qualified typed policy identity.
    pub policy: &'a str,
    /// Normalized owner-specific data; only that owner interprets it.
    pub value: &'a [u8],
    /// The matching source-world decoder had this owner's interpretation available.
    /// A comparison policy cannot replace an unavailable original interpretation.
    pub interpretation_available: bool,
}

/// An approved acquisition's borrowed facts; no decoder or runtime dependencies.
#[derive(Clone, Copy, Debug, Default)]
pub struct World<'a> {
    /// Complete acquired import world.
    pub files: &'a [File<'a>],
    /// Every resolvable message/enum symbol, including unselected imports.
    pub symbols: &'a [&'a str],
    /// Selected native owners.
    pub messages: &'a [Message<'a>],
    /// Selected owner field correspondences.
    pub fields: &'a [Field<'a>],
    /// Selected method intents.
    pub methods: &'a [Method<'a>],
    /// All acquired extension allocations.
    pub extensions: &'a [Extension<'a>],
    /// Selected external typed meanings.
    pub facets: &'a [Facet<'a>],
}

/// Semantic conclusion under an explicitly named policy; no wire fallback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Disposition {
    /// All selected meaning was understood and accepted by that policy.
    Compatible,
    /// The world is invalid or the policy rejects a meaningful change.
    Incompatible,
    /// Required interpretation is unavailable; callers must not claim compatibility.
    Unknown,
}

/// External owner interpretation remains outside the pure structural foundation.
pub trait InterpretationPolicy {
    /// Stable name attributed on every result.
    fn name(&self) -> &str;
    /// Known interoperability metadata versions.
    fn supports_metadata_version(&self, version: u32) -> bool { version == VERSION }
    /// Available immutable owner interpretation, selected explicitly by the caller.
    fn supports_interpretation(&self, _subject: &str, _version: u32) -> bool { false }
    /// Explicit old/new owner interpretation relation; changing a version is unknown by default.
    fn interpretation_change(&self, _subject: &str, _old: u32, _new: u32) -> Disposition { Disposition::Unknown }
    /// Owner-specific meaning is unknown without its policy.
    fn check_facet(&self, _facet: &Facet<'_>) -> Disposition { Disposition::Unknown }
    /// A wire-compatible facet change has no automatic semantic compatibility.
    fn compare_facet(&self, _old: &Facet<'_>, _new: &Facet<'_>) -> Disposition { Disposition::Unknown }
    /// Introducing meaning on an already selected subject needs an owner decision.
    fn introduced_facet(&self, _new: &Facet<'_>) -> Disposition { Disposition::Unknown }
}

/// Strict core-only interpretation, refusing selected domain policy it does not own.
#[derive(Debug)]
pub struct CorePolicy<'a> {
    /// Approved owner/version associations, independent of interoperability version.
    pub interpretations: &'a [(&'a str, u32)],
}
impl InterpretationPolicy for CorePolicy<'_> {
    fn name(&self) -> &str { "pse.contract.v1/core" }
    fn supports_interpretation(&self, subject: &str, version: u32) -> bool { self.interpretations.contains(&(subject, version)) }
}

/// Attributed explanation, without fabricated physical source coordinates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    /// Original acquisition file.
    pub file: String,
    /// Exact metadata subject.
    pub subject: String,
    /// Interpretation outcome.
    pub disposition: Disposition,
    /// Specific rule or changed meaning.
    pub reason: String,
}

/// Collected outcome, preserving independent findings and unknown interpretation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assessment {
    /// Explicit interpretation policy.
    pub policy: String,
    /// Aggregate result; incompatibility dominates unknown.
    pub disposition: Disposition,
    /// Attributed reasons for every non-compatible fact.
    pub findings: Vec<Finding>,
}
impl Assessment {
    fn new(policy: &impl InterpretationPolicy) -> Self { Self { policy: policy.name().into(), disposition: Disposition::Compatible, findings: vec![] } }
    /// Include an attributed acquisition/interpretation finding with the same
    /// aggregation rules as the normalized metadata checks.
    pub fn record(&mut self, file: &str, subject: &str, disposition: Disposition, reason: impl Into<String>) {
        if disposition == Disposition::Compatible { return; }
        if disposition == Disposition::Incompatible || self.disposition == Disposition::Compatible { self.disposition = disposition; }
        self.findings.push(Finding { file: file.into(), subject: subject.into(), disposition, reason: reason.into() });
    }
    fn required<T>(&mut self, file: &str, subject: &str, value: Option<T>, name: &str) -> Option<T> {
        if value.is_none() { self.record(file, subject, Disposition::Incompatible, format!("missing required {name}")); }
        value
    }
    fn version(&mut self, policy: &impl InterpretationPolicy, file: &str, subject: &str, value: Option<u32>) {
        if let Some(version) = self.required(file, subject, value, "metadata version")
            && !policy.supports_metadata_version(version) {
            self.record(file, subject, Disposition::Unknown, format!("unavailable metadata version {version}"));
        }
    }
    fn enumeration<T>(&mut self, file: &str, subject: &str, value: Option<EnumValue<T>>, name: &str) -> Option<T> {
        match self.required(file, subject, value, name)? {
            EnumValue::Known(value) => Some(value),
            EnumValue::Unknown(number) => { self.record(file, subject, Disposition::Unknown, format!("unavailable {name} value {number}")); None }
        }
    }
    fn effects(&mut self, file: &str, subject: &str, effects: Effects) {
        let scope = self.enumeration(file, subject, effects.scope, "effect scope");
        let read_only = self.required(file, subject, effects.read_only, "read_only");
        self.required(file, subject, effects.idempotent, "idempotent");
        let destructive = self.required(file, subject, effects.destructive, "destructive");
        if let (Some(scope), Some(read_only), Some(destructive)) = (scope, read_only, destructive)
            && !effects_coherent(scope, read_only, destructive) {
            self.record(file, subject, Disposition::Incompatible, "effect scope conflicts with observation or custody release facts");
        }
    }
}

pub(crate) fn effects_coherent(scope: EffectScope, read_only: bool, destructive: bool) -> bool {
    read_only == matches!(scope, EffectScope::Observation) && destructive == matches!(scope, EffectScope::RetentionRelease)
}

/// Check required options, versions, closure, references, allocations and effect coherence.
pub fn check(world: &World<'_>, policy: &impl InterpretationPolicy) -> Assessment {
    let mut result = Assessment::new(policy);
    if policy.name().is_empty() { result.record("", "interpretation policy", Disposition::Incompatible, "interpretation policy must have an explicit name"); }
    let mut files = BTreeSet::new();
    for file in world.files {
        if !files.insert(file.name) { result.record(file.name, file.name, Disposition::Incompatible, "duplicate acquisition filename"); }
    }
    for file in world.files { for import in file.imports { if !files.contains(import) { result.record(file.name, import, Disposition::Incompatible, "missing transitive import"); } } }
    let mut symbols = BTreeSet::new();
    for symbol in world.symbols { if !symbols.insert(*symbol) { result.record("", symbol, Disposition::Incompatible, "duplicate world symbol"); } }
    let mut names = BTreeSet::new();
    let mut allocations = BTreeSet::new();
    for extension in world.extensions {
        if !names.insert(extension.name) || !allocations.insert((extension.extendee, extension.number)) {
            result.record(extension.file, extension.name, Disposition::Incompatible, "extension name or extendee/number collision");
        }
        if !symbols.contains(extension.extendee) { result.record(extension.file, extension.name, Disposition::Incompatible, "unresolved extension extendee"); }
    }
    let mut selected = BTreeSet::new();
    for message in world.messages {
        if !selected.insert(message.name) { result.record(message.file, message.name, Disposition::Incompatible, "duplicate selected message"); }
        if !files.contains(message.file) || !symbols.contains(message.name) { result.record(message.file, message.name, Disposition::Incompatible, "selected message is outside acquired world"); }
        if let Some(metadata) = result.required(message.file, message.name, message.metadata, "message metadata") {
            result.version(policy, message.file, message.name, metadata.metadata_version);
            if let Some(owner) = result.required(message.file, message.name, metadata.owner, "native owner")
                && (owner.rust_type.is_empty() || owner.declaration_file.is_empty()) { result.record(message.file, message.name, Disposition::Incompatible, "empty native owner provenance"); }
            if let Some(version) = result.required(message.file, message.name, metadata.interpretation_version, "interpretation version") {
                if version == 0 { result.record(message.file, message.name, Disposition::Incompatible, "zero interpretation version"); }
                else if !policy.supports_interpretation(message.name, version) { result.record(message.file, message.name, Disposition::Unknown, format!("unavailable owner interpretation {version}")); }
            }
            result.enumeration(message.file, message.name, metadata.direction, "direction");
            result.enumeration(message.file, message.name, metadata.construction, "construction role");
            if let Some(effects) = metadata.effects { result.effects(message.file, message.name, effects); }
        }
    }
    selected.clear();
    for field in world.fields {
        if !selected.insert(field.name) { result.record(field.file, field.name, Disposition::Incompatible, "duplicate selected field"); }
        if !files.contains(field.file) { result.record(field.file, field.name, Disposition::Incompatible, "field is outside acquired world"); }
        if let Some(reference) = field.reference && !symbols.contains(reference) { result.record(field.file, field.name, Disposition::Incompatible, format!("unresolved reference {reference}")); }
        if let Some(metadata) = result.required(field.file, field.name, field.metadata, "field metadata") {
            result.version(policy, field.file, field.name, metadata.metadata_version);
            if result.required(field.file, field.name, metadata.native_field, "native field").is_some_and(str::is_empty) { result.record(field.file, field.name, Disposition::Incompatible, "empty native field"); }
            result.enumeration(field.file, field.name, metadata.direction, "direction");
        }
    }
    selected.clear();
    for method in world.methods {
        if !selected.insert(method.name) { result.record(method.file, method.name, Disposition::Incompatible, "duplicate selected method"); }
        if !files.contains(method.file) { result.record(method.file, method.name, Disposition::Incompatible, "method is outside acquired world"); }
        for reference in [method.input, method.output] { if !symbols.contains(reference) { result.record(method.file, method.name, Disposition::Incompatible, format!("unresolved method root {reference}")); } }
        if let Some(metadata) = result.required(method.file, method.name, method.metadata, "method metadata") {
            result.version(policy, method.file, method.name, metadata.metadata_version);
            if let Some(effects) = result.required(method.file, method.name, metadata.effects, "owner effects") {
                result.effects(method.file, method.name, effects);
            }
        }
    }
    for facet in world.facets {
        let disposition = if facet.interpretation_available { policy.check_facet(facet) } else { Disposition::Unknown };
        result.record(facet.file, facet.subject, disposition, format!("selected domain interpretation {}", facet.policy));
    }
    result
}

/// Compare meaningful metadata independently of wire compatibility. Provenance-only
/// native type/file moves do not change physical interpretation. Missing external
/// policy remains unknown even when encoded bytes happen to match.
pub fn compare(old: &World<'_>, new: &World<'_>, policy: &impl InterpretationPolicy) -> Assessment {
    let mut result = check(old, policy);
    let current = check(new, policy);
    for finding in current.findings { result.record(&finding.file, &finding.subject, finding.disposition, finding.reason); }
    for before in old.messages {
        match new.messages.iter().find(|after| after.name == before.name) {
            None => result.record(before.file, before.name, Disposition::Incompatible, "selected message removed"),
            Some(after) => if let (Some(a), Some(b)) = (before.metadata, after.metadata) {
                if known_changed(a.direction, b.direction) || known_changed(a.construction, b.construction) { result.record(after.file, after.name, Disposition::Incompatible, "direction or construction role changed"); }
                if a.metadata_version != b.metadata_version { result.record(after.file, after.name, Disposition::Unknown, "metadata version changed"); }
                if effects_changed(a.effects, b.effects) { result.record(after.file, after.name, Disposition::Incompatible, "source action effects changed"); }
                if let (Some(a), Some(b)) = (a.interpretation_version, b.interpretation_version) && a != b { result.record(after.file, after.name, policy.interpretation_change(after.name, a, b), "owner interpretation version changed"); }
            }
        }
    }
    for before in old.fields {
        match new.fields.iter().find(|after| after.name == before.name) {
            None => result.record(before.file, before.name, Disposition::Incompatible, "selected field removed"),
            Some(after) => {
                if before.reference != after.reference { result.record(after.file, after.name, Disposition::Incompatible, "field reference changed"); }
                if let (Some(a), Some(b)) = (before.metadata, after.metadata) {
                    if a.metadata_version != b.metadata_version { result.record(after.file, after.name, Disposition::Unknown, "field metadata version changed"); }
                    if a.native_field != b.native_field || known_changed(a.direction, b.direction) { result.record(after.file, after.name, Disposition::Incompatible, "field correspondence or direction changed"); }
                }
            }
        }
    }
    for before in old.methods {
        match new.methods.iter().find(|after| after.name == before.name) {
            None => result.record(before.file, before.name, Disposition::Incompatible, "selected method removed"),
            Some(after) => {
                if before.input != after.input || before.output != after.output { result.record(after.file, after.name, Disposition::Incompatible, "method roots changed"); }
                if let (Some(a), Some(b)) = (before.metadata, after.metadata) {
                    if a.metadata_version != b.metadata_version { result.record(after.file, after.name, Disposition::Unknown, "method metadata version changed"); }
                    if effects_changed(a.effects, b.effects) { result.record(after.file, after.name, Disposition::Incompatible, "method effects changed"); }
                }
            }
        }
    }
    for before in old.facets {
        match new.facets.iter().find(|after| after.subject == before.subject && after.policy == before.policy) {
            None => result.record(before.file, before.subject, Disposition::Unknown, format!("selected domain interpretation {} removed", before.policy)),
            Some(after) => {
                let disposition = if before.interpretation_available && after.interpretation_available { policy.compare_facet(before, after) } else { Disposition::Unknown };
                result.record(after.file, after.subject, disposition, format!("domain interpretation {} comparison", after.policy));
            }
        }
    }
    for after in new.facets {
        let existing_subject = old.messages.iter().any(|item| item.name == after.subject) || old.fields.iter().any(|item| item.name == after.subject) || old.methods.iter().any(|item| item.name == after.subject);
        if existing_subject && !old.facets.iter().any(|before| before.subject == after.subject && before.policy == after.policy) {
            let disposition = if after.interpretation_available { policy.introduced_facet(after) } else { Disposition::Unknown };
            result.record(after.file, after.subject, disposition, format!("domain interpretation {} introduced on existing subject", after.policy));
        }
    }
    result
}

fn known_changed<T: PartialEq>(old: Option<EnumValue<T>>, new: Option<EnumValue<T>>) -> bool {
    matches!((old, new), (Some(EnumValue::Known(old)), Some(EnumValue::Known(new))) if old != new)
}

fn effects_changed(old: Option<Effects>, new: Option<Effects>) -> bool {
    match (old, new) {
        (Some(old), Some(new)) => known_changed(old.scope, new.scope) || old.read_only != new.read_only || old.idempotent != new.idempotent || old.destructive != new.destructive,
        (None, None) => false,
        _ => true,
    }
}

#[cfg(test)]
mod tests;
