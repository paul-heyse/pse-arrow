// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Schema-addressed correction, derived from original validator structures.
use super::RedactionPolicy;
use protovalidate_buffa::{ValidationError, proto};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pse_contract::projection_enum! {
/// Validator execution and ordinary rejected constraints remain distinct.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValidationOutcome {
    projection: { name: "ValidationOutcome", rust_path: crate::envelope::ValidationOutcome, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Ordinary constraints rejected the request.
    Violations => 1;
    /// A malformed or unsupported rule failed compilation.
    CompileDefect => 2;
    /// The selected validator failed to evaluate.
    EvaluationDefect => 3;
    /// An error with neither defects nor violations is a validator defect.
    InvalidEmptyError => 4;
}
}
pse_contract::projection_enum! {
/// Coverage belongs to the origin; a receiver's visible count cannot establish it.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValidationCoverage {
    projection: { name: "ValidationCoverage", rust_path: crate::envelope::ValidationCoverage, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// No originating completeness assertion is available.
    Unknown => 1;
    /// Every original violation is present.
    Complete => 2;
    /// At least one original whole violation was omitted.
    Truncated => 3;
    /// Validator defects do not constitute a violation population.
    NotApplicable => 4;
}
}
pse_contract::projection_enum! {
/// Original disclosure policy; missing keys/messages cannot be reconstructed.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValidationExposure {
    projection: { name: "ValidationExposure", rust_path: crate::envelope::ValidationExposure, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// The original disclosure policy is unavailable.
    Unknown => 1;
    /// The origin intentionally omitted input-bearing detail.
    OmittedByPolicy => 2;
    /// Only the received representation is being described.
    AsReceived => 3;
}
}
pse_contract::projection_struct! {
/// Origin-owned facts about canonical violation publication.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ValidationDisclosure {
    projection: { name: "ValidationDisclosure", rust_path: crate::envelope::ValidationDisclosure, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Fully qualified schema message when the origin knows it.
    pub message_type: Option<String> => { tag: 1, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Exact compiled descriptor world, independently of payload validity.
    pub descriptor_sha256: Option<String> => { tag: 2, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Native validator disposition.
    pub outcome: ValidationOutcome => { tag: 3, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Count at the originating selected validator, never inferred by receivers.
    pub source_violation_count: Option<u64> => { tag: 4, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Whole canonical violations actually emitted by that origin.
    pub emitted_violation_count: Option<u64> => { tag: 5, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Origin-established whole-record coverage.
    pub coverage: ValidationCoverage => { tag: 6, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Policy applied to input-bearing map selectors.
    pub selectors: ValidationExposure => { tag: 7, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Policy applied to input-bearing diagnostic messages.
    pub messages: ValidationExposure => { tag: 8, direction: pse_contract::BoundaryDirection::Output, rules: [] };
}
}
pse_contract::projection_enum! {
/// Exact index, or an explicitly undisclosed map selector.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ValidationSelector {
    projection: { name: "ValidationSelector", rust_path: crate::envelope::ValidationSelector, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Exact decimal text avoids integer rounding by JSON consumers.
    Index(String) => 1;
    /// A map location is unavailable under the publication policy.
    RedactedMapKey => 2;
}
}
pse_contract::projection_struct! {
/// Schema path element, preserving number/name and collection structure.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ValidationPathElement {
    projection: { name: "ValidationPathElement", rust_path: crate::envelope::ValidationPathElement, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Authored field number, when supplied by the validator.
    pub field_number: Option<i32> => { tag: 1, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Authored field name; absent names remain unavailable.
    pub field_name: Option<String> => { tag: 2, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original schema scalar/message category.
    pub field_type: Option<String> => { tag: 3, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original map key category, independently of key disclosure.
    pub key_type: Option<String> => { tag: 4, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original map value category.
    pub value_type: Option<String> => { tag: 5, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Repeated position or explicitly undisclosed map entry.
    pub selector: Option<ValidationSelector> => { tag: 6, direction: pse_contract::BoundaryDirection::Output, rules: [] };
}
}
pse_contract::projection_struct! {
/// One original supported violation; schema locations are not source byte spans.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ValidationViolation {
    projection: { name: "ValidationViolation", rust_path: crate::envelope::ValidationViolation, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Numbered/indexed path into the original request.
    pub field: Vec<ValidationPathElement> => { tag: 1, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Numbered/indexed path into the authored validation rule.
    pub rule: Vec<ValidationPathElement> => { tag: 2, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Upstream rule identity without interpreting the rendered diagnostic.
    pub rule_id: Option<String> => { tag: 3, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Only the message actually disclosed by the original representation.
    pub message: Option<String> => { tag: 4, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// True distinguishes a map-key rule from a value rule.
    pub for_key: Option<bool> => { tag: 5, direction: pse_contract::BoundaryDirection::Output, rules: [] };
}
}
pse_contract::projection_struct! {
/// Public correction projected from originals, independently of their custody.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ValidationCorrection {
    projection: { name: "ValidationCorrection", rust_path: crate::envelope::ValidationCorrection, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Origin-established disclosure and coverage limits.
    pub disclosure: ValidationDisclosure => { tag: 1, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Only the supported original whole violations present in this representation.
    pub violations: Vec<ValidationViolation> => { tag: 2, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Detail decode/projection defects, never relabelled as user violations.
    pub diagnostics: Vec<String> => { tag: 3, direction: pse_contract::BoundaryDirection::Output, rules: [] };
}
}
impl ValidationDisclosure {
    pub(crate) fn original(
        error: &ValidationError,
        message_type: &str,
        descriptor_sha256: Option<String>,
    ) -> Self {
        let outcome = if error.compile_error.is_some() {
            ValidationOutcome::CompileDefect
        } else if error.runtime_error.is_some() {
            ValidationOutcome::EvaluationDefect
        } else if error.violations.is_empty() {
            ValidationOutcome::InvalidEmptyError
        } else {
            ValidationOutcome::Violations
        };
        Self {
            message_type: Some(message_type.into()),
            descriptor_sha256,
            outcome,
            source_violation_count: Some(error.violations.len() as u64),
            emitted_violation_count: None,
            coverage: if outcome == ValidationOutcome::Violations {
                ValidationCoverage::Unknown
            } else {
                ValidationCoverage::NotApplicable
            },
            selectors: ValidationExposure::OmittedByPolicy,
            messages: ValidationExposure::OmittedByPolicy,
        }
    }
    pub(crate) fn unknown() -> Self {
        Self {
            message_type: None,
            descriptor_sha256: None,
            outcome: ValidationOutcome::Violations,
            source_violation_count: None,
            emitted_violation_count: None,
            coverage: ValidationCoverage::Unknown,
            selectors: ValidationExposure::Unknown,
            messages: ValidationExposure::Unknown,
        }
    }
}
impl ValidationCorrection {
    /// Apply the actual caller's disclosure policy to a copied public correction.
    pub(crate) fn redacted(mut self, policy: &RedactionPolicy) -> Self {
        fn text(value: &mut Option<String>, policy: &RedactionPolicy) {
            if let Some(value) = value {
                *value = policy.redact(value);
            }
        }
        text(&mut self.disclosure.message_type, policy);
        text(&mut self.disclosure.descriptor_sha256, policy);
        for violation in &mut self.violations {
            text(&mut violation.rule_id, policy);
            text(&mut violation.message, policy);
            for element in violation.field.iter_mut().chain(&mut violation.rule) {
                text(&mut element.field_name, policy);
                text(&mut element.field_type, policy);
                text(&mut element.key_type, policy);
                text(&mut element.value_type, policy);
                // Index is a typed integer represented as decimal text, not input-bearing text.
            }
        }
        for diagnostic in &mut self.diagnostics {
            *diagnostic = policy.redact(diagnostic);
        }
        self
    }
    /// Local compiled validation uses upstream canonical conversion, preserving defect identity.
    pub(crate) fn local(error: &ValidationError, message_type: &str) -> Self {
        let mut disclosure = ValidationDisclosure::original(error, message_type, None);
        disclosure.messages = ValidationExposure::AsReceived;
        let proto = error.to_proto();
        if disclosure.outcome == ValidationOutcome::Violations {
            disclosure.emitted_violation_count = disclosure.source_violation_count;
            disclosure.coverage = ValidationCoverage::Complete;
        }
        let policy = RedactionPolicy::from_environment();
        let diagnostics = error
            .compile_error
            .iter()
            .chain(error.runtime_error.iter())
            .map(|text| policy.redact(text))
            .collect();
        Self {
            disclosure,
            violations: proto
                .violations
                .iter()
                .map(|value| violation(value, &policy))
                .collect(),
            diagnostics,
        }
    }
    /// Decode retained canonical details; originals and unavailable coverage stay independent.
    pub(crate) fn rpc(
        details: &[connectrpc::ErrorDetail],
        disclosure: Option<ValidationDisclosure>,
    ) -> Option<Self> {
        let known = disclosure.is_some();
        let mut result = Self {
            disclosure: disclosure.unwrap_or_else(ValidationDisclosure::unknown),
            violations: Vec::new(),
            diagnostics: Vec::new(),
        };
        let policy = RedactionPolicy::from_environment();
        let mut found = known;
        for detail in details {
            match protovalidate_buffa::decode_violations(detail) {
                Ok(Some(values)) => {
                    found = true;
                    result.violations.extend(
                        values
                            .violations
                            .iter()
                            .map(|value| violation(value, &policy)),
                    );
                }
                Ok(None) => {}
                Err(error) => {
                    found = true;
                    result.diagnostics.push(error.to_string());
                }
            }
        }
        if result.disclosure.messages == ValidationExposure::OmittedByPolicy {
            for violation in &mut result.violations {
                violation.message = None;
            }
        }
        if let Some(emitted) = result.disclosure.emitted_violation_count
            && emitted != result.violations.len() as u64
        {
            result.diagnostics.push(
                "origin validation disclosure disagrees with retained canonical detail population"
                    .into(),
            );
        }
        if !result.diagnostics.is_empty() {
            result.disclosure.coverage = ValidationCoverage::Unknown;
        }
        found.then_some(result)
    }
}
fn violation(value: &proto::Violation, policy: &RedactionPolicy) -> ValidationViolation {
    ValidationViolation {
        field: path(value.field.as_option(), policy),
        rule: path(value.rule.as_option(), policy),
        rule_id: value.rule_id.as_deref().map(|text| policy.redact(text)),
        message: value.message.as_deref().map(|text| policy.redact(text)),
        for_key: value.for_key,
    }
}
fn path(value: Option<&proto::FieldPath>, policy: &RedactionPolicy) -> Vec<ValidationPathElement> {
    use proto::field_path_element::Subscript;
    value
        .into_iter()
        .flat_map(|path| &path.elements)
        .map(|element| ValidationPathElement {
            field_number: element.field_number,
            field_name: element
                .field_name
                .as_deref()
                .map(|text| policy.redact(text)),
            field_type: element.field_type.map(|value| format!("{value:?}")),
            key_type: element.key_type.map(|value| format!("{value:?}")),
            value_type: element.value_type.map(|value| format!("{value:?}")),
            selector: element.subscript.as_ref().map(|value| match value {
                Subscript::Index(index) => ValidationSelector::Index(index.to_string()),
                _ => ValidationSelector::RedactedMapKey,
            }),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn error() -> ValidationError {
        use protovalidate_buffa::{FieldPath, FieldPathElement, Subscript, Violation};
        ValidationError {
            violations: vec![Violation {
                field: FieldPath {
                    elements: vec![
                        FieldPathElement {
                            field_number: Some(3),
                            field_name: Some("items".into()),
                            subscript: Some(Subscript::Index(9_007_199_254_740_993)),
                            ..Default::default()
                        },
                        FieldPathElement {
                            field_number: Some(4),
                            field_name: Some("entries".into()),
                            subscript: Some(Subscript::StringKey("private-map-key".into())),
                            ..Default::default()
                        },
                    ],
                },
                rule: FieldPath::default(),
                rule_id: "required".into(),
                message: "required input".into(),
                for_key: true,
            }],
            ..Default::default()
        }
    }
    #[test]
    fn original_paths_keep_exact_indices_and_redact_map_selectors() {
        let correction = ValidationCorrection::local(&error(), "example.Request");
        assert_eq!(correction.disclosure.coverage, ValidationCoverage::Complete);
        assert_eq!(
            correction.violations[0].rule_id.as_deref(),
            Some("required")
        );
        assert!(
            matches!(&correction.violations[0].field[0].selector, Some(ValidationSelector::Index(index)) if index=="9007199254740993")
        );
        assert!(matches!(
            correction.violations[0].field[1].selector,
            Some(ValidationSelector::RedactedMapKey)
        ));
        assert!(
            !serde_json::to_string(&correction)
                .unwrap()
                .contains("private-map-key")
        );
    }
    #[test]
    fn decoded_peer_records_do_not_establish_total_or_completeness() {
        let transport = error().into_connect_error();
        let correction = ValidationCorrection::rpc(&transport.details, None).unwrap();
        assert_eq!(correction.disclosure.coverage, ValidationCoverage::Unknown);
        assert_eq!(correction.disclosure.source_violation_count, None);
        assert_eq!(correction.violations.len(), 1);
        assert_eq!(correction.violations[0].message, None);
        assert!(correction.violations[0].field[1].selector.is_none());
    }
    #[test]
    fn claimed_disclosure_cannot_restore_messages_or_hide_mismatched_detail_counts() {
        let original = error();
        let detail =
            connectrpc::ErrorDetail::from_message("buf.validate.Violations", &original.to_proto());
        let mut disclosure = ValidationDisclosure::original(&original, "example.Request", None);
        disclosure.emitted_violation_count = Some(2);
        disclosure.coverage = ValidationCoverage::Complete;
        let correction = ValidationCorrection::rpc(&[detail], Some(disclosure)).unwrap();
        assert_eq!(correction.violations[0].message, None);
        assert_eq!(correction.disclosure.coverage, ValidationCoverage::Unknown);
        assert!(!correction.diagnostics.is_empty());
        assert!(
            !serde_json::to_string(&correction)
                .unwrap()
                .contains("private-map-key")
        );
    }
    #[test]
    fn validator_defects_are_distinct_from_ordinary_constraints() {
        for (compile_error, runtime_error, expected) in [
            (
                Some("bad rule".into()),
                None,
                ValidationOutcome::CompileDefect,
            ),
            (
                None,
                Some("failed CEL".into()),
                ValidationOutcome::EvaluationDefect,
            ),
            (None, None, ValidationOutcome::InvalidEmptyError),
        ] {
            let error = ValidationError {
                compile_error,
                runtime_error,
                ..Default::default()
            };
            let correction = ValidationCorrection::local(&error, "example.Request");
            assert_eq!(correction.disclosure.outcome, expected);
            assert_eq!(
                correction.disclosure.coverage,
                ValidationCoverage::NotApplicable
            );
        }
    }
}
