// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original-world metadata comparison using the shared pure checker and native policies.
use super::{Bundle, SchemaError};
use pse_contract::metadata;
use pse_contract_proto::{DescriptorInput, DescriptorPolicy, DescriptorWorld};
use schemars::JsonSchema;
use serde::Serialize;

const RECORDED_RESULT_FILE: &str = "recorded_result.proto";
#[path = "../../generated/recorded-result-policy/native-policy.rs"]
#[rustfmt::skip]
#[allow(unreachable_pub, reason = "generated policy constants are private to this comparator")]
mod recorded_result_policy;

struct RecordedResultPolicy;
impl metadata::InterpretationPolicy for RecordedResultPolicy {
    fn name(&self) -> &str {
        "pse.engineering.v1/recorded-result"
    }
    fn supports_interpretation(&self, subject: &str, version: u32) -> bool {
        recorded_result_policy::INTERPRETATIONS.contains(&(subject, version))
    }
}
impl DescriptorPolicy for RecordedResultPolicy {
    fn native_selection(&self) -> Option<pse_contract_proto::NativeSelection<'_>> {
        Some(recorded_result_policy::NATIVE_SELECTION)
    }
}

/// An explicitly selected available native policy, never supplied by acquired options.
#[derive(Clone, Copy, Debug, Default, clap::ValueEnum)]
pub enum Policy {
    /// Understand core metadata only; native source coverage remains unavailable.
    #[default]
    Core,
    /// Current independent-host developer evidence source policy and generated coverage.
    DeveloperEvidence,
    /// Original recorded-result observation metadata for the explicitly selected result file.
    RecordedResult,
}
/// A conclusion under the selected policy, independent of wire compatibility.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    /// Every selected fact was understood and accepted by this policy.
    Compatible,
    /// A selected fact is invalid or its understood meaning changed incompatibly.
    Incompatible,
    /// Required coverage or interpretation is unavailable.
    Unknown,
}
impl From<metadata::Disposition> for Disposition {
    fn from(value: metadata::Disposition) -> Self {
        match value {
            metadata::Disposition::Compatible => Self::Compatible,
            metadata::Disposition::Incompatible => Self::Incompatible,
            metadata::Disposition::Unknown => Self::Unknown,
        }
    }
}
/// One original attributed checker finding.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Finding {
    /// Original file within its acquired world.
    pub file: String,
    /// Exact selected metadata subject.
    pub subject: String,
    /// Checker conclusion for this fact.
    pub disposition: Disposition,
    /// Original checker explanation.
    pub reason: String,
}
/// Semantic assessment with deliberately bounded native ownership and interpretation.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Assessment {
    /// Explicit native policy identity.
    pub policy: String,
    /// Incompatibility dominates unknown; wire results never affect this conclusion.
    pub disposition: Disposition,
    /// Selected original files; imports supply context, not additional native selection.
    pub selected_files: Vec<String>,
    /// Whether this policy supplies source-owned native coverage for the selected files.
    pub native_coverage_available: bool,
    /// Attributed non-compatible facts from the shared checker.
    pub findings: Vec<Finding>,
    /// Unselected or unavailable meaning and authority.
    pub limitations: Vec<String>,
}

/// Compare selected files under their original acquired pools and an explicit native policy.
/// No normalization callback or policy is loaded from the acquired schema.
/// # Errors
/// A file selector is absent from both worlds or the shared checker rejects malformed metadata.
pub fn assess(
    current: &Bundle,
    baseline: &Bundle,
    selected_files: &[String],
    policy: Policy,
) -> Result<Assessment, SchemaError> {
    match policy {
        Policy::Core => compare_with_policy(
            current,
            baseline,
            selected_files,
            &metadata::CorePolicy {
                interpretations: &[],
            },
            false,
        ),
        Policy::DeveloperEvidence => {
            if selected_files
                .iter()
                .any(|file| file != "developer_evidence.proto")
            {
                return Err(SchemaError::Invalid("developer-evidence policy covers only developer_evidence.proto; imports are context, not native source coverage".into()));
            }
            compare_with_policy(
                current,
                baseline,
                selected_files,
                &crate::developer_evidence_contracts::Policy,
                true,
            )
        }
        Policy::RecordedResult => {
            if selected_files
                .iter()
                .any(|file| file != RECORDED_RESULT_FILE)
            {
                return Err(SchemaError::Invalid(
                    "recorded-result policy covers only recorded_result.proto; imports are context, not native source coverage".into(),
                ));
            }
            compare_with_policy(
                current,
                baseline,
                selected_files,
                &RecordedResultPolicy,
                true,
            )
        }
    }
}

