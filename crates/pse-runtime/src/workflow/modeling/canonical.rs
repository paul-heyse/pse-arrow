// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Immutable source objects and protected lexical selection before shared admission.
use super::{
    Declaration, DeclarationId, ModelingPackage, PhysicalScope, Runtime, WorkflowError, contract,
};
use pse_authoring::{SourceSpan, dsl};
use pse_ids::{SemanticId, roles::SourceRevisionHash};
use pse_model::generated::runtime::canonical_revisions::Row as Revision;
use pse_model::{HeapUsage, generated::enums::ModelingDeclarationKind as Kind};
use pse_modeling::selected_source::{self, LexicalLookup, Lookup};
use pse_operations::{canonical::ObjectEdit, canonical_selection::SelectedRead};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
mod delta;
mod local;
pub(crate) use local::{AdmittedModelRevision, LocalSourcePremises};
use local::{LocalInventory, LocalSourceObject, SourceMember};
pub(crate) mod reload;

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
    pub(in crate::workflow) fn identity(&self) -> SourceRevisionHash {
        self.identity
    }
    pub(crate) fn revision_id(&self) -> &str {
        &self.revision_id
    }
    pub(crate) fn lineage_id(&self) -> &str {
        &self.lineage_id
    }
    pub(crate) fn generation(&self) -> i64 {
        self.generation
    }
    pub(crate) fn origin(&self) -> Option<&Revision> {
        self.origin.as_ref()
    }
    pub(super) fn require_origin(&self) -> Result<&Revision, WorkflowError> {
        self.origin().ok_or(WorkflowError::RecordingUnavailable)
    }
    pub(super) async fn open(
        runtime: &Runtime,
        canonical: Revision,
    ) -> Result<Self, WorkflowError> {
        let store = runtime.canonical_store()?;
        let protection = store
            .protect(canonical.clone(), std::time::Duration::from_secs(3600))
            .await?;
        let mut read = SelectedRead::new(protection);
        let result: Result<_, WorkflowError> = async {
            let identity = context(runtime, &mut read).await?;
            let cancel = crate::CancelSource::new();
            let mut after = String::new();
            let mut imported = Vec::new();
            let mut leases = Vec::new();
            loop {
                let page = store
                    .membership_page(read.selection(), None, &after)
                    .await?;
                let Some(last) = page.last() else { break };
                after = last.key.clone();
                for chunk in page.chunks(pse_operations::canonical_staging::SELECTED_OBJECT_BATCH) {
                    let versions = chunk
                        .iter()
                        .map(|member| member.version.clone())
                        .collect::<Vec<_>>();
                    for (member, object) in chunk
                        .iter()
                        .zip(selected_objects(runtime, &read, &versions, &cancel).await?)
                    {
                        if member.logical != object.logical || member.version != object.key {
                            return Err(contract("imported source membership differs"));
                        }
                        leases.push(object.lease.clone());
                        imported.push(ObjectEdit {
                            logical: member.logical.clone(),
                            scope: member.scope.clone(),
                            name: member.name.clone(),
                            version: Some(object.row),
                            references: Vec::new(),
                        });
                    }
                }
            }
            let mut index = SourceIndex {
                roots_complete: true,
                ..Default::default()
            };
            for object in &imported {
                if object.logical.starts_with("modeling:")
                    && object.logical != CONTEXT_LOGICAL
                    && object.logical != PHYSICAL_SCOPE_LOGICAL
                {
                    let row: SourcePayload = decode(
                        object
                            .version
                            .as_ref()
                            .ok_or_else(|| contract("imported version absent"))?
                            .payload
                            .as_slice(),
                    )?;
                    index.parsed_fields.insert(
                        row.0.declaration_id,
                        parsed_fields(runtime, &row.0, row.1.as_ref())?,
                    );
                    index.scopes.insert(row.0.declaration_id);
                    index.imports.insert(row.0.declaration_id);
                    index.insert(row)?;
                }
            }
            index.requests.clear();
            for object in &mut imported {
                if object.logical.starts_with("modeling:")
                    && object.logical != CONTEXT_LOGICAL
                    && object.logical != PHYSICAL_SCOPE_LOGICAL
                {
                    let (row, _): SourcePayload = decode(
                        object
                            .version
                            .as_ref()
                            .ok_or_else(|| contract("imported version absent"))?
                            .payload
                            .as_slice(),
                    )?;
                    object.references = index
                        .references(&row)?
                        .iter()
                        .map(|id| {
                            let target = &index.rows[id];
                            (scope(target.parent_id), target.name.clone())
                        })
                        .collect();
                } else if object.logical.starts_with("record:") {
                    let supplier: DeclarationId = decode(
                        object
                            .version
                            .as_ref()
                            .ok_or_else(|| contract("imported version absent"))?
                            .payload
                            .as_slice(),
                    )?;
                    let row = index
                        .rows
                        .get(&supplier)
                        .ok_or_else(|| contract("imported record supplier absent"))?;
                    object.references = vec![(scope(row.parent_id), row.name.clone())];
                }
                let version = object
                    .version
                    .as_ref()
                    .ok_or_else(|| contract("imported version absent"))?;
                let expected = edit(
                    object.logical.clone(),
                    object.scope.clone(),
                    object.name.clone(),
                    version.kind.clone(),
                    version.payload.as_slice().to_vec(),
                    object.references.clone(),
                );
                if expected.version.as_ref() != Some(version) {
                    return Err(contract(
                        "imported immutable source bytes or references differ",
                    ));
                }
            }
            let admitted = AdmittedModelRevision::apply(runtime, None, &imported)?;
            Ok((admitted, identity))
        }
        .await;
        let release = read.finish().await;
        let (admitted, identity) = result?;
        release?;
        Self::admitted(admitted, identity, Some(canonical), None)
    }
    fn admitted(
        admitted: Arc<AdmittedModelRevision>,
        identity: SourceRevisionHash,
        origin: Option<Revision>,
        previous: Option<&Self>,
    ) -> Result<Self, WorkflowError> {
        let revision_id = pse_operations::mint_id::<SemanticId>().to_string();
        if let Some(previous) = previous {
            // This is committed only after durable acknowledgement. Concurrent siblings
            // serialize here; the loser branches instead of rewinding the live lane.
            let mut head = previous
                .head
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if head.revision == previous.revision_id {
                let generation = head
                    .generation
                    .checked_add(1)
                    .ok_or_else(|| contract("local source generation extent"))?;
                head.revision = revision_id.clone();
                head.generation = generation;
                return Ok(Self {
                    origin,
                    admitted,
                    identity,
                    revision_id,
                    lineage_id: previous.lineage_id.clone(),
                    generation,
                    head: previous.head.clone(),
                });
            }
        }
        let lineage_id = pse_operations::mint_id::<SemanticId>().to_string();
        let head = Arc::new(std::sync::Mutex::new(local::LocalHead {
            revision: revision_id.clone(),
            generation: 0,
        }));
        Ok(Self {
            origin,
            admitted,
            identity,
            revision_id,
            lineage_id,
            generation: 0,
            head,
        })
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
    let validation = runtime.admission_context();
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
        let retained = edits
            .iter()
            .map(|edit| edit.logical.clone())
            .collect::<BTreeSet<_>>();
        edits.retain(|edit| {
            previous
                .admitted
                .objects
                .get(&edit.logical)
                .is_none_or(|old| {
                    edit.version.as_ref().is_none_or(|version| {
                        old.scope != edit.scope
                            || old.name != edit.name
                            || old.version != version.key
                            || old.kind != version.kind
                            || old.payload.as_slice() != version.payload.as_slice()
                            || old.references != edit.references
                    })
                })
        });
        edits.extend(
            previous
                .admitted
                .objects
                .values()
                .filter(|member| {
                    (member.logical.starts_with("modeling:")
                        || member.logical.starts_with("record:")
                        || member.logical.starts_with("physical_scope_document:"))
                        && !retained.contains(&member.logical)
                })
                .map(|member| ObjectEdit {
                    logical: member.logical.clone(),
                    scope: member.scope.clone(),
                    name: member.name.clone(),
                    version: None,
                    references: Vec::new(),
                }),
        );
        if edits.is_empty() && identity == previous.identity {
            return Ok(previous.clone());
        }
    }
    admit_revision(runtime, previous, identity, &edits).await
}

