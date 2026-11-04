// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Registry-owned, typed option publication. The string map is an internal source
//! carrier only; canonical structured facets are parsed by their native owners.

use buffa_descriptor::{DynamicMessage, Value};
use pse_contract::boundary_projection::{Field, ProjectionGraph, Shape, TypeRef};
use pse_schema::{SchemaError, model::{
    CollectionContract, CollectionOrder, ColumnRole, ExtensionUse, ExpressionGrammar,
    BoundaryShape, FieldContract, IntegerRange, ReferenceContract, ReferenceNullPolicy, TaggedAlternative,
    field_facets as keys,
}};

use super::{error, metadata::OptionEncoder};

pub(super) const IMPORT: &str = "pse/schema/v1/field_facets.proto";
pub(super) const EXTENSION: &str = "pse.schema.v1.registry_field";

#[cfg(test)]
mod tests;

fn vocabularies(pool: &buffa_descriptor::DescriptorPool) -> Result<(), SchemaError> {
    for (name, expected) in [
        ("ColumnRole", &["COLUMN_ROLE_UNSPECIFIED", "COLUMN_ROLE_KEY", "COLUMN_ROLE_REFERENCE", "COLUMN_ROLE_MEASURE", "COLUMN_ROLE_LABEL", "COLUMN_ROLE_PAYLOAD", "COLUMN_ROLE_PROVENANCE"][..]),
        ("SimpleExtension", &["SIMPLE_EXTENSION_UNSPECIFIED", "SIMPLE_EXTENSION_SEMANTIC_ID", "SIMPLE_EXTENSION_CONTENT_HASH", "SIMPLE_EXTENSION_DIMENSION_VECTOR", "SIMPLE_EXTENSION_QUANTITY_VALUE", "SIMPLE_EXTENSION_BOUND", "SIMPLE_EXTENSION_INDEX_TUPLE", "SIMPLE_EXTENSION_SOURCE_SPAN", "SIMPLE_EXTENSION_TARGET_PATH"][..]),
        ("ExpressionGrammar", &["EXPRESSION_GRAMMAR_UNSPECIFIED", "EXPRESSION_GRAMMAR_EXPRESSION", "EXPRESSION_GRAMMAR_EQUATION", "EXPRESSION_GRAMMAR_PREDICATE", "EXPRESSION_GRAMMAR_PROPOSITION", "EXPRESSION_GRAMMAR_STATIC", "EXPRESSION_GRAMMAR_ANNOTATION_ARGUMENTS", "EXPRESSION_GRAMMAR_ANNOTATION_TARGET"][..]),
        ("GraphEndpoint", &["GRAPH_ENDPOINT_UNSPECIFIED", "GRAPH_ENDPOINT_INCOMING", "GRAPH_ENDPOINT_OUTGOING"][..]),
        ("CollectionOrder", &["COLLECTION_ORDER_UNSPECIFIED", "COLLECTION_ORDER_SEQUENCE", "COLLECTION_ORDER_UNORDERED"][..]),
        ("ReferenceNullPolicy", &["REFERENCE_NULL_POLICY_UNSPECIFIED", "REFERENCE_NULL_POLICY_REQUIRED", "REFERENCE_NULL_POLICY_ALL_OR_NONE"][..]),
    ] {
        let full_name = format!("pse.schema.v1.{name}");
        let enumeration = pool.enum_by_name(&full_name).ok_or_else(|| error(format!("{full_name}: missing native vocabulary")))?;
        if enumeration.values().len() != expected.len() || expected.iter().enumerate().any(|(number, name)| enumeration.value_by_name(name).is_none_or(|value| value.number() != number as i32)) {
            return Err(error(format!("{full_name}: schema/native vocabulary correspondence changed")));
        }
    }
    Ok(())
}

fn message(encoder: &OptionEncoder, name: &str, fields: &[(&str, Value)]) -> Result<Value, SchemaError> {
    encoder.message(&format!("pse.schema.v1.{name}"), fields).map(Value::Message)
}

