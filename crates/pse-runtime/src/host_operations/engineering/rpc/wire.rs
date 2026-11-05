// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit adapters between generated messages and their actual native operation owners.

use super::{EngineeringServiceError, native, pb};

pub(super) fn admit_from_pb(value: pb::EngineeringAdmitSource) -> native::EngineeringAdmitSource {
    native::EngineeringAdmitSource {
        operation: value.operation,
        physical: value
            .physical
            .into_iter()
            .map(|package| native::EngineeringPackage {
                files: package.files.into_iter().collect(),
            })
            .collect(),
        modeling: value
            .modeling
            .into_iter()
            .map(|package| native::EngineeringPackage {
                files: package.files.into_iter().collect(),
            })
            .collect(),
    }
}
pub(super) fn key_from_pb(value: pb::EngineeringOperationKey) -> native::EngineeringOperationKey {
    native::EngineeringOperationKey {
        operation: value.operation,
    }
}
pub(super) fn run_key_from_pb(
    value: pb::EngineeringRunKey,
) -> Result<native::EngineeringRunKey, EngineeringServiceError> {
    Ok(native::EngineeringRunKey {
        operation: value.operation,
        run: semantic(&value.run)?,
        attempt: semantic(&value.attempt)?,
    })
}
pub(super) fn inspect_from_pb(
    value: pb::EngineeringInspectSource,
) -> native::EngineeringInspectSource {
    native::EngineeringInspectSource {
        operation: value.operation,
        offset: value.offset,
        limit: value.limit,
    }
}
pub(super) fn prepare_from_pb(
    value: pb::EngineeringPrepareCase,
) -> Result<native::EngineeringPrepareCase, EngineeringServiceError> {
    let intent = value
        .intent
        .into_option()
        .and_then(|intent| {
            intent
                .value
                .parse::<pse_model::generated::enums::NativeSolveIntent>()
                .ok()
        })
        .ok_or_else(|| {
            EngineeringServiceError::Request("a supported native solve intent is required".into())
        })?;
    let backend = value
        .backend
        .into_option()
        .map(|backend| {
            backend
                .value
                .parse::<pse_model::generated::enums::NativeBackend>()
                .map_err(|error| {
                    EngineeringServiceError::Request(format!("unsupported native backend: {error}"))
                })
        })
        .transpose()?;
    Ok(native::EngineeringPrepareCase {
        operation: value.operation,
        source_operation: value.source_operation,
        expected_local_revision: value.expected_local_revision,
        case: pse_ids::SemanticId::try_from_slice(&value.case)
            .map_err(|error| EngineeringServiceError::Request(error.to_string()))?,
        intent,
        backend,
    })
}
pub(super) fn submit_from_pb(
    value: pb::EngineeringSubmitRun,
) -> Result<native::EngineeringSubmitRun, EngineeringServiceError> {
    Ok(native::EngineeringSubmitRun {
        operation: value.operation,
        preparation_operation: value.preparation_operation,
        run: semantic(&value.run)?,
        attempt: semantic(&value.attempt)?,
        expected_preparation: pse_ids::ContentHash::try_from_slice(&value.expected_preparation)
            .map_err(|error| EngineeringServiceError::Request(error.to_string()))?,
    })
}
pub(super) fn read_from_pb(
    value: pb::EngineeringReadResult,
) -> Result<native::EngineeringReadResult, EngineeringServiceError> {
    Ok(native::EngineeringReadResult {
        operation: value.operation,
        relation: value.relation,
        start: value.start,
        end: value.end,
        maximum_bytes: value.maximum_bytes,
        run: semantic(&value.run)?,
        attempt: semantic(&value.attempt)?,
    })
}
fn semantic(bytes: &[u8]) -> Result<pse_ids::SemanticId, EngineeringServiceError> {
    pse_ids::SemanticId::try_from_slice(bytes)
        .map_err(|error| EngineeringServiceError::Request(error.to_string()))
}

