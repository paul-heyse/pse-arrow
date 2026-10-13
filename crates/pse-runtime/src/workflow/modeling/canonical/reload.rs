// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Loader-issued document ownership and exact-parent parsed reload.
use super::*;
use pse_admission::authoring_driver::document::{
    DocumentBundle, EncodedReloadDocument, OwnedDocumentSet, ParsedReload,
};
use pse_model::generated::identities::FitId;
use pse_relations::columnar::RelationRow;

/// Private ingress provenance. It is deliberately outside scientific V4 framing.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LoaderOrigins {
    version: u32,
    packages: Vec<PackageOrigin>,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PackageOrigin {
    package: SemanticId,
    documents: Vec<DocumentOrigin>,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct DocumentOrigin {
    document: SemanticId,
    path: String,
    hash: pse_ids::ContentHash,
    member: String,
    declarations: Vec<(DeclarationId, String)>,
    fits: Vec<(FitId, String)>,
}
impl LoaderOrigins {
    pub(super) fn from_bundles(
        bundles: &[DocumentBundle],
        encoded: &[EncodedReloadDocument],
        edits: &[ObjectEdit],
    ) -> Result<Self, WorkflowError> {
        let versions = edits
            .iter()
            .filter_map(|edit| {
                edit.version
                    .as_ref()
                    .map(|version| (edit.logical.clone(), version.key.clone()))
            })
            .collect();
        Self::from_versions(bundles, encoded, &versions)
    }
    fn from_versions(
        bundles: &[DocumentBundle],
        encoded: &[EncodedReloadDocument],
        versions: &BTreeMap<String, String>,
    ) -> Result<Self, WorkflowError> {
        let version = |logical: String| {
            versions
                .get(&logical)
                .cloned()
                .ok_or_else(|| contract(format!("loader origin member absent: {logical}")))
        };
        let mut packages = Vec::with_capacity(bundles.len());
        for bundle in bundles {
            let mut documents = Vec::new();
            for document in &bundle.documents {
                let declarations = document.modeling_rows().map_or(Ok(Vec::new()), |rows| {
                    rows.iter()
                        .map(|row| Ok((row.declaration_id, version(logical(row.declaration_id))?)))
                        .collect::<Result<_, WorkflowError>>()
                })?;
                let fits = document
                    .batches
                    .get(&pse_relations::generated::authored::fit_cases::RELATION_ID)
                    .map(pse_relations::generated::authored::fit_cases::Row::rows)
                    .transpose()
                    .map_err(super::super::super::relation)?
                    .unwrap_or_default()
                    .into_iter()
                    .map(|row| Ok((row.fit_id, version(format!("fit:{}", row.fit_id))?)))
                    .collect::<Result<_, WorkflowError>>()?;
                documents.push(DocumentOrigin {
                    document: document.id,
                    path: document.path.clone(),
                    hash: document.content_hash,
                    member: version(format!("document:{}", document.id))?,
                    declarations,
                    fits,
                });
            }
            for document in encoded
                .iter()
                .filter(|document| document.package() == bundle.package.package_id.as_id())
            {
                documents.push(DocumentOrigin {
                    document: document.id(),
                    path: document.path().into(),
                    hash: document.content_hash(),
                    member: version(format!("document:{}", document.id()))?,
                    declarations: Vec::new(),
                    fits: Vec::new(),
                });
            }
            documents.sort_by(|left, right| left.path.cmp(&right.path));
            if documents
                .windows(2)
                .any(|pair| pair[0].document == pair[1].document || pair[0].path == pair[1].path)
            {
                return Err(contract("loader origin repeats a document"));
            }
            packages.push(PackageOrigin {
                package: bundle.package.package_id.as_id(),
                documents,
            });
        }
        Ok(Self {
            version: 1,
            packages,
        })
    }
    fn documents(&self) -> impl Iterator<Item = &DocumentOrigin> {
        self.packages.iter().flat_map(|package| &package.documents)
    }
    pub(super) fn invalidate_fits(&mut self, changed: &BTreeSet<String>) {
        for package in &mut self.packages {
            for document in &mut package.documents {
                document
                    .fits
                    .retain(|(id, _)| !changed.contains(&format!("fit:{id}")));
            }
        }
    }
}

#[derive(Debug)]
enum ParsedOwner {
    Initial(OwnedDocumentSet),
    Reload(Arc<ParsedReload>),
}
/// Service-local, charged, evictable parser accelerator; canonical provenance is authority.
#[derive(Debug)]
pub(crate) struct RetainedIngress {
    revision: String,
    source: ParsedOwner,
    bytes: usize,
    _lease: Arc<pse_columnar::AllocationLease>,
}
impl RetainedIngress {
    pub(crate) fn retained_bytes(&self) -> usize {
        self.bytes
    }
    fn bundles(&self) -> &[DocumentBundle] {
        match &self.source {
            ParsedOwner::Initial(source) => source.bundles(),
            ParsedOwner::Reload(source) => source.bundles(),
        }
    }
}
fn owners(package: &ModelingPackage) -> (usize, usize) {
    (
        Arc::as_ptr(&package.runtime.validation_context()) as usize,
        Arc::as_ptr(package.runtime.registry()) as usize,
    )
}
pub(in crate::workflow::modeling) fn retain_initial(
    package: &ModelingPackage,
    documents: &OwnedDocumentSet,
) -> Result<(), WorkflowError> {
    let bytes = pse_admission::authoring_driver::document::workspace_extent(documents)?;
    retain(package, ParsedOwner::Initial(documents.clone()), bytes)
}
fn retain(
    package: &ModelingPackage,
    source: ParsedOwner,
    bytes: usize,
) -> Result<(), WorkflowError> {
    let math = package.runtime.shared.math();
    let generation = math.modeling_cache.generation();
    let extent=package.revision_id().len().checked_mul(2).and_then(|n|n.checked_add(size_of::<RetainedIngress>()+1024)).ok_or_else(||contract("retained ingress handle extent"))?;
    // Source bodies have their independent loader leases. Charge only the new handle.
    let lease = math.reserve("modeling:retained-ingress", extent)?;
    let value = Arc::new(RetainedIngress {
        revision: package.revision_id().into(),
        source,
        bytes: bytes.saturating_add(extent),
        _lease: lease,
    });
    let (validation, registry) = owners(package);
    math.modeling_cache.retain_ingress(
        generation,
        package.revision_id().into(),
        validation,
        registry,
        package.physical.key,
        value,
    );
    Ok(())
}

/// Trusted loader extras, unavailable to public IR edit intent.
pub(super) struct ParsedEdit {
    pub(super) order: Vec<DeclarationId>,
    pub(super) fields: BTreeMap<DeclarationId, FieldSpans>,
    pub(super) documents: BTreeMap<SemanticId, pse_model::generated::authored::documents::Row>,
    pub(super) data: BTreeMap<SemanticId, pse_ids::ContentHash>,
    pub(super) physical: PhysicalScope,
    pub(super) packages: Vec<ObjectEdit>,
    pub(super) parsed: Arc<ParsedReload>,
    _lease: Arc<pse_columnar::AllocationLease>,
}
impl ParsedEdit {
    pub(super) fn origins(
        &self,
        members: &BTreeMap<String, SourceMember>,
        edits: &BTreeMap<String, ObjectEdit>,
    ) -> Result<LoaderOrigins, WorkflowError> {
        let mut versions = members
            .iter()
            .map(|(logical, member)| (logical.clone(), member.version.clone()))
            .collect::<BTreeMap<_, _>>();
        for edit in edits.values() {
            match &edit.version {
                Some(version) => {
                    versions.insert(edit.logical.clone(), version.key.clone());
                }
                None => {
                    versions.remove(&edit.logical);
                }
            }
        }
        LoaderOrigins::from_versions(self.parsed.bundles(), self.parsed.encoded_data(), &versions)
    }
}

fn source_rows(
    parsed: &ParsedReload,
    package: &ModelingPackage,
    cancel: &crate::CancelSource,
) -> Result<super::super::DocumentCompilerContext, WorkflowError> {
    super::super::document_inputs_bundles(
        parsed.bundles(),
        package.runtime.registry(),
        &package.physical,
        package.runtime.shared.budget().math.workspace_bytes,
        &package.runtime.shared.pool(),
        &cancel.token(),
    )
}
async fn cold_owner(
    package: &ModelingPackage,
    parent: &delta::Parent,
    cancel: &crate::CancelSource,
) -> Result<Arc<ParsedReload>, WorkflowError> {
    let extent = parent
        .documents
        .values()
        .try_fold(0usize, |n, row| n.checked_add(row.owned_bytes()))
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| contract("cold parser source extent"))?;
    let _scratch = package
        .runtime
        .shared
        .math()
        .reserve("modeling:cold-parser-source", extent)?;
    let mut grouped = BTreeMap::<SemanticId, BTreeMap<String, Vec<u8>>>::new();
    let mut data_leases = Vec::new();
    for document in parent.documents.values() {
        checkpoint(cancel)?;
        let bytes = if let Some(text) = &document.source_text {
            text.as_bytes().to_vec()
        } else if let Some(hash) = parent.frame.data.get(&document.document_id) {
            let logical = format!("data:{}", document.document_id);
            let value=package.local_object(&logical)?;
            if pse_ids::encoding_checksum(value.payload.as_slice()).content_hash() != *hash {
                return Err(contract("cold loader data bytes differ from ingress frame"));
            }
            data_leases.push(
                package.runtime.shared.math().reserve(
                    "modeling:cold-encoded-copy",
                    value
                        .payload
                        .len()
                        .checked_mul(2)
                        .ok_or_else(|| contract("cold encoded extent"))?,
                )?,
            );
            let bytes = value.payload.as_slice().to_vec();
            data_leases.push(value.lease.clone());
            bytes
        } else {
            return Err(contract(
                "parsed reload cannot recover a document with no exact loader source",
            ));
        };
        grouped
            .entry(document.package_id.as_id())
            .or_default()
            .insert(document.path.clone(), bytes);
    }
    let mut sources = Vec::new();
    if let Some(origins) = &parent.frame.origins {
        if origins.version != 1 {
            return Err(contract("unsupported loader origin interpretation"));
        }
        for package in &origins.packages {
            if let Some(source) = grouped.remove(&package.package) {
                sources.push(source);
            }
        }
    } else {
        // Older frames did not retain package order. Declaration order identifies
        // every nonempty package; empty packages contribute no scientific rows.
        for row in &parent.rows {
            let document = parent
                .documents
                .get(&row.document_id)
                .ok_or_else(|| contract("cold declaration document absent"))?;
            if let Some(source) = grouped.remove(&document.package_id.as_id()) {
                sources.push(source);
            }
        }
    }
    sources.extend(grouped.into_values());
    let parsed = Arc::new(ParsedReload::load(
        &sources,
        &[],
        package.runtime.registry(),
        Default::default(),
        &package.runtime.shared.pool(),
        &cancel.token(),
        &package.runtime.validation_context(),
    )?);
    let (rows, _, fits, _, inventory, _lease) = source_rows(&parsed, package, cancel)?;
    let expected = rows
        .iter()
        .map(|row| (row.declaration_id, row))
        .collect::<BTreeMap<_, _>>();
    let actual = parent
        .rows
        .iter()
        .map(|row| (row.declaration_id, row))
        .collect::<BTreeMap<_, _>>();
    if expected != actual || inventory.field_spans != parent.fields {
        return Err(contract(
            "cold loader texts do not exactly own parent declarations and field ranges",
        ));
    }
    // Fit IDs alone confer no deletion authority. Require the exact immutable
    // parser-produced member, then retain unrelated IR fits outside that origin.
    for fit in &fits.fits {
        let logical = format!("fit:{}", fit.fit_id);
        let expected = edit(
            logical.clone(),
            "fits".into(),
            fit.fit_id.to_string(),
            "fit".into(),
            payload(fit)?,
            Vec::new(),
        );
        if parent.members.get(&logical).is_none_or(|member| {
            expected
                .version
                .as_ref()
                .is_none_or(|version| member.version != version.key)
        }) && parent.frame.origins.is_none()
        {
            return Err(contract(
                "older ingress frame has ambiguous fit document ownership",
            ));
        }
    }
    Ok(parsed)
}