fn extension(value: ExtensionUse<'_>, encoder: &OptionEncoder) -> Result<Value, SchemaError> {
    let field = match value {
        ExtensionUse::SemanticId => ("simple", Value::EnumNumber(1)),
        ExtensionUse::ContentHash => ("simple", Value::EnumNumber(2)),
        ExtensionUse::DimensionVector => ("simple", Value::EnumNumber(3)),
        ExtensionUse::QuantityValue => ("simple", Value::EnumNumber(4)),
        ExtensionUse::Bound => ("simple", Value::EnumNumber(5)),
        ExtensionUse::IndexTuple => ("simple", Value::EnumNumber(6)),
        ExtensionUse::SourceSpan => ("simple", Value::EnumNumber(7)),
        ExtensionUse::TargetPath => ("simple", Value::EnumNumber(8)),
        ExtensionUse::Enum(name) => ("enumeration", Value::String(name.into())),
        ExtensionUse::OrdinalRef { target } => ("ordinal_target_relation", Value::String(target.into())),
        ExtensionUse::ExprDsl(grammar) => ("expression_grammar", Value::EnumNumber(match grammar {
            ExpressionGrammar::Expression => 1, ExpressionGrammar::Equation => 2,
            ExpressionGrammar::Predicate => 3, ExpressionGrammar::Proposition => 4,
            ExpressionGrammar::Static => 5, ExpressionGrammar::AnnotationArguments => 6,
            ExpressionGrammar::AnnotationTarget => 7,
        })),
    };
    message(encoder, "ExtensionUse", &[field])
}

