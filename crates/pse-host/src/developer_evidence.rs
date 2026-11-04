// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Portable observations preserve original bytes without recreating native authority.
use crate::{HostError, developer_evidence_contracts as contracts, envelope::CliInvocationId, invocation::InvocationStore};
use buffa::Message as _;
use pse_contract_proto::support::{NativeFrom, decode_root, required, unknown_fields};
use protovalidate_buffa::Validate as _;
use schemars::JsonSchema;
use serde::Serialize;
use sha2::{Digest as _, Sha256};
use std::{fs::OpenOptions, io::Write as _, os::unix::fs::OpenOptionsExt as _, path::Path};

pub(crate) const MAX_INPUT_BYTES: usize = 64 * 1024 * 1024;

/// Source-attributed refusal, retaining the original decoder and native owner causes.
#[derive(Debug, thiserror::Error)]
pub enum EvidenceError {
    /// A bounded read or supplied buffer exceeded the selected evidence family limit.
    #[error("evidence input contains at least {observed_at_least} bytes, exceeding limit {maximum}")]
    InputLimit {
        /// Observed extent; a bounded file read need not consume the remaining bytes.
        observed_at_least: usize,
        /// Explicit reader limit; no partial native interpretation was produced.
        maximum: usize,
    },
    /// Original encoder refusal.
    #[error("evidence encoding: {0}")]
    Encode(#[from] buffa::EncodeError),
    /// The supplied artifact exceeds explicit decode bounds or has invalid wire structure.
    #[error("evidence wire: {0}")]
    Decode(#[from] buffa::DecodeError),
    /// Original source metadata checker refusal.
    #[error("evidence metadata: {0}")]
    Metadata(#[from] pse_contract_proto::MetadataError),
    /// The one shared checker retained an unsupported or incompatible interpretation.
    #[error("evidence interpretation: {assessment:?}")]
    Interpretation {
        /// Original typed disposition and every source-attributed finding.
        assessment: pse_contract::metadata::Assessment,
    },
    /// Native correspondence or maintained-validator refusal.
    #[error("evidence projection: {0}")]
    Projection(#[from] pse_contract_proto::support::BoundaryProjectionError),
    /// Original passive report/inventory parser refusal.
    #[error("evidence original JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// Original workflow association refusal.
    #[error("evidence workflow: {0}")]
    Workflow(#[from] crate::test_workflow::WorkflowError),
    /// Explicit unsupported or inconsistent association.
    #[error("evidence association: {0}")]
    Invalid(String),
}
pse_diagnostics::impl_diagnostic! {EvidenceError,
    code(_this){Some(pse_diagnostics::DiagnosticCode::HostOwnership)},
    forward(_this){None},help(_this){None},related(_this){None},source(_this){None}
}
impl From<pse_contract::boundary_projection::AdapterError> for EvidenceError {
    fn from(error: pse_contract::boundary_projection::AdapterError) -> Self { Self::Projection(error.into()) }
}
impl From<protovalidate_buffa::ValidationError> for EvidenceError {
    fn from(error: protovalidate_buffa::ValidationError) -> Self { Self::Projection(error.into()) }
}

/// User-requested export receipt, never a retained verdict store.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Export {
    /// Original performing invocation.
    pub invocation: CliInvocationId,
    /// Exclusively created user-selected destination.
    pub output: crate::investigation::NativePath,
    /// Exact generated root message.
    pub message: String,
    /// Digest of the exact exported payload bytes.
    pub sha256: String,
    /// Complete exported byte extent.
    pub bytes: usize,
}

/// Export from the existing native custody reader. Existing destinations refuse.
pub fn export(store: &InvocationStore, selector: &str, session: Option<&str>, output: &Path, report_only: bool) -> Result<Export, HostError> {
    let (invocation, message, bytes) = if report_only {
        let original = crate::test_inventory::captured_report(store, selector, session)?;
        let wire = contracts::encode_captured_workflow_report(&original).map_err(EvidenceError::from)?;
        (original.id, "CapturedWorkflowReport", wire.try_encode_to_vec().map_err(EvidenceError::from)?)
    } else {
        let original = crate::test_inventory::captured(store, selector, session)?;
        let wire = contracts::encode_captured_workflow_evidence(&original).map_err(EvidenceError::from)?;
        (original.id, "CapturedWorkflowEvidence", wire.try_encode_to_vec().map_err(EvidenceError::from)?)
    };
    let mut destination = OpenOptions::new().create_new(true).write(true).mode(0o600).open(output)?;
    destination.write_all(&bytes)?;
    destination.sync_all()?;
    Ok(Export { invocation, output: std::path::absolute(output)?.into(), message: format!("pse.developer.evidence.v1.{message}"), sha256: digest(&bytes), bytes: bytes.len() })
}

/// Integrity of one acquired observation, separate from native execution eligibility.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Readback {
    /// Exact original invocation, correlated with the original terminal/report.
    pub invocation: CliInvocationId,
    /// Exact retained report bytes committed by the included terminal.
    pub report_sha256: String,
    /// Original report interpretation; legacy one has no inventory commitment.
    pub report_version: u32,
    /// Present only for full evidence whose original v2 report committed these bytes.
    pub inventory_sha256: Option<String>,
    /// Original complete selected identities, reconciled with the inventory.
    pub selected_tests: Option<usize>,
    /// Original native listing observation, unavailable for compiler actions.
    pub selected_population_complete: Option<bool>,
    /// Manager's original recorded-at-read claim, separate from physical state now.
    pub recorded_manager_settlement: Option<crate::envelope::Settlement>,
    /// Missing historical native observations stay unavailable.
    pub recorded_native_lifetime_pending: Option<bool>,
    /// Content consistency does not independently authenticate portable producer identity.
    pub limitations: Vec<String>,
}

/// Read an explicitly acquired descriptor world and binary payload with finite limits.
/// Buffa 0.9.2 DecodeOptions limits input size, recursion and materialized elements;
/// <https://github.com/anthropics/buffa/blob/v0.9.2/docs/guide.md#decode-options>.
/// This constructs passive data only, never CapturedEvidence, Report or a verdict.
pub fn inspect(descriptors: &buffa_descriptor::generated::descriptor::FileDescriptorSet, message: &str, bytes: &[u8]) -> Result<Readback, EvidenceError> {
    use contracts::pb::pse::developer::evidence::v1 as pb;
    if bytes.len() > MAX_INPUT_BYTES { return Err(EvidenceError::InputLimit { observed_at_least: bytes.len(), maximum: MAX_INPUT_BYTES }); }
    let assessment = pse_contract_proto::check_descriptors(descriptors, "developer_evidence.proto", &contracts::Policy, &[])?;
    if assessment.disposition != pse_contract::metadata::Disposition::Compatible {
        return Err(EvidenceError::Interpretation { assessment });
    }
    let compiled = buffa_descriptor::generated::descriptor::FileDescriptorSet::decode_from_slice(contracts::DESCRIPTORS)?;
    if descriptors != &compiled {
        return Err(EvidenceError::Invalid("developer_evidence.proto: this compiled reader requires the complete compiled descriptor content; acquired wire tags, kinds, nested shapes or options cannot select a different decoder world".into()));
    }
    let options = buffa::DecodeOptions::new().with_max_message_size(MAX_INPUT_BYTES).with_recursion_limit(64).with_element_memory_limit(MAX_INPUT_BYTES);
    let (invocation, original, observation, terminal, inventory, selected, complete, settlement, pending) = match message {
        "pse.developer.evidence.v1.CapturedWorkflowReport" => {
            let wire: pb::CapturedWorkflowReport = options.decode_from_slice(bytes)?;
            wire.validate()?;
            unknown_fields(&wire.__buffa_unknown_fields, "CapturedWorkflowReport")?;
            (wire.invocation, wire.report_original, wire.report_observation, wire.terminal, None, None, None, wire.recorded_manager_settlement, wire.recorded_native_lifetime_pending)
        }
        "pse.developer.evidence.v1.CapturedWorkflowEvidence" => {
            let wire: pb::CapturedWorkflowEvidence = options.decode_from_slice(bytes)?;
            wire.validate()?;
            unknown_fields(&wire.__buffa_unknown_fields, "CapturedWorkflowEvidence")?;
            let inventory = (wire.inventory_original, wire.inventory_sha256);
            (wire.invocation, wire.report_original, wire.report_observation, wire.terminal, Some(inventory), Some(wire.selected_tests), wire.selected_population_complete, wire.recorded_manager_settlement, wire.recorded_native_lifetime_pending)
        }
        _ => return Err(EvidenceError::Invalid("selected message is not an independently captured evidence root".into())),
    };
    let id = CliInvocationId::from_wire(required(invocation.into_option(), "invocation")?, "invocation")?;
    let observation: crate::test_workflow::execute::ReportObservation = decode_root(required(observation.into_option(), "report_observation")?, "report_observation")?;
    let terminal: crate::test_workflow::execute::TerminalRecord = decode_root(required(terminal.into_option(), "terminal")?, "terminal")?;
    if id != observation.invocation { return Err(EvidenceError::Invalid("projected invocation differs from original report".into())); }
    terminal.report.validate_content(original.len() as u64, &digest(&original))?;
    let parsed: crate::test_workflow::execute::ReportObservation = serde_json::from_slice(&original)?;
    parsed.validate_against(&terminal)?;
    // Compare complete passive source data, including all failure tree fields. JSON is
    // the native report format, not a wire carrier or alternative constraint language.
    if serde_json::to_vec(&parsed)? != serde_json::to_vec(&observation)? {
        return Err(EvidenceError::Invalid("projected report facts differ from exact original report".into()));
    }
    let mut result = Readback { invocation: id, report_sha256: terminal.report.sha256, report_version: parsed.version, inventory_sha256: None, selected_tests: None, selected_population_complete: None,
        recorded_manager_settlement: settlement.map(|wire| crate::envelope::Settlement::from_wire(wire, "recorded_manager_settlement")).transpose()?, recorded_native_lifetime_pending: pending,
        limitations: vec!["The included terminal commits report content; matching portable bytes alone does not authenticate an original native producer or grant execution, rerun or current settlement authority.".into(), "Manager facts are source-owner recorded-at-read observations, separate from the helper's original report commitment; no original manager-result bytes are included.".into(), "Original raw streams, XML artifacts and executable custody remain with native retention owners.".into()] };
    if let Some((original, sha)) = inventory {
        parsed.validate_inventory(&original)?;
        if digest(&original) != sha { return Err(EvidenceError::Invalid("inventory digest differs from original bytes".into())); }
        let inventory: crate::discovery::Inventory = serde_json::from_slice(&original)?;
        if !inventory.capture_complete() { return Err(EvidenceError::Invalid("original inventory capture is incomplete".into())); }
        let selected = selected.unwrap_or_default().into_iter().map(|wire| decode_root(wire, "selected_tests")).collect::<Result<Vec<crate::discovery::TestIdentity>, _>>()?;
        let expected = inventory.selected_tests();
        let expected_complete = (!parsed.action.is_compiler()).then(|| inventory.selected_population_complete());
        parsed.validate_selected_population(&inventory)?;
        if selected != expected || complete != expected_complete {
            return Err(EvidenceError::Invalid("selected population differs from original native inventory".into()));
        }
        result.inventory_sha256 = Some(sha); result.selected_tests = Some(expected.len()); result.selected_population_complete = complete;
    }
    Ok(result)
}
fn digest(bytes: &[u8]) -> String { Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect() }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{envelope::{Envelope, ErrorRecord, RedactionPolicy}, managed::result::{ManagedSummary, WorkflowSummary}, test_workflow::execute::{Action, Outcome, ReportObservation, ReportReference, TerminalRecord}};
    use contracts::pb::pse::developer::evidence::v1 as pb;
    use buffa_descriptor::generated::descriptor::FileDescriptorSet;

    fn original() -> pb::CapturedWorkflowReport {
        let id = CliInvocationId::new();
        let failure = ErrorRecord::from_typed(&HostError::Ownership("original prerequisite unavailable".into()), &RedactionPolicy::default());
        let report = serde_json::to_vec(&serde_json::json!({"version":1,"invocation":id,"action":"test","outcome":"incomplete","required_capture_complete":false,"failures":[failure],"observations":[]})).unwrap();
        let observation: ReportObservation = serde_json::from_slice(&report).unwrap();
        let terminal = TerminalRecord { protocol: "pse.test-workflow".into(), version: 1, invocation: id.clone(), action: Action::Test, outcome: Outcome::Incomplete,
            report: ReportReference { path: "report.json".into(), sha256: digest(&report), byte_length: report.len() as u64 }, required_capture_complete: false };
        let mut original = Envelope::response("test", "original incomplete capture", ManagedSummary { bundle: None, workflow: Some(WorkflowSummary { terminal, report: "report.json".into() }), native_lifetime_pending: None, child_exit: Some(125), stdout: None, stderr: None, borrowed_reservation: false, service_result: None });
        original.invocation_id = id.clone();
        let captured = crate::test_inventory::CapturedReport { id, report, observation, original, parent: "unused-native-owner-fixture".into() };
        contracts::encode_captured_workflow_report(&captured).unwrap()
    }
    fn descriptors() -> FileDescriptorSet { FileDescriptorSet::decode_from_slice(contracts::DESCRIPTORS).unwrap() }
    fn read(wire: &pb::CapturedWorkflowReport) -> Result<Readback, EvidenceError> { inspect(&descriptors(), "pse.developer.evidence.v1.CapturedWorkflowReport", &wire.encode_to_vec()) }

    #[test]
    fn independent_legacy_report_preserves_original_failure_and_unavailable_facts() {
        let wire = original();
        let result = read(&wire).unwrap();
        assert_eq!(result.report_version, 1);
        assert!(result.inventory_sha256.is_none());
        assert!(result.selected_population_complete.is_none());
        assert!(result.recorded_native_lifetime_pending.is_none());
        assert!(result.selected_tests.is_none());
        assert!(String::from_utf8(wire.report_original.clone()).unwrap().contains("original prerequisite unavailable"));
    }
    #[test]
    fn independent_report_refuses_changed_original_bytes_and_passive_facts() {
        let wire = original();
        let mut changed = wire.clone();
        changed.report_original[0] = b'[';
        assert!(read(&changed).is_err());
        let mut changed = wire.clone();
        changed.report_observation.as_option_mut().unwrap().failures.clear();
        assert!(read(&changed).is_err());
        let mut changed = wire;
        changed.report_original.clear();
        assert!(read(&changed).is_err());
    }
    #[test]
    fn independent_report_requires_source_coverage_and_valid_wire() {
        let wire = original();
        let mut descriptors = descriptors();
        let file = descriptors.file.iter_mut().find(|file| file.name.as_deref() == Some("developer_evidence.proto")).unwrap();
        file.message_type.iter_mut().find(|message| message.name.as_deref() == Some("CapturedWorkflowReport")).unwrap().options = Default::default();
        assert!(inspect(&descriptors, "pse.developer.evidence.v1.CapturedWorkflowReport", &wire.encode_to_vec()).is_err());
        assert!(matches!(inspect(&self::descriptors(), "pse.developer.evidence.v1.CapturedWorkflowReport", &[0x0a, 0xff]), Err(EvidenceError::Decode(_))));
        let mut unknown = wire.encode_to_vec();
        unknown.extend_from_slice(&[0xf8, 0x7f, 0x01]);
        assert!(matches!(inspect(&self::descriptors(), "pse.developer.evidence.v1.CapturedWorkflowReport", &unknown), Err(EvidenceError::Projection(_))));
    }
    #[test]
    fn independent_report_binds_compiled_wire_tags_kinds_and_nested_shapes() {
        use buffa_descriptor::generated::descriptor::field_descriptor_proto::Type;
        let wire = original().encode_to_vec();
        for change in 0..3 {
            let mut descriptors = descriptors();
            let file = descriptors.file.iter_mut().find(|file| file.name.as_deref() == Some("developer_evidence.proto")).unwrap();
            let root = file.message_type.iter_mut().find(|message| message.name.as_deref() == Some("CapturedWorkflowReport")).unwrap();
            match change {
                0 => root.field.iter_mut().find(|field| field.name.as_deref() == Some("report_original")).unwrap().number = Some(99),
                1 => root.field.iter_mut().find(|field| field.name.as_deref() == Some("report_original")).unwrap().r#type = Some(Type::TYPE_INT32.into()),
                _ => root.field.iter_mut().find(|field| field.name.as_deref() == Some("report_observation")).unwrap().type_name = Some(".pse.developer.evidence.v1.WorkflowTerminalRecord".into()),
            }
            let metadata = pse_contract_proto::check_descriptors(&descriptors, "developer_evidence.proto", &contracts::Policy, &[]).unwrap();
            match metadata.disposition {
                pse_contract::metadata::Disposition::Compatible => assert!(matches!(inspect(&descriptors, "pse.developer.evidence.v1.CapturedWorkflowReport", &wire), Err(EvidenceError::Invalid(reason)) if reason.contains("complete compiled descriptor content")), "surviving metadata never selects a different compiled wire world: drift {change}"),
                _ => assert!(matches!(inspect(&descriptors, "pse.developer.evidence.v1.CapturedWorkflowReport", &wire), Err(EvidenceError::Interpretation { assessment }) if assessment.disposition == metadata.disposition), "a stronger shared checker may refuse earlier without losing its typed disposition"),
            }
        }
    }
    #[test]
    fn independent_report_refuses_missing_import_and_foreign_family() {
        let wire = original().encode_to_vec();
        let mut missing = descriptors();
        missing.file.retain(|file| file.name.as_deref() != Some("google/protobuf/descriptor.proto"));
        assert!(matches!(inspect(&missing, "pse.developer.evidence.v1.CapturedWorkflowReport", &wire), Err(EvidenceError::Metadata(_))));
        let foreign = FileDescriptorSet::decode_from_slice(crate::code_intelligence_contracts::DESCRIPTORS).unwrap();
        assert!(matches!(inspect(&foreign, "pse.developer.evidence.v1.CapturedWorkflowReport", &wire), Err(EvidenceError::Metadata(_))));
    }
    #[test]
    fn independent_report_refuses_oversize_without_partial_readback() {
        let oversized = vec![0; MAX_INPUT_BYTES + 1];
        assert!(matches!(inspect(&descriptors(), "pse.developer.evidence.v1.CapturedWorkflowReport", &oversized), Err(EvidenceError::InputLimit { observed_at_least, maximum }) if observed_at_least == MAX_INPUT_BYTES + 1 && maximum == MAX_INPUT_BYTES));
    }

    fn raw_preview(bytes: &[u8], descriptors: &FileDescriptorSet) -> Result<crate::schema::payload::Report, crate::schema::SchemaError> {
        let directory = tempfile::tempdir().unwrap();
        let descriptor = directory.path().join("original-descriptor.bin");
        let input = directory.path().join("original-payload.bin");
        std::fs::write(&descriptor, descriptors.encode_to_vec()).unwrap();
        std::fs::write(&input, bytes).unwrap();
        crate::schema::payload::inspect(&crate::schema::payload::Args { descriptor, input, message: "pse.developer.evidence.v1.CapturedWorkflowReport".into(), encoding: crate::schema::payload::Encoding::Binary, depth: 1, limit: 10, original: true })
    }
    #[test]
    fn raw_payload_preserves_corrupt_original_without_constructing_native_facts() {
        let mut wire = original();
        wire.report_original[0] = b'[';
        assert!(matches!(read(&wire), Err(EvidenceError::Workflow(_))), "the direct native integrity reader remains strict");
        let bytes = wire.encode_to_vec();
        let report = raw_preview(&bytes, &descriptors()).unwrap();
        assert_eq!(report.validation.state, "passed", "local wire validity is separate from original content integrity");
        assert!(report.developer_evidence.is_none());
        assert!(report.developer_evidence_refusal.is_some());
        assert!(matches!(report.preview, crate::schema::payload::Preview::Message { .. }));
        use base64::Engine as _;
        assert_eq!(base64::engine::general_purpose::STANDARD.decode(report.original_base64.unwrap()).unwrap(), bytes);
    }
    #[test]
    fn raw_payload_keeps_unavailable_policy_in_its_original_acquired_world() {
        use buffa_descriptor::{DescriptorPool, DynamicMessage, ReflectMessageMut, Value};
        let mut descriptors = descriptors();
        let pool = std::sync::Arc::new(DescriptorPool::new(descriptors.clone()).unwrap());
        let root = descriptors.file.iter_mut().find(|file| file.name.as_deref() == Some("developer_evidence.proto")).unwrap().message_type.iter_mut().find(|message| message.name.as_deref() == Some("CapturedWorkflowReport")).unwrap();
        let mut options = DynamicMessage::from_options(std::sync::Arc::clone(&pool), root.options.as_option().unwrap()).unwrap();
        let extension = pool.extension_by_name("pse.contract.v1.message").unwrap();
        let Some(Value::Message(mut metadata)) = options.field_by_number(extension.field().number()).cloned() else { panic!("actual typed source option"); };
        let version = pool.message(metadata.message_index()).field_by_name("metadata_version").unwrap();
        metadata.try_set(version, Value::U32(99)).unwrap();
        options.try_set(extension.field(), Value::Message(metadata)).unwrap();
        root.options = buffa::MessageField::some(buffa_descriptor::generated::descriptor::MessageOptions::decode_from_slice(&options.try_encode_to_vec().unwrap()).unwrap());
        let bytes = original().encode_to_vec();
        assert!(matches!(inspect(&descriptors, "pse.developer.evidence.v1.CapturedWorkflowReport", &bytes), Err(EvidenceError::Interpretation { assessment }) if assessment.disposition == pse_contract::metadata::Disposition::Unknown));
        let report = raw_preview(&bytes, &descriptors).unwrap();
        assert_eq!(report.representation, "acquired_dynamic");
        assert_eq!(report.validation.state, "not_evaluated");
        assert!(report.developer_evidence.is_none());
        let refusal = report.developer_evidence_refusal.unwrap();
        assert!(refusal.message.contains("CapturedWorkflowReport"));
        assert!(refusal.message.contains("developer_evidence.proto"));
        assert!(matches!(report.preview, crate::schema::payload::Preview::Message { .. }));
    }
    #[test]
    fn raw_payload_file_read_stops_at_the_evidence_family_limit() {
        let directory = tempfile::tempdir().unwrap();
        let descriptor = directory.path().join("descriptor.bin");
        let input = directory.path().join("oversized.bin");
        std::fs::write(&descriptor, contracts::DESCRIPTORS).unwrap();
        std::fs::File::create(&input).unwrap().set_len((MAX_INPUT_BYTES * 4) as u64).unwrap();
        let result = crate::schema::payload::inspect(&crate::schema::payload::Args { descriptor, input, message: "pse.developer.evidence.v1.CapturedWorkflowReport".into(), encoding: crate::schema::payload::Encoding::Binary, depth: 1, limit: 10, original: false });
        assert!(matches!(result, Err(crate::schema::SchemaError::DeveloperEvidence(error)) if matches!(*error, EvidenceError::InputLimit { observed_at_least, maximum } if observed_at_least == MAX_INPUT_BYTES + 1 && maximum == MAX_INPUT_BYTES)));
    }
}
