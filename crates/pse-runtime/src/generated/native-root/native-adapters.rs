use super::{
    AdapterError, AdapterFailure, BoundaryProjectionError, NativeInto, NativeFrom,
    layout, child_path, refused, required, unknown_fields, native_into, native_from,
    encode_root, decode_root, items, from_items, unique_pairs, wire_map,
};
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityClaim,
> for pse_model::applicability::Claim {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityClaim,
        AdapterError,
    > {
        layout(
            <pse_model::applicability::Claim as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::applicability::Claim as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "id"),
                (2, "coverage"),
                (3, "owner"),
                (4, "owner_lineage"),
                (5, "evidence"),
                (6, "form"),
                (7, "call"),
                (8, "records"),
                (9, "dependencies"),
                (10, "layer"),
                (11, "basis"),
                (12, "reason"),
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
        ) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityClaim {
            id: field_0
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                            value,
                        ),
                    )
                })
                .transpose()?,
            coverage: field_1
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                            value,
                        ),
                    )
                })
                .transpose()?,
            owner: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_2,
            ),
            owner_lineage: items(field_3)
                .map(
                    pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes,
                )
                .collect::<Vec<_>>(),
            evidence: field_4
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                            value,
                        ),
                    )
                })
                .transpose()?,
            form: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_5,
            ),
            call: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_6,
            ),
            records: items(field_7)
                .map(
                    pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes,
                )
                .collect::<Vec<_>>(),
            dependencies: items(field_8)
                .map(
                    pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes,
                )
                .collect::<Vec<_>>(),
            layer: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::ModelingValidityLayer,
                >(field_9, &child_path(path, "layer"))?,
            ),
            basis: field_10
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ModelingApplicabilityBasis,
                        >(value, &child_path(path, "basis"))?,
                    )
                })
                .transpose()?
                .into(),
            reason: field_11
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_into::<_, String>(value, &child_path(path, "reason"))?)
                })
                .transpose()?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityClaim,
> for pse_model::applicability::Claim {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityClaim,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_model::applicability::Claim as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::applicability::Claim as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "id"),
                (2, "coverage"),
                (3, "owner"),
                (4, "owner_lineage"),
                (5, "evidence"),
                (6, "form"),
                (7, "call"),
                (8, "records"),
                (9, "dependencies"),
                (10, "layer"),
                (11, "basis"),
                (12, "reason"),
            ],
            path,
        )?;
        let fields = (
            wire
                .id
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                            value,
                            &child_path(path, "id"),
                        )?,
                    )
                })
                .transpose()?,
            wire
                .coverage
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                            value,
                            &child_path(path, "coverage"),
                        )?,
                    )
                })
                .transpose()?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.owner,
                &child_path(path, "owner"),
            )?,
            {
                let current_path = &child_path(path, "owner_lineage");
                let values = wire
                    .owner_lineage
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                                value,
                                &format!("{path}[{index}]", path = current_path),
                            )?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            wire
                .evidence
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                            value,
                            &child_path(path, "evidence"),
                        )?,
                    )
                })
                .transpose()?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.form,
                &child_path(path, "form"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.call,
                &child_path(path, "call"),
            )?,
            {
                let current_path = &child_path(path, "records");
                let values = wire
                    .records
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                                value,
                                &format!("{path}[{index}]", path = current_path),
                            )?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "dependencies");
                let values = wire
                    .dependencies
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                                value,
                                &format!("{path}[{index}]", path = current_path),
                            )?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::ModelingValidityLayer,
            >(
                required(wire.layer.into_option(), &child_path(path, "layer"))?,
                &child_path(path, "layer"),
            )?,
            wire
                .basis
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ModelingApplicabilityBasis,
                        >(value, &child_path(path, "basis"))?,
                    )
                })
                .transpose()?,
            wire
                .reason
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_from::<_, String>(value, &child_path(path, "reason"))?)
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
    crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityInput,
