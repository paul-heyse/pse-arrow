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
#[cfg(test)]
mod incremental_tests;
mod local;
mod rebind;
pub(crate) use local::{AdmittedModelRevision, LocalSourcePremises};
use local::{
    LocalInventory, LocalSourceObject, SourceMember, TypedObjectPayload, TypedSourceInputs,
};
pub(crate) mod reload;

fn checkpoint(cancel: &crate::CancelSource) -> Result<(), WorkflowError> {
    if cancel.token().is_cancelled() {
        Err(crate::math::MathRuntimeError::Cancelled.into())
    } else if cancel
        .deadline()
        .is_some_and(|deadline| std::time::Instant::now() >= deadline)
    {
        Err(
            crate::math::MathRuntimeError::Solve(pse_backend_native::ProblemError::Limit {
                kind: pse_backend_native::LimitKind::Time,
                detail: "canonical source operation deadline".into(),
            })
            .into(),
        )
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
    delta_products: Arc<std::sync::OnceLock<Arc<delta::Products>>>,
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
            let mut typed = TypedSourceInputs::default();
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
                    let parsed = parsed_fields(runtime, &row.0, row.1.as_ref())?;
                    typed.decoded(object, row.clone(), parsed.clone())?;
                    index.parsed_fields.insert(row.0.declaration_id, parsed);
                    index.complete_scope(row.0.declaration_id);
                    index.complete_imports(row.0.declaration_id);
                    index.insert(row)?;
                }
            }
            index.requests.clear();
            for object in &mut imported {
                if object.logical.starts_with("modeling:")
                    && object.logical != CONTEXT_LOGICAL
                    && object.logical != PHYSICAL_SCOPE_LOGICAL
                {
                    let (row, _) = typed
                        .source(&object.logical)
                        .ok_or_else(|| contract("imported typed source absent"))?;
                    object.references = index
                        .references(row)?
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
            let admitted = AdmittedModelRevision::apply_inputs(
                runtime,
                None,
                &imported,
                &typed,
                Some(&cancel),
            )?;
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
                    delta_products: Default::default(),
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
            delta_products: Default::default(),
        })
    }
}

type FieldSpans = BTreeMap<String, SourceSpan>;
type SourcePayload = (Declaration, Option<FieldSpans>);

trait LocalSourceRow: Clone {
    fn carried(_source: &TypedObjectPayload) -> Option<&Self> {
        None
    }
}
macro_rules! carried_source_row {
    ($variant:ident, $row:ty) => {
        impl LocalSourceRow for $row {
            fn carried(source: &TypedObjectPayload) -> Option<&Self> {
                match source {
                    TypedObjectPayload::$variant(row) => Some(row.as_ref()),
                    _ => None,
                }
            }
        }
    };
}
carried_source_row!(Fit, pse_model::generated::authored::fit_cases::Row);
carried_source_row!(Document, pse_model::generated::authored::documents::Row);
carried_source_row!(Package, pse_model::generated::authored::packages::Row);
impl LocalSourceRow for IngressFrame {}
impl LocalSourceRow for PhysicalScope {}
impl LocalSourceRow for DeclarationId {}

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
pub(super) struct CanonicalIngress<'a> {
    pub(super) rows: Vec<Declaration>,
    pub(super) physical: &'a super::PhysicalContext,
    pub(super) physical_scope: PhysicalScope,
    pub(super) documents: &'a pse_modeling::document::DocumentInventory,
    pub(super) sources: &'a pse_admission::authoring_driver::document::Batches,
    pub(super) fits: &'a super::super::fitting::FitDeclarations,
    pub(super) loader: Option<&'a pse_admission::authoring_driver::document::OwnedDocumentSet>,
}
pub(super) async fn persist(
    runtime: &Runtime,
    ingress: CanonicalIngress<'_>,
    previous: Option<&SourceRevision>,
    cancel: &crate::CancelSource,
) -> Result<SourceRevision, WorkflowError> {
    persist_with_project(runtime, ingress, previous, cancel, None).await
}
pub(super) async fn persist_with_project(
    runtime: &Runtime,
    ingress: CanonicalIngress<'_>,
    previous: Option<&SourceRevision>,
    cancel: &crate::CancelSource,
    project: Option<&pse_operations::canonical_projects::ProjectPublication>,
) -> Result<SourceRevision, WorkflowError> {
    checkpoint(cancel)?;
    let CanonicalIngress {
        rows,
        physical,
        physical_scope,
        documents,
        sources,
        fits,
        loader,
    } = ingress;
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
        index.complete_scope(row.declaration_id);
        index.complete_imports(row.declaration_id);
    }
    index.requests.clear();
    let mut edits = Vec::with_capacity(rows.len() + sources.len() + fits.fits.len() + 1);
    let mut typed = TypedSourceInputs::default();
    for row in &rows {
        let references = index
            .references(row)?
            .into_iter()
            .map(|id| {
                let target = &index.rows[&id];
                (scope(target.parent_id), target.name.clone())
            })
            .collect();
        edits.push(typed.declaration(
            row,
            documents.field_spans.get(&row.declaration_id).cloned(),
            index.parsed_fields[&row.declaration_id].clone(),
            references,
        )?);
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
        let data = runtime
            .shared
            .math()
            .modeling_revision(
                &workspace,
                data_rows,
                physical_scope.clone(),
                Arc::new(data_documents),
                &physical.key,
                cancel,
            )
            .await?;
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
            edits.push(typed.package(row)?);
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
            edits.push(typed.document(row)?);
        }
    }
    for row in &fits.fits {
        edits.push(typed.fit(row.clone())?);
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
        if project.is_none() && edits.is_empty() && identity == previous.identity {
            return Ok(previous.clone());
        }
    }
    checkpoint(cancel)?;
    admit_revision_authorized(
        runtime,
        previous,
        identity,
        &edits,
        &typed,
        Some(cancel),
        project,
    )
    .await
}

/// Stage complete local ownership before issuing the existing durable effect.
async fn admit_revision(
    runtime: &Runtime,
    previous: Option<&SourceRevision>,
    identity: SourceRevisionHash,
    edits: &[ObjectEdit],
    typed: &TypedSourceInputs,
    cancel: Option<&crate::CancelSource>,
) -> Result<SourceRevision, WorkflowError> {
    admit_revision_authorized(runtime, previous, identity, edits, typed, cancel, None).await
}
async fn admit_revision_authorized(
    runtime: &Runtime,
    previous: Option<&SourceRevision>,
    identity: SourceRevisionHash,
    edits: &[ObjectEdit],
    typed: &TypedSourceInputs,
    cancel: Option<&crate::CancelSource>,
    project: Option<&pse_operations::canonical_projects::ProjectPublication>,
) -> Result<SourceRevision, WorkflowError> {
    let admitted = AdmittedModelRevision::apply_inputs(
        runtime,
        previous.map(|p| p.admitted.as_ref()),
        edits,
        typed,
        cancel,
    )?;
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
            if let Some(cancel) = cancel {
                checkpoint(cancel)?;
            }
            let recorded = if let Some(intent) = project {
                store.edit_project(intent, edits).await.map_err(|source| {
                    crate::workflow::source_settlement::source_edit_error(
                        intent.problem().into(),
                        intent.operation().into(),
                        source,
                    )
                })?
            } else {
                store
                    .edit(&problem, old.map(|old| old.key.as_str()), &operation, edits)
                    .await
                    .map_err(|source| {
                        crate::workflow::source_settlement::source_edit_error(
                            problem, operation, source,
                        )
                    })?
            };
            Some(recorded)
        }
    };
    SourceRevision::admitted(admitted, identity, origin, previous)
}

