use super::{
    AdapterError, AdapterFailure, BoundaryProjectionError, NativeInto, NativeFrom,
    layout, child_path, refused, unknown_fields, native_into,
};
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::RecordedResultContext,
> for &crate::workflow::PackResultContext {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::RecordedResultContext,
        AdapterError,
    > {
        layout(
            <crate::workflow::PackResultContext as pse_contract::boundary_projection::ProjectionObservationRecord>::VERSION,
            <crate::workflow::PackResultContext as pse_contract::boundary_projection::ProjectionObservationRecord>::FIELD_LAYOUT,
            1,
            &[(1, "problem"), (2, "revision"), (3, "manifest"), (4, "interpretation")],
            path,
        )?;
        let (field_0, field_1, field_2, field_3) = pse_contract::boundary_projection::ProjectionObservationRecord::observation_fields(
                self,
            )
            .map_err(|error| error.at(path))?;
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::RecordedResultContext {
            problem: native_into::<_, String>(field_0, &child_path(path, "problem"))?,
            revision: native_into::<_, String>(field_1, &child_path(path, "revision"))?,
            manifest: native_into::<_, String>(field_2, &child_path(path, "manifest"))?,
            interpretation: native_into::<
                _,
                String,
            >(field_3, &child_path(path, "interpretation"))?,
            ..Default::default()
        })
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::RecordedResultPredicate,
> for &crate::workflow::PackResultPredicate {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::RecordedResultPredicate,
        AdapterError,
    > {
        layout(
            <crate::workflow::PackResultPredicate as pse_contract::boundary_projection::ProjectionObservationRecord>::VERSION,
            <crate::workflow::PackResultPredicate as pse_contract::boundary_projection::ProjectionObservationRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "output"),
                (2, "partition"),
                (3, "minimum_bits"),
                (4, "maximum_bits"),
                (5, "missing"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4) = pse_contract::boundary_projection::ProjectionObservationRecord::observation_fields(
                self,
            )
            .map_err(|error| error.at(path))?;
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::RecordedResultPredicate {
            output: native_into::<_, String>(field_0, &child_path(path, "output"))?,
            partition: native_into::<
                _,
                String,
            >(field_1, &child_path(path, "partition"))?,
            minimum_bits: field_2
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<_, u64>(value, &child_path(path, "minimum_bits"))
                })
                .transpose()?,
            maximum_bits: field_3
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<_, u64>(value, &child_path(path, "maximum_bits"))
                })
                .transpose()?,
            missing: field_4
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<_, bool>(value, &child_path(path, "missing"))
                })
                .transpose()?,
            ..Default::default()
        })
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::RecordedResultSelection,
> for &crate::workflow::PackResultSelection {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::RecordedResultSelection,
        AdapterError,
    > {
        layout(
            <crate::workflow::PackResultSelection as pse_contract::boundary_projection::ProjectionObservationRecord>::VERSION,
            <crate::workflow::PackResultSelection as pse_contract::boundary_projection::ProjectionObservationRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "run"),
                (2, "attempt"),
                (3, "relation"),
                (4, "schema_version"),
                (5, "row_start"),
                (6, "row_end"),
                (7, "field"),
                (8, "terminal_class"),
                (9, "predicate"),
                (10, "usability"),
                (11, "context"),
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
        ) = pse_contract::boundary_projection::ProjectionObservationRecord::observation_fields(
                self,
            )
            .map_err(|error| error.at(path))?;
        Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::RecordedResultSelection {
            run: native_into::<_, String>(field_0, &child_path(path, "run"))?,
            attempt: native_into::<_, String>(field_1, &child_path(path, "attempt"))?,
            relation: native_into::<_, String>(field_2, &child_path(path, "relation"))?,
            schema_version: native_into::<
                _,
                u32,
            >(field_3, &child_path(path, "schema_version"))?,
            row_start: native_into::<_, u64>(field_4, &child_path(path, "row_start"))?,
            row_end: native_into::<_, u64>(field_5, &child_path(path, "row_end"))?,
            field: field_6
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<_, String>(value, &child_path(path, "field"))
                })
                .transpose()?,
            terminal_class: native_into::<
                _,
                String,
            >(field_7, &child_path(path, "terminal_class"))?,
            predicate: field_8
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<
                        _,
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::RecordedResultPredicate,
                    >(
                        ::core::borrow::Borrow::<
                            crate::workflow::PackResultPredicate,
                        >::borrow(&value),
                        &child_path(path, "predicate"),
                    )
                })
                .transpose()?
                .into(),
            usability: buffa::MessageField::some(
                native_into::<
                    _,
                    crate::host_operations::engineering::generated::pb::pse::engineering::v1::RecordedScientificUsability,
                >(field_9, &child_path(path, "usability"))?,
            ),
            context: field_10
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<
                        _,
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::RecordedResultContext,
                    >(
                        ::core::borrow::Borrow::<
                            crate::workflow::PackResultContext,
                        >::borrow(&value),
                        &child_path(path, "context"),
                    )
                })
                .transpose()?
                .into(),
            ..Default::default()
        })
    }
}
impl NativeInto<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::RecordedScientificUsability,
> for crate::workflow::PackScientificState {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::generated::pb::pse::engineering::v1::RecordedScientificUsability,
        AdapterError,
    > {
        let value = self;
        layout(
            <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::RecordedScientificUsability {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::recorded_scientific_usability::Kind::Unknown(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::recorded_scientific_usability::Alternative1 {
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
            <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::RecordedScientificUsability {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::recorded_scientific_usability::Kind::Usable(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::recorded_scientific_usability::Alternative2 {
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
            <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::VERSION,
            <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
            3,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::RecordedScientificUsability {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::recorded_scientific_usability::Kind::Partial(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::recorded_scientific_usability::Alternative3 {
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
            <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::VERSION,
            <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
            4,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::RecordedScientificUsability {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::recorded_scientific_usability::Kind::Failed(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::recorded_scientific_usability::Alternative4 {
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
            <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
                5,
            >>::VERSION,
            <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
                5,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
            5,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::host_operations::engineering::generated::pb::pse::engineering::v1::RecordedScientificUsability {
                    kind: Some(
                        crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::recorded_scientific_usability::Kind::Unusable(
                            Box::new(crate::host_operations::engineering::generated::pb::pse::engineering::v1::recorded_scientific_usability::Alternative5 {
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
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::RecordedScientificUsability,
> for crate::workflow::PackScientificState {
    fn from_wire(
        wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::RecordedScientificUsability,
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
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::recorded_scientific_usability::Kind::Unknown(
                wire,
            ) => {
                let alternative_path = child_path(path, "Unknown");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::recorded_scientific_usability::Kind::Usable(
                wire,
            ) => {
                let alternative_path = child_path(path, "Usable");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::recorded_scientific_usability::Kind::Partial(
                wire,
            ) => {
                let alternative_path = child_path(path, "Partial");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::recorded_scientific_usability::Kind::Failed(
                wire,
            ) => {
                let alternative_path = child_path(path, "Failed");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::generated::pb::pse::engineering::v1::__buffa::oneof::recorded_scientific_usability::Kind::Unusable(
                wire,
            ) => {
                let alternative_path = child_path(path, "Unusable");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::VERSION,
                    <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <crate::workflow::PackScientificState as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
/// Observe `RecordedResultSelection` without consuming native custody or constructing native authority.
pub(crate) fn encode_recorded_result_selection(
    native: &crate::workflow::PackResultSelection,
) -> Result<
    crate::host_operations::engineering::generated::pb::pse::engineering::v1::RecordedResultSelection,
    BoundaryProjectionError,
> {
    pse_contract::boundary_projection::ValidateProjection::validate_projection(native)
        .map_err(BoundaryProjectionError::OutgoingNative)?;
    let wire: crate::host_operations::engineering::generated::pb::pse::engineering::v1::RecordedResultSelection = native_into(
            native,
            "RecordedResultSelection",
        )
        .map_err(BoundaryProjectionError::OutgoingAdoption)?;
    protovalidate_buffa::Validate::validate(&wire)
        .map_err(BoundaryProjectionError::OutgoingWire)?;
    Ok(wire)
}
