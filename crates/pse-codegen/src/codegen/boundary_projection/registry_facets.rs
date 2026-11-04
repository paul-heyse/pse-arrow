// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Registry-owned, typed option publication. The string map is an internal source
//! carrier only; canonical structured facets are parsed by their native owners.

use buffa_descriptor::{DynamicMessage, Value, ReflectMessage};
use pse_contract::boundary_projection::{Field, ProjectionGraph, Shape, TypeRef};
use pse_schema::{model::{
    CollectionContract, CollectionOrder, ColumnRole, ExtensionUse, ExpressionGrammar,
    BoundaryShape, CanonicalKeyKind, ExpressionBinder, ExpressionOrdinal,
    ExpressionSourceContract, FieldContract, IdentityContract, IntegerRange, ReferenceContract, ReferenceNullPolicy, TaggedAlternative,
    field_facets as keys,
}};

use super::{error, ProjectionCodegenError};
use pse_contract_proto::OptionEncoder;

pub(super) const IMPORT: &str = "pse/schema/v1/field_facets.proto";
pub(super) const EXTENSION: &str = "pse.schema.v1.registry_field";

#[cfg(test)]
mod tests;
mod native_ipc;

pub(super) fn has_options(field: &Field) -> bool { !field.named_facets.is_empty() }

// One source-key -> typed-field correspondence is shared by coverage and emission
// validation. This never walks an acquired descriptor to approve its own scope.
const SOURCE_FIELDS: &[(&str, &str)] = &[
    (keys::EXTENSION, "extension"), (keys::PARAMETER, "extension"), (keys::ROLE, "role"),
    (keys::DOC, "documentation"), (keys::STRUCTURE, "structure_name"),
    (keys::QUANTITY, "quantity"), (keys::TRANSFER_CONTEXT, "transfer_context_version"),
    (keys::KEY_REFERENCE, "reference"), (keys::FK_RELATION, "reference"), (keys::FK_COLUMN, "reference"),
    (keys::GRAPH_ENDPOINT, "graph_endpoint"), (keys::KEY_COLLECTION, "collection"),
    (keys::KEY_INTEGER_RANGE, "integer_range"), (keys::KEY_TAGGED_ALTERNATIVE, "tagged_alternative"),
    (keys::IDENTITY, "identity"), (keys::IDENTITY_OWNER, "identity"),
    (keys::CANONICAL_KEY, "canonical.key_kind"), (keys::CANONICAL_READONLY, "canonical.readonly"), (keys::CANONICAL_RECORD_KEY, "canonical.record_key"),
    (keys::CANONICAL_ASSERT, "canonical.assertion"), (keys::CANONICAL_DEFAULT, "canonical.default_expression"), (keys::CANONICAL_TRANSITION, "canonical.initial_member"),
    (keys::CANONICAL_TRANSITION_EXCEPTION, "canonical.transition_exception"), (keys::CANONICAL_SEALED, "canonical.sealed"), (keys::CANONICAL_SEAL_REFUSAL, "canonical.seal_refusal"), (keys::CANONICAL_REFERENCE_KIND, "canonical.reference_kind"),
    (keys::EXPRESSION_ROLE, "expression_source"), (keys::EXPRESSION_ORDINAL, "expression_source"), (keys::EXPRESSION_BINDER, "expression_source"),
    (super::REGISTRY_FIELD_SOURCE, "native_field_ipc"),
];
pub(super) fn required_fields(field: &Field) -> Vec<&'static str> {
    let mut result = vec!["metadata_version"];
    for (key, target) in SOURCE_FIELDS { if field.named_facets.contains_key(*key) && !result.contains(target) { result.push(target); } }
    result
}