/// Publish the already admitted immutable result through the sole canonical edit path.
/// Receiving admission belongs to the ordinary typed admission owner; this helper cannot accept raw edits.
pub(super) async fn publish_admitted_project(
    runtime: &Runtime,
    intent: &pse_operations::canonical_projects::ProjectPublication,
    parent: &ModelingPackage,
    admitted: &ModelingPackage,
    cancel: &crate::CancelSource,
) -> Result<ModelingPackage, WorkflowError> {
    intent.validate()?;
    if intent.expected() != Some(parent.recorded_revision()?.key.as_str())
        || parent.physical.key != admitted.physical.key
    {
        return Err(contract("project publication exact parent or physical context differs"));
    }
    let mut edits = Vec::new();
    for object in admitted.revision.admitted.objects.values() {
        if parent.revision.admitted.objects.get(&object.logical).is_none_or(|old| old.value != object.value) {
            edits.push(object.edit());
        }
    }
    for old in parent.revision.admitted.objects.values() {
        if !admitted.revision.admitted.objects.contains_key(&old.logical) {
            edits.push(ObjectEdit { logical: old.logical.clone(), scope: old.scope.clone(), name: old.name.clone(), version: None, references: Vec::new() });
        }
    }
    checkpoint(cancel)?;
    let recorded = runtime.canonical_store()?.edit_project(intent, &edits).await.map_err(|source| {
        crate::workflow::source_settlement::source_edit_error(intent.problem().into(), intent.operation().into(), source)
    })?;
    let revision = SourceRevision::admitted(admitted.revision.admitted.clone(), admitted.revision.identity, Some(recorded), Some(&parent.revision))?;
    Ok(ModelingPackage { runtime: runtime.clone(), revision, held_preparation: None, project_context: None, ..admitted.clone() })
}

