// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive validation relevance: original outcomes and source observations keep their owners.
use crate::{
    HostError,
    investigation::{self, Locator, ResultReference, SourceGuard, SourceKind},
    source_read::{SourceRequest, SourceText},
};
use schemars::JsonSchema;
use serde::Serialize;
use std::path::Path;
use tokio_util::sync::CancellationToken;

/// One bounded original-failure or source-to-validation inquiry.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Report {
    /// Original operand, retaining its evidence universe.
    pub reference: ResultReference,
    /// Exact frozen row and original phase/outcome context when selected.
    pub original: Option<crate::test_inventory::FrozenRecord>,
    /// Original compiler-owner detail/page, serialized without reconstructing its verdict.
    pub diagnostic: Option<serde_json::Value>,
    /// Guarded selected source bytes, independently useful without native preparation.
    pub source: Option<investigation::SearchResult>,
    /// Written setup/helper calls, bindings and observable assertion forms.
    pub structure: Option<crate::source_query::ValidationStructure>,
    /// Optional original native runnables/relatedness/cfg facts.
    pub native_tests: Option<crate::semantic::Answer>,
    /// Optional resolved calls; shared-helper fan-out remains bounded native evidence.
    pub native_calls: Option<crate::semantic::Answer>,
    /// Safe original inspection/selection actions; never execution actions.
    pub actions: Vec<crate::envelope::NextAction>,
    /// Complete selected source/row delivery, independent of assurance adequacy.
    pub complete: bool,
    /// Unknown correspondence, provider limits and omitted scope.
    pub limitations: Vec<String>,
}

