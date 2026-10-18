// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact-parent delta ingress. Unchanged data bytes never enter the ordinary
//! metadata path; V4 is framed from its authenticated ordered parent metadata.
use super::*;
use crate::workflow::SourceDelta;
use pse_model::generated::authored::documents as document_wire;

pub(in crate::workflow::modeling) struct Parent {
    pub(in crate::workflow::modeling) rows: Vec<Declaration>,
    pub(super) frame: IngressFrame,
    pub(super) physical: PhysicalScope,
    pub(super) documents: BTreeMap<SemanticId, document_wire::Row>,
    pub(super) members: BTreeMap<String, SourceMember>,
    pub(super) fields: BTreeMap<DeclarationId, FieldSpans>,
    pub(super) _leases: Vec<Arc<pse_columnar::AllocationLease>>,
}

/// Validate disjoint intent and existence before any publication or mutation.
fn intentions<K: Ord + Clone>(
    existing: &BTreeSet<K>,
    additions: impl Iterator<Item = K>,
    replacements: impl Iterator<Item = K>,
    removals: impl Iterator<Item = K>,
) -> Result<BTreeSet<K>, WorkflowError> {
    let mut touched = BTreeSet::new();
    for (kind, keys) in [
        ("addition", additions.collect::<Vec<_>>()),
        ("replacement", replacements.collect()),
        ("removal", removals.collect()),
    ] {
        for key in keys {
            if !touched.insert(key.clone()) {
                return Err(contract(
                    "source delta contains duplicate or overlapping intents",
                ));
            }
            if existing.contains(&key) == (kind == "addition") {
                return Err(contract(format!(
                    "source delta {kind} has an incompatible parent membership"
                )));
            }
        }
    }
    Ok(touched)
}

fn source_index(
    runtime: &Runtime,
    rows: &[Declaration],
    fields: &BTreeMap<DeclarationId, FieldSpans>,
) -> Result<SourceIndex, WorkflowError> {
    let mut index = SourceIndex {
        roots_complete: true,
        ..Default::default()
    };
    for row in rows {
        index.insert((row.clone(), fields.get(&row.declaration_id).cloned()))?;
        index.parsed_fields.insert(
            row.declaration_id,
            parsed_fields(runtime, row, fields.get(&row.declaration_id))?,
        );
        index.complete_scope(row.declaration_id);
        index.complete_imports(row.declaration_id);
    }
    index.requests.clear();
    Ok(index)
}

/// Only consumed document interpretation premises can invalidate retained data
/// admission. An unrelated text or document ownership edit is not a data edit.
fn data_documents(
    rows: &[Declaration],
    documents: &BTreeMap<SemanticId, document_wire::Row>,
) -> Result<BTreeSet<SemanticId>, WorkflowError> {
    let mut consumed = BTreeSet::new();
    for row in rows {
        if let Some(path) = row
            .value
            .dataset
            .as_ref()
            .and_then(|dataset| dataset.document.as_ref())
        {
            let owner = documents
                .get(&row.document_id)
                .ok_or_else(|| contract("data supplier package absent"))?;
            consumed.insert(row.document_id);
            consumed.insert(pse_ids::named_id(owner.package_id.as_id(), path));
        }
    }
    Ok(consumed)
}