pub(super) fn source_to_pb(
    value: native::EngineeringSourceSelection,
) -> pb::EngineeringSourceSelection {
    pb::EngineeringSourceSelection {
        operation: value.operation,
        local_revision: value.local_revision,
        source_revision: value.source_revision.as_bytes().to_vec(),
        physical_context: value.physical_context.as_bytes().to_vec(),
        ..Default::default()
    }
}
pub(super) fn failure_to_pb(value: native::EngineeringFailure) -> pb::EngineeringFailure {
    pb::EngineeringFailure {
        code: value.code,
        message: value.message,
        sources: value
            .sources
            .into_iter()
            .map(|source| source.as_bytes().to_vec())
            .collect(),
        rule: value.rule,
        field_path: value.field_path,
        field_rule: value.field_rule,
        ..Default::default()
    }
}
pub(super) fn failure(error: &EngineeringServiceError) -> native::EngineeringFailure {
    super::super::session::failure(error)
}
pub(super) fn declaration_page_to_pb(
    value: native::EngineeringDeclarationPage,
) -> pb::EngineeringDeclarationPage {
    pb::EngineeringDeclarationPage {
        source: source_to_pb(value.source).into(),
        total: value.total,
        offset: value.offset,
        next_offset: value.next_offset,
        declarations: value
            .declarations
            .into_iter()
            .map(|declaration| pb::EngineeringDeclarationHeader {
                declaration: declaration.declaration.as_bytes().to_vec(),
                name: declaration.name,
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }
}
pub(super) fn prepared_to_pb(
    value: native::EngineeringPreparedSelection,
) -> pb::EngineeringPreparedSelection {
    pb::EngineeringPreparedSelection {
        operation: value.operation,
        source: source_to_pb(value.source).into(),
        preparation: value.preparation.as_bytes().to_vec(),
        ..Default::default()
    }
}
pub(super) fn status_to_pb(value: native::EngineeringRunStatus) -> pb::EngineeringRunStatus {
    pb::EngineeringRunStatus {
        operation: value.operation,
        run: value.run.map(|run| run.as_bytes().to_vec()),
        settled: value.settled,
        usable: value.usable,
        cancellation_requested: value.cancellation_requested,
        failure: value.failure.map(failure_to_pb).into(),
        canonical_run: value.canonical_run,
        canonical_attempt: value.canonical_attempt,
        progress: value.progress.map(progress_to_pb).into(),
        ..Default::default()
    }
}
fn progress_to_pb(value: native::EngineeringRunProgress) -> pb::EngineeringRunProgress {
    pb::EngineeringRunProgress {
        generation: value.generation,
        highest_issued_sequence: value.highest_issued_sequence,
        dropped: value.dropped,
        current_is_latest: value.current_is_latest,
        phase: value.phase,
        elapsed_seconds_bits: value.elapsed_seconds_bits,
        sequence: value.sequence,
        values: value
            .values
            .into_iter()
            .map(|(key, value)| (key, progress_value_to_pb(value)))
            .collect(),
        observations_available: value.observations_available,
        observation_updates_dropped: value.observation_updates_dropped,
        ..Default::default()
    }
}
fn progress_value_to_pb(value: native::EngineeringProgressValue) -> pb::EngineeringProgressValue {
    use pb::engineering_progress_value::{
        Alternative1, Alternative2, Alternative3, Alternative4, Alternative5, Kind,
    };
    let kind = match value {
        native::EngineeringProgressValue::Integer(value) => Kind::Integer(Box::new(Alternative1 {
            value,
            ..Default::default()
        })),
        native::EngineeringProgressValue::RealBits(value) => {
            Kind::RealBits(Box::new(Alternative2 {
                value,
                ..Default::default()
            }))
        }
        native::EngineeringProgressValue::Text(value) => Kind::Text(Box::new(Alternative3 {
            value,
            ..Default::default()
        })),
        native::EngineeringProgressValue::Boolean(value) => Kind::Boolean(Box::new(Alternative4 {
            value,
            ..Default::default()
        })),
        native::EngineeringProgressValue::Unavailable(value) => {
            Kind::Unavailable(Box::new(Alternative5 {
                value: pb::EvidenceUnavailableReason {
                    value: value.as_str().into(),
                    ..Default::default()
                }
                .into(),
                ..Default::default()
            }))
        }
    };
    pb::EngineeringProgressValue {
        kind: Some(kind),
        ..Default::default()
    }
}
pub(super) fn page_to_pb(value: native::EngineeringResultPage) -> pb::EngineeringResultPage {
    pb::EngineeringResultPage {
        run: value.run.as_bytes().to_vec(),
        relation: value.relation,
        start: value.start,
        end: value.end,
        rows: value.rows,
        ipc: value.ipc,
        ..Default::default()
    }
}