/// Inspect existing evidence only. Optional semantic enrichment never prepares a provider.
pub async fn inspect(
    root: &Path,
    locator: &Locator,
    semantic: bool,
    budget: usize,
    token: &CancellationToken,
) -> Result<Report, HostError> {
    if token.is_cancelled() {
        return Err(investigation::Error::Cancelled.into());
    }
    let reference = ResultReference::new(locator.clone());
    reference.validate()?;
    let mut report = Report { reference: reference.clone(), original: None, diagnostic: None, source: None, structure: None, native_tests: None, native_calls: None, actions: Vec::new(), complete: true,
        limitations: vec!["References, shared setup, observable assertions, execution outcome and independent assurance are separate evidence. No coverage, adequacy or oracle-independence verdict is inferred.".into(), "Expected-value derivation is exposed as written initializers and positional operands, not complete data flow. A test call can leave the important invariant unasserted.".into()] };
    match locator {
        Locator::FrozenTest { invocation, .. } => {
            let state = crate::paths::CheckoutState::resolve(root, &std::env::vars_os().collect())?;
            let original = crate::test_inventory::frozen(&crate::invocation::InvocationStore::new(state), &reference)?;
            let incomplete = frozen_incompleteness(&original);
            report.complete &= incomplete.is_empty(); report.limitations.extend(incomplete);
            report.original = Some(original);
            report.actions.push(crate::envelope::NextAction::Command { argv: vec!["pse".into(), "inspect".into(), "invocation".into(), invocation.clone()], reason: "read the original performing outcome without rerunning".into() });
            report.actions.push(crate::envelope::NextAction::Command { argv: vec!["pse".into(), "inspect".into(), "tests".into(), invocation.clone(), "--failures".into()], reason: "read original diagnostic/setup/test failure rows and original artifact addresses".into() });
            report.limitations.push("An original harness identity or XML source spelling does not identify exact current test/production bytes. Select a separately guarded source witness before inspecting current assertions or checks.".into());
            if semantic { report.limitations.push("Native enrichment requires an independently selected current source; frozen outcomes are not reinterpreted in a current world.".into()); }
        }
        Locator::Evidence { invocation, kind: investigation::EvidenceKind::Diagnostic, selector } => {
            let state = crate::paths::CheckoutState::resolve(root, &std::env::vars_os().collect())?;
            let store = crate::invocation::InvocationStore::new(state);
            let limit = std::num::NonZeroUsize::new(budget.max(1)).ok_or_else(|| investigation::Error::Invalid("invalid diagnostic allowance".into()))?;
            let original = if let Some(selector) = selector {
                let address: crate::compiler_investigation::DiagnosticAddress = serde_json::from_str(selector).map_err(std::io::Error::other)?;
                if address.invocation.as_str() != invocation { return Err(investigation::Error::Invalid("diagnostic reference and original address name different invocations".into()).into()); }
                let bytes = std::num::NonZeroUsize::new(4_194_304).ok_or_else(|| investigation::Error::Invalid("invalid diagnostic record allowance".into()))?;
                let evidence = crate::compiler_investigation::diagnostic(&store, &address, bytes)?;
                serde_json::to_value(evidence.page(root, None, limit, true, false)?).map_err(std::io::Error::other)?
            } else {
                serde_json::to_value(crate::compiler_investigation::diagnostics(&store, invocation, None, None, limit, root)?).map_err(std::io::Error::other)?
            };
            report.diagnostic = Some(original);
            report.actions.push(crate::envelope::NextAction::Command { argv: vec!["pse".into(), "inspect".into(), "--result".into(), reference.encode()?], reason: "expand the original compiler diagnostic through its original owner".into() });
            report.actions.push(crate::envelope::NextAction::Command { argv: vec!["pse".into(), "inspect".into(), "tests".into(), invocation.clone(), "--failures".into()], reason: "inspect original setup, infrastructure and test failure distinctions".into() });
            report.complete = false;
            report.limitations.push("Original diagnostic reference retained. Its source span alone does not establish full original source identity or correspondence to current source.".into());
        }
        Locator::Source { .. } | Locator::Semantic { .. } => {
            let (source_root, selected, text) = source_witness(root, locator, token).await?;
            let mut source = investigation::providers::inspect(&source_root, &selected)?;
            if matches!(locator, Locator::Source { .. }) {
                report.reference = ResultReference::new(selected.clone());
            }
            let outline = crate::source_read::rust(text.text(), crate::source_read::RustEdition::E2024);
            let selected_range = match &selected { Locator::Source { range: Some(range), .. } => range.byte_start..range.byte_end, _ => 0..text.text().len() };
            let containing = outline.entries.iter().filter(|entry| entry.range.byte_start <= selected_range.start && entry.range.byte_end >= selected_range.end)
                .min_by_key(|entry| entry.range.byte_end - entry.range.byte_start);
            let range = containing.map_or(selected_range, |entry| entry.range.byte_start..entry.range.byte_end);
            let structure = crate::source_query::validation_structure(&text, range, budget);
            report.complete = structure.omitted == 0 && structure.parse_issues.is_empty();
            if let Some(address) = source.reference.as_ref()
                && matches!(address.locator, Locator::Source { source: SourceKind::Working, .. })
            {
                report.actions.push(crate::envelope::NextAction::Command { argv: vec!["pse".into(), "inspect".into(), "test-selection".into(), "--from-result".into(), address.encode()?], reason: "validate the current source witness and ask the existing owner for containing-package test candidates; no listing or execution".into() });
            }
            if semantic {
                let native = investigation::rust::source_inquiry(root, locator, token).await;
                match native {
                    Ok(native) => {
                        for (kind, tests) in [(crate::semantic::Kind::Tests, true), (crate::semantic::Kind::Calls, false)] {
                            match investigation::rust::semantic_operation_cancellable(root, &native, kind, token).await {
                                Ok(answer) => { let incomplete = native_incompleteness(&answer); report.complete &= incomplete.is_empty(); report.limitations.extend(incomplete); if tests { report.native_tests = Some(answer); } else { report.native_calls = Some(answer); } },
                                Err(error) => { if token.is_cancelled() { return Err(error.into()); } report.complete = false; report.limitations.push(format!("requested native enrichment unavailable: {error}")); }
                            }
                        }
                    }
                    Err(error) => { if token.is_cancelled() { return Err(error.into()); } report.complete = false; report.limitations.push(format!("requested native source correspondence unavailable: {error}")); }
                }
            }
            // Keep the original source reference and textual evidence; no whole-file rewrite.
            source.context.clear(); report.source = Some(source); report.structure = Some(structure);
            report.limitations.push("Written call/initializer relationships do not establish resolved helper identity, effective cfg or setup execution. Native tests distinguish discovery, relatedness and eligibility; neither they nor affected selection prove complete dependency impact.".into());
        }
        _ => return Err(investigation::Error::Invalid("relevance requires guarded source/native, frozen test/failure or original diagnostic evidence".into()).into()),
    }
    Ok(report)
}

