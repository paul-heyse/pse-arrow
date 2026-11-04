// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive original admitted groups, composed through their existing authenticated owners.
use super::*;
use crate::{
    envelope::{CommandOutcome, ErrorRecord, Observation, Settlement},
    paths::CheckoutState,
    test_workflow::execute::Action,
};

/// The digest's source domain remains explicit even though both contain admitted units.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UnitContextOrigin {
    /// Original compiler/probe context JSON, bound by its native performing terminal.
    CompilerArtifact {
        /// Actual acquisition family recorded by the compiler owner.
        acquisition: artifact::Kind,
    },
    /// Entire original immutable workflow report, bound by its strict helper terminal.
    WorkflowReport {
        /// Original workflow action authenticated against its manifest and terminal.
        action: Action,
        /// Original performing report outcome, independent of evidence availability.
        workflow_outcome: serde_json::Value,
    },
}
/// Original selection data grants no native verdict or execution authority.
#[derive(Debug, Serialize, JsonSchema)]
pub struct UnitContextObservation {
    /// Original performing invocation; inspection creates no replacement operation.
    pub original_invocation: CliInvocationId,
    /// Exact context or whole-report digest in the separately declared origin domain.
    pub context_sha256: String,
    /// Owner and digest domain of the retained selection.
    pub origin: UnitContextOrigin,
    /// True includes an explicitly admitted zero-group selection; false marks missing admission.
    pub admission_available: bool,
    /// Ordered original groups stay separate, including overlapping unit populations.
    pub admission: Vec<Resolution>,
    /// Original performing outcome, not the success of this read.
    pub original_outcome: CommandOutcome,
    /// Original subject state reported by the performing owner.
    pub original_subject_state: SubjectState,
    /// Original operation settlement; inspection does not execute or settle it again.
    pub original_settlement: Settlement,
    /// Original observation coverage and limitations.
    pub original_observation: Observation,
    /// Original typed failures, retained even when selection data is available.
    pub original_failures: Vec<ErrorRecord>,
    /// Exact original bytes when the owning reader supplies a checked file address.
    pub full_context: Option<artifact::ArtifactFile>,
}

/// Select existing retained evidence only. Never runs metadata, admission or compilation.
pub fn unit_context(
    root: &Path,
    state: &CheckoutState,
    selector: &str,
) -> Result<Envelope<UnitContextObservation>, HostError> {
    let store = InvocationStore::new(state.clone());
    let id = store
        .resolve(selector, None)?
        .ok_or_else(|| invalid("original admitted-context invocation is absent"))?;
    let metadata = store.metadata::<serde::de::IgnoredAny>(&id)?;
    let command = &metadata.selection.command;
    if matches!(command.as_str(), "compiler generate" | "compiler probe") {
        let original = artifact::observe(root, state, id.as_str())?;
        let sha = original
            .providers
            .iter()
            .find(|provider| provider.provider == "original-managed-compiler-outcome/v1")
            .and_then(|provider| provider.analysis_generation.clone())
            .ok_or_else(|| invalid("original compiler context digest is absent"))?;
        let selected = artifact::selected_context(&original.payload);
        let mut response = Envelope::response(
            "inspect unit-context",
            "original admitted compiler context; no compilation or new selection",
            UnitContextObservation {
                original_invocation: original.payload.original_invocation,
                context_sha256: sha.clone(),
                origin: UnitContextOrigin::CompilerArtifact {
                    acquisition: original.payload.context.kind,
                },
                admission_available: true,
                admission: vec![original.payload.context.admitted],
                original_outcome: original.payload.original_outcome,
                original_subject_state: original.payload.original_subject_state,
                original_settlement: original.payload.original_settlement,
                original_observation: original.payload.original_observation,
                original_failures: original.payload.original_failures,
                full_context: selected.full_context,
            },
        );
        response.providers = original.providers;
        response.artifacts = original.artifacts;
        response.observation.scope = "original_admitted_compiler_artifact_context".into();
        return Ok(response);
    }
    let action = Action::from_command(command).ok_or_else(|| {
        invalid("selected original operation has no supported admitted compiler context")
    })?;
    let captured = crate::test_inventory::captured_report(&store, id.as_str(), None)?;
    let terminal = captured
        .original
        .payload
        .workflow
        .as_ref()
        .ok_or_else(|| invalid("original workflow terminal is absent"))?;
    let report = workflow_context(
        &captured.report,
        &terminal.terminal,
    )?;
    let sha = digest(&captured.report);
    let mut response = Envelope::response(
        "inspect unit-context",
        "original admitted workflow groups; metadata and compiler evidence remain separate",
        UnitContextObservation {
            original_invocation: id,
            context_sha256: sha.clone(),
            origin: UnitContextOrigin::WorkflowReport {
                action,
                workflow_outcome: report.outcome,
            },
            admission_available: report.selection.is_some(),
            admission: report.admission,
            original_outcome: captured.original.outcome,
            original_subject_state: captured.original.subject_state,
            original_settlement: captured.original.settlement,
            original_observation: captured.original.observation,
            original_failures: captured.original.failures,
            full_context: Some(artifact::ArtifactFile {
                path: captured.parent.join("report.json").into_os_string(),
                bytes: captured.report.len() as u64,
                sha256: sha.clone(),
            }),
        },
    );
    response.providers = captured.original.providers;
    response.artifacts = captured.original.artifacts;
    response.providers.push(ProviderContext {
        provider: "original-managed-workflow-unit-context/v1".into(),
        context: Some(format!(
            "original {} report; ordered admission groups",
            command
        )),
        analysis_generation: Some(sha),
        availability: if response.payload.admission_available {
            SubjectState::Available
        } else {
            SubjectState::Unknown
        },
        coordinates: Vec::new(),
    });
    response.observation.scope = "original_admitted_workflow_context".into();
    if !response.payload.admission_available {
        response.observation.completeness = Completeness::Partial;
        response.observation.limitations.push("The original workflow did not publish an admitted selection; no current selection was substituted.".into());
    }
    Ok(response)
}