fn vocabularies(pool: &buffa_descriptor::DescriptorPool) -> Result<(), ProjectionCodegenError> {
    schema_shapes(pool)?;
    let extension = pool.extension_by_name(EXTENSION).ok_or_else(|| error("registry option extension is absent from its import world"))?;
    let option = pool.message_index("pse.schema.v1.RegistryFieldFacets").ok_or_else(|| error("registry option message is absent"))?;
    if extension.field().number() != 51066
        || pool.message(extension.extendee()).full_name() != "google.protobuf.FieldOptions"
        || extension.field().kind() != buffa_descriptor::FieldKind::Singular(buffa_descriptor::SingularKind::Message(option))
    { return Err(error("registry extension allocation, extendee or value type changed")); }
    for (name, expected) in [
        ("ColumnRole", &["COLUMN_ROLE_UNSPECIFIED", "COLUMN_ROLE_KEY", "COLUMN_ROLE_REFERENCE", "COLUMN_ROLE_MEASURE", "COLUMN_ROLE_LABEL", "COLUMN_ROLE_PAYLOAD", "COLUMN_ROLE_PROVENANCE"][..]),
        ("SimpleExtension", &["SIMPLE_EXTENSION_UNSPECIFIED", "SIMPLE_EXTENSION_SEMANTIC_ID", "SIMPLE_EXTENSION_CONTENT_HASH", "SIMPLE_EXTENSION_DIMENSION_VECTOR", "SIMPLE_EXTENSION_QUANTITY_VALUE", "SIMPLE_EXTENSION_BOUND", "SIMPLE_EXTENSION_INDEX_TUPLE", "SIMPLE_EXTENSION_SOURCE_SPAN", "SIMPLE_EXTENSION_TARGET_PATH"][..]),
        ("ExpressionGrammar", &["EXPRESSION_GRAMMAR_UNSPECIFIED", "EXPRESSION_GRAMMAR_EXPRESSION", "EXPRESSION_GRAMMAR_EQUATION", "EXPRESSION_GRAMMAR_PREDICATE", "EXPRESSION_GRAMMAR_PROPOSITION", "EXPRESSION_GRAMMAR_STATIC", "EXPRESSION_GRAMMAR_ANNOTATION_ARGUMENTS", "EXPRESSION_GRAMMAR_ANNOTATION_TARGET"][..]),
        ("GraphEndpoint", &["GRAPH_ENDPOINT_UNSPECIFIED", "GRAPH_ENDPOINT_INCOMING", "GRAPH_ENDPOINT_OUTGOING"][..]),
        ("CollectionOrder", &["COLLECTION_ORDER_UNSPECIFIED", "COLLECTION_ORDER_SEQUENCE", "COLLECTION_ORDER_UNORDERED"][..]),
        ("ReferenceNullPolicy", &["REFERENCE_NULL_POLICY_UNSPECIFIED", "REFERENCE_NULL_POLICY_REQUIRED", "REFERENCE_NULL_POLICY_ALL_OR_NONE"][..]),
        ("ExpressionOrdinal", &["EXPRESSION_ORDINAL_UNSPECIFIED", "EXPRESSION_ORDINAL_LIST", "EXPRESSION_ORDINAL_ZERO", "EXPRESSION_ORDINAL_FLATTEN"][..]),
        ("ExpressionBinder", &["EXPRESSION_BINDER_UNSPECIFIED", "EXPRESSION_BINDER_DOMAIN_BEFORE_NAME"][..]),
    ] {
        let full_name = format!("pse.schema.v1.{name}");
        let enumeration = pool.enum_by_name(&full_name).ok_or_else(|| error(format!("{full_name}: missing native vocabulary")))?;
        if enumeration.values().len() != expected.len() || expected.iter().enumerate().any(|(number, name)| enumeration.value_by_name(name).is_none_or(|value| value.number() != number as i32)) {
            return Err(error(format!("{full_name}: schema/native vocabulary correspondence changed")));
        }
    }
    let keys = pool.enum_by_name("pse.schema.v1.CanonicalKeyKind").ok_or_else(|| error("missing canonical key vocabulary"))?;
    if keys.values().len() != CanonicalKeyKind::ALL.len() + 1
        || keys.value_by_name("CANONICAL_KEY_KIND_UNSPECIFIED").is_none_or(|value| value.number() != 0)
    { return Err(error("canonical key schema/native vocabulary correspondence changed")); }
    for kind in CanonicalKeyKind::ALL { canonical_number(pool, *kind)?; }
    Ok(())
}

