// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Immutable source objects and protected lexical selection before shared admission.
use super::{
    Declaration, DeclarationId, ModelingPackage, PhysicalScope, Runtime, WorkflowError, contract,
};
use pse_authoring::{SourceSpan, dsl};
use pse_ids::{SemanticId, roles::SourceRevisionHash};
use pse_model::generated::runtime::canonical_revisions::Row as Revision;
use pse_model::{
    HeapUsage,
    generated::{
        enums::ModelingDeclarationKind as Kind, runtime::canonical_memberships::Row as Membership,
    },
};
use pse_modeling::selected_source::{self, LexicalLookup, Lookup};
use pse_operations::{
    canonical::ObjectEdit,
    canonical_selection::{
        SELECTED_INVENTORY_BATCH, SELECTED_RECHECK_WINDOW, SelectedRead, SelectedRecheckWindow,
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
mod delta;
mod local;
pub(crate) use local::{AdmittedModelRevision, LocalSourcePremises};
use local::{LocalInventory, LocalSourceObject, SourceMember};
pub(crate) mod reload;

// Only the newest bounded window survives the qualification turn. Its scratch
// and candidate handles remain admitted, and it retains no protection follower.
struct SelectedCandidateWindow {
    observations: SelectedRecheckWindow,
    _scratch: Arc<pse_columnar::AllocationLease>,
    _handles: Arc<pse_columnar::AllocationLease>,
}

fn checkpoint(cancel: &crate::CancelSource) -> Result<(), WorkflowError> {
    if cancel.token().is_cancelled() {
        Err(crate::math::MathRuntimeError::Cancelled.into())
    } else if cancel
        .deadline()
        .is_some_and(|deadline| std::time::Instant::now() >= deadline)
    {
        Err(crate::math::MathRuntimeError::Limit("canonical source operation deadline").into())
    } else {
        Ok(())
    }
}

pub(super) const ROOT_SCOPE: &str = "modeling:root";
// v2 source metadata classifies package engineering rules for bounded selection.
// Older revisions must be restaged, rather than silently losing their rules.
const CONTEXT: &str = "modeling:context:v2";
const CONTEXT_LOGICAL: &str = "modeling:context";
const PHYSICAL_SCOPE_LOGICAL: &str = "modeling:physical-scope";
const ENGINEERING_RULE: &str = "modeling:engineering_rule";
const INGRESS_FRAME: &str = "source:ingress-frame:v1";

fn parsed_fields(
    runtime: &Runtime,
    row: &Declaration,
    fields: Option<&FieldSpans>,
) -> Result<Arc<pse_modeling::expression::occurrences::ParsedFields>, WorkflowError> {
    let math = runtime.shared.math();
    let generation = math.modeling_cache.generation();
    let extent = row
        .owned_bytes()
        .checked_mul(256)
        .and_then(|n| n.checked_add(8192))
        .ok_or_else(|| contract("shared parsed field extent"))?;
    let scratch = math.reserve("modeling:shared-parsed-fields", extent)?;
    let bytes = payload(&(row, fields))?;
    let key = pse_ids::encoding_checksum(&bytes).content_hash();
    if let Some(parsed) = math.modeling_cache.parsed_fields(key)
        && parsed.matches(row, fields)
    {
        return Ok(parsed);
    }
    let parsed = pse_modeling::expression::occurrences::ParsedFields::parse(row, fields)
        .map_err(super::super::modeling_error)?;
    let mut retained = scratch
        .partition(&[parsed.retained_bytes()])
        .map_err(|_| contract("parsed field extent exceeds admitted scratch"))?
        .into_iter();
    let lease = retained
        .next()
        .ok_or_else(|| contract("parsed field allocation owner absent"))?;
    let parsed = Arc::new(parsed.with_owner(lease));
    math.modeling_cache
        .retain_parsed_fields(generation, key, parsed.clone());
    Ok(parsed)
}

/// Ordered V4 ingress metadata. The canonical source objects still own bytes and
/// references; this frame makes their original order recoverable without data decode.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct IngressFrame {
    order: Vec<DeclarationId>,
    data: BTreeMap<SemanticId, pse_ids::ContentHash>,
    #[serde(default)]
    origins: Option<reload::LoaderOrigins>,
}
impl IngressFrame {
    fn new(rows: &[Declaration], documents: &pse_modeling::document::DocumentInventory) -> Self {
        Self {
            order: rows.iter().map(|row| row.declaration_id).collect(),
            data: documents
                .documents
                .iter()
                .map(|(id, document)| (*id, document.content_hash))
                .collect(),
            origins: None,
        }
    }
}

pub(super) fn logical(id: DeclarationId) -> String {
    format!("modeling:{}", id.as_id())
}
pub(super) fn scope(parent: Option<DeclarationId>) -> String {
    parent.map_or_else(|| ROOT_SCOPE.into(), logical)
}
pub(super) fn kind(row: &Declaration) -> String {
    if selected_source::is_engineering_rule_marker(row) {
        return ENGINEERING_RULE.into();
    }
    format!("modeling:{}", row.value.kind.as_str())
}

/// Owned local scientific authority; optional canonical origin is effect attribution.
#[derive(Clone, Debug)]
pub(in crate::workflow) struct SourceRevision {
    pub(in crate::workflow) origin: Option<Revision>,
    pub(crate) admitted: Arc<AdmittedModelRevision>,
    identity: SourceRevisionHash,
    revision_id: String,
    lineage_id: String,
    generation: i64,
    head: Arc<std::sync::Mutex<local::LocalHead>>,
}
impl SourceRevision {
    pub(in crate::workflow) fn identity(&self) -> SourceRevisionHash { self.identity }
    pub(crate) fn revision_id(&self) -> &str { &self.revision_id }
    pub(crate) fn lineage_id(&self) -> &str { &self.lineage_id }
    pub(crate) fn generation(&self) -> i64 { self.generation }
    pub(crate) fn origin(&self) -> Option<&Revision> { self.origin.as_ref() }
    pub(super) fn require_origin(&self) -> Result<&Revision, WorkflowError> {
        self.origin().ok_or(WorkflowError::RecordingUnavailable)
    }
    pub(super) async fn open(runtime: &Runtime, canonical: Revision) -> Result<Self, WorkflowError> {
        let store = runtime.canonical_store()?;
        let protection = store.protect(canonical.clone(), std::time::Duration::from_secs(3600)).await?;
        let mut read = SelectedRead::new(protection);
        let result: Result<_,WorkflowError> = async {
            let identity = context(runtime,&mut read).await?;
            let cancel = crate::CancelSource::new();
            let mut after = String::new();
            let mut imported = Vec::new();
            let mut leases = Vec::new();
            loop {
                let page = store.membership_page(read.selection(),None,&after).await?;
                let Some(last) = page.last() else {break}; after = last.key.clone();
                for chunk in page.chunks(pse_operations::canonical_staging::SELECTED_OBJECT_BATCH) {
                    let versions=chunk.iter().map(|member|member.version.clone()).collect::<Vec<_>>();
                    for (member,object) in chunk.iter().zip(selected_objects(runtime,&read,&versions,&cancel).await?) {
                        if member.logical != object.logical || member.version != object.key { return Err(contract("imported source membership differs")); }
                        leases.push(object.lease.clone());
                        imported.push(ObjectEdit { logical:member.logical.clone(),scope:member.scope.clone(),name:member.name.clone(),version:Some(object.row),references:Vec::new() });
                    }
                }
            }
            let mut index=SourceIndex {roots_complete:true,..Default::default()};
            for object in &imported {
                if object.logical.starts_with("modeling:") && object.logical != CONTEXT_LOGICAL && object.logical != PHYSICAL_SCOPE_LOGICAL {
                    let row:SourcePayload=decode(object.version.as_ref().ok_or_else(||contract("imported version absent"))?.payload.as_slice())?;
                    index.parsed_fields.insert(row.0.declaration_id,parsed_fields(runtime,&row.0,row.1.as_ref())?);
                    index.scopes.insert(row.0.declaration_id); index.imports.insert(row.0.declaration_id); index.insert(row)?;
                }
            }
            index.requests.clear();
            for object in &mut imported {
                if object.logical.starts_with("modeling:") && object.logical != CONTEXT_LOGICAL && object.logical != PHYSICAL_SCOPE_LOGICAL {
                    let (row,_):SourcePayload=decode(object.version.as_ref().ok_or_else(||contract("imported version absent"))?.payload.as_slice())?;
                    object.references=index.references(&row)?.iter().map(|id| {let target=&index.rows[id];(scope(target.parent_id),target.name.clone())}).collect();
                } else if object.logical.starts_with("record:") {
                    let supplier:DeclarationId=decode(object.version.as_ref().ok_or_else(||contract("imported version absent"))?.payload.as_slice())?;
                    let row=index.rows.get(&supplier).ok_or_else(||contract("imported record supplier absent"))?;
                    object.references=vec![(scope(row.parent_id),row.name.clone())];
                }
                let version=object.version.as_ref().ok_or_else(||contract("imported version absent"))?;
                let expected=edit(object.logical.clone(),object.scope.clone(),object.name.clone(),version.kind.clone(),version.payload.as_slice().to_vec(),object.references.clone());
                if expected.version.as_ref() != Some(version) { return Err(contract("imported immutable source bytes or references differ")); }
            }
            let admitted=AdmittedModelRevision::apply(runtime,None,&imported)?;
            Ok((admitted,identity))
        }.await;
        let release=read.finish().await;
        let (admitted,identity)=result?; release?;
        Self::admitted(admitted,identity,Some(canonical),None)
    }
    fn admitted(
        admitted: Arc<AdmittedModelRevision>, identity: SourceRevisionHash,
        origin: Option<Revision>, previous: Option<&Self>,
    ) -> Result<Self, WorkflowError> {
        let revision_id = pse_operations::mint_id::<SemanticId>().to_string();
        if let Some(previous) = previous {
            // This is committed only after durable acknowledgement. Concurrent siblings
            // serialize here; the loser branches instead of rewinding the live lane.
            let mut head = previous.head.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            if head.revision == previous.revision_id {
                let generation = head.generation.checked_add(1).ok_or_else(||contract("local source generation extent"))?;
                head.revision = revision_id.clone(); head.generation = generation;
                return Ok(Self { origin, admitted, identity, revision_id, lineage_id: previous.lineage_id.clone(), generation, head: previous.head.clone() });
            }
        }
        let lineage_id = pse_operations::mint_id::<SemanticId>().to_string();
        let head = Arc::new(std::sync::Mutex::new(local::LocalHead { revision: revision_id.clone(), generation: 0 }));
        Ok(Self { origin, admitted, identity, revision_id, lineage_id, generation: 0, head })
    }
}

type FieldSpans = BTreeMap<String, SourceSpan>;
type SourcePayload = (Declaration, Option<FieldSpans>);

fn payload<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, WorkflowError> {
    serde_json::to_vec(value)
        .map_err(|error| contract(format!("canonical source encoding: {error}")))
}
fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, WorkflowError> {
    serde_json::from_slice(bytes)
        .map_err(|error| contract(format!("canonical source decoding: {error}")))
}
pub(super) fn edit(
    logical: String,
    scope: String,
    name: String,
    kind: String,
    bytes: Vec<u8>,
    references: Vec<(String, String)>,
) -> ObjectEdit {
    let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::CanonicalSourceObjectV1);
    hash.str(pse_operations::generated::surreal::INTERPRETATION)
        .str(&kind)
        .str(&logical)
        .part(&bytes)
        .u64(references.len() as u64);
    for (scope, name) in &references {
        hash.str(scope).str(name);
    }
    let identity = hash.finish_hash();
    ObjectEdit {
        logical: logical.clone(),
        scope,
        name,
        version: Some(pse_model::generated::runtime::canonical_versions::Row {
            key: identity.to_string(),
            logical,
            kind,
            payload: bytes.into(),
            interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
        }),
        references,
    }
}

#[derive(Debug)]
struct OwnedObject {
    row: pse_model::generated::runtime::canonical_versions::Row,
    lease: Arc<pse_columnar::AllocationLease>,
}
impl std::ops::Deref for OwnedObject {
    type Target = pse_model::generated::runtime::canonical_versions::Row;
    fn deref(&self) -> &Self::Target {
        &self.row
    }
}
async fn selected_object(
    runtime: &Runtime,
    read: &SelectedRead,
    version: &str,
) -> Result<OwnedObject, WorkflowError> {
    let store = runtime.canonical_store()?;
    let extent = store
        .selected_object_extent(read.selection(), version)
        .await?
        .ok_or_else(|| contract("selected canonical source extent absent"))?;
    let bytes = extent
        .checked_mul(4)
        .and_then(|n| n.checked_add(size_of::<Declaration>() + 1024))
        .ok_or_else(|| contract("canonical source decode extent overflow"))?;
    let lease = runtime
        .shared
        .math()
        .reserve("modeling:canonical-source-decode", bytes)?;
    let row = store
        .selected_object(read.selection(), version)
        .await?
        .ok_or_else(|| contract("selected canonical source version absent"))?;
    if row.payload.len() != extent {
        return Err(contract(
            "canonical payload extent differs from closed manifest",
        ));
    }
    Ok(OwnedObject { row, lease })
}

