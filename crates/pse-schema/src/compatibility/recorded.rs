// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Interpretation of portable v2 declarations, independent of the live registry.
use super::{CompatibilityError, malformed};
use crate::{
    fingerprint::SemanticContract,
    model::{FieldContract, ReferenceContract},
};
use arrow_schema::{Field, Schema};
use pse_ids::SemanticId;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Relation {
    relation: String,
    id: SemanticId,
    authority: crate::model::Authority,
    snapshot: crate::model::SnapshotClass,
    stability: crate::model::Stability,
    granularity: Option<crate::model::DerivationGranularity>,
    primary_key: Vec<String>,
    fields: Vec<Field>,
    enums: BTreeMap<String, Domain>,
    extensions: BTreeMap<String, Extension>,
    checks: BTreeMap<String, String>,
    invariants: BTreeMap<String, Invariant>,
    #[serde(default)]
    unique_keys: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    foreign_keys: BTreeMap<String, ForeignKey>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Domain {
    id: SemanticId,
    members: BTreeMap<String, bool>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Extension {
    metadata_kind: String,
    version: u32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Invariant {
    kind: crate::model::InvariantKind,
    query: String,
    severity: crate::model::Severity,
    inputs: BTreeSet<String>,
    keys: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ForeignKey {
    columns: Vec<String>,
    target: SemanticId,
    target_columns: Vec<String>,
}
fn descriptions(
    contract: &SemanticContract,
) -> Result<BTreeMap<SemanticId, Relation>, CompatibilityError> {
    contract
        .relations
        .iter()
        .map(|(id, value)| {
            Ok((
                *id,
                serde_json::from_value(value.clone()).map_err(malformed)?,
            ))
        })
        .collect()
}
fn named<'a>(
    relations: &'a BTreeMap<SemanticId, Relation>,
    owner: &'a Relation,
    name: &str,
) -> Result<&'a Relation, CompatibilityError> {
    if owner
        .relation
        .rsplit_once('@')
        .map(|(qualified, _)| qualified)
        == Some(name)
    {
        return Ok(owner);
    }
    let mut matching = relations
        .values()
        .filter(|r| r.relation.rsplit_once('@').map(|(n, _)| n) == Some(name));
    let relation = matching
        .next()
        .ok_or_else(|| malformed(format!("missing support relation {name}")))?;
    if matching.next().is_some() {
        return Err(malformed(format!("ambiguous support relation {name}")));
    }
    Ok(relation)
}
fn local_columns(relation: &Relation, columns: &[String]) -> Result<(), CompatibilityError> {
    let distinct: BTreeSet<_> = columns.iter().collect();
    if distinct.len() != columns.len()
        || columns
            .iter()
            .any(|name| !relation.fields.iter().any(|f| f.name() == name))
    {
        return Err(malformed("key names unknown or duplicate fields"));
    }
    Ok(())
}
fn add_name(
    relations: &BTreeMap<SemanticId, Relation>,
    owner: &Relation,
    name: &str,
    dependencies: &mut BTreeSet<SemanticId>,
) -> Result<(), CompatibilityError> {
    dependencies.insert(named(relations, owner, name)?.id);
    Ok(())
}
fn field_support(
    field: &Field,
    owner: &Relation,
    relations: &BTreeMap<SemanticId, Relation>,
    dependencies: &mut BTreeSet<SemanticId>,
    observed: bool,
) -> Result<(), CompatibilityError> {
    let metadata = field.metadata();
    if let Some(text) = metadata.get(crate::arrow::KEY_ENUM) {
        let id = SemanticId::parse_hex(text).map_err(malformed)?;
        let (name, _) = owner
            .enums
            .iter()
            .find(|(_, domain)| domain.id == id)
            .ok_or_else(|| malformed("field enum identity has no recorded domain"))?;
        if FieldContract::enum_name_of(field).is_some_and(|declared| declared != name)
            || metadata
                .get(crate::arrow::KEY_EXTENSION_NAME)
                .map(String::as_str)
                != Some("pse.enum")
        {
            return Err(malformed(
                "field enum identity disagrees with recorded domain",
            ));
        }
    } else if metadata
        .get(crate::arrow::KEY_EXTENSION_NAME)
        .map(String::as_str)
        == Some("pse.enum")
    {
        return Err(malformed("enum field has no domain identity"));
    }
    if let Some(name) = metadata.get(crate::arrow::KEY_EXTENSION_NAME) {
        let extension = owner.extensions.get(name);
        // Extension-internal fields (e.g. BoundKind) use the platform extension,
        // whose layout is established by the observed parent descriptor.
        let spec = crate::model::EXTENSION_TYPES
            .iter()
            .find(|e| e.name == name)
            .ok_or_else(|| {
                CompatibilityError::UnsupportedEncoding(format!("unknown extension {name}"))
            })?;
        if extension.is_none() && !observed {
            return Err(malformed(format!("missing extension {name}")));
        }
        if extension.is_some_and(|e| {
            e.version != spec.metadata_version || e.metadata_kind != spec.metadata.to_string()
        }) {
            return Err(CompatibilityError::UnsupportedEncoding(format!(
                "extension contract {name}"
            )));
        }
        let text = metadata
            .get(crate::arrow::KEY_EXTENSION_METADATA)
            .ok_or_else(|| malformed("missing extension descriptor"))?;
        let value: serde_json::Value = serde_json::from_str(text).map_err(malformed)?;
        let object = value
            .as_object()
            .ok_or_else(|| malformed("extension descriptor is not an object"))?;
        let expected_keys = 1 + usize::from(spec.metadata.id_key().is_some());
        if object.len() != expected_keys
            || value["v"].as_u64() != Some(u64::from(spec.metadata_version))
        {
            return Err(malformed(
                "extension descriptor generation or fields disagree",
            ));
        }
        if let Some(key) = spec.metadata.id_key() {
            let text = value[key]
                .as_str()
                .ok_or_else(|| malformed("missing extension identity"))?;
            let id = SemanticId::parse_hex(text).map_err(malformed)?;
            if key == "enum_id" {
                if metadata.get(crate::arrow::KEY_ENUM) != Some(&id.to_hex()) {
                    return Err(malformed("extension enum identity differs"));
                }
            } else {
                if !relations.contains_key(&id) {
                    return Err(malformed("missing ordinal reference support"));
                }
                dependencies.insert(id);
            }
        }
        match name.as_str() {
            "pse.source_span" => add_name(relations, owner, "authored.documents", dependencies)?,
            "pse.quantity_value" => {
                add_name(relations, owner, "reference.quantity_types", dependencies)?;
                add_name(relations, owner, "reference.units", dependencies)?;
            }
            _ => {}
        }
    }
    if let Some(text) = metadata.get(crate::arrow::KEY_FK) {
        let (name, column) = text
            .rsplit_once('.')
            .ok_or_else(|| malformed("invalid scalar reference"))?;
        let target = named(relations, owner, name)?;
        local_columns(target, &[column.to_owned()])?;
        dependencies.insert(target.id);
    }
    if let Some(text) = metadata.get(crate::model::reference::KEY_REFERENCE) {
        let reference: ReferenceContract = serde_json::from_str(text).map_err(malformed)?;
        if reference.columns.is_empty() || reference.canonical().map_err(malformed)? != *text {
            return Err(malformed("invalid correlated reference"));
        }
        let target = named(relations, owner, &reference.relation)?;
        local_columns(
            target,
            &reference
                .columns
                .iter()
                .map(|c| c.target.clone())
                .collect::<Vec<_>>(),
        )?;
        dependencies.insert(target.id);
    }
    let children = FieldContract::from_field(field.clone()).children();
    let mut names = BTreeSet::new();
    for child in children {
        if !names.insert(child.name().to_owned()) {
            return Err(malformed("duplicate nested field"));
        }
        field_support(child.field(), owner, relations, dependencies, observed)?;
    }
    Ok(())
}

fn validated_graph(
    contract: &SemanticContract,
) -> Result<BTreeMap<SemanticId, BTreeSet<SemanticId>>, CompatibilityError> {
    if !contract
        .roots
        .iter()
        .all(|id| contract.relations.contains_key(id))
    {
        return Err(malformed("absent or unknown roots"));
    }
    let relations = descriptions(contract)?;
    let mut names = BTreeSet::new();
    let mut domains = BTreeMap::new();
    let mut graph = BTreeMap::new();
    for (id, relation) in &relations {
        let (name, version) = relation
            .relation
            .rsplit_once('@')
            .ok_or_else(|| malformed("missing relation version"))?;
        let (namespace, table) = name
            .split_once('.')
            .ok_or_else(|| malformed("invalid relation name"))?;
        if table.is_empty()
            || !crate::model::Namespace::ALL
                .iter()
                .any(|v| v.as_str() == namespace)
            || version
                .parse::<u32>()
                .ok()
                .is_none_or(|v| v == 0 || v.to_string() != version)
            || !names.insert(&relation.relation)
            || *id != relation.id
            || relation.id
                != crate::builder::registry_id(&format!("relation:{}", relation.relation))
        {
            return Err(malformed("relation name, identity or version disagrees"));
        }
        // Parsing the closed platform declarations above establishes their interpretation.
        let _ = (
            &relation.authority,
            &relation.snapshot,
            &relation.stability,
            &relation.granularity,
        );
        let mut fields = BTreeSet::new();
        let mut dependencies = BTreeSet::new();
        for field in &relation.fields {
            if field.name().is_empty() || !fields.insert(field.name()) {
                return Err(malformed("duplicate or empty field"));
            }
            field_support(field, relation, &relations, &mut dependencies, false)?;
        }
        local_columns(relation, &relation.primary_key)?;
        for columns in relation.unique_keys.values() {
            local_columns(relation, columns)?;
        }
        for (name, domain) in &relation.enums {
            if name.is_empty()
                || domain.id != crate::builder::registry_id(&format!("enum:{name}"))
                || domain.members.is_empty()
                || domain.members.keys().any(String::is_empty)
            {
                return Err(malformed("invalid enum declaration"));
            }
            if domains
                .get(name)
                .is_some_and(|old| old != &(domain.id, &domain.members))
            {
                return Err(malformed("recorded support has contradictory enum domains"));
            }
            domains.insert(name, (domain.id, &domain.members));
        }
        for (name, extension) in &relation.extensions {
            let spec = crate::model::EXTENSION_TYPES
                .iter()
                .find(|e| e.name == name)
                .ok_or_else(|| {
                    CompatibilityError::UnsupportedEncoding(format!("unknown extension {name}"))
                })?;
            if extension.metadata_kind != spec.metadata.to_string()
                || extension.version != spec.metadata_version
            {
                return Err(CompatibilityError::UnsupportedEncoding(format!(
                    "extension {name}"
                )));
            }
        }
        for (name, sql) in &relation.checks {
            if name.is_empty()
                || crate::fingerprint::canonical_sql(sql, true).map_err(malformed)? != *sql
            {
                return Err(malformed("noncanonical declared check"));
            }
        }
        for (name, invariant) in &relation.invariants {
            let _ = (&invariant.kind, &invariant.severity);
            if name.is_empty()
                || crate::fingerprint::canonical_sql(&invariant.query, false).map_err(malformed)?
                    != invariant.query
            {
                return Err(malformed("noncanonical invariant"));
            }
            local_columns(relation, &invariant.keys)?;
            for input in &invariant.inputs {
                add_name(&relations, relation, input, &mut dependencies)?;
            }
        }
        for reference in relation.foreign_keys.values() {
            local_columns(relation, &reference.columns)?;
            let target = relations
                .get(&reference.target)
                .ok_or_else(|| malformed("missing foreign-key support"))?;
            local_columns(target, &reference.target_columns)?;
            if reference.columns.is_empty()
                || reference.columns.len() != reference.target_columns.len()
            {
                return Err(malformed("foreign-key arity differs"));
            }
            dependencies.insert(target.id);
        }
        graph.insert(*id, dependencies);
    }
    Ok(graph)
}
fn reachable(
    graph: &BTreeMap<SemanticId, BTreeSet<SemanticId>>,
    roots: &BTreeSet<SemanticId>,
) -> Result<BTreeSet<SemanticId>, CompatibilityError> {
    let mut selected = BTreeSet::new();
    let mut pending: Vec<_> = roots.iter().copied().collect();
    while let Some(id) = pending.pop() {
        if selected.insert(id) {
            pending.extend(
                graph
                    .get(&id)
                    .ok_or_else(|| malformed("unknown selected root"))?
                    .iter()
                    .copied(),
            );
        }
    }
    Ok(selected)
}
pub(super) fn validate(contract: &SemanticContract) -> Result<(), CompatibilityError> {
    let graph = validated_graph(contract)?;
    if reachable(&graph, &contract.roots)? != contract.relations.keys().copied().collect() {
        return Err(malformed(
            "recorded support closure contains unreachable declarations",
        ));
    }
    Ok(())
}
pub(super) fn select(
    contract: &SemanticContract,
    roots: &BTreeSet<SemanticId>,
) -> Result<SemanticContract, CompatibilityError> {
    let graph = validated_graph(contract)?;
    let selected = reachable(&graph, roots)?;
    Ok(SemanticContract {
        version: contract.version,
        roots: roots.clone(),
        relations: contract
            .relations
            .iter()
            .filter(|(id, _)| selected.contains(id))
            .map(|(id, value)| (*id, value.clone()))
            .collect(),
    })
}
pub(super) fn observed_fields(
    contract: &SemanticContract,
    id: SemanticId,
    schema: &Schema,
) -> Result<(), CompatibilityError> {
    crate::field_contract::declaration(schema).map_err(malformed)?;
    let relations = descriptions(contract)?;
    let relation = relations
        .get(&id)
        .ok_or_else(|| malformed("unknown observed relation"))?;
    let fields = schema
        .fields()
        .iter()
        .map(|f| crate::fingerprint::semantic_field(f).map_err(malformed))
        .collect::<Result<Vec<_>, _>>()?;
    if fields != relation.fields {
        return Err(malformed("observed fields contradict recorded meaning"));
    }
    let mut dependencies = BTreeSet::new();
    for field in schema.fields() {
        field_support(field, relation, &relations, &mut dependencies, true)?;
    }
    Ok(())
}
pub(super) fn project(
    source: &SemanticContract,
    target: &SemanticContract,
) -> Result<BTreeMap<SemanticId, Vec<Option<usize>>>, CompatibilityError> {
    let mut projection = BTreeMap::new();
    for (id, expected) in &target.relations {
        let actual = source
            .relations
            .get(id)
            .ok_or_else(|| incompatible("consumed support is absent"))?;
        let mut actual = actual.clone();
        let mut expected = expected.clone();
        let source_fields: Vec<Field> =
            serde_json::from_value(actual["fields"].clone()).map_err(malformed)?;
        let target_fields: Vec<Field> =
            serde_json::from_value(expected["fields"].clone()).map_err(malformed)?;
        let mut slots = Vec::new();
        for field in &target_fields {
            if let Some(index) = source_fields.iter().position(|f| f.name() == field.name()) {
                if source_fields[index] != *field {
                    return Err(incompatible("consumed field meaning differs"));
                }
                slots.push(Some(index));
            } else {
                return Err(incompatible("consumed field is absent"));
            }
        }
        actual["fields"] = serde_json::Value::Null;
        expected["fields"] = serde_json::Value::Null;
        let target_enums = expected["enums"]
            .as_object()
            .ok_or_else(|| malformed("enum map absent"))?;
        for (name, domain) in target_enums {
            let recorded = &actual["enums"][name];
            if recorded["id"] != domain["id"]
                || !recorded["members"]
                    .as_object()
                    .ok_or_else(|| incompatible("consumed enum is absent"))?
                    .keys()
                    .all(|member| domain["members"].get(member).is_some())
            {
                return Err(incompatible(
                    "consumer cannot interpret recorded enum members",
                ));
            }
        }
        for (name, extension) in expected["extensions"]
            .as_object()
            .ok_or_else(|| malformed("extension map absent"))?
        {
            if actual["extensions"].get(name) != Some(extension) {
                return Err(incompatible("consumed extension contract differs"));
            }
        }
        actual["extensions"] = serde_json::Value::Null;
        expected["extensions"] = serde_json::Value::Null;
        // Unconsumed domains and deprecation flags do not alter interpreted meaning.
        actual["enums"] = serde_json::Value::Null;
        expected["enums"] = serde_json::Value::Null;
        if actual != expected {
            return Err(incompatible(
                "consumed constraints, references or extension meaning differs",
            ));
        }
        projection.insert(*id, slots);
    }
    Ok(projection)
}
fn incompatible(reason: &str) -> CompatibilityError {
    CompatibilityError::Incompatible(reason.into())
}