// Acceptance contract of the known v1 interpreter, not a descriptor producer.
fn schema_shapes(pool: &buffa_descriptor::DescriptorPool) -> Result<(), ProjectionCodegenError> {
    use buffa_descriptor::{FieldKind::{Singular, List}, SingularKind::{Scalar, Message, Enum}, ScalarType::{Uint32, Int64, String, Bool, Bytes}};
    let message = |name: &str| pool.message_index(&format!("pse.schema.v1.{name}")).map(Message).ok_or_else(|| error(format!("missing registry option message {name}")));
    let enumeration = |name: &str| pool.enum_index(&format!("pse.schema.v1.{name}")).map(Enum).map(Singular).ok_or_else(|| error(format!("missing registry option enum {name}")));
    let scalar = |kind| Singular(Scalar(kind));
    for (name, expected) in [
        ("RegistryFieldFacets", vec![("metadata_version", scalar(Uint32)), ("extension", Singular(message("ExtensionUse")?)), ("role", enumeration("ColumnRole")?), ("documentation", scalar(String)), ("structure_name", scalar(String)), ("quantity", Singular(message("QuantityReference")?)), ("transfer_context_version", scalar(Uint32)), ("reference", Singular(message("ReferenceContract")?)), ("graph_endpoint", enumeration("GraphEndpoint")?), ("collection", Singular(message("CollectionContract")?)), ("integer_range", Singular(message("IntegerRange")?)), ("tagged_alternative", Singular(message("TaggedAlternative")?)), ("identity", Singular(message("IdentityContract")?)), ("canonical", Singular(message("CanonicalFieldContract")?)), ("expression_source", Singular(message("ExpressionSourceContract")?)), ("native_field_ipc", scalar(Bytes))]),
        ("IdentityContract", vec![("entity_name", scalar(String)), ("owns_identity", scalar(Bool))]),
        ("CanonicalFieldContract", vec![("key_kind", enumeration("CanonicalKeyKind")?), ("readonly", scalar(Bool)), ("record_key", scalar(Bool)), ("assertion", scalar(String)), ("default_expression", scalar(String)), ("initial_member", scalar(String)), ("transition_exception", scalar(String)), ("sealed", scalar(String)), ("seal_refusal", scalar(String)), ("reference_kind", scalar(String))]),
        ("ExpressionSourceContract", vec![("role_path_template", scalar(String)), ("ordinal", enumeration("ExpressionOrdinal")?), ("binder", enumeration("ExpressionBinder")?)]),
        ("ExtensionUse", vec![("simple", enumeration("SimpleExtension")?), ("enumeration", scalar(String)), ("ordinal_target_relation", scalar(String)), ("expression_grammar", enumeration("ExpressionGrammar")?)]),
        ("QuantityReference", vec![("qualified_name", scalar(String))]),
        ("CollectionContract", vec![("order", enumeration("CollectionOrder")?), ("minimum", scalar(Int64)), ("maximum", scalar(Int64)), ("unique", scalar(Bool))]),
        ("IntegerRange", vec![("minimum", scalar(Int64)), ("maximum", scalar(Int64))]),
        ("ReferenceColumn", vec![("source", List(Scalar(String))), ("target", scalar(String))]),
        ("ReferenceContract", vec![("relation", scalar(String)), ("columns", List(message("ReferenceColumn")?)), ("null_policy", enumeration("ReferenceNullPolicy")?)]),
        ("AlternativeArm", vec![("tag", scalar(String)), ("payload_field", scalar(String))]),
        ("TaggedAlternative", vec![("discriminator", scalar(String)), ("arms", List(message("AlternativeArm")?))]),
    ] {
        let descriptor = pool.message_by_name(&format!("pse.schema.v1.{name}")).ok_or_else(|| error(format!("missing registry option shape {name}")))?;
        let has_oneof = name == "ExtensionUse";
        if descriptor.fields().len() != expected.len() || !descriptor.extension_ranges().is_empty()
            || descriptor.oneofs().len() != usize::from(has_oneof)
            || (has_oneof && descriptor.oneofs()[0].name() != "form")
            || expected.iter().enumerate().any(|(index, (name, kind))| descriptor.field_by_name(name).is_none_or(|field| {
                field.number() != (index + 1) as u32 || field.kind() != *kind || field.is_delimited()
                    || field.oneof_index() != has_oneof.then_some(0)
                    || field.presence() != if matches!(kind, List(_)) { buffa::editions::FieldPresence::Implicit } else { buffa::editions::FieldPresence::Explicit }
            }))
        { return Err(error(format!("pse.schema.v1.{name}: unsupported v1 registry option schema correspondence"))); }
    }
    let raw = pool.file_by_name(IMPORT).ok_or_else(|| error("registry option source schema is absent"))?;
    if raw.message_type.iter().flat_map(|message| &message.field).any(|field| field.default_value.is_some()) {
        return Err(error("registry option schema cannot redefine absent source facets through defaults"));
    }
    Ok(())
}