/// Active eligibility belongs to this call, never to retained admission.
#[derive(Debug)]
pub(super) struct CheckedSelection {
    pub(super) admission: Arc<super::SelectedAdmission>,
    source: Arc<AdmittedModelRevision>,
    physical: super::PhysicalContext,
    providers: Arc<BTreeMap<String, pse_kernels::Registration>>,
    runtime: Runtime,
    validation: Arc<pse_relations::validate::ValidationContext>,
    _metadata: Arc<pse_columnar::AllocationLease>,
}
impl CheckedSelection {
    fn new(
        package: &ModelingPackage,
        admission: Arc<super::SelectedAdmission>,
        cancel: &crate::CancelSource,
    ) -> Result<Self, WorkflowError> {
        checkpoint(cancel)?;
        let metadata = package.runtime.shared.math().reserve(
            "modeling:checked-selection-relation",
            package
                .physical
                .metadata_bytes()
                .checked_add(size_of::<Self>() + 512)
                .ok_or_else(|| contract("checked source relation metadata extent"))?,
        )?;
        let validation = package.runtime.admission_context();
        let physical =
            package
                .physical
                .readmit_context(&package.runtime, &validation, &cancel.token())?;
        let spec = package
            .runtime
            .registry()
            .relation("authored.modeling_declarations")
            .ok_or_else(|| contract("selected declaration relation absent"))?;
        let prepared = validation
            .relation(package.runtime.registry(), spec)
            .map_err(super::super::relation)?;
        if prepared.has_scope_checks() {
            let bytes = admission
                .revision
                .declarations()
                .iter()
                .try_fold(8192usize, |n, row| {
                    n.checked_add(row.owned_bytes().checked_mul(32)?)
                })
                .ok_or_else(|| contract("selected scoped declaration extent"))?;
            let _scratch = package
                .runtime
                .shared
                .math()
                .reserve("modeling:selected-scoped-rows", bytes)?;
            let mut builder =
                pse_relations::generated::authored::modeling_declarations::Builder::with_registry(
                    package.runtime.registry(),
                    admission.revision.declarations().len(),
                    &validation,
                )
                .map_err(super::super::relation)?;
            for row in admission.revision.declarations() {
                checkpoint(cancel)?;
                builder.push(row.clone()).map_err(super::super::relation)?;
            }
            builder.finish().map_err(super::super::relation)?;
        }
        checkpoint(cancel)?;
        Ok(Self {
            admission,
            source: package.revision.admitted.clone(),
            physical,
            providers: package.providers.clone(),
            runtime: package.runtime.clone(),
            validation,
            _metadata: metadata,
        })
    }
    pub(super) fn carries(&self, package: &ModelingPackage) -> bool {
        Arc::ptr_eq(&self.source, &package.revision.admitted)
            && self.physical.same_sources(&package.physical)
            && Arc::ptr_eq(&self.providers, &package.providers)
            && Arc::ptr_eq(self.runtime.sessions(), package.runtime.sessions())
            && Arc::ptr_eq(self.runtime.registry(), package.runtime.registry())
            && self
                .validation
                .admission_scope()
                .same_invocation(package.runtime.admission_context().admission_scope())
            && self.admission.request.physical_owners
                == (
                    Arc::as_ptr(&package.physical.quantities) as usize,
                    Arc::as_ptr(&package.physical.preconditions) as usize,
                )
            && self.admission.request.physical == package.physical.key
            && self.admission.request.providers.len() == package.providers.len()
            && self
                .admission
                .request
                .providers
                .iter()
                .zip(package.providers.iter())
                .all(|((name, key, configuration), (actual_name, actual))| {
                    name == actual_name
                        && *key == actual.spec().key()
                        && *configuration == actual.configuration_key()
                })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Request {
    Logical(DeclarationId),
    Name(Option<DeclarationId>, String),
    Imports(DeclarationId),
    Members(DeclarationId),
}

#[derive(Default)]
struct ReferenceTrace(std::sync::Mutex<Option<BTreeSet<Request>>>);
impl ReferenceTrace {
    fn record(&self, request: Request) {
        if let Some(reads) = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_mut()
        {
            reads.insert(request);
        }
    }
    fn read(&self, owner: DeclarationId) {
        self.record(Request::Logical(owner));
    }
    fn name(&self, parent: Option<DeclarationId>, name: &str) {
        self.record(Request::Name(parent, name.into()));
    }
    fn active(&self) -> bool {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
    }
}

#[derive(Default)]
struct SourceIndex {
    trace: ReferenceTrace,
    rows: BTreeMap<DeclarationId, Declaration>,
    fields: BTreeMap<DeclarationId, FieldSpans>,
    parsed_fields:
        BTreeMap<DeclarationId, Arc<pse_modeling::expression::occurrences::ParsedFields>>,
    children: BTreeMap<DeclarationId, Vec<DeclarationId>>,
    named_children: BTreeMap<DeclarationId, BTreeMap<String, Vec<DeclarationId>>>,
    named_roots: BTreeMap<String, Vec<DeclarationId>>,
    guards: BTreeMap<DeclarationId, Vec<DeclarationId>>,
    imported_names: BTreeMap<DeclarationId, BTreeMap<String, Vec<String>>>,
    names: BTreeMap<Option<DeclarationId>, BTreeSet<String>>,
    imports: BTreeSet<DeclarationId>,
    scopes: BTreeSet<DeclarationId>,
    requests: BTreeSet<Request>,
    new_rows: BTreeSet<DeclarationId>,
    pending_references: BTreeMap<DeclarationId, selected_source::DeclarationReferences>,
    probing: BTreeMap<DeclarationId, BTreeSet<String>>,
    member_memo: BTreeMap<DeclarationId, BTreeMap<String, Lookup>>,
    bases: BTreeMap<DeclarationId, Arc<[String]>>,
    roots_complete: bool,
    versions: BTreeMap<String, String>,
    record_sources: BTreeMap<DeclarationId, DeclarationId>,
}
struct LocalSelection {
    index: SourceIndex,
    physical: PhysicalScope,
    documents: pse_modeling::document::DocumentInventory,
    leases: Vec<Arc<pse_columnar::AllocationLease>>,
    dependencies: LocalSourcePremises,
}
impl SourceIndex {
    fn invalidate_members(&mut self) {
        self.member_memo.clear();
    }
    #[cfg(test)]
    fn complete_roots(&mut self) {
        if !self.roots_complete {
            self.roots_complete = true;
            self.invalidate_members();
        }
    }
    fn complete_scope(&mut self, owner: DeclarationId) {
        if self.scopes.insert(owner) {
            self.invalidate_members();
        }
    }
    fn complete_imports(&mut self, owner: DeclarationId) {
        if self.imports.insert(owner) {
            self.invalidate_members();
        }
    }
    fn complete_name(&mut self, parent: Option<DeclarationId>, name: String) {
        if self.names.entry(parent).or_default().insert(name) {
            self.invalidate_members();
        }
    }
    fn has_name(&self, parent: Option<DeclarationId>, name: &str) -> bool {
        self.names
            .get(&parent)
            .is_some_and(|names| names.contains(name))
    }
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
        self.invalidate_members();
        if let Some(scope) = &row.value.scope {
            self.bases.insert(id, scope.bases.clone().into());
        }
        if let Some(parent) = row.parent_id {
            self.children.entry(parent).or_default().push(id);
            self.named_children
                .entry(parent)
                .or_default()
                .entry(row.name.clone())
                .or_default()
                .push(id);
            if row.value.kind == Kind::When {
                self.guards.entry(parent).or_default().push(id);
            }
            if let Some(import) = &row.value.import {
                self.imported_names
                    .entry(parent)
                    .or_default()
                    .entry(import.alias.as_deref().unwrap_or(&row.name).into())
                    .or_default()
                    .push(row.name.clone());
            }
            if !self.rows.contains_key(&parent) {
                self.requests.insert(Request::Logical(parent));
            }
        } else {
            // The old row-map scan yielded declaration order, independently of
            // source acquisition order. Keep that contract in each indexed bucket.
            let roots = self.named_roots.entry(row.name.clone()).or_default();
            if let Err(position) = roots.binary_search(&id) {
                roots.insert(position, id);
            }
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
                    Some(parent) => self
                        .named_children
                        .get(&parent)
                        .and_then(|names| names.get(row.name.as_str())),
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
        !self.has_name(parent, name)
            && !parent.is_some_and(|parent| self.scopes.contains(&parent))
            && !(parent.is_none() && self.roots_complete)
    }
    fn type_bound(&self, mut owner: DeclarationId, name: &str) -> bool {
        let mut seen = BTreeSet::new();
        while seen.insert(owner) {
            self.trace.read(owner);
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
        self.trace.read(owner);
        self.trace.name(Some(owner), name);
        self.trace.record(Request::Members(owner));
        let mut ready = true;
        found.extend(
            self.named_children
                .get(&owner)
                .and_then(|names| names.get(name))
                .into_iter()
                .flatten()
                .copied(),
        );
        let mut position = 0;
        while let Some(child) = self
            .guards
            .get(&owner)
            .and_then(|ids| ids.get(position))
            .copied()
        {
            position += 1;
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
            && !self.has_name(Some(owner), name)
            && !self.scopes.contains(&owner)
        {
            self.requests
                .insert(Request::Name(Some(owner), name.into()));
            return Lookup::Pending;
        }
        if let Some(own) = self
            .named_children
            .get(&owner)
            .and_then(|names| names.get(name))
            && !own.is_empty()
        {
            return Lookup::Found(own.clone());
        }
        // Shared immutable names avoid copying inherited spellings during recursion.
        let bases = self.bases.get(&owner).cloned().unwrap_or_default();
        let mut found = BTreeSet::new();
        let mut ready = self.guarded(owner, name, &mut found);
        for base in bases.iter() {
            match self.resolve_name(owner, base) {
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
        self.trace.read(id);
        self.rows.get(&id)
    }
    fn bound(&self, owner: DeclarationId, name: &str) -> bool {
        self.trace.read(owner);
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
        self.trace.read(owner);
        self.trace.record(Request::Imports(owner));
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
            .get(&owner)
            .and_then(|names| names.get(name))
            .map_or(&[][..], Vec::as_slice);
        if imports.is_empty() {
            return Lookup::Absent;
        }
        let mut found = Vec::new();
        for import in imports {
            self.trace.name(None, import);
            if !self.roots_complete && !self.has_name(None, import) {
                self.requests.insert(Request::Name(None, import.clone()));
                return Lookup::Pending;
            }
            let candidates = self.named_roots.get(import).map_or(&[][..], Vec::as_slice);
            if let Some(context) = pse_diagnostics::observation::current() {
                context.count(
                    "modeling.import_candidates",
                    "declarations",
                    candidates.len() as u64,
                );
            }
            found.extend_from_slice(candidates);
        }
        if found.is_empty() {
            Lookup::Absent
        } else {
            Lookup::Found(found)
        }
    }
    fn member(&mut self, owner: DeclarationId, name: &str) -> Lookup {
        self.trace.read(owner);
        self.trace.name(Some(owner), name);
        let root = self.probing.is_empty();
        if root
            && !self.trace.active()
            && let Some(result) = self
                .member_memo
                .get(&owner)
                .and_then(|names| names.get(name))
        {
            if let Some(context) = pse_diagnostics::observation::current() {
                context.count("modeling.member_memo_hits", "lookups", 1);
            }
            return result.clone();
        }
        if self
            .probing
            .get(&owner)
            .is_some_and(|names| names.contains(name))
        {
            return Lookup::Absent;
        }
        self.probing.entry(owner).or_default().insert(name.into());
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("modeling.member_expansions", "lookups", 1);
        }
        let result = self.member_candidates(owner, name);
        if let Some(names) = self.probing.get_mut(&owner) {
            names.remove(name);
            if names.is_empty() {
                self.probing.remove(&owner);
            }
        }
        // A recursive result (even positive) can depend on the active cut. Pending
        // is an acquisition obligation, never a completed negative observation.
        if root && result != Lookup::Pending {
            self.member_memo
                .entry(owner)
                .or_default()
                .insert(name.into(), result.clone());
        }
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
    fn source_decode<T: serde::de::DeserializeOwned + LocalSourceRow>(
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
        let value = object
            .typed
            .as_ref()
            .and_then(T::carried)
            .cloned()
            .map(Ok)
            .unwrap_or_else(|| decode(&object.payload))?;
        Ok((value, owner))
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
        if let Some(missing) = selected
            .iter()
            .find(|id| !rows.iter().any(|row| row.declaration_id == **id))
        {
            return Err(contract(format!("selected declaration absent: {missing}")));
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
    pub(super) fn selection_request(&self, roots: &[DeclarationId]) -> Arc<super::SelectedRequest> {
        Arc::new(super::SelectedRequest {
            roots: roots.to_vec(),
            physical: self.physical.key,
            physical_owners: (
                Arc::as_ptr(&self.physical.quantities) as usize,
                Arc::as_ptr(&self.physical.preconditions) as usize,
            ),
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
        })
    }
    pub(super) async fn checked_selection(
        &self,
        roots: &[DeclarationId],
        cancel: &crate::CancelSource,
    ) -> Result<CheckedSelection, WorkflowError> {
        checkpoint(cancel)?;
        let math = self.runtime.shared.math();
        let request = self.selection_request(roots);
        if let Some(candidates) = math.modeling_cache.selected(&request) {
            for admission in candidates.rchunks(8).flat_map(|window| window.iter().rev()) {
                checkpoint(cancel)?;
                if admission
                    .dependencies
                    .matches_scoped(&self.revision.admitted, cancel)?
                {
                    return CheckedSelection::new(self, admission.clone(), cancel);
                }
            }
        }
        let generation = math.modeling_cache.generation();
        let LocalSelection {
            index,
            physical,
            documents,
            leases,
            dependencies,
        } = self.select_local(roots, cancel)?;
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
        let producer_deadline = service.preparation_producer_deadline();
        let operation = math.selected_flights.load_owned(key, move |control| {
            population.run(async move {
                let producer_deadline = producer_deadline?;
                let worker_service = service.clone();
                let workspace = service.compiler_lane(lineage, inputs)?;
                let pending_workspace = workspace.clone();
                let admitted = service
                    .compiler_job(
                        workspace,
                        crate::math::ConstructionEstimate::Known(0),
                        control,
                        Some(producer_deadline),
                        move |authority, flag| {
                            let _sources = leases;
                            let _metadata = flight_metadata;
                            if flag.load(std::sync::atomic::Ordering::Acquire) {
                                return Err(crate::math::MathRuntimeError::Cancelled);
                            }
                            worker_service.modeling_revision_at(
                                authority,
                                rows,
                                physical,
                                Arc::new(documents),
                                &physical_key,
                                stamp,
                                flag.flag(),
                                &suppliers,
                            )
                        },
                    )
                    .await;
                if admitted.is_err() {
                    service.discard_uninitialized_compiler_lane(&pending_workspace);
                }
                let admitted = admitted
                    .map_err(crate::math::MathRuntimeError::preparation_producer_failure)?;
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
        CheckedSelection::new(self, admission, cancel)
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
    ) -> Result<LocalSelection, WorkflowError> {
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
                        index.complete_name(parent, name);
                        objects
                    }
                    Request::Members(owner) => {
                        let objects =
                            premises.inventory(source, LocalInventory::Scope(logical(owner)));
                        index.complete_scope(owner);
                        index.complete_imports(owner);
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
                        index.complete_imports(owner);
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
        Ok(LocalSelection {
            index,
            physical,
            documents,
            leases,
            dependencies: premises,
        })
    }
}

/// A bounded counting sink; retained Arrow buffers are borrowed by StreamWriter.
struct ReviewSourceIpcExtent {
    written: usize,
    limit: usize,
}
impl std::io::Write for ReviewSourceIpcExtent {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.written = self
            .written
            .checked_add(bytes.len())
            .filter(|total| *total <= self.limit)
            .ok_or_else(|| {
                std::io::Error::other("selected review physical source exceeds finite allocation")
            })?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
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
    /// Bound only descriptive fields before cloning them. Admitted parse trees,
    /// typed inventories, physical registries and runtime owners are not exported.
    pub(in crate::workflow) fn review_source_extent(
        &self,
        maximum_bytes: usize,
    ) -> Result<usize, WorkflowError> {
        use datafusion::arrow::ipc::writer::StreamWriter;
        let mut extent = 1024_usize;
        if let Some(context) = &self.project_context {
            extent = extent.checked_add(crate::workflow::replay::encoded_extent(context, maximum_bytes)?)
                .filter(|bytes| *bytes <= maximum_bytes)
                .ok_or_else(|| contract("selected historical context exceeds finite allocation"))?;
        }
        {
            let mut add = |bytes: usize| -> Result<(), WorkflowError> {
                extent = extent
                    .checked_add(bytes)
                    .filter(|total| *total <= maximum_bytes)
                    .ok_or_else(|| contract("selected review source exceeds finite allocation"))?;
                Ok(())
            };
            for object in self.revision.admitted.objects.values() {
                // JSON text may escape every byte as six ASCII bytes; a byte-array
                // element needs at most three digits plus a delimiter. Fixed headers
                // also cover owned String/Vec/reference metadata in the copied object.
                let strings = [
                    &object.logical,
                    &object.scope,
                    &object.name,
                    &object.version,
                    &object.kind,
                ]
                .into_iter()
                .fold(0_usize, |bytes, value| bytes.saturating_add(value.len()));
                add(512_usize
                    .saturating_add(strings.saturating_mul(6))
                    .saturating_add(object.payload.len().saturating_mul(4)))?;
                for (scope, name) in &object.references {
                    add(128_usize
                        .saturating_add(scope.len().saturating_add(name.len()).saturating_mul(6)))?;
                }
            }
            for (name, provider) in self.providers.iter() {
                let spec = provider.spec();
                add(1024_usize
                    .saturating_add(name.len().saturating_mul(6))
                    .saturating_add(
                        spec.inputs
                            .len()
                            .saturating_add(spec.outputs.len())
                            .saturating_mul(256),
                    ))?;
                for shape in spec.shapes.inputs.iter().chain(&spec.shapes.outputs) {
                    add(512_usize.saturating_add(
                        shape
                            .axes
                            .len()
                            .saturating_add(shape.cells.len())
                            .saturating_mul(64),
                    ))?;
                    for coordinates in &shape.coordinates {
                        add(128_usize.saturating_add(coordinates.len().saturating_mul(64)))?;
                    }
                }
            }
        }
        // Count the original IPC stream without allocating a Vec of encoded data.
        // StreamWriter forwards the admitted arrays to this finite sink; no source
        // reconstruction, numerical preparation or provider callback is involved.
        for batch in self.physical.sources.values() {
            let remaining = maximum_bytes
                .checked_sub(extent)
                .and_then(|bytes| bytes.checked_sub(128))
                .ok_or_else(|| contract("selected review source exceeds finite allocation"))?;
            let mut sink = ReviewSourceIpcExtent {
                written: 0,
                limit: remaining / 4,
            };
            {
                let mut writer = StreamWriter::try_new(&mut sink, &batch.batch().schema())
                    .map_err(|error| contract(format!("review physical source extent: {error}")))?;
                writer
                    .write(batch.batch())
                    .map_err(|error| contract(format!("review physical source extent: {error}")))?;
                writer
                    .finish()
                    .map_err(|error| contract(format!("review physical source extent: {error}")))?;
            }
            extent = extent
                .checked_add(128 + sink.written * 4)
                .ok_or_else(|| contract("selected review source extent overflow"))?;
        }
        if extent > maximum_bytes {
            return Err(contract("selected review source exceeds finite allocation"));
        }
        Ok(extent)
    }
    /// Record retained immutable object edits in the receiving canonical store.
    pub(in crate::workflow) async fn record_study_source(
        &self,
        receiver: &Runtime,
        cancel: &crate::CancelSource,
    ) -> Result<String, WorkflowError> {
        if let Some(selected) = &self.project_context {
            receiver.canonical_store()?.validate_project_context(selected).await?;
            if selected.context.source_identity != self.revision.identity().to_string()
                || selected.context.physical_context != self.physical.identity().to_string()
            { return Err(contract("historical study source no longer matches frozen context")); }
            // Keep the original canonical source, rather than restaging an identical copy
            // and losing the exact retained context-to-source relationship.
            return Ok(selected.context.revision.clone());
        }
        let extent = self.revision.admitted.retained_bytes();
        let _owner = receiver
            .shared
            .math()
            .reserve("study:model-recording", extent)?;
        let edits = self
            .revision
            .admitted
            .objects
            .values()
            .map(|object| ObjectEdit {
                logical: object.logical.clone(),
                scope: object.scope.clone(),
                name: object.name.clone(),
                version: Some(pse_model::generated::runtime::canonical_versions::Row {
                    key: object.version.clone(),
                    logical: object.logical.clone(),
                    kind: object.kind.clone(),
                    payload: object.payload.clone().into(),
                    interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
                }),
                references: object.references.clone(),
            })
            .collect::<Vec<_>>();
        let problem = pse_operations::mint_id::<SemanticId>().to_string();
        let operation = pse_operations::mint_id::<SemanticId>().to_string();
        let receipt = crate::workflow::study::creation_effect(cancel, async {
            receiver
                .canonical_store()?
                .edit(&problem, None, &operation, &edits)
                .await
                .map_err(|source| {
                    crate::workflow::source_settlement::source_edit_error(
                        problem.clone(),
                        operation.clone(),
                        source,
                    )
                })
        })
        .await?;
        Ok(receipt.key)
    }

    /// Publish one selected immutable variant in its own occurrence-qualified source
    /// problem. The admitted base's durable and local heads are never advanced.
    pub(in crate::workflow) async fn record_study_variant(
        &self,
        authority: &pse_operations::canonical_studies::StudyAdmissionAuthority,
        cancel: &crate::CancelSource,
    ) -> Result<Self, WorkflowError> {
        let point = authority.point_key();
        if point.is_empty() {
            return Err(contract("variant publication requires its occurrence"));
        }
        let problem = format!("study-variant:{point}");
        let operation = format!("study-variant-source:{point}");
        let extent = self.revision.admitted.retained_bytes();
        let _owner = self
            .runtime
            .shared
            .math()
            .reserve("study:variant-source-publication", extent)?;
        let edits = self
            .revision
            .admitted
            .objects
            .values()
            .map(|object| ObjectEdit {
                logical: object.logical.clone(),
                scope: object.scope.clone(),
                name: object.name.clone(),
                version: Some(pse_model::generated::runtime::canonical_versions::Row {
                    key: object.version.clone(),
                    logical: object.logical.clone(),
                    kind: object.kind.clone(),
                    payload: object.payload.clone().into(),
                    interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
                }),
                references: object.references.clone(),
            })
            .collect::<Vec<_>>();
        let receipt = crate::workflow::study::creation_effect(cancel, async {
            self.runtime
                .canonical_store()?
                .edit_with_study_admission(&problem, None, &operation, &edits, authority)
                .await
                .map_err(|source| {
                    crate::workflow::source_settlement::source_edit_error(
                        problem.clone(),
                        operation.clone(),
                        source,
                    )
                })
        })
        .await?;
        let revision = SourceRevision::admitted(
            self.revision.admitted.clone(),
            self.revision.identity,
            Some(receipt),
            None,
        )?;
        Ok(Self {
            revision,
            held_preparation: None,
            ..self.clone()
        })
    }

    /// Export retained source facts without querying executable factory configuration.
    pub(in crate::workflow) fn review_source(
        &self,
    ) -> Result<crate::workflow::review_context::ReviewSourceDocument, WorkflowError> {
        use crate::workflow::{
            replay::ReplaySourceObject,
            review_context::{ReviewProviderDescriptor, ReviewSourceDocument},
        };
        Ok(ReviewSourceDocument {
            context: self.project_context.clone(),
            identity: self.revision.identity().as_id(),
            interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
            build: pse_backend_native::execution::LINKED.build_identity(),
            physical_identity: self.physical.identity(),
            physical_sources: crate::workflow::replay::physical_sources(&self.physical)?,
            providers: self
                .providers
                .iter()
                .map(|(name, provider)| ReviewProviderDescriptor::new(name, provider))
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
    /// Replay adds its stronger actual factory configuration premise to the shared source facts.
    pub(in crate::workflow) fn replay_source(
        &self,
    ) -> Result<crate::workflow::replay::ReplaySourceDocument, WorkflowError> {
        use crate::workflow::replay::{ReplayProvider, ReplaySourceDocument};
        let source = self.review_source()?;
        Ok(ReplaySourceDocument {
            context: self.project_context.clone(),
            identity: source.identity,
            interpretation: source.interpretation,
            build: source.build,
            physical_identity: source.physical_identity,
            physical_sources: source.physical_sources,
            objects: source.objects,
            providers: self
                .providers
                .iter()
                .map(|(name, provider)| ReplayProvider::new(name, provider))
                .collect(),
        })
    }
    /// Bind retained immutable inputs to an admitted receiver without a transport crossing.
    pub(in crate::workflow) fn reconstruct_replay_source(
        &self,
        runtime: &Runtime,
    ) -> Result<ModelingPackage, WorkflowError> {
        rebind::rebind(self, runtime)
    }
    /// Candidate only; all owners stay borrowed and exact premises are checked after lookup.
    pub(in crate::workflow) fn replay_source_owner_key(
        &self,
    ) -> (usize, usize, usize, usize, usize, usize) {
        (
            Arc::as_ptr(&self.revision.admitted) as usize,
            Arc::as_ptr(&self.physical.quantities) as usize,
            Arc::as_ptr(&self.physical.preconditions) as usize,
            Arc::as_ptr(&self.providers) as usize,
            Arc::as_ptr(&self.accelerators) as usize,
            Arc::as_ptr(&self.runtime.shared) as usize,
        )
    }
    /// Exact retained premises for operation-local coalescing, never a durable identity.
    pub(in crate::workflow) fn same_replay_source(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.revision.admitted, &other.revision.admitted)
            && self.project_context == other.project_context
            && self.physical.same_sources(&other.physical)
            && Arc::ptr_eq(&self.providers, &other.providers)
            && Arc::ptr_eq(&self.accelerators, &other.accelerators)
            && Arc::ptr_eq(&self.runtime.shared, &other.runtime.shared)
    }
}
impl Runtime {
    pub(in crate::workflow) fn admit_replay_source(
        &self,
        document: &crate::workflow::replay::ReplaySourceDocument,
        mut physical: super::PhysicalContext,
        providers: BTreeMap<String, pse_kernels::Registration>,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingPackage, WorkflowError> {
        checkpoint(cancel)?;
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
            .and_then(|bytes| bytes.checked_add(document.objects.len().checked_mul(256)?))
            .filter(|bytes| *bytes <= 256 * 1024 * 1024)
            .ok_or_else(|| {
                contract("complete replay premises exceed finite reconstruction allocation")
            })?;
        let _scratch = self
            .shared
            .math()
            .reserve("replay:source-reconstruction", bytes)?;
        let validation = self.admission_context();
        for (key, batch) in &mut physical.sources {
            checkpoint(cancel)?;
            let spec = self
                .registry()
                .relation_by_key(*key)
                .ok_or_else(|| contract("supplied replay physical relation absent"))?;
            *batch = batch
                .readmit_context(self.registry(), spec, &validation, &cancel.token())
                .map_err(super::super::relation)?;
        }
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
        let mut objects = BTreeMap::new();
        let mut names = BTreeSet::new();
        for object in &document.objects {
            checkpoint(cancel)?;
            if objects.insert(object.logical.as_str(), object).is_some()
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
        let context = objects
            .get(CONTEXT_LOGICAL)
            .ok_or_else(|| contract("replay source context absent"))?;
        if context.kind != CONTEXT
            || !objects.contains_key(PHYSICAL_SCOPE_LOGICAL)
            || !objects.contains_key(INGRESS_FRAME)
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
        let mut typed = TypedSourceInputs::default();
        for (object, edited) in document.objects.iter().zip(&edits) {
            checkpoint(cancel)?;
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
                let parsed = parsed_fields(self, &source.0, source.1.as_ref())?;
                typed.decoded(edited, source.clone(), parsed.clone())?;
                index.parsed_fields.insert(source.0.declaration_id, parsed);
                index.complete_scope(source.0.declaration_id);
                index.complete_imports(source.0.declaration_id);
                index.insert(source)?;
            }
        }
        let rows: Vec<_> = index.rows.values().cloned().collect();
        for row in &rows {
            checkpoint(cancel)?;
            let references: BTreeSet<_> = index
                .references(row)?
                .into_iter()
                .map(|id| {
                    let target = &index.rows[&id];
                    (scope(target.parent_id), target.name.clone())
                })
                .collect();
            let object = objects
                .get(logical(row.declaration_id).as_str())
                .ok_or_else(|| contract("replay declaration body absent"))?;
            if references != object.references.iter().cloned().collect() {
                return Err(contract(
                    "replay lexical dependency closure differs from actual source meaning",
                ));
            }
        }
        let frame_object = objects
            .get(INGRESS_FRAME)
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
        let physical_object = objects
            .get(PHYSICAL_SCOPE_LOGICAL)
            .ok_or_else(|| contract("replay physical scope absent"))?;
        let mut physical_scope: PhysicalScope = decode(&physical_object.payload)?;
        if let Some(documents) = &mut physical_scope.documents {
            for object in &document.objects {
                checkpoint(cancel)?;
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
            let data = objects
                .get(format!("data:{id}").as_str())
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
        checkpoint(cancel)?;
        let admitted = AdmittedModelRevision::apply_inputs(self, None, &edits, &typed, None)?;
        let revision = SourceRevision::admitted(admitted, identity, None, None)?;
        let mut package = self.package_from_revision(revision, physical, providers)?;
        if let Some(selected) = &document.context {
            if selected.context.source_identity != package.revision.identity().to_string()
                || selected.context.physical_context != package.physical.identity().to_string()
                || selected.context.project != selected.cut.project
                || selected.context.recorded_sequence > selected.cut.sequence
                || selected.context.effective_from_us > selected.effective_at_us
                || selected.context.effective_to_us.is_some_and(|to| to <= selected.effective_at_us)
                || selected.context.deleted
            { return Err(contract("portable context differs from actual source or original selection")); }
            // Portable replay preserves attribution; native context authority is checked again
            // by any later durable recording, not fabricated by embedded metadata.
            package.project_context = Some(selected.clone());
        }
        rebind::admit_rows(&package, self, &validation, cancel)?;
        checkpoint(cancel)?;
        Ok(package)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canonical_source_checkpoint_preserves_time_and_cancellation_causes() {
        let driver = crate::CancelSource::new().with_deadline(Some(
            std::time::Instant::now() - std::time::Duration::from_millis(1),
        ));
        assert!(matches!(
            checkpoint(&driver),
            Err(WorkflowError::Math(crate::math::MathRuntimeError::Solve(
                pse_backend_native::ProblemError::Limit {
                    kind: pse_backend_native::LimitKind::Time,
                    ..
                }
            )))
        ));
        assert!(!driver.token().is_cancelled());
        driver.cancel();
        assert!(matches!(
            checkpoint(&driver),
            Err(WorkflowError::Math(
                crate::math::MathRuntimeError::Cancelled
            ))
        ));
        checkpoint(&crate::CancelSource::new()).unwrap();
    }
    fn parse(source: &str) -> Vec<Declaration> {
        pse_authoring::language::parse(
            source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap()
    }
    fn indexed_root(name: &str, byte: u8) -> Declaration {
        let mut row = parse(&format!("package {name} {{}}"))[0].clone();
        row.declaration_id = DeclarationId::from(SemanticId::from_bytes([byte; 16]));
        row
    }

    #[test]
    fn indexed_imports_preserve_declaration_order_group_order_and_multiplicity() {
        let mut index = SourceIndex::default();
        let owner = indexed_root("owner", 20);
        let owner_id = owner.declaration_id;
        for row in [
            owner,
            indexed_root("first", 9),
            indexed_root("second", 7),
            indexed_root("first", 3),
        ] {
            index.insert((row, None)).unwrap();
        }
        index.complete_imports(owner_id);
        index.complete_roots();
        index.imported_names.insert(
            owner_id,
            BTreeMap::from([(
                "picked".into(),
                vec!["first".into(), "second".into(), "first".into()],
            )]),
        );
        let id = |byte| DeclarationId::from(SemanticId::from_bytes([byte; 16]));
        assert_eq!(
            index.imported(owner_id, "picked"),
            Lookup::Found(vec![id(3), id(9), id(7), id(3), id(9)])
        );
        assert!(index.requests.is_empty());
    }

    #[test]
    fn indexed_imports_keep_pending_distinct_from_absence_and_acquired_candidates() {
        let mut index = SourceIndex::default();
        let owner = indexed_root("owner", 20);
        let owner_id = owner.declaration_id;
        index.insert((owner, None)).unwrap();
        index
            .imported_names
            .entry(owner_id)
            .or_default()
            .insert("picked".into(), vec!["first".into()]);
        assert_eq!(index.imported(owner_id, "picked"), Lookup::Pending);
        assert!(index.requests.contains(&Request::Imports(owner_id)));
        index.complete_imports(owner_id);
        index.requests.clear();
        let first = indexed_root("first", 3);
        let first_id = first.declaration_id;
        index.insert((first, None)).unwrap();
        assert_eq!(index.imported(owner_id, "picked"), Lookup::Pending);
        assert!(
            index
                .requests
                .contains(&Request::Name(None, "first".into()))
        );
        index.complete_name(None, "first".into());
        assert_eq!(
            index.imported(owner_id, "picked"),
            Lookup::Found(vec![first_id])
        );
        index
            .imported_names
            .entry(owner_id)
            .or_default()
            .insert("empty".into(), vec!["absent".into()]);
        assert_eq!(index.imported(owner_id, "empty"), Lookup::Pending);
        index.complete_name(None, "absent".into());
        assert_eq!(index.imported(owner_id, "empty"), Lookup::Absent);
    }

    #[test]
    fn indexed_imports_examine_only_the_requested_root_bucket() {
        use pse_diagnostics::observation::{ObservationHub, Policy, Value};
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("source.import.lookup");
        let context = operation.context();
        let _scope = context.enter();
        let mut index = SourceIndex::default();
        let owner = indexed_root("owner", 200);
        let owner_id = owner.declaration_id;
        index.insert((owner, None)).unwrap();
        for byte in 1..100 {
            index
                .insert((indexed_root(&format!("unrelated{byte}"), byte), None))
                .unwrap();
        }
        let target = indexed_root("target", 201);
        let target_id = target.declaration_id;
        index.insert((target, None)).unwrap();
        index.complete_imports(owner_id);
        index.complete_roots();
        index
            .imported_names
            .entry(owner_id)
            .or_default()
            .insert("picked".into(), vec!["target".into()]);
        assert_eq!(
            index.imported(owner_id, "picked"),
            Lookup::Found(vec![target_id])
        );
        let snapshot = operation
            .reader()
            .snapshot(std::time::Duration::from_secs(60));
        let candidates = snapshot
            .metrics
            .iter()
            .find(|metric| metric.name == "modeling.import_candidates")
            .unwrap();
        assert_eq!(candidates.cumulative, Value::Count(1));
        assert_eq!(index.rows.len(), 101);
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
        index.complete_scope(root);
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

    fn complete_index(source: &str) -> SourceIndex {
        let mut index = SourceIndex::default();
        for row in parse(source) {
            let id = row.declaration_id;
            index.insert((row, None)).unwrap();
            index.complete_scope(id);
        }
        index.complete_roots();
        index.requests.clear();
        index
    }
    #[test]
    fn root_member_memo_preserves_cycles_diamonds_shadowing_and_frontier_changes() {
        let mut index = complete_index(
            "package p { interface A extends B,C {} interface B extends A {} interface C {let x:Scalar=1;} interface D extends A,C {} interface Own extends D {let x:Scalar=2;} }",
        );
        let id = |index: &SourceIndex, name: &str| {
            index
                .rows
                .values()
                .find(|r| r.name == name)
                .unwrap()
                .declaration_id
        };
        let (a, b, c, d, own) = (
            id(&index, "A"),
            id(&index, "B"),
            id(&index, "C"),
            id(&index, "D"),
            id(&index, "Own"),
        );
        let member = |index: &SourceIndex, parent| {
            index
                .rows
                .values()
                .find(|r| r.name == "x" && r.parent_id == Some(parent))
                .unwrap()
                .declaration_id
        };
        let inherited = member(&index, c);
        let direct = member(&index, own);
        assert_eq!(index.member(a, "x"), Lookup::Found(vec![inherited]));
        assert!(
            !index.member_memo.contains_key(&b),
            "nested cycle-cut absence is never retained"
        );
        assert_eq!(index.member(b, "x"), Lookup::Found(vec![inherited]));
        assert_eq!(
            index.member(d, "x"),
            Lookup::Found(vec![inherited]),
            "diamond deduplicates representatives"
        );
        assert_eq!(index.member(own, "x"), Lookup::Found(vec![direct]));
        for owner in [a, b, c, d, own] {
            let expected = index.member(owner, "x");
            index.invalidate_members();
            assert_eq!(index.member(owner, "x"), expected);
        }
        assert_eq!(index.member(c, "missing"), Lookup::Absent);
        let mut added = index.rows[&direct].clone();
        added.declaration_id = DeclarationId::from(SemanticId::from_bytes([223; 16]));
        added.parent_id = Some(c);
        added.name = "missing".into();
        let added_id = added.declaration_id;
        index.insert((added, None)).unwrap();
        assert!(index.member_memo.is_empty());
        assert_eq!(index.member(c, "missing"), Lookup::Found(vec![added_id]));
    }
    #[test]
    fn root_member_memo_never_consults_from_nested_context_or_retains_pending() {
        let mut index =
            complete_index("package p { interface A extends B {} interface B {let x:Scalar=1;} }");
        let a = index
            .rows
            .values()
            .find(|r| r.name == "A")
            .unwrap()
            .declaration_id;
        let b = index
            .rows
            .values()
            .find(|r| r.name == "B")
            .unwrap()
            .declaration_id;
        let x = index
            .rows
            .values()
            .find(|r| r.name == "x")
            .unwrap()
            .declaration_id;
        index.scopes.remove(&b);
        index.invalidate_members();
        assert_eq!(index.member(a, "x"), Lookup::Pending);
        assert!(index.member_memo.is_empty());
        index.complete_scope(b);
        assert_eq!(index.member(a, "x"), Lookup::Found(vec![x]));
        // A deliberately conflicting sentinel makes any nested consultation visible.
        index
            .member_memo
            .entry(b)
            .or_default()
            .insert("x".into(), Lookup::Absent);
        index.invalidate_members();
        index
            .member_memo
            .entry(b)
            .or_default()
            .insert("x".into(), Lookup::Absent);
        assert_eq!(index.member(a, "x"), Lookup::Found(vec![x]));
        assert_eq!(index.member(a, "missing"), Lookup::Absent);
        index.complete_name(Some(a), "new".into());
        assert!(index.member_memo.is_empty());
        index.member(a, "missing");
        index.imports.remove(&a);
        index.complete_imports(a);
        assert!(index.member_memo.is_empty());
    }
    #[test]
    fn root_member_memo_avoids_repeated_inherited_expansion_with_guard_representatives() {
        use pse_diagnostics::observation::{ObservationHub, Policy, Value};
        let mut index = complete_index(
            "package p { interface A {when true {let x:Scalar=1;} when false {let x:Scalar=2;}} interface B extends A {} }",
        );
        let b = index
            .rows
            .values()
            .find(|r| r.name == "B")
            .unwrap()
            .declaration_id;
        let mut expected = index
            .rows
            .values()
            .filter(|r| r.name == "x")
            .map(|r| r.declaration_id)
            .collect::<Vec<_>>();
        expected.sort();
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("source.member.memo");
        let context = operation.context();
        let _scope = context.enter();
        assert_eq!(index.member(b, "x"), Lookup::Found(expected.clone()));
        assert_eq!(index.member(b, "x"), Lookup::Found(expected));
        let snapshot = operation
            .reader()
            .snapshot(std::time::Duration::from_secs(60));
        let count = |name| {
            snapshot
                .metrics
                .iter()
                .find(|m| m.name == name)
                .unwrap()
                .cumulative
                .clone()
        };
        assert_eq!(count("modeling.member_memo_hits"), Value::Count(1));
        assert!(matches!(count("modeling.member_expansions"), Value::Count(n) if n >= 2));
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
    #[tokio::test(flavor = "current_thread")]
    async fn scoped_local_selection_inherits_parent_cancellation_before_native_entry() {
        let runtime = crate::workflow::tests::runtime();
        let rows = parse("package p {def Root {var x:Scalar;eq e:x==1;}}");
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(rows, crate::workflow::tests::physical())
            .await
            .unwrap();
        let service = runtime.shared.math();
        let (admitted, admission) = tokio::sync::oneshot::channel();
        let (exit, gate) = std::sync::mpsc::channel::<()>();
        let mut submission = crate::math::Submission::ephemeral();
        submission.admitted = Some(admitted);
        let blocker = service
            .submit_with(service.cores(), 0, submission, move |_, _| {
                gate.recv().map_err(|error| {
                    crate::math::MathRuntimeError::Infrastructure(error.to_string())
                })?;
                Ok(((), 0))
            })
            .unwrap();
        admission.await.unwrap();
        let parent = crate::CancelSource::new();
        let roots = [root];
        let mut selection = Box::pin(package.checked_selection_scoped(
            &roots,
            &parent,
            std::time::Instant::now() + std::time::Duration::from_secs(60),
        ));
        let queued = tokio::time::timeout(std::time::Duration::from_secs(10), async {
            loop {
                assert!(futures_util::poll!(selection.as_mut()).is_pending());
                if service.selected_flights.active() != 0 {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await;
        parent.cancel();
        let stopped =
            tokio::time::timeout(std::time::Duration::from_secs(10), selection.as_mut()).await;
        drop(selection);
        let drained = tokio::time::timeout(std::time::Duration::from_secs(10), async {
            while service.selected_flights.active() != 0 {
                tokio::task::yield_now().await;
            }
        })
        .await;
        exit.send(()).unwrap();
        blocker.finish().await.unwrap();
        queued.unwrap();
        drained.unwrap();
        assert!(matches!(
            stopped.unwrap(),
            Err(WorkflowError::Math(
                crate::math::MathRuntimeError::Cancelled
            ))
        ));
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
        let forged_owner = AdmittedModelRevision::apply(
            &runtime,
            Some(&package.revision.admitted),
            &[forged],
            None,
        )
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
    /// G1 composes real durable publication/execution with disconnected local replay.
    /// Requires the canonical fixture server and linked KINSOL; no storage fallback.
    #[cfg(all(feature = "canonical-tests", feature = "solver-kinsol"))]
    #[tokio::test]
    async fn solve_durable_replay_after_publication_close_requires_explicit_local_receiver() {
        use crate::workflow::{Durability, RunDurability, RunReport, RunResult};
        use pse_backend_native::solve::{Backend, SolverSelection};
        use pse_relations::generated::enums::NumericalTarget;

        fn assert_original_root(result: &RunResult) {
            assert!(result.usable(), "{:?}", result.report());
            assert_eq!(result.assessments().len(), 1);
            assert!(result.assessments()[0].permits_result);
            let RunReport::Modeling(steps) = result.report().unwrap() else {
                panic!("expected the original algebraic root result")
            };
            assert_eq!(steps.len(), 1);
            let step = &steps[0];
            assert!(step.accepted);
            let crate::math::solves::Outcome::Native(native) = &step.outcome else {
                panic!("the nonlinear equation requires actual native execution")
            };
            assert_eq!(native.backend, Backend::Kinsol);
            let model = &step.prepared.model.model.compiled().model;
            let variable = model
                .symbols
                .values()
                .find(|symbol| symbol.lineage.path.rsplit('.').next() == Some("x"))
                .unwrap()
                .id;
            let equation = model
                .equations
                .iter()
                .find(|row| row.lineage.path.rsplit('.').next() == Some("balance"))
                .unwrap()
                .id;
            let numerics = step.prepared.solve.numerics();
            let variable_budget = crate::workflow::tests::engineering_target(
                numerics,
                NumericalTarget::Variable,
                variable,
            )
            .budget;
            let residual_budget = crate::workflow::tests::engineering_target(
                numerics,
                NumericalTarget::Row,
                equation,
            )
            .budget;
            let x = step.values.scalars[&variable];
            // Independent expected positive root and original equation, not replayed output.
            assert!((x - 2.).abs() <= variable_budget, "x={x}");
            assert!((x * x - 4.).abs() <= residual_budget, "x={x}");
        }

        let runtime = crate::workflow::tests::durable_runtime();
        let rows = parse(
            "package p {def Root {param p:Scalar=4;var x:Scalar;eq balance:x*x==p;annotation start x(1);}}",
        );
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(rows, crate::workflow::tests::physical())
            .await
            .unwrap();
        assert!(package.canonical_revision().is_some());
        let prepared = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Default::default(),
                Default::default(),
                Default::default(),
                pse_kernels::DerivativeOrder::First,
                crate::workflow::tests::compiler_profile(),
                crate::math::solves::SolverProfile {
                    selection: SolverSelection::Explicit(Backend::Kinsol),
                    ..crate::workflow::tests::profile()
                },
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let original = prepared.start().unwrap().wait().await.unwrap();
        assert_original_root(&original);
        let RunDurability::Durable(record) = original.durability() else {
            panic!("the source run must acknowledge real durable recording")
        };
        assert!(record.attempt.is_ok(), "{:?}", record.attempt);
        assert!(record.manifest.is_some());
        assert!(original.canonical_run_key().is_some());
        assert!(original.canonical_attempt_key().is_some());
        let replay = original.replay();
        let document = replay.document().unwrap();
        assert_eq!(document.sources.len(), 1);
        assert_eq!(document.case_constructors.len(), 1);
        assert!(!document.native_attempts.is_empty());

        // Close the actual publication/recording owner, retaining its owned scientific closure.
        runtime.canonical_store().unwrap().close();
        let refusal = runtime
            .start_modeling(vec![prepared.clone()], false, &crate::CancelSource::new())
            .await
            .unwrap_err();
        assert!(
            matches!(
                refusal,
                WorkflowError::Canonical(pse_operations::canonical::CanonicalError::CallerClosed)
            ),
            "{refusal}"
        );
        // Synchronous refusal precedes a RunHandle/native dispatch; there is no success to wait for.
        assert!(matches!(runtime.durability(), Durability::Durable(_)));

        let local =
            Runtime::from_shared_local(runtime.shared.clone(), runtime.model.clone()).unwrap();
        assert!(matches!(local.durability(), Durability::Ephemeral));
        assert!(matches!(
            local.canonical(),
            Err(WorkflowError::RecordingUnavailable)
        ));
        let sources = replay.reconstruct_sources(&local).unwrap();
        assert_eq!(sources.len(), 1);
        assert!(sources[0].canonical_revision().is_none());
        assert_eq!(
            serde_json::to_value(sources[0].replay_source().unwrap()).unwrap(),
            serde_json::to_value(&document.sources[0]).unwrap(),
            "the receiver admits the complete exact retained source closure",
        );
        let restored = document.case_constructors[0]
            .reconstruct(&sources[0], &crate::CancelSource::new())
            .await
            .unwrap();
        assert_eq!(
            restored.solve.original_identity().unwrap(),
            prepared.solve.original_identity().unwrap()
        );
        let fresh = restored.start().unwrap().wait().await.unwrap();
        assert_ne!(fresh.run_id, original.run_id);
        assert_original_root(&fresh);
        assert!(
            !fresh
                .replay()
                .document()
                .unwrap()
                .native_attempts
                .is_empty()
        );
        assert!(matches!(fresh.durability(), RunDurability::Ephemeral));
        assert!(fresh.canonical_run_key().is_none());
        assert!(fresh.canonical_attempt_key().is_none());
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

#[cfg(test)]
mod plan39_selection_tests {
    use super::*;
    fn rows(value: u32) -> Vec<Declaration> {
        pse_authoring::language::parse(
            &format!("package p {{def Root {{param a:Scalar={value};var x:Scalar;eq e:x==a;}}}}"),
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap()
    }
    #[tokio::test]
    async fn plan39_checked_selection_carries_exact_source_context_and_invocation() {
        let runtime = crate::workflow::tests::runtime();
        let declarations = rows(1);
        let root = declarations
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(declarations, crate::workflow::tests::physical())
            .await
            .unwrap();
        runtime
            .scope_activity(async {
                let cancel = crate::CancelSource::new();
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
                let mut selected = package
                    .checked_selection_scoped(&[root], &cancel, deadline)
                    .await?;
                assert!(selected.carries(&package));
                let admission = selected.admission.clone();
                package
                    .prepare_selected_basis(
                        &mut selected,
                        root,
                        pse_modeling::specialize::root_instance(root),
                        Default::default(),
                        Default::default(),
                        &cancel,
                        deadline,
                    )
                    .await?;
                assert!(Arc::ptr_eq(&admission, &selected.admission));
                let cancelled = crate::CancelSource::new();
                cancelled.cancel();
                assert!(
                    package
                        .prepare_selected_basis(
                            &mut selected,
                            root,
                            pse_modeling::specialize::root_instance(root),
                            Default::default(),
                            Default::default(),
                            &cancelled,
                            deadline
                        )
                        .await
                        .is_err()
                );
                assert!(
                    package
                        .prepare_selected_basis(
                            &mut selected,
                            root,
                            pse_modeling::specialize::root_instance(root),
                            Default::default(),
                            Default::default(),
                            &cancel,
                            std::time::Instant::now() - std::time::Duration::from_secs(1)
                        )
                        .await
                        .is_err()
                );
                let mut foreign_physical = package.clone();
                foreign_physical.physical = crate::workflow::tests::physical();
                assert!(
                    !selected.carries(&foreign_physical),
                    "equal physical content is not the same owner"
                );
                let changed = package.with_declarations(rows(2)).await?;
                assert!(!selected.carries(&changed));
                changed
                    .prepare_selected_basis(
                        &mut selected,
                        root,
                        pse_modeling::specialize::root_instance(root),
                        Default::default(),
                        Default::default(),
                        &cancel,
                        deadline,
                    )
                    .await?;
                assert!(selected.carries(&changed));
                assert_ne!(admission.dependencies, selected.admission.dependencies);
                Ok::<_, WorkflowError>(())
            })
            .await
            .unwrap();
    }
    #[tokio::test]
    async fn plan39_checked_selection_new_invocation_reestablishes_relation() {
        let runtime = crate::workflow::tests::runtime();
        let declarations = rows(1);
        let root = declarations
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(declarations, crate::workflow::tests::physical())
            .await
            .unwrap();
        let mut selected = runtime
            .scope_activity(package.checked_selection(&[root], &crate::CancelSource::new()))
            .await
            .unwrap();
        runtime
            .scope_activity(async {
                assert!(!selected.carries(&package));
                let cancel = crate::CancelSource::new();
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
                package
                    .prepare_selected_basis(
                        &mut selected,
                        root,
                        pse_modeling::specialize::root_instance(root),
                        Default::default(),
                        Default::default(),
                        &cancel,
                        deadline,
                    )
                    .await?;
                assert!(selected.carries(&package));
                Ok::<_, WorkflowError>(())
            })
            .await
            .unwrap();
    }
}

#[cfg(test)]
mod review_source_extent_tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn physical_source_extent_sink_refuses_before_crossing_its_bound() {
        let mut sink = ReviewSourceIpcExtent {
            written: 0,
            limit: 3,
        };
        sink.write_all(&[1, 2, 3]).unwrap();
        assert_eq!(sink.written, 3);
        assert!(sink.write_all(&[4]).is_err());
        assert_eq!(sink.written, 3);
    }

    #[test]
    fn physical_source_extent_counts_original_ipc_and_refuses_a_short_allowance() {
        use datafusion::arrow::{
            array::{ArrayRef, Int32Array},
            datatypes::{DataType, Field, Schema},
            ipc::writer::StreamWriter,
            record_batch::RecordBatch,
        };
        let schema = Arc::new(Schema::new(vec![Field::new(
            "original",
            DataType::Int32,
            false,
        )]));
        let values: ArrayRef = Arc::new(Int32Array::from(vec![1, 2, 3]));
        let batch = RecordBatch::try_new(schema.clone(), vec![values]).unwrap();
        let mut actual = Vec::new();
        {
            let mut writer = StreamWriter::try_new(&mut actual, &schema).unwrap();
            writer.write(&batch).unwrap();
            writer.finish().unwrap();
        }
        let mut sink = ReviewSourceIpcExtent {
            written: 0,
            limit: actual.len(),
        };
        {
            let mut writer = StreamWriter::try_new(&mut sink, &schema).unwrap();
            writer.write(&batch).unwrap();
            writer.finish().unwrap();
        }
        assert_eq!(sink.written, actual.len());
        let mut short = ReviewSourceIpcExtent {
            written: 0,
            limit: actual.len() - 1,
        };
        let mut writer = StreamWriter::try_new(&mut short, &schema).unwrap();
        writer.write(&batch).unwrap();
        assert!(writer.finish().is_err());
    }

    #[tokio::test]
    async fn descriptive_source_extent_bounds_actual_bytes_without_runtime_inventory() {
        let runtime = crate::workflow::tests::runtime();
        let rows = pse_authoring::language::parse(
            "package review { def Root { var x:Scalar; eq balance:x==3; } }",
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let package = runtime
            .modeling_package(rows, crate::workflow::tests::physical())
            .await
            .unwrap();
        let extent = package.review_source_extent(16 << 20).unwrap();
        let source = package.review_source().unwrap();
        assert!(serde_json::to_vec(&source).unwrap().len() <= extent);
        assert!(package.review_source_extent(1).is_err());
        let exact_bound = package.review_source_extent(extent).unwrap();
        assert_eq!(exact_bound, extent);
        assert!(package.review_source_extent(extent - 1).is_err());
        // Broad replay admission owns parsed/typed/physical/runtime material that
        // this descriptive source never clones into the review document.
        assert!(extent < package.replay_source_extent());
    }
}
