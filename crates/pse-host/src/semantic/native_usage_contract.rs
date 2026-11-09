// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native census boundary and its shared source context and byte coordinates.

use super::{
    AnalysisContext, Configuration, DependencyUsageAnswer, DependencyUsageQuery, NativeLocation,
};
use pse_contract::boundary_projection::{
    AdapterError, AdapterFailure, CheckedProjectionRecord, Direction, Field, LocalRule,
    ProjectionError, ProjectionGraph, ProjectionType, Shape, TypeRef, ValidateProjection,
    ValidationError,
};
use std::path::PathBuf;

/// Collect both current dependency-usage crossings and their exact native dependencies.
/// # Errors
/// A conflicting or invalid source declaration refuses the projection.
pub fn dependency_usage_projection() -> Result<ProjectionGraph, ProjectionError> {
    let mut graph = ProjectionGraph::of::<DependencyUsageQuery>()?;
    graph.register::<DependencyUsageAnswer>()?;
    Ok(graph)
}

impl ProjectionType for NativeLocation {
    fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
        let root = graph.declare::<Self>(
            "NativeByteLocation",
            file!(),
            Self::VERSION,
            Direction::Both,
            &[],
            &[],
            |graph| {
                Ok(Shape::Record(vec![
                    Field::of::<PathBuf>(
                        graph,
                        "path",
                        1,
                        Direction::Both,
                        vec![LocalRule::Required],
                    )?,
                    Field::of::<usize>(graph, "start", 2, Direction::Both, vec![])?,
                    Field::of::<usize>(graph, "end", 3, Direction::Both, vec![])?,
                ]))
            },
        )?;
        graph.public_path::<Self>(&root, "crate::semantic::NativeLocation")?;
        graph.checked_record::<Self>(&root)?;
        Ok(root)
    }
}

impl ValidateProjection for NativeLocation {
    fn validate_at(&self, path: &str) -> Result<(), ValidationError> {
        if self.path.as_os_str().is_empty() || self.end < self.start {
            return Err(ValidationError::Rule {
                path: path.into(),
                rule: "native_location",
                reason: "native byte location requires a path and an ordered interval".into(),
            });
        }
        Ok(())
    }
}

impl CheckedProjectionRecord for NativeLocation {
    const VERSION: u32 = 1;
    const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[(1, "path"), (2, "start"), (3, "end")];
    type Fields = (PathBuf, usize, usize);

    fn try_into_projection_fields(self) -> Result<Self::Fields, AdapterError> {
        self.validate_projection()?;
        Ok((self.path, self.start, self.end))
    }

    fn from_projection_fields((path, start, end): Self::Fields) -> Result<Self, AdapterError> {
        let location = Self { path, start, end };
        location.validate_projection()?;
        Ok(location)
    }
}

impl ProjectionType for AnalysisContext {
    fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
        let root = graph.declare::<Self>(
            "NativeAnalysisContext",
            file!(),
            Self::VERSION,
            Direction::Both,
            &[],
            &[],
            |graph| {
                Ok(Shape::Record(vec![
                    Field::of::<String>(graph, "backend", 1, Direction::Both, vec![])?,
                    Field::of::<Configuration>(
                        graph,
                        "configuration",
                        2,
                        Direction::Both,
                        vec![LocalRule::Required],
                    )?,
                    Field::of::<String>(graph, "position_encoding", 3, Direction::Both, vec![])?,
                    Field::of::<Option<Vec<u8>>>(
                        graph,
                        "backend_status",
                        4,
                        Direction::Both,
                        vec![],
                    )?,
                    Field::of::<String>(graph, "preparation", 5, Direction::Both, vec![])?,
                ]))
            },
        )?;
        graph.public_path::<Self>(&root, "crate::semantic::AnalysisContext")?;
        graph.checked_record::<Self>(&root)?;
        Ok(root)
    }
}

impl ValidateProjection for AnalysisContext {
    fn validate_at(&self, path: &str) -> Result<(), ValidationError> {
        self.configuration
            .validate_at(&pse_contract::boundary_projection::field_path(
                path,
                "configuration",
            ))
    }
}

impl CheckedProjectionRecord for AnalysisContext {
    const VERSION: u32 = 1;
    const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[
        (1, "backend"),
        (2, "configuration"),
        (3, "position_encoding"),
        (4, "backend_status"),
        (5, "preparation"),
    ];
    type Fields = (String, Configuration, String, Option<Vec<u8>>, String);

    fn try_into_projection_fields(self) -> Result<Self::Fields, AdapterError> {
        let status = self
            .backend_status
            .as_ref()
            .map(serde_json::to_vec)
            .transpose()
            .map_err(status_error)?;
        Ok((
            self.backend,
            self.configuration,
            self.position_encoding,
            status,
            self.preparation,
        ))
    }

    fn from_projection_fields(
        (backend, configuration, position_encoding, status, preparation): Self::Fields,
    ) -> Result<Self, AdapterError> {
        let backend_status = status
            .as_deref()
            .map(serde_json::from_slice)
            .transpose()
            .map_err(status_error)?;
        Ok(Self {
            backend,
            configuration,
            position_encoding,
            backend_status,
            preparation,
        })
    }
}

fn status_error(error: serde_json::Error) -> AdapterError {
    AdapterError::refused(
        "backend_status",
        AdapterFailure::InvalidValue,
        error.to_string(),
    )
}