fn canonical_number(pool: &buffa_descriptor::DescriptorPool, kind: CanonicalKeyKind) -> Result<i32, ProjectionCodegenError> {
    let expected = match kind {
        CanonicalKeyKind::Problem => 1, CanonicalKeyKind::Revision => 2, CanonicalKeyKind::Run => 3,
        CanonicalKeyKind::Attempt => 4, CanonicalKeyKind::Study => 5, CanonicalKeyKind::StudyPoint => 6,
        CanonicalKeyKind::Operation => 7, CanonicalKeyKind::Artifact => 8, CanonicalKeyKind::FileInstance => 9,
        CanonicalKeyKind::Project => 10, CanonicalKeyKind::ProjectBranch => 11, CanonicalKeyKind::ProjectDraft => 12,
        CanonicalKeyKind::DraftVersion => 13, CanonicalKeyKind::ProjectSavepoint => 14, CanonicalKeyKind::ProjectBookmark => 15,
        CanonicalKeyKind::ProjectAncestry => 16, CanonicalKeyKind::DraftDiagnostic => 17, CanonicalKeyKind::ProjectContext => 18,
        CanonicalKeyKind::ContextAncestry => 19, CanonicalKeyKind::ResearchReference => 20, CanonicalKeyKind::ResearchDataset => 21,
        CanonicalKeyKind::ResearchSelection => 22, CanonicalKeyKind::ResearchPublication => 23, CanonicalKeyKind::ProjectImport => 24,
        CanonicalKeyKind::Product => 25, CanonicalKeyKind::BodyContent => 26, CanonicalKeyKind::SelectionReceipt => 27,
        CanonicalKeyKind::ProductAssociation => 28, CanonicalKeyKind::ProductDescriptor => 29, CanonicalKeyKind::Analysis => 30,
        CanonicalKeyKind::AnalysisNode => 31, CanonicalKeyKind::MaintenanceOperation => 32, CanonicalKeyKind::MaintenanceActivation => 33,
        CanonicalKeyKind::InlinePayload => 34, CanonicalKeyKind::FileOwner => 35, CanonicalKeyKind::FileBorrower => 36,
        CanonicalKeyKind::ManifestTicket => 37,
    };
    let name = format!("CANONICAL_KEY_KIND_{}", kind.as_str().to_ascii_uppercase());
    pool.enum_by_name("pse.schema.v1.CanonicalKeyKind").and_then(|enumeration| enumeration.value_by_name(&name))
        .filter(|value| value.number() == expected).map(|value| value.number())
        .ok_or_else(|| error(format!("{name}: missing native canonical key correspondence")))
}

fn message(encoder: &OptionEncoder, name: &str, fields: &[(&str, Value)]) -> Result<Value, ProjectionCodegenError> {
    Ok(Value::Message(encoder.message(&format!("pse.schema.v1.{name}"), fields)?))
}