/// Stage complete local ownership before issuing the existing durable effect.
async fn admit_revision(
    runtime: &Runtime,
    previous: Option<&SourceRevision>,
    identity: SourceRevisionHash,
    edits: &[ObjectEdit],
) -> Result<SourceRevision, WorkflowError> {
    let admitted =
        AdmittedModelRevision::apply(runtime, previous.map(|p| p.admitted.as_ref()), edits)?;
    let origin = match runtime.durability() {
        crate::workflow::Durability::Ephemeral => None,
        crate::workflow::Durability::Durable(_) => {
            let store = runtime.canonical_store()?;
            let old = previous.map(SourceRevision::require_origin).transpose()?;
            let problem = old.map_or_else(
                || pse_operations::mint_id::<SemanticId>().to_string(),
                |old| old.problem.clone(),
            );
            let operation = pse_operations::mint_id::<SemanticId>().to_string();
            let recorded = store
                .edit(&problem, old.map(|old| old.key.as_str()), &operation, edits)
                .await
                .map_err(|source| {
                    crate::workflow::source_settlement::source_edit_error(
                        problem, operation, source,
                    )
                })?;
            Some(recorded)
        }
    };
    SourceRevision::admitted(admitted, identity, origin, previous)
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
        self.revision
            .admitted
            .objects
            .get(logical)
            .cloned()
            .ok_or_else(|| contract(format!("owned source object absent: {logical}")))
    }
    fn source_decode<T: serde::de::DeserializeOwned>(
        &self,
        object: &LocalSourceObject,
    ) -> Result<(T, Arc<pse_columnar::AllocationLease>), WorkflowError> {
        let extent = object
            .payload
            .len()
            .checked_mul(32)
            .and_then(|n| n.checked_add(8192))
            .ok_or_else(|| contract("owned source decode extent"))?;
        let owner = self
            .runtime
            .shared
            .math()
            .reserve("modeling:owned-source-decode", extent)?;
        Ok((decode(&object.payload)?, owner))
    }
    pub(super) async fn declaration_inventory(
        &self,
    ) -> Result<super::OwnedDeclarations, WorkflowError> {
        let parent = self.delta_parent(&crate::CancelSource::new()).await?;
        Ok(super::OwnedDeclarations {
            rows: parent.rows,
            _leases: parent._leases,
        })
    }
    /// Explicit fitting inventory from the owned immutable source and its allocations.
    pub async fn fit_declarations(
        &self,
    ) -> Result<super::super::fitting::FitDeclarations, WorkflowError> {
        let activity = self.runtime.admit_activity()?;
        activity
            .scope(async {
                let mut rows = Vec::new();
                let mut leases = Vec::new();
                for object in self
                    .revision
                    .admitted
                    .objects
                    .values()
                    .filter(|o| o.kind == "fit")
                {
                    let (row, lease) = self.source_decode(object)?;
                    rows.push(row);
                    leases.push(lease);
                }
                Ok(super::super::fitting::FitDeclarations::from_owned_rows(
                    rows, leases,
                ))
            })
            .await
    }
    pub(in crate::workflow) async fn fit_declaration(
        &self,
        id: pse_model::generated::identities::FitId,
    ) -> Result<pse_model::generated::authored::fit_cases::Row, WorkflowError> {
        let object = self.local_object(&format!("fit:{id}"))?;
        Ok(self.source_decode(&object)?.0)
    }
    pub(in crate::workflow) async fn replace_fits(
        &self,
        fits: &[pse_model::generated::authored::fit_cases::Row],
    ) -> Result<SourceRevision, WorkflowError> {
        let existing = self.fit_declarations().await?;
        let old = existing
            .fits
            .iter()
            .map(|row| (row.fit_id, row))
            .collect::<BTreeMap<_, _>>();
        let new = fits
            .iter()
            .map(|row| (row.fit_id, row))
            .collect::<BTreeMap<_, _>>();
        let delta = crate::workflow::SourceDelta {
            parent: self.revision_id().into(),
            fits: crate::workflow::SourceFitDelta {
                additions: fits
                    .iter()
                    .filter(|row| !old.contains_key(&row.fit_id))
                    .cloned()
                    .collect(),
                replacements: fits
                    .iter()
                    .filter(|row| old.get(&row.fit_id).is_some_and(|old| *old != *row))
                    .cloned()
                    .collect(),
                removals: old
                    .keys()
                    .filter(|id| !new.contains_key(id))
                    .copied()
                    .collect(),
            },
            ..Default::default()
        };
        Ok(self
            .with_delta(delta, &crate::CancelSource::new())
            .await?
            .revision)
    }
    pub(super) async fn full_source(
        &self,
    ) -> Result<
        (
            Vec<Declaration>,
            PhysicalScope,
            Arc<pse_modeling::document::DocumentInventory>,
            pse_admission::authoring_driver::document::Batches,
            Vec<Arc<pse_columnar::AllocationLease>>,
        ),
        WorkflowError,
    > {
        use pse_relations::generated::authored::{documents, fit_cases, packages};
        let parent = self.delta_parent(&crate::CancelSource::new()).await?;
        let mut leases = parent._leases;
        let validation = self.runtime.admission_context();
        let mut inventory = pse_modeling::document::DocumentInventory {
            field_spans: parent.fields,
            ..Default::default()
        };
        let mut document_builder = documents::Builder::with_registry(
            self.runtime.registry(),
            parent.documents.len(),
            &validation,
        )
        .map_err(super::super::relation)?;
        for mut row in parent.documents.into_values() {
            inventory
                .packages
                .insert(row.document_id, row.package_id.as_id());
            if let Some(data) = self
                .revision
                .admitted
                .objects
                .get(&format!("data:{}", row.document_id))
            {
                let (document, lease) =
                    pse_admission::authoring_driver::document::decode_selected_data(
                        row.document_id,
                        row.path.clone(),
                        data.payload.clone().into(),
                        Default::default(),
                        &self.runtime.shared.pool(),
                        &crate::CancelSource::new().token(),
                    )?;
                inventory.documents.insert(row.document_id, document);
                leases.push(lease);
                row.content = Some(data.payload.clone().into());
            }
            document_builder.push(row).map_err(super::super::relation)?;
        }
        let mut package_builder =
            packages::Builder::with_registry(self.runtime.registry(), 0, &validation)
                .map_err(super::super::relation)?;
        let mut fit_builder =
            fit_cases::Builder::with_registry(self.runtime.registry(), 0, &validation)
                .map_err(super::super::relation)?;
        for object in self.revision.admitted.objects.values() {
            match object.kind.as_str() {
                "package" => {
                    let (row, lease) = self.source_decode(object)?;
                    package_builder.push(row).map_err(super::super::relation)?;
                    leases.push(lease);
                }
                "fit" => {
                    let (row, lease) = self.source_decode(object)?;
                    fit_builder.push(row).map_err(super::super::relation)?;
                    leases.push(lease);
                }
                _ => {}
            }
        }
        for row in &parent.rows {
            inventory.parsed_fields.insert(
                row.declaration_id,
                parsed_fields(
                    &self.runtime,
                    row,
                    inventory.field_spans.get(&row.declaration_id),
                )?,
            );
        }
        let sources = BTreeMap::from([
            (
                documents::RELATION_ID,
                document_builder.finish().map_err(super::super::relation)?,
            ),
            (
                packages::RELATION_ID,
                package_builder.finish().map_err(super::super::relation)?,
            ),
            (
                fit_cases::RELATION_ID,
                fit_builder.finish().map_err(super::super::relation)?,
            ),
        ]);
        Ok((
            parent.rows,
            parent.physical,
            Arc::new(inventory),
            sources,
            leases,
        ))
    }
    pub(super) async fn selected_declarations(
        &self,
        selected: &BTreeSet<DeclarationId>,
        cancel: &crate::CancelSource,
    ) -> Result<super::OwnedDeclarations, WorkflowError> {
        if selected.is_empty() {
            return Err(contract("a fixture selection names at least one test"));
        }
        let parent = self.delta_parent(cancel).await?;
        let rows = parent
            .rows
            .into_iter()
            .filter(|row| selected.contains(&row.declaration_id))
            .collect::<Vec<_>>();
        if rows.len() != selected.len() {
            return Err(contract("selected declaration absent"));
        }
        Ok(super::OwnedDeclarations {
            rows,
            _leases: parent._leases,
        })
    }
    pub(super) async fn declaration(
        &self,
        id: DeclarationId,
    ) -> Result<Declaration, WorkflowError> {
        Ok(self
            .local_object(&logical(id))?
            .declaration
            .as_ref()
            .ok_or_else(|| contract("typed declaration absent"))?
            .0
            .clone())
    }
    pub(super) async fn checked_selection(
        &self,
        roots: &[DeclarationId],
        cancel: &crate::CancelSource,
    ) -> Result<CheckedSelection, WorkflowError> {
        checkpoint(cancel)?;
        let math = self.runtime.shared.math();
        let request = Arc::new(super::SelectedRequest {
            roots: roots.to_vec(),
            physical: self.physical.key,
            providers: self
                .providers
                .iter()
                .map(|(name, provider)| {
                    (
                        name.clone(),
                        provider.spec().key(),
                        provider.configuration_key(),
                    )
                })
                .collect(),
            validation: Arc::as_ptr(self.runtime.sessions()) as usize,
            registry: Arc::as_ptr(self.runtime.registry()) as usize,
        });
        if let Some(candidates) = math.modeling_cache.selected(&request) {
            for admission in candidates.rchunks(8).flat_map(|window| window.iter().rev()) {
                checkpoint(cancel)?;
                if admission.dependencies.matches(&self.revision.admitted) {
                    return Ok(CheckedSelection {
                        admission: admission.clone(),
                        _premises_metadata: None,
                    });
                }
            }
        }
        let generation = math.modeling_cache.generation();
        let (index, physical, documents, leases, dependencies) =
            self.select_local(roots, cancel)?;
        let dependencies = Arc::new(dependencies);
        let extent = dependencies
            .retained_bytes()
            .checked_add(request.retained_bytes())
            .and_then(|n| {
                n.checked_add(
                    index
                        .versions
                        .iter()
                        .map(|(k, v)| k.capacity() + v.capacity() + 128)
                        .sum::<usize>(),
                )
            })
            .and_then(|n| n.checked_add(size_of::<super::SelectedAdmission>() + 512))
            .ok_or_else(|| contract("local selected metadata extent"))?;
        let metadata = math.reserve("modeling:local-selected-admission", extent)?;
        let rows = index.rows.into_values().collect();
        let suppliers = index.record_sources;
        let versions = Arc::new(index.versions);
        let inputs = super::compiler_context(&self.physical, &self.providers);
        let lineage = self.compiler_lineage()?;
        let stamp = crate::math::modeling::LaneStamp::new(
            self.revision.generation(),
            self.revision.revision_id(),
        );
        let physical_key = self.physical.key;
        let service = math.clone();
        let flight_metadata = metadata.clone();
        let key = (request.clone(), dependencies.clone());
        let population = crate::math::PopulationScope::capture();
        let operation = math.selected_flights.load_owned(key, move |control| {
            population.run(async move {
                let worker_service = service.clone();
                let admitted = service
                    .job(1, 0, control, move |flag| {
                        let _sources = leases;
                        let _metadata = flight_metadata;
                        if flag.load(std::sync::atomic::Ordering::Acquire) {
                            return Err(crate::math::MathRuntimeError::Cancelled);
                        }
                        let workspace = worker_service.compiler_lane(lineage, inputs)?;
                        worker_service.modeling_revision_at(
                            &workspace,
                            rows,
                            physical,
                            Arc::new(documents),
                            &physical_key,
                            stamp,
                            flag,
                            &suppliers,
                        )
                    })
                    .await?;
                Ok(Arc::new(admitted))
            })
        });
        tokio::pin!(operation);
        let revision = tokio::select! {
            result=&mut operation=>result.map_err(|error|match error {
                pse_columnar::flight::FlightError::Load(error)=>crate::math::MathRuntimeError::Shared(error),
                pse_columnar::flight::FlightError::Capacity=>crate::math::MathRuntimeError::Limit("selected admission flights"),
                pse_columnar::flight::FlightError::Retiring=>crate::math::MathRuntimeError::Retiring,
                pse_columnar::flight::FlightError::Panicked=>crate::math::MathRuntimeError::Panic("selected admission task panic".into()),
            })?,
            ()=cancel.cancelled()=>return Err(crate::math::MathRuntimeError::Cancelled.into()),
        };
        checkpoint(cancel)?;
        let admission = Arc::new(super::SelectedAdmission {
            request,
            revision: revision.as_ref().clone(),
            dependencies,
            versions,
            metadata,
            _validation_owner: self.runtime.model.clone(),
        });
        math.modeling_cache
            .retain_selected(generation, admission.clone());
        Ok(CheckedSelection {
            admission,
            _premises_metadata: None,
        })
    }
    fn compiler_lineage(&self) -> Result<crate::math::modeling::CompilerLineage, WorkflowError> {
        Ok(crate::math::modeling::CompilerLineage::new(
            self.revision.lineage_id(),
            pse_operations::generated::surreal::INTERPRETATION,
            self.runtime.sessions(),
            self.runtime.registry().clone(),
            self.runtime.model.clone(),
        )?)
    }
    pub(crate) fn pure_workspace(&self) -> Result<crate::math::Workspace, WorkflowError> {
        Ok(self.runtime.shared.math().compiler_lane(
            self.compiler_lineage()?,
            super::compiler_context(&self.physical, &self.providers),
        )?)
    }
    pub(in crate::workflow) async fn selected_revision(
        &self,
        root: DeclarationId,
        cancel: &crate::CancelSource,
    ) -> Result<crate::math::modeling::ModelingRevision, WorkflowError> {
        Ok(self
            .checked_selection(&[root], cancel)
            .await?
            .admission
            .revision
            .clone())
    }
    pub(in crate::workflow) async fn selected_revision_roots(
        &self,
        roots: &[DeclarationId],
        cancel: &crate::CancelSource,
    ) -> Result<crate::math::modeling::ModelingRevision, WorkflowError> {
        Ok(self
            .checked_selection(roots, cancel)
            .await?
            .admission
            .revision
            .clone())
    }
    pub(in crate::workflow) async fn all_revision(
        &self,
    ) -> Result<crate::math::modeling::ModelingRevision, WorkflowError> {
        let parent = self.delta_parent(&crate::CancelSource::new()).await?;
        let roots = parent
            .rows
            .iter()
            .map(|row| row.declaration_id)
            .collect::<Vec<_>>();
        self.selected_revision_roots(&roots, &crate::CancelSource::new())
            .await
    }
    fn insert_local(
        &self,
        index: &mut SourceIndex,
        objects: Vec<Arc<LocalSourceObject>>,
        leases: &mut Vec<Arc<pse_columnar::AllocationLease>>,
        cancel: &crate::CancelSource,
    ) -> Result<(), WorkflowError> {
        for object in objects {
            checkpoint(cancel)?;
            if index.versions.contains_key(&object.logical) {
                continue;
            }
            if !object.kind.starts_with("modeling:") || object.kind == CONTEXT {
                return Err(contract("local lexical scope contains foreign object kind"));
            }
            let source = object
                .declaration
                .as_ref()
                .ok_or_else(|| contract("owned typed declaration absent"))?
                .clone();
            let lease = self.runtime.shared.math().reserve(
                "modeling:local-declaration-copy",
                source
                    .0
                    .owned_bytes()
                    .saturating_add(source.1.as_ref().map_or(0, |fields| fields.len() * 256))
                    .saturating_add(1024),
            )?;
            if let Some(parsed) = &object.parsed {
                index
                    .parsed_fields
                    .insert(source.0.declaration_id, parsed.clone());
            }
            if logical(source.0.declaration_id) != object.logical
                || kind(&source.0) != object.kind
                || scope(source.0.parent_id) != object.scope
                || source.0.name != object.name
            {
                return Err(contract(
                    "owned source metadata differs from generated declaration",
                ));
            }
            index.insert(source)?;
            index
                .versions
                .insert(object.logical.clone(), object.version.clone());
            leases.push(lease);
        }
        Ok(())
    }
    fn select_local(
        &self,
        roots: &[DeclarationId],
        cancel: &crate::CancelSource,
    ) -> Result<
        (
            SourceIndex,
            PhysicalScope,
            pse_modeling::document::DocumentInventory,
            Vec<Arc<pse_columnar::AllocationLease>>,
            LocalSourcePremises,
        ),
        WorkflowError,
    > {
        let source = &self.revision.admitted;
        let mut premises = LocalSourcePremises::default();
        let physical_object = premises
            .logical(source, PHYSICAL_SCOPE_LOGICAL)
            .ok_or_else(|| contract("owned physical scope absent"))?;
        let (mut physical, lease) = self.source_decode::<PhysicalScope>(&physical_object)?;
        let mut leases = vec![lease];
        let mut index = SourceIndex::default();
        index.versions.insert(
            physical_object.logical.clone(),
            physical_object.version.clone(),
        );
        index
            .requests
            .extend(roots.iter().copied().map(Request::Logical));
        let mut suppliers = BTreeSet::new();
        loop {
            checkpoint(cancel)?;
            let prior_rows = index.rows.len();
            let mut requests = std::mem::take(&mut index.requests)
                .into_iter()
                .collect::<Vec<_>>();
            requests.sort_by_key(|request| match request {
                Request::Members(_) => 0,
                Request::Logical(_) => 1,
                Request::Imports(_) => 2,
                Request::Name(_, _) => 3,
            });
            let requested_scopes = requests
                .iter()
                .filter_map(|request| match request {
                    Request::Members(owner) => Some(*owner),
                    _ => None,
                })
                .collect::<BTreeSet<_>>();
            for request in requests {
                let objects = match request {
                    Request::Logical(id)
                        if !index.rows.contains_key(&id)
                            && !index.record_sources.contains_key(&id) =>
                    {
                        if let Some(object) = premises.logical(source, &logical(id)) {
                            vec![object]
                        } else {
                            let object=premises.logical(source,&format!("record:{id}")).ok_or_else(||contract("selected modeling declaration, record or lexical ancestor absent"))?;
                            let (supplier, lease) = self.source_decode::<DeclarationId>(&object)?;
                            leases.push(lease);
                            index
                                .versions
                                .insert(object.logical.clone(), object.version.clone());
                            index.record_sources.insert(id, supplier);
                            index.requests.insert(Request::Logical(supplier));
                            Vec::new()
                        }
                    }
                    Request::Name(parent, name) if index.needs_name(parent, &name) => {
                        let objects = premises
                            .inventory(source, LocalInventory::Name(scope(parent), name.clone()));
                        index.names.insert((parent, name));
                        objects
                    }
                    Request::Members(owner) => {
                        let objects =
                            premises.inventory(source, LocalInventory::Scope(logical(owner)));
                        index.scopes.insert(owner);
                        index.imports.insert(owner);
                        objects
                    }
                    Request::Imports(owner)
                        if index.scopes.contains(&owner) || requested_scopes.contains(&owner) =>
                    {
                        Vec::new()
                    }
                    Request::Imports(owner) if !index.rows.contains_key(&owner) => {
                        index.requests.insert(Request::Logical(owner));
                        index.requests.insert(Request::Imports(owner));
                        Vec::new()
                    }
                    Request::Imports(owner) => {
                        let mut objects = premises.inventory(
                            source,
                            LocalInventory::Kind(Some(logical(owner)), "modeling:import".into()),
                        );
                        if index.rows[&owner].value.kind == Kind::Package {
                            objects.extend(premises.inventory(
                                source,
                                LocalInventory::Kind(Some(logical(owner)), ENGINEERING_RULE.into()),
                            ));
                        }
                        index.imports.insert(owner);
                        objects
                    }
                    _ => Vec::new(),
                };
                self.insert_local(&mut index, objects, &mut leases, cancel)?;
            }
            let new_rows = std::mem::take(&mut index.new_rows);
            let mut supplier_demands = Vec::new();
            for id in new_rows {
                checkpoint(cancel)?;
                let row = index.rows[&id].clone();
                if index.needs_name(row.parent_id, &row.name) {
                    index
                        .requests
                        .insert(Request::Name(row.parent_id, row.name.clone()));
                }
                if selected_source::requires_members(&row) && !index.scopes.contains(&id) {
                    index.requests.insert(Request::Members(id));
                } else if selected_source::owns_namespace(&row)
                    && !index.imports.contains(&id)
                    && !index.scopes.contains(&id)
                {
                    index.requests.insert(Request::Imports(id));
                }
                if row.value.import.is_some() && index.needs_name(None, &row.name) {
                    index.requests.insert(Request::Name(None, row.name.clone()));
                }
                let parsed = index
                    .parsed_fields
                    .get(&id)
                    .cloned()
                    .map(Ok)
                    .unwrap_or_else(|| parsed_fields(&self.runtime, &row, index.fields.get(&id)))?;
                let references = selected_source::declaration_references_parsed(&row, &parsed)
                    .map_err(super::super::modeling_error)?;
                let lease = self
                    .runtime
                    .shared
                    .math()
                    .reserve("modeling:local-reference-paths", references.owned_bytes())?;
                index.parsed_fields.insert(id, parsed);
                index.pending_references.insert(id, references);
                leases.push(lease);
                if selected_source::requires_suppliers(&row) && suppliers.insert(id) {
                    let kinds: &[&str] = match row.value.kind {
                        Kind::IdentifierScheme => &["modeling:attribute"],
                        Kind::EntityKind => &[
                            "modeling:dataset",
                            "modeling:entity",
                            "modeling:entity_kind",
                        ],
                        _ => &["modeling:dataset"],
                    };
                    for kind in kinds {
                        supplier_demands.push(LocalInventory::References(
                            scope(row.parent_id),
                            row.name.clone(),
                            (*kind).into(),
                        ));
                    }
                }
            }
            for query in supplier_demands {
                let objects = premises.inventory(source, query);
                self.insert_local(&mut index, objects, &mut leases, cancel)?;
            }
            for (owner, references) in std::mem::take(&mut index.pending_references) {
                checkpoint(cancel)?;
                index.pending(owner, references);
            }
            let mut requests = std::mem::take(&mut index.requests);
            requests.retain(|request| match request {
                Request::Logical(id) => {
                    !index.rows.contains_key(id) && !index.record_sources.contains_key(id)
                }
                Request::Name(parent, name) => index.needs_name(*parent, name),
                Request::Imports(id) => !index.imports.contains(id) && !index.scopes.contains(id),
                Request::Members(id) => !index.scopes.contains(id),
            });
            index.requests = requests;
            if index.requests.is_empty() && index.rows.len() == prior_rows {
                break;
            }
        }
        if roots
            .iter()
            .any(|root| !index.rows.contains_key(root) && !index.record_sources.contains_key(root))
        {
            return Err(contract("selected modeling root absent"));
        }
        let document_ids = index
            .rows
            .values()
            .map(|row| row.document_id)
            .collect::<BTreeSet<_>>();
        let mut documents = pse_modeling::document::DocumentInventory {
            field_spans: index.fields.clone(),
            parsed_fields: index.parsed_fields.clone(),
            ..Default::default()
        };
        let mut metadata_by_document = BTreeMap::new();
        for id in document_ids {
            if physical.documents.is_some()
                && let Some(object) =
                    premises.logical(source, &format!("physical_scope_document:{id}"))
            {
                let (entry, lease) = self.source_decode::<PhysicalScope>(&object)?;
                if entry.package != physical.package
                    || entry.documents != Some(BTreeSet::from([id]))
                {
                    return Err(contract("owned document physical scope differs"));
                }
                if let Some(visible) = &mut physical.documents {
                    visible.insert(id);
                }
                index
                    .versions
                    .insert(object.logical.clone(), object.version.clone());
                leases.push(lease);
            }
            if let Some(object) = premises.logical(source, &format!("document:{id}")) {
                let (row, lease) =
                    self.source_decode::<pse_model::generated::authored::documents::Row>(&object)?;
                documents.packages.insert(id, row.package_id.as_id());
                metadata_by_document.insert(id, row);
                index
                    .versions
                    .insert(object.logical.clone(), object.version.clone());
                leases.push(lease);
            }
        }
        let mut data_documents = BTreeMap::new();
        for row in index.rows.values() {
            if let Some(path) = row
                .value
                .dataset
                .as_ref()
                .and_then(|dataset| dataset.document.as_ref())
            {
                let owner = metadata_by_document.get(&row.document_id).ok_or_else(|| {
                    contract("selected dataset declaring document metadata absent")
                })?;
                data_documents.insert(
                    pse_ids::named_id(owner.package_id.as_id(), path),
                    path.clone(),
                );
            }
        }
        for (id, path) in data_documents {
            checkpoint(cancel)?;
            let object = premises
                .logical(source, &format!("data:{id}"))
                .ok_or_else(|| contract("selected data document absent"))?;
            let (document, lease) =
                pse_admission::authoring_driver::document::decode_selected_data(
                    id,
                    path,
                    object.payload.clone().into(),
                    Default::default(),
                    &self.runtime.shared.pool(),
                    &cancel.token(),
                )?;
            documents.documents.insert(id, document);
            index
                .versions
                .insert(object.logical.clone(), object.version.clone());
            leases.push(lease);
        }
        Ok((index, physical, documents, leases, premises))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    async fn local_source_admission_and_selection_need_no_store() {
        let runtime = crate::workflow::tests::runtime();
        assert!(matches!(
            runtime.canonical(),
            Err(WorkflowError::RecordingUnavailable)
        ));
        let rows = parse(
            "package p {param a:Scalar=1;param b:Scalar=2;def Root {var x:Scalar;eq e:x==a;}}",
        );
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(rows.clone(), crate::workflow::tests::physical())
            .await
            .unwrap();
        assert!(package.canonical_revision().is_none());
        let selected = package
            .checked_selection(&[root], &crate::CancelSource::new())
            .await
            .unwrap();
        let declarations = selected.admission.revision.declarations();
        assert!(declarations.iter().any(|r| r.name == "a"));
        assert!(!declarations.iter().any(|r| r.name == "b"));
        assert_eq!(&*package.declarations().await.unwrap(), rows.as_slice());
        let same = runtime
            .modeling_package(rows, package.physical.clone())
            .await
            .unwrap();
        assert_ne!(same.revision_id(), package.revision_id());
        assert_ne!(same.revision.lineage_id(), package.revision.lineage_id());
        let reused = same
            .checked_selection(&[root], &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(Arc::ptr_eq(&selected.admission, &reused.admission));
    }
    #[tokio::test]
    async fn local_siblings_branch_and_cancel_keeps_previous_head() {
        let runtime = crate::workflow::tests::runtime();
        let package = runtime
            .modeling_package(
                parse("package p {param a:Scalar=1;}"),
                crate::workflow::tests::physical(),
            )
            .await
            .unwrap();
        let left = parse("package p {param a:Scalar=2;}");
        let right = parse("package p {param a:Scalar=3;}");
        let (left, right) = tokio::join!(
            package.with_declarations(left),
            package.with_declarations(right)
        );
        let left = left.unwrap();
        let right = right.unwrap();
        assert_ne!(left.revision_id(), right.revision_id());
        assert_ne!(left.revision.lineage_id(), right.revision.lineage_id());
        assert!(
            left.revision.lineage_id() == package.revision.lineage_id()
                || right.revision.lineage_id() == package.revision.lineage_id()
        );
        assert_eq!(
            &*package.declarations().await.unwrap(),
            parse("package p {param a:Scalar=1;}").as_slice()
        );
        let before = left.revision.head.lock().unwrap().revision.clone();
        let cancel = crate::CancelSource::new();
        cancel.cancel();
        assert!(
            left.with_delta(
                crate::workflow::SourceDelta {
                    parent: left.revision_id().into(),
                    ..Default::default()
                },
                &cancel
            )
            .await
            .is_err()
        );
        assert_eq!(left.revision.head.lock().unwrap().revision, before);
    }
    #[tokio::test]
    async fn local_exact_premises_reject_payload_collision_and_new_shadow() {
        let runtime = crate::workflow::tests::runtime();
        let package = runtime
            .modeling_package(
                parse("package p {param a:Scalar=1;def Root {var x:Scalar;eq e:x==a;}}"),
                crate::workflow::tests::physical(),
            )
            .await
            .unwrap();
        let root = package
            .declarations()
            .await
            .unwrap()
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let selected = package
            .checked_selection(&[root], &crate::CancelSource::new())
            .await
            .unwrap();
        let object = package
            .revision
            .admitted
            .objects
            .values()
            .find(|o| o.name == "a")
            .unwrap();
        let mut forged = object.edit();
        let replacement = parse("package p {param a:Scalar=2;def Root {var x:Scalar;eq e:x==a;}}")
            .into_iter()
            .find(|r| r.name == "a")
            .unwrap();
        forged.version.as_mut().unwrap().payload =
            payload(&(replacement, None::<FieldSpans>)).unwrap().into();
        // Keep its correspondence key deliberately unchanged: full equality refuses reuse.
        let forged_owner =
            AdmittedModelRevision::apply(&runtime, Some(&package.revision.admitted), &[forged])
                .unwrap();
        assert!(!selected.admission.dependencies.matches(&forged_owner));
        let shadow = package
            .with_declarations(parse(
                "package p {param a:Scalar=1;def Root {param a:Scalar=4;var x:Scalar;eq e:x==a;}}",
            ))
            .await
            .unwrap();
        assert!(
            !selected
                .admission
                .dependencies
                .matches(&shadow.revision.admitted)
        );
        let current = shadow
            .checked_selection(&[root], &crate::CancelSource::new())
            .await
            .unwrap();
        assert_ne!(
            selected.admission.dependencies,
            current.admission.dependencies
        );
    }
    #[cfg(feature = "canonical-tests")]
    #[tokio::test]
    async fn durable_source_ack_conflict_and_disconnected_owned_compute() {
        let runtime = crate::workflow::tests::durable_runtime();
        let package = runtime
            .modeling_package(
                parse("package p {param a:Scalar=1;def Root {var x:Scalar;eq e:x==a;}}"),
                crate::workflow::tests::physical(),
            )
            .await
            .unwrap();
        let root = package
            .declarations()
            .await
            .unwrap()
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let origin = package.canonical_revision().unwrap().clone();
        let imported = runtime
            .modeling_revision(origin.clone(), package.physical.clone(), BTreeMap::new())
            .await
            .unwrap();
        assert_ne!(imported.revision_id(), package.revision_id());
        assert_eq!(imported.revision.identity, package.revision.identity);
        let changed = package
            .with_declarations(parse(
                "package p {param a:Scalar=2;def Root {var x:Scalar;eq e:x==a;}}",
            ))
            .await
            .unwrap();
        let conflict = imported
            .with_declarations(parse(
                "package p {param a:Scalar=3;def Root {var x:Scalar;eq e:x==a;}}",
            ))
            .await
            .unwrap_err();
        assert!(
            matches!(
                conflict,
                WorkflowError::Canonical(
                    pse_operations::canonical::CanonicalError::SourceEditRefused { .. }
                )
            ),
            "head conflict is proved refusal: {conflict}"
        );
        assert_eq!(imported.canonical_revision().unwrap(), &origin);
        runtime.canonical_store().unwrap().close();
        assert!(
            changed
                .checked_selection(&[root], &crate::CancelSource::new())
                .await
                .is_ok()
        );
        assert!(
            changed
                .with_declarations(parse(
                    "package p {param a:Scalar=4;def Root {var x:Scalar;eq e:x==a;}}"
                ))
                .await
                .is_err()
        );
    }
}

impl ModelingPackage {
    pub(in crate::workflow) fn replay_source_extent(&self) -> usize {
        let providers = self
            .providers
            .iter()
            .map(|(name, provider)| {
                let spec = provider.spec();
                name.len()
                    .saturating_add((spec.inputs.len() + spec.outputs.len()).saturating_mul(256))
                    .saturating_add(
                        spec.shapes
                            .inputs
                            .iter()
                            .chain(&spec.shapes.outputs)
                            .map(|shape| {
                                (shape.axes.len()
                                    + shape.cells.len()
                                    + shape.coordinates.iter().map(Vec::len).sum::<usize>())
                                .saturating_mul(64)
                                .saturating_add(shape.coordinates.len().saturating_mul(128))
                                .saturating_add(512)
                            })
                            .sum::<usize>(),
                    )
                    .saturating_add(1024)
            })
            .sum::<usize>();
        self.revision
            .admitted
            .retained_bytes()
            .saturating_add(self.physical.retained_bytes())
            .saturating_mul(16)
            .saturating_add(providers)
            .saturating_add(65536)
    }
    /// Export owned immutable inputs; physical sources are actual admitted bytes.
    pub(in crate::workflow) fn replay_source(
        &self,
    ) -> Result<crate::workflow::replay::ReplaySourceDocument, WorkflowError> {
        use crate::workflow::replay::{ReplayProvider, ReplaySourceDocument, ReplaySourceObject};
        Ok(ReplaySourceDocument {
            identity: self.revision.identity().as_id(),
            interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
            build: pse_backend_native::execution::LINKED.build_identity(),
            physical_identity: self.physical.identity(),
            physical_sources: crate::workflow::replay::physical_sources(&self.physical)?,
            providers: self
                .providers
                .iter()
                .map(|(name, provider)| ReplayProvider::new(name, provider))
                .collect(),
            objects: self
                .revision
                .admitted
                .objects
                .values()
                .map(|object| ReplaySourceObject {
                    logical: object.logical.clone(),
                    scope: object.scope.clone(),
                    name: object.name.clone(),
                    version: object.version.clone(),
                    kind: object.kind.clone(),
                    payload: object.payload.clone(),
                    references: object.references.clone(),
                })
                .collect(),
        })
    }
    /// Reconstruct through production local admission while retaining actual callable premises.
    pub(in crate::workflow) fn reconstruct_replay_source(
        &self,
        runtime: &Runtime,
    ) -> Result<ModelingPackage, WorkflowError> {
        let bytes = self.replay_source_extent();
        let _scratch = runtime
            .shared
            .math()
            .reserve("replay:retained-source-reconstruction", bytes)?;
        self.replay_source()?.reconstruct(
            runtime,
            self.physical.clone(),
            self.providers.as_ref().clone(),
        )
    }
}
impl Runtime {
    pub(in crate::workflow) fn admit_replay_source(
        &self,
        document: &crate::workflow::replay::ReplaySourceDocument,
        physical: super::PhysicalContext,
        providers: BTreeMap<String, pse_kernels::Registration>,
    ) -> Result<ModelingPackage, WorkflowError> {
        // Refuse cheap identity mismatches before copying actual supplied premises.
        if document.interpretation != pse_operations::generated::surreal::INTERPRETATION
            || document.build != pse_backend_native::execution::LINKED.build_identity()
            || document.physical_identity != physical.identity()
            || document.providers.len() != providers.len()
        {
            return Err(contract(
                "replay interpretation, native build, physical or provider premise differs",
            ));
        }
        let actual_providers = providers
            .iter()
            .try_fold(0usize, |bytes, (name, provider)| {
                let spec = provider.spec();
                let shapes = spec
                    .shapes
                    .inputs
                    .iter()
                    .chain(&spec.shapes.outputs)
                    .try_fold(0usize, |bytes, shape| {
                        let coordinates = shape
                            .coordinates
                            .iter()
                            .try_fold(0usize, |n, coordinate| n.checked_add(coordinate.len()))?;
                        bytes
                            .checked_add(
                                shape
                                    .axes
                                    .len()
                                    .checked_add(shape.cells.len())?
                                    .checked_add(coordinates)?
                                    .checked_mul(64)?,
                            )?
                            .checked_add(shape.coordinates.len().checked_mul(128)?)?
                            .checked_add(512)
                    })?;
                bytes
                    .checked_add(name.len())?
                    .checked_add(
                        spec.inputs
                            .len()
                            .checked_add(spec.outputs.len())?
                            .checked_mul(256)?,
                    )?
                    .checked_add(shapes)
            })
            .ok_or_else(|| contract("actual replay provider premise extent overflow"))?;
        let actual_bytes = physical
            .retained_bytes()
            .checked_mul(16)
            .and_then(|bytes| bytes.checked_add(actual_providers))
            .and_then(|bytes| bytes.checked_add(65536))
            .filter(|bytes| *bytes <= 256 * 1024 * 1024)
            .ok_or_else(|| {
                contract("actual replay premises exceed finite reconstruction allocation")
            })?;
        let bytes = crate::workflow::replay::source_reconstruction_extent(document)?
            .checked_add(actual_bytes)
            .filter(|bytes| *bytes <= 256 * 1024 * 1024)
            .ok_or_else(|| {
                contract("complete replay premises exceed finite reconstruction allocation")
            })?;
        let _scratch = self
            .shared
            .math()
            .reserve("replay:source-reconstruction", bytes)?;
        if document.interpretation != pse_operations::generated::surreal::INTERPRETATION
            || document.build != pse_backend_native::execution::LINKED.build_identity()
            || document.physical_identity != physical.identity()
            || document.physical_sources != crate::workflow::replay::physical_sources(&physical)?
        {
            return Err(contract(
                "replay interpretation, native build or actual physical sources differ",
            ));
        }
        let actual: Vec<_> = providers
            .iter()
            .map(|(name, provider)| crate::workflow::replay::ReplayProvider::new(name, provider))
            .collect();
        if document.providers != actual {
            return Err(contract(
                "replay requires every exact executable provider registration",
            ));
        }
        let mut edits = Vec::with_capacity(document.objects.len());
        let mut logicals = BTreeSet::new();
        let mut names = BTreeSet::new();
        for object in &document.objects {
            if !logicals.insert(object.logical.clone())
                || !names.insert((object.scope.clone(), object.name.clone()))
            {
                return Err(contract(
                    "replay source contains duplicate logical or lexical identity",
                ));
            }
            if object.references.iter().collect::<BTreeSet<_>>().len() != object.references.len() {
                return Err(contract(
                    "replay source contains duplicate immutable reference edges",
                ));
            }
            let edited = edit(
                object.logical.clone(),
                object.scope.clone(),
                object.name.clone(),
                object.kind.clone(),
                object.payload.clone(),
                object.references.clone(),
            );
            if edited
                .version
                .as_ref()
                .is_none_or(|version| version.key != object.version)
            {
                return Err(contract(
                    "replay source object body/reference identity differs",
                ));
            }
            edits.push(edited);
        }
        if document
            .objects
            .iter()
            .flat_map(|object| &object.references)
            .any(|reference| !names.contains(reference))
        {
            return Err(contract(
                "replay source is missing a referenced immutable object",
            ));
        }
        let context = document
            .objects
            .iter()
            .find(|object| object.logical == CONTEXT_LOGICAL)
            .ok_or_else(|| contract("replay source context absent"))?;
        if context.kind != CONTEXT
            || !logicals.contains(PHYSICAL_SCOPE_LOGICAL)
            || !logicals.contains(INGRESS_FRAME)
        {
            return Err(contract(
                "replay source interpretation or physical/ingress frame absent",
            ));
        }
        let identity: SourceRevisionHash = decode(&context.payload)?;
        if identity.as_id() != document.identity {
            return Err(contract(
                "replay source identity differs from its actual context body",
            ));
        }
        // Recompute lexical dependency meaning from actual declarations. Self-consistent
        // hashes alone do not establish a complete source closure.
        let mut index = SourceIndex {
            roots_complete: true,
            ..Default::default()
        };
        for object in &document.objects {
            if object.kind.starts_with("modeling:") && object.kind != CONTEXT {
                let source: SourcePayload = decode(&object.payload)?;
                if object.logical != logical(source.0.declaration_id)
                    || object.scope != scope(source.0.parent_id)
                    || object.name != source.0.name
                    || object.kind != kind(&source.0)
                {
                    return Err(contract(
                        "replay declaration membership differs from its body",
                    ));
                }
                index.parsed_fields.insert(
                    source.0.declaration_id,
                    parsed_fields(self, &source.0, source.1.as_ref())?,
                );
                index.scopes.insert(source.0.declaration_id);
                index.imports.insert(source.0.declaration_id);
                index.insert(source)?;
            }
        }
        let rows: Vec<_> = index.rows.values().cloned().collect();
        for row in &rows {
            let references: BTreeSet<_> = index
                .references(row)?
                .into_iter()
                .map(|id| {
                    let target = &index.rows[&id];
                    (scope(target.parent_id), target.name.clone())
                })
                .collect();
            let object = document
                .objects
                .iter()
                .find(|object| object.logical == logical(row.declaration_id))
                .ok_or_else(|| contract("replay declaration body absent"))?;
            if references != object.references.iter().cloned().collect() {
                return Err(contract(
                    "replay lexical dependency closure differs from actual source meaning",
                ));
            }
        }
        let frame_object = document
            .objects
            .iter()
            .find(|object| object.logical == INGRESS_FRAME)
            .ok_or_else(|| contract("replay ingress frame absent"))?;
        let frame: IngressFrame = decode(&frame_object.payload)?;
        let ordered: BTreeSet<_> = frame.order.iter().copied().collect();
        if ordered.len() != frame.order.len() || ordered != index.rows.keys().copied().collect() {
            return Err(contract(
                "replay ingress order omits or duplicates a declaration",
            ));
        }
        let ordered_rows = frame
            .order
            .iter()
            .map(|id| index.rows[id].clone())
            .collect::<Vec<_>>();
        let physical_object = document
            .objects
            .iter()
            .find(|object| object.logical == PHYSICAL_SCOPE_LOGICAL)
            .ok_or_else(|| contract("replay physical scope absent"))?;
        let mut physical_scope: PhysicalScope = decode(&physical_object.payload)?;
        if let Some(documents) = &mut physical_scope.documents {
            for object in &document.objects {
                if object.kind == "physical_scope_document" {
                    let member: PhysicalScope = decode(&object.payload)?;
                    if member.package != physical_scope.package {
                        return Err(contract("replay physical document package differs"));
                    }
                    if let Some(visible) = member.documents {
                        documents.extend(visible);
                    }
                }
            }
        }
        for (id, expected) in &frame.data {
            let data = document
                .objects
                .iter()
                .find(|object| object.logical == format!("data:{id}"))
                .ok_or_else(|| contract("replay data payload absent"))?;
            if pse_ids::preimage::hash(&data.payload) != *expected {
                return Err(contract(
                    "replay data bytes differ from original ingress content",
                ));
            }
        }
        let names = selected_source::physical_bindings(
            &ordered_rows,
            &physical_scope,
            &physical.quantities,
        );
        let actual_identity = crate::math::modeling::source_revision_frame(
            &ordered_rows,
            frame.data.iter().map(|(id, content)| (*id, *content)),
            &physical.key,
            &names,
        );
        if actual_identity != identity {
            return Err(contract(
                "replay source context differs from actual complete scientific inputs",
            ));
        }
        let admitted = AdmittedModelRevision::apply(self, None, &edits)?;
        let revision = SourceRevision::admitted(admitted, identity, None, None)?;
        self.package_from_revision(revision, physical, providers)
    }
}
