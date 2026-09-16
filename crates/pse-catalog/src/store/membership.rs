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
use pse_relations::columnar::FieldCheckedBatch;
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
    /// Actual auxiliary bindings retained by the producing invocation.
    pub invocation: Option<super::invocation::OwnedInvocation>,
    /// Shared transient owners for one cold admission, never serialized or retained by snapshots.
    pub traversal: Arc<super::traversal::AdmissionTraversal>,
}

/// Resolve the exact relation inventory and validate its parent bindings.
pub(crate) fn inventory<'a>(
    reg: &'a Registry,
    manifest: &Manifest,
    context: &AdmissionContext,
    admission: &Arc<crate::store::open::CatalogContext>,
) -> Result<BTreeMap<String, &'a RelationSpec>, CatalogError> {
    validate_parents(manifest, context, admission)?;
    match manifest.snapshot_kind {
        SnapshotKind::Model | SnapshotKind::Case => {
            if context.stage_pass.is_some() || context.invocation.is_some() {
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
    admission: &Arc<crate::store::open::CatalogContext>,
) -> Result<(), CatalogError> {
    if let Some(invocation) = &context.invocation {
        if invocation
            .document_source
            .as_ref()
            .is_some_and(|source| !Arc::ptr_eq(admission, &source.admission))
        {
            return Err(refused(
                "a foreign document source must be reopened by this catalog",
            ));
        }
        for policy in invocation.policies.values() {
            if !Arc::ptr_eq(admission, &policy.snapshot.admission) {
                return Err(refused(
                    "a foreign policy owner must be reopened by this catalog",
                ));
            }
        }
    }
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

/// Bind checked candidate fields to the exact admitted parent scope for native obligations.
pub(crate) fn complete_context(
    reg: &Registry,
    candidates: &BTreeMap<RelationKey, FieldCheckedBatch>,
    context: &AdmissionContext,
) -> Result<BTreeMap<RelationKey, FieldCheckedBatch>, CatalogError> {
    let mut complete = candidates.clone();
    let mut selected = Vec::new();
    bound_parent_members(reg, &context.parents, context.stage_pass, &mut selected)?;
    for (parent, selected_port) in selected {
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
                if existing.batch() != relation.batch() {
                    return Err(refused(&format!(
                        "parent bindings offer ambiguous actual rows for {}",
                        spec.key
                    )));
                }
            } else {
                complete.insert(spec.key, relation.checked().clone());
            }
        }
    }
    // A stage's current declared inputs are authoritative. An ancestor's derived
    // output is not an additional implicit input and must not replace or conflict
    // with the selected current relation. Only primitive model/case context is
    // inherited for domain/reference obligations; derived reads require a port.
    let current = complete.keys().copied().collect::<BTreeSet<_>>();
    let mut stack = context.parents.values().collect::<Vec<_>>();
    let mut visited = BTreeSet::new();
    while let Some(parent) = stack.pop() {
        if !visited.insert(Arc::as_ptr(parent)) {
            continue;
        }
        if matches!(
            parent.manifest.snapshot_kind,
            SnapshotKind::Model | SnapshotKind::Case
        ) {
            for relation in parent.relations.values() {
                let spec = reg
                    .relation_by_id(relation.member.relation_id)
                    .ok_or_else(|| refused("primitive parent declaration is missing"))?;
                if current.contains(&spec.key) {
                    continue;
                }
                relation.contract.validate_against_registry(reg, spec)?;
                if let Some(existing) = complete.get(&spec.key) {
                    if existing.batch() != relation.batch() {
                        return Err(refused(&format!(
                            "primitive parent scope offers ambiguous actual rows for {}",
                            spec.key
                        )));
                    }
                } else {
                    complete.insert(spec.key, relation.checked().clone());
                }
            }
        }
        stack.extend(parent.parents.values());
    }
    Ok(complete)
}

// A model/case parent supplies its complete primitive scope. A stage parent supplies
// the exact explicitly bound producer output. Only primitive ancestor scope is inherited.
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
/// predicates over those rows before a snapshot handle is constructed. Every callback
/// receives the actual operation session and must retain its runtime, allocator,
/// functions and scoped policies when constructing its validation workspace.
pub trait SemanticValidator: Send + Sync + std::fmt::Debug {
    /// Establish obligations affected by newly constructed relations while retaining
    /// complete admitted parent inputs for every positive and negative read.
    /// The default executes the complete program; rule implementations may select
    /// obligations from their actual declared dependency closure.
    fn validate_affected<'a>(
        &'a self,
        reg: &'a Registry,
        rows: &'a BTreeMap<RelationKey, FieldCheckedBatch>,
        _changed: &'a BTreeSet<RelationKey>,
        session: &'a crate::session::SnapshotSession,
        cancel: &'a pse_ids::CancellationToken,
    ) -> crate::BoxFut<'a, Result<(), CatalogError>> {
        self.validate_checked(reg, rows, session, cancel)
    }
    /// Actual typed local source construction consumer, bound when the catalog is opened.
    fn source_producer(&self) -> Option<Arc<dyn crate::source_production::SourceProducer>> {
        None
    }
    /// Evaluate residual relation obligations over actual locally checked fields.
    /// Implementations should retain these owners in their native input providers.
    fn validate_checked<'a>(
        &'a self,
        reg: &'a Registry,
        rows: &'a BTreeMap<RelationKey, FieldCheckedBatch>,
        session: &'a crate::session::SnapshotSession,
        cancel: &'a pse_ids::CancellationToken,
    ) -> crate::BoxFut<'a, Result<(), CatalogError>> {
        Box::pin(async move {
            let raw = rows
                .iter()
                .map(|(key, batch)| (*key, batch.batch().clone()))
                .collect();
            self.validate(reg, &raw, session, cancel).await
        })
    }
    /// Actual producing implementation shared by local execution and reopening.
    fn stage_producer(
        &self,
        _pass: SemanticId,
    ) -> Option<Arc<dyn crate::computation::StageProducer>> {
        None
    }
    /// Validate all applicable registered invariants against actual candidate and parent
    /// rows. A matching fingerprint or cached certificate cannot replace evaluation.
    fn validate<'a>(
        &'a self,
        reg: &'a Registry,
        rows: &'a BTreeMap<RelationKey, RecordBatch>,
        session: &'a crate::session::SnapshotSession,
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
        session: &'a crate::session::SnapshotSession,
        cancel: &'a pse_ids::CancellationToken,
    ) -> crate::BoxFut<'a, Result<(), CatalogError>> {
        self.validate(reg, rows, session, cancel)
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
    candidates: &BTreeMap<RelationKey, FieldCheckedBatch>,
    context: &AdmissionContext,
) -> Result<usize, CatalogError> {
    let mut total = candidates.values().try_fold(0usize, |total, _batch| {
        total.checked_add(1024).ok_or_else(super::encode::overflow)
    })?;
    let mut stack = context.parents.values().collect::<Vec<_>>();
    let mut visited = BTreeSet::new();
    while let Some(parent) = stack.pop() {
        if !visited.insert(Arc::as_ptr(parent)) {
            continue;
        }
        for _relation in parent.relations.values() {
            total = total
                .checked_add(1024)
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
