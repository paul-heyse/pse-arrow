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
                    native_into::<
                        _,
                        crate::math::native_root_generated::pb::pse::native_root::v2::ModelingApplicabilityBasis,
                    >(value, &child_path(path, "basis"))
                })
                .transpose()?
                .into(),
            reason: field_11
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<_, String>(value, &child_path(path, "reason"))
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
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                        value,
                        &child_path(path, "id"),
                    )
                })
                .transpose()?,
            wire
                .coverage
                .map(|value| -> Result<_, AdapterError> {
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                        value,
                        &child_path(path, "coverage"),
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
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                            value,
                            &format!("{path}[{index}]", path = current_path),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            wire
                .evidence
                .map(|value| -> Result<_, AdapterError> {
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                        value,
                        &child_path(path, "evidence"),
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
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                            value,
                            &format!("{path}[{index}]", path = current_path),
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
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                            value,
                            &format!("{path}[{index}]", path = current_path),
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
                    native_from::<
                        _,
                        crate::math::native_root_generated::pb::pse::native_root::v2::ModelingApplicabilityBasis,
                    >(value, &child_path(path, "basis"))
                })
                .transpose()?,
            wire
                .reason
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<_, String>(value, &child_path(path, "reason"))
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
    crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityNode,
> for pse_model::applicability::Node {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityNode,
        AdapterError,
    > {
        layout(
            <pse_model::applicability::Node as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::applicability::Node as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "claim"),
                (2, "region"),
                (3, "dependencies"),
                (4, "inputs"),
                (5, "permissions"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityNode {
            claim: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityClaim,
                >(field_0, &child_path(path, "claim"))?,
            ),
            region: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityRegion,
                >(field_1, &child_path(path, "region"))?,
            ),
            dependencies: {
                let current_path = &child_path(path, "dependencies");
                items(field_2)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityNode,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            inputs: {
                let current_path = &child_path(path, "inputs");
                items(field_3)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::Tuple3OfStringAndUsizeAndSemanticId,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            permissions: {
                let current_path = &child_path(path, "permissions");
                items(field_4)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityPermission,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityNode,
> for pse_model::applicability::Node {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityNode,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_model::applicability::Node as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::applicability::Node as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "claim"),
                (2, "region"),
                (3, "dependencies"),
                (4, "inputs"),
                (5, "permissions"),
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
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityRegion,
            >(
                required(wire.region.into_option(), &child_path(path, "region"))?,
                &child_path(path, "region"),
            )?,
            {
                let current_path = &child_path(path, "dependencies");
                let values = wire
                    .dependencies
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityNode,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "inputs");
                let values = wire
                    .inputs
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::Tuple3OfStringAndUsizeAndSemanticId,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "permissions");
                let values = wire
                    .permissions
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityPermission,
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
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityInput,
                        >(value, &format!("{path}[{index}]", path = current_path))
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
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityPermission,
                        >(value, &format!("{path}[{index}]", path = current_path))
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
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                        value,
                        &child_path(path, "instance"),
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
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityInput,
                        >(value, &format!("{path}[{index}]", path = current_path))
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
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityPermission,
                        >(value, &format!("{path}[{index}]", path = current_path))
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
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                            value,
                            &format!("{path}[{index}]", path = current_path),
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
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityRegion,
> for pse_model::applicability::Region {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityRegion,
        AdapterError,
    > {
        let value = self;
        layout(
            <pse_model::applicability::Region as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <pse_model::applicability::Region as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "predicate")],
            path,
        )?;
        let value = match <pse_model::applicability::Region as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Predicate");
                let path = alternative_path.as_str();
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityRegion {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::applicability_region::Kind::Predicate(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::applicability_region::Alternative1 {
                                predicate: native_into::<
                                    _,
                                    u64,
                                >(field_0, &child_path(path, "predicate"))?,
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
            <pse_model::applicability::Region as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <pse_model::applicability::Region as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_model::applicability::Region as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityRegion {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::applicability_region::Kind::Unrestricted(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::applicability_region::Alternative2 {
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
            <pse_model::applicability::Region as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::VERSION,
            <pse_model::applicability::Region as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_model::applicability::Region as pse_contract::boundary_projection::ProjectionAlternative<
            3,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityRegion {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::applicability_region::Kind::Unknown(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::applicability_region::Alternative3 {
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
            <pse_model::applicability::Region as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::VERSION,
            <pse_model::applicability::Region as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "alternatives")],
            path,
        )?;
        let value = match <pse_model::applicability::Region as pse_contract::boundary_projection::ProjectionAlternative<
            4,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Union");
                let path = alternative_path.as_str();
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityRegion {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::applicability_region::Kind::Union(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::applicability_region::Alternative4 {
                                alternatives: {
                                    let current_path = &child_path(path, "alternatives");
                                    items(field_0)
                                        .enumerate()
                                        .map(|(index, value)| -> Result<_, AdapterError> {
                                            native_into::<
                                                _,
                                                crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityNode,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityRegion,
> for pse_model::applicability::Region {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityRegion,
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
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::applicability_region::Kind::Predicate(
                wire,
            ) => {
                let alternative_path = child_path(path, "Predicate");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_model::applicability::Region as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_model::applicability::Region as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "predicate")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        u64,
                    >(wire.predicate, &child_path(path, "predicate"))?,
                );
                Ok(
                    <pse_model::applicability::Region as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::applicability_region::Kind::Unrestricted(
                wire,
            ) => {
                let alternative_path = child_path(path, "Unrestricted");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_model::applicability::Region as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_model::applicability::Region as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_model::applicability::Region as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::applicability_region::Kind::Unknown(
                wire,
            ) => {
                let alternative_path = child_path(path, "Unknown");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_model::applicability::Region as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_model::applicability::Region as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_model::applicability::Region as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::applicability_region::Kind::Union(
                wire,
            ) => {
                let alternative_path = child_path(path, "Union");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_model::applicability::Region as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <pse_model::applicability::Region as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "alternatives")],
                    path,
                )?;
                let fields = (
                    {
                        let current_path = &child_path(path, "alternatives");
                        let values = wire
                            .alternatives
                            .into_iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                native_from::<
                                    _,
                                    crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityNode,
                                >(value, &format!("{path}[{index}]", path = current_path))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_model::applicability::Region as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
        }
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
                        native_into::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
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
                        native_from::<
                            _,
                            u64,
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
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::Array16U8,
                        >(value, &format!("{path}[{index}]", path = current_path))
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
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::Array16U8,
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
                                            native_into::<
                                                _,
                                                crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticOperandContract,
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
                                native_from::<
                                    _,
                                    crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticOperandContract,
                                >(value, &format!("{path}[{index}]", path = current_path))
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
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::Array3Array16U8,
                        >(value, &format!("{path}[{index}]", path = current_path))
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
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::Array3Array16U8,
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
                        native_into::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            variable_nominals: {
                let current_path = &child_path(path, "variable_nominals");
                items(field_1)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            variable_tolerance: {
                let current_path = &child_path(path, "variable_tolerance");
                items(field_2)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            residual_tolerance: {
                let current_path = &child_path(path, "residual_tolerance");
                items(field_3)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
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
                        native_from::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
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
                        native_from::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
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
                        native_from::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
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
                        native_from::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
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
    crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeOrder,
> for pse_kernels::DerivativeOrder {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeOrder,
        AdapterError,
    > {
        let value = self;
        layout(
            <pse_kernels::DerivativeOrder as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <pse_kernels::DerivativeOrder as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_kernels::DerivativeOrder as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeOrder {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::kernel_derivative_order::Kind::Value(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::kernel_derivative_order::Alternative1 {
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
            <pse_kernels::DerivativeOrder as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <pse_kernels::DerivativeOrder as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_kernels::DerivativeOrder as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeOrder {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::kernel_derivative_order::Kind::First(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::kernel_derivative_order::Alternative2 {
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
            <pse_kernels::DerivativeOrder as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::VERSION,
            <pse_kernels::DerivativeOrder as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_kernels::DerivativeOrder as pse_contract::boundary_projection::ProjectionAlternative<
            3,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeOrder {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::kernel_derivative_order::Kind::Second(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::kernel_derivative_order::Alternative3 {
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
    crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeOrder,
> for pse_kernels::DerivativeOrder {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeOrder,
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
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::kernel_derivative_order::Kind::Value(
                wire,
            ) => {
                let alternative_path = child_path(path, "Value");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_kernels::DerivativeOrder as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_kernels::DerivativeOrder as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_kernels::DerivativeOrder as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::kernel_derivative_order::Kind::First(
                wire,
            ) => {
                let alternative_path = child_path(path, "First");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_kernels::DerivativeOrder as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_kernels::DerivativeOrder as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_kernels::DerivativeOrder as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::kernel_derivative_order::Kind::Second(
                wire,
            ) => {
                let alternative_path = child_path(path, "Second");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_kernels::DerivativeOrder as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_kernels::DerivativeOrder as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_kernels::DerivativeOrder as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeRequirements,
> for pse_kernels::DerivativeRequirements {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeRequirements,
        AdapterError,
    > {
        layout(
            <pse_kernels::DerivativeRequirements as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_kernels::DerivativeRequirements as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "residual_available"),
                (2, "output_smoothness"),
                (3, "selector_neighborhood"),
                (4, "inner_minimum"),
                (5, "requested_output"),
                (6, "residual_compilation"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeRequirements {
            residual_available: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeOrder,
                >(field_0, &child_path(path, "residual_available"))?,
            ),
            output_smoothness: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeOrder,
                >(field_1, &child_path(path, "output_smoothness"))?,
            ),
            selector_neighborhood: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeOrder,
                >(field_2, &child_path(path, "selector_neighborhood"))?,
            ),
            inner_minimum: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeOrder,
                >(field_3, &child_path(path, "inner_minimum"))?,
            ),
            requested_output: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeOrder,
                >(field_4, &child_path(path, "requested_output"))?,
            ),
            residual_compilation: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeOrder,
                >(field_5, &child_path(path, "residual_compilation"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeRequirements,
> for pse_kernels::DerivativeRequirements {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeRequirements,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_kernels::DerivativeRequirements as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_kernels::DerivativeRequirements as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "residual_available"),
                (2, "output_smoothness"),
                (3, "selector_neighborhood"),
                (4, "inner_minimum"),
                (5, "requested_output"),
                (6, "residual_compilation"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeOrder,
            >(
                required(
                    wire.residual_available.into_option(),
                    &child_path(path, "residual_available"),
                )?,
                &child_path(path, "residual_available"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeOrder,
            >(
                required(
                    wire.output_smoothness.into_option(),
                    &child_path(path, "output_smoothness"),
                )?,
                &child_path(path, "output_smoothness"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeOrder,
            >(
                required(
                    wire.selector_neighborhood.into_option(),
                    &child_path(path, "selector_neighborhood"),
                )?,
                &child_path(path, "selector_neighborhood"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeOrder,
            >(
                required(
                    wire.inner_minimum.into_option(),
                    &child_path(path, "inner_minimum"),
                )?,
                &child_path(path, "inner_minimum"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeOrder,
            >(
                required(
                    wire.requested_output.into_option(),
                    &child_path(path, "requested_output"),
                )?,
                &child_path(path, "requested_output"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeOrder,
            >(
                required(
                    wire.residual_compilation.into_option(),
                    &child_path(path, "residual_compilation"),
                )?,
                &child_path(path, "residual_compilation"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::MathApplicabilitySnapshot,
> for pse_math::transfer::ApplicabilitySnapshot {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::MathApplicabilitySnapshot,
        AdapterError,
    > {
        layout(
            <pse_math::transfer::ApplicabilitySnapshot as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::transfer::ApplicabilitySnapshot as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "stages"),
                (2, "frame"),
                (3, "reads"),
                (4, "layout"),
                (5, "predicates"),
                (6, "inputs"),
                (7, "token"),
                (8, "plan"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5, field_6, field_7) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::MathApplicabilitySnapshot {
            stages: {
                let current_path = &child_path(path, "stages");
                items(field_0)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::MathStageSnapshot,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            frame: native_into::<_, u64>(field_1, &child_path(path, "frame"))?,
            reads: {
                let current_path = &child_path(path, "reads");
                items(field_2)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            layout: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::MathJetLayoutSnapshot,
                >(field_3, &child_path(path, "layout"))?,
            ),
            predicates: {
                let current_path = &child_path(path, "predicates");
                items(field_4)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            inputs: {
                let current_path = &child_path(path, "inputs");
                items(field_5)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            token: native_into::<_, u64>(field_6, &child_path(path, "token"))?,
            plan: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityNode,
                >(field_7, &child_path(path, "plan"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::MathApplicabilitySnapshot,
> for pse_math::transfer::ApplicabilitySnapshot {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::MathApplicabilitySnapshot,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_math::transfer::ApplicabilitySnapshot as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::transfer::ApplicabilitySnapshot as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "stages"),
                (2, "frame"),
                (3, "reads"),
                (4, "layout"),
                (5, "predicates"),
                (6, "inputs"),
                (7, "token"),
                (8, "plan"),
            ],
            path,
        )?;
        let fields = (
            {
                let current_path = &child_path(path, "stages");
                let values = wire
                    .stages
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::MathStageSnapshot,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<_, u64>(wire.frame, &child_path(path, "frame"))?,
            {
                let current_path = &child_path(path, "reads");
                let values = wire
                    .reads
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::MathJetLayoutSnapshot,
            >(
                required(wire.layout.into_option(), &child_path(path, "layout"))?,
                &child_path(path, "layout"),
            )?,
            {
                let current_path = &child_path(path, "predicates");
                let values = wire
                    .predicates
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "inputs");
                let values = wire
                    .inputs
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<_, u64>(wire.token, &child_path(path, "token"))?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityNode,
            >(
                required(wire.plan.into_option(), &child_path(path, "plan"))?,
                &child_path(path, "plan"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::MathBlockSnapshot,
> for pse_math::transfer::BlockSnapshot {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::MathBlockSnapshot,
        AdapterError,
    > {
        layout(
            <pse_math::transfer::BlockSnapshot as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::transfer::BlockSnapshot as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "program"),
                (2, "resettable"),
                (3, "components"),
                (4, "inputs"),
                (5, "outputs"),
                (6, "gather"),
                (7, "scatter"),
                (8, "arguments"),
                (9, "values"),
                (10, "source"),
                (11, "checks"),
                (12, "checked_ends"),
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
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::MathBlockSnapshot {
            program: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_0,
            ),
            resettable: native_into::<
                _,
                bool,
            >(field_1, &child_path(path, "resettable"))?,
            components: {
                let current_path = &child_path(path, "components");
                items(field_2)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            inputs: {
                let current_path = &child_path(path, "inputs");
                items(field_3)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            outputs: {
                let current_path = &child_path(path, "outputs");
                items(field_4)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            gather: {
                let current_path = &child_path(path, "gather");
                items(field_5)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::Tuple2OfUsizeAndUsize,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            scatter: {
                let current_path = &child_path(path, "scatter");
                items(field_6)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::Tuple2OfUsizeAndOptionalUsize,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            arguments: native_into::<_, u64>(field_7, &child_path(path, "arguments"))?,
            values: native_into::<_, u64>(field_8, &child_path(path, "values"))?,
            source: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_9,
            ),
            checks: {
                let current_path = &child_path(path, "checks");
                items(field_10)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::MathSourceCheckSnapshot,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            checked_ends: {
                let current_path = &child_path(path, "checked_ends");
                items(field_11)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::MathBlockSnapshot,
> for pse_math::transfer::BlockSnapshot {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::MathBlockSnapshot,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_math::transfer::BlockSnapshot as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::transfer::BlockSnapshot as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "program"),
                (2, "resettable"),
                (3, "components"),
                (4, "inputs"),
                (5, "outputs"),
                (6, "gather"),
                (7, "scatter"),
                (8, "arguments"),
                (9, "values"),
                (10, "source"),
                (11, "checks"),
                (12, "checked_ends"),
            ],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.program,
                &child_path(path, "program"),
            )?,
            native_from::<_, bool>(wire.resettable, &child_path(path, "resettable"))?,
            {
                let current_path = &child_path(path, "components");
                let values = wire
                    .components
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "inputs");
                let values = wire
                    .inputs
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "outputs");
                let values = wire
                    .outputs
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "gather");
                let values = wire
                    .gather
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::Tuple2OfUsizeAndUsize,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "scatter");
                let values = wire
                    .scatter
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::Tuple2OfUsizeAndOptionalUsize,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<_, u64>(wire.arguments, &child_path(path, "arguments"))?,
            native_from::<_, u64>(wire.values, &child_path(path, "values"))?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.source,
                &child_path(path, "source"),
            )?,
            {
                let current_path = &child_path(path, "checks");
                let values = wire
                    .checks
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::MathSourceCheckSnapshot,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "checked_ends");
                let values = wire
                    .checked_ends
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            u64,
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
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                            value,
                            &format!("{path}[{index}]", path = current_path),
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
    crate::math::native_root_generated::pb::pse::native_root::v2::MathComparison,
> for pse_math::guarded::Comparison {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::MathComparison,
        AdapterError,
    > {
        let value = self;
        layout(
            <pse_math::guarded::Comparison as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <pse_math::guarded::Comparison as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_math::guarded::Comparison as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::MathComparison {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::math_comparison::Kind::Eq(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::math_comparison::Alternative1 {
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
            <pse_math::guarded::Comparison as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <pse_math::guarded::Comparison as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_math::guarded::Comparison as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::MathComparison {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::math_comparison::Kind::Ne(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::math_comparison::Alternative2 {
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
            <pse_math::guarded::Comparison as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::VERSION,
            <pse_math::guarded::Comparison as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_math::guarded::Comparison as pse_contract::boundary_projection::ProjectionAlternative<
            3,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::MathComparison {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::math_comparison::Kind::Lt(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::math_comparison::Alternative3 {
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
            <pse_math::guarded::Comparison as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::VERSION,
            <pse_math::guarded::Comparison as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_math::guarded::Comparison as pse_contract::boundary_projection::ProjectionAlternative<
            4,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::MathComparison {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::math_comparison::Kind::Le(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::math_comparison::Alternative4 {
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
    crate::math::native_root_generated::pb::pse::native_root::v2::MathComparison,
> for pse_math::guarded::Comparison {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::MathComparison,
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
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::math_comparison::Kind::Eq(
                wire,
            ) => {
                let alternative_path = child_path(path, "Eq");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_math::guarded::Comparison as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_math::guarded::Comparison as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_math::guarded::Comparison as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::math_comparison::Kind::Ne(
                wire,
            ) => {
                let alternative_path = child_path(path, "Ne");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_math::guarded::Comparison as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_math::guarded::Comparison as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_math::guarded::Comparison as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::math_comparison::Kind::Lt(
                wire,
            ) => {
                let alternative_path = child_path(path, "Lt");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_math::guarded::Comparison as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_math::guarded::Comparison as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_math::guarded::Comparison as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::math_comparison::Kind::Le(
                wire,
            ) => {
                let alternative_path = child_path(path, "Le");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_math::guarded::Comparison as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <pse_math::guarded::Comparison as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_math::guarded::Comparison as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::MathCondition,
> for pse_math::guarded::Condition {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::MathCondition,
        AdapterError,
    > {
        let value = self;
        layout(
            <pse_math::guarded::Condition as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <pse_math::guarded::Condition as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_math::guarded::Condition as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::MathCondition {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::math_condition::Kind::Positive(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::math_condition::Alternative1 {
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
            <pse_math::guarded::Condition as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <pse_math::guarded::Condition as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_math::guarded::Condition as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::MathCondition {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::math_condition::Kind::Nonnegative(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::math_condition::Alternative2 {
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
            <pse_math::guarded::Condition as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::VERSION,
            <pse_math::guarded::Condition as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_math::guarded::Condition as pse_contract::boundary_projection::ProjectionAlternative<
            3,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::MathCondition {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::math_condition::Kind::Nonzero(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::math_condition::Alternative3 {
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
    crate::math::native_root_generated::pb::pse::native_root::v2::MathCondition,
> for pse_math::guarded::Condition {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::MathCondition,
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
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::math_condition::Kind::Positive(
                wire,
            ) => {
                let alternative_path = child_path(path, "Positive");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_math::guarded::Condition as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_math::guarded::Condition as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_math::guarded::Condition as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::math_condition::Kind::Nonnegative(
                wire,
            ) => {
                let alternative_path = child_path(path, "Nonnegative");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_math::guarded::Condition as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_math::guarded::Condition as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_math::guarded::Condition as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::math_condition::Kind::Nonzero(
                wire,
            ) => {
                let alternative_path = child_path(path, "Nonzero");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_math::guarded::Condition as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_math::guarded::Condition as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_math::guarded::Condition as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::MathDomainSnapshot,
> for pse_math::transfer::DomainSnapshot {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::MathDomainSnapshot,
        AdapterError,
    > {
        layout(
            <pse_math::transfer::DomainSnapshot as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::transfer::DomainSnapshot as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "stages"),
                (2, "frame"),
                (3, "reads"),
                (4, "layout"),
                (5, "argument"),
                (6, "token"),
                (7, "lineage"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5, field_6) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::MathDomainSnapshot {
            stages: {
                let current_path = &child_path(path, "stages");
                items(field_0)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::MathStageSnapshot,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            frame: native_into::<_, u64>(field_1, &child_path(path, "frame"))?,
            reads: {
                let current_path = &child_path(path, "reads");
                items(field_2)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            layout: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::MathJetLayoutSnapshot,
                >(field_3, &child_path(path, "layout"))?,
            ),
            argument: native_into::<_, u64>(field_4, &child_path(path, "argument"))?,
            token: native_into::<_, u64>(field_5, &child_path(path, "token"))?,
            lineage: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::ModelValidityLineage,
                >(field_6, &child_path(path, "lineage"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::MathDomainSnapshot,
> for pse_math::transfer::DomainSnapshot {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::MathDomainSnapshot,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_math::transfer::DomainSnapshot as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::transfer::DomainSnapshot as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "stages"),
                (2, "frame"),
                (3, "reads"),
                (4, "layout"),
                (5, "argument"),
                (6, "token"),
                (7, "lineage"),
            ],
            path,
        )?;
        let fields = (
            {
                let current_path = &child_path(path, "stages");
                let values = wire
                    .stages
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::MathStageSnapshot,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<_, u64>(wire.frame, &child_path(path, "frame"))?,
            {
                let current_path = &child_path(path, "reads");
                let values = wire
                    .reads
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::MathJetLayoutSnapshot,
            >(
                required(wire.layout.into_option(), &child_path(path, "layout"))?,
                &child_path(path, "layout"),
            )?,
            native_from::<_, u64>(wire.argument, &child_path(path, "argument"))?,
            native_from::<_, u64>(wire.token, &child_path(path, "token"))?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::ModelValidityLineage,
            >(
                required(wire.lineage.into_option(), &child_path(path, "lineage"))?,
                &child_path(path, "lineage"),
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
    crate::math::native_root_generated::pb::pse::native_root::v2::MathExecutionAttributionSnapshot,
> for pse_math::ExecutionAttributionSnapshot {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::MathExecutionAttributionSnapshot,
        AdapterError,
    > {
        layout(
            <pse_math::ExecutionAttributionSnapshot as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::ExecutionAttributionSnapshot as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
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
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::MathExecutionAttributionSnapshot {
            instance: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_0,
            ),
            checked_members: field_1
                .into_iter()
                .map(|(key, value)| -> Result<_, AdapterError> {
                    Ok(crate::math::native_root_generated::pb::pse::native_root::v2::math_execution_attribution_snapshot::Field2Entry {
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
                    Ok(crate::math::native_root_generated::pb::pse::native_root::v2::math_execution_attribution_snapshot::Field3Entry {
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
                    Ok(crate::math::native_root_generated::pb::pse::native_root::v2::math_execution_attribution_snapshot::Field4Entry {
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
    crate::math::native_root_generated::pb::pse::native_root::v2::MathExecutionAttributionSnapshot,
> for pse_math::ExecutionAttributionSnapshot {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::MathExecutionAttributionSnapshot,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_math::ExecutionAttributionSnapshot as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::ExecutionAttributionSnapshot as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::MathExecutionSnapshot,
> for pse_math::transfer::ExecutionSnapshot {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::MathExecutionSnapshot,
        AdapterError,
    > {
        layout(
            <pse_math::transfer::ExecutionSnapshot as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::transfer::ExecutionSnapshot as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "version"),
                (2, "inputs"),
                (3, "input_formals"),
                (4, "coordinate_offsets"),
                (5, "slots"),
                (6, "outputs"),
                (7, "limits"),
                (8, "value_layout"),
                (9, "first_layout"),
                (10, "first_rows"),
                (11, "first_axes"),
                (12, "value"),
                (13, "first"),
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
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::MathExecutionSnapshot {
            version: native_into::<_, u32>(field_0, &child_path(path, "version"))?,
            inputs: native_into::<_, u64>(field_1, &child_path(path, "inputs"))?,
            input_formals: {
                let current_path = &child_path(path, "input_formals");
                items(field_2)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            coordinate_offsets: {
                let current_path = &child_path(path, "coordinate_offsets");
                items(field_3)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            slots: native_into::<_, u64>(field_4, &child_path(path, "slots"))?,
            outputs: {
                let current_path = &child_path(path, "outputs");
                items(field_5)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            limits: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::MathEvaluationLimits,
                >(field_6, &child_path(path, "limits"))?,
            ),
            value_layout: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::MathJetLayoutSnapshot,
                >(field_7, &child_path(path, "value_layout"))?,
            ),
            first_layout: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::MathJetLayoutSnapshot,
                >(field_8, &child_path(path, "first_layout"))?,
            ),
            first_rows: {
                let current_path = &child_path(path, "first_rows");
                items(field_9)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::Tuple2OfUsizeAndUsize,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            first_axes: {
                let current_path = &child_path(path, "first_axes");
                items(field_10)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            value: {
                let current_path = &child_path(path, "value");
                items(field_11)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::MathStageSnapshot,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            first: {
                let current_path = &child_path(path, "first");
                items(field_12)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::MathStageSnapshot,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::MathExecutionSnapshot,
> for pse_math::transfer::ExecutionSnapshot {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::MathExecutionSnapshot,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_math::transfer::ExecutionSnapshot as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::transfer::ExecutionSnapshot as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "version"),
                (2, "inputs"),
                (3, "input_formals"),
                (4, "coordinate_offsets"),
                (5, "slots"),
                (6, "outputs"),
                (7, "limits"),
                (8, "value_layout"),
                (9, "first_layout"),
                (10, "first_rows"),
                (11, "first_axes"),
                (12, "value"),
                (13, "first"),
            ],
            path,
        )?;
        let fields = (
            native_from::<_, u32>(wire.version, &child_path(path, "version"))?,
            native_from::<_, u64>(wire.inputs, &child_path(path, "inputs"))?,
            {
                let current_path = &child_path(path, "input_formals");
                let values = wire
                    .input_formals
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "coordinate_offsets");
                let values = wire
                    .coordinate_offsets
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<_, u64>(wire.slots, &child_path(path, "slots"))?,
            {
                let current_path = &child_path(path, "outputs");
                let values = wire
                    .outputs
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::MathEvaluationLimits,
            >(
                required(wire.limits.into_option(), &child_path(path, "limits"))?,
                &child_path(path, "limits"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::MathJetLayoutSnapshot,
            >(
                required(
                    wire.value_layout.into_option(),
                    &child_path(path, "value_layout"),
                )?,
                &child_path(path, "value_layout"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::MathJetLayoutSnapshot,
            >(
                required(
                    wire.first_layout.into_option(),
                    &child_path(path, "first_layout"),
                )?,
                &child_path(path, "first_layout"),
            )?,
            {
                let current_path = &child_path(path, "first_rows");
                let values = wire
                    .first_rows
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::Tuple2OfUsizeAndUsize,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "first_axes");
                let values = wire
                    .first_axes
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "value");
                let values = wire
                    .value
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::MathStageSnapshot,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "first");
                let values = wire
                    .first
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::MathStageSnapshot,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::MathJetLayoutSnapshot,
> for pse_math::jets::JetLayoutSnapshot {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::MathJetLayoutSnapshot,
        AdapterError,
    > {
        layout(
            <pse_math::jets::JetLayoutSnapshot as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::jets::JetLayoutSnapshot as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "coordinates"),
                (2, "shape"),
                (3, "pairs"),
                (4, "order"),
                (5, "slot_offsets"),
                (6, "slot_components"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::MathJetLayoutSnapshot {
            coordinates: {
                let current_path = &child_path(path, "coordinates");
                items(field_0)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            shape: {
                let current_path = &child_path(path, "shape");
                items(field_1)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::math_jet_layout_snapshot::Field2Value {
                            value: {
                                let current_path = &format!(
                                    "{path}[{index}]",
                                    path = current_path,
                                );
                                items(value)
                                    .enumerate()
                                    .map(|(index, value)| -> Result<_, AdapterError> {
                                        native_into::<
                                            _,
                                            u64,
                                        >(value, &format!("{path}[{index}]", path = current_path))
                                    })
                                    .collect::<Result<Vec<_>, _>>()?
                            },
                            ..Default::default()
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            pairs: {
                let current_path = &child_path(path, "pairs");
                items(field_2)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::Tuple2OfUsizeAndUsize,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            order: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeOrder,
                >(field_3, &child_path(path, "order"))?,
            ),
            slot_offsets: field_4
                .map(|value| -> Result<_, AdapterError> {
                    Ok(crate::math::native_root_generated::pb::pse::native_root::v2::math_jet_layout_snapshot::Field5Value {
                        value: {
                            let current_path = &child_path(path, "slot_offsets");
                            items(value)
                                .enumerate()
                                .map(|(index, value)| -> Result<_, AdapterError> {
                                    native_into::<
                                        _,
                                        u64,
                                    >(value, &format!("{path}[{index}]", path = current_path))
                                })
                                .collect::<Result<Vec<_>, _>>()?
                        },
                        ..Default::default()
                    })
                })
                .transpose()?
                .into(),
            slot_components: field_5
                .map(|value| -> Result<_, AdapterError> {
                    Ok(crate::math::native_root_generated::pb::pse::native_root::v2::math_jet_layout_snapshot::Field6Value {
                        value: {
                            let current_path = &child_path(path, "slot_components");
                            items(value)
                                .enumerate()
                                .map(|(index, value)| -> Result<_, AdapterError> {
                                    native_into::<
                                        _,
                                        u64,
                                    >(value, &format!("{path}[{index}]", path = current_path))
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
    crate::math::native_root_generated::pb::pse::native_root::v2::MathJetLayoutSnapshot,
> for pse_math::jets::JetLayoutSnapshot {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::MathJetLayoutSnapshot,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_math::jets::JetLayoutSnapshot as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::jets::JetLayoutSnapshot as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "coordinates"),
                (2, "shape"),
                (3, "pairs"),
                (4, "order"),
                (5, "slot_offsets"),
                (6, "slot_components"),
            ],
            path,
        )?;
        let fields = (
            {
                let current_path = &child_path(path, "coordinates");
                let values = wire
                    .coordinates
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "shape");
                let values = wire
                    .shape
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        {
                            let wrapped = value;
                            unknown_fields(
                                &wrapped.__buffa_unknown_fields,
                                &format!("{path}[{index}]", path = current_path),
                            )?;
                            {
                                let current_path = &format!(
                                    "{path}[{index}]",
                                    path = current_path,
                                );
                                let values = wrapped
                                    .value
                                    .into_iter()
                                    .enumerate()
                                    .map(|(index, value)| -> Result<_, AdapterError> {
                                        native_from::<
                                            _,
                                            u64,
                                        >(value, &format!("{path}[{index}]", path = current_path))
                                    })
                                    .collect::<Result<Vec<_>, _>>()?;
                                from_items(values, current_path)
                            }
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "pairs");
                let values = wire
                    .pairs
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::Tuple2OfUsizeAndUsize,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeOrder,
            >(
                required(wire.order.into_option(), &child_path(path, "order"))?,
                &child_path(path, "order"),
            )?,
            wire
                .slot_offsets
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    {
                        let wrapped = value;
                        unknown_fields(
                            &wrapped.__buffa_unknown_fields,
                            &child_path(path, "slot_offsets"),
                        )?;
                        {
                            let current_path = &child_path(path, "slot_offsets");
                            let values = wrapped
                                .value
                                .into_iter()
                                .enumerate()
                                .map(|(index, value)| -> Result<_, AdapterError> {
                                    native_from::<
                                        _,
                                        u64,
                                    >(value, &format!("{path}[{index}]", path = current_path))
                                })
                                .collect::<Result<Vec<_>, _>>()?;
                            from_items(values, current_path)
                        }
                    }
                })
                .transpose()?,
            wire
                .slot_components
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    {
                        let wrapped = value;
                        unknown_fields(
                            &wrapped.__buffa_unknown_fields,
                            &child_path(path, "slot_components"),
                        )?;
                        {
                            let current_path = &child_path(path, "slot_components");
                            let values = wrapped
                                .value
                                .into_iter()
                                .enumerate()
                                .map(|(index, value)| -> Result<_, AdapterError> {
                                    native_from::<
                                        _,
                                        u64,
                                    >(value, &format!("{path}[{index}]", path = current_path))
                                })
                                .collect::<Result<Vec<_>, _>>()?;
                            from_items(values, current_path)
                        }
                    }
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
    crate::math::native_root_generated::pb::pse::native_root::v2::MathRootDefinitionSnapshot,
> for pse_math::implicit::transfer::RootDefinitionSnapshot {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::MathRootDefinitionSnapshot,
        AdapterError,
    > {
        layout(
            <pse_math::implicit::transfer::RootDefinitionSnapshot as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::implicit::transfer::RootDefinitionSnapshot as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "version"),
                (2, "id"),
                (3, "identity"),
                (4, "unknowns"),
                (5, "rows"),
                (6, "inputs"),
                (7, "residual_offsets"),
                (8, "requirements"),
                (9, "native_order"),
                (10, "col_ptr"),
                (11, "row_idx"),
                (12, "attribution_commitment"),
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
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::MathRootDefinitionSnapshot {
            version: native_into::<_, u32>(field_0, &child_path(path, "version"))?,
            id: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_1,
            ),
            identity: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_2,
            ),
            unknowns: {
                let current_path = &child_path(path, "unknowns");
                items(field_3)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ImplicitUnknown,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            rows: items(field_4)
                .map(
                    pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes,
                )
                .collect::<Vec<_>>(),
            inputs: native_into::<_, u64>(field_5, &child_path(path, "inputs"))?,
            residual_offsets: {
                let current_path = &child_path(path, "residual_offsets");
                items(field_6)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            requirements: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeRequirements,
                >(field_7, &child_path(path, "requirements"))?,
            ),
            native_order: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeOrder,
                >(field_8, &child_path(path, "native_order"))?,
            ),
            col_ptr: {
                let current_path = &child_path(path, "col_ptr");
                items(field_9)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            row_idx: {
                let current_path = &child_path(path, "row_idx");
                items(field_10)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            attribution_commitment: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_11,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::MathRootDefinitionSnapshot,
> for pse_math::implicit::transfer::RootDefinitionSnapshot {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::MathRootDefinitionSnapshot,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_math::implicit::transfer::RootDefinitionSnapshot as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::implicit::transfer::RootDefinitionSnapshot as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "version"),
                (2, "id"),
                (3, "identity"),
                (4, "unknowns"),
                (5, "rows"),
                (6, "inputs"),
                (7, "residual_offsets"),
                (8, "requirements"),
                (9, "native_order"),
                (10, "col_ptr"),
                (11, "row_idx"),
                (12, "attribution_commitment"),
            ],
            path,
        )?;
        let fields = (
            native_from::<_, u32>(wire.version, &child_path(path, "version"))?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.id,
                &child_path(path, "id"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.identity,
                &child_path(path, "identity"),
            )?,
            {
                let current_path = &child_path(path, "unknowns");
                let values = wire
                    .unknowns
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ImplicitUnknown,
                        >(value, &format!("{path}[{index}]", path = current_path))
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
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                            value,
                            &format!("{path}[{index}]", path = current_path),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<_, u64>(wire.inputs, &child_path(path, "inputs"))?,
            {
                let current_path = &child_path(path, "residual_offsets");
                let values = wire
                    .residual_offsets
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeRequirements,
            >(
                required(
                    wire.requirements.into_option(),
                    &child_path(path, "requirements"),
                )?,
                &child_path(path, "requirements"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeOrder,
            >(
                required(
                    wire.native_order.into_option(),
                    &child_path(path, "native_order"),
                )?,
                &child_path(path, "native_order"),
            )?,
            {
                let current_path = &child_path(path, "col_ptr");
                let values = wire
                    .col_ptr
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "row_idx");
                let values = wire
                    .row_idx
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.attribution_commitment,
                &child_path(path, "attribution_commitment"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::MathRootExecutionSnapshot,
> for pse_math::implicit::transfer::RootExecutionSnapshot {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::MathRootExecutionSnapshot,
        AdapterError,
    > {
        layout(
            <pse_math::implicit::transfer::RootExecutionSnapshot as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::implicit::transfer::RootExecutionSnapshot as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "definition"), (2, "attribution"), (3, "numerical")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::MathRootExecutionSnapshot {
            definition: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::MathRootDefinitionSnapshot,
                >(field_0, &child_path(path, "definition"))?,
            ),
            attribution: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::MathExecutionAttributionSnapshot,
                >(field_1, &child_path(path, "attribution"))?,
            ),
            numerical: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::MathExecutionSnapshot,
                >(field_2, &child_path(path, "numerical"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::MathRootExecutionSnapshot,
> for pse_math::implicit::transfer::RootExecutionSnapshot {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::MathRootExecutionSnapshot,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_math::implicit::transfer::RootExecutionSnapshot as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::implicit::transfer::RootExecutionSnapshot as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "definition"), (2, "attribution"), (3, "numerical")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::MathRootDefinitionSnapshot,
            >(
                required(
                    wire.definition.into_option(),
                    &child_path(path, "definition"),
                )?,
                &child_path(path, "definition"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::MathExecutionAttributionSnapshot,
            >(
                required(
                    wire.attribution.into_option(),
                    &child_path(path, "attribution"),
                )?,
                &child_path(path, "attribution"),
            )?,
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::MathExecutionSnapshot,
            >(
                required(wire.numerical.into_option(), &child_path(path, "numerical"))?,
                &child_path(path, "numerical"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::MathSourceCheckSnapshot,
> for pse_math::transfer::CheckSnapshot {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::MathSourceCheckSnapshot,
        AdapterError,
    > {
        layout(
            <pse_math::transfer::CheckSnapshot as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::transfer::CheckSnapshot as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "end_output"), (2, "source")],
            path,
        )?;
        let (field_0, field_1) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::MathSourceCheckSnapshot {
            end_output: native_into::<_, u64>(field_0, &child_path(path, "end_output"))?,
            source: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_1,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::MathSourceCheckSnapshot,
> for pse_math::transfer::CheckSnapshot {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::MathSourceCheckSnapshot,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_math::transfer::CheckSnapshot as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::transfer::CheckSnapshot as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "end_output"), (2, "source")],
            path,
        )?;
        let fields = (
            native_from::<_, u64>(wire.end_output, &child_path(path, "end_output"))?,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::MathStageSnapshot,
> for pse_math::transfer::StageSnapshot {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::MathStageSnapshot,
        AdapterError,
    > {
        let value = self;
        layout(
            <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Block");
                let path = alternative_path.as_str();
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::MathStageSnapshot {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::math_stage_snapshot::Kind::Block(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::math_stage_snapshot::Alternative1 {
                                value: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::math::native_root_generated::pb::pse::native_root::v2::MathBlockSnapshot,
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
            <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[
                (1, "argument"),
                (2, "condition"),
                (3, "order"),
                (4, "source"),
                (5, "lineage"),
            ],
            path,
        )?;
        let value = match <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1, field_2, field_3, field_4)) => {
                let alternative_path = child_path(path, "Require");
                let path = alternative_path.as_str();
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::MathStageSnapshot {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::math_stage_snapshot::Kind::Require(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::math_stage_snapshot::Alternative2 {
                                argument: native_into::<
                                    _,
                                    u64,
                                >(field_0, &child_path(path, "argument"))?,
                                condition: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::math::native_root_generated::pb::pse::native_root::v2::MathCondition,
                                    >(field_1, &child_path(path, "condition"))?,
                                ),
                                order: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeOrder,
                                    >(field_2, &child_path(path, "order"))?,
                                ),
                                source: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                                    field_3,
                                ),
                                lineage: field_4
                                    .map(|value| -> Result<_, AdapterError> {
                                        native_into::<
                                            _,
                                            crate::math::native_root_generated::pb::pse::native_root::v2::ModelValidityLineage,
                                        >(value, &child_path(path, "lineage"))
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
            <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::VERSION,
            <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
            3,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Domain");
                let path = alternative_path.as_str();
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::MathStageSnapshot {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::math_stage_snapshot::Kind::Domain(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::math_stage_snapshot::Alternative3 {
                                value: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::math::native_root_generated::pb::pse::native_root::v2::MathDomainSnapshot,
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
            <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::VERSION,
            <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
            4,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Applicability");
                let path = alternative_path.as_str();
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::MathStageSnapshot {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::math_stage_snapshot::Kind::Applicability(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::math_stage_snapshot::Alternative4 {
                                value: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::math::native_root_generated::pb::pse::native_root::v2::MathApplicabilitySnapshot,
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
            <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
                5,
            >>::VERSION,
            <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
                5,
            >>::FIELD_LAYOUT,
            1,
            &[
                (1, "require_separation"),
                (2, "comparison"),
                (3, "left"),
                (4, "right"),
                (5, "then"),
                (6, "otherwise"),
            ],
            path,
        )?;
        let value = match <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
            5,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1, field_2, field_3, field_4, field_5)) => {
                let alternative_path = child_path(path, "Branch");
                let path = alternative_path.as_str();
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::MathStageSnapshot {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::math_stage_snapshot::Kind::Branch(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::math_stage_snapshot::Alternative5 {
                                require_separation: native_into::<
                                    _,
                                    bool,
                                >(field_0, &child_path(path, "require_separation"))?,
                                comparison: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::math::native_root_generated::pb::pse::native_root::v2::MathComparison,
                                    >(field_1, &child_path(path, "comparison"))?,
                                ),
                                left: native_into::<
                                    _,
                                    u64,
                                >(field_2, &child_path(path, "left"))?,
                                right: native_into::<
                                    _,
                                    u64,
                                >(field_3, &child_path(path, "right"))?,
                                then: {
                                    let current_path = &child_path(path, "then");
                                    items(field_4)
                                        .enumerate()
                                        .map(|(index, value)| -> Result<_, AdapterError> {
                                            native_into::<
                                                _,
                                                crate::math::native_root_generated::pb::pse::native_root::v2::MathStageSnapshot,
                                            >(value, &format!("{path}[{index}]", path = current_path))
                                        })
                                        .collect::<Result<Vec<_>, _>>()?
                                },
                                otherwise: {
                                    let current_path = &child_path(path, "otherwise");
                                    items(field_5)
                                        .enumerate()
                                        .map(|(index, value)| -> Result<_, AdapterError> {
                                            native_into::<
                                                _,
                                                crate::math::native_root_generated::pb::pse::native_root::v2::MathStageSnapshot,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::MathStageSnapshot,
> for pse_math::transfer::StageSnapshot {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::MathStageSnapshot,
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
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::math_stage_snapshot::Kind::Block(
                wire,
            ) => {
                let alternative_path = child_path(path, "Block");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::math::native_root_generated::pb::pse::native_root::v2::MathBlockSnapshot,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::math_stage_snapshot::Kind::Require(
                wire,
            ) => {
                let alternative_path = child_path(path, "Require");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[
                        (1, "argument"),
                        (2, "condition"),
                        (3, "order"),
                        (4, "source"),
                        (5, "lineage"),
                    ],
                    path,
                )?;
                let fields = (
                    native_from::<_, u64>(wire.argument, &child_path(path, "argument"))?,
                    native_from::<
                        _,
                        crate::math::native_root_generated::pb::pse::native_root::v2::MathCondition,
                    >(
                        required(
                            wire.condition.into_option(),
                            &child_path(path, "condition"),
                        )?,
                        &child_path(path, "condition"),
                    )?,
                    native_from::<
                        _,
                        crate::math::native_root_generated::pb::pse::native_root::v2::KernelDerivativeOrder,
                    >(
                        required(wire.order.into_option(), &child_path(path, "order"))?,
                        &child_path(path, "order"),
                    )?,
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                        wire.source,
                        &child_path(path, "source"),
                    )?,
                    wire
                        .lineage
                        .into_option()
                        .map(|value| -> Result<_, AdapterError> {
                            native_from::<
                                _,
                                crate::math::native_root_generated::pb::pse::native_root::v2::ModelValidityLineage,
                            >(value, &child_path(path, "lineage"))
                        })
                        .transpose()?,
                );
                Ok(
                    <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::math_stage_snapshot::Kind::Domain(
                wire,
            ) => {
                let alternative_path = child_path(path, "Domain");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::math::native_root_generated::pb::pse::native_root::v2::MathDomainSnapshot,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::math_stage_snapshot::Kind::Applicability(
                wire,
            ) => {
                let alternative_path = child_path(path, "Applicability");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::math::native_root_generated::pb::pse::native_root::v2::MathApplicabilitySnapshot,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::math_stage_snapshot::Kind::Branch(
                wire,
            ) => {
                let alternative_path = child_path(path, "Branch");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::VERSION,
                    <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::FIELD_LAYOUT,
                    1,
                    &[
                        (1, "require_separation"),
                        (2, "comparison"),
                        (3, "left"),
                        (4, "right"),
                        (5, "then"),
                        (6, "otherwise"),
                    ],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        bool,
                    >(wire.require_separation, &child_path(path, "require_separation"))?,
                    native_from::<
                        _,
                        crate::math::native_root_generated::pb::pse::native_root::v2::MathComparison,
                    >(
                        required(
                            wire.comparison.into_option(),
                            &child_path(path, "comparison"),
                        )?,
                        &child_path(path, "comparison"),
                    )?,
                    native_from::<_, u64>(wire.left, &child_path(path, "left"))?,
                    native_from::<_, u64>(wire.right, &child_path(path, "right"))?,
                    {
                        let current_path = &child_path(path, "then");
                        let values = wire
                            .then
                            .into_iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                native_from::<
                                    _,
                                    crate::math::native_root_generated::pb::pse::native_root::v2::MathStageSnapshot,
                                >(value, &format!("{path}[{index}]", path = current_path))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                    {
                        let current_path = &child_path(path, "otherwise");
                        let values = wire
                            .otherwise
                            .into_iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                native_from::<
                                    _,
                                    crate::math::native_root_generated::pb::pse::native_root::v2::MathStageSnapshot,
                                >(value, &format!("{path}[{index}]", path = current_path))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_math::transfer::StageSnapshot as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
        }
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
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ModelBoundaryDiagnostic,
                        >(value, &format!("{path}[{index}]", path = current_path))
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
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ModelSourceLocation,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            validity: field_9
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<
                        _,
                        crate::math::native_root_generated::pb::pse::native_root::v2::ModelValidityLineage,
                    >(value, &child_path(path, "validity"))
                })
                .transpose()?
                .into(),
            applicability: {
                let current_path = &child_path(path, "applicability");
                items(field_10)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityObservation,
                        >(value, &format!("{path}[{index}]", path = current_path))
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
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ModelBoundaryDiagnostic,
                        >(value, &format!("{path}[{index}]", path = current_path))
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
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                            value,
                            &format!("{path}[{index}]", path = current_path),
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
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ModelSourceLocation,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            wire
                .validity
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        crate::math::native_root_generated::pb::pse::native_root::v2::ModelValidityLineage,
                    >(value, &child_path(path, "validity"))
                })
                .transpose()?,
            {
                let current_path = &child_path(path, "applicability");
                let values = wire
                    .applicability
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            crate::math::native_root_generated::pb::pse::native_root::v2::ApplicabilityObservation,
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
                    native_into::<_, String>(value, &child_path(path, "occurrence_role"))
                })
                .transpose()?,
            occurrence_position: field_4
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<
                        _,
                        u32,
                    >(value, &child_path(path, "occurrence_position"))
                })
                .transpose()?,
            path: native_into::<_, String>(field_5, &child_path(path, "path"))?,
            name: field_6
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<_, String>(value, &child_path(path, "name"))
                })
                .transpose()?,
            start: field_7
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<_, u32>(value, &child_path(path, "start"))
                })
                .transpose()?,
            end: field_8
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<_, u32>(value, &child_path(path, "end"))
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
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                        value,
                        &child_path(path, "revision"),
                    )
                })
                .transpose()?,
            wire
                .document
                .map(|value| -> Result<_, AdapterError> {
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                        value,
                        &child_path(path, "document"),
                    )
                })
                .transpose()?,
            wire
                .occurrence_role
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<_, String>(value, &child_path(path, "occurrence_role"))
                })
                .transpose()?,
            wire
                .occurrence_position
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        u32,
                    >(value, &child_path(path, "occurrence_position"))
                })
                .transpose()?,
            native_from::<_, String>(wire.path, &child_path(path, "path"))?,
            wire
                .name
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<_, String>(value, &child_path(path, "name"))
                })
                .transpose()?,
            wire
                .start
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<_, u32>(value, &child_path(path, "start"))
                })
                .transpose()?,
            wire
                .end
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<_, u32>(value, &child_path(path, "end"))
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
                        native_into::<
                            _,
                            u32,
                        >(value, &format!("{path}[{index}]", path = current_path))
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
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                        value,
                        &child_path(path, "form"),
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
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                            value,
                            &format!("{path}[{index}]", path = current_path),
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
                        native_from::<
                            _,
                            u32,
                        >(value, &format!("{path}[{index}]", path = current_path))
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
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                            value,
                            &format!("{path}[{index}]", path = current_path),
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
            4,
            &[(2, "spec"), (8, "parameters"), (9, "options"), (13, "execution")],
            path,
        )?;
        let (field_0, field_1, field_2, field_3) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootInput {
            spec: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::MathBodySpec,
                >(field_0, &child_path(path, "spec"))?,
            ),
            parameters: {
                let current_path = &child_path(path, "parameters");
                items(field_1)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            options: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::ImplicitOptions,
                >(field_2, &child_path(path, "options"))?,
            ),
            execution: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::math::native_root_generated::pb::pse::native_root::v2::MathRootExecutionSnapshot,
                >(field_3, &child_path(path, "execution"))?,
            ),
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
            4,
            &[(2, "spec"), (8, "parameters"), (9, "options"), (13, "execution")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::math::native_root_generated::pb::pse::native_root::v2::MathBodySpec,
            >(
                required(wire.spec.into_option(), &child_path(path, "spec"))?,
                &child_path(path, "spec"),
            )?,
            {
                let current_path = &child_path(path, "parameters");
                let values = wire
                    .parameters
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            u64,
                        >(value, &format!("{path}[{index}]", path = current_path))
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
                crate::math::native_root_generated::pb::pse::native_root::v2::MathRootExecutionSnapshot,
            >(
                required(wire.execution.into_option(), &child_path(path, "execution"))?,
                &child_path(path, "execution"),
            )?,
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
                                            native_into::<
                                                _,
                                                u64,
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
                                native_from::<
                                    _,
                                    u64,
                                >(value, &format!("{path}[{index}]", path = current_path))
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
                    native_into::<
                        _,
                        crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticOperationId,
                    >(value, &child_path(path, "diagnostic_parent"))
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
                    native_from::<
                        _,
                        crate::math::native_root_generated::pb::pse::native_root::v2::DiagnosticOperationId,
                    >(value, &child_path(path, "diagnostic_parent"))
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
    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRestoreGrant,
> for crate::math::native_root_protocol::RestoreGrant {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRestoreGrant,
        AdapterError,
    > {
        layout(
            <crate::math::native_root_protocol::RestoreGrant as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::math::native_root_protocol::RestoreGrant as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "attempt"), (2, "execution"), (3, "request"), (4, "admitted_bytes")],
            path,
        )?;
        let (field_0, field_1, field_2, field_3) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRestoreGrant {
            attempt: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_0,
            ),
            execution: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_1,
            ),
            request: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_2,
            ),
            admitted_bytes: native_into::<
                _,
                u64,
            >(field_3, &child_path(path, "admitted_bytes"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRestoreGrant,
> for crate::math::native_root_protocol::RestoreGrant {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRestoreGrant,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::math::native_root_protocol::RestoreGrant as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::math::native_root_protocol::RestoreGrant as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "attempt"), (2, "execution"), (3, "request"), (4, "admitted_bytes")],
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
                u64,
            >(wire.admitted_bytes, &child_path(path, "admitted_bytes"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRestorePreparation,
> for crate::math::native_root_protocol::RestorePreparation {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRestorePreparation,
        AdapterError,
    > {
        layout(
            <crate::math::native_root_protocol::RestorePreparation as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::math::native_root_protocol::RestorePreparation as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "attempt"), (2, "execution"), (3, "request"), (4, "result")],
            path,
        )?;
        let (field_0, field_1, field_2, field_3) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRestorePreparation {
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
                    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRestorePreparationOutcome,
                >(field_3, &child_path(path, "result"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRestorePreparation,
> for crate::math::native_root_protocol::RestorePreparation {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRestorePreparation,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::math::native_root_protocol::RestorePreparation as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::math::native_root_protocol::RestorePreparation as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
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
                crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRestorePreparationOutcome,
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
    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRestorePreparationOutcome,
> for crate::math::native_root_protocol::RestorePreparationOutcome {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRestorePreparationOutcome,
        AdapterError,
    > {
        let value = self;
        layout(
            <crate::math::native_root_protocol::RestorePreparationOutcome as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <crate::math::native_root_protocol::RestorePreparationOutcome as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::math::native_root_protocol::RestorePreparationOutcome as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Demand");
                let path = alternative_path.as_str();
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRestorePreparationOutcome {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::native_root_restore_preparation_outcome::Kind::Demand(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::native_root_restore_preparation_outcome::Alternative1 {
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
            <crate::math::native_root_protocol::RestorePreparationOutcome as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <crate::math::native_root_protocol::RestorePreparationOutcome as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::math::native_root_protocol::RestorePreparationOutcome as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Refused");
                let path = alternative_path.as_str();
                return Ok(crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRestorePreparationOutcome {
                    kind: Some(
                        crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::native_root_restore_preparation_outcome::Kind::Refused(
                            Box::new(crate::math::native_root_generated::pb::pse::native_root::v2::native_root_restore_preparation_outcome::Alternative2 {
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
    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRestorePreparationOutcome,
> for crate::math::native_root_protocol::RestorePreparationOutcome {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRestorePreparationOutcome,
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
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::native_root_restore_preparation_outcome::Kind::Demand(
                wire,
            ) => {
                let alternative_path = child_path(path, "Demand");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::math::native_root_protocol::RestorePreparationOutcome as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <crate::math::native_root_protocol::RestorePreparationOutcome as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<_, u64>(wire.value, &child_path(path, "value"))?,
                );
                Ok(
                    <crate::math::native_root_protocol::RestorePreparationOutcome as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::math::native_root_generated::pb::pse::native_root::v2::__buffa::oneof::native_root_restore_preparation_outcome::Kind::Refused(
                wire,
            ) => {
                let alternative_path = child_path(path, "Refused");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::math::native_root_protocol::RestorePreparationOutcome as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <crate::math::native_root_protocol::RestorePreparationOutcome as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
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
                    <crate::math::native_root_protocol::RestorePreparationOutcome as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
        }
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
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::Tuple2OfUsizeAndOptionalUsize,
> for (usize, Option<usize>) {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::Tuple2OfUsizeAndOptionalUsize,
        AdapterError,
    > {
        layout(
            <(
                usize,
                Option<usize>,
            ) as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <(
                usize,
                Option<usize>,
            ) as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "first"), (2, "second")],
            path,
        )?;
        let (field_0, field_1) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::Tuple2OfUsizeAndOptionalUsize {
            first: native_into::<_, u64>(field_0, &child_path(path, "first"))?,
            second: field_1
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<_, u64>(value, &child_path(path, "second"))
                })
                .transpose()?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::Tuple2OfUsizeAndOptionalUsize,
> for (usize, Option<usize>) {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::Tuple2OfUsizeAndOptionalUsize,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <(
                usize,
                Option<usize>,
            ) as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <(
                usize,
                Option<usize>,
            ) as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "first"), (2, "second")],
            path,
        )?;
        let fields = (
            native_from::<_, u64>(wire.first, &child_path(path, "first"))?,
            wire
                .second
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<_, u64>(value, &child_path(path, "second"))
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
    crate::math::native_root_generated::pb::pse::native_root::v2::Tuple2OfUsizeAndUsize,
> for (usize, usize) {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::Tuple2OfUsizeAndUsize,
        AdapterError,
    > {
        layout(
            <(
                usize,
                usize,
            ) as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <(
                usize,
                usize,
            ) as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "first"), (2, "second")],
            path,
        )?;
        let (field_0, field_1) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::Tuple2OfUsizeAndUsize {
            first: native_into::<_, u64>(field_0, &child_path(path, "first"))?,
            second: native_into::<_, u64>(field_1, &child_path(path, "second"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::Tuple2OfUsizeAndUsize,
> for (usize, usize) {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::Tuple2OfUsizeAndUsize,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <(
                usize,
                usize,
            ) as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <(
                usize,
                usize,
            ) as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "first"), (2, "second")],
            path,
        )?;
        let fields = (
            native_from::<_, u64>(wire.first, &child_path(path, "first"))?,
            native_from::<_, u64>(wire.second, &child_path(path, "second"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::math::native_root_generated::pb::pse::native_root::v2::Tuple3OfStringAndUsizeAndSemanticId,
> for (String, usize, pse_ids::SemanticId) {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::math::native_root_generated::pb::pse::native_root::v2::Tuple3OfStringAndUsizeAndSemanticId,
        AdapterError,
    > {
        layout(
            <(
                String,
                usize,
                pse_ids::SemanticId,
            ) as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <(
                String,
                usize,
                pse_ids::SemanticId,
            ) as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "first"), (2, "second"), (3, "third")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::math::native_root_generated::pb::pse::native_root::v2::Tuple3OfStringAndUsizeAndSemanticId {
            first: native_into::<_, String>(field_0, &child_path(path, "first"))?,
            second: native_into::<_, u64>(field_1, &child_path(path, "second"))?,
            third: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_2,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::math::native_root_generated::pb::pse::native_root::v2::Tuple3OfStringAndUsizeAndSemanticId,
> for (String, usize, pse_ids::SemanticId) {
    fn from_wire(
        wire: crate::math::native_root_generated::pb::pse::native_root::v2::Tuple3OfStringAndUsizeAndSemanticId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <(
                String,
                usize,
                pse_ids::SemanticId,
            ) as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <(
                String,
                usize,
                pse_ids::SemanticId,
            ) as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "first"), (2, "second"), (3, "third")],
            path,
        )?;
        let fields = (
            native_from::<_, String>(wire.first, &child_path(path, "first"))?,
            native_from::<_, u64>(wire.second, &child_path(path, "second"))?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.third,
                &child_path(path, "third"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
/// Encode the actual native `MathExecutionSnapshot` and validate outgoing local rules.
pub(crate) fn encode_math_execution_snapshot(
    native: pse_math::transfer::ExecutionSnapshot,
) -> Result<
    crate::math::native_root_generated::pb::pse::native_root::v2::MathExecutionSnapshot,
    BoundaryProjectionError,
> {
    encode_root(native, "MathExecutionSnapshot")
}
/// Adopt known `MathExecutionSnapshot` meaning under this package's declared unknown-field policy.
pub(crate) fn decode_math_execution_snapshot(
    wire: crate::math::native_root_generated::pb::pse::native_root::v2::MathExecutionSnapshot,
) -> Result<pse_math::transfer::ExecutionSnapshot, BoundaryProjectionError> {
    decode_root(wire, "MathExecutionSnapshot")
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
/// Encode the actual native `NativeRootRestorePreparation` and validate outgoing local rules.
pub(crate) fn encode_native_root_restore_preparation(
    native: crate::math::native_root_protocol::RestorePreparation,
) -> Result<
    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRestorePreparation,
    BoundaryProjectionError,
> {
    encode_root(native, "NativeRootRestorePreparation")
}
/// Adopt known `NativeRootRestorePreparation` meaning under this package's declared unknown-field policy.
pub(crate) fn decode_native_root_restore_preparation(
    wire: crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRestorePreparation,
) -> Result<
    crate::math::native_root_protocol::RestorePreparation,
    BoundaryProjectionError,
> {
    decode_root(wire, "NativeRootRestorePreparation")
}
/// Encode the actual native `NativeRootRestoreGrant` and validate outgoing local rules.
pub(crate) fn encode_native_root_restore_grant(
    native: crate::math::native_root_protocol::RestoreGrant,
) -> Result<
    crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRestoreGrant,
    BoundaryProjectionError,
> {
    encode_root(native, "NativeRootRestoreGrant")
}
/// Adopt known `NativeRootRestoreGrant` meaning under this package's declared unknown-field policy.
pub(crate) fn decode_native_root_restore_grant(
    wire: crate::math::native_root_generated::pb::pse::native_root::v2::NativeRootRestoreGrant,
) -> Result<crate::math::native_root_protocol::RestoreGrant, BoundaryProjectionError> {
    decode_root(wire, "NativeRootRestoreGrant")
}