async fn selected_objects(
    runtime: &Runtime,
    read: &SelectedRead,
    versions: &[String],
    cancel: &crate::CancelSource,
) -> Result<Vec<OwnedObject>, WorkflowError> {
    use pse_operations::canonical_staging::{SELECTED_OBJECT_HEADER_SCRATCH, SOURCE_BLOCK_BYTES};
    checkpoint(cancel)?;
    // Manifest and bounded block responses are transient, accounted before the
    // first protected query, independently of the selected scientific payloads.
    let _scratch = runtime.shared.math().reserve(
        "modeling:source-frontier-headers",
        SELECTED_OBJECT_HEADER_SCRATCH,
    )?;
    let store = runtime.canonical_store()?;
    let batch = store
        .selected_object_batch(read.selection(), versions)
        .await?;
    checkpoint(cancel)?;
    let mut leases = Vec::new();
    let mut extents = Vec::new();
    for (_, extent) in batch.extents() {
        let bytes = extent
            .checked_mul(4)
            .and_then(|bytes| bytes.checked_add(size_of::<Declaration>() + 1024))
            .ok_or_else(|| contract("canonical source decode extent overflow"))?;
        leases.push(
            runtime
                .shared
                .math()
                .reserve("modeling:canonical-source-decode", bytes)?,
        );
        extents.push(extent);
    }
    let mut payloads = Vec::new();
    for extent in &extents {
        let mut payload = Vec::new();
        payload
            .try_reserve_exact(*extent)
            .map_err(|error| contract(format!("selected source allocation: {error}")))?;
        payloads.push(payload);
    }
    while let Some(object) = payloads
        .iter()
        .zip(&extents)
        .position(|(payload, extent)| payload.len() < *extent)
    {
        checkpoint(cancel)?;
        let ordinal = (payloads[object].len() / SOURCE_BLOCK_BYTES) as u64;
        let blocks = store
            .selected_object_blocks(&batch, object, ordinal)
            .await?;
        checkpoint(cancel)?;
        for (object, block) in blocks {
            if block.ordinal as usize * SOURCE_BLOCK_BYTES != payloads[object].len() {
                return Err(contract("selected grouped block order differs"));
            }
            payloads[object].extend_from_slice(block.payload.as_slice());
        }
    }
    checkpoint(cancel)?;
    let rows = store.finish_selected_object_batch(batch, payloads).await?;
    checkpoint(cancel)?;
    Ok(rows
        .into_iter()
        .zip(leases)
        .map(|(row, lease)| OwnedObject { row, lease })
        .collect())
}
async fn object(
    runtime: &Runtime,
    read: &mut SelectedRead,
    name: &str,
) -> Result<Option<OwnedObject>, WorkflowError> {
    let members = runtime
        .canonical_store()?
        .resolve_logicals(read, &[name.into()])
        .await?;
    if members.len() > 1 {
        return Err(contract("canonical logical source identity is not unique"));
    }
    let Some(member) = members.first() else {
        return Ok(None);
    };
    selected_object(runtime, read, &member.version)
        .await
        .map(Some)
}
/// Resolve each exact nullable identity once, then precharge and hydrate its
/// immutable objects as a group. Absent identities remain read premises.
async fn objects(
    runtime: &Runtime,
    read: &mut SelectedRead,
    logicals: &[String],
    cancel: &crate::CancelSource,
) -> Result<BTreeMap<String, OwnedObject>, WorkflowError> {
    let mut objects = BTreeMap::new();
    for logicals in logicals.chunks(pse_operations::canonical_staging::SELECTED_OBJECT_BATCH) {
        checkpoint(cancel)?;
        let _scratch = runtime.shared.math().reserve(
            "modeling:exact-object-metadata",
            pse_operations::canonical_staging::SELECTED_OBJECT_HEADER_SCRATCH,
        )?;
        let members = runtime
            .canonical_store()?
            .resolve_logicals(read, logicals)
            .await?;
        let versions = members
            .iter()
            .map(|member| member.version.clone())
            .collect::<Vec<_>>();
        if versions.is_empty() {
            continue;
        }
        let selected = selected_objects(runtime, read, &versions, cancel).await?;
        for (member, object) in members.into_iter().zip(selected) {
            if member.logical != object.logical || member.version != object.key {
                return Err(contract("canonical exact object association differs"));
            }
            objects.insert(member.logical, object);
        }
    }
    Ok(objects)
}
async fn context(
    runtime: &Runtime,
    read: &mut SelectedRead,
) -> Result<SourceRevisionHash, WorkflowError> {
    let context = object(runtime, read, CONTEXT_LOGICAL)
        .await?
        .ok_or_else(|| contract("canonical modeling context absent"))?;
    if context.kind != CONTEXT {
        return Err(contract(format!(
            "canonical modeling context interpretation differs: expected {CONTEXT}, observed {}; restage the authored source under the current interpretation",
            context.kind
        )));
    }
    decode(context.payload.as_slice())
}
async fn physical_context(
    runtime: &Runtime,
    read: &mut SelectedRead,
) -> Result<(PhysicalScope, Arc<pse_columnar::AllocationLease>, String), WorkflowError> {
    let value = object(runtime, read, PHYSICAL_SCOPE_LOGICAL)
        .await?
        .ok_or_else(|| contract("canonical physical name scope absent"))?;
    if value.kind != "physical_scope" {
        return Err(contract(
            "canonical physical name scope interpretation differs",
        ));
    }
    Ok((
        decode(value.payload.as_slice())?,
        value.lease,
        value.row.key,
    ))
}

/// Native typed rows enter the immutable store once. No checked package or Arrow
/// source projection escapes this ingress operation.
#[expect(
    clippy::too_many_arguments,
    reason = "one immutable revision persists typed declarations with their physical interpretation and scope, document provenance, source bytes, fit declarations and optional prior revision fence"
)]
pub(super) async fn persist(
    runtime: &Runtime,
    rows: Vec<Declaration>,
    physical: &super::PhysicalContext,
    physical_scope: PhysicalScope,
    documents: &pse_modeling::document::DocumentInventory,
    sources: &pse_admission::authoring_driver::document::Batches,
    fits: &super::super::fitting::FitDeclarations,
    previous: Option<&SourceRevision>,
    loader: Option<&pse_admission::authoring_driver::document::OwnedDocumentSet>,
) -> Result<SourceRevision, WorkflowError> {
    use pse_relations::columnar::RelationRow;
    let bytes = rows
        .owned_bytes()
        .checked_add(fits.fits.owned_bytes())
        .and_then(|n| n.checked_mul(8))
        .and_then(|n| n.checked_add(documents.retained_bytes()))
        .ok_or_else(|| contract("canonical ingress extent"))?;
    let _lease = runtime
        .shared
        .math()
        .reserve("modeling:canonical-ingress", bytes)?;
    let validation = runtime.validation_context();
    let mut builder =
        pse_relations::generated::authored::modeling_declarations::Builder::with_registry(
            runtime.registry(),
            rows.len(),
            &validation,
        )
        .map_err(super::super::relation)?;
    for row in &rows {
        builder.push(row.clone()).map_err(super::super::relation)?;
    }
    let native = builder.finish().map_err(super::super::relation)?;
    drop(native);
    let names = selected_source::physical_bindings(&rows, &physical_scope, &physical.quantities);
    let identity = crate::math::modeling::source_revision(&rows, documents, &physical.key, &names);
    let mut index = SourceIndex {
        roots_complete: true,
        ..Default::default()
    };
    for row in &rows {
        index.insert((
            row.clone(),
            documents.field_spans.get(&row.declaration_id).cloned(),
        ))?;
        index.parsed_fields.insert(
            row.declaration_id,
            parsed_fields(runtime, row, documents.field_spans.get(&row.declaration_id))?,
        );
        index.scopes.insert(row.declaration_id);
        index.imports.insert(row.declaration_id);
    }
    index.requests.clear();
    let mut edits = Vec::with_capacity(rows.len() + sources.len() + fits.fits.len() + 1);
    for row in &rows {
        let references = index
            .references(row)?
            .into_iter()
            .map(|id| {
                let target = &index.rows[&id];
                (scope(target.parent_id), target.name.clone())
            })
            .collect();
        edits.push(edit(
            logical(row.declaration_id),
            scope(row.parent_id),
            row.name.clone(),
            kind(row),
            payload(&(row, documents.field_spans.get(&row.declaration_id).cloned()))?,
            references,
        ));
    }
    index.requests.clear();
    let data_rows = index.data_closure()?;
    if !data_rows.is_empty() {
        let workspace = runtime.shared.math().workspace(
            super::compiler_context(physical, &BTreeMap::new()),
            pse_compiler::workspace::WorkspaceLimits::default(),
        )?;
        let mut data_documents = documents.clone();
        data_documents.parsed_fields = index.parsed_fields.clone();
        let data = runtime.shared.math().modeling_revision(
            &workspace,
            data_rows,
            physical_scope.clone(),
            Arc::new(data_documents),
            &physical.key,
        )?;
        for (record, supplier) in selected_source::record_sources(data.checked()) {
            let source = &index.rows[&supplier];
            edits.push(edit(
                format!("record:{record}"),
                "records".into(),
                record.to_string(),
                "record_supplier".into(),
                payload(&supplier)?,
                vec![(scope(source.parent_id), source.name.clone())],
            ));
        }
    }
    edits.push(edit(
        CONTEXT_LOGICAL.into(),
        "context".into(),
        CONTEXT.into(),
        CONTEXT.into(),
        payload(&identity)?,
        Vec::new(),
    ));
    let mut header = physical_scope.clone();
    header.documents = physical_scope.documents.as_ref().map(|_| BTreeSet::new());
    edits.push(edit(
        PHYSICAL_SCOPE_LOGICAL.into(),
        "context".into(),
        "physical_scope".into(),
        "physical_scope".into(),
        payload(&header)?,
        Vec::new(),
    ));
    if let Some(visible) = &physical_scope.documents {
        for document in visible {
            let entry = PhysicalScope {
                package: physical_scope.package.clone(),
                documents: Some(BTreeSet::from([*document])),
            };
            edits.push(edit(
                format!("physical_scope_document:{document}"),
                "physical_name_scopes".into(),
                document.to_string(),
                "physical_scope_document".into(),
                payload(&entry)?,
                Vec::new(),
            ));
        }
    }
    if let Some(batch) = sources.get(&pse_relations::generated::authored::packages::RELATION_ID) {
        for row in pse_relations::generated::authored::packages::Row::rows(batch)
            .map_err(super::super::relation)?
        {
            edits.push(edit(
                format!("package:{}", row.package_id),
                "packages".into(),
                row.package_id.to_string(),
                "package".into(),
                payload(&row)?,
                Vec::new(),
            ));
        }
    }
    if let Some(batch) = sources.get(&pse_relations::generated::authored::documents::RELATION_ID) {
        for mut row in pse_relations::generated::authored::documents::Row::rows(batch)
            .map_err(super::super::relation)?
        {
            if let Some(content) = row.content.take() {
                edits.push(edit(
                    format!("data:{}", row.document_id),
                    "data".into(),
                    row.document_id.to_string(),
                    "data".into(),
                    content.into_vec(),
                    Vec::new(),
                ));
            }
            edits.push(edit(
                format!("document:{}", row.document_id),
                "documents".into(),
                row.document_id.to_string(),
                "document".into(),
                payload(&row)?,
                Vec::new(),
            ));
        }
    }
    for row in &fits.fits {
        edits.push(edit(
            format!("fit:{}", row.fit_id),
            "fits".into(),
            row.fit_id.to_string(),
            "fit".into(),
            payload(row)?,
            Vec::new(),
        ));
    }
    let mut membership_leases = Vec::new();
    let mut frame = IngressFrame::new(&rows, documents);
    if let Some(loader) = loader {
        frame.origins = Some(reload::LoaderOrigins::from_bundles(
            loader.bundles(),
            &[],
            &edits,
        )?);
    }
    edits.push(edit(
        INGRESS_FRAME.into(),
        "context".into(),
        "ingress_frame".into(),
        "ingress_frame".into(),
        payload(&frame)?,
        Vec::new(),
    ));
    if let Some(previous) = previous {
        let retained = edits.iter().map(|edit|edit.logical.clone()).collect::<BTreeSet<_>>();
        edits.retain(|edit| previous.admitted.objects.get(&edit.logical).is_none_or(|old| {
            edit.version.as_ref().is_none_or(|version| old.scope != edit.scope || old.name != edit.name
                || old.version != version.key || old.kind != version.kind
                || old.payload.as_slice() != version.payload.as_slice() || old.references != edit.references)
        }));
        edits.extend(previous.admitted.objects.values().filter(|member| {
            (member.logical.starts_with("modeling:") || member.logical.starts_with("record:") || member.logical.starts_with("physical_scope_document:"))
                && !retained.contains(&member.logical)
        }).map(|member| ObjectEdit { logical:member.logical.clone(),scope:member.scope.clone(),name:member.name.clone(),version:None,references:Vec::new() }));
        if edits.is_empty() && identity == previous.identity { return Ok(previous.clone()); }
    }
    admit_revision(runtime, previous, identity, &edits).await
}