fn make_edit(
    package: &ModelingPackage,
    parent: &delta::Parent,
    parsed: Arc<ParsedReload>,
    origins: &LoaderOrigins,
    cancel: &crate::CancelSource,
) -> Result<(crate::workflow::SourceDelta, ParsedEdit), WorkflowError> {
    let extent = parsed
        .retained_bytes()
        .checked_add(parent.rows.owned_bytes())
        .and_then(|n| n.checked_mul(8))
        .and_then(|n| n.checked_add(8192))
        .ok_or_else(|| contract("parsed reload edit extent"))?;
    let edit_lease = package
        .runtime
        .shared
        .math()
        .reserve("modeling:parsed-reload-edit", extent)?;
    let (rows, mut physical, fits, sources, inventory, _lease) =
        source_rows(&parsed, package, cancel)?;
    let mut documents = BTreeMap::new();
    if let Some(batch) = sources.get(&pse_relations::generated::authored::documents::RELATION_ID) {
        for row in pse_relations::generated::authored::documents::Row::rows(batch)
            .map_err(super::super::super::relation)?
        {
            documents.insert(row.document_id, row);
        }
    }
    let data = parsed
        .encoded_data()
        .iter()
        .map(|document| (document.id(), document.content_hash()))
        .collect::<BTreeMap<_, _>>();
    // The captured data intents have no decoded Document wrapper, but retain the
    // same package dependency visibility as independent fresh ingress.
    if let Some(visible) = &mut physical.documents {
        for bundle in parsed.bundles() {
            if bundle
                .documents
                .iter()
                .any(|document| visible.contains(&document.id))
            {
                visible.extend(
                    parsed
                        .encoded_data()
                        .iter()
                        .filter(|document| document.package() == bundle.package.package_id.as_id())
                        .map(EncodedReloadDocument::id),
                );
            }
        }
    }
    let mut delta = crate::workflow::SourceDelta {
        parent: package.revision_id().into(),
        ..Default::default()
    };
    let old_rows = parent
        .rows
        .iter()
        .map(|row| (row.declaration_id, row))
        .collect::<BTreeMap<_, _>>();
    let new_ids = rows
        .iter()
        .map(|row| row.declaration_id)
        .collect::<BTreeSet<_>>();
    for row in &rows {
        match old_rows.get(&row.declaration_id) {
            None => delta.additions.push(row.clone()),
            Some(old) if *old != row => delta.replacements.push(row.clone()),
            Some(_) => {}
        }
    }
    delta.removals.extend(
        parent
            .rows
            .iter()
            .filter(|row| !new_ids.contains(&row.declaration_id))
            .map(|row| row.declaration_id),
    );
    delta.documents.removals.extend(
        parent
            .documents
            .keys()
            .filter(|id| !documents.contains_key(id))
            .copied(),
    );
    for (id, row) in &documents {
        let changed_data = data.get(id) != parent.frame.data.get(id);
        if parent.documents.get(id) == Some(row) && !changed_data {
            continue;
        }
        let mut intent = row.clone();
        if changed_data
            && let Some(encoded) = parsed
                .encoded_data()
                .iter()
                .find(|document| document.id() == *id)
        {
            intent.content = Some(encoded.bytes().to_vec().into());
        }
        if parent.documents.contains_key(id) {
            delta.documents.replacements.push(intent);
        } else {
            delta.documents.additions.push(intent);
        }
    }
    let fit_ids = fits
        .fits
        .iter()
        .map(|row| row.fit_id)
        .collect::<BTreeSet<_>>();
    for row in fits.fits {
        if parent.members.contains_key(&format!("fit:{}", row.fit_id)) {
            delta.fits.replacements.push(row);
        } else {
            delta.fits.additions.push(row);
        }
    }
    for document in origins.documents() {
        for (id, version) in &document.fits {
            if !fit_ids.contains(id)
                && parent
                    .members
                    .get(&format!("fit:{id}"))
                    .is_some_and(|member| &member.version == version)
            {
                delta.fits.removals.push(*id);
            }
        }
    }
    let mut packages = Vec::new();
    if let Some(batch) = sources.get(&pse_relations::generated::authored::packages::RELATION_ID) {
        for row in pse_relations::generated::authored::packages::Row::rows(batch)
            .map_err(super::super::super::relation)?
        {
            packages.push(edit(
                format!("package:{}", row.package_id),
                "packages".into(),
                row.package_id.to_string(),
                "package".into(),
                payload(&row)?,
                Vec::new(),
            ));
        }
    }
    Ok((
        delta,
        ParsedEdit {
            order: rows.iter().map(|row| row.declaration_id).collect(),
            fields: inventory.field_spans.clone(),
            documents,
            data,
            physical,
            packages,
            parsed,
            _lease: edit_lease,
        },
    ))
}
impl ModelingPackage {
    /// Reload complete ordered package document inventories against this exact
    /// parent. Unchanged parser products and data admissions retain their owners;
    /// changed texts receive fresh complete reference resolution before one CAS.
    pub async fn reload_from_parent(
        &self,
        sources: &[BTreeMap<String, Vec<u8>>],
        cancel: &crate::CancelSource,
    ) -> Result<Self, WorkflowError> {
        let activity=self.runtime.admit_activity()?;
        activity.scope(async {
            let deadline=super::super::preparation_deadline(cancel)?;let scoped=cancel.with_deadline(Some(deadline));checkpoint(&scoped)?;
            let (package,parsed)=self.reload_protected(sources,&scoped).await?;
            let _=retain(&package,ParsedOwner::Reload(parsed.clone()),parsed.retained_bytes());
            Ok(package)
        }).await
    }
    async fn reload_protected(
        &self,
        sources: &[BTreeMap<String, Vec<u8>>],
            cancel: &crate::CancelSource,
    ) -> Result<(Self, Arc<ParsedReload>), WorkflowError> {
        let parent = self.delta_parent(cancel).await?;
        let (validation, registry) = owners(self);
        let warm = self
            .runtime
            .shared
            .math()
            .modeling_cache
            .ingress(
                self.revision_id(),
                validation,
                registry,
                self.physical.key,
            )
            .filter(|owner| owner.revision == self.revision_id());
        let exact_texts = parent.documents.values().all(|document| {
            document.source_text.is_some() || parent.frame.data.contains_key(&document.document_id)
        });
        let cold = if warm.is_none() && (parent.frame.origins.is_none() || exact_texts) {
            Some(cold_owner(self, &parent, cancel).await?)
        } else {
            None
        };
        let previous = warm.as_ref().map_or_else(
            || cold.as_ref().map_or(&[][..], |parsed| parsed.bundles()),
            |owner| owner.bundles(),
        );
        let origins = if let Some(origins) = &parent.frame.origins {
            if origins.version != 1 {
                return Err(contract("unsupported loader origin interpretation"));
            }
            origins.clone()
        } else {
            let versions = parent
                .members
                .iter()
                .map(|(logical, member)| (logical.clone(), member.version.clone()))
                .collect();
            let cold = cold.as_ref().ok_or_else(|| {
                contract("older ingress requires exact cold parser reconstruction")
            })?;
            LoaderOrigins::from_versions(cold.bundles(), cold.encoded_data(), &versions)?
        };
        let parsed = Arc::new(ParsedReload::load(
            sources,
            previous,
            self.runtime.registry(),
            Default::default(),
            &self.runtime.shared.pool(),
            &cancel.token(),
            &self.runtime.validation_context(),
        )?);
        let (delta, extras) = make_edit(self, &parent, parsed.clone(), &origins, cancel)?;
        let result = self
            .apply_delta_parent(delta, parent, cancel, Some(extras))
            .await?;
        Ok((result, parsed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "canonical-tests")]
    fn sources(text: &str) -> BTreeMap<String, Vec<u8>> {
        BTreeMap::from([
            (
                "package.toml".into(),
                include_bytes!(
                    "../../../../../../tests/fixtures/packages/minimal_named/package.toml"
                )
                .to_vec(),
            ),
            ("models/bank.pse".into(), text.as_bytes().to_vec()),
        ])
    }
    #[cfg(feature = "canonical-tests")]
    async fn parent(package: &ModelingPackage) -> delta::Parent {
        let protection = package
            .runtime
            .canonical_store()?
            .protect(
                package.revision.canonical.clone(),
                std::time::Duration::from_secs(30),
            )
            .await
            .unwrap();
        let mut read = SelectedRead::new(protection);
        let result = package
            .delta_parent(&mut read, &crate::CancelSource::new())
            .await
            .unwrap();
        read.finish().await.unwrap();
        result
    }
    #[test]
    fn ir_fit_intent_invalidates_exact_document_origin_without_inferring_other_fits() {
        let document = SemanticId::from_bytes([1; 16]);
        let original = FitId::from(SemanticId::from_bytes([2; 16]));
        let retained = FitId::from(SemanticId::from_bytes([3; 16]));
        let mut origins = LoaderOrigins {
            version: 1,
            packages: vec![PackageOrigin {
                package: document,
                documents: vec![DocumentOrigin {
                    document,
                    path: "cases/source.yaml".into(),
                    hash: pse_ids::encoding_checksum(b"source").content_hash(),
                    member: "document-version".into(),
                    declarations: Vec::new(),
                    fits: vec![(original, "old".into()), (retained, "retained".into())],
                }],
            }],
        };
        origins.invalidate_fits(&BTreeSet::from([format!("fit:{original}")]));
        assert_eq!(
            origins.documents().next().unwrap().fits,
            [(retained, "retained".into())]
        );
    }
    #[cfg(feature = "canonical-tests")]
    #[tokio::test]
    async fn canonical_parsed_reload_noop_order_add_remove_and_exact_fresh_v4() {
        let runtime = crate::workflow::tests::runtime();
        let physical = crate::workflow::tests::physical();
        let cancel = crate::CancelSource::new();
        let original = sources("package minimal_named {param a:Scalar=1;}");
        let package = runtime
            .package_from_sources(&[original.clone()], physical.clone(), &cancel)
            .await
            .unwrap();
        let no_op = package
            .reload_from_parent(&[original.clone()], &cancel)
            .await
            .unwrap();
        assert_eq!(no_op.revision.canonical, package.revision.canonical);
        let mut expanded = original.clone();
        expanded.insert(
            "models/a.pse".into(),
            b"package earlier {param b:Scalar=2;}".to_vec(),
        );
        let reloaded = package
            .reload_from_parent(&[expanded.clone()], &cancel)
            .await
            .unwrap();
        let fresh = runtime
            .package_from_sources(&[expanded], physical, &cancel)
            .await
            .unwrap();
        let actual = parent(&reloaded).await;
        let expected = parent(&fresh).await;
        assert_eq!(reloaded.revision.identity, fresh.revision.identity);
        assert_eq!(
            actual.rows, expected.rows,
            "private trusted row order equals independent fresh ingress"
        );
        assert_eq!(actual.fields, expected.fields);
        assert_eq!(actual.documents, expected.documents);
        assert_eq!(
            reloaded.revision.canonical.problem,
            package.revision.canonical.problem
        );
        for (logical, member) in &parent(&package).await.members {
            if logical.starts_with("modeling:")
                && logical != CONTEXT_LOGICAL
                && logical != PHYSICAL_SCOPE_LOGICAL
            {
                assert_eq!(actual.members[logical].version, member.version);
            }
        }
        let removed = reloaded
            .reload_from_parent(&[original], &cancel)
            .await
            .unwrap();
        assert_eq!(removed.revision.identity, package.revision.identity);
        assert_eq!(parent(&removed).await.rows, parent(&package).await.rows);
        assert!(
            parent(&package)
                .await
                .documents
                .values()
                .all(|row| row.path != "models/a.pse")
        );
    }
    #[cfg(feature = "canonical-tests")]
    #[tokio::test]
    async fn canonical_parsed_reload_rebinds_stable_name_and_preserves_failed_parent() {
        let runtime = crate::workflow::tests::runtime();
        let cancel = crate::CancelSource::new();
        let original = sources(
            "package minimal_named {param a:Scalar=1; param b:Scalar=2; def Root {var x:Scalar; eq e:x==a;}}",
        );
        let package = runtime
            .package_from_sources(
                &[original.clone()],
                crate::workflow::tests::physical(),
                &cancel,
            )
            .await
            .unwrap();
        let root = parent(&package)
            .await
            .rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let changed = sources(
            "package minimal_named {param a:Scalar=1; param b:Scalar=2; def Root {var x:Scalar; eq e:x==b;}}",
        );
        let result = package
            .reload_from_parent(&[changed], &cancel)
            .await
            .unwrap();
        let selected = result.selected_source(root, &cancel).await.unwrap();
        assert!(selected.rows.iter().any(|row| row.name == "b"));
        assert!(!selected.rows.iter().any(|row| row.name == "a"));
        selected.read.finish().await.unwrap();
        // Reference admission is selected, as on independent initial ingress.
        let unresolved =
            sources("package minimal_named {def Root {var x:Scalar; eq e:x==missing;}}");
        let unresolved = result
            .reload_from_parent(&[unresolved], &cancel)
            .await
            .unwrap();
        assert!(
            unresolved
                .checked_selection(&[root], &cancel)
                .await
                .is_err()
        );
        // Malformed syntax must still refuse the reload atomically.
        let invalid = sources("package minimal_named {def Root {var x:Scalar; eq e:;}}");
        let store = runtime.canonical_store()?;
        let before = pse_operations::testing::source_problem_head(
            store,
            &unresolved.revision.canonical.problem,
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(before.head, unresolved.revision.canonical.key);
        let error = unresolved
            .reload_from_parent(&[invalid], &cancel)
            .await
            .unwrap_err();
        assert!(
            matches!(
                error,
                WorkflowError::Authoring(pse_admission::authoring_driver::DriverError::Authoring(
                    pse_authoring::AuthoringError::Syntax { .. }
                ))
            ),
            "malformed reload refuses at the parser rather than parent CAS: {error}"
        );
        let after = pse_operations::testing::source_problem_head(
            store,
            &unresolved.revision.canonical.problem,
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(after.head, before.head);
        assert_eq!(after.sequence, before.sequence);
        assert_eq!(
            parent(&result)
                .await
                .rows
                .iter()
                .find(|row| row.name == "e")
                .unwrap()
                .value
                .equation
                .as_ref()
                .unwrap()
                .expression,
            "x==b"
        );
        assert_eq!(
            parent(&package)
                .await
                .rows
                .iter()
                .find(|row| row.name == "e")
                .unwrap()
                .value
                .equation
                .as_ref()
                .unwrap()
                .expression,
            "x==a"
        );
        let cancelled = crate::CancelSource::new();
        cancelled.cancel();
        assert!(
            result
                .reload_from_parent(&[original], &cancelled)
                .await
                .is_err()
        );
    }
    #[cfg(feature = "canonical-tests")]
    #[tokio::test]
    async fn canonical_parsed_reload_cold_older_frame_recovers_exact_owner_and_ir_fit_survives() {
        let runtime = crate::workflow::tests::runtime();
        let cancel = crate::CancelSource::new();
        let original = sources("package minimal_named {param a:Scalar=1;}");
        let package = runtime
            .package_from_sources(
                &[original.clone()],
                crate::workflow::tests::physical(),
                &cancel,
            )
            .await
            .unwrap();
        let mut frame = parent(&package).await.frame;
        frame.origins = None;
        let operation = pse_operations::mint_id::<SemanticId>().to_string();
        let canonical = runtime
            .canonical_store()?
            .edit(
                &package.revision.canonical.problem,
                Some(&package.revision.canonical.key),
                &operation,
                &[edit(
                    INGRESS_FRAME.into(),
                    "context".into(),
                    "ingress_frame".into(),
                    "ingress_frame".into(),
                    payload(&frame).unwrap(),
                    Vec::new(),
                )],
            )
            .await
            .unwrap();
        let older = ModelingPackage {
            revision: SourceRevision {
                canonical,
                identity: package.revision.identity,
            },
            ..package.clone()
        };
        runtime.clear_program_cache();
        let loaded = older
            .reload_from_parent(&[original.clone()], &cancel)
            .await
            .unwrap();
        assert_eq!(loaded.revision.identity, older.revision.identity);
        assert!(parent(&loaded).await.frame.origins.is_some());
        let id = FitId::from(SemanticId::from_bytes([77; 16]));
        let with_fit = loaded
            .with_delta(
                crate::workflow::SourceDelta {
                    parent: loaded.revision.canonical.key.clone(),
                    fits: crate::workflow::SourceFitDelta {
                        additions: vec![pse_model::generated::authored::fit_cases::Row {
                            fit_id: id,
                            parameters: Vec::new(),
                            experiments: Vec::new(),
                            observations: Vec::new(),
                        }],
                        ..Default::default()
                    },
                    ..Default::default()
                },
                &cancel,
            )
            .await
            .unwrap();
        let before = parent(&with_fit).await.members[&format!("fit:{id}")]
            .version
            .clone();
        let removed = with_fit.reload_from_parent(&[], &cancel).await.unwrap();
        assert!(parent(&removed).await.rows.is_empty());
        assert_eq!(
            parent(&removed).await.members[&format!("fit:{id}")].version,
            before,
            "full parsed inventory is no authority to delete unrelated IR fits"
        );
    }
    #[cfg(feature = "canonical-tests")]
    #[tokio::test]
    async fn canonical_parsed_reload_unrelated_document_keeps_data_without_compiler_readmission() {
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
        let mut inventory = sources(
            "package minimal_named {entity kind source provenance {} entity source s {} enum role {given} entity kind node {key n:Integer; attribute value:Integer;} dataset bank:node provenance(s,role.given) from \"data/bank.parquet\";}",
        );
        inventory.insert("data/bank.parquet".into(), bytes);
        let cancel = crate::CancelSource::new();
        let package = crate::workflow::tests::runtime()
            .package_from_sources(
                &[inventory.clone()],
                crate::workflow::tests::physical(),
                &cancel,
            )
            .await
            .unwrap();
        let before = parent(&package).await;
        let small = crate::workflow::tests::runtime_on(
            256 << 20,
            crate::math::MathPolicy {
                workspace_bytes: 1,
                ..Default::default()
            },
        );
        let receiver = Runtime::from_shared(
            small.shared.clone(),
            small.model.clone(),
            package.runtime.canonical.clone(),
        )
        .unwrap();
        let confined = ModelingPackage {
            runtime: receiver,
            ..package.clone()
        };
        assert!(
            confined.all_revision().await.is_err(),
            "the independent full-data compiler control must refuse"
        );
        inventory.insert(
            "models/unrelated.pse".into(),
            b"package unrelated {param a:Scalar=1;}".to_vec(),
        );
        let changed = confined
            .reload_from_parent(&[inventory], &cancel)
            .await
            .unwrap();
        let after = parent(&changed).await;
        assert_eq!(before.frame.data, after.frame.data);
        for (logical, member) in before
            .members
            .iter()
            .filter(|(logical, _)| logical.starts_with("data:") || logical.starts_with("record:"))
        {
            assert_eq!(after.members[logical].version, member.version);
        }
    }
    #[cfg(feature = "canonical-tests")]
    #[tokio::test]
    async fn canonical_parsed_reload_removed_document_fit_is_exact_and_ir_replacement_survives() {
        let runtime = crate::workflow::tests::runtime();
        let cancel = crate::CancelSource::new();
        let mut inventory = sources("package minimal_named {param a:Scalar=1;}");
        let fit = "01991d6a13a070008000000000000077";
        inventory.insert("cases/fit.yaml".into(), format!("fit_cases:\n  - fit_id: \"{fit}\"\n    parameters: []\n    experiments: []\n    observations: []\n").into_bytes());
        let original = runtime
            .package_from_sources(
                &[inventory.clone()],
                crate::workflow::tests::physical(),
                &cancel,
            )
            .await
            .unwrap();
        let id: FitId = SemanticId::parse_hex(fit).unwrap().into();
        let old = parent(&original).await;
        assert!(old.members.contains_key(&format!("fit:{id}")));
        let mut removed_inventory = inventory.clone();
        removed_inventory.remove("cases/fit.yaml");
        let removed = original
            .reload_from_parent(&[removed_inventory], &cancel)
            .await
            .unwrap();
        assert!(
            !parent(&removed)
                .await
                .members
                .contains_key(&format!("fit:{id}")),
            "only the exact loader-owned fit is deleted with its document"
        );
        // Restore the loader-owned document on the current head before replacing
        // its fit through IR. Historical packages cannot authorize a later edit.
        let current = removed
            .reload_from_parent(&[inventory.clone()], &cancel)
            .await
            .unwrap();
        let mut ir = current.fit_declaration(id).await.unwrap();
        ir.parameters.push(
            pse_model::generated::authored::fit_cases::AuthoredFitCasesFieldParametersItem {
                symbol_id: SemanticId::from_bytes([55; 16]),
                fixed: false,
                value: 1.0,
                lower: None,
                upper: None,
                scale: 1.0,
            },
        );
        let replaced = current
            .with_delta(
                crate::workflow::SourceDelta {
                    parent: current.revision.canonical.key.clone(),
                    fits: crate::workflow::SourceFitDelta {
                        replacements: vec![ir],
                        ..Default::default()
                    },
                    ..Default::default()
                },
                &cancel,
            )
            .await
            .unwrap();
        inventory.remove("cases/fit.yaml");
        let preserved = replaced
            .reload_from_parent(&[inventory], &cancel)
            .await
            .unwrap();
        // Independent initial ingress also advances this problem's head, so it
        // follows the complete parent-relative mutation chain above.
        let owned = runtime
            .package_from_sources(
                &[sources("package minimal_named {param a:Scalar=1;}")],
                crate::workflow::tests::physical(),
                &cancel,
            )
            .await
            .unwrap();
        let actual = parent(&preserved).await;
        assert_eq!(
            actual.members[&format!("fit:{id}")].version,
            parent(&replaced).await.members[&format!("fit:{id}")].version
        );
        assert_eq!(
            preserved.revision.identity, owned.revision.identity,
            "unrelated fit retention does not change V4 scientific framing"
        );
    }
}
