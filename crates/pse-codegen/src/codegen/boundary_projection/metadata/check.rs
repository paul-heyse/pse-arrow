// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Lossless descriptor adapter for the one pure native metadata checker.
use super::*;
use pse_contract::metadata as core;

struct Selected {
    name: String,
    file: String,
    metadata: Option<DynamicMessage>,
    references: Vec<String>,
}
struct SelectedFacet { subject: String, file: String, policy: String, value: Vec<u8>, interpretation_available: bool }

/// Decode the selected file's typed metadata and apply one explicit policy. Domain
/// extensions are selected by fully qualified extension name; an unavailable domain
/// policy returns Unknown through the pure checker. Unknown unselected imported
/// options remain preserved in the input descriptor set and are not silently adopted.
pub fn check_descriptors(
    descriptors: &FileDescriptorSet,
    selected_file: &str,
    policy: &impl DescriptorPolicy,
    selected_extensions: &[&str],
) -> Result<core::Assessment, SchemaError> {
    let mut result = with_world(descriptors, selected_file, selected_extensions, policy, |world| core::check(world, policy))?;
    if policy.native_selection().is_none() { result.record(selected_file, "native selection", core::Disposition::Unknown, "approved native owner/field/method coverage is unavailable"); }
    Ok(result)
}

/// Compare decoded typed metadata under the same pure checker used in generation.
/// Wire compatibility remains an independent result; it cannot replace Unknown.
pub fn compare_descriptors<A: DescriptorPolicy, B: DescriptorPolicy>(
    old: DescriptorInput<'_, A>,
    new: DescriptorInput<'_, B>,
    policy: &impl core::InterpretationPolicy,
    selected_extensions: &[&str],
) -> Result<core::Assessment, SchemaError> {
    with_world(old.descriptors, old.selected_file, selected_extensions, old.policy, |before| with_world(new.descriptors, new.selected_file, selected_extensions, new.policy, |after| {
        let mut result = core::compare(before, after, policy);
        for (label, source) in [("old", core::check(before, old.policy)), ("new", core::check(after, new.policy))] {
            for finding in source.findings { result.record(&finding.file, &finding.subject, finding.disposition, format!("{label} source policy {}: {}", source.policy, finding.reason)); }
        }
        if old.policy.native_selection().is_none() || new.policy.native_selection().is_none() { result.record(new.selected_file, "native selection", core::Disposition::Unknown, "old/new approved native coverage is unavailable"); }
        result
    }))?
}