fn extension(value: ExtensionUse<'_>, encoder: &OptionEncoder) -> Result<Value, ProjectionCodegenError> {
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
pub(super) fn field_options(field: &Field, graph: &ProjectionGraph, encoder: &OptionEncoder) -> Result<Option<DynamicMessage>, ProjectionCodegenError> {
    let value = lower_field_options(field, graph, encoder)?;
    if let Some(value) = &value { check_option(value)?; }
    Ok(value)
}

fn lower_field_options(field: &Field, graph: &ProjectionGraph, encoder: &OptionEncoder) -> Result<Option<DynamicMessage>, ProjectionCodegenError> {
    if !has_options(field) { return Ok(None); }
    let path = &field.name;
    let invalid = |key: &str, reason: &str| error(format!("{path}.{key}: {reason}"));
    let get = |key: &str| field.named_facets.get(key).map(String::as_str);
    let mut fields = vec![("metadata_version", Value::U32(2))];
    if let Some(source) = get(super::REGISTRY_FIELD_SOURCE) {
        let native: arrow_schema::Field = serde_json::from_str(source)?;
        if native.name() != &field.name { return Err(invalid(super::REGISTRY_FIELD_SOURCE, "native IPC field differs from its source field")); }
        fields.push(("native_field_ipc", Value::Bytes(native_ipc::encode(&native)?)));
    }
    if let Some(text) = get(keys::BOUNDARY_SHAPE) { check_shape(field, graph, text)?; }
    if let Some(identity) = IdentityContract::parse_metadata(get(keys::IDENTITY), get(keys::IDENTITY_OWNER), path)? {
        fields.push(("identity", message(encoder, "IdentityContract", &[
            ("entity_name", Value::String(identity.name.into())),
            ("owns_identity", Value::Bool(identity.owns_identity)),
        ])?));
    }
    let source_metadata = field.named_facets.iter().map(|(key, value)| (key.clone(), value.clone())).collect();
    if let Some(policy) = pse_schema::model::CanonicalFieldPolicy::parse_metadata(&source_metadata, path)? {
        fields.push(("canonical", Value::Message(canonical_message(&policy, encoder)?)));
    }
    if let Some(expression) = ExpressionSourceContract::parse_metadata(get(keys::EXPRESSION_ROLE), get(keys::EXPRESSION_ORDINAL), get(keys::EXPRESSION_BINDER), path)? {
        let mut values = Vec::new();
        if let Some(role) = expression.role { values.push(("role_path_template", Value::String(role.into()))); }
        if let Some(ordinal) = expression.ordinal { values.push(("ordinal", Value::EnumNumber(match ordinal { ExpressionOrdinal::List => 1, ExpressionOrdinal::Zero => 2, ExpressionOrdinal::Flatten => 3 }))); }
        if let Some(ExpressionBinder::DomainBeforeName) = expression.binder { values.push(("binder", Value::EnumNumber(1))); }
        fields.push(("expression_source", message(encoder, "ExpressionSourceContract", &values)?));
    }
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
        if SOURCE_FIELDS.iter().any(|(known, _)| known == key) { continue; }
        match key.as_str() {
            keys::BOUNDARY_TAG => {
                if value != &field.tag.to_string() { return Err(invalid(key, "source boundary tag differs from the selected descriptor number")); }
            }
            keys::BOUNDARY_SHAPE | keys::EXTENSION | keys::PARAMETER | keys::ROLE | keys::DOC | keys::STRUCTURE
            | keys::QUANTITY | keys::TRANSFER_CONTEXT | keys::FK_RELATION | keys::FK_COLUMN
            | keys::GRAPH_ENDPOINT | keys::KEY_REFERENCE | keys::KEY_COLLECTION
            | keys::KEY_INTEGER_RANGE | keys::KEY_TAGGED_ALTERNATIVE | keys::IDENTITY | keys::IDENTITY_OWNER
            | keys::CANONICAL_KEY | keys::CANONICAL_READONLY | keys::CANONICAL_RECORD_KEY
            | keys::EXPRESSION_ROLE | keys::EXPRESSION_ORDINAL | keys::EXPRESSION_BINDER => {}
            _ => return Err(invalid(key, "selected facet has no typed protobuf interpretation; add its owner policy before publication")),
        }
    }
    let value = encoder.message("pse.schema.v1.RegistryFieldFacets", &fields)?;
    for name in required_fields(field) {
        if !has_path(&value, name) { return Err(invalid(name, "source facet failed to publish its required typed field")); }
    }
    Ok(Some(value))
}

