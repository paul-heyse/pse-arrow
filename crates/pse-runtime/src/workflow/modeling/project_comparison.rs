// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Logical authored-object comparison and explicit three-way publication decisions.
//! No solver, expression canonicalizer or mutable project name participates.
use super::{ModelingPackage, PhysicalContext, Runtime, WorkflowError, contract};
use crate::{CancelSource, workflow::SourceDelta};
use pse_authoring::language::Declaration;
use pse_model::generated::{
    authored::{documents, fit_cases, packages},
    enums::ModelingDeclarationKind as Kind,
};
use pse_operations::canonical_projects::{DraftVersion, ProjectPublication};
use std::collections::{BTreeMap, BTreeSet};

/// Exact source identity used by a comparison; local tokens never stand in for durable keys.
/// Compact changes between exact admitted authored inventories.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ProjectComparisonSelection {
    /// Original canonical issuing authority, absent for a local source.
    pub issuing_store: Option<String>,
    /// Exact durable source revision, when recorded.
    pub revision: Option<String>,
    /// Exact process-local parent token of the admitted package.
    pub local_revision: String,
    /// Scientific source identity carried by the admitted package.
    pub source: String,
    /// Exact physical closure identity used for semantic interpretation.
    pub physical: String,
}
/// Conservative impacts from authored meaning, not a claim of numerical equivalence.
pub use pse_model::generated::enums::ProjectChangeImpact;
/// One changed logical object. Values are available only by selected detail request.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectObjectChange {
    /// Stable authored logical identity.
    pub logical: String,
    /// The object is absent from the original selection.
    pub added: bool,
    /// The object is absent from the proposed selection.
    pub removed: bool,
    /// Conservative source and scientific impacts supported by authored evidence.
    pub impacts: Vec<ProjectChangeImpact>,
    /// Changed top-level typed fields; detailed values require explicit selection.
    pub fields: Vec<String>,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