fn with_world<T>(
    descriptors: &FileDescriptorSet,
    selected_file: &str,
    selected_extensions: &[&str],
    policy: &impl DescriptorPolicy,
    consume: impl FnOnce(&core::World<'_>) -> T,
) -> Result<T, SchemaError> {
    let encoder = OptionEncoder::new(&descriptors.file)?;
    let file = descriptors.file.iter().find(|file| file.name.as_deref() == Some(selected_file)).ok_or_else(|| error(format!("selected metadata file {selected_file} is absent")))?;
    let package = file.package.as_deref().unwrap_or_default();
    let mut messages = Vec::new();
    let mut fields = Vec::new();
    let mut methods = Vec::new();
    let mut facets = Vec::new();
    for message in &file.message_type {
        let name = format!("{package}.{}", message.name.as_deref().unwrap_or_default());
        messages.push(Selected { name: name.clone(), file: selected_file.into(), metadata: located(selected_file, &name, message.options.as_option().map(|options| encoder.decode_extension(options, "pse.contract.v1.message")).transpose())?.flatten(), references: vec![] });
        collect_fields(&encoder, message, &name, selected_file, true, selected_extensions, policy, &mut fields, &mut facets)?;
    }
    for service in &file.service { for method in &service.method {
        methods.push(Selected {
            name: format!("{package}.{}.{}", service.name.as_deref().unwrap_or_default(), method.name.as_deref().unwrap_or_default()),
            file: selected_file.into(),
            metadata: method.options.as_option().map(|options| encoder.decode_extension(options, "pse.contract.v1.method")).transpose()?.flatten(),
            references: vec![method.input_type.as_deref().unwrap_or_default().trim_start_matches('.').into(), method.output_type.as_deref().unwrap_or_default().trim_start_matches('.').into()],
        });
    } }
    if let Some(selection) = policy.native_selection() {
        if selection.owners.is_empty() { return Err(error(format!("{selected_file}: approved native owner selection is empty"))); }
        for (kind, expected, actual) in [
            ("owner", selection.owners, &messages), ("field", selection.fields, &fields), ("method", selection.methods, &methods),
        ] {
            for name in expected { if !actual.iter().any(|item| item.name == *name) { return Err(error(format!("{selected_file}:{name}: required selected native {kind} descriptor is absent"))); } }
            for item in actual { if !expected.contains(&item.name.as_str()) { return Err(error(format!("{selected_file}:{}: native {kind} is outside approved source coverage", item.name))); } }
        }
    }
    let imports = descriptors.file.iter().map(|file| file.dependency.iter().map(String::as_str).collect::<Vec<_>>()).collect::<Vec<_>>();
    let files = descriptors.file.iter().zip(&imports).map(|(file, imports)| core::File { name: file.name.as_deref().unwrap_or_default(), imports }).collect::<Vec<_>>();
    let symbols = encoder.pool.messages().iter().map(|symbol| symbol.full_name()).chain(encoder.pool.enums().iter().map(|symbol| symbol.full_name())).collect::<Vec<_>>();
    let mut extension_declarations = Vec::new();
    for file in &descriptors.file {
        collect_extensions(file, &mut extension_declarations);
    }
    let extensions = extension_declarations.iter().map(|(name, extendee, number, file)| core::Extension { name, extendee, number: *number, file }).collect::<Vec<_>>();
    let messages = messages.iter().map(|value| Ok(core::Message { name: &value.name, file: &value.file, metadata: located(&value.file, &value.name, value.metadata.as_ref().map(message_metadata).transpose())? })).collect::<Result<Vec<_>, SchemaError>>()?;
    let fields = fields.iter().map(|value| Ok(core::Field { name: &value.name, file: &value.file, metadata: located(&value.file, &value.name, value.metadata.as_ref().map(field_metadata).transpose())?, reference: value.references.first().map(String::as_str) })).collect::<Result<Vec<_>, SchemaError>>()?;
    let methods = methods.iter().map(|value| Ok(core::Method { name: &value.name, file: &value.file, metadata: located(&value.file, &value.name, value.metadata.as_ref().map(method_metadata).transpose())?, input: &value.references[0], output: &value.references[1] })).collect::<Result<Vec<_>, SchemaError>>()?;
    let facets = facets.iter().map(|facet| core::Facet { subject: &facet.subject, file: &facet.file, policy: &facet.policy, value: &facet.value, interpretation_available: facet.interpretation_available }).collect::<Vec<_>>();
    Ok(consume(&core::World { files: &files, symbols: &symbols, messages: &messages, fields: &fields, methods: &methods, extensions: &extensions, facets: &facets }))
}

fn located<T>(file: &str, subject: &str, result: Result<T, SchemaError>) -> Result<T, SchemaError> {
    result.map_err(|error_value| error(format!("{file}:{subject}: {error_value}")))
}

#[allow(clippy::too_many_arguments, reason = "one recursive walk carries the selected owner and borrowed output accumulators")]
fn collect_fields(encoder: &OptionEncoder, message: &descriptor::DescriptorProto, owner: &str, file: &str, required: bool, selected_extensions: &[&str], policy: &impl DescriptorPolicy, fields: &mut Vec<Selected>, facets: &mut Vec<SelectedFacet>) -> Result<(), SchemaError> {
    for field in &message.field {
        let name = format!("{owner}.{}", field.name.as_deref().unwrap_or_default());
        let metadata = located(file, &name, field.options.as_option().map(|options| encoder.decode_extension(options, "pse.contract.v1.field")).transpose())?.flatten();
        let expected = policy.native_selection().is_some_and(|selection| selection.fields.contains(&name.as_str()));
        if required || expected || metadata.is_some() {
            fields.push(Selected { name: name.clone(), file: file.into(), metadata, references: field.type_name.as_deref().map(|name| name.trim_start_matches('.').into()).into_iter().collect() });
        }
        for extension in selected_extensions {
            if let Some(options) = field.options.as_option()
                && let Some(value) = encoder.decode_extension(options, extension)? {
                let interpretation_available = if let Some(result) = policy.check_decoded_facet(extension, &value) { located(file, &name, result)?; true } else { false };
                facets.push(SelectedFacet { subject: name.clone(), file: file.into(), policy: (*extension).into(), value: value.try_encode_to_vec().map_err(|e| error(e.to_string()))?, interpretation_available });
            }
        }
    }
    for nested in &message.nested_type {
        collect_fields(encoder, nested, &format!("{owner}.{}", nested.name.as_deref().unwrap_or_default()), file, false, selected_extensions, policy, fields, facets)?;
    }
    Ok(())
}

fn collect_extensions<'a>(file: &'a FileDescriptorProto, output: &mut Vec<(String, &'a str, u32, &'a str)>) {
    fn collect<'a>(fields: &'a [descriptor::FieldDescriptorProto], prefix: &str, file: &'a str, output: &mut Vec<(String, &'a str, u32, &'a str)>) {
        for field in fields { output.push((format!("{prefix}.{}", field.name.as_deref().unwrap_or_default()).trim_start_matches('.').into(), field.extendee.as_deref().unwrap_or_default().trim_start_matches('.'), field.number.unwrap_or_default() as u32, file)); }
    }
    fn nested<'a>(message: &'a descriptor::DescriptorProto, prefix: &str, file: &'a str, output: &mut Vec<(String, &'a str, u32, &'a str)>) {
        let owner = format!("{prefix}.{}", message.name.as_deref().unwrap_or_default());
        collect(&message.extension, &owner, file, output);
        for message in &message.nested_type { nested(message, &owner, file, output); }
    }
    let name = file.name.as_deref().unwrap_or_default();
    let package = file.package.as_deref().unwrap_or_default();
    collect(&file.extension, package, name, output);
    for message in &file.message_type { nested(message, package, name, output); }
}

