// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed dependency census mappings retain native location and package contexts.
use super::*;
pub(super) fn answer_to_pb(
    value: DependencyUsageAnswer,
) -> Result<pb::NativeDependencyUsageAnswer> {
    Ok(pb::NativeDependencyUsageAnswer {
        session: analysis_to_pb(value.session)?.into(),
        generation: value.generation,
        package_ids: value.package_ids,
        unavailable_package_ids: value.unavailable_package_ids,
        indexed_files: number(value.indexed_files),
        unmapped_definitions: number(value.unmapped_definitions),
        limitations: value.limitations,
        references: value
            .references
            .into_iter()
            .map(|value| pb::NativeDependencyReference {
                definition: native_location_to_pb(value.definition).into(),
                profiles: value.profiles.into_iter().map(library_to_pb).collect(),
                occurrence: native_location_to_pb(value.occurrence).into(),
                source_sha256: value.source_sha256,
                symbol_kind: value.symbol_kind,
                name: value.name,
                role: value.role,
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    })
}
pub(in crate::semantic) fn answer_from_pb(
    mut value: pb::NativeDependencyUsageAnswer,
) -> Result<DependencyUsageAnswer> {
    Ok(DependencyUsageAnswer {
        session: analysis_from_pb(required(value.session.take(), "usage.session")?)?,
        generation: value.generation,
        package_ids: value.package_ids,
        unavailable_package_ids: value.unavailable_package_ids,
        indexed_files: size(value.indexed_files, "usage.indexed_files")?,
        unmapped_definitions: size(value.unmapped_definitions, "usage.unmapped_definitions")?,
        limitations: value.limitations,
        references: value
            .references
            .into_iter()
            .map(|mut value| {
                Ok(DependencyReference {
                    definition: native_location_from_pb(required(
                        value.definition.take(),
                        "reference.definition",
                    )?)?,
                    occurrence: native_location_from_pb(required(
                        value.occurrence.take(),
                        "reference.occurrence",
                    )?)?,
                    profiles: value.profiles.into_iter().map(library_from_pb).collect(),
                    source_sha256: value.source_sha256,
                    symbol_kind: value.symbol_kind,
                    name: value.name,
                    role: value.role,
                })
            })
            .collect::<Result<_>>()?,
    })
}