/// Stage complete local ownership before issuing the existing durable effect.
async fn admit_revision(
    runtime: &Runtime, previous: Option<&SourceRevision>, identity: SourceRevisionHash,
    edits: &[ObjectEdit],
) -> Result<SourceRevision, WorkflowError> {
    let admitted = AdmittedModelRevision::apply(runtime, previous.map(|p|p.admitted.as_ref()), edits)?;
    let origin = match runtime.durability() {
        crate::workflow::Durability::Ephemeral => None,
        crate::workflow::Durability::Durable(_) => {
            let store = runtime.canonical_store()?;
            let old = previous.map(SourceRevision::require_origin).transpose()?;
            let problem = old.map_or_else(||pse_operations::mint_id::<SemanticId>().to_string(), |old|old.problem.clone());
            let operation = pse_operations::mint_id::<SemanticId>().to_string();
            let recorded = store.edit(&problem,old.map(|old|old.key.as_str()),&operation,edits).await
                .map_err(|source|pse_operations::canonical::CanonicalError::SourceEditUnsettled {problem,operation,source:Box::new(source)})?;
            Some(recorded)
        }
    };
    SourceRevision::admitted(admitted,identity,origin,previous)
}

/// One compiler input, resolved entirely from immutable canonical records. Read
/// premises remain protected until the preparation or portable publication ends.
#[cfg(all(test, feature = "canonical-tests"))]
pub(super) struct SelectedSource {
    pub(super) rows: Vec<Declaration>,
    pub(super) physical: PhysicalScope,
    pub(super) read: SelectedRead,
    pub(super) versions: BTreeMap<String, String>,
    record_sources: BTreeMap<DeclarationId, DeclarationId>,
    _leases: Vec<Arc<pse_columnar::AllocationLease>>,
}