/// Lower every selected annotation or refuse it with its native field/key attribution.
pub(super) fn field_options(field: &Field, graph: &ProjectionGraph, encoder: &OptionEncoder) -> Result<Option<DynamicMessage>, SchemaError> {
    if field.named_facets.is_empty() { return Ok(None); }
    let path = &field.name;
    let invalid = |key: &str, reason: &str| error(format!("{path}.{key}: {reason}"));
    let get = |key: &str| field.named_facets.get(key).map(String::as_str);
    let mut fields = vec![("metadata_version", Value::U32(1))];
    if let Some(text) = get(keys::BOUNDARY_SHAPE) { check_shape(field, graph, text)?; }
    if let Some(name) = get(keys::EXTENSION) {
        fields.push(("extension", extension(ExtensionUse::parse_metadata(name, get(keys::PARAMETER), path)?, encoder)?));
    } else if get(keys::PARAMETER).is_some() {
        return Err(invalid(keys::PARAMETER, "parameter has no declared extension"));
    }
    if let Some(role) = get(keys::ROLE) {
        fields.push(("role", Value::EnumNumber(match FieldContract::parse_role_metadata(role, path)? {
            ColumnRole::Key => 1, ColumnRole::Reference => 2, ColumnRole::Measure => 3,
            ColumnRole::Label => 4, ColumnRole::Payload => 5, ColumnRole::Provenance => 6,
        })));
    }
    for (key, name) in [(keys::DOC, "documentation"), (keys::STRUCTURE, "structure_name")] {
        if let Some(text) = get(key) { fields.push((name, Value::String(text.into()))); }
    }
    if let Some(name) = get(keys::QUANTITY) {
        if name.is_empty() { return Err(invalid(keys::QUANTITY, "quantity reference is empty")); }
        fields.push(("quantity", message(encoder, "QuantityReference", &[("qualified_name", Value::String(name.into()))])?));
    }
    if let Some(version) = get(keys::TRANSFER_CONTEXT) {
        if version != "1" { return Err(invalid(keys::TRANSFER_CONTEXT, "unknown native transfer context version")); }
        fields.push(("transfer_context_version", Value::U32(1)));
    }
    let scalar_reference = match (get(keys::FK_RELATION), get(keys::FK_COLUMN)) {
        (None, None) => None,
        (Some(relation), Some(column)) if !relation.is_empty() && !column.is_empty() => Some((relation, column)),
        _ => return Err(invalid(keys::FK_RELATION, "incomplete scalar foreign key")),
    };
    if let Some(text) = get(keys::KEY_REFERENCE) {
        let reference = ReferenceContract::parse_metadata(text, path)?;
        if let Some((relation, column)) = scalar_reference
            && (reference.relation != relation || reference.columns.len() != 1
                || !reference.columns[0].source.is_empty() || reference.columns[0].target != column)
        { return Err(invalid(keys::KEY_REFERENCE, "normalized reference disagrees with its scalar native declaration")); }
        let columns = reference.columns.iter().map(|column| message(encoder, "ReferenceColumn", &[
            ("source", Value::List(column.source.iter().cloned().map(Value::String).collect())),
            ("target", Value::String(column.target.clone())),
        ])).collect::<Result<Vec<_>, _>>()?;
        fields.push(("reference", message(encoder, "ReferenceContract", &[
            ("relation", Value::String(reference.relation)), ("columns", Value::List(columns)),
            ("null_policy", Value::EnumNumber(match reference.null_policy { ReferenceNullPolicy::Required => 1, ReferenceNullPolicy::AllOrNone => 2 })),
        ])?));
    } else if scalar_reference.is_some() {
        return Err(invalid(keys::FK_RELATION, "source owner must normalize reference nullability before projection"));
    }
    if let Some(endpoint) = get(keys::GRAPH_ENDPOINT) {
        let number = match endpoint { "in" => 1, "out" => 2, _ => return Err(invalid(keys::GRAPH_ENDPOINT, "unknown graph endpoint")) };
        if get(keys::KEY_REFERENCE).is_none() { return Err(invalid(keys::GRAPH_ENDPOINT, "graph endpoint has no reference")); }
        fields.push(("graph_endpoint", Value::EnumNumber(number)));
    }
    if let Some(text) = get(keys::KEY_COLLECTION) {
        let collection = CollectionContract::parse_metadata(text, path)?;
        let mut values = vec![
            ("order", Value::EnumNumber(match collection.order { CollectionOrder::Sequence => 1, CollectionOrder::Unordered => 2 })),
            ("minimum", Value::I64(collection.minimum)), ("unique", Value::Bool(collection.unique)),
        ];
        if let Some(maximum) = collection.maximum { values.push(("maximum", Value::I64(maximum))); }
        fields.push(("collection", message(encoder, "CollectionContract", &values)?));
    }
    if let Some(text) = get(keys::KEY_INTEGER_RANGE) {
        let range = IntegerRange::parse_metadata(text, path)?;
        fields.push(("integer_range", message(encoder, "IntegerRange", &[
            ("minimum", Value::I64(range.minimum)), ("maximum", Value::I64(range.maximum)),
        ])?));
    }
    if let Some(text) = get(keys::KEY_TAGGED_ALTERNATIVE) {
        let alternative = TaggedAlternative::parse_metadata(text, path)?;
        let arms = alternative.arms.into_iter().map(|(tag, payload)| {
            let mut values = vec![("tag", Value::String(tag))];
            if let Some(payload) = payload { values.push(("payload_field", Value::String(payload))); }
            message(encoder, "AlternativeArm", &values)
        }).collect::<Result<Vec<_>, _>>()?;
        fields.push(("tagged_alternative", message(encoder, "TaggedAlternative", &[
            ("discriminator", Value::String(alternative.discriminator)), ("arms", Value::List(arms)),
        ])?));
    }
    for (key, value) in &field.named_facets {
        match key.as_str() {
            keys::BOUNDARY_TAG => {
                if value != &field.tag.to_string() { return Err(invalid(key, "source boundary tag differs from the selected descriptor number")); }
            }
            keys::BOUNDARY_SHAPE | keys::EXTENSION | keys::PARAMETER | keys::ROLE | keys::DOC | keys::STRUCTURE
            | keys::QUANTITY | keys::TRANSFER_CONTEXT | keys::FK_RELATION | keys::FK_COLUMN
            | keys::GRAPH_ENDPOINT | keys::KEY_REFERENCE | keys::KEY_COLLECTION
            | keys::KEY_INTEGER_RANGE | keys::KEY_TAGGED_ALTERNATIVE => {}
            _ => return Err(invalid(key, "selected facet has no typed protobuf interpretation; add its owner policy before publication")),
        }
    }
    let value = encoder.message("pse.schema.v1.RegistryFieldFacets", &fields)?;
    check_option(&value)?;
    Ok(Some(value))
}

