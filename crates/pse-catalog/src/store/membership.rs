// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Completing snapshot membership from the registry: every relation of the snapshot
//! kind's class in the admitted scope, including the empty ones (blueprint §5.3
//! step 7).
//!
//! Parent handles carry admitted rows; digest equality is only an identity check after
//! checking the role, declaration and actual referenced content.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use datafusion::arrow::array::RecordBatch;
use pse_ids::{SemanticId, SnapshotKind, model_port_name};
use pse_schema::Registry;
use pse_schema::model::{PortSource, RelationKey, RelationSpec, SnapshotClass};

use super::manifest::Manifest;
use crate::{CatalogError, Snapshot};

/// Explicit semantic bindings supplied to snapshot publication or reopening.
#[derive(Clone, Debug, Default)]
pub struct AdmissionContext {
    /// Each manifest parent role bound to a handle admitted by this catalog.
    pub parents: BTreeMap<String, Arc<Snapshot>>,
    /// The exact registered producing pass for a stage snapshot.
    pub stage_pass: Option<SemanticId>,
}

/// Resolve the exact relation inventory and validate its parent bindings.
pub(crate) fn inventory<'a>(
    reg: &'a Registry,
    manifest: &Manifest,
    context: &AdmissionContext,
    admission: &Arc<()>,
) -> Result<BTreeMap<String, &'a RelationSpec>, CatalogError> {
    validate_parents(manifest, context, admission)?;
    match manifest.snapshot_kind {
        SnapshotKind::Model | SnapshotKind::Case => {
            if context.stage_pass.is_some() {
                return Err(refused("a model or case has no producing stage pass"));
            }
            let class = if manifest.snapshot_kind == SnapshotKind::Model {
                SnapshotClass::Model
            } else {
                SnapshotClass::Case
            };
            Ok(reg
                .relations()
                .iter()
                .filter(|spec| {
                    spec.snapshot_class == class
                        && reg.relation(&spec.key.qualified_name()) == Some(*spec)
                })
                .map(|spec| (model_port_name(spec.key.namespace.as_str(), spec.id), spec))
                .collect())
        }
        SnapshotKind::Stage => stage_inventory(reg, manifest, context),
        SnapshotKind::Run => Err(CatalogError::UnknownVersion {
            field: "snapshot producer".to_owned(),
            value: "run: no executable run producer declaration is registered in Wave 1".to_owned(),
        }),
    }
}

fn validate_parents(
    manifest: &Manifest,
    context: &AdmissionContext,
    admission: &Arc<()>,
) -> Result<(), CatalogError> {
    let roles = manifest
        .semantic_parents
        .iter()
        .map(|parent| parent.role.as_str())
        .collect::<BTreeSet<_>>();
    if roles != context.parents.keys().map(String::as_str).collect() {
        return Err(refused(
            "explicit admitted parent role set differs from semantic_parents; missing context must be reopened first",
        ));
    }
    for parent in &manifest.semantic_parents {
        let handle = &context.parents[&parent.role];
        if !Arc::ptr_eq(admission, &handle.admission) {
            return Err(refused(
                "a foreign catalog parent must be reopened and admitted by this catalog",
            ));
        }
        if handle.snapshot_id() != parent.snapshot_id {
            return Err(refused(&format!(
                "parent role {} has the wrong snapshot identity",
                parent.role
            )));
        }
    }
    if manifest.snapshot_kind == SnapshotKind::Case
        && context
            .parents
            .get("model")
            .is_none_or(|parent| parent.manifest.snapshot_kind != SnapshotKind::Model)
    {
        return Err(refused(
            "a case requires an admitted model snapshot under role model",
        ));
    }
    Ok(())
}

