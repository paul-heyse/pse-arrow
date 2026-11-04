use super::{
    AdapterError, AdapterFailure, BoundaryProjectionError, NativeInto, NativeFrom,
    layout, child_path, refused, required, unknown_fields, native_into, native_from,
    encode_root, decode_root, items, from_items, unique_pairs, wire_map,
};
impl NativeInto<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array16U8,
> for [u8; 16] {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array16U8,
        AdapterError,
    > {
        layout(
            <[u8; 16] as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <[u8; 16] as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "values")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array16U8 {
            values: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_0,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array16U8,
> for [u8; 16] {
    fn from_wire(
        wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array16U8,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <[u8; 16] as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <[u8; 16] as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "values")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.values,
                &child_path(path, "values"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array32U8,
> for [u8; 32] {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array32U8,
        AdapterError,
    > {
        layout(
            <[u8; 32] as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <[u8; 32] as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "values")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array32U8 {
            values: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_0,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array32U8,
> for [u8; 32] {
    fn from_wire(
        wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array32U8,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <[u8; 32] as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <[u8; 32] as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "values")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.values,
                &child_path(path, "values"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array3Array16U8,
> for [[u8; 16]; 3] {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array3Array16U8,
        AdapterError,
    > {
        layout(
            <[[u8; 16]; 3] as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <[[u8; 16]; 3] as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "values")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array3Array16U8 {
            values: {
                let current_path = &child_path(path, "values");
                items(field_0)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array16U8,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array3Array16U8,
> for [[u8; 16]; 3] {
    fn from_wire(
        wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array3Array16U8,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <[[u8; 16]; 3] as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <[[u8; 16]; 3] as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "values")],
            path,
        )?;
        let fields = (
            {
                let current_path = &child_path(path, "values");
                let values = wire
                    .values
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array16U8,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
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
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::CapturedWorkflowEvidence,
> for &crate::test_inventory::CapturedEvidence {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::CapturedWorkflowEvidence,
        AdapterError,
    > {
        layout(
            <crate::test_inventory::CapturedEvidence as pse_contract::boundary_projection::ProjectionObservationRecord>::VERSION,
            <crate::test_inventory::CapturedEvidence as pse_contract::boundary_projection::ProjectionObservationRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "invocation"),
                (2, "report_original"),
                (3, "inventory_original"),
                (4, "inventory_sha256"),
                (5, "report_observation"),
                (6, "terminal"),
                (7, "selected_tests"),
                (8, "selected_population_complete"),
                (9, "recorded_manager_settlement"),
                (10, "recorded_native_lifetime_pending"),
            ],
            path,
        )?;
        let (
            field_0,
            field_1,
            field_2,
            field_3,
            field_4,
            field_5,
            field_6,
            field_7,
            field_8,
            field_9,
        ) = pse_contract::boundary_projection::ProjectionObservationRecord::observation_fields(
                self,
            )
            .map_err(|error| error.at(path))?;
        Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::CapturedWorkflowEvidence {
            invocation: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::CliInvocationId,
                >(field_0, &child_path(path, "invocation"))?,
            ),
            report_original: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_1,
            ),
            inventory_original: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_2,
            ),
            inventory_sha256: native_into::<
                _,
                String,
            >(field_3, &child_path(path, "inventory_sha256"))?,
            report_observation: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowReportObservation,
                >(field_4, &child_path(path, "report_observation"))?,
            ),
            terminal: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowTerminalRecord,
                >(field_5, &child_path(path, "terminal"))?,
            ),
            selected_tests: {
                let current_path = &child_path(path, "selected_tests");
                items(field_6)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::TestIdentity,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            selected_population_complete: field_7
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            bool,
                        >(value, &child_path(path, "selected_population_complete"))?,
                    )
                })
                .transpose()?,
            recorded_manager_settlement: field_8
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ManagerSettlement,
                        >(value, &child_path(path, "recorded_manager_settlement"))?,
                    )
                })
                .transpose()?
                .into(),
            recorded_native_lifetime_pending: field_9
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            bool,
                        >(value, &child_path(path, "recorded_native_lifetime_pending"))?,
                    )
                })
                .transpose()?,
            ..Default::default()
        })
    }
}
impl NativeInto<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::CapturedWorkflowReport,
> for &crate::test_inventory::CapturedReport {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::CapturedWorkflowReport,
        AdapterError,
    > {
        layout(
            <crate::test_inventory::CapturedReport as pse_contract::boundary_projection::ProjectionObservationRecord>::VERSION,
            <crate::test_inventory::CapturedReport as pse_contract::boundary_projection::ProjectionObservationRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "invocation"),
                (2, "report_original"),
                (3, "report_observation"),
                (4, "terminal"),
                (5, "recorded_manager_settlement"),
                (6, "recorded_native_lifetime_pending"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5) = pse_contract::boundary_projection::ProjectionObservationRecord::observation_fields(
                self,
            )
            .map_err(|error| error.at(path))?;
        Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::CapturedWorkflowReport {
            invocation: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::CliInvocationId,
                >(field_0, &child_path(path, "invocation"))?,
            ),
            report_original: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_1,
            ),
            report_observation: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowReportObservation,
                >(field_2, &child_path(path, "report_observation"))?,
            ),
            terminal: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowTerminalRecord,
                >(field_3, &child_path(path, "terminal"))?,
            ),
            recorded_manager_settlement: field_4
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ManagerSettlement,
                        >(value, &child_path(path, "recorded_manager_settlement"))?,
                    )
                })
                .transpose()?
                .into(),
            recorded_native_lifetime_pending: field_5
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            bool,
                        >(value, &child_path(path, "recorded_native_lifetime_pending"))?,
                    )
                })
                .transpose()?,
            ..Default::default()
        })
    }
}
impl NativeInto<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::CliInvocationId,
> for crate::envelope::CliInvocationId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::CliInvocationId,
        AdapterError,
    > {
        layout(
            <crate::envelope::CliInvocationId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::envelope::CliInvocationId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::CliInvocationId {
            value: native_into::<_, String>(field_0, &child_path(path, "value"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::CliInvocationId,
> for crate::envelope::CliInvocationId {
    fn from_wire(
        wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::CliInvocationId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::envelope::CliInvocationId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::envelope::CliInvocationId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let fields = (
            native_from::<_, String>(wire.value, &child_path(path, "value"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticFacts,
> for crate::envelope::FactsRecord {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticFacts,
        AdapterError,
    > {
        layout(
            <crate::envelope::FactsRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::envelope::FactsRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "sources"), (2, "locations"), (3, "rule"), (4, "observations")],
            path,
        )?;
        let (field_0, field_1, field_2, field_3) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticFacts {
            sources: {
                let current_path = &child_path(path, "sources");
                items(field_0)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array16U8,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            locations: {
                let current_path = &child_path(path, "locations");
                items(field_1)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticLocation,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            rule: field_2
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_into::<_, String>(value, &child_path(path, "rule"))?)
                })
                .transpose()?,
            observations: wire_map(
                field_3
                    .into_iter()
                    .map(|(key, value)| -> Result<_, AdapterError> {
                        Ok((
                            native_into::<
                                _,
                                String,
                            >(key, &child_path(path, "observations"))?,
                            native_into::<
                                _,
                                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticObservation,
                            >(value, &child_path(path, "observations"))?,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter(),
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticFacts,
> for crate::envelope::FactsRecord {
    fn from_wire(
        wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticFacts,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::envelope::FactsRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::envelope::FactsRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "sources"), (2, "locations"), (3, "rule"), (4, "observations")],
            path,
        )?;
        let fields = (
            {
                let current_path = &child_path(path, "sources");
                let values = wire
                    .sources
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array16U8,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "locations");
                let values = wire
                    .locations
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticLocation,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            wire
                .rule
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_from::<_, String>(value, &child_path(path, "rule"))?)
                })
                .transpose()?,
            unique_pairs(
                wire
                    .observations
                    .into_iter()
                    .map(|(key, value)| -> Result<_, AdapterError> {
                        Ok((
                            native_from::<
                                _,
                                String,
                            >(key, &child_path(path, "observations"))?,
                            native_from::<
                                _,
                                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticObservation,
                            >(value, &child_path(path, "observations"))?,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                &child_path(path, "observations"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticLabel,
> for crate::envelope::LabelRecord {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticLabel,
        AdapterError,
    > {
        layout(
            <crate::envelope::LabelRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::envelope::LabelRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "label"),
                (2, "primary"),
                (3, "byte_offset"),
                (4, "byte_length"),
                (5, "source"),
                (6, "line"),
                (7, "column"),
                (8, "column_encoding"),
                (9, "unavailable"),
            ],
            path,
        )?;
        let (
            field_0,
            field_1,
            field_2,
            field_3,
            field_4,
            field_5,
            field_6,
            field_7,
            field_8,
        ) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticLabel {
            label: field_0
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_into::<_, String>(value, &child_path(path, "label"))?)
                })
                .transpose()?,
            primary: native_into::<_, bool>(field_1, &child_path(path, "primary"))?,
            byte_offset: native_into::<
                _,
                u64,
            >(field_2, &child_path(path, "byte_offset"))?,
            byte_length: native_into::<
                _,
                u64,
            >(field_3, &child_path(path, "byte_length"))?,
            source: field_4
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_into::<_, String>(value, &child_path(path, "source"))?)
                })
                .transpose()?,
            line: field_5
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_into::<_, u64>(value, &child_path(path, "line"))?)
                })
                .transpose()?,
            column: field_6
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_into::<_, u64>(value, &child_path(path, "column"))?)
                })
                .transpose()?,
            column_encoding: native_into::<
                _,
                String,
            >(field_7, &child_path(path, "column_encoding"))?,
            unavailable: field_8
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            String,
                        >(value, &child_path(path, "unavailable"))?,
                    )
                })
                .transpose()?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticLabel,
> for crate::envelope::LabelRecord {
    fn from_wire(
        wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticLabel,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::envelope::LabelRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::envelope::LabelRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "label"),
                (2, "primary"),
                (3, "byte_offset"),
                (4, "byte_length"),
                (5, "source"),
                (6, "line"),
                (7, "column"),
                (8, "column_encoding"),
                (9, "unavailable"),
            ],
            path,
        )?;
        let fields = (
            wire
                .label
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_from::<_, String>(value, &child_path(path, "label"))?)
                })
                .transpose()?,
            native_from::<_, bool>(wire.primary, &child_path(path, "primary"))?,
            native_from::<_, u64>(wire.byte_offset, &child_path(path, "byte_offset"))?,
            native_from::<_, u64>(wire.byte_length, &child_path(path, "byte_length"))?,
            wire
                .source
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_from::<_, String>(value, &child_path(path, "source"))?)
                })
                .transpose()?,
            wire
                .line
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_from::<_, u64>(value, &child_path(path, "line"))?)
                })
                .transpose()?,
            wire
                .column
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_from::<_, u64>(value, &child_path(path, "column"))?)
                })
                .transpose()?,
            native_from::<
                _,
                String,
            >(wire.column_encoding, &child_path(path, "column_encoding"))?,
            wire
                .unavailable
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            String,
                        >(value, &child_path(path, "unavailable"))?,
                    )
                })
                .transpose()?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticLocation,