fn compare_with_policy(
    current: &Bundle,
    baseline: &Bundle,
    selected_files: &[String],
    policy: &impl DescriptorPolicy,
    native_coverage: bool,
) -> Result<Assessment, SchemaError> {
    fn original(bundle: &Bundle) -> DescriptorWorld<'_> {
        // Bundle's navigation pool omits redundant identical file occurrences.
        // The semantic checker still sees that original acquisition population.
        if bundle.set.file.len() == bundle.pool.files().len() {
            DescriptorWorld::Acquired(&bundle.pool)
        } else {
            DescriptorWorld::Source(&bundle.set)
        }
    }
    let files = selected_files
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let mut result = metadata::Assessment {
        policy: policy.name().into(),
        disposition: metadata::Disposition::Compatible,
        findings: vec![],
    };
    let mut originals_available = !files.is_empty();
    if files.is_empty() {
        result.record("", "file selection", metadata::Disposition::Unknown,
            "no original metadata files selected; use --semantic-file for explicit semantic coverage");
    }
    for file in &files {
        let before = baseline
            .set
            .file
            .iter()
            .any(|item| item.name.as_ref() == Some(file));
        let after = current
            .set
            .file
            .iter()
            .any(|item| item.name.as_ref() == Some(file));
        if !before && !after {
            return Err(SchemaError::Invalid(format!(
                "semantic file {file} is absent from both acquired worlds"
            )));
        }
        if !before || !after {
            originals_available = false;
            result.record(file, "original world", metadata::Disposition::Unknown,
                "selected file is absent from one original world; structural addition/removal does not supply missing owner interpretation");
            continue;
        }
        let assessed = pse_contract_proto::compare_descriptors(
            DescriptorInput {
                descriptors: original(baseline),
                selected_file: file,
                policy,
            },
            DescriptorInput {
                descriptors: original(current),
                selected_file: file,
                policy,
            },
            policy,
            &[],
        )?;
        for finding in assessed.findings {
            result.record(
                &finding.file,
                &finding.subject,
                finding.disposition,
                finding.reason,
            );
        }
    }
    Ok(Assessment {
        policy: result.policy, disposition: result.disposition.into(),
        native_coverage_available: native_coverage && originals_available,
        selected_files: files.into_iter().collect(),
        findings: result.findings.into_iter().map(|finding| Finding {
            file: finding.file, subject: finding.subject,
            disposition: finding.disposition.into(), reason: finding.reason,
        }).collect(),
        limitations: vec![
            "Only explicitly selected files and core native metadata are assessed; imported options and domain facets are not implicitly selected.".into(),
            "Each original pool supplies its own option interpretation. Unsupported versions/owners remain unknown; no current-world normalization substitutes for unavailable original meaning.".into(),
            "Compatibility under this policy is not wire compatibility, payload validation, native adoption, scientific permission or current readiness.".into(),
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use buffa::Message as _;
    use buffa_descriptor::generated::descriptor::FileDescriptorSet;
    use pse_contract::boundary_projection::{Direction, Scalar, TypeRef};
    const FILE: &str = "developer_evidence.proto";
    fn source() -> FileDescriptorSet {
        FileDescriptorSet::decode_from_slice(crate::developer_evidence_contracts::DESCRIPTORS)
            .unwrap()
    }
    fn bundle(set: &FileDescriptorSet) -> Bundle {
        Bundle::from_bytes("original.bin".into(), set.encode_to_vec()).unwrap()
    }
    fn assessment(old: &FileDescriptorSet, new: &FileDescriptorSet, policy: Policy) -> Assessment {
        assess(&bundle(new), &bundle(old), &[FILE.into()], policy).unwrap()
    }
    fn recorded_result_world(name: &str) -> Bundle {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pse-runtime/src/generated/recorded-result")
            .join(name);
        Bundle::load(&path).unwrap()
    }
    #[test]
    fn recorded_result_semantics_uses_explicit_file_and_original_worlds() {
        let runtime = recorded_result_world("descriptor.bin");
        let source = recorded_result_world("descriptor-source.bin");
        let file = [RECORDED_RESULT_FILE.into()];
        let known = assess(&runtime, &source, &file, Policy::RecordedResult).unwrap();
        assert_eq!(known.disposition, Disposition::Compatible, "{known:?}");
        assert!(known.native_coverage_available);
        assert_eq!(known.policy, "pse.engineering.v1/recorded-result");
        for unknown in [
            assess(&runtime, &source, &file, Policy::Core).unwrap(),
            assess(&runtime, &source, &[], Policy::RecordedResult).unwrap(),
        ] {
            assert_eq!(unknown.disposition, Disposition::Unknown, "{unknown:?}");
            assert!(!unknown.native_coverage_available);
        }
        let imported = [pse_contract_proto::CORE_IMPORT.into()];
        assert!(assess(&runtime, &source, &imported, Policy::RecordedResult).is_err());
        let foreign = [FILE.into()];
        assert!(assess(&runtime, &source, &foreign, Policy::RecordedResult).is_err());
        let mut absent = source.set.clone();
        absent
            .file
            .retain(|file| file.name.as_deref() != Some(RECORDED_RESULT_FILE));
        let missing = assess(&runtime, &bundle(&absent), &file, Policy::RecordedResult).unwrap();
        assert_eq!(missing.disposition, Disposition::Unknown, "{missing:?}");
        assert!(!missing.native_coverage_available);

        let newer =
            with_unknown_version(&source.set, RECORDED_RESULT_FILE, "RecordedResultSelection");
        let unknown = assess(&runtime, &bundle(&newer), &file, Policy::RecordedResult).unwrap();
        assert_eq!(unknown.disposition, Disposition::Unknown, "{unknown:?}");
        assert!(unknown.native_coverage_available);
        assert!(
            unknown
                .findings
                .iter()
                .any(|finding| finding.reason.contains("interpretation"))
        );
    }

    #[test]
    fn recorded_result_semantics_refuses_wire_identical_coordinate_metadata_drift() {
        let runtime = recorded_result_world("descriptor.bin");
        let encoder = pse_contract_proto::OptionEncoder::new(&runtime.set.file).unwrap();
        let mut changed = runtime.set.clone();
        let field = changed
            .file
            .iter_mut()
            .find(|file| file.name.as_deref() == Some(RECORDED_RESULT_FILE))
            .unwrap()
            .message_type
            .iter_mut()
            .find(|message| message.name.as_deref() == Some("RecordedResultSelection"))
            .unwrap()
            .field
            .iter_mut()
            .find(|field| field.name.as_deref() == Some("row_start"))
            .unwrap();
        let mut original_wire = field.clone();
        original_wire.options = Default::default();
        let mut options = Default::default();
        encoder
            .field_correspondence(
                &mut options,
                "row_start",
                Direction::Output,
                &TypeRef::Scalar(Scalar::I64),
                &[],
                "pse.engineering.v1",
            )
            .unwrap();
        field.options = options.into();
        let mut changed_wire = field.clone();
        changed_wire.options = Default::default();
        assert_eq!(changed_wire, original_wire);
        let result = assess(
            &bundle(&changed),
            &runtime,
            &[RECORDED_RESULT_FILE.into()],
            Policy::RecordedResult,
        )
        .unwrap();
        assert_eq!(result.disposition, Disposition::Incompatible, "{result:?}");
        assert!(
            result
                .findings
                .iter()
                .any(|finding| finding.subject.ends_with(".row_start"))
        );
    }

    #[test]
    fn schema_semantics_available_source_policy_and_unknown_coverage_are_distinct() {
        let original = source();
        let known = assessment(&original, &original, Policy::DeveloperEvidence);
        assert_eq!(known.disposition, Disposition::Compatible, "{known:?}");
        assert!(known.native_coverage_available);
        assert_eq!(known.policy, "pse.developer.evidence.v1/native");
        let unknown = assessment(&original, &original, Policy::Core);
        assert_eq!(unknown.disposition, Disposition::Unknown);
        assert!(!unknown.native_coverage_available);
        assert!(
            unknown
                .findings
                .iter()
                .any(|item| item.subject == "native selection")
        );
        let unselected = assess(
            &bundle(&original),
            &bundle(&original),
            &[],
            Policy::DeveloperEvidence,
        )
        .unwrap();
        assert_eq!(unselected.disposition, Disposition::Unknown);
        assert!(!unselected.native_coverage_available);
    }
    #[test]
    fn schema_semantics_wire_identical_native_unsigned_and_ordered_changes_refuse() {
        let original = source();
        let encoder = pse_contract_proto::OptionEncoder::new(&original.file).unwrap();
        // Literal native meanings replace only option metadata; wire tags/types stay unchanged.
        for (owner, field_name, ty) in [
            (
                "ContentIdentity",
                "byte_length",
                TypeRef::Scalar(Scalar::I64),
            ),
            (
                "WorkflowReportObservation",
                "failures",
                TypeRef::Set(Box::new(TypeRef::Named("ErrorRecord".into()))),
            ),
        ] {
            let mut changed = original.clone();
            let field = changed
                .file
                .iter_mut()
                .find(|item| item.name.as_deref() == Some(FILE))
                .unwrap()
                .message_type
                .iter_mut()
                .find(|item| item.name.as_deref() == Some(owner))
                .unwrap()
                .field
                .iter_mut()
                .find(|item| item.name.as_deref() == Some(field_name))
                .unwrap();
            let mut wire_before = field.clone();
            wire_before.options = Default::default();
            let mut options = Default::default();
            encoder
                .field_correspondence(
                    &mut options,
                    field_name,
                    Direction::Output,
                    &ty,
                    &[],
                    "pse.developer.evidence.v1",
                )
                .unwrap();
            field.options = options.into();
            let mut wire_after = field.clone();
            wire_after.options = Default::default();
            assert_eq!(wire_before, wire_after);
            let report = assessment(&original, &changed, Policy::DeveloperEvidence);
            assert_eq!(
                report.disposition,
                Disposition::Incompatible,
                "{owner}: {report:?}"
            );
            assert!(
                report
                    .findings
                    .iter()
                    .any(|item| item.subject.contains(field_name))
            );
        }
    }
    #[test]
    fn schema_semantics_unavailable_original_world_and_foreign_selection_are_explicit() {
        let original = source();
        let mut foreign = original.clone();
        foreign
            .file
            .retain(|file| file.name.as_deref() != Some(FILE));
        let report = assessment(&foreign, &original, Policy::DeveloperEvidence);
        assert_eq!(report.disposition, Disposition::Unknown);
        assert!(!report.native_coverage_available);
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.subject == "original world")
        );
        assert!(
            assess(
                &bundle(&original),
                &bundle(&original),
                &["absent.proto".into()],
                Policy::Core
            )
            .is_err()
        );
        assert!(
            assess(
                &bundle(&original),
                &bundle(&original),
                &[pse_contract_proto::CORE_IMPORT.into()],
                Policy::DeveloperEvidence
            )
            .is_err()
        );
        let mut unrecognized = original.clone();
        // Missing original metadata import is unavailable, not wire-compatible meaning.
        unrecognized
            .file
            .retain(|file| file.name.as_deref() != Some(pse_contract_proto::CORE_IMPORT));
        // Bundle acquisition correctly refuses an incomplete import closure before comparison.
        assert!(Bundle::from_bytes("incomplete.bin".into(), unrecognized.encode_to_vec()).is_err());
    }
    fn with_unknown_version(
        original: &FileDescriptorSet,
        file: &str,
        owner_name: &str,
    ) -> FileDescriptorSet {
        use buffa_descriptor::{ReflectMessageMut as _, Value};
        let encoder = pse_contract_proto::OptionEncoder::new(&original.file).unwrap();
        let mut changed = original.clone();
        let owner = changed
            .file
            .iter_mut()
            .find(|candidate| candidate.name.as_deref() == Some(file))
            .unwrap()
            .message_type
            .iter_mut()
            .find(|owner| owner.name.as_deref() == Some(owner_name))
            .unwrap();
        let mut metadata = encoder
            .decode_extension(
                owner.options.as_option().unwrap(),
                "pse.contract.v1.message",
            )
            .unwrap()
            .unwrap();
        let pool = std::sync::Arc::clone(metadata.pool());
        let descriptor = pool.message(
            pool.message_index("pse.contract.v1.MessageMetadata")
                .unwrap(),
        );
        metadata
            .try_set(
                descriptor.field_by_name("interpretation_version").unwrap(),
                Value::U32(999),
            )
            .unwrap();
        let mut options = buffa_descriptor::generated::descriptor::MessageOptions::default();
        encoder
            .attach(&mut options, "pse.contract.v1.message", metadata)
            .unwrap();
        owner.options = options.into();
        changed
    }

    #[test]
    fn schema_semantics_original_interpretation_unavailable_is_unknown_with_native_coverage() {
        let original = source();
        let changed = with_unknown_version(&original, FILE, "ContentIdentity");
        let report = assessment(&original, &changed, Policy::DeveloperEvidence);
        assert_eq!(report.disposition, Disposition::Unknown, "{report:?}");
        assert!(report.native_coverage_available);
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.reason.contains("interpretation"))
        );
    }
}
