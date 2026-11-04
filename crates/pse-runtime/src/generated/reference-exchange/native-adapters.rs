use super::{
    AdapterError, AdapterFailure, BoundaryProjectionError, NativeInto, NativeFrom,
    layout, child_path, refused, required, native_into, native_from, encode_root,
    decode_root, items, from_items,
};
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
