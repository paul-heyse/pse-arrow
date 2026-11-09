use super::{
    AdapterError, AdapterFailure, BoundaryProjectionError, NativeInto, NativeFrom,
    layout, child_path, refused, required, unknown_fields, native_into, native_from,
    encode_root, decode_root, items, from_items,
};
impl NativeInto<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeAnalysisBackend,
> for crate::semantic::Backend {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeAnalysisBackend,
        AdapterError,
    > {
        let value = self;
        layout(
            <crate::semantic::Backend as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <crate::semantic::Backend as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::semantic::Backend as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeAnalysisBackend {
                    kind: Some(
                        crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::oneof::native_analysis_backend::Kind::Lsp(
                            Box::new(crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::native_analysis_backend::Alternative1 {
                                ..Default::default()
                            }),
                        ),
                    ),
                    ..Default::default()
                });
            }
            Err(value) => value,
        };
        layout(
            <crate::semantic::Backend as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <crate::semantic::Backend as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::semantic::Backend as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeAnalysisBackend {
                    kind: Some(
                        crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::oneof::native_analysis_backend::Kind::NativeSnapshot(
                            Box::new(crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::native_analysis_backend::Alternative2 {
                                ..Default::default()
                            }),
                        ),
                    ),
                    ..Default::default()
                });
            }
            Err(value) => value,
        };
        let _ = value;
        Err(
            refused(
                path,
                AdapterFailure::UnknownAlternative,
                "actual alternative is outside the generated source closure",
            ),
        )
    }
}
impl NativeFrom<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeAnalysisBackend,
> for crate::semantic::Backend {
    fn from_wire(
        wire: crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeAnalysisBackend,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        match wire
            .kind
            .ok_or_else(|| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "required alternative is absent",
            ))?
        {
            crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::oneof::native_analysis_backend::Kind::Lsp(
                wire,
            ) => {
                let alternative_path = child_path(path, "Lsp");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::semantic::Backend as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <crate::semantic::Backend as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::semantic::Backend as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::oneof::native_analysis_backend::Kind::NativeSnapshot(
                wire,
            ) => {
                let alternative_path = child_path(path, "NativeSnapshot");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::semantic::Backend as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <crate::semantic::Backend as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::semantic::Backend as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeAnalysisContext,
> for crate::semantic::AnalysisContext {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeAnalysisContext,
        AdapterError,
    > {
        layout(
            <crate::semantic::AnalysisContext as pse_contract::boundary_projection::CheckedProjectionRecord>::VERSION,
            <crate::semantic::AnalysisContext as pse_contract::boundary_projection::CheckedProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "backend"),
                (2, "configuration"),
                (3, "position_encoding"),
                (4, "backend_status"),
                (5, "preparation"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4) = pse_contract::boundary_projection::CheckedProjectionRecord::try_into_projection_fields(
                self,
            )
            .map_err(|error| error.at(path))?;
        Ok(crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeAnalysisContext {
            backend: native_into::<_, String>(field_0, &child_path(path, "backend"))?,
            configuration: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeConfiguration,
                >(field_1, &child_path(path, "configuration"))?,
            ),
            position_encoding: native_into::<
                _,
                String,
            >(field_2, &child_path(path, "position_encoding"))?,
            backend_status: field_3
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                            value,
                        ),
                    )
                })
                .transpose()?,
            preparation: native_into::<
                _,
                String,
            >(field_4, &child_path(path, "preparation"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeAnalysisContext,
> for crate::semantic::AnalysisContext {
    fn from_wire(
        wire: crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeAnalysisContext,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::semantic::AnalysisContext as pse_contract::boundary_projection::CheckedProjectionRecord>::VERSION,
            <crate::semantic::AnalysisContext as pse_contract::boundary_projection::CheckedProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "backend"),
                (2, "configuration"),
                (3, "position_encoding"),
                (4, "backend_status"),
                (5, "preparation"),
            ],
            path,
        )?;
        let fields = (
            native_from::<_, String>(wire.backend, &child_path(path, "backend"))?,
            native_from::<
                _,
                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeConfiguration,
            >(
                required(
                    wire.configuration.into_option(),
                    &child_path(path, "configuration"),
                )?,
                &child_path(path, "configuration"),
            )?,
            native_from::<
                _,
                String,
            >(wire.position_encoding, &child_path(path, "position_encoding"))?,
            wire
                .backend_status
                .map(|value| -> Result<_, AdapterError> {
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                        value,
                        &child_path(path, "backend_status"),
                    )
                })
                .transpose()?,
            native_from::<_, String>(wire.preparation, &child_path(path, "preparation"))?,
        );
        pse_contract::boundary_projection::CheckedProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeByteLocation,
> for crate::semantic::NativeLocation {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeByteLocation,
        AdapterError,
    > {
        layout(
            <crate::semantic::NativeLocation as pse_contract::boundary_projection::CheckedProjectionRecord>::VERSION,
            <crate::semantic::NativeLocation as pse_contract::boundary_projection::CheckedProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "path"), (2, "start"), (3, "end")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_contract::boundary_projection::CheckedProjectionRecord::try_into_projection_fields(
                self,
            )
            .map_err(|error| error.at(path))?;
        Ok(crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeByteLocation {
            path: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePathBuf,
                >(field_0, &child_path(path, "path"))?,
            ),
            start: native_into::<_, u64>(field_1, &child_path(path, "start"))?,
            end: native_into::<_, u64>(field_2, &child_path(path, "end"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeByteLocation,
> for crate::semantic::NativeLocation {
    fn from_wire(
        wire: crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeByteLocation,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::semantic::NativeLocation as pse_contract::boundary_projection::CheckedProjectionRecord>::VERSION,
            <crate::semantic::NativeLocation as pse_contract::boundary_projection::CheckedProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "path"), (2, "start"), (3, "end")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePathBuf,
            >(
                required(wire.path.into_option(), &child_path(path, "path"))?,
                &child_path(path, "path"),
            )?,
            native_from::<_, u64>(wire.start, &child_path(path, "start"))?,
            native_from::<_, u64>(wire.end, &child_path(path, "end"))?,
        );
        pse_contract::boundary_projection::CheckedProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeConfiguration,
> for crate::semantic::Configuration {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeConfiguration,
        AdapterError,
    > {
        layout(
            <crate::semantic::Configuration as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::semantic::Configuration as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "backend"),
                (2, "project"),
                (3, "features"),
                (4, "no_default_features"),
                (5, "target"),
                (6, "build_macros"),
                (7, "test_context"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5, field_6) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeConfiguration {
            backend: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeAnalysisBackend,
                >(field_0, &child_path(path, "backend"))?,
            ),
            project: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePathBuf,
                >(field_1, &child_path(path, "project"))?,
            ),
            features: {
                let current_path = &child_path(path, "features");
                items(field_2)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            String,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            no_default_features: native_into::<
                _,
                bool,
            >(field_3, &child_path(path, "no_default_features"))?,
            target: field_4
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<_, String>(value, &child_path(path, "target"))
                })
                .transpose()?,
            build_macros: native_into::<
                _,
                bool,
            >(field_5, &child_path(path, "build_macros"))?,
            test_context: native_into::<
                _,
                bool,
            >(field_6, &child_path(path, "test_context"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeConfiguration,
> for crate::semantic::Configuration {
    fn from_wire(
        wire: crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeConfiguration,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::semantic::Configuration as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::semantic::Configuration as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "backend"),
                (2, "project"),
                (3, "features"),
                (4, "no_default_features"),
                (5, "target"),
                (6, "build_macros"),
                (7, "test_context"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeAnalysisBackend,
            >(
                required(wire.backend.into_option(), &child_path(path, "backend"))?,
                &child_path(path, "backend"),
            )?,
            native_from::<
                _,
                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePathBuf,
            >(
                required(wire.project.into_option(), &child_path(path, "project"))?,
                &child_path(path, "project"),
            )?,
            {
                let current_path = &child_path(path, "features");
                let values = wire
                    .features
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            String,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                bool,
            >(wire.no_default_features, &child_path(path, "no_default_features"))?,
            wire
                .target
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<_, String>(value, &child_path(path, "target"))
                })
                .transpose()?,
            native_from::<
                _,
                bool,
            >(wire.build_macros, &child_path(path, "build_macros"))?,
            native_from::<_, bool>(wire.test_context, &child_path(path, "test_context"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeDependencyReference,
> for crate::semantic::DependencyReference {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeDependencyReference,
        AdapterError,
    > {
        layout(
            <crate::semantic::DependencyReference as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::semantic::DependencyReference as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "definition"),
                (2, "profiles"),
                (3, "occurrence"),
                (4, "source_sha256"),
                (5, "symbol_kind"),
                (6, "name"),
                (7, "role"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5, field_6) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeDependencyReference {
            definition: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeByteLocation,
                >(field_0, &child_path(path, "definition"))?,
            ),
            profiles: {
                let current_path = &child_path(path, "profiles");
                items(field_1)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeLibraryProfile,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            occurrence: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeByteLocation,
                >(field_2, &child_path(path, "occurrence"))?,
            ),
            source_sha256: native_into::<
                _,
                String,
            >(field_3, &child_path(path, "source_sha256"))?,
            symbol_kind: native_into::<
                _,
                String,
            >(field_4, &child_path(path, "symbol_kind"))?,
            name: field_5
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<_, String>(value, &child_path(path, "name"))
                })
                .transpose()?,
            role: native_into::<_, String>(field_6, &child_path(path, "role"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeDependencyReference,
> for crate::semantic::DependencyReference {
    fn from_wire(
        wire: crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeDependencyReference,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::semantic::DependencyReference as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::semantic::DependencyReference as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "definition"),
                (2, "profiles"),
                (3, "occurrence"),
                (4, "source_sha256"),
                (5, "symbol_kind"),
                (6, "name"),
                (7, "role"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeByteLocation,
            >(
                required(
                    wire.definition.into_option(),
                    &child_path(path, "definition"),
                )?,
                &child_path(path, "definition"),
            )?,
            {
                let current_path = &child_path(path, "profiles");
                let values = wire
                    .profiles
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeLibraryProfile,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeByteLocation,
            >(
                required(
                    wire.occurrence.into_option(),
                    &child_path(path, "occurrence"),
                )?,
                &child_path(path, "occurrence"),
            )?,
            native_from::<
                _,
                String,
            >(wire.source_sha256, &child_path(path, "source_sha256"))?,
            native_from::<
                _,
                String,
            >(wire.symbol_kind, &child_path(path, "symbol_kind"))?,
            wire
                .name
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<_, String>(value, &child_path(path, "name"))
                })
                .transpose()?,
            native_from::<_, String>(wire.role, &child_path(path, "role"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeDependencyUsageAnswer,
> for crate::semantic::DependencyUsageAnswer {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeDependencyUsageAnswer,
        AdapterError,
    > {
        layout(
            <crate::semantic::DependencyUsageAnswer as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::semantic::DependencyUsageAnswer as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "session"),
                (2, "generation"),
                (3, "package_ids"),
                (8, "unavailable_package_ids"),
                (4, "references"),
                (5, "indexed_files"),
                (6, "unmapped_definitions"),
                (7, "limitations"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5, field_6, field_7) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeDependencyUsageAnswer {
            session: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeAnalysisContext,
                >(field_0, &child_path(path, "session"))?,
            ),
            generation: native_into::<_, u64>(field_1, &child_path(path, "generation"))?,
            package_ids: {
                let current_path = &child_path(path, "package_ids");
                items(field_2)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            String,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            unavailable_package_ids: {
                let current_path = &child_path(path, "unavailable_package_ids");
                items(field_3)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            String,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            references: {
                let current_path = &child_path(path, "references");
                items(field_4)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeDependencyReference,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            indexed_files: native_into::<
                _,
                u64,
            >(field_5, &child_path(path, "indexed_files"))?,
            unmapped_definitions: native_into::<
                _,
                u64,
            >(field_6, &child_path(path, "unmapped_definitions"))?,
            limitations: {
                let current_path = &child_path(path, "limitations");
                items(field_7)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            String,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeDependencyUsageAnswer,
> for crate::semantic::DependencyUsageAnswer {
    fn from_wire(
        wire: crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeDependencyUsageAnswer,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::semantic::DependencyUsageAnswer as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::semantic::DependencyUsageAnswer as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "session"),
                (2, "generation"),
                (3, "package_ids"),
                (8, "unavailable_package_ids"),
                (4, "references"),
                (5, "indexed_files"),
                (6, "unmapped_definitions"),
                (7, "limitations"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeAnalysisContext,
            >(
                required(wire.session.into_option(), &child_path(path, "session"))?,
                &child_path(path, "session"),
            )?,
            native_from::<_, u64>(wire.generation, &child_path(path, "generation"))?,
            {
                let current_path = &child_path(path, "package_ids");
                let values = wire
                    .package_ids
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            String,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "unavailable_package_ids");
                let values = wire
                    .unavailable_package_ids
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            String,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "references");
                let values = wire
                    .references
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeDependencyReference,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                u64,
            >(wire.indexed_files, &child_path(path, "indexed_files"))?,
            native_from::<
                _,
                u64,
            >(wire.unmapped_definitions, &child_path(path, "unmapped_definitions"))?,
            {
                let current_path = &child_path(path, "limitations");
                let values = wire
                    .limitations
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            String,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeDependencyUsageQuery,
> for crate::semantic::DependencyUsageQuery {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeDependencyUsageQuery,
        AdapterError,
    > {
        layout(
            <crate::semantic::DependencyUsageQuery as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::semantic::DependencyUsageQuery as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "package_ids")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeDependencyUsageQuery {
            package_ids: {
                let current_path = &child_path(path, "package_ids");
                items(field_0)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            String,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeDependencyUsageQuery,
> for crate::semantic::DependencyUsageQuery {
    fn from_wire(
        wire: crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeDependencyUsageQuery,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::semantic::DependencyUsageQuery as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::semantic::DependencyUsageQuery as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "package_ids")],
            path,
        )?;
        let fields = (
            {
                let current_path = &child_path(path, "package_ids");
                let values = wire
                    .package_ids
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            String,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeLibraryProfile,
> for crate::semantic::NativeLibraryProfile {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeLibraryProfile,
        AdapterError,
    > {
        layout(
            <crate::semantic::NativeLibraryProfile as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::semantic::NativeLibraryProfile as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "package_id"),
                (2, "manifest"),
                (3, "source_root"),
                (4, "library_root"),
                (5, "features"),
                (6, "target"),
                (7, "role"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5, field_6) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeLibraryProfile {
            package_id: native_into::<
                _,
                String,
            >(field_0, &child_path(path, "package_id"))?,
            manifest: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePathBuf,
                >(field_1, &child_path(path, "manifest"))?,
            ),
            source_root: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePathBuf,
                >(field_2, &child_path(path, "source_root"))?,
            ),
            library_root: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePathBuf,
                >(field_3, &child_path(path, "library_root"))?,
            ),
            features: {
                let current_path = &child_path(path, "features");
                items(field_4)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            String,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            target: field_5
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<_, String>(value, &child_path(path, "target"))
                })
                .transpose()?,
            role: native_into::<_, String>(field_6, &child_path(path, "role"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeLibraryProfile,
> for crate::semantic::NativeLibraryProfile {
    fn from_wire(
        wire: crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeLibraryProfile,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::semantic::NativeLibraryProfile as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::semantic::NativeLibraryProfile as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "package_id"),
                (2, "manifest"),
                (3, "source_root"),
                (4, "library_root"),
                (5, "features"),
                (6, "target"),
                (7, "role"),
            ],
            path,
        )?;
        let fields = (
            native_from::<_, String>(wire.package_id, &child_path(path, "package_id"))?,
            native_from::<
                _,
                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePathBuf,
            >(
                required(wire.manifest.into_option(), &child_path(path, "manifest"))?,
                &child_path(path, "manifest"),
            )?,
            native_from::<
                _,
                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePathBuf,
            >(
                required(
                    wire.source_root.into_option(),
                    &child_path(path, "source_root"),
                )?,
                &child_path(path, "source_root"),
            )?,
            native_from::<
                _,
                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePathBuf,
            >(
                required(
                    wire.library_root.into_option(),
                    &child_path(path, "library_root"),
                )?,
                &child_path(path, "library_root"),
            )?,
            {
                let current_path = &child_path(path, "features");
                let values = wire
                    .features
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            String,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            wire
                .target
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<_, String>(value, &child_path(path, "target"))
                })
                .transpose()?,
            native_from::<_, String>(wire.role, &child_path(path, "role"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePathBuf,
> for std::path::PathBuf {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePathBuf,
        AdapterError,
    > {
        layout(
            <std::path::PathBuf as pse_contract::boundary_projection::CheckedProjectionRecord>::VERSION,
            <std::path::PathBuf as pse_contract::boundary_projection::CheckedProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "platform")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::CheckedProjectionRecord::try_into_projection_fields(
                self,
            )
            .map_err(|error| error.at(path))?;
        Ok(crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePathBuf {
            platform: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::PlatformPathBytes,
                >(field_0, &child_path(path, "platform"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePathBuf,
> for std::path::PathBuf {
    fn from_wire(
        wire: crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativePathBuf,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <std::path::PathBuf as pse_contract::boundary_projection::CheckedProjectionRecord>::VERSION,
            <std::path::PathBuf as pse_contract::boundary_projection::CheckedProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "platform")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::PlatformPathBytes,
            >(
                required(wire.platform.into_option(), &child_path(path, "platform"))?,
                &child_path(path, "platform"),
            )?,
        );
        pse_contract::boundary_projection::CheckedProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::PlatformPathBytes,
> for pse_contract::boundary_projection::PlatformPathBytes {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::PlatformPathBytes,
        AdapterError,
    > {
        let value = self;
        layout(
            <pse_contract::boundary_projection::PlatformPathBytes as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <pse_contract::boundary_projection::PlatformPathBytes as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_contract::boundary_projection::PlatformPathBytes as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                return Ok(crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::PlatformPathBytes {
                    kind: Some(
                        crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::oneof::platform_path_bytes::Kind::Unix(
                            Box::new(crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::platform_path_bytes::Alternative1 {
                                value: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                                    field_0,
                                ),
                                ..Default::default()
                            }),
                        ),
                    ),
                    ..Default::default()
                });
            }
            Err(value) => value,
        };
        layout(
            <pse_contract::boundary_projection::PlatformPathBytes as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <pse_contract::boundary_projection::PlatformPathBytes as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_contract::boundary_projection::PlatformPathBytes as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Windows");
                let path = alternative_path.as_str();
                return Ok(crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::PlatformPathBytes {
                    kind: Some(
                        crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::oneof::platform_path_bytes::Kind::Windows(
                            Box::new(crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::platform_path_bytes::Alternative2 {
                                value: {
                                    let current_path = &child_path(path, "value");
                                    items(field_0)
                                        .enumerate()
                                        .map(|(index, value)| -> Result<_, AdapterError> {
                                            native_into::<
                                                _,
                                                u32,
                                            >(value, &format!("{path}[{index}]", path = current_path))
                                        })
                                        .collect::<Result<Vec<_>, _>>()?
                                },
                                ..Default::default()
                            }),
                        ),
                    ),
                    ..Default::default()
                });
            }
            Err(value) => value,
        };
        let _ = value;
        Err(
            refused(
                path,
                AdapterFailure::UnknownAlternative,
                "actual alternative is outside the generated source closure",
            ),
        )
    }
}
impl NativeFrom<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::PlatformPathBytes,
> for pse_contract::boundary_projection::PlatformPathBytes {
    fn from_wire(
        wire: crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::PlatformPathBytes,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        match wire
            .kind
            .ok_or_else(|| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "required alternative is absent",
            ))?
        {
            crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::oneof::platform_path_bytes::Kind::Unix(
                wire,
            ) => {
                let alternative_path = child_path(path, "Unix");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_contract::boundary_projection::PlatformPathBytes as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_contract::boundary_projection::PlatformPathBytes as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                        wire.value,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <pse_contract::boundary_projection::PlatformPathBytes as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::__buffa::oneof::platform_path_bytes::Kind::Windows(
                wire,
            ) => {
                let alternative_path = child_path(path, "Windows");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_contract::boundary_projection::PlatformPathBytes as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_contract::boundary_projection::PlatformPathBytes as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    {
                        let current_path = &child_path(path, "value");
                        let values = wire
                            .value
                            .into_iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                native_from::<
                                    _,
                                    u32,
                                >(value, &format!("{path}[{index}]", path = current_path))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_contract::boundary_projection::PlatformPathBytes as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
/// Encode the actual native `NativeDependencyUsageQuery` and validate outgoing local rules.
pub(crate) fn encode_native_dependency_usage_query(
    native: crate::semantic::DependencyUsageQuery,
) -> Result<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeDependencyUsageQuery,
    BoundaryProjectionError,
> {
    encode_root(native, "NativeDependencyUsageQuery")
}
/// Adopt known `NativeDependencyUsageQuery` meaning under this package's declared unknown-field policy.
pub(crate) fn decode_native_dependency_usage_query(
    wire: crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeDependencyUsageQuery,
) -> Result<crate::semantic::DependencyUsageQuery, BoundaryProjectionError> {
    decode_root(wire, "NativeDependencyUsageQuery")
}
/// Encode the actual native `NativeDependencyUsageAnswer` and validate outgoing local rules.
pub(crate) fn encode_native_dependency_usage_answer(
    native: crate::semantic::DependencyUsageAnswer,
) -> Result<
    crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeDependencyUsageAnswer,
    BoundaryProjectionError,
> {
    encode_root(native, "NativeDependencyUsageAnswer")
}
/// Adopt known `NativeDependencyUsageAnswer` meaning under this package's declared unknown-field policy.
pub(crate) fn decode_native_dependency_usage_answer(
    wire: crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::NativeDependencyUsageAnswer,
) -> Result<crate::semantic::DependencyUsageAnswer, BoundaryProjectionError> {
    decode_root(wire, "NativeDependencyUsageAnswer")
}
