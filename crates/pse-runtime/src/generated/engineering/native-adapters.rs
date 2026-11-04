use super::{
    AdapterError, AdapterFailure, BoundaryProjectionError, NativeInto, NativeFrom,
    layout, child_path, refused, required, unknown_fields, native_into, native_from,
    encode_root, decode_root, items, from_items, unique_pairs, wire_map,
};
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::Array8PhysicalRatio,
> for [pse_quantity::dimension::Ratio; 8] {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::Array8PhysicalRatio,
        AdapterError,
    > {
        layout(
            <[pse_quantity::dimension::Ratio; 8] as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <[pse_quantity::dimension::Ratio; 8] as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "values")],
            path,
        )?;
        let (field_0,) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::Array8PhysicalRatio {
            values: {
                let current_path = &child_path(path, "values");
                items(field_0)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalRatio,
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
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::Array8PhysicalRatio,
> for [pse_quantity::dimension::Ratio; 8] {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::Array8PhysicalRatio,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <[pse_quantity::dimension::Ratio; 8] as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <[pse_quantity::dimension::Ratio; 8] as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
                                crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalRatio,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::BasisId,
> for pse_quantity::ids::BasisId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::BasisId,
        AdapterError,
    > {
        layout(
            <pse_quantity::ids::BasisId as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::BasisId as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let (field_0,) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::BasisId {
            id: Some(native_into::<_, Vec<u8>>(field_0, &child_path(path, "id"))?),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::BasisId,
> for pse_quantity::ids::BasisId {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::BasisId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::BasisId as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::BasisId as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                Vec<u8>,
            >(required(wire.id, &child_path(path, "id"))?, &child_path(path, "id"))?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::BasisKind,
> for pse_quantity::enums::BasisKind {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::BasisKind,
        AdapterError,
    > {
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::BasisKind {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::BasisKind,
> for pse_quantity::enums::BasisKind {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::BasisKind,
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
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::CanonicalProducerClass,
> for pse_model::generated::enums::CanonicalProducerClass {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::CanonicalProducerClass,
        AdapterError,
    > {
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::CanonicalProducerClass {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::CanonicalProducerClass,
> for pse_model::generated::enums::CanonicalProducerClass {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::CanonicalProducerClass,
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
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::CompositionBasis,
> for pse_quantity::enums::CompositionBasis {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::CompositionBasis,
        AdapterError,
    > {
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::CompositionBasis {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::CompositionBasis,
> for pse_quantity::enums::CompositionBasis {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::CompositionBasis,
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
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::DeclarationId,
> for pse_model::generated::identities::DeclarationId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::DeclarationId,
        AdapterError,
    > {
        layout(
            <pse_model::generated::identities::DeclarationId as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::generated::identities::DeclarationId as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let (field_0,) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::DeclarationId {
            id: Some(native_into::<_, Vec<u8>>(field_0, &child_path(path, "id"))?),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::DeclarationId,
> for pse_model::generated::identities::DeclarationId {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::DeclarationId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_model::generated::identities::DeclarationId as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::generated::identities::DeclarationId as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                Vec<u8>,
            >(required(wire.id, &child_path(path, "id"))?, &child_path(path, "id"))?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::EntityKindId,
> for pse_quantity::ids::EntityKindId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::EntityKindId,
        AdapterError,
    > {
        layout(
            <pse_quantity::ids::EntityKindId as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::EntityKindId as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let (field_0,) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::EntityKindId {
            id: Some(native_into::<_, Vec<u8>>(field_0, &child_path(path, "id"))?),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::EntityKindId,
> for pse_quantity::ids::EntityKindId {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::EntityKindId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::EntityKindId as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::EntityKindId as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                Vec<u8>,
            >(required(wire.id, &child_path(path, "id"))?, &child_path(path, "id"))?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::InstanceId,
> for pse_model::generated::identities::InstanceId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::InstanceId,
        AdapterError,
    > {
        layout(
            <pse_model::generated::identities::InstanceId as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::generated::identities::InstanceId as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let (field_0,) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::InstanceId {
            id: Some(native_into::<_, Vec<u8>>(field_0, &child_path(path, "id"))?),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::InstanceId,
> for pse_model::generated::identities::InstanceId {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::InstanceId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_model::generated::identities::InstanceId as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::generated::identities::InstanceId as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                Vec<u8>,
            >(required(wire.id, &child_path(path, "id"))?, &child_path(path, "id"))?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::KindDefinition,
> for pse_quantity::kind::KindDefinition {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::KindDefinition,
        AdapterError,
    > {
        layout(
            <pse_quantity::kind::KindDefinition as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::kind::KindDefinition as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
        let (field_0, field_1, field_2, field_3, field_4, field_5) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::KindDefinition {
            monomial: {
                let current_path = &child_path(path, "monomial");
                items(field_0)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::host_operations::engineering::generated::pb::pse::engineering::v1::KindFactor,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            canonical_unit: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::UnitId,
                >(field_1, &child_path(path, "canonical_unit"))?,
            ),
            basis: field_2
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::BasisId,
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
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceStateId,
                        >(value, &child_path(path, "reference_state"))?,
                    )
                })
                .transpose()?
                .into(),
            scale_kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ScaleKind,
                >(field_4, &child_path(path, "scale_kind"))?,
            ),
            subject_kind: field_5
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::EntityKindId,
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
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::KindDefinition,
> for pse_quantity::kind::KindDefinition {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::KindDefinition,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::kind::KindDefinition as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::kind::KindDefinition as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
                                crate::host_operations::engineering::generated::pb::pse::engineering::v1::KindFactor,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::UnitId,
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
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::BasisId,
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
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceStateId,
                        >(value, &child_path(path, "reference_state"))?,
                    )
                })
                .transpose()?,
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::ScaleKind,
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
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::EntityKindId,
                        >(value, &child_path(path, "subject_kind"))?,
                    )
                })
                .transpose()?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::KindFactor,
> for pse_quantity::kind::KindFactor {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::KindFactor,
        AdapterError,
    > {
        layout(
            <pse_quantity::kind::KindFactor as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::kind::KindFactor as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "kind"), (2, "exponent")],
            path,
        )?;
        let (field_0, field_1) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::KindFactor {
            kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityKindId,
                >(field_0, &child_path(path, "kind"))?,
            ),
            exponent: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalRatio,
                >(field_1, &child_path(path, "exponent"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::KindFactor,
> for pse_quantity::kind::KindFactor {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::KindFactor,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::kind::KindFactor as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::kind::KindFactor as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "kind"), (2, "exponent")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityKindId,
            >(
                required(wire.kind.into_option(), &child_path(path, "kind"))?,
                &child_path(path, "kind"),
            )?,
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalRatio,
            >(
                required(wire.exponent.into_option(), &child_path(path, "exponent"))?,
                &child_path(path, "exponent"),
            )?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingBindings,
> for pse_modeling::specialize::Bindings {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingBindings,
        AdapterError,
    > {
        layout(
            <pse_modeling::specialize::Bindings as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::specialize::Bindings as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "formulation"),
                (2, "arguments"),
                (3, "scope"),
                (4, "facts"),
                (5, "demand"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingBindings {
            formulation: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingFormulation,
                >(field_0, &child_path(path, "formulation"))?,
            ),
            arguments: wire_map(
                field_1
                    .into_iter()
                    .map(|(key, value)| -> Result<_, AdapterError> {
                        Ok((
                            native_into::<
                                _,
                                String,
                            >(key, &child_path(path, "arguments"))?,
                            native_into::<
                                _,
                                crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue,
                            >(value, &child_path(path, "arguments"))?,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter(),
            ),
            scope: wire_map(
                field_2
                    .into_iter()
                    .map(|(key, value)| -> Result<_, AdapterError> {
                        Ok((
                            native_into::<_, String>(key, &child_path(path, "scope"))?,
                            native_into::<
                                _,
                                crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue,
                            >(value, &child_path(path, "scope"))?,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter(),
            ),
            facts: field_3
                .into_iter()
                .map(|(key, value)| -> Result<_, AdapterError> {
                    Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::modeling_bindings::Field4Entry {
                        key: buffa::MessageField::some(
                            native_into::<
                                _,
                                crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingFact,
                            >(key, &child_path(path, "facts"))?,
                        ),
                        value: buffa::MessageField::some(
                            native_into::<
                                _,
                                crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue,
                            >(value, &child_path(path, "facts"))?,
                        ),
                        ..Default::default()
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
            demand: {
                let current_path = &child_path(path, "demand");
                items(field_4)
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
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingBindings,
> for pse_modeling::specialize::Bindings {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingBindings,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::specialize::Bindings as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::specialize::Bindings as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "formulation"),
                (2, "arguments"),
                (3, "scope"),
                (4, "facts"),
                (5, "demand"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingFormulation,
            >(
                required(
                    wire.formulation.into_option(),
                    &child_path(path, "formulation"),
                )?,
                &child_path(path, "formulation"),
            )?,
            unique_pairs(
                wire
                    .arguments
                    .into_iter()
                    .map(|(key, value)| -> Result<_, AdapterError> {
                        Ok((
                            native_from::<
                                _,
                                String,
                            >(key, &child_path(path, "arguments"))?,
                            native_from::<
                                _,
                                crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue,
                            >(value, &child_path(path, "arguments"))?,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                &child_path(path, "arguments"),
            )?,
            unique_pairs(
                wire
                    .scope
                    .into_iter()
                    .map(|(key, value)| -> Result<_, AdapterError> {
                        Ok((
                            native_from::<_, String>(key, &child_path(path, "scope"))?,
                            native_from::<
                                _,
                                crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue,
                            >(value, &child_path(path, "scope"))?,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                &child_path(path, "scope"),
            )?,
            unique_pairs(
                wire
                    .facts
                    .into_iter()
                    .map(|entry| -> Result<_, AdapterError> {
                        unknown_fields(
                            &entry.__buffa_unknown_fields,
                            &child_path(path, "facts"),
                        )?;
                        Ok((
                            native_from::<
                                _,
                                crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingFact,
                            >(
                                required(
                                    entry.key.into_option(),
                                    &child_path(path, "facts"),
                                )?,
                                &child_path(path, "facts"),
                            )?,
                            native_from::<
                                _,
                                crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue,
                            >(
                                required(
                                    entry.value.into_option(),
                                    &child_path(path, "facts"),
                                )?,
                                &child_path(path, "facts"),
                            )?,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                &child_path(path, "facts"),
            )?,
            {
                let current_path = &child_path(path, "demand");
                let values = wire
                    .demand
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
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingFact,
> for pse_modeling::analysis::Fact {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingFact,
        AdapterError,
    > {
        let value = self;
        layout(
            <pse_modeling::analysis::Fact as pse_model::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <pse_modeling::analysis::Fact as pse_model::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_modeling::analysis::Fact as pse_model::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingFact {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::modeling_fact::Kind::Route(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::modeling_fact::Alternative1 {
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
            <pse_modeling::analysis::Fact as pse_model::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <pse_modeling::analysis::Fact as pse_model::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_modeling::analysis::Fact as pse_model::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingFact {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::modeling_fact::Kind::Dynamic(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::modeling_fact::Alternative2 {
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
            <pse_modeling::analysis::Fact as pse_model::boundary_projection::ProjectionAlternative<
                3,
            >>::VERSION,
            <pse_modeling::analysis::Fact as pse_model::boundary_projection::ProjectionAlternative<
                3,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_modeling::analysis::Fact as pse_model::boundary_projection::ProjectionAlternative<
            3,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingFact {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::modeling_fact::Kind::ObjectiveLevel(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::modeling_fact::Alternative3 {
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
            <pse_modeling::analysis::Fact as pse_model::boundary_projection::ProjectionAlternative<
                4,
            >>::VERSION,
            <pse_modeling::analysis::Fact as pse_model::boundary_projection::ProjectionAlternative<
                4,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_modeling::analysis::Fact as pse_model::boundary_projection::ProjectionAlternative<
            4,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Stage");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingFact {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::modeling_fact::Kind::Stage(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::modeling_fact::Alternative4 {
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
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingFact,
> for pse_modeling::analysis::Fact {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingFact,
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
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::modeling_fact::Kind::Route(
                wire,
            ) => {
                let alternative_path = child_path(path, "Route");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::analysis::Fact as pse_model::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_modeling::analysis::Fact as pse_model::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::analysis::Fact as pse_model::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::modeling_fact::Kind::Dynamic(
                wire,
            ) => {
                let alternative_path = child_path(path, "Dynamic");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::analysis::Fact as pse_model::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_modeling::analysis::Fact as pse_model::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::analysis::Fact as pse_model::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::modeling_fact::Kind::ObjectiveLevel(
                wire,
            ) => {
                let alternative_path = child_path(path, "ObjectiveLevel");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::analysis::Fact as pse_model::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_modeling::analysis::Fact as pse_model::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::analysis::Fact as pse_model::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::modeling_fact::Kind::Stage(
                wire,
            ) => {
                let alternative_path = child_path(path, "Stage");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::analysis::Fact as pse_model::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <pse_modeling::analysis::Fact as pse_model::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<_, String>(wire.value, &child_path(path, "value"))?,
                );
                Ok(
                    <pse_modeling::analysis::Fact as pse_model::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingFormulation,
> for pse_modeling::specialize::Formulation {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingFormulation,
        AdapterError,
    > {
        layout(
            <pse_modeling::specialize::Formulation as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::specialize::Formulation as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "omitted"), (2, "elastic")],
            path,
        )?;
        let (field_0, field_1) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingFormulation {
            omitted: {
                let current_path = &child_path(path, "omitted");
                items(field_0)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                Vec<u8>,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            elastic: field_1
                .into_iter()
                .map(|(key, value)| -> Result<_, AdapterError> {
                    Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::modeling_formulation::Field2Entry {
                        key: native_into::<
                            _,
                            Vec<u8>,
                        >(key, &child_path(path, "elastic"))?,
                        value: buffa::MessageField::some(
                            native_into::<
                                _,
                                crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue,
                            >(value, &child_path(path, "elastic"))?,
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
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingFormulation,
> for pse_modeling::specialize::Formulation {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingFormulation,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::specialize::Formulation as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::specialize::Formulation as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "omitted"), (2, "elastic")],
            path,
        )?;
        let fields = (
            {
                let current_path = &child_path(path, "omitted");
                let values = wire
                    .omitted
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                Vec<u8>,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            unique_pairs(
                wire
                    .elastic
                    .into_iter()
                    .map(|entry| -> Result<_, AdapterError> {
                        unknown_fields(
                            &entry.__buffa_unknown_fields,
                            &child_path(path, "elastic"),
                        )?;
                        Ok((
                            native_from::<
                                _,
                                Vec<u8>,
                            >(entry.key, &child_path(path, "elastic"))?,
                            native_from::<
                                _,
                                crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue,
                            >(
                                required(
                                    entry.value.into_option(),
                                    &child_path(path, "elastic"),
                                )?,
                                &child_path(path, "elastic"),
                            )?,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                &child_path(path, "elastic"),
            )?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingLimits,
> for pse_modeling::specialize::Limits {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingLimits,
        AdapterError,
    > {
        layout(
            <pse_modeling::specialize::Limits as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::specialize::Limits as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "depth"),
                (2, "items"),
                (3, "members"),
                (4, "body_occurrences"),
                (5, "body_slots"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingLimits {
            depth: native_into::<_, u64>(field_0, &child_path(path, "depth"))?,
            items: native_into::<_, u64>(field_1, &child_path(path, "items"))?,
            members: native_into::<_, u64>(field_2, &child_path(path, "members"))?,
            body_occurrences: field_3
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            u64,
                        >(value, &child_path(path, "body_occurrences"))?,
                    )
                })
                .transpose()?,
            body_slots: field_4
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_into::<_, u64>(value, &child_path(path, "body_slots"))?)
                })
                .transpose()?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingLimits,
> for pse_modeling::specialize::Limits {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingLimits,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::specialize::Limits as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::specialize::Limits as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "depth"),
                (2, "items"),
                (3, "members"),
                (4, "body_occurrences"),
                (5, "body_slots"),
            ],
            path,
        )?;
        let fields = (
            native_from::<_, u64>(wire.depth, &child_path(path, "depth"))?,
            native_from::<_, u64>(wire.items, &child_path(path, "items"))?,
            native_from::<_, u64>(wire.members, &child_path(path, "members"))?,
            wire
                .body_occurrences
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            u64,
                        >(value, &child_path(path, "body_occurrences"))?,
                    )
                })
                .transpose()?,
            wire
                .body_slots
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_from::<_, u64>(value, &child_path(path, "body_slots"))?)
                })
                .transpose()?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::OriginalSourceFile,
> for crate::workflow::OriginalSourceFile {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::OriginalSourceFile,
        AdapterError,
    > {
        layout(
            <crate::workflow::OriginalSourceFile as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::OriginalSourceFile as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "local_revision"),
                (2, "package"),
                (3, "document"),
                (4, "path"),
                (5, "document_version"),
                (6, "original_hash"),
                (7, "is_text"),
                (8, "body"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5, field_6, field_7) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::OriginalSourceFile {
            local_revision: native_into::<
                _,
                String,
            >(field_0, &child_path(path, "local_revision"))?,
            package: native_into::<_, Vec<u8>>(field_1, &child_path(path, "package"))?,
            document: native_into::<_, Vec<u8>>(field_2, &child_path(path, "document"))?,
            path: native_into::<_, String>(field_3, &child_path(path, "path"))?,
            document_version: native_into::<
                _,
                String,
            >(field_4, &child_path(path, "document_version"))?,
            original_hash: native_into::<
                _,
                Vec<u8>,
            >(field_5, &child_path(path, "original_hash"))?,
            is_text: native_into::<_, bool>(field_6, &child_path(path, "is_text"))?,
            body: native_into::<_, Vec<u8>>(field_7, &child_path(path, "body"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::OriginalSourceFile,
> for crate::workflow::OriginalSourceFile {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::OriginalSourceFile,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::workflow::OriginalSourceFile as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::OriginalSourceFile as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "local_revision"),
                (2, "package"),
                (3, "document"),
                (4, "path"),
                (5, "document_version"),
                (6, "original_hash"),
                (7, "is_text"),
                (8, "body"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                String,
            >(wire.local_revision, &child_path(path, "local_revision"))?,
            native_from::<_, Vec<u8>>(wire.package, &child_path(path, "package"))?,
            native_from::<_, Vec<u8>>(wire.document, &child_path(path, "document"))?,
            native_from::<_, String>(wire.path, &child_path(path, "path"))?,
            native_from::<
                _,
                String,
            >(wire.document_version, &child_path(path, "document_version"))?,
            native_from::<
                _,
                Vec<u8>,
            >(wire.original_hash, &child_path(path, "original_hash"))?,
            native_from::<_, bool>(wire.is_text, &child_path(path, "is_text"))?,
            native_from::<_, Vec<u8>>(wire.body, &child_path(path, "body"))?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::OriginalSourceFileRequest,
> for crate::workflow::OriginalSourceFileRequest {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::OriginalSourceFileRequest,
        AdapterError,
    > {
        layout(
            <crate::workflow::OriginalSourceFileRequest as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::OriginalSourceFileRequest as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "parent"), (2, "package"), (3, "path"), (4, "maximum_bytes")],
            path,
        )?;
        let (field_0, field_1, field_2, field_3) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::OriginalSourceFileRequest {
            parent: native_into::<_, String>(field_0, &child_path(path, "parent"))?,
            package: native_into::<_, Vec<u8>>(field_1, &child_path(path, "package"))?,
            path: native_into::<_, String>(field_2, &child_path(path, "path"))?,
            maximum_bytes: native_into::<
                _,
                u64,
            >(field_3, &child_path(path, "maximum_bytes"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::OriginalSourceFileRequest,
> for crate::workflow::OriginalSourceFileRequest {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::OriginalSourceFileRequest,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::workflow::OriginalSourceFileRequest as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::OriginalSourceFileRequest as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "parent"), (2, "package"), (3, "path"), (4, "maximum_bytes")],
            path,
        )?;
        let fields = (
            native_from::<_, String>(wire.parent, &child_path(path, "parent"))?,
            native_from::<_, Vec<u8>>(wire.package, &child_path(path, "package"))?,
            native_from::<_, String>(wire.path, &child_path(path, "path"))?,
            native_from::<
                _,
                u64,
            >(wire.maximum_bytes, &child_path(path, "maximum_bytes"))?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalBasis,
> for pse_quantity::basis::Basis {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalBasis,
        AdapterError,
    > {
        layout(
            <pse_quantity::basis::Basis as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::basis::Basis as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
        let (field_0, field_1, field_2, field_3, field_4) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalBasis {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::BasisId,
                >(field_0, &child_path(path, "id"))?,
            ),
            kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::BasisKind,
                >(field_1, &child_path(path, "kind"))?,
            ),
            composition_basis: field_2
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::CompositionBasis,
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
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::RateBasis,
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
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceStateId,
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
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalBasis,
> for pse_quantity::basis::Basis {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalBasis,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::basis::Basis as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::basis::Basis as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::BasisId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::BasisKind,
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
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::CompositionBasis,
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
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::RateBasis,
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
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceStateId,
                        >(value, &child_path(path, "reference_conditions"))?,
                    )
                })
                .transpose()?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalDimensionVector,
> for pse_quantity::dimension::DimensionVector {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalDimensionVector,
        AdapterError,
    > {
        layout(
            <pse_quantity::dimension::DimensionVector as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::dimension::DimensionVector as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "exponents")],
            path,
        )?;
        let (field_0,) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalDimensionVector {
            exponents: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::Array8PhysicalRatio,
                >(field_0, &child_path(path, "exponents"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalDimensionVector,
> for pse_quantity::dimension::DimensionVector {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalDimensionVector,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::dimension::DimensionVector as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::dimension::DimensionVector as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "exponents")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::Array8PhysicalRatio,
            >(
                required(wire.exponents.into_option(), &child_path(path, "exponents"))?,
                &child_path(path, "exponents"),
            )?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalEntityKind,
> for pse_quantity::entity_kind::EntityKind {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalEntityKind,
        AdapterError,
    > {
        layout(
            <pse_quantity::entity_kind::EntityKind as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::entity_kind::EntityKind as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "name")],
            path,
        )?;
        let (field_0, field_1) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalEntityKind {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::EntityKindId,
                >(field_0, &child_path(path, "id"))?,
            ),
            name: native_into::<_, String>(field_1, &child_path(path, "name"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalEntityKind,
> for pse_quantity::entity_kind::EntityKind {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalEntityKind,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::entity_kind::EntityKind as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::entity_kind::EntityKind as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "name")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::EntityKindId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<_, String>(wire.name, &child_path(path, "name"))?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalInspectionDocument,
> for crate::workflow::PhysicalInspectionDocument {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalInspectionDocument,
        AdapterError,
    > {
        let value = self;
        layout(
            <crate::workflow::PhysicalInspectionDocument as pse_model::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <crate::workflow::PhysicalInspectionDocument as pse_model::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[
                (1, "context"),
                (2, "count"),
                (3, "offset"),
                (4, "quantities"),
                (5, "reference_states"),
                (6, "next"),
            ],
            path,
        )?;
        let value = match <crate::workflow::PhysicalInspectionDocument as pse_model::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1, field_2, field_3, field_4, field_5)) => {
                let alternative_path = child_path(path, "Page");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalInspectionDocument {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::physical_inspection_document::Kind::Page(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::physical_inspection_document::Alternative1 {
                                context: Some(
                                    native_into::<
                                        _,
                                        Vec<u8>,
                                    >(field_0, &child_path(path, "context"))?,
                                ),
                                count: native_into::<
                                    _,
                                    u64,
                                >(field_1, &child_path(path, "count"))?,
                                offset: native_into::<
                                    _,
                                    u64,
                                >(field_2, &child_path(path, "offset"))?,
                                quantities: {
                                    let current_path = &child_path(path, "quantities");
                                    items(field_3)
                                        .enumerate()
                                        .map(|(index, value)| -> Result<_, AdapterError> {
                                            Ok(
                                                native_into::<
                                                    _,
                                                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalQuantityDescription,
                                                >(value, &format!("{path}[{index}]", path = current_path))?,
                                            )
                                        })
                                        .collect::<Result<Vec<_>, _>>()?
                                },
                                reference_states: {
                                    let current_path = &child_path(path, "reference_states");
                                    items(field_4)
                                        .enumerate()
                                        .map(|(index, value)| -> Result<_, AdapterError> {
                                            Ok(
                                                native_into::<
                                                    _,
                                                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceState,
                                                >(value, &format!("{path}[{index}]", path = current_path))?,
                                            )
                                        })
                                        .collect::<Result<Vec<_>, _>>()?
                                },
                                next: field_5
                                    .map(|value| -> Result<_, AdapterError> {
                                        Ok(native_into::<_, u64>(value, &child_path(path, "next"))?)
                                    })
                                    .transpose()?,
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
            <crate::workflow::PhysicalInspectionDocument as pse_model::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <crate::workflow::PhysicalInspectionDocument as pse_model::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "context"), (2, "reason")],
            path,
        )?;
        let value = match <crate::workflow::PhysicalInspectionDocument as pse_model::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "Unavailable");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalInspectionDocument {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::physical_inspection_document::Kind::Unavailable(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::physical_inspection_document::Alternative2 {
                                context: Some(
                                    native_into::<
                                        _,
                                        Vec<u8>,
                                    >(field_0, &child_path(path, "context"))?,
                                ),
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
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalInspectionDocument,
> for crate::workflow::PhysicalInspectionDocument {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalInspectionDocument,
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
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::physical_inspection_document::Kind::Page(
                wire,
            ) => {
                let alternative_path = child_path(path, "Page");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::workflow::PhysicalInspectionDocument as pse_model::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <crate::workflow::PhysicalInspectionDocument as pse_model::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[
                        (1, "context"),
                        (2, "count"),
                        (3, "offset"),
                        (4, "quantities"),
                        (5, "reference_states"),
                        (6, "next"),
                    ],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        Vec<u8>,
                    >(
                        required(wire.context, &child_path(path, "context"))?,
                        &child_path(path, "context"),
                    )?,
                    native_from::<_, u64>(wire.count, &child_path(path, "count"))?,
                    native_from::<_, u64>(wire.offset, &child_path(path, "offset"))?,
                    {
                        let current_path = &child_path(path, "quantities");
                        let values = wire
                            .quantities
                            .into_iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                Ok(
                                    native_from::<
                                        _,
                                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalQuantityDescription,
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
                                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceState,
                                    >(value, &format!("{path}[{index}]", path = current_path))?,
                                )
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                    wire
                        .next
                        .map(|value| -> Result<_, AdapterError> {
                            Ok(native_from::<_, u64>(value, &child_path(path, "next"))?)
                        })
                        .transpose()?,
                );
                Ok(
                    <crate::workflow::PhysicalInspectionDocument as pse_model::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::physical_inspection_document::Kind::Unavailable(
                wire,
            ) => {
                let alternative_path = child_path(path, "Unavailable");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::workflow::PhysicalInspectionDocument as pse_model::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <crate::workflow::PhysicalInspectionDocument as pse_model::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "context"), (2, "reason")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        Vec<u8>,
                    >(
                        required(wire.context, &child_path(path, "context"))?,
                        &child_path(path, "context"),
                    )?,
                    native_from::<_, String>(wire.reason, &child_path(path, "reason"))?,
                );
                Ok(
                    <crate::workflow::PhysicalInspectionDocument as pse_model::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalInspectionQuery,
> for crate::workflow::PhysicalInspectionQuery {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalInspectionQuery,
        AdapterError,
    > {
        let value = self;
        layout(
            <crate::workflow::PhysicalInspectionQuery as pse_model::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <crate::workflow::PhysicalInspectionQuery as pse_model::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::workflow::PhysicalInspectionQuery as pse_model::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalInspectionQuery {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::physical_inspection_query::Kind::Quantities(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::physical_inspection_query::Alternative1 {
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
            <crate::workflow::PhysicalInspectionQuery as pse_model::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <crate::workflow::PhysicalInspectionQuery as pse_model::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "unit")],
            path,
        )?;
        let value = match <crate::workflow::PhysicalInspectionQuery as pse_model::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "Quantity");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalInspectionQuery {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::physical_inspection_query::Kind::Quantity(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::physical_inspection_query::Alternative2 {
                                id: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityTypeId,
                                    >(field_0, &child_path(path, "id"))?,
                                ),
                                unit: field_1
                                    .map(|value| -> Result<_, AdapterError> {
                                        Ok(
                                            native_into::<
                                                _,
                                                crate::host_operations::engineering::generated::pb::pse::engineering::v1::UnitId,
                                            >(value, &child_path(path, "unit"))?,
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
            <crate::workflow::PhysicalInspectionQuery as pse_model::boundary_projection::ProjectionAlternative<
                3,
            >>::VERSION,
            <crate::workflow::PhysicalInspectionQuery as pse_model::boundary_projection::ProjectionAlternative<
                3,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "name")],
            path,
        )?;
        let value = match <crate::workflow::PhysicalInspectionQuery as pse_model::boundary_projection::ProjectionAlternative<
            3,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "QuantityNamed");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalInspectionQuery {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::physical_inspection_query::Kind::QuantityNamed(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::physical_inspection_query::Alternative3 {
                                name: native_into::<
                                    _,
                                    String,
                                >(field_0, &child_path(path, "name"))?,
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
            <crate::workflow::PhysicalInspectionQuery as pse_model::boundary_projection::ProjectionAlternative<
                4,
            >>::VERSION,
            <crate::workflow::PhysicalInspectionQuery as pse_model::boundary_projection::ProjectionAlternative<
                4,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "name")],
            path,
        )?;
        let value = match <crate::workflow::PhysicalInspectionQuery as pse_model::boundary_projection::ProjectionAlternative<
            4,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "ReferenceStateNamed");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalInspectionQuery {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::physical_inspection_query::Kind::ReferenceStateNamed(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::physical_inspection_query::Alternative4 {
                                name: native_into::<
                                    _,
                                    String,
                                >(field_0, &child_path(path, "name"))?,
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
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalInspectionQuery,
> for crate::workflow::PhysicalInspectionQuery {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalInspectionQuery,
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
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::physical_inspection_query::Kind::Quantities(
                wire,
            ) => {
                let alternative_path = child_path(path, "Quantities");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::workflow::PhysicalInspectionQuery as pse_model::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <crate::workflow::PhysicalInspectionQuery as pse_model::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::workflow::PhysicalInspectionQuery as pse_model::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::physical_inspection_query::Kind::Quantity(
                wire,
            ) => {
                let alternative_path = child_path(path, "Quantity");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::workflow::PhysicalInspectionQuery as pse_model::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <crate::workflow::PhysicalInspectionQuery as pse_model::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "id"), (2, "unit")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityTypeId,
                    >(
                        required(wire.id.into_option(), &child_path(path, "id"))?,
                        &child_path(path, "id"),
                    )?,
                    wire
                        .unit
                        .into_option()
                        .map(|value| -> Result<_, AdapterError> {
                            Ok(
                                native_from::<
                                    _,
                                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::UnitId,
                                >(value, &child_path(path, "unit"))?,
                            )
                        })
                        .transpose()?,
                );
                Ok(
                    <crate::workflow::PhysicalInspectionQuery as pse_model::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::physical_inspection_query::Kind::QuantityNamed(
                wire,
            ) => {
                let alternative_path = child_path(path, "QuantityNamed");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::workflow::PhysicalInspectionQuery as pse_model::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <crate::workflow::PhysicalInspectionQuery as pse_model::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "name")],
                    path,
                )?;
                let fields = (
                    native_from::<_, String>(wire.name, &child_path(path, "name"))?,
                );
                Ok(
                    <crate::workflow::PhysicalInspectionQuery as pse_model::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::physical_inspection_query::Kind::ReferenceStateNamed(
                wire,
            ) => {
                let alternative_path = child_path(path, "ReferenceStateNamed");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::workflow::PhysicalInspectionQuery as pse_model::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <crate::workflow::PhysicalInspectionQuery as pse_model::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "name")],
                    path,
                )?;
                let fields = (
                    native_from::<_, String>(wire.name, &child_path(path, "name"))?,
                );
                Ok(
                    <crate::workflow::PhysicalInspectionQuery as pse_model::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalInspectionRequest,
> for crate::workflow::PhysicalInspectionRequest {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalInspectionRequest,
        AdapterError,
    > {
        layout(
            <crate::workflow::PhysicalInspectionRequest as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::PhysicalInspectionRequest as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "query"), (2, "offset"), (3, "limit")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalInspectionRequest {
            query: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalInspectionQuery,
                >(field_0, &child_path(path, "query"))?,
            ),
            offset: native_into::<_, u64>(field_1, &child_path(path, "offset"))?,
            limit: native_into::<_, u64>(field_2, &child_path(path, "limit"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalInspectionRequest,
> for crate::workflow::PhysicalInspectionRequest {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalInspectionRequest,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::workflow::PhysicalInspectionRequest as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::PhysicalInspectionRequest as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "query"), (2, "offset"), (3, "limit")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalInspectionQuery,
            >(
                required(wire.query.into_option(), &child_path(path, "query"))?,
                &child_path(path, "query"),
            )?,
            native_from::<_, u64>(wire.offset, &child_path(path, "offset"))?,
            native_from::<_, u64>(wire.limit, &child_path(path, "limit"))?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalQuantityDescription,
> for crate::workflow::PhysicalQuantityDescription {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalQuantityDescription,
        AdapterError,
    > {
        layout(
            <crate::workflow::PhysicalQuantityDescription as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::PhysicalQuantityDescription as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "quantity"),
                (2, "quantity_kind"),
                (3, "canonical_unit"),
                (4, "representation_unit"),
                (5, "scale_to_canonical"),
                (6, "offset_to_canonical"),
                (7, "basis"),
                (8, "basis_reference_state"),
                (9, "reference_state"),
                (10, "shape"),
                (11, "subject"),
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
        ) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalQuantityDescription {
            quantity: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityType,
                >(field_0, &child_path(path, "quantity"))?,
            ),
            quantity_kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalQuantityKind,
                >(field_1, &child_path(path, "quantity_kind"))?,
            ),
            canonical_unit: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalUnit,
                >(field_2, &child_path(path, "canonical_unit"))?,
            ),
            representation_unit: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalUnit,
                >(field_3, &child_path(path, "representation_unit"))?,
            ),
            scale_to_canonical: native_into::<
                _,
                u64,
            >(field_4, &child_path(path, "scale_to_canonical"))?,
            offset_to_canonical: native_into::<
                _,
                u64,
            >(field_5, &child_path(path, "offset_to_canonical"))?,
            basis: field_6
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalBasis,
                        >(value, &child_path(path, "basis"))?,
                    )
                })
                .transpose()?
                .into(),
            basis_reference_state: field_7
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceState,
                        >(value, &child_path(path, "basis_reference_state"))?,
                    )
                })
                .transpose()?
                .into(),
            reference_state: field_8
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceState,
                        >(value, &child_path(path, "reference_state"))?,
                    )
                })
                .transpose()?
                .into(),
            shape: {
                let current_path = &child_path(path, "shape");
                items(field_9)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalEntityKind,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            subject: field_10
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalEntityKind,
                        >(value, &child_path(path, "subject"))?,
                    )
                })
                .transpose()?
                .into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalQuantityDescription,
> for crate::workflow::PhysicalQuantityDescription {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalQuantityDescription,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::workflow::PhysicalQuantityDescription as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::PhysicalQuantityDescription as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "quantity"),
                (2, "quantity_kind"),
                (3, "canonical_unit"),
                (4, "representation_unit"),
                (5, "scale_to_canonical"),
                (6, "offset_to_canonical"),
                (7, "basis"),
                (8, "basis_reference_state"),
                (9, "reference_state"),
                (10, "shape"),
                (11, "subject"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityType,
            >(
                required(wire.quantity.into_option(), &child_path(path, "quantity"))?,
                &child_path(path, "quantity"),
            )?,
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalQuantityKind,
            >(
                required(
                    wire.quantity_kind.into_option(),
                    &child_path(path, "quantity_kind"),
                )?,
                &child_path(path, "quantity_kind"),
            )?,
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalUnit,
            >(
                required(
                    wire.canonical_unit.into_option(),
                    &child_path(path, "canonical_unit"),
                )?,
                &child_path(path, "canonical_unit"),
            )?,
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalUnit,
            >(
                required(
                    wire.representation_unit.into_option(),
                    &child_path(path, "representation_unit"),
                )?,
                &child_path(path, "representation_unit"),
            )?,
            native_from::<
                _,
                u64,
            >(wire.scale_to_canonical, &child_path(path, "scale_to_canonical"))?,
            native_from::<
                _,
                u64,
            >(wire.offset_to_canonical, &child_path(path, "offset_to_canonical"))?,
            wire
                .basis
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalBasis,
                        >(value, &child_path(path, "basis"))?,
                    )
                })
                .transpose()?,
            wire
                .basis_reference_state
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceState,
                        >(value, &child_path(path, "basis_reference_state"))?,
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
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceState,
                        >(value, &child_path(path, "reference_state"))?,
                    )
                })
                .transpose()?,
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
                                crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalEntityKind,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            wire
                .subject
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalEntityKind,
                        >(value, &child_path(path, "subject"))?,
                    )
                })
                .transpose()?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalQuantityKind,
> for pse_quantity::kind::QuantityKind {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalQuantityKind,
        AdapterError,
    > {
        layout(
            <pse_quantity::kind::QuantityKind as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::kind::QuantityKind as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
        let (field_0, field_1, field_2, field_3, field_4, field_5) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalQuantityKind {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityKindId,
                >(field_0, &child_path(path, "id"))?,
            ),
            dimension: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalDimensionVector,
                >(field_1, &child_path(path, "dimension"))?,
            ),
            extensive: native_into::<_, bool>(field_2, &child_path(path, "extensive"))?,
            addition_kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityAdditionKind,
                >(field_3, &child_path(path, "addition_kind"))?,
            ),
            category: field_4
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityKindCategory,
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
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::KindDefinition,
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
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalQuantityKind,
> for pse_quantity::kind::QuantityKind {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalQuantityKind,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::kind::QuantityKind as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::kind::QuantityKind as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityKindId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalDimensionVector,
            >(
                required(wire.dimension.into_option(), &child_path(path, "dimension"))?,
                &child_path(path, "dimension"),
            )?,
            native_from::<_, bool>(wire.extensive, &child_path(path, "extensive"))?,
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityAdditionKind,
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
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityKindCategory,
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
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::KindDefinition,
                        >(value, &child_path(path, "definition"))?,
                    )
                })
                .transpose()?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalRatio,
> for pse_quantity::dimension::Ratio {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalRatio,
        AdapterError,
    > {
        layout(
            <pse_quantity::dimension::Ratio as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::dimension::Ratio as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "num"), (2, "den")],
            path,
        )?;
        let (field_0, field_1) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalRatio {
            num: native_into::<_, i32>(field_0, &child_path(path, "num"))?,
            den: native_into::<_, i32>(field_1, &child_path(path, "den"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalRatio,
> for pse_quantity::dimension::Ratio {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalRatio,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::dimension::Ratio as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::dimension::Ratio as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "num"), (2, "den")],
            path,
        )?;
        let fields = (
            native_from::<_, i32>(wire.num, &child_path(path, "num"))?,
            native_from::<_, i32>(wire.den, &child_path(path, "den"))?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalUnit,
> for pse_quantity::unit::Unit {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalUnit,
        AdapterError,
    > {
        layout(
            <pse_quantity::unit::Unit as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::unit::Unit as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
        let (field_0, field_1, field_2, field_3, field_4, field_5, field_6, field_7) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalUnit {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::UnitId,
                >(field_0, &child_path(path, "id"))?,
            ),
            symbol: native_into::<_, String>(field_1, &child_path(path, "symbol"))?,
            dimension: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalDimensionVector,
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
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceStateId,
                        >(value, &child_path(path, "reference_state"))?,
                    )
                })
                .transpose()?
                .into(),
            definition: field_7
                .map(|value| -> Result<_, AdapterError> {
                    Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::physical_unit::Field8Value {
                        value: {
                            let current_path = &child_path(path, "definition");
                            items(value)
                                .enumerate()
                                .map(|(index, value)| -> Result<_, AdapterError> {
                                    Ok(
                                        native_into::<
                                            _,
                                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::UnitFactor,
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
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalUnit,
> for pse_quantity::unit::Unit {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalUnit,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::unit::Unit as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::unit::Unit as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::UnitId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<_, String>(wire.symbol, &child_path(path, "symbol"))?,
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalDimensionVector,
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
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceStateId,
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
                                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::UnitFactor,
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
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationDiscoveryReport,
> for crate::workflow::PreparationDiscoveryReport {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationDiscoveryReport,
        AdapterError,
    > {
        layout(
            <crate::workflow::PreparationDiscoveryReport as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::PreparationDiscoveryReport as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "revision_id"),
                (2, "physical"),
                (3, "resident"),
                (4, "residents_truncated"),
                (5, "durable"),
                (6, "durable_eligibility"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationDiscoveryReport {
            revision_id: native_into::<
                _,
                String,
            >(field_0, &child_path(path, "revision_id"))?,
            physical: native_into::<_, String>(field_1, &child_path(path, "physical"))?,
            resident: {
                let current_path = &child_path(path, "resident");
                items(field_2)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::host_operations::engineering::generated::pb::pse::engineering::v1::ResidentPreparation,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            residents_truncated: native_into::<
                _,
                bool,
            >(field_3, &child_path(path, "residents_truncated"))?,
            durable: field_4
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::ProductHeaderPage,
                        >(value, &child_path(path, "durable"))?,
                    )
                })
                .transpose()?
                .into(),
            durable_eligibility: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationEligibility,
                >(field_5, &child_path(path, "durable_eligibility"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationDiscoveryReport,
> for crate::workflow::PreparationDiscoveryReport {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationDiscoveryReport,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::workflow::PreparationDiscoveryReport as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::PreparationDiscoveryReport as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "revision_id"),
                (2, "physical"),
                (3, "resident"),
                (4, "residents_truncated"),
                (5, "durable"),
                (6, "durable_eligibility"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                String,
            >(wire.revision_id, &child_path(path, "revision_id"))?,
            native_from::<_, String>(wire.physical, &child_path(path, "physical"))?,
            {
                let current_path = &child_path(path, "resident");
                let values = wire
                    .resident
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::host_operations::engineering::generated::pb::pse::engineering::v1::ResidentPreparation,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                bool,
            >(wire.residents_truncated, &child_path(path, "residents_truncated"))?,
            wire
                .durable
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::ProductHeaderPage,
                        >(value, &child_path(path, "durable"))?,
                    )
                })
                .transpose()?,
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationEligibility,
            >(
                required(
                    wire.durable_eligibility.into_option(),
                    &child_path(path, "durable_eligibility"),
                )?,
                &child_path(path, "durable_eligibility"),
            )?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationDiscoveryRequest,
> for crate::workflow::PreparationDiscoveryRequest {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationDiscoveryRequest,
        AdapterError,
    > {
        layout(
            <crate::workflow::PreparationDiscoveryRequest as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::PreparationDiscoveryRequest as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            2,
            &[
                (1, "root"),
                (2, "instance"),
                (3, "bindings"),
                (4, "limits"),
                (5, "durable_after"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationDiscoveryRequest {
            root: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::DeclarationId,
                >(field_0, &child_path(path, "root"))?,
            ),
            instance: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::InstanceId,
                >(field_1, &child_path(path, "instance"))?,
            ),
            bindings: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingBindings,
                >(field_2, &child_path(path, "bindings"))?,
            ),
            limits: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingLimits,
                >(field_3, &child_path(path, "limits"))?,
            ),
            durable_after: field_4
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::ProductHeaderCursor,
                        >(value, &child_path(path, "durable_after"))?,
                    )
                })
                .transpose()?
                .into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationDiscoveryRequest,
> for crate::workflow::PreparationDiscoveryRequest {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationDiscoveryRequest,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::workflow::PreparationDiscoveryRequest as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::PreparationDiscoveryRequest as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            2,
            &[
                (1, "root"),
                (2, "instance"),
                (3, "bindings"),
                (4, "limits"),
                (5, "durable_after"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::DeclarationId,
            >(
                required(wire.root.into_option(), &child_path(path, "root"))?,
                &child_path(path, "root"),
            )?,
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::InstanceId,
            >(
                required(wire.instance.into_option(), &child_path(path, "instance"))?,
                &child_path(path, "instance"),
            )?,
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingBindings,
            >(
                required(wire.bindings.into_option(), &child_path(path, "bindings"))?,
                &child_path(path, "bindings"),
            )?,
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::ModelingLimits,
            >(
                required(wire.limits.into_option(), &child_path(path, "limits"))?,
                &child_path(path, "limits"),
            )?,
            wire
                .durable_after
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::ProductHeaderCursor,
                        >(value, &child_path(path, "durable_after"))?,
                    )
                })
                .transpose()?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationEligibility,
> for pse_model::generated::enums::PreparationEligibility {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationEligibility,
        AdapterError,
    > {
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationEligibility {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationEligibility,
> for pse_model::generated::enums::PreparationEligibility {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationEligibility,
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
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationProductKind,
> for pse_model::generated::enums::PreparationProductKind {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationProductKind,
        AdapterError,
    > {
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationProductKind {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationProductKind,
> for pse_model::generated::enums::PreparationProductKind {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationProductKind,
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
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ProductHeader,
> for pse_operations::canonical_selection::ProductHeader {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::ProductHeader,
        AdapterError,
    > {
        layout(
            <pse_operations::canonical_selection::ProductHeader as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_operations::canonical_selection::ProductHeader as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "key"),
                (2, "revision"),
                (3, "producer"),
                (4, "producer_class"),
                (5, "interpretation"),
                (6, "descriptor"),
                (7, "content"),
                (8, "receipt"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5, field_6, field_7) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::ProductHeader {
            key: native_into::<_, String>(field_0, &child_path(path, "key"))?,
            revision: native_into::<_, String>(field_1, &child_path(path, "revision"))?,
            producer: native_into::<_, String>(field_2, &child_path(path, "producer"))?,
            producer_class: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::CanonicalProducerClass,
                >(field_3, &child_path(path, "producer_class"))?,
            ),
            interpretation: native_into::<
                _,
                String,
            >(field_4, &child_path(path, "interpretation"))?,
            descriptor: native_into::<
                _,
                String,
            >(field_5, &child_path(path, "descriptor"))?,
            content: native_into::<_, String>(field_6, &child_path(path, "content"))?,
            receipt: native_into::<_, String>(field_7, &child_path(path, "receipt"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ProductHeader,
> for pse_operations::canonical_selection::ProductHeader {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::ProductHeader,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_operations::canonical_selection::ProductHeader as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_operations::canonical_selection::ProductHeader as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "key"),
                (2, "revision"),
                (3, "producer"),
                (4, "producer_class"),
                (5, "interpretation"),
                (6, "descriptor"),
                (7, "content"),
                (8, "receipt"),
            ],
            path,
        )?;
        let fields = (
            native_from::<_, String>(wire.key, &child_path(path, "key"))?,
            native_from::<_, String>(wire.revision, &child_path(path, "revision"))?,
            native_from::<_, String>(wire.producer, &child_path(path, "producer"))?,
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::CanonicalProducerClass,
            >(
                required(
                    wire.producer_class.into_option(),
                    &child_path(path, "producer_class"),
                )?,
                &child_path(path, "producer_class"),
            )?,
            native_from::<
                _,
                String,
            >(wire.interpretation, &child_path(path, "interpretation"))?,
            native_from::<_, String>(wire.descriptor, &child_path(path, "descriptor"))?,
            native_from::<_, String>(wire.content, &child_path(path, "content"))?,
            native_from::<_, String>(wire.receipt, &child_path(path, "receipt"))?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ProductHeaderCursor,
> for pse_operations::canonical_selection::ProductHeaderCursor {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::ProductHeaderCursor,
        AdapterError,
    > {
        layout(
            <pse_operations::canonical_selection::ProductHeaderCursor as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_operations::canonical_selection::ProductHeaderCursor as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "issuing_store"), (2, "problem"), (3, "revision"), (4, "after")],
            path,
        )?;
        let (field_0, field_1, field_2, field_3) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::ProductHeaderCursor {
            issuing_store: native_into::<
                _,
                String,
            >(field_0, &child_path(path, "issuing_store"))?,
            problem: native_into::<_, String>(field_1, &child_path(path, "problem"))?,
            revision: native_into::<_, String>(field_2, &child_path(path, "revision"))?,
            after: native_into::<_, String>(field_3, &child_path(path, "after"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ProductHeaderCursor,
> for pse_operations::canonical_selection::ProductHeaderCursor {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::ProductHeaderCursor,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_operations::canonical_selection::ProductHeaderCursor as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_operations::canonical_selection::ProductHeaderCursor as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "issuing_store"), (2, "problem"), (3, "revision"), (4, "after")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                String,
            >(wire.issuing_store, &child_path(path, "issuing_store"))?,
            native_from::<_, String>(wire.problem, &child_path(path, "problem"))?,
            native_from::<_, String>(wire.revision, &child_path(path, "revision"))?,
            native_from::<_, String>(wire.after, &child_path(path, "after"))?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ProductHeaderPage,
> for pse_operations::canonical_selection::ProductHeaderPage {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::ProductHeaderPage,
        AdapterError,
    > {
        layout(
            <pse_operations::canonical_selection::ProductHeaderPage as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_operations::canonical_selection::ProductHeaderPage as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "rows"), (2, "next")],
            path,
        )?;
        let (field_0, field_1) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::ProductHeaderPage {
            rows: {
                let current_path = &child_path(path, "rows");
                items(field_0)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::host_operations::engineering::generated::pb::pse::engineering::v1::ProductHeader,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            next: field_1
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::ProductHeaderCursor,
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
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ProductHeaderPage,
> for pse_operations::canonical_selection::ProductHeaderPage {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::ProductHeaderPage,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_operations::canonical_selection::ProductHeaderPage as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_operations::canonical_selection::ProductHeaderPage as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "rows"), (2, "next")],
            path,
        )?;
        let fields = (
            {
                let current_path = &child_path(path, "rows");
                let values = wire
                    .rows
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::host_operations::engineering::generated::pb::pse::engineering::v1::ProductHeader,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            wire
                .next
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::ProductHeaderCursor,
                        >(value, &child_path(path, "next"))?,
                    )
                })
                .transpose()?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityAdditionKind,
> for pse_quantity::enums::QuantityAdditionKind {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityAdditionKind,
        AdapterError,
    > {
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityAdditionKind {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityAdditionKind,
> for pse_quantity::enums::QuantityAdditionKind {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityAdditionKind,
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
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityKindCategory,
> for pse_quantity::enums::QuantityKindCategory {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityKindCategory,
        AdapterError,
    > {
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityKindCategory {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityKindCategory,
> for pse_quantity::enums::QuantityKindCategory {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityKindCategory,
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
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityKindId,
> for pse_quantity::ids::QuantityKindId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityKindId,
        AdapterError,
    > {
        layout(
            <pse_quantity::ids::QuantityKindId as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::QuantityKindId as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let (field_0,) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityKindId {
            id: Some(native_into::<_, Vec<u8>>(field_0, &child_path(path, "id"))?),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityKindId,
> for pse_quantity::ids::QuantityKindId {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityKindId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::QuantityKindId as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::QuantityKindId as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                Vec<u8>,
            >(required(wire.id, &child_path(path, "id"))?, &child_path(path, "id"))?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityType,
> for pse_quantity::quantity_type::QuantityType {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityType,
        AdapterError,
    > {
        layout(
            <pse_quantity::quantity_type::QuantityType as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::quantity_type::QuantityType as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
        let (field_0, field_1, field_2, field_3, field_4) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityType {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityTypeId,
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
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityTypeKey,
                >(field_2, &child_path(path, "key"))?,
            ),
            canonical_unit: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::UnitId,
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
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityType,
> for pse_quantity::quantity_type::QuantityType {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityType,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::quantity_type::QuantityType as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::quantity_type::QuantityType as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityTypeId,
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
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityTypeKey,
            >(
                required(wire.key.into_option(), &child_path(path, "key"))?,
                &child_path(path, "key"),
            )?,
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::UnitId,
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
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityTypeId,
> for pse_quantity::ids::QuantityTypeId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityTypeId,
        AdapterError,
    > {
        layout(
            <pse_quantity::ids::QuantityTypeId as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::QuantityTypeId as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let (field_0,) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityTypeId {
            id: Some(native_into::<_, Vec<u8>>(field_0, &child_path(path, "id"))?),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityTypeId,
> for pse_quantity::ids::QuantityTypeId {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityTypeId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::QuantityTypeId as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::QuantityTypeId as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                Vec<u8>,
            >(required(wire.id, &child_path(path, "id"))?, &child_path(path, "id"))?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityTypeKey,
> for pse_quantity::quantity_type::QuantityTypeKey {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityTypeKey,
        AdapterError,
    > {
        layout(
            <pse_quantity::quantity_type::QuantityTypeKey as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::quantity_type::QuantityTypeKey as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
        let (field_0, field_1, field_2, field_3, field_4, field_5) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityTypeKey {
            kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityKindId,
                >(field_0, &child_path(path, "kind"))?,
            ),
            basis: field_1
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::BasisId,
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
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceStateId,
                        >(value, &child_path(path, "reference_state"))?,
                    )
                })
                .transpose()?
                .into(),
            scale_kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ScaleKind,
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
                                crate::host_operations::engineering::generated::pb::pse::engineering::v1::EntityKindId,
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
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::EntityKindId,
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
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityTypeKey,
> for pse_quantity::quantity_type::QuantityTypeKey {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityTypeKey,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::quantity_type::QuantityTypeKey as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::quantity_type::QuantityTypeKey as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityKindId,
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
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::BasisId,
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
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceStateId,
                        >(value, &child_path(path, "reference_state"))?,
                    )
                })
                .transpose()?,
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::ScaleKind,
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
                                crate::host_operations::engineering::generated::pb::pse::engineering::v1::EntityKindId,
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
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::EntityKindId,
                        >(value, &child_path(path, "subject_kind"))?,
                    )
                })
                .transpose()?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::RateBasis,
> for pse_quantity::enums::RateBasis {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::RateBasis,
        AdapterError,
    > {
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::RateBasis {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::RateBasis,
> for pse_quantity::enums::RateBasis {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::RateBasis,
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
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceCondition,
> for pse_quantity::reference_state::ReferenceCondition {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceCondition,
        AdapterError,
    > {
        layout(
            <pse_quantity::reference_state::ReferenceCondition as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::reference_state::ReferenceCondition as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value"), (2, "quantity_type"), (3, "unit")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceCondition {
            value: native_into::<_, u64>(field_0, &child_path(path, "value"))?,
            quantity_type: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityTypeId,
                >(field_1, &child_path(path, "quantity_type"))?,
            ),
            unit: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::UnitId,
                >(field_2, &child_path(path, "unit"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceCondition,
> for pse_quantity::reference_state::ReferenceCondition {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceCondition,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::reference_state::ReferenceCondition as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::reference_state::ReferenceCondition as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value"), (2, "quantity_type"), (3, "unit")],
            path,
        )?;
        let fields = (
            native_from::<_, u64>(wire.value, &child_path(path, "value"))?,
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityTypeId,
            >(
                required(
                    wire.quantity_type.into_option(),
                    &child_path(path, "quantity_type"),
                )?,
                &child_path(path, "quantity_type"),
            )?,
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::UnitId,
            >(
                required(wire.unit.into_option(), &child_path(path, "unit"))?,
                &child_path(path, "unit"),
            )?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceState,
> for pse_quantity::reference_state::ReferenceState {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceState,
        AdapterError,
    > {
        layout(
            <pse_quantity::reference_state::ReferenceState as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::reference_state::ReferenceState as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
        let (field_0, field_1, field_2, field_3, field_4, field_5, field_6) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceState {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceStateId,
                >(field_0, &child_path(path, "id"))?,
            ),
            name: native_into::<_, String>(field_1, &child_path(path, "name"))?,
            kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceStateKind,
                >(field_2, &child_path(path, "kind"))?,
            ),
            temperature: field_3
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceCondition,
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
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceCondition,
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
                    Ok(native_into::<_, Vec<u8>>(value, &child_path(path, "subject"))?)
                })
                .transpose()?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceState,
> for pse_quantity::reference_state::ReferenceState {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceState,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::reference_state::ReferenceState as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::reference_state::ReferenceState as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceStateId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<_, String>(wire.name, &child_path(path, "name"))?,
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceStateKind,
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
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceCondition,
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
                            crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceCondition,
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
                    Ok(native_from::<_, Vec<u8>>(value, &child_path(path, "subject"))?)
                })
                .transpose()?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceStateId,
> for pse_quantity::ids::ReferenceStateId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceStateId,
        AdapterError,
    > {
        layout(
            <pse_quantity::ids::ReferenceStateId as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::ReferenceStateId as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let (field_0,) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceStateId {
            id: Some(native_into::<_, Vec<u8>>(field_0, &child_path(path, "id"))?),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceStateId,
> for pse_quantity::ids::ReferenceStateId {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceStateId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::ReferenceStateId as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::ReferenceStateId as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                Vec<u8>,
            >(required(wire.id, &child_path(path, "id"))?, &child_path(path, "id"))?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceStateKind,
> for pse_quantity::enums::ReferenceStateKind {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceStateKind,
        AdapterError,
    > {
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceStateKind {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceStateKind,
> for pse_quantity::enums::ReferenceStateKind {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceStateKind,
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
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ResidentPreparation,
> for crate::workflow::ResidentPreparation {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::ResidentPreparation,
        AdapterError,
    > {
        layout(
            <crate::workflow::ResidentPreparation as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::ResidentPreparation as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "kind"),
                (2, "eligibility"),
                (3, "retained_bytes"),
                (4, "semantic_identity"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::ResidentPreparation {
            kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationProductKind,
                >(field_0, &child_path(path, "kind"))?,
            ),
            eligibility: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationEligibility,
                >(field_1, &child_path(path, "eligibility"))?,
            ),
            retained_bytes: native_into::<
                _,
                u64,
            >(field_2, &child_path(path, "retained_bytes"))?,
            semantic_identity: field_3
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            String,
                        >(value, &child_path(path, "semantic_identity"))?,
                    )
                })
                .transpose()?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ResidentPreparation,
> for crate::workflow::ResidentPreparation {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::ResidentPreparation,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::workflow::ResidentPreparation as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::ResidentPreparation as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "kind"),
                (2, "eligibility"),
                (3, "retained_bytes"),
                (4, "semantic_identity"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationProductKind,
            >(
                required(wire.kind.into_option(), &child_path(path, "kind"))?,
                &child_path(path, "kind"),
            )?,
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationEligibility,
            >(
                required(
                    wire.eligibility.into_option(),
                    &child_path(path, "eligibility"),
                )?,
                &child_path(path, "eligibility"),
            )?,
            native_from::<
                _,
                u64,
            >(wire.retained_bytes, &child_path(path, "retained_bytes"))?,
            wire
                .semantic_identity
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_from::<
                            _,
                            String,
                        >(value, &child_path(path, "semantic_identity"))?,
                    )
                })
                .transpose()?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ScaleKind,
> for pse_quantity::enums::ScaleKind {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::ScaleKind,
        AdapterError,
    > {
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::ScaleKind {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::ScaleKind,
> for pse_quantity::enums::ScaleKind {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::ScaleKind,
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
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue,
> for pse_modeling::specialize::Value {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue,
        AdapterError,
    > {
        let value = self;
        layout(
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Boolean");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Boolean(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::structural_value::Alternative1 {
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
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Integer");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Integer(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::structural_value::Alternative2 {
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
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                3,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                3,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "bits"), (2, "quantity")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
            3,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "Number");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Number(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::structural_value::Alternative3 {
                                bits: native_into::<
                                    _,
                                    u64,
                                >(field_0, &child_path(path, "bits"))?,
                                quantity: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityTypeId,
                                    >(field_1, &child_path(path, "quantity"))?,
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
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                4,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                4,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "bits"), (3, "quantity")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
            4,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1, field_2)) => {
                let alternative_path = child_path(path, "Coordinate");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Coordinate(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::structural_value::Alternative4 {
                                id: native_into::<
                                    _,
                                    Vec<u8>,
                                >(field_0, &child_path(path, "id"))?,
                                bits: native_into::<
                                    _,
                                    u64,
                                >(field_1, &child_path(path, "bits"))?,
                                quantity: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityTypeId,
                                    >(field_2, &child_path(path, "quantity"))?,
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
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                5,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                5,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
            5,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Text");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Text(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::structural_value::Alternative5 {
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
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                6,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                6,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "kind")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
            6,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "Entity");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Entity(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::structural_value::Alternative6 {
                                id: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::DeclarationId,
                                    >(field_0, &child_path(path, "id"))?,
                                ),
                                kind: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::DeclarationId,
                                    >(field_1, &child_path(path, "kind"))?,
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
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                7,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                7,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "enumeration"), (2, "member")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
            7,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "Enum");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Enum(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::structural_value::Alternative7 {
                                enumeration: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::DeclarationId,
                                    >(field_0, &child_path(path, "enumeration"))?,
                                ),
                                member: native_into::<
                                    _,
                                    Vec<u8>,
                                >(field_1, &child_path(path, "member"))?,
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
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                8,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                8,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "scheme"), (2, "value")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
            8,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "Identifier");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Identifier(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::structural_value::Alternative8 {
                                scheme: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::DeclarationId,
                                    >(field_0, &child_path(path, "scheme"))?,
                                ),
                                value: native_into::<
                                    _,
                                    String,
                                >(field_1, &child_path(path, "value"))?,
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
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                9,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                9,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "bindings")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
            9,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "Definition");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Definition(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::structural_value::Alternative9 {
                                id: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::DeclarationId,
                                    >(field_0, &child_path(path, "id"))?,
                                ),
                                bindings: wire_map(
                                    field_1
                                        .into_iter()
                                        .map(|(key, value)| -> Result<_, AdapterError> {
                                            Ok((
                                                native_into::<
                                                    _,
                                                    String,
                                                >(key, &child_path(path, "bindings"))?,
                                                native_into::<
                                                    _,
                                                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue,
                                                >(value, &child_path(path, "bindings"))?,
                                            ))
                                        })
                                        .collect::<Result<Vec<_>, _>>()?
                                        .into_iter(),
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
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                10,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                10,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
            10,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Function");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Function(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::structural_value::Alternative10 {
                                value: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::DeclarationId,
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
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                11,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                11,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "table"), (2, "names"), (3, "fields")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
            11,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1, field_2)) => {
                let alternative_path = child_path(path, "Row");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Row(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::structural_value::Alternative11 {
                                table: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::DeclarationId,
                                    >(field_0, &child_path(path, "table"))?,
                                ),
                                names: {
                                    let current_path = &child_path(path, "names");
                                    items(field_1)
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
                                fields: {
                                    let current_path = &child_path(path, "fields");
                                    items(field_2)
                                        .enumerate()
                                        .map(|(index, value)| -> Result<_, AdapterError> {
                                            Ok(
                                                native_into::<
                                                    _,
                                                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue,
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
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                12,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                12,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
            12,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Set");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Set(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::structural_value::Alternative12 {
                                value: {
                                    let current_path = &child_path(path, "value");
                                    items(field_0)
                                        .enumerate()
                                        .map(|(index, value)| -> Result<_, AdapterError> {
                                            Ok(
                                                native_into::<
                                                    _,
                                                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue,
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
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                13,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                13,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
            13,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Tuple");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Tuple(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::structural_value::Alternative13 {
                                value: {
                                    let current_path = &child_path(path, "value");
                                    items(field_0)
                                        .enumerate()
                                        .map(|(index, value)| -> Result<_, AdapterError> {
                                            Ok(
                                                native_into::<
                                                    _,
                                                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue,
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
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                14,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                14,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
            14,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "QuantityType");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::QuantityType(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::structural_value::Alternative14 {
                                value: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityTypeId,
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
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                15,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                15,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
            15,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "ReferenceState");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::ReferenceState(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::structural_value::Alternative15 {
                                value: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceStateId,
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
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                16,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                16,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
            16,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Missing(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::structural_value::Alternative16 {
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
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue,
> for pse_modeling::specialize::Value {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue,
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
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Boolean(
                wire,
            ) => {
                let alternative_path = child_path(path, "Boolean");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<_, bool>(wire.value, &child_path(path, "value"))?,
                );
                Ok(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Integer(
                wire,
            ) => {
                let alternative_path = child_path(path, "Integer");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<_, i64>(wire.value, &child_path(path, "value"))?,
                );
                Ok(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Number(
                wire,
            ) => {
                let alternative_path = child_path(path, "Number");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "bits"), (2, "quantity")],
                    path,
                )?;
                let fields = (
                    native_from::<_, u64>(wire.bits, &child_path(path, "bits"))?,
                    native_from::<
                        _,
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityTypeId,
                    >(
                        required(
                            wire.quantity.into_option(),
                            &child_path(path, "quantity"),
                        )?,
                        &child_path(path, "quantity"),
                    )?,
                );
                Ok(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Coordinate(
                wire,
            ) => {
                let alternative_path = child_path(path, "Coordinate");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "id"), (2, "bits"), (3, "quantity")],
                    path,
                )?;
                let fields = (
                    native_from::<_, Vec<u8>>(wire.id, &child_path(path, "id"))?,
                    native_from::<_, u64>(wire.bits, &child_path(path, "bits"))?,
                    native_from::<
                        _,
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityTypeId,
                    >(
                        required(
                            wire.quantity.into_option(),
                            &child_path(path, "quantity"),
                        )?,
                        &child_path(path, "quantity"),
                    )?,
                );
                Ok(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Text(
                wire,
            ) => {
                let alternative_path = child_path(path, "Text");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        5,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        5,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<_, String>(wire.value, &child_path(path, "value"))?,
                );
                Ok(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Entity(
                wire,
            ) => {
                let alternative_path = child_path(path, "Entity");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        6,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        6,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "id"), (2, "kind")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::DeclarationId,
                    >(
                        required(wire.id.into_option(), &child_path(path, "id"))?,
                        &child_path(path, "id"),
                    )?,
                    native_from::<
                        _,
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::DeclarationId,
                    >(
                        required(wire.kind.into_option(), &child_path(path, "kind"))?,
                        &child_path(path, "kind"),
                    )?,
                );
                Ok(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        6,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Enum(
                wire,
            ) => {
                let alternative_path = child_path(path, "Enum");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        7,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        7,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "enumeration"), (2, "member")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::DeclarationId,
                    >(
                        required(
                            wire.enumeration.into_option(),
                            &child_path(path, "enumeration"),
                        )?,
                        &child_path(path, "enumeration"),
                    )?,
                    native_from::<_, Vec<u8>>(wire.member, &child_path(path, "member"))?,
                );
                Ok(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        7,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Identifier(
                wire,
            ) => {
                let alternative_path = child_path(path, "Identifier");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        8,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        8,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "scheme"), (2, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::DeclarationId,
                    >(
                        required(
                            wire.scheme.into_option(),
                            &child_path(path, "scheme"),
                        )?,
                        &child_path(path, "scheme"),
                    )?,
                    native_from::<_, String>(wire.value, &child_path(path, "value"))?,
                );
                Ok(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        8,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Definition(
                wire,
            ) => {
                let alternative_path = child_path(path, "Definition");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        9,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        9,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "id"), (2, "bindings")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::DeclarationId,
                    >(
                        required(wire.id.into_option(), &child_path(path, "id"))?,
                        &child_path(path, "id"),
                    )?,
                    unique_pairs(
                        wire
                            .bindings
                            .into_iter()
                            .map(|(key, value)| -> Result<_, AdapterError> {
                                Ok((
                                    native_from::<
                                        _,
                                        String,
                                    >(key, &child_path(path, "bindings"))?,
                                    native_from::<
                                        _,
                                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue,
                                    >(value, &child_path(path, "bindings"))?,
                                ))
                            })
                            .collect::<Result<Vec<_>, _>>()?,
                        &child_path(path, "bindings"),
                    )?,
                );
                Ok(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        9,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Function(
                wire,
            ) => {
                let alternative_path = child_path(path, "Function");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        10,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        10,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::DeclarationId,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        10,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Row(
                wire,
            ) => {
                let alternative_path = child_path(path, "Row");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        11,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        11,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "table"), (2, "names"), (3, "fields")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::DeclarationId,
                    >(
                        required(wire.table.into_option(), &child_path(path, "table"))?,
                        &child_path(path, "table"),
                    )?,
                    {
                        let current_path = &child_path(path, "names");
                        let values = wire
                            .names
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
                    {
                        let current_path = &child_path(path, "fields");
                        let values = wire
                            .fields
                            .into_iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                Ok(
                                    native_from::<
                                        _,
                                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue,
                                    >(value, &format!("{path}[{index}]", path = current_path))?,
                                )
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        11,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Set(
                wire,
            ) => {
                let alternative_path = child_path(path, "Set");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        12,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        12,
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
                                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue,
                                    >(value, &format!("{path}[{index}]", path = current_path))?,
                                )
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        12,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Tuple(
                wire,
            ) => {
                let alternative_path = child_path(path, "Tuple");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        13,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        13,
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
                                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue,
                                    >(value, &format!("{path}[{index}]", path = current_path))?,
                                )
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        13,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::QuantityType(
                wire,
            ) => {
                let alternative_path = child_path(path, "QuantityType");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        14,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        14,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::QuantityTypeId,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        14,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::ReferenceState(
                wire,
            ) => {
                let alternative_path = child_path(path, "ReferenceState");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        15,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        15,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::ReferenceStateId,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        15,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::structural_value::Kind::Missing(
                wire,
            ) => {
                let alternative_path = child_path(path, "Missing");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        16,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        16,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::specialize::Value as pse_model::boundary_projection::ProjectionAlternative<
                        16,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::UnitFactor,
> for pse_quantity::unit::UnitFactor {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::UnitFactor,
        AdapterError,
    > {
        layout(
            <pse_quantity::unit::UnitFactor as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::unit::UnitFactor as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "unit"), (2, "exponent")],
            path,
        )?;
        let (field_0, field_1) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::UnitFactor {
            unit: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::UnitId,
                >(field_0, &child_path(path, "unit"))?,
            ),
            exponent: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalRatio,
                >(field_1, &child_path(path, "exponent"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::UnitFactor,
> for pse_quantity::unit::UnitFactor {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::UnitFactor,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::unit::UnitFactor as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::unit::UnitFactor as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "unit"), (2, "exponent")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::UnitId,
            >(
                required(wire.unit.into_option(), &child_path(path, "unit"))?,
                &child_path(path, "unit"),
            )?,
            native_from::<
                _,
                crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalRatio,
            >(
                required(wire.exponent.into_option(), &child_path(path, "exponent"))?,
                &child_path(path, "exponent"),
            )?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::UnitId,
> for pse_quantity::ids::UnitId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::UnitId,
        AdapterError,
    > {
        layout(
            <pse_quantity::ids::UnitId as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::UnitId as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let (field_0,) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::UnitId {
            id: Some(native_into::<_, Vec<u8>>(field_0, &child_path(path, "id"))?),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::UnitId,
> for pse_quantity::ids::UnitId {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::UnitId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::UnitId as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::UnitId as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                Vec<u8>,
            >(required(wire.id, &child_path(path, "id"))?, &child_path(path, "id"))?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
/// Encode the actual native `PreparationDiscoveryRequest` and validate outgoing local rules.
pub fn encode_preparation_discovery_request(
    native: crate::workflow::PreparationDiscoveryRequest,
) -> Result<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationDiscoveryRequest,
    BoundaryProjectionError,
> {
    encode_root(native, "PreparationDiscoveryRequest")
}
/// Adopt known `PreparationDiscoveryRequest` meaning under this package's declared unknown-field policy.
pub fn decode_preparation_discovery_request(
    wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationDiscoveryRequest,
) -> Result<crate::workflow::PreparationDiscoveryRequest, BoundaryProjectionError> {
    decode_root(wire, "PreparationDiscoveryRequest")
}
/// Encode the actual native `PreparationDiscoveryReport` and validate outgoing local rules.
pub fn encode_preparation_discovery_report(
    native: crate::workflow::PreparationDiscoveryReport,
) -> Result<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationDiscoveryReport,
    BoundaryProjectionError,
> {
    encode_root(native, "PreparationDiscoveryReport")
}
/// Adopt known `PreparationDiscoveryReport` meaning under this package's declared unknown-field policy.
pub fn decode_preparation_discovery_report(
    wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::PreparationDiscoveryReport,
) -> Result<crate::workflow::PreparationDiscoveryReport, BoundaryProjectionError> {
    decode_root(wire, "PreparationDiscoveryReport")
}
/// Encode the actual native `StructuralValue` and validate outgoing local rules.
pub fn encode_structural_value(
    native: pse_modeling::specialize::Value,
) -> Result<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue,
    BoundaryProjectionError,
> {
    encode_root(native, "StructuralValue")
}
/// Adopt known `StructuralValue` meaning under this package's declared unknown-field policy.
pub fn decode_structural_value(
    wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::StructuralValue,
) -> Result<pse_modeling::specialize::Value, BoundaryProjectionError> {
    decode_root(wire, "StructuralValue")
}
/// Encode the actual native `OriginalSourceFileRequest` and validate outgoing local rules.
pub fn encode_original_source_file_request(
    native: crate::workflow::OriginalSourceFileRequest,
) -> Result<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::OriginalSourceFileRequest,
    BoundaryProjectionError,
> {
    encode_root(native, "OriginalSourceFileRequest")
}
/// Adopt known `OriginalSourceFileRequest` meaning under this package's declared unknown-field policy.
pub fn decode_original_source_file_request(
    wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::OriginalSourceFileRequest,
) -> Result<crate::workflow::OriginalSourceFileRequest, BoundaryProjectionError> {
    decode_root(wire, "OriginalSourceFileRequest")
}
/// Encode the actual native `OriginalSourceFile` and validate outgoing local rules.
pub fn encode_original_source_file(
    native: crate::workflow::OriginalSourceFile,
) -> Result<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::OriginalSourceFile,
    BoundaryProjectionError,
> {
    encode_root(native, "OriginalSourceFile")
}
/// Adopt known `OriginalSourceFile` meaning under this package's declared unknown-field policy.
pub fn decode_original_source_file(
    wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::OriginalSourceFile,
) -> Result<crate::workflow::OriginalSourceFile, BoundaryProjectionError> {
    decode_root(wire, "OriginalSourceFile")
}
/// Encode the actual native `PhysicalInspectionRequest` and validate outgoing local rules.
pub fn encode_physical_inspection_request(
    native: crate::workflow::PhysicalInspectionRequest,
) -> Result<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalInspectionRequest,
    BoundaryProjectionError,
> {
    encode_root(native, "PhysicalInspectionRequest")
}
/// Adopt known `PhysicalInspectionRequest` meaning under this package's declared unknown-field policy.
pub fn decode_physical_inspection_request(
    wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalInspectionRequest,
) -> Result<crate::workflow::PhysicalInspectionRequest, BoundaryProjectionError> {
    decode_root(wire, "PhysicalInspectionRequest")
}
/// Encode the actual native `PhysicalInspectionDocument` and validate outgoing local rules.
pub fn encode_physical_inspection_document(
    native: crate::workflow::PhysicalInspectionDocument,
) -> Result<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalInspectionDocument,
    BoundaryProjectionError,
> {
    encode_root(native, "PhysicalInspectionDocument")
}
/// Adopt known `PhysicalInspectionDocument` meaning under this package's declared unknown-field policy.
pub fn decode_physical_inspection_document(
    wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::PhysicalInspectionDocument,
) -> Result<crate::workflow::PhysicalInspectionDocument, BoundaryProjectionError> {
    decode_root(wire, "PhysicalInspectionDocument")
}