fn check_shape(field: &Field, graph: &ProjectionGraph, text: &str) -> Result<(), ProjectionCodegenError> {
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

fn has_path(message: &DynamicMessage, path: &str) -> bool {
    let (name, rest) = path.split_once('.').map_or((path, None), |(name, rest)| (name, Some(rest)));
    match (value(message, name), rest) { (Some(_), None) => true, (Some(Value::Message(value)), Some(rest)) => has_path(value, rest), _ => false }
}

const CANONICAL_CLAUSES: &[(&str, &str)] = &[
    (keys::CANONICAL_ASSERT, "assertion"), (keys::CANONICAL_DEFAULT, "default_expression"),
    (keys::CANONICAL_TRANSITION, "initial_member"), (keys::CANONICAL_TRANSITION_EXCEPTION, "transition_exception"),
    (keys::CANONICAL_SEALED, "sealed"), (keys::CANONICAL_SEAL_REFUSAL, "seal_refusal"), (keys::CANONICAL_REFERENCE_KIND, "reference_kind"),
];
fn canonical_message(policy: &pse_schema::model::CanonicalFieldPolicy, encoder: &OptionEncoder) -> Result<DynamicMessage, ProjectionCodegenError> {
    let mut values = Vec::new();
    if let Some(contract) = policy.contract {
        values.push(("record_key", Value::Bool(contract.record_key)));
        if let Some(kind) = contract.key { values.push(("key_kind", Value::EnumNumber(canonical_number(&encoder.pool(), kind)?))); }
        if let Some(readonly) = contract.readonly { values.push(("readonly", Value::Bool(readonly))); }
    }
    for (name, expression) in [("assertion", &policy.assertion), ("default_expression", &policy.default), ("transition_exception", &policy.transition_exception), ("sealed", &policy.sealed)] {
        if let Some(expression) = expression { values.push((name, Value::String(expression.canonical()))); }
    }
    if let Some(initial) = &policy.initial { values.push(("initial_member", Value::String(initial.clone()))); }
    if let Some(kind) = &policy.reference_kind { values.push(("reference_kind", Value::String(kind.clone()))); }
    if let Some(refusal) = policy.seal_refusal { values.push(("seal_refusal", Value::String(refusal.wire().into()))); }
    Ok(encoder.message("pse.schema.v1.CanonicalFieldContract", &values)?)
}
fn canonical_policy(canonical: &DynamicMessage) -> Result<pse_schema::model::CanonicalFieldPolicy, ProjectionCodegenError> {
    closed(canonical, "CanonicalFieldContract", &["key_kind", "readonly", "record_key", "assertion", "default_expression", "initial_member", "transition_exception", "sealed", "seal_refusal", "reference_kind"])?;
    let mut metadata = std::collections::HashMap::new();
    if let Some(value) = value(canonical, "key_kind") {
        let Value::EnumNumber(number) = value else { return Err(error("canonical key is not an enum")); };
        let key = CanonicalKeyKind::ALL.iter().copied().find(|kind| canonical_number(canonical.pool(), *kind).is_ok_and(|value| value == *number)).ok_or_else(|| error("unknown native canonical key kind"))?;
        metadata.insert(keys::CANONICAL_KEY.into(), key.as_str().into());
    }
    if let Some(readonly) = value(canonical, "readonly") {
        let Value::Bool(readonly) = readonly else { return Err(error("canonical readonly is not bool")); };
        metadata.insert(keys::CANONICAL_READONLY.into(), readonly.to_string());
    }
    match value(canonical, "record_key") { Some(Value::Bool(true)) => { metadata.insert(keys::CANONICAL_RECORD_KEY.into(), "true".into()); }, None | Some(Value::Bool(false)) => {}, _ => return Err(error("canonical record-key declaration is not bool")) }
    for (key, name) in CANONICAL_CLAUSES { if value(canonical, name).is_some() { metadata.insert((*key).into(), text(canonical, name)?); } }
    pse_schema::model::CanonicalFieldPolicy::parse_metadata(&metadata, "typed registry canonical policy")?.ok_or_else(|| error("empty canonical source policy"))
}

fn value<'a>(message: &'a DynamicMessage, name: &str) -> Option<&'a Value> {
    let descriptor = message.pool().message(message.message_index());
    descriptor.field_by_name(name).and_then(|field| message.field_by_number(field.number()))
}

fn required<'a>(message: &'a DynamicMessage, name: &str) -> Result<&'a Value, ProjectionCodegenError> {
    value(message, name).ok_or_else(|| error(format!("typed registry option requires {name}")))
}

fn text(message: &DynamicMessage, name: &str) -> Result<String, ProjectionCodegenError> {
    match required(message, name)? { Value::String(text) => Ok(text.clone()), _ => Err(error(format!("{name}: expected string"))) }
}