fn workflow_context(
    bytes: &[u8],
    terminal: &crate::test_workflow::execute::TerminalRecord,
) -> Result<ReportSource, HostError> {
    let observation: crate::test_workflow::execute::ReportObservation = serde_json::from_slice(bytes).map_err(std::io::Error::other)?;
    observation.validate_against(terminal).map_err(|error| HostError::Workflow(Box::new(error)))?;
    let report: ReportSource = serde_json::from_slice(bytes).map_err(std::io::Error::other)?;
    if report.selection.is_none() && !report.admission.is_empty() {
        return Err(invalid(
            "original admission groups have no recorded selection",
        ));
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retained_unit_context_keeps_groups_and_rejects_changed_original_identity() {
        let id = CliInvocationId::new();
        let group = serde_json::to_value(Resolution {
            arguments: vec!["check".into(), "--lib".into()],
            packages: vec!["case".into()],
            units: Vec::new(),
            native_capabilities: Default::default(),
        })
        .unwrap();
        let report = serde_json::json!({"version":1,"invocation":id,"action":"check","outcome":"check_succeeded","selection":{},"admission":[group.clone(),group],"phases":[],"observations":[],"required_capture_complete":true,"failures":[]});
        let bytes = serde_json::to_vec(&report).unwrap();
        use crate::test_workflow::execute::{Outcome, ReportReference, TerminalRecord};
        let terminal = TerminalRecord { protocol:"pse.test-workflow".into(), version:1, invocation:id, action:Action::Check, outcome:Outcome::CheckSucceeded, required_capture_complete:true, report:ReportReference { path:"report.json".into(), sha256:digest(&bytes), byte_length:bytes.len() as u64 } };
        let original = workflow_context(&bytes, &terminal).unwrap();
        assert_eq!(original.admission.len(), 2);
        let mut changed = terminal.clone(); changed.invocation=CliInvocationId::new(); assert!(workflow_context(&bytes,&changed).is_err());
        let mut changed = terminal.clone(); changed.action=Action::Test; assert!(workflow_context(&bytes,&changed).is_err());
        let mut changed = terminal.clone(); changed.outcome=Outcome::Failed; assert!(workflow_context(&bytes,&changed).is_err());
        let mut changed = terminal.clone(); changed.required_capture_complete=false; assert!(workflow_context(&bytes,&changed).is_err());
        let mut missing = report;
        missing["selection"] = serde_json::Value::Null;
        assert!(
            workflow_context(&serde_json::to_vec(&missing).unwrap(), &terminal)
            .is_err()
        );
    }
}