fn frozen_incompleteness(original: &crate::test_inventory::FrozenRecord) -> Vec<String> {
    if original
        .context
        .get("required_capture_complete")
        .and_then(serde_json::Value::as_bool)
        != Some(true)
    {
        vec!["authenticated original report does not establish complete required raw capture; its original fields and failure row remain unchanged".into()]
    } else {
        Vec::new()
    }
}

pub(crate) fn native_incompleteness(answer: &crate::semantic::Answer) -> Vec<String> {
    let mut limitations = Vec::new();
    for (relation, omitted) in &answer.omitted_relationships {
        if *omitted != 0 {
            limitations.push(format!("selected native {relation:?} observation omits {omitted} whole relationship records"));
        }
    }
    for section in &answer.sections {
        if section.omitted != 0 {
            limitations.push(format!(
                "selected native section {} omits {} whole records",
                section.category, section.omitted
            ));
        }
    }
    if let Some(investigation) = &answer.observed.investigation {
        limitations.extend(
            investigation.limitations.iter().map(|limitation| {
                format!("selected native investigation limitation: {limitation}")
            }),
        );
    }
    limitations
}

/// Validate source guards before parsing; native locations retain their original byte witness.
pub(crate) async fn source_witness(
    root: &Path,
    locator: &Locator,
    token: &CancellationToken,
) -> Result<(std::path::PathBuf, Locator, SourceText), HostError> {
    let mut source_root = root.to_path_buf();
    let source = match locator {
        Locator::Source { .. } => locator.clone(),
        Locator::Semantic {
            line,
            generation,
            source_sha256,
            session_invocation,
            ..
        } => {
            if generation.is_none() || source_sha256.is_none() || session_invocation.is_none() {
                return Err(investigation::Error::Unavailable(
                    "native source witness requires original session, generation and source guards"
                        .into(),
                )
                .into());
            }
            // The original selected native owner validates session, project/configuration,
            // generation and bytes before any source-only projection is constructed.
            let answer = investigation::rust::semantic_operation_cancellable(
                root,
                locator,
                crate::semantic::Kind::Explain,
                token,
            )
            .await?;
            source_root = answer.session.configuration.project.canonicalize()?;
            let start = answer.source_byte_offset;
            Locator::Source {
                path: answer.source.path().to_path_buf().into(),
                source: SourceKind::Working,
                range: Some(investigation::MatchRange {
                    byte_start: start,
                    byte_end: start,
                    line_start: *line,
                    line_end: *line,
                }),
                guard: Some(SourceGuard {
                    sha256: Some(answer.source.source_sha256().into()),
                    root: Some(source_root.clone().into()),
                    ..Default::default()
                }),
            }
        }
        _ => {
            return Err(
                investigation::Error::Invalid("expected source/native witness".into()).into(),
            );
        }
    };
    let row = investigation::providers::inspect(&source_root, &source)?;
    let guarded = row
        .reference
        .ok_or_else(|| {
            investigation::Error::Invalid("source reader did not retain original reference".into())
        })?
        .locator;
    let Locator::Source {
        path, source: kind, ..
    } = &guarded
    else {
        return Err(investigation::Error::Invalid(
            "source reader returned another evidence kind".into(),
        )
        .into());
    };
    let request = match kind {
        SourceKind::Working => SourceRequest::Working {
            path: source_root.join(path.to_path_buf()),
        },
        SourceKind::Index => SourceRequest::Index {
            repository: source_root.clone(),
            path: path.to_path_buf(),
        },
        SourceKind::Revision { revision } => SourceRequest::Blob {
            repository: source_root.clone(),
            path: path.to_path_buf(),
            revision: revision.clone(),
        },
    };
    let text = request
        .acquire()
        .map_err(crate::source_command::SourceCommandError::from)?;
    // The acquire follows the guarded read, so revalidate its full identity before use.
    if let Locator::Source {
        guard: Some(guard), ..
    } = &guarded
        && guard
            .sha256
            .as_deref()
            .is_some_and(|sha| sha != text.identity().source_sha256())
    {
        return Err(investigation::Error::Stale(
            "source changed while acquiring validation facts".into(),
        )
        .into());
    }
    Ok((source_root, guarded, text))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authenticated_incomplete_original_report_stays_incomplete_and_changed_bytes_refuse() {
        use crate::{
            envelope::{CliInvocationId, Envelope, ErrorRecord, RedactionPolicy},
            managed::result::{ManagedSummary, WorkflowSummary},
            test_workflow::execute::{Action, Outcome, ReportReference, TerminalRecord},
        };
        use std::os::unix::fs::{DirBuilderExt as _, PermissionsExt as _};
        let scratch = tempfile::tempdir().unwrap();
        let environment = crate::environment::Environment::from([(
            "XDG_STATE_HOME".into(),
            scratch.path().join("state").into_os_string(),
        )]);
        let state = crate::paths::CheckoutState::from_paths(
            &scratch.path().join("checkout"),
            &scratch.path().join("common.git"),
            &environment,
        )
        .unwrap();
        let store = crate::invocation::InvocationStore::new(state);
        let id = CliInvocationId::new();
        let directory = store
            .begin(crate::invocation::InvocationSpec {
                invocation_id: id.clone(),
                command: "test".into(),
                session: None,
                parent: None,
                operation: (),
                control: crate::generation::PreparedGeneration {
                    generation: "unit.original".into(),
                    protocol: 1,
                    manifest_digest: "a".repeat(64),
                },
                environment_digest: "b".repeat(64),
                scientific: Vec::new(),
            })
            .unwrap();
        let parent = directory.path().join("workflow");
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&parent)
            .unwrap();
        let failure = ErrorRecord::from_typed(
            &HostError::Ownership("original unit setup capture incomplete".into()),
            &RedactionPolicy::default(),
        );
        let bytes = serde_json::to_vec(&serde_json::json!({"version":1,"invocation":id,"action":"test","outcome":"incomplete","required_capture_complete":false,"failures":[failure],"observations":[]})).unwrap();
        let sha = crate::source_read::source_digest(std::str::from_utf8(&bytes).unwrap());
        let report_path = parent.join("report.json");
        std::fs::write(&report_path, &bytes).unwrap();
        std::fs::set_permissions(&report_path, std::fs::Permissions::from_mode(0o400)).unwrap();
        let terminal = TerminalRecord {
            protocol: "pse.test-workflow".into(),
            version: 1,
            invocation: id.clone(),
            action: Action::Test,
            outcome: Outcome::Incomplete,
            report: ReportReference {
                path: "report.json".into(),
                sha256: sha.clone(),
                bytes: bytes.len() as u64,
            },
            required_capture_complete: false,
        };
        let mut envelope = Envelope::response(
            "test",
            "original incomplete unit evidence",
            ManagedSummary {
                bundle: None,
                native_lifetime_pending: false,
                workflow: Some(WorkflowSummary {
                    terminal,
                    report: report_path.clone().into_os_string(),
                }),
                child_exit: Some(125),
                stdout: None,
                stderr: None,
                borrowed_reservation: false,
                service_result: None,
            },
        );
        envelope.invocation_id = id.clone();
        let result_path = directory.path().join("result.json");
        std::fs::write(&result_path, serde_json::to_vec(&envelope).unwrap()).unwrap();
        std::fs::set_permissions(&result_path, std::fs::Permissions::from_mode(0o400)).unwrap();
        let reference = ResultReference::new(Locator::FrozenTest {
            invocation: id.to_string(),
            report_sha256: sha,
            inventory_sha256: None,
            selection: investigation::FrozenTestSelection::Failure { ordinal: 0 },
        });
        let original = crate::test_inventory::frozen(&store, &reference).unwrap();
        assert_eq!(original.context["required_capture_complete"], false);
        assert!(!frozen_incompleteness(&original).is_empty());
        std::fs::set_permissions(&report_path, std::fs::Permissions::from_mode(0o600)).unwrap();
        std::fs::write(&report_path, b"changed original").unwrap();
        assert!(crate::test_inventory::frozen(&store, &reference).is_err());
    }
}
