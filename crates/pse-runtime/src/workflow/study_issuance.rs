// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original study coordinates and the one persisted native creation request interpretation.

use super::{StudyDefinition, WorkflowError, contract};
use pse_ids::{ContentHash, Frame};
use pse_model::{
    document::Version,
    generated::identities::{RunId, StudyId},
    study::OccurrenceKey,
};

pse_contract::projection_struct! {
/// Caller-issued coordinates fixed before any canonical creation or acknowledgement.
/// Decoding this identity grants no authority to resume, cancel or read a study.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IssuedStudyIdentity {
    projection: { name: "IssuedStudyIdentity", rust_path: crate::workflow::IssuedStudyIdentity,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Exact original native study identity, distinct from its scientific content.
    pub study: StudyId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::Required] };
    /// Original caller operation bound by canonical registration.
    pub operation: String => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_model::engineering::OPERATION_ID_RULE] };
    /// Complete original submit envelope, including immutable preparation identity.
    pub submission: ContentHash => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::Required] };
}
}

/// Current native parent request. Earlier positional parent requests retain their old meaning
/// and are refused by current caller-issued recovery rather than being reinterpreted.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StudyCreationRequest {
    /// Explicit interpretation of this original request record.
    pub version: Version<2>,
    /// Original study identity also used by the existing canonical study owner.
    pub study: StudyId,
    /// Deterministic original parent execution identity.
    pub parent: RunId,
    /// Full admitted immutable definition identity, including source, policy and numerical controls.
    pub definition: ContentHash,
    /// Caller submission identity, absent only for ordinary native starts without caller issuance.
    pub issued: Option<IssuedStudyIdentity>,
}

impl IssuedStudyIdentity {
    /// Exact parent execution known before the first acknowledgement.
    #[must_use]
    pub fn parent_run(&self) -> RunId {
        Self::parent_run_for(self.study)
    }

    /// Original parent lookup requires only the caller-issued study coordinate, not a
    /// reconstructed or fabricated submit envelope.
    #[must_use]
    pub fn parent_run_for(study: StudyId) -> RunId {
        parent_run(study)
    }

    pub(super) fn validate(&self) -> Result<(), WorkflowError> {
        use pse_contract::boundary_projection::ValidateProjection as _;
        self.validate_projection().map_err(|error| {
            WorkflowError::Typed(pse_model::diagnostic::DiagnosticCause::new(error))
        })
    }
}

impl StudyCreationRequest {
    pub(super) fn new(
        study: StudyId,
        definition: &StudyDefinition,
        issued: Option<IssuedStudyIdentity>,
    ) -> Result<Self, WorkflowError> {
        if let Some(identity) = &issued {
            identity.validate()?;
            if identity.study != study {
                return Err(contract(
                    "issued study differs from original creation coordinates",
                ));
            }
        }
        Ok(Self {
            version: Version,
            study,
            parent: parent_run(study),
            definition: pse_ids::document::of(Frame::DurableStudyRequestV5, definition)
                .map_err(|error| contract(error.to_string()))?,
            issued,
        })
    }

    /// Decode the exact original current request before trusting caller recovery coordinates.
    /// # Errors
    /// Historical shapes, changed versions, unknown fields or inconsistent parent identities refuse.
    pub fn read(bytes: &[u8]) -> Result<Self, WorkflowError> {
        let request: Self = serde_json::from_slice(bytes)
            .map_err(|error| contract(format!("original study creation request: {error}")))?;
        if request.parent != parent_run(request.study)
            || request
                .issued
                .as_ref()
                .is_some_and(|issued| issued.study != request.study)
        {
            return Err(contract(
                "original study creation coordinates are inconsistent",
            ));
        }
        if let Some(issued) = &request.issued {
            issued.validate()?;
        }
        Ok(request)
    }
}

pub(super) fn parent_run(study: StudyId) -> RunId {
    RunId::from_id(pse_ids::derive_id(
        Frame::StudyExecutionCoordinateV1,
        &[study.as_bytes(), b"parent"],
    ))
}

pub(super) fn occurrence_run(study: StudyId, occurrence: OccurrenceKey) -> RunId {
    RunId::from_id(pse_ids::derive_id(
        Frame::StudyExecutionCoordinateV1,
        &[study.as_bytes(), b"occurrence", &occurrence.0.to_le_bytes()],
    ))
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "exact source-owned identity codec controls"
    )]
    use super::*;

    #[test]
    fn study_creation_coordinates_preserve_original_study_role_and_occurrence() {
        let original = StudyId::from_bytes([1; 16]);
        let other = StudyId::from_bytes([2; 16]);
        assert_eq!(parent_run(original), parent_run(original));
        assert_eq!(
            occurrence_run(original, OccurrenceKey(7)),
            occurrence_run(original, OccurrenceKey(7))
        );
        assert_ne!(
            parent_run(original),
            occurrence_run(original, OccurrenceKey(0))
        );
        assert_ne!(parent_run(original), parent_run(other));
        assert_ne!(
            occurrence_run(original, OccurrenceKey(7)),
            occurrence_run(original, OccurrenceKey(8))
        );
        assert_ne!(
            occurrence_run(original, OccurrenceKey(7)),
            occurrence_run(other, OccurrenceKey(7))
        );
    }

    #[test]
    fn original_study_creation_request_refuses_changed_version_and_fabricated_coordinates() {
        let study = StudyId::from_bytes([3; 16]);
        let request = StudyCreationRequest {
            version: Version,
            study,
            parent: parent_run(study),
            definition: ContentHash::from_bytes([4; 32]),
            issued: Some(IssuedStudyIdentity {
                study,
                operation: "original-study".into(),
                submission: ContentHash::from_bytes([5; 32]),
            }),
        };
        let bytes = serde_json::to_vec(&request).unwrap();
        assert_eq!(StudyCreationRequest::read(&bytes).unwrap(), request);
        let mut changed = serde_json::to_value(&request).unwrap();
        changed["version"] = 1.into();
        assert!(StudyCreationRequest::read(&serde_json::to_vec(&changed).unwrap()).is_err());
        changed["version"] = 2.into();
        changed["parent"] = serde_json::to_value(RunId::from_bytes([9; 16])).unwrap();
        assert!(StudyCreationRequest::read(&serde_json::to_vec(&changed).unwrap()).is_err());
        assert!(StudyCreationRequest::read(br#"[1,0,0,0]"#).is_err());
    }
}
