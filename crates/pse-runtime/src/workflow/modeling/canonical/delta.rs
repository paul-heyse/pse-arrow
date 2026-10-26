// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact-parent delta ingress. Unchanged data bytes never enter the ordinary
//! metadata path; V4 is framed from its authenticated ordered parent metadata.
use super::*;
use crate::workflow::SourceDelta;
use pse_model::generated::authored::documents as document_wire;

#[derive(Clone)]
pub(in crate::workflow::modeling) struct Parent {
    pub(in crate::workflow::modeling) rows: Vec<Declaration>,
    pub(super) frame: IngressFrame,
    pub(super) physical: PhysicalScope,
    pub(in crate::workflow::modeling) documents: BTreeMap<SemanticId, document_wire::Row>,
    pub(super) members: BTreeMap<String, SourceMember>,
    pub(super) fields: BTreeMap<DeclarationId, FieldSpans>,
    pub(super) _leases: Vec<Arc<pse_columnar::AllocationLease>>,
    product_owner: Option<Arc<pse_columnar::AllocationLease>>,
}

/// Reconstructible products of one completely admitted immutable revision.
/// These never supply transport trust or publication authority.
pub(super) struct Products {
    parent: Parent,
    index: SourceIndex,
    data: Vec<Declaration>,
    dependencies: BTreeMap<DeclarationId, BTreeSet<Request>>,
    extent: usize,
    #[cfg(test)]
    work: (usize, usize, usize),
}
impl std::fmt::Debug for Products {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeltaProducts")
            .field("declarations", &self.parent.rows.len())
            .finish_non_exhaustive()
    }
}
impl Products {
    fn copy_parent(
        &self,
        services: &crate::workflow::PreparationServices,
    ) -> Result<Parent, WorkflowError> {
        let owner = services
            .math()
            .reserve("modeling:delta-product-copy", self.extent)?;
        let mut parent = self.parent.clone();
        if let Some(previous) = parent.product_owner.take() {
            parent
                ._leases
                .retain(|lease| !Arc::ptr_eq(lease, &previous));
        }
        parent._leases.push(owner.clone());
        parent.product_owner = Some(owner);
        Ok(parent)
    }
}
fn product_extent(
    revision: &AdmittedModelRevision,
    dependencies: &BTreeMap<DeclarationId, BTreeSet<Request>>,
) -> Result<usize, WorkflowError> {
    let metadata = revision
        .objects
        .values()
        .filter(|object| object.kind != "data")
        .try_fold(65536usize, |bytes, object| {
            bytes
                .checked_add(object.payload.len().checked_mul(32)?)
                .and_then(|bytes| bytes.checked_add(4096))
        })
        .ok_or_else(|| contract("delta product metadata extent"))?;
    dependencies
        .values()
        .flatten()
        .try_fold(metadata, |bytes, read| {
            let name = match read {
                Request::Name(_, name) => name.capacity(),
                _ => 0,
            };
            bytes
                .checked_add(size_of::<Request>() + 128 + name)
                .ok_or_else(|| contract("delta dependency extent"))
        })
}

