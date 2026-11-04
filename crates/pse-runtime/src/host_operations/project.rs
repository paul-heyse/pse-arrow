// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Existing typed native project operator actions and effect declarations.
//! This is an interface to existing typed owners, not a workflow interpreter.

/// Shared implementations used by native project adapters.
pub mod handler;

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

/// Shared original effect declarations; adapters do not restate their meaning.
pub use pse_model::operation_effects::{EffectFacts, EffectScope};

/// Shared default branch for an explicitly issued receiving project intent.
pub const DEFAULT_BRANCH: &str = "main";
/// Shared default bounded project document/member extent.
pub const DEFAULT_DOCUMENT_BYTES: usize = 16 << 20;
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
    /// Native owner preparation interpretation; obsolete custom JSON is not accepted.
    pub version: pse_model::document::Version<2>,
    /// Required only for scientific source loading/admission; ordinary authored documents.
    pub physical: Option<ProjectDraftDocuments>,
    /// One original typed operation.
    pub action: ProjectAction,
}

/// A deliberately selected actual action root and its original declared effects.
/// This does not describe unselected alternatives of the native project operator.
#[derive(Debug)]
pub struct SelectedActionProjection {
    /// Complete selected request type world.
    pub graph: pse_model::boundary_projection::ProjectionGraph,
    /// The selected actual action payload root.
    pub root: pse_model::boundary_projection::TypeRef,
    /// Original macro-declared effect facts, without a second interpretation table.
    pub effects: EffectFacts,
}

macro_rules! project_actions {
    ($( $(#[$attribute:meta])* $effect:ident => $name:ident
        $( [projection: $wire:literal, version: $version:literal, register: $register:ident] )?
        { $( $(#[$field_attribute:meta])* $field:ident : $field_type:ty $( => $tag:expr )?, )* }, )*) => {
        #[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
        #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
        /// The original finite project operator action contract.
        pub enum ProjectAction {
            $(
                $(#[$attribute])*
                #[doc = concat!("Existing ", stringify!($name), " operation.")]
                #[schemars(extend("x-pse-effect-facts" = EffectFacts::$effect))]
                $name { $(
                    $(#[$field_attribute])*
                    #[doc = concat!("Original ", stringify!($field), " input.")]
                    $field: $field_type,
                )* },
            )*
        }
        impl ProjectAction {
            /// Original typed effect metadata for this action; never parsed from its name.
            pub const fn effect_facts(&self) -> EffectFacts {
                match self { $(Self::$name { .. } => EffectFacts::$effect,)* }
            }
            /// Inspect this explicitly selected action, refusing unselected boundaries.
            /// # Errors
            /// Unselected actions and invalid/unsupported selected field types refuse.
            pub fn selected_projection(&self) -> Result<SelectedActionProjection, pse_model::boundary_projection::ProjectionError> {
                let mut graph = pse_model::boundary_projection::ProjectionGraph::default();
                let (root, effects) = match self {
                    $( Self::$name { .. } => project_actions!(@selected graph; $name $(, $register)?), )*
                }?;
                Ok(SelectedActionProjection { graph, root, effects })
            }
            $( project_actions!(@method $effect, $name $(, $wire, $version, $register)?; $( $field: $field_type $( => $tag )?, )*); )*
        }
    };
    (@selected $graph:ident; $name:ident, $register:ident) => { Self::$register(&mut $graph) };
    (@selected $graph:ident; $name:ident) => {
        Err(pse_model::boundary_projection::ProjectionError::invalid(
            concat!("ProjectAction::", stringify!($name)), "this action is outside the selected boundary projection"))
    };
    (@method $effect:ident, $name:ident, $wire:literal, $version:literal, $register:ident; $( $field:ident: $field_type:ty => $tag:expr, )*) => {
        /// Resolve this selected actual action's fields and original effect facts.
        /// # Errors
        /// Invalid or unsupported actual field projections refuse.
        pub fn $register(graph: &mut pse_model::boundary_projection::ProjectionGraph)
            -> Result<(pse_model::boundary_projection::TypeRef, EffectFacts), pse_model::boundary_projection::ProjectionError> {
            let root = graph.declare::<Self>($wire, file!(), $version,
                pse_model::boundary_projection::Direction::Input, &[], &[], |graph| {
                    Ok(pse_model::boundary_projection::Shape::Record(vec![$(
                        pse_model::boundary_projection::Field::of::<$field_type>(graph,
                            stringify!($field), $tag, pse_model::boundary_projection::Direction::Input, vec![])?,
                    )*]))
                })?;
            Ok((root, EffectFacts::$effect))
        }
    };
    (@method $effect:ident, $name:ident; $( $field:ident: $field_type:ty $( => $tag:expr )?, )*) => {};
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
    OBSERVATION => PreparationDiscovery [projection: "PreparationDiscoveryAction", version: 2, register: preparation_discovery_projection] {
        revision: String => 1,
        request: PreparationDiscoveryRequest => 2,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_preparation_action_projects_actual_types_and_original_effect_facts() {
        let root = pse_modeling::DeclarationId::from_bytes([1; 16]);
        let action = ProjectAction::PreparationDiscovery {
            revision: "exact-local-parent".into(),
            request: PreparationDiscoveryRequest {
                root,
                instance: pse_modeling::specialize::root_instance(root),
                bindings: pse_modeling::Bindings::default(),
                limits: pse_modeling::Limits::default(),
                durable_after: None,
            },
        };
        let selected = action.selected_projection().unwrap();
        assert!(matches!(selected.effects.scope, EffectScope::Observation));
        assert!(selected.effects.read_only && selected.effects.idempotent);
        assert!(!selected.effects.destructive);
        assert_eq!(
            selected.root,
            pse_model::boundary_projection::TypeRef::Named("PreparationDiscoveryAction".into())
        );
        let owner = &selected.graph.definitions()["PreparationDiscoveryAction"];
        assert_eq!(owner.version, 2);
        let pse_model::boundary_projection::Shape::Record(fields) = &owner.shape else {
            panic!("actual selected action record")
        };
        assert_eq!(
            fields
                .iter()
                .map(|field| (field.name.as_str(), field.tag))
                .collect::<Vec<_>>(),
            [("revision", 1), ("request", 2)]
        );
        assert_eq!(
            fields[1].ty,
            pse_model::boundary_projection::TypeRef::Named("PreparationDiscoveryRequest".into())
        );
        assert!(selected.graph.definitions().contains_key("StructuralValue"));
        assert!(
            selected
                .graph
                .definitions()
                .contains_key("ProductHeaderCursor")
        );
        assert!(
            ProjectAction::DraftRead {
                key: "unselected".into()
            }
            .selected_projection()
            .is_err()
        );
    }
    #[test]
    fn project_operator_wire_defaults_effects_and_native_schema_remain_owned() {
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
        assert_eq!(cohort["x-pse-effect-facts"]["read_only"], false);
        assert_eq!(cohort["x-pse-effect-facts"]["idempotent"], false);
        assert_eq!(cohort["x-pse-effect-facts"]["destructive"], false);
        let admitted = variants
            .iter()
            .find(|variant| variant["properties"]["kind"]["const"] == "draft_admit_version")
            .unwrap();
        assert_eq!(admitted["x-pse-effect-facts"]["scope"], "source_admission");
    }
}