fn stage_inventory<'a>(
    reg: &'a Registry,
    manifest: &Manifest,
    context: &AdmissionContext,
) -> Result<BTreeMap<String, &'a RelationSpec>, CatalogError> {
    let pass = context
        .stage_pass
        .and_then(|id| reg.passes().iter().find(|pass| pass.id == id))
        .ok_or_else(|| refused("a stage requires an explicit registered producing pass"))?;
    if !manifest
        .compiler
        .passes
        .iter()
        .any(|producer| producer.pass_id == pass.id && producer.version == pass.version)
    {
        return Err(refused(
            "manifest compiler.passes does not contain the exact declared producer version",
        ));
    }
    for role in context.parents.keys() {
        if !pass.inputs.iter().any(|port| port.port == role) {
            return Err(refused(&format!(
                "parent role {role} is not an input port of {}",
                pass.name
            )));
        }
    }
    for port in &pass.inputs {
        let Some(parent) = context.parents.get(port.port) else {
            if port.required {
                return Err(refused(&format!(
                    "required input port {} is absent",
                    port.port
                )));
            }
            continue;
        };
        let spec = reg
            .relation(&port.relation)
            .ok_or_else(|| refused("registered input relation is missing"))?;
        let relation = match port.source {
            PortSource::Derived { port, .. } => parent.relation_port(port),
            PortSource::Pinned => parent.relation(spec.key.namespace.as_str(), spec.key.name),
        }
        .ok_or_else(|| {
            refused(&format!(
                "input role {} lacks one unambiguous declared relation {}",
                port.port, port.relation
            ))
        })?;
        relation.contract.validate_against_registry(reg, spec)?;
        if let PortSource::Derived {
            pass: source_pass,
            port: source_port,
        } = port.source
        {
            let producer = reg
                .pass(source_pass)
                .ok_or_else(|| refused("declared source pass is missing"))?;
            if parent.stage_pass != Some(producer.id) || relation.member.port != source_port {
                return Err(refused(&format!(
                    "input {} is not from its declared producer output",
                    port.port
                )));
            }
        }
    }
    pass.outputs
        .iter()
        .map(|port| {
            reg.relation(&port.relation)
                .map(|spec| (port.port.to_owned(), spec))
                .ok_or_else(|| refused("registered output relation is missing"))
        })
        .collect()
}

/// Check direct batch/schema/value/key/FK content with the bound parent rows.
pub(crate) fn validate_content(
    reg: &Registry,
    candidates: &BTreeMap<RelationKey, RecordBatch>,
    context: &AdmissionContext,
) -> Result<BTreeMap<RelationKey, RecordBatch>, CatalogError> {
    let mut complete = candidates.clone();
    let mut stack = Vec::new();
    bound_parent_members(reg, &context.parents, context.stage_pass, &mut stack)?;
    let mut expanded = BTreeSet::new();
    let mut visited = BTreeSet::new();
    while let Some((parent, selected_port)) = stack.pop() {
        if !visited.insert((Arc::as_ptr(parent), selected_port)) {
            continue;
        }
        for (port, relation) in parent.relations.iter() {
            if selected_port.is_some_and(|selected| selected != port) {
                continue;
            }
            let spec = reg
                .relation_by_id(relation.member.relation_id)
                .ok_or_else(|| refused("parent declaration is missing"))?;
            relation.contract.validate_against_registry(reg, spec)?;
            if candidates.contains_key(&spec.key) {
                continue;
            }
            if let Some(existing) = complete.get(&spec.key) {
                if existing != &relation.batch {
                    return Err(refused(&format!(
                        "parent bindings offer ambiguous actual rows for {}",
                        spec.key
                    )));
                }
            } else {
                complete.insert(spec.key, relation.batch.clone());
            }
        }
        if expanded.insert(Arc::as_ptr(parent)) {
            bound_parent_members(reg, &parent.parents, parent.stage_pass, &mut stack)?;
        }
    }
    pse_relations::validate::validate_bundle(reg, &complete).map_err(|errors| {
        CatalogError::Admission {
            path: "snapshot bundle".to_owned(),
            reason: errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; "),
        }
    })?;
    Ok(complete)
}