impl SourceIndex {
    /// Full ingress indexes have no pending discovery. Clone immutable admitted
    /// products; changed rows replace only their buckets and parsed occurrences.
    fn copy_complete(&self) -> Self {
        Self {
            rows: self.rows.clone(),
            fields: self.fields.clone(),
            parsed_fields: self.parsed_fields.clone(),
            children: self.children.clone(),
            named_children: self.named_children.clone(),
            named_roots: self.named_roots.clone(),
            guards: self.guards.clone(),
            imported_names: self.imported_names.clone(),
            names: self.names.clone(),
            imports: self.imports.clone(),
            scopes: self.scopes.clone(),
            bases: self.bases.clone(),
            roots_complete: self.roots_complete,
            versions: self.versions.clone(),
            record_sources: self.record_sources.clone(),
            ..Default::default()
        }
    }
    fn remove_complete(&mut self, id: DeclarationId) {
        let Some(row) = self.rows.remove(&id) else {
            return;
        };
        self.fields.remove(&id);
        self.parsed_fields.remove(&id);
        self.bases.remove(&id);
        self.scopes.remove(&id);
        self.imports.remove(&id);
        if let Some(parent) = row.parent_id {
            if let Some(children) = self.children.get_mut(&parent) {
                children.retain(|child| *child != id);
            }
            if let Some(names) = self.named_children.get_mut(&parent)
                && let Some(children) = names.get_mut(&row.name)
            {
                children.retain(|child| *child != id);
            }
            if let Some(guards) = self.guards.get_mut(&parent) {
                guards.retain(|child| *child != id);
            }
            if let Some(import) = &row.value.import
                && let Some(imports) = self
                    .imported_names
                    .get_mut(&parent)
                    .and_then(|names| names.get_mut(import.alias.as_deref().unwrap_or(&row.name)))
                && let Some(position) = imports.iter().position(|name| *name == row.name)
            {
                imports.remove(position);
            }
        } else if let Some(roots) = self.named_roots.get_mut(&row.name) {
            roots.retain(|root| *root != id);
        }
        self.invalidate_members();
    }
    fn traced_references(
        &mut self,
        row: &Declaration,
    ) -> Result<(BTreeSet<DeclarationId>, BTreeSet<Request>), WorkflowError> {
        *self
            .trace
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(BTreeSet::new());
        let result = self.references(row);
        let reads = self
            .trace
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .unwrap_or_default();
        Ok((result?, reads))
    }
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

/// Owner-qualified source preimages. Constructed only from immutable admitted inputs;
/// never deserialized from a public SourceDelta or a saved editing document.
pub(in crate::workflow::modeling) struct AdmittedDocumentComposition {
    documents: BTreeMap<SemanticId, document_wire::Row>,
    fields: BTreeMap<DeclarationId, FieldSpans>,
    origins: BTreeMap<SemanticId, Option<reload::LoaderOrigins>>,
    packages: Vec<pse_model::generated::authored::packages::Row>,
    _owners: Vec<Parent>,
}
fn same_declaration_meaning(a: &Declaration, b: &Declaration) -> bool {
    let mut a = a.clone();
    a.source_start = b.source_start;
    a.source_end = b.source_end;
    a == *b
}

impl ModelingPackage {
    pub(in crate::workflow::modeling) async fn delta_parent(
        &self,
        services: &crate::workflow::PreparationServices,
        cancel: &crate::CancelSource,
    ) -> Result<Parent, WorkflowError> {
        self.check_service_bindings(services)?;
        checkpoint(cancel)?;
        if let Some(products) = self.revision.delta_products.get() {
            return products.copy_parent(&services);
        }
        let (frame, frame_lease) = self
            .source_decode::<IngressFrame>(services, self.local_object(INGRESS_FRAME)?.as_ref())?;
        let (mut physical, physical_lease) = self.source_decode::<PhysicalScope>(
            services,
            self.local_object(PHYSICAL_SCOPE_LOGICAL)?.as_ref(),
        )?;
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
                let lease = services.math().reserve(
                    "modeling:delta-declaration-copy",
                    source.0.owned_bytes().saturating_add(8192),
                )?;
                index.insert(source)?;
                leases.push(lease);
            } else if object.kind == "document" {
                let (row, lease) = self.source_decode::<document_wire::Row>(services, object)?;
                if row.content.is_some() {
                    return Err(contract(
                        "document metadata unexpectedly contains data bytes",
                    ));
                }
                documents.insert(row.document_id, row);
                leases.push(lease);
            } else if object.kind == "physical_scope_document" {
                let (entry, lease) = self.source_decode::<PhysicalScope>(services, object)?;
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
        let mut parent = Parent {
            rows,
            frame,
            physical,
            documents,
            members,
            fields: index.fields,
            _leases: leases,
            product_owner: None,
        };
        let owner = services.math().reserve(
            "modeling:delta-products",
            parent
                .rows
                .owned_bytes()
                .saturating_mul(32)
                .saturating_add(65536),
        )?;
        parent._leases.push(owner.clone());
        parent.product_owner = Some(owner);
        let mut index = SourceIndex {
            roots_complete: true,
            ..Default::default()
        };
        for row in &parent.rows {
            index.insert((row.clone(), parent.fields.get(&row.declaration_id).cloned()))?;
            index.complete_scope(row.declaration_id);
            index.complete_imports(row.declaration_id);
        }
        // Typed ingress already owns parsed fields. Avoid serializing them again
        // merely to rediscover the same parsed-field cache key.
        for object in self.revision.admitted.objects.values() {
            if let (Some((row, _)), Some(parsed)) = (&object.declaration, &object.parsed) {
                index
                    .parsed_fields
                    .insert(row.declaration_id, parsed.clone());
            }
        }
        let mut dependencies = BTreeMap::new();
        for row in &parent.rows {
            checkpoint(cancel)?;
            let (_, reads) = index.traced_references(row)?;
            dependencies.insert(row.declaration_id, reads);
        }
        index.requests.clear();
        let data = index.data_closure()?;
        let extent = product_extent(&self.revision.admitted, &dependencies)?;
        let retained = services.math().reserve("modeling:delta-products", extent)?;
        parent._leases = vec![retained.clone()];
        parent.product_owner = Some(retained);
        let products = Arc::new(Products {
            parent,
            index,
            data,
            dependencies,
            extent,
            #[cfg(test)]
            work: (0, 0, 0),
        });
        let _ = self.revision.delta_products.set(products.clone());
        self.revision
            .delta_products
            .get()
            .unwrap_or(&products)
            .copy_parent(&services)
    }
    /// Stage an exact local parent edit; durable recording is a separate CAS effect.
    pub async fn with_delta(
        &self,
        services: &crate::workflow::SourceServices,
        delta: SourceDelta,
        cancel: &crate::CancelSource,
    ) -> Result<Self, WorkflowError> {
        self.check_service_bindings(services.preparation())?;
        let activity = services.preparation().admit_activity()?;
        activity
            .scope(async {
                if delta.parent != self.revision_id() {
                    return Err(contract("source delta names a different local parent"));
                }
                let deadline = services.preparation().math().preparation_deadline(cancel)?;
                let scoped = cancel.with_deadline(Some(deadline));
                let parent = self.delta_parent(services.preparation(), &scoped).await?;
                self.apply_delta_parent(services, delta, parent, &scoped, None)
                    .await
            })
            .await
    }