fn check_shape(field: &Field, graph: &ProjectionGraph, text: &str) -> Result<(), SchemaError> {
    let shape = BoundaryShape::parse_metadata(text, &field.name)?;
    let invalid = || error(format!("{}.{}: source presentation disagrees with actual native graph owner", field.name, keys::BOUNDARY_SHAPE));
    let mut ty = &field.ty;
    while let TypeRef::Optional(inner) = ty { ty = inner; }
    let TypeRef::Named(name) = ty else { return Err(invalid()); };
    let definition = graph.definitions().get(name).ok_or_else(invalid)?;
    if shape.name.as_ref().is_some_and(|name| name != &definition.name)
        || shape.source != definition.source || shape.version != definition.version
        || shape.direction != definition.direction || shape.reserved_tags != definition.reserved_tags
        || shape.reserved_names != definition.reserved_names
    { return Err(invalid()); }
    let alternatives = match &definition.shape {
        Shape::Record(_) => Vec::new(),
        Shape::Alternatives(arms) => arms.iter().map(|arm| (arm.name.clone(), arm.tag)).collect(),
        Shape::StringEnum(_) => return Err(invalid()),
    };
    let mut expected = shape.alternatives;
    let mut actual = alternatives;
    expected.sort(); actual.sort();
    if expected != actual { return Err(invalid()); }
    Ok(())
}

fn value<'a>(message: &'a DynamicMessage, name: &str) -> Option<&'a Value> {
    let descriptor = message.pool().message(message.message_index());
    descriptor.field_by_name(name).and_then(|field| message.field_by_number(field.number()))
}

fn required<'a>(message: &'a DynamicMessage, name: &str) -> Result<&'a Value, SchemaError> {
    value(message, name).ok_or_else(|| error(format!("typed registry option requires {name}")))
}

fn text(message: &DynamicMessage, name: &str) -> Result<String, SchemaError> {
    match required(message, name)? { Value::String(text) => Ok(text.clone()), _ => Err(error(format!("{name}: expected string"))) }
}

fn integer(message: &DynamicMessage, name: &str) -> Result<i64, SchemaError> {
    match required(message, name)? { Value::I64(value) => Ok(*value), _ => Err(error(format!("{name}: expected int64"))) }
}

fn enumeration(message: &DynamicMessage, name: &str, maximum: i32) -> Result<i32, SchemaError> {
    match required(message, name)? { Value::EnumNumber(value) if (1..=maximum).contains(value) => Ok(*value), _ => Err(error(format!("{name}: unknown native vocabulary value"))) }
}

fn nested<'a>(message: &'a DynamicMessage, name: &str) -> Result<&'a DynamicMessage, SchemaError> {
    match required(message, name)? { Value::Message(value) => Ok(value), _ => Err(error(format!("{name}: expected typed message"))) }
}

fn list<'a>(message: &'a DynamicMessage, name: &str) -> Result<&'a [Value], SchemaError> {
    match value(message, name) { Some(Value::List(values)) => Ok(values), None => Ok(&[]), _ => Err(error(format!("{name}: expected repeated field"))) }
}

fn closed(message: &DynamicMessage, name: &str, fields: &[&str]) -> Result<(), SchemaError> {
    let descriptor = message.pool().message(message.message_index());
    if descriptor.full_name() != format!("pse.schema.v1.{name}") || !message.unknown_fields().is_empty()
        || descriptor.fields().iter().any(|field| value(message, field.name()).is_some() && !fields.contains(&field.name()))
    { return Err(error(format!("{name}: unknown selected typed option shape"))); }
    Ok(())
}

