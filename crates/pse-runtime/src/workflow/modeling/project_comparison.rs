// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Logical authored-object comparison and explicit three-way publication decisions.
//! No solver, expression canonicalizer or mutable project name participates.
use super::{ModelingPackage, Runtime, PhysicalContext, WorkflowError, contract};
use crate::{CancelSource, workflow::SourceDelta};
use pse_authoring::language::{Declaration};
use pse_model::generated::{authored::{documents, fit_cases, packages}, enums::ModelingDeclarationKind as Kind};
use pse_operations::canonical_projects::ProjectPublication;
use std::collections::{BTreeMap, BTreeSet};

/// Exact source identity used by a comparison; local tokens never stand in for durable keys.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectComparisonSelection {
    pub issuing_store: Option<String>,
    pub revision: Option<String>,
    pub local_revision: String,
    pub source: String,
    pub physical: String,
}
/// Conservative impacts from authored meaning, not a claim of numerical equivalence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProjectChangeImpact { Source, Value, Structural, Unknown }
/// One changed logical object. Values are available only by selected detail request.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectObjectChange {
    pub logical: String,
    pub added: bool,
    pub removed: bool,
    pub impacts: Vec<ProjectChangeImpact>,
    pub fields: Vec<String>,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectComparison {
    pub from: ProjectComparisonSelection,
    pub to: ProjectComparisonSelection,
    pub semantic_context_matches: bool,
    pub changes: Vec<ProjectObjectChange>,
}
/// One explicitly selected admitted authored object, retaining its native typed shape.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ProjectSourceObject { Declaration(Declaration), Document(ProjectDocumentSource), Fit(fit_cases::Row), Package(packages::Row) }
/// Source metadata plus exact data identity. Passive comparison does not copy binary payloads.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectDocumentSource {
    pub document: documents::Row,
    pub content_digest: Option<String>,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectSourceDetail { pub logical: String, pub object: Option<ProjectSourceObject> }