    /// Authenticate selected source text against complete admitted document ownership.
    /// A composed IR state lacking an exact preimage uses the existing absent-text convention.
    pub(in crate::workflow::modeling) async fn project_compose_documents(
        &self,
        services: &crate::workflow::SourceServices,
        delta: &SourceDelta,
        sources: &[&Self],
        cancel: &crate::CancelSource,
    ) -> Result<
        (
            SourceDelta,
            AdmittedDocumentComposition,
            Vec<super::super::project_comparison::ProjectDocumentComposition>,
        ),
        WorkflowError,
    > {
        self.check_service_bindings(services.preparation())?;
        if delta.parent != self.revision_id() {
            return Err(contract("composed delta names another exact parent"));
        }
        let parent = self.delta_parent(services.preparation(), cancel).await?;
        let mut rows = parent
            .rows
            .iter()
            .cloned()
            .map(|row| (row.declaration_id, row))
            .collect::<BTreeMap<_, _>>();
        for id in &delta.removals {
            rows.remove(id);
        }
        for row in delta.additions.iter().chain(&delta.replacements) {
            rows.insert(row.declaration_id, row.clone());
        }
        let mut documents = parent.documents.clone();
        for id in &delta.documents.removals {
            documents.remove(id);
        }
        for row in delta
            .documents
            .additions
            .iter()
            .chain(&delta.documents.replacements)
        {
            documents.insert(row.document_id, row.clone());
        }
        let mut fits = self
            .fit_declarations(services.preparation())
            .await?
            .fits
            .iter()
            .cloned()
            .map(|row| (row.fit_id, row))
            .collect::<BTreeMap<_, _>>();
        for id in &delta.fits.removals {
            fits.remove(id);
        }
        for row in delta.fits.additions.iter().chain(&delta.fits.replacements) {
            fits.insert(row.fit_id, row.clone());
        }
        let mut affected = delta
            .documents
            .additions
            .iter()
            .chain(&delta.documents.replacements)
            .map(|row| row.document_id)
            .collect::<BTreeSet<_>>();
        for row in delta.additions.iter().chain(&delta.replacements) {
            affected.insert(row.document_id);
        }
        for id in &delta.removals {
            if let Some(row) = parent.rows.iter().find(|row| row.declaration_id == *id) {
                affected.insert(row.document_id);
            }
        }
        // With no new fit-to-document claim, conservatively require the entire fit
        // inventory to match a source before retaining that source's text preimage.
        if !delta.fits.additions.is_empty()
            || !delta.fits.replacements.is_empty()
            || !delta.fits.removals.is_empty()
        {
            affected.extend(documents.keys());
        }
        let fit_payload = serde_json::to_vec(&fits)
            .map_err(|error| contract(format!("composed fit payload: {error}")))?;
        let mut owners = Vec::new();
        let mut source_fits = Vec::new();
        for source in sources {
            if source.physical.key != self.physical.key {
                return Err(contract("composed source physical context differs"));
            }
            owners.push(source.delta_parent(services.preparation(), cancel).await?);
            let rows = source
                .fit_declarations(services.preparation())
                .await?
                .fits
                .iter()
                .cloned()
                .map(|row| (row.fit_id, row))
                .collect::<BTreeMap<_, _>>();
            source_fits.push(
                serde_json::to_vec(&rows)
                    .map_err(|error| contract(format!("admitted fit payload: {error}")))?,
            );
        }
        let mut proof = AdmittedDocumentComposition {
            documents: BTreeMap::new(),
            fields: BTreeMap::new(),
            origins: BTreeMap::new(),
            packages: Vec::new(),
            _owners: Vec::new(),
        };
        let mut notes = Vec::new();
        let mut next = delta.clone();
        for id in affected {
            checkpoint(cancel)?;
            let Some(document) = documents.get(&id) else {
                continue;
            };
            if document.source_text.is_none() {
                continue;
            }
            let selected = owners
                .iter()
                .zip(&source_fits)
                .find(|(owner, owner_fits)| {
                    owner.documents.get(&id) == Some(document) && **owner_fits == fit_payload && {
                        let expected = owner
                            .rows
                            .iter()
                            .filter(|row| row.document_id == id)
                            .map(|row| (row.declaration_id, row))
                            .collect::<BTreeMap<_, _>>();
                        let actual = rows
                            .values()
                            .filter(|row| row.document_id == id)
                            .map(|row| (row.declaration_id, row))
                            .collect::<BTreeMap<_, _>>();
                        expected.len() == actual.len()
                            && expected.iter().all(|(key, row)| {
                                actual
                                    .get(key)
                                    .is_some_and(|actual| same_declaration_meaning(row, actual))
                            })
                    }
                })
                .map(|(owner, _)| owner);
            let retained = selected.is_some();
            if let Some(owner) = selected {
                proof.documents.insert(id, document.clone());
                proof.origins.insert(id, owner.frame.origins.clone());
                for row in owner.rows.iter().filter(|row| row.document_id == id) {
                    if let Some(fields) = owner.fields.get(&row.declaration_id) {
                        proof.fields.insert(row.declaration_id, fields.clone());
                    }
                    if let Some(added) = next
                        .additions
                        .iter_mut()
                        .find(|added| added.declaration_id == row.declaration_id)
                    {
                        *added = row.clone();
                    } else if let Some(replaced) = next
                        .replacements
                        .iter_mut()
                        .find(|replaced| replaced.declaration_id == row.declaration_id)
                    {
                        *replaced = row.clone();
                    } else if parent
                        .rows
                        .iter()
                        .find(|old| old.declaration_id == row.declaration_id)
                        .is_some_and(|old| old != row)
                    {
                        next.replacements.push(row.clone());
                    }
                }
            } else {
                let mut unavailable = document.clone();
                unavailable.source_text = None;
                if let Some(added) = next
                    .documents
                    .additions
                    .iter_mut()
                    .find(|row| row.document_id == id)
                {
                    *added = unavailable;
                } else if let Some(replaced) = next
                    .documents
                    .replacements
                    .iter_mut()
                    .find(|row| row.document_id == id)
                {
                    *replaced = unavailable;
                } else {
                    next.documents.replacements.push(unavailable);
                }
                proof.origins.insert(id, None);
            }
            notes.push(
                super::super::project_comparison::ProjectDocumentComposition {
                    logical: format!("document:{id}"),
                    original_text_retained: retained,
                },
            );
        }
        // Recompute only the existing loader's derived inventory checksum. All
        // authored package fields remain the target's admitted manifest declaration.
        // With any absent IR text, retain the existing IR convention instead of
        // inventing an original package byte preimage.
        for row in next
            .documents
            .additions
            .iter()
            .chain(&next.documents.replacements)
        {
            documents.insert(row.document_id, row.clone());
        }
        for object in self
            .revision
            .admitted
            .objects
            .values()
            .filter(|object| object.kind == "package")
        {
            let mut package: pse_model::generated::authored::packages::Row =
                decode(&object.payload)?;
            let changed_documents = next
                .documents
                .additions
                .iter()
                .chain(&next.documents.replacements)
                .map(|document| document.document_id)
                .collect::<BTreeSet<_>>();
            let mut inventory = BTreeMap::<String, &[u8]>::new();
            let mut complete = true;
            for document in documents
                .values()
                .filter(|document| document.package_id == package.package_id)
            {
                let bytes = if let Some(text) = &document.source_text {
                    Some(text.as_bytes())
                } else if let Some(content) = &document.content {
                    Some(content.as_slice())
                } else if !changed_documents.contains(&document.document_id) {
                    self.revision
                        .admitted
                        .objects
                        .get(&format!("data:{}", document.document_id))
                        .map(|object| object.payload.as_slice())
                } else {
                    None
                };
                match bytes {
                    Some(bytes) => {
                        inventory.insert(document.path.clone(), bytes);
                    }
                    None => {
                        complete = false;
                        break;
                    }
                }
            }
            if complete && inventory.contains_key("package.toml") {
                package.content_hash =
                    pse_admission::authoring_driver::document::package_checksum(&inventory);
                proof.packages.push(package);
            }
        }
        proof._owners = owners;
        Ok((next, proof, notes))
    }
    pub(in crate::workflow::modeling) async fn project_admit_composed_delta(
        &self,
        services: &crate::workflow::SourceServices,
        delta: SourceDelta,
        composition: AdmittedDocumentComposition,
        cancel: &crate::CancelSource,
    ) -> Result<Self, WorkflowError> {
        self.check_service_bindings(services.preparation())?;
        if delta.parent != self.revision_id() {
            return Err(contract("composed delta names another exact parent"));
        }
        let parent = self.delta_parent(services.preparation(), cancel).await?;
        self.apply_delta_parent_ordered_mode(
            services,
            delta,
            parent,
            cancel,
            None,
            None,
            false,
            Some(composition),
        )
        .await
    }