impl ModelingPackage {
    pub(in crate::workflow::modeling) async fn delta_parent(
        &self,
        cancel: &crate::CancelSource,
    ) -> Result<Parent, WorkflowError> {
        checkpoint(cancel)?;
        let (frame, frame_lease) =
            self.source_decode::<IngressFrame>(self.local_object(INGRESS_FRAME)?.as_ref())?;
        let (mut physical, physical_lease) = self
            .source_decode::<PhysicalScope>(self.local_object(PHYSICAL_SCOPE_LOGICAL)?.as_ref())?;
        let mut leases = vec![frame_lease, physical_lease];
        let mut members = BTreeMap::new();
        let mut documents = BTreeMap::new();
        let mut index = SourceIndex::default();
        for object in self.revision.admitted.objects.values() {
            checkpoint(cancel)?;
            if object.logical.starts_with("modeling:")
                && object.logical != CONTEXT_LOGICAL
                && object.logical != PHYSICAL_SCOPE_LOGICAL
            {
                let source = object
                    .declaration
                    .as_ref()
                    .ok_or_else(|| contract("owned typed declaration absent"))?
                    .clone();
                let lease = self.runtime.shared.math().reserve(
                    "modeling:delta-declaration-copy",
                    source.0.owned_bytes().saturating_add(8192),
                )?;
                index.insert(source)?;
                leases.push(lease);
            } else if object.kind == "document" {
                let (row, lease) = self.source_decode::<document_wire::Row>(object)?;
                if row.content.is_some() {
                    return Err(contract(
                        "document metadata unexpectedly contains data bytes",
                    ));
                }
                documents.insert(row.document_id, row);
                leases.push(lease);
            } else if object.kind == "physical_scope_document" {
                let (entry, lease) = self.source_decode::<PhysicalScope>(object)?;
                if entry.package != physical.package {
                    return Err(contract("delta physical package differs"));
                }
                physical
                    .documents
                    .as_mut()
                    .ok_or_else(|| contract("scoped entry in global physical context"))?
                    .extend(
                        entry
                            .documents
                            .ok_or_else(|| contract("global entry in scoped physical context"))?,
                    );
                leases.push(lease);
            }
            members.insert(object.logical.clone(), SourceMember::from(object.as_ref()));
        }
        let mut rows = Vec::with_capacity(frame.order.len());
        let mut unique = BTreeSet::new();
        for id in &frame.order {
            if !unique.insert(*id) {
                return Err(contract("ordered parent frame repeats a declaration"));
            }
            rows.push(
                index
                    .rows
                    .remove(id)
                    .ok_or_else(|| contract("ordered parent frame names absent declaration"))?,
            );
        }
        if !index.rows.is_empty() {
            return Err(contract("ordered parent frame omits declarations"));
        }
        let names = selected_source::physical_bindings(&rows, &physical, &self.physical.quantities);
        let identity = crate::math::modeling::source_revision_frame(
            &rows,
            frame.data.iter().map(|(id, hash)| (*id, *hash)),
            &self.physical.key,
            &names,
        );
        if identity != self.revision.identity {
            return Err(contract(
                "ordered parent metadata differs from semantic source identity",
            ));
        }
        Ok(Parent {
            rows,
            frame,
            physical,
            documents,
            members,
            fields: index.fields,
            _leases: leases,
        })
    }
    /// Stage an exact local parent edit; durable recording is a separate CAS effect.
    pub async fn with_delta(
        &self,
        delta: SourceDelta,
        cancel: &crate::CancelSource,
    ) -> Result<Self, WorkflowError> {
        let activity = self.runtime.admit_activity()?;
        activity
            .scope(async {
                if delta.parent != self.revision_id() {
                    return Err(contract("source delta names a different local parent"));
                }
                let deadline = self.runtime.shared.math().preparation_deadline(cancel)?;
                let scoped = cancel.with_deadline(Some(deadline));
                let parent = self.delta_parent(&scoped).await?;
                self.apply_delta_parent(delta, parent, &scoped, None).await
            })
            .await
    }

    pub(super) async fn apply_delta_parent(
        &self,
        delta: SourceDelta,
        parent: Parent,
        cancel: &crate::CancelSource,
        parsed: Option<reload::ParsedEdit>,
    ) -> Result<Self, WorkflowError> {
        self.apply_delta_parent_ordered(delta, parent, cancel, parsed, None)
            .await
    }
    pub(in crate::workflow) async fn construct_study_variant(
        &self,
        source: &crate::workflow::OperationSource,
        mut delta: SourceDelta,
        cancel: &crate::CancelSource,
    ) -> Result<Self, WorkflowError> {
        source.check(self)?;
        // The descriptor already checked its original local parent token. Reconstruction
        // rebinds only after the complete immutable source/context identity agrees.
        delta.parent = self.revision_id().into();
        let deadline = self.runtime.shared.math().preparation_deadline(cancel)?;
        let scoped = cancel.with_deadline(Some(deadline));
        let parent = self.delta_parent(&scoped).await?;
        self.apply_delta_parent_ordered_mode(delta, parent, &scoped, None, None, false)
            .await
    }

    pub(in crate::workflow::modeling) async fn apply_delta_parent_ordered(
        &self,
        delta: SourceDelta,
        parent: Parent,
        cancel: &crate::CancelSource,
        parsed: Option<reload::ParsedEdit>,
        order: Option<Vec<DeclarationId>>,
    ) -> Result<Self, WorkflowError> {
        self.apply_delta_parent_ordered_mode(delta, parent, cancel, parsed, order, true)
            .await
    }