/// Active eligibility belongs to this call, never to retained admission.
#[derive(Debug)]
pub(super) struct CheckedSelection {
    pub(super) admission: Arc<super::SelectedAdmission>,
    pub(super) _premises_metadata: Option<Arc<pse_columnar::AllocationLease>>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Request {
    Logical(DeclarationId),
    Name(Option<DeclarationId>, String),
    Imports(DeclarationId),
    Members(DeclarationId),
}

#[derive(Default)]
struct SourceIndex {
    rows: BTreeMap<DeclarationId, Declaration>,
    fields: BTreeMap<DeclarationId, FieldSpans>,
    parsed_fields:
        BTreeMap<DeclarationId, Arc<pse_modeling::expression::occurrences::ParsedFields>>,
    children: BTreeMap<DeclarationId, Vec<DeclarationId>>,
    named_children: BTreeMap<(DeclarationId, String), Vec<DeclarationId>>,
    named_roots: BTreeMap<String, Vec<DeclarationId>>,
    guards: BTreeMap<DeclarationId, Vec<DeclarationId>>,
    imported_names: BTreeMap<(DeclarationId, String), Vec<String>>,
    names: BTreeSet<(Option<DeclarationId>, String)>,
    imports: BTreeSet<DeclarationId>,
    scopes: BTreeSet<DeclarationId>,
    requests: BTreeSet<Request>,
    new_rows: BTreeSet<DeclarationId>,
    pending_references: BTreeMap<DeclarationId, selected_source::DeclarationReferences>,
    probing: BTreeSet<(DeclarationId, String)>,
    roots_complete: bool,
    versions: BTreeMap<String, String>,
    record_sources: BTreeMap<DeclarationId, DeclarationId>,
}
impl SourceIndex {
    fn insert(&mut self, (row, fields): SourcePayload) -> Result<(), WorkflowError> {
        let id = row.declaration_id;
        if let Some(previous) = self.rows.get(&id) {
            return if previous == &row && self.fields.get(&id) == fields.as_ref() {
                Ok(())
            } else {
                Err(contract(
                    "one selected declaration has incompatible source versions",
                ))
            };
        }
        if let Some(parent) = row.parent_id {
            self.children.entry(parent).or_default().push(id);
            self.named_children
                .entry((parent, row.name.clone()))
                .or_default()
                .push(id);
            if row.value.kind == Kind::When {
                self.guards.entry(parent).or_default().push(id);
            }
            if let Some(import) = &row.value.import {
                self.imported_names
                    .entry((parent, import.alias.as_deref().unwrap_or(&row.name).into()))
                    .or_default()
                    .push(row.name.clone());
            }
            if !self.rows.contains_key(&parent) {
                self.requests.insert(Request::Logical(parent));
            }
        } else {
            self.named_roots
                .entry(row.name.clone())
                .or_default()
                .push(id);
        }
        if let Some(fields) = fields {
            self.fields.insert(id, fields);
        }
        self.rows.insert(id, row);
        self.new_rows.insert(id);
        Ok(())
    }
    fn data_closure(&mut self) -> Result<Vec<Declaration>, WorkflowError> {
        let mut keep = BTreeSet::new();
        let mut pending = self
            .rows
            .values()
            .filter(|row| {
                row.value.entity.is_some()
                    || row.value.dataset.is_some()
                    || matches!(row.value.kind, Kind::EntityKind | Kind::IdentifierScheme)
            })
            .map(|row| row.declaration_id)
            .collect::<Vec<_>>();
        while let Some(id) = pending.pop() {
            if !keep.insert(id) {
                continue;
            }
            let row = self.rows[&id].clone();
            pending.extend(row.parent_id);
            pending.extend(
                match row.parent_id {
                    Some(parent) => self.named_children.get(&(parent, row.name.clone())),
                    None => self.named_roots.get(&row.name),
                }
                .into_iter()
                .flatten()
                .copied(),
            );
            pending.extend(
                self.children
                    .get(&id)
                    .into_iter()
                    .flatten()
                    .filter(|child| {
                        selected_source::requires_members(&row)
                            || self.rows[child].value.import.is_some()
                    })
                    .copied(),
            );
            if row.value.import.is_some() {
                pending.extend(
                    self.named_roots
                        .get(&row.name)
                        .into_iter()
                        .flatten()
                        .copied(),
                );
            }
            pending.extend(self.references(&row)?);
        }
        if !self.requests.is_empty() {
            return Err(contract(
                "complete ingress source index left an unresolved lexical read",
            ));
        }
        Ok(self
            .rows
            .values()
            .filter(|row| keep.contains(&row.declaration_id))
            .cloned()
            .collect())
    }
    fn resolve(&mut self, owner: DeclarationId, path: &dsl::Path) -> Lookup {
        selected_source::resolve_segments(self, owner, &path.segments)
    }
    fn resolve_name(&mut self, owner: DeclarationId, name: &str) -> Lookup {
        let atomic = dsl::Path {
            segments: vec![dsl::PathSegment {
                name: name.into(),
                indices: Vec::new(),
            }],
        };
        match self.resolve(owner, &atomic) {
            Lookup::Absent => {
                let path = name
                    .split('.')
                    .map(|name| dsl::PathSegment {
                        name: name.into(),
                        indices: Vec::new(),
                    })
                    .collect();
                self.resolve(owner, &dsl::Path { segments: path })
            }
            other => other,
        }
    }
    fn references(&mut self, row: &Declaration) -> Result<BTreeSet<DeclarationId>, WorkflowError> {
        let parsed = self
            .parsed_fields
            .get(&row.declaration_id)
            .ok_or_else(|| contract("source references lack shared parsed fields"))?
            .clone();
        let occurrences = parsed.expressions();
        let mut found = BTreeSet::new();
        for path in selected_source::type_paths(row) {
            if path.segments.len() == 1
                && self.type_bound(row.declaration_id, &path.segments[0].name)
            {
                continue;
            }
            if let Lookup::Found(ids) = self.resolve(row.declaration_id, &path) {
                found.extend(ids);
            }
        }
        for path in selected_source::structural_paths(row) {
            for end in (1..=path.segments.len()).rev() {
                match selected_source::resolve_segments(
                    self,
                    row.declaration_id,
                    &path.segments[..end],
                ) {
                    Lookup::Absent => {}
                    Lookup::Pending => break,
                    Lookup::Found(ids) => {
                        found.extend(ids);
                        break;
                    }
                }
            }
        }
        // Structural spellings not represented by a parsed expression occurrence
        // include dataset targets, entity kinds, bases and authored provenance.
        for text in selected_source::requirements(row).names {
            if occurrences
                .values()
                .any(|occurrence| occurrence.text == text)
            {
                continue;
            }
            if let Ok(expression) = dsl::parse_expr(text) {
                for reference in selected_source::syntax_references(
                    &pse_modeling::expression::occurrences::Syntax::Expression(expression),
                ) {
                    match reference {
                        selected_source::SourceReference::Name(name) => {
                            if let Lookup::Found(ids) = self.resolve_name(row.declaration_id, &name)
                            {
                                found.extend(ids);
                            }
                        }
                        selected_source::SourceReference::Segments(path) => {
                            if let Lookup::Found(ids) = self.resolve(row.declaration_id, &path) {
                                found.extend(ids);
                            }
                        }
                    }
                }
            } else if let Lookup::Found(ids) = self.resolve_name(row.declaration_id, text) {
                found.extend(ids);
            }
        }
        for occurrence in occurrences.values() {
            for reference in selected_source::syntax_references(&occurrence.syntax) {
                match reference {
                    selected_source::SourceReference::Segments(path) => {
                        if path.segments.first().is_some_and(|segment| {
                            occurrence
                                .index_obligations
                                .iter()
                                .any(|(name, _)| name == &segment.name)
                        }) {
                            continue;
                        }
                        for end in (1..=path.segments.len()).rev() {
                            match selected_source::resolve_segments(
                                self,
                                row.declaration_id,
                                &path.segments[..end],
                            ) {
                                Lookup::Absent => {}
                                Lookup::Pending => break,
                                Lookup::Found(ids) => {
                                    found.extend(ids);
                                    break;
                                }
                            }
                        }
                    }
                    selected_source::SourceReference::Name(name) => {
                        if let Lookup::Found(ids) = self.resolve_name(row.declaration_id, &name) {
                            found.extend(ids);
                        }
                    }
                }
            }
        }
        Ok(found)
    }
    fn pending(
        &mut self,
        owner: DeclarationId,
        mut references: selected_source::DeclarationReferences,
    ) {
        references.types.retain(|path| {
            if path.segments.len() == 1 && self.type_bound(owner, &path.segments[0].name) {
                return false;
            }
            self.resolve(owner, path) == Lookup::Pending
        });
        references
            .exact
            .retain(|path| self.resolve(owner, path) == Lookup::Pending);
        references.prefixes.retain(|path| {
            for end in (1..=path.segments.len()).rev() {
                match selected_source::resolve_segments(self, owner, &path.segments[..end]) {
                    Lookup::Absent => {}
                    Lookup::Pending => return true,
                    Lookup::Found(_) => return false,
                }
            }
            false
        });
        references
            .names
            .retain(|name| self.resolve_name(owner, name) == Lookup::Pending);
        if !references.types.is_empty()
            || !references.exact.is_empty()
            || !references.prefixes.is_empty()
            || !references.names.is_empty()
        {
            self.pending_references.insert(owner, references);
        }
    }
    fn needs_name(&self, parent: Option<DeclarationId>, name: &str) -> bool {
        !self.names.contains(&(parent, name.into()))
            && !parent.is_some_and(|parent| self.scopes.contains(&parent))
            && !(parent.is_none() && self.roots_complete)
    }
    fn type_bound(&self, mut owner: DeclarationId, name: &str) -> bool {
        let mut seen = BTreeSet::new();
        while seen.insert(owner) {
            let Some(row) = self.rows.get(&owner) else {
                return false;
            };
            if row
                .value
                .scope
                .iter()
                .flat_map(|scope| &scope.type_parameters)
                .any(|parameter| parameter == name)
                || row
                    .value
                    .function
                    .iter()
                    .flat_map(|function| &function.type_parameters)
                    .any(|parameter| parameter == name)
            {
                return true;
            }
            let Some(parent) = row.parent_id else {
                return false;
            };
            owner = parent;
        }
        false
    }
    fn guarded(
        &mut self,
        owner: DeclarationId,
        name: &str,
        found: &mut BTreeSet<DeclarationId>,
    ) -> bool {
        let mut ready = true;
        found.extend(
            self.named_children
                .get(&(owner, name.into()))
                .into_iter()
                .flatten()
                .copied(),
        );
        for child in self.guards.get(&owner).cloned().unwrap_or_default() {
            if !self.scopes.contains(&child) {
                self.requests.insert(Request::Members(child));
                ready = false;
            } else {
                ready &= self.guarded(child, name, found);
            }
        }
        ready
    }
    fn member_candidates(&mut self, owner: DeclarationId, name: &str) -> Lookup {
        let Some(row) = self.rows.get(&owner) else {
            self.requests.insert(Request::Logical(owner));
            return Lookup::Pending;
        };
        if !selected_source::owns_namespace(row) {
            return Lookup::Absent;
        }
        if row.value.kind != Kind::Package && !self.scopes.contains(&owner) {
            self.requests.insert(Request::Members(owner));
            return Lookup::Pending;
        }
        if row.value.kind == Kind::Package
            && !self.names.contains(&(Some(owner), name.into()))
            && !self.scopes.contains(&owner)
        {
            self.requests
                .insert(Request::Name(Some(owner), name.into()));
            return Lookup::Pending;
        }
        let bases = row
            .value
            .scope
            .as_ref()
            .map(|scope| scope.bases.clone())
            .unwrap_or_default();
        let own = self
            .named_children
            .get(&(owner, name.into()))
            .cloned()
            .unwrap_or_default();
        if !own.is_empty() {
            return Lookup::Found(own);
        }
        let mut found = BTreeSet::new();
        let mut ready = self.guarded(owner, name, &mut found);
        for base in bases {
            match self.resolve_name(owner, &base) {
                Lookup::Found(bases) => {
                    for base in bases {
                        match self.member(base, name) {
                            Lookup::Found(ids) => found.extend(ids),
                            Lookup::Pending => ready = false,
                            Lookup::Absent => {}
                        }
                    }
                }
                Lookup::Pending => ready = false,
                Lookup::Absent => {}
            }
        }
        if !ready {
            Lookup::Pending
        } else if found.is_empty() {
            Lookup::Absent
        } else {
            Lookup::Found(found.into_iter().collect())
        }
    }
}
impl LexicalLookup for SourceIndex {
    fn declaration(&self, id: DeclarationId) -> Option<&Declaration> {
        self.rows.get(&id)
    }
    fn bound(&self, owner: DeclarationId, name: &str) -> bool {
        self.rows.get(&owner).is_some_and(|row| {
            row.value
                .scope
                .iter()
                .flat_map(|scope| &scope.parameters)
                .any(|parameter| parameter.name == name)
                || row
                    .value
                    .function
                    .iter()
                    .flat_map(|function| &function.arguments)
                    .any(|argument| argument.name == name)
                || row
                    .value
                    .coordinate_map
                    .iter()
                    .flat_map(|map| &map.arguments)
                    .any(|argument| argument.name == name)
                || row
                    .value
                    .response
                    .iter()
                    .flat_map(|function| &function.arguments)
                    .any(|argument| argument.name == name)
                || row
                    .value
                    .reconstruction
                    .iter()
                    .flat_map(|function| &function.arguments)
                    .any(|argument| argument.name == name)
                || row
                    .value
                    .reference_translation
                    .iter()
                    .flat_map(|function| &function.arguments)
                    .any(|argument| argument.name == name)
        })
    }
    fn imported(&mut self, owner: DeclarationId, name: &str) -> Lookup {
        if self
            .rows
            .get(&owner)
            .is_some_and(|row| !selected_source::owns_namespace(row))
        {
            return Lookup::Absent;
        }
        if !self.imports.contains(&owner) && !self.scopes.contains(&owner) {
            self.requests.insert(Request::Imports(owner));
            return Lookup::Pending;
        }
        let imports = self
            .imported_names
            .get(&(owner, name.into()))
            .cloned()
            .unwrap_or_default();
        if imports.is_empty() {
            return Lookup::Absent;
        }
        let mut found = Vec::new();
        for import in imports {
            if !self.roots_complete && !self.names.contains(&(None, import.clone())) {
                self.requests.insert(Request::Name(None, import));
                return Lookup::Pending;
            }
            found.extend(
                self.rows
                    .values()
                    .filter(|row| row.parent_id.is_none() && row.name == import)
                    .map(|row| row.declaration_id),
            );
        }
        if found.is_empty() {
            Lookup::Absent
        } else {
            Lookup::Found(found)
        }
    }
    fn member(&mut self, owner: DeclarationId, name: &str) -> Lookup {
        let key = (owner, name.into());
        if !self.probing.insert(key.clone()) {
            return Lookup::Absent;
        }
        let result = self.member_candidates(owner, name);
        self.probing.remove(&key);
        result
    }
}

impl ModelingPackage {
    fn local_object(&self, logical: &str) -> Result<Arc<LocalSourceObject>, WorkflowError> {
        self.revision.admitted.objects.get(logical).cloned().ok_or_else(||contract(format!("owned source object absent: {logical}")))
    }
    fn source_decode<T:serde::de::DeserializeOwned>(&self, object:&LocalSourceObject) -> Result<(T,Arc<pse_columnar::AllocationLease>),WorkflowError> {
        let extent=object.payload.len().checked_mul(32).and_then(|n|n.checked_add(8192)).ok_or_else(||contract("owned source decode extent"))?;
        let owner=self.runtime.shared.math().reserve("modeling:owned-source-decode",extent)?;
        Ok((decode(&object.payload)?,owner))
    }
    pub(super) async fn declaration_inventory(&self) -> Result<super::OwnedDeclarations,WorkflowError> {
        let parent=self.delta_parent(&crate::CancelSource::new()).await?;
        Ok(super::OwnedDeclarations {rows:parent.rows,_leases:parent._leases})
    }
    pub async fn fit_declarations(&self) -> Result<super::super::fitting::FitDeclarations,WorkflowError> {
        let mut rows=Vec::new();let mut leases=Vec::new();
        for object in self.revision.admitted.objects.values().filter(|o|o.kind=="fit") {
            let (row,lease)=self.source_decode(object)?;rows.push(row);leases.push(lease);
        }
        Ok(super::super::fitting::FitDeclarations::from_owned_rows(rows,leases))
    }
    pub(in crate::workflow) async fn fit_declaration(&self,id:pse_model::generated::identities::FitId)->Result<pse_model::generated::authored::fit_cases::Row,WorkflowError> {
        let object=self.local_object(&format!("fit:{id}"))?;
        Ok(self.source_decode(&object)?.0)
    }
    pub(in crate::workflow) async fn replace_fits(&self,fits:&[pse_model::generated::authored::fit_cases::Row])->Result<SourceRevision,WorkflowError> {
        let existing=self.fit_declarations().await?;
        let old=existing.fits.iter().map(|row|(row.fit_id,row)).collect::<BTreeMap<_,_>>();
        let new=fits.iter().map(|row|(row.fit_id,row)).collect::<BTreeMap<_,_>>();
        let delta=crate::workflow::SourceDelta {
            parent:self.revision_id().into(),
            fits:crate::workflow::SourceFitDelta {
                additions:fits.iter().filter(|row|!old.contains_key(&row.fit_id)).cloned().collect(),
                replacements:fits.iter().filter(|row|old.get(&row.fit_id).is_some_and(|old|*old!=*row)).cloned().collect(),
                removals:old.keys().filter(|id|!new.contains_key(id)).copied().collect(),
            },..Default::default()
        };
        Ok(self.with_delta(delta,&crate::CancelSource::new()).await?.revision)
    }
    pub(super) async fn full_source(&self)->Result<(Vec<Declaration>,PhysicalScope,Arc<pse_modeling::document::DocumentInventory>,pse_admission::authoring_driver::document::Batches,Vec<Arc<pse_columnar::AllocationLease>>),WorkflowError> {
        use pse_relations::generated::authored::{documents,fit_cases,packages};
        let parent=self.delta_parent(&crate::CancelSource::new()).await?;
        let mut leases=parent._leases;
        let validation=self.runtime.validation_context();
        let mut inventory=pse_modeling::document::DocumentInventory {field_spans:parent.fields,..Default::default()};
        let mut document_builder=documents::Builder::with_registry(self.runtime.registry(),parent.documents.len(),&validation).map_err(super::super::relation)?;
        for mut row in parent.documents.into_values() {
            inventory.packages.insert(row.document_id,row.package_id.as_id());
            if let Some(data)=self.revision.admitted.objects.get(&format!("data:{}",row.document_id)) {
                let (document,lease)=pse_admission::authoring_driver::document::decode_selected_data(row.document_id,row.path.clone(),data.payload.clone().into(),Default::default(),&self.runtime.shared.pool(),&crate::CancelSource::new().token())?;
                inventory.documents.insert(row.document_id,document);leases.push(lease);row.content=Some(data.payload.clone().into());
            }
            document_builder.push(row).map_err(super::super::relation)?;
        }
        let mut package_builder=packages::Builder::with_registry(self.runtime.registry(),0,&validation).map_err(super::super::relation)?;
        let mut fit_builder=fit_cases::Builder::with_registry(self.runtime.registry(),0,&validation).map_err(super::super::relation)?;
        for object in self.revision.admitted.objects.values() {
            match object.kind.as_str() {
                "package"=>{let (row,lease)=self.source_decode(object)?;package_builder.push(row).map_err(super::super::relation)?;leases.push(lease);},
                "fit"=>{let (row,lease)=self.source_decode(object)?;fit_builder.push(row).map_err(super::super::relation)?;leases.push(lease);},_=>{}
            }
        }
        for row in &parent.rows {inventory.parsed_fields.insert(row.declaration_id,parsed_fields(&self.runtime,row,inventory.field_spans.get(&row.declaration_id))?);}
        let sources=BTreeMap::from([(documents::RELATION_ID,document_builder.finish().map_err(super::super::relation)?),(packages::RELATION_ID,package_builder.finish().map_err(super::super::relation)?),(fit_cases::RELATION_ID,fit_builder.finish().map_err(super::super::relation)?)]);
        Ok((parent.rows,parent.physical,Arc::new(inventory),sources,leases))
    }
    pub(super) async fn selected_declarations(&self,selected:&BTreeSet<DeclarationId>,cancel:&crate::CancelSource)->Result<super::OwnedDeclarations,WorkflowError> {
        if selected.is_empty(){return Err(contract("a fixture selection names at least one test"));}
        let parent=self.delta_parent(cancel).await?;
        let rows=parent.rows.into_iter().filter(|row|selected.contains(&row.declaration_id)).collect::<Vec<_>>();
        if rows.len()!=selected.len(){return Err(contract("selected declaration absent"));}
        Ok(super::OwnedDeclarations {rows,_leases:parent._leases})
    }
    pub(super) async fn declaration(&self,id:DeclarationId)->Result<Declaration,WorkflowError> {
        Ok(self.source_decode::<SourcePayload>(&self.local_object(&logical(id))?)?.0.0)
    }
    pub(super) async fn checked_selection(&self,roots:&[DeclarationId],cancel:&crate::CancelSource)->Result<CheckedSelection,WorkflowError> {
        checkpoint(cancel)?;
        let math=self.runtime.shared.math();
        let request=Arc::new(super::SelectedRequest {
            roots:roots.to_vec(),physical:self.physical.key,
            providers:self.providers.iter().map(|(name,provider)|(name.clone(),provider.spec().key(),provider.configuration_key())).collect(),
            validation:Arc::as_ptr(self.runtime.sessions()) as usize,registry:Arc::as_ptr(self.runtime.registry()) as usize,
        });
        if let Some(candidates)=math.modeling_cache.selected(&request) {
            for admission in candidates.iter().rev().take(8) {
                checkpoint(cancel)?;
                if admission.dependencies.matches(&self.revision.admitted) {
                    return Ok(CheckedSelection {admission:admission.clone(),_premises_metadata:None});
                }
            }
        }
        let generation=math.modeling_cache.generation();
        let (index,physical,documents,leases,dependencies)=self.select_local(roots,cancel)?;
        let dependencies=Arc::new(dependencies);
        let extent=dependencies.retained_bytes().checked_add(request.retained_bytes()).and_then(|n|n.checked_add(index.versions.iter().map(|(k,v)|k.capacity()+v.capacity()+128).sum::<usize>())).and_then(|n|n.checked_add(size_of::<super::SelectedAdmission>()+512)).ok_or_else(||contract("local selected metadata extent"))?;
        let metadata=math.reserve("modeling:local-selected-admission",extent)?;
        let rows=index.rows.into_values().collect();let suppliers=index.record_sources;let versions=Arc::new(index.versions);
        let inputs=super::compiler_context(&self.physical,&self.providers);let lineage=self.compiler_lineage()?;
        let stamp=crate::math::modeling::LaneStamp::new(self.revision.generation(),self.revision.revision_id());
        let physical_key=self.physical.key;let service=math.clone();let flight_metadata=metadata.clone();
        let key=(request.clone(),dependencies.clone());let population=crate::math::PopulationScope::capture();
        let operation=math.selected_flights.load_owned(key,move |control|population.run(async move {
            let worker_service=service.clone();
            let admitted=service.job(1,0,control,move |flag| {
                let _sources=leases;let _metadata=flight_metadata;
                if flag.load(std::sync::atomic::Ordering::Acquire){return Err(crate::math::MathRuntimeError::Cancelled);}
                let workspace=worker_service.compiler_lane(lineage,inputs)?;
                worker_service.modeling_revision_at(&workspace,rows,physical,Arc::new(documents),&physical_key,stamp,flag,&suppliers)
            }).await?;Ok(Arc::new(admitted))
        }));
        tokio::pin!(operation);
        let revision=tokio::select! {
            result=&mut operation=>result.map_err(|error|match error {
                pse_columnar::flight::FlightError::Load(error)=>crate::math::MathRuntimeError::Shared(error),
                pse_columnar::flight::FlightError::Capacity=>crate::math::MathRuntimeError::Limit("selected admission flights"),
                pse_columnar::flight::FlightError::Retiring=>crate::math::MathRuntimeError::Retiring,
                pse_columnar::flight::FlightError::Panicked=>crate::math::MathRuntimeError::Panic("selected admission task panic".into()),
            })?,
            ()=cancel.cancelled()=>return Err(crate::math::MathRuntimeError::Cancelled.into()),
        };
        checkpoint(cancel)?;
        let admission=Arc::new(super::SelectedAdmission {request,revision:revision.as_ref().clone(),dependencies,versions,metadata,_validation_owner:self.runtime.model.clone()});
        math.modeling_cache.retain_selected(generation,admission.clone());
        Ok(CheckedSelection {admission,_premises_metadata:None})
    }
    fn compiler_lineage(&self)->Result<crate::math::modeling::CompilerLineage,WorkflowError> {
        Ok(crate::math::modeling::CompilerLineage::new(self.revision.lineage_id(),pse_operations::generated::surreal::INTERPRETATION,self.runtime.sessions(),self.runtime.registry().clone(),self.runtime.model.clone())?)
    }
    pub(super) fn pure_workspace(&self)->Result<crate::math::Workspace,WorkflowError> {
        Ok(self.runtime.shared.math().compiler_lane(self.compiler_lineage()?,super::compiler_context(&self.physical,&self.providers))?)
    }
    pub(in crate::workflow) async fn selected_revision(&self,root:DeclarationId,cancel:&crate::CancelSource)->Result<crate::math::modeling::ModelingRevision,WorkflowError> {
        Ok(self.checked_selection(&[root],cancel).await?.admission.revision.clone())
    }
    pub(in crate::workflow) async fn selected_revision_roots(&self,roots:&[DeclarationId],cancel:&crate::CancelSource)->Result<crate::math::modeling::ModelingRevision,WorkflowError> {
        Ok(self.checked_selection(roots,cancel).await?.admission.revision.clone())
    }
    pub(in crate::workflow) async fn all_revision(&self)->Result<crate::math::modeling::ModelingRevision,WorkflowError> {
        let parent=self.delta_parent(&crate::CancelSource::new()).await?;
        let roots=parent.rows.iter().map(|row|row.declaration_id).collect::<Vec<_>>();
        self.selected_revision_roots(&roots,&crate::CancelSource::new()).await
    }
    fn insert_local(&self,index:&mut SourceIndex,objects:Vec<Arc<LocalSourceObject>>,leases:&mut Vec<Arc<pse_columnar::AllocationLease>>,cancel:&crate::CancelSource)->Result<(),WorkflowError> {
        for object in objects {
            checkpoint(cancel)?;
            if index.versions.contains_key(&object.logical){continue;}
            if !object.kind.starts_with("modeling:") || object.kind==CONTEXT {return Err(contract("local lexical scope contains foreign object kind"));}
            let (source,lease)=self.source_decode::<SourcePayload>(&object)?;
            if logical(source.0.declaration_id)!=object.logical || kind(&source.0)!=object.kind || scope(source.0.parent_id)!=object.scope || source.0.name!=object.name {return Err(contract("owned source metadata differs from generated declaration"));}
            index.insert(source)?;index.versions.insert(object.logical.clone(),object.version.clone());leases.push(lease);
        }
        Ok(())
    }
    fn select_local(&self,roots:&[DeclarationId],cancel:&crate::CancelSource)->Result<(SourceIndex,PhysicalScope,pse_modeling::document::DocumentInventory,Vec<Arc<pse_columnar::AllocationLease>>,LocalSourcePremises),WorkflowError> {
        let source=&self.revision.admitted;let mut premises=LocalSourcePremises::default();
        let physical_object=premises.logical(source,PHYSICAL_SCOPE_LOGICAL).ok_or_else(||contract("owned physical scope absent"))?;
        let (mut physical,lease)=self.source_decode::<PhysicalScope>(&physical_object)?;
        let mut leases=vec![lease];let mut index=SourceIndex::default();
        index.versions.insert(physical_object.logical.clone(),physical_object.version.clone());
        index.requests.extend(roots.iter().copied().map(Request::Logical));let mut suppliers=BTreeSet::new();
        loop {
            checkpoint(cancel)?;let prior_rows=index.rows.len();
            let mut requests=std::mem::take(&mut index.requests).into_iter().collect::<Vec<_>>();
            requests.sort_by_key(|request|match request {Request::Members(_)=>0,Request::Logical(_)=>1,Request::Imports(_)=>2,Request::Name(_,_)=>3});
            let requested_scopes=requests.iter().filter_map(|request|match request {Request::Members(owner)=>Some(*owner),_=>None}).collect::<BTreeSet<_>>();
            for request in requests {
                let objects=match request {
                    Request::Logical(id) if !index.rows.contains_key(&id) && !index.record_sources.contains_key(&id)=>{
                        if let Some(object)=premises.logical(source,&logical(id)){vec![object]}else{
                            let object=premises.logical(source,&format!("record:{id}")).ok_or_else(||contract("selected modeling declaration, record or lexical ancestor absent"))?;
                            let (supplier,lease)=self.source_decode::<DeclarationId>(&object)?;leases.push(lease);
                            index.versions.insert(object.logical.clone(),object.version.clone());index.record_sources.insert(id,supplier);index.requests.insert(Request::Logical(supplier));Vec::new()
                        }
                    }
                    Request::Name(parent,name) if index.needs_name(parent,&name)=>{
                        let objects=premises.inventory(source,LocalInventory::Name(scope(parent),name.clone()));index.names.insert((parent,name));objects
                    }
                    Request::Members(owner)=>{
                        let objects=premises.inventory(source,LocalInventory::Scope(logical(owner)));index.scopes.insert(owner);index.imports.insert(owner);objects
                    }
                    Request::Imports(owner) if index.scopes.contains(&owner) || requested_scopes.contains(&owner)=>Vec::new(),
                    Request::Imports(owner) if !index.rows.contains_key(&owner)=>{index.requests.insert(Request::Logical(owner));index.requests.insert(Request::Imports(owner));Vec::new()},
                    Request::Imports(owner)=>{
                        let mut objects=premises.inventory(source,LocalInventory::Kind(Some(logical(owner)),"modeling:import".into()));
                        if index.rows[&owner].value.kind==Kind::Package {objects.extend(premises.inventory(source,LocalInventory::Kind(Some(logical(owner)),ENGINEERING_RULE.into())));}
                        index.imports.insert(owner);objects
                    }
                    _=>Vec::new(),
                };
                self.insert_local(&mut index,objects,&mut leases,cancel)?;
            }
            let new_rows=std::mem::take(&mut index.new_rows);let mut supplier_demands=Vec::new();
            for id in new_rows {
                checkpoint(cancel)?;let row=index.rows[&id].clone();
                if index.needs_name(row.parent_id,&row.name){index.requests.insert(Request::Name(row.parent_id,row.name.clone()));}
                if selected_source::requires_members(&row) && !index.scopes.contains(&id){index.requests.insert(Request::Members(id));}
                else if selected_source::owns_namespace(&row) && !index.imports.contains(&id) && !index.scopes.contains(&id){index.requests.insert(Request::Imports(id));}
                if row.value.import.is_some() && index.needs_name(None,&row.name){index.requests.insert(Request::Name(None,row.name.clone()));}
                let parsed=parsed_fields(&self.runtime,&row,index.fields.get(&id))?;
                let references=selected_source::declaration_references_parsed(&row,&parsed).map_err(super::super::modeling_error)?;
                let lease=self.runtime.shared.math().reserve("modeling:local-reference-paths",references.owned_bytes())?;
                index.parsed_fields.insert(id,parsed);index.pending_references.insert(id,references);leases.push(lease);
                if selected_source::requires_suppliers(&row) && suppliers.insert(id) {
                    let kinds:&[&str]=match row.value.kind {Kind::IdentifierScheme=>&["modeling:attribute"],Kind::EntityKind=>&["modeling:dataset","modeling:entity","modeling:entity_kind"],_=>&["modeling:dataset"]};
                    for kind in kinds {supplier_demands.push(LocalInventory::References(scope(row.parent_id),row.name.clone(),(*kind).into()));}
                }
            }
            for query in supplier_demands {let objects=premises.inventory(source,query);self.insert_local(&mut index,objects,&mut leases,cancel)?;}
            for (owner,references) in std::mem::take(&mut index.pending_references){checkpoint(cancel)?;index.pending(owner,references);}
            let mut requests=std::mem::take(&mut index.requests);
            requests.retain(|request|match request {Request::Logical(id)=>!index.rows.contains_key(id)&&!index.record_sources.contains_key(id),Request::Name(parent,name)=>index.needs_name(*parent,name),Request::Imports(id)=>!index.imports.contains(id)&&!index.scopes.contains(id),Request::Members(id)=>!index.scopes.contains(id)});
            index.requests=requests;
            if index.requests.is_empty() && index.rows.len()==prior_rows {break;}
        }
        if roots.iter().any(|root|!index.rows.contains_key(root)&&!index.record_sources.contains_key(root)){return Err(contract("selected modeling root absent"));}
        let document_ids=index.rows.values().map(|row|row.document_id).collect::<BTreeSet<_>>();
        let mut documents=pse_modeling::document::DocumentInventory {field_spans:index.fields.clone(),parsed_fields:index.parsed_fields.clone(),..Default::default()};
        let mut metadata_by_document=BTreeMap::new();
        for id in document_ids {
            if physical.documents.is_some() && let Some(object)=premises.logical(source,&format!("physical_scope_document:{id}")) {
                let (entry,lease)=self.source_decode::<PhysicalScope>(&object)?;
                if entry.package!=physical.package || entry.documents!=Some(BTreeSet::from([id])) {return Err(contract("owned document physical scope differs"));}
                if let Some(visible)=&mut physical.documents {visible.insert(id);}
                index.versions.insert(object.logical.clone(),object.version.clone());leases.push(lease);
            }
            if let Some(object)=premises.logical(source,&format!("document:{id}")) {
                let (row,lease)=self.source_decode::<pse_model::generated::authored::documents::Row>(&object)?;
                documents.packages.insert(id,row.package_id.as_id());metadata_by_document.insert(id,row);
                index.versions.insert(object.logical.clone(),object.version.clone());leases.push(lease);
            }
        }
        let mut data_documents=BTreeMap::new();
        for row in index.rows.values() {
            if let Some(path)=row.value.dataset.as_ref().and_then(|dataset|dataset.document.as_ref()) {
                let owner=metadata_by_document.get(&row.document_id).ok_or_else(||contract("selected dataset declaring document metadata absent"))?;
                data_documents.insert(pse_ids::named_id(owner.package_id.as_id(),path),path.clone());
            }
        }
        for (id,path) in data_documents {
            checkpoint(cancel)?;let object=premises.logical(source,&format!("data:{id}")).ok_or_else(||contract("selected data document absent"))?;
            let (document,lease)=pse_admission::authoring_driver::document::decode_selected_data(id,path,object.payload.clone().into(),Default::default(),&self.runtime.shared.pool(),&cancel.token())?;
            documents.documents.insert(id,document);index.versions.insert(object.logical.clone(),object.version.clone());leases.push(lease);
        }
        Ok((index,physical,documents,leases,premises))
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "namespace inventory fixtures require valid authored declarations and their expected identities"
    )]
    use super::*;
    use pse_relations::columnar::RelationRow;
    fn parse(source: &str) -> Vec<Declaration> {
        pse_authoring::language::parse(
            source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap()
    }

    #[test]
    fn complete_namespace_inventory_resolves_nearer_names_without_leaf_or_import_probes() {
        let rows = parse(
            "package p {param x:Scalar=9; def D {param x:Scalar=2; var y:Scalar; eq e:y==x;} }",
        );
        let root = rows
            .iter()
            .find(|row| row.name == "D")
            .unwrap()
            .declaration_id;
        let equation = rows
            .iter()
            .find(|row| row.name == "e")
            .unwrap()
            .declaration_id;
        let inner = rows
            .iter()
            .find(|row| row.name == "x" && row.parent_id == Some(root))
            .unwrap()
            .declaration_id;
        let mut index = SourceIndex::default();
        for row in rows {
            index.insert((row, None)).unwrap();
        }
        index.scopes.insert(root);
        index.requests.clear();
        assert_eq!(
            index.resolve_name(equation, "x"),
            Lookup::Found(vec![inner])
        );
        assert_eq!(index.member_candidates(root, "missing"), Lookup::Absent);
        assert_eq!(index.imported(root, "missing"), Lookup::Absent);
        assert!(!index.needs_name(Some(root), "missing"));
        assert!(!index.needs_name(Some(root), "x"));
        assert!(
            index.requests.is_empty(),
            "complete inventories and grammar leaves need no negative re-probe"
        );
    }

    #[tokio::test]
    async fn full_source_batches_preserve_order_nullable_data_scopes_and_owned_leases() {
        use pse_model::generated::{
            enums::{IdPolicy, PackageKind},
            identities::{FitId, PackageId},
        };
        use pse_operations::canonical_staging::{SELECTED_OBJECT_BATCH, SOURCE_BLOCK_BYTES};
        use pse_relations::generated::authored::{documents, fit_cases, packages};
        let runtime = crate::workflow::tests::runtime();
        let rows = parse("package p {def Root {var x:Scalar; eq e:x==1;}}");
        let mut package = runtime
            .modeling_package(rows.clone(), crate::workflow::tests::physical())
            .await
            .unwrap();
        let data_bytes = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../packages/reference/data/poling2000/data/water_antoine.parquet"
        ));
        let count = SELECTED_OBJECT_BATCH + 3;
        let mut edits = Vec::new();
        let physical = PhysicalScope {
            package: Some("physical".into()),
            documents: Some(BTreeSet::new()),
        };
        edits.push(edit(
            PHYSICAL_SCOPE_LOGICAL.into(),
            "context".into(),
            "physical_scope".into(),
            "physical_scope".into(),
            payload(&physical).unwrap(),
            Vec::new(),
        ));
        let mut expected_documents = BTreeMap::new();
        let mut expected_packages = BTreeMap::new();
        let mut expected_fits = BTreeMap::new();
        let mut visible = BTreeSet::new();
        for index in 0..count {
            let mut bytes = [37; 16];
            bytes[..8].copy_from_slice(&(index as u64).to_le_bytes());
            let document_id = SemanticId::from_bytes(bytes);
            let package_id = PackageId::from_id(document_id);
            let text = index == 1;
            let row = documents::Row {
                document_id,
                package_id,
                path: if text {
                    "source.pse".into()
                } else {
                    format!("data/{index}.parquet")
                },
                // Cross a payload-block boundary on a nullable-data document.
                source_text: text.then(|| " ".repeat(SOURCE_BLOCK_BYTES + 17)),
                content: None,
            };
            let logical = format!("document:{document_id}");
            edits.push(edit(
                logical.clone(),
                "documents".into(),
                document_id.to_string(),
                "document".into(),
                payload(&row).unwrap(),
                Vec::new(),
            ));
            let mut expected = row.clone();
            if !text {
                edits.push(edit(
                    format!("data:{document_id}"),
                    "data".into(),
                    document_id.to_string(),
                    "data".into(),
                    data_bytes.to_vec(),
                    Vec::new(),
                ));
                expected.content = Some(data_bytes.to_vec().into());
            }
            expected_documents.insert(logical, expected);
            let row = packages::Row {
                package_id,
                name: format!("p{index}"),
                version: "1.0.0".into(),
                kind: PackageKind::Model,
                id_policy: IdPolicy::Named,
                dependencies: Vec::new(),
                content_hash: pse_ids::encoding_checksum(&bytes).content_hash(),
                doc: String::new(),
            };
            let logical = format!("package:{package_id}");
            edits.push(edit(
                logical.clone(),
                "packages".into(),
                package_id.to_string(),
                "package".into(),
                payload(&row).unwrap(),
                Vec::new(),
            ));
            expected_packages.insert(logical, row);
            let row = fit_cases::Row {
                fit_id: FitId::from_id(document_id),
                parameters: Vec::new(),
                experiments: Vec::new(),
                observations: Vec::new(),
            };
            let logical = format!("fit:{}", row.fit_id);
            edits.push(edit(
                logical.clone(),
                "fits".into(),
                row.fit_id.to_string(),
                "fit".into(),
                payload(&row).unwrap(),
                Vec::new(),
            ));
            expected_fits.insert(logical, row);
            visible.insert(document_id);
            let entry = PhysicalScope {
                package: physical.package.clone(),
                documents: Some(BTreeSet::from([document_id])),
            };
            edits.push(edit(
                format!("physical_scope_document:{document_id}"),
                "physical_name_scopes".into(),
                document_id.to_string(),
                "physical_scope_document".into(),
                payload(&entry).unwrap(),
                Vec::new(),
            ));
        }
        let store = runtime.canonical_store()?;
        let revision = store
            .edit(
                &package.revision.canonical.problem,
                Some(&package.revision.canonical.key),
                "full-source-batch-fixture",
                &edits,
            )
            .await
            .unwrap();
        package.revision.canonical = revision.clone();
        let mut read = SelectedRead::new(
            store
                .protect(revision, std::time::Duration::from_secs(60))
                .await
                .unwrap(),
        );
        let document_order = store
            .resolve_kind_scope(&mut read, None, "document")
            .await
            .unwrap();
        let package_order = store
            .resolve_kind_scope(&mut read, None, "package")
            .await
            .unwrap();
        let fit_order = store
            .resolve_kind_scope(&mut read, None, "fit")
            .await
            .unwrap();
        read.finish().await.unwrap();
        let baseline = runtime.shared.pool().reserved();
        let (acquired, physical, inventory, sources, owners) = package.full_source().await.unwrap();
        assert_eq!(acquired.len(), rows.len());
        assert!(rows.iter().all(|row| {
            acquired
                .iter()
                .any(|actual| actual.declaration_id == row.declaration_id)
        }));
        assert_eq!(physical.documents, Some(visible));
        assert_eq!(inventory.packages.len(), count);
        assert_eq!(inventory.documents.len(), count - 1);
        assert_eq!(
            documents::Row::rows(&sources[&documents::RELATION_ID]).unwrap(),
            document_order
                .iter()
                .map(|member| expected_documents[&member.logical].clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            packages::Row::rows(&sources[&packages::RELATION_ID]).unwrap(),
            package_order
                .iter()
                .map(|member| expected_packages[&member.logical].clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            fit_cases::Row::rows(&sources[&fit_cases::RELATION_ID]).unwrap(),
            fit_order
                .iter()
                .map(|member| expected_fits[&member.logical].clone())
                .collect::<Vec<_>>()
        );
        assert!(owners.len() > 4 * count);
        assert!(runtime.shared.pool().reserved() > baseline);
        drop((acquired, physical, inventory, sources, owners));
        assert_eq!(runtime.shared.pool().reserved(), baseline);

        // Refuse a malformed payload in the final document batch; no partial
        // export or accumulated reservation escapes the failed acquisition.
        let last = document_order.last().unwrap();
        let bad = edit(
            last.logical.clone(),
            last.scope.clone(),
            last.name.clone(),
            "document".into(),
            b"{".to_vec(),
            Vec::new(),
        );
        package.revision.canonical = store
            .edit(
                &package.revision.canonical.problem,
                Some(&package.revision.canonical.key),
                "full-source-late-refusal",
                &[bad],
            )
            .await
            .unwrap();
        assert!(
            package
                .full_source()
                .await
                .unwrap_err()
                .to_string()
                .contains("canonical source decoding")
        );
        assert_eq!(runtime.shared.pool().reserved(), baseline);
    }

    #[tokio::test]
    async fn selected_declaration_windows_preserve_sparse_inventory_order_and_cancellation() {
        let runtime = crate::workflow::tests::runtime();
        let fixtures = (0..145)
            .map(|index| format!("test f{index} {{expect 1==1 tolerance 1e-8;}}"))
            .collect::<String>();
        let rows = parse(&format!(
            "package p {{{fixtures} def Unrelated {{var bad:NotPhysical;}}}}"
        ));
        let selected = rows
            .iter()
            .filter(|row| row.value.kind == Kind::Test && row.name != "f70")
            .map(|row| row.declaration_id)
            .collect::<BTreeSet<_>>();
        assert!(selected.len() > 2 * pse_operations::canonical_staging::SELECTED_OBJECT_BATCH);
        let package = runtime
            .modeling_package(rows, crate::workflow::tests::physical())
            .await
            .unwrap();
        let cancel = crate::CancelSource::new();
        let full = package.declaration_inventory().await.unwrap();
        let expected = full
            .rows
            .iter()
            .filter(|row| selected.contains(&row.declaration_id))
            .map(|row| (row.declaration_id, row.name.clone()))
            .collect::<Vec<_>>();
        let acquired = package
            .selected_declarations(&selected, &cancel)
            .await
            .unwrap();
        assert_eq!(
            acquired
                .rows
                .iter()
                .map(|row| (row.declaration_id, row.name.clone()))
                .collect::<Vec<_>>(),
            expected
        );
        let sparse = BTreeSet::from([
            expected[0].0,
            expected[expected.len() - 1].0,
            DeclarationId::from_bytes([7; 16]),
        ]);
        let sparse_rows = package
            .selected_declarations(&sparse, &cancel)
            .await
            .unwrap();
        assert_eq!(sparse_rows.rows.len(), 2);
        assert_eq!(sparse_rows.rows[0].declaration_id, expected[0].0);
        assert_eq!(
            sparse_rows.rows[1].declaration_id,
            expected[expected.len() - 1].0
        );
        cancel.cancel();
        assert!(matches!(
            package.selected_declarations(&selected, &cancel).await,
            Err(WorkflowError::Math(
                crate::math::MathRuntimeError::Cancelled
            ))
        ));
    }

    #[tokio::test]
    async fn grouped_scalar_frontier_preserves_closure_replay_and_numerical_results() {
        let runtime = crate::workflow::tests::runtime();
        let parameters = (0..128)
            .map(|index| format!("param c{index}:Scalar=1;"))
            .collect::<String>();
        let expression = (0..128)
            .map(|index| format!("c{index}"))
            .collect::<Vec<_>>()
            .join("+");
        let rows = parse(&format!(
            "package p {{def Root {{{parameters}var x:Scalar; eq e:x=={expression};}} def Unrelated {{var bad:NotPhysical;}} }}"
        ));
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(rows, crate::workflow::tests::physical())
            .await
            .unwrap();
        let cancel = crate::CancelSource::new();
        let selected = package.selected_source(root, &cancel).await.unwrap();
        assert_eq!(selected.rows.len(), 132);
        assert!(
            !selected
                .rows
                .iter()
                .any(|row| row.name == "Unrelated" || row.name == "bad")
        );
        let selected_versions = selected.versions.clone();
        runtime
            .canonical_store()?
            .release(selected.read.selection())
            .await
            .unwrap();
        drop(selected);
        let mut previous = None;
        for _ in 0..2 {
            let reopened = runtime
                .modeling_revision(
                    package.canonical_revision().clone(),
                    crate::workflow::tests::physical(),
                    BTreeMap::new(),
                )
                .await
                .unwrap();
            let model = reopened
                .prepare(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Default::default(),
                    Default::default(),
                    &cancel,
                )
                .await
                .unwrap();
            assert_eq!(model.consumed_source_versions(), &selected_versions);
            let keys = model
                .compiled()
                .admitted
                .bodies
                .keys()
                .copied()
                .collect::<Vec<_>>();
            if let Some(previous) = &previous {
                assert_eq!(&keys, previous);
            }
            previous = Some(keys);
            let values = pse_math::binding::CaseValues {
                scalars: model
                    .compiled()
                    .model
                    .symbols
                    .values()
                    .map(|symbol| {
                        (
                            symbol.id,
                            if symbol.lineage.path.ends_with(".x") {
                                128.0
                            } else {
                                1.0
                            },
                        )
                    })
                    .collect(),
            };
            let bound = reopened
                .bound_case(
                    &model,
                    values.clone(),
                    &BTreeMap::new(),
                    pse_kernels::DerivativeOrder::Second,
                    crate::workflow::tests::compiler_profile(),
                    &cancel,
                )
                .await
                .unwrap();
            let executable = runtime.shared.math().assemble(bound.case).await.unwrap();
            let mut changed = values.clone();
            let x = model
                .compiled()
                .model
                .symbols
                .values()
                .find(|symbol| symbol.lineage.path.ends_with(".x"))
                .unwrap()
                .id;
            changed.scalars.insert(x, 129.0);
            let (initial, changed) = runtime
                .shared
                .math()
                .with_worker(executable, BTreeMap::new(), &cancel, move |worker| {
                    Ok((worker.constraints(&values)?, worker.constraints(&changed)?))
                })
                .await
                .unwrap();
            assert_eq!(initial, [0.0]);
            assert_eq!(changed, [1.0]);
        }
    }

    #[tokio::test]
    async fn grouped_source_precharges_resources_and_preserves_cancellation() {
        let runtime = crate::workflow::tests::runtime();
        let rows = parse("package p {def Root {var x:Scalar; eq e:x==1;} }");
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(rows, crate::workflow::tests::physical())
            .await
            .unwrap();
        let cancel = crate::CancelSource::new();
        let selected = package.selected_source(root, &cancel).await.unwrap();
        let versions = vec![selected.versions[&logical(root)].clone()];
        // Measure live and transient owners exactly. Optional retained products
        // may yield under pressure; the selected source's escaped owners remain.
        runtime.shared.math().clear_program_cache();
        let before = runtime.shared.pool().reserved();
        let available = runtime.shared.budget().memory_limit_bytes.get() - before;
        let hold = runtime
            .shared
            .math()
            .reserve("test:source-frontier-pressure", available - (8 << 20))
            .unwrap();
        let error = match package.selected_source(root, &cancel).await {
            Ok(_) => panic!("low-budget selected source unexpectedly admitted"),
            Err(error) => error,
        };
        assert_eq!(
            error.boundary_diagnostic().class,
            pse_model::diagnostic::BoundaryClass::ResourceLimit,
            "{error}"
        );
        assert!(
            error
                .to_string()
                .contains("modeling:logical-frontier-metadata"),
            "the entrypoint must refuse at the precharged logical RPC, before payload/header acquisition: {error}"
        );
        assert_eq!(
            runtime.shared.pool().reserved(),
            before + hold.size(),
            "entrypoint refusal releases physical decode and metadata allowances"
        );
        let error = selected_objects(&runtime, &selected.read, &versions, &cancel)
            .await
            .unwrap_err();
        assert_eq!(
            error.boundary_diagnostic().class,
            pse_model::diagnostic::BoundaryClass::ResourceLimit,
            "{error}"
        );
        let mut pending = selected.read.clone();
        let mut cursors = vec![
            runtime
                .canonical_store()?
                .scope_pages(&mut pending, &logical(root))
                .unwrap(),
        ];
        let mut index = SourceIndex::default();
        let mut leases = Vec::new();
        let error = package
            .insert_inventories(&mut index, &mut pending, &mut cursors, &mut leases, &cancel)
            .await
            .unwrap_err();
        assert_eq!(
            error.boundary_diagnostic().class,
            pse_model::diagnostic::BoundaryClass::ResourceLimit,
            "{error}"
        );
        assert!(index.rows.is_empty() && leases.is_empty());
        assert!(
            pending.dependency_bytes().is_err(),
            "an unacquired inventory cannot publish partial meaning"
        );
        drop(hold);
        assert_eq!(
            runtime.shared.pool().reserved(),
            before,
            "failed acquisitions release their transient reservations"
        );
        cancel.cancel();
        let error = selected_objects(&runtime, &selected.read, &versions, &cancel)
            .await
            .unwrap_err();
        assert_eq!(
            error.boundary_diagnostic().class,
            pse_model::diagnostic::BoundaryClass::Cancelled,
            "{error}"
        );
        let error = package
            .insert_inventories(&mut index, &mut pending, &mut cursors, &mut leases, &cancel)
            .await
            .unwrap_err();
        assert_eq!(
            error.boundary_diagnostic().class,
            pse_model::diagnostic::BoundaryClass::Cancelled,
            "{error}"
        );
        assert!(index.rows.is_empty() && leases.is_empty());
        assert_eq!(runtime.shared.pool().reserved(), before);
        runtime
            .canonical_store()?
            .release(selected.read.selection())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn canonical_selected_source_acquires_package_engineering_rules_without_unrelated_growth()
    {
        async fn description(
            runtime: &Runtime,
            read: &SelectedRead,
            request: &[u8],
        ) -> pse_operations::canonical_selection::ProductCandidate {
            let store = runtime.canonical_store()?;
            let producer = "engineering-rule-frontier-fixture";
            store
                .publish_product(
                    read,
                    pse_model::generated::runtime::canonical_products::Row {
                        key: "frontier-description".into(),
                        problem: read.selection().revision().problem.clone(),
                        revision: read.selection().revision().key.clone(),
                        request: request.to_vec().into(),
                        payload: b"dependency-reuse-control".to_vec().into(),
                        dependencies: Vec::new().into(),
                        producer: producer.into(),
                        interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
                    },
                )
                .await
                .unwrap();
            store
                .product_candidate(read.selection(), request, producer, "")
                .await
                .unwrap()
                .unwrap()
        }
        let runtime = crate::workflow::tests::runtime();
        let source = "package p {use policy @\"1.0.0\"; def D {var T:Temperature; eq temperature:T==280{K};}} package policy {entity kind source provenance {attribute title:Text;} enum role {published} entity source maintainer {title=\"engineering policy\"} constant temperature:DeltaTemperature=0.1{K} provenance(maintainer,role.published); annotation engineering_rule policy.temperature;}";
        let rows = parse(source);
        let root = rows
            .iter()
            .find(|row| row.name == "D")
            .unwrap()
            .declaration_id;
        let rule = rows
            .iter()
            .find(|row| row.value.constant.is_some())
            .unwrap()
            .declaration_id;
        let marker = rows
            .iter()
            .find(|row| row.value.annotation.is_some())
            .unwrap()
            .declaration_id;
        let policy = rows
            .iter()
            .find(|row| row.name == "policy" && row.value.kind == Kind::Package)
            .unwrap()
            .declaration_id;
        assert_eq!(
            kind(
                rows.iter()
                    .find(|row| row.declaration_id == marker)
                    .unwrap()
            ),
            ENGINEERING_RULE
        );
        let cancel = crate::CancelSource::new();
        // Edit the authoritative relation while keeping every other authored row
        // (including source provenance) exact, so marker membership is the premise.
        let unmarked_rows = rows
            .iter()
            .filter(|row| row.declaration_id != marker)
            .cloned()
            .collect::<Vec<_>>();
        let unmarked = runtime
            .modeling_package(unmarked_rows.clone(), crate::workflow::tests::physical())
            .await
            .unwrap();
        let selected = unmarked.selected_source(root, &cancel).await.unwrap();
        assert!(!selected.rows.iter().any(|row| row.declaration_id == marker));
        let unmarked_candidate =
            description(&runtime, &selected.read, b"frontier-without-rule").await;
        let _unmarked_lease = runtime
            .shared
            .math()
            .reserve(
                "test:engineering-rule-unmarked-product",
                unmarked_candidate.retained_bytes(),
            )
            .unwrap();
        assert!(
            runtime
                .canonical_store()?
                .qualify_product(&mut selected.read.clone(), &unmarked_candidate)
                .await
                .unwrap()
                .is_some()
        );
        runtime
            .canonical_store()?
            .release(selected.read.selection())
            .await
            .unwrap();
        drop(selected);
        let package = unmarked.with_declarations(rows.clone()).await.unwrap();
        let selected = package.selected_source(root, &cancel).await.unwrap();
        assert!(
            runtime
                .canonical_store()?
                .qualify_product(&mut selected.read.clone(), &unmarked_candidate)
                .await
                .unwrap()
                .is_none(),
            "adding a package rule invalidates the recorded empty marker inventory"
        );
        assert!(selected.rows.iter().any(|row| row.declaration_id == marker));
        assert!(selected.rows.iter().any(|row| row.declaration_id == rule));
        let extent = selected.rows.owned_bytes();
        let count = selected.rows.len();
        let dependency_bytes = selected.read.dependency_bytes().unwrap();
        let prepared = package
            .prepare(
                root,
                pse_modeling::specialize::root_instance(root),
                Default::default(),
                Default::default(),
                &cancel,
            )
            .await
            .unwrap();
        let rules = &prepared.compiled().model.engineering_rules;
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, rule.as_id());
        assert_eq!(rules[0].marker, marker);
        let pse_modeling::specialize::Value::Number { bits, quantity } = &rules[0].value else {
            panic!("shared rule retains its typed constant value")
        };
        assert_eq!(f64::from_bits(*bits), 0.1);
        assert_eq!(
            package
                .physical
                .quantities
                .quantity_type(*quantity)
                .unwrap()
                .key
                .scale_kind,
            pse_quantity::ScaleKind::Difference
        );
        let marked_candidate = description(&runtime, &selected.read, b"frontier-with-rule").await;
        let _marked_lease = runtime
            .shared
            .math()
            .reserve(
                "test:engineering-rule-marked-product",
                marked_candidate.retained_bytes(),
            )
            .unwrap();
        assert!(
            runtime
                .canonical_store()?
                .qualify_product(&mut selected.read.clone(), &marked_candidate)
                .await
                .unwrap()
                .is_some(),
            "the marked product qualifies against its own protected source"
        );
        let marked_versions = selected.versions.clone();
        runtime
            .canonical_store()?
            .release(selected.read.selection())
            .await
            .unwrap();
        drop(selected);
        // Unrelated parameters and ordinary package annotations stay outside the
        // selected closure, even when they cannot pass scientific admission.
        let extras = (0..64).map(|index| format!(
            "param ignored{index}:NotPhysical=1; annotation check ignored{index}(ignored{index}>0);"
        )).collect::<String>();
        let grown = source.strip_suffix('}').unwrap().to_owned() + extras.as_str() + "}";
        let grown_rows = parse(&grown);
        let revised = package.with_declarations(grown_rows.clone()).await.unwrap();
        let selected = revised.selected_source(root, &cancel).await.unwrap();
        assert_eq!(selected.rows.owned_bytes(), extent);
        assert_eq!(selected.rows.len(), count);
        assert_eq!(
            selected.read.dependency_bytes().unwrap(),
            dependency_bytes,
            "unrelated package members do not grow semantic marker/import premises"
        );
        assert!(
            !selected
                .rows
                .iter()
                .any(|row| row.name.starts_with("ignored"))
        );
        let checked = revised.checked_selection(&[root], &cancel).await.unwrap();
        runtime
            .canonical_store()?
            .release(checked.read.selection())
            .await
            .unwrap();
        let changed = selected
            .versions
            .iter()
            .filter(|(logical, version)| marked_versions.get(*logical) != Some(*version))
            .map(|(logical, _)| logical.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            changed,
            [logical(policy)],
            "parsed package extension changes its closing source location, preserving other selected versions"
        );
        assert!(
            runtime
                .canonical_store()?
                .qualify_product(&mut selected.read.clone(), &marked_candidate)
                .await
                .unwrap()
                .is_none(),
            "changed package source provenance invalidates exact reuse"
        );
        runtime
            .canonical_store()?
            .release(selected.read.selection())
            .await
            .unwrap();
        drop(selected);
        // A relation extension leaves all existing rows exact. Its unrelated members
        // must neither grow selected acquisition nor invalidate marker/import meaning.
        let existing = rows
            .iter()
            .map(|row| row.declaration_id)
            .collect::<BTreeSet<_>>();
        let additions = grown_rows
            .into_iter()
            .filter(|row| !existing.contains(&row.declaration_id))
            .collect::<Vec<_>>();
        let mut extended_rows = rows;
        extended_rows.extend(additions.iter().cloned());
        let extended = revised.with_declarations(extended_rows).await.unwrap();
        let selected = extended.selected_source(root, &cancel).await.unwrap();
        assert_eq!(selected.rows.owned_bytes(), extent);
        assert_eq!(selected.rows.len(), count);
        assert_eq!(selected.read.dependency_bytes().unwrap(), dependency_bytes);
        assert_eq!(
            selected.versions, marked_versions,
            "independent relation additions preserve exact selected object versions"
        );
        assert!(
            runtime
                .canonical_store()?
                .qualify_product(&mut selected.read.clone(), &marked_candidate)
                .await
                .unwrap()
                .is_some(),
            "unrelated relation growth preserves the actual reuse premises"
        );
        runtime
            .canonical_store()?
            .release(selected.read.selection())
            .await
            .unwrap();
        drop(selected);
        let mut removed_rows = unmarked_rows;
        removed_rows.extend(additions);
        let removed = extended.with_declarations(removed_rows).await.unwrap();
        let selected = removed.selected_source(root, &cancel).await.unwrap();
        assert!(
            runtime
                .canonical_store()?
                .qualify_product(&mut selected.read.clone(), &marked_candidate)
                .await
                .unwrap()
                .is_none(),
            "removing the package rule invalidates its recorded marker inventory"
        );
        assert!(
            runtime
                .canonical_store()?
                .qualify_product(&mut selected.read.clone(), &unmarked_candidate)
                .await
                .unwrap()
                .is_some(),
            "restoring the original unmarked meaning permits its original product"
        );
        let prepared = removed
            .prepare(
                root,
                pse_modeling::specialize::root_instance(root),
                Default::default(),
                Default::default(),
                &cancel,
            )
            .await
            .unwrap();
        assert!(prepared.compiled().model.engineering_rules.is_empty());
        runtime
            .canonical_store()?
            .release(selected.read.selection())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn canonical_source_revision_refuses_old_context_interpretation() {
        let runtime = crate::workflow::tests::runtime();
        let package = runtime
            .modeling_package(
                parse("package p {def D {var x:Scalar; eq e:x==2;}}"),
                crate::workflow::tests::physical(),
            )
            .await
            .unwrap();
        let current = package.canonical_revision().clone();
        assert_eq!(
            SourceRevision::open(&runtime, current.clone())
                .await
                .unwrap()
                .identity(),
            package.revision.identity()
        );
        // Stage a genuine old-kind object with the same logical ID and payload.
        // Only its source interpretation differs; no fixture bypass changes open.
        let old_operation = pse_operations::mint_id::<SemanticId>().to_string();
        let old = runtime
            .canonical_store()?
            .edit(
                &current.problem,
                Some(&current.key),
                &old_operation,
                &[edit(
                    CONTEXT_LOGICAL.into(),
                    "context".into(),
                    "modeling:context".into(),
                    "modeling:context".into(),
                    payload(&package.revision.identity()).unwrap(),
                    Vec::new(),
                )],
            )
            .await
            .unwrap();
        let error = SourceRevision::open(&runtime, old.clone())
            .await
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("canonical modeling context interpretation differs"),
            "{error}"
        );
        assert!(
            error.to_string().contains("restage the authored source"),
            "{error}"
        );
        let current_operation = pse_operations::mint_id::<SemanticId>().to_string();
        let restaged = runtime
            .canonical_store()?
            .edit(
                &current.problem,
                Some(&old.key),
                &current_operation,
                &[edit(
                    CONTEXT_LOGICAL.into(),
                    "context".into(),
                    CONTEXT.into(),
                    CONTEXT.into(),
                    payload(&package.revision.identity()).unwrap(),
                    Vec::new(),
                )],
            )
            .await
            .unwrap();
        assert_eq!(
            SourceRevision::open(&runtime, restaged)
                .await
                .unwrap()
                .identity(),
            package.revision.identity()
        );
    }

    #[tokio::test]
    async fn canonical_selected_source_precedes_admission_and_survives_unrelated_growth() {
        let runtime = crate::workflow::tests::runtime();
        let source = "package p {fn shared()->Scalar=3; fn law(x:Scalar)->Scalar=x*x; def D {var x:Scalar; eq e:law(x)==shared();} def Unrelated {var wrong:NotPhysical;} }";
        let rows = parse(source);
        let root = rows
            .iter()
            .find(|row| row.name == "D")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(rows, crate::workflow::tests::physical())
            .await
            .unwrap();
        let cancel = crate::CancelSource::new();
        let selected = package.selected_source(root, &cancel).await.unwrap();
        let extent = selected.rows.owned_bytes();
        let names = selected
            .rows
            .iter()
            .map(|row| row.name.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            names,
            ["p", "shared", "law", "D", "x", "e"].into_iter().collect()
        );
        let checked = package.checked_selection(&[root], &cancel).await.unwrap();
        let admitted = checked.admission.revision.clone();
        runtime
            .canonical_store()?
            .release(checked.read.selection())
            .await
            .unwrap();
        assert!(admitted.checked().entry("p.D").is_some());
        assert!(admitted.checked().entry("p.Unrelated").is_none());
        runtime
            .canonical_store()?
            .release(selected.read.selection())
            .await
            .unwrap();
        drop(selected);
        let preparation = package
            .prepare(
                root,
                pse_modeling::specialize::root_instance(root),
                Default::default(),
                Default::default(),
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(preparation.compiled().model.equations.len(), 1);
        let extras = (0..64)
            .map(|index| format!("def Extra{index} {{var bad:NotPhysical;}} "))
            .collect::<String>();
        let grown = source.strip_suffix('}').unwrap().to_owned() + extras.as_str() + "}";
        let revised = package.with_declarations(parse(&grown)).await.unwrap();
        let selected = revised.selected_source(root, &cancel).await.unwrap();
        assert_eq!(
            selected.rows.owned_bytes(),
            extent,
            "unrelated declarations never enter the typed compiler input"
        );
        assert_eq!(selected.rows.len(), 6);
        let checked = revised.checked_selection(&[root], &cancel).await.unwrap();
        runtime
            .canonical_store()?
            .release(checked.read.selection())
            .await
            .unwrap();
        runtime
            .canonical_store()?
            .release(selected.read.selection())
            .await
            .unwrap();
        let reopened = runtime
            .modeling_revision(
                revised.canonical_revision().clone(),
                crate::workflow::tests::physical(),
                BTreeMap::new(),
            )
            .await
            .unwrap();
        let prepared = reopened
            .prepare(
                root,
                pse_modeling::specialize::root_instance(root),
                Default::default(),
                Default::default(),
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(prepared.compiled().model.equations.len(), 1);
    }

    #[tokio::test]
    async fn canonical_selected_source_keeps_nearer_names_and_unused_formals() {
        let runtime = crate::workflow::tests::runtime();
        let rows = parse(
            "package p {param outer:Scalar=8; fn law(outer:Scalar)->Scalar=outer; def D {param outer:Scalar=2; var x:Scalar; eq e:law(x)==outer;} }",
        );
        let root = rows
            .iter()
            .find(|row| row.name == "D")
            .unwrap()
            .declaration_id;
        let inner = rows
            .iter()
            .find(|row| row.name == "outer" && row.parent_id == Some(root))
            .unwrap()
            .declaration_id;
        let outer = rows
            .iter()
            .find(|row| row.name == "outer" && row.parent_id != Some(root))
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(rows, crate::workflow::tests::physical())
            .await
            .unwrap();
        let selected = package
            .selected_source(root, &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(selected.rows.iter().any(|row| row.declaration_id == inner));
        assert!(!selected.rows.iter().any(|row| row.declaration_id == outer));
        let checked = package
            .checked_selection(&[root], &crate::CancelSource::new())
            .await
            .unwrap();
        let revision = checked.admission.revision.clone();
        runtime
            .canonical_store()?
            .release(checked.read.selection())
            .await
            .unwrap();
        assert_eq!(revision.checked().resolve(root, "outer"), Some(inner));
        runtime
            .canonical_store()?
            .release(selected.read.selection())
            .await
            .unwrap();
    }
    #[tokio::test]
    async fn canonical_synthetic_record_selects_its_admitted_supplier() {
        let runtime = crate::workflow::tests::runtime();
        let rows = parse(
            "package p {entity kind sample {key id:Integer;attribute value:Scalar;} entity kind source provenance {} entity source origin {} enum role {published} dataset readings:sample provenance(origin,role.published) {[7]=[2];[8]=[3];[9]=[4];} def D {var x:Scalar;eq e:x==2;} def Unrelated {var broken:MissingType;}}",
        );
        let root = rows
            .iter()
            .find(|row| row.name == "D")
            .unwrap()
            .declaration_id;
        let kind = rows
            .iter()
            .find(|row| row.name == "sample")
            .unwrap()
            .declaration_id;
        let supplier = rows
            .iter()
            .find(|row| row.name == "readings")
            .unwrap()
            .declaration_id;
        let record = pse_modeling::entity::keyed_identity(
            kind,
            &[pse_modeling::specialize::Value::Integer(7)],
        );
        let other_records = [8, 9].map(|key| {
            pse_modeling::entity::keyed_identity(
                kind,
                &[pse_modeling::specialize::Value::Integer(key)],
            )
        });
        let package = runtime
            .modeling_package(rows, crate::workflow::tests::physical())
            .await
            .unwrap();
        let selected = package
            .selected_sources(
                &[root, record, other_records[0], other_records[1]],
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert_eq!(selected.record_sources[&record], supplier);
        assert!(selected.versions.contains_key(&format!("record:{record}")));
        assert!(
            !selected
                .rows
                .iter()
                .any(|row| row.name == "Unrelated" || row.name == "broken")
        );
        for record in other_records {
            assert_eq!(selected.record_sources[&record], supplier);
            assert!(selected.versions.contains_key(&format!("record:{record}")));
        }
        assert_eq!(
            selected
                .rows
                .iter()
                .filter(|row| row.declaration_id == supplier)
                .count(),
            1,
            "shared suppliers are hydrated once across exact record demands"
        );
        let checked = package
            .checked_selection(
                &[root, record, other_records[0], other_records[1]],
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let admitted = checked.admission.revision.clone();
        runtime
            .canonical_store()?
            .release(checked.read.selection())
            .await
            .unwrap();
        for (record, expected) in other_records.into_iter().zip([3.0, 4.0]) {
            let value = admitted.checked().record(record).unwrap();
            assert_eq!(value.origin, supplier);
            assert!(
                matches!(value.values["value"], pse_modeling::specialize::Value::Number { bits, .. } if f64::from_bits(bits) == expected)
            );
        }
        assert_eq!(admitted.checked().record(record).unwrap().origin, supplier);
        assert!(
            matches!(admitted.checked().record(record).unwrap().values["value"], pse_modeling::specialize::Value::Number { bits, .. } if f64::from_bits(bits) == 2.0)
        );
        runtime
            .canonical_store()?
            .release(selected.read.selection())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn canonical_document_physical_visibility_and_inventory_owners_are_selected() {
        let runtime = crate::workflow::tests::runtime();
        let physical = crate::workflow::tests::physical();
        let rows = parse("package p {def D {var x:Scalar;eq e:x==2;}}");
        let root = rows
            .iter()
            .find(|row| row.name == "D")
            .unwrap()
            .declaration_id;
        let visible = std::iter::once(SemanticId::NIL)
            .chain((1..=64).map(|id| SemanticId::from_bytes([id; 16])))
            .collect::<BTreeSet<_>>();
        let revision = persist(
            &runtime,
            rows,
            &physical,
            PhysicalScope {
                package: None,
                documents: Some(visible),
            },
            &Default::default(),
            &Default::default(),
            &Default::default(),
            None,
            None,
        )
        .await
        .unwrap();
        let package = runtime
            .package_from_canonical(revision, physical, BTreeMap::new())
            .unwrap();
        let selected = package
            .selected_source(root, &crate::CancelSource::new())
            .await
            .unwrap();
        assert_eq!(
            selected.physical.documents,
            Some(BTreeSet::from([SemanticId::NIL]))
        );
        assert_eq!(
            selected
                .versions
                .keys()
                .filter(|key| key.starts_with("physical_scope_document:"))
                .count(),
            1
        );
        let checked = package
            .checked_selection(&[root], &crate::CancelSource::new())
            .await
            .unwrap();
        runtime
            .canonical_store()?
            .release(checked.read.selection())
            .await
            .unwrap();
        runtime
            .canonical_store()?
            .release(selected.read.selection())
            .await
            .unwrap();
        drop(selected);
        let baseline = runtime.shared.pool().reserved();
        let inventory = package.declarations().await.unwrap();
        assert!(runtime.shared.pool().reserved() > baseline);
        drop(package);
        assert!(
            runtime.shared.pool().reserved() > baseline,
            "escaped generated rows retain their decode grants"
        );
        drop(inventory);
        assert_eq!(runtime.shared.pool().reserved(), baseline);
    }
}
