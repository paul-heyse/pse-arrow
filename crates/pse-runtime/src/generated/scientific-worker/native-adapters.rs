use super::{
    AdapterError, AdapterFailure, BoundaryProjectionError, NativeInto, NativeFrom,
    layout, child_path, refused, required, unknown_fields, native_into, native_from,
    encode_root, decode_root, items, from_items,
};
impl NativeInto<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Array8OptionalUnitId,
> for [Option<pse_quantity::ids::UnitId>; 8] {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Array8OptionalUnitId,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Array8OptionalUnitId {
            values: {
                let current_path = &child_path(path, "values");
                items(field_0)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::array8optional_unit_id::Field1Value {
                            value: value
                                .map(|value| -> Result<_, AdapterError> {
                                    Ok(
                                        native_into::<
                                            _,
                                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitId,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Array8OptionalUnitId,
> for [Option<pse_quantity::ids::UnitId>; 8] {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Array8OptionalUnitId,
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
                                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitId,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Array8PhysicalRatio,
> for [pse_quantity::dimension::Ratio; 8] {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Array8PhysicalRatio,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Array8PhysicalRatio {
            values: {
                let current_path = &child_path(path, "values");
                items(field_0)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalRatio,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Array8PhysicalRatio,
> for [pse_quantity::dimension::Ratio; 8] {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Array8PhysicalRatio,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalRatio,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisId,
> for pse_quantity::ids::BasisId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisId,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisId {
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisId,
> for pse_quantity::ids::BasisId {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisId,
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
impl NativeInto<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisKind,
> for pse_quantity::enums::BasisKind {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisKind,
        AdapterError,
    > {
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisKind {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisKind,
> for pse_quantity::enums::BasisKind {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisKind,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisRule,
> for pse_quantity::enums::BasisRule {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisRule,
        AdapterError,
    > {
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisRule {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisRule,
> for pse_quantity::enums::BasisRule {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisRule,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Batch,
> for crate::scientific_worker::protocol::Batch {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Batch,
        AdapterError,
    > {
        layout(
            <crate::scientific_worker::protocol::Batch as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::scientific_worker::protocol::Batch as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "handle"), (2, "invocation"), (3, "points")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Batch {
            handle: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Handle,
                >(field_0, &child_path(path, "handle"))?,
            ),
            invocation: native_into::<_, u64>(field_1, &child_path(path, "invocation"))?,
            points: {
                let current_path = &child_path(path, "points");
                items(field_2)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchOutcome,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Batch,
> for crate::scientific_worker::protocol::Batch {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Batch,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::scientific_worker::protocol::Batch as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::scientific_worker::protocol::Batch as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "handle"), (2, "invocation"), (3, "points")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Handle,
            >(
                required(wire.handle.into_option(), &child_path(path, "handle"))?,
                &child_path(path, "handle"),
            )?,
            native_from::<_, u64>(wire.invocation, &child_path(path, "invocation"))?,
            {
                let current_path = &child_path(path, "points");
                let values = wire
                    .points
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchOutcome,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchOutcome,
> for crate::scientific_worker::protocol::BatchOutcome {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchOutcome,
        AdapterError,
    > {
        layout(
            <crate::scientific_worker::protocol::BatchOutcome as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::scientific_worker::protocol::BatchOutcome as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "result")],
            path,
        )?;
        let (field_0, field_1) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchOutcome {
            id: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_0,
            ),
            result: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchResult,
                >(field_1, &child_path(path, "result"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchOutcome,
> for crate::scientific_worker::protocol::BatchOutcome {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchOutcome,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::scientific_worker::protocol::BatchOutcome as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::scientific_worker::protocol::BatchOutcome as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "result")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.id,
                &child_path(path, "id"),
            )?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchResult,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchPoint,
> for crate::scientific_worker::protocol::BatchPoint {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchPoint,
        AdapterError,
    > {
        layout(
            <crate::scientific_worker::protocol::BatchPoint as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::scientific_worker::protocol::BatchPoint as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "id"),
                (2, "inputs"),
                (3, "directions"),
                (4, "cotangents"),
                (5, "weights"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchPoint {
            id: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_0,
            ),
            inputs: {
                let current_path = &child_path(path, "inputs");
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
            directions: {
                let current_path = &child_path(path, "directions");
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
            cotangents: {
                let current_path = &child_path(path, "cotangents");
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
            weights: {
                let current_path = &child_path(path, "weights");
                items(field_4)
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
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchPoint,
> for crate::scientific_worker::protocol::BatchPoint {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchPoint,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::scientific_worker::protocol::BatchPoint as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::scientific_worker::protocol::BatchPoint as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "id"),
                (2, "inputs"),
                (3, "directions"),
                (4, "cotangents"),
                (5, "weights"),
            ],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.id,
                &child_path(path, "id"),
            )?,
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
                                u64,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "directions");
                let values = wire
                    .directions
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
                let current_path = &child_path(path, "cotangents");
                let values = wire
                    .cotangents
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
                let current_path = &child_path(path, "weights");
                let values = wire
                    .weights
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchProduct,
> for crate::scientific_worker::protocol::BatchProduct {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchProduct,
        AdapterError,
    > {
        layout(
            <crate::scientific_worker::protocol::BatchProduct as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::scientific_worker::protocol::BatchProduct as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "values"), (2, "jacobian"), (3, "hessians")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchProduct {
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
            jacobian: {
                let current_path = &child_path(path, "jacobian");
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
            hessians: {
                let current_path = &child_path(path, "hessians");
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
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchProduct,
> for crate::scientific_worker::protocol::BatchProduct {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchProduct,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::scientific_worker::protocol::BatchProduct as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::scientific_worker::protocol::BatchProduct as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "values"), (2, "jacobian"), (3, "hessians")],
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
            {
                let current_path = &child_path(path, "jacobian");
                let values = wire
                    .jacobian
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
                let current_path = &child_path(path, "hessians");
                let values = wire
                    .hessians
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchResult,
> for crate::scientific_worker::protocol::BatchResult {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchResult,
        AdapterError,
    > {
        let value = self;
        layout(
            <crate::scientific_worker::protocol::BatchResult as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <crate::scientific_worker::protocol::BatchResult as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::BatchResult as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Product");
                let path = alternative_path.as_str();
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchResult {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::batch_result::Kind::Product(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::batch_result::Alternative1 {
                                value: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchProduct,
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
            <crate::scientific_worker::protocol::BatchResult as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <crate::scientific_worker::protocol::BatchResult as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::BatchResult as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Refused");
                let path = alternative_path.as_str();
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchResult {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::batch_result::Kind::Refused(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::batch_result::Alternative2 {
                                value: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Failure,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchResult,
> for crate::scientific_worker::protocol::BatchResult {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchResult,
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
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::batch_result::Kind::Product(
                wire,
            ) => {
                let alternative_path = child_path(path, "Product");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::BatchResult as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::BatchResult as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchProduct,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <crate::scientific_worker::protocol::BatchResult as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::batch_result::Kind::Refused(
                wire,
            ) => {
                let alternative_path = child_path(path, "Refused");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::BatchResult as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::BatchResult as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Failure,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <crate::scientific_worker::protocol::BatchResult as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::CompositionBasis,
> for pse_quantity::enums::CompositionBasis {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::CompositionBasis,
        AdapterError,
    > {
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::CompositionBasis {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::CompositionBasis,
> for pse_quantity::enums::CompositionBasis {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::CompositionBasis,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ConversionId,
> for pse_quantity::ids::ConversionId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ConversionId,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ConversionId {
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ConversionId,
> for pse_quantity::ids::ConversionId {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ConversionId,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ConversionKind,
> for pse_quantity::enums::ConversionKind {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ConversionKind,
        AdapterError,
    > {
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ConversionKind {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ConversionKind,
> for pse_quantity::enums::ConversionKind {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ConversionKind,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::DeclaredDefinedUnit,
> for pse_quantity::DefinedUnit {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::DeclaredDefinedUnit,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::DeclaredDefinedUnit {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitId,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitFactor,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::DeclaredDefinedUnit,
> for pse_quantity::DefinedUnit {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::DeclaredDefinedUnit,
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
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitId,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitFactor,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::DeclaredDerivedKind,
> for pse_quantity::DerivedKind {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::DeclaredDerivedKind,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::DeclaredDerivedKind {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityKindId,
                >(field_0, &child_path(path, "id"))?,
            ),
            extensive: native_into::<_, bool>(field_1, &child_path(path, "extensive"))?,
            addition_kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityAdditionKind,
                >(field_2, &child_path(path, "addition_kind"))?,
            ),
            definition: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KindDefinition,
                >(field_3, &child_path(path, "definition"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::DeclaredDerivedKind,
> for pse_quantity::DerivedKind {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::DeclaredDerivedKind,
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
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityKindId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<_, bool>(wire.extensive, &child_path(path, "extensive"))?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityAdditionKind,
            >(
                required(
                    wire.addition_kind.into_option(),
                    &child_path(path, "addition_kind"),
                )?,
                &child_path(path, "addition_kind"),
            )?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KindDefinition,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Demand,
> for crate::scientific_worker::protocol::Demand {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Demand,
        AdapterError,
    > {
        let value = self;
        layout(
            <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Demand {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::demand::Kind::CompleteJets(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::demand::Alternative1 {
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
            <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "SelectedColumns");
                let path = alternative_path.as_str();
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Demand {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::demand::Kind::SelectedColumns(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::demand::Alternative2 {
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
            <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::VERSION,
            <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
            3,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "JacobianAction");
                let path = alternative_path.as_str();
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Demand {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::demand::Kind::JacobianAction(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::demand::Alternative3 {
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
            <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::VERSION,
            <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
            4,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "TransposeAction");
                let path = alternative_path.as_str();
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Demand {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::demand::Kind::TransposeAction(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::demand::Alternative4 {
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
            <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
                5,
            >>::VERSION,
            <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
                5,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
            5,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "WeightedSecondAction");
                let path = alternative_path.as_str();
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Demand {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::demand::Kind::WeightedSecondAction(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::demand::Alternative5 {
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Demand,
> for crate::scientific_worker::protocol::Demand {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Demand,
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
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::demand::Kind::CompleteJets(
                wire,
            ) => {
                let alternative_path = child_path(path, "CompleteJets");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::demand::Kind::SelectedColumns(
                wire,
            ) => {
                let alternative_path = child_path(path, "SelectedColumns");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
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
                                        u64,
                                    >(value, &format!("{path}[{index}]", path = current_path))?,
                                )
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::demand::Kind::JacobianAction(
                wire,
            ) => {
                let alternative_path = child_path(path, "JacobianAction");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<_, u64>(wire.value, &child_path(path, "value"))?,
                );
                Ok(
                    <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::demand::Kind::TransposeAction(
                wire,
            ) => {
                let alternative_path = child_path(path, "TransposeAction");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<_, u64>(wire.value, &child_path(path, "value"))?,
                );
                Ok(
                    <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::demand::Kind::WeightedSecondAction(
                wire,
            ) => {
                let alternative_path = child_path(path, "WeightedSecondAction");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<_, u64>(wire.value, &child_path(path, "value"))?,
                );
                Ok(
                    <crate::scientific_worker::protocol::Demand as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::EntityKindId,
> for pse_quantity::ids::EntityKindId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::EntityKindId,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::EntityKindId {
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::EntityKindId,
> for pse_quantity::ids::EntityKindId {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::EntityKindId,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Evaluate,
> for crate::scientific_worker::protocol::Evaluate {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Evaluate,
        AdapterError,
    > {
        layout(
            <crate::scientific_worker::protocol::Evaluate as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::scientific_worker::protocol::Evaluate as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "handle"),
                (2, "invocation"),
                (3, "inputs"),
                (4, "directions"),
                (5, "cotangents"),
                (6, "weights"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Evaluate {
            handle: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Handle,
                >(field_0, &child_path(path, "handle"))?,
            ),
            invocation: native_into::<_, u64>(field_1, &child_path(path, "invocation"))?,
            inputs: {
                let current_path = &child_path(path, "inputs");
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
            directions: {
                let current_path = &child_path(path, "directions");
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
            cotangents: {
                let current_path = &child_path(path, "cotangents");
                items(field_4)
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
            weights: {
                let current_path = &child_path(path, "weights");
                items(field_5)
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
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Evaluate,
> for crate::scientific_worker::protocol::Evaluate {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Evaluate,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::scientific_worker::protocol::Evaluate as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::scientific_worker::protocol::Evaluate as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "handle"),
                (2, "invocation"),
                (3, "inputs"),
                (4, "directions"),
                (5, "cotangents"),
                (6, "weights"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Handle,
            >(
                required(wire.handle.into_option(), &child_path(path, "handle"))?,
                &child_path(path, "handle"),
            )?,
            native_from::<_, u64>(wire.invocation, &child_path(path, "invocation"))?,
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
                                u64,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "directions");
                let values = wire
                    .directions
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
                let current_path = &child_path(path, "cotangents");
                let values = wire
                    .cotangents
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
                let current_path = &child_path(path, "weights");
                let values = wire
                    .weights
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::EvaluateBatch,
> for crate::scientific_worker::protocol::EvaluateBatch {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::EvaluateBatch,
        AdapterError,
    > {
        layout(
            <crate::scientific_worker::protocol::EvaluateBatch as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::scientific_worker::protocol::EvaluateBatch as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "handle"), (2, "invocation"), (3, "points")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::EvaluateBatch {
            handle: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Handle,
                >(field_0, &child_path(path, "handle"))?,
            ),
            invocation: native_into::<_, u64>(field_1, &child_path(path, "invocation"))?,
            points: {
                let current_path = &child_path(path, "points");
                items(field_2)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchPoint,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::EvaluateBatch,
> for crate::scientific_worker::protocol::EvaluateBatch {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::EvaluateBatch,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::scientific_worker::protocol::EvaluateBatch as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::scientific_worker::protocol::EvaluateBatch as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "handle"), (2, "invocation"), (3, "points")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Handle,
            >(
                required(wire.handle.into_option(), &child_path(path, "handle"))?,
                &child_path(path, "handle"),
            )?,
            native_from::<_, u64>(wire.invocation, &child_path(path, "invocation"))?,
            {
                let current_path = &child_path(path, "points");
                let values = wire
                    .points
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_from::<
                                _,
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BatchPoint,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ExternalDerivativeSource,
> for pse_model::generated::enums::ExternalDerivativeSource {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ExternalDerivativeSource,
        AdapterError,
    > {
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ExternalDerivativeSource {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ExternalDerivativeSource,
> for pse_model::generated::enums::ExternalDerivativeSource {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ExternalDerivativeSource,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Failure,
> for crate::scientific_worker::protocol::Failure {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Failure,
        AdapterError,
    > {
        let value = self;
        layout(
            <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Failure {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::failure::Kind::Cancelled(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::failure::Alternative1 {
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
            <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Failure {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::failure::Kind::Deadline(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::failure::Alternative2 {
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
            <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::VERSION,
            <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
            3,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Resource");
                let path = alternative_path.as_str();
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Failure {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::failure::Kind::Resource(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::failure::Alternative3 {
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
            <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::VERSION,
            <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
            4,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Contract");
                let path = alternative_path.as_str();
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Failure {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::failure::Kind::Contract(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::failure::Alternative4 {
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
            <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                5,
            >>::VERSION,
            <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                5,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
            5,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Trial");
                let path = alternative_path.as_str();
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Failure {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::failure::Kind::Trial(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::failure::Alternative5 {
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
            <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                6,
            >>::VERSION,
            <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                6,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "axis"), (2, "value"), (3, "lower"), (4, "upper")],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
            6,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1, field_2, field_3)) => {
                let alternative_path = child_path(path, "OutsideEnvelope");
                let path = alternative_path.as_str();
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Failure {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::failure::Kind::OutsideEnvelope(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::failure::Alternative6 {
                                axis: native_into::<
                                    _,
                                    String,
                                >(field_0, &child_path(path, "axis"))?,
                                value: native_into::<
                                    _,
                                    u64,
                                >(field_1, &child_path(path, "value"))?,
                                lower: native_into::<
                                    _,
                                    u64,
                                >(field_2, &child_path(path, "lower"))?,
                                upper: native_into::<
                                    _,
                                    u64,
                                >(field_3, &child_path(path, "upper"))?,
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
            <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                7,
            >>::VERSION,
            <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                7,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
            7,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Terminal");
                let path = alternative_path.as_str();
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Failure {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::failure::Kind::Terminal(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::failure::Alternative7 {
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
            <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                8,
            >>::VERSION,
            <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                8,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
            8,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "UnknownCompletion");
                let path = alternative_path.as_str();
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Failure {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::failure::Kind::UnknownCompletion(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::failure::Alternative8 {
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Failure,
> for crate::scientific_worker::protocol::Failure {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Failure,
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
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::failure::Kind::Cancelled(
                wire,
            ) => {
                let alternative_path = child_path(path, "Cancelled");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::failure::Kind::Deadline(
                wire,
            ) => {
                let alternative_path = child_path(path, "Deadline");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::failure::Kind::Resource(
                wire,
            ) => {
                let alternative_path = child_path(path, "Resource");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
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
                    <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::failure::Kind::Contract(
                wire,
            ) => {
                let alternative_path = child_path(path, "Contract");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
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
                    <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::failure::Kind::Trial(
                wire,
            ) => {
                let alternative_path = child_path(path, "Trial");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
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
                    <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::failure::Kind::OutsideEnvelope(
                wire,
            ) => {
                let alternative_path = child_path(path, "OutsideEnvelope");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "axis"), (2, "value"), (3, "lower"), (4, "upper")],
                    path,
                )?;
                let fields = (
                    native_from::<_, String>(wire.axis, &child_path(path, "axis"))?,
                    native_from::<_, u64>(wire.value, &child_path(path, "value"))?,
                    native_from::<_, u64>(wire.lower, &child_path(path, "lower"))?,
                    native_from::<_, u64>(wire.upper, &child_path(path, "upper"))?,
                );
                Ok(
                    <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::failure::Kind::Terminal(
                wire,
            ) => {
                let alternative_path = child_path(path, "Terminal");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
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
                    <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::failure::Kind::UnknownCompletion(
                wire,
            ) => {
                let alternative_path = child_path(path, "UnknownCompletion");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<_, String>(wire.value, &child_path(path, "value"))?,
                );
                Ok(
                    <crate::scientific_worker::protocol::Failure as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Handle,
> for crate::scientific_worker::protocol::Handle {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Handle,
        AdapterError,
    > {
        layout(
            <crate::scientific_worker::protocol::Handle as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::scientific_worker::protocol::Handle as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "attempt"), (2, "session"), (3, "generation")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Handle {
            attempt: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_0,
            ),
            session: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_1,
            ),
            generation: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_2,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Handle,
> for crate::scientific_worker::protocol::Handle {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Handle,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::scientific_worker::protocol::Handle as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::scientific_worker::protocol::Handle as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "attempt"), (2, "session"), (3, "generation")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.attempt,
                &child_path(path, "attempt"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.session,
                &child_path(path, "session"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.generation,
                &child_path(path, "generation"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::InvariantId,
> for pse_quantity::ids::InvariantId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::InvariantId,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::InvariantId {
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::InvariantId,
> for pse_quantity::ids::InvariantId {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::InvariantId,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelDerivativeOrder,
> for pse_kernels::DerivativeOrder {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelDerivativeOrder,
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
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelDerivativeOrder {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::kernel_derivative_order::Kind::Value(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::kernel_derivative_order::Alternative1 {
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
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelDerivativeOrder {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::kernel_derivative_order::Kind::First(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::kernel_derivative_order::Alternative2 {
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
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelDerivativeOrder {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::kernel_derivative_order::Kind::Second(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::kernel_derivative_order::Alternative3 {
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelDerivativeOrder,
> for pse_kernels::DerivativeOrder {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelDerivativeOrder,
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
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::kernel_derivative_order::Kind::Value(
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
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::kernel_derivative_order::Kind::First(
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
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::kernel_derivative_order::Kind::Second(
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderPort,
> for pse_kernels::Port {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderPort,
        AdapterError,
    > {
        layout(
            <pse_kernels::Port as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_kernels::Port as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "quantity"), (3, "unit")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderPort {
            id: Some(
                pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                    field_0,
                ),
            ),
            quantity: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityTypeId,
                >(field_1, &child_path(path, "quantity"))?,
            ),
            unit: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitId,
                >(field_2, &child_path(path, "unit"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderPort,
> for pse_kernels::Port {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderPort,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_kernels::Port as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_kernels::Port as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "quantity"), (3, "unit")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                required(wire.id, &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityTypeId,
            >(
                required(wire.quantity.into_option(), &child_path(path, "quantity"))?,
                &child_path(path, "quantity"),
            )?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitId,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderRecord,
> for pse_kernels::portable::ProviderRecord {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderRecord,
        AdapterError,
    > {
        layout(
            <pse_kernels::portable::ProviderRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_kernels::portable::ProviderRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            2,
            &[(1, "version"), (2, "key"), (3, "spec")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderRecord {
            version: native_into::<_, u32>(field_0, &child_path(path, "version"))?,
            key: Some(
                pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                    field_1,
                ),
            ),
            spec: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderSpec,
                >(field_2, &child_path(path, "spec"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderRecord,
> for pse_kernels::portable::ProviderRecord {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderRecord,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_kernels::portable::ProviderRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_kernels::portable::ProviderRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            2,
            &[(1, "version"), (2, "key"), (3, "spec")],
            path,
        )?;
        let fields = (
            native_from::<_, u32>(wire.version, &child_path(path, "version"))?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                required(wire.key, &child_path(path, "key"))?,
                &child_path(path, "key"),
            )?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderSpec,
            >(
                required(wire.spec.into_option(), &child_path(path, "spec"))?,
                &child_path(path, "spec"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderShape,
> for pse_kernels::ProviderShape {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderShape,
        AdapterError,
    > {
        layout(
            <pse_kernels::ProviderShape as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_kernels::ProviderShape as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "axes"), (3, "coordinates"), (4, "cells")],
            path,
        )?;
        let (field_0, field_1, field_2, field_3) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderShape {
            id: Some(
                pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                    field_0,
                ),
            ),
            axes: items(field_1)
                .map(
                    pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes,
                )
                .collect::<Vec<_>>(),
            coordinates: items(field_2)
                .map(|value| -> Result<_, AdapterError> {
                    Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::kernel_provider_shape::Field3Value {
                        value: items(value)
                            .map(
                                pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes,
                            )
                            .collect::<Vec<_>>(),
                        ..Default::default()
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
            cells: items(field_3)
                .map(
                    pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes,
                )
                .collect::<Vec<_>>(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderShape,
> for pse_kernels::ProviderShape {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderShape,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_kernels::ProviderShape as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_kernels::ProviderShape as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "axes"), (3, "coordinates"), (4, "cells")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                required(wire.id, &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            {
                let current_path = &child_path(path, "axes");
                let values = wire
                    .axes
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
                let current_path = &child_path(path, "coordinates");
                let values = wire
                    .coordinates
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok({
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
                                        Ok(
                                            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                                                value,
                                                &format!("{path}[{index}]", path = current_path),
                                            )?,
                                        )
                                    })
                                    .collect::<Result<Vec<_>, _>>()?;
                                from_items(values, current_path)?
                            }
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "cells");
                let values = wire
                    .cells
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderShapes,
> for pse_kernels::ProviderShapes {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderShapes,
        AdapterError,
    > {
        layout(
            <pse_kernels::ProviderShapes as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_kernels::ProviderShapes as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "inputs"), (2, "outputs")],
            path,
        )?;
        let (field_0, field_1) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderShapes {
            inputs: {
                let current_path = &child_path(path, "inputs");
                items(field_0)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderShape,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            outputs: {
                let current_path = &child_path(path, "outputs");
                items(field_1)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderShape,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderShapes,
> for pse_kernels::ProviderShapes {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderShapes,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_kernels::ProviderShapes as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_kernels::ProviderShapes as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "inputs"), (2, "outputs")],
            path,
        )?;
        let fields = (
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderShape,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
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
                        Ok(
                            native_from::<
                                _,
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderShape,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderSpec,
> for pse_kernels::ProviderSpec {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderSpec,
        AdapterError,
    > {
        layout(
            <pse_kernels::ProviderSpec as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_kernels::ProviderSpec as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "shapes"),
                (2, "derivative_source"),
                (3, "id"),
                (4, "revision"),
                (5, "data"),
                (6, "inputs"),
                (7, "outputs"),
                (8, "derivatives"),
                (9, "smoothness"),
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderSpec {
            shapes: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderShapes,
                >(field_0, &child_path(path, "shapes"))?,
            ),
            derivative_source: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ExternalDerivativeSource,
                >(field_1, &child_path(path, "derivative_source"))?,
            ),
            id: Some(
                pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                    field_2,
                ),
            ),
            revision: Some(
                pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                    field_3,
                ),
            ),
            data: Some(
                pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                    field_4,
                ),
            ),
            inputs: {
                let current_path = &child_path(path, "inputs");
                items(field_5)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderPort,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            outputs: {
                let current_path = &child_path(path, "outputs");
                items(field_6)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderPort,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            derivatives: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelDerivativeOrder,
                >(field_7, &child_path(path, "derivatives"))?,
            ),
            smoothness: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelDerivativeOrder,
                >(field_8, &child_path(path, "smoothness"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderSpec,
> for pse_kernels::ProviderSpec {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderSpec,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_kernels::ProviderSpec as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_kernels::ProviderSpec as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "shapes"),
                (2, "derivative_source"),
                (3, "id"),
                (4, "revision"),
                (5, "data"),
                (6, "inputs"),
                (7, "outputs"),
                (8, "derivatives"),
                (9, "smoothness"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderShapes,
            >(
                required(wire.shapes.into_option(), &child_path(path, "shapes"))?,
                &child_path(path, "shapes"),
            )?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ExternalDerivativeSource,
            >(
                required(
                    wire.derivative_source.into_option(),
                    &child_path(path, "derivative_source"),
                )?,
                &child_path(path, "derivative_source"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                required(wire.id, &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                required(wire.revision, &child_path(path, "revision"))?,
                &child_path(path, "revision"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                required(wire.data, &child_path(path, "data"))?,
                &child_path(path, "data"),
            )?,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderPort,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
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
                        Ok(
                            native_from::<
                                _,
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderPort,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelDerivativeOrder,
            >(
                required(
                    wire.derivatives.into_option(),
                    &child_path(path, "derivatives"),
                )?,
                &child_path(path, "derivatives"),
            )?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelDerivativeOrder,
            >(
                required(
                    wire.smoothness.into_option(),
                    &child_path(path, "smoothness"),
                )?,
                &child_path(path, "smoothness"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KindDefinition,
> for pse_quantity::kind::KindDefinition {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KindDefinition,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KindDefinition {
            monomial: {
                let current_path = &child_path(path, "monomial");
                items(field_0)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KindFactor,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            canonical_unit: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitId,
                >(field_1, &child_path(path, "canonical_unit"))?,
            ),
            basis: field_2
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisId,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceStateId,
                        >(value, &child_path(path, "reference_state"))?,
                    )
                })
                .transpose()?
                .into(),
            scale_kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ScaleKind,
                >(field_4, &child_path(path, "scale_kind"))?,
            ),
            subject_kind: field_5
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::EntityKindId,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KindDefinition,
> for pse_quantity::kind::KindDefinition {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KindDefinition,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KindFactor,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitId,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisId,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceStateId,
                        >(value, &child_path(path, "reference_state"))?,
                    )
                })
                .transpose()?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ScaleKind,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::EntityKindId,
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
impl NativeInto<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KindFactor,
> for pse_quantity::kind::KindFactor {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KindFactor,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KindFactor {
            kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityKindId,
                >(field_0, &child_path(path, "kind"))?,
            ),
            exponent: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalRatio,
                >(field_1, &child_path(path, "exponent"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KindFactor,
> for pse_quantity::kind::KindFactor {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KindFactor,
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
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityKindId,
            >(
                required(wire.kind.into_option(), &child_path(path, "kind"))?,
                &child_path(path, "kind"),
            )?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalRatio,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Opcode,
> for pse_quantity::enums::Opcode {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Opcode,
        AdapterError,
    > {
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Opcode {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Opcode,
> for pse_quantity::enums::Opcode {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Opcode,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::OperationId,
> for pse_quantity::ids::OperationId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::OperationId,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::OperationId {
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::OperationId,
> for pse_quantity::ids::OperationId {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::OperationId,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalBasis,
> for pse_quantity::basis::Basis {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalBasis,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalBasis {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisId,
                >(field_0, &child_path(path, "id"))?,
            ),
            kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisKind,
                >(field_1, &child_path(path, "kind"))?,
            ),
            composition_basis: field_2
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::CompositionBasis,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::RateBasis,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceStateId,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalBasis,
> for pse_quantity::basis::Basis {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalBasis,
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
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisKind,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::CompositionBasis,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::RateBasis,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceStateId,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalConversionDeclaration,
> for pse_quantity::ConversionRule {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalConversionDeclaration,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalConversionDeclaration {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ConversionId,
                >(field_0, &child_path(path, "id"))?,
            ),
            from: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityTypeId,
                >(field_1, &child_path(path, "from"))?,
            ),
            to: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityTypeId,
                >(field_2, &child_path(path, "to"))?,
            ),
            kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ConversionKind,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalConversionDeclaration,
> for pse_quantity::ConversionRule {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalConversionDeclaration,
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
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ConversionId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityTypeId,
            >(
                required(wire.from.into_option(), &child_path(path, "from"))?,
                &child_path(path, "from"),
            )?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityTypeId,
            >(
                required(wire.to.into_option(), &child_path(path, "to"))?,
                &child_path(path, "to"),
            )?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ConversionKind,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalDimensionVector,
> for pse_quantity::dimension::DimensionVector {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalDimensionVector,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalDimensionVector {
            exponents: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Array8PhysicalRatio,
                >(field_0, &child_path(path, "exponents"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalDimensionVector,
> for pse_quantity::dimension::DimensionVector {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalDimensionVector,
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
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Array8PhysicalRatio,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalEntityKind,
> for pse_quantity::entity_kind::EntityKind {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalEntityKind,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalEntityKind {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::EntityKindId,
                >(field_0, &child_path(path, "id"))?,
            ),
            name: native_into::<_, String>(field_1, &child_path(path, "name"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalEntityKind,
> for pse_quantity::entity_kind::EntityKind {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalEntityKind,
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
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::EntityKindId,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalInputConversionDeclaration,
> for pse_quantity::InputConversion {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalInputConversionDeclaration,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalInputConversionDeclaration {
            operand: native_into::<_, u32>(field_0, &child_path(path, "operand"))?,
            conversion: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ConversionId,
                >(field_1, &child_path(path, "conversion"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalInputConversionDeclaration,
> for pse_quantity::InputConversion {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalInputConversionDeclaration,
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
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ConversionId,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalOperationDeclaration,
> for pse_quantity::QuantityOperation {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalOperationDeclaration,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalOperationDeclaration {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::OperationId,
                >(field_0, &child_path(path, "id"))?,
            ),
            opcode: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Opcode,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityKindId,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            result_kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityKindId,
                >(field_3, &child_path(path, "result_kind"))?,
            ),
            basis_rule: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisRule,
                >(field_4, &child_path(path, "basis_rule"))?,
            ),
            reference_rule: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceRule,
                >(field_5, &child_path(path, "reference_rule"))?,
            ),
            scale_rule: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityScaleRule,
                >(field_6, &child_path(path, "scale_rule"))?,
            ),
            shape_rule: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityShapeRule,
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
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::SubjectRule,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::EntityKindId,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisId,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceStateId,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalInputConversionDeclaration,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::InvariantId,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalOperationDeclaration,
> for pse_quantity::QuantityOperation {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalOperationDeclaration,
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
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::OperationId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Opcode,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityKindId,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityKindId,
            >(
                required(
                    wire.result_kind.into_option(),
                    &child_path(path, "result_kind"),
                )?,
                &child_path(path, "result_kind"),
            )?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisRule,
            >(
                required(
                    wire.basis_rule.into_option(),
                    &child_path(path, "basis_rule"),
                )?,
                &child_path(path, "basis_rule"),
            )?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceRule,
            >(
                required(
                    wire.reference_rule.into_option(),
                    &child_path(path, "reference_rule"),
                )?,
                &child_path(path, "reference_rule"),
            )?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityScaleRule,
            >(
                required(
                    wire.scale_rule.into_option(),
                    &child_path(path, "scale_rule"),
                )?,
                &child_path(path, "scale_rule"),
            )?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityShapeRule,
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
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::SubjectRule,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::EntityKindId,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisId,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceStateId,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalInputConversionDeclaration,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::InvariantId,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalQuantityKind,
> for pse_quantity::kind::QuantityKind {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalQuantityKind,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalQuantityKind {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityKindId,
                >(field_0, &child_path(path, "id"))?,
            ),
            dimension: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalDimensionVector,
                >(field_1, &child_path(path, "dimension"))?,
            ),
            extensive: native_into::<_, bool>(field_2, &child_path(path, "extensive"))?,
            addition_kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityAdditionKind,
                >(field_3, &child_path(path, "addition_kind"))?,
            ),
            category: field_4
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityKindCategory,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KindDefinition,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalQuantityKind,
> for pse_quantity::kind::QuantityKind {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalQuantityKind,
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
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityKindId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalDimensionVector,
            >(
                required(wire.dimension.into_option(), &child_path(path, "dimension"))?,
                &child_path(path, "dimension"),
            )?,
            native_from::<_, bool>(wire.extensive, &child_path(path, "extensive"))?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityAdditionKind,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityKindCategory,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KindDefinition,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalRatio,
> for pse_quantity::dimension::Ratio {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalRatio,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalRatio {
            num: native_into::<_, i32>(field_0, &child_path(path, "num"))?,
            den: native_into::<_, i32>(field_1, &child_path(path, "den"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalRatio,
> for pse_quantity::dimension::Ratio {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalRatio,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalRegistryDeclarations,
> for pse_quantity::QuantityRegistryBuilder {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalRegistryDeclarations,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalRegistryDeclarations {
            entity_kinds: {
                let current_path = &child_path(path, "entity_kinds");
                items(field_0)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        Ok(
                            native_into::<
                                _,
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalEntityKind,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalUnit,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::DeclaredDefinedUnit,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalQuantityKind,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::DeclaredDerivedKind,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalBasis,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceState,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityType,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalConversionDeclaration,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalOperationDeclaration,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Tuple2OfOperationIdAndEntityKindId,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalUnitSet,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityTypeId,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalRegistryDeclarations,
> for pse_quantity::QuantityRegistryBuilder {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalRegistryDeclarations,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalEntityKind,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalUnit,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::DeclaredDefinedUnit,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalQuantityKind,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::DeclaredDerivedKind,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalBasis,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceState,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityType,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalConversionDeclaration,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalOperationDeclaration,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Tuple2OfOperationIdAndEntityKindId,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalUnitSet,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityTypeId,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalUnit,
> for pse_quantity::unit::Unit {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalUnit,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalUnit {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitId,
                >(field_0, &child_path(path, "id"))?,
            ),
            symbol: native_into::<_, String>(field_1, &child_path(path, "symbol"))?,
            dimension: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalDimensionVector,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceStateId,
                        >(value, &child_path(path, "reference_state"))?,
                    )
                })
                .transpose()?
                .into(),
            definition: field_7
                .map(|value| -> Result<_, AdapterError> {
                    Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::physical_unit::Field8Value {
                        value: {
                            let current_path = &child_path(path, "definition");
                            items(value)
                                .enumerate()
                                .map(|(index, value)| -> Result<_, AdapterError> {
                                    Ok(
                                        native_into::<
                                            _,
                                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitFactor,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalUnit,
> for pse_quantity::unit::Unit {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalUnit,
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
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<_, String>(wire.symbol, &child_path(path, "symbol"))?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalDimensionVector,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceStateId,
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
                                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitFactor,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalUnitSet,
> for pse_quantity::UnitSet {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalUnitSet,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalUnitSet {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitSetId,
                >(field_0, &child_path(path, "id"))?,
            ),
            base: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Array8OptionalUnitId,
                >(field_1, &child_path(path, "base"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalUnitSet,
> for pse_quantity::UnitSet {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalUnitSet,
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
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitSetId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Array8OptionalUnitId,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Prepare,
> for crate::scientific_worker::protocol::Prepare {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Prepare,
        AdapterError,
    > {
        layout(
            <crate::scientific_worker::protocol::Prepare as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::scientific_worker::protocol::Prepare as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            2,
            &[
                (1, "version"),
                (2, "handle"),
                (3, "spec"),
                (4, "configuration"),
                (6, "regime"),
                (7, "outputs"),
                (8, "order"),
                (9, "demand"),
                (10, "remaining_ns"),
                (11, "result_limit"),
                (12, "registry"),
                (13, "physical"),
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Prepare {
            version: native_into::<_, u32>(field_0, &child_path(path, "version"))?,
            handle: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Handle,
                >(field_1, &child_path(path, "handle"))?,
            ),
            spec: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderRecord,
                >(field_2, &child_path(path, "spec"))?,
            ),
            configuration: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_3,
            ),
            regime: native_into::<_, String>(field_4, &child_path(path, "regime"))?,
            outputs: {
                let current_path = &child_path(path, "outputs");
                items(field_5)
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
            order: native_into::<_, u32>(field_6, &child_path(path, "order"))?,
            demand: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Demand,
                >(field_7, &child_path(path, "demand"))?,
            ),
            remaining_ns: field_8
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_into::<_, u64>(value, &child_path(path, "remaining_ns"))?)
                })
                .transpose()?,
            result_limit: native_into::<
                _,
                u64,
            >(field_9, &child_path(path, "result_limit"))?,
            registry: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalRegistryDeclarations,
                >(field_10, &child_path(path, "registry"))?,
            ),
            physical: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_11,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Prepare,
> for crate::scientific_worker::protocol::Prepare {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Prepare,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::scientific_worker::protocol::Prepare as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::scientific_worker::protocol::Prepare as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            2,
            &[
                (1, "version"),
                (2, "handle"),
                (3, "spec"),
                (4, "configuration"),
                (6, "regime"),
                (7, "outputs"),
                (8, "order"),
                (9, "demand"),
                (10, "remaining_ns"),
                (11, "result_limit"),
                (12, "registry"),
                (13, "physical"),
            ],
            path,
        )?;
        let fields = (
            native_from::<_, u32>(wire.version, &child_path(path, "version"))?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Handle,
            >(
                required(wire.handle.into_option(), &child_path(path, "handle"))?,
                &child_path(path, "handle"),
            )?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::KernelProviderRecord,
            >(
                required(wire.spec.into_option(), &child_path(path, "spec"))?,
                &child_path(path, "spec"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.configuration,
                &child_path(path, "configuration"),
            )?,
            native_from::<_, String>(wire.regime, &child_path(path, "regime"))?,
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
                                u64,
                            >(value, &format!("{path}[{index}]", path = current_path))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<_, u32>(wire.order, &child_path(path, "order"))?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Demand,
            >(
                required(wire.demand.into_option(), &child_path(path, "demand"))?,
                &child_path(path, "demand"),
            )?,
            wire
                .remaining_ns
                .map(|value| -> Result<_, AdapterError> {
                    Ok(native_from::<_, u64>(value, &child_path(path, "remaining_ns"))?)
                })
                .transpose()?,
            native_from::<_, u64>(wire.result_limit, &child_path(path, "result_limit"))?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalRegistryDeclarations,
            >(
                required(wire.registry.into_option(), &child_path(path, "registry"))?,
                &child_path(path, "registry"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.physical,
                &child_path(path, "physical"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Prepared,
> for crate::scientific_worker::protocol::Prepared {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Prepared,
        AdapterError,
    > {
        layout(
            <crate::scientific_worker::protocol::Prepared as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::scientific_worker::protocol::Prepared as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "handle"), (2, "batch_supported")],
            path,
        )?;
        let (field_0, field_1) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Prepared {
            handle: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Handle,
                >(field_0, &child_path(path, "handle"))?,
            ),
            batch_supported: native_into::<
                _,
                bool,
            >(field_1, &child_path(path, "batch_supported"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Prepared,
> for crate::scientific_worker::protocol::Prepared {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Prepared,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::scientific_worker::protocol::Prepared as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::scientific_worker::protocol::Prepared as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "handle"), (2, "batch_supported")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Handle,
            >(
                required(wire.handle.into_option(), &child_path(path, "handle"))?,
                &child_path(path, "handle"),
            )?,
            native_from::<
                _,
                bool,
            >(wire.batch_supported, &child_path(path, "batch_supported"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityAdditionKind,
> for pse_quantity::enums::QuantityAdditionKind {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityAdditionKind,
        AdapterError,
    > {
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityAdditionKind {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityAdditionKind,
> for pse_quantity::enums::QuantityAdditionKind {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityAdditionKind,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityKindCategory,
> for pse_quantity::enums::QuantityKindCategory {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityKindCategory,
        AdapterError,
    > {
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityKindCategory {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityKindCategory,
> for pse_quantity::enums::QuantityKindCategory {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityKindCategory,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityKindId,
> for pse_quantity::ids::QuantityKindId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityKindId,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityKindId {
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityKindId,
> for pse_quantity::ids::QuantityKindId {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityKindId,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityScaleRule,
> for pse_quantity::enums::QuantityScaleRule {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityScaleRule,
        AdapterError,
    > {
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityScaleRule {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityScaleRule,
> for pse_quantity::enums::QuantityScaleRule {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityScaleRule,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityShapeRule,
> for pse_quantity::enums::QuantityShapeRule {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityShapeRule,
        AdapterError,
    > {
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityShapeRule {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityShapeRule,
> for pse_quantity::enums::QuantityShapeRule {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityShapeRule,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityType,
> for pse_quantity::quantity_type::QuantityType {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityType,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityType {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityTypeId,
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
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityTypeKey,
                >(field_2, &child_path(path, "key"))?,
            ),
            canonical_unit: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitId,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityType,
> for pse_quantity::quantity_type::QuantityType {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityType,
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
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityTypeId,
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
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityTypeKey,
            >(
                required(wire.key.into_option(), &child_path(path, "key"))?,
                &child_path(path, "key"),
            )?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitId,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityTypeId,
> for pse_quantity::ids::QuantityTypeId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityTypeId,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityTypeId {
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityTypeId,
> for pse_quantity::ids::QuantityTypeId {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityTypeId,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityTypeKey,
> for pse_quantity::quantity_type::QuantityTypeKey {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityTypeKey,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityTypeKey {
            kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityKindId,
                >(field_0, &child_path(path, "kind"))?,
            ),
            basis: field_1
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisId,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceStateId,
                        >(value, &child_path(path, "reference_state"))?,
                    )
                })
                .transpose()?
                .into(),
            scale_kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ScaleKind,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::EntityKindId,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::EntityKindId,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityTypeKey,
> for pse_quantity::quantity_type::QuantityTypeKey {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityTypeKey,
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
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityKindId,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::BasisId,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceStateId,
                        >(value, &child_path(path, "reference_state"))?,
                    )
                })
                .transpose()?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ScaleKind,
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
                                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::EntityKindId,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::EntityKindId,
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
impl NativeInto<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::RateBasis,
> for pse_quantity::enums::RateBasis {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::RateBasis,
        AdapterError,
    > {
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::RateBasis {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::RateBasis,
> for pse_quantity::enums::RateBasis {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::RateBasis,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceCondition,
> for pse_quantity::reference_state::ReferenceCondition {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceCondition,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceCondition {
            value: native_into::<_, u64>(field_0, &child_path(path, "value"))?,
            quantity_type: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityTypeId,
                >(field_1, &child_path(path, "quantity_type"))?,
            ),
            unit: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitId,
                >(field_2, &child_path(path, "unit"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceCondition,
> for pse_quantity::reference_state::ReferenceCondition {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceCondition,
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
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::QuantityTypeId,
            >(
                required(
                    wire.quantity_type.into_option(),
                    &child_path(path, "quantity_type"),
                )?,
                &child_path(path, "quantity_type"),
            )?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitId,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceRule,
> for pse_quantity::enums::ReferenceRule {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceRule,
        AdapterError,
    > {
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceRule {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceRule,
> for pse_quantity::enums::ReferenceRule {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceRule,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceState,
> for pse_quantity::reference_state::ReferenceState {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceState,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceState {
            id: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceStateId,
                >(field_0, &child_path(path, "id"))?,
            ),
            name: native_into::<_, String>(field_1, &child_path(path, "name"))?,
            kind: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceStateKind,
                >(field_2, &child_path(path, "kind"))?,
            ),
            temperature: field_3
                .map(|value| -> Result<_, AdapterError> {
                    Ok(
                        native_into::<
                            _,
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceCondition,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceCondition,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceState,
> for pse_quantity::reference_state::ReferenceState {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceState,
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
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceStateId,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<_, String>(wire.name, &child_path(path, "name"))?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceStateKind,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceCondition,
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
                            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceCondition,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceStateId,
> for pse_quantity::ids::ReferenceStateId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceStateId,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceStateId {
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceStateId,
> for pse_quantity::ids::ReferenceStateId {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceStateId,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceStateKind,
> for pse_quantity::enums::ReferenceStateKind {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceStateKind,
        AdapterError,
    > {
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceStateKind {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceStateKind,
> for pse_quantity::enums::ReferenceStateKind {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ReferenceStateKind,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Request,
> for crate::scientific_worker::protocol::Request {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Request,
        AdapterError,
    > {
        let value = self;
        layout(
            <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Prepare");
                let path = alternative_path.as_str();
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Request {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::request::Kind::Prepare(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::request::Alternative1 {
                                value: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Prepare,
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
            <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Evaluate");
                let path = alternative_path.as_str();
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Request {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::request::Kind::Evaluate(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::request::Alternative2 {
                                value: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Evaluate,
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
            <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::VERSION,
            <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
            3,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Stop");
                let path = alternative_path.as_str();
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Request {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::request::Kind::Stop(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::request::Alternative3 {
                                value: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Handle,
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
            <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::VERSION,
            <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
            4,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Close");
                let path = alternative_path.as_str();
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Request {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::request::Kind::Close(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::request::Alternative4 {
                                value: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Handle,
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
            <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                5,
            >>::VERSION,
            <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                5,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
            5,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Request {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::request::Kind::Shutdown(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::request::Alternative5 {
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
            <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                6,
            >>::VERSION,
            <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                6,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
            6,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "EvaluateBatch");
                let path = alternative_path.as_str();
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Request {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::request::Kind::EvaluateBatch(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::request::Alternative6 {
                                value: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::EvaluateBatch,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Request,
> for crate::scientific_worker::protocol::Request {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Request,
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
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::request::Kind::Prepare(
                wire,
            ) => {
                let alternative_path = child_path(path, "Prepare");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Prepare,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::request::Kind::Evaluate(
                wire,
            ) => {
                let alternative_path = child_path(path, "Evaluate");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Evaluate,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::request::Kind::Stop(
                wire,
            ) => {
                let alternative_path = child_path(path, "Stop");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Handle,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::request::Kind::Close(
                wire,
            ) => {
                let alternative_path = child_path(path, "Close");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Handle,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::request::Kind::Shutdown(
                wire,
            ) => {
                let alternative_path = child_path(path, "Shutdown");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::request::Kind::EvaluateBatch(
                wire,
            ) => {
                let alternative_path = child_path(path, "EvaluateBatch");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::EvaluateBatch,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <crate::scientific_worker::protocol::Request as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Response,
> for crate::scientific_worker::protocol::Response {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Response,
        AdapterError,
    > {
        let value = self;
        layout(
            <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Prepared");
                let path = alternative_path.as_str();
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Response {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::response::Kind::Prepared(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::response::Alternative1 {
                                value: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Prepared,
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
            <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Values");
                let path = alternative_path.as_str();
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Response {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::response::Kind::Values(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::response::Alternative2 {
                                value: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Values,
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
            <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::VERSION,
            <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
            3,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Failure");
                let path = alternative_path.as_str();
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Response {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::response::Kind::Failure(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::response::Alternative3 {
                                value: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Failure,
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
            <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::VERSION,
            <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
            4,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Closed");
                let path = alternative_path.as_str();
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Response {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::response::Kind::Closed(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::response::Alternative4 {
                                value: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Handle,
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
            <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                5,
            >>::VERSION,
            <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                5,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
            5,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Response {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::response::Kind::Shutdown(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::response::Alternative5 {
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
            <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                6,
            >>::VERSION,
            <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                6,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
            6,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Batch");
                let path = alternative_path.as_str();
                return Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Response {
                    kind: Some(
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::response::Kind::Batch(
                            Box::new(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::response::Alternative6 {
                                value: buffa::MessageField::some(
                                    native_into::<
                                        _,
                                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Batch,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Response,
> for crate::scientific_worker::protocol::Response {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Response,
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
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::response::Kind::Prepared(
                wire,
            ) => {
                let alternative_path = child_path(path, "Prepared");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Prepared,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::response::Kind::Values(
                wire,
            ) => {
                let alternative_path = child_path(path, "Values");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Values,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::response::Kind::Failure(
                wire,
            ) => {
                let alternative_path = child_path(path, "Failure");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Failure,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::response::Kind::Closed(
                wire,
            ) => {
                let alternative_path = child_path(path, "Closed");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Handle,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::response::Kind::Shutdown(
                wire,
            ) => {
                let alternative_path = child_path(path, "Shutdown");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::scientific_worker::generated::pb::pse::scientific_worker::v1::__buffa::oneof::response::Kind::Batch(
                wire,
            ) => {
                let alternative_path = child_path(path, "Batch");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::VERSION,
                    <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Batch,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <crate::scientific_worker::protocol::Response as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ScaleKind,
> for pse_quantity::enums::ScaleKind {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ScaleKind,
        AdapterError,
    > {
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ScaleKind {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ScaleKind,
> for pse_quantity::enums::ScaleKind {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::ScaleKind,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::SubjectRule,
> for pse_quantity::enums::SubjectRule {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::SubjectRule,
        AdapterError,
    > {
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::SubjectRule {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::SubjectRule,
> for pse_quantity::enums::SubjectRule {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::SubjectRule,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Tuple2OfOperationIdAndEntityKindId,
> for (pse_quantity::ids::OperationId, pse_quantity::ids::EntityKindId) {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Tuple2OfOperationIdAndEntityKindId,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Tuple2OfOperationIdAndEntityKindId {
            first: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::OperationId,
                >(field_0, &child_path(path, "first"))?,
            ),
            second: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::EntityKindId,
                >(field_1, &child_path(path, "second"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Tuple2OfOperationIdAndEntityKindId,
> for (pse_quantity::ids::OperationId, pse_quantity::ids::EntityKindId) {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Tuple2OfOperationIdAndEntityKindId,
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
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::OperationId,
            >(
                required(wire.first.into_option(), &child_path(path, "first"))?,
                &child_path(path, "first"),
            )?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::EntityKindId,
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
impl NativeInto<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitFactor,
> for pse_quantity::unit::UnitFactor {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitFactor,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitFactor {
            unit: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitId,
                >(field_0, &child_path(path, "unit"))?,
            ),
            exponent: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalRatio,
                >(field_1, &child_path(path, "exponent"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitFactor,
> for pse_quantity::unit::UnitFactor {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitFactor,
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
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitId,
            >(
                required(wire.unit.into_option(), &child_path(path, "unit"))?,
                &child_path(path, "unit"),
            )?,
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::PhysicalRatio,
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitId,
> for pse_quantity::ids::UnitId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitId,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitId {
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitId,
> for pse_quantity::ids::UnitId {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitId,
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
impl NativeInto<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitSetId,
> for pse_quantity::ids::UnitSetId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitSetId,
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
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitSetId {
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
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitSetId,
> for pse_quantity::ids::UnitSetId {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::UnitSetId,
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
impl NativeInto<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Values,
> for crate::scientific_worker::protocol::Values {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Values,
        AdapterError,
    > {
        layout(
            <crate::scientific_worker::protocol::Values as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::scientific_worker::protocol::Values as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "handle"),
                (2, "invocation"),
                (3, "values"),
                (4, "jacobian"),
                (5, "hessians"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Values {
            handle: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Handle,
                >(field_0, &child_path(path, "handle"))?,
            ),
            invocation: native_into::<_, u64>(field_1, &child_path(path, "invocation"))?,
            values: {
                let current_path = &child_path(path, "values");
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
            jacobian: {
                let current_path = &child_path(path, "jacobian");
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
            hessians: {
                let current_path = &child_path(path, "hessians");
                items(field_4)
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
impl NativeFrom<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Values,
> for crate::scientific_worker::protocol::Values {
    fn from_wire(
        wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Values,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::scientific_worker::protocol::Values as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::scientific_worker::protocol::Values as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "handle"),
                (2, "invocation"),
                (3, "values"),
                (4, "jacobian"),
                (5, "hessians"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Handle,
            >(
                required(wire.handle.into_option(), &child_path(path, "handle"))?,
                &child_path(path, "handle"),
            )?,
            native_from::<_, u64>(wire.invocation, &child_path(path, "invocation"))?,
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
            {
                let current_path = &child_path(path, "jacobian");
                let values = wire
                    .jacobian
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
                let current_path = &child_path(path, "hessians");
                let values = wire
                    .hessians
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
/// Encode the actual native `Request` and validate outgoing local rules.
pub(super) fn encode_request(
    native: crate::scientific_worker::protocol::Request,
) -> Result<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Request,
    BoundaryProjectionError,
> {
    encode_root(native, "Request")
}
/// Adopt known `Request` meaning under this package's declared unknown-field policy.
pub(super) fn decode_request(
    wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Request,
) -> Result<crate::scientific_worker::protocol::Request, BoundaryProjectionError> {
    decode_root(wire, "Request")
}
/// Encode the actual native `Response` and validate outgoing local rules.
pub(super) fn encode_response(
    native: crate::scientific_worker::protocol::Response,
) -> Result<
    crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Response,
    BoundaryProjectionError,
> {
    encode_root(native, "Response")
}
/// Adopt known `Response` meaning under this package's declared unknown-field policy.
pub(super) fn decode_response(
    wire: crate::scientific_worker::generated::pb::pse::scientific_worker::v1::Response,
) -> Result<crate::scientific_worker::protocol::Response, BoundaryProjectionError> {
    decode_root(wire, "Response")
}
