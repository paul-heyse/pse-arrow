// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed custom options against the actual compiled interoperability schema.
//!
//! Buffa 0.9.2 `DescriptorPool` links the complete lossless descriptor world;
//! `DynamicMessage` plus `ReflectMessageMut::try_set` checks message/field membership
//! and value shape. `ExtensionSet` retains encoded options in unknown-field storage.
//! No ordinary Prost descriptor re-encoding or ad-hoc tag codec intervenes.
//! Sources: <https://github.com/anthropics/buffa/blob/v0.9.2/docs/guide.md>
//! and <https://buf.build/docs/reference/protobuf-language-spec/#options>.

use std::sync::Arc;
use buffa::{ExtensionSet, UnknownFields};
use buffa_descriptor::{DescriptorPool, DynamicMessage, Value, ReflectMessage, ReflectMessageMut};
use pse_contract::{boundary_projection::{ConstructionRole, Definition, Direction, Field}, operation_effects::{EffectFacts, EffectScope}};
use super::{descriptor::{self, FileDescriptorProto, FileDescriptorSet}, error};
use pse_schema::error::SchemaError;

/// Actual source schema import required by every owner-derived generation world.
pub const CORE_IMPORT: &str = "pse/contract/v1/metadata.proto";

mod check;
pub use check::{check_descriptors, compare_descriptors};
#[cfg(test)]
mod tests;

/// Checked dynamic encoder shared with owner-specific typed facet lowerers.
pub(super) struct OptionEncoder {
    pool: Arc<DescriptorPool>,
}

impl OptionEncoder {
    pub(super) fn pool(&self) -> Arc<DescriptorPool> { Arc::clone(&self.pool) }
    pub(super) fn new(imports: &[FileDescriptorProto]) -> Result<Self, SchemaError> {
        if !imports.iter().any(|file| file.name.as_deref() == Some(CORE_IMPORT)) {
            return Err(error(format!("required typed metadata import {CORE_IMPORT} is missing; compile the source schema with its complete import closure")));
        }
        let pool = DescriptorPool::new(FileDescriptorSet { file: imports.to_vec(), ..Default::default() }).map_err(|e| error(format!("typed metadata import world: {e}")))?;
        let encoder = Self { pool: Arc::new(pool) };
        // These are representations of actual native members, not a second native
        // vocabulary. Refuse a schema that changes the correspondence silently.
        encoder.vocabulary("pse.contract.v1.Direction", &["DIRECTION_UNSPECIFIED", "DIRECTION_INPUT", "DIRECTION_OUTPUT", "DIRECTION_BOTH"])?;
        encoder.vocabulary("pse.contract.v1.ConstructionRole", &["CONSTRUCTION_ROLE_UNSPECIFIED", "CONSTRUCTION_ROLE_DATA", "CONSTRUCTION_ROLE_OBSERVATION_ONLY"])?;
        encoder.vocabulary("pse.contract.v1.EffectScope", &["EFFECT_SCOPE_UNSPECIFIED", "EFFECT_SCOPE_OBSERVATION", "EFFECT_SCOPE_SOURCE_ADMISSION", "EFFECT_SCOPE_CANONICAL_COMMAND", "EFFECT_SCOPE_RETENTION_RELEASE", "EFFECT_SCOPE_FILE_EXPORT", "EFFECT_SCOPE_CAPTURE", "EFFECT_SCOPE_NATIVE_EXECUTION"])?;
        encoder.core_shapes()?;
        for (name, extendee, ty) in [
            ("message", "google.protobuf.MessageOptions", "pse.contract.v1.MessageMetadata"),
            ("field", "google.protobuf.FieldOptions", "pse.contract.v1.FieldMetadata"),
            ("method", "google.protobuf.MethodOptions", "pse.contract.v1.MethodMetadata"),
        ] {
            let extension = encoder.pool.extension_by_name(&format!("pse.contract.v1.{name}")).ok_or_else(|| error(format!("required typed metadata extension {name} is absent")))?;
            let expected = encoder.pool.message_index(ty).ok_or_else(|| error(format!("required typed option message {ty} is absent")))?;
            if extension.field().number() != 51065 || encoder.pool.message(extension.extendee()).full_name() != extendee
                || extension.field().kind() != buffa_descriptor::FieldKind::Singular(buffa_descriptor::SingularKind::Message(expected)) {
                return Err(error(format!("typed metadata extension {name} has a changed allocation, extendee or value type")));
            }
        }
        Ok(encoder)
    }