// A model/case parent supplies its complete primitive scope. A stage parent supplies
// the exact explicitly bound producer output, with ancestors added independently.
// This prevents another same-schema output from silently replacing the chosen port.
fn bound_parent_members<'a>(
    reg: &'a Registry,
    parents: &'a BTreeMap<String, Arc<Snapshot>>,
    stage_pass: Option<SemanticId>,
    stack: &mut Vec<(&'a Arc<Snapshot>, Option<&'a str>)>,
) -> Result<(), CatalogError> {
    for (role, parent) in parents {
        let selected = if parent.manifest.snapshot_kind == SnapshotKind::Stage {
            let port = stage_pass
                .and_then(|id| reg.passes().iter().find(|pass| pass.id == id))
                .and_then(|pass| pass.inputs.iter().find(|input| input.port == role))
                .ok_or_else(|| refused("stage parent needs an explicit registered input role"))?;
            let relation = match port.source {
                PortSource::Derived { port, .. } => parent.relation_port(port),
                PortSource::Pinned => reg
                    .relation(&port.relation)
                    .and_then(|spec| parent.relation(spec.key.namespace.as_str(), spec.key.name)),
            }
            .ok_or_else(|| refused("stage parent has no unambiguous actual bound output"))?;
            Some(relation.member.port.as_str())
        } else {
            None
        };
        stack.push((parent, selected));
    }
    Ok(())
}

pub(crate) fn refused(reason: &str) -> CatalogError {
    CatalogError::Membership {
        reason: reason.to_owned(),
    }
}

/// Required domain/invariant admission supplied by the rule layer. The catalog owns
/// physical, schema, key and FK admission; this callback executes the actual registered
/// predicates over those rows before a snapshot handle is constructed.
pub trait SemanticValidator: Send + Sync + std::fmt::Debug {
    /// Validate all applicable registered invariants against actual candidate and parent
    /// rows. A matching fingerprint or cached certificate cannot replace evaluation.
    fn validate<'a>(
        &'a self,
        reg: &'a Registry,
        rows: &'a BTreeMap<RelationKey, RecordBatch>,
        cancel: &'a pse_ids::CancellationToken,
    ) -> crate::BoxFut<'a, Result<(), CatalogError>>;

    /// Validate invariants whose complete dependency set belongs to this standalone
    /// sidecar relation. Cross-artifact references are admitted by the typed receipt
    /// helpers once their actual supporting artifacts exist (blueprint §22.2).
    /// The default conservatively runs the full validator.
    fn validate_sidecar<'a>(
        &'a self,
        reg: &'a Registry,
        rows: &'a BTreeMap<RelationKey, RecordBatch>,
        cancel: &'a pse_ids::CancellationToken,
    ) -> crate::BoxFut<'a, Result<(), CatalogError>> {
        self.validate(reg, rows, cancel)
    }

    /// Establish correspondence between actual document bytes and their declared
    /// Model/Case row projections. Called only when the admitted candidate/parent
    /// inventory claims nonempty `authored.documents`; generic invariants do not
    /// establish this relationship. Candidate rows belong to the current snapshot,
    /// while exact admitted parent rows remain accessible through `context`.
    fn validate_snapshot_sources<'a>(
        &'a self,
        _catalog: &'a crate::Catalog,
        _kind: SnapshotKind,
        _context: &'a AdmissionContext,
        _candidates: &'a BTreeMap<RelationKey, RecordBatch>,
        _cancel: &'a pse_ids::CancellationToken,
    ) -> crate::BoxFut<'a, Result<(), CatalogError>> {
        Box::pin(async {
            Err(refused(
                "source-bearing snapshot admission requires executable document-to-row correspondence validation",
            ))
        })
    }

    /// Establish the actual registered producer semantics for every declared output port.
    /// Generic invariant validity alone cannot certify source-to-derived meaning.
    /// Implementations must inspect actual parents, sources and outputs; the default
    /// refuses to mint a stage whose producing computation has not been established.
    fn validate_stage<'a>(
        &'a self,
        _catalog: &'a crate::Catalog,
        _context: &'a AdmissionContext,
        _candidates: &'a BTreeMap<String, RecordBatch>,
        _cancel: &'a pse_ids::CancellationToken,
    ) -> crate::BoxFut<'a, Result<(), CatalogError>> {
        Box::pin(async {
            Err(refused(
                "stage admission requires an executable validator for the actual registered producer",
            ))
        })
    }
}

