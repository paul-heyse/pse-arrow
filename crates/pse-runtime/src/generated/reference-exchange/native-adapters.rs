use super::{
    AdapterError, AdapterFailure, BoundaryProjectionError, NativeInto, NativeFrom,
    layout, child_path, refused, required, native_into, native_from, encode_root,
    decode_root, items, from_items,
};
impl NativeInto<
    crate::reference_exchange::generated::pb::pse::reference::v1::Array8PhysicalRatio,
> for [pse_quantity::dimension::Ratio; 8] {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::Array8PhysicalRatio,
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
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::Array8PhysicalRatio {
            values: {
                let current_path = &child_path(path, "values");
                items(field_0)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalRatio,
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
    crate::reference_exchange::generated::pb::pse::reference::v1::Array8PhysicalRatio,
> for [pse_quantity::dimension::Ratio; 8] {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::Array8PhysicalRatio,
        path: &str,
    ) -> Result<Self, AdapterError> {
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
                                crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalRatio,
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
impl NativeInto<crate::reference_exchange::generated::pb::pse::reference::v1::BasisId>
for pse_quantity::ids::BasisId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::BasisId,
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
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::BasisId {
            id: Some(native_into::<_, Vec<u8>>(field_0, &child_path(path, "id"))?),
            ..Default::default()
        })
    }
}
impl NativeFrom<crate::reference_exchange::generated::pb::pse::reference::v1::BasisId>
for pse_quantity::ids::BasisId {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::BasisId,
        path: &str,
    ) -> Result<Self, AdapterError> {
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
impl NativeInto<crate::reference_exchange::generated::pb::pse::reference::v1::BasisKind>
for pse_quantity::enums::BasisKind {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::BasisKind,
        AdapterError,
    > {
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::BasisKind {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<crate::reference_exchange::generated::pb::pse::reference::v1::BasisKind>
for pse_quantity::enums::BasisKind {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::BasisKind,
        path: &str,
    ) -> Result<Self, AdapterError> {
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
    crate::reference_exchange::generated::pb::pse::reference::v1::CompositionBasis,
> for pse_quantity::enums::CompositionBasis {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::CompositionBasis,
        AdapterError,
    > {
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::CompositionBasis {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::reference_exchange::generated::pb::pse::reference::v1::CompositionBasis,
> for pse_quantity::enums::CompositionBasis {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::CompositionBasis,
        path: &str,
    ) -> Result<Self, AdapterError> {
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
    crate::reference_exchange::generated::pb::pse::reference::v1::DocumentVersion1,
> for pse_model::document::Version<1> {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::DocumentVersion1,
        AdapterError,
    > {
        layout(
            <pse_model::document::Version<
                1,
            > as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::document::Version<
                1,
            > as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let (field_0,) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::DocumentVersion1 {
            value: native_into::<_, u32>(field_0, &child_path(path, "value"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::reference_exchange::generated::pb::pse::reference::v1::DocumentVersion1,
> for pse_model::document::Version<1> {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::DocumentVersion1,
        path: &str,
    ) -> Result<Self, AdapterError> {
        layout(
            <pse_model::document::Version<
                1,
            > as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::document::Version<
                1,
            > as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let fields = (native_from::<_, u32>(wire.value, &child_path(path, "value"))?,);
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::reference_exchange::generated::pb::pse::reference::v1::EntityKindId,
> for pse_quantity::ids::EntityKindId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::EntityKindId,
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
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::EntityKindId {
            id: Some(native_into::<_, Vec<u8>>(field_0, &child_path(path, "id"))?),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::reference_exchange::generated::pb::pse::reference::v1::EntityKindId,
> for pse_quantity::ids::EntityKindId {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::EntityKindId,
        path: &str,
    ) -> Result<Self, AdapterError> {
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
    crate::reference_exchange::generated::pb::pse::reference::v1::KindDefinition,
> for pse_quantity::kind::KindDefinition {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::KindDefinition,
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
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::KindDefinition {
            monomial: {
                let current_path = &child_path(path, "monomial");
                items(field_0)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::reference_exchange::generated::pb::pse::reference::v1::KindFactor,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            canonical_unit: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::UnitId,
                >(field_1, &child_path(path, "canonical_unit"))?,
            ),
            basis: field_2
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::reference_exchange::generated::pb::pse::reference::v1::BasisId,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceStateId,
                        >(value, &child_path(path, "reference_state"))?,
                    )
                })
                .transpose()?
                .into(),
            scale_kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::ScaleKind,
                >(field_4, &child_path(path, "scale_kind"))?,
            ),
            subject_kind: field_5
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::reference_exchange::generated::pb::pse::reference::v1::EntityKindId,
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
    crate::reference_exchange::generated::pb::pse::reference::v1::KindDefinition,
> for pse_quantity::kind::KindDefinition {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::KindDefinition,
        path: &str,
    ) -> Result<Self, AdapterError> {
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
                                crate::reference_exchange::generated::pb::pse::reference::v1::KindFactor,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                crate::reference_exchange::generated::pb::pse::reference::v1::UnitId,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::BasisId,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceStateId,
                        >(value, &child_path(path, "reference_state"))?,
                    )
                })
                .transpose()?,
            native_from::<
                _,
                crate::reference_exchange::generated::pb::pse::reference::v1::ScaleKind,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::EntityKindId,
                        >(value, &child_path(path, "subject_kind"))?,
                    )
                })
                .transpose()?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<crate::reference_exchange::generated::pb::pse::reference::v1::KindFactor>
for pse_quantity::kind::KindFactor {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::KindFactor,
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
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::KindFactor {
            kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::QuantityKindId,
                >(field_0, &child_path(path, "kind"))?,
            ),
            exponent: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalRatio,
                >(field_1, &child_path(path, "exponent"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<crate::reference_exchange::generated::pb::pse::reference::v1::KindFactor>
for pse_quantity::kind::KindFactor {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::KindFactor,
        path: &str,
    ) -> Result<Self, AdapterError> {
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
                crate::reference_exchange::generated::pb::pse::reference::v1::QuantityKindId,
            >(
                required(wire.kind.into_option(), &child_path(path, "kind"))?,
                &child_path(path, "kind"),
            )?,
            native_from::<
                _,
                crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalRatio,
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
    crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalBasis,
> for pse_quantity::basis::Basis {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalBasis,
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
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalBasis {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::BasisId,
                >(field_0, &child_path(path, "id"))?,
            ),
            kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::BasisKind,
                >(field_1, &child_path(path, "kind"))?,
            ),
            composition_basis: field_2
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::reference_exchange::generated::pb::pse::reference::v1::CompositionBasis,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::RateBasis,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceStateId,
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
    crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalBasis,
> for pse_quantity::basis::Basis {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalBasis,
        path: &str,
    ) -> Result<Self, AdapterError> {
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
                crate::reference_exchange::generated::pb::pse::reference::v1::BasisId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<
                _,
                crate::reference_exchange::generated::pb::pse::reference::v1::BasisKind,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::CompositionBasis,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::RateBasis,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceStateId,
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
    crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalDimensionVector,
> for pse_quantity::dimension::DimensionVector {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalDimensionVector,
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
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalDimensionVector {
            exponents: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::Array8PhysicalRatio,
                >(field_0, &child_path(path, "exponents"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalDimensionVector,
> for pse_quantity::dimension::DimensionVector {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalDimensionVector,
        path: &str,
    ) -> Result<Self, AdapterError> {
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
                crate::reference_exchange::generated::pb::pse::reference::v1::Array8PhysicalRatio,
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
    crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalEntityKind,
> for pse_quantity::entity_kind::EntityKind {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalEntityKind,
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
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalEntityKind {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::EntityKindId,
                >(field_0, &child_path(path, "id"))?,
            ),
            name: native_into::<_, String>(field_1, &child_path(path, "name"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalEntityKind,
> for pse_quantity::entity_kind::EntityKind {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalEntityKind,
        path: &str,
    ) -> Result<Self, AdapterError> {
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
                crate::reference_exchange::generated::pb::pse::reference::v1::EntityKindId,
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
    crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalQuantityDescription,
> for crate::workflow::PhysicalQuantityDescription {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalQuantityDescription,
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
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalQuantityDescription {
            quantity: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::QuantityType,
                >(field_0, &child_path(path, "quantity"))?,
            ),
            quantity_kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalQuantityKind,
                >(field_1, &child_path(path, "quantity_kind"))?,
            ),
            canonical_unit: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalUnit,
                >(field_2, &child_path(path, "canonical_unit"))?,
            ),
            representation_unit: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalUnit,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalBasis,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceState,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceState,
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
                                crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalEntityKind,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalEntityKind,
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
    crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalQuantityDescription,
> for crate::workflow::PhysicalQuantityDescription {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalQuantityDescription,
        path: &str,
    ) -> Result<Self, AdapterError> {
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
                crate::reference_exchange::generated::pb::pse::reference::v1::QuantityType,
            >(
                required(wire.quantity.into_option(), &child_path(path, "quantity"))?,
                &child_path(path, "quantity"),
            )?,
            native_from::<
                _,
                crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalQuantityKind,
            >(
                required(
                    wire.quantity_kind.into_option(),
                    &child_path(path, "quantity_kind"),
                )?,
                &child_path(path, "quantity_kind"),
            )?,
            native_from::<
                _,
                crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalUnit,
            >(
                required(
                    wire.canonical_unit.into_option(),
                    &child_path(path, "canonical_unit"),
                )?,
                &child_path(path, "canonical_unit"),
            )?,
            native_from::<
                _,
                crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalUnit,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalBasis,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceState,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceState,
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
                                crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalEntityKind,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalEntityKind,
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
    crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalQuantityKind,
> for pse_quantity::kind::QuantityKind {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalQuantityKind,
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
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalQuantityKind {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::QuantityKindId,
                >(field_0, &child_path(path, "id"))?,
            ),
            dimension: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalDimensionVector,
                >(field_1, &child_path(path, "dimension"))?,
            ),
            extensive: native_into::<_, bool>(field_2, &child_path(path, "extensive"))?,
            addition_kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::QuantityAdditionKind,
                >(field_3, &child_path(path, "addition_kind"))?,
            ),
            category: field_4
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::reference_exchange::generated::pb::pse::reference::v1::QuantityKindCategory,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::KindDefinition,
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
    crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalQuantityKind,
> for pse_quantity::kind::QuantityKind {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalQuantityKind,
        path: &str,
    ) -> Result<Self, AdapterError> {
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
                crate::reference_exchange::generated::pb::pse::reference::v1::QuantityKindId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<
                _,
                crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalDimensionVector,
            >(
                required(wire.dimension.into_option(), &child_path(path, "dimension"))?,
                &child_path(path, "dimension"),
            )?,
            native_from::<_, bool>(wire.extensive, &child_path(path, "extensive"))?,
            native_from::<
                _,
                crate::reference_exchange::generated::pb::pse::reference::v1::QuantityAdditionKind,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::QuantityKindCategory,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::KindDefinition,
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
    crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalRatio,
> for pse_quantity::dimension::Ratio {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalRatio,
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
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalRatio {
            num: native_into::<_, i32>(field_0, &child_path(path, "num"))?,
            den: native_into::<_, i32>(field_1, &child_path(path, "den"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalRatio,
> for pse_quantity::dimension::Ratio {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalRatio,
        path: &str,
    ) -> Result<Self, AdapterError> {
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
    crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalUnit,
> for pse_quantity::unit::Unit {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalUnit,
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
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalUnit {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::UnitId,
                >(field_0, &child_path(path, "id"))?,
            ),
            symbol: native_into::<_, String>(field_1, &child_path(path, "symbol"))?,
            dimension: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalDimensionVector,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceStateId,
                        >(value, &child_path(path, "reference_state"))?,
                    )
                })
                .transpose()?
                .into(),
            definition: field_7
                .map(|value| -> Result<_, AdapterError> {
                    Ok(crate::reference_exchange::generated::pb::pse::reference::v1::physical_unit::Field8Value {
                        value: {
                            let current_path = &child_path(path, "definition");
                            items(value)
                                .enumerate()
                                .map(|(index, value)| -> Result<_, AdapterError> {
                                    Ok(
                                        native_into::<
                                            _,
                                            crate::reference_exchange::generated::pb::pse::reference::v1::UnitFactor,
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
    crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalUnit,
> for pse_quantity::unit::Unit {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalUnit,
        path: &str,
    ) -> Result<Self, AdapterError> {
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
                crate::reference_exchange::generated::pb::pse::reference::v1::UnitId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<_, String>(wire.symbol, &child_path(path, "symbol"))?,
            native_from::<
                _,
                crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalDimensionVector,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceStateId,
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
                                            crate::reference_exchange::generated::pb::pse::reference::v1::UnitFactor,
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
    crate::reference_exchange::generated::pb::pse::reference::v1::QuantityAdditionKind,
> for pse_quantity::enums::QuantityAdditionKind {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::QuantityAdditionKind,
        AdapterError,
    > {
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::QuantityAdditionKind {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::reference_exchange::generated::pb::pse::reference::v1::QuantityAdditionKind,
> for pse_quantity::enums::QuantityAdditionKind {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::QuantityAdditionKind,
        path: &str,
    ) -> Result<Self, AdapterError> {
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
    crate::reference_exchange::generated::pb::pse::reference::v1::QuantityKindCategory,
> for pse_quantity::enums::QuantityKindCategory {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::QuantityKindCategory,
        AdapterError,
    > {
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::QuantityKindCategory {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::reference_exchange::generated::pb::pse::reference::v1::QuantityKindCategory,
> for pse_quantity::enums::QuantityKindCategory {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::QuantityKindCategory,
        path: &str,
    ) -> Result<Self, AdapterError> {
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
    crate::reference_exchange::generated::pb::pse::reference::v1::QuantityKindId,
> for pse_quantity::ids::QuantityKindId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::QuantityKindId,
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
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::QuantityKindId {
            id: Some(native_into::<_, Vec<u8>>(field_0, &child_path(path, "id"))?),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::reference_exchange::generated::pb::pse::reference::v1::QuantityKindId,
> for pse_quantity::ids::QuantityKindId {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::QuantityKindId,
        path: &str,
    ) -> Result<Self, AdapterError> {
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
    crate::reference_exchange::generated::pb::pse::reference::v1::QuantityType,
> for pse_quantity::quantity_type::QuantityType {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::QuantityType,
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
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::QuantityType {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::QuantityTypeId,
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
                    crate::reference_exchange::generated::pb::pse::reference::v1::QuantityTypeKey,
                >(field_2, &child_path(path, "key"))?,
            ),
            canonical_unit: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::UnitId,
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
    crate::reference_exchange::generated::pb::pse::reference::v1::QuantityType,
> for pse_quantity::quantity_type::QuantityType {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::QuantityType,
        path: &str,
    ) -> Result<Self, AdapterError> {
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
                crate::reference_exchange::generated::pb::pse::reference::v1::QuantityTypeId,
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
                crate::reference_exchange::generated::pb::pse::reference::v1::QuantityTypeKey,
            >(
                required(wire.key.into_option(), &child_path(path, "key"))?,
                &child_path(path, "key"),
            )?,
            native_from::<
                _,
                crate::reference_exchange::generated::pb::pse::reference::v1::UnitId,
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
    crate::reference_exchange::generated::pb::pse::reference::v1::QuantityTypeId,
> for pse_quantity::ids::QuantityTypeId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::QuantityTypeId,
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
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::QuantityTypeId {
            id: Some(native_into::<_, Vec<u8>>(field_0, &child_path(path, "id"))?),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::reference_exchange::generated::pb::pse::reference::v1::QuantityTypeId,
> for pse_quantity::ids::QuantityTypeId {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::QuantityTypeId,
        path: &str,
    ) -> Result<Self, AdapterError> {
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
    crate::reference_exchange::generated::pb::pse::reference::v1::QuantityTypeKey,
> for pse_quantity::quantity_type::QuantityTypeKey {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::QuantityTypeKey,
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
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::QuantityTypeKey {
            kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::QuantityKindId,
                >(field_0, &child_path(path, "kind"))?,
            ),
            basis: field_1
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::reference_exchange::generated::pb::pse::reference::v1::BasisId,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceStateId,
                        >(value, &child_path(path, "reference_state"))?,
                    )
                })
                .transpose()?
                .into(),
            scale_kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::ScaleKind,
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
                                crate::reference_exchange::generated::pb::pse::reference::v1::EntityKindId,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::EntityKindId,
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
    crate::reference_exchange::generated::pb::pse::reference::v1::QuantityTypeKey,
> for pse_quantity::quantity_type::QuantityTypeKey {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::QuantityTypeKey,
        path: &str,
    ) -> Result<Self, AdapterError> {
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
                crate::reference_exchange::generated::pb::pse::reference::v1::QuantityKindId,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::BasisId,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceStateId,
                        >(value, &child_path(path, "reference_state"))?,
                    )
                })
                .transpose()?,
            native_from::<
                _,
                crate::reference_exchange::generated::pb::pse::reference::v1::ScaleKind,
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
                                crate::reference_exchange::generated::pb::pse::reference::v1::EntityKindId,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::EntityKindId,
                        >(value, &child_path(path, "subject_kind"))?,
                    )
                })
                .transpose()?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<crate::reference_exchange::generated::pb::pse::reference::v1::RateBasis>
for pse_quantity::enums::RateBasis {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::RateBasis,
        AdapterError,
    > {
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::RateBasis {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<crate::reference_exchange::generated::pb::pse::reference::v1::RateBasis>
for pse_quantity::enums::RateBasis {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::RateBasis,
        path: &str,
    ) -> Result<Self, AdapterError> {
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
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceBlock,
> for pse_model::reference_exchange::ReferenceBlock {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceBlock,
        AdapterError,
    > {
        layout(
            <pse_model::reference_exchange::ReferenceBlock as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::reference_exchange::ReferenceBlock as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "content"), (2, "schema"), (3, "rows")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceBlock {
            content: native_into::<_, Vec<u8>>(field_0, &child_path(path, "content"))?,
            schema: native_into::<_, Vec<u8>>(field_1, &child_path(path, "schema"))?,
            rows: native_into::<_, u64>(field_2, &child_path(path, "rows"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceBlock,
> for pse_model::reference_exchange::ReferenceBlock {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceBlock,
        path: &str,
    ) -> Result<Self, AdapterError> {
        layout(
            <pse_model::reference_exchange::ReferenceBlock as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::reference_exchange::ReferenceBlock as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "content"), (2, "schema"), (3, "rows")],
            path,
        )?;
        let fields = (
            native_from::<_, Vec<u8>>(wire.content, &child_path(path, "content"))?,
            native_from::<_, Vec<u8>>(wire.schema, &child_path(path, "schema"))?,
            native_from::<_, u64>(wire.rows, &child_path(path, "rows"))?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceCaseContract,
> for crate::reference_exchange::ReferenceCaseContract {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceCaseContract,
        AdapterError,
    > {
        layout(
            <crate::reference_exchange::ReferenceCaseContract as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <crate::reference_exchange::ReferenceCaseContract as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "version"),
                (2, "case_name"),
                (3, "meaning"),
                (4, "observables"),
                (5, "specification"),
                (6, "artifacts"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceCaseContract {
            version: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::DocumentVersion1,
                >(field_0, &child_path(path, "version"))?,
            ),
            case_name: native_into::<
                _,
                String,
            >(field_1, &child_path(path, "case_name"))?,
            meaning: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceMeaning,
                >(field_2, &child_path(path, "meaning"))?,
            ),
            observables: {
                let current_path = &child_path(path, "observables");
                items(field_3)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceObservableContract,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            specification: {
                let current_path = &child_path(path, "specification");
                items(field_4)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceFixedInput,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            artifacts: {
                let current_path = &child_path(path, "artifacts");
                items(field_5)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceInputArtifact,
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
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceCaseContract,
> for crate::reference_exchange::ReferenceCaseContract {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceCaseContract,
        path: &str,
    ) -> Result<Self, AdapterError> {
        layout(
            <crate::reference_exchange::ReferenceCaseContract as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <crate::reference_exchange::ReferenceCaseContract as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "version"),
                (2, "case_name"),
                (3, "meaning"),
                (4, "observables"),
                (5, "specification"),
                (6, "artifacts"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::reference_exchange::generated::pb::pse::reference::v1::DocumentVersion1,
            >(
                required(wire.version.into_option(), &child_path(path, "version"))?,
                &child_path(path, "version"),
            )?,
            native_from::<_, String>(wire.case_name, &child_path(path, "case_name"))?,
            native_from::<
                _,
                crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceMeaning,
            >(
                required(wire.meaning.into_option(), &child_path(path, "meaning"))?,
                &child_path(path, "meaning"),
            )?,
            {
                let current_path = &child_path(path, "observables");
                let values = wire
                    .observables
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceObservableContract,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "specification");
                let values = wire
                    .specification
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceFixedInput,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "artifacts");
                let values = wire
                    .artifacts
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceInputArtifact,
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
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceCondition,
> for pse_quantity::reference_state::ReferenceCondition {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceCondition,
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
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceCondition {
            value: native_into::<_, u64>(field_0, &child_path(path, "value"))?,
            quantity_type: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::QuantityTypeId,
                >(field_1, &child_path(path, "quantity_type"))?,
            ),
            unit: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::UnitId,
                >(field_2, &child_path(path, "unit"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceCondition,
> for pse_quantity::reference_state::ReferenceCondition {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceCondition,
        path: &str,
    ) -> Result<Self, AdapterError> {
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
                crate::reference_exchange::generated::pb::pse::reference::v1::QuantityTypeId,
            >(
                required(
                    wire.quantity_type.into_option(),
                    &child_path(path, "quantity_type"),
                )?,
                &child_path(path, "quantity_type"),
            )?,
            native_from::<
                _,
                crate::reference_exchange::generated::pb::pse::reference::v1::UnitId,
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
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceEnvelope,
> for pse_model::reference_exchange::ReferenceEnvelope {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceEnvelope,
        AdapterError,
    > {
        layout(
            <pse_model::reference_exchange::ReferenceEnvelope as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::reference_exchange::ReferenceEnvelope as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "version"), (2, "meaning"), (3, "required"), (4, "outcome")],
            path,
        )?;
        let (field_0, field_1, field_2, field_3) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceEnvelope {
            version: native_into::<_, u32>(field_0, &child_path(path, "version"))?,
            meaning: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceMeaning,
                >(field_1, &child_path(path, "meaning"))?,
            ),
            required: {
                let current_path = &child_path(path, "required");
                items(field_2)
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
            outcome: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceOutcome,
                >(field_3, &child_path(path, "outcome"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceEnvelope,
> for pse_model::reference_exchange::ReferenceEnvelope {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceEnvelope,
        path: &str,
    ) -> Result<Self, AdapterError> {
        layout(
            <pse_model::reference_exchange::ReferenceEnvelope as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::reference_exchange::ReferenceEnvelope as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "version"), (2, "meaning"), (3, "required"), (4, "outcome")],
            path,
        )?;
        let fields = (
            native_from::<_, u32>(wire.version, &child_path(path, "version"))?,
            native_from::<
                _,
                crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceMeaning,
            >(
                required(wire.meaning.into_option(), &child_path(path, "meaning"))?,
                &child_path(path, "meaning"),
            )?,
            {
                let current_path = &child_path(path, "required");
                let values = wire
                    .required
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
            native_from::<
                _,
                crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceOutcome,
            >(
                required(wire.outcome.into_option(), &child_path(path, "outcome"))?,
                &child_path(path, "outcome"),
            )?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceFixedInput,
> for crate::reference_exchange::ReferenceFixedInput {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceFixedInput,
        AdapterError,
    > {
        layout(
            <crate::reference_exchange::ReferenceFixedInput as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <crate::reference_exchange::ReferenceFixedInput as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "key"), (2, "value"), (3, "physical")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceFixedInput {
            key: native_into::<_, String>(field_0, &child_path(path, "key"))?,
            value: native_into::<_, u64>(field_1, &child_path(path, "value"))?,
            physical: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalQuantityDescription,
                >(field_2, &child_path(path, "physical"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceFixedInput,
> for crate::reference_exchange::ReferenceFixedInput {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceFixedInput,
        path: &str,
    ) -> Result<Self, AdapterError> {
        layout(
            <crate::reference_exchange::ReferenceFixedInput as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <crate::reference_exchange::ReferenceFixedInput as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "key"), (2, "value"), (3, "physical")],
            path,
        )?;
        let fields = (
            native_from::<_, String>(wire.key, &child_path(path, "key"))?,
            native_from::<_, u64>(wire.value, &child_path(path, "value"))?,
            native_from::<
                _,
                crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalQuantityDescription,
            >(
                required(wire.physical.into_option(), &child_path(path, "physical"))?,
                &child_path(path, "physical"),
            )?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceInputArtifact,
> for crate::reference_exchange::ReferenceInputArtifact {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceInputArtifact,
        AdapterError,
    > {
        layout(
            <crate::reference_exchange::ReferenceInputArtifact as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <crate::reference_exchange::ReferenceInputArtifact as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "path"), (2, "sha256")],
            path,
        )?;
        let (field_0, field_1) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceInputArtifact {
            path: native_into::<_, String>(field_0, &child_path(path, "path"))?,
            sha256: native_into::<_, Vec<u8>>(field_1, &child_path(path, "sha256"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceInputArtifact,
> for crate::reference_exchange::ReferenceInputArtifact {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceInputArtifact,
        path: &str,
    ) -> Result<Self, AdapterError> {
        layout(
            <crate::reference_exchange::ReferenceInputArtifact as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <crate::reference_exchange::ReferenceInputArtifact as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "path"), (2, "sha256")],
            path,
        )?;
        let fields = (
            native_from::<_, String>(wire.path, &child_path(path, "path"))?,
            native_from::<_, Vec<u8>>(wire.sha256, &child_path(path, "sha256"))?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceMeaning,
> for pse_model::reference_exchange::ReferenceMeaning {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceMeaning,
        AdapterError,
    > {
        layout(
            <pse_model::reference_exchange::ReferenceMeaning as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::reference_exchange::ReferenceMeaning as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "case"),
                (2, "source"),
                (3, "formulation"),
                (4, "physical"),
                (5, "configuration"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceMeaning {
            case: native_into::<_, Vec<u8>>(field_0, &child_path(path, "case"))?,
            source: native_into::<_, Vec<u8>>(field_1, &child_path(path, "source"))?,
            formulation: native_into::<
                _,
                Vec<u8>,
            >(field_2, &child_path(path, "formulation"))?,
            physical: native_into::<_, Vec<u8>>(field_3, &child_path(path, "physical"))?,
            configuration: native_into::<
                _,
                Vec<u8>,
            >(field_4, &child_path(path, "configuration"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceMeaning,
> for pse_model::reference_exchange::ReferenceMeaning {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceMeaning,
        path: &str,
    ) -> Result<Self, AdapterError> {
        layout(
            <pse_model::reference_exchange::ReferenceMeaning as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::reference_exchange::ReferenceMeaning as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "case"),
                (2, "source"),
                (3, "formulation"),
                (4, "physical"),
                (5, "configuration"),
            ],
            path,
        )?;
        let fields = (
            native_from::<_, Vec<u8>>(wire.case, &child_path(path, "case"))?,
            native_from::<_, Vec<u8>>(wire.source, &child_path(path, "source"))?,
            native_from::<
                _,
                Vec<u8>,
            >(wire.formulation, &child_path(path, "formulation"))?,
            native_from::<_, Vec<u8>>(wire.physical, &child_path(path, "physical"))?,
            native_from::<
                _,
                Vec<u8>,
            >(wire.configuration, &child_path(path, "configuration"))?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceObservable,
> for pse_model::reference_exchange::ReferenceObservable {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceObservable,
        AdapterError,
    > {
        layout(
            <pse_model::reference_exchange::ReferenceObservable as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::reference_exchange::ReferenceObservable as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "id"),
                (2, "quantity"),
                (3, "unit"),
                (4, "basis"),
                (5, "datum"),
                (6, "value"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceObservable {
            id: native_into::<_, Vec<u8>>(field_0, &child_path(path, "id"))?,
            quantity: native_into::<_, Vec<u8>>(field_1, &child_path(path, "quantity"))?,
            unit: native_into::<_, Vec<u8>>(field_2, &child_path(path, "unit"))?,
            basis: field_3
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_into::<_, Vec<u8>>(value, &child_path(path, "basis"))?)
                })
                .transpose()?,
            datum: field_4
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_into::<_, Vec<u8>>(value, &child_path(path, "datum"))?)
                })
                .transpose()?,
            value: native_into::<_, u64>(field_5, &child_path(path, "value"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceObservable,
> for pse_model::reference_exchange::ReferenceObservable {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceObservable,
        path: &str,
    ) -> Result<Self, AdapterError> {
        layout(
            <pse_model::reference_exchange::ReferenceObservable as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::reference_exchange::ReferenceObservable as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "id"),
                (2, "quantity"),
                (3, "unit"),
                (4, "basis"),
                (5, "datum"),
                (6, "value"),
            ],
            path,
        )?;
        let fields = (
            native_from::<_, Vec<u8>>(wire.id, &child_path(path, "id"))?,
            native_from::<_, Vec<u8>>(wire.quantity, &child_path(path, "quantity"))?,
            native_from::<_, Vec<u8>>(wire.unit, &child_path(path, "unit"))?,
            wire
                .basis
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_from::<_, Vec<u8>>(value, &child_path(path, "basis"))?)
                })
                .transpose()?,
            wire
                .datum
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_from::<_, Vec<u8>>(value, &child_path(path, "datum"))?)
                })
                .transpose()?,
            native_from::<_, u64>(wire.value, &child_path(path, "value"))?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceObservableContract,
> for crate::reference_exchange::ReferenceObservableContract {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceObservableContract,
        AdapterError,
    > {
        layout(
            <crate::reference_exchange::ReferenceObservableContract as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <crate::reference_exchange::ReferenceObservableContract as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "output"), (2, "key"), (3, "physical")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_model::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceObservableContract {
            output: native_into::<_, Vec<u8>>(field_0, &child_path(path, "output"))?,
            key: native_into::<_, String>(field_1, &child_path(path, "key"))?,
            physical: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalQuantityDescription,
                >(field_2, &child_path(path, "physical"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceObservableContract,
> for crate::reference_exchange::ReferenceObservableContract {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceObservableContract,
        path: &str,
    ) -> Result<Self, AdapterError> {
        layout(
            <crate::reference_exchange::ReferenceObservableContract as pse_model::boundary_projection::ProjectionRecord>::VERSION,
            <crate::reference_exchange::ReferenceObservableContract as pse_model::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "output"), (2, "key"), (3, "physical")],
            path,
        )?;
        let fields = (
            native_from::<_, Vec<u8>>(wire.output, &child_path(path, "output"))?,
            native_from::<_, String>(wire.key, &child_path(path, "key"))?,
            native_from::<
                _,
                crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalQuantityDescription,
            >(
                required(wire.physical.into_option(), &child_path(path, "physical"))?,
                &child_path(path, "physical"),
            )?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceOutcome,
> for pse_model::reference_exchange::ReferenceOutcome {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceOutcome,
        AdapterError,
    > {
        let value = self;
        layout(
            <pse_model::reference_exchange::ReferenceOutcome as pse_model::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <pse_model::reference_exchange::ReferenceOutcome as pse_model::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "outputs"), (2, "blocks")],
            path,
        )?;
        let value = match <pse_model::reference_exchange::ReferenceOutcome as pse_model::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "Completed");
                let path = alternative_path.as_str();
                return Ok(crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceOutcome {
                    kind: Some(
                        crate::reference_exchange::generated::pb::pse::reference::v1::__buffa::oneof::reference_outcome::Kind::Completed(
                            Box::new(crate::reference_exchange::generated::pb::pse::reference::v1::reference_outcome::Alternative1 {
                                outputs: {
                                    let current_path = &child_path(path, "outputs");
                                    items(field_0)
                                        .enumerate()
                                        .map(|(index, value)| -> Result<_, AdapterError> {
                                            Ok(
                                                native_into::<
                                                    _,
                                                    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceObservable,
                                                >(value, &format!("{path}[{index}]", path = current_path))?,
                                            )
                                        })
                                        .collect::<Result<Vec<_>, _>>()?
                                },
                                blocks: {
                                    let current_path = &child_path(path, "blocks");
                                    items(field_1)
                                        .enumerate()
                                        .map(|(index, value)| -> Result<_, AdapterError> {
                                            Ok(
                                                native_into::<
                                                    _,
                                                    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceBlock,
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
            <pse_model::reference_exchange::ReferenceOutcome as pse_model::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <pse_model::reference_exchange::ReferenceOutcome as pse_model::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "code"), (2, "stage")],
            path,
        )?;
        let value = match <pse_model::reference_exchange::ReferenceOutcome as pse_model::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "Failed");
                let path = alternative_path.as_str();
                return Ok(crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceOutcome {
                    kind: Some(
                        crate::reference_exchange::generated::pb::pse::reference::v1::__buffa::oneof::reference_outcome::Kind::Failed(
                            Box::new(crate::reference_exchange::generated::pb::pse::reference::v1::reference_outcome::Alternative2 {
                                code: native_into::<
                                    _,
                                    String,
                                >(field_0, &child_path(path, "code"))?,
                                stage: native_into::<
                                    _,
                                    String,
                                >(field_1, &child_path(path, "stage"))?,
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
            <pse_model::reference_exchange::ReferenceOutcome as pse_model::boundary_projection::ProjectionAlternative<
                3,
            >>::VERSION,
            <pse_model::reference_exchange::ReferenceOutcome as pse_model::boundary_projection::ProjectionAlternative<
                3,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_model::reference_exchange::ReferenceOutcome as pse_model::boundary_projection::ProjectionAlternative<
            3,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Unavailable");
                let path = alternative_path.as_str();
                return Ok(crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceOutcome {
                    kind: Some(
                        crate::reference_exchange::generated::pb::pse::reference::v1::__buffa::oneof::reference_outcome::Kind::Unavailable(
                            Box::new(crate::reference_exchange::generated::pb::pse::reference::v1::reference_outcome::Alternative3 {
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
            <pse_model::reference_exchange::ReferenceOutcome as pse_model::boundary_projection::ProjectionAlternative<
                4,
            >>::VERSION,
            <pse_model::reference_exchange::ReferenceOutcome as pse_model::boundary_projection::ProjectionAlternative<
                4,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_model::reference_exchange::ReferenceOutcome as pse_model::boundary_projection::ProjectionAlternative<
            4,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "NotRun");
                let path = alternative_path.as_str();
                return Ok(crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceOutcome {
                    kind: Some(
                        crate::reference_exchange::generated::pb::pse::reference::v1::__buffa::oneof::reference_outcome::Kind::NotRun(
                            Box::new(crate::reference_exchange::generated::pb::pse::reference::v1::reference_outcome::Alternative4 {
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
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceOutcome,
> for pse_model::reference_exchange::ReferenceOutcome {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceOutcome,
        path: &str,
    ) -> Result<Self, AdapterError> {
        match wire
            .kind
            .ok_or_else(|| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "required alternative is absent",
            ))?
        {
            crate::reference_exchange::generated::pb::pse::reference::v1::__buffa::oneof::reference_outcome::Kind::Completed(
                wire,
            ) => {
                let alternative_path = child_path(path, "Completed");
                let path = alternative_path.as_str();
                let wire = *wire;
                layout(
                    <pse_model::reference_exchange::ReferenceOutcome as pse_model::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_model::reference_exchange::ReferenceOutcome as pse_model::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "outputs"), (2, "blocks")],
                    path,
                )?;
                let fields = (
                    {
                        let current_path = &child_path(path, "outputs");
                        let values = wire
                            .outputs
                            .into_iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                Ok(
                                    native_from::<
                                        _,
                                        crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceObservable,
                                    >(value, &format!("{path}[{index}]", path = current_path))?,
                                )
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                    {
                        let current_path = &child_path(path, "blocks");
                        let values = wire
                            .blocks
                            .into_iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                Ok(
                                    native_from::<
                                        _,
                                        crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceBlock,
                                    >(value, &format!("{path}[{index}]", path = current_path))?,
                                )
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_model::reference_exchange::ReferenceOutcome as pse_model::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::reference_exchange::generated::pb::pse::reference::v1::__buffa::oneof::reference_outcome::Kind::Failed(
                wire,
            ) => {
                let alternative_path = child_path(path, "Failed");
                let path = alternative_path.as_str();
                let wire = *wire;
                layout(
                    <pse_model::reference_exchange::ReferenceOutcome as pse_model::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_model::reference_exchange::ReferenceOutcome as pse_model::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "code"), (2, "stage")],
                    path,
                )?;
                let fields = (
                    native_from::<_, String>(wire.code, &child_path(path, "code"))?,
                    native_from::<_, String>(wire.stage, &child_path(path, "stage"))?,
                );
                Ok(
                    <pse_model::reference_exchange::ReferenceOutcome as pse_model::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::reference_exchange::generated::pb::pse::reference::v1::__buffa::oneof::reference_outcome::Kind::Unavailable(
                wire,
            ) => {
                let alternative_path = child_path(path, "Unavailable");
                let path = alternative_path.as_str();
                let wire = *wire;
                layout(
                    <pse_model::reference_exchange::ReferenceOutcome as pse_model::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_model::reference_exchange::ReferenceOutcome as pse_model::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<_, String>(wire.value, &child_path(path, "value"))?,
                );
                Ok(
                    <pse_model::reference_exchange::ReferenceOutcome as pse_model::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::reference_exchange::generated::pb::pse::reference::v1::__buffa::oneof::reference_outcome::Kind::NotRun(
                wire,
            ) => {
                let alternative_path = child_path(path, "NotRun");
                let path = alternative_path.as_str();
                let wire = *wire;
                layout(
                    <pse_model::reference_exchange::ReferenceOutcome as pse_model::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <pse_model::reference_exchange::ReferenceOutcome as pse_model::boundary_projection::ProjectionAlternative<
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
                    <pse_model::reference_exchange::ReferenceOutcome as pse_model::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceState,
> for pse_quantity::reference_state::ReferenceState {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceState,
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
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceState {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceStateId,
                >(field_0, &child_path(path, "id"))?,
            ),
            name: native_into::<_, String>(field_1, &child_path(path, "name"))?,
            kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceStateKind,
                >(field_2, &child_path(path, "kind"))?,
            ),
            temperature: field_3
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceCondition,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceCondition,
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
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceState,
> for pse_quantity::reference_state::ReferenceState {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceState,
        path: &str,
    ) -> Result<Self, AdapterError> {
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
                crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceStateId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<_, String>(wire.name, &child_path(path, "name"))?,
            native_from::<
                _,
                crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceStateKind,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceCondition,
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
                            crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceCondition,
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
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceStateId,
> for pse_quantity::ids::ReferenceStateId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceStateId,
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
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceStateId {
            id: Some(native_into::<_, Vec<u8>>(field_0, &child_path(path, "id"))?),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceStateId,
> for pse_quantity::ids::ReferenceStateId {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceStateId,
        path: &str,
    ) -> Result<Self, AdapterError> {
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
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceStateKind,
> for pse_quantity::enums::ReferenceStateKind {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceStateKind,
        AdapterError,
    > {
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceStateKind {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceStateKind,
> for pse_quantity::enums::ReferenceStateKind {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceStateKind,
        path: &str,
    ) -> Result<Self, AdapterError> {
        wire.value
            .parse::<Self>()
            .map_err(|_| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "unknown canonical source vocabulary member",
            ))
    }
}
impl NativeInto<crate::reference_exchange::generated::pb::pse::reference::v1::ScaleKind>
for pse_quantity::enums::ScaleKind {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::ScaleKind,
        AdapterError,
    > {
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::ScaleKind {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<crate::reference_exchange::generated::pb::pse::reference::v1::ScaleKind>
for pse_quantity::enums::ScaleKind {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::ScaleKind,
        path: &str,
    ) -> Result<Self, AdapterError> {
        wire.value
            .parse::<Self>()
            .map_err(|_| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "unknown canonical source vocabulary member",
            ))
    }
}
impl NativeInto<crate::reference_exchange::generated::pb::pse::reference::v1::UnitFactor>
for pse_quantity::unit::UnitFactor {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::UnitFactor,
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
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::UnitFactor {
            unit: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::UnitId,
                >(field_0, &child_path(path, "unit"))?,
            ),
            exponent: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalRatio,
                >(field_1, &child_path(path, "exponent"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<crate::reference_exchange::generated::pb::pse::reference::v1::UnitFactor>
for pse_quantity::unit::UnitFactor {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::UnitFactor,
        path: &str,
    ) -> Result<Self, AdapterError> {
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
                crate::reference_exchange::generated::pb::pse::reference::v1::UnitId,
            >(
                required(wire.unit.into_option(), &child_path(path, "unit"))?,
                &child_path(path, "unit"),
            )?,
            native_from::<
                _,
                crate::reference_exchange::generated::pb::pse::reference::v1::PhysicalRatio,
            >(
                required(wire.exponent.into_option(), &child_path(path, "exponent"))?,
                &child_path(path, "exponent"),
            )?,
        );
        pse_model::boundary_projection::ProjectionRecord::from_projection_fields(fields)
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<crate::reference_exchange::generated::pb::pse::reference::v1::UnitId>
for pse_quantity::ids::UnitId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::reference_exchange::generated::pb::pse::reference::v1::UnitId,
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
        Ok(crate::reference_exchange::generated::pb::pse::reference::v1::UnitId {
            id: Some(native_into::<_, Vec<u8>>(field_0, &child_path(path, "id"))?),
            ..Default::default()
        })
    }
}
impl NativeFrom<crate::reference_exchange::generated::pb::pse::reference::v1::UnitId>
for pse_quantity::ids::UnitId {
    fn from_wire(
        wire: crate::reference_exchange::generated::pb::pse::reference::v1::UnitId,
        path: &str,
    ) -> Result<Self, AdapterError> {
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
/// Encode the actual native `ReferenceEnvelope` and validate outgoing local rules.
pub(super) fn encode_reference_envelope(
    native: pse_model::reference_exchange::ReferenceEnvelope,
) -> Result<
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceEnvelope,
    BoundaryProjectionError,
> {
    encode_root(native, "ReferenceEnvelope")
}
/// Adopt known `ReferenceEnvelope` meaning under this package's declared unknown-field policy.
pub(super) fn decode_reference_envelope(
    wire: crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceEnvelope,
) -> Result<pse_model::reference_exchange::ReferenceEnvelope, BoundaryProjectionError> {
    decode_root(wire, "ReferenceEnvelope")
}
/// Encode the actual native `ReferenceCaseContract` and validate outgoing local rules.
pub(super) fn encode_reference_case_contract(
    native: crate::reference_exchange::ReferenceCaseContract,
) -> Result<
    crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceCaseContract,
    BoundaryProjectionError,
> {
    encode_root(native, "ReferenceCaseContract")
}
/// Adopt known `ReferenceCaseContract` meaning under this package's declared unknown-field policy.
pub(super) fn decode_reference_case_contract(
    wire: crate::reference_exchange::generated::pb::pse::reference::v1::ReferenceCaseContract,
) -> Result<crate::reference_exchange::ReferenceCaseContract, BoundaryProjectionError> {
    decode_root(wire, "ReferenceCaseContract")
}