    // This is the v1 interpreter's acceptance contract, never a descriptor producer.
    // The actual .proto remains the declaration; imported lookalikes cannot change
    // known member types, tags or presence and then be normalized as compatible v1.
    fn core_shapes(&self) -> Result<(), SchemaError> {
        use buffa_descriptor::{FieldKind::Singular, SingularKind::{Scalar, Message, Enum}, ScalarType::{Uint32, String, Bool}};
        let message = |name: &str| self.pool.message_index(&format!("pse.contract.v1.{name}")).map(Message).map(Singular).ok_or_else(|| error(format!("missing v1 metadata message {name}")));
        let enumeration = |name: &str| self.pool.enum_index(&format!("pse.contract.v1.{name}")).map(Enum).map(Singular).ok_or_else(|| error(format!("missing v1 metadata enum {name}")));
        let scalar = |kind| Singular(Scalar(kind));
        for (name, expected) in [
            ("Owner", vec![("rust_type", scalar(String)), ("declaration_file", scalar(String))]),
            ("MessageMetadata", vec![("metadata_version", scalar(Uint32)), ("owner", message("Owner")?), ("interpretation_version", scalar(Uint32)), ("direction", enumeration("Direction")?), ("construction_role", enumeration("ConstructionRole")?), ("effects", message("Effects")?)]),
            ("FieldMetadata", vec![("metadata_version", scalar(Uint32)), ("native_field", scalar(String)), ("direction", enumeration("Direction")?)]),
            ("MethodMetadata", vec![("metadata_version", scalar(Uint32)), ("effects", message("Effects")?)]),
            ("Effects", vec![("scope", enumeration("EffectScope")?), ("read_only", scalar(Bool)), ("idempotent", scalar(Bool)), ("destructive", scalar(Bool))]),
        ] {
            let full_name = format!("pse.contract.v1.{name}");
            let descriptor = self.pool.message_by_name(&full_name).ok_or_else(|| error(format!("missing v1 metadata message {name}")))?;
            if descriptor.fields().len() != expected.len() || !descriptor.oneofs().is_empty() || !descriptor.extension_ranges().is_empty()
                || expected.iter().enumerate().any(|(index, (name, kind))| descriptor.field_by_name(name).is_none_or(|field| field.name() != *name || field.number() != (index + 1) as u32 || field.kind() != *kind || field.presence() != buffa::editions::FieldPresence::Explicit)) {
                return Err(error(format!("{full_name}: unsupported v1 metadata field schema correspondence")));
            }
        }
        Ok(())
    }

    pub(super) fn vocabulary(&self, name: &str, expected: &[&str]) -> Result<(), SchemaError> {
        let enumeration = self.pool.enum_by_name(name).ok_or_else(|| error(format!("missing typed metadata vocabulary {name}")))?;
        if enumeration.values().len() != expected.len() || expected.iter().enumerate().any(|(number, name)| enumeration.value_by_name(name).is_none_or(|value| value.number() != number as i32)) {
            return Err(error(format!("{name}: schema/native vocabulary correspondence changed")));
        }
        Ok(())
    }