/// Conservative temporary extents for recursive Cell rows, tagged key/FK values and
/// ordered validation indexes. Input buffers retain their own owners separately.
///
/// # Errors
/// Checked addressable-size arithmetic or an unsupported dictionary expansion extent.
pub fn validation_extent(batch: &RecordBatch) -> Result<usize, CatalogError> {
    pse_ids::validation_extent(batch).map_err(Into::into)
}

pub(crate) fn context_validation_extent(
    candidates: &BTreeMap<RelationKey, RecordBatch>,
    context: &AdmissionContext,
) -> Result<usize, CatalogError> {
    let mut total = candidates.values().try_fold(0usize, |total, batch| {
        total
            .checked_add(validation_extent(batch)?)
            .ok_or_else(super::encode::overflow)
    })?;
    let mut stack = context.parents.values().collect::<Vec<_>>();
    let mut visited = BTreeSet::new();
    while let Some(parent) = stack.pop() {
        if !visited.insert(Arc::as_ptr(parent)) {
            continue;
        }
        for relation in parent.relations.values() {
            total = total
                .checked_add(validation_extent(&relation.batch)?)
                .ok_or_else(super::encode::overflow)?;
        }
        stack.extend(parent.parents.values());
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::arrow::array::{
        ArrayData, DictionaryArray, Int32Array, StringArray, make_array,
    };
    use datafusion::arrow::buffer::Buffer;
    use datafusion::arrow::datatypes::{DataType, Field, Int32Type, Schema};

    #[test]
    fn validation_copy_extent_is_independent_of_shared_input_backing_capacity() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "key",
            DataType::UInt64,
            false,
        )]));
        let batch = |buffer| {
            RecordBatch::try_new(
                Arc::clone(&schema),
                vec![make_array(
                    ArrayData::builder(DataType::UInt64)
                        .len(1)
                        .add_buffer(buffer)
                        .build()
                        .expect("valid data"),
                )],
            )
            .expect("valid batch")
        };
        let compact = batch(Buffer::from_slice_ref([0u64]));
        let backing = Buffer::from_slice_ref(vec![0u64; 65_536]);
        let shared = batch(backing.slice_with_length(128, 8));
        assert!(
            pse_ids::owned_buffer::retained_buffer_bytes(&shared).expect("size")
                > pse_ids::owned_buffer::retained_buffer_bytes(&compact).expect("size")
        );
        assert_eq!(
            validation_extent(&compact).expect("temporary"),
            validation_extent(&shared).expect("temporary")
        );
    }

    #[test]
    fn repeated_dictionary_key_text_has_a_reserved_expansion_extent() {
        let text = "x".repeat(1024);
        let dictionary = DictionaryArray::<Int32Type>::try_new(
            Int32Array::from(vec![0; 1024]),
            Arc::new(StringArray::from(vec![text])),
        )
        .expect("dictionary");
        let schema = Arc::new(Schema::new(vec![Field::new(
            "key",
            DataType::Dictionary(Box::new(DataType::Int32), Box::new(DataType::Utf8)),
            false,
        )]));
        let batch = RecordBatch::try_new(schema, vec![Arc::new(dictionary)]).expect("batch");
        assert!(validation_extent(&batch).expect("extent") >= 1024 * 1024 * 32);
    }
}