> for crate::envelope::LocationRecord {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticLocation,
        AdapterError,
    > {
        layout(
            <crate::envelope::LocationRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::envelope::LocationRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "source"),
                (2, "revision"),
                (3, "document"),
                (4, "occurrence_role"),
                (5, "occurrence_position"),
                (6, "path"),
                (7, "name"),
                (8, "start"),
                (9, "end"),
            ],
            path,
        )?;
        let (
            field_0,
            field_1,
            field_2,
            field_3,
            field_4,
            field_5,
            field_6,
            field_7,
            field_8,
        ) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticLocation {
            source: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array16U8,
                >(field_0, &child_path(path, "source"))?,
            ),
            revision: field_1
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array32U8,
                        >(value, &child_path(path, "revision"))?,
                    )
                })
                .transpose()?
                .into(),
            document: field_2
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array16U8,
                        >(value, &child_path(path, "document"))?,
                    )
                })
                .transpose()?
                .into(),
            occurrence_role: field_3
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            String,
                        >(value, &child_path(path, "occurrence_role"))?,
                    )
                })
                .transpose()?,
            occurrence_position: field_4
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            u32,
                        >(value, &child_path(path, "occurrence_position"))?,
                    )
                })
                .transpose()?,
            path: native_into::<_, String>(field_5, &child_path(path, "path"))?,
            name: field_6
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_into::<_, String>(value, &child_path(path, "name"))?)
                })
                .transpose()?,
            start: field_7
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_into::<_, u32>(value, &child_path(path, "start"))?)
                })
                .transpose()?,
            end: field_8
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_into::<_, u32>(value, &child_path(path, "end"))?)
                })
                .transpose()?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticLocation,
> for crate::envelope::LocationRecord {
    fn from_wire(
        wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticLocation,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::envelope::LocationRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::envelope::LocationRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "source"),
                (2, "revision"),
                (3, "document"),
                (4, "occurrence_role"),
                (5, "occurrence_position"),
                (6, "path"),
                (7, "name"),
                (8, "start"),
                (9, "end"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array16U8,
            >(
                required(wire.source.into_option(), &child_path(path, "source"))?,
                &child_path(path, "source"),
            )?,
            wire
                .revision
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array32U8,
                        >(value, &child_path(path, "revision"))?,
                    )
                })
                .transpose()?,
            wire
                .document
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array16U8,
                        >(value, &child_path(path, "document"))?,
                    )
                })
                .transpose()?,
            wire
                .occurrence_role
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            String,
                        >(value, &child_path(path, "occurrence_role"))?,
                    )
                })
                .transpose()?,
            wire
                .occurrence_position
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            u32,
                        >(value, &child_path(path, "occurrence_position"))?,
                    )
                })
                .transpose()?,
            native_from::<_, String>(wire.path, &child_path(path, "path"))?,
            wire
                .name
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_from::<_, String>(value, &child_path(path, "name"))?)
                })
                .transpose()?,
            wire
                .start
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_from::<_, u32>(value, &child_path(path, "start"))?)
                })
                .transpose()?,
            wire
                .end
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_from::<_, u32>(value, &child_path(path, "end"))?)
                })
                .transpose()?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticObservation,