    pub(super) fn message(&self, name: &str, fields: &[(&str, Value)]) -> Result<DynamicMessage, SchemaError> {
        let mut message = DynamicMessage::new_by_name(Arc::clone(&self.pool), name).ok_or_else(|| error(format!("typed option message {name} absent from import world")))?;
        let descriptor = self.pool.message(message.message_index());
        let mut supplied = std::collections::BTreeSet::new();
        for (name, value) in fields {
            if !supplied.insert(*name) { return Err(error(format!("duplicate typed option field {name}"))); }
            let field = descriptor.field_by_name(name).ok_or_else(|| error(format!("{}.{} absent from typed option schema", descriptor.full_name(), name)))?;
            message.try_set(field, value.clone()).map_err(|e| error(format!("{}.{}: {e}", descriptor.full_name(), name)))?;
        }
        Ok(message)
    }

    pub(super) fn attach(&self, options: &mut impl ExtensionSet, name: &str, value: DynamicMessage) -> Result<(), SchemaError> {
        let extension = self.pool.extension_by_name(name).ok_or_else(|| error(format!("typed option extension {name} absent from import world")))?;
        self.attach_checked(options, extension, value)
    }

    pub(super) fn decode_extension<T: ExtensionSet + buffa::Message>(&self, options: &T, name: &str) -> Result<Option<DynamicMessage>, SchemaError> {
        let extension = self.pool.extension_by_name(name).ok_or_else(|| error(format!("typed option extension {name} absent from world")))?;
        if self.pool.message(extension.extendee()).full_name() != T::PROTO_FQN { return Err(error(format!("typed option {name} has wrong extendee"))); }
        let count = options.unknown_fields().iter().filter(|field| field.number == extension.field().number()).count();
        if count == 0 { return Ok(None); }
        if count != 1 { return Err(error(format!("duplicate selected typed option {name}"))); }
        let bytes = options.try_encode_to_vec().map_err(|e| error(e.to_string()))?;
        let decoded = DynamicMessage::decode(Arc::clone(&self.pool), extension.extendee(), &bytes).map_err(|e| error(format!("decode {name}: {e}")))?;
        match decoded.field_by_number(extension.field().number()) {
            Some(Value::Message(value)) => Ok(Some(value.clone())),
            _ => Err(error(format!("{name}: selected option does not decode to its declared message"))),
        }
    }

    fn attach_checked<T: ExtensionSet>(&self, options: &mut T, extension: &buffa_descriptor::ExtensionDescriptor, value: DynamicMessage) -> Result<(), SchemaError> {
        let name = extension.full_name();
        let actual_extendee = self.pool.message(extension.extendee()).full_name();
        if actual_extendee != T::PROTO_FQN { return Err(error(format!("{name} extends {actual_extendee}, not {}", T::PROTO_FQN))); }
        if options.unknown_fields().iter().any(|field| field.number == extension.field().number()) { return Err(error(format!("duplicate selected typed option {name}"))); }
        let mut container = DynamicMessage::new(Arc::clone(&self.pool), extension.extendee());
        container.try_set(extension.field(), Value::Message(value)).map_err(|e| error(format!("{name}: {e}")))?;
        let bytes = container.try_encode_to_vec().map_err(|e| error(e.to_string()))?;
        let fields = UnknownFields::decode_from_slice(&bytes).map_err(|e| error(e.to_string()))?;
        for field in fields { options.unknown_fields_mut().push(field); }
        Ok(())
    }

    pub(super) fn message_options(&self, definition: &Definition) -> Result<descriptor::MessageOptions, SchemaError> {
        let owner = self.message("pse.contract.v1.Owner", &[("rust_type", Value::String(definition.rust_type.into())), ("declaration_file", Value::String(definition.source.into()))])?;
        let construction = match definition.construction { ConstructionRole::Data => 1, ConstructionRole::ObservationOnly => 2 };
        let mut fields = vec![
            ("metadata_version", Value::U32(pse_contract::metadata::VERSION)), ("owner", Value::Message(owner)),
            ("interpretation_version", Value::U32(definition.version)), ("direction", direction(definition.direction)),
            ("construction_role", Value::EnumNumber(construction)),
        ];
        if let Some(effects) = definition.effects { fields.push(("effects", Value::Message(self.effects(effects)?))); }
        let metadata = self.message("pse.contract.v1.MessageMetadata", &fields)?;
        let mut options = descriptor::MessageOptions::default();
        self.attach(&mut options, "pse.contract.v1.message", metadata)?;
        Ok(options)
    }