    async fn apply_delta_parent_ordered_mode(
        &self,
        delta: SourceDelta,
        mut parent: Parent,
        cancel: &crate::CancelSource,
        parsed: Option<reload::ParsedEdit>,
        order: Option<Vec<DeclarationId>>,
        record: bool,
    ) -> Result<Self, WorkflowError> {
        let activity = self.runtime.admit_activity()?;
        activity.scope(async {
        let original_physical = parent.physical.clone();
        let fit_intents = delta
            .fits
            .additions
            .iter()
            .chain(&delta.fits.replacements)
            .map(|row| format!("fit:{}", row.fit_id))
            .chain(delta.fits.removals.iter().map(|id| format!("fit:{id}")))
            .collect::<BTreeSet<_>>();
        let existing = parent.rows.iter().map(|row| row.declaration_id).collect();
        intentions(
            &existing,
            delta.additions.iter().map(|r| r.declaration_id),
            delta.replacements.iter().map(|r| r.declaration_id),
            delta.removals.iter().copied(),
        )?;
        let old_rows = parent
            .rows
            .iter()
            .map(|row| (row.declaration_id, row))
            .collect::<BTreeMap<_, _>>();
        let changed = delta
            .additions
            .iter()
            .map(|row| row.declaration_id)
            .chain(delta.removals.iter().copied())
            .chain(
                delta
                    .replacements
                    .iter()
                    .filter(|row| {
                        old_rows
                            .get(&row.declaration_id)
                            .is_none_or(|old| *old != *row)
                    })
                    .map(|row| row.declaration_id),
            )
            .collect::<BTreeSet<_>>();
        let document_changes = intentions(
            &parent.documents.keys().copied().collect(),
            delta.documents.additions.iter().map(|r| r.document_id),
            delta.documents.replacements.iter().map(|r| r.document_id),
            delta.documents.removals.iter().copied(),
        )?;
        let fit_keys = parent
            .members
            .keys()
            .filter(|key| key.starts_with("fit:"))
            .cloned()
            .collect();
        intentions(
            &fit_keys,
            delta
                .fits
                .additions
                .iter()
                .map(|r| format!("fit:{}", r.fit_id)),
            delta
                .fits
                .replacements
                .iter()
                .map(|r| format!("fit:{}", r.fit_id)),
            delta.fits.removals.iter().map(|id| format!("fit:{id}")),
        )?;
        let extent = parent
            .rows
            .owned_bytes()
            .checked_add(delta.additions.owned_bytes())
            .and_then(|n| n.checked_add(delta.replacements.owned_bytes()))
            .and_then(|n| n.checked_add(delta.documents.additions.owned_bytes()))
            .and_then(|n| n.checked_add(delta.documents.replacements.owned_bytes()))
            .and_then(|n| n.checked_add(delta.fits.additions.owned_bytes()))
            .and_then(|n| n.checked_add(delta.fits.replacements.owned_bytes()))
            .and_then(|n| n.checked_mul(16))
            .ok_or_else(|| contract("source delta extent"))?;
        let _scratch = self
            .runtime
            .shared
            .math()
            .reserve("modeling:source-delta", extent)?;
        let validation = self.runtime.admission_context();
        let mut declarations =
            pse_relations::generated::authored::modeling_declarations::Builder::with_registry(
                self.runtime.registry(),
                delta.additions.len() + delta.replacements.len(),
                &validation,
            )
            .map_err(super::super::super::relation)?;
        for row in delta.additions.iter().chain(&delta.replacements) {
            checkpoint(cancel)?;
            declarations
                .push(row.clone())
                .map_err(super::super::super::relation)?;
        }
        drop(
            declarations
                .finish()
                .map_err(super::super::super::relation)?,
        );
        let mut document_builder =
            pse_relations::generated::authored::documents::Builder::with_registry(
                self.runtime.registry(),
                delta.documents.additions.len() + delta.documents.replacements.len(),
                &validation,
            )
            .map_err(super::super::super::relation)?;
        for row in delta
            .documents
            .additions
            .iter()
            .chain(&delta.documents.replacements)
        {
            if parsed.is_none() && row.source_text.is_some() {
                return Err(contract(
                    "IR document changes cannot assert source text; use parent-preserving parsed reload",
                ));
            }
            if row.document_id != pse_ids::named_id(row.package_id.as_id(), &row.path) {
                return Err(contract(
                    "delta document identity does not match its package and path",
                ));
            }
            document_builder
                .push(row.clone())
                .map_err(super::super::super::relation)?;
        }
        drop(
            document_builder
                .finish()
                .map_err(super::super::super::relation)?,
        );
        let mut fit_builder =
            pse_relations::generated::authored::fit_cases::Builder::with_registry(
                self.runtime.registry(),
                delta.fits.additions.len() + delta.fits.replacements.len(),
                &validation,
            )
            .map_err(super::super::super::relation)?;
        for row in delta.fits.additions.iter().chain(&delta.fits.replacements) {
            fit_builder
                .push(row.clone())
                .map_err(super::super::super::relation)?;
        }
        drop(
            fit_builder
                .finish()
                .map_err(super::super::super::relation)?,
        );

        let old_data = source_index(&self.runtime, &parent.rows, &parent.fields)?.data_closure()?;
        let old_data_documents = data_documents(&old_data, &parent.documents)?;
        let edited_documents = parent
            .rows
            .iter()
            .filter(|r| changed.contains(&r.declaration_id))
            .map(|r| r.document_id)
            .chain(
                delta
                    .additions
                    .iter()
                    .chain(&delta.replacements)
                    .filter(|r| changed.contains(&r.declaration_id))
                    .map(|r| r.document_id),
            )
            .chain(document_changes.iter().copied())
            .collect::<BTreeSet<_>>();
        let removals = delta.removals.iter().copied().collect::<BTreeSet<_>>();
        let mut replacements = delta
            .replacements
            .into_iter()
            .map(|r| (r.declaration_id, r))
            .collect::<BTreeMap<_, _>>();
        parent
            .rows
            .retain(|row| !removals.contains(&row.declaration_id));
        for row in &mut parent.rows {
            if let Some(replacement) = replacements.remove(&row.declaration_id) {
                *row = replacement;
            }
        }
        parent.rows.extend(delta.additions);
        if let Some(order)=order {
            let mut rows=parent.rows.drain(..).map(|row|(row.declaration_id,row)).collect::<BTreeMap<_,_>>();
            for id in order {parent.rows.push(rows.remove(&id).ok_or_else(||contract("complete row order repeats or names absent declaration"))?);}
            if !rows.is_empty(){return Err(contract("complete row order omits declarations"));}
        }
        if let Some(parsed) = &parsed {
            let mut rows = parent
                .rows
                .drain(..)
                .map(|row| (row.declaration_id, row))
                .collect::<BTreeMap<_, _>>();
            for id in &parsed.order {
                parent.rows.push(rows.remove(id).ok_or_else(|| {
                    contract("parsed row order is not an exact merged inventory")
                })?);
            }
            if !rows.is_empty() {
                return Err(contract("parsed row order omits retained declarations"));
            }
            parent.physical = parsed.physical.clone();
        }
        // An IR edit no longer has an exact text preimage. Clear the whole affected
        // document's attribution rather than keeping plausible but obsolete spans.
        let stale_spans = parent
            .rows
            .iter()
            .filter(|r| edited_documents.contains(&r.document_id))
            .map(|r| r.declaration_id)
            .collect::<BTreeSet<_>>();
        parent
            .fields
            .retain(|id, _| !changed.contains(id) && !stale_spans.contains(id));
        if let Some(parsed) = &parsed {
            parent.fields = parsed.fields.clone();
        }
        let mut edits = BTreeMap::<String, ObjectEdit>::new();
        let mut typed = TypedSourceInputs::default();
        for id in &delta.documents.removals {
            parent.documents.remove(id);
            parent.frame.data.remove(id);
            for logical in [format!("document:{id}"), format!("data:{id}")] {
                if let Some(member) = parent.members.get(&logical) {
                    edits.insert(logical.clone(), removal(member));
                }
            }
            if let Some(documents) = &mut parent.physical.documents {
                documents.remove(id);
            }
        }
        let mut inventory = pse_modeling::document::DocumentInventory::default();
        for mut row in delta
            .documents
            .additions
            .into_iter()
            .chain(delta.documents.replacements)
        {
            let id = row.document_id;
            if let Some(bytes) = row.content.take() {
                if let Some(parsed) = &parsed {
                    let hash =
                        parsed.data.get(&id).copied().ok_or_else(|| {
                            contract("encoded parsed document lacks its loader hash")
                        })?;
                    parent.frame.data.insert(id, hash);
                } else {
                    let (document, lease) =
                        pse_admission::authoring_driver::document::decode_selected_data(
                            id,
                            row.path.clone(),
                            bytes.as_slice().to_vec().into(),
                            Default::default(),
                            &self.runtime.shared.pool(),
                            &cancel.token(),
                        )?;
                    parent.frame.data.insert(id, document.content_hash);
                    inventory.documents.insert(id, document);
                    parent._leases.push(lease);
                }
                edits.insert(
                    format!("data:{id}"),
                    edit(
                        format!("data:{id}"),
                        "data".into(),
                        id.to_string(),
                        "data".into(),
                        bytes.into_vec(),
                        Vec::new(),
                    ),
                );
            } else if parsed
                .as_ref()
                .is_none_or(|parsed| !parsed.data.contains_key(&id))
            {
                parent.frame.data.remove(&id);
                if let Some(member) = parent.members.get(&format!("data:{id}")) {
                    edits.insert(member.logical.clone(), removal(member));
                }
            }
            parent.documents.insert(id, row);
        }
        for (id, row) in &mut parent.documents {
            if let Some(parsed) = &parsed {
                if let Some(exact) = parsed.documents.get(id) {
                    *row = exact.clone();
                }
            } else if edited_documents.contains(id) {
                row.source_text = None;
            }
            if document_changes.contains(id) || edited_documents.contains(id) {
                edits.insert(
                    format!("document:{id}"),
                    typed.document(row.clone())?,
                );
            }
            inventory.packages.insert(*id, row.package_id.as_id());
        }
        if parent
            .rows
            .iter()
            .any(|row| delta.documents.removals.contains(&row.document_id))
        {
            return Err(contract(
                "removed document still owns retained declarations",
            ));
        }
        inventory.field_spans = parent.fields.clone();
        let mut index = source_index(&self.runtime, &parent.rows, &parent.fields)?;
        inventory.parsed_fields = index.parsed_fields.clone();
        // Re-resolve every outgoing reference against the merged namespace. This
        // conservatively covers positive/absent/shadowed/imported bindings and stable
        // names whose expression bytes changed, without trusting a spelling shortcut.
        for row in &parent.rows {
            checkpoint(cancel)?;
            let references = index
                .references(row)?
                .into_iter()
                .map(|id| {
                    let target = &index.rows[&id];
                    (scope(target.parent_id), target.name.clone())
                })
                .collect();
            edits.insert(
                logical(row.declaration_id),
                typed.declaration(
                    row,
                    parent.fields.get(&row.declaration_id).cloned(),
                    index.parsed_fields[&row.declaration_id].clone(),
                    references,
                )?,
            );
        }
        for id in removals {
            let member = parent
                .members
                .get(&logical(id))
                .ok_or_else(|| contract("removed declaration membership absent"))?;
            edits.insert(member.logical.clone(), removal(member));
        }
        index.requests.clear();
        let new_data = index.data_closure()?;
        let new_data_documents = data_documents(&new_data, &parent.documents)?;
        if new_data != old_data
            || new_data_documents != old_data_documents
            || parent.physical != original_physical
            || document_changes
                .iter()
                .any(|id| old_data_documents.contains(id) || new_data_documents.contains(id))
        {
            // Only actual data suppliers of the changed semantic data closure are
            // hydrated; unrelated inherited tables remain unopened and unvalidated.
            for row in &new_data {
                if let Some(path) = row
                    .value
                    .dataset
                    .as_ref()
                    .and_then(|dataset| dataset.document.as_ref())
                {
                    let package = inventory
                        .packages
                        .get(&row.document_id)
                        .ok_or_else(|| contract("data supplier package absent"))?;
                    let id = pse_ids::named_id(*package, path);
                    if let std::collections::btree_map::Entry::Vacant(entry) =
                        inventory.documents.entry(id)
                    {
                        let logical = format!("data:{id}");
                        let metadata = parent
                            .documents
                            .get(&id)
                            .ok_or_else(|| contract("data supplier metadata absent"))?;
                        let bytes = if let Some(version) =
                            edits.get(&logical).and_then(|edit| edit.version.as_ref())
                        {
                            version.payload.as_slice().to_vec()
                        } else {
                            self.local_object(&logical)?.payload.clone()
                        };
                        let (document, lease) =
                            pse_admission::authoring_driver::document::decode_selected_data(
                                id,
                                metadata.path.clone(),
                                bytes.into(),
                                Default::default(),
                                &self.runtime.shared.pool(),
                                &cancel.token(),
                            )?;
                        if parent.frame.data.get(&id) != Some(&document.content_hash) {
                            return Err(contract(
                                "data supplier differs from authenticated ingress frame",
                            ));
                        }
                        entry.insert(document);
                        parent._leases.push(lease);
                    }
                }
            }
            let mut records = BTreeSet::new();
            if !new_data.is_empty() {
                let workspace = self.runtime.shared.math().workspace(
                    super::super::compiler_context(&self.physical, &BTreeMap::new()),
                    pse_compiler::workspace::WorkspaceLimits::default(),
                )?;
                let data = self.runtime.shared.math().modeling_revision(
                    &workspace,
                    new_data,
                    parent.physical.clone(),
                    Arc::new(inventory),
                    &self.physical.key,
                    cancel,
                ).await?;
                for (record, supplier) in selected_source::record_sources(data.checked()) {
                    let source = &index.rows[&supplier];
                    let logical = format!("record:{record}");
                    records.insert(logical.clone());
                    edits.insert(
                        logical.clone(),
                        edit(
                            logical,
                            "records".into(),
                            record.to_string(),
                            "record_supplier".into(),
                            payload(&supplier)?,
                            vec![(scope(source.parent_id), source.name.clone())],
                        ),
                    );
                }
            }
            for member in parent
                .members
                .values()
                .filter(|member| member.logical.starts_with("record:"))
            {
                if !records.contains(&member.logical) {
                    edits.insert(member.logical.clone(), removal(member));
                }
            }
        }
        for row in delta
            .fits
            .additions
            .into_iter()
            .chain(delta.fits.replacements)
        {
            let logical = format!("fit:{}", row.fit_id);
            edits.insert(
                logical.clone(),
                typed.fit(row)?,
            );
        }
        for id in delta.fits.removals {
            let member = &parent.members[&format!("fit:{id}")];
            edits.insert(member.logical.clone(), removal(member));
        }
        if let Some(parsed) = &parsed {
            let current_packages = parsed
                .packages
                .iter()
                .map(|edit| edit.logical.clone())
                .collect::<BTreeSet<_>>();
            for member in parent.members.values().filter(|member| {
                member.logical.starts_with("package:")
                    && !current_packages.contains(&member.logical)
            }) {
                edits.insert(member.logical.clone(), removal(member));
            }
            for entry in &parsed.packages {
                edits.insert(entry.logical.clone(), entry.clone());
            }
            let mut header = parent.physical.clone();
            header.documents = parent.physical.documents.as_ref().map(|_| BTreeSet::new());
            edits.insert(
                PHYSICAL_SCOPE_LOGICAL.into(),
                edit(
                    PHYSICAL_SCOPE_LOGICAL.into(),
                    "context".into(),
                    "physical_scope".into(),
                    "physical_scope".into(),
                    payload(&header)?,
                    Vec::new(),
                ),
            );
            let visible = parent.physical.documents.as_ref();
            for member in parent
                .members
                .values()
                .filter(|member| member.logical.starts_with("physical_scope_document:"))
            {
                if !visible.is_some_and(|ids| {
                    ids.iter()
                        .any(|id| member.logical == format!("physical_scope_document:{id}"))
                }) {
                    edits.insert(member.logical.clone(), removal(member));
                }
            }
            if let Some(visible) = visible {
                for id in visible {
                    let entry = PhysicalScope {
                        package: parent.physical.package.clone(),
                        documents: Some(BTreeSet::from([*id])),
                    };
                    let logical = format!("physical_scope_document:{id}");
                    edits.insert(
                        logical.clone(),
                        edit(
                            logical,
                            "physical_name_scopes".into(),
                            id.to_string(),
                            "physical_scope_document".into(),
                            payload(&entry)?,
                            Vec::new(),
                        ),
                    );
                }
            }
            parent.frame.origins = Some(parsed.origins(&parent.members, &edits)?);
        } else if let Some(origins) = &mut parent.frame.origins {
            origins.invalidate_fits(&fit_intents);
        }
        let names = selected_source::physical_bindings(
            &parent.rows,
            &parent.physical,
            &self.physical.quantities,
        );
        let identity = crate::math::modeling::source_revision_frame(
            &parent.rows,
            parent.frame.data.iter().map(|(id, hash)| (*id, *hash)),
            &self.physical.key,
            &names,
        );
        parent.frame.order = parent.rows.iter().map(|r| r.declaration_id).collect();
        edits.insert(
            INGRESS_FRAME.into(),
            edit(
                INGRESS_FRAME.into(),
                "context".into(),
                "ingress_frame".into(),
                "ingress_frame".into(),
                payload(&parent.frame)?,
                Vec::new(),
            ),
        );
        edits.insert(
            CONTEXT_LOGICAL.into(),
            edit(
                CONTEXT_LOGICAL.into(),
                "context".into(),
                CONTEXT.into(),
                CONTEXT.into(),
                payload(&identity)?,
                Vec::new(),
            ),
        );
        // Removed document-scoped name visibility is explicit; unrelated scope
        // members retain their original immutable versions.
        for id in delta.documents.removals {
            if let Some(member) = parent.members.get(&format!("physical_scope_document:{id}")) {
                edits.insert(member.logical.clone(), removal(member));
            }
        }
        let edits=edits.into_values().filter(|edit|match (&edit.version,self.revision.admitted.objects.get(&edit.logical)) {
            (None,None)=>false,
            (Some(version),Some(old))=>old.scope!=edit.scope || old.name!=edit.name || old.version!=version.key || old.kind!=version.kind || old.payload.as_slice()!=version.payload.as_slice() || old.references!=edit.references,
            _=>true,
        }).collect::<Vec<_>>();
        if edits.is_empty() && identity == self.revision.identity {
            return Ok(self.clone());
        }
        checkpoint(cancel)?;
        let revision = if record {
            admit_revision(&self.runtime, Some(&self.revision), identity, &edits, &typed, Some(cancel)).await?
        } else {
            let admitted = AdmittedModelRevision::apply_inputs(&self.runtime, Some(&self.revision.admitted), &edits, &typed, Some(cancel))?;
            SourceRevision::admitted(admitted, identity, None, None)?
        };
        Ok(Self {revision,held_preparation:None,..self.clone()})

        }).await
    }
}

