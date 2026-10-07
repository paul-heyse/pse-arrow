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
    canonical_selection::{SELECTED_INVENTORY_BATCH, SelectedRead},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

fn checkpoint(cancel: &crate::CancelSource) -> Result<(), WorkflowError> {
    if cancel.token().is_cancelled() {
        Err(crate::math::MathRuntimeError::Cancelled.into())
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

/// Canonical revision identity and the original semantic source identity have
/// different meanings. Neither retains declarations or admitted package state.
#[derive(Clone, Debug)]
pub(in crate::workflow) struct SourceRevision {
    pub(in crate::workflow) canonical: Revision,
    identity: SourceRevisionHash,
}
impl SourceRevision {
    pub(in crate::workflow) fn identity(&self) -> SourceRevisionHash {
        self.identity
    }
    pub(super) async fn open(
        runtime: &Runtime,
        canonical: Revision,
    ) -> Result<Self, WorkflowError> {
        let store = runtime.canonical.store();
        let protection = store
            .protect(canonical.clone(), std::time::Duration::from_secs(3600))
            .await?;
        let mut read = SelectedRead::new(protection);
        let result = context(runtime, &mut read).await;
        let release = store.release(read.selection()).await;
        let identity = result?;
        release?;
        Ok(Self {
            canonical,
            identity,
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
    let store = runtime.canonical.store();
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
    let store = runtime.canonical.store();
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
        .canonical
        .store()
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
            .canonical
            .store()
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
    sources: &crate::authoring_driver::document::Batches,
    fits: &super::super::fitting::FitDeclarations,
    previous: Option<&SourceRevision>,
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
    let validation = runtime.validation_context()?;
    let mut builder =
        pse_relations::generated::authored::modeling_declarations::Builder::with_registry(
            &runtime.registry,
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
        let data = runtime.shared.math().modeling_revision(
            &workspace,
            data_rows,
            physical_scope.clone(),
            Arc::new(documents.clone()),
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
    if let Some(previous) = previous {
        let protection = runtime
            .canonical
            .store()
            .protect(
                previous.canonical.clone(),
                std::time::Duration::from_secs(3600),
            )
            .await?;
        let old: Result<_, WorkflowError> = async {
            // This explicit complete-source replacement needs only membership metadata.
            // Scientific preparation still resolves its selected closure separately.
            let mut old = BTreeMap::new();
            let mut after = String::new();
            loop {
                let scratch = runtime
                    .shared
                    .math()
                    .reserve("modeling:authoring-membership-page", 16 * 1024 * 1024)?;
                let page = runtime
                    .canonical
                    .store()
                    .membership_page(&protection, None, &after)
                    .await?;
                if page.is_empty() {
                    break;
                }
                let extent = page
                    .owned_bytes()
                    .checked_mul(2)
                    .ok_or_else(|| contract("authoring membership extent overflow"))?;
                let owners = scratch
                    .partition(&[extent])
                    .map_err(|_| contract("authoring membership page exceeds admitted extent"))?;
                membership_leases.extend(owners);
                after = page
                    .last()
                    .map(|member| member.key.clone())
                    .ok_or_else(|| contract("authoring membership page absent"))?;
                for member in page {
                    old.insert(member.logical.clone(), member);
                }
            }
            Ok(old)
        }
        .await;
        let release = runtime.canonical.store().release(&protection).await;
        let old = old?;
        release?;
        let retained = edits
            .iter()
            .map(|edit| edit.logical.clone())
            .collect::<BTreeSet<_>>();
        edits.retain(|edit| {
            old.get(&edit.logical).is_none_or(|member| {
                member.scope != edit.scope
                    || member.name != edit.name
                    || edit
                        .version
                        .as_ref()
                        .is_none_or(|version| member.version != version.key)
            })
        });
        // Complete edits replace declarations; data documents retain original ownership.
        edits.extend(
            old.into_values()
                .filter(|member| {
                    (member.logical.starts_with("modeling:")
                        || member.logical.starts_with("record:")
                        || member.logical.starts_with("physical_scope_document:"))
                        && !retained.contains(&member.logical)
                })
                .map(|member| ObjectEdit {
                    logical: member.logical,
                    scope: member.scope,
                    name: member.name,
                    version: None,
                    references: Vec::new(),
                }),
        );
        if edits.is_empty() && identity == previous.identity {
            return Ok(previous.clone());
        }
    }
    let problem = previous.map_or_else(
        || pse_operations::mint_id::<SemanticId>().to_string(),
        |previous| previous.canonical.problem.clone(),
    );
    let operation = pse_operations::mint_id::<SemanticId>().to_string();
    let revision = runtime
        .canonical
        .store()
        .edit(
            &problem,
            previous.map(|previous| previous.canonical.key.as_str()),
            &operation,
            &edits,
        )
        .await?;
    Ok(SourceRevision {
        canonical: revision,
        identity,
    })
}

/// One compiler input, resolved entirely from immutable canonical records. Read
/// premises remain protected until the preparation or portable publication ends.
pub(super) struct SelectedSource {
    pub(super) rows: Vec<Declaration>,
    pub(super) physical: PhysicalScope,
    pub(super) documents: Arc<pse_modeling::document::DocumentInventory>,
    pub(super) read: SelectedRead,
    pub(super) versions: BTreeMap<String, String>,
    record_sources: BTreeMap<DeclarationId, DeclarationId>,
    _leases: Vec<Arc<pse_columnar::AllocationLease>>,
}

pub(super) struct CancellationWatch(tokio::task::JoinHandle<()>);
impl Drop for CancellationWatch {
    fn drop(&mut self) {
        self.0.abort();
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
struct SourceIndex {
    rows: BTreeMap<DeclarationId, Declaration>,
    fields: BTreeMap<DeclarationId, FieldSpans>,
    children: BTreeMap<DeclarationId, Vec<DeclarationId>>,
    named_children: BTreeMap<(DeclarationId, String), Vec<DeclarationId>>,
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
                self.rows
                    .values()
                    .filter(|candidate| {
                        candidate.parent_id == row.parent_id && candidate.name == row.name
                    })
                    .map(|row| row.declaration_id),
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
                    self.rows
                        .values()
                        .filter(|target| target.parent_id.is_none() && target.name == row.name)
                        .map(|row| row.declaration_id),
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
        let occurrences = selected_source::occurrences(row, self.fields.get(&row.declaration_id))
            .map_err(super::super::modeling_error)?;
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
    pub(super) async fn declaration_inventory(
        &self,
    ) -> Result<super::OwnedDeclarations, WorkflowError> {
        let store = self.runtime.canonical.store();
        let protection = store
            .protect(
                self.revision.canonical.clone(),
                std::time::Duration::from_secs(3600),
            )
            .await?;
        let read = SelectedRead::new(protection);
        let result: Result<_, WorkflowError> = async {
            let mut rows = Vec::new();
            let mut leases = Vec::new();
            let mut after = String::new();
            loop {
                let page = store
                    .membership_page(read.selection(), None, &after)
                    .await?;
                let Some(last) = page.last() else {
                    break;
                };
                after = last.key.clone();
                for member in page.into_iter().filter(|member| {
                    member.logical.starts_with("modeling:")
                        && member.logical != CONTEXT_LOGICAL
                        && member.logical != PHYSICAL_SCOPE_LOGICAL
                }) {
                    let value = selected_object(&self.runtime, &read, &member.version).await?;
                    let (row, _): SourcePayload = decode(value.payload.as_slice())?;
                    leases.push(value.lease.clone());
                    rows.push(row);
                }
            }
            Ok(super::OwnedDeclarations {
                rows,
                _leases: leases,
            })
        }
        .await;
        let release = store.release(read.selection()).await;
        let result = result?;
        release?;
        Ok(result)
    }
    /// Explicit full fitting inventory, retaining each generated row decode lease.
    pub async fn fit_declarations(
        &self,
    ) -> Result<super::super::fitting::FitDeclarations, WorkflowError> {
        let store = self.runtime.canonical.store();
        let protection = store
            .protect(
                self.revision.canonical.clone(),
                std::time::Duration::from_secs(3600),
            )
            .await?;
        let mut read = SelectedRead::new(protection);
        let result: Result<_, WorkflowError> = async {
            let mut fits = Vec::new();
            let mut leases = Vec::new();
            for member in store.resolve_kind_scope(&mut read, None, "fit").await? {
                let value = selected_object(&self.runtime, &read, &member.version).await?;
                fits.push(decode(value.payload.as_slice())?);
                leases.push(value.lease.clone());
            }
            Ok(super::super::fitting::FitDeclarations::from_owned_rows(
                fits, leases,
            ))
        }
        .await;
        let release = store.release(read.selection()).await;
        let result = result?;
        release?;
        Ok(result)
    }
    pub(in crate::workflow) async fn fit_declaration(
        &self,
        id: pse_model::generated::identities::FitId,
    ) -> Result<pse_model::generated::authored::fit_cases::Row, WorkflowError> {
        let store = self.runtime.canonical.store();
        let protection = store
            .protect(
                self.revision.canonical.clone(),
                std::time::Duration::from_secs(3600),
            )
            .await?;
        let mut read = SelectedRead::new(protection);
        let result: Result<_, WorkflowError> = async {
            let value = object(&self.runtime, &mut read, &format!("fit:{id}"))
                .await?
                .ok_or_else(|| contract("unknown fitting declaration"))?;
            decode(value.payload.as_slice())
        }
        .await;
        let release = store.release(read.selection()).await;
        let result = result?;
        release?;
        Ok(result)
    }
    pub(in crate::workflow) async fn replace_fits(
        &self,
        fits: &[pse_model::generated::authored::fit_cases::Row],
    ) -> Result<SourceRevision, WorkflowError> {
        let extent = fits
            .iter()
            .try_fold(size_of::<Vec<Declaration>>(), |n, row| {
                n.checked_add(row.owned_bytes())
            })
            .and_then(|n| n.checked_mul(8))
            .ok_or_else(|| contract("canonical fitting edit extent"))?;
        let _scratch = self
            .runtime
            .shared
            .math()
            .reserve("modeling:canonical-fit-edit", extent)?;
        let validation = self.runtime.validation_context()?;
        let mut builder = pse_relations::generated::authored::fit_cases::Builder::with_registry(
            &self.runtime.registry,
            fits.len(),
            &validation,
        )
        .map_err(super::super::relation)?;
        for row in fits {
            builder.push(row.clone()).map_err(super::super::relation)?;
        }
        drop(builder.finish().map_err(super::super::relation)?);
        let store = self.runtime.canonical.store();
        let protection = store
            .protect(
                self.revision.canonical.clone(),
                std::time::Duration::from_secs(3600),
            )
            .await?;
        let mut read = SelectedRead::new(protection);
        let old = store.resolve_kind_scope(&mut read, None, "fit").await;
        let release = store.release(read.selection()).await;
        let old = old?;
        release?;
        let mut edits = fits
            .iter()
            .map(|row| {
                Ok(edit(
                    format!("fit:{}", row.fit_id),
                    "fits".into(),
                    row.fit_id.to_string(),
                    "fit".into(),
                    payload(row)?,
                    Vec::new(),
                ))
            })
            .collect::<Result<Vec<_>, WorkflowError>>()?;
        edits.extend(
            old.into_iter()
                .filter(|member| {
                    !fits
                        .iter()
                        .any(|row| format!("fit:{}", row.fit_id) == member.logical)
                })
                .map(|member| ObjectEdit {
                    logical: member.logical,
                    scope: member.scope,
                    name: member.name,
                    version: None,
                    references: Vec::new(),
                }),
        );
        let operation = pse_operations::mint_id::<SemanticId>().to_string();
        let canonical = store
            .edit(
                &self.revision.canonical.problem,
                Some(&self.revision.canonical.key),
                &operation,
                &edits,
            )
            .await?;
        Ok(SourceRevision {
            canonical,
            identity: self.revision.identity,
        })
    }
    /// Explicit full inventory/export request. This path is never a prerequisite
    /// of a selected preparation and keeps no source rows on the package handle.
    pub(super) async fn full_source(
        &self,
    ) -> Result<
        (
            Vec<Declaration>,
            PhysicalScope,
            Arc<pse_modeling::document::DocumentInventory>,
            crate::authoring_driver::document::Batches,
            Vec<Arc<pse_columnar::AllocationLease>>,
        ),
        WorkflowError,
    > {
        use pse_relations::generated::authored::{documents, fit_cases, packages};
        let store = self.runtime.canonical.store();
        let protection = store
            .protect(
                self.revision.canonical.clone(),
                std::time::Duration::from_secs(3600),
            )
            .await?;
        let mut read = SelectedRead::new(protection);
        let result: Result<_, WorkflowError> =
            async {
                let (mut physical, physical_lease, _physical_version) =
                    physical_context(&self.runtime, &mut read).await?;
                let mut index = SourceIndex::default();
                let mut leases = vec![physical_lease];
                if let Some(visible) = &mut physical.documents {
                    for member in store
                        .resolve_kind_scope(&mut read, None, "physical_scope_document")
                        .await?
                    {
                        let value = selected_object(&self.runtime, &read, &member.version).await?;
                        let entry: PhysicalScope = decode(value.payload.as_slice())?;
                        if entry.package != physical.package {
                            return Err(contract("canonical document physical package differs"));
                        }
                        visible.extend(entry.documents.ok_or_else(|| {
                            contract("canonical scoped physical entry is global")
                        })?);
                        leases.push(value.lease.clone());
                    }
                }
                let cancel = crate::CancelSource::new();
                let mut after = String::new();
                loop {
                    let page = store
                        .membership_page(read.selection(), None, &after)
                        .await?;
                    let Some(last) = page.last() else {
                        break;
                    };
                    after = last.key.clone();
                    let modeling = page
                        .into_iter()
                        .filter(|member| {
                            member.logical.starts_with("modeling:")
                                && member.logical != CONTEXT_LOGICAL
                                && member.logical != PHYSICAL_SCOPE_LOGICAL
                        })
                        .collect();
                    self.insert_members(&mut index, &mut read, modeling, &mut leases, &cancel)
                        .await?;
                }
                let validation = self.runtime.validation_context()?;
                let mut document_builder =
                    documents::Builder::with_registry(&self.runtime.registry, 0, &validation)
                        .map_err(super::super::relation)?;
                let mut inventory = pse_modeling::document::DocumentInventory {
                    field_spans: index.fields,
                    ..Default::default()
                };
                for member in store
                    .resolve_kind_scope(&mut read, None, "document")
                    .await?
                {
                    let value = selected_object(&self.runtime, &read, &member.version).await?;
                    let mut row: pse_model::generated::authored::documents::Row =
                        decode(value.payload.as_slice())?;
                    leases.push(value.lease.clone());
                    inventory
                        .packages
                        .insert(row.document_id, row.package_id.as_id());
                    if let Some(data) = object(
                        &self.runtime,
                        &mut read,
                        &format!("data:{}", row.document_id),
                    )
                    .await?
                    {
                        leases.push(data.lease.clone());
                        let bytes = data.row.payload.into_vec();
                        let (document, lease) =
                            crate::authoring_driver::document::decode_selected_data(
                                row.document_id,
                                row.path.clone(),
                                bytes.clone().into(),
                                Default::default(),
                                &self.runtime.shared.pool(),
                                &cancel.token(),
                            )
                            .map_err(WorkflowError::Authoring)?;
                        inventory.documents.insert(row.document_id, document);
                        leases.push(lease);
                        row.content = Some(bytes.into());
                    }
                    document_builder.push(row).map_err(super::super::relation)?;
                }
                let mut package_builder =
                    packages::Builder::with_registry(&self.runtime.registry, 0, &validation)
                        .map_err(super::super::relation)?;
                for member in store.resolve_kind_scope(&mut read, None, "package").await? {
                    let value = selected_object(&self.runtime, &read, &member.version).await?;
                    leases.push(value.lease.clone());
                    package_builder
                        .push(decode(value.payload.as_slice())?)
                        .map_err(super::super::relation)?;
                }
                let mut fit_builder =
                    fit_cases::Builder::with_registry(&self.runtime.registry, 0, &validation)
                        .map_err(super::super::relation)?;
                for member in store.resolve_kind_scope(&mut read, None, "fit").await? {
                    let value = selected_object(&self.runtime, &read, &member.version).await?;
                    fit_builder
                        .push(decode(value.payload.as_slice())?)
                        .map_err(super::super::relation)?;
                    leases.push(value.lease.clone());
                }
                let sources = BTreeMap::from([
                    (
                        fit_cases::RELATION_ID,
                        fit_builder.finish().map_err(super::super::relation)?,
                    ),
                    (
                        documents::RELATION_ID,
                        document_builder.finish().map_err(super::super::relation)?,
                    ),
                    (
                        packages::RELATION_ID,
                        package_builder.finish().map_err(super::super::relation)?,
                    ),
                ]);
                Ok((
                    index.rows.into_values().collect(),
                    physical,
                    Arc::new(inventory),
                    sources,
                    leases,
                ))
            }
            .await;
        let release = store.release(read.selection()).await;
        let result = result?;
        release?;
        Ok(result)
    }
    /// Read exactly one authored declaration for an operation's entry policy.
    pub(super) async fn declaration(
        &self,
        id: DeclarationId,
    ) -> Result<Declaration, WorkflowError> {
        let store = self.runtime.canonical.store();
        let protection = store
            .protect(
                self.revision.canonical.clone(),
                std::time::Duration::from_secs(3600),
            )
            .await?;
        let mut read = SelectedRead::new(protection);
        let result: Result<_, WorkflowError> = async {
            let value = object(&self.runtime, &mut read, &logical(id))
                .await?
                .ok_or_else(|| contract("canonical selected declaration absent"))?;
            let (row, _): SourcePayload = decode(value.payload.as_slice())?;
            Ok(row)
        }
        .await;
        let release = store.release(read.selection()).await;
        let result = result?;
        release?;
        Ok(result)
    }
    pub(super) fn admit_source(
        &self,
        source: &SelectedSource,
    ) -> Result<crate::math::modeling::ModelingRevision, WorkflowError> {
        self.admit_source_in(&self.numerical_workspace()?, source)
    }
    pub(super) fn admit_source_in(
        &self,
        workspace: &crate::math::Workspace,
        source: &SelectedSource,
    ) -> Result<crate::math::modeling::ModelingRevision, WorkflowError> {
        let revision = self.runtime.shared.math().modeling_revision(
            workspace,
            source.rows.clone(),
            source.physical.clone(),
            source.documents.clone(),
            &self.physical.key,
        )?;
        for (record, supplier) in &source.record_sources {
            if revision
                .checked()
                .record(*record)
                .map(|record| record.origin)
                != Some(*supplier)
            {
                return Err(contract(
                    "canonical record supplier differs from admitted scientific identity",
                ));
            }
        }
        Ok(revision)
    }
    pub(super) fn canonical_workspace(
        &self,
        source: &SelectedSource,
        cancel: &crate::CancelSource,
    ) -> Result<(crate::math::Workspace, CancellationWatch), WorkflowError> {
        let flag = Arc::new(std::sync::atomic::AtomicBool::new(
            cancel.token().is_cancelled(),
        ));
        let token = cancel.token();
        let watched = flag.clone();
        let watch = CancellationWatch(tokio::spawn(async move {
            token.cancelled().await;
            watched.store(true, std::sync::atomic::Ordering::Release);
        }));
        let workspace = self.runtime.shared.math().canonical_workspace(
            super::compiler_context(&self.physical, &self.providers),
            pse_compiler::workspace::WorkspaceLimits::default(),
            Arc::new(self.runtime.canonical.store().clone()),
            Arc::new(std::sync::Mutex::new(source.read.clone())),
            self.runtime.canonical.producer().cloned(),
            self.runtime.canonical.attestation().build,
            flag,
        )?;
        Ok((workspace, watch))
    }
    pub(super) async fn canonical_point(
        &self,
        root: DeclarationId,
        bindings: pse_modeling::Bindings,
        limits: pse_modeling::Limits,
        profile: pse_compiler::workspace::Profile,
        cancel: &crate::CancelSource,
    ) -> Result<pse_compiler::workspace::ModelingPointChecks, WorkflowError> {
        let selected = self.selected_source(root, cancel).await?;
        let admitted = (|| {
            let (workspace, watch) = self.canonical_workspace(&selected, cancel)?;
            let revision = self.admit_source_in(&workspace, &selected)?;
            Ok::<_, WorkflowError>((workspace, watch, revision))
        })();
        let result = match admitted {
            Ok((workspace, _watch, revision)) => self
                .runtime
                .shared
                .math()
                .modeling_point(workspace, revision, root, bindings, limits, profile, cancel)
                .await
                .map_err(WorkflowError::from),
            Err(error) => Err(error),
        };
        let release = self
            .runtime
            .canonical
            .store()
            .release(selected.read.selection())
            .await;
        let result = result?;
        release?;
        Ok(result)
    }
    pub(in crate::workflow) async fn selected_revision(
        &self,
        root: DeclarationId,
        cancel: &crate::CancelSource,
    ) -> Result<crate::math::modeling::ModelingRevision, WorkflowError> {
        let selected = self.selected_source(root, cancel).await?;
        let result = self.admit_source(&selected);
        let release = self
            .runtime
            .canonical
            .store()
            .release(selected.read.selection())
            .await;
        let result = result?;
        release?;
        Ok(result)
    }
    pub(in crate::workflow) async fn selected_revision_roots(
        &self,
        roots: &[DeclarationId],
        cancel: &crate::CancelSource,
    ) -> Result<crate::math::modeling::ModelingRevision, WorkflowError> {
        let selected = self.selected_sources(roots, cancel).await?;
        let result = self.admit_source(&selected);
        let release = self
            .runtime
            .canonical
            .store()
            .release(selected.read.selection())
            .await;
        let result = result?;
        release?;
        Ok(result)
    }
    pub(in crate::workflow) async fn all_revision(
        &self,
    ) -> Result<crate::math::modeling::ModelingRevision, WorkflowError> {
        let (rows, physical, documents, _, _source_owners) = self.full_source().await?;
        Ok(self.runtime.shared.math().modeling_revision(
            &self.numerical_workspace()?,
            rows,
            physical,
            documents,
            &self.physical.key,
        )?)
    }
    pub(super) async fn selected_source(
        &self,
        root: DeclarationId,
        cancel: &crate::CancelSource,
    ) -> Result<SelectedSource, WorkflowError> {
        self.selected_sources(&[root], cancel).await
    }
    async fn selected_sources(
        &self,
        roots: &[DeclarationId],
        cancel: &crate::CancelSource,
    ) -> Result<SelectedSource, WorkflowError> {
        let store = self.runtime.canonical.store();
        checkpoint(cancel)?;
        let protection = store
            .protect(
                self.revision.canonical.clone(),
                std::time::Duration::from_secs(3600),
            )
            .await?;
        let mut read = SelectedRead::new(protection);
        let result = self.select_into(roots, &mut read, cancel).await;
        match result {
            Ok((index, physical, documents, leases)) => Ok(SelectedSource {
                record_sources: index.record_sources,
                versions: index.versions,
                rows: index.rows.into_values().collect(),
                physical,
                documents: Arc::new(documents),
                read,
                _leases: leases,
            }),
            Err(error) => {
                let _ = store.release(read.selection()).await;
                Err(error)
            }
        }
    }
    async fn select_into(
        &self,
        roots: &[DeclarationId],
        read: &mut SelectedRead,
        cancel: &crate::CancelSource,
    ) -> Result<
        (
            SourceIndex,
            PhysicalScope,
            pse_modeling::document::DocumentInventory,
            Vec<Arc<pse_columnar::AllocationLease>>,
        ),
        WorkflowError,
    > {
        let store = self.runtime.canonical.store();
        let (mut physical, physical_lease, physical_version) =
            physical_context(&self.runtime, read).await?;
        let mut index = SourceIndex::default();
        let mut leases = vec![physical_lease];
        index
            .versions
            .insert(PHYSICAL_SCOPE_LOGICAL.into(), physical_version);
        index
            .requests
            .extend(roots.iter().copied().map(Request::Logical));
        let mut suppliers = BTreeSet::new();
        loop {
            checkpoint(cancel)?;
            let prior_rows = index.rows.len();
            let requests = std::mem::take(&mut index.requests);
            let mut requests = requests.into_iter().collect::<Vec<_>>();
            requests.sort_by_key(|request| match request {
                Request::Members(_) => 0,
                Request::Logical(_) => 1,
                Request::Imports(_) => 2,
                Request::Name(_, _) => 3,
            });
            let mut logicals = Vec::new();
            let mut names = BTreeMap::<Option<DeclarationId>, Vec<String>>::new();
            let mut inventories = Vec::new();
            let requested_scopes = requests
                .iter()
                .filter_map(|request| match request {
                    Request::Members(owner) => Some(*owner),
                    _ => None,
                })
                .collect::<BTreeSet<_>>();
            for request in requests {
                match request {
                    Request::Logical(id) => logicals.push(id),
                    Request::Name(parent, name) => {
                        if index.needs_name(parent, &name) {
                            names.entry(parent).or_default().push(name);
                        }
                    }
                    Request::Imports(owner)
                        if index.scopes.contains(&owner) || requested_scopes.contains(&owner) => {}
                    Request::Imports(owner) if !index.rows.contains_key(&owner) => {
                        logicals.push(owner);
                        index.requests.insert(Request::Imports(owner));
                    }
                    request => inventories.push(request),
                }
            }
            for requests in inventories.chunks(SELECTED_INVENTORY_BATCH) {
                let mut cursors = requests
                    .iter()
                    .map(|request| match request {
                        Request::Imports(owner) => {
                            store.kind_pages(read, Some(&logical(*owner)), "modeling:import")
                        }
                        Request::Members(owner) => store.scope_pages(read, &logical(*owner)),
                        _ => Err(pse_operations::canonical::CanonicalError::Configuration(
                            "invalid inventory frontier".into(),
                        )),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                self.insert_inventories(&mut index, read, &mut cursors, &mut leases, cancel)
                    .await?;
                // Package rules are semantic defaults, not named expression members.
                // Select their typed marker metadata beside imports; target traversal
                // then acquires each constant without hydrating the whole package.
                let mut rules = requests
                    .iter()
                    .filter_map(|request| match request {
                        Request::Imports(owner)
                            if index.rows[owner].value.kind == Kind::Package =>
                        {
                            Some(*owner)
                        }
                        _ => None,
                    })
                    .map(|owner| store.kind_pages(read, Some(&logical(owner)), ENGINEERING_RULE))
                    .collect::<Result<Vec<_>, _>>()?;
                if !rules.is_empty() {
                    self.insert_inventories(&mut index, read, &mut rules, &mut leases, cancel)
                        .await?;
                }
                for request in requests {
                    match request {
                        Request::Imports(owner) => {
                            index.imports.insert(*owner);
                        }
                        Request::Members(owner) => {
                            index.scopes.insert(*owner);
                            index.imports.insert(*owner);
                        }
                        _ => return Err(contract("invalid inventory frontier")),
                    }
                }
            }
            logicals.retain(|id| {
                !index.rows.contains_key(id) && !index.record_sources.contains_key(id)
            });
            for ids in logicals.chunks(64) {
                checkpoint(cancel)?;
                let scratch = self.runtime.shared.math().reserve(
                    "modeling:logical-frontier-metadata",
                    pse_operations::canonical_staging::SELECTED_OBJECT_HEADER_SCRATCH,
                )?;
                let members = store
                    .resolve_logicals(read, &ids.iter().copied().map(logical).collect::<Vec<_>>())
                    .await?;
                let present = members
                    .iter()
                    .map(|member| member.logical.clone())
                    .collect::<BTreeSet<_>>();
                self.insert_members(&mut index, read, members, &mut leases, cancel)
                    .await?;
                let missing = ids
                    .iter()
                    .filter(|id| !present.contains(&logical(**id)))
                    .copied()
                    .collect::<Vec<_>>();
                drop(present);
                drop(scratch);
                let mut records = objects(
                    &self.runtime,
                    read,
                    &missing
                        .iter()
                        .map(|id| format!("record:{id}"))
                        .collect::<Vec<_>>(),
                    cancel,
                )
                .await?;
                for id in missing {
                    let supplier = records.remove(&format!("record:{id}")).ok_or_else(|| {
                        contract("selected modeling declaration, record or lexical ancestor absent")
                    })?;
                    if supplier.kind != "record_supplier" {
                        return Err(contract("canonical record supplier interpretation differs"));
                    }
                    let origin: DeclarationId = decode(supplier.payload.as_slice())?;
                    index
                        .versions
                        .insert(supplier.logical.clone(), supplier.key.clone());
                    leases.push(supplier.lease.clone());
                    index.record_sources.insert(id, origin);
                    index.requests.insert(Request::Logical(origin));
                }
            }
            let names = names
                .into_iter()
                .flat_map(|(parent, names)| names.into_iter().map(move |name| (parent, name)))
                .filter(|(parent, name)| index.needs_name(*parent, name))
                .collect::<Vec<_>>();
            for names in names.chunks(64) {
                checkpoint(cancel)?;
                let _scratch = self.runtime.shared.math().reserve(
                    "modeling:lexical-pair-metadata",
                    pse_operations::canonical_staging::SELECTED_OBJECT_HEADER_SCRATCH,
                )?;
                let pairs = names
                    .iter()
                    .map(|(parent, name)| (scope(*parent), name.clone()))
                    .collect::<Vec<_>>();
                let members = store.resolve_name_pairs(read, &pairs).await?;
                self.insert_members(&mut index, read, members, &mut leases, cancel)
                    .await?;
                index.names.extend(names.iter().cloned());
            }
            let new_rows = std::mem::take(&mut index.new_rows);
            let mut supplier_demands = Vec::new();
            for id in new_rows {
                checkpoint(cancel)?;
                let scratch = index.rows[&id]
                    .owned_bytes()
                    .checked_mul(256)
                    .and_then(|n| n.checked_add(8192))
                    .ok_or_else(|| contract("selected reference extent overflow"))?;
                let scratch_lease = self
                    .runtime
                    .shared
                    .math()
                    .reserve("modeling:source-reference-parse", scratch)?;
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
                // Reserve before parsing. Keep this light path inventory beside the
                // immutable typed row, rather than retaining expression ASTs or
                // reparsing already-resolved source fields at every frontier.
                let references =
                    selected_source::declaration_references(&row, index.fields.get(&id))
                        .map_err(super::super::modeling_error)?;
                let reference_lease = self
                    .runtime
                    .shared
                    .math()
                    .reserve("modeling:source-reference-paths", references.owned_bytes())?;
                index.pending_references.insert(id, references);
                leases.push(reference_lease);
                drop(scratch_lease);
                if selected_source::requires_suppliers(&row) && suppliers.insert(id) {
                    let source_kinds: &[&str] = match row.value.kind {
                        Kind::IdentifierScheme => &["modeling:attribute"],
                        Kind::EntityKind => &[
                            "modeling:dataset",
                            "modeling:entity",
                            "modeling:entity_kind",
                        ],
                        _ => &["modeling:dataset"],
                    };
                    for source_kind in source_kinds {
                        supplier_demands.push((
                            scope(row.parent_id),
                            row.name.clone(),
                            *source_kind,
                        ));
                    }
                }
            }
            for demands in supplier_demands.chunks(SELECTED_INVENTORY_BATCH) {
                let mut cursors = demands
                    .iter()
                    .map(|(scope, name, kind)| store.reference_pages(read, scope, name, kind))
                    .collect::<Result<Vec<_>, _>>()?;
                self.insert_inventories(&mut index, read, &mut cursors, &mut leases, cancel)
                    .await?;
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
        let mut metadata_ids = document_ids
            .iter()
            .map(|id| format!("document:{id}"))
            .collect::<Vec<_>>();
        if physical.documents.is_some() {
            metadata_ids.extend(
                document_ids
                    .iter()
                    .map(|id| format!("physical_scope_document:{id}")),
            );
        }
        let mut metadata_objects = objects(&self.runtime, read, &metadata_ids, cancel).await?;
        let mut documents = pse_modeling::document::DocumentInventory {
            field_spans: index.fields.clone(),
            ..Default::default()
        };
        let mut metadata_by_document = BTreeMap::new();
        for id in document_ids {
            if let Some(visible) = &mut physical.documents
                && let Some(value) =
                    metadata_objects.remove(&format!("physical_scope_document:{id}"))
            {
                let entry: PhysicalScope = decode(value.payload.as_slice())?;
                if value.kind != "physical_scope_document"
                    || entry.package != physical.package
                    || entry.documents != Some(BTreeSet::from([id]))
                {
                    return Err(contract("canonical document physical scope differs"));
                }
                visible.insert(id);
                index
                    .versions
                    .insert(value.logical.clone(), value.key.clone());
                leases.push(value.lease.clone());
            }
            if let Some(metadata) = metadata_objects.remove(&format!("document:{id}")) {
                if metadata.kind != "document" {
                    return Err(contract(
                        "canonical document metadata interpretation differs",
                    ));
                }
                index
                    .versions
                    .insert(metadata.logical.clone(), metadata.key.clone());
                leases.push(metadata.lease.clone());
                let metadata: pse_model::generated::authored::documents::Row =
                    decode(metadata.payload.as_slice())?;
                documents.packages.insert(id, metadata.package_id.as_id());
                metadata_by_document.insert(id, metadata);
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
                let metadata = metadata_by_document.get(&row.document_id).ok_or_else(|| {
                    contract("selected dataset declaring document metadata absent")
                })?;
                let id = pse_ids::named_id(metadata.package_id.as_id(), path);
                data_documents.insert(id, path.clone());
            }
        }
        let mut data_objects = objects(
            &self.runtime,
            read,
            &data_documents
                .keys()
                .map(|id| format!("data:{id}"))
                .collect::<Vec<_>>(),
            cancel,
        )
        .await?;
        for (id, path) in data_documents {
            checkpoint(cancel)?;
            let bytes = data_objects
                .remove(&format!("data:{id}"))
                .ok_or_else(|| contract("selected dataset data document absent"))?;
            if bytes.kind != "data" {
                return Err(contract(
                    "canonical dataset document interpretation differs",
                ));
            }
            index
                .versions
                .insert(bytes.logical.clone(), bytes.key.clone());
            leases.push(bytes.lease.clone());
            let (document, lease) = crate::authoring_driver::document::decode_selected_data(
                id,
                path,
                bytes.row.payload.into_vec().into(),
                Default::default(),
                &self.runtime.shared.pool(),
                &cancel.token(),
            )
            .map_err(WorkflowError::Authoring)?;
            documents.documents.insert(id, document);
            leases.push(lease);
        }
        Ok((index, physical, documents, leases))
    }
    async fn insert_inventories(
        &self,
        index: &mut SourceIndex,
        read: &mut SelectedRead,
        cursors: &mut Vec<pse_operations::canonical_selection::SelectedMemberships>,
        leases: &mut Vec<Arc<pse_columnar::AllocationLease>>,
        cancel: &crate::CancelSource,
    ) -> Result<(), WorkflowError> {
        while !cursors.is_empty() {
            checkpoint(cancel)?;
            let scratch = self.runtime.shared.math().reserve(
                "modeling:namespace-page",
                pse_operations::canonical_staging::SELECTED_OBJECT_HEADER_SCRATCH,
            )?;
            let pages = self
                .runtime
                .canonical
                .store()
                .next_membership_pages(read, cursors)
                .await?;
            checkpoint(cancel)?;
            let completed = pages.iter().map(Vec::is_empty).collect::<Vec<_>>();
            let members = pages.into_iter().flatten().collect::<Vec<_>>();
            // Every cursor records its own exact complete premise, while shared
            // source objects are decoded just once across overlapping suppliers.
            let extent = members
                .owned_bytes()
                .checked_mul(4)
                .and_then(|bytes| bytes.checked_add(4096))
                .ok_or_else(|| contract("selected namespace metadata extent overflow"))?;
            leases.push(
                self.runtime
                    .shared
                    .math()
                    .reserve("modeling:namespace-identities", extent)?,
            );
            self.insert_members(index, read, members, leases, cancel)
                .await?;
            let mut position = 0;
            cursors.retain(|_| {
                let retain = !completed[position];
                position += 1;
                retain
            });
            drop(scratch);
        }
        Ok(())
    }
    async fn insert_members(
        &self,
        index: &mut SourceIndex,
        read: &mut SelectedRead,
        members: Vec<Membership>,
        leases: &mut Vec<Arc<pse_columnar::AllocationLease>>,
        cancel: &crate::CancelSource,
    ) -> Result<(), WorkflowError> {
        let mut unread = BTreeMap::<String, Membership>::new();
        for member in members {
            checkpoint(cancel)?;
            if let Some(version) = index.versions.get(&member.logical) {
                if version != &member.version {
                    return Err(contract("one selected logical has incompatible versions"));
                }
                continue;
            }
            if let Some(previous) = unread.get(&member.logical) {
                if previous.version != member.version {
                    return Err(contract("one selected logical has incompatible versions"));
                }
            } else {
                unread.insert(member.logical.clone(), member);
            }
        }
        let unread = unread.into_values().collect::<Vec<_>>();
        for members in unread.chunks(pse_operations::canonical_staging::SELECTED_OBJECT_BATCH) {
            let versions = members
                .iter()
                .map(|member| member.version.clone())
                .collect::<Vec<_>>();
            let objects = selected_objects(&self.runtime, read, &versions, cancel).await?;
            for (member, object) in members.iter().zip(objects) {
                checkpoint(cancel)?;
                if !object.kind.starts_with("modeling:") || object.kind == CONTEXT {
                    return Err(contract(
                        "canonical lexical scope contains foreign object kind",
                    ));
                }
                let lease = object.lease.clone();
                let source: SourcePayload = decode(object.payload.as_slice())?;
                if logical(source.0.declaration_id) != object.logical
                    || kind(&source.0) != object.kind
                    || member.logical != object.logical
                    || member.scope != scope(source.0.parent_id)
                    || member.name != source.0.name
                {
                    return Err(contract(
                        "canonical source metadata differs from generated declaration",
                    ));
                }
                if source.0.owned_bytes().saturating_add(1024) > lease.size() {
                    return Err(contract("canonical source decode extent insufficient"));
                }
                index.insert(source)?;
                index
                    .versions
                    .insert(member.logical.clone(), member.version.clone());
                leases.push(lease);
            }
        }
        Ok(())
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
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
            .canonical
            .store()
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
                .canonical
                .store()
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
            .canonical
            .store()
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
            let store = runtime.canonical.store();
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
                .canonical
                .store()
                .qualify_product(&mut selected.read.clone(), &unmarked_candidate)
                .await
                .unwrap()
                .is_some()
        );
        runtime
            .canonical
            .store()
            .release(selected.read.selection())
            .await
            .unwrap();
        drop(selected);
        let package = unmarked.with_declarations(rows.clone()).await.unwrap();
        let selected = package.selected_source(root, &cancel).await.unwrap();
        assert!(
            runtime
                .canonical
                .store()
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
                .canonical
                .store()
                .qualify_product(&mut selected.read.clone(), &marked_candidate)
                .await
                .unwrap()
                .is_some(),
            "the marked product qualifies against its own protected source"
        );
        let marked_versions = selected.versions.clone();
        runtime
            .canonical
            .store()
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
        assert!(revised.admit_source(&selected).is_ok());
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
                .canonical
                .store()
                .qualify_product(&mut selected.read.clone(), &marked_candidate)
                .await
                .unwrap()
                .is_none(),
            "changed package source provenance invalidates exact reuse"
        );
        runtime
            .canonical
            .store()
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
                .canonical
                .store()
                .qualify_product(&mut selected.read.clone(), &marked_candidate)
                .await
                .unwrap()
                .is_some(),
            "unrelated relation growth preserves the actual reuse premises"
        );
        runtime
            .canonical
            .store()
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
                .canonical
                .store()
                .qualify_product(&mut selected.read.clone(), &marked_candidate)
                .await
                .unwrap()
                .is_none(),
            "removing the package rule invalidates its recorded marker inventory"
        );
        assert!(
            runtime
                .canonical
                .store()
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
            .canonical
            .store()
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
            .canonical
            .store()
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
            .canonical
            .store()
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
        let admitted = package.admit_source(&selected).unwrap();
        assert!(admitted.checked().entry("p.D").is_some());
        assert!(admitted.checked().entry("p.Unrelated").is_none());
        runtime
            .canonical
            .store()
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
        assert!(revised.admit_source(&selected).is_ok());
        runtime
            .canonical
            .store()
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
        let revision = package.admit_source(&selected).unwrap();
        assert_eq!(revision.checked().resolve(root, "outer"), Some(inner));
        runtime
            .canonical
            .store()
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
        let admitted = package.admit_source(&selected).unwrap();
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
            .canonical
            .store()
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
        assert!(package.admit_source(&selected).is_ok());
        runtime
            .canonical
            .store()
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