    pub(super) fn field_options(&self, options: &mut descriptor::FieldOptions, field: &Field) -> Result<(), SchemaError> {
        self.field_correspondence(options, &field.name, field.direction)
    }

    pub(super) fn field_correspondence(&self, options: &mut descriptor::FieldOptions, name: &str, field_direction: Direction) -> Result<(), SchemaError> {
        let metadata = self.message("pse.contract.v1.FieldMetadata", &[("metadata_version", Value::U32(pse_contract::metadata::VERSION)), ("native_field", Value::String(name.into())), ("direction", direction(field_direction))])?;
        self.attach(options, "pse.contract.v1.field", metadata)
    }

    fn effects(&self, effects: EffectFacts) -> Result<DynamicMessage, SchemaError> {
        let scope = match effects.scope {
            EffectScope::Observation => 1, EffectScope::SourceAdmission => 2, EffectScope::CanonicalCommand => 3,
            EffectScope::RetentionRelease => 4, EffectScope::FileExport => 5, EffectScope::Capture => 6, EffectScope::NativeExecution => 7,
        };
        self.message("pse.contract.v1.Effects", &[("scope", Value::EnumNumber(scope)), ("read_only", Value::Bool(effects.read_only)), ("idempotent", Value::Bool(effects.idempotent)), ("destructive", Value::Bool(effects.destructive))])
    }

    pub(super) fn method_options(&self, effects: Option<EffectFacts>) -> Result<descriptor::MethodOptions, SchemaError> {
        let effects = self.effects(effects.ok_or_else(|| error("selected method has no explicitly declared owner effects"))?)?;
        let metadata = self.message("pse.contract.v1.MethodMetadata", &[("metadata_version", Value::U32(pse_contract::metadata::VERSION)), ("effects", Value::Message(effects))])?;
        let mut options = descriptor::MethodOptions::default();
        self.attach(&mut options, "pse.contract.v1.method", metadata)?;
        Ok(options)
    }
}

fn direction(value: Direction) -> Value {
    Value::EnumNumber(match value { Direction::Input => 1, Direction::Output => 2, Direction::Both => 3 })
}

type DomainCheck = fn(&DynamicMessage) -> Result<(), SchemaError>;

/// Transport adapter for an explicit native interpretation policy. Each selected
/// typed facet is checked against its original acquired pool before normalization;
/// old payloads are never reinterpreted through only the current world's pool.
pub trait DescriptorPolicy: pse_contract::metadata::InterpretationPolicy {
    /// Native source selection, independent of whether acquired options survived.
    fn native_selection(&self) -> Option<NativeSelection<'_>> { None }
    /// None means that the original interpretation is unavailable, hence Unknown.
    fn check_decoded_facet(&self, _extension: &str, _value: &DynamicMessage) -> Option<Result<(), SchemaError>> { None }
}
impl DescriptorPolicy for pse_contract::metadata::CorePolicy<'_> {}

/// Borrowed native source coverage, never inferred from option presence.
#[derive(Clone, Copy, Debug)]
pub struct NativeSelection<'a> {
    /// Fully qualified expected native owner symbols.
    pub owners: &'a [&'a str],
    /// Expected native field correspondence paths, including alternative payloads.
    pub fields: &'a [&'a str],
    /// Expected source-composed method paths.
    pub methods: &'a [&'a str],
}

/// One acquired descriptor world with its own available native policy.
#[derive(Clone, Copy, Debug)]
pub struct DescriptorInput<'a, P> {
    /// Acquired descriptors, preserving their original schema pool.
    pub descriptors: &'a FileDescriptorSet,
    /// Explicit selected native source file.
    pub selected_file: &'a str,
    /// Available source interpretation and expected native coverage.
    pub policy: &'a P,
}