> for crate::envelope::ObservationValue {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticObservation,
        AdapterError,
    > {
        let value = self;
        layout(
            <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticObservation {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::diagnostic_observation::Kind::Missing(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::diagnostic_observation::Alternative1 {
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
            <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Number");
                let path = alternative_path.as_str();
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticObservation {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::diagnostic_observation::Kind::Number(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::diagnostic_observation::Alternative2 {
                                value: native_into::<
                                    _,
                                    u64,
                                >(field_0, &child_path(path, "value"))?,
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
            <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::VERSION,
            <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
            3,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "NonFinite");
                let path = alternative_path.as_str();
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticObservation {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::diagnostic_observation::Kind::NonFinite(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::diagnostic_observation::Alternative3 {
                                value: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NonFinite,
                                    >(field_0, &child_path(path, "value"))?,
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
            <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::VERSION,
            <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
            4,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Integer");
                let path = alternative_path.as_str();
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticObservation {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::diagnostic_observation::Kind::Integer(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::diagnostic_observation::Alternative4 {
                                value: native_into::<
                                    _,
                                    i64,
                                >(field_0, &child_path(path, "value"))?,
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
            <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                5,
            >>::VERSION,
            <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                5,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
            5,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Boolean");
                let path = alternative_path.as_str();
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticObservation {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::diagnostic_observation::Kind::Boolean(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::diagnostic_observation::Alternative5 {
                                value: native_into::<
                                    _,
                                    bool,
                                >(field_0, &child_path(path, "value"))?,
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
            <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                6,
            >>::VERSION,
            <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                6,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
            6,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Text");
                let path = alternative_path.as_str();
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticObservation {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::diagnostic_observation::Kind::Text(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::diagnostic_observation::Alternative6 {
                                value: native_into::<
                                    _,
                                    String,
                                >(field_0, &child_path(path, "value"))?,
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
            <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                7,
            >>::VERSION,
            <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                7,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
            7,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Contracts");
                let path = alternative_path.as_str();
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticObservation {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::diagnostic_observation::Kind::Contracts(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::diagnostic_observation::Alternative7 {
                                value: {
                                    let current_path = &child_path(path, "value");
                                    items(field_0)
                                        .enumerate()
                                        .map(|(index, value)| -> Result<_, AdapterError> {
                                            Ok(
                                                native_into::<
                                                    _,
                                                    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticOperandContract,
                                                >(value, &format!("{path}[{index}]", path = current_path))?,
                                            )
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
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticObservation,
> for crate::envelope::ObservationValue {
    fn from_wire(
        wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticObservation,
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
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::diagnostic_observation::Kind::Missing(
                wire,
            ) => {
                let alternative_path = child_path(path, "Missing");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::diagnostic_observation::Kind::Number(
                wire,
            ) => {
                let alternative_path = child_path(path, "Number");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<_, u64>(wire.value, &child_path(path, "value"))?,
                );
                Ok(
                    <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::diagnostic_observation::Kind::NonFinite(
                wire,
            ) => {
                let alternative_path = child_path(path, "NonFinite");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NonFinite,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::diagnostic_observation::Kind::Integer(
                wire,
            ) => {
                let alternative_path = child_path(path, "Integer");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<_, i64>(wire.value, &child_path(path, "value"))?,
                );
                Ok(
                    <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::diagnostic_observation::Kind::Boolean(
                wire,
            ) => {
                let alternative_path = child_path(path, "Boolean");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::VERSION,
                    <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<_, bool>(wire.value, &child_path(path, "value"))?,
                );
                Ok(
                    <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::diagnostic_observation::Kind::Text(
                wire,
            ) => {
                let alternative_path = child_path(path, "Text");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::VERSION,
                    <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<_, String>(wire.value, &child_path(path, "value"))?,
                );
                Ok(
                    <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::diagnostic_observation::Kind::Contracts(
                wire,
            ) => {
                let alternative_path = child_path(path, "Contracts");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::VERSION,
                    <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
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
                                Ok(
                                    native_from::<
                                        _,
                                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticOperandContract,
                                    >(value, &format!("{path}[{index}]", path = current_path))?,
                                )
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <crate::envelope::ObservationValue as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticOperandContract,
> for pse_diagnostics::OperandContract {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticOperandContract,
        AdapterError,
    > {
        layout(
            <pse_diagnostics::OperandContract as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_diagnostics::OperandContract as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "quantity"), (2, "indices")],
            path,
        )?;
        let (field_0, field_1) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticOperandContract {
            quantity: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array16U8,
                >(field_0, &child_path(path, "quantity"))?,
            ),
            indices: {
                let current_path = &child_path(path, "indices");
                items(field_1)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array3Array16U8,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticOperandContract,
> for pse_diagnostics::OperandContract {
    fn from_wire(
        wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticOperandContract,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_diagnostics::OperandContract as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_diagnostics::OperandContract as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "quantity"), (2, "indices")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array16U8,
            >(
                required(wire.quantity.into_option(), &child_path(path, "quantity"))?,
                &child_path(path, "quantity"),
            )?,
            {
                let current_path = &child_path(path, "indices");
                let values = wire
                    .indices
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::Array3Array16U8,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
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
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticRecovery,
> for crate::envelope::RecoveryRecord {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticRecovery,
        AdapterError,
    > {
        layout(
            <crate::envelope::RecoveryRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::envelope::RecoveryRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "operation"),
                (2, "operand"),
                (3, "origin"),
                (4, "cause"),
                (5, "next"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticRecovery {
            operation: native_into::<
                _,
                String,
            >(field_0, &child_path(path, "operation"))?,
            operand: field_1
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NativePath,
                        >(value, &child_path(path, "operand"))?,
                    )
                })
                .transpose()?
                .into(),
            origin: field_2
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::InputOrigin,
                        >(value, &child_path(path, "origin"))?,
                    )
                })
                .transpose()?
                .into(),
            cause: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticRecoveryCause,
                >(field_3, &child_path(path, "cause"))?,
            ),
            next: field_4
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NextAction,
                        >(value, &child_path(path, "next"))?,
                    )
                })
                .transpose()?
                .into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticRecovery,
> for crate::envelope::RecoveryRecord {
    fn from_wire(
        wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticRecovery,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::envelope::RecoveryRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::envelope::RecoveryRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "operation"),
                (2, "operand"),
                (3, "origin"),
                (4, "cause"),
                (5, "next"),
            ],
            path,
        )?;
        let fields = (
            native_from::<_, String>(wire.operation, &child_path(path, "operation"))?,
            wire
                .operand
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NativePath,
                        >(value, &child_path(path, "operand"))?,
                    )
                })
                .transpose()?,
            wire
                .origin
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::InputOrigin,
                        >(value, &child_path(path, "origin"))?,
                    )
                })
                .transpose()?,
            native_from::<
                _,
                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticRecoveryCause,
            >(
                required(wire.cause.into_option(), &child_path(path, "cause"))?,
                &child_path(path, "cause"),
            )?,
            wire
                .next
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NextAction,
                        >(value, &child_path(path, "next"))?,
                    )
                })
                .transpose()?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticRecoveryCause,
> for crate::envelope::RecoveryCause {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticRecoveryCause,
        AdapterError,
    > {
        let value = self;
        layout(
            <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticRecoveryCause {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::diagnostic_recovery_cause::Kind::MissingRoot(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::diagnostic_recovery_cause::Alternative1 {
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
            <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticRecoveryCause {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::diagnostic_recovery_cause::Kind::SourceDisappeared(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::diagnostic_recovery_cause::Alternative2 {
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
            <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::VERSION,
            <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
            3,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticRecoveryCause {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::diagnostic_recovery_cause::Kind::PermissionDenied(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::diagnostic_recovery_cause::Alternative3 {
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
            <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::VERSION,
            <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
            4,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticRecoveryCause {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::diagnostic_recovery_cause::Kind::SourceChanged(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::diagnostic_recovery_cause::Alternative4 {
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
            <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                5,
            >>::VERSION,
            <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                5,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
            5,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticRecoveryCause {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::diagnostic_recovery_cause::Kind::InputOutput(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::diagnostic_recovery_cause::Alternative5 {
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
            <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                6,
            >>::VERSION,
            <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                6,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
            6,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Native");
                let path = alternative_path.as_str();
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticRecoveryCause {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::diagnostic_recovery_cause::Kind::Native(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::diagnostic_recovery_cause::Alternative6 {
                                value: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::FailureReason,
                                    >(field_0, &child_path(path, "value"))?,
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
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticRecoveryCause,
> for crate::envelope::RecoveryCause {
    fn from_wire(
        wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticRecoveryCause,
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
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::diagnostic_recovery_cause::Kind::MissingRoot(
                wire,
            ) => {
                let alternative_path = child_path(path, "MissingRoot");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::diagnostic_recovery_cause::Kind::SourceDisappeared(
                wire,
            ) => {
                let alternative_path = child_path(path, "SourceDisappeared");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::diagnostic_recovery_cause::Kind::PermissionDenied(
                wire,
            ) => {
                let alternative_path = child_path(path, "PermissionDenied");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::diagnostic_recovery_cause::Kind::SourceChanged(
                wire,
            ) => {
                let alternative_path = child_path(path, "SourceChanged");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::diagnostic_recovery_cause::Kind::InputOutput(
                wire,
            ) => {
                let alternative_path = child_path(path, "InputOutput");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::VERSION,
                    <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::diagnostic_recovery_cause::Kind::Native(
                wire,
            ) => {
                let alternative_path = child_path(path, "Native");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::VERSION,
                    <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::FailureReason,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <crate::envelope::RecoveryCause as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ErrorRecord,
> for crate::envelope::ErrorRecord {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ErrorRecord,
        AdapterError,
    > {
        layout(
            <crate::envelope::ErrorRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::envelope::ErrorRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "version"),
                (2, "message"),
                (3, "code"),
                (4, "class"),
                (5, "severity"),
                (6, "help"),
                (7, "url"),
                (8, "facts"),
                (9, "children"),
                (10, "causes"),
                (11, "diagnostic_source"),
                (12, "related"),
                (13, "labels"),
                (14, "parser_context"),
                (15, "recovery"),
            ],
            path,
        )?;
        let (
            field_0,
            field_1,
            field_2,
            field_3,
            field_4,
            field_5,
            field_6,
            field_7,
            field_8,
            field_9,
            field_10,
            field_11,
            field_12,
            field_13,
            field_14,
        ) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ErrorRecord {
            version: native_into::<_, u32>(field_0, &child_path(path, "version"))?,
            message: native_into::<_, String>(field_1, &child_path(path, "message"))?,
            code: field_2
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_into::<_, String>(value, &child_path(path, "code"))?)
                })
                .transpose()?,
            class: field_3
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_into::<_, String>(value, &child_path(path, "class"))?)
                })
                .transpose()?,
            severity: field_4
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_into::<_, String>(value, &child_path(path, "severity"))?)
                })
                .transpose()?,
            help: field_5
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_into::<_, String>(value, &child_path(path, "help"))?)
                })
                .transpose()?,
            url: field_6
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_into::<_, String>(value, &child_path(path, "url"))?)
                })
                .transpose()?,
            facts: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticFacts,
                >(field_7, &child_path(path, "facts"))?,
            ),
            children: {
                let current_path = &child_path(path, "children");
                items(field_8)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ErrorRecord,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            causes: {
                let current_path = &child_path(path, "causes");
                items(field_9)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                String,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            diagnostic_source: field_10
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ErrorRecord,
                        >(value, &child_path(path, "diagnostic_source"))?,
                    )
                })
                .transpose()?
                .into(),
            related: {
                let current_path = &child_path(path, "related");
                items(field_11)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ErrorRecord,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            labels: {
                let current_path = &child_path(path, "labels");
                items(field_12)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticLabel,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            parser_context: wire_map(
                field_13
                    .into_iter()
                    .map(|(key, value)| -> Result<_, AdapterError> {
                        Ok((
                            native_into::<
                                _,
                                String,
                            >(key, &child_path(path, "parser_context"))?,
                            native_into::<
                                _,
                                String,
                            >(value, &child_path(path, "parser_context"))?,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter(),
            ),
            recovery: field_14
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticRecovery,
                        >(value, &child_path(path, "recovery"))?,
                    )
                })
                .transpose()?
                .into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ErrorRecord,
> for crate::envelope::ErrorRecord {
    fn from_wire(
        wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ErrorRecord,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::envelope::ErrorRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::envelope::ErrorRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "version"),
                (2, "message"),
                (3, "code"),
                (4, "class"),
                (5, "severity"),
                (6, "help"),
                (7, "url"),
                (8, "facts"),
                (9, "children"),
                (10, "causes"),
                (11, "diagnostic_source"),
                (12, "related"),
                (13, "labels"),
                (14, "parser_context"),
                (15, "recovery"),
            ],
            path,
        )?;
        let fields = (
            native_from::<_, u32>(wire.version, &child_path(path, "version"))?,
            native_from::<_, String>(wire.message, &child_path(path, "message"))?,
            wire
                .code
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_from::<_, String>(value, &child_path(path, "code"))?)
                })
                .transpose()?,
            wire
                .class
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_from::<_, String>(value, &child_path(path, "class"))?)
                })
                .transpose()?,
            wire
                .severity
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_from::<_, String>(value, &child_path(path, "severity"))?)
                })
                .transpose()?,
            wire
                .help
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_from::<_, String>(value, &child_path(path, "help"))?)
                })
                .transpose()?,
            wire
                .url
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_from::<_, String>(value, &child_path(path, "url"))?)
                })
                .transpose()?,
            native_from::<
                _,
                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticFacts,
            >(
                required(wire.facts.into_option(), &child_path(path, "facts"))?,
                &child_path(path, "facts"),
            )?,
            {
                let current_path = &child_path(path, "children");
                let values = wire
                    .children
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ErrorRecord,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "causes");
                let values = wire
                    .causes
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                String,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            wire
                .diagnostic_source
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ErrorRecord,
                        >(value, &child_path(path, "diagnostic_source"))?,
                    )
                })
                .transpose()?,
            {
                let current_path = &child_path(path, "related");
                let values = wire
                    .related
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ErrorRecord,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "labels");
                let values = wire
                    .labels
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticLabel,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            unique_pairs(
                wire
                    .parser_context
                    .into_iter()
                    .map(|(key, value)| -> Result<_, AdapterError> {
                        Ok((
                            native_from::<
                                _,
                                String,
                            >(key, &child_path(path, "parser_context"))?,
                            native_from::<
                                _,
                                String,
                            >(value, &child_path(path, "parser_context"))?,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                &child_path(path, "parser_context"),
            )?,
            wire
                .recovery
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::DiagnosticRecovery,
                        >(value, &child_path(path, "recovery"))?,
                    )
                })
                .transpose()?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::FailureReason,
> for crate::semantic::FailureReason {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::FailureReason,
        AdapterError,
    > {
        let value = self;
        layout(
            <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::FailureReason {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::failure_reason::Kind::ToolUnavailable(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_reason::Alternative1 {
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
            <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::FailureReason {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::failure_reason::Kind::SessionUnavailable(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_reason::Alternative2 {
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
            <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::VERSION,
            <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
            3,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::FailureReason {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::failure_reason::Kind::ConfigurationChanged(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_reason::Alternative3 {
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
            <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::VERSION,
            <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
            4,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::FailureReason {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::failure_reason::Kind::SourceChanged(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_reason::Alternative4 {
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
            <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                5,
            >>::VERSION,
            <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                5,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
            5,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::FailureReason {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::failure_reason::Kind::OutsidePreparedWorld(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_reason::Alternative5 {
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
            <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                6,
            >>::VERSION,
            <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                6,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
            6,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::FailureReason {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::failure_reason::Kind::InactiveSource(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_reason::Alternative6 {
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
            <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                7,
            >>::VERSION,
            <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                7,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
            7,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::FailureReason {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::failure_reason::Kind::OriginalMappingUnavailable(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_reason::Alternative7 {
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
            <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                8,
            >>::VERSION,
            <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                8,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
            8,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::FailureReason {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::failure_reason::Kind::UnsupportedFact(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_reason::Alternative8 {
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
            <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                9,
            >>::VERSION,
            <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                9,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
            9,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::FailureReason {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::failure_reason::Kind::ProviderFailed(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::failure_reason::Alternative9 {
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
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::FailureReason,
> for crate::semantic::FailureReason {
    fn from_wire(
        wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::FailureReason,
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
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::failure_reason::Kind::ToolUnavailable(
                wire,
            ) => {
                let alternative_path = child_path(path, "ToolUnavailable");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::failure_reason::Kind::SessionUnavailable(
                wire,
            ) => {
                let alternative_path = child_path(path, "SessionUnavailable");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::failure_reason::Kind::ConfigurationChanged(
                wire,
            ) => {
                let alternative_path = child_path(path, "ConfigurationChanged");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::failure_reason::Kind::SourceChanged(
                wire,
            ) => {
                let alternative_path = child_path(path, "SourceChanged");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::failure_reason::Kind::OutsidePreparedWorld(
                wire,
            ) => {
                let alternative_path = child_path(path, "OutsidePreparedWorld");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::VERSION,
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::failure_reason::Kind::InactiveSource(
                wire,
            ) => {
                let alternative_path = child_path(path, "InactiveSource");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::VERSION,
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::failure_reason::Kind::OriginalMappingUnavailable(
                wire,
            ) => {
                let alternative_path = child_path(path, "OriginalMappingUnavailable");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::VERSION,
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::failure_reason::Kind::UnsupportedFact(
                wire,
            ) => {
                let alternative_path = child_path(path, "UnsupportedFact");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::VERSION,
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::failure_reason::Kind::ProviderFailed(
                wire,
            ) => {
                let alternative_path = child_path(path, "ProviderFailed");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::VERSION,
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::semantic::FailureReason as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::InputOrigin,
> for crate::investigation::config::InputOrigin {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::InputOrigin,
        AdapterError,
    > {
        layout(
            <crate::investigation::config::InputOrigin as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::investigation::config::InputOrigin as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "provider"), (2, "source")],
            path,
        )?;
        let (field_0, field_1) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::InputOrigin {
            provider: native_into::<_, String>(field_0, &child_path(path, "provider"))?,
            source: field_1
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NativePath,
                        >(value, &child_path(path, "source"))?,
                    )
                })
                .transpose()?
                .into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::InputOrigin,
> for crate::investigation::config::InputOrigin {
    fn from_wire(
        wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::InputOrigin,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::investigation::config::InputOrigin as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::investigation::config::InputOrigin as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "provider"), (2, "source")],
            path,
        )?;
        let fields = (
            native_from::<_, String>(wire.provider, &child_path(path, "provider"))?,
            wire
                .source
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NativePath,
                        >(value, &child_path(path, "source"))?,
                    )
                })
                .transpose()?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::InventoryContent,
> for crate::test_workflow::execute::InventoryContent {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::InventoryContent,
        AdapterError,
    > {
        layout(
            <crate::test_workflow::execute::InventoryContent as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::test_workflow::execute::InventoryContent as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "sha256"), (2, "bytes")],
            path,
        )?;
        let (field_0, field_1) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::InventoryContent {
            sha256: native_into::<_, String>(field_0, &child_path(path, "sha256"))?,
            bytes: native_into::<_, u64>(field_1, &child_path(path, "bytes"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::InventoryContent,
> for crate::test_workflow::execute::InventoryContent {
    fn from_wire(
        wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::InventoryContent,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::test_workflow::execute::InventoryContent as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::test_workflow::execute::InventoryContent as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "sha256"), (2, "bytes")],
            path,
        )?;
        let fields = (
            native_from::<_, String>(wire.sha256, &child_path(path, "sha256"))?,
            native_from::<_, u64>(wire.bytes, &child_path(path, "bytes"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ManagerSettlement,
> for crate::envelope::Settlement {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ManagerSettlement,
        AdapterError,
    > {
        let value = self;
        layout(
            <crate::envelope::Settlement as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <crate::envelope::Settlement as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::envelope::Settlement as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ManagerSettlement {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::manager_settlement::Kind::NotRequired(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::manager_settlement::Alternative1 {
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
            <crate::envelope::Settlement as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <crate::envelope::Settlement as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::envelope::Settlement as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ManagerSettlement {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::manager_settlement::Kind::Complete(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::manager_settlement::Alternative2 {
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
            <crate::envelope::Settlement as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::VERSION,
            <crate::envelope::Settlement as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::envelope::Settlement as pse_contract::boundary_projection::ProjectionAlternative<
            3,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ManagerSettlement {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::manager_settlement::Kind::Incomplete(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::manager_settlement::Alternative3 {
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
            <crate::envelope::Settlement as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::VERSION,
            <crate::envelope::Settlement as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::envelope::Settlement as pse_contract::boundary_projection::ProjectionAlternative<
            4,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ManagerSettlement {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::manager_settlement::Kind::Unknown(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::manager_settlement::Alternative4 {
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
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ManagerSettlement,
> for crate::envelope::Settlement {
    fn from_wire(
        wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ManagerSettlement,
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
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::manager_settlement::Kind::NotRequired(
                wire,
            ) => {
                let alternative_path = child_path(path, "NotRequired");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::envelope::Settlement as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <crate::envelope::Settlement as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::envelope::Settlement as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::manager_settlement::Kind::Complete(
                wire,
            ) => {
                let alternative_path = child_path(path, "Complete");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::envelope::Settlement as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <crate::envelope::Settlement as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::envelope::Settlement as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::manager_settlement::Kind::Incomplete(
                wire,
            ) => {
                let alternative_path = child_path(path, "Incomplete");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::envelope::Settlement as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <crate::envelope::Settlement as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::envelope::Settlement as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::manager_settlement::Kind::Unknown(
                wire,
            ) => {
                let alternative_path = child_path(path, "Unknown");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::envelope::Settlement as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <crate::envelope::Settlement as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::envelope::Settlement as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NativePath,
> for crate::investigation::NativePath {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NativePath,
        AdapterError,
    > {
        let value = self;
        layout(
            <crate::investigation::NativePath as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <crate::investigation::NativePath as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::investigation::NativePath as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Utf8");
                let path = alternative_path.as_str();
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NativePath {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::native_path::Kind::Utf8(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::native_path::Alternative1 {
                                value: native_into::<
                                    _,
                                    String,
                                >(field_0, &child_path(path, "value"))?,
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
            <crate::investigation::NativePath as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <crate::investigation::NativePath as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "unix")],
            path,
        )?;
        let value = match <crate::investigation::NativePath as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NativePath {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::native_path::Kind::Raw(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::native_path::Alternative2 {
                                unix: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
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
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NativePath,
> for crate::investigation::NativePath {
    fn from_wire(
        wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NativePath,
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
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::native_path::Kind::Utf8(
                wire,
            ) => {
                let alternative_path = child_path(path, "Utf8");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::investigation::NativePath as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <crate::investigation::NativePath as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<_, String>(wire.value, &child_path(path, "value"))?,
                );
                Ok(
                    <crate::investigation::NativePath as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::native_path::Kind::Raw(
                wire,
            ) => {
                let alternative_path = child_path(path, "Raw");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::investigation::NativePath as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <crate::investigation::NativePath as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "unix")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                        wire.unix,
                        &child_path(path, "unix"),
                    )?,
                );
                Ok(
                    <crate::investigation::NativePath as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NativeWorkflowReport,
> for &crate::test_workflow::execute::Report {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NativeWorkflowReport,
        AdapterError,
    > {
        layout(
            <crate::test_workflow::execute::Report as pse_contract::boundary_projection::ProjectionObservationRecord>::VERSION,
            <crate::test_workflow::execute::Report as pse_contract::boundary_projection::ProjectionObservationRecord>::FIELD_LAYOUT,
            1,
            &[(1, "observation")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionObservationRecord::observation_fields(
                self,
            )
            .map_err(|error| error.at(path))?;
        Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NativeWorkflowReport {
            observation: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowReportObservation,
                >(field_0, &child_path(path, "observation"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeInto<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NextAction,
> for crate::envelope::NextAction {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NextAction,
        AdapterError,
    > {
        let value = self;
        layout(
            <crate::envelope::NextAction as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <crate::envelope::NextAction as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "argv"), (2, "reason")],
            path,
        )?;
        let value = match <crate::envelope::NextAction as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "Command");
                let path = alternative_path.as_str();
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NextAction {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::next_action::Kind::Command(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::next_action::Alternative1 {
                                argv: {
                                    let current_path = &child_path(path, "argv");
                                    items(field_0)
                                        .enumerate()
                                        .map(|(index, value)| -> Result<_, AdapterError> {
                                            Ok(
                                                native_into::<
                                                    _,
                                                    String,
                                                >(value, &format!("{path}[{index}]", path = current_path))?,
                                            )
                                        })
                                        .collect::<Result<Vec<_>, _>>()?
                                },
                                reason: native_into::<
                                    _,
                                    String,
                                >(field_1, &child_path(path, "reason"))?,
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
            <crate::envelope::NextAction as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <crate::envelope::NextAction as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "location"), (2, "reason")],
            path,
        )?;
        let value = match <crate::envelope::NextAction as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "ReadArtifact");
                let path = alternative_path.as_str();
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NextAction {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::next_action::Kind::ReadArtifact(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::next_action::Alternative2 {
                                location: native_into::<
                                    _,
                                    String,
                                >(field_0, &child_path(path, "location"))?,
                                reason: native_into::<
                                    _,
                                    String,
                                >(field_1, &child_path(path, "reason"))?,
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
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NextAction,
> for crate::envelope::NextAction {
    fn from_wire(
        wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NextAction,
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
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::next_action::Kind::Command(
                wire,
            ) => {
                let alternative_path = child_path(path, "Command");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::envelope::NextAction as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <crate::envelope::NextAction as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "argv"), (2, "reason")],
                    path,
                )?;
                let fields = (
                    {
                        let current_path = &child_path(path, "argv");
                        let values = wire
                            .argv
                            .into_iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                Ok(
                                    native_from::<
                                        _,
                                        String,
                                    >(value, &format!("{path}[{index}]", path = current_path))?,
                                )
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                    native_from::<_, String>(wire.reason, &child_path(path, "reason"))?,
                );
                Ok(
                    <crate::envelope::NextAction as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::next_action::Kind::ReadArtifact(
                wire,
            ) => {
                let alternative_path = child_path(path, "ReadArtifact");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::envelope::NextAction as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <crate::envelope::NextAction as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "location"), (2, "reason")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        String,
                    >(wire.location, &child_path(path, "location"))?,
                    native_from::<_, String>(wire.reason, &child_path(path, "reason"))?,
                );
                Ok(
                    <crate::envelope::NextAction as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NonFinite,
> for pse_diagnostics::NonFinite {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NonFinite,
        AdapterError,
    > {
        Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NonFinite {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NonFinite,
> for pse_diagnostics::NonFinite {
    fn from_wire(
        wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NonFinite,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        wire.value
            .parse::<Self>()
            .map_err(|_| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "unknown canonical source vocabulary member",
            ))
    }
}
impl NativeInto<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::TestIdentity,
> for crate::discovery::TestIdentity {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::TestIdentity,
        AdapterError,
    > {
        layout(
            <crate::discovery::TestIdentity as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::discovery::TestIdentity as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "group"), (2, "package_id"), (3, "binary_id"), (4, "test_name")],
            path,
        )?;
        let (field_0, field_1, field_2, field_3) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::TestIdentity {
            group: native_into::<_, u64>(field_0, &child_path(path, "group"))?,
            package_id: native_into::<
                _,
                String,
            >(field_1, &child_path(path, "package_id"))?,
            binary_id: native_into::<
                _,
                String,
            >(field_2, &child_path(path, "binary_id"))?,
            test_name: native_into::<
                _,
                String,
            >(field_3, &child_path(path, "test_name"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::TestIdentity,
> for crate::discovery::TestIdentity {
    fn from_wire(
        wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::TestIdentity,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::discovery::TestIdentity as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::discovery::TestIdentity as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "group"), (2, "package_id"), (3, "binary_id"), (4, "test_name")],
            path,
        )?;
        let fields = (
            native_from::<_, u64>(wire.group, &child_path(path, "group"))?,
            native_from::<_, String>(wire.package_id, &child_path(path, "package_id"))?,
            native_from::<_, String>(wire.binary_id, &child_path(path, "binary_id"))?,
            native_from::<_, String>(wire.test_name, &child_path(path, "test_name"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowAction,
> for crate::test_workflow::execute::Action {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowAction,
        AdapterError,
    > {
        let value = self;
        layout(
            <crate::test_workflow::execute::Action as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <crate::test_workflow::execute::Action as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::test_workflow::execute::Action as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowAction {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::workflow_action::Kind::List(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_action::Alternative1 {
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
            <crate::test_workflow::execute::Action as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <crate::test_workflow::execute::Action as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::test_workflow::execute::Action as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowAction {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::workflow_action::Kind::Test(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_action::Alternative2 {
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
            <crate::test_workflow::execute::Action as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::VERSION,
            <crate::test_workflow::execute::Action as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::test_workflow::execute::Action as pse_contract::boundary_projection::ProjectionAlternative<
            3,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowAction {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::workflow_action::Kind::Check(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_action::Alternative3 {
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
            <crate::test_workflow::execute::Action as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::VERSION,
            <crate::test_workflow::execute::Action as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::test_workflow::execute::Action as pse_contract::boundary_projection::ProjectionAlternative<
            4,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowAction {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::workflow_action::Kind::Rustdoc(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_action::Alternative4 {
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
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowAction,
> for crate::test_workflow::execute::Action {
    fn from_wire(
        wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowAction,
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
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::workflow_action::Kind::List(
                wire,
            ) => {
                let alternative_path = child_path(path, "List");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::test_workflow::execute::Action as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <crate::test_workflow::execute::Action as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::test_workflow::execute::Action as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::workflow_action::Kind::Test(
                wire,
            ) => {
                let alternative_path = child_path(path, "Test");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::test_workflow::execute::Action as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <crate::test_workflow::execute::Action as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::test_workflow::execute::Action as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::workflow_action::Kind::Check(
                wire,
            ) => {
                let alternative_path = child_path(path, "Check");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::test_workflow::execute::Action as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <crate::test_workflow::execute::Action as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::test_workflow::execute::Action as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::workflow_action::Kind::Rustdoc(
                wire,
            ) => {
                let alternative_path = child_path(path, "Rustdoc");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::test_workflow::execute::Action as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <crate::test_workflow::execute::Action as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::test_workflow::execute::Action as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowOutcome,
> for crate::test_workflow::execute::Outcome {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowOutcome,
        AdapterError,
    > {
        let value = self;
        layout(
            <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowOutcome {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::workflow_outcome::Kind::Listed(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_outcome::Alternative1 {
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
            <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowOutcome {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::workflow_outcome::Kind::CheckSucceeded(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_outcome::Alternative2 {
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
            <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::VERSION,
            <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
            3,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowOutcome {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::workflow_outcome::Kind::TestsSucceeded(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_outcome::Alternative3 {
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
            <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::VERSION,
            <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
            4,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowOutcome {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::workflow_outcome::Kind::Failed(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_outcome::Alternative4 {
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
            <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                5,
            >>::VERSION,
            <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                5,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
            5,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowOutcome {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::workflow_outcome::Kind::ZeroMatch(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_outcome::Alternative5 {
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
            <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                6,
            >>::VERSION,
            <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                6,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
            6,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowOutcome {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::workflow_outcome::Kind::Pending(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_outcome::Alternative6 {
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
            <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                7,
            >>::VERSION,
            <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                7,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
            7,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowOutcome {
                    kind: Some(
                        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::workflow_outcome::Kind::Incomplete(
                            Box::new(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::workflow_outcome::Alternative7 {
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
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowOutcome,
> for crate::test_workflow::execute::Outcome {
    fn from_wire(
        wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowOutcome,
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
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::workflow_outcome::Kind::Listed(
                wire,
            ) => {
                let alternative_path = child_path(path, "Listed");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::workflow_outcome::Kind::CheckSucceeded(
                wire,
            ) => {
                let alternative_path = child_path(path, "CheckSucceeded");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::workflow_outcome::Kind::TestsSucceeded(
                wire,
            ) => {
                let alternative_path = child_path(path, "TestsSucceeded");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::workflow_outcome::Kind::Failed(
                wire,
            ) => {
                let alternative_path = child_path(path, "Failed");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::workflow_outcome::Kind::ZeroMatch(
                wire,
            ) => {
                let alternative_path = child_path(path, "ZeroMatch");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::VERSION,
                    <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::workflow_outcome::Kind::Pending(
                wire,
            ) => {
                let alternative_path = child_path(path, "Pending");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::VERSION,
                    <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::__buffa::oneof::workflow_outcome::Kind::Incomplete(
                wire,
            ) => {
                let alternative_path = child_path(path, "Incomplete");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::VERSION,
                    <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::test_workflow::execute::Outcome as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowReportObservation,
> for crate::test_workflow::execute::ReportObservation {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowReportObservation,
        AdapterError,
    > {
        layout(
            <crate::test_workflow::execute::ReportObservation as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::test_workflow::execute::ReportObservation as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "version"),
                (2, "invocation"),
                (3, "action"),
                (4, "outcome"),
                (5, "inventory"),
                (6, "selected_tests"),
                (7, "required_capture_complete"),
                (8, "failures"),
                (9, "inventory_content"),
            ],
            path,
        )?;
        let (
            field_0,
            field_1,
            field_2,
            field_3,
            field_4,
            field_5,
            field_6,
            field_7,
            field_8,
        ) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowReportObservation {
            version: native_into::<_, u32>(field_0, &child_path(path, "version"))?,
            invocation: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::CliInvocationId,
                >(field_1, &child_path(path, "invocation"))?,
            ),
            action: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowAction,
                >(field_2, &child_path(path, "action"))?,
            ),
            outcome: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowOutcome,
                >(field_3, &child_path(path, "outcome"))?,
            ),
            inventory: field_4
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_into::<_, String>(value, &child_path(path, "inventory"))?)
                })
                .transpose()?,
            selected_tests: field_5
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            u64,
                        >(value, &child_path(path, "selected_tests"))?,
                    )
                })
                .transpose()?,
            required_capture_complete: native_into::<
                _,
                bool,
            >(field_6, &child_path(path, "required_capture_complete"))?,
            failures: {
                let current_path = &child_path(path, "failures");
                items(field_7)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ErrorRecord,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            inventory_content: field_8
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::InventoryContent,
                        >(value, &child_path(path, "inventory_content"))?,
                    )
                })
                .transpose()?
                .into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowReportObservation,
> for crate::test_workflow::execute::ReportObservation {
    fn from_wire(
        wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowReportObservation,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::test_workflow::execute::ReportObservation as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::test_workflow::execute::ReportObservation as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "version"),
                (2, "invocation"),
                (3, "action"),
                (4, "outcome"),
                (5, "inventory"),
                (6, "selected_tests"),
                (7, "required_capture_complete"),
                (8, "failures"),
                (9, "inventory_content"),
            ],
            path,
        )?;
        let fields = (
            native_from::<_, u32>(wire.version, &child_path(path, "version"))?,
            native_from::<
                _,
                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::CliInvocationId,
            >(
                required(
                    wire.invocation.into_option(),
                    &child_path(path, "invocation"),
                )?,
                &child_path(path, "invocation"),
            )?,
            native_from::<
                _,
                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowAction,
            >(
                required(wire.action.into_option(), &child_path(path, "action"))?,
                &child_path(path, "action"),
            )?,
            native_from::<
                _,
                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowOutcome,
            >(
                required(wire.outcome.into_option(), &child_path(path, "outcome"))?,
                &child_path(path, "outcome"),
            )?,
            wire
                .inventory
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_from::<_, String>(value, &child_path(path, "inventory"))?)
                })
                .transpose()?,
            wire
                .selected_tests
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            u64,
                        >(value, &child_path(path, "selected_tests"))?,
                    )
                })
                .transpose()?,
            native_from::<
                _,
                bool,
            >(
                wire.required_capture_complete,
                &child_path(path, "required_capture_complete"),
            )?,
            {
                let current_path = &child_path(path, "failures");
                let values = wire
                    .failures
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ErrorRecord,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            wire
                .inventory_content
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::InventoryContent,
                        >(value, &child_path(path, "inventory_content"))?,
                    )
                })
                .transpose()?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowReportReference,
> for crate::test_workflow::execute::ReportReference {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowReportReference,
        AdapterError,
    > {
        layout(
            <crate::test_workflow::execute::ReportReference as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::test_workflow::execute::ReportReference as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "path"), (2, "sha256"), (3, "bytes")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowReportReference {
            path: native_into::<_, String>(field_0, &child_path(path, "path"))?,
            sha256: native_into::<_, String>(field_1, &child_path(path, "sha256"))?,
            bytes: native_into::<_, u64>(field_2, &child_path(path, "bytes"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowReportReference,
> for crate::test_workflow::execute::ReportReference {
    fn from_wire(
        wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowReportReference,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::test_workflow::execute::ReportReference as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::test_workflow::execute::ReportReference as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "path"), (2, "sha256"), (3, "bytes")],
            path,
        )?;
        let fields = (
            native_from::<_, String>(wire.path, &child_path(path, "path"))?,
            native_from::<_, String>(wire.sha256, &child_path(path, "sha256"))?,
            native_from::<_, u64>(wire.bytes, &child_path(path, "bytes"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowTerminalRecord,
> for crate::test_workflow::execute::TerminalRecord {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowTerminalRecord,
        AdapterError,
    > {
        layout(
            <crate::test_workflow::execute::TerminalRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::test_workflow::execute::TerminalRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "protocol"),
                (2, "version"),
                (3, "invocation"),
                (4, "action"),
                (5, "outcome"),
                (6, "report"),
                (7, "required_capture_complete"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5, field_6) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowTerminalRecord {
            protocol: native_into::<_, String>(field_0, &child_path(path, "protocol"))?,
            version: native_into::<_, u32>(field_1, &child_path(path, "version"))?,
            invocation: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::CliInvocationId,
                >(field_2, &child_path(path, "invocation"))?,
            ),
            action: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowAction,
                >(field_3, &child_path(path, "action"))?,
            ),
            outcome: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowOutcome,
                >(field_4, &child_path(path, "outcome"))?,
            ),
            report: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowReportReference,
                >(field_5, &child_path(path, "report"))?,
            ),
            required_capture_complete: native_into::<
                _,
                bool,
            >(field_6, &child_path(path, "required_capture_complete"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowTerminalRecord,
> for crate::test_workflow::execute::TerminalRecord {
    fn from_wire(
        wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowTerminalRecord,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::test_workflow::execute::TerminalRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::test_workflow::execute::TerminalRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "protocol"),
                (2, "version"),
                (3, "invocation"),
                (4, "action"),
                (5, "outcome"),
                (6, "report"),
                (7, "required_capture_complete"),
            ],
            path,
        )?;
        let fields = (
            native_from::<_, String>(wire.protocol, &child_path(path, "protocol"))?,
            native_from::<_, u32>(wire.version, &child_path(path, "version"))?,
            native_from::<
                _,
                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::CliInvocationId,
            >(
                required(
                    wire.invocation.into_option(),
                    &child_path(path, "invocation"),
                )?,
                &child_path(path, "invocation"),
            )?,
            native_from::<
                _,
                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowAction,
            >(
                required(wire.action.into_option(), &child_path(path, "action"))?,
                &child_path(path, "action"),
            )?,
            native_from::<
                _,
                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowOutcome,
            >(
                required(wire.outcome.into_option(), &child_path(path, "outcome"))?,
                &child_path(path, "outcome"),
            )?,
            native_from::<
                _,
                crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowReportReference,
            >(
                required(wire.report.into_option(), &child_path(path, "report"))?,
                &child_path(path, "report"),
            )?,
            native_from::<
                _,
                bool,
            >(
                wire.required_capture_complete,
                &child_path(path, "required_capture_complete"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
/// Observe `NativeWorkflowReport` without consuming native custody or constructing native authority.
pub fn encode_native_workflow_report(
    native: &crate::test_workflow::execute::Report,
) -> Result<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NativeWorkflowReport,
    BoundaryProjectionError,
> {
    pse_contract::boundary_projection::ValidateProjection::validate_projection(native)
        .map_err(BoundaryProjectionError::OutgoingNative)?;
    let wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::NativeWorkflowReport = native_into(
            native,
            "NativeWorkflowReport",
        )
        .map_err(BoundaryProjectionError::OutgoingAdoption)?;
    protovalidate_buffa::Validate::validate(&wire)
        .map_err(BoundaryProjectionError::OutgoingWire)?;
    Ok(wire)
}
/// Observe `CapturedWorkflowReport` without consuming native custody or constructing native authority.
pub fn encode_captured_workflow_report(
    native: &crate::test_inventory::CapturedReport,
) -> Result<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::CapturedWorkflowReport,
    BoundaryProjectionError,
> {
    pse_contract::boundary_projection::ValidateProjection::validate_projection(native)
        .map_err(BoundaryProjectionError::OutgoingNative)?;
    let wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::CapturedWorkflowReport = native_into(
            native,
            "CapturedWorkflowReport",
        )
        .map_err(BoundaryProjectionError::OutgoingAdoption)?;
    protovalidate_buffa::Validate::validate(&wire)
        .map_err(BoundaryProjectionError::OutgoingWire)?;
    Ok(wire)
}
/// Observe `CapturedWorkflowEvidence` without consuming native custody or constructing native authority.
pub fn encode_captured_workflow_evidence(
    native: &crate::test_inventory::CapturedEvidence,
) -> Result<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::CapturedWorkflowEvidence,
    BoundaryProjectionError,
> {
    pse_contract::boundary_projection::ValidateProjection::validate_projection(native)
        .map_err(BoundaryProjectionError::OutgoingNative)?;
    let wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::CapturedWorkflowEvidence = native_into(
            native,
            "CapturedWorkflowEvidence",
        )
        .map_err(BoundaryProjectionError::OutgoingAdoption)?;
    protovalidate_buffa::Validate::validate(&wire)
        .map_err(BoundaryProjectionError::OutgoingWire)?;
    Ok(wire)
}
/// Encode the actual native `WorkflowReportObservation` and validate outgoing local rules.
pub fn encode_workflow_report_observation(
    native: crate::test_workflow::execute::ReportObservation,
) -> Result<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowReportObservation,
    BoundaryProjectionError,
> {
    encode_root(native, "WorkflowReportObservation")
}
/// Adopt known `WorkflowReportObservation` meaning under this package's declared unknown-field policy.
pub fn decode_workflow_report_observation(
    wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowReportObservation,
) -> Result<crate::test_workflow::execute::ReportObservation, BoundaryProjectionError> {
    decode_root(wire, "WorkflowReportObservation")
}
/// Encode the actual native `WorkflowTerminalRecord` and validate outgoing local rules.
pub fn encode_workflow_terminal_record(
    native: crate::test_workflow::execute::TerminalRecord,
) -> Result<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowTerminalRecord,
    BoundaryProjectionError,
> {
    encode_root(native, "WorkflowTerminalRecord")
}
/// Adopt known `WorkflowTerminalRecord` meaning under this package's declared unknown-field policy.
pub fn decode_workflow_terminal_record(
    wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::WorkflowTerminalRecord,
) -> Result<crate::test_workflow::execute::TerminalRecord, BoundaryProjectionError> {
    decode_root(wire, "WorkflowTerminalRecord")
}
/// Encode the actual native `TestIdentity` and validate outgoing local rules.
pub fn encode_test_identity(
    native: crate::discovery::TestIdentity,
) -> Result<
    crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::TestIdentity,
    BoundaryProjectionError,
> {
    encode_root(native, "TestIdentity")
}
/// Adopt known `TestIdentity` meaning under this package's declared unknown-field policy.
pub fn decode_test_identity(
    wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::TestIdentity,
) -> Result<crate::discovery::TestIdentity, BoundaryProjectionError> {
    decode_root(wire, "TestIdentity")
}