/// Explicit resolution of an actual conflict. Delete is never inferred from missing input.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProjectMergeChoice { Target, Proposal, Base, Delete }
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectMergeResolution { pub logical: String, pub choice: ProjectMergeChoice }
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectMergeConflict { pub logical: String, pub kind: ProjectMergeConflictKind }
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProjectMergeConflictKind { DeleteEdit, ConflictingAddition, DifferentEdits }
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectMergePreview { pub conflicts: Vec<ProjectMergeConflict>, pub changes: Vec<ProjectObjectChange> }
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProjectMergeMode { Merge, Undo }
/// Complete immutable decision retained before issuing a source effect.
/// The canonical request binds its commitment and additional ancestry in the source transaction.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectMergeIntent {
    pub version: pse_model::document::Version<1>,
    pub publication: ProjectPublication,
    pub mode: ProjectMergeMode,
    pub base: String,
    pub target: String,
    pub proposal: String,
    pub physical: String,
    pub resolutions: Vec<ProjectMergeResolution>,
}
struct Snapshot {
    order: Vec<String>,
    objects: BTreeMap<String, ProjectSourceObject>,
    // Decoded clones remain accounted to this exact observation.
    _owner: std::sync::Arc<pse_columnar::AllocationLease>,
}
fn checkpoint(cancel: &CancelSource) -> Result<(), WorkflowError> {
    cancel.token().checkpoint().map_err(pse_engine::EngineError::from)?;
    if cancel.deadline().is_some_and(|deadline| std::time::Instant::now() >= deadline) { return Err(contract("project comparison deadline elapsed")); }
    Ok(())
}
impl ModelingPackage {
    fn project_comparison_selection(&self) -> Result<ProjectComparisonSelection, WorkflowError> {
        Ok(ProjectComparisonSelection { issuing_store: match &self.runtime.durability() { crate::workflow::Durability::Durable(_) => Some(self.runtime.canonical_store()?.issuing_store()?.into()), crate::workflow::Durability::Ephemeral => None }, revision: self.canonical_revision().map(|r| r.key.clone()), local_revision: self.revision_id().into(), source: self.revision.identity().to_string(), physical: self.physical.key.to_string() })
    }
    async fn project_snapshot(&self, cancel: &CancelSource) -> Result<Snapshot, WorkflowError> {
        checkpoint(cancel)?;
        let parent = self.delta_parent(cancel).await?;
        let extent = self.revision.admitted.objects.values().filter(|o| o.kind != "data").try_fold(65536usize, |n, o| n.checked_add(o.retained_bytes().checked_mul(3)?)).ok_or_else(|| contract("comparison inventory extent"))?;
        let owner = self.runtime.shared.math().reserve("project:comparison-inventory", extent)?;
        let mut objects = BTreeMap::new();
        let mut order = Vec::new();
        for row in parent.rows {
            let key = format!("modeling:{}", row.declaration_id.as_id());
            order.push(key.clone()); objects.insert(key, ProjectSourceObject::Declaration(row));
        }
        for row in parent.documents.into_values() {
            let content_digest = self.revision.admitted.objects.get(&format!("data:{}", row.document_id)).map(|data| data.version.clone());
            let key = format!("document:{}", row.document_id);
            order.push(key.clone()); objects.insert(key, ProjectSourceObject::Document(ProjectDocumentSource { document: row, content_digest }));
        }
        for object in self.revision.admitted.objects.values().filter(|o| o.kind == "package") {
            let row: packages::Row = serde_json::from_slice(&object.payload).map_err(|e| contract(e.to_string()))?;
            order.push(object.logical.clone()); objects.insert(object.logical.clone(), ProjectSourceObject::Package(row));
        }
        let fits = self.fit_declarations().await?;
        for row in &fits.fits {
            let key = format!("fit:{}", row.fit_id);
            order.push(key.clone()); objects.insert(key, ProjectSourceObject::Fit(row.clone()));
        }
        checkpoint(cancel)?;
        Ok(Snapshot { order, objects, _owner: owner })
    }
    /// Compact authored-object comparison. A physical mismatch refuses semantic interpretation.
    pub async fn project_compare(&self, other: &Self, cancel: &CancelSource) -> Result<ProjectComparison, WorkflowError> {
        self.runtime.scope_activity(async {
            if self.physical.key != other.physical.key { return Err(contract("semantic comparison physical contexts differ; request source-only comparison explicitly")); }
            self.project_compare_mode(other, true, cancel).await
        }).await
    }
    /// Explicit source-only view across contexts, never a scientific compatibility claim.
    pub async fn project_compare_source(&self, other: &Self, cancel: &CancelSource) -> Result<ProjectComparison, WorkflowError> {
        self.runtime.scope_activity(self.project_compare_mode(other, false, cancel)).await
    }
    async fn project_compare_mode(&self, other: &Self, semantic: bool, cancel: &CancelSource) -> Result<ProjectComparison, WorkflowError> {
        let from = self.project_snapshot(cancel).await?;
        let to = other.project_snapshot(cancel).await?;
        let mut changes = compare(&from.objects, &to.objects)?;
        if !semantic { for change in &mut changes { change.impacts = vec![ProjectChangeImpact::Source, ProjectChangeImpact::Unknown]; } }
        Ok(ProjectComparison { from: self.project_comparison_selection()?, to: other.project_comparison_selection()?, semantic_context_matches: semantic && self.physical.key == other.physical.key, changes })
    }
    /// Fetch only caller-selected typed object detail; unknown identities remain explicit absences.
    pub async fn project_source_detail(&self, logical: &[String], cancel: &CancelSource) -> Result<Vec<ProjectSourceDetail>, WorkflowError> {
        self.runtime.scope_activity(async {
            if logical.len() > 256 || logical.iter().any(|k| k.len() > 4096) { return Err(contract("project selected detail request exceeds bounds")); }
            let snapshot = self.project_snapshot(cancel).await?;
            Ok(logical.iter().map(|key| ProjectSourceDetail { logical: key.clone(), object: snapshot.objects.get(key).cloned() }).collect())
        }).await
    }
}
fn fields(a: &ProjectSourceObject, b: &ProjectSourceObject) -> Result<Vec<String>, WorkflowError> {
    let a = serde_json::to_value(a).map_err(|e| contract(e.to_string()))?;
    let b = serde_json::to_value(b).map_err(|e| contract(e.to_string()))?;
    let (Some(a), Some(b)) = (a.get("value").and_then(serde_json::Value::as_object), b.get("value").and_then(serde_json::Value::as_object)) else { return Ok(vec!["kind".into()]); };
    Ok(a.keys().chain(b.keys()).collect::<BTreeSet<_>>().into_iter().filter(|k| a.get(*k) != b.get(*k)).cloned().collect())
}
fn impacts(a: Option<&ProjectSourceObject>, b: Option<&ProjectSourceObject>) -> Vec<ProjectChangeImpact> {
    use ProjectChangeImpact::{Source, Value, Structural, Unknown};
    match (a,b) {
        (Some(ProjectSourceObject::Declaration(a)), Some(ProjectSourceObject::Declaration(b))) => {
            let mut normalized = a.clone(); normalized.source_start = b.source_start; normalized.source_end = b.source_end;
            if normalized == *b { return vec![Source]; }
            // Only numeric literal parameter initializers are known value-only edits.
            // A changed reference, equation, unit/type or domain remains structural.
            if a.value.kind == Kind::Parameter && b.value.kind == Kind::Parameter {
                if let (Some(before), Some(after)) = (&mut normalized.value.binding, &b.value.binding) {
                    let literal = |s: &Option<String>| s.as_ref().is_some_and(|s| s.parse::<f64>().is_ok_and(f64::is_finite));
                    if literal(&before.expression) && literal(&after.expression) {
                        before.expression = after.expression.clone();
                        if normalized == *b { return vec![Source, Value]; }
                    }
                }
            }
            vec![Source, Structural]
        }
        (Some(ProjectSourceObject::Document(a)), Some(ProjectSourceObject::Document(b))) => {
            if a.content_digest == b.content_digest && a.document.path == b.document.path && a.document.package_id == b.document.package_id { vec![Source] } else { vec![Source, Unknown] }
        }
        (Some(ProjectSourceObject::Fit(_)), Some(ProjectSourceObject::Fit(_))) => vec![Source, Structural],
        (None, Some(ProjectSourceObject::Declaration(_))) | (Some(ProjectSourceObject::Declaration(_)), None) | (None, Some(ProjectSourceObject::Fit(_))) | (Some(ProjectSourceObject::Fit(_)), None) => vec![Source, Structural],
        _ => vec![Source, Unknown],
    }
}
fn compare(a: &BTreeMap<String, ProjectSourceObject>, b: &BTreeMap<String, ProjectSourceObject>) -> Result<Vec<ProjectObjectChange>, WorkflowError> {
    a.keys().chain(b.keys()).collect::<BTreeSet<_>>().into_iter().filter_map(|key| {
        let (old,new) = (a.get(key), b.get(key));
        if old == new { return None; }
        Some(Ok(ProjectObjectChange { logical: key.clone(), added: old.is_none(), removed: new.is_none(), impacts: impacts(old,new), fields: match (old,new) { (Some(a),Some(b)) => match fields(a,b) { Ok(fields) => fields, Err(e) => return Some(Err(e)) }, _ => Vec::new() } }))
    }).collect()
}
fn merge_objects(base: &Snapshot, target: &Snapshot, proposal: &Snapshot, resolutions: &[ProjectMergeResolution]) -> Result<(BTreeMap<String, ProjectSourceObject>, Vec<ProjectMergeConflict>), WorkflowError> {
    let mut decisions = BTreeMap::new();
    for resolution in resolutions {
        if decisions.insert(resolution.logical.clone(), resolution.choice).is_some() { return Err(contract("merge resolution repeats a logical identity")); }
    }
    let keys = base.objects.keys().chain(target.objects.keys()).chain(proposal.objects.keys()).collect::<BTreeSet<_>>();
    let mut merged = BTreeMap::new();
    let mut conflicts = Vec::new();
    for key in keys {
        let (b,t,p) = (base.objects.get(key), target.objects.get(key), proposal.objects.get(key));
        let conflict = t != b && p != b && t != p;
        let selected = if conflict {
            match decisions.remove(key) {
                Some(ProjectMergeChoice::Target) => t,
                Some(ProjectMergeChoice::Proposal) => p,
                Some(ProjectMergeChoice::Base) => b,
                Some(ProjectMergeChoice::Delete) => None,
                None => { conflicts.push(ProjectMergeConflict { logical: key.clone(), kind: if t.is_none() || p.is_none() { ProjectMergeConflictKind::DeleteEdit } else if b.is_none() { ProjectMergeConflictKind::ConflictingAddition } else { ProjectMergeConflictKind::DifferentEdits } }); t }
            }
        } else { if t == b { p } else { t } };
        if let Some(value) = selected { merged.insert(key.clone(), value.clone()); }
    }
    if !decisions.is_empty() { return Err(contract("resolution does not name an actual merge conflict")); }
    Ok((merged, conflicts))
}
async fn delta_to(target: &ModelingPackage, old: &Snapshot, new: &BTreeMap<String, ProjectSourceObject>, proposal_order: &[String], sources: &[&ModelingPackage]) -> Result<SourceDelta, WorkflowError> {
    let mut delta = SourceDelta { parent: target.revision_id().into(), ..Default::default() };
    let order = old.order.iter().chain(proposal_order).chain(new.keys()).collect::<Vec<_>>();
    let mut seen = BTreeSet::new();
    for key in order {
        if !seen.insert(key) { continue; }
        match (old.objects.get(key), new.get(key)) {
            (a,b) if a == b => {},
            (None,Some(ProjectSourceObject::Declaration(row))) => delta.additions.push(row.clone()),
            (Some(_),Some(ProjectSourceObject::Declaration(row))) => delta.replacements.push(row.clone()),
            (Some(ProjectSourceObject::Declaration(row)),None) => delta.removals.push(row.declaration_id),
            (None,Some(ProjectSourceObject::Document(row))) => delta.documents.additions.push(document_body(row,sources)?),
            (Some(_),Some(ProjectSourceObject::Document(row))) => delta.documents.replacements.push(document_body(row,sources)?),
            (Some(ProjectSourceObject::Document(row)),None) => delta.documents.removals.push(row.document.document_id),
            (None,Some(ProjectSourceObject::Fit(row))) => delta.fits.additions.push(row.clone()),
            (Some(_),Some(ProjectSourceObject::Fit(row))) => delta.fits.replacements.push(row.clone()),
            (Some(ProjectSourceObject::Fit(row)),None) => delta.fits.removals.push(row.fit_id),
            _ => return Err(contract("merge logical identity changed object family")),
        }
    }
    Ok(delta)
}
fn document_body(source: &ProjectDocumentSource, packages: &[&ModelingPackage]) -> Result<documents::Row, WorkflowError> {
    let mut row = source.document.clone();
    if let Some(version) = &source.content_digest {
        let logical = format!("data:{}", row.document_id);
        let data = packages.iter().find_map(|package| package.revision.admitted.objects.get(&logical).filter(|object| &object.version == version)).ok_or_else(|| contract("merged document body unavailable from exact original source"))?;
        row.content = Some(data.payload.clone().into());
    }
    Ok(row)
}
impl ProjectMergeIntent {
    fn commitment(&self) -> Result<String, WorkflowError> {
        let bytes = serde_json::to_vec(&(1u8, &self.mode, &self.base, &self.target, &self.proposal, &self.physical, &self.resolutions)).map_err(|e| contract(e.to_string()))?;
        Ok(pse_operations::canonical_execution::result_payload_digest(&bytes))
    }
    pub fn validate(&self) -> Result<(), WorkflowError> {
        self.publication.validate()?;
        if self.base.is_empty() || self.proposal.is_empty() || self.target.is_empty() || self.physical.is_empty() || self.publication.expected() != Some(self.target.as_str()) || self.publication.context_commitment() != Some(self.commitment()?.as_str()) {
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
        let mut package = local.modeling_from_documents(documents, physical, cancel).await?;
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
        let key = intent.expected().ok_or_else(|| contract("admitted publication requires an exact parent"))?;
        let revision = store.revision(key).await?.ok_or_else(|| contract("admitted publication parent unavailable"))?;
        let parent = self.modeling_revision(revision,admitted.physical.clone(),admitted.providers.as_ref().clone()).await?;
        super::canonical::publish_admitted_project(self,intent,&parent,admitted,cancel).await
    }
    async fn project_is_ancestor(&self, ancestor: &str, descendant: &str, cancel: &CancelSource) -> Result<bool, WorkflowError> {
        let store = self.canonical_store()?;
        let mut pending = vec![descendant.to_owned()]; let mut seen = BTreeSet::new();
        while let Some(key) = pending.pop() {
            checkpoint(cancel)?;
            if key == ancestor { return Ok(true); }
            if !seen.insert(key.clone()) { continue; }
            if seen.len() > 8192 { return Err(contract("project ancestry traversal exceeds bounded source history")); }
            let revision = store.revision(&key).await?.ok_or_else(|| contract("project ancestry source unavailable"))?;
            pending.extend(store.project_revision_parents(&revision).await?);
        }
        Ok(false)
    }
    pub async fn project_merge_preview(&self, base: &ModelingPackage, target: &ModelingPackage, proposal: &ModelingPackage, resolutions: &[ProjectMergeResolution], cancel: &CancelSource) -> Result<ProjectMergePreview, WorkflowError> {
        self.scope_activity(async {
            if base.physical.key != target.physical.key || proposal.physical.key != target.physical.key { return Err(contract("merge physical contexts differ")); }
            let (b,t,p) = (base.project_snapshot(cancel).await?, target.project_snapshot(cancel).await?, proposal.project_snapshot(cancel).await?);
            let (merged,conflicts) = merge_objects(&b,&t,&p,resolutions)?;
            Ok(ProjectMergePreview { conflicts, changes: compare(&t.objects,&merged)? })
        }).await
    }
    /// Retain an exact decision before publication; unresolved conflicts issue no intent.
    pub async fn project_merge_intent(&self, publication: ProjectPublication, base: &ModelingPackage, target: &ModelingPackage, proposal: &ModelingPackage, resolutions: Vec<ProjectMergeResolution>, cancel: &CancelSource) -> Result<ProjectMergeIntent, WorkflowError> {
        self.scope_activity(async {
            let (base_key,target_key,proposal_key) = (&base.recorded_revision()?.key,&target.recorded_revision()?.key,&proposal.recorded_revision()?.key);
            if publication.expected() != Some(target_key.as_str()) || !self.project_is_ancestor(base_key,target_key,cancel).await? || !self.project_is_ancestor(base_key,proposal_key,cancel).await? { return Err(contract("merge base is not an ancestor of both exact selections")); }
            let preview = self.project_merge_preview(base,target,proposal,&resolutions,cancel).await?;
            if !preview.conflicts.is_empty() { return Err(contract("merge has unresolved logical-object conflicts")); }
            let mut intent = ProjectMergeIntent { version: Default::default(), publication: publication.with_merge_ancestry(base_key,proposal_key)?, mode: ProjectMergeMode::Merge, base: base_key.clone(), target: target_key.clone(), proposal: proposal_key.clone(), physical: target.physical.key.to_string(), resolutions };
            let commitment = intent.commitment()?;
            intent.publication = intent.publication.with_context_commitment(&commitment)?;
            intent.validate()?;
            Ok(intent)
        }).await
    }
    /// Undo means a new admitted source revision with the chosen earlier authored state.
    pub async fn project_undo_intent(&self, publication: ProjectPublication, target: &ModelingPackage, restore: &ModelingPackage, cancel: &CancelSource) -> Result<ProjectMergeIntent, WorkflowError> {
        self.scope_activity(async {
            let target_key = &target.recorded_revision()?.key; let restore_key = &restore.recorded_revision()?.key;
            if publication.expected() != Some(target_key.as_str()) || target.physical.key != restore.physical.key || !self.project_is_ancestor(restore_key,target_key,cancel).await? { return Err(contract("undo source is not a compatible exact ancestor")); }
            let mut intent = ProjectMergeIntent { version: Default::default(), publication: publication.with_merge_ancestry(restore_key,restore_key)?, mode: ProjectMergeMode::Undo, base: restore_key.clone(), target: target_key.clone(), proposal: restore_key.clone(), physical: target.physical.key.to_string(), resolutions: Vec::new() };
            let commitment = intent.commitment()?;
            intent.publication = intent.publication.with_context_commitment(&commitment)?;
            intent.validate()?; Ok(intent)
        }).await
    }
    /// Settle original acknowledgement first; never reinterpret a later head or draft as this intent.
    pub async fn project_publish_merge(&self, intent: &ProjectMergeIntent, physical: PhysicalContext, cancel: &CancelSource) -> Result<ModelingPackage, WorkflowError> {
        self.scope_activity(async {
            intent.validate()?;
            if physical.key.to_string() != intent.physical { return Err(contract("merge receiving physical context differs")); }
            let store = self.canonical_store()?;
            if let Some(revision) = store.settled_project_publication(&intent.publication).await? { return self.modeling_revision(revision,physical,BTreeMap::new()).await; }
            let mut packages = Vec::new();
            for key in [&intent.base,&intent.target,&intent.proposal] {
                let revision = store.revision(key).await?.ok_or_else(|| contract("merge original source unavailable"))?;
                packages.push(self.modeling_revision(revision,physical.clone(),BTreeMap::new()).await?);
            }
            let (base,target,proposal) = (&packages[0],&packages[1],&packages[2]);
            if !self.project_is_ancestor(&intent.base,&intent.target,cancel).await? || !self.project_is_ancestor(&intent.base,&intent.proposal,cancel).await? { return Err(contract("retained merge ancestry differs")); }
            let (b,t,p) = (base.project_snapshot(cancel).await?,target.project_snapshot(cancel).await?,proposal.project_snapshot(cancel).await?);
            let (merged,conflicts) = match intent.mode { ProjectMergeMode::Merge => merge_objects(&b,&t,&p,&intent.resolutions)?, ProjectMergeMode::Undo => { if intent.base != intent.proposal || !intent.resolutions.is_empty() { return Err(contract("undo decision shape differs")); } (p.objects.clone(),Vec::new()) } };
            if !conflicts.is_empty() { return Err(contract("retained merge has unresolved conflicts")); }
            // Package metadata changes cannot be smuggled through declaration/document merging.
            for source in [base,proposal] {
                let metadata = |package: &ModelingPackage| package.revision.admitted.objects.values().filter(|o| o.kind == "package").map(|o| (o.logical.clone(),o.payload.clone())).collect::<BTreeMap<_,_>>();
                if metadata(source) != metadata(target) { return Err(contract("merge package manifests differ; explicit compatible admission required")); }
            }
            let delta = delta_to(target,&t,&merged,&p.order,&[target,proposal,base]).await?;
            let admitted = target.project_admit_delta(&delta,cancel).await?;
            super::canonical::publish_admitted_project(self,&intent.publication,target,&admitted,cancel).await
        }).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn declarations(text: &str) -> Vec<Declaration> {
        pse_authoring::language::parse(text,pse_ids::SemanticId::NIL,pse_authoring::language::IdentityPolicy::Named,Default::default()).unwrap()
    }
    fn declaration(text: &str, name: &str) -> Declaration {
        declarations(text).into_iter().find(|row| row.name == name).unwrap()
    }
    fn snapshot(runtime: &Runtime, rows: Vec<Declaration>) -> Snapshot {
        let order = rows.iter().map(|row| format!("modeling:{}",row.declaration_id.as_id())).collect();
        let objects = rows.into_iter().map(|row| (format!("modeling:{}",row.declaration_id.as_id()),ProjectSourceObject::Declaration(row))).collect();
        Snapshot { order, objects, _owner: runtime.shared.math().reserve("project:test-snapshot",65536).unwrap() }
    }
    #[test]
    fn authored_changes_have_conservative_independent_meaning() {
        let original = declaration("package p { def R { param k:Scalar=1; var x:Scalar; eq balance:x==k; } }","k");
        let changed = declaration("package p { def R { param k:Scalar=2; var x:Scalar; eq balance:x==k; } }","k");
        assert_eq!(impacts(Some(&ProjectSourceObject::Declaration(original.clone())),Some(&ProjectSourceObject::Declaration(changed))),vec![ProjectChangeImpact::Source,ProjectChangeImpact::Value]);
        let mut presentation = original.clone(); presentation.source_start += 3; presentation.source_end += 3;
        assert_eq!(impacts(Some(&ProjectSourceObject::Declaration(original.clone())),Some(&ProjectSourceObject::Declaration(presentation))),vec![ProjectChangeImpact::Source]);
        let old_equation = declaration("package p { def R { param k:Scalar=1; var x:Scalar; eq balance:x==k; } }","balance");
        let changed_equation = declaration("package p { def R { param k:Scalar=1; var x:Scalar; eq balance:2*x==k; } }","balance");
        assert_eq!(impacts(Some(&ProjectSourceObject::Declaration(old_equation)),Some(&ProjectSourceObject::Declaration(changed_equation))),vec![ProjectChangeImpact::Source,ProjectChangeImpact::Structural]);
        let doc = |path: &str| ProjectSourceObject::Document(ProjectDocumentSource { document: documents::Row { document_id: pse_ids::SemanticId::NIL, package_id: pse_model::generated::identities::PackageId::from(pse_ids::SemanticId::NIL), path:path.into(),source_text:Some("# comment".into()),content:None }, content_digest:None });
        assert_eq!(impacts(Some(&doc("old")),Some(&doc("new"))),vec![ProjectChangeImpact::Source,ProjectChangeImpact::Unknown]);
    }
    #[tokio::test]
    async fn independent_objects_merge_and_same_object_needs_an_explicit_choice() {
        let runtime = crate::workflow::tests::runtime();
        let original = declarations("package p { param a:Scalar=1; param b:Scalar=2; }");
        let mut target_rows = original.clone();
        target_rows.iter_mut().find(|r| r.name == "a").unwrap().value.binding.as_mut().unwrap().expression = Some("3".into());
        let mut proposal_rows = original.clone();
        proposal_rows.iter_mut().find(|r| r.name == "b").unwrap().value.binding.as_mut().unwrap().expression = Some("4".into());
        let (base,target,proposal) = (snapshot(&runtime,original.clone()),snapshot(&runtime,target_rows.clone()),snapshot(&runtime,proposal_rows));
        let (merged,conflicts) = merge_objects(&base,&target,&proposal,&[]).unwrap();
        assert!(conflicts.is_empty());
        let values = merged.values().filter_map(|v| match v { ProjectSourceObject::Declaration(row) if row.name == "a" || row.name == "b" => Some((row.name.clone(),row.value.binding.as_ref().unwrap().expression.clone().unwrap())),_=>None }).collect::<BTreeMap<_,_>>();
        assert_eq!(values,BTreeMap::from([("a".into(),"3".into()),("b".into(),"4".into())]));
        let mut other_rows = original; other_rows.iter_mut().find(|r|r.name == "a").unwrap().value.binding.as_mut().unwrap().expression = Some("9".into());
        let other = snapshot(&runtime,other_rows);
        let (_,conflicts) = merge_objects(&base,&target,&other,&[]).unwrap(); assert_eq!(conflicts.len(),1);
        let logical = conflicts[0].logical.clone();
        let (resolved,conflicts) = merge_objects(&base,&target,&other,&[ProjectMergeResolution { logical:logical.clone(),choice:ProjectMergeChoice::Proposal }]).unwrap();
        assert!(conflicts.is_empty()); assert_eq!(resolved[&logical],other.objects[&logical]);
        assert!(merge_objects(&base,&target,&proposal,&[ProjectMergeResolution { logical,choice:ProjectMergeChoice::Target }]).is_err());
    }
    #[tokio::test]
    async fn delete_edit_and_conflicting_additions_do_not_choose_last_writer() {
        let runtime = crate::workflow::tests::runtime();
        let row = declaration("package p { param k:Scalar=1; }","k");
        let mut changed = row.clone(); changed.value.binding.as_mut().unwrap().expression = Some("5".into());
        let base = snapshot(&runtime,vec![row.clone()]); let deleted = snapshot(&runtime,vec![]); let edited = snapshot(&runtime,vec![changed]);
        let (_,conflicts) = merge_objects(&base,&deleted,&edited,&[]).unwrap(); assert_eq!(conflicts[0].kind,ProjectMergeConflictKind::DeleteEdit);
        let empty = snapshot(&runtime,vec![]); let added = snapshot(&runtime,vec![row]);
        let (_,conflicts) = merge_objects(&empty,&added,&edited,&[]).unwrap(); assert_eq!(conflicts[0].kind,ProjectMergeConflictKind::ConflictingAddition);
    }
    #[tokio::test]
    async fn local_merge_admission_preserves_target_order_and_refuses_wrong_parent() {
        let runtime = crate::workflow::tests::runtime();
        let rows = declarations("package p { param a:Scalar=1; param b:Scalar=2; }");
        let package = runtime.modeling_package(rows.clone(),crate::workflow::tests::physical()).await.unwrap();
        let mut next = rows.clone(); next.iter_mut().find(|r|r.name=="a").unwrap().value.binding.as_mut().unwrap().expression=Some("3".into());
        let target = package.project_snapshot(&CancelSource::new()).await.unwrap();
        let proposal = snapshot(&runtime,next.clone());
        let delta = delta_to(&package,&target,&proposal.objects,&proposal.order,&[&package]).await.unwrap();
        let admitted = package.project_admit_delta(&delta,&CancelSource::new()).await.unwrap();
        assert_eq!(&*admitted.declarations().await.unwrap(),next.as_slice());
        assert!(admitted.canonical_revision().is_none());
        assert!(package.project_admit_delta(&SourceDelta { parent:"unrelated".into(),..Default::default() },&CancelSource::new()).await.is_err());
    }
}