#[derive(Debug)]
pub(super) struct SelectionNames {
    pub(super) owners: Vec<String>,
    pub(super) fields: Vec<String>,
    pub(super) methods: Vec<String>,
}
pub(super) fn native_selection(graph: &pse_contract::boundary_projection::ProjectionGraph, package: &str, direction: Direction) -> SelectionNames {
    use pse_contract::boundary_projection::Shape;
    let mut selection = SelectionNames { owners: vec![], fields: vec![], methods: vec![] };
    for owner in graph.definitions().values() {
        let name = format!("{package}.{}", owner.name);
        selection.owners.push(name.clone());
        match &owner.shape {
            Shape::Record(fields) => for field in fields.iter().filter(|field| field.direction.includes(direction)) { selection.fields.push(format!("{name}.{}", field.name)); },
            Shape::Alternatives(alternatives) => for alternative in alternatives {
                selection.fields.push(format!("{name}.{}", super::snake_case(&alternative.name)));
                for field in alternative.fields.iter().filter(|field| field.direction.includes(direction)) { selection.fields.push(format!("{name}.Alternative{}.{}", alternative.tag, field.name)); }
            },
            Shape::StringEnum(_) => selection.fields.push(format!("{name}.value")),
        }
    }
    for service in graph.services().values() { for method in &service.methods { selection.methods.push(format!("{package}.{}.{}", service.name, method.name)); } }
    selection
}

/// Explicit selected domain policies; unknown ownership never falls back to wire
/// compatibility. Owner callbacks validate typed payloads under the acquired pool.
pub(super) struct TypedPolicy<'a> {
    validators: &'a [(&'a str, DomainCheck)],
    interpretations: &'a [(&'a str, u32)],
    selection: NativeSelection<'a>,
}
impl<'a> TypedPolicy<'a> {
    pub(super) fn new(validators: &'a [(&'a str, DomainCheck)], interpretations: &'a [(&'a str, u32)], selection: NativeSelection<'a>) -> Self { Self { validators, interpretations, selection } }
    fn validate(&self, facet: &pse_contract::metadata::Facet<'_>) -> pse_contract::metadata::Disposition {
        use pse_contract::metadata::Disposition;
        if facet.interpretation_available && self.validators.iter().any(|(name, _)| *name == facet.policy) { Disposition::Compatible } else { Disposition::Unknown }
    }
}
impl DescriptorPolicy for TypedPolicy<'_> {
    fn native_selection(&self) -> Option<NativeSelection<'_>> { Some(self.selection) }
    fn check_decoded_facet(&self, extension: &str, value: &DynamicMessage) -> Option<Result<(), SchemaError>> {
        self.validators.iter().find(|(name, _)| *name == extension).map(|(_, validate)| validate(value))
    }
}
impl pse_contract::metadata::InterpretationPolicy for TypedPolicy<'_> {
    fn name(&self) -> &str { "pse.contract.v1/core+selected-domain" }
    fn supports_interpretation(&self, subject: &str, version: u32) -> bool { self.interpretations.contains(&(subject, version)) }
    fn check_facet(&self, facet: &pse_contract::metadata::Facet<'_>) -> pse_contract::metadata::Disposition { self.validate(facet) }
    fn compare_facet(&self, old: &pse_contract::metadata::Facet<'_>, new: &pse_contract::metadata::Facet<'_>) -> pse_contract::metadata::Disposition {
        use pse_contract::metadata::Disposition;
        match (self.validate(old), self.validate(new)) {
            (Disposition::Compatible, Disposition::Compatible) => if old.value == new.value { Disposition::Compatible } else { Disposition::Incompatible },
            (Disposition::Incompatible, _) | (_, Disposition::Incompatible) => Disposition::Incompatible,
            _ => Disposition::Unknown,
        }
    }
    fn introduced_facet(&self, new: &pse_contract::metadata::Facet<'_>) -> pse_contract::metadata::Disposition {
        use pse_contract::metadata::Disposition;
        if self.validate(new) == Disposition::Compatible { Disposition::Incompatible } else { Disposition::Unknown }
    }
}