fn integer(message: &DynamicMessage, name: &str) -> Result<i64, ProjectionCodegenError> {
    match required(message, name)? { Value::I64(value) => Ok(*value), _ => Err(error(format!("{name}: expected int64"))) }
}

fn enumeration(message: &DynamicMessage, name: &str, maximum: i32) -> Result<i32, ProjectionCodegenError> {
    match required(message, name)? { Value::EnumNumber(value) if (1..=maximum).contains(value) => Ok(*value), _ => Err(error(format!("{name}: unknown native vocabulary value"))) }
}

fn nested<'a>(message: &'a DynamicMessage, name: &str) -> Result<&'a DynamicMessage, ProjectionCodegenError> {
    match required(message, name)? { Value::Message(value) => Ok(value), _ => Err(error(format!("{name}: expected typed message"))) }
}

fn list<'a>(message: &'a DynamicMessage, name: &str) -> Result<&'a [Value], ProjectionCodegenError> {
    match value(message, name) { Some(Value::List(values)) => Ok(values), None => Ok(&[]), _ => Err(error(format!("{name}: expected repeated field"))) }
}

fn closed(message: &DynamicMessage, name: &str, fields: &[&str]) -> Result<(), ProjectionCodegenError> {
    let descriptor = message.pool().message(message.message_index());
    if descriptor.full_name() != format!("pse.schema.v1.{name}") || !message.unknown_fields().is_empty()
        || descriptor.fields().iter().any(|field| !fields.contains(&field.name()))
    { return Err(error(format!("{name}: unknown selected typed option shape"))); }
    Ok(())
}

/// Interpret selected extension payloads through the same native domain contracts.
/// Reading an option does not admit Arrow storage, resolve quantities or run a catalog.
pub(super) fn check_option(option: &DynamicMessage) -> Result<(), ProjectionCodegenError> {
    if let Some(Value::U32(version)) = value(option, "metadata_version") && *version != 2 {
        return Err(pse_contract_proto::MetadataError::Unavailable { reason: format!("unavailable registry interpretation version {version}") }.into());
    }
    vocabularies(option.pool())?;
    closed(option, "RegistryFieldFacets", &["metadata_version", "extension", "role", "documentation", "structure_name", "quantity", "transfer_context_version", "reference", "graph_endpoint", "collection", "integer_range", "tagged_alternative", "identity", "canonical", "expression_source", "native_field_ipc"])?;
    if required(option, "metadata_version")? != &Value::U32(2) { return Err(error("unknown registry metadata version")); }
    if value(option, "role").is_some() { enumeration(option, "role", 6)?; }
    if value(option, "identity").is_some() {
        let identity = nested(option, "identity")?;
        closed(identity, "IdentityContract", &["entity_name", "owns_identity"])?;
        let name = text(identity, "entity_name")?;
        let owner = match required(identity, "owns_identity")? {
            Value::Bool(true) => Some("true"), Value::Bool(false) => None,
            _ => return Err(error("identity ownership is not bool")),
        };
        IdentityContract::parse_metadata(Some(&name), owner, "typed registry identity")?;
    }
    if value(option, "canonical").is_some() { canonical_policy(nested(option, "canonical")?)?; }
    if value(option, "expression_source").is_some() {
        let expression = nested(option, "expression_source")?;
        closed(expression, "ExpressionSourceContract", &["role_path_template", "ordinal", "binder"])?;
        let role = value(expression, "role_path_template").map(|_| text(expression, "role_path_template")).transpose()?;
        let ordinal = value(expression, "ordinal").map(|_| enumeration(expression, "ordinal", 3)).transpose()?.map(|value| match value { 1 => ExpressionOrdinal::List, 2 => ExpressionOrdinal::Zero, _ => ExpressionOrdinal::Flatten });
        let binder = value(expression, "binder").map(|_| enumeration(expression, "binder", 1)).transpose()?.map(|_| ExpressionBinder::DomainBeforeName);
        if ExpressionSourceContract::parse_metadata(role.as_deref(), ordinal.map(ExpressionOrdinal::as_str), binder.map(ExpressionBinder::as_str), "typed registry expression source")?.is_none() { return Err(error("empty expression source policy")); }
    }
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
        }).collect::<Result<Vec<_>, ProjectionCodegenError>>()?;
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
    if let Some(carrier) = value(option, "native_field_ipc") {
        let Value::Bytes(carrier) = carrier else { return Err(error("native field carrier is not bytes")); };
        let field = native_ipc::decode(carrier)?;
        FieldContract::from_field(field.clone()).canonical_policy()?;
        let encoder = OptionEncoder::from_pool(std::sync::Arc::clone(option.pool()))?;
        let expected = lower_native_field(&field, &encoder)?;
        for descriptor in option.message_descriptor().fields() {
            if descriptor.name() == "native_field_ipc" { continue; }
            if option.field_by_number(descriptor.number()) != expected.field_by_number(descriptor.number()) {
                return Err(error(format!("native IPC field disagrees with typed registry fact {}", descriptor.name())));
            }
        }
    }
    Ok(())
}

