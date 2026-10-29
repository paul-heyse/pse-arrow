// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Existing project operator actions and public host projection declarations.
//! This is an interface to existing typed owners, not a workflow interpreter.
use crate::workflow::{
    PreparationDiscoveryRequest, ProjectDraftDocuments, ProjectDraftSaveIntent,
    ProjectExportRequest, ProjectImportIntent, ProjectMergeIntent, ProjectMergeResolution,
    ProjectSelectRequest, ProjectSelection, ResearchDatasetRequest, ResearchPublicationIntent,
    ResearchPublicationRequest, ResearchReadRequest, ResearchSelectionRequest, StudyCohortRequest,
    StudyKpiRequest, StudyObservationCursor, StudyObservationRequest, StudyWorkbenchRequest,
};
use pse_operations::{
    canonical_contexts::{ContextCut, ContextSelection},
    canonical_lifecycle::{LifecycleSelection, LifecycleTarget},
    canonical_projects::{Branch, Draft, DraftVersion, Project, ProjectPublication},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Shared default branch for an explicitly issued receiving project intent.
pub const DEFAULT_BRANCH: &str = "main";
/// Shared default bounded project document/member extent.
pub const DEFAULT_DOCUMENT_BYTES: usize = super::HOST_DOCUMENT_MAX;
/// Historical one-shot project operator resource defaults, distinct from capability profiles.
#[derive(Clone, Copy, Debug)]
pub struct ProjectOperatorDefaults {
    /// Accounted runtime memory allowance.
    pub memory_limit_bytes: usize,
    /// Existing scientific permit count, not host workload placement.
    pub threads: usize,
    /// Historical project-operator spill allowance.
    pub max_spill_bytes: u64,
}
/// One declaration of this operator's existing selected policy.
pub const PROJECT_OPERATOR_DEFAULTS: ProjectOperatorDefaults = ProjectOperatorDefaults {
    memory_limit_bytes: 16 * 1024 * 1024 * 1024,
    threads: 16,
    max_spill_bytes: 16 * 1024 * 1024 * 1024,
};

/// Existing operator spelling projects the shared document extent owner.
pub use DEFAULT_DOCUMENT_BYTES as PROJECT_DOCUMENT_BYTES;
/// Reuse the original engine setting default, preserving the historical 8192 rows.
pub fn default_batch_size() -> usize {
    pse_engine::ExecutionSettings::default().batch_size
}

fn project_document_bytes() -> usize {
    PROJECT_DOCUMENT_BYTES
}
/// Default bounded byte extent for an operator's frozen study-cohort export.
pub fn cohort_export_bytes() -> u64 {
    16 * 1024 * 1024
}
fn project_branch_name() -> String {
    DEFAULT_BRANCH.into()
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
/// Original bounded operator request and its finite typed action.
pub struct ProjectOperationRequest {
    /// Existing operator wire interpretation.
    pub version: pse_model::document::Version<1>,
    /// Required only for scientific source loading/admission; ordinary authored documents.
    pub physical: Option<ProjectDraftDocuments>,
    /// One original typed operation.
    pub action: ProjectAction,
}
macro_rules! project_actions {
    ($( $(#[$attribute:meta])* $effect:ident => $name:ident { $( $(#[$field_attribute:meta])* $field:ident : $field_type:ty, )* }, )*) => {
        #[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
        #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
        /// The original finite project operator action contract.
        pub enum ProjectAction {
            $(
                $(#[$attribute])*
                #[doc = concat!("Existing ", stringify!($name), " operation.")]
                #[schemars(extend("x-pse-effect-facts" = super::EffectFacts::$effect))]
                $name { $(
                    $(#[$field_attribute])*
                    #[doc = concat!("Original ", stringify!($field), " input.")]
                    $field: $field_type,
                )* },
            )*
        }
        impl ProjectAction {
            /// Original typed effect metadata for this action; never parsed from its name.
            pub const fn effect_facts(&self) -> super::EffectFacts {
                match self { $(Self::$name { .. } => super::EffectFacts::$effect,)* }
            }
        }
    };
}
project_actions! {
    OBSERVATION => ProjectClosure {
        request: ProjectExportRequest,
    },
    FILE_EXPORT => ProjectExport {
        destination: PathBuf,
        request: ProjectExportRequest,
    },
    CANONICAL_COMMAND => ProjectImportIssue {
        input: PathBuf,
        title: String,
        #[serde(default = "project_branch_name")]
        branch_name: String,
        #[serde(default = "project_document_bytes")]
        maximum_document_bytes: usize,
    },
    OBSERVATION => ProjectImportStatus {
        intent: ProjectImportIntent,
    },
    CANONICAL_COMMAND => ProjectImport {
        input: PathBuf,
        intent: ProjectImportIntent,
        #[serde(default = "project_document_bytes")]
        maximum_document_bytes: usize,
    },
    CANONICAL_COMMAND => ProjectImportRecover {
        intent: ProjectImportIntent,
        original: pse_operations::canonical_artifacts::Artifact,
    },
    OBSERVATION => ProjectImportOpen {
        intent: ProjectImportIntent,
    },
    OBSERVATION => LifecycleSelect {
        target: LifecycleTarget,
    },
    OBSERVATION => LifecycleExplain {
        selection: LifecycleSelection,
    },
    RETENTION_RELEASE => LifecycleRelease {
        selection: LifecycleSelection,
    },
    CANONICAL_COMMAND => ResearchDatasetCreate {
        request: ResearchDatasetRequest,
    },
    CANONICAL_COMMAND => ResearchSelectionCreate {
        request: ResearchSelectionRequest,
    },
    OBSERVATION => ResearchRead {
        request: ResearchReadRequest,
    },
    SOURCE_ADMISSION => ResearchDatasetOpen {
        key: String,
        context: ContextSelection,
    },
    SOURCE_ADMISSION => ResearchSelectionOpen {
        key: String,
    },
    OBSERVATION => ResearchPublicationPreview {
        request: ResearchPublicationRequest,
    },
    CANONICAL_COMMAND => ResearchPublish {
        intent: Box<ResearchPublicationIntent>,
    },
    SOURCE_ADMISSION => ResearchPublicationOpen {
        key: String,
    },
    OBSERVATION => StudyWorkbench {
        request: StudyWorkbenchRequest,
    },
    /// Capture counts only, or publish bounded NDJSON pages under the same live cut.
    CAPTURE => StudyCohort {
        request: StudyCohortRequest,
        /// Optional page export; receipt identifiers alone never reopen a capture.
        output: Option<PathBuf>,
        #[serde(default = "cohort_export_bytes")]
        maximum_bytes: u64,
    },
    FILE_EXPORT => StudyAnalyze {
        request: StudyCohortRequest,
        kpi: StudyKpiRequest,
        output: PathBuf,
    },
    FILE_EXPORT => StudyResult {
        request: StudyCohortRequest,
        member: usize,
        relation: String,
        start: u64,
        end: u64,
        output: PathBuf,
        maximum_bytes: u64,
    },
    OBSERVATION => StudyObserve {
        request: StudyObservationRequest,
        cursor: Option<StudyObservationCursor>,
        delay_ms: Option<u64>,
    },
    CANONICAL_COMMAND => Import {
        documents: ProjectDraftDocuments,
    },
    CANONICAL_COMMAND => ImportFiles {
        packages: Vec<PathBuf>,
        physical_packages: Vec<PathBuf>,
    },
    SOURCE_ADMISSION => Load {
        revision: String,
    },
    SOURCE_ADMISSION => Select {
        request: ProjectSelectRequest,
    },
    SOURCE_ADMISSION => OpenSelection {
        selection: ProjectSelection,
    },
    CANONICAL_COMMAND => Fork {
        project: Project,
        branch: String,
        problem: String,
        operation: String,
        source: String,
    },
    OBSERVATION => DraftRead {
        key: String,
    },
    CANONICAL_COMMAND => DraftIssue {
        operation: String,
        project: Project,
        branch: Branch,
        key: String,
        base: String,
        expected: Option<u64>,
        input: PathBuf,
    },
    CANONICAL_COMMAND => DraftSave {
        intent: ProjectDraftSaveIntent,
        input: PathBuf,
    },
    CANONICAL_COMMAND => DraftAdmit {
        operation: String,
        key: String,
    },
    SOURCE_ADMISSION => DraftAdmitVersion {
        version: DraftVersion,
    },
    CANONICAL_COMMAND => PublicationIssue {
        project: Project,
        branch: Branch,
        expected: String,
        draft: Option<Draft>,
        operation: String,
    },
    CANONICAL_COMMAND => DraftPublish {
        intent: ProjectPublication,
    },
    OBSERVATION => Compare {
        left: String,
        right: String,
        #[serde(default)]
        source_only: bool,
        #[serde(default)]
        right_physical: Option<ProjectDraftDocuments>,
    },
    OBSERVATION => SourceDetail {
        revision: String,
        logical: Vec<String>,
    },
    OBSERVATION => MergePreview {
        base: String,
        target: String,
        proposal: String,
        resolutions: Vec<ProjectMergeResolution>,
    },
    CANONICAL_COMMAND => MergeIssue {
        publication: ProjectPublication,
        base: String,
        target: String,
        proposal: String,
        resolutions: Vec<ProjectMergeResolution>,
    },
    CANONICAL_COMMAND => MergeDraftIssue {
        publication: ProjectPublication,
        base: String,
        target: String,
        version: DraftVersion,
        resolutions: Vec<ProjectMergeResolution>,
    },
    CANONICAL_COMMAND => MergePublish {
        intent: ProjectMergeIntent,
    },
    CANONICAL_COMMAND => UndoIssue {
        publication: ProjectPublication,
        target: String,
        restore: String,
    },
    CANONICAL_COMMAND => ContextRecord {
        operation: String,
        project: Project,
        scope: String,
        revision: String,
        effective_from_us: i64,
        effective_to_us: Option<i64>,
        corrects: Option<String>,
        deleted: bool,
    },
    OBSERVATION => ContextCut {
        project: String,
    },
    OBSERVATION => ContextSelect {
        cut: ContextCut,
        scope: String,
        effective_at_us: i64,
    },
    SOURCE_ADMISSION => ContextLoad {
        selection: ContextSelection,
    },
    SOURCE_ADMISSION => ContextReopen {
        selection: ContextSelection,
    },
    OBSERVATION => PreparationDiscovery {
        revision: String,
        request: PreparationDiscoveryRequest,
    },
}

/// Original public facade family; no additional operation is published automatically.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FacadeFamily {
    /// Existing project, context, source and lifecycle receiver methods.
    Project,
    /// Existing immutable research receiver methods.
    Research,
    /// Existing bounded study observation/cohort receiver methods.
    Study,
}
/// A finite public projection over genuinely typed receiver glue.
/// The helper AST supplies types; this owner supplies publication and shared defaults.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct ProjectMethod {
    /// Existing public method name on the native runtime and facade.
    pub name: &'static str,
    /// Existing facade composition owner.
    pub family: FacadeFamily,
    /// Actual PyO3 calling convention, including owner-referenced defaults.
    pub pyo3_signature: Option<&'static str>,
    /// Actual public operation effect facts, independent of host representation.
    pub effect_facts: super::EffectFacts,
}
const METHODS: &[ProjectMethod] = &[
    ProjectMethod {
        name: "freeze_study_cohort",
        family: FacadeFamily::Study,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::CAPTURE,
    },
    ProjectMethod {
        name: "observe_study",
        family: FacadeFamily::Study,
        pyo3_signature: Some("signature=(request,*,cursor=None)"),
        effect_facts: super::EffectFacts::OBSERVATION,
    },
    ProjectMethod {
        name: "modeling_revision",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::SOURCE_ADMISSION,
    },
    ProjectMethod {
        name: "project_context_cut",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::OBSERVATION,
    },
    ProjectMethod {
        name: "project_context_select",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::OBSERVATION,
    },
    ProjectMethod {
        name: "project_context_record",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::OBSERVATION,
    },
    ProjectMethod {
        name: "project_context_reopen",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::SOURCE_ADMISSION,
    },
    ProjectMethod {
        name: "project_closure",
        family: FacadeFamily::Project,
        pyo3_signature: Some("signature=(request,physical=None)"),
        effect_facts: super::EffectFacts::OBSERVATION,
    },
    ProjectMethod {
        name: "project_export",
        family: FacadeFamily::Project,
        pyo3_signature: Some("signature=(destination,request,physical=None)"),
        effect_facts: super::EffectFacts::FILE_EXPORT,
    },
    ProjectMethod {
        name: "project_import_issue",
        family: FacadeFamily::Project,
        pyo3_signature: Some(
            "signature=(bundle,*,title,branch_name=pse_runtime::host_operations::project::DEFAULT_BRANCH)",
        ),
        effect_facts: super::EffectFacts::CANONICAL_COMMAND,
    },
    ProjectMethod {
        name: "project_import",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::CANONICAL_COMMAND,
    },
    ProjectMethod {
        name: "project_import_status",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::OBSERVATION,
    },
    ProjectMethod {
        name: "project_import_recover",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::CANONICAL_COMMAND,
    },
    ProjectMethod {
        name: "project_import_open",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::OBSERVATION,
    },
    ProjectMethod {
        name: "lifecycle_select",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::OBSERVATION,
    },
    ProjectMethod {
        name: "lifecycle_explain",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::OBSERVATION,
    },
    ProjectMethod {
        name: "lifecycle_release",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::RETENTION_RELEASE,
    },
    ProjectMethod {
        name: "research_dataset_create",
        family: FacadeFamily::Research,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::CANONICAL_COMMAND,
    },
    ProjectMethod {
        name: "research_selection_create",
        family: FacadeFamily::Research,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::CANONICAL_COMMAND,
    },
    ProjectMethod {
        name: "research_read",
        family: FacadeFamily::Research,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::OBSERVATION,
    },
    ProjectMethod {
        name: "research_dataset_open",
        family: FacadeFamily::Research,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::SOURCE_ADMISSION,
    },
    ProjectMethod {
        name: "research_selection_open",
        family: FacadeFamily::Research,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::SOURCE_ADMISSION,
    },
    ProjectMethod {
        name: "research_publication_preview",
        family: FacadeFamily::Research,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::OBSERVATION,
    },
    ProjectMethod {
        name: "research_publish",
        family: FacadeFamily::Research,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::CANONICAL_COMMAND,
    },
    ProjectMethod {
        name: "research_publication_open",
        family: FacadeFamily::Research,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::SOURCE_ADMISSION,
    },
    ProjectMethod {
        name: "project_context_package",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::SOURCE_ADMISSION,
    },
    ProjectMethod {
        name: "project_context_append",
        family: FacadeFamily::Project,
        pyo3_signature: Some(
            "signature=(operation,project,scope,source,effective_from_us,*,effective_to_us=None,corrects=None,deleted=false)",
        ),
        effect_facts: super::EffectFacts::CANONICAL_COMMAND,
    },
    ProjectMethod {
        name: "project_create",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::CANONICAL_COMMAND,
    },
    ProjectMethod {
        name: "project_record",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::OBSERVATION,
    },
    ProjectMethod {
        name: "project_branch_record",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::OBSERVATION,
    },
    ProjectMethod {
        name: "project_branches",
        family: FacadeFamily::Project,
        pyo3_signature: Some("signature=(project, *, cursor=None)"),
        effect_facts: super::EffectFacts::OBSERVATION,
    },
    ProjectMethod {
        name: "project_fork",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::CANONICAL_COMMAND,
    },
    ProjectMethod {
        name: "project_publication_intent",
        family: FacadeFamily::Project,
        pyo3_signature: Some("signature=(project,branch,expected,operation,*,draft=None)"),
        effect_facts: super::EffectFacts::CANONICAL_COMMAND,
    },
    ProjectMethod {
        name: "project_draft_save_intent",
        family: FacadeFamily::Project,
        pyo3_signature: Some(
            "signature=(operation,project,branch,key,base,content,*,expected=None)",
        ),
        effect_facts: super::EffectFacts::CANONICAL_COMMAND,
    },
    ProjectMethod {
        name: "project_draft_save",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::CANONICAL_COMMAND,
    },
    ProjectMethod {
        name: "project_draft_record",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::OBSERVATION,
    },
    ProjectMethod {
        name: "project_draft_bytes",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::OBSERVATION,
    },
    ProjectMethod {
        name: "project_select",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::SOURCE_ADMISSION,
    },
    ProjectMethod {
        name: "project_open_selection",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::SOURCE_ADMISSION,
    },
    ProjectMethod {
        name: "project_navigate",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::CANONICAL_COMMAND,
    },
    ProjectMethod {
        name: "project_metadata",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::OBSERVATION,
    },
    ProjectMethod {
        name: "project_draft_version",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::OBSERVATION,
    },
    ProjectMethod {
        name: "project_draft_version_bytes",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::OBSERVATION,
    },
    ProjectMethod {
        name: "project_draft_delta",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::OBSERVATION,
    },
    ProjectMethod {
        name: "project_admit_draft",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::CANONICAL_COMMAND,
    },
    ProjectMethod {
        name: "project_acknowledgement",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::OBSERVATION,
    },
    ProjectMethod {
        name: "project_publish_draft",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::CANONICAL_COMMAND,
    },
    ProjectMethod {
        name: "project_merge_preview",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::OBSERVATION,
    },
    ProjectMethod {
        name: "project_merge_intent",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::CANONICAL_COMMAND,
    },
    ProjectMethod {
        name: "project_merge_draft_intent",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::CANONICAL_COMMAND,
    },
    ProjectMethod {
        name: "project_undo_intent",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::CANONICAL_COMMAND,
    },
    ProjectMethod {
        name: "project_publish_merge",
        family: FacadeFamily::Project,
        pyo3_signature: None,
        effect_facts: super::EffectFacts::CANONICAL_COMMAND,
    },
];
/// The exact existing public interface projected into PyO3 and Python.
pub fn methods() -> &'static [ProjectMethod] {
    METHODS
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn project_operator_wire_defaults_effects_and_public_projection_set_remain_owned() {
        let action: ProjectAction = serde_json::from_value(serde_json::json!({
            "kind": "project_import_issue", "input": "original.pack", "title": "Original"
        }))
        .unwrap();
        let ProjectAction::ProjectImportIssue {
            branch_name,
            maximum_document_bytes,
            ..
        } = &action
        else {
            panic!("original action expected")
        };
        assert_eq!(branch_name, "main");
        assert_eq!(*maximum_document_bytes, 16 << 20);
        assert!(!action.effect_facts().read_only);
        assert_eq!(PROJECT_OPERATOR_DEFAULTS.max_spill_bytes, 16 << 30);
        assert_eq!(
            default_batch_size(),
            pse_engine::ExecutionSettings::default().batch_size
        );
        let names = methods()
            .iter()
            .map(|method| method.name)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(names.len(), 52);
        assert!(!names.contains("study_workbench_page"));
        for expected in [
            "project_import_issue",
            "project_context_append",
            "lifecycle_release",
            "research_publish",
            "freeze_study_cohort",
            "modeling_revision",
        ] {
            assert!(names.contains(expected));
        }
        assert!(
            methods()
                .iter()
                .find(|method| method.name == "lifecycle_release")
                .unwrap()
                .effect_facts
                .destructive
        );
        let capture = methods()
            .iter()
            .find(|method| method.name == "freeze_study_cohort")
            .unwrap()
            .effect_facts;
        assert!(matches!(capture.scope, super::super::EffectScope::Capture));
        assert!(!capture.read_only && !capture.idempotent && !capture.destructive);
        assert!(matches!(
            methods()
                .iter()
                .find(|method| method.name == "project_select")
                .unwrap()
                .effect_facts
                .scope,
            super::super::EffectScope::SourceAdmission
        ));
        assert!(matches!(
            methods()
                .iter()
                .find(|method| method.name == "project_admit_draft")
                .unwrap()
                .effect_facts
                .scope,
            super::super::EffectScope::CanonicalCommand
        ));
        let schema = schemars::schema_for!(ProjectAction).to_value();
        let variants = schema
            .get("oneOf")
            .or_else(|| schema.get("anyOf"))
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(variants.len(), 49);
        assert!(
            variants
                .iter()
                .all(|variant| variant.get("x-pse-effect-facts").is_some())
        );
        let cohort = variants
            .iter()
            .find(|variant| variant["properties"]["kind"]["const"] == "study_cohort")
            .unwrap();
        assert_eq!(cohort["x-pse-effect-facts"]["scope"], "capture");
        let admitted = variants
            .iter()
            .find(|variant| variant["properties"]["kind"]["const"] == "draft_admit_version")
            .unwrap();
        assert_eq!(admitted["x-pse-effect-facts"]["scope"], "source_admission");
    }
}