fn removal(member: &SourceMember) -> ObjectEdit {
    ObjectEdit {
        logical: member.logical.clone(),
        scope: member.scope.clone(),
        name: member.name.clone(),
        version: None,
        references: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::SourceDocumentDelta;

    async fn document_package(text: &str, data: Option<Vec<u8>>) -> ModelingPackage {
        let mut sources = BTreeMap::from([
            (
                "package.toml".into(),
                include_bytes!(
                    "../../../../../../tests/fixtures/packages/minimal_named/package.toml"
                )
                .to_vec(),
            ),
            ("models/bank.pse".into(), text.as_bytes().to_vec()),
        ]);
        if let Some(data) = data {
            sources.insert("data/bank.parquet".into(), data);
        }
        crate::workflow::tests::runtime()
            .package_from_sources(
                &[sources],
                crate::workflow::tests::physical(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap()
    }

    async fn parent_metadata(package: &ModelingPackage) -> Parent {
        package
            .delta_parent(&crate::CancelSource::new())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn local_delta_document_only_replacement_retires_exact_text_and_spans() {
        let package = document_package("package minimal_named { param a:Scalar=1; }", None).await;
        let before = parent_metadata(&package).await;
        let mut document = before
            .documents
            .values()
            .find(|row| row.path == "models/bank.pse" && row.source_text.is_some())
            .unwrap()
            .clone();
        let document_id = document.document_id;
        let declarations = before
            .rows
            .iter()
            .filter(|row| row.document_id == document_id)
            .map(|row| row.declaration_id)
            .collect::<BTreeSet<_>>();
        assert!(!declarations.is_empty());
        assert!(declarations.iter().any(|id| before.fields.contains_key(id)));
        document.source_text = None;
        let changed = package
            .with_delta(
                SourceDelta {
                    parent: package.revision_id().into(),
                    documents: SourceDocumentDelta {
                        replacements: vec![document],
                        ..Default::default()
                    },
                    ..Default::default()
                },
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let after = parent_metadata(&changed).await;
        assert_eq!(
            before.rows, after.rows,
            "document intent does not alter declaration values or order"
        );
        assert!(after.documents[&document_id].source_text.is_none());
        assert!(
            declarations.iter().all(|id| !after.fields.contains_key(id)),
            "spans cannot outlive their exact document text"
        );
        assert_eq!(changed.revision.identity, package.revision.identity);
        assert_ne!(changed.revision_id(), package.revision_id());
        assert!(
            parent_metadata(&package).await.documents[&document_id]
                .source_text
                .is_some(),
            "the admitted parent remains immutable"
        );
    }

    #[tokio::test]
    async fn local_delta_unrelated_document_edit_retains_data_without_readmission() {
        use datafusion::arrow::{
            array::Int64Array,
            datatypes::{DataType, Field, Schema},
            record_batch::RecordBatch,
        };
        let batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("n", DataType::Int64, false),
                Field::new("value", DataType::Int64, false),
            ])),
            vec![
                Arc::new(Int64Array::from(vec![1])),
                Arc::new(Int64Array::from(vec![2])),
            ],
        )
        .unwrap();
        let mut bytes = Vec::new();
        let mut writer =
            parquet::arrow::ArrowWriter::try_new(&mut bytes, batch.schema(), None).unwrap();
        writer.write(&batch).unwrap();
        writer.close().unwrap();
        let package = document_package("package minimal_named { entity kind source provenance {} entity source s {} enum role {given} entity kind node {key n:Integer; attribute value:Integer;} dataset bank:node provenance(s,role.given) from \"data/bank.parquet\"; }", Some(bytes)).await;
        let before = parent_metadata(&package).await;
        let owner = before.documents.values().next().unwrap().package_id;
        let path = "models/unrelated.pse";
        let document = document_wire::Row {
            document_id: pse_ids::named_id(owner.as_id(), path),
            package_id: owner,
            path: path.into(),
            source_text: None,
            content: None,
        };
        // A cold receiver admits metadata work but cannot admit even one compiler
        // input byte. Revalidating an unchanged supplier would therefore refuse.
        let small = crate::workflow::tests::runtime_on(
            256 << 20,
            crate::math::MathPolicy {
                workspace_bytes: 1,
                ..Default::default()
            },
        );
        let receiver =
            Runtime::from_shared_local(small.shared.clone(), small.model.clone()).unwrap();
        let confined = ModelingPackage {
            runtime: receiver,
            ..package.clone()
        };
        assert!(
            confined.all_revision().await.is_err(),
            "the control really refuses full data admission"
        );
        let changed = confined
            .with_delta(
                SourceDelta {
                    parent: package.revision_id().into(),
                    documents: SourceDocumentDelta {
                        additions: vec![document],
                        ..Default::default()
                    },
                    ..Default::default()
                },
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let after = parent_metadata(&changed).await;
        let retained = before
            .members
            .iter()
            .filter(|(logical, _)| logical.starts_with("data:") || logical.starts_with("record:"))
            .collect::<Vec<_>>();
        assert!(
            retained
                .iter()
                .any(|(logical, _)| logical.starts_with("data:"))
        );
        assert!(
            retained
                .iter()
                .any(|(logical, _)| logical.starts_with("record:"))
        );
        for (logical, member) in retained {
            assert_eq!(after.members[logical].version, member.version);
        }
        assert_eq!(changed.revision.identity, package.revision.identity);
        assert_eq!(after.frame.data, before.frame.data);
    }

    #[test]
    fn delta_intents_reject_overlap_and_wrong_existence_before_effects() {
        let existing = BTreeSet::from([1, 2]);
        assert!(intentions(&existing, [3].into_iter(), [2].into_iter(), [1].into_iter()).is_ok());
        for (add, replace, remove) in [
            (vec![1], vec![], vec![]),
            (vec![], vec![3], vec![]),
            (vec![], vec![], vec![3]),
            (vec![3, 3], vec![], vec![]),
            (vec![], vec![1], vec![1]),
        ] {
            assert!(
                intentions(
                    &existing,
                    add.into_iter(),
                    replace.into_iter(),
                    remove.into_iter()
                )
                .is_err()
            );
        }
    }

    #[tokio::test]
    async fn local_delta_preserves_order_parent_and_replaces_stable_name_references() {
        let runtime = crate::workflow::tests::runtime();
        let physical = crate::workflow::tests::physical();
        let parse = |source: &str| {
            pse_authoring::language::parse(
                source,
                SemanticId::NIL,
                pse_authoring::language::IdentityPolicy::Named,
                Default::default(),
            )
            .unwrap()
        };
        let old = parse(
            "package p { param a:Scalar=1; param b:Scalar=2; def Root { var x:Scalar; eq e:x==a; } }",
        );
        let fresh = parse(
            "package p { param a:Scalar=1; param b:Scalar=2; def Root { var x:Scalar; eq e:x==b; } }",
        );
        let root = old
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(old.clone(), physical.clone())
            .await
            .unwrap();
        let cancel = crate::CancelSource::new();
        let unchanged = package
            .with_delta(
                SourceDelta {
                    parent: package.revision_id().into(),
                    ..Default::default()
                },
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(unchanged.revision_id(), package.revision_id());
        let new_equation = fresh.iter().find(|row| row.name == "e").unwrap().clone();
        let changed = package
            .with_delta(
                SourceDelta {
                    parent: package.revision_id().into(),
                    replacements: vec![new_equation],
                    ..Default::default()
                },
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(changed.revision.lineage_id(), package.revision.lineage_id());
        let independent = runtime
            .modeling_package(fresh.clone(), physical)
            .await
            .unwrap();
        assert_eq!(changed.revision.identity, independent.revision.identity);
        assert_eq!(changed.full_source().await.unwrap().0, fresh);
        assert_eq!(package.full_source().await.unwrap().0, old);
        let selected = changed.checked_selection(&[root], &cancel).await.unwrap();
        let names = selected
            .admission
            .revision
            .declarations()
            .iter()
            .map(|row| row.name.as_str())
            .collect::<Vec<_>>();
        assert!(names.contains(&"b"));
        assert!(!names.contains(&"a"));
        let no_op = changed
            .with_delta(
                SourceDelta {
                    parent: changed.revision_id().into(),
                    ..Default::default()
                },
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(no_op.revision_id(), changed.revision_id());
        assert!(
            changed
                .with_delta(
                    SourceDelta {
                        parent: package.revision_id().into(),
                        ..Default::default()
                    },
                    &cancel
                )
                .await
                .is_err()
        );
    }
}