fn lower_native_field(native: &arrow_schema::Field, encoder: &OptionEncoder) -> Result<DynamicMessage, ProjectionCodegenError> {
    let mut metadata = native.metadata().iter().map(|(key, value)| (key.clone(), value.clone())).collect::<std::collections::BTreeMap<_, _>>();
    // Presentation was checked against the actual source graph before publication;
    // it has no additional registry-execution facts. Never infer a source graph
    // from an acquired schema or let this retained presentation grant admission.
    metadata.remove(keys::BOUNDARY_SHAPE);
    let tag = metadata.get(keys::BOUNDARY_TAG).map(|tag| tag.parse::<u32>()).transpose().map_err(|_| error("native field has an invalid boundary number"))?.unwrap_or(1);
    let field = Field { name: native.name().clone(), tag, ty: TypeRef::Scalar(pse_contract::boundary_projection::Scalar::Bool), direction: pse_contract::boundary_projection::Direction::Both, rules: vec![], named_facets: metadata };
    lower_field_options(&field, &ProjectionGraph::default(), encoder)?.ok_or_else(|| error("native IPC field has no declared registry facets"))
}

/// Normalization for native crossing + registry execution + canonical publication.
/// Original options/IPC remain in the acquired FDS. Other identity purposes retain
/// their own source-owned rules (for example structure names in storage identity).
pub(super) fn normalize_option(option: &DynamicMessage) -> Result<Option<Vec<u8>>, ProjectionCodegenError> {
    use buffa_descriptor::ReflectMessageMut;
    check_option(option)?;
    let mut normalized = option.clone();
    let encoder = OptionEncoder::from_pool(std::sync::Arc::clone(option.pool()))?;
    if let Some(Value::Message(canonical)) = value(option, "canonical") {
        let policy = canonical_policy(canonical)?;
        let normalized_value = canonical_message(&policy, &encoder)?;
        let pool = std::sync::Arc::clone(normalized.pool());
        let field = pool.message(normalized.message_index()).field_by_name("canonical").ok_or_else(|| error("known canonical option is absent"))?;
        normalized.try_set(field, Value::Message(normalized_value)).map_err(pse_contract_proto::MetadataError::from)?;
    }

    let pool = std::sync::Arc::clone(normalized.pool());
    let descriptor = pool.message(normalized.message_index());
    for name in ["documentation", "structure_name"] {
        let field = descriptor.field_by_name(name).ok_or_else(|| error("known registry provenance field is absent"))?;
        normalized.try_clear(field).map_err(pse_contract_proto::MetadataError::from)?;
    }
    if let Some(Value::Bytes(carrier)) = value(option, "native_field_ipc") {
        let field = native_ipc::decode(carrier)?;
        let field = pse_columnar::native_field::project(&field, pse_columnar::native_field::MetadataPurpose::ExecutionIdentity)?;
        let canonical = pse_schema::model::field::render_field(&field)?;
        let field = descriptor.field_by_name("native_field_ipc").ok_or_else(|| error("known native IPC option field is absent"))?;
        normalized.try_set(field, Value::Bytes(canonical.into_bytes())).map_err(pse_contract_proto::MetadataError::from)?;
    }
    let has_meaning = normalized.message_descriptor().fields().iter().any(|field| field.name() != "metadata_version" && normalized.field_by_number(field.number()).is_some());
    if !has_meaning { return Ok(None); }
    Ok(Some(normalized.try_encode_to_vec().map_err(pse_contract_proto::MetadataError::from)?))
}
