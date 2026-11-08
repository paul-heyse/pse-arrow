// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Read-only evidence from the existing selection, Cargo, Nextest and reporting owners.
use super::{Context, Stage, execute, pending};
use crate::{affected, catalog::description::Information, selection::Plan};
use schemars::JsonSchema;
use serde::Serialize;
use std::collections::BTreeMap;

/// Evidence independent of the normalized selection and actual compiler units.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SelectionEvidence<'a> {
    /// Conservative affected-package reach and widening from the changed-source owner.
    pub affected: Information<&'a AffectedSelection>,
    /// Validated current source from the source or analyzer owner, when supplied.
    pub source_candidate: Information<SourceCandidate>,
    /// Metadata declarations of targets in the selected packages. Admission identifies actual units.
    pub cargo_targets: Vec<TargetDeclaration<'a>>,
    /// Rustdoc route when the selected scope contains an eligible Cargo declaration.
    pub doctest_route: Information<&'static str>,
    /// Exact test identities are available only from the original Nextest inventory owner.
    pub nextest_inventory: Information<&'static str>,
    /// Original performing test outcome, available here only for a failed-test rerun selection.
    pub original_verdict: Information<&'a OriginalVerdict>,
}

/// Package reach from the same affected traversal that produced the selection.
#[derive(Debug, Serialize, JsonSchema)]
pub struct AffectedSelection {
    /// Direct changed-source owners and generated input consumers.
    pub changed_packages: Vec<String>,
    /// Selected packages before applying any explicit Cargo target selector.
    pub selected_packages: Vec<affected::SelectedPackage>,
    /// Availability of the affected declaration graph.
    pub graph: affected::Graph,
    /// Reasons for conservatively widening to all members.
    pub widening: Vec<affected::Widening>,
}
impl From<affected::Preview> for AffectedSelection {
    fn from(preview: affected::Preview) -> Self {
        Self {
            changed_packages: preview.changed_packages,
            selected_packages: preview.selected_packages,
            graph: preview.graph,
            widening: preview.widening,
        }
    }
}

/// Candidate lane, distinct from admission, enumeration and execution.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CandidateLane {
    /// Native harness selection remains subject to Cargo and Nextest.
    Nextest,
    /// Current Cargo declarations allow rustdoc to consider the candidate.
    Rustdoc,
}

/// Current source evidence validated by the selection receiver.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SourceCandidate {
    /// Exact working or semantic witness rechecked for this selection.
    pub reference: crate::investigation::ResultReference,
    /// Current containing workspace package.
    pub package_id: String,
    /// The owner of any subsequent executable inventory or doc-test execution.
    pub lane: CandidateLane,
}

/// Exact authored Cargo target properties, independent of runtime eligibility.
#[derive(Debug, Serialize, JsonSchema)]
pub struct TargetDeclaration<'a> {
    /// Cargo package ID, shared with admitted units.
    pub package_id: &'a str,
    /// Cargo target name.
    pub name: &'a str,
    /// Cargo target roles.
    pub kinds: Vec<String>,
    /// Target source file, shared with admitted units.
    pub source_path: &'a str,
    /// Cargo's test-harness declaration.
    pub test: bool,
    /// Cargo's doctest eligibility. Execution belongs to the `just doctest` lane.
    pub doctest: bool,
    /// Cargo's required feature names. Compiler units carry actual resolved features.
    pub required_features: &'a [String],
}

/// Observation authenticated by the original performing workflow owner.
#[derive(Debug, Serialize, JsonSchema)]
pub struct OriginalVerdict {
    /// Original test invocation.
    pub invocation: crate::envelope::CliInvocationId,
    /// Original reconciled outcome.
    pub outcome: execute::Outcome,
    /// Whether the owner established the required capture.
    pub required_capture_complete: bool,
}
impl OriginalVerdict {
    pub(crate) fn of(report: &execute::ReportObservation) -> Option<Self> {
        (report.action == execute::Action::Test).then(|| Self {
            invocation: report.invocation.clone(),
            outcome: report.outcome,
            required_capture_complete: report.required_capture_complete,
        })
    }
}

impl Context {
    pub(super) fn selection_evidence<'a>(
        &'a self,
        plan: &'a Plan,
        stage: Stage,
    ) -> SelectionEvidence<'a> {
        let mut roots = BTreeMap::<_, Vec<_>>::new();
        for root in plan.groups().iter().flat_map(|group| group.roots()) {
            roots.entry(root.package_id()).or_default().push(root);
        }
        let cargo_targets: Vec<_> = self
            .metadata
            .workspace_packages()
            .into_iter()
            .flat_map(|package| {
                let selected_roots = roots.get(package.id.repr.as_str());
                package.targets.iter().filter_map(move |target| {
                    let selected_roots = selected_roots?;
                    let kinds: Vec<_> = target.kind.iter().map(ToString::to_string).collect();
                    let source_path = target.src_path.as_str();
                    selected_roots
                        .iter()
                        .any(|root| {
                            root.target().is_none_or(|declared| {
                                declared.name == target.name
                                    && declared.kinds == kinds
                                    && declared.source_path == source_path
                            })
                        })
                        .then_some(TargetDeclaration {
                            package_id: package.id.repr.as_str(),
                            name: &target.name,
                            kinds,
                            source_path,
                            test: target.test,
                            doctest: target.doctest,
                            required_features: &target.required_features,
                        })
                })
            })
            .collect();
        let doctest_route = if cargo_targets.iter().any(|target| target.doctest) {
            Information::Available("just doctest")
        } else {
            pending("no selected Cargo target declares doctest eligibility")
        };
        SelectionEvidence {
            affected: self.source_evidence.affected.as_ref().map_or_else(
                || pending("changed-source reach was not requested"),
                Information::Available,
            ),
            source_candidate: pending("no guarded current source candidate was supplied"),
            cargo_targets,
            doctest_route,
            nextest_inventory: match (plan.operation(), stage) {
                (crate::selection::Operation::Test, Stage::Listed) => {
                    Information::Available("original Nextest inventory")
                }
                (crate::selection::Operation::Test, _) => {
                    pending("Nextest inventory requires explicit build/list")
                }
                _ => pending("compiler checks have no Nextest inventory"),
            },
            original_verdict: self.source_evidence.original_verdict.as_ref().map_or_else(
                || pending("no original performing test invocation was selected"),
                Information::Available,
            ),
        }
    }
}