/// Interpret selected extension payloads through the same native domain contracts.
/// Reading an option does not admit Arrow storage, resolve quantities or run a catalog.
pub(super) fn check_option(option: &DynamicMessage) -> Result<(), SchemaError> {
    vocabularies(option.pool())?;
    closed(option, "RegistryFieldFacets", &["metadata_version", "extension", "role", "documentation", "structure_name", "quantity", "transfer_context_version", "reference", "graph_endpoint", "collection", "integer_range", "tagged_alternative"])?;
    if required(option, "metadata_version")? != &Value::U32(1) { return Err(error("unknown registry metadata version")); }
    if value(option, "role").is_some() { enumeration(option, "role", 6)?; }
    for name in ["documentation", "structure_name"] { if value(option, name).is_some() { text(option, name)?; } }
    if value(option, "extension").is_some() {
        let extension = nested(option, "extension")?;
        let forms = ["simple", "enumeration", "ordinal_target_relation", "expression_grammar"];
        closed(extension, "ExtensionUse", &forms)?;
        if forms.iter().filter(|name| value(extension, name).is_some()).count() != 1 { return Err(error("extension requires exactly one native form")); }
        for (name, maximum) in [("simple", 8), ("expression_grammar", 7)] { if value(extension, name).is_some() { enumeration(extension, name, maximum)?; } }
        for name in ["enumeration", "ordinal_target_relation"] { if value(extension, name).is_some() && text(extension, name)?.is_empty() { return Err(error("extension parameter is empty")); } }
    }
    if value(option, "quantity").is_some() {
        let quantity = nested(option, "quantity")?;
        closed(quantity, "QuantityReference", &["qualified_name"])?;
        if text(quantity, "qualified_name")?.is_empty() { return Err(error("empty quantity reference")); }
    }
    if value(option, "transfer_context_version").is_some() && required(option, "transfer_context_version")? != &Value::U32(1) { return Err(error("unknown transfer context version")); }
    if value(option, "reference").is_some() {
        let reference = nested(option, "reference")?;
        closed(reference, "ReferenceContract", &["relation", "columns", "null_policy"])?;
        let columns = list(reference, "columns")?.iter().map(|column| {
            let Value::Message(column) = column else { return Err(error("reference column is not typed")); };
            closed(column, "ReferenceColumn", &["source", "target"])?;
            let source = list(column, "source")?.iter().map(|part| match part { Value::String(part) => Ok(part.clone()), _ => Err(error("reference path segment is not a string")) }).collect::<Result<Vec<_>, _>>()?;
            Ok(pse_schema::model::ReferenceColumn { source, target: text(column, "target")? })
        }).collect::<Result<Vec<_>, SchemaError>>()?;
        let native = ReferenceContract { relation: text(reference, "relation")?, columns, null_policy: match enumeration(reference, "null_policy", 2)? { 1 => ReferenceNullPolicy::Required, _ => ReferenceNullPolicy::AllOrNone } };
        ReferenceContract::parse_metadata(&native.canonical()?, "typed registry reference")?;
    }
    if value(option, "graph_endpoint").is_some() {
        enumeration(option, "graph_endpoint", 2)?;
        if value(option, "reference").is_none() { return Err(error("graph endpoint has no reference")); }
    }
    if value(option, "collection").is_some() {
        let collection = nested(option, "collection")?;
        closed(collection, "CollectionContract", &["order", "minimum", "maximum", "unique"])?;
        let native = CollectionContract {
            order: if enumeration(collection, "order", 2)? == 1 { CollectionOrder::Sequence } else { CollectionOrder::Unordered },
            minimum: integer(collection, "minimum")?, maximum: value(collection, "maximum").map(|_| integer(collection, "maximum")).transpose()?,
            unique: match required(collection, "unique")? { Value::Bool(value) => *value, _ => return Err(error("collection unique is not bool")) },
        };
        CollectionContract::parse_metadata(&native.canonical(), "typed registry collection")?;
    }
    if value(option, "integer_range").is_some() {
        let range = nested(option, "integer_range")?;
        closed(range, "IntegerRange", &["minimum", "maximum"])?;
        let native = IntegerRange { minimum: integer(range, "minimum")?, maximum: integer(range, "maximum")? };
        IntegerRange::parse_metadata(&native.canonical(), "typed registry integer range")?;
    }
    if value(option, "tagged_alternative").is_some() {
        let alternative = nested(option, "tagged_alternative")?;
        closed(alternative, "TaggedAlternative", &["discriminator", "arms"])?;
        let mut arms = std::collections::BTreeMap::new();
        for arm in list(alternative, "arms")? {
            let Value::Message(arm) = arm else { return Err(error("alternative arm is not typed")); };
            closed(arm, "AlternativeArm", &["tag", "payload_field"])?;
            if arms.insert(text(arm, "tag")?, value(arm, "payload_field").map(|_| text(arm, "payload_field")).transpose()?).is_some() { return Err(error("duplicate alternative tag")); }
        }
        let native = TaggedAlternative { discriminator: text(alternative, "discriminator")?, arms };
        TaggedAlternative::parse_metadata(&native.canonical(), "typed registry alternative")?;
    }
    Ok(())
}