> for pse_model::applicability::Input {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityInput,
        AdapterError,
    > {
        layout(
            <pse_model::applicability::Input as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::applicability::Input as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "name"), (2, "value"), (3, "quantity_type")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityInput {
            name: native_into::<_, String>(field_0, &child_path(path, "name"))?,
            value: native_into::<_, u64>(field_1, &child_path(path, "value"))?,
            quantity_type: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_2,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityInput,
> for pse_model::applicability::Input {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityInput,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_model::applicability::Input as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::applicability::Input as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "name"), (2, "value"), (3, "quantity_type")],
            path,
        )?;
        let fields = (
            native_from::<_, String>(wire.name, &child_path(path, "name"))?,
            native_from::<_, u64>(wire.value, &child_path(path, "value"))?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.quantity_type,
                &child_path(path, "quantity_type"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityObservation,
> for pse_model::applicability::Observation {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityObservation,
        AdapterError,
    > {
        layout(
            <pse_model::applicability::Observation as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::applicability::Observation as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "claim"),
                (2, "instance"),
                (3, "inputs"),
                (4, "outcome"),
                (5, "required"),
                (6, "permissions"),
                (7, "unknown_allowed"),
                (8, "extrapolation_allowed"),
                (9, "admitted"),
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
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityObservation {
            claim: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityClaim,
                >(field_0, &child_path(path, "claim"))?,
            ),
            instance: field_1
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                            value,
                        ),
                    )
                })
                .transpose()?,
            inputs: {
                let current_path = &child_path(path, "inputs");
                items(field_2)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityInput,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            outcome: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::ModelingApplicabilityOutcome,
                >(field_3, &child_path(path, "outcome"))?,
            ),
            required: native_into::<_, bool>(field_4, &child_path(path, "required"))?,
            permissions: {
                let current_path = &child_path(path, "permissions");
                items(field_5)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityPermission,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            unknown_allowed: native_into::<
                _,
                bool,
            >(field_6, &child_path(path, "unknown_allowed"))?,
            extrapolation_allowed: native_into::<
                _,
                bool,
            >(field_7, &child_path(path, "extrapolation_allowed"))?,
            admitted: native_into::<_, bool>(field_8, &child_path(path, "admitted"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityObservation,
> for pse_model::applicability::Observation {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityObservation,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_model::applicability::Observation as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::applicability::Observation as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "claim"),
                (2, "instance"),
                (3, "inputs"),
                (4, "outcome"),
                (5, "required"),
                (6, "permissions"),
                (7, "unknown_allowed"),
                (8, "extrapolation_allowed"),
                (9, "admitted"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityClaim,
            >(
                required(wire.claim.into_option(), &child_path(path, "claim"))?,
                &child_path(path, "claim"),
            )?,
            wire
                .instance
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                            value,
                            &child_path(path, "instance"),
                        )?,
                    )
                })
                .transpose()?,
            {
                let current_path = &child_path(path, "inputs");
                let values = wire
                    .inputs
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityInput,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::ModelingApplicabilityOutcome,
            >(
                required(wire.outcome.into_option(), &child_path(path, "outcome"))?,
                &child_path(path, "outcome"),
            )?,
            native_from::<_, bool>(wire.required, &child_path(path, "required"))?,
            {
                let current_path = &child_path(path, "permissions");
                let values = wire
                    .permissions
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityPermission,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                bool,
            >(wire.unknown_allowed, &child_path(path, "unknown_allowed"))?,
            native_from::<
                _,
                bool,
            >(wire.extrapolation_allowed, &child_path(path, "extrapolation_allowed"))?,
            native_from::<_, bool>(wire.admitted, &child_path(path, "admitted"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityPermission,
> for pse_model::applicability::Permission {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityPermission,
        AdapterError,
    > {
        layout(
            <pse_model::applicability::Permission as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::applicability::Permission as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "id"),
                (2, "scope"),
                (3, "target_kind"),
                (4, "targets"),
                (5, "allow_unknown"),
                (6, "allow_extrapolation"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityPermission {
            id: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_0,
            ),
            scope: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_1,
            ),
            target_kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::ModelingPermissionTarget,
                >(field_2, &child_path(path, "target_kind"))?,
            ),
            targets: items(field_3)
                .map(
                    pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes,
                )
                .collect::<Vec<_>>(),
            allow_unknown: native_into::<
                _,
                bool,
            >(field_4, &child_path(path, "allow_unknown"))?,
            allow_extrapolation: native_into::<
                _,
                bool,
            >(field_5, &child_path(path, "allow_extrapolation"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityPermission,
> for pse_model::applicability::Permission {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityPermission,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_model::applicability::Permission as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::applicability::Permission as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "id"),
                (2, "scope"),
                (3, "target_kind"),
                (4, "targets"),
                (5, "allow_unknown"),
                (6, "allow_extrapolation"),
            ],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.id,
                &child_path(path, "id"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.scope,
                &child_path(path, "scope"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::ModelingPermissionTarget,
            >(
                required(
                    wire.target_kind.into_option(),
                    &child_path(path, "target_kind"),
                )?,
                &child_path(path, "target_kind"),
            )?,
            {
                let current_path = &child_path(path, "targets");
                let values = wire
                    .targets
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                                value,
                                &format!("{path}[{index}]", path = current_path),
                            )?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                bool,
            >(wire.allow_unknown, &child_path(path, "allow_unknown"))?,
            native_from::<
                _,
                bool,
            >(wire.allow_extrapolation, &child_path(path, "allow_extrapolation"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<crate::math::native_root_generated::pb::pse::native_root::v2::Array16U8>
for [u8; 16] {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::Array16U8,
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
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::Array16U8 {
            values: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_0,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<crate::math::native_root_generated::pb::pse::native_root::v2::Array16U8>
for [u8; 16] {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::Array16U8,
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
impl NativeInto<crate::math::native_root_generated::pb::pse::native_root::v2::Array2U64>
for [u64; 2] {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::Array2U64,
        AdapterError,
    > {
        layout(
            <[u64; 2] as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <[u64; 2] as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "values")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::Array2U64 {
            values: {
                let current_path = &child_path(path, "values");
                items(field_0)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                u64,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            ..Default::default()
        })
    }
}
impl NativeFrom<crate::math::native_root_generated::pb::pse::native_root::v2::Array2U64>
for [u64; 2] {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::Array2U64,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <[u64; 2] as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <[u64; 2] as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
                                u64,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::Array3Array16U8,
> for [[u8; 16]; 3] {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::Array3Array16U8,
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
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::Array3Array16U8 {
            values: {
                let current_path = &child_path(path, "values");
                items(field_0)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::Array16U8,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::Array3Array16U8,
> for [[u8; 16]; 3] {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::Array3Array16U8,
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
                                crate::math::native_root_generated::pb::pse::native_root::v2::Array16U8,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::Array8OptionalUnitId,
> for [Option<pse_quantity::ids::UnitId>; 8] {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::Array8OptionalUnitId,
        AdapterError,
    > {
        layout(
            <[Option<
                pse_quantity::ids::UnitId,
            >; 8] as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <[Option<
                pse_quantity::ids::UnitId,
            >; 8] as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "values")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::Array8OptionalUnitId {
            values: {
                let current_path = &child_path(path, "values");
                items(field_0)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::array8optional_unit_id::Field1Value {
                            value: value
                                .map(|value| -> Result<_, AdapterError> {
                                    Ok(
                                        native_into::<
                                            _,
                                            crate::math::native_root_generated::pb::pse::native_root::v2::UnitId,
                                        >(value, &format!("{path}[{index}]", path = current_path))?,
                                    )
                                })
                                .transpose()?
                                .into(),
                            ..Default::default()
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::Array8OptionalUnitId,
> for [Option<pse_quantity::ids::UnitId>; 8] {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::Array8OptionalUnitId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <[Option<
                pse_quantity::ids::UnitId,
            >; 8] as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <[Option<
                pse_quantity::ids::UnitId,
            >; 8] as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
                        Ok({
                            let wrapped = value;
                            unknown_fields(
                                &wrapped.__buffa_unknown_fields,
                                &format!("{path}[{index}]", path = current_path),
                            )?;
                            wrapped
                                .value
                                .into_option()
                                .map(|value| -> Result<_, AdapterError> {
                                    Ok(
                                        native_from::<
                                            _,
                                            crate::math::native_root_generated::pb::pse::native_root::v2::UnitId,
                                        >(value, &format!("{path}[{index}]", path = current_path))?,
                                    )
                                })
                                .transpose()?
                        })
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
    crate::math::native_root_generated::pb::pse::native_root::v2::Array8PhysicalRatio,
> for [pse_quantity::dimension::Ratio; 8] {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::Array8PhysicalRatio,
        AdapterError,
    > {
        layout(
            <[pse_quantity::dimension::Ratio; 8] as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <[pse_quantity::dimension::Ratio; 8] as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "values")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::Array8PhysicalRatio {
            values: {
                let current_path = &child_path(path, "values");
                items(field_0)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalRatio,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::Array8PhysicalRatio,
> for [pse_quantity::dimension::Ratio; 8] {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::Array8PhysicalRatio,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <[pse_quantity::dimension::Ratio; 8] as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <[pse_quantity::dimension::Ratio; 8] as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
                                crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalRatio,
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
impl NativeInto<crate::math::native_root_generated::pb::pse::native_root::v2::BasisId>
for pse_quantity::ids::BasisId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::BasisId,
        AdapterError,
    > {
        layout(
            <pse_quantity::ids::BasisId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::BasisId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::BasisId {
            id: Some(
                pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                    field_0,
                ),
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<crate::math::native_root_generated::pb::pse::native_root::v2::BasisId>
for pse_quantity::ids::BasisId {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::BasisId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::BasisId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::BasisId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                required(wire.id, &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<crate::math::native_root_generated::pb::pse::native_root::v2::BasisKind>
for pse_quantity::enums::BasisKind {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::BasisKind,
        AdapterError,
    > {
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::BasisKind {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<crate::math::native_root_generated::pb::pse::native_root::v2::BasisKind>
for pse_quantity::enums::BasisKind {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::BasisKind,
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
impl NativeInto<crate::math::native_root_generated::pb::pse::native_root::v2::BasisRule>
for pse_quantity::enums::BasisRule {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::BasisRule,
        AdapterError,
    > {
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::BasisRule {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<crate::math::native_root_generated::pb::pse::native_root::v2::BasisRule>
for pse_quantity::enums::BasisRule {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::BasisRule,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::CapturedBuildInfo,
> for pse_buildinfo::BuildInfo {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::CapturedBuildInfo,
        AdapterError,
    > {
        layout(
            <pse_buildinfo::BuildInfo as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_buildinfo::BuildInfo as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "version"),
                (2, "rustc_version"),
                (3, "profile"),
                (4, "git_sha"),
                (5, "lockfile_hash"),
                (6, "cargo_lock_sha256"),
                (7, "uv_lock_sha256"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5, field_6) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::CapturedBuildInfo {
            version: native_into::<_, String>(field_0, &child_path(path, "version"))?,
            rustc_version: native_into::<
                _,
                String,
            >(field_1, &child_path(path, "rustc_version"))?,
            profile: native_into::<_, String>(field_2, &child_path(path, "profile"))?,
            git_sha: native_into::<_, String>(field_3, &child_path(path, "git_sha"))?,
            lockfile_hash: native_into::<
                _,
                String,
            >(field_4, &child_path(path, "lockfile_hash"))?,
            cargo_lock_sha256: native_into::<
                _,
                String,
            >(field_5, &child_path(path, "cargo_lock_sha256"))?,
            uv_lock_sha256: native_into::<
                _,
                String,
            >(field_6, &child_path(path, "uv_lock_sha256"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::CapturedBuildInfo,
> for pse_buildinfo::BuildInfo {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::CapturedBuildInfo,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_buildinfo::BuildInfo as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_buildinfo::BuildInfo as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "version"),
                (2, "rustc_version"),
                (3, "profile"),
                (4, "git_sha"),
                (5, "lockfile_hash"),
                (6, "cargo_lock_sha256"),
                (7, "uv_lock_sha256"),
            ],
            path,
        )?;
        let fields = (
            native_from::<_, String>(wire.version, &child_path(path, "version"))?,
            native_from::<
                _,
                String,
            >(wire.rustc_version, &child_path(path, "rustc_version"))?,
            native_from::<_, String>(wire.profile, &child_path(path, "profile"))?,
            native_from::<_, String>(wire.git_sha, &child_path(path, "git_sha"))?,
            native_from::<
                _,
                String,
            >(wire.lockfile_hash, &child_path(path, "lockfile_hash"))?,
            native_from::<
                _,
                String,
            >(wire.cargo_lock_sha256, &child_path(path, "cargo_lock_sha256"))?,
            native_from::<
                _,
                String,
            >(wire.uv_lock_sha256, &child_path(path, "uv_lock_sha256"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::CompositionBasis,
> for pse_quantity::enums::CompositionBasis {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::CompositionBasis,
        AdapterError,
    > {
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::CompositionBasis {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::CompositionBasis,
> for pse_quantity::enums::CompositionBasis {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::CompositionBasis,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::ConversionId,
> for pse_quantity::ids::ConversionId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::ConversionId,
        AdapterError,
    > {
        layout(
            <pse_quantity::ids::ConversionId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::ConversionId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::ConversionId {
            id: Some(
                pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                    field_0,
                ),
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::ConversionId,
> for pse_quantity::ids::ConversionId {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::ConversionId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::ConversionId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::ConversionId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                required(wire.id, &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::ConversionKind,
> for pse_quantity::enums::ConversionKind {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::ConversionKind,
        AdapterError,
    > {
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::ConversionKind {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::ConversionKind,
> for pse_quantity::enums::ConversionKind {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::ConversionKind,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::DeclaredDefinedUnit,
> for pse_quantity::DefinedUnit {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::DeclaredDefinedUnit,
        AdapterError,
    > {
        layout(
            <pse_quantity::DefinedUnit as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::DefinedUnit as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "symbol"), (3, "composition")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::DeclaredDefinedUnit {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::UnitId,
                >(field_0, &child_path(path, "id"))?,
            ),
            symbol: native_into::<_, String>(field_1, &child_path(path, "symbol"))?,
            composition: {
                let current_path = &child_path(path, "composition");
                items(field_2)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::UnitFactor,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::DeclaredDefinedUnit,
> for pse_quantity::DefinedUnit {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::DeclaredDefinedUnit,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::DefinedUnit as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::DefinedUnit as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "symbol"), (3, "composition")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::UnitId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<_, String>(wire.symbol, &child_path(path, "symbol"))?,
            {
                let current_path = &child_path(path, "composition");
                let values = wire
                    .composition
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::UnitFactor,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::DeclaredDerivedKind,
> for pse_quantity::DerivedKind {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::DeclaredDerivedKind,
        AdapterError,
    > {
        layout(
            <pse_quantity::DerivedKind as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::DerivedKind as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "extensive"), (3, "addition_kind"), (4, "definition")],
            path,
        )?;
        let (field_0, field_1, field_2, field_3) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::DeclaredDerivedKind {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityKindId,
                >(field_0, &child_path(path, "id"))?,
            ),
            extensive: native_into::<_, bool>(field_1, &child_path(path, "extensive"))?,
            addition_kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityAdditionKind,
                >(field_2, &child_path(path, "addition_kind"))?,
            ),
            definition: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::KindDefinition,
                >(field_3, &child_path(path, "definition"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::DeclaredDerivedKind,
> for pse_quantity::DerivedKind {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::DeclaredDerivedKind,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::DerivedKind as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::DerivedKind as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "extensive"), (3, "addition_kind"), (4, "definition")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::QuantityKindId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<_, bool>(wire.extensive, &child_path(path, "extensive"))?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::QuantityAdditionKind,
            >(
                required(
                    wire.addition_kind.into_option(),
                    &child_path(path, "addition_kind"),
                )?,
                &child_path(path, "addition_kind"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::KindDefinition,
            >(
                required(
                    wire.definition.into_option(),
                    &child_path(path, "definition"),
                )?,
                &child_path(path, "definition"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticCode,
> for pse_diagnostics::DiagnosticCode {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticCode,
        AdapterError,
    > {
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticCode {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticCode,
> for pse_diagnostics::DiagnosticCode {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticCode,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticNonfiniteObservation,
> for pse_model::generated::enums::DiagnosticNonfiniteObservation {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticNonfiniteObservation,
        AdapterError,
    > {
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticNonfiniteObservation {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticNonfiniteObservation,
> for pse_model::generated::enums::DiagnosticNonfiniteObservation {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticNonfiniteObservation,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticObservation,
> for pse_model::diagnostic::Observation {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticObservation,
        AdapterError,
    > {
        let value = self;
        layout(
            <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticObservation {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::diagnostic_observation::Kind::Missing(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::diagnostic_observation::Alternative1 {
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
            <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Real");
                let path = alternative_path.as_str();
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticObservation {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::diagnostic_observation::Kind::Real(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::diagnostic_observation::Alternative2 {
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
            <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::VERSION,
            <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
            3,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Nonfinite");
                let path = alternative_path.as_str();
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticObservation {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::diagnostic_observation::Kind::Nonfinite(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::diagnostic_observation::Alternative3 {
                                value: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticNonfiniteObservation,
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
            <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::VERSION,
            <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
            4,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Physical");
                let path = alternative_path.as_str();
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticObservation {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::diagnostic_observation::Kind::Physical(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::diagnostic_observation::Alternative4 {
                                value: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticPhysicalObservation,
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
            <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                5,
            >>::VERSION,
            <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                5,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
            5,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Integer");
                let path = alternative_path.as_str();
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticObservation {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::diagnostic_observation::Kind::Integer(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::diagnostic_observation::Alternative5 {
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
            <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                6,
            >>::VERSION,
            <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                6,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
            6,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Boolean");
                let path = alternative_path.as_str();
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticObservation {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::diagnostic_observation::Kind::Boolean(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::diagnostic_observation::Alternative6 {
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
            <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                7,
            >>::VERSION,
            <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                7,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
            7,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Text");
                let path = alternative_path.as_str();
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticObservation {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::diagnostic_observation::Kind::Text(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::diagnostic_observation::Alternative7 {
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
            <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                8,
            >>::VERSION,
            <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                8,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
            8,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Contracts");
                let path = alternative_path.as_str();
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticObservation {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::diagnostic_observation::Kind::Contracts(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::diagnostic_observation::Alternative8 {
                                value: {
                                    let current_path = &child_path(path, "value");
                                    items(field_0)
                                        .enumerate()
                                        .map(|(index, value)| -> Result<_, AdapterError> {
                                            Ok(
                                                native_into::<
                                                    _,
                                                    crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticOperandContract,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticObservation,
> for pse_model::diagnostic::Observation {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticObservation,
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
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::diagnostic_observation::Kind::Missing(
                wire,
            ) => {
                let alternative_path = child_path(path, "Missing");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::diagnostic_observation::Kind::Real(
                wire,
            ) => {
                let alternative_path = child_path(path, "Real");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
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
                    <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::diagnostic_observation::Kind::Nonfinite(
                wire,
            ) => {
                let alternative_path = child_path(path, "Nonfinite");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticNonfiniteObservation,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::diagnostic_observation::Kind::Physical(
                wire,
            ) => {
                let alternative_path = child_path(path, "Physical");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticPhysicalObservation,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::diagnostic_observation::Kind::Integer(
                wire,
            ) => {
                let alternative_path = child_path(path, "Integer");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::VERSION,
                    <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<_, i64>(wire.value, &child_path(path, "value"))?,
                );
                Ok(
                    <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::diagnostic_observation::Kind::Boolean(
                wire,
            ) => {
                let alternative_path = child_path(path, "Boolean");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::VERSION,
                    <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<_, bool>(wire.value, &child_path(path, "value"))?,
                );
                Ok(
                    <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::diagnostic_observation::Kind::Text(
                wire,
            ) => {
                let alternative_path = child_path(path, "Text");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::VERSION,
                    <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<_, String>(wire.value, &child_path(path, "value"))?,
                );
                Ok(
                    <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::diagnostic_observation::Kind::Contracts(
                wire,
            ) => {
                let alternative_path = child_path(path, "Contracts");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::VERSION,
                    <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
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
                                        crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticOperandContract,
                                    >(value, &format!("{path}[{index}]", path = current_path))?,
                                )
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_model::diagnostic::Observation as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticOperandContract,
> for pse_diagnostics::OperandContract {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticOperandContract,
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
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticOperandContract {
            quantity: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::Array16U8,
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
                                crate::math::native_root_generated::pb::pse::native_root::v2::Array3Array16U8,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticOperandContract,
> for pse_diagnostics::OperandContract {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticOperandContract,
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
                crate::math::native_root_generated::pb::pse::native_root::v2::Array16U8,
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
                                crate::math::native_root_generated::pb::pse::native_root::v2::Array3Array16U8,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticOperationId,
> for pse_diagnostics::observation::OperationId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticOperationId,
        AdapterError,
    > {
        layout(
            <pse_diagnostics::observation::OperationId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_diagnostics::observation::OperationId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "process_instance"), (2, "sequence")],
            path,
        )?;
        let (field_0, field_1) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticOperationId {
            process_instance: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::Array2U64,
                >(field_0, &child_path(path, "process_instance"))?,
            ),
            sequence: native_into::<_, u64>(field_1, &child_path(path, "sequence"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticOperationId,
> for pse_diagnostics::observation::OperationId {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticOperationId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_diagnostics::observation::OperationId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_diagnostics::observation::OperationId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "process_instance"), (2, "sequence")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::Array2U64,
            >(
                required(
                    wire.process_instance.into_option(),
                    &child_path(path, "process_instance"),
                )?,
                &child_path(path, "process_instance"),
            )?,
            native_from::<_, u64>(wire.sequence, &child_path(path, "sequence"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticPhysicalObservation,
> for pse_model::diagnostic::PhysicalObservation {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticPhysicalObservation,
        AdapterError,
    > {
        layout(
            <pse_model::diagnostic::PhysicalObservation as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::diagnostic::PhysicalObservation as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "magnitude"), (2, "quantity"), (3, "unit"), (4, "context")],
            path,
        )?;
        let (field_0, field_1, field_2, field_3) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticPhysicalObservation {
            magnitude: native_into::<_, u64>(field_0, &child_path(path, "magnitude"))?,
            quantity: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_1,
            ),
            unit: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_2,
            ),
            context: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_3,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticPhysicalObservation,
> for pse_model::diagnostic::PhysicalObservation {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticPhysicalObservation,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_model::diagnostic::PhysicalObservation as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::diagnostic::PhysicalObservation as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "magnitude"), (2, "quantity"), (3, "unit"), (4, "context")],
            path,
        )?;
        let fields = (
            native_from::<_, u64>(wire.magnitude, &child_path(path, "magnitude"))?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.quantity,
                &child_path(path, "quantity"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.unit,
                &child_path(path, "unit"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.context,
                &child_path(path, "context"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticRule,
> for pse_diagnostics::DiagnosticRule {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticRule,
        AdapterError,
    > {
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticRule {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticRule,
> for pse_diagnostics::DiagnosticRule {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticRule,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticSeverity,
> for pse_model::generated::enums::DiagnosticSeverity {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticSeverity,
        AdapterError,
    > {
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticSeverity {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticSeverity,
> for pse_model::generated::enums::DiagnosticSeverity {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticSeverity,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticStage,
> for pse_diagnostics::DiagnosticStage {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticStage,
        AdapterError,
    > {
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticStage {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticStage,
> for pse_diagnostics::DiagnosticStage {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticStage,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::EntityKindId,
> for pse_quantity::ids::EntityKindId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::EntityKindId,
        AdapterError,
    > {
        layout(
            <pse_quantity::ids::EntityKindId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::EntityKindId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::EntityKindId {
            id: Some(
                pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                    field_0,
                ),
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::EntityKindId,
> for pse_quantity::ids::EntityKindId {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::EntityKindId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::EntityKindId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::EntityKindId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                required(wire.id, &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::FileObservation,
> for pse_buildinfo::FileObservation {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::FileObservation,
        AdapterError,
    > {
        layout(
            <pse_buildinfo::FileObservation as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_buildinfo::FileObservation as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "path"), (2, "sha256"), (3, "canonical"), (4, "mode")],
            path,
        )?;
        let (field_0, field_1, field_2, field_3) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::FileObservation {
            path: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::NativePathBuf,
                >(field_0, &child_path(path, "path"))?,
            ),
            sha256: native_into::<_, String>(field_1, &child_path(path, "sha256"))?,
            canonical: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::NativePathBuf,
                >(field_2, &child_path(path, "canonical"))?,
            ),
            mode: native_into::<_, u32>(field_3, &child_path(path, "mode"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::FileObservation,
> for pse_buildinfo::FileObservation {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::FileObservation,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_buildinfo::FileObservation as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_buildinfo::FileObservation as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "path"), (2, "sha256"), (3, "canonical"), (4, "mode")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::NativePathBuf,
            >(
                required(wire.path.into_option(), &child_path(path, "path"))?,
                &child_path(path, "path"),
            )?,
            native_from::<_, String>(wire.sha256, &child_path(path, "sha256"))?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::NativePathBuf,
            >(
                required(wire.canonical.into_option(), &child_path(path, "canonical"))?,
                &child_path(path, "canonical"),
            )?,
            native_from::<_, u32>(wire.mode, &child_path(path, "mode"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::ImplicitOptions,
> for pse_math::implicit::Options {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::ImplicitOptions,
        AdapterError,
    > {
        layout(
            <pse_math::implicit::Options as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::implicit::Options as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "start"),
                (2, "variable_nominals"),
                (3, "variable_tolerance"),
                (4, "residual_tolerance"),
                (5, "iterations"),
                (6, "time_limit"),
                (7, "derivative_tolerance"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5, field_6) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::ImplicitOptions {
            start: {
                let current_path = &child_path(path, "start");
                items(field_0)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                u64,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            variable_nominals: {
                let current_path = &child_path(path, "variable_nominals");
                items(field_1)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                u64,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            variable_tolerance: {
                let current_path = &child_path(path, "variable_tolerance");
                items(field_2)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                u64,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            residual_tolerance: {
                let current_path = &child_path(path, "residual_tolerance");
                items(field_3)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                u64,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            iterations: native_into::<_, u32>(field_4, &child_path(path, "iterations"))?,
            time_limit: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::NonnegativeDuration,
                >(field_5, &child_path(path, "time_limit"))?,
            ),
            derivative_tolerance: native_into::<
                _,
                u64,
            >(field_6, &child_path(path, "derivative_tolerance"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::ImplicitOptions,
> for pse_math::implicit::Options {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::ImplicitOptions,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_math::implicit::Options as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::implicit::Options as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "start"),
                (2, "variable_nominals"),
                (3, "variable_tolerance"),
                (4, "residual_tolerance"),
                (5, "iterations"),
                (6, "time_limit"),
                (7, "derivative_tolerance"),
            ],
            path,
        )?;
        let fields = (
            {
                let current_path = &child_path(path, "start");
                let values = wire
                    .start
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                u64,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "variable_nominals");
                let values = wire
                    .variable_nominals
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                u64,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "variable_tolerance");
                let values = wire
                    .variable_tolerance
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                u64,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "residual_tolerance");
                let values = wire
                    .residual_tolerance
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                u64,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<_, u32>(wire.iterations, &child_path(path, "iterations"))?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::NonnegativeDuration,
            >(
                required(
                    wire.time_limit.into_option(),
                    &child_path(path, "time_limit"),
                )?,
                &child_path(path, "time_limit"),
            )?,
            native_from::<
                _,
                u64,
            >(wire.derivative_tolerance, &child_path(path, "derivative_tolerance"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::ImplicitUnknown,
> for pse_math::implicit::Unknown {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::ImplicitUnknown,
        AdapterError,
    > {
        layout(
            <pse_math::implicit::Unknown as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::implicit::Unknown as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "lower"), (3, "upper")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::ImplicitUnknown {
            id: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_0,
            ),
            lower: native_into::<_, u64>(field_1, &child_path(path, "lower"))?,
            upper: native_into::<_, u64>(field_2, &child_path(path, "upper"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::ImplicitUnknown,
> for pse_math::implicit::Unknown {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::ImplicitUnknown,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_math::implicit::Unknown as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::implicit::Unknown as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "lower"), (3, "upper")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.id,
                &child_path(path, "id"),
            )?,
            native_from::<_, u64>(wire.lower, &child_path(path, "lower"))?,
            native_from::<_, u64>(wire.upper, &child_path(path, "upper"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::InvariantId,
> for pse_quantity::ids::InvariantId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::InvariantId,
        AdapterError,
    > {
        layout(
            <pse_quantity::ids::InvariantId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::InvariantId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::InvariantId {
            id: Some(
                pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                    field_0,
                ),
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::InvariantId,
> for pse_quantity::ids::InvariantId {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::InvariantId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::InvariantId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::InvariantId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                required(wire.id, &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::KindDefinition,
> for pse_quantity::kind::KindDefinition {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::KindDefinition,
        AdapterError,
    > {
        layout(
            <pse_quantity::kind::KindDefinition as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::kind::KindDefinition as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "monomial"),
                (2, "canonical_unit"),
                (3, "basis"),
                (4, "reference_state"),
                (5, "scale_kind"),
                (6, "subject_kind"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::KindDefinition {
            monomial: {
                let current_path = &child_path(path, "monomial");
                items(field_0)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::KindFactor,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            canonical_unit: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::UnitId,
                >(field_1, &child_path(path, "canonical_unit"))?,
            ),
            basis: field_2
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::BasisId,
                        >(value, &child_path(path, "basis"))?,
                    )
                })
                .transpose()?
                .into(),
            reference_state: field_3
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceStateId,
                        >(value, &child_path(path, "reference_state"))?,
                    )
                })
                .transpose()?
                .into(),
            scale_kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::ScaleKind,
                >(field_4, &child_path(path, "scale_kind"))?,
            ),
            subject_kind: field_5
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::EntityKindId,
                        >(value, &child_path(path, "subject_kind"))?,
                    )
                })
                .transpose()?
                .into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::KindDefinition,
> for pse_quantity::kind::KindDefinition {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::KindDefinition,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::kind::KindDefinition as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::kind::KindDefinition as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "monomial"),
                (2, "canonical_unit"),
                (3, "basis"),
                (4, "reference_state"),
                (5, "scale_kind"),
                (6, "subject_kind"),
            ],
            path,
        )?;
        let fields = (
            {
                let current_path = &child_path(path, "monomial");
                let values = wire
                    .monomial
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::KindFactor,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::UnitId,
            >(
                required(
                    wire.canonical_unit.into_option(),
                    &child_path(path, "canonical_unit"),
                )?,
                &child_path(path, "canonical_unit"),
            )?,
            wire
                .basis
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::BasisId,
                        >(value, &child_path(path, "basis"))?,
                    )
                })
                .transpose()?,
            wire
                .reference_state
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceStateId,
                        >(value, &child_path(path, "reference_state"))?,
                    )
                })
                .transpose()?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::ScaleKind,
            >(
                required(
                    wire.scale_kind.into_option(),
                    &child_path(path, "scale_kind"),
                )?,
                &child_path(path, "scale_kind"),
            )?,
            wire
                .subject_kind
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::EntityKindId,
                        >(value, &child_path(path, "subject_kind"))?,
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
impl NativeInto<crate::math::native_root_generated::pb::pse::native_root::v2::KindFactor>
for pse_quantity::kind::KindFactor {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::KindFactor,
        AdapterError,
    > {
        layout(
            <pse_quantity::kind::KindFactor as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::kind::KindFactor as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "kind"), (2, "exponent")],
            path,
        )?;
        let (field_0, field_1) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::KindFactor {
            kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityKindId,
                >(field_0, &child_path(path, "kind"))?,
            ),
            exponent: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalRatio,
                >(field_1, &child_path(path, "exponent"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<crate::math::native_root_generated::pb::pse::native_root::v2::KindFactor>
for pse_quantity::kind::KindFactor {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::KindFactor,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::kind::KindFactor as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::kind::KindFactor as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "kind"), (2, "exponent")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::QuantityKindId,
            >(
                required(wire.kind.into_option(), &child_path(path, "kind"))?,
                &child_path(path, "kind"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalRatio,
            >(
                required(wire.exponent.into_option(), &child_path(path, "exponent"))?,
                &child_path(path, "exponent"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::MathBodySpec,
> for pse_math::binding::BodySpec {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::MathBodySpec,
        AdapterError,
    > {
        layout(
            <pse_math::binding::BodySpec as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::binding::BodySpec as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "definition"),
                (2, "structure"),
                (3, "physical"),
                (4, "admissions"),
                (5, "providers"),
                (6, "policy"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::MathBodySpec {
            definition: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_0,
            ),
            structure: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_1,
            ),
            physical: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_2,
            ),
            admissions: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_3,
            ),
            providers: items(field_4)
                .map(
                    pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes,
                )
                .collect::<Vec<_>>(),
            policy: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_5,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::MathBodySpec,
> for pse_math::binding::BodySpec {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::MathBodySpec,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_math::binding::BodySpec as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::binding::BodySpec as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "definition"),
                (2, "structure"),
                (3, "physical"),
                (4, "admissions"),
                (5, "providers"),
                (6, "policy"),
            ],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.definition,
                &child_path(path, "definition"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.structure,
                &child_path(path, "structure"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.physical,
                &child_path(path, "physical"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.admissions,
                &child_path(path, "admissions"),
            )?,
            {
                let current_path = &child_path(path, "providers");
                let values = wire
                    .providers
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                                value,
                                &format!("{path}[{index}]", path = current_path),
                            )?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.policy,
                &child_path(path, "policy"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::MathEvaluationLimits,
> for pse_math::jets::EvaluationLimits {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::MathEvaluationLimits,
        AdapterError,
    > {
        layout(
            <pse_math::jets::EvaluationLimits as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::jets::EvaluationLimits as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "derivative_components"),
                (2, "operations"),
                (3, "scratch_bytes"),
                (4, "provider_calls"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::MathEvaluationLimits {
            derivative_components: native_into::<
                _,
                u64,
            >(field_0, &child_path(path, "derivative_components"))?,
            operations: native_into::<_, u64>(field_1, &child_path(path, "operations"))?,
            scratch_bytes: native_into::<
                _,
                u64,
            >(field_2, &child_path(path, "scratch_bytes"))?,
            provider_calls: native_into::<
                _,
                u64,
            >(field_3, &child_path(path, "provider_calls"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::MathEvaluationLimits,
> for pse_math::jets::EvaluationLimits {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::MathEvaluationLimits,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_math::jets::EvaluationLimits as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::jets::EvaluationLimits as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "derivative_components"),
                (2, "operations"),
                (3, "scratch_bytes"),
                (4, "provider_calls"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                u64,
            >(wire.derivative_components, &child_path(path, "derivative_components"))?,
            native_from::<_, u64>(wire.operations, &child_path(path, "operations"))?,
            native_from::<
                _,
                u64,
            >(wire.scratch_bytes, &child_path(path, "scratch_bytes"))?,
            native_from::<
                _,
                u64,
            >(wire.provider_calls, &child_path(path, "provider_calls"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::ModelBoundaryDiagnostic,
> for pse_model::diagnostic::BoundaryDiagnostic {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::ModelBoundaryDiagnostic,
        AdapterError,
    > {
        layout(
            <pse_model::diagnostic::BoundaryDiagnostic as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::diagnostic::BoundaryDiagnostic as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "code"),
                (2, "causes"),
                (3, "class"),
                (4, "severity"),
                (5, "stage"),
                (6, "sources"),
                (7, "rule"),
                (8, "observations"),
                (9, "locations"),
                (10, "validity"),
                (11, "applicability"),
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
        ) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::ModelBoundaryDiagnostic {
            code: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticCode,
                >(field_0, &child_path(path, "code"))?,
            ),
            causes: {
                let current_path = &child_path(path, "causes");
                items(field_1)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::ModelBoundaryDiagnostic,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            class: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::NativeBoundaryClass,
                >(field_2, &child_path(path, "class"))?,
            ),
            severity: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticSeverity,
                >(field_3, &child_path(path, "severity"))?,
            ),
            stage: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticStage,
                >(field_4, &child_path(path, "stage"))?,
            ),
            sources: items(field_5)
                .map(
                    pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes,
                )
                .collect::<Vec<_>>(),
            rule: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticRule,
                >(field_6, &child_path(path, "rule"))?,
            ),
            observations: wire_map(
                field_7
                    .into_iter()
                    .map(|(key, value)| -> Result<_, AdapterError> {
                        Ok((
                            native_into::<
                                _,
                                String,
                            >(key, &child_path(path, "observations"))?,
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticObservation,
                            >(value, &child_path(path, "observations"))?,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter(),
            ),
            locations: {
                let current_path = &child_path(path, "locations");
                items(field_8)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::ModelSourceLocation,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            validity: field_9
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ModelValidityLineage,
                        >(value, &child_path(path, "validity"))?,
                    )
                })
                .transpose()?
                .into(),
            applicability: {
                let current_path = &child_path(path, "applicability");
                items(field_10)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityObservation,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::ModelBoundaryDiagnostic,
> for pse_model::diagnostic::BoundaryDiagnostic {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::ModelBoundaryDiagnostic,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_model::diagnostic::BoundaryDiagnostic as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::diagnostic::BoundaryDiagnostic as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "code"),
                (2, "causes"),
                (3, "class"),
                (4, "severity"),
                (5, "stage"),
                (6, "sources"),
                (7, "rule"),
                (8, "observations"),
                (9, "locations"),
                (10, "validity"),
                (11, "applicability"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticCode,
            >(
                required(wire.code.into_option(), &child_path(path, "code"))?,
                &child_path(path, "code"),
            )?,
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
                                crate::math::native_root_generated::pb::pse::native_root::v2::ModelBoundaryDiagnostic,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::NativeBoundaryClass,
            >(
                required(wire.class.into_option(), &child_path(path, "class"))?,
                &child_path(path, "class"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticSeverity,
            >(
                required(wire.severity.into_option(), &child_path(path, "severity"))?,
                &child_path(path, "severity"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticStage,
            >(
                required(wire.stage.into_option(), &child_path(path, "stage"))?,
                &child_path(path, "stage"),
            )?,
            {
                let current_path = &child_path(path, "sources");
                let values = wire
                    .sources
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                                value,
                                &format!("{path}[{index}]", path = current_path),
                            )?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticRule,
            >(
                required(wire.rule.into_option(), &child_path(path, "rule"))?,
                &child_path(path, "rule"),
            )?,
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
                                crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticObservation,
                            >(value, &child_path(path, "observations"))?,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                &child_path(path, "observations"),
            )?,
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
                                crate::math::native_root_generated::pb::pse::native_root::v2::ModelSourceLocation,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            wire
                .validity
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ModelValidityLineage,
                        >(value, &child_path(path, "validity"))?,
                    )
                })
                .transpose()?,
            {
                let current_path = &child_path(path, "applicability");
                let values = wire
                    .applicability
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityObservation,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::ModelSourceLocation,
> for pse_model::diagnostic::SourceLocation {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::ModelSourceLocation,
        AdapterError,
    > {
        layout(
            <pse_model::diagnostic::SourceLocation as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::diagnostic::SourceLocation as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::ModelSourceLocation {
            source: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_0,
            ),
            revision: field_1
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                            value,
                        ),
                    )
                })
                .transpose()?,
            document: field_2
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                            value,
                        ),
                    )
                })
                .transpose()?,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::ModelSourceLocation,
> for pse_model::diagnostic::SourceLocation {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::ModelSourceLocation,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_model::diagnostic::SourceLocation as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::diagnostic::SourceLocation as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.source,
                &child_path(path, "source"),
            )?,
            wire
                .revision
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                            value,
                            &child_path(path, "revision"),
                        )?,
                    )
                })
                .transpose()?,
            wire
                .document
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                            value,
                            &child_path(path, "document"),
                        )?,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::ModelValidityLineage,
> for pse_model::diagnostic::ValidityLineage {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::ModelValidityLineage,
        AdapterError,
    > {
        layout(
            <pse_model::diagnostic::ValidityLineage as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::diagnostic::ValidityLineage as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "layer"),
                (2, "source"),
                (3, "form"),
                (4, "sets"),
                (5, "variables"),
                (6, "members"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::ModelValidityLineage {
            layer: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::ModelingValidityLayer,
                >(field_0, &child_path(path, "layer"))?,
            ),
            source: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_1,
            ),
            form: field_2
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                            value,
                        ),
                    )
                })
                .transpose()?,
            sets: items(field_3)
                .map(
                    pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes,
                )
                .collect::<Vec<_>>(),
            variables: {
                let current_path = &child_path(path, "variables");
                items(field_4)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                u32,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            members: items(field_5)
                .map(
                    pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes,
                )
                .collect::<Vec<_>>(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::ModelValidityLineage,
> for pse_model::diagnostic::ValidityLineage {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::ModelValidityLineage,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_model::diagnostic::ValidityLineage as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::diagnostic::ValidityLineage as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "layer"),
                (2, "source"),
                (3, "form"),
                (4, "sets"),
                (5, "variables"),
                (6, "members"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::ModelingValidityLayer,
            >(
                required(wire.layer.into_option(), &child_path(path, "layer"))?,
                &child_path(path, "layer"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.source,
                &child_path(path, "source"),
            )?,
            wire
                .form
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                            value,
                            &child_path(path, "form"),
                        )?,
                    )
                })
                .transpose()?,
            {
                let current_path = &child_path(path, "sets");
                let values = wire
                    .sets
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                                value,
                                &format!("{path}[{index}]", path = current_path),
                            )?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "variables");
                let values = wire
                    .variables
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                u32,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "members");
                let values = wire
                    .members
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                                value,
                                &format!("{path}[{index}]", path = current_path),
                            )?,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::ModelingApplicabilityBasis,
> for pse_model::generated::enums::ModelingApplicabilityBasis {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::ModelingApplicabilityBasis,
        AdapterError,
    > {
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::ModelingApplicabilityBasis {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::ModelingApplicabilityBasis,
> for pse_model::generated::enums::ModelingApplicabilityBasis {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::ModelingApplicabilityBasis,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::ModelingApplicabilityOutcome,
> for pse_model::generated::enums::ModelingApplicabilityOutcome {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::ModelingApplicabilityOutcome,
        AdapterError,
    > {
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::ModelingApplicabilityOutcome {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::ModelingApplicabilityOutcome,
> for pse_model::generated::enums::ModelingApplicabilityOutcome {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::ModelingApplicabilityOutcome,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::ModelingPermissionTarget,
> for pse_model::generated::enums::ModelingPermissionTarget {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::ModelingPermissionTarget,
        AdapterError,
    > {
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::ModelingPermissionTarget {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::ModelingPermissionTarget,
> for pse_model::generated::enums::ModelingPermissionTarget {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::ModelingPermissionTarget,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::ModelingValidityLayer,
> for pse_model::generated::enums::ModelingValidityLayer {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::ModelingValidityLayer,
        AdapterError,
    > {
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::ModelingValidityLayer {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::ModelingValidityLayer,
> for pse_model::generated::enums::ModelingValidityLayer {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::ModelingValidityLayer,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::NativeBoundaryClass,
> for pse_model::generated::enums::NativeBoundaryClass {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::NativeBoundaryClass,
        AdapterError,
    > {
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::NativeBoundaryClass {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::NativeBoundaryClass,
> for pse_model::generated::enums::NativeBoundaryClass {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::NativeBoundaryClass,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::NativePathBuf,
> for std::path::PathBuf {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::NativePathBuf,
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
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::NativePathBuf {
            platform: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::PlatformPathBytes,
                >(field_0, &child_path(path, "platform"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::NativePathBuf,
> for std::path::PathBuf {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::NativePathBuf,
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
                crate::math::native_root_generated::pb::pse::native_root::v2::PlatformPathBytes,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootBindings,
> for crate::math::native_root_protocol::RootBindings {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootBindings,
        AdapterError,
    > {
        layout(
            <crate::math::native_root_protocol::RootBindings as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::math::native_root_protocol::RootBindings as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            2,
            &[
                (1, "instance"),
                (2, "checked_members"),
                (3, "checked_sources"),
                (4, "occurrences"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootBindings {
            instance: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_0,
            ),
            checked_members: field_1
                .into_iter()
                .map(|(key, value)| -> Result<_, AdapterError> {
                    Ok(crate::math::native_root_generated::pb::pse::native_root::v2::native_root_bindings::Field2Entry {
                        key: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                            key,
                        ),
                        value: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                            value,
                        ),
                        ..Default::default()
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
            checked_sources: field_2
                .into_iter()
                .map(|(key, value)| -> Result<_, AdapterError> {
                    Ok(crate::math::native_root_generated::pb::pse::native_root::v2::native_root_bindings::Field3Entry {
                        key: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                            key,
                        ),
                        value: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                            value,
                        ),
                        ..Default::default()
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
            occurrences: field_3
                .into_iter()
                .map(|(key, value)| -> Result<_, AdapterError> {
                    Ok(crate::math::native_root_generated::pb::pse::native_root::v2::native_root_bindings::Field4Entry {
                        key: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                            key,
                        ),
                        value: buffa::MessageField::some(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::ModelSourceLocation,
                            >(value, &child_path(path, "occurrences"))?,
                        ),
                        ..Default::default()
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootBindings,
> for crate::math::native_root_protocol::RootBindings {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootBindings,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::math::native_root_protocol::RootBindings as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::math::native_root_protocol::RootBindings as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            2,
            &[
                (1, "instance"),
                (2, "checked_members"),
                (3, "checked_sources"),
                (4, "occurrences"),
            ],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.instance,
                &child_path(path, "instance"),
            )?,
            unique_pairs(
                wire
                    .checked_members
                    .into_iter()
                    .map(|entry| -> Result<_, AdapterError> {
                        unknown_fields(
                            &entry.__buffa_unknown_fields,
                            &child_path(path, "checked_members"),
                        )?;
                        Ok((
                            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                                entry.key,
                                &child_path(path, "checked_members"),
                            )?,
                            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                                entry.value,
                                &child_path(path, "checked_members"),
                            )?,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                &child_path(path, "checked_members"),
            )?,
            unique_pairs(
                wire
                    .checked_sources
                    .into_iter()
                    .map(|entry| -> Result<_, AdapterError> {
                        unknown_fields(
                            &entry.__buffa_unknown_fields,
                            &child_path(path, "checked_sources"),
                        )?;
                        Ok((
                            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                                entry.key,
                                &child_path(path, "checked_sources"),
                            )?,
                            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                                entry.value,
                                &child_path(path, "checked_sources"),
                            )?,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                &child_path(path, "checked_sources"),
            )?,
            unique_pairs(
                wire
                    .occurrences
                    .into_iter()
                    .map(|entry| -> Result<_, AdapterError> {
                        unknown_fields(
                            &entry.__buffa_unknown_fields,
                            &child_path(path, "occurrences"),
                        )?;
                        Ok((
                            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                                entry.key,
                                &child_path(path, "occurrences"),
                            )?,
                            native_from::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::ModelSourceLocation,
                            >(
                                required(
                                    entry.value.into_option(),
                                    &child_path(path, "occurrences"),
                                )?,
                                &child_path(path, "occurrences"),
                            )?,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                &child_path(path, "occurrences"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootHello,
> for crate::math::native_root_protocol::Hello {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootHello,
        AdapterError,
    > {
        layout(
            <crate::math::native_root_protocol::Hello as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::math::native_root_protocol::Hello as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            2,
            &[
                (1, "diagnostic_operation"),
                (2, "protocol"),
                (3, "attempt"),
                (4, "pid"),
                (5, "parent"),
                (6, "executable"),
                (7, "build"),
                (8, "native_build"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5, field_6, field_7) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootHello {
            diagnostic_operation: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticOperationId,
                >(field_0, &child_path(path, "diagnostic_operation"))?,
            ),
            protocol: native_into::<_, String>(field_1, &child_path(path, "protocol"))?,
            attempt: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_2,
            ),
            pid: native_into::<_, u32>(field_3, &child_path(path, "pid"))?,
            parent: native_into::<_, u32>(field_4, &child_path(path, "parent"))?,
            executable: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::FileObservation,
                >(field_5, &child_path(path, "executable"))?,
            ),
            build: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::CapturedBuildInfo,
                >(field_6, &child_path(path, "build"))?,
            ),
            native_build: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_7,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootHello,
> for crate::math::native_root_protocol::Hello {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootHello,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::math::native_root_protocol::Hello as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::math::native_root_protocol::Hello as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            2,
            &[
                (1, "diagnostic_operation"),
                (2, "protocol"),
                (3, "attempt"),
                (4, "pid"),
                (5, "parent"),
                (6, "executable"),
                (7, "build"),
                (8, "native_build"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticOperationId,
            >(
                required(
                    wire.diagnostic_operation.into_option(),
                    &child_path(path, "diagnostic_operation"),
                )?,
                &child_path(path, "diagnostic_operation"),
            )?,
            native_from::<_, String>(wire.protocol, &child_path(path, "protocol"))?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.attempt,
                &child_path(path, "attempt"),
            )?,
            native_from::<_, u32>(wire.pid, &child_path(path, "pid"))?,
            native_from::<_, u32>(wire.parent, &child_path(path, "parent"))?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::FileObservation,
            >(
                required(
                    wire.executable.into_option(),
                    &child_path(path, "executable"),
                )?,
                &child_path(path, "executable"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::CapturedBuildInfo,
            >(
                required(wire.build.into_option(), &child_path(path, "build"))?,
                &child_path(path, "build"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.native_build,
                &child_path(path, "native_build"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootInput,
> for crate::math::native_root_protocol::RootInput {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootInput,
        AdapterError,
    > {
        layout(
            <crate::math::native_root_protocol::RootInput as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::math::native_root_protocol::RootInput as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            3,
            &[
                (1, "body"),
                (2, "spec"),
                (3, "source"),
                (4, "registry"),
                (6, "unknowns"),
                (7, "rows"),
                (8, "parameters"),
                (9, "options"),
                (10, "evaluation"),
                (11, "max_entries"),
                (12, "preconditions"),
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
        ) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootInput {
            body: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_0,
            ),
            spec: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::MathBodySpec,
                >(field_1, &child_path(path, "spec"))?,
            ),
            source: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootBindings,
                >(field_2, &child_path(path, "source"))?,
            ),
            registry: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalRegistryDeclarations,
                >(field_3, &child_path(path, "registry"))?,
            ),
            unknowns: {
                let current_path = &child_path(path, "unknowns");
                items(field_4)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::ImplicitUnknown,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            rows: items(field_5)
                .map(
                    pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes,
                )
                .collect::<Vec<_>>(),
            parameters: {
                let current_path = &child_path(path, "parameters");
                items(field_6)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                u64,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            options: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::ImplicitOptions,
                >(field_7, &child_path(path, "options"))?,
            ),
            evaluation: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::MathEvaluationLimits,
                >(field_8, &child_path(path, "evaluation"))?,
            ),
            max_entries: native_into::<
                _,
                u64,
            >(field_9, &child_path(path, "max_entries"))?,
            preconditions: {
                let current_path = &child_path(path, "preconditions");
                items(field_10)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalPrerequisiteDeclaration,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootInput,
> for crate::math::native_root_protocol::RootInput {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootInput,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::math::native_root_protocol::RootInput as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::math::native_root_protocol::RootInput as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            3,
            &[
                (1, "body"),
                (2, "spec"),
                (3, "source"),
                (4, "registry"),
                (6, "unknowns"),
                (7, "rows"),
                (8, "parameters"),
                (9, "options"),
                (10, "evaluation"),
                (11, "max_entries"),
                (12, "preconditions"),
            ],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.body,
                &child_path(path, "body"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::MathBodySpec,
            >(
                required(wire.spec.into_option(), &child_path(path, "spec"))?,
                &child_path(path, "spec"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootBindings,
            >(
                required(wire.source.into_option(), &child_path(path, "source"))?,
                &child_path(path, "source"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalRegistryDeclarations,
            >(
                required(wire.registry.into_option(), &child_path(path, "registry"))?,
                &child_path(path, "registry"),
            )?,
            {
                let current_path = &child_path(path, "unknowns");
                let values = wire
                    .unknowns
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::ImplicitUnknown,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "rows");
                let values = wire
                    .rows
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                                value,
                                &format!("{path}[{index}]", path = current_path),
                            )?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "parameters");
                let values = wire
                    .parameters
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                u64,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::ImplicitOptions,
            >(
                required(wire.options.into_option(), &child_path(path, "options"))?,
                &child_path(path, "options"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::MathEvaluationLimits,
            >(
                required(
                    wire.evaluation.into_option(),
                    &child_path(path, "evaluation"),
                )?,
                &child_path(path, "evaluation"),
            )?,
            native_from::<_, u64>(wire.max_entries, &child_path(path, "max_entries"))?,
            {
                let current_path = &child_path(path, "preconditions");
                let values = wire
                    .preconditions
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalPrerequisiteDeclaration,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootOutcome,
> for crate::math::native_root_protocol::RootOutcome {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootOutcome,
        AdapterError,
    > {
        let value = self;
        layout(
            <crate::math::native_root_protocol::RootOutcome as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <crate::math::native_root_protocol::RootOutcome as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            2,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::math::native_root_protocol::RootOutcome as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Proposal");
                let path = alternative_path.as_str();
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootOutcome {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::native_root_outcome::Kind::Proposal(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::native_root_outcome::Alternative1 {
                                value: {
                                    let current_path = &child_path(path, "value");
                                    items(field_0)
                                        .enumerate()
                                        .map(|(index, value)| -> Result<_, AdapterError> {
                                            Ok(
                                                native_into::<
                                                    _,
                                                    u64,
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
        layout(
            <crate::math::native_root_protocol::RootOutcome as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <crate::math::native_root_protocol::RootOutcome as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            2,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::math::native_root_protocol::RootOutcome as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Refused");
                let path = alternative_path.as_str();
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootOutcome {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::native_root_outcome::Kind::Refused(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::native_root_outcome::Alternative2 {
                                value: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::math::native_root_generated::pb::pse::native_root::v2::ModelBoundaryDiagnostic,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootOutcome,
> for crate::math::native_root_protocol::RootOutcome {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootOutcome,
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
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::native_root_outcome::Kind::Proposal(
                wire,
            ) => {
                let alternative_path = child_path(path, "Proposal");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::math::native_root_protocol::RootOutcome as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <crate::math::native_root_protocol::RootOutcome as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    2,
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
                                        u64,
                                    >(value, &format!("{path}[{index}]", path = current_path))?,
                                )
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <crate::math::native_root_protocol::RootOutcome as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::native_root_outcome::Kind::Refused(
                wire,
            ) => {
                let alternative_path = child_path(path, "Refused");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::math::native_root_protocol::RootOutcome as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <crate::math::native_root_protocol::RootOutcome as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    2,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::math::native_root_generated::pb::pse::native_root::v2::ModelBoundaryDiagnostic,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <crate::math::native_root_protocol::RootOutcome as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRequest,
> for crate::math::native_root_protocol::Request {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRequest,
        AdapterError,
    > {
        layout(
            <crate::math::native_root_protocol::Request as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::math::native_root_protocol::Request as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            2,
            &[
                (1, "diagnostic_parent"),
                (2, "attempt"),
                (3, "execution"),
                (4, "request"),
                (5, "remaining"),
                (6, "source"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRequest {
            diagnostic_parent: field_0
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticOperationId,
                        >(value, &child_path(path, "diagnostic_parent"))?,
                    )
                })
                .transpose()?
                .into(),
            attempt: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_1,
            ),
            execution: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_2,
            ),
            request: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_3,
            ),
            remaining: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::NonnegativeDuration,
                >(field_4, &child_path(path, "remaining"))?,
            ),
            source: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_5,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRequest,
> for crate::math::native_root_protocol::Request {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRequest,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::math::native_root_protocol::Request as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::math::native_root_protocol::Request as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            2,
            &[
                (1, "diagnostic_parent"),
                (2, "attempt"),
                (3, "execution"),
                (4, "request"),
                (5, "remaining"),
                (6, "source"),
            ],
            path,
        )?;
        let fields = (
            wire
                .diagnostic_parent
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticOperationId,
                        >(value, &child_path(path, "diagnostic_parent"))?,
                    )
                })
                .transpose()?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.attempt,
                &child_path(path, "attempt"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.execution,
                &child_path(path, "execution"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.request,
                &child_path(path, "request"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::NonnegativeDuration,
            >(
                required(wire.remaining.into_option(), &child_path(path, "remaining"))?,
                &child_path(path, "remaining"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.source,
                &child_path(path, "source"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootResponse,
> for crate::math::native_root_protocol::Response {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootResponse,
        AdapterError,
    > {
        layout(
            <crate::math::native_root_protocol::Response as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::math::native_root_protocol::Response as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            2,
            &[(1, "attempt"), (2, "execution"), (3, "request"), (4, "result")],
            path,
        )?;
        let (field_0, field_1, field_2, field_3) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootResponse {
            attempt: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_0,
            ),
            execution: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_1,
            ),
            request: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_2,
            ),
            result: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootOutcome,
                >(field_3, &child_path(path, "result"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootResponse,
> for crate::math::native_root_protocol::Response {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootResponse,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::math::native_root_protocol::Response as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::math::native_root_protocol::Response as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            2,
            &[(1, "attempt"), (2, "execution"), (3, "request"), (4, "result")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.attempt,
                &child_path(path, "attempt"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.execution,
                &child_path(path, "execution"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.request,
                &child_path(path, "request"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootOutcome,
            >(
                required(wire.result.into_option(), &child_path(path, "result"))?,
                &child_path(path, "result"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::NonnegativeDuration,
> for core::time::Duration {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::NonnegativeDuration,
        AdapterError,
    > {
        layout(
            <core::time::Duration as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <core::time::Duration as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "secs"), (2, "nanos")],
            path,
        )?;
        let (field_0, field_1) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::NonnegativeDuration {
            secs: native_into::<_, u64>(field_0, &child_path(path, "secs"))?,
            nanos: native_into::<_, u32>(field_1, &child_path(path, "nanos"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::NonnegativeDuration,
> for core::time::Duration {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::NonnegativeDuration,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <core::time::Duration as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <core::time::Duration as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "secs"), (2, "nanos")],
            path,
        )?;
        let fields = (
            native_from::<_, u64>(wire.secs, &child_path(path, "secs"))?,
            native_from::<_, u32>(wire.nanos, &child_path(path, "nanos"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<crate::math::native_root_generated::pb::pse::native_root::v2::Opcode>
for pse_quantity::enums::Opcode {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::Opcode,
        AdapterError,
    > {
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::Opcode {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<crate::math::native_root_generated::pb::pse::native_root::v2::Opcode>
for pse_quantity::enums::Opcode {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::Opcode,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::OperationId,
> for pse_quantity::ids::OperationId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::OperationId,
        AdapterError,
    > {
        layout(
            <pse_quantity::ids::OperationId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::OperationId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::OperationId {
            id: Some(
                pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                    field_0,
                ),
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::OperationId,
> for pse_quantity::ids::OperationId {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::OperationId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::OperationId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::OperationId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                required(wire.id, &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalBasis,
> for pse_quantity::basis::Basis {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalBasis,
        AdapterError,
    > {
        layout(
            <pse_quantity::basis::Basis as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::basis::Basis as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "id"),
                (2, "kind"),
                (3, "composition_basis"),
                (4, "rate_basis"),
                (5, "reference_conditions"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalBasis {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::BasisId,
                >(field_0, &child_path(path, "id"))?,
            ),
            kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::BasisKind,
                >(field_1, &child_path(path, "kind"))?,
            ),
            composition_basis: field_2
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::CompositionBasis,
                        >(value, &child_path(path, "composition_basis"))?,
                    )
                })
                .transpose()?
                .into(),
            rate_basis: field_3
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::RateBasis,
                        >(value, &child_path(path, "rate_basis"))?,
                    )
                })
                .transpose()?
                .into(),
            reference_conditions: field_4
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceStateId,
                        >(value, &child_path(path, "reference_conditions"))?,
                    )
                })
                .transpose()?
                .into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalBasis,
> for pse_quantity::basis::Basis {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalBasis,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::basis::Basis as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::basis::Basis as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "id"),
                (2, "kind"),
                (3, "composition_basis"),
                (4, "rate_basis"),
                (5, "reference_conditions"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::BasisId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::BasisKind,
            >(
                required(wire.kind.into_option(), &child_path(path, "kind"))?,
                &child_path(path, "kind"),
            )?,
            wire
                .composition_basis
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::CompositionBasis,
                        >(value, &child_path(path, "composition_basis"))?,
                    )
                })
                .transpose()?,
            wire
                .rate_basis
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::RateBasis,
                        >(value, &child_path(path, "rate_basis"))?,
                    )
                })
                .transpose()?,
            wire
                .reference_conditions
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceStateId,
                        >(value, &child_path(path, "reference_conditions"))?,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalConversionDeclaration,
> for pse_quantity::ConversionRule {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalConversionDeclaration,
        AdapterError,
    > {
        layout(
            <pse_quantity::ConversionRule as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ConversionRule as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "id"),
                (2, "from"),
                (3, "to"),
                (4, "kind"),
                (5, "kernel"),
                (6, "required_parameters"),
                (7, "scale"),
                (8, "offset"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5, field_6, field_7) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalConversionDeclaration {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::ConversionId,
                >(field_0, &child_path(path, "id"))?,
            ),
            from: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityTypeId,
                >(field_1, &child_path(path, "from"))?,
            ),
            to: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityTypeId,
                >(field_2, &child_path(path, "to"))?,
            ),
            kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::ConversionKind,
                >(field_3, &child_path(path, "kind"))?,
            ),
            kernel: field_4
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                            value,
                        ),
                    )
                })
                .transpose()?,
            required_parameters: {
                let current_path = &child_path(path, "required_parameters");
                items(field_5)
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
            scale: field_6
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_into::<_, u64>(value, &child_path(path, "scale"))?)
                })
                .transpose()?,
            offset: field_7
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_into::<_, u64>(value, &child_path(path, "offset"))?)
                })
                .transpose()?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalConversionDeclaration,
> for pse_quantity::ConversionRule {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalConversionDeclaration,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ConversionRule as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ConversionRule as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "id"),
                (2, "from"),
                (3, "to"),
                (4, "kind"),
                (5, "kernel"),
                (6, "required_parameters"),
                (7, "scale"),
                (8, "offset"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::ConversionId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::QuantityTypeId,
            >(
                required(wire.from.into_option(), &child_path(path, "from"))?,
                &child_path(path, "from"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::QuantityTypeId,
            >(
                required(wire.to.into_option(), &child_path(path, "to"))?,
                &child_path(path, "to"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::ConversionKind,
            >(
                required(wire.kind.into_option(), &child_path(path, "kind"))?,
                &child_path(path, "kind"),
            )?,
            wire
                .kernel
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                            value,
                            &child_path(path, "kernel"),
                        )?,
                    )
                })
                .transpose()?,
            {
                let current_path = &child_path(path, "required_parameters");
                let values = wire
                    .required_parameters
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
                .scale
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_from::<_, u64>(value, &child_path(path, "scale"))?)
                })
                .transpose()?,
            wire
                .offset
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_from::<_, u64>(value, &child_path(path, "offset"))?)
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
    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalDimensionVector,
> for pse_quantity::dimension::DimensionVector {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalDimensionVector,
        AdapterError,
    > {
        layout(
            <pse_quantity::dimension::DimensionVector as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::dimension::DimensionVector as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "exponents")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalDimensionVector {
            exponents: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::Array8PhysicalRatio,
                >(field_0, &child_path(path, "exponents"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalDimensionVector,
> for pse_quantity::dimension::DimensionVector {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalDimensionVector,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::dimension::DimensionVector as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::dimension::DimensionVector as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "exponents")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::Array8PhysicalRatio,
            >(
                required(wire.exponents.into_option(), &child_path(path, "exponents"))?,
                &child_path(path, "exponents"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalEntityKind,
> for pse_quantity::entity_kind::EntityKind {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalEntityKind,
        AdapterError,
    > {
        layout(
            <pse_quantity::entity_kind::EntityKind as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::entity_kind::EntityKind as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "name")],
            path,
        )?;
        let (field_0, field_1) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalEntityKind {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::EntityKindId,
                >(field_0, &child_path(path, "id"))?,
            ),
            name: native_into::<_, String>(field_1, &child_path(path, "name"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalEntityKind,
> for pse_quantity::entity_kind::EntityKind {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalEntityKind,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::entity_kind::EntityKind as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::entity_kind::EntityKind as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "name")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::EntityKindId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<_, String>(wire.name, &child_path(path, "name"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalInputConversionDeclaration,
> for pse_quantity::InputConversion {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalInputConversionDeclaration,
        AdapterError,
    > {
        layout(
            <pse_quantity::InputConversion as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::InputConversion as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "operand"), (2, "conversion")],
            path,
        )?;
        let (field_0, field_1) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalInputConversionDeclaration {
            operand: native_into::<_, u32>(field_0, &child_path(path, "operand"))?,
            conversion: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::ConversionId,
                >(field_1, &child_path(path, "conversion"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalInputConversionDeclaration,
> for pse_quantity::InputConversion {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalInputConversionDeclaration,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::InputConversion as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::InputConversion as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "operand"), (2, "conversion")],
            path,
        )?;
        let fields = (
            native_from::<_, u32>(wire.operand, &child_path(path, "operand"))?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::ConversionId,
            >(
                required(
                    wire.conversion.into_option(),
                    &child_path(path, "conversion"),
                )?,
                &child_path(path, "conversion"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalOperationDeclaration,
> for pse_quantity::QuantityOperation {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalOperationDeclaration,
        AdapterError,
    > {
        layout(
            <pse_quantity::QuantityOperation as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::QuantityOperation as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "id"),
                (2, "opcode"),
                (3, "input_kinds"),
                (4, "result_kind"),
                (5, "basis_rule"),
                (6, "reference_rule"),
                (7, "scale_rule"),
                (8, "shape_rule"),
                (9, "basis_source"),
                (10, "reference_source"),
                (11, "scale_source"),
                (12, "shape_source"),
                (13, "subject_rule"),
                (14, "subject_source"),
                (15, "result_subject_kind"),
                (16, "result_basis"),
                (17, "result_reference_state"),
                (18, "input_conversions"),
                (19, "precondition_invariants"),
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
            field_15,
            field_16,
            field_17,
            field_18,
        ) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalOperationDeclaration {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::OperationId,
                >(field_0, &child_path(path, "id"))?,
            ),
            opcode: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::Opcode,
                >(field_1, &child_path(path, "opcode"))?,
            ),
            input_kinds: {
                let current_path = &child_path(path, "input_kinds");
                items(field_2)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::QuantityKindId,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            result_kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityKindId,
                >(field_3, &child_path(path, "result_kind"))?,
            ),
            basis_rule: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::BasisRule,
                >(field_4, &child_path(path, "basis_rule"))?,
            ),
            reference_rule: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceRule,
                >(field_5, &child_path(path, "reference_rule"))?,
            ),
            scale_rule: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityScaleRule,
                >(field_6, &child_path(path, "scale_rule"))?,
            ),
            shape_rule: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityShapeRule,
                >(field_7, &child_path(path, "shape_rule"))?,
            ),
            basis_source: field_8
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_into::<_, u32>(value, &child_path(path, "basis_source"))?)
                })
                .transpose()?,
            reference_source: field_9
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            u32,
                        >(value, &child_path(path, "reference_source"))?,
                    )
                })
                .transpose()?,
            scale_source: field_10
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_into::<_, u32>(value, &child_path(path, "scale_source"))?)
                })
                .transpose()?,
            shape_source: field_11
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_into::<_, u32>(value, &child_path(path, "shape_source"))?)
                })
                .transpose()?,
            subject_rule: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::SubjectRule,
                >(field_12, &child_path(path, "subject_rule"))?,
            ),
            subject_source: field_13
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            u32,
                        >(value, &child_path(path, "subject_source"))?,
                    )
                })
                .transpose()?,
            result_subject_kind: field_14
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::EntityKindId,
                        >(value, &child_path(path, "result_subject_kind"))?,
                    )
                })
                .transpose()?
                .into(),
            result_basis: field_15
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::BasisId,
                        >(value, &child_path(path, "result_basis"))?,
                    )
                })
                .transpose()?
                .into(),
            result_reference_state: field_16
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceStateId,
                        >(value, &child_path(path, "result_reference_state"))?,
                    )
                })
                .transpose()?
                .into(),
            input_conversions: {
                let current_path = &child_path(path, "input_conversions");
                items(field_17)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalInputConversionDeclaration,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            precondition_invariants: {
                let current_path = &child_path(path, "precondition_invariants");
                items(field_18)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::InvariantId,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalOperationDeclaration,
> for pse_quantity::QuantityOperation {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalOperationDeclaration,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::QuantityOperation as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::QuantityOperation as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "id"),
                (2, "opcode"),
                (3, "input_kinds"),
                (4, "result_kind"),
                (5, "basis_rule"),
                (6, "reference_rule"),
                (7, "scale_rule"),
                (8, "shape_rule"),
                (9, "basis_source"),
                (10, "reference_source"),
                (11, "scale_source"),
                (12, "shape_source"),
                (13, "subject_rule"),
                (14, "subject_source"),
                (15, "result_subject_kind"),
                (16, "result_basis"),
                (17, "result_reference_state"),
                (18, "input_conversions"),
                (19, "precondition_invariants"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::OperationId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::Opcode,
            >(
                required(wire.opcode.into_option(), &child_path(path, "opcode"))?,
                &child_path(path, "opcode"),
            )?,
            {
                let current_path = &child_path(path, "input_kinds");
                let values = wire
                    .input_kinds
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::QuantityKindId,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::QuantityKindId,
            >(
                required(
                    wire.result_kind.into_option(),
                    &child_path(path, "result_kind"),
                )?,
                &child_path(path, "result_kind"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::BasisRule,
            >(
                required(
                    wire.basis_rule.into_option(),
                    &child_path(path, "basis_rule"),
                )?,
                &child_path(path, "basis_rule"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceRule,
            >(
                required(
                    wire.reference_rule.into_option(),
                    &child_path(path, "reference_rule"),
                )?,
                &child_path(path, "reference_rule"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::QuantityScaleRule,
            >(
                required(
                    wire.scale_rule.into_option(),
                    &child_path(path, "scale_rule"),
                )?,
                &child_path(path, "scale_rule"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::QuantityShapeRule,
            >(
                required(
                    wire.shape_rule.into_option(),
                    &child_path(path, "shape_rule"),
                )?,
                &child_path(path, "shape_rule"),
            )?,
            wire
                .basis_source
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_from::<_, u32>(value, &child_path(path, "basis_source"))?)
                })
                .transpose()?,
            wire
                .reference_source
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            u32,
                        >(value, &child_path(path, "reference_source"))?,
                    )
                })
                .transpose()?,
            wire
                .scale_source
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_from::<_, u32>(value, &child_path(path, "scale_source"))?)
                })
                .transpose()?,
            wire
                .shape_source
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_from::<_, u32>(value, &child_path(path, "shape_source"))?)
                })
                .transpose()?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::SubjectRule,
            >(
                required(
                    wire.subject_rule.into_option(),
                    &child_path(path, "subject_rule"),
                )?,
                &child_path(path, "subject_rule"),
            )?,
            wire
                .subject_source
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            u32,
                        >(value, &child_path(path, "subject_source"))?,
                    )
                })
                .transpose()?,
            wire
                .result_subject_kind
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::EntityKindId,
                        >(value, &child_path(path, "result_subject_kind"))?,
                    )
                })
                .transpose()?,
            wire
                .result_basis
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::BasisId,
                        >(value, &child_path(path, "result_basis"))?,
                    )
                })
                .transpose()?,
            wire
                .result_reference_state
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceStateId,
                        >(value, &child_path(path, "result_reference_state"))?,
                    )
                })
                .transpose()?,
            {
                let current_path = &child_path(path, "input_conversions");
                let values = wire
                    .input_conversions
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalInputConversionDeclaration,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "precondition_invariants");
                let values = wire
                    .precondition_invariants
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::InvariantId,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalPrerequisiteDeclaration,
> for pse_quantity::PhysicalPrecondition {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalPrerequisiteDeclaration,
        AdapterError,
    > {
        layout(
            <pse_quantity::PhysicalPrecondition as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::PhysicalPrecondition as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "operand_positions"), (3, "requirement")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalPrerequisiteDeclaration {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::InvariantId,
                >(field_0, &child_path(path, "id"))?,
            ),
            operand_positions: {
                let current_path = &child_path(path, "operand_positions");
                items(field_1)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                u32,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            requirement: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalPrerequisitePredicate,
                >(field_2, &child_path(path, "requirement"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalPrerequisiteDeclaration,
> for pse_quantity::PhysicalPrecondition {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalPrerequisiteDeclaration,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::PhysicalPrecondition as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::PhysicalPrecondition as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "operand_positions"), (3, "requirement")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::InvariantId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            {
                let current_path = &child_path(path, "operand_positions");
                let values = wire
                    .operand_positions
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                u32,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalPrerequisitePredicate,
            >(
                required(
                    wire.requirement.into_option(),
                    &child_path(path, "requirement"),
                )?,
                &child_path(path, "requirement"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalPrerequisitePredicate,
> for pse_quantity::PhysicalRequirement {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalPrerequisitePredicate,
        AdapterError,
    > {
        let value = self;
        layout(
            <pse_quantity::PhysicalRequirement as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <pse_quantity::PhysicalRequirement as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "required")],
            path,
        )?;
        let value = match <pse_quantity::PhysicalRequirement as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "EqualOperandBases");
                let path = alternative_path.as_str();
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalPrerequisitePredicate {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::physical_prerequisite_predicate::Kind::EqualOperandBases(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::physical_prerequisite_predicate::Alternative1 {
                                required: field_0
                                    .map(|value| -> Result<_, AdapterError> {
                                        Ok(
                                            native_into::<
                                                _,
                                                crate::math::native_root_generated::pb::pse::native_root::v2::BasisId,
                                            >(value, &child_path(path, "required"))?,
                                        )
                                    })
                                    .transpose()?
                                    .into(),
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
            <pse_quantity::PhysicalRequirement as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <pse_quantity::PhysicalRequirement as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "required"), (2, "match_shape")],
            path,
        )?;
        let value = match <pse_quantity::PhysicalRequirement as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "OperandQuantityContract");
                let path = alternative_path.as_str();
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalPrerequisitePredicate {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::physical_prerequisite_predicate::Kind::OperandQuantityContract(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::physical_prerequisite_predicate::Alternative2 {
                                required: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::math::native_root_generated::pb::pse::native_root::v2::QuantityTypeId,
                                    >(field_0, &child_path(path, "required"))?,
                                ),
                                match_shape: native_into::<
                                    _,
                                    bool,
                                >(field_1, &child_path(path, "match_shape"))?,
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
            <pse_quantity::PhysicalRequirement as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::VERSION,
            <pse_quantity::PhysicalRequirement as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "required"), (2, "match_shape")],
            path,
        )?;
        let value = match <pse_quantity::PhysicalRequirement as pse_contract::boundary_projection::ProjectionAlternative<
            3,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "SameReferenceDifferences");
                let path = alternative_path.as_str();
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalPrerequisitePredicate {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::physical_prerequisite_predicate::Kind::SameReferenceDifferences(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::physical_prerequisite_predicate::Alternative3 {
                                required: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::math::native_root_generated::pb::pse::native_root::v2::QuantityTypeId,
                                    >(field_0, &child_path(path, "required"))?,
                                ),
                                match_shape: native_into::<
                                    _,
                                    bool,
                                >(field_1, &child_path(path, "match_shape"))?,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalPrerequisitePredicate,
> for pse_quantity::PhysicalRequirement {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalPrerequisitePredicate,
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
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::physical_prerequisite_predicate::Kind::EqualOperandBases(
                wire,
            ) => {
                let alternative_path = child_path(path, "EqualOperandBases");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_quantity::PhysicalRequirement as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_quantity::PhysicalRequirement as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "required")],
                    path,
                )?;
                let fields = (
                    wire
                        .required
                        .into_option()
                        .map(|value| -> Result<_, AdapterError> {
                            Ok(
                                native_from::<
                                    _,
                                    crate::math::native_root_generated::pb::pse::native_root::v2::BasisId,
                                >(value, &child_path(path, "required"))?,
                            )
                        })
                        .transpose()?,
                );
                Ok(
                    <pse_quantity::PhysicalRequirement as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::physical_prerequisite_predicate::Kind::OperandQuantityContract(
                wire,
            ) => {
                let alternative_path = child_path(path, "OperandQuantityContract");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_quantity::PhysicalRequirement as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_quantity::PhysicalRequirement as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "required"), (2, "match_shape")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::math::native_root_generated::pb::pse::native_root::v2::QuantityTypeId,
                    >(
                        required(
                            wire.required.into_option(),
                            &child_path(path, "required"),
                        )?,
                        &child_path(path, "required"),
                    )?,
                    native_from::<
                        _,
                        bool,
                    >(wire.match_shape, &child_path(path, "match_shape"))?,
                );
                Ok(
                    <pse_quantity::PhysicalRequirement as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::physical_prerequisite_predicate::Kind::SameReferenceDifferences(
                wire,
            ) => {
                let alternative_path = child_path(path, "SameReferenceDifferences");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_quantity::PhysicalRequirement as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_quantity::PhysicalRequirement as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "required"), (2, "match_shape")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::math::native_root_generated::pb::pse::native_root::v2::QuantityTypeId,
                    >(
                        required(
                            wire.required.into_option(),
                            &child_path(path, "required"),
                        )?,
                        &child_path(path, "required"),
                    )?,
                    native_from::<
                        _,
                        bool,
                    >(wire.match_shape, &child_path(path, "match_shape"))?,
                );
                Ok(
                    <pse_quantity::PhysicalRequirement as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalQuantityKind,
> for pse_quantity::kind::QuantityKind {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalQuantityKind,
        AdapterError,
    > {
        layout(
            <pse_quantity::kind::QuantityKind as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::kind::QuantityKind as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "id"),
                (2, "dimension"),
                (3, "extensive"),
                (4, "addition_kind"),
                (5, "category"),
                (6, "definition"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalQuantityKind {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityKindId,
                >(field_0, &child_path(path, "id"))?,
            ),
            dimension: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalDimensionVector,
                >(field_1, &child_path(path, "dimension"))?,
            ),
            extensive: native_into::<_, bool>(field_2, &child_path(path, "extensive"))?,
            addition_kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityAdditionKind,
                >(field_3, &child_path(path, "addition_kind"))?,
            ),
            category: field_4
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::QuantityKindCategory,
                        >(value, &child_path(path, "category"))?,
                    )
                })
                .transpose()?
                .into(),
            definition: field_5
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::KindDefinition,
                        >(value, &child_path(path, "definition"))?,
                    )
                })
                .transpose()?
                .into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalQuantityKind,
> for pse_quantity::kind::QuantityKind {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalQuantityKind,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::kind::QuantityKind as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::kind::QuantityKind as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "id"),
                (2, "dimension"),
                (3, "extensive"),
                (4, "addition_kind"),
                (5, "category"),
                (6, "definition"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::QuantityKindId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalDimensionVector,
            >(
                required(wire.dimension.into_option(), &child_path(path, "dimension"))?,
                &child_path(path, "dimension"),
            )?,
            native_from::<_, bool>(wire.extensive, &child_path(path, "extensive"))?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::QuantityAdditionKind,
            >(
                required(
                    wire.addition_kind.into_option(),
                    &child_path(path, "addition_kind"),
                )?,
                &child_path(path, "addition_kind"),
            )?,
            wire
                .category
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::QuantityKindCategory,
                        >(value, &child_path(path, "category"))?,
                    )
                })
                .transpose()?,
            wire
                .definition
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::KindDefinition,
                        >(value, &child_path(path, "definition"))?,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalRatio,
> for pse_quantity::dimension::Ratio {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalRatio,
        AdapterError,
    > {
        layout(
            <pse_quantity::dimension::Ratio as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::dimension::Ratio as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "num"), (2, "den")],
            path,
        )?;
        let (field_0, field_1) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalRatio {
            num: native_into::<_, i32>(field_0, &child_path(path, "num"))?,
            den: native_into::<_, i32>(field_1, &child_path(path, "den"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalRatio,
> for pse_quantity::dimension::Ratio {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalRatio,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::dimension::Ratio as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::dimension::Ratio as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "num"), (2, "den")],
            path,
        )?;
        let fields = (
            native_from::<_, i32>(wire.num, &child_path(path, "num"))?,
            native_from::<_, i32>(wire.den, &child_path(path, "den"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalRegistryDeclarations,
> for pse_quantity::QuantityRegistryBuilder {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalRegistryDeclarations,
        AdapterError,
    > {
        layout(
            <pse_quantity::QuantityRegistryBuilder as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::QuantityRegistryBuilder as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "entity_kinds"),
                (2, "units"),
                (3, "defined_units"),
                (4, "kinds"),
                (5, "derived_kinds"),
                (6, "bases"),
                (7, "reference_states"),
                (8, "quantity_types"),
                (9, "conversions"),
                (10, "operations"),
                (11, "reduction_domains"),
                (12, "unit_sets"),
                (13, "neutral"),
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
        ) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalRegistryDeclarations {
            entity_kinds: {
                let current_path = &child_path(path, "entity_kinds");
                items(field_0)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalEntityKind,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            units: {
                let current_path = &child_path(path, "units");
                items(field_1)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalUnit,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            defined_units: {
                let current_path = &child_path(path, "defined_units");
                items(field_2)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::DeclaredDefinedUnit,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            kinds: {
                let current_path = &child_path(path, "kinds");
                items(field_3)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalQuantityKind,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            derived_kinds: {
                let current_path = &child_path(path, "derived_kinds");
                items(field_4)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::DeclaredDerivedKind,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            bases: {
                let current_path = &child_path(path, "bases");
                items(field_5)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalBasis,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            reference_states: {
                let current_path = &child_path(path, "reference_states");
                items(field_6)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceState,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            quantity_types: {
                let current_path = &child_path(path, "quantity_types");
                items(field_7)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::QuantityType,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            conversions: {
                let current_path = &child_path(path, "conversions");
                items(field_8)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalConversionDeclaration,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            operations: {
                let current_path = &child_path(path, "operations");
                items(field_9)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalOperationDeclaration,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            reduction_domains: {
                let current_path = &child_path(path, "reduction_domains");
                items(field_10)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::Tuple2OfOperationIdAndEntityKindId,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            unit_sets: {
                let current_path = &child_path(path, "unit_sets");
                items(field_11)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalUnitSet,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            neutral: {
                let current_path = &child_path(path, "neutral");
                items(field_12)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::QuantityTypeId,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalRegistryDeclarations,
> for pse_quantity::QuantityRegistryBuilder {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalRegistryDeclarations,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::QuantityRegistryBuilder as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::QuantityRegistryBuilder as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "entity_kinds"),
                (2, "units"),
                (3, "defined_units"),
                (4, "kinds"),
                (5, "derived_kinds"),
                (6, "bases"),
                (7, "reference_states"),
                (8, "quantity_types"),
                (9, "conversions"),
                (10, "operations"),
                (11, "reduction_domains"),
                (12, "unit_sets"),
                (13, "neutral"),
            ],
            path,
        )?;
        let fields = (
            {
                let current_path = &child_path(path, "entity_kinds");
                let values = wire
                    .entity_kinds
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalEntityKind,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "units");
                let values = wire
                    .units
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalUnit,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "defined_units");
                let values = wire
                    .defined_units
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::DeclaredDefinedUnit,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "kinds");
                let values = wire
                    .kinds
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalQuantityKind,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "derived_kinds");
                let values = wire
                    .derived_kinds
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::DeclaredDerivedKind,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "bases");
                let values = wire
                    .bases
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalBasis,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "reference_states");
                let values = wire
                    .reference_states
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceState,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "quantity_types");
                let values = wire
                    .quantity_types
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::QuantityType,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "conversions");
                let values = wire
                    .conversions
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalConversionDeclaration,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "operations");
                let values = wire
                    .operations
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalOperationDeclaration,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "reduction_domains");
                let values = wire
                    .reduction_domains
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::Tuple2OfOperationIdAndEntityKindId,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "unit_sets");
                let values = wire
                    .unit_sets
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalUnitSet,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "neutral");
                let values = wire
                    .neutral
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::QuantityTypeId,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalUnit,
> for pse_quantity::unit::Unit {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalUnit,
        AdapterError,
    > {
        layout(
            <pse_quantity::unit::Unit as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::unit::Unit as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "id"),
                (2, "symbol"),
                (3, "dimension"),
                (4, "scale_to_canonical"),
                (5, "offset_to_canonical"),
                (6, "is_affine"),
                (7, "reference_state"),
                (8, "definition"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5, field_6, field_7) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalUnit {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::UnitId,
                >(field_0, &child_path(path, "id"))?,
            ),
            symbol: native_into::<_, String>(field_1, &child_path(path, "symbol"))?,
            dimension: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalDimensionVector,
                >(field_2, &child_path(path, "dimension"))?,
            ),
            scale_to_canonical: native_into::<
                _,
                u64,
            >(field_3, &child_path(path, "scale_to_canonical"))?,
            offset_to_canonical: native_into::<
                _,
                u64,
            >(field_4, &child_path(path, "offset_to_canonical"))?,
            is_affine: native_into::<_, bool>(field_5, &child_path(path, "is_affine"))?,
            reference_state: field_6
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceStateId,
                        >(value, &child_path(path, "reference_state"))?,
                    )
                })
                .transpose()?
                .into(),
            definition: field_7
                .map(|value| -> Result<_, AdapterError> {
                    Ok(crate::math::native_root_generated::pb::pse::native_root::v2::physical_unit::Field8Value {
                        value: {
                            let current_path = &child_path(path, "definition");
                            items(value)
                                .enumerate()
                                .map(|(index, value)| -> Result<_, AdapterError> {
                                    Ok(
                                        native_into::<
                                            _,
                                            crate::math::native_root_generated::pb::pse::native_root::v2::UnitFactor,
                                        >(value, &format!("{path}[{index}]", path = current_path))?,
                                    )
                                })
                                .collect::<Result<Vec<_>, _>>()?
                        },
                        ..Default::default()
                    })
                })
                .transpose()?
                .into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalUnit,
> for pse_quantity::unit::Unit {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalUnit,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::unit::Unit as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::unit::Unit as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "id"),
                (2, "symbol"),
                (3, "dimension"),
                (4, "scale_to_canonical"),
                (5, "offset_to_canonical"),
                (6, "is_affine"),
                (7, "reference_state"),
                (8, "definition"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::UnitId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<_, String>(wire.symbol, &child_path(path, "symbol"))?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalDimensionVector,
            >(
                required(wire.dimension.into_option(), &child_path(path, "dimension"))?,
                &child_path(path, "dimension"),
            )?,
            native_from::<
                _,
                u64,
            >(wire.scale_to_canonical, &child_path(path, "scale_to_canonical"))?,
            native_from::<
                _,
                u64,
            >(wire.offset_to_canonical, &child_path(path, "offset_to_canonical"))?,
            native_from::<_, bool>(wire.is_affine, &child_path(path, "is_affine"))?,
            wire
                .reference_state
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceStateId,
                        >(value, &child_path(path, "reference_state"))?,
                    )
                })
                .transpose()?,
            wire
                .definition
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok({
                        let wrapped = value;
                        unknown_fields(
                            &wrapped.__buffa_unknown_fields,
                            &child_path(path, "definition"),
                        )?;
                        {
                            let current_path = &child_path(path, "definition");
                            let values = wrapped
                                .value
                                .into_iter()
                                .enumerate()
                                .map(|(index, value)| -> Result<_, AdapterError> {
                                    Ok(
                                        native_from::<
                                            _,
                                            crate::math::native_root_generated::pb::pse::native_root::v2::UnitFactor,
                                        >(value, &format!("{path}[{index}]", path = current_path))?,
                                    )
                                })
                                .collect::<Result<Vec<_>, _>>()?;
                            from_items(values, current_path)?
                        }
                    })
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
    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalUnitSet,
> for pse_quantity::UnitSet {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalUnitSet,
        AdapterError,
    > {
        layout(
            <pse_quantity::UnitSet as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::UnitSet as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "base")],
            path,
        )?;
        let (field_0, field_1) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalUnitSet {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::UnitSetId,
                >(field_0, &child_path(path, "id"))?,
            ),
            base: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::Array8OptionalUnitId,
                >(field_1, &child_path(path, "base"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalUnitSet,
> for pse_quantity::UnitSet {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalUnitSet,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::UnitSet as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::UnitSet as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "base")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::UnitSetId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::Array8OptionalUnitId,
            >(
                required(wire.base.into_option(), &child_path(path, "base"))?,
                &child_path(path, "base"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::PlatformPathBytes,
> for pse_contract::boundary_projection::PlatformPathBytes {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::PlatformPathBytes,
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
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::PlatformPathBytes {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::platform_path_bytes::Kind::Unix(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::platform_path_bytes::Alternative1 {
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
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::PlatformPathBytes {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::platform_path_bytes::Kind::Windows(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::platform_path_bytes::Alternative2 {
                                value: {
                                    let current_path = &child_path(path, "value");
                                    items(field_0)
                                        .enumerate()
                                        .map(|(index, value)| -> Result<_, AdapterError> {
                                            Ok(
                                                native_into::<
                                                    _,
                                                    u32,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::PlatformPathBytes,
> for pse_contract::boundary_projection::PlatformPathBytes {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::PlatformPathBytes,
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
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::platform_path_bytes::Kind::Unix(
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
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::platform_path_bytes::Kind::Windows(
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
                                Ok(
                                    native_from::<
                                        _,
                                        u32,
                                    >(value, &format!("{path}[{index}]", path = current_path))?,
                                )
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
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityAdditionKind,
> for pse_quantity::enums::QuantityAdditionKind {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::QuantityAdditionKind,
        AdapterError,
    > {
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::QuantityAdditionKind {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityAdditionKind,
> for pse_quantity::enums::QuantityAdditionKind {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::QuantityAdditionKind,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityKindCategory,
> for pse_quantity::enums::QuantityKindCategory {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::QuantityKindCategory,
        AdapterError,
    > {
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::QuantityKindCategory {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityKindCategory,
> for pse_quantity::enums::QuantityKindCategory {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::QuantityKindCategory,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityKindId,
> for pse_quantity::ids::QuantityKindId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::QuantityKindId,
        AdapterError,
    > {
        layout(
            <pse_quantity::ids::QuantityKindId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::QuantityKindId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::QuantityKindId {
            id: Some(
                pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                    field_0,
                ),
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityKindId,
> for pse_quantity::ids::QuantityKindId {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::QuantityKindId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::QuantityKindId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::QuantityKindId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                required(wire.id, &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityScaleRule,
> for pse_quantity::enums::QuantityScaleRule {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::QuantityScaleRule,
        AdapterError,
    > {
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::QuantityScaleRule {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityScaleRule,
> for pse_quantity::enums::QuantityScaleRule {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::QuantityScaleRule,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityShapeRule,
> for pse_quantity::enums::QuantityShapeRule {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::QuantityShapeRule,
        AdapterError,
    > {
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::QuantityShapeRule {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityShapeRule,
> for pse_quantity::enums::QuantityShapeRule {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::QuantityShapeRule,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityType,
> for pse_quantity::quantity_type::QuantityType {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::QuantityType,
        AdapterError,
    > {
        layout(
            <pse_quantity::quantity_type::QuantityType as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::quantity_type::QuantityType as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "id"),
                (2, "name"),
                (3, "key"),
                (4, "canonical_unit"),
                (5, "nominal_magnitude"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::QuantityType {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityTypeId,
                >(field_0, &child_path(path, "id"))?,
            ),
            name: field_1
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_into::<_, String>(value, &child_path(path, "name"))?)
                })
                .transpose()?,
            key: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityTypeKey,
                >(field_2, &child_path(path, "key"))?,
            ),
            canonical_unit: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::UnitId,
                >(field_3, &child_path(path, "canonical_unit"))?,
            ),
            nominal_magnitude: field_4
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            u64,
                        >(value, &child_path(path, "nominal_magnitude"))?,
                    )
                })
                .transpose()?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityType,
> for pse_quantity::quantity_type::QuantityType {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::QuantityType,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::quantity_type::QuantityType as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::quantity_type::QuantityType as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "id"),
                (2, "name"),
                (3, "key"),
                (4, "canonical_unit"),
                (5, "nominal_magnitude"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::QuantityTypeId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            wire
                .name
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_from::<_, String>(value, &child_path(path, "name"))?)
                })
                .transpose()?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::QuantityTypeKey,
            >(
                required(wire.key.into_option(), &child_path(path, "key"))?,
                &child_path(path, "key"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::UnitId,
            >(
                required(
                    wire.canonical_unit.into_option(),
                    &child_path(path, "canonical_unit"),
                )?,
                &child_path(path, "canonical_unit"),
            )?,
            wire
                .nominal_magnitude
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            u64,
                        >(value, &child_path(path, "nominal_magnitude"))?,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityTypeId,
> for pse_quantity::ids::QuantityTypeId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::QuantityTypeId,
        AdapterError,
    > {
        layout(
            <pse_quantity::ids::QuantityTypeId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::QuantityTypeId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::QuantityTypeId {
            id: Some(
                pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                    field_0,
                ),
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityTypeId,
> for pse_quantity::ids::QuantityTypeId {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::QuantityTypeId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::QuantityTypeId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::QuantityTypeId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                required(wire.id, &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityTypeKey,
> for pse_quantity::quantity_type::QuantityTypeKey {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::QuantityTypeKey,
        AdapterError,
    > {
        layout(
            <pse_quantity::quantity_type::QuantityTypeKey as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::quantity_type::QuantityTypeKey as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "kind"),
                (2, "basis"),
                (3, "reference_state"),
                (4, "scale_kind"),
                (5, "shape"),
                (6, "subject_kind"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::QuantityTypeKey {
            kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityKindId,
                >(field_0, &child_path(path, "kind"))?,
            ),
            basis: field_1
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::BasisId,
                        >(value, &child_path(path, "basis"))?,
                    )
                })
                .transpose()?
                .into(),
            reference_state: field_2
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceStateId,
                        >(value, &child_path(path, "reference_state"))?,
                    )
                })
                .transpose()?
                .into(),
            scale_kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::ScaleKind,
                >(field_3, &child_path(path, "scale_kind"))?,
            ),
            shape: {
                let current_path = &child_path(path, "shape");
                items(field_4)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::EntityKindId,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            subject_kind: field_5
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::EntityKindId,
                        >(value, &child_path(path, "subject_kind"))?,
                    )
                })
                .transpose()?
                .into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityTypeKey,
> for pse_quantity::quantity_type::QuantityTypeKey {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::QuantityTypeKey,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::quantity_type::QuantityTypeKey as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::quantity_type::QuantityTypeKey as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "kind"),
                (2, "basis"),
                (3, "reference_state"),
                (4, "scale_kind"),
                (5, "shape"),
                (6, "subject_kind"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::QuantityKindId,
            >(
                required(wire.kind.into_option(), &child_path(path, "kind"))?,
                &child_path(path, "kind"),
            )?,
            wire
                .basis
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::BasisId,
                        >(value, &child_path(path, "basis"))?,
                    )
                })
                .transpose()?,
            wire
                .reference_state
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceStateId,
                        >(value, &child_path(path, "reference_state"))?,
                    )
                })
                .transpose()?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::ScaleKind,
            >(
                required(
                    wire.scale_kind.into_option(),
                    &child_path(path, "scale_kind"),
                )?,
                &child_path(path, "scale_kind"),
            )?,
            {
                let current_path = &child_path(path, "shape");
                let values = wire
                    .shape
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::EntityKindId,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            wire
                .subject_kind
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::EntityKindId,
                        >(value, &child_path(path, "subject_kind"))?,
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
impl NativeInto<crate::math::native_root_generated::pb::pse::native_root::v2::RateBasis>
for pse_quantity::enums::RateBasis {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::RateBasis,
        AdapterError,
    > {
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::RateBasis {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<crate::math::native_root_generated::pb::pse::native_root::v2::RateBasis>
for pse_quantity::enums::RateBasis {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::RateBasis,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceCondition,
> for pse_quantity::reference_state::ReferenceCondition {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceCondition,
        AdapterError,
    > {
        layout(
            <pse_quantity::reference_state::ReferenceCondition as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::reference_state::ReferenceCondition as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value"), (2, "quantity_type"), (3, "unit")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceCondition {
            value: native_into::<_, u64>(field_0, &child_path(path, "value"))?,
            quantity_type: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::QuantityTypeId,
                >(field_1, &child_path(path, "quantity_type"))?,
            ),
            unit: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::UnitId,
                >(field_2, &child_path(path, "unit"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceCondition,
> for pse_quantity::reference_state::ReferenceCondition {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceCondition,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::reference_state::ReferenceCondition as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::reference_state::ReferenceCondition as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value"), (2, "quantity_type"), (3, "unit")],
            path,
        )?;
        let fields = (
            native_from::<_, u64>(wire.value, &child_path(path, "value"))?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::QuantityTypeId,
            >(
                required(
                    wire.quantity_type.into_option(),
                    &child_path(path, "quantity_type"),
                )?,
                &child_path(path, "quantity_type"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::UnitId,
            >(
                required(wire.unit.into_option(), &child_path(path, "unit"))?,
                &child_path(path, "unit"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceRule,
> for pse_quantity::enums::ReferenceRule {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceRule,
        AdapterError,
    > {
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceRule {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceRule,
> for pse_quantity::enums::ReferenceRule {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceRule,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceState,
> for pse_quantity::reference_state::ReferenceState {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceState,
        AdapterError,
    > {
        layout(
            <pse_quantity::reference_state::ReferenceState as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::reference_state::ReferenceState as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "id"),
                (2, "name"),
                (3, "kind"),
                (4, "temperature"),
                (5, "pressure"),
                (6, "include_enthalpy_of_formation"),
                (7, "subject"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5, field_6) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceState {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceStateId,
                >(field_0, &child_path(path, "id"))?,
            ),
            name: native_into::<_, String>(field_1, &child_path(path, "name"))?,
            kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceStateKind,
                >(field_2, &child_path(path, "kind"))?,
            ),
            temperature: field_3
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceCondition,
                        >(value, &child_path(path, "temperature"))?,
                    )
                })
                .transpose()?
                .into(),
            pressure: field_4
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceCondition,
                        >(value, &child_path(path, "pressure"))?,
                    )
                })
                .transpose()?
                .into(),
            include_enthalpy_of_formation: native_into::<
                _,
                bool,
            >(field_5, &child_path(path, "include_enthalpy_of_formation"))?,
            subject: field_6
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                            value,
                        ),
                    )
                })
                .transpose()?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceState,
> for pse_quantity::reference_state::ReferenceState {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceState,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::reference_state::ReferenceState as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::reference_state::ReferenceState as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "id"),
                (2, "name"),
                (3, "kind"),
                (4, "temperature"),
                (5, "pressure"),
                (6, "include_enthalpy_of_formation"),
                (7, "subject"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceStateId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<_, String>(wire.name, &child_path(path, "name"))?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceStateKind,
            >(
                required(wire.kind.into_option(), &child_path(path, "kind"))?,
                &child_path(path, "kind"),
            )?,
            wire
                .temperature
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceCondition,
                        >(value, &child_path(path, "temperature"))?,
                    )
                })
                .transpose()?,
            wire
                .pressure
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceCondition,
                        >(value, &child_path(path, "pressure"))?,
                    )
                })
                .transpose()?,
            native_from::<
                _,
                bool,
            >(
                wire.include_enthalpy_of_formation,
                &child_path(path, "include_enthalpy_of_formation"),
            )?,
            wire
                .subject
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                            value,
                            &child_path(path, "subject"),
                        )?,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceStateId,
> for pse_quantity::ids::ReferenceStateId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceStateId,
        AdapterError,
    > {
        layout(
            <pse_quantity::ids::ReferenceStateId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::ReferenceStateId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceStateId {
            id: Some(
                pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                    field_0,
                ),
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceStateId,
> for pse_quantity::ids::ReferenceStateId {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceStateId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::ReferenceStateId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::ReferenceStateId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                required(wire.id, &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceStateKind,
> for pse_quantity::enums::ReferenceStateKind {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceStateKind,
        AdapterError,
    > {
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceStateKind {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceStateKind,
> for pse_quantity::enums::ReferenceStateKind {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::ReferenceStateKind,
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
impl NativeInto<crate::math::native_root_generated::pb::pse::native_root::v2::ScaleKind>
for pse_quantity::enums::ScaleKind {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::ScaleKind,
        AdapterError,
    > {
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::ScaleKind {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<crate::math::native_root_generated::pb::pse::native_root::v2::ScaleKind>
for pse_quantity::enums::ScaleKind {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::ScaleKind,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::SubjectRule,
> for pse_quantity::enums::SubjectRule {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::SubjectRule,
        AdapterError,
    > {
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::SubjectRule {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::SubjectRule,
> for pse_quantity::enums::SubjectRule {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::SubjectRule,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::Tuple2OfOperationIdAndEntityKindId,
> for (pse_quantity::ids::OperationId, pse_quantity::ids::EntityKindId) {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::Tuple2OfOperationIdAndEntityKindId,
        AdapterError,
    > {
        layout(
            <(
                pse_quantity::ids::OperationId,
                pse_quantity::ids::EntityKindId,
            ) as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <(
                pse_quantity::ids::OperationId,
                pse_quantity::ids::EntityKindId,
            ) as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "first"), (2, "second")],
            path,
        )?;
        let (field_0, field_1) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::Tuple2OfOperationIdAndEntityKindId {
            first: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::OperationId,
                >(field_0, &child_path(path, "first"))?,
            ),
            second: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::EntityKindId,
                >(field_1, &child_path(path, "second"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::Tuple2OfOperationIdAndEntityKindId,
> for (pse_quantity::ids::OperationId, pse_quantity::ids::EntityKindId) {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::Tuple2OfOperationIdAndEntityKindId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <(
                pse_quantity::ids::OperationId,
                pse_quantity::ids::EntityKindId,
            ) as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <(
                pse_quantity::ids::OperationId,
                pse_quantity::ids::EntityKindId,
            ) as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "first"), (2, "second")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::OperationId,
            >(
                required(wire.first.into_option(), &child_path(path, "first"))?,
                &child_path(path, "first"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::EntityKindId,
            >(
                required(wire.second.into_option(), &child_path(path, "second"))?,
                &child_path(path, "second"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<crate::math::native_root_generated::pb::pse::native_root::v2::UnitFactor>
for pse_quantity::unit::UnitFactor {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::UnitFactor,
        AdapterError,
    > {
        layout(
            <pse_quantity::unit::UnitFactor as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::unit::UnitFactor as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "unit"), (2, "exponent")],
            path,
        )?;
        let (field_0, field_1) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::UnitFactor {
            unit: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::UnitId,
                >(field_0, &child_path(path, "unit"))?,
            ),
            exponent: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalRatio,
                >(field_1, &child_path(path, "exponent"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<crate::math::native_root_generated::pb::pse::native_root::v2::UnitFactor>
for pse_quantity::unit::UnitFactor {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::UnitFactor,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::unit::UnitFactor as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::unit::UnitFactor as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "unit"), (2, "exponent")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::UnitId,
            >(
                required(wire.unit.into_option(), &child_path(path, "unit"))?,
                &child_path(path, "unit"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::PhysicalRatio,
            >(
                required(wire.exponent.into_option(), &child_path(path, "exponent"))?,
                &child_path(path, "exponent"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<crate::math::native_root_generated::pb::pse::native_root::v2::UnitId>
for pse_quantity::ids::UnitId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::UnitId,
        AdapterError,
    > {
        layout(
            <pse_quantity::ids::UnitId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::UnitId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::UnitId {
            id: Some(
                pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                    field_0,
                ),
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<crate::math::native_root_generated::pb::pse::native_root::v2::UnitId>
for pse_quantity::ids::UnitId {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::UnitId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::UnitId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::UnitId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                required(wire.id, &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<crate::math::native_root_generated::pb::pse::native_root::v2::UnitSetId>
for pse_quantity::ids::UnitSetId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::UnitSetId,
        AdapterError,
    > {
        layout(
            <pse_quantity::ids::UnitSetId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::UnitSetId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::UnitSetId {
            id: Some(
                pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                    field_0,
                ),
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<crate::math::native_root_generated::pb::pse::native_root::v2::UnitSetId>
for pse_quantity::ids::UnitSetId {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::UnitSetId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::UnitSetId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::UnitSetId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                required(wire.id, &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
/// Encode the actual native `NativeRootInput` and validate outgoing local rules.
pub(crate) fn encode_native_root_input(
    native: crate::math::native_root_protocol::RootInput,
) -> Result<
    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootInput,
    BoundaryProjectionError,
> {
    encode_root(native, "NativeRootInput")
}
/// Adopt known `NativeRootInput` meaning under this package's declared unknown-field policy.
pub(crate) fn decode_native_root_input(
    wire: crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootInput,
) -> Result<crate::math::native_root_protocol::RootInput, BoundaryProjectionError> {
    decode_root(wire, "NativeRootInput")
}
/// Encode the actual native `NativeRootHello` and validate outgoing local rules.
pub(crate) fn encode_native_root_hello(
    native: crate::math::native_root_protocol::Hello,
) -> Result<
    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootHello,
    BoundaryProjectionError,
> {
    encode_root(native, "NativeRootHello")
}
/// Adopt known `NativeRootHello` meaning under this package's declared unknown-field policy.
pub(crate) fn decode_native_root_hello(
    wire: crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootHello,
) -> Result<crate::math::native_root_protocol::Hello, BoundaryProjectionError> {
    decode_root(wire, "NativeRootHello")
}
/// Encode the actual native `NativeRootRequest` and validate outgoing local rules.
pub(crate) fn encode_native_root_request(
    native: crate::math::native_root_protocol::Request,
) -> Result<
    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRequest,
    BoundaryProjectionError,
> {
    encode_root(native, "NativeRootRequest")
}
/// Adopt known `NativeRootRequest` meaning under this package's declared unknown-field policy.
pub(crate) fn decode_native_root_request(
    wire: crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRequest,
) -> Result<crate::math::native_root_protocol::Request, BoundaryProjectionError> {
    decode_root(wire, "NativeRootRequest")
}
/// Encode the actual native `NativeRootResponse` and validate outgoing local rules.
pub(crate) fn encode_native_root_response(
    native: crate::math::native_root_protocol::Response,
) -> Result<
    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootResponse,
    BoundaryProjectionError,
> {
    encode_root(native, "NativeRootResponse")
}
/// Adopt known `NativeRootResponse` meaning under this package's declared unknown-field policy.
pub(crate) fn decode_native_root_response(
    wire: crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootResponse,
) -> Result<crate::math::native_root_protocol::Response, BoundaryProjectionError> {
    decode_root(wire, "NativeRootResponse")
}