/// Compact exact-source authored change summary; no numerical equivalence is asserted.
pub struct ProjectComparison {
    /// Original exact admitted source selection.
    pub from: ProjectComparisonSelection,
    /// Compared exact admitted source selection.
    pub to: ProjectComparisonSelection,
    /// Whether this operation requested and established matching physical context.
    pub semantic_context_matches: bool,
    /// Only changed logical objects, ordered by stable logical identity.
    pub changes: Vec<ProjectObjectChange>,
}
/// One explicitly selected admitted authored object, retaining its native typed shape.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ProjectSourceObject {
    /// Admitted modeling declaration with its stable identity.
    Declaration(Box<Declaration>),
    /// Admitted document metadata and exact data identity.
    Document(ProjectDocumentSource),
    /// Admitted fitting declaration.
    Fit(fit_cases::Row),
    /// Package ownership metadata; automatic package-manifest merging is unsupported.
    Package(packages::Row),
}
/// Source metadata plus exact data identity. Passive comparison does not copy binary payloads.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectDocumentSource {
    /// Original document metadata and optional text, without binary content.
    pub document: documents::Row,
    /// Exact admitted data object version, absent for a text-only document.
    pub content_digest: Option<String>,
}
/// One selected authored object without implicit full source export.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectSourceDetail {
    /// Requested stable logical identity.
    pub logical: String,
    /// Selected admitted object, or explicit absence at this revision.
    pub object: Option<ProjectSourceObject>,
}
/// Explicit resolution of an actual conflict. Delete is never inferred from missing input.
pub use pse_model::generated::enums::ProjectMergeChoice;
/// An explicit conflict decision for one stable logical object.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectMergeResolution {
    /// The actual conflicting logical identity.
    pub logical: String,
    /// Explicit selected original side or deletion.
    pub choice: ProjectMergeChoice,
}
/// A typed conflict between two changes of one logical object.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectMergeConflict {
    /// Conflicting stable authored logical identity.
    pub logical: String,
    /// Conflict evidence requiring an explicit resolution.
    pub kind: ProjectMergeConflictKind,
}
/// The authored evidence that prevents an automatic object merge.
pub use pse_model::generated::enums::ProjectMergeConflictKind;
/// Whether an affected document keeps an authenticated exact source-text preimage.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectDocumentComposition {
    /// Affected original document identity.
    pub logical: String,
    /// False means composed IR has no exact original text; retained ancestors still own originals.
    pub original_text_retained: bool,
}
/// Prospective target changes and conflicts without any publication effect.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectMergePreview {
    /// Authenticated text-preimage availability after composition, before intent issuance.
    pub documents: Vec<ProjectDocumentComposition>,
    /// Unresolved objects; these prevent issuing a publication intent.
    pub conflicts: Vec<ProjectMergeConflict>,
    /// Changes against the target after applying explicit resolutions.
    pub changes: Vec<ProjectObjectChange>,
}
/// Whether the retained decision composes edits or compensates a prior state.
pub use pse_model::generated::enums::ProjectMergeMode;
/// Original admitted proposal ownership, independent of later branch or draft heads.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProjectMergeProposal {
    /// An exact recorded admitted source revision.
    Revision {
        /// Original immutable canonical revision identity.
        revision: String,
    },
    /// An exact immutable saved raw version, admitted before publication.
    Draft {
        /// Full original version descriptor authenticated by its storage owner.
        version: DraftVersion,
    },
}
impl ProjectMergeProposal {
    fn ancestor(&self) -> &str {
        match self {
            Self::Revision { revision } => revision,
            Self::Draft { version } => &version.base_revision,
        }
    }
}
/// Complete immutable decision retained before issuing a source effect.
/// The canonical request binds its commitment and additional ancestry in the source transaction.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectMergeIntent {
    /// Closed original decision document version.
    pub version: pse_model::document::Version<1>,
    /// Original canonical source CAS and immutable decision commitment.
    pub publication: ProjectPublication,
    /// Three-way composition or compensating undo.
    pub mode: ProjectMergeMode,
    /// Exact admitted common ancestor revision.
    pub base: String,
    /// Exact original source head receiving the result.
    pub target: String,
    /// Original admitted source or immutable saved draft version.
    pub proposal: ProjectMergeProposal,
    /// Exact physical closure required for admission and publication.
    pub physical: String,
    /// Complete explicit original conflict choices.
    pub resolutions: Vec<ProjectMergeResolution>,
}
struct Snapshot {
    order: Vec<String>,
    objects: BTreeMap<String, ProjectSourceObject>,
    // Decoded clones remain accounted to this exact observation.
    _owner: std::sync::Arc<pse_columnar::AllocationLease>,
}
fn operation_deadline(cancel: &CancelSource) -> std::time::Instant {
    let limit = std::time::Instant::now() + pse_operations::canonical::REQUEST_TIMEOUT;
    cancel
        .deadline()
        .map_or(limit, |deadline| deadline.min(limit))
}
fn checkpoint(cancel: &CancelSource) -> Result<(), WorkflowError> {
    cancel
        .token()
        .checkpoint()
        .map_err(pse_engine::EngineError::from)?;
    if cancel
        .deadline()
        .is_some_and(|deadline| std::time::Instant::now() >= deadline)
    {
        return Err(contract("project comparison deadline elapsed"));
    }
    Ok(())
}
impl ModelingPackage {
    pub(in crate::workflow) fn runtime_store_issuer(&self) -> Result<&str, WorkflowError> {
        Ok(self.runtime.canonical_store()?.issuing_store()?)
    }
    fn project_comparison_selection(&self) -> Result<ProjectComparisonSelection, WorkflowError> {
        Ok(ProjectComparisonSelection {
            issuing_store: match &self.runtime.durability() {
                crate::workflow::Durability::Durable(_) => {
                    Some(self.runtime.canonical_store()?.issuing_store()?.into())
                }
                crate::workflow::Durability::Ephemeral => None,
            },
            revision: self.canonical_revision().map(|r| r.key.clone()),
            local_revision: self.revision_id().into(),
            source: self.revision.identity().to_string(),
            physical: self.physical.key.to_string(),
        })
    }
    async fn project_snapshot(&self, cancel: &CancelSource) -> Result<Snapshot, WorkflowError> {
        checkpoint(cancel)?;
        let parent = self.delta_parent(cancel).await?;
        let extent = self
            .revision
            .admitted
            .objects
            .values()
            .filter(|o| o.kind != "data")
            .try_fold(65536usize, |n, o| {
                n.checked_add(o.retained_bytes().checked_mul(3)?)
            })
            .ok_or_else(|| contract("comparison inventory extent"))?;
        let owner = self
            .runtime
            .shared
            .math()
            .reserve("project:comparison-inventory", extent)?;
        let mut objects = BTreeMap::new();
        let mut order = Vec::new();
        for row in parent.rows {
            let key = format!("modeling:{}", row.declaration_id.as_id());
            order.push(key.clone());
            objects.insert(key, ProjectSourceObject::Declaration(Box::new(row)));
        }
        for row in parent.documents.into_values() {
            let content_digest = self
                .revision
                .admitted
                .objects
                .get(&format!("data:{}", row.document_id))
                .map(|data| data.version.clone());
            let key = format!("document:{}", row.document_id);
            order.push(key.clone());
            objects.insert(
                key,
                ProjectSourceObject::Document(ProjectDocumentSource {
                    document: row,
                    content_digest,
                }),
            );
        }
        for object in self
            .revision
            .admitted
            .objects
            .values()
            .filter(|o| o.kind == "package")
        {
            let row: packages::Row =
                serde_json::from_slice(&object.payload).map_err(|e| contract(e.to_string()))?;
            order.push(object.logical.clone());
            objects.insert(object.logical.clone(), ProjectSourceObject::Package(row));
        }
        let fits = self.fit_declarations().await?;
        for row in &fits.fits {
            let key = format!("fit:{}", row.fit_id);
            order.push(key.clone());
            objects.insert(key, ProjectSourceObject::Fit(row.clone()));
        }
        checkpoint(cancel)?;
        Ok(Snapshot {
            order,
            objects,
            _owner: owner,
        })
    }
    /// Compact authored-object comparison. A physical mismatch refuses semantic interpretation.
    pub async fn project_compare(
        &self,
        other: &Self,
        cancel: &CancelSource,
    ) -> Result<ProjectComparison, WorkflowError> {
        self.runtime.scope_activity(pse_operations::canonical::operation_scope(operation_deadline(cancel), async {
            if self.physical.key != other.physical.key { return Err(contract("semantic comparison physical contexts differ; request source-only comparison explicitly")); }
            self.project_compare_mode(other, true, cancel).await
        })).await
    }
    /// Explicit source-only view across contexts, never a scientific compatibility claim.
    pub async fn project_compare_source(
        &self,
        other: &Self,
        cancel: &CancelSource,
    ) -> Result<ProjectComparison, WorkflowError> {
        self.runtime
            .scope_activity(pse_operations::canonical::operation_scope(
                operation_deadline(cancel),
                self.project_compare_mode(other, false, cancel),
            ))
            .await
    }
    async fn project_compare_mode(
        &self,
        other: &Self,
        semantic: bool,
        cancel: &CancelSource,
    ) -> Result<ProjectComparison, WorkflowError> {
        let from = self.project_snapshot(cancel).await?;
        let to = other.project_snapshot(cancel).await?;
        let mut changes = compare(&from.objects, &to.objects)?;
        if !semantic {
            for change in &mut changes {
                change.impacts = vec![ProjectChangeImpact::Source, ProjectChangeImpact::Unknown];
            }
        }
        Ok(ProjectComparison {
            from: self.project_comparison_selection()?,
            to: other.project_comparison_selection()?,
            semantic_context_matches: semantic && self.physical.key == other.physical.key,
            changes,
        })
    }
    /// Fetch only caller-selected typed object detail; unknown identities remain explicit absences.
    pub async fn project_source_detail(
        &self,
        logical: &[String],
        cancel: &CancelSource,
    ) -> Result<Vec<ProjectSourceDetail>, WorkflowError> {
        self.runtime
            .scope_activity(pse_operations::canonical::operation_scope(
                operation_deadline(cancel),
                async {
                    if logical.len() > 256 || logical.iter().any(|k| k.len() > 4096) {
                        return Err(contract("project selected detail request exceeds bounds"));
                    }
                    let snapshot = self.project_snapshot(cancel).await?;
                    Ok(logical
                        .iter()
                        .map(|key| ProjectSourceDetail {
                            logical: key.clone(),
                            object: snapshot.objects.get(key).cloned(),
                        })
                        .collect())
                },
            ))
            .await
    }
}
fn fields(a: &ProjectSourceObject, b: &ProjectSourceObject) -> Result<Vec<String>, WorkflowError> {
    let a = serde_json::to_value(a).map_err(|e| contract(e.to_string()))?;
    let b = serde_json::to_value(b).map_err(|e| contract(e.to_string()))?;
    let (Some(a), Some(b)) = (
        a.get("value").and_then(serde_json::Value::as_object),
        b.get("value").and_then(serde_json::Value::as_object),
    ) else {
        return Ok(vec!["kind".into()]);
    };
    Ok(a.keys()
        .chain(b.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|k| a.get(*k) != b.get(*k))
        .cloned()
        .collect())
}
fn impacts(
    a: Option<&ProjectSourceObject>,
    b: Option<&ProjectSourceObject>,
) -> Vec<ProjectChangeImpact> {
    use ProjectChangeImpact::{Source, Structural, Unknown, Value};
    match (a, b) {
        (Some(ProjectSourceObject::Declaration(a)), Some(ProjectSourceObject::Declaration(b))) => {
            let mut normalized = a.clone();
            normalized.source_start = b.source_start;
            normalized.source_end = b.source_end;
            if normalized == *b {
                return vec![Source];
            }
            // Only numeric literal parameter initializers are known value-only edits.
            // A changed reference, equation, unit/type or domain remains structural.
            if a.value.kind == Kind::Parameter && b.value.kind == Kind::Parameter
                && let (Some(before), Some(after)) =
                    (&mut normalized.value.binding, &b.value.binding)
            {
                    let literal = |s: &Option<String>| {
                        s.as_ref()
                            .is_some_and(|s| s.parse::<f64>().is_ok_and(f64::is_finite))
                    };
                    if literal(&before.expression) && literal(&after.expression) {
                        before.expression = after.expression.clone();
                        if normalized == *b {
                            return vec![Source, Value];
                        }
                    }
            }
            vec![Source, Structural]
        }
        (Some(ProjectSourceObject::Document(a)), Some(ProjectSourceObject::Document(b))) => {
            if a.content_digest == b.content_digest
                && a.document.path == b.document.path
                && a.document.package_id == b.document.package_id
            {
                vec![Source]
            } else {
                vec![Source, Unknown]
            }
        }
        (Some(a @ ProjectSourceObject::Package(_)), Some(b @ ProjectSourceObject::Package(_)))
            if same_merge_object(Some(a), Some(b)) =>
        {
            vec![Source]
        }
        (Some(ProjectSourceObject::Fit(_)), Some(ProjectSourceObject::Fit(_))) => {
            vec![Source, Structural]
        }
        (None, Some(ProjectSourceObject::Declaration(_)))
        | (Some(ProjectSourceObject::Declaration(_)), None)
        | (None, Some(ProjectSourceObject::Fit(_)))
        | (Some(ProjectSourceObject::Fit(_)), None) => vec![Source, Structural],
        _ => vec![Source, Unknown],
    }
}
fn compare(
    a: &BTreeMap<String, ProjectSourceObject>,
    b: &BTreeMap<String, ProjectSourceObject>,
) -> Result<Vec<ProjectObjectChange>, WorkflowError> {
    a.keys()
        .chain(b.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter_map(|key| {
            let (old, new) = (a.get(key), b.get(key));
            if old == new {
                return None;
            }
            Some(Ok(ProjectObjectChange {
                logical: key.clone(),
                added: old.is_none(),
                removed: new.is_none(),
                impacts: impacts(old, new),
                fields: match (old, new) {
                    (Some(a), Some(b)) => match fields(a, b) {
                        Ok(fields) => fields,
                        Err(e) => return Some(Err(e)),
                    },
                    _ => Vec::new(),
                },
            }))
        })
        .collect()
}
fn same_merge_object(a: Option<&ProjectSourceObject>, b: Option<&ProjectSourceObject>) -> bool {
    match (a, b) {
        (Some(ProjectSourceObject::Declaration(a)), Some(ProjectSourceObject::Declaration(b))) => {
            let mut a = a.clone();
            a.source_start = b.source_start;
            a.source_end = b.source_end;
            a == *b
        }
        (Some(ProjectSourceObject::Package(a)), Some(ProjectSourceObject::Package(b))) => {
            let mut a = a.clone();
            a.content_hash = b.content_hash;
            a == *b
        }
        _ => a == b,
    }
}
fn merge_objects(
    base: &Snapshot,
    target: &Snapshot,
    proposal: &Snapshot,
    resolutions: &[ProjectMergeResolution],
) -> Result<
    (
        BTreeMap<String, ProjectSourceObject>,
        Vec<ProjectMergeConflict>,
    ),
    WorkflowError,
> {
    let mut decisions = BTreeMap::new();
    for resolution in resolutions {
        if decisions
            .insert(resolution.logical.clone(), resolution.choice)
            .is_some()
        {
            return Err(contract("merge resolution repeats a logical identity"));
        }
    }
    let keys = base
        .objects
        .keys()
        .chain(target.objects.keys())
        .chain(proposal.objects.keys())
        .collect::<BTreeSet<_>>();
    let mut merged = BTreeMap::new();
    let mut conflicts = Vec::new();
    for key in keys {
        let (b, t, p) = (
            base.objects.get(key),
            target.objects.get(key),
            proposal.objects.get(key),
        );
        let conflict =
            !same_merge_object(t, b) && !same_merge_object(p, b) && !same_merge_object(t, p);
        let selected = if conflict {
            match decisions.remove(key) {
                Some(ProjectMergeChoice::Target) => t,
                Some(ProjectMergeChoice::Proposal) => p,
                Some(ProjectMergeChoice::Base) => b,
                Some(ProjectMergeChoice::Delete) => None,
                None => {
                    conflicts.push(ProjectMergeConflict {
                        logical: key.clone(),
                        kind: if t.is_none() || p.is_none() {
                            ProjectMergeConflictKind::DeleteEdit
                        } else if b.is_none() {
                            ProjectMergeConflictKind::ConflictingAddition
                        } else {
                            ProjectMergeConflictKind::DifferentEdits
                        },
                    });
                    t
                }
            }
        } else {
            if same_merge_object(t, b) && !same_merge_object(p, b) {
                p
            } else {
                t
            }
        };
        if let Some(value) = selected {
            merged.insert(key.clone(), value.clone());
        }
    }
    if !decisions.is_empty() {
        return Err(contract(
            "resolution does not name an actual merge conflict",
        ));
    }
    Ok((merged, conflicts))
}
async fn delta_to(
    target: &ModelingPackage,
    old: &Snapshot,
    new: &BTreeMap<String, ProjectSourceObject>,
    proposal_order: &[String],
    sources: &[&ModelingPackage],
) -> Result<SourceDelta, WorkflowError> {
    let mut delta = SourceDelta {
        parent: target.revision_id().into(),
        ..Default::default()
    };
    let order = old
        .order
        .iter()
        .chain(proposal_order)
        .chain(new.keys())
        .collect::<Vec<_>>();
    let mut seen = BTreeSet::new();
    for key in order {
        if !seen.insert(key) {
            continue;
        }
        match (old.objects.get(key), new.get(key)) {
            (a, b) if a == b => {}
            (
                Some(a @ ProjectSourceObject::Package(_)),
                Some(b @ ProjectSourceObject::Package(_)),
            ) if same_merge_object(Some(a), Some(b)) => {}
            (None, Some(ProjectSourceObject::Declaration(row))) => {
                delta.additions.push(row.as_ref().clone())
            }
            (Some(_), Some(ProjectSourceObject::Declaration(row))) => {
                delta.replacements.push(row.as_ref().clone())
            }
            (Some(ProjectSourceObject::Declaration(row)), None) => {
                delta.removals.push(row.declaration_id)
            }
            (None, Some(ProjectSourceObject::Document(row))) => {
                delta.documents.additions.push(document_body(row, sources)?)
            }
            (Some(_), Some(ProjectSourceObject::Document(row))) => delta
                .documents
                .replacements
                .push(document_body(row, sources)?),
            (Some(ProjectSourceObject::Document(row)), None) => {
                delta.documents.removals.push(row.document.document_id)
            }
            (None, Some(ProjectSourceObject::Fit(row))) => delta.fits.additions.push(row.clone()),
            (Some(_), Some(ProjectSourceObject::Fit(row))) => {
                delta.fits.replacements.push(row.clone())
            }
            (Some(ProjectSourceObject::Fit(row)), None) => delta.fits.removals.push(row.fit_id),
            _ => return Err(contract("merge logical identity changed object family")),
        }
    }
    Ok(delta)
}
fn document_body(
    source: &ProjectDocumentSource,
    packages: &[&ModelingPackage],
) -> Result<documents::Row, WorkflowError> {
    let mut row = source.document.clone();
    if let Some(version) = &source.content_digest {
        let logical = format!("data:{}", row.document_id);
        let data = packages
            .iter()
            .find_map(|package| {
                package
                    .revision
                    .admitted
                    .objects
                    .get(&logical)
                    .filter(|object| &object.version == version)
            })
            .ok_or_else(|| {
                contract("merged document body unavailable from exact original source")
            })?;
        row.content = Some(data.payload.clone().into());
    }
    Ok(row)
}
impl ProjectMergeIntent {
    fn commitment(&self) -> Result<String, WorkflowError> {
        let bytes = serde_json::to_vec(&(
            1u8,
            &self.mode,
            &self.base,
            &self.target,
            &self.proposal,
            &self.physical,
            &self.resolutions,
        ))
        .map_err(|e| contract(e.to_string()))?;
        Ok(pse_operations::canonical_execution::result_payload_digest(
            &bytes,
        ))
    }
    /// Validate the complete decision against its original canonical request commitment.
    pub fn validate(&self) -> Result<(), WorkflowError> {
        self.publication.validate()?;
        if self.base.is_empty()
            || self.proposal.ancestor().is_empty()
            || self.target.is_empty()
            || self.physical.is_empty()
            || self.publication.expected() != Some(self.target.as_str())
            || self.publication.context_commitment() != Some(self.commitment()?.as_str())
        {
            return Err(pse_operations::canonical::CanonicalError::OperationReused.into());
        }
        Ok(())
    }
}
impl Runtime {
    /// Reuse complete ordinary admission with a local durability owner. This call does
    /// not publish a source or make saved raw text into accepted shared state.
    pub(crate) async fn project_admit_documents_locally(
        &self,
        documents: &pse_admission::authoring_driver::document::OwnedDocumentSet,
        physical: PhysicalContext,
        cancel: &CancelSource,
    ) -> Result<ModelingPackage, WorkflowError> {
        let local = Self::from_shared_local(self.shared.clone(), self.model.clone())?;
        let mut package = self
            .project_boxed(local.modeling_from_documents(documents, physical, cancel))?
            .await?;
        package.runtime = self.clone();
        Ok(package)
    }
    /// Publish a previously locally admitted proposal through the original exact source owner.
    pub(in crate::workflow) async fn project_publish_admitted(
        &self,
        intent: &ProjectPublication,
        admitted: &ModelingPackage,
        cancel: &CancelSource,
    ) -> Result<ModelingPackage, WorkflowError> {
        intent.validate()?;
        let store = self.canonical_store()?;
        let key = intent
            .expected()
            .ok_or_else(|| contract("admitted publication requires an exact parent"))?;
        let revision = store
            .revision(key)
            .await?
            .ok_or_else(|| contract("admitted publication parent unavailable"))?;
        let parent = self
            .project_boxed(self.modeling_revision(
                revision,
                admitted.physical.clone(),
                admitted.providers.as_ref().clone(),
            ))?
            .await?;
        self.project_boxed(super::canonical::publish_admitted_project(
            self, intent, &parent, admitted, cancel,
        ))?
        .await
    }
    async fn project_is_ancestor(
        &self,
        ancestor: &str,
        descendant: &str,
        cancel: &CancelSource,
    ) -> Result<bool, WorkflowError> {
        let store = self.canonical_store()?;
        let mut pending = vec![descendant.to_owned()];
        let mut seen = BTreeSet::new();
        while let Some(key) = pending.pop() {
            checkpoint(cancel)?;
            if key == ancestor {
                return Ok(true);
            }
            if !seen.insert(key.clone()) {
                continue;
            }
            if seen.len() > 8192 {
                return Err(contract(
                    "project ancestry traversal exceeds bounded source history",
                ));
            }
            let revision = store
                .revision(&key)
                .await?
                .ok_or_else(|| contract("project ancestry source unavailable"))?;
            pending.extend(store.project_revision_parents(&revision).await?);
        }
        Ok(false)
    }
    /// Inspect exact logical-object composition and unresolved conflicts without effects.
    pub async fn project_merge_preview(
        &self,
        base: &ModelingPackage,
        target: &ModelingPackage,
        proposal: &ModelingPackage,
        resolutions: &[ProjectMergeResolution],
        cancel: &CancelSource,
    ) -> Result<ProjectMergePreview, WorkflowError> {
        self.scope_activity(pse_operations::canonical::operation_scope(
            operation_deadline(cancel),
            async {
                if base.physical.key != target.physical.key
                    || proposal.physical.key != target.physical.key
                {
                    return Err(contract("merge physical contexts differ"));
                }
                let (b, t, p) = (
                    base.project_snapshot(cancel).await?,
                    target.project_snapshot(cancel).await?,
                    proposal.project_snapshot(cancel).await?,
                );
                let (merged, conflicts) = merge_objects(&b, &t, &p, resolutions)?;
                let delta =
                    delta_to(target, &t, &merged, &p.order, &[target, proposal, base]).await?;
                let (_, _, documents) = target
                    .project_compose_documents(&delta, &[target, proposal, base], cancel)
                    .await?;
                Ok(ProjectMergePreview {
                    conflicts,
                    changes: compare(&t.objects, &merged)?,
                    documents,
                })
            },
        ))
        .await
    }
    /// Retain an exact admitted revision decision before publication.
    pub async fn project_merge_intent(
        &self,
        publication: ProjectPublication,
        base: &ModelingPackage,
        target: &ModelingPackage,
        proposal: &ModelingPackage,
        resolutions: Vec<ProjectMergeResolution>,
        cancel: &CancelSource,
    ) -> Result<ProjectMergeIntent, WorkflowError> {
        let descriptor = ProjectMergeProposal::Revision {
            revision: proposal.recorded_revision()?.key.clone(),
        };
        self.project_merge_decision(
            publication,
            base,
            target,
            proposal,
            descriptor,
            resolutions,
            cancel,
        )
        .await
    }
    /// Retain one immutable saved version, without publishing a temporary source revision.
    pub async fn project_merge_draft_intent(
        &self,
        publication: ProjectPublication,
        base: &ModelingPackage,
        target: &ModelingPackage,
        version: &DraftVersion,
        resolutions: Vec<ProjectMergeResolution>,
        cancel: &CancelSource,
    ) -> Result<ProjectMergeIntent, WorkflowError> {
        self.scope_activity(pse_operations::canonical::operation_scope(
            operation_deadline(cancel),
            async {
                let proposal = self
                    .project_boxed(self.project_admit_draft_version(
                        version,
                        target.physical.clone(),
                        cancel,
                    ))?
                    .await?;
                self.project_merge_decision(
                    publication,
                    base,
                    target,
                    &proposal,
                    ProjectMergeProposal::Draft {
                        version: version.clone(),
                    },
                    resolutions,
                    cancel,
                )
                .await
            },
        ))
        .await
    }
    #[expect(clippy::too_many_arguments, reason = "A merge decision binds three independently admitted parents, selected resolutions, original publication, mode and cancellation")]
    async fn project_merge_decision(
        &self,
        publication: ProjectPublication,
        base: &ModelingPackage,
        target: &ModelingPackage,
        proposal: &ModelingPackage,
        descriptor: ProjectMergeProposal,
        resolutions: Vec<ProjectMergeResolution>,
        cancel: &CancelSource,
    ) -> Result<ProjectMergeIntent, WorkflowError> {
        self.scope_activity(pse_operations::canonical::operation_scope(
            operation_deadline(cancel),
            async {
                for package in [base, target, proposal] {
                    if package.runtime_store_issuer()? != self.canonical_store()?.issuing_store()? {
                        return Err(contract("merge source issuing authority differs"));
                    }
                }
                let (base_key, target_key, proposal_key) = (
                    &base.recorded_revision()?.key,
                    &target.recorded_revision()?.key,
                    descriptor.ancestor(),
                );
                if publication.expected() != Some(target_key.as_str())
                    || !self
                        .project_is_ancestor(base_key, target_key, cancel)
                        .await?
                    || !self
                        .project_is_ancestor(base_key, proposal_key, cancel)
                        .await?
                {
                    return Err(contract(
                        "merge base is not an ancestor of both exact selections",
                    ));
                }
                let preview = self
                    .project_merge_preview(base, target, proposal, &resolutions, cancel)
                    .await?;
                if !preview.conflicts.is_empty() {
                    return Err(contract("merge has unresolved logical-object conflicts"));
                }
                let publication = publication.with_merge_ancestry(base_key, proposal_key)?;
                let mut intent = ProjectMergeIntent {
                    version: Default::default(),
                    publication,
                    mode: ProjectMergeMode::Merge,
                    base: base_key.clone(),
                    target: target_key.clone(),
                    proposal: descriptor,
                    physical: target.physical.key.to_string(),
                    resolutions,
                };
                let commitment = intent.commitment()?;
                intent.publication = intent.publication.with_context_commitment(&commitment)?;
                intent.validate()?;
                Ok(intent)
            },
        ))
        .await
    }
    /// Undo means a new admitted source revision with the chosen earlier authored state.
    pub async fn project_undo_intent(
        &self,
        publication: ProjectPublication,
        target: &ModelingPackage,
        restore: &ModelingPackage,
        cancel: &CancelSource,
    ) -> Result<ProjectMergeIntent, WorkflowError> {
        self.scope_activity(pse_operations::canonical::operation_scope(
            operation_deadline(cancel),
            async {
                for package in [target, restore] {
                    if package.runtime_store_issuer()? != self.canonical_store()?.issuing_store()? {
                        return Err(contract("undo source issuing authority differs"));
                    }
                }
                let target_key = &target.recorded_revision()?.key;
                let restore_key = &restore.recorded_revision()?.key;
                if publication.expected() != Some(target_key.as_str())
                    || target.physical.key != restore.physical.key
                    || !self
                        .project_is_ancestor(restore_key, target_key, cancel)
                        .await?
                {
                    return Err(contract("undo source is not a compatible exact ancestor"));
                }
                let mut intent = ProjectMergeIntent {
                    version: Default::default(),
                    publication: publication.with_merge_ancestry(restore_key, restore_key)?,
                    mode: ProjectMergeMode::Undo,
                    base: restore_key.clone(),
                    target: target_key.clone(),
                    proposal: ProjectMergeProposal::Revision {
                        revision: restore_key.clone(),
                    },
                    physical: target.physical.key.to_string(),
                    resolutions: Vec::new(),
                };
                let commitment = intent.commitment()?;
                intent.publication = intent.publication.with_context_commitment(&commitment)?;
                intent.validate()?;
                Ok(intent)
            },
        ))
        .await
    }
    /// Settle original acknowledgement first; never reinterpret a later head or draft as this intent.
    pub async fn project_publish_merge(
        &self,
        intent: &ProjectMergeIntent,
        physical: PhysicalContext,
        cancel: &CancelSource,
    ) -> Result<ModelingPackage, WorkflowError> {
        self.scope_activity(pse_operations::canonical::operation_scope(operation_deadline(cancel), async {
            intent.validate()?;
            if physical.key.to_string() != intent.physical { return Err(contract("merge receiving physical context differs")); }
            let store = self.canonical_store()?;
            if let Some(revision) = store.settled_project_publication(&intent.publication).await? { return self.modeling_revision(revision,physical,BTreeMap::new()).await; }
            let mut packages = Vec::new();
            for key in [&intent.base,&intent.target] {
                let revision = store.revision(key).await?.ok_or_else(|| contract("merge original source unavailable"))?;
                packages.push(self.modeling_revision(revision,physical.clone(),BTreeMap::new()).await?);
            }
            let proposal = match &intent.proposal {
                ProjectMergeProposal::Revision { revision } => {
                    let selected = store.revision(revision).await?.ok_or_else(|| contract("merge proposal source unavailable"))?;
                    self.modeling_revision(selected,physical.clone(),BTreeMap::new()).await?
                }
                ProjectMergeProposal::Draft { version } => self.project_boxed(self.project_admit_draft_version(version,physical.clone(),cancel))?.await?,
            };
            let (base,target,proposal) = (&packages[0],&packages[1],&proposal);
            if !self.project_is_ancestor(&intent.base,&intent.target,cancel).await? || !self.project_is_ancestor(&intent.base,intent.proposal.ancestor(),cancel).await? { return Err(contract("retained merge ancestry differs")); }
            let (b,t,p) = (base.project_snapshot(cancel).await?,target.project_snapshot(cancel).await?,proposal.project_snapshot(cancel).await?);
            let (merged,conflicts) = match intent.mode { ProjectMergeMode::Merge => merge_objects(&b,&t,&p,&intent.resolutions)?, ProjectMergeMode::Undo => { if !matches!(&intent.proposal,ProjectMergeProposal::Revision { revision } if revision == &intent.base) || !intent.resolutions.is_empty() { return Err(contract("undo decision shape differs")); } (p.objects.clone(),Vec::new()) } };
            if !conflicts.is_empty() { return Err(contract("retained merge has unresolved conflicts")); }
            // Package metadata changes cannot be smuggled through declaration/document merging.
            for source in [base,proposal] {
                let source_packages=source.project_snapshot(cancel).await?.objects.into_iter().filter(|(_,object)|matches!(object,ProjectSourceObject::Package(_))).collect::<BTreeMap<_,_>>();
                let target_packages=t.objects.iter().filter(|(_,object)|matches!(object,ProjectSourceObject::Package(_))).collect::<BTreeMap<_,_>>();
                if source_packages.len()!=target_packages.len() || source_packages.iter().any(|(key,row)|!same_merge_object(Some(row),target_packages.get(key).copied())) { return Err(contract("merge package manifests differ; explicit compatible admission required")); }
            }
            let delta = delta_to(target,&t,&merged,&p.order,&[target,proposal,base]).await?;
            let (delta,composition,_)=target.project_compose_documents(&delta,&[target,proposal,base],cancel).await?;
            let admitted = target.project_admit_composed_delta(delta,composition,cancel).await?;
            super::canonical::publish_admitted_project(self,&intent.publication,target,&admitted,cancel).await
        })).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn declarations(text: &str) -> Vec<Declaration> {
        pse_authoring::language::parse(
            text,
            pse_ids::SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap()
    }
    fn declaration(text: &str, name: &str) -> Declaration {
        declarations(text)
            .into_iter()
            .find(|row| row.name == name)
            .unwrap()
    }
    fn snapshot(runtime: &Runtime, rows: Vec<Declaration>) -> Snapshot {
        let order = rows
            .iter()
            .map(|row| format!("modeling:{}", row.declaration_id.as_id()))
            .collect();
        let objects = rows
            .into_iter()
            .map(|row| {
                (
                    format!("modeling:{}", row.declaration_id.as_id()),
                    ProjectSourceObject::Declaration(Box::new(row)),
                )
            })
            .collect();
        Snapshot {
            order,
            objects,
            _owner: runtime
                .shared
                .math()
                .reserve("project:test-snapshot", 65536)
                .unwrap(),
        }
    }
    #[test]
    fn authored_changes_have_conservative_independent_meaning() {
        let original = declaration(
            "package p { def R { param k:Scalar=1; var x:Scalar; eq balance:x==k; } }",
            "k",
        );
        let changed = declaration(
            "package p { def R { param k:Scalar=2; var x:Scalar; eq balance:x==k; } }",
            "k",
        );
        assert_eq!(
            impacts(
                Some(&ProjectSourceObject::Declaration(Box::new(original.clone()))),
                Some(&ProjectSourceObject::Declaration(Box::new(changed)))
            ),
            vec![ProjectChangeImpact::Source, ProjectChangeImpact::Value]
        );
        let mut presentation = original.clone();
        presentation.source_start += 3;
        presentation.source_end += 3;
        assert_eq!(
            impacts(
                Some(&ProjectSourceObject::Declaration(Box::new(original.clone()))),
                Some(&ProjectSourceObject::Declaration(Box::new(presentation)))
            ),
            vec![ProjectChangeImpact::Source]
        );
        let old_equation = declaration(
            "package p { def R { param k:Scalar=1; var x:Scalar; eq balance:x==k; } }",
            "balance",
        );
        let changed_equation = declaration(
            "package p { def R { param k:Scalar=1; var x:Scalar; eq balance:2*x==k; } }",
            "balance",
        );
        assert_eq!(
            impacts(
                Some(&ProjectSourceObject::Declaration(Box::new(old_equation))),
                Some(&ProjectSourceObject::Declaration(Box::new(changed_equation)))
            ),
            vec![ProjectChangeImpact::Source, ProjectChangeImpact::Structural]
        );
        let doc = |path: &str| {
            ProjectSourceObject::Document(ProjectDocumentSource {
                document: documents::Row {
                    document_id: pse_ids::SemanticId::NIL,
                    package_id: pse_model::generated::identities::PackageId::from(
                        pse_ids::SemanticId::NIL,
                    ),
                    path: path.into(),
                    source_text: Some("# comment".into()),
                    content: None,
                },
                content_digest: None,
            })
        };
        assert_eq!(
            impacts(Some(&doc("old")), Some(&doc("new"))),
            vec![ProjectChangeImpact::Source, ProjectChangeImpact::Unknown]
        );
    }
    #[tokio::test]
    async fn independent_objects_merge_and_same_object_needs_an_explicit_choice() {
        let runtime = crate::workflow::tests::runtime();
        let original = declarations("package p { param a:Scalar=1; param b:Scalar=2; }");
        let mut target_rows = original.clone();
        target_rows
            .iter_mut()
            .find(|r| r.name == "a")
            .unwrap()
            .value
            .binding
            .as_mut()
            .unwrap()
            .expression = Some("3".into());
        let mut proposal_rows = original.clone();
        proposal_rows
            .iter_mut()
            .find(|r| r.name == "b")
            .unwrap()
            .value
            .binding
            .as_mut()
            .unwrap()
            .expression = Some("4".into());
        let (base, target, proposal) = (
            snapshot(&runtime, original.clone()),
            snapshot(&runtime, target_rows.clone()),
            snapshot(&runtime, proposal_rows),
        );
        let (merged, conflicts) = merge_objects(&base, &target, &proposal, &[]).unwrap();
        assert!(conflicts.is_empty());
        let values = merged
            .values()
            .filter_map(|v| match v {
                ProjectSourceObject::Declaration(row) if row.name == "a" || row.name == "b" => {
                    Some((
                        row.name.clone(),
                        row.value
                            .binding
                            .as_ref()
                            .unwrap()
                            .expression
                            .clone()
                            .unwrap(),
                    ))
                }
                _ => None,
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            values,
            BTreeMap::from([("a".into(), "3".into()), ("b".into(), "4".into())])
        );
        let mut other_rows = original;
        other_rows
            .iter_mut()
            .find(|r| r.name == "a")
            .unwrap()
            .value
            .binding
            .as_mut()
            .unwrap()
            .expression = Some("9".into());
        let other = snapshot(&runtime, other_rows);
        let (_, conflicts) = merge_objects(&base, &target, &other, &[]).unwrap();
        assert_eq!(conflicts.len(), 1);
        let logical = conflicts[0].logical.clone();
        let (resolved, conflicts) = merge_objects(
            &base,
            &target,
            &other,
            &[ProjectMergeResolution {
                logical: logical.clone(),
                choice: ProjectMergeChoice::Proposal,
            }],
        )
        .unwrap();
        assert!(conflicts.is_empty());
        assert_eq!(resolved[&logical], other.objects[&logical]);
        assert!(
            merge_objects(
                &base,
                &target,
                &proposal,
                &[ProjectMergeResolution {
                    logical,
                    choice: ProjectMergeChoice::Target
                }]
            )
            .is_err()
        );
    }
    #[tokio::test]
    async fn delete_edit_and_conflicting_additions_do_not_choose_last_writer() {
        let runtime = crate::workflow::tests::runtime();
        let row = declaration("package p { param k:Scalar=1; }", "k");
        let mut changed = row.clone();
        changed.value.binding.as_mut().unwrap().expression = Some("5".into());
        let base = snapshot(&runtime, vec![row.clone()]);
        let deleted = snapshot(&runtime, vec![]);
        let edited = snapshot(&runtime, vec![changed]);
        let (_, conflicts) = merge_objects(&base, &deleted, &edited, &[]).unwrap();
        assert_eq!(conflicts[0].kind, ProjectMergeConflictKind::DeleteEdit);
        let empty = snapshot(&runtime, vec![]);
        let added = snapshot(&runtime, vec![row]);
        let (_, conflicts) = merge_objects(&empty, &added, &edited, &[]).unwrap();
        assert_eq!(
            conflicts[0].kind,
            ProjectMergeConflictKind::ConflictingAddition
        );
    }
    pub(super) fn source_documents(a: &str, b: &str) -> BTreeMap<String, Vec<u8>> {
        BTreeMap::from([
            (
                "package.toml".into(),
                include_bytes!("../../../../../tests/fixtures/packages/minimal_named/package.toml")
                    .to_vec(),
            ),
            ("models/a.pse".into(), a.as_bytes().to_vec()),
            ("models/b.pse".into(), b.as_bytes().to_vec()),
        ])
    }
    async fn document_package(runtime: &Runtime, a: &str, b: &str) -> ModelingPackage {
        runtime
            .package_from_sources(
                &[source_documents(a, b)],
                crate::workflow::tests::physical(),
                &CancelSource::new(),
            )
            .await
            .unwrap()
    }
    #[tokio::test]
    async fn unequal_length_parsed_edits_do_not_create_declaration_offset_conflicts() {
        let runtime = crate::workflow::tests::runtime();
        let base = document_package(
            &runtime,
            "package p {param a:Scalar=1;param b:Scalar=2;}",
            "package q {param untouched:Scalar=0;}",
        )
        .await;
        let target = document_package(
            &runtime,
            "package p {param a:Scalar=100000;param b:Scalar=2;}",
            "package q {param untouched:Scalar=0;}",
        )
        .await;
        let proposal = document_package(
            &runtime,
            "package p {param a:Scalar=1;param b:Scalar=200000;}",
            "package q {param untouched:Scalar=0;}",
        )
        .await;
        let preview = runtime
            .project_merge_preview(&base, &target, &proposal, &[], &CancelSource::new())
            .await
            .unwrap();
        // The single original text document changed on both sides, but its distinct
        // parameter declarations are nonoverlapping despite shifted byte positions.
        assert_eq!(preview.conflicts.len(), 1);
        assert!(preview.conflicts[0].logical.starts_with("document:"));
        let resolution = ProjectMergeResolution {
            logical: preview.conflicts[0].logical.clone(),
            choice: ProjectMergeChoice::Target,
        };
        let preview = runtime
            .project_merge_preview(
                &base,
                &target,
                &proposal,
                std::slice::from_ref(&resolution),
                &CancelSource::new(),
            )
            .await
            .unwrap();
        assert!(preview.conflicts.is_empty());
        assert!(
            preview
                .documents
                .iter()
                .any(|document| document.logical == resolution.logical
                    && !document.original_text_retained)
        );
        let (b, t, p) = (
            base.project_snapshot(&CancelSource::new()).await.unwrap(),
            target.project_snapshot(&CancelSource::new()).await.unwrap(),
            proposal
                .project_snapshot(&CancelSource::new())
                .await
                .unwrap(),
        );
        let (merged, conflicts) = merge_objects(&b, &t, &p, &[resolution]).unwrap();
        assert!(conflicts.is_empty());
        let delta = delta_to(&target, &t, &merged, &p.order, &[&target, &proposal, &base])
            .await
            .unwrap();
        let (delta, proof, _) = target
            .project_compose_documents(&delta, &[&target, &proposal, &base], &CancelSource::new())
            .await
            .unwrap();
        let admitted = target
            .project_admit_composed_delta(delta, proof, &CancelSource::new())
            .await
            .unwrap();
        let rows = admitted.declarations().await.unwrap();
        for (name, value) in [("a", "100000"), ("b", "200000")] {
            assert_eq!(
                rows.iter()
                    .find(|row| row.name == name)
                    .unwrap()
                    .value
                    .binding
                    .as_ref()
                    .unwrap()
                    .expression
                    .as_deref(),
                Some(value)
            );
        }
        let snapshot = admitted
            .project_snapshot(&CancelSource::new())
            .await
            .unwrap();
        assert!(snapshot.objects.values().any(|object|matches!(object,ProjectSourceObject::Document(document) if document.document.path=="models/a.pse"&&document.document.source_text.is_none())));
    }
    #[tokio::test]
    async fn admitted_document_composition_preserves_exact_preimages_and_public_ir_guard() {
        let runtime = crate::workflow::tests::runtime();
        let cancel = CancelSource::new();
        let base = document_package(
            &runtime,
            "package p {param a:Scalar=1;}",
            "package q {param b:Scalar=2;}",
        )
        .await;
        let target = document_package(
            &runtime,
            "package p {param a:Scalar=3;}",
            "package q {param b:Scalar=2;}",
        )
        .await;
        let proposal = document_package(
            &runtime,
            "package p {param a:Scalar=1;}",
            "package q {param b:Scalar=4;}",
        )
        .await;
        let (b, t, p) = (
            base.project_snapshot(&cancel).await.unwrap(),
            target.project_snapshot(&cancel).await.unwrap(),
            proposal.project_snapshot(&cancel).await.unwrap(),
        );
        let (merged, conflicts) = merge_objects(&b, &t, &p, &[]).unwrap();
        assert!(conflicts.is_empty());
        let delta = delta_to(&target, &t, &merged, &p.order, &[&target, &proposal, &base])
            .await
            .unwrap();
        assert!(
            target.project_admit_delta(&delta, &cancel).await.is_err(),
            "public IR cannot assert an unparsed source preimage"
        );
        let (delta, proof, notes) = target
            .project_compose_documents(&delta, &[&target, &proposal, &base], &cancel)
            .await
            .unwrap();
        assert!(notes.iter().all(|note| note.original_text_retained));
        let admitted = target
            .project_admit_composed_delta(delta, proof, &cancel)
            .await
            .unwrap();
        let snapshot = admitted.project_snapshot(&cancel).await.unwrap();
        let expected_hash =
            pse_admission::authoring_driver::document::package_checksum(&source_documents(
                "package p {param a:Scalar=3;}",
                "package q {param b:Scalar=4;}",
            ));
        assert!(snapshot.objects.values().any(|object|matches!(object,ProjectSourceObject::Package(package) if package.content_hash==expected_hash)),"derived bundle checksum uses the complete merged original bytes");
        for (path, text) in [
            ("models/a.pse", "package p {param a:Scalar=3;}"),
            ("models/b.pse", "package q {param b:Scalar=4;}"),
        ] {
            assert!(snapshot.objects.values().any(|object|matches!(object,ProjectSourceObject::Document(document) if document.document.path==path&&document.document.source_text.as_deref()==Some(text))));
        }
        // The original text remains loader-owned: a subsequent ordinary parsed reload
        // can reconstruct it using the transferred admitted provenance.
        let reloaded = admitted
            .reload_from_parent(
                &[source_documents(
                    "package p {param a:Scalar=3;}",
                    "package q {param b:Scalar=5;}",
                )],
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(
            reloaded
                .declarations()
                .await
                .unwrap()
                .iter()
                .find(|row| row.name == "b")
                .unwrap()
                .value
                .binding
                .as_ref()
                .unwrap()
                .expression
                .as_deref(),
            Some("5")
        );
    }

    #[tokio::test]
    async fn local_merge_admission_preserves_target_order_and_refuses_wrong_parent() {
        let runtime = crate::workflow::tests::runtime();
        let rows = declarations("package p { param a:Scalar=1; param b:Scalar=2; }");
        let package = runtime
            .modeling_package(rows.clone(), crate::workflow::tests::physical())
            .await
            .unwrap();
        let mut next = rows.clone();
        next.iter_mut()
            .find(|r| r.name == "a")
            .unwrap()
            .value
            .binding
            .as_mut()
            .unwrap()
            .expression = Some("3".into());
        let target = package
            .project_snapshot(&CancelSource::new())
            .await
            .unwrap();
        let proposal = snapshot(&runtime, next.clone());
        let delta = delta_to(
            &package,
            &target,
            &proposal.objects,
            &proposal.order,
            &[&package],
        )
        .await
        .unwrap();
        let admitted = package
            .project_admit_delta(&delta, &CancelSource::new())
            .await
            .unwrap();
        assert_eq!(&*admitted.declarations().await.unwrap(), next.as_slice());
        assert!(admitted.canonical_revision().is_none());
        assert!(
            package
                .project_admit_delta(
                    &SourceDelta {
                        parent: "unrelated".into(),
                        ..Default::default()
                    },
                    &CancelSource::new()
                )
                .await
                .is_err()
        );
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
mod native_tests {
    use super::*;
    fn rows() -> Vec<Declaration> {
        pse_authoring::language::parse(
            "package p { param a:Scalar=1; param b:Scalar=2; }",
            pse_ids::SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap()
    }
    async fn change(
        runtime: &Runtime,
        package: &ModelingPackage,
        name: &str,
        value: &str,
        operation: &str,
        branch_key: &str,
    ) -> ModelingPackage {
        let mut row = package
            .declarations()
            .await
            .unwrap()
            .iter()
            .find(|r| r.name == name)
            .unwrap()
            .clone();
        row.value.binding.as_mut().unwrap().expression = Some(value.into());
        let local = package
            .project_admit_delta(
                &SourceDelta {
                    parent: package.revision_id().into(),
                    replacements: vec![row],
                    ..Default::default()
                },
                &CancelSource::new(),
            )
            .await
            .unwrap();
        let store = runtime.canonical_store().unwrap();
        let project = store.project("project").await.unwrap().unwrap();
        let branch = store.project_branch(branch_key).await.unwrap().unwrap();
        let intent = store
            .project_publication_intent(
                &project,
                &branch,
                &package.recorded_revision().unwrap().key,
                None,
                operation,
            )
            .unwrap();
        runtime
            .project_publish_admitted(&intent, &local, &CancelSource::new())
            .await
            .unwrap()
    }
    #[tokio::test]
    async fn native_merge_and_undo_refuse_foreign_issuer_despite_exact_revision_key_collision() {
        let runtime = crate::workflow::tests::durable_runtime();
        let foreign = crate::workflow::tests::durable_runtime();
        let store = runtime.canonical_store().unwrap();
        let foreign_store = foreign.canonical_store().unwrap();
        let physical = crate::workflow::tests::physical();
        let cancel = CancelSource::new();
        assert_ne!(
            store.issuing_store().unwrap(),
            foreign_store.issuing_store().unwrap()
        );
        let origin = runtime
            .modeling_package(rows(), physical.clone())
            .await
            .unwrap();
        store
            .create_project(
                "create-issuer",
                "issuer-project",
                "Issuer collision control",
            )
            .await
            .unwrap();
        let project = store.project("issuer-project").await.unwrap().unwrap();
        let revision = runtime
            .project_fork(
                &project,
                "issuer-branch",
                "issuer-problem",
                "shared-exact-revision",
                &origin,
            )
            .await
            .unwrap();
        let local_base = runtime
            .modeling_revision(revision, physical.clone(), BTreeMap::new())
            .await
            .unwrap();
        let project = store.project("issuer-project").await.unwrap().unwrap();
        let branch = store
            .project_branch("issuer-branch")
            .await
            .unwrap()
            .unwrap();

        let mut foreign_rows = rows();
        foreign_rows
            .iter_mut()
            .find(|row| row.name == "a")
            .unwrap()
            .value
            .binding
            .as_mut()
            .unwrap()
            .expression = Some("99".into());
        let foreign_content = foreign
            .modeling_package(foreign_rows, physical.clone())
            .await
            .unwrap();
        let edits = foreign_content
            .revision
            .admitted
            .objects
            .values()
            .map(|object| object.edit())
            .collect::<Vec<_>>();
        // Both databases genuinely accept the same caller-selected immutable key.
        // The scientific contents and issuing authorities are deliberately different.
        let collision = foreign_store
            .edit(
                "foreign-collision",
                None,
                &local_base.recorded_revision().unwrap().key,
                &edits,
            )
            .await
            .unwrap();
        let foreign_base = foreign
            .modeling_revision(collision, physical, BTreeMap::new())
            .await
            .unwrap();
        assert_eq!(
            local_base.recorded_revision().unwrap().key,
            foreign_base.recorded_revision().unwrap().key
        );
        assert_ne!(
            local_base.revision.identity(),
            foreign_base.revision.identity()
        );
        assert_eq!(
            foreign_base
                .declarations()
                .await
                .unwrap()
                .iter()
                .find(|row| row.name == "a")
                .unwrap()
                .value
                .binding
                .as_ref()
                .unwrap()
                .expression
                .as_deref(),
            Some("99")
        );

        let publication = store
            .project_publication_intent(
                &project,
                &branch,
                &local_base.recorded_revision().unwrap().key,
                None,
                "foreign-must-not-issue",
            )
            .unwrap();
        for (base, target, proposal) in [
            (&foreign_base, &local_base, &local_base),
            (&local_base, &foreign_base, &local_base),
            (&local_base, &local_base, &foreign_base),
        ] {
            let error = runtime
                .project_merge_intent(publication.clone(), base, target, proposal, vec![], &cancel)
                .await
                .unwrap_err();
            assert!(error.to_string().contains("issuing authority"), "{error}");
        }
        let error = runtime
            .project_undo_intent(publication.clone(), &local_base, &foreign_base, &cancel)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("issuing authority"), "{error}");
        // A local no-op decision still issues: neither ancestry nor source content
        // mismatch accidentally substitutes for the explicit issuer requirement.
        runtime
            .project_merge_intent(
                publication,
                &local_base,
                &local_base,
                &local_base,
                vec![],
                &cancel,
            )
            .await
            .unwrap()
            .validate()
            .unwrap();
        assert!(
            store
                .project_acknowledgement("foreign-must-not-issue")
                .await
                .unwrap()
                .is_none()
        );
        assert_eq!(
            store
                .project_branch_revision("issuer-branch")
                .await
                .unwrap()
                .unwrap(),
            *local_base.recorded_revision().unwrap()
        );
        foreign_store.remove_isolated_fixture().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn native_document_saved_proposal_merge_and_exact_text_undo() {
        let runtime = crate::workflow::tests::durable_runtime();
        let store = runtime.canonical_store().unwrap();
        let physical = crate::workflow::tests::physical();
        let cancel = CancelSource::new();
        let originals = tests::source_documents(
            "package p {param a:Scalar=1;}",
            "package q {param b:Scalar=2;}",
        );
        let origin = runtime
            .package_from_sources(&[originals], physical.clone(), &cancel)
            .await
            .unwrap();
        store
            .create_project("create-doc", "project-doc", "Document composition")
            .await
            .unwrap();
        let project = store.project("project-doc").await.unwrap().unwrap();
        let base = runtime
            .project_fork(&project, "main-doc", "doc-problem", "fork-doc", &origin)
            .await
            .unwrap();
        let base = runtime
            .modeling_revision(base, physical.clone(), BTreeMap::new())
            .await
            .unwrap();
        let project = store.project("project-doc").await.unwrap().unwrap();
        let branch = store.project_branch("main-doc").await.unwrap().unwrap();
        let target_body = serde_json::to_vec(&crate::workflow::ProjectDraftDocuments {
            version: Default::default(),
            packages: vec![tests::source_documents(
                "package p {param a:Scalar=3;}",
                "package q {param b:Scalar=2;}",
            )],
        })
        .unwrap();
        runtime
            .project_save_draft(
                "save-target-doc",
                &project,
                &branch,
                "target-doc",
                &base.recorded_revision().unwrap().key,
                None,
                &target_body,
            )
            .await
            .unwrap();
        let draft = store.project_draft("target-doc").await.unwrap().unwrap();
        let publication = store
            .project_publication_intent(
                &project,
                &branch,
                &base.recorded_revision().unwrap().key,
                Some(&draft),
                "publish-target-doc",
            )
            .unwrap();
        let target = runtime
            .project_publish_draft(&publication, physical.clone(), &cancel)
            .await
            .unwrap();
        let body = serde_json::to_vec(&crate::workflow::ProjectDraftDocuments {
            version: Default::default(),
            packages: vec![tests::source_documents(
                "package p {param a:Scalar=1;}",
                "package q {param b:Scalar=4;}",
            )],
        })
        .unwrap();
        runtime
            .project_save_draft(
                "save-proposal-doc",
                &project,
                &branch,
                "proposal-doc",
                &base.recorded_revision().unwrap().key,
                None,
                &body,
            )
            .await
            .unwrap();
        let version = store
            .project_draft("proposal-doc")
            .await
            .unwrap()
            .unwrap()
            .version;
        let branch = store.project_branch("main-doc").await.unwrap().unwrap();
        let publication = store
            .project_publication_intent(
                &project,
                &branch,
                &target.recorded_revision().unwrap().key,
                None,
                "merge-doc",
            )
            .unwrap();
        let intent = runtime
            .project_merge_draft_intent(publication, &base, &target, &version, vec![], &cancel)
            .await
            .unwrap();
        let merged = runtime
            .project_publish_merge(&intent, physical.clone(), &cancel)
            .await
            .unwrap();
        let objects = merged.project_snapshot(&cancel).await.unwrap();
        for (path, text) in [
            ("models/a.pse", "package p {param a:Scalar=3;}"),
            ("models/b.pse", "package q {param b:Scalar=4;}"),
        ] {
            assert!(objects.objects.values().any(|object|matches!(object,ProjectSourceObject::Document(document) if document.document.path==path&&document.document.source_text.as_deref()==Some(text))));
        }
        let branch = store.project_branch("main-doc").await.unwrap().unwrap();
        let publication = store
            .project_publication_intent(
                &project,
                &branch,
                &merged.recorded_revision().unwrap().key,
                None,
                "undo-doc",
            )
            .unwrap();
        let undo = runtime
            .project_undo_intent(publication, &merged, &base, &cancel)
            .await
            .unwrap();
        let restored = runtime
            .project_publish_merge(&undo, physical, &cancel)
            .await
            .unwrap();
        assert!(
            restored
                .project_compare(&base, &cancel)
                .await
                .unwrap()
                .changes
                .is_empty(),
            "compensating undo restores exact authored text and declarations"
        );
        assert_eq!(
            restored.recorded_revision().unwrap().parent.as_deref(),
            Some("merge-doc")
        );
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn native_merge_undo_exact_replay_and_head_conflict() {
        let runtime = crate::workflow::tests::durable_runtime();
        let store = runtime.canonical_store().unwrap();
        let physical = crate::workflow::tests::physical();
        let cancel = CancelSource::new();
        let base = runtime
            .modeling_package(rows(), physical.clone())
            .await
            .unwrap();
        store
            .create_project("create", "project", "Experiment")
            .await
            .unwrap();
        let project = store.project("project").await.unwrap().unwrap();
        let a = runtime
            .project_fork(&project, "a", "problem-a", "fork-a", &base)
            .await
            .unwrap();
        let project = store.project("project").await.unwrap().unwrap();
        let b = runtime
            .project_fork(&project, "b", "problem-b", "fork-b", &base)
            .await
            .unwrap();
        let a = runtime
            .modeling_revision(a, physical.clone(), BTreeMap::new())
            .await
            .unwrap();
        let b = runtime
            .modeling_revision(b, physical.clone(), BTreeMap::new())
            .await
            .unwrap();
        let target = change(&runtime, &a, "a", "3", "edit-a", "a").await;
        let proposal = change(&runtime, &b, "b", "4", "edit-b", "b").await;
        let project = store.project("project").await.unwrap().unwrap();
        let branch = store.project_branch("a").await.unwrap().unwrap();
        let publication = store
            .project_publication_intent(
                &project,
                &branch,
                &target.recorded_revision().unwrap().key,
                None,
                "merge",
            )
            .unwrap();
        let intent = runtime
            .project_merge_intent(publication, &base, &target, &proposal, vec![], &cancel)
            .await
            .unwrap();
        let merged = runtime
            .project_publish_merge(&intent, physical.clone(), &cancel)
            .await
            .unwrap();
        let again = runtime
            .project_publish_merge(&intent, physical.clone(), &cancel)
            .await
            .unwrap();
        assert_eq!(merged.canonical_revision(), again.canonical_revision());
        let merged_rows = merged.declarations().await.unwrap();
        for (name, value) in [("a", "3"), ("b", "4")] {
            assert_eq!(
                merged_rows
                    .iter()
                    .find(|r| r.name == name)
                    .unwrap()
                    .value
                    .binding
                    .as_ref()
                    .unwrap()
                    .expression
                    .as_deref(),
                Some(value)
            );
        }
        let mut forged = intent.clone();
        forged.resolutions.push(ProjectMergeResolution {
            logical: "unrelated".into(),
            choice: ProjectMergeChoice::Delete,
        });
        assert!(
            runtime
                .project_publish_merge(&forged, physical.clone(), &cancel)
                .await
                .is_err()
        );
        let branch = store.project_branch("a").await.unwrap().unwrap();
        let undo_publication = store
            .project_publication_intent(
                &project,
                &branch,
                &merged.recorded_revision().unwrap().key,
                None,
                "undo",
            )
            .unwrap();
        let undo = runtime
            .project_undo_intent(undo_publication, &merged, &a, &cancel)
            .await
            .unwrap();
        let restored = runtime
            .project_publish_merge(&undo, physical.clone(), &cancel)
            .await
            .unwrap();
        assert_eq!(
            restored.recorded_revision().unwrap().parent,
            Some(merged.recorded_revision().unwrap().key.clone())
        );
        assert_eq!(
            &*restored.declarations().await.unwrap(),
            &*a.declarations().await.unwrap()
        );
        // The original branch and accepted merge remain independently readable after undo.
        assert_eq!(
            &*runtime
                .modeling_revision(
                    merged.recorded_revision().unwrap().clone(),
                    physical.clone(),
                    BTreeMap::new()
                )
                .await
                .unwrap()
                .declarations()
                .await
                .unwrap(),
            &*merged_rows
        );
        assert_eq!(
            runtime
                .project_publish_merge(&intent, physical.clone(), &cancel)
                .await
                .unwrap()
                .canonical_revision(),
            merged.canonical_revision()
        );
        let branch = store.project_branch("a").await.unwrap().unwrap();
        let stale_publication = store
            .project_publication_intent(
                &project,
                &branch,
                &restored.recorded_revision().unwrap().key,
                None,
                "stale-merge",
            )
            .unwrap();
        let stale = runtime
            .project_merge_intent(
                stale_publication,
                &base,
                &restored,
                &proposal,
                vec![],
                &cancel,
            )
            .await
            .unwrap();
        let moved = change(&runtime, &restored, "a", "8", "move-a", "a").await;
        assert!(
            runtime
                .project_publish_merge(&stale, physical, &cancel)
                .await
                .is_err()
        );
        assert_eq!(
            store.project_branch_revision("a").await.unwrap().unwrap(),
            *moved.recorded_revision().unwrap()
        );
        store.remove_isolated_fixture().await.unwrap();
    }
}