    /// Admit a proposal without publishing a durable source head. Project merge and undo
    /// issue their separately retained publication intent only after this succeeds.
    pub(crate) async fn project_admit_delta(
        &self,
        services: &crate::workflow::SourceServices,
        delta: &SourceDelta,
        cancel: &crate::CancelSource,
    ) -> Result<Self, WorkflowError> {
        self.check_service_bindings(services.preparation())?;
        if delta.parent != self.revision_id() {
            return Err(contract("project delta names a different exact parent"));
        }
        let parent = self.delta_parent(services.preparation(), cancel).await?;
        self.apply_delta_parent_ordered_mode(
            services,
            delta.clone(),
            parent,
            cancel,
            None,
            None,
            false,
            None,
        )
        .await
    }

    pub(super) async fn apply_delta_parent(
        &self,
        services: &crate::workflow::SourceServices,
        delta: SourceDelta,
        parent: Parent,
        cancel: &crate::CancelSource,
        parsed: Option<reload::ParsedEdit>,
    ) -> Result<Self, WorkflowError> {
        self.check_service_bindings(services.preparation())?;
        self.apply_delta_parent_ordered(services, delta, parent, cancel, parsed, None)
            .await
    }
    pub(in crate::workflow) async fn construct_study_variant(
        &self,
        services: &crate::workflow::SourceServices,
        source: &crate::workflow::OperationSource,
        mut delta: SourceDelta,
        cancel: &crate::CancelSource,
    ) -> Result<Self, WorkflowError> {
        self.check_service_bindings(services.preparation())?;
        source.check(self)?;
        // The descriptor already checked its original local parent token. Reconstruction
        // rebinds only after the complete immutable source/context identity agrees.
        delta.parent = self.revision_id().into();
        let deadline = services.preparation().math().preparation_deadline(cancel)?;
        let scoped = cancel.with_deadline(Some(deadline));
        let parent = self.delta_parent(services.preparation(), &scoped).await?;
        self.apply_delta_parent_ordered_mode(
            services, delta, parent, &scoped, None, None, false, None,
        )
        .await
    }