fn value<'a>(message: &'a DynamicMessage, name: &str) -> Result<Option<&'a Value>, SchemaError> {
    let descriptor = message.message_descriptor();
    let field = descriptor.field_by_name(name).ok_or_else(|| error(format!("{}.{name}: unavailable typed metadata field", descriptor.full_name())))?;
    Ok(message.field_by_number(field.number()))
}
fn uint(message: &DynamicMessage, name: &str) -> Result<Option<u32>, SchemaError> { match value(message, name)? { None => Ok(None), Some(Value::U32(value)) => Ok(Some(*value)), _ => Err(error(format!("{name}: expected metadata uint32"))) } }
fn boolean(message: &DynamicMessage, name: &str) -> Result<Option<bool>, SchemaError> { match value(message, name)? { None => Ok(None), Some(Value::Bool(value)) => Ok(Some(*value)), _ => Err(error(format!("{name}: expected metadata bool"))) } }
fn text<'a>(message: &'a DynamicMessage, name: &str) -> Result<Option<&'a str>, SchemaError> { match value(message, name)? { None => Ok(None), Some(Value::String(value)) => Ok(Some(value)), _ => Err(error(format!("{name}: expected metadata string"))) } }
fn child<'a>(message: &'a DynamicMessage, name: &str) -> Result<Option<&'a DynamicMessage>, SchemaError> { match value(message, name)? { None => Ok(None), Some(Value::Message(value)) => Ok(Some(value)), _ => Err(error(format!("{name}: expected metadata message"))) } }
fn enumeration<T>(message: &DynamicMessage, name: &str, known: impl FnOnce(i32) -> Option<T>) -> Result<Option<core::EnumValue<T>>, SchemaError> {
    let raw = match value(message, name)? {
        Some(Value::EnumNumber(value)) => Some(*value),
        None => {
            let number = message.message_descriptor().field_by_name(name).ok_or_else(|| error("unavailable metadata enum"))?.number();
            message.unknown_fields().iter().find_map(|field| if field.number == number { if let buffa::UnknownFieldData::Varint(value) = field.data { Some(value as i32) } else { None } } else { None })
        },
        _ => return Err(error(format!("{name}: expected metadata enum"))),
    };
    Ok(raw.map(|number| known(number).map(core::EnumValue::Known).unwrap_or(core::EnumValue::Unknown(number))))
}
fn direction(number: i32) -> Option<Direction> { match number { 1 => Some(Direction::Input), 2 => Some(Direction::Output), 3 => Some(Direction::Both), _ => None } }
fn known_fields(message: &DynamicMessage, expected: &[&str]) -> Result<(), SchemaError> {
    for field in message.message_descriptor().fields() {
        if message.field_by_number(field.number()).is_some() && !expected.contains(&field.name()) { return Err(error(format!("{}: unsupported selected metadata field {}", message.message_descriptor().full_name(), field.name()))); }
    }
    for field in message.unknown_fields() {
        let known_enum_number = message.message_descriptor().field(field.number).is_some_and(|descriptor| matches!(descriptor.kind(), buffa_descriptor::FieldKind::Singular(buffa_descriptor::SingularKind::Enum(_)))) && matches!(field.data, buffa::UnknownFieldData::Varint(_));
        if !known_enum_number { return Err(error(format!("{}: unknown selected metadata field {}", message.message_descriptor().full_name(), field.number))); }
    }
    Ok(())
}
fn message_metadata(message: &DynamicMessage) -> Result<core::MessageMetadata<'_>, SchemaError> {
    known_fields(message, &["metadata_version", "owner", "interpretation_version", "direction", "construction_role", "effects"])?;
    Ok(core::MessageMetadata {
        metadata_version: uint(message, "metadata_version")?,
        owner: child(message, "owner")?.map(|owner| { known_fields(owner, &["rust_type", "declaration_file"])?; Ok(core::Owner { rust_type: text(owner, "rust_type")?.unwrap_or_default(), declaration_file: text(owner, "declaration_file")?.unwrap_or_default() }) }).transpose()?,
        interpretation_version: uint(message, "interpretation_version")?, direction: enumeration(message, "direction", direction)?,
        construction: enumeration(message, "construction_role", |number| match number { 1 => Some(ConstructionRole::Data), 2 => Some(ConstructionRole::ObservationOnly), _ => None })?,
        effects: child(message, "effects")?.map(effects_metadata).transpose()?,
    })
}
fn field_metadata(message: &DynamicMessage) -> Result<core::FieldMetadata<'_>, SchemaError> { known_fields(message, &["metadata_version", "native_field", "direction"])?; Ok(core::FieldMetadata { metadata_version: uint(message, "metadata_version")?, native_field: text(message, "native_field")?, direction: enumeration(message, "direction", direction)? }) }
fn method_metadata(message: &DynamicMessage) -> Result<core::MethodMetadata, SchemaError> {
    known_fields(message, &["metadata_version", "effects"])?;
    Ok(core::MethodMetadata { metadata_version: uint(message, "metadata_version")?, effects: child(message, "effects")?.map(effects_metadata).transpose()? })
}
fn effects_metadata(effects: &DynamicMessage) -> Result<core::Effects, SchemaError> {
    known_fields(effects, &["scope", "read_only", "idempotent", "destructive"])?;
    Ok(core::Effects {
        scope: enumeration(effects, "scope", |number| match number { 1 => Some(EffectScope::Observation), 2 => Some(EffectScope::SourceAdmission), 3 => Some(EffectScope::CanonicalCommand), 4 => Some(EffectScope::RetentionRelease), 5 => Some(EffectScope::FileExport), 6 => Some(EffectScope::Capture), 7 => Some(EffectScope::NativeExecution), _ => None })?,
        read_only: boolean(effects, "read_only")?, idempotent: boolean(effects, "idempotent")?, destructive: boolean(effects, "destructive")?,
    })
}
