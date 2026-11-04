// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed authored-file intent; no scientific parsing or source interpretation here.
use super::*;

pub(super) fn from_pb(
    value: pb::EngineeringEditSource,
) -> Result<native::EngineeringEditSource, EngineeringServiceError> {
    let delta = value.delta.into_option().ok_or_else(|| {
        EngineeringServiceError::Request("exact-parent file delta is required".into())
    })?;
    let edits = delta
        .edits
        .into_iter()
        .map(|edit| {
            let change = edit
                .change
                .into_option()
                .and_then(|change| change.kind)
                .ok_or_else(|| {
                    EngineeringServiceError::Request(
                        "explicit source file change is required".into(),
                    )
                })?;
            use pb::source_file_change::Kind;
            let change = match change {
                Kind::Add(value) => native::SourceFileChange::Add(value.value),
                Kind::Replace(value) => native::SourceFileChange::Replace(value.value),
                Kind::Remove(_) => native::SourceFileChange::Remove,
            };
            Ok(native::SourceFileEdit {
                package: pse_ids::SemanticId::try_from_slice(&edit.package)
                    .map_err(|error| EngineeringServiceError::Request(error.to_string()))?,
                path: edit.path,
                change,
            })
        })
        .collect::<Result<_, EngineeringServiceError>>()?;
    Ok(native::EngineeringEditSource {
        operation: value.operation,
        source_operation: value.source_operation,
        delta: native::SourceFileDelta {
            parent: delta.parent,
            edits,
        },
    })
}