    pub(in crate::workflow::modeling) async fn apply_delta_parent_ordered(
        &self,
        services: &crate::workflow::SourceServices,
        delta: SourceDelta,
        parent: Parent,
        cancel: &crate::CancelSource,
        parsed: Option<reload::ParsedEdit>,
        order: Option<Vec<DeclarationId>>,
    ) -> Result<Self, WorkflowError> {
        self.check_service_bindings(services.preparation())?;
        self.apply_delta_parent_ordered_mode(
            services, delta, parent, cancel, parsed, order, true, None,
        )
        .await
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "One admitted source composition preserves its original parent, parsed order, cancellation and publication mode without a parallel edit path"
    )]
    async fn apply_delta_parent_ordered_mode(
        &self,
        services: &crate::workflow::SourceServices,
        delta: SourceDelta,
        mut parent: Parent,
        cancel: &crate::CancelSource,
        parsed: Option<reload::ParsedEdit>,
        order: Option<Vec<DeclarationId>>,
        record: bool,
        composition: Option<AdmittedDocumentComposition>,
    ) -> Result<Self, WorkflowError> {
        let activity = services.preparation().admit_activity()?;
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
        let _scratch = services.preparation()
            .math()
            .reserve("modeling:source-delta", extent)?;
        let validation = services.preparation().admission_context();
        let mut declarations =
            pse_relations::generated::authored::modeling_declarations::Builder::with_registry(
                services.preparation().registry(),
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
                services.preparation().registry(),
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
            if parsed.is_none() && row.source_text.is_some() && !composition.as_ref().is_some_and(|proof|proof.documents.get(&row.document_id)==Some(row)) {
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
                services.preparation().registry(),
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

        let products = self.revision.delta_products.get().ok_or_else(|| contract("delta products absent after exact parent admission"))?;
        let old_data = &products.data;
        let old_data_documents = data_documents(old_data, &parent.documents)?;
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
        if let Some(proof)=&composition { parent.fields.extend(proof.fields.clone()); }
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
                            &services.preparation().pool(),
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
            } else if let Some(exact)=composition.as_ref().and_then(|proof|proof.documents.get(id)) {
                *row=exact.clone();
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
        let mut index = products.index.copy_complete();
        #[cfg(test)]
        let mut work = (0, 0, 0);
        let mut invalidated = changed.iter().copied().map(Request::Logical).collect::<BTreeSet<_>>();
        let mut source_changes = BTreeSet::new();
        let mut namespace_changed = false;
        let current = parent.rows.iter().map(|row| (row.declaration_id, row)).collect::<BTreeMap<_, _>>();
        for id in products.index.rows.keys().chain(current.keys()).copied().collect::<BTreeSet<_>>() {
            let before = products.index.rows.get(&id);
            let after = current.get(&id).copied();
            if before == after && products.index.fields.get(&id) == parent.fields.get(&id) { continue; }
            source_changes.insert(id);
            #[cfg(test)] { work.2 += 1; }
            let same_namespace = before.zip(after).is_some_and(|(a, b)| a.parent_id == b.parent_id && a.name == b.name && a.value.kind == b.value.kind && a.value.import == b.value.import);
            if !same_namespace {
                namespace_changed = true;
                for row in before.into_iter().chain(after) {
                    invalidated.insert(Request::Name(row.parent_id, row.name.clone()));
                    if let Some(parent) = row.parent_id {
                        if row.value.import.is_some() { invalidated.insert(Request::Imports(parent)); }
                        if row.value.kind == Kind::When { invalidated.insert(Request::Members(parent)); }
                    }
                }
            }
            index.remove_complete(id);
            if let Some(row) = after {
                let fields = parent.fields.get(&id).cloned();
                index.insert((row.clone(), fields.clone()))?;
                index.parsed_fields.insert(id, parsed_fields(&services.preparation(), row, fields.as_ref())?);
                index.complete_scope(id); index.complete_imports(id);
            }
        }
        let ranks = parent.rows.iter().enumerate().map(|(rank, row)| (row.declaration_id, rank)).collect::<BTreeMap<_, _>>();
        for ids in index.children.values_mut().chain(index.guards.values_mut()).chain(index.named_children.values_mut().flat_map(|names| names.values_mut())) {
            ids.sort_by_key(|id| ranks.get(id).copied());
        }
        index.requests.clear();
        inventory.parsed_fields = index.parsed_fields.clone();
        let mut dependencies = products.dependencies.clone();
        dependencies.retain(|id, _| current.contains_key(id));
        for row in &parent.rows {
            checkpoint(cancel)?;
            let affected = source_changes.contains(&row.declaration_id)
                || dependencies.get(&row.declaration_id).is_none_or(|reads| !reads.is_disjoint(&invalidated))
                || parent.physical != original_physical;
            if !affected { continue; }
            if let Some(context) = pse_diagnostics::observation::current() { context.count("modeling.delta.examined_references", "declarations", 1); }
            #[cfg(test)] { work.0 += 1; }
            let (references, reads) = index.traced_references(row)?;
            dependencies.insert(row.declaration_id, reads);
            let references = references.into_iter().map(|id| {
                let target = &index.rows[&id];
                (scope(target.parent_id), target.name.clone())
            }).collect::<Vec<_>>();
            let previous = self.revision.admitted.objects.get(&logical(row.declaration_id));
            if !source_changes.contains(&row.declaration_id) && previous.is_some_and(|old| old.references == references) { continue; }
            if let Some(context) = pse_diagnostics::observation::current() { context.count("modeling.delta.canonical_declarations", "declarations", 1); }
            #[cfg(test)] { work.1 += 1; }
            edits.insert(logical(row.declaration_id), typed.declaration(row, parent.fields.get(&row.declaration_id).cloned(), index.parsed_fields[&row.declaration_id].clone(), references)?);
        }
        for id in removals {
            let member = parent
                .members
                .get(&logical(id))
                .ok_or_else(|| contract("removed declaration membership absent"))?;
            edits.insert(member.logical.clone(), removal(member));
        }
        index.requests.clear();
        // Data closure depends on resolved namespace and actual supplier rows.
        // Source-only scalar/default edits outside the retained closure reuse it.
        let old_data_ids = old_data.iter().map(|row| row.declaration_id).collect::<BTreeSet<_>>();
        let data_changed = namespace_changed || changed.iter().any(|id| old_data_ids.contains(id) || current.get(id).is_some_and(|row| row.value.entity.is_some() || row.value.dataset.is_some() || matches!(row.value.kind, Kind::EntityKind | Kind::IdentifierScheme)));
        let new_data = if data_changed { index.data_closure()? } else { old_data.clone() };
        let new_data_documents = data_documents(&new_data, &parent.documents)?;
        if &new_data != old_data
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
                                &services.preparation().pool(),
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
                let workspace = services.preparation().math().workspace(
                    super::super::compiler_context(&self.physical, &BTreeMap::new()),
                    pse_compiler::workspace::WorkspaceLimits::default(),
                )?;
                let data = services.preparation().math().modeling_revision(
                    &workspace,
                    new_data.clone(),
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
        if let Some(proof)=&composition {
            let mut builder=pse_relations::generated::authored::packages::Builder::with_registry(services.preparation().registry(),proof.packages.len(),&validation).map_err(super::super::super::relation)?;
            for package in &proof.packages {builder.push(package.clone()).map_err(super::super::super::relation)?;}
            drop(builder.finish().map_err(super::super::super::relation)?);
            for package in &proof.packages {let entry=typed.package(package.clone())?;edits.insert(entry.logical.clone(),entry);}
            let origins=parent.frame.origins.get_or_insert_with(reload::LoaderOrigins::empty);
            for (document,source) in &proof.origins { origins.replace_document_from(*document,source.as_ref()); }
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
            admit_revision(services, Some(&self.revision), identity, &edits, &typed, Some(cancel)).await?
        } else {
            let admitted = AdmittedModelRevision::apply_inputs(&services.preparation(), Some(&self.revision.admitted), &edits, &typed, Some(cancel))?;
            SourceRevision::admitted(admitted, identity, None, None, self.revision.issuer.clone())?
        };
        let extent = product_extent(&revision.admitted, &dependencies)?;
        let retained = match parent.product_owner.take().filter(|owner| owner.size() >= extent) {
            Some(owner) => owner,
            None => services.preparation().math().reserve("modeling:delta-products", extent)?,
        };
        parent._leases = vec![retained.clone()];
        parent.product_owner = Some(retained);
        parent.members = revision.admitted.objects.values().map(|object| (object.logical.clone(), SourceMember::from(object.as_ref()))).collect();
        let next_products = Arc::new(Products { parent, index, data: new_data, dependencies, extent, #[cfg(test)] work });
        let _ = revision.delta_products.set(next_products);
        Ok(Self {revision,held_preparation:None,research_source:None,project_context:None,..self.clone()})

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

    async fn document_package(
        runtime: &Runtime,
        text: &str,
        data: Option<Vec<u8>>,
    ) -> ModelingPackage {
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
        runtime
            .package_from_sources(
                &[sources],
                crate::workflow::tests::physical(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap()
    }

    async fn parent_metadata(
        services: &crate::workflow::PreparationServices,
        package: &ModelingPackage,
    ) -> Parent {
        package
            .delta_parent(services, &crate::CancelSource::new())
            .await
            .unwrap()
    }

    fn parse(text: &str) -> Vec<Declaration> {
        pse_authoring::language::parse(
            text,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap()
    }
    async fn compare_fresh(
        services: &crate::workflow::SourceServices,
        package: &ModelingPackage,
        rows: Vec<Declaration>,
    ) {
        let fresh = services
            .modeling_package(rows.clone(), crate::workflow::tests::physical())
            .await
            .unwrap();
        assert_eq!(package.revision.identity(), fresh.revision.identity());
        assert_eq!(
            &*package.declarations(services.preparation()).await.unwrap(),
            rows.as_slice()
        );
        for row in rows {
            let key = logical(row.declaration_id);
            assert_eq!(
                package.revision.admitted.objects[&key].references,
                fresh.revision.admitted.objects[&key].references,
                "fresh binding oracle: {}",
                row.name
            );
            assert_eq!(
                package.revision.admitted.objects[&key].payload,
                fresh.revision.admitted.objects[&key].payload
            );
        }
    }

    #[tokio::test]
    async fn delta_products_local_default_reuses_unrelated_parsed_and_canonical_products() {
        for inventory in [8, 32, 128] {
            let mut text =
                "package p { def Root { param k:Scalar=1; var x:Scalar; eq a:x==k; }".to_owned();
            for i in 0..inventory {
                text.push_str(&format!("param spare{i}:Scalar={i};"));
            }
            text.push('}');
            let mut rows = parse(&text);
            let runtime = crate::workflow::tests::runtime();
            let original = runtime
                .modeling_package(rows.clone(), crate::workflow::tests::physical())
                .await
                .unwrap();
            let _ = original
                .delta_parent(&runtime.preparation_services(), &crate::CancelSource::new())
                .await
                .unwrap();
            let retained = original.revision.delta_products.get().unwrap();
            let id = rows
                .iter()
                .find(|row| row.name == "k")
                .unwrap()
                .declaration_id;
            let spare = rows
                .iter()
                .find(|row| row.name == "spare0")
                .unwrap()
                .declaration_id;
            let mut replacement = parse("package p { def Root { param k:Scalar=2; } }")
                .into_iter()
                .find(|row| row.name == "k")
                .unwrap();
            let old = rows
                .iter_mut()
                .find(|row| row.declaration_id == id)
                .unwrap();
            replacement.source_start = old.source_start;
            replacement.source_end = old.source_end;
            *old = replacement.clone();
            let changed = original
                .with_delta(
                    &runtime.source_services(),
                    SourceDelta {
                        parent: original.revision_id().into(),
                        replacements: vec![replacement],
                        ..Default::default()
                    },
                    &crate::CancelSource::new(),
                )
                .await
                .unwrap();
            let products = changed.revision.delta_products.get().unwrap();
            assert_eq!(
                products.work,
                (1, 1, 1),
                "unrelated inventory cannot increase local reference/serialization/index work"
            );
            assert!(Arc::ptr_eq(
                &retained.index.parsed_fields[&spare],
                &products.index.parsed_fields[&spare]
            ));
            assert!(Arc::ptr_eq(
                &original.revision.admitted.objects[&logical(spare)],
                &changed.revision.admitted.objects[&logical(spare)]
            ));
            eprintln!(
                "delta-products inventory={inventory} examined={} canonical={} updated_index_rows={} copied_index_rows={} retained={}",
                products.work.0,
                products.work.1,
                products.work.2,
                products.index.rows.len(),
                products.parent.product_owner.as_ref().unwrap().size()
            );
            compare_fresh(&runtime.source_services(), &changed, rows).await;
        }
    }

    #[tokio::test]
    async fn delta_products_absence_shadow_import_delete_order_and_span_match_fresh() {
        let sources = [
            "package q { param x:Scalar=3; } package p { param x:Scalar=1; def Root { eq a:x==missing; } }",
            "package q { param x:Scalar=3; } package p { param x:Scalar=1; def Root { param missing:Scalar=4; eq a:x==missing; } }",
            "package q { param x:Scalar=3; } package p { param x:Scalar=1; def Root { param x:Scalar=2; param missing:Scalar=4; eq a:x==missing; } }",
            "package q { param x:Scalar=3; } package p { use q @ \"1.0.0\"; param x:Scalar=1; def Root { param missing:Scalar=4; eq a:q.x==missing; } }",
            "package q { param x:Scalar=3; } package p { param x:Scalar=1; def Root { eq a:x==missing; } }",
        ];
        let runtime = crate::workflow::tests::runtime();
        let mut current = runtime
            .modeling_package(parse(sources[0]), crate::workflow::tests::physical())
            .await
            .unwrap();
        for source in &sources[1..] {
            let rows = parse(source);
            current = current
                .with_declarations(&runtime.source_services(), rows.clone())
                .await
                .unwrap();
            compare_fresh(&runtime.source_services(), &current, rows).await;
        }
        let mut rows = current
            .declarations(&runtime.preparation_services())
            .await
            .unwrap()
            .to_vec();
        rows.reverse();
        current = current
            .with_declarations(&runtime.source_services(), rows.clone())
            .await
            .unwrap();
        compare_fresh(&runtime.source_services(), &current, rows.clone()).await;
        for row in &mut rows {
            row.source_start += 7;
            row.source_end += 7;
        }
        current = current
            .with_declarations(&runtime.source_services(), rows.clone())
            .await
            .unwrap();
        compare_fresh(&runtime.source_services(), &current, rows).await;
    }

    #[tokio::test]
    async fn local_delta_document_only_replacement_retires_exact_text_and_spans() {
        let runtime = crate::workflow::tests::runtime();
        let package = document_package(
            &runtime,
            "package minimal_named { param a:Scalar=1; }",
            None,
        )
        .await;
        let before = parent_metadata(&runtime.preparation_services(), &package).await;
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
                &runtime.source_services(),
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
        let after = parent_metadata(&runtime.preparation_services(), &changed).await;
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
            parent_metadata(&runtime.preparation_services(), &package)
                .await
                .documents[&document_id]
                .source_text
                .is_some(),
            "the admitted parent remains immutable"
        );
    }

    #[tokio::test]
    async fn local_delta_unrelated_document_edit_retains_data_without_readmission() {
        let runtime = crate::workflow::tests::runtime();
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
        let package = document_package(&runtime, "package minimal_named { entity kind source provenance {} entity source s {} enum role {given} entity kind node {key n:Integer; attribute value:Integer;} dataset bank:node provenance(s,role.given) from \"data/bank.parquet\"; }", Some(bytes)).await;
        let before = parent_metadata(&runtime.preparation_services(), &package).await;
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
        let confined = package
            .replay_source()
            .unwrap()
            .reconstruct(
                &receiver.source_services(),
                package.physical.clone(),
                (*package.providers).clone(),
                &crate::CancelSource::new(),
            )
            .unwrap();
        assert!(
            confined
                .all_revision(&receiver.preparation_services())
                .await
                .is_err(),
            "the control really refuses full data admission"
        );
        let changed = confined
            .with_delta(
                &receiver.source_services(),
                SourceDelta {
                    parent: confined.revision_id().into(),
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
        let after = parent_metadata(&receiver.preparation_services(), &changed).await;
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
                &runtime.source_services(),
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
                &runtime.source_services(),
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
        assert_eq!(
            changed
                .full_source(&runtime.preparation_services(),)
                .await
                .unwrap()
                .0,
            fresh
        );
        assert_eq!(
            package
                .full_source(&runtime.preparation_services(),)
                .await
                .unwrap()
                .0,
            old
        );
        let selected = changed
            .checked_selection(&runtime.preparation_services(), &[root], &cancel)
            .await
            .unwrap();
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
                &runtime.source_services(),
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
                    &runtime.source_services(),
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
